# Agent Instructions for `rust-utils`

## Project Overview

This project provides a suite of high-performance, memory-safe Rust CLI utilities for system
administration and development workflows. The project is designed to be extensible, with a clean
architecture for adding new utilities.

**Current Features:**
- **UID/GID Remapping**: `remap` command for LXC filesystem ownership changes
- **Pattern Exclusions**: Flexible file filtering with glob-like patterns
- **Hard Link Detection**: Intelligent handling to avoid duplicate processing
- **Dry Run Support**: Safe testing mode for all operations

## Technical Stack & Architecture

- **Language**: Rust 2021 Edition (minimum version 1.70+)
- **CLI Framework**: `clap` v4.4+ with derive macros for argument parsing
- **Error Handling**: 
  - `thiserror` for custom error types with structured error information
  - `anyhow` for application-level error propagation and context
- **Filesystem Operations**: 
  - `walkdir` for efficient recursive directory traversal
  - `nix` crate for Unix system operations (UID/GID changes, file metadata)
- **Logging**: `tracing` and `tracing-subscriber` for structured logging
- **Testing Infrastructure**:
  - `tempfile` for safe test filesystem operations
  - `assert_cmd` for CLI integration testing
  - `predicates` for advanced test assertions
  - `criterion` for performance benchmarking

### **Architecture Pattern**
- **Command Pattern**: Each utility is a separate command module
- **Library + Binary**: Core logic in `lib.rs`, CLI in `main.rs`
- **Module Organization**:
  ```
  src/
  ├── lib.rs          # Library root with public API
  ├── main.rs         # CLI entry point with tracing setup
  ├── cli.rs          # Clap CLI definitions
  ├── error.rs        # Custom error types
  ├── fs.rs           # Filesystem utilities
  └── commands/       # Command implementations
      ├── mod.rs      # Command module declarations
      └── remap.rs    # UID/GID remapping command
  ``` 

## Core Principles & Coding Standards

- **Idiomatic Rust**: Code must adhere to modern Rust idioms and best practices
- **Memory Safety First**: Zero `unsafe` code, minimal use of `unwrap()`/`expect()` 
- **Error Handling**: 
  - Use `Result<T, E>` for all fallible operations
  - Provide meaningful error messages with context
  - Custom error types in `error.rs` with proper `Display` implementations
- **Testing Excellence**:
  - **Target**: Maintain >60% overall test coverage
  - **Test Types**: Unit tests, integration tests, benchmark tests
  - **Requirements**: All new features need comprehensive tests
- **Code Quality**:
  - **Zero warnings**: Must pass `cargo clippy --all-targets --all-features`
  - **Consistent formatting**: Code must be formatted with `cargo fmt`
  - **Documentation**: All public APIs must have `///` documentation comments

### **Performance Standards**
- Operations should be efficient for large directory trees
- Use iterators and zero-copy operations where possible
- Benchmark critical paths with `criterion`
- Profile memory usage for recursive operations

## Documentation Standards

### **Code Documentation**
- **Public APIs**: Every public function, struct, enum, and module must have `///` documentation
- **Examples**: Include usage examples in doc comments for complex functions
- **Error Documentation**: Document error conditions and return types
- **Panics**: Document any conditions that could cause panics

