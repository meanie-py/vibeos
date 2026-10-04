//! dispcall.rs — DispCallFunc (OLEAUT32.@) + the per-arch call_method /
//! call_double_method / call_float_method trampolines (typelib.c lines
//! ~6405–7079). Lane A2 continuation.
//!
//! All three project targets are implemented with `global_asm!`, each a
//! faithful transcription of the Wine `__ASM_GLOBAL_FUNC` bodies:
//!   * i386   — stack push-loop trampoline (stdcall/cdecl agnostic; reports
//!              the callee stack delta so DispCallFunc can detect a broken
//!              stdcall callee), AT&T syntax kept verbatim.
//!   * x86_64 — Win64 register marshaling (first 4 slots mirrored into both
//!              GPRs and XMMs, rest on the stack).
//!   * aarch64— AAPCS64 marshaling (d0-d7 + x0-x8 loaded from a reg block,
//!              rest on the stack).
//! The ARM(32) flavor is not a project target and is not carried.
//!
//! DEVIATION (documented in MANIFEST): the CFI/SEH assembler annotations of
//! the C (__ASM_CFI/__ASM_SEH) are dropped — they only affect unwind
//! metadata, not execution. Everything else is verbatim.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_variables, unused_mut, unused_unsafe, unused_imports)]

use core::arch::global_asm;
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr::null_mut;

use super::super::olesys::*;
use super::data::*;
use super::data::{ERR, FIXME, TRACE, WARN};
use super::sys::*;

pub const DISP_E_BADCALLEE: HRESULT = 0x8002_0010u32 as HRESULT;

// ===========================================================================
// i386
// ===========================================================================

#[cfg(target_arch = "x86")]
global_asm!(
    r#"
    .text
    .align 4
    .globl _call_method
_call_method:
    pushl %ebp
    movl %esp,%ebp
    pushl %esi
    pushl %edi
    movl 12(%ebp),%edx
    movl %esp,%edi
    shll $2,%edx
    jz 1f
    subl %edx,%edi
    andl $-16,%edi
    movl %edi,%esp
    movl 12(%ebp),%ecx
    movl 16(%ebp),%esi
    cld
    rep; movsl
1:  call *8(%ebp)
    subl %esp,%edi
    movl 20(%ebp),%ecx
    movl %edi,(%ecx)
    leal -8(%ebp),%esp
    popl %edi
    popl %esi
    popl %ebp
    ret
    .globl _call_double_method
_call_double_method:
    jmp _call_method
"#,
    options(att_syntax)
);

#[cfg(target_arch = "x86")]
extern "C" {
    /// LONGLONG call_method(void *func, int nb_args, const DWORD *args, int *stack_offset)
    pub fn call_method(func: *mut c_void, nb_args: i32, args: *const u32, stack_offset: *mut i32) -> i64;
    /// double call_double_method(void *func, int nb_args, const DWORD *args, int *stack_offset)
    pub fn call_double_method(func: *mut c_void, nb_args: i32, args: *const u32, stack_offset: *mut i32) -> f64;
}

