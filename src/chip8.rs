/**
 * Reference:
 * 0x000-0x1FF - Chip8 interpreter (contains the font set in emulation)
 * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
 * 0x200-0xFFF - Prgarm ROM and work RAM
 */
use super::stack::Stack;

use super::graphics::{draw_block, Display, HEIGHT, WIDTH, BLACK, WHITE};

use piston_window::types::Color;
use piston_window::*;
use rand::prelude::random;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type opcode = u16;

const VFLAG: usize = 15;
const PERIOD: f64 = 1.0;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip {
    registers: [u8; 16],
    stack: Stack,
    mem: [u8; 4096],
    index: u16,
    program_counter: usize,
    pub display: Display,
    timer_delay: u8,
    timer_sound: u8,
    key: [bool; 16],
    cycle: f64,
}

impl Chip {
    pub fn new(program: &str) -> Chip {
        let mut chip = Chip {
            registers: [0; 16],
            stack: Stack::new(),
            mem: [0; 4096],
            index: 0,
            display: Display::new(),
            program_counter: 0x200,
            timer_delay: 0,
            timer_sound: 0,
            key: [false; 16],
            cycle: 0.0,
        };

        // Open up program file path and file
        let prog_path = PathBuf::from(program);
        let mut input_stream = match File::open(prog_path.as_path()) {
            Err(e) => panic!("Couldn't open Chip8 program: {}", e.description()),
            Ok(stream) => stream,
        };

        // Load program
        match input_stream.read(&mut chip.mem[0x200..0xFFF]) {
            Err(e) => panic!("Couldn't read Chip8 program: {}", e.description()),
            Ok(_) => println!("Successful read."),
        }

        for x in 0x200..0xFFF {
            print!("{:02X}", chip.mem[x]);
        }

        // Load fontset
        for i in 0..80 {
            chip.mem[i] = FONTSET[i];
        }

        chip
    }

