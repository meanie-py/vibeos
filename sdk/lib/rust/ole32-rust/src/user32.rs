//! user32 / kernel32 / advapi32 / rpcrt4 import surface for the ring's OLE2, clipboard
//! and moniker paths (drag-drop hooks a message loop, the clipboard owns a hidden
//! window, the ROT talks RPC, and the moniker/comcat code walks HKCR).
//!
//! Same ABI rule as win32.rs / gdi32.rs: all WINAPI, so `extern "system"` (stdcall on
//! i386). Declaring these `extern "C"` links on amd64/arm64 and breaks i386 — the raw
//! rustconv output does exactly that, which is why they are centralized here.
#![allow(clippy::all)]
use crate::prelude::*;
// One declaration per external symbol: the variant matching the C prototype.
pub use crate::ring::ole2impl::CreateBitmapIndirect;
pub use crate::ring::ole2impl::CreatePalette;
pub use crate::compobj::ffi::CreateWindowExW;
pub use crate::compobj::ffi::DestroyWindow;
pub use crate::compobj::ffi::DispatchMessageW;
pub use crate::compobj::ffi::GetMessageW;
pub use crate::ring::ole2impl::GetPaletteEntries;
pub use crate::marshal::types::PostMessageW;
pub use crate::marshal::types::ReadFile;
pub use crate::compobj::ffi::SendMessageW;
pub use crate::marshal::types::WriteFile;
use core::ffi::{c_char, c_int, c_void};

pub type HHOOK = *mut c_void;
pub type HPALETTE = *mut c_void;
// HENHMETAFILE is RE-EXPORTED, not re-aliased: com-sys already defines it
// (`*mut HENHMETAFILE__`). A second, independent `pub type HENHMETAFILE = *mut c_void`
// here would be a DIFFERENT item with the same name, reachable through two of the
// prelude's globs — an E0659 ambiguity the moment anything uses it. Re-exporting the
// same item collapses both paths onto one definition and keeps `crate::user32::
// HENHMETAFILE` (which translated call sites spell out) resolving.
pub use com_sys::HENHMETAFILE;
pub type HKEY = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type SC_HANDLE = *mut c_void;
pub type REGSAM = u32;
pub type HOOKPROC = Option<unsafe extern "system" fn(c_int, WPARAM, LPARAM) -> LRESULT>;

/// winreg.h: `#define HKEY_CLASSES_ROOT ((HKEY)(LONG_PTR)(LONG)0x80000000)`
/// The LONG→LONG_PTR step SIGN-EXTENDS on 64-bit (→ 0xFFFFFFFF80000000). Getting this
/// wrong yields a bogus key handle and every registry lookup silently misses.
pub const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as i32 as isize as usize as HKEY;
pub const HKEY_LOCAL_MACHINE: HKEY = 0x80000002u32 as i32 as isize as usize as HKEY;
pub const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as i32 as isize as usize as HKEY;

pub const KEY_READ: REGSAM = 0x2_0019;
pub const KEY_WRITE: REGSAM = 0x2_0006;
pub const KEY_ALL_ACCESS: REGSAM = 0xF_003F;
pub const ERROR_SUCCESS: LONG = 0;

pub const WH_GETMESSAGE: c_int = 3;
pub const WH_CALLWNDPROC: c_int = 4;

