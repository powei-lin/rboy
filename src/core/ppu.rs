use crate::core::constants::*;

use super::memory;

const GRAY_SHADES: [u8; 4] = [255, 170, 85, 0];

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

    /// false = 9800–9BFF; true = 9C00–9FFF
    window_tile_map_area: bool,
    window_enable: bool,

    /// false = 8800–97FF; true = 8000–8FFF
    bg_and_window_tile_data_area: bool,

    /// false = 9800–9BFF; true = 9C00–9FFF
    bg_tile_map_area: bool,

    /// false = 8×8; true = 8×16
    obj_size: bool,
    obj_enable: bool,
    bg_and_window_enable_priority: bool,
    frame_buffer: Vec<u8>,
    bg_frame_buffer: Vec<u8>,
    remained_cycle: u8,
    current_state: PPUState,
    current_state_cycle: u16,
    pixel_fifo: Vec<u8>,
}
fn calculate_tile(data: &[u8], palette: u8) -> [u8; 64] {
    let mut tile_data = [0; 64];
    let colors: Vec<u8> = (0..4).map(|x| (palette >> (x * 2)) & 0b11).collect();
    for y in 0..8 {
        let idx = y * 2;
        for c in 0..8 {
            let shift = 7 - c;
            let upper_bit = (data[idx] >> shift) & 1;
            let lower_bit = (data[idx + 1] >> shift) & 1;
            let color_idx = ((upper_bit << 1) + lower_bit) as usize;
            let color = colors[color_idx];
            tile_data[y * 8 + c] = GRAY_SHADES[color as usize];
        }
    }
    tile_data
}

impl PPU {
    pub fn new() -> PPU {
        let frame_buffer =
            Vec::from_iter(std::iter::repeat(255).take((LCD_HEIGHT * LCD_WIDTH * 4) as usize));
        let bg_frame_buffer =
            Vec::from_iter(std::iter::repeat(255).take((BG_SIZE * BG_SIZE * 4) as usize));
        PPU {
            lcd_ppu_enable: false,
            window_tile_map_area: false,
            window_enable: false,
            bg_and_window_tile_data_area: false,
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
    fn bg_tile_map_range(&self) -> std::ops::Range<u16> {
        if self.bg_tile_map_area {
            0x9c00..0xa000
        } else {
            0x9800..0x9c00
        }
    }
    fn bg_tile_data_start_addr(&self) -> u16 {
        if self.bg_and_window_tile_data_area {
            0x8000
        } else {
            0x8800
        }
    }
    fn draw_bg_frame(&mut self, mem: &memory::Memory) {
        for (i, addr) in self.bg_tile_map_range().enumerate() {
            let tile_idx = mem.get(addr);
            let tile_data_addr = tile_idx as u16 * 16 + self.bg_tile_data_start_addr();
            let palette = mem.get(BG_PALETTE_DATA as u16);

            let tile_data = calculate_tile(mem.get_chunck(tile_data_addr, 16), palette);

            for y in 0..8 {
                let row = (i / 32) * 8 + y;
                for x in 0..8 {
                    let col = (i % 32) * 8 + x;
                    let bg_frame_buffer_idx = (row * BG_SIZE as usize + col) * 4;
                    for j in 0..3 {
                        self.bg_frame_buffer[bg_frame_buffer_idx + j] = tile_data[y * 8 + x];
                    }
                }
            }
        }
    }
    fn draw_view_port(&mut self, mem: &memory::Memory) {
        let scy = mem.get(SCROLL_Y_RW);
        let scx = mem.get(SCROLL_X_RW);
        let bg_usize = BG_SIZE as usize;
        for c in (scx as usize)..(scx as usize + LCD_WIDTH as usize) {
            let c = c % bg_usize;
            let idx = ((scy as usize) * bg_usize + c) * 4;
            self.bg_frame_buffer[idx] = 255;
            self.bg_frame_buffer[idx + 1] = 0;
            self.bg_frame_buffer[idx + 2] = 0;
            let idx = (((scy as usize + LCD_HEIGHT as usize - 1) % bg_usize) * bg_usize + c) * 4;
            self.bg_frame_buffer[idx] = 255;
            self.bg_frame_buffer[idx + 1] = 0;
            self.bg_frame_buffer[idx + 2] = 0;
        }
        for r in (scy as usize)..(scy as usize + LCD_HEIGHT as usize) {
            let r = r % bg_usize;
            let idx = (r * bg_usize + scx as usize) * 4;
            self.bg_frame_buffer[idx] = 255;
            self.bg_frame_buffer[idx + 1] = 0;
            self.bg_frame_buffer[idx + 2] = 0;
            let idx = (r * bg_usize + ((scx as usize) + LCD_WIDTH as usize - 1) % bg_usize) * 4;
            self.bg_frame_buffer[idx] = 255;
            self.bg_frame_buffer[idx + 1] = 0;
            self.bg_frame_buffer[idx + 2] = 0;
        }
    }
    pub fn bg_frame_buffer(&self) -> &Vec<u8> {
        &self.bg_frame_buffer
    }

    fn check_lcdc(&mut self, mem: &memory::Memory) {
        self.lcd_ppu_enable = mem.get_bit(LCD_CONTROL_RW, 7);
        self.window_tile_map_area = mem.get_bit(LCD_CONTROL_RW, 6);
        self.window_enable = mem.get_bit(LCD_CONTROL_RW, 5);
        self.bg_and_window_tile_data_area = mem.get_bit(LCD_CONTROL_RW, 4);
        self.bg_tile_map_area = mem.get_bit(LCD_CONTROL_RW, 3);
        self.obj_size = mem.get_bit(LCD_CONTROL_RW, 2);
        self.obj_enable = mem.get_bit(LCD_CONTROL_RW, 1);
        self.bg_and_window_enable_priority = mem.get_bit(LCD_CONTROL_RW, 0);
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
                    let line_y = mem.get(Y_COORDINATE_R);
                    if line_y < LCD_HEIGHT as u8 {
                        mem.set(Y_COORDINATE_R, line_y + 1);
                        self.current_state = PPUState::OAM;
                    } else {
                        self.current_state = PPUState::VBLANK;
                        // draw frame
                        self.draw_bg_frame(mem);
                        self.draw_view_port(mem);
                        return true;
                    }
                }
            }
            PPUState::VBLANK => {
                if self.current_state_cycle >= VBLANK_CYCLE_IN_4MHZ {
                    self.current_state_cycle -= VBLANK_CYCLE_IN_4MHZ;
                    if mem.get(Y_COORDINATE_R) >= VBLANK_END_LY {
                        self.current_state = PPUState::OAM;
                    }
                }
            }
        }
        false
    }
}
