mod virtualmachine;
use virtualmachine::{OP_ADD, OP_ADDI, OP_HALT, VM};

fn main() {
    let mut vm = VM::new();

    // Our new program, written in RISC-V style assembly
    let program = vec![
        OP_ADDI, 5, 0, 10, // addi x5, 10
        OP_ADDI, 6, 0, 25, // addi x6, 25
        OP_ADD, 7, 5, 6,       // add x7, x5, x6
        OP_HALT, // halt
    ];

    vm.load_program(&program);
    vm.run();

    println!("VM execution finished.");
    println!("Final state of registers:");

    for i in 0..32 {
        println!("x{}: {}", i, vm.registers[i]);
    }

    println!("\nValue in register x7 should be 35: {}", vm.registers[7]);
}
