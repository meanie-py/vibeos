//! netcfgx-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
//!
//! netcfgx is a **C-style COM** server: `precomp.h` defines `COBJMACROS`, so every call
//! through an interface is the untyped macro `IFACE_Method(p,...)` → `(p)->lpVtbl->Method(p,...)`.
//! Those stand-ins live in the "COM dispatch" section below and were GENERATED from the
//! island's own `IFACEVtbl` structs, so an argument list here cannot drift from the vtable
//! the translation actually dispatches through.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// `NDEBUG` silences DPRINT only. DPRINT1 sits OUTSIDE that guard in
// sdk/include/reactos/debug.h, so it is live whenever DBG is set — and our media is
// built -DCMAKE_BUILD_TYPE=Debug, i.e. -DDBG=1. Both stay MACROS, never out-of-line
// fns (field manual §9.65: a no-op fn is an optimization barrier). The C passes
// __RELFILE__/__LINE__ as two extra varargs; file!()/line!() fold into the literal.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
// ntdll exports DbgPrint; every usermode DLL links it.
extern "C" {
    pub fn DbgPrint(Format: *const c_char, ...) -> u32;
}
#[macro_export]
macro_rules! DPRINT1 {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe {
            $crate::prelude::DbgPrint(
                concat!("(", file!(), ":", line!(), ") ", $fmt, "\0").as_ptr() as *const _
                $(, $arg)*,
            )
        }
    };
}
pub use crate::{DPRINT, DPRINT1};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= type aliases the island does not carry =====================================
// Each is a header typedef/#define whose right-hand side bindgen did emit; alias rather than
// re-declare, so there is exactly one Rust type identity per C type.
pub type REFIID = *const IID;
pub type REFCLSID = *const CLSID;
pub type REFGUID = *const GUID;
pub type UUID = GUID;                                   // rpcdce.h: typedef GUID UUID
pub type RPC_WSTR = *mut c_ushort;                      // rpcdce.h
pub type LPCLASSFACTORY = *mut IClassFactory;
// iptypes.h emits the tag struct and the P-form, but not the plain typedef.
pub type IP_ADAPTER_INFO = _IP_ADAPTER_INFO;
// iptypes.h:69 `typedef IP_PER_ADAPTER_INFO_W2KSP1 IP_PER_ADAPTER_INFO`.
pub type IP_PER_ADAPTER_INFO = IP_PER_ADAPTER_INFO_W2KSP1;

// UNICODE aliases: netcfgx builds -DUNICODE, so the SDK's `#define FOO FOOW` picks the W entry
// point. Those are preprocessor renames, invisible to a translator reading the .c.
pub type PROPSHEETPAGE = PROPSHEETPAGEW;
pub type LPPROPSHEETPAGE = LPPROPSHEETPAGEW;
pub type PROPSHEETHEADER = PROPSHEETHEADERW;
pub type LV_COLUMN = LVCOLUMNW;
pub type LVCOLUMN = LVCOLUMNW;

// ================= enum member aliases ========================================================
// bindgen prefixes C enum members with their enum's tag, so the bare C spelling is absent.
// Alias, never re-declare: a literal here could drift from the island's value silently.
pub const NCRP_QUERY_PROPERTY_UI: NCRP_FLAGS = NCRP_FLAGS_NCRP_QUERY_PROPERTY_UI;

// ================= constants bindgen cannot emit ==============================================
// Every one of these is a CAST expression in the header (`((HRESULT)0x80004005L)`), not an
// integer literal, so bindgen's macro evaluator skips it. Transcribed with the cast reproduced.
// winerror.h / winnt.h line references are the definition sites.
pub const S_OK: HRESULT = 0;
pub const S_FALSE: HRESULT = 1;
pub const NOERROR: HRESULT = 0;
pub const E_NOTIMPL: HRESULT = 0x8000_4001u32 as HRESULT;
pub const E_NOINTERFACE: HRESULT = 0x8000_4002u32 as HRESULT;
pub const E_POINTER: HRESULT = 0x8000_4003u32 as HRESULT;
pub const E_FAIL: HRESULT = 0x8000_4005u32 as HRESULT;
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;
pub const E_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as HRESULT;
pub const E_INVALIDARG: HRESULT = 0x8007_0057u32 as HRESULT;
pub const CLASS_E_CLASSNOTAVAILABLE: HRESULT = 0x8004_0111u32 as HRESULT;
// olectl.h: SELFREG_E_CLASS = SELFREG_E_FIRST + 1, and
// SELFREG_E_FIRST = MAKE_SCODE(SEVERITY_ERROR, FACILITY_ITF, 0x0200) = 0x80040200.
pub const SELFREG_E_CLASS: HRESULT = 0x8004_0201u32 as HRESULT;
pub const RPC_S_OK: RPC_STATUS = 0; // winerror.h: ERROR_SUCCESS

