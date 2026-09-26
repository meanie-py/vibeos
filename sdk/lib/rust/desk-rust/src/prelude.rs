//! desk-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// `NDEBUG` silences DPRINT only. DPRINT1 sits OUTSIDE that guard in
// sdk/include/reactos/debug.h, so it is live whenever DBG is set — and our media is
// built -DCMAKE_BUILD_TYPE=Debug, i.e. -DDBG=1. Both stay MACROS, never out-of-line
// fns (field manual §9.65: a no-op fn is an optimization barrier). The C passes
// __RELFILE__/__LINE__ as two extra varargs; file!()/line!() fold into the literal.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
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
    LoadCursorW as LoadCursor, LoadLibraryExW as LoadLibraryEx, LoadMenuW as LoadMenu,
    LoadStringW as LoadString, MessageBoxW as MessageBox, PathCombineW as PathCombine,
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
pub use crate::types::{
    ImageLockMode_ImageLockModeRead as ImageLockModeRead,
    ImageLockMode_ImageLockModeUserInputBuf as ImageLockModeUserInputBuf,
    SHGFP_TYPE_SHGFP_TYPE_DEFAULT as SHGFP_TYPE_DEFAULT,
    tagCOINIT_COINIT_APARTMENTTHREADED as COINIT_APARTMENTTHREADED,
    tagDATADIR_DATADIR_GET as DATADIR_GET, tagDVASPECT_DVASPECT_CONTENT as DVASPECT_CONTENT,
    tagLOGFONTW as LOGFONTW_STRUCT, tagTYMED_TYMED_HGLOBAL as TYMED_HGLOBAL,
};
// vsstyle.h's theme part/state enums, same prefixing rule.
pub use crate::types::{
    BUTTONPARTS_BP_PUSHBUTTON as BP_PUSHBUTTON, PUSHBUTTONSTATES_PBS_NORMAL as PBS_NORMAL,
};

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
pub use crate::types::Status_Ok as Ok;
pub const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFFFFFF; // winuser.h:179 `((DWORD)-1)`
// winuser.h:1215 `((HWND)0xffff)`. Stays here: HWND is `*mut HWND__`, a nominal opaque struct
// this island emits for itself, so it cannot come from a shared crate.
pub const HWND_BROADCAST: HWND = 0xffffusize as HWND;
// INVALID_HANDLE_VALUE and the DWLP_* offsets are shared — HANDLE really is `*mut c_void` in
// every island, and win32macros-rust records why DWLP_* must stay computed per target.
pub use win32macros_rust::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, INVALID_HANDLE_VALUE};
// winuser.h `#define IDC_ARROW MAKEINTRESOURCE(32512)` / `#define RT_STRING MAKEINTRESOURCE(6)`
// — a resource ORDINAL smuggled through a pointer, which is why the type is LPCWSTR and not an
// integer: every call site hands it straight to LoadCursorW / FindResourceW.
pub const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR; // winuser.h:695
pub const RT_STRING: LPCWSTR = 6usize as LPCWSTR; // winuser.h:605
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

// ================= header macros ==============================================================
// C macros never reach the island. All stay `macro_rules!` and never become out-of-line fns: an
// out-of-line forwarder is an optimization barrier that uncloaks latent ABI mismatches (§9.65).
//
// The word/byte/colour/locale family lives in `win32macros-rust`, ported once from the headers
// and known-answer tested there. Re-exported through this prelude under the same names, so no
// call site changed.
pub use win32macros_rust::{
    ARRAYSIZE, GetBValue, GetGValue, GetRValue, HIWORD, LOWORD, MAKEINTRESOURCEW, MAKELONG, RGB,
    _countof,
};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because desk compiles -DUNICODE. That is
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

// ================= COM ========================================================================
// objbase.h's `#define IFace_Method(This, ..) (This)->lpVtbl->Method(This, ..)`. Generic over the
// concrete implementation type so a `CDevSettings*` can be handed straight in, the same shape
// netcfgx-rust uses. Each goes through the island's vtable struct, so the slot ORDER is
// bindgen's and never hand-counted.
pub type REFIID = *const IID;
#[inline(always)]
pub unsafe fn IDataObject_QueryGetData<T>(p: *mut T, pformatetc: *mut FORMATETC) -> HRESULT {
    let vtbl = *(p as *mut *const IDataObjectVtbl);
    ((*vtbl).QueryGetData.unwrap_unchecked())(p as *mut IDataObject, pformatetc)
}
#[inline(always)]
pub unsafe fn IDataObject_GetData<T>(p: *mut T, pformatetcIn: *mut FORMATETC, pmedium: *mut STGMEDIUM) -> HRESULT {
    let vtbl = *(p as *mut *const IDataObjectVtbl);
    ((*vtbl).GetData.unwrap_unchecked())(p as *mut IDataObject, pformatetcIn, pmedium)
}
#[inline(always)]
pub unsafe fn IDataObject_Release<T>(p: *mut T) -> ULONG {
    let vtbl = *(p as *mut *const IDataObjectVtbl);
    ((*vtbl).Release.unwrap_unchecked())(p as *mut IDataObject)
}
// devsett.c:1500 `#define impl_to_interface(impl,iface) (struct iface *)(&(impl)->lp##iface##Vtbl)`
// — a token-pasting macro, so the interface NAME is spelled out here rather than parameterised.
// The cast is on the ADDRESS OF THE VTABLE FIELD, not the object: that field is the interface,
// and the C's whole COM layout depends on it being first.
#[macro_export]
macro_rules! impl_to_interface {
    ($imp:expr, IDataObject $(,)?) => {
        core::ptr::addr_of_mut!((*$imp).lpIDataObjectVtbl) as *mut IDataObject
    };
}
pub use crate::impl_to_interface;

