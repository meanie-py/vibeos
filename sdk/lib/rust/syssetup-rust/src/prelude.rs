//! syssetup-rust prelude — the single import surface the translated modules use.
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

// The rest of the CRT surface syssetup uses. bindgen does not emit these because the .c reach
// them through the CRT headers, not the SDK ones in the island's allowlist. Declared, never
// implemented — msvcrt exports every one.
extern "C" {
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcsncpy(dst: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcsncat(dst: *mut u16, src: *const u16, n: usize) -> *mut u16;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcstoul(s: *const u16, end: *mut *mut u16, base: c_int) -> u32;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
    pub fn strcat(dst: *mut c_char, src: *const c_char) -> *mut c_char;
    // The `v*` printf family: variadic only in their va_list argument, which is a plain pointer
    // on all three targets, so these ARE expressible in stable Rust — unlike the `...` entry
    // points, which live in rust_varargs_shim.c.
    pub fn vsprintf(buf: *mut c_char, fmt: *const c_char, ap: *mut c_char) -> c_int;
    pub fn _vsnwprintf(buf: *mut u16, n: usize, fmt: *const u16, ap: *mut c_char) -> c_int;
    pub fn _snwprintf(buf: *mut u16, n: usize, fmt: *const u16, ...) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
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
// somebody counted. wizard.c's six registry-path statics arrived with five of the six lengths
// wrong.
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
// A C file-scope `static const` TABLE OF POINTERS (wizard.c's sound-event and theme tables) has
// no direct Rust spelling: a raw pointer is not `Sync`, so an array of them cannot be a
// `static`. This newtype promises exactly what the C already promises — the table is read-only
// and every pointer in it is `'static` (each one comes from `w!`, i.e. another static). `Deref`
// keeps the call sites spelled as the C spells them: `Themes[n].ThemeFile`, `table.len()`.
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

// setupapi's own default copy-queue callback, which install.c and netinstall.c hand straight to
// SetupInstallFilesFromInfSectionW as a PSP_FILE_CALLBACK_W. The island declares it in the
// ARCH's literal convention (`extern "stdcall"` on i386), and Rust compares fn-pointer ABIs
// NOMINALLY, so that fn item does not fit a slot normalize_island_abi spelled `system` — even
// though the two ARE the same ABI there. Re-declared once with the portable spelling: same
// symbol, same ABI, and this item shadows the glob-imported one from `crate::types::*`.
extern "system" {
    pub fn SetupDefaultQueueCallbackW(
        Context: PVOID,
        Notification: UINT,
        Param1: UINT_PTR,
        Param2: UINT_PTR,
    ) -> UINT;
}

// ================= UNICODE A/W aliases ========================================================
// syssetup compiles with -DUNICODE, so `#define SendMessage SendMessageW` and its siblings are
// resolved by the PREPROCESSOR and never reach the island. DERIVED, not typed: each name below
// was produced by taking the island's `<N>W` for every unresolved `<N>` the compiler named, so a
// pair cannot be mismatched. Aliases and not re-declarations — a hand-written prototype for
// SendMessageW is the exact shape that links and then corrupts.
pub use crate::types::{
    ConvertStringSidToSidW as ConvertStringSidToSid, CreateDialogParamW as CreateDialogParam,
    CreateDirectoryW as CreateDirectory, CreatePropertySheetPageW as CreatePropertySheetPage,
    DispatchMessageW as DispatchMessage, GetMessageW as GetMessage, GetObjectW as GetObject,
    IsDialogMessageW as IsDialogMessage, LoadIconW as LoadIcon,
    LookupPrivilegeValueW as LookupPrivilegeValue, PathAddBackslashW as PathAddBackslash,
    PostMessageW as PostMessage, PostThreadMessageW as PostThreadMessage,
    PropertySheetW as PropertySheet, RegSetValueExW as RegSetValueEx,
    SendDlgItemMessageW as SendDlgItemMessage, SendMessageW as SendMessage,
    SetupDefaultQueueCallbackW as SetupDefaultQueueCallback,
    SetupFindFirstLineW as SetupFindFirstLine, SetupGetMultiSzFieldW as SetupGetMultiSzField,
    SetupGetStringFieldW as SetupGetStringField, StartServiceW as StartService,
    wsprintfW as wsprintf,
};
pub use crate::types::{
    LPPROPSHEETPAGEW as LPPROPSHEETPAGE, LVITEMW as LVITEM, PROPSHEETHEADERW as PROPSHEETHEADER,
    PROPSHEETPAGEW as PROPSHEETPAGE,
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
// `PolicyAccountDomainInformation` is emitted as
// `_POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation`. Aliased rather than re-declared
// with a hand-typed value: a second `const` that is merely plausible is exactly how a `match`
// arm ends up comparing against the wrong number (§9.112 / task #38). Every line below was
// derived by grepping the island for the suffix, not typed.
pub use crate::types::{
    _DOMAIN_INFORMATION_CLASS_DomainLockoutInformation as DomainLockoutInformation,
    _DOMAIN_INFORMATION_CLASS_DomainNameInformation as DomainNameInformation,
    _DOMAIN_INFORMATION_CLASS_DomainPasswordInformation as DomainPasswordInformation,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryAccountLogon as AuditCategoryAccountLogon,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryAccountManagement as AuditCategoryAccountManagement,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryDetailedTracking as AuditCategoryDetailedTracking,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryDirectoryServiceAccess as AuditCategoryDirectoryServiceAccess,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryLogon as AuditCategoryLogon,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryObjectAccess as AuditCategoryObjectAccess,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryPolicyChange as AuditCategoryPolicyChange,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategoryPrivilegeUse as AuditCategoryPrivilegeUse,
    _POLICY_AUDIT_EVENT_TYPE_AuditCategorySystem as AuditCategorySystem,
    _POLICY_INFORMATION_CLASS_PolicyAccountDomainInformation as PolicyAccountDomainInformation,
    _POLICY_INFORMATION_CLASS_PolicyAuditEventsInformation as PolicyAuditEventsInformation,
    _POLICY_INFORMATION_CLASS_PolicyPrimaryDomainInformation as PolicyPrimaryDomainInformation,
    _USER_INFORMATION_CLASS_UserAccountNameInformation as UserAccountNameInformation,
    _USER_INFORMATION_CLASS_UserControlInformation as UserControlInformation,
    _USER_INFORMATION_CLASS_UserNameInformation as UserNameInformation,
    _USER_INFORMATION_CLASS_UserSetPasswordInformation as UserSetPasswordInformation,
};
// syssetup's own enum (sdk/include/reactos/libs/syssetup/syssetup.h:27), same prefixing rule.
pub use crate::types::{
    _PRODUCT_OPTION_PRODUCT_OPTION_DEFAULT as PRODUCT_OPTION_DEFAULT,
    _PRODUCT_OPTION_PRODUCT_OPTION_SERVER as PRODUCT_OPTION_SERVER,
    _PRODUCT_OPTION_PRODUCT_OPTION_WORKSTATION as PRODUCT_OPTION_WORKSTATION,
};

// ================= constants bindgen cannot fold ==============================================
// Cast expressions and macro expressions in their headers, so the island carries none of them.
// The value is transcribed WITH the header and line, because a hand-typed constant that is
// merely plausible is exactly how a `match` arm ends up comparing against the wrong number.
// ntdll-sys's island types these `u32` (bindgen's default for a bare `#define 1`), but every
// use in this module is a BOOL — a return value, a struct field, a Win32 argument — and BOOL is
// a signed LONG. Shadowing the glob here is one line; the alternative was ~35 per-site casts,
// each of which would have had to be checked for being the RIGHT cast.
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF; // fileapi.h: ((DWORD)-1)
pub use win32macros_rust::{INVALID_HANDLE_VALUE, UNICODE_NULL};
pub const HWND_TOP: HWND = 0 as HWND; // winuser.h: ((HWND)0)
pub const RPC_S_OK: RPC_STATUS = 0; // winerror.h: ERROR_SUCCESS
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // wtypesbase.h, an enum member bindgen prefixes
pub const SHGFP_TYPE_DEFAULT: c_int = 1; // shlobj.h SHGFP_TYPE enum
pub const ENUM_CURRENT_SETTINGS: DWORD = 0xFFFFFFFF; // wingdi.h: ((DWORD)-1)
pub const GEOCLASS_NATION: SYSGEOCLASS = 16; // winnls.h SYSGEOCLASS enum
pub const GEO_FRIENDLYNAME: SYSGEOTYPE = 8; // winnls.h SYSGEOTYPE enum
pub const SC_STATUS_PROCESS_INFO: SC_STATUS_TYPE = 0; // winsvc.h SC_STATUS_TYPE enum
pub const SPINST_REGISTERCALLBACKAWARE: DWORD = 0x00000400; // setupapi.h
pub const IDI_WINLOGO: LPCWSTR = 32517 as LPCWSTR; // winuser.h: MAKEINTRESOURCE(32517)
// winuser.h:881-883, a three-macro chain ending in
// `DWLP_USER (DWLP_DLGPROC + sizeof(DLGPROC))` — computed, so bindgen emits none of it. Shared,
// and the shared copy records why it must stay computed rather than pasted per target.
pub use win32macros_rust::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER};
// winreg.h predefined keys: `#define HKEY_LOCAL_MACHINE ((HKEY)(ULONG_PTR)((LONG)0x80000002))`.
pub const HKEY_CLASSES_ROOT: HKEY = 0x8000_0000u32 as i32 as isize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u32 as i32 as isize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
pub const HKEY_USERS: HKEY = 0x8000_0003u32 as i32 as isize as HKEY;
// winnt.h `#define SE_SHUTDOWN_NAME TEXT("SeShutdownPrivilege")` — wide under UNICODE, so it
// needs the w!() backing rather than the island's narrow byte array.
#[macro_export]
macro_rules! SE_SHUTDOWN_NAME { () => { $crate::w!("SeShutdownPrivilege") }; }
pub use crate::SE_SHUTDOWN_NAME;
// DEFINE_GUID data exports; uuid.lib (already in target_link_libraries) provides them. Data
// externs, so there is no calling convention to get wrong.
extern "C" {
    pub static IID_IPersistFile: IID;
}
pub use crate::types::IID_IShellLinkW as IID_IShellLink;

// ================= header macros ==============================================================
// C macros never reach the island. All stay `macro_rules!` and never become out-of-line fns: an
// out-of-line forwarder is an optimization barrier that uncloaks latent ABI mismatches (§9.65).
// The word/colour/locale/resource family lives in `win32macros-rust`, ported once from the
// headers and known-answer tested there. Re-exported here under the same names, so no call site
// changed.
pub use win32macros_rust::{
    ARRAYSIZE, HIWORD, LOWORD, MAKEINTRESOURCEW, MAKELCID, MAKELONG, PRIMARYLANGID, RGB, _countof,
};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because syssetup compiles -DUNICODE. That
// is this module's property, not the shared crate's, so the alias stays here.
#[macro_export]
macro_rules! MAKEINTRESOURCE { ($i:expr $(,)?) => { $crate::prelude::MAKEINTRESOURCEW!($i) }; }
pub use crate::MAKEINTRESOURCE;
// winuser.h:4110. Stays module-local because LPARAM is a pointer-width island alias (c_long on
// i386, c_longlong on the 64-bit pair) — NOT isize, which is a distinct primitive and would not
// unify. Only the MAKELONG inside it is shared.
#[macro_export]
macro_rules! MAKELPARAM { ($lo:expr, $hi:expr $(,)?) => { $crate::prelude::MAKELONG!($lo, $hi) as LPARAM }; }
pub use crate::MAKELPARAM;
// winerror.h `#define FAILED(hr) (((HRESULT)(hr)) < 0)`. The comparison is on the SIGNED
// HRESULT: an intervening `as u32` makes it unconditionally false and deletes the failure branch.
#[macro_export]
macro_rules! FAILED { ($hr:expr $(,)?) => { (($hr as HRESULT) < 0) }; }
pub use crate::FAILED;
#[macro_export]
macro_rules! SUCCEEDED { ($hr:expr $(,)?) => { (($hr as HRESULT) >= 0) }; }
pub use crate::SUCCEEDED;
// winbase.h `#define ZeroMemory RtlZeroMemory` / `#define RtlCopyMemory memcpy`. NEVER a
// hand-rolled byte loop: LLVM folds one back into a call to memset/memcpy itself → infinite
// self-recursion → silent hang (AGENTS.md).
pub use win32macros_rust::ZeroMemory;
#[macro_export]
macro_rules! RtlCopyMemory {
    ($d:expr, $s:expr, $l:expr $(,)?) => { { $crate::prelude::memcpy($d as *mut core::ffi::c_void, $s as *const core::ffi::c_void, $l as usize); } };
}
pub use crate::RtlCopyMemory;
#[macro_export]
macro_rules! UNREFERENCED_PARAMETER { ($p:expr $(,)?) => { let _ = &$p; }; }
pub use crate::UNREFERENCED_PARAMETER;
// commctrl.h control wrappers — each is a SendMessage in a trenchcoat.
#[macro_export]
macro_rules! ListBox_AddString {
    ($hwnd:expr, $s:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LB_ADDSTRING, 0, $s as LPARAM) };
}
pub use crate::ListBox_AddString;
#[macro_export]
macro_rules! ListView_InsertItem {
    ($hwnd:expr, $item:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_INSERTITEMW, 0, $item as *mut _ as LPARAM) as c_int };
}
pub use crate::ListView_InsertItem;
#[macro_export]
macro_rules! ListView_SetBkColor {
    ($hwnd:expr, $clr:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_SETBKCOLOR, 0, $clr as LPARAM) };
}
pub use crate::ListView_SetBkColor;
#[macro_export]
macro_rules! ListView_SetTextBkColor {
    ($hwnd:expr, $clr:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, LVM_SETTEXTBKCOLOR, 0, $clr as LPARAM) };
}
pub use crate::ListView_SetTextBkColor;
#[macro_export]
macro_rules! ListView_SetImageList {
    ($hwnd:expr, $himl:expr, $kind:expr $(,)?) => {
        $crate::prelude::SendMessageW($hwnd, LVM_SETIMAGELIST, $kind as WPARAM, $himl as LPARAM) as HIMAGELIST
    };
}
pub use crate::ListView_SetImageList;
#[macro_export]
macro_rules! ListView_SetIconSpacing {
    ($hwnd:expr, $cx:expr, $cy:expr $(,)?) => {
        $crate::prelude::SendMessageW($hwnd, LVM_SETICONSPACING, 0, $crate::MAKELPARAM!($cx, $cy)) as DWORD
    };
}
pub use crate::ListView_SetIconSpacing;
// prsht.h wizard-button / page wrappers.
#[macro_export]
macro_rules! PropSheet_SetWizButtons {
    ($hdlg:expr, $flags:expr $(,)?) => { $crate::prelude::PostMessageW($hdlg, PSM_SETWIZBUTTONS, 0, $flags as LPARAM) };
}
pub use crate::PropSheet_SetWizButtons;
#[macro_export]
macro_rules! PropSheet_Changed {
    ($hdlg:expr, $hpage:expr $(,)?) => { $crate::prelude::SendMessageW($hdlg, PSM_CHANGED, $hpage as WPARAM, 0) };
}
pub use crate::PropSheet_Changed;
#[macro_export]
macro_rules! PropSheet_PressButton {
    ($hdlg:expr, $btn:expr $(,)?) => { $crate::prelude::SendMessageW($hdlg, PSM_PRESSBUTTON, $btn as WPARAM, 0) };
}
pub use crate::PropSheet_PressButton;
// commctrl.h date/time picker wrappers.
#[macro_export]
macro_rules! DateTime_GetSystemtime {
    ($hwnd:expr, $st:expr $(,)?) => { $crate::prelude::SendMessageW($hwnd, DTM_GETSYSTEMTIME, 0, $st as *mut _ as LPARAM) as DWORD };
}
pub use crate::DateTime_GetSystemtime;
#[macro_export]
macro_rules! DateTime_SetSystemtime {
    ($hwnd:expr, $flag:expr, $st:expr $(,)?) => {
        // `*const _`, not `*mut _`: DTM_SETSYSTEMTIME only READS the SYSTEMTIME, and the callers
        // pass a shared `&LocalTime`, which has no `*mut` cast. The pointee kind still has to be
        // inferable for the `as LPARAM` int-cast, which `_` gets from the argument.
        $crate::prelude::SendMessageW($hwnd, DTM_SETSYSTEMTIME, $flag as WPARAM, $st as *const _ as LPARAM) as BOOL
    };
}
pub use crate::DateTime_SetSystemtime;
// syssetup.h `#define LogItem(a, b) pSetupDebugPrint(__FILE__W, __LINE__, a, b)` — the file and
// line are the C's, kept as the Rust source position so a log line still points somewhere real.
#[macro_export]
macro_rules! LogItem {
    ($tag:expr, $msg:expr $(,)?) => {
        $crate::prelude::pSetupDebugPrint($crate::w!(file!()), line!() as i32, $tag, $msg)
    };
}
pub use crate::LogItem;

