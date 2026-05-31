//! etw/ — main ntdll.dll Event Tracing (ETW) user-mode API.
//! Mirrors the C layout `dll/ntdll/etw/` (the `etwtrace` static lib).
//! These land in ntdll.dll proper (via libntdll_rust.a).

pub mod trace;
