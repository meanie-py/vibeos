//! sas.c — the Secure Attention Sequence state machine (logon/logoff/shutdown,
//! the SAS window proc, hotkeys, sound). Fourth/last GINA/SAS/RPC core (rustconv
//! Opus 4.8, integrated). LINK-CHECKED: the live (DUMMY_PSEH) boot path never
//! enters the SAS loop, so only LuidNone (read by wlx.rs) is runtime-live.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::preamble::{
    hAppInstance, wcslen, RtlGetProcessHeap, SetWindowStationUser, WLSession, HKEY_LOCAL_MACHINE,
};
use crate::{wstr, wstr_arr};
use core::ffi::c_void;
use core::ptr::{addr_of, addr_of_mut, null, null_mut};
use winlogon_sys::nt::*;

// ---- base/system/winlogon/resource.h control id; .rc resource ids are not in any
// bindgen island, so this arm bound every WM_COMMAND control id (§9.112a). ----
const IDC_BTNSHTDOWNCOMPUTER: u32 = 303;

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


// Debug macros dropped (as in the other flipped files; call-sites already `!`).

// C macros → helper fns.
#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
#[inline]
fn WLX_LOGGINGOFF(a: DWORD) -> bool {
    a == WLX_SAS_ACTION_LOGOFF || a == WLX_SAS_ACTION_FORCE_LOGOFF
}
#[inline]
fn WLX_SHUTTINGDOWN(a: DWORD) -> bool {
    a == WLX_SAS_ACTION_SHUTDOWN
        || a == WLX_SAS_ACTION_SHUTDOWN_POWER_OFF
        || a == WLX_SAS_ACTION_SHUTDOWN_REBOOT
}
#[inline]
fn WLX_SUSPENDING(a: DWORD) -> bool {
    a == WLX_SAS_ACTION_SHUTDOWN_SLEEP
        || a == WLX_SAS_ACTION_SHUTDOWN_SLEEP2
        || a == WLX_SAS_ACTION_SHUTDOWN_HIBERNATE
}

// FORCEINLINE helpers (no exported symbol).
#[inline]
unsafe fn SetLogonTimestamp(Session: PWLSESSION) {
    NtQuerySystemTime(addr_of_mut!((*Session).LastLogon));
}
#[inline]
unsafe fn SecureZeroMemory(ptr: *mut c_void, cnt: usize) -> *mut c_void {
    let p = ptr as *mut u8;
    let mut i = 0usize;
    while i < cnt {
        core::ptr::write_volatile(p.add(i), 0u8);
        i += 1;
    }
    ptr
}

// GetWindowLongPtrW/SetWindowLongPtrW don't exist on i386 (macros onto the
// non-Ptr variants; LONG_PTR == LONG there) — cfg shims, like shutdown.rs.
#[cfg(target_arch = "x86")]
unsafe fn GetWindowLongPtr(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    GetWindowLongW(hWnd, nIndex) as LONG_PTR
}
#[cfg(not(target_arch = "x86"))]
unsafe fn GetWindowLongPtr(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    GetWindowLongPtrW(hWnd, nIndex)
}
#[cfg(target_arch = "x86")]
unsafe fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hWnd, nIndex, dwNewLong as LONG) as LONG_PTR
}

// ---- substrate: types/consts/APIs winlogon-sys bindgen didn't cover ---------
type SC_HANDLE = *mut c_void;

#[repr(C)]
struct PROFILEINFOW {
    dwSize: DWORD,
    dwFlags: DWORD,
    lpUserName: LPWSTR,
    lpProfilePath: LPWSTR,
    lpDefaultPath: LPWSTR,
    lpServerName: LPWSTR,
    lpPolicyPath: LPWSTR,
    hProfile: HANDLE,
}

#[repr(C)]
struct NETRESOURCEW {
    dwScope: DWORD,
    dwType: DWORD,
    dwDisplayType: DWORD,
    dwUsage: DWORD,
    lpLocalName: LPWSTR,
    lpRemoteName: LPWSTR,
    lpComment: LPWSTR,
    lpProvider: LPWSTR,
}
type LPNETRESOURCE = *mut NETRESOURCEW;

#[repr(C)]
struct SERVICE_STATUS_PROCESS {
    dwServiceType: DWORD,
    dwCurrentState: DWORD,
    dwControlsAccepted: DWORD,
    dwWin32ExitCode: DWORD,
    dwServiceSpecificExitCode: DWORD,
    dwCheckPoint: DWORD,
    dwWaitHint: DWORD,
    dwProcessId: DWORD,
    dwServiceFlags: DWORD,
}

#[repr(C)]
struct TRUSTEE_W {
    pMultipleTrustee: *mut TRUSTEE_W,
    MultipleTrusteeOperation: i32,
    TrusteeForm: i32,
    TrusteeType: i32,
    ptstrName: LPWSTR,
}
#[repr(C)]
struct EXPLICIT_ACCESS_W {
    grfAccessPermissions: DWORD,
    grfAccessMode: i32,
    grfInheritance: DWORD,
    Trustee: TRUSTEE_W,
}
type EXPLICIT_ACCESS = EXPLICIT_ACCESS_W;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC000_0001u32 as NTSTATUS;
const STATUS_NO_MEMORY: NTSTATUS = 0xC000_0017u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;
const SND_ASYNC: DWORD = 0x0001;
const SND_NODEFAULT: DWORD = 0x0002;
const SND_ALIAS: DWORD = 0x0001_0000;
const SE_SHUTDOWN_PRIVILEGE: DWORD = 19;
const SE_ASSIGNPRIMARYTOKEN_PRIVILEGE: DWORD = 3;
const SERVICE_RUNNING: DWORD = 0x4;
const SC_MANAGER_CONNECT: DWORD = 0x0001;
const SC_STATUS_PROCESS_INFO: i32 = 0;
const WN_SUCCESS: DWORD = 0;
const WN_NO_MORE_ENTRIES: DWORD = 259;
const WN_MORE_DATA: DWORD = 234;
const TRUSTEE_IS_SID: i32 = 0;
const TRUSTEE_IS_WELL_KNOWN_GROUP: i32 = 5;
const SET_ACCESS: i32 = 2;
// bindgen TRUE/FALSE are u32; BOOL is i32 → shadow as BOOL for the many
// `BOOL = FALSE` / `return TRUE` sites (the security.rs pattern).
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;
const NO_INHERITANCE: DWORD = 0;
const RESOURCE_REMEMBERED: DWORD = 3;
const RESOURCETYPE_DISK: DWORD = 1;
const PI_NOUI: DWORD = 1;
const SECURITY_DESCRIPTOR_MIN_LENGTH: DWORD = core::mem::size_of::<SECURITY_DESCRIPTOR>() as DWORD;

// resource.h string/dialog IDs.
const IDD_SHUTDOWNCOMPUTER: UINT = 300;
const IDS_APPLYINGYOURPERSONALSETTINGS: UINT = 1679;
const IDS_LOADINGYOURPERSONALSETTINGS: UINT = 1682;
const IDS_CLOSINGNETWORKCONNECTIONS: UINT = 1683;
const IDS_REACTOSISSHUTTINGDOWN: UINT = 1684;
const IDS_SAVEYOURSETTINGS: UINT = 1687;
const IDS_LOGGINGOFF: UINT = 1691;
const IDS_REACTOSISRESTARTING: UINT = 1697;

#[repr(C)]
struct SERVICE_STATUS {
    dwServiceType: DWORD,
    dwCurrentState: DWORD,
    dwControlsAccepted: DWORD,
    dwWin32ExitCode: DWORD,
    dwServiceSpecificExitCode: DWORD,
    dwCheckPoint: DWORD,
    dwWaitHint: DWORD,
}

// WINAPI (stdcall) APIs not in the winlogon-sys glob (userenv/advapi32/mpr/winsvc).
extern "system" {
    fn CreateEnvironmentBlock(lpEnvironment: *mut LPVOID, hToken: HANDLE, bInherit: BOOL) -> BOOL;
    fn DestroyEnvironmentBlock(lpEnvironment: LPVOID) -> BOOL;
    fn LoadUserProfileW(hToken: HANDLE, lpProfileInfo: *mut PROFILEINFOW) -> BOOL;
    fn UnloadUserProfile(hToken: HANDLE, hProfile: HANDLE) -> BOOL;
    fn SetEntriesInAclW(
        cCountOfExplicitEntries: ULONG,
        pListOfExplicitEntries: *mut EXPLICIT_ACCESS_W,
        OldAcl: PACL,
        NewAcl: *mut PACL,
    ) -> DWORD;
    fn OpenSCManagerW(
        lpMachineName: LPCWSTR,
        lpDatabaseName: LPCWSTR,
        dwDesiredAccess: DWORD,
    ) -> SC_HANDLE;
    fn OpenServiceW(hSCManager: SC_HANDLE, lpServiceName: LPCWSTR, dwDesiredAccess: DWORD)
        -> SC_HANDLE;
    fn QueryServiceStatusEx(
        hService: SC_HANDLE,
        InfoLevel: i32,
        lpBuffer: *mut u8,
        cbBufSize: DWORD,
        pcbBytesNeeded: *mut DWORD,
    ) -> BOOL;
    fn ControlService(
        hService: SC_HANDLE,
        dwControl: DWORD,
        lpServiceStatus: *mut SERVICE_STATUS,
    ) -> BOOL;
    fn CloseServiceHandle(hSCObject: SC_HANDLE) -> BOOL;
    fn WNetOpenEnumW(
        dwScope: DWORD,
        dwType: DWORD,
        dwUsage: DWORD,
        lpNetResource: LPNETRESOURCE,
        lphEnum: *mut HANDLE,
    ) -> DWORD;
    fn WNetEnumResourceW(
        hEnum: HANDLE,
        lpcCount: *mut DWORD,
        lpBuffer: LPVOID,
        lpBufferSize: *mut DWORD,
    ) -> DWORD;
    fn WNetCloseEnum(hEnum: HANDLE) -> DWORD;
    fn WNetAddConnectionW(
        lpRemoteName: LPCWSTR,
        lpPassword: LPCWSTR,
        lpLocalName: LPCWSTR,
    ) -> DWORD;
    fn WNetClearConnections(hwndOwner: HWND) -> DWORD;
    fn Beep(dwFreq: DWORD, dwDuration: DWORD) -> BOOL;
    // user32: SetLogonNotifyWindow(long) → NtUserSetLogonNotifyWindow.
    fn SetLogonNotifyWindow(hwnd: HWND) -> BOOL;
    #[link_name = "DefWindowProcW"]
    fn DefWindowProc(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// Cross-file Rust symbols (cdecl, #[no_mangle] in sibling modules).
extern "C" {
    fn DisplayStatusMessage(Session: PWLSESSION, hDesktop: HDESK, ResourceId: DWORD) -> BOOL;
    fn CreateUserEnvironment(Session: PWLSESSION) -> BOOL;
    fn AllowAccessOnSession(Session: PWLSESSION) -> BOOL;
    fn CallNotificationDlls(pSession: PWLSESSION, Type: NOTIFICATION_TYPE);
    fn InitializeScreenSaver(Session: PWLSESSION) -> BOOL;
    fn StartScreenSaver(Session: PWLSESSION);
    fn RemoveStatusMessage(Session: PWLSESSION) -> BOOL;
    fn CloseAllDialogWindows();
    fn WlStrDup(str_: LPCWSTR) -> LPWSTR;
    fn GetSetupType() -> DWORD;
}

// ---- file-scope preamble (translated) ----
pub const WINLOGON_SAS_CLASS: &[u16] = &[0x53, 0x41, 0x53, 0x20, 0x57, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x20, 0x63, 0x6C, 0x61, 0x73, 0x73, 0x00];

pub const WINLOGON_SAS_TITLE: &[u16] = &[0x53, 0x41, 0x53, 0x20, 0x77, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x00];

pub const IDHK_CTRL_ALT_DEL: u32 = 0;
pub const IDHK_CTRL_SHIFT_ESC: u32 = 1;
pub const IDHK_WIN_L: u32 = 2;
pub const IDHK_WIN_U: u32 = 3;

pub const EWX_ACTION_MASK: u32 = 0x5C0F;

#[repr(C)]
pub struct tagLOGOFF_SHUTDOWN_DATA {
    pub Flags: UINT,
    pub Session: PWLSESSION,
}
pub type LOGOFF_SHUTDOWN_DATA = tagLOGOFF_SHUTDOWN_DATA;
pub type PLOGOFF_SHUTDOWN_DATA = *mut tagLOGOFF_SHUTDOWN_DATA;

pub static mut ExitReactOSInProgress: BOOL = 0;

#[no_mangle]
pub static mut LuidNone: LUID = LUID { LowPart: 0, HighPart: 0 };

#[repr(C)]
pub struct tagLOGON_SOUND_DATA {
    pub UserToken: HANDLE,
    pub IsStartup: BOOL,
}
pub type LOGON_SOUND_DATA = tagLOGON_SOUND_DATA;
pub type PLOGON_SOUND_DATA = *mut tagLOGON_SOUND_DATA;

unsafe fn StartTaskManager(Session: PWLSESSION) -> BOOL {
    let mut lpEnvironment: LPVOID = core::ptr::null_mut();

    if (*Session).Gina.Functions.WlxStartApplication.is_none() {
        return FALSE as BOOL;
    }

    if CreateEnvironmentBlock(
        core::ptr::addr_of_mut!(lpEnvironment),
        (*Session).UserToken,
        TRUE as BOOL,
    ) == 0
    {
        return FALSE as BOOL;
    }

    let ret = ((*Session).Gina.Functions.WlxStartApplication.unwrap())(
        (*Session).Gina.Context,
        wstr!("Default").as_ptr() as LPWSTR,
        lpEnvironment,
        wstr!("taskmgr.exe").as_ptr() as LPWSTR,
    );

    DestroyEnvironmentBlock(lpEnvironment);
    ret
}

unsafe fn StartUserShell(Session: PWLSESSION) -> BOOL {
    let mut lpEnvironment: LPVOID = core::ptr::null_mut();
    let mut Old: BOOLEAN = 0;
    let ret: BOOL;

    if CreateEnvironmentBlock(
        core::ptr::addr_of_mut!(lpEnvironment),
        (*Session).UserToken,
        TRUE,
    ) == 0
    {
        WARN!(b"WL: CreateEnvironmentBlock() failed\n\0".as_ptr() as *const _);
        return FALSE;
    }

    RtlAdjustPrivilege(
        SE_ASSIGNPRIMARYTOKEN_PRIVILEGE,
        TRUE as BOOLEAN,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(Old),
    );

    ret = ((*Session).Gina.Functions.WlxActivateUserShell.unwrap())(
        (*Session).Gina.Context,
        b"D\0e\0f\0a\0u\0l\0t\0\0\0".as_ptr() as *mut _,
        core::ptr::null_mut(),
        lpEnvironment,
    );

    DestroyEnvironmentBlock(lpEnvironment);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SetDefaultLanguage(Session: PWLSESSION) -> BOOL {
    let mut ret: BOOL = 0;
    let UserProfile: BOOL;
    let mut rc: i32;
    let mut UserKey: HKEY = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let SubKey: LPCWSTR;
    let ValueName: LPCWSTR;
    let mut dwType: u32 = 0;
    let mut dwSize: u32 = 0;
    let mut Value: LPWSTR = core::ptr::null_mut();
    let mut ValueString: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: i32;
    let mut Lcid: LCID = 0;

    UserProfile = (!Session.is_null() && !(*Session).UserToken.is_null()) as BOOL;

    if UserProfile != 0 && ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        return 0;
    }

    'cleanup: {
        if UserProfile != 0 {
            rc = RegOpenCurrentUser(MAXIMUM_ALLOWED, &mut UserKey);
            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            }

            SubKey = wstr!("Control Panel\\International").as_ptr();
            ValueName = wstr!("Locale").as_ptr();

            rc = RegOpenKeyExW(if !UserKey.is_null() { UserKey } else { HKEY_LOCAL_MACHINE },
                               SubKey,
                               0,
                               KEY_READ,
                               &mut hKey);

            if !UserKey.is_null() {
                RegCloseKey(UserKey);
            }

            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            }

            rc = RegQueryValueExW(hKey, ValueName, core::ptr::null_mut(), &mut dwType, core::ptr::null_mut(), &mut dwSize);
            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            } else if dwType != REG_SZ {
                break 'cleanup;
            }

            Value = HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T) as LPWSTR;
            if Value.is_null() {
                break 'cleanup;
            }
            rc = RegQueryValueExW(hKey, ValueName, core::ptr::null_mut(), core::ptr::null_mut(), Value as LPBYTE, &mut dwSize);
            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            }

            ValueString.Length = dwSize as u16;
            ValueString.MaximumLength = dwSize as u16;
            ValueString.Buffer = Value;
            Status = RtlUnicodeStringToInteger(&mut ValueString, 16, &mut Lcid as *mut LCID as *mut u32);
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            Status = NtSetDefaultLocale(UserProfile as u8, Lcid);
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            ret = 1;
        } else {
            UserKey = core::ptr::null_mut();
            SubKey = wstr!("System\\CurrentControlSet\\Control\\Nls\\Language").as_ptr();
            ValueName = wstr!("Default").as_ptr();

            rc = RegOpenKeyExW(if !UserKey.is_null() { UserKey } else { HKEY_LOCAL_MACHINE },
                               SubKey,
                               0,
                               KEY_READ,
                               &mut hKey);

            if !UserKey.is_null() {
                RegCloseKey(UserKey);
            }

            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            }

            rc = RegQueryValueExW(hKey, ValueName, core::ptr::null_mut(), &mut dwType, core::ptr::null_mut(), &mut dwSize);
            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            } else if dwType != REG_SZ {
                break 'cleanup;
            }

            Value = HeapAlloc(GetProcessHeap(), 0, dwSize as SIZE_T) as LPWSTR;
            if Value.is_null() {
                break 'cleanup;
            }
            rc = RegQueryValueExW(hKey, ValueName, core::ptr::null_mut(), core::ptr::null_mut(), Value as LPBYTE, &mut dwSize);
            if rc != ERROR_SUCCESS as i32 {
                break 'cleanup;
            }

            ValueString.Length = dwSize as u16;
            ValueString.MaximumLength = dwSize as u16;
            ValueString.Buffer = Value;
            Status = RtlUnicodeStringToInteger(&mut ValueString, 16, &mut Lcid as *mut LCID as *mut u32);
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            Status = NtSetDefaultLocale(UserProfile as u8, Lcid);
            if !NT_SUCCESS(Status) {
                break 'cleanup;
            }

            ret = 1;
        }
    }

    if !Value.is_null() {
        HeapFree(GetProcessHeap(), 0, Value as *mut _);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if UserProfile != 0 {
        RevertToSelf();
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn PlaySoundRoutine(
    FileName: LPCWSTR,
    bLogon: UINT,
    Flags: UINT,
) -> BOOL {
    type PLAYSOUNDW = unsafe extern "system" fn(LPCWSTR, HMODULE, DWORD) -> BOOL;
    type WAVEOUTGETNUMDEVS = unsafe extern "system" fn() -> UINT;
    let Play: Option<PLAYSOUNDW>;
    let waveOutGetNumDevs: Option<WAVEOUTGETNUMDEVS>;
    let NumDevs: UINT;
    let hLibrary: HMODULE;
    let mut Ret: BOOL = 0;

    hLibrary = LoadLibraryW(w!("winmm.dll").as_ptr() as LPCWSTR);
    if hLibrary.is_null() {
        return 0;
    }

    waveOutGetNumDevs = core::mem::transmute::<_, Option<WAVEOUTGETNUMDEVS>>(
        GetProcAddress(hLibrary, b"waveOutGetNumDevs\0".as_ptr() as *const i8),
    );
    Play = core::mem::transmute::<_, Option<PLAYSOUNDW>>(
        GetProcAddress(hLibrary, b"PlaySoundW\0".as_ptr() as *const i8),
    );

    'block: {
        if let Some(waveOutGetNumDevs) = waveOutGetNumDevs {
            NumDevs = waveOutGetNumDevs();
            if NumDevs == 0 {
                if bLogon == 0 {
                    Beep(440, 125);
                }
                break 'block;
            }
        }

        if let Some(Play) = Play {
            Ret = Play(FileName, core::ptr::null_mut(), Flags);
        }
    }

    FreeLibrary(hLibrary);

    Ret
}
unsafe extern "system" fn PlayLogonSoundThread(lpParameter: LPVOID) -> DWORD {
    let SoundData: PLOGON_SOUND_DATA = lpParameter as PLOGON_SOUND_DATA;
    let mut Info: SERVICE_STATUS_PROCESS = core::mem::zeroed();
    let mut dwSize: DWORD = 0;
    let mut Index: ULONG = 0;
    let hSCManager: SC_HANDLE;
    let hService: SC_HANDLE;

    'cleanup: loop {
        /* Open the service manager */
        hSCManager = OpenSCManagerW(core::ptr::null_mut(), core::ptr::null_mut(), SC_MANAGER_CONNECT);
        if hSCManager.is_null() {
            break 'cleanup;
        }

        /* Open the wdmaud service */
        hService = OpenServiceW(
            hSCManager,
            w!("wdmaud").as_ptr() as LPCWSTR,
            GENERIC_READ,
        );
        if hService.is_null() {
            /* The service is not installed */
            CloseServiceHandle(hSCManager);
            break 'cleanup;
        }

        /* Wait for wdmaud to start */
        loop {
            if QueryServiceStatusEx(
                hService,
                SC_STATUS_PROCESS_INFO,
                core::ptr::addr_of_mut!(Info) as LPBYTE,
                core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD,
                core::ptr::addr_of_mut!(dwSize),
            ) == 0
            {
                break;
            }

            if Info.dwCurrentState == SERVICE_RUNNING {
                break;
            }

            Sleep(1000);

            let cur = Index;
            Index += 1;
            if !(cur < 20) {
                break;
            }
        }

        CloseServiceHandle(hService);
        CloseServiceHandle(hSCManager);

        /* If wdmaud is not running exit */
        if Info.dwCurrentState != SERVICE_RUNNING {
            break 'cleanup;
        }

        /* Sound subsystem is running. Play logon sound. */
        if ImpersonateLoggedOnUser((*SoundData).UserToken) == 0 {
            // ERR
        } else {
            PlaySoundRoutine(
                if (*SoundData).IsStartup != 0 {
                    w!("SystemStart").as_ptr() as LPCWSTR
                } else {
                    w!("WindowsLogon").as_ptr() as LPCWSTR
                },
                1,
                SND_ALIAS | SND_NODEFAULT,
            );
            RevertToSelf();
        }

        break 'cleanup;
    }

    HeapFree(GetProcessHeap(), 0, SoundData as LPVOID);
    0
}

