//! Rust wrappers for the cross-subsystem `inline::common` helpers (probes,
//! splay-link, list ops, lookaside, exception conversion, RTL string copy,
//! interlocked add/dec/cmpxchg, FsRtl region, init helpers, etc.) that
//! previously lived in `ntoskrnl-rust::shim_exports`.
//!
//! Same augment+isolated-rlib pattern as `cm-inlines` / `ke-inlines` /
//! `ob-inlines` / `ps-inlines` / `se-inlines`: pull the wrappers into a
//! sibling rlib so `lto = false` + crate-boundary stops LLVM
//! captures(none) cross-CGU contagion.
//!
//! Each wrapper is a `#[no_mangle] #[inline(never)]` extern entry that
//! launders pointer arguments through `opaque_ptr` and delegates to the
//! corresponding `ntkrnl_sys::inline::common::*` body. Behaviour is
//! identical to the previous `shim_exports` placement.
//!
//! 34 functions total — every wrapper that delegated to
//! `ntkrnl_sys::inline::common::*` in `shim_exports.rs`.
//!
//! Calling-convention notes:
//! * All wrappers use `extern "C"` except `PsGetCurrentThread`, which uses
//!   `extern "system"` because callers expect the NTAPI = stdcall decoration
//!   on i386 (`_PsGetCurrentThread@0`). Mismatching the convention within a
//!   single LLVM CGU is UB and triggers the cdecl-vs-stdcall noreturn
//!   cascade documented in `feedback_extern_c_abi.md`.

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

