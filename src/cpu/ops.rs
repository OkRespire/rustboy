use crate::{
    cpu::{AccFlag, Alu, CBRot, Cpu},
    memory::Memory,
    registers::{Condition, Register, RegisterPair},
};

impl Cpu {
    fn pop(&mut self, memory: &mut Memory) -> u16 {
        let low = memory.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        let high = memory.read(self.registers.sp);
        self.registers.sp = self.registers.sp.wrapping_add(1);
        (u16::from(high) << 8) | u16::from(low)
    }

    pub fn push(&mut self, nn: u16, memory: &mut Memory) {
        let low = (nn & 0xFF) as u8;
        let high = (nn >> 8) as u8;
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        memory.write(self.registers.sp, high);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        memory.write(self.registers.sp, low);
    }
    pub fn pop_rp2(&mut self, rp: &RegisterPair, memory: &mut Memory) {
        let val = self.pop(memory);
        self.registers.set_rp(rp, val);
    }

    pub fn push_rp2(&mut self, rp: &RegisterPair, memory: &mut Memory) {
        let val = self.registers.get_rp(rp);
        let low = (val & 0xFF) as u8;
        let high = (val >> 8) as u8;

        self.registers.sp = self.registers.sp.wrapping_sub(1);
        memory.write(self.registers.sp, high);
        self.registers.sp = self.registers.sp.wrapping_sub(1);
        memory.write(self.registers.sp, low);
    }

    pub fn jr(&mut self, cc: &Condition, n: i8) {
        if self.registers.cc_match(cc) {
            self.pc = self.pc.wrapping_add_signed(i16::from(n));
        }
    }

    pub fn jp(&mut self, cc: &Condition, nn: u16) {
        if self.registers.cc_match(cc) {
            self.pc = nn;
        }
    }

    pub fn call(&mut self, cc: &Condition, nn: u16, memory: &mut Memory) {
        if self.registers.cc_match(cc) {
            self.push(self.pc, memory);
            self.pc = nn;
        }
    }
    pub fn rst(&mut self, y: u8, memory: &mut Memory) {
        self.push(self.pc, memory);
        self.pc = u16::from(y * 8);
    }

    pub fn ret(&mut self, cc: &Condition, memory: &mut Memory) {
        if self.registers.cc_match(cc) {
            self.pc = self.pop(memory);
        }
    }

    pub fn reti(&mut self, memory: &mut Memory) {
        self.ei();
        self.ret(&Condition::None, memory);
    }

    pub fn ei(&mut self) {
        self.ime = true;
    }
    pub fn di(&mut self) {
        self.ime = false;
    }
    pub fn bit(&mut self, y: u8, r: &Register, memory: &mut Memory) {
        let n = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        self.registers.f.zero = (n >> y) & 1 == 0;
        self.registers.f.half_carry = true;
        self.registers.f.subtract = false;
    }

    pub fn acc_flags(&mut self, af: &AccFlag) {
        match af {
            AccFlag::Rlca => self.rotate_left_a(false),
            AccFlag::Rrca => self.rotate_right_a(false),
            AccFlag::Rla => self.rotate_left_a(true),
            AccFlag::Rra => self.rotate_right_a(true),
            AccFlag::Daa => self.registers.daa(),
            AccFlag::Cpl => self.registers.cpl(),
            AccFlag::Scf => self.registers.scf(),
            AccFlag::Ccf => self.registers.ccf(),
        }
    }

    pub fn rot_table(&mut self, rot: &CBRot, memory: &mut Memory) {
        match rot {
            CBRot::Rlc(register) => self.rotate_left(register, false, memory),
            CBRot::Rrc(register) => self.rotate_right(register, false, memory),
            CBRot::Rl(register) => self.rotate_left(register, true, memory),
            CBRot::Rr(register) => self.rotate_right(register, true, memory),
            CBRot::Sla(register) => self.sla(register, memory),
            CBRot::Sra(register) => self.sra(register, memory),
            CBRot::Swap(register) => self.swap(register, memory),
            CBRot::Srl(register) => self.srl(register, memory),
        }
    }