// ================= COBJMACROS =================================================================
// objbase.h's `#define IFace_Method(This, ..) (This)->lpVtbl->Method(This, ..)`. Written out
// rather than wrapped in a helper fn for the §9.65 reason, and each goes through the island's
// vtable struct so the slot ORDER is bindgen's, never hand-counted.
#[macro_export]
macro_rules! com_call {
    ($this:expr, $method:ident $(, $arg:expr)* $(,)?) => {
        ((*(*$this).lpVtbl).$method.unwrap())($this $(, $arg)*)
    };
}
pub use crate::com_call;
#[macro_export]
macro_rules! IShellLinkW_QueryInterface {
    ($t:expr, $riid:expr, $ppv:expr $(,)?) => { $crate::com_call!($t, QueryInterface, $riid, $ppv) };
}
pub use crate::IShellLinkW_QueryInterface;
#[macro_export]
macro_rules! IShellLinkW_Release { ($t:expr $(,)?) => { $crate::com_call!($t, Release) }; }
pub use crate::IShellLinkW_Release;
#[macro_export]
macro_rules! IShellLinkW_SetPath { ($t:expr, $p:expr $(,)?) => { $crate::com_call!($t, SetPath, $p) }; }
pub use crate::IShellLinkW_SetPath;
#[macro_export]
macro_rules! IShellLinkW_SetArguments { ($t:expr, $p:expr $(,)?) => { $crate::com_call!($t, SetArguments, $p) }; }
pub use crate::IShellLinkW_SetArguments;
#[macro_export]
macro_rules! IShellLinkW_SetDescription { ($t:expr, $p:expr $(,)?) => { $crate::com_call!($t, SetDescription, $p) }; }
pub use crate::IShellLinkW_SetDescription;
#[macro_export]
macro_rules! IShellLinkW_SetWorkingDirectory { ($t:expr, $p:expr $(,)?) => { $crate::com_call!($t, SetWorkingDirectory, $p) }; }
pub use crate::IShellLinkW_SetWorkingDirectory;
#[macro_export]
macro_rules! IShellLinkW_SetIconLocation {
    ($t:expr, $p:expr, $i:expr $(,)?) => { $crate::com_call!($t, SetIconLocation, $p, $i) };
}
pub use crate::IShellLinkW_SetIconLocation;
#[macro_export]
macro_rules! IPersistFile_Release { ($t:expr $(,)?) => { $crate::com_call!($t, Release) }; }
pub use crate::IPersistFile_Release;
#[macro_export]
macro_rules! IPersistFile_Save {
    ($t:expr, $f:expr, $r:expr $(,)?) => { $crate::com_call!($t, Save, $f, $r) };
}
pub use crate::IPersistFile_Save;