// INVALID_HANDLE_VALUE is shared — HANDLE really is `*mut c_void` in every island, and the
// shared copy is written through isize so the value is all-ones at the target's pointer width
// rather than a zero-extended 0xFFFFFFFF on 64-bit.
pub use win32macros_rust::INVALID_HANDLE_VALUE;
// winreg.h: ((HKEY)(LONG_PTR)(LONG)0x80000002). The sign-extend through LONG matters on
// 64-bit: 0x80000002 as i32 as isize gives 0xFFFFFFFF80000002, which is what the C produces.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub use win32macros_rust::UNICODE_NULL; // ntdef.h `((WCHAR)0)`
// wtypes.idl enum tagCLSCTX. Not in the island: objbase.h is reached with COM_NO_WINDOWS_H,
// which keeps the enum out of the parse the profile allowlists.
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;
// winuser.h: DWLP_USER = DWLP_MSGRESULT + sizeof(LRESULT) + sizeof(DLGPROC). POINTER-WIDTH
// DEPENDENT (8 on i386, 16 on amd64/arm64) — computed, never a literal, or the 64-bit builds
// read the wrong dialog word. Shared, still computed.
pub use win32macros_rust::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER};

// DEFINE_GUID statics. netcfgx links `uuid` (the CMakeLists' target_link_libraries), which is
// where the C picks them up too — declared, never defined here.
extern "C" {
    pub static IID_IUnknown: IID;
    pub static IID_IClassFactory: IID;
}

// ================= function-like MACROS =======================================================
// These are macros in C and stay macros here. Two reasons, both load-bearing: an out-of-line fn
// is an optimization barrier that can uncloak a latent ABI mismatch into `ud2` (field manual
// §9.65), and — the reason that actually bites — the C macros are UNTYPED, so one call site
// passes a WPARAM and the next an LPARAM or a DWORD. A concrete parameter type would force a
// cast at every use, i.e. an edit to paid translation output.
//
// Each body is transcribed from the header named beside it, cast for cast.

// The word/resource family lives in `win32macros-rust`, ported once from the headers and
// known-answer tested there. Re-exported here under the same names, so no call site changed.
// (MAKEINTRESOURCEW's WORD truncation is load-bearing and the shared port keeps it.)
pub use win32macros_rust::{HIWORD, LOWORD, MAKEINTRESOURCEW};
// winnt.h FIELD_OFFSET(Type, Field) = ((LONG)(LONG_PTR)&(((Type*)0)->Field))
#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:ident $(,)?) => { (core::mem::offset_of!($t, $f) as LONG) }; }
// winbase.h ZeroMemory→RtlZeroMemory, CopyMemory→RtlCopyMemory. Never a Rust byte loop: LLVM
// folds any such loop back into a call to memset/memcpy. The zeroing half is shared; CopyMemory
// below stays here because it reaches this crate's `memcpy` declaration.
pub use win32macros_rust::ZeroMemory;
#[macro_export]
macro_rules! CopyMemory {
    ($d:expr, $s:expr, $l:expr $(,)?) => {
        $crate::prelude::memcpy($d as *mut c_void, $s as *const c_void, $l as usize)
    };
}
// guiddef.h: (!memcmp(rguid1, rguid2, sizeof(GUID))). IsEqualIID/IsEqualCLSID are the same
// macro under another name.
#[macro_export]
macro_rules! IsEqualGUID {
    // `as *const _ as *const c_void`, in two steps: call sites pass BOTH a raw `*const GUID`
    // and a `&GUID` (`&IID_IUnknown`), and a reference cannot go straight to *const c_void.
    ($a:expr, $b:expr $(,)?) => {
        $crate::prelude::memcmp(
            $a as *const GUID as *const c_void,
            $b as *const GUID as *const c_void,
            core::mem::size_of::<GUID>(),
        ) == 0
    };
}
#[macro_export]
macro_rules! IsEqualIID { ($a:expr, $b:expr $(,)?) => { $crate::IsEqualGUID!($a, $b) }; }
#[macro_export]
macro_rules! IsEqualCLSID { ($a:expr, $b:expr $(,)?) => { $crate::IsEqualGUID!($a, $b) }; }

// commctrl.h IP-address control. MAKEIPADDRESS packs four octets big-endian into an LPARAM;
// the *_IPADDRESS accessors unpack it. MAKEIPRANGE packs a low/high pair into the low word.
#[macro_export]
macro_rules! MAKEIPADDRESS {
    ($b1:expr, $b2:expr, $b3:expr, $b4:expr $(,)?) => {
        ((((($b1 as DWORD) << 24)
            + (($b2 as DWORD) << 16)
            + (($b3 as DWORD) << 8)
            + ($b4 as DWORD)) as LPARAM)) as u32
    };
}
#[macro_export]
macro_rules! MAKEIPRANGE {
    ($low:expr, $high:expr $(,)?) => {
        (((($high as u8) as WORD) << 8) + (($low as u8) as WORD)) as LPARAM
    };
}
#[macro_export]
macro_rules! FIRST_IPADDRESS  { ($x:expr $(,)?) => { (($x >> 24) & 0xff) }; }
#[macro_export]
macro_rules! SECOND_IPADDRESS { ($x:expr $(,)?) => { (($x >> 16) & 0xff) }; }
#[macro_export]
macro_rules! THIRD_IPADDRESS  { ($x:expr $(,)?) => { (($x >> 8) & 0xff) }; }
#[macro_export]
macro_rules! FOURTH_IPADDRESS { ($x:expr $(,)?) => { ($x & 0xff) }; }

