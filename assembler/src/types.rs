use std::error::Error;
use std::fmt;

pub const BASE_ADDRESS: u64 = 0x80000000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Text,
    Data,
    Bss,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Executable {
    pub text: Vec<u8>,
    pub data: Vec<u8>,
    pub bss_size: u64,
    pub entry_point: u64,
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
    ValueOutOfRange(String),
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
            Self::ValueOutOfRange(msg) => write!(f, "Value out of range: {}", msg),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembler_error_kind_display() {
        assert_eq!(
            AssemblerErrorKind::InvalidRegister("t7".to_string()).to_string(),
            "Invalid register name: 't7'"
        );
        assert_eq!(
            AssemblerErrorKind::InvalidMemoryOperand("8(sp)".to_string()).to_string(),
            "Invalid memory operand format: '8(sp)'"
        );
        assert_eq!(
            AssemblerErrorKind::InvalidImmediateValue("0xFAIL".to_string()).to_string(),
            "Cannot parse immediate value: '0xFAIL'"
        );
        assert_eq!(
            AssemblerErrorKind::ImmediateOutOfRange("4096".to_string()).to_string(),
            "Immediate value out of range: '4096'"
        );
        assert_eq!(
            AssemblerErrorKind::UndefinedLabel("my_label".to_string()).to_string(),
            "Use of undefined label: 'my_label'"
        );
        assert_eq!(
            AssemblerErrorKind::UnknownInstruction("jump".to_string()).to_string(),
            "Unknown instruction: 'jump'"
        );
        assert_eq!(
            AssemblerErrorKind::UnknownDirective(".dataz".to_string()).to_string(),
            "Unknown directive: '.dataz'"
        );
        assert_eq!(
            AssemblerErrorKind::ParseError("Missing operand".to_string()).to_string(),
            "Parse error: Missing operand"
        );
    }

    #[test]
    fn test_assembler_error_display() {
        let error = AssemblerError {
            line: 42,
            kind: AssemblerErrorKind::UnknownInstruction("fly".to_string()),
        };
        assert_eq!(error.to_string(), "Line 42: Unknown instruction: 'fly'");

        let error_two = AssemblerError {
            line: 1,
            kind: AssemblerErrorKind::ParseError("Unexpected token".to_string()),
        };
        assert_eq!(
            error_two.to_string(),
            "Line 1: Parse error: Unexpected token"
        );
    }
}
