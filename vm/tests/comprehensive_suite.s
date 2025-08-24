# =============================================================================
# comprehensive_suite.s
#
# An extremely thorough test suite for the RV64IM instruction set.
#
# REFACTORED: All tests are now independent. A failure in one test will
# not prevent subsequent tests from running. This provides a complete
# report of all successes and failures in a single run.
# =============================================================================

.global _start

# =============================================================================
# DATA SECTION
# =============================================================================
.section .data
    .align 3
    val1:               .quad 20480
    val2:               .quad 0xFFFFFF
    val3:               .quad 0x0000000080000000
    val4:               .quad 0x0FFFFFFF
    half_word:          .quad 65535
    fivek:              .quad 5000
    neg_5k:             .quad -5000
    neg_30k:            .quad -30000
    fiftyk:             .quad 50000
    mul_neg:            .quad -1794967296
    huge:               .quad 0x4000000000000000
    val_zero:           .quad 0
    val_one:            .quad 1
    val_neg_one:        .quad -1
    val_pos_10:         .quad 10
    val_pos_50:         .quad 50
    val_pos_100:        .quad 100
    val_neg_50:         .quad -50
    val_neg_100:        .quad -100
    val_i64_max:        .quad 0x7FFFFFFFFFFFFFFF
    val_i64_min:        .quad 0x8000000000000000

    .align 2
    val_i32_max:        .word 0x7FFFFFFF
    val_i32_min:        .word 0x80000000

    .align 3
    val_u64_large:      .quad 0xFFFFFFFFFFFFFFF0

    .align 2
    val_u32_large:      .word 0xFFFFFFF0

    .align 3
    val_bitmask_a:      .quad 0xAAAAAAAAAAAAAAAA
    val_bitmask_5:      .quad 0x5555555555555555
    val_store_pattern:  .quad 0xCAFEF00DBAADF00D

