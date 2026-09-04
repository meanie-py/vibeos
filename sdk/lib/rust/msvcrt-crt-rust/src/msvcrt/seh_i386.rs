//! except_i386.c's substrate — the six asm thunks and the two frame-list inlines.
//!
//! Nothing here is a translation decision. It is the part of except_i386.c that has no C
//! *text* to translate:
//!
//!   * six `__ASM_GLOBAL_FUNC` blocks — irreducible machine code. rustconv's scanner sees
//!     functions, and these are macro invocations at file scope, so they arrive in the
//!     translation only as commented-out C. They are transcribed here VERBATIM, in AT&T
//!     syntax (`options(att_syntax)`) rather than converted to Intel, precisely so that
//!     "transcribed" means byte-for-byte and not "re-derived by hand". The `__ASM_CFI(...)`
//!     directives in the C expand to nothing on a PE target (`__GCC_HAVE_DWARF2_CFI_ASM` is
//!     not defined there), so their absence here is the same object, not a dropped line.
//!   * `__wine_push_frame` / `__wine_pop_frame` — `static inline` in wine/exception.h, so
//!     they define no symbol anywhere and no object file mentions them.
//!
//! Why these four thunks exist at all, since a Rust `fn` could not replace them: each one
//! runs on a frame that BELONGS TO SOMEONE ELSE. `call_handler` sets `%ebp` to the throwing
//! function's frame pointer before calling its `__finally` block, because that block reads
//! the throwing function's locals through `%ebp`. `continue_after_catch` restores `%esp`
//! from `-4(%edx)` and jumps — it never returns to its caller. That is the whole 32-bit SEH
//! contract, and it is not expressible in any language with a normal prologue.

#![cfg(target_arch = "x86")]

use msvcrt_sys::{EXCEPTION_REGISTRATION_RECORD, PEXCEPTION_POINTERS, _TEB};

// The six `__ASM_GLOBAL_FUNC` bodies from dll/win32/msvcrt/except_i386.c, in source order.
//
// The INSTRUCTIONS appear once, inside this macro. Only the symbol DECORATION is passed in,
// because that is the only thing an object-format change may move: i386 PE decorates a cdecl
// symbol with a leading underscore (`__ASM_NAME`) and types it with the COFF
// `.def/.scl 2/.type 32/.endef` triple (`__ASM_FUNC_TYPE`) — drop that and the linker sees
// data where a function should be — while i386 ELF decorates nothing and types with
// `.type name,@function`. Writing the two out separately would mean two transcriptions of
// irreducible machine code that must stay byte-identical forever, which is the drift class
// this repo keeps paying for; one macro makes the identity structural instead of a promise.
//
// The ELF arm exists for `msvcrt-seh-difftest`, which links this module into a 32-bit Linux
// test binary beside the genuine except_i386.c. It is not reachable from any product build.
macro_rules! seh_thunks {
    (
        continue_after_catch = ($cac_g:literal, $cac_d:literal, $cac_l:literal),
        call_finally_block   = ($cfb_g:literal, $cfb_d:literal, $cfb_l:literal),
        call_filter          = ($cfi_g:literal, $cfi_d:literal, $cfi_l:literal),
        call_handler         = ($chd_g:literal, $chd_d:literal, $chd_l:literal),
        CxxFrameHandler      = ($cfh_g:literal, $cfh_d:literal, $cfh_l:literal, $cfh_call:literal),
        EH_prolog            = ($ehp_g:literal, $ehp_d:literal, $ehp_l:literal),
    ) => {
        core::arch::global_asm!(
            ".text",

            // continue_after_catch( cxx_exception_frame *frame, void *addr ) — DECLSPEC_NORETURN.
            ".align 4", $cac_g, $cac_d, $cac_l,
            "movl 4(%esp), %edx",
            "movl 8(%esp), %eax",
            "movl -4(%edx), %esp",
            "leal 12(%edx), %ebp",
            "jmp *%eax",

            // call_finally_block( void *code_block, void *base_ptr ) — DECLSPEC_NORETURN.
            ".align 4", $cfb_g, $cfb_d, $cfb_l,
            "movl 8(%esp), %ebp",
            "jmp *4(%esp)",

            // call_filter( int (*func)(PEXCEPTION_POINTERS), void *arg, void *ebp )
            ".align 4", $cfi_g, $cfi_d, $cfi_l,
            "pushl %ebp",
            "pushl 12(%esp)",
            "movl 20(%esp), %ebp",
            "call *12(%esp)",
            "popl %ebp",
            "popl %ebp",
            "ret",

            // call_handler( void *(*func)(void), void *ebp )
            ".align 4", $chd_g, $chd_d, $chd_l,
            "pushl %ebp",
            "pushl %ebx",
            "pushl %esi",
            "pushl %edi",
            "movl 24(%esp), %ebp",
            "call *20(%esp)",
            "popl %edi",
            "popl %esi",
            "popl %ebx",
            "popl %ebp",
            "ret",

            // __CxxFrameHandler — the exported entry point. It exists as asm because the
            // function descriptor arrives in %eax, which no calling convention can name; the
            // thunk pushes it as the 5th argument of the cdecl `cxx_frame_handler` below.
            ".align 4", $cfh_g, $cfh_d, $cfh_l,
            "pushl $0",         // nested_frame
            "pushl %eax",       // descr
            "pushl 24(%esp)",   // dispatch
            "pushl 24(%esp)",   // context
            "pushl 24(%esp)",   // frame
            "pushl 24(%esp)",   // rec
            $cfh_call,
            "add $24,%esp",
            "ret",

            // _EH_prolog — "Provided for VC++ binary compatibility only", says the C. It links
            // a registration record onto fs:0 and rebuilds the caller's frame around it, then
            // `ret`s to an address it pushed itself.
            ".align 4", $ehp_g, $ehp_d, $ehp_l,
            "pushl $-1",
            "pushl %eax",
            "pushl %fs:0",
            "movl  %esp, %fs:0",
            "movl  12(%esp), %eax",
            "movl  %ebp, 12(%esp)",
            "leal  12(%esp), %ebp",
            "pushl %eax",
            "ret",

            options(att_syntax)
        );
    };
}

