.section .data
str_3: .ascii "Total Winners:\n"
.section .text
.section .data
str_0: .ascii "Running 50 Probability Tests...\n"
.section .text
.global _start
.section .text

_start:
    mov x11, #10
    mov x0, #0
    mov x1, #4096
    mov x2, #3
    mov x3, #34
    mov x4, #-1
    mov x5, #0
    mov x8, #222
    svc #0
    mov x20, x0
    mov x12, x20
    add x20, x20, #16
    mov x1, #31415
    str x1, [x12, #0]
    
    mrs x0 , cntvct_el0 
    mov x1 , sp 
    eor x0 , x0 , x1 
    mov x1 , x12 
    str x0 , [ x1 , #8 ] 
    mov x13, x20
    add x20, x20, #16
    mov x14, #50
    mov x15, #0
    mov x0, #1
    ldr x1, =str_0
    mov x2, #32
    mov x8, #64
    svc #0
.Lloop1:
    mov x1, x14
    cmp x1, #0
    b.le .Lexit1

    // Prob Roll 30%
    ldr x1, [x13, #8]
    cmp x1, #0
    b.ne .Lseedok2
    mrs x1, cntvct_el0
.Lseedok2:
    ldr x2, =0x9E3779B97F4A7C15
    mul x1, x1, x2
    eor x1, x1, x1, lsr #33
    str x1, [x13, #8]
    and x1, x1, #0x7FFFFFFF
    mov x2, #100
    udiv x3, x1, x2
    msub x1, x3, x2, x1
    cmp x1, #30
    b.hs .Lif2
    mov x1, x15
    add x1, x1, #1
    mov x15, x1
.Lif2:
    mov x1, x14
    sub x1, x1, #1
    mov x14, x1
    b .Lloop1
.Lexit1:
    mov x0, #1
    ldr x1, =str_3
    mov x2, #15
    mov x8, #64
    svc #0

    stp x0, x1, [sp, #-16]!
    mov x0, x15
    sub sp, sp, #32
    mov x1, sp
    add x1, x1, #31
    mov w2, #10
    strb w2, [x1]
.Lp1279:
    sub x1, x1, #1
    udiv x2, x0, x11
    msub x3, x2, x11, x0
    add x3, x3, #48
    strb w3, [x1]
    mov x0, x2
    cbnz x0, .Lp1279
    mov x0, #1
    mov x2, sp
    add x2, x2, #32
    sub x2, x2, x1
    mov x8, #64
    svc #0
    add sp, sp, #32
    ldp x0, x1, [sp], #16

    mov x0, #0
    mov x8, #93
    svc #0
