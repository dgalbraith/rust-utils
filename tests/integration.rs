use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use tempfile::TempDir;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("A collection of Rust utilities"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("remap"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rust-utils"));
}

#[test]
fn test_remap_help() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args(["remap", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remap UID/GID ranges"))
        .stdout(predicate::str::contains("--from-base"))
        .stdout(predicate::str::contains("--to-base"))
        .stdout(predicate::str::contains("--dry-run"));
}

#[test]
fn test_remap_missing_args() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args(["remap", "/tmp"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_remap_nonexistent_directory() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args([
        "remap",
        "/nonexistent/directory/path",
        "--from-base",
        "100000",
        "--to-base",
        "50000000",
        "--dry-run",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("Directory not found"));
}

#[test]
fn test_remap_file_instead_of_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test_file.txt");
    File::create(&file_path)?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args([
        "remap",
        file_path.to_str().unwrap(),
        "--from-base",
        "100000",
        "--to-base",
        "50000000",
        "--dry-run",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("not a directory"));

    Ok(())
}

#[test]
fn test_remap_dry_run_success() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create a simple directory structure
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir)?;
    File::create(temp_dir.path().join("file1.txt"))?;
    File::create(subdir.join("file2.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.env("RUST_LOG", "info")
        .args([
            "remap",
            temp_dir.path().to_str().unwrap(),
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("DRY RUN MODE"))
        .stdout(predicate::str::contains("Starting UID/GID remapping"))
        .stdout(predicate::str::contains("Remapping completed"));

    Ok(())
}

#[test]
fn test_remap_verbose_output() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    File::create(temp_dir.path().join("test.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.env("RUST_LOG", "info")
        .args([
            "remap",
            temp_dir.path().to_str().unwrap(),
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
            "--dry-run",
            "--verbose",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("From range: 100000-165535"))
        .stdout(predicate::str::contains("To range: 50000000-50065535"));

    Ok(())
}

#[test]
fn test_remap_with_exclusions() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;

    // Create files that should be excluded
    File::create(temp_dir.path().join("app.log"))?;
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir(&tmp_dir)?;
    File::create(tmp_dir.join("temp.txt"))?;
    File::create(temp_dir.path().join("regular.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.env("RUST_LOG", "info")
        .args([
            "remap",
            temp_dir.path().to_str().unwrap(),
            "--from-base",
            "100000",
            "--to-base",
            "50000000",
            "--dry-run",
            "--verbose",
            "--exclude",
            "*.log",
            "--exclude",
            "tmp/*",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Starting UID/GID remapping"));

    Ok(())
}

#[test]
fn test_remap_uid_only() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    File::create(temp_dir.path().join("test.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args([
        "remap",
        temp_dir.path().to_str().unwrap(),
        "--from-base",
        "100000",
        "--to-base",
        "50000000",
        "--dry-run",
        "--uid-only",
    ])
    .assert()
    .success();

    Ok(())
}

#[test]
fn test_remap_gid_only() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    File::create(temp_dir.path().join("test.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args([
        "remap",
        temp_dir.path().to_str().unwrap(),
        "--from-base",
        "100000",
        "--to-base",
        "50000000",
        "--dry-run",
        "--gid-only",
    ])
    .assert()
    .success();

    Ok(())
}

#[test]
fn test_remap_custom_range_size() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    File::create(temp_dir.path().join("test.txt"))?;

    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.env("RUST_LOG", "info")
        .args([
            "remap",
            temp_dir.path().to_str().unwrap(),
            "--from-base",
            "1000",
            "--to-base",
            "101000",
            "--range-size",
            "1000",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("From range: 1000-1999"))
        .stdout(predicate::str::contains("To range: 101000-101999"));

    Ok(())
}

#[test]
fn test_remap_invalid_range_overflow() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.args([
        "remap",
        "/tmp",
        "--from-base",
        "4294900000", // Near u32::MAX
        "--to-base",
        "50000000",
        "--range-size",
        "100000", // Would overflow
        "--dry-run",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains("overflow"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("rust-utils").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}
