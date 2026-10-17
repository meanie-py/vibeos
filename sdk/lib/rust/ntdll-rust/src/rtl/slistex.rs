//! `RtlInterlockedPushListSListEx` on i386 (Windows 8+).
//!
//! The Ex form is the STDCALL spelling of the FASTCALL `RtlInterlockedPushListSList`
//! (`@RtlInterlockedPushListSList@16`, Rust in sdk/lib/rust/slist): same arguments, same
//! semantics — splice the caller's pre-linked chain `List..ListEnd` (Count entries) onto the
//! head in one compare-exchange, point `ListEnd->Next` at the previous first entry, return
//! that previous first entry (NULL for an empty list), Depth += Count, Sequence += 1. Wine
//! implements it the same way round (dlls/ntdll/sync.c: the FASTCALL one calls the Ex one).
//!
//! On amd64/arm64 both names already share one calling convention, and ntdll.spec exports
//! the Ex name straight at the RtlInterlockedPushListSList body; only i386 needs a body.

#![cfg(target_arch = "x86")]

extern "fastcall" {
    fn RtlInterlockedPushListSList(ListHead: *mut u8, List: *mut u8, ListEnd: *mut u8, Count: u32) -> *mut u8;
}

#[no_mangle]
pub unsafe extern "stdcall" fn RtlInterlockedPushListSListEx(
    ListHead: *mut u8,
    List: *mut u8,
    ListEnd: *mut u8,
    Count: u32,
) -> *mut u8 {
    RtlInterlockedPushListSList(ListHead, List, ListEnd, Count)
}
