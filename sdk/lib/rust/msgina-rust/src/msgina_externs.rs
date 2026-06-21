//! Cross-DLL out-call declarations for msgina-rust.
//!
//! msgina calls the PUBLIC Win32/LSA APIs in user32/gdi32/advapi32/kernel32/secur32.
//! Those symbols resolve at LINK time via the cmake importlibs (`add_importlibs(msgina
//! advapi32 user32 gdi32 userenv msvcrt kernel32 ntdll)` + delay secur32), so here we
//! only declare them. WINAPI both lowers to `extern "system"` (stdcall on i386, cdecl
//! on amd64/arm64). CRT helpers are `extern "C"` (cdecl). Grown per-file as each
//! translation surfaces a new out-call.
#![allow(non_snake_case, non_upper_case_globals, dead_code)]

use crate::prelude::*;
use crate::msgina_types::*;
use core::ffi::c_int;

extern "system" {
    // --- advapi32: tokens / impersonation / registry ---
    pub fn ImpersonateLoggedOnUser(hToken: HANDLE) -> BOOL;
    pub fn RevertToSelf() -> BOOL;
    pub fn OpenThreadToken(
        ThreadHandle: HANDLE,
        DesiredAccess: DWORD,
        OpenAsSelf: BOOL,
        TokenHandle: PHANDLE,
    ) -> BOOL;
    pub fn OpenProcessToken(
        ProcessHandle: HANDLE,
        DesiredAccess: DWORD,
        TokenHandle: PHANDLE,
    ) -> BOOL;
    pub fn GetTokenInformation(
        TokenHandle: HANDLE,
        TokenInformationClass: TOKEN_INFORMATION_CLASS,
        TokenInformation: PVOID,
        TokenInformationLength: DWORD,
        ReturnLength: *mut DWORD,
    ) -> BOOL;
    // Return LSTATUS — declared as DWORD so the Win32 ERROR_*/REG_* (u32) constants
    // and these status returns share one type (LSTATUS/LONG and DWORD are the same
    // 32-bit value; ABI-identical for the cohab).
    pub fn RegOpenCurrentUser(samDesired: REGSAM, phkResult: PHKEY) -> DWORD;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: PCWSTR,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: PBYTE,
        lpcbData: *mut DWORD,
    ) -> DWORD;

    // --- kernel32: handles / heap / process+thread ---
    pub fn GetLastError() -> DWORD;
    pub fn GetCurrentThread() -> HANDLE;
    pub fn GetCurrentProcess() -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> PVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    pub fn LocalAlloc(uFlags: UINT, uBytes: SIZE_T) -> PVOID;
    pub fn LocalFree(hMem: PVOID) -> PVOID;

    // --- advapi32: registry ---
    pub fn RegCloseKey(hKey: HKEY) -> DWORD;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: PCWSTR,
        ulOptions: DWORD,
        samDesired: REGSAM,
        phkResult: PHKEY,
    ) -> DWORD;
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: PCWSTR,
        Reserved: DWORD,
        lpClass: PWSTR,
        dwOptions: DWORD,
        samDesired: REGSAM,
        lpSecurityAttributes: PVOID,
        phkResult: PHKEY,
        lpdwDisposition: *mut DWORD,
    ) -> DWORD;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: PCWSTR,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const u8,
        cbData: DWORD,
    ) -> DWORD;

    // --- advapi32: tokens / process creation / unique id ---
    pub fn DuplicateTokenEx(
        hExistingToken: HANDLE,
        dwDesiredAccess: DWORD,
        lpTokenAttributes: PVOID,
        ImpersonationLevel: SECURITY_IMPERSONATION_LEVEL,
        TokenType: TOKEN_TYPE,
        phNewToken: PHANDLE,
    ) -> BOOL;
    pub fn CreateProcessAsUserW(
        hToken: HANDLE,
        lpApplicationName: PCWSTR,
        lpCommandLine: PWSTR,
        lpProcessAttributes: PVOID,
        lpThreadAttributes: PVOID,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: PVOID,
        lpCurrentDirectory: PCWSTR,
        lpStartupInfo: PVOID,
        lpProcessInformation: PVOID,
    ) -> BOOL;
    pub fn GetUserNameW(lpBuffer: PWSTR, pcbBuffer: *mut DWORD) -> BOOL;

    // --- kernel32: misc ---
    pub fn ExpandEnvironmentStringsW(lpSrc: PCWSTR, lpDst: PWSTR, nSize: DWORD) -> DWORD;
    pub fn GetComputerNameW(lpBuffer: PWSTR, nSize: *mut DWORD) -> BOOL;
    pub fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
    pub fn GetVersionExW(lpVersionInformation: PVOID) -> BOOL;
    pub fn GetWindowsDirectoryW(lpBuffer: PWSTR, uSize: UINT) -> UINT;
    pub fn GetDateFormatW(
        Locale: DWORD,
        dwFlags: DWORD,
        lpDate: *const SYSTEMTIME,
        lpFormat: PCWSTR,
        lpDateStr: PWSTR,
        cchDate: c_int,
    ) -> c_int;
    pub fn GetTimeFormatW(
        Locale: DWORD,
        dwFlags: DWORD,
        lpTime: *const SYSTEMTIME,
        lpFormat: PCWSTR,
        lpTimeStr: PWSTR,
        cchTime: c_int,
    ) -> c_int;
    pub fn GetUserDefaultLangID() -> WORD;
    pub fn AllocConsole() -> BOOL;
    pub fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
    pub fn CreateEventW(
        lpEventAttributes: PVOID,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: PCWSTR,
    ) -> HANDLE;
    pub fn SetEvent(hEvent: HANDLE) -> BOOL;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
    pub fn CreateThread(
        lpThreadAttributes: PVOID,
        dwStackSize: SIZE_T,
        lpStartAddress: PVOID,
        lpParameter: PVOID,
        dwCreationFlags: DWORD,
        lpThreadId: *mut DWORD,
    ) -> HANDLE;
    pub fn DuplicateHandle(
        hSourceProcessHandle: HANDLE,
        hSourceHandle: HANDLE,
        hTargetProcessHandle: HANDLE,
        lpTargetHandle: PHANDLE,
        dwDesiredAccess: DWORD,
        bInheritHandle: BOOL,
        dwOptions: DWORD,
    ) -> BOOL;

    // --- user32: window / dialog management ---
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: usize, lParam: isize) -> LRESULT;
    pub fn SendDlgItemMessageW(
        hDlg: HWND,
        nIDDlgItem: c_int,
        Msg: UINT,
        wParam: usize,
        lParam: isize,
    ) -> LRESULT;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: c_int) -> HWND;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn EndDialog(hDlg: HWND, nResult: isize) -> BOOL;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: PCWSTR,
        lpWindowName: PCWSTR,
        dwStyle: DWORD,
        X: c_int,
        Y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: PVOID,
    ) -> HWND;
    pub fn DialogBoxParamW(
        hInstance: HINSTANCE,
        lpTemplateName: PCWSTR,
        hWndParent: HWND,
        lpDialogFunc: DLGPROC,
        dwInitParam: LPARAM,
    ) -> INT_PTR;
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: c_int) -> isize;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: c_int, dwNewLong: isize) -> isize;
    pub fn SetDlgItemTextW(hDlg: HWND, nIDDlgItem: c_int, lpString: PCWSTR) -> BOOL;
    pub fn GetDlgItemTextW(hDlg: HWND, nIDDlgItem: c_int, lpString: PWSTR, cchMax: c_int) -> UINT;
    pub fn GetParent(hWnd: HWND) -> HWND;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: c_int) -> BOOL;
    pub fn CallWindowProcW(
        lpPrevWndFunc: WNDPROC,
        hWnd: HWND,
        Msg: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
    pub fn ClientToScreen(hWnd: HWND, lpPoint: *mut POINT) -> BOOL;
    pub fn GetCapture() -> HWND;
    pub fn SetCapture(hWnd: HWND) -> HWND;
    pub fn ReleaseCapture() -> BOOL;
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> c_int;
    pub fn GetDlgCtrlID(hWnd: HWND) -> c_int;
    pub fn GetKeyState(nVirtKey: c_int) -> SHORT;
    pub fn GetSystemMetrics(nIndex: c_int) -> c_int;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetDesktopWindow() -> HWND;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn KillTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL;
    pub fn SetTimer(hWnd: HWND, nIDEvent: UINT_PTR, uElapse: UINT, lpTimerFunc: PVOID) -> UINT_PTR;
    pub fn LoadBitmapW(hInstance: HINSTANCE, lpBitmapName: PCWSTR) -> HBITMAP;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: PCWSTR) -> HCURSOR;
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: PCWSTR,
        type_: UINT,
        cx: c_int,
        cy: c_int,
        fuLoad: UINT,
    ) -> HANDLE;
    pub fn MapWindowPoints(
        hWndFrom: HWND,
        hWndTo: HWND,
        lpPoints: *mut POINT,
        cPoints: UINT,
    ) -> c_int;
    pub fn MessageBoxW(hWnd: HWND, lpText: PCWSTR, lpCaption: PCWSTR, uType: UINT) -> c_int;
    pub fn MoveWindow(
        hWnd: HWND,
        X: c_int,
        Y: c_int,
        nWidth: c_int,
        nHeight: c_int,
        bRepaint: BOOL,
    ) -> BOOL;
    pub fn SetCursor(hCursor: HCURSOR) -> HCURSOR;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: c_int,
        Y: c_int,
        cx: c_int,
        cy: c_int,
        uFlags: UINT,
    ) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: PCWSTR) -> BOOL;
    pub fn SetRect(lprc: *mut RECT, xLeft: c_int, yTop: c_int, xRight: c_int, yBottom: c_int)
        -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn WindowFromPoint(Point: POINT) -> HWND;
    pub fn LoadStringW(
        hInstance: HINSTANCE,
        uID: c_int,
        lpBuffer: PWSTR,
        cchBufferMax: c_int,
    ) -> c_int;
    pub fn AdjustWindowRectEx(lpRect: *mut RECT, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD)
        -> BOOL;
    pub fn SetThreadDesktop(hDesktop: HDESK) -> BOOL;

    // --- gdi32 ---
    pub fn MulDiv(nNumber: c_int, nNumerator: c_int, nDenominator: c_int) -> c_int;
    pub fn GetObjectW(h: HGDIOBJ, c: c_int, pv: PVOID) -> c_int;
    pub fn BitBlt(
        hdc: HDC,
        x: c_int,
        y: c_int,
        cx: c_int,
        cy: c_int,
        hdcSrc: HDC,
        x1: c_int,
        y1: c_int,
        rop: DWORD,
    ) -> BOOL;
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn CreateFontW(
        cHeight: c_int,
        cWidth: c_int,
        cEscapement: c_int,
        cOrientation: c_int,
        cWeight: c_int,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: PCWSTR,
    ) -> HFONT;
    pub fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    pub fn GetDeviceCaps(hdc: HDC, index: c_int) -> c_int;
    pub fn GetStockObject(i: c_int) -> HGDIOBJ;
    pub fn SetBkMode(hdc: HDC, mode: c_int) -> c_int;
    pub fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    pub fn SelectObject(hdc: HDC, h: HGDIOBJ) -> HGDIOBJ;
}

