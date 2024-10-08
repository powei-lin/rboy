pub const LCD_CONTROL_RW: u16 = 0xff40;
pub const STATUS_ADDR_RW: u16 = 0xff41;
pub const SCROLL_Y_RW: u16 = 0xff42;
pub const SCROLL_X_RW: u16 = 0xff43;
pub const Y_COORDINATE_R: u16 = 0xff44;
pub const LY_COMPARE_RW: u16 = 0xff45;
pub const BG_PALETTE_DATA: u16 = 0xff47;
pub const WINDOW_Y_POSITION_RW: u16 = 0xff4a;
pub const WINDOW_X_POSITION_MINUS_7_RW: u16 = 0xff4b;

// interrupt
pub const INTERRUPT_ENABLE: u16 = 0xffff;
pub const INTERRUPT_FLAG: u16 = 0xff0f;
pub const INTR_VBLANK: u16 = 0x0040;
pub const INTR_LCDC: u16 = 0x0048;
pub const INTR_TIMER: u16 = 0x0050;
pub const INTR_SERIAL: u16 = 0x0058;
pub const INTR_HIGHTOLOW: u16 = 0x0060;
pub const INTERRPUT_LIST: [u16; 5] = [
    INTR_VBLANK,
    INTR_LCDC,
    INTR_TIMER,
    INTR_SERIAL,
    INTR_HIGHTOLOW,
];
pub const INTR_VBLANK_BIT: u8 = 0;
pub const INTR_LCDC_BIT: u8 = 1;
pub const INTR_TIMER_BIT: u8 = 2;
pub const INTR_SERIAL_BIT: u8 = 3;
pub const INTR_HIGHTOLOW_BIT: u8 = 4;

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
pub const OAM_RAM_SIZE: usize = 0xa0;

/// Joypad addr
pub const IO_START: usize = 0xff00;
pub const HRAM_START: usize = 0xff80;

// clocks
pub const OAM_CYCLE_IN_4MHZ: u16 = 80;
pub const MAX_DRAWING_CYCLE_IN_4MHZ: u16 = 289;
pub const DRAW_AND_HBLANK_CYCLE_IN_4MHZ: u16 = 376;
pub const VBLANK_CYCLE_IN_4MHZ: u16 = 456;
pub const VBLANK_END_LY: u8 = 153;