// prsht.h PropSheet_Changed(d,w) = SendMessage(d, PSM_CHANGED, (WPARAM)w, 0)
#[macro_export]
macro_rules! PropSheet_Changed {
    ($d:expr, $w:expr $(,)?) => { $crate::prelude::SendMessageW($d, (PSM_CHANGED) as u32, $w as WPARAM, 0) };
}

// windowsx.h control wrappers. SNDMSG is SendMessage, i.e. SendMessageW under -DUNICODE.
#[macro_export]
macro_rules! ComboBox_AddString {
    ($h:expr, $s:expr $(,)?) => { $crate::prelude::SendMessageW($h, CB_ADDSTRING as UINT, 0, $s as LPARAM) as c_int };
}
#[macro_export]
macro_rules! ComboBox_GetCurSel {
    ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, CB_GETCURSEL as UINT, 0, 0) as c_int };
}
#[macro_export]
macro_rules! ComboBox_SetCurSel {
    ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, CB_SETCURSEL as UINT, $i as WPARAM, 0) as c_int };
}
#[macro_export]
macro_rules! ComboBox_ResetContent {
    ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, CB_RESETCONTENT as UINT, 0, 0) as c_int };
}
#[macro_export]
macro_rules! ListBox_AddString {
    ($h:expr, $s:expr $(,)?) => { $crate::prelude::SendMessageW($h, LB_ADDSTRING as UINT, 0, $s as LPARAM) as c_int };
}
#[macro_export]
macro_rules! ListBox_GetCurSel {
    ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LB_GETCURSEL as UINT, 0, 0) as c_int };
}
#[macro_export]
macro_rules! ListBox_SetCurSel {
    ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LB_SETCURSEL as UINT, $i as WPARAM, 0) as c_int };
}
// NB: LB_GETITEMDATA comes back as an LRESULT, not an int — it carries a pointer-sized
// payload and truncating it to c_int would lose the upper half on 64-bit.
#[macro_export]
macro_rules! ListBox_GetItemData {
    ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LB_GETITEMDATA as UINT, $i as WPARAM, 0) as LRESULT };
}
#[macro_export]
macro_rules! ListBox_SetItemData {
    ($h:expr, $i:expr, $d:expr $(,)?) => {
        $crate::prelude::SendMessageW($h, LB_SETITEMDATA as UINT, $i as WPARAM, $d as LPARAM) as c_int
    };
}
#[macro_export]
macro_rules! Button_SetCheck {
    ($h:expr, $c:expr $(,)?) => {{ $crate::prelude::SendMessageW($h, BM_SETCHECK as UINT, $c as WPARAM, 0); }};
}
#[macro_export]
macro_rules! Edit_SetText { ($h:expr, $s:expr $(,)?) => { $crate::prelude::SetWindowTextW($h, $s) }; }
#[macro_export]
macro_rules! Edit_GetText {
    ($h:expr, $b:expr, $n:expr $(,)?) => { $crate::prelude::GetWindowTextW($h, $b, $n) };
}
#[macro_export]
macro_rules! Edit_GetTextLength { ($h:expr $(,)?) => { $crate::prelude::GetWindowTextLengthW($h) }; }
#[macro_export]
macro_rules! Edit_LimitText {
    ($h:expr, $n:expr $(,)?) => {{ $crate::prelude::SendMessageW($h, EM_LIMITTEXT as UINT, $n as WPARAM, 0); }};
}
// commctrl.h ListView_*.
//
// A cast belongs at the CALL SITE, never in one of these bodies. rustc reports a macro
// expansion's type error at the macro DEFINITION's span, so `crate-repair` walked this
// body and appended one more cast per pass — it reached eleven nested no-op casts here
// (`… as c_int) as u32) as i32) as u32) …`) and could never converge, because the two
// call sites want different types and only one of them can be satisfied at the definition.
#[macro_export]
macro_rules! ListView_GetItemCount {
    ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETITEMCOUNT as UINT, 0, 0) as c_int };
}
#[macro_export]
macro_rules! ListView_DeleteItem {
    ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_DELETEITEM as UINT, $i as WPARAM, 0) as BOOL };
}

pub use crate::{Button_SetCheck, ComboBox_AddString, ComboBox_GetCurSel, ComboBox_ResetContent, ComboBox_SetCurSel, CopyMemory, Edit_GetText, Edit_GetTextLength, Edit_LimitText, Edit_SetText, FIELD_OFFSET, FIRST_IPADDRESS, FOURTH_IPADDRESS, IsEqualCLSID, IsEqualGUID, IsEqualIID, ListBox_AddString, ListBox_GetCurSel, ListBox_GetItemData, ListBox_SetCurSel, ListBox_SetItemData, ListView_DeleteItem, ListView_GetItemCount, MAKEIPADDRESS, MAKEIPRANGE, PropSheet_Changed, SECOND_IPADDRESS, THIRD_IPADDRESS};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because netcfgx compiles -DUNICODE.
// That is this module's property, not the shared crate's, so the alias stays here.
pub use win32macros_rust::MAKEINTRESOURCEW as MAKEINTRESOURCE;

