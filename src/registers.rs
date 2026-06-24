/// Adapted from https://rylev.github.io/DMG-01/public/book/cpu/registers.html

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const SUBTRACT_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct FlagsRegister {
    pub zero: bool,
    pub subtract: bool,
    pub half_carry: bool,
    pub carry: bool,
}

#[allow(dead_code)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    F,
}

#[allow(dead_code)]
pub enum RegisterPair {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> SUBTRACT_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        Self {
            zero,
            subtract,
            half_carry,
            carry,
        }
    }
}

impl From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> Self {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.subtract { 1 } else { 0 }) << SUBTRACT_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

#[allow(dead_code)]
pub struct Registers {
    pub a: u8,
    pub f: FlagsRegister,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0x01,
            f: 0xB0.into(),
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
        }
    }
}

impl Registers {
    pub fn set_bc(&mut self, nn: u16) {
        self.b = (nn >> 8) as u8;
        self.c = (nn & 0xFF) as u8;
    }
    pub fn set_de(&mut self, nn: u16) {
        self.d = (nn >> 8) as u8;
        self.e = (nn & 0xFF) as u8;
    }
    pub fn set_hl(&mut self, nn: u16) {
        self.h = (nn >> 8) as u8;
        self.l = (nn & 0xFF) as u8;
    }
    pub fn set_af(&mut self, nn: u16) {
        self.a = (nn >> 8) as u8;
        self.f = ((nn & 0xFF) as u8).into();
    }
    pub fn af(&self) -> u16 {
        (self.a as u16) << 8 | u8::from(self.f) as u16
    }
    pub fn bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }
    pub fn hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }
    pub fn rotate_left(&mut self, r: Register, through_carry: bool) {
        let val = self.get_r(&r);
        let bit7 = val >> 7;
        let carry_in = if through_carry {
            self.f.carry as u8
        } else {
            bit7
        };

        let res = (val << 1) | carry_in;
        self.set_r(r, res);
        self.f.carry = bit7 == 1;
        self.f.subtract = false;
        self.f.half_carry = false;
        self.f.zero = false;
    }

    pub fn rotate_right(&mut self, r: Register, through_carry: bool) {
        let val = self.get_r(&r);
        let bit0 = val & 1;
        let carry_in = if through_carry {
            self.f.carry as u8
        } else {
            bit0
        };
        let res = (carry_in << 7) | val >> 1;
        self.set_r(r, res);
        self.f.carry = bit0 == 1;
        self.f.subtract = false;
        self.f.half_carry = false;
        self.f.zero = false;
    }

    pub fn daa(&mut self) {
        let mut adj: u8 = 0;
        let mut carry = self.f.carry;

        if self.f.subtract {
            if self.f.half_carry {
                adj += 0x06;
            }
            if self.f.carry {
                adj += 0x60;
            }
            self.a = self.a.wrapping_sub(adj);
        } else {
            if self.f.half_carry || self.a & 0x0F > 0x09 {
                adj += 0x06;
            }
            if self.f.carry || self.a > 0x99 {
                adj += 0x60;
                carry = true;
            }
            self.a = self.a.wrapping_add(adj);
        }

        self.f.zero = self.a == 0;
        self.f.carry = carry;
        self.f.half_carry = false;
    }

    pub fn cpl(&mut self) {
        self.a = !self.a;
        self.f.subtract = true;
        self.f.half_carry = true;
    }
    pub fn scf(&mut self) {
        self.f.carry = true;
        self.f.subtract = false;
        self.f.half_carry = false;
    }

    pub fn ccf(&mut self) {
        self.f.carry = !self.f.carry;
        self.f.subtract = false;
        self.f.half_carry = false;
    }

    pub fn add_hl(&mut self, r2: RegisterPair) {
        let hl = self.hl();
        let r2_val = self.get_rp(&r2);
        let (new_value, did_overflow) = hl.overflowing_add(r2_val);
        self.f.subtract = false;
        self.f.carry = did_overflow;
        self.f.half_carry = (hl & 0xFFF) + (r2_val & 0xFFF) > 0xFFF;
        self.set_hl(new_value);
    }

    pub fn set_rp(&mut self, rp: RegisterPair, nn: u16) {
        match rp {
            RegisterPair::BC => self.set_bc(nn),
            RegisterPair::DE => self.set_de(nn),
            RegisterPair::HL => self.set_hl(nn),
            RegisterPair::AF => self.set_af(nn),
            RegisterPair::SP => self.sp = nn,
        }
    }

    pub fn inc_rp(&mut self, rp: RegisterPair) {
        match rp {
            RegisterPair::BC => self.set_bc(self.bc().wrapping_add(1)),
            RegisterPair::DE => self.set_de(self.de().wrapping_add(1)),
            RegisterPair::HL => self.set_hl(self.hl().wrapping_add(1)),
            RegisterPair::SP => self.sp = self.sp.wrapping_add(1),
            _ => unreachable!(),
        }
    }

    pub fn dec_rp(&mut self, rp: RegisterPair) {
        match rp {
            RegisterPair::BC => self.set_bc(self.bc().wrapping_sub(1)),
            RegisterPair::DE => self.set_de(self.de().wrapping_sub(1)),
            RegisterPair::HL => self.set_hl(self.hl().wrapping_sub(1)),
            RegisterPair::SP => self.sp = self.sp.wrapping_sub(1),
            _ => unreachable!(),
        }
    }

    pub fn get_rp(&self, rp: &RegisterPair) -> u16 {
        match rp {
            RegisterPair::AF => self.af(),
            RegisterPair::BC => self.bc(),
            RegisterPair::DE => self.de(),
            RegisterPair::HL => self.hl(),
            RegisterPair::SP => self.sp,
        }
    }
    pub fn set_r(&mut self, r: Register, val: u8) {
        match r {
            Register::A => self.a = val,
            Register::B => self.b = val,
            Register::C => self.c = val,
            Register::D => self.d = val,
            Register::E => self.e = val,
            Register::H => self.h = val,
            Register::L => self.l = val,
            Register::F => self.f = val.into(),
        }
    }

    pub fn inc_r(&mut self, r: Register) {
        match r {
            Register::A => self.a = self.a.wrapping_add(1),
            Register::B => self.b = self.b.wrapping_add(1),
            Register::C => self.c = self.c.wrapping_add(1),
            Register::D => self.d = self.d.wrapping_add(1),
            Register::E => self.e = self.e.wrapping_add(1),
            Register::H => self.h = self.h.wrapping_add(1),
            Register::L => self.l = self.l.wrapping_add(1),
            _ => unreachable!(),
        }
    }

    pub fn dec_r(&mut self, r: Register) {
        match r {
            Register::A => self.a = self.a.wrapping_sub(1),
            Register::B => self.b = self.b.wrapping_sub(1),
            Register::C => self.c = self.c.wrapping_sub(1),
            Register::D => self.d = self.d.wrapping_sub(1),
            Register::E => self.e = self.e.wrapping_sub(1),
            Register::H => self.h = self.h.wrapping_sub(1),
            Register::L => self.l = self.l.wrapping_sub(1),
            _ => unreachable!(),
        }
    }

    pub fn get_r(&self, r: &Register) -> u8 {
        match r {
            Register::A => self.a,
            Register::B => self.b,
            Register::C => self.c,
            Register::D => self.d,
            Register::E => self.e,
            Register::H => self.h,
            Register::L => self.l,
            Register::F => u8::from(self.f),
        }
    }
}
