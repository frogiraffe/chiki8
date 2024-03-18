// TODO: Add args for screen size, scaling, speed, sound
// TODO: Add tests
// TODO : Add a way to change keymaps
// TODO: Add a way to change color
#![allow(dead_code)]
#![allow(unused_imports)]
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
pub mod audio;
pub mod cpu;
use audio::*;
use cpu::*;
use getopts::Options;
use std::env;
use std::thread;
use std::time::Duration;

fn draw_screen(canvas: &mut Canvas<Window>, cpu: &Cpu) {
    canvas.set_draw_color(Color::RGB(
        background_color.0,
        background_color.1,
        background_color.2,
    ));
    canvas.clear();
    let screen_buf = cpu.get_display();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as i32;
            let y = (i / SCREEN_WIDTH) as i32;
            match canvas.fill_rect(Rect::new(
                x * SCALE as i32,
                y * SCALE as i32,
                SCALE as u32,
                SCALE as u32,
            )) {
                Ok(_) => {}
                Err(e) => println!("Error: {}", e),
            }
        }
    }
    canvas.present();
}

fn keycode(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

const TICKS_PER_FRAME: u32 = 10;

fn main() {
    env::set_var("SDL_VIDEODRIVER", "wayland");
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("f", "file", "set the file", "FILE");
    opts.optopt("s", "scale", "set the scale", "SCALE");
    opts.optopt("w", "width", "set the width", "WIDTH");
    opts.optopt("h", "height", "set the height", "HEIGHT");
    opts.optopt(
        "b",
        "background",
        "set the background color as a comma-separated string (e.g., '255,255,255')",
        "BACKGROUND",
    );

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let file_path: String = matches.opt_str("f").unwrap().to_string();
    let screen_width: u32 = matches
        .opt_str("w")
        .unwrap_or("640".to_string())
        .parse()
        .unwrap();
    let screen_height: u32 = matches
        .opt_str("h")
        .unwrap_or("480".to_string())
        .parse()
        .unwrap();
    let scale: u32 = matches
        .opt_str("s")
        .unwrap_or("1".to_string())
        .parse()
        .unwrap();
    let background_color: [u8; 3] = matches
        .opt_str("b")
        .map(|s| {
            let mut iter = s.split(',').map(|num_str| num_str.parse().unwrap_or(0));
            [
                iter.next().unwrap_or(0),
                iter.next().unwrap_or(0),
                iter.next().unwrap_or(0),
            ]
        })
        .unwrap_or([0, 0, 0]); // Default to black if no color is provided

    let mut cpu = Cpu::new();
    let path = &args[1];
    cpu.load(path);

    let sdl_context = sdl2::init().unwrap();
    let mut sound = Sound::new(&sdl_context);
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Chip-8",
            SCREEN_WIDTH as u32 * SCALE as u32,
            SCREEN_HEIGHT as u32 * SCALE as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'emuloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'emuloop;
                }
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = keycode(key) {
                        cpu.keypress(k, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(k) = keycode(key) {
                        cpu.keypress(k, false);
                    }
                }
                _ => {}
            }
        }
        for _ in 0..TICKS_PER_FRAME {
            cpu.tick();
            thread::sleep(Duration::from_millis(2));
        }
        // draw_screen(&mut canvas, &cpu);
        if cpu.st > 0 {
            play_sound(&mut sound);
        } else {
            sound.device.pause()
        }
        cpu.timers();
    }
}
