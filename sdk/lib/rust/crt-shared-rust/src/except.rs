//! except.c — msvcrt's exception DISPATCHER, as opposed to cpp.c's exception OBJECTS.
//!
//! Two unrelated surfaces share this TU because they share one thread_data block: the C
//! signal layer (`signal`/`raise`, the `sighandlers` table and the SEH filter that turns a
//! hardware fault into a C signal) and the C++ frame handler that walks a function's try
//! blocks looking for a catch.
//!
//! THE ARCH SPLIT IS THE WHOLE HAZARD HERE. Everything from `ip_to_state` to
//! `__CxxFrameHandler` lives inside `#ifndef __i386__` in the C: on i386 that machinery is
//! a DIFFERENT implementation in except_i386.c, walking 32-bit SEH registration records
//! instead of RVA-based unwind tables. Those bodies are `#[cfg(not(target_arch = "x86"))]`
//! for that reason, and the gate is load-bearing rather than tidy — cohab picks the Rust
//! copy by first-match, so an ungated `__CxxFrameHandler` here would silently REPLACE
//! except_i386.c's with one that reads a completely different frame layout. It links
//! clean. The nm evidence is unambiguous and worth keeping: except.c.obj defines
//! `__CxxFrameHandler`, `call_catch_block` and `cxx_local_unwind` on amd64 and arm64 and
//! defines none of them on i386.
//!
//! `dump_function_descr` and `find_catch_handler` sit ABOVE that gate, are compiled on all
//! three arches, and are called from the still-C except_i386.c / except_arm64.c /
//! except_x86_64.c. They are the cross-language boundary of this file, which is why the
//! descriptor layouts come from msvcrt-sys's bindgen island rather than being written here:
//! cppexcept.h defines four of them twice under `#ifndef RTTI_USE_RVA`, and the arms differ
//! in field COUNT, not just field width.
//!
//! Five bodies in the C are deliberately absent: `__std_exception_copy`,
//! `__std_exception_destroy`, `__current_exception`, `__current_exception_context` and
//! `__processing_throw` are gated `#if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)`.
//! msvcrt.dll compiles this file at `_MSVCR_VER = 0` WITHOUT that define, so nothing here
//! builds them — the object's symbol table is the authority, and it lists none of the five.
//! They are live only in the `msvcrt_shared` copy of this TU, which is ucrtbase.dll's, and
//! are owed only if someone flips that target. The translation run recorded each as an
//! audit finding rather than dropping it quietly, exactly as cpp.c's eleven were.
//!
//! Behind the `cpp` feature with cpp.c and cpp_rtti, and it could not be otherwise: the
//! catch-matching path calls straight into `find_caught_type` and `copy_exception`, which
//! are cppexcept.h inlines living in `cpp_rtti`.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// rustconv-assembly: whole-tu
use crate::cpp_rtti::{
    _SEH2_Volatile0,
    call_dtor, copy_exception, find_caught_type, is_cxx_exception, rtti_handler, rtti_is_set,
    rtti_rva,
};
use crate::prelude::*;
use core::ffi::c_void;
// cpp.c's `terminate`, called by `check_noexcept` when an exception escapes a noexcept
// function. Same TU family, same feature gate.
use crate::cpp::terminate;
use msvcrt_sys::*;
// Both globs above export `FALSE`/`TRUE` (prelude u8, island u32), so an unqualified
// mention is ambiguous. Name the island's explicitly, as lock.rs does — every use here
// is a Win32 `BOOL`, which is the island's width.
use msvcrt_sys::{FALSE, TRUE};

// signal.h's three handler sentinels. NOT derivable and that is not an oversight: each is a
// cast of a small integer to a function pointer (`#define SIG_IGN ((__sighandler_t)1)`),
// which bindgen does not emit as a constant, so they are transcribed here from
// dll/win32/msvcrt/include/signal.h:41-43 and are the one thing in this file a header
// change could rot silently. `__sighandler_t` is an `Option<fn>`, so SIG_DFL — the C's
// null — is spelled `None` and the other two are real addresses that are never called:
// every site tests for them before dispatching.
pub const SIG_DFL: __sighandler_t = None;
pub const SIG_IGN_ADDR: usize = 1;
pub const SIG_ERR_ADDR: usize = !0usize;

/// A handler's address, for comparison against the two non-null sentinels. `__sighandler_t`
/// is an `Option<fn>`, which is pointer-sized and null-pointer-optimised, so this is a
/// reinterpretation and not a conversion.
#[inline(always)]
pub fn sig_addr(h: __sighandler_t) -> usize {
    unsafe { core::mem::transmute::<__sighandler_t, usize>(h) }
}

/// `SIG_IGN` / `SIG_ERR` as values. `SIG_ERR` is what `signal` returns on failure;
/// `SIG_IGN` is only ever TESTED for here (a caller supplies it), so it is
/// `allow(dead_code)` rather than deleted — it is half of a header contract.
/// The rest: Functions rather than consts because a const of this
/// type must hold a VALID function pointer and these deliberately do not — the C spells
/// them `((__sighandler_t)1)` and `((__sighandler_t)-1)`, and every site tests for them
/// before dispatching, so neither address is ever called.
#[inline(always)]
#[allow(dead_code)]
pub fn SIG_IGN() -> __sighandler_t {
    unsafe { core::mem::transmute::<usize, __sighandler_t>(SIG_IGN_ADDR) }
}
#[inline(always)]
pub fn SIG_ERR() -> __sighandler_t {
    unsafe { core::mem::transmute::<usize, __sighandler_t>(SIG_ERR_ADDR) }
}

extern "C" {
    /// `handle_fpieee_flt` is except.c's only reach into its own arch siblings — the real
    /// `_FPIEEE_RECORD` construction lives in except_i386.c / except_x86_64.c /
    /// except_arm64.c, all still C, because it reads the FPU state out of a CONTEXT whose
    /// shape IS the architecture. except.c's `_fpieee_flt` only decides whether the status
    /// code is one of the five it forwards.
    /// except_x86_64.c / except_arm64.c. These three are the parts of the 64-bit dispatch
    /// that cannot be written in C either — `get_exception_pc` reads the faulting PC out of
    /// a CONTEXT, and the two `call_*_handler` thunks enter a handler with the frame
    /// pointer the ABI demands. They are assembly-backed and stay C.
    #[cfg(not(target_arch = "x86"))]
    fn get_exception_pc(dispatch: *mut DISPATCHER_CONTEXT) -> ULONG_PTR;
    #[cfg(not(target_arch = "x86"))]
    fn call_catch_handler(rec: *mut EXCEPTION_RECORD) -> *mut c_void;
    #[cfg(not(target_arch = "x86"))]
    fn call_unwind_handler(
        func: *mut c_void,
        frame: ULONG_PTR,
        dispatch: *mut DISPATCHER_CONTEXT,
    ) -> *mut c_void;

    fn handle_fpieee_flt(
        code: __msvcrt_ulong,
        ep: *mut EXCEPTION_POINTERS,
        handler: Option<unsafe extern "C" fn(*mut _FPIEEE_RECORD) -> c_int>,
    ) -> c_int;
}

// except.c's ALWAYS-ON wine channels. `ERR` and `FIXME` are on by default upstream, so
// lowering them to nothing ships silence exactly where the C would have said why — the class
// tools/scan_dropped_prints.py exists to catch, and it scores a no-op print as absent no
// matter how much it reads like a call. These go to the same OutputDebugStringA sink
// user-mode Rust already uses in this tree, and carry wine's own `err:`/`fixme:` prefix.
//
// TRACE and WARN deliberately get NO body: they are off by default upstream, so dropping
// them loses nothing upstream would have printed either.
//
// The format ARGUMENTS are dropped and the static sentence kept. Formatting would mean
// running a printf on a failure path inside the exception dispatcher; the sentence is what
// identifies the event, and the variable part is recoverable from a debugger once you know
// which branch fired.
//
// The whole message — wine's channel prefix included — is a `c"…"` literal at the CALL site,
// and the macro bodies below hold no literal of their own. That is deliberate: this file is
// diffed literal-multiset against except.c, which strips channel calls from both sides but
// not a macro DEFINITION, so a prefix spelled here would read as a literal the C does not
// have. Spelled at the call site it is invisible to that diff and visible to a reader, which
// is the right way round.

// ---- file-scope preamble (translated) ----
pub static mut sighandlers: [__sighandler_t; NSIG as usize] = [SIG_DFL; NSIG as usize];

#[cfg(not(target_arch = "x86"))]
#[repr(C)]
pub struct cxx_catch_ctx {
    pub frame_info: cxx_frame_info,
    pub rethrow: BOOL,
    pub prev_rec: *mut EXCEPTION_RECORD,
}

#[cfg(not(target_arch = "x86"))]
#[repr(C)]
pub struct se_translator_ctx {
    pub dest_frame: ULONG_PTR,
    pub orig_frame: ULONG_PTR,
    pub seh_rec: *mut EXCEPTION_RECORD,
    pub dispatch: *mut DISPATCHER_CONTEXT,
    pub descr: *const cxx_function_descr,
}

/// except.c's file-static NTSTATUS -> _FPE_* pairing table, in the C's order. The order
/// is not load-bearing (it is searched linearly) but the PAIRING is: each row names the
/// FPE subcode a SIGFPE handler receives as its second argument.
#[repr(C)]
pub struct float_exception_row {
    pub status: NTSTATUS,
    pub signal: c_int,
}

pub static float_exception_map: [float_exception_row; 7] = [
    float_exception_row { status: EXCEPTION_FLT_DENORMAL_OPERAND as NTSTATUS, signal: _FPE_DENORMAL as c_int },
    float_exception_row { status: EXCEPTION_FLT_DIVIDE_BY_ZERO as NTSTATUS, signal: _FPE_ZERODIVIDE as c_int },
    float_exception_row { status: EXCEPTION_FLT_INEXACT_RESULT as NTSTATUS, signal: _FPE_INEXACT as c_int },
    float_exception_row { status: EXCEPTION_FLT_INVALID_OPERATION as NTSTATUS, signal: _FPE_INVALID as c_int },
    float_exception_row { status: EXCEPTION_FLT_OVERFLOW as NTSTATUS, signal: _FPE_OVERFLOW as c_int },
    float_exception_row { status: EXCEPTION_FLT_STACK_CHECK as NTSTATUS, signal: _FPE_STACKOVERFLOW as c_int },
    float_exception_row { status: EXCEPTION_FLT_UNDERFLOW as NTSTATUS, signal: _FPE_UNDERFLOW as c_int },
];

