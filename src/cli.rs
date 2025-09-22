use clap::{Parser, Subcommand};

use crate::commands::remap::RemapArgs;

#[derive(Parser)]
#[command(name = "rust-utils")]
#[command(
    about = "A collection of Rust utilities for system administration and development workflows"
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Remap UID/GID ranges in LXC filesystem
    Remap(RemapArgs),
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_cli_parsing_remap_basic() {
        let args = vec![
            "rust-utils",
            "remap",
            "/test/path",
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Remap(remap_args) => {
                assert_eq!(remap_args.base_directory, PathBuf::from("/test/path"));
                assert_eq!(remap_args.from_base, 100000);
                assert_eq!(remap_args.to_base, 50000000);
                assert_eq!(remap_args.range_size, 65536); // default
                assert!(!remap_args.dry_run);
                assert!(!remap_args.verbose);
                assert!(!remap_args.uid_only);
                assert!(!remap_args.gid_only);
                assert!(remap_args.exclude.is_empty());
            }
        }
    }

    #[test]
    fn test_cli_parsing_remap_all_options() {
        let args = vec![
            "rust-utils",
            "remap",
            "/test/path",
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
            "--range-size",
            "32768",
            "--dry-run",
            "--verbose",
            "--uid-only",
            "--exclude",
            "*.log",
            "--exclude",
            "tmp/*",
        ];

        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Remap(remap_args) => {
                assert_eq!(remap_args.base_directory, PathBuf::from("/test/path"));
                assert_eq!(remap_args.from_base, 100000);
                assert_eq!(remap_args.to_base, 50000000);
                assert_eq!(remap_args.range_size, 32768);
                assert!(remap_args.dry_run);
                assert!(remap_args.verbose);
                assert!(remap_args.uid_only);
                assert!(!remap_args.gid_only);
                assert_eq!(remap_args.exclude, vec!["*.log", "tmp/*"]);
            }
        }
    }

    #[test]
    fn test_cli_parsing_missing_required_args() {
        let args = vec![
            "rust-utils",
            "remap",
            "/test/path",
            // Missing --from-base and --to-base
        ];

        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_parsing_invalid_range_size() {
        let args = vec![
            "rust-utils",
            "remap",
            "/test/path",
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
            "--range-size",
            "invalid",
        ];

        let result = Cli::try_parse_from(args);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_help() {
        let args = vec!["rust-utils", "--help"];
        let result = Cli::try_parse_from(args);
        // Should fail with help message, not an error
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_version() {
        let args = vec!["rust-utils", "--version"];
        let result = Cli::try_parse_from(args);
        // Should fail with version message, not an error
        assert!(result.is_err());
    }

    #[test]
    fn test_remap_help() {
        let args = vec!["rust-utils", "remap", "--help"];
        let result = Cli::try_parse_from(args);
        // Should fail with help message, not an error
        assert!(result.is_err());
    }
}