# BSS SECTION (THE REPORT CARD)
# =============================================================================
.section .bss
results_start:
    res_addi_pos: .zero 8
    res_addi_neg: .zero 8
    res_addi_zero: .zero 8
    res_addi_overflow: .zero 8
    res_addiw_pos: .zero 8
    res_addiw_neg: .zero 8
    res_addiw_overflow_pos: .zero 8
    res_addiw_overflow_neg: .zero 8
    res_addiw_sign_extend: .zero 8
    res_slti_less: .zero 8
    res_slti_equal: .zero 8
    res_slti_greater: .zero 8
    res_sltiu_less: .zero 8
    res_sltiu_equal: .zero 8
    res_sltiu_greater: .zero 8
    res_sltiu_neg_one_vs_one: .zero 8
    res_xori_simple: .zero 8
    res_ori_simple: .zero 8
    res_andi_simple: .zero 8
    res_slli_simple: .zero 8
    res_slli_by_0: .zero 8
    res_slli_by_63: .zero 8
    res_slliw_simple: .zero 8
    res_slliw_by_31: .zero 8
    res_srli_simple: .zero 8
    res_srli_by_0: .zero 8
    res_srli_by_63: .zero 8
    res_srliw_simple: .zero 8
    res_srliw_by_31: .zero 8
    res_srai_positive: .zero 8
    res_srai_negative: .zero 8
    res_srai_by_0: .zero 8
    res_srai_by_63: .zero 8
    res_sraiw_positive: .zero 8
    res_sraiw_negative: .zero 8
    res_sraiw_by_31: .zero 8
    res_ld_simple: .zero 8
    res_lw_positive: .zero 8
    res_lw_negative: .zero 8
    res_lh_positive: .zero 8
    res_lh_negative: .zero 8
    res_lb_positive: .zero 8
    res_lb_negative: .zero 8
    res_lwu_simple: .zero 8
    res_lhu_simple: .zero 8
    res_lbu_simple: .zero 8
    res_sd_simple: .zero 8
    res_sw_simple: .zero 8
    res_sh_simple: .zero 8
    res_sb_simple: .zero 8
    res_add_simple: .zero 8
    res_add_overflow: .zero 8
    res_addw_simple: .zero 8
    res_addw_sign_extend: .zero 8
    res_sub_simple: .zero 8
    res_sub_underflow: .zero 8
    res_subw_simple: .zero 8
    res_subw_sign_extend: .zero 8
    res_sll_simple: .zero 8
    res_sll_by_64: .zero 8
    res_sllw_simple: .zero 8
    res_sllw_by_32: .zero 8
    res_slt_less: .zero 8
    res_slt_equal: .zero 8
    res_slt_greater: .zero 8
    res_sltu_less: .zero 8
    res_sltu_equal: .zero 8
    res_sltu_greater: .zero 8
    res_sltu_neg_one_vs_one: .zero 8
    res_xor_simple: .zero 8
    res_or_simple: .zero 8
    res_and_simple: .zero 8
    res_srl_simple: .zero 8
    res_srl_by_64: .zero 8
    res_srlw_simple: .zero 8
    res_srlw_by_32: .zero 8
    res_sra_positive: .zero 8
    res_sra_negative: .zero 8
    res_sra_by_64: .zero 8
    res_sraw_positive: .zero 8
    res_sraw_negative: .zero 8
    res_sraw_by_32: .zero 8
    res_mul_pos_pos: .zero 8
    res_mul_pos_neg: .zero 8
    res_mul_neg_neg: .zero 8
    res_mul_by_zero: .zero 8
    res_mulw_simple: .zero 8
    res_mulw_overflow: .zero 8
    res_mulh_neg_neg: .zero 8
    res_mulhsu_neg_pos: .zero 8
    res_mulhu_large: .zero 8
    res_div_simple: .zero 8
    res_div_by_zero: .zero 8
    res_div_min_by_neg_one: .zero 8
    res_divu_simple: .zero 8
    res_divu_by_zero: .zero 8
    res_divw_simple: .zero 8
    res_divw_by_zero: .zero 8
    res_divw_min_by_neg_one: .zero 8
    res_divuw_simple: .zero 8
    res_divuw_by_zero: .zero 8
    res_rem_simple: .zero 8
    res_rem_by_zero: .zero 8
    res_rem_min_by_neg_one: .zero 8
    res_remu_simple: .zero 8
    res_remu_by_zero: .zero 8
    res_remw_simple: .zero 8
    res_remw_by_zero: .zero 8
    res_remw_min_by_neg_one: .zero 8
    res_remuw_simple: .zero 8
    res_remuw_by_zero: .zero 8
    res_beq_taken: .zero 8
    res_beq_not_taken: .zero 8
    res_bne_taken: .zero 8
    res_bne_not_taken: .zero 8
    res_blt_taken: .zero 8
    res_blt_not_taken: .zero 8
    res_bge_taken: .zero 8
    res_bge_not_taken: .zero 8
    res_bltu_taken: .zero 8
    res_bltu_not_taken: .zero 8
    res_bgeu_taken: .zero 8
    res_bgeu_not_taken: .zero 8
    res_jal_simple: .zero 8
    res_jalr_simple: .zero 8
    res_csrrw_simple: .zero 8
    res_csrrs_simple: .zero 8
    res_csrrc_simple: .zero 8
    res_csrrwi_simple: .zero 8
    res_csrrsi_simple: .zero 8
    res_csrrci_simple: .zero 8
results_end:

# =============================================================================
# TEXT SECTION
# =============================================================================
.section .text
_start:
    la gp, results_start
    addi t5, zero, 1

    # --- ADDI ---
    addi t0, zero, 100
    addi t0, t0, 50
    addi t1, zero, 150
    bne t0, t1, next1
    sd t5, 0(gp)
next1:
    addi t0, zero, -100
    addi t0, t0, -50
    addi t1, zero, -150
    bne t0, t1, next2
    sd t5, 8(gp)
next2:
    addi t0, zero, 100
    addi t0, t0, 0
    addi t1, zero, 100
    bne t0, t1, next3
    sd t5, 16(gp)
next3:
    la t0, val_i64_max
    ld t0, 0(t0)
    addi t0, t0, 1
    la t1, val_i64_min
    ld t1, 0(t1)
    bne t0, t1, next4
    sd t5, 24(gp)
