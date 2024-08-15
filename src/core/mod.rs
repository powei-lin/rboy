pub mod cpu;
pub mod memory;
use std::fs;

pub struct Core {
    pub cpu: cpu::CPU,
    memory: memory::Memory,
}

impl Core {
    pub fn new(randomize: bool) -> Core {
        Core {
            cpu: cpu::CPU::new(),
            memory: memory::Memory::new(randomize),
        }
    }
    pub fn load_game_rom(&mut self, game_rom_path: &str) {
        let game_rom = fs::read(game_rom_path).expect("game rom path");
        self.memory.game_rom = game_rom;
    }
    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.memory);
    }
}
