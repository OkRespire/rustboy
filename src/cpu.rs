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
                            0..=3 => self.set_rp(rp(p), nn),
                            _ => unreachable!(),
                        }
                    }
                    1 => match p {
                        0..=3 => self.add_hl(rp(p)),
                        _ => unreachable!(),
                    },
                    _ => unreachable!(),
                }
            }
            "00yyy010" => {
                let q = y % 2;
                let p = y >> 1;
                let addr = self.get_rp(rp(p));
                match p {
                    2 => self.inc_rp(RegisterPair::HL),
                    3 => self.dec_rp(RegisterPair::HL),
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
                    0 => self.inc_rp(rp(p)),
                    1 => self.dec_rp(rp(p)),
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
                        self.inc_r(r(y))
                    }
                }
                5 => {
                    if y == 6 {
                        let val = self.memory.read(self.registers.hl());
                        let new_val = val.wrapping_sub(1);
                        self.memory.write(self.registers.hl(), new_val);
                    } else {
                        self.dec_r(r(y))
                    }
                }
                6 => {
                    let n = self.fetch();
                    if y == 6 {
                        self.memory.write(self.registers.hl(), n);
                    } else {
                        self.set_r(r(y), n)
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

    fn acc_flags(&self, af: AccFlag) {
        match af {
            AccFlag::RLCA => todo!(),
            AccFlag::RRCA => todo!(),
            AccFlag::RLA => todo!(),
            AccFlag::RRA => todo!(),
            AccFlag::DAA => todo!(),
            AccFlag::CPL => todo!(),
            AccFlag::SCF => todo!(),
            AccFlag::CCF => todo!(),
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
        let r2_val = self.get_rp(r2);
        let (new_value, did_overflow) = hl.overflowing_add(r2_val);
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        self.registers.f.half_carry = (hl & 0xFFF) + (r2_val & 0xFFF) > 0xFFF;
        self.registers.set_hl(new_value);
    }

    fn set_rp(&mut self, rp: RegisterPair, nn: u16) {
        match rp {
            RegisterPair::BC => self.registers.set_bc(nn),
            RegisterPair::DE => self.registers.set_de(nn),
            RegisterPair::HL => self.registers.set_hl(nn),
            RegisterPair::SP => self.sp = nn,
            _ => unreachable!(),
        }
    }

    fn inc_rp(&mut self, rp: RegisterPair) {
        match rp {
            RegisterPair::BC => self.registers.set_bc(self.registers.bc().wrapping_add(1)),
            RegisterPair::DE => self.registers.set_de(self.registers.de().wrapping_add(1)),
            RegisterPair::HL => self.registers.set_hl(self.registers.hl().wrapping_add(1)),
            RegisterPair::SP => self.sp = self.sp.wrapping_add(1),
            _ => unreachable!(),
        }
    }

    fn dec_rp(&mut self, rp: RegisterPair) {
        match rp {
            RegisterPair::BC => self.registers.set_bc(self.registers.bc().wrapping_sub(1)),
            RegisterPair::DE => self.registers.set_de(self.registers.de().wrapping_sub(1)),
            RegisterPair::HL => self.registers.set_hl(self.registers.hl().wrapping_sub(1)),
            RegisterPair::SP => self.sp = self.sp.wrapping_sub(1),
            _ => unreachable!(),
        }
    }

    fn get_rp(&self, rp: RegisterPair) -> u16 {
        match rp {
            RegisterPair::AF => self.registers.af(),
            RegisterPair::BC => self.registers.bc(),
            RegisterPair::DE => self.registers.de(),
            RegisterPair::HL => self.registers.hl(),
            RegisterPair::SP => self.sp,
        }
    }
    fn set_r(&mut self, r: Register, val: u8) {
        match r {
            Register::A => self.registers.a = val,
            Register::B => self.registers.b = val,
            Register::C => self.registers.c = val,
            Register::D => self.registers.d = val,
            Register::E => self.registers.e = val,
            Register::H => self.registers.h = val,
            Register::L => self.registers.l = val,
        }
    }

    fn inc_r(&mut self, r: Register) {
        match r {
            Register::A => self.registers.a = self.registers.a.wrapping_add(1),
            Register::B => self.registers.b = self.registers.b.wrapping_add(1),
            Register::C => self.registers.c = self.registers.c.wrapping_add(1),
            Register::D => self.registers.d = self.registers.d.wrapping_add(1),
            Register::E => self.registers.e = self.registers.e.wrapping_add(1),
            Register::H => self.registers.h = self.registers.h.wrapping_add(1),
            Register::L => self.registers.l = self.registers.l.wrapping_add(1),
        }
    }

    fn dec_r(&mut self, r: Register) {
        match r {
            Register::A => self.registers.a = self.registers.a.wrapping_sub(1),
            Register::B => self.registers.b = self.registers.b.wrapping_sub(1),
            Register::C => self.registers.c = self.registers.c.wrapping_sub(1),
            Register::D => self.registers.d = self.registers.d.wrapping_sub(1),
            Register::E => self.registers.e = self.registers.e.wrapping_sub(1),
            Register::H => self.registers.h = self.registers.h.wrapping_sub(1),
            Register::L => self.registers.l = self.registers.l.wrapping_sub(1),
        }
    }

    fn get_r(&self, r: Register) -> u8 {
        match r {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::H => self.registers.h,
            Register::L => self.registers.l,
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