// ================= COM dispatch (COBJMACROS) ==================================================
// `#define IFACE_Method(p,a,b) (p)->lpVtbl->Method(p,a,b)` — netcfgx's precomp.h defines
// COBJMACROS, so this is how every interface call in the module is spelled.
//
// GENERATED from the island's `IFACEVtbl` structs, so a parameter list here cannot drift from
// the vtable actually dispatched through. Generic over the pointee for a reason: the C macro is
// untyped, and the module RELIES on that — e.g. inetcfgcomp_iface.c:1773 calls
// `INetCfgComponent_Release((INetCfg*)This)`, dispatching an INetCfgComponent slot through an
// INetCfg pointer. It works because `lpVtbl` is at offset 0 in every one of these objects and
// Release is IUnknown slot 2. Reproducing the C exactly means reproducing that; a signature
// pinned to one interface would reject the cast and force an edit to paid output.
#[inline(always)]
pub unsafe fn IClassFactory_QueryInterface<T>(p: *mut T, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IClassFactoryVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IClassFactory, riid, ppvObject)
}
#[inline(always)]
pub unsafe fn IClassFactory_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IClassFactoryVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IClassFactory)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgBindingInterface_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IEnumNetCfgBindingInterfaceVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IEnumNetCfgBindingInterface, riid, ppv)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgBindingInterface_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IEnumNetCfgBindingInterfaceVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IEnumNetCfgBindingInterface)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgBindingPath_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IEnumNetCfgBindingPathVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IEnumNetCfgBindingPath, riid, ppv)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgBindingPath_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IEnumNetCfgBindingPathVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IEnumNetCfgBindingPath)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgComponent_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IEnumNetCfgComponentVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IEnumNetCfgComponent, riid, ppv)
}
#[inline(always)]
pub unsafe fn IEnumNetCfgComponent_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IEnumNetCfgComponentVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IEnumNetCfgComponent)
}
#[inline(always)]
pub unsafe fn INetCfgBindingInterface_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgBindingInterfaceVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfgBindingInterface, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfgBindingInterface_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgBindingInterfaceVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgBindingInterface)
}
#[inline(always)]
pub unsafe fn INetCfgBindingPath_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgBindingPathVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfgBindingPath, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfgBindingPath_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgBindingPathVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgBindingPath)
}
#[inline(always)]
pub unsafe fn INetCfgClass_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgClassVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgClass)
}
#[inline(always)]
pub unsafe fn INetCfgClass_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgClassVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfgClass, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfgClass_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgClassVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgClass)
}
#[inline(always)]
pub unsafe fn INetCfgComponentBindings_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentBindingsVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgComponentBindings)
}
#[inline(always)]
pub unsafe fn INetCfgComponentControl_ApplyRegistryChanges<T>(p: *mut T) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentControlVtbl);
    ((*vtbl).ApplyRegistryChanges.unwrap_unchecked())(p as *mut INetCfgComponentControl)
}
#[inline(always)]
pub unsafe fn INetCfgComponentControl_CancelChanges<T>(p: *mut T) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentControlVtbl);
    ((*vtbl).CancelChanges.unwrap_unchecked())(p as *mut INetCfgComponentControl)
}
#[inline(always)]
pub unsafe fn INetCfgComponentControl_Initialize<T>(p: *mut T, pIComp: *mut INetCfgComponent, pINetCfg: *mut INetCfg, fInstalling: BOOL) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentControlVtbl);
    ((*vtbl).Initialize.unwrap_unchecked())(p as *mut INetCfgComponentControl, pIComp, pINetCfg, fInstalling)
}
#[inline(always)]
pub unsafe fn INetCfgComponentControl_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentControlVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgComponentControl)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPrivate_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentPrivateVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgComponentPrivate)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_MergePropPages<T>(p: *mut T, pdwDefPages: *mut DWORD, pahpspPrivate: *mut *mut BYTE, pcPages: *mut UINT, hwndParent: HWND, pszStartPage: *mut LPCWSTR) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).MergePropPages.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi, pdwDefPages, pahpspPrivate, pcPages, hwndParent, pszStartPage)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_QueryPropertyUi<T>(p: *mut T, pUnkReserved: *mut IUnknown) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).QueryPropertyUi.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi, pUnkReserved)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi)
}
#[inline(always)]
pub unsafe fn INetCfgComponentPropertyUi_SetContext<T>(p: *mut T, pUnkReserved: *mut IUnknown) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentPropertyUiVtbl);
    ((*vtbl).SetContext.unwrap_unchecked())(p as *mut INetCfgComponentPropertyUi, pUnkReserved)
}
#[inline(always)]
pub unsafe fn INetCfgComponent_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgComponent)
}
#[inline(always)]
pub unsafe fn INetCfgComponent_GetClassGuid<T>(p: *mut T, pGuid: *mut GUID) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentVtbl);
    ((*vtbl).GetClassGuid.unwrap_unchecked())(p as *mut INetCfgComponent, pGuid)
}
#[inline(always)]
pub unsafe fn INetCfgComponent_GetInstanceGuid<T>(p: *mut T, pGuid: *mut GUID) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentVtbl);
    ((*vtbl).GetInstanceGuid.unwrap_unchecked())(p as *mut INetCfgComponent, pGuid)
}
#[inline(always)]
pub unsafe fn INetCfgComponent_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgComponentVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfgComponent, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfgComponent_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgComponentVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfgComponent)
}
#[inline(always)]
pub unsafe fn INetCfgLock_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgLockVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgLock)
}
#[inline(always)]
pub unsafe fn INetCfgPnpReconfigCallback_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgPnpReconfigCallbackVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfgPnpReconfigCallback)
}
#[inline(always)]
pub unsafe fn INetCfg_AddRef<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgVtbl);
    ((*vtbl).AddRef.unwrap_unchecked())(p as *mut INetCfg)
}
#[inline(always)]
pub unsafe fn INetCfg_QueryInterface<T>(p: *mut T, riid: *const IID, ppv: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const INetCfgVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut INetCfg, riid, ppv)
}
#[inline(always)]
pub unsafe fn INetCfg_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const INetCfgVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut INetCfg)
}
#[inline(always)]
pub unsafe fn INetLanConnectionUiInfo_GetDeviceGuid<T>(p: *mut T, pguid: *mut GUID) -> HRESULT {
    let vtbl = *(p as *mut *const INetLanConnectionUiInfoVtbl);
    ((*vtbl).GetDeviceGuid.unwrap_unchecked())(p as *mut INetLanConnectionUiInfo, pguid)
}
#[inline(always)]
pub unsafe fn IUnknown_QueryInterface<T>(p: *mut T, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let vtbl = *(p as *mut *const IUnknownVtbl);
    ((*vtbl).QueryInterface.unwrap_unchecked())(p as *mut IUnknown, riid, ppvObject)
}
#[inline(always)]
pub unsafe fn IUnknown_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IUnknownVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IUnknown)
}