next4:

    # --- ADDIW ---
    addiw t0, zero, 100
    addiw t0, t0, 50
    addiw t1, zero, 150
    bne t0, t1, next5
    sd t5, 32(gp)
next5:
    addiw t0, zero, -100
    addiw t0, t0, -50
    addiw t1, zero, -150
    bne t0, t1, next6
    sd t5, 40(gp)
next6:
    la t0, val_i32_max
    lw t0, 0(t0)
    addiw t0, t0, 1
    la t1, val_i32_min
    lw t1, 0(t1)
    bne t0, t1, next7
    sd t5, 48(gp)
next7:
    la t0, val_i32_min
    lw t0, 0(t0)
    addiw t0, t0, -1
    la t1, val_i32_max
    lw t1, 0(t1)
    bne t0, t1, next8
    sd t5, 56(gp)
next8:
    addiw t0, zero, -1
    la t6, val_neg_one
    ld t1, 0(t6)
    bne t0, t1, next9
    sd t5, 64(gp)
next9:

    # --- SLTI / SLTIU ---
    slti t0, zero, 1
    bne t0, t5, next10
    sd t5, 72(gp)
next10:
    slti t0, zero, 0
    bne t0, zero, next11
    sd t5, 80(gp)
next11:
    addi t0, zero, 1
    slti t0, t0, 1
    bne t0, zero, next12
    sd t5, 88(gp)
next12:
    sltiu t0, zero, 1
    bne t0, t5, next13
    sd t5, 96(gp)
next13:
    sltiu t0, zero, 0
    bne t0, zero, next14
    sd t5, 104(gp)
next14:
    addi t0, zero, 1
    sltiu t0, t0, 1
    bne t0, zero, next15
    sd t5, 112(gp)
next15:
    addi t0, zero, -1
    sltiu t0, t0, 1
    bne t0, zero, next16
    sd t5, 120(gp)
next16:

    # --- XORI / ORI / ANDI ---
    addi t0, zero, 0xAA
    xori t0, t0, 0xFF
    addi t1, zero, 0x55
    bne t0, t1, next17
    sd t5, 128(gp)
next17:
    addi t0, zero, 0xAA
    ori t0, t0, 0x55
    addi t1, zero, 0xFF
    bne t0, t1, next18
    sd t5, 136(gp)
next18:
    addi t0, zero, 0xAF
    andi t0, t0, 0xF5
    addi t1, zero, 0xA5
    bne t0, t1, next19
    sd t5, 144(gp)
next19:

    # --- SLLI / SLLIW ---
    addi t0, zero, 20
    slli t0, t0, 10
    la t1, val1
    ld t1, 0(t1)
    bne t0, t1, next20
    sd t5, 152(gp)
next20:
    addi t0, zero, 20
    slli t0, t0, 0
    addi t1, zero, 20
    bne t0, t1, next21
    sd t5, 160(gp)
next21:
    addi t0, zero, 1
    slli t0, t0, 63
    la t1, val_i64_min
    ld t1, 0(t1)
    bne t0, t1, next22
    sd t5, 168(gp)
next22:
    addiw t0, zero, 1
    slliw t0, t0, 10
    addiw t1, zero, 1024
    bne t0, t1, next23
    sd t5, 176(gp)
next23:
    addiw t0, zero, 1
    slliw t0, t0, 31
    la t1, val_i32_min
    lw t1, 0(t1)
    bne t0, t1, next24
    sd t5, 184(gp)
next24:

    # --- SRLI / SRLIW ---
    la t0, val_u64_large
    ld t0, 0(t0)
    srli t0, t0, 8
    addi t1, zero, -1
    srli t1, t1, 8
    bne t0, t1, next25
    sd t5, 192(gp)
next25:
    addi t0, zero, 123
    srli t0, t0, 0
    addi t1, zero, 123
    bne t0, t1, next26
    sd t5, 200(gp)
next26:
    la t0, val_i64_min
    ld t0, 0(t0)
    srli t0, t0, 63
    addi t1, zero, 1
    bne t0, t1, next27
    sd t5, 208(gp)
next27:
    la t0, val_u32_large
    lw t0, 0(t0)
    srliw t0, t0, 8
    la t1, val2
    ld t1, 0(t1)
    bne t0, t1, next28
    sd t5, 216(gp)
