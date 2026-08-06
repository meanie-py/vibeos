//! except_x86_64.c — the amd64 arm of the arch gate `except.rs` sits on.
//!
//! `except.rs` is the shared 64-bit C++ exception dispatcher; the four functions here are
//! everything it cannot express portably — how to enter a handler on somebody else's frame,
//! and where the faulting PC lives in this architecture's `DISPATCHER_CONTEXT`. The i386
//! twin is `except_i386.rs`; the arm64 one is `except_arm64.rs`.
#![cfg(target_arch = "x86_64")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// wine's always-on channels. The whole message — the `fixme:seh:` prefix included — is a
// `c"…"` literal at the CALL site, and this body holds no literal of its own: the file is
// diffed literal-multiset against except_x86_64.c, and that diff strips channel CALLS from
// both sides but not a macro DEFINITION, so a prefix spelled here would read as a literal
// the C does not have.
//
// The trailing arguments are deliberately NOT evaluated, which is why the parameters they
// name read as unused. That is not laziness: wine's `FIXME` evaluates its arguments only when
// the channel is on, and one of them here is `*(ULONG64*)ep->ContextRecord->Rip` — a load
// through the FAULTING PC. Evaluating it unconditionally would take a second fault inside a
// filter that is about to return CONTINUE_SEARCH, turning a diagnostic into the crash.
macro_rules! FIXME {
    ($msg:literal $(, $($arg:tt)*)?) => {{
        OutputDebugStringA($msg.as_ptr());
    }};
}

// `__ASM_GLOBAL_FUNC( call_exc_handler, ... )` from the C, transcribed. It has no C text for
// rustconv to translate — it is a file-scope macro invocation — so it is hand-carried, and it
// cannot become a Rust `fn` in any case: it calls the handler with the SEH prologue's
// scratch slots laid out where the unwinder expects to find them.
//
// The expansion is fixed for this target and was read off `sdk/include/wine/asm.h`, not
// guessed: clang defines `__SEH__` for x86_64-w64-mingw32 and does NOT define
// `__GCC_HAVE_DWARF2_CFI_ASM`, so every `__ASM_CFI(...)` expands to nothing and every
// `__ASM_SEH(...)` expands to its argument. `__ASM_NAME` adds no underscore off i386.
// AT&T operands are pasted verbatim under `options(att_syntax)`, which is what makes the
// result checkable: `llvm-objdump -d` of this against the C's own .obj must agree byte for
// byte.
core::arch::global_asm!(
    ".text",
    ".align 4",
    ".globl call_exc_handler",
    ".def call_exc_handler; .scl 2; .type 32; .endef",
    ".seh_proc call_exc_handler",
    "call_exc_handler:",
    "subq $0x28,%rsp",
    ".seh_stackalloc 0x28",
    ".seh_endprologue",
    "movq %rcx, 0x0(%rsp)",
    "movl %r8d, 0x8(%rsp)",
    "movq %rdx, 0x10(%rsp)",
    "callq *%rcx",
    "addq $0x28,%rsp",
    "ret",
    ".seh_endproc",
    options(att_syntax)
);

