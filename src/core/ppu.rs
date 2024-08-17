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
pub struct PPU {}

impl PPU {
    pub fn new() -> PPU {
        PPU {}
    }
}
