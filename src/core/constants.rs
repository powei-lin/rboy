pub const CONTROL_ADDR_RW: usize = 0xff40;
pub const STATUS_ADDR_RW: usize = 0xff41;
pub const SCROLL_Y_RW: usize = 0xff42;
pub const SCROLL_X_RW: usize = 0xff43;
pub const Y_COORDINATE_R: usize = 0xff44;
pub const LY_COMPARE_RW: usize = 0xff45;
pub const WINDOW_Y_POSITION_RW: usize = 0xff4a;
pub const WINDOW_X_POSITION_MINUS_7_RW: usize = 0xff4b;

pub const LCD_WIDTH: u32 = 160;
pub const LCD_HEIGHT: u32 = 144;
pub const BG_SIZE: u32 = 256;
pub const GRAY_SHADES: [u8; 4] = [255, 170, 85, 0];
