use crate::cpu::Cpu;
use sdl2::pixels::PixelFormatEnum;
use std::{env, mem::size_of};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

mod cpu;

const VIDEO_WIDTH: usize = 64;
const VIDEO_HEIGHT: usize = 32;

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        println!("Usage: {} <Scale> <Delay> <ROM>", args[0]);
    }
    let scale = str::parse::<usize>(&args[1]).map_err(|e| e.to_string())?;
    let delay = str::parse::<usize>(&args[2]).map_err(|e| e.to_string())?;
    let rom_path = &args[3];

    let sdl_context = sdl2::init()?;
    let window = sdl_context
        .video()?
        .window("Chip8 Emulator", (VIDEO_WIDTH * scale) as u32, (VIDEO_HEIGHT * scale) as u32)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let mut cpu = Cpu::new(rom_path);
    let texture_creator = canvas.texture_creator();

    let mut event_pump = sdl_context.event_pump()?;

    let mut last_cycle_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    'running: loop {
        if cpu.process_input(&mut event_pump) {
            break 'running;
        }

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let dt = current_time - last_cycle_time;

        if dt > Duration::from_millis(delay as u64) {
            last_cycle_time = current_time;

            cpu.cycle();
            let mut texture = texture_creator
                .create_texture_streaming(PixelFormatEnum::RGBA8888, 64, 32)
                .map_err(|e| e.to_string())?;
            // Create a red-green gradient
            texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                let video = cpu.video;
                for y in 0..VIDEO_HEIGHT {
                    for x in 0..VIDEO_WIDTH {
                        let offset = y * pitch + x * 4;
                        let v = video[y * VIDEO_WIDTH + x];
                        let r= ((v >> 24) & 0xFF) as u8;
                        let g= ((v >> 16) & 0xFF) as u8;
                        let b= ((v >> 8) & 0xFF) as u8;
                        let a= ((v) & 0xFF) as u8;

                        buffer[offset] = r;
                        buffer[offset + 1] = g;
                        buffer[offset + 2] = b;
                        buffer[offset + 3] = a;
                    }
                }
            })?;

            canvas.clear();
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }
    }
    Ok(())
}

fn process_input(keypad: &[u8; 16]) -> bool {
    false
}
