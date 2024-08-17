use crate::core::constants::*;

enum PPUState {
    HBLANK,
    VBLANK,
    OAM,
    DRAWING,
}

enum FetcherState {
    ReadTileID,
    ReadTileData,
    PushToFIFO,
    Idle,
}
pub struct PPU {
    lcd_ppu_enable: bool,
    window_tile_map_area: bool, // false = 9800–9BFF; true = 9C00–9FFF
    window_enable: bool,
    bg_and_window_tile_area: bool, // false = 8800–97FF; true = 8000–8FFF
    bg_tile_map_area: bool,        // false = 9800–9BFF; true = 9C00–9FFF
    obj_size: bool,                // false = 8×8; true = 8×16
    obj_enable: bool,
    bg_and_window_enable_priority: bool,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            lcd_ppu_enable: false,
            window_tile_map_area: false,
            window_enable: false,
            bg_and_window_tile_area: false,
            bg_tile_map_area: false,
            obj_size: false,
            obj_enable: false,
            bg_and_window_enable_priority: false,
        }
    }
}
