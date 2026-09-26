//! sysdm-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
//!
//! The CRT block, the wide-literal macros, the UNICODE A/W aliases, the enum-prefix aliases and
//! the header-macro block below are desk-rust's, carried across verbatim: they are Win32/CRT
//! surface, not desk's own, and four property-sheet modules have now needed the same set. That
//! duplication is a detection event in its own right (see strsafe-rust for the same story), but
//! a prelude also globs its OWN island, so the shared part cannot simply be a crate the way
//! strsafe could — the split has to be macros-and-externs versus island-glob, and that is a
//! refactor to do deliberately rather than in the middle of a flip.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{
    c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ulong, c_ushort, c_void,
};
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

// The rest of the CRT surface desk uses. bindgen does not emit these because the .c reach them
// through the CRT headers, not the SDK ones in the island's allowlist. Declared, never
// implemented — msvcrt exports every one.
extern "C" {
    pub fn malloc(n: usize) -> *mut c_void;
    pub fn free(p: *mut c_void);
    pub fn labs(n: c_long) -> c_long;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcsncmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcstok(s: *mut u16, delim: *const u16) -> *mut u16;
    pub fn _wcsdup(s: *const u16) -> *mut u16;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsnicmp(a: *const u16, b: *const u16, n: usize) -> c_int;
    pub fn _wtoi(s: *const u16) -> c_int;
    // Variadic DECLARATIONS are stable Rust; only a variadic DEFINITION needs c_variadic. These
    // two are real msvcrt exports, so nothing in the tree has to define them.
    pub fn _snwprintf(buf: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;
    pub fn swprintf(buf: *mut u16, fmt: *const u16, ...) -> c_int;
}
// The TCHAR CRT spellings, resolved by the PREPROCESSOR under -DUNICODE and so never present in
// any object file. Aliases onto the wide entry points above, never re-declarations: a second
// prototype that is merely plausible is the shape that links and then corrupts.
pub use self::{
    _snwprintf as _sntprintf, _wcsicmp as _tcsicmp, _wtoi as _ttoi, swprintf as _stprintf,
    wcslen as _tcslen, wcsrchr as _tcsrchr, wcstok as _tcstok,
};

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// C's `sizeof(T)`, which the translator lowers to this turbofish spelling. A const generic fn,
// not a macro: `sizeof::<PROPSHEETHEADER>()` is already the call form in the translated bodies.
#[inline(always)]
pub const fn sizeof<T>() -> usize {
    core::mem::size_of::<T>()
}

// ================= wide literals ==============================================================
// This replaces the byte-string-as-wide-pointer form, which is wrong twice over: `b"..."` has
// ALIGNMENT 1 (an odd address is an amd64-only 0x1E STATUS_DATATYPE_MISALIGNMENT), and Rust
// does not append the SECOND zero byte a UTF-16 terminator needs — `b"\\\0"` is one code unit
// with no terminator at all, and an odd byte count cannot be whole u16 units.
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
pub use crate::w;
// The ARRAY sibling of `w!`. A C `static const WCHAR x[] = L"..."` is an ARRAY, and the
// difference matters twice: `[WCHAR; N]` is `Sync` where `*const u16` is not (so it can BE a
// static), and the declared `N` is checked against the literal instead of being a number
// somebody counted.
#[macro_export]
macro_rules! wide {
    ($s:expr $(,)?) => {{
        const INPUT: &[u8] = $s.as_bytes();
        const OUT: [u16; INPUT.len() + 1] = {
            let mut out = [0u16; INPUT.len() + 1];
            let mut i = 0;
            while i < INPUT.len() {
                assert!(INPUT[i] < 0x80, "wide!() takes ASCII literals only");
                out[i] = INPUT[i] as u16;
                i += 1;
            }
            out // trailing element stays 0 => NUL-terminated
        };
        OUT
    }};
}
pub use crate::wide;
// A C file-scope `static const` TABLE OF POINTERS has no direct Rust spelling: a raw pointer is
// not `Sync`, so an array of them cannot be a `static`. This newtype promises exactly what the C
// already promises — the table is read-only and every pointer in it is `'static` (each one comes
// from `w!`, i.e. another static). `Deref` keeps the call sites spelled as the C spells them.
#[repr(transparent)]
pub struct SyncTable<T: 'static>(pub T);
unsafe impl<T: 'static> Sync for SyncTable<T> {}
impl<T: 'static> core::ops::Deref for SyncTable<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

// ================= UNICODE A/W aliases ========================================================
// desk compiles with -DUNICODE, so `#define SendMessage SendMessageW` and its siblings are
// resolved by the PREPROCESSOR and never reach the island. DERIVED, not typed: each name below
// was produced by taking the island's `<N>W` for every unresolved `<N>` the compiler named, so a
// pair cannot be mismatched. Aliases and not re-declarations — a hand-written prototype for
// SendMessageW is the exact shape that links and then corrupts.
pub use crate::types::{
    CallWindowProcW as CallWindowProc, ChangeDisplaySettingsExW as ChangeDisplaySettingsEx,
    ChooseColorW as ChooseColor, CreateFontIndirectW as CreateFontIndirect,
    CreateICW as CreateIC, CreateProcessW as CreateProcess,
    CreatePropertySheetPageW as CreatePropertySheetPage, CreateWindowExW as CreateWindowEx,
    DefWindowProcW as DefWindowProc, DialogBoxParamW as DialogBoxParam,
    DispatchMessageW as DispatchMessage, DrawTextW as DrawText,
    EnumDisplayDevicesW as EnumDisplayDevices, EnumDisplaySettingsExW as EnumDisplaySettingsEx,
    EnumFontFamiliesExW as EnumFontFamiliesEx,
    ExpandEnvironmentStringsW as ExpandEnvironmentStrings, ExtractIconW as ExtractIcon,
    FindFirstFileW as FindFirstFile, FindNextFileW as FindNextFile,
    GetClipboardFormatNameW as GetClipboardFormatName, GetDlgItemTextW as GetDlgItemText,
    GetModuleFileNameW as GetModuleFileName, GetModuleHandleW as GetModuleHandle,
    GetObjectW as GetObject, GetOpenFileNameW as GetOpenFileName,
    GetSystemDirectoryW as GetSystemDirectory, GetTextMetricsW as GetTextMetrics,
    GetWindowsDirectoryW as GetWindowsDirectory, IsDialogMessageW as IsDialogMessage,
    FindResourceW as FindResource, LoadCursorW as LoadCursor, LoadImageW as LoadImage,
    LoadLibraryExW as LoadLibraryEx, LoadMenuW as LoadMenu,
    LoadStringW as LoadString, MessageBoxW as MessageBox, lstrcpynW as lstrcpyn, PathCombineW as PathCombine,
    PathParseIconLocationW as PathParseIconLocation,
    PathRemoveExtensionW as PathRemoveExtension,
    PathUnExpandEnvStringsW as PathUnExpandEnvStrings, PeekMessageW as PeekMessage,
    PostMessageW as PostMessage, PropertySheetW as PropertySheet,
    RegCreateKeyExW as RegCreateKeyEx, RegDeleteValueW as RegDeleteValue,
    RegOpenKeyExW as RegOpenKeyEx, RegQueryValueExW as RegQueryValueEx,
    RegSetValueExW as RegSetValueEx, RegisterClassExW as RegisterClassEx,
    RegisterClassW as RegisterClass, RegisterClipboardFormatW as RegisterClipboardFormat,
    SHGetFileInfoW as SHGetFileInfo, SHGetFolderPathW as SHGetFolderPath,
    SHGetValueW as SHGetValue, SHRegGetBoolUSValueW as SHRegGetBoolUSValue,
    SHRegGetPathW as SHRegGetPath, SHSetValueW as SHSetValue,
    SendDlgItemMessageW as SendDlgItemMessage, SendMessageW as SendMessage,
    SetDlgItemTextW as SetDlgItemText, SetWindowTextW as SetWindowText,
    SetupDiGetActualSectionToInstallW as SetupDiGetActualSectionToInstall,
    SetupDiGetDeviceInstallParamsW as SetupDiGetDeviceInstallParams,
    SetupDiGetDeviceInstanceIdW as SetupDiGetDeviceInstanceId,
    SetupDiGetDeviceRegistryPropertyW as SetupDiGetDeviceRegistryProperty,
    SetupDiGetDriverInfoDetailW as SetupDiGetDriverInfoDetail,
    SetupDiGetSelectedDriverW as SetupDiGetSelectedDriver,
    SetupDiSetDeviceInstallParamsW as SetupDiSetDeviceInstallParams,
    SetupFindFirstLineW as SetupFindFirstLine, SetupGetStringFieldW as SetupGetStringField,
    SetupInstallFromInfSectionW as SetupInstallFromInfSection,
    SetupOpenInfFileW as SetupOpenInfFile, SystemParametersInfoW as SystemParametersInfo,
    TextOutW as TextOut, UnregisterClassW as UnregisterClass, lstrcpyW as lstrcpy,
    lstrlenW as lstrlen, wsprintfW as wsprintf,
};
// The A/W TYPE half of the same rule.
pub use crate::types::{
    LPFNPSPCALLBACKW as LPFNPSPCALLBACK, LPPROPSHEETPAGEW as LPPROPSHEETPAGE,
    LVFINDINFOW as LVFINDINFO, LVITEMW as LVITEM, PROPSHEETHEADERW as PROPSHEETHEADER,
    PROPSHEETPAGEW as PROPSHEETPAGE,
};
// commctrl.h's own legacy spellings: `#define LV_ITEM LVITEM` on top of the A/W pair, so under
// UNICODE `LV_ITEM` is two macro hops from `LVITEMW` and reaches the island as neither.
pub use crate::types::{
    LVCOLUMNW as LVCOLUMN, LVCOLUMNW as LV_COLUMN, LVFINDINFOW as LV_FINDINFO,
    LVITEMW as LV_ITEM,
};
// The A/W STRUCT family, for exactly the reason spelled out below: ntdll-sys is built WITHOUT
// UNICODE, so its glob supplies `WIN32_FIND_DATA = WIN32_FIND_DATAA` and friends, and a UNICODE
// module that picks those up passes an ANSI layout to a `...W` API — which links and then reads
// the wrong bytes, since the two structs differ in the width of every character field.
pub use crate::types::{
    STARTUPINFOW as STARTUPINFO, WIN32_FIND_DATAW as WIN32_FIND_DATA,
    WNDCLASSEXW as WNDCLASSEX, WNDCLASSW as WNDCLASS,
};
// The TCHAR TYPE family, spelled out rather than inherited: ntdll-sys is built WITHOUT UNICODE,
// so its glob supplies `LPTSTR = LPSTR`, and a UNICODE module that picks that up has a
// correctness bug rather than a compile error — a `size_of::<TCHAR>()` allocation would compile
// clean and size every buffer at half.
pub use crate::types::{
    LPCWSTR as LPCTSTR, LPWSTR as LPTSTR, PCWSTR as PCTSTR, PWSTR as PTSTR, WCHAR as TCHAR,
};
// On i386 the Ptr forms are `#define`s onto the 32-bit calls, so they have no `W` twin there.
#[cfg(target_pointer_width = "32")]
pub use crate::types::{
    GetWindowLongW as GetWindowLongPtr, GetWindowLongW as GetWindowLongPtrW,
    SetWindowLongW as SetWindowLongPtr, SetWindowLongW as SetWindowLongPtrW,
};
#[cfg(target_pointer_width = "64")]
pub use crate::types::{
    GetWindowLongPtrW as GetWindowLongPtr, SetWindowLongPtrW as SetWindowLongPtr,
};

// ================= enum members bindgen PREFIXES ==============================================
// bindgen prefixes an anonymous enum's members with the typedef name, so the C's bare
// `DVASPECT_CONTENT` is emitted as `tagDVASPECT_DVASPECT_CONTENT`. Aliased rather than
// re-declared with a hand-typed value: a second `const` that is merely plausible is exactly how
// a `match` arm ends up comparing against the wrong number (§9.112). Every line below was
// derived by grepping the island for the suffix, not typed — and the widl-GENERATED COM headers
// (objidl.h, wtypes.h) had to be added to the profile's allowlist first, because
// `--allowlist-file` matches where an item is DEFINED and objbase.h merely #includes them.
// vsstyle.h's theme part/state enums, same prefixing rule.

// ================= constants bindgen cannot fold ==============================================
// Cast expressions and macro expressions in their headers, so the island carries none of them.
// The value is transcribed WITH the header and line, because a hand-typed constant that is
// merely plausible is exactly how a `match` arm ends up comparing against the wrong number.
// ntdll-sys's island types these `u32` (bindgen's default for a bare `#define 1`), but every
// use in this module is a BOOL — a return value, a struct field, a Win32 argument — and BOOL is
// a signed LONG. Shadowing the glob here is two lines; the alternative was ~90 per-site casts,
// each of which would have had to be checked for being the RIGHT cast.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
// gdiplustypes.h's `Status` enum, whose OK member is spelled `Ok`. Left unaliased it resolves
// to Rust's own `Result::Ok` — which is a CONSTRUCTOR, so `!= Ok` compiles as far as a type
// error rather than a name error, and on a less lucky day would compile outright.
pub const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFFFFFF; // winuser.h:179 `((DWORD)-1)`
// winuser.h:1215 `((HWND)0xffff)`. Stays here: HWND is `*mut HWND__`, a nominal opaque struct
// this island emits for itself, so it cannot come from a shared crate.
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND;
// winuser.h `#define IDC_ARROW MAKEINTRESOURCE(32512)` / `#define RT_STRING MAKEINTRESOURCE(6)`
// — a resource ORDINAL smuggled through a pointer, which is why the type is LPCWSTR and not an
// integer: every call site hands it straight to LoadCursorW / FindResourceW.
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // winuser.h:695
pub const RT_STRING: LPCWSTR = 6usize as LPCWSTR; // winuser.h:605
// INVALID_HANDLE_VALUE and the DWLP_* offsets are shared — HANDLE really is `*mut c_void` in
// every island, and win32macros-rust records why DWLP_* must stay computed per target.
pub use win32macros_rust::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, INVALID_HANDLE_VALUE};
// winreg.h:10-12 `#define HKEY_LOCAL_MACHINE ((HKEY)(LONG_PTR)(LONG)0x80000002)`. The LONG cast
// SIGN-EXTENDS on the 64-bit targets, which is why this goes through `as i32 as isize` and not
// a bare `as usize`: 0x80000002 widened unsigned is a different key handle.
pub const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as i32 as isize as HKEY;
// winerror.h `_HRESULT_TYPEDEF_(x)` is `((HRESULT)x)` — a cast macro, so bindgen folds none of
// them. HRESULT is a SIGNED LONG; the `as HRESULT` on each is what makes FAILED() work.
pub const S_OK: HRESULT = 0; // winerror.h:3449
pub const E_NOTIMPL: HRESULT = 0x80004001u32 as HRESULT; // winerror.h:3478
pub const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT; // winerror.h:3479
pub const E_FAIL: HRESULT = 0x80004005u32 as HRESULT; // winerror.h:3482
pub const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT; // winerror.h:4118
pub const OLE_E_ADVISENOTSUPPORTED: HRESULT = 0x80040003u32 as HRESULT; // winerror.h:3728
pub const DV_E_FORMATETC: HRESULT = 0x80040064u32 as HRESULT; // winerror.h:3744
pub const DV_E_LINDEX: HRESULT = 0x80040068u32 as HRESULT; // winerror.h:3748
pub const DV_E_TYMED: HRESULT = 0x80040069u32 as HRESULT; // winerror.h:3749
pub const DV_E_DVASPECT: HRESULT = 0x8004006Bu32 as HRESULT; // winerror.h:3751
pub const DATA_S_SAMEFORMATETC: HRESULT = 0x00040130; // winerror.h:3783
// background.c:313 defines this locally and `#undef`s it three lines later — a shell32 resource
// ordinal the module deliberately does not want in its own header.
pub const IDI_SHELL_NO: WORD = 200;
// commctrl.h:137 `#define NM_CUSTOMDRAW (NM_FIRST-12)` where NM_FIRST is `(0U- 0U)`. bindgen
// emits NM_FIRST but NOT this one — a macro whose body is an EXPRESSION does not always fold —
// so the name was absent from the island and `NM_CUSTOMDRAW =>` in background.rs was a fresh
// BINDING that matched every notification and made all seven arms below it dead code (§9.112,
// the single largest and least visible translation bug class). `profile_derive` had already
// flagged this exact risk as SWITCH_CONST_EXPRESSION: "bindgen may not fold it — verify it
// lands in the island". That finding is a TODO, not a note.
//
// Built from the header's own macro rather than a pasted 0xFFFFFFF4, so it stays checkable:
// the C arithmetic is UNSIGNED (`0U-0U`, then `-12`), which wraps.
pub const NM_CUSTOMDRAW: UINT = NM_FIRST.wrapping_sub(12);
// The rest of that same family, and the reason it needs restating: prsht.h and commctrl.h build
// every notification code by UNSIGNED wraparound (`#define PSN_FIRST (0U - 200U)`), and the
// NMHDR::code field they are compared against is a UINT. bindgen folds the arithmetic and then
// emits the RESULT as a signed `i32` (`PSN_APPLY: i32 = -202`), so the C's one comparison becomes
// a Rust type error — and "fix" it by casting the scrutinee and the arms stop being one family.
//
// DERIVED from the island's own values by reinterpreting the bits, never re-typed: `-202i32 as
// u32` is 0xFFFFFF36, which is exactly what the preprocessor computes. A hand-written 0xFFFFFF36
// would be the §9.112 shape.
pub const PSN_APPLY: UINT = crate::types::PSN_APPLY as UINT;
pub const PSN_KILLACTIVE: UINT = crate::types::PSN_KILLACTIVE as UINT;
pub const PSN_SETACTIVE: UINT = crate::types::PSN_SETACTIVE as UINT;
pub const LVN_ITEMCHANGED: UINT = crate::types::LVN_ITEMCHANGED as UINT;
pub const LVN_DELETEITEM: UINT = crate::types::LVN_DELETEITEM as UINT;
pub const TVN_KEYDOWN: UINT = crate::types::TVN_KEYDOWN as UINT;
// commctrl.h:3163 `#define LVN_BEGINLABELEDIT __MINGW_NAME_AW(LVN_BEGINLABELEDIT)` — the
// PREPROCESSOR picks the half, so bindgen emits only the A and W names. Left unresolved, the
// bare name in a `match` arm is a fresh BINDING that matches everything (§9.112): the
// LVN_BEGINLABELEDIT arm was swallowing every notification the list view sent, and the
// LVN_ENDLABELEDIT arm below it was dead. -DUNICODE, so W.
pub const LVN_BEGINLABELEDIT: UINT = crate::types::LVN_BEGINLABELEDITW as UINT;
pub const LVN_ENDLABELEDIT: UINT = crate::types::LVN_ENDLABELEDITW as UINT;
pub const LVN_KEYDOWN: UINT = crate::types::LVN_KEYDOWN as UINT;
// ChangeDisplaySettingsEx returns a LONG and winuser.h:190-196 spells this family as plain
// signed integers, but bindgen splits it: the two non-negative members come out `u32` and the
// five negative ones `i32`, so a `match` over them cannot type-check at all. One family, one
// type, taken from the function's own return type.
pub const DISP_CHANGE_SUCCESSFUL: LONG = crate::types::DISP_CHANGE_SUCCESSFUL as LONG;
pub const DISP_CHANGE_RESTART: LONG = crate::types::DISP_CHANGE_RESTART as LONG;
pub const DISP_CHANGE_FAILED: LONG = crate::types::DISP_CHANGE_FAILED as LONG;
pub const DISP_CHANGE_BADMODE: LONG = crate::types::DISP_CHANGE_BADMODE as LONG;
pub const DISP_CHANGE_NOTUPDATED: LONG = crate::types::DISP_CHANGE_NOTUPDATED as LONG;
pub const DISP_CHANGE_BADFLAGS: LONG = crate::types::DISP_CHANGE_BADFLAGS as LONG;
pub const DISP_CHANGE_BADPARAM: LONG = crate::types::DISP_CHANGE_BADPARAM as LONG;

