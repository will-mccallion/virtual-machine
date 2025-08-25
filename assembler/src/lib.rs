pub mod dissassembler;
pub mod encoder;
pub mod parser;
pub mod types;

pub use dissassembler::disassemble;
pub use parser::parse_program;
pub use types::{AssemblerError, AssemblerErrorKind, Executable};
