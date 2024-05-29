use assert_cmd::Command;
use predicates::prelude::predicate;
use rstest::rstest;
use std::fs::read_to_string;
use tempfile::tempdir;

#[rstest]
#[case("tests/data/integration/gtf_paths.txt", "gtf")]
#[case("tests/data/integration/gff_paths.txt", "gff")]
fn test_tuni(#[case] gtf_gff_path: &str, #[case] gtf_gff_extension: &str) {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let temp_dir = tempdir().unwrap();

    // RUST_LOG=INFO env var is used to ensure env_logger stores logs to the stderr.
    cmd.env("RUST_LOG", "INFO")
        .arg("--gtf-gff-path")
        .arg(gtf_gff_path)
        .arg("--output-dir")
        .arg(temp_dir.path());

    let check_warning_message = predicate::str::contains("Unrecognised transcript ID found");

    cmd.assert().success().stderr(check_warning_message);

    // Check assigned unified IDs are correct.
    // For descriptions of test cases, see test_case attribute in input gtfs.
    assert_eq!(
        read_to_string(format!(
            "tests/data/integration/expected_sample_1.tuni.{}",
            gtf_gff_extension
        ))
        .unwrap(),
        read_to_string(
            temp_dir
                .path()
                .join(format!("sample_1.tuni.{}", gtf_gff_extension))
        )
        .unwrap(),
    );
    assert_eq!(
        read_to_string(format!(
            "tests/data/integration/expected_sample_2.tuni.{}",
            gtf_gff_extension
        ))
        .unwrap(),
        read_to_string(
            temp_dir
                .path()
                .join(format!("sample_2.tuni.{}", gtf_gff_extension))
        )
        .unwrap(),
    );
}