next28:
    la t0, val_i32_min
    lw t0, 0(t0)
    srliw t0, t0, 31
    addiw t1, zero, 1
    bne t0, t1, next29
    sd t5, 224(gp)
next29:

    # --- SRAI / SRAIW ---
    addi t0, zero, 1024
    srai t0, t0, 5
    addi t1, zero, 32
    bne t0, t1, next30
    sd t5, 232(gp)
next30:
    addi t0, zero, -1024
    srai t0, t0, 5
    addi t1, zero, -32
    bne t0, t1, next31
    sd t5, 240(gp)
next31:
    addi t0, zero, -123
    srai t0, t0, 0
    addi t1, zero, -123
    bne t0, t1, next32
    sd t5, 248(gp)
next32:
    la t0, val_i64_min
    ld t0, 0(t0)
    srai t0, t0, 63
    addi t1, zero, -1
    bne t0, t1, next33
    sd t5, 256(gp)
next33:
    addiw t0, zero, 1024
    sraiw t0, t0, 5
    addiw t1, zero, 32
    bne t0, t1, next34
    sd t5, 264(gp)
next34:
    addiw t0, zero, -1024
    sraiw t0, t0, 5
    addiw t1, zero, -32
    bne t0, t1, next35
    sd t5, 272(gp)
next35:
    la t0, val_i32_min
    lw t0, 0(t0)
    sraiw t0, t0, 31
    addiw t1, zero, -1
    bne t0, t1, next36
    sd t5, 280(gp)
next36:

    # --- Loads ---
    la t0, val_store_pattern
    ld t1, 0(t0)
    la t2, val_store_pattern
    ld t2, 0(t2)
    bne t1, t2, next37
    sd t5, 288(gp)
next37:
    la t0, val_i32_max
    lw t1, 0(t0)
    la t2, val_i32_max
    lw t2, 0(t2)
    bne t1, t2, next38
    sd t5, 296(gp)
next38:
    la t0, val_i32_min
    lw t1, 0(t0)
    la t2, val_i32_min
    lw t2, 0(t2)
    bne t1, t2, next39
    sd t5, 304(gp)
next39:
    addi t0, zero, 12345
    la t6, val_zero
    sh t0, 0(t6)
    lh t1, 0(t6)
    bne t0, t1, next40
    sd t5, 312(gp)
next40:
    addi t0, zero, -12345
    la t6, val_zero
    sh t0, 0(t6)
    lh t1, 0(t6)
    bne t0, t1, next41
    sd t5, 320(gp)
next41:
    addi t0, zero, 123
    la t6, val_zero
    sb t0, 0(t6)
    lb t1, 0(t6)
    bne t0, t1, next42
    sd t5, 328(gp)
next42:
    addi t0, zero, -123
    la t6, val_zero
    sb t0, 0(t6)
    lb t1, 0(t6)
    bne t0, t1, next43
    sd t5, 336(gp)
next43:
    la t0, val_i32_min
    lwu t1, 0(t0)
    la t2, val3
    ld t2, 0(t2)
    bne t1, t2, next44
    sd t5, 344(gp)
next44:
    addi t0, zero, -1
    la t6, val_zero
    sh t0, 0(t6)
    lhu t1, 0(t6)
    la t2, half_word
    ld t2, 0(t2)
    bne t1, t2, next45
    sd t5, 352(gp)
next45:
    addi t0, zero, -1
    la t6, val_zero
    sb t0, 0(t6)
    lbu t1, 0(t6)
    addi t2, zero, 255
    bne t1, t2, next46
    sd t5, 360(gp)
next46:

    # --- Stores ---
    la t0, val_store_pattern
    ld t1, 0(t0)
    la t6, val_zero
    sd t1, 0(t6)
    ld t2, 0(t6)
    bne t1, t2, next47
    sd t5, 368(gp)
next47:
    la t0, val_i32_min
    lw t1, 0(t0)
    la t6, val_zero
    sw t1, 0(t6)
    lw t2, 0(t6)
    bne t1, t2, next48
    sd t5, 376(gp)
