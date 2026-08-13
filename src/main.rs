// CHIP-8 Emulator - chiki8
// ========================
// Usage: cargo run -- -f <rom_file> [options]
//
// Options:
//   -f, --file       ROM file (required)
//   -s, --scale      Pixel scale (default: 15)
//   -p, --speed      Speed multiplier (default: 10)
//   -v, --volume     Volume level 0-100 (default: 25)
//   -b, --background Background color R,G,B (default: 0,0,0)
//   -c, --color      Foreground color R,G,B (default: 255,255,255)
//   --config         Config file path
//   --create-config  Create example config file
//
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
pub mod audio;
pub mod config;
pub mod cpu;
use audio::*;
use config::Config;
use cpu::*;
use getopts::Options;
use sha2::{Digest, Sha256};
use std::env;
use std::ops::AddAssign;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

const MAX_CATCH_UP: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Work {
    cpu_steps: u32,
    timer_ticks: u32,
    presentations: u32,
}

impl AddAssign for Work {
    fn add_assign(&mut self, other: Self) {
        self.cpu_steps += other.cpu_steps;
        self.timer_ticks += other.timer_ticks;
        self.presentations += other.presentations;
    }
}

struct Scheduler {
    cpu_rate: u64,
    presentation_rate: u64,
    cpu_debt: u64,
    timer_debt: u64,
    presentation_debt: u64,
}

impl Scheduler {
    fn new(cpu_rate: u32, presentation_rate: u32) -> Self {
        Self {
            cpu_rate: cpu_rate.into(),
            presentation_rate: presentation_rate.into(),
            cpu_debt: 0,
            timer_debt: 0,
            presentation_debt: 0,
        }
    }

    fn advance(&mut self, elapsed: Duration) -> Work {
        const SECOND_NANOS: u64 = 1_000_000_000;
        let nanos = elapsed.min(MAX_CATCH_UP).as_nanos() as u64;
        self.cpu_debt += nanos * self.cpu_rate;
        self.timer_debt += nanos * 60;
        self.presentation_debt += nanos * self.presentation_rate;
        let work = Work {
            cpu_steps: (self.cpu_debt / SECOND_NANOS) as u32,
            timer_ticks: (self.timer_debt / SECOND_NANOS) as u32,
            presentations: (self.presentation_debt / SECOND_NANOS) as u32,
        };
        self.cpu_debt %= SECOND_NANOS;
        self.timer_debt %= SECOND_NANOS;
        self.presentation_debt %= SECOND_NANOS;
        work
    }
}

fn parse_frames(value: &str) -> Result<u64, String> {
    value
        .parse::<u64>()
        .ok()
        .filter(|frames| *frames > 0)
        .ok_or_else(|| "CLI frames: expected a positive integer".to_string())
}

