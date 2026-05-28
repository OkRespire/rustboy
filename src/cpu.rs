use crate::{memory::Memory, registers::Registers};

enum Condition {
    NZ,
    Z,
    NC,
    C,
    None,
}

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

    pub fn decode(&mut self, op: u8) {
        let x = (op >> 6) & 0x03;
        let y = (op >> 3) & 0x07;
        let z = op & 0x07;

        let p = y << 1;
        let q = y % 2;

        match x {
            0 => match y {
                0 => match z {
                    0 => {}
                    1 => {}
                    2 => todo!(),
                    3 => todo!(),
                    4 => todo!(),
                    5 => todo!(),
                    6 => todo!(),
                    7 => todo!(),
                    _ => unreachable!("fail"),
                },
                1 => match z {
                    0 => {
                        let high = self.fetch();
                        let low = self.fetch();
                        let nn: u16 = ((high as u16) << 8) | low as u16;
                        self.sp = nn;
                    }
                    _ => todo!(),
                },
                2 => match z {
                    0 => {
                        self.fetch();
                    }
                    _ => todo!(),
                },
                3 => match z {
                    0 => {
                        let n = self.fetch() as i8;
                        self.jr(Condition::None, n);
                    }
                    _ => todo!(),
                },
                4 => match z {
                    0 => {
                        let n = self.fetch() as i8;
                        self.jr(Condition::NZ, n)
                    }
                    _ => todo!(),
                },

                5 => match z {
                    0 => {
                        let n = self.fetch() as i8;
                        self.jr(Condition::Z, n)
                    }
                    _ => todo!(),
                },

                6 => match z {
                    0 => {
                        let n = self.fetch() as i8;
                        self.jr(Condition::NC, n)
                    }
                    _ => todo!(),
                },

                7 => match z {
                    0 => {
                        let n = self.fetch() as i8;
                        self.jr(Condition::C, n)
                    }
                    _ => todo!(),
                },

                _ => unreachable!("fail"),
            },
            1 => todo!(),
            2 => todo!(),
            3 => todo!(),
            _ => unreachable!("opcode unusable"),
        }
    }

    fn jr(&mut self, cc: Condition, n: i8) {
        let should_jump = match cc {
            Condition::NZ => !self.registers.f.zero,
            Condition::Z => self.registers.f.zero,
            Condition::NC => !self.registers.f.carry,
            Condition::C => self.registers.f.carry,
            Condition::None => true,
        };
        if should_jump {
            self.pc = self.pc.wrapping_add_signed(n as i16);
        }
    }
}
