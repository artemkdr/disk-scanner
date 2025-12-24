use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Get a command for running the disk-scanner binary
fn cmd() -> Command {
    cargo_bin_cmd!("disk-scanner")
}

#[test]
fn test_help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("disk-scanner"))
        .stdout(predicate::str::contains("PATH"));
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_scan_current_directory() {
    cmd().assert().success();
}

#[test]
fn test_scan_specific_directory() {
    let dir = tempdir().unwrap();

    // Create test structure
    fs::write(dir.path().join("file1.txt"), "hello world").unwrap();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    fs::write(dir.path().join("subdir/file2.txt"), "content").unwrap();

    cmd()
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Disk Usage Report"));
}

#[test]
fn test_count_flag() {
    let dir = tempdir().unwrap();

    // Create multiple directories
    for i in 0..5 {
        let subdir = dir.path().join(format!("dir{}", i));
        fs::create_dir(&subdir).unwrap();
        fs::write(subdir.join("file.txt"), "x".repeat(i * 100)).unwrap();
    }

    cmd()
        .arg(dir.path())
        .args(["-n", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Top 3 by size"));
}

#[test]
fn test_all_flag_includes_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("testfile.txt"), "content").unwrap();

    cmd()
        .arg(dir.path())
        .arg("--all")
        .assert()
        .success()
        .stdout(predicate::str::contains("testfile.txt"));
}

#[test]
fn test_nonexistent_path() {
    cmd()
        .arg("/nonexistent/path/that/does/not/exist")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot access path"));
}

#[test]
fn test_file_instead_of_directory() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();

    cmd()
        .arg(&file_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("is not a directory"));
}

#[test]
fn test_depth_flag() {
    let dir = tempdir().unwrap();

    // Create nested structure
    let level1 = dir.path().join("level1");
    let level2 = level1.join("level2");
    let level3 = level2.join("level3");

    fs::create_dir_all(&level3).unwrap();
    fs::write(level3.join("deep.txt"), "deep content").unwrap();

    cmd().arg(dir.path()).args(["-d", "1"]).assert().success();
}
