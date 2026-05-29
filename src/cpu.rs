use bitmatch::bitmatch;

use crate::{memory::Memory, registers::Registers};

#[allow(dead_code)]
enum Condition {
    NZ,
    Z,
    NC,
    C,
    None,
}

#[allow(dead_code)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[allow(dead_code)]
enum RegisterPair {
    AF,
    BC,
    DE,
    HL,
    SP,
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

    #[bitmatch]
    pub fn decode(&mut self, op: u8) {
        #[bitmatch]
        match op {
            "00yyy000" => match y {
                0 => {}
                1 => {
                    let low = self.fetch();
                    let high = self.fetch();
                    let nn: u16 = ((high as u16) << 8) | low as u16;
                    self.memory.write(nn, (self.sp & 0xFF) as u8);
                    self.memory.write(nn + 1, (self.sp >> 8) as u8);
                }
                2 => {
                    self.fetch();
                }
                3 => {
                    let n = self.fetch() as i8;
                    self.jr(Condition::None, n);
                }
                4..=7 => {
                    let n = self.fetch() as i8;
                    self.jr(cc(y), n);
                }
                _ => unreachable!(),
            },
            "00yyy001" => {
                let q = y % 2;
                let p = y >> 1;
                match q {
                    0 => {
                        let low = self.fetch();
                        let high = self.fetch();
                        let nn: u16 = ((high as u16) << 8) | low as u16;

                        match p {
                            0 => self.registers.set_bc(nn),
                            1 => self.registers.set_de(nn),
                            2 => self.registers.set_hl(nn),
                            3 => self.sp = nn,

                            _ => unreachable!(),
                        }
                    }
                    1 => match p {
                        0..=3 => {
                            let rp = rp(p);
                            self.add_hl(rp);
                        }
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            _ => unimplemented!(),
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

    fn add_hl(&mut self, r2: RegisterPair) {
        let hl = self.registers.hl();
        let r2_val = self.get_register_pair(r2);
        let (new_value, did_overflow) = hl.overflowing_add(r2_val);
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (hl & 0xFFF) + (r2_val & 0xFFF) > 0xFFF;
        self.registers.set_hl(new_value);
    }

    fn get_register_pair(&self, r: RegisterPair) -> u16 {
        match r {
            RegisterPair::AF => self.registers.af(),
            RegisterPair::BC => self.registers.bc(),
            RegisterPair::DE => self.registers.de(),
            RegisterPair::HL => self.registers.hl(),
            RegisterPair::SP => self.sp,
        }
    }
}

fn rp(p: u8) -> RegisterPair {
    match p {
        0 => RegisterPair::BC,
        1 => RegisterPair::DE,
        2 => RegisterPair::HL,
        3 => RegisterPair::SP,
        _ => unreachable!(),
    }
}

fn cc(y: u8) -> Condition {
    match y {
        4 => Condition::NZ,
        5 => Condition::Z,
        6 => Condition::NC,
        7 => Condition::C,
        _ => unreachable!(),
    }
}
