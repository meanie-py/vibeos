//! Loader asm flipped from standalone `.S` files to Rust `global_asm!`, with
//! asm.inc macros expanded inline (PUBLIC→.global, EXTERN→dropped, HEX(x)→0x…,
//! END→dropped, .code32/64 native) and pcbios.h/linux.h constants `.set` in the
//! block — so each is self-contained (no asm.inc, no include dir, no `.S`).
//!
//! Per-file feature gating matches which loader links it:
//!   misc.S   (amd64) — uefildr + rosload  → any(uefi, rosload)
//!   linux.S  (i386/amd64) — rosload        → rosload
//!   i386trap.S (i386) — uefildr            → uefi
//! (uefiasm.S lives in uefiasm.rs; drvmap.S is 16-bit gcc, can't be global_asm!.)

// ---------------------------------------------------------------------------
// amd64 misc.S — __fastfail / __lgdt / __ltr / __sgdt  (GAS / !_USE_ML)
// ---------------------------------------------------------------------------
#[cfg(all(target_arch = "x86_64", any(feature = "uefi", feature = "rosload")))]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code64

/* void __fastfail(unsigned int Code<rcx>); — does not return */
.global __fastfail
__fastfail:
    int 0x29

/* void __lgdt(void *Source<rcx>); */
.global __lgdt
__lgdt:
    lgdt cs:[rcx]
    ret

/* void __ltr(unsigned short Source<rcx>); */
.global __ltr
__ltr:
    ltr cx
    ret

/* void _sgdt(void *Destination<rcx>); */
.global __sgdt
__sgdt:
    sgdt cs:[rcx]
    ret
"#
);

// ---------------------------------------------------------------------------
// i386 linux.S — _BootLinuxKernel  (REGS_* from pcbios.h via .set; #if 0→#else)
// ---------------------------------------------------------------------------
#[cfg(all(target_arch = "x86", feature = "rosload"))]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code32
.set REGS_DS, 28
.set REGS_ES, 30
.set REGS_FS, 32
.set REGS_GS, 34
.set REGS_SIZE, 40

Regs:
    .space REGS_SIZE

.global _BootLinuxKernel
_BootLinuxKernel:
    /* Stop the floppy drive motor */
    call _DiskStopFloppyMotor
    /* Set all segment registers to 0x9000 */
    mov ax, 0x9000
    mov word ptr [Regs + REGS_DS], ax
    mov word ptr [Regs + REGS_ES], ax
    mov word ptr [Regs + REGS_FS], ax
    mov word ptr [Regs + REGS_GS], ax
    /* Get KernelSize in ECX */
    mov ecx, dword ptr [esp + 4]
    test ecx, ecx
    jz after_reloc
    /* Load the source and target addresses */
    mov esi, dword ptr [esp + 8]
    mov edi, dword ptr [esp + 12]
    cmp edi, esi
    je after_reloc
    ja reloc_up
reloc_down:
    cld
    mov edx, ecx
    and edx, 0x0FFFFFFFC
    xor edx, ecx
    shr ecx, 2
    rep movsd
    mov ecx, edx
    rep movsb
    jmp after_reloc
reloc_up:
    std
    add esi, ecx
    add edi, ecx
    dec esi
    dec edi
    rep movsb
after_reloc:
    push 0x0000      /* CodePointer */
    push 0x9020      /* CodeSegment */
    push 0x9000      /* StackPointer */
    push 0x9000      /* StackSegment */
    mov eax, offset Regs
    push eax
    call _Relocator16Boot
    /* We must never get there */
    int 3
"#
);

// ---------------------------------------------------------------------------
// amd64 linux.S — BootLinuxKernel
// ---------------------------------------------------------------------------
#[cfg(all(target_arch = "x86_64", feature = "rosload"))]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code64
.set REGS_DS, 28
.set REGS_ES, 30
.set REGS_FS, 32
.set REGS_GS, 34
.set REGS_SIZE, 40

