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
use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

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
    canvas.set_draw_color(Color::RGB(
        foreground_color[0],
        foreground_color[1],
        foreground_color[2],
    ));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as i32;
            let y = (i / SCREEN_WIDTH) as i32;
            match canvas.fill_rect(Rect::new(x * scale as i32, y * scale as i32, scale, scale)) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            }
        }
    }
    canvas.present();
}

fn keycode_to_string(key: Keycode) -> Option<String> {
    match key {
        Keycode::Num1 => Some("1".to_string()),
        Keycode::Num2 => Some("2".to_string()),
        Keycode::Num3 => Some("3".to_string()),
        Keycode::Num4 => Some("4".to_string()),
        Keycode::Num5 => Some("5".to_string()),
        Keycode::Num6 => Some("6".to_string()),
        Keycode::Num7 => Some("7".to_string()),
        Keycode::Num8 => Some("8".to_string()),
        Keycode::Num9 => Some("9".to_string()),
        Keycode::Num0 => Some("0".to_string()),
        Keycode::Q => Some("Q".to_string()),
        Keycode::W => Some("W".to_string()),
        Keycode::E => Some("E".to_string()),
        Keycode::R => Some("R".to_string()),
        Keycode::T => Some("T".to_string()),
        Keycode::Y => Some("Y".to_string()),
        Keycode::U => Some("U".to_string()),
        Keycode::I => Some("I".to_string()),
        Keycode::O => Some("O".to_string()),
        Keycode::P => Some("P".to_string()),
        Keycode::A => Some("A".to_string()),
        Keycode::S => Some("S".to_string()),
        Keycode::D => Some("D".to_string()),
        Keycode::F => Some("F".to_string()),
        Keycode::G => Some("G".to_string()),
        Keycode::H => Some("H".to_string()),
        Keycode::J => Some("J".to_string()),
        Keycode::K => Some("K".to_string()),
        Keycode::L => Some("L".to_string()),
        Keycode::Z => Some("Z".to_string()),
        Keycode::X => Some("X".to_string()),
        Keycode::C => Some("C".to_string()),
        Keycode::V => Some("V".to_string()),
        Keycode::B => Some("B".to_string()),
        Keycode::N => Some("N".to_string()),
        Keycode::M => Some("M".to_string()),
        _ => None,
    }
}

fn parse_color(s: &str) -> [u8; 3] {
    let mut iter = s
        .split(',')
        .map(|num_str| num_str.trim().parse().unwrap_or(0));
    [
        iter.next().unwrap_or(0),
        iter.next().unwrap_or(0),
        iter.next().unwrap_or(0),
    ]
}

fn main() {
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
    opts.optflag("", "create-config", "Create example config file");
    opts.optflag("", "help", "Show help message");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f),
    };

    if matches.opt_present("create-config") {
        let config_path = Path::new("chiki8.toml");
        match config::create_example_config(config_path) {
            Ok(_) => {
                println!("Example config file created: chiki8.toml");
                return;
            }
            Err(e) => {
                eprintln!("Could not create config file: {}", e);
                return;
            }
        }
    }

    let config_path = matches
        .opt_str("config")
        .unwrap_or_else(|| "chiki8.toml".to_string());
    let config = Config::load(Path::new(&config_path)).unwrap_or_else(|e| {
        eprintln!("Could not load config, using defaults: {}", e);
        Config::default()
    });

    if matches.opt_present("help") {
        print_keymap();
        println!("Usage: {} -f <rom_file> [options]", args[0]);
        println!("{}", opts.usage(""));
        return;
    }

    print_keymap();

    let file_path: String = matches
        .opt_str("f")
        .expect("Please specify ROM file: -f <file_path>");

    let scale: u32 = matches
        .opt_str("s")
        .map(|s| s.parse().unwrap_or(config.display.scale))
        .unwrap_or(config.display.scale);

    let speed: u32 = matches
        .opt_str("p")
        .map(|s| s.parse().unwrap_or(config.emulation.speed))
        .unwrap_or(config.emulation.speed);

    let volume: u32 = matches
        .opt_str("v")
        .map(|s| s.parse().unwrap_or(config.audio.volume))
        .unwrap_or(config.audio.volume);
    let volume_f32 = (volume.min(100) as f32) / 100.0;

    let background_color: [u8; 3] = matches
        .opt_str("b")
        .map(|s| parse_color(&s))
        .unwrap_or(config.display.background);

    let foreground_color: [u8; 3] = matches
        .opt_str("c")
        .map(|s| parse_color(&s))
        .unwrap_or(config.display.foreground);

    println!("ROM: {}", file_path);
    println!("Scale: {}x", scale);
    println!("Speed: {}x", speed);
    println!("Volume: {}%", volume);
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

    let mut cpu = Cpu::new();
    if let Err(error) = cpu.load(Path::new(&file_path)) {
        eprintln!("{error}");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let mut sound = Sound::new(&sdl_context, volume_f32);
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Chiki8 - CHIP-8 Emulator",
            SCREEN_WIDTH as u32 * scale,
            SCREEN_HEIGHT as u32 * scale,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    use std::time::Instant;
    let frame_duration = Duration::from_micros(16667);
    'emuloop: loop {
        let frame_start = Instant::now();

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
                    if let Some(key_str) = keycode_to_string(key) {
                        if let Some(k) = config.get_keycode(&key_str) {
                            cpu.keypress(k, true);
                        }
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(key_str) = keycode_to_string(key) {
                        if let Some(k) = config.get_keycode(&key_str) {
                            cpu.keypress(k, false);
                        }
                    }
                }
                _ => {}
            }
        }

        for _ in 0..speed {
            cpu.tick();
        }

        draw_screen(
            &mut canvas,
            &cpu,
            &background_color,
            &foreground_color,
            scale,
        );

        cpu.timers();

        if cpu.st > 0 {
            play_sound(&mut sound);
        } else {
            sound.device.pause()
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
}
