//! except_arm64.c — the arm64 arm of the arch gate `except.rs` sits on.
//!
//! Same three jobs as `except_x86_64.rs`, and one more that is arm64's alone: the handler
//! runs with the throwing frame's CALLEE-SAVED registers restored, so `call_exc_handler`
//! takes a `nonvol_regs` block and reloads x19–x28/x29 from it before branching. The i386
//! twin is `except_i386.rs`.
#![cfg(target_arch = "aarch64")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// See except_x86_64.rs for why the whole message including wine's channel prefix is a
// `c"…"` literal at the call site and this body carries no literal of its own.

// The C's two `__ASM_GLOBAL_FUNC`s, transcribed — neither has C text for rustconv to
// translate. clang defines `__SEH__` for aarch64-w64-mingw32 and not
// `__GCC_HAVE_DWARF2_CFI_ASM`, so the `.seh_*` unwind directives are live and every
// `__ASM_CFI(...)` is empty; `__ASM_NAME` adds no underscore. Those `.seh_save_*` are not
// decoration — they ARE the unwind info for a frame the OS must walk while an exception is
// already in flight, so dropping one produces a correct-looking function that hangs the
// unwinder.
core::arch::global_asm!(
    ".text",
    ".align 4",
    ".globl call_exc_handler",
    ".def call_exc_handler; .scl 2; .type 32; .endef",
    ".seh_proc call_exc_handler",
    "call_exc_handler:",
    "stp x29, x30, [sp, #-96]!",
    ".seh_save_fplr_x 96",
    "stp x19, x20, [sp, #16]",
    ".seh_save_regp x19, 16",
    "stp x21, x22, [sp, #32]",
    ".seh_save_regp x21, 32",
    "stp x23, x24, [sp, #48]",
    ".seh_save_regp x23, 48",
    "stp x25, x26, [sp, #64]",
    ".seh_save_regp x25, 64",
    "stp x27, x28, [sp, #80]",
    ".seh_save_regp x27, 80",
    "str x1, [sp, #-16]!",
    ".seh_stackalloc 16",
    ".seh_endprologue",
    "ldp x19, x20, [x3, #0]", // nonvolatile regs
    "ldp x21, x22, [x3, #16]",
    "ldp x23, x24, [x3, #32]",
    "ldp x25, x26, [x3, #48]",
    "ldp x27, x28, [x3, #64]",
    "ldr x29, [x3, #80]",
    "blr x0",
    "add sp, sp, 16",
    "ldp x19, x20, [sp, #16]",
    "ldp x21, x22, [sp, #32]",
    "ldp x23, x24, [sp, #48]",
    "ldp x25, x26, [sp, #64]",
    "ldp x27, x28, [sp, #80]",
    "ldp x29, x30, [sp], #96",
    "ret",
    ".seh_endproc",
);

// `_setjmp (MSVCRT.@)` is a tail-branch alias onto `_setjmpex` — one instruction, and it must
// be a branch rather than a call so the returned-to frame is the CALLER's.
core::arch::global_asm!(
    ".text",
    ".align 4",
    ".globl _setjmp",
    ".def _setjmp; .scl 2; .type 32; .endef",
    ".seh_proc _setjmp",
    "_setjmp:",
    "b _setjmpex",
    ".seh_endproc",
);

