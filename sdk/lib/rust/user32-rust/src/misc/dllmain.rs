use win32k_sys::win32k::*;
// TEB/PEB self-pointer inlines (ntuser.h FORCEINLINE) live in crate::internal.
use crate::internal::{NtCurrentPeb, NtCurrentTeb};
use core::ffi::c_void;

// ---- debug-print macros (ReactOS wine debug — no-ops in the cohab build) ----

// ---- constants not present in the win32k bindgen surface ----
// ntstatus.h
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;
const STATUS_NO_MEMORY: NTSTATUS = 0xC0000017u32 as i32;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as i32;
// winbase.h: TLS_OUT_OF_INDEXES
const TLS_OUT_OF_INDEXES: DWORD = 0xFFFFFFFF;
// ntuser.h: `USER_VERSION MAKELONG(0x0000, 0x0005)` == 0x0005_0000
const USER_VERSION: ULONG = 0x0005_0000;
// winuser.h: IDI_* := MAKEINTRESOURCEW(n) — passed to LoadImageW as LPCWSTR.
const IDI_APPLICATION: LPCWSTR = 32512usize as LPCWSTR;
const IDI_HAND: LPCWSTR = 32513usize as LPCWSTR;
const IDI_QUESTION: LPCWSTR = 32514usize as LPCWSTR;
const IDI_EXCLAMATION: LPCWSTR = 32515usize as LPCWSTR;
const IDI_ASTERISK: LPCWSTR = 32516usize as LPCWSTR;
const IDI_WINLOGO: LPCWSTR = 32517usize as LPCWSTR;

// winmsg.h: `USERSRV_API_CONNECTINFO := USERCONNECT`.
pub type USERSRV_API_CONNECTINFO = USERCONNECT;

// ntdll.h: `NTSTATUS NT_SUCCESS(NTSTATUS) := (Status >= 0)`.
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ntuser.h: `GetWin32ClientInfo() := (PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo)`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}

// user_x.h: `SharedPtrToUser(Ptr) := (Ptr - gSharedInfo.ulSharedDelta)`.
#[inline]
unsafe fn SharedPtrToUser(Ptr: PVOID) -> PVOID {
    (Ptr as ULONG_PTR - gSharedInfo.ulSharedDelta) as PVOID
}

// winnt.h: `NtCurrentProcess() := (HANDLE)-1`.
#[inline]
fn NtCurrentProcess() -> HANDLE {
    -1isize as HANDLE
}


// user32 imm dispatch table + helpers owned by misc/imm.rs.
use crate::misc::imm::{ghImm32, gImmApiEntries, InitializeImmEntryTable};
// CRITICAL_SECTION type aliases (winbase glue) + the crit-section globals owned by
// sibling modules (windows/accel.rs, misc/usrapihk.rs).
use crate::winbase::{CRITICAL_SECTION, LPCRITICAL_SECTION};
use crate::windows::accel::U32AccelCacheLock;
use crate::misc::usrapihk::gcsUserApiHook;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    // ntoskrnl/ntdll cdecl variadic debug print.
    fn DbgPrint(Format: *const i8, ...) -> ULONG;
    // CRT cdecl (`_wcslen`, no @N) — was wrongly in the "system" block; only
    // linkable before because its callers were UB-folded (see note below).
    fn wcslen(s: *const u16) -> usize;
}

extern "system" {
    // gdi32 stdcall export (`_TranslateCharsetInfo@12`).
    fn TranslateCharsetInfo(lpSrc: *mut DWORD, lpCs: PVOID, dwFlags: DWORD) -> BOOL;
    fn FreeLibrary(hModule: HMODULE) -> i32;
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn GetObjectW(h: HANDLE, c: core::ffi::c_int, pv: PVOID) -> core::ffi::c_int;
    fn LoadBitmapW(hInstance: HINSTANCE, lpBitmapName: LPCWSTR) -> HBITMAP;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;

    // ── ntdll exports (NTAPI) ──
    fn NtClose(Handle: HANDLE) -> NTSTATUS;
    fn NtOpenKey(KeyHandle: PHANDLE, DesiredAccess: ACCESS_MASK, ObjectAttributes: POBJECT_ATTRIBUTES) -> NTSTATUS;
    fn NtQueryValueKey(
        KeyHandle: HANDLE,
        ValueName: PUNICODE_STRING,
        KeyValueInformationClass: KEY_VALUE_INFORMATION_CLASS,
        KeyValueInformation: PVOID,
        Length: ULONG,
        ResultLength: PULONG,
    ) -> NTSTATUS;
    fn ZwCallbackReturn(OutputBuffer: PVOID, OutputLength: ULONG, Status: NTSTATUS) -> NTSTATUS;
    fn RtlInitializeCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlZeroMemory(Destination: PVOID, Length: SIZE_T);

    // ── kernel32 exports ──
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> PVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: PVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn TlsAlloc() -> DWORD;
    fn TlsFree(dwTlsIndex: DWORD) -> BOOL;
    fn InitializeCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);
    fn DeleteCriticalSection(lpCriticalSection: LPCRITICAL_SECTION);

    // ── gdi32 exports ──
    fn ExtTextOutW(hdc: HDC, x: INT, y: INT, options: UINT, lprect: *const RECT, lpString: LPCWSTR, c: UINT, lpDx: *const INT) -> BOOL;
    fn GdiDllInitialize(hDll: HANDLE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL;
    fn GdiProcessSetup() -> BOOL;
    fn GdiPrinterThunk(pumpd: PVOID, pvOut: PVOID, cjOut: ULONG) -> ULONG;

    // ── user32's OWN exports / internal helpers (sibling crate modules) ──
    fn LoadImageW(hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: INT, cy: INT, fuLoad: UINT) -> HANDLE;
    fn GetSystemMetrics(nIndex: INT) -> INT;
    fn RegisterClientPFN() -> NTSTATUS;
}

