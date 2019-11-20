//!
//! ZRust bytecode library.
//!

pub mod instructions;
pub mod vlq;

pub use crate::instructions::*;

use std::fmt::Debug;
use std::cmp;

#[derive(Debug)]
pub enum InstructionCode {
    NoOperation,

    // Stack
    Pop,
    Push,
    Copy,

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Neg,

    // Boolean
    Not,
    And,
    Or,
    Xor,

    // Comparison
    Lt,
    Le,
    Eq,
    Ne,
    Ge,
    Gt,

    Cast,

    // Flow control
    ConditionalSelect,
    LoopBegin,
    LoopEnd,
    Call,
    Return,

    // Condition utils
    Assert,
    PushCondition,
    PopCondition,

    Exit,
}

pub trait Instruction: Debug {
    fn to_assembly(&self) -> String;
    fn code(&self) -> InstructionCode;
    fn encode(&self) -> Vec<u8>;
    fn inputs_count(&self) -> usize;
    fn outputs_count(&self) -> usize;
}

#[derive(Debug,PartialEq)]
pub enum DecodingError {
    UnexpectedEOF,
    UnknownInstructionCode(u8),
    ConstantTooLong,
}

pub fn decode_all_instructions(bytes: &[u8]) -> Result<Vec<Box<dyn Instruction>>, DecodingError> {
    let mut instructions = Vec::new();

    let mut offset = 0;
    while offset < bytes.len() {
        match decode_instruction(&bytes[offset..]) {
            Ok((instr, len)) => {
                instructions.push(instr);
                offset += len;
            },
            Err(err) => {
                let last = cmp::min(bytes.len(), offset + 10);
                log::warn!("failed to decode bytes {:?} at offset {}", &bytes[offset..last], offset);
                return Err(err);
            }
        };
    }

    Ok(instructions)
}

pub fn decode_instruction(bytes: &[u8]) -> Result<(Box<dyn Instruction>, usize), DecodingError> {
    if bytes.len() < 1 {
        return Err(DecodingError::UnexpectedEOF);
    }

    match bytes[0] {
        x if x == InstructionCode::NoOperation as u8 =>
            NoOperation::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Push as u8 =>
            Push::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Pop as u8 =>
            Pop::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Copy as u8 =>
            Copy::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Add as u8 =>
            Add::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Sub as u8 =>
            Sub::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Mul as u8 =>
            Mul::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Div as u8 =>
            Div::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Rem as u8 =>
            Rem::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Neg as u8 =>
            Neg::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Not as u8 =>
            Not::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::And as u8 =>
            And::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Or as u8 =>
            Or::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Xor as u8 =>
            Xor::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Lt as u8 =>
            Lt::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Le as u8 =>
            Le::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Eq as u8 =>
            Eq::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Ne as u8 =>
            Ne::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Ge as u8 =>
            Ge::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Gt as u8 =>
            Gt::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Cast as u8 =>
            Cast::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::ConditionalSelect as u8 =>
            ConditionalSelect::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::LoopBegin as u8 =>
            LoopBegin::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::LoopEnd as u8 =>
            LoopEnd::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Call as u8 =>
            Call::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Return as u8 =>
            Return::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        x if x == InstructionCode::Assert as u8 =>
            Assert::decode(bytes).map(|(s, len)| -> (Box<dyn Instruction>, usize) {(Box::new(s), len)}),

        code => Err(DecodingError::UnknownInstructionCode(code))
    }
}