Regs:
    .space REGS_SIZE

.global BootLinuxKernel
BootLinuxKernel:
    /* Save home registers */
    mov r11, rsp
    mov dword ptr [r11 +  8], ecx
    mov qword ptr [r11 + 16], rdx
    mov qword ptr [r11 + 24], r8
    /* Save non-volatile registers */
    push rsi
    push rdi
    /* Allocate stack space for home registers (+ alignment) */
    sub rsp, (8*4 + 8)
    /* Stop the floppy drive motor */
    call DiskStopFloppyMotor
    /* Set all segment registers to 0x9000 */
    mov ax, 0x9000
    mov word ptr [Regs + REGS_DS], ax
    mov word ptr [Regs + REGS_ES], ax
    mov word ptr [Regs + REGS_FS], ax
    mov word ptr [Regs + REGS_GS], ax
    /* Get KernelSize in ECX */
    xor rcx, rcx
    mov ecx, dword ptr [r11 + 8]
    test rcx, rcx
    jz after_reloc
    /* Load the source and target addresses */
    mov rsi, qword ptr [r11 + 16]
    mov rdi, qword ptr [r11 + 24]
    cmp rdi, rsi
    je after_reloc
    ja reloc_up
reloc_down:
    cld
    mov edx, ecx
    and edx, 0x0FFFFFFFC
    xor edx, ecx
    shr ecx, 2
    rep movsd
    mov ecx, edx
    rep movsb
    jmp after_reloc
reloc_up:
    std
    add rsi, rcx
    add rdi, rcx
    dec rsi
    dec rdi
    rep movsb
after_reloc:
    mov word ptr [rsp-8 + 40], 0x0000   /* CodePointer */
    mov r9w, 0x9020                      /* CodeSegment */
    mov r8w, 0x9000                      /* StackPointer */
    mov  dx, 0x9000                      /* StackSegment */
    mov rcx, offset Regs
    call Relocator16Boot
    /* Cleanup and return */
    add rsp, (8*4 + 8)
    pop rdi
    pop rsi
    /* We must never get there */
    int 3
"#
);

