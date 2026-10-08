#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]
//! setjmp-rust — the vcruntime `setjmp`/`longjmp` translated to Rust.
//!
//! Faithful translation of sdk/lib/vcruntime/{amd64,i386,arm64}/{_setjmp*.s,
//! __longjmp_*.s,longjmp.c}. Two consumers: the amd64 kernel's SEH unwind path
//! (pseh-rust/setjmp-rust's `kernel` build) and, since 2026-10-04, the vcruntime library and
//! the CRT DLLs' own Rust archives (all three arches) -- the CRT
//! DLLs no longer compile the .s/.c. `_seh_longjmp_unwind` (i386) is not here:
//! crt-shared-rust's except_i386.rs already provides it.
//!
//! The register save/restore is irreducible machine code, so it lives in
//! `global_asm!` (transcribed verbatim from the .s) — but every `_JUMP_BUFFER`
//! field offset is supplied as a `const offset_of!(...)` operand, so the asm
//! cannot drift from the struct layout. `longjmp` (the thin RtlUnwind glue) is
//! genuine Rust. Validated by disassembly-equivalence vs the cmake libsetjmp.a.

/// The two NT items `longjmp` needs, declared here rather than taken from `ntkrnl-sys`: the
/// CRT DLLs whole-archive this crate, and every object of a dependency comes with it, so
/// the kernel bindings dragged references to ObfDereferenceObject/ExFreePoolWithTag into
/// msvcrt.dll. `EXCEPTION_RECORD` is winnt.h's, the same layout on every arch (it is
/// `ntkrnl_sys::ddk::EXCEPTION_RECORD` field for field); `RtlUnwind` is exported by both
/// ntoskrnl and ntdll.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
mod nt {
    #[repr(C)]
    pub struct EXCEPTION_RECORD {
        pub ExceptionCode: i32,
        pub ExceptionFlags: u32,
        pub ExceptionRecord: *mut EXCEPTION_RECORD,
        pub ExceptionAddress: *mut core::ffi::c_void,
        pub NumberParameters: u32,
        pub ExceptionInformation: [usize; 15],
    }
    // winnt.h's win64 size; this copy is not generated, so pin it.
    const _: () = assert!(core::mem::size_of::<EXCEPTION_RECORD>() == 152);
    extern "system" {
        pub fn RtlUnwind(
            target_frame: *mut core::ffi::c_void,
            target_ip: *mut core::ffi::c_void,
            exception_record: *mut EXCEPTION_RECORD,
            return_value: *mut core::ffi::c_void,
        );
    }
}

#[cfg(target_arch = "x86_64")]
mod amd64 {
    use core::mem::offset_of;
    use crate::nt::{RtlUnwind, EXCEPTION_RECORD};

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

#[cfg(target_arch = "x86")]
mod i386 {
    use core::mem::offset_of;

    /// i386 `_JUMP_BUFFER` (sdk/include/vcruntime/setjmp.h).
    #[repr(C)]
    pub struct JumpBuffer {
        Ebp: u32,
        Ebx: u32,
        Edi: u32,
        Esi: u32,
        Esp: u32,
        Eip: u32,
        Registration: u32,
        TryLevel: u32,
        Cookie: u32,
        UnwindFunc: u32,
        UnwindData: [u32; 6],
    }

    /// `_JUMP_BUFFER_COOKIE` ("VC20"): marks a `_setjmp3` buffer.
    const JUMP_BUFFER_COOKIE: u32 = 0x5643_3230;

