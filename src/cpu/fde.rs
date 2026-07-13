use crate::{
    cpu::helpers::{af, alu, cc, r, rot, rp, rp2},
    memory::Memory,
    registers::{Condition, Register, RegisterPair},
};
use bitmatch::bitmatch;

use crate::cpu::Cpu;

#[allow(dead_code)]
impl Cpu {
    pub fn fetch(&mut self, memory: &mut Memory) -> u8 {
        let op = memory.read(self.pc);
        self.pc = if self.halt_bug {
            self.halt_bug = false;
            self.pc
        } else {
            self.pc.wrapping_add(1)
        };
        op
    }

    fn fetch_u16(&mut self, memory: &mut Memory) -> u16 {
        let low = self.fetch(memory);
        let high = self.fetch(memory);
        (u16::from(high) << 8) | u16::from(low)
    }

    #[allow(clippy::too_many_lines)]
    #[bitmatch]
    pub fn decode(&mut self, op: u8, memory: &mut Memory) {
        if op == 0xCB {
            let n = self.fetch(memory);
            self.decode_cb(n, memory);
            return;
        }
        #[bitmatch]
        match op {
            "00yyyzzz" => match z {
                0 => match y {
                    0 => {}
                    1 => {
                        let nn: u16 = self.fetch_u16(memory);
                        self.ld_nn_sp(nn, memory);
                    }
                    2 => {
                        self.fetch(memory);
                    }
                    3 => {
                        let n = self.fetch(memory).cast_signed();
                        self.jr(&Condition::None, n);
                    }
                    4..=7 => {
                        let n = self.fetch(memory).cast_signed();
                        self.jr(&cc(y), n);
                    }
                    _ => unreachable!(),
                },

                1 => {
                    let q = y % 2;
                    let p = y >> 1;
                    match q {
                        0 => {
                            let nn: u16 = self.fetch_u16(memory);
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
                        0 => memory.write(addr, self.registers.a),
                        1 => self.registers.a = memory.read(addr),
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
                        let val = memory.read(self.registers.hl());
                        let new_val = val.wrapping_add(1);
                        memory.write(self.registers.hl(), new_val);
                    } else {
                        self.inc_r(&r(y), memory);
                    }
                }

                5 => {
                    if y == 6 {
                        let val = memory.read(self.registers.hl());
                        let new_val = val.wrapping_sub(1);
                        memory.write(self.registers.hl(), new_val);
                    } else {
                        self.dec_r(&r(y), memory);
                    }
                }

                6 => {
                    let n = self.fetch(memory);
                    if y == 6 {
                        memory.write(self.registers.hl(), n);
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
                    self.halt(memory);
                } else if y != 6 && z == 6 {
                    self.ld_r_hl(&r(y), memory);
                } else if y == 6 && z != 6 {
                    self.ld_hl_r(&r(z), memory);
                } else {
                    self.ld_r_r(&r(y), &r(z));
                }
            }
            "10yyyzzz" => self.alu_op_r(&alu(y), &r(z), memory),
            "11yyyzzz" => match z {
                0 => match y {
                    0..=3 => {
                        self.ret(&cc(y), memory);
                    }
                    4 => {
                        let n = self.fetch(memory);
                        self.ld_addr_r(0xFF00 + u16::from(n), &Register::A, memory);
                    }
                    5 => {
                        let d = self.fetch(memory).cast_signed();
                        self.add_sp_d(d);
                    }
                    6 => {
                        let n = self.fetch(memory);
                        self.ld_r_addr(&Register::A, 0xFF00 + u16::from(n), memory);
                    }
                    7 => {
                        let d = self.fetch(memory).cast_signed();
                        self.ld_hl_sp_d(d);
                    }
                    _ => unreachable!(),
                },
                1 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.pop_rp2(&rp2(p), memory);
                    } else {
                        match p {
                            0 => {
                                self.ret(&Condition::None, memory);
                            }
                            1 => {
                                self.reti(memory);
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
                        let nn = self.fetch_u16(memory);
                        self.jp(&cc(y), nn);
                    }
                    4 => {
                        let addr = 0xFF00 + u16::from(self.registers.c);
                        self.ld_addr_r(addr, &Register::A, memory);
                    }
                    5 => {
                        let nn = self.fetch_u16(memory);
                        self.ld_nn_r(nn, &Register::A, memory);
                    }
                    6 => {
                        let addr = 0xFF00 + u16::from(self.registers.c);
                        self.ld_r_addr(&Register::A, addr, memory);
                    }
                    7 => {
                        let nn = self.fetch_u16(memory);
                        self.ld_r_nn(&Register::A, nn, memory);
                    }
                    _ => unreachable!(),
                },
                3 => match y {
                    0 => {
                        let nn = self.fetch_u16(memory);
                        self.jp(&Condition::None, nn);
                    }
                    1 => unimplemented!(),
                    6 => self.di(),
                    7 => self.ei(),
                    _ => unreachable!(),
                },
                4 => match y {
                    0..=3 => {
                        let nn = self.fetch_u16(memory);
                        self.call(&cc(y), nn, memory);
                    }
                    _ => unreachable!(),
                },
                5 => {
                    let q = y % 2;
                    let p = y >> 1;

                    if q == 0 {
                        self.push_rp2(&rp2(p), memory);
                    } else {
                        match p {
                            0 => {
                                let nn = self.fetch_u16(memory);
                                self.call(&Condition::None, nn, memory);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                6 => {
                    let n = self.fetch(memory);
                    self.alu_op_n(&alu(y), n);
                }
                7 => self.rst(y, memory),
                _ => unreachable!(),
            },

            _ => unimplemented!(),
        }
    }

    #[bitmatch]
    fn decode_cb(&mut self, n: u8, memory: &mut Memory) {
        #[bitmatch]
        match n {
            "xxyyyzzz" => match x {
                0 => self.rot_table(&rot(y, z), memory),
                1 => self.bit(y, &r(z), memory),
                2 => self.res(y, &r(z), memory),
                3 => self.set(y, &r(z), memory),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }
}
