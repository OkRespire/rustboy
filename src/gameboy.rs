use crate::{cpu::Cpu, memory::Memory, ppu::Ppu};

pub struct GameBoy {
    cpu: Cpu,
    ppu: Ppu,
    memory: Memory,
}

impl Default for GameBoy {
    fn default() -> Self {
        Self {
            cpu: Cpu::default(),
            ppu: Ppu::default(),
            memory: Memory::default(),
        }
    }
}

impl GameBoy {
    pub fn run(&mut self) {
        let pending = self.memory.read(0xFFFF) & self.memory.read(0xFF0F);
        if pending != 0 {
            self.cpu.halted = false;
            for bit in 0u8..5u8 {
                if self.cpu.ime {
                    if pending & (1 << bit) != 0 {
                        let current_if = self.memory.read(0xFF0F);
                        self.cpu.ime = false;
                        self.cpu.push(self.cpu.pc, &mut self.memory);
                        self.memory.write(0xFF0F, current_if & !(1 << bit));
                        self.cpu.pc = 0x0040 + (8 * u16::from(bit));
                        break;
                    }
                }
            }
        }
        if !self.cpu.halted {
            let op = self.cpu.fetch(&mut self.memory);
            self.cpu.decode(op, &mut self.memory);
        }
    }
}