next48:
    la t0, neg_30k
    ld t0, 0(t0)
    la t6, val_zero
    sh t0, 0(t6)
    lh t1, 0(t6)
    bne t0, t1, next49
    sd t5, 384(gp)
next49:
    addi t0, zero, -100
    la t6, val_zero
    sb t0, 0(t6)
    lb t1, 0(t6)
    bne t0, t1, next50
    sd t5, 392(gp)
next50:

    # --- ADD / ADDW ---
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_pos_50
    ld t1, 0(t6)
    add t2, t0, t1
    la t6, val_pos_100
    ld t3, 0(t6)
    addi t3, t3, 50
    bne t2, t3, next51
    sd t5, 400(gp)
next51:
    la t0, val_i64_max
    ld t0, 0(t0)
    la t1, val_one
    ld t1, 0(t1)
    add t2, t0, t1
    la t3, val_i64_min
    ld t3, 0(t3)
    bne t2, t3, next52
    sd t5, 408(gp)
next52:
    la t0, val_pos_100
    ld t0, 0(t0)
    la t1, val_pos_50
    ld t1, 0(t1)
    addw t2, t0, t1
    addiw t3, zero, 150
    bne t2, t3, next53
    sd t5, 416(gp)
next53:
    addiw t0, zero, -2
    addw t1, t0, zero
    bne t0, t1, next54
    sd t5, 424(gp)
next54:

    # --- SUB / SUBW ---
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_pos_50
    ld t1, 0(t6)
    sub t2, t0, t1
    bne t2, t1, next55
    sd t5, 432(gp)
next55:
    la t0, val_i64_min
    ld t0, 0(t0)
    la t1, val_one
    ld t1, 0(t1)
    sub t2, t0, t1
    la t3, val_i64_max
    ld t3, 0(t3)
    bne t2, t3, next56
    sd t5, 440(gp)
next56:
    la t0, val_pos_100
    ld t0, 0(t0)
    la t1, val_pos_50
    ld t1, 0(t1)
    subw t2, t0, t1
    bne t2, t1, next57
    sd t5, 448(gp)
next57:
    subw t0, zero, t5
    la t6, val_neg_one
    ld t1, 0(t6)
    bne t0, t1, next58
    sd t5, 456(gp)
next58:

    # --- SLL / SLLW ---
    addi t0, zero, 20
    addi t1, zero, 10
    sll t2, t0, t1
    la t3, val1
    ld t3, 0(t3)
    bne t2, t3, next59
    sd t5, 464(gp)
next59:
    addi t0, zero, 123
    addi t1, zero, 64
    sll t2, t0, t1
    bne t2, t0, next60
    sd t5, 472(gp)
next60:
    addiw t0, zero, 20
    addiw t1, zero, 10
    sllw t2, t0, t1
    la t3, val1
    ld t3, 0(t3)
    bne t2, t3, next61
    sd t5, 480(gp)
next61:
    addiw t0, zero, 123
    addiw t1, zero, 32
    sllw t2, t0, t1
    bne t2, t0, next62
    sd t5, 488(gp)
next62:

    # --- SLT / SLTU ---
    la t6, val_neg_100
    ld t0, 0(t6)
    la t6, val_pos_100
    ld t1, 0(t6)
    slt t2, t0, t1
    bne t2, t5, next63
    sd t5, 496(gp)
next63:
    la t6, val_pos_100
    ld t0, 0(t6)
    slt t2, t0, t1
    bne t2, zero, next64
    sd t5, 504(gp)
next64:
    slt t2, t1, t0
    bne t2, zero, next65
    sd t5, 512(gp)
next65:
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_u64_large
    ld t1, 0(t6)
    sltu t2, t0, t1
    bne t2, t5, next66
    sd t5, 520(gp)
next66:
    sltu t2, t0, t0
    bne t2, zero, next67
    sd t5, 528(gp)
next67:
    sltu t2, t1, t0
    bne t2, zero, next68
    sd t5, 536(gp)
next68:
    la t6, val_neg_one
    ld t0, 0(t6)
    la t6, val_one
    ld t1, 0(t6)
    sltu t2, t0, t1
    bne t2, zero, next69
    sd t5, 544(gp)