    // i386/_setjmp.s, _setjmp3.s, __longjmp_nounwind.s, verbatim; offsets from offset_of!.
    // `Esp` is the stack pointer AFTER _setjmp returns (esp + 4), as Windows stores it --
    // ReactOS's asm stored esp at the return address and had __longjmp_nounwind add the 4
    // back, consistent with itself but 4 below what ucrtbase_apitest setjmp.c:268/435
    // reads. Both copies moved together.
    // The symbol names carry the i386 cdecl underscore explicitly (global_asm is not
    // decorated): `_setjmp` is `__setjmp`, and so on.
    core::arch::global_asm!(
        ".intel_syntax noprefix",
        ".text",
        // int ___intrinsic_setjmp(_JUMP_BUFFER* Buf) / _setjmp
        ".globl ___intrinsic_setjmp",
        ".globl __setjmp",
        "___intrinsic_setjmp:",
        "__setjmp:",
        "    mov edx, [esp + 4]",
        "    mov [edx + {Ebp}], ebp",
        "    mov [edx + {Ebx}], ebx",
        "    mov [edx + {Esi}], esi",
        "    mov [edx + {Edi}], edi",
        "    mov eax, [esp]",
        "    mov [edx + {Eip}], eax",
        "    lea eax, [esp + 4]",
        "    mov [edx + {Esp}], eax",
        "    mov eax, fs:[0]",
        "    mov [edx + {Registration}], eax",
        "    cmp eax, 0xFFFFFFFF",
        "    je 1f",
        "    mov eax, [eax + 12]", // fs:[0]->TryLevel
        "1:",
        "    mov [edx + {TryLevel}], eax",
        "    xor eax, eax",
        "    ret",
        // int _setjmp3(_JUMP_BUFFER* Buf, unsigned Count, void* UnwindFunc, unsigned TryLevel, ...)
        ".globl __setjmp3",
        "__setjmp3:",
        "    mov edx, [esp + 4]",
        "    mov ecx, [esp + 8]",
        "    mov [edx + {Ebp}], ebp",
        "    mov [edx + {Ebx}], ebx",
        "    mov [edx + {Esi}], esi",
        "    mov [edx + {Edi}], edi",
        "    mov eax, [esp]",
        "    mov [edx + {Eip}], eax",
        "    lea eax, [esp + 4]",
        "    mov [edx + {Esp}], eax",
        "    mov dword ptr [edx + {Cookie}], {COOKIE}",
        "    xor eax, eax",
        "    mov [edx + {UnwindFunc}], eax",
        "    dec eax",
        "    mov [edx + {TryLevel}], eax",
        "    mov eax, fs:[0]",
        "    mov [edx + {Registration}], eax",
        "    cmp eax, 0xFFFFFFFF",
        "    je 3f",
        "    mov eax, [eax + 12]",
        "    mov [edx + {TryLevel}], eax",
        "    cmp ecx, 0",
        "    je 3f",
        "    mov eax, [esp + 12]",
        "    mov [edx + {UnwindFunc}], eax",
        "    dec ecx",
        "    jnz 2f",
        // "This is likely a bug in the MS implementation, we emulate it" (the C)
        "    mov eax, [eax + 12]",
        "    mov [edx + {TryLevel}], eax",
        "    jmp 3f",
        "2:",
        "    mov eax, [esp + 16]",
        "    mov [edx + {TryLevel}], eax",
        "    dec ecx",
        "    jz 3f",
        "    cmp ecx, 6",
        "    jbe 4f",
        "    mov ecx, 6",
        "4:",
        "    push esi",
        "    push edi",
        "    lea esi, [esp + 28]",
        "    lea edi, [edx + {UnwindData}]",
        "    rep movsd",
        "    pop edi",
        "    pop esi",
        "3:",
        "    xor eax, eax",
        "    ret",
        // __declspec(noreturn) void __longjmp_nounwind(const _JUMP_BUFFER*, int)
        ".globl ___longjmp_nounwind",
        "___longjmp_nounwind:",
        "    mov edx, [esp + 4]",
        "    mov eax, [esp + 8]",
        "    mov ebp, [edx + {Ebp}]",
        "    mov ebx, [edx + {Ebx}]",
        "    mov edi, [edx + {Edi}]",
        "    mov esi, [edx + {Esi}]",
        "    mov esp, [edx + {Esp}]",
        "    jmp dword ptr [edx + {Eip}]",
        Ebp = const offset_of!(JumpBuffer, Ebp),
        Ebx = const offset_of!(JumpBuffer, Ebx),
        Edi = const offset_of!(JumpBuffer, Edi),
        Esi = const offset_of!(JumpBuffer, Esi),
        Esp = const offset_of!(JumpBuffer, Esp),
        Eip = const offset_of!(JumpBuffer, Eip),
        Registration = const offset_of!(JumpBuffer, Registration),
        TryLevel = const offset_of!(JumpBuffer, TryLevel),
        Cookie = const offset_of!(JumpBuffer, Cookie),
        UnwindFunc = const offset_of!(JumpBuffer, UnwindFunc),
        UnwindData = const offset_of!(JumpBuffer, UnwindData),
        COOKIE = const JUMP_BUFFER_COOKIE,
    );

