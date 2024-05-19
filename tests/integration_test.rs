use assert_cmd::Command;

#[test]
fn test_tuni() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--gtf-paths")
        .arg("tests/data/integration/gtf_paths.txt")
        .arg("--output-dir")
        .arg("tests/data/integration/");
    cmd.assert().success();

    let output = cmd.unwrap();
    println!("{:?}", output);
}