unsafe fn PlayLogonSound(Session: PWLSESSION) {
    let SoundData = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<LOGON_SOUND_DATA>() as SIZE_T) as PLOGON_SOUND_DATA;
    if SoundData.is_null() {
        return;
    }

    (*SoundData).UserToken = (*Session).UserToken;
    (*SoundData).IsStartup = IsFirstLogon(Session);

    let hThread = CreateThread(
        core::ptr::null_mut(),
        0,
        core::mem::transmute(PlayLogonSoundThread as *const ()),
        SoundData as *mut core::ffi::c_void,
        0,
        core::ptr::null_mut(),
    );
    if hThread.is_null() {
        HeapFree(GetProcessHeap(), 0, SoundData as *mut core::ffi::c_void);
        return;
    }
    CloseHandle(hThread);
}

unsafe fn PlayLogoffShutdownSound(
    Session: PWLSESSION,
    bShutdown: BOOL,
) {
    if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        return;
    }

    PlaySoundRoutine(
        if bShutdown != 0 {
            w!("SystemExit").as_ptr() as LPCWSTR
        } else {
            w!("WindowsLogoff").as_ptr() as LPCWSTR
        },
        0,
        SND_ALIAS | SND_NODEFAULT,
    );

    RevertToSelf();
}

unsafe fn PlayEventSound(
    Session: PWLSESSION,
    EventName: LPCWSTR,
) -> BOOL {
    let bRet: BOOL;

    if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        return 0;
    }

    bRet = PlaySoundRoutine(EventName, 0, SND_ALIAS | SND_ASYNC | SND_NODEFAULT);

    RevertToSelf();

    bRet
}

unsafe fn RestoreAllConnections(Session: PWLSESSION) {
    let mut dRet: DWORD;
    let mut hEnum: HANDLE = core::ptr::null_mut();
    let lpRes: LPNETRESOURCE;
    let mut dSize: DWORD = 0x1000;
    let mut dCount: DWORD = -1i32 as DWORD;
    let mut lpCur: LPNETRESOURCE;
    let UserProfile: BOOL;

    UserProfile = (!Session.is_null() && !(*Session).UserToken.is_null()) as BOOL;
    if UserProfile == 0 {
        return;
    }

    if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        ERR!("WL: ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
        return;
    }

    'quit: {
        dRet = WNetOpenEnumW(RESOURCE_REMEMBERED, RESOURCETYPE_DISK, 0, core::ptr::null_mut(), &mut hEnum);
        if dRet != WN_SUCCESS {
            ERR!("Failed to open enumeration: %lu\n", dRet);
            break 'quit;
        }

        lpRes = HeapAlloc(GetProcessHeap(), 0, dSize as SIZE_T) as LPNETRESOURCE;
        if lpRes.is_null() {
            ERR!("Failed to allocate memory\n");
            WNetCloseEnum(hEnum);
            break 'quit;
        }

        loop {
            dSize = 0x1000;
            dCount = -1i32 as DWORD;

            core::ptr::write_bytes(lpRes as *mut u8, 0, dSize as usize);
            dRet = WNetEnumResourceW(hEnum, &mut dCount, lpRes as *mut _, &mut dSize);
            if dRet == WN_SUCCESS || dRet == WN_MORE_DATA {
                lpCur = lpRes;
                while dCount != 0 {
                    WNetAddConnectionW((*lpCur).lpRemoteName, core::ptr::null_mut(), (*lpCur).lpLocalName);
                    lpCur = lpCur.add(1);
                    dCount -= 1;
                }
            }

            if dRet == WN_NO_MORE_ENTRIES {
                break;
            }
        }

        HeapFree(GetProcessHeap(), 0, lpRes as *mut _);
        WNetCloseEnum(hEnum);
    }

    RevertToSelf();
}

unsafe fn CloseAllConnections(Session: PWLSESSION) {

    if (*Session).UserToken.is_null() || ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        return;
    }
    WNetClearConnections(core::ptr::null_mut());
    RevertToSelf();
}

unsafe fn FreeWlxProfileInfo(Profile: PVOID) {
    let pProfile = Profile as PWLX_PROFILE_V2_0;

    if (*pProfile).dwType != WLX_PROFILE_TYPE_V1_0
        && (*pProfile).dwType != WLX_PROFILE_TYPE_V2_0
    {
        return;
    }

    if !(*pProfile).pszProfile.is_null() {
        LocalFree((*pProfile).pszProfile as _);
    }
    if (*pProfile).dwType >= WLX_PROFILE_TYPE_V2_0 {
        if !(*pProfile).pszPolicy.is_null() {
            LocalFree((*pProfile).pszPolicy as _);
        }
        if !(*pProfile).pszNetworkDefaultUserProfile.is_null() {
            LocalFree((*pProfile).pszNetworkDefaultUserProfile as _);
        }
        if !(*pProfile).pszServerName.is_null() {
            LocalFree((*pProfile).pszServerName as _);
        }
        if !(*pProfile).pszEnvironment.is_null() {
            LocalFree((*pProfile).pszEnvironment as _);
        }
    }
}

unsafe fn FreeWlxMprInfo(MprNotifyInfo: PWLX_MPR_NOTIFY_INFO) {

    if !(*MprNotifyInfo).pszUserName.is_null() {
        LocalFree((*MprNotifyInfo).pszUserName as *mut _);
    }
    if !(*MprNotifyInfo).pszDomain.is_null() {
        LocalFree((*MprNotifyInfo).pszDomain as *mut _);
    }
    if !(*MprNotifyInfo).pszPassword.is_null() {
        let pwdLen = (wcslen((*MprNotifyInfo).pszPassword) + 1) * core::mem::size_of::<u16>();
        SecureZeroMemory((*MprNotifyInfo).pszPassword as *mut _, pwdLen);
        LocalFree((*MprNotifyInfo).pszPassword as *mut _);
    }
    if !(*MprNotifyInfo).pszOldPassword.is_null() {
        let pwdLen = (wcslen((*MprNotifyInfo).pszOldPassword) + 1) * core::mem::size_of::<u16>();
        SecureZeroMemory((*MprNotifyInfo).pszOldPassword as *mut _, pwdLen);
        LocalFree((*MprNotifyInfo).pszOldPassword as *mut _);
    }
}

