//! The invalid-parameter handler machinery and `_invoke_watson`. Replaces
//! `sdk/lib/ucrt/misc/invalid_parameter.cpp` in full.
//!
//! This is the machinery every `_VALIDATE_*`/`_UCRT_VALIDATE_*` macro in the
//! rest of ucrt-rust bottoms out in (`_invalid_parameter_noinfo`,
//! `_invalid_parameter_internal` via [`crate::ptd_host::PtdHost`]), so a bug
//! here is invisible everywhere at once rather than local to one export.
//!
//! ## `_CRT_GLOBAL_STATE_ISOLATION` is OFF in this tree
//!
//! `sdk/lib/ucrt_reactos/inc/corecrt_internal_state_isolation.h` `#error`s if
//! that macro is ever defined, and fixes `state_index_count = 1`. The real
//! `dual_state_global<T>` is therefore a single `T`, not an array indexed by
//! a global-state index (the OS-vs-app split Microsoft ships never applies
//! here), and `__acrt_initialize_invalid_parameter_handler`'s C never builds
//! the second (`invalid_parameter_handler_continue`) array slot -- so neither
//! do we.
//!
//! ## The encoding
//!
//! The global handler is stored through `__crt_fast_encode_pointer`/
//! `__crt_fast_decode_pointer` (`sdk/lib/vcruntime/inc/internal_shared.h`):
//! both are `ptr ^ __security_cookie` (a plain XOR against the process's GS
//! cookie, NOT `EncodePointer`/`DecodePointer`, which are a different,
//! heap-manager-keyed Win32 API). Critically, `__acrt_initialize_invalid_parameter_handler`
//! stores its `encoded_null` argument **directly, with no further encoding**
//! -- the caller (`initialize_pointers`, still C) already encoded it before
//! the call. So the stored word is always the encoded bit pattern; only
//! `_set_invalid_parameter_handler` encodes on the way in and
//! `_get_invalid_parameter_handler`/`_invalid_parameter_internal` decode on
//! the way out.
//!
//! The thread-local handler (`__acrt_ptd::_thread_local_iph`, `ptd.rs`'s
//! `thread_local_iph`) is NEVER encoded -- it is a plain pointer, read and
//! written as one.
//!
//! ## Precedence (`_invalid_parameter_internal`)
//!
//! 1. The calling thread's thread-local handler, if one is installed --
//!    called and returns, the global handler and Watson are never consulted.
//! 2. Else the global handler, if one is installed -- likewise returns
//!    without reaching Watson.
//! 3. Else `_invoke_watson`, which does not return: the process ends.
//!
//! ## `_invoke_watson` per architecture
//!
//! * i386/amd64: fast-fail if `PF_FASTFAIL_AVAILABLE`; otherwise
//!   `__acrt_call_reportfault` (fake an exception, clear the process filter,
//!   call `UnhandledExceptionFilter`) and `TerminateProcess`. The fallback is
//!   LIVE here: only amd64's KiInit sets `PF_FASTFAIL_AVAILABLE`, so every i386
//!   `_invoke_watson` and `abort()` takes it.
//! * arm64: unconditional fast-fail, no feature check at all (the C's `#if`
//!   only builds the report-fault machinery for `_M_IX86`/`_M_X64`).
//!
//! `__acrt_call_reportfault` needs what the C takes from inline asm and the
//! `_ReturnAddress()`/`_AddressOfReturnAddress()` intrinsics: on i386 it is a
//! `global_asm!` entry stub that snapshots the registers, segments, flags and the
//! caller's Eip/Esp/Ebp at entry; on amd64, `RtlCaptureContext` unwound two
//! frames, which lands on the same caller Rip/Rsp the C writes. `CONTEXT` and
//! `EXCEPTION_RECORD` are opaque blocks of the MEASURED size with the few fields
//! written at measured offsets.
//!
//! ## Details that are silent if wrong
//!
//! * `_invalid_parameter_noinfo_noreturn` calls `_invalid_parameter` first
//!   (which may itself return, if a handler is installed that doesn't
//!   terminate) and THEN unconditionally calls `_invoke_watson` -- so
//!   `__declspec(noreturn)` holds even when a non-terminating handler runs.
//! * `_get_thread_local_invalid_parameter_handler` uses the NOEXIT ptd lookup
//!   and returns `NULL` if no ptd is available; `_set_thread_local_invalid_parameter_handler`
//!   uses the aborting lookup (matches the C: `__acrt_getptd()` vs
//!   `__acrt_getptd_noexit()`).
//! * `_invalid_parameter_internal` looks up its own ptd via
//!   `__acrt_getptd_noexit()` directly rather than through
//!   [`crate::ptd_host::PtdHost`]'s private cache -- `PtdHost` exposes only
//!   the ABORTING `raw_ptd()` accessor, and using that here would turn a rare,
//!   gracefully-handled `__acrt_getptd_noexit()` failure into an process abort
//!   that the real UCRT does not have. The cost is a possible extra
//!   FlsGetValue if the host had already cached a ptd from earlier in the
//!   same call -- a performance difference only, never a correctness one,
//!   since nothing before this call path caches a ptd on the host.