// i386 PE — msvcrt.dll. This is the arm the product builds.
#[cfg(target_os = "windows")]
seh_thunks! {
    continue_after_catch = (".globl _continue_after_catch",
                            ".def _continue_after_catch; .scl 2; .type 32; .endef",
                            "_continue_after_catch:"),
    call_finally_block   = (".globl _call_finally_block",
                            ".def _call_finally_block; .scl 2; .type 32; .endef",
                            "_call_finally_block:"),
    call_filter          = (".globl _call_filter",
                            ".def _call_filter; .scl 2; .type 32; .endef",
                            "_call_filter:"),
    call_handler         = (".globl _call_handler",
                            ".def _call_handler; .scl 2; .type 32; .endef",
                            "_call_handler:"),
    CxxFrameHandler      = (".globl ___CxxFrameHandler",
                            ".def ___CxxFrameHandler; .scl 2; .type 32; .endef",
                            "___CxxFrameHandler:",
                            "call _cxx_frame_handler"),
    EH_prolog            = (".globl __EH_prolog",
                            ".def __EH_prolog; .scl 2; .type 32; .endef",
                            "__EH_prolog:"),
}

// i386 ELF — the msvcrt-seh-difftest host harness only.
#[cfg(not(target_os = "windows"))]
seh_thunks! {
    continue_after_catch = (".globl continue_after_catch",
                            ".type continue_after_catch,@function",
                            "continue_after_catch:"),
    call_finally_block   = (".globl call_finally_block",
                            ".type call_finally_block,@function",
                            "call_finally_block:"),
    call_filter          = (".globl call_filter",
                            ".type call_filter,@function",
                            "call_filter:"),
    call_handler         = (".globl call_handler",
                            ".type call_handler,@function",
                            "call_handler:"),
    CxxFrameHandler      = (".globl __CxxFrameHandler",
                            ".type __CxxFrameHandler,@function",
                            "__CxxFrameHandler:",
                            "call cxx_frame_handler"),
    EH_prolog            = (".globl _EH_prolog",
                            ".type _EH_prolog,@function",
                            "_EH_prolog:"),
}

unsafe extern "C" {
    /// Restore the throwing frame's `%esp`/`%ebp` and jump to *addr*. Never returns.
    pub fn continue_after_catch(frame: *mut core::ffi::c_void, addr: *mut core::ffi::c_void);
    /// Set `%ebp` to *base_ptr* and jump to *code_block*. Never returns.
    pub fn call_finally_block(code_block: *mut core::ffi::c_void, base_ptr: *mut core::ffi::c_void);
    /// Call an `__except` filter with `%ebp` set to the guarded frame's.
    pub fn call_filter(
        func: Option<unsafe extern "C" fn(PEXCEPTION_POINTERS) -> i32>,
        arg: *mut core::ffi::c_void,
        ebp: *mut core::ffi::c_void,
    ) -> i32;
    /// Call an unwind/catch handler with `%ebp` set to the guarded frame's.
    pub fn call_handler(
        func: Option<unsafe extern "C" fn() -> *mut core::ffi::c_void>,
        ebp: *mut core::ffi::c_void,
    ) -> *mut core::ffi::c_void;
}

/// `NtCurrentTeb()` — a FORCEINLINE macro in C, so there is no symbol to import.
///
/// heap.rs carries its own copy for the same reason. They stay separate on purpose: that one
/// is behind the `heap` feature and this one behind `cpp`, and either may be built without
/// the other.
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, nomem, pure));
    teb
}

/// wine/exception.h's `__wine_push_frame` — link *frame* onto the head of the thread's SEH
/// registration list and return the previous head.
///
/// The i386 arm goes straight to `fs:0` rather than through the TEB pointer, exactly as the
/// C does: `NT_TIB.ExceptionList` is the first field of the TEB, so the two are the same
/// address, and the direct form is one instruction.
#[inline(always)]
pub unsafe fn __wine_push_frame(
    frame: *mut EXCEPTION_REGISTRATION_RECORD,
) -> *mut EXCEPTION_REGISTRATION_RECORD {
    let prev: *mut EXCEPTION_REGISTRATION_RECORD;
    core::arch::asm!("mov {}, fs:[0]", out(reg) prev, options(nostack, readonly));
    (*frame).Prev = prev;
    core::arch::asm!("mov fs:[0], {}", in(reg) frame, options(nostack));
    prev
}

/// wine/exception.h's `__wine_pop_frame` — unlink *frame* by making its `Prev` the head.
#[inline(always)]
pub unsafe fn __wine_pop_frame(
    frame: *mut EXCEPTION_REGISTRATION_RECORD,
) -> *mut EXCEPTION_REGISTRATION_RECORD {
    let prev = (*frame).Prev;
    core::arch::asm!("mov fs:[0], {}", in(reg) prev, options(nostack));
    prev
}
