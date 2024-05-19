use assert_cmd::Command;
use std::fs::read_to_string;
use tempfile::tempdir;

#[test]
fn test_tuni() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let temp_dir = tempdir().unwrap();

    cmd.arg("--gtf-paths")
        .arg("tests/data/integration/gtf_paths.txt")
        .arg("--output-dir")
        .arg(temp_dir.path());

    cmd.assert().success();

    // Check assigned unified IDs are correct.
    // For descriptions of test cases, see test_case attribute in input gtfs.
    assert_eq!(
        read_to_string("tests/data/integration/expected_sample_1.tuni.gtf").unwrap(),
        read_to_string(temp_dir.path().join("sample_1.tuni.gtf")).unwrap(),
    );
    assert_eq!(
        read_to_string("tests/data/integration/expected_sample_2.tuni.gtf").unwrap(),
        read_to_string(temp_dir.path().join("sample_2.tuni.gtf")).unwrap(),
    );
}
