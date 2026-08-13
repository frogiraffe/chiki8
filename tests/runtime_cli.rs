use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "chiki8-runtime-{}-{}-{name}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run_rom(bytes: &[u8], extra: &[&str]) -> Output {
    let rom = temp_path("rom.ch8");
    fs::write(&rom, bytes).expect("write ROM fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--file", rom.to_str().unwrap()])
        .args(extra)
        .env("SDL_VIDEODRIVER", "dummy")
        .env("SDL_AUDIODRIVER", "dummy")
        .output()
        .expect("run chiki8");
    fs::remove_file(rom).expect("remove ROM fixture");
    output
}

#[test]
fn bounded_run_terminates_without_input() {
    let output = run_rom(&[0x12, 0x00], &["--frames", "2"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bounded_run_rejects_invalid_budget_before_sdl() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--frames", "0"])
        .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
        .output()
        .expect("run chiki8");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("CLI frames"));
}

#[test]
fn bounded_run_preserves_cpu_stop_outcomes() {
    for rom in [[0x00, 0xfd], [0x50, 0x01]] {
        let output = run_rom(&rom, &["--profile", "superchip-1.1", "--frames", "20"]);
        assert!(output.status.success(), "{:?}", output.status);
    }
}
