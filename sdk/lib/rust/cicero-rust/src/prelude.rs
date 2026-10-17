//! cicero-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= ATL ========================================================================
// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.
pub use atl_rust::{atl_format, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CSimpleMap, CStringW, POSITION};

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
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FD_SET, FD_ZERO, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_variant, debugstr_vt, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_longlong, wine_dbgstr_variant, wine_dbgstr_vt, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// REFIID/REFCLSID: C++'s `const IID &`, a pointer at the ABI (as the island's vtables take
// it). resolve leaves them to a human: a dozen SDK headers each define them.
pub type REFIID = *const IID;
pub type REFCLSID = *const IID;
// cicbase.h's cicMemAllocClear is translated twice (cicbase_h, cicbase_h_h): the pub one.
pub use crate::cicero::cicbase_h_h::cicMemAllocClear;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::cicero::cpp_classes::*;
pub use crate::cicero::cpp_classes::{CIC_IMC_LOCK, CIC_IMC_LOCK__delete, CIC_IMC_LOCK__make_0, CIC_IMC_LOCK__new_0, CIC_IMC_LOCK__new_in_0, CIC_IMC_LOCK__zeroed, CUIFBalloonButton, CUIFBalloonButton__VTBL_b0b0b0, CUIFBalloonButton__delete, CUIFBalloonButton__make_0, CUIFBalloonButton__new_0, CUIFBalloonButton__new_in_0, CUIFBalloonButton__zeroed, CUIFBalloonWindow, CUIFBalloonWindow__VTBL_b0b0b0, CUIFBalloonWindow__delete, CUIFBalloonWindow__make_0, CUIFBalloonWindow__new_0, CUIFBalloonWindow__new_in_0, CUIFBalloonWindow__zeroed, CUIFBitmapDC, CUIFBitmapDC__delete, CUIFBitmapDC__make_0, CUIFBitmapDC__new_0, CUIFBitmapDC__new_in_0, CUIFBitmapDC__s_fInitBitmapDCs, CUIFBitmapDC__s_phdcDst, CUIFBitmapDC__s_phdcMask, CUIFBitmapDC__s_phdcSrc, CUIFBitmapDC__zeroed, CUIFButton, CUIFButton2, CUIFButton2__VTBL_b0b0b0, CUIFButton2__delete, CUIFButton2__make_0, CUIFButton2__new_0, CUIFButton2__new_in_0, CUIFButton2__zeroed, CUIFButton__VTBL_b0b0, CUIFButton__delete, CUIFButton__make_0, CUIFButton__new_0, CUIFButton__new_in_0, CUIFButton__vtbl, CUIFButton__zeroed, CUIFColorTable, CUIFColorTableOff10, CUIFColorTableOff10__VTBL_b0, CUIFColorTableOff10__delete, CUIFColorTableOff10__make_0, CUIFColorTableOff10__new_0, CUIFColorTableOff10__new_in_0, CUIFColorTableOff10__zeroed, CUIFColorTableSys, CUIFColorTableSys__VTBL_b0, CUIFColorTableSys__delete, CUIFColorTableSys__make_0, CUIFColorTableSys__new_0, CUIFColorTableSys__new_in_0, CUIFColorTableSys__zeroed, CUIFColorTable__VTBL_own, CUIFColorTable__delete, CUIFColorTable__make_0, CUIFColorTable__new_0, CUIFColorTable__new_in_0, CUIFColorTable__vtbl, CUIFColorTable__zeroed, CUIFGripper, CUIFGripper__VTBL_b0b0, CUIFGripper__delete, CUIFGripper__make_0, CUIFGripper__new_0, CUIFGripper__new_in_0, CUIFGripper__zeroed, CUIFIcon, CUIFIcon__delete, CUIFIcon__make_0, CUIFIcon__new_0, CUIFIcon__new_in_0, CUIFIcon__zeroed, CUIFMenu, CUIFMenuItem, CUIFMenuItemSeparator, CUIFMenuItemSeparator__VTBL_b0b0b0, CUIFMenuItemSeparator__delete, CUIFMenuItemSeparator__make_0, CUIFMenuItemSeparator__new_0, CUIFMenuItemSeparator__new_in_0, CUIFMenuItemSeparator__zeroed, CUIFMenuItem__VTBL_b0b0, CUIFMenuItem__delete, CUIFMenuItem__make_0, CUIFMenuItem__new_0, CUIFMenuItem__new_in_0, CUIFMenuItem__vtbl, CUIFMenuItem__zeroed, CUIFMenu__VTBL_b0b0b0, CUIFMenu__delete, CUIFMenu__make_0, CUIFMenu__new_0, CUIFMenu__new_in_0, CUIFMenu__vtbl, CUIFMenu__zeroed, CUIFObject, CUIFObjectArray, CUIFObjectArray__delete, CUIFObjectArray__make_0, CUIFObjectArray__new_0, CUIFObjectArray__new_in_0, CUIFObjectArray__zeroed, CUIFObject__VTBL_b0, CUIFObject__delete, CUIFObject__make_0, CUIFObject__new_0, CUIFObject__new_in_0, CUIFObject__vtbl, CUIFObject__zeroed, CUIFScheme, CUIFSchemeDef, CUIFSchemeDef__VTBL_b0, CUIFSchemeDef__delete, CUIFSchemeDef__make_0, CUIFSchemeDef__new_0, CUIFSchemeDef__new_in_0, CUIFSchemeDef__zeroed, CUIFScheme__VTBL_own, CUIFScheme__delete, CUIFScheme__make_0, CUIFScheme__new_0, CUIFScheme__new_in_0, CUIFScheme__s_pColorTableOff10, CUIFScheme__s_pColorTableSys, CUIFScheme__vtbl, CUIFScheme__zeroed, CUIFShadow, CUIFShadow__VTBL_b0b0b0, CUIFShadow__delete, CUIFShadow__make_0, CUIFShadow__new_0, CUIFShadow__new_in_0, CUIFShadow__zeroed, CUIFSolidBrush, CUIFSolidBrush__delete, CUIFSolidBrush__make_0, CUIFSolidBrush__new_0, CUIFSolidBrush__new_in_0, CUIFSolidBrush__zeroed, CUIFSystemInfo, CUIFSystemInfo__delete, CUIFSystemInfo__make_0, CUIFSystemInfo__new_0, CUIFSystemInfo__new_in_0, CUIFSystemInfo__s_pSystemInfo, CUIFSystemInfo__zeroed, CUIFTheme, CUIFTheme__VTBL_own, CUIFTheme__delete, CUIFTheme__make_0, CUIFTheme__new_0, CUIFTheme__new_in_0, CUIFTheme__s_fnCloseThemeData, CUIFTheme__s_fnDrawThemeBackground, CUIFTheme__s_fnDrawThemeEdge, CUIFTheme__s_fnDrawThemeIcon, CUIFTheme__s_fnDrawThemeParentBackground, CUIFTheme__s_fnDrawThemeText, CUIFTheme__s_fnGetThemeBackgroundContentRect, CUIFTheme__s_fnGetThemeBackgroundExtent, CUIFTheme__s_fnGetThemeColor, CUIFTheme__s_fnGetThemeFont, CUIFTheme__s_fnGetThemeMargins, CUIFTheme__s_fnGetThemePartSize, CUIFTheme__s_fnGetThemeSysColor, CUIFTheme__s_fnGetThemeSysSize, CUIFTheme__s_fnGetThemeTextExtent, CUIFTheme__s_fnOpenThemeData, CUIFTheme__s_hUXTHEME, CUIFTheme__vtbl, CUIFTheme__zeroed, CUIFToolTip, CUIFToolTip__VTBL_b0b0b0, CUIFToolTip__delete, CUIFToolTip__make_0, CUIFToolTip__new_0, CUIFToolTip__new_in_0, CUIFToolTip__zeroed, CUIFToolbarButton, CUIFToolbarButtonElement, CUIFToolbarButtonElement__VTBL_b0b0b0b0, CUIFToolbarButtonElement__delete, CUIFToolbarButtonElement__make_0, CUIFToolbarButtonElement__new_0, CUIFToolbarButtonElement__new_in_0, CUIFToolbarButtonElement__zeroed, CUIFToolbarButton__VTBL_b0b0, CUIFToolbarButton__delete, CUIFToolbarButton__make_0, CUIFToolbarButton__new_0, CUIFToolbarButton__new_in_0, CUIFToolbarButton__vtbl, CUIFToolbarButton__zeroed, CUIFToolbarMenuButton, CUIFToolbarMenuButton__VTBL_b0b0b0b0, CUIFToolbarMenuButton__delete, CUIFToolbarMenuButton__make_0, CUIFToolbarMenuButton__new_0, CUIFToolbarMenuButton__new_in_0, CUIFToolbarMenuButton__zeroed, CUIFWindow, CUIFWindow__VTBL_b0b0, CUIFWindow__delete, CUIFWindow__make_0, CUIFWindow__new_0, CUIFWindow__new_in_0, CUIFWindow__vtbl, CUIFWindow__zeroed, CUIFWndFrame, CUIFWndFrame__VTBL_b0b0, CUIFWndFrame__delete, CUIFWndFrame__make_0, CUIFWndFrame__new_0, CUIFWndFrame__new_in_0, CUIFWndFrame__zeroed, CicArrayBase, CicArrayBase__VTBL_own, CicArrayBase__delete, CicArrayBase__make_0, CicArrayBase__new_0, CicArrayBase__new_in_0, CicArrayBase__vtbl, CicArrayBase__zeroed, CicCaret, CicCaret__VTBL_own, CicCaret__delete, CicCaret__make_0, CicCaret__new_0, CicCaret__new_in_0, CicCaret__vtbl, CicCaret__zeroed, CicFileMapping, CicFileMappingStatic, CicFileMappingStatic__delete, CicFileMappingStatic__make_0, CicFileMappingStatic__new_0, CicFileMappingStatic__new_in_0, CicFileMappingStatic__zeroed, CicFileMapping__VTBL_own, CicFileMapping__delete, CicFileMapping__make_0, CicFileMapping__new_0, CicFileMapping__new_in_0, CicFileMapping__vtbl, CicFileMapping__zeroed, CicIMCLock, CicIMCLock__delete, CicIMCLock__make_0, CicIMCLock__new_0, CicIMCLock__new_in_0, CicIMCLock__zeroed, CicMutex, CicMutex__delete, CicMutex__make_0, CicMutex__new_0, CicMutex__new_in_0, CicMutex__zeroed, CicNoThrow__delete, CicNoThrow__make_0, CicNoThrow__new_0, CicNoThrow__new_in_0, CicNoThrow__zeroed, CicRegKey, CicRegKey__delete, CicRegKey__make_0, CicRegKey__new_0, CicRegKey__new_in_0, CicRegKey__zeroed, CicSystemModulePath, CicSystemModulePath__delete, CicSystemModulePath__make_0, CicSystemModulePath__new_0, CicSystemModulePath__new_in_0, CicSystemModulePath__zeroed, POINTING_TIMER_ID, ProcessBasicInformation, ProcessBreakOnTermination, ProcessDebugPort, ProcessImageFileName, ProcessWow64Information, TIMER_ID, TOOLTIP_TIMER_ID, USER_TIMER_ID, _PROCESSINFOCLASS, cpp_alloc, cpp_alloc_CicNoThrow, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_array_CicNoThrow, cpp_new_in, cpp_pure_virtual_call, tagCTFIMECONTEXT__delete, tagCTFIMECONTEXT__make_0, tagCTFIMECONTEXT__new_0, tagCTFIMECONTEXT__new_in_0, tagCTFIMECONTEXT__zeroed};
// ==== end port cppapp ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const BP_PUSHBUTTON: BUTTONPARTS = BUTTONPARTS_BP_PUSHBUTTON; // island enum
pub const ID: i32 = 10; // ftp-rust/src/ftp/ruserpass.rs
pub const ILC_COLOR32: u32 = 32; // access-rust/src/types_amd64.rs
pub const ILC_MASK: u32 = 1; // access-rust/src/types_amd64.rs
pub const ILC_MIRROR: u32 = 8192; // access-rust/src/types_amd64.rs
pub const RP_GRIPPER: REBARPARTS = REBARPARTS_RP_GRIPPER; // island enum
pub const RP_GRIPPERVERT: REBARPARTS = REBARPARTS_RP_GRIPPERVERT; // island enum
extern "C" {
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FlushViewOfFile(arg1: LPCVOID, arg2: SIZE_T) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetACP() -> UINT; // desk-rust/src/types_i386.rs
    pub fn GetCurrentProcess() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_i386.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalSize(arg1: HLOCAL) -> SIZE_T; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn lstrcpynW(arg1: LPWSTR, arg2: LPCWSTR, arg3: ::core::ffi::c_int) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::cicero::cicreg_cpp_h::_cicRegKey_Create;
pub use crate::cicero::cicreg_cpp_h::_cicRegKey_EnumValue;
pub use crate::cicero::cicreg_cpp_h::_cicRegKey_Open;
pub use crate::cicero::cicreg_cpp_h::_cicRegKey_QuerySz;
pub use crate::cicero::cicreg_cpp_h::_cicRegKey_RecurseDeleteKey;
pub use crate::cicero::cicbase_h_h::cicGetFN__T1;
pub use crate::cicero::cicbase_h_h::cicGetFN__T10;
pub use crate::cicero::cicbase_h_h::cicGetFN__T11;
pub use crate::cicero::cicbase_h_h::cicGetFN__T12;
pub use crate::cicero::cicbase_h_h::cicGetFN__T13;
pub use crate::cicero::cicbase_h_h::cicGetFN__T14;
pub use crate::cicero::cicbase_h_h::cicGetFN__T2;
pub use crate::cicero::cicbase_h_h::cicGetFN__T3;
pub use crate::cicero::cicbase_h_h::cicGetFN__T4;
pub use crate::cicero::cicbase_h_h::cicGetFN__T5;
pub use crate::cicero::cicbase_h_h::cicGetFN__T6;
pub use crate::cicero::cicbase_h_h::cicGetFN__T7;
pub use crate::cicero::cicbase_h_h::cicGetFN__T8;
pub use crate::cicero::cicbase_h_h::cicGetFN__T9;
pub use crate::cicero::cicbase_cpp_h::cicLoadSystemLibrary;
pub use crate::cicero::cicbase_h_h::cicMemFree;
pub use crate::cicero::cicbase_h_h::cicMemReAlloc;
pub use crate::cicero::cicbase_cpp_h::cxx_operator_delete__0;
pub use crate::cicero::cicbase_cpp_h::cxx_operator_delete_array__0;
pub use crate::cicero::cicbase_cpp_h::cxx_operator_new;
pub use crate::cicero::cicbase_cpp_h::cxx_operator_new_array;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CUSTOM_CAND_INFO_SIZE: i32 = 0x7AC; // clang, cicimc.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CreateDCW( pszDriver: LPCWSTR, pszDevice: LPCWSTR, psz: LPCWSTR, pdmInit: *const DEVMODEW, ) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFontIndirectW(arg1: *const LOGFONTW) -> HFONT; // aclui-rust/src/types_i386.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DrawStateW( arg1: HDC, arg2: HBRUSH, arg3: DRAWSTATEPROC, arg4: LPARAM, arg5: WPARAM, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: ::core::ffi::c_int, arg9: ::core::ffi::c_int, arg10: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetClassInfoExW(arg1: HINSTANCE, arg2: LPCWSTR, arg3: LPWNDCLASSEXW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetMonitorInfoW(arg1: HMONITOR, arg2: LPMONITORINFO) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetObjectW(h: HANDLE, c: ::core::ffi::c_int, pv: LPVOID) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetSystemWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTextMetricsW(arg1: HDC, arg2: LPTEXTMETRICW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetVersionExW(arg1: LPOSVERSIONINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    // i386 only: there `GetWindowLongPtrW` is a winuser.h macro for `GetWindowLongW`, so the island has no
    // declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_arch = "x86")]
    #[link_name = "GetWindowLongW"]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn OpenFileMappingW(arg1: DWORD, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: LPCWSTR) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegisterClassExW(arg1: *const WNDCLASSEXW) -> ATOM; // aclui-rust/src/types_i386.rs
    // i386 only: there `SetWindowLongPtrW` is a winuser.h macro for `SetWindowLongW`, so the island has no
    // declaration of it. On 64-bit the island's header-exact one is used.
    #[cfg(target_arch = "x86")]
    #[link_name = "SetWindowLongW"]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::CreateDCW as CreateDC; // sdk alias
pub use crate::prelude::CreateFileMappingW as CreateFileMapping; // sdk alias
pub use crate::prelude::CreateFontIndirectW as CreateFontIndirect; // sdk alias
pub use crate::prelude::CreateMutexW as CreateMutex; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DefWindowProcW as DefWindowProc; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::DrawStateW as DrawState; // sdk alias
pub use crate::prelude::GetClassInfoExW as GetClassInfoEx; // sdk alias
pub use crate::prelude::GetMessageW as GetMessage; // sdk alias
pub use crate::prelude::GetModuleHandleW as GetModuleHandle; // sdk alias
pub use crate::prelude::GetMonitorInfoW as GetMonitorInfo; // sdk alias
pub use crate::prelude::GetObjectW as GetObject; // sdk alias
pub use crate::prelude::GetSystemDirectoryW as GetSystemDirectory; // sdk alias
pub use crate::prelude::GetSystemWindowsDirectoryW as GetSystemWindowsDirectory; // sdk alias
pub use crate::prelude::GetTextMetricsW as GetTextMetrics; // sdk alias
pub use crate::prelude::GetVersionExW as GetVersionEx; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadLibraryW as LoadLibrary; // sdk alias
pub use crate::prelude::OpenFileMappingW as OpenFileMapping; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegDeleteKeyW as RegDeleteKey; // sdk alias
pub use crate::prelude::RegDeleteValueW as RegDeleteValue; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegEnumValueW as RegEnumValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::RegisterClassExW as RegisterClassEx; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use strsafe_rust::StringCchCatW as StringCchCat; // sdk alias
pub use crate::prelude::SystemParametersInfoW as SystemParametersInfo; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
pub use crate::prelude::lstrlenW as lstrlen; // sdk alias
// ==== end port externs ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
pub type CREATESTRUCT = CREATESTRUCTW;
pub type HIGHCONTRAST = HIGHCONTRASTW;
pub type OSVERSIONINFO = OSVERSIONINFOW;
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const IDC_SIZEALL: LPWSTR = 0x7f86usize as LPWSTR;
pub const IDC_SIZENESW: LPWSTR = 0x7f83usize as LPWSTR;
pub const IDC_SIZENS: LPWSTR = 0x7f85usize as LPWSTR;
pub const IDC_SIZENWSE: LPWSTR = 0x7f82usize as LPWSTR;
pub const IDC_SIZEWE: LPWSTR = 0x7f84usize as LPWSTR;
pub const S_OK: HRESULT = ((0) as HRESULT);
pub use crate::{MAKELPARAM, max};
// ==== end SDK header macros ====

// hand: cicbase.h's cicGetFN template (cicbase_h.rs) and the one libloaderapi call it makes.
pub use crate::cicero::cicbase_h::cicGetFN;
extern "system" {
    // FARPROC is not in the island; cicGetFN only reinterprets the pointer as its T_FN.
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void;
}

extern "system" {
    pub fn RegDeleteValueW(hKey: HKEY, lpValueName: LPCWSTR) -> LSTATUS; // hand: cicreg.h RegDeleteValue (UNICODE)
}
pub use crate::cicero::cicimc_t::CicIMCCLock__make_0; // hand: the template generic's by-value constructor
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};