    // Emulates one cycle of the Chip-8 CPU
    pub fn cycle(&mut self) {
        // fetch opcode
        let operation: opcode = (self.mem[self.program_counter] as u16) << 8
            | (self.mem[self.program_counter + 1] as u16); // Have to cast bytes as u16 to satisfy Rust's type requirements
//            print!("Opcode: {:X}", operation);

        // decode operation
        let n1 = (operation & 0xF000) >> 12;    //  Opcodes can be split up as OXYN where typically
        let n2 = (operation & 0x0F00) >> 8;     //  O = operation, X, Y, N = sub operation, byte, etc
        let n3 = (operation & 0x00F0) >> 4;     //  by splitting the opcode up into nibbles, we can match
        let n4 = operation & 0x000F;            //  via tuple when we decode: (n1, n2, n3, n4)
//        println!("\tNibbles: {:X} : {:X} : {:X} : {:X}", n1, n2, n3, n4);

        let nnn = (operation & 0x0FFF) as usize;
        let x = ((operation & 0x0F00) >> 8) as u8;
        let y = ((operation & 0x00F0) >> 4) as u8;
        let Vx = ((operation & 0x0F00) >> 8) as usize;
        let Vy = ((operation & 0x00F0) >> 4) as usize;
        let kk = (operation & 0x00FF) as u8;
        let n = (operation & 0x000F) as u8;
//        println!("nnn: {:X} x: {:X} y: {:X} Vx: {:X} Vy: {:X} kk: {:X} n: {:X}", nnn, x, y, Vx, Vy, kk, n);

        // update program_counter
        self.program_counter += 2;

        // execute operation
        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.display.cls(),                             // Cls (clear screen)
            (0x0, 0x0, 0xE, 0xE) => {                                       // Ret
                self.program_counter = self.stack.pop().unwrap() as usize;
            },
            (0x0, _, _, _) => self.program_counter = nnn,                   // Sys **Not used
            (0x1, _, _, _) => self.program_counter = nnn,                   // Jmp
            (0x2, _, _, _) => {                                             // Call
                match self.stack.push(self.program_counter as u16) {    
                    (_) => {},
                    Err(s) => panic!("Error with stack: {}", s),
                };
                self.program_counter = nnn;
            },
            (0x3, _, _, _) => {                                             // SE
                if self.registers[Vx] == kk {
                    self.program_counter += 2;
                }
            },
            (0x4, _, _, _) => {                                             // SNE
                if self.registers[Vx] != kk {
                    self.program_counter += 2;
                }
            },
            (0x5, _, _, 0x0) => {                                           // SE
                if self.registers[Vx] == self.registers[Vy] {           
                    self.program_counter += 2;
                }
            },
            (0x6, _, _, _) => self.registers[Vx] = kk,                      // Mov
            (0x7, _, _, _) => self.registers[Vx] += kk,                     // Add
            (0x8, _, _, 0x0) => self.registers[Vx] = self.registers[Vy],    // Mov
            (0x8, _, _, 0x1) => self.registers[Vx] |= self.registers[Vy],   // Or
            (0x8, _, _, 0x2) => self.registers[Vx] &= self.registers[Vy],   // And
            (0x8, _, _, 0x3) => self.registers[Vx] ^= self.registers[Vy],   // Xor
            (0x8, _, _, 0x4) => {                                           // Add
                let mut total: u32 = self.registers[Vx] as u32 + self.registers[Vy] as u32;
                if total > 255 {
                    self.registers[VFLAG] = 1;
                } else {
                    self.registers[VFLAG] = 0;
                }
                total &= 0xFF;
                self.registers[Vx] = total as u8;
            },
            (0x8, _, _, 0x5) => {                                           // Sub
                if self.registers[Vx] < self.registers[Vy] {
                    self.registers[VFLAG] = 1;
                } else {
                    self.registers[VFLAG] = 0;
                }
                self.registers[Vx] -= self.registers[Vy];
            },
            (0x8, _, _, 0x6) => {                                           // SHR
                if (self.registers[Vx] & 0x01) == 0x01 {
                    self.registers[VFLAG] = 1;
                } else {
                    self.registers[VFLAG] = 0;
                }
                self.registers[Vx] >>= 1;
            },
            (0x8, _, _, 0x7) => {                                           // Subn
                if self.registers[Vx] < self.registers[Vy] {
                    self.registers[VFLAG] = 1;
                } else {
                    self.registers[VFLAG] = 0;
                }
                self.registers[Vx] = self.registers[Vy] - self.registers[Vx];
            },
            (0x8, _, _, 0xE) => {                                           // SHL
                if (self.registers[Vx] & 0x80) == 0x80 {
                    self.registers[VFLAG] = 1;
                } else {
                    self.registers[VFLAG] = 0;
                }
                self.registers[Vx] <<= 1;
            },
            (0x9, _, _, 0x0) => {
                if self.registers[Vx] != self.registers[Vy] {
                    self.program_counter += 2;
                }
            },
            (0xA, _, _, _) => self.program_counter = nnn,
            (0xB, _, _, _) => self.program_counter = nnn + (self.registers[0] as usize),
            (0xC, _, _, _) => {
                let r: u8 = random();
                self.registers[Vx] &= kk;
            },
            (0xD, _, _, _) => {                                             // Drw
                // Need to think about
                self.registers[VFLAG] = 0;
                if self.display.update_screen(Vx, Vy, &self.mem[self.index as usize .. (self.index + n as u16) as usize]) {
                    self.registers[VFLAG] = 1;
                }
            },
            (0xE, _, 0x9, 0xE) => {
                if self.key[Vx] {
                    self.program_counter += 2;
                }
            },
            (0xE, _, 0xA, 0x1) => {
                if !self.key[Vx] {
                    self.program_counter += 2;
                }
            },
            (0xF, _, 0x0, 0x7) => self.registers[Vx] = self.timer_delay,
            (0xF, _, 0x0, 0xA) => {
                // Need to think about
 //               self.program_counter -= 2;
 //               for (i, key) in self.keypad.keys.iter().enumerate() {
 //                   if *key == true {
 //                       self.registers[Vx] = i as u8;
 //                       self.program_counter += 2;
 //                   }
 //               }
            }, 
            (0xF, _, 0x1, 0x5) => self.timer_delay = self.registers[Vx],
            (0xF, _, 0x1, 0x8) => self.timer_sound = self.registers[Vx],
            (0xF, _, 0x1, 0xE) => self.index += self.registers[Vx] as u16,
            (0xF, _, 0x2, 0x9) => self.index = self.index + self.registers[Vx] as u16,
            (0xF, _, 0x3, 0x3) => {
                self.mem[self.index as usize] = x /100;
                self.mem[self.index as usize + 1] = (x / 10) % 10;
                self.mem[self.index as usize + 2] = (x % 100) % 10;
            },
            (0xF, _, 0x5, 0x5) => self.mem[self.index as usize .. self.index as usize + Vx + 1].copy_from_slice(&self.registers[0..Vx + 1]),
            (0xF, _, 0x6, 0x5) => self.registers[0..Vx + 1].copy_from_slice(&self.mem[self.index as usize..self.index as usize + Vx + 1]),
            (_, _, _, _) => println!("Unknown opcode: 0x{:X}{:X}{:X}{:X}", n1, n2, n3, n4),
        }

        // update timers
        if self.timer_delay > 0 {
            self.timer_delay -= 1;
        }
        if self.timer_sound > 0 {
            self.timer_sound -= 1;
        }
        
        self.cycle = 0.0;
    }

    pub fn picture_test(&mut self, x: usize, y: usize) {
        self.display.mem[x][y] = 0x1;
    }

 //   pub fn draw(&self, x: usize, y: usize, sprite: &[u8], con: &Context, g: &mut G2d) {
 //       for x in 0..64 {
 //           for y in 0..32 {
 //               //println!("x: {} y: {}", x, y);
 //               if self.display.mem[x][y] != 0x0 {
 //                   draw_block(BLACK, x as i32, y as i32, con, g);
 //               } else {
 //                   draw_block(WHITE, x as i32, y as i32, con, g);
 //               }
 //           }
 //       }
 //   }

    pub fn update(&mut self, delta_time: f64) {
        self.cycle += delta_time;
        if self.cycle > PERIOD {
            self.cycle();
        }
    }

    pub fn run(&self, con: &Context, g: &mut G2d) {}

    pub fn key_pressed(&mut self, key: Key) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dumb_shift_test() {
        assert_eq!(2, 4 >> 1)
    }

    #[test]
    fn instruction_test() {
        
    }
}