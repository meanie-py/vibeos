//! Grades cpp.c (reference TU #14) and except.c (#15) — the C++ EH subsystem.
//!
//! WHAT MAKES THIS FILE DIFFERENT FROM THE OTHERS: most of these exports do not RETURN their
//! behaviour. They mutate per-thread state, register into a linked list, install a handler,
//! terminate the process, or raise an exception that never comes back. So the rows here are
//! built on three instruments the rest of the harness does not use:
//!
//! * **cppimpl.c's `RaiseException` sink.** `_CxxThrowException` marshals an exception object
//!   and its descriptor into an EXCEPTION_RECORD and never returns. The sink RECORDS the code,
//!   flags, parameter count and every ExceptionInformation slot, then longjmps back to the
//!   row. That marshalling is where a translation bug in a throw would live, and no other
//!   instrument here can see it. Faking an unwind instead would let a row "pass" having graded
//!   a control flow that does not exist.
//!
//! * **`fork()` for the arms that terminate.** `raise` with SIG_DFL is `_exit(3)`; `terminate`
//!   with no handler is `abort()`. Both sides really do it, in a child, and the row compares
//!   the child's raw wait status — so SIGABRT-vs-exit(3) is itself part of the verdict. The
//!   alternative, an overlay that records and returns, is unsound here and was tried first:
//!   `_exit` is DECLSPEC_NORETURN, the compiler emits nothing after the call, and an overlay
//!   that returned would drop the caller into whatever instruction was laid out next.
//!
//! * **STRUCTURAL comparison of handler pointers.** `set_terminate` and `signal` return the
//!   PREVIOUS handler, and the two sides install different functions, so the raw pointers can
//!   never be equal. Each is reduced to a small tag — SIG_DFL / SIG_IGN / SIG_ERR / "the one I
//!   installed" / "some other" — which is the part that means the same thing on both sides.
//!
//! WHAT IS OUT OF REACH, and recorded as such rather than faked: everything under
//! `__CxxFrameHandler`. Table-based dispatch needs a real PE unwind table and a real stack,
//! and cppimpl.c's `RtlUnwindEx` / `call_*_handler` / `handle_fpieee_flt` are loud sinks that
//! `_Exit(94)` rather than return a plausible value. So `__CxxFrameHandler`, `_fpieee_flt`,
//! `find_catch_handler` and `dump_function_descr` carry no rows; the tail of this file counts
//! them so the coverage claim stays honest.
//!
//! The i386 model is a different struct set entirely (no RVA) and is graded by the sibling
//! msvcrt-seh-difftest.

#![allow(non_snake_case)]

extern crate msvcrt_crt_rust;

use core::ffi::{c_char, c_int, c_uint, c_void};
use msvcrt_sys as sys;

// ── the two cxx.h objects, mirrored ──────────────────────────────────────────────────────
//
// A transcription, and asserted against DERIVED numbers before any row runs — see
// `check_layout`. cxx.h is included only by the reference TUs, so the layout guardian cannot
// reach these two the way it reaches EXCEPTION_RECORD; the generator measures them instead
// and cppimpl.c hands the numbers over.

#[repr(C)]
struct Exception {
    vtable: *const c_void,
    name: *mut c_char,
    do_free: c_int,
}

