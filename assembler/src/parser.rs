use crate::encoder::encode_instruction;
use crate::types::{AssemblerError, AssemblerErrorKind, Executable, Section};
use std::collections::HashMap;

pub fn parse_program(program: &str) -> Result<Executable, AssemblerError> {
    // PRIO 4: # TODO: Add support for standard assembler directives like `.global` (to define entry points), `.equ` (to define constants), and `.align n` (to ensure proper memory alignment).
    let mut text_labels = HashMap::new();
    let mut data_labels = HashMap::new();

    let mut data_segment = Vec::new();
    let mut text_segment_size: u64 = 0;

    let mut current_section = Section::Text;

    // First pass: Calculate label addresses and data segment contents
    for (i, line) in program.lines().enumerate() {
        let line_number = i + 1;
        let clean_line = line.split('#').next().unwrap_or("").trim();
        if clean_line.is_empty() {
            continue;
        }

        if clean_line == ".text" {
            current_section = Section::Text;
            continue;
        } else if clean_line == ".data" {
            current_section = Section::Data;
            continue;
        }

        if let Some(label) = clean_line.strip_suffix(':') {
            match current_section {
                Section::Text => {
                    text_labels.insert(label.to_string(), text_segment_size);
                }
                Section::Data => {
                    data_labels.insert(label.to_string(), data_segment.len() as u64);
                }
            }
            continue;
        }

        let tokens: Vec<&str> = clean_line.split_whitespace().collect();
        let mnemonic = tokens[0].to_lowercase();

        match current_section {
            Section::Text => {
                // PRIO 2: # FIX: This instruction size logic is not robust. A proper function should determine the size of each instruction, including pseudo-instructions which can expand to multiple real instructions.
                // PRIO 2: # TODO: Create a function `get_instruction_size(mnemonic, operands)` that returns the byte size (e.g., `la` is 8, `jal` is 4) to correctly calculate `text_segment_size`.
                if mnemonic == "la" {
                    text_segment_size += 8;
                } else {
                    text_segment_size += 4;
                }
            }
            Section::Data => {
                // PRIO 3: # TODO: Add support for other standard data directives: `.byte` (8-bit), `.half` (16-bit), `.dword` (32-bit), `.quad` (64-bit), and `.zero n` (to allocate n zero-filled bytes).
                let directive = &mnemonic;
                let operands = &tokens[1..];
                match directive.as_str() {
                    ".word" => {
                        for op in operands {
                            let value_str = op.trim_end_matches(',');
                            let value = value_str.parse::<u32>().map_err(|_| AssemblerError {
                                line: line_number,
                                kind: AssemblerErrorKind::InvalidImmediateValue(
                                    value_str.to_string(),
                                ),
                            })?;
                            data_segment.extend_from_slice(&value.to_le_bytes());
                        }
                    }
                    ".asciz" => {
                        let s = operands.join(" ").trim_matches('"').to_string();
                        data_segment.extend_from_slice(s.as_bytes());
                        data_segment.push(0);
                    }
                    _ => {
                        return Err(AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::UnknownDirective(directive.to_string()),
                        })
                    }
                }
            }
        }
    }

    let mut text_segment = Vec::new();
    let mut current_address: u64 = 0;
    current_section = Section::Text;

    // Second pass: Encode instructions
    for (i, line) in program.lines().enumerate() {
        let line_number = i + 1;
        let clean_line = line.split('#').next().unwrap_or("").trim();

        if clean_line.is_empty() || clean_line.ends_with(':') {
            continue;
        }
        if clean_line == ".text" {
            current_section = Section::Text;
            continue;
        }
        if clean_line == ".data" {
            current_section = Section::Data;
            continue;
        }
        if current_section == Section::Data {
            continue;
        }

        let tokens: Vec<&str> = clean_line.split_whitespace().collect();
        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];

        let encoded_insts = encode_instruction(
            &instruction,
            operands,
            current_address,
            &text_labels,
            &data_labels,
            text_segment_size,
        )
        .map_err(|kind| AssemblerError {
            line: line_number,
            kind,
        })?;

        for inst in encoded_insts {
            text_segment.extend_from_slice(&inst.to_le_bytes());
            current_address += 4;
        }
    }

    Ok(Executable {
        text: text_segment,
        data: data_segment,
    })
}
