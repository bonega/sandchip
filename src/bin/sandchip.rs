use sandchiplib::cpu::CPU;
use sandchiplib::display::Display;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::cmp::max;
use std::env;
use std::fs;
use std::thread;

const OP_BUNDLE_SIZE: usize = 10;
const HZ: usize = 400;
const MS_TARGET: usize = (1000 as f32 / HZ as f32 * OP_BUNDLE_SIZE as f32) as usize;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide path to a chip8 rom as an argument");
        std::process::exit(1);
    }
    let filename = &args[1];
    let data = fs::read(filename).unwrap_or_else(|_| {
        println!("No such file {:?}", filename);
        std::process::exit(1);
    });
    let mut cpu = CPU::new();
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context);
    let mut keys_pressed = Vec::new();
    let mut event_pump = sdl_context.event_pump().unwrap();
    cpu.load_rom(data);
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {}
                Event::KeyDown { keycode, .. } => {
                    if keys_pressed.contains(&keycode) {
                        continue;
                    }
                    match keycode.unwrap() {
                        Keycode::Escape => std::process::exit(0),
                        _ => continue,
                    }
                }
                _ => continue,
            }
        }
        let start = std::time::Instant::now();
        for _ in 0..OP_BUNDLE_SIZE {
            cpu.tick();
            if cpu.update_screen {
                cpu.update_screen = false;
                display.draw(&cpu.vram);
            }
        }
        let sleep_duration = MS_TARGET - start.elapsed().as_millis() as usize;
        if sleep_duration > 0 {
            thread::sleep(std::time::Duration::from_millis(sleep_duration as u64));
        }
    }
}
