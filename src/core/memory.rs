use rand::{self, RngCore};

const BOOT_ROM_BYTES: &[u8; 256] = include_bytes!("DMG_ROM.bin");
const RAM_SIZE: usize = 2usize.pow(16);

pub struct Memory {
    data: [u8; RAM_SIZE],
    // VRAM and OAM access
    vram_accessible: bool,
    oam_accessible: bool,
    initialized: bool,
}

impl Memory {
    pub fn new(randomize: bool) -> Memory {
        let mut data = [0; RAM_SIZE];
        if randomize {
            rand::thread_rng().fill_bytes(&mut data);
        }

        // io map is not ramdom
        data[0xff40] = 0;
        data[0xff41] = 0x84;
        data[0xff42] = 0;
        data[0xff43] = 0;
        data[0xff44] = 0;
        data[0xff45] = 0;
        data[0xff46] = 0xff;
        data[0xff47] = 0xfc;
        data[0xff48] = 0xff;
        data[0xff49] = 0xff;
        data[0xff4a] = 0;
        data[0xff4b] = 0;

        Memory {
            data,
            vram_accessible: true,
            oam_accessible: true,
            initialized: false,
        }
    }
    pub fn get(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
}
