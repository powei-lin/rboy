use macroquad::miniquad::window::set_window_size;

pub const GAMEBOY_WINDOW_WIDTH: u32 = 160;
pub const GAMEBOY_WINDOW_HEIGHT: u32 = 144;
pub const GAMEBOY_WINDOW_PIXELS: u32 = GAMEBOY_WINDOW_HEIGHT * GAMEBOY_WINDOW_WIDTH;

pub fn set_gameboy_window_scale(scale: u8) {
    set_window_size(
        GAMEBOY_WINDOW_WIDTH * scale as u32,
        GAMEBOY_WINDOW_HEIGHT * scale as u32,
    );
}
pub fn gameboy_window_size(scale: u8) -> (u32, u32) {
    (
        GAMEBOY_WINDOW_WIDTH * scale as u32,
        GAMEBOY_WINDOW_HEIGHT * scale as u32,
    )
}
