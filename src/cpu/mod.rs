use crate::registers::{Register, Registers};
pub mod fde;
pub mod helpers;
pub mod ops;

#[allow(dead_code)]
pub enum AccFlag {
    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,
}

pub enum CBRot {
    Rlc(Register),
    Rrc(Register),
    Rl(Register),
    Rr(Register),
    Sla(Register),
    Sra(Register),
    Swap(Register),
    Srl(Register),
}

#[allow(dead_code)]
pub enum Alu {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

#[allow(dead_code)]
pub struct Cpu {
    pub registers: Registers,
    pub pc: u16,
    pub halted: bool,
    pub halt_bug: bool,
    pub ime: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            registers: Registers::default(),
            pc: 0x0100,
            halted: false,
            halt_bug: false,
            ime: false,
        }
    }
}