next69:

    # --- XOR / OR / AND ---
    la t6, val_bitmask_a
    ld t0, 0(t6)
    la t6, val_bitmask_5
    ld t1, 0(t6)
    xor t2, t0, t1
    la t6, val_neg_one
    ld t3, 0(t6)
    bne t2, t3, next70
    sd t5, 552(gp)
next70:
    or t2, t0, t1
    bne t2, t3, next71
    sd t5, 560(gp)
next71:
    and t2, t0, t1
    bne t2, zero, next72
    sd t5, 568(gp)
next72:

    # --- SRL / SRLW ---
    la t0, val_u64_large
    ld t0, 0(t0)
    addi t1, zero, 4
    srl t2, t0, t1
    addi t3, zero, -1
    srli t3, t3, 4
    bne t2, t3, next73
    sd t5, 576(gp)
next73:
    addi t0, zero, 123
    addi t1, zero, 64
    srl t2, t0, t1
    bne t2, t0, next74
    sd t5, 584(gp)
next74:
    la t0, val_u32_large
    lw t0, 0(t0)
    addiw t1, zero, 4
    srlw t2, t0, t1
    la t3, val4
    ld t3, 0(t3)
    bne t2, t3, next75
    sd t5, 592(gp)
next75:
    addiw t0, zero, 123
    addiw t1, zero, 32
    srlw t2, t0, t1
    bne t2, t0, next76
    sd t5, 600(gp)
next76:

    # --- SRA / SRAW ---
    addi t0, zero, 1024
    addi t1, zero, 5
    sra t2, t0, t1
    addi t3, zero, 32
    bne t2, t3, next77
    sd t5, 608(gp)
next77:
    addi t0, zero, -1024
    addi t1, zero, 5
    sra t2, t0, t1
    addi t3, zero, -32
    bne t2, t3, next78
    sd t5, 616(gp)
next78:
    addi t0, zero, 123
    addi t1, zero, 64
    sra t2, t0, t1
    bne t2, t0, next79
    sd t5, 624(gp)
next79:
    addiw t0, zero, 1024
    addiw t1, zero, 5
    sraw t2, t0, t1
    addiw t3, zero, 32
    bne t2, t3, next80
    sd t5, 632(gp)
next80:
    addiw t0, zero, -1024
    addiw t1, zero, 5
    sraw t2, t0, t1
    addiw t3, zero, -32
    bne t2, t3, next81
    sd t5, 640(gp)
next81:
    addiw t0, zero, 123
    addiw t1, zero, 32
    sraw t2, t0, t1
    bne t2, t0, next82
    sd t5, 648(gp)
next82:

    # --- MUL / MULW ---
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_pos_50
    ld t1, 0(t6)
    mul t2, t0, t1
    la t3, fivek
    ld t3, 0(t3)
    bne t2, t3, next83
    sd t5, 656(gp)
next83:
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_neg_50
    ld t1, 0(t6)
    mul t2, t0, t1
    la t3, neg_5k
    ld t3, 0(t3)
    bne t2, t3, next84
    sd t5, 664(gp)
next84:
    la t6, val_neg_100
    ld t0, 0(t6)
    la t6, val_neg_50
    ld t1, 0(t6)
    mul t2, t0, t1
    la t3, fivek
    ld t3, 0(t3)
    bne t2, t3, next85
    sd t5, 672(gp)
next85:
    la t6, val_pos_100
    ld t0, 0(t6)
    mul t2, t0, zero
    bne t2, zero, next86
    sd t5, 680(gp)
next86:
    la t0, fiftyk
    ld t0, 0(t0)
    add t1, zero, t0
    mulw t2, t0, t1
    la t3, mul_neg
    ld t3, 0(t3)
    bne t2, t3, next87
    sd t5, 688(gp)
next87:
    la t0, val_i32_max
    lw t0, 0(t0)
    mulw t2, t0, t0
    addi t3, zero, 1
    bne t2, t3, next88
    sd t5, 696(gp)
