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
}
