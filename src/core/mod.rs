pub mod cpu;
pub mod memory;

pub struct Core {
    cpu: cpu::CPU,
    memory: memory::Memory,
}

impl Core {
    pub fn new(randomize: bool) -> Core {
        Core {
            cpu: cpu::CPU::new(),
            memory: memory::Memory::new(randomize),
        }
    }
    pub fn tick(&mut self) {
        self.cpu.tick(&mut self.memory);
    }
}
