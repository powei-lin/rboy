pub mod constants;
pub mod cpu;
pub mod memory;
pub mod ppu;
use std::fs;

use constants::DISABLE_BOOT_ROM;

pub struct Core {
    pub cpu: cpu::CPU,
    mem: memory::Memory,
    ppu: ppu::PPU,
}

impl Core {
    pub fn new(randomize: bool) -> Core {
        Core {
            cpu: cpu::CPU::new(),
            mem: memory::Memory::new(randomize),
            ppu: ppu::PPU::new(),
        }
    }
    pub fn load_game_rom(&mut self, game_rom_path: &str) {
        let game_rom = fs::read(game_rom_path).expect("game rom path");
        self.mem.game_rom = game_rom;
    }
    pub fn tick(&mut self, break_point_option: Option<u16>) -> bool {
        // println!("cpu start");
        // let cpu_before_tick = self.cpu.clone();
        let cpu_cycle_in_4mhz = self.cpu.tick(&mut self.mem, break_point_option);
        if cpu_cycle_in_4mhz == 0 {
            return true;
        }
        // if cpu_before_tick == self.cpu{
        //     panic!("cpu is not changing. {}", self.cpu);
        // }
        // println!("cpu end");
        self.ppu.tick(&mut self.mem, cpu_cycle_in_4mhz)
    }
    pub fn get_new_frame_buffer(&self) {}
    pub fn get_bg_frame_buffer(&self) -> &Vec<u8> {
        self.ppu.bg_frame_buffer()
    }
    pub fn get_tiles_frame_buffer(&self) -> &Vec<u8> {
        self.ppu.tiles_frame_buffer()
    }
}
