use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let bios_in_path = "../bootloader/src/bios.s";
    let kernel_in_path = "../kernel/src/kernel.s";

    let bios_out_path = Path::new(&out_dir).join("bios.bin");
    let kernel_out_path = Path::new(&out_dir).join("kernel.bin");

    println!("cargo:rerun-if-changed={}", bios_in_path);
    println!("cargo:rerun-if-changed={}", kernel_in_path);
    println!("cargo:rerun-if-changed=../assembler/src");

    println!("--- Assembling BIOS using `assembler` library ---");
    let bios_source = fs::read_to_string(bios_in_path)
        .unwrap_or_else(|e| panic!("Failed to read BIOS source file '{}': {}", bios_in_path, e));

    let bios_executable = assembler::parse_program(&bios_source)
        .expect("Failed to parse BIOS assembly. Check for errors in your assembly code.");

    let mut bios_bytes = Vec::new();
    bios_bytes.extend(&bios_executable.text);
    bios_bytes.extend(&bios_executable.data);

    fs::write(&bios_out_path, bios_bytes).expect("Failed to write bios.bin to OUT_DIR");

    println!("--- Assembling Kernel using `assembler` library ---");
    let kernel_source = fs::read_to_string(kernel_in_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read kernel source file '{}': {}",
            kernel_in_path, e
        )
    });

    let kernel_executable = assembler::parse_program(&kernel_source)
        .expect("Failed to parse kernel assembly. Check for errors in your assembly code.");

    let mut kernel_bytes = Vec::new();
    kernel_bytes.extend(&kernel_executable.text);
    kernel_bytes.extend(&kernel_executable.data);

    fs::write(&kernel_out_path, kernel_bytes).expect("Failed to write kernel.bin to OUT_DIR");

    println!("--- Build script finished successfully ---");
}