// deskcplx.h:7-16 — the clipboard-format NAMES the desk.cpl <-> display-driver shell-extension
// protocol is keyed on. `TEXT("...")` under UNICODE, so they are wide literals rather than
// numbers, and they stay MACROS for the same reason syssetup's SE_SHUTDOWN_NAME does: a
// `*const u16` is not `Sync`, so it cannot be a `static`, and a `const` would need a separate
// backing array per name. Getting one of these strings wrong is not a build error — it is a
// clipboard format that never matches and a settings page that silently shows nothing.
#[macro_export] macro_rules! DESK_EXT_EXTINTERFACE { () => { $crate::w!("Desk.cpl extension interface") }; }
#[macro_export] macro_rules! DESK_EXT_PRUNINGMODE { () => { $crate::w!("Pruning Mode") }; }
#[macro_export] macro_rules! DESK_EXT_DISPLAYDEVICE { () => { $crate::w!("Display Device") }; }
#[macro_export] macro_rules! DESK_EXT_DISPLAYNAME { () => { $crate::w!("Display Name") }; }
#[macro_export] macro_rules! DESK_EXT_DISPLAYID { () => { $crate::w!("Display ID") }; }
#[macro_export] macro_rules! DESK_EXT_DISPLAYKEY { () => { $crate::w!("Display Key") }; }
#[macro_export] macro_rules! DESK_EXT_DISPLAYSTATEFLAGS { () => { $crate::w!("Display State Flags") }; }
#[macro_export] macro_rules! DESK_EXT_MONITORNAME { () => { $crate::w!("Monitor Name") }; }
#[macro_export] macro_rules! DESK_EXT_MONITORDEVICE { () => { $crate::w!("Monitor Device") }; }
pub use crate::{DESK_EXT_DISPLAYDEVICE, DESK_EXT_DISPLAYID, DESK_EXT_DISPLAYKEY, DESK_EXT_DISPLAYNAME, DESK_EXT_DISPLAYSTATEFLAGS, DESK_EXT_EXTINTERFACE, DESK_EXT_MONITORDEVICE, DESK_EXT_MONITORNAME, DESK_EXT_PRUNINGMODE};
// regstr.h:76. In the C this is only ever used in a string-literal CONCATENATION
// (`REGSTR_PATH_CONTROLSFOLDER TEXT("\\Desk")`), which has no Rust spelling at all — so the
// macro takes the suffix and produces the whole path in one `w!`. The translation had rendered
// the concatenation as a COMMA, turning a 4-argument call into a 5-argument one.
#[macro_export]
macro_rules! REGSTR_PATH_CONTROLSFOLDER {
    () => { $crate::w!("Software\\Microsoft\\Windows\\CurrentVersion\\Controls Folder") };
    ($suffix:literal) => {
        $crate::w!(concat!("Software\\Microsoft\\Windows\\CurrentVersion\\Controls Folder", $suffix))
    };
}
pub use crate::REGSTR_PATH_CONTROLSFOLDER;

