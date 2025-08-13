use std::error::Error;
use std::fmt;

pub const BASE_ADDRESS: u64 = 0x80000000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Text,
    Data,
}

#[derive(Debug, Clone)]
pub struct Executable {
    // PRIO 5: # TODO: Add a `.bss` field to the executable to represent the size of the zero-initialized data section.
    pub text: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssemblerErrorKind {
    InvalidRegister(String),
    InvalidMemoryOperand(String),
    InvalidImmediateValue(String),
    ImmediateOutOfRange(String),
    UndefinedLabel(String),
    UnknownInstruction(String),
    UnknownDirective(String),
    ParseError(String),
}

impl fmt::Display for AssemblerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidRegister(reg) => write!(f, "Invalid register name: '{}'", reg),
            Self::InvalidMemoryOperand(op) => write!(f, "Invalid memory operand format: '{}'", op),
            Self::InvalidImmediateValue(val) => {
                write!(f, "Cannot parse immediate value: '{}'", val)
            }
            Self::ImmediateOutOfRange(val) => write!(f, "Immediate value out of range: '{}'", val),
            Self::UndefinedLabel(label) => write!(f, "Use of undefined label: '{}'", label),
            Self::UnknownInstruction(inst) => write!(f, "Unknown instruction: '{}'", inst),
            Self::UnknownDirective(dir) => write!(f, "Unknown directive: '{}'", dir),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssemblerError {
    pub line: usize,
    pub kind: AssemblerErrorKind,
}

impl fmt::Display for AssemblerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Line {}: {}", self.line, self.kind)
    }
}
impl Error for AssemblerError {}
