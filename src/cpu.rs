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
enum Alu {
    ADD,
    ADC,
    SUB,
    SBC,
    AND,
    XOR,
    OR,
    CP,
}

#[allow(dead_code)]
pub struct Cpu {
    registers: Registers,
    memory: Memory,
    pc: u16,
    halted: bool,
    halt_bug: bool,
    ime: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            memory: Memory::default(),
            pc: 0x0100,
            halted: false,
            halt_bug: false,
            ime: false,
        }
    }
}

#[allow(dead_code)]
impl Cpu {
    pub fn run(&mut self) {
        if self.halted {
            // interrupt handle
        } else {
            let op = self.fetch();
            self.decode(op);
        }
    }
    pub fn fetch(&mut self) -> u8 {
        let op = self.memory.read(self.pc);
        self.pc = if self.halt_bug {
            self.halt_bug = false;
            self.pc
        } else {
            self.pc + 1
        };
        op
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch();
        let high = self.fetch();
        ((high as u16) << 8) | low as u16
    }

    #[bitmatch]
    pub fn decode(&mut self, op: u8) {
        #[bitmatch]
        match op {
            "00yyyzzz" => match z {
                0 => match y {
                    0 => {}
                    1 => {
                        let nn: u16 = self.fetch_u16();
                        self.ld_nn_sp(nn);
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

                1 => {
                    let q = y % 2;
                    let p = y >> 1;
                    match q {
                        0 => {
                            let nn: u16 = self.fetch_u16();
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

                2 => {
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

                3 => {
                    let q = y % 2;
                    let p = y >> 1;
                    match q {
                        0 => self.registers.inc_rp(rp(p)),
                        1 => self.registers.dec_rp(rp(p)),
                        _ => unreachable!(),
                    }
                }

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
            "01yyyzzz" => {
                if y == 6 && z == 6 {
                    self.halt();
                } else {
                    self.ld_r_r(r(y), r(z));
                }
            }
            "10yyyzzz" => self.alu_op_r(alu(y), r(z)),

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

    fn alu_op_r(&mut self, alu: Alu, r: Register) {
        let n = self.registers.get_r(&r);
        match alu {
            Alu::ADD => self.add_a_n(n),
            Alu::ADC => self.adc_a_n(n),
            Alu::SUB => self.sub_n(n),
            Alu::SBC => self.sbc_n(n),
            Alu::AND => self.and_n(n),
            Alu::XOR => self.xor_n(n),
            Alu::OR => self.or_n(n),
            Alu::CP => self.cp_n(n),
        }
    }

    fn ld_nn_sp(&mut self, nn: u16) {
        self.memory.write(nn, (self.registers.sp & 0xFF) as u8);
        self.memory.write(nn + 1, (self.registers.sp >> 8) as u8);
    }

    fn ld_r_r(&mut self, r1: Register, r2: Register) {
        let r2_val = self.registers.get_r(&r2);
        self.registers.set_r(r1, r2_val);
    }

    fn halt(&mut self) {
        let pending = self.memory.read(0xFFFF) & self.memory.read(0xFF0F) != 0;
        if self.ime {
            self.halted = true;
        } else if !pending {
            self.halted = true;
        } else {
            self.halt_bug = true;
        }
    }

    fn add_a_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let (res, carry) = self.registers.a.overflowing_add(n);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (n & 0xF) + (old_a & 0xF) > 0xF;
    }

    fn adc_a_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let carry_in = self.registers.f.carry as u8;
        let (no_carry_res, carry1) = self.registers.a.overflowing_add(n);
        let (res, carry2) = no_carry_res.overflowing_add(carry_in);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry1 || carry2;
        self.registers.f.half_carry = (n & 0xF) + (old_a & 0xF) + carry_in > 0xF;
    }
    fn sub_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let (res, carry) = self.registers.a.overflowing_sub(n);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_a & 0xF) < (n & 0xF);
    }
    fn sbc_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let carry_in = self.registers.f.carry as u8;
        let (res1, carry1) = self.registers.a.overflowing_sub(n);
        let (res2, carry2) = res1.overflowing_sub(carry_in);
        self.registers.a = res2;
        self.registers.f.zero = res2 == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry1 || carry2;
        self.registers.f.half_carry = (old_a & 0xF) < (n & 0xF) + carry_in;
    }
    fn and_n(&mut self, n: u8) {
        self.registers.a = n & self.registers.a;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = true;
    }
    fn xor_n(&mut self, n: u8) {
        self.registers.a = n ^ self.registers.a;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
    }
    fn or_n(&mut self, n: u8) {
        self.registers.a = n | self.registers.a;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
    }
    fn cp_n(&mut self, n: u8) {
        let (res, carry) = self.registers.a.overflowing_sub(n);
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (n & 0xF);
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

fn alu(y: u8) -> Alu {
    match y {
        0 => Alu::ADD,
        1 => Alu::ADC,
        2 => Alu::SUB,
        3 => Alu::SBC,
        4 => Alu::AND,
        5 => Alu::XOR,
        6 => Alu::OR,
        7 => Alu::XOR,
        _ => unreachable!(),
    }
}