    pub fn alu_op_r(&mut self, alu: &Alu, r: &Register, memory: &mut Memory) {
        let n = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        match alu {
            Alu::Add => self.add_a_n(n),
            Alu::Adc => self.adc_a_n(n),
            Alu::Sub => self.sub_n(n),
            Alu::Sbc => self.sbc_n(n),
            Alu::And => self.and_n(n),
            Alu::Xor => self.xor_n(n),
            Alu::Or => self.or_n(n),
            Alu::Cp => self.cp_n(n),
        }
    }
    pub fn alu_op_n(&mut self, alu: &Alu, n: u8) {
        match alu {
            Alu::Add => self.add_a_n(n),
            Alu::Adc => self.adc_a_n(n),
            Alu::Sub => self.sub_n(n),
            Alu::Sbc => self.sbc_n(n),
            Alu::And => self.and_n(n),
            Alu::Xor => self.xor_n(n),
            Alu::Or => self.or_n(n),
            Alu::Cp => self.cp_n(n),
        }
    }

    pub fn ld_nn_sp(&mut self, nn: u16, memory: &mut Memory) {
        memory.write(nn, (self.registers.sp & 0xFF) as u8);
        memory.write(nn + 1, (self.registers.sp >> 8) as u8);
    }
    #[allow(clippy::cast_possible_truncation)]
    pub fn ld_hl_sp_d(&mut self, d: i8) {
        let old_sp_low = self.registers.sp as u8;
        let d_unsigned = d.cast_unsigned();

        let new_sp = self.registers.sp.wrapping_add_signed(i16::from(d));

        let (_, carry) = old_sp_low.overflowing_add(d_unsigned);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_sp_low & 0xF) + (d_unsigned & 0xF) > 0xF;
        self.registers.set_hl(new_sp);
    }
    pub fn ld_r_hl(&mut self, r: &Register, memory: &mut Memory) {
        let val = memory.read(self.registers.hl());
        self.registers.set_r(r, val);
    }
    pub fn ld_hl_r(&mut self, r: &Register, memory: &mut Memory) {
        let val = self.registers.get_r(r);
        memory.write(self.registers.hl(), val);
    }
    pub fn ld_nn_r(&mut self, nn: u16, r: &Register, memory: &mut Memory) {
        let reg = self.registers.get_r(r);
        memory.write(nn, reg);
    }

    pub fn ld_r_nn(&mut self, r: &Register, nn: u16, memory: &mut Memory) {
        let val = memory.read(nn);
        self.registers.set_r(r, val);
    }

    pub fn ld_r_addr(&mut self, r: &Register, addr: u16, memory: &mut Memory) {
        self.registers.set_r(r, memory.read(addr));
    }
    pub fn ld_addr_r(&mut self, addr: u16, r: &Register, memory: &mut Memory) {
        let reg = self.registers.get_r(r);
        memory.write(addr, reg);
    }

    pub fn ld_r_r(&mut self, r1: &Register, r2: &Register) {
        let r2_val = self.registers.get_r(r2);
        self.registers.set_r(r1, r2_val);
    }

    pub fn halt(&mut self, memory: &mut Memory) {
        let pending = memory.read(0xFFFF) & memory.read(0xFF0F) != 0;
        if self.ime || !pending {
            self.halted = true;
        } else {
            self.halt_bug = true;
        }
    }

    pub fn add_a_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let (res, carry) = self.registers.a.overflowing_add(n);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (n & 0xF) + (old_a & 0xF) > 0xF;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn add_sp_d(&mut self, d: i8) {
        let old_sp_low = self.registers.sp as u8;
        let d_unsigned = d.cast_unsigned();

        self.registers.sp = self.registers.sp.wrapping_add_signed(i16::from(d));

        let (_, carry) = old_sp_low.overflowing_add(d_unsigned);

        self.registers.f.zero = false;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_sp_low & 0xF) + (d_unsigned & 0xF) > 0xF;
    }

    pub fn adc_a_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let carry_in = u8::from(self.registers.f.carry);
        let (no_carry_res, carry1) = self.registers.a.overflowing_add(n);
        let (res, carry2) = no_carry_res.overflowing_add(carry_in);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = carry1 || carry2;
        self.registers.f.half_carry = (n & 0xF) + (old_a & 0xF) + carry_in > 0xF;
    }
    pub fn sub_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let (res, carry) = self.registers.a.overflowing_sub(n);
        self.registers.a = res;
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (old_a & 0xF) < (n & 0xF);
    }
    pub fn sbc_n(&mut self, n: u8) {
        let old_a = self.registers.a;
        let carry_in = u8::from(self.registers.f.carry);
        let (res1, carry1) = self.registers.a.overflowing_sub(n);
        let (res2, carry2) = res1.overflowing_sub(carry_in);
        self.registers.a = res2;
        self.registers.f.zero = res2 == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry1 || carry2;
        self.registers.f.half_carry = (old_a & 0xF) < (n & 0xF) + carry_in;
    }
    pub fn and_n(&mut self, n: u8) {
        self.registers.a &= n;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = true;
    }
    pub fn xor_n(&mut self, n: u8) {
        self.registers.a ^= n;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
    }
    pub fn or_n(&mut self, n: u8) {
        self.registers.a |= n;
        self.registers.f.zero = self.registers.a == 0;
        self.registers.f.subtract = false;
        self.registers.f.carry = false;
        self.registers.f.half_carry = false;
    }
    pub fn cp_n(&mut self, n: u8) {
        let (res, carry) = self.registers.a.overflowing_sub(n);
        self.registers.f.zero = res == 0;
        self.registers.f.subtract = true;
        self.registers.f.carry = carry;
        self.registers.f.half_carry = (self.registers.a & 0xF) < (n & 0xF);
    }

    pub fn rotate_left(&mut self, r: &Register, through_carry: bool, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        let bit7 = val >> 7;
        let carry_in = if through_carry {
            u8::from(self.registers.f.carry)
        } else {
            bit7
        };

        let res = (val << 1) | carry_in;
        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
        self.registers.f.carry = bit7 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }
    pub fn rotate_left_a(&mut self, through_carry: bool) {
        let val = self.registers.a;
        let bit7 = val >> 7;
        let carry_in = if through_carry {
            u8::from(self.registers.f.carry)
        } else {
            bit7
        };

        let res = (val << 1) | carry_in;
        self.registers.set_r(&Register::A, res);
        self.registers.f.carry = bit7 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = false;
    }

    pub fn rotate_right(&mut self, r: &Register, through_carry: bool, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        let bit0 = val & 1;
        let carry_in = if through_carry {
            u8::from(self.registers.f.carry)
        } else {
            bit0
        };
        let res = (carry_in << 7) | val >> 1;
        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }
    pub fn rotate_right_a(&mut self, through_carry: bool) {
        let val = self.registers.a;
        let bit0 = val & 1;
        let carry_in = if through_carry {
            u8::from(self.registers.f.carry)
        } else {
            bit0
        };
        let res = (carry_in << 7) | val >> 1;
        self.registers.set_r(&Register::A, res);
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = false;
    }

    pub fn inc_r(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        let res = val.wrapping_add(1);
        self.registers.f.subtract = false;
        self.registers.f.zero = res == 0;
        self.registers.f.half_carry = (val & 0xF) + (1 & 0xF) > 0xF;

        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
    }

    pub fn dec_r(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        let res = val.wrapping_sub(1);
        self.registers.f.subtract = true;
        self.registers.f.zero = res == 0;
        self.registers.f.half_carry = val.trailing_zeros() >= 4;

        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
    }
    pub fn sla(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        let bit7 = (val >> 7) & 1;
        let res = val << 1;
        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
        self.registers.f.carry = bit7 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }

    pub fn sra(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        let bit0 = val & 1;
        let bit7 = val & 0x80;
        let res = val >> 1 | bit7;
        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }

    pub fn srl(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };
        let bit0 = val & 1;
        let res = val >> 1;
        match r {
            Register::HLDirect => memory.write(self.registers.hl(), res),
            _ => self.registers.set_r(r, res),
        }
        self.registers.f.carry = bit0 == 1;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = res == 0;
    }
    pub fn swap(&mut self, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        let low = val & 0x0F;
        let high = val >> 4;
        let new_val = ((low) << 4) | high;

        match r {
            Register::HLDirect => memory.write(self.registers.hl(), new_val),
            _ => self.registers.set_r(r, new_val),
        }

        self.registers.f.carry = false;
        self.registers.f.subtract = false;
        self.registers.f.half_carry = false;
        self.registers.f.zero = new_val == 0;
    }

    pub fn res(&mut self, y: u8, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        let mask = !(1u8 << y);

        let new_val = val & mask;

        match r {
            Register::HLDirect => memory.write(self.registers.hl(), new_val),
            _ => self.registers.set_r(r, new_val),
        }
    }
    pub fn set(&mut self, y: u8, r: &Register, memory: &mut Memory) {
        let val = match r {
            Register::HLDirect => memory.read(self.registers.hl()),
            _ => self.registers.get_r(r),
        };

        let mask = 1u8 << y;

        let new_val = val | mask;

        match r {
            Register::HLDirect => memory.write(self.registers.hl(), new_val),
            _ => self.registers.set_r(r, new_val),
        }
    }
}
