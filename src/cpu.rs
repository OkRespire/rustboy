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

enum CBRot {
    RLC(Register),
    RRC(Register),
    RL(Register),
    RR(Register),
    SLA(Register),
    SRA(Register),
    SWAP(Register),
    SRL(Register),
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

    fn pop(&mut self) -> u16 {
        let low = self.memory.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = self.memory.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        ((high as u16) << 8) | low as u16
    }

    fn push(&mut self, nn: u16) {
        let low = (nn & 0xFF) as u8;
        let high = (nn >> 8) as u8;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory.write(self.registers.sp, high);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory.write(self.registers.sp, low);
    }

    fn pop_rp2(&mut self, rp: RegisterPair) {
        let val = self.pop();
        self.registers.set_rp(rp, val);
    }

    fn push_rp2(&mut self, rp: RegisterPair) {
        let val = self.registers.get_rp(&rp);
        let low = (val & 0xFF) as u8;
        let high = (val >> 8) as u8;

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory.write(self.registers.sp, high);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        self.memory.write(self.registers.sp, low);
    }

    #[bitmatch]
    pub fn decode(&mut self, op: u8) {
        if op == 0xCB {
            let n = self.fetch();
            self.decode_cb(n);
            return;
        }
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
                    let addr = if p == 2 || p == 3 {
                        self.registers.get_rp(&RegisterPair::HL)
                    } else {
                        self.registers.get_rp(&rp(p))
                    };

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
                } else if y != 6 && z == 6 {
                    self.ld_r_hl(r(y))
                } else if y == 6 && z != 6 {
                    self.ld_hl_r(r(z))
                } else {
                    self.ld_r_r(r(y), r(z));
                }
            }
            "10yyyzzz" => self.alu_op_r(alu(y), r(z)),
            "11yyyzzz" => match z {
                0 => match y {
                    0..=3 => {
                        self.ret(cc(y));
                    }
                    4 => {
                        let n = self.fetch();
                        self.ld_addr_r(0xFF00 + n as u16, Register::A);
                    }
                    5 => {
                        let d = self.fetch() as i8;
                        self.add_sp_d(d);
                    }
                    6 => {
                        let n = self.fetch();
                        self.ld_r_addr(Register::A, 0xFF00 + n as u16)
                    }
                    7 => {
                        let d = self.fetch() as i8;
                        self.ld_hl_sp_d(d);
                    }
                    _ => unreachable!(),
                },
                1 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.pop_rp2(rp2(p));
                    } else {
                        match p {
                            0 => {
                                self.ret(Condition::None);
                            }
                            1 => {
                                self.reti();
                            }
                            2 => {
                                let nn = self.registers.hl();
                                self.jp(Condition::None, nn);
                            }
                            3 => {
                                self.registers.sp = self.registers.hl();
                            }

                            _ => unreachable!(),
                        }
                    }
                }
                2 => match y {
                    0..=3 => {
                        let nn = self.fetch_u16();
                        self.jp(cc(y), nn);
                    }
                    4 => {
                        let addr = 0xFF00 + self.registers.c as u16;
                        self.ld_addr_r(addr, Register::A)
                    }
                    5 => {
                        let nn = self.fetch_u16();
                        self.ld_nn_r(nn, Register::A);
                    }
                    6 => {
                        let addr = 0xFF00 + self.registers.c as u16;
                        self.ld_r_addr(Register::A, addr);
                    }
                    7 => {
                        let nn = self.fetch_u16();
                        self.ld_r_nn(Register::A, nn);
                    }
                    _ => unreachable!(),
                },
                3 => match y {
                    0 => {
                        let nn = self.fetch_u16();
                        self.jp(Condition::None, nn);
                    }
                    1 => unimplemented!(),
                    6 => self.di(),
                    7 => self.ei(),
                    _ => unreachable!(),
                },
                4 => match y {
                    0..=3 => {
                        let nn = self.fetch_u16();
                        self.call(cc(y), nn);
                    }
                    _ => unreachable!(),
                },
                5 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.push_rp2(rp2(p));
                    } else {
                        match p {
                            0 => {
                                let nn = self.fetch_u16();
                                self.call(Condition::None, nn);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                6 => {
                    let n = self.fetch();
                    self.alu_op_n(alu(y), n);
                }
                7 => self.rst(y),
                _ => unreachable!(),
            },

            _ => unimplemented!(),
        }
    }

    #[bitmatch]
    fn decode_cb(&mut self, n: u8) {
        #[bitmatch]
        match n {
            "xxyyyzzz" => match x {
                0 => {
                    if z == 6 {
                        self.rot_table_hl(rot(y, 6));
                    } else {
                    }
                }
                1 => todo!(),
                2 => todo!(),
                3 => todo!(),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    fn cc_match(&self, cc: Condition) -> bool {
        match cc {
            Condition::NZ => !self.registers.f.zero,
            Condition::Z => self.registers.f.zero,
            Condition::NC => !self.registers.f.carry,
            Condition::C => self.registers.f.carry,
            Condition::None => true,
        }
    }
    fn jr(&mut self, cc: Condition, n: i8) {
        if self.cc_match(cc) {
            self.pc = self.pc.wrapping_add_signed(n as i16);
        }
    }

    fn jp(&mut self, cc: Condition, nn: u16) {
        if self.cc_match(cc) {
            self.pc = nn;
        }
    }

    fn call(&mut self, cc: Condition, nn: u16) {
        if self.cc_match(cc) {
            self.push(self.pc);
            self.pc = nn;
        }
    }
    fn rst(&mut self, y: u8) {
        self.push(self.pc);
        self.pc = (y * 8) as u16;
    }

    fn ret(&mut self, cc: Condition) {
        if self.cc_match(cc) {
            self.pc = self.pop();
        }
    }

    fn reti(&mut self) {
        self.ei();
        self.ret(Condition::None)
    }

    fn ei(&mut self) {
        self.ime = true;
    }
    fn di(&mut self) {
        self.ime = false;
    }

    fn acc_flags(&mut self, af: AccFlag) {
        match af {
            AccFlag::RLCA => self.rotate_left_a(false),
            AccFlag::RRCA => self.rotate_right_a(false),
            AccFlag::RLA => self.rotate_left_a(true),
            AccFlag::RRA => self.rotate_right_a(true),
            AccFlag::DAA => self.registers.daa(),
            AccFlag::CPL => self.registers.cpl(),
            AccFlag::SCF => self.registers.scf(),
            AccFlag::CCF => self.registers.ccf(),
        }
    }

    fn rot_table(&mut self, rot: CBRot) {
        match rot {
            CBRot::RLC(register) => self.rotate_left(register, false),
            CBRot::RRC(register) => self.rotate_right(register, false),
            CBRot::RL(register) => self.rotate_left(register, true),
            CBRot::RR(register) => self.rotate_right(register, true),
            CBRot::SLA(register) => todo!(),
            CBRot::SRA(register) => todo!(),
            CBRot::SWAP(register) => todo!(),

            CBRot::SRL(register) => todo!(),
        }
    }

    fn alu_op_r(&mut self, alu: Alu, r: Register) {
        let n = match r {
            Register::HLDirect => self.memory.read(self.registers.hl()),
            _ => self.registers.get_r(&r),
        };
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
    fn alu_op_n(&mut self, alu: Alu, n: u8) {
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
    fn ld_hl_sp_d(&mut self, d: i8) {
        let old_sp_low = self.registers.sp as u8;
        let d_unsigned = d as u8;

        let new_sp = self.registers.sp.wrapping_add_signed(d as i16);

        let (_, carry) = old_sp_low.overflowing_add(d_unsigned);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_sp_low & 0xF) + (d_unsigned & 0xF) > 0xF;
        self.registers.set_hl(new_sp);
    }
    fn ld_r_hl(&mut self, r: Register) {
        let val = self.memory.read(self.registers.hl());
        self.registers.set_r(r, val);
    }
    fn ld_hl_r(&mut self, r: Register) {
        let val = self.registers.get_r(&r);
        self.memory.write(self.registers.hl(), val);
    }
    fn ld_nn_r(&mut self, nn: u16, r: Register) {
        let reg = self.registers.get_r(&r);
        self.memory.write(nn, reg);
    }

    fn ld_r_nn(&mut self, r: Register, nn: u16) {
        let val = self.memory.read(nn);
        self.registers.set_r(r, val);
    }

    fn ld_r_addr(&mut self, r: Register, addr: u16) {
        self.registers.set_r(r, self.memory.read(addr));
    }
    fn ld_addr_r(&mut self, addr: u16, r: Register) {
        let reg = self.registers.get_r(&r);
        self.memory.write(addr, reg);
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

    fn add_sp_d(&mut self, d: i8) {
        let old_sp_low = self.registers.sp as u8;
        let d_unsigned = d as u8;

        self.registers.sp = self.registers.sp.wrapping_add_signed(d as i16);

        let (_, carry) = old_sp_low.overflowing_add(d_unsigned);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_sp_low & 0xF) + (d_unsigned & 0xF) > 0xF;
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

    fn rotate_left(&mut self, r: Register, through_carry: bool) {
        let val = match r {
            Register::HLDirect => self.memory.read(self.registers.hl()),
            _ => self.registers.get_r(&r),
        };
        let bit7 = val >> 7;
        let carry_in = if through_carry {
            self.registers.f.carry as u8
        } else {
            bit7
        };

        let res = (val << 1) | carry_in;
        match r {
            Register::HLDirect => self.memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        };
        self.registers.f.carry = bit7 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }
    fn rotate_left_a(&mut self, through_carry: bool) {
        let val = self.registers.a;
        let bit7 = val >> 7;
        let carry_in = if through_carry {
            self.registers.f.carry as u8
        } else {
            bit7
        };

        let res = (val << 1) | carry_in;
        self.registers.set_r(Register::A, res);
        self.registers.f.carry = bit7 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = false;
    }

    fn rotate_right(&mut self, r: Register, through_carry: bool) {
        let val = match r {
            Register::HLDirect => self.memory.read(self.registers.hl()),
            _ => self.registers.get_r(&r),
        };
        let bit0 = val & 1;
        let carry_in = if through_carry {
            self.registers.f.carry as u8
        } else {
            bit0
        };
        let res = (carry_in << 7) | val >> 1;
        match r {
            Register::HLDirect => self.memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        };
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }
    fn rotate_right_a(&mut self, through_carry: bool) {
        let val = self.registers.a;
        let bit0 = val & 1;
        let carry_in = if through_carry {
            self.registers.f.carry as u8
        } else {
            bit0
        };
        let res = (carry_in << 7) | val >> 1;
        self.registers.set_r(Register::A, res);
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = false;
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

fn rp2(p: u8) -> RegisterPair {
    match p {
        0 => RegisterPair::BC,
        1 => RegisterPair::DE,
        2 => RegisterPair::HL,
        3 => RegisterPair::AF,
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
        6 => Register::HLDirect,
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
        7 => Alu::CP,
        _ => unreachable!(),
    }
}

fn rot(y: u8, z: u8) -> CBRot {
    match y {
        0 => CBRot::RLC(r(z)),
        1 => CBRot::RRC(r(z)),
        2 => CBRot::RL(r(z)),
        3 => CBRot::RR(r(z)),
        4 => CBRot::SLA(r(z)),
        5 => CBRot::SRA(r(z)),
        6 => CBRot::SWAP(r(z)),
        7 => CBRot::SRL(r(z)),
        _ => unreachable!(),
    }
}