unsafe extern "C" {
    fn call_exc_handler(
        handler: *mut c_void,
        frame: ULONG_PTR,
        flags: UINT,
        nonvol_regs: *mut BYTE,
    ) -> *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn call_catch_handler(rec: *mut EXCEPTION_RECORD) -> *mut core::ffi::c_void {
    let frame: ULONG_PTR = (*rec).ExceptionInformation[1];
    let handler: *mut core::ffi::c_void = (*rec).ExceptionInformation[5] as *mut core::ffi::c_void;
    let nonvol_regs: *mut BYTE = (*rec).ExceptionInformation[10] as *mut BYTE;

    return call_exc_handler(handler, frame, 0x100, nonvol_regs);
}

#[no_mangle]
pub unsafe extern "C" fn call_unwind_handler(
    handler: *mut core::ffi::c_void,
    frame: ULONG_PTR,
    dispatch: *mut DISPATCHER_CONTEXT,
) -> *mut core::ffi::c_void {
    call_exc_handler(
        handler,
        frame,
        0x100,
        (*dispatch).NonVolatileRegisters,
    )
}
/// msvcrt.spec exports `_local_unwind` for win64, and except_arm64.c never defined it, so on
/// arm64 the name fell through to kernel32's import library -- a forwarder to
/// `ntdll._local_unwind`, which ntdll exports for x86_64 only. The export was a dangling
/// forward chain. Same body as except_x86_64's (Wine's `_local_unwind`); arm64's ntdll has
/// RtlUnwind. Not under `ucrtsupport`: ucrtbase.spec forwards the name (the same dangling
/// forward, on the roadmap) and a body here would be an undeclared ucrt takeover.
#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn _local_unwind(frame: *mut c_void, target: *mut c_void) {
    RtlUnwind(frame, target, core::ptr::null_mut(), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn get_exception_pc(dispatch: *mut DISPATCHER_CONTEXT) -> ULONG_PTR {
    let mut pc: ULONG_PTR = (*dispatch).ControlPc;
    if (*dispatch).ControlPcIsUnwound != 0 {
        pc = pc.wrapping_sub(4);
    }
    pc
}
#[no_mangle]
pub unsafe extern "C" fn handle_fpieee_flt(
    exception_code: __msvcrt_ulong,
    ep: *mut EXCEPTION_POINTERS,
    handler: Option<unsafe extern "C" fn(*mut _FPIEEE_RECORD) -> i32>
) -> i32 {
    FIXME!("seh:(%lx %p %p)\n", exception_code, ep, handler);
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
//  * Copyright 2013 André Hentschel
//  * Copyright 2017 Martin Storsjo
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
// #ifdef __aarch64__
//
// #include <setjmp.h>
// #include <stdarg.h>
// #include <fpieee.h>
//
// #include "ntstatus.h"
// #define WIN32_NO_STATUS
// #include "windef.h"
// #include "winbase.h"
// #include "winternl.h"
// #include "msvcrt.h"
// #include "excpt.h"
// #include "wine/debug.h"
//
// #include "cppexcept.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(seh);
//
//
// extern void *call_exc_handler( void *handler, ULONG_PTR frame, UINT flags, BYTE *nonvol_regs );
// __ASM_GLOBAL_FUNC( call_exc_handler,
//                    "stp x29, x30, [sp, #-96]!\n\t"
//                    ".seh_save_fplr_x 96\n\t"
//                    "stp x19, x20, [sp, #16]\n\t"
//                    ".seh_save_regp x19, 16\n\t"
//                    "stp x21, x22, [sp, #32]\n\t"
//                    ".seh_save_regp x21, 32\n\t"
//                    "stp x23, x24, [sp, #48]\n\t"
//                    ".seh_save_regp x23, 48\n\t"
//                    "stp x25, x26, [sp, #64]\n\t"
//                    ".seh_save_regp x25, 64\n\t"
//                    "stp x27, x28, [sp, #80]\n\t"
//                    ".seh_save_regp x27, 80\n\t"
//                    "str x1, [sp, #-16]!\n\t"
//                    ".seh_stackalloc 16\n\t"
//                    ".seh_endprologue\n\t"
//                    "ldp x19, x20, [x3, #0]\n\t" /* nonvolatile regs */
//                    "ldp x21, x22, [x3, #16]\n\t"
//                    "ldp x23, x24, [x3, #32]\n\t"
//                    "ldp x25, x26, [x3, #48]\n\t"
//                    "ldp x27, x28, [x3, #64]\n\t"
//                    "ldr x29, [x3, #80]\n\t"
//                    "blr x0\n\t"
//                    "add sp, sp, 16\n\t"
//                    "ldp x19, x20, [sp, #16]\n\t"
//                    "ldp x21, x22, [sp, #32]\n\t"
//                    "ldp x23, x24, [sp, #48]\n\t"
//                    "ldp x25, x26, [sp, #64]\n\t"
//                    "ldp x27, x28, [sp, #80]\n\t"
//                    "ldp x29, x30, [sp], #96\n\t"
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
//     BYTE *nonvol_regs = (BYTE *)rec->ExceptionInformation[10];
//
//     TRACE( "calling %p frame %Ix\n", handler, frame );
//     return call_exc_handler( handler, frame, 0x100, nonvol_regs );
// }
//
//
// /*******************************************************************
//  *		call_unwind_handler
//  */
// void *call_unwind_handler( void *handler, ULONG_PTR frame, DISPATCHER_CONTEXT *dispatch )
// {
//     TRACE( "calling %p frame %Ix\n", handler, frame );
//     return call_exc_handler( handler, frame, 0x100, dispatch->NonVolatileRegisters );
// }
//
//
// /*******************************************************************
//  *		get_exception_pc
//  */
// ULONG_PTR get_exception_pc( DISPATCHER_CONTEXT *dispatch )
// {
//     ULONG_PTR pc = dispatch->ControlPc;
//     if (dispatch->ControlPcIsUnwound) pc -= 4;
//     return pc;
// }
//
//
// /*******************************************************************
//  *		_setjmp (MSVCRT.@)
//  */
// __ASM_GLOBAL_FUNC( _setjmp, "b _setjmpex" );
//
//
// /*********************************************************************
//  *              handle_fpieee_flt
//  */
// int handle_fpieee_flt( __msvcrt_ulong exception_code, EXCEPTION_POINTERS *ep,
//                        int (__cdecl *handler)(_FPIEEE_RECORD*) )
// {
//     FIXME("(%lx %p %p)\n", exception_code, ep, handler);
//     return EXCEPTION_CONTINUE_SEARCH;
// }
//
// #endif  /* __aarch64__ */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