// dmilib.h's `_ID_STRINGS` enum, which smbios.c uses by BARE MEMBER NAME to index
// `PCHAR DmiStrings[ID_STRINGS_MAX]`. bindgen prefixes every member with the enum's own tag,
// so the C spelling resolves to nothing — and as an ARRAY SUBSCRIPT rather than a `case` label,
// an unresolved one is a hard error rather than the silent catch-all of field manual §9.112.
// Aliased rather than re-typed: the island holds the values, this only restores the names.
pub use crate::types::_ID_STRINGS_BIOS_DATE as BIOS_DATE;
pub use crate::types::_ID_STRINGS_BIOS_VENDOR as BIOS_VENDOR;
pub use crate::types::_ID_STRINGS_BIOS_VERSION as BIOS_VERSION;
pub use crate::types::_ID_STRINGS_BOARD_ASSET_TAG as BOARD_ASSET_TAG;
pub use crate::types::_ID_STRINGS_BOARD_NAME as BOARD_NAME;
pub use crate::types::_ID_STRINGS_BOARD_SERIAL as BOARD_SERIAL;
pub use crate::types::_ID_STRINGS_BOARD_VENDOR as BOARD_VENDOR;
pub use crate::types::_ID_STRINGS_BOARD_VERSION as BOARD_VERSION;
pub use crate::types::_ID_STRINGS_ID_NONE as ID_NONE;
pub use crate::types::_ID_STRINGS_ID_STRINGS_MAX as ID_STRINGS_MAX;
pub use crate::types::_ID_STRINGS_SYS_FAMILY as SYS_FAMILY;
pub use crate::types::_ID_STRINGS_SYS_PRODUCT as SYS_PRODUCT;
pub use crate::types::_ID_STRINGS_SYS_SERIAL as SYS_SERIAL;
pub use crate::types::_ID_STRINGS_SYS_SKU as SYS_SKU;
pub use crate::types::_ID_STRINGS_SYS_VENDOR as SYS_VENDOR;
pub use crate::types::_ID_STRINGS_SYS_VERSION as SYS_VERSION;