// ================= Interlocked* — compiler intrinsics, not symbols ============================
// xdk/interlocked.h makes these `#define InterlockedIncrement _InterlockedIncrement`, so there
// is no symbol to link against. Same implementation the winsock crates settled on. The C
// contract returns the value AFTER the operation, which is why the +1/-1 is there —
// `fetch_add` returns the value BEFORE. Getting that wrong off-by-ones every refcount here.
use core::sync::atomic::{AtomicI32, Ordering};

#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}

// ================= CRT ========================================================================
// msvcrt entry points. `extern "C"` (cdecl) throughout — the CRT is cdecl even where the Win32
// surface beside it is stdcall.
extern "C" {
    pub fn _wcsicmp(_Str1: *const wchar_t, _Str2: *const wchar_t) -> c_int;
    pub fn _wtoi(_Str: *const wchar_t) -> c_int;
    pub fn iswalnum(_C: wint_t) -> c_int;
    pub fn wcscat(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t;
    pub fn wcschr(
        _Str: *const c_ushort,
        _Ch: c_ushort,
    ) -> *mut c_ushort;
    pub fn wcscmp(
        _Str1: *const c_ushort,
        _Str2: *const c_ushort,
    ) -> c_int;
    pub fn wcscpy(_Dest: *mut wchar_t, _Source: *const wchar_t) -> *mut wchar_t;
    pub fn wcslen(_Str: *const c_ushort) -> usize;
    pub fn wcsrchr(_Str: *const wchar_t, _Ch: wchar_t) -> *mut wchar_t;
    pub fn wcstol(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: c_int,
    ) -> c_long;
    pub fn wcstoul(
        _Str: *const wchar_t,
        _EndPtr: *mut *mut wchar_t,
        _Radix: c_int,
    ) -> c_ulong;
}

// C-variadic: DECLARING and CALLING one is stable Rust; only *defining* one needs nightly.
// `extern "C"`, NOT "system": a variadic cannot be stdcall — the callee cannot know how many
// bytes to pop — so the CRT declares these __cdecl even under -DUNICODE.
extern "C" {
    pub fn swprintf(buffer: *mut u16, format: *const u16, ...) -> c_int;
}
// strsafe.h's printf entry points are VARIADIC, and stable Rust can CALL a C-variadic though
// it cannot define one; so these are strsafe-rust macros that call msvcrt's `_snwprintf` in
// place, with strsafe's bounds and HRESULTs. The C trampoline that used to define them
// (rust_varargs_shim.c) retired 2026-09-25. Call sites say `StringCchPrintfW!(...)`.
pub use strsafe_rust::StringCchPrintfW;

// A wide string literal, NUL-terminated, with one stable address for the program's life.
// `static` and not `const`: a const is inlined at each use site, so `.as_ptr()` would hand the
// callee the address of a temporary. ASCII-only by construction, so a non-ASCII literal fails
// at COMPILE time instead of silently truncating.
#[macro_export]
macro_rules! w {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "w!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT.as_ptr()
    }};
}
// The same literal as a SLICE, NUL included, for the `WCHAR buf[N] = L"..."` array
// initializers: C copies the literal plus its terminator into the head of the array and
// zero-fills the rest, so the copy length is `wsz!(..).len()`.
#[macro_export]
macro_rules! wsz {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        static OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "wsz!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out
        };
        &OUT[..]
    }};
}
pub use crate::{w, wsz};

