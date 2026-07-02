use crate::{
    cpu::helpers::{af, alu, cc, r, rot, rp, rp2},
    registers::{Condition, Register, RegisterPair},
};
use bitmatch::bitmatch;

use crate::cpu::Cpu;

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
            self.pc.wrapping_add(1)
        };
        op
    }

    fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch();
        let high = self.fetch();
        (u16::from(high) << 8) | u16::from(low)
    }

    #[allow(clippy::too_many_lines)]
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
                        let n = self.fetch().cast_signed();
                        self.jr(&Condition::None, n);
                    }
                    4..=7 => {
                        let n = self.fetch().cast_signed();
                        self.jr(&cc(y), n);
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
                                0..=3 => self.registers.set_rp(&rp(p), nn),
                                _ => unreachable!(),
                            }
                        }
                        1 => match p {
                            0..=3 => self.registers.add_hl(&rp(p)),
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
                        2 => self.registers.inc_rp(&RegisterPair::HL),
                        3 => self.registers.dec_rp(&RegisterPair::HL),
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
                        0 => self.registers.inc_rp(&rp(p)),
                        1 => self.registers.dec_rp(&rp(p)),
                        _ => unreachable!(),
                    }
                }

                4 => {
                    if y == 6 {
                        let val = self.memory.read(self.registers.hl());
                        let new_val = val.wrapping_add(1);
                        self.memory.write(self.registers.hl(), new_val);
                    } else {
                        self.inc_r(&r(y));
                    }
                }

                5 => {
                    if y == 6 {
                        let val = self.memory.read(self.registers.hl());
                        let new_val = val.wrapping_sub(1);
                        self.memory.write(self.registers.hl(), new_val);
                    } else {
                        self.dec_r(&r(y));
                    }
                }

                6 => {
                    let n = self.fetch();
                    if y == 6 {
                        self.memory.write(self.registers.hl(), n);
                    } else {
                        self.registers.set_r(&r(y), n);
                    }
                }

                7 => {
                    self.acc_flags(&af(y));
                }
                _ => unreachable!(),
            },
            "01yyyzzz" => {
                if y == 6 && z == 6 {
                    self.halt();
                } else if y != 6 && z == 6 {
                    self.ld_r_hl(&r(y));
                } else if y == 6 && z != 6 {
                    self.ld_hl_r(&r(z));
                } else {
                    self.ld_r_r(&r(y), &r(z));
                }
            }
            "10yyyzzz" => self.alu_op_r(&alu(y), &r(z)),
            "11yyyzzz" => match z {
                0 => match y {
                    0..=3 => {
                        self.ret(&cc(y));
                    }
                    4 => {
                        let n = self.fetch();
                        self.ld_addr_r(0xFF00 + u16::from(n), &Register::A);
                    }
                    5 => {
                        let d = self.fetch().cast_signed();
                        self.add_sp_d(d);
                    }
                    6 => {
                        let n = self.fetch();
                        self.ld_r_addr(&Register::A, 0xFF00 + u16::from(n));
                    }
                    7 => {
                        let d = self.fetch().cast_signed();
                        self.ld_hl_sp_d(d);
                    }
                    _ => unreachable!(),
                },
                1 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.pop_rp2(&rp2(p));
                    } else {
                        match p {
                            0 => {
                                self.ret(&Condition::None);
                            }
                            1 => {
                                self.reti();
                            }
                            2 => {
                                let nn = self.registers.hl();
                                self.jp(&Condition::None, nn);
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
                        self.jp(&cc(y), nn);
                    }
                    4 => {
                        let addr = 0xFF00 + u16::from(self.registers.c);
                        self.ld_addr_r(addr, &Register::A);
                    }
                    5 => {
                        let nn = self.fetch_u16();
                        self.ld_nn_r(nn, &Register::A);
                    }
                    6 => {
                        let addr = 0xFF00 + u16::from(self.registers.c);
                        self.ld_r_addr(&Register::A, addr);
                    }
                    7 => {
                        let nn = self.fetch_u16();
                        self.ld_r_nn(&Register::A, nn);
                    }
                    _ => unreachable!(),
                },
                3 => match y {
                    0 => {
                        let nn = self.fetch_u16();
                        self.jp(&Condition::None, nn);
                    }
                    1 => unimplemented!(),
                    6 => self.di(),
                    7 => self.ei(),
                    _ => unreachable!(),
                },
                4 => match y {
                    0..=3 => {
                        let nn = self.fetch_u16();
                        self.call(&cc(y), nn);
                    }
                    _ => unreachable!(),
                },
                5 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.push_rp2(&rp2(p));
                    } else {
                        match p {
                            0 => {
                                let nn = self.fetch_u16();
                                self.call(&Condition::None, nn);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                6 => {
                    let n = self.fetch();
                    self.alu_op_n(&alu(y), n);
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
                0 => self.rot_table(&rot(y, z)),
                1 => self.bit(y, &r(z)),
                2 => self.res(y, &r(z)),
                3 => self.set(y, &r(z)),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
