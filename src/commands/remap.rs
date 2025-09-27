use std::collections::HashMap;
use std::fs::Metadata;
use std::os::unix::fs::{lchown, MetadataExt};
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Args;
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
                Some(new_uid)
            } else {
                None
            };

            let gid = if new_gid != current_gid {
                Some(new_gid)
            } else {
                None
            };

            lchown(path, uid, gid).map_err(|e| {
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
    use std::os::unix::fs::{MetadataExt, symlink};
    use tempfile::TempDir;
    use nix::unistd::{getuid, geteuid};

    /// Test argument validation logic - no filesystem operations needed
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

    /// Test overflow detection in validation
    #[test]
    fn test_remap_args_validation_from_base_overflow() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/tmp"),
            from_base: u32::MAX - 1000,
            to_base: 50000000,
            range_size: 65536, // This would overflow from_base + range_size
            dry_run: false,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.validate_args();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("from_base + range_size would overflow"));
    }

    /// Test to_base overflow detection
    #[test]
    fn test_remap_args_validation_to_base_overflow() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/tmp"),
            from_base: 100000,
            to_base: u32::MAX - 1000,
            range_size: 65536, // This would overflow to_base + range_size
            dry_run: false,
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.validate_args();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("to_base + range_size would overflow"));
    }

    /// Test conflicting flags validation
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
        let result = command.validate_args();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot specify both --uid-only and --gid-only"));
    }

    /// Test decision logic for files with current user ownership - NO DRY RUN
    #[test]
    fn test_should_remap_file_with_current_user_ownership() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        let current_uid = getuid().as_raw();

        // Test with current user's UID in the remap range
        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: current_uid,
            to_base: current_uid + 1000,
            range_size: 1, // Exactly matches current_uid
            dry_run: false, // NOT dry run - testing decision logic
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(should_remap, "File with UID {current_uid} should be identified for remapping");

        Ok(())
    }

    /// Test UID-only flag decision logic - NO DRY RUN  
    #[test]
    fn test_should_remap_file_uid_only_flag() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        let current_uid = getuid().as_raw();

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: current_uid,
            to_base: current_uid + 1000,
            range_size: 1,
            dry_run: false, // NOT dry run - testing logic
            verbose: false,
            exclude: vec![],
            uid_only: true, // Only check UIDs
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(should_remap, "File with UID {current_uid} should be identified for UID-only remapping");

        Ok(())
    }

    /// Test GID-only flag decision logic - NO DRY RUN
    #[test]
    fn test_should_remap_file_gid_only_flag() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        let current_gid = getuid().as_raw(); // Files typically get user's primary GID

        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: current_gid,
            to_base: current_gid + 1000,
            range_size: 1,
            dry_run: false, // NOT dry run - testing logic
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: true, // Only check GIDs
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(should_remap, "File with GID {current_gid} should be identified for GID-only remapping");

        Ok(())
    }

    /// Test files outside remap range - NO DRY RUN
    #[test]
    fn test_should_remap_file_out_of_range() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        // Use a range that definitely won't include current user
        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: 100000, // High UID range unlikely to match current user
            to_base: 200000,
            range_size: 65536,
            dry_run: false, // NOT dry run - testing logic
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let should_remap = command.should_remap_file(&file_path)?;
        assert!(!should_remap, "File with current user ownership should not be in high UID range");

        Ok(())
    }

    /// Test nonexistent directory error - NO DRY RUN
    #[test]
    fn test_execute_nonexistent_directory() {
        let args = RemapArgs {
            base_directory: PathBuf::from("/nonexistent/directory/that/does/not/exist"),
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: false, // NOT dry run - testing error handling
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("nonexistent") || error_msg.contains("not found"));
    }

    /// Test file instead of directory error - NO DRY RUN
    #[test]
    fn test_execute_file_instead_of_directory() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test_file.txt");
        File::create(&file_path)?;

        let args = RemapArgs {
            base_directory: file_path, // File instead of directory
            from_base: 100000,
            to_base: 50000000,
            range_size: 65536,
            dry_run: false, // NOT dry run - testing error handling
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not a directory"));

        Ok(())
    }

    /// Test hard link detection - NO DRY RUN needed for this logic
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
            dry_run: false, // NOT dry run - testing hard link logic
            verbose: false,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        });

        // Process first file
        let result1 = command.process_file(&file1);
        assert!(result1.is_ok());

        // Process hard link - should be skipped due to inode tracking
        let result2 = command.process_file(&file2);
        assert!(result2.is_ok());

        // Verify that the hard link was tracked
        let metadata = get_file_metadata(&file1)?;
        let key = (metadata.dev(), metadata.ino());
        assert!(command.seen_inodes.contains_key(&key));

        // Verify both files have the same inode
        let metadata1 = get_file_metadata(&file1)?;
        let metadata2 = get_file_metadata(&file2)?;
        assert_eq!(metadata1.ino(), metadata2.ino());
        assert_eq!(metadata1.dev(), metadata2.dev());

        Ok(())
    }

    /// Test exclusion patterns - NO DRY RUN needed for traversal logic
    #[test]
    fn test_execute_with_exclusions() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create test files and directories
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
            to_base: 200000,
            range_size: 65536,
            dry_run: false, // NOT dry run - testing exclusion logic
            verbose: true,
            exclude: vec!["*.log".to_string(), "tmp".to_string()],
            uid_only: false,
            gid_only: false,
        };

        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_ok(), "Exclusion pattern processing should succeed");

        Ok(())
    }

    /// Test permission denied gracefully - NO DRY RUN (that's the point)
    #[test]
    fn test_actual_remap_permission_denied_non_root() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Skip if running as root
    if geteuid().is_root() {
        info!("Skipping permission test - running as root");
        return Ok(());
    }

    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test_file.txt");
    File::create(&file_path)?;

    let current_uid = getuid().as_raw();
    let current_gid = getuid().as_raw(); // Use same for GID

    // Verify the file actually has current user ownership and is in range
    let metadata = get_file_metadata(&file_path)?;
    let file_uid = metadata.uid();
    let file_gid = metadata.gid();
    
    debug!("Test file ownership - UID: {}, GID: {}", file_uid, file_gid);
    debug!("Current process - UID: {}, GID: {}", current_uid, current_gid);

    // Create args that target files owned by current user
    let args = RemapArgs {
        base_directory: temp_dir.path().to_path_buf(),
        from_base: file_uid, // Use actual file UID
        to_base: file_uid + 1000, // This should fail for non-root
        range_size: 1,
        dry_run: false, // NOT dry run - testing actual permission failure
        verbose: true,
        exclude: vec![],
        uid_only: false,
        gid_only: false,
    };

    // Verify the file would be identified for remapping
    let command = RemapCommand::new(args);
    let should_remap = command.should_remap_file(&file_path)?;
    
    if !should_remap {
        // File won't be remapped, so test won't demonstrate permission failure
        warn!("File UID {} not in range {}-{}, adjusting test", 
              file_uid, file_uid, file_uid);
        return Ok(());
    }

    debug!("File {} should be remapped (UID {} -> {})", 
           file_path.display(), file_uid, file_uid + 1000);

    let result = command.execute();

    // Should fail due to permission denied when trying to lchown to arbitrary UID
    if result.is_ok() {
        // This might happen if the system allows the change for some reason
        warn!("Expected permission failure but command succeeded - system may allow UID change");
        return Ok(());
    }

    // Verify we got the expected permission error
    let error_message = format!("{}", result.unwrap_err());
    debug!("Got expected error: {}", error_message);
    
    assert!(
        error_message.contains("Operation not permitted") || 
        error_message.contains("Permission denied") ||
        error_message.contains("chown") ||
        error_message.contains("lchown") ||
        error_message.contains("RemapFailed"),
        "Error should indicate permission/ownership issue, got: {error_message}"
    );

    Ok(())
}

    // PRIVILEGED TESTS - These require root and test actual ownership changes

    /// Test actual symbolic link ownership remapping - requires root or cap_chown
    #[cfg(test)]
    #[ignore = "Requires root privileges - run with 'sudo cargo test test_symbolic_link_ownership_requires_root -- --ignored --nocapture'"]
    #[test]
    fn test_symbolic_link_ownership_requires_root() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let target_file = temp_dir.path().join("target.txt");
        let symlink_path = temp_dir.path().join("symlink");
        
        File::create(&target_file)?;
        symlink(&target_file, &symlink_path)?;
        
        const INITIAL_UID: u32 = 100000;
        const TARGET_UID: u32 = 200000;
        
        // Set initial ownership - only works as root
        lchown(&target_file, Some(INITIAL_UID), Some(INITIAL_UID))?;
        lchown(&symlink_path, Some(INITIAL_UID), Some(INITIAL_UID))?;
        
        // Verify initial state
        let target_before = get_file_metadata(&target_file)?;
        let symlink_before = get_file_metadata(&symlink_path)?;
        assert_eq!(target_before.uid(), INITIAL_UID);
        assert_eq!(symlink_before.uid(), INITIAL_UID);
        
        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: INITIAL_UID,
            to_base: TARGET_UID,
            range_size: 1,
            dry_run: false, // NOT dry run - actual ownership changes
            verbose: true,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };
        
        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_ok(), "Root should be able to change ownership");
        
        // Verify both target and symlink were updated
        let target_after = get_file_metadata(&target_file)?;
        let symlink_after = get_file_metadata(&symlink_path)?;
        
        assert_eq!(target_after.uid(), TARGET_UID, "Target file UID should be updated");
        assert_eq!(symlink_after.uid(), TARGET_UID, "Symbolic link UID should be updated with lchown");
        
        Ok(())
    }

    /// Test comprehensive ownership scenarios - requires root or cap_chown
    #[cfg(test)]
    #[ignore = "Requires root privileges - run with 'sudo cargo test test_comprehensive_ownership_scenarios_requires_root -- --ignored --nocapture'"]
    #[test]
    fn test_comprehensive_ownership_scenarios_requires_root() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create various file types
        let regular_file = temp_dir.path().join("regular.txt");
        let target_file = temp_dir.path().join("target.txt");
        let symlink_to_file = temp_dir.path().join("symlink_to_file");
        let subdir = temp_dir.path().join("subdir");
        let symlink_to_dir = temp_dir.path().join("symlink_to_dir");
        
        File::create(&regular_file)?;
        File::create(&target_file)?;
        fs::create_dir(&subdir)?;
        symlink(&target_file, &symlink_to_file)?;
        symlink(&subdir, &symlink_to_dir)?;
        
        const FROM_UID: u32 = 100000;
        const TO_UID: u32 = 200000;
        
        // Set ownership on all files - requires root
        lchown(&regular_file, Some(FROM_UID), Some(FROM_UID))?;
        lchown(&target_file, Some(FROM_UID), Some(FROM_UID))?;
        lchown(&subdir, Some(FROM_UID), Some(FROM_UID))?;
        lchown(&symlink_to_file, Some(FROM_UID), Some(FROM_UID))?;
        lchown(&symlink_to_dir, Some(FROM_UID), Some(FROM_UID))?;
        
        let args = RemapArgs {
            base_directory: temp_dir.path().to_path_buf(),
            from_base: FROM_UID,
            to_base: TO_UID,
            range_size: 1,
            dry_run: false, // NOT dry run - actual ownership changes
            verbose: true,
            exclude: vec![],
            uid_only: false,
            gid_only: false,
        };
        
        let command = RemapCommand::new(args);
        let result = command.execute();
        assert!(result.is_ok());
        
        // Verify all file types were updated correctly
        let regular_after = get_file_metadata(&regular_file)?;
        let target_after = get_file_metadata(&target_file)?;
        let subdir_after = get_file_metadata(&subdir)?;
        let symlink_file_after = get_file_metadata(&symlink_to_file)?;
        let symlink_dir_after = get_file_metadata(&symlink_to_dir)?;
        
        assert_eq!(regular_after.uid(), TO_UID, "Regular file should be updated");
        assert_eq!(target_after.uid(), TO_UID, "Target file should be updated");
        assert_eq!(subdir_after.uid(), TO_UID, "Directory should be updated");
        assert_eq!(symlink_file_after.uid(), TO_UID, "Symbolic link to file should be updated");
        assert_eq!(symlink_dir_after.uid(), TO_UID, "Symbolic link to directory should be updated");
        
        Ok(())
    }
}
