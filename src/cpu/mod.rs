use crate::{
    memory::Memory,
    registers::{Register, Registers},
};
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