// The conventions below MUST match the sibling-module definitions exactly.
// A decl/def calling-convention mismatch on the same symbol name is UB to
// LLVM: on i386 it silently DELETED the whole MessageInit/MenuInit/
// InitializeCriticalSection(U32AccelCacheLock) branch of
// ClientThreadSetupHelper (no ud2 — the branch folds away), leaving
// U32AccelCacheLock a zeroed CS whose first EnterCriticalSection deadlocks
// (explorer desktop thread hung in LoadAcceleratorsW → desktop view never
// shown → no icons). MessageInit/MessageCleanup are FASTCALL on x86
// (message.rs); MenuInit/MenuCleanup/DeleteFrameBrushes are cdecl.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MessageInit() -> BOOL;
    fn MessageCleanup();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MessageInit() -> BOOL;
    fn MessageCleanup();
}
extern "C" {
    fn MenuInit() -> BOOL;
    fn MenuCleanup();
    fn DeleteFrameBrushes();
}

// `imm32.dll` wide literal (dllmain.c: `User32GetSystemFilePath(.., L"imm32.dll")`).
const IMM32_DLL_W: &[u16] = &[
    0x0069, 0x006d, 0x006d, 0x0033, 0x0032, 0x002e, 0x0064, 0x006c, 0x006c, 0x0000,
];
#[inline]
unsafe fn L_imm32_dll() -> LPCWSTR { IMM32_DLL_W.as_ptr() }

// misc/imm.c: `User32GetSystemFilePath` is module-local in imm.rs; reproduced here
// (GetSystemDirectoryW + "\\" + filename) for the DllMain IMM init path.
extern "system" {
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
}

// ── strsafe.h inline helpers (no linkable symbol — reproduced faithfully as
//   bounded NUL-terminated copy/concat on UTF-16 buffers; mirror misc/imm.rs).
unsafe fn StringCchCopyW(dst: PWSTR, cch: SIZE_T, src: PCWSTR) -> HRESULT {
    let cch = cch as usize;
    if cch == 0 {
        return 0x80070057u32 as i32 as HRESULT; /* HRESULT_FROM_WIN32(ERROR_INVALID_PARAMETER) */
    }
    let mut i: usize = 0;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
    if *src.add(i) != 0 {
        0x8007007Au32 as i32 as HRESULT /* STRSAFE_E_INSUFFICIENT_BUFFER */
    } else {
        0 /* S_OK */
    }
}
unsafe fn StringCchCatW(dst: PWSTR, cch: SIZE_T, src: PCWSTR) -> HRESULT {
    let cch = cch as usize;
    let mut len: usize = 0;
    while len < cch && *dst.add(len) != 0 {
        len += 1;
    }
    if len >= cch {
        return 0x8007007Au32 as i32 as HRESULT;
    }
    StringCchCopyW(dst.add(len), (cch - len) as SIZE_T, src)
}
unsafe fn User32GetSystemFilePath(lpBuffer: PWSTR, cchBuffer: SIZE_T, pszFileName: PCWSTR) -> HRESULT {
    let length: UINT = GetSystemDirectoryW(lpBuffer, cchBuffer as UINT);
    if length != 0 && (length as SIZE_T) < cchBuffer {
        StringCchCatW(lpBuffer, cchBuffer, w!("\\").as_ptr() as PCWSTR);
        return StringCchCatW(lpBuffer, cchBuffer, pszFileName);
    }
    StringCchCopyW(lpBuffer, cchBuffer, pszFileName);
    0
}