next88:

    # --- MULH / MULHSU / MULHU ---
    li t0, -2
    la t1, huge
    ld t1, 0(t1)
    mulh t2, t0, t1
    li t3, -1
    bne t2, t3, next89
    sd t5, 704(gp)
next89:
    addi t0, zero, -1
    addi t1, zero, 2
    mulhsu t2, t0, t1
    bne t2, t0, next90
    sd t5, 712(gp)
next90:
    addi t0, zero, -1
    addi t1, zero, -2
    mulhu t2, t0, t1
    li t3, -3
    bne t2, t3, next91
    sd t5, 720(gp)
next91:

    # --- DIV / DIVU / DIVW / DIVUW ---
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_pos_10
    ld t1, 0(t6)
    div t2, t0, t1
    bne t2, t1, next92
    sd t5, 728(gp)
next92:
    la t6, val_pos_100
    ld t0, 0(t6)
    div t2, t0, zero
    la t6, val_neg_one
    ld t3, 0(t6)
    bne t2, t3, next93
    sd t5, 736(gp)
next93:
    la t0, val_i64_min
    ld t0, 0(t0)
    la t6, val_neg_one
    ld t1, 0(t6)
    div t2, t0, t1
    bne t2, t0, next94
    sd t5, 744(gp)
next94:
    la t6, val_pos_100
    ld t0, 0(t6)
    la t6, val_pos_10
    ld t1, 0(t6)
    divu t2, t0, t1
    bne t2, t1, next95
    sd t5, 752(gp)
next95:
    la t6, val_pos_100
    ld t0, 0(t6)
    divu t2, t0, zero
    la t6, val_neg_one
    ld t3, 0(t6)
    bne t2, t3, next96
    sd t5, 760(gp)
next96:
    addiw t0, zero, 100
    addiw t1, zero, 10
    divw t2, t0, t1
    bne t2, t1, next97
    sd t5, 768(gp)
next97:
    addiw t0, zero, 100
    divw t2, t0, zero
    la t6, val_neg_one
    ld t3, 0(t6)
    bne t2, t3, next98
    sd t5, 776(gp)
next98:
    la t0, val_i32_min
    lw t0, 0(t0)
    la t6, val_neg_one
    ld t1, 0(t6)
    divw t2, t0, t1
    bne t2, t0, next99
    sd t5, 784(gp)
next99:
    addiw t0, zero, 100
    addiw t1, zero, 10
    divuw t2, t0, t1
    bne t2, t1, next100
    sd t5, 792(gp)
next100:
    addiw t0, zero, 100
    divuw t2, t0, zero
    la t6, val_neg_one
    ld t3, 0(t6)
    bne t2, t3, next101
    sd t5, 800(gp)
next101:

    # --- REM / REMU / REMW / REMUW ---
    la t6, val_pos_100
    ld t0, 0(t6)
    addi t1, zero, 13
    rem t2, t0, t1
    addi t3, zero, 9
    bne t2, t3, next102
    sd t5, 808(gp)
next102:
    la t6, val_pos_100
    ld t0, 0(t6)
    rem t2, t0, zero
    bne t2, t0, next103
    sd t5, 816(gp)
next103:
    la t0, val_i64_min
    ld t0, 0(t0)
    la t6, val_neg_one
    ld t1, 0(t6)
    rem t2, t0, t1
    bne t2, zero, next104
    sd t5, 824(gp)
next104:
    la t6, val_pos_100
    ld t0, 0(t6)
    addi t1, zero, 13
    remu t2, t0, t1
    addi t3, zero, 9
    bne t2, t3, next105
    sd t5, 832(gp)
next105:
    la t6, val_pos_100
    ld t0, 0(t6)
    remu t2, t0, zero
    bne t2, t0, next106
    sd t5, 840(gp)
next106:
    addiw t0, zero, 100
    addiw t1, zero, 13
    remw t2, t0, t1
    addiw t3, zero, 9
    bne t2, t3, next107
    sd t5, 848(gp)
next107:
    addiw t0, zero, 100
    remw t2, t0, zero
    bne t2, t0, next108
    sd t5, 856(gp)
next108:
    la t0, val_i32_min
    lw t0, 0(t0)
    la t6, val_neg_one
    ld t1, 0(t6)
    remw t2, t0, t1
    bne t2, zero, next109
    sd t5, 864(gp)