// ---- CRT the module reaches that desk's block did not need ----
extern "C" {
    pub fn strstr(h: *const c_char, n: *const c_char) -> *mut c_char;
    pub fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn _stricmp(a: *const c_char, b: *const c_char) -> c_int;
    pub fn wcsstr(h: *const WCHAR, n: *const WCHAR) -> *mut WCHAR;
    pub fn wcschr(s: *const WCHAR, c: WCHAR) -> *mut WCHAR;
    pub fn wcscat(d: *mut WCHAR, s: *const WCHAR) -> *mut WCHAR;
    pub fn towlower(c: c_int) -> c_int;
    pub fn towupper(c: c_int) -> c_int;
}
// The TCHAR spellings, resolved by the module's -DUNICODE exactly as tchar.h resolves them.
pub use self::{wcscat as _tcscat, wcschr as _tcschr, wcsstr as _tcsstr};

// winuser.h:1216 `#define HWND_DESKTOP ((HWND)0)` — a NULL window handle, not an integer.
pub const HWND_DESKTOP: HWND = core::ptr::null_mut();
pub use win32macros_rust::UNICODE_NULL; // ntdef.h `((WCHAR)0)`

// ---- the remaining CRT / TCHAR surface ----
extern "C" {
    pub fn strlen(s: *const c_char) -> usize;
    pub fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;
    pub fn _strupr(s: *mut c_char) -> *mut c_char;
    pub fn wcstoul(s: *const WCHAR, end: *mut *mut WCHAR, base: c_int) -> c_ulong;
    pub fn _wcsupr(s: *mut WCHAR) -> *mut WCHAR;
    pub fn wcsncpy(d: *mut WCHAR, s: *const WCHAR, n: usize) -> *mut WCHAR;
}
pub use self::{
    wcscmp as _tcscmp, wcscpy as _tcscpy, wcsncpy as _tcsncpy, _wcsnicmp as _tcsnicmp,
    _wcsupr as _tcsupr,
};

// windef.h `#define min(a,b) (((a) < (b)) ? (a) : (b))` — a MACRO, so a generic fn would not
// accept the mixed integer types the C hands it.
#[macro_export]
macro_rules! min { ($a:expr, $b:expr $(,)?) => {{ let (x, y) = ($a, $b); if x < y { x } else { y } }}; }
#[macro_export]
macro_rules! max { ($a:expr, $b:expr $(,)?) => {{ let (x, y) = ($a, $b); if x > y { x } else { y } }}; }
pub use crate::{max, min};