use core::ffi::{c_int, c_uint, c_void};
use core::ptr::null_mut;

use crate::locale_update::__acrt_getptd;
use crate::ptd::AcrtPtd;
use crate::ptd_host::PtdHost;

/// `_invalid_parameter_handler` (`corecrt.h`): `void (__cdecl *)(wchar_t
/// const*, wchar_t const*, wchar_t const*, unsigned int, uintptr_t)`.
pub type InvalidParameterHandler =
    unsafe extern "C" fn(*const u16, *const u16, *const u16, c_uint, usize);

extern "C" {
    fn _errno() -> *mut c_int;
    /// `sdk/lib/ucrt/internal/per_thread_data.cpp` (still C): the NOEXIT ptd
    /// lookup, used here in place of `PtdHost`'s private (aborting) cache --
    /// see the module doc.
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;

}

// Win32, transcribed from `sdk/include/psdk/winbase.h` / `winnt.h`. WINAPI, so
// `extern "system"` -- stdcall on i386, where a cdecl declaration would bind the
// wrong decorated symbol. Only the i386/amd64 arms reach these; arm64 fast-fails
// unconditionally, exactly as the C's `#if` gates them out there.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn IsProcessorFeaturePresent(feature: u32) -> i32;
    fn GetCurrentProcess() -> *mut c_void;
    fn TerminateProcess(process: *mut c_void, exit_code: u32) -> i32;
    fn IsDebuggerPresent() -> i32;
    fn SetUnhandledExceptionFilter(filter: *mut c_void) -> *mut c_void;
    fn UnhandledExceptionFilter(pointers: *mut ExceptionPointers) -> i32;
    #[cfg(target_arch = "x86_64")]
    fn RtlCaptureContext(context: *mut Context);
    #[cfg(target_arch = "x86_64")]
    fn RtlLookupFunctionEntry(pc: u64, image_base: *mut u64, history: *mut c_void) -> *mut c_void;
    #[cfg(target_arch = "x86_64")]
    fn RtlVirtualUnwind(
        handler_type: u32,
        image_base: u64,
        pc: u64,
        function_entry: *mut c_void,
        context: *mut Context,
        handler_data: *mut *mut c_void,
        establisher_frame: *mut u64,
        context_pointers: *mut c_void,
    ) -> *mut c_void;
}

// `__security_cookie` (`sdk/lib/rust/crt-startup-rust/src/gs.rs`): the GS
// cookie every `__crt_fast_encode_pointer`/`__crt_fast_decode_pointer` XORs
// against. Declared `extern` here rather than depended on, for the same
// reason `ucrt-rust` never depends on `msvcrt-crt-rust`: this crate reaches
// the symbol, not the crate that owns it.
extern "C" {
    static mut __security_cookie: usize;
}

/// `__crt_fast_encode_pointer` / `__crt_fast_decode_pointer`: both are the
/// SAME operation (XOR is self-inverse), kept as two names to match the two
/// call sites' intent.
#[inline]
unsafe fn encode(ptr: usize) -> usize {
    ptr ^ __security_cookie
}
#[inline]
unsafe fn decode(ptr: usize) -> usize {
    ptr ^ __security_cookie
}

#[inline]
unsafe fn encode_handler(h: Option<InvalidParameterHandler>) -> usize {
    encode(h.map_or(0, |f| f as usize))
}
#[inline]
unsafe fn decode_handler(encoded: usize) -> Option<InvalidParameterHandler> {
    let raw = decode(encoded);
    if raw == 0 {
        None
    } else {
        Some(core::mem::transmute::<usize, InvalidParameterHandler>(raw))
    }
}