// ================= sibling-module functions ===================================================
// The C reaches these through syssetup.h; in Rust they are items of another module of this same
// crate, so they are re-exported here rather than declared `extern`. An extern DECLARATION for
// something the crate itself defines is the drift shape AGENTS.md warns about — it would link
// against our own symbol with a signature nothing checks.
pub use crate::syssetup::install::{InstallStartMenuItems, RegisterTypeLibraries};
pub use crate::syssetup::strsafe::{StringCbCopyW, StringCchCatW, StringCchCopyW};
// pSetupDebugPrint is NOT re-exported from logfile: the crate only defines the fixed-arity twin
// `pSetupDebugPrint_v`. The variadic entry point is rust_varargs_shim.c's, and the island already
// declares it (`unsafe extern "C" { pub fn pSetupDebugPrint(.., ...) }`), so it arrives with the
// rest of `crate::types::*`.
pub use crate::syssetup::logfile::{InitializeSetupActionLog, TerminateSetupActionLog};
pub use crate::syssetup::netinstall::InstallNetworkComponent;
pub use crate::syssetup::security::{
    InstallSecurity, SetAccountsDomainSid, SetAdministratorPassword, SetAutoAdminLogon,
};
pub use crate::syssetup::wizard::{EnableVisualTheme, InstallWizard};