// winnt.h:`#define LANG_USER_DEFAULT MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT)` and
// winnls.h `#define LOCALE_USER_DEFAULT MAKELCID(LANG_USER_DEFAULT, SORT_DEFAULT)` — macro
// expressions, so the island carries neither. SUBLANG_DEFAULT is 1, SORT_DEFAULT is 0.
pub const LANG_USER_DEFAULT: WORD = (1 << 10) | 0x00; // MAKELANGID(LANG_NEUTRAL=0x00, 1)
pub const LOCALE_USER_DEFAULT: LCID = LANG_USER_DEFAULT as LCID; // MAKELCID(.., SORT_DEFAULT=0)

// A/W struct aliases the module spells without the suffix, resolved by its -DUNICODE.
pub use crate::types::{
    tagBROWSEINFOW as BROWSEINFO, tagLVITEMW as _LVITEMW,
    tagTVHITTESTINFO as TV_HITTESTINFO, tagTVKEYDOWN as TV_KEYDOWN,
    NMLVDISPINFOW as NMLVDISPINFO,
};

// shlobj_undoc.h's `tagWALK_TREE_CMD` enum, which performance.c passes to
// IRegTreeOptions_WalkTree by bare member name. Same bindgen enum-tag prefix as dmilib's.
pub use crate::types::{
    tagWALK_TREE_CMD_WALK_TREE_DEFAULT as WALK_TREE_DEFAULT,
    tagWALK_TREE_CMD_WALK_TREE_DESTROY as WALK_TREE_DESTROY,
    tagWALK_TREE_CMD_WALK_TREE_REFRESH as WALK_TREE_REFRESH,
    tagWALK_TREE_CMD_WALK_TREE_SAVE as WALK_TREE_SAVE,
};

// ndk/ketypes.h:189 `#define SharedUserData ((KUSER_SHARED_DATA *)USER_SHARED_DATA)` — a MACRO
// casting a fixed VA, so no island carries it. Same spelling kernel32-rust uses.
pub const SharedUserData: *mut KUSER_SHARED_DATA = 0x7FFE_0000 as *mut KUSER_SHARED_DATA;

// commctrl.h:130-131 `#define NM_CLICK (NM_FIRST-2)` / `NM_DBLCLK (NM_FIRST-3)`. bindgen emits
// NM_FIRST but not the names derived from it, and the subtraction wraps in u32 exactly as the
// preprocessor's does -- same UINT family as NM_CUSTOMDRAW above.
pub const NM_CLICK: UINT = NM_FIRST.wrapping_sub(2);
pub const NM_DBLCLK: UINT = NM_FIRST.wrapping_sub(3);

/// strsafe.h `StringCchCopyA` — the NARROW half. `STRSAFEAPI` is `static __inline`, so no
/// object file in the tree defines it, and strsafe-rust ports only the wide entry points.
/// smbios.c:768 is the single caller. Truncates and NUL-terminates exactly as strsafe does,
/// and returns STRSAFE_E_INSUFFICIENT_BUFFER when it had to.
#[inline]
pub unsafe fn StringCchCopyA(pszDest: *mut c_char, cchDest: usize, pszSrc: *const c_char) -> HRESULT {
    if cchDest == 0 {
        return STRSAFE_E_INVALID_PARAMETER;
    }
    let mut i = 0usize;
    while i + 1 < cchDest && *pszSrc.add(i) != 0 {
        *pszDest.add(i) = *pszSrc.add(i);
        i += 1;
    }
    *pszDest.add(i) = 0;
    if *pszSrc.add(i) != 0 {
        return STRSAFE_E_INSUFFICIENT_BUFFER;
    }
    0
}

// general.c:580 `PCWSTR pwszVersion = L" " TEXT(KERNEL_VERSION_RC);` -- a C string-literal
// CONCATENATION whose second half is buildno.h's per-arch build string, so it is not a literal
// in the Rust source and `w!` cannot see it. Widen the island's bytes at compile time instead.
// buildno.h already ends the macro with a `\0`, so the extra element here is C's own implicit
// terminator, and wcslen sees the same length the C does.
#[macro_export]
macro_rules! VERSION_WITH_LEADING_SPACE {
    () => {{
        const SRC: &[u8] = $crate::prelude::KERNEL_VERSION_RC;
        static OUT: [u16; SRC.len() + 1] = {
            let mut out = [0u16; SRC.len() + 1];
            out[0] = b' ' as u16;
            let mut i = 0;
            while i < SRC.len() {
                out[i + 1] = SRC[i] as u16;
                i += 1;
            }
            out
        };
        OUT.as_ptr()
    }};
}
pub use crate::VERSION_WITH_LEADING_SPACE;

// ================= sibling-module functions ===================================================
// precomp.h declares these; each is DEFINED in another of the 12 translated modules, and the C
// reaches them by plain name because a TU-crossing call needs no more than the prototype. In
// Rust the modules are siblings, so the prelude is where the name is made to resolve — exactly
// as desk-rust does it.
pub use crate::sysdm::advanced::AdvancedPageProc;
pub use crate::sysdm::environment::EnvironmentDlgProc;
pub use crate::sysdm::general::{GeneralPageProc, ShowLastWin32Error};
pub use crate::sysdm::hardprof::HardProfDlgProc;
pub use crate::sysdm::hardware::HardwarePageProc;
pub use crate::sysdm::licence::LicenceDlgProc;
pub use crate::sysdm::performance::ShowPerformanceOptions;
pub use crate::sysdm::smbios::GetSystemName;
pub use crate::sysdm::startrec::StartRecDlgProc;
pub use crate::sysdm::userprofile::UserProfileDlgProc;
pub use crate::sysdm::virtmem::VirtMemDlgProc;

// ================= strsafe.h ==================================================================
// The shared crate (sdk/lib/rust/strsafe-rust). `STRSAFEAPI` is `static __inline`, so no object
// file in the tree defines these — see that crate for why they are ported once rather than per
// module. The variadic entry points are trampolines in dll/cpl/sysdm/rust_varargs_shim.c.
pub use strsafe_rust::{
    StringCbCatW, StringCbCopyW, StringCchCatW, StringCchCopyW, StringCchLengthW,
    StringCchVPrintfW, STRSAFE_E_INSUFFICIENT_BUFFER, STRSAFE_E_INVALID_PARAMETER,
};