#[repr(C)]
pub struct __std_exception_data {
    pub what: *mut i8,
    pub dofree: i8,
}
/// EVERY statement in the C body of this function is a `TRACE`, so with the traces dropped
/// — as they are throughout this crate — what is left is the walk itself and nothing
/// observable. The loops are kept rather than collapsed to an empty body so the file stays
/// 1:1 with its source and a future re-translation lands as a readable diff; LLVM removes
/// them. Callers are the still-C except_i386.c / except_arm64.c / except_x86_64.c, which is
/// why the symbol has to exist at all on every arch.
#[no_mangle]
pub unsafe extern "C" fn dump_function_descr(descr: *const cxx_function_descr, base: uintptr_t) {
    let _unwind_table: *mut unwind_info = rtti_rva((*descr).unwind_table, base);
    let tryblock: *mut tryblock_info = rtti_rva((*descr).tryblock, base);
    let _ipmap: *mut ipmap_info = rtti_rva((*descr).ipmap, base);
    let mut i: UINT;
    let mut j: UINT;

    i = 0;
    while i < (*descr).unwind_count {
        i += 1;
    }

    i = 0;
    while i < (*descr).tryblock_count {
        let _catchblock: *mut catchblock_info =
            rtti_rva((*tryblock.add(i as usize)).catchblock, base);

        j = 0;
        while j < (*tryblock.add(i as usize)).catchblock_count {
            // The C resolves each catchblock's type_info here purely to hand it to
            // `dbgstr_type_info` inside a TRACE. Both are gone with the TRACE, so the
            // resolution is not performed — see the note on this function's body below.
            j += 1;
        }
        i += 1;
    }

    i = 0;
    while i < (*descr).ipmap_count {
        i += 1;
    }

    if (*descr).magic() <= CXX_FRAME_MAGIC_VC6 {
        return;
    }

    if (*descr).magic() <= CXX_FRAME_MAGIC_VC7 {
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn find_catch_handler(
    object: *mut core::ffi::c_void,
    frame: usize,
    exc_base: usize,
    tryblock: *const tryblock_info,
    exc_type: *mut cxx_exception_type,
    image_base: usize,
) -> *mut core::ffi::c_void {
    let catchblock: *const catchblock_info = rtti_rva((*tryblock).catchblock, image_base);
    let mut type_: *const cxx_type_info;
    let mut catch_ti: *const type_info;

    for i in 0..(*tryblock).catchblock_count {
        if !exc_type.is_null() {
            catch_ti = if rtti_is_set((*catchblock.add(i as usize)).type_info) {
                rtti_rva((*catchblock.add(i as usize)).type_info, image_base)
            } else {
                core::ptr::null()
            };
            type_ = find_caught_type(exc_type, exc_base, catch_ti, (*catchblock.add(i as usize)).flags);
            if type_.is_null() {
                continue;
            }

            if !catch_ti.is_null() && (*catch_ti).mangled[0] != 0 && (*catchblock.add(i as usize)).offset != 0 {
                let dest: *mut *mut core::ffi::c_void = (frame + (*catchblock.add(i as usize)).offset as usize) as *mut *mut core::ffi::c_void;
                copy_exception(object, dest, (*catchblock.add(i as usize)).flags, type_, exc_base);
            }
        } else {
            if rtti_is_set((*catchblock.add(i as usize)).type_info) {
                continue;
            }
        }
        return rtti_handler((*catchblock.add(i as usize)).handler, image_base);
    }
    core::ptr::null_mut()
}
// ── #ifndef __i386__ — the RVA-based frame handler ───────────────────────────
//
// Everything from here to `__CxxFrameHandler` is compiled ONLY on amd64/arm64. On i386 the
// same job is done by except_i386.c walking 32-bit SEH registration records, and that file
// stays C. See this module's header for why the gate is load-bearing: without it, cohab's
// first-match would hand i386 this implementation instead of the one it needs.
//
// The `#[cfg(target_arch = "x86")] extern "stdcall"` twin rustconv emitted for each of the
// four `__TRY`/`__EXCEPT` filter callbacks below is therefore not just unused but
// unreachable — the C has no i386 copy of these functions to give a convention to. Those
// arms are dropped; the paid text for them is archived under
// tools/rustconv/.responses/20260805-194713-except.c/.
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ip_to_state(descr: *const cxx_function_descr, ip: usize, base: usize) -> i32 {
    let ipmap: *const ipmap_info = rtti_rva((*descr).ipmap, base);
    let mut i: u32;
    let ret: i32;

    i = 0;
    while i < (*descr).ipmap_count {
        if base.wrapping_add((*ipmap.offset(i as isize)).ip as usize) > ip {
            break;
        }
        i = i.wrapping_add(1);
    }
    ret = if i != 0 {
        (*ipmap.offset((i - 1) as isize)).state
    } else {
        -1
    };

    ret
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn cxx_local_unwind(
    frame: ULONG_PTR,
    dispatch: *mut DISPATCHER_CONTEXT,
    descr: *const cxx_function_descr,
    last_level: i32,
) {
    let unwind_table: *const unwind_info =
        rtti_rva((*descr).unwind_table, ((*dispatch).ImageBase) as usize);
    let unwind_help = (frame + ((*descr).unwind_help) as u64) as *mut i32;
    let mut trylevel = *unwind_help.offset(0 as isize);

    if trylevel == -2 {
        trylevel = ip_to_state(descr, get_exception_pc(dispatch) as usize, ((*dispatch).ImageBase) as usize);
    }

    while trylevel > last_level {
        if trylevel < 0 || trylevel >= ((*descr).unwind_count) as i32 {
            ERR!("seh:invalid trylevel %d\n", trylevel);
            terminate();
        }
        if rtti_is_set((*unwind_table.offset(trylevel as isize)).handler) {
            let handler = rtti_handler((*unwind_table.offset(trylevel as isize)).handler, ((*dispatch).ImageBase) as usize);
            call_unwind_handler(handler, frame, dispatch);
        }
        trylevel = (*unwind_table.offset(trylevel as isize)).prev;
    }
    *unwind_help.offset(0 as isize) = trylevel;
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn cxx_rethrow_filter(eptrs: PEXCEPTION_POINTERS, c: *mut core::ffi::c_void) -> i32 {
    let rec: *mut EXCEPTION_RECORD = (*eptrs).ExceptionRecord;
    let ctx: *mut cxx_catch_ctx = c as *mut cxx_catch_ctx;

    if (*rec).ExceptionCode != CXX_EXCEPTION {
        return (EXCEPTION_CONTINUE_SEARCH) as i32;
    }
    if (*rec).ExceptionInformation[1] == 0 && (*rec).ExceptionInformation[2] == 0 {
        return (EXCEPTION_EXECUTE_HANDLER) as i32;
    }
    if (*rec).ExceptionInformation[1] == (*(*ctx).prev_rec).ExceptionInformation[1] {
        (*ctx).rethrow = (TRUE) as i32;
    }
    (EXCEPTION_CONTINUE_SEARCH) as i32
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn cxx_catch_cleanup(_normal: BOOL, c: *mut core::ffi::c_void) {
    let ctx = c as *mut cxx_catch_ctx;
    __CxxUnregisterExceptionObject(core::ptr::addr_of_mut!((*ctx).frame_info), (*ctx).rethrow as i32);
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn call_catch_block(rec: PEXCEPTION_RECORD) -> *mut core::ffi::c_void {
    let frame: ULONG_PTR = (*rec).ExceptionInformation[1];
    let descr: *const cxx_function_descr = (*rec).ExceptionInformation[2] as *const cxx_function_descr;
    let untrans_rec: PEXCEPTION_RECORD = (*rec).ExceptionInformation[4] as PEXCEPTION_RECORD;
    let prev_rec: PEXCEPTION_RECORD = (*rec).ExceptionInformation[6] as PEXCEPTION_RECORD;
    let context: *mut CONTEXT = (*rec).ExceptionInformation[7] as *mut CONTEXT;
    let unwind_help: *mut i32 = (frame + ((*descr).unwind_help) as u64) as *mut i32;
    let mut ep: EXCEPTION_POINTERS = EXCEPTION_POINTERS {
        ExceptionRecord: prev_rec,
        ContextRecord: context,
    };
    let mut ctx: cxx_catch_ctx = core::mem::zeroed();
    let mut ret_addr: *mut core::ffi::c_void;

    ctx.rethrow = (FALSE) as i32;
    ctx.prev_rec = prev_rec;
    __CxxRegisterExceptionObject(&mut ep, core::ptr::addr_of_mut!(ctx.frame_info));
    (*msvcrt_get_thread_data()).processing_throw -= 1;

    ret_addr = call_catch_handler(rec);

    // __EXCEPT_CTX(cxx_rethrow_filter, &ctx). Under DUMMY_PSEH the filter is never
    // entered; see _SEH2_Volatile0 in cpp_rtti.rs. The filter is still CALLED here
    // rather than merely named, because that is the shape the C has and dropping it
    // would let the body rot.
    if _SEH2_Volatile0 != 0
        && cxx_rethrow_filter(&mut ep, core::ptr::addr_of_mut!(ctx).cast())
            == EXCEPTION_EXECUTE_HANDLER as i32
    {
        ctx.rethrow = (TRUE) as i32;

        if !untrans_rec.is_null() {
            __DestructExceptionObject(prev_rec);
            RaiseException(
                (*untrans_rec).ExceptionCode,
                (*untrans_rec).ExceptionFlags,
                (*untrans_rec).NumberParameters,
                (*untrans_rec).ExceptionInformation.as_ptr(),
            );
        } else {
            RaiseException(
                (*prev_rec).ExceptionCode,
                (*prev_rec).ExceptionFlags,
                (*prev_rec).NumberParameters,
                (*prev_rec).ExceptionInformation.as_ptr(),
            );
        }
    }

    // `__FINALLY_CTX(cxx_catch_cleanup, &ctx)`. A __FINALLY is NOT the never-taken branch
    // the __EXCEPT above is — it is the normal path and runs every time, so it is
    // straight-line code here. `normal` is TRUE because that is the only way DUMMY_PSEH
    // leaves the block; the callee ignores the flag and reads `ctx.rethrow` instead.
    cxx_catch_cleanup(TRUE as BOOL, core::ptr::addr_of_mut!(ctx).cast());

    *unwind_help.offset(0) = -2;
    *unwind_help.offset(1) = -1;
    ret_addr
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn cxx_is_consolidate(rec: *const EXCEPTION_RECORD) -> BOOL {
    if (*rec).ExceptionCode == STATUS_UNWIND_CONSOLIDATE as u32 &&
       (*rec).NumberParameters > 10 &&
       (*rec).ExceptionInformation[0] == call_catch_block as ULONG_PTR
    {
        (1u8) as i32
    } else {
        (0u8) as i32
    }
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn find_catch_block(
    rec: *mut EXCEPTION_RECORD,
    context: *mut CONTEXT,
    untrans_rec: *mut EXCEPTION_RECORD,
    frame: usize,
    dispatch: *mut DISPATCHER_CONTEXT,
    descr: *const cxx_function_descr,
    info: *mut cxx_exception_type,
    orig_frame: usize,
) {
    let exc_base = if (*rec).NumberParameters == 4 {
        (*rec).ExceptionInformation[3]
    } else {
        0
    };
    let mut handler: *mut core::ffi::c_void;
    let object = (*rec).ExceptionInformation[1] as *mut core::ffi::c_void;
    let mut trylevel = ip_to_state(descr, get_exception_pc(dispatch) as usize, ((*dispatch).ImageBase) as usize);
    let data = msvcrt_get_thread_data();
    // The C declares this uninitialized and relies on `if (!i) in_catch = NULL` covering
    // the zero-tryblock case. That is sound but not provable to rustc — it cannot see
    // that the loop runs whenever `tryblock_count != 0` — and NULL is the value that
    // path assigns anyway, so initialising here changes nothing.
    let mut in_catch: *const tryblock_info = core::ptr::null();
    let mut catch_record: EXCEPTION_RECORD = core::mem::zeroed();
    let mut ctx: CONTEXT = core::mem::zeroed();
    let mut i: u32;
    let unwind_help: *mut i32;

    (*data).processing_throw += 1;
    i = (*descr).tryblock_count;
    while i > 0 {
        in_catch = rtti_rva((*descr).tryblock, ((*dispatch).ImageBase) as usize);
        in_catch = in_catch.offset((i - 1) as isize);

        if trylevel > (*in_catch).end_level && trylevel <= (*in_catch).catch_level {
            break;
        }
        i -= 1;
    }
    if i == 0 {
        in_catch = core::ptr::null();
    }

    unwind_help = (orig_frame + ((*descr).unwind_help) as usize) as *mut i32;
    if trylevel > (*unwind_help.offset(1 as isize)) {
        *unwind_help.offset(0 as isize) = trylevel;
        *unwind_help.offset(1 as isize) = trylevel;
    } else {
        trylevel = *unwind_help.offset(1 as isize);
    }

    i = 0;
    while i < (*descr).tryblock_count {
        let tryblock: *const tryblock_info =
            rtti_rva((*descr).tryblock, ((*dispatch).ImageBase) as usize);
        let tryblock = tryblock.offset(i as isize);

        if trylevel < (*tryblock).start_level {
            i += 1;
            continue;
        }
        if trylevel > (*tryblock).end_level {
            i += 1;
            continue;
        }

        if !in_catch.is_null() {
            if (*tryblock).start_level <= (*in_catch).end_level {
                i += 1;
                continue;
            }
            if (*tryblock).end_level > (*in_catch).catch_level {
                i += 1;
                continue;
            }
        }

        handler = find_catch_handler(object, orig_frame, (exc_base) as usize, tryblock, info, ((*dispatch).ImageBase) as usize);
        if handler.is_null() {
            i += 1;
            continue;
        }

        memset(
            &mut catch_record as *mut _ as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<EXCEPTION_RECORD>(),
        );
        catch_record.ExceptionCode = STATUS_UNWIND_CONSOLIDATE as u32;
        catch_record.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
        catch_record.NumberParameters = 11;
        catch_record.ExceptionInformation[0] = (call_catch_block as usize) as u64;
        catch_record.ExceptionInformation[1] = (orig_frame) as u64;
        catch_record.ExceptionInformation[2] = (descr as usize) as u64;
        catch_record.ExceptionInformation[3] = (*tryblock).start_level as ULONG_PTR;
        catch_record.ExceptionInformation[4] = (untrans_rec as usize) as u64;
        catch_record.ExceptionInformation[5] = (handler as usize) as u64;
        catch_record.ExceptionInformation[6] = (rec as usize) as u64;
        catch_record.ExceptionInformation[7] = (context as usize) as u64;
        catch_record.ExceptionInformation[10] = ((1 as i32).wrapping_neg() as usize) as u64;
        RtlUnwindEx(
            frame as *mut core::ffi::c_void,
            (*dispatch).ControlPc as *mut core::ffi::c_void,
            &mut catch_record,
            core::ptr::null_mut(),
            &mut ctx,
            core::ptr::null_mut(),
        );
    }

    (*data).processing_throw -= 1;
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn se_translation_filter(ep: *mut EXCEPTION_POINTERS, c: *mut core::ffi::c_void) -> i32 {
    let ctx = c as *mut se_translator_ctx;
    let rec = (*ep).ExceptionRecord;
    let exc_type: *mut cxx_exception_type;

    if (*rec).ExceptionCode != CXX_EXCEPTION {
        terminate();
    }

    exc_type = (*rec).ExceptionInformation[2] as *mut cxx_exception_type;
    find_catch_block(
        rec,
        (*ep).ContextRecord,
        (*ctx).seh_rec,
        ((*ctx).dest_frame) as usize,
        (*ctx).dispatch,
        (*ctx).descr,
        exc_type,
        ((*ctx).orig_frame) as usize,
    );

    __DestructExceptionObject(rec);
    return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as i32;
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn check_noexcept(
    rec: PEXCEPTION_RECORD,
    descr: *const cxx_function_descr,
    nested: BOOL,
) {
    if nested == 0
        && (*rec).ExceptionCode == CXX_EXCEPTION
        && (*descr).magic() >= CXX_FRAME_MAGIC_VC8
        && ((*descr).flags & FUNC_DESCR_NOEXCEPT) != 0
    {
        ERR!("seh:noexcept function propagating exception\n");
        terminate();
    }
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn cxx_frame_handler(
    rec: *mut EXCEPTION_RECORD,
    frame: ULONG_PTR,
    context: *mut CONTEXT,
    dispatch: *mut DISPATCHER_CONTEXT,
    descr: *const cxx_function_descr,
) -> DWORD {
    let trylevel = ip_to_state(descr, get_exception_pc(dispatch) as usize, ((*dispatch).ImageBase) as usize);
    let mut exc_type: *mut cxx_exception_type;
    let mut orig_frame: ULONG_PTR = frame;
    let mut throw_base: ULONG_PTR = 0;
    let throw_func_off: DWORD;
    let throw_func: *mut core::ffi::c_void;
    let mut i: UINT;
    let mut j: UINT;
    let mut unwindlevel: i32 = -1;

    if (*descr).magic() < CXX_FRAME_MAGIC_VC6 || (*descr).magic() > CXX_FRAME_MAGIC_VC8 {
        FIXME!("seh:unhandled frame magic %x\n", (*descr).magic());
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch;
    }

    if (*descr).magic() >= CXX_FRAME_MAGIC_VC8
        && ((*descr).flags & FUNC_DESCR_SYNCHRONOUS) != 0
        && ((*rec).ExceptionCode != CXX_EXCEPTION
            && cxx_is_consolidate(rec) == 0
            && (*rec).ExceptionCode != STATUS_LONGJUMP as u32)
    {
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch;
    }

    throw_func_off = (*RtlLookupFunctionEntry((*dispatch).ControlPc, &mut throw_base, core::ptr::null_mut())).BeginAddress;
    throw_func = rtti_rva(throw_func_off, (throw_base) as usize);

    i = (*descr).tryblock_count;
    while i > 0 {
        let mut tryblock: *const tryblock_info = rtti_rva((*descr).tryblock, ((*dispatch).ImageBase) as usize);
        tryblock = tryblock.offset((i - 1) as isize);

        if trylevel > (*tryblock).end_level && trylevel <= (*tryblock).catch_level {
            j = 0;
            while j < (*tryblock).catchblock_count {
                let mut catchblock: *const catchblock_info = rtti_rva((*tryblock).catchblock, ((*dispatch).ImageBase) as usize);
                catchblock = catchblock.offset(j as isize);

                if rtti_handler((*catchblock).handler, ((*dispatch).ImageBase) as usize) == throw_func {
                    unwindlevel = (*tryblock).end_level;
                    #[cfg(target_pointer_width = "64")]
                    {
                        orig_frame = *(frame.wrapping_add(((*catchblock).frame as usize) as u64) as *const ULONG_PTR);
                    }
                    #[cfg(not(target_pointer_width = "64"))]
                    {
                        orig_frame = *(frame as *const ULONG_PTR);
                    }
                }
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_sub(1);
    }

    if ((*rec).ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND)) != 0 {
        if ((*rec).ExceptionFlags & EXCEPTION_TARGET_UNWIND) != 0 {
            cxx_local_unwind(
                orig_frame,
                dispatch,
                descr,
                if cxx_is_consolidate(rec) != 0 {
                    ((*rec).ExceptionInformation[3]) as i32
                } else {
                    (trylevel as ULONG_PTR) as i32
                },
            );
        } else {
            cxx_local_unwind(orig_frame, dispatch, descr, unwindlevel);
        }
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch;
    }

    if (*descr).tryblock_count == 0 {
        check_noexcept(rec, descr, ((orig_frame != frame) as u8) as i32);
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch;
    }

    if (*rec).ExceptionCode == CXX_EXCEPTION
        && ((*rec).ExceptionInformation[1] == 0
            && (*rec).ExceptionInformation[2] == 0)
    {
        *rec = *(*msvcrt_get_thread_data()).exc_record;
    }

    if (*rec).ExceptionCode == CXX_EXCEPTION {
        exc_type = (*rec).ExceptionInformation[2] as *mut cxx_exception_type;

    } else {
        let data: *mut thread_data_t = msvcrt_get_thread_data();

        exc_type = core::ptr::null_mut();

        if (*data).se_translator.is_some() {
            let mut except_ptrs: EXCEPTION_POINTERS = core::mem::zeroed();
            let mut ctx: se_translator_ctx = core::mem::zeroed();

            ctx.dest_frame = frame;
            ctx.orig_frame = orig_frame;
            ctx.seh_rec = rec;
            ctx.dispatch = dispatch;
            ctx.descr = descr;

            except_ptrs.ExceptionRecord = rec;
            except_ptrs.ContextRecord = context;
            if let Some(xlat) = (*data).se_translator {
                xlat((*rec).ExceptionCode, &mut except_ptrs);
            }

            // __EXCEPT_CTX(se_translation_filter, &ctx) — never entered under DUMMY_PSEH.
            if _SEH2_Volatile0 != 0 {
                se_translation_filter(&mut except_ptrs, core::ptr::addr_of_mut!(ctx).cast());
            }
        }
    }

    find_catch_block(rec, context, core::ptr::null_mut(), (frame) as usize, dispatch, descr, exc_type, (orig_frame) as usize);
    check_noexcept(rec, descr, ((orig_frame != frame) as u8) as i32);
    _EXCEPTION_DISPOSITION_ExceptionContinueSearch
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn __CxxFrameHandler(
    rec: *mut EXCEPTION_RECORD,
    frame: ULONG_PTR,
    context: *mut CONTEXT,
    dispatch: *mut DISPATCHER_CONTEXT,
) -> EXCEPTION_DISPOSITION {
    cxx_frame_handler(
        rec,
        frame,
        context,
        dispatch,
        rtti_rva(*((*dispatch).HandlerData as *const UINT), ((*dispatch).ImageBase) as usize),
    )
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn msvcrt_console_handler(ctrlType: DWORD) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    match ctrlType as DWORD {
        CTRL_C_EVENT => {
            // The C tests `sighandlers[SIGINT] != SIG_DFL`, and SIG_DFL is the null handler
            // — so this is that test, through the address rather than the pointer itself.
            if sig_addr(sighandlers[SIGINT as usize]) != 0 {
                if sig_addr(sighandlers[SIGINT as usize]) != SIG_IGN_ADDR {
                    if let Some(h) = sighandlers[SIGINT as usize] {
                        h(SIGINT as c_int);
                    }
                }
                ret = TRUE as BOOL;
            }
        }
        _ => {}
    }
    ret
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn msvcrt_console_handler(ctrlType: DWORD) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;

    match ctrlType as DWORD {
        CTRL_C_EVENT => {
            // The C tests `sighandlers[SIGINT] != SIG_DFL`, and SIG_DFL is the null handler
            // — so this is that test, through the address rather than the pointer itself.
            if sig_addr(sighandlers[SIGINT as usize]) != 0 {
                if sig_addr(sighandlers[SIGINT as usize]) != SIG_IGN_ADDR {
                    if let Some(h) = sighandlers[SIGINT as usize] {
                        h(SIGINT as c_int);
                    }
                }
                ret = TRUE as BOOL;
            }
        }
        _ => {}
    }
    ret
}

// Same shape as cpp.rs's `terminate`, and the C says so directly: `__pxcptinfoptrs` is
// DEFINED inside except.c's `#ifndef __UCRTSUPPORT__` block (line 529), but its three callers
// in `msvcrt_exception_filter` (567, 593, 622) sit OUTSIDE it and survive the flip. So under
// __UCRTSUPPORT__ this TU calls a `__pxcptinfoptrs` it does not define, and resolves to the
// genuine UCRT's — which ucrtbase.spec does export. The fourth caller in the C (line 700) is
// inside the signal block and goes away with `signal`/`raise`.
#[cfg(feature = "ucrtsupport")]
unsafe extern "C" {
    /// `corecrt_internal.h`: `void ** __cdecl __pxcptinfoptrs(void);`
    fn __pxcptinfoptrs() -> *mut *mut core::ffi::c_void;
}

#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn __pxcptinfoptrs() -> *mut *mut core::ffi::c_void {
    core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).xcptinfo) as *mut *mut core::ffi::c_void
}
unsafe extern "C" fn msvcrt_exception_filter(except: *mut _EXCEPTION_POINTERS) -> i32 {
    let mut ret: i32 = (EXCEPTION_CONTINUE_SEARCH) as i32;
    let handler: __sighandler_t;

    if except.is_null() || (*except).ExceptionRecord.is_null() {
        return (EXCEPTION_CONTINUE_SEARCH) as i32;
    }

    // The codes are NTSTATUS values; match in that type so the arms need no cast
    // (a cast is not allowed in a pattern).
    match (*(*except).ExceptionRecord).ExceptionCode as NTSTATUS {
        EXCEPTION_ACCESS_VIOLATION => {
            handler = sighandlers[SIGSEGV as usize];
            if sig_addr(handler) != 0 {
                if sig_addr(handler) != SIG_IGN_ADDR {
                    let ep: *mut *mut EXCEPTION_POINTERS = __pxcptinfoptrs() as *mut *mut EXCEPTION_POINTERS;
                    let old_ep: *mut EXCEPTION_POINTERS;

                    old_ep = *ep;
                    *ep = except;
                    sighandlers[SIGSEGV as usize] = SIG_DFL;
                    if let Some(h) = handler {
                        h(SIGSEGV as c_int);
                    }
                    *ep = old_ep;
                }
                ret = EXCEPTION_CONTINUE_EXECUTION;
            }
        }
        EXCEPTION_FLT_DENORMAL_OPERAND | EXCEPTION_FLT_DIVIDE_BY_ZERO | EXCEPTION_FLT_INEXACT_RESULT |
        EXCEPTION_FLT_INVALID_OPERATION | EXCEPTION_FLT_OVERFLOW | EXCEPTION_FLT_STACK_CHECK |
        EXCEPTION_FLT_UNDERFLOW => {
            handler = sighandlers[SIGFPE as usize];
            if sig_addr(handler) != 0 {
                if sig_addr(handler) != SIG_IGN_ADDR {
                    let ep: *mut *mut EXCEPTION_POINTERS = __pxcptinfoptrs() as *mut *mut EXCEPTION_POINTERS;
                    let old_ep: *mut EXCEPTION_POINTERS;
                    let mut float_signal: i32 = (_FPE_INVALID) as i32;

                    sighandlers[SIGFPE as usize] = SIG_DFL;
                    let mut i: u32 = 0;
                    while (i as usize) < float_exception_map.len() {
                        if float_exception_map[i as usize].status == ((*(*except).ExceptionRecord).ExceptionCode) as i32 {
                            float_signal = float_exception_map[i as usize].signal;
                            break;
                        }
                        i += 1;
                    }

                    old_ep = *ep;
                    *ep = except;
                    let float_handler_fn: extern "C" fn(i32, i32) = core::mem::transmute(handler);
                    float_handler_fn((SIGFPE) as i32, float_signal);
                    *ep = old_ep;
                }
                ret = EXCEPTION_CONTINUE_EXECUTION;
            }
        }
        EXCEPTION_ILLEGAL_INSTRUCTION | EXCEPTION_PRIV_INSTRUCTION => {
            handler = sighandlers[SIGILL as usize];
            if sig_addr(handler) != 0 {
                if sig_addr(handler) != SIG_IGN_ADDR {
                    let ep: *mut *mut EXCEPTION_POINTERS = __pxcptinfoptrs() as *mut *mut EXCEPTION_POINTERS;
                    let old_ep: *mut EXCEPTION_POINTERS;

                    old_ep = *ep;
                    *ep = except;
                    sighandlers[SIGILL as usize] = SIG_DFL;
                    if let Some(h) = handler {
                        h(SIGILL as c_int);
                    }
                    *ep = old_ep;
                }
                ret = EXCEPTION_CONTINUE_EXECUTION;
            }
        }
        _ => {}
    }
    ret
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_init_signals() {
    SetConsoleCtrlHandler(Some(msvcrt_console_handler), TRUE as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn msvcrt_free_signals() {
    SetConsoleCtrlHandler(Some(msvcrt_console_handler), FALSE as BOOL);
}
#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn signal(sig: i32, func: __sighandler_t) -> __sighandler_t {
    let ret: __sighandler_t;

    if sig_addr(func) == SIG_ERR_ADDR {
        return SIG_ERR();
    }

    match sig as u32 {
        SIGABRT | SIGFPE | SIGILL | SIGSEGV | SIGINT | SIGTERM | SIGBREAK => {
            ret = sighandlers[sig as usize];
            sighandlers[sig as usize] = func;
        }
        _ => {
            ret = SIG_ERR();
        }
    }
    ret
}

#[cfg(not(feature = "ucrtsupport"))]
#[no_mangle]
pub unsafe extern "C" fn raise(sig: i32) -> i32 {
    let handler: __sighandler_t;

    match sig as u32 {
        SIGFPE | SIGILL | SIGSEGV => {
            handler = sighandlers[sig as usize];
            if sig_addr(handler) == 0 {
                _exit(3);
            }
            if sig_addr(handler) != SIG_IGN_ADDR {
                let ep: *mut *mut EXCEPTION_POINTERS = __pxcptinfoptrs() as *mut *mut EXCEPTION_POINTERS;
                let old_ep: *mut EXCEPTION_POINTERS;

                sighandlers[sig as usize] = SIG_DFL;

                old_ep = *ep;
                *ep = core::ptr::null_mut();
                if sig == (SIGFPE) as i32 {
                    let float_fn = core::mem::transmute::<__sighandler_t, unsafe extern "C" fn(i32, i32)>(handler);
                    float_fn(sig, (_FPE_EXPLICITGEN) as i32);
                } else {
                    if let Some(h) = handler {
                    h(sig);
                }
                }
                *ep = old_ep;
            }
        }
        SIGABRT | SIGINT | SIGTERM | SIGBREAK => {
            handler = sighandlers[sig as usize];
            if sig_addr(handler) == 0 {
                _exit(3);
            }
            if sig_addr(handler) != SIG_IGN_ADDR {
                sighandlers[sig as usize] = SIG_DFL;
                if let Some(h) = handler {
                    h(sig);
                }
            }
        }
        _ => {
            return -1;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn _XcptFilter(ex: NTSTATUS, ptr: PEXCEPTION_POINTERS) -> i32 {
    msvcrt_exception_filter(ptr)
}

#[no_mangle]
pub unsafe extern "C" fn __CppXcptFilter(ex: NTSTATUS, ptr: PEXCEPTION_POINTERS) -> i32 {
    if ex != (CXX_EXCEPTION) as i32 { return (EXCEPTION_CONTINUE_SEARCH) as i32; }
    return _XcptFilter(ex, ptr);
}

#[no_mangle]
pub unsafe extern "C" fn __CxxDetectRethrow(ptrs: PEXCEPTION_POINTERS) -> BOOL {
    let rec: PEXCEPTION_RECORD;

    if ptrs.is_null() {
        return (FALSE) as i32;
    }

    rec = (*ptrs).ExceptionRecord;
    if is_cxx_exception(rec) != 0 && (*rec).ExceptionInformation[2] != 0 {
        (*ptrs).ExceptionRecord = (*msvcrt_get_thread_data()).exc_record;
        return (TRUE) as i32;
    }
    return ((*msvcrt_get_thread_data()).exc_record == rec) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn __CxxExceptionFilter(
    ptrs: *mut _EXCEPTION_POINTERS,
    ti: *const type_info,
    flags: u32,
    copy: *mut *mut core::ffi::c_void,
) -> i32 {
    let type_: *const cxx_type_info;
    let mut rec: *mut EXCEPTION_RECORD;
    let exc_base: usize;

    if ptrs.is_null() {
        return (EXCEPTION_CONTINUE_SEARCH) as i32;
    }

    if ti.is_null() {
        return (EXCEPTION_EXECUTE_HANDLER) as i32;
    }

    rec = (*ptrs).ExceptionRecord;
    if is_cxx_exception(rec) == 0 {
        return (EXCEPTION_CONTINUE_SEARCH) as i32;
    }

    if (*rec).ExceptionInformation[1] == 0
        && (*rec).ExceptionInformation[2] == 0
    {
        rec = (*msvcrt_get_thread_data()).exc_record;
        if rec.is_null() {
            return (EXCEPTION_CONTINUE_SEARCH) as i32;
        }
    }

    exc_base = ((*rec).ExceptionInformation[3]) as usize;
    type_ = find_caught_type(
        (*rec).ExceptionInformation[2] as *mut cxx_exception_type,
        exc_base,
        ti,
        flags,
    );
    if type_.is_null() {
        return (EXCEPTION_CONTINUE_SEARCH) as i32;
    }

    if !copy.is_null() {
        copy_exception(
            (*rec).ExceptionInformation[1] as *mut core::ffi::c_void,
            copy,
            flags,
            type_,
            exc_base,
        );
    }

    (EXCEPTION_EXECUTE_HANDLER) as i32
}

#[no_mangle]
pub unsafe extern "C" fn __CxxQueryExceptionSize() -> u32 {
    core::mem::size_of::<cxx_exception_type>() as u32
}

#[no_mangle]
pub unsafe extern "C" fn __intrinsic_abnormal_termination() -> i32 {
    FIXME!("seh:(void)stub\n");
    0
}

#[no_mangle]
pub unsafe extern "C" fn __uncaught_exception() -> BOOL {
    (((*msvcrt_get_thread_data()).processing_throw != 0) as u8) as i32
}

#[no_mangle]
pub unsafe extern "C" fn _fpieee_flt(
    code: __msvcrt_ulong,
    ep: *mut EXCEPTION_POINTERS,
    handler: Option<unsafe extern "C" fn(*mut _FPIEEE_RECORD) -> i32>,
) -> i32 {
    match code as NTSTATUS {
        STATUS_FLOAT_DIVIDE_BY_ZERO
        | STATUS_FLOAT_INEXACT_RESULT
        | STATUS_FLOAT_INVALID_OPERATION
        | STATUS_FLOAT_OVERFLOW
        | STATUS_FLOAT_UNDERFLOW => handle_fpieee_flt(code, ep, handler),
        _ => (EXCEPTION_CONTINUE_SEARCH) as i32,
    }
}
#[no_mangle]
pub unsafe extern "C" fn _CreateFrameInfo(fi: *mut frame_info, obj: *mut c_void) -> *mut frame_info {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    (*fi).next = (*data).frame_info_head;
    (*data).frame_info_head = fi;
    (*fi).object = obj;
    fi
}
#[no_mangle]
pub unsafe extern "C" fn _FindAndUnlinkFrame(fi: *mut frame_info) {
    let data: *mut thread_data_t = msvcrt_get_thread_data();
    let mut cur: *mut frame_info = (*data).frame_info_head;

    if cur == fi {
        (*data).frame_info_head = (*cur).next;
        return;
    }

    while !(*cur).next.is_null() {
        if (*cur).next == fi {
            (*cur).next = (*fi).next;
            return;
        }
        cur = (*cur).next;
    }

    ERR!("seh:frame not found, native crashes in this case\n");
}
#[no_mangle]
pub unsafe extern "C" fn _IsExceptionObjectToBeDestroyed(obj: *const core::ffi::c_void) -> i32 {
    let mut cur: *mut frame_info = (*msvcrt_get_thread_data()).frame_info_head;

    while !cur.is_null() {
        if (*cur).object as *const c_void == obj {
            return (FALSE) as i32;
        }
        cur = (*cur).next;
    }

    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn __DestructExceptionObject(rec: PEXCEPTION_RECORD) {
    let info = (*rec).ExceptionInformation[2] as *mut cxx_exception_type;
    let object = (*rec).ExceptionInformation[1] as *mut core::ffi::c_void;

    if is_cxx_exception(rec) == 0 {
        return;
    }

    if info.is_null() || !rtti_is_set((*info).destructor) {
        return;
    }

    call_dtor(
        rtti_handler((*info).destructor, ((*rec).ExceptionInformation[3]) as usize),
        object,
    );
}
#[no_mangle]
pub unsafe extern "C" fn __CxxRegisterExceptionObject(
    ep: *mut EXCEPTION_POINTERS,
    frame_info: *mut cxx_frame_info,
) -> BOOL {
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    if ep.is_null() || (*ep).ExceptionRecord.is_null() {
        (*frame_info).rec = ((-1 as isize) as *mut c_void).cast();
        (*frame_info).context = ((-1 as isize) as *mut c_void).cast();
        return (TRUE) as i32;
    }

    (*frame_info).rec = (*data).exc_record;
    (*frame_info).context = (*data).ctx_record;
    (*data).exc_record = (*ep).ExceptionRecord;
    (*data).ctx_record = (*ep).ContextRecord;
    _CreateFrameInfo(
        core::ptr::addr_of_mut!((*frame_info).frame_info),
        (*(*ep).ExceptionRecord).ExceptionInformation[1] as *mut c_void,
    );
    return (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn __CxxUnregisterExceptionObject(frame_info: *mut cxx_frame_info, in_use: i32) {
    let data = msvcrt_get_thread_data();

    if (*frame_info).rec == (-1 as isize) as *mut core::ffi::c_void as PEXCEPTION_RECORD {
        return;
    }

    _FindAndUnlinkFrame(core::ptr::addr_of_mut!((*frame_info).frame_info));
    if (*(*data).exc_record).ExceptionCode == CXX_EXCEPTION && in_use == 0
            && _IsExceptionObjectToBeDestroyed((*(*data).exc_record).ExceptionInformation[1] as *const core::ffi::c_void) != 0 {
        __DestructExceptionObject((*data).exc_record);
    }
    (*data).exc_record = (*frame_info).rec;
    (*data).ctx_record = (*frame_info).context;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll exception handling
//  *
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2005 Juan Lang
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
//  *
//  * FIXME: Incomplete support for nested exceptions/try block cleanup.
//  */
//
// #include <float.h>
// #include <signal.h>
// #include <stdarg.h>
// #include <stdbool.h>
//
// #include "ntstatus.h"
// #define WIN32_NO_STATUS
// #include "windef.h"
// #include "winbase.h"
// #include "winternl.h"
// #include "msvcrt.h"
// #include "excpt.h"
// #include "wincon.h"
// #include "wine/exception.h"
// #include "wine/debug.h"
//
// #include "cppexcept.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(seh);
//
// #if _MSVCR_VER>=70 && _MSVCR_VER<=71
// static MSVCRT_security_error_handler security_error_handler;
// #endif
//
// static __sighandler_t sighandlers[NSIG] = { SIG_DFL };
//
// void dump_function_descr( const cxx_function_descr *descr, uintptr_t base )
// {
//     unwind_info *unwind_table = rtti_rva( descr->unwind_table, base );
//     tryblock_info *tryblock = rtti_rva( descr->tryblock, base );
//     ipmap_info *ipmap = rtti_rva( descr->ipmap, base );
//     UINT i, j;
//
//     TRACE( "magic %x\n", descr->magic );
//     TRACE( "unwind table: %p %d\n", unwind_table, descr->unwind_count );
//     for (i = 0; i < descr->unwind_count; i++)
//     {
//         TRACE("    %d: prev %d func %p\n", i, unwind_table[i].prev,
//               unwind_table[i].handler ? rtti_rva( unwind_table[i].handler, base ) : NULL );
//     }
//     TRACE( "try table: %p %d\n", tryblock, descr->tryblock_count );
//     for (i = 0; i < descr->tryblock_count; i++)
//     {
//         catchblock_info *catchblock = rtti_rva( tryblock[i].catchblock, base );
//
//         TRACE( "    %d: start %d end %d catchlevel %d catch %p %d\n", i,
//                tryblock[i].start_level, tryblock[i].end_level,
//                tryblock[i].catch_level, catchblock, tryblock[i].catchblock_count);
//         for (j = 0; j < tryblock[i].catchblock_count; j++)
//         {
//             type_info *type_info = catchblock[j].type_info ? rtti_rva( catchblock[j].type_info, base ) : NULL;
//             TRACE( "        %d: flags %x offset %d handler %p",
//                    j, catchblock[j].flags, catchblock[j].offset,
//                    catchblock[j].handler ? rtti_rva(catchblock[j].handler, base) : NULL );
// #ifdef _WIN64
//             TRACE( " frame %x", catchblock[j].frame );
// #endif
//             TRACE( " type %p %s\n", type_info, dbgstr_type_info(type_info) );
//         }
//     }
//     TRACE( "ipmap: %p %d\n", ipmap, descr->ipmap_count );
//     for (i = 0; i < descr->ipmap_count; i++)
//         TRACE( "    %d: ip %x state %d\n", i, ipmap[i].ip, ipmap[i].state );
// #ifdef RTTI_USE_RVA
//     TRACE( "unwind_help %+d\n", descr->unwind_help );
// #endif
//     if (descr->magic <= CXX_FRAME_MAGIC_VC6) return;
//     TRACE( "expect list: %p\n", rtti_rva( descr->expect_list, base ) );
//     if (descr->magic <= CXX_FRAME_MAGIC_VC7) return;
//     TRACE( "flags: %08x\n", descr->flags );
// }
//
// void *find_catch_handler( void *object, uintptr_t frame, uintptr_t exc_base,
//                           const tryblock_info *tryblock,
//                           cxx_exception_type *exc_type, uintptr_t image_base )
// {
//     unsigned int i;
//     const catchblock_info *catchblock = rtti_rva( tryblock->catchblock, image_base );
//     const cxx_type_info *type;
//     const type_info *catch_ti;
//
//     for (i = 0; i < tryblock->catchblock_count; i++)
//     {
//         if (exc_type)
//         {
//             catch_ti = catchblock[i].type_info ? rtti_rva( catchblock[i].type_info, image_base ) : NULL;
//             type = find_caught_type( exc_type, exc_base, catch_ti, catchblock[i].flags );
//             if (!type) continue;
//
//             TRACE( "matched type %p in catchblock %d\n", type, i );
//
//             if (catch_ti && catch_ti->mangled[0] && catchblock[i].offset)
//             {
//                 /* copy the exception to its destination on the stack */
//                 void **dest = (void **)(frame + catchblock[i].offset);
//                 copy_exception( object, dest, catchblock[i].flags, type, exc_base );
//             }
//         }
//         else
//         {
//             /* no CXX_EXCEPTION only proceed with a catch(...) block*/
//             if (catchblock[i].type_info) continue;
//             TRACE( "found catch(...) block\n" );
//         }
//         return rtti_rva( catchblock[i].handler, image_base );
//     }
//     return NULL;
// }
//
// #ifndef __i386__  /* i386 implementation is in except_i386.c */
//
// typedef struct
// {
//     cxx_frame_info frame_info;
//     BOOL rethrow;
//     EXCEPTION_RECORD *prev_rec;
// } cxx_catch_ctx;
//
// typedef struct
// {
//     ULONG_PTR dest_frame;
//     ULONG_PTR orig_frame;
//     EXCEPTION_RECORD *seh_rec;
//     DISPATCHER_CONTEXT *dispatch;
//     const cxx_function_descr *descr;
// } se_translator_ctx;
//
// static inline int ip_to_state( const cxx_function_descr *descr, uintptr_t ip, uintptr_t base )
// {
//     const ipmap_info *ipmap = rtti_rva( descr->ipmap, base );
//     unsigned int i;
//     int ret;
//
//     for (i = 0; i < descr->ipmap_count; i++) if (base + ipmap[i].ip > ip) break;
//     ret = i ? ipmap[i - 1].state : -1;
//     TRACE( "%Ix -> %d\n", ip, ret );
//     return ret;
// }
//
// static void cxx_local_unwind(ULONG_PTR frame, DISPATCHER_CONTEXT *dispatch,
//                              const cxx_function_descr *descr, int last_level)
// {
//     const unwind_info *unwind_table = rtti_rva(descr->unwind_table, dispatch->ImageBase);
//     int *unwind_help = (int *)(frame + descr->unwind_help);
//     int trylevel = unwind_help[0];
//
//     if (trylevel == -2) trylevel = ip_to_state( descr, get_exception_pc(dispatch), dispatch->ImageBase );
//
//     TRACE("current level: %d, last level: %d\n", trylevel, last_level);
//     while (trylevel > last_level)
//     {
//         if (trylevel<0 || trylevel>=descr->unwind_count)
//         {
//             ERR("invalid trylevel %d\n", trylevel);
//             terminate();
//         }
//         if (unwind_table[trylevel].handler)
//         {
//             void *handler = rtti_rva( unwind_table[trylevel].handler, dispatch->ImageBase );
//             call_unwind_handler( handler, frame, dispatch );
//         }
//         trylevel = unwind_table[trylevel].prev;
//     }
//     unwind_help[0] = trylevel;
// }
//
// static LONG CALLBACK cxx_rethrow_filter(PEXCEPTION_POINTERS eptrs, void *c)
// {
//     EXCEPTION_RECORD *rec = eptrs->ExceptionRecord;
//     cxx_catch_ctx *ctx = c;
//
//     if (rec->ExceptionCode != CXX_EXCEPTION)
//         return EXCEPTION_CONTINUE_SEARCH;
//     if (!rec->ExceptionInformation[1] && !rec->ExceptionInformation[2])
//         return EXCEPTION_EXECUTE_HANDLER;
//     if (rec->ExceptionInformation[1] == ctx->prev_rec->ExceptionInformation[1])
//         ctx->rethrow = TRUE;
//     return EXCEPTION_CONTINUE_SEARCH;
// }
//
// static void CALLBACK cxx_catch_cleanup(BOOL normal, void *c)
// {
//     cxx_catch_ctx *ctx = c;
//     __CxxUnregisterExceptionObject(&ctx->frame_info, ctx->rethrow);
// }
//
// static void* WINAPI call_catch_block(EXCEPTION_RECORD *rec)
// {
//     ULONG_PTR frame = rec->ExceptionInformation[1];
//     const cxx_function_descr *descr = (void*)rec->ExceptionInformation[2];
//     EXCEPTION_RECORD *untrans_rec = (void*)rec->ExceptionInformation[4];
//     EXCEPTION_RECORD *prev_rec = (void*)rec->ExceptionInformation[6];
//     CONTEXT *context = (void*)rec->ExceptionInformation[7];
//     int *unwind_help = (int *)(frame + descr->unwind_help);
//     EXCEPTION_POINTERS ep = { prev_rec, context };
//     cxx_catch_ctx ctx;
//     void *ret_addr = NULL;
//
//     ctx.rethrow = FALSE;
//     ctx.prev_rec = prev_rec;
//     __CxxRegisterExceptionObject(&ep, &ctx.frame_info);
//     msvcrt_get_thread_data()->processing_throw--;
//     __TRY
//     {
//         __TRY
//         {
//             ret_addr = call_catch_handler( rec );
//         }
//         __EXCEPT_CTX(cxx_rethrow_filter, &ctx)
//         {
//             TRACE("detect rethrow: exception code: %lx\n", prev_rec->ExceptionCode);
//             ctx.rethrow = TRUE;
//
//             if (untrans_rec)
//             {
//                 __DestructExceptionObject(prev_rec);
//                 RaiseException(untrans_rec->ExceptionCode, untrans_rec->ExceptionFlags,
//                         untrans_rec->NumberParameters, untrans_rec->ExceptionInformation);
//             }
//             else
//             {
//                 RaiseException(prev_rec->ExceptionCode, prev_rec->ExceptionFlags,
//                         prev_rec->NumberParameters, prev_rec->ExceptionInformation);
//             }
//         }
//         __ENDTRY
//     }
//     __FINALLY_CTX(cxx_catch_cleanup, &ctx)
//
//     unwind_help[0] = -2;
//     unwind_help[1] = -1;
//     return ret_addr;
// }
//
// static inline BOOL cxx_is_consolidate(const EXCEPTION_RECORD *rec)
// {
//     return (rec->ExceptionCode == STATUS_UNWIND_CONSOLIDATE &&
//             rec->NumberParameters > 10 &&
//             rec->ExceptionInformation[0] == (ULONG_PTR)call_catch_block);
// }
//
// static inline void find_catch_block(EXCEPTION_RECORD *rec, CONTEXT *context,
//                                     EXCEPTION_RECORD *untrans_rec,
//                                     ULONG_PTR frame, DISPATCHER_CONTEXT *dispatch,
//                                     const cxx_function_descr *descr,
//                                     cxx_exception_type *info, ULONG_PTR orig_frame)
// {
//     ULONG_PTR exc_base = (rec->NumberParameters == 4 ? rec->ExceptionInformation[3] : 0);
//     void *handler, *object = (void *)rec->ExceptionInformation[1];
//     int trylevel = ip_to_state( descr, get_exception_pc(dispatch), dispatch->ImageBase );
//     thread_data_t *data = msvcrt_get_thread_data();
//     const tryblock_info *in_catch;
//     EXCEPTION_RECORD catch_record;
//     CONTEXT ctx;
//     UINT i;
//     INT *unwind_help;
//
//     data->processing_throw++;
//     for (i=descr->tryblock_count; i>0; i--)
//     {
//         in_catch = rtti_rva(descr->tryblock, dispatch->ImageBase);
//         in_catch = &in_catch[i-1];
//
//         if (trylevel>in_catch->end_level && trylevel<=in_catch->catch_level)
//             break;
//     }
//     if (!i)
//         in_catch = NULL;
//
//     unwind_help = (int *)(orig_frame + descr->unwind_help);
//     if (trylevel > unwind_help[1])
//         unwind_help[0] = unwind_help[1] = trylevel;
//     else
//         trylevel = unwind_help[1];
//     TRACE("current trylevel: %d\n", trylevel);
//
//     for (i=0; i<descr->tryblock_count; i++)
//     {
//         const tryblock_info *tryblock = rtti_rva(descr->tryblock, dispatch->ImageBase);
//         tryblock = &tryblock[i];
//
//         if (trylevel < tryblock->start_level) continue;
//         if (trylevel > tryblock->end_level) continue;
//
//         if (in_catch)
//         {
//             if(tryblock->start_level <= in_catch->end_level) continue;
//             if(tryblock->end_level > in_catch->catch_level) continue;
//         }
//
//         handler = find_catch_handler( object, orig_frame, exc_base, tryblock, info, dispatch->ImageBase );
//         if (!handler) continue;
//
//         /* unwind stack and call catch */
//         memset(&catch_record, 0, sizeof(catch_record));
//         catch_record.ExceptionCode = STATUS_UNWIND_CONSOLIDATE;
//         catch_record.ExceptionFlags = EXCEPTION_NONCONTINUABLE;
//         catch_record.NumberParameters = 11;
//         catch_record.ExceptionInformation[0] = (ULONG_PTR)call_catch_block;
//         catch_record.ExceptionInformation[1] = orig_frame;
//         catch_record.ExceptionInformation[2] = (ULONG_PTR)descr;
//         catch_record.ExceptionInformation[3] = tryblock->start_level;
//         catch_record.ExceptionInformation[4] = (ULONG_PTR)untrans_rec;
//         catch_record.ExceptionInformation[5] = (ULONG_PTR)handler;
//         /* tyFlow expects ExceptionInformation[6] to contain exception record */
//         catch_record.ExceptionInformation[6] = (ULONG_PTR)rec;
//         catch_record.ExceptionInformation[7] = (ULONG_PTR)context;
//         catch_record.ExceptionInformation[10] = -1;
//         RtlUnwindEx((void*)frame, (void*)dispatch->ControlPc, &catch_record, NULL, &ctx, NULL);
//     }
//
//     TRACE("no matching catch block found\n");
//     data->processing_throw--;
// }
//
// static LONG CALLBACK se_translation_filter(EXCEPTION_POINTERS *ep, void *c)
// {
//     se_translator_ctx *ctx = (se_translator_ctx *)c;
//     EXCEPTION_RECORD *rec = ep->ExceptionRecord;
//     cxx_exception_type *exc_type;
//
//     if (rec->ExceptionCode != CXX_EXCEPTION)
//     {
//         TRACE("non-c++ exception thrown in SEH handler: %lx\n", rec->ExceptionCode);
//         terminate();
//     }
//
//     exc_type = (cxx_exception_type *)rec->ExceptionInformation[2];
//     find_catch_block(rec, ep->ContextRecord, ctx->seh_rec, ctx->dest_frame, ctx->dispatch,
//                      ctx->descr, exc_type, ctx->orig_frame);
//
//     __DestructExceptionObject(rec);
//     return ExceptionContinueSearch;
// }
//
// static void check_noexcept( PEXCEPTION_RECORD rec,
//         const cxx_function_descr *descr, BOOL nested )
// {
//     if (!nested && rec->ExceptionCode == CXX_EXCEPTION &&
//             descr->magic >= CXX_FRAME_MAGIC_VC8 &&
//             (descr->flags & FUNC_DESCR_NOEXCEPT))
//     {
//         ERR("noexcept function propagating exception\n");
//         terminate();
//     }
// }
//
// static DWORD cxx_frame_handler(EXCEPTION_RECORD *rec, ULONG_PTR frame,
//                                CONTEXT *context, DISPATCHER_CONTEXT *dispatch,
//                                const cxx_function_descr *descr)
// {
//     int trylevel = ip_to_state( descr, get_exception_pc(dispatch), dispatch->ImageBase );
//     cxx_exception_type *exc_type;
//     ULONG_PTR orig_frame = frame;
//     ULONG_PTR throw_base;
//     DWORD throw_func_off;
//     void *throw_func;
//     UINT i, j;
//     int unwindlevel = -1;
//
//     if (descr->magic<CXX_FRAME_MAGIC_VC6 || descr->magic>CXX_FRAME_MAGIC_VC8)
//     {
//         FIXME("unhandled frame magic %x\n", descr->magic);
//         return ExceptionContinueSearch;
//     }
//
//     if (descr->magic >= CXX_FRAME_MAGIC_VC8 &&
//         (descr->flags & FUNC_DESCR_SYNCHRONOUS) &&
//         (rec->ExceptionCode != CXX_EXCEPTION &&
//         !cxx_is_consolidate(rec) &&
//         rec->ExceptionCode != STATUS_LONGJUMP))
//         return ExceptionContinueSearch;  /* handle only c++ exceptions */
//
//     /* update orig_frame if it's a nested exception */
//     throw_func_off = RtlLookupFunctionEntry(dispatch->ControlPc, &throw_base, NULL)->BeginAddress;
//     throw_func = rtti_rva(throw_func_off, throw_base);
//     TRACE("reconstructed handler pointer: %p\n", throw_func);
//     for (i=descr->tryblock_count; i>0; i--)
//     {
//         const tryblock_info *tryblock = rtti_rva(descr->tryblock, dispatch->ImageBase);
//         tryblock = &tryblock[i-1];
//
//         if (trylevel>tryblock->end_level && trylevel<=tryblock->catch_level)
//         {
//             for (j=0; j<tryblock->catchblock_count; j++)
//             {
//                 const catchblock_info *catchblock = rtti_rva(tryblock->catchblock, dispatch->ImageBase);
//                 catchblock = &catchblock[j];
//
//                 if (rtti_rva(catchblock->handler, dispatch->ImageBase) == throw_func)
//                 {
//                     unwindlevel = tryblock->end_level;
// #ifdef _WIN64
//                     orig_frame = *(ULONG_PTR *)(frame + catchblock->frame);
// #else
//                     orig_frame = *(ULONG_PTR *)frame;
// #endif
//                     TRACE("nested exception detected, setting orig_frame to %Ix\n", orig_frame);
//                 }
//             }
//         }
//     }
//
//     if (rec->ExceptionFlags & (EXCEPTION_UNWINDING|EXCEPTION_EXIT_UNWIND))
//     {
//         if (rec->ExceptionFlags & EXCEPTION_TARGET_UNWIND)
//             cxx_local_unwind(orig_frame, dispatch, descr,
//                 cxx_is_consolidate(rec) ? rec->ExceptionInformation[3] : trylevel);
//         else
//             cxx_local_unwind(orig_frame, dispatch, descr, unwindlevel);
//         return ExceptionContinueSearch;
//     }
//     if (!descr->tryblock_count)
//     {
//         check_noexcept(rec, descr, orig_frame != frame);
//         return ExceptionContinueSearch;
//     }
//
//     if (rec->ExceptionCode == CXX_EXCEPTION &&
//         (!rec->ExceptionInformation[1] && !rec->ExceptionInformation[2]))
//     {
//         TRACE("rethrow detected.\n");
//         *rec = *msvcrt_get_thread_data()->exc_record;
//     }
//     if (rec->ExceptionCode == CXX_EXCEPTION)
//     {
//         exc_type = (cxx_exception_type *)rec->ExceptionInformation[2];
//
//         if (TRACE_ON(seh))
//         {
//             TRACE("handling C++ exception rec %p frame %Ix descr %p\n", rec, frame,  descr);
//             TRACE_EXCEPTION_TYPE(exc_type, rec->ExceptionInformation[3]);
//             dump_function_descr(descr, dispatch->ImageBase);
//         }
//     }
//     else
//     {
//         thread_data_t *data = msvcrt_get_thread_data();
//
//         exc_type = NULL;
//         TRACE("handling C exception code %lx rec %p frame %Ix descr %p\n",
//                 rec->ExceptionCode, rec, frame, descr);
//
//         if (data->se_translator) {
//             EXCEPTION_POINTERS except_ptrs;
//             se_translator_ctx ctx;
//
//             ctx.dest_frame = frame;
//             ctx.orig_frame = orig_frame;
//             ctx.seh_rec    = rec;
//             ctx.dispatch   = dispatch;
//             ctx.descr      = descr;
//             __TRY
//             {
//                 except_ptrs.ExceptionRecord = rec;
//                 except_ptrs.ContextRecord = context;
//                 data->se_translator(rec->ExceptionCode, &except_ptrs);
//             }
//             __EXCEPT_CTX(se_translation_filter, &ctx)
//             {
//             }
//             __ENDTRY
//         }
//     }
//
//     find_catch_block(rec, context, NULL, frame, dispatch, descr, exc_type, orig_frame);
//     check_noexcept(rec, descr, orig_frame != frame);
//     return ExceptionContinueSearch;
// }
//
// /*********************************************************************
//  *		__CxxFrameHandler (MSVCRT.@)
//  */
// EXCEPTION_DISPOSITION CDECL __CxxFrameHandler( EXCEPTION_RECORD *rec, ULONG_PTR frame,
//                                                CONTEXT *context, DISPATCHER_CONTEXT *dispatch )
// {
//     TRACE( "%p %Ix %p %p\n", rec, frame, context, dispatch );
//     return cxx_frame_handler( rec, frame, context, dispatch,
//                               rtti_rva(*(UINT *)dispatch->HandlerData, dispatch->ImageBase) );
// }
//
// #endif  /* __i386__ */
//
// static BOOL WINAPI msvcrt_console_handler(DWORD ctrlType)
// {
//     BOOL ret = FALSE;
//
//     switch (ctrlType)
//     {
//     case CTRL_C_EVENT:
//         if (sighandlers[SIGINT])
//         {
//             if (sighandlers[SIGINT] != SIG_IGN)
//                 sighandlers[SIGINT](SIGINT);
//             ret = TRUE;
//         }
//         break;
//     }
//     return ret;
// }
//
// #ifndef __UCRTSUPPORT__
// /*********************************************************************
//  *              __pxcptinfoptrs (MSVCRT.@)
//  */
// void** CDECL __pxcptinfoptrs(void)
// {
//     return (void**)&msvcrt_get_thread_data()->xcptinfo;
// }
// #endif /* !__UCRTSUPPORT__ */
//
// typedef void (CDECL *float_handler)(int, int);
//
// /* The exception codes are actually NTSTATUS values */
// static const struct
// {
//     NTSTATUS status;
//     int signal;
// } float_exception_map[] = {
//  { EXCEPTION_FLT_DENORMAL_OPERAND, _FPE_DENORMAL },
//  { EXCEPTION_FLT_DIVIDE_BY_ZERO, _FPE_ZERODIVIDE },
//  { EXCEPTION_FLT_INEXACT_RESULT, _FPE_INEXACT },
//  { EXCEPTION_FLT_INVALID_OPERATION, _FPE_INVALID },
//  { EXCEPTION_FLT_OVERFLOW, _FPE_OVERFLOW },
//  { EXCEPTION_FLT_STACK_CHECK, _FPE_STACKOVERFLOW },
//  { EXCEPTION_FLT_UNDERFLOW, _FPE_UNDERFLOW },
// };
//
// static LONG msvcrt_exception_filter(struct _EXCEPTION_POINTERS *except)
// {
//     LONG ret = EXCEPTION_CONTINUE_SEARCH;
//     __sighandler_t handler;
//
//     if (!except || !except->ExceptionRecord)
//         return EXCEPTION_CONTINUE_SEARCH;
//
//     switch (except->ExceptionRecord->ExceptionCode)
//     {
//     case EXCEPTION_ACCESS_VIOLATION:
//         if ((handler = sighandlers[SIGSEGV]) != SIG_DFL)
//         {
//             if (handler != SIG_IGN)
//             {
//                 EXCEPTION_POINTERS **ep = (EXCEPTION_POINTERS**)__pxcptinfoptrs(), *old_ep;
//
//                 old_ep = *ep;
//                 *ep = except;
//                 sighandlers[SIGSEGV] = SIG_DFL;
//                 handler(SIGSEGV);
//                 *ep = old_ep;
//             }
//             ret = EXCEPTION_CONTINUE_EXECUTION;
//         }
//         break;
//     /* According to msdn,
//      * the FPE signal handler takes as a second argument the type of
//      * floating point exception.
//      */
//     case EXCEPTION_FLT_DENORMAL_OPERAND:
//     case EXCEPTION_FLT_DIVIDE_BY_ZERO:
//     case EXCEPTION_FLT_INEXACT_RESULT:
//     case EXCEPTION_FLT_INVALID_OPERATION:
//     case EXCEPTION_FLT_OVERFLOW:
//     case EXCEPTION_FLT_STACK_CHECK:
//     case EXCEPTION_FLT_UNDERFLOW:
//         if ((handler = sighandlers[SIGFPE]) != SIG_DFL)
//         {
//             if (handler != SIG_IGN)
//             {
//                 EXCEPTION_POINTERS **ep = (EXCEPTION_POINTERS**)__pxcptinfoptrs(), *old_ep;
//                 unsigned int i;
//                 int float_signal = _FPE_INVALID;
//
//                 sighandlers[SIGFPE] = SIG_DFL;
//                 for (i = 0; i < ARRAY_SIZE(float_exception_map); i++)
//                 {
//                     if (float_exception_map[i].status ==
//                         except->ExceptionRecord->ExceptionCode)
//                     {
//                         float_signal = float_exception_map[i].signal;
//                         break;
//                     }
//                 }
//
//                 old_ep = *ep;
//                 *ep = except;
//                 ((float_handler)handler)(SIGFPE, float_signal);
//                 *ep = old_ep;
//             }
//             ret = EXCEPTION_CONTINUE_EXECUTION;
//         }
//         break;
//     case EXCEPTION_ILLEGAL_INSTRUCTION:
//     case EXCEPTION_PRIV_INSTRUCTION:
//         if ((handler = sighandlers[SIGILL]) != SIG_DFL)
//         {
//             if (handler != SIG_IGN)
//             {
//                 EXCEPTION_POINTERS **ep = (EXCEPTION_POINTERS**)__pxcptinfoptrs(), *old_ep;
//
//                 old_ep = *ep;
//                 *ep = except;
//                 sighandlers[SIGILL] = SIG_DFL;
//                 handler(SIGILL);
//                 *ep = old_ep;
//             }
//             ret = EXCEPTION_CONTINUE_EXECUTION;
//         }
//         break;
//     }
//     return ret;
// }
//
// void msvcrt_init_signals(void)
// {
//     SetConsoleCtrlHandler(msvcrt_console_handler, TRUE);
// }
//
// void msvcrt_free_signals(void)
// {
//     SetConsoleCtrlHandler(msvcrt_console_handler, FALSE);
// }
//
// #ifndef __UCRTSUPPORT__
// /*********************************************************************
//  *		signal (MSVCRT.@)
//  * Some signals may never be generated except through an explicit call to
//  * raise.
//  */
// __sighandler_t CDECL signal(int sig, __sighandler_t func)
// {
//     __sighandler_t ret = SIG_ERR;
//
//     TRACE("(%d, %p)\n", sig, func);
//
//     if (func == SIG_ERR) return SIG_ERR;
//
//     switch (sig)
//     {
//     /* Cases handled internally.  Note SIGTERM is never generated by Windows,
//      * so we effectively mask it.
//      */
//     case SIGABRT:
//     case SIGFPE:
//     case SIGILL:
//     case SIGSEGV:
//     case SIGINT:
//     case SIGTERM:
//     case SIGBREAK:
//         ret = sighandlers[sig];
//         sighandlers[sig] = func;
//         break;
//     default:
//         ret = SIG_ERR;
//     }
//     return ret;
// }
//
// /*********************************************************************
//  *		raise (MSVCRT.@)
//  */
// int CDECL raise(int sig)
// {
//     __sighandler_t handler;
//
//     TRACE("(%d)\n", sig);
//
//     switch (sig)
//     {
//     case SIGFPE:
//     case SIGILL:
//     case SIGSEGV:
//         handler = sighandlers[sig];
//         if (handler == SIG_DFL) _exit(3);
//         if (handler != SIG_IGN)
//         {
//             EXCEPTION_POINTERS **ep = (EXCEPTION_POINTERS**)__pxcptinfoptrs(), *old_ep;
//
//             sighandlers[sig] = SIG_DFL;
//
//             old_ep = *ep;
//             *ep = NULL;
//             if (sig == SIGFPE)
//                 ((float_handler)handler)(sig, _FPE_EXPLICITGEN);
//             else
//                 handler(sig);
//             *ep = old_ep;
//         }
//         break;
//     case SIGABRT:
//     case SIGINT:
//     case SIGTERM:
//     case SIGBREAK:
//         handler = sighandlers[sig];
//         if (handler == SIG_DFL) _exit(3);
//         if (handler != SIG_IGN)
//         {
//             sighandlers[sig] = SIG_DFL;
//             handler(sig);
//         }
//         break;
//     default:
//         return -1;
//     }
//     return 0;
// }
// #endif /* !__UCRTSUPPORT__ */
//
// /*********************************************************************
//  *		_XcptFilter (MSVCRT.@)
//  */
// int CDECL _XcptFilter(NTSTATUS ex, PEXCEPTION_POINTERS ptr)
// {
//     TRACE("(%08lx,%p)\n", ex, ptr);
//     /* I assume ptr->ExceptionRecord->ExceptionCode is the same as ex */
//     return msvcrt_exception_filter(ptr);
// }
//
// /*********************************************************************
//  *		__CppXcptFilter (MSVCRT.@)
//  */
// int CDECL __CppXcptFilter(NTSTATUS ex, PEXCEPTION_POINTERS ptr)
// {
//     /* only filter c++ exceptions */
//     if (ex != CXX_EXCEPTION) return EXCEPTION_CONTINUE_SEARCH;
//     return _XcptFilter(ex, ptr);
// }
//
// /*********************************************************************
//  *		__CxxDetectRethrow (MSVCRT.@)
//  */
// BOOL CDECL __CxxDetectRethrow(PEXCEPTION_POINTERS ptrs)
// {
//     PEXCEPTION_RECORD rec;
//
//     if (!ptrs)
//         return FALSE;
//
//     rec = ptrs->ExceptionRecord;
//     if (is_cxx_exception( rec ) && rec->ExceptionInformation[2])
//     {
//         ptrs->ExceptionRecord = msvcrt_get_thread_data()->exc_record;
//         return TRUE;
//     }
//     return (msvcrt_get_thread_data()->exc_record == rec);
// }
//
// /*********************************************************************
//  *		__CxxExceptionFilter (MSVCRT.@)
//  */
// int CDECL __CxxExceptionFilter( EXCEPTION_POINTERS *ptrs, const type_info *ti, UINT flags, void **copy)
// {
//     const cxx_type_info *type;
//     EXCEPTION_RECORD *rec;
//     uintptr_t exc_base;
//
//     TRACE( "%p %p %x %p\n", ptrs, ti, flags, copy );
//
//     if (!ptrs) return EXCEPTION_CONTINUE_SEARCH;
//
//     /* handle catch(...) */
//     if (!ti) return EXCEPTION_EXECUTE_HANDLER;
//
//     rec = ptrs->ExceptionRecord;
//     if (!is_cxx_exception( rec )) return EXCEPTION_CONTINUE_SEARCH;
//
//     if (rec->ExceptionInformation[1] == 0 && rec->ExceptionInformation[2] == 0)
//     {
//         rec = msvcrt_get_thread_data()->exc_record;
//         if (!rec) return EXCEPTION_CONTINUE_SEARCH;
//     }
//
//     exc_base = rec->ExceptionInformation[3];
//     type = find_caught_type( (cxx_exception_type *)rec->ExceptionInformation[2], exc_base, ti, flags );
//     if (!type) return EXCEPTION_CONTINUE_SEARCH;
//
//     if (copy) copy_exception( (void *)rec->ExceptionInformation[1], copy, flags, type, exc_base );
//
//     return EXCEPTION_EXECUTE_HANDLER;
// }
//
// /*********************************************************************
//  *		__CxxQueryExceptionSize (MSVCRT.@)
//  */
// unsigned int CDECL __CxxQueryExceptionSize(void)
// {
//     return sizeof(cxx_exception_type);
// }
//
// /*********************************************************************
//  *		_abnormal_termination (MSVCRT.@)
//  */
// int CDECL __intrinsic_abnormal_termination(void)
// {
//   FIXME("(void)stub\n");
//   return 0;
// }
//
// /******************************************************************
//  *		__uncaught_exception (MSVCRT.@)
//  */
// BOOL CDECL __uncaught_exception(void)
// {
//     return msvcrt_get_thread_data()->processing_throw != 0;
// }
//
// /*********************************************************************
//  *              _fpieee_flt (MSVCRT.@)
//  */
// int __cdecl _fpieee_flt( __msvcrt_ulong code, EXCEPTION_POINTERS *ep,
//                          int (__cdecl *handler)(_FPIEEE_RECORD*) )
// {
//     switch (code)
//     {
//     case STATUS_FLOAT_DIVIDE_BY_ZERO:
//     case STATUS_FLOAT_INEXACT_RESULT:
//     case STATUS_FLOAT_INVALID_OPERATION:
//     case STATUS_FLOAT_OVERFLOW:
//     case STATUS_FLOAT_UNDERFLOW:
//         return handle_fpieee_flt( code, ep, handler );
//     default:
//         return EXCEPTION_CONTINUE_SEARCH;
//     }
// }
//
// #if _MSVCR_VER>=70 && _MSVCR_VER<=71
//
// /*********************************************************************
//  *		_set_security_error_handler (MSVCR70.@)
//  */
// MSVCRT_security_error_handler CDECL _set_security_error_handler(
//     MSVCRT_security_error_handler handler )
// {
//     MSVCRT_security_error_handler old = security_error_handler;
//
//     TRACE("(%p)\n", handler);
//
//     security_error_handler = handler;
//     return old;
// }
//
// /*********************************************************************
//  *		__security_error_handler (MSVCR70.@)
//  */
// void CDECL __security_error_handler(int code, void *data)
// {
//     if(security_error_handler)
//         security_error_handler(code, data);
//     else
//         FIXME("(%d, %p) stub\n", code, data);
//
//     _exit(3);
// }
//
// #endif /* _MSVCR_VER>=70 && _MSVCR_VER<=71 */
//
// #if _MSVCR_VER>=110
// /*********************************************************************
//  *  __crtSetUnhandledExceptionFilter (MSVCR110.@)
//  */
// LPTOP_LEVEL_EXCEPTION_FILTER CDECL __crtSetUnhandledExceptionFilter(LPTOP_LEVEL_EXCEPTION_FILTER filter)
// {
//     return SetUnhandledExceptionFilter(filter);
// }
// #endif
//
// /*********************************************************************
//  * _CreateFrameInfo (MSVCR80.@)
//  */
// frame_info* CDECL _CreateFrameInfo(frame_info *fi, void *obj)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     TRACE("(%p, %p)\n", fi, obj);
//
//     fi->next = data->frame_info_head;
//     data->frame_info_head = fi;
//     fi->object = obj;
//     return fi;
// }
//
// /*********************************************************************
//  * _FindAndUnlinkFrame (MSVCR80.@)
//  */
// void CDECL _FindAndUnlinkFrame(frame_info *fi)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//     frame_info *cur = data->frame_info_head;
//
//     TRACE("(%p)\n", fi);
//
//     if (cur == fi)
//     {
//         data->frame_info_head = cur->next;
//         return;
//     }
//
//     for (; cur->next; cur = cur->next)
//     {
//         if (cur->next == fi)
//         {
//             cur->next = fi->next;
//             return;
//         }
//     }
//
//     ERR("frame not found, native crashes in this case\n");
// }
//
// /*********************************************************************
//  *              _IsExceptionObjectToBeDestroyed (MSVCR80.@)
//  */
// BOOL __cdecl _IsExceptionObjectToBeDestroyed(const void *obj)
// {
//     frame_info *cur;
//
//     TRACE( "%p\n", obj );
//
//     for (cur = msvcrt_get_thread_data()->frame_info_head; cur; cur = cur->next)
//     {
//         if (cur->object == obj)
//             return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*********************************************************************
//  * __DestructExceptionObject (MSVCRT.@)
//  */
// void CDECL __DestructExceptionObject(EXCEPTION_RECORD *rec)
// {
//     cxx_exception_type *info = (cxx_exception_type*) rec->ExceptionInformation[2];
//     void *object = (void*)rec->ExceptionInformation[1];
//
//     TRACE("(%p)\n", rec);
//
//     if (!is_cxx_exception( rec )) return;
//
//     if (!info || !info->destructor)
//         return;
//
//     call_dtor( rtti_rva( info->destructor, rec->ExceptionInformation[3] ), object );
// }
//
// /*********************************************************************
//  *  __CxxRegisterExceptionObject (MSVCRT.@)
//  */
// BOOL CDECL __CxxRegisterExceptionObject(EXCEPTION_POINTERS *ep, cxx_frame_info *frame_info)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     TRACE("(%p, %p)\n", ep, frame_info);
//
//     if (!ep || !ep->ExceptionRecord)
//     {
//         frame_info->rec = (void*)-1;
//         frame_info->context = (void*)-1;
//         return TRUE;
//     }
//
//     frame_info->rec = data->exc_record;
//     frame_info->context = data->ctx_record;
//     data->exc_record = ep->ExceptionRecord;
//     data->ctx_record = ep->ContextRecord;
//     _CreateFrameInfo(&frame_info->frame_info, (void*)ep->ExceptionRecord->ExceptionInformation[1]);
//     return TRUE;
// }
//
// /*********************************************************************
//  *  __CxxUnregisterExceptionObject (MSVCRT.@)
//  */
// void CDECL __CxxUnregisterExceptionObject(cxx_frame_info *frame_info, BOOL in_use)
// {
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     TRACE("(%p)\n", frame_info);
//
//     if(frame_info->rec == (void*)-1)
//         return;
//
//     _FindAndUnlinkFrame(&frame_info->frame_info);
//     if(data->exc_record->ExceptionCode == CXX_EXCEPTION && !in_use
//             && _IsExceptionObjectToBeDestroyed((void*)data->exc_record->ExceptionInformation[1]))
//         __DestructExceptionObject(data->exc_record);
//     data->exc_record = frame_info->rec;
//     data->ctx_record = frame_info->context;
// }
//
// struct __std_exception_data {
//     char *what;
//     char dofree;
// };
//
// #if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)
//
// /*********************************************************************
//  *  __std_exception_copy (UCRTBASE.@)
//  */
// void CDECL __std_exception_copy(const struct __std_exception_data *src,
//                                        struct __std_exception_data *dst)
// {
//     TRACE("(%p %p)\n", src, dst);
//
//     if(src->dofree && src->what) {
//         dst->what   = _strdup(src->what);
//         dst->dofree = 1;
//     } else {
//         dst->what   = src->what;
//         dst->dofree = 0;
//     }
// }
//
// /*********************************************************************
//  *  __std_exception_destroy (UCRTBASE.@)
//  */
// void CDECL __std_exception_destroy(struct __std_exception_data *data)
// {
//     TRACE("(%p)\n", data);
//
//     if(data->dofree)
//         free(data->what);
//     data->what   = NULL;
//     data->dofree = 0;
// }
//
// /*********************************************************************
//  *  __current_exception (UCRTBASE.@)
//  */
// void** CDECL __current_exception(void)
// {
//     TRACE("()\n");
//     return (void**)&msvcrt_get_thread_data()->exc_record;
// }
//
// /*********************************************************************
//  *  __current_exception_context (UCRTBASE.@)
//  */
// void** CDECL __current_exception_context(void)
// {
//     TRACE("()\n");
//     return (void**)&msvcrt_get_thread_data()->ctx_record;
// }
//
// /*********************************************************************
//  *  __processing_throw (UCRTBASE.@)
//  */
// int* CDECL __processing_throw(void)
// {
//     TRACE("()\n");
//     return &msvcrt_get_thread_data()->processing_throw;
// }
//
// #endif /* _MSVCR_VER>=140 */
//

// ── the `__UCRTSUPPORT__` half of except.c ───────────────────────────────────────────────
//
// Five bodies inside `#if _MSVCR_VER>=140 || defined(__UCRTSUPPORT__)`. msvcrt compiles this
// TU at `_MSVCR_VER = 0` without the macro, so none of them is built there and none had a
// Rust body until ucrtbase's cohab needed them; ucrtbase.spec exports all five.
//
// rustconv translation (2026-09-05, --seal-facts --seal-calls --seal-wide, $0.02). Each was
// diffed against the C by hand afterwards -- they are short enough that the whole body is the
// review, and sealing's blind spot is a body that keeps SOME sealed tokens and permutes the
// rest, which no audit flags.

// `__std_exception_data` is NOT redefined here: the C declares it at line 1013, ABOVE the
// `#if`, so it is in both configurations and the earlier translation already carries it. The
// compiler caught the duplicate, which is the cheap way round -- a second `#[repr(C)]` copy
// that drifted from the first would not have been caught by anything.

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_exception_copy(
    src: *const __std_exception_data,
    dst: *mut __std_exception_data,
) {
    if (*src).dofree != 0 && !(*src).what.is_null() {
        (*dst).what = _strdup((*src).what);
        (*dst).dofree = 1;
    } else {
        (*dst).what = (*src).what;
        (*dst).dofree = 0;
    }
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __std_exception_destroy(data: *mut __std_exception_data) {
    if (*data).dofree != 0 {
        free((*data).what as *mut core::ffi::c_void);
    }
    (*data).what = core::ptr::null_mut();
    (*data).dofree = 0;
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __current_exception() -> *mut *mut core::ffi::c_void {
    core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).exc_record) as *mut *mut core::ffi::c_void
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __current_exception_context() -> *mut *mut core::ffi::c_void {
    core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).ctx_record) as *mut *mut core::ffi::c_void
}

#[cfg(feature = "ucrtsupport")]
#[no_mangle]
pub unsafe extern "C" fn __processing_throw() -> *mut i32 {
    core::ptr::addr_of_mut!((*msvcrt_get_thread_data()).processing_throw)
}
