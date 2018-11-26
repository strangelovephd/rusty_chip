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