// ================= commctrl.h / windowsx.h control wrappers ===================================
// Every one is a MACRO around SendMessage, so no island carries them, and each is transcribed
// with the header line. The `LV_ITEM _ms_lvi;` family really does declare a local in the C —
// the block form below is that, not an approximation.
#[macro_export]
macro_rules! ListView_GetItemCount { ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETITEMCOUNT, 0, 0) as c_int }; }
#[macro_export]
// LVM_GETITEM / LVM_EDITLABEL are `#define`d to the W spelling under -DUNICODE, so no island
// carries the undecorated name. The `as *mut _` hop is the C cast `(LPARAM)(LV_ITEM FAR*)`:
// a Rust reference cannot go straight to an integer.
macro_rules! ListView_GetItem { ($h:expr, $p:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETITEMW, 0, $p as *mut _ as LPARAM) as BOOL }; }
#[macro_export]
macro_rules! ListView_DeleteItem { ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_DELETEITEM, $i as WPARAM, 0) as BOOL }; }
#[macro_export]
macro_rules! ListView_DeleteAllItems { ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_DELETEALLITEMS, 0, 0) as BOOL }; }
#[macro_export]
macro_rules! ListView_SetColumnWidth { ($h:expr, $c:expr, $cx:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_SETCOLUMNWIDTH, $c as WPARAM, $crate::MAKELPARAM!($cx, 0)) as BOOL }; }
#[macro_export]
macro_rules! ListView_GetColumnWidth { ($h:expr, $c:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETCOLUMNWIDTH, $c as WPARAM, 0) as c_int }; }
#[macro_export]
macro_rules! ListView_GetNextItem { ($h:expr, $i:expr, $f:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETNEXTITEM, $i as WPARAM, $crate::MAKELPARAM!($f, 0)) as c_int }; }
#[macro_export]
macro_rules! ListView_GetSelectedCount { ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETSELECTEDCOUNT, 0, 0) as UINT }; }
#[macro_export]
macro_rules! ListView_EditLabel { ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_EDITLABELW, $i as WPARAM, 0) as HWND }; }
#[macro_export]
macro_rules! ListView_GetEditControl { ($h:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_GETEDITCONTROL, 0, 0) as HWND }; }
#[macro_export]
macro_rules! ListView_Update { ($h:expr, $i:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_UPDATE, $i as WPARAM, 0) as BOOL }; }
#[macro_export]
macro_rules! ListView_SetExtendedListViewStyle { ($h:expr, $dw:expr $(,)?) => { $crate::prelude::SendMessageW($h, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, $dw as LPARAM) as DWORD }; }
// commctrl.h:2689/2696 — these two declare a local LV_ITEM in the macro body.
#[macro_export]
macro_rules! ListView_GetItemText {
    ($h:expr, $i:expr, $sub:expr, $psz:expr, $cch:expr $(,)?) => {{
        let mut _ms_lvi: LV_ITEM = core::mem::zeroed();
        _ms_lvi.iSubItem = $sub as c_int;
        _ms_lvi.cchTextMax = $cch as c_int;
        _ms_lvi.pszText = $psz;
        $crate::prelude::SendMessageW($h, LVM_GETITEMTEXTW, $i as WPARAM, core::ptr::addr_of_mut!(_ms_lvi) as LPARAM)
    }};
}
#[macro_export]
macro_rules! ListView_SetItemText {
    ($h:expr, $i:expr, $sub:expr, $psz:expr $(,)?) => {{
        let mut _ms_lvi: LV_ITEM = core::mem::zeroed();
        _ms_lvi.iSubItem = $sub as c_int;
        _ms_lvi.pszText = $psz;
        $crate::prelude::SendMessageW($h, LVM_SETITEMTEXTW, $i as WPARAM, core::ptr::addr_of_mut!(_ms_lvi) as LPARAM)
    }};
}
#[macro_export]
macro_rules! TreeView_GetNextItem { ($h:expr, $i:expr, $c:expr $(,)?) => { $crate::prelude::SendMessageW($h, TVM_GETNEXTITEM, $c as WPARAM, $i as LPARAM) as HTREEITEM }; }
#[macro_export]
macro_rules! TreeView_GetSelection { ($h:expr $(,)?) => { $crate::TreeView_GetNextItem!($h, core::ptr::null_mut::<core::ffi::c_void>(), TVGN_CARET) }; }
#[macro_export]
macro_rules! TreeView_HitTest { ($h:expr, $p:expr $(,)?) => { $crate::prelude::SendMessageW($h, TVM_HITTEST, 0, $p as *mut _ as LPARAM) as HTREEITEM }; }
#[macro_export]
macro_rules! Button_SetCheck { ($h:expr, $c:expr $(,)?) => { { $crate::prelude::SendMessageW($h, BM_SETCHECK, $c as WPARAM, 0); } }; }
pub use crate::{Button_SetCheck, ListView_DeleteAllItems, ListView_DeleteItem, ListView_EditLabel, ListView_GetColumnWidth, ListView_GetEditControl, ListView_GetItem, ListView_GetItemCount, ListView_GetItemText, ListView_GetNextItem, ListView_GetSelectedCount, ListView_SetColumnWidth, ListView_SetExtendedListViewStyle, ListView_SetItemText, ListView_Update, TreeView_GetNextItem, TreeView_GetSelection, TreeView_HitTest};

// sysdm.c's own variadic. Stable Rust cannot DEFINE one, so sysdm.rs RECEIVES it the va-slots
// way (fixed trailing slot parameters); this declaration is how the Rust callers reach it.
// `extern "C"`, not "system", even though the C says `__cdecl` explicitly: no ABI on any of our
// targets has a variadic stdcall, so the two agree.
extern "C" {
    pub fn ResourceMessageBox(
        hInstance: HINSTANCE,
        hwnd: HWND,
        uType: UINT,
        uCaption: UINT,
        uText: UINT,
        ...
    ) -> INT;
}

// The A/W pairs desk's block did not carry. sysdm compiles -DUNICODE, so every unsuffixed
// spelling in the C is the W entry point — the same rule winuser.h/winbase.h apply with their
// own `#define X XW`. Aliased, never re-declared: a second `extern` for a symbol the island
// already has is where a pointer level goes missing silently.
pub use crate::types::{
    ConvertStringSidToSidW as ConvertStringSidToSid,
    FormatMessageW as FormatMessage,
    GetDiskFreeSpaceExW as GetDiskFreeSpaceEx,
    GetDriveTypeW as GetDriveType,
    GetLocaleInfoW as GetLocaleInfo,
    GetLogicalDriveStringsW as GetLogicalDriveStrings,
    GetVolumeInformationW as GetVolumeInformation,
    GetWindowTextW as GetWindowText,
    LoadIconW as LoadIcon,
    LoadLibraryW as LoadLibrary,
    MoveFileExW as MoveFileEx,
    RegEnumValueW as RegEnumValue,
    RegQueryInfoKeyW as RegQueryInfoKey,
    SHBrowseForFolderW as SHBrowseForFolder,
    SHGetPathFromIDListW as SHGetPathFromIDList,
    ShellExecuteW as ShellExecute,
    ShellExecuteExW as ShellExecuteEx,
};
// strsafe.h's TCHAR spellings, from the shared crate.
pub use strsafe_rust::{StringCchCatW as StringCchCat, StringCchCopyW as StringCchCopy};
// strsafe.h's printf entry points are VARIADIC, and stable Rust can CALL a C-variadic though
// it cannot define one; so these are strsafe-rust macros that call msvcrt's `_snwprintf` in
// place, with strsafe's bounds and HRESULTs. The C trampoline that used to define them
// (rust_varargs_shim.c) retired 2026-09-25. Call sites say `StringCchPrintfW!(...)`.
pub use strsafe_rust::{StringCchPrintfW, StringCchPrintfW as StringCchPrintf};

