mod assembler;

use assembler::{parse_command, parse_program, read_file, save_assembly};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let program_args = &args[1..];

    let (input, output) = parse_command(program_args);

    let file = read_file(input.as_str()).expect("Unable to read file.");

    let program = parse_program(file);

    save_assembly(output.as_str(), &program).expect("Unable to save file.");
}
