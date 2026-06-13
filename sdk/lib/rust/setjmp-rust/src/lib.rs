#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]
//! setjmp-rust — the amd64 vcruntime `setjmp`/`longjmp` translated to Rust.
//!
//! Faithful translation of sdk/lib/vcruntime/amd64/{_setjmp.s,
//! __longjmp_noframe.s,longjmp.c}. Only amd64 needs these (the kernel's SEH
//! unwind path references `_setjmp`/`_setjmpex`/`longjmp`); i386 uses DUMMY_PSEH
//! and arm64's kernel doesn't reference them, so `libsetjmp.a` is inert there.
//!
//! The register save/restore is irreducible machine code, so it lives in
//! `global_asm!` (transcribed verbatim from the .s) — but every `_JUMP_BUFFER`
//! field offset is supplied as a `const offset_of!(...)` operand, so the asm
//! cannot drift from the struct layout. `longjmp` (the thin RtlUnwind glue) is
//! genuine Rust. Validated by disassembly-equivalence vs the cmake libsetjmp.a.

#[cfg(target_arch = "x86_64")]
mod amd64 {
    use core::mem::offset_of;
    use ntkrnl_sys::ddk::{RtlUnwind, EXCEPTION_RECORD};

    /// 16-byte XMM save slot (`SETJMP_FLOAT128`), 16-aligned for `movdqa`.
    #[repr(C, align(16))]
    struct Float128([u8; 16]);

    /// amd64 `_JUMP_BUFFER` (sdk/include/vcruntime/setjmp.h). `repr(C, align(16))`
    /// so the `Xmm*` slots land 16-aligned (the genuine jmp_buf is 16-aligned)
    /// and `offset_of!` yields the exact ksamd64.inc `Jb*` offsets.
    #[repr(C, align(16))]
    struct JumpBuffer {
        Frame: u64,
        Rbx: u64,
        Rsp: u64,
        Rbp: u64,
        Rsi: u64,
        Rdi: u64,
        R12: u64,
        R13: u64,
        R14: u64,
        R15: u64,
        Rip: u64,
        MxCsr: u32,
        FpCsr: u16,
        Spare: u16,
        Xmm6: Float128,
        Xmm7: Float128,
        Xmm8: Float128,
        Xmm9: Float128,
        Xmm10: Float128,
        Xmm11: Float128,
        Xmm12: Float128,
        Xmm13: Float128,
        Xmm14: Float128,
        Xmm15: Float128,
    }

