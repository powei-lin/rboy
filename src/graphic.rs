use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::{draw_texture, WHITE};
use macroquad::texture::Texture2D;

pub const GAMEBOY_WINDOW_WIDTH: u32 = 160;
pub const GAMEBOY_WINDOW_HEIGHT: u32 = 144;
pub const GAMEBOY_WINDOW_PIXELS: u32 = GAMEBOY_WINDOW_HEIGHT * GAMEBOY_WINDOW_WIDTH;

pub fn set_gameboy_window_scale(scale: u8) {
    set_window_size(
        GAMEBOY_WINDOW_WIDTH * scale as u32,
        GAMEBOY_WINDOW_HEIGHT * scale as u32,
    );
}
fn gameboy_window_size(scale: u8) -> (u32, u32) {
    (
        GAMEBOY_WINDOW_WIDTH * scale as u32,
        GAMEBOY_WINDOW_HEIGHT * scale as u32,
    )
}

pub struct Screen {
    scale: u8,
    window_width: u32,
    window_height: u32,
    frame_buffer: Vec<u8>,
    debug: bool,
}
impl Screen {
    pub fn new(scale: u8, debug: bool) -> Screen {
        set_gameboy_window_scale(scale);

        let (window_width, window_height) = gameboy_window_size(scale);
        let rgba_pixels = (window_height * window_width * 4) as usize;
        let frame_buffer: Vec<u8> = Vec::from_iter(std::iter::repeat(255).take(rgba_pixels));
        Screen {
            scale,
            window_width,
            window_height,
            frame_buffer,
            debug,
        }
    }
    pub fn update_pixel_in_buffer(&mut self, x: u32, y: u32, val: u8) {
        for c in (x * self.scale as u32)..((x + 1) * self.scale as u32) {
            for r in (y * self.scale as u32)..((y + 1) * self.scale as u32) {
                let idx = ((r * self.window_width + c) * 4) as usize;
                self.frame_buffer[idx] = val;
                self.frame_buffer[idx + 1] = val;
                self.frame_buffer[idx + 2] = val;
            }
        }
    }
    pub fn draw_frame(&self) {
        let texture = Texture2D::from_rgba8(
            self.window_width as u16,
            self.window_height as u16,
            self.frame_buffer.as_slice(),
        );
        draw_texture(&texture, 0.0, 0.0, WHITE);
    }
}
