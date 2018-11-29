use piston_window::{rectangle, Context, G2d};
use piston_window::types::Color;

pub const WHITE: Color = [0.00, 0.00, 0.00, 1.00];
pub const BLACK: Color = [1.00, 1.00, 1.00, 1.00];

pub const BLOCK_SIZE: f64 = 15.0;

pub fn to_coord(c: i32) -> f64 {
    (c as f64) * BLOCK_SIZE
}

pub fn to_coord_u32(c: i32) -> u32 {
    to_coord(c) as u32
}

pub fn draw_block(color: Color, x: i32, y: i32, con: &Context, g: &mut G2d) {
    let gui_x = to_coord(x);
    let gui_y = to_coord(y);

    rectangle(color, [gui_x, gui_y, BLOCK_SIZE, BLOCK_SIZE], con.transform, g);
}

pub fn draw_rectangle(color: Color, x: i32, y: i32, width: i32, height: i32, con: &Context, g: &mut G2d) {
    let x = to_coord(x);
    let y = to_coord(y);

    rectangle(color, [x, y, BLOCK_SIZE * (width as f64), BLOCK_SIZE * (height as f64)], con.transform, g);
}

pub const HEIGHT: usize = 32;
pub const WIDTH: usize = 64;

pub struct Display {
    pub mem: [[u8; 32]; 64],
}

impl Display {
    pub fn new() -> Display {
        Display {
            mem: [[0; 32]; 64],
        }
    }

    #[inline]
    pub fn set_pixel(&mut self, x: usize, y: usize, state: bool) {
        self.mem[x][y] = state as u8;
    }

    #[inline]
    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.mem[x][y] == 0x01
    }

    #[inline]
    pub fn cls(&mut self) {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                self.set_pixel(x, y, false);
            }
        }
    }

    pub fn draw(&self, con: &Context, g: &mut G2d) {
        for x in 0..64 {
            for y in 0..32 {
                if self.mem[x][y] != 0x0 {
                    draw_block(BLACK, x as i32, y as i32, con, g);
                } else {
                    draw_block(WHITE, x as i32, y as i32, con, g);
                }
            }
        }
    }

    #[inline]
    pub fn update_screen(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let rows = sprite.len();
        let mut collision = false;
        for j in 0..rows {
            let row = sprite[j];
            for i in 0..8 {
                let new_val = row >> (7 - i) & 0x01;
                if new_val == 0x01 {
                    let xi = (x + i) % WIDTH;
                    let yj = (y + j) % HEIGHT;
                    let old_val = self.get_pixel(xi, yj);
                    if old_val {
                        collision = true;
                    }
                    self.set_pixel(xi, yj, (new_val == 1) ^ old_val);
                }
            }
        }
        collision
    }
}