// ================= COM (COBJMACROS) ===========================================================
// performance.c drives shell32's IRegTreeOptions through objbase.h's COBJMACROS, which expand to
// `(This)->lpVtbl->Method(This, ..)`. bindgen emits the vtbl but not the macros, so each call is
// a shim here — same shape desk-rust uses for IDataObject. `unwrap_unchecked` and not `unwrap`:
// a None slot in a vtable a COM server just handed us is not a recoverable condition, and
// `unwrap` would put a panic path in a no_std staticlib (scan_silent_failures flags it).
#[inline(always)]
pub unsafe fn IRegTreeOptions_InitTree<T>(
    p: *mut T, hTV: HWND, hKey: HKEY, SubKey: LPCSTR, pUnknown: *const c_char,
) -> HRESULT {
    let vtbl = *(p as *mut *const IRegTreeOptionsVtbl);
    ((*vtbl).InitTree.unwrap_unchecked())(p as *mut IRegTreeOptions, hTV, hKey, SubKey, pUnknown)
}
#[inline(always)]
pub unsafe fn IRegTreeOptions_WalkTree<T>(p: *mut T, cmd: WALK_TREE_CMD) -> HRESULT {
    let vtbl = *(p as *mut *const IRegTreeOptionsVtbl);
    ((*vtbl).WalkTree.unwrap_unchecked())(p as *mut IRegTreeOptions, cmd)
}
#[inline(always)]
pub unsafe fn IRegTreeOptions_ToggleItem<T>(p: *mut T, hTI: HTREEITEM) -> HRESULT {
    let vtbl = *(p as *mut *const IRegTreeOptionsVtbl);
    ((*vtbl).ToggleItem.unwrap_unchecked())(p as *mut IRegTreeOptions, hTI)
}
#[inline(always)]
pub unsafe fn IRegTreeOptions_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IRegTreeOptionsVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IRegTreeOptions)
}

