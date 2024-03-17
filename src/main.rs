// TODO: Change sound code and seperate it
// TODO: Reduce flickering
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
use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioDevice, AudioStatus};
use sdl2::Sdl;
pub mod cpu;
use cpu::*;
use std::env;
use std::thread;
use std::time::Duration;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;
    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

pub struct Sound {
    device: AudioDevice<SquareWave>,
}

impl Sound{
    pub fn new(sdl_context: &Sdl) -> Sound {
        let audio_subsystem = sdl_context.audio().unwrap();
        let desired_spec = AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        }).unwrap();
        Sound { device }
    }
}

pub fn play_sound(sound: &mut Sound) {
    match sound.device.status() {
        AudioStatus::Paused => sound.device.resume(),
        AudioStatus::Playing => {}
        AudioStatus::Stopped => sound.device.resume(),
    }
}

fn draw_screen(canvas: &mut Canvas<Window>, cpu: &Cpu) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
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
            )){
                Ok(_) => {},
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
    if args.len() != 2 {
        panic!("Usage: cargo run <filename>")
    }

    let mut cpu = Cpu::new();
    let path = &args[1];
    cpu.load(path);

    let sdl_context = sdl2::init().unwrap();
    let mut sound = Sound::new(&sdl_context);
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8", SCREEN_WIDTH as u32 * SCALE as u32, SCREEN_HEIGHT as u32 * SCALE as u32)
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
                Event::Quit { .. } => {break 'emuloop;},
            Event::KeyDown { keycode: Some(key), .. } => {
                 if let Some(k) = keycode(key) {
                     cpu.keypress(k, true);
                 }
             },
            Event::KeyUp { keycode: Some(key), .. } => {
                if let Some(k) = keycode(key) {
                    cpu.keypress(k, false);
                }
            },
                _ => {}
            }
        }
        for _ in 0..TICKS_PER_FRAME{
            cpu.tick();
            thread::sleep(Duration::from_millis(1));

        }
        draw_screen(&mut canvas, &cpu);
        if cpu.st > 0 {
            play_sound(&mut sound);
        } else {
            sound.device.pause();
        }
        cpu.timers();

    }
}
