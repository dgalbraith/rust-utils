# Testing Documentation

## Overview

The rust-utils project maintains comprehensive test coverage across unit tests, integration tests, and benchmarking infrastructure. The test suite is designed to ensure reliability, validate all functionality, and provide confidence in code changes.

## Test Statistics

To get current test statistics, run:

```bash
# Get test count and results
cargo test --quiet

# Get detailed coverage report
cargo tarpaulin

# Get coverage breakdown by file
cargo tarpaulin --out Html --output-dir coverage
```

**Current test structure:**
- **Unit Tests**: Comprehensive testing across all modules
- **Integration Tests**: End-to-end CLI testing
- **Coverage Target**: Maintain >60% overall coverage

## Test Structure

### Unit Tests

#### CLI Module (`src/cli.rs`)
Tests CLI argument parsing, help messages, and command validation:
- ✅ Help message display validation
- ✅ Version flag functionality  
- ✅ Command parsing (basic and advanced options)
- ✅ Error handling for missing/invalid arguments
- ✅ Range validation testing

#### Commands Module (`src/commands/remap.rs`)
Tests core business logic and command execution:
- ✅ Dry run functionality
- ✅ Directory validation and error handling
- ✅ Pattern exclusion functionality
- ✅ Hard link detection and tracking
- ✅ UID/GID range validation and overflow protection
- ✅ Argument validation and mutual exclusivity

#### Error Module (`src/error.rs`)
Tests error handling and reporting:
- ✅ Error message formatting and display
- ✅ Debug trait implementations
- ✅ Error type conversions (IO, etc.)
- ✅ Result type alias verification

#### Filesystem Module (`src/fs.rs`)
Tests filesystem operations and utilities:
- ✅ File metadata retrieval and error handling
- ✅ Pattern matching logic and edge cases
- ✅ Exclusion logic with multiple patterns
- ✅ Empty pattern list handling

### Integration Tests

#### CLI Integration (`tests/integration.rs`)
End-to-end testing of complete CLI workflows:
- ✅ CLI help and version display integration
- ✅ Invalid command handling
- ✅ Argument validation and error reporting
- ✅ Dry run end-to-end testing
- ✅ Directory and file validation
- ✅ UID-only and GID-only remapping modes
- ✅ Custom range validation
- ✅ Pattern exclusion integration
- ✅ Verbose logging validation
- ✅ Range overflow protection

## Coverage Analysis

### Getting Current Coverage

```bash
# Terminal coverage summary
cargo tarpaulin

# Detailed HTML report with line-by-line coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Coverage Targets by Module

| Module | Target Coverage | Notes |
|--------|----------------|-------|
| `error.rs` | 100% | Error handling should be fully tested |
| `fs.rs` | 95%+ | Core filesystem operations |
| `commands/remap.rs` | 70%+ | Business logic coverage |
| `main.rs` | 50%+ | Entry point integration |

**Overall Target**: Maintain >60% total coverage

### Understanding Coverage Reports

The HTML coverage report shows:
- **File-by-file coverage percentages**
- **Line-by-line coverage highlighting**
  - 🟢 Green: Lines covered by tests
  - 🔴 Red: Lines not covered by tests
  - 🟡 Yellow: Partially covered lines
- **Function-level coverage statistics**

## Testing Infrastructure

### Dependencies
- **Unit Testing**: Built-in Rust test framework
- **Integration Testing**: `assert_cmd` for CLI testing
- **Test Utilities**: `tempfile`, `predicates` for file operations
- **Benchmarking**: `criterion` for performance testing
- **Coverage**: `cargo-tarpaulin` for coverage analysis

### Running Tests

#### Basic Test Execution

```bash
# Run all tests (unit + integration tests)
cargo test

# Run with verbose output to see individual test names
cargo test --verbose

# Run tests with output from println! and tracing
cargo test -- --nocapture

# Run tests quietly (suppress compilation output)
cargo test --quiet
```

#### Targeted Test Execution

```bash
# Run only unit tests (library tests)
cargo test --lib

# Run specific test module
cargo test commands::remap::tests

# Run integration tests only
cargo test --test integration

# Run specific test by name
cargo test test_execute_dry_run

# Run tests matching a pattern
cargo test remap

# Run tests in a specific file
cargo test --test integration test_cli_help
```

#### Test Environment Configuration

```bash
# Run tests with debug logging enabled
RUST_LOG=debug cargo test

# Run tests with specific log level for integration tests
RUST_LOG=info cargo test --test integration

# Run tests with colored output disabled
NO_COLOR=1 cargo test
```

### Generating Coverage Reports

#### Prerequisites

First, install the coverage tool (one-time setup):

```bash
cargo install cargo-tarpaulin
```

#### Basic Coverage Analysis

```bash
# Generate terminal coverage report
cargo tarpaulin

# Example output:
# 65.12% coverage, 84/129 lines covered
```

#### HTML Coverage Reports (Recommended)

```bash
# Generate detailed HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# Open the report in your browser
xdg-open coverage/tarpaulin-report.html

# Or manually navigate to:
# file:///path/to/rust-utils/coverage/tarpaulin-report.html
```

#### Multiple Output Formats

```bash
# Generate multiple report formats
cargo tarpaulin --out Html --out Lcov --out Json --output-dir coverage