unsafe extern "C" {
    fn call_exc_handler(handler: *mut c_void, frame: ULONG_PTR, flags: UINT) -> *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn call_catch_handler(rec: *mut EXCEPTION_RECORD) -> *mut core::ffi::c_void {
    let frame: ULONG_PTR = (*rec).ExceptionInformation[1];
    let handler: *mut core::ffi::c_void = (*rec).ExceptionInformation[5] as *mut core::ffi::c_void;

    return call_exc_handler(handler, frame, 0x100);
}

#[no_mangle]
pub unsafe extern "C" fn call_unwind_handler(
    handler: *mut core::ffi::c_void,
    frame: DWORD_PTR,
    dispatch: *mut DISPATCHER_CONTEXT,
) -> *mut core::ffi::c_void {
    return call_exc_handler(handler, frame, 0x100);
}

#[no_mangle]
pub unsafe extern "C" fn get_exception_pc(dispatch: *mut DISPATCHER_CONTEXT) -> ULONG_PTR {
    (*dispatch).ControlPc
}
#[no_mangle]
pub unsafe extern "C" fn _local_unwind(frame: *mut core::ffi::c_void, target: *mut core::ffi::c_void) {
    // The C's trailing `0` is the `PVOID ReturnValue` argument, not a count.
    RtlUnwind(frame, target, core::ptr::null_mut(), core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "C" fn handle_fpieee_flt(
    exception_code: __msvcrt_ulong,
    ep: *mut EXCEPTION_POINTERS,
    handler: Option<unsafe extern "C" fn(*mut _FPIEEE_RECORD) -> i32>,
) -> i32 {
    FIXME!(
        c"fixme:seh:(%lx %p %p) opcode: %#I64x\n",
        exception_code,
        ep,
        handler,
        *((*(*ep).ContextRecord).Rip as *const ULONG64)
    );
    EXCEPTION_CONTINUE_SEARCH as i32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt C++ exception handling
//  *
//  * Copyright 2011 Alexandre Julliard
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #if defined(__x86_64__) && !defined(__arm64ec__)
//
// #include <stdarg.h>
// #include <fpieee.h>
// #define longjmp ms_longjmp  /* avoid prototype mismatch */
// #include <setjmp.h>
// #undef longjmp
//
// #include "ntstatus.h"
// #define WIN32_NO_STATUS
// #include "windef.h"
// #include "winbase.h"
// #include "winternl.h"
// #include "msvcrt.h"
// #include "wine/exception.h"
// #include "excpt.h"
// #include "wine/debug.h"
//
// #include "cppexcept.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(seh);
//
// extern void *call_exc_handler( void *handler, ULONG_PTR frame, UINT flags );
// __ASM_GLOBAL_FUNC( call_exc_handler,
//                    "subq $0x28,%rsp\n\t"
//                    __ASM_CFI(".cfi_adjust_cfa_offset 0x28\n\t")
//                    __ASM_SEH(".seh_stackalloc 0x28\n\t")
//                    __ASM_SEH(".seh_endprologue\n\t")
//                    "movq %rcx, 0x0(%rsp)\n\t"
//                    "movl %r8d, 0x8(%rsp)\n\t"
//                    "movq %rdx, 0x10(%rsp)\n\t"
//                    "callq *%rcx\n\t"
//                    "addq $0x28,%rsp\n\t"
//                    __ASM_CFI(".cfi_adjust_cfa_offset -0x28\n\t")
//                    "ret" )
//
//
// /*******************************************************************
//  *		call_catch_handler
//  */
// void *call_catch_handler( EXCEPTION_RECORD *rec )
// {
//     ULONG_PTR frame = rec->ExceptionInformation[1];
//     void *handler = (void *)rec->ExceptionInformation[5];
//
//     TRACE( "calling %p frame %Ix\n", handler, frame );
//     return call_exc_handler( handler, frame, 0x100 );
// }
//
//
// /*******************************************************************
//  *		call_unwind_handler
//  */
// void *call_unwind_handler( void *handler, ULONG_PTR frame, DISPATCHER_CONTEXT *dispatch )
// {
//     TRACE( "calling %p frame %Ix\n", handler, frame );
//     return call_exc_handler( handler, frame, 0x100 );
// }
//
//
// /*******************************************************************
//  *		get_exception_pc
//  */
// ULONG_PTR get_exception_pc( DISPATCHER_CONTEXT *dispatch )
// {
//     return dispatch->ControlPc;
// }
//
//
// #ifndef __REACTOS__
// /*******************************************************************
//  *		longjmp (MSVCRT.@)
//  */
// #ifndef __WINE_PE_BUILD
// void __cdecl longjmp( _JUMP_BUFFER *jmp, int retval )
// {
//     EXCEPTION_RECORD rec;
//
//     if (!retval) retval = 1;
//     if (jmp->Frame)
//     {
//         rec.ExceptionCode = STATUS_LONGJUMP;
//         rec.ExceptionFlags = 0;
//         rec.ExceptionRecord = NULL;
//         rec.ExceptionAddress = NULL;
//         rec.NumberParameters = 1;
//         rec.ExceptionInformation[0] = (DWORD_PTR)jmp;
//         RtlUnwind( (void *)jmp->Frame, (void *)jmp->Rip, &rec, IntToPtr(retval) );
//     }
//     __wine_longjmp( (__wine_jmp_buf *)jmp, retval );
// }
// #endif
// #endif // __REACTOS__
//
// /*******************************************************************
//  *		_local_unwind (MSVCRT.@)
//  */
// void __cdecl _local_unwind( void *frame, void *target )
// {
//     RtlUnwind( frame, target, NULL, 0 );
// }
//
// /*********************************************************************
//  *              handle_fpieee_flt
//  */
// int handle_fpieee_flt( __msvcrt_ulong exception_code, EXCEPTION_POINTERS *ep,
//                        int (__cdecl *handler)(_FPIEEE_RECORD*) )
// {
//     FIXME("(%lx %p %p) opcode: %#I64x\n", exception_code, ep, handler,
//             *(ULONG64*)ep->ContextRecord->Rip);
//     return EXCEPTION_CONTINUE_SEARCH;
// }
//
//
// #endif  /* __x86_64__ */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