fn print_keymap() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    CHIKI8 - CHIP-8 Emulator                ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Keyboard Mapping:                                         ║");
    println!("║                                                            ║");
    println!("║  CHIP-8:          Keyboard:                                ║");
    println!("║  ┌───┬───┬───┬───┐    ┌───┬───┬───┬───┐                    ║");
    println!("║  │ 1 │ 2 │ 3 │ C │    │ 1 │ 2 │ 3 │ 4 │                    ║");
    println!("║  ├───┼───┼───┼───┤    ├───┼───┼───┼───┤                    ║");
    println!("║  │ 4 │ 5 │ 6 │ D │    │ Q │ W │ E │ R │                    ║");
    println!("║  ├───┼───┼───┼───┤    ├───┼───┼───┼───┤                    ║");
    println!("║  │ 7 │ 8 │ 9 │ E │    │ A │ S │ D │ F │                    ║");
    println!("║  ├───┼───┼───┼───┤    ├───┼───┼───┼───┤                    ║");
    println!("║  │ A │ 0 │ B │ F │    │ Z │ X │ C │ V │                    ║");
    println!("║  └───┴───┴───┴───┘    └───┴───┴───┴───┘                    ║");
    println!("║                                                            ║");
    println!("║  To exit: Close the window or press ESC                    ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}

fn draw_screen(
    canvas: &mut Canvas<Window>,
    cpu: &Cpu,
    background_color: &[u8; 3],
    foreground_color: &[u8; 3],
    scale: u32,
) {
    canvas.set_draw_color(Color::RGB(
        background_color[0],
        background_color[1],
        background_color[2],
    ));
    canvas.clear();
    let screen_buf = cpu.get_display();
    let (width, height) = cpu.active_dimensions();
    let backing_scale = PHYSICAL_SCREEN_WIDTH / width;
    canvas.set_draw_color(Color::RGB(
        foreground_color[0],
        foreground_color[1],
        foreground_color[2],
    ));
    for y in 0..height {
        for x in 0..width {
            let pixel = screen_buf[x * backing_scale + y * backing_scale * PHYSICAL_SCREEN_WIDTH];
            if pixel {
                match canvas.fill_rect(Rect::new(
                    x as i32 * scale as i32,
                    y as i32 * scale as i32,
                    scale,
                    scale,
                )) {
                    Ok(_) => {}
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
    }
    canvas.present();
}

fn step_outcome_controls_loop(outcome: StepOutcome) -> bool {
    outcome == StepOutcome::Executed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProfileSource {
    Cli,
    Toml,
    KnownRom,
    Fallback,
}

struct ProfileResolution {
    profile: Profile,
    source: ProfileSource,
}

const TIMENDUS_SCROLLING_SHA256: [u8; 32] = [
    63, 67, 80, 124, 69, 169, 73, 229, 176, 20, 68, 88, 83, 32, 93, 209, 243, 107, 181, 50, 207,
    37, 186, 162, 34, 9, 183, 195, 0, 197, 150, 215,
];

fn resolve_profile(
    explicit: Option<Profile>,
    configured: Option<Profile>,
    rom: &[u8],
) -> ProfileResolution {
    if let Some(profile) = explicit {
        return ProfileResolution {
            profile,
            source: ProfileSource::Cli,
        };
    }
    if let Some(profile) = configured {
        return ProfileResolution {
            profile,
            source: ProfileSource::Toml,
        };
    }
    // Timendus CHIP-8 test suite v4.2, commit cb24d5595384a80b49ddedae13bec4042b16d41d,
    // bin/8-scrolling.ch8. This is a selector mapping, not conformance evidence.
    if profile_for_digest(Sha256::digest(rom).as_slice()).is_some() {
        return ProfileResolution {
            profile: Profile::SuperChip11,
            source: ProfileSource::KnownRom,
        };
    }
    ProfileResolution {
        profile: Profile::Classic,
        source: ProfileSource::Fallback,
    }
}

fn profile_for_digest(digest: &[u8]) -> Option<Profile> {
    (digest == TIMENDUS_SCROLLING_SHA256).then_some(Profile::SuperChip11)
}

fn profile_name(profile: Profile) -> &'static str {
    match profile {
        Profile::Classic => "classic",
        Profile::SuperChip11 => "superchip-1.1",
    }
}

fn profile_diagnostic(resolution: &ProfileResolution) -> String {
    let profile = profile_name(resolution.profile);
    match resolution.source {
        ProfileSource::Cli => format!("Profile: {profile} (source: cli)"),
        ProfileSource::Toml => format!("Profile: {profile} (source: toml)"),
        ProfileSource::KnownRom => format!("Profile: {profile} (source: known-rom)"),
        ProfileSource::Fallback => format!(
            "Profile: {profile} (source: fallback; reason: no explicit profile or known ROM hash match)"
        ),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("f", "file", "ROM file", "FILE");
    opts.optopt("s", "scale", "Pixel scale (default: 15)", "SCALE");
    opts.optopt("p", "speed", "Speed multiplier (default: 10)", "SPEED");
    opts.optopt("v", "volume", "Volume level 0-100 (default: 25)", "VOLUME");
    opts.optopt(
        "b",
        "background",
        "Background color R,G,B (default: 0,0,0)",
        "COLOR",
    );
    opts.optopt(
        "c",
        "color",
        "Foreground color R,G,B (default: 255,255,255)",
        "COLOR",
    );
    opts.optopt("", "config", "Config file path", "PATH");
    opts.optopt("", "refresh", "Refresh rate (30, 60, or 120)", "HZ");
    opts.optopt("", "filter", "Texture filter (nearest or linear)", "FILTER");
    opts.optflag("", "integer-scaling", "Use native SDL integer scaling");
    opts.optopt("", "frames", "Stop after N presented frames", "N");
    opts.optopt(
        "",
        "profile",
        "Execution profile (default: classic)",
        "PROFILE",
    );
    opts.optflag("", "create-config", "Create example config file");
    opts.optflag("", "help", "Show help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(error) => {
            eprintln!("CLI: {error}");
            std::process::exit(2);
        }
    };

    let explicit_profile = match matches.opt_str("profile") {
        Some(value) => match value.parse() {
            Ok(profile) => Some(profile),
            Err(error) => {
                eprintln!("CLI emulation.profile: {error}");
                std::process::exit(2);
            }
        },
        None => None,
    };

    if matches.opt_present("create-config") {
        let config_path = Path::new("chiki8.toml");
        match config::create_example_config(config_path) {
            Ok(_) => {
                println!("Example config file created: chiki8.toml");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Could not create config file: {}", e);
                std::process::exit(2);
            }
        }
    }

    if matches.opt_present("help") {
        print_keymap();
        println!("Usage: {} -f <rom_file> [options]", args[0]);
        println!("{}", opts.usage(""));
        return Ok(());
    }

    let explicit_config = matches.opt_str("config");
    let config_path = explicit_config
        .clone()
        .unwrap_or_else(|| "chiki8.toml".to_string());
    let config = match explicit_config {
        Some(_) => Config::load_required(Path::new(&config_path)),
        None => Config::load(Path::new(&config_path)),
    }
    .unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });

    print_keymap();

    let cli_value = |name: &str, field: &str, fallback: u32, min: u32, max: u32| -> u32 {
        matches
            .opt_str(name)
            .map(|value| config::parse_bounded_u32(field, &value, min, max))
            .transpose()
            .unwrap_or_else(|error| {
                eprintln!("{error}");
                std::process::exit(2);
            })
            .unwrap_or(fallback)
    };

    let scale = cli_value("scale", "CLI display.scale", config.display.scale, 1, 64);
    let speed = cli_value(
        "speed",
        "CLI emulation.speed",
        config.emulation.speed,
        1,
        1000,
    );
    let volume = cli_value("volume", "CLI audio.volume", config.audio.volume, 0, 100);
    let background_color = matches
        .opt_str("background")
        .map(|value| config::parse_color("CLI display.background", &value))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        })
        .unwrap_or(config.display.background);
    let foreground_color = matches
        .opt_str("color")
        .map(|value| config::parse_color("CLI display.foreground", &value))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        })
        .unwrap_or(config.display.foreground);
    let refresh = matches
        .opt_str("refresh")
        .map(|value| config::parse_refresh("CLI emulation.refresh", &value))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        })
        .unwrap_or(config.emulation.refresh);
    let filter = matches
        .opt_str("filter")
        .map(|value| {
            value
                .parse::<config::Filter>()
                .map_err(|error| format!("CLI display.filter: {error}"))
        })
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        })
        .unwrap_or_else(|| {
            config
                .display
                .filter
                .parse()
                .expect("validated config filter")
        });
    let frame_budget = matches
        .opt_str("frames")
        .map(|value| parse_frames(&value))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(2);
        });
    let integer_scaling = matches.opt_present("integer-scaling") || config.display.integer_scaling;

    let file_path: String = match matches.opt_str("f") {
        Some(path) => path,
        None => {
            eprintln!("CLI file: Please specify ROM file: -f <file_path>");
            std::process::exit(2);
        }
    };
    let rom = match read_rom(Path::new(&file_path)) {
        Ok(rom) => rom,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let configured_profile = config
        .emulation
        .profile
        .as_deref()
        .map(str::parse)
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("Config '{}' field emulation.profile: {error}", config_path);
            std::process::exit(2);
        });
    let resolution = resolve_profile(explicit_profile, configured_profile, &rom);
    let mut cpu = Cpu::new(resolution.profile);
    if let Err(error) = cpu.load_rom(&rom) {
        eprintln!("{error}");
        std::process::exit(2);
    }
    println!("{}", profile_diagnostic(&resolution));

    let volume_f32 = (volume as f32) / 100.0;

    println!("ROM: {}", file_path);
    println!("Scale: {}x", scale);
    println!("Speed: {}x", speed);
    println!("Volume: {}%", volume);
    println!("Refresh: {refresh} Hz");
    println!(
        "Filter: {}",
        match filter {
            config::Filter::Nearest => "nearest",
            config::Filter::Linear => "linear",
        }
    );
    println!(
        "Background: RGB({}, {}, {})",
        background_color[0], background_color[1], background_color[2]
    );
    println!(
        "Foreground: RGB({}, {}, {})",
        foreground_color[0], foreground_color[1], foreground_color[2]
    );
    if Path::new(&config_path).exists() {
        println!("Config: {}", config_path);
    }
    println!();

    let sdl_context = sdl2::init().map_err(|error| format!("SDL initialization failed: {error}"))?;
    let mut sound = Sound::new(&sdl_context, volume_f32);
    let video_subsystem = sdl_context
        .video()
        .map_err(|error| format!("SDL video initialization failed: {error}"))?;
    let window = video_subsystem
        .window(
            "Chiki8 - CHIP-8 Emulator",
            SCREEN_WIDTH as u32 * scale,
            SCREEN_HEIGHT as u32 * scale,
        )
        .position_centered()
        .build()
        .map_err(|error| format!("SDL window creation failed: {error}"))?;
    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|error| format!("SDL canvas creation failed: {error}"))?;
    canvas
        .set_integer_scale(integer_scaling)
        .map_err(|error| format!("SDL integer scaling ({integer_scaling}) failed: {error}"))?;
    println!(
        "Integer scaling: {}",
        if integer_scaling { "enabled" } else { "disabled" }
    );
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| format!("SDL event initialization failed: {error}"))?;

    let mut scheduler = Scheduler::new(speed * 60, refresh);
    let mut previous = Instant::now();
    let mut presented = 0_u64;
    'emuloop: loop {
        let now = Instant::now();
        let work = scheduler.advance(now.duration_since(previous));
        previous = now;

        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => {
                    break 'emuloop;
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false,
                    ..
                } => {
                    if key == Keycode::Escape {
                        break 'emuloop;
                    }
                    if let Some(k) = config.get_keycode(key) {
                        cpu.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = config.get_keycode(key) {
                        cpu.keypress(k, false);
                    }
                }
                _ => {}
            }
        }

        for _ in 0..work.cpu_steps {
            let outcome = cpu.tick();
            if !step_outcome_controls_loop(outcome) {
                if outcome == StepOutcome::Unsupported {
                    eprintln!("Unsupported opcode");
                }
                break 'emuloop;
            }
        }

        for _ in 0..work.timer_ticks {
            cpu.timers();
        }

        if cpu.st > 0 {
            play_sound(&mut sound);
        } else {
            sound.device.pause()
        }

        for _ in 0..work.presentations {
            draw_screen(
                &mut canvas,
                &cpu,
                &background_color,
                &foreground_color,
                scale,
            );
            presented += 1;
            if frame_budget.is_some_and(|budget| presented >= budget) {
                break 'emuloop;
            }
        }
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduler_keeps_cpu_and_timers_independent_from_presentation() {
        for refresh in [30, 60, 120] {
            let mut scheduler = Scheduler::new(600, refresh);
            let mut totals = Work::default();
            let base = 1_000_000_000 / refresh as u64;
            let remainder = 1_000_000_000 % refresh as u64;
            for frame in 0..refresh {
                totals += scheduler.advance(Duration::from_nanos(
                    base + u64::from(frame < remainder as u32),
                ));
            }
            assert_eq!(totals.cpu_steps, 600, "refresh {refresh}");
            assert_eq!(totals.timer_ticks, 60, "refresh {refresh}");
            assert_eq!(totals.presentations, refresh, "refresh {refresh}");
        }
    }

    #[test]
    fn scheduler_carries_fractional_debt_and_clamps_stalls() {
        let mut scheduler = Scheduler::new(600, 60);
        let first = scheduler.advance(Duration::from_micros(833));
        let second = scheduler.advance(Duration::from_micros(834));
        assert_eq!(first.cpu_steps, 0);
        assert_eq!(second.cpu_steps, 1);

        let stalled = scheduler.advance(Duration::from_secs(60));
        assert!(stalled.cpu_steps <= 600 * MAX_CATCH_UP.as_secs() as u32);
        assert!(stalled.timer_ticks <= 60 * MAX_CATCH_UP.as_secs() as u32);
    }

    #[test]
    fn frames_require_a_positive_integer() {
        assert_eq!(parse_frames("1"), Ok(1));
        assert!(parse_frames("0").unwrap_err().contains("CLI frames"));
        assert!(parse_frames("nope").unwrap_err().contains("CLI frames"));
    }

    #[test]
    fn display_pixels_pack_low_resolution_backing_cells() {
        let mut display = [false; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT];
        display[2 + 4 * PHYSICAL_SCREEN_WIDTH] = true;
        let pixels = display_pixels(&display, (64, 32), &[1, 2, 3], &[4, 5, 6]);
        assert_eq!(pixels.len(), 64 * 32 * 3);
        assert_eq!(&pixels[(1 + 2 * 64) * 3..][..3], &[4, 5, 6]);
        assert_eq!(&pixels[..3], &[1, 2, 3]);
    }

    #[test]
    fn display_pixels_pack_every_high_resolution_pixel() {
        let mut display = [false; PHYSICAL_SCREEN_WIDTH * PHYSICAL_SCREEN_HEIGHT];
        display[127 + 63 * PHYSICAL_SCREEN_WIDTH] = true;
        let pixels = display_pixels(&display, (128, 64), &[0, 0, 0], &[9, 8, 7]);
        assert_eq!(pixels.len(), 128 * 64 * 3);
        assert_eq!(&pixels[pixels.len() - 3..], &[9, 8, 7]);
    }

    #[test]
    fn step_outcome_controls_loop_contract() {
        assert!(step_outcome_controls_loop(StepOutcome::Executed));
        assert!(!step_outcome_controls_loop(StepOutcome::Halted));
        assert!(!step_outcome_controls_loop(StepOutcome::Unsupported));
    }

    #[test]
    fn profile_resolution_known_digest_mapping() {
        assert_eq!(
            profile_for_digest(&TIMENDUS_SCROLLING_SHA256),
            Some(Profile::SuperChip11)
        );
        assert_eq!(profile_for_digest(&[0; 32]), None);
    }
}
