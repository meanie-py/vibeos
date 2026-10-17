//! SCM database core — Rust flip of base/system/services/database.c (SCM wave 7).
//! Translated by rustconv (Opus 4.8); the FFI surface was retargeted by hand
//! from the kernel ntkrnl-sys substrate (rustconv's default) to the user-mode
//! winsrv-sys substrate used by the rest of services-rust.

use winsrv_sys::winsrv::*;
use winsrv_sys::consts::{
    EnterCriticalSection, InitializeCriticalSection, LeaveCriticalSection, RtlZeroMemory,
};
use winsrv_sys::nt::{
    InitializeListHead, InsertTailList, RemoveEntryList, INVALID_HANDLE_VALUE, STATUS_SUCCESS,
};
use crate::types::*;
use crate::wstr;
use core::ffi::c_void;

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


// ntstatus.h code not carried by winsrv-sys.
const STATUS_NO_MORE_ENTRIES: NTSTATUS = 0x8000001Au32 as NTSTATUS;

// userenv.h _PROFILEINFOW (LoadUserProfileW) — not in the winsrv-sys surface.
#[repr(C)]
pub struct PROFILEINFOW {
    pub dwSize: DWORD,
    pub dwFlags: DWORD,
    pub lpUserName: LPWSTR,
    pub lpProfilePath: LPWSTR,
    pub lpDefaultPath: LPWSTR,
    pub lpServerName: LPWSTR,
    pub lpPolicyPath: LPWSTR,
    pub hProfile: HANDLE,
}
const PI_NOUI: DWORD = 1;
// winsvc.h enum-service-state + accepted-controls absent from winsrv-sys.
const SERVICE_ACTIVE: DWORD = 1;
const SERVICE_INACTIVE: DWORD = 2;
const SERVICE_ACCEPT_SHUTDOWN: DWORD = 4;
// resource.h / services.mc — string + eventlog message IDs.
const IDS_SERVICE_START: DWORD = 100;
const EVENT_SERVICE_START_FAILED: DWORD = 0xC000_1B58;
const EVENT_SERVICE_CONTROL_SUCCESS: DWORD = 0x4000_1B7B;

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// FIELD_OFFSET(SERVICE, szServiceName) + cch*sizeof(WCHAR) — the variable-tail
// SERVICE record alloc size.
#[inline]
unsafe fn field_offset_szservicename(cch: usize) -> SIZE_T {
    (core::mem::offset_of!(SERVICE, szServiceName) + cch * core::mem::size_of::<u16>()) as SIZE_T
}

// Cross-module SCM exports (config.c/security.c, now Rust) + still-C globals.
extern "C" {
    fn ScmReadString(hServiceKey: HKEY, lpValueName: LPCWSTR, lpValue: *mut LPWSTR) -> DWORD;
    fn ScmIsDeleteFlagSet(hServiceKey: HKEY) -> BOOL;
    fn ScmWriteSecurityDescriptor(hServiceKey: HKEY, pSD: PSECURITY_DESCRIPTOR) -> DWORD;
    fn ScmReadSecurityDescriptor(hServiceKey: HKEY, ppSD: *mut PSECURITY_DESCRIPTOR) -> DWORD;
    fn ScmCreateDefaultServiceSD(ppSD: *mut PSECURITY_DESCRIPTOR) -> DWORD;
}
// Exported globals defined elsewhere (groupdb.rs / security.rs / services.c).
extern "C" {
    static mut GroupListHead: LIST_ENTRY;
    static mut pPipeSD: PSECURITY_DESCRIPTOR;
    static mut ScmInitialize: BOOL;
    static mut ScmLiveSetup: BOOL;
    static mut ScmSetupInProgress: BOOL;
}

// CRT wide-string + RPC alloc — cdecl, not in the winsrv-sys surface.
extern "C" {
    fn wcsstr(s: *const u16, find: *const u16) -> *mut u16;
    fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcscat(dst: PWSTR, src: *const u16) -> PWSTR;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcscpy(dst: PWSTR, src: *const u16) -> PWSTR;
    fn wcslen(s: *const u16) -> usize;
}

// Cross-file SCM helpers (services.h internal = cdecl): defined in sibling Rust
// modules (lock/config/groupdb/driver) or still-C files (services.c).
extern "C" {
    fn ScmSetServiceGroup(lpService: PSERVICE, lpGroupName: LPCWSTR) -> DWORD;
    fn ScmCreateGroupList() -> DWORD;
    fn ScmDeleteRegKey(hKey: HKEY, lpszSubKey: LPCWSTR) -> DWORD;
    fn ScmStartDriver(Service: PSERVICE) -> DWORD;
    fn ScmAcquireServiceStartLock(bExclusive: BOOL, Lock: *mut SC_RPC_LOCK) -> DWORD;
    fn ScmReleaseServiceStartLock(lpLock: LPSC_RPC_LOCK) -> DWORD;
    fn ScmLogEvent(dwEventId: DWORD, wType: WORD, wStrings: WORD, lpStrings: *mut LPCWSTR,);
    fn ScmWaitForLsa();
    fn SetSecurityServicesEvent() -> DWORD;
    fn Int_EnumDependentServicesW(
        hServicesKey: HKEY,
        lpService: PSERVICE,
        dwServiceState: DWORD,
        lpServices: *mut PSERVICE,
        pcbBytesNeeded: LPDWORD,
        lpServicesReturned: LPDWORD,
    ) -> DWORD;
}