    // _setjmp/_setjmpex/__intrinsic_setjmp[ex]: save the call's rsp+return addr,
    // the Frame (rdx), FP control state, and the non-volatile GPRs + xmm6-15
    // into the _JUMP_BUFFER (rcx). Returns 0. __longjmp_noframe: the inverse —
    // restore everything and jmp to the saved Rip, returning _Value (edx) or 1.
    // Transcribed verbatim from amd64/_setjmp.s + __longjmp_noframe.s; offsets
    // are offset_of! const operands.
    core::arch::global_asm!(
        ".intel_syntax noprefix",
        ".text",
        ".globl _setjmp",
        ".globl _setjmpex",
        ".globl __intrinsic_setjmp",
        ".globl __intrinsic_setjmpex",
        "_setjmp:",
        "_setjmpex:",
        "__intrinsic_setjmp:",
        "__intrinsic_setjmpex:",
        "    lea rax, [rsp + 8]",
        "    mov r8, [rsp]",
        "    mov [rcx + {Frame}], rdx",
        "    stmxcsr [rcx + {MxCsr}]",
        "    fnstcw [rcx + {FpCsr}]",
        "    mov [rcx + {Rbx}], rbx",
        "    mov [rcx + {Rbp}], rbp",
        "    mov [rcx + {Rsi}], rsi",
        "    mov [rcx + {Rdi}], rdi",
        "    mov [rcx + {R12}], r12",
        "    mov [rcx + {R13}], r13",
        "    mov [rcx + {R14}], r14",
        "    mov [rcx + {R15}], r15",
        "    mov [rcx + {Rsp}], rax",
        "    mov [rcx + {Rip}], r8",
        "    movdqa [rcx + {Xmm6}], xmm6",
        "    movdqa [rcx + {Xmm7}], xmm7",
        "    movdqa [rcx + {Xmm8}], xmm8",
        "    movdqa [rcx + {Xmm9}], xmm9",
        "    movdqa [rcx + {Xmm10}], xmm10",
        "    movdqa [rcx + {Xmm11}], xmm11",
        "    movdqa [rcx + {Xmm12}], xmm12",
        "    movdqa [rcx + {Xmm13}], xmm13",
        "    movdqa [rcx + {Xmm14}], xmm14",
        "    movdqa [rcx + {Xmm15}], xmm15",
        "    xor rax, rax",
        "    ret",
        ".globl __longjmp_noframe",
        "__longjmp_noframe:",
        "    ldmxcsr [rcx + {MxCsr}]",
        "    fldcw [rcx + {FpCsr}]",
        "    fnclex",
        "    mov rbx, [rcx + {Rbx}]",
        "    mov rbp, [rcx + {Rbp}]",
        "    mov rsi, [rcx + {Rsi}]",
        "    mov rdi, [rcx + {Rdi}]",
        "    mov r12, [rcx + {R12}]",
        "    mov r13, [rcx + {R13}]",
        "    mov r14, [rcx + {R14}]",
        "    mov r15, [rcx + {R15}]",
        "    movdqa xmm6, [rcx + {Xmm6}]",
        "    movdqa xmm7, [rcx + {Xmm7}]",
        "    movdqa xmm8, [rcx + {Xmm8}]",
        "    movdqa xmm9, [rcx + {Xmm9}]",
        "    movdqa xmm10, [rcx + {Xmm10}]",
        "    movdqa xmm11, [rcx + {Xmm11}]",
        "    movdqa xmm12, [rcx + {Xmm12}]",
        "    movdqa xmm13, [rcx + {Xmm13}]",
        "    movdqa xmm14, [rcx + {Xmm14}]",
        "    movdqa xmm15, [rcx + {Xmm15}]",
        "    mov r8, [rcx + {Rip}]",
        "    mov eax, 1",
        "    test edx, edx",
        "    cmovnz eax, edx",
        "    mov rsp, [rcx + {Rsp}]",
        "    jmp qword ptr [rcx + {Rip}]",
        Frame = const offset_of!(JumpBuffer, Frame),
        MxCsr = const offset_of!(JumpBuffer, MxCsr),
        FpCsr = const offset_of!(JumpBuffer, FpCsr),
        Rbx = const offset_of!(JumpBuffer, Rbx),
        Rbp = const offset_of!(JumpBuffer, Rbp),
        Rsi = const offset_of!(JumpBuffer, Rsi),
        Rdi = const offset_of!(JumpBuffer, Rdi),
        R12 = const offset_of!(JumpBuffer, R12),
        R13 = const offset_of!(JumpBuffer, R13),
        R14 = const offset_of!(JumpBuffer, R14),
        R15 = const offset_of!(JumpBuffer, R15),
        Rsp = const offset_of!(JumpBuffer, Rsp),
        Rip = const offset_of!(JumpBuffer, Rip),
        Xmm6 = const offset_of!(JumpBuffer, Xmm6),
        Xmm7 = const offset_of!(JumpBuffer, Xmm7),
        Xmm8 = const offset_of!(JumpBuffer, Xmm8),
        Xmm9 = const offset_of!(JumpBuffer, Xmm9),
        Xmm10 = const offset_of!(JumpBuffer, Xmm10),
        Xmm11 = const offset_of!(JumpBuffer, Xmm11),
        Xmm12 = const offset_of!(JumpBuffer, Xmm12),
        Xmm13 = const offset_of!(JumpBuffer, Xmm13),
        Xmm14 = const offset_of!(JumpBuffer, Xmm14),
        Xmm15 = const offset_of!(JumpBuffer, Xmm15),
    );

    const STATUS_LONGJUMP: i32 = 0x8000_0026u32 as i32;

    extern "C" {
        fn __longjmp_noframe(buf: *const JumpBuffer, value: i32) -> !;
    }

    /// `void __cdecl longjmp(jmp_buf _Buf, int _Value)` — faithful port of
    /// amd64/longjmp.c. If the buffer carries an SEH frame, unwind to it via
    /// RtlUnwind (a STATUS_LONGJUMP exception carrying the jmp_buf); otherwise
    /// restore directly. _Value is forced non-zero.
    #[no_mangle]
    pub unsafe extern "C" fn longjmp(buf: *mut JumpBuffer, value: i32) -> ! {
        let value = if value == 0 { 1 } else { value };
        let jb = buf;
        if (*jb).Frame != 0 {
            let mut er: EXCEPTION_RECORD = core::mem::zeroed();
            er.ExceptionCode = STATUS_LONGJUMP;
            er.NumberParameters = 1;
            er.ExceptionInformation[0] = jb as usize as _;
            RtlUnwind(
                (*jb).Frame as *mut _,
                (*jb).Rip as *mut _,
                &mut er,
                value as usize as *mut _,
            );
        }
        __longjmp_noframe(jb, value)
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