    extern "C" {
        fn _global_unwind2(registration: *mut core::ffi::c_void);
        fn __longjmp_nounwind(buf: *const JumpBuffer, value: i32) -> !;
    }
    extern "stdcall" {
        fn _seh_longjmp_unwind(buf: *const JumpBuffer);
    }

    /// `void __cdecl longjmp(jmp_buf, int)` -- i386/longjmp.c.
    #[no_mangle]
    pub unsafe extern "C" fn longjmp(buf: *mut JumpBuffer, value: i32) -> ! {
        let jb = buf as *const JumpBuffer;
        let value = if value == 0 { 1 } else { value };
        let fs0: u32;
        core::arch::asm!("mov {}, fs:[0]", out(reg) fs0, options(nostack, readonly, preserves_flags));
        // A registration from a previous function: global unwind to the one that owns it.
        if (*jb).Registration != fs0 {
            _global_unwind2((*jb).Registration as usize as *mut _);
        }
        if (*jb).Registration != 0 {
            if (*jb).Cookie == JUMP_BUFFER_COOKIE {
                // a _setjmp3 buffer: its unwind function, if any
                if (*jb).UnwindFunc != 0 {
                    let f: extern "stdcall" fn(*const JumpBuffer) =
                        core::mem::transmute((*jb).UnwindFunc as usize);
                    f(jb);
                }
            } else if (*jb).Registration != 0xFFFF_FFFF {
                // a legacy _setjmp buffer: old-style unwind
                _seh_longjmp_unwind(jb);
            }
        }
        __longjmp_nounwind(jb, value)
    }
}

#[cfg(target_arch = "aarch64")]
mod arm64 {
    use core::mem::offset_of;
    use crate::nt::{RtlUnwind, EXCEPTION_RECORD};

    /// arm64 `_JUMP_BUFFER` (sdk/include/vcruntime/setjmp.h).
    #[repr(C)]
    pub struct JumpBuffer {
        Frame: u64,
        Reserved: u64,
        X19: u64,
        X20: u64,
        X21: u64,
        X22: u64,
        X23: u64,
        X24: u64,
        X25: u64,
        X26: u64,
        X27: u64,
        X28: u64,
        Fp: u64,
        Lr: u64,
        Sp: u64,
        Fpcr: u32,
        Fpsr: u32,
        D: [f64; 8],
    }