// ----------------------------------------------------------------------------
// SE buffer-check helpers (NTAPI)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DefaultQueryInfoBufferCheck(
    class: ULONG,
    class_list: *const _INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    flags: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    return_length: *mut ULONG,
    return_length_ptr: *mut ULONG_PTR,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let buffer = opaque_ptr(buffer);
    let return_length = opaque_ptr(return_length);
    let return_length_ptr = opaque_ptr(return_length_ptr);
    ntkrnl_sys::inline::common::DefaultQueryInfoBufferCheck(
        class,
        class_list,
        class_list_entries,
        flags,
        buffer,
        buffer_length,
        return_length,
        return_length_ptr as *mut usize,
        previous_mode,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn DefaultSetInfoBufferCheck(
    class: ULONG,
    class_list: *const _INFORMATION_CLASS_INFO,
    class_list_entries: ULONG,
    buffer: PVOID,
    buffer_length: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let buffer = opaque_ptr(buffer);
    ntkrnl_sys::inline::common::DefaultSetInfoBufferCheck(
        class,
        class_list,
        class_list_entries,
        buffer,
        buffer_length,
        previous_mode,
    )
}

// ----------------------------------------------------------------------------
// Probe* family (NTAPI; pre-Result, exception-style probes)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadLargeInteger(
    addr: PLARGE_INTEGER,
) -> _LARGE_INTEGER {
    let addr = opaque_ptr(addr);
    ntkrnl_sys::inline::common::ProbeForReadLargeInteger(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadUchar(addr: *const u8) -> u8 {
    ntkrnl_sys::inline::common::ProbeForReadUchar(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForReadUshort(addr: *const u16) -> u16 {
    ntkrnl_sys::inline::common::ProbeForReadUshort(addr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteHandle(ptr: *mut HANDLE) {
    let ptr = opaque_ptr(ptr);
    ntkrnl_sys::inline::common::ProbeForWriteHandle(ptr)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWritePointer(address: *mut PVOID) {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::common::ProbeForWritePointer(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn ProbeForWriteUlong(address: *mut ULONG) {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::common::ProbeForWriteUlong(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteBoolean(address: *mut BOOLEAN) {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::common::ProbeForWriteBoolean(address)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeForWriteLangId(address: *mut u16) {
    let address = opaque_ptr(address);
    ntkrnl_sys::inline::common::ProbeForWriteLangId(address)
}

// ----------------------------------------------------------------------------
// Capture / release captured unicode string (NTAPI)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ProbeAndCaptureUnicodeString(
    dest: PUNICODE_STRING,
    mode: KPROCESSOR_MODE,
    src: *const _UNICODE_STRING,
) -> NTSTATUS {
    let dest = opaque_ptr(dest);
    ntkrnl_sys::inline::common::ProbeAndCaptureUnicodeString(dest, mode, src)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ReleaseCapturedUnicodeString(
    str: PUNICODE_STRING,
    mode: KPROCESSOR_MODE,
) {
    let str = opaque_ptr(str);
    ntkrnl_sys::inline::common::ReleaseCapturedUnicodeString(str, mode)
}

// ----------------------------------------------------------------------------
// Object-attributes / resource init helpers (NTAPI)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: PUNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    let p = opaque_ptr(p);
    let n = opaque_ptr(n);
    let r = opaque_ptr(r);
    let s = opaque_ptr(s);
    ntkrnl_sys::inline::common::InitializeObjectAttributes(p, n, a, r, s)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExInitializeResource(resource: PERESOURCE) {
    ntkrnl_sys::inline::common::ExInitializeResource(resource);
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PspClearCrossThreadFlag(thread: PETHREAD, flag: ULONG) {
    let thread = opaque_ptr(thread);
    ntkrnl_sys::inline::common::PspClearCrossThreadFlag(thread, flag)
}

// ----------------------------------------------------------------------------
// FsRtl region / CcBugCheck (NTAPI / cdecl)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FsRtlEnterFileSystem() {
    ntkrnl_sys::inline::common::FsRtlEnterFileSystem()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn FsRtlExitFileSystem() {
    ntkrnl_sys::inline::common::FsRtlExitFileSystem()
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn CcBugCheck(a: ULONG, b: ULONG, c: ULONG) {
    ntkrnl_sys::inline::common::CcBugCheck(a, b, c)
}

// ----------------------------------------------------------------------------
// RTL splay-link helpers (NTAPI)
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitializeSplayLinks(links: PRTL_SPLAY_LINKS) {
    let links = opaque_ptr(links);
    ntkrnl_sys::inline::common::RtlInitializeSplayLinks(links)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInsertAsLeftChild(
    parent: PRTL_SPLAY_LINKS,
    child: PRTL_SPLAY_LINKS,
) {
    let parent = opaque_ptr(parent);
    let child = opaque_ptr(child);
    ntkrnl_sys::inline::common::RtlInsertAsLeftChild(parent, child)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInsertAsRightChild(
    parent: PRTL_SPLAY_LINKS,
    child: PRTL_SPLAY_LINKS,
) {
    let parent = opaque_ptr(parent);
    let child = opaque_ptr(child);
    ntkrnl_sys::inline::common::RtlInsertAsRightChild(parent, child)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlIsLeftChild(links: PRTL_SPLAY_LINKS) -> BOOLEAN {
    let links = opaque_ptr(links);
    ntkrnl_sys::inline::common::RtlIsLeftChild(links)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlParent(links: PRTL_SPLAY_LINKS) -> PRTL_SPLAY_LINKS {
    let links = opaque_ptr(links);
    ntkrnl_sys::inline::common::RtlParent(links)
}

// ----------------------------------------------------------------------------
// Misc: int-safe add, exception-record conversion, list-ops, ANSI/UNICODE
// init, RtlStringCb, lookaside-list pop, interlocked
// ----------------------------------------------------------------------------

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlLongLongAdd(
    augend: i64,
    addend: i64,
    result: *mut i64,
) -> NTSTATUS {
    let result = opaque_ptr(result);
    ntkrnl_sys::inline::common::RtlLongLongAdd(augend, addend, result)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ExceptionRecord32To64(
    ex32: *const _EXCEPTION_RECORD32,
    ex64: *mut _EXCEPTION_RECORD64,
) {
    let ex64 = opaque_ptr(ex64);
    ntkrnl_sys::inline::common::ExceptionRecord32To64(ex32, ex64)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitEmptyAnsiString(
    ansi_string: PANSI_STRING,
    buffer: *mut i8,
    maximum_length: u16,
) {
    let ansi_string = opaque_ptr(ansi_string);
    let buffer = opaque_ptr(buffer);
    ntkrnl_sys::inline::common::RtlInitEmptyAnsiString(ansi_string, buffer, maximum_length)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlInitEmptyUnicodeString(
    unicode_string: PUNICODE_STRING,
    buffer: *mut u16,
    buffer_size: u16,
) {
    let buffer = opaque_ptr(buffer);
    ntkrnl_sys::inline::common::RtlInitEmptyUnicodeString(
        unicode_string,
        buffer,
        buffer_size,
    )
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn InsertTailList(
    list_head: PLIST_ENTRY,
    entry: PLIST_ENTRY,
) {
    let list_head = opaque_ptr(list_head);
    let entry = opaque_ptr(entry);
    ntkrnl_sys::inline::common::InsertTailList(list_head, entry)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn PopEntryList(
    list_head: *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY,
) -> *mut ntkrnl_sys::ddk::_SINGLE_LIST_ENTRY {
    let list_head = opaque_ptr(list_head);
    ntkrnl_sys::inline::common::PopEntryList(list_head)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn RtlStringCbCopyA(
    psz_dest: *mut u8,
    cb_dest: usize,
    psz_src: *const u8,
) -> NTSTATUS {
    let psz_dest = opaque_ptr(psz_dest);
    ntkrnl_sys::inline::common::RtlStringCbCopyA(psz_dest, cb_dest, psz_src)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedExchangeAdd64(
    addend: *mut i64,
    value: i64,
) -> i64 {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::common::InterlockedExchangeAdd64(addend, value)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedDecrementUL(addend: *mut ULONG) -> ULONG {
    let addend = opaque_ptr(addend);
    ntkrnl_sys::inline::common::InterlockedDecrementUL(addend)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn InterlockedCompareExchangePointer(
    destination: *mut PVOID,
    exchange: PVOID,
    comparand: PVOID,
) -> PVOID {
    let destination = opaque_ptr(destination);
    let exchange = opaque_ptr(exchange);
    let comparand = opaque_ptr(comparand);
    ntkrnl_sys::inline::common::InterlockedCompareExchangePointer(
        destination, exchange, comparand,
    )
}

// ----------------------------------------------------------------------------
// PsGetCurrentThread — special: NTAPI = stdcall on i386
// ----------------------------------------------------------------------------
//
// Must be `extern "system"` to match the calling convention used by all
// `ntkrnl-sys` extern declarations of `PsGetCurrentThread`. Mismatching the
// convention within a single LLVM CGU is UB and triggers the cdecl-vs-stdcall
// noreturn cascade documented in `feedback_extern_c_abi.md`.

#[no_mangle]
#[inline(never)]
pub unsafe extern "system" fn PsGetCurrentThread() -> PETHREAD {
    ntkrnl_sys::inline::common::PsGetCurrentThread()
}

// On i386, alias the cdecl name `_PsGetCurrentThread` to the stdcall name
// `_PsGetCurrentThread@0` (which the wrapper above produces). 0-argument
// functions are ABI-identical between cdecl and stdcall on i386 (both end in
// `ret`), so this is safe. Required because some Rust callers in
// ntoskrnl-rust still declare PsGetCurrentThread with the cdecl convention.
//
// This used to live in `ntoskrnl-rust::shim_exports` alongside other
// stdcall→cdecl aliases (KeEnterCriticalRegion, IoGetCurrentProcess, …).
// It moved here when the wrapper itself moved here so the alias source and
// target sit in the same translation unit.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".intel_syntax noprefix",
    ".globl _PsGetCurrentThread",
    ".set _PsGetCurrentThread, _PsGetCurrentThread@0",
);