### **Project Documentation**
- **README.md**: Installation, basic usage, features overview
- **CONTRIBUTING.md**: Development setup, testing guidelines, PR process
- **CHANGELOG.md**: Version history following [Keep a Changelog](https://keepachangelog.com/)
- **docs/**: Command-specific documentation and examples
- **docs/TESTING.md**: Comprehensive testing documentation

### **Documentation Quality**
- Use clear, concise language
- Provide practical examples
- Keep documentation current with code changes
- Include performance considerations where relevant

## Naming Conventions

Follow Rust's standard naming conventions strictly:
- **Functions & Variables**: `snake_case` (e.g., `process_file`, `base_directory`)
- **Structs & Enums**: `PascalCase` (e.g., `RemapCommand`, `RustUtilsError`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_RANGE_SIZE`)
- **Modules**: `snake_case` (e.g., `commands`, `error`)
- **CLI Arguments**: `kebab-case` for user-facing args (e.g., `--dry-run`, `--from-base`)

## Testing Framework & Standards

### **Test Organization**
```
tests/
├── integration.rs      # CLI integration tests
├── main_integration.rs # Main function testing
└── (future test files)

src/
├── lib.rs             # Unit tests embedded in modules
├── commands/remap.rs  # Unit tests for remap functionality
├── error.rs           # Error handling tests
└── fs.rs              # Filesystem operation tests

benches/
└── pattern_matching.rs # Performance benchmarks
```

### **Testing Requirements**
- **Unit Tests**: Test individual functions and methods in isolation
- **Integration Tests**: Test complete CLI workflows using `assert_cmd`
- **Error Testing**: Verify all error conditions and edge cases
- **Performance Tests**: Benchmark critical operations with `criterion`

### **Test Naming Convention**
```rust
#[test]
fn test_{functionality}_{scenario}() {
    // Examples:
    // test_should_remap_file_uid_in_range()
    // test_execute_nonexistent_directory()
    // test_matches_pattern_edge_cases()
}
```

## Command Implementation Guidelines

### **New Command Checklist**
When adding a new command:

1. **Create Command Module**: `src/commands/new_command.rs`
2. **Define CLI Args**: Struct with `#[derive(Args)]`
3. **Implement Command**: Struct with `new()` and `execute()` methods
4. **Add to CLI**: Update `src/cli.rs` with new command variant
5. **Error Handling**: Use custom error types from `error.rs`
6. **Testing**: Add unit tests and integration tests
7. **Documentation**: Update relevant docs and examples
8. **Benchmarks**: Add performance tests if needed

### **Command Structure Pattern**
```rust
use anyhow::Result;
use clap::Args;
use tracing::{info, debug};

#[derive(Args)]
pub struct NewCommandArgs {
    /// Required argument description
    pub required_arg: PathBuf,
    
    /// Optional flag with short and long form
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Optional value with default
    #[arg(long, default_value = "1000")]
    pub some_value: u32,
}

pub struct NewCommand {
    args: NewCommandArgs,
}

impl NewCommand {
    pub fn new(args: NewCommandArgs) -> Self {
        Self { args }
    }
    
    pub fn execute(&self) -> Result<()> {
        info!("Starting {} command", "new_command");
        
        // Validation
        self.validate_args()?;
        
        // Implementation
        self.run_command()?;
        
        info!("Command completed successfully");
        Ok(())
    }
    
    fn validate_args(&self) -> Result<()> {
        // Argument validation logic
        Ok(())
    }
    
    fn run_command(&self) -> Result<()> {
        // Main command logic
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_new_command_success() {
        // Test implementation
    }
    
    #[test]
    fn test_new_command_error_case() {
        // Error case testing
    }
}
```

## CI/CD and Development Environment

### **Development Workflow**
```bash
# Setup
cargo build                    # Development build
cargo build --release         # Optimized build

# Quality Assurance (must pass before PR)
cargo test --all-targets      # Run all tests (53+ tests)
cargo clippy --all-targets --all-features -- -D warnings  # Zero warnings
cargo fmt --check             # Code formatting
cargo tarpaulin --out Html     # Generate coverage report (target: >60%)

# Performance Validation
cargo bench                    # Run benchmarks

# Documentation
cargo doc --no-deps --open    # Generate and view docs
```

### **Pre-commit Checklist**
- [ ] All tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Coverage maintained (use `cargo tarpaulin`)
- [ ] Documentation updated
- [ ] Changelog updated (for releases)

### **Release Process**
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with new features/fixes
3. Run full test suite and quality checks
4. Create git tag with `v{version}` format
5. Build release binary: `cargo build --release`

## Current Project Metrics

### **Code Quality Metrics**
- **Clippy Warnings**: 0 (zero tolerance policy)
- **Documentation Coverage**: All public APIs documented
- **Performance**: Benchmarked with criterion
- **Memory Safety**: Zero unsafe code blocks

### **Dependency Health**
- All dependencies actively maintained
- Security audit clean (`cargo audit`)
- Minimal dependency tree for fast builds
- No deprecated crates

## Agent-Specific Guidelines

### **When Adding Features**
1. **Understand Context**: Review existing patterns in codebase
2. **Follow Architecture**: Use command pattern for new utilities
3. **Test Thoroughly**: Aim for >70% coverage on new code
4. **Document Everything**: Code comments + user documentation
5. **Performance Aware**: Consider large directory tree scenarios

### **Common Patterns to Follow**
- Use `tracing::info!` for user-visible operations
- Use `tracing::debug!` for internal state logging  
- Handle filesystem errors gracefully with meaningful messages
- Use `tempfile::TempDir` for all test filesystem operations
- Validate user inputs early and provide helpful error messages

### **Anti-Patterns to Avoid**
- Don't use `unwrap()` or `panic!()` in library code
- Don't ignore errors - always handle or propagate
- Don't write tests without cleanup (use `TempDir`)
- Don't add dependencies without justification
- Don't break existing CLI interfaces without major version bump