unsafe fn HandleLogon(Session: PWLSESSION) -> BOOL {

    let mut ret: BOOL = FALSE;

    'cleanup: {
        DisplayStatusMessage(Session, (*Session).WinlogonDesktop, IDS_LOADINGYOURPERSONALSETTINGS);

        (*Session).hProfileInfo = core::ptr::null_mut();
        if ((*Session).Options & WLX_LOGON_OPT_NO_PROFILE) == 0 {
            let mut hKey: HKEY = core::ptr::null_mut();
            let mut lError: LONG;
            let mut bNoPopups: BOOL = FALSE;
            let mut ProfileInfo: PROFILEINFOW = core::mem::zeroed();

            if (*Session).Profile.is_null()
                || ((*(*Session).Profile).dwType != WLX_PROFILE_TYPE_V1_0
                    && (*(*Session).Profile).dwType != WLX_PROFILE_TYPE_V2_0)
            {
                ERR!("WL: Wrong profile\n");
                break 'cleanup;
            }

            lError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                wstr!("System\\CurrentControlSet\\Control\\Windows").as_ptr(),
                0,
                KEY_QUERY_VALUE,
                core::ptr::addr_of_mut!(hKey),
            );
            if lError == ERROR_SUCCESS as LONG {
                let mut dwValue: DWORD = 0;
                let mut dwType: DWORD = 0;
                let mut cbData: DWORD = core::mem::size_of::<DWORD>() as DWORD;
                lError = RegQueryValueExW(
                    hKey,
                    wstr!("NoPopupsOnBoot").as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwType),
                    core::ptr::addr_of_mut!(dwValue) as PBYTE,
                    core::ptr::addr_of_mut!(cbData),
                );
                if lError == ERROR_SUCCESS as LONG
                    && dwType == REG_DWORD
                    && cbData == core::mem::size_of::<DWORD>() as DWORD
                {
                    bNoPopups = (dwValue != 0) as BOOL;
                }

                RegCloseKey(hKey);
            }

            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(ProfileInfo),
                0,
                1,
            );
            ProfileInfo.dwSize = core::mem::size_of::<PROFILEINFOW>() as DWORD;
            if bNoPopups != 0 {
                ProfileInfo.dwFlags |= PI_NOUI;
            }
            ProfileInfo.lpUserName = (*Session).MprNotifyInfo.pszUserName;
            ProfileInfo.lpProfilePath = (*(*Session).Profile).pszProfile;
            if (*(*Session).Profile).dwType >= WLX_PROFILE_TYPE_V2_0 {
                ProfileInfo.lpDefaultPath = (*(*Session).Profile).pszNetworkDefaultUserProfile;
                ProfileInfo.lpServerName = (*(*Session).Profile).pszServerName;
                ProfileInfo.lpPolicyPath = (*(*Session).Profile).pszPolicy;
            }

            if LoadUserProfileW((*Session).UserToken, core::ptr::addr_of_mut!(ProfileInfo)) == 0 {
                ERR!("WL: LoadUserProfileW() failed\n");
                break 'cleanup;
            }
            (*Session).hProfileInfo = ProfileInfo.hProfile;
        }

        (*Session).UserName = WlStrDup((*Session).MprNotifyInfo.pszUserName);
        (*Session).Domain = WlStrDup((*Session).MprNotifyInfo.pszDomain);

        if CreateUserEnvironment(Session) == 0 {
            WARN!(b"WL: CreateUserEnvironment() failed\n\0".as_ptr() as *const _);
            break 'cleanup;
        }

        DisplayStatusMessage(Session, (*Session).WinlogonDesktop, IDS_APPLYINGYOURPERSONALSETTINGS);
        UpdatePerUserSystemParameters(0, TRUE);

        if SetDefaultLanguage(Session) == 0 {
            WARN!(b"WL: SetDefaultLanguage() failed\n\0".as_ptr() as *const _);
            break 'cleanup;
        }

        if AllowAccessOnSession(Session) == 0 {
            WARN!(b"WL: AllowAccessOnSession() failed to give winsta & desktop access for this session\n\0".as_ptr() as *const _);
            break 'cleanup;
        }

        CallNotificationDlls(Session, LogonHandler);

        RestoreAllConnections(Session);

        CallNotificationDlls(Session, StartShellHandler);
        if StartUserShell(Session) == 0 {
            WARN!(b"WL: WlxActivateUserShell() failed\n\0".as_ptr() as *const _);
            break 'cleanup;
        }
        CallNotificationDlls(Session, PostShellHandler);

        if InitializeScreenSaver(Session) == 0 {
            WARN!(b"WL: Failed to initialize screen saver\n\0".as_ptr() as *const _);
        }

        PlayLogonSound(Session);

        SetLogonTimestamp(Session);
        ret = TRUE;
    }

    if !(*Session).Profile.is_null() {
        FreeWlxProfileInfo((*Session).Profile as PVOID);
        LocalFree((*Session).Profile as *mut _);
        (*Session).Profile = core::ptr::null_mut();
    }
    FreeWlxMprInfo(core::ptr::addr_of_mut!((*Session).MprNotifyInfo));
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*Session).MprNotifyInfo),
        0,
        1,
    );

    RemoveStatusMessage(Session);

    if ret == 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName as PVOID);
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).Domain as PVOID);
        (*Session).Domain = core::ptr::null_mut();
        (*Session).UserName = core::ptr::null_mut();

        if !(*Session).hProfileInfo.is_null() {
            UnloadUserProfile((*Session).UserToken, (*Session).hProfileInfo);
        }
        (*Session).hProfileInfo = core::ptr::null_mut();

        UpdatePerUserSystemParameters(0, FALSE);

        SetWindowStationUser(
            (*Session).InteractiveWindowStation,
            core::ptr::addr_of_mut!(LuidNone),
            core::ptr::null_mut(),
            0,
        );

        CloseHandle((*Session).UserToken);
        (*Session).UserToken = core::ptr::null_mut();
        (*Session).LogonId = LuidNone;
    } else {
        SwitchDesktop((*Session).ApplicationDesktop);
        (*Session).LogonState = STATE_LOGGED_ON;
    }
    ret
}
unsafe extern "system" fn LogoffShutdownThread(Parameter: LPVOID) -> DWORD {
    let LSData = Parameter as PLOGOFF_SHUTDOWN_DATA;
    let UserToken = (*(*LSData).Session).UserToken;
    let mut ret: DWORD = 1;
    let uFlags: UINT;

    if !UserToken.is_null() && ImpersonateLoggedOnUser(UserToken) == 0 {
        return 0;
    }

    uFlags = EWX_CALLER_WINLOGON | ((*LSData).Flags & 0x0F);

    ExitReactOSInProgress = 1;

    if ExitWindowsEx(uFlags, 0) == 0 {
        ret = 0;
    }

    if !UserToken.is_null() {
        RevertToSelf();
    }

    ret
}

unsafe fn RunLogoffShutdownThread(
    Session: PWLSESSION,
    psa: PSECURITY_ATTRIBUTES,
    wlxAction: DWORD,
) -> NTSTATUS {
    let _pDescName: PCSTR;
    let LSData: PLOGOFF_SHUTDOWN_DATA;
    let hThread: HANDLE;
    let mut dwExitCode: DWORD = 0;

    if WLX_LOGGINGOFF(wlxAction) {
        _pDescName = b"Logoff\0".as_ptr() as PCSTR;
    } else if WLX_SHUTTINGDOWN(wlxAction) {
        _pDescName = b"Shutdown\0".as_ptr() as PCSTR;
    } else {
        return STATUS_INVALID_PARAMETER;
    }

    LSData = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<LOGOFF_SHUTDOWN_DATA>() as SIZE_T,
    ) as PLOGOFF_SHUTDOWN_DATA;
    if LSData.is_null() {
        return STATUS_NO_MEMORY;
    }

    if WLX_LOGGINGOFF(wlxAction) {
        (*LSData).Flags = EWX_LOGOFF;
        if wlxAction == WLX_SAS_ACTION_FORCE_LOGOFF {
            (*LSData).Flags |= EWX_FORCE;
        }
    } else {
        (*LSData).Flags = EWX_SHUTDOWN | EWX_FORCE;
        if wlxAction == WLX_SAS_ACTION_SHUTDOWN_POWER_OFF {
            (*LSData).Flags |= EWX_POWEROFF;
        } else if wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT {
            (*LSData).Flags |= EWX_REBOOT;
        }
    }

    (*LSData).Session = Session;

    hThread = CreateThread(
        psa,
        0,
        core::mem::transmute(LogoffShutdownThread as *const ()),
        LSData as *mut core::ffi::c_void,
        0,
        core::ptr::null_mut(),
    );
    if hThread.is_null() {
        HeapFree(GetProcessHeap(), 0, LSData as *mut core::ffi::c_void);
        return STATUS_UNSUCCESSFUL;
    }
    WaitForSingleObject(hThread, INFINITE);
    HeapFree(GetProcessHeap(), 0, LSData as *mut core::ffi::c_void);
    if GetExitCodeThread(hThread, core::ptr::addr_of_mut!(dwExitCode)) == 0 {
        CloseHandle(hThread);
        return STATUS_UNSUCCESSFUL;
    }
    CloseHandle(hThread);
    if dwExitCode == 0 {
        return STATUS_UNSUCCESSFUL;
    }
    STATUS_SUCCESS
}

unsafe fn CreateLogoffSecurityAttributes(
    ppsa: *mut PSECURITY_ATTRIBUTES,
) -> NTSTATUS {
    let SecurityDescriptor: PSECURITY_DESCRIPTOR;
    let psa: PSECURITY_ATTRIBUTES;
    let pMem: *mut BYTE;
    let mut pACL: PACL;
    let mut Access: EXPLICIT_ACCESS = core::mem::zeroed();
    let mut pEveryoneSID: PSID = core::ptr::null_mut();
    static mut WorldAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };

    *ppsa = core::ptr::null_mut();

    if AllocateAndInitializeSid(
        core::ptr::addr_of_mut!(WorldAuthority),
        1,
        SECURITY_WORLD_RID as _,
        0, 0, 0, 0, 0, 0, 0,
        &mut pEveryoneSID,
    ) == 0
    {
        ERR!("Failed to initialize security descriptor for logoff thread!\n");
        return STATUS_UNSUCCESSFUL;
    }

    pMem = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<SECURITY_ATTRIBUTES>()
            + SECURITY_DESCRIPTOR_MIN_LENGTH as usize
            + core::mem::size_of::<ACL>()) as SIZE_T,
    ) as *mut BYTE;
    if pMem.is_null() {
        ERR!("Failed to allocate memory for logoff security descriptor!\n");
        return STATUS_NO_MEMORY;
    }

    psa = pMem as PSECURITY_ATTRIBUTES;
    SecurityDescriptor =
        pMem.add(core::mem::size_of::<SECURITY_ATTRIBUTES>()) as PSECURITY_DESCRIPTOR;
    pACL = (SecurityDescriptor as PBYTE).add(SECURITY_DESCRIPTOR_MIN_LENGTH as usize) as PACL;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(Access) as *mut u8,
        0,
        core::mem::size_of::<EXPLICIT_ACCESS>(),
    );
    Access.grfAccessPermissions = THREAD_SET_THREAD_TOKEN as _;
    Access.grfAccessMode = SET_ACCESS;
    Access.grfInheritance = NO_INHERITANCE as _;
    Access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
    Access.Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
    Access.Trustee.ptstrName = pEveryoneSID as _;

    if SetEntriesInAclW(1, &mut Access, core::ptr::null_mut(), &mut pACL) != ERROR_SUCCESS {
        ERR!("Failed to set Access Rights for logoff thread. Logging out will most likely fail.\n");
        HeapFree(GetProcessHeap(), 0, pMem as _);
        return STATUS_UNSUCCESSFUL;
    }

    if InitializeSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION as _) == 0 {
        ERR!("Failed to initialize security descriptor for logoff thread!\n");
        HeapFree(GetProcessHeap(), 0, pMem as _);
        return STATUS_UNSUCCESSFUL;
    }

    if SetSecurityDescriptorDacl(SecurityDescriptor, 1, pACL, 0) == 0 {
        ERR!("SetSecurityDescriptorDacl Error %lu\n", GetLastError());
        HeapFree(GetProcessHeap(), 0, pMem as _);
        return STATUS_UNSUCCESSFUL;
    }

    (*psa).nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as _;
    (*psa).lpSecurityDescriptor = SecurityDescriptor;
    (*psa).bInheritHandle = 0;

    *ppsa = psa;

    STATUS_SUCCESS
}

unsafe fn DestroyLogoffSecurityAttributes(psa: PSECURITY_ATTRIBUTES) {
    if !psa.is_null() {
        HeapFree(GetProcessHeap(), 0, psa as *mut _);
    }
}

// IsFirstLogon (FORCEINLINE in winlogon.h) — no symbol.
#[inline]
unsafe fn IsFirstLogon(Session: PWLSESSION) -> BOOL {
    ((*Session).LastLogon.QuadPart == 0) as BOOL
}

// KillComProcesses — WINAPI thread routine (rustconv left only the C in
// comments; hand-translated). extern "system" for the correct machine ABI;
// the CreateThread call site transmutes it into the LPTHREAD_START_ROUTINE slot.
unsafe extern "system" fn KillComProcesses(Parameter: LPVOID) -> DWORD {
    let UserToken = Parameter as HANDLE;
    let mut ret: DWORD = TRUE as DWORD;

    if !UserToken.is_null() && ImpersonateLoggedOnUser(UserToken) == 0 {
        return FALSE as DWORD;
    }

    /* Attempt to kill remaining processes. No notifications needed. */
    if ExitWindowsEx(
        EWX_CALLER_WINLOGON | EWX_NONOTIFY | EWX_FORCE | EWX_LOGOFF,
        0,
    ) == 0
    {
        ret = FALSE as DWORD;
    }

    if !UserToken.is_null() {
        RevertToSelf();
    }

    ret
}

unsafe fn HandleLogoff(
    Session: PWLSESSION,
    wlxAction: DWORD,
) -> NTSTATUS {

    let mut psa: PSECURITY_ATTRIBUTES = core::ptr::null_mut();
    let hThread: HANDLE;
    let mut Status: NTSTATUS;

    Status = CreateLogoffSecurityAttributes(&mut psa);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RunLogoffShutdownThread(
        Session,
        psa,
        if WLX_LOGGINGOFF(wlxAction) {
            wlxAction
        } else {
            WLX_SAS_ACTION_LOGOFF as DWORD
        },
    );
    if !NT_SUCCESS(Status) {
        DestroyLogoffSecurityAttributes(psa);
        return Status;
    }

    SwitchDesktop((*Session).ApplicationDesktop);
    DisplayStatusMessage(Session, (*Session).ApplicationDesktop, IDS_LOGGINGOFF);
    CallNotificationDlls(Session, LogoffHandler);
    RemoveStatusMessage(Session);

    SwitchDesktop((*Session).WinlogonDesktop);

    PlayLogoffShutdownSound(Session, WLX_SHUTTINGDOWN(wlxAction) as BOOL);

    DisplayStatusMessage(Session, (*Session).WinlogonDesktop, IDS_CLOSINGNETWORKCONNECTIONS);
    CloseAllConnections(Session);

    SetWindowStationUser(
        (*Session).InteractiveWindowStation,
        core::ptr::addr_of_mut!(LuidNone),
        core::ptr::null_mut(),
        0,
    );

    hThread = CreateThread(
        psa,
        0,
        core::mem::transmute(KillComProcesses as *const ()),
        (*Session).UserToken as PVOID,
        0,
        core::ptr::null_mut(),
    );
    if !hThread.is_null() {
        WaitForSingleObject(hThread, INFINITE);
        CloseHandle(hThread);
    }

    DestroyLogoffSecurityAttributes(psa);

    DisplayStatusMessage(Session, (*Session).WinlogonDesktop, IDS_SAVEYOURSETTINGS);

    RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).UserName as PVOID);
    RtlFreeHeap(RtlGetProcessHeap(), 0, (*Session).Domain as PVOID);
    (*Session).UserName = core::ptr::null_mut();
    (*Session).Domain = core::ptr::null_mut();

    if !(*Session).hProfileInfo.is_null() {
        UnloadUserProfile((*Session).UserToken, (*Session).hProfileInfo);
    }
    (*Session).hProfileInfo = core::ptr::null_mut();

    UpdatePerUserSystemParameters(0, 0u8 as BOOL);

    CloseHandle((*Session).UserToken);
    (*Session).UserToken = core::ptr::null_mut();
    (*Session).LogonId = LuidNone;

    (*Session).LogonState = STATE_LOGGED_OFF;

    STATUS_SUCCESS
}
unsafe fn ShutdownComputerWindowProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let _ = lParam;

    match uMsg {
        WM_COMMAND => {
            match (wParam & 0xFFFF) as u32 {
                IDC_BTNSHTDOWNCOMPUTER => {
                    EndDialog(hwndDlg, IDC_BTNSHTDOWNCOMPUTER as INT_PTR);
                    return 1 as INT_PTR;
                }
                _ => {}
            }
        }
        WM_INITDIALOG => {
            DeleteMenu(GetSystemMenu(hwndDlg, 0), SC_CLOSE as u32, MF_BYCOMMAND as u32);
            return 1 as INT_PTR;
        }
        _ => {}
    }
    0 as INT_PTR
}

unsafe fn UninitializeSAS(Session: PWLSESSION) {
    if !(*Session).SASWindow.is_null() {
        DestroyWindow((*Session).SASWindow);
        (*Session).SASWindow = core::ptr::null_mut();
    }
    if !(*Session).hEndOfScreenSaverThread.is_null() {
        SetEvent((*Session).hEndOfScreenSaverThread);
    }
    UnregisterClassW(
        wstr!("SAS Window class").as_ptr(),
        hAppInstance,
    );
}
#[no_mangle]
pub unsafe extern "C" fn HandleShutdown(Session: PWLSESSION, wlxAction: DWORD) -> NTSTATUS {
    let Status: NTSTATUS;
    let uMsgId: UINT;
    let mut Old: BOOLEAN = 0;

    DbgPrint(b"[WINLOGON] HandleShutdown(action=%lu)\n\0".as_ptr() as *const i8, wlxAction);
    if wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT {
        uMsgId = IDS_REACTOSISRESTARTING;
    } else {
        uMsgId = IDS_REACTOSISSHUTTINGDOWN;
    }

    DisplayStatusMessage(Session, (*Session).WinlogonDesktop, uMsgId);

    CallNotificationDlls(Session, ShutdownHandler);
    (*Session).Gina.Functions.WlxShutdown.unwrap()((*Session).Gina.Context, wlxAction);

    Status = RunLogoffShutdownThread(Session, core::ptr::null_mut(), wlxAction);
    if !NT_SUCCESS(Status) {
        ERR!("Failed to start the Shutdown thread, Status 0x%08lx\n", Status);
    }

    DisplayStatusMessage(Session, (*Session).WinlogonDesktop, uMsgId);

    UninitializeSAS(Session);

    ERR!("Shutting down NT...\n");
    RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1u8, 0u8, core::ptr::addr_of_mut!(Old));
    if wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT {
        NtShutdownSystem(ShutdownReboot);
    } else {
        if false {
            // DialogBox(hAppInstance, IDD_SHUTDOWNCOMPUTER, …, ShutdownComputerWindowProc)
            // — dead (`if false`) in the C too; ShutdownComputerWindowProc stays as
            // dead_code (MAKEINTRESOURCE/DialogBox not needed).
            let _ = IDD_SHUTDOWNCOMPUTER;
        }
        if wlxAction == WLX_SAS_ACTION_SHUTDOWN_POWER_OFF {
            NtShutdownSystem(ShutdownPowerOff);
        } else {
            NtShutdownSystem(ShutdownNoReboot);
        }
    }
    RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, Old, 0u8, core::ptr::addr_of_mut!(Old));
    STATUS_SUCCESS
}