extern "system" {
    // --- clipboard ---
    pub fn OpenClipboard(hWndNewOwner: HWND) -> BOOL;
    pub fn CloseClipboard() -> BOOL;
    pub fn EmptyClipboard() -> BOOL;
    pub fn GetClipboardData(uFormat: UINT) -> HANDLE;
    pub fn SetClipboardData(uFormat: UINT, hMem: HANDLE) -> HANDLE;
    pub fn EnumClipboardFormats(format: UINT) -> UINT;
    pub fn IsClipboardFormatAvailable(format: UINT) -> BOOL;
    pub fn RegisterClipboardFormatW(lpszFormat: *const u16) -> UINT;
    pub fn GetClipboardFormatNameW(format: UINT, lpszFormatName: *mut u16, cchMaxCount: c_int) -> c_int;
    pub fn GetClipboardSequenceNumber() -> DWORD;

    // --- windows / messages (the clipboard's hidden window + DoDragDrop's loop) ---
    pub fn RegisterClassExW(unnamedParam1: *const c_void) -> u16;
    pub fn UnregisterClassW(lpClassName: *const u16, hInstance: HINSTANCE) -> BOOL;
    pub fn IsWindow(hWnd: HWND) -> BOOL;
    pub fn IsWindowUnicode(hWnd: HWND) -> BOOL;
    pub fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn PostQuitMessage(nExitCode: c_int);
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;

    // --- window props (drag-drop target registration) ---
    pub fn SetPropW(hWnd: HWND, lpString: *const u16, hData: HANDLE) -> BOOL;
    pub fn GetPropW(hWnd: HWND, lpString: *const u16) -> HANDLE;
    pub fn RemovePropW(hWnd: HWND, lpString: *const u16) -> HANDLE;

    // --- input / hooks (DoDragDrop) ---
    pub fn SetWindowsHookExW(idHook: c_int, lpfn: HOOKPROC, hmod: HINSTANCE, dwThreadId: DWORD) -> HHOOK;
    pub fn UnhookWindowsHookEx(hhk: HHOOK) -> BOOL;
    pub fn CallNextHookEx(hhk: HHOOK, nCode: c_int, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn GetKeyState(nVirtKey: c_int) -> i16;
    pub fn GetKeyboardState(lpKeyState: *mut u8) -> BOOL;
    pub fn SetCapture(hWnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> BOOL;
    pub fn GetMenuItemCount(hMenu: HMENU) -> c_int;
    pub fn GetSubMenu(hMenu: HMENU, nPos: c_int) -> HMENU;
    pub fn CopyAcceleratorTableW(hAccelSrc: HANDLE, lpAccelDst: *mut c_void, cAccelEntries: c_int) -> c_int;

    // --- gdi (clipboard bitmap/metafile formats) ---
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: c_int, cy: c_int) -> HBITMAP;
    pub fn GetBitmapBits(hbit: HBITMAP, cb: LONG, lpvBits: LPVOID) -> LONG;
    pub fn GetObjectW(h: HANDLE, c: c_int, pv: LPVOID) -> c_int;
    pub fn BitBlt(hdc: HDC, x: c_int, y: c_int, cx: c_int, cy: c_int,
                  hdcSrc: HDC, x1: c_int, y1: c_int, rop: DWORD) -> BOOL;
    pub fn CopyMetaFileW(hmfSrc: HANDLE, lpszFile: *const u16) -> HANDLE;
    pub fn CopyEnhMetaFileW(hEnh: HENHMETAFILE, lpFileName: *const u16) -> HENHMETAFILE;
    pub fn DeleteEnhMetaFile(hmf: HENHMETAFILE) -> BOOL;
    pub fn GetMetaFileBitsEx(hMF: HANDLE, cbBuffer: UINT, lpData: LPVOID) -> UINT;
    pub fn GetEnhMetaFileBits(hEMF: HENHMETAFILE, nSize: UINT, lpData: *mut u8) -> UINT;
    pub fn SetEnhMetaFileBits(nSize: UINT, pb: *const u8) -> HENHMETAFILE;

    // --- kernel32 ---
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn GetTickCount64() -> u64;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn SetFilePointerEx(hFile: HANDLE, liDistanceToMove: LARGE_INTEGER,
                            lpNewFilePointer: *mut LARGE_INTEGER, dwMoveMethod: DWORD) -> BOOL;
    pub fn SetEndOfFile(hFile: HANDLE) -> BOOL;
    pub fn GetFileSize(hFile: HANDLE, lpFileSizeHigh: *mut DWORD) -> DWORD;
    pub fn GetFileAttributesW(lpFileName: *const u16) -> DWORD;
    pub fn DeleteFileW(lpFileName: *const u16) -> BOOL;
    pub fn GetFullPathNameW(lpFileName: *const u16, nBufferLength: DWORD,
                            lpBuffer: *mut u16, lpFilePart: *mut *mut u16) -> DWORD;
    pub fn LockFileEx(hFile: HANDLE, dwFlags: DWORD, dwReserved: DWORD,
                      nNumberOfBytesToLockLow: DWORD, nNumberOfBytesToLockHigh: DWORD,
                      lpOverlapped: *mut c_void) -> BOOL;
    pub fn UnlockFileEx(hFile: HANDLE, dwReserved: DWORD,
                        nNumberOfBytesToUnlockLow: DWORD, nNumberOfBytesToUnlockHigh: DWORD,
                        lpOverlapped: *mut c_void) -> BOOL;
    pub fn MultiByteToWideChar(CodePage: UINT, dwFlags: DWORD, lpMultiByteStr: *const c_char,
                               cbMultiByte: c_int, lpWideCharStr: *mut u16, cchWideChar: c_int) -> c_int;
    pub fn RaiseException(dwExceptionCode: DWORD, dwExceptionFlags: DWORD,
                          nNumberOfArguments: DWORD, lpArguments: *const usize);

    // --- WINAPI string helpers (stdcall — NOT the CRT) ---
    // NOTE wsprintfW lives in crt.rs, not here: it is VARIADIC, and a variadic Win32
    // function is __cdecl even on i386 (stdcall's callee-cleans-stack cannot express a
    // variable arg count). rustc enforces this — `extern "system"` + `...` is rejected.
    pub fn lstrlenA(lpString: *const c_char) -> c_int;
    pub fn lstrcatW(lpString1: *mut u16, lpString2: *const u16) -> *mut u16;
    pub fn lstrcpynW(lpString1: *mut u16, lpString2: *const u16, iMaxLength: c_int) -> *mut u16;

    // --- advapi32: registry + SCM (comcat / classmoniker / the ROT's rpcss check) ---
    pub fn RegOpenKeyExW(hKey: HKEY, lpSubKey: *const u16, ulOptions: DWORD,
                         samDesired: REGSAM, phkResult: *mut HKEY) -> LONG;
    pub fn RegCloseKey(hKey: HKEY) -> LONG;
    pub fn RegDeleteKeyW(hKey: HKEY, lpSubKey: *const u16) -> LONG;
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: *mut u16, cchName: DWORD) -> LONG;
    pub fn RegEnumKeyExW(hKey: HKEY, dwIndex: DWORD, lpName: *mut u16, lpcchName: *mut DWORD,
                         lpReserved: *mut DWORD, lpClass: *mut u16, lpcchClass: *mut DWORD,
                         lpftLastWriteTime: *mut FILETIME) -> LONG;
    pub fn RegQueryValueW(hKey: HKEY, lpSubKey: *const u16, lpData: *mut u16, lpcbData: *mut LONG) -> LONG;
    pub fn RegQueryValueExW(hKey: HKEY, lpValueName: *const u16, lpReserved: *mut DWORD,
                            lpType: *mut DWORD, lpData: *mut u8, lpcbData: *mut DWORD) -> LONG;
    pub fn RegSetValueW(hKey: HKEY, lpSubKey: *const u16, dwType: DWORD,
                        lpData: *const u16, cbData: DWORD) -> LONG;
    pub fn RegSetValueExW(hKey: HKEY, lpValueName: *const u16, Reserved: DWORD, dwType: DWORD,
                          lpData: *const u8, cbData: DWORD) -> LONG;
    pub fn OpenSCManagerW(lpMachineName: *const u16, lpDatabaseName: *const u16,
                          dwDesiredAccess: DWORD) -> SC_HANDLE;
    pub fn OpenServiceW(hSCManager: SC_HANDLE, lpServiceName: *const u16,
                        dwDesiredAccess: DWORD) -> SC_HANDLE;
    pub fn StartServiceW(hService: SC_HANDLE, dwNumServiceArgs: DWORD,
                         lpServiceArgVectors: *const *const u16) -> BOOL;
    pub fn QueryServiceStatusEx(hService: SC_HANDLE, InfoLevel: c_int, lpBuffer: *mut u8,
                                cbBufSize: DWORD, pcbBytesNeeded: *mut DWORD) -> BOOL;
    pub fn CloseServiceHandle(hSCObject: SC_HANDLE) -> BOOL;

    // --- rpcrt4 (the running-object-table talks to rpcss) ---
    pub fn RpcStringBindingComposeW(ObjUuid: *const u16, ProtSeq: *const u16, NetworkAddr: *const u16,
                                    Endpoint: *const u16, Options: *const u16,
                                    StringBinding: *mut *mut u16) -> LONG;
    pub fn RpcBindingFromStringBindingW(StringBinding: *const u16, Binding: *mut *mut c_void) -> LONG;
    pub fn RpcStringFreeW(String: *mut *mut u16) -> LONG;
    pub fn RpcBindingFree(Binding: *mut *mut c_void) -> LONG;
    pub fn RpcRaiseException(exception: LONG) -> !;
    pub fn I_RpcExceptionFilter(ExceptionCode: ULONG) -> c_int;

    // ---- Tier-D additions ----
    pub fn RegisterClipboardFormatA(lpszFormat: *const c_char) -> UINT;
    pub fn GetClipboardFormatNameA(format: UINT, lpszFormatName: *mut c_char, cchMaxCount: c_int) -> c_int;
}