    // arm64/_setjmp.s + __longjmp_noframe.s, verbatim. `_setjmp` stays WEAK, as in the
    // .s: msvcrt's MSVCRT__setjmp (except_arm64) overrides it where both are linked.
    core::arch::global_asm!(
        ".text",
        ".globl _setjmpex",
        ".globl __intrinsic_setjmpex",
        ".globl __intrinsic_setjmp",
        ".globl setjmp",
        ".globl __wine_setjmpex",
        ".weak _setjmp",
        ".p2align 2",
        "_setjmpex:",
        "__intrinsic_setjmpex:",
        "__intrinsic_setjmp:",
        "setjmp:",
        "__wine_setjmpex:",
        "_setjmp:",
        "    str x1, [x0, #{Frame}]",
        "    str xzr, [x0, #{Reserved}]",
        "    stp x19, x20, [x0, #{X19}]",
        "    stp x21, x22, [x0, #{X21}]",
        "    stp x23, x24, [x0, #{X23}]",
        "    stp x25, x26, [x0, #{X25}]",
        "    stp x27, x28, [x0, #{X27}]",
        "    str x29, [x0, #{Fp}]",
        "    str x30, [x0, #{Lr}]",
        "    mov x2, sp",
        "    str x2, [x0, #{Sp}]",
        "    mrs x2, fpcr",
        "    str w2, [x0, #{Fpcr}]",
        "    mrs x2, fpsr",
        "    str w2, [x0, #{Fpsr}]",
        "    stp d8, d9, [x0, #{D8}]",
        "    stp d10, d11, [x0, #{D10}]",
        "    stp d12, d13, [x0, #{D12}]",
        "    stp d14, d15, [x0, #{D14}]",
        "    mov x0, #0",
        "    ret",
        ".globl __longjmp_noframe",
        ".p2align 2",
        "__longjmp_noframe:",
        "    ldp d8, d9, [x0, #{D8}]",
        "    ldp d10, d11, [x0, #{D10}]",
        "    ldp d12, d13, [x0, #{D12}]",
        "    ldp d14, d15, [x0, #{D14}]",
        "    ldr w2, [x0, #{Fpcr}]",
        "    msr fpcr, x2",
        "    ldr w2, [x0, #{Fpsr}]",
        "    msr fpsr, x2",
        "    ldp x19, x20, [x0, #{X19}]",
        "    ldp x21, x22, [x0, #{X21}]",
        "    ldp x23, x24, [x0, #{X23}]",
        "    ldp x25, x26, [x0, #{X25}]",
        "    ldp x27, x28, [x0, #{X27}]",
        "    ldr x29, [x0, #{Fp}]",
        "    ldr x30, [x0, #{Lr}]",
        "    ldr x2, [x0, #{Sp}]",
        "    mov sp, x2",
        "    cmp x1, #0",
        "    csinc x0, x1, xzr, ne",
        "    ret",
        Frame = const offset_of!(JumpBuffer, Frame),
        Reserved = const offset_of!(JumpBuffer, Reserved),
        X19 = const offset_of!(JumpBuffer, X19),
        X21 = const offset_of!(JumpBuffer, X21),
        X23 = const offset_of!(JumpBuffer, X23),
        X25 = const offset_of!(JumpBuffer, X25),
        X27 = const offset_of!(JumpBuffer, X27),
        Fp = const offset_of!(JumpBuffer, Fp),
        Lr = const offset_of!(JumpBuffer, Lr),
        Sp = const offset_of!(JumpBuffer, Sp),
        Fpcr = const offset_of!(JumpBuffer, Fpcr),
        Fpsr = const offset_of!(JumpBuffer, Fpsr),
        D8 = const offset_of!(JumpBuffer, D),
        D10 = const offset_of!(JumpBuffer, D) + 16,
        D12 = const offset_of!(JumpBuffer, D) + 32,
        D14 = const offset_of!(JumpBuffer, D) + 48,
    );

    const STATUS_LONGJUMP: i32 = 0x8000_0026u32 as i32;

    extern "C" {
        fn __longjmp_noframe(buf: *const JumpBuffer, value: i32) -> !;
    }

    /// `void __cdecl longjmp(jmp_buf, int)` -- arm64/longjmp.c (the exception record is
    /// built unconditionally there, as here).
    #[no_mangle]
    pub unsafe extern "C" fn longjmp(buf: *mut JumpBuffer, value: i32) -> ! {
        let value = if value == 0 { 1 } else { value };
        let jb = buf;
        let mut er: EXCEPTION_RECORD = core::mem::zeroed();
        er.ExceptionCode = STATUS_LONGJUMP;
        er.NumberParameters = 1;
        er.ExceptionInformation[0] = jb as usize as _;
        if (*jb).Frame != 0 {
            RtlUnwind((*jb).Frame as *mut _, (*jb).Lr as *mut _, &mut er, value as usize as *mut _);
        }
        __longjmp_noframe(jb, value)
    }
}