// --- secur32 / advapi32 LSA (NTAPI → extern "system") ---
extern "system" {
    pub fn LsaRegisterLogonProcess(
        LogonProcessName: PLSA_STRING,
        LsaHandle: PHANDLE,
        SecurityMode: PLSA_OPERATIONAL_MODE,
    ) -> NTSTATUS;
    pub fn LsaLookupAuthenticationPackage(
        LsaHandle: HANDLE,
        PackageName: PLSA_STRING,
        AuthenticationPackage: *mut ULONG,
    ) -> NTSTATUS;
    pub fn LsaLogonUser(
        LsaHandle: HANDLE,
        OriginName: PLSA_STRING,
        LogonType: SECURITY_LOGON_TYPE,
        AuthenticationPackage: ULONG,
        AuthenticationInformation: PVOID,
        AuthenticationInformationLength: ULONG,
        LocalGroups: PTOKEN_GROUPS,
        SourceContext: PTOKEN_SOURCE,
        ProfileBuffer: *mut PVOID,
        ProfileBufferLength: *mut ULONG,
        LogonId: PLUID,
        Token: PHANDLE,
        Quotas: PQUOTA_LIMITS,
        SubStatus: PNTSTATUS,
    ) -> NTSTATUS;
    pub fn LsaCallAuthenticationPackage(
        LsaHandle: HANDLE,
        AuthenticationPackage: ULONG,
        ProtocolSubmitBuffer: PVOID,
        SubmitBufferLength: ULONG,
        ProtocolReturnBuffer: *mut PVOID,
        ReturnBufferLength: *mut ULONG,
        ProtocolStatus: PNTSTATUS,
    ) -> NTSTATUS;
    pub fn LsaClose(ObjectHandle: HANDLE) -> NTSTATUS;
    pub fn LsaDeregisterLogonProcess(LsaHandle: HANDLE) -> NTSTATUS;
    pub fn LsaFreeMemory(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaFreeReturnBuffer(Buffer: PVOID) -> NTSTATUS;
    pub fn LsaOpenPolicy(
        SystemName: PLSA_UNICODE_STRING,
        ObjectAttributes: PVOID,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: PHANDLE,
    ) -> NTSTATUS;
    pub fn LsaRetrievePrivateData(
        PolicyHandle: HANDLE,
        KeyName: PLSA_UNICODE_STRING,
        PrivateData: *mut PLSA_UNICODE_STRING,
    ) -> NTSTATUS;
    pub fn AllocateLocallyUniqueId(Luid: PLUID) -> BOOL;
}

// `LsaLookupAuthenticationPackage`'s SECURITY_LOGON_TYPE — alias used above.
pub type SECURITY_LOGON_TYPE = ULONG;

// RtlGetProcessHeap() is an inline (NtCurrentPeb()->ProcessHeap) in the NT headers;
// it returns the same handle as kernel32 GetProcessHeap(). Faithful shim.
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    GetProcessHeap()
}

// RtlSecureZeroMemory inline (no link symbol): volatile-zero `cnt` bytes at `ptr`,
// returns ptr. Faithful to the strsafe/winnt inline.
#[inline(never)]
pub unsafe fn SecureZeroMemory(ptr: PVOID, cnt: usize) -> PVOID {
    let p = ptr as *mut u8;
    let mut i: usize = 0;
    while i < cnt {
        core::ptr::write_volatile(p.add(i), 0u8);
        i += 1;
    }
    ptr
}

extern "C" {
    // --- msvcrt CRT ---
    pub fn wcstoul(nptr: *const u16, endptr: *mut *mut u16, base: c_int) -> c_ulong;
    pub fn wcslen(s: *const u16) -> usize;
    pub fn wcscpy(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn wcscmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16;
    pub fn wcscat(dst: *mut u16, src: *const u16) -> *mut u16;
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> c_int;
    pub fn wsprintfW(buf: PWSTR, fmt: PCWSTR, ...) -> c_int;
}
