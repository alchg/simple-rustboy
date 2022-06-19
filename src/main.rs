extern crate sdl2;
use sdl2::audio::AudioQueue;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;

use std::time;

mod dmg01cpu;

pub enum LogMode {
    INFO = 0x01,
    CPU = 0x02,
    IO = 0x04,
    ROM = 0x10,
    PPU = 0x20,
    APU = 0x40,
}

pub struct Log {
    pub mode: u8,
}

impl Log {
    fn check_mode(mode: u8, value: LogMode) -> bool {
        if mode & value as u8 > 0 {
            true
        } else {
            false
        }
    }

    pub fn info(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::INFO) {
            println!("{}", text);
        }
    }
    pub fn cpu(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::CPU) {
            println!("{}", text);
        }
    }
    pub fn io(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::IO) {
            println!("IO::{}", text);
        }
    }
    pub fn rom(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::ROM) {
            println!("ROM::{}", text);
        }
    }
    pub fn ppu(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::PPU) {
            println!("PPU::{}", text);
        }
    }
    pub fn apu(text: String, mode: u8) {
        if Self::check_mode(mode, LogMode::APU) {
            println!("APU::{}", text);
        }
    }
}

pub struct Common {}
impl Common {
    pub const SAMPLE_RATE: u32 = 44100;

    /*
    fn is_bit_n_on(value: u8, bit: u8) -> bool {
        let result: u8 = match bit {
            0 => value & 0x01,
            1 => value & 0x02,
            2 => value & 0x04,
            3 => value & 0x08,
            4 => value & 0x10,
            5 => value & 0x20,
            6 => value & 0x40,
            7 => value & 0x80,
            _ => panic!("unexpected bit {}", bit),
        };
        if result == 0x00 {
            false
        } else {
            true
        }
    }
    */
}

fn main() {
    let mut log_mode = 0;
    let romfile: String; // rom file path
    let mut system: dmg01cpu::Dmg01Cpu;

    println!("A Game Boy emulator in Rust.");

    let args: std::env::Args = std::env::args();
    if 1 < args.len() && args.len() < 4 {
        romfile = std::env::args().nth(1).unwrap();
        println!("ROM :{}", romfile);

        if args.len() == 3 {
            let value: String = std::env::args().nth(2).unwrap();
            if value.len() == 2 {
                match value.parse::<u8>() {
                    Ok(_) => {
                        let mode0: u8 = value.chars().nth(1).unwrap().to_string().parse().unwrap();
                        let mode1: u8 = value.chars().nth(0).unwrap().to_string().parse().unwrap();
                        log_mode = mode1 * 16 + mode0;
                        println!("DEBUG MODE {:#04x}", log_mode);
                    }
                    Err(_) => (),
                };
            }
        }

        system = dmg01cpu::Dmg01Cpu::new(log_mode, romfile);
    } else {
        println!("Usage:simple-rustboy <ROM file path>");
        std::process::exit(1);
    }

    let sdl: sdl2::Sdl = match sdl2::init() {
        Ok(result) => result,
        Err(error) => panic!("sdl2 init error:{}", error),
    };
    let video: sdl2::VideoSubsystem = match sdl.video() {
        Ok(result) => result,
        Err(error) => panic!("sdl2 video error:{}", error),
    };
    let window: sdl2::video::Window = match video
        .window("simple-rustboy", 320, 288)
        .position_centered()
        .build()
    {
        Ok(result) => result,
        Err(error) => panic!("sdl2 window error:{}", error),
    };
    let mut canvas: sdl2::render::Canvas<sdl2::video::Window> = match window.into_canvas().build() {
        Ok(result) => result,
        Err(error) => panic!("sdl2 canvas error:{}", error),
    };
    let texture_creator: sdl2::render::TextureCreator<sdl2::video::WindowContext> =
        canvas.texture_creator();
    let mut texture =
        match texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, 160, 144) {
            Ok(result) => result,
            Err(error) => panic!("sdl2 create_texture_streaming error:{}", error),
        };

    let mut events: sdl2::EventPump = match sdl.event_pump() {
        Ok(result) => result,
        Err(error) => panic!("sdl2 event_pump error:{}", error),
    };

    let audio: sdl2::AudioSubsystem = match sdl.audio() {
        Ok(result) => result,
        Err(error) => panic!("sdl2 video error:{}", error),
    };

    let audio_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(Common::SAMPLE_RATE as i32),
        channels: Some(1),
        samples: None,
    };

    let audio_queue: AudioQueue<i16> = match audio.open_queue::<i16, _>(None, &audio_spec) {
        Ok(result) => result,
        Err(error) => panic!("sdl2 video error:{}", error),
    };

    let mut apu_correction = 0;
    const AUDIO_BUFFER: u32 = 7350;

    audio_queue.resume();

    let wait: time::Duration = time::Duration::from_micros(1000000 / 60);
    'running: loop {
        let start: time::Instant = time::Instant::now();
        let mut cycle: u32 = 0;

        // cpu clock 4.194304 MHz
        while cycle < 4194304 / 60 {
            cycle += system.execute() as u32;
        }

        let wave = system.apu.execute(apu_correction);
        let queue_size = audio_queue.size();
        if queue_size == 0 {
            apu_correction += 25;
        } else if queue_size < AUDIO_BUFFER {
            apu_correction += 2;
        } else if queue_size > AUDIO_BUFFER {
            if apu_correction > 1 {
                apu_correction -= 2;
            } else {
                apu_correction = 0;
            }
        }
        audio_queue.queue_audio(&wave).unwrap();

        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let frame_buffer = system.ppu.get_frame_buffer();

                for y in 0..144 {
                    for x in 0..160 {
                        let offset = y * pitch + x * 3;
                        let color = frame_buffer[y * 160 + x];

                        buffer[offset] = color.saturating_sub(25);
                        buffer[offset + 1] = color;
                        buffer[offset + 2] = color.saturating_sub(25);
                    }
                }
            })
            .unwrap();

        //canvas.set_draw_color(sdl2::pixels::Color::RGB(175, 200, 175));
        canvas.clear();
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(sdl2::keyboard::Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => system.joypad.keydown(keycode),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => system.joypad.keyup(keycode),
                _ => (),
            }
        }

        let elapsed: time::Duration = start.elapsed();
        if elapsed < wait {
            std::thread::sleep(wait - elapsed);
        }
    }

    std::process::exit(0);
}