#[repr(C)]
struct TypeInfo {
    vtable: *const c_void,
    name: *mut c_char,
    mangled: [u8; 128],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FrameInfo {
    object: *mut c_void,
    next: *mut FrameInfo,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct CxxFrameInfo {
    frame_info: FrameInfo,
    rec: *mut c_void,
    context: *mut c_void,
}

#[repr(C)]
struct ExceptionRecord {
    ExceptionCode: u32,
    ExceptionFlags: u32,
    ExceptionRecord: *mut ExceptionRecord,
    ExceptionAddress: *mut c_void,
    NumberParameters: u32,
    ExceptionInformation: [usize; 15],
}

#[repr(C)]
struct ExceptionPointers {
    ExceptionRecord: *mut ExceptionRecord,
    ContextRecord: *mut c_void,
}

type SigHandler = Option<unsafe extern "C" fn(c_int)>;
type VoidFn = Option<unsafe extern "C" fn()>;
type SeTranslator = Option<unsafe extern "C" fn(c_uint, *mut ExceptionPointers)>;

extern "C" {
    // ── cpp.c: type_info ────────────────────────────────────────────────────────────────
    fn type_info_name(this: *mut TypeInfo) -> *const c_char;
    fn ref_type_info_name(this: *mut TypeInfo) -> *const c_char;
    fn type_info_raw_name(this: *mut TypeInfo) -> *const c_char;
    fn ref_type_info_raw_name(this: *mut TypeInfo) -> *const c_char;
    fn type_info_dtor(this: *mut TypeInfo);
    fn ref_type_info_dtor(this: *mut TypeInfo);
    fn type_info_opequals_equals(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;
    fn ref_type_info_opequals_equals(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;
    fn type_info_opnot_equals(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;
    fn ref_type_info_opnot_equals(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;
    fn type_info_before(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;
    fn ref_type_info_before(this: *mut TypeInfo, rhs: *const TypeInfo) -> c_int;

    // ── cpp.c: the exception object family ──────────────────────────────────────────────
    fn exception_default_ctor(this: *mut Exception) -> *mut Exception;
    fn ref_exception_default_ctor(this: *mut Exception) -> *mut Exception;
    fn exception_ctor(this: *mut Exception, name: *const *const c_char) -> *mut Exception;
    fn ref_exception_ctor(this: *mut Exception, name: *const *const c_char) -> *mut Exception;
    fn exception_ctor_noalloc(
        this: *mut Exception, name: *mut *mut c_char, noalloc: c_int,
    ) -> *mut Exception;
    fn ref_exception_ctor_noalloc(
        this: *mut Exception, name: *mut *mut c_char, noalloc: c_int,
    ) -> *mut Exception;
    fn exception_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_exception_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn exception_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_exception_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn exception_what(this: *mut Exception) -> *const c_char;
    fn ref_exception_what(this: *mut Exception) -> *const c_char;
    fn exception_dtor(this: *mut Exception);
    fn ref_exception_dtor(this: *mut Exception);
    fn exception_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_exception_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn exception_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_exception_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;

    fn bad_typeid_ctor(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn ref_bad_typeid_ctor(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn bad_typeid_default_ctor(this: *mut Exception) -> *mut Exception;
    fn ref_bad_typeid_default_ctor(this: *mut Exception) -> *mut Exception;
    fn bad_typeid_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_bad_typeid_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn bad_typeid_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_bad_typeid_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn bad_typeid_dtor(this: *mut Exception);
    fn ref_bad_typeid_dtor(this: *mut Exception);
    fn bad_typeid_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_bad_typeid_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn bad_typeid_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_bad_typeid_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;

    fn bad_cast_ctor(this: *mut Exception, name: *const *const c_char) -> *mut Exception;
    fn ref_bad_cast_ctor(this: *mut Exception, name: *const *const c_char) -> *mut Exception;
    fn bad_cast_ctor_charptr(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn ref_bad_cast_ctor_charptr(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn bad_cast_default_ctor(this: *mut Exception) -> *mut Exception;
    fn ref_bad_cast_default_ctor(this: *mut Exception) -> *mut Exception;
    fn bad_cast_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_bad_cast_copy_ctor(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn bad_cast_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn ref_bad_cast_opequals(this: *mut Exception, rhs: *const Exception) -> *mut Exception;
    fn bad_cast_dtor(this: *mut Exception);
    fn ref_bad_cast_dtor(this: *mut Exception);
    fn bad_cast_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_bad_cast_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn bad_cast_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref_bad_cast_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;

    fn __non_rtti_object_ctor(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn ref___non_rtti_object_ctor(this: *mut Exception, name: *const c_char) -> *mut Exception;
    fn __non_rtti_object_copy_ctor(this: *mut Exception, rhs: *const Exception)
        -> *mut Exception;
    fn ref___non_rtti_object_copy_ctor(
        this: *mut Exception, rhs: *const Exception,
    ) -> *mut Exception;
    fn __non_rtti_object_opequals(this: *mut Exception, rhs: *const Exception)
        -> *mut Exception;
    fn ref___non_rtti_object_opequals(
        this: *mut Exception, rhs: *const Exception,
    ) -> *mut Exception;
    fn __non_rtti_object_dtor(this: *mut Exception);
    fn ref___non_rtti_object_dtor(this: *mut Exception);
    fn __non_rtti_object_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref___non_rtti_object_scalar_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn __non_rtti_object_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;
    fn ref___non_rtti_object_vector_dtor(this: *mut Exception, flags: c_uint) -> *mut c_void;

    // ── cpp.c: the per-thread handlers ──────────────────────────────────────────────────
    fn set_terminate(f: VoidFn) -> VoidFn;
    fn ref_set_terminate(f: VoidFn) -> VoidFn;
    fn _get_terminate() -> VoidFn;
    fn ref__get_terminate() -> VoidFn;
    fn set_unexpected(f: VoidFn) -> VoidFn;
    fn ref_set_unexpected(f: VoidFn) -> VoidFn;
    fn _get_unexpected() -> VoidFn;
    fn ref__get_unexpected() -> VoidFn;
    fn _set_se_translator(f: SeTranslator) -> SeTranslator;
    fn ref__set_se_translator(f: SeTranslator) -> SeTranslator;
    fn terminate();
    fn ref_terminate();
    fn unexpected();
    fn ref_unexpected();

    // ── cpp.c: the throw path and RTTI init ─────────────────────────────────────────────
    fn msvcrt_init_exception(base: *mut c_void);
    fn ref_msvcrt_init_exception(base: *mut c_void);
    fn throw_exception(msg: *const c_char);
    fn ref_throw_exception(msg: *const c_char);

    // ── except.c: the signal layer ──────────────────────────────────────────────────────
    //
    // `__real_signal`, NOT `signal`, and this is the whole reason the first run of this suite
    // reported a divergence on `signal(1, SIG_DFL)` that was the harness's. build.rs passes
    // `-Wl,--wrap=signal` so Rust's startup call does not land in msvcrt's `signal` (which has
    // no SIGPIPE and answers SIG_ERR, which std treats as fatal). `--wrap` redirects EVERY
    // reference in the link, including this file's — so a row calling `signal` was grading
    // cppimpl.c's `sigaction` shim against the genuine C and reporting the difference as a bug
    // in the subject. `__real_signal` is the linker's name for the definition itself, which is
    // exactly the subject's `#[no_mangle] signal`.
    #[link_name = "__real_signal"]
    fn signal(sig: c_int, func: SigHandler) -> SigHandler;
    fn ref_signal(sig: c_int, func: SigHandler) -> SigHandler;
    fn raise(sig: c_int) -> c_int;
    fn ref_raise(sig: c_int) -> c_int;
    fn msvcrt_init_signals();
    fn ref_msvcrt_init_signals();
    fn msvcrt_free_signals();
    fn ref_msvcrt_free_signals();
    fn __pxcptinfoptrs() -> *mut *mut c_void;
    fn ref___pxcptinfoptrs() -> *mut *mut c_void;

    // ── except.c: the frame-info list and exception-object registration ─────────────────
    fn _CreateFrameInfo(fi: *mut FrameInfo, obj: *mut c_void) -> *mut FrameInfo;
    fn ref__CreateFrameInfo(fi: *mut FrameInfo, obj: *mut c_void) -> *mut FrameInfo;
    fn _FindAndUnlinkFrame(fi: *mut FrameInfo);
    fn ref__FindAndUnlinkFrame(fi: *mut FrameInfo);
    fn _IsExceptionObjectToBeDestroyed(obj: *const c_void) -> c_int;
    fn ref__IsExceptionObjectToBeDestroyed(obj: *const c_void) -> c_int;
    fn __CxxRegisterExceptionObject(
        ep: *mut ExceptionPointers, fi: *mut CxxFrameInfo,
    ) -> c_int;
    fn ref___CxxRegisterExceptionObject(
        ep: *mut ExceptionPointers, fi: *mut CxxFrameInfo,
    ) -> c_int;
    fn __CxxUnregisterExceptionObject(fi: *mut CxxFrameInfo, in_use: c_int);
    fn ref___CxxUnregisterExceptionObject(fi: *mut CxxFrameInfo, in_use: c_int);
    fn __DestructExceptionObject(rec: *mut ExceptionRecord);
    fn ref___DestructExceptionObject(rec: *mut ExceptionRecord);

    // ── except.c: the filters and the trivia ────────────────────────────────────────────
    fn _XcptFilter(ex: c_int, ptr: *mut ExceptionPointers) -> c_int;
    fn ref__XcptFilter(ex: c_int, ptr: *mut ExceptionPointers) -> c_int;
    fn __CppXcptFilter(ex: c_int, ptr: *mut ExceptionPointers) -> c_int;
    fn ref___CppXcptFilter(ex: c_int, ptr: *mut ExceptionPointers) -> c_int;
    fn __CxxDetectRethrow(ptrs: *mut ExceptionPointers) -> c_int;
    fn ref___CxxDetectRethrow(ptrs: *mut ExceptionPointers) -> c_int;
    fn __CxxExceptionFilter(
        ptrs: *mut ExceptionPointers, ti: *const c_void, flags: c_uint, copy: *mut *mut c_void,
    ) -> c_int;
    fn ref___CxxExceptionFilter(
        ptrs: *mut ExceptionPointers, ti: *const c_void, flags: c_uint, copy: *mut *mut c_void,
    ) -> c_int;
    fn __CxxQueryExceptionSize() -> c_uint;
    fn ref___CxxQueryExceptionSize() -> c_uint;
    fn __intrinsic_abnormal_termination() -> c_int;
    fn ref___intrinsic_abnormal_termination() -> c_int;
    fn __uncaught_exception() -> c_int;
    fn ref___uncaught_exception() -> c_int;

    // ── cppimpl.c: the harness's own instruments ────────────────────────────────────────
    fn ref_run_throw(f: unsafe extern "C" fn(*mut c_void), ctx: *mut c_void) -> c_int;
    fn ref_raise_fired() -> c_int;
    fn ref_raise_code() -> u32;
    fn ref_raise_flags() -> u32;
    fn ref_raise_nargs() -> u32;
    fn ref_raise_arg(i: c_uint) -> usize;
    fn ref_console_handler_get() -> *mut c_void;
    fn ref_console_installs_get() -> c_int;
    fn ref_console_removes_get() -> c_int;
    fn ref_console_reset();
    fn ref_cpp_exit_expect(on: c_int);
    fn ref_sizeof_exception() -> usize;
    fn ref_off_exception_name() -> usize;
    fn ref_off_exception_do_free() -> usize;
    fn ref_sizeof_type_info() -> usize;
    fn ref_off_type_info_name() -> usize;
    fn ref_off_type_info_mangled() -> usize;
}

static mut ROWS: usize = 0;
fn row() {
    unsafe { ROWS += 1 }
}

/// Every row is `(subject, reference)` and every one of them is this assertion. Kept as one
/// function so the message shape is identical everywhere and a DIVERGE reads the same however
/// it was reached.
fn same<T: PartialEq + core::fmt::Debug>(what: &str, subj: T, refr: T) {
    assert_eq!(subj, refr, "{what}: SUBJECT vs REFERENCE diverge");
    row();
}

// ── strings ──────────────────────────────────────────────────────────────────────────────

unsafe fn cstr(p: *const c_char) -> Option<Vec<u8>> {
    if p.is_null() {
        return None;
    }
    let mut out = Vec::new();
    let mut i = 0isize;
    while *p.offset(i) != 0 {
        out.push(*p.offset(i) as u8);
        i += 1;
        assert!(i < 4096, "unterminated string from the code under test");
    }
    Some(out)
}

// ── the layout gate ──────────────────────────────────────────────────────────────────────

/// Runs before any row. The two `#[repr(C)]` mirrors above are transcriptions, and a mirror
/// that is wrong does not fail — it silently grades the wrong bytes and reports MATCH.
fn check_layout() {
    unsafe {
        assert_eq!(core::mem::size_of::<Exception>(), ref_sizeof_exception(), "sizeof(exception)");
        assert_eq!(
            core::mem::offset_of!(Exception, name),
            ref_off_exception_name(),
            "exception.name"
        );
        assert_eq!(
            core::mem::offset_of!(Exception, do_free),
            ref_off_exception_do_free(),
            "exception.do_free"
        );
        assert_eq!(core::mem::size_of::<TypeInfo>(), ref_sizeof_type_info(), "sizeof(type_info)");
        assert_eq!(
            core::mem::offset_of!(TypeInfo, name),
            ref_off_type_info_name(),
            "type_info.name"
        );
        assert_eq!(
            core::mem::offset_of!(TypeInfo, mangled),
            ref_off_type_info_mangled(),
            "type_info.mangled"
        );
    }
}

// ── fork: the only honest way to grade an arm that terminates ────────────────────────────

/// Runs `f` in a child and returns the RAW wait status, which is what makes the comparison
/// meaningful: `_exit(3)` and death by SIGABRT are different statuses, and which one an arm
/// produces is exactly the behaviour under test. A child that returns from `f` exits 99, so
/// "the arm did not terminate at all" is its own distinguishable verdict.
fn child_status(f: impl FnOnce()) -> c_int {
    unsafe {
        let pid = libc::fork();
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            ref_cpp_exit_expect(1);
            f();
            libc::_exit(99);
        }
        let mut st: c_int = 0;
        libc::waitpid(pid, &mut st, 0);
        st
    }
}

// ── handler pointers, reduced to what means the same on both sides ───────────────────────

const SIG_DFL: usize = 0;
const SIG_IGN: usize = 1;
const SIG_ERR: usize = usize::MAX;

/// A returned `__sighandler_t` as a TAG. The two sides install different functions, so the
/// raw pointers can never be equal; only "which of the sentinels, or the one I just
/// installed" carries across.
fn tag(p: usize, mine: usize) -> &'static str {
    match p {
        SIG_DFL => "SIG_DFL",
        SIG_IGN => "SIG_IGN",
        SIG_ERR => "SIG_ERR",
        _ if p == mine => "MINE",
        _ => "OTHER",
    }
}

// ── the recording handlers ───────────────────────────────────────────────────────────────
//
// One pair per side. Shared storage would let the subject's call satisfy the reference's
// expectation — the §9.73 trap, one level up from data.

static mut SUBJ_LOG: Vec<(c_int, c_int)> = Vec::new();
static mut REF_LOG: Vec<(c_int, c_int)> = Vec::new();

fn log_reset() {
    unsafe {
        (*core::ptr::addr_of_mut!(SUBJ_LOG)).clear();
        (*core::ptr::addr_of_mut!(REF_LOG)).clear();
    }
}
fn log_get(side: usize) -> Vec<(c_int, c_int)> {
    unsafe {
        if side == 0 {
            (*core::ptr::addr_of!(SUBJ_LOG)).clone()
        } else {
            (*core::ptr::addr_of!(REF_LOG)).clone()
        }
    }
}

/// TWO shapes, because msvcrt calls a signal handler two different ways and the difference is
/// under test. Every signal but SIGFPE is called `handler(sig)`; SIGFPE is cast to
/// `float_handler` and called `handler(sig, subcode)`, where the subcode is `_FPE_EXPLICITGEN`
/// from `raise` and one of the seven `_FPE_*` codes from the SEH filter's mapping table.
///
/// A single two-parameter handler for both was the first attempt and it does not work: for the
/// one-argument calls the second parameter reads whatever was in the register, so the row
/// compared two pieces of stack garbage and diverged. The shape has to match the call.
unsafe extern "C" fn h1_subj(sig: c_int) {
    (*core::ptr::addr_of_mut!(SUBJ_LOG)).push((sig, -1));
}
unsafe extern "C" fn h1_ref(sig: c_int) {
    (*core::ptr::addr_of_mut!(REF_LOG)).push((sig, -1));
}
unsafe extern "C" fn h2_subj(sig: c_int, sub: c_int) {
    (*core::ptr::addr_of_mut!(SUBJ_LOG)).push((sig, sub));
}
unsafe extern "C" fn h2_ref(sig: c_int, sub: c_int) {
    (*core::ptr::addr_of_mut!(REF_LOG)).push((sig, sub));
}

/// The handler a row installs for `sig`, as a raw address. SIGFPE is the two-parameter one.
fn handler_for(sig: c_int, subject: bool) -> usize {
    if sig == sys::SIGFPE as c_int {
        if subject { h2_subj as usize } else { h2_ref as usize }
    } else if subject {
        h1_subj as usize
    } else {
        h1_ref as usize
    }
}

static mut SUBJ_VOID: c_int = 0;
static mut REF_VOID: c_int = 0;
unsafe extern "C" fn v_subj() {
    SUBJ_VOID += 1;
}
unsafe extern "C" fn v_ref() {
    REF_VOID += 1;
}

// A second void handler, so "previous handler" rows can tell MINE from a DIFFERENT one of
// mine — a `set_terminate` that returned the value it was passed instead of the value it
// replaced would otherwise look right.
unsafe extern "C" fn v2_subj() {}
unsafe extern "C" fn v2_ref() {}

unsafe extern "C" fn se_subj(_c: c_uint, _p: *mut ExceptionPointers) {}
unsafe extern "C" fn se_ref(_c: c_uint, _p: *mut ExceptionPointers) {}

// ── fixtures ─────────────────────────────────────────────────────────────────────────────

/// A `type_info` carrying `mangled`. The leading '.' is not decoration: cpp.c's own comment
/// says the mangled name in a type_info always starts with one and that it is not part of a
/// valid mangled name, and every one of the four readers here skips it with `mangled + 1`. A
/// fixture without it would grade the demangler on a string msvcrt never hands it.
fn mk_type_info(mangled: &str) -> Box<TypeInfo> {
    let mut ti = Box::new(TypeInfo {
        vtable: core::ptr::null(),
        name: core::ptr::null_mut(),
        mangled: [0u8; 128],
    });
    let b = mangled.as_bytes();
    assert!(b.len() < 128, "fixture longer than cxx.h's mangled tail");
    ti.mangled[..b.len()].copy_from_slice(b);
    ti
}

const MANGLED: &[&str] = &[
    ".?AVexception@@",
    ".?AVbad_cast@std@@",
    ".?AVbad_typeid@std@@",
    ".?AV__non_rtti_object@std@@",
    ".?AVfoo@@",
    ".?AUbar@@",
    ".PAX",
    ".H",
    // Not decodable: `__unDName` returns NULL and `type_info_name` leaves `name` NULL, which
    // is the arm a caller sees as "no demangled name" rather than an empty string.
    ".@@@",
    ".",
    // Trailing blanks are the reason type_info_name has a trim loop at all — cpp.c says
    // "_unDName may leave blanks at the end of the demangled name".
    ".?AV?$basic_string@DU?$char_traits@D@std@@V?$allocator@D@2@@std@@",
];

/// A heap `exception` whose storage the object's own `operator_delete` may free. Rows that
/// pass `flags & 1` to a scalar dtor need this; a stack object would be freed by the code
/// under test and corrupt the harness rather than fail a row.
unsafe fn heap_exception() -> *mut Exception {
    libc::calloc(1, core::mem::size_of::<Exception>()) as *mut Exception
}

/// An ARRAY of `exception` with the element count stored in the word BEFORE the first object,
/// which is the layout `*_vector_dtor` reads when `flags & 2`. Returns the pointer to element
/// zero, which is what the C is handed.
unsafe fn heap_exception_array(n: usize) -> *mut Exception {
    let sz = core::mem::size_of::<Exception>();
    let raw = libc::calloc(1, core::mem::size_of::<usize>() + n * sz) as *mut usize;
    *raw = n;
    raw.add(1) as *mut Exception
}

fn main() {
    check_layout();
    eprintln!("\ncpp.c + except.c differential (reference TUs #14 / #15)\n");

    unsafe {
        // ═══ cpp.c: type_info ═════════════════════════════════════════════════════════
        //
        // `type_info_name` is the interesting one: it demangles LAZILY through __unDName,
        // trims trailing blanks, and installs the result with an interlocked compare-exchange
        // so a losing thread frees its own copy. Every part of that is observable — the
        // string, the caching (second call must return the SAME pointer), and the NULL-stays-
        // NULL arm for a name the demangler refuses.
        for m in MANGLED {
            let (mut s, mut r) = (mk_type_info(m), mk_type_info(m));

            same(
                &format!("type_info_raw_name({m})"),
                cstr(type_info_raw_name(&mut *s)),
                cstr(ref_type_info_raw_name(&mut *r)),
            );

            let (sn, rn) = (type_info_name(&mut *s), ref_type_info_name(&mut *r));
            same(&format!("type_info_name({m})"), cstr(sn), cstr(rn));
            // The cache, which the value comparison above is blind to: a body that
            // demangled afresh every call would return equal strings from different
            // allocations and score MATCH, while leaking one per call in the DLL.
            same(
                &format!("type_info_name({m}) caches"),
                (type_info_name(&mut *s) == sn, s.name.is_null()),
                (ref_type_info_name(&mut *r) == rn, r.name.is_null()),
            );

            type_info_dtor(&mut *s);
            ref_type_info_dtor(&mut *r);
            // The dtor frees `name` and — faithfully to the C — does NOT clear it. Re-null it
            // here so the box can be dropped without a double free on the next fixture.
            s.name = core::ptr::null_mut();
            r.name = core::ptr::null_mut();
            row();
        }

        // The comparison trio, over every ordered pair including self-comparison. All three
        // read `mangled + 1`, so the '.' skip is under test as much as the strcmp is.
        for a in MANGLED {
            for b in MANGLED {
                let (mut sa, sb) = (mk_type_info(a), mk_type_info(b));
                let (mut ra, rb) = (mk_type_info(a), mk_type_info(b));
                same(
                    &format!("type_info_opequals_equals({a},{b})"),
                    type_info_opequals_equals(&mut *sa, &*sb),
                    ref_type_info_opequals_equals(&mut *ra, &*rb),
                );
                same(
                    &format!("type_info_opnot_equals({a},{b})"),
                    type_info_opnot_equals(&mut *sa, &*sb),
                    ref_type_info_opnot_equals(&mut *ra, &*rb),
                );
                same(
                    &format!("type_info_before({a},{b})"),
                    type_info_before(&mut *sa, &*sb),
                    ref_type_info_before(&mut *ra, &*rb),
                );
            }
        }

        // ═══ cpp.c: the exception object family ═══════════════════════════════════════
        //
        // Five classes with the same shape — `exception` and the three typedefs of it, each
        // with its own vtable and rtti. The VTABLE POINTER is deliberately not compared: the
        // two sides install their own, and the addresses mean nothing across them. What is
        // compared is `name` (contents, not pointer), `do_free`, and what `what()` reports.
        let names: &[Option<&str>] = &[
            Some("a message"),
            Some(""),
            Some("x"),
            // 'Unknown exception' is what what() reports for a NULL name, and the ctors reach
            // it by two different routes — default_ctor, and ctor with a NULL name pointer.
            None,
        ];

        for nm in names {
            let cs = nm.map(|s| format!("{s}\0"));
            let p: *const c_char =
                cs.as_ref().map_or(core::ptr::null(), |s| s.as_ptr() as *const c_char);

            let (mut s, mut r) = (
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 },
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 },
            );
            exception_ctor(&mut s, &p);
            ref_exception_ctor(&mut r, &p);
            same(
                &format!("exception_ctor({nm:?})"),
                (cstr(s.name), s.do_free, cstr(exception_what(&mut s))),
                (cstr(r.name), r.do_free, cstr(ref_exception_what(&mut r))),
            );

            // The copy: `name` must be a NEW allocation with the same contents — cxx.h says
            // "always a new copy for each object" — so a copy_ctor that aliased the source
            // would double-free in the DLL and is caught here by the pointer inequality.
            let (mut s2, mut r2) = (
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 },
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 },
            );
            exception_copy_ctor(&mut s2, &s);
            ref_exception_copy_ctor(&mut r2, &r);
            same(
                &format!("exception_copy_ctor({nm:?})"),
                (cstr(s2.name), s2.do_free, s2.name == s.name && !s.name.is_null()),
                (cstr(r2.name), r2.do_free, r2.name == r.name && !r.name.is_null()),
            );

            // operator=, whose body is dtor-then-copy-ctor with a self-assignment guard.
            exception_opequals(&mut s2, &s);
            ref_exception_opequals(&mut r2, &r);
            same(
                &format!("exception_opequals({nm:?})"),
                (cstr(s2.name), s2.do_free),
                (cstr(r2.name), r2.do_free),
            );
            // Self-assignment: the guard is the whole body here, and without it this frees
            // the name and then copies from the freed pointer.
            exception_opequals(&mut s2, &s2);
            ref_exception_opequals(&mut r2, &r2);
            same(
                &format!("exception_opequals(self,{nm:?})"),
                (cstr(s2.name), s2.do_free),
                (cstr(r2.name), r2.do_free),
            );

            exception_dtor(&mut s2);
            ref_exception_dtor(&mut r2);
            exception_dtor(&mut s);
            ref_exception_dtor(&mut r);
            row();
        }

        // default_ctor and ctor_noalloc, which are the two ways to get do_free == FALSE.
        {
            let (mut s, mut r) = (
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 7 },
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 7 },
            );
            exception_default_ctor(&mut s);
            ref_exception_default_ctor(&mut r);
            same(
                "exception_default_ctor",
                (cstr(s.name), s.do_free, cstr(exception_what(&mut s))),
                (cstr(r.name), r.do_free, cstr(ref_exception_what(&mut r))),
            );

            // ctor_noalloc TAKES the caller's string rather than copying it, and records
            // do_free = FALSE so the dtor leaves it alone. A translation that copied anyway
            // would look identical through `what()` and leak on every throw.
            let mut lit = *b"borrowed\0";
            let mut pl = lit.as_mut_ptr() as *mut c_char;
            let (mut s, mut r) = (
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 7 },
                Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 7 },
            );
            exception_ctor_noalloc(&mut s, &mut pl, 1);
            ref_exception_ctor_noalloc(&mut r, &mut pl, 1);
            same(
                "exception_ctor_noalloc",
                (cstr(s.name), s.do_free, s.name == pl),
                (cstr(r.name), r.do_free, r.name == pl),
            );
            exception_dtor(&mut s);
            ref_exception_dtor(&mut r);
        }

        // The three typedef classes. Their ctors differ in ONE way that matters and is easy
        // to get backwards: bad_cast_ctor takes `const char **`, bad_cast_ctor_charptr and
        // bad_typeid_ctor take `const char *`.
        {
            let msg = b"typed\0".as_ptr() as *const c_char;
            let msgp = &msg as *const *const c_char;

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_typeid_ctor(&mut s, msg);
            ref_bad_typeid_ctor(&mut r, msg);
            same("bad_typeid_ctor", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));

            let mut s2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_typeid_copy_ctor(&mut s2, &s);
            ref_bad_typeid_copy_ctor(&mut r2, &r);
            same("bad_typeid_copy_ctor", (cstr(s2.name), s2.do_free), (cstr(r2.name), r2.do_free));
            bad_typeid_opequals(&mut s2, &s);
            ref_bad_typeid_opequals(&mut r2, &r);
            same("bad_typeid_opequals", (cstr(s2.name), s2.do_free), (cstr(r2.name), r2.do_free));
            bad_typeid_dtor(&mut s2);
            ref_bad_typeid_dtor(&mut r2);
            bad_typeid_dtor(&mut s);
            ref_bad_typeid_dtor(&mut r);
            row();

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_typeid_default_ctor(&mut s);
            ref_bad_typeid_default_ctor(&mut r);
            same("bad_typeid_default_ctor", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));
            bad_typeid_dtor(&mut s);
            ref_bad_typeid_dtor(&mut r);

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_cast_ctor(&mut s, msgp);
            ref_bad_cast_ctor(&mut r, msgp);
            same("bad_cast_ctor", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));

            let mut s2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_cast_copy_ctor(&mut s2, &s);
            ref_bad_cast_copy_ctor(&mut r2, &r);
            same("bad_cast_copy_ctor", (cstr(s2.name), s2.do_free), (cstr(r2.name), r2.do_free));
            bad_cast_opequals(&mut s2, &s);
            ref_bad_cast_opequals(&mut r2, &r);
            same("bad_cast_opequals", (cstr(s2.name), s2.do_free), (cstr(r2.name), r2.do_free));
            bad_cast_dtor(&mut s2);
            ref_bad_cast_dtor(&mut r2);
            bad_cast_dtor(&mut s);
            ref_bad_cast_dtor(&mut r);
            row();

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_cast_ctor_charptr(&mut s, msg);
            ref_bad_cast_ctor_charptr(&mut r, msg);
            same("bad_cast_ctor_charptr", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));
            bad_cast_dtor(&mut s);
            ref_bad_cast_dtor(&mut r);

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            bad_cast_default_ctor(&mut s);
            ref_bad_cast_default_ctor(&mut r);
            same("bad_cast_default_ctor", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));
            bad_cast_dtor(&mut s);
            ref_bad_cast_dtor(&mut r);

            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            __non_rtti_object_ctor(&mut s, msg);
            ref___non_rtti_object_ctor(&mut r, msg);
            same("__non_rtti_object_ctor", (cstr(s.name), s.do_free), (cstr(r.name), r.do_free));