/// `static __crt_state_management::dual_state_global<_invalid_parameter_handler>
/// __acrt_invalid_parameter_handler`, collapsed to its single slot (see the
/// module doc). Holds the ENCODED bit pattern, exactly as the C's `_value[0]`
/// does. Not atomic: the C does a plain, unsynchronized read/modify/write
/// here too (`dual_state_global::value()` returns a plain reference), so a
/// lock would not be a faithful port.
static mut ACRT_INVALID_PARAMETER_HANDLER: usize = 0;

/// `__acrt_initialize_invalid_parameter_handler`. Called once, at CRT startup,
/// by `sdk/lib/ucrt/internal/initialization.cpp` (still C) with an
/// ALREADY-ENCODED null pointer -- stored verbatim, not re-encoded, exactly as
/// the C's `iph[] = { reinterpret_cast<_invalid_parameter_handler>(encoded_null) }`.
#[no_mangle]
pub unsafe extern "C" fn __acrt_initialize_invalid_parameter_handler(encoded_null: *mut c_void) {
    ACRT_INVALID_PARAMETER_HANDLER = encoded_null as usize;
}

/// `_invalid_parameter_internal`. The signature (five args plus the ptd host
/// by pointer) is fixed by `crate::ptd_host`'s existing `extern "C"`
/// declaration, which every `_UCRT_VALIDATE_*` caller already links against;
/// this is that symbol's one definition.
#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter_internal(
    expression: *const u16,
    function_name: *const u16,
    file_name: *const u16,
    line_number: c_uint,
    reserved: usize,
    _ptd: *mut PtdHost,
) {
    // get_raw_ptd_noexit() -- see the module doc for why this is a fresh
    // lookup rather than routed through `_ptd`'s cache.
    let raw_ptd = __acrt_getptd_noexit();
    if !raw_ptd.is_null() && !(*raw_ptd).thread_local_iph.is_null() {
        let handler: InvalidParameterHandler =
            core::mem::transmute((*raw_ptd).thread_local_iph);
        handler(expression, function_name, file_name, line_number, reserved);
        return;
    }

    if let Some(handler) = decode_handler(ACRT_INVALID_PARAMETER_HANDLER) {
        handler(expression, function_name, file_name, line_number, reserved);
        return;
    }

    _invoke_watson(expression, function_name, file_name, line_number, reserved);
}

#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter(
    expression: *const u16,
    function_name: *const u16,
    file_name: *const u16,
    line_number: c_uint,
    reserved: usize,
) {
    // `__crt_cached_ptd_host ptd;` -- a fresh, default-constructed host, alive
    // for exactly this call (its `Drop` writes back errno/doserrno exactly as
    // the C++ destructor does, on scope exit below).
    let mut ptd = PtdHost::new(null_mut());
    _invalid_parameter_internal(
        expression,
        function_name,
        file_name,
        line_number,
        reserved,
        &mut ptd,
    );
}

#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter_noinfo() {
    _invalid_parameter(null_mut(), null_mut(), null_mut(), 0, 0);
}

/// `__declspec(noreturn)`: always reaches `_invoke_watson`, even if the
/// installed handler (global or thread-local) returns instead of
/// terminating -- matching the C exactly.
#[no_mangle]
pub unsafe extern "C" fn _invalid_parameter_noinfo_noreturn() -> ! {
    _invalid_parameter(null_mut(), null_mut(), null_mut(), 0, 0);
    _invoke_watson(null_mut(), null_mut(), null_mut(), 0, 0);
}

/// `FAST_FAIL_INVALID_ARG` (`sdk/include/xdk/rtlfuncs.h`).
const FAST_FAIL_INVALID_ARG: u32 = 5;
/// `PF_FASTFAIL_AVAILABLE` (`sdk/include/xdk/ketypes.h`).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const PF_FASTFAIL_AVAILABLE: u32 = 23;
/// `STATUS_INVALID_CRUNTIME_PARAMETER` (`sdk/include/psdk/ntstatus.h`).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const STATUS_INVALID_CRUNTIME_PARAMETER: u32 = 0xC000_0417;
/// `_CRT_DEBUGGER_INVALIDPARAMETER` (vcruntime internal_shared.h).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const CRT_DEBUGGER_INVALIDPARAMETER: c_int = 2;
/// `EXCEPTION_NONCONTINUABLE` (winnt.h).
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const EXCEPTION_NONCONTINUABLE: u32 = 0x1;

