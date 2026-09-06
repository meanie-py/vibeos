//! netid-rust prelude — the single import surface the translated modules use.
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
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int;
    pub fn _wcsicmp(a: *const u16, b: *const u16) -> c_int;
}

// C-variadic: DECLARING and CALLING one is stable Rust; only *defining* one needs nightly.
// `extern "C"`, NOT "system": wsprintf is WINAPIV (__cdecl), because a variadic cannot be
// stdcall — the callee cannot know how many bytes to pop. Declaring it "system" is both a
// nightly feature ("C-variadic functions with the system calling convention are unstable")
// and, on i386, the wrong ABI: the caller would leave the stack unbalanced.
extern "C" {
    pub fn wsprintfW(buf: *mut u16, format: *const u16, ...) -> c_int;
}
pub use self::wsprintfW as wsprintf;

// A wide string literal, NUL-terminated, with one stable address for the program's life.
// `static` and not `const`: a const is inlined at each use site, so `.as_ptr()` would hand the
// Win32 callee the address of a temporary. ASCII-only by construction, so a non-ASCII literal
// fails at COMPILE time instead of silently truncating.
#[macro_export]
macro_rules! w {
    ($s:expr) => {{
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

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= UNICODE aliases ============================================================
// netid builds with -DUNICODE -D_UNICODE, so the SDK's `#define FOO FOOW` picks the W entry
// point. Those are preprocessor renames and invisible to a translator reading the .c, exactly
// like rpcndr.h's midl_user_allocate -> MIDL_user_allocate. Alias rather than re-declare.
pub type PROPSHEETPAGE = PROPSHEETPAGEW;
pub type LPPROPSHEETPAGE = LPPROPSHEETPAGEW;

// ================= enum member aliases ========================================================
// bindgen prefixes C enum members with their enum's tag, so the bare C spelling is absent.
// Alias, never re-declare: a literal here could drift from the island's value silently.
pub const NetSetupUnknownStatus: _NETSETUP_JOIN_STATUS = _NETSETUP_JOIN_STATUS_NetSetupUnknownStatus;
pub const NetSetupDomainName: _NETSETUP_JOIN_STATUS = _NETSETUP_JOIN_STATUS_NetSetupDomainName;
pub const NetSetupWorkgroupName: _NETSETUP_JOIN_STATUS =
    _NETSETUP_JOIN_STATUS_NetSetupWorkgroupName;
pub const DnsNameDomain: _DNS_NAME_FORMAT = _DNS_NAME_FORMAT_DnsNameDomain;
pub const DnsNameHostnameLabel: _DNS_NAME_FORMAT = _DNS_NAME_FORMAT_DnsNameHostnameLabel;

// ================= constants bindgen cannot emit ==============================================
// lmcons.h `#define NET_API_STATUS DWORD` — an object-like macro, so there is no typedef for
// bindgen to emit and the name simply vanishes from the island.
pub type NET_API_STATUS = DWORD;
pub use win32macros_rust::UNICODE_NULL;
pub const NERR_Success: NET_API_STATUS = 0;
// winnt.h `#define SECURITY_NT_AUTHORITY {0,0,0,0,0,5}` — a braced INITIALIZER, not a value,
// so it is not a constant bindgen can emit. Spelled as the array the C initializes.
pub const SECURITY_NT_AUTHORITY: [u8; 6] = [0, 0, 0, 0, 0, 5];
// winreg.h HKEY_LOCAL_MACHINE = ((HKEY)(ULONG_PTR)((LONG)0x80000002)) — a cast chain, not an
// integer constant. The sign-extend through LONG matters on 64-bit: 0x80000002 as i32 as isize
// gives 0xFFFFFFFF80000002, which is what the C produces and what the registry expects.
pub const HKEY_LOCAL_MACHINE: HKEY = 0x8000_0002u32 as i32 as isize as HKEY;
// winuser.h: DWLP_USER = DWLP_MSGRESULT + sizeof(LRESULT) + sizeof(DLGPROC). POINTER-WIDTH
// DEPENDENT (8 on i386, 16 on amd64/arm64) — computed, never a literal, or the 64-bit builds
// read the wrong dialog word.
pub use win32macros_rust::{DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER};

// ================= function-like MACROS =======================================================
// These are macros in C. Keep them macros in Rust: an out-of-line fn is an optimization
// barrier that can uncloak a latent ABI mismatch into ud2 (field manual §9.65), and ARRAYSIZE
// must stay generic over the array type anyway.
// The word/resource family lives in `win32macros-rust`, ported once from the headers and
// known-answer tested there. Re-exported here under the same names, so no call site changed.
pub use win32macros_rust::{ARRAYSIZE, HIWORD, LOWORD, MAKEINTRESOURCEW};
// MAKEINTRESOURCE (no suffix) is MAKEINTRESOURCEW only because netid compiles -DUNICODE. That is
// this module's property, not the shared crate's, so the alias stays here. The WORD truncation
// inside it is load-bearing and the shared port keeps it.
pub use win32macros_rust::MAKEINTRESOURCEW as MAKEINTRESOURCE;
// prsht.h PropSheet_Changed(hDlg, hPage) = SendMessage(hDlg, PSM_CHANGED, (WPARAM)hPage, 0)
#[macro_export]
macro_rules! PropSheet_Changed {
    ($hdlg:expr, $hpage:expr) => {
        $crate::prelude::SendMessageW($hdlg, PSM_CHANGED, $hpage as WPARAM, 0)
    };
}
pub use crate::PropSheet_Changed;

// ================= Win32 imports ==============================================================
// Declared, never defined: these live in the DLLs netid links against (add_importlibs).
extern "system" {
    // user32
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: c_int) -> HWND;
    pub fn GetDlgItemTextW(hDlg: HWND, nIDDlgItem: c_int, lpString: LPWSTR, cchMax: c_int)
        -> UINT;
    pub fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: c_int, lpString: LPCWSTR) -> BOOL;
    pub fn SendDlgItemMessageW(
        hDlg: HWND,
        nIDDlgItem: c_int,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetParent(hWnd: HWND) -> HWND;
    pub fn EndDialog(hDlg: HWND, nResult: INT_PTR) -> BOOL;
    pub fn MessageBoxW(hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT) -> c_int;
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: UINT,
        lpBuffer: LPWSTR,
        cchBufferMax: c_int,
    ) -> c_int;
    pub fn GetWindowTextW(hWnd: HWND, lpString: LPWSTR, nMaxCount: c_int) -> c_int;
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: LPCWSTR,
        hWndParent: HWND,
        lpDialogFunc: DLGPROC,
        dwInitParam: LPARAM,
    ) -> INT_PTR;

    // comctl32 (prsht)
    pub fn CreatePropertySheetPageW(constPropSheetPagePointer: LPCPROPSHEETPAGEW)
        -> HPROPSHEETPAGE;

    // kernel32
    pub fn GetLastError() -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> *mut c_void;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: *mut c_void) -> BOOL;
    pub fn FormatMessageW(
        dwFlags: DWORD,
        lpSource: *const c_void,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: LPWSTR,
        nSize: DWORD,
        Arguments: *mut c_void,
    ) -> DWORD;
    pub fn SetComputerNameExW(NameType: COMPUTER_NAME_FORMAT, lpBuffer: LPCWSTR) -> BOOL;
    pub fn DisableThreadLibraryCalls(hLibModule: HINSTANCE) -> BOOL;

    // advapi32
    pub fn AllocateAndInitializeSid(
        pIdentifierAuthority: PSID_IDENTIFIER_AUTHORITY,
        nSubAuthorityCount: u8,
        nSubAuthority0: DWORD,
        nSubAuthority1: DWORD,
        nSubAuthority2: DWORD,
        nSubAuthority3: DWORD,
        nSubAuthority4: DWORD,
        nSubAuthority5: DWORD,
        nSubAuthority6: DWORD,
        nSubAuthority7: DWORD,
        pSid: *mut PSID,
    ) -> BOOL;
    pub fn CheckTokenMembership(TokenHandle: HANDLE, SidToCheck: PSID, IsMember: *mut BOOL)
        -> BOOL;
    pub fn FreeSid(pSid: PSID) -> *mut c_void;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: LPCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: *mut HKEY,
    ) -> LONG;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut u8,
        lpcbData: *mut DWORD,
    ) -> LONG;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: LPCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const u8,
        cbData: DWORD,
    ) -> LONG;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;

    // netapi32
    pub fn NetApiBufferFree(Buffer: *mut c_void) -> NET_API_STATUS;
    pub fn NetGetJoinInformation(
        lpServer: LPCWSTR,
        lpNameBuffer: *mut LPWSTR,
        BufferType: *mut NETSETUP_JOIN_STATUS,
    ) -> NET_API_STATUS;

    // dnsapi
    pub fn DnsValidateName_W(pszName: LPCWSTR, Format: DNS_NAME_FORMAT) -> DNS_STATUS;
    pub fn DnsHostnameToComputerNameW(
        Hostname: LPCWSTR,
        ComputerName: LPWSTR,
        nSize: *mut DWORD,
    ) -> BOOL;
}

