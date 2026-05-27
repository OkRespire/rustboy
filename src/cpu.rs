use crate::{memory::Memory, registers::Registers};

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
    sp: u16,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::default(),
            pc: 0x0100,
            sp: 0xFFFE,
        }
    }
}

#[allow(dead_code)]
impl Cpu {
    pub fn fetch(&mut self) -> u8 {
        let op = self.memory.read(self.pc);
        self.pc += 1;
        op
    }

    pub fn decode(&self, op: u8) {
        let x = (op >> 6) & 0x03;
        let y = (op >> 3) & 0x07;
        let z = op & 0x07;
        todo!()
    }
}