// ---- imports the island did not carry ----
// ole32 (objbase.h declares them, but performance.c is the only caller and the allowlist that
// brought IRegTreeOptions in did not reach these loose prototypes) and shell32's undocumented
// broadcast helper.
extern "system" {
    pub fn CoTaskMemFree(pv: LPVOID);
    pub fn CoInitialize(pvReserved: LPVOID) -> HRESULT;
    pub fn CoCreateInstance(
        rclsid: *const IID,
        pUnkOuter: *mut c_void,
        dwClsContext: DWORD,
        riid: *const IID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CoUninitialize();
    pub fn SHSendMessageBroadcastW(uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}
// wtypesbase.h CLSCTX_INPROC_SERVER — an enum MEMBER, and bindgen does not emit the bare name
// for one, so the island has the type but not this. Value is fixed by the COM ABI.
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1;

// winuser.h `#define DialogBox(i,t,p,f) DialogBoxParam(i,t,p,f,0)` and prsht.h
// `#define PropSheet_HwndToIndex(hDlg,hwnd) SNDMSG(hDlg,PSM_HWNDTOINDEX,(WPARAM)hwnd,0)`.
#[macro_export]
macro_rules! DialogBoxW {
    ($inst:expr, $tmpl:expr, $parent:expr, $proc:expr $(,)?) => {
        $crate::prelude::DialogBoxParamW($inst, $tmpl, $parent, $proc, 0)
    };
}
#[macro_export]
macro_rules! PropSheet_HwndToIndex {
    ($hdlg:expr, $hwnd:expr $(,)?) => {
        $crate::prelude::SendMessageW($hdlg, PSM_HWNDTOINDEX, $hwnd as WPARAM, 0) as c_int
    };
}
pub use crate::{DialogBoxW, PropSheet_HwndToIndex};

// ================= header macros ==============================================================
// C macros never reach the island. All stay `macro_rules!` and never become out-of-line fns: an
// out-of-line forwarder is an optimization barrier that uncloaks latent ABI mismatches (§9.65).
//
// The word/byte/colour/coordinate family lives in `win32macros-rust`, ported once from the
// headers and known-answer tested there. Re-exported here under the same names, so no call site
// changed.
pub use win32macros_rust::{
    ARRAYSIZE, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIWORD, LOWORD,
    MAKEINTRESOURCEW, MAKELONG, RGB, _countof,
};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because sysdm compiles -DUNICODE. That is
// this module's property, not the shared crate's, so the alias stays here.
#[macro_export]
macro_rules! MAKEINTRESOURCE { ($i:expr $(,)?) => { $crate::prelude::MAKEINTRESOURCEW!($i) }; }
pub use crate::MAKEINTRESOURCE;
// winuser.h:4110-4111. These stay module-local because LPARAM/WPARAM are pointer-width island
// aliases (c_long/c_uint on i386, c_longlong/c_ulonglong on the 64-bit pair) — NOT isize/usize,
// which are distinct primitives and would not unify. Only the MAKELONG inside them is shared.
#[macro_export]
macro_rules! MAKELPARAM { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as LPARAM }; }
pub use crate::MAKELPARAM;
#[macro_export]
macro_rules! MAKEWPARAM { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as WPARAM }; }
pub use crate::MAKEWPARAM;
// basetsd.h:83 `#define IntToPtr(i) ((VOID*)(INT_PTR)((int)i))` — the `(int)` cast is load-
// bearing: it TRUNCATES to 32 bits and then sign-extends, which is what the SystemParametersInfo
// callers depend on. Kept as a macro because the C macro is untyped and the call sites pass both
// BOOL and UINT.
#[macro_export]
macro_rules! IntToPtr { ($i:expr $(,)?) => { (($i as c_int) as isize) as PVOID }; }
pub use crate::IntToPtr;
// winnt.h `#define FIELD_OFFSET(t, f) ((LONG)(LONG_PTR)&(((t*)0)->f))`. A macro because it takes
// a TYPE and a FIELD NAME; `offset_of!` is the stable spelling of exactly this since 1.77.
#[macro_export]
macro_rules! FIELD_OFFSET { ($t:ty, $f:ident $(,)?) => { core::mem::offset_of!($t, $f) as ULONG_PTR }; }
pub use crate::FIELD_OFFSET;
// guiddef.h:147 `#define IsEqualGUID(a, b) (!memcmp(a, b, sizeof(GUID)))`. Returns C's int, so
// the call sites' `!= 0` reads the same either way.
#[macro_export]
macro_rules! IsEqualGUID {
    ($a:expr, $b:expr $(,)?) => {
        ((unsafe { $crate::prelude::memcmp($a as *const _ as *const core::ffi::c_void, $b as *const _ as *const core::ffi::c_void, core::mem::size_of::<GUID>()) } == 0) as c_int)
    };
}
pub use crate::IsEqualGUID;
// winbase.h `#define ZeroMemory RtlZeroMemory` / `#define CopyMemory RtlCopyMemory`. NEVER a
// hand-rolled byte loop: LLVM folds one back into a call to memset/memcpy itself → infinite
// self-recursion → silent hang (AGENTS.md). The zeroing pair is shared; CopyMemory below stays
// here because it reaches this crate's `memcpy` declaration.
pub use win32macros_rust::{RtlZeroMemory, ZeroMemory};
#[macro_export]
macro_rules! CopyMemory {
    ($d:expr, $s:expr, $l:expr $(,)?) => { { $crate::prelude::memcpy($d as *mut _ as *mut core::ffi::c_void, $s as *const _ as *const core::ffi::c_void, $l as usize); } };
}
pub use crate::CopyMemory;
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER { ($p:expr $(,)?) => { let _ = &$p; }; }
pub use crate::UNREFERENCED_PARAMETER;
// windowsx.h:518 — the twelve messages whose DIALOG return value is the value ITSELF rather than
// a DWLP_MSGRESULT store. Transcribed as the header's own list, in the header's own order.
#[macro_export]
macro_rules! SetDlgMsgResult {
    ($hwnd:expr, $msg:expr, $result:expr $(,)?) => {{
        let __m = $msg as UINT;
        if __m == WM_CTLCOLORMSGBOX
            || __m == WM_CTLCOLOREDIT
            || __m == WM_CTLCOLORLISTBOX
            || __m == WM_CTLCOLORBTN
            || __m == WM_CTLCOLORDLG
            || __m == WM_CTLCOLORSCROLLBAR
            || __m == WM_CTLCOLORSTATIC
            || __m == WM_COMPAREITEM
            || __m == WM_VKEYTOITEM
            || __m == WM_CHARTOITEM
            || __m == WM_QUERYDRAGICON
            || __m == WM_INITDIALOG
        {
            $result as INT_PTR
        } else {
            $crate::prelude::SetWindowLongPtr($hwnd, DWLP_MSGRESULT, $result as LONG_PTR);
            TRUE as INT_PTR
        }
    }};
}
pub use crate::SetDlgMsgResult;
// winuser.h `#define DialogBox(i,t,p,f) DialogBoxParamW(i,t,p,f,0)` under UNICODE.
#[macro_export]
macro_rules! DialogBox {
    ($inst:expr, $tmpl:expr, $parent:expr, $proc:expr $(,)?) => {
        $crate::prelude::DialogBoxParamW($inst, $tmpl, $parent, $proc, 0)
    };
}
pub use crate::DialogBox;
// commctrl.h / prsht.h control wrappers — each is a SendMessage in a trenchcoat. `SNDMSG` is the
// header's own alias for SendMessage, and under UNICODE each LVM_* resolves to its W twin.
#[macro_export]
macro_rules! ListView_GetImageList {
    ($hwnd:expr, $kind:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_GETIMAGELIST, $kind as WPARAM, 0) as HIMAGELIST };
}
pub use crate::ListView_GetImageList;
#[macro_export]
macro_rules! ListView_SetImageList {
    ($hwnd:expr, $himl:expr, $kind:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_SETIMAGELIST, $kind as WPARAM, $himl as LPARAM) as HIMAGELIST };
}
pub use crate::ListView_SetImageList;
#[macro_export]
macro_rules! ListView_InsertItem {
    ($hwnd:expr, $item:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_INSERTITEMW, 0, $item as *const _ as *const core::ffi::c_void as LPARAM) as c_int };
}
pub use crate::ListView_InsertItem;
#[macro_export]
macro_rules! ListView_InsertColumn {
    ($hwnd:expr, $col:expr, $pcol:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_INSERTCOLUMNW, $col as c_int as WPARAM, $pcol as *const _ as *const core::ffi::c_void as LPARAM) as c_int };
}
pub use crate::ListView_InsertColumn;
#[macro_export]
macro_rules! ListView_FindItem {
    ($hwnd:expr, $start:expr, $plvfi:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_FINDITEMW, $start as c_int as WPARAM, $plvfi as *const _ as *const core::ffi::c_void as LPARAM) as c_int };
}
pub use crate::ListView_FindItem;
// commctrl.h:2678 — a STATEMENT macro that builds its own LV_ITEM on the stack. The `state` and
// `stateMask` fields are the only two it fills; every other field of the temporary is
// indeterminate in the C and unread by the message, so zeroing the rest is faithful and safer.
#[macro_export]
macro_rules! ListView_SetItemState {
    ($hwnd:expr, $i:expr, $data:expr, $mask:expr $(,)?) => {{
        let mut __ms_lvi: LVITEMW = core::mem::zeroed();
        __ms_lvi.stateMask = $mask as UINT;
        __ms_lvi.state = $data as UINT;
        $crate::prelude::SendMessageW($hwnd, LVM_SETITEMSTATE, $i as WPARAM, core::ptr::addr_of_mut!(__ms_lvi) as LPARAM);
    }};
}
pub use crate::ListView_SetItemState;
// commctrl.h:415 `#define ImageList_AddIcon(himl,hicon) ImageList_ReplaceIcon(himl,-1,hicon)`
#[macro_export]
macro_rules! ImageList_AddIcon {
    ($himl:expr, $hicon:expr $(,)?) => { $crate::prelude::ImageList_ReplaceIcon($himl, -1, $hicon) };
}
pub use crate::ImageList_AddIcon;
// prsht.h:344 `#define PropSheet_Changed(d,w) SendMessage(d,PSM_CHANGED,(WPARAM)w,0)`
#[macro_export]
macro_rules! PropSheet_Changed {
    ($hdlg:expr, $hpage:expr $(,)?) => { $crate::prelude::SendMessageW($hdlg, PSM_CHANGED, $hpage as WPARAM, 0) };
}
pub use crate::PropSheet_Changed;
// winerror.h `#define FAILED(hr) (((HRESULT)(hr)) < 0)`. The comparison is on the SIGNED
// HRESULT: an intervening `as u32` makes it unconditionally false and deletes the failure branch.
#[macro_export]
macro_rules! FAILED { ($hr:expr $(,)?) => { (($hr as HRESULT) < 0) }; }
pub use crate::FAILED;
#[macro_export]
macro_rules! SUCCEEDED { ($hr:expr $(,)?) => { (($hr as HRESULT) >= 0) }; }
pub use crate::SUCCEEDED;

// winnt.h `#define TEXT(x) L##x` under UNICODE, and tchar.h's `_T` is the same macro. By the time
// the translator emits a call, the argument is ALREADY the wide literal (`w!("…")`), so what is
// left of the macro is the identity. A `const fn` and not a `macro_rules!` because the call sites
// spell it as a call; `#[inline(always)] const` cannot become the out-of-line no-op §9.65 warns
// about — it has a value to return and folds at compile time.
#[inline(always)]
pub const fn TEXT<T: Copy>(s: T) -> T {
    s
}
#[inline(always)]
pub const fn _T<T: Copy>(s: T) -> T {
    s
}

// ================= interlocked ================================================================
// winnt.h's InterlockedIncrement/Decrement are intrinsics under MSVC and `static __inline` here,
// so no object file defines them. Rust's own atomics are the portable spelling of the same
// LOCK-prefixed read-modify-write, and they return the NEW value exactly as the Win32 API does.
#[inline(always)]
pub unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline(always)]
pub unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1
}
