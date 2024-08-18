pub const LCD_CONTROL_RW: u16 = 0xff40;
pub const STATUS_ADDR_RW: u16 = 0xff41;
pub const SCROLL_Y_RW: u16 = 0xff42;
pub const SCROLL_X_RW: u16 = 0xff43;
pub const Y_COORDINATE_R: u16 = 0xff44;
pub const LY_COMPARE_RW: u16 = 0xff45;
pub const BG_PALETTE_DATA: u16 = 0xff47;
pub const WINDOW_Y_POSITION_RW: u16 = 0xff4a;
pub const WINDOW_X_POSITION_MINUS_7_RW: u16 = 0xff4b;

// io ranges
pub const DISABLE_BOOT_ROM: usize = 0xff50;

pub const LCD_WIDTH: u32 = 160;
pub const LCD_HEIGHT: u32 = 144;
pub const BG_SIZE: u32 = 256;
pub const GRAY_SHADES: [u8; 4] = [255, 170, 85, 0];
pub const TILE_DATA_SIZE: usize = 16;

// ram addr
pub const VRAM_START: usize = 0x8000;
pub const EXTERNAL_RAM_START: usize = 0xa000;
pub const RAM_START: usize = 0xc000;
pub const OAM_RAM_START: usize = 0xfe00;
pub const IO_START: usize = 0xff00;
pub const HRAM_START: usize = 0xff80;

// clocks
pub const OAM_CYCLE_IN_4MHZ: u16 = 80;
pub const MAX_DRAWING_CYCLE_IN_4MHZ: u16 = 289;
pub const DRAW_AND_HBLANK_CYCLE_IN_4MHZ: u16 = 376;
pub const VBLANK_CYCLE_IN_4MHZ: u16 = 456;
pub const VBLANK_END_LY: u8 = 153;