            let mut s2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r2 = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            __non_rtti_object_copy_ctor(&mut s2, &s);
            ref___non_rtti_object_copy_ctor(&mut r2, &r);
            same(
                "__non_rtti_object_copy_ctor",
                (cstr(s2.name), s2.do_free),
                (cstr(r2.name), r2.do_free),
            );
            __non_rtti_object_opequals(&mut s2, &s);
            ref___non_rtti_object_opequals(&mut r2, &r);
            same(
                "__non_rtti_object_opequals",
                (cstr(s2.name), s2.do_free),
                (cstr(r2.name), r2.do_free),
            );
            __non_rtti_object_dtor(&mut s2);
            ref___non_rtti_object_dtor(&mut r2);
            __non_rtti_object_dtor(&mut s);
            ref___non_rtti_object_dtor(&mut r);
            row();
        }

        // ═══ the deleting destructors ═════════════════════════════════════════════════
        //
        // `flags & 1` on a scalar dtor and `flags & 2` on a vector dtor both hand the storage
        // to `operator_delete`, so the objects have to be heap ones with the right shape —
        // and the vector form reads the element count out of the word BEFORE element zero.
        // The verdict is the returned pointer (always `_this`) plus survival: a dtor that got
        // the count or the stride wrong walks off the allocation, which the allocator or the
        // guard page reports rather than a value comparison.
        for flags in [0u32, 1] {
            let (s, r) = (heap_exception(), heap_exception());
            exception_default_ctor(s);
            ref_exception_default_ctor(r);
            same(
                &format!("exception_scalar_dtor(flags={flags})"),
                exception_scalar_dtor(s, flags) == s as *mut c_void,
                ref_exception_scalar_dtor(r, flags) == r as *mut c_void,
            );
            if flags & 1 == 0 {
                libc::free(s as *mut c_void);
                libc::free(r as *mut c_void);
            }

            let (s, r) = (heap_exception(), heap_exception());
            bad_typeid_default_ctor(s);
            ref_bad_typeid_default_ctor(r);
            same(
                &format!("bad_typeid_scalar_dtor(flags={flags})"),
                bad_typeid_scalar_dtor(s, flags) == s as *mut c_void,
                ref_bad_typeid_scalar_dtor(r, flags) == r as *mut c_void,
            );
            if flags & 1 == 0 {
                libc::free(s as *mut c_void);
                libc::free(r as *mut c_void);
            }

            let (s, r) = (heap_exception(), heap_exception());
            bad_cast_default_ctor(s);
            ref_bad_cast_default_ctor(r);
            same(
                &format!("bad_cast_scalar_dtor(flags={flags})"),
                bad_cast_scalar_dtor(s, flags) == s as *mut c_void,
                ref_bad_cast_scalar_dtor(r, flags) == r as *mut c_void,
            );
            if flags & 1 == 0 {
                libc::free(s as *mut c_void);
                libc::free(r as *mut c_void);
            }

            let (s, r) = (heap_exception(), heap_exception());
            bad_typeid_default_ctor(s);
            ref_bad_typeid_default_ctor(r);
            same(
                &format!("__non_rtti_object_scalar_dtor(flags={flags})"),
                __non_rtti_object_scalar_dtor(s, flags) == s as *mut c_void,
                ref___non_rtti_object_scalar_dtor(r, flags) == r as *mut c_void,
            );
            if flags & 1 == 0 {
                libc::free(s as *mut c_void);
                libc::free(r as *mut c_void);
            }
        }

        for n in [1usize, 2, 5] {
            let (s, r) = (heap_exception_array(n), heap_exception_array(n));
            for i in 0..n {
                exception_default_ctor(s.add(i));
                ref_exception_default_ctor(r.add(i));
            }
            same(
                &format!("exception_vector_dtor(flags=2,n={n})"),
                exception_vector_dtor(s, 2) == s as *mut c_void,
                ref_exception_vector_dtor(r, 2) == r as *mut c_void,
            );

            let (s, r) = (heap_exception_array(n), heap_exception_array(n));
            for i in 0..n {
                bad_typeid_default_ctor(s.add(i));
                ref_bad_typeid_default_ctor(r.add(i));
            }
            same(
                &format!("bad_typeid_vector_dtor(flags=2,n={n})"),
                bad_typeid_vector_dtor(s, 2) == s as *mut c_void,
                ref_bad_typeid_vector_dtor(r, 2) == r as *mut c_void,
            );

            let (s, r) = (heap_exception_array(n), heap_exception_array(n));
            for i in 0..n {
                bad_cast_default_ctor(s.add(i));
                ref_bad_cast_default_ctor(r.add(i));
            }
            same(
                &format!("bad_cast_vector_dtor(flags=2,n={n})"),
                bad_cast_vector_dtor(s, 2) == s as *mut c_void,
                ref_bad_cast_vector_dtor(r, 2) == r as *mut c_void,
            );

            let (s, r) = (heap_exception_array(n), heap_exception_array(n));
            for i in 0..n {
                bad_typeid_default_ctor(s.add(i));
                ref_bad_typeid_default_ctor(r.add(i));
            }
            same(
                &format!("__non_rtti_object_vector_dtor(flags=2,n={n})"),
                __non_rtti_object_vector_dtor(s, 2) == s as *mut c_void,
                ref___non_rtti_object_vector_dtor(r, 2) == r as *mut c_void,
            );
        }

        // The non-deleting arm of the vector dtors, where the object is a single stack one and
        // no count word exists.
        {
            let mut s = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            let mut r = Exception { vtable: core::ptr::null(), name: core::ptr::null_mut(), do_free: 0 };
            exception_default_ctor(&mut s);
            ref_exception_default_ctor(&mut r);
            same(
                "exception_vector_dtor(flags=0)",
                exception_vector_dtor(&mut s, 0) == (&mut s) as *mut Exception as *mut c_void,
                ref_exception_vector_dtor(&mut r, 0) == (&mut r) as *mut Exception as *mut c_void,
            );
        }

        // ═══ cpp.c: the per-thread handlers ═══════════════════════════════════════════
        //
        // Three independent slots in thread_data. Each row installs, reads back, and replaces
        // — the replace is what catches a body that returns its ARGUMENT instead of the value
        // it displaced, which reading back alone cannot see.
        {
            let (a_s, a_r): (VoidFn, VoidFn) = (Some(v_subj), Some(v_ref));
            let (b_s, b_r): (VoidFn, VoidFn) = (Some(v2_subj), Some(v2_ref));
            let fp = |f: VoidFn| f.map_or(0usize, |x| x as usize);

            same(
                "set_terminate: first install returns NULL",
                fp(set_terminate(a_s)),
                fp(ref_set_terminate(a_r)),
            );
            same(
                "_get_terminate reads it back",
                _get_terminate().map(|x| x as usize) == Some(fp(a_s)),
                ref__get_terminate().map(|x| x as usize) == Some(fp(a_r)),
            );
            same(
                "set_terminate returns the DISPLACED handler",
                fp(set_terminate(b_s)) == fp(a_s),
                fp(ref_set_terminate(b_r)) == fp(a_r),
            );
            same(
                "set_terminate(NULL) clears",
                (fp(set_terminate(None)) == fp(b_s), _get_terminate().is_none()),
                (fp(ref_set_terminate(None)) == fp(b_r), ref__get_terminate().is_none()),
            );

            same(
                "set_unexpected: first install returns NULL",
                fp(set_unexpected(a_s)),
                fp(ref_set_unexpected(a_r)),
            );
            same(
                "_get_unexpected reads it back",
                _get_unexpected().map(|x| x as usize) == Some(fp(a_s)),
                ref__get_unexpected().map(|x| x as usize) == Some(fp(a_r)),
            );
            same(
                "set_unexpected returns the DISPLACED handler",
                fp(set_unexpected(b_s)) == fp(a_s),
                fp(ref_set_unexpected(b_r)) == fp(a_r),
            );
            same(
                "set_unexpected(NULL) clears",
                (fp(set_unexpected(None)) == fp(b_s), _get_unexpected().is_none()),
                (fp(ref_set_unexpected(None)) == fp(b_r), ref__get_unexpected().is_none()),
            );

            let sef = |f: SeTranslator| f.map_or(0usize, |x| x as usize);
            let (t_s, t_r): (SeTranslator, SeTranslator) = (Some(se_subj), Some(se_ref));
            same(
                "_set_se_translator: first install returns NULL",
                sef(_set_se_translator(t_s)),
                sef(ref__set_se_translator(t_r)),
            );
            same(
                "_set_se_translator returns the DISPLACED handler",
                sef(_set_se_translator(None)) == sef(t_s),
                sef(ref__set_se_translator(None)) == sef(t_r),
            );
        }

        // `terminate` and `unexpected` both end the process, so they are graded under fork
        // and the verdict is the child's wait status. Three arms each, and the three are
        // genuinely different code paths: handler runs and returns (falls through to the next
        // stage), handler runs and exits (the C never regains control), no handler at all.
        {
            let st_s = child_status(|| {
                set_terminate(Some(exit77_subj));
                terminate();
            });
            let st_r = child_status(|| {
                ref_set_terminate(Some(exit77_ref));
                ref_terminate();
            });
            same("terminate: installed handler runs", st_s, st_r);

            // No handler: the C falls straight through to abort(), so the child dies by
            // signal rather than exiting. Comparing the RAW status is what makes that
            // distinguishable from an exit with the same number.
            let st_s = child_status(|| terminate());
            let st_r = child_status(|| ref_terminate());
            same("terminate: no handler reaches abort()", st_s, st_r);

            // unexpected() runs its own handler and then falls into terminate(), which with
            // no terminate handler installed is abort(). Both stages in one status.
            let st_s = child_status(|| {
                set_unexpected(Some(exit77_subj));
                unexpected();
            });
            let st_r = child_status(|| {
                ref_set_unexpected(Some(exit77_ref));
                ref_unexpected();
            });
            same("unexpected: installed handler runs", st_s, st_r);

            let st_s = child_status(|| unexpected());
            let st_r = child_status(|| ref_unexpected());
            same("unexpected: falls through to terminate()", st_s, st_r);
        }

        // ═══ cpp.c: RTTI init and the throw path ══════════════════════════════════════
        //
        // `msvcrt_init_exception` is what turns the static RTTI tables from absolute pointers
        // into the RVAs the x86-64 model uses, so nothing below it means anything until both
        // sides have run it against the SAME base — which is cppimpl.c's, since neither side's
        // `RtlPcToFileHeader` is renamed and both reach the one body.
        {
            let b = ref_image_base();
            msvcrt_init_exception(b);
            ref_msvcrt_init_exception(b);
            row();

            // `throw_exception` builds an `exception` and throws it, so it exercises the whole
            // marshalling in one call. The sink records; the row compares everything about the
            // record that is not an address.
            for msg in ["boom\0", "\0", "a longer message than the last one\0"] {
                THROW_MSG = msg.as_ptr() as *const c_char;
                let s = capture_throw(do_throw_subj);
                let r = capture_throw(do_throw_ref);
                same(&format!("throw_exception({msg:?})"), s, r);
            }
        }

        // ═══ except.c: the signal layer ═══════════════════════════════════════════════
        //
        // `signal` is a validation switch over msvcrt's SEVEN signals and a store into a file
        // static table; everything else is SIG_ERR. The sweep runs well past NSIG on purpose:
        // an implementation that bounds-checked with `<=` instead of `<`, or that accepted
        // every signal glibc has, differs from the C only outside the seven.
        let sigs: Vec<c_int> = (-2..=26)
            .chain([sys::SIGABRT as c_int, sys::NSIG as c_int, 100, c_int::MAX])
            .collect();

        for &sig in &sigs {
            for (what, h_s, h_r) in [
                ("SIG_DFL", SIG_DFL, SIG_DFL),
                ("SIG_IGN", SIG_IGN, SIG_IGN),
                ("SIG_ERR", SIG_ERR, SIG_ERR),
                ("handler", handler_for(sig, true), handler_for(sig, false)),
            ] {
                let ps = signal(sig, core::mem::transmute::<usize, SigHandler>(h_s));
                let pr = ref_signal(sig, core::mem::transmute::<usize, SigHandler>(h_r));
                same(
                    &format!("signal({sig},{what})"),
                    tag(ps.map_or(0, |f| f as usize), handler_for(sig, true)),
                    tag(pr.map_or(0, |f| f as usize), handler_for(sig, false)),
                );
            }
        }

        // Reset the table to SIG_DFL on both sides before the raise rows, which are the ones
        // that CARE what is installed.
        for sig in 0..(sys::NSIG as c_int + 2) {
            signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
            ref_signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
        }

        // `raise` with a handler installed. Three things are under test at once and only one
        // of them is the return value: WHICH argument the handler is called with (SIGFPE gets
        // the `_FPE_EXPLICITGEN` subcode and nothing else does), and that the handler is
        // RESET to SIG_DFL before the call — the one-shot contract, which a body that reset
        // it after would pass a value comparison on.
        for &sig in &[
            sys::SIGABRT as c_int,
            sys::SIGINT as c_int,
            sys::SIGTERM as c_int,
            sys::SIGBREAK as c_int,
            sys::SIGFPE as c_int,
            sys::SIGILL as c_int,
            sys::SIGSEGV as c_int,
        ] {
            log_reset();
            signal(sig, core::mem::transmute::<usize, SigHandler>(handler_for(sig, true)));
            ref_signal(sig, core::mem::transmute::<usize, SigHandler>(handler_for(sig, false)));

            let rs = raise(sig);
            let rr = ref_raise(sig);
            same(&format!("raise({sig}) return"), rs, rr);
            same(&format!("raise({sig}) handler arguments"), log_get(0), log_get(1));

            // The one-shot reset, read back by installing SIG_IGN and looking at what comes
            // out. SIG_DFL means `raise` cleared the slot before dispatching.
            let ps = signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_IGN));
            let pr = ref_signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_IGN));
            same(
                &format!("raise({sig}) resets the handler to SIG_DFL"),
                tag(ps.map_or(0, |f| f as usize), handler_for(sig, true)),
                tag(pr.map_or(0, |f| f as usize), handler_for(sig, false)),
            );

            // SIG_IGN is now installed: raise must return 0 having called nothing, and must
            // NOT clear the slot.
            log_reset();
            same(&format!("raise({sig}) with SIG_IGN"), raise(sig), ref_raise(sig));
            same(&format!("raise({sig}) SIG_IGN calls nothing"), log_get(0), log_get(1));
            let ps = signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
            let pr = ref_signal(sig, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
            same(
                &format!("raise({sig}) leaves SIG_IGN in place"),
                tag(ps.map_or(0, |f| f as usize), handler_for(sig, true)),
                tag(pr.map_or(0, |f| f as usize), handler_for(sig, false)),
            );
        }

        // The signals `raise` refuses.
        for &sig in &[-1, 0, 1, 3, 5, 20, sys::NSIG as c_int, 100] {
            same(&format!("raise({sig}) rejected"), raise(sig), ref_raise(sig));
        }

        // SIG_DFL is `_exit(3)`, for all seven. Under fork, because both sides really do it.
        for &sig in &[
            sys::SIGABRT as c_int,
            sys::SIGINT as c_int,
            sys::SIGTERM as c_int,
            sys::SIGBREAK as c_int,
            sys::SIGFPE as c_int,
            sys::SIGILL as c_int,
            sys::SIGSEGV as c_int,
        ] {
            let st_s = child_status(|| {
                raise(sig);
            });
            let st_r = child_status(|| {
                ref_raise(sig);
            });
            same(&format!("raise({sig}) with SIG_DFL terminates"), st_s, st_r);
        }

        // init/free_signals: the only observable is the SetConsoleCtrlHandler traffic, which
        // cppimpl.c records. The handler POINTER differs between the sides, so what is
        // compared is the shape — one install, then one remove OF THE SAME POINTER, leaving
        // nothing registered. A `free` that passed a different handler would leave the DLL's
        // console handler installed for the life of the process.
        {
            ref_console_reset();
            msvcrt_init_signals();
            let s_installed = !ref_console_handler_get().is_null();
            let s_ins = ref_console_installs_get();
            msvcrt_free_signals();
            let s = (s_installed, s_ins, ref_console_removes_get(), ref_console_handler_get().is_null());

            ref_console_reset();
            ref_msvcrt_init_signals();
            let r_installed = !ref_console_handler_get().is_null();
            let r_ins = ref_console_installs_get();
            ref_msvcrt_free_signals();
            let r = (r_installed, r_ins, ref_console_removes_get(), ref_console_handler_get().is_null());

            same("msvcrt_init_signals / msvcrt_free_signals", s, r);
        }

        // `__pxcptinfoptrs` hands out the ADDRESS of a thread_data slot, so the row writes
        // through it and reads back — the pointers themselves are per-side and meaningless.
        {
            let (ps, pr) = (__pxcptinfoptrs(), ref___pxcptinfoptrs());
            same("__pxcptinfoptrs is non-NULL", ps.is_null(), pr.is_null());
            *ps = 0x1234 as *mut c_void;
            *pr = 0x1234 as *mut c_void;
            same("__pxcptinfoptrs round-trips", *__pxcptinfoptrs(), *ref___pxcptinfoptrs());
            *ps = core::ptr::null_mut();
            *pr = core::ptr::null_mut();
        }

        // ═══ except.c: the frame-info list ════════════════════════════════════════════
        //
        // A per-thread singly-linked stack, and its only observable is
        // `_IsExceptionObjectToBeDestroyed` — which walks it looking for an object. So the
        // rows drive push/pop sequences and ask about membership after each, which grades the
        // LINKAGE without reading the head: unlink-from-the-middle, unlink-the-head, and
        // unlink-something-not-there (whose C comment says native crashes, and which must
        // therefore leave the list intact rather than corrupt it).
        {
            let objs: Vec<usize> = (1..=4).map(|i| i * 0x1000).collect();
            let mut sf: Vec<FrameInfo> = (0..4)
                .map(|_| FrameInfo { object: core::ptr::null_mut(), next: core::ptr::null_mut() })
                .collect();
            let mut rf = sf.clone();

            // Push all four, checking membership grows correctly.
            for i in 0..4 {
                let ret_s = _CreateFrameInfo(&mut sf[i], objs[i] as *mut c_void);
                let ret_r = ref__CreateFrameInfo(&mut rf[i], objs[i] as *mut c_void);
                same(
                    &format!("_CreateFrameInfo #{i} returns its argument"),
                    ret_s == &mut sf[i] as *mut FrameInfo,
                    ret_r == &mut rf[i] as *mut FrameInfo,
                );
                same(
                    &format!("_CreateFrameInfo #{i} records the object"),
                    sf[i].object as usize,
                    rf[i].object as usize,
                );
                for j in 0..4 {
                    same(
                        &format!("membership after push {i}, probe {j}"),
                        _IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                        ref__IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                    );
                }
            }

            // Unlink from the MIDDLE, which is the loop arm rather than the head shortcut.
            _FindAndUnlinkFrame(&mut sf[1]);
            ref__FindAndUnlinkFrame(&mut rf[1]);
            for j in 0..4 {
                same(
                    &format!("membership after middle unlink, probe {j}"),
                    _IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                    ref__IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                );
            }

            // Unlink the HEAD, the early-return arm.
            _FindAndUnlinkFrame(&mut sf[3]);
            ref__FindAndUnlinkFrame(&mut rf[3]);
            for j in 0..4 {
                same(
                    &format!("membership after head unlink, probe {j}"),
                    _IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                    ref__IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                );
            }

            // Unlink something that is NOT on the list. The C walks off the end and logs;
            // what matters is that the list survives, which the probes after it establish.
            let mut orphan_s = FrameInfo { object: 0x9999 as *mut c_void, next: core::ptr::null_mut() };
            let mut orphan_r = FrameInfo { object: 0x9999 as *mut c_void, next: core::ptr::null_mut() };
            _FindAndUnlinkFrame(&mut orphan_s);
            ref__FindAndUnlinkFrame(&mut orphan_r);
            for j in 0..4 {
                same(
                    &format!("membership after orphan unlink, probe {j}"),
                    _IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                    ref__IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                );
            }

            // Drain, so the registration rows below start from an empty list.
            _FindAndUnlinkFrame(&mut sf[2]);
            ref__FindAndUnlinkFrame(&mut rf[2]);
            _FindAndUnlinkFrame(&mut sf[0]);
            ref__FindAndUnlinkFrame(&mut rf[0]);
            for j in 0..4 {
                same(
                    &format!("membership after drain, probe {j}"),
                    _IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                    ref__IsExceptionObjectToBeDestroyed(objs[j] as *const c_void),
                );
            }
        }

        // ═══ except.c: exception-object registration ══════════════════════════════════
        //
        // Register/unregister is the frame-info list plus a two-slot save/restore of the
        // thread's current EXCEPTION_RECORD and CONTEXT. The NULL arm is the one worth
        // spelling out: it writes the sentinel (void*)-1 into both slots and pushes NOTHING,
        // and unregister has to recognise that and return without touching the list — an
        // implementation that unlinked anyway would corrupt a list it never joined.
        {
            let mut rec = mk_cxx_record(0x4242);
            let mut ep = ExceptionPointers {
                ExceptionRecord: &mut rec,
                ContextRecord: 0x5150 as *mut c_void,
            };
            let mut sfi = zero_cxx_frame();
            let mut rfi = zero_cxx_frame();

            same(
                "__CxxRegisterExceptionObject",
                __CxxRegisterExceptionObject(&mut ep, &mut sfi),
                ref___CxxRegisterExceptionObject(&mut ep, &mut rfi),
            );
            same(
                "…pushes the thrown object onto the frame list",
                _IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
                ref__IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
            );
            same(
                "…saves the previous record/context (both NULL here)",
                (sfi.rec as usize, sfi.context as usize),
                (rfi.rec as usize, rfi.context as usize),
            );

            // `in_use` = TRUE, so the object is NOT destructed on the way out. The record has
            // no destructor, so what this grades is the unlink and the restore.
            __CxxUnregisterExceptionObject(&mut sfi, 1);
            ref___CxxUnregisterExceptionObject(&mut rfi, 1);
            same(
                "__CxxUnregisterExceptionObject pops the object",
                _IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
                ref__IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
            );

            // The NULL arm.
            let mut sfi = zero_cxx_frame();
            let mut rfi = zero_cxx_frame();
            same(
                "__CxxRegisterExceptionObject(NULL)",
                __CxxRegisterExceptionObject(core::ptr::null_mut(), &mut sfi),
                ref___CxxRegisterExceptionObject(core::ptr::null_mut(), &mut rfi),
            );
            same(
                "…writes the (void*)-1 sentinel into both slots",
                (sfi.rec as usize, sfi.context as usize),
                (rfi.rec as usize, rfi.context as usize),
            );
            __CxxUnregisterExceptionObject(&mut sfi, 0);
            ref___CxxUnregisterExceptionObject(&mut rfi, 0);
            same(
                "…and unregister returns without touching the list",
                _IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
                ref__IsExceptionObjectToBeDestroyed(0x4242 as *const c_void),
            );

            // An EXCEPTION_POINTERS with a NULL record takes the same sentinel arm through a
            // different test.
            let mut ep0 = ExceptionPointers {
                ExceptionRecord: core::ptr::null_mut(),
                ContextRecord: core::ptr::null_mut(),
            };
            let mut sfi = zero_cxx_frame();
            let mut rfi = zero_cxx_frame();
            same(
                "__CxxRegisterExceptionObject(ep with NULL record)",
                (
                    __CxxRegisterExceptionObject(&mut ep0, &mut sfi),
                    sfi.rec as usize,
                    sfi.context as usize,
                ),
                (
                    ref___CxxRegisterExceptionObject(&mut ep0, &mut rfi),
                    rfi.rec as usize,
                    rfi.context as usize,
                ),
            );
            __CxxUnregisterExceptionObject(&mut sfi, 0);
            ref___CxxUnregisterExceptionObject(&mut rfi, 0);
        }

        // `__DestructExceptionObject` on a record that is not a C++ exception, and on one that
        // is but has no descriptor: both are early returns, and both are the arms a caller
        // reaches for every non-C++ SEH exception in the process.
        {
            let mut plain = mk_cxx_record(0x11);
            plain.ExceptionCode = 0xC0000005;
            __DestructExceptionObject(&mut plain);
            ref___DestructExceptionObject(&mut plain);
            row();

            let mut nodescr = mk_cxx_record(0x11);
            nodescr.ExceptionInformation[2] = 0;
            __DestructExceptionObject(&mut nodescr);
            ref___DestructExceptionObject(&mut nodescr);
            row();
        }

        // ═══ except.c: the filters ════════════════════════════════════════════════════
        //
        // Every one of these is a decision function over an EXCEPTION_RECORD, so they grade
        // cleanly on fabricated records — which is the whole reason the record's layout is
        // under an assertion wall. `is_cxx_exception` is the shared gate: code, parameter
        // COUNT, and the magic in slot 0 must all match, and the count is the one a
        // translation is most likely to drop.
        {
            // The `bool` is "does this record pass `is_cxx_exception`", and it gates ONE row.
            // A record that passes it sends `__CxxExceptionFilter` into `find_caught_type`,
            // which walks the thrown object's descriptor graph — and a fabricated record has
            // no graph to walk, so the C dereferences the NULL descriptor exactly as msvcrt
            // would. That is not a bug to report, it is a fixture this host cannot build: the
            // graph belongs to a LIVE thrown object, and the RaiseException sink longjmps out
            // of the throw, which is what makes the object dead by the time a filter sees it.
            // So the typed row runs only on the records that are refused before the walk, and
            // the walk itself is listed as ungraded in the epilogue rather than faked.
            let mut cases: Vec<(&str, ExceptionRecord, bool)> = Vec::new();
            cases.push(("cxx VC6", mk_cxx_record(0x11), true));
            let mut vc8 = mk_cxx_record(0x11);
            vc8.ExceptionInformation[0] = 0x19930522;
            cases.push(("cxx VC8", vc8, true));
            let mut too_new = mk_cxx_record(0x11);
            too_new.ExceptionInformation[0] = 0x19930523;
            cases.push(("magic just past VC8", too_new, false));
            let mut too_old = mk_cxx_record(0x11);
            too_old.ExceptionInformation[0] = 0x1993051F;
            cases.push(("magic just below VC6", too_old, false));
            let mut wrong_count = mk_cxx_record(0x11);
            wrong_count.NumberParameters = sys::CXX_EXCEPTION_PARAMS - 1;
            cases.push(("right magic, wrong parameter count", wrong_count, false));
            let mut av = mk_cxx_record(0x11);
            av.ExceptionCode = 0xC0000005;
            cases.push(("access violation", av, false));
            let mut nocode = mk_cxx_record(0);
            nocode.ExceptionInformation[1] = 0;
            nocode.ExceptionInformation[2] = 0;
            // Both slots zero: `__CxxExceptionFilter` swaps in the thread's saved record,
            // finds none, and returns CONTINUE_SEARCH before any walk.
            cases.push(("cxx with a null object and descriptor", nocode, false));

            for (what, rec, is_cxx) in cases.iter_mut() {
                let mut ep = ExceptionPointers {
                    ExceptionRecord: rec,
                    ContextRecord: core::ptr::null_mut(),
                };
                same(
                    &format!("__CxxDetectRethrow({what})"),
                    __CxxDetectRethrow(&mut ep),
                    ref___CxxDetectRethrow(&mut ep),
                );
                // catch(...) — a NULL type_info is EXECUTE_HANDLER whatever the record says,
                // and it is checked BEFORE the record is looked at.
                same(
                    &format!("__CxxExceptionFilter(catch(...), {what})"),
                    __CxxExceptionFilter(&mut ep, core::ptr::null(), 0, core::ptr::null_mut()),
                    ref___CxxExceptionFilter(&mut ep, core::ptr::null(), 0, core::ptr::null_mut()),
                );
                // A non-NULL type_info against a record the gate refuses: CONTINUE_SEARCH,
                // decided before the table walk this host cannot build a fixture for.
                if !*is_cxx {
                    let ti = mk_type_info(".?AVfoo@@");
                    same(
                        &format!("__CxxExceptionFilter(typed, {what})"),
                        __CxxExceptionFilter(
                            &mut ep,
                            &*ti as *const TypeInfo as *const c_void,
                            0,
                            core::ptr::null_mut(),
                        ),
                        ref___CxxExceptionFilter(
                            &mut ep,
                            &*ti as *const TypeInfo as *const c_void,
                            0,
                            core::ptr::null_mut(),
                        ),
                    );
                }
                // The two SEH filters. With every handler at SIG_DFL the mapping switch falls
                // through to CONTINUE_SEARCH; the codes that map to a signal are swept below.
                same(
                    &format!("_XcptFilter({what})"),
                    _XcptFilter(rec.ExceptionCode as c_int, &mut ep),
                    ref__XcptFilter(rec.ExceptionCode as c_int, &mut ep),
                );
                same(
                    &format!("__CppXcptFilter({what})"),
                    __CppXcptFilter(rec.ExceptionCode as c_int, &mut ep),
                    ref___CppXcptFilter(rec.ExceptionCode as c_int, &mut ep),
                );
            }

            // NULL pointers, which every one of the four guards against separately.
            same(
                "__CxxDetectRethrow(NULL)",
                __CxxDetectRethrow(core::ptr::null_mut()),
                ref___CxxDetectRethrow(core::ptr::null_mut()),
            );
            same(
                "__CxxExceptionFilter(NULL)",
                __CxxExceptionFilter(
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    0,
                    core::ptr::null_mut(),
                ),
                ref___CxxExceptionFilter(
                    core::ptr::null_mut(),
                    core::ptr::null(),
                    0,
                    core::ptr::null_mut(),
                ),
            );
            same(
                "_XcptFilter(NULL)",
                _XcptFilter(0, core::ptr::null_mut()),
                ref__XcptFilter(0, core::ptr::null_mut()),
            );
        }

        // The SEH-to-signal mapping, which is `msvcrt_exception_filter`'s whole body and is
        // reachable only through `_XcptFilter`. With SIG_DFL everywhere every code returns
        // CONTINUE_SEARCH; with a handler installed the FP codes each carry a distinct
        // `_FPE_*` subcode into it, and getting one of those wrong is invisible to any test
        // that only reads the return value. This is the row the derived `_FPE_*` block exists
        // for — glibc has no such constants at all.
        {
            let codes: &[u32] = &[
                0xC000008D, // FLT_DENORMAL_OPERAND
                0xC000008E, // FLT_DIVIDE_BY_ZERO
                0xC000008F, // FLT_INEXACT_RESULT
                0xC0000090, // FLT_INVALID_OPERATION
                0xC0000091, // FLT_OVERFLOW
                0xC0000092, // FLT_STACK_CHECK
                0xC0000093, // FLT_UNDERFLOW
                0xC0000094, // INT_DIVIDE_BY_ZERO
                0xC0000095, // INT_OVERFLOW
                0xC0000005, // ACCESS_VIOLATION
                0xC00000FD, // STACK_OVERFLOW
                0xC000001D, // ILLEGAL_INSTRUCTION
                0xC0000096, // PRIV_INSTRUCTION
                0xE06D7363, // a C++ throw, which this filter does not handle
            ];
            for &code in codes {
                for handler in [SIG_DFL, SIG_IGN, usize::MAX - 1] {
                    log_reset();
                    for sig in [sys::SIGFPE, sys::SIGSEGV, sys::SIGILL] {
                        let real = handler == usize::MAX - 1;
                        let hs = if real { handler_for(sig as c_int, true) } else { handler };
                        let hr = if real { handler_for(sig as c_int, false) } else { handler };
                        signal(sig as c_int, core::mem::transmute::<usize, SigHandler>(hs));
                        ref_signal(sig as c_int, core::mem::transmute::<usize, SigHandler>(hr));
                    }
                    let mut rec = mk_cxx_record(0x11);
                    rec.ExceptionCode = code;
                    let mut ep = ExceptionPointers {
                        ExceptionRecord: &mut rec,
                        ContextRecord: core::ptr::null_mut(),
                    };
                    let hn = match handler {
                        SIG_DFL => "SIG_DFL",
                        SIG_IGN => "SIG_IGN",
                        _ => "handler",
                    };
                    same(
                        &format!("_XcptFilter(0x{code:08X}, {hn})"),
                        _XcptFilter(code as c_int, &mut ep),
                        ref__XcptFilter(code as c_int, &mut ep),
                    );
                    same(
                        &format!("_XcptFilter(0x{code:08X}, {hn}) subcode"),
                        log_get(0),
                        log_get(1),
                    );
                }
            }
            for sig in [sys::SIGFPE, sys::SIGSEGV, sys::SIGILL] {
                signal(sig as c_int, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
                ref_signal(sig as c_int, core::mem::transmute::<usize, SigHandler>(SIG_DFL));
            }
        }

        // ═══ except.c: the constants-shaped exports ═══════════════════════════════════
        same(
            "__CxxQueryExceptionSize",
            __CxxQueryExceptionSize(),
            ref___CxxQueryExceptionSize(),
        );
        same(
            "__intrinsic_abnormal_termination",
            __intrinsic_abnormal_termination(),
            ref___intrinsic_abnormal_termination(),
        );
        same("__uncaught_exception", __uncaught_exception(), ref___uncaught_exception());
    }

    eprintln!("cpp.c + except.c: {} rows\n", unsafe { ROWS });
    eprintln!(
        "OUT OF REACH on this host, and carrying no rows rather than fake ones: \
         __CxxFrameHandler, find_catch_handler, dump_function_descr, _fpieee_flt, \
         __RTDynamicCast/__RTtypeid/__RTCastToVoid, and __CxxExceptionFilter's type-matching \
         walk. The first four need a real PE unwind table and a real stack; cppimpl.c's stubs \
         for them _Exit(94) rather than return a plausible value, so a row that reached one \
         would abort the suite instead of reporting a verdict about a dispatch that never \
         happened. The last needs a LIVE thrown object's descriptor graph, which the sink's \
         longjmp out of the throw is what makes unavailable. See the module docs."
    );
}

