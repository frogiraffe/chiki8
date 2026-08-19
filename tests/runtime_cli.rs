use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn temp_path(name: &str) -> PathBuf {
    let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("chiki8-runtime-{}-{id}-{name}", std::process::id()))
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
fn invalid_audio_driver_returns_an_sdl_audio_error() {
    let rom = temp_path("rom.ch8");
    fs::write(&rom, [0x12, 0x00]).expect("write ROM fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--file", rom.to_str().unwrap(), "--frames", "1"])
        .env("SDL_VIDEODRIVER", "dummy")
        .env("SDL_AUDIODRIVER", "chiki8-invalid-driver")
        .output()
        .expect("run chiki8");
    fs::remove_file(rom).expect("remove ROM fixture");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("SDL audio"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn bounded_run_preserves_cpu_stop_outcomes() {
    let halted = run_rom(
        &[0x00, 0xfd],
        &["--profile", "superchip-1.1", "--frames", "20"],
    );
    assert!(halted.status.success(), "{:?}", halted.status);

    let unsupported = run_rom(
        &[0x50, 0x01],
        &["--profile", "superchip-1.1", "--frames", "20"],
    );
    assert_eq!(unsupported.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("Unsupported opcode"));
}

#[test]
fn cycle_budget_terminates_and_captures_after_exact_ticks() {
    let capture = temp_path("cycle-frame.bmp");
    let output = run_rom(
        &[0x60, 0x00, 0x61, 0x00, 0xa0, 0x00, 0xd0, 0x15, 0x12, 0x08],
        &[
            "--cycles",
            "4",
            "--capture-frame",
            capture.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(&fs::read(&capture).expect("read cycle capture")[..2], b"BM");
    fs::remove_file(capture).unwrap();
}

#[test]
fn cycle_budget_rejects_invalid_values_before_sdl() {
    for value in ["0", "nope"] {
        let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
            .args(["--cycles", value])
            .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
            .output()
            .expect("run chiki8");
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr).contains("CLI cycles"));
    }
}

#[test]
fn cycle_budget_rejects_a_simultaneous_frame_budget_before_sdl() {
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--frames", "1", "--cycles", "1"])
        .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
        .output()
        .expect("run chiki8");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be combined"));
}

#[test]
fn cycle_budget_preserves_cpu_stop_outcomes() {
    let halted = run_rom(
        &[0x00, 0xfd],
        &["--profile", "superchip-1.1", "--cycles", "20"],
    );
    assert!(halted.status.success(), "{:?}", halted.status);

    let unsupported = run_rom(
        &[0x50, 0x01],
        &["--profile", "superchip-1.1", "--cycles", "20"],
    );
    assert_eq!(unsupported.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unsupported.stderr).contains("Unsupported opcode"));
}

#[test]
fn cycle_budget_captures_a_halted_final_canvas() {
    let capture = temp_path("halted-cycle-frame.bmp");
    let output = run_rom(
        &[0x00, 0xff, 0x00, 0xfd],
        &[
            "--profile",
            "superchip-1.1",
            "--cycles",
            "20",
            "--capture-frame",
            capture.to_str().unwrap(),
        ],
    );
    assert!(output.status.success());
    assert_eq!(
        &fs::read(&capture).expect("read halted capture")[..2],
        b"BM"
    );
    fs::remove_file(capture).unwrap();
}

#[test]
fn suite_selector_rejects_values_outside_the_documented_range_before_sdl() {
    for value in ["0", "6", "nope"] {
        let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
            .args(["--suite-selector", value])
            .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
            .output()
            .expect("run chiki8");
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("expected an integer from 1 through 5"));
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
        &[
            "--frames",
            "2",
            "--background",
            "1,2,3",
            "--color",
            "4,5,6",
            "--capture-frame",
            capture.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let bmp = fs::read(&capture).expect("read captured BMP");
    assert!(bmp.len() > 54);
    assert_eq!(&bmp[..2], b"BM");
    let u32_at = |offset| u32::from_le_bytes(bmp[offset..offset + 4].try_into().unwrap());
    let (width, height) = (u32_at(18), u32_at(22));
    assert_eq!((width, height), (64 * 15, 32 * 15));
    let pixel_offset = u32_at(10) as usize;
    let row_stride = (width as usize * 3).div_ceil(4) * 4;
    let top_row = pixel_offset + (height as usize - 1) * row_stride;
    assert_eq!(&bmp[top_row..top_row + 3], &[6, 5, 4]);
    assert_eq!(&bmp[top_row + 60 * 3..top_row + 60 * 3 + 3], &[3, 2, 1]);
    fs::remove_file(capture).unwrap();
}

#[test]
fn capture_frame_names_an_unwritable_destination() {
    let destination = temp_path("directory");
    fs::create_dir(&destination).unwrap();
    let output = run_rom(
        &[0x12, 0x00],
        &[
            "--frames",
            "1",
            "--capture-frame",
            destination.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains(destination.to_str().unwrap()));
    fs::remove_dir(destination).unwrap();
}