# Available formats:
# - Html: Interactive web-based report
# - Lcov: LCOV format for external tools (e.g., IDE integrations)
# - Json: Machine-readable JSON format
# - Xml: Cobertura XML format for CI/CD systems
```

#### Advanced Coverage Options

```bash
# Coverage for specific test types
cargo tarpaulin --lib                    # Unit tests only
cargo tarpaulin --test integration       # Integration tests only
cargo tarpaulin --tests                  # All test binaries

# Exclude specific files from coverage
cargo tarpaulin --exclude-files "target/*" --exclude-files "tests/*"

# Include ignored tests
cargo tarpaulin -- --ignored

# Generate coverage with timeout
cargo tarpaulin --timeout 120

# Verbose coverage generation
cargo tarpaulin --verbose --out Html --output-dir coverage
```

#### Coverage Report Contents

The HTML coverage report includes:
- **File-by-file coverage percentages**
- **Line-by-line coverage highlighting**
  - 🟢 Green: Lines covered by tests
  - 🔴 Red: Lines not covered by tests
  - 🟡 Yellow: Partially covered lines
- **Function-level coverage statistics**
- **Interactive source code browsing**
- **Coverage trends and summaries**

#### Current Project Coverage Stats

```bash
# Get current coverage statistics:
cargo tarpaulin

# Example output format:
# || Tested/Total Lines:
# || src/commands/remap.rs: XX/YY
# || src/error.rs: X/Y
# || src/fs.rs: XX/YY
# || src/main.rs: X/Y
# || 
# XX.XX% coverage, XXX/YYY lines covered
```

### Benchmarking

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench remap_performance

# Generate benchmark report
cargo bench -- --output-format html
```

### Continuous Integration Testing

```bash
# CI-friendly test command (no interactive output)
cargo test --color never --quiet

# Generate coverage for CI
cargo tarpaulin --out Xml --output-dir coverage

# Full CI test pipeline
cargo check && cargo test && cargo tarpaulin --out Lcov --output-dir coverage
```

## Test Quality Indicators

### Comprehensive Error Testing
- ✅ Invalid arguments and edge cases
- ✅ File system error conditions
- ✅ Permission and access errors
- ✅ Numeric overflow protection

### Real-world Scenarios
- ✅ Temporary directory operations
- ✅ Pattern matching with actual files
- ✅ CLI argument validation
- ✅ Dry run vs actual execution

### Performance Considerations
- ✅ Benchmark infrastructure in place
- ✅ Hard link detection efficiency
- ✅ Pattern matching performance

## Continuous Integration

The test suite is designed to be CI-friendly:
- Fast execution (< 1 second for all tests)
- No external dependencies required
- Deterministic results
- Clear failure reporting

## Test Data Management

Tests use:
- Temporary directories for filesystem operations
- Mock data structures for unit tests
- Controlled CLI input/output for integration tests
- No external test data files required

## Quick Reference

### Common Test Commands

```bash
# Standard development workflow
cargo test                                    # Run all tests
cargo test --verbose                         # See individual test names
cargo tarpaulin --out Html --output-dir coverage  # Generate coverage report

# Debugging specific issues
cargo test test_name -- --nocapture         # Debug specific test with output
RUST_LOG=debug cargo test integration       # Run with debug logging
cargo test --test integration -- --nocapture # Integration tests with output

# CI/Production commands
cargo test --quiet                          # Minimal output for CI
cargo tarpaulin --out Lcov --output-dir coverage  # Coverage for CI systems
```

### Coverage Targets by Module

| Module | Current Status | Target | Priority |
|--------|---------------|---------|----------|
| `error.rs` | ✅ Excellent | 100% | Maintain |
| `fs.rs` | ✅ Near Complete | 95%+ | Maintain |
| `commands/remap.rs` | 🟢 Good | 70%+ | Stable |
| `main.rs` | 🔴 Needs Work | 50%+ | Improve |

*Run `cargo tarpaulin` for current exact percentages*

### Troubleshooting

**Tests failing due to permissions?**
```bash
# Run tests with appropriate permissions
sudo -E cargo test  # Preserve environment variables
```

**Coverage tool not found?**
```bash
cargo install cargo-tarpaulin
```

**Tests hanging or slow?**
```bash
cargo test --timeout 30  # Set timeout
cargo test -- --test-threads=1  # Single-threaded execution
```

**Need clean test environment?**
```bash
cargo clean && cargo test  # Clean rebuild and test
```

### Version Control Best Practices

**Coverage reports should NOT be checked into Git:**
- The `/coverage/` directory is excluded in `.gitignore`
- Coverage reports are generated artifacts, not source code
- They can be regenerated anytime with `cargo tarpaulin`
- Different environments may produce different coverage results

**Files excluded from version control:**
```gitignore
# Coverage reports
/coverage/
*.profraw
tarpaulin-report.html
cobertura.xml
lcov.info
```

**For CI/CD pipelines:**
- Generate coverage reports as build artifacts
- Upload to coverage services (e.g., Codecov, Coveralls)
- Store reports as CI artifacts for download

This comprehensive test suite ensures reliability, maintainability, and confidence in the rust-utils functionality.