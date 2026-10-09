//! except_i386.c — 32-bit SEH, the other half of the arch gate in `except.rs`.
//!
//! On i386 an exception frame is a node in a linked list rooted at `fs:0`, and a function's
//! try blocks are found by walking that list; on amd64/arm64 the unwinder looks the frame up
//! by PC in a table. That is why `except.c` puts everything from `ip_to_state` to
//! `__CxxFrameHandler` inside `#ifndef __i386__` and this whole file inside `#ifdef __i386__`
//! — they are two implementations of the same exports, not two halves of one.
//!
//! The asm thunks (`call_handler`, `continue_after_catch`, …) and wine/exception.h's
//! `__wine_push_frame`/`__wine_pop_frame` live in the companion `seh_i386` module: they have
//! no C text, only macro expansions and `static inline`s.

#![cfg(target_arch = "x86")]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

// rustconv-assembly: whole-tu
use crate::prelude::*;
use msvcrt_sys::*;
// Disambiguate the two globs' `FALSE` (prelude's is a `u8`, the island's the Win32 `BOOL`);
// every use in this file is the BOOL.
use msvcrt_sys::FALSE;
// The module compiles with `-DRtlUnwind=__wine_RtlUnwind`, and the substitute is msvcrt's own
// `i386/__wine_RtlUnwind.s`. Aliasing rather than renaming the call sites keeps them reading
// as the C reads, and keeps the fact that they are NOT ntdll's RtlUnwind in one place.
use msvcrt_sys::__wine_RtlUnwind as RtlUnwind;
// The parts of this TU that are macro expansions or header inlines, so have no C text.
use crate::seh_i386::{
    call_filter, call_finally_block, call_handler, continue_after_catch, NtCurrentTeb,
    __wine_pop_frame, __wine_push_frame,
};
// The siblings this file calls, all already Rust. `find_catch_handler` and
// `dump_function_descr` sit ABOVE except.c's `#ifndef __i386__` gate precisely so both arch
// implementations can call them; the rest is cpp.c's exception-object registry.
use crate::cpp::terminate;
use crate::except::{
    dump_function_descr, find_catch_handler, _CreateFrameInfo, __CxxUnregisterExceptionObject,
    __DestructExceptionObject,
};
use core::ffi::c_void;

// Off by default upstream, so a body here would print what the C does not. `TRACE_ON!(seh)`
// is the guard the C puts in front of the expensive ones and is false for the same reason.
macro_rules! TRACE_ON {
    ($($arg:tt)*) => {
        0
    };
}
macro_rules! TRACE_EXCEPTION_TYPE {
    ($($arg:tt)*) => {{}};
}

// except_i386.c's always-on wine channels, spelled exactly as `except.rs` spells them: the
// message including wine's prefix is a `c"…"` literal at the call site, so the macro bodies
// carry no literal of their own and the literal-multiset diff against the C stays honest.
// TRACE and WARN get no body — they are off by default upstream, so dropping them loses
// nothing upstream would have printed.

