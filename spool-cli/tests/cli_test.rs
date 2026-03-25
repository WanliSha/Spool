use std::process::Command;

fn spool_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_spool"))
}

fn fixture_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn copy_test_jpg(name: &str) -> std::path::PathBuf {
    let src = fixture_path().join("test.jpg");
    let dst = fixture_path().join(name);
    std::fs::copy(&src, &dst).unwrap();
    dst
}

#[test]
fn test_list_directory() {
    let output = spool_bin()
        .args(["list", fixture_path().to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test.jpg"));
    assert!(output.status.success());
}

#[test]
fn test_list_empty_dir() {
    let dir = fixture_path().join("empty_subdir");
    std::fs::create_dir_all(&dir).unwrap();
    let output = spool_bin()
        .args(["list", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
    std::fs::remove_dir(&dir).ok();
}

#[test]
fn test_get_nonexistent_file() {
    let output = spool_bin()
        .args(["get", "/tmp/nonexistent_spool_test.jpg"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_set_and_get_field() {
    let file = copy_test_jpg("test_set_get.jpg");
    let path = file.to_str().unwrap();

    // Set a field
    let output = spool_bin()
        .args(["set", path, "--field", "Author", "--value", "Test Photographer"])
        .output()
        .unwrap();
    assert!(output.status.success(), "set failed: {}", String::from_utf8_lossy(&output.stderr));

    // Get it back
    let output = spool_bin()
        .args(["get", path, "Author"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Test Photographer");

    std::fs::remove_file(&file).ok();
}

#[test]
fn test_set_date_normalizes() {
    let file = copy_test_jpg("test_date_norm.jpg");
    let path = file.to_str().unwrap();

    let output = spool_bin()
        .args(["set", path, "--field", "DateTaken", "--value", "2024-12-25"])
        .output()
        .unwrap();
    assert!(output.status.success(), "set failed: {}", String::from_utf8_lossy(&output.stderr));

    let output = spool_bin()
        .args(["get", path, "DateTaken"])
        .output()
        .unwrap();
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert!(value.contains("12:00:00"), "Expected 12:00:00 default time, got: {}", value);

    std::fs::remove_file(&file).ok();
}

#[test]
fn test_set_json_multiple_fields() {
    let file = copy_test_jpg("test_json_set.jpg");
    let path = file.to_str().unwrap();

    let output = spool_bin()
        .args(["set", path, "--json", r#"{"Author":"Alice","Copyright":"2024 Alice"}"#])
        .output()
        .unwrap();
    assert!(output.status.success(), "set --json failed: {}", String::from_utf8_lossy(&output.stderr));

    let output = spool_bin()
        .args(["get", path, "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Alice"));

    std::fs::remove_file(&file).ok();
}

#[test]
fn test_get_json_output() {
    let file = copy_test_jpg("test_json_get.jpg");
    let path = file.to_str().unwrap();

    spool_bin()
        .args(["set", path, "--field", "Author", "--value", "Bob"])
        .output()
        .unwrap();

    let output = spool_bin()
        .args(["get", path, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("output should be valid JSON");
    assert_eq!(parsed["Author"], "Bob");

    std::fs::remove_file(&file).ok();
}

#[test]
fn test_unsupported_file() {
    let output = spool_bin()
        .args(["set", "/tmp/test.txt", "--field", "Author", "--value", "X"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported") || stderr.contains("not found"));
}

#[test]
fn test_set_multiple_files() {
    let file1 = copy_test_jpg("test_multi_1.jpg");
    let file2 = copy_test_jpg("test_multi_2.jpg");
    let path1 = file1.to_str().unwrap();
    let path2 = file2.to_str().unwrap();

    // Set same field on both files at once
    let output = spool_bin()
        .args(["set", path1, path2, "--json", r#"{"Author":"BatchTest","CameraMake":"Nikon"}"#])
        .output()
        .unwrap();
    assert!(output.status.success(), "batch set failed: {}", String::from_utf8_lossy(&output.stderr));

    // Verify both files
    for path in [path1, path2] {
        let output = spool_bin()
            .args(["get", path, "Author"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "BatchTest");
    }

    std::fs::remove_file(&file1).ok();
    std::fs::remove_file(&file2).ok();
}

#[test]
fn test_set_multiple_files_with_field_value() {
    let file1 = copy_test_jpg("test_multi_fv_1.jpg");
    let file2 = copy_test_jpg("test_multi_fv_2.jpg");
    let path1 = file1.to_str().unwrap();
    let path2 = file2.to_str().unwrap();

    let output = spool_bin()
        .args(["set", path1, path2, "--field", "Copyright", "--value", "2024 Batch"])
        .output()
        .unwrap();
    assert!(output.status.success(), "batch set failed: {}", String::from_utf8_lossy(&output.stderr));

    for path in [path1, path2] {
        let output = spool_bin()
            .args(["get", path, "Copyright"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "2024 Batch");
    }

    std::fs::remove_file(&file1).ok();
    std::fs::remove_file(&file2).ok();
}
