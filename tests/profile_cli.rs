use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn rom(bytes: &[u8]) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "chiki8-profile-{}-{}.ch8",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::write(&path, bytes).expect("write ROM fixture");
    path
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(args)
        .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
        .output()
        .expect("run chiki8")
}

#[test]
fn superchip_profile_is_accepted_exactly() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--profile", "superchip-1.1", "--help"])
        .output()
        .expect("run chiki8");

    assert!(output.status.success());
}

#[test]
fn superchip_profile_is_case_sensitive() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--profile", "SuperChip-1.1", "--help"])
        .output()
        .expect("run chiki8");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unknown profile 'SuperChip-1.1'; expected 'classic' or 'superchip-1.1'"));
}

#[test]
fn invalid_profile_exits_with_an_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--profile", "not-a-profile", "--help"])
        .output()
        .expect("run chiki8");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("unknown profile 'not-a-profile'; expected 'classic' or 'superchip-1.1'"));
}

#[test]
fn profile_resolution_cli_is_reported_before_sdl() {
    let rom = rom(&[0x00, 0xfd]);
    let output = run(&[
        "--profile",
        "superchip-1.1",
        "--file",
        rom.to_str().unwrap(),
    ]);
    fs::remove_file(rom).unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("Profile: superchip-1.1 (source: cli)"));
}

#[test]
fn profile_resolution_unknown_rom_falls_back_before_sdl() {
    let rom = rom(&[0x00, 0xe0]);
    let output = run(&["--file", rom.to_str().unwrap()]);
    fs::remove_file(rom).unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "Profile: classic (source: fallback; reason: no explicit profile or known ROM hash match)"
    ));
}

#[test]
fn profile_resolution_bad_cli_and_rom_are_status_two() {
    let malformed = run(&["--unknown"]);
    assert_eq!(malformed.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&malformed.stderr).is_empty());

    let unreadable = run(&["--file", "/definitely/not/a/chiki8-rom"]);
    assert_eq!(unreadable.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unreadable.stderr).contains("Could not read ROM"));
}
