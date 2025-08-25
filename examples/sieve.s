# =============================================================================
# sieve.s
#
# A test program to find the number of prime numbers up to a given limit
# using the Sieve of Eratosthenes algorithm.
#
# This program is designed to be a comprehensive test for a 64-bit RISC-V VM,
# exercising memory access (la, ld, lb, sb), integer arithmetic (add, mul),
# and complex branching logic (blt, bne).
#
# EXPECTED RESULT: The program should exit with code 168.
# (There are 168 prime numbers between 1 and 1000).
# =============================================================================

.section .data
    # The upper bound for the sieve. Must be loaded from memory due to `li` limitations.
    .align 3
    SIEVE_LIMIT: .quad 1000

.section .bss
    # A byte array to mark numbers. is_prime[i] = 1 means i is potentially prime.
    # We need SIEVE_LIMIT + 1 bytes.
    .align 3
    is_prime_array: .space 1001

.section .text
.global _start
_start:
    # --- Setup ---
    # Load the sieve limit from .data into s0
    la s0, SIEVE_LIMIT
    ld s0, 0(s0)            # s0 = 1000

    # Load the base address of our marking array into s1
    la s1, is_prime_array   # s1 = &is_prime_array

    # --- Step 1: Initialize the array ---
    # Assume all numbers from 2 to N are prime.
    # We will set every byte from index 2 to N to 1.
    li s2, 2                # s2 = i = 2 (loop counter)
init_loop:
    # Set is_prime_array[i] = 1
    add t0, s1, s2          # t0 = address of is_prime_array[i]
    li t1, 1
    sb t1, 0(t0)            # Store 1 at that address

    addi s2, s2, 1          # i++
    bge s0, s2, init_loop   # Loop while limit >= i

    # --- Step 2: Run the Sieve ---
    # Start with p = 2, the first prime.
    li s2, 2                # s2 = p = 2
sieve_outer_loop:
    # Optimization: if p*p > limit, we can stop.
    mul t0, s2, s2          # t0 = p*p
    blt s0, t0, sieve_done  # if (limit < p*p), we are done

    # Check if is_prime_array[p] is still 1. If not, it's not a prime, so skip.
    add t0, s1, s2          # t0 = &is_prime_array[p]
    lb t1, 0(t0)            # t1 = is_prime_array[p]
    bne t1, zero, mark_multiples # if is_prime_array[p] != 0, then p is prime
    j continue_outer        # Otherwise, skip to the next p

mark_multiples:
    # p is prime. Mark all multiples of p as not prime.
    # Start marking from p*p.
    mul s3, s2, s2          # s3 = j = p*p (inner loop counter)
sieve_inner_loop:
    # --- CORRECTED LINE ---
    # if (j > limit), we are done with this p. This is equivalent to `blt limit, j`.
    blt s0, s3, continue_outer

    # Set is_prime_array[j] = 0
    add t0, s1, s3          # t0 = &is_prime_array[j]
    sb zero, 0(t0)

    add s3, s3, s2          # j = j + p
    j sieve_inner_loop

continue_outer:
    addi s2, s2, 1          # p++
    bge s0, s2, sieve_outer_loop # Loop while limit >= p

sieve_done:
    # --- Step 3: Count the primes ---
    # The number of primes is the number of remaining 1s in the array.
    li s2, 2                # s2 = i = 2 (loop counter)
    li s4, 0                # s4 = prime_count = 0
count_loop:
    add t0, s1, s2          # t0 = &is_prime_array[i]
    lb t1, 0(t0)            # t1 = is_prime_array[i]
    beq t1, zero, skip_inc  # If it's 0, it's not prime

    addi s4, s4, 1          # prime_count++

skip_inc:
    addi s2, s2, 1          # i++
    bge s0, s2, count_loop  # Loop while limit >= i

    # --- Final Exit ---
    # Move the final count into a0 to use as the exit code.
    add a0, s4, zero

    # Standard exit ecall
    li a7, 93
    ecall
