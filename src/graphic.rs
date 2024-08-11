use macroquad::miniquad::window::set_window_size;

const GAMEBOY_WINDOW_WIDTH: u32 = 160;
const GAMEBOY_WINDOW_HEIGHT: u32 = 144;

pub fn set_gameboy_window_scale(scale: u8) {
    set_window_size(
        GAMEBOY_WINDOW_WIDTH * scale as u32,
        GAMEBOY_WINDOW_HEIGHT * scale as u32,
    );
}