next109:
    addiw t0, zero, 100
    addiw t1, zero, 13
    remuw t2, t0, t1
    addiw t3, zero, 9
    bne t2, t3, next110
    sd t5, 872(gp)
next110:
    addiw t0, zero, 100
    remuw t2, t0, zero
    bne t2, t0, next111
    sd t5, 880(gp)
next111:

    # --- Branches ---
    beq zero, zero, beq_taken_target
    j beq_not_taken_target
beq_taken_target:
    sd t5, 888(gp)
beq_not_taken_target:
    beq zero, t5, next112
    sd t5, 896(gp)
next112:
    bne zero, t5, bne_taken_target
    j bne_not_taken_target
bne_taken_target:
    sd t5, 904(gp)
bne_not_taken_target:
    bne zero, zero, next113
    sd t5, 912(gp)
next113:
    addi t0, zero, -1
    blt t0, zero, blt_taken_target
    j blt_not_taken_target
blt_taken_target:
    sd t5, 920(gp)
blt_not_taken_target:
    blt zero, t0, next114
    sd t5, 928(gp)
next114:
    bge zero, t0, bge_taken_target
    j bge_not_taken_target
bge_taken_target:
    sd t5, 936(gp)
bge_not_taken_target:
    bge t0, zero, next115
    sd t5, 944(gp)
next115:
    bltu zero, t5, bltu_taken_target
    j bltu_not_taken_target
bltu_taken_target:
    sd t5, 952(gp)
bltu_not_taken_target:
    bltu t5, zero, next116
    sd t5, 960(gp)
next116:
    bgeu t5, zero, bgeu_taken_target
    j bgeu_not_taken_target
bgeu_taken_target:
    sd t5, 968(gp)
bgeu_not_taken_target:
    bgeu zero, t5, next117
    sd t5, 976(gp)
next117:

    # --- Jumps ---
    auipc t0, 0
    addi t0, t0, 12
    jal ra, jal_target
    nop
jal_target:
    bne ra, t0, next118
    sd t5, 984(gp)
next118:
    auipc t0, 0
    addi t0, t0, 20 # la is actually 2 instructions so 8 more than the previous addition
    la t1, jalr_target
    jalr ra, 0(t1)
    nop
jalr_target:
    bne ra, t0, next119
    sd t5, 992(gp)
next119:

    # --- CSRs ---
    addi t0, zero, 123
    csrrw t1, mscratch, t0
    csrrw t2, mscratch, t1
    bne t0, t2, next120
    sd t5, 1000(gp)
next120:
    csrrw zero, mscratch, zero
    addi t0, zero, 0b0101
    csrrw zero, mscratch, t0
    addi t1, zero, 0b1001
    csrrs zero, mscratch, t1
    csrrs t2, mscratch, zero
    addi t3, zero, 0b1101
    bne t2, t3, next121
    sd t5, 1008(gp)
next121:
    addi t0, zero, -1
    csrrw zero, mscratch, t0
    addi t1, zero, 0b0101
    csrrc zero, mscratch, t1
    csrrs t2, mscratch, zero
    addi t3, zero, -1
    xori t3, t3, 0b0101
    bne t2, t3, next122
    sd t5, 1016(gp)
next122:
    csrrwi t1, mscratch, 12
    csrrwi t2, mscratch, 0
    addi t3, zero, 12
    bne t2, t3, next123
    sd t5, 1024(gp)
next123:
    csrrw zero, mscratch, zero
    csrrsi zero, mscratch, 0b0101
    csrrsi zero, mscratch, 0b1001
    csrrs t2, mscratch, zero
    addi t3, zero, 0b1101
    bne t2, t3, next124
    sd t5, 1032(gp)
next124:
    addi t0, zero, -1
    csrrw zero, mscratch, t0
    csrrci zero, mscratch, 0b0101
    csrrs t2, mscratch, zero
    addi t3, zero, -1
    xori t3, t3, 0b0101
    bne t2, t3, next125
    sd t5, 1040(gp)
next125:

final_exit:
    addi a7, zero, 93
    ecall
