//! MIDL client-stub prototypes for the netapi32 `types_browser` type island.
//!
//! @generated — do NOT edit by hand. Regenerate with a functions-enabled run of the
//! `netapi32-browser` profile in tools/rustconv/gen_driver_types.sh (that profile sets
//! BLOCKLIST_FUNCS=1 so its `types_browser` island carries the wire STRUCTS only; this file is
//! the matching FUNCTION half, extracted from the same bindgen pass and ABI-normalised).
//!
//! These symbols are DEFINED by the WIDL-generated `*_c.c` client stubs, which cmake still
//! compiles as C (dll/win32/netapi32/CMakeLists.txt) — the cohab flip moves only the
//! hand-written `.c` to Rust, never the generated MIDL marshalling. So these are genuine
//! cross-language imports, not translation surface.
//!
//! ABI: bindgen emits these `__stdcall` prototypes as `extern "stdcall"` on i386 and
//! `extern "C"` on amd64/arm64. `extern "system"` IS exactly that per-arch pair, so one
//! arch-neutral block replaces three — verified: the generated signatures are byte-identical
//! across all three arches, only the ABI string differed.
//!
//! EXCLUDED (2): BROWSER_IDENTIFY_HANDLE_bind, BROWSER_IDENTIFY_HANDLE_unbind.
//! These are declared in the same headers but DEFINED by the translated modules (they are the
//! client's binding-handle callbacks), so an extern decl here would collide with the definition.
#![allow(non_snake_case, non_camel_case_types)]

use crate::types_browser::*;

extern "system" {
    pub fn I_BrowserrServerEnum(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        Transport: LPWSTR,
        ClientName: LPWSTR,
        EnumStruct: LPSERVER_ENUM_STRUCT,
        PreferedMaximumLength: DWORD,
        TotalEntries: LPDWORD,
        ServerType: DWORD,
        Domain: LPWSTR,
        ResumeHandle: LPDWORD,
    ) -> DWORD;
    pub fn BrowserOpnum1NotUsedOnWire() -> DWORD;
    pub fn I_BrowserrQueryOtherDomains(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        EnumStruct: LPSERVER_ENUM_STRUCT,
        TotalEntries: LPDWORD,
    ) -> DWORD;
    pub fn I_BrowserrResetNetlogonState(ServerName: BROWSER_IDENTIFY_HANDLE) -> DWORD;
    pub fn I_BrowserrDebugTrace(ServerName: BROWSER_IDENTIFY_HANDLE, String: LPSTR) -> DWORD;
    pub fn I_BrowserrQueryStatistics(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        Statistics: *mut LPBROWSER_STATISTICS,
    ) -> DWORD;
    pub fn I_BrowserrResetStatistics(ServerName: BROWSER_IDENTIFY_HANDLE) -> DWORD;
    pub fn NetrBrowserStatisticsClear(ServerName: BROWSER_IDENTIFY_HANDLE) -> DWORD;
    pub fn NetrBrowserStatisticsGet(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        Level: DWORD,
        StatisticsStruct: LPBROWSER_STATISTICS_STRUCT,
    ) -> DWORD;
    pub fn I_BrowserrSetNetlogonState(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        DomainName: LPWSTR,
        EmulatedComputerName: LPWSTR,
        Role: DWORD,
    ) -> DWORD;
    pub fn I_BrowserrQueryEmulatedDomains(
        ServerName: BROWSER_IDENTIFY_HANDLE,
        EmulatedDomains: PBROWSER_EMULATED_DOMAIN_CONTAINER,
    ) -> DWORD;
    pub fn BrowserOpnum11NotUsedOnWire() -> DWORD;
}