// IMM_FN(name) := gImmApiEntries.name (user32p.h dispatch-table macro).
#[inline]
unsafe fn IMM_FN_ImmRegisterClient(psi: LPVOID, hImm: HINSTANCE) -> BOOL {
    (gImmApiEntries.ImmRegisterClient)(psi, hImm)
}
#[inline]
unsafe fn IMM_FN_ImmProcessKey(hWnd: HWND, hKL: HKL, vKey: UINT, lParam: LPARAM, dwHotKeyID: DWORD) -> DWORD {
    (gImmApiEntries.ImmProcessKey)(hWnd, hKL, vKey, lParam, dwHotKeyID)
}
#[inline]
unsafe fn ImmLoadLayout(hKL: HKL, piiex: PIMEINFOEX) -> BOOL {
    (gImmApiEntries.ImmLoadLayout)(hKL, piiex)
}

// MSVC intrinsic: `_InterlockedExchange8(target, value)` — atomic xchg, returns prior.
#[inline]
unsafe fn _InterlockedExchange8(target: PCHAR, value: CHAR) -> CHAR {
    let p = target as *mut core::sync::atomic::AtomicI8;
    (*p).swap(value, core::sync::atomic::Ordering::SeqCst)
}

// ---- file-scope preamble (translated) ----
pub const MAX_USER_MODE_DRV_BUFFER: INT = 526;

#[repr(C)]
pub struct _UMPDPKTHEAD {
    pub Size: INT,
    pub Index: INT,
    pub RetSize: INT,
    pub Reserved: DWORD,
    pub humpd: HUMPD,
    pub Buffer: __IncompleteArrayField<ULONG_PTR>,
}
pub type UMPDPKTHEAD = _UMPDPKTHEAD;
pub type PUMPDPKTHEAD = *mut _UMPDPKTHEAD;

pub const KEY_LENGTH: usize = 1024;

pub static mut User32TlsIndex: ULONG = 0;
#[no_mangle]
pub static mut User32Instance: HINSTANCE = core::ptr::null_mut();
pub static mut g_ppi: PPROCESSINFO = core::ptr::null_mut();
#[no_mangle]
pub static mut gSharedInfo: SHAREDINFO = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut gpsi: PSERVERINFO = core::ptr::null_mut();
pub static mut gHandleTable: PUSER_HANDLE_TABLE = core::ptr::null_mut();
pub static mut gHandleEntries: PUSER_HANDLE_ENTRY = core::ptr::null_mut();
pub static mut gfLogonProcess: BOOLEAN = 0u8;
#[no_mangle]
pub static mut gfServerProcess: BOOLEAN = 0u8;
pub static mut gfFirstThread: BOOLEAN = 1u8;
pub static mut hIconSmWindows: HICON = core::ptr::null_mut();
pub static mut hIconWindows: HICON = core::ptr::null_mut();
pub static mut szAppInit: [WCHAR; KEY_LENGTH] = [0; KEY_LENGTH];

