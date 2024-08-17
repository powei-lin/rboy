pub mod constants;
pub mod cpu;
pub mod memory;
pub mod ppu;
use std::fs;

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
    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.mem);
    }
    pub fn get_new_frame_buffer() {}
}
