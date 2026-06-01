use bitmatch::bitmatch;

use crate::{
    memory::Memory,
    registers::{Register, RegisterPair, Registers},
};

#[allow(dead_code)]
enum Condition {
    NZ,
    Z,
    NC,
    C,
    None,
}
#[allow(dead_code)]
enum AccFlag {
    RLCA,
    RRCA,
    RLA,
    RRA,
    DAA,
    CPL,
    SCF,
    CCF,
}

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::default(),
            pc: 0x0100,
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
                    self.memory.write(nn, (self.registers.sp & 0xFF) as u8);
                    self.memory.write(nn + 1, (self.registers.sp >> 8) as u8);
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
                            0..=3 => self.registers.set_rp(rp(p), nn),
                            _ => unreachable!(),
                        }
                    }
                    1 => match p {
                        0..=3 => self.registers.add_hl(rp(p)),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            "00yyy010" => {
                let q = y % 2;
                let p = y >> 1;
                let addr = self.registers.get_rp(&rp(p));
                match p {
                    2 => self.registers.inc_rp(RegisterPair::HL),
                    3 => self.registers.dec_rp(RegisterPair::HL),
                    _ => {}
                }
                match q {
                    0 => self.memory.write(addr, self.registers.a),
                    1 => self.registers.a = self.memory.read(addr),
                    _ => unreachable!(),
                }
            }
            "00yyy011" => {
                let q = y % 2;
                let p = y >> 1;
                match q {
                    0 => self.registers.inc_rp(rp(p)),
                    1 => self.registers.dec_rp(rp(p)),
                    _ => unreachable!(),
                }
            }
            "00yyyzzz" => match z {
                4 => {
                    if y == 6 {
                        let val = self.memory.read(self.registers.hl());
                        let new_val = val.wrapping_add(1);
                        self.memory.write(self.registers.hl(), new_val);
                    } else {
                        self.registers.inc_r(r(y))
                    }
                }
                5 => {
                    if y == 6 {
                        let val = self.memory.read(self.registers.hl());
                        let new_val = val.wrapping_sub(1);
                        self.memory.write(self.registers.hl(), new_val);
                    } else {
                        self.registers.dec_r(r(y))
                    }
                }
                6 => {
                    let n = self.fetch();
                    if y == 6 {
                        self.memory.write(self.registers.hl(), n);
                    } else {
                        self.registers.set_r(r(y), n)
                    }
                }
                7 => {
                    self.acc_flags(af(y));
                }
                _ => unreachable!(),
            },
            _ => unimplemented!(),
        }
    }

    fn acc_flags(&mut self, af: AccFlag) {
        match af {
            AccFlag::RLCA => self.registers.rotate_left(Register::A, false),
            AccFlag::RRCA => self.registers.rotate_right(Register::A, false),
            AccFlag::RLA => self.registers.rotate_left(Register::A, true),
            AccFlag::RRA => self.registers.rotate_right(Register::A, true),
            AccFlag::DAA => self.registers.daa(),
            AccFlag::CPL => self.registers.cpl(),
            AccFlag::SCF => self.registers.scf(),
            AccFlag::CCF => self.registers.ccf(),
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

fn rp(p: u8) -> RegisterPair {
    match p {
        0 => RegisterPair::BC,
        1 => RegisterPair::DE,
        2 | 6 => RegisterPair::HL,
        3 => RegisterPair::SP,
        _ => unreachable!(),
    }
}

fn r(y: u8) -> Register {
    match y {
        0 => Register::B,
        1 => Register::C,
        2 => Register::D,
        3 => Register::E,
        4 => Register::H,
        5 => Register::L,
        7 => Register::A,
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

fn af(y: u8) -> AccFlag {
    match y {
        0 => AccFlag::RLCA,
        1 => AccFlag::RRCA,
        2 => AccFlag::RLA,
        3 => AccFlag::RRA,
        4 => AccFlag::DAA,
        5 => AccFlag::CPL,
        6 => AccFlag::SCF,
        7 => AccFlag::CCF,
        _ => unreachable!(),
    }
}