/*******************************************************************************
 *		DispCallFunc (OLEAUT32.146) — i386 flavor
 */
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn DispCallFunc(
    pvInstance: *mut c_void,
    oVft: ULONG_PTR,
    cc: CALLCONV,
    vtReturn: VARTYPE,
    cActuals: UINT,
    prgvt: *mut VARTYPE,
    prgpvarg: *mut *mut VARIANTARG,
    pvargResult: *mut VARIANT,
) -> HRESULT {
    let mut argspos: i32 = 0;
    let mut stack_offset: i32 = 0;
    let func: *mut c_void;

    TRACE!();

    if cc != CC_STDCALL && cc != CC_CDECL {
        FIXME!("unsupported calling convention {}", cc);
        return E_INVALIDARG;
    }

    /* maximum size for an argument is sizeof(VARIANT) */
    let args = heap_alloc(size_of::<VARIANT>() * cActuals as usize + size_of::<DWORD>() * 2)
        as *mut u32;

    if !pvInstance.is_null() {
        let vtable = *(pvInstance as *const *const *mut c_void);
        func = *vtable.add(oVft / size_of::<*mut c_void>());
        *args.add(argspos as usize) = pvInstance as u32; /* the This pointer is always the first parameter */
        argspos += 1;
    } else {
        func = oVft as *mut c_void;
    }

    match vtReturn {
        VT_DECIMAL | VT_VARIANT => {
            *args.add(argspos as usize) = pvargResult as u32; /* arg 0 is a pointer to the result */
            argspos += 1;
        }
        VT_HRESULT => {
            WARN!();
            heap_free(args as *mut c_void);
            return E_INVALIDARG;
        }
        _ => {}
    }

    let mut i: UINT = 0;
    while i < cActuals {
        let arg = *prgpvarg.add(i as usize);

        match *prgvt.add(i as usize) {
            VT_EMPTY => {}
            VT_I8 | VT_UI8 | VT_R8 | VT_DATE | VT_CY => {
                memcpy(
                    args.add(argspos as usize) as *mut c_void,
                    &raw const (*arg).n3.llVal as *const c_void,
                    size_of::<i64>(),
                );
                argspos += (size_of::<i64>() / size_of::<DWORD>()) as i32;
            }
            VT_DECIMAL | VT_VARIANT => {
                memcpy(
                    args.add(argspos as usize) as *mut c_void,
                    arg as *const c_void,
                    size_of::<VARIANT>(),
                );
                argspos += (size_of::<VARIANT>() / size_of::<DWORD>()) as i32;
            }
            VT_BOOL => {
                /* VT_BOOL is 16-bit but BOOL is 32-bit, needs to be extended */
                *args.add(argspos as usize) = (*arg).n3.boolVal as i32 as u32;
                argspos += 1;
            }
            _ => {
                *args.add(argspos as usize) = (*arg).n3.ulVal;
                argspos += 1;
            }
        }
        TRACE!();
        i += 1;
    }

    match vtReturn {
        VT_EMPTY | VT_DECIMAL | VT_VARIANT => {
            call_method(func, argspos, args, &raw mut stack_offset);
        }
        VT_R4 => {
            (*pvargResult).n3.fltVal =
                call_double_method(func, argspos, args, &raw mut stack_offset) as f32;
        }
        VT_R8 | VT_DATE => {
            (*pvargResult).n3.dblVal =
                call_double_method(func, argspos, args, &raw mut stack_offset);
        }
        VT_I8 | VT_UI8 | VT_CY => {
            (*pvargResult).n3.ullVal =
                call_method(func, argspos, args, &raw mut stack_offset) as u64;
        }
        _ => {
            (*pvargResult).n3.ulVal =
                call_method(func, argspos, args, &raw mut stack_offset) as u32;
        }
    }
    heap_free(args as *mut c_void);
    if stack_offset != 0 && cc == CC_STDCALL {
        WARN!();
        return DISP_E_BADCALLEE;
    }
    if vtReturn != VT_VARIANT {
        (*pvargResult).vt = vtReturn;
    }
    TRACE!();
    S_OK
}

// ===========================================================================
// x86_64
// ===========================================================================

#[cfg(target_arch = "x86_64")]
global_asm!(
    r#"
    .text
    .align 4
    .globl call_method
call_method:
    pushq %rbp
    movq %rsp,%rbp
    pushq %rsi
    pushq %rdi
    movq %rcx,%rax
    movq $4,%rcx
    cmp %rcx,%rdx
    cmovgq %rdx,%rcx
    leaq 0(,%rcx,8),%rdx
    subq %rdx,%rsp
    andq $-16,%rsp
    movq %rsp,%rdi
    movq %r8,%rsi
    rep; movsq
    movq 0(%rsp),%rcx
    movq 8(%rsp),%rdx
    movq 16(%rsp),%r8
    movq 24(%rsp),%r9
    movq 0(%rsp),%xmm0
    movq 8(%rsp),%xmm1
    movq 16(%rsp),%xmm2
    movq 24(%rsp),%xmm3
    callq *%rax
    leaq -16(%rbp),%rsp
    popq %rdi
    popq %rsi
    popq %rbp
    ret
    .globl call_double_method
call_double_method:
    jmp call_method
    .globl call_float_method
call_float_method:
    jmp call_method
"#,
    options(att_syntax)
);

