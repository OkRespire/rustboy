#[allow(dead_code)]
pub struct Memory {
    rom: [u8; 16 * 1024],
    rom_bank: [u8; 16 * 1024],
    vram: [u8; 8 * 1024],
    ram: [u8; 8 * 1024],
    ex_ram: [u8; 8 * 1024],
    oam: [u8; 160],
    io: [u8; 128],
    hram: [u8; 127],
    ie: u8,
}

impl Default for Memory {
    fn default() -> Self {
        let mut io = [0u8; 128];
        io[15] = 0b1110_0001;

        Self {
            rom: [0; 16 * 1024],
            rom_bank: [0; 16 * 1024],
            vram: [0; 8 * 1024],
            ram: [0; 8 * 1024],
            ex_ram: [0; 8 * 1024],
            oam: [0; 160],
            io: io,
            hram: [0; 127],
            ie: 0b1110_0000,
        }
    }
}

#[allow(dead_code)]
impl Memory {
    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[addr as usize],
            0x4000..=0x7FFF => self.rom_bank[addr as usize - 0x4000],
            0x8000..=0x9FFF => self.vram[addr as usize - 0x8000],
            0xA000..=0xBFFF => self.ex_ram[addr as usize - 0xA000],
            0xC000..=0xDFFF => self.ram[addr as usize - 0xC000],
            0xE000..=0xFDFF => self.ram[addr as usize - 0x2000],
            0xFE00..=0xFE9F => self.oam[addr as usize - 0xFE00],
            0xFF00..=0xFF7F => self.io[addr as usize - 0xFF00],
            0xFF80..=0xFFFE => self.hram[addr as usize - 0xFF80],
            0xFFFF => self.ie,
            _ => 0xFF, // open bus, returns 0xFF on real hardware
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF => {
                self.rom[addr as usize] = val;
            }
            0x4000..=0x7FFF => {
                self.rom_bank[addr as usize - 0x4000] = val;
            }
            0x8000..=0x9FFF => {
                self.vram[addr as usize - 0x8000] = val;
            }
            0xA000..=0xBFFF => {
                self.ex_ram[addr as usize - 0xA000] = val;
            }
            0xC000..=0xDFFF => {
                self.ram[addr as usize - 0xC000] = val;
            }
            0xE000..=0xFDFF => {
                self.ram[addr as usize - 0x2000] = val;
            }
            0xFE00..=0xFE9F => {
                self.oam[addr as usize - 0xFE00] = val;
            }
            0xFF00..=0xFF7F => {
                self.io[addr as usize - 0xFF00] = val;
            }
            0xFF80..=0xFFFE => {
                self.hram[addr as usize - 0xFF80] = val;
            }
            0xFFFF => {
                self.ie = val;
            }
            _ => {}
        }
    }
}
