use crate::encoder::encode_instruction;
use crate::types::{AssemblerError, AssemblerErrorKind, Section};
use riscv_core::{Executable, BASE_ADDRESS};
use std::collections::HashMap;

fn parse_data_value(value_str: &str) -> Result<i64, std::num::ParseIntError> {
    let s = value_str.trim_end_matches(',');
    if s.starts_with("0x") {
        u64::from_str_radix(&s[2..], 16).map(|val| val as i64)
    } else {
        s.parse::<i64>()
    }
}

fn align_up(value: u64, alignment: u64) -> u64 {
    (value + alignment - 1) & !(alignment - 1)
}

pub fn parse_program(program: &str) -> Result<Executable, AssemblerError> {
    let mut text_labels = HashMap::new();
    let mut data_labels = HashMap::new();
    let mut bss_labels = HashMap::new();

    let mut data_segment = Vec::new();
    let mut text_segment_size: u64 = 0;
    let mut bss_segment_size: u64 = 0;

    let mut current_section = Section::Text;
    let mut global_label_name: Option<String> = None;

    for (i, line) in program.lines().enumerate() {
        let line_number = i + 1;
        let clean_line = line.split('#').next().unwrap_or("").trim();
        if clean_line.is_empty() {
            continue;
        }

        let (label, rest) = if let Some((l, r)) = clean_line.split_once(':') {
            (Some(l.trim()), r.trim())
        } else {
            (None, clean_line)
        };

        if rest.is_empty() {
            if let Some(l_name) = label {
                match current_section {
                    Section::Text => {
                        text_labels.insert(l_name.to_string(), text_segment_size);
                    }
                    Section::Data => {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    Section::Bss => {
                        bss_labels.insert(l_name.to_string(), bss_segment_size);
                    }
                }
            }
            continue;
        }

        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let mnemonic = tokens[0].to_lowercase();

        if mnemonic.starts_with('.') {
            match mnemonic.as_str() {
                ".global" => {
                    if tokens.len() > 1 {
                        global_label_name = Some(tokens[1].to_string());
                    }
                }
                ".section" => {
                    if tokens.len() > 1 {
                        let new_section_name = tokens[1];
                        if new_section_name != ".text" && current_section == Section::Text {
                            text_segment_size = align_up(text_segment_size, 8);
                        }
                        if new_section_name != ".data" && current_section == Section::Data {
                            while data_segment.len() % 8 != 0 {
                                data_segment.push(0);
                            }
                        }

                        match new_section_name {
                            ".text" => current_section = Section::Text,
                            ".data" => current_section = Section::Data,
                            ".bss" => current_section = Section::Bss,
                            _ => {}
                        }
                    }
                }
                ".text" => {
                    current_section = Section::Text;
                }
                ".data" => {
                    if current_section == Section::Text {
                        text_segment_size = align_up(text_segment_size, 8);
                    }
                    current_section = Section::Data;
                }
                ".align" => {
                    if tokens.len() < 2 {
                        return Err(AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::ParseError("Invalid .align".to_string()),
                        });
                    }
                    let alignment = parse_data_value(tokens[1]).map_err(|_| AssemblerError {
                        line: line_number,
                        kind: AssemblerErrorKind::InvalidImmediateValue(tokens[1].to_string()),
                    })?;
                    if alignment < 0 {
                        return Err(AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::InvalidImmediateValue(
                                "Alignment must be non-negative".to_string(),
                            ),
                        });
                    }
                    let align_bytes = 1u64 << alignment;

                    match current_section {
                        Section::Text => {
                            text_segment_size = align_up(text_segment_size, align_bytes);
                            if let Some(l_name) = label {
                                text_labels.insert(l_name.to_string(), text_segment_size);
                            }
                        }
                        Section::Data => {
                            while (data_segment.len() as u64 % align_bytes) != 0 {
                                data_segment.push(0);
                            }
                            if let Some(l_name) = label {
                                data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                            }
                        }
                        Section::Bss => {
                            bss_segment_size = align_up(bss_segment_size, align_bytes);
                            if let Some(l_name) = label {
                                bss_labels.insert(l_name.to_string(), bss_segment_size);
                            }
                        }
                    }
                }
                ".byte" => {
                    if let Some(l_name) = label {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    for op in &tokens[1..] {
                        let value = parse_data_value(op).map_err(|_| AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::InvalidImmediateValue(op.to_string()),
                        })?;
                        data_segment.push(value as u8);
                    }
                }
                ".half" => {
                    if let Some(l_name) = label {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    for op in &tokens[1..] {
                        let value = parse_data_value(op).map_err(|_| AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::InvalidImmediateValue(op.to_string()),
                        })?;
                        data_segment.extend_from_slice(&(value as u16).to_le_bytes());
                    }
                }
                ".word" | ".dword" => {
                    if let Some(l_name) = label {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    for op in &tokens[1..] {
                        let value = parse_data_value(op).map_err(|_| AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::InvalidImmediateValue(op.to_string()),
                        })?;
                        data_segment.extend_from_slice(&(value as u32).to_le_bytes());
                    }
                }
                ".quad" => {
                    if let Some(l_name) = label {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    for op in &tokens[1..] {
                        let value = parse_data_value(op).map_err(|_| AssemblerError {
                            line: line_number,
                            kind: AssemblerErrorKind::InvalidImmediateValue(op.to_string()),
                        })?;
                        data_segment.extend_from_slice(&(value as u64).to_le_bytes());
                    }
                }
                ".asciz" => {
                    if let Some(l_name) = label {
                        data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                    }
                    let s = tokens[1..].join(" ").trim_matches('"').to_string();
                    data_segment.extend_from_slice(s.as_bytes());
                    data_segment.push(0);
                }
                ".zero" | ".space" => {
                    let count = parse_data_value(tokens[1]).map_err(|_| AssemblerError {
                        line: line_number,
                        kind: AssemblerErrorKind::InvalidImmediateValue(tokens[1].to_string()),
                    })?;
                    if current_section == Section::Bss {
                        if let Some(l_name) = label {
                            bss_labels.insert(l_name.to_string(), bss_segment_size);
                        }
                        bss_segment_size += count as u64;
                    } else {
                        if let Some(l_name) = label {
                            data_labels.insert(l_name.to_string(), data_segment.len() as u64);
                        }
                        data_segment.extend(vec![0; count as usize]);
                    }
                }
                _ => {
                    return Err(AssemblerError {
                        line: line_number,
                        kind: AssemblerErrorKind::UnknownDirective(mnemonic.to_string()),
                    })
                }
            }
        } else {
            if current_section == Section::Text {
                if let Some(l_name) = label {
                    text_labels.insert(l_name.to_string(), text_segment_size);
                }
                if mnemonic == "la" {
                    text_segment_size += 8;
                } else {
                    text_segment_size += 4;
                }
            }
        }
    }

    match current_section {
        Section::Text => text_segment_size = align_up(text_segment_size, 8),
        Section::Data => {
            while data_segment.len() % 8 != 0 {
                data_segment.push(0);
            }
        }
        Section::Bss => bss_segment_size = align_up(bss_segment_size, 8),
    }

    let mut text_segment = Vec::with_capacity(text_segment_size as usize);
    let mut current_address: u64 = 0;
    let final_data_size = data_segment.len() as u64;

    for (i, line) in program.lines().enumerate() {
        let line_number = i + 1;
        let mut clean_line = line.split('#').next().unwrap_or("").trim();
        if let Some((_, rest)) = clean_line.split_once(':') {
            clean_line = rest.trim();
        }
        if clean_line.is_empty() {
            continue;
        }
        let tokens: Vec<&str> = clean_line.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }

        let mnemonic = tokens[0].to_lowercase();
        if mnemonic.starts_with('.') {
            if mnemonic == ".align" {
                if tokens.len() < 2 {
                    return Err(AssemblerError {
                        line: line_number,
                        kind: AssemblerErrorKind::ParseError("Invalid .align".to_string()),
                    });
                }
                let alignment = parse_data_value(tokens[1]).map_err(|_| AssemblerError {
                    line: line_number,
                    kind: AssemblerErrorKind::InvalidImmediateValue(tokens[1].to_string()),
                })?;
                if alignment >= 0 {
                    let align_bytes = 1u64 << alignment;
                    if align_bytes > 0 {
                        while (current_address % align_bytes) != 0 {
                            text_segment.extend_from_slice(&0x00000013_u32.to_le_bytes());
                            current_address += 4;
                        }
                    }
                }
            }
            continue;
        }

        let instruction = tokens[0].to_lowercase();
        let operands = &tokens[1..];
        let encoded_insts = encode_instruction(
            &instruction,
            operands,
            current_address,
            &text_labels,
            &data_labels,
            &bss_labels,
            text_segment_size,
            final_data_size,
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

    while (text_segment.len() as u64) < text_segment_size {
        text_segment.extend_from_slice(&0x00000013_u32.to_le_bytes());
    }

    let entry_point_address = if let Some(label_name) = global_label_name {
        let offset = text_labels.get(&label_name).ok_or_else(|| AssemblerError {
            line: 0,
            kind: AssemblerErrorKind::UndefinedLabel(label_name),
        })?;
        BASE_ADDRESS + offset
    } else {
        BASE_ADDRESS
    };

    Ok(Executable {
        text: text_segment,
        data: data_segment,
        bss_size: bss_segment_size,
        entry_point: entry_point_address,
    })
}