unsafe fn DoGenericAction(Session: PWLSESSION, wlxAction: DWORD) {

    match wlxAction {
        WLX_SAS_ACTION_LOGON => {
            if (*Session).LogonState == STATE_LOGGED_OFF_SAS {
                if HandleLogon(Session) == 0 {
                    (*Session).LogonState = STATE_LOGGED_OFF;
                    (*Session).Gina.Functions.WlxDisplaySASNotice.unwrap()((*Session).Gina.Context);
                }
            }
        }
        WLX_SAS_ACTION_NONE => {
            if (*Session).LogonState == STATE_LOGGED_OFF_SAS {
                (*Session).LogonState = STATE_LOGGED_OFF;
                (*Session).Gina.Functions.WlxDisplaySASNotice.unwrap()((*Session).Gina.Context);
            } else if (*Session).LogonState == STATE_LOGGED_ON_SAS {
                (*Session).LogonState = STATE_LOGGED_ON;
            } else if (*Session).LogonState == STATE_LOCKED_SAS {
                (*Session).LogonState = STATE_LOCKED;
                (*Session).Gina.Functions.WlxDisplayLockedNotice.unwrap()((*Session).Gina.Context);
            }
        }
        WLX_SAS_ACTION_LOCK_WKSTA => {
            if (*Session).LogonState == STATE_LOGGED_ON || (*Session).LogonState == STATE_LOGGED_ON_SAS {
                if (*Session).Gina.Functions.WlxIsLockOk.unwrap()((*Session).Gina.Context) != 0 {
                    (*Session).LogonState = STATE_LOCKED;
                    SwitchDesktop((*Session).WinlogonDesktop);
                    CloseAllDialogWindows();
                    CallNotificationDlls(Session, LockHandler);
                    (*Session).Gina.Functions.WlxDisplayLockedNotice.unwrap()((*Session).Gina.Context);
                }
            }
        }
        WLX_SAS_ACTION_LOGOFF
        | WLX_SAS_ACTION_SHUTDOWN
        | WLX_SAS_ACTION_FORCE_LOGOFF
        | WLX_SAS_ACTION_SHUTDOWN_POWER_OFF
        | WLX_SAS_ACTION_SHUTDOWN_REBOOT => {
            'block: {
                if (*Session).LogonState != STATE_INIT
                    && (*Session).LogonState != STATE_LOGGED_OFF
                    && (*Session).LogonState != STATE_LOGGED_OFF_SAS
                    && (*Session).LogonState != STATE_SHUT_DOWN
                {
                    if (*Session).Gina.Functions.WlxIsLogoffOk.unwrap()((*Session).Gina.Context) == 0 {
                        break 'block;
                    }
                    if !NT_SUCCESS(HandleLogoff(Session, wlxAction)) {
                        RemoveStatusMessage(Session);
                        break 'block;
                    }
                    (*Session).Gina.Functions.WlxLogoff.unwrap()((*Session).Gina.Context);
                }
                if WLX_SHUTTINGDOWN(wlxAction) {
                    if !NT_SUCCESS(HandleShutdown(Session, wlxAction)) {
                        RemoveStatusMessage(Session);
                        (*Session).LogonState = STATE_LOGGED_OFF;
                        (*Session).Gina.Functions.WlxDisplaySASNotice.unwrap()((*Session).Gina.Context);
                    }
                } else {
                    RemoveStatusMessage(Session);
                    (*Session).LogonState = STATE_LOGGED_OFF;
                    (*Session).Gina.Functions.WlxDisplaySASNotice.unwrap()((*Session).Gina.Context);
                }
            }
        }
        WLX_SAS_ACTION_TASKLIST => {
            if (*Session).LogonState == STATE_LOGGED_ON || (*Session).LogonState == STATE_LOGGED_ON_SAS {
                StartTaskManager(Session);
            }
        }
        WLX_SAS_ACTION_UNLOCK_WKSTA => {
            if (*Session).LogonState == STATE_LOCKED || (*Session).LogonState == STATE_LOCKED_SAS {
                CallNotificationDlls(Session, UnlockHandler);
                SwitchDesktop((*Session).ApplicationDesktop);
                (*Session).LogonState = STATE_LOGGED_ON;
            }
        }
        _ => {}
    }
}

unsafe fn DispatchSAS(Session: PWLSESSION, dwSasType: DWORD) {

    let mut wlxAction: DWORD = WLX_SAS_ACTION_NONE;
    let LogonSid: PSID = core::ptr::null_mut();
    let mut bSecure: BOOL = 1;

    match dwSasType {
        WLX_SAS_TYPE_CTRL_ALT_DEL => {
            match (*Session).LogonState {
                STATE_INIT => {
                    (*Session).LogonState = STATE_LOGGED_OFF;
                    (*Session).Gina.Functions.WlxDisplaySASNotice.unwrap()((*Session).Gina.Context);
                    return;
                }
                STATE_LOGGED_OFF => {
                    (*Session).LogonState = STATE_LOGGED_OFF_SAS;

                    CloseAllDialogWindows();

                    (*Session).Options = 0;

                    wlxAction = (*Session).Gina.Functions.WlxLoggedOutSAS.unwrap()(
                        (*Session).Gina.Context,
                        (*Session).SASAction,
                        core::ptr::addr_of_mut!((*Session).LogonId),
                        LogonSid,
                        core::ptr::addr_of_mut!((*Session).Options),
                        core::ptr::addr_of_mut!((*Session).UserToken),
                        core::ptr::addr_of_mut!((*Session).MprNotifyInfo),
                        core::ptr::addr_of_mut!((*Session).Profile) as *mut PVOID,
                    ) as DWORD;
                }
                STATE_LOGGED_OFF_SAS => {
                    return;
                }
                STATE_LOGGED_ON => {
                    (*Session).LogonState = STATE_LOGGED_ON_SAS;
                    SwitchDesktop((*Session).WinlogonDesktop);
                    wlxAction = (*Session).Gina.Functions.WlxLoggedOnSAS.unwrap()((*Session).Gina.Context, dwSasType, core::ptr::null_mut()) as DWORD;
                    if wlxAction == WLX_SAS_ACTION_NONE || wlxAction == WLX_SAS_ACTION_TASKLIST {
                        SwitchDesktop((*Session).ApplicationDesktop);
                        (*Session).LogonState = STATE_LOGGED_ON;
                    }
                }
                STATE_LOGGED_ON_SAS => {
                    return;
                }
                STATE_LOCKED => {
                    (*Session).LogonState = STATE_LOCKED_SAS;

                    CloseAllDialogWindows();

                    wlxAction = (*Session).Gina.Functions.WlxWkstaLockedSAS.unwrap()((*Session).Gina.Context, dwSasType) as DWORD;
                }
                STATE_LOCKED_SAS => {
                    return;
                }
                _ => {
                    return;
                }
            }
        }
        WLX_SAS_TYPE_TIMEOUT => {
            return;
        }
        WLX_SAS_TYPE_SCRNSVR_TIMEOUT => {
            if (*Session).Gina.Functions.WlxScreenSaverNotify.unwrap()((*Session).Gina.Context, core::ptr::addr_of_mut!(bSecure)) == 0 {
                SetEvent((*Session).hEndOfScreenSaver);
            } else {
                StartScreenSaver(Session);
                if bSecure != 0 {
                    wlxAction = WLX_SAS_ACTION_LOCK_WKSTA;
                }
            }
        }
        WLX_SAS_TYPE_SCRNSVR_ACTIVITY => {
            SetEvent((*Session).hUserActivity);
        }
        _ => {}
    }

    DoGenericAction(Session, wlxAction);
}

unsafe fn RegisterHotKeys(
    Session: PWLSESSION,
    hwndSAS: HWND,
) -> BOOL {
    if RegisterHotKey(hwndSAS, IDHK_CTRL_ALT_DEL as i32, (MOD_CONTROL | MOD_ALT) as u32, VK_DELETE as u32) == 0 {
        return 0;
    }

    (*Session).TaskManHotkey = RegisterHotKey(hwndSAS, IDHK_CTRL_SHIFT_ESC as i32, (MOD_CONTROL | MOD_SHIFT) as u32, VK_ESCAPE as u32);

    (*Session).LockWkStaHotkey = RegisterHotKey(hwndSAS, IDHK_WIN_L as i32, MOD_WIN as u32, b'L' as u32);

    (*Session).UtilManHotkey = RegisterHotKey(hwndSAS, IDHK_WIN_U as i32, MOD_WIN as u32, b'U' as u32);

    1
}

unsafe fn UnregisterHotKeys(
    Session: PWLSESSION,
    hwndSAS: HWND,
) -> BOOL {
    UnregisterHotKey(hwndSAS, 0);

    if (*Session).TaskManHotkey != 0 {
        UnregisterHotKey(hwndSAS, 1);
    }

    if (*Session).LockWkStaHotkey != 0 {
        UnregisterHotKey(hwndSAS, 2);
    }

    if (*Session).UtilManHotkey != 0 {
        UnregisterHotKey(hwndSAS, 3);
    }

    1
}

unsafe fn HandleMessageBeep(
    Session: PWLSESSION,
    uType: UINT,
) -> BOOL {
    let EventName: LPWSTR;

    match uType {
        0xFFFFFFFF => {
            EventName = core::ptr::null_mut();
        }
        MB_OK => {
            EventName = wstr!("SystemDefault").as_ptr() as LPWSTR;
        }
        MB_ICONASTERISK => {
            EventName = wstr!("SystemAsterisk").as_ptr() as LPWSTR;
        }
        MB_ICONEXCLAMATION => {
            EventName = wstr!("SystemExclamation").as_ptr() as LPWSTR;
        }
        MB_ICONHAND => {
            EventName = wstr!("SystemHand").as_ptr() as LPWSTR;
        }
        MB_ICONQUESTION => {
            EventName = wstr!("SystemQuestion").as_ptr() as LPWSTR;
        }
        _ => {
            EventName = wstr!("SystemDefault").as_ptr() as LPWSTR;
        }
    }

    PlayEventSound(Session, EventName)
}
unsafe extern "system" fn SASWindowProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {

    let mut Session: PWLSESSION = GetWindowLongPtr(hwndDlg, GWLP_USERDATA) as PWLSESSION;

    match uMsg {
        WM_HOTKEY => {
            match wParam as u32 {
                IDHK_CTRL_ALT_DEL => {
                    if (*Session).Gina.UseCtrlAltDelete == 0 {
                        return DefWindowProc(hwndDlg, uMsg, wParam, lParam);
                    }
                    PostMessageW((*Session).SASWindow, WLX_WM_SAS, WLX_SAS_TYPE_CTRL_ALT_DEL as WPARAM, 0);
                    return 1;
                }
                IDHK_CTRL_SHIFT_ESC => {
                    DoGenericAction(Session, WLX_SAS_ACTION_TASKLIST);
                    return 1;
                }
                IDHK_WIN_L => {
                    PostMessageW((*Session).SASWindow, WM_LOGONNOTIFY, LN_LOCK_WORKSTATION as WPARAM, 0);
                    return 1;
                }
                IDHK_WIN_U => {
                    return 1;
                }
                _ => {}
            }
        }
        WM_CREATE => {
            Session = (*(lParam as LPCREATESTRUCT)).lpCreateParams as PWLSESSION;
            SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, Session as LONG_PTR);
            if GetSetupType() != 0 {
                return 1;
            }
            return RegisterHotKeys(Session, hwndDlg) as LRESULT;
        }
        WM_DESTROY => {
            if GetSetupType() == 0 {
                UnregisterHotKeys(Session, hwndDlg);
            }
            PostQuitMessage(0);
            return 1;
        }
        WM_SETTINGCHANGE => {
            let uiAction = wParam as UINT;
            if uiAction == SPI_SETSCREENSAVETIMEOUT || uiAction == SPI_SETSCREENSAVEACTIVE {
                SetEvent((*Session).hScreenSaverParametersChanged);
            }
            return 1;
        }
        WM_LOGONNOTIFY => {
            match wParam as u32 {
                LN_MESSAGE_BEEP => {
                    return HandleMessageBeep(Session, lParam as UINT) as LRESULT;
                }
                LN_SHELL_EXITED => {
                    if lParam != 1
                        && (*Session).LogonState != STATE_LOGGED_OFF
                        && (*Session).LogonState != STATE_LOGGED_OFF_SAS
                    {
                        SetTimer(hwndDlg, 1, 1000, None);
                    }
                }
                LN_START_SCREENSAVE => {
                    DispatchSAS(Session, WLX_SAS_TYPE_SCRNSVR_TIMEOUT);
                }
                LN_LOCK_WORKSTATION => {
                    DoGenericAction(Session, WLX_SAS_ACTION_LOCK_WKSTA);
                }
                LN_LOGOFF => {
                    let Flags = lParam as UINT;
                    DbgPrint(b"[WINLOGON] LN_LOGOFF flags=0x%x state=%u\n\0".as_ptr() as *const i8, Flags, (*Session).LogonState as u32);
                    let mut Action = Flags & EWX_ACTION_MASK as UINT;
                    let wlxAction: DWORD;

                    if (Action & (EWX_POWEROFF | EWX_REBOOT)) != 0 && (Action & EWX_SHUTDOWN) == 0 {
                        return STATUS_INVALID_PARAMETER as LRESULT;
                    }

                    if (Action & EWX_POWEROFF) != 0 {
                        wlxAction = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF;
                        Action &= !(EWX_SHUTDOWN | EWX_POWEROFF);
                    } else if (Action & EWX_REBOOT) != 0 {
                        wlxAction = WLX_SAS_ACTION_SHUTDOWN_REBOOT;
                        Action &= !(EWX_SHUTDOWN | EWX_REBOOT);
                    } else if (Action & EWX_SHUTDOWN) != 0 {
                        wlxAction = WLX_SAS_ACTION_SHUTDOWN;
                        Action &= !EWX_SHUTDOWN;
                    } else {
                        if (Action & EWX_FORCE) != 0 {
                            wlxAction = WLX_SAS_ACTION_FORCE_LOGOFF;
                        } else {
                            wlxAction = WLX_SAS_ACTION_LOGOFF;
                        }
                        Action &= !(EWX_LOGOFF | EWX_FORCE);
                    }

                    DoGenericAction(Session, wlxAction);
                    return 1;
                }
                LN_LOGOFF_CANCELED => {
                    ExitReactOSInProgress = 0;
                    return 1;
                }
                _ => {}
            }
            return 0;
        }
        WM_TIMER => {
            if wParam == 1 {
                KillTimer(hwndDlg, 1);
                StartUserShell(Session);
            }
        }
        WLX_WM_SAS => {
            DispatchSAS(Session, wParam as DWORD);
            return 1;
        }
        _ => {}
    }

    DefWindowProc(hwndDlg, uMsg, wParam, lParam)
}
#[no_mangle]
pub unsafe extern "C" fn InitializeSAS(Session: PWLSESSION) -> BOOL {
    let mut swc: WNDCLASSEXW = core::mem::zeroed();
    let mut ret: BOOL = 0;

    'cleanup: {
        if SwitchDesktop((*Session).WinlogonDesktop) == 0 {
            break 'cleanup;
        }

        swc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
        swc.style = CS_SAVEBITS;
        swc.lpfnWndProc = core::mem::transmute(SASWindowProc as *const ());
        swc.cbClsExtra = 0;
        swc.cbWndExtra = 0;
        swc.hInstance = hAppInstance;
        swc.hIcon = core::ptr::null_mut();
        swc.hCursor = core::ptr::null_mut();
        swc.hbrBackground = core::ptr::null_mut();
        swc.lpszMenuName = core::ptr::null_mut();
        swc.lpszClassName = wstr!("SAS Window class").as_ptr();
        swc.hIconSm = core::ptr::null_mut();
        if RegisterClassExW(&mut swc) == 0 {
            break 'cleanup;
        }

        (*Session).SASWindow = CreateWindowExW(
            0,
            wstr!("SAS Window class").as_ptr(),
            wstr!("SAS window").as_ptr(),
            WS_POPUP,
            0, 0, 0, 0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            hAppInstance,
            Session as *mut core::ffi::c_void,
        );
        if (*Session).SASWindow.is_null() {
            break 'cleanup;
        }

        if SetLogonNotifyWindow((*Session).SASWindow) == 0 {
            break 'cleanup;
        }

        if SetDefaultLanguage(core::ptr::null_mut()) == 0 {
            return 0;
        }

        ret = 1;
    }

    if ret == 0 {
        UninitializeSAS(Session);
    }
    ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Winlogon
