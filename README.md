# Rust Utils

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

High-performance Rust utilities for system administration and development workflows.

## Overview

Rust Utils provides a suite of fast, reliable command-line tools designed to streamline common system administration and development tasks. Built with Rust's performance and safety guarantees, these utilities offer memory-safe alternatives to traditional system tools.

**Key Features:**
- 🚀 **High Performance**: Rust's zero-cost abstractions for maximum speed
- 🔒 **Memory Safety**: No segfaults or buffer overflows
- 🔧 **Cross-Platform**: Works on all Unix-like systems
- ⚡ **Extensible**: Easy to add new utilities and commands

## Installation

### From Source

```bash
git clone https://github.com/dgalbraith/rust-utils.git
cd rust-utils
cargo build --release
sudo cp target/release/rust-utils /usr/local/bin/
```
d
### Using Cargo

```bash
cargo install rust-utils
```

## Quick Start

```bash
# Install from source
git clone https://github.com/dgalbraith/rust-utils.git
cd rust-utils && cargo install --path .

# Basic usage - preview UID/GID remapping
rust-utils remap /path/to/directory \
  --from-base 100000 --to-base 50000000 --dry-run

# Perform the actual remapping
rust-utils remap /path/to/directory \
  --from-base 100000 --to-base 50000000
```

## Available Utilities

| Command | Description | Documentation |
|---------|-------------|---------------|
| `remap` | UID/GID filesystem remapping | [Command Reference](docs/remap.md) |

## Documentation

- 📖 **[Command Reference](docs/remap.md)** - Complete command documentation
- 🚀 **[Examples](docs/examples.md)** - Real-world usage examples  
- 💻 **[Development](#development)** - Building and contributing
- 🧪 **[Testing](docs/TESTING.md)** - Comprehensive test coverage
- 🤝 **[Contributing](CONTRIBUTING.md)** - Contribution guidelines and setup
- 📋 **[Changelog](CHANGELOG.md)** - Version history and release notes

## Common Use Cases

- **Container Migration**: Move containers between hosts with different ID mappings
- **Privilege Conversion**: Convert privileged containers to unprivileged
- **System Administration**: Bulk UID/GID changes for system maintenance
- **Development**: Test applications with different user namespace configurations

## Performance

These Rust utilities offer significant performance improvements over traditional shell scripts and Python alternatives:

- **~3-10x faster** processing due to compiled nature and efficient algorithms
- **Lower memory usage** with efficient data structures and zero-copy operations
- **Better error handling** with Result types and proper error propagation
- **Safe concurrency** potential for future parallel processing features

## Development

### Prerequisites

- Rust 1.70+ (2021 edition)
- Unix-like system with standard filesystem utilities

### Building

```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Check code with clippy
cargo clippy

# Format code
cargo fmt
```

### Project Structure

```
src/
├── main.rs           # Application entry point
├── lib.rs            # Library root
├── cli.rs            # Command-line interface
├── error.rs          # Error types and handling
├── fs.rs             # Filesystem utilities
└── commands/
    ├── mod.rs        # Commands module
    └── remap.rs      # Remap command implementation
```

## Logging and Debugging

Set the `RUST_LOG` environment variable to control logging:

```bash
# Enable debug logging
RUST_LOG=debug rust-utils remap ...

# Enable info logging (default)
RUST_LOG=info rust-utils remap ...

# Enable trace logging (very verbose)
RUST_LOG=trace rust-utils remap ...
```

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines on:

- Setting up the development environment
- Code quality standards and testing requirements
- Commit message conventions
- Pull request process
- Adding new features and commands

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Rust community for excellent tooling and libraries
- System administrators who inspired these utilities
- Open source contributors and maintainers