unsafe extern "C" {
    /// `sdk/lib/pseh/i386/seh.s`. The ReactOS arm of except_i386.c only DECLARES it — the
    /// wine body next to that declaration is `#ifndef __REACTOS__` and never compiled.
    fn _global_unwind2(frame: *mut EXCEPTION_REGISTRATION_RECORD);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct __cxx_exception_frame {
    pub frame: EXCEPTION_REGISTRATION_RECORD,
    pub trylevel: i32,
    pub ebp: u32,
}

pub type cxx_exception_frame = __cxx_exception_frame;

#[repr(C)]
pub struct catch_func_nested_frame {
    pub frame: EXCEPTION_REGISTRATION_RECORD,
    pub cxx_frame: *mut cxx_exception_frame,
    pub descr: *const cxx_function_descr,
    pub trylevel: i32,
    pub frame_info: cxx_frame_info,
}

#[repr(C)]
pub struct se_translator_ctx {
    pub frame: *mut cxx_exception_frame,
    pub descr: *const cxx_function_descr,
    pub nested_frame: *mut catch_func_nested_frame,
}

#[repr(C)]
pub struct _SCOPETABLE {
    pub previousTryLevel: i32,
    pub lpfnFilter: Option<unsafe extern "C" fn(PEXCEPTION_POINTERS) -> i32>,
    pub lpfnHandler: Option<unsafe extern "C" fn() -> *mut core::ffi::c_void>,
}

pub type SCOPETABLE = _SCOPETABLE;
pub type PSCOPETABLE = *mut _SCOPETABLE;

#[repr(C)]
pub struct MSVCRT_EXCEPTION_FRAME {
    pub prev: *mut EXCEPTION_REGISTRATION_RECORD,
    pub handler: Option<unsafe extern "C" fn(
        PEXCEPTION_RECORD,
        *mut EXCEPTION_REGISTRATION_RECORD,
        PCONTEXT,
        PEXCEPTION_RECORD,
    )>,
    pub scopetable: PSCOPETABLE,
    pub trylevel: i32,
    pub _ebp: i32,
    pub xpointers: PEXCEPTION_POINTERS,
}

#[repr(C)]
pub struct SCOPETABLE_V4 {
    pub gs_cookie_offset: i32,
    pub gs_cookie_xor: u32,
    pub eh_cookie_offset: i32,
    pub eh_cookie_xor: u32,
    /// `SCOPETABLE entries[1]` in the C — the flexible-array idiom, not a one-element
    /// array. The compiler emits as many entries as the function has `__try` levels, and
    /// every reader indexes past the declared bound.
    pub entries: [SCOPETABLE; 1],
}

/// `scope_table->entries[i]`, as the C means it.
///
/// A plain `entries[i as usize]` is a Rust ARRAY index against the declared length of 1,
/// and that bounds check is not a debug-only one — so the shipping i386 DLL panicked inside
/// its own SEH handler for any function with more than one `__try` level. `.offset()` on
/// the first element is what the C compiles to.
#[inline(always)]
unsafe fn scope_entry(table: *const SCOPETABLE_V4, index: i32) -> *const SCOPETABLE {
    (*table).entries.as_ptr().offset(index as isize)
}

pub const TRYLEVEL_END: i32 = -1;

pub type cxx_exc_custom_handler = Option<unsafe extern "C" fn(
    PEXCEPTION_RECORD,
    *mut cxx_exception_frame,
    PCONTEXT,
    *mut *mut EXCEPTION_REGISTRATION_RECORD,
    *const cxx_function_descr,
    i32,
    *mut EXCEPTION_REGISTRATION_RECORD,
    u32,
) -> u32>;
unsafe extern "C" fn cxx_local_unwind(
    frame: *mut cxx_exception_frame,
    descr: *const cxx_function_descr,
    last_level: i32,
) {
    let mut handler: Option<unsafe extern "C" fn() -> *mut core::ffi::c_void>;
    let mut trylevel: i32 = (*frame).trylevel;

    while trylevel != last_level {
        if trylevel < 0 || trylevel >= (*descr).unwind_count as i32 {
            ERR!("seh:invalid trylevel %d
", trylevel);
            terminate();
        }
        handler = (*descr).unwind_table.offset(trylevel as isize).read().handler;
        if handler.is_some() {
            call_handler(handler, core::ptr::addr_of_mut!((*frame).ebp).cast());
        }
        trylevel = (*descr).unwind_table.offset(trylevel as isize).read().prev;
    }
    (*frame).trylevel = last_level;
}

unsafe extern "C" fn catch_function_nested_handler(
    rec: PEXCEPTION_RECORD,
    frame: *mut EXCEPTION_REGISTRATION_RECORD,
    context: PCONTEXT,
    dispatcher: *mut *mut EXCEPTION_REGISTRATION_RECORD,
) -> u32 {
    let nested_frame = frame as *mut catch_func_nested_frame;

    if (*rec).ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND) != 0 {
        __CxxUnregisterExceptionObject(
            core::ptr::addr_of_mut!((*nested_frame).frame_info),
            FALSE as i32,
        );
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }

    if (*rec).ExceptionCode == CXX_EXCEPTION {
        let prev_rec = (*msvcrt_get_thread_data()).exc_record;

        if ((*rec).ExceptionInformation[1] == 0
            && (*rec).ExceptionInformation[2] == 0)
            || ((*prev_rec).ExceptionCode == CXX_EXCEPTION
                && (*rec).ExceptionInformation[1]
                    == (*prev_rec).ExceptionInformation[1]
                && (*rec).ExceptionInformation[2]
                    == (*prev_rec).ExceptionInformation[2])
        {
            *rec = *prev_rec;
            (*rec).ExceptionFlags &= !EXCEPTION_UNWINDING;
            if TRACE_ON!(seh) != 0 {
                if (*rec).ExceptionCode == CXX_EXCEPTION {
                    TRACE!(
                        b"re-propagate: obj: %Ix, type: %Ix\n\0".as_ptr() as PCHAR,
                        (*rec).ExceptionInformation[1],
                        (*rec).ExceptionInformation[2],
                    );
                }
            }
        }
    }

    cxx_frame_handler(
        rec,
        (*nested_frame).cxx_frame,
        context,
        core::ptr::null_mut(),
        (*nested_frame).descr,
        nested_frame,
    )
}
unsafe extern "C" fn call_catch_block(
    rec: PEXCEPTION_RECORD,
    context: PCONTEXT,
    frame: *mut cxx_exception_frame,
    descr: *const cxx_function_descr,
    catch_frame: *mut catch_func_nested_frame,
    info: *mut cxx_exception_type,
) {
    let mut i: u32;
    let mut addr: *mut core::ffi::c_void;
    let mut handler: *mut core::ffi::c_void;
    let object = (*rec).ExceptionInformation[1] as *mut core::ffi::c_void;
    let mut nested_frame: catch_func_nested_frame = core::mem::zeroed();
    let mut trylevel = (*frame).trylevel;
    let save_esp = *((frame as *mut u32).offset(-(1 as isize)));
    let data = msvcrt_get_thread_data();

    (*data).processing_throw += 1;
    i = 0;
    while i < (*descr).tryblock_count {
        let tryblock = &*(*descr).tryblock.offset(i as isize);

        if !catch_frame.is_null() && (*catch_frame).trylevel > tryblock.start_level {
            i += 1;
            continue;
        }

        if trylevel < tryblock.start_level {
            i += 1;
            continue;
        }
        if trylevel > tryblock.end_level {
            i += 1;
            continue;
        }

        handler = find_catch_handler(
            object,
            core::ptr::addr_of_mut!((*frame).ebp) as usize,
            0,
            tryblock,
            info,
            0,
        );
        if handler.is_null() {
            i += 1;
            continue;
        }

        _CreateFrameInfo(
            core::ptr::addr_of_mut!(nested_frame.frame_info.frame_info),
            object,
        );

        RtlUnwind(
            if !catch_frame.is_null() {
                core::ptr::addr_of_mut!((*catch_frame).frame).cast()
            } else {
                core::ptr::addr_of_mut!((*frame).frame).cast()
            },
            core::ptr::null_mut(),
            rec,
            core::ptr::null_mut(),
        );
        cxx_local_unwind(frame, descr, tryblock.start_level);
        (*frame).trylevel = tryblock.end_level + 1;

        nested_frame.frame_info.rec = (*data).exc_record;
        nested_frame.frame_info.context = (*data).ctx_record;
        (*data).exc_record = rec;
        (*data).ctx_record = context;
        (*data).processing_throw -= 1;

        nested_frame.frame.Handler = Some(catch_function_nested_handler);
        nested_frame.cxx_frame = frame;
        nested_frame.descr = descr;
        nested_frame.trylevel = tryblock.end_level + 1;

        __wine_push_frame(core::ptr::addr_of_mut!(nested_frame.frame));
        addr = call_handler(
            core::mem::transmute(handler),
            core::ptr::addr_of_mut!((*frame).ebp).cast(),
        );
        __wine_pop_frame(core::ptr::addr_of_mut!(nested_frame.frame));

        *((frame as *mut u32).offset(-(1 as isize))) = save_esp;
        __CxxUnregisterExceptionObject(
            core::ptr::addr_of_mut!(nested_frame.frame_info),
            FALSE as i32,
        );

        continue_after_catch((frame).cast(), addr);

        i += 1;
    }
    (*data).processing_throw -= 1;
}
// `LONG CALLBACK` — stdcall, and the only convention this file's cfg pair was ever about.
// rustconv emits such a pair for every callback; here the `not(x86)` twin was unreachable
// (the whole TU is `#ifdef __i386__`) and is deleted rather than left to read like an arch
// gate, which is the confusion that cost wave 29 an investigation.
unsafe extern "stdcall" fn se_translation_filter(
    ep: *mut EXCEPTION_POINTERS,
    c: *mut core::ffi::c_void,
) -> i32 {
    let ctx = c as *mut se_translator_ctx;
    let rec = (*ep).ExceptionRecord;
    let exc_type: *mut cxx_exception_type;

    if (*rec).ExceptionCode != CXX_EXCEPTION {
        terminate();
    }

    exc_type = (*rec).ExceptionInformation[2] as *mut cxx_exception_type;
    call_catch_block(
        rec,
        (*ep).ContextRecord,
        (*ctx).frame,
        (*ctx).descr,
        (*ctx).nested_frame,
        exc_type,
    );

    __DestructExceptionObject(rec);
    _EXCEPTION_DISPOSITION_ExceptionContinueSearch as i32
}

unsafe extern "C" fn check_noexcept(
    rec: PEXCEPTION_RECORD,
    descr: *const cxx_function_descr,
    nested: u8,
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
#[no_mangle]
pub unsafe extern "C" fn cxx_frame_handler(
    rec: PEXCEPTION_RECORD,
    frame: *mut cxx_exception_frame,
    context: PCONTEXT,
    dispatch: *mut *mut EXCEPTION_REGISTRATION_RECORD,
    descr: *const cxx_function_descr,
    nested_frame: *mut catch_func_nested_frame,
) -> u32 {
    let mut exc_type: *mut cxx_exception_type;

    if (*descr).magic() < CXX_FRAME_MAGIC_VC6 || (*descr).magic() > CXX_FRAME_MAGIC_VC8 {
        ERR!("seh:invalid frame magic %x
", (*descr).magic());
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }
    if (*descr).magic() >= CXX_FRAME_MAGIC_VC8
        && ((*descr).flags & FUNC_DESCR_SYNCHRONOUS) != 0
        && (*rec).ExceptionCode != CXX_EXCEPTION
    {
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }

    if ((*rec).ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND)) != 0 {
        if (*descr).unwind_count != 0 && nested_frame.is_null() {
            cxx_local_unwind(frame, descr, -1);
        }
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }
    if (*descr).tryblock_count == 0 {
        check_noexcept(rec, descr, if !nested_frame.is_null() { 1u8 } else { 0u8 });
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }

    if (*rec).ExceptionCode == CXX_EXCEPTION
        && (*rec).ExceptionInformation[1] == 0
        && (*rec).ExceptionInformation[2] == 0
    {
        *rec = *(*msvcrt_get_thread_data()).exc_record;
        (*rec).ExceptionFlags &= !EXCEPTION_UNWINDING;
        if TRACE_ON!(seh) != 0 {
            if (*rec).ExceptionCode == CXX_EXCEPTION {
                TRACE!(
                    b"re-propagate: obj: %Ix, type: %Ix\n\0".as_ptr() as *const i8,
                    (*rec).ExceptionInformation[1],
                    (*rec).ExceptionInformation[2],
                );
            }
        }
    }

    if (*rec).ExceptionCode == CXX_EXCEPTION {
        exc_type = (*rec).ExceptionInformation[2] as *mut cxx_exception_type;

        if (*rec).ExceptionInformation[0] > CXX_FRAME_MAGIC_VC8
            && !(*exc_type).custom_handler.is_null()
        {
            // `custom_handler` is a bare `void *` in the descriptor; the shape it is
            // called through is this file's own `cxx_exc_custom_handler` typedef.
            let handler: cxx_exc_custom_handler =
                core::mem::transmute((*exc_type).custom_handler);
            let handler = handler.unwrap_unchecked();
            return handler(
                rec,
                frame,
                context,
                dispatch,
                descr,
                if !nested_frame.is_null() {
                    (*nested_frame).trylevel
                } else {
                    0
                },
                if !nested_frame.is_null() {
                    core::ptr::addr_of_mut!((*nested_frame).frame)
                } else {
                    core::ptr::null_mut()
                },
                0,
            );
        }

        if TRACE_ON!(seh) != 0 {
            TRACE_EXCEPTION_TYPE!(exc_type, 0);
            dump_function_descr(descr, 0);
        }
    } else {
        let data: *mut thread_data_t = msvcrt_get_thread_data();

        exc_type = core::ptr::null_mut();

        if let Some(se_translator) = (*data).se_translator {
            let mut except_ptrs: EXCEPTION_POINTERS = core::mem::zeroed();
            let mut ctx: se_translator_ctx = core::mem::zeroed();

            ctx.frame = frame;
            ctx.descr = descr;
            ctx.nested_frame = nested_frame;
            except_ptrs.ExceptionRecord = rec;
            except_ptrs.ContextRecord = context;
            se_translator((*rec).ExceptionCode, &mut except_ptrs);

            // __EXCEPT_CTX(se_translation_filter, &ctx) -- never entered under DUMMY_PSEH,
            // as in except.rs. Dropping it entirely left `ctx` built and never read.
            if crate::cpp_rtti::_SEH2_Volatile0 != 0 {
                se_translation_filter(&mut except_ptrs, core::ptr::addr_of_mut!(ctx).cast());
            }
        }
    }

    call_catch_block(rec, context, frame, descr, nested_frame, exc_type);
    check_noexcept(rec, descr, if !nested_frame.is_null() { 1u8 } else { 0u8 });
    return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
}

#[no_mangle]
pub unsafe extern "system" fn __CxxLongjmpUnwind(buf: *const _JUMP_BUFFER) {
    let frame = (*buf).Registration as *mut cxx_exception_frame;
    let descr = (*buf).UnwindData[0] as *const cxx_function_descr;

    cxx_local_unwind(frame, descr, (*buf).TryLevel as i32);
}
unsafe extern "C" fn get_scopetable_v4(
    frame: *mut MSVCRT_EXCEPTION_FRAME,
    cookie: ULONG_PTR,
) -> *const SCOPETABLE_V4 {
    (((*frame).scopetable as ULONG_PTR) ^ cookie) as *const SCOPETABLE_V4
}
unsafe extern "C" fn MSVCRT_nested_handler(
    rec: PEXCEPTION_RECORD,
    frame: *mut EXCEPTION_REGISTRATION_RECORD,
    context: PCONTEXT,
    dispatch: *mut *mut EXCEPTION_REGISTRATION_RECORD,
) -> u32 {
    if ((*rec).ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND)) == 0 {
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as u32;
    }
    *dispatch = frame;
    return _EXCEPTION_DISPOSITION_ExceptionCollidedUnwind as u32;
}
unsafe extern "C" fn msvcrt_local_unwind2(frame: *mut MSVCRT_EXCEPTION_FRAME, trylevel: i32, ebp: *mut core::ffi::c_void) {
    let mut reg: EXCEPTION_REGISTRATION_RECORD = core::mem::zeroed();

    reg.Handler = Some(MSVCRT_nested_handler);
    reg.Prev = (*NtCurrentTeb()).Tib.ExceptionList;
    __wine_push_frame(&mut reg);

    while (*frame).trylevel != TRYLEVEL_END && (*frame).trylevel != trylevel {
        let level = (*frame).trylevel;
        (*frame).trylevel = (*(*frame).scopetable.offset(level as isize)).previousTryLevel;
        if (*(*frame).scopetable.offset(level as isize)).lpfnFilter.is_none() {
            call_handler((*(*frame).scopetable.offset(level as isize)).lpfnHandler, ebp);
        }
    }
    __wine_pop_frame(&mut reg);
}
unsafe extern "C" fn msvcrt_local_unwind4(
    cookie: *mut ULONG,
    frame: *mut MSVCRT_EXCEPTION_FRAME,
    trylevel: i32,
    ebp: *mut core::ffi::c_void,
) {
    let mut reg: EXCEPTION_REGISTRATION_RECORD = core::mem::zeroed();
    let scopetable = get_scopetable_v4(frame, *cookie);

    reg.Handler = Some(MSVCRT_nested_handler);
    reg.Prev = (*NtCurrentTeb()).Tib.ExceptionList;
    __wine_push_frame(&mut reg);

    while (*frame).trylevel != -2 && (*frame).trylevel != trylevel {
        let level = (*frame).trylevel;
        (*frame).trylevel = (*scope_entry(scopetable, level)).previousTryLevel;
        if (*scope_entry(scopetable, level)).lpfnFilter.is_none() {
            call_handler((*scope_entry(scopetable, level)).lpfnHandler, ebp);
        }
    }
    __wine_pop_frame(&mut reg);
}

#[no_mangle]
pub unsafe extern "C" fn _local_unwind4(
    cookie: *mut ULONG,
    frame: *mut MSVCRT_EXCEPTION_FRAME,
    trylevel: i32,
) {
    msvcrt_local_unwind4(cookie, frame, trylevel, core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn _except_handler4_common(
    cookie: *mut ULONG,
    check_cookie: Option<unsafe extern "C" fn()>,
    rec: *mut EXCEPTION_RECORD,
    frame: *mut MSVCRT_EXCEPTION_FRAME,
    context: *mut CONTEXT,
    dispatcher: *mut *mut EXCEPTION_REGISTRATION_RECORD,
) -> i32 {
    let mut retval: i32;
    let mut trylevel: i32;
    let mut exceptPtrs: EXCEPTION_POINTERS = core::mem::zeroed();
    let scope_table: *const SCOPETABLE_V4 = get_scopetable_v4(frame, *cookie);

    if (*rec).ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND) != 0 {
        msvcrt_local_unwind4(cookie, frame, -2, core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void);
        return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as i32;
    } else {
        exceptPtrs.ExceptionRecord = rec;
        exceptPtrs.ContextRecord = context;
        *((frame as *mut DWORD).offset(-(1 as isize))) = core::ptr::addr_of!(exceptPtrs) as DWORD;
        trylevel = (*frame).trylevel;

        while trylevel != -2 {
            if (*scope_entry(scope_table, trylevel)).lpfnFilter.is_some() {
                retval = call_filter(
                    (*scope_entry(scope_table, trylevel)).lpfnFilter,
                    (core::ptr::addr_of_mut!(exceptPtrs)).cast(),
                    core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void,
                );

                if retval == EXCEPTION_CONTINUE_EXECUTION as i32 {
                    return _EXCEPTION_DISPOSITION_ExceptionContinueExecution as i32;
                }

                if retval == EXCEPTION_EXECUTE_HANDLER as i32 {
                    __DestructExceptionObject(rec);

                    _global_unwind2(frame as *mut EXCEPTION_REGISTRATION_RECORD);
                    msvcrt_local_unwind4(cookie, frame, trylevel, core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void);

                    (*frame).trylevel = (*scope_entry(scope_table, trylevel)).previousTryLevel;
                    call_finally_block(
                        core::mem::transmute((*scope_entry(scope_table, trylevel)).lpfnHandler),
                        core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void,
                    );
                }
            }
            trylevel = (*scope_entry(scope_table, trylevel)).previousTryLevel;
        }
    }
    return _EXCEPTION_DISPOSITION_ExceptionContinueSearch as i32;
}

/// The ebp a scope handler of `frame` runs with: the address just past the EH3/EH4
/// registration node, which is what `_except_handler3`/`4` hand it. The C passes the
/// jmp_buf's saved `Ebp` instead. In an MSVC frame the two are the same address, but
/// clang realigns the stack of some SEH functions and then addresses their locals from
/// `node + 0x10`, not from the frame pointer. Handed the frame pointer, such a
/// function's `__finally` ran against the wrong base and stored its locals into the
/// middle of the caller's jmp_buf (ucrtbase_apitest setjmp.c:363).
unsafe fn scope_handler_ebp(frame: *mut MSVCRT_EXCEPTION_FRAME) -> *mut core::ffi::c_void {
    core::ptr::addr_of_mut!((*frame)._ebp) as *mut core::ffi::c_void
}

#[no_mangle]
pub unsafe extern "system" fn _seh_longjmp_unwind(jmp: *mut _JUMP_BUFFER) {
    let frame = (*jmp).Registration as *mut MSVCRT_EXCEPTION_FRAME;
    msvcrt_local_unwind2(frame, (*jmp).TryLevel as i32, scope_handler_ebp(frame));
}

#[no_mangle]
pub unsafe extern "system" fn _seh_longjmp_unwind4(jmp: *mut _JUMP_BUFFER) {
    let frame = (*jmp).Registration as *mut MSVCRT_EXCEPTION_FRAME;
    msvcrt_local_unwind4(
        core::ptr::addr_of_mut!((*jmp).Cookie) as *mut ULONG,
        frame,
        (*jmp).TryLevel as i32,
        scope_handler_ebp(frame),
    );
}
#[no_mangle]
pub unsafe extern "C" fn handle_fpieee_flt(
    exception_code: __msvcrt_ulong,
    ep: *mut EXCEPTION_POINTERS,
    handler: Option<unsafe extern "C" fn(*mut _FPIEEE_RECORD) -> i32>,
) -> i32 {
    let ctx: *mut FLOATING_SAVE_AREA = core::ptr::addr_of_mut!((*(*ep).ContextRecord).FloatSave);
    let mut rec: _FPIEEE_RECORD = core::mem::zeroed();
    let ret: i32;

    memset(
        core::ptr::addr_of_mut!(rec) as *mut core::ffi::c_void,
        0,
        core::mem::size_of_val(&rec),
    );
    // Every field of `_FPIEEE_RECORD` below the `Cause/Enable/Status` triple is a BITFIELD,
    // so bindgen exposes it as `set_X()`/`X()` and not as a field. Read this block against
    // the C line by line: it is a positional transcription of the x87 control and status
    // words into the IEEE record, and every one of the five flags appears in three places
    // (Status from the exception, Enable from the mask, Cause = Enable & Status). A pair
    // swapped between two of those three is invisible to every other check we run.
    rec.set_RoundingMode((*ctx).ControlWord >> 10);
    match ((*ctx).ControlWord >> 8) & 0x3 {
        0 => {
            rec.set_Precision(2);
        }
        1 => {
            rec.set_Precision(3);
        }
        2 => {
            rec.set_Precision(1);
        }
        3 => {
            rec.set_Precision(0);
        }
        _ => {}
    }
    rec.Status.set_InvalidOperation((*ctx).StatusWord & 0x1);
    rec.Status.set_ZeroDivide((((*ctx).StatusWord & 0x4) != 0) as u32);
    rec.Status.set_Overflow((((*ctx).StatusWord & 0x8) != 0) as u32);
    rec.Status.set_Underflow((((*ctx).StatusWord & 0x10) != 0) as u32);
    rec.Status.set_Inexact((((*ctx).StatusWord & 0x20) != 0) as u32);
    rec.Enable.set_InvalidOperation((((*ctx).ControlWord & 0x1) == 0) as u32);
    rec.Enable.set_ZeroDivide((((*ctx).ControlWord & 0x4) == 0) as u32);
    rec.Enable.set_Overflow((((*ctx).ControlWord & 0x8) == 0) as u32);
    rec.Enable.set_Underflow((((*ctx).ControlWord & 0x10) == 0) as u32);
    rec.Enable.set_Inexact((((*ctx).ControlWord & 0x20) == 0) as u32);
    rec.Cause
        .set_InvalidOperation(rec.Enable.InvalidOperation() & rec.Status.InvalidOperation());
    rec.Cause.set_ZeroDivide(rec.Enable.ZeroDivide() & rec.Status.ZeroDivide());
    rec.Cause.set_Overflow(rec.Enable.Overflow() & rec.Status.Overflow());
    rec.Cause.set_Underflow(rec.Enable.Underflow() & rec.Status.Underflow());
    rec.Cause.set_Inexact(rec.Enable.Inexact() & rec.Status.Inexact());

    if *((*ctx).ErrorOffset as *const u16) == 0x35dc {
        if exception_code == STATUS_FLOAT_DIVIDE_BY_ZERO as u32
            || exception_code == STATUS_FLOAT_INVALID_OPERATION as u32
        {
            rec.Operand1.set_OperandValid(1);
            rec.Result.set_OperandValid(0);
        } else {
            rec.Operand1.set_OperandValid(0);
            rec.Result.set_OperandValid(1);
        }
        rec.Operand2.set_OperandValid(1);
        rec.set_Operation(_FP_OPERATION_CODE__FpCodeDivide as u32);
        rec.Operand1.set_Format(_FPIEEE_FORMAT__FpFormatFp80 as u32);
        memcpy(
            core::ptr::addr_of_mut!(rec.Operand1.Value.Fp80Value) as *mut core::ffi::c_void,
            (*ctx).RegisterArea.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<_FP80>(),
        );
        rec.Operand2.set_Format(_FPIEEE_FORMAT__FpFormatFp64 as u32);
        rec.Operand2.Value.Fp64Value = *((*ctx).DataOffset as *const _FP64);
        rec.Result.set_Format(_FPIEEE_FORMAT__FpFormatFp80 as u32);
        memcpy(
            core::ptr::addr_of_mut!(rec.Result.Value.Fp80Value) as *mut core::ffi::c_void,
            (*ctx).RegisterArea.as_ptr() as *const core::ffi::c_void,
            core::mem::size_of::<_FP80>(),
        );

        ret = match handler {
            Some(h) => h(core::ptr::addr_of_mut!(rec)),
            None => return EXCEPTION_CONTINUE_SEARCH as i32,
        };

        if ret == EXCEPTION_CONTINUE_EXECUTION as i32 {
            memcpy(
                (*ctx).RegisterArea.as_mut_ptr() as *mut core::ffi::c_void,
                core::ptr::addr_of!(rec.Result.Value.Fp80Value) as *const core::ffi::c_void,
                core::mem::size_of::<_FP80>(),
            );
        }
        return ret;
    }

    FIXME!("seh:unsupported opcode: %lx\n", *((*ctx).ErrorOffset as *const u32));
    return EXCEPTION_CONTINUE_SEARCH as i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt C++ exception handling
//  *
//  * Copyright 2000 Jon Griffiths
//  * Copyright 2002 Alexandre Julliard
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
//  * NOTES
//  * A good reference is the article "How a C++ compiler implements
//  * exception handling" by Vishal Kochhar, available on
//  * www.thecodeproject.com.
//  */
//
// #ifdef __i386__
//
// #include <stdarg.h>
// #include <fpieee.h>
// #define longjmp ms_longjmp  /* avoid prototype mismatch */
// #include <setjmp.h>
// #undef longjmp
//
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
//
// /* the exception frame used by CxxFrameHandler */
// typedef struct __cxx_exception_frame
// {
//     EXCEPTION_REGISTRATION_RECORD  frame;    /* the standard exception frame */
//     int                            trylevel;
//     DWORD                          ebp;
// } cxx_exception_frame;
//
// /* exception frame for nested exceptions in catch block */
// typedef struct
// {
//     EXCEPTION_REGISTRATION_RECORD frame;     /* standard exception frame */
//     cxx_exception_frame          *cxx_frame; /* frame of parent exception */
//     const cxx_function_descr     *descr;     /* descriptor of parent exception */
//     int                           trylevel;  /* current try level */
//     cxx_frame_info                frame_info;
// } catch_func_nested_frame;
//
// typedef struct
// {
//     cxx_exception_frame *frame;
//     const cxx_function_descr *descr;
//     catch_func_nested_frame *nested_frame;
// } se_translator_ctx;
//
// typedef struct _SCOPETABLE
// {
//   int previousTryLevel;
//   int (*lpfnFilter)(PEXCEPTION_POINTERS);
//   void * (*lpfnHandler)(void);
// } SCOPETABLE, *PSCOPETABLE;
//
// typedef struct MSVCRT_EXCEPTION_FRAME
// {
//   EXCEPTION_REGISTRATION_RECORD *prev;
//   void (*handler)(PEXCEPTION_RECORD, EXCEPTION_REGISTRATION_RECORD*,
//                   PCONTEXT, PEXCEPTION_RECORD);
//   PSCOPETABLE scopetable;
//   int trylevel;
//   int _ebp;
//   PEXCEPTION_POINTERS xpointers;
// } MSVCRT_EXCEPTION_FRAME;
//
// typedef struct
// {
//   int   gs_cookie_offset;
//   ULONG gs_cookie_xor;
//   int   eh_cookie_offset;
//   ULONG eh_cookie_xor;
//   SCOPETABLE entries[1];
// } SCOPETABLE_V4;
//
// #define TRYLEVEL_END (-1) /* End of trylevel list */
//
// typedef DWORD (CDECL *cxx_exc_custom_handler)( PEXCEPTION_RECORD, cxx_exception_frame*,
//                                                PCONTEXT, EXCEPTION_REGISTRATION_RECORD**,
//                                                const cxx_function_descr*, int nested_trylevel,
//                                                EXCEPTION_REGISTRATION_RECORD *nested_frame, DWORD unknown );
//
// DWORD CDECL cxx_frame_handler( PEXCEPTION_RECORD rec, cxx_exception_frame* frame,
//                                PCONTEXT context, EXCEPTION_REGISTRATION_RECORD** dispatch,
//                                const cxx_function_descr *descr,
//                                catch_func_nested_frame* nested_frame );
//
// /* continue execution to the specified address after exception is caught */
// extern void DECLSPEC_NORETURN continue_after_catch( cxx_exception_frame* frame, void *addr );
//
// __ASM_GLOBAL_FUNC( continue_after_catch,
//                    "movl 4(%esp), %edx\n\t"
//                    "movl 8(%esp), %eax\n\t"
//                    "movl -4(%edx), %esp\n\t"
//                    "leal 12(%edx), %ebp\n\t"
//                    "jmp *%eax" );
//
// extern void DECLSPEC_NORETURN call_finally_block( void *code_block, void *base_ptr );
//
// __ASM_GLOBAL_FUNC( call_finally_block,
//                    "movl 8(%esp), %ebp\n\t"
//                    "jmp *4(%esp)" );
//
// extern int call_filter( int (*func)(PEXCEPTION_POINTERS), void *arg, void *ebp );
//
// __ASM_GLOBAL_FUNC( call_filter,
//                    "pushl %ebp\n\t"
//                    "pushl 12(%esp)\n\t"
//                    "movl 20(%esp), %ebp\n\t"
//                    "call *12(%esp)\n\t"
//                    "popl %ebp\n\t"
//                    "popl %ebp\n\t"
//                    "ret" );
//
// extern void *call_handler( void * (*func)(void), void *ebp );
//
// __ASM_GLOBAL_FUNC( call_handler,
//                    "pushl %ebp\n\t"
//                    "pushl %ebx\n\t"
//                    "pushl %esi\n\t"
//                    "pushl %edi\n\t"
//                    "movl 24(%esp), %ebp\n\t"
//                    "call *20(%esp)\n\t"
//                    "popl %edi\n\t"
//                    "popl %esi\n\t"
//                    "popl %ebx\n\t"
//                    "popl %ebp\n\t"
//                    "ret" );
//
// /* unwind the local function up to a given trylevel */
// static void cxx_local_unwind( cxx_exception_frame* frame, const cxx_function_descr *descr, int last_level)
// {
//     void * (*handler)(void);
//     int trylevel = frame->trylevel;
//
//     while (trylevel != last_level)
//     {
//         if (trylevel < 0 || trylevel >= descr->unwind_count)
//         {
//             ERR( "invalid trylevel %d\n", trylevel );
//             terminate();
//         }
//         handler = descr->unwind_table[trylevel].handler;
//         if (handler)
//         {
//             TRACE( "calling unwind handler %p trylevel %d last %d ebp %p\n",
//                    handler, trylevel, last_level, &frame->ebp );
//             call_handler( handler, &frame->ebp );
//         }
//         trylevel = descr->unwind_table[trylevel].prev;
//     }
//     frame->trylevel = last_level;
// }
//
// /* handler for exceptions happening while calling a catch function */
// static DWORD catch_function_nested_handler( EXCEPTION_RECORD *rec, EXCEPTION_REGISTRATION_RECORD *frame,
//                                             CONTEXT *context, EXCEPTION_REGISTRATION_RECORD **dispatcher )
// {
//     catch_func_nested_frame *nested_frame = (catch_func_nested_frame *)frame;
//
//     if (rec->ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND))
//     {
//         __CxxUnregisterExceptionObject(&nested_frame->frame_info, FALSE);
//         return ExceptionContinueSearch;
//     }
//
//     TRACE( "got nested exception in catch function\n" );
//
//     if(rec->ExceptionCode == CXX_EXCEPTION)
//     {
//         PEXCEPTION_RECORD prev_rec = msvcrt_get_thread_data()->exc_record;
//
//         if((rec->ExceptionInformation[1] == 0 && rec->ExceptionInformation[2] == 0) ||
//                 (prev_rec->ExceptionCode == CXX_EXCEPTION &&
//                  rec->ExceptionInformation[1] == prev_rec->ExceptionInformation[1] &&
//                  rec->ExceptionInformation[2] == prev_rec->ExceptionInformation[2]))
//         {
//             /* exception was rethrown */
//             *rec = *prev_rec;
//             rec->ExceptionFlags &= ~EXCEPTION_UNWINDING;
//             if(TRACE_ON(seh)) {
//                 TRACE("detect rethrow: exception code: %lx\n", rec->ExceptionCode);
//                 if(rec->ExceptionCode == CXX_EXCEPTION)
//                     TRACE("re-propagate: obj: %Ix, type: %Ix\n",
//                             rec->ExceptionInformation[1], rec->ExceptionInformation[2]);
//             }
//         }
//         else
//         {
//             TRACE("detect threw new exception in catch block\n");
//         }
//     }
//
//     return cxx_frame_handler( rec, nested_frame->cxx_frame, context,
//                               NULL, nested_frame->descr, nested_frame );
// }
//
// /* find and call the appropriate catch block for an exception */
// /* returns the address to continue execution to after the catch block was called */
// static inline void call_catch_block( PEXCEPTION_RECORD rec, CONTEXT *context,
//                                      cxx_exception_frame *frame,
//                                      const cxx_function_descr *descr,
//                                      catch_func_nested_frame *catch_frame,
//                                      cxx_exception_type *info )
// {
//     UINT i;
//     void *addr, *handler, *object = (void *)rec->ExceptionInformation[1];
//     catch_func_nested_frame nested_frame;
//     int trylevel = frame->trylevel;
//     DWORD save_esp = ((DWORD*)frame)[-1];
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     data->processing_throw++;
//     for (i = 0; i < descr->tryblock_count; i++)
//     {
//         const tryblock_info *tryblock = &descr->tryblock[i];
//
//         /* only handle try blocks inside current catch block */
//         if (catch_frame && catch_frame->trylevel > tryblock->start_level) continue;
//
//         if (trylevel < tryblock->start_level) continue;
//         if (trylevel > tryblock->end_level) continue;
//
//         handler = find_catch_handler( object, (uintptr_t)&frame->ebp, 0, tryblock, info, 0 );
//         if (!handler) continue;
//
//         /* Add frame info here so exception is not freed inside RtlUnwind call */
//         _CreateFrameInfo(&nested_frame.frame_info.frame_info, object);
//
//         /* unwind the stack */
//         RtlUnwind( catch_frame ? &catch_frame->frame : &frame->frame, 0, rec, 0 );
//         cxx_local_unwind( frame, descr, tryblock->start_level );
//         frame->trylevel = tryblock->end_level + 1;
//
//         nested_frame.frame_info.rec = data->exc_record;
//         nested_frame.frame_info.context = data->ctx_record;
//         data->exc_record = rec;
//         data->ctx_record = context;
//         data->processing_throw--;
//
//         /* call the catch block */
//         TRACE( "calling handler %p ebp %p\n", handler, &frame->ebp );
//
//         /* setup an exception block for nested exceptions */
//         nested_frame.frame.Handler = catch_function_nested_handler;
//         nested_frame.cxx_frame = frame;
//         nested_frame.descr     = descr;
//         nested_frame.trylevel  = tryblock->end_level + 1;
//
//         __wine_push_frame( &nested_frame.frame );
//         addr = call_handler( handler, &frame->ebp );
//         __wine_pop_frame( &nested_frame.frame );
//
//         ((DWORD*)frame)[-1] = save_esp;
//         __CxxUnregisterExceptionObject(&nested_frame.frame_info, FALSE);
//         TRACE( "done, continuing at %p\n", addr );
//
//         continue_after_catch( frame, addr );
//     }
//     data->processing_throw--;
// }
//
// static LONG __stdcall se_translation_filter( EXCEPTION_POINTERS *ep, void *c )
// {
//     se_translator_ctx *ctx = (se_translator_ctx *)c;
//     EXCEPTION_RECORD *rec = ep->ExceptionRecord;
//     cxx_exception_type *exc_type;
//
//     if (rec->ExceptionCode != CXX_EXCEPTION)
//     {
//         TRACE( "non-c++ exception thrown in SEH handler: %lx\n", rec->ExceptionCode );
//         terminate();
//     }
//
//     exc_type = (cxx_exception_type *)rec->ExceptionInformation[2];
//     call_catch_block( rec, ep->ContextRecord, ctx->frame, ctx->descr,
//             ctx->nested_frame, exc_type );
//
//     __DestructExceptionObject( rec );
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
// /*********************************************************************
//  *		cxx_frame_handler
//  *
//  * Implementation of __CxxFrameHandler.
//  */
// DWORD CDECL cxx_frame_handler( PEXCEPTION_RECORD rec, cxx_exception_frame* frame,
//                                PCONTEXT context, EXCEPTION_REGISTRATION_RECORD** dispatch,
//                                const cxx_function_descr *descr,
//                                catch_func_nested_frame* nested_frame )
// {
//     cxx_exception_type *exc_type;
//
//     if (descr->magic < CXX_FRAME_MAGIC_VC6 || descr->magic > CXX_FRAME_MAGIC_VC8)
//     {
//         ERR( "invalid frame magic %x\n", descr->magic );
//         return ExceptionContinueSearch;
//     }
//     if (descr->magic >= CXX_FRAME_MAGIC_VC8 &&
//         (descr->flags & FUNC_DESCR_SYNCHRONOUS) &&
//         (rec->ExceptionCode != CXX_EXCEPTION))
//         return ExceptionContinueSearch;  /* handle only c++ exceptions */
//
//     if (rec->ExceptionFlags & (EXCEPTION_UNWINDING|EXCEPTION_EXIT_UNWIND))
//     {
//         if (descr->unwind_count && !nested_frame) cxx_local_unwind( frame, descr, -1 );
//         return ExceptionContinueSearch;
//     }
//     if (!descr->tryblock_count)
//     {
//         check_noexcept(rec, descr, nested_frame != NULL);
//         return ExceptionContinueSearch;
//     }
//
//     if(rec->ExceptionCode == CXX_EXCEPTION &&
//             rec->ExceptionInformation[1] == 0 && rec->ExceptionInformation[2] == 0)
//     {
//         *rec = *msvcrt_get_thread_data()->exc_record;
//         rec->ExceptionFlags &= ~EXCEPTION_UNWINDING;
//         if(TRACE_ON(seh)) {
//             TRACE("detect rethrow: exception code: %lx\n", rec->ExceptionCode);
//             if(rec->ExceptionCode == CXX_EXCEPTION)
//                 TRACE("re-propagate: obj: %Ix, type: %Ix\n",
//                         rec->ExceptionInformation[1], rec->ExceptionInformation[2]);
//         }
//     }
//
//     if(rec->ExceptionCode == CXX_EXCEPTION)
//     {
//         exc_type = (cxx_exception_type *)rec->ExceptionInformation[2];
//
//         if (rec->ExceptionInformation[0] > CXX_FRAME_MAGIC_VC8 &&
//                 exc_type->custom_handler)
//         {
//             cxx_exc_custom_handler handler = exc_type->custom_handler;
//             return handler( rec, frame, context, dispatch, descr,
//                             nested_frame ? nested_frame->trylevel : 0,
//                             nested_frame ? &nested_frame->frame : NULL, 0 );
//         }
//
//         if (TRACE_ON(seh))
//         {
//             TRACE("handling C++ exception rec %p frame %p trylevel %d descr %p nested_frame %p\n",
//                   rec, frame, frame->trylevel, descr, nested_frame );
//             TRACE_EXCEPTION_TYPE( exc_type, 0 );
//             dump_function_descr( descr, 0 );
//         }
//     }
//     else
//     {
//         thread_data_t *data = msvcrt_get_thread_data();
//
//         exc_type = NULL;
//         TRACE("handling C exception code %lx  rec %p frame %p trylevel %d descr %p nested_frame %p\n",
//               rec->ExceptionCode,  rec, frame, frame->trylevel, descr, nested_frame );
//
//         if (data->se_translator) {
//             EXCEPTION_POINTERS except_ptrs;
//             se_translator_ctx ctx;
//
//             ctx.frame = frame;
//             ctx.descr = descr;
//             ctx.nested_frame = nested_frame;
//             __TRY
//             {
//                 except_ptrs.ExceptionRecord = rec;
//                 except_ptrs.ContextRecord = context;
//                 data->se_translator( rec->ExceptionCode, &except_ptrs );
//             }
//             __EXCEPT_CTX(se_translation_filter, &ctx)
//             {
//             }
//             __ENDTRY
//         }
//     }
//
//     call_catch_block( rec, context, frame, descr,
//             nested_frame, exc_type );
//     check_noexcept(rec, descr, nested_frame != NULL);
//     return ExceptionContinueSearch;
// }
//
//
// /*********************************************************************
//  *		__CxxFrameHandler (MSVCRT.@)
//  */
// extern DWORD CDECL __CxxFrameHandler( PEXCEPTION_RECORD rec, EXCEPTION_REGISTRATION_RECORD* frame,
//                                       PCONTEXT context, EXCEPTION_REGISTRATION_RECORD** dispatch );
// __ASM_GLOBAL_FUNC( __CxxFrameHandler,
//                    "pushl $0\n\t"        /* nested_frame */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "pushl %eax\n\t"      /* descr */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "pushl 24(%esp)\n\t"  /* dispatch */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "pushl 24(%esp)\n\t"  /* context */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "pushl 24(%esp)\n\t"  /* frame */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "pushl 24(%esp)\n\t"  /* rec */
//                    __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                    "call " __ASM_NAME("cxx_frame_handler") "\n\t"
//                    "add $24,%esp\n\t"
//                    __ASM_CFI(".cfi_adjust_cfa_offset -24\n\t")
//                    "ret" )
//
//
// /*********************************************************************
//  *		__CxxLongjmpUnwind (MSVCRT.@)
//  *
//  * Callback meant to be used as UnwindFunc for setjmp/longjmp.
//  */
// void __stdcall __CxxLongjmpUnwind( const _JUMP_BUFFER *buf )
// {
//     cxx_exception_frame *frame = (cxx_exception_frame *)buf->Registration;
//     const cxx_function_descr *descr = (const cxx_function_descr *)buf->UnwindData[0];
//
//     TRACE( "unwinding frame %p descr %p trylevel %ld\n", frame, descr, buf->TryLevel );
//     cxx_local_unwind( frame, descr, buf->TryLevel );
// }
//
// /*********************************************************************
//  *		_EH_prolog (MSVCRT.@)
//  */
//
// /* Provided for VC++ binary compatibility only */
// __ASM_GLOBAL_FUNC(_EH_prolog,
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")  /* skip ret addr */
//                   "pushl $-1\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   "pushl %eax\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   "pushl %fs:0\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   "movl  %esp, %fs:0\n\t"
//                   "movl  12(%esp), %eax\n\t"
//                   "movl  %ebp, 12(%esp)\n\t"
//                   "leal  12(%esp), %ebp\n\t"
//                   "pushl %eax\n\t"
//                   __ASM_CFI(".cfi_adjust_cfa_offset 4\n\t")
//                   "ret")
//
// static const SCOPETABLE_V4 *get_scopetable_v4( MSVCRT_EXCEPTION_FRAME *frame, ULONG_PTR cookie )
// {
//     return (const SCOPETABLE_V4 *)((ULONG_PTR)frame->scopetable ^ cookie);
// }
//
// static DWORD MSVCRT_nested_handler(PEXCEPTION_RECORD rec,
//                                    EXCEPTION_REGISTRATION_RECORD* frame,
//                                    PCONTEXT context,
//                                    EXCEPTION_REGISTRATION_RECORD** dispatch)
// {
//   if (!(rec->ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND)))
//     return ExceptionContinueSearch;
//   *dispatch = frame;
//   return ExceptionCollidedUnwind;
// }
//
// static void msvcrt_local_unwind2(MSVCRT_EXCEPTION_FRAME* frame, int trylevel, void *ebp)
// {
//   EXCEPTION_REGISTRATION_RECORD reg;
//
//   TRACE("(%p,%d,%d)\n",frame, frame->trylevel, trylevel);
//
//   /* Register a handler in case of a nested exception */
//   reg.Handler = MSVCRT_nested_handler;
//   reg.Prev = NtCurrentTeb()->Tib.ExceptionList;
//   __wine_push_frame(&reg);
//
//   while (frame->trylevel != TRYLEVEL_END && frame->trylevel != trylevel)
//   {
//       int level = frame->trylevel;
//       frame->trylevel = frame->scopetable[level].previousTryLevel;
//       if (!frame->scopetable[level].lpfnFilter)
//       {
//           TRACE( "__try block cleanup level %d handler %p ebp %p\n",
//                  level, frame->scopetable[level].lpfnHandler, ebp );
//           call_handler( frame->scopetable[level].lpfnHandler, ebp );
//       }
//   }
//   __wine_pop_frame(&reg);
//   TRACE("unwound OK\n");
// }
//
// static void msvcrt_local_unwind4( ULONG *cookie, MSVCRT_EXCEPTION_FRAME* frame, int trylevel, void *ebp )
// {
//     EXCEPTION_REGISTRATION_RECORD reg;
//     const SCOPETABLE_V4 *scopetable = get_scopetable_v4( frame, *cookie );
//
//     TRACE("(%p,%d,%d)\n",frame, frame->trylevel, trylevel);
//
//     /* Register a handler in case of a nested exception */
//     reg.Handler = MSVCRT_nested_handler;
//     reg.Prev = NtCurrentTeb()->Tib.ExceptionList;
//     __wine_push_frame(&reg);
//
//     while (frame->trylevel != -2 && frame->trylevel != trylevel)
//     {
//         int level = frame->trylevel;
//         frame->trylevel = scopetable->entries[level].previousTryLevel;
//         if (!scopetable->entries[level].lpfnFilter)
//         {
//             TRACE( "__try block cleanup level %d handler %p ebp %p\n",
//                    level, scopetable->entries[level].lpfnHandler, ebp );
//             call_handler( scopetable->entries[level].lpfnHandler, ebp );
//         }
//     }
//     __wine_pop_frame(&reg);
//     TRACE("unwound OK\n");
// }
//
// #ifndef __REACTOS__
// /*******************************************************************
//  *		_local_unwind2 (MSVCRT.@)
//  */
// void CDECL _local_unwind2(MSVCRT_EXCEPTION_FRAME* frame, int trylevel)
// {
//     msvcrt_local_unwind2( frame, trylevel, &frame->_ebp );
// }
// #endif
//
// /*******************************************************************
//  *		_local_unwind4 (MSVCRT.@)
//  */
// void CDECL _local_unwind4( ULONG *cookie, MSVCRT_EXCEPTION_FRAME* frame, int trylevel )
// {
//     msvcrt_local_unwind4( cookie, frame, trylevel, &frame->_ebp );
// }
//
// #ifdef __REACTOS__
// void CDECL _global_unwind2(EXCEPTION_REGISTRATION_RECORD* frame);
// #else
// /*******************************************************************
//  *		_global_unwind2 (MSVCRT.@)
//  */
// void CDECL _global_unwind2(EXCEPTION_REGISTRATION_RECORD* frame)
// {
//     TRACE("(%p)\n",frame);
//     RtlUnwind( frame, 0, 0, 0 );
// }
//
// /*********************************************************************
//  *		_except_handler2 (MSVCRT.@)
//  */
// int CDECL _except_handler2(PEXCEPTION_RECORD rec,
//                            EXCEPTION_REGISTRATION_RECORD* frame,
//                            PCONTEXT context,
//                            EXCEPTION_REGISTRATION_RECORD** dispatcher)
// {
//   FIXME("exception %lx flags=%lx at %p handler=%p %p %p stub\n",
//         rec->ExceptionCode, rec->ExceptionFlags, rec->ExceptionAddress,
//         frame->Handler, context, dispatcher);
//   return ExceptionContinueSearch;
// }
//
// /*********************************************************************
//  *		_except_handler3 (MSVCRT.@)
//  */
// int CDECL _except_handler3(PEXCEPTION_RECORD rec,
//                            MSVCRT_EXCEPTION_FRAME* frame,
//                            PCONTEXT context, void* dispatcher)
// {
//   int retval, trylevel;
//   EXCEPTION_POINTERS exceptPtrs;
//   PSCOPETABLE pScopeTable;
//
//   TRACE("exception %lx flags=%lx at %p handler=%p %p %p semi-stub\n",
//         rec->ExceptionCode, rec->ExceptionFlags, rec->ExceptionAddress,
//         frame->handler, context, dispatcher);
//
// #ifdef _MSC_VER
//   __asm{ cld }
// #else
//   __asm__ __volatile__ ("cld");
// #endif
//   if (rec->ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND))
//   {
//     /* Unwinding the current frame */
//     msvcrt_local_unwind2(frame, TRYLEVEL_END, &frame->_ebp);
//     TRACE("unwound current frame, returning ExceptionContinueSearch\n");
//     return ExceptionContinueSearch;
//   }
//   else
//   {
//     /* Hunting for handler */
//     exceptPtrs.ExceptionRecord = rec;
//     exceptPtrs.ContextRecord = context;
//     *((DWORD *)frame-1) = (DWORD)&exceptPtrs;
//     trylevel = frame->trylevel;
//     pScopeTable = frame->scopetable;
//
//     while (trylevel != TRYLEVEL_END)
//     {
//       TRACE( "level %d prev %d filter %p\n", trylevel, pScopeTable[trylevel].previousTryLevel,
//              pScopeTable[trylevel].lpfnFilter );
//       if (pScopeTable[trylevel].lpfnFilter)
//       {
//         retval = call_filter( pScopeTable[trylevel].lpfnFilter, &exceptPtrs, &frame->_ebp );
//
//         TRACE("filter returned %s\n", retval == EXCEPTION_CONTINUE_EXECUTION ?
//               "CONTINUE_EXECUTION" : retval == EXCEPTION_EXECUTE_HANDLER ?
//               "EXECUTE_HANDLER" : "CONTINUE_SEARCH");
//
//         if (retval == EXCEPTION_CONTINUE_EXECUTION)
//           return ExceptionContinueExecution;
//
//         if (retval == EXCEPTION_EXECUTE_HANDLER)
//         {
//           /* Unwind all higher frames, this one will handle the exception */
//           _global_unwind2((EXCEPTION_REGISTRATION_RECORD*)frame);
//           msvcrt_local_unwind2(frame, trylevel, &frame->_ebp);
//
//           /* Set our trylevel to the enclosing block, and call the __finally
//            * code, which won't return
//            */
//           frame->trylevel = pScopeTable[trylevel].previousTryLevel;
//           TRACE("__finally block %p\n",pScopeTable[trylevel].lpfnHandler);
//           call_finally_block(pScopeTable[trylevel].lpfnHandler, &frame->_ebp);
//        }
//       }
//       trylevel = pScopeTable[trylevel].previousTryLevel;
//     }
//   }
//   TRACE("reached TRYLEVEL_END, returning ExceptionContinueSearch\n");
//   return ExceptionContinueSearch;
// }
// #endif
//
// /*********************************************************************
//  *		_except_handler4_common (MSVCRT.@)
//  */
// int CDECL _except_handler4_common( ULONG *cookie, void (*check_cookie)(void),
//                                    EXCEPTION_RECORD *rec, MSVCRT_EXCEPTION_FRAME *frame,
//                                    CONTEXT *context, EXCEPTION_REGISTRATION_RECORD **dispatcher )
// {
//     int retval, trylevel;
//     EXCEPTION_POINTERS exceptPtrs;
//     const SCOPETABLE_V4 *scope_table = get_scopetable_v4( frame, *cookie );
//
//     TRACE( "exception %lx flags=%lx at %p handler=%p %p %p cookie=%lx scope table=%p cookies=%d/%lx,%d/%lx\n",
//            rec->ExceptionCode, rec->ExceptionFlags, rec->ExceptionAddress,
//            frame->handler, context, dispatcher, *cookie, scope_table,
//            scope_table->gs_cookie_offset, scope_table->gs_cookie_xor,
//            scope_table->eh_cookie_offset, scope_table->eh_cookie_xor );
//
//     /* FIXME: no cookie validation yet */
//
//     if (rec->ExceptionFlags & (EXCEPTION_UNWINDING | EXCEPTION_EXIT_UNWIND))
//     {
//         /* Unwinding the current frame */
//         msvcrt_local_unwind4( cookie, frame, -2, &frame->_ebp );
//         TRACE("unwound current frame, returning ExceptionContinueSearch\n");
//         return ExceptionContinueSearch;
//     }
//     else
//     {
//         /* Hunting for handler */
//         exceptPtrs.ExceptionRecord = rec;
//         exceptPtrs.ContextRecord = context;
//         *((DWORD *)frame-1) = (DWORD)&exceptPtrs;
//         trylevel = frame->trylevel;
//
//         while (trylevel != -2)
//         {
//             TRACE( "level %d prev %d filter %p\n", trylevel,
//                    scope_table->entries[trylevel].previousTryLevel,
//                    scope_table->entries[trylevel].lpfnFilter );
//             if (scope_table->entries[trylevel].lpfnFilter)
//             {
//                 retval = call_filter( scope_table->entries[trylevel].lpfnFilter, &exceptPtrs, &frame->_ebp );
//
//                 TRACE("filter returned %s\n", retval == EXCEPTION_CONTINUE_EXECUTION ?
//                       "CONTINUE_EXECUTION" : retval == EXCEPTION_EXECUTE_HANDLER ?
//                       "EXECUTE_HANDLER" : "CONTINUE_SEARCH");
//
//                 if (retval == EXCEPTION_CONTINUE_EXECUTION)
//                     return ExceptionContinueExecution;
//
//                 if (retval == EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     __DestructExceptionObject(rec);
//
//                     /* Unwind all higher frames, this one will handle the exception */
//                     _global_unwind2((EXCEPTION_REGISTRATION_RECORD*)frame);
//                     msvcrt_local_unwind4( cookie, frame, trylevel, &frame->_ebp );
//
//                     /* Set our trylevel to the enclosing block, and call the __finally
//                      * code, which won't return
//                      */
//                     frame->trylevel = scope_table->entries[trylevel].previousTryLevel;
//                     TRACE("__finally block %p\n",scope_table->entries[trylevel].lpfnHandler);
//                     call_finally_block(scope_table->entries[trylevel].lpfnHandler, &frame->_ebp);
//                 }
//             }
//             trylevel = scope_table->entries[trylevel].previousTryLevel;
//         }
//     }
//     TRACE("reached -2, returning ExceptionContinueSearch\n");
//     return ExceptionContinueSearch;
// }
//
//
// #ifndef __REACTOS__
// /*
//  * setjmp/longjmp implementation
//  */
//
// #define MSVCRT_JMP_MAGIC 0x56433230 /* ID value for new jump structure */
// typedef void (__stdcall *MSVCRT_unwind_function)(const _JUMP_BUFFER *);
//
// /* define an entrypoint for setjmp/setjmp3 that stores the registers in the jmp buf */
// /* and then jumps to the C backend function */
// #define DEFINE_SETJMP_ENTRYPOINT(name) \
//     __ASM_GLOBAL_FUNC( name, \
//                        "movl 4(%esp),%ecx\n\t"   /* jmp_buf */      \
//                        "movl %ebp,0(%ecx)\n\t"   /* jmp_buf.Ebp */  \
//                        "movl %ebx,4(%ecx)\n\t"   /* jmp_buf.Ebx */  \
//                        "movl %edi,8(%ecx)\n\t"   /* jmp_buf.Edi */  \
//                        "movl %esi,12(%ecx)\n\t"  /* jmp_buf.Esi */  \
//                        "movl %esp,16(%ecx)\n\t"  /* jmp_buf.Esp */  \
//                        "movl 0(%esp),%eax\n\t"                      \
//                        "movl %eax,20(%ecx)\n\t"  /* jmp_buf.Eip */  \
//                        "jmp " __ASM_NAME("__regs_") # name )
//
// /*******************************************************************
//  *		_setjmp (MSVCRT.@)
//  */
// #undef _setjmp
// DEFINE_SETJMP_ENTRYPOINT( _setjmp )
// int CDECL __regs__setjmp(_JUMP_BUFFER *jmp)
// {
//     jmp->Registration = (unsigned long)NtCurrentTeb()->Tib.ExceptionList;
//     if (jmp->Registration == ~0UL)
//         jmp->TryLevel = TRYLEVEL_END;
//     else
//         jmp->TryLevel = ((MSVCRT_EXCEPTION_FRAME*)jmp->Registration)->trylevel;
//
//     TRACE("buf=%p ebx=%08lx esi=%08lx edi=%08lx ebp=%08lx esp=%08lx eip=%08lx frame=%08lx\n",
//           jmp, jmp->Ebx, jmp->Esi, jmp->Edi, jmp->Ebp, jmp->Esp, jmp->Eip, jmp->Registration );
//     return 0;
// }
//
// /*******************************************************************
//  *		_setjmp3 (MSVCRT.@)
//  */
// DEFINE_SETJMP_ENTRYPOINT( _setjmp3 )
// int __cdecl __regs__setjmp3(_JUMP_BUFFER *jmp, int nb_args, ...)
// {
//     jmp->Cookie = MSVCRT_JMP_MAGIC;
//     jmp->UnwindFunc = 0;
//     jmp->Registration = (unsigned long)NtCurrentTeb()->Tib.ExceptionList;
//     if (jmp->Registration == ~0UL)
//     {
//         jmp->TryLevel = TRYLEVEL_END;
//     }
//     else
//     {
//         int i;
//         va_list args;
//
//         va_start( args, nb_args );
//         if (nb_args > 0) jmp->UnwindFunc = va_arg( args, unsigned long );
//         if (nb_args > 1) jmp->TryLevel = va_arg( args, unsigned long );
//         else jmp->TryLevel = ((MSVCRT_EXCEPTION_FRAME*)jmp->Registration)->trylevel;
//         for (i = 0; i < 6 && i < nb_args - 2; i++)
//             jmp->UnwindData[i] = va_arg( args, unsigned long );
//         va_end( args );
//     }
//
//     TRACE("buf=%p ebx=%08lx esi=%08lx edi=%08lx ebp=%08lx esp=%08lx eip=%08lx frame=%08lx\n",
//           jmp, jmp->Ebx, jmp->Esi, jmp->Edi, jmp->Ebp, jmp->Esp, jmp->Eip, jmp->Registration );
//     return 0;
// }
//
// /*********************************************************************
//  *		longjmp (MSVCRT.@)
//  */
// void __cdecl longjmp(_JUMP_BUFFER *jmp, int retval)
// {
//     unsigned long cur_frame = 0;
//
//     TRACE("buf=%p ebx=%08lx esi=%08lx edi=%08lx ebp=%08lx esp=%08lx eip=%08lx frame=%08lx retval=%08x\n",
//           jmp, jmp->Ebx, jmp->Esi, jmp->Edi, jmp->Ebp, jmp->Esp, jmp->Eip, jmp->Registration, retval );
//
//     cur_frame=(unsigned long)NtCurrentTeb()->Tib.ExceptionList;
//     TRACE("cur_frame=%lx\n",cur_frame);
//
//     if (cur_frame != jmp->Registration)
//         _global_unwind2((EXCEPTION_REGISTRATION_RECORD*)jmp->Registration);
//
//     if (jmp->Registration)
//     {
//         if (IsBadReadPtr(&jmp->Cookie, sizeof(long)) || jmp->Cookie != MSVCRT_JMP_MAGIC)
//         {
//             msvcrt_local_unwind2((MSVCRT_EXCEPTION_FRAME*)jmp->Registration,
//                                  jmp->TryLevel, (void *)jmp->Ebp);
//         }
//         else if(jmp->UnwindFunc)
//         {
//             MSVCRT_unwind_function unwind_func;
//
//             unwind_func=(MSVCRT_unwind_function)jmp->UnwindFunc;
//             unwind_func(jmp);
//         }
//     }
//
//     if (!retval)
//         retval = 1;
//
//     __wine_longjmp( (__wine_jmp_buf *)jmp, retval );
// }
// #endif // __REACTOS__
//
// /*********************************************************************
//  *		_seh_longjmp_unwind (MSVCRT.@)
//  */
// void __stdcall _seh_longjmp_unwind(_JUMP_BUFFER *jmp)
// {
//     msvcrt_local_unwind2( (MSVCRT_EXCEPTION_FRAME *)jmp->Registration, jmp->TryLevel, (void *)jmp->Ebp );
// }
//
// /*********************************************************************
//  *		_seh_longjmp_unwind4 (MSVCRT.@)
//  */
// void __stdcall _seh_longjmp_unwind4(_JUMP_BUFFER *jmp)
// {
//     msvcrt_local_unwind4( (ULONG *)&jmp->Cookie, (MSVCRT_EXCEPTION_FRAME *)jmp->Registration,
//                           jmp->TryLevel, (void *)jmp->Ebp );
// }
//
// /*********************************************************************
//  *              handle_fpieee_flt
//  */
// int handle_fpieee_flt( __msvcrt_ulong exception_code, EXCEPTION_POINTERS *ep,
//                        int (__cdecl *handler)(_FPIEEE_RECORD*) )
// {
//     FLOATING_SAVE_AREA *ctx = &ep->ContextRecord->FloatSave;
//     _FPIEEE_RECORD rec;
//     int ret;
//
//     memset(&rec, 0, sizeof(rec));
//     rec.RoundingMode = ctx->ControlWord >> 10;
//     switch((ctx->ControlWord >> 8) & 0x3) {
//     case 0: rec.Precision = 2; break;
//     case 1: rec.Precision = 3; break;
//     case 2: rec.Precision = 1; break;
//     case 3: rec.Precision = 0; break;
//     }
//     rec.Status.InvalidOperation = ctx->StatusWord & 0x1;
//     rec.Status.ZeroDivide = ((ctx->StatusWord & 0x4) != 0);
//     rec.Status.Overflow = ((ctx->StatusWord & 0x8) != 0);
//     rec.Status.Underflow = ((ctx->StatusWord & 0x10) != 0);
//     rec.Status.Inexact = ((ctx->StatusWord & 0x20) != 0);
//     rec.Enable.InvalidOperation = ((ctx->ControlWord & 0x1) == 0);
//     rec.Enable.ZeroDivide = ((ctx->ControlWord & 0x4) == 0);
//     rec.Enable.Overflow = ((ctx->ControlWord & 0x8) == 0);
//     rec.Enable.Underflow = ((ctx->ControlWord & 0x10) == 0);
//     rec.Enable.Inexact = ((ctx->ControlWord & 0x20) == 0);
//     rec.Cause.InvalidOperation = rec.Enable.InvalidOperation & rec.Status.InvalidOperation;
//     rec.Cause.ZeroDivide = rec.Enable.ZeroDivide & rec.Status.ZeroDivide;
//     rec.Cause.Overflow = rec.Enable.Overflow & rec.Status.Overflow;
//     rec.Cause.Underflow = rec.Enable.Underflow & rec.Status.Underflow;
//     rec.Cause.Inexact = rec.Enable.Inexact & rec.Status.Inexact;
//
//     TRACE("code %lx handler %p opcode %lx\n", exception_code, handler,
//           *(ULONG*)ep->ContextRecord->FloatSave.ErrorOffset);
//
//     if(*(WORD*)ctx->ErrorOffset == 0x35dc) { /* fdiv m64fp */
//         if(exception_code==STATUS_FLOAT_DIVIDE_BY_ZERO || exception_code==STATUS_FLOAT_INVALID_OPERATION) {
//             rec.Operand1.OperandValid = 1;
//             rec.Result.OperandValid = 0;
//         } else {
//             rec.Operand1.OperandValid = 0;
//             rec.Result.OperandValid = 1;
//         }
//         rec.Operand2.OperandValid = 1;
//         rec.Operation = _FpCodeDivide;
//         rec.Operand1.Format = _FpFormatFp80;
//         memcpy(&rec.Operand1.Value.Fp80Value, ctx->RegisterArea, sizeof(rec.Operand1.Value.Fp80Value));
//         rec.Operand2.Format = _FpFormatFp64;
//         rec.Operand2.Value.Fp64Value = *(double*)ctx->DataOffset;
//         rec.Result.Format = _FpFormatFp80;
//         memcpy(&rec.Result.Value.Fp80Value, ctx->RegisterArea, sizeof(rec.Operand1.Value.Fp80Value));
//
//         ret = handler(&rec);
//
//         if(ret == EXCEPTION_CONTINUE_EXECUTION)
//             memcpy(ctx->RegisterArea, &rec.Result.Value.Fp80Value, sizeof(rec.Operand1.Value.Fp80Value));
//         return ret;
//     }
//
//     FIXME("unsupported opcode: %lx\n", *(ULONG*)ep->ContextRecord->FloatSave.ErrorOffset);
//     return EXCEPTION_CONTINUE_SEARCH;
// }
//
// #endif  /* __i386__ */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
