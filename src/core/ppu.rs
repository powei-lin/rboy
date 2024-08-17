use crate::core::constants::*;

use super::memory;

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

/// Run at 4Mhz
struct PixelFifo {}

/// Run at 2 Mhz
struct PixelFetcher {
    state: FetcherState,
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
    frame_buffer: Vec<u8>,
    bg_frame_buffer: Vec<u8>,
    remained_cycle: u8,
    current_state: PPUState,
    current_state_cycle: u16,
    pixel_fifo: Vec<u8>,
}
// fn data_to_tile(data: &[u8], palette: int = 0) -> np.ndarray:
// if len(data) % 2 != 0:
//     raise ValueError
// i = np.array(data, dtype=np.uint8).reshape(-1, 2)
// i = np.unpackbits(i, axis=1)
// i = (i[:, :8] + i[:, 8:] * 2)
// p = [0b11 & (palette >> (j * 2)) for j in range(4)]
// p = [GRAY_SHADES[j] for j in p]
// return np.vectorize(p.__getitem__)(i).astype(np.uint8)

impl PPU {
    pub fn new() -> PPU {
        let frame_buffer =
            Vec::from_iter(std::iter::repeat(255).take((LCD_HEIGHT * LCD_WIDTH * 4) as usize));
        let bg_frame_buffer =
            Vec::from_iter(std::iter::repeat(155).take((BG_SIZE * BG_SIZE * 4) as usize));
        PPU {
            lcd_ppu_enable: false,
            window_tile_map_area: false,
            window_enable: false,
            bg_and_window_tile_area: false,
            bg_tile_map_area: false,
            obj_size: false,
            obj_enable: false,
            bg_and_window_enable_priority: false,
            frame_buffer,
            bg_frame_buffer,
            remained_cycle: 0,
            current_state: PPUState::OAM,
            current_state_cycle: 0,
            pixel_fifo: vec![],
        }
    }
    fn draw_bg_frame(&mut self, mem: &memory::Memory) {}
    pub fn bg_frame_buffer(&self) -> &Vec<u8> {
        &self.bg_frame_buffer
    }

    fn check_lcdc(&mut self, mem: &memory::Memory) {
        self.lcd_ppu_enable = mem.get_bit(LCD_CONTROL_RW as u16, 7);
        self.window_tile_map_area = mem.get_bit(LCD_CONTROL_RW as u16, 6);
        self.window_enable = mem.get_bit(LCD_CONTROL_RW as u16, 5);
        self.bg_and_window_tile_area = mem.get_bit(LCD_CONTROL_RW as u16, 4);
        self.bg_tile_map_area = mem.get_bit(LCD_CONTROL_RW as u16, 3);
        self.obj_size = mem.get_bit(LCD_CONTROL_RW as u16, 2);
        self.obj_enable = mem.get_bit(LCD_CONTROL_RW as u16, 1);
        self.bg_and_window_enable_priority = mem.get_bit(LCD_CONTROL_RW as u16, 0);
    }

    fn check_all_registers(&mut self, mem: &memory::Memory) {
        self.check_lcdc(mem);
    }

    /// PPU runs at 4MHz
    /// return has frame
    pub fn tick(&mut self, mem: &mut memory::Memory, cpu_cycle_in_4mhz: u8) -> bool {
        self.check_all_registers(mem);
        if !self.lcd_ppu_enable {
            return false;
        }
        self.remained_cycle += cpu_cycle_in_4mhz;
        self.current_state_cycle += cpu_cycle_in_4mhz as u16;
        match self.current_state {
            PPUState::OAM => {
                if self.current_state_cycle >= OAM_CYCLE_IN_4MHZ {
                    // panic!("start DRAWING");
                    self.current_state = PPUState::DRAWING;
                    self.current_state_cycle -= OAM_CYCLE_IN_4MHZ;
                }
            }
            PPUState::DRAWING => {
                if self.current_state_cycle >= MAX_DRAWING_CYCLE_IN_4MHZ {
                    // panic!("start HB");
                    self.current_state = PPUState::HBLANK;
                }
            }
            PPUState::HBLANK => {
                if self.current_state_cycle >= DRAW_AND_HBLANK_CYCLE_IN_4MHZ {
                    self.current_state_cycle -= DRAW_AND_HBLANK_CYCLE_IN_4MHZ;
                    let line_y = mem.get(Y_COORDINATE_R as u16);
                    if line_y < LCD_HEIGHT as u8 {
                        mem.set(Y_COORDINATE_R as u16, line_y + 1);
                        self.current_state = PPUState::OAM;
                    } else {
                        self.current_state = PPUState::VBLANK;
                        // draw frame
                        self.draw_bg_frame(mem);
                        return true;
                    }
                }
            }
            PPUState::VBLANK => {
                if self.current_state_cycle >= VBLANK_CYCLE_IN_4MHZ {
                    self.current_state_cycle -= VBLANK_CYCLE_IN_4MHZ;
                    if mem.get(Y_COORDINATE_R as u16) >= VBLANK_END_LY {
                        self.current_state = PPUState::OAM;
                    }
                }
            }
        }
        false
    }
}
