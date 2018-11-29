extern crate piston_window;
extern crate rand;

mod stack;
mod chip8;
mod graphics;

use self::chip8::Chip;
use self::graphics::{BLACK, BLOCK_SIZE};

use piston_window::*;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print!("No Chip8 program entered\n\nExiting...");
        return
    }

    let (width, height) = (64, 32);

    let mut window: PistonWindow = WindowSettings::new("Chip8 Emulator", [width * (BLOCK_SIZE as u32), height * (BLOCK_SIZE as u32)])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut chip = Chip::new(args[1].as_str());

    while let Some(event) = window.next() {
        if let Some(Button::Keyboard(key)) = event.press_args() {
            chip.key_pressed(key);
        }

        window.draw_2d(&event, |c, g| {
            clear(BLACK, g);
            chip.display.draw(&c, g);
        });

        event.update(|arg| {
            chip.update(arg.dt);
        });
    }
}