// ================= imports from other DLLs ====================================================
// shell32's rundll32 entry point. Not in the island because syssetup reaches it by
// GetProcAddress-shaped declaration in its own .c rather than through a public header.
extern "system" {
    pub fn Control_RunDLLW(hWnd: HWND, hInst: HINSTANCE, cmd: LPCWSTR, nCmdShow: c_int);
}
// winuser.h `#define DialogBox(i,t,p,f) DialogBoxParamW(i,t,p,f,0)` under UNICODE.
#[macro_export]
macro_rules! DialogBox {
    ($inst:expr, $tmpl:expr, $parent:expr, $proc:expr $(,)?) => {
        $crate::prelude::DialogBoxParamW($inst, $tmpl, $parent, $proc, 0)
    };
}
pub use crate::DialogBox;

// ================= CRT the island does not carry ==============================================
// strsafe.h's StringCch*/StringCb* are STRSAFEAPI `static __inline`, so no object file in the
// tree defines them — they are ported in syssetup::strsafe and re-exported below. The VARIADIC
// StringCchPrintfW is a strsafe-rust macro (calls `_snwprintf` in place): `StringCchPrintfW!(..)`.
pub use strsafe_rust::StringCchPrintfW;
extern "C" {
    pub fn swprintf(buf: *mut u16, fmt: *const u16, ...) -> c_int;
    pub fn _ultow(value: u32, buf: *mut u16, radix: c_int) -> *mut u16;
    pub fn _wtoi(s: *const u16) -> c_int;
    pub fn isprint(c: c_int) -> c_int;
    pub fn rand() -> c_int;
    pub fn srand(seed: c_uint);
    pub fn time(t: *mut i64) -> i64;
}
// ntdll inline: `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)`. bindgen cannot see
// through it, and GetProcessHeap is the documented kernel32 spelling of the same thing.
#[inline(always)]
pub unsafe fn RtlGetProcessHeap() -> HANDLE {
    GetProcessHeap()
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;