// winuser.h only EXPORTS GetWindowLongPtrW/SetWindowLongPtrW on _WIN64; on 32-bit they are
// `#define`s onto the plain LONG entry points (winuser.h:4931/5457). Declaring the Ptr forms
// unconditionally would link-fail on i386 against a symbol user32 does not export there.
// LONG and LONG_PTR are the same width on 32-bit, so the alias is exact either way.
#[cfg(target_pointer_width = "64")]
extern "system" {
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> LONG_PTR;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG_PTR) -> LONG_PTR;
}
#[cfg(not(target_pointer_width = "64"))]
extern "system" {
    pub fn GetWindowLongW(hWnd: HWND, nIndex: c_int) -> LONG;
    pub fn SetWindowLongW(hWnd: HWND, nIndex: c_int, dwNewLong: LONG) -> LONG;
}
#[cfg(not(target_pointer_width = "64"))]
pub use self::{GetWindowLongW as GetWindowLongPtrW, SetWindowLongW as SetWindowLongPtrW};

// The UNICODE `#define FOO FOOW` renames, as aliases so there is exactly one symbol identity.
pub use self::{
    CreatePropertySheetPageW as CreatePropertySheetPage, DialogBoxParamW as DialogBoxParam,
    GetDlgItemTextW as GetDlgItemText, GetWindowLongPtrW as GetWindowLongPtr,
    GetWindowTextW as GetWindowText, SendDlgItemMessageW as SendDlgItemMessage,
    SendMessageW as SendMessage, SetDlgItemTextW as SetDlgItemText,
    SetWindowLongPtrW as SetWindowLongPtr,
};