// Win32 imports absent from the winsrv-sys bindgen (WINAPI = stdcall on i386).
extern "system" {
    fn ConnectNamedPipe(hNamedPipe: HANDLE, lpOverlapped: LPOVERLAPPED) -> BOOL;
    fn TransactNamedPipe(
        hNamedPipe: HANDLE,
        lpInBuffer: PVOID,
        nInBufferSize: DWORD,
        lpOutBuffer: PVOID,
        nOutBufferSize: DWORD,
        lpBytesRead: *mut DWORD,
        lpOverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    fn ImpersonateLoggedOnUser(hToken: HANDLE) -> BOOL;
    fn RevertToSelf() -> BOOL;
    fn LoadUserProfileW(hToken: HANDLE, lpProfileInfo: *mut PROFILEINFOW) -> BOOL;
    fn UnloadUserProfile(hToken: HANDLE, hProfile: HANDLE) -> BOOL;
    fn LogonUserW(
        lpszUsername: PWSTR,
        lpszDomain: PWSTR,
        lpszPassword: PWSTR,
        dwLogonType: DWORD,
        dwLogonProvider: DWORD,
        phToken: *mut HANDLE,
    ) -> BOOL;
    fn CreateEnvironmentBlock(lpEnvironment: *mut LPVOID, hToken: HANDLE, bInherit: BOOL) -> BOOL;
    fn DestroyEnvironmentBlock(lpEnvironment: LPVOID) -> BOOL;
    // RPC MIDL allocator — __RPC_USER = stdcall on i386 (rpcserver.c defines them).
    fn MIDL_user_allocate(len: SIZE_T) -> *mut c_void;
    fn MIDL_user_free(ptr: *mut c_void);
}

// ---- file-scope globals (database.c) ----
// ImageListHead/ServiceListHead are read by rpcserver.c (still C) via the
// `extern LIST_ENTRY` decls in services.h → must export the C symbol.
#[no_mangle]
pub static mut ImageListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ServiceListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut DatabaseLock: RTL_RESOURCE = unsafe { core::mem::zeroed() };
static mut ResumeCount: DWORD = 1;
static mut NoInteractiveServices: DWORD = 0;
static mut ServiceTag: DWORD = 0;
static mut ControlServiceCriticalSection: CRITICAL_SECTION = unsafe { core::mem::zeroed() };
static mut PipeTimeout: DWORD = 30000;

// Interlocked intrinsics (return the NEW value) over a *mut LONG field.
#[inline]
unsafe fn InterlockedIncrement(p: *mut LONG) -> LONG {
    (*(p as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
unsafe fn InterlockedDecrement(p: *mut LONG) -> LONG {
    (*(p as *const core::sync::atomic::AtomicI32)).fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

// Bounded strsafe replacements (the C used StringCch{Copy,Cat}W; both cch-bounded
// and always NUL-terminate). StringCchPrintfW call sites are rewritten inline
// (each is a single %lu) via scm_append_u32.
unsafe fn StringCchCopyW(dst: *mut u16, cch: usize, src: *const u16) {
    if cch == 0 {
        return;
    }
    let mut i = 0usize;
    while i + 1 < cch && *src.add(i) != 0 {
        *dst.add(i) = *src.add(i);
        i += 1;
    }
    *dst.add(i) = 0;
}
unsafe fn StringCchCatW(dst: *mut u16, cch: usize, src: *const u16) {
    if cch == 0 {
        return;
    }
    let mut d = 0usize;
    while d < cch && *dst.add(d) != 0 {
        d += 1;
    }
    let mut s = 0usize;
    while d + 1 < cch && *src.add(s) != 0 {
        *dst.add(d) = *src.add(s);
        d += 1;
        s += 1;
    }
    if d < cch {
        *dst.add(d) = 0;
    }
}
// Append `value` as decimal then NUL into dst[at..], bounded by cch.
unsafe fn scm_append_u32(dst: *mut u16, cch: usize, mut at: usize, value: u32) {
    let mut tmp = [0u16; 10];
    let mut n = 0usize;
    let mut v = value;
    if v == 0 {
        tmp[0] = b'0' as u16;
        n = 1;
    }
    while v > 0 {
        tmp[n] = b'0' as u16 + (v % 10) as u16;
        v /= 10;
        n += 1;
    }
    while n > 0 && at + 1 < cch {
        n -= 1;
        *dst.add(at) = tmp[n];
        at += 1;
    }
    if at < cch {
        *dst.add(at) = 0;
    }
}

unsafe fn ScmIsSecurityService(pServiceImage: PSERVICE_IMAGE) -> BOOL {
    let needle: [u16; 20] = [
        0x005C, b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
        b'm' as u16, b'3' as u16, b'2' as u16, 0x005C, b'l' as u16, b's' as u16,
        b'a' as u16, b's' as u16, b's' as u16, b'.' as u16, b'e' as u16, b'x' as u16,
        b'e' as u16, 0x0000,
    ];
    (wcsstr((*pServiceImage).pszImagePath, needle.as_ptr()) != core::ptr::null_mut()) as BOOL
}

unsafe fn ScmCreateNewControlPipe(
    pServiceImage: PSERVICE_IMAGE,
    bSecurityServiceProcess: BOOL,
) -> DWORD {
    let mut szControlPipeName: [WCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];
    let mut SecurityAttributes: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut hServiceCurrentKey: HKEY = INVALID_HANDLE_VALUE as HKEY;
    let mut dwServiceCurrent: DWORD = 1;
    let mut dwKeyDisposition: DWORD = 0;
    let mut dwKeySize: DWORD;
    let mut dwError: DWORD;

    if bSecurityServiceProcess == FALSE as BOOL {
        dwError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            wstr!("SYSTEM\\CurrentControlSet\\Control\\ServiceCurrent\0").as_ptr(),
            0,
            core::ptr::null_mut(),
            REG_OPTION_VOLATILE,
            KEY_WRITE | KEY_READ,
            core::ptr::null_mut(),
            &mut hServiceCurrentKey,
            &mut dwKeyDisposition,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            return dwError;
        }

        // Advance the counter so each service image gets a UNIQUE control pipe
        // (\\.\pipe\net\NtControlPipe<N>). We must NOT gate this on
        // dwKeyDisposition == REG_OPENED_EXISTING_KEY: RegCreateKeyExW reports
        // REG_CREATED_NEW_KEY *every* time for this volatile key (the key value
        // still persists, but the disposition is wrong), so the old gated read
        // never ran → the counter was stuck at 1 → every service collided on
        // NtControlPipe1 → control-pipe connect cross-talk → intermittent 1053
        // service-connect timeouts → EventLog cascade-wedged the boot (~50% x86).
        // Read the existing value unconditionally; if present, advance past it.
        let _ = dwKeyDisposition;
        dwKeySize = core::mem::size_of::<DWORD>() as DWORD;
        let mut dwExisting: DWORD = 0;
        if RegQueryValueExW(
            hServiceCurrentKey,
            wstr!("\0").as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwExisting as *mut DWORD as *mut BYTE,
            &mut dwKeySize,
        ) as DWORD == ERROR_SUCCESS
        {
            dwServiceCurrent = dwExisting + 1;
        }

        dwError = RegSetValueExW(
            hServiceCurrentKey,
            wstr!("\0").as_ptr(),
            0,
            REG_DWORD,
            &mut dwServiceCurrent as *mut DWORD as *mut BYTE,
            core::mem::size_of_val(&dwServiceCurrent) as DWORD,
        ) as DWORD;

        RegCloseKey(hServiceCurrentKey);

        if dwError != ERROR_SUCCESS {
            return dwError;
        }
    } else {
        dwServiceCurrent = 0;
    }

    // StringCchPrintfW(szControlPipeName, len, L"\\.\pipe\net\NtControlPipe%lu", dwServiceCurrent)
    StringCchCopyW(
        szControlPipeName.as_mut_ptr(),
        szControlPipeName.len(),
        wstr!("\\\\.\\pipe\\net\\NtControlPipe").as_ptr(),
    );
    scm_append_u32(
        szControlPipeName.as_mut_ptr(),
        szControlPipeName.len(),
        wcslen(szControlPipeName.as_ptr()),
        dwServiceCurrent,
    );

    SecurityAttributes.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    SecurityAttributes.lpSecurityDescriptor = pPipeSD;
    SecurityAttributes.bInheritHandle = FALSE as BOOL;

    (*pServiceImage).hControlPipe = CreateNamedPipeW(
        szControlPipeName.as_ptr(),
        PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
        PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
        100,
        8000,
        4,
        PipeTimeout,
        &mut SecurityAttributes,
    );

    if (*pServiceImage).hControlPipe == INVALID_HANDLE_VALUE {
        return GetLastError();
    }

    // Temp control-pipe-race instrumentation (DcomLaunch wedge investigation):
    // SCM created NtControlPipe<dwServiceCurrent>. Compare against the service's
    // [SCTRL] connecting-pipe number; a mismatch = the shared-ServiceCurrent race.
    DbgPrint(
        b"[SERVICES] ScmCreateNewControlPipe: created NtControlPipe%lu (disposition=%lu 1=NEW 2=EXISTING)\n\0".as_ptr() as *const i8,
        dwServiceCurrent,
        dwKeyDisposition,
    );

    ERROR_SUCCESS
}

unsafe fn ScmGetServiceImageByImagePath(lpImagePath: LPWSTR) -> PSERVICE_IMAGE {
    let mut ImageEntry: PLIST_ENTRY;
    let _CurrentImage: PSERVICE_IMAGE;

    ImageEntry = ImageListHead.Flink;
    while ImageEntry != core::ptr::addr_of_mut!(ImageListHead) {
        let CurrentImage = (ImageEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE_IMAGE, ImageListEntry) as isize))
            as PSERVICE_IMAGE;
        if _wcsicmp((*CurrentImage).pszImagePath, lpImagePath) == 0 {
            return CurrentImage;
        }

        ImageEntry = (*ImageEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn ScmGetServiceNameFromTag(
    InParams: PTAG_INFO_NAME_FROM_TAG_IN_PARAMS,
    OutParams: *mut PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS,
) -> DWORD {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE = core::ptr::null_mut();
    let mut CurrentImage: PSERVICE_IMAGE;
    let mut OutBuffer: PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS = core::ptr::null_mut();
    let dwError: DWORD;

    /* Lock the database */
    ScmLockDatabaseExclusive();

    /* Find the matching service */
    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        /* We must match the tag */
        if (*CurrentService).dwServiceTag == (*InParams).dwTag
            && !(*CurrentService).lpImage.is_null()
        {
            CurrentImage = (*CurrentService).lpImage;
            /* And matching the PID */
            if (*CurrentImage).dwProcessId == (*InParams).dwPid {
                break;
            }
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    'cleanup: loop {
        /* No match! */
        if ServiceEntry == core::ptr::addr_of_mut!(ServiceListHead) {
            dwError = ERROR_RETRY;
            break 'cleanup;
        }

        /* Allocate the output buffer */
        OutBuffer = MIDL_user_allocate(
            core::mem::size_of::<TAG_INFO_NAME_FROM_TAG_OUT_PARAMS>() as SIZE_T,
        ) as PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS;
        if OutBuffer.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        /* And the buffer for the name */
        (*OutBuffer).pszName = MIDL_user_allocate(
            (wcslen((*CurrentService).lpServiceName) * core::mem::size_of::<WCHAR>()
                + core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as *mut WCHAR;
        if (*OutBuffer).pszName.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'cleanup;
        }

        /* Fill in output data */
        wcscpy((*OutBuffer).pszName, (*CurrentService).lpServiceName);
        (*OutBuffer).TagType = TagTypeService;

        /* And return */
        *OutParams = OutBuffer;
        dwError = ERROR_SUCCESS;
        break 'cleanup;
    }

    /* Unlock database */
    ScmUnlockDatabase();

    /* If failure, free allocated memory */
    if dwError != ERROR_SUCCESS {
        if !OutBuffer.is_null() {
            MIDL_user_free(OutBuffer as *mut core::ffi::c_void);
        }
    }

    /* Return error/success */
    dwError
}

unsafe fn ScmIsSameServiceAccount(
    pszAccountName1: PCWSTR,
    pszAccountName2: PCWSTR,
) -> BOOL {

    let local_system: [u16; 12] = [
        b'L' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b'l' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
        b'm' as u16, 0,
    ];

    if pszAccountName1.is_null() && pszAccountName2.is_null() {
        return 1;
    }

    if pszAccountName1.is_null()
        && !pszAccountName2.is_null()
        && _wcsicmp(pszAccountName2, local_system.as_ptr()) == 0
    {
        return 1;
    }

    if !pszAccountName1.is_null()
        && pszAccountName2.is_null()
        && _wcsicmp(pszAccountName1, local_system.as_ptr()) == 0
    {
        return 1;
    }

    if !pszAccountName1.is_null()
        && !pszAccountName2.is_null()
        && _wcsicmp(pszAccountName1, pszAccountName2) == 0
    {
        return 1;
    }

    0
}

unsafe fn ScmIsLocalSystemAccount(pszAccountName: PCWSTR) -> BOOL {

    if pszAccountName.is_null()
        || _wcsicmp(
            pszAccountName,
            w!("LocalSystem").as_ptr() as *const u16,
        ) == 0
    {
        return 1;
    }

    0
}

unsafe fn ScmEnableBackupRestorePrivileges(
    hToken: HANDLE,
    bEnable: BOOL,
) -> BOOL {

    const HEAP_ZERO_MEMORY: DWORD = 0x00000008;
    const SE_BACKUP_PRIVILEGE: u32 = 17;
    const SE_RESTORE_PRIVILEGE: u32 = 18;
    const ERROR_NOT_ALL_ASSIGNED: DWORD = 1300;

    let mut pTokenPrivileges: PTOKEN_PRIVILEGES = core::ptr::null_mut();
    let dwSize: DWORD;
    let mut bRet: BOOL = 0;

    'done: {
        dwSize = (core::mem::size_of::<TOKEN_PRIVILEGES>()
            + 2 * core::mem::size_of::<LUID_AND_ATTRIBUTES>()) as DWORD;
        pTokenPrivileges =
            HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize as SIZE_T) as PTOKEN_PRIVILEGES;
        if pTokenPrivileges.is_null() {
            break 'done;
        }

        (*pTokenPrivileges).PrivilegeCount = 2;
        let privileges = (*pTokenPrivileges).Privileges.as_mut_ptr();
        (*privileges.offset(0)).Luid.LowPart = SE_BACKUP_PRIVILEGE;
        (*privileges.offset(0)).Luid.HighPart = 0;
        (*privileges.offset(0)).Attributes = if bEnable != 0 { SE_PRIVILEGE_ENABLED } else { 0 };
        (*privileges.offset(1)).Luid.LowPart = SE_RESTORE_PRIVILEGE;
        (*privileges.offset(1)).Luid.HighPart = 0;
        (*privileges.offset(1)).Attributes = if bEnable != 0 { SE_PRIVILEGE_ENABLED } else { 0 };

        bRet = AdjustTokenPrivileges(
            hToken,
            0,
            pTokenPrivileges,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if bRet == 0 {
        } else if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
            bRet = 0;
        }
    }

    if !pTokenPrivileges.is_null() {
        HeapFree(GetProcessHeap(), 0, pTokenPrivileges as *mut core::ffi::c_void);
    }

    bRet
}

unsafe fn ScmLogonService(
    pService: PSERVICE,
    pImage: PSERVICE_IMAGE,
) -> DWORD {

    let mut ProfileInfo: PROFILEINFOW = core::mem::zeroed();
    let mut pszUserName: PWSTR = core::ptr::null_mut();
    let mut pszDomainName: PWSTR = core::ptr::null_mut();
    let mut pszPassword: PWSTR = core::ptr::null_mut();
    let ptr: *mut u16;
    let mut dwError: DWORD = ERROR_SUCCESS;

    if ScmIsLocalSystemAccount((*pImage).pszAccountName) != 0
        || ScmLiveSetup != 0
        || ScmSetupInProgress != 0
    {
        return ERROR_SUCCESS;
    }

    /* Get the user and domain names */
    ptr = wcschr((*pImage).pszAccountName, b'\\' as u16);
    if !ptr.is_null() {
        *ptr = 0;
        pszUserName = ptr.add(1);
        pszDomainName = (*pImage).pszAccountName;
    } else {
        pszUserName = (*pImage).pszAccountName;
        pszDomainName = core::ptr::null_mut();
    }

    'done: {
        /* Build the service 'password' */
        pszPassword = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((wcslen((*pService).lpServiceName) + 5) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        if pszPassword.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        wcscpy(pszPassword, w!("_SC_").as_ptr() as *const u16);
        wcscat(pszPassword, (*pService).lpServiceName);

        /* Do the service logon */
        if LogonUserW(
            pszUserName,
            pszDomainName,
            pszPassword,
            LOGON32_LOGON_SERVICE,
            LOGON32_PROVIDER_DEFAULT,
            core::ptr::addr_of_mut!((*pImage).hToken),
        ) == 0
        {
            dwError = GetLastError();

            /* Normalize the returned error */
            dwError = ERROR_SERVICE_LOGON_FAILED;
            break 'done;
        }

        /* Load the user profile */
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(ProfileInfo) as *mut u8,
            0,
            core::mem::size_of::<PROFILEINFOW>(),
        );
        ProfileInfo.dwSize = core::mem::size_of::<PROFILEINFOW>() as DWORD;
        ProfileInfo.dwFlags = PI_NOUI;
        ProfileInfo.lpUserName = pszUserName;

        ScmEnableBackupRestorePrivileges((*pImage).hToken, TRUE as BOOL);
        if LoadUserProfileW((*pImage).hToken, core::ptr::addr_of_mut!(ProfileInfo)) == 0 {
            dwError = GetLastError();
        }
        ScmEnableBackupRestorePrivileges((*pImage).hToken, FALSE as BOOL);

        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        (*pImage).hProfile = ProfileInfo.hProfile;
    }

    if !pszPassword.is_null() {
        HeapFree(GetProcessHeap(), 0, pszPassword as *mut core::ffi::c_void);
    }

    if !ptr.is_null() {
        *ptr = b'\\' as u16;
    }

    dwError
}

unsafe fn ScmCreateOrReferenceServiceImage(pService: PSERVICE) -> DWORD {

    let mut QueryTable: [RTL_QUERY_REGISTRY_TABLE; 3] = core::mem::zeroed();
    let mut ImagePath: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut pServiceImage: PSERVICE_IMAGE = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let dwRecordSize: DWORD;
    let mut pString: LPWSTR;
    let bSecurityService: BOOL;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ImagePath), core::ptr::null_mut());
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ObjectName), core::ptr::null_mut());

    RtlZeroMemory(
        core::ptr::addr_of_mut!(QueryTable) as *mut core::ffi::c_void,
        core::mem::size_of::<[RTL_QUERY_REGISTRY_TABLE; 3]>(),
    );

    let image_path_name: [u16; 10] = [
        b'I' as u16, b'm' as u16, b'a' as u16, b'g' as u16, b'e' as u16,
        b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0,
    ];
    let object_name_name: [u16; 11] = [
        b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16,
        b't' as u16, b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, 0,
    ];

    QueryTable[0].Name = image_path_name.as_ptr() as PWSTR;
    QueryTable[0].Flags = (RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED) as ULONG;
    QueryTable[0].EntryContext = core::ptr::addr_of_mut!(ImagePath) as *mut core::ffi::c_void;
    QueryTable[1].Name = object_name_name.as_ptr() as PWSTR;
    QueryTable[1].Flags = RTL_QUERY_REGISTRY_DIRECT as ULONG;
    QueryTable[1].EntryContext = core::ptr::addr_of_mut!(ObjectName) as *mut core::ffi::c_void;

    Status = RtlQueryRegistryValues(
        RTL_REGISTRY_SERVICES,
        (*pService).lpServiceName,
        QueryTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return RtlNtStatusToDosError(Status);
    }

    pServiceImage = ScmGetServiceImageByImagePath(ImagePath.Buffer);
    'done: {
        if pServiceImage.is_null() {
            dwRecordSize = core::mem::size_of::<SERVICE_IMAGE>() as DWORD
                + ImagePath.Length as DWORD
                + core::mem::size_of::<WCHAR>() as DWORD
                + (if ObjectName.Length != 0 {
                    ObjectName.Length as DWORD + core::mem::size_of::<WCHAR>() as DWORD
                } else {
                    0
                });

            pServiceImage = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwRecordSize as SIZE_T)
                as PSERVICE_IMAGE;
            if pServiceImage.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'done;
            }

            (*pServiceImage).dwImageRunCount = 1;
            (*pServiceImage).hControlPipe = INVALID_HANDLE_VALUE as HANDLE;
            (*pServiceImage).hProcess = INVALID_HANDLE_VALUE as HANDLE;

            pString = ((pServiceImage as INT_PTR)
                + core::mem::size_of::<SERVICE_IMAGE>() as INT_PTR)
                as PWSTR;

            (*pServiceImage).pszImagePath = pString;
            wcscpy((*pServiceImage).pszImagePath, ImagePath.Buffer);

            if ObjectName.Length > 0 {
                pString = pString.add(wcslen(pString) + 1);

                (*pServiceImage).pszAccountName = pString;
                wcscpy((*pServiceImage).pszAccountName, ObjectName.Buffer);
            }

            dwError = ScmLogonService(pService, pServiceImage);
            if dwError != ERROR_SUCCESS {
                HeapFree(GetProcessHeap(), 0, pServiceImage as *mut core::ffi::c_void);
                break 'done;
            }

            bSecurityService = ScmIsSecurityService(pServiceImage);

            dwError = ScmCreateNewControlPipe(pServiceImage, bSecurityService);
            if dwError != ERROR_SUCCESS {
                if !(*pServiceImage).hProfile.is_null() {
                    ScmEnableBackupRestorePrivileges((*pServiceImage).hToken, TRUE as BOOL);
                    UnloadUserProfile((*pServiceImage).hToken, (*pServiceImage).hProfile);
                    ScmEnableBackupRestorePrivileges((*pServiceImage).hToken, FALSE as BOOL);
                }

                if !(*pServiceImage).hToken.is_null() {
                    CloseHandle((*pServiceImage).hToken);
                }

                HeapFree(GetProcessHeap(), 0, pServiceImage as *mut core::ffi::c_void);
                break 'done;
            }

            if bSecurityService != 0 {
                SetSecurityServicesEvent();
            }

            InsertTailList(
                core::ptr::addr_of_mut!(ImageListHead),
                core::ptr::addr_of_mut!((*pServiceImage).ImageListEntry),
            );
        } else {
            if ScmIsSameServiceAccount((*pServiceImage).pszAccountName, ObjectName.Buffer) == 0 {
                dwError = ERROR_DIFFERENT_SERVICE_ACCOUNT;
                break 'done;
            }

            (*pServiceImage).dwImageRunCount += 1;
        }

        (*pService).lpImage = pServiceImage;
    }

    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ObjectName));
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(ImagePath));

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmRemoveServiceImage(pServiceImage: PSERVICE_IMAGE) {

    /* FIXME: Terminate the process */

    /* Remove the service image from the list */
    RemoveEntryList(core::ptr::addr_of_mut!((*pServiceImage).ImageListEntry));

    /* Close the process handle */
    if (*pServiceImage).hProcess != INVALID_HANDLE_VALUE as HANDLE {
        CloseHandle((*pServiceImage).hProcess);
    }

    /* Close the control pipe */
    if (*pServiceImage).hControlPipe != INVALID_HANDLE_VALUE as HANDLE {
        CloseHandle((*pServiceImage).hControlPipe);
    }

    /* Unload the user profile */
    if !(*pServiceImage).hProfile.is_null() {
        ScmEnableBackupRestorePrivileges((*pServiceImage).hToken, 1);
        UnloadUserProfile((*pServiceImage).hToken, (*pServiceImage).hProfile);
        ScmEnableBackupRestorePrivileges((*pServiceImage).hToken, 0);
    }

    /* Close the logon token */
    if !(*pServiceImage).hToken.is_null() {
        CloseHandle((*pServiceImage).hToken);
    }

    /* Release the service image */
    HeapFree(GetProcessHeap(), 0, pServiceImage as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ScmGetServiceEntryByName(lpServiceName: LPCWSTR) -> PSERVICE {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as usize
            - core::mem::offset_of!(SERVICE, ServiceListEntry)) as PSERVICE;
        if _wcsicmp((*CurrentService).lpServiceName, lpServiceName) == 0 {
            return CurrentService;
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn ScmGetServiceEntryByDisplayName(lpDisplayName: LPCWSTR) -> PSERVICE {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_SERVICE, ServiceListEntry) as isize))
            as *mut _SERVICE;

        if _wcsicmp((*CurrentService).lpDisplayName, lpDisplayName) == 0 {
            return CurrentService;
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn ScmGetServiceEntryByResumeCount(dwResumeCount: DWORD) -> PSERVICE {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;
        if (*CurrentService).dwResumeCount > dwResumeCount {
            return CurrentService;
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn ScmGenerateServiceTag(lpServiceRecord: PSERVICE) -> DWORD {
    /* Check for an overflow */
    if ServiceTag == 0xFFFFFFFF {
        return ERROR_INVALID_DATA;
    }

    /* This is only valid for Win32 services */
    if (*lpServiceRecord).Status.dwServiceType & SERVICE_WIN32 == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    /* Increment the tag counter and set it */
    ServiceTag = ServiceTag % 0xFFFFFFFF + 1;
    (*lpServiceRecord).dwServiceTag = ServiceTag;

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ScmCreateNewServiceRecord(
    lpServiceName: LPCWSTR,
    lpServiceRecord: *mut PSERVICE,
    dwServiceType: DWORD,
    dwStartType: DWORD,
) -> DWORD {
    let mut lpService: PSERVICE = core::ptr::null_mut();

    let offset = field_offset_szservicename(wcslen(lpServiceName) + 1);

    lpService = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, offset) as PSERVICE;
    if lpService.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    *lpServiceRecord = lpService;

    wcscpy(core::ptr::addr_of_mut!((*lpService).szServiceName) as *mut u16, lpServiceName);
    (*lpService).lpServiceName = core::ptr::addr_of_mut!((*lpService).szServiceName) as *mut u16;
    (*lpService).lpDisplayName = (*lpService).lpServiceName;

    (*lpService).dwStartType = dwStartType;

    (*lpService).dwResumeCount = ResumeCount;
    ResumeCount += 1;

    InsertTailList(
        core::ptr::addr_of_mut!(ServiceListHead),
        core::ptr::addr_of_mut!((*lpService).ServiceListEntry),
    );

    (*lpService).Status.dwServiceType = dwServiceType;
    (*lpService).Status.dwCurrentState = SERVICE_STOPPED;
    (*lpService).Status.dwControlsAccepted = 0;
    (*lpService).Status.dwWin32ExitCode = if dwStartType == SERVICE_DISABLED {
        ERROR_SERVICE_DISABLED
    } else {
        ERROR_SERVICE_NEVER_STARTED
    };
    (*lpService).Status.dwServiceSpecificExitCode = 0;
    (*lpService).Status.dwCheckPoint = 0;
    (*lpService).Status.dwWaitHint = if (dwServiceType & SERVICE_DRIVER) != 0 {
        0
    } else {
        2000
    };

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ScmDeleteServiceRecord(lpService: PSERVICE) {

    /* Delete the display name */
    if (*lpService).lpDisplayName != core::ptr::null_mut()
        && (*lpService).lpDisplayName != (*lpService).lpServiceName
    {
        HeapFree(GetProcessHeap(), 0, (*lpService).lpDisplayName as LPVOID);
    }

    /* Dereference the service image */
    if !(*lpService).lpImage.is_null() {
        (*(*lpService).lpImage).dwImageRunCount -= 1;

        if (*(*lpService).lpImage).dwImageRunCount == 0 {
            ScmRemoveServiceImage((*lpService).lpImage);
            (*lpService).lpImage = core::ptr::null_mut();
        }
    }

    /* Decrement the group reference counter */
    ScmSetServiceGroup(lpService, core::ptr::null_mut());

    /* Release the SecurityDescriptor */
    if (*lpService).pSecurityDescriptor != core::ptr::null_mut() {
        HeapFree(GetProcessHeap(), 0, (*lpService).pSecurityDescriptor as LPVOID);
    }

    /* Remove the Service from the List */
    RemoveEntryList(core::ptr::addr_of_mut!((*lpService).ServiceListEntry));

    /* Delete the service record */
    HeapFree(GetProcessHeap(), 0, lpService as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn ScmDeleteService(lpService: PSERVICE) -> DWORD {

    let mut dwError: DWORD;
    let mut pcbBytesNeeded: DWORD = 0;
    let mut dwServicesReturned: DWORD = 0;
    let mut hServicesKey: HKEY = core::ptr::null_mut();

    /* Open the Services Reg key */
    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("System\\CurrentControlSet\\Services\0").as_ptr(),
        0,
        (KEY_SET_VALUE | KEY_READ) as REGSAM,
        &mut hServicesKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Call the function with NULL, just to get bytes we need */
    Int_EnumDependentServicesW(
        hServicesKey,
        lpService,
        SERVICE_ACTIVE,
        core::ptr::null_mut(),
        &mut pcbBytesNeeded,
        &mut dwServicesReturned,
    );

    /* If pcbBytesNeeded returned a value then there are services running that are dependent on this service */
    if pcbBytesNeeded != 0 {
        RegCloseKey(hServicesKey);
        return ERROR_DEPENDENT_SERVICES_RUNNING;
    }

    /* Delete the Service Key */
    dwError = ScmDeleteRegKey(hServicesKey, (*lpService).lpServiceName);

    RegCloseKey(hServicesKey);

    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    /* Delete the Service */
    ScmDeleteServiceRecord(lpService);

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ScmReferenceService(lpService: PSERVICE) -> DWORD {
    InterlockedIncrement(core::ptr::addr_of_mut!((*lpService).RefCount)) as DWORD
}
#[no_mangle]
pub unsafe extern "C" fn ScmDereferenceService(lpService: PSERVICE) -> DWORD {
    let ref_count = InterlockedDecrement(core::ptr::addr_of_mut!((*lpService).RefCount));

    if ref_count == 0
        && (*lpService).bDeleted != 0
        && (*lpService).Status.dwCurrentState == SERVICE_STOPPED
    {
        ScmDeleteService(lpService);
    }

    ref_count as DWORD
}

unsafe fn CreateServiceListEntry(
    lpServiceName: LPCWSTR,
    hServiceKey: HKEY,
) -> DWORD {
    let mut lpService: PSERVICE = core::ptr::null_mut();
    let mut lpDisplayName: LPWSTR = core::ptr::null_mut();
    let mut lpGroup: LPWSTR = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut dwError: DWORD;
    let dwServiceType: DWORD;
    let dwStartType: DWORD;
    let dwErrorControl: DWORD;
    let dwTagId: DWORD;

    let mut dwServiceType_v: DWORD = 0;
    let mut dwStartType_v: DWORD = 0;
    let mut dwErrorControl_v: DWORD = 0;
    let mut dwTagId_v: DWORD = 0;

    if *lpServiceName == b'{' as u16 {
        return ERROR_SUCCESS;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hServiceKey,
        wstr!("Type").as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwServiceType_v) as LPBYTE,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return ERROR_SUCCESS;
    }
    dwServiceType = dwServiceType_v;

    if ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS)
        && ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS)
        && (dwServiceType != SERVICE_KERNEL_DRIVER)
        && (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER)
    {
        return ERROR_SUCCESS;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hServiceKey,
        wstr!("Start").as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwStartType_v) as LPBYTE,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return ERROR_SUCCESS;
    }
    dwStartType = dwStartType_v;

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = RegQueryValueExW(
        hServiceKey,
        wstr!("ErrorControl").as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwErrorControl_v) as LPBYTE,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return ERROR_SUCCESS;
    }
    dwErrorControl = dwErrorControl_v;

    dwError = RegQueryValueExW(
        hServiceKey,
        wstr!("Tag").as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(dwTagId_v) as LPBYTE,
        &mut dwSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        dwTagId_v = 0;
    }
    dwTagId = dwTagId_v;

    dwError = ScmReadString(hServiceKey, wstr!("Group").as_ptr(), &mut lpGroup);
    if dwError != ERROR_SUCCESS {
        lpGroup = core::ptr::null_mut();
    }

    dwError = ScmReadString(hServiceKey, wstr!("DisplayName").as_ptr(), &mut lpDisplayName);
    if dwError != ERROR_SUCCESS {
        lpDisplayName = core::ptr::null_mut();
    }

    'done: {
        dwError = ScmCreateNewServiceRecord(
            lpServiceName,
            &mut lpService,
            dwServiceType,
            dwStartType,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        (*lpService).dwErrorControl = dwErrorControl;
        (*lpService).dwTag = dwTagId;

        if !lpGroup.is_null() {
            dwError = ScmSetServiceGroup(lpService, lpGroup);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpDisplayName.is_null() {
            (*lpService).lpDisplayName = lpDisplayName;
            lpDisplayName = core::ptr::null_mut();
        }

        if ScmIsDeleteFlagSet(hServiceKey) != 0 {
            (*lpService).bDeleted = TRUE as BOOL;
        } else {
            ScmGenerateServiceTag(lpService);
        }

        if (*lpService).Status.dwServiceType & SERVICE_WIN32 != 0 {
            dwError = ScmReadSecurityDescriptor(
                hServiceKey,
                core::ptr::addr_of_mut!((*lpService).pSecurityDescriptor),
            );
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            if (*lpService).pSecurityDescriptor.is_null() {
                dwError = ScmCreateDefaultServiceSD(core::ptr::addr_of_mut!(
                    (*lpService).pSecurityDescriptor
                ));
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }

                dwError = ScmWriteSecurityDescriptor(
                    hServiceKey,
                    (*lpService).pSecurityDescriptor,
                );
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }
            }
        }
    }

    let _ = dwTagId;

    if !lpGroup.is_null() {
        HeapFree(GetProcessHeap(), 0, lpGroup as *mut core::ffi::c_void);
    }

    if !lpDisplayName.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDisplayName as *mut core::ffi::c_void);
    }

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmDeleteMarkedServices() {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        ServiceEntry = (*ServiceEntry).Flink;

        if (*CurrentService).bDeleted != 0 {
            dwError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                wstr!("System\\CurrentControlSet\\Services\0").as_ptr(),
                0,
                DELETE,
                core::ptr::addr_of_mut!(hServicesKey),
            ) as DWORD;
            if dwError == ERROR_SUCCESS {
                dwError = ScmDeleteRegKey(hServicesKey, (*CurrentService).lpServiceName);
                RegCloseKey(hServicesKey);
                if dwError == ERROR_SUCCESS {
                    RemoveEntryList(core::ptr::addr_of_mut!((*CurrentService).ServiceListEntry));
                    HeapFree(GetProcessHeap(), 0, CurrentService as *mut _);
                }
            }
        }
    }
}