// ================= sibling-module functions ===================================================
// The C reaches these through desk.h / draw.h / preview.h / monslctl.h; in Rust they are items of
// another module of this same crate, so they are re-exported here rather than declared `extern`.
// An extern DECLARATION for something the crate itself defines is the drift shape AGENTS.md warns
// about — it would link against our own symbol with a signature nothing checks.
pub use crate::desk::advappdlg::AdvAppearanceDlgProc;
pub use crate::desk::advmon::DisplayAdvancedSettings;
pub use crate::desk::appearance::AppearancePageProc;
pub use crate::desk::background::BackgroundPageProc;
pub use crate::desk::desk::LoadPopupMenu;
pub use crate::desk::deskcplx::QueryDeskCplString;
pub use crate::desk::desktop::{
    DesktopPageProc, InitDesktopSettings, SaveDesktopSettings, SetDesktopSettings,
};
pub use crate::desk::devsett::CreateDevSettings;
pub use crate::desk::dibitmap::{DibFreeImage, DibLoadImage};
pub use crate::desk::draw::{
    MyDrawCaptionButtons, MyDrawCaptionTemp, MyDrawEdge, MyDrawFrameControl, MyDrawMenuBarTemp,
    MyDrawScrollbar,
};
pub use crate::desk::effappdlg::EffAppearanceDlgProc;
pub use crate::desk::general::AdvGeneralPageProc;
pub use crate::desk::misc::{AllocAndLoadString, ClrBtn_CustomDraw};
pub use crate::desk::monslctl::{
    RegisterMonitorSelectionControl, UnregisterMonitorSelectionControl,
};
pub use crate::desk::muireg::RegLoadMUIStringW;
pub use crate::desk::preview::{RegisterPreviewControl, UnregisterPreviewControl};
pub use crate::desk::screensaver::ScreenSaverPageProc;
pub use crate::desk::settings::{SettingsPageCallbackProc, SettingsPageProc, SwitchDisplayMode};
pub use crate::desk::strsafe::{
    StringCbCatW, StringCbCopyW, StringCchCatW, StringCchCopyW, StringCchLengthW,
};
pub use crate::desk::theme::{
    ActivateTheme, ActivateThemeFile, ApplyScheme, DrawThemePreview, FindOrAppendTheme,
    GetActiveClassicTheme, GetActiveTheme, LoadCurrentScheme, LoadSchemeFromReg,
    LoadSchemeFromTheme, LoadThemes, SchemeGetFont, SchemeGetMetric, SchemeSetMetric,
};
// The TCHAR half of strsafe, same preprocessor rule as the CRT block above.
pub use crate::desk::strsafe::{
    StringCbCatW as StringCbCat, StringCbCopyW as StringCbCopy, StringCchCatW as StringCchCat,
    StringCchCopyW as StringCchCopy, StringCchLengthW as StringCchLength,
};

// ================= imports from other DLLs ====================================================
// shell32's rundll32 entry point. Not in the island because desk reaches it by a
// GetProcAddress-shaped declaration in its own .c rather than through a public header.
extern "system" {
    pub fn Control_RunDLLW(hWnd: HWND, hInst: HINSTANCE, cmd: LPCWSTR, nCmdShow: c_int);
    // ole32. Declared in objidl.h as a loose prototype rather than an interface method, so the
    // allowlist that brought IDataObject in did not bring this.
    pub fn ReleaseStgMedium(pmedium: *mut STGMEDIUM);
}
// strsafe.h's printf entry points are VARIADIC, and stable Rust can CALL a C-variadic though
// it cannot define one; so these are strsafe-rust macros that call msvcrt's `_snwprintf` in
// place, with strsafe's bounds and HRESULTs. The C trampoline that used to define them
// (rust_varargs_shim.c) retired 2026-09-25. Call sites say `StringCchPrintfW!(...)`.
pub use strsafe_rust::{
    StringCbPrintfExW, StringCbPrintfExW as StringCbPrintfEx, StringCbPrintfW,
    StringCbPrintfW as StringCbPrintf, StringCchPrintfW, StringCchPrintfW as StringCchPrintf,
};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;