// ================= Win32 imports ==============================================================
// Declared, never defined: these live in the DLLs netcfgx links against (add_importlibs).
// GENERATED by bindgen from the same header set and the same -D flags the .c compiles with,
// then checked to be IDENTICAL across all three arches — a hand-written signature that is
// merely plausible links fine and corrupts silently.
extern "system" {
    pub fn GetWindowTextLengthW(arg1: HWND) -> c_int;
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL;
    pub fn AddIPAddress(
        arg1: IPAddr,
        arg2: IPMask,
        arg3: DWORD,
        arg4: PULONG,
        arg5: PULONG,
    ) -> DWORD;
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT;
    pub fn CheckDlgButton(arg1: HWND, arg2: c_int, arg3: UINT) -> BOOL;
    pub fn CheckRadioButton(
        arg1: HWND,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
    ) -> BOOL;
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL;
    pub fn CoCreateInstance(
        rclsid: *const CLSID,
        pUnkOuter: LPUNKNOWN,
        dwClsContext: DWORD,
        iid: *const IID,
        ppv: *mut LPVOID,
    ) -> HRESULT;
    pub fn CoTaskMemAlloc(size: SIZE_T) -> LPVOID;
    pub fn CoTaskMemFree(ptr: LPVOID);
    pub fn CreateIpForwardEntry(arg1: PMIB_IPFORWARDROW) -> DWORD;
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE;
    pub fn CreatePropertySheetPageW(arg1: LPCPROPSHEETPAGEW) -> HPROPSHEETPAGE;
    pub fn DeleteIPAddress(arg1: ULONG) -> DWORD;
    pub fn DeleteIpForwardEntry(arg1: PMIB_IPFORWARDROW) -> DWORD;
    pub fn DestroyPropertySheetPage(arg1: HPROPSHEETPAGE) -> BOOL;
    pub fn DhcpCApiCleanup();
    pub fn DhcpCApiInitialize(arg1: *mut DWORD) -> DWORD;
    pub fn DhcpStaticRefreshParams(AdapterIndex: DWORD, Address: DWORD, Netmask: DWORD) -> DWORD;
    pub fn DialogBoxParamW(
        arg1: HINSTANCE,
        arg2: LPCWSTR,
        arg3: HWND,
        arg4: DLGPROC,
        arg5: LPARAM,
    ) -> INT_PTR;
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL;
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL;
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL;
    pub fn FindClose(arg1: HANDLE) -> BOOL;
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE;
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL;
    pub fn GetAdaptersInfo(arg1: PIP_ADAPTER_INFO, arg2: PULONG) -> DWORD;
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL;
    pub fn GetDlgItem(arg1: HWND, arg2: c_int) -> HWND;
    pub fn GetDlgItemInt(arg1: HWND, arg2: c_int, arg3: PBOOL, arg4: BOOL) -> UINT;
    pub fn GetIpForwardTable(arg1: PMIB_IPFORWARDTABLE, arg2: PULONG, arg3: BOOL) -> DWORD;
    pub fn GetLastError() -> DWORD;
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
    pub fn GetParent(arg1: HWND) -> HWND;
    pub fn GetPerAdapterInfo(arg1: ULONG, arg2: PIP_PER_ADAPTER_INFO, arg3: PULONG) -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn GetWindowLongW(arg1: HWND, arg2: c_int) -> LONG;
    pub fn GetWindowTextW(
        hWnd: HWND,
        lpString: LPWSTR,
        nMaxCount: c_int,
    ) -> c_int;
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT;
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID;
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL;
    pub fn InitCommonControls();
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: c_int) -> UINT;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON;
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: c_int,
    ) -> c_int;
    pub fn MessageBoxW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
    ) -> c_int;
    pub fn MultiByteToWideChar(
        arg1: UINT,
        arg2: DWORD,
        arg3: LPCSTR,
        arg4: c_int,
        arg5: LPWSTR,
        arg6: c_int,
    ) -> c_int;
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE;
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE;
    pub fn PropertySheetW(arg1: LPCPROPSHEETHEADERW) -> INT_PTR;
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS;
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        Reserved: DWORD,
        lpClass: LPWSTR,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        phkResult: PHKEY,
        lpdwDisposition: PDWORD,
    ) -> LSTATUS;
    pub fn RegDeleteValueW(arg1: HKEY, arg2: LPCWSTR) -> LSTATUS;
    pub fn RegEnumKeyExW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpName: LPWSTR,
        lpcchName: LPDWORD,
        lpReserved: LPDWORD,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS;
    pub fn RegEnumValueW(
        hKey: HKEY,
        dwIndex: DWORD,
        lpValueName: LPWSTR,
        lpcchValueName: LPDWORD,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
    pub fn RegOpenKeyExW(
        arg1: HKEY,
        arg2: LPCWSTR,
        arg3: DWORD,
        arg4: REGSAM,
        arg5: PHKEY,
    ) -> LSTATUS;
    pub fn RegQueryInfoKeyW(
        hKey: HKEY,
        lpClass: LPWSTR,
        lpcchClass: LPDWORD,
        lpReserved: LPDWORD,
        lpcSubKeys: LPDWORD,
        lpcbMaxSubKeyLen: LPDWORD,
        lpcbMaxClassLen: LPDWORD,
        lpcValues: LPDWORD,
        lpcbMaxValueNameLen: LPDWORD,
        lpcbMaxValueLen: LPDWORD,
        lpcbSecurityDescriptor: LPDWORD,
        lpftLastWriteTime: PFILETIME,
    ) -> LSTATUS;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: LPDWORD,
        lpType: LPDWORD,
        lpData: LPBYTE,
        lpcbData: LPDWORD,
    ) -> LSTATUS;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const BYTE,
        cbData: DWORD,
    ) -> LSTATUS;
    pub fn RegSetValueW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        dwType: DWORD,
        lpData: LPCWSTR,
        cbData: DWORD,
    ) -> LSTATUS;
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL;
    pub fn RpcStringFreeW(String: *mut RPC_WSTR) -> RPC_STATUS;
    pub fn SendDlgItemMessageA(
        arg1: HWND,
        arg2: c_int,
        arg3: UINT,
        arg4: WPARAM,
        arg5: LPARAM,
    ) -> LRESULT;
    pub fn SendDlgItemMessageW(
        arg1: HWND,
        arg2: c_int,
        arg3: UINT,
        arg4: WPARAM,
        arg5: LPARAM,
    ) -> LRESULT;
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT;
    pub fn SetDlgItemInt(arg1: HWND, arg2: c_int, arg3: UINT, arg4: BOOL) -> BOOL;
    pub fn SetFocus(arg1: HWND) -> HWND;
    pub fn SetLastError(arg1: DWORD);
    pub fn SetWindowLongW(arg1: HWND, arg2: c_int, arg3: LONG) -> LONG;
    pub fn SetupCloseInfFile(arg1: HINF);
    pub fn SetupDefaultQueueCallbackW(
        arg1: PVOID,
        arg2: UINT,
        arg3: UINT_PTR,
        arg4: UINT_PTR,
    ) -> UINT;
    pub fn SetupDiCreateDevRegKeyW(
        arg1: HDEVINFO,
        arg2: PSP_DEVINFO_DATA,
        arg3: DWORD,
        arg4: DWORD,
        arg5: DWORD,
        arg6: HINF,
        arg7: PCWSTR,
    ) -> HKEY;
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL;
    pub fn SetupDiEnumDeviceInfo(arg1: HDEVINFO, arg2: DWORD, arg3: PSP_DEVINFO_DATA) -> BOOL;
    pub fn SetupDiGetActualSectionToInstallW(
        InfHandle: HINF,
        InfSectionName: PCWSTR,
        InfSectionWithExt: PWSTR,
        InfSectionWithExtSize: DWORD,
        RequiredSize: PDWORD,
        Extension: *mut PWSTR,
    ) -> BOOL;
    pub fn SetupDiGetClassDevsW(
        arg1: *const GUID,
        arg2: PCWSTR,
        arg3: HWND,
        arg4: DWORD,
    ) -> HDEVINFO;
    pub fn SetupDiGetDeviceInstallParamsW(
        arg1: HDEVINFO,
        arg2: PSP_DEVINFO_DATA,
        arg3: PSP_DEVINSTALL_PARAMS_W,
    ) -> BOOL;
    pub fn SetupDiGetDeviceInstanceIdW(
        DeviceInfoSet: HDEVINFO,
        DeviceInfoData: PSP_DEVINFO_DATA,
        DeviceInstanceId: PWSTR,
        DeviceInstanceIdSize: DWORD,
        RequiredSize: PDWORD,
    ) -> BOOL;
    pub fn SetupDiGetDeviceRegistryPropertyW(
        DeviceInfoSet: HDEVINFO,
        DeviceInfoData: PSP_DEVINFO_DATA,
        Property: DWORD,
        PropertyRegDataType: PDWORD,
        PropertyBuffer: PBYTE,
        PropertyBufferSize: DWORD,
        RequiredSize: PDWORD,
    ) -> BOOL;
    pub fn SetupDiGetDriverInfoDetailW(
        DeviceInfoSet: HDEVINFO,
        DeviceInfoData: PSP_DEVINFO_DATA,
        DriverInfoData: PSP_DRVINFO_DATA_W,
        DriverInfoDetailData: PSP_DRVINFO_DETAIL_DATA_W,
        DriverInfoDetailDataSize: DWORD,
        RequiredSize: PDWORD,
    ) -> BOOL;
    pub fn SetupDiGetSelectedDriverW(
        arg1: HDEVINFO,
        arg2: PSP_DEVINFO_DATA,
        arg3: PSP_DRVINFO_DATA_W,
    ) -> BOOL;
    pub fn SetupDiInstallDevice(arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA) -> BOOL;
    pub fn SetupDiOpenDevRegKey(
        arg1: HDEVINFO,
        arg2: PSP_DEVINFO_DATA,
        arg3: DWORD,
        arg4: DWORD,
        arg5: DWORD,
        arg6: REGSAM,
    ) -> HKEY;
    pub fn SetupDiRestartDevices(arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA) -> BOOL;
    pub fn SetupDiSetDeviceInstallParamsW(
        arg1: HDEVINFO,
        arg2: PSP_DEVINFO_DATA,
        arg3: PSP_DEVINSTALL_PARAMS_W,
    ) -> BOOL;
    pub fn SetupFindFirstLineW(arg1: HINF, arg2: PCWSTR, arg3: PCWSTR, arg4: PINFCONTEXT) -> BOOL;
    pub fn SetupFindNextLine(arg1: PINFCONTEXT, arg2: PINFCONTEXT) -> BOOL;
    pub fn SetupGetIntField(arg1: PINFCONTEXT, arg2: DWORD, arg3: PINT) -> BOOL;
    pub fn SetupGetStringFieldW(
        Context: PINFCONTEXT,
        FieldIndex: DWORD,
        ReturnBuffer: PWSTR,
        ReturnBufferSize: DWORD,
        RequiredSize: PDWORD,
    ) -> BOOL;
    pub fn SetupInitDefaultQueueCallback(arg1: HWND) -> PVOID;
    pub fn SetupInstallFromInfSectionW(
        arg1: HWND,
        arg2: HINF,
        arg3: PCWSTR,
        arg4: UINT,
        arg5: HKEY,
        arg6: PCWSTR,
        arg7: UINT,
        arg8: PSP_FILE_CALLBACK_W,
        arg9: PVOID,
        arg10: HDEVINFO,
        arg11: PSP_DEVINFO_DATA,
    ) -> BOOL;
    pub fn SetupInstallServicesFromInfSectionW(arg1: HINF, arg2: PCWSTR, arg3: DWORD) -> BOOL;
    pub fn SetupOpenInfFileW(
        FileName: PCWSTR,
        InfClass: PCWSTR,
        InfStyle: DWORD,
        ErrorLine: PUINT,
    ) -> HINF;
    pub fn SetupTermDefaultQueueCallback(arg1: PVOID);
    pub fn ShowWindow(arg1: HWND, arg2: c_int) -> BOOL;
    pub fn StartServiceW(
        hService: SC_HANDLE,
        dwNumServiceArgs: DWORD,
        lpServiceArgVectors: *mut LPCWSTR,
    ) -> BOOL;
    pub fn StringFromCLSID(id: *const CLSID, arg1: *mut LPOLESTR) -> HRESULT;
    pub fn UuidCreate(Uuid: *mut UUID) -> RPC_STATUS;
    pub fn UuidToStringW(Uuid: *mut UUID, StringUuid: *mut RPC_WSTR) -> RPC_STATUS;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn htonl(arg1: u_long) -> u_long;
}

