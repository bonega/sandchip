use sandchiplib::cpu::CPU;
use sandchiplib::display::Display;
use sandchiplib::input;
use sdl2::event::Event;
use std::{env, fs, thread};

const OP_BUNDLE_SIZE: usize = 5;
const HZ: usize = 400;
const MS_TARGET: usize = (1000 as f32 / HZ as f32 * OP_BUNDLE_SIZE as f32) as usize;

fn main() {
    let input = input::Input::load();
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
    cpu.load_rom(&data);
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context);
    let mut event_pump = sdl_context.event_pump().unwrap();
    loop {
        let start = std::time::Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {}
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(k) => input.handle_key_down(k, &mut cpu),
                    None => continue,
                },
                Event::KeyUp { keycode, .. } => match keycode {
                    Some(k) => input.handle_key_up(k, &mut cpu),
                    None => continue,
                },
                _ => continue,
            }
        }
        for _ in 0..OP_BUNDLE_SIZE {
            cpu.tick();
            if cpu.update_screen {
                cpu.update_screen = false;
                display.draw(&cpu.vram);
            }
        }
        let sleep_duration = MS_TARGET as isize - start.elapsed().as_millis() as isize;
        if sleep_duration > 0 {
            thread::sleep(std::time::Duration::from_millis(sleep_duration as u64));
        }
    }
}