/// `__fastfail(Code)`: `int 0x29` with the code in `ecx` on i386/amd64
/// (`sdk/include/vcruntime/mingw32/intrin_x86.h`'s own fallback body: `"int
/// $0x29" : : "c"(Code)`), `brk #0xF003` with the code in `x0` on arm64
/// (the instruction itself is confirmed in-tree --
/// `dll/ntdll/dispatch/arm64/dispatch.S` -- but that use is exception
/// re-dispatch, not `__fastfail`'s own calling convention for the code
/// register, which is NOT independently confirmed in this tree; flagged in
/// the batch report). Diverges: the instruction terminates the process.
///
/// On x86 this is NOT declared `noreturn`: LLVM plants a `ud2` trap after a
/// noreturn asm block, which `scan_ud2` rightly refuses as an unexplained trap
/// in shipping code. `int 0x29` ends the process, so the caller's fall-through
/// (the report-fault path, as when fast-fail is unavailable) is never reached.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn fastfail(code: u32) {
    core::arch::asm!("int 0x29", in("ecx") code, options(nomem, nostack));
}
#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn fastfail(code: u32) -> ! {
    core::arch::asm!("brk #0xF003", in("x0") code as u64, options(noreturn));
}

/// `_invoke_watson`. `expression`/`function_name`/`file_name`/`line_number`/
/// `reserved` are `UNREFERENCED_PARAMETER`d by the C on every architecture
/// (they exist purely for a debugger/Watson report the simplified
/// `__acrt_call_reportfault` below does not produce), so they are accepted
/// and ignored here too.
#[no_mangle]
pub unsafe extern "C" fn _invoke_watson(
    _expression: *const u16,
    _function_name: *const u16,
    _file_name: *const u16,
    _line_number: c_uint,
    _reserved: usize,
) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if IsProcessorFeaturePresent(PF_FASTFAIL_AVAILABLE) != 0 {
            fastfail(FAST_FAIL_INVALID_ARG);
        }
        // Reached on i386 ReactOS: only amd64's KiInit sets
        // PF_FASTFAIL_AVAILABLE (ntoskrnl ke/amd64/kiinit).
        __acrt_call_reportfault(
            CRT_DEBUGGER_INVALIDPARAMETER,
            STATUS_INVALID_CRUNTIME_PARAMETER,
            EXCEPTION_NONCONTINUABLE,
        );
        // The C's `UNREACHABLE;` after this call: terminating the CURRENT process
        // does not return. Looping on the call itself says so without an empty
        // spin or an unchecked assumption -- were it ever to return, it retries.
        loop {
            TerminateProcess(GetCurrentProcess(), STATUS_INVALID_CRUNTIME_PARAMETER);
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // arm64 has no feature check at all in the C -- always fast-fail.
        fastfail(FAST_FAIL_INVALID_ARG);
    }
}

