use assembler;
use riscv_core::abi;
use vm;

#[test]
fn run_comprehensive_test_suite() {
    let program_asm = include_str!("comprehensive_suite.s");

    let executable = assembler::parse_program(program_asm)
        .unwrap_or_else(|e| panic!("Failed to assemble the comprehensive test program: {}", e));

    let mut vm = vm::VM::new();
    vm.load_executable(&executable)
        .expect("Failed to load executable into the VM.");

    vm.run()
        .expect("VM execution failed with an unexpected trap.");

    // --- Verification Step ---
    let results_base_addr = vm.registers[abi::GP as usize];
    assert_ne!(
        results_base_addr, 0,
        "GP was not set, cannot find results array."
    );

    // This list MUST EXACTLY MATCH the order of the .zero directives in the .bss section.
    let test_names = [
        // I-Type
        "addi_pos",
        "addi_neg",
        "addi_zero",
        "addi_overflow",
        "addiw_pos",
        "addiw_neg",
        "addiw_overflow_pos",
        "addiw_overflow_neg",
        "addiw_sign_extend",
        "slti_less",
        "slti_equal",
        "slti_greater",
        "sltiu_less",
        "sltiu_equal",
        "sltiu_greater",
        "sltiu_neg_one_vs_one",
        "xori_simple",
        "ori_simple",
        "andi_simple",
        "slli_simple",
        "slli_by_0",
        "slli_by_63",
        "slliw_simple",
        "slliw_by_31",
        "srli_simple",
        "srli_by_0",
        "srli_by_63",
        "srliw_simple",
        "srliw_by_31",
        "srai_positive",
        "srai_negative",
        "srai_by_0",
        "srai_by_63",
        "sraiw_positive",
        "sraiw_negative",
        "sraiw_by_31",
        // Loads
        "ld_simple",
        "lw_positive",
        "lw_negative",
        "lh_positive",
        "lh_negative",
        "lb_positive",
        "lb_negative",
        "lwu_simple",
        "lhu_simple",
        "lbu_simple",
        // Stores
        "sd_simple",
        "sw_simple",
        "sh_simple",
        "sb_simple",
        // R-Type
        "add_simple",
        "add_overflow",
        "addw_simple",
        "addw_sign_extend",
        "sub_simple",
        "sub_underflow",
        "subw_simple",
        "subw_sign_extend",
        "sll_simple",
        "sll_by_64",
        "sllw_simple",
        "sllw_by_32",
        "slt_less",
        "slt_equal",
        "slt_greater",
        "sltu_less",
        "sltu_equal",
        "sltu_greater",
        "sltu_neg_one_vs_one",
        "xor_simple",
        "or_simple",
        "and_simple",
        "srl_simple",
        "srl_by_64",
        "srlw_simple",
        "srlw_by_32",
        "sra_positive",
        "sra_negative",
        "sra_by_64",
        "sraw_positive",
        "sraw_negative",
        "sraw_by_32",
        // M-Extension
        "mul_pos_pos",
        "mul_pos_neg",
        "mul_neg_neg",
        "mul_by_zero",
        "mulw_simple",
        "mulw_overflow",
        "mulh_neg_neg",
        "mulhsu_neg_pos",
        "mulhu_large",
        "div_simple",
        "div_by_zero",
        "div_min_by_neg_one",
        "divu_simple",
        "divu_by_zero",
        "divw_simple",
        "divw_by_zero",
        "divw_min_by_neg_one",
        "divuw_simple",
        "divuw_by_zero",
        "rem_simple",
        "rem_by_zero",
        "rem_min_by_neg_one",
        "remu_simple",
        "remu_by_zero",
        "remw_simple",
        "remw_by_zero",
        "remw_min_by_neg_one",
        "remuw_simple",
        "remuw_by_zero",
        // Branch
        "beq_taken",
        "beq_not_taken",
        "bne_taken",
        "bne_not_taken",
        "blt_taken",
        "blt_not_taken",
        "bge_taken",
        "bge_not_taken",
        "bltu_taken",
        "bltu_not_taken",
        "bgeu_taken",
        "bgeu_not_taken",
        // Jumps
        "jal_simple",
        "jalr_simple",
        // CSR
        "csrrw_simple",
        "csrrs_simple",
        "csrrc_simple",
        "csrrwi_simple",
        "csrrsi_simple",
        "csrrci_simple",
    ];

    let mut all_passed = true;
    for (i, &test_name) in test_names.iter().enumerate() {
        let result_addr = results_base_addr + (i as u64 * 8);
        let paddr = vm.translate_addr(result_addr).unwrap();
        let result_bytes: [u8; 8] = vm.memory[paddr..paddr + 8].try_into().unwrap();
        let result_val = u64::from_le_bytes(result_bytes);

        let expected_val: u64 = 1;

        if result_val != expected_val {
            println!(
                "[FAIL {:03}] {:<25} | expected: {}, got: {} (addr=0x{:X}, paddr=0x{:X})",
                i, test_name, expected_val, result_val, result_addr, paddr
            );
            all_passed = false;
        }
    }

    assert!(all_passed, "One or more tests failed. See output above.");
}
