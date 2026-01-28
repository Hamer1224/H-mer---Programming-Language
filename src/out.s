.global _start
.section .text

_start:
    mov x0, #42

    // --- Probability Block ---
    sub sp, sp, #16
    mov x0, sp
    mov x1, #1
    mov x8, #278
    svc #0
    ldrb w0, [sp]
    add sp, sp, #16
    cmp w0, #128
    b.hi .L_done
    mov x0, #7
.L_done:
    mov x8, #93
    svc #0