unsafe fn ScmGetNoInteractiveServicesValue() {

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwKeySize: DWORD;
    let mut lError: LONG;

    let subKey: [u16; 47] = [
        0x53, 0x59, 0x53, 0x54, 0x45, 0x4D, 0x5C, 0x43, 0x75, 0x72, 0x72, 0x65, 0x6E, 0x74,
        0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53, 0x65, 0x74, 0x5C, 0x43, 0x6F, 0x6E,
        0x74, 0x72, 0x6F, 0x6C, 0x5C, 0x57, 0x69, 0x6E, 0x64, 0x6F, 0x77, 0x73, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    let valueName: [u16; 22] = [
        0x4E, 0x6F, 0x49, 0x6E, 0x74, 0x65, 0x72, 0x61, 0x63, 0x74, 0x69, 0x76, 0x65, 0x53,
        0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x00,
    ];

    lError = RegOpenKeyExW(
        0x8000_0002u32 as i32 as isize as HKEY,
        subKey.as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    );
    if lError == 0 {
        dwKeySize = core::mem::size_of_val(&NoInteractiveServices) as DWORD;
        lError = RegQueryValueExW(
            hKey,
            valueName.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(NoInteractiveServices) as LPBYTE,
            &mut dwKeySize,
        );
        let _ = lError;
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScmCreateServiceDatabase() -> DWORD {

    let mut szSubKey: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut dwSubKey: DWORD;
    let mut dwSubKeyLength: DWORD;
    let mut ftLastChanged: FILETIME = core::mem::zeroed();
    let mut dwError: DWORD;

    ScmGetNoInteractiveServicesValue();

    dwError = ScmCreateGroupList();
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    InitializeListHead(core::ptr::addr_of_mut!(ImageListHead));
    InitializeListHead(core::ptr::addr_of_mut!(ServiceListHead));

    RtlInitializeResource(core::ptr::addr_of_mut!(DatabaseLock));

    let szServices: [u16; 38] = [
        0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x5C, 0x43, 0x75, 0x72, 0x72, 0x65,
        0x6E, 0x74, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53, 0x65, 0x74,
        0x5C, 0x53, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x73, 0x00, 0, 0, 0, 0,
    ];

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szServices.as_ptr(),
        0,
        KEY_READ,
        &mut hServicesKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwSubKey = 0;
    loop {
        dwSubKeyLength = MAX_PATH;
        dwError = RegEnumKeyExW(
            hServicesKey,
            dwSubKey,
            szSubKey.as_mut_ptr(),
            &mut dwSubKeyLength,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut ftLastChanged,
        ) as DWORD;
        if dwError == ERROR_SUCCESS && szSubKey[0] != 0x7B {
            dwError = RegOpenKeyExW(
                hServicesKey,
                szSubKey.as_ptr(),
                0,
                KEY_READ,
                &mut hServiceKey,
            ) as DWORD;
            if dwError == ERROR_SUCCESS {
                dwError = CreateServiceListEntry(szSubKey.as_ptr(), hServiceKey);

                RegCloseKey(hServiceKey);
            }
        }

        if dwError != ERROR_SUCCESS {
            break;
        }

        dwSubKey += 1;
    }

    RegCloseKey(hServicesKey);

    ScmWaitForLsa();

    ScmDeleteMarkedServices();

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ScmShutdownServiceDatabase() {
    ScmDeleteMarkedServices();
    RtlDeleteResource(core::ptr::addr_of_mut!(DatabaseLock));
}

unsafe fn ScmCheckDriver(Service: PSERVICE) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirName: _UNICODE_STRING = core::mem::zeroed();
    let mut DirHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let DirInfo: POBJECT_DIRECTORY_INFORMATION;
    let BufferLength: ULONG;
    let mut DataLength: ULONG = 0;
    let mut Index: ULONG;

    if (*Service).Status.dwServiceType == SERVICE_KERNEL_DRIVER {
        RtlInitUnicodeString(
            &mut DirName,
            wstr!("\\Driver\0").as_ptr(),
        );
    } else {
        RtlInitUnicodeString(
            &mut DirName,
            wstr!("\\FileSystem\0").as_ptr(),
        );
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.ObjectName = &mut DirName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenDirectoryObject(
        &mut DirHandle,
        DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
        &mut ObjectAttributes,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    BufferLength = (core::mem::size_of::<OBJECT_DIRECTORY_INFORMATION>()
        + 2 * MAX_PATH as usize * core::mem::size_of::<u16>()) as ULONG;
    DirInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, BufferLength as SIZE_T)
        as POBJECT_DIRECTORY_INFORMATION;

    Index = 0;
    loop {
        Status = NtQueryDirectoryObject(
            DirHandle,
            DirInfo as *mut core::ffi::c_void,
            BufferLength,
            1u8,
            0u8,
            &mut Index,
            &mut DataLength,
        );
        if Status == STATUS_NO_MORE_ENTRIES {
            break;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if _wcsicmp((*Service).lpServiceName, (*DirInfo).Name.Buffer) == 0 {
            (*Service).Status.dwCurrentState = SERVICE_RUNNING;
            (*Service).Status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
            (*Service).Status.dwWin32ExitCode = ERROR_SUCCESS;
            (*Service).Status.dwServiceSpecificExitCode = 0;
            (*Service).Status.dwCheckPoint = 0;
            (*Service).Status.dwWaitHint = 0;

            if !(*Service).lpGroup.is_null() {
                (*(*Service).lpGroup).ServicesRunning = 1u8;
            }

            break;
        }
    }

    HeapFree(GetProcessHeap(), 0, DirInfo as *mut core::ffi::c_void);
    NtClose(DirHandle);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn ScmGetBootAndSystemDriverState() {
    let mut ServiceEntry: PLIST_ENTRY;
    let _CurrentService: PSERVICE;

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        let CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(_SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        if (*CurrentService).dwStartType == SERVICE_BOOT_START
            || (*CurrentService).dwStartType == SERVICE_SYSTEM_START
        {
            ScmCheckDriver(CurrentService);
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScmSendControlPacket(
    hControlPipe: HANDLE,
    _pServiceName: PCWSTR,
    _dwControl: DWORD,
    dwControlPacketSize: DWORD,
    pControlPacket: PVOID,
) -> DWORD {

    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut bResult: BOOL;
    let mut ReplyPacket: SCM_REPLY_PACKET = core::mem::zeroed();
    let mut dwReadCount: DWORD = 0;
    let mut Overlapped: OVERLAPPED = core::mem::zeroed();

    EnterCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));

    bResult = TransactNamedPipe(
        hControlPipe,
        pControlPacket,
        dwControlPacketSize,
        core::ptr::addr_of_mut!(ReplyPacket) as PVOID,
        core::mem::size_of::<SCM_REPLY_PACKET>() as DWORD,
        &mut dwReadCount,
        &mut Overlapped,
    );

    'done: {
        if bResult == 0 {
            dwError = GetLastError();
            if dwError != ERROR_IO_PENDING {
                break 'done;
            }

            dwError = WaitForSingleObject(hControlPipe, PipeTimeout);

            if dwError == WAIT_TIMEOUT {
                bResult = CancelIo(hControlPipe);
                let _ = bResult;

                dwError = ERROR_SERVICE_REQUEST_TIMEOUT;
            } else if dwError == WAIT_OBJECT_0 {
                bResult = GetOverlappedResult(
                    hControlPipe,
                    &mut Overlapped,
                    &mut dwReadCount,
                    1,
                );
                if bResult == 0 {
                    dwError = GetLastError();
                }
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));

    if dwReadCount == core::mem::size_of::<SCM_REPLY_PACKET>() as DWORD {
        dwError = ReplyPacket.dwError;
    }

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmControlServiceEx(
    hControlPipe: HANDLE,
    pServiceName: PCWSTR,
    dwControl: DWORD,
    hServiceStatus: SERVICE_STATUS_HANDLE,
    dwServiceTag: DWORD,
    argc: DWORD,
    argv: *const PCWSTR,
) -> DWORD {

    let dwError: DWORD;
    let ControlPacket: PSCM_CONTROL_PACKET;
    let mut PacketSize: DWORD;
    let mut i: DWORD;
    let mut Ptr: PWSTR;

    let align_up_dword = |val: DWORD, align: usize| -> DWORD {
        (((val as usize) + align - 1) & !(align - 1)) as DWORD
    };
    let align_up_ptr = |val: usize, align: usize| -> usize {
        (val + align - 1) & !(align - 1)
    };

    let ptr_size = core::mem::size_of::<PWSTR>();
    let wchar_size = core::mem::size_of::<u16>();

    PacketSize = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD;
    PacketSize += ((wcslen(pServiceName) + 1) * wchar_size) as DWORD;

    if argc > 0 && !argv.is_null() {
        PacketSize = align_up_dword(PacketSize, ptr_size);
        PacketSize += argc * ptr_size as DWORD;

        i = 0;
        while i < argc {
            PacketSize += ((wcslen(*argv.add(i as usize)) + 1) * wchar_size) as DWORD;
            i += 1;
        }
    }

    ControlPacket = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize as SIZE_T) as PSCM_CONTROL_PACKET;
    if ControlPacket.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    (*ControlPacket).dwSize = PacketSize;
    (*ControlPacket).dwControl = dwControl;
    (*ControlPacket).hServiceStatus = hServiceStatus;
    (*ControlPacket).dwServiceTag = dwServiceTag;

    (*ControlPacket).dwServiceNameOffset = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD;
    Ptr = ((ControlPacket as usize) + (*ControlPacket).dwServiceNameOffset as usize) as PWSTR;
    wcscpy(Ptr, pServiceName);

    (*ControlPacket).dwArgumentsCount = 0;
    (*ControlPacket).dwArgumentsOffset = 0;

    if argc > 0 && !argv.is_null() {
        Ptr = Ptr.add(wcslen(pServiceName) + 1);
        let pOffPtr = align_up_ptr(Ptr as usize, ptr_size) as *mut PWSTR;
        let mut pArgPtr = ((pOffPtr as usize) + argc as usize * ptr_size) as PWSTR;

        (*ControlPacket).dwArgumentsCount = argc;
        (*ControlPacket).dwArgumentsOffset = ((pOffPtr as usize) - (ControlPacket as usize)) as DWORD;

        i = 0;
        while i < argc {
            wcscpy(pArgPtr, *argv.add(i as usize));
            *pOffPtr.add(i as usize) = ((pArgPtr as usize) - (pOffPtr as usize)) as PWSTR;
            pArgPtr = pArgPtr.add(wcslen(*argv.add(i as usize)) + 1);
            i += 1;
        }
    }

    dwError = ScmSendControlPacket(
        hControlPipe,
        pServiceName,
        dwControl,
        PacketSize,
        ControlPacket as PVOID,
    );

    HeapFree(GetProcessHeap(), 0, ControlPacket as PVOID);

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmControlService(
    hControlPipe: HANDLE,
    pServiceName: PCWSTR,
    dwControl: DWORD,
    hServiceStatus: SERVICE_STATUS_HANDLE,
) -> DWORD {

    let mut dwError: DWORD = ERROR_SUCCESS;
    let ControlPacket: PSCM_CONTROL_PACKET;
    let mut PacketSize: DWORD;
    let Ptr: PWSTR;

    /* Calculate the total size of the control packet */
    PacketSize = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD;
    PacketSize += ((wcslen(pServiceName) + 1) * core::mem::size_of::<u16>()) as DWORD;

    /* Allocate the control packet */
    ControlPacket = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize as SIZE_T)
        as PSCM_CONTROL_PACKET;
    if ControlPacket.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    (*ControlPacket).dwSize = PacketSize;
    (*ControlPacket).dwControl = dwControl;
    (*ControlPacket).hServiceStatus = hServiceStatus;

    /* Copy the service name */
    (*ControlPacket).dwServiceNameOffset = core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD;
    Ptr = ((ControlPacket as ULONG_PTR) + (*ControlPacket).dwServiceNameOffset as ULONG_PTR)
        as PWSTR;
    wcscpy(Ptr, pServiceName);

    /* Send the control packet */
    dwError = ScmSendControlPacket(
        hControlPipe,
        pServiceName,
        dwControl,
        PacketSize,
        ControlPacket as PVOID,
    );

    /* Free the control packet */
    HeapFree(GetProcessHeap(), 0, ControlPacket as PVOID);

    dwError
}

unsafe fn ScmWaitForServiceConnect(Service: PSERVICE) -> DWORD {

    let mut dwRead: DWORD = 0;
    let mut dwProcessId: DWORD = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut bResult: BOOL;
    let mut Overlapped: OVERLAPPED = core::mem::zeroed();

    bResult = ConnectNamedPipe(
        (*(*Service).lpImage).hControlPipe,
        &mut Overlapped,
    );
    if bResult == FALSE as BOOL {
        dwError = GetLastError();
        if dwError == ERROR_IO_PENDING {
            dwError = WaitForSingleObject(
                (*(*Service).lpImage).hControlPipe,
                PipeTimeout,
            );

            if dwError == WAIT_TIMEOUT {
                bResult = CancelIo((*(*Service).lpImage).hControlPipe);
                if bResult == FALSE as BOOL {
                    let _ = GetLastError();
                }

                return ERROR_SERVICE_REQUEST_TIMEOUT;
            } else if dwError == WAIT_OBJECT_0 {
                bResult = GetOverlappedResult(
                    (*(*Service).lpImage).hControlPipe,
                    &mut Overlapped,
                    &mut dwRead,
                    TRUE as BOOL,
                );
                if bResult == FALSE as BOOL {
                    dwError = GetLastError();
                    return dwError;
                }
            }
        } else if dwError != ERROR_PIPE_CONNECTED {
            return dwError;
        }
    }

    Overlapped.hEvent = core::ptr::null_mut();

    bResult = ReadFile(
        (*(*Service).lpImage).hControlPipe,
        &mut dwProcessId as *mut DWORD as LPVOID,
        core::mem::size_of::<DWORD>() as DWORD,
        &mut dwRead,
        &mut Overlapped,
    );
    if bResult == FALSE as BOOL {
        dwError = GetLastError();
        if dwError == ERROR_IO_PENDING {
            dwError = WaitForSingleObject(
                (*(*Service).lpImage).hControlPipe,
                PipeTimeout,
            );
            if dwError == WAIT_TIMEOUT {
                bResult = CancelIo((*(*Service).lpImage).hControlPipe);
                if bResult == FALSE as BOOL {
                    let _ = GetLastError();
                }

                return ERROR_SERVICE_REQUEST_TIMEOUT;
            } else if dwError == WAIT_OBJECT_0 {
                bResult = GetOverlappedResult(
                    (*(*Service).lpImage).hControlPipe,
                    &mut Overlapped,
                    &mut dwRead,
                    TRUE as BOOL,
                );
                if bResult == FALSE as BOOL {
                    dwError = GetLastError();
                    return dwError;
                }
            }
        } else {
            return dwError;
        }
    }

    if (ScmIsSecurityService((*Service).lpImage) == FALSE as BOOL)
        && (dwProcessId != (*(*Service).lpImage).dwProcessId)
    {
    }

    ERROR_SUCCESS
}

unsafe fn ScmStartUserModeService(
    Service: PSERVICE,
    argc: DWORD,
    argv: *const PCWSTR,
) -> DWORD {

    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut lpEnvironment: LPVOID = core::ptr::null_mut();
    let mut Result: BOOL = 0;
    let mut dwError: DWORD = ERROR_SUCCESS;

    'quit: {
        /* If the image is already running, just send a start command */
        if (*(*Service).lpImage).dwImageRunCount > 1 {
            break 'quit;
        }

        /* Otherwise start its process */
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(StartupInfo) as *mut u8,
            0,
            core::mem::size_of::<STARTUPINFOW>(),
        );
        StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        core::ptr::write_bytes(
            core::ptr::addr_of_mut!(ProcessInformation) as *mut u8,
            0,
            core::mem::size_of::<PROCESS_INFORMATION>(),
        );

        if !(*(*Service).lpImage).hToken.is_null() {
            /* User token: Run the service under the user account */

            if CreateEnvironmentBlock(
                core::ptr::addr_of_mut!(lpEnvironment),
                (*(*Service).lpImage).hToken,
                0,
            ) == 0
            {
                lpEnvironment = core::ptr::null_mut();
            }

            /* Impersonate the new user */
            Result = ImpersonateLoggedOnUser((*(*Service).lpImage).hToken);
            if Result != 0 {
                Result = CreateProcessAsUserW(
                    (*(*Service).lpImage).hToken,
                    core::ptr::null_mut(),
                    (*(*Service).lpImage).pszImagePath,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                    CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS | CREATE_SUSPENDED,
                    lpEnvironment,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(StartupInfo),
                    core::ptr::addr_of_mut!(ProcessInformation),
                );
                if Result == 0 {
                    dwError = GetLastError();
                }

                RevertToSelf();
            } else {
                dwError = GetLastError();
            }
        } else {
            /* No user token: Run the service under the LocalSystem account */

            if CreateEnvironmentBlock(
                core::ptr::addr_of_mut!(lpEnvironment),
                core::ptr::null_mut(),
                1,
            ) == 0
            {
                lpEnvironment = core::ptr::null_mut();
            }

            if (NoInteractiveServices == 0)
                && ((*Service).Status.dwServiceType & SERVICE_INTERACTIVE_PROCESS) != 0
            {
                StartupInfo.dwFlags |= STARTF_INHERITDESKTOP;
                let desktop: [u16; 16] = [
                    b'W' as u16, b'i' as u16, b'n' as u16, b'S' as u16, b't' as u16,
                    b'a' as u16, b'0' as u16, 0x005C, b'D' as u16, b'e' as u16,
                    b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16,
                    0x0000,
                ];
                static DESKTOP: [u16; 16] = [
                    b'W' as u16, b'i' as u16, b'n' as u16, b'S' as u16, b't' as u16,
                    b'a' as u16, b'0' as u16, 0x005C, b'D' as u16, b'e' as u16,
                    b'f' as u16, b'a' as u16, b'u' as u16, b'l' as u16, b't' as u16,
                    0x0000,
                ];
                let _ = desktop;
                StartupInfo.lpDesktop = DESKTOP.as_ptr() as LPWSTR;
            }

            if ScmIsSecurityService((*Service).lpImage) == 0 {
                DbgPrint(b"[SERVICES] SUMS(%S): before CreateProcessW\n\0".as_ptr() as *const i8, (*Service).lpServiceName);
                Result = CreateProcessW(
                    core::ptr::null_mut(),
                    (*(*Service).lpImage).pszImagePath,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    0,
                    CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS | CREATE_SUSPENDED,
                    lpEnvironment,
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(StartupInfo),
                    core::ptr::addr_of_mut!(ProcessInformation),
                );
                if Result == 0 {
                    dwError = GetLastError();
                }
            } else {
                Result = 1;
                dwError = ERROR_SUCCESS;
            }
        }

        if !lpEnvironment.is_null() {
            DestroyEnvironmentBlock(lpEnvironment);
        }

        if Result == 0 {
            return dwError;
        }

        DbgPrint(b"[SERVICES] SUMS(%S): CreateProcessW returned pid=%lu\n\0".as_ptr() as *const i8, (*Service).lpServiceName, ProcessInformation.dwProcessId);
        /* Get the process handle and ID */
        (*(*Service).lpImage).hProcess = ProcessInformation.hProcess;
        (*(*Service).lpImage).dwProcessId = ProcessInformation.dwProcessId;

        /* Resume the main thread and close its handle */
        ResumeThread(ProcessInformation.hThread);
        DbgPrint(b"[SERVICES] SUMS(%S): after ResumeThread\n\0".as_ptr() as *const i8, (*Service).lpServiceName);
        CloseHandle(ProcessInformation.hThread);

        /* Connect control pipe */
        DbgPrint(b"[SERVICES] ScmStartUserModeService(%S): waiting for connect\n\0".as_ptr() as *const i8, (*Service).lpServiceName);
        dwError = ScmWaitForServiceConnect(Service);
        DbgPrint(b"[SERVICES] ScmStartUserModeService(%S): connect result=%lu\n\0".as_ptr() as *const i8, (*Service).lpServiceName, dwError);
        if dwError != ERROR_SUCCESS {
            (*(*Service).lpImage).dwProcessId = 0;
            return dwError;
        }
    }

    /* Send the start command and return */
    DbgPrint(b"[SERVICES] ScmStartUserModeService(%S): sending START (untimed?)\n\0".as_ptr() as *const i8, (*Service).lpServiceName);
    let _r = ScmControlServiceEx(
        (*(*Service).lpImage).hControlPipe,
        (*Service).lpServiceName,
        if ((*Service).Status.dwServiceType & SERVICE_WIN32_OWN_PROCESS) != 0 {
            SERVICE_CONTROL_START_OWN
        } else {
            SERVICE_CONTROL_START_SHARE
        },
        Service as SERVICE_STATUS_HANDLE,
        (*Service).dwServiceTag,
        argc,
        argv,
    );
    DbgPrint(b"[SERVICES] ScmStartUserModeService(%S): START returned=%lu\n\0".as_ptr() as *const i8, (*Service).lpServiceName, _r);
    _r
}

unsafe fn ScmLoadService(
    Service: PSERVICE,
    argc: DWORD,
    argv: *const PCWSTR,
) -> DWORD {

    let Group: PSERVICE_GROUP = (*Service).lpGroup;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut lpLogStrings: [LPCWSTR; 2] = [core::ptr::null(); 2];
    let mut szLogBuffer: [WCHAR; 80] = [0; 80];

    DbgPrint(
        b"[SERVICES] ScmLoadService(%S) type=%lu\n\0".as_ptr() as *const i8,
        (*Service).lpServiceName,
        (*Service).Status.dwServiceType,
    );

    if (*Service).Status.dwCurrentState != SERVICE_STOPPED {
        return ERROR_SERVICE_ALREADY_RUNNING;
    }

    if ((*Service).Status.dwServiceType & SERVICE_DRIVER) != 0 {
        dwError = ScmStartDriver(Service);
    } else {
        DbgPrint(
            b"[SERVICES] CreateOrReferenceServiceImage(%S)\n\0".as_ptr() as *const i8,
            (*Service).lpServiceName,
        );
        dwError = ScmCreateOrReferenceServiceImage(Service);
        DbgPrint(
            b"[SERVICES] CreateOrReferenceServiceImage(%S) = %lu\n\0".as_ptr() as *const i8,
            (*Service).lpServiceName,
            dwError,
        );
        if dwError == ERROR_SUCCESS {
            DbgPrint(
                b"[SERVICES] ScmStartUserModeService(%S)\n\0".as_ptr() as *const i8,
                (*Service).lpServiceName,
            );
            dwError = ScmStartUserModeService(Service, argc, argv);
            if dwError == ERROR_SUCCESS {
                (*Service).Status.dwCurrentState = SERVICE_START_PENDING;
                (*Service).Status.dwControlsAccepted = 0;
                ScmReferenceService(Service);
            } else {
                (*(*Service).lpImage).dwImageRunCount -= 1;
                if (*(*Service).lpImage).dwImageRunCount == 0 {
                    ScmRemoveServiceImage((*Service).lpImage);
                    (*Service).lpImage = core::ptr::null_mut();
                }
            }
        }
    }

    DbgPrint(
        b"[SERVICES] ScmLoadService(%S) done: %lu\n\0".as_ptr() as *const i8,
        (*Service).lpServiceName,
        dwError,
    );

    if dwError == ERROR_SUCCESS {
        if !Group.is_null() {
            (*Group).ServicesRunning = TRUE as BOOLEAN;
        }

        DbgPrint(
            b"[SERVICES] ScmLoadService(%S) logging success event\n\0".as_ptr() as *const i8,
            (*Service).lpServiceName,
        );

        LoadStringW(
            GetModuleHandleW(core::ptr::null_mut()),
            IDS_SERVICE_START,
            szLogBuffer.as_mut_ptr(),
            80,
        );
        DbgPrint(
            b"[SERVICES] ScmLoadService(%S) LoadStringW done\n\0".as_ptr() as *const i8,
            (*Service).lpServiceName,
        );
        lpLogStrings[0] = (*Service).lpDisplayName;
        lpLogStrings[1] = szLogBuffer.as_ptr();

        ScmLogEvent(
            EVENT_SERVICE_CONTROL_SUCCESS,
            EVENTLOG_INFORMATION_TYPE as WORD,
            2,
            lpLogStrings.as_mut_ptr(),
        );
        DbgPrint(
            b"[SERVICES] ScmLoadService(%S) event logged OK\n\0".as_ptr() as *const i8,
            (*Service).lpServiceName,
        );
    } else {
        if (*Service).dwErrorControl != SERVICE_ERROR_IGNORE {
            // StringCchPrintfW(szLogBuffer, len, L"%lu", dwError)
            scm_append_u32(szLogBuffer.as_mut_ptr(), szLogBuffer.len(), 0, dwError);
            lpLogStrings[0] = (*Service).lpServiceName;
            lpLogStrings[1] = szLogBuffer.as_ptr();
            ScmLogEvent(
                EVENT_SERVICE_START_FAILED,
                EVENTLOG_ERROR_TYPE as WORD,
                2,
                lpLogStrings.as_mut_ptr(),
            );
        }
    }

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmStartService(
    Service: PSERVICE,
    argc: DWORD,
    argv: *const PCWSTR,
) -> DWORD {

    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut Lock: SC_RPC_LOCK = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));

    'done: {
        if ScmInitialize == 0 {
            dwError = ScmAcquireServiceStartLock(1, &mut Lock);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        dwError = ScmLoadService(Service, argc, argv);

        if !Lock.is_null() {
            ScmReleaseServiceStartLock(&mut Lock);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmAutoStartServices() {

    let mut dwError: DWORD;
    let mut GroupEntry: PLIST_ENTRY;
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentGroup: PSERVICE_GROUP;
    let mut CurrentService: PSERVICE;
    let mut szSafeBootServicePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut SafeBootEnabled: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwKeySize: DWORD;
    let mut i: ULONG;

    let mut OptionPath: [u16; 64] = [0; 64];
    {
        let s = "SYSTEM\\CurrentControlSet\\Control\\SafeBoot\\Option";
        let mut idx = 0;
        for c in s.encode_utf16() {
            OptionPath[idx] = c;
            idx += 1;
        }
        OptionPath[idx] = 0;
    }

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        OptionPath.as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    ) as DWORD;
    if dwError == ERROR_SUCCESS {
        dwKeySize = core::mem::size_of::<DWORD>() as DWORD;
        let mut OptionValue: [u16; 16] = [0; 16];
        {
            let s = "OptionValue";
            let mut idx = 0;
            for c in s.encode_utf16() {
                OptionValue[idx] = c;
                idx += 1;
            }
            OptionValue[idx] = 0;
        }
        dwError = RegQueryValueExW(
            hKey,
            OptionValue.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut SafeBootEnabled as *mut DWORD as LPBYTE,
            &mut dwKeySize,
        ) as DWORD;
        RegCloseKey(hKey);
    }

    if dwError != ERROR_SUCCESS {
        SafeBootEnabled = 0;
    }

    DbgPrint(b"[SERVICES] ScmAutoStartServices entered, SafeBoot=%lu\n\0".as_ptr() as *const i8, SafeBootEnabled);

    EnterCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection) as PRTL_CRITICAL_SECTION);

    let to_wstr = |dst: &mut [u16], s: &str| {
        let mut idx = 0;
        for c in s.encode_utf16() {
            dst[idx] = c;
            idx += 1;
        }
        dst[idx] = 0;
    };

    let mut basePath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    to_wstr(&mut basePath, "SYSTEM\\CurrentControlSet\\Control\\SafeBoot");
    let mut minimalSuffix: [u16; 16] = [0; 16];
    to_wstr(&mut minimalSuffix, "\\Minimal\\");
    let mut networkSuffix: [u16; 16] = [0; 16];
    to_wstr(&mut networkSuffix, "\\Network\\");

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        StringCchCopyW(
            szSafeBootServicePath.as_mut_ptr(),
            szSafeBootServicePath.len(),
            basePath.as_ptr(),
        );

        match SafeBootEnabled {
            1 | 3 => {
                StringCchCatW(
                    szSafeBootServicePath.as_mut_ptr(),
                    szSafeBootServicePath.len(),
                    minimalSuffix.as_ptr(),
                );
            }
            2 => {
                StringCchCatW(
                    szSafeBootServicePath.as_mut_ptr(),
                    szSafeBootServicePath.len(),
                    networkSuffix.as_ptr(),
                );
            }
            _ => {}
        }

        if SafeBootEnabled != 0 {
            dwError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                szSafeBootServicePath.as_ptr(),
                0,
                KEY_READ,
                &mut hKey,
            ) as DWORD;
            if dwError == ERROR_SUCCESS {
                RegCloseKey(hKey);

                StringCchCatW(
                    szSafeBootServicePath.as_mut_ptr(),
                    szSafeBootServicePath.len(),
                    (*CurrentService).lpServiceName,
                );

                dwError = RegOpenKeyExW(
                    HKEY_LOCAL_MACHINE,
                    szSafeBootServicePath.as_ptr(),
                    0,
                    KEY_READ,
                    &mut hKey,
                ) as DWORD;
                if dwError != ERROR_SUCCESS {
                    (*CurrentService).ServiceVisited = 1;
                } else {
                    RegCloseKey(hKey);
                    (*CurrentService).ServiceVisited = 0;
                }
            } else {
                (*CurrentService).ServiceVisited = 0;
            }
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    DbgPrint(b"[SERVICES] Starting grouped services\n\0".as_ptr() as *const i8);

    GroupEntry = GroupListHead.Flink;
    while GroupEntry != core::ptr::addr_of_mut!(GroupListHead) {
        CurrentGroup = (GroupEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE_GROUP, GroupListEntry) as isize))
            as PSERVICE_GROUP;

        i = 0;
        while i < (*CurrentGroup).TagCount {
            ServiceEntry = ServiceListHead.Flink;
            while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
                CurrentService = (ServiceEntry as *mut u8)
                    .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
                    as PSERVICE;

                if (*CurrentService).lpGroup == CurrentGroup
                    && (*CurrentService).dwStartType == SERVICE_AUTO_START
                    && (*CurrentService).ServiceVisited == 0
                    && (*CurrentService).dwTag == *(*CurrentGroup).TagArray.offset(i as isize)
                {
                    (*CurrentService).ServiceVisited = 1;
                    DbgPrint(b"[SERVICES] Loading(tag): %S\n\0".as_ptr() as *const i8, (*CurrentService).lpServiceName);
                    ScmLoadService(CurrentService, 0, core::ptr::null());
                }

                ServiceEntry = (*ServiceEntry).Flink;
            }
            i += 1;
        }

        ServiceEntry = ServiceListHead.Flink;
        while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as *mut u8)
                .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
                as PSERVICE;

            if (*CurrentService).lpGroup == CurrentGroup
                && (*CurrentService).dwStartType == SERVICE_AUTO_START
                && (*CurrentService).ServiceVisited == 0
            {
                (*CurrentService).ServiceVisited = 1;
                DbgPrint(b"[SERVICES] Loading(grp): %S\n\0".as_ptr() as *const i8, (*CurrentService).lpServiceName);
                ScmLoadService(CurrentService, 0, core::ptr::null());
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        GroupEntry = (*GroupEntry).Flink;
    }

    DbgPrint(b"[SERVICES] Starting non-existing-group services\n\0".as_ptr() as *const i8);

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        if !(*CurrentService).lpGroup.is_null()
            && (*CurrentService).dwStartType == SERVICE_AUTO_START
            && (*CurrentService).ServiceVisited == 0
        {
            (*CurrentService).ServiceVisited = 1;
            DbgPrint(b"[SERVICES] Loading(nogrp): %S\n\0".as_ptr() as *const i8, (*CurrentService).lpServiceName);
            ScmLoadService(CurrentService, 0, core::ptr::null());
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    DbgPrint(b"[SERVICES] Starting ungrouped services\n\0".as_ptr() as *const i8);

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;

        if (*CurrentService).lpGroup.is_null()
            && (*CurrentService).dwStartType == SERVICE_AUTO_START
            && (*CurrentService).ServiceVisited == 0
        {
            (*CurrentService).ServiceVisited = 1;
            DbgPrint(b"[SERVICES] Loading(none): %S\n\0".as_ptr() as *const i8, (*CurrentService).lpServiceName);
            ScmLoadService(CurrentService, 0, core::ptr::null());
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .offset(-(core::mem::offset_of!(SERVICE, ServiceListEntry) as isize))
            as PSERVICE;
        (*CurrentService).ServiceVisited = 0;
        ServiceEntry = (*ServiceEntry).Flink;
    }

    DbgPrint(b"[SERVICES] ScmAutoStartServices completed\n\0".as_ptr() as *const i8);

    LeaveCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection) as PRTL_CRITICAL_SECTION);
}
#[no_mangle]
pub unsafe extern "C" fn ScmAutoShutdownServices() {
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;

    ScmLockDatabaseExclusive();

    ServiceEntry = ServiceListHead.Flink;
    while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
        CurrentService = (ServiceEntry as *mut u8)
            .sub(core::mem::offset_of!(_SERVICE, ServiceListEntry)) as PSERVICE;

        if ((*CurrentService).Status.dwControlsAccepted & SERVICE_ACCEPT_SHUTDOWN) != 0
            && ((*CurrentService).Status.dwCurrentState == SERVICE_RUNNING
                || (*CurrentService).Status.dwCurrentState == SERVICE_START_PENDING)
        {
            ScmControlService(
                (*(*CurrentService).lpImage).hControlPipe,
                (*CurrentService).lpServiceName,
                SERVICE_CONTROL_SHUTDOWN,
                CurrentService as SERVICE_STATUS_HANDLE,
            );
        }

        ServiceEntry = (*ServiceEntry).Flink;
    }

    ScmUnlockDatabase();
}
#[no_mangle]
pub unsafe extern "C" fn ScmLockDatabaseExclusive() -> BOOL {
    RtlAcquireResourceExclusive(core::ptr::addr_of_mut!(DatabaseLock), 1u8) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn ScmLockDatabaseShared() -> BOOL {
    RtlAcquireResourceShared(core::ptr::addr_of_mut!(DatabaseLock), 1u8) as BOOL
}
#[no_mangle]
pub unsafe extern "C" fn ScmUnlockDatabase() {
    RtlReleaseResource(core::ptr::addr_of_mut!(DatabaseLock));
}
#[no_mangle]
pub unsafe extern "C" fn ScmInitNamedPipeCriticalSection() {

    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwKeySize: DWORD;
    let dwError: DWORD;

    InitializeCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));

    let subkey: [u16; 35] = [
        b'S' as u16, b'Y' as u16, b'S' as u16, b'T' as u16, b'E' as u16, b'M' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
        b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
        b'o' as u16, b'l' as u16, 0, 0, 0,
    ];

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        subkey.as_ptr(),
        0,
        KEY_READ,
        core::ptr::addr_of_mut!(hKey),
    ) as DWORD;

    if dwError == ERROR_SUCCESS {
        dwKeySize = core::mem::size_of_val(&PipeTimeout) as DWORD;
        let valueName: [u16; 20] = [
            b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
            b'e' as u16, b's' as u16, b'P' as u16, b'i' as u16, b'p' as u16, b'e' as u16,
            b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, b'o' as u16, b'u' as u16,
            b't' as u16, 0,
        ];
        RegQueryValueExW(
            hKey,
            valueName.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(PipeTimeout) as LPBYTE,
            core::ptr::addr_of_mut!(dwKeySize),
        );
        RegCloseKey(hKey);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScmDeleteNamedPipeCriticalSection() {
    DeleteCriticalSection(core::ptr::addr_of_mut!(ControlServiceCriticalSection));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Service Control Manager
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/system/services/database.c
//  * PURPOSE:     Database control interface
//  * COPYRIGHT:   Copyright 2002-2006 Eric Kohl
//  *              Copyright 2006 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2007 Ged Murphy <gedmurphy@reactos.org>
//  *                             Gregor Brunmar <gregor.brunmar@home.se>
//  *
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "services.h"
//
// #include <winbase_undoc.h>
// #include <userenv.h>
// #include <strsafe.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS *******************************************************************/
//
// LIST_ENTRY ImageListHead;
// LIST_ENTRY ServiceListHead;
//
// static RTL_RESOURCE DatabaseLock;
// static DWORD ResumeCount = 1;
// static DWORD NoInteractiveServices = 0;
// static DWORD ServiceTag = 0;
//
// /* The critical section synchronizes service control requests */
// static CRITICAL_SECTION ControlServiceCriticalSection;
// static DWORD PipeTimeout = 30000; /* 30 Seconds */
//
//
// /* FUNCTIONS *****************************************************************/
//
// static
// BOOL
// ScmIsSecurityService(
//      PSERVICE_IMAGE pServiceImage)
// {
//     return (wcsstr(pServiceImage->pszImagePath, L"\\system32\\lsass.exe") != NULL);
// }
//
//
// static DWORD
// ScmCreateNewControlPipe(
//      PSERVICE_IMAGE pServiceImage,
//      BOOL bSecurityServiceProcess)
// {
//     WCHAR szControlPipeName[MAX_PATH + 1];
//     SECURITY_ATTRIBUTES SecurityAttributes;
//     HKEY hServiceCurrentKey = INVALID_HANDLE_VALUE;
//     DWORD dwServiceCurrent = 1;
//     DWORD dwKeyDisposition;
//     DWORD dwKeySize;
//     DWORD dwError;
//
//     /* Get the service number */
//     if (bSecurityServiceProcess == FALSE)
//     {
//         /* TODO: Create registry entry with correct write access */
//         dwError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                                   L"SYSTEM\\CurrentControlSet\\Control\\ServiceCurrent",
//                                   0,
//                                   NULL,
//                                   REG_OPTION_VOLATILE,
//                                   KEY_WRITE | KEY_READ,
//                                   NULL,
//                                   &hServiceCurrentKey,
//                                   &dwKeyDisposition) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegCreateKeyEx() failed with error %lu\n", dwError);
//             return dwError;
//         }
//
//         if (dwKeyDisposition == REG_OPENED_EXISTING_KEY)
//         {
//             dwKeySize = sizeof(DWORD);
//             dwError = RegQueryValueExW(hServiceCurrentKey,
//                                        L"",
//                                        0,
//                                        NULL,
//                                        (BYTE*)&dwServiceCurrent,
//                                        &dwKeySize) as DWORD;
//             if (dwError != ERROR_SUCCESS)
//             {
//                 RegCloseKey(hServiceCurrentKey);
//                 DPRINT1("RegQueryValueEx() failed with error %lu\n", dwError);
//                 return dwError;
//             }
//
//             dwServiceCurrent++;
//         }
//
//         dwError = RegSetValueExW(hServiceCurrentKey,
//                                  L"",
//                                  0,
//                                  REG_DWORD,
//                                  (BYTE*)&dwServiceCurrent,
//                                  sizeof(dwServiceCurrent)) as DWORD;
//
//         RegCloseKey(hServiceCurrentKey);
//
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("RegSetValueExW() failed (Error %lu)\n", dwError);
//             return dwError;
//         }
//     }
//     else
//     {
//         dwServiceCurrent = 0;
//     }
//
//     /* Create '\\.\pipe\net\NtControlPipeXXX' instance */
//     StringCchPrintfW(szControlPipeName, ARRAYSIZE(szControlPipeName),
//                      L"\\\\.\\pipe\\net\\NtControlPipe%lu", dwServiceCurrent);
//
//     DPRINT("PipeName: %S\n", szControlPipeName);
//
//     SecurityAttributes.nLength = sizeof(SecurityAttributes);
//     SecurityAttributes.lpSecurityDescriptor = pPipeSD;
//     SecurityAttributes.bInheritHandle = FALSE;
//
//     pServiceImage->hControlPipe = CreateNamedPipeW(szControlPipeName,
//                                                    PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
//                                                    PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT,
//                                                    100,
//                                                    8000,
//                                                    4,
//                                                    PipeTimeout,
//                                                    &SecurityAttributes);
//     DPRINT("CreateNamedPipeW(%S) done\n", szControlPipeName);
//     if (pServiceImage->hControlPipe == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("Failed to create control pipe\n");
//         return GetLastError();
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// static PSERVICE_IMAGE
// ScmGetServiceImageByImagePath(LPWSTR lpImagePath)
// {
//     PLIST_ENTRY ImageEntry;
//     PSERVICE_IMAGE CurrentImage;
//
//     DPRINT("ScmGetServiceImageByImagePath(%S) called\n", lpImagePath);
//
//     ImageEntry = ImageListHead.Flink;
//     while (ImageEntry != &ImageListHead)
//     {
//         CurrentImage = CONTAINING_RECORD(ImageEntry,
//                                          SERVICE_IMAGE,
//                                          ImageListEntry);
//         if (_wcsicmp(CurrentImage->pszImagePath, lpImagePath) == 0)
//         {
//             DPRINT("Found image: '%S'\n", CurrentImage->pszImagePath);
//             return CurrentImage;
//         }
//
//         ImageEntry = ImageEntry->Flink;
//     }
//
//     DPRINT("Couldn't find a matching image\n");
//
//     return NULL;
//
// }
//
//
// DWORD
// ScmGetServiceNameFromTag(IN PTAG_INFO_NAME_FROM_TAG_IN_PARAMS InParams,
//                          OUT PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS *OutParams)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//     PSERVICE_IMAGE CurrentImage;
//     PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS OutBuffer = NULL;
//     DWORD dwError;
//
//     /* Lock the database */
//     ScmLockDatabaseExclusive();
//
//     /* Find the matching service */
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         /* We must match the tag */
//         if (CurrentService->dwServiceTag == InParams->dwTag &&
//             CurrentService->lpImage != NULL)
//         {
//             CurrentImage = CurrentService->lpImage;
//             /* And matching the PID */
//             if (CurrentImage->dwProcessId == InParams->dwPid)
//             {
//                 break;
//             }
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     /* No match! */
//     if (ServiceEntry == &ServiceListHead)
//     {
//         dwError = ERROR_RETRY;
//         goto Cleanup;
//     }
//
//     /* Allocate the output buffer */
//     OutBuffer = MIDL_user_allocate(sizeof(TAG_INFO_NAME_FROM_TAG_OUT_PARAMS));
//     if (OutBuffer == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto Cleanup;
//     }
//
//     /* And the buffer for the name */
//     OutBuffer->pszName = MIDL_user_allocate(wcslen(CurrentService->lpServiceName) * sizeof(WCHAR) + sizeof(UNICODE_NULL));
//     if (OutBuffer->pszName == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto Cleanup;
//     }
//
//     /* Fill in output data */
//     wcscpy(OutBuffer->pszName, CurrentService->lpServiceName);
//     OutBuffer->TagType = TagTypeService;
//
//     /* And return */
//     *OutParams = OutBuffer;
//     dwError = ERROR_SUCCESS;
//
// Cleanup:
//
//     /* Unlock database */
//     ScmUnlockDatabase();
//
//     /* If failure, free allocated memory */
//     if (dwError != ERROR_SUCCESS)
//     {
//         if (OutBuffer != NULL)
//         {
//             MIDL_user_free(OutBuffer);
//         }
//     }
//
//     /* Return error/success */
//     return dwError;
// }
//
//
// static
// BOOL
// ScmIsSameServiceAccount(
//      PCWSTR pszAccountName1,
//      PCWSTR pszAccountName2)
// {
//     if (pszAccountName1 == NULL &&
//         pszAccountName2 == NULL)
//         return TRUE;
//
//     if (pszAccountName1 == NULL &&
//         pszAccountName2 != NULL &&
//         _wcsicmp(pszAccountName2, L"LocalSystem") == 0)
//         return TRUE;
//
//     if (pszAccountName1 != NULL &&
//         pszAccountName2 == NULL &&
//         _wcsicmp(pszAccountName1, L"LocalSystem") == 0)
//         return TRUE;
//
//     if (pszAccountName1 != NULL &&
//         pszAccountName2 != NULL &&
//         _wcsicmp(pszAccountName1, pszAccountName2) == 0)
//         return TRUE;
//
//     return FALSE;
// }
//
//
// static
// BOOL
// ScmIsLocalSystemAccount(
//      PCWSTR pszAccountName)
// {
//     if (pszAccountName == NULL ||
//         _wcsicmp(pszAccountName, L"LocalSystem") == 0)
//         return TRUE;
//
//     return FALSE;
// }
//
//
// static
// BOOL
// ScmEnableBackupRestorePrivileges(
//      HANDLE hToken,
//      BOOL bEnable)
// {
//     PTOKEN_PRIVILEGES pTokenPrivileges = NULL;
//     DWORD dwSize;
//     BOOL bRet = FALSE;
//
//     DPRINT("ScmEnableBackupRestorePrivileges(%p %d)\n", hToken, bEnable);
//
//     dwSize = sizeof(TOKEN_PRIVILEGES) + 2 * sizeof(LUID_AND_ATTRIBUTES);
//     pTokenPrivileges = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwSize);
//     if (pTokenPrivileges == NULL)
//     {
//         DPRINT1("Failed to allocate privilege buffer\n");
//         goto done;
//     }
//
//     pTokenPrivileges->PrivilegeCount = 2;
//     pTokenPrivileges->Privileges[0].Luid.LowPart = SE_BACKUP_PRIVILEGE;
//     pTokenPrivileges->Privileges[0].Luid.HighPart = 0;
//     pTokenPrivileges->Privileges[0].Attributes = (bEnable ? SE_PRIVILEGE_ENABLED : 0);
//     pTokenPrivileges->Privileges[1].Luid.LowPart = SE_RESTORE_PRIVILEGE;
//     pTokenPrivileges->Privileges[1].Luid.HighPart = 0;
//     pTokenPrivileges->Privileges[1].Attributes = (bEnable ? SE_PRIVILEGE_ENABLED : 0);
//
//     bRet = AdjustTokenPrivileges(hToken, FALSE, pTokenPrivileges, 0, NULL, NULL);
//     if (!bRet)
//     {
//         DPRINT1("AdjustTokenPrivileges() failed with error %lu\n", GetLastError());
//     }
//     else if (GetLastError() == ERROR_NOT_ALL_ASSIGNED)
//     {
//         DPRINT1("AdjustTokenPrivileges() succeeded, but with not all privileges assigned\n");
//         bRet = FALSE;
//     }
//
// done:
//     if (pTokenPrivileges != NULL)
//         HeapFree(GetProcessHeap(), 0, pTokenPrivileges);
//
//     return bRet;
// }
//
//
// static
// DWORD
// ScmLogonService(
//     IN PSERVICE pService,
//     IN PSERVICE_IMAGE pImage)
// {
//     PROFILEINFOW ProfileInfo;
//     PWSTR pszUserName = NULL;
//     PWSTR pszDomainName = NULL;
//     PWSTR pszPassword = NULL;
//     PWSTR ptr;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("ScmLogonService(%p %p)\n", pService, pImage);
//     DPRINT("Service %S\n", pService->lpServiceName);
//
//     if (ScmIsLocalSystemAccount(pImage->pszAccountName) || ScmLiveSetup || ScmSetupInProgress)
//         return ERROR_SUCCESS;
//
//     /* Get the user and domain names */
//     ptr = wcschr(pImage->pszAccountName, L'\\');
//     if (ptr != NULL)
//     {
//         *ptr = L'\0';
//         pszUserName = ptr + 1;
//         pszDomainName = pImage->pszAccountName;
//     }
//     else
//     {
//         // ERROR_INVALID_SERVICE_ACCOUNT
//         pszUserName = pImage->pszAccountName;
//         pszDomainName = NULL;
//     }
//
//     /* Build the service 'password' */
//     pszPassword = HeapAlloc(GetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             (wcslen(pService->lpServiceName) + 5) * sizeof(WCHAR));
//     if (pszPassword == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto done;
//     }
//
//     wcscpy(pszPassword, L"_SC_");
//     wcscat(pszPassword, pService->lpServiceName);
//
//     DPRINT("Domain: %S  User: %S  Password: %S\n", pszDomainName, pszUserName, pszPassword);
//
//     /* Do the service logon */
//     if (!LogonUserW(pszUserName,
//                     pszDomainName,
//                     pszPassword,
//                     LOGON32_LOGON_SERVICE,
//                     LOGON32_PROVIDER_DEFAULT,
//                     &pImage->hToken))
//     {
//         dwError = GetLastError();
//         DPRINT1("LogonUserW() failed (Error %lu)\n", dwError);
//
//         /* Normalize the returned error */
//         dwError = ERROR_SERVICE_LOGON_FAILED;
//         goto done;
//     }
//
//     /* Load the user profile; the per-user environment variables are thus correctly initialized */
//     ZeroMemory(&ProfileInfo, sizeof(ProfileInfo));
//     ProfileInfo.dwSize = sizeof(ProfileInfo);
//     ProfileInfo.dwFlags = PI_NOUI;
//     ProfileInfo.lpUserName = pszUserName;
//     // ProfileInfo.lpProfilePath = NULL;
//     // ProfileInfo.lpDefaultPath = NULL;
//     // ProfileInfo.lpServerName = NULL;
//     // ProfileInfo.lpPolicyPath = NULL;
//     // ProfileInfo.hProfile = NULL;
//
//     ScmEnableBackupRestorePrivileges(pImage->hToken, TRUE);
//     if (!LoadUserProfileW(pImage->hToken, &ProfileInfo))
//         dwError = GetLastError();
//     ScmEnableBackupRestorePrivileges(pImage->hToken, FALSE);
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("LoadUserProfileW() failed (Error %lu)\n", dwError);
//         goto done;
//     }
//
//     pImage->hProfile = ProfileInfo.hProfile;
//
// done:
//     if (pszPassword != NULL)
//         HeapFree(GetProcessHeap(), 0, pszPassword);
//
//     if (ptr != NULL)
//         *ptr = L'\\';
//
//     return dwError;
// }
//
//
// static DWORD
// ScmCreateOrReferenceServiceImage(PSERVICE pService)
// {
//     RTL_QUERY_REGISTRY_TABLE QueryTable[3];
//     UNICODE_STRING ImagePath;
//     UNICODE_STRING ObjectName;
//     PSERVICE_IMAGE pServiceImage = NULL;
//     NTSTATUS Status;
//     DWORD dwError = ERROR_SUCCESS;
//     DWORD dwRecordSize;
//     LPWSTR pString;
//     BOOL bSecurityService;
//
//     DPRINT("ScmCreateOrReferenceServiceImage(%p)\n", pService);
//
//     RtlInitUnicodeString(&ImagePath, NULL);
//     RtlInitUnicodeString(&ObjectName, NULL);
//
//     /* Get service data */
//     RtlZeroMemory(&QueryTable,
//                   sizeof(QueryTable));
//
//     QueryTable[0].Name = L"ImagePath";
//     QueryTable[0].Flags = RTL_QUERY_REGISTRY_DIRECT | RTL_QUERY_REGISTRY_REQUIRED;
//     QueryTable[0].EntryContext = &ImagePath;
//     QueryTable[1].Name = L"ObjectName";
//     QueryTable[1].Flags = RTL_QUERY_REGISTRY_DIRECT;
//     QueryTable[1].EntryContext = &ObjectName;
//
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_SERVICES,
//                                     pService->lpServiceName,
//                                     QueryTable,
//                                     NULL,
//                                     NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("RtlQueryRegistryValues() failed (Status %lx)\n", Status);
//         return RtlNtStatusToDosError(Status);
//     }
//
//     DPRINT("ImagePath: '%wZ'\n", &ImagePath);
//     DPRINT("ObjectName: '%wZ'\n", &ObjectName);
//
//     pServiceImage = ScmGetServiceImageByImagePath(ImagePath.Buffer);
//     if (pServiceImage == NULL)
//     {
//         dwRecordSize = sizeof(SERVICE_IMAGE) +
//                        ImagePath.Length + sizeof(WCHAR) +
//                        ((ObjectName.Length != 0) ? (ObjectName.Length + sizeof(WCHAR)) : 0);
//
//         /* Create a new service image */
//         pServiceImage = HeapAlloc(GetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   dwRecordSize);
//         if (pServiceImage == NULL)
//         {
//             dwError = ERROR_NOT_ENOUGH_MEMORY;
//             goto done;
//         }
//
//         pServiceImage->dwImageRunCount = 1;
//         pServiceImage->hControlPipe = INVALID_HANDLE_VALUE;
//         pServiceImage->hProcess = INVALID_HANDLE_VALUE;
//
//         pString = (PWSTR)((INT_PTR)pServiceImage + sizeof(SERVICE_IMAGE));
//
//         /* Set the image path */
//         pServiceImage->pszImagePath = pString;
//         wcscpy(pServiceImage->pszImagePath,
//                ImagePath.Buffer);
//
//         /* Set the account name */
//         if (ObjectName.Length > 0)
//         {
//             pString = pString + wcslen(pString) + 1;
//
//             pServiceImage->pszAccountName = pString;
//             wcscpy(pServiceImage->pszAccountName,
//                    ObjectName.Buffer);
//         }
//
//         /* Service logon */
//         dwError = ScmLogonService(pService, pServiceImage);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("ScmLogonService() failed (Error %lu)\n", dwError);
//
//             /* Release the service image */
//             HeapFree(GetProcessHeap(), 0, pServiceImage);
//
//             goto done;
//         }
//
//         bSecurityService = ScmIsSecurityService(pServiceImage);
//
//         /* Create the control pipe */
//         dwError = ScmCreateNewControlPipe(pServiceImage,
//                                           bSecurityService);
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT1("ScmCreateNewControlPipe() failed (Error %lu)\n", dwError);
//
//             /* Unload the user profile */
//             if (pServiceImage->hProfile != NULL)
//             {
//                 ScmEnableBackupRestorePrivileges(pServiceImage->hToken, TRUE);
//                 UnloadUserProfile(pServiceImage->hToken, pServiceImage->hProfile);
//                 ScmEnableBackupRestorePrivileges(pServiceImage->hToken, FALSE);
//             }
//
//             /* Close the logon token */
//             if (pServiceImage->hToken != NULL)
//                 CloseHandle(pServiceImage->hToken);
//
//             /* Release the service image */
//             HeapFree(GetProcessHeap(), 0, pServiceImage);
//
//             goto done;
//         }
//
//         if (bSecurityService)
//         {
//             SetSecurityServicesEvent();
//         }
//
//         /* FIXME: Add more initialization code here */
//
//
//         /* Append service record */
//         InsertTailList(&ImageListHead,
//                        &pServiceImage->ImageListEntry);
//     }
//     else
//     {
// //        if ((lpService->Status.dwServiceType & SERVICE_WIN32_SHARE_PROCESS) == 0)
//
//         /* Fail if services in an image use different accounts */
//         if (!ScmIsSameServiceAccount(pServiceImage->pszAccountName, ObjectName.Buffer))
//         {
//             dwError = ERROR_DIFFERENT_SERVICE_ACCOUNT;
//             goto done;
//         }
//
//         /* Increment the run counter */
//         pServiceImage->dwImageRunCount++;
//     }
//
//     DPRINT("pServiceImage->pszImagePath: %S\n", pServiceImage->pszImagePath);
//     DPRINT("pServiceImage->pszAccountName: %S\n", pServiceImage->pszAccountName);
//     DPRINT("pServiceImage->dwImageRunCount: %lu\n", pServiceImage->dwImageRunCount);
//
//     /* Link the service image to the service */
//     pService->lpImage = pServiceImage;
//
// done:
//     RtlFreeUnicodeString(&ObjectName);
//     RtlFreeUnicodeString(&ImagePath);
//
//     DPRINT("ScmCreateOrReferenceServiceImage() done (Error: %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// VOID
// ScmRemoveServiceImage(PSERVICE_IMAGE pServiceImage)
// {
//     DPRINT1("ScmRemoveServiceImage() called\n");
//
//     /* FIXME: Terminate the process */
//
//     /* Remove the service image from the list */
//     RemoveEntryList(&pServiceImage->ImageListEntry);
//
//     /* Close the process handle */
//     if (pServiceImage->hProcess != INVALID_HANDLE_VALUE)
//         CloseHandle(pServiceImage->hProcess);
//
//     /* Close the control pipe */
//     if (pServiceImage->hControlPipe != INVALID_HANDLE_VALUE)
//         CloseHandle(pServiceImage->hControlPipe);
//
//     /* Unload the user profile */
//     if (pServiceImage->hProfile != NULL)
//     {
//         ScmEnableBackupRestorePrivileges(pServiceImage->hToken, TRUE);
//         UnloadUserProfile(pServiceImage->hToken, pServiceImage->hProfile);
//         ScmEnableBackupRestorePrivileges(pServiceImage->hToken, FALSE);
//     }
//
//     /* Close the logon token */
//     if (pServiceImage->hToken != NULL)
//         CloseHandle(pServiceImage->hToken);
//
//     /* Release the service image */
//     HeapFree(GetProcessHeap(), 0, pServiceImage);
// }
//
//
// PSERVICE
// ScmGetServiceEntryByName(LPCWSTR lpServiceName)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     DPRINT("ScmGetServiceEntryByName() called\n");
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//         if (_wcsicmp(CurrentService->lpServiceName, lpServiceName) == 0)
//         {
//             DPRINT("Found service: '%S'\n", CurrentService->lpServiceName);
//             return CurrentService;
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DPRINT("Couldn't find a matching service\n");
//
//     return NULL;
// }
//
//
// PSERVICE
// ScmGetServiceEntryByDisplayName(LPCWSTR lpDisplayName)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     DPRINT("ScmGetServiceEntryByDisplayName() called\n");
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//         if (_wcsicmp(CurrentService->lpDisplayName, lpDisplayName) == 0)
//         {
//             DPRINT("Found service: '%S'\n", CurrentService->lpDisplayName);
//             return CurrentService;
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DPRINT("Couldn't find a matching service\n");
//
//     return NULL;
// }
//
//
// PSERVICE
// ScmGetServiceEntryByResumeCount(DWORD dwResumeCount)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     DPRINT("ScmGetServiceEntryByResumeCount() called\n");
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//         if (CurrentService->dwResumeCount > dwResumeCount)
//         {
//             DPRINT("Found service: '%S'\n", CurrentService->lpDisplayName);
//             return CurrentService;
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DPRINT("Couldn't find a matching service\n");
//
//     return NULL;
// }
//
//
// DWORD
// ScmGenerateServiceTag(PSERVICE lpServiceRecord)
// {
//     /* Check for an overflow */
//     if (ServiceTag == -1)
//     {
//         return ERROR_INVALID_DATA;
//     }
//
//     /* This is only valid for Win32 services */
//     if (!(lpServiceRecord->Status.dwServiceType & SERVICE_WIN32))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Increment the tag counter and set it */
//     ServiceTag = ServiceTag % 0xFFFFFFFF + 1;
//     lpServiceRecord->dwServiceTag = ServiceTag;
//
//     return ERROR_SUCCESS;
// }
//
//
// DWORD
// ScmCreateNewServiceRecord(LPCWSTR lpServiceName,
//                           PSERVICE *lpServiceRecord,
//                           DWORD dwServiceType,
//                           DWORD dwStartType)
// {
//     PSERVICE lpService = NULL;
//
//     DPRINT("Service: '%S'\n", lpServiceName);
//
//     /* Allocate service entry */
//     lpService = HeapAlloc(GetProcessHeap(),
//                           HEAP_ZERO_MEMORY,
//                           FIELD_OFFSET(SERVICE, szServiceName[wcslen(lpServiceName) + 1]));
//     if (lpService == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     *lpServiceRecord = lpService;
//
//     /* Copy service name */
//     wcscpy(lpService->szServiceName, lpServiceName);
//     lpService->lpServiceName = lpService->szServiceName;
//     lpService->lpDisplayName = lpService->lpServiceName;
//
//     /* Set the start type */
//     lpService->dwStartType = dwStartType;
//
//     /* Set the resume count */
//     lpService->dwResumeCount = ResumeCount++;
//
//     /* Append service record */
//     InsertTailList(&ServiceListHead,
//                    &lpService->ServiceListEntry);
//
//     /* Initialize the service status */
//     lpService->Status.dwServiceType = dwServiceType;
//     lpService->Status.dwCurrentState = SERVICE_STOPPED;
//     lpService->Status.dwControlsAccepted = 0;
//     lpService->Status.dwWin32ExitCode =
//         (dwStartType == SERVICE_DISABLED) ? ERROR_SERVICE_DISABLED : ERROR_SERVICE_NEVER_STARTED;
//     lpService->Status.dwServiceSpecificExitCode = 0;
//     lpService->Status.dwCheckPoint = 0;
//     lpService->Status.dwWaitHint =
//         (dwServiceType & SERVICE_DRIVER) ? 0 : 2000; /* 2 seconds */
//
//     return ERROR_SUCCESS;
// }
//
//
// VOID
// ScmDeleteServiceRecord(PSERVICE lpService)
// {
//     DPRINT("Deleting Service %S\n", lpService->lpServiceName);
//
//     /* Delete the display name */
//     if (lpService->lpDisplayName != NULL &&
//         lpService->lpDisplayName != lpService->lpServiceName)
//         HeapFree(GetProcessHeap(), 0, lpService->lpDisplayName);
//
//     /* Dereference the service image */
//     if (lpService->lpImage)
//     {
//         lpService->lpImage->dwImageRunCount--;
//
//         if (lpService->lpImage->dwImageRunCount == 0)
//         {
//             ScmRemoveServiceImage(lpService->lpImage);
//             lpService->lpImage = NULL;
//         }
//     }
//
//     /* Decrement the group reference counter */
//     ScmSetServiceGroup(lpService, NULL);
//
//     /* Release the SecurityDescriptor */
//     if (lpService->pSecurityDescriptor != NULL)
//         HeapFree(GetProcessHeap(), 0, lpService->pSecurityDescriptor);
//
//     /* Remove the Service from the List */
//     RemoveEntryList(&lpService->ServiceListEntry);
//
//     DPRINT("Deleted Service %S\n", lpService->lpServiceName);
//
//     /* Delete the service record */
//     HeapFree(GetProcessHeap(), 0, lpService);
//
//     DPRINT("Done\n");
// }
//
// DWORD
// Int_EnumDependentServicesW(HKEY hServicesKey,
//                            PSERVICE lpService,
//                            DWORD dwServiceState,
//                            PSERVICE *lpServices,
//                            LPDWORD pcbBytesNeeded,
//                            LPDWORD lpServicesReturned);
//
// DWORD ScmDeleteService(PSERVICE lpService)
// {
//     DWORD dwError;
//     DWORD pcbBytesNeeded = 0;
//     DWORD dwServicesReturned = 0;
//     HKEY hServicesKey;
//
//     ASSERT(lpService->RefCount == 0);
//
//     /* Open the Services Reg key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_SET_VALUE | KEY_READ,
//                             &hServicesKey) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to open services key\n");
//         return dwError;
//     }
//
//     /* Call the function with NULL, just to get bytes we need */
//     Int_EnumDependentServicesW(hServicesKey,
//                                lpService,
//                                SERVICE_ACTIVE,
//                                NULL,
//                                &pcbBytesNeeded,
//                                &dwServicesReturned);
//
//     /* If pcbBytesNeeded returned a value then there are services running that are dependent on this service */
//     if (pcbBytesNeeded)
//     {
//         DPRINT1("Deletion failed due to running dependencies\n");
//         RegCloseKey(hServicesKey);
//         return ERROR_DEPENDENT_SERVICES_RUNNING;
//     }
//
//     /* There are no references and no running dependencies,
//        it is now safe to delete the service */
//
//     /* Delete the Service Key */
//     dwError = ScmDeleteRegKey(hServicesKey, lpService->lpServiceName);
//
//     RegCloseKey(hServicesKey);
//
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Failed to delete the Service Registry key\n");
//         return dwError;
//     }
//
//     /* Delete the Service */
//     ScmDeleteServiceRecord(lpService);
//
//     return ERROR_SUCCESS;
// }
//
// /*
//  * This function allows the caller to be sure that the service won't be freed unexpectedly.
//  * In order to be sure that lpService will be valid until the reference is added
//  * the caller needs to hold the database lock.
//  * A running service will keep a reference for the whole time it is not SERVICE_STOPPED.
//  * A service handle will also keep a reference to a service. Keeping a reference is
//  * really needed so that ScmControlService can be called without keeping the database locked.
//  * This means that eventually the correct order of operations to send a control message to
//  * a service looks like: lock, reference, unlock, send control, lock, dereference, unlock.
//  */
// DWORD
// ScmReferenceService(PSERVICE lpService)
// {
//     return InterlockedIncrement(&lpService->RefCount);
// }
//
// /* This function must be called with the database lock held exclusively as
//    it can end up deleting the service */
// DWORD
// ScmDereferenceService(PSERVICE lpService)
// {
//     DWORD ref;
//
//     ASSERT(lpService->RefCount > 0);
//
//     ref = InterlockedDecrement(&lpService->RefCount);
//
//     if (ref == 0 && lpService->bDeleted &&
//         lpService->Status.dwCurrentState == SERVICE_STOPPED)
//     {
//         ScmDeleteService(lpService);
//     }
//     return ref;
// }
//
// static DWORD
// CreateServiceListEntry(LPCWSTR lpServiceName,
//                        HKEY hServiceKey)
// {
//     PSERVICE lpService = NULL;
//     LPWSTR lpDisplayName = NULL;
//     LPWSTR lpGroup = NULL;
//     DWORD dwSize;
//     DWORD dwError;
//     DWORD dwServiceType;
//     DWORD dwStartType;
//     DWORD dwErrorControl;
//     DWORD dwTagId;
//
//     DPRINT("Service: '%S'\n", lpServiceName);
//     if (*lpServiceName == L'{')
//         return ERROR_SUCCESS;
//
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hServiceKey,
//                                L"Type",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwServiceType,
//                                &dwSize) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         return ERROR_SUCCESS;
//
//     if (((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS) &&
//         ((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS) &&
//         (dwServiceType != SERVICE_KERNEL_DRIVER) &&
//         (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER))
//         return ERROR_SUCCESS;
//
//     DPRINT("Service type: %lx\n", dwServiceType);
//
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hServiceKey,
//                                L"Start",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwStartType,
//                                &dwSize) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         return ERROR_SUCCESS;
//
//     DPRINT("Start type: %lx\n", dwStartType);
//
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hServiceKey,
//                                L"ErrorControl",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwErrorControl,
//                                &dwSize) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         return ERROR_SUCCESS;
//
//     DPRINT("Error control: %lx\n", dwErrorControl);
//
//     dwError = RegQueryValueExW(hServiceKey,
//                                L"Tag",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwTagId,
//                                &dwSize) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         dwTagId = 0;
//
//     DPRINT("Tag: %lx\n", dwTagId);
//
//     dwError = ScmReadString(hServiceKey,
//                             L"Group",
//                             &lpGroup);
//     if (dwError != ERROR_SUCCESS)
//         lpGroup = NULL;
//
//     DPRINT("Group: %S\n", lpGroup);
//
//     dwError = ScmReadString(hServiceKey,
//                             L"DisplayName",
//                             &lpDisplayName);
//     if (dwError != ERROR_SUCCESS)
//         lpDisplayName = NULL;
//
//     DPRINT("Display name: %S\n", lpDisplayName);
//
//     dwError = ScmCreateNewServiceRecord(lpServiceName,
//                                         &lpService,
//                                         dwServiceType,
//                                         dwStartType);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     lpService->dwErrorControl = dwErrorControl;
//     lpService->dwTag = dwTagId;
//
//     if (lpGroup != NULL)
//     {
//         dwError = ScmSetServiceGroup(lpService, lpGroup);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     if (lpDisplayName != NULL)
//     {
//         lpService->lpDisplayName = lpDisplayName;
//         lpDisplayName = NULL;
//     }
//
//     DPRINT("ServiceName: '%S'\n", lpService->lpServiceName);
//     if (lpService->lpGroup != NULL)
//     {
//         DPRINT("Group: '%S'\n", lpService->lpGroup->lpGroupName);
//     }
//     DPRINT("Start %lx  Type %lx  Tag %lx  ErrorControl %lx\n",
//            lpService->dwStartType,
//            lpService->Status.dwServiceType,
//            lpService->dwTag,
//            lpService->dwErrorControl);
//
//     if (ScmIsDeleteFlagSet(hServiceKey))
//         lpService->bDeleted = TRUE;
//     else
//         ScmGenerateServiceTag(lpService);
//
//     if (lpService->Status.dwServiceType & SERVICE_WIN32)
//     {
//         dwError = ScmReadSecurityDescriptor(hServiceKey,
//                                             &lpService->pSecurityDescriptor);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         /* Assing the default security descriptor if the security descriptor cannot be read */
//         if (lpService->pSecurityDescriptor == NULL)
//         {
//             DPRINT("No security descriptor found! Assign default security descriptor\n");
//             dwError = ScmCreateDefaultServiceSD(&lpService->pSecurityDescriptor);
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//
//             dwError = ScmWriteSecurityDescriptor(hServiceKey,
//                                                  lpService->pSecurityDescriptor);
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//         }
//     }
//
// done:
//     if (lpGroup != NULL)
//         HeapFree(GetProcessHeap(), 0, lpGroup);
//
//     if (lpDisplayName != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDisplayName);
//
//     if (lpService != NULL)
//     {
//         ASSERT(lpService->lpImage == NULL);
//     }
//
//     return dwError;
// }
//
//
// VOID
// ScmDeleteMarkedServices(VOID)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//     HKEY hServicesKey;
//     DWORD dwError;
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         ServiceEntry = ServiceEntry->Flink;
//
//         if (CurrentService->bDeleted != FALSE)
//         {
//             dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                     L"System\\CurrentControlSet\\Services",
//                                     0,
//                                     DELETE,
//                                     &hServicesKey) as DWORD;
//             if (dwError == ERROR_SUCCESS)
//             {
//                 dwError = ScmDeleteRegKey(hServicesKey, CurrentService->lpServiceName);
//                 RegCloseKey(hServicesKey);
//                 if (dwError == ERROR_SUCCESS)
//                 {
//                     RemoveEntryList(&CurrentService->ServiceListEntry);
//                     HeapFree(GetProcessHeap(), 0, CurrentService);
//                 }
//             }
//
//             if (dwError != ERROR_SUCCESS)
//                 DPRINT1("Delete service failed: %S\n", CurrentService->lpServiceName);
//         }
//     }
// }
//
//
// static
// VOID
// ScmGetNoInteractiveServicesValue(VOID)
// {
//     HKEY hKey;
//     DWORD dwKeySize;
//     LONG lError;
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"SYSTEM\\CurrentControlSet\\Control\\Windows",
//                            0,
//                            KEY_READ,
//                            &hKey);
//     if (lError == ERROR_SUCCESS)
//     {
//         dwKeySize = sizeof(NoInteractiveServices);
//         lError = RegQueryValueExW(hKey,
//                                   L"NoInteractiveServices",
//                                   0,
//                                   NULL,
//                                   (LPBYTE)&NoInteractiveServices,
//                                   &dwKeySize);
//         RegCloseKey(hKey);
//     }
// }
//
//
// DWORD
// ScmCreateServiceDatabase(VOID)
// {
//     WCHAR szSubKey[MAX_PATH];
//     HKEY hServicesKey;
//     HKEY hServiceKey;
//     DWORD dwSubKey;
//     DWORD dwSubKeyLength;
//     FILETIME ftLastChanged;
//     DWORD dwError;
//
//     DPRINT("ScmCreateServiceDatabase() called\n");
//
//     /* Retrieve the NoInteractiveServies value */
//     ScmGetNoInteractiveServicesValue();
//
//     /* Create the service group list */
//     dwError = ScmCreateGroupList();
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     /* Initialize image and service lists */
//     InitializeListHead(&ImageListHead);
//     InitializeListHead(&ServiceListHead);
//
//     /* Initialize the database lock */
//     RtlInitializeResource(&DatabaseLock);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_READ,
//                             &hServicesKey) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     dwSubKey = 0;
//     for (;;)
//     {
//         dwSubKeyLength = MAX_PATH;
//         dwError = RegEnumKeyExW(hServicesKey,
//                                 dwSubKey,
//                                 szSubKey,
//                                 &dwSubKeyLength,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 &ftLastChanged) as DWORD;
//         if (dwError == ERROR_SUCCESS &&
//             szSubKey[0] != L'{')
//         {
//             DPRINT("SubKeyName: '%S'\n", szSubKey);
//
//             dwError = RegOpenKeyExW(hServicesKey,
//                                     szSubKey,
//                                     0,
//                                     KEY_READ,
//                                     &hServiceKey) as DWORD;
//             if (dwError == ERROR_SUCCESS)
//             {
//                 dwError = CreateServiceListEntry(szSubKey,
//                                                  hServiceKey);
//
//                 RegCloseKey(hServiceKey);
//             }
//         }
//
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         dwSubKey++;
//     }
//
//     RegCloseKey(hServicesKey);
//
//     /* Wait for the LSA server */
//     ScmWaitForLsa();
//
//     /* Delete services that are marked for delete */
//     ScmDeleteMarkedServices();
//
//     DPRINT("ScmCreateServiceDatabase() done\n");
//
//     return ERROR_SUCCESS;
// }
//
//
// VOID
// ScmShutdownServiceDatabase(VOID)
// {
//     DPRINT("ScmShutdownServiceDatabase() called\n");
//
//     ScmDeleteMarkedServices();
//     RtlDeleteResource(&DatabaseLock);
//
//     DPRINT("ScmShutdownServiceDatabase() done\n");
// }
//
//
// static NTSTATUS
// ScmCheckDriver(PSERVICE Service)
// {
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING DirName;
//     HANDLE DirHandle;
//     NTSTATUS Status;
//     POBJECT_DIRECTORY_INFORMATION DirInfo;
//     ULONG BufferLength;
//     ULONG DataLength;
//     ULONG Index;
//
//     DPRINT("ScmCheckDriver(%S) called\n", Service->lpServiceName);
//
//     if (Service->Status.dwServiceType == SERVICE_KERNEL_DRIVER)
//     {
//         RtlInitUnicodeString(&DirName, L"\\Driver");
//     }
//     else // if (Service->Status.dwServiceType == SERVICE_FILE_SYSTEM_DRIVER)
//     {
//         ASSERT(Service->Status.dwServiceType == SERVICE_FILE_SYSTEM_DRIVER);
//         RtlInitUnicodeString(&DirName, L"\\FileSystem");
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DirName,
//                                0,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenDirectoryObject(&DirHandle,
//                                    DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
//                                    &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     BufferLength = sizeof(OBJECT_DIRECTORY_INFORMATION) +
//                        2 * MAX_PATH * sizeof(WCHAR);
//     DirInfo = HeapAlloc(GetProcessHeap(),
//                         HEAP_ZERO_MEMORY,
//                         BufferLength);
//
//     Index = 0;
//     while (TRUE)
//     {
//         Status = NtQueryDirectoryObject(DirHandle,
//                                         DirInfo,
//                                         BufferLength,
//                                         TRUE,
//                                         FALSE,
//                                         &Index,
//                                         &DataLength);
//         if (Status == STATUS_NO_MORE_ENTRIES)
//         {
//             /* FIXME: Add current service to 'failed service' list */
//             DPRINT("Service '%S' failed\n", Service->lpServiceName);
//             break;
//         }
//
//         if (!NT_SUCCESS(Status))
//             break;
//
//         DPRINT("Comparing: '%S'  '%wZ'\n", Service->lpServiceName, &DirInfo->Name);
//
//         if (_wcsicmp(Service->lpServiceName, DirInfo->Name.Buffer) == 0)
//         {
//             DPRINT("Found: '%S'  '%wZ'\n",
//                    Service->lpServiceName, &DirInfo->Name);
//
//             /* Mark service as 'running' */
//             Service->Status.dwCurrentState = SERVICE_RUNNING;
//             Service->Status.dwControlsAccepted = SERVICE_ACCEPT_STOP;
//             Service->Status.dwWin32ExitCode = ERROR_SUCCESS;
//             Service->Status.dwServiceSpecificExitCode = 0;
//             Service->Status.dwCheckPoint = 0;
//             Service->Status.dwWaitHint = 0;
//
//             /* Mark the service group as 'running' */
//             if (Service->lpGroup != NULL)
//             {
//                 Service->lpGroup->ServicesRunning = TRUE;
//             }
//
//             break;
//         }
//     }
//
//     HeapFree(GetProcessHeap(),
//              0,
//              DirInfo);
//     NtClose(DirHandle);
//
//     return STATUS_SUCCESS;
// }
//
//
// VOID
// ScmGetBootAndSystemDriverState(VOID)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     DPRINT("ScmGetBootAndSystemDriverState() called\n");
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         if (CurrentService->dwStartType == SERVICE_BOOT_START ||
//             CurrentService->dwStartType == SERVICE_SYSTEM_START)
//         {
//             /* Check driver */
//             DPRINT("  Checking service: %S\n", CurrentService->lpServiceName);
//
//             ScmCheckDriver(CurrentService);
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DPRINT("ScmGetBootAndSystemDriverState() done\n");
// }
//
//
// /*
//  * ScmSendControlPacket must never be called with the database lock being held.
//  * The service passed must always be referenced instead.
//  */
// DWORD
// ScmSendControlPacket(
//      HANDLE hControlPipe,
//      PCWSTR pServiceName,
//      DWORD dwControl,
//      DWORD dwControlPacketSize,
//      PVOID pControlPacket)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     BOOL bResult;
//     SCM_REPLY_PACKET ReplyPacket;
//     DWORD dwReadCount = 0;
//     OVERLAPPED Overlapped = {0};
//
//     DPRINT("ScmSendControlPacket(%p, %lu, %p) called\n",
//            hControlPipe, dwControlPacketSize, pControlPacket);
//
//     /* Acquire the service control critical section, to synchronize requests */
//     EnterCriticalSection(&ControlServiceCriticalSection);
//
//     bResult = TransactNamedPipe(hControlPipe,
//                                 pControlPacket,
//                                 dwControlPacketSize,
//                                 &ReplyPacket,
//                                 sizeof(ReplyPacket),
//                                 &dwReadCount,
//                                 &Overlapped);
//     if (!bResult)
//     {
//         /* Fail for any error other than pending IO */
//         dwError = GetLastError();
//         if (dwError != ERROR_IO_PENDING)
//         {
//             DPRINT1("TransactNamedPipe(%S, %d) failed (Error %lu)\n", pServiceName, dwControl, dwError);
//             goto Done;
//         }
//
//         DPRINT("TransactNamedPipe(%S, %d) returned ERROR_IO_PENDING\n", pServiceName, dwControl);
//
//         dwError = WaitForSingleObject(hControlPipe, PipeTimeout);
//         DPRINT("WaitForSingleObject(%S, %d) returned %lu\n", pServiceName, dwControl, dwError);
//
//         if (dwError == WAIT_TIMEOUT)
//         {
//             DPRINT1("WaitForSingleObject(%S, %d) timed out\n", pServiceName, dwControl);
//             bResult = CancelIo(hControlPipe);
//             if (!bResult)
//                 DPRINT1("CancelIo(%S, %d) failed (Error %lu)\n", pServiceName, dwControl, GetLastError());
//
//             dwError = ERROR_SERVICE_REQUEST_TIMEOUT;
//         }
//         else if (dwError == WAIT_OBJECT_0)
//         {
//             bResult = GetOverlappedResult(hControlPipe,
//                                           &Overlapped,
//                                           &dwReadCount,
//                                           TRUE);
//             if (!bResult)
//             {
//                 dwError = GetLastError();
//                 DPRINT1("GetOverlappedResult(%S, %d) failed (Error %lu)\n", pServiceName, dwControl, dwError);
//             }
//         }
//     }
//
// Done:
//     /* Release the service control critical section */
//     LeaveCriticalSection(&ControlServiceCriticalSection);
//
//     if (dwReadCount == sizeof(ReplyPacket))
//         dwError = ReplyPacket.dwError;
//
//     return dwError;
// }
//
//
// DWORD
// ScmControlServiceEx(
//      HANDLE hControlPipe,
//      PCWSTR pServiceName,
//      DWORD dwControl,
//      SERVICE_STATUS_HANDLE hServiceStatus,
//      DWORD dwServiceTag,
//      DWORD argc,
//      const PCWSTR* argv)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSCM_CONTROL_PACKET ControlPacket;
//     DWORD PacketSize;
//     DWORD i;
//     PWSTR Ptr;
//
//     DPRINT("ScmControlServiceEx(%S, %d) called\n", pServiceName, dwControl);
//
//     /* Calculate the total size of the control packet:
//      * initial structure, the start command line, and the argument vector */
//     PacketSize = sizeof(SCM_CONTROL_PACKET);
//     PacketSize += (DWORD)((wcslen(pServiceName) + 1) * sizeof(WCHAR));
//
//     /*
//      * Calculate the required packet size for the start argument vector 'argv',
//      * composed of the pointer offsets list, followed by UNICODE strings.
//      * The strings are stored successively after the offsets vector, with
//      * the offsets being relative to the beginning of the vector, as in the
//      * following layout (with N == argc):
//      *     [argOff(0)]...[argOff(N-1)][str(0)]...[str(N-1)] .
//      */
//     if (argc > 0 && argv != NULL)
//     {
//         PacketSize = ALIGN_UP(PacketSize, PWSTR);
//         PacketSize += (argc * sizeof(PWSTR));
//
//         DPRINT("Argc: %lu\n", argc);
//         for (i = 0; i < argc; i++)
//         {
//             DPRINT("Argv[%lu]: %S\n", i, argv[i]);
//             PacketSize += (DWORD)((wcslen(argv[i]) + 1) * sizeof(WCHAR));
//         }
//     }
//
//     /* Allocate the control packet */
//     ControlPacket = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize);
//     if (!ControlPacket)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     ControlPacket->dwSize = PacketSize;
//     ControlPacket->dwControl = dwControl;
//     ControlPacket->hServiceStatus = hServiceStatus;
//     ControlPacket->dwServiceTag = dwServiceTag;
//
//     /* Copy the start command line */
//     ControlPacket->dwServiceNameOffset = sizeof(SCM_CONTROL_PACKET);
//     Ptr = (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwServiceNameOffset);
//     wcscpy(Ptr, pServiceName);
//
//     ControlPacket->dwArgumentsCount  = 0;
//     ControlPacket->dwArgumentsOffset = 0;
//
//     /* Copy the argument vector */
//     if (argc > 0 && argv != NULL)
//     {
//         PWSTR *pOffPtr, pArgPtr;
//
//         Ptr += wcslen(pServiceName) + 1;
//         pOffPtr = (PWSTR*)ALIGN_UP_POINTER(Ptr, PWSTR);
//         pArgPtr = (PWSTR)((ULONG_PTR)pOffPtr + argc * sizeof(PWSTR));
//
//         ControlPacket->dwArgumentsCount  = argc;
//         ControlPacket->dwArgumentsOffset = (DWORD)((ULONG_PTR)pOffPtr - (ULONG_PTR)ControlPacket);
//
//         DPRINT("dwArgumentsCount: %lu\n", ControlPacket->dwArgumentsCount);
//         DPRINT("dwArgumentsOffset: %lu\n", ControlPacket->dwArgumentsOffset);
//
//         for (i = 0; i < argc; i++)
//         {
//             wcscpy(pArgPtr, argv[i]);
//             pOffPtr[i] = (PWSTR)((ULONG_PTR)pArgPtr - (ULONG_PTR)pOffPtr);
//             DPRINT("offset[%lu]: %p\n", i, pOffPtr[i]);
//             pArgPtr += wcslen(argv[i]) + 1;
//         }
//     }
//
//     dwError = ScmSendControlPacket(hControlPipe,
//                                    pServiceName,
//                                    dwControl,
//                                    PacketSize,
//                                    ControlPacket);
//
//     /* Free the control packet */
//     HeapFree(GetProcessHeap(), 0, ControlPacket);
//
//     DPRINT("ScmControlServiceEx(%S, %d) done (Error %lu)\n", pServiceName, dwControl, dwError);
//     return dwError;
// }
//
//
// DWORD
// ScmControlService(
//      HANDLE hControlPipe,
//      PCWSTR pServiceName,
//      DWORD dwControl,
//      SERVICE_STATUS_HANDLE hServiceStatus)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSCM_CONTROL_PACKET ControlPacket;
//     DWORD PacketSize;
//     PWSTR Ptr;
//
//     DPRINT("ScmControlService(%S, %d) called\n", pServiceName, dwControl);
//
//     /* Calculate the total size of the control packet:
//      * initial structure, the start command line, and the argument vector */
//     PacketSize = sizeof(SCM_CONTROL_PACKET);
//     PacketSize += (DWORD)((wcslen(pServiceName) + 1) * sizeof(WCHAR));
//
//     /* Allocate the control packet */
//     ControlPacket = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize);
//     if (!ControlPacket)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     ControlPacket->dwSize = PacketSize;
//     ControlPacket->dwControl = dwControl;
//     ControlPacket->hServiceStatus = hServiceStatus;
//
//     /* Copy the service name */
//     ControlPacket->dwServiceNameOffset = sizeof(SCM_CONTROL_PACKET);
//     Ptr = (PWSTR)((ULONG_PTR)ControlPacket + ControlPacket->dwServiceNameOffset);
//     wcscpy(Ptr, pServiceName);
//
//     /* Send the control packet */
//     dwError = ScmSendControlPacket(hControlPipe,
//                                    pServiceName,
//                                    dwControl,
//                                    PacketSize,
//                                    ControlPacket);
//
//     /* Free the control packet */
//     HeapFree(GetProcessHeap(), 0, ControlPacket);
//
//     DPRINT("ScmControlService(%S, %d) done (Error %lu)\n", pServiceName, dwControl, dwError);
//     return dwError;
// }
//
//
// static DWORD
// ScmWaitForServiceConnect(PSERVICE Service)
// {
//     DWORD dwRead = 0;
//     DWORD dwProcessId = 0;
//     DWORD dwError = ERROR_SUCCESS;
//     BOOL bResult;
//     OVERLAPPED Overlapped = {0};
//
//     DPRINT("ScmWaitForServiceConnect()\n");
//
//     bResult = ConnectNamedPipe(Service->lpImage->hControlPipe,
//                                &Overlapped);
//     if (bResult == FALSE)
//     {
//         DPRINT("ConnectNamedPipe() returned FALSE\n");
//
//         dwError = GetLastError();
//         if (dwError == ERROR_IO_PENDING)
//         {
//             DPRINT("dwError: ERROR_IO_PENDING\n");
//
//             dwError = WaitForSingleObject(Service->lpImage->hControlPipe,
//                                           PipeTimeout);
//             DPRINT("WaitForSingleObject() returned %lu\n", dwError);
//
//             if (dwError == WAIT_TIMEOUT)
//             {
//                 DPRINT("WaitForSingleObject() returned WAIT_TIMEOUT\n");
//
//                 bResult = CancelIo(Service->lpImage->hControlPipe);
//                 if (bResult == FALSE)
//                 {
//                     DPRINT1("CancelIo() failed (Error: %lu)\n", GetLastError());
//                 }
//
//                 DPRINT1("Log EVENT_CONNECTION_TIMEOUT by %S\n", Service->lpDisplayName);
//
//                 return ERROR_SERVICE_REQUEST_TIMEOUT;
//             }
//             else if (dwError == WAIT_OBJECT_0)
//             {
//                 bResult = GetOverlappedResult(Service->lpImage->hControlPipe,
//                                               &Overlapped,
//                                               &dwRead,
//                                               TRUE);
//                 if (bResult == FALSE)
//                 {
//                     dwError = GetLastError();
//                     DPRINT1("GetOverlappedResult failed (Error %lu)\n", dwError);
//
//                     return dwError;
//                 }
//             }
//         }
//         else if (dwError != ERROR_PIPE_CONNECTED)
//         {
//             DPRINT1("ConnectNamedPipe failed (Error %lu)\n", dwError);
//             return dwError;
//         }
//     }
//
//     DPRINT("Control pipe connected\n");
//
//     Overlapped.hEvent = NULL;
//
//     /* Read the process id from pipe */
//     bResult = ReadFile(Service->lpImage->hControlPipe,
//                        (LPVOID)&dwProcessId,
//                        sizeof(DWORD),
//                        &dwRead,
//                        &Overlapped);
//     if (bResult == FALSE)
//     {
//         DPRINT("ReadFile() returned FALSE\n");
//
//         dwError = GetLastError();
//         if (dwError == ERROR_IO_PENDING)
//         {
//             DPRINT("dwError: ERROR_IO_PENDING\n");
//
//             dwError = WaitForSingleObject(Service->lpImage->hControlPipe,
//                                           PipeTimeout);
//             if (dwError == WAIT_TIMEOUT)
//             {
//                 DPRINT("WaitForSingleObject() returned WAIT_TIMEOUT\n");
//
//                 bResult = CancelIo(Service->lpImage->hControlPipe);
//                 if (bResult == FALSE)
//                 {
//                     DPRINT1("CancelIo() failed (Error: %lu)\n", GetLastError());
//                 }
//
//                 DPRINT1("Log EVENT_READFILE_TIMEOUT by %S\n", Service->lpDisplayName);
//
//                 return ERROR_SERVICE_REQUEST_TIMEOUT;
//             }
//             else if (dwError == WAIT_OBJECT_0)
//             {
//                 DPRINT("WaitForSingleObject() returned WAIT_OBJECT_0\n");
//
//                 DPRINT("Process Id: %lu\n", dwProcessId);
//
//                 bResult = GetOverlappedResult(Service->lpImage->hControlPipe,
//                                               &Overlapped,
//                                               &dwRead,
//                                               TRUE);
//                 if (bResult == FALSE)
//                 {
//                     dwError = GetLastError();
//                     DPRINT1("GetOverlappedResult() failed (Error %lu)\n", dwError);
//
//                     return dwError;
//                 }
//             }
//             else
//             {
//                 DPRINT1("WaitForSingleObject() returned %lu\n", dwError);
//             }
//         }
//         else
//         {
//             DPRINT1("ReadFile() failed (Error %lu)\n", dwError);
//             return dwError;
//         }
//     }
//
//     if ((ScmIsSecurityService(Service->lpImage) == FALSE)&&
//         (dwProcessId != Service->lpImage->dwProcessId))
//     {
//
//         DPRINT1("Log EVENT_SERVICE_DIFFERENT_PID_CONNECTED by %S\n", Service->lpDisplayName);
//     }
//
//     DPRINT("ScmWaitForServiceConnect() done\n");
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScmStartUserModeService(PSERVICE Service,
//                         DWORD argc,
//                         const PCWSTR* argv)
// {
//     PROCESS_INFORMATION ProcessInformation;
//     STARTUPINFOW StartupInfo;
//     LPVOID lpEnvironment;
//     BOOL Result;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("ScmStartUserModeService(%p)\n", Service);
//
//     /* If the image is already running, just send a start command */
//     if (Service->lpImage->dwImageRunCount > 1)
//         goto Quit;
//
//     /* Otherwise start its process */
//     ZeroMemory(&StartupInfo, sizeof(StartupInfo));
//     StartupInfo.cb = sizeof(StartupInfo);
//     ZeroMemory(&ProcessInformation, sizeof(ProcessInformation));
//
//     if (Service->lpImage->hToken)
//     {
//         /* User token: Run the service under the user account */
//
//         if (!CreateEnvironmentBlock(&lpEnvironment, Service->lpImage->hToken, FALSE))
//         {
//             /* We failed, run the service with the current environment */
//             DPRINT1("CreateEnvironmentBlock() failed with error %d; service '%S' will run with current environment\n",
//                     GetLastError(), Service->lpServiceName);
//             lpEnvironment = NULL;
//         }
//
//         /* Impersonate the new user */
//         Result = ImpersonateLoggedOnUser(Service->lpImage->hToken);
//         if (Result)
//         {
//             /* Launch the process in the user's logon session */
//             Result = CreateProcessAsUserW(Service->lpImage->hToken,
//                                           NULL,
//                                           Service->lpImage->pszImagePath,
//                                           NULL,
//                                           NULL,
//                                           FALSE,
//                                           CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS | CREATE_SUSPENDED,
//                                           lpEnvironment,
//                                           NULL,
//                                           &StartupInfo,
//                                           &ProcessInformation);
//             if (!Result)
//                 dwError = GetLastError();
//
//             /* Revert the impersonation */
//             RevertToSelf();
//         }
//         else
//         {
//             dwError = GetLastError();
//             DPRINT1("ImpersonateLoggedOnUser() failed with error %d\n", dwError);
//         }
//     }
//     else
//     {
//         /* No user token: Run the service under the LocalSystem account */
//
//         if (!CreateEnvironmentBlock(&lpEnvironment, NULL, TRUE))
//         {
//             /* We failed, run the service with the current environment */
//             DPRINT1("CreateEnvironmentBlock() failed with error %d; service '%S' will run with current environment\n",
//                     GetLastError(), Service->lpServiceName);
//             lpEnvironment = NULL;
//         }
//
//         /* Use the interactive desktop if the service is interactive */
//         if ((NoInteractiveServices == 0) &&
//             (Service->Status.dwServiceType & SERVICE_INTERACTIVE_PROCESS))
//         {
//             StartupInfo.dwFlags |= STARTF_INHERITDESKTOP;
//             StartupInfo.lpDesktop = L"WinSta0\\Default";
//         }
//
//         if (!ScmIsSecurityService(Service->lpImage))
//         {
//             Result = CreateProcessW(NULL,
//                                     Service->lpImage->pszImagePath,
//                                     NULL,
//                                     NULL,
//                                     FALSE,
//                                     CREATE_UNICODE_ENVIRONMENT | DETACHED_PROCESS | CREATE_SUSPENDED,
//                                     lpEnvironment,
//                                     NULL,
//                                     &StartupInfo,
//                                     &ProcessInformation);
//             if (!Result)
//                 dwError = GetLastError();
//         }
//         else
//         {
//             Result = TRUE;
//             dwError = ERROR_SUCCESS;
//         }
//     }
//
//     if (lpEnvironment)
//         DestroyEnvironmentBlock(lpEnvironment);
//
//     if (!Result)
//     {
//         DPRINT1("Starting '%S' failed with error %d\n",
//                 Service->lpServiceName, dwError);
//         return dwError;
//     }
//
//     DPRINT("Process Id: %lu  Handle %p\n",
//            ProcessInformation.dwProcessId,
//            ProcessInformation.hProcess);
//     DPRINT("Thread Id: %lu  Handle %p\n",
//            ProcessInformation.dwThreadId,
//            ProcessInformation.hThread);
//
//     /* Get the process handle and ID */
//     Service->lpImage->hProcess = ProcessInformation.hProcess;
//     Service->lpImage->dwProcessId = ProcessInformation.dwProcessId;
//
//     /* Resume the main thread and close its handle */
//     ResumeThread(ProcessInformation.hThread);
//     CloseHandle(ProcessInformation.hThread);
//
//     /* Connect control pipe */
//     dwError = ScmWaitForServiceConnect(Service);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("Connecting control pipe failed! (Error %lu)\n", dwError);
//         Service->lpImage->dwProcessId = 0;
//         return dwError;
//     }
//
// Quit:
//     /* Send the start command and return */
//     return ScmControlServiceEx(Service->lpImage->hControlPipe,
//                                Service->lpServiceName,
//                                (Service->Status.dwServiceType & SERVICE_WIN32_OWN_PROCESS)
//                                   ? SERVICE_CONTROL_START_OWN : SERVICE_CONTROL_START_SHARE,
//                                (SERVICE_STATUS_HANDLE)Service,
//                                Service->dwServiceTag,
//                                argc, argv);
// }
//
//
// static DWORD
// ScmLoadService(PSERVICE Service,
//                DWORD argc,
//                const PCWSTR* argv)
// {
//     PSERVICE_GROUP Group = Service->lpGroup;
//     DWORD dwError = ERROR_SUCCESS;
//     LPCWSTR lpLogStrings[2];
//     WCHAR szLogBuffer[80];
//
//     DbgPrint("[SERVICES] ScmLoadService(%S) type=%lu\n", Service->lpServiceName, Service->Status.dwServiceType);
//
//     if (Service->Status.dwCurrentState != SERVICE_STOPPED)
//     {
//         return ERROR_SERVICE_ALREADY_RUNNING;
//     }
//
//     if (Service->Status.dwServiceType & SERVICE_DRIVER)
//     {
//         /* Start the driver */
//         dwError = ScmStartDriver(Service);
//     }
//     else // if (Service->Status.dwServiceType & (SERVICE_WIN32 | SERVICE_INTERACTIVE_PROCESS))
//     {
//         /* Start user-mode service */
//         DbgPrint("[SERVICES] CreateOrReferenceServiceImage(%S)\n", Service->lpServiceName);
//         dwError = ScmCreateOrReferenceServiceImage(Service);
//         DbgPrint("[SERVICES] CreateOrReferenceServiceImage(%S) = %lu\n", Service->lpServiceName, dwError);
//         if (dwError == ERROR_SUCCESS)
//         {
//             DbgPrint("[SERVICES] ScmStartUserModeService(%S)\n", Service->lpServiceName);
//             dwError = ScmStartUserModeService(Service, argc, argv);
//             if (dwError == ERROR_SUCCESS)
//             {
//                 Service->Status.dwCurrentState = SERVICE_START_PENDING;
//                 Service->Status.dwControlsAccepted = 0;
//                 ScmReferenceService(Service);
//             }
//             else
//             {
//                 Service->lpImage->dwImageRunCount--;
//                 if (Service->lpImage->dwImageRunCount == 0)
//                 {
//                     ScmRemoveServiceImage(Service->lpImage);
//                     Service->lpImage = NULL;
//                 }
//             }
//         }
//     }
//
//     DbgPrint("[SERVICES] ScmLoadService(%S) done: %lu\n", Service->lpServiceName, dwError);
//
//     if (dwError == ERROR_SUCCESS)
//     {
//         if (Group != NULL)
//         {
//             Group->ServicesRunning = TRUE;
//         }
//
//         DbgPrint("[SERVICES] ScmLoadService(%S) logging success event\n", Service->lpServiceName);
//
//         /* Log a successful service start */
//         LoadStringW(GetModuleHandleW(NULL), IDS_SERVICE_START, szLogBuffer, 80);
//         DbgPrint("[SERVICES] ScmLoadService(%S) LoadStringW done\n", Service->lpServiceName);
//         lpLogStrings[0] = Service->lpDisplayName;
//         lpLogStrings[1] = szLogBuffer;
//
//         ScmLogEvent(EVENT_SERVICE_CONTROL_SUCCESS,
//                     EVENTLOG_INFORMATION_TYPE,
//                     2,
//                     lpLogStrings);
//         DbgPrint("[SERVICES] ScmLoadService(%S) event logged OK\n", Service->lpServiceName);
//     }
//     else
//     {
//         if (Service->dwErrorControl != SERVICE_ERROR_IGNORE)
//         {
//             /* Log a failed service start */
//             StringCchPrintfW(szLogBuffer, ARRAYSIZE(szLogBuffer),
//                              L"%lu", dwError);
//             lpLogStrings[0] = Service->lpServiceName;
//             lpLogStrings[1] = szLogBuffer;
//             ScmLogEvent(EVENT_SERVICE_START_FAILED,
//                         EVENTLOG_ERROR_TYPE,
//                         2,
//                         lpLogStrings);
//         }
//
//     }
//
//     return dwError;
// }
//
//
// DWORD
// ScmStartService(PSERVICE Service,
//                 DWORD argc,
//                 const PCWSTR* argv)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     SC_RPC_LOCK Lock = NULL;
//
//     DPRINT("ScmStartService() called\n");
//     DPRINT("Start Service %p (%S)\n", Service, Service->lpServiceName);
//
//     /* Acquire the service control critical section, to synchronize starts */
//     EnterCriticalSection(&ControlServiceCriticalSection);
//
//     /*
//      * Acquire the user service start lock while the service is starting, if
//      * needed (i.e. if we are not starting it during the initialization phase).
//      * If we don't success, bail out.
//      */
//     if (!ScmInitialize)
//     {
//         dwError = ScmAcquireServiceStartLock(TRUE, &Lock);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Really start the service */
//     dwError = ScmLoadService(Service, argc, argv);
//
//     /* Release the service start lock, if needed, and the critical section */
//     if (Lock) ScmReleaseServiceStartLock(&Lock);
//
// done:
//     LeaveCriticalSection(&ControlServiceCriticalSection);
//
//     DPRINT("ScmStartService() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// VOID
// ScmAutoStartServices(VOID)
// {
//     DWORD dwError;
//     PLIST_ENTRY GroupEntry;
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE_GROUP CurrentGroup;
//     PSERVICE CurrentService;
//     WCHAR szSafeBootServicePath[MAX_PATH];
//     DWORD SafeBootEnabled;
//     HKEY hKey;
//     DWORD dwKeySize;
//     ULONG i;
//
//     /*
//      * This function MUST be called ONLY at initialization time.
//      * Therefore, no need to acquire the user service start lock.
//      */
//     ASSERT(ScmInitialize);
//
//     /* Retrieve the SafeBoot parameter */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Control\\SafeBoot\\Option",
//                             0,
//                             KEY_READ,
//                             &hKey) as DWORD;
//     if (dwError == ERROR_SUCCESS)
//     {
//         dwKeySize = sizeof(SafeBootEnabled);
//         dwError = RegQueryValueExW(hKey,
//                                    L"OptionValue",
//                                    0,
//                                    NULL,
//                                    (LPBYTE)&SafeBootEnabled,
//                                    &dwKeySize) as DWORD;
//         RegCloseKey(hKey);
//     }
//
//     /* Default to Normal boot if the value doesn't exist */
//     if (dwError != ERROR_SUCCESS)
//         SafeBootEnabled = 0;
//
//     DbgPrint("[SERVICES] ScmAutoStartServices entered, SafeBoot=%lu\n", SafeBootEnabled);
//
//     /* Acquire the service control critical section, to synchronize starts */
//     EnterCriticalSection(&ControlServiceCriticalSection);
//
//     /* Clear 'ServiceVisited' flag (or set if not to start in Safe Mode) */
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         /* Build the safe boot path */
//         StringCchCopyW(szSafeBootServicePath, ARRAYSIZE(szSafeBootServicePath),
//                        L"SYSTEM\\CurrentControlSet\\Control\\SafeBoot");
//
//         switch (SafeBootEnabled)
//         {
//             /* NOTE: Assumes MINIMAL (1) and DSREPAIR (3) load same items */
//             case 1:
//             case 3:
//                 StringCchCatW(szSafeBootServicePath, ARRAYSIZE(szSafeBootServicePath),
//                               L"\\Minimal\\");
//                 break;
//
//             case 2:
//                 StringCchCatW(szSafeBootServicePath, ARRAYSIZE(szSafeBootServicePath),
//                               L"\\Network\\");
//                 break;
//         }
//
//         if (SafeBootEnabled != 0)
//         {
//             /* If key does not exist then do not assume safe mode */
//             dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                     szSafeBootServicePath,
//                                     0,
//                                     KEY_READ,
//                                     &hKey) as DWORD;
//             if (dwError == ERROR_SUCCESS)
//             {
//                 RegCloseKey(hKey);
//
//                 /* Finish Safe Boot path off */
//                 StringCchCatW(szSafeBootServicePath, ARRAYSIZE(szSafeBootServicePath),
//                               CurrentService->lpServiceName);
//
//                 /* Check that the key is in the Safe Boot path */
//                 dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                         szSafeBootServicePath,
//                                         0,
//                                         KEY_READ,
//                                         &hKey) as DWORD;
//                 if (dwError != ERROR_SUCCESS)
//                 {
//                     /* Mark service as visited so it is not auto-started */
//                     CurrentService->ServiceVisited = TRUE;
//                 }
//                 else
//                 {
//                     /* Must be auto-started in safe mode - mark as unvisited */
//                     RegCloseKey(hKey);
//                     CurrentService->ServiceVisited = FALSE;
//                 }
//             }
//             else
//             {
//                 DPRINT1("WARNING: Could not open the associated Safe Boot key\n");
//                 CurrentService->ServiceVisited = FALSE;
//             }
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DbgPrint("[SERVICES] Starting grouped services\n");
//
//     /* Start all services which are members of an existing group */
//     GroupEntry = GroupListHead.Flink;
//     while (GroupEntry != &GroupListHead)
//     {
//         CurrentGroup = CONTAINING_RECORD(GroupEntry, SERVICE_GROUP, GroupListEntry);
//
//         /* Start all services witch have a valid tag */
//         for (i = 0; i < CurrentGroup->TagCount; i++)
//         {
//             ServiceEntry = ServiceListHead.Flink;
//             while (ServiceEntry != &ServiceListHead)
//             {
//                 CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//                 if ((CurrentService->lpGroup == CurrentGroup) &&
//                     (CurrentService->dwStartType == SERVICE_AUTO_START) &&
//                     (CurrentService->ServiceVisited == FALSE) &&
//                     (CurrentService->dwTag == CurrentGroup->TagArray[i]))
//                 {
//                     CurrentService->ServiceVisited = TRUE;
//                     DbgPrint("[SERVICES] Loading(tag): %S\n", CurrentService->lpServiceName);
//                     ScmLoadService(CurrentService, 0, NULL);
//                 }
//
//                 ServiceEntry = ServiceEntry->Flink;
//              }
//         }
//
//         /* Start all services which have an invalid tag or which do not have a tag */
//         ServiceEntry = ServiceListHead.Flink;
//         while (ServiceEntry != &ServiceListHead)
//         {
//             CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//             if ((CurrentService->lpGroup == CurrentGroup) &&
//                 (CurrentService->dwStartType == SERVICE_AUTO_START) &&
//                 (CurrentService->ServiceVisited == FALSE))
//             {
//                 CurrentService->ServiceVisited = TRUE;
//                 DbgPrint("[SERVICES] Loading(grp): %S\n", CurrentService->lpServiceName);
//                 ScmLoadService(CurrentService, 0, NULL);
//             }
//
//             ServiceEntry = ServiceEntry->Flink;
//         }
//
//         GroupEntry = GroupEntry->Flink;
//     }
//
//     DbgPrint("[SERVICES] Starting non-existing-group services\n");
//
//     /* Start all services which are members of any non-existing group */
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         if ((CurrentService->lpGroup != NULL) &&
//             (CurrentService->dwStartType == SERVICE_AUTO_START) &&
//             (CurrentService->ServiceVisited == FALSE))
//         {
//             CurrentService->ServiceVisited = TRUE;
//             DbgPrint("[SERVICES] Loading(nogrp): %S\n", CurrentService->lpServiceName);
//             ScmLoadService(CurrentService, 0, NULL);
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DbgPrint("[SERVICES] Starting ungrouped services\n");
//
//     /* Start all services which are not a member of any group */
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         if ((CurrentService->lpGroup == NULL) &&
//             (CurrentService->dwStartType == SERVICE_AUTO_START) &&
//             (CurrentService->ServiceVisited == FALSE))
//         {
//             CurrentService->ServiceVisited = TRUE;
//             DbgPrint("[SERVICES] Loading(none): %S\n", CurrentService->lpServiceName);
//             ScmLoadService(CurrentService, 0, NULL);
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     /* Clear 'ServiceVisited' flag again */
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//         CurrentService->ServiceVisited = FALSE;
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     DbgPrint("[SERVICES] ScmAutoStartServices completed\n");
//
//     /* Release the critical section */
//     LeaveCriticalSection(&ControlServiceCriticalSection);
// }
//
//
// VOID
// ScmAutoShutdownServices(VOID)
// {
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     DPRINT("ScmAutoShutdownServices() called\n");
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     ServiceEntry = ServiceListHead.Flink;
//     while (ServiceEntry != &ServiceListHead)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//
//         if ((CurrentService->Status.dwControlsAccepted & SERVICE_ACCEPT_SHUTDOWN) &&
//             (CurrentService->Status.dwCurrentState == SERVICE_RUNNING ||
//              CurrentService->Status.dwCurrentState == SERVICE_START_PENDING))
//         {
//             /* Send the shutdown notification */
//             DPRINT("Shutdown service: %S\n", CurrentService->lpServiceName);
//             ScmControlService(CurrentService->lpImage->hControlPipe,
//                               CurrentService->lpServiceName,
//                               SERVICE_CONTROL_SHUTDOWN,
//                               (SERVICE_STATUS_HANDLE)CurrentService);
//         }
//
//         ServiceEntry = ServiceEntry->Flink;
//     }
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("ScmAutoShutdownServices() done\n");
// }
//
//
// BOOL
// ScmLockDatabaseExclusive(VOID)
// {
//     return RtlAcquireResourceExclusive(&DatabaseLock, TRUE);
// }
//
//
// BOOL
// ScmLockDatabaseShared(VOID)
// {
//     return RtlAcquireResourceShared(&DatabaseLock, TRUE);
// }
//
//
// VOID
// ScmUnlockDatabase(VOID)
// {
//     RtlReleaseResource(&DatabaseLock);
// }
//
//
// VOID
// ScmInitNamedPipeCriticalSection(VOID)
// {
//     HKEY hKey;
//     DWORD dwKeySize;
//     DWORD dwError;
//
//     InitializeCriticalSection(&ControlServiceCriticalSection);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"SYSTEM\\CurrentControlSet\\Control",
//                             0,
//                             KEY_READ,
//                             &hKey) as DWORD;
//    if (dwError == ERROR_SUCCESS)
//    {
//         dwKeySize = sizeof(PipeTimeout);
//         RegQueryValueExW(hKey,
//                          L"ServicesPipeTimeout",
//                          0,
//                          NULL,
//                          (LPBYTE)&PipeTimeout,
//                          &dwKeySize);
//        RegCloseKey(hKey);
//    }
// }
//
//
// VOID
// ScmDeleteNamedPipeCriticalSection(VOID)
// {
//     DeleteCriticalSection(&ControlServiceCriticalSection);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
