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

#[test]
fn capture_frame_requires_a_bounded_run() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--capture-frame", "/tmp/chiki8-unused.bmp"])
        .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
        .output()
        .expect("run chiki8");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("capture-frame"));
}

#[test]
fn capture_frame_writes_the_final_production_canvas() {
    let capture = temp_path("frame.bmp");
    let output = run_rom(
        &[0x60, 0x00, 0x61, 0x00, 0xa0, 0x00, 0xd0, 0x15, 0x12, 0x08],
        &["--frames", "2", "--capture-frame", capture.to_str().unwrap()],
    );
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let bmp = fs::read(&capture).expect("read captured BMP");
    assert!(bmp.len() > 54);
    assert_eq!(&bmp[..2], b"BM");
    fs::remove_file(capture).unwrap();
}

#[test]
fn capture_frame_names_an_unwritable_destination() {
    let destination = temp_path("directory");
    fs::create_dir(&destination).unwrap();
    let output = run_rom(
        &[0x12, 0x00],
        &["--frames", "1", "--capture-frame", destination.to_str().unwrap()],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(destination.to_str().unwrap()));
    fs::remove_dir(destination).unwrap();
}