/// `CONTEXT` and `EXCEPTION_RECORD`, as opaque blocks of the measured size
/// with the few fields `__acrt_call_reportfault` writes at measured offsets
/// (probe TU with the build's flags, 2026-09-27: i386 CONTEXT 716 bytes,
/// align 4; amd64 1232, align 16; EXCEPTION_RECORD 80 / 152).
#[cfg(target_arch = "x86")]
#[repr(C, align(4))]
pub struct Context([u8; 716]);
#[cfg(target_arch = "x86_64")]
#[repr(C, align(16))]
pub struct Context([u8; 1232]);
#[cfg(target_arch = "x86")]
mod ctx {
    pub const CONTEXT_FLAGS: usize = 0;
    pub const CONTEXT_CONTROL: u32 = 0x0001_0001;
    // SegGs..SegDs, Edi..Eax, Ebp, Eip, SegCs, EFlags, Esp, SegSs: one dword
    // each from 140, in exactly the order the snapshot below is laid out.
    pub const REGS_AT: usize = 140;
}
#[cfg(target_arch = "x86_64")]
mod ctx {
    pub const RIP: usize = 248;
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[repr(C)]
pub struct ExceptionRecord {
    code: u32,
    flags: u32,
    record: *mut ExceptionRecord,
    address: *mut c_void,
    number_parameters: u32,
    information: [usize; 15],
}
#[cfg(target_arch = "x86")]
const _: () = assert!(core::mem::size_of::<ExceptionRecord>() == 80);
#[cfg(target_arch = "x86_64")]
const _: () = assert!(core::mem::size_of::<ExceptionRecord>() == 152);

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[repr(C)]
pub struct ExceptionPointers {
    record: *mut ExceptionRecord,
    context: *mut Context,
}

/// `_CRT_DEBUGGER_IGNORE`. `_CRT_DEBUGGER_HOOK` itself expands to nothing in
/// this build (sdk/lib/vcruntime/inc/internal_shared.h:92), so both of the C's
/// hook calls vanish; the code is only compared.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const CRT_DEBUGGER_IGNORE: c_int = -1;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
const EXCEPTION_CONTINUE_SEARCH: i32 = 0;

/// The common tail of `__acrt_call_reportfault`: raise the faked exception
/// through `UnhandledExceptionFilter` with the process filter cleared, so an
/// attached debugger or the system's handler sees it.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn report_fault(
    debugger_hook_code: c_int,
    record: &mut ExceptionRecord,
    context: &mut Context,
) {
    let was_debugger_present = IsDebuggerPresent();
    let mut pointers = ExceptionPointers { record, context };
    SetUnhandledExceptionFilter(null_mut());
    let handler_result = UnhandledExceptionFilter(&mut pointers);
    // `_CRT_DEBUGGER_HOOK(code)` when nothing handled it and no debugger was
    // already attached -- a no-op macro in this build.
    let _ = (handler_result == EXCEPTION_CONTINUE_SEARCH)
        && was_debugger_present == 0
        && debugger_hook_code != CRT_DEBUGGER_IGNORE;
}

// i386: `__acrt_call_reportfault` is an entry stub, because the C snapshots the
// general registers, segment registers and flags with inline asm AT ENTRY, and
// its caller's EIP/ESP/EBP from `_ReturnAddress()` / `_AddressOfReturnAddress()`
// -- neither of which stable Rust can express. The stub records exactly those
// sixteen dwords, in CONTEXT's own order from SegGs, and hands them over.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".globl ___acrt_call_reportfault",
    "___acrt_call_reportfault:",
    "push ebp",
    "mov ebp, esp",
    "sub esp, 64",
    // eax is saved first, then reused to move the segment registers.
    "mov [esp + 44], eax",
    "mov [esp + 40], ecx",
    "mov [esp + 36], edx",
    "mov [esp + 32], ebx",
    "mov [esp + 28], esi",
    "mov [esp + 24], edi",
    "xor eax, eax",
    "mov ax, gs",
    "mov [esp + 0], eax",
    "mov ax, fs",
    "mov [esp + 4], eax",
    "mov ax, es",
    "mov [esp + 8], eax",
    "mov ax, ds",
    "mov [esp + 12], eax",
    "mov ax, cs",
    "mov [esp + 52], eax",
    "mov ax, ss",
    "mov [esp + 64 - 4], eax",
    "pushfd",
    "pop eax",
    "mov [esp + 56], eax",
    // Ebp = the caller's (what this frame saved), Eip = the return address,
    // Esp = the address of the return address -- the C's three expressions.
    "mov eax, [ebp]",
    "mov [esp + 16], eax",
    "mov eax, [ebp + 4]",
    "mov [esp + 20], eax",
    "lea eax, [ebp + 4]",
    "mov [esp + 60 - 4], eax",
    "mov eax, esp",
    "push eax",
    "push dword ptr [ebp + 16]",
    "push dword ptr [ebp + 12]",
    "push dword ptr [ebp + 8]",
    "call _vibeos_ucrt_reportfault_x86",
    "add esp, 16",
    "mov esp, ebp",
    "pop ebp",
    "ret",
);