// winuser.h only EXPORTS GetWindowLongPtrW/SetWindowLongPtrW on _WIN64; on 32-bit they are
// `#define`s onto the plain LONG entry points. Declaring the Ptr forms unconditionally would
// link-fail on i386 against a symbol user32 does not export there. LONG and LONG_PTR are the
// same width on 32-bit, so the alias is exact either way.
#[cfg(target_pointer_width = "64")]
extern "system" {
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}
#[cfg(not(target_pointer_width = "64"))]
pub use self::{GetWindowLongW as GetWindowLongPtrW, SetWindowLongW as SetWindowLongPtrW};

// The UNICODE `#define FOO FOOW` renames, as aliases so there is exactly one symbol identity.
pub use self::{
    GetWindowLongPtrW as GetWindowLongPtr, SendDlgItemMessageW as SendDlgItemMessage,
    SendMessageW as SendMessage, SetWindowLongPtrW as SetWindowLongPtr,
    SetupDiGetClassDevsW as SetupDiGetClassDevs, StartServiceW as StartService,
};

// ================= sibling-TU declarations ====================================================
// precomp.h ends with a block of prototypes for the functions each TU defines and the others
// call (the constructors, plus netinstall's InstallNetworkComponent). Re-exported here so every
// module sees them through `use crate::prelude::*`, exactly as every .c saw them through the PCH.
pub use crate::netcfgx::{
    classfactory::IClassFactory_fnConstructor,
    inetcfgcomp_iface::{IEnumNetCfgComponent_Constructor, INetCfgComponent_Constructor},
    netcfg_iface::INetCfg_Constructor,
    netcfgbindinginterface_iface::{
        IEnumNetCfgBindingInterface_Constructor, INetCfgBindingInterface_Constructor,
    },
    netcfgbindingpath_iface::{IEnumNetCfgBindingPath_Constructor, INetCfgBindingPath_Constructor},
    netcfgclass_iface::INetCfgClass_Constructor,
    netinstall::InstallNetworkComponent,
    tcpipconf_notify::TcpipConfigNotify_Constructor,
};

// winnt.h `#define TRUE 1` / `FALSE 0`. ntdll-sys's bindgen types them u32; every use in this
// module is a BOOL (a `return TRUE` from a DLGPROC, an `EndDialog(hwnd, FALSE)`). An explicit
// item beats a glob import, so naming them here re-types them crate-wide without a cast at
// each of the 36 sites.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// Case labels, and a match PATTERN cannot carry a cast. `IDCANCEL` reaches these switches
// through C's int promotion of LOWORD(wParam); `PSCB_INITIALIZED` is a UINT message code.
// Both arrive from ntdll-sys as u32; an explicit item beats the glob import.
pub const IDCANCEL: c_int = 2;        // winuser.h
pub const PSCB_INITIALIZED: UINT = 1; // prsht.h
