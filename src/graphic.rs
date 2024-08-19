use crate::core::constants::*;
use macroquad::math::vec2;
use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::WHITE;
use macroquad::texture::{draw_texture_ex, DrawTextureParams, Texture2D};

pub const GAMEBOY_WINDOW_PIXELS: u32 = LCD_HEIGHT * LCD_WIDTH;

pub fn set_gameboy_window_scale(scale: u8) {
    set_window_size(LCD_WIDTH * scale as u32, LCD_HEIGHT * scale as u32);
}
fn gameboy_window_size(scale: u8) -> (u32, u32) {
    (LCD_WIDTH * scale as u32, LCD_HEIGHT * scale as u32)
}
pub fn set_debug_window(scale: u8) {
    set_window_size((LCD_WIDTH + BG_SIZE) * scale as u32, BG_SIZE * scale as u32);
}

pub struct Screen {
    scale: u8,
    frame_buffer: Vec<u8>,
    debug: bool,
}
impl Screen {
    pub fn new(scale: u8, debug: bool) -> Screen {
        let frame_buffer: Vec<u8> =
            Vec::from_iter(std::iter::repeat(255).take((LCD_HEIGHT * LCD_WIDTH * 4) as usize));
        let mut bg_frame_buffer = Vec::<u8>::new();
        if debug {
            bg_frame_buffer =
                Vec::from_iter(std::iter::repeat(255).take((BG_SIZE * BG_SIZE * 4) as usize));
            set_debug_window(scale);
        } else {
            set_gameboy_window_scale(scale);
        }
        Screen {
            scale,
            frame_buffer,
            debug,
        }
    }
    pub fn update_pixel_in_buffer(&mut self, x: u32, y: u32, val: u8) {
        let idx = ((y * LCD_WIDTH + x) * 4) as usize;
        self.frame_buffer[idx] = val;
        self.frame_buffer[idx + 1] = val;
        self.frame_buffer[idx + 2] = val;
    }
    pub fn draw_frame(&self) {
        let texture = Texture2D::from_rgba8(
            LCD_WIDTH as u16,
            LCD_HEIGHT as u16,
            self.frame_buffer.as_slice(),
        );
        texture.set_filter(macroquad::texture::FilterMode::Nearest);
        let (window_width, window_height) = gameboy_window_size(self.scale);
        draw_texture_ex(
            &texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(window_width as f32, window_height as f32)),
                ..Default::default()
            },
        );
    }
    pub fn draw_bg_frame(&self, bg_frame_buffer: &Vec<u8>) {
        let texture: Texture2D =
            Texture2D::from_rgba8(BG_SIZE as u16, BG_SIZE as u16, bg_frame_buffer.as_slice());
        texture.set_filter(macroquad::texture::FilterMode::Nearest);
        let scaled_bg_size = BG_SIZE as f32 * self.scale as f32;
        let (window_width, _window_height) = gameboy_window_size(self.scale);
        draw_texture_ex(
            &texture,
            window_width as f32,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(scaled_bg_size, scaled_bg_size)),
                ..Default::default()
            },
        );
    }
    pub fn draw_tiles_frame(&self, tiles_frame_buffer: &Vec<u8>) {
        let texture: Texture2D = Texture2D::from_rgba8(
            LCD_WIDTH as u16,
            BG_SIZE as u16 - LCD_HEIGHT as u16,
            tiles_frame_buffer.as_slice(),
        );
        texture.set_filter(macroquad::texture::FilterMode::Nearest);
        let scaled_bg_size = BG_SIZE as f32 * self.scale as f32;
        let (window_width, window_height) = gameboy_window_size(self.scale);
        draw_texture_ex(
            &texture,
            0.0,
            window_height as f32,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    window_width as f32,
                    scaled_bg_size as f32 - window_height as f32,
                )),
                ..Default::default()
            },
        );
    }
}
