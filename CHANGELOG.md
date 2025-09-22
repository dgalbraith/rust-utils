# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-09-21

### Added
- **Core UID/GID remapping functionality** (`remap` command)
  - Support for remapping user and group IDs across filesystem hierarchies
  - Configurable source and target ID ranges with 65536 default range size
  - Dry run mode for previewing changes without execution
  - Pattern-based file exclusion with glob-style matching
  - Hard link detection and tracking to prevent corruption
  - Verbose logging with structured output using tracing

- **Robust CLI interface** with clap derive macros
  - Comprehensive argument validation and error handling
  - Help system with detailed command documentation
  - Support for both UID-only and GID-only remapping modes
  - Multiple exclusion patterns support

- **Comprehensive error handling**
  - Custom error types with thiserror for structured errors
  - Graceful handling of permission errors and filesystem issues
  - Detailed error messages with context using anyhow
  - Overflow protection for numeric range calculations

- **Extensive testing infrastructure** (43 tests total)
  - 29 unit tests covering all modules and functions
  - 14 integration tests for end-to-end CLI validation
  - Test coverage reporting with HTML output
  - Temporary filesystem operations for safe testing
  - Mock scenarios for edge cases and error conditions

- **Documentation**
  - Complete README with installation and usage examples
  - Detailed command reference in `docs/remap.md`
  - Real-world usage examples in `docs/examples.md`
  - Comprehensive testing guide in `docs/TESTING.md`
  - Contributing guidelines in `CONTRIBUTING.md`

- **Development infrastructure**
  - Structured logging with tracing and tracing-subscriber
  - Performance benchmarking with criterion
  - Code quality enforcement with clippy integration
  - Automated formatting with rustfmt
  - Cross-platform Unix support using nix crate

- **Security and reliability features**
  - Memory-safe filesystem operations
  - Atomic operations where possible
  - Input validation and sanitization
  - Safe handling of symbolic links and special files

### Technical Details
- **Language**: Rust 2021 Edition
- **Minimum Rust Version**: 1.70+
- **Dependencies**:
  - `clap` 4.4+ for CLI argument parsing
  - `anyhow` 1.0+ for error handling
  - `thiserror` 1.0+ for custom error types
  - `walkdir` 2.4+ for filesystem traversal
  - `nix` 0.27+ for Unix system operations
  - `tracing` 0.1+ for structured logging
- **Dev Dependencies**:
  - `tempfile` 3.8+ for test filesystem operations
  - `assert_cmd` 2.0+ for CLI integration testing
  - `predicates` 3.0+ for test assertions
  - `criterion` 0.5+ for benchmarking

### Package Information
- **License**: MIT
- **Repository**: https://github.com/dgalbraith/rust-utils
- **Categories**: command-line-utilities, development-tools
- **Keywords**: utilities, system, admin, tools, workflow

### Initial Use Cases
- **Container Migration**: Remap filesystem ownership during LXC/Docker migrations
- **User Namespace Changes**: Adjust file ownership for user namespace configurations
- **System Administration**: Bulk ownership changes with exclusion patterns
- **Development Workflows**: Test environment setup with proper file permissions

### Quality Metrics
- **Test Coverage**: >60% line coverage across all modules
- **Performance**: Optimized release binary ~1.4MB
- **Code Quality**: Zero clippy warnings, formatted with rustfmt
- **Documentation**: Comprehensive guides and examples

---

## Release Notes

### Version 0.1.0 - Initial Release

This is the inaugural release of rust-utils, introducing a production-ready UID/GID remapping utility built with Rust's performance and safety guarantees.

**Key Highlights:**
- 🚀 **High Performance**: Zero-cost abstractions for maximum speed
- 🔒 **Memory Safety**: No segfaults or buffer overflows
- 🔧 **Cross-Platform**: Works on all Unix-like systems
- ⚡ **Extensible**: Clean architecture for adding new utilities
- 🧪 **Well-Tested**: 43 comprehensive tests with >60% coverage
- 📚 **Documented**: Professional documentation and examples

**Perfect for:**
- System administrators managing container migrations
- DevOps engineers handling user namespace configurations
- Developers needing reliable filesystem ownership tools
- Anyone seeking memory-safe alternatives to traditional Unix tools

**Getting Started:**
```bash
git clone https://github.com/dgalbraith/rust-utils.git
cd rust-utils && cargo install --path .
rust-utils remap /path/to/directory --from-base 100000 --to-base 50000000 --dry-run
```

---

## Future Roadmap

### Planned Features
- Additional filesystem utilities (permissions, timestamps, etc.)
- Enhanced pattern matching with regex support
- Configuration file support
- Progress reporting for large operations
- Parallel processing for improved performance

### Integration Goals
- Package distribution (AUR, Homebrew, apt/yum repositories)
- CI/CD pipeline with automated testing and releases
- Docker image for containerized usage
- Shell completion scripts

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for information on how to contribute to this project.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.