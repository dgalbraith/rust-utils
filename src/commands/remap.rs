use std::collections::HashMap;
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
use nix::unistd::{chown, Gid, Uid};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::error::{Result as RustUtilsResult, RustUtilsError};
use crate::fs::{get_file_metadata, should_exclude};

#[derive(Args)]
pub struct RemapArgs {
    /// Base directory path to remap (e.g., /var/lib/lxc/container/rootfs)
    pub base_directory: PathBuf,

    /// Source UID/GID base range (e.g., 100000)
    #[arg(long)]
    pub from_base: u32,

    /// Target UID/GID base range (e.g., 50000000)
    #[arg(long)]
    pub to_base: u32,

    /// Size of the ID range to remap
    #[arg(long, default_value = "65536")]
    pub range_size: u32,

    /// Show what would be changed without making modifications
    #[arg(long)]
    pub dry_run: bool,

    /// Show detailed output for each file processed
    #[arg(long)]
    pub verbose: bool,

    /// Exclude paths matching pattern (can be used multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Only remap UIDs, leave GIDs unchanged
    #[arg(long)]
    pub uid_only: bool,

    /// Only remap GIDs, leave UIDs unchanged
    #[arg(long)]
    pub gid_only: bool,
}

pub struct RemapCommand {
    args: RemapArgs,
    seen_inodes: HashMap<(u64, u64), PathBuf>, // (device, inode) -> first path
}

impl RemapCommand {
    pub fn new(args: RemapArgs) -> Self {
        Self {
            args,
            seen_inodes: HashMap::new(),
        }
    }

    pub fn execute(mut self) -> Result<()> {
        self.validate_args()?;

        if !self.args.base_directory.exists() {
            return Err(RustUtilsError::DirectoryNotFound(
                self.args.base_directory.display().to_string(),
            )
            .into());
        }

        if !self.args.base_directory.is_dir() {
            return Err(RustUtilsError::DirectoryNotFound(format!(
                "{} is not a directory",
                self.args.base_directory.display()
            ))
            .into());
        }

        if self.args.dry_run {
            info!("DRY RUN MODE - No changes will be made");
        }

        info!("Starting UID/GID remapping");
        info!("Base directory: {}", self.args.base_directory.display());
        info!(
            "From range: {}-{}",
            self.args.from_base,
            self.args.from_base + self.args.range_size - 1
        );
        info!(
            "To range: {}-{}",
            self.args.to_base,
            self.args.to_base + self.args.range_size - 1
        );

        let mut files_processed = 0;
        let mut files_remapped = 0;

        // Collect paths first to avoid borrowing issues
        let entries: Result<Vec<_>, _> = WalkDir::new(&self.args.base_directory)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !should_exclude(e.path(), &self.args.exclude))
            .collect();

        for entry in entries? {
            let path = entry.path();

            files_processed += 1;

            if let Err(e) = self.process_file(path) {
                warn!("Failed to process {}: {}", path.display(), e);
                continue;
            }

            if self.should_remap_file(path)? {
                files_remapped += 1;
            }

            if self.args.verbose && files_processed % 1000 == 0 {
                info!(
                    "Processed {} files, remapped {}",
                    files_processed, files_remapped
                );
            }
        }

        info!("Remapping completed");
        info!("Files processed: {}", files_processed);
        info!("Files remapped: {}", files_remapped);