// u32cb.h kernel-callback dispatch table — 20 entries, kept in index order (the
// kernel indexes KernelCallbackTable by USER32_CALLBACK id). Callbacks defined in
// sibling modules are referenced via the local extern block below; the ones defined
// in this file are referenced directly.
extern "system" {
    fn User32CallWindowProcFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallSendAsyncProcForKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32LoadSysMenuTemplateForKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32SetupDefaultCursors(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallHookProcFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallEventProcFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallLoadMenuFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallClientLoadLibraryFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallCopyImageFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallDDEPostFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
    fn User32CallDDEGetFromKernel(a: PVOID, n: ULONG) -> NTSTATUS;
}

pub static mut apfnDispatch: [PVOID; USER32_CALLBACK_COUNT as usize] = [
    User32CallWindowProcFromKernel as PVOID,            // 0  WINDOWPROC
    User32CallSendAsyncProcForKernel as PVOID,          // 1  SENDASYNCPROC
    User32LoadSysMenuTemplateForKernel as PVOID,        // 2  LOADSYSMENUTEMPLATE
    User32SetupDefaultCursors as PVOID,                 // 3  LOADDEFAULTCURSORS
    User32CallHookProcFromKernel as PVOID,              // 4  HOOKPROC
    User32CallEventProcFromKernel as PVOID,             // 5  EVENTPROC
    User32CallLoadMenuFromKernel as PVOID,              // 6  LOADMENU
    User32CallClientThreadSetupFromKernel as PVOID,     // 7  CLIENTTHREADSTARTUP
    User32CallClientLoadLibraryFromKernel as PVOID,     // 8  CLIENTLOADLIBRARY
    User32CallGetCharsetInfo as PVOID,                  // 9  GETCHARSETINFO
    User32CallCopyImageFromKernel as PVOID,             // 10 COPYIMAGE
    User32CallSetWndIconsFromKernel as PVOID,           // 11 SETWNDICONS
    User32DeliverUserAPC as PVOID,                      // 12 DELIVERUSERAPC
    User32CallDDEPostFromKernel as PVOID,               // 13 DDEPOST
    User32CallDDEGetFromKernel as PVOID,                // 14 DDEGET
    User32CallOBMFromKernel as PVOID,                   // 15 SETOBM
    User32CallLPKFromKernel as PVOID,                   // 16 LPK
    User32CallUMPDFromKernel as PVOID,                  // 17 UMPD
    User32CallImmProcessKeyFromKernel as PVOID,         // 18 IMMPROCESSKEY
    User32CallImmLoadLayoutFromKernel as PVOID,         // 19 IMMLOADLAYOUT
];

unsafe fn GetDllList() -> BOOL {
    let mut Status: NTSTATUS;
    let mut Attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut bRet: BOOL = 0;
    let mut bLoad: BOOL = 0;
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut kvpInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();

    let szKeyName_buf: &[u16] = &[
        0x005c,0x0052,0x0065,0x0067,0x0069,0x0073,0x0074,0x0072,0x0079,0x005c,
        0x004d,0x0061,0x0063,0x0068,0x0069,0x006e,0x0065,0x005c,
        0x0053,0x006f,0x0066,0x0074,0x0077,0x0061,0x0072,0x0065,0x005c,
        0x004d,0x0069,0x0063,0x0072,0x006f,0x0073,0x006f,0x0066,0x0074,0x005c,
        0x0057,0x0069,0x006e,0x0064,0x006f,0x0077,0x0073,0x0020,0x004e,0x0054,0x005c,
        0x0043,0x0075,0x0072,0x0072,0x0065,0x006e,0x0074,0x0056,0x0065,0x0072,0x0073,0x0069,0x006f,0x006e,0x005c,
        0x0057,0x0069,0x006e,0x0064,0x006f,0x0077,0x0073,0x0000u16,
    ];
    let mut szKeyName: _UNICODE_STRING = core::mem::zeroed();
    szKeyName.Length = ((szKeyName_buf.len() - 1) * 2) as u16;
    szKeyName.MaximumLength = (szKeyName_buf.len() * 2) as u16;
    szKeyName.Buffer = szKeyName_buf.as_ptr() as *mut u16;

    let szLoadName_buf: &[u16] = &[
        0x004c,0x006f,0x0061,0x0064,0x0041,0x0070,0x0070,0x0049,0x006e,0x0069,0x0074,
        0x005f,0x0044,0x004c,0x004c,0x0073,0x0000u16,
    ];
    let mut szLoadName: _UNICODE_STRING = core::mem::zeroed();
    szLoadName.Length = ((szLoadName_buf.len() - 1) * 2) as u16;
    szLoadName.MaximumLength = (szLoadName_buf.len() * 2) as u16;
    szLoadName.Buffer = szLoadName_buf.as_ptr() as *mut u16;

    let szDllsName_buf: &[u16] = &[
        0x0041,0x0070,0x0070,0x0049,0x006e,0x0069,0x0074,
        0x005f,0x0044,0x004c,0x004c,0x0073,0x0000u16,
    ];
    let mut szDllsName: _UNICODE_STRING = core::mem::zeroed();
    szDllsName.Length = ((szDllsName_buf.len() - 1) * 2) as u16;
    szDllsName.MaximumLength = (szDllsName_buf.len() * 2) as u16;
    szDllsName.Buffer = szDllsName_buf.as_ptr() as *mut u16;

    Attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    Attributes.RootDirectory = core::ptr::null_mut();
    Attributes.Attributes = OBJ_CASE_INSENSITIVE;
    Attributes.ObjectName = &mut szKeyName;
    Attributes.SecurityDescriptor = core::ptr::null_mut();
    Attributes.SecurityQualityOfService = core::ptr::null_mut();

    'end: {
        Status = NtOpenKey(&mut hKey, KEY_READ, &mut Attributes);
        if NT_SUCCESS(Status) {
            dwSize = (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()
                + core::mem::size_of::<DWORD>()) as DWORD;
            kvpInfo = HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
            if kvpInfo.is_null() {
                break 'end;
            }

            Status = NtQueryValueKey(
                hKey,
                &mut szLoadName,
                KeyValuePartialInformation,
                kvpInfo as *mut _,
                dwSize,
                &mut dwSize,
            );
            if !NT_SUCCESS(Status) {
                break 'end;
            }

            core::ptr::copy(
                core::ptr::addr_of_mut!((*kvpInfo).Data) as *const u8,
                &mut bLoad as *mut BOOL as *mut u8,
                (*kvpInfo).DataLength as usize,
            );

            HeapFree(GetProcessHeap(), 0, kvpInfo as *mut _);
            kvpInfo = core::ptr::null_mut();

            if bLoad != 0 {
                Status = NtQueryValueKey(
                    hKey,
                    &mut szDllsName,
                    KeyValuePartialInformation,
                    core::ptr::null_mut(),
                    0,
                    &mut dwSize,
                );
                if Status != STATUS_BUFFER_TOO_SMALL {
                    break 'end;
                }

                kvpInfo = HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T) as PKEY_VALUE_PARTIAL_INFORMATION;
                if kvpInfo.is_null() {
                    break 'end;
                }

                Status = NtQueryValueKey(
                    hKey,
                    &mut szDllsName,
                    KeyValuePartialInformation,
                    kvpInfo as *mut _,
                    dwSize,
                    &mut dwSize,
                );
                if NT_SUCCESS(Status) {
                    let lpBuffer = core::ptr::addr_of_mut!((*kvpInfo).Data) as LPWSTR;
                    if *lpBuffer != 0 {
                        let bytesToCopy: INT;
                        let nullPos: INT;

                        let a = (*kvpInfo).DataLength as INT;
                        let b = (KEY_LENGTH * core::mem::size_of::<u16>()) as INT;
                        bytesToCopy = if a < b { a } else { b };

                        if bytesToCopy != 0 {
                            core::ptr::copy(
                                core::ptr::addr_of_mut!((*kvpInfo).Data) as *const u8,
                                (*(&raw mut szAppInit)).as_mut_ptr() as *mut u8,
                                bytesToCopy as usize,
                            );

                            nullPos = (bytesToCopy / core::mem::size_of::<u16>() as INT) - 1;

                            szAppInit[nullPos as usize] = 0;

                            bRet = 1;
                        }
                    }
                }
            }
        }
    }

    if !hKey.is_null() {
        NtClose(hKey);
    }

    if !kvpInfo.is_null() {
        HeapFree(GetProcessHeap(), 0, kvpInfo as *mut _);
    }

    bRet
}

