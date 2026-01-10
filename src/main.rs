// CHIP-8 Emulator - chiki8
// ========================
// Kullanim: cargo run -- -f <rom_dosyasi> [secenekler]
//
// Secenekler:
//   -f, --file       ROM dosyasi (zorunlu)
//   -s, --scale      Piksel olcegi (varsayilan: 15)
//   -p, --speed      Hiz carpani (varsayilan: 10)
//   -v, --volume     Ses seviyesi 0-100 (varsayilan: 25)
//   -b, --background Arka plan rengi R,G,B (varsayilan: 0,0,0)
//   -c, --color      On plan rengi R,G,B (varsayilan: 255,255,255)
//   --config         Config dosyasi yolu
//   --create-config  Ornek config dosyasi olustur
//
// TODO: Add tests
//   - [x] cpu opcode testleri
//   - [x] timer testleri
//   - [x] display testleri

#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
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
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn print_keymap(keymap: &HashMap<String, u8>) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║                    CHIKI8 - CHIP-8 Emulator                ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  Klavye Haritasi:                                          ║");
    println!("║                                                            ║");
    println!("║  CHIP-8:          Klavye:                                  ║");
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
    println!("║  Cikmak icin: Pencereyi kapatin veya ESC                   ║");
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
            match canvas.fill_rect(Rect::new(
                x * scale as i32,
                y * scale as i32,
                scale,
                scale,
            )) {
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
    let mut iter = s.split(',').map(|num_str| num_str.trim().parse().unwrap_or(0));
    [
        iter.next().unwrap_or(0),
        iter.next().unwrap_or(0),
        iter.next().unwrap_or(0),
    ]
}

fn main() {
    // Wayland'de SDL2 event sorunlari oldugu icin x11 kullaniyoruz
    env::set_var("SDL_VIDEODRIVER", "x11");
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("f", "file", "ROM dosyasi", "FILE");
    opts.optopt("s", "scale", "Piksel olcegi (varsayilan: 15)", "SCALE");
    opts.optopt("p", "speed", "Hiz carpani (varsayilan: 10)", "SPEED");
    opts.optopt("v", "volume", "Ses seviyesi 0-100 (varsayilan: 25)", "VOLUME");
    opts.optopt(
        "b",
        "background",
        "Arka plan rengi R,G,B (varsayilan: 0,0,0)",
        "COLOR",
    );
    opts.optopt(
        "c",
        "color",
        "On plan rengi R,G,B (varsayilan: 255,255,255)",
        "COLOR",
    );
    opts.optopt("", "config", "Config dosyasi yolu", "PATH");
    opts.optflag("", "create-config", "Ornek config dosyasi olustur");
    opts.optflag("", "help", "Yardim mesajini goster");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f),
    };

    // Ornek config dosyasi olustur
    if matches.opt_present("create-config") {
        let config_path = Path::new("chiki8.toml");
        match config::create_example_config(config_path) {
            Ok(_) => {
                println!("Ornek config dosyasi olusturuldu: chiki8.toml");
                return;
            }
            Err(e) => {
                eprintln!("Config dosyasi olusturulamadi: {}", e);
                return;
            }
        }
    }

    // Config dosyasini yukle
    let config_path = matches.opt_str("config").unwrap_or_else(|| "chiki8.toml".to_string());
    let config = Config::load(Path::new(&config_path)).unwrap_or_else(|e| {
        eprintln!("Config yuklenemedi, varsayilan ayarlar kullaniliyor: {}", e);
        Config::default()
    });

    if matches.opt_present("help") {
        print_keymap(&config.keymap.keys);
        println!("Kullanim: {} -f <rom_dosyasi> [secenekler]", args[0]);
        println!("{}", opts.usage(""));
        return;
    }

    // Tus haritasini goster
    print_keymap(&config.keymap.keys);

    let file_path: String = matches
        .opt_str("f")
        .expect("Lutfen ROM dosyasi belirtin: -f <dosya_yolu>");

    // Komut satiri argumanlari config'den oncelikli
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
    println!("Olcek: {}x", scale);
    println!("Hiz: {}x", speed);
    println!("Ses: {}%", volume);
    println!(
        "Arka plan: RGB({}, {}, {})",
        background_color[0], background_color[1], background_color[2]
    );
    println!(
        "On plan: RGB({}, {}, {})",
        foreground_color[0], foreground_color[1], foreground_color[2]
    );
    if Path::new(&config_path).exists() {
        println!("Config: {}", config_path);
    }
    println!();

    let mut cpu = Cpu::new();
    cpu.load(&file_path);

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

    // 60Hz frame timing icin
    use std::time::Instant;
    let frame_duration = Duration::from_micros(16667); // ~60 FPS
    let _last_frame = Instant::now();

    'emuloop: loop {
        let frame_start = Instant::now();
        
        while let Some(event) = event_pump.poll_event() {
            match event {
                Event::Quit { .. } => {
                    break 'emuloop;
                }
                Event::KeyDown {
                    keycode: Some(key),
                    repeat: false, // Tuş tekrarını engelle
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
        
        // CPU ticks (speed kadar opcode calistir)
        for _ in 0..speed {
            cpu.tick();
        }
        
        // Ekrani ciz
        draw_screen(&mut canvas, &cpu, &background_color, &foreground_color, scale);
        
        // Timer'lari guncelle (60Hz)
        cpu.timers();
        
        // Ses kontrolu
        if cpu.st > 0 {
            play_sound(&mut sound);
        } else {
            sound.device.pause()
        }
        
        // Frame rate limiti
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }
}
