use crate::{
    cpu::{AccFlag, Alu, CBRot},
    registers::{Condition, Register, RegisterPair},
};

pub fn rp(p: u8) -> RegisterPair {
    match p {
        0 => RegisterPair::BC,
        1 => RegisterPair::DE,
        2 => RegisterPair::HL,
        3 => RegisterPair::SP,
        _ => unreachable!(),
    }
}

pub fn rp2(p: u8) -> RegisterPair {
    match p {
        0 => RegisterPair::BC,
        1 => RegisterPair::DE,
        2 => RegisterPair::HL,
        3 => RegisterPair::AF,
        _ => unreachable!(),
    }
}

pub fn r(y: u8) -> Register {
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

pub fn cc(y: u8) -> Condition {
    match y {
        4 => Condition::NZ,
        5 => Condition::Z,
        6 => Condition::NC,
        7 => Condition::C,
        _ => unreachable!(),
    }
}

pub fn af(y: u8) -> AccFlag {
    match y {
        0 => AccFlag::Rlca,
        1 => AccFlag::Rrca,
        2 => AccFlag::Rla,
        3 => AccFlag::Rra,
        4 => AccFlag::Daa,
        5 => AccFlag::Cpl,
        6 => AccFlag::Scf,
        7 => AccFlag::Ccf,
        _ => unreachable!(),
    }
}

pub fn alu(y: u8) -> Alu {
    match y {
        0 => Alu::Add,
        1 => Alu::Adc,
        2 => Alu::Sub,
        3 => Alu::Sbc,
        4 => Alu::And,
        5 => Alu::Xor,
        6 => Alu::Or,
        7 => Alu::Cp,
        _ => unreachable!(),
    }
}

pub fn rot(y: u8, z: u8) -> CBRot {
    match y {
        0 => CBRot::Rlc(r(z)),
        1 => CBRot::Rrc(r(z)),
        2 => CBRot::Rl(r(z)),
        3 => CBRot::Rr(r(z)),
        4 => CBRot::Sla(r(z)),
        5 => CBRot::Sra(r(z)),
        6 => CBRot::Swap(r(z)),
        7 => CBRot::Srl(r(z)),
        _ => unreachable!(),
    }
}