#[cfg(target_arch = "x86_64")]
extern "C" {
    /// DWORD_PTR call_method(void *func, int nb_args, const DWORD_PTR *args)
    pub fn call_method(func: *mut c_void, nb_args: i32, args: *const usize) -> usize;
    /// double call_double_method(void *func, int nb_args, const DWORD_PTR *args)
    pub fn call_double_method(func: *mut c_void, nb_args: i32, args: *const usize) -> f64;
    /// float call_float_method(void *func, int nb_args, const DWORD_PTR *args): the same
    /// trampoline, typed so the caller reads xmm0 as the float the callee left there.
    pub fn call_float_method(func: *mut c_void, nb_args: i32, args: *const usize) -> f32;
}

/*******************************************************************************
 *		DispCallFunc (OLEAUT32.146) — x86_64 flavor
 */
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn DispCallFunc(
    pvInstance: *mut c_void,
    oVft: ULONG_PTR,
    cc: CALLCONV,
    vtReturn: VARTYPE,
    cActuals: UINT,
    prgvt: *mut VARTYPE,
    prgpvarg: *mut *mut VARIANTARG,
    pvargResult: *mut VARIANT,
) -> HRESULT {
    let mut argspos: i32 = 0;
    let func: *mut c_void;

    TRACE!();

    if cc != CC_STDCALL && cc != CC_CDECL {
        FIXME!("unsupported calling convention {}", cc);
        return E_INVALIDARG;
    }

    /* maximum size for an argument is sizeof(DWORD_PTR) */
    let args = heap_alloc(size_of::<DWORD_PTR>() * (cActuals as usize + 2)) as *mut usize;

    if !pvInstance.is_null() {
        let vtable = *(pvInstance as *const *const *mut c_void);
        func = *vtable.add(oVft / size_of::<*mut c_void>());
        *args.add(argspos as usize) = pvInstance as usize; /* the This pointer is always the first parameter */
        argspos += 1;
    } else {
        func = oVft as *mut c_void;
    }

    match vtReturn {
        VT_DECIMAL | VT_VARIANT => {
            *args.add(argspos as usize) = pvargResult as usize; /* arg 0 is a pointer to the result */
            argspos += 1;
        }
        VT_HRESULT => {
            WARN!();
            heap_free(args as *mut c_void);
            return E_INVALIDARG;
        }
        _ => {}
    }

    let mut i: UINT = 0;
    while i < cActuals {
        let arg = *prgpvarg.add(i as usize);

        match *prgvt.add(i as usize) {
            VT_DECIMAL | VT_VARIANT => {
                *args.add(argspos as usize) = arg as usize;
                argspos += 1;
            }
            VT_BOOL => {
                /* VT_BOOL is 16-bit but BOOL is 32-bit, needs to be extended */
                *args.add(argspos as usize) = (*arg).n3.boolVal as isize as usize;
                argspos += 1;
            }
            _ => {
                *args.add(argspos as usize) = (*arg).n3.ullVal as usize;
                argspos += 1;
            }
        }
        TRACE!();
        i += 1;
    }

    match vtReturn {
        VT_R4 => {
            // Divergence from typelib.c, which reads VT_R4 through call_double_method here too:
            // a float return occupies only xmm0's low 32 bits, so reading it as a double
            // reinterprets those bits as a denormal and narrowing gives 0.0. (i386 is immune,
            // since x87 st(0) holds either type; arm64's C already has call_float_method.)
            // Found by tools/payloads/dispprobe.
            (*pvargResult).n3.fltVal = call_float_method(func, argspos, args);
        }
        VT_R8 | VT_DATE => {
            (*pvargResult).n3.dblVal = call_double_method(func, argspos, args);
        }
        VT_DECIMAL | VT_VARIANT => {
            call_method(func, argspos, args);
        }
        _ => {
            (*pvargResult).n3.ullVal = call_method(func, argspos, args) as u64;
        }
    }
    heap_free(args as *mut c_void);
    if vtReturn != VT_VARIANT {
        (*pvargResult).vt = vtReturn;
    }
    TRACE!();
    S_OK
}

