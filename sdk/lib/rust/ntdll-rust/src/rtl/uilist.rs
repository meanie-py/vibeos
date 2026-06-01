//! Rust translation of `dll/ntdll/rtl/uilist.c`.
//!
//! `RtlConvertUiListToApiList` — RTL UI→API network-computers list helper for
//! netapi32.dll. `@unimplemented`: the C body's `DPRINT1`/`UNIMPLEMENTED` are
//! NDEBUG-elided in the release build; the actual behavior is to set `ApiList`
//! to an empty UNICODE_STRING and return `STATUS_SUCCESS` (the C comment notes
//! returning success + empty behaves better than a failure code).

use core::ptr::{addr_of_mut, write_volatile};
use ntdll_sys::nt::{BOOLEAN, NTSTATUS, PUNICODE_STRING, STATUS_SUCCESS};

/// C: `NTSTATUS NTAPI RtlConvertUiListToApiList(PUNICODE_STRING UiList,
/// PUNICODE_STRING ApiList, BOOLEAN SpaceAsSeparator)`. i386 NTAPI=stdcall →
/// `_RtlConvertUiListToApiList@12` (matches `def/ntdll.spec`).
#[no_mangle]
pub unsafe extern "system" fn RtlConvertUiListToApiList(
    _ui_list: PUNICODE_STRING,
    api_list: PUNICODE_STRING,
    _space_as_separator: BOOLEAN,
) -> NTSTATUS {
    // RtlInitEmptyUnicodeString(ApiList, NULL, 0) — inlined: it is a
    // FORCEINLINE with no exported symbol, so the bindgen extern would not
    // link. write_volatile keeps the output-pointer field stores from being
    // elided under LLVM captures(none).
    write_volatile(addr_of_mut!((*api_list).Length), 0u16);
    write_volatile(addr_of_mut!((*api_list).MaximumLength), 0u16);
    write_volatile(addr_of_mut!((*api_list).Buffer), core::ptr::null_mut());
    STATUS_SUCCESS
}
