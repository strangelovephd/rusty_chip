/**
 * Reference:
 * 0x000-0x1FF - Chip8 interpreter (contains the font set in emulation)
 * 0x050-0x0A0 - Used for the built in 4x5 pixel font set (0-F)
 * 0x200-0xFFF - Prgarm ROM and work RAM
 */
use super::stack::Stack;

use super::graphics::{draw_block, BLACK, WHITE};

use piston_window::types::Color;
use piston_window::*;
use rand::prelude::random;

use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

type opcode = u16;

const VFLAG: usize = 15;

#[derive(Debug, Eq, PartialEq)]
pub enum Instruction {
    CLS(opcode),        //  00E0 - CLS
    RET(opcode),        //  00EE - RET
    SYS(opcode),        //  0nnn - SYS addr
    JP(opcode),         //  1nnn - JP addr
    CALL(opcode),       //  2nnn - CALL addr
    SEVxb(opcode),      //  3xkk - SE Vx, byte
    SNE(opcode),        //  4xkk - SNE Vx, byte
    SE(opcode),         //  5xy0 - SE Vx, Vy
    LDVxb(opcode),      //  6xkk - LD Vx, byte
    ADDVxb(opcode),     //  7xkk - ADD Vx, byte
    LDVxVy(opcode),     //  8xy0 - LD Vx, Vy
    OR(opcode),         //  8xy1 - OR Vx, Vy
    AND(opcode),        //  8xy2 - AND Vx, Vy
    XOR(opcode),        //  8xy3 - XOR Vx, Vy
    ADDVxVy(opcode),    //  8xy4 - ADD Vx, Vy
    SUB(opcode),        //  8xy5 - SUB Vx, Vy
    SHR(opcode),        //  8xy6 - SHR Vx {, Vy}
    SUBN(opcode),       //  8xy7 - SUBN Vx, Vy
    SHL(opcode),        //  8xyE - SHL Vx {, Vy}
    SNEVxVy(opcode),    //  9xy0 - SNE Vx, Vy
    LDIAddr(opcode),    //  Annn - LD I, addr
    JPV0Addr(opcode),   //  Bnnn - JP V0, addr
    RND(opcode),        //  Cxkk - RND Vx, byte
    DRW(opcode),        //  Dxyn - DRW Vx, Vy, nibble
    SKP(opcode),        //  Ex9E - SKP Vx
    SKNP(opcode),       //  ExA1 - SKNP Vx
    LDVxDT(opcode),     //  Fx07 - LD Vx, DT
    LDVxK(opcode),      //  Fx0A - LD Vx, K
    LDDTVx(opcode),     //  Fx15 - LD DT, Vx
    LDSTVx(opcode),     //  Fx18 - LD ST, Vx
    ADDIVx(opcode),     //  Fx1E - ADD I, Vx
    LDFVx(opcode),      //  Fx29 - LD F, Vx
    LDBVx(opcode),      //  Fx33 - LD B, Vx
    LDIVx(opcode),      //  Fx55 - LD [I], Vx
    LDVxI(opcode),      //  Fx65 - LD Vx, [I]
    None,
}

pub struct Chip {
    registers: [u8; 16],
    stack: Stack,
    mem: [u8; 4096],
    index: u16,
    program_counter: usize,
    display: [[bool; 32]; 64],
    timer_delay: u8,
    timer_sound: u8,
    key: [bool; 16],
}

impl Chip {
    pub fn new(program: &str) -> Chip {
        let mut chip = Chip {
            registers: [0; 16],
            stack: Stack::new(),
            mem: [0; 4096],
            index: 0,
            display: [[false; 32]; 64],
            program_counter: 0x200,
            timer_delay: 0,
            timer_sound: 0,
            key: [false; 16],
        };

        let prog_path = PathBuf::from(program);

        let mut input_stream = match File::open(prog_path.as_path()) {
            Err(e) => panic!("Couldn't open Chip8 program: {}", e.description()),
            Ok(stream) => stream,
        };

        match input_stream.read(&mut chip.mem[0x200..0xFFF]) {
            Err(e) => panic!("Couldn't read Chip8 program: {}", e.description()),
            Ok(_) => println!("Successful read."),
        }

        for x in 0x200..0xFFF {
            print!("{}", chip.mem[x]);
        }

        chip
    }

    fn cycle(&mut self) {
        // fetch opcode
        let operation: opcode = (self.mem[self.program_counter] as u16) << 8
            | (self.mem[self.program_counter + 1] as u16); // Have to cast bytes as u16 to satisfy Rust's type requirements

        // decode operation
        let n1 = (operation & 0xF000) >> 12;    //  Opcodes can be split up as OXYN where typically
        let n2 = (operation & 0x0F00) >> 8;     //  O = operation, X, Y, N = sub operation, byte, etc
        let n3 = (operation & 0x00F0) >> 4;     //  by splitting the opcode up into nibbles, we can match
        let n4 = operation & 0x000F;            //  via tuple when we decode: (n1, n2, n3, n4)

        let nnn = (operation & 0x0FFF) as usize;
        let x = (operation & 0x0F00) as u8;
        let y = (operation & 0x00F0) as u8;
        let Vx = (operation & 0x0F00) as usize;
        let Vy = (operation & 0x00F0) as usize;
        let kk = (operation & 0x00FF) as u8;
        let n = (operation & 0x000F) as u8;

        // execute operation
        match (n1, n2, n3, n4) {
            (0x0, 0x0, 0xE, 0x0) => self.cls(),                             // Cls (clear screen)
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
            }, 
            (0xF, _, 0x1, 0x5) => self.timer_delay = self.registers[Vx],
            (0xF, _, 0x1, 0x8) => self.timer_sound = self.registers[Vx],
            (0xF, _, 0x1, 0xE) => self.index += self.registers[Vx] as u16,
            (0xF, _, 0x2, 0x9) => {
                // need to think about sprites
            },
            (0xF, _, 0x3, 0x3) => {
                self.mem[self.index as usize] = x /100;
                self.mem[self.index as usize + 1] = (x / 10) % 10;
                self.mem[self.index as usize + 2] = (x % 100) % 10;
            },
            (0xF, _, 0x5, 0x5) => {
                for i in 0..15 {
                    self.mem[(self.index as usize) + i] = self.registers[i];
                }
            },
            (0xF, _, 0x6, 0x5) => {
                for i in 0..15 {
                    self.registers[i] = self.mem[(self.index as usize) + i];
                }
            },
            (_, _, _, _) => {},
        }
        // update timers
    }

    fn cls(&mut self) {
        for x in 0..64 {
            for y in 0..32 {
                self.display[x][y] = false;
            }
        }
    }

    pub fn picture_test(&mut self, x: usize, y: usize) {
        self.display[x][y] = true;
    }

    pub fn draw(&self, con: &Context, g: &mut G2d) {
        for x in 0..64 {
            for y in 0..32 {
                //println!("x: {} y: {}", x, y);
                if self.display[x][y] == true {
                    draw_block(BLACK, x as i32, y as i32, con, g);
                } else {
                    draw_block(WHITE, x as i32, y as i32, con, g);
                }
            }
        }
    }

    pub fn update(&mut self, delta_time: f64) {}

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
}