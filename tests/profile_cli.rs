use std::process::Command;

#[test]
fn invalid_profile_exits_with_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--profile", "not-a-profile", "--help"])
        .output()
        .expect("run chiki8");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unknown profile 'not-a-profile'; expected 'classic'"));
}
