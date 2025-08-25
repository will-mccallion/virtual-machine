# kernel.s
# This is a placeholder OS. It will be loaded by the BIOS to 0x80100000.

.section .text
.global _start

_start:
    # The OS is now in control!
    # In a real OS, you would now initialize drivers, filesystems, etc.
    # For now, we just prove we got here by entering an infinite loop.
os_loop:
    j os_loop