// ── helpers that need the externs above ──────────────────────────────────────────────────

unsafe extern "C" fn exit77_subj() {
    libc::_exit(77);
}
unsafe extern "C" fn exit77_ref() {
    libc::_exit(77);
}

/// The image base cppimpl.c anchors the RVA model on, obtained the way the code under test
/// obtains it rather than reconstructed.
unsafe fn ref_image_base() -> *mut c_void {
    let mut base: *mut c_void = core::ptr::null_mut();
    RtlPcToFileHeader(core::ptr::null_mut(), &mut base);
    base
}

extern "C" {
    fn RtlPcToFileHeader(pc: *mut c_void, base: *mut *mut c_void) -> *mut c_void;
}

/// A minimal C++ EXCEPTION_RECORD: the code, the VC6 magic in slot 0, the object in slot 1,
/// no descriptor, and the image base in slot 3. `NumberParameters` is CXX_EXCEPTION_PARAMS
/// because `is_cxx_exception` checks it — a record with the right magic and the wrong count
/// is not a C++ exception, which is its own row above.
unsafe fn mk_cxx_record(obj: usize) -> ExceptionRecord {
    ExceptionRecord {
        ExceptionCode: sys::CXX_EXCEPTION,
        ExceptionFlags: 0,
        ExceptionRecord: core::ptr::null_mut(),
        ExceptionAddress: core::ptr::null_mut(),
        NumberParameters: sys::CXX_EXCEPTION_PARAMS,
        ExceptionInformation: {
            let mut a = [0usize; 15];
            a[0] = 0x19930520;
            a[1] = obj;
            a[2] = 0;
            a[3] = ref_image_base() as usize;
            a
        },
    }
}

