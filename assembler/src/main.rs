mod assembler;

use assembler::parse;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let program_args = &args[1..];

    let (input, output) = parse_command(program_args);

    println!("{} {}", input, output);
}