//  * FILE:            base/system/winlogon/sas.c
//  * PURPOSE:         Secure Attention Sequence
//  * PROGRAMMERS:     Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  *                  Hervé Poussineau (hpoussin@reactos.org)
//  *                  Arnav Bhatt (arnavbhatt288@gmail.com)
//  * UPDATE HISTORY:
//  *                  Created 28/03/2004
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "winlogon.h"
//
// #define WIN32_LEAN_AND_MEAN
// #include <aclapi.h>
// #include <mmsystem.h>
// #include <userenv.h>
// #include <ndk/setypes.h>
// #include <ndk/sefuncs.h>
//
// /* GLOBALS ******************************************************************/
//
// #define WINLOGON_SAS_CLASS L"SAS Window class"
// #define WINLOGON_SAS_TITLE L"SAS window"
//
// #define IDHK_CTRL_ALT_DEL   0
// #define IDHK_CTRL_SHIFT_ESC 1
// #define IDHK_WIN_L          2
// #define IDHK_WIN_U          3
//
// // #define EWX_FLAGS_MASK  0x00000014
// // #define EWX_ACTION_MASK ~EWX_FLAGS_MASK
//
// // FIXME: At the moment we use this value (select the lowbyte flags and some highbytes ones).
// // It should be set such that it makes winlogon accepting only valid flags.
// #define EWX_ACTION_MASK 0x5C0F
//
// typedef struct tagLOGOFF_SHUTDOWN_DATA
// {
//     UINT Flags;
//     PWLSESSION Session;
// } LOGOFF_SHUTDOWN_DATA, *PLOGOFF_SHUTDOWN_DATA;
//
// static BOOL ExitReactOSInProgress = FALSE;
//
// LUID LuidNone = {0, 0};
//
// typedef struct tagLOGON_SOUND_DATA
// {
//     HANDLE UserToken;
//     BOOL IsStartup;
// } LOGON_SOUND_DATA, *PLOGON_SOUND_DATA;
//
// /* FUNCTIONS ****************************************************************/
//
// static BOOL
// StartTaskManager(
//     IN OUT PWLSESSION Session)
// {
//     LPVOID lpEnvironment;
//     BOOL ret;
//
//     if (!Session->Gina.Functions.WlxStartApplication)
//         return FALSE;
//
//     if (!CreateEnvironmentBlock(
//         &lpEnvironment,
//         Session->UserToken,
//         TRUE))
//     {
//         return FALSE;
//     }
//
//     ret = Session->Gina.Functions.WlxStartApplication(
//         Session->Gina.Context,
//         L"Default",
//         lpEnvironment,
//         L"taskmgr.exe");
//
//     DestroyEnvironmentBlock(lpEnvironment);
//     return ret;
// }
//
// static BOOL
// StartUserShell(
//     IN OUT PWLSESSION Session)
// {
//     LPVOID lpEnvironment = NULL;
//     BOOLEAN Old;
//     BOOL ret;
//
//     /* Create environment block for the user */
//     if (!CreateEnvironmentBlock(&lpEnvironment, Session->UserToken, TRUE))
//     {
//         WARN!("WL: CreateEnvironmentBlock() failed\n");
//         return FALSE;
//     }
//
//     /* Get privilege */
//     /* FIXME: who should do it? winlogon or gina? */
//     /* FIXME: reverting to lower privileges after creating user shell? */
//     RtlAdjustPrivilege(SE_ASSIGNPRIMARYTOKEN_PRIVILEGE, TRUE, FALSE, &Old);
//
//     ret = Session->Gina.Functions.WlxActivateUserShell(
//                 Session->Gina.Context,
//                 L"Default",
//                 NULL, /* FIXME */
//                 lpEnvironment);
//
//     DestroyEnvironmentBlock(lpEnvironment);
//     return ret;
// }
//
//
// BOOL
// SetDefaultLanguage(
//     IN PWLSESSION Session)
// {
//     BOOL ret = FALSE;
//     BOOL UserProfile;
//     LONG rc;
//     HKEY UserKey, hKey = NULL;
//     LPCWSTR SubKey, ValueName;
//     DWORD dwType, dwSize;
//     LPWSTR Value = NULL;
//     UNICODE_STRING ValueString;
//     NTSTATUS Status;
//     LCID Lcid;
//
//     UserProfile = (Session && Session->UserToken);
//
//     if (UserProfile && !ImpersonateLoggedOnUser(Session->UserToken))
//     {
//         ERR!("WL: ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
//         return FALSE;
//         // FIXME: ... or use the default language of the system??
//         // UserProfile = FALSE;
//     }
//
//     if (UserProfile)
//     {
//         rc = RegOpenCurrentUser(MAXIMUM_ALLOWED, &UserKey);
//         if (rc != ERROR_SUCCESS)
//         {
//             TRACE!("RegOpenCurrentUser() failed with error %lu\n", rc);
//             goto cleanup;
//         }
//
//         SubKey = L"Control Panel\\International";
//         ValueName = L"Locale";
//     }
//     else
//     {
//         UserKey = NULL;
//         SubKey = L"System\\CurrentControlSet\\Control\\Nls\\Language";
//         ValueName = L"Default";
//     }
//
//     rc = RegOpenKeyExW(UserKey ? UserKey : HKEY_LOCAL_MACHINE,
//                        SubKey,
//                        0,
//                        KEY_READ,
//                        &hKey);
//
//     if (UserKey)
//         RegCloseKey(UserKey);
//
//     if (rc != ERROR_SUCCESS)
//     {
//         TRACE!("RegOpenKeyEx() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     rc = RegQueryValueExW(hKey,
//                           ValueName,
//                           NULL,
//                           &dwType,
//                           NULL,
//                           &dwSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         TRACE!("RegQueryValueEx() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//     else if (dwType != REG_SZ)
//     {
//         TRACE!("Wrong type for %S\\%S registry entry (got 0x%lx, expected 0x%x)\n",
//             SubKey, ValueName, dwType, REG_SZ);
//         goto cleanup;
//     }
//
//     Value = HeapAlloc(GetProcessHeap(), 0, dwSize);
//     if (!Value)
//     {
//         TRACE!("HeapAlloc() failed\n");
//         goto cleanup;
//     }
//     rc = RegQueryValueExW(hKey,
//                           ValueName,
//                           NULL,
//                           NULL,
//                           (LPBYTE)Value,
//                           &dwSize);
//     if (rc != ERROR_SUCCESS)
//     {
//         TRACE!("RegQueryValueEx() failed with error %lu\n", rc);
//         goto cleanup;
//     }
//
//     /* Convert Value to a Lcid */
//     ValueString.Length = ValueString.MaximumLength = (USHORT)dwSize;
//     ValueString.Buffer = Value;
//     Status = RtlUnicodeStringToInteger(&ValueString, 16, (PULONG)&Lcid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE!("RtlUnicodeStringToInteger() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     TRACE!("%s language is 0x%08lx\n",
//         UserProfile ? "User" : "System", Lcid);
//     Status = NtSetDefaultLocale(UserProfile, Lcid);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE!("NtSetDefaultLocale() failed with status 0x%08lx\n", Status);
//         goto cleanup;
//     }
//
//     ret = TRUE;
//
// cleanup:
//     if (Value)
//         HeapFree(GetProcessHeap(), 0, Value);
//
//     if (hKey)
//         RegCloseKey(hKey);
//
//     if (UserProfile)
//         RevertToSelf();
//
//     return ret;
// }
//
// BOOL
// PlaySoundRoutine(
//     IN LPCWSTR FileName,
//     IN UINT bLogon,
//     IN UINT Flags)
// {
//     typedef BOOL (WINAPI *PLAYSOUNDW)(LPCWSTR,HMODULE,DWORD);
//     typedef UINT (WINAPI *WAVEOUTGETNUMDEVS)(VOID);
//     PLAYSOUNDW Play;
//     WAVEOUTGETNUMDEVS waveOutGetNumDevs;
//     UINT NumDevs;
//     HMODULE hLibrary;
//     BOOL Ret = FALSE;
//
//     hLibrary = LoadLibraryW(L"winmm.dll");
//     if (!hLibrary)
//         return FALSE;
//
//     waveOutGetNumDevs = (WAVEOUTGETNUMDEVS)GetProcAddress(hLibrary, "waveOutGetNumDevs");
//     Play = (PLAYSOUNDW)GetProcAddress(hLibrary, "PlaySoundW");
//
//     _SEH2_TRY
//     {
//         if (waveOutGetNumDevs)
//         {
//             NumDevs = waveOutGetNumDevs();
//             if (!NumDevs)
//             {
//                 if (!bLogon)
//                     Beep(440, 125);
//                 _SEH2_LEAVE;
//             }
//         }
//
//         if (Play)
//             Ret = Play(FileName, NULL, Flags);
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         ERR!("WL: Exception while playing sound '%S', Status 0x%08lx\n",
//             FileName ? FileName : L"(n/a)", _SEH2_GetExceptionCode());
//     }
//     _SEH2_END;
//
//     FreeLibrary(hLibrary);
//
//     return Ret;
// }
//
// static
// DWORD
// WINAPI
// PlayLogonSoundThread(
//      LPVOID lpParameter)
// {
//     PLOGON_SOUND_DATA SoundData = (PLOGON_SOUND_DATA)lpParameter;
//     SERVICE_STATUS_PROCESS Info;
//     DWORD dwSize;
//     ULONG Index = 0;
//     SC_HANDLE hSCManager, hService;
//
//     /* Open the service manager */
//     hSCManager = OpenSCManagerW(NULL, NULL, SC_MANAGER_CONNECT);
//     if (!hSCManager)
//     {
//         ERR!("OpenSCManager failed (%x)\n", GetLastError());
//         goto Cleanup;
//     }
//
//     /* Open the wdmaud service */
//     hService = OpenServiceW(hSCManager, L"wdmaud", GENERIC_READ);
//     if (!hService)
//     {
//         /* The service is not installed */
//         TRACE!("Failed to open wdmaud service (%x)\n", GetLastError());
//         CloseServiceHandle(hSCManager);
//         goto Cleanup;
//     }
//
//     /* Wait for wdmaud to start */
//     do
//     {
//         if (!QueryServiceStatusEx(hService, SC_STATUS_PROCESS_INFO, (LPBYTE)&Info, sizeof(SERVICE_STATUS_PROCESS), &dwSize))
//         {
//             TRACE!("QueryServiceStatusEx failed (%x)\n", GetLastError());
//             break;
//         }
//
//         if (Info.dwCurrentState == SERVICE_RUNNING)
//             break;
//
//         Sleep(1000);
//
//     } while (Index++ < 20);
//
//     CloseServiceHandle(hService);
//     CloseServiceHandle(hSCManager);
//
//     /* If wdmaud is not running exit */
//     if (Info.dwCurrentState != SERVICE_RUNNING)
//     {
//         WARN!("wdmaud has not started!\n");
//         goto Cleanup;
//     }
//
//     /* Sound subsystem is running. Play logon sound. */
//     TRACE!("Playing %s sound\n", SoundData->IsStartup ? "startup" : "logon");
//     if (!ImpersonateLoggedOnUser(SoundData->UserToken))
//     {
//         ERR!("ImpersonateLoggedOnUser failed (%x)\n", GetLastError());
//     }
//     else
//     {
//         PlaySoundRoutine(SoundData->IsStartup ? L"SystemStart" : L"WindowsLogon",
//                          TRUE,
//                          SND_ALIAS | SND_NODEFAULT);
//         RevertToSelf();
//     }
//
// Cleanup:
//     HeapFree(GetProcessHeap(), 0, SoundData);
//     return 0;
// }
//
// static
// VOID
// PlayLogonSound(
//      PWLSESSION Session)
// {
//     PLOGON_SOUND_DATA SoundData;
//     HANDLE hThread;
//
//     SoundData = HeapAlloc(GetProcessHeap(), 0, sizeof(LOGON_SOUND_DATA));
//     if (!SoundData)
//         return;
//
//     SoundData->UserToken = Session->UserToken;
//     SoundData->IsStartup = IsFirstLogon(Session);
//
//     hThread = CreateThread(NULL, 0, PlayLogonSoundThread, SoundData, 0, NULL);
//     if (!hThread)
//     {
//         HeapFree(GetProcessHeap(), 0, SoundData);
//         return;
//     }
//     CloseHandle(hThread);
// }
//
// static
// VOID
// PlayLogoffShutdownSound(
//      PWLSESSION Session,
//      BOOL bShutdown)
// {
//     if (!ImpersonateLoggedOnUser(Session->UserToken))
//         return;
//
//     /* NOTE: Logoff and shutdown sounds play synchronously */
//     PlaySoundRoutine(bShutdown ? L"SystemExit" : L"WindowsLogoff",
//                      FALSE,
//                      SND_ALIAS | SND_NODEFAULT);
//
//     RevertToSelf();
// }
//
// static
// BOOL
// PlayEventSound(
//      PWLSESSION Session,
//      LPCWSTR EventName)
// {
//     BOOL bRet;
//
//     if (!ImpersonateLoggedOnUser(Session->UserToken))
//         return FALSE;
//
//     bRet = PlaySoundRoutine(EventName, FALSE, SND_ALIAS | SND_ASYNC | SND_NODEFAULT);
//
//     RevertToSelf();
//
//     return bRet;
// }
//
// static
// VOID
// RestoreAllConnections(
//      PWLSESSION Session)
// {
//     DWORD dRet;
//     HANDLE hEnum;
//     LPNETRESOURCE lpRes;
//     DWORD dSize = 0x1000;
//     DWORD dCount = -1;
//     LPNETRESOURCE lpCur;
//     BOOL UserProfile;
//
//     UserProfile = (Session && Session->UserToken);
//     if (!UserProfile)
//         return;
//
//     if (!ImpersonateLoggedOnUser(Session->UserToken))
//     {
//         ERR!("WL: ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
//         return;
//     }
//
//     dRet = WNetOpenEnumW(RESOURCE_REMEMBERED, RESOURCETYPE_DISK, 0, NULL, &hEnum);
//     if (dRet != WN_SUCCESS)
//     {
//         ERR!("Failed to open enumeration: %lu\n", dRet);
//         goto quit;
//     }
//
//     lpRes = HeapAlloc(GetProcessHeap(), 0, dSize);
//     if (!lpRes)
//     {
//         ERR!("Failed to allocate memory\n");
//         WNetCloseEnum(hEnum);
//         goto quit;
//     }
//
//     do
//     {
//         dSize = 0x1000;
//         dCount = -1;
//
//         ZeroMemory(lpRes, dSize);
//         dRet = WNetEnumResourceW(hEnum, &dCount, lpRes, &dSize);
//         if (dRet == WN_SUCCESS || dRet == WN_MORE_DATA)
//         {
//             lpCur = lpRes;
//             for (; dCount; dCount--)
//             {
//                 WNetAddConnection2W(lpCur->lpRemoteName, NULL, lpCur->lpLocalName);
//                 lpCur++;
//             }
//         }
//     } while (dRet != WN_NO_MORE_ENTRIES);
//
//     HeapFree(GetProcessHeap(), 0, lpRes);
//     WNetCloseEnum(hEnum);
//
// quit:
//     RevertToSelf();
// }
//
// static
// VOID
// CloseAllConnections(
//      PWLSESSION Session)
// {
//     if (!Session->UserToken || !ImpersonateLoggedOnUser(Session->UserToken))
//         return;
//     WNetClearConnections(NULL);
//     RevertToSelf();
// }
//
// /**
//  * @brief
//  * Frees the Profile information structure (WLX_PROFILE_V1_0
//  * or WLX_PROFILE_V2_0) allocated by the GINA.
//  **/
// static VOID
// FreeWlxProfileInfo(
//      PVOID Profile)
// {
//     PWLX_PROFILE_V2_0 pProfile = (PWLX_PROFILE_V2_0)Profile;
//
//     if (pProfile->dwType != WLX_PROFILE_TYPE_V1_0
//      && pProfile->dwType != WLX_PROFILE_TYPE_V2_0)
//     {
//         ERR!("WL: Wrong profile info\n");
//         return;
//     }
//
//     if (pProfile->pszProfile)
//         LocalFree(pProfile->pszProfile);
//     if (pProfile->dwType >= WLX_PROFILE_TYPE_V2_0)
//     {
//         if (pProfile->pszPolicy)
//             LocalFree(pProfile->pszPolicy);
//         if (pProfile->pszNetworkDefaultUserProfile)
//             LocalFree(pProfile->pszNetworkDefaultUserProfile);
//         if (pProfile->pszServerName)
//             LocalFree(pProfile->pszServerName);
//         if (pProfile->pszEnvironment)
//             LocalFree(pProfile->pszEnvironment);
//     }
// }
//
// /**
//  * @brief
//  * Frees the MPR information structure allocated by the GINA.
//  *
//  * @note
//  * Currently used only in HandleLogon(), but will also be used
//  * by WlxChangePasswordNotify(Ex) once implemented.
//  **/
// static VOID
// FreeWlxMprInfo(
//      PWLX_MPR_NOTIFY_INFO MprNotifyInfo)
// {
//     if (MprNotifyInfo->pszUserName)
//         LocalFree(MprNotifyInfo->pszUserName);
//     if (MprNotifyInfo->pszDomain)
//         LocalFree(MprNotifyInfo->pszDomain);
//     if (MprNotifyInfo->pszPassword)
//     {
//         /* Zero out the password buffer before freeing it */
//         SIZE_T pwdLen = (wcslen(MprNotifyInfo->pszPassword) + 1) * sizeof(WCHAR);
//         SecureZeroMemory(MprNotifyInfo->pszPassword, pwdLen);
//         LocalFree(MprNotifyInfo->pszPassword);
//     }
//     if (MprNotifyInfo->pszOldPassword)
//     {
//         /* Zero out the password buffer before freeing it */
//         SIZE_T pwdLen = (wcslen(MprNotifyInfo->pszOldPassword) + 1) * sizeof(WCHAR);
//         SecureZeroMemory(MprNotifyInfo->pszOldPassword, pwdLen);
//         LocalFree(MprNotifyInfo->pszOldPassword);
//     }
// }
//
// static
// BOOL
// HandleLogon(
//     IN OUT PWLSESSION Session)
// {
//     BOOL ret = FALSE;
//
//     /* Loading personal settings */
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_LOADINGYOURPERSONALSETTINGS);
//
//     Session->hProfileInfo = NULL;
//     if (!(Session->Options & WLX_LOGON_OPT_NO_PROFILE))
//     {
//         HKEY hKey;
//         LONG lError;
//         BOOL bNoPopups = FALSE;
//         PROFILEINFOW ProfileInfo;
//
//         if (Session->Profile == NULL
//          || (Session->Profile->dwType != WLX_PROFILE_TYPE_V1_0
//           && Session->Profile->dwType != WLX_PROFILE_TYPE_V2_0))
//         {
//             ERR!("WL: Wrong profile\n");
//             goto cleanup;
//         }
//
//         /* Check whether error messages may be displayed when loading the user profile */
//         lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                L"System\\CurrentControlSet\\Control\\Windows",
//                                0,
//                                KEY_QUERY_VALUE,
//                                &hKey);
//         if (lError == ERROR_SUCCESS)
//         {
//             DWORD dwValue, dwType, cbData = sizeof(dwValue);
//             lError = RegQueryValueExW(hKey, L"NoPopupsOnBoot", NULL,
//                                       &dwType, (PBYTE)&dwValue, &cbData);
//             if ((lError == ERROR_SUCCESS) && (dwType == REG_DWORD) && (cbData == sizeof(dwValue)))
//                 bNoPopups = !!dwValue;
//
//             RegCloseKey(hKey);
//         }
//
//         /* Load the user profile */
//         ZeroMemory(&ProfileInfo, sizeof(ProfileInfo));
//         ProfileInfo.dwSize = sizeof(ProfileInfo);
//         if (bNoPopups)
//             ProfileInfo.dwFlags |= PI_NOUI;
//         ProfileInfo.lpUserName = Session->MprNotifyInfo.pszUserName;
//         ProfileInfo.lpProfilePath = Session->Profile->pszProfile;
//         if (Session->Profile->dwType >= WLX_PROFILE_TYPE_V2_0)
//         {
//             ProfileInfo.lpDefaultPath = Session->Profile->pszNetworkDefaultUserProfile;
//             ProfileInfo.lpServerName = Session->Profile->pszServerName;
//             ProfileInfo.lpPolicyPath = Session->Profile->pszPolicy;
//         }
//
//         if (!LoadUserProfileW(Session->UserToken, &ProfileInfo))
//         {
//             ERR!("WL: LoadUserProfileW() failed\n");
//             goto cleanup;
//         }
//         Session->hProfileInfo = ProfileInfo.hProfile;
//     }
//
//     /* Cache the username and domain */
//     Session->UserName = WlStrDup(Session->MprNotifyInfo.pszUserName);
//     Session->Domain = WlStrDup(Session->MprNotifyInfo.pszDomain);
//
//     /* Create environment block for the user */
//     if (!CreateUserEnvironment(Session))
//     {
//         WARN!("WL: CreateUserEnvironment() failed\n");
//         goto cleanup;
//     }
//
//     /* Enable per-user settings */
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_APPLYINGYOURPERSONALSETTINGS);
//     UpdatePerUserSystemParameters(0, TRUE);
//
//     /* Set default user language */
//     if (!SetDefaultLanguage(Session))
//     {
//         WARN!("WL: SetDefaultLanguage() failed\n");
//         goto cleanup;
//     }
//
//     /* Allow winsta and desktop access for this session */
//     if (!AllowAccessOnSession(Session))
//     {
//         WARN!("WL: AllowAccessOnSession() failed to give winsta & desktop access for this session\n");
//         goto cleanup;
//     }
//
//     CallNotificationDlls(Session, LogonHandler);
//
//     /* Connect remote resources */
//     RestoreAllConnections(Session);
//
//     /* Start the user shell */
//     CallNotificationDlls(Session, StartShellHandler);
//     if (!StartUserShell(Session))
//     {
//         //WCHAR StatusMsg[256];
//         WARN!("WL: WlxActivateUserShell() failed\n");
//         //LoadStringW(hAppInstance, IDS_FAILEDACTIVATEUSERSHELL, StatusMsg, ARRAYSIZE(StatusMsg));
//         //MessageBoxW(0, StatusMsg, NULL, MB_ICONERROR);
//         goto cleanup;
//     }
//     CallNotificationDlls(Session, PostShellHandler);
//
//     if (!InitializeScreenSaver(Session))
//         WARN!("WL: Failed to initialize screen saver\n");
//
//     /* Logon has succeeded. Play sound. */
//     PlayLogonSound(Session);
//
//     /* NOTE: The logon timestamp has to be set after calling PlayLogonSound
//      * to correctly detect the startup event (first logon) */
//     SetLogonTimestamp(Session);
//     ret = TRUE;
//
// cleanup:
//     if (Session->Profile)
//     {
//         FreeWlxProfileInfo(Session->Profile);
//         LocalFree(Session->Profile);
//         Session->Profile = NULL;
//     }
//     FreeWlxMprInfo(&Session->MprNotifyInfo);
//     ZeroMemory(&Session->MprNotifyInfo, sizeof(Session->MprNotifyInfo));
//
//     RemoveStatusMessage(Session);
//
//     if (!ret)
//     {
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Session->Domain);
//         Session->UserName = Session->Domain = NULL;
//
//         if (Session->hProfileInfo)
//             UnloadUserProfile(Session->UserToken, Session->hProfileInfo);
//         Session->hProfileInfo = NULL;
//
//         /* Restore default system parameters */
//         UpdatePerUserSystemParameters(0, FALSE);
//
//         // TODO: Remove session access to window station
//         // (revert what security.c!AllowAccessOnSession() does).
//         SetWindowStationUser(Session->InteractiveWindowStation,
//                              &LuidNone, NULL, 0);
//
//         /* Switch back to default SYSTEM user */
//         CloseHandle(Session->UserToken);
//         Session->UserToken = NULL;
//         Session->LogonId = LuidNone;
//     }
//     else // if (ret)
//     {
//         SwitchDesktop(Session->ApplicationDesktop);
//         Session->LogonState = STATE_LOGGED_ON;
//     }
//     return ret;
// }
//
//
// static
// DWORD
// WINAPI
// LogoffShutdownThread(
//     LPVOID Parameter)
// {
//     PLOGOFF_SHUTDOWN_DATA LSData = (PLOGOFF_SHUTDOWN_DATA)Parameter;
//     HANDLE UserToken = LSData->Session->UserToken;
//     DWORD ret = TRUE;
//     UINT uFlags;
//
//     if (UserToken && !ImpersonateLoggedOnUser(UserToken))
//     {
//         ERR!("ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     // FIXME: To be really fixed: need to check what needs to be kept and what needs to be removed there.
//     //
//     // uFlags = EWX_INTERNAL_KILL_USER_APPS | (LSData->Flags & EWX_FLAGS_MASK) |
//              // ((LSData->Flags & EWX_ACTION_MASK) == EWX_LOGOFF ? EWX_CALLER_WINLOGON_LOGOFF : 0);
//
//     uFlags = EWX_CALLER_WINLOGON | (LSData->Flags & 0x0F);
//
//     TRACE!("In LogoffShutdownThread with uFlags == 0x%x; exit_in_progress == %s\n",
//         uFlags, ExitReactOSInProgress ? "TRUE" : "FALSE");
//
//     ExitReactOSInProgress = TRUE;
//
//     /* Close processes of the interactive user */
//     if (!ExitWindowsEx(uFlags, 0))
//     {
//         ERR!("Unable to kill user apps, error %lu\n", GetLastError());
//         ret = FALSE;
//     }
//
//     if (UserToken)
//         RevertToSelf();
//
//     return ret;
// }
//
// static
// NTSTATUS
// RunLogoffShutdownThread(
//      PWLSESSION Session,
//      PSECURITY_ATTRIBUTES psa,
//      DWORD wlxAction)
// {
//     PCSTR pDescName;
//     PLOGOFF_SHUTDOWN_DATA LSData;
//     HANDLE hThread;
//     DWORD dwExitCode;
//
//     /* Validate the action */
//     if (WLX_LOGGINGOFF(wlxAction))
//     {
//         pDescName = "Logoff";
//     }
//     else if (WLX_SHUTTINGDOWN(wlxAction))
//     {
//         pDescName = "Shutdown";
//     }
//     else
//     {
//         ASSERT(FALSE);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Prepare data for the logoff/shutdown thread */
//     LSData = HeapAlloc(GetProcessHeap(), 0, sizeof(*LSData));
//     if (!LSData)
//     {
//         ERR!("Failed to allocate %s thread data\n", pDescName);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Set the flags accordingly */
//     if (WLX_LOGGINGOFF(wlxAction))
//     {
//         LSData->Flags = EWX_LOGOFF;
//         if (wlxAction == WLX_SAS_ACTION_FORCE_LOGOFF)
//             LSData->Flags |= EWX_FORCE;
//     }
//     else // if (WLX_SHUTTINGDOWN(wlxAction))
//     {
//         /* Because we are shutting down the OS, force processes termination too */
//         LSData->Flags = EWX_SHUTDOWN | EWX_FORCE;
//         if (wlxAction == WLX_SAS_ACTION_SHUTDOWN_POWER_OFF)
//             LSData->Flags |= EWX_POWEROFF;
//         else if (wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT)
//             LSData->Flags |= EWX_REBOOT;
//     }
//
//     LSData->Session = Session;
//
//     /* Run the logoff/shutdown thread */
//     hThread = CreateThread(psa, 0, LogoffShutdownThread, LSData, 0, NULL);
//     if (!hThread)
//     {
//         ERR!("Unable to create %s thread, error %lu\n", pDescName, GetLastError());
//         HeapFree(GetProcessHeap(), 0, LSData);
//         return STATUS_UNSUCCESSFUL;
//     }
//     WaitForSingleObject(hThread, INFINITE);
//     HeapFree(GetProcessHeap(), 0, LSData);
//     if (!GetExitCodeThread(hThread, &dwExitCode))
//     {
//         ERR!("Unable to get %s thread exit code (error %lu)\n", pDescName, GetLastError());
//         CloseHandle(hThread);
//         return STATUS_UNSUCCESSFUL;
//     }
//     CloseHandle(hThread);
//     if (dwExitCode == 0)
//     {
//         ERR!("%s thread returned failure\n", pDescName);
//         return STATUS_UNSUCCESSFUL;
//     }
//     return STATUS_SUCCESS;
// }
//
// static
// DWORD
// WINAPI
// KillComProcesses(
//     LPVOID Parameter)
// {
//     HANDLE UserToken = (HANDLE)Parameter;
//     DWORD ret = TRUE;
//
//     TRACE!("In KillComProcesses\n");
//
//     if (UserToken && !ImpersonateLoggedOnUser(UserToken))
//     {
//         ERR!("ImpersonateLoggedOnUser() failed with error %lu\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Attempt to kill remaining processes. No notifications needed. */
//     if (!ExitWindowsEx(EWX_CALLER_WINLOGON | EWX_NONOTIFY | EWX_FORCE | EWX_LOGOFF, 0))
//     {
//         ERR!("Unable to kill COM apps, error %lu\n", GetLastError());
//         ret = FALSE;
//     }
//
//     if (UserToken)
//         RevertToSelf();
//
//     return ret;
// }
//
//
// static
// NTSTATUS
// CreateLogoffSecurityAttributes(
//     OUT PSECURITY_ATTRIBUTES* ppsa)
// {
//     /* The following code is not working yet and messy */
//     /* Still, it gives some ideas about data types and functions involved and */
//     /* required to set up a SECURITY_DESCRIPTOR for a SECURITY_ATTRIBUTES */
//     /* instance for a thread, to allow that  thread to ImpersonateLoggedOnUser(). */
//     /* Specifically THREAD_SET_THREAD_TOKEN is required. */
//     PSECURITY_DESCRIPTOR SecurityDescriptor = NULL;
//     PSECURITY_ATTRIBUTES psa = 0;
//     BYTE* pMem;
//     PACL pACL;
//     EXPLICIT_ACCESS Access;
//     PSID pEveryoneSID = NULL;
//     static SID_IDENTIFIER_AUTHORITY WorldAuthority = { SECURITY_WORLD_SID_AUTHORITY };
//
//     *ppsa = NULL;
//
//     // Let's first try to enumerate what kind of data we need for this to ever work:
//     // 1.  The Winlogon SID, to be able to give it THREAD_SET_THREAD_TOKEN.
//     // 2.  The users SID (the user trying to logoff, or rather shut down the system).
//     // 3.  At least two EXPLICIT_ACCESS instances:
//     // 3.1 One for Winlogon itself, giving it the rights
//     //     required to THREAD_SET_THREAD_TOKEN (as it's needed to successfully call
//     //     ImpersonateLoggedOnUser).
//     // 3.2 One for the user, to allow *that* thread to perform its work.
//     // 4.  An ACL to hold the these EXPLICIT_ACCESS ACE's.
//     // 5.  A SECURITY_DESCRIPTOR to hold the ACL, and finally.
//     // 6.  A SECURITY_ATTRIBUTES instance to pull all of this required stuff
//     //     together, to hand it to CreateThread.
//     //
//     // However, it seems struct LOGOFF_SHUTDOWN_DATA doesn't contain
//     // these required SID's, why they'd have to be added.
//     // The Winlogon's own SID should probably only be created once,
//     // while the user's SID obviously must be created for each new user.
//     // Might as well store it when the user logs on?
//
//     if(!AllocateAndInitializeSid(&WorldAuthority,
//                                  1,
//                                  SECURITY_WORLD_RID,
//                                  0, 0, 0, 0, 0, 0, 0,
//                                  &pEveryoneSID))
//     {
//         ERR!("Failed to initialize security descriptor for logoff thread!\n");
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* set up the required security attributes to be able to shut down */
//     /* To save space and time, allocate a single block of memory holding */
//     /* both SECURITY_ATTRIBUTES and SECURITY_DESCRIPTOR */
//     pMem = HeapAlloc(GetProcessHeap(),
//                      0,
//                      sizeof(SECURITY_ATTRIBUTES) +
//                      SECURITY_DESCRIPTOR_MIN_LENGTH +
//                      sizeof(ACL));
//     if (!pMem)
//     {
//         ERR!("Failed to allocate memory for logoff security descriptor!\n");
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Note that the security descriptor needs to be in _absolute_ format, */
//     /* meaning its members must be pointers to other structures, rather */
//     /* than the relative format using offsets */
//     psa = (PSECURITY_ATTRIBUTES)pMem;
//     SecurityDescriptor = (PSECURITY_DESCRIPTOR)(pMem + sizeof(SECURITY_ATTRIBUTES));
//     pACL = (PACL)(((PBYTE)SecurityDescriptor) + SECURITY_DESCRIPTOR_MIN_LENGTH);
//
//     // Initialize an EXPLICIT_ACCESS structure for an ACE.
//     // The ACE will allow this thread to log off (and shut down the system, currently).
//     ZeroMemory(&Access, sizeof(Access));
//     Access.grfAccessPermissions = THREAD_SET_THREAD_TOKEN;
//     Access.grfAccessMode = SET_ACCESS; // GRANT_ACCESS?
//     Access.grfInheritance = NO_INHERITANCE;
//     Access.Trustee.TrusteeForm = TRUSTEE_IS_SID;
//     Access.Trustee.TrusteeType = TRUSTEE_IS_WELL_KNOWN_GROUP;
//     Access.Trustee.ptstrName = pEveryoneSID;
//
//     if (SetEntriesInAclW(1, &Access, NULL, &pACL) != ERROR_SUCCESS)
//     {
//         ERR!("Failed to set Access Rights for logoff thread. Logging out will most likely fail.\n");
//
//         HeapFree(GetProcessHeap(), 0, pMem);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (!InitializeSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION))
//     {
//         ERR!("Failed to initialize security descriptor for logoff thread!\n");
//         HeapFree(GetProcessHeap(), 0, pMem);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (!SetSecurityDescriptorDacl(SecurityDescriptor,
//                                    TRUE,     // bDaclPresent flag
//                                    pACL,
//                                    FALSE))   // not a default DACL
//     {
//         ERR!("SetSecurityDescriptorDacl Error %lu\n", GetLastError());
//         HeapFree(GetProcessHeap(), 0, pMem);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     psa->nLength = sizeof(SECURITY_ATTRIBUTES);
//     psa->lpSecurityDescriptor = SecurityDescriptor;
//     psa->bInheritHandle = FALSE;
//
//     *ppsa = psa;
//
//     return STATUS_SUCCESS;
// }
//
// static
// VOID
// DestroyLogoffSecurityAttributes(
//     IN PSECURITY_ATTRIBUTES psa)
// {
//     if (psa)
//     {
//         HeapFree(GetProcessHeap(), 0, psa);
//     }
// }
//
//
// static
// NTSTATUS
// HandleLogoff(
//      PWLSESSION Session,
//      DWORD wlxAction)
// {
//     PSECURITY_ATTRIBUTES psa;
//     HANDLE hThread;
//     NTSTATUS Status;
//
//     Status = CreateLogoffSecurityAttributes(&psa);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Failed to create Logoff security descriptor. Status 0x%08lx\n", Status);
//         return Status;
//     }
//
//     /* Run the Logoff thread. Log off as well if we are
//      * invoked as part of a shutdown operation. */
//     Status = RunLogoffShutdownThread(Session, psa,
//                                      WLX_LOGGINGOFF(wlxAction)
//                                         ? wlxAction
//                                         : WLX_SAS_ACTION_LOGOFF);
//     if (!NT_SUCCESS(Status))
//     {
//         ERR!("Failed to start the Logoff thread, Status 0x%08lx\n", Status);
//         DestroyLogoffSecurityAttributes(psa);
//         return Status;
//     }
//
//     /* Invoke Logoff notifications on the application desktop */
//     SwitchDesktop(Session->ApplicationDesktop);
//     DisplayStatusMessage(Session, Session->ApplicationDesktop, IDS_LOGGINGOFF);
//     CallNotificationDlls(Session, LogoffHandler);
//     RemoveStatusMessage(Session);
//
//     /* The remaining Logoff steps run on the Winlogon desktop */
//     SwitchDesktop(Session->WinlogonDesktop);
//
//     PlayLogoffShutdownSound(Session, WLX_SHUTTINGDOWN(wlxAction));
//
//     /* Close all user network connections */
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_CLOSINGNETWORKCONNECTIONS);
//     CloseAllConnections(Session);
//     // TODO: Do any other user-specific network-related cleaning:
//     // user-added NetAPI message aliases; user cached credentials (remote login)...
//
//     SetWindowStationUser(Session->InteractiveWindowStation,
//                          &LuidNone, NULL, 0);
//
//     /* Kill remaining COM processes that may have been started by logoff scripts */
//     hThread = CreateThread(psa, 0, KillComProcesses, (PVOID)Session->UserToken, 0, NULL);
//     if (hThread)
//     {
//         WaitForSingleObject(hThread, INFINITE);
//         CloseHandle(hThread);
//     }
//
//     /* We're done with the SECURITY_DESCRIPTOR */
//     DestroyLogoffSecurityAttributes(psa);
//
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_SAVEYOURSETTINGS);
//
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Session->UserName);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Session->Domain);
//     Session->UserName = Session->Domain = NULL;
//
//     if (Session->hProfileInfo)
//         UnloadUserProfile(Session->UserToken, Session->hProfileInfo);
//     Session->hProfileInfo = NULL;
//
//     /* Restore default system parameters */
//     UpdatePerUserSystemParameters(0, FALSE);
//
//     // TODO: Remove session access to window station
//     // (revert what security.c!AllowAccessOnSession() does).
//
//     /* Switch back to default SYSTEM user */
//     CloseHandle(Session->UserToken);
//     Session->UserToken = NULL;
//     Session->LogonId = LuidNone;
//
//     Session->LogonState = STATE_LOGGED_OFF;
//
//     return STATUS_SUCCESS;
// }
//
// static
// INT_PTR
// __stdcall
// ShutdownComputerWindowProc(
//     IN HWND hwndDlg,
//     IN UINT uMsg,
//     IN WPARAM wParam,
//     IN LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//
//     switch (uMsg)
//     {
//         case WM_COMMAND:
//         {
//             switch (LOWORD(wParam))
//             {
//                 case IDC_BTNSHTDOWNCOMPUTER:
//                     EndDialog(hwndDlg, IDC_BTNSHTDOWNCOMPUTER);
//                     return TRUE;
//             }
//             break;
//         }
//         case WM_INITDIALOG:
//         {
//             /* Remove the Close menu item */
//             DeleteMenu(GetSystemMenu(hwndDlg, FALSE), SC_CLOSE, MF_BYCOMMAND);
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
// static
// VOID
// UninitializeSAS(
//     IN OUT PWLSESSION Session)
// {
//     if (Session->SASWindow)
//     {
//         DestroyWindow(Session->SASWindow);
//         Session->SASWindow = NULL;
//     }
//     if (Session->hEndOfScreenSaverThread)
//         SetEvent(Session->hEndOfScreenSaverThread);
//     UnregisterClassW(WINLOGON_SAS_CLASS, hAppInstance);
// }
//
// NTSTATUS
// HandleShutdown(
//     IN OUT PWLSESSION Session,
//     IN DWORD wlxAction)
// {
//     NTSTATUS Status;
//     UINT uMsgId;
//     BOOLEAN Old;
//
//     /* Display the appropriate shutdown or reboot message */
//     if (wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT)
//         uMsgId = IDS_REACTOSISRESTARTING;
//     else
//         uMsgId = IDS_REACTOSISSHUTTINGDOWN;
//
//     // SwitchDesktop(Session->WinlogonDesktop);
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, uMsgId);
//
//     /* Invoke Shutdown notifications and notify GINA */
//     CallNotificationDlls(Session, ShutdownHandler);
//     Session->Gina.Functions.WlxShutdown(Session->Gina.Context, wlxAction);
//
//     /* Run the shutdown thread. *IGNORE* all failures as we want to force shutting down! */
//     Status = RunLogoffShutdownThread(Session, NULL, wlxAction);
//     if (!NT_SUCCESS(Status))
//         ERR!("Failed to start the Shutdown thread, Status 0x%08lx\n", Status);
//
//     /* Show again the shutdown message */
//     // SwitchDesktop(Session->WinlogonDesktop); // Re-enable if you notice the desktop may have switched to something else.
//     DisplayStatusMessage(Session, Session->WinlogonDesktop, uMsgId);
//
//     /* Destroy SAS window */
//     UninitializeSAS(Session);
//
//     /* Now we can shut down NT */
//     ERR!("Shutting down NT...\n");
//     RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, TRUE, FALSE, &Old);
//     if (wlxAction == WLX_SAS_ACTION_SHUTDOWN_REBOOT)
//     {
//         NtShutdownSystem(ShutdownReboot);
//     }
//     else
//     {
//         if (FALSE)
//         {
//             /* FIXME - only show this dialog if it's a shutdown and the computer doesn't support APM */
//             DialogBox(hAppInstance, MAKEINTRESOURCE(IDD_SHUTDOWNCOMPUTER),
//                       GetDesktopWindow(), ShutdownComputerWindowProc);
//         }
//         if (wlxAction == WLX_SAS_ACTION_SHUTDOWN_POWER_OFF)
//             NtShutdownSystem(ShutdownPowerOff);
//         else // if (wlxAction == WLX_SAS_ACTION_SHUTDOWN)
//             NtShutdownSystem(ShutdownNoReboot);
//     }
//     RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, Old, FALSE, &Old);
//     return STATUS_SUCCESS;
// }
//
// static
// VOID
// DoGenericAction(
//     IN OUT PWLSESSION Session,
//     IN DWORD wlxAction)
// {
//     switch (wlxAction)
//     {
//         case WLX_SAS_ACTION_LOGON: /* 0x01 */
//             if (Session->LogonState == STATE_LOGGED_OFF_SAS)
//             {
//                 if (!HandleLogon(Session))
//                 {
//                     Session->LogonState = STATE_LOGGED_OFF;
//                     Session->Gina.Functions.WlxDisplaySASNotice(Session->Gina.Context);
//                 }
//             }
//             break;
//         case WLX_SAS_ACTION_NONE: /* 0x02 */
//             if (Session->LogonState == STATE_LOGGED_OFF_SAS)
//             {
//                 Session->LogonState = STATE_LOGGED_OFF;
//                 Session->Gina.Functions.WlxDisplaySASNotice(Session->Gina.Context);
//             }
//             else if (Session->LogonState == STATE_LOGGED_ON_SAS)
//             {
//                 Session->LogonState = STATE_LOGGED_ON;
//             }
//             else if (Session->LogonState == STATE_LOCKED_SAS)
//             {
//                 Session->LogonState = STATE_LOCKED;
//                 Session->Gina.Functions.WlxDisplayLockedNotice(Session->Gina.Context);
//             }
//             break;
//         case WLX_SAS_ACTION_LOCK_WKSTA: /* 0x03 */
//             if ((Session->LogonState == STATE_LOGGED_ON) ||
//                 (Session->LogonState == STATE_LOGGED_ON_SAS))
//             {
//                 if (Session->Gina.Functions.WlxIsLockOk(Session->Gina.Context))
//                 {
//                     Session->LogonState = STATE_LOCKED;
//                     SwitchDesktop(Session->WinlogonDesktop);
//                     /* We may be on the Logged-On SAS dialog, in which case
//                      * we need to close it if the lock action came via Win-L */
//                     CloseAllDialogWindows();
//                     CallNotificationDlls(Session, LockHandler);
//                     Session->Gina.Functions.WlxDisplayLockedNotice(Session->Gina.Context);
//                 }
//             }
//             break;
//         case WLX_SAS_ACTION_LOGOFF: /* 0x04 */
//         case WLX_SAS_ACTION_SHUTDOWN: /* 0x05 */
//         case WLX_SAS_ACTION_FORCE_LOGOFF: /* 0x09 */
//         case WLX_SAS_ACTION_SHUTDOWN_POWER_OFF: /* 0x0a */
//         case WLX_SAS_ACTION_SHUTDOWN_REBOOT: /* 0x0b */
//             if ((Session->LogonState != STATE_INIT) &&
//                 (Session->LogonState != STATE_LOGGED_OFF) &&
//                 (Session->LogonState != STATE_LOGGED_OFF_SAS) &&
//                 (Session->LogonState != STATE_SHUT_DOWN))
//             {
//                 ASSERT((Session->LogonState == STATE_LOGGED_ON) ||
//                        (Session->LogonState == STATE_LOGGED_ON_SAS) ||
//                        (Session->LogonState == STATE_LOCKED) ||
//                        (Session->LogonState == STATE_LOCKED_SAS));
//
//                 if (!Session->Gina.Functions.WlxIsLogoffOk(Session->Gina.Context))
//                     break;
//                 if (!NT_SUCCESS(HandleLogoff(Session, wlxAction)))
//                 {
//                     RemoveStatusMessage(Session);
//                     break;
//                 }
//                 Session->Gina.Functions.WlxLogoff(Session->Gina.Context);
//             }
//             if (WLX_SHUTTINGDOWN(wlxAction))
//             {
//                 if (!NT_SUCCESS(HandleShutdown(Session, wlxAction)))
//                 {
//                     RemoveStatusMessage(Session);
//                     Session->LogonState = STATE_LOGGED_OFF;
//                     Session->Gina.Functions.WlxDisplaySASNotice(Session->Gina.Context);
//                 }
//             }
//             else
//             {
//                 RemoveStatusMessage(Session);
//                 Session->LogonState = STATE_LOGGED_OFF;
//                 Session->Gina.Functions.WlxDisplaySASNotice(Session->Gina.Context);
//             }
//             break;
//         case WLX_SAS_ACTION_TASKLIST: /* 0x07 */
//             if ((Session->LogonState == STATE_LOGGED_ON) ||
//                 (Session->LogonState == STATE_LOGGED_ON_SAS))
//             {
//                 /* Start a Task-Manager instance on the application desktop.
//                  * If the user pressed Ctrl-Shift-Esc while being on the
//                  * Logged-On SAS dialog (on the Winlogon desktop), stay there. */
//                 StartTaskManager(Session);
//             }
//             break;
//         case WLX_SAS_ACTION_UNLOCK_WKSTA: /* 0x08 */
//             if ((Session->LogonState == STATE_LOCKED) ||
//                 (Session->LogonState == STATE_LOCKED_SAS))
//             {
//                 CallNotificationDlls(Session, UnlockHandler);
//                 SwitchDesktop(Session->ApplicationDesktop);
//                 Session->LogonState = STATE_LOGGED_ON;
//             }
//             break;
//         default:
//             WARN!("Unknown SAS action 0x%lx\n", wlxAction);
//     }
// }
//
// static
// VOID
// DispatchSAS(
//     IN OUT PWLSESSION Session,
//     IN DWORD dwSasType)
// {
//     DWORD wlxAction = WLX_SAS_ACTION_NONE;
//     PSID LogonSid = NULL; /* FIXME */
//     BOOL bSecure = TRUE;
//
//     switch (dwSasType)
//     {
//         case WLX_SAS_TYPE_CTRL_ALT_DEL:
//             switch (Session->LogonState)
//             {
//                 case STATE_INIT:
//                     Session->LogonState = STATE_LOGGED_OFF;
//                     Session->Gina.Functions.WlxDisplaySASNotice(Session->Gina.Context);
//                     return;
//
//                 case STATE_LOGGED_OFF:
//                     Session->LogonState = STATE_LOGGED_OFF_SAS;
//
//                     CloseAllDialogWindows();
//
//                     Session->Options = 0;
//
//                     wlxAction = (DWORD)Session->Gina.Functions.WlxLoggedOutSAS(
//                         Session->Gina.Context,
//                         Session->SASAction,
//                         &Session->LogonId,
//                         LogonSid,
//                         &Session->Options,
//                         &Session->UserToken,
//                         &Session->MprNotifyInfo,
//                         (PVOID*)&Session->Profile);
//                     break;
//
//                 case STATE_LOGGED_OFF_SAS:
//                     /* Ignore SAS if we are already in an SAS state */
//                     return;
//
//                 case STATE_LOGGED_ON:
//                     Session->LogonState = STATE_LOGGED_ON_SAS;
//                     SwitchDesktop(Session->WinlogonDesktop);
//                     wlxAction = (DWORD)Session->Gina.Functions.WlxLoggedOnSAS(Session->Gina.Context, dwSasType, NULL);
//                     if ((wlxAction == WLX_SAS_ACTION_NONE) ||
//                         (wlxAction == WLX_SAS_ACTION_TASKLIST))
//                     {
//                         /*
//                          * If the user canceled (WLX_SAS_ACTION_NONE) the
//                          * Logged-On SAS dialog, or clicked on the Task-Manager
//                          * button (WLX_SAS_ACTION_TASKLIST), switch back to
//                          * the application desktop and return to log-on state.
//                          * In the latter case, the Task-Manager is launched
//                          * by DoGenericAction(WLX_SAS_ACTION_TASKLIST), which
//                          * doesn't automatically do the switch back, because
//                          * the user may have also pressed on Ctrl-Shift-Esc
//                          * to start it while being on the Logged-On SAS dialog
//                          * and wanting to stay there.
//                          */
//                         SwitchDesktop(Session->ApplicationDesktop);
//                         Session->LogonState = STATE_LOGGED_ON;
//                     }
//                     break;
//
//                 case STATE_LOGGED_ON_SAS:
//                     /* Ignore SAS if we are already in an SAS state */
//                     return;
//
//                 case STATE_LOCKED:
//                     Session->LogonState = STATE_LOCKED_SAS;
//
//                     CloseAllDialogWindows();
//
//                     wlxAction = (DWORD)Session->Gina.Functions.WlxWkstaLockedSAS(Session->Gina.Context, dwSasType);
//                     break;
//
//                 case STATE_LOCKED_SAS:
//                     /* Ignore SAS if we are already in an SAS state */
//                     return;
//
//                 default:
//                     return;
//             }
//             break;
//
//         case WLX_SAS_TYPE_TIMEOUT:
//             return;
//
//         case WLX_SAS_TYPE_SCRNSVR_TIMEOUT:
//             if (!Session->Gina.Functions.WlxScreenSaverNotify(Session->Gina.Context, &bSecure))
//             {
//                 /* Skip start of screen saver */
//                 SetEvent(Session->hEndOfScreenSaver);
//             }
//             else
//             {
//                 StartScreenSaver(Session);
//                 if (bSecure)
//                 {
//                     wlxAction = WLX_SAS_ACTION_LOCK_WKSTA;
// //                    DoGenericAction(Session, WLX_SAS_ACTION_LOCK_WKSTA);
//                 }
//             }
//             break;
//
//         case WLX_SAS_TYPE_SCRNSVR_ACTIVITY:
//             SetEvent(Session->hUserActivity);
//             break;
//     }
//
//     DoGenericAction(Session, wlxAction);
// }
//
// static
// BOOL
// RegisterHotKeys(
//     IN PWLSESSION Session,
//     IN HWND hwndSAS)
// {
//     /* Register Ctrl+Alt+Del hotkey */
//     if (!RegisterHotKey(hwndSAS, IDHK_CTRL_ALT_DEL, MOD_CONTROL | MOD_ALT, VK_DELETE))
//     {
//         ERR!("WL: Unable to register Ctrl+Alt+Del hotkey\n");
//         return FALSE;
//     }
//
//     /* Register Ctrl+Shift+Esc "Task Manager" hotkey (optional) */
//     Session->TaskManHotkey = RegisterHotKey(hwndSAS, IDHK_CTRL_SHIFT_ESC, MOD_CONTROL | MOD_SHIFT, VK_ESCAPE);
//     if (!Session->TaskManHotkey)
//         WARN!("WL: Unable to register Ctrl+Shift+Esc hotkey\n");
//
//     /* Register Win+L "Lock Workstation" hotkey (optional) */
//     Session->LockWkStaHotkey = RegisterHotKey(hwndSAS, IDHK_WIN_L, MOD_WIN, 'L');
//     if (!Session->LockWkStaHotkey)
//         WARN!("WL: Unable to register Win+L hotkey\n");
//
//     /* Register Win+U "Accessibility Utility" hotkey (optional) */
//     Session->UtilManHotkey = RegisterHotKey(hwndSAS, IDHK_WIN_U, MOD_WIN, 'U');
//     if (!Session->UtilManHotkey)
//         WARN!("WL: Unable to register Win+U hotkey\n");
//
//     return TRUE;
// }
//
// static
// BOOL
// UnregisterHotKeys(
//     IN PWLSESSION Session,
//     IN HWND hwndSAS)
// {
//     /* Unregister the hotkeys */
//     UnregisterHotKey(hwndSAS, IDHK_CTRL_ALT_DEL);
//
//     if (Session->TaskManHotkey)
//         UnregisterHotKey(hwndSAS, IDHK_CTRL_SHIFT_ESC);
//
//     if (Session->LockWkStaHotkey)
//         UnregisterHotKey(hwndSAS, IDHK_WIN_L);
//
//     if (Session->UtilManHotkey)
//         UnregisterHotKey(hwndSAS, IDHK_WIN_U);
//
//     return TRUE;
// }
//
// static
// BOOL
// HandleMessageBeep(
//      PWLSESSION Session,
//      UINT uType)
// {
//     LPWSTR EventName;
//
//     switch (uType)
//     {
//         case 0xFFFFFFFF:
//             EventName = NULL;
//             break;
//         case MB_OK:
//             EventName = L"SystemDefault";
//             break;
//         case MB_ICONASTERISK:
//             EventName = L"SystemAsterisk";
//             break;
//         case MB_ICONEXCLAMATION:
//             EventName = L"SystemExclamation";
//             break;
//         case MB_ICONHAND:
//             EventName = L"SystemHand";
//             break;
//         case MB_ICONQUESTION:
//             EventName = L"SystemQuestion";
//             break;
//         default:
//             WARN!("Unhandled type %d\n", uType);
//             EventName = L"SystemDefault";
//     }
//
//     return PlayEventSound(Session, EventName);
// }
//
// static
// LRESULT
// __stdcall
// SASWindowProc(
//     IN HWND hwndDlg,
//     IN UINT uMsg,
//     IN WPARAM wParam,
//     IN LPARAM lParam)
// {
//     PWLSESSION Session = (PWLSESSION)GetWindowLongPtr(hwndDlg, GWLP_USERDATA);
//
//     switch (uMsg)
//     {
//         case WM_HOTKEY:
//         {
//             switch (wParam)
//             {
//                 case IDHK_CTRL_ALT_DEL:
//                 {
//                     TRACE!("SAS: CONTROL+ALT+DELETE\n");
//                     if (!Session->Gina.UseCtrlAltDelete)
//                         break;
//                     PostMessageW(Session->SASWindow, WLX_WM_SAS, WLX_SAS_TYPE_CTRL_ALT_DEL, 0);
//                     return TRUE;
//                 }
//                 case IDHK_CTRL_SHIFT_ESC:
//                 {
//                     TRACE!("SAS: CONTROL+SHIFT+ESCAPE\n");
//                     DoGenericAction(Session, WLX_SAS_ACTION_TASKLIST);
//                     return TRUE;
//                 }
//                 case IDHK_WIN_L:
//                 {
//                     TRACE!("SAS: WIN+L\n");
//                     PostMessageW(Session->SASWindow, WM_LOGONNOTIFY, LN_LOCK_WORKSTATION, 0);
//                     return TRUE;
//                 }
//                 case IDHK_WIN_U:
//                 {
//                     TRACE!("SAS: WIN+U\n");
//                     // PostMessageW(Session->SASWindow, WM_LOGONNOTIFY, LN_ACCESSIBILITY, 0);
//                     return TRUE;
//                 }
//             }
//             break;
//         }
//         case WM_CREATE:
//         {
//             /* Get the session pointer from the create data */
//             Session = (PWLSESSION)((LPCREATESTRUCT)lParam)->lpCreateParams;
//
//             /* Save the Session pointer */
//             SetWindowLongPtrW(hwndDlg, GWLP_USERDATA, (LONG_PTR)Session);
//             if (GetSetupType())
//                 return TRUE;
//             return RegisterHotKeys(Session, hwndDlg);
//         }
//         case WM_DESTROY:
//         {
//             if (!GetSetupType())
//                 UnregisterHotKeys(Session, hwndDlg);
//             PostQuitMessage(0);
//             return TRUE;
//         }
//         case WM_SETTINGCHANGE:
//         {
//             UINT uiAction = (UINT)wParam;
//             if (uiAction == SPI_SETSCREENSAVETIMEOUT
//              || uiAction == SPI_SETSCREENSAVEACTIVE)
//             {
//                 SetEvent(Session->hScreenSaverParametersChanged);
//             }
//             return TRUE;
//         }
//         case WM_LOGONNOTIFY:
//         {
//             switch(wParam)
//             {
//                 case LN_MESSAGE_BEEP:
//                 {
//                     return HandleMessageBeep(Session, lParam);
//                 }
//                 case LN_SHELL_EXITED:
//                 {
//                     /* lParam is the exit code */
//                     if (lParam != 1 &&
//                         Session->LogonState != STATE_LOGGED_OFF &&
//                         Session->LogonState != STATE_LOGGED_OFF_SAS)
//                     {
//                         SetTimer(hwndDlg, 1, 1000, NULL);
//                     }
//                     break;
//                 }
//                 case LN_START_SCREENSAVE:
//                 {
//                     DispatchSAS(Session, WLX_SAS_TYPE_SCRNSVR_TIMEOUT);
//                     break;
//                 }
//                 case LN_LOCK_WORKSTATION:
//                 {
//                     DoGenericAction(Session, WLX_SAS_ACTION_LOCK_WKSTA);
//                     break;
//                 }
//                 case LN_LOGOFF:
//                 {
//                     UINT Flags = (UINT)lParam;
//                     UINT Action = Flags & EWX_ACTION_MASK;
//                     DWORD wlxAction;
//
//                     TRACE!("\tFlags : 0x%lx\n", lParam);
//
//                     /*
//                      * Our caller (USERSRV) should have added the shutdown flag
//                      * when setting also poweroff or reboot.
//                      */
//                     if ((Action & (EWX_POWEROFF | EWX_REBOOT)) && !(Action & EWX_SHUTDOWN))
//                     {
//                         ERR!("Missing EWX_SHUTDOWN flag for poweroff or reboot; action 0x%x\n", Action);
//                         return STATUS_INVALID_PARAMETER;
//                     }
//
//                     // INVESTIGATE: Our HandleLogoff/HandleShutdown may instead
//                     // take an EWX_* flags combination to determine what to do
//                     // more precisely.
//                     /* Map EWX_* flags to WLX_* actions and check for any unhandled flag */
//                     if (Action & EWX_POWEROFF)
//                     {
//                         wlxAction = WLX_SAS_ACTION_SHUTDOWN_POWER_OFF;
//                         Action &= ~(EWX_SHUTDOWN | EWX_POWEROFF);
//                     }
//                     else if (Action & EWX_REBOOT)
//                     {
//                         wlxAction = WLX_SAS_ACTION_SHUTDOWN_REBOOT;
//                         Action &= ~(EWX_SHUTDOWN | EWX_REBOOT);
//                     }
//                     else if (Action & EWX_SHUTDOWN)
//                     {
//                         wlxAction = WLX_SAS_ACTION_SHUTDOWN;
//                         Action &= ~EWX_SHUTDOWN;
//                     }
//                     else // EWX_LOGOFF
//                     {
//                         if (Action & EWX_FORCE)
//                             wlxAction = WLX_SAS_ACTION_FORCE_LOGOFF;
//                         else
//                             wlxAction = WLX_SAS_ACTION_LOGOFF;
//                         Action &= ~(EWX_LOGOFF | EWX_FORCE);
//                     }
//                     if (Action)
//                         ERR!("Unhandled EWX_* action flags: 0x%x\n", Action);
//
//                     TRACE!("In LN_LOGOFF, exit_in_progress == %s\n",
//                         ExitReactOSInProgress ? "TRUE" : "FALSE");
//
//                     /*
//                      * In case a parallel shutdown request is done (while we are
//                      * being to shut down) and it was not done by Winlogon itself,
//                      * then just stop here.
//                      */
//                     /* Now do the shutdown action proper */
//                     DoGenericAction(Session, wlxAction);
//                     return 1;
//                 }
//                 case LN_LOGOFF_CANCELED:
//                 {
//                     ERR!("Logoff canceled! Before: exit_in_progress == %s; After: FALSE\n",
//                         ExitReactOSInProgress ? "TRUE" : "FALSE");
//
//                     ExitReactOSInProgress = FALSE;
//                     return 1;
//                 }
//                 default:
//                 {
//                     ERR!("WM_LOGONNOTIFY case %d is unimplemented\n", wParam);
//                 }
//             }
//             return 0;
//         }
//         case WM_TIMER:
//         {
//             if (wParam == 1)
//             {
//                 KillTimer(hwndDlg, 1);
//                 StartUserShell(Session);
//             }
//             break;
//         }
//         case WLX_WM_SAS:
//         {
//             DispatchSAS(Session, (DWORD)wParam);
//             return TRUE;
//         }
//     }
//
//     return DefWindowProc(hwndDlg, uMsg, wParam, lParam);
// }
//
// BOOL
// InitializeSAS(
//     IN OUT PWLSESSION Session)
// {
//     WNDCLASSEXW swc;
//     BOOL ret = FALSE;
//
//     if (!SwitchDesktop(Session->WinlogonDesktop))
//     {
//         ERR!("WL: Failed to switch to winlogon desktop\n");
//         goto cleanup;
//     }
//
//     /* Register SAS window class */
//     swc.cbSize = sizeof(WNDCLASSEXW);
//     swc.style = CS_SAVEBITS;
//     swc.lpfnWndProc = SASWindowProc;
//     swc.cbClsExtra = 0;
//     swc.cbWndExtra = 0;
//     swc.hInstance = hAppInstance;
//     swc.hIcon = NULL;
//     swc.hCursor = NULL;
//     swc.hbrBackground = NULL;
//     swc.lpszMenuName = NULL;
//     swc.lpszClassName = WINLOGON_SAS_CLASS;
//     swc.hIconSm = NULL;
//     if (RegisterClassExW(&swc) == 0)
//     {
//         ERR!("WL: Failed to register SAS window class\n");
//         goto cleanup;
//     }
//
//     /* Create invisible SAS window */
//     Session->SASWindow = CreateWindowExW(
//         0,
//         WINLOGON_SAS_CLASS,
//         WINLOGON_SAS_TITLE,
//         WS_POPUP,
//         0, 0, 0, 0, 0, 0,
//         hAppInstance, Session);
//     if (!Session->SASWindow)
//     {
//         ERR!("WL: Failed to create SAS window\n");
//         goto cleanup;
//     }
//
//     /* Register SAS window to receive SAS notifications */
//     if (!SetLogonNotifyWindow(Session->SASWindow))
//     {
//         ERR!("WL: Failed to register SAS window\n");
//         goto cleanup;
//     }
//
//     if (!SetDefaultLanguage(NULL))
//         return FALSE;
//
//     ret = TRUE;
//
// cleanup:
//     if (!ret)
//         UninitializeSAS(Session);
//     return ret;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
