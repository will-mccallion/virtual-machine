pub mod encoder;
pub mod parser;
pub mod types;

// Re-export the primary functions and types for easy library access.
pub use parser::parse_program;
pub use types::{AssemblerError, AssemblerErrorKind, Executable};