fn zero_cxx_frame() -> CxxFrameInfo {
    CxxFrameInfo {
        frame_info: FrameInfo { object: core::ptr::null_mut(), next: core::ptr::null_mut() },
        rec: core::ptr::null_mut(),
        context: core::ptr::null_mut(),
    }
}

/// Runs a body that THROWS and returns everything the sink recorded that is not an address.
///
/// The `setjmp` lives in cppimpl.c's `ref_run_throw`, not here: a `longjmp` back into a Rust
/// frame across an FFI boundary is undefined, and the buffer's own frame has to still be live
/// when the jump lands. So this hands over a callback and C is the landing pad.
static mut THROW_MSG: *const c_char = core::ptr::null();

unsafe extern "C" fn do_throw_subj(_ctx: *mut c_void) {
    throw_exception(THROW_MSG);
}
unsafe extern "C" fn do_throw_ref(_ctx: *mut c_void) {
    ref_throw_exception(THROW_MSG);
}

unsafe fn capture_throw(
    f: unsafe extern "C" fn(*mut c_void),
) -> (c_int, c_int, u32, u32, u32, usize) {
    let threw = ref_run_throw(f, core::ptr::null_mut());
    (
        // "did it come back at all" is part of the verdict: `_CxxThrowException` is
        // DECLSPEC_NORETURN, so a side that returned normally did not throw.
        threw,
        ref_raise_fired(),
        ref_raise_code(),
        ref_raise_flags(),
        ref_raise_nargs(),
        // Slot 0 is the frame magic, the one parameter of a throw whose VALUE is the same on
        // both sides. Slots 1-3 are the object, its descriptor and the image base: two
        // addresses that differ by construction, and a base both sides take from the same
        // body, so comparing them would grade the harness.
        ref_raise_arg(0),
    )
}
