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

fn temp_dir() -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "chiki8-config-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    fs::create_dir(&path).unwrap();
    path
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("Profile: superchip-1.1 (source: cli)")
    );
}

#[test]
fn profile_resolution_unknown_rom_falls_back_before_sdl() {
    let rom = rom(&[0x00, 0xe0]);
    let config = rom.with_extension("toml");
    fs::write(&config, "").unwrap();
    let output = run(&[
        "--config",
        config.to_str().unwrap(),
        "--file",
        rom.to_str().unwrap(),
    ]);
    fs::remove_file(rom).unwrap();
    fs::remove_file(config).unwrap();

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

#[test]
fn rom_preflight_rejects_oversized_rom_before_sdl() {
    let rom = rom(&vec![0xaa; 4096 - 0x200 + 1]);
    let output = run(&["--file", rom.to_str().unwrap()]);
    fs::remove_file(rom).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("maximum supported size"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("chiki8-invalid-driver"));
}

#[test]
fn config_preflight_toml_profile_precedence_and_cli_override() {
    let dir = temp_dir();
    let rom = dir.join("rom.ch8");
    let config = dir.join("config.toml");
    fs::write(&rom, [0x00, 0xfd]).unwrap();
    fs::write(&config, "[emulation]\nprofile = 'superchip-1.1'\n").unwrap();

    let toml = run(&[
        "--config",
        config.to_str().unwrap(),
        "--file",
        rom.to_str().unwrap(),
    ]);
    assert!(String::from_utf8_lossy(&toml.stdout).contains("Profile: superchip-1.1 (source: toml)"));

    let cli = run(&[
        "--config",
        config.to_str().unwrap(),
        "--profile",
        "classic",
        "--file",
        rom.to_str().unwrap(),
    ]);
    assert!(String::from_utf8_lossy(&cli.stdout).contains("Profile: classic (source: cli)"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn config_preflight_explicit_missing_is_status_two() {
    let output = run(&[
        "--config",
        "/definitely/not/a/chiki8-config.toml",
        "--file",
        "/also/not/read",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Config"));
}

#[test]
fn config_preflight_implicit_absence_uses_defaults() {
    let dir = temp_dir();
    let rom = dir.join("rom.ch8");
    fs::write(&rom, [0x00, 0xe0]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_chiki8"))
        .args(["--file", rom.to_str().unwrap()])
        .current_dir(&dir)
        .env("SDL_VIDEODRIVER", "chiki8-invalid-driver")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&output.stdout).contains("source: fallback"));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_preflight_rejects_every_invalid_override_before_sdl() {
    let dir = temp_dir();
    let rom = dir.join("rom.ch8");
    let config = dir.join("config.toml");
    fs::write(&rom, [0x00, 0xe0]).unwrap();
    fs::write(&config, "").unwrap();
    let base = [
        "--config",
        config.to_str().unwrap(),
        "--file",
        rom.to_str().unwrap(),
    ];
    for (flag, value, field) in [
        ("--scale", "0", "CLI display.scale"),
        ("--speed", "0", "CLI emulation.speed"),
        ("--volume", "101", "CLI audio.volume"),
        ("--background", "1,2", "CLI display.background"),
        ("--color", "1,2,999", "CLI display.foreground"),
        ("--refresh", "59", "CLI emulation.refresh"),
        ("--filter", "blur", "CLI display.filter"),
        ("--profile", "invalid", "CLI emulation.profile"),
    ] {
        let output = run(&[base[0], base[1], base[2], base[3], flag, value]);
        assert_eq!(output.status.code(), Some(2), "{flag}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(field),
            "{flag}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn cli_preflight_valid_overrides_win_over_toml() {
    let dir = temp_dir();
    let rom = dir.join("rom.ch8");
    let config = dir.join("config.toml");
    fs::write(&rom, [0x00, 0xe0]).unwrap();
    fs::write(
        &config,
        "[display]\nscale = 2\nbackground = [1, 2, 3]\nforeground = [4, 5, 6]\nfilter = 'nearest'\n[audio]\nvolume = 7\n[emulation]\nspeed = 8\nrefresh = 30\nprofile = 'superchip-1.1'\n",
    )
    .unwrap();
    let output = run(&[
        "--config",
        config.to_str().unwrap(),
        "--file",
        rom.to_str().unwrap(),
        "--scale",
        "3",
        "--speed",
        "9",
        "--volume",
        "10",
        "--background",
        "11,12,13",
        "--color",
        "14,15,16",
        "--profile",
        "classic",
        "--refresh",
        "120",
        "--filter",
        "linear",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Profile: classic (source: cli)"));
    assert!(stdout.contains("Scale: 3x"));
    assert!(stdout.contains("Speed: 9x"));
    assert!(stdout.contains("Volume: 10%"));
    assert!(stdout.contains("Background: RGB(11, 12, 13)"));
    assert!(stdout.contains("Foreground: RGB(14, 15, 16)"));
    assert!(stdout.contains("Refresh: 120 Hz"));
    assert!(stdout.contains("Filter: linear"));
    fs::remove_dir_all(dir).unwrap();
}