// ---------------------------------------------------------------------------
// i386 i386trap.S — exception trap stubs  (MACRO→.macro, VAL(x)→\x; DBG=1 so the
// breakpoint templates are included; !_USE_ML label form)
// ---------------------------------------------------------------------------
#[cfg(all(target_arch = "x86", feature = "uefi"))]
core::arch::global_asm!(
    r#"
.intel_syntax noprefix
.code32

.macro SAVE_CPU_REGS
    /* push the rest of the KTRAP_FRAME */
    push ebp
    push ebx
    push esi
    push edi
    push fs
    push 0
    push 0
    push eax
    push ecx
    push edx
    push ds
    push es
    push gs
    mov eax, dr7
    push eax
    mov eax, dr6
    push eax
    mov eax, dr3
    push eax
    mov eax, dr2
    push eax
    mov eax, dr1
    push eax
    mov eax, dr0
    push eax
    sub esp, 6 * 4
    /* push KSPECIAL_REGISTERS: Gdtr, Idtr, Tr, Ldtr, Reserved */
    sub esp, 44
    sgdt [esp]
    sidt [esp + 8]
    str word ptr [esp + 16]
    sldt word ptr [esp + 18]
    mov eax, dr7
    push eax
    mov eax, dr6
    push eax
    mov eax, dr3
    push eax
    mov eax, dr2
    push eax
    mov eax, dr1
    push eax
    mov eax, dr0
    push eax
    mov eax, cr4
    push eax
    mov eax, cr3
    push eax
    mov eax, cr2
    push eax
    mov eax, cr0
    push eax
.endm

/* Set by each exception handler to the index of the description text */
i386ExceptionIndex:
    .long 0

i386CommonExceptionHandler:
    SAVE_CPU_REGS
    lea eax, [esp + (21 * 4)]            /* KTRAP_FRAME */
    push esp                             /* KSPECIAL_REGISTERS */
    push eax
    push dword ptr ds:[i386ExceptionIndex]
    call _i386PrintExceptionText@12
    cli
i386ExceptionHandlerHang:
    hlt
    jmp i386ExceptionHandlerHang
    iret

.macro TRAP_STUB function, index
    .global \function
\function\():
    push 0                              /* Fake error code */
    mov dword ptr ds:[i386ExceptionIndex], \index
    jmp i386CommonExceptionHandler
.endm

.macro TRAP_STUB2 function, index
    .global \function
\function\():
    mov dword ptr ds:[i386ExceptionIndex], \index
    jmp i386CommonExceptionHandler
.endm

TRAP_STUB _i386DivideByZero, 0
TRAP_STUB _i386DebugException, 1
TRAP_STUB _i386NMIException, 2
TRAP_STUB _i386Breakpoint, 3
TRAP_STUB _i386Overflow, 4
TRAP_STUB _i386BoundException, 5
TRAP_STUB _i386InvalidOpcode, 6
TRAP_STUB _i386FPUNotAvailable, 7
TRAP_STUB2 _i386DoubleFault, 8
TRAP_STUB _i386CoprocessorSegment, 9
TRAP_STUB2 _i386InvalidTSS, 10
TRAP_STUB2 _i386SegmentNotPresent, 11
TRAP_STUB2 _i386StackException, 12
TRAP_STUB2 _i386GeneralProtectionFault, 13
TRAP_STUB2 _i386PageFault, 14
TRAP_STUB _i386CoprocessorError, 16
TRAP_STUB _i386AlignmentCheck, 17
TRAP_STUB _i386MachineCheck, 18
TRAP_STUB _i386SimdFloatError, 19

/* DBG=1: breakpoint debugging support */
.macro BREAKPOINT_TEMPLATE functionname, mask1, mask2
    .global \functionname
\functionname\():
    push eax
    mov eax, [esp + 8]
    mov dr3, eax
    mov eax, dr7
    and eax, \mask1
    or  eax, \mask2
    mov dr7, eax
    pop eax
    ret
.endm

BREAKPOINT_TEMPLATE _INSTRUCTION_BREAKPOINT1, 0x0fff0ffff, 0x000000303
BREAKPOINT_TEMPLATE _MEMORY_READWRITE_BREAKPOINT1, 0x0fff0ffff, 0x000030303
BREAKPOINT_TEMPLATE _MEMORY_WRITE_BREAKPOINT1, 0x0fff0ffff, 0x000010303
BREAKPOINT_TEMPLATE _INSTRUCTION_BREAKPOINT2, 0x0ff0fffff, 0x00000030c
BREAKPOINT_TEMPLATE _MEMORY_READWRITE_BREAKPOINT2, 0x0ff0fffff, 0x00030030c
BREAKPOINT_TEMPLATE _MEMORY_WRITE_BREAKPOINT2, 0x0ff0fffff, 0x00010030c
BREAKPOINT_TEMPLATE _INSTRUCTION_BREAKPOINT3, 0x0f0ffffff, 0x000000330
BREAKPOINT_TEMPLATE _MEMORY_READWRITE_BREAKPOINT3, 0x0f0ffffff, 0x003000330
BREAKPOINT_TEMPLATE _MEMORY_WRITE_BREAKPOINT3, 0x0f0ffffff, 0x001000330
BREAKPOINT_TEMPLATE _INSTRUCTION_BREAKPOINT4, 0x00fffffff, 0x0000003c0
BREAKPOINT_TEMPLATE _MEMORY_READWRITE_BREAKPOINT4, 0x00fffffff, 0x0300003c0
BREAKPOINT_TEMPLATE _MEMORY_WRITE_BREAKPOINT4, 0x00fffffff, 0x0100003c0
"#
);
