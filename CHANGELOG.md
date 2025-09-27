# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2024-12-19

### Fixed
- Fixed symbolic link ownership handling in `remap` command
  - Changed from `nix::unistd::chown` to `std::os::unix::fs::lchown` to properly handle symbolic links
  - Symbolic links now have their ownership changed instead of following the link to the target
- Improved error handling for permission-denied scenarios in non-root environments
- Enhanced test coverage for ownership change operations

### Changed
- Improved test suite with comprehensive privilege detection
- Enhanced documentation for root-required vs capability-based operations
- Better error messages for ownership change failures

### Added
- Comprehensive test coverage for various file types (regular files, directories, symbolic links)
- Permission detection tests for different privilege scenarios (root, CAP_CHOWN, user namespaces)
- Enhanced logging for ownership change operations

## [0.1.0] - 2024-12-18

### Added
- Initial release of `rust-utils` CLI toolkit
- `remap` command for UID/GID remapping in LXC container filesystems
- Support for dry-run mode to preview changes safely
- Pattern-based file exclusion with glob-like syntax
- Hard link detection to avoid duplicate processing
- Comprehensive error handling with structured error types
- Full test suite with >60% coverage
- Performance benchmarking with criterion
- Documentation and usage examples

### Features
- **UID/GID Remapping**: Efficiently remap ownership across directory trees
- **Pattern Exclusions**: Flexible file filtering with `--exclude` patterns
- **Hard Link Detection**: Intelligent handling to process files only once
- **Dry Run Support**: Safe testing mode for all operations
- **Verbose Output**: Detailed logging for operations and progress
- **Range Validation**: Overflow protection for UID/GID ranges
- **Cross-Platform**: Unix/Linux filesystem operations

[0.1.1]: https://github.com/yourusername/rust-utils/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/yourusername/rust-utils/releases/tag/v0.1.0