/// The body behind the i386 stub. `regs` is the stub's 16-dword snapshot, laid
/// out as CONTEXT from `SegGs` (offset 140) through `SegSs`.
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "C" fn vibeos_ucrt_reportfault_x86(
    debugger_hook_code: c_int,
    exception_code: u32,
    exception_flags: u32,
    regs: *const [u32; 16],
) {
    let mut record: ExceptionRecord = core::mem::zeroed();
    let mut context: Context = core::mem::zeroed();
    let base = context.0.as_mut_ptr();
    core::ptr::copy_nonoverlapping(regs as *const u8, base.add(ctx::REGS_AT), 64);
    (base.add(ctx::CONTEXT_FLAGS) as *mut u32).write(ctx::CONTEXT_CONTROL);
    record.code = exception_code;
    record.flags = exception_flags;
    // ExceptionAddress = _ReturnAddress(): the snapshot's Eip slot.
    record.address = (*regs)[11] as usize as *mut c_void;
    report_fault(debugger_hook_code, &mut record, &mut context);
}

/// amd64: `capture_current_context` (RtlCaptureContext, then one
/// RtlVirtualUnwind), and then the C overwrites Rip/Rsp with
/// `_ReturnAddress()` / `_AddressOfReturnAddress() + 8` -- the CALLER's resume
/// point. Stable Rust has no such intrinsic, so the capture is unwound one
/// frame further, which lands on exactly that Rip and Rsp.
#[cfg(target_arch = "x86_64")]
#[inline(never)]
unsafe fn capture_caller_context(context: &mut Context) {
    RtlCaptureContext(context);
    for _ in 0..2 {
        let rip = (context.0.as_ptr().add(ctx::RIP) as *const u64).read();
        let mut image_base: u64 = 0;
        let entry = RtlLookupFunctionEntry(rip, &mut image_base, null_mut());
        if entry.is_null() {
            break;
        }
        let mut handler_data: *mut c_void = null_mut();
        let mut establisher_frame: u64 = 0;
        RtlVirtualUnwind(
            0, // UNW_FLAG_NHANDLER
            image_base,
            rip,
            entry,
            context,
            &mut handler_data,
            &mut establisher_frame,
            null_mut(),
        );
    }
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn __acrt_call_reportfault(
    debugger_hook_code: c_int,
    exception_code: u32,
    exception_flags: u32,
) {
    let mut record: ExceptionRecord = core::mem::zeroed();
    let mut context: Context = core::mem::zeroed();
    capture_caller_context(&mut context);
    record.code = exception_code;
    record.flags = exception_flags;
    record.address = (context.0.as_ptr().add(ctx::RIP) as *const u64).read() as usize as *mut c_void;
    report_fault(debugger_hook_code, &mut record, &mut context);
}

#[cfg(target_arch = "x86")]
extern "C" {
    fn __acrt_call_reportfault(debugger_hook_code: c_int, exception_code: u32, exception_flags: u32);
}

// ── handler accessors ───────────────────────────────────────────────────────

#[no_mangle]
pub unsafe extern "C" fn _set_invalid_parameter_handler(
    new_handler: Option<InvalidParameterHandler>,
) -> Option<InvalidParameterHandler> {
    let old = decode_handler(ACRT_INVALID_PARAMETER_HANDLER);
    ACRT_INVALID_PARAMETER_HANDLER = encode_handler(new_handler);
    old
}

#[no_mangle]
pub unsafe extern "C" fn _get_invalid_parameter_handler() -> Option<InvalidParameterHandler> {
    decode_handler(ACRT_INVALID_PARAMETER_HANDLER)
}

#[no_mangle]
pub unsafe extern "C" fn _set_thread_local_invalid_parameter_handler(
    new_handler: Option<InvalidParameterHandler>,
) -> Option<InvalidParameterHandler> {
    // `__acrt_getptd()` -- the ABORTING lookup, matching the C exactly (this
    // one, unlike the getter below, does not use `_noexit`).
    let ptd = __acrt_getptd();
    let old_raw = (*ptd).thread_local_iph;
    let old: Option<InvalidParameterHandler> = if old_raw.is_null() {
        None
    } else {
        Some(core::mem::transmute(old_raw))
    };
    (*ptd).thread_local_iph = new_handler.map_or(null_mut(), |f| f as *mut c_void);
    old
}

#[no_mangle]
pub unsafe extern "C" fn _get_thread_local_invalid_parameter_handler(
) -> Option<InvalidParameterHandler> {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        return None;
    }
    let raw = (*ptd).thread_local_iph;
    if raw.is_null() {
        None
    } else {
        Some(core::mem::transmute(raw))
    }
}
