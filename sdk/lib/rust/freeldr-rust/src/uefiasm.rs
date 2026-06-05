//! UEFI escape helpers — flipped from `arch/uefi/<arch>/uefiasm.S` to Rust
//! `global_asm!`. These switch off the UEFI boot-services stack, call
//! ExitBootServices / ExecuteLoaderCleanly, and (x86) install FreeLDR's own
//! GDT/IDT before handing off to the OS loader.
//!
//! asm.inc macros are expanded inline (PUBLIC→`.global`, EXTERN→dropped,
//! HEX(x)→`0x…`, END→dropped, `.code32/64` native; arm64 TEXTAREA→`.text` +
//! LEAF_ENTRY/END→`.globl`/label), so no asm.inc / include dir is needed — the
//! asm is self-contained Rust. Built into libfreeldr_rust_uefi.a (feature =
//! "uefi"); `_exituefi`/`_changestack` are referenced from uefimem.rs/uefildr.rs.
//!
//! The symbol names match the C originals' per-arch decoration: i386 cdecl adds
//! a leading `_`, so the C `_exituefi` is the asm symbol `__exituefi`; amd64 and
//! arm64 are undecorated (`_exituefi`).
#![cfg(feature = "uefi")]

// ---------------------------------------------------------------------------
// i386 — arch/uefi/i386/uefiasm.S  (GAS / !_USE_ML branch)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code32

.global __exituefi
__exituefi:
    push ebp
    push esi
    push edi
    push ebx
    /* Save the old stack */
    mov ebx, esp
    /* Load the new stack */
    xor ebp, ebp
    mov esp, _UefiServiceStack
    /* Call the entry routine */
    call _UefiExitBootServices
    /* Restore old stack */
    mov esp, ebx
    /* Restore non-volatiles */
    pop ebx
    pop edi
    pop esi
    pop ebp
    lgdt cs:[_gdtptr]
    lidt _i386idtptr
    ret

.global __changestack
__changestack:
    mov eax, esp
    mov esp, _BasicStack
    push eax
    call _ExecuteLoaderCleanly
    ret

    .align 4
gdt:
    /* NULL Descriptor */
    .word 0x0000
    .word 0x0000
    .word 0x0000
    .word 0x0000
    /* 32-bit flat CS */
    .word 0xFFFF
    .word 0x0000
    .word 0x9A00
    .word 0x00CF
    /* 32-bit flat DS */
    .word 0xFFFF
    .word 0x0000
    .word 0x9200
    .word 0x00CF
    /* 16-bit real mode CS */
    .word 0xFFFF
    .word 0x0000
    .word 0x9E00
    .word 0x0000
    /* 16-bit real mode DS */
    .word 0xFFFF
    .word 0x0000
    .word 0x9200
    .word 0x0000

.global _gdtptr
_gdtptr:
    .word 0x27       /* Limit */
    .long gdt, 0     /* Base Address */

.global _i386idtptr
_i386idtptr:
    .word 255        /* Limit */
    .long _i386Idt   /* Base Address */
"#
);

// ---------------------------------------------------------------------------
// amd64 — arch/uefi/amd64/uefiasm.S  (GAS / !_USE_ML branch)
// ---------------------------------------------------------------------------
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code64

.global _exituefi
_exituefi:
    push rbp
    push rsi
    push rdi
    push rbx
    /* Save the old stack */
    mov rbx, rsp
    /* Load the new stack */
    xor rbp, rbp
    mov rsp, qword ptr UefiServiceStack[rip]
    /* MS x64 ABI: 16-byte alignment + 32-byte shadow space */
    and rsp, -16
    sub rsp, 0x20
    call UefiExitBootServices
    /* Restore old stack */
    mov rsp, rbx
    pop rbx
    pop rdi
    pop rsi
    pop rbp
    /* Disable maskable ints across the GDT swap */
    cli
    lgdt cs:[_gdtptr][rip]
    /* Reload CS=0x10 via far return */
    lea rax, reload_cs[rip]
    push 0x10
    push rax
    .byte 0x48, 0xCB     /* retfq */

reload_cs:
    mov ax, 0x18
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax
    mov ax, 0x28
    mov ss, ax
    ret

.global _changestack
_changestack:
    mov rax, rsp
    mov rsp, BasicStack[rip]
    and rsp, -16
    sub rsp, 0x20
    mov rcx, rax
    call ExecuteLoaderCleanly
    ret

.align 8
gdt:
    .word 0x0000, 0x0000, 0x0000, 0x0000 /* 00: NULL */
    .word 0x0000, 0x0000, 0x0000, 0x0000 /* 08: */
    .word 0x0000, 0x0000, 0x9800, 0x0020 /* 10: long mode CS */
    .word 0xFFFF, 0x0000, 0xF300, 0x00CF /* 18: long mode DS */
    .word 0xFFFF, 0x0000, 0x9E00, 0x0000 /* 20: 16-bit real mode CS */
    .word 0xFFFF, 0x0000, 0x9200, 0x0000 /* 28: 16-bit real mode DS */
    .word 0xFFFF, 0x0000, 0x9B00, 0x00CF /* 30: compat mode CS */
    .word 0x0000, 0x0000, 0x9800, 0x0020 /* 38: long mode CS (UEFI compat) */

.global _gdtptr
_gdtptr:
    .word 0x3F           /* Limit */
    .quad gdt, 0         /* Base Address */
"#
);

// ---------------------------------------------------------------------------
// arm64 — arch/uefi/arm64/uefiasm.S
// ---------------------------------------------------------------------------
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    r#"
.text
.p2align 2

.globl _exituefi
.p2align 2
_exituefi:
    /* Save callee-saved registers */
    stp x19, x20, [sp, #-16]!
    stp x21, x22, [sp, #-16]!
    stp x23, x24, [sp, #-16]!
    stp x25, x26, [sp, #-16]!
    stp x27, x28, [sp, #-16]!
    stp x29, x30, [sp, #-16]!
    /* Save the old stack pointer */
    mov x19, sp
    /* Load the UEFI service stack (adrp+add+ldr; avoid :lo12: align reqs) */
    adrp x0, UefiServiceStack
    add x0, x0, :lo12:UefiServiceStack
    ldr x0, [x0]
    mov sp, x0
    /* Call UefiExitBootServices */
    bl UefiExitBootServices
    /* Restore old stack */
    mov sp, x19
    /* Restore callee-saved registers */
    ldp x29, x30, [sp], #16
    ldp x27, x28, [sp], #16
    ldp x25, x26, [sp], #16
    ldp x23, x24, [sp], #16
    ldp x21, x22, [sp], #16
    ldp x19, x20, [sp], #16
    ret

.globl _changestack
.p2align 2
_changestack:
    mov x0, sp
    adrp x1, BasicStack
    add x1, x1, :lo12:BasicStack
    ldr x1, [x1]
    mov sp, x1
    str x0, [sp, #-16]!
    bl ExecuteLoaderCleanly
    ldr x0, [sp], #16
    mov sp, x0
    ret
"#
);