        Ok(())
    }

    fn validate_args(&self) -> RustUtilsResult<()> {
        if self.args.from_base >= u32::MAX - self.args.range_size {
            return Err(RustUtilsError::InvalidRange(
                "from_base + range_size would overflow".to_string(),
            ));
        }

        if self.args.to_base >= u32::MAX - self.args.range_size {
            return Err(RustUtilsError::InvalidRange(
                "to_base + range_size would overflow".to_string(),
            ));
        }

        if self.args.uid_only && self.args.gid_only {
            return Err(RustUtilsError::InvalidRange(
                "Cannot specify both --uid-only and --gid-only".to_string(),
            ));
        }

        Ok(())
    }

    fn process_file(&mut self, path: &Path) -> RustUtilsResult<()> {
        let metadata = get_file_metadata(path)?;

        // Check for hard links
        if metadata.nlink() > 1 {
            let key = (metadata.dev(), metadata.ino());
            if let Some(first_path) = self.seen_inodes.get(&key) {
                debug!(
                    "Skipping hard link: {} -> {}",
                    path.display(),
                    first_path.display()
                );
                return Ok(());
            }
            self.seen_inodes.insert(key, path.to_path_buf());
        }

        if self.should_remap_file(path)? {
            self.remap_file(path, &metadata)?;
        }

        Ok(())
    }

    fn should_remap_file(&self, path: &Path) -> RustUtilsResult<bool> {
        let metadata = get_file_metadata(path)?;
        let uid = metadata.uid();
        let gid = metadata.gid();

        let uid_in_range =
            uid >= self.args.from_base && uid < self.args.from_base + self.args.range_size;
        let gid_in_range =
            gid >= self.args.from_base && gid < self.args.from_base + self.args.range_size;

        let should_remap = match (self.args.uid_only, self.args.gid_only) {
            (true, false) => uid_in_range,
            (false, true) => gid_in_range,
            (false, false) => uid_in_range || gid_in_range,
            (true, true) => unreachable!(), // Validated in validate_args
        };

        Ok(should_remap)
    }

    fn remap_file(&self, path: &Path, metadata: &Metadata) -> RustUtilsResult<()> {
        let current_uid = metadata.uid();
        let current_gid = metadata.gid();

        let new_uid = if self.args.gid_only {
            current_uid
        } else if current_uid >= self.args.from_base
            && current_uid < self.args.from_base + self.args.range_size
        {
            let offset = current_uid - self.args.from_base;
            self.args.to_base + offset
        } else {
            current_uid
        };

        let new_gid = if self.args.uid_only {
            current_gid
        } else if current_gid >= self.args.from_base
            && current_gid < self.args.from_base + self.args.range_size
        {
            let offset = current_gid - self.args.from_base;
            self.args.to_base + offset
        } else {
            current_gid
        };

        if (self.args.verbose || self.args.dry_run)
            && (new_uid != current_uid || new_gid != current_gid)
        {
            info!(
                "{}: {}:{} -> {}:{}{}",
                path.display(),
                current_uid,
                current_gid,
                new_uid,
                new_gid,
                if self.args.dry_run { " (dry run)" } else { "" }
            );
        }

        if !self.args.dry_run && (new_uid != current_uid || new_gid != current_gid) {
            let uid = if new_uid != current_uid {
                Some(Uid::from_raw(new_uid))
            } else {
                None
            };

            let gid = if new_gid != current_gid {
                Some(Gid::from_raw(new_gid))
            } else {
                None
            };

            chown(path, uid, gid).map_err(|e| {
                RustUtilsError::RemapFailed(format!("Failed to chown {}: {}", path.display(), e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::os::unix::fs::{chown, MetadataExt};
    use tempfile::TempDir;

    #[test]
    fn test_remap_args_validation() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/tmp"),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: false,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        assert!(command.validate_args().is_ok());
    }

    #[test]
    fn test_remap_args_validation_overflow() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/tmp"),
            from_base: u32::MAX - 1000,
            to_base: 50000000,
            range_size: 65536, // This would overflow
            dry_run: false,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        assert!(command.validate_args().is_err());
    }

    #[test]
    fn test_remap_args_validation_both_uid_gid_only() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/tmp"),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: false,
            verbose: false,
            exclude: vec![],
            uid_only: true,
            gid_only: true, // Both flags set - should error
        };

        let command = RemapCommand::new(args);
        assert!(command.validate_args().is_err());
    }

    #[test]
    fn test_should_remap_file_uid_in_range() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        // Change file ownership to be in the range we want to remap
        // Note: This test requires running as root or with appropriate capabilities
        // For CI/testing, we'll skip if we can't change ownership
        if chown(&file_path, Some(100000), Some(100000)).is_err() {
            // Skip test if we can't change ownership (not root)
            return Ok(());
        }

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(should_remap);

        Ok(())
    }

    #[test]
    fn test_should_remap_file_uid_only_flag() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        // Skip if we can't change ownership
        if chown(&file_path, Some(100000), Some(50000)).is_err() {
            return Ok(());
        }

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: true, // Only check UIDs
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(should_remap); // UID is in range

        Ok(())
    }

    #[test]
    fn test_should_remap_file_out_of_range() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        // File will have default ownership (likely 0 or current user)
        // which should be outside our test range

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        // Should not remap files outside the range
        assert!(!should_remap);

        Ok(())
    }

    #[test]
    fn test_execute_nonexistent_directory() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/nonexistent/directory"),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_file_instead_of_directory(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        let args = RemapArgs {
            base_directory: file_path, // File instead of directory
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn test_execute_dry_run() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create a test file structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir)?;

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");
        File::create(&file1)?;
        File::create(&file2)?;

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true, // Dry run should not fail
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();

        // Dry run should succeed even if we don't have permission to change ownership
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_execute_with_exclusions() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create test files
        let log_file = temp_dir.path().join("app.log");
        let tmp_dir = temp_dir.path().join("tmp");
        fs::create_dir(&tmp_dir)?;
        let tmp_file = tmp_dir.join("temp.txt");
        let regular_file = temp_dir.path().join("data.txt");

        File::create(&log_file)?;
        File::create(&tmp_file)?;
        File::create(&regular_file)?;

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: true,
            exclude: vec!["*.log".to_string(), "tmp/*".to_string()],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_ok());

        Ok(())
    }

    #[test]
    fn test_hard_link_detection() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        // Create original file
        File::create(&file1)?;

        // Create hard link
        fs::hard_link(&file1, &file2)?;

        let mut command = RemapCommand::new(RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: true,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        });

        // Process first file
        let result1 = command.process_file(&file1);
        assert!(result1.is_ok());

        // Process hard link - should be skipped
        let result2 = command.process_file(&file2);
        assert!(result2.is_ok());

        // Verify that the hard link was tracked
        let metadata = get_file_metadata(&file1)?;
        let key = (metadata.dev(), metadata.ino());
        assert!(command.seen_inodes.contains_key(&key));

        Ok(())
    }
}