// ===========================================================================
// aarch64
// ===========================================================================

#[cfg(target_arch = "aarch64")]
global_asm!(
    r#"
    .text
    .align 4
    .globl call_method
call_method:
    stp x29, x30, [sp, #-16]!
    mov x29, sp
    sub sp, sp, x1, lsl #3
    cbz x1, 2f
1:  sub x1, x1, #1
    ldr x4, [x2, x1, lsl #3]
    str x4, [sp, x1, lsl #3]
    cbnz x1, 1b
2:  mov x16, x0
    mov x9, x3
    ldp d0, d1, [x9]
    ldp d2, d3, [x9, #0x10]
    ldp d4, d5, [x9, #0x20]
    ldp d6, d7, [x9, #0x30]
    ldp x0, x1, [x9, #0x40]
    ldp x2, x3, [x9, #0x50]
    ldp x4, x5, [x9, #0x60]
    ldp x6, x7, [x9, #0x70]
    ldr x8, [x9, #0x80]
    blr x16
    mov sp, x29
    ldp x29, x30, [sp], #16
    ret
    .globl call_float_method
call_float_method:
    b call_method
    .globl call_double_method
call_double_method:
    b call_method
"#
);

#[cfg(target_arch = "aarch64")]
extern "C" {
    /// DWORD_PTR call_method(void *func, int nb_stk_args, const DWORD_PTR *stk_args, const DWORD_PTR *reg_args)
    pub fn call_method(func: *mut c_void, nb_stk_args: i32, stk_args: *const usize, reg_args: *const usize) -> usize;
    pub fn call_float_method(func: *mut c_void, nb_stk_args: i32, stk_args: *const usize, reg_args: *const usize) -> f32;
    pub fn call_double_method(func: *mut c_void, nb_stk_args: i32, stk_args: *const usize, reg_args: *const usize) -> f64;
}

/// register block passed to the aarch64 call_method (layout is load-bearing:
/// 8 × 8-byte fp regs, then x0..x8)
#[cfg(target_arch = "aarch64")]
#[repr(C)]
union FpReg {
    f: f32,
    d: f64,
}
#[cfg(target_arch = "aarch64")]
#[repr(C)]
struct Aarch64Regs {
    fp: [FpReg; 8],
    x: [DWORD_PTR; 9],
}

/*******************************************************************************
 *		DispCallFunc (OLEAUT32.146) — aarch64 flavor
 */
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "system" fn DispCallFunc(
    instance: *mut c_void,
    offset: ULONG_PTR,
    cc: CALLCONV,
    ret_type: VARTYPE,
    count: UINT,
    types: *mut VARTYPE,
    vargs: *mut *mut VARIANTARG,
    result: *mut VARIANT,
) -> HRESULT {
    let mut argspos: i32 = 0;
    let func: *mut c_void;
    let mut regs: Aarch64Regs = core::mem::zeroed();
    let mut rcount: i32 = 0; /* 64-bit register index count */
    let mut fpcount: i32 = 0; /* float register index count */

    TRACE!();

    if cc != CC_STDCALL && cc != CC_CDECL {
        FIXME!("unsupported calling convention {}", cc);
        return E_INVALIDARG;
    }

    if !instance.is_null() {
        let vtable = *(instance as *const *const *mut c_void);
        func = *vtable.add(offset / size_of::<*mut c_void>());
        regs.x[rcount as usize] = instance as usize; /* the This pointer is always the first parameter */
        rcount += 1;
    } else {
        func = offset as *mut c_void;
    }

    /* Determine if we need to pass a pointer for the return value as arg 0.
     * If so, do that first as it will need to be in the 'x' registers: */
    match ret_type {
        VT_DECIMAL | VT_VARIANT => {
            regs.x[8] = result as usize; /* x8 is a pointer to the result */
        }
        VT_HRESULT => {
            WARN!();
            return E_INVALIDARG;
        }
        _ => {}
    }

    /* maximum size for an argument is sizeof(VARIANT). Also allow for return
     * pointer and stack alignment. */
    let args = heap_alloc(size_of::<VARIANT>() * count as usize + size_of::<DWORD_PTR>() * 4)
        as *mut usize;

    let mut i: UINT = 0;
    while i < count {
        let arg = *vargs.add(i as usize);

        // Wine 10.0: no VT_EMPTY case, an EMPTY argument takes a slot like
        // any other integer argument.
        match *types.add(i as usize) {
            VT_R4 => {
                if fpcount < 8 {
                    regs.fp[fpcount as usize].f = (*arg).n3.fltVal;
                    fpcount += 1;
                } else {
                    *(args.add(argspos as usize) as *mut f32) = (*arg).n3.fltVal;
                    argspos += 1;
                }
            }
            VT_R8 | VT_DATE => {
                if fpcount < 8 {
                    regs.fp[fpcount as usize].d = (*arg).n3.dblVal;
                    fpcount += 1;
                } else {
                    *(args.add(argspos as usize) as *mut f64) = (*arg).n3.dblVal;
                    argspos += 1;
                }
            }
            // A DECIMAL (16 bytes) travels by value in two slots. Wine copies
            // sizeof(VARIANT) = 24 bytes there, one slot more than it reserves,
            // which can land in x8 (the result pointer); copy the 16 it uses.
            VT_DECIMAL => {
                if rcount < 7 {
                    memcpy(
                        (&raw mut regs.x as *mut usize).add(rcount as usize) as *mut c_void,
                        arg as *const c_void,
                        size_of::<DECIMAL>(),
                    );
                    rcount += 2;
                } else {
                    memcpy(
                        args.add(argspos as usize) as *mut c_void,
                        arg as *const c_void,
                        size_of::<DECIMAL>(),
                    );
                    argspos += 2;
                }
            }
            // Wine 10.0: a VARIANT (24 bytes, larger than 16) is passed by
            // REFERENCE under AAPCS64, one slot holding its address. The
            // pre-10 code copied it by value into two slots.
            VT_VARIANT => {
                if rcount < 8 {
                    regs.x[rcount as usize] = arg as usize;
                    rcount += 1;
                } else {
                    *args.add(argspos as usize) = arg as usize;
                    argspos += 1;
                }
            }
            VT_BOOL => {
                /* VT_BOOL is 16-bit but BOOL is 32-bit, needs to be extended */
                if rcount < 8 {
                    regs.x[rcount as usize] = (*arg).n3.boolVal as isize as usize;
                    rcount += 1;
                } else {
                    *args.add(argspos as usize) = (*arg).n3.boolVal as isize as usize;
                    argspos += 1;
                }
            }
            _ => {
                if rcount < 8 {
                    regs.x[rcount as usize] = (*arg).n3.ullVal as usize;
                    rcount += 1;
                } else {
                    *args.add(argspos as usize) = (*arg).n3.ullVal as usize;
                    argspos += 1;
                }
            }
        }
        TRACE!();
        i += 1;
    }

    argspos += argspos % 2; /* Make sure stack function alignment is 16-byte */

    match ret_type {
        VT_EMPTY      /* EMPTY = no return value */
        | VT_DECIMAL  /* DECIMAL and VARIANT already have a pointer argument passed (see above) */
        | VT_VARIANT => {
            call_method(func, argspos, args, &raw const regs as *const usize);
        }
        VT_R4 => {
            (*result).n3.fltVal =
                call_float_method(func, argspos, args, &raw const regs as *const usize);
        }
        VT_R8 | VT_DATE => {
            (*result).n3.dblVal =
                call_double_method(func, argspos, args, &raw const regs as *const usize);
        }
        _ => {
            (*result).n3.ullVal =
                call_method(func, argspos, args, &raw const regs as *const usize) as u64;
        }
    }
    heap_free(args as *mut c_void);
    if ret_type != VT_VARIANT {
        (*result).vt = ret_type;
    }
    TRACE!();
    S_OK
}