unsafe fn LoadAppInitDlls() {
    szAppInit[0] = 0;

    if GetDllList() != 0 {
        let mut buffer: [u16; KEY_LENGTH as usize] = [0; KEY_LENGTH as usize];
        let ptr: LPWSTR;
        let mut i: usize;

        core::ptr::copy_nonoverlapping(
            (*(&raw const szAppInit)).as_ptr(),
            buffer.as_mut_ptr(),
            KEY_LENGTH as usize,
        );

        i = 0;
        while i < KEY_LENGTH as usize {
            if buffer[i] == b' ' as u16 || buffer[i] == b',' as u16 {
                buffer[i] = 0;
            }
            i += 1;
        }

        i = 0;
        while i < KEY_LENGTH as usize {
            if buffer[i] == 0 {
                i += 1;
            } else {
                let ptr = buffer.as_mut_ptr().add(i);
                i += wcslen(ptr) as usize;
                LoadLibraryW(ptr);
            }
        }
    }
}

unsafe fn UnloadAppInitDlls() {
    if szAppInit[0] != 0 {
        let mut buffer: [u16; 1024] = [0; 1024];
        let mut hModule: HMODULE;
        let mut ptr: LPWSTR;
        let mut i: usize;

        core::ptr::copy_nonoverlapping(
            (*(&raw const szAppInit)).as_ptr(),
            buffer.as_mut_ptr(),
            1024,
        );

        i = 0;
        while i < 1024 {
            if buffer[i] == b' ' as u16 || buffer[i] == b',' as u16 {
                buffer[i] = 0;
            }
            i += 1;
        }

        i = 0;
        while i < 1024 {
            if buffer[i] == 0 {
                i += 1;
            } else {
                ptr = buffer.as_mut_ptr().add(i);
                i += wcslen(ptr) as usize;
                hModule = GetModuleHandleW(ptr);
                FreeLibrary(hModule);
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ClientThreadSetupHelper(IsCallback: BOOL) -> BOOL {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();
    let IsFirstThread: BOOLEAN =
        _InterlockedExchange8(core::ptr::addr_of_mut!(gfFirstThread) as PCHAR, FALSE as CHAR) as BOOLEAN;

    if IsFirstThread != 0 {
        GdiProcessSetup();
    }

    if (*ClientInfo).CI_flags & CI_INITTHREAD as ULONG_PTR != 0 {
        return FALSE as BOOL;
    }

    if gfServerProcess != 0 && IsFirstThread != 0 {
        let Status: NTSTATUS;
        let mut UserCon: USERCONNECT = core::mem::zeroed();

        RtlZeroMemory(
            core::ptr::addr_of_mut!(UserCon) as PVOID,
            core::mem::size_of::<USERCONNECT>() as SIZE_T,
        );

        UserCon.ulVersion = USER_VERSION;

        Status = NtUserProcessConnect(
            -1isize as HANDLE,
            core::ptr::addr_of_mut!(UserCon),
            core::mem::size_of::<USERCONNECT>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            return FALSE as BOOL;
        }

        g_ppi = (*ClientInfo).ppi;
        gSharedInfo = UserCon.siClient;
        gpsi = gSharedInfo.psi;
        gHandleTable = gSharedInfo.aheList as PUSER_HANDLE_TABLE;
        // WoW64: the shared handle table is 64-bit-laid-out, so `handles` is at the
        // wrong offset here and gHandleEntries would be a garbage pointer.
        // GetUser32Handle() reads entries through crate::wow64 instead.
        crate::wow64::UserWow64Init();
        if !crate::wow64::IsWow64() {
            gHandleEntries = SharedPtrToUser((*gHandleTable).handles as PVOID) as PUSER_HANDLE_ENTRY;
        }
    }

    if RegisterClientPFN() == 0 {
        return FALSE as BOOL;
    }

    (*ClientInfo).CI_flags |= CI_INITTHREAD as ULONG_PTR;

    if IsFirstThread != 0 {
        User32TlsIndex = TlsAlloc();
        if User32TlsIndex == TLS_OUT_OF_INDEXES {
            return FALSE as BOOL;
        }

        if !gpsi.is_null() {
            if MessageInit() != 0 {
                if MenuInit() != 0 {
                    InitializeCriticalSection(core::ptr::addr_of_mut!(U32AccelCacheLock));
                    LoadAppInitDlls();
                    return TRUE as BOOL;
                }
                MessageCleanup();
            }

            TlsFree(User32TlsIndex);
            return FALSE as BOOL;
        }
    }

    TRUE as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ClientThreadSetup() -> BOOL {
    1
}

#[no_mangle]
pub unsafe extern "system" fn Init(UserCon: PUSERCONNECT) -> BOOL {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));

    /* Initialize callback table in PEB data */
    (*NtCurrentPeb()).KernelCallbackTable = (*(&raw mut apfnDispatch)).as_mut_ptr() as *mut core::ffi::c_void;
    (*NtCurrentPeb()).PostProcessInitRoutine = None;

    // This is a HACK!! //
    gfServerProcess = 0u8;
    gfFirstThread = 1u8;
    //// End of HACK!! ///

    if gfServerProcess == 0u8 {
        {
            RtlZeroMemory(UserCon as *mut core::ffi::c_void, core::mem::size_of::<USERCONNECT>() as SIZE_T);

            /* Minimal setup of the connect info structure */
            (*UserCon).ulVersion = USER_VERSION;

            /* Connect to win32k */
            Status = NtUserProcessConnect(
                NtCurrentProcess(),
                UserCon,
                core::mem::size_of::<USERCONNECT>() as ULONG,
            );
            if !NT_SUCCESS(Status) {
                return 0;
            }
        }

        /* Retrieve data */
        g_ppi = (*GetWin32ClientInfo()).ppi;
        gSharedInfo = (*UserCon).siClient;
        gpsi = gSharedInfo.psi;
        gHandleTable = gSharedInfo.aheList as PUSER_HANDLE_TABLE;
        crate::wow64::UserWow64Init();
        if !gHandleTable.is_null() && !crate::wow64::IsWow64() {
            gHandleEntries = SharedPtrToUser((*gHandleTable).handles as *mut core::ffi::c_void) as PUSER_HANDLE_ENTRY;
        }
    }

    if ClientThreadSetupHelper(0) == 0 {
        return 0;
    }

    NT_SUCCESS(Status) as BOOL
}

unsafe fn Cleanup() {
    UnloadAppInitDlls();
    DeleteCriticalSection(core::ptr::addr_of_mut!(U32AccelCacheLock));
    MenuCleanup();
    MessageCleanup();
    TlsFree(User32TlsIndex);
    DeleteFrameBrushes();
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hDll: HANDLE,
    dwReason: ULONG,
    pReserved: PVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: ULONG = 1;
    const DLL_PROCESS_DETACH: ULONG = 0;

    match dwReason {
        DLL_PROCESS_ATTACH => {
            let mut ConnectInfo: USERSRV_API_CONNECTINFO = core::mem::zeroed();
            DbgPrint(
                b"[user32] DllMain PROCESS_ATTACH PID=%lu\n\0".as_ptr() as *const i8,
                (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
            );

            User32Instance = hDll as HINSTANCE;

            TRACE!(b"Checkpoint (call Init)\n\0".as_ptr());
            if Init(core::ptr::addr_of_mut!(ConnectInfo) as PUSERCONNECT) == 0 {
                return 0;
            }

            if gfServerProcess == 0 {
                let mut hImm32: HINSTANCE = core::ptr::null_mut();

                if !gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0 {
                    let mut szImmFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
                    InitializeImmEntryTable();
                    User32GetSystemFilePath(
                        szImmFile.as_mut_ptr(),
                        szImmFile.len() as SIZE_T,
                        L_imm32_dll(),
                    );
                    hImm32 = GetModuleHandleW(szImmFile.as_ptr());
                }

                if IMM_FN_ImmRegisterClient(core::ptr::addr_of_mut!(gSharedInfo) as LPVOID, hImm32) == 0 {
                    return 0;
                }
            }
        }

        DLL_PROCESS_DETACH => {
            if !ghImm32.is_null() {
                FreeLibrary(ghImm32);
            }

            Cleanup();
        }

        _ => {}
    }

    let gdiResult: BOOL = GdiDllInitialize(hDll, dwReason, pReserved);
    if dwReason == DLL_PROCESS_ATTACH {
        DbgPrint(
            b"[user32] DllMain PROCESS_ATTACH done, GdiDllInitialize=%d PID=%lu\n\0".as_ptr()
                as *const i8,
            gdiResult,
            (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
        );
    }
    gdiResult
}
#[no_mangle]
pub unsafe extern "system" fn User32CallClientThreadSetupFromKernel(
    Arguments: *mut core::ffi::c_void,
    ArgumentLength: u32,
) -> i32 {
    ZwCallbackReturn(core::ptr::null_mut(), 0, STATUS_SUCCESS)
}
#[no_mangle]
pub unsafe extern "system" fn User32CallGetCharsetInfo(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let pgci: PGET_CHARSET_INFO = Arguments as PGET_CHARSET_INFO;

    let Ret = TranslateCharsetInfo(
        (*pgci).Locale as ULONG_PTR as *mut DWORD,
        core::ptr::addr_of_mut!((*pgci).Cs) as PVOID,
        TCI_SRCLOCALE,
    );

    ZwCallbackReturn(
        Arguments,
        ArgumentLength,
        if Ret != 0 { STATUS_SUCCESS } else { STATUS_UNSUCCESSFUL },
    )
}
#[no_mangle]
pub unsafe extern "system" fn User32CallSetWndIconsFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let Common: PSETWNDICONS_CALLBACK_ARGUMENTS = Arguments as PSETWNDICONS_CALLBACK_ARGUMENTS;

    if gpsi.is_null() || crate::wow64::psi::hicon_sm_windows().is_null() {
        (*Common).hIconSample = LoadImageW(
            0 as HINSTANCE,
            IDI_APPLICATION,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconHand = LoadImageW(
            0 as HINSTANCE,
            IDI_HAND,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconQuestion = LoadImageW(
            0 as HINSTANCE,
            IDI_QUESTION,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconBang = LoadImageW(
            0 as HINSTANCE,
            IDI_EXCLAMATION,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconNote = LoadImageW(
            0 as HINSTANCE,
            IDI_ASTERISK,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconWindows = LoadImageW(
            0 as HINSTANCE,
            IDI_WINLOGO,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HICON;
        (*Common).hIconSmWindows = LoadImageW(
            0 as HINSTANCE,
            IDI_WINLOGO,
            IMAGE_ICON,
            GetSystemMetrics(SM_CXSMICON as INT),
            GetSystemMetrics(SM_CYSMICON as INT),
            0,
        ) as HICON;
        hIconWindows = (*Common).hIconWindows;
        hIconSmWindows = (*Common).hIconSmWindows;
    }
    ERR!(
        "hIconSmWindows %p hIconWindows %p \n",
        hIconSmWindows,
        hIconWindows,
    );
    ZwCallbackReturn(Arguments, ArgumentLength, STATUS_SUCCESS)
}
#[no_mangle]
pub unsafe extern "system" fn User32DeliverUserAPC(
    Arguments: *mut core::ffi::c_void,
    ArgumentLength: u32,
) -> i32 {
    ZwCallbackReturn(core::ptr::null_mut(), 0, STATUS_SUCCESS)
}
#[no_mangle]
pub unsafe extern "system" fn User32CallOBMFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {

    let mut bmp: BITMAP = core::mem::zeroed();
    let Common = Arguments as PSETOBM_CALLBACK_ARGUMENTS;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_CLOSE as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_CLOSE as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_CLOSE as usize].cy = bmp.bmHeight;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_MNARROW as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_MNARROW as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_MNARROW as usize].cy = bmp.bmHeight;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_DNARROW as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_DNARROW as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_DNARROW as usize].cy = bmp.bmHeight;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_DNARROWI as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_DNARROWI as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_DNARROWI as usize].cy = bmp.bmHeight;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_UPARROW as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_UPARROW as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_UPARROW as usize].cy = bmp.bmHeight;

    GetObjectW(
        LoadBitmapW(core::ptr::null_mut(), OBM_UPARROWI as usize as LPCWSTR) as HANDLE,
        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
        core::ptr::addr_of_mut!(bmp) as PVOID,
    );
    (*Common).oembmi[OBI_UPARROWI as usize].cx = bmp.bmWidth;
    (*Common).oembmi[OBI_UPARROWI as usize].cy = bmp.bmHeight;

    ZwCallbackReturn(Arguments, ArgumentLength, STATUS_SUCCESS)
}
#[no_mangle]
pub unsafe extern "system" fn User32CallLPKFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let bResult: BOOL;
    let Argument: PLPK_CALLBACK_ARGUMENTS;

    Argument = Arguments as PLPK_CALLBACK_ARGUMENTS;

    (*Argument).lpString =
        ((*Argument).lpString as ULONG_PTR + Argument as ULONG_PTR) as LPWSTR;

    bResult = ExtTextOutW(
        (*Argument).hdc,
        (*Argument).x,
        (*Argument).y,
        (*Argument).flags,
        if (*Argument).bRect != 0 {
            core::ptr::addr_of_mut!((*Argument).rect) as *const RECT
        } else {
            core::ptr::null_mut()
        },
        (*Argument).lpString,
        (*Argument).count,
        core::ptr::null_mut(),
    );

    ZwCallbackReturn(
        core::ptr::addr_of!(bResult) as PVOID,
        core::mem::size_of::<BOOL>() as ULONG,
        STATUS_SUCCESS,
    )
}
#[no_mangle]
pub unsafe extern "system" fn User32CallUMPDFromKernel(Arguments: PVOID, ArgumentLength: ULONG) -> NTSTATUS {
    let mut Buffer: [DWORD; MAX_USER_MODE_DRV_BUFFER as usize] = [0; MAX_USER_MODE_DRV_BUFFER as usize];
    let mut cbSize: INT = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let pkt: PUMPDPKTHEAD = Arguments as PUMPDPKTHEAD;
    let mut pktOut: PUMPDPKTHEAD = core::ptr::null_mut();

    if (*pkt).RetSize as usize <= core::mem::size_of_val(&Buffer) {
        pktOut = Buffer.as_mut_ptr() as PUMPDPKTHEAD;

        if GdiPrinterThunk(pkt as PVOID, pktOut as PVOID, (*pkt).RetSize as ULONG) == GDI_ERROR as ULONG {
            pktOut = core::ptr::null_mut();
            Status = STATUS_UNSUCCESSFUL;
        } else {
            cbSize = (*pkt).RetSize;
        }
    } else {
        Status = STATUS_NO_MEMORY;
    }

    ZwCallbackReturn(pktOut as PVOID, cbSize as ULONG, Status)
}
#[no_mangle]
pub unsafe extern "system" fn User32CallImmProcessKeyFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let Common: PIMMPROCESSKEY_CALLBACK_ARGUMENTS = Arguments as PIMMPROCESSKEY_CALLBACK_ARGUMENTS;
    let mut Result: DWORD = IMM_FN_ImmProcessKey(
        (*Common).hWnd,
        (*Common).hKL,
        (*Common).vKey,
        (*Common).lParam,
        (*Common).dwHotKeyID,
    );

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(Result) as PVOID,
        core::mem::size_of::<DWORD>() as ULONG,
        STATUS_SUCCESS,
    )
}
#[no_mangle]
pub unsafe extern "system" fn User32CallImmLoadLayoutFromKernel(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let Common: PIMMLOADLAYOUT_CALLBACK_ARGUMENTS = Arguments as PIMMLOADLAYOUT_CALLBACK_ARGUMENTS;
    let mut Result: IMMLOADLAYOUT_CALLBACK_OUTPUT = core::mem::zeroed();
    Result.ret = ImmLoadLayout((*Common).hKL, core::ptr::addr_of_mut!(Result.iiex));
    ZwCallbackReturn(
        core::ptr::addr_of_mut!(Result) as PVOID,
        core::mem::size_of::<IMMLOADLAYOUT_CALLBACK_OUTPUT>() as ULONG,
        STATUS_SUCCESS,
    )
}
