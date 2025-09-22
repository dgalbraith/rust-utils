# Contributing to rust-utils

Thank you for your interest in contributing to rust-utils! This document provides guidelines and information for contributors.

## Contributing

**Quick testing checklist when contributing:**
1. **Unit Tests**: Add tests for new functions/methods
2. **Integration Tests**: Add CLI tests for new commands
3. **Error Cases**: Test all error conditions
4. **Documentation**: Update relevant documentation
5. **Coverage**: Aim to maintain >60% overall coverage


## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+** (2021 edition)
- **Unix-like system** with standard filesystem utilities
- **Git** for version control

### Development Setup

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/rust-utils.git
   cd rust-utils
   ```

2. **Build the project:**
   ```bash
   # Development build
   cargo build
   
   # Release build with optimizations
   cargo build --release
   ```

3. **Run tests to ensure everything works:**
   ```bash
   cargo test
   ```

4. **Check code quality:**
   ```bash
   cargo clippy
   cargo fmt --check
   ```

## 📝 Contribution Process

### 1. Fork and Branch

1. **Fork the repository** on GitHub
2. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/amazing-feature
   ```

### 2. Development Guidelines

#### Code Quality Standards

Your code must:
- ✅ **Follow Rust idioms and best practices**
- ✅ **Pass all existing tests**
- ✅ **Include tests for new functionality**
- ✅ **Pass `cargo clippy` without warnings**
- ✅ **Be formatted with `cargo fmt`**
- ✅ **Have appropriate documentation**

#### Testing Requirements

When adding new features:
- **Unit Tests**: Add tests for new functions/methods
- **Integration Tests**: Add CLI tests for new commands  
- **Error Cases**: Test all error conditions
- **Documentation**: Update relevant documentation
- **Coverage**: Aim to maintain >60% overall coverage

#### Running Quality Checks

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy --all-targets --all-features

# Run all tests
cargo test

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage

# Run benchmarks
cargo bench
```

### 3. Commit Guidelines

#### Commit Message Format

Use clear, descriptive commit messages:

```
type(scope): brief description

Longer description if needed explaining what and why,
not how (the code explains how).

Fixes #issue_number
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Maintenance tasks

**Examples:**
```bash
git commit -m "feat(remap): add support for symbolic links"
git commit -m "fix(cli): handle invalid range size gracefully"
git commit -m "docs(testing): update coverage documentation"
```

### 4. Pull Request Process

1. **Push your changes:**
   ```bash
   git push origin feature/amazing-feature
   ```

2. **Create a Pull Request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Reference to any related issues
   - Screenshots or examples if applicable

3. **Ensure CI passes** - all tests and checks must pass

4. **Respond to feedback** - address review comments promptly

### 5. Release Process

When preparing releases:
- Update version in `Cargo.toml`
- Update `CHANGELOG.md` following [Keep a Changelog](https://keepachangelog.com/) format
- Tag the release with semantic versioning (e.g., `v0.1.0`)
- Update documentation if needed

## 🧪 Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test commands::remap::tests

# Run integration tests only
cargo test --test integration

# Run with verbose output
cargo test -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo test
```

### Writing Tests

#### Unit Tests
- Test individual functions and methods
- Use descriptive test names: `test_should_remap_file_uid_in_range`
- Test both success and error cases
- Use `tempfile` for filesystem operations

#### Integration Tests
- Test complete CLI workflows
- Use `assert_cmd` for CLI testing
- Test all command combinations
- Verify output and exit codes

#### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_feature_success_case() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        
        // Act
        let result = your_function(&temp_dir.path());
        
        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_error_case() {
        // Test error conditions
        let result = your_function(Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}
```

## 📚 Documentation Guidelines

### Code Documentation

- Use `///` for public API documentation
- Include examples in doc comments
- Document error conditions
- Keep documentation up to date

### Project Documentation

- Update relevant `.md` files in `docs/`
- Add examples for new features
- Update command reference for CLI changes
- Keep README.md current

## 🎯 Adding New Features

### New Commands

1. **Create command module** in `src/commands/`
2. **Add CLI arguments** in `src/cli.rs`
3. **Implement command logic** with proper error handling
4. **Add comprehensive tests** (unit + integration)
5. **Update documentation** in `docs/`
6. **Add usage examples**

### Example Command Structure

```rust
// src/commands/newcommand.rs
use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct NewCommandArgs {
    /// Input file path
    pub input: PathBuf,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

pub struct NewCommand {
    args: NewCommandArgs,
}

impl NewCommand {
    pub fn new(args: NewCommandArgs) -> Self {
        Self { args }
    }
    
    pub fn execute(&self) -> Result<()> {
        // Implementation
        Ok(())
    }
}
```

## 🐛 Reporting Issues

### Bug Reports

Please include:
- **Rust version** (`rustc --version`)
- **Operating system** and version
- **Command that failed** (exact command line)
- **Expected behavior**
- **Actual behavior**
- **Error messages** or logs
- **Steps to reproduce**

### Feature Requests

Please include:
- **Use case** - what problem does this solve?
- **Proposed solution** - how should it work?
- **Alternatives considered**
- **Implementation notes** if you have ideas

## 🔧 Development Tools

### Useful Commands

```bash
# Watch for changes and run tests
cargo watch -x test

# Generate documentation
cargo doc --open

# Check dependency tree
cargo tree

# Update dependencies
cargo update

# Security audit
cargo audit
```

## 📋 Code Review Checklist

Before submitting your PR, ensure:

- [ ] Code follows Rust best practices
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] No breaking changes (or properly documented)
- [ ] Performance impact considered
- [ ] Error handling is appropriate

## 📞 Getting Help

- **Questions**: Open a GitHub issue with the "question" label
- **Discussions**: Use GitHub Discussions for broader topics
- **Security**: See SECURITY.md for responsible disclosure

## 📄 License

By contributing to rust-utils, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to rust-utils! 🦀✨