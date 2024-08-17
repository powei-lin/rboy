use crate::core::constants::*;
use rand::{self, RngCore};

const BOOT_ROM_BYTES: &[u8; 256] = include_bytes!("DMG_ROM.bin");
const RAM_SIZE: usize = 2usize.pow(16);

pub struct Memory {
    data: [u8; RAM_SIZE],
    pub game_rom: Vec<u8>,
    // VRAM and OAM access
    vram_accessible: bool,
    oam_accessible: bool,
}

impl Memory {
    pub fn new(randomize: bool) -> Memory {
        let mut data = [0; RAM_SIZE];
        if randomize {
            rand::thread_rng().fill_bytes(&mut data);
        }

        // io map is not ramdom
        data[LCD_CONTROL_RW] = 0;
        data[STATUS_ADDR_RW] = 0x84;
        data[SCROLL_Y_RW] = 0;
        data[SCROLL_X_RW] = 0;
        data[Y_COORDINATE_R] = 0;
        data[LY_COMPARE_RW] = 0;
        data[0xff46] = 0xff;
        data[0xff47] = 0xfc;
        data[0xff48] = 0xff;
        data[0xff49] = 0xff;
        data[WINDOW_Y_POSITION_RW] = 0;
        data[WINDOW_X_POSITION_MINUS_7_RW] = 0;
        data[DISABLE_BOOT_ROM] = 0;

        Memory {
            data,
            game_rom: Vec::<u8>::new(),
            vram_accessible: true,
            oam_accessible: true,
        }
    }
    pub fn get(&self, addr: u16) -> u8 {
        if self.data[DISABLE_BOOT_ROM] > 0 || addr > 0xff {
            if addr < 0x8000 {
                self.game_rom[addr as usize]
            } else if addr < 0xa000 {
                // VRAM
                if self.vram_accessible {
                    self.data[addr as usize]
                } else {
                    0xff
                }
            } else {
                self.data[addr as usize]
            }
        } else {
            BOOT_ROM_BYTES[addr as usize]
        }
    }
    pub fn set(&mut self, addr: u16, val: u8) {
        self.data[addr as usize] = val;
    }
}
