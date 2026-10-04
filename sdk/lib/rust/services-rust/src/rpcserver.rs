//! SCM svcctl RPC server — Rust flip of base/system/services/rpcserver.c
//! (SCM wave 8, the final SCM file). Translated by rustconv (Opus 4.8); the FFI
//! surface was retargeted by hand from the kernel ntkrnl-sys substrate to the
//! user-mode winsrv-sys substrate. The R* exports are the svcctl RPC interface
//! handlers (RPC = stdcall = extern "system" on i386).

#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use winsrv_sys::winsrv::*;
use winsrv_sys::nt::{STATUS_BUFFER_TOO_SMALL, STATUS_SUCCESS};
use crate::types::*;
use crate::svcctl::*;
use crate::wstr;
use core::ffi::c_void;

type wchar_t = u16;
// rpcdce.h
type RPC_STATUS = core::ffi::c_long;
const RPC_S_OK: RPC_STATUS = 0;

// resource.h string IDs.
const IDS_SERVICE_STOP: DWORD = 101;
const IDS_SERVICE_PAUSE: DWORD = 102;
const IDS_SERVICE_RESUME: DWORD = 103;
const IDS_SERVICE_RUNNING: DWORD = 104;
const IDS_SERVICE_STOPPED: DWORD = 105;
const IDS_SERVICE_PAUSED: DWORD = 106;
// services.mc eventlog message IDs.
const EVENT_SERVICE_CONTROL_SUCCESS: DWORD = 0x4000_1b7b;
const EVENT_SERVICE_STATUS_SUCCESS: DWORD = 0x4000_1b7c;
const EVENT_SERVICE_EXIT_FAILED: DWORD = 0xc000_1b6f;
// ntstatus.h
const STATUS_BAD_DESCRIPTOR_FORMAT: NTSTATUS = 0xC00000E7u32 as NTSTATUS;

// winsvc.h database names (L"ServicesActive" / L"ServicesFailed").
static SERVICES_ACTIVE_DATABASEW: [u16; 15] = [
    b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
    b'e' as u16, b's' as u16, b'A' as u16, b'c' as u16, b't' as u16, b'i' as u16,
    b'v' as u16, b'e' as u16, 0,
];
static SERVICES_FAILED_DATABASEW: [u16; 15] = [
    b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16,
    b'e' as u16, b's' as u16, b'F' as u16, b'a' as u16, b'i' as u16, b'l' as u16,
    b'e' as u16, b'd' as u16, 0,
];

// Globals defined elsewhere: ScmShutdown in services.c (still C); ServiceListHead
// in database.rs (#[no_mangle]).
extern "C" {
    static mut ScmShutdown: BOOL;
    static mut ServiceListHead: LIST_ENTRY;
}

#[inline]
fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// SecureZeroMemory — non-elidable zero (winbase macro; uses write_volatile).
#[inline]
unsafe fn SecureZeroMemory(ptr: *mut c_void, cnt: usize) -> *mut c_void {
    let p = ptr as *mut u8;
    let mut i = 0usize;
    while i < cnt {
        core::ptr::write_volatile(p.add(i), 0);
        i += 1;
    }
    ptr
}

// StringCchPrintfW(dst,cch,L"%lu",v) replacement — the one call site is a
// single %lu. Appends decimal `value` then NUL into dst[at..], bounded by cch.
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

// CRT (cdecl) — not in the winsrv-sys surface.
extern "C" {
    fn _wcsicmp(s1: LPCWSTR, s2: LPCWSTR) -> i32;
    fn _wcsnicmp(s1: *const wchar_t, s2: *const wchar_t, n: usize) -> i32;
    fn wcscat(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t;
    fn wcscpy(dst: *mut wchar_t, src: *const wchar_t) -> *mut wchar_t;
    fn wcslen(s: *const wchar_t) -> usize;
    fn wcsncpy(dst: *mut wchar_t, src: *const wchar_t, n: usize) -> *mut wchar_t;
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// Cross-file SCM exports (config/security/database/controlset are now Rust; all
// services.h-internal = cdecl).
extern "C" {
    fn ScmAcceptBoot() -> DWORD;
    fn ScmRunLastKnownGood() -> DWORD;
    fn ScmGetServiceEntryByDisplayName(lpDisplayName: LPCWSTR) -> PSERVICE;
    fn ScmGetServiceEntryByName(lpServiceName: LPCWSTR) -> PSERVICE;
    fn ScmGetServiceNameFromTag(
        InParams: PTAG_INFO_NAME_FROM_TAG_IN_PARAMS,
        OutParams: *mut PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS,
    ) -> DWORD;
    pub fn ScmLockDatabaseExclusive() -> BOOL;
    fn ScmUnlockDatabase();
    fn ScmOpenServiceKey(lpServiceName: LPWSTR, DesiredAccess: REGSAM, phKey: *mut HKEY) -> DWORD;
    fn ScmWriteSecurityDescriptor(hServiceKey: HKEY, pSecurityDescriptor: PSECURITY_DESCRIPTOR) -> DWORD;
    fn ScmDecryptPassword(
        hService: SC_RPC_HANDLE,
        lpPassword: LPBYTE,
        dwPwSize: DWORD,
        lpClearTextPassword: *mut LPWSTR,
    ) -> DWORD;
    fn ScmSetServiceGroup(lpService: PSERVICE, lpGroupName: LPCWSTR) -> DWORD;
    fn ScmSetServicePassword(pszServiceName: *const wchar_t, pszPassword: *const wchar_t) -> DWORD;
    fn ScmWriteDependencies(hServiceKey: HKEY, lpDependencies: LPCWSTR, dwDependenciesLength: DWORD) -> DWORD;
    fn ScmCreateDefaultServiceSD(ppSecurityDescriptor: *mut PSECURITY_DESCRIPTOR) -> DWORD;
    fn ScmCreateNewServiceRecord(lpServiceName: LPCWSTR, lppService: *mut PSERVICE, dwServiceType: DWORD, dwStartType: DWORD) -> DWORD;
    fn ScmCreateServiceKey(lpServiceName: LPCWSTR, samDesired: u32, phKey: *mut HKEY) -> DWORD;
    fn ScmGenerateServiceTag(lpServiceRecord: PSERVICE) -> u32;
    fn ScmReferenceService(lpService: PSERVICE) -> u32;
    fn ScmControlService(hControlPipe: *mut c_void, lpServiceName: LPCWSTR, dwControl: DWORD, hServiceStatus: *mut c_void) -> DWORD;
    fn ScmRemoveServiceImage(pServiceImage: *mut c_void);
    fn ScmLogEvent(dwEventId: DWORD, wType: u16, wStrings: u16, lpStrings: *mut LPCWSTR);
    fn ScmDereferenceService(lpService: *mut c_void) -> DWORD;
}

// Win32 + RPC runtime absent from winsrv-sys bindgen (WINAPI/RPC = stdcall i386).
extern "system" {
    fn ExpandEnvironmentStringsW(lpSrc: *const wchar_t, lpDst: *mut wchar_t, nSize: DWORD) -> DWORD;
    fn RpcServerUseProtseqEpW(
        Protseq: *mut u16,
        MaxCalls: u32,
        Endpoint: *mut u16,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
    fn RpcServerRegisterIf(
        IfSpec: RPC_IF_HANDLE,
        MgrTypeUuid: *mut c_void,
        MgrEpv: *mut c_void,
    ) -> RPC_STATUS;
    fn RpcServerListen(MinimumCallThreads: u32, MaxCalls: u32, DontWait: i32) -> RPC_STATUS;
    fn I_ScSetServiceBitsW(
        hServiceStatus: SC_RPC_HANDLE,
        dwServiceBits: DWORD,
        bSetBitsOn: i32,
        bUpdateImmediately: i32,
        lpString: *mut wchar_t,
    ) -> i32;
}

// svcctl RPC interface spec — generated into svcctl_s.c (still compiled).
extern "C" {
    static svcctl_v2_0_s_ifspec: RPC_IF_HANDLE;
}

// More cross-file SCM exports (database/config/lock/groupdb/driver = Rust; cdecl).
extern "C" {
    fn ScmReadString(hServiceKey: HKEY, lpValueName: LPCWSTR, lpValue: *mut LPWSTR) -> DWORD;
    fn ScmReadDependencies(hServiceKey: HKEY, lpDependencies: *mut LPWSTR, lpdwDependenciesLength: *mut DWORD) -> DWORD;
    fn ScmStartService(Service: PSERVICE, argc: DWORD, argv: *const PCWSTR) -> DWORD;
    fn ScmGetServiceEntryByResumeCount(dwResumeCount: DWORD) -> PSERVICE;
    fn ScmSendControlPacket(hControlPipe: HANDLE, pServiceName: PCWSTR, dwControl: DWORD, dwControlPacketSize: DWORD, pControlPacket: PVOID) -> DWORD;
    fn ScmAcquireServiceStartLock(IsServiceController: BOOL, lpLock: LPSC_RPC_LOCK) -> DWORD;
    fn ScmReleaseServiceStartLock(lpLock: LPSC_RPC_LOCK) -> DWORD;
    fn ScmQueryServiceLockStatusW(lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSW);
    fn ScmQueryServiceLockStatusA(lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSA);
    fn ScmMarkServiceForDelete(pService: PSERVICE) -> DWORD;
    fn ScmGetServiceGroupByName(lpGroupName: LPCWSTR) -> PSERVICE_GROUP;
    fn ScmControlDriver(lpService: PSERVICE, dwControl: DWORD, lpServiceStatus: LPSERVICE_STATUS) -> DWORD;
    // CRT mem (cdecl).
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn memmove(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// RtlCopyMemory(dst, src, len) — winbase macro (memcpy semantics).
#[inline]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// RtlInitEmptyUnicodeString — FORCEINLINE in ntrtl.h (no exported symbol).
#[inline]
unsafe fn RtlInitEmptyUnicodeString(dest: *mut UNICODE_STRING, buffer: PWSTR, len: u16) {
    (*dest).Length = 0;
    (*dest).MaximumLength = len;
    (*dest).Buffer = buffer;
}

// Shadow the glob HeapAlloc with a wrapper accepting either usize or u32 sizes
// (the translated body mixes both; SIZE_T is u32 on i386). Shadows the glob.
#[inline]
unsafe fn HeapAlloc<N: TryInto<SIZE_T>>(hHeap: HANDLE, dwFlags: DWORD, dwBytes: N) -> PVOID {
    let n: SIZE_T = match dwBytes.try_into() {
        Ok(v) => v,
        Err(_) => SIZE_T::MAX,
    };
    winsrv_sys::winsrv::HeapAlloc(hHeap, dwFlags, n)
}

use core::cmp::{max, min};

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


// ---- file-scope preamble (translated) ----
pub const MANAGER_TAG: DWORD = 0x72674D68;

pub const SERVICE_TAG: DWORD = 0x63765368;

pub const INVALID_TAG: DWORD = 0xAABBCCDD;

#[repr(C)]
pub struct _SCMGR_HANDLE {
    pub Tag: DWORD,
    pub DesiredAccess: DWORD,
}
pub type SCMGR_HANDLE = _SCMGR_HANDLE;

#[repr(C)]
pub struct _MANAGER_HANDLE {
    pub Handle: SCMGR_HANDLE,
    pub DatabaseName: [WCHAR; 1],
}
pub type MANAGER_HANDLE = _MANAGER_HANDLE;
pub type PMANAGER_HANDLE = *mut _MANAGER_HANDLE;

#[repr(C)]
pub struct _SERVICE_HANDLE {
    pub Handle: SCMGR_HANDLE,
    pub ServiceEntry: PSERVICE,
}
pub type SERVICE_HANDLE = _SERVICE_HANDLE;
pub type PSERVICE_HANDLE = *mut _SERVICE_HANDLE;

pub const SC_MANAGER_READ: DWORD =
    STANDARD_RIGHTS_READ | SC_MANAGER_QUERY_LOCK_STATUS | SC_MANAGER_ENUMERATE_SERVICE;

pub const SC_MANAGER_WRITE: DWORD =
    STANDARD_RIGHTS_WRITE | SC_MANAGER_MODIFY_BOOT_CONFIG | SC_MANAGER_CREATE_SERVICE;

pub const SC_MANAGER_EXECUTE: DWORD = STANDARD_RIGHTS_EXECUTE
    | SC_MANAGER_LOCK
    | SC_MANAGER_ENUMERATE_SERVICE
    | SC_MANAGER_CONNECT
    | SC_MANAGER_CREATE_SERVICE;

pub const SERVICE_READ: DWORD = STANDARD_RIGHTS_READ
    | SERVICE_INTERROGATE
    | SERVICE_ENUMERATE_DEPENDENTS
    | SERVICE_QUERY_STATUS
    | SERVICE_QUERY_CONFIG;

pub const SERVICE_WRITE: DWORD = STANDARD_RIGHTS_WRITE | SERVICE_CHANGE_CONFIG;

pub const SERVICE_EXECUTE: DWORD = STANDARD_RIGHTS_EXECUTE
    | SERVICE_USER_DEFINED_CONTROL
    | SERVICE_PAUSE_CONTINUE
    | SERVICE_STOP
    | SERVICE_START;

pub const TAG_ARRAY_SIZE: DWORD = 32;

pub static mut ScmManagerMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: SC_MANAGER_READ,
    GenericWrite: SC_MANAGER_WRITE,
    GenericExecute: SC_MANAGER_EXECUTE,
    GenericAll: SC_MANAGER_ALL_ACCESS,
};

pub static mut ScmServiceMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: SERVICE_READ,
    GenericWrite: SERVICE_WRITE,
    GenericExecute: SERVICE_EXECUTE,
    GenericAll: SERVICE_ALL_ACCESS,
};

pub static mut g_dwServiceBits: DWORD = 0;
#[no_mangle]
pub unsafe extern "C" fn ScmStartRpcServer() {
    let mut Status: RPC_STATUS;

    Status = RpcServerUseProtseqEpW(
        w!("ncacn_np").as_ptr() as *mut u16,
        10,
        w!("\\pipe\\ntsvcs").as_ptr() as *mut u16,
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK as RPC_STATUS {
        return;
    }

    Status = RpcServerRegisterIf(
        svcctl_v2_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK as RPC_STATUS {
        return;
    }

    Status = RpcServerListen(1, 20, 1u8 as i32);
    if Status != RPC_S_OK as RPC_STATUS {
        return;
    }
}
unsafe fn ScmCreateManagerHandle(mut lpDatabaseName: LPWSTR,
                                 Handle: *mut SC_HANDLE) -> DWORD {
    let Ptr: PMANAGER_HANDLE;

    if lpDatabaseName.is_null() {
        lpDatabaseName = SERVICES_ACTIVE_DATABASEW.as_ptr() as LPWSTR;
    }

    if _wcsicmp(lpDatabaseName, SERVICES_FAILED_DATABASEW.as_ptr() as LPWSTR) == 0 {
        return ERROR_DATABASE_DOES_NOT_EXIST;
    } else if _wcsicmp(lpDatabaseName, SERVICES_ACTIVE_DATABASEW.as_ptr() as LPWSTR) != 0 {
        return ERROR_INVALID_NAME;
    }

    let size = core::mem::size_of::<SCMGR_HANDLE>()
        + (wcslen(lpDatabaseName) as usize + 1) * core::mem::size_of::<u16>();

    Ptr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, size) as PMANAGER_HANDLE;
    if Ptr.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    (*Ptr).Handle.Tag = MANAGER_TAG;

    wcscpy(core::ptr::addr_of_mut!((*Ptr).DatabaseName) as LPWSTR, lpDatabaseName);

    *Handle = Ptr as SC_HANDLE;

    ERROR_SUCCESS
}

unsafe fn ScmCreateServiceHandle(lpServiceEntry: PSERVICE, Handle: *mut SC_HANDLE) -> DWORD {
    let Ptr: PSERVICE_HANDLE = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<SERVICE_HANDLE>()) as PSERVICE_HANDLE;
    if Ptr.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    (*Ptr).Handle.Tag = SERVICE_TAG;

    (*Ptr).ServiceEntry = lpServiceEntry;

    *Handle = Ptr as SC_HANDLE;

    ERROR_SUCCESS
}

unsafe fn ScmGetServiceManagerFromHandle(Handle: SC_RPC_HANDLE) -> PMANAGER_HANDLE {
    let mut pManager: PMANAGER_HANDLE = core::ptr::null_mut();

    if (*(Handle as PMANAGER_HANDLE)).Handle.Tag == MANAGER_TAG {
        pManager = Handle as PMANAGER_HANDLE;
    }

    pManager
}

unsafe fn ScmGetServiceFromHandle(Handle: SC_RPC_HANDLE) -> PSERVICE_HANDLE {
    let mut pService: PSERVICE_HANDLE = core::ptr::null_mut();

    if (*(Handle as PSERVICE_HANDLE)).Handle.Tag == SERVICE_TAG {
        pService = Handle as PSERVICE_HANDLE;
    }

    pService
}

unsafe fn ScmCheckAccess(Handle: SC_HANDLE, mut dwDesiredAccess: DWORD) -> DWORD {
    let hMgr = Handle as PMANAGER_HANDLE;

    if (*hMgr).Handle.Tag == MANAGER_TAG {
        RtlMapGenericMask(&mut dwDesiredAccess, core::ptr::addr_of_mut!(ScmManagerMapping));

        (*hMgr).Handle.DesiredAccess = dwDesiredAccess;

        return ERROR_SUCCESS;
    } else if (*hMgr).Handle.Tag == SERVICE_TAG {
        RtlMapGenericMask(&mut dwDesiredAccess, core::ptr::addr_of_mut!(ScmServiceMapping));

        (*hMgr).Handle.DesiredAccess = dwDesiredAccess;

        return ERROR_SUCCESS;
    }

    ERROR_INVALID_HANDLE
}
#[no_mangle]
pub unsafe extern "C" fn ScmAssignNewTag(lpService: PSERVICE) -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;
    let mut dwGroupTagCount: DWORD = 0;
    let mut pdwGroupTags: PDWORD = core::ptr::null_mut();
    let mut dwFreeTag: DWORD = 0;
    let mut dwTagUsedBase: DWORD = 1;
    let mut TagUsed: [BOOLEAN; TAG_ARRAY_SIZE as usize] = [0u8; TAG_ARRAY_SIZE as usize];
    let mut nTagOffset: INT;
    let mut i: DWORD;
    let mut cbDataSize: DWORD;
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;

    'cleanup: loop {
        'findFreeTag: loop {
            dwError = RegOpenKeyExW(
                HKEY_LOCAL_MACHINE,
                wstr!("System\\CurrentControlSet\\Control\\GroupOrderList").as_ptr(),
                0,
                KEY_READ,
                &mut hKey,
            ) as DWORD;

            if dwError != ERROR_SUCCESS {
                break 'findFreeTag;
            }

            cbDataSize = 0;
            dwError = RegQueryValueExW(
                hKey,
                (*(*lpService).lpGroup).szGroupName.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut cbDataSize,
            ) as DWORD;

            if dwError != ERROR_SUCCESS && dwError != ERROR_MORE_DATA {
                break 'findFreeTag;
            }

            pdwGroupTags = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbDataSize as usize) as PDWORD;
            if pdwGroupTags.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'cleanup;
            }

            dwError = RegQueryValueExW(
                hKey,
                (*(*lpService).lpGroup).szGroupName.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                pdwGroupTags as LPBYTE,
                &mut cbDataSize,
            ) as DWORD;

            if dwError != ERROR_SUCCESS {
                break 'findFreeTag;
            }

            if (cbDataSize as usize) < core::mem::size_of::<DWORD>() {
                break 'findFreeTag;
            }

            let count = *pdwGroupTags;
            let limit = cbDataSize / core::mem::size_of::<DWORD>() as DWORD - 1;
            dwGroupTagCount = if count < limit { count } else { limit };

            break 'findFreeTag;
        }

        loop {
            i = 0;
            while i < TAG_ARRAY_SIZE {
                TagUsed[i as usize] = 0u8;
                i += 1;
            }

            i = 1;
            while i <= dwGroupTagCount {
                nTagOffset = (*pdwGroupTags.add(i as usize)).wrapping_sub(dwTagUsedBase) as INT;
                if nTagOffset >= 0 && nTagOffset < TAG_ARRAY_SIZE as INT {
                    TagUsed[nTagOffset as usize] = 1u8;
                }
                i += 1;
            }

            ServiceEntry = (*lpService).ServiceListEntry.Flink;
            while ServiceEntry != core::ptr::addr_of_mut!((*lpService).ServiceListEntry) {
                CurrentService = (ServiceEntry as usize
                    - core::mem::offset_of!(SERVICE, ServiceListEntry))
                    as PSERVICE;
                if (*CurrentService).lpGroup == (*lpService).lpGroup {
                    nTagOffset = (*CurrentService).dwTag.wrapping_sub(dwTagUsedBase) as INT;
                    if nTagOffset >= 0 && nTagOffset < TAG_ARRAY_SIZE as INT {
                        TagUsed[nTagOffset as usize] = 1u8;
                    }
                }

                ServiceEntry = (*ServiceEntry).Flink;
            }

            i = 0;
            while i < TAG_ARRAY_SIZE {
                if TagUsed[i as usize] == 0u8 {
                    dwFreeTag = dwTagUsedBase + i;
                    break;
                }
                i += 1;
            }

            dwTagUsedBase += TAG_ARRAY_SIZE;

            if dwFreeTag != 0 {
                break;
            }
        }

        break 'cleanup;
    }

    if !pdwGroupTags.is_null() {
        HeapFree(GetProcessHeap(), 0, pdwGroupTags as *mut _);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if dwFreeTag != 0 {
        (*lpService).dwTag = dwFreeTag;
        dwError = ERROR_SUCCESS;
    }

    dwError
}
#[no_mangle]
pub unsafe extern "C" fn ScmConvertToBootPathName(CanonName: *mut wchar_t, RelativeName: *mut *mut wchar_t) -> DWORD {

    const ERROR_SUCCESS: DWORD = 0;
    const ERROR_INVALID_PARAMETER: DWORD = 87;
    const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
    const ERROR_INVALID_ENVIRONMENT: DWORD = 1805;
    const HEAP_ZERO_MEMORY: DWORD = 0x00000008;
    const UNICODE_NULL: wchar_t = 0;
    // ntbasedef.h: 65534. Was 65535 — this bounds-checks RPC input
    // (`BufferSize > UNICODE_STRING_MAX_BYTES - sizeof(UNICODE_NULL)`), so the
    // wrong value admitted a buffer one byte larger than NT accepts.
    const UNICODE_STRING_MAX_BYTES: DWORD = 65534;

    let ServiceNameLen: usize;
    let mut ExpandedLen: usize;
    let mut BufferSize: DWORD;
    let mut Dest: wchar_t = 0;
    let mut Expanded: *mut wchar_t;
    let mut NtPathName: _UNICODE_STRING = core::mem::zeroed();
    let mut SystemRoot: _UNICODE_STRING = core::mem::zeroed();
    let mut LinkTarget: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: i32;
    let mut SymbolicLinkHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    if RelativeName.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    *RelativeName = core::ptr::null_mut();

    ServiceNameLen = wcslen(CanonName);

    let sysroot12: [wchar_t; 13] = [0x5C, b'S' as wchar_t, b'y' as wchar_t, b's' as wchar_t, b't' as wchar_t, b'e' as wchar_t, b'm' as wchar_t, b'R' as wchar_t, b'o' as wchar_t, b'o' as wchar_t, b't' as wchar_t, 0x5C, 0];
    let pctsysroot13: [wchar_t; 14] = [b'%' as wchar_t, b'S' as wchar_t, b'y' as wchar_t, b's' as wchar_t, b't' as wchar_t, b'e' as wchar_t, b'm' as wchar_t, b'R' as wchar_t, b'o' as wchar_t, b'o' as wchar_t, b't' as wchar_t, b'%' as wchar_t, 0x5C, 0];
    let sysroot_full: [wchar_t; 12] = [0x5C, b'S' as wchar_t, b'y' as wchar_t, b's' as wchar_t, b't' as wchar_t, b'e' as wchar_t, b'm' as wchar_t, b'R' as wchar_t, b'o' as wchar_t, b'o' as wchar_t, b't' as wchar_t, 0];

    if ServiceNameLen > 12 && _wcsnicmp(sysroot12.as_ptr(), CanonName, 12) == 0 {
        *RelativeName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        if (*RelativeName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        wcscpy(*RelativeName, CanonName);
        return ERROR_SUCCESS;
    }

    if ServiceNameLen > 13 && _wcsnicmp(pctsysroot13.as_ptr(), CanonName, 13) == 0 {
        *RelativeName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ServiceNameLen * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        if (*RelativeName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        wcscpy(*RelativeName, sysroot12.as_ptr());
        wcscat(*RelativeName, CanonName.add(13));
        return ERROR_SUCCESS;
    }

    BufferSize = ExpandEnvironmentStringsW(pctsysroot13.as_ptr(), &mut Dest, 1);
    if BufferSize <= 1 {
        return ERROR_INVALID_ENVIRONMENT;
    }

    Expanded = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (BufferSize as usize + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if Expanded.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    if ExpandEnvironmentStringsW(pctsysroot13.as_ptr(), Expanded, BufferSize) > BufferSize {
        HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    if RtlDosPathNameToNtPathName_U(Expanded, &mut NtPathName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        return ERROR_INVALID_ENVIRONMENT;
    }

    HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);

    Expanded = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, NtPathName.Length as usize + core::mem::size_of::<wchar_t>()) as *mut wchar_t;
    if Expanded.is_null() {
        RtlFreeUnicodeString(&mut NtPathName);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    ExpandedLen = NtPathName.Length as usize / core::mem::size_of::<wchar_t>();
    wcsncpy(Expanded, NtPathName.Buffer, ExpandedLen);
    *Expanded.add(ExpandedLen) = UNICODE_NULL;
    RtlFreeUnicodeString(&mut NtPathName);

    if ServiceNameLen > ExpandedLen && _wcsnicmp(Expanded, CanonName, ExpandedLen) == 0 {
        HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);

        *RelativeName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen - ExpandedLen) * core::mem::size_of::<wchar_t>() + 13 * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
        if (*RelativeName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
        wcscpy(*RelativeName, sysroot12.as_ptr());
        wcscat(*RelativeName, CanonName.add(ExpandedLen));
        return ERROR_SUCCESS;
    }

    HeapFree(GetProcessHeap(), 0, Expanded as *mut core::ffi::c_void);

    RtlInitUnicodeString(&mut SystemRoot, sysroot_full.as_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut SystemRoot;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenSymbolicLinkObject(&mut SymbolicLinkHandle, SYMBOLIC_LINK_QUERY, &mut ObjectAttributes);
    if NT_SUCCESS(Status) {
        RtlInitEmptyUnicodeString(&mut LinkTarget, core::ptr::null_mut(), 0);
        Status = NtQuerySymbolicLinkObject(SymbolicLinkHandle, &mut LinkTarget, &mut BufferSize);
        if NT_SUCCESS(Status) || Status == STATUS_BUFFER_TOO_SMALL {
            if BufferSize > UNICODE_STRING_MAX_BYTES - core::mem::size_of::<wchar_t>() as u32 {
                NtClose(SymbolicLinkHandle);
                return ERROR_NOT_ENOUGH_MEMORY;
            }

            LinkTarget.Length = BufferSize as u16;
            LinkTarget.MaximumLength = LinkTarget.Length + core::mem::size_of::<wchar_t>() as u16;
            LinkTarget.Buffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, LinkTarget.MaximumLength as usize) as *mut wchar_t;
            if LinkTarget.Buffer.is_null() {
                NtClose(SymbolicLinkHandle);
                return ERROR_NOT_ENOUGH_MEMORY;
            }

            Status = NtQuerySymbolicLinkObject(SymbolicLinkHandle, &mut LinkTarget, &mut BufferSize);
            NtClose(SymbolicLinkHandle);
            if NT_SUCCESS(Status) {
                ExpandedLen = LinkTarget.Length as usize / core::mem::size_of::<wchar_t>();
                if ServiceNameLen > ExpandedLen && _wcsnicmp(LinkTarget.Buffer, CanonName, ExpandedLen) == 0 {
                    *RelativeName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen - ExpandedLen) * core::mem::size_of::<wchar_t>() + 13 * core::mem::size_of::<wchar_t>()) as *mut wchar_t;
                    if (*RelativeName).is_null() {
                        return ERROR_NOT_ENOUGH_MEMORY;
                    }
                    wcscpy(*RelativeName, sysroot12.as_ptr());
                    wcscat(*RelativeName, CanonName.add(ExpandedLen + 1));
                    return ERROR_SUCCESS;
                } else {
                    return ERROR_INVALID_PARAMETER;
                }
            } else {
                return ERROR_INVALID_PARAMETER;
            }
        } else {
            NtClose(SymbolicLinkHandle);
            return ERROR_INVALID_PARAMETER;
        }
    } else {
        return ERROR_INVALID_PARAMETER;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ScmCanonDriverImagePath(dwStartType: DWORD, lpServiceName: *const wchar_t, lpCanonName: *mut *mut wchar_t) -> DWORD {

    const ERROR_SUCCESS: DWORD = 0;
    const NO_ERROR: DWORD = 0;
    const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
    const ERROR_INVALID_PARAMETER: DWORD = 87;
    const HEAP_ZERO_MEMORY: DWORD = 0x00000008;
    const SERVICE_BOOT_START: DWORD = 0x00000000;

    let Result: DWORD;
    let ServiceNameLen: usize;
    let mut NtServiceName: _UNICODE_STRING = core::mem::zeroed();
    let mut RelativeName: *mut wchar_t = core::ptr::null_mut();
    let mut SourceName: *const wchar_t = lpServiceName;

    ServiceNameLen = wcslen(lpServiceName);

    if ServiceNameLen > 12 && _wcsnicmp(win32macros_rust::wide_bytes!(b"\\\0S\0y\0s\0t\0e\0m\0R\0o\0o\0t\0\\\0\0\0") as *const wchar_t, lpServiceName, 12) == 0 {
        *lpCanonName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;

        if (*lpCanonName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        if dwStartType == SERVICE_BOOT_START {
            SourceName = SourceName.add(12);
        }

        wcscpy(*lpCanonName, SourceName);

        return NO_ERROR;
    }

    if ServiceNameLen > 13 && _wcsnicmp(win32macros_rust::wide_bytes!(b"%\0S\0y\0s\0t\0e\0m\0R\0o\0o\0t\0%\0\\\0\0\0") as *const wchar_t, lpServiceName, 13) == 0 {
        *lpCanonName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;

        if (*lpCanonName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        if dwStartType == SERVICE_BOOT_START {
            wcscpy(*lpCanonName, win32macros_rust::wide_bytes!(b"\\\0S\0y\0s\0t\0e\0m\0R\0o\0o\0t\0\\\0\0\0") as *const wchar_t);
        }

        wcscat(*lpCanonName, lpServiceName.add(13));

        return NO_ERROR;
    }

    if *lpServiceName != ('\\' as wchar_t) && *lpServiceName.add(1) != (':' as wchar_t) {
        *lpCanonName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (ServiceNameLen + 1) * core::mem::size_of::<wchar_t>()) as *mut wchar_t;

        if (*lpCanonName).is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        wcscpy(*lpCanonName, lpServiceName);

        return NO_ERROR;
    }

    if RtlDosPathNameToNtPathName_U(lpServiceName, &mut NtServiceName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    *lpCanonName = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, NtServiceName.Length as usize + core::mem::size_of::<wchar_t>()) as *mut wchar_t;

    if (*lpCanonName).is_null() {
        RtlFreeUnicodeString(&mut NtServiceName);
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    wcsncpy(*lpCanonName, NtServiceName.Buffer, NtServiceName.Length as usize / core::mem::size_of::<wchar_t>());

    RtlFreeUnicodeString(&mut NtServiceName);

    if dwStartType != SERVICE_BOOT_START {
        return NO_ERROR;
    }

    Result = ScmConvertToBootPathName(*lpCanonName, &mut RelativeName);
    if Result != 0 {
        HeapFree(GetProcessHeap(), 0, *lpCanonName as *mut core::ffi::c_void);
        return Result;
    }

    wcscpy(*lpCanonName, RelativeName.add(12));

    HeapFree(GetProcessHeap(), 0, RelativeName as *mut core::ffi::c_void);

    NO_ERROR
}
#[no_mangle]
pub unsafe extern "C" fn Int_EnumDependentServicesW(
    hServicesKey: HKEY,
    lpService: PSERVICE,
    dwServiceState: DWORD,
    lpServices: *mut PSERVICE,
    pcbBytesNeeded: LPDWORD,
    lpServicesReturned: LPDWORD,
) -> DWORD {
    let mut dwError: DWORD;
    let mut szNameBuf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szValueBuf: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let lpszNameBuf: *mut WCHAR = szNameBuf.as_mut_ptr();
    let lpszValueBuf: *mut WCHAR = szValueBuf.as_mut_ptr();
    let mut dwSize: DWORD;
    let mut dwNumSubKeys: DWORD = 0;
    let mut dwIteration: DWORD;
    let mut lpCurrentService: PSERVICE;
    let mut hServiceEnumKey: HKEY = core::ptr::null_mut();
    let mut dwCurrentServiceState: DWORD = SERVICE_ACTIVE;
    let mut dwDependServiceStrPtr: DWORD;
    let mut dwRequiredSize: DWORD;

    dwError = RegQueryInfoKeyW(
        hServicesKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwNumSubKeys,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwIteration = 0;
    while dwIteration < dwNumSubKeys {
        dwSize = MAX_PATH;
        dwError = RegEnumKeyExW(
            hServicesKey,
            dwIteration,
            lpszNameBuf,
            &mut dwSize,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            return dwError;
        }

        dwError = RegOpenKeyExW(
            hServicesKey,
            lpszNameBuf,
            0,
            KEY_READ,
            &mut hServiceEnumKey,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            return dwError;
        }

        dwSize = MAX_PATH * core::mem::size_of::<WCHAR>() as DWORD;

        dwError = RegQueryValueExW(
            hServiceEnumKey,
            wstr!("DependOnService\0").as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            lpszValueBuf as LPBYTE,
            &mut dwSize,
        ) as DWORD;

        if dwError == ERROR_SUCCESS {
            dwDependServiceStrPtr = 0;

            while wcslen(lpszValueBuf.add(dwDependServiceStrPtr as usize)) > 0 {
                if _wcsicmp(
                    lpszValueBuf.add(dwDependServiceStrPtr as usize),
                    (*lpService).lpServiceName,
                ) == 0
                {
                    lpCurrentService = ScmGetServiceEntryByName(lpszNameBuf);

                    if lpCurrentService.is_null() {
                        return ERROR_NOT_FOUND;
                    }

                    if (*lpCurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                        dwCurrentServiceState = SERVICE_INACTIVE;
                    }

                    if (dwCurrentServiceState == dwServiceState)
                        || (dwServiceState == SERVICE_STATE_ALL)
                    {
                        dwRequiredSize = 0;
                        dwRequiredSize += core::mem::size_of::<SERVICE_STATUS>() as DWORD;
                        dwRequiredSize += ((wcslen((*lpCurrentService).lpServiceName) + 1)
                            * core::mem::size_of::<WCHAR>())
                            as DWORD;
                        dwRequiredSize += ((wcslen((*lpCurrentService).lpDisplayName) + 1)
                            * core::mem::size_of::<WCHAR>())
                            as DWORD;

                        dwRequiredSize += (2 * core::mem::size_of::<PVOID>()) as DWORD;

                        *pcbBytesNeeded = *pcbBytesNeeded + dwRequiredSize;

                        Int_EnumDependentServicesW(
                            hServicesKey,
                            lpCurrentService,
                            dwServiceState,
                            lpServices,
                            pcbBytesNeeded,
                            lpServicesReturned,
                        );

                        if !lpServices.is_null() {
                            *lpServices.add(*lpServicesReturned as usize) = lpCurrentService;
                        }

                        *lpServicesReturned = *lpServicesReturned + 1;
                    }
                }

                dwDependServiceStrPtr +=
                    (wcslen(lpszValueBuf.add(dwDependServiceStrPtr as usize)) + 1) as DWORD;
            }
        } else if *pcbBytesNeeded != 0 {
            dwError = ERROR_SUCCESS;
        }

        RegCloseKey(hServiceEnumKey);

        dwIteration += 1;
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RCloseServiceHandle(
    hSCObject: LPSC_RPC_HANDLE,
) -> DWORD {
    let mut hManager: PMANAGER_HANDLE;
    let mut hService: PSERVICE_HANDLE;
    let lpService: PSERVICE;

    if (*hSCObject).is_null() {
        return ERROR_INVALID_HANDLE;
    }

    hManager = ScmGetServiceManagerFromHandle(*hSCObject);
    hService = ScmGetServiceFromHandle(*hSCObject);

    if !hManager.is_null() {
        (*hManager).Handle.Tag = INVALID_TAG;

        HeapFree(GetProcessHeap(), 0, hManager as _);
        hManager = core::ptr::null_mut();
        let _ = hManager;

        *hSCObject = core::ptr::null_mut();

        return ERROR_SUCCESS;
    } else if !hService.is_null() {
        ScmLockDatabaseExclusive();

        lpService = (*hService).ServiceEntry;

        (*hService).Handle.Tag = INVALID_TAG;

        HeapFree(GetProcessHeap(), 0, hService as _);
        hService = core::ptr::null_mut();
        let _ = hService;

        ScmDereferenceService(lpService as *mut c_void);

        ScmUnlockDatabase();

        *hSCObject = core::ptr::null_mut();

        return ERROR_SUCCESS;
    }

    ERROR_INVALID_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn RControlService(
    hService: SC_RPC_HANDLE,
    dwControl: DWORD,
    lpServiceStatus: LPSERVICE_STATUS,
) -> DWORD {
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let DesiredAccess: ACCESS_MASK;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut pcbBytesNeeded: DWORD = 0;
    let mut dwServicesReturned: DWORD = 0;
    let dwControlsAccepted: DWORD;
    let dwCurrentState: DWORD;
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut lpLogStrings: [LPCWSTR; 2] = [core::ptr::null(); 2];
    let mut szLogBuffer: [WCHAR; 80] = [0; 80];
    let mut uID: UINT = 0;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    match dwControl {
        SERVICE_CONTROL_STOP => {
            DesiredAccess = SERVICE_STOP;
        }
        SERVICE_CONTROL_PAUSE
        | SERVICE_CONTROL_CONTINUE
        | SERVICE_CONTROL_PARAMCHANGE
        | SERVICE_CONTROL_NETBINDADD
        | SERVICE_CONTROL_NETBINDREMOVE
        | SERVICE_CONTROL_NETBINDENABLE
        | SERVICE_CONTROL_NETBINDDISABLE => {
            DesiredAccess = SERVICE_PAUSE_CONTINUE;
        }
        SERVICE_CONTROL_INTERROGATE => {
            DesiredAccess = SERVICE_INTERROGATE;
        }
        _ => {
            if dwControl >= 128 && dwControl <= 255 {
                DesiredAccess = SERVICE_USER_DEFINED_CONTROL;
            } else {
                return ERROR_INVALID_PARAMETER;
            }
        }
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, DesiredAccess) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    RtlCopyMemory(
        lpServiceStatus as *mut core::ffi::c_void,
        core::ptr::addr_of!((*lpService).Status) as *const core::ffi::c_void,
        core::mem::size_of::<SERVICE_STATUS>(),
    );

    if dwControl == SERVICE_CONTROL_STOP {
        dwError = RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("System\\CurrentControlSet\\Services").as_ptr() as LPCWSTR,
            0,
            KEY_READ,
            &mut hServicesKey,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            return dwError;
        }

        Int_EnumDependentServicesW(
            hServicesKey,
            lpService,
            SERVICE_ACTIVE,
            core::ptr::null_mut(),
            &mut pcbBytesNeeded,
            &mut dwServicesReturned,
        );

        RegCloseKey(hServicesKey);

        if pcbBytesNeeded != 0 {
            return ERROR_DEPENDENT_SERVICES_RUNNING;
        }
    }

    if (*lpService).Status.dwServiceType & SERVICE_DRIVER != 0 {
        dwError = ScmControlDriver(lpService, dwControl, lpServiceStatus);
    } else {
        dwControlsAccepted = (*lpService).Status.dwControlsAccepted;
        dwCurrentState = (*lpService).Status.dwCurrentState;

        if (*lpService).lpImage.is_null() || dwCurrentState == SERVICE_STOPPED {
            return ERROR_SERVICE_NOT_ACTIVE;
        }

        match dwCurrentState {
            SERVICE_STOP_PENDING | SERVICE_STOPPED => {
                return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
            }
            SERVICE_START_PENDING => match dwControl {
                SERVICE_CONTROL_STOP => {}
                SERVICE_CONTROL_INTERROGATE => {
                    RtlCopyMemory(
                        lpServiceStatus as *mut core::ffi::c_void,
                        core::ptr::addr_of!((*lpService).Status) as *const core::ffi::c_void,
                        core::mem::size_of::<SERVICE_STATUS>(),
                    );
                    return ERROR_SUCCESS;
                }
                _ => {
                    return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
                }
            },
            _ => {}
        }

        match dwControl {
            SERVICE_CONTROL_STOP => {
                if dwControlsAccepted & SERVICE_ACCEPT_STOP == 0 {
                    return ERROR_INVALID_SERVICE_CONTROL;
                }
            }
            SERVICE_CONTROL_PAUSE | SERVICE_CONTROL_CONTINUE => {
                if dwControlsAccepted & SERVICE_ACCEPT_PAUSE_CONTINUE == 0 {
                    return ERROR_INVALID_SERVICE_CONTROL;
                }
            }
            SERVICE_CONTROL_PARAMCHANGE => {
                if dwControlsAccepted & SERVICE_ACCEPT_PARAMCHANGE == 0 {
                    return ERROR_INVALID_SERVICE_CONTROL;
                }
            }
            SERVICE_CONTROL_NETBINDADD
            | SERVICE_CONTROL_NETBINDREMOVE
            | SERVICE_CONTROL_NETBINDENABLE
            | SERVICE_CONTROL_NETBINDDISABLE => {
                if dwControlsAccepted & SERVICE_ACCEPT_NETBINDCHANGE == 0 {
                    return ERROR_INVALID_SERVICE_CONTROL;
                }
            }
            _ => {}
        }

        dwError = ScmControlService(
            (*(*lpService).lpImage).hControlPipe,
            (*lpService).lpServiceName,
            dwControl,
            lpService as SERVICE_STATUS_HANDLE,
        );

        RtlCopyMemory(
            lpServiceStatus as *mut core::ffi::c_void,
            core::ptr::addr_of!((*lpService).Status) as *const core::ffi::c_void,
            core::mem::size_of::<SERVICE_STATUS>(),
        );
    }

    if dwError == ERROR_SUCCESS {
        if dwControl == SERVICE_CONTROL_STOP
            || dwControl == SERVICE_CONTROL_PAUSE
            || dwControl == SERVICE_CONTROL_CONTINUE
        {
            match dwControl {
                SERVICE_CONTROL_STOP => {
                    uID = IDS_SERVICE_STOP;
                }
                SERVICE_CONTROL_PAUSE => {
                    uID = IDS_SERVICE_PAUSE;
                }
                SERVICE_CONTROL_CONTINUE => {
                    uID = IDS_SERVICE_RESUME;
                }
                _ => {}
            }
            LoadStringW(
                GetModuleHandleW(core::ptr::null()),
                uID,
                szLogBuffer.as_mut_ptr(),
                szLogBuffer.len() as core::ffi::c_int,
            );

            lpLogStrings[0] = (*lpService).lpDisplayName;
            lpLogStrings[1] = szLogBuffer.as_ptr();

            ScmLogEvent(
                EVENT_SERVICE_CONTROL_SUCCESS,
                EVENTLOG_INFORMATION_TYPE as u16,
                2,
                lpLogStrings.as_mut_ptr(),
            );
        }
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RDeleteService(hService: SC_RPC_HANDLE) -> DWORD {
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let dwError: DWORD;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, DELETE) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseExclusive();

    'Done: {
        if (*lpService).bDeleted != 0 {
            dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
            break 'Done;
        }

        (*lpService).bDeleted = TRUE as _;

        dwError = ScmMarkServiceForDelete(lpService);
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RLockServiceDatabase(
    hSCManager: SC_RPC_HANDLE,
    lpLock: LPSC_RPC_LOCK,
) -> DWORD {
    let hMgr: PMANAGER_HANDLE;

    *lpLock = core::ptr::null_mut();

    hMgr = ScmGetServiceManagerFromHandle(hSCManager);
    if hMgr.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hMgr).Handle.DesiredAccess, SC_MANAGER_LOCK) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    ScmAcquireServiceStartLock(0, lpLock)
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceObjectSecurity(
    hService: SC_RPC_HANDLE,
    dwSecurityInformation: SECURITY_INFORMATION,
    lpSecurityDescriptor: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
) -> DWORD {
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut DesiredAccess: ULONG = 0;
    let Status: NTSTATUS;
    let mut dwBytesNeeded: DWORD = 0;
    let dwError: DWORD;

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if dwSecurityInformation
        & (DACL_SECURITY_INFORMATION
            | GROUP_SECURITY_INFORMATION
            | OWNER_SECURITY_INFORMATION)
        != 0
    {
        DesiredAccess |= READ_CONTROL;
    }

    if dwSecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, DesiredAccess) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    Status = RtlQuerySecurityObject(
        (*lpService).pSecurityDescriptor,
        dwSecurityInformation,
        lpSecurityDescriptor as PSECURITY_DESCRIPTOR,
        cbBufSize,
        &mut dwBytesNeeded,
    );

    ScmUnlockDatabase();

    if NT_SUCCESS(Status) {
        *pcbBytesNeeded = dwBytesNeeded;
        dwError = STATUS_SUCCESS as DWORD;
    } else if Status == STATUS_BUFFER_TOO_SMALL {
        *pcbBytesNeeded = dwBytesNeeded;
        dwError = ERROR_INSUFFICIENT_BUFFER;
    } else if Status == STATUS_BAD_DESCRIPTOR_FORMAT {
        dwError = ERROR_GEN_FAILURE;
    } else {
        dwError = RtlNtStatusToDosError(Status);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RSetServiceObjectSecurity(
    hService: SC_RPC_HANDLE,
    dwSecurityInformation: DWORD,
    lpSecurityDescriptor: LPBYTE,
    _dwSecurityDescriptorSize: DWORD,
) -> DWORD {
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut DesiredAccess: ACCESS_MASK = 0;
    let hToken: HANDLE = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut bDatabaseLocked: BOOL = 0;
    let Status: NTSTATUS;
    let mut dwError: DWORD;

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if dwSecurityInformation == 0
        || (dwSecurityInformation
            & !(OWNER_SECURITY_INFORMATION
                | GROUP_SECURITY_INFORMATION
                | DACL_SECURITY_INFORMATION
                | SACL_SECURITY_INFORMATION))
            != 0
    {
        return ERROR_INVALID_PARAMETER;
    }

    if RtlValidSecurityDescriptor(lpSecurityDescriptor as PSECURITY_DESCRIPTOR) == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    if dwSecurityInformation & SACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= ACCESS_SYSTEM_SECURITY;
    }

    if dwSecurityInformation & DACL_SECURITY_INFORMATION != 0 {
        DesiredAccess |= WRITE_DAC;
    }

    if dwSecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION) != 0 {
        DesiredAccess |= WRITE_OWNER;
    }

    if (dwSecurityInformation & OWNER_SECURITY_INFORMATION != 0)
        && (*(lpSecurityDescriptor as PISECURITY_DESCRIPTOR)).Owner.is_null()
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwSecurityInformation & GROUP_SECURITY_INFORMATION != 0)
        && (*(lpSecurityDescriptor as PISECURITY_DESCRIPTOR)).Group.is_null()
    {
        return ERROR_INVALID_PARAMETER;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, DesiredAccess) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if (*lpService).bDeleted != 0 {
        return ERROR_SERVICE_MARKED_FOR_DELETE;
    }

    'block: {
        Status = RtlSetSecurityObject(
            dwSecurityInformation,
            lpSecurityDescriptor as PSECURITY_DESCRIPTOR,
            core::ptr::addr_of_mut!((*lpService).pSecurityDescriptor),
            &raw mut ScmServiceMapping,
            hToken,
        );
        if !NT_SUCCESS(Status) {
            dwError = RtlNtStatusToDosError(Status);
            break 'block;
        }

        ScmLockDatabaseExclusive();
        bDatabaseLocked = 1;

        dwError = ScmOpenServiceKey(
            (*lpService).lpServiceName,
            READ_CONTROL | KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
            &mut hServiceKey,
        );
        if dwError != ERROR_SUCCESS {
            break 'block;
        }

        dwError = ScmWriteSecurityDescriptor(hServiceKey, (*lpService).pSecurityDescriptor);

        RegFlushKey(hServiceKey);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    if bDatabaseLocked == 1 {
        ScmUnlockDatabase();
    }

    if !hToken.is_null() {
        NtClose(hToken);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceStatus(
    hService: SC_RPC_HANDLE,
    lpServiceStatus: LPSERVICE_STATUS,
) -> DWORD {
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_STATUS) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    RtlCopyMemory(
        lpServiceStatus as *mut core::ffi::c_void,
        core::ptr::addr_of!((*lpService).Status) as *const core::ffi::c_void,
        core::mem::size_of::<SERVICE_STATUS>(),
    );

    ScmUnlockDatabase();

    ERROR_SUCCESS
}

unsafe fn ScmIsValidServiceState(dwCurrentState: DWORD) -> BOOL {
    match dwCurrentState {
        SERVICE_STOPPED
        | SERVICE_START_PENDING
        | SERVICE_STOP_PENDING
        | SERVICE_RUNNING
        | SERVICE_CONTINUE_PENDING
        | SERVICE_PAUSE_PENDING
        | SERVICE_PAUSED => TRUE as BOOL,
        _ => FALSE as BOOL,
    }
}
unsafe fn ScmStopThread_impl(pParam: *mut core::ffi::c_void) -> DWORD {
    let lpService = pParam as PSERVICE;
    let mut szLogBuffer: [u16; 80] = [0; 80];
    let mut lpLogStrings: [LPCWSTR; 2] = [core::ptr::null(); 2];

    if (*(*lpService).lpImage).dwImageRunCount == 1 {
        ScmControlService(
            (*(*lpService).lpImage).hControlPipe,
            [0u16].as_ptr(),
            SERVICE_CONTROL_STOP,
            lpService as *mut core::ffi::c_void,
        );
    }

    ScmLockDatabaseExclusive();

    (*(*lpService).lpImage).dwImageRunCount -= 1;

    if (*(*lpService).lpImage).dwImageRunCount == 0 {
        ScmRemoveServiceImage((*lpService).lpImage as *mut core::ffi::c_void);
        (*lpService).lpImage = core::ptr::null_mut();
    }

    if (*lpService).Status.dwWin32ExitCode != ERROR_SUCCESS {
        scm_append_u32(
            szLogBuffer.as_mut_ptr(),
            szLogBuffer.len(),
            0,
            (*lpService).Status.dwWin32ExitCode,
        );
        lpLogStrings[0] = (*lpService).lpDisplayName;
        lpLogStrings[1] = szLogBuffer.as_ptr();

        ScmLogEvent(
            EVENT_SERVICE_EXIT_FAILED,
            EVENTLOG_ERROR_TYPE as u16,
            lpLogStrings.len() as u16,
            lpLogStrings.as_mut_ptr(),
        );
    } else {
        LoadStringW(
            GetModuleHandleW(core::ptr::null()),
            IDS_SERVICE_STOPPED,
            szLogBuffer.as_mut_ptr(),
            szLogBuffer.len() as i32,
        );
        lpLogStrings[0] = (*lpService).lpDisplayName;
        lpLogStrings[1] = szLogBuffer.as_ptr();

        ScmLogEvent(
            EVENT_SERVICE_STATUS_SUCCESS,
            EVENTLOG_INFORMATION_TYPE as u16,
            lpLogStrings.len() as u16,
            lpLogStrings.as_mut_ptr(),
        );
    }

    ScmDereferenceService(lpService as *mut core::ffi::c_void);

    ScmUnlockDatabase();

    0
}

// CreateThread's LPTHREAD_START_ROUTINE is `extern "stdcall"` on i386 but
// `extern "C"` on amd64/arm64. Rust treats these as distinct fn-item types, so
// the trampoline's ABI must match the platform's thread-routine type exactly.
unsafe extern "system" fn ScmStopThread(pParam: *mut core::ffi::c_void) -> DWORD {
    ScmStopThread_impl(pParam)
}

#[no_mangle]
pub unsafe extern "system" fn RSetServiceStatus(
    hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
    lpServiceStatus: LPSERVICE_STATUS,
) -> DWORD {
    let lpService: PSERVICE;
    let dwPreviousState: DWORD;
    let dwPreviousType: DWORD;
    let mut lpLogStrings: [LPCWSTR; 2] = [core::ptr::null(); 2];
    let mut szLogBuffer: [WCHAR; 80] = [0; 80];
    let mut uID: UINT = 0;

    if hServiceStatus == 0 {
        return ERROR_INVALID_HANDLE;
    }

    lpService = hServiceStatus as PSERVICE;

    /* Check current state */
    if ScmIsValidServiceState((*lpServiceStatus).dwCurrentState) == 0 {
        return ERROR_INVALID_DATA;
    }

    /* Check service type */
    if ((*lpServiceStatus).dwServiceType & SERVICE_WIN32) == 0
        && ((*lpServiceStatus).dwServiceType & SERVICE_DRIVER) != 0
    {
        return ERROR_INVALID_DATA;
    }

    /* Check accepted controls */
    if (*lpServiceStatus).dwControlsAccepted & !0xFFu32 != 0 {
        return ERROR_INVALID_DATA;
    }

    /* Set the wait hint and check point only if the service is in a pending state,
       otherwise they should be 0 */
    if (*lpServiceStatus).dwCurrentState == SERVICE_STOPPED
        || (*lpServiceStatus).dwCurrentState == SERVICE_PAUSED
        || (*lpServiceStatus).dwCurrentState == SERVICE_RUNNING
    {
        (*lpServiceStatus).dwWaitHint = 0;
        (*lpServiceStatus).dwCheckPoint = 0;
    }

    /* Lock the service database exclusively */
    ScmLockDatabaseExclusive();

    /* Save the current service state */
    dwPreviousState = (*lpService).Status.dwCurrentState;

    /* Save the current service type */
    dwPreviousType = (*lpService).Status.dwServiceType;

    /* Update the service status */
    RtlCopyMemory(
        core::ptr::addr_of_mut!((*lpService).Status) as *mut core::ffi::c_void,
        lpServiceStatus as *const core::ffi::c_void,
        core::mem::size_of::<SERVICE_STATUS>(),
    );

    /* Restore the previous service type */
    (*lpService).Status.dwServiceType = dwPreviousType;

    if (*lpServiceStatus).dwCurrentState != SERVICE_STOPPED && dwPreviousState == SERVICE_STOPPED {
        /* Keep a reference on all non stopped services */
        ScmReferenceService(lpService);
    }

    /* Check if the service just stopped */
    if (*lpServiceStatus).dwCurrentState == SERVICE_STOPPED && dwPreviousState != SERVICE_STOPPED {
        let hStopThread: HANDLE;
        let mut dwStopThreadId: DWORD = 0;

        hStopThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(ScmStopThread),
            lpService as LPVOID,
            0,
            &mut dwStopThreadId,
        );
        if hStopThread.is_null() {
            /* We can't leave without releasing the reference.
             * We also can't remove it without holding the lock. */
            ScmDereferenceService(lpService as *mut c_void);
        } else {
            CloseHandle(hStopThread);
        }
    }

    /* Unlock the service database */
    ScmUnlockDatabase();

    if (*lpServiceStatus).dwCurrentState != dwPreviousState
        && ((*lpServiceStatus).dwCurrentState == SERVICE_RUNNING
            || (*lpServiceStatus).dwCurrentState == SERVICE_PAUSED)
    {
        /* Log a successful service status change */
        match (*lpServiceStatus).dwCurrentState {
            SERVICE_RUNNING => {
                uID = IDS_SERVICE_RUNNING;
            }
            SERVICE_PAUSED => {
                uID = IDS_SERVICE_PAUSED;
            }
            _ => {}
        }

        LoadStringW(
            GetModuleHandleW(core::ptr::null()),
            uID,
            szLogBuffer.as_mut_ptr(),
            szLogBuffer.len() as core::ffi::c_int,
        );
        lpLogStrings[0] = (*lpService).lpDisplayName;
        lpLogStrings[1] = szLogBuffer.as_ptr();

        ScmLogEvent(
            EVENT_SERVICE_STATUS_SUCCESS,
            EVENTLOG_INFORMATION_TYPE as u16,
            2,
            lpLogStrings.as_mut_ptr(),
        );
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RUnlockServiceDatabase(Lock: LPSC_RPC_LOCK) -> DWORD {
    ScmReleaseServiceStartLock(Lock)
}

#[no_mangle]
pub unsafe extern "system" fn RNotifyBootConfigStatus(
    _lpMachineName: SVCCTL_HANDLEW,
    BootAcceptable: DWORD,
) -> DWORD {
    if BootAcceptable != 0 {
        return ScmAcceptBoot();
    }

    ScmRunLastKnownGood()
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScSetServiceBitsW(
    hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
    dwServiceBits: DWORD,
    bSetBitsOn: core::ffi::c_int,
    _bUpdateImmediately: core::ffi::c_int,
    lpString: *mut wchar_t,
) -> DWORD {
    let pService: PSERVICE;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if !lpString.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if hServiceStatus == 0 {
        return ERROR_INVALID_HANDLE;
    }

    pService = hServiceStatus as PSERVICE;

    if bSetBitsOn != 0 {
        (*pService).dwServiceBits |= dwServiceBits;
        g_dwServiceBits |= dwServiceBits;
    } else {
        (*pService).dwServiceBits &= !dwServiceBits;
        g_dwServiceBits &= !dwServiceBits;
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RChangeServiceConfigW(
    hService: SC_RPC_HANDLE,
    dwServiceType: DWORD,
    dwStartType: DWORD,
    dwErrorControl: DWORD,
    lpBinaryPathName: LPWSTR,
    lpLoadOrderGroup: LPWSTR,
    lpdwTagId: LPDWORD,
    lpDependencies: LPBYTE,
    dwDependSize: DWORD,
    lpServiceStartName: LPWSTR,
    lpPassword: LPBYTE,
    dwPwSize: DWORD,
    lpDisplayName: LPWSTR,
) -> DWORD {

    const ERROR_SUCCESS: DWORD = 0;
    const ERROR_INVALID_HANDLE: DWORD = 6;
    const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
    const ERROR_INVALID_PARAMETER: DWORD = 87;
    const ERROR_FILE_NOT_FOUND: DWORD = 2;
    const ERROR_ACCESS_DENIED: DWORD = 5;
    const ERROR_SHUTDOWN_IN_PROGRESS: DWORD = 1115;
    const ERROR_SERVICE_MARKED_FOR_DELETE: DWORD = 1072;

    const SERVICE_NO_CHANGE: DWORD = 0xffffffff;
    const SERVICE_KERNEL_DRIVER: DWORD = 0x00000001;
    const SERVICE_FILE_SYSTEM_DRIVER: DWORD = 0x00000002;
    const SERVICE_WIN32_OWN_PROCESS: DWORD = 0x00000010;
    const SERVICE_WIN32_SHARE_PROCESS: DWORD = 0x00000020;
    const SERVICE_INTERACTIVE_PROCESS: DWORD = 0x00000100;
    const SERVICE_DRIVER: DWORD = 0x0000000b;
    const SERVICE_WIN32: DWORD = 0x00000030;

    const SERVICE_BOOT_START: DWORD = 0x00000000;
    const SERVICE_SYSTEM_START: DWORD = 0x00000001;
    const SERVICE_AUTO_START: DWORD = 0x00000002;
    const SERVICE_DEMAND_START: DWORD = 0x00000003;
    const SERVICE_DISABLED: DWORD = 0x00000004;

    const SERVICE_ERROR_IGNORE: DWORD = 0x00000000;
    const SERVICE_ERROR_NORMAL: DWORD = 0x00000001;
    const SERVICE_ERROR_SEVERE: DWORD = 0x00000002;
    const SERVICE_ERROR_CRITICAL: DWORD = 0x00000003;

    const SERVICE_CHANGE_CONFIG: DWORD = 0x0002;
    const HEAP_ZERO_MEMORY: DWORD = 0x00000008;

    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let lpDisplayNameW: LPWSTR;
    let mut lpImagePathW: LPWSTR;
    let mut lpClearTextPassword: LPWSTR = core::ptr::null_mut();

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_CHANGE_CONFIG) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    if (dwServiceType != SERVICE_NO_CHANGE)
        && (dwServiceType != SERVICE_KERNEL_DRIVER)
        && (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER)
        && ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS)
        && ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwStartType != SERVICE_NO_CHANGE)
        && (dwStartType != SERVICE_BOOT_START)
        && (dwStartType != SERVICE_SYSTEM_START)
        && (dwStartType != SERVICE_AUTO_START)
        && (dwStartType != SERVICE_DEMAND_START)
        && (dwStartType != SERVICE_DISABLED)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwStartType == SERVICE_BOOT_START) || (dwStartType == SERVICE_SYSTEM_START) {
        if (dwServiceType != SERVICE_KERNEL_DRIVER) && (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER) {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if (dwErrorControl != SERVICE_NO_CHANGE)
        && (dwErrorControl != SERVICE_ERROR_IGNORE)
        && (dwErrorControl != SERVICE_ERROR_NORMAL)
        && (dwErrorControl != SERVICE_ERROR_SEVERE)
        && (dwErrorControl != SERVICE_ERROR_CRITICAL)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if !lpdwTagId.is_null() && (lpLoadOrderGroup.is_null() || *lpLoadOrderGroup == 0) {
        return ERROR_INVALID_PARAMETER;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseExclusive();

    'done: loop {
        if (*lpService).bDeleted != 0 {
            dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
            break 'done;
        }

        dwError = ScmOpenServiceKey(
            (*lpService).szServiceName.as_mut_ptr(),
            KEY_SET_VALUE,
            &mut hServiceKey,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if !lpDisplayName.is_null() && *lpDisplayName != 0 {
            RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"D\0i\0s\0p\0l\0a\0y\0N\0a\0m\0e\0\0\0") as *const wchar_t,
                0,
                REG_SZ,
                lpDisplayName as *const u8,
                ((wcslen(lpDisplayName) + 1) * core::mem::size_of::<u16>()) as DWORD,
            );

            lpDisplayNameW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (wcslen(lpDisplayName) + 1) * core::mem::size_of::<u16>(),
            ) as LPWSTR;
            if lpDisplayNameW.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'done;
            }

            wcscpy(lpDisplayNameW, lpDisplayName);
            if (*lpService).lpDisplayName != (*lpService).lpServiceName {
                HeapFree(GetProcessHeap(), 0, (*lpService).lpDisplayName as *mut core::ffi::c_void);
            }

            (*lpService).lpDisplayName = lpDisplayNameW;
        }

        if dwServiceType != SERVICE_NO_CHANGE {
            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"T\0y\0p\0e\0\0\0") as *const wchar_t,
                0,
                REG_DWORD,
                &dwServiceType as *const DWORD as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            (*lpService).Status.dwServiceType = dwServiceType;
        }

        if dwStartType != SERVICE_NO_CHANGE {
            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"S\0t\0a\0r\0t\0\0\0") as *const wchar_t,
                0,
                REG_DWORD,
                &dwStartType as *const DWORD as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            (*lpService).dwStartType = dwStartType;
        }

        if dwErrorControl != SERVICE_NO_CHANGE {
            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"E\0r\0r\0o\0r\0C\0o\0n\0t\0r\0o\0l\0\0\0") as *const wchar_t,
                0,
                REG_DWORD,
                &dwErrorControl as *const DWORD as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            (*lpService).dwErrorControl = dwErrorControl;
        }

        if !lpBinaryPathName.is_null() && *lpBinaryPathName != 0 {
            lpImagePathW = lpBinaryPathName;

            if (*lpService).Status.dwServiceType & SERVICE_DRIVER != 0 {
                dwError = ScmCanonDriverImagePath(
                    (*lpService).dwStartType,
                    lpBinaryPathName,
                    &mut lpImagePathW,
                );

                if dwError != ERROR_SUCCESS {
                    break 'done;
                }
            }

            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"I\0m\0a\0g\0e\0P\0a\0t\0h\0\0\0") as *const wchar_t,
                0,
                REG_EXPAND_SZ,
                lpImagePathW as *const u8,
                ((wcslen(lpImagePathW) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;

            if lpImagePathW != lpBinaryPathName {
                HeapFree(GetProcessHeap(), 0, lpImagePathW as *mut core::ffi::c_void);
            }

            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpLoadOrderGroup.is_null() && *lpLoadOrderGroup != 0 {
            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"G\0r\0o\0u\0p\0\0\0") as *const wchar_t,
                0,
                REG_SZ,
                lpLoadOrderGroup as *const u8,
                ((wcslen(lpLoadOrderGroup) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            dwError = ScmSetServiceGroup(lpService, lpLoadOrderGroup as LPWSTR);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpdwTagId.is_null() {
            dwError = ScmAssignNewTag(lpService);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            dwError = RegSetValueExW(
                hServiceKey,
                win32macros_rust::wide_bytes!(b"T\0a\0g\0\0\0") as *const wchar_t,
                0,
                REG_DWORD,
                core::ptr::addr_of!((*lpService).dwTag) as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            *lpdwTagId = (*lpService).dwTag;
        }

        if !lpDependencies.is_null() && *lpDependencies != 0 {
            dwError = ScmWriteDependencies(hServiceKey, lpDependencies as LPWSTR, dwDependSize);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if (*lpService).Status.dwServiceType & SERVICE_WIN32 != 0 {
            if !lpServiceStartName.is_null() && *lpServiceStartName != 0 {
                dwError = RegSetValueExW(
                    hServiceKey,
                    win32macros_rust::wide_bytes!(b"O\0b\0j\0e\0c\0t\0N\0a\0m\0e\0\0\0") as *const wchar_t,
                    0,
                    REG_SZ,
                    lpServiceStartName as *const u8,
                    ((wcslen(lpServiceStartName) + 1) * core::mem::size_of::<u16>()) as DWORD,
                ) as DWORD;
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }
            }

            if !lpPassword.is_null() {
                if *(lpPassword as LPWSTR) != 0 {
                    dwError = ScmDecryptPassword(
                        hService,
                        lpPassword,
                        dwPwSize,
                        &mut lpClearTextPassword,
                    );
                    if dwError != ERROR_SUCCESS {
                        break 'done;
                    }

                    dwError = ScmSetServicePassword(
                        (*lpService).szServiceName.as_ptr(),
                        lpClearTextPassword,
                    );
                    if dwError != ERROR_SUCCESS {
                        break 'done;
                    }
                } else {
                    dwError = ScmSetServicePassword(
                        (*lpService).szServiceName.as_ptr(),
                        core::ptr::null_mut(),
                    );
                    if dwError == ERROR_FILE_NOT_FOUND {
                        dwError = ERROR_SUCCESS;
                    }

                    if dwError != ERROR_SUCCESS {
                        break 'done;
                    }
                }
            }
        }

        break 'done;
    }

    if !lpClearTextPassword.is_null() {
        SecureZeroMemory(
            lpClearTextPassword as *mut core::ffi::c_void,
            (wcslen(lpClearTextPassword) + 1) * core::mem::size_of::<u16>(),
        );
        HeapFree(GetProcessHeap(), 0, lpClearTextPassword as *mut core::ffi::c_void);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RCreateServiceW(
    hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPCWSTR,
    lpDisplayName: LPCWSTR,
    dwDesiredAccess: DWORD,
    dwServiceType: DWORD,
    dwStartType: DWORD,
    dwErrorControl: DWORD,
    lpBinaryPathName: LPCWSTR,
    lpLoadOrderGroup: LPCWSTR,
    lpdwTagId: LPDWORD,
    lpDependencies: LPBYTE,
    dwDependSize: DWORD,
    lpServiceStartName: LPCWSTR,
    lpPassword: LPBYTE,
    dwPwSize: DWORD,
    lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {

    const ERROR_SUCCESS: DWORD = 0;
    const ERROR_INVALID_HANDLE: DWORD = 6;
    const ERROR_ACCESS_DENIED: DWORD = 5;
    const ERROR_INVALID_NAME: DWORD = 123;
    const ERROR_INVALID_PARAMETER: DWORD = 87;
    const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
    const ERROR_SHUTDOWN_IN_PROGRESS: DWORD = 1115;
    const ERROR_SERVICE_MARKED_FOR_DELETE: DWORD = 1072;
    const ERROR_SERVICE_EXISTS: DWORD = 1073;
    const ERROR_DUPLICATE_SERVICE_NAME: DWORD = 1078;
    const HEAP_ZERO_MEMORY: DWORD = 0x00000008;
    const KEY_WRITE: u32 = 0x00020006;

    const SC_MANAGER_CREATE_SERVICE: DWORD = 0x0002;
    const SERVICE_KERNEL_DRIVER: DWORD = 0x00000001;
    const SERVICE_FILE_SYSTEM_DRIVER: DWORD = 0x00000002;
    const SERVICE_WIN32_OWN_PROCESS: DWORD = 0x00000010;
    const SERVICE_WIN32_SHARE_PROCESS: DWORD = 0x00000020;
    const SERVICE_INTERACTIVE_PROCESS: DWORD = 0x00000100;
    const SERVICE_WIN32: DWORD = SERVICE_WIN32_OWN_PROCESS | SERVICE_WIN32_SHARE_PROCESS;
    const SERVICE_DRIVER: DWORD = SERVICE_KERNEL_DRIVER | SERVICE_FILE_SYSTEM_DRIVER;
    const SERVICE_BOOT_START: DWORD = 0x00000000;
    const SERVICE_SYSTEM_START: DWORD = 0x00000001;
    const SERVICE_AUTO_START: DWORD = 0x00000002;
    const SERVICE_DEMAND_START: DWORD = 0x00000003;
    const SERVICE_DISABLED: DWORD = 0x00000004;
    const SERVICE_ERROR_IGNORE: DWORD = 0x00000000;
    const SERVICE_ERROR_NORMAL: DWORD = 0x00000001;
    const SERVICE_ERROR_SEVERE: DWORD = 0x00000002;
    const SERVICE_ERROR_CRITICAL: DWORD = 0x00000003;

    let hManager: PMANAGER_HANDLE;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut lpService: PSERVICE = core::ptr::null_mut();
    let mut hServiceHandle: SC_HANDLE = core::ptr::null_mut();
    let mut lpImagePath: LPWSTR = core::ptr::null_mut();
    let mut lpClearTextPassword: LPWSTR = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let lpObjectName: LPWSTR;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hManager).Handle.DesiredAccess, SC_MANAGER_CREATE_SERVICE) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    if *lpServiceName == 0 {
        return ERROR_INVALID_NAME;
    }

    if *lpBinaryPathName == 0 {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwServiceType != SERVICE_KERNEL_DRIVER)
        && (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER)
        && ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS)
        && ((dwServiceType & !SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwStartType != SERVICE_BOOT_START)
        && (dwStartType != SERVICE_SYSTEM_START)
        && (dwStartType != SERVICE_AUTO_START)
        && (dwStartType != SERVICE_DEMAND_START)
        && (dwStartType != SERVICE_DISABLED)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwStartType == SERVICE_BOOT_START) || (dwStartType == SERVICE_SYSTEM_START) {
        if (dwServiceType != SERVICE_KERNEL_DRIVER) && (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER) {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if (dwErrorControl != SERVICE_ERROR_IGNORE)
        && (dwErrorControl != SERVICE_ERROR_NORMAL)
        && (dwErrorControl != SERVICE_ERROR_SEVERE)
        && (dwErrorControl != SERVICE_ERROR_CRITICAL)
    {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwServiceType == (SERVICE_WIN32_OWN_PROCESS | SERVICE_INTERACTIVE_PROCESS)) && !lpServiceStartName.is_null() {
        if _wcsicmp(lpServiceStartName, w!("LocalSystem").as_ptr() as LPCWSTR) != 0 {
            return ERROR_INVALID_PARAMETER;
        }
    }

    if !lpdwTagId.is_null() && (lpLoadOrderGroup.is_null() || *lpLoadOrderGroup == 0) {
        return ERROR_INVALID_PARAMETER;
    }

    ScmLockDatabaseExclusive();

    lpService = ScmGetServiceEntryByName(lpServiceName);
    if !lpService.is_null() {
        ScmUnlockDatabase();

        if (*lpService).bDeleted != 0 {
            return ERROR_SERVICE_MARKED_FOR_DELETE;
        }

        return ERROR_SERVICE_EXISTS;
    }

    if !lpDisplayName.is_null() && !ScmGetServiceEntryByDisplayName(lpDisplayName).is_null() {
        ScmUnlockDatabase();

        return ERROR_DUPLICATE_SERVICE_NAME;
    }

    'done: loop {
        if dwServiceType & SERVICE_DRIVER != 0 {
            dwError = ScmCanonDriverImagePath(dwStartType, lpBinaryPathName, &mut lpImagePath);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        } else {
            if dwStartType == SERVICE_BOOT_START || dwStartType == SERVICE_SYSTEM_START {
                ScmUnlockDatabase();

                return ERROR_INVALID_PARAMETER;
            }
        }

        dwError = ScmCreateNewServiceRecord(lpServiceName, &mut lpService, dwServiceType, dwStartType);
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        (*lpService).dwErrorControl = dwErrorControl;

        if !lpDisplayName.is_null()
            && *lpDisplayName != 0
            && _wcsicmp((*lpService).lpDisplayName, lpDisplayName) != 0
        {
            (*lpService).lpDisplayName = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (wcslen(lpDisplayName) + 1) * core::mem::size_of::<u16>(),
            ) as LPWSTR;
            if (*lpService).lpDisplayName.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'done;
            }
            wcscpy((*lpService).lpDisplayName, lpDisplayName);
        }

        if !lpLoadOrderGroup.is_null() && *lpLoadOrderGroup != 0 {
            dwError = ScmSetServiceGroup(lpService, lpLoadOrderGroup as LPWSTR);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpdwTagId.is_null() {
            dwError = ScmAssignNewTag(lpService);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if dwServiceType & SERVICE_WIN32 != 0 {
            dwError = ScmCreateDefaultServiceSD(core::ptr::addr_of_mut!((*lpService).pSecurityDescriptor));
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        dwError = ScmCreateServiceKey(lpServiceName, KEY_WRITE, &mut hServiceKey);
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if !lpDisplayName.is_null() && *lpDisplayName != 0 {
            RegSetValueExW(
                hServiceKey,
                w!("DisplayName").as_ptr() as LPCWSTR,
                0,
                REG_SZ,
                lpDisplayName as *const u8,
                ((wcslen(lpDisplayName) + 1) * core::mem::size_of::<u16>()) as DWORD,
            );
        }

        dwError = RegSetValueExW(
            hServiceKey,
            w!("Type").as_ptr() as LPCWSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(dwServiceType) as *const u8,
            core::mem::size_of::<DWORD>() as DWORD,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = RegSetValueExW(
            hServiceKey,
            w!("Start").as_ptr() as LPCWSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(dwStartType) as *const u8,
            core::mem::size_of::<DWORD>() as DWORD,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = RegSetValueExW(
            hServiceKey,
            w!("ErrorControl").as_ptr() as LPCWSTR,
            0,
            REG_DWORD,
            core::ptr::addr_of!(dwErrorControl) as *const u8,
            core::mem::size_of::<DWORD>() as DWORD,
        ) as DWORD;
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if dwServiceType & SERVICE_WIN32 != 0 {
            dwError = RegSetValueExW(
                hServiceKey,
                w!("ImagePath").as_ptr() as LPCWSTR,
                0,
                REG_EXPAND_SZ,
                lpBinaryPathName as *const u8,
                ((wcslen(lpBinaryPathName) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        } else if dwServiceType & SERVICE_DRIVER != 0 {
            dwError = RegSetValueExW(
                hServiceKey,
                w!("ImagePath").as_ptr() as LPCWSTR,
                0,
                REG_EXPAND_SZ,
                lpImagePath as *const u8,
                ((wcslen(lpImagePath) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
                if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpLoadOrderGroup.is_null() && *lpLoadOrderGroup != 0 {
            dwError = RegSetValueExW(
                hServiceKey,
                w!("Group").as_ptr() as LPCWSTR,
                0,
                REG_SZ,
                lpLoadOrderGroup as *const u8,
                ((wcslen(lpLoadOrderGroup) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpdwTagId.is_null() {
            dwError = RegSetValueExW(
                hServiceKey,
                w!("Tag").as_ptr() as LPCWSTR,
                0,
                REG_DWORD,
                core::ptr::addr_of!((*lpService).dwTag) as *const u8,
                core::mem::size_of::<DWORD>() as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if !lpDependencies.is_null() && *lpDependencies != 0 {
            dwError = ScmWriteDependencies(hServiceKey, lpDependencies as LPCWSTR, dwDependSize);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        if dwServiceType & SERVICE_WIN32 != 0 {
            lpObjectName = if !lpServiceStartName.is_null() {
                lpServiceStartName as LPWSTR
            } else {
                w!("LocalSystem").as_ptr() as LPWSTR
            };
            dwError = RegSetValueExW(
                hServiceKey,
                w!("ObjectName").as_ptr() as LPCWSTR,
                0,
                REG_SZ,
                lpObjectName as *const u8,
                ((wcslen(lpObjectName) + 1) * core::mem::size_of::<u16>()) as DWORD,
            ) as DWORD;
            if dwError != ERROR_SUCCESS {
                break 'done;
            }

            if !lpPassword.is_null() && *(lpPassword as LPWSTR) != 0 {
                dwError = ScmDecryptPassword(hSCManager, lpPassword, dwPwSize, &mut lpClearTextPassword);
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }

                dwError = ScmSetServicePassword(lpServiceName, lpClearTextPassword);
                if dwError != ERROR_SUCCESS {
                    break 'done;
                }
            }

            dwError = ScmWriteSecurityDescriptor(hServiceKey, (*lpService).pSecurityDescriptor);
            if dwError != ERROR_SUCCESS {
                break 'done;
            }
        }

        dwError = ScmCreateServiceHandle(lpService, &mut hServiceHandle);
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = ScmCheckAccess(hServiceHandle, dwDesiredAccess);
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        ScmReferenceService(lpService);

        ScmGenerateServiceTag(lpService);

        break 'done;
    }

    ScmUnlockDatabase();

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    if !lpClearTextPassword.is_null() {
        SecureZeroMemory(
            lpClearTextPassword as *mut core::ffi::c_void,
            (wcslen(lpClearTextPassword) + 1) * core::mem::size_of::<u16>(),
        );
        HeapFree(GetProcessHeap(), 0, lpClearTextPassword as *mut core::ffi::c_void);
    }

    if dwError == ERROR_SUCCESS {
        *lpServiceHandle = hServiceHandle as SC_RPC_HANDLE;

        if !lpdwTagId.is_null() {
            *lpdwTagId = (*lpService).dwTag;
        }
    } else {
        if !lpService.is_null() && !(*lpService).lpServiceName.is_null() {
            HeapFree(GetProcessHeap(), 0, (*lpService).lpDisplayName as *mut core::ffi::c_void);
        }

        if !hServiceHandle.is_null() {
            HeapFree(GetProcessHeap(), 0, hServiceHandle as *mut core::ffi::c_void);
        }
    }

    if !lpImagePath.is_null() {
        HeapFree(GetProcessHeap(), 0, lpImagePath as *mut core::ffi::c_void);
    }

    if dwError != ERROR_SUCCESS {
        DbgPrint(
            b"[SERVICES] RCreateServiceW(%S) FAILED: %lu (late failure leaves a half-created service - upstream FIXME)\n\0".as_ptr() as *const i8,
            lpServiceName, dwError,
        );
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumDependentServicesW(
    hService: SC_RPC_HANDLE,
    dwServiceState: DWORD,
    lpServices: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut dwServicesReturned: DWORD = 0;
    let mut dwServiceCount: DWORD;
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let hSvc: PSERVICE_HANDLE;
    let mut lpService: PSERVICE;
    let mut lpServicesArray: *mut PSERVICE = core::ptr::null_mut();
    let mut lpServicesPtr: LPENUM_SERVICE_STATUSW;
    let mut lpStr: LPWSTR;

    *pcbBytesNeeded = 0;
    *lpServicesReturned = 0;

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    lpService = (*hSvc).ServiceEntry;

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SC_MANAGER_ENUMERATE_SERVICE) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services").as_ptr() as LPCWSTR,
        0,
        KEY_READ,
        &mut hServicesKey,
    ) as DWORD;
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    'done: {
        dwError = Int_EnumDependentServicesW(
            hServicesKey,
            lpService,
            dwServiceState,
            core::ptr::null_mut(),
            pcbBytesNeeded,
            &mut dwServicesReturned,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if lpServices.is_null() || cbBufSize < *pcbBytesNeeded {
            dwError = ERROR_MORE_DATA;
            break 'done;
        }

        lpServicesArray = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((dwServicesReturned + 1) as usize) * core::mem::size_of::<PSERVICE>(),
        ) as *mut PSERVICE;
        if lpServicesArray.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        dwServicesReturned = 0;
        *pcbBytesNeeded = 0;

        dwError = Int_EnumDependentServicesW(
            hServicesKey,
            lpService,
            dwServiceState,
            lpServicesArray,
            pcbBytesNeeded,
            &mut dwServicesReturned,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        lpServicesPtr = lpServices as LPENUM_SERVICE_STATUSW;
        lpStr = (lpServices.add(
            (dwServicesReturned as usize) * core::mem::size_of::<ENUM_SERVICE_STATUSW>(),
        )) as LPWSTR;

        dwServiceCount = 0;
        while dwServiceCount < dwServicesReturned {
            lpService = *lpServicesArray.add(dwServiceCount as usize);

            memcpy(
                core::ptr::addr_of_mut!((*lpServicesPtr).ServiceStatus) as *mut _,
                core::ptr::addr_of!((*lpService).Status) as *const _,
                core::mem::size_of::<SERVICE_STATUS>(),
            );

            wcscpy(lpStr, (*lpService).lpDisplayName);
            (*lpServicesPtr).lpDisplayName =
                ((lpStr as ULONG_PTR) - (lpServices as ULONG_PTR)) as LPWSTR;
            lpStr = lpStr.add((wcslen((*lpService).lpDisplayName) + 1) as usize);

            wcscpy(lpStr, (*lpService).lpServiceName);
            (*lpServicesPtr).lpServiceName =
                ((lpStr as ULONG_PTR) - (lpServices as ULONG_PTR)) as LPWSTR;
            lpStr = lpStr.add((wcslen((*lpService).lpServiceName) + 1) as usize);

            lpServicesPtr = lpServicesPtr.add(1);

            dwServiceCount += 1;
        }

        *lpServicesReturned = dwServicesReturned;
    }

    if !lpServicesArray.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServicesArray as *mut _);
    }

    RegCloseKey(hServicesKey);

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumServicesStatusW(
    hSCManager: SC_RPC_HANDLE,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    lpBuffer: LPBYTE,
    dwBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
    lpResumeHandle: LPBOUNDED_DWORD_256K,
) -> DWORD {
    REnumServiceGroupW(
        hSCManager,
        dwServiceType,
        dwServiceState,
        lpBuffer,
        dwBufSize,
        pcbBytesNeeded,
        lpServicesReturned,
        lpResumeHandle,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn ROpenSCManagerW(
    _lpMachineName: LPWSTR,
    lpDatabaseName: LPWSTR,
    dwDesiredAccess: DWORD,
    lpScHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    let mut dwError: DWORD;
    let mut hHandle: SC_HANDLE = core::ptr::null_mut();

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if lpScHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    dwError = ScmCreateManagerHandle(lpDatabaseName, &mut hHandle);
    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    dwError = ScmCheckAccess(hHandle, dwDesiredAccess | SC_MANAGER_CONNECT);
    if dwError != ERROR_SUCCESS {
        HeapFree(GetProcessHeap(), 0, hHandle);
        return dwError;
    }

    *lpScHandle = hHandle as SC_RPC_HANDLE;

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn ROpenServiceW(
    hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPWSTR,
    dwDesiredAccess: DWORD,
    lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    let lpService: PSERVICE;
    let hManager: PMANAGER_HANDLE;
    let mut hHandle: SC_HANDLE = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if lpServiceHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    if lpServiceName.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    ScmLockDatabaseExclusive();

    'done: {
        lpService = ScmGetServiceEntryByName(lpServiceName);
        if lpService.is_null() {
            dwError = ERROR_SERVICE_DOES_NOT_EXIST;
            break 'done;
        }

        dwError = ScmCreateServiceHandle(lpService, &mut hHandle);
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = ScmCheckAccess(hHandle, dwDesiredAccess);
        if dwError != ERROR_SUCCESS {
            HeapFree(GetProcessHeap(), 0, hHandle);
            break 'done;
        }

        ScmReferenceService(lpService);

        *lpServiceHandle = hHandle as SC_RPC_HANDLE;
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceConfigW(
    hService: SC_RPC_HANDLE,
    lpBuf: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_8K,
) -> DWORD {
    let lpServiceConfig: LPQUERY_SERVICE_CONFIGW = lpBuf as LPQUERY_SERVICE_CONFIGW;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let mut lpService: PSERVICE = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut lpImagePath: LPWSTR = core::ptr::null_mut();
    let mut lpServiceStartName: LPWSTR = core::ptr::null_mut();
    let mut lpDependencies: LPWSTR = core::ptr::null_mut();
    let mut dwDependenciesLength: DWORD = 0;
    let mut dwRequiredSize: DWORD = 0;
    let mut lpStr: LPWSTR;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_CONFIG) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    'Done: {
        dwError = ScmOpenServiceKey(
            (*lpService).lpServiceName,
            KEY_READ,
            core::ptr::addr_of_mut!(hServiceKey),
        );
        if dwError != ERROR_SUCCESS {
            break 'Done;
        }

        dwError = ScmReadString(
            hServiceKey,
            wstr!("ImagePath\0").as_ptr(),
            core::ptr::addr_of_mut!(lpImagePath),
        );
        if dwError != ERROR_SUCCESS {
            break 'Done;
        }

        ScmReadString(
            hServiceKey,
            wstr!("ObjectName\0").as_ptr(),
            core::ptr::addr_of_mut!(lpServiceStartName),
        );

        ScmReadDependencies(
            hServiceKey,
            core::ptr::addr_of_mut!(lpDependencies),
            core::ptr::addr_of_mut!(dwDependenciesLength),
        );

        dwRequiredSize = core::mem::size_of::<QUERY_SERVICE_CONFIGW>() as DWORD;

        if !lpImagePath.is_null() {
            dwRequiredSize += ((wcslen(lpImagePath) + 1) * core::mem::size_of::<u16>()) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<u16>() as DWORD;
        }

        if !(*lpService).lpGroup.is_null() && !(*(*lpService).lpGroup).lpGroupName.is_null() {
            dwRequiredSize += ((wcslen((*(*lpService).lpGroup).lpGroupName) + 1)
                * core::mem::size_of::<u16>()) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<u16>() as DWORD;
        }

        if !lpDependencies.is_null() {
            dwRequiredSize += dwDependenciesLength * core::mem::size_of::<u16>() as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<u16>() as DWORD;
        }

        if !lpServiceStartName.is_null() {
            dwRequiredSize +=
                ((wcslen(lpServiceStartName) + 1) * core::mem::size_of::<u16>()) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<u16>() as DWORD;
        }

        if !(*lpService).lpDisplayName.is_null() {
            dwRequiredSize +=
                ((wcslen((*lpService).lpDisplayName) + 1) * core::mem::size_of::<u16>()) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<u16>() as DWORD;
        }

        if lpServiceConfig.is_null() || cbBufSize < dwRequiredSize {
            dwError = ERROR_INSUFFICIENT_BUFFER;
        } else {
            (*lpServiceConfig).dwServiceType = (*lpService).Status.dwServiceType;
            (*lpServiceConfig).dwStartType = (*lpService).dwStartType;
            (*lpServiceConfig).dwErrorControl = (*lpService).dwErrorControl;
            (*lpServiceConfig).dwTagId = (*lpService).dwTag;

            lpStr = lpServiceConfig.add(1) as LPWSTR;

            if !lpImagePath.is_null() {
                wcscpy(lpStr, lpImagePath);
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpBinaryPathName =
                ((lpStr as usize) - (lpServiceConfig as usize)) as LPWSTR;
            lpStr = lpStr.add(wcslen(lpStr) + 1);

            if !(*lpService).lpGroup.is_null() && !(*(*lpService).lpGroup).lpGroupName.is_null() {
                wcscpy(lpStr, (*(*lpService).lpGroup).lpGroupName);
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpLoadOrderGroup =
                ((lpStr as usize) - (lpServiceConfig as usize)) as LPWSTR;
            lpStr = lpStr.add(wcslen(lpStr) + 1);

            if !lpDependencies.is_null() {
                memcpy(
                    lpStr as *mut core::ffi::c_void,
                    lpDependencies as *const core::ffi::c_void,
                    (dwDependenciesLength as usize) * core::mem::size_of::<u16>(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpDependencies =
                ((lpStr as usize) - (lpServiceConfig as usize)) as LPWSTR;
            if !lpDependencies.is_null() {
                lpStr = lpStr.add(dwDependenciesLength as usize);
            } else {
                lpStr = lpStr.add(wcslen(lpStr) + 1);
            }

            if !lpServiceStartName.is_null() {
                wcscpy(lpStr, lpServiceStartName);
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpServiceStartName =
                ((lpStr as usize) - (lpServiceConfig as usize)) as LPWSTR;
            lpStr = lpStr.add(wcslen(lpStr) + 1);

            if !(*lpService).lpDisplayName.is_null() {
                wcscpy(lpStr, (*lpService).lpDisplayName);
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpDisplayName =
                ((lpStr as usize) - (lpServiceConfig as usize)) as LPWSTR;
        }

        if !pcbBytesNeeded.is_null() {
            *pcbBytesNeeded = dwRequiredSize;
        }
    }

    ScmUnlockDatabase();

    if !lpImagePath.is_null() {
        HeapFree(GetProcessHeap(), 0, lpImagePath as *mut core::ffi::c_void);
    }

    if !lpServiceStartName.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServiceStartName as *mut core::ffi::c_void);
    }

    if !lpDependencies.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDependencies as *mut core::ffi::c_void);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceLockStatusW(
    hSCManager: SC_RPC_HANDLE,
    lpBuf: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_4K,
) -> DWORD {
    let lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSW = lpBuf as LPQUERY_SERVICE_LOCK_STATUSW;
    let hMgr: PMANAGER_HANDLE;
    let dwRequiredSize: DWORD;

    if lpLockStatus.is_null() || pcbBytesNeeded.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    hMgr = ScmGetServiceManagerFromHandle(hSCManager);
    if hMgr.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hMgr).Handle.DesiredAccess, SC_MANAGER_QUERY_LOCK_STATUS) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    dwRequiredSize = (core::mem::size_of::<QUERY_SERVICE_LOCK_STATUSW>()
        + core::mem::size_of::<WCHAR>()) as DWORD;
    *pcbBytesNeeded = dwRequiredSize;

    if cbBufSize < dwRequiredSize {
        return ERROR_INSUFFICIENT_BUFFER;
    }

    ScmQueryServiceLockStatusW(lpLockStatus);

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RStartServiceW(
    hService: SC_RPC_HANDLE,
    argc: DWORD,
    argv: LPSTRING_PTRSW,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_START) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if (*lpService).dwStartType == SERVICE_DISABLED {
        return ERROR_SERVICE_DISABLED;
    }

    if (*lpService).bDeleted != 0 {
        return ERROR_SERVICE_MARKED_FOR_DELETE;
    }

    dwError = ScmStartService(lpService, argc, argv as *mut PCWSTR);

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RGetServiceDisplayNameW(
    _hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPCWSTR,
    lpDisplayName: LPWSTR,
    lpcchBuffer: *mut DWORD,
) -> DWORD {
    let lpService: PSERVICE;
    let lpSvcDisplayName: LPCWSTR;
    let dwLength: DWORD;
    let dwError: DWORD;

    lpService = ScmGetServiceEntryByName(lpServiceName);
    if lpService.is_null() {
        return ERROR_SERVICE_DOES_NOT_EXIST;
    }

    if !(*lpService).lpDisplayName.is_null() {
        lpSvcDisplayName = (*lpService).lpDisplayName;
    } else {
        lpSvcDisplayName = (*lpService).lpServiceName;
    }

    dwLength = wcslen(lpSvcDisplayName) as DWORD;

    if *lpcchBuffer > dwLength {
        if !lpDisplayName.is_null() {
            wcscpy(lpDisplayName, lpSvcDisplayName);
        }

        dwError = ERROR_SUCCESS;
    } else {
        dwError = ERROR_INSUFFICIENT_BUFFER;
    }

    *lpcchBuffer = dwLength;

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RGetServiceKeyNameW(
    _hSCManager: SC_RPC_HANDLE,
    lpDisplayName: LPCWSTR,
    lpServiceName: LPWSTR,
    lpcchBuffer: *mut DWORD,
) -> DWORD {
    let lpService: PSERVICE;
    let dwLength: DWORD;
    let dwError: DWORD;

    lpService = ScmGetServiceEntryByDisplayName(lpDisplayName);
    if lpService.is_null() {
        return ERROR_SERVICE_DOES_NOT_EXIST;
    }

    dwLength = wcslen((*lpService).lpServiceName) as DWORD;

    if *lpcchBuffer > dwLength {
        if !lpServiceName.is_null() {
            wcscpy(lpServiceName, (*lpService).lpServiceName);
        }

        dwError = ERROR_SUCCESS;
    } else {
        dwError = ERROR_INSUFFICIENT_BUFFER;
    }

    *lpcchBuffer = dwLength;

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScSetServiceBitsA(
    hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
    dwServiceBits: DWORD,
    bSetBitsOn: core::ffi::c_int,
    bUpdateImmediately: core::ffi::c_int,
    lpString: *mut core::ffi::c_char,
) -> DWORD {
    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if !lpString.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    RI_ScSetServiceBitsW(
        hServiceStatus,
        dwServiceBits,
        bSetBitsOn,
        bUpdateImmediately,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn RChangeServiceConfigA(
    hService: SC_RPC_HANDLE,
    dwServiceType: DWORD,
    dwStartType: DWORD,
    dwErrorControl: DWORD,
    lpBinaryPathName: LPSTR,
    lpLoadOrderGroup: LPSTR,
    lpdwTagId: LPDWORD,
    lpDependencies: LPBYTE,
    _dwDependSize: DWORD,
    lpServiceStartName: LPSTR,
    lpPassword: LPBYTE,
    dwPwSize: DWORD,
    lpDisplayName: LPSTR,
) -> DWORD {

    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut lpBinaryPathNameW: LPWSTR = core::ptr::null_mut();
    let mut lpLoadOrderGroupW: LPWSTR = core::ptr::null_mut();
    let mut lpDependenciesW: LPWSTR = core::ptr::null_mut();
    let mut lpServiceStartNameW: LPWSTR = core::ptr::null_mut();
    let mut lpDisplayNameW: LPWSTR = core::ptr::null_mut();
    let mut dwDependenciesLength: DWORD = 0;
    let mut cchLength: usize;
    let mut len: core::ffi::c_int;
    let mut lpStr: LPCSTR;

    'cleanup: {
        if !lpBinaryPathName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, core::ptr::null_mut(), 0);
            lpBinaryPathNameW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            if lpBinaryPathNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, lpBinaryPathNameW, len);
        }

        if !lpLoadOrderGroup.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, core::ptr::null_mut(), 0);
            lpLoadOrderGroupW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            if lpLoadOrderGroupW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, lpLoadOrderGroupW, len);
        }

        if !lpDependencies.is_null() {
            lpStr = lpDependencies as LPCSTR;
            while *lpStr != 0 {
                cchLength = strlen(lpStr) + 1;
                dwDependenciesLength += cchLength as DWORD;
                lpStr = lpStr.add(cchLength);
            }
            dwDependenciesLength += 1;

            lpDependenciesW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                dwDependenciesLength as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            if lpDependenciesW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(
                CP_ACP,
                0,
                lpDependencies as LPCSTR,
                dwDependenciesLength as core::ffi::c_int,
                lpDependenciesW,
                dwDependenciesLength as core::ffi::c_int,
            );
        }

        if !lpServiceStartName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, core::ptr::null_mut(), 0);
            lpServiceStartNameW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            if lpServiceStartNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, lpServiceStartNameW, len);
        }

        if !lpDisplayName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, core::ptr::null_mut(), 0);
            lpDisplayNameW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                len as SIZE_T * core::mem::size_of::<WCHAR>() as SIZE_T,
            ) as LPWSTR;
            if lpDisplayNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, lpDisplayNameW, len);
        }

        dwError = RChangeServiceConfigW(
            hService,
            dwServiceType,
            dwStartType,
            dwErrorControl,
            lpBinaryPathNameW,
            lpLoadOrderGroupW,
            lpdwTagId,
            lpDependenciesW as LPBYTE,
            dwDependenciesLength,
            lpServiceStartNameW,
            lpPassword,
            dwPwSize,
            lpDisplayNameW,
        );
    }

    if !lpBinaryPathNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBinaryPathNameW as LPVOID);
    }

    if !lpLoadOrderGroupW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpLoadOrderGroupW as LPVOID);
    }

    if !lpDependenciesW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDependenciesW as LPVOID);
    }

    if !lpServiceStartNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServiceStartNameW as LPVOID);
    }

    if !lpDisplayNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDisplayNameW as LPVOID);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RCreateServiceA(
    hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPSTR,
    lpDisplayName: LPSTR,
    dwDesiredAccess: DWORD,
    dwServiceType: DWORD,
    dwStartType: DWORD,
    dwErrorControl: DWORD,
    lpBinaryPathName: LPSTR,
    lpLoadOrderGroup: LPSTR,
    lpdwTagId: LPDWORD,
    lpDependencies: LPBYTE,
    _dwDependSize: DWORD,
    lpServiceStartName: LPSTR,
    lpPassword: LPBYTE,
    dwPwSize: DWORD,
    lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {

    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut lpServiceNameW: LPWSTR = core::ptr::null_mut();
    let mut lpDisplayNameW: LPWSTR = core::ptr::null_mut();
    let mut lpBinaryPathNameW: LPWSTR = core::ptr::null_mut();
    let mut lpLoadOrderGroupW: LPWSTR = core::ptr::null_mut();
    let mut lpDependenciesW: LPWSTR = core::ptr::null_mut();
    let mut lpServiceStartNameW: LPWSTR = core::ptr::null_mut();
    let mut dwDependenciesLength: DWORD = 0;
    let mut cchLength: usize;
    let mut len: i32;
    let mut lpStr: LPCSTR;

    'cleanup: {
        if !lpServiceName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpServiceName, -1, core::ptr::null_mut(), 0);
            lpServiceNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpServiceNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpServiceName, -1, lpServiceNameW, len);
        }

        if !lpDisplayName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, core::ptr::null_mut(), 0);
            lpDisplayNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpDisplayNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, lpDisplayNameW, len);
        }

        if !lpBinaryPathName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, core::ptr::null_mut(), 0);
            lpBinaryPathNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpBinaryPathNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, lpBinaryPathNameW, len);
        }

        if !lpLoadOrderGroup.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, core::ptr::null_mut(), 0);
            lpLoadOrderGroupW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpLoadOrderGroupW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, lpLoadOrderGroupW, len);
        }

        if !lpDependencies.is_null() {
            lpStr = lpDependencies as LPCSTR;
            while *lpStr != 0 {
                cchLength = strlen(lpStr) + 1;
                dwDependenciesLength += cchLength as DWORD;
                lpStr = lpStr.add(cchLength);
            }
            dwDependenciesLength += 1;

            lpDependenciesW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (dwDependenciesLength as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpDependenciesW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpDependencies as LPCSTR, dwDependenciesLength as i32, lpDependenciesW, dwDependenciesLength as i32);
        }

        if !lpServiceStartName.is_null() {
            len = MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, core::ptr::null_mut(), 0);
            lpServiceStartNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (len as usize * core::mem::size_of::<u16>()) as SIZE_T) as LPWSTR;
            if lpServiceStartNameW.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, lpServiceStartNameW, len);
        }

        dwError = RCreateServiceW(
            hSCManager,
            lpServiceNameW,
            lpDisplayNameW,
            dwDesiredAccess,
            dwServiceType,
            dwStartType,
            dwErrorControl,
            lpBinaryPathNameW,
            lpLoadOrderGroupW,
            lpdwTagId,
            lpDependenciesW as LPBYTE,
            dwDependenciesLength,
            lpServiceStartNameW,
            lpPassword,
            dwPwSize,
            lpServiceHandle,
        );
    }

    if !lpServiceNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServiceNameW as *mut core::ffi::c_void);
    }

    if !lpDisplayNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDisplayNameW as *mut core::ffi::c_void);
    }

    if !lpBinaryPathNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBinaryPathNameW as *mut core::ffi::c_void);
    }

    if !lpLoadOrderGroupW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpLoadOrderGroupW as *mut core::ffi::c_void);
    }

    if !lpDependenciesW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDependenciesW as *mut core::ffi::c_void);
    }

    if !lpServiceStartNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServiceStartNameW as *mut core::ffi::c_void);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumDependentServicesA(
    hService: SC_RPC_HANDLE,
    dwServiceState: DWORD,
    lpServices: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut dwServicesReturned: DWORD = 0;
    let mut dwServiceCount: DWORD;
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let hSvc: PSERVICE_HANDLE;
    let mut lpService: PSERVICE = core::ptr::null_mut();
    let mut lpServicesArray: *mut PSERVICE = core::ptr::null_mut();
    let mut lpServicesPtr: LPENUM_SERVICE_STATUSA = core::ptr::null_mut();
    let mut lpStr: LPSTR;

    *pcbBytesNeeded = 0;
    *lpServicesReturned = 0;

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    lpService = (*hSvc).ServiceEntry;

    if RtlAreAllAccessesGranted(
        (*hSvc).Handle.DesiredAccess,
        SC_MANAGER_ENUMERATE_SERVICE,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    dwError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("System\\CurrentControlSet\\Services").as_ptr(),
        0,
        KEY_READ,
        &mut hServicesKey,
    ) as DWORD;

    if dwError != ERROR_SUCCESS {
        return dwError;
    }

    'done: {
        dwError = Int_EnumDependentServicesW(
            hServicesKey,
            lpService,
            dwServiceState,
            core::ptr::null_mut(),
            pcbBytesNeeded,
            &mut dwServicesReturned,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if lpServices.is_null() || cbBufSize < *pcbBytesNeeded {
            dwError = ERROR_MORE_DATA;
            break 'done;
        }

        lpServicesArray = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((dwServicesReturned + 1) as usize) * core::mem::size_of::<PSERVICE>(),
        ) as *mut PSERVICE;
        if lpServicesArray.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        dwServicesReturned = 0;
        *pcbBytesNeeded = 0;

        dwError = Int_EnumDependentServicesW(
            hServicesKey,
            lpService,
            dwServiceState,
            lpServicesArray,
            pcbBytesNeeded,
            &mut dwServicesReturned,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        lpServicesPtr = lpServices as LPENUM_SERVICE_STATUSA;
        lpStr = lpServices.add(
            (dwServicesReturned as usize) * core::mem::size_of::<ENUM_SERVICE_STATUSA>(),
        ) as LPSTR;

        dwServiceCount = 0;
        while dwServiceCount < dwServicesReturned {
            lpService = *lpServicesArray.add(dwServiceCount as usize);

            memcpy(
                core::ptr::addr_of_mut!((*lpServicesPtr).ServiceStatus) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*lpService).Status) as *const core::ffi::c_void,
                core::mem::size_of::<SERVICE_STATUS>(),
            );

            WideCharToMultiByte(
                CP_ACP,
                0,
                (*lpService).lpDisplayName,
                -1,
                lpStr,
                wcslen((*lpService).lpDisplayName) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*lpServicesPtr).lpDisplayName =
                (lpStr as ULONG_PTR - lpServices as ULONG_PTR) as LPSTR;
            lpStr = lpStr.add(strlen(lpStr) + 1);

            WideCharToMultiByte(
                CP_ACP,
                0,
                (*lpService).lpServiceName,
                -1,
                lpStr,
                wcslen((*lpService).lpServiceName) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );
            (*lpServicesPtr).lpServiceName =
                (lpStr as ULONG_PTR - lpServices as ULONG_PTR) as LPSTR;
            lpStr = lpStr.add(strlen(lpStr) + 1);

            lpServicesPtr = lpServicesPtr.add(1);
            dwServiceCount += 1;
        }

        *lpServicesReturned = dwServicesReturned;
    }

    if !lpServicesArray.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServicesArray as *mut core::ffi::c_void);
    }

    RegCloseKey(hServicesKey);

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumServicesStatusA(
    hSCManager: SC_RPC_HANDLE,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    lpBuffer: LPBYTE,
    dwBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
    lpResumeHandle: LPBOUNDED_DWORD_256K,
) -> DWORD {
    let mut lpStatusPtrW: LPENUM_SERVICE_STATUSW = core::ptr::null_mut();
    let mut lpStatusPtrIncrW: LPENUM_SERVICE_STATUSW;
    let mut lpStatusPtrA: LPENUM_SERVICE_STATUSA;
    let mut lpStringPtrW: LPWSTR;
    let mut lpStringPtrA: LPSTR;
    let dwError: DWORD;
    let mut dwServiceCount: DWORD;

    if pcbBytesNeeded.is_null() || lpServicesReturned.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    if dwBufSize > 0 && !lpBuffer.is_null() {
        lpStatusPtrW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwBufSize as usize) as LPENUM_SERVICE_STATUSW;
        if lpStatusPtrW.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }
    }

    dwError = REnumServicesStatusW(
        hSCManager,
        dwServiceType,
        dwServiceState,
        lpStatusPtrW as LPBYTE,
        dwBufSize,
        pcbBytesNeeded,
        lpServicesReturned,
        lpResumeHandle,
    );

    'done: {
        if *lpServicesReturned == 0 {
            break 'done;
        }

        lpStatusPtrIncrW = lpStatusPtrW;
        lpStatusPtrA = lpBuffer as LPENUM_SERVICE_STATUSA;
        lpStringPtrA = (lpBuffer as ULONG_PTR
            + (*lpServicesReturned as ULONG_PTR)
                * core::mem::size_of::<ENUM_SERVICE_STATUSA>() as ULONG_PTR)
            as LPSTR;
        lpStringPtrW = (lpStatusPtrW as ULONG_PTR
            + (*lpServicesReturned as ULONG_PTR)
                * core::mem::size_of::<ENUM_SERVICE_STATUSW>() as ULONG_PTR)
            as LPWSTR;

        dwServiceCount = 0;
        while dwServiceCount < *lpServicesReturned {
            WideCharToMultiByte(
                CP_ACP,
                0,
                lpStringPtrW,
                -1,
                lpStringPtrA,
                wcslen(lpStringPtrW) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );

            (*lpStatusPtrA).lpServiceName =
                (lpStringPtrA as ULONG_PTR - lpBuffer as ULONG_PTR) as LPSTR;
            lpStringPtrA = lpStringPtrA.add(wcslen(lpStringPtrW) + 1);
            lpStringPtrW = lpStringPtrW.add(wcslen(lpStringPtrW) + 1);

            WideCharToMultiByte(
                CP_ACP,
                0,
                lpStringPtrW,
                -1,
                lpStringPtrA,
                wcslen(lpStringPtrW) as i32,
                core::ptr::null(),
                core::ptr::null_mut(),
            );

            (*lpStatusPtrA).lpDisplayName =
                (lpStringPtrA as ULONG_PTR - lpBuffer as ULONG_PTR) as LPSTR;
            lpStringPtrA = lpStringPtrA.add(wcslen(lpStringPtrW) + 1);
            lpStringPtrW = lpStringPtrW.add(wcslen(lpStringPtrW) + 1);

            memcpy(
                core::ptr::addr_of_mut!((*lpStatusPtrA).ServiceStatus) as *mut _,
                core::ptr::addr_of!((*lpStatusPtrIncrW).ServiceStatus) as *const _,
                core::mem::size_of::<SERVICE_STATUS>(),
            );

            lpStatusPtrIncrW = lpStatusPtrIncrW.add(1);
            lpStatusPtrA = lpStatusPtrA.add(1);

            dwServiceCount += 1;
        }
    }

    if !lpStatusPtrW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpStatusPtrW as _);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn ROpenSCManagerA(
    lpMachineName: LPSTR,
    lpDatabaseName: LPSTR,
    dwDesiredAccess: DWORD,
    lpScHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    let mut MachineName: _UNICODE_STRING = core::mem::zeroed();
    let mut DatabaseName: _UNICODE_STRING = core::mem::zeroed();
    let dwError: DWORD;

    if !lpMachineName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut MachineName, lpMachineName);
    }

    if !lpDatabaseName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut DatabaseName, lpDatabaseName);
    }

    dwError = ROpenSCManagerW(
        if !lpMachineName.is_null() { MachineName.Buffer } else { core::ptr::null_mut() },
        if !lpDatabaseName.is_null() { DatabaseName.Buffer } else { core::ptr::null_mut() },
        dwDesiredAccess,
        lpScHandle,
    );

    if !lpMachineName.is_null() {
        RtlFreeUnicodeString(&mut MachineName);
    }

    if !lpDatabaseName.is_null() {
        RtlFreeUnicodeString(&mut DatabaseName);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn ROpenServiceA(
    hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPSTR,
    dwDesiredAccess: DWORD,
    lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    let mut ServiceName: _UNICODE_STRING = core::mem::zeroed();
    let dwError: DWORD;

    if !lpServiceName.is_null() {
        RtlCreateUnicodeStringFromAsciiz(
            core::ptr::addr_of_mut!(ServiceName),
            lpServiceName as *mut _,
        );
    }

    dwError = ROpenServiceW(
        hSCManager,
        if !lpServiceName.is_null() {
            ServiceName.Buffer
        } else {
            core::ptr::null_mut()
        },
        dwDesiredAccess,
        lpServiceHandle,
    );

    if !lpServiceName.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(ServiceName));
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceConfigA(
    hService: SC_RPC_HANDLE,
    lpBuf: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_8K,
) -> DWORD {
    let lpServiceConfig: LPQUERY_SERVICE_CONFIGA = lpBuf as LPQUERY_SERVICE_CONFIGA;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut lpImagePath: LPWSTR = core::ptr::null_mut();
    let mut lpServiceStartName: LPWSTR = core::ptr::null_mut();
    let mut lpDependencies: LPWSTR = core::ptr::null_mut();
    let mut dwDependenciesLength: DWORD = 0;
    let mut dwRequiredSize: DWORD = 0;
    let mut lpStr: LPSTR;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_CONFIG) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    'done: loop {
        dwError = ScmOpenServiceKey(
            (*lpService).lpServiceName,
            KEY_READ,
            &mut hServiceKey,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        dwError = ScmReadString(
            hServiceKey,
            w!("ImagePath").as_ptr() as LPWSTR,
            &mut lpImagePath,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        ScmReadString(
            hServiceKey,
            w!("ObjectName").as_ptr() as LPWSTR,
            &mut lpServiceStartName,
        );

        ScmReadDependencies(hServiceKey, &mut lpDependencies, &mut dwDependenciesLength);

        dwRequiredSize = core::mem::size_of::<QUERY_SERVICE_CONFIGA>() as DWORD;

        if !lpImagePath.is_null() {
            dwRequiredSize += (wcslen(lpImagePath) + 1) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<CHAR>() as DWORD;
        }

        if !(*lpService).lpGroup.is_null() && !(*(*lpService).lpGroup).lpGroupName.is_null() {
            dwRequiredSize += (wcslen((*(*lpService).lpGroup).lpGroupName) + 1) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<CHAR>() as DWORD;
        }

        if !lpDependencies.is_null() {
            dwRequiredSize += dwDependenciesLength;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<CHAR>() as DWORD;
        }

        if !lpServiceStartName.is_null() {
            dwRequiredSize += (wcslen(lpServiceStartName) + 1) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<CHAR>() as DWORD;
        }

        if !(*lpService).lpDisplayName.is_null() {
            dwRequiredSize += (wcslen((*lpService).lpDisplayName) + 1) as DWORD;
        } else {
            dwRequiredSize += 2 * core::mem::size_of::<CHAR>() as DWORD;
        }

        if lpServiceConfig.is_null() || cbBufSize < dwRequiredSize {
            dwError = ERROR_INSUFFICIENT_BUFFER;
        } else {
            (*lpServiceConfig).dwServiceType = (*lpService).Status.dwServiceType;
            (*lpServiceConfig).dwStartType = (*lpService).dwStartType;
            (*lpServiceConfig).dwErrorControl = (*lpService).dwErrorControl;
            (*lpServiceConfig).dwTagId = (*lpService).dwTag;

            lpStr = lpServiceConfig.add(1) as LPSTR;

            if !lpImagePath.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpImagePath,
                    -1,
                    lpStr,
                    (wcslen(lpImagePath) + 1) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpBinaryPathName =
                (lpStr as ULONG_PTR - lpServiceConfig as ULONG_PTR) as LPSTR;
            lpStr = lpStr.add(strlen(lpStr) + 1);

            if !(*lpService).lpGroup.is_null() && !(*(*lpService).lpGroup).lpGroupName.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*(*lpService).lpGroup).lpGroupName,
                    -1,
                    lpStr,
                    (wcslen((*(*lpService).lpGroup).lpGroupName) + 1) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpLoadOrderGroup =
                (lpStr as ULONG_PTR - lpServiceConfig as ULONG_PTR) as LPSTR;
            lpStr = lpStr.add(strlen(lpStr) + 1);

            if !lpDependencies.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpDependencies,
                    dwDependenciesLength as i32,
                    lpStr,
                    dwDependenciesLength as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpDependencies =
                (lpStr as ULONG_PTR - lpServiceConfig as ULONG_PTR) as LPSTR;
            if !lpDependencies.is_null() {
                lpStr = lpStr.add(dwDependenciesLength as usize);
            } else {
                lpStr = lpStr.add(strlen(lpStr) + 1);
            }

            if !lpServiceStartName.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpServiceStartName,
                    -1,
                    lpStr,
                    (wcslen(lpServiceStartName) + 1) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpServiceStartName =
                (lpStr as ULONG_PTR - lpServiceConfig as ULONG_PTR) as LPSTR;
            lpStr = lpStr.add(strlen(lpStr) + 1);

            if !(*lpService).lpDisplayName.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    (*lpService).lpDisplayName,
                    -1,
                    lpStr,
                    (wcslen((*lpService).lpDisplayName) + 1) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
            } else {
                *lpStr = 0;
            }

            (*lpServiceConfig).lpDisplayName =
                (lpStr as ULONG_PTR - lpServiceConfig as ULONG_PTR) as LPSTR;
        }

        if !pcbBytesNeeded.is_null() {
            *pcbBytesNeeded = dwRequiredSize;
        }

        break 'done;
    }

    ScmUnlockDatabase();

    if !lpImagePath.is_null() {
        HeapFree(GetProcessHeap(), 0, lpImagePath as _);
    }

    if !lpServiceStartName.is_null() {
        HeapFree(GetProcessHeap(), 0, lpServiceStartName as _);
    }

    if !lpDependencies.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDependencies as _);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceLockStatusA(
    hSCManager: SC_RPC_HANDLE,
    lpBuf: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_4K,
) -> DWORD {
    let lpLockStatus: LPQUERY_SERVICE_LOCK_STATUSA = lpBuf as LPQUERY_SERVICE_LOCK_STATUSA;
    let hMgr: PMANAGER_HANDLE;
    let dwRequiredSize: DWORD;

    if lpLockStatus.is_null() || pcbBytesNeeded.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    hMgr = ScmGetServiceManagerFromHandle(hSCManager);
    if hMgr.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted(
        (*hMgr).Handle.DesiredAccess,
        SC_MANAGER_QUERY_LOCK_STATUS,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    dwRequiredSize = (core::mem::size_of::<QUERY_SERVICE_LOCK_STATUSA>()
        + core::mem::size_of::<CHAR>()) as DWORD;
    *pcbBytesNeeded = dwRequiredSize;

    if cbBufSize < dwRequiredSize {
        return ERROR_INSUFFICIENT_BUFFER;
    }

    ScmQueryServiceLockStatusA(lpLockStatus);

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RStartServiceA(
    hService: SC_RPC_HANDLE,
    argc: DWORD,
    argv: LPSTRING_PTRSA,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut pVector: *mut PWSTR = core::ptr::null_mut();
    let mut i: DWORD;
    let mut dwLength: DWORD;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_START) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if (*lpService).dwStartType == SERVICE_DISABLED {
        return ERROR_SERVICE_DISABLED;
    }

    if (*lpService).bDeleted != 0 {
        return ERROR_SERVICE_MARKED_FOR_DELETE;
    }

    'done: {
        if argc > 0 {
            pVector = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (argc as usize) * core::mem::size_of::<PWSTR>(),
            ) as *mut PWSTR;
            if pVector.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY;
            }

            i = 0;
            while i < argc {
                dwLength = MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*argv.offset(i as isize)).StringPtr,
                    -1,
                    core::ptr::null_mut(),
                    0,
                ) as DWORD;

                *pVector.offset(i as isize) = HeapAlloc(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    (dwLength as usize) * core::mem::size_of::<WCHAR>(),
                ) as PWSTR;
                if (*pVector.offset(i as isize)).is_null() {
                    dwError = ERROR_NOT_ENOUGH_MEMORY;
                    break 'done;
                }

                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*argv.offset(i as isize)).StringPtr,
                    -1,
                    *pVector.offset(i as isize),
                    dwLength as i32,
                );

                i += 1;
            }
        }

        dwError = ScmStartService(lpService, argc, pVector as *mut PCWSTR);
    }

    if !pVector.is_null() {
        i = 0;
        while i < argc {
            if !(*pVector.offset(i as isize)).is_null() {
                HeapFree(GetProcessHeap(), 0, *pVector.offset(i as isize) as *mut _);
            }
            i += 1;
        }
        HeapFree(GetProcessHeap(), 0, pVector as *mut _);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RGetServiceDisplayNameA(
    _hSCManager: SC_RPC_HANDLE,
    lpServiceName: LPCSTR,
    lpDisplayName: LPSTR,
    lpcchBuffer: LPBOUNDED_DWORD_4K,
) -> DWORD {
    let mut lpService: PSERVICE = core::ptr::null_mut();
    let lpSvcDisplayName: LPCWSTR;
    let lpServiceNameW: LPWSTR;
    let mut dwLength: DWORD;

    if lpServiceName != core::ptr::null_mut() {
        dwLength = (strlen(lpServiceName) + 1) as DWORD;
        lpServiceNameW = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwLength as usize * core::mem::size_of::<WCHAR>(),
        ) as LPWSTR;
        if lpServiceNameW.is_null() {
            return ERROR_NOT_ENOUGH_MEMORY;
        }

        MultiByteToWideChar(CP_ACP, 0, lpServiceName, -1, lpServiceNameW, dwLength as i32);

        lpService = ScmGetServiceEntryByName(lpServiceNameW);

        HeapFree(GetProcessHeap(), 0, lpServiceNameW as *mut _);
    }

    if lpService == core::ptr::null_mut() {
        return ERROR_SERVICE_DOES_NOT_EXIST;
    }

    if !(*lpService).lpDisplayName.is_null() {
        lpSvcDisplayName = (*lpService).lpDisplayName;
    } else {
        lpSvcDisplayName = (*lpService).lpServiceName;
    }

    dwLength = wcslen(lpSvcDisplayName) as DWORD;

    if *lpcchBuffer > dwLength {
        if lpDisplayName != core::ptr::null_mut()
            && WideCharToMultiByte(
                CP_ACP,
                0,
                lpSvcDisplayName,
                -1,
                lpDisplayName,
                *lpcchBuffer as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
        {
            dwLength = WideCharToMultiByte(
                CP_ACP,
                0,
                lpSvcDisplayName,
                dwLength as i32,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            *lpDisplayName = 0;
            *lpcchBuffer = dwLength;
            return ERROR_INSUFFICIENT_BUFFER;
        }

        ERROR_SUCCESS
    } else {
        *lpcchBuffer = dwLength * core::mem::size_of::<WCHAR>() as DWORD;
        ERROR_INSUFFICIENT_BUFFER
    }
}

#[no_mangle]
pub unsafe extern "system" fn RGetServiceKeyNameA(
    _hSCManager: SC_RPC_HANDLE,
    lpDisplayName: LPCSTR,
    lpServiceName: LPSTR,
    lpcchBuffer: LPBOUNDED_DWORD_4K,
) -> DWORD {
    let lpService: PSERVICE;
    let lpDisplayNameW: LPWSTR;
    let mut dwLength: DWORD;

    dwLength = (strlen(lpDisplayName) + 1) as DWORD;
    lpDisplayNameW = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (dwLength as usize) * core::mem::size_of::<u16>(),
    ) as LPWSTR;
    if lpDisplayNameW.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        lpDisplayName,
        -1,
        lpDisplayNameW,
        dwLength as i32,
    );

    lpService = ScmGetServiceEntryByDisplayName(lpDisplayNameW);

    HeapFree(GetProcessHeap(), 0, lpDisplayNameW as *mut _);

    if lpService.is_null() {
        return ERROR_SERVICE_DOES_NOT_EXIST;
    }

    dwLength = wcslen((*lpService).lpServiceName) as DWORD;

    if *lpcchBuffer > dwLength {
        if !lpServiceName.is_null()
            && WideCharToMultiByte(
                CP_ACP,
                0,
                (*lpService).lpServiceName,
                -1,
                lpServiceName,
                *lpcchBuffer as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) == 0
        {
            dwLength = WideCharToMultiByte(
                CP_ACP,
                0,
                (*lpService).lpServiceName,
                dwLength as i32,
                core::ptr::null_mut(),
                0,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            ) as DWORD;
            *lpServiceName = 0;
            *lpcchBuffer = dwLength;
            return ERROR_INSUFFICIENT_BUFFER;
        }

        ERROR_SUCCESS
    } else {
        *lpcchBuffer = dwLength * core::mem::size_of::<u16>() as DWORD;
        ERROR_INSUFFICIENT_BUFFER
    }
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScGetCurrentGroupStateW(
    hSCManager: SC_RPC_HANDLE,
    lpLoadOrderGroup: LPWSTR,
    lpState: LPDWORD,
) -> DWORD {
    let hManager: PMANAGER_HANDLE;
    let pServiceGroup: PSERVICE_GROUP;
    let mut dwError: DWORD = ERROR_SUCCESS;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted(
        (*hManager).Handle.DesiredAccess,
        SC_MANAGER_ENUMERATE_SERVICE,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    ScmLockDatabaseShared();

    pServiceGroup = ScmGetServiceGroupByName(lpLoadOrderGroup);
    if pServiceGroup.is_null() {
        dwError = ERROR_SERVICE_DOES_NOT_EXIST;
    } else {
        *lpState = 0;
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumServiceGroupW(
    hSCManager: SC_RPC_HANDLE,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
    lpResumeIndex: LPBOUNDED_DWORD_256K,
    pszGroupName: LPCWSTR,
) -> DWORD {
    let hManager: PMANAGER_HANDLE;
    let lpService: PSERVICE;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;
    let mut dwState: DWORD;
    let mut dwRequiredSize: DWORD;
    let mut dwServiceCount: DWORD;
    let mut dwSize: DWORD;
    let mut dwLastResumeCount: DWORD = 0;
    let mut lpStatusPtr: LPENUM_SERVICE_STATUSW;
    let mut lpStringPtr: LPWSTR;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if pcbBytesNeeded.is_null() || lpServicesReturned.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    *pcbBytesNeeded = 0;
    *lpServicesReturned = 0;

    if (dwServiceType == 0) || ((dwServiceType & !SERVICE_TYPE_ALL) != 0) {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwServiceState == 0) || ((dwServiceState & !SERVICE_STATE_ALL) != 0) {
        return ERROR_INVALID_PARAMETER;
    }

    if RtlAreAllAccessesGranted(
        (*hManager).Handle.DesiredAccess,
        SC_MANAGER_ENUMERATE_SERVICE,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    if !lpResumeIndex.is_null() {
        dwLastResumeCount = *lpResumeIndex;
    }

    ScmLockDatabaseShared();

    lpService = ScmGetServiceEntryByResumeCount(dwLastResumeCount);

    'done: {
        if lpService.is_null() {
            dwError = ERROR_SUCCESS;
            break 'done;
        }

        dwRequiredSize = 0;
        dwServiceCount = 0;

        ServiceEntry = core::ptr::addr_of_mut!((*lpService).ServiceListEntry);
        'enum1: while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            'cont: {
                if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                    break 'cont;
                }

                dwState = SERVICE_ACTIVE;
                if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                    dwState = SERVICE_INACTIVE;
                }

                if (dwState & dwServiceState) == 0 {
                    break 'cont;
                }

                if !pszGroupName.is_null() {
                    if *pszGroupName == 0 {
                        if !(*CurrentService).lpGroup.is_null() {
                            break 'cont;
                        }
                    } else {
                        if (*CurrentService).lpGroup.is_null()
                            || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                        {
                            break 'cont;
                        }
                    }
                }

                dwSize = core::mem::size_of::<ENUM_SERVICE_STATUSW>() as DWORD
                    + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD
                    + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD;

                if dwRequiredSize + dwSize > cbBufSize {
                    break 'enum1;
                }

                dwRequiredSize += dwSize;
                dwServiceCount += 1;
                dwLastResumeCount = (*CurrentService).dwResumeCount;
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            'cont: {
                if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                    break 'cont;
                }

                dwState = SERVICE_ACTIVE;
                if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                    dwState = SERVICE_INACTIVE;
                }

                if (dwState & dwServiceState) == 0 {
                    break 'cont;
                }

                if !pszGroupName.is_null() {
                    if *pszGroupName == 0 {
                        if !(*CurrentService).lpGroup.is_null() {
                            break 'cont;
                        }
                    } else {
                        if (*CurrentService).lpGroup.is_null()
                            || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                        {
                            break 'cont;
                        }
                    }
                }

                dwRequiredSize += core::mem::size_of::<ENUM_SERVICE_STATUSW>() as DWORD
                    + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD
                    + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD;

                dwError = ERROR_MORE_DATA;
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        if !lpResumeIndex.is_null() {
            *lpResumeIndex = dwLastResumeCount;
        }

        *lpServicesReturned = dwServiceCount;
        *pcbBytesNeeded = dwRequiredSize;

        lpStatusPtr = lpBuffer as LPENUM_SERVICE_STATUSW;
        lpStringPtr = (lpBuffer as usize
            + (dwServiceCount as usize) * core::mem::size_of::<ENUM_SERVICE_STATUSW>())
            as LPWSTR;

        dwRequiredSize = 0;
        ServiceEntry = core::ptr::addr_of_mut!((*lpService).ServiceListEntry);
        'enum2: while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            'cont: {
                if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                    break 'cont;
                }

                dwState = SERVICE_ACTIVE;
                if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                    dwState = SERVICE_INACTIVE;
                }

                if (dwState & dwServiceState) == 0 {
                    break 'cont;
                }

                if !pszGroupName.is_null() {
                    if *pszGroupName == 0 {
                        if !(*CurrentService).lpGroup.is_null() {
                            break 'cont;
                        }
                    } else {
                        if (*CurrentService).lpGroup.is_null()
                            || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                        {
                            break 'cont;
                        }
                    }
                }

                dwSize = core::mem::size_of::<ENUM_SERVICE_STATUSW>() as DWORD
                    + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD
                    + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>())
                        as DWORD;

                if dwRequiredSize + dwSize > cbBufSize {
                    break 'enum2;
                }

                wcscpy(lpStringPtr, (*CurrentService).lpServiceName);
                (*lpStatusPtr).lpServiceName =
                    (lpStringPtr as usize - lpBuffer as usize) as LPWSTR;
                lpStringPtr = lpStringPtr.add(wcslen((*CurrentService).lpServiceName) + 1);

                wcscpy(lpStringPtr, (*CurrentService).lpDisplayName);
                (*lpStatusPtr).lpDisplayName =
                    (lpStringPtr as usize - lpBuffer as usize) as LPWSTR;
                lpStringPtr = lpStringPtr.add(wcslen((*CurrentService).lpDisplayName) + 1);

                memcpy(
                    core::ptr::addr_of_mut!((*lpStatusPtr).ServiceStatus) as *mut _,
                    core::ptr::addr_of!((*CurrentService).Status) as *const _,
                    core::mem::size_of::<SERVICE_STATUS>(),
                );

                lpStatusPtr = lpStatusPtr.add(1);
                dwRequiredSize += dwSize;
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        if dwError == ERROR_SUCCESS {
            *pcbBytesNeeded = 0;
            if !lpResumeIndex.is_null() {
                *lpResumeIndex = 0;
            }
        }
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RChangeServiceConfig2A(
    hService: SC_RPC_HANDLE,
    Info: SC_RPC_CONFIG_INFOA,
) -> DWORD {
    let mut InfoW: SC_RPC_CONFIG_INFOW = core::mem::zeroed();
    let dwRet: DWORD;
    let dwLength: DWORD;
    let mut ptr: PVOID = core::ptr::null_mut();

    if (Info.dwInfoLevel < SERVICE_CONFIG_DESCRIPTION)
        || (Info.dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS)
    {
        return ERROR_INVALID_LEVEL;
    }

    InfoW.dwInfoLevel = Info.dwInfoLevel;

    if InfoW.dwInfoLevel == SERVICE_CONFIG_DESCRIPTION {
        let lpServiceDescriptionW: LPSERVICE_DESCRIPTIONW;
        let lpServiceDescriptionA: LPSERVICE_DESCRIPTIONA;

        lpServiceDescriptionA = Info.__bindgen_anon_1.psd;

        if !lpServiceDescriptionA.is_null()
            && !(*lpServiceDescriptionA).lpDescription.is_null()
        {
            dwLength = ((strlen((*lpServiceDescriptionA).lpDescription) + 1)
                * core::mem::size_of::<WCHAR>()) as DWORD;

            lpServiceDescriptionW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                dwLength as usize + core::mem::size_of::<SERVICE_DESCRIPTIONW>(),
            ) as LPSERVICE_DESCRIPTIONW;
            if lpServiceDescriptionW.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY;
            }

            (*lpServiceDescriptionW).lpDescription =
                lpServiceDescriptionW.add(1) as LPWSTR;

            MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpServiceDescriptionA).lpDescription,
                -1,
                (*lpServiceDescriptionW).lpDescription,
                dwLength as i32,
            );

            ptr = lpServiceDescriptionW as PVOID;
            InfoW.__bindgen_anon_1.psd = lpServiceDescriptionW;
        }
    } else if Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
        let lpServiceFailureActionsW: LPSERVICE_FAILURE_ACTIONSW;
        let lpServiceFailureActionsA: LPSERVICE_FAILURE_ACTIONSA;
        let mut dwRebootLen: DWORD = 0;
        let mut dwCommandLen: DWORD = 0;
        let mut dwActionArrayLen: DWORD = 0;
        let mut lpStr: LPWSTR;

        lpServiceFailureActionsA = Info.__bindgen_anon_1.psfa;

        if !lpServiceFailureActionsA.is_null() {
            if !(*lpServiceFailureActionsA).lpRebootMsg.is_null() {
                dwRebootLen = ((strlen((*lpServiceFailureActionsA).lpRebootMsg) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD;
            }
            if !(*lpServiceFailureActionsA).lpCommand.is_null() {
                dwCommandLen = ((strlen((*lpServiceFailureActionsA).lpCommand) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD;
            }

            if (*lpServiceFailureActionsA).cActions > 0
                && !(*lpServiceFailureActionsA).lpsaActions.is_null()
            {
                dwActionArrayLen = (*lpServiceFailureActionsA).cActions
                    * core::mem::size_of::<SC_ACTION>() as DWORD;
            }

            dwLength = core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD
                + dwActionArrayLen
                + dwRebootLen
                + dwCommandLen;

            lpServiceFailureActionsW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                dwLength as usize,
            ) as LPSERVICE_FAILURE_ACTIONSW;
            if lpServiceFailureActionsW.is_null() {
                return ERROR_NOT_ENOUGH_MEMORY;
            }

            (*lpServiceFailureActionsW).dwResetPeriod =
                (*lpServiceFailureActionsA).dwResetPeriod;
            (*lpServiceFailureActionsW).cActions =
                (*lpServiceFailureActionsA).cActions;

            if dwActionArrayLen > 0 {
                (*lpServiceFailureActionsW).lpsaActions =
                    lpServiceFailureActionsW.add(1) as LPSC_ACTION;

                RtlCopyMemory(
                    (*lpServiceFailureActionsW).lpsaActions as PVOID,
                    (*lpServiceFailureActionsA).lpsaActions as PVOID,
                    dwActionArrayLen as usize,
                );
            } else {
                (*lpServiceFailureActionsW).lpsaActions = core::ptr::null_mut();
            }

            lpStr = ((lpServiceFailureActionsW.add(1) as ULONG_PTR)
                + dwActionArrayLen as ULONG_PTR) as LPWSTR;

            (*lpServiceFailureActionsW).lpRebootMsg = core::ptr::null_mut();
            (*lpServiceFailureActionsW).lpCommand = core::ptr::null_mut();

            if dwRebootLen != 0 {
                (*lpServiceFailureActionsW).lpRebootMsg = lpStr;

                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*lpServiceFailureActionsA).lpRebootMsg,
                    -1,
                    (*lpServiceFailureActionsW).lpRebootMsg,
                    dwRebootLen as i32,
                );

                lpStr = lpStr.add((dwRebootLen / core::mem::size_of::<WCHAR>() as DWORD) as usize);
            }

            if dwCommandLen != 0 {
                (*lpServiceFailureActionsW).lpCommand = lpStr;

                MultiByteToWideChar(
                    CP_ACP,
                    0,
                    (*lpServiceFailureActionsA).lpCommand,
                    -1,
                    (*lpServiceFailureActionsW).lpCommand,
                    dwCommandLen as i32,
                );
            }

            ptr = lpServiceFailureActionsW as PVOID;
            InfoW.__bindgen_anon_1.psfa = lpServiceFailureActionsW;
        }
    }

    dwRet = RChangeServiceConfig2W(hService, InfoW);

    HeapFree(GetProcessHeap(), 0, ptr);

    dwRet
}

unsafe fn ScmSetFailureActions(
    hServiceKey: HKEY,
    lpFailureActions: LPSERVICE_FAILURE_ACTIONSW,
) -> DWORD {
    let mut lpReadBuffer: LPSERVICE_FAILURE_ACTIONSW = core::ptr::null_mut();
    let mut lpWriteBuffer: LPSERVICE_FAILURE_ACTIONSW = core::ptr::null_mut();
    let mut dwRequiredSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut dwError: DWORD;

    if lpFailureActions.is_null() {
        return ERROR_SUCCESS;
    }

    dwError = RegQueryValueExW(
        hServiceKey,
        wstr!("FailureActions").as_ptr(),
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut dwRequiredSize,
    ) as DWORD;
    if dwError != ERROR_SUCCESS
        && dwError != ERROR_MORE_DATA
        && dwError != ERROR_FILE_NOT_FOUND
    {
        return dwError;
    }

    dwRequiredSize = if dwType == REG_BINARY {
        core::cmp::max(
            core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD,
            dwRequiredSize,
        )
    } else {
        core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD
    };

    lpReadBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwRequiredSize as usize)
        as LPSERVICE_FAILURE_ACTIONSW;
    if lpReadBuffer.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    'done: {
        if dwError != ERROR_FILE_NOT_FOUND && dwType == REG_BINARY {
            dwError = RegQueryValueExW(
                hServiceKey,
                wstr!("FailureActions").as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                lpReadBuffer as LPBYTE,
                &mut dwRequiredSize,
            ) as DWORD;
            if dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND {
                break 'done;
            }

            if dwRequiredSize < core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD {
                dwRequiredSize = core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD;
            }
        } else {
            dwError = ERROR_INVALID_DATA;
        }

        if dwError == ERROR_SUCCESS {
            (*lpReadBuffer).cActions = core::cmp::min(
                (*lpReadBuffer).cActions,
                (dwRequiredSize - core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD)
                    / core::mem::size_of::<SC_ACTION>() as DWORD,
            );
            (*lpReadBuffer).lpsaActions = if (*lpReadBuffer).cActions > 0 {
                lpReadBuffer.add(1) as LPSC_ACTION
            } else {
                core::ptr::null_mut()
            };
        } else {
            (*lpReadBuffer).dwResetPeriod = 0;
            (*lpReadBuffer).cActions = 0;
            (*lpReadBuffer).lpsaActions = core::ptr::null_mut();
        }

        (*lpReadBuffer).lpRebootMsg = core::ptr::null_mut();
        (*lpReadBuffer).lpCommand = core::ptr::null_mut();

        dwRequiredSize = core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD;

        if (*lpFailureActions).lpsaActions.is_null() {
            dwRequiredSize +=
                (*lpReadBuffer).cActions * core::mem::size_of::<SC_ACTION>() as DWORD;
        } else {
            dwRequiredSize +=
                (*lpFailureActions).cActions * core::mem::size_of::<SC_ACTION>() as DWORD;
        }

        lpWriteBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwRequiredSize as usize)
            as LPSERVICE_FAILURE_ACTIONSW;
        if lpWriteBuffer.is_null() {
            dwError = ERROR_NOT_ENOUGH_MEMORY;
            break 'done;
        }

        (*lpWriteBuffer).lpRebootMsg = core::ptr::null_mut();
        (*lpWriteBuffer).lpCommand = core::ptr::null_mut();
        (*lpWriteBuffer).lpsaActions = core::ptr::null_mut();

        if (*lpFailureActions).lpsaActions.is_null() {
            (*lpWriteBuffer).dwResetPeriod = (*lpReadBuffer).dwResetPeriod;
            (*lpWriteBuffer).cActions = (*lpReadBuffer).cActions;

            if !(*lpReadBuffer).lpsaActions.is_null() {
                memmove(
                    lpWriteBuffer.add(1) as *mut core::ffi::c_void,
                    (*lpReadBuffer).lpsaActions as *const core::ffi::c_void,
                    ((*lpReadBuffer).cActions as usize)
                        * core::mem::size_of::<SC_ACTION>(),
                );
            }
        } else {
            if (*lpFailureActions).cActions > 0 {
                (*lpWriteBuffer).dwResetPeriod = (*lpFailureActions).dwResetPeriod;
                (*lpWriteBuffer).cActions = (*lpFailureActions).cActions;

                memmove(
                    lpWriteBuffer.add(1) as *mut core::ffi::c_void,
                    (*lpFailureActions).lpsaActions as *const core::ffi::c_void,
                    ((*lpFailureActions).cActions as usize)
                        * core::mem::size_of::<SC_ACTION>(),
                );
            } else {
                (*lpWriteBuffer).dwResetPeriod = 0;
                (*lpWriteBuffer).cActions = 0;
            }
        }

        dwError = RegSetValueExW(
            hServiceKey,
            wstr!("FailureActions").as_ptr(),
            0,
            REG_BINARY,
            lpWriteBuffer as LPBYTE,
            dwRequiredSize,
        ) as DWORD;

        if dwError == ERROR_SUCCESS {
            if !(*lpFailureActions).lpRebootMsg.is_null() {
                if *(*lpFailureActions).lpRebootMsg == 0 {
                    RegDeleteValueW(hServiceKey, wstr!("RebootMessage").as_ptr());
                } else {
                    RegSetValueExW(
                        hServiceKey,
                        wstr!("RebootMessage").as_ptr(),
                        0,
                        REG_SZ,
                        (*lpFailureActions).lpRebootMsg as LPBYTE,
                        ((wcslen((*lpFailureActions).lpRebootMsg) + 1)
                            * core::mem::size_of::<WCHAR>()) as DWORD,
                    );
                }
            }

            if !(*lpFailureActions).lpCommand.is_null() {
                if *(*lpFailureActions).lpCommand == 0 {
                    RegDeleteValueW(hServiceKey, wstr!("FailureCommand").as_ptr());
                } else {
                    RegSetValueExW(
                        hServiceKey,
                        wstr!("FailureCommand").as_ptr(),
                        0,
                        REG_SZ,
                        (*lpFailureActions).lpCommand as LPBYTE,
                        ((wcslen((*lpFailureActions).lpCommand) + 1)
                            * core::mem::size_of::<WCHAR>()) as DWORD,
                    );
                }
            }
        }
    }

    if !lpWriteBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, lpWriteBuffer as *mut core::ffi::c_void);
    }

    if !lpReadBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, lpReadBuffer as *mut core::ffi::c_void);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RChangeServiceConfig2W(
    hService: SC_RPC_HANDLE,
    Info: SC_RPC_CONFIG_INFOW,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut RequiredAccess: ACCESS_MASK = SERVICE_CHANGE_CONFIG;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if Info.dwInfoLevel < SERVICE_CONFIG_DESCRIPTION
        || Info.dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS
    {
        return ERROR_INVALID_LEVEL;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
        RequiredAccess |= SERVICE_START;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, RequiredAccess) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    if Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
        /* FIXME: Check if the caller has the SE_SHUTDOWN_NAME privilege */
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
        if (*lpService).Status.dwServiceType & SERVICE_DRIVER != 0 {
            return ERROR_CANNOT_DETECT_DRIVER_FAILURE;
        }
    }

    ScmLockDatabaseExclusive();

    'done: {
        if (*lpService).bDeleted != 0 {
            dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
            break 'done;
        }

        dwError = ScmOpenServiceKey(
            (*lpService).szServiceName.as_mut_ptr(),
            KEY_READ | KEY_SET_VALUE,
            &mut hServiceKey,
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if Info.dwInfoLevel == SERVICE_CONFIG_DESCRIPTION {
            let lpServiceDescription: LPSERVICE_DESCRIPTIONW =
                Info.__bindgen_anon_1.psd as LPSERVICE_DESCRIPTIONW;

            if !lpServiceDescription.is_null()
                && !(*lpServiceDescription).lpDescription.is_null()
            {
                if *(*lpServiceDescription).lpDescription == 0 {
                    dwError = RegDeleteValueW(
                        hServiceKey,
                        w!("Description").as_ptr() as *const u16,
                    ) as DWORD;

                    if dwError == ERROR_FILE_NOT_FOUND {
                        dwError = ERROR_SUCCESS;
                    }
                } else {
                    dwError = RegSetValueExW(
                        hServiceKey,
                        w!("Description").as_ptr() as *const u16,
                        0,
                        REG_SZ,
                        (*lpServiceDescription).lpDescription as LPBYTE,
                        ((wcslen((*lpServiceDescription).lpDescription) + 1)
                            * core::mem::size_of::<WCHAR>()) as DWORD,
                    ) as DWORD;
                }
            } else {
                dwError = ERROR_SUCCESS;
            }
        } else if Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
            dwError = ScmSetFailureActions(
                hServiceKey,
                Info.__bindgen_anon_1.psfa as LPSERVICE_FAILURE_ACTIONSW,
            );
        }
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    ScmUnlockDatabase();

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceConfig2A(
    hService: SC_RPC_HANDLE,
    dwInfoLevel: DWORD,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_8K,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut dwRequiredSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut lpDescriptionW: LPWSTR = core::ptr::null_mut();
    let mut lpRebootMessageW: LPWSTR = core::ptr::null_mut();
    let mut lpFailureCommandW: LPWSTR = core::ptr::null_mut();

    if lpBuffer.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if dwInfoLevel < SERVICE_CONFIG_DESCRIPTION || dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS {
        return ERROR_INVALID_LEVEL;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_CONFIG) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    'done: {
        dwError = ScmOpenServiceKey(
            (*lpService).lpServiceName,
            KEY_READ,
            core::ptr::addr_of_mut!(hServiceKey),
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if dwInfoLevel == SERVICE_CONFIG_DESCRIPTION {
            let lpServiceDescription: LPSERVICE_DESCRIPTIONA = lpBuffer as LPSERVICE_DESCRIPTIONA;
            let lpStr: LPSTR;

            dwError = ScmReadString(
                hServiceKey,
                w!("Description").as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(lpDescriptionW),
            );
            if dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND {
                break 'done;
            }

            *pcbBytesNeeded = core::mem::size_of::<SERVICE_DESCRIPTIONA>() as DWORD;
            if dwError == ERROR_SUCCESS {
                *pcbBytesNeeded += ((wcslen(lpDescriptionW) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD;
            }

            if cbBufSize < *pcbBytesNeeded {
                dwError = ERROR_INSUFFICIENT_BUFFER;
                break 'done;
            }

            if dwError == ERROR_SUCCESS {
                lpStr = lpServiceDescription.add(1) as LPSTR;

                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpDescriptionW,
                    -1,
                    lpStr,
                    wcslen(lpDescriptionW) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                (*lpServiceDescription).lpDescription =
                    (lpStr as ULONG_PTR - lpServiceDescription as ULONG_PTR) as LPSTR;
            } else {
                (*lpServiceDescription).lpDescription = core::ptr::null_mut();
                dwError = ERROR_SUCCESS;
            }
        } else if dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
            let lpFailureActions: LPSERVICE_FAILURE_ACTIONSA =
                lpBuffer as LPSERVICE_FAILURE_ACTIONSA;
            let mut lpStr: LPSTR;

            dwError = RegQueryValueExW(
                hServiceKey,
                w!("FailureActions").as_ptr() as LPWSTR,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwRequiredSize),
            ) as DWORD;
            if dwError != ERROR_SUCCESS
                && dwError != ERROR_MORE_DATA
                && dwError != ERROR_FILE_NOT_FOUND
            {
                break 'done;
            }

            dwRequiredSize = if dwType == REG_BINARY {
                max(
                    core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as DWORD,
                    dwRequiredSize,
                )
            } else {
                core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as DWORD
            };

            ScmReadString(
                hServiceKey,
                w!("FailureCommand").as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(lpFailureCommandW),
            );

            ScmReadString(
                hServiceKey,
                w!("RebootMessage").as_ptr() as LPWSTR,
                core::ptr::addr_of_mut!(lpRebootMessageW),
            );

            if !lpRebootMessageW.is_null() {
                dwRequiredSize += ((wcslen(lpRebootMessageW) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD;
            }

            if !lpFailureCommandW.is_null() {
                dwRequiredSize += ((wcslen(lpFailureCommandW) + 1)
                    * core::mem::size_of::<WCHAR>()) as DWORD;
            }

            if cbBufSize < dwRequiredSize {
                *pcbBytesNeeded = dwRequiredSize;
                dwError = ERROR_INSUFFICIENT_BUFFER;
                break 'done;
            }

            if dwError != ERROR_FILE_NOT_FOUND && dwType == REG_BINARY {
                dwError = RegQueryValueExW(
                    hServiceKey,
                    w!("FailureActions").as_ptr() as LPWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    lpFailureActions as LPBYTE,
                    core::ptr::addr_of_mut!(dwRequiredSize),
                ) as DWORD;
                if dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND {
                    break 'done;
                }

                if dwRequiredSize < core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as DWORD {
                    dwRequiredSize = core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as DWORD;
                }
            } else {
                dwError = ERROR_INVALID_DATA;
            }

            if dwError == ERROR_SUCCESS {
                (*lpFailureActions).cActions = min(
                    (*lpFailureActions).cActions,
                    (dwRequiredSize - core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as DWORD)
                        / core::mem::size_of::<SC_ACTION>() as DWORD,
                );

                (*lpFailureActions).lpsaActions = if (*lpFailureActions).cActions > 0 {
                    core::mem::size_of::<SERVICE_FAILURE_ACTIONSA>() as ULONG_PTR as LPSC_ACTION
                } else {
                    core::ptr::null_mut()
                };

                lpStr = (lpFailureActions.add(1) as ULONG_PTR
                    + (*lpFailureActions).cActions as ULONG_PTR
                        * core::mem::size_of::<SC_ACTION>() as ULONG_PTR)
                    as LPSTR;
            } else {
                (*lpFailureActions).dwResetPeriod = 0;
                (*lpFailureActions).cActions = 0;
                (*lpFailureActions).lpsaActions = core::ptr::null_mut();
                lpStr = lpFailureActions.add(1) as LPSTR;
            }

            (*lpFailureActions).lpRebootMsg = core::ptr::null_mut();
            (*lpFailureActions).lpCommand = core::ptr::null_mut();

            if !lpRebootMessageW.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpRebootMessageW,
                    -1,
                    lpStr,
                    wcslen(lpRebootMessageW) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                (*lpFailureActions).lpRebootMsg =
                    (lpStr as ULONG_PTR - lpFailureActions as ULONG_PTR) as LPSTR;
                lpStr = lpStr.add(strlen(lpStr) + 1);
            }

            if !lpFailureCommandW.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpFailureCommandW,
                    -1,
                    lpStr,
                    wcslen(lpFailureCommandW) as i32,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                (*lpFailureActions).lpCommand =
                    (lpStr as ULONG_PTR - lpFailureActions as ULONG_PTR) as LPSTR;
            }

            dwError = ERROR_SUCCESS;
        }
    }

    ScmUnlockDatabase();

    if !lpDescriptionW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDescriptionW as *mut core::ffi::c_void);
    }

    if !lpRebootMessageW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpRebootMessageW as *mut core::ffi::c_void);
    }

    if !lpFailureCommandW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpFailureCommandW as *mut core::ffi::c_void);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceConfig2W(
    hService: SC_RPC_HANDLE,
    dwInfoLevel: DWORD,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_8K,
) -> DWORD {
    let mut dwError: DWORD = ERROR_SUCCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut dwRequiredSize: DWORD = 0;
    let mut dwType: DWORD = 0;
    let mut lpDescription: LPWSTR = core::ptr::null_mut();
    let mut lpRebootMessage: LPWSTR = core::ptr::null_mut();
    let mut lpFailureCommand: LPWSTR = core::ptr::null_mut();

    if lpBuffer.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if dwInfoLevel < SERVICE_CONFIG_DESCRIPTION || dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS {
        return ERROR_INVALID_LEVEL;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_CONFIG) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    'done: {
        dwError = ScmOpenServiceKey(
            (*lpService).lpServiceName,
            KEY_READ,
            core::ptr::addr_of_mut!(hServiceKey),
        );
        if dwError != ERROR_SUCCESS {
            break 'done;
        }

        if dwInfoLevel == SERVICE_CONFIG_DESCRIPTION {
            let lpServiceDescription = lpBuffer as LPSERVICE_DESCRIPTIONW;
            let lpStr: LPWSTR;

            dwError = ScmReadString(
                hServiceKey,
                wstr!("Description").as_ptr(),
                core::ptr::addr_of_mut!(lpDescription),
            );
            if dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND {
                break 'done;
            }

            *pcbBytesNeeded = core::mem::size_of::<SERVICE_DESCRIPTIONW>() as DWORD;
            if dwError == ERROR_SUCCESS {
                *pcbBytesNeeded += ((wcslen(lpDescription) + 1)
                    * core::mem::size_of::<u16>()) as DWORD;
            }

            if cbBufSize < *pcbBytesNeeded {
                dwError = ERROR_INSUFFICIENT_BUFFER;
                break 'done;
            }

            if dwError == ERROR_SUCCESS {
                lpStr = lpServiceDescription.add(1) as LPWSTR;
                wcscpy(lpStr, lpDescription);
                (*lpServiceDescription).lpDescription =
                    ((lpStr as usize) - (lpServiceDescription as usize)) as LPWSTR;
            } else {
                (*lpServiceDescription).lpDescription = core::ptr::null_mut();
                dwError = ERROR_SUCCESS;
            }
        } else if dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS {
            let lpFailureActions = lpBuffer as LPSERVICE_FAILURE_ACTIONSW;
            let mut lpStr: LPWSTR;

            dwError = RegQueryValueExW(
                hServiceKey,
                wstr!("FailureActions").as_ptr(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwType),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(dwRequiredSize),
            ) as DWORD;
            if dwError != ERROR_SUCCESS
                && dwError != ERROR_MORE_DATA
                && dwError != ERROR_FILE_NOT_FOUND
            {
                break 'done;
            }

            dwRequiredSize = if dwType == REG_BINARY {
                max(
                    core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD,
                    dwRequiredSize,
                )
            } else {
                core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD
            };

            ScmReadString(
                hServiceKey,
                wstr!("FailureCommand").as_ptr(),
                core::ptr::addr_of_mut!(lpFailureCommand),
            );

            ScmReadString(
                hServiceKey,
                wstr!("RebootMessage").as_ptr(),
                core::ptr::addr_of_mut!(lpRebootMessage),
            );

            if !lpRebootMessage.is_null() {
                dwRequiredSize +=
                    ((wcslen(lpRebootMessage) + 1) * core::mem::size_of::<u16>()) as DWORD;
            }

            if !lpFailureCommand.is_null() {
                dwRequiredSize +=
                    ((wcslen(lpFailureCommand) + 1) * core::mem::size_of::<u16>()) as DWORD;
            }

            if cbBufSize < dwRequiredSize {
                *pcbBytesNeeded = dwRequiredSize;
                dwError = ERROR_INSUFFICIENT_BUFFER;
                break 'done;
            }

            if dwError != ERROR_FILE_NOT_FOUND && dwType == REG_BINARY {
                dwError = RegQueryValueExW(
                    hServiceKey,
                    wstr!("FailureActions").as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    lpFailureActions as LPBYTE,
                    core::ptr::addr_of_mut!(dwRequiredSize),
                ) as DWORD;
                if dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND {
                    break 'done;
                }

                if dwRequiredSize < core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD {
                    dwRequiredSize = core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD;
                }
            } else {
                dwError = ERROR_INVALID_DATA;
            }

            if dwError == ERROR_SUCCESS {
                (*lpFailureActions).cActions = min(
                    (*lpFailureActions).cActions,
                    (dwRequiredSize - core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as DWORD)
                        / core::mem::size_of::<SC_ACTION>() as DWORD,
                );

                (*lpFailureActions).lpsaActions = if (*lpFailureActions).cActions > 0 {
                    core::mem::size_of::<SERVICE_FAILURE_ACTIONSW>() as LPSC_ACTION
                } else {
                    core::ptr::null_mut()
                };

                lpStr = ((lpFailureActions.add(1) as usize)
                    + (*lpFailureActions).cActions as usize
                        * core::mem::size_of::<SC_ACTION>()) as LPWSTR;
            } else {
                (*lpFailureActions).dwResetPeriod = 0;
                (*lpFailureActions).cActions = 0;
                (*lpFailureActions).lpsaActions = core::ptr::null_mut();
                lpStr = lpFailureActions.add(1) as LPWSTR;
            }

            (*lpFailureActions).lpRebootMsg = core::ptr::null_mut();
            (*lpFailureActions).lpCommand = core::ptr::null_mut();

            if !lpRebootMessage.is_null() {
                wcscpy(lpStr, lpRebootMessage);
                (*lpFailureActions).lpRebootMsg =
                    ((lpStr as usize) - (lpFailureActions as usize)) as LPWSTR;
                lpStr = lpStr.add(wcslen(lpStr) + 1);
            }

            if !lpFailureCommand.is_null() {
                wcscpy(lpStr, lpFailureCommand);
                (*lpFailureActions).lpCommand =
                    ((lpStr as usize) - (lpFailureActions as usize)) as LPWSTR;
            }

            dwError = ERROR_SUCCESS;
        }
    }

    ScmUnlockDatabase();

    if !lpDescription.is_null() {
        HeapFree(GetProcessHeap(), 0, lpDescription as *mut core::ffi::c_void);
    }

    if !lpRebootMessage.is_null() {
        HeapFree(GetProcessHeap(), 0, lpRebootMessage as *mut core::ffi::c_void);
    }

    if !lpFailureCommand.is_null() {
        HeapFree(GetProcessHeap(), 0, lpFailureCommand as *mut core::ffi::c_void);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RQueryServiceStatusEx(
    hService: SC_RPC_HANDLE,
    InfoLevel: SC_STATUS_TYPE,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_8K,
) -> DWORD {
    let lpStatus: LPSERVICE_STATUS_PROCESS;
    let hSvc: PSERVICE_HANDLE;
    let lpService: PSERVICE;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if InfoLevel != SC_STATUS_PROCESS_INFO {
        return ERROR_INVALID_LEVEL;
    }

    *pcbBytesNeeded = core::mem::size_of::<SERVICE_STATUS_PROCESS>() as DWORD;

    if (cbBufSize as usize) < core::mem::size_of::<SERVICE_STATUS_PROCESS>() {
        return ERROR_INSUFFICIENT_BUFFER;
    }

    hSvc = ScmGetServiceFromHandle(hService);
    if hSvc.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted((*hSvc).Handle.DesiredAccess, SERVICE_QUERY_STATUS) == 0 {
        return ERROR_ACCESS_DENIED;
    }

    lpService = (*hSvc).ServiceEntry;
    if lpService.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    ScmLockDatabaseShared();

    lpStatus = lpBuffer as LPSERVICE_STATUS_PROCESS;

    RtlCopyMemory(
        lpStatus as *mut _,
        core::ptr::addr_of!((*lpService).Status) as *const _,
        core::mem::size_of::<SERVICE_STATUS>(),
    );

    if (*lpService).Status.dwCurrentState == SERVICE_STOPPED || (*lpService).lpImage.is_null() {
        (*lpStatus).dwProcessId = 0;
    } else {
        (*lpStatus).dwProcessId = (*(*lpService).lpImage).dwProcessId;
    }

    (*lpStatus).dwServiceFlags = 0;

    ScmUnlockDatabase();

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn REnumServicesStatusExA(
    hSCManager: SC_RPC_HANDLE,
    InfoLevel: SC_ENUM_TYPE,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
    lpResumeIndex: LPBOUNDED_DWORD_256K,
    pszGroupName: LPCSTR,
) -> DWORD {
    let mut lpStatusPtrW: LPENUM_SERVICE_STATUS_PROCESSW = core::ptr::null_mut();
    let mut lpStatusPtrIncrW: LPENUM_SERVICE_STATUS_PROCESSW;
    let mut lpStatusPtrA: LPENUM_SERVICE_STATUS_PROCESSA = core::ptr::null_mut();
    let mut lpStringPtrW: LPWSTR;
    let mut lpStringPtrA: LPSTR;
    let mut pszGroupNameW: LPWSTR = core::ptr::null_mut();
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut dwServiceCount: DWORD;

    if pcbBytesNeeded.is_null() || lpServicesReturned.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    'done: loop {
        if !pszGroupName.is_null() {
            pszGroupNameW = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (strlen(pszGroupName) + 1) * core::mem::size_of::<WCHAR>(),
            ) as LPWSTR;
            if pszGroupNameW.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'done;
            }

            MultiByteToWideChar(
                CP_ACP,
                0,
                pszGroupName,
                -1,
                pszGroupNameW,
                (strlen(pszGroupName) + 1) as i32,
            );
        }

        if cbBufSize > 0 && !lpBuffer.is_null() {
            lpStatusPtrW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbBufSize as usize)
                as LPENUM_SERVICE_STATUS_PROCESSW;
            if lpStatusPtrW.is_null() {
                dwError = ERROR_NOT_ENOUGH_MEMORY;
                break 'done;
            }
        }

        dwError = REnumServicesStatusExW(
            hSCManager,
            InfoLevel,
            dwServiceType,
            dwServiceState,
            lpStatusPtrW as LPBYTE,
            cbBufSize,
            pcbBytesNeeded,
            lpServicesReturned,
            lpResumeIndex,
            pszGroupNameW,
        );

        if *lpServicesReturned == 0 {
            break 'done;
        }

        lpStatusPtrIncrW = lpStatusPtrW;
        lpStatusPtrA = lpBuffer as LPENUM_SERVICE_STATUS_PROCESSA;
        lpStringPtrA = (lpBuffer as usize
            + *lpServicesReturned as usize
                * core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSA>())
            as LPSTR;
        lpStringPtrW = (lpStatusPtrW as usize
            + *lpServicesReturned as usize
                * core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>())
            as LPWSTR;

        dwServiceCount = 0;
        while dwServiceCount < *lpServicesReturned {
            WideCharToMultiByte(
                CP_ACP,
                0,
                lpStringPtrW,
                -1,
                lpStringPtrA,
                wcslen(lpStringPtrW) as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            (*lpStatusPtrA).lpServiceName =
                (lpStringPtrA as ULONG_PTR - lpBuffer as ULONG_PTR) as LPSTR;
            lpStringPtrA = lpStringPtrA.add(wcslen(lpStringPtrW) + 1);
            lpStringPtrW = lpStringPtrW.add(wcslen(lpStringPtrW) + 1);

            WideCharToMultiByte(
                CP_ACP,
                0,
                lpStringPtrW,
                -1,
                lpStringPtrA,
                wcslen(lpStringPtrW) as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );

            (*lpStatusPtrA).lpDisplayName =
                (lpStringPtrA as ULONG_PTR - lpBuffer as ULONG_PTR) as LPSTR;
            lpStringPtrA = lpStringPtrA.add(wcslen(lpStringPtrW) + 1);
            lpStringPtrW = lpStringPtrW.add(wcslen(lpStringPtrW) + 1);

            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*lpStatusPtrIncrW).ServiceStatusProcess) as *const u8,
                core::ptr::addr_of_mut!((*lpStatusPtrA).ServiceStatusProcess) as *mut u8,
                core::mem::size_of::<SERVICE_STATUS>(),
            );

            (*lpStatusPtrA).ServiceStatusProcess.dwProcessId =
                (*lpStatusPtrIncrW).ServiceStatusProcess.dwProcessId;

            (*lpStatusPtrA).ServiceStatusProcess.dwServiceFlags = 0;

            lpStatusPtrIncrW = lpStatusPtrIncrW.add(1);
            lpStatusPtrA = lpStatusPtrA.add(1);

            dwServiceCount += 1;
        }

        break 'done;
    }

    if !pszGroupNameW.is_null() {
        HeapFree(GetProcessHeap(), 0, pszGroupNameW as *mut core::ffi::c_void);
    }

    if !lpStatusPtrW.is_null() {
        HeapFree(GetProcessHeap(), 0, lpStatusPtrW as *mut core::ffi::c_void);
    }

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn REnumServicesStatusExW(
    hSCManager: SC_RPC_HANDLE,
    InfoLevel: SC_ENUM_TYPE,
    dwServiceType: DWORD,
    dwServiceState: DWORD,
    lpBuffer: LPBYTE,
    cbBufSize: DWORD,
    pcbBytesNeeded: LPBOUNDED_DWORD_256K,
    lpServicesReturned: LPBOUNDED_DWORD_256K,
    lpResumeIndex: LPBOUNDED_DWORD_256K,
    pszGroupName: LPCWSTR,
) -> DWORD {
    let hManager: PMANAGER_HANDLE;
    let lpService: PSERVICE;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut ServiceEntry: PLIST_ENTRY;
    let mut CurrentService: PSERVICE;
    let mut dwState: DWORD;
    let mut dwRequiredSize: DWORD;
    let mut dwServiceCount: DWORD;
    let mut dwSize: DWORD;
    let mut dwLastResumeCount: DWORD = 0;
    let mut lpStatusPtr: LPENUM_SERVICE_STATUS_PROCESSW;
    let mut lpStringPtr: LPWSTR;

    if ScmShutdown != 0 {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if InfoLevel != SC_ENUM_PROCESS_INFO {
        return ERROR_INVALID_LEVEL;
    }

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if pcbBytesNeeded.is_null() || lpServicesReturned.is_null() {
        return ERROR_INVALID_ADDRESS;
    }

    *pcbBytesNeeded = 0;
    *lpServicesReturned = 0;

    if (dwServiceType == 0) || ((dwServiceType & !SERVICE_TYPE_ALL) != 0) {
        return ERROR_INVALID_PARAMETER;
    }

    if (dwServiceState == 0) || ((dwServiceState & !SERVICE_STATE_ALL) != 0) {
        return ERROR_INVALID_PARAMETER;
    }

    if RtlAreAllAccessesGranted(
        (*hManager).Handle.DesiredAccess,
        SC_MANAGER_ENUMERATE_SERVICE,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    if !lpResumeIndex.is_null() {
        dwLastResumeCount = *lpResumeIndex;
    }

    ScmLockDatabaseShared();

    lpService = ScmGetServiceEntryByResumeCount(dwLastResumeCount);

    'done: {
        if lpService.is_null() {
            dwError = ERROR_SUCCESS;
            break 'done;
        }

        dwRequiredSize = 0;
        dwServiceCount = 0;

        ServiceEntry = core::ptr::addr_of_mut!((*lpService).ServiceListEntry);
        while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            dwState = SERVICE_ACTIVE;
            if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                dwState = SERVICE_INACTIVE;
            }

            if (dwState & dwServiceState) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            if !pszGroupName.is_null() {
                if *pszGroupName == 0 {
                    if !(*CurrentService).lpGroup.is_null() {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                } else {
                    if (*CurrentService).lpGroup.is_null()
                        || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                    {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                }
            }

            dwSize = (core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>() as DWORD)
                + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD
                + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;

            if dwRequiredSize + dwSize <= cbBufSize {
                dwRequiredSize += dwSize;
                dwServiceCount += 1;
                dwLastResumeCount = (*CurrentService).dwResumeCount;
            } else {
                break;
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            dwState = SERVICE_ACTIVE;
            if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                dwState = SERVICE_INACTIVE;
            }

            if (dwState & dwServiceState) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            if !pszGroupName.is_null() {
                if *pszGroupName == 0 {
                    if !(*CurrentService).lpGroup.is_null() {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                } else {
                    if (*CurrentService).lpGroup.is_null()
                        || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                    {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                }
            }

            dwRequiredSize += (core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>() as DWORD)
                + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD
                + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;

            dwError = ERROR_MORE_DATA;

            ServiceEntry = (*ServiceEntry).Flink;
        }

        if !lpResumeIndex.is_null() {
            *lpResumeIndex = dwLastResumeCount;
        }

        *lpServicesReturned = dwServiceCount;
        *pcbBytesNeeded = dwRequiredSize;

        if dwServiceCount == 0 && dwError != ERROR_MORE_DATA {
            dwError = ERROR_SERVICE_DOES_NOT_EXIST;
            break 'done;
        }

        lpStatusPtr = lpBuffer as LPENUM_SERVICE_STATUS_PROCESSW;
        lpStringPtr = (lpBuffer as usize
            + (dwServiceCount as usize) * core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>())
            as LPWSTR;

        dwRequiredSize = 0;
        ServiceEntry = core::ptr::addr_of_mut!((*lpService).ServiceListEntry);
        while ServiceEntry != core::ptr::addr_of_mut!(ServiceListHead) {
            CurrentService = (ServiceEntry as usize
                - core::mem::offset_of!(SERVICE, ServiceListEntry))
                as PSERVICE;

            if ((*CurrentService).Status.dwServiceType & dwServiceType) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            dwState = SERVICE_ACTIVE;
            if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED {
                dwState = SERVICE_INACTIVE;
            }

            if (dwState & dwServiceState) == 0 {
                ServiceEntry = (*ServiceEntry).Flink;
                continue;
            }

            if !pszGroupName.is_null() {
                if *pszGroupName == 0 {
                    if !(*CurrentService).lpGroup.is_null() {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                } else {
                    if (*CurrentService).lpGroup.is_null()
                        || _wcsicmp(pszGroupName, (*(*CurrentService).lpGroup).lpGroupName) != 0
                    {
                        ServiceEntry = (*ServiceEntry).Flink;
                        continue;
                    }
                }
            }

            dwSize = (core::mem::size_of::<ENUM_SERVICE_STATUS_PROCESSW>() as DWORD)
                + ((wcslen((*CurrentService).lpServiceName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD
                + ((wcslen((*CurrentService).lpDisplayName) + 1) * core::mem::size_of::<WCHAR>()) as DWORD;

            if dwRequiredSize + dwSize <= cbBufSize {
                wcscpy(lpStringPtr, (*CurrentService).lpServiceName);
                (*lpStatusPtr).lpServiceName =
                    (lpStringPtr as usize - lpBuffer as usize) as LPWSTR;
                lpStringPtr = lpStringPtr.add(wcslen((*CurrentService).lpServiceName) + 1);

                wcscpy(lpStringPtr, (*CurrentService).lpDisplayName);
                (*lpStatusPtr).lpDisplayName =
                    (lpStringPtr as usize - lpBuffer as usize) as LPWSTR;
                lpStringPtr = lpStringPtr.add(wcslen((*CurrentService).lpDisplayName) + 1);

                memcpy(
                    core::ptr::addr_of_mut!((*lpStatusPtr).ServiceStatusProcess) as *mut _,
                    core::ptr::addr_of!((*CurrentService).Status) as *const _,
                    core::mem::size_of::<SERVICE_STATUS>(),
                );

                if (*CurrentService).Status.dwCurrentState == SERVICE_STOPPED
                    || (*CurrentService).lpImage.is_null()
                {
                    (*lpStatusPtr).ServiceStatusProcess.dwProcessId = 0;
                } else {
                    (*lpStatusPtr).ServiceStatusProcess.dwProcessId =
                        (*(*CurrentService).lpImage).dwProcessId;
                }

                (*lpStatusPtr).ServiceStatusProcess.dwServiceFlags = 0;

                lpStatusPtr = lpStatusPtr.add(1);
                dwRequiredSize += dwSize;
            } else {
                break;
            }

            ServiceEntry = (*ServiceEntry).Flink;
        }

        if dwError == 0 {
            *pcbBytesNeeded = 0;
            if !lpResumeIndex.is_null() {
                *lpResumeIndex = 0;
            }
        }
    }

    ScmUnlockDatabase();

    dwError
}
#[no_mangle]
pub unsafe extern "system" fn RSendTSMessage(_BindingHandle: handle_t) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RCreateServiceWOW64A(
    _BindingHandle: handle_t,
    _lpServiceName: LPSTR,
    _lpDisplayName: LPSTR,
    _dwDesiredAccess: DWORD,
    _dwServiceType: DWORD,
    _dwStartType: DWORD,
    _dwErrorControl: DWORD,
    _lpBinaryPathName: LPSTR,
    _lpLoadOrderGroup: LPSTR,
    _lpdwTagId: LPDWORD,
    _lpDependencies: LPBYTE,
    _dwDependSize: DWORD,
    _lpServiceStartName: LPSTR,
    _lpPassword: LPBYTE,
    _dwPwSize: DWORD,
    _lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RCreateServiceWOW64W(
    _BindingHandle: handle_t,
    _lpServiceName: LPWSTR,
    _lpDisplayName: LPWSTR,
    _dwDesiredAccess: DWORD,
    _dwServiceType: DWORD,
    _dwStartType: DWORD,
    _dwErrorControl: DWORD,
    _lpBinaryPathName: LPWSTR,
    _lpLoadOrderGroup: LPWSTR,
    _lpdwTagId: LPDWORD,
    _lpDependencies: LPBYTE,
    _dwDependSize: DWORD,
    _lpServiceStartName: LPWSTR,
    _lpPassword: LPBYTE,
    _dwPwSize: DWORD,
    _lpServiceHandle: LPSC_RPC_HANDLE,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScQueryServiceTagInfo(
    hSCManager: SC_RPC_HANDLE,
    dwInfoLevel: TAG_INFO_LEVEL,
    lpInParams: *mut PTAG_INFO_NAME_FROM_TAG_IN_PARAMS,
    lpOutParams: *mut PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS,
) -> DWORD {
    let hManager: PMANAGER_HANDLE;

    /* Validate handle */
    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    /* FIXME: should check whether client is local */

    /* Check access rights */
    if RtlAreAllAccessesGranted(
        (*hManager).Handle.DesiredAccess,
        SC_MANAGER_ENUMERATE_SERVICE,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    /* Check parameters */
    if lpInParams.is_null() || lpOutParams.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    /* Check info level */
    if dwInfoLevel != TagInfoLevelNameFromTag {
        return ERROR_RETRY;
    }

    /* Call internal helper */
    ScmGetServiceNameFromTag(*lpInParams, lpOutParams)
}
#[no_mangle]
pub unsafe extern "system" fn RNotifyServiceStatusChange(
    _hService: SC_RPC_HANDLE,
    _NotifyParams: SC_RPC_NOTIFY_PARAMS,
    _pClientProcessGuid: *mut GUID,
    _pSCMProcessGuid: *mut GUID,
    _pfCreateRemoteQueue: PBOOL,
    _phNotify: LPSC_NOTIFY_RPC_HANDLE,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RGetNotifyResults(
    _hNotify: SC_NOTIFY_RPC_HANDLE,
    _ppNotifyParams: *mut PSC_RPC_NOTIFY_PARAMS_LIST,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RCloseNotifyHandle(
    _phNotify: LPSC_NOTIFY_RPC_HANDLE,
    _pfApcFired: PBOOL,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RControlServiceExA(
    _hService: SC_RPC_HANDLE,
    _dwControl: DWORD,
    _dwInfoLevel: DWORD,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RControlServiceExW(
    _hService: SC_RPC_HANDLE,
    _dwControl: DWORD,
    _dwInfoLevel: DWORD,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScSendPnPMessage(
    hServiceStatus: RPC_SERVICE_STATUS_HANDLE,
    dwControl: DWORD,
    dwEventType: DWORD,
    dwEventSize: DWORD,
    pEventData: LPBYTE,
) -> DWORD {
    let ControlPacket: PSCM_CONTROL_PACKET;
    let PacketSize: DWORD;
    let pService: PSERVICE;
    let mut Ptr: ULONG_PTR;
    let dwControlsAccepted: DWORD;
    let dwCurrentState: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;

    /* FIXME: Verify the status handle */
    pService = hServiceStatus as PSERVICE;

    /* Fail, if the service is a driver */
    if (*pService).Status.dwServiceType & SERVICE_DRIVER != 0 {
        return ERROR_INVALID_SERVICE_CONTROL;
    }

    dwControlsAccepted = (*pService).Status.dwControlsAccepted;
    dwCurrentState = (*pService).Status.dwCurrentState;

    /* Return ERROR_SERVICE_NOT_ACTIVE if the service has not been started */
    if (*pService).lpImage.is_null() || dwCurrentState == SERVICE_STOPPED {
        return ERROR_SERVICE_NOT_ACTIVE;
    }

    /* The service cannot accept a control code if it is not running */
    if dwCurrentState != SERVICE_RUNNING {
        return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
    }

    /* Check if the control code is acceptable to the service */
    match dwControl {
        SERVICE_CONTROL_DEVICEEVENT => {}

        SERVICE_CONTROL_HARDWAREPROFILECHANGE => {
            if (dwControlsAccepted & SERVICE_ACCEPT_HARDWAREPROFILECHANGE) == 0 {
                return ERROR_INVALID_SERVICE_CONTROL;
            }
        }

        SERVICE_CONTROL_POWEREVENT => {
            if (dwControlsAccepted & SERVICE_ACCEPT_POWEREVENT) == 0 {
                return ERROR_INVALID_SERVICE_CONTROL;
            }
        }

        SERVICE_CONTROL_SESSIONCHANGE => {
            if (dwControlsAccepted & SERVICE_ACCEPT_SESSIONCHANGE) == 0 {
                return ERROR_INVALID_SERVICE_CONTROL;
            }
        }

        _ => {
            return ERROR_INVALID_SERVICE_CONTROL;
        }
    }

    /* Calculate the total size of the control packet:
     * initial structure, event type and event data */
    PacketSize = (core::mem::size_of::<SCM_CONTROL_PACKET>() as DWORD)
        + (core::mem::size_of::<DWORD>() as DWORD)
        + dwEventSize;

    /* Allocate the control packet */
    ControlPacket =
        HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize as SIZE_T) as PSCM_CONTROL_PACKET;
    if ControlPacket.is_null() {
        return ERROR_NOT_ENOUGH_MEMORY;
    }

    (*ControlPacket).dwSize = PacketSize;
    (*ControlPacket).dwControl = dwControl;

    /* Copy the event type and event data into the control packet */
    Ptr = (ControlPacket as ULONG_PTR) + (core::mem::size_of::<SCM_CONTROL_PACKET>() as ULONG_PTR);
    RtlCopyMemory(
        Ptr as *mut core::ffi::c_void,
        core::ptr::addr_of!(dwEventType) as *const core::ffi::c_void,
        core::mem::size_of::<DWORD>() as usize,
    );
    if dwEventSize != 0 {
        Ptr = Ptr + (core::mem::size_of::<DWORD>() as ULONG_PTR);
        RtlCopyMemory(
            Ptr as *mut core::ffi::c_void,
            pEventData as *const core::ffi::c_void,
            dwEventSize as usize,
        );
    }

    /* Send the control packet */
    dwError = ScmSendControlPacket(
        (*(*pService).lpImage).hControlPipe,
        (*pService).lpServiceName,
        dwControl,
        PacketSize,
        ControlPacket as PVOID,
    );

    /* Free the control packet */
    HeapFree(GetProcessHeap(), 0, ControlPacket as *mut core::ffi::c_void);

    dwError
}

#[no_mangle]
pub unsafe extern "system" fn RI_ScValidatePnPService(
    hSCManager: SC_RPC_HANDLE,
    pszServiceName: LPWSTR,
    phServiceStatus: *mut RPC_SERVICE_STATUS_HANDLE,
) -> DWORD {
    let hManager: PMANAGER_HANDLE;
    let pService: PSERVICE;

    hManager = ScmGetServiceManagerFromHandle(hSCManager);
    if hManager.is_null() {
        return ERROR_INVALID_HANDLE;
    }

    if RtlAreAllAccessesGranted(
        (*hManager).Handle.DesiredAccess,
        SC_MANAGER_CONNECT,
    ) == 0
    {
        return ERROR_ACCESS_DENIED;
    }

    pService = ScmGetServiceEntryByName(pszServiceName);
    if pService.is_null() {
        return ERROR_SERVICE_DOES_NOT_EXIST;
    }

    *phServiceStatus = pService as RPC_SERVICE_STATUS_HANDLE;

    ERROR_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn ROpenServiceStatusHandle(
    _BindingHandle: handle_t,
) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn RFunction55(_BindingHandle: handle_t) -> DWORD {
    ERROR_CALL_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
#[no_mangle]
pub unsafe extern "system" fn SC_RPC_HANDLE_rundown(mut hSCObject: SC_RPC_HANDLE) {
    RCloseServiceHandle(&mut hSCObject);
}
#[no_mangle]
pub unsafe extern "system" fn SC_RPC_LOCK_rundown(mut Lock: SC_RPC_LOCK) {
    RUnlockServiceDatabase(&mut Lock);
}
#[no_mangle]
pub unsafe extern "system" fn SC_NOTIFY_RPC_HANDLE_rundown(_hNotify: SC_NOTIFY_RPC_HANDLE) {
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Service Control Manager
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        base/system/services/rpcserver.c
//  * PURPOSE:     RPC server interface for the advapi32 calls
//  * COPYRIGHT:   Copyright 2005-2006 Eric Kohl
//  *              Copyright 2006-2007 Hervé Poussineau <hpoussin@reactos.org>
//  *              Copyright 2007 Ged Murphy <gedmurphy@reactos.org>
//  */
//
// /* INCLUDES ****************************************************************/
//
// #include "services.h"
//
// #include <winnls.h>
// #include <strsafe.h>
//
// #include <pseh/pseh2.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *****************************************************************/
//
// #define MANAGER_TAG 0x72674D68  /* 'hMgr' */
// #define SERVICE_TAG 0x63765368  /* 'hSvc' */
// #define INVALID_TAG 0xAABBCCDD
//
// typedef struct _SCMGR_HANDLE
// {
//     DWORD Tag;
//     DWORD DesiredAccess;
// } SCMGR_HANDLE;
//
//
// typedef struct _MANAGER_HANDLE
// {
//     SCMGR_HANDLE Handle;
//     WCHAR DatabaseName[1];
// } MANAGER_HANDLE, *PMANAGER_HANDLE;
//
//
// typedef struct _SERVICE_HANDLE
// {
//     SCMGR_HANDLE Handle;
//     PSERVICE ServiceEntry;
// } SERVICE_HANDLE, *PSERVICE_HANDLE;
//
//
// #define SC_MANAGER_READ \
//   (STANDARD_RIGHTS_READ | \
//    SC_MANAGER_QUERY_LOCK_STATUS | \
//    SC_MANAGER_ENUMERATE_SERVICE)
//
// #define SC_MANAGER_WRITE \
//   (STANDARD_RIGHTS_WRITE | \
//    SC_MANAGER_MODIFY_BOOT_CONFIG | \
//    SC_MANAGER_CREATE_SERVICE)
//
// #define SC_MANAGER_EXECUTE \
//   (STANDARD_RIGHTS_EXECUTE | \
//    SC_MANAGER_LOCK | \
//    SC_MANAGER_ENUMERATE_SERVICE | \
//    SC_MANAGER_CONNECT | \
//    SC_MANAGER_CREATE_SERVICE)
//
//
// #define SERVICE_READ \
//   (STANDARD_RIGHTS_READ | \
//    SERVICE_INTERROGATE | \
//    SERVICE_ENUMERATE_DEPENDENTS | \
//    SERVICE_QUERY_STATUS | \
//    SERVICE_QUERY_CONFIG)
//
// #define SERVICE_WRITE \
//   (STANDARD_RIGHTS_WRITE | \
//    SERVICE_CHANGE_CONFIG)
//
// #define SERVICE_EXECUTE \
//   (STANDARD_RIGHTS_EXECUTE | \
//    SERVICE_USER_DEFINED_CONTROL | \
//    SERVICE_PAUSE_CONTINUE | \
//    SERVICE_STOP | \
//    SERVICE_START)
//
// #define TAG_ARRAY_SIZE 32
//
// /* VARIABLES ***************************************************************/
//
// static GENERIC_MAPPING
// ScmManagerMapping = {SC_MANAGER_READ,
//                      SC_MANAGER_WRITE,
//                      SC_MANAGER_EXECUTE,
//                      SC_MANAGER_ALL_ACCESS};
//
// static GENERIC_MAPPING
// ScmServiceMapping = {SERVICE_READ,
//                      SERVICE_WRITE,
//                      SERVICE_EXECUTE,
//                      SERVICE_ALL_ACCESS};
//
// DWORD g_dwServiceBits = 0;
//
// /* FUNCTIONS ***************************************************************/
//
// VOID
// ScmStartRpcServer(VOID)
// {
//     RPC_STATUS Status;
//
//     DPRINT("ScmStartRpcServer() called\n");
//
//     Status = RpcServerUseProtseqEpW(L"ncacn_np",
//                                     10,
//                                     L"\\pipe\\ntsvcs",
//                                     NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT1("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     Status = RpcServerRegisterIf(svcctl_v2_0_s_ifspec,
//                                  NULL,
//                                  NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT1("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     Status = RpcServerListen(1, 20, TRUE);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT1("RpcServerListen() failed (Status %lx)\n", Status);
//         return;
//     }
//
//     DPRINT("ScmStartRpcServer() done\n");
// }
//
//
// static DWORD
// ScmCreateManagerHandle(LPWSTR lpDatabaseName,
//                        SC_HANDLE *Handle)
// {
//     PMANAGER_HANDLE Ptr;
//
//     if (lpDatabaseName == NULL)
//         lpDatabaseName = SERVICES_ACTIVE_DATABASEW;
//
//     if (_wcsicmp(lpDatabaseName, SERVICES_FAILED_DATABASEW) == 0)
//     {
//         DPRINT("Database %S, does not exist\n", lpDatabaseName);
//         return ERROR_DATABASE_DOES_NOT_EXIST;
//     }
//     else if (_wcsicmp(lpDatabaseName, SERVICES_ACTIVE_DATABASEW) != 0)
//     {
//         DPRINT("Invalid Database name %S\n", lpDatabaseName);
//         return ERROR_INVALID_NAME;
//     }
//
//     Ptr = HeapAlloc(GetProcessHeap(),
//                     HEAP_ZERO_MEMORY,
//                     FIELD_OFFSET(MANAGER_HANDLE, DatabaseName[wcslen(lpDatabaseName) + 1]));
//     if (Ptr == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     Ptr->Handle.Tag = MANAGER_TAG;
//
//     wcscpy(Ptr->DatabaseName, lpDatabaseName);
//
//     *Handle = (SC_HANDLE)Ptr;
//
//     return ERROR_SUCCESS;
// }
//
//
// static DWORD
// ScmCreateServiceHandle(PSERVICE lpServiceEntry,
//                        SC_HANDLE *Handle)
// {
//     PSERVICE_HANDLE Ptr;
//
//     Ptr = HeapAlloc(GetProcessHeap(),
//                     HEAP_ZERO_MEMORY,
//                     sizeof(SERVICE_HANDLE));
//     if (Ptr == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     Ptr->Handle.Tag = SERVICE_TAG;
//
//     Ptr->ServiceEntry = lpServiceEntry;
//
//     *Handle = (SC_HANDLE)Ptr;
//
//     return ERROR_SUCCESS;
// }
//
//
// static PMANAGER_HANDLE
// ScmGetServiceManagerFromHandle(SC_RPC_HANDLE Handle)
// {
//     PMANAGER_HANDLE pManager = NULL;
//
//     _SEH2_TRY
//     {
//         if (((PMANAGER_HANDLE)Handle)->Handle.Tag == MANAGER_TAG)
//             pManager = (PMANAGER_HANDLE)Handle;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("Exception: Invalid Service Manager handle\n");
//     }
//     _SEH2_END;
//
//     return pManager;
// }
//
//
// static PSERVICE_HANDLE
// ScmGetServiceFromHandle(SC_RPC_HANDLE Handle)
// {
//     PSERVICE_HANDLE pService = NULL;
//
//     _SEH2_TRY
//     {
//         if (((PSERVICE_HANDLE)Handle)->Handle.Tag == SERVICE_TAG)
//             pService = (PSERVICE_HANDLE)Handle;
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         DPRINT1("Exception: Invalid Service handle\n");
//     }
//     _SEH2_END;
//
//     return pService;
// }
//
//
// static DWORD
// ScmCheckAccess(SC_HANDLE Handle,
//                DWORD dwDesiredAccess)
// {
//     PMANAGER_HANDLE hMgr;
//
//     hMgr = (PMANAGER_HANDLE)Handle;
//     if (hMgr->Handle.Tag == MANAGER_TAG)
//     {
//         RtlMapGenericMask(&dwDesiredAccess,
//                           &ScmManagerMapping);
//
//         hMgr->Handle.DesiredAccess = dwDesiredAccess;
//
//         return ERROR_SUCCESS;
//     }
//     else if (hMgr->Handle.Tag == SERVICE_TAG)
//     {
//         RtlMapGenericMask(&dwDesiredAccess,
//                           &ScmServiceMapping);
//
//         hMgr->Handle.DesiredAccess = dwDesiredAccess;
//
//         return ERROR_SUCCESS;
//     }
//
//     return ERROR_INVALID_HANDLE;
// }
//
//
// DWORD
// ScmAssignNewTag(PSERVICE lpService)
// {
//     HKEY hKey = NULL;
//     DWORD dwError;
//     DWORD dwGroupTagCount = 0;
//     PDWORD pdwGroupTags = NULL;
//     DWORD dwFreeTag = 0;
//     DWORD dwTagUsedBase = 1;
//     BOOLEAN TagUsed[TAG_ARRAY_SIZE];
//     INT nTagOffset;
//     DWORD i;
//     DWORD cbDataSize;
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//
//     ASSERT(lpService != NULL);
//     ASSERT(lpService->lpGroup != NULL);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Control\\GroupOrderList",
//                             0,
//                             KEY_READ,
//                             &hKey) as DWORD;
//
//     if (dwError != ERROR_SUCCESS)
//         goto findFreeTag;
//
//     /* query value length */
//     cbDataSize = 0;
//     dwError = RegQueryValueExW(hKey,
//                                lpService->lpGroup->szGroupName,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &cbDataSize) as DWORD;
//
//     if (dwError != ERROR_SUCCESS && dwError != ERROR_MORE_DATA)
//         goto findFreeTag;
//
//     pdwGroupTags = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbDataSize);
//     if (!pdwGroupTags)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     dwError = RegQueryValueExW(hKey,
//                                lpService->lpGroup->szGroupName,
//                                NULL,
//                                NULL,
//                                (LPBYTE)pdwGroupTags,
//                                &cbDataSize) as DWORD;
//
//     if (dwError != ERROR_SUCCESS)
//         goto findFreeTag;
//
//     if (cbDataSize < sizeof(pdwGroupTags[0]))
//         goto findFreeTag;
//
//     dwGroupTagCount = min(pdwGroupTags[0], cbDataSize / sizeof(pdwGroupTags[0]) - 1);
//
// findFreeTag:
//     do
//     {
//         /* mark all tags as unused */
//         for (i = 0; i < TAG_ARRAY_SIZE; i++)
//             TagUsed[i] = FALSE;
//
//         /* mark tags in GroupOrderList as used */
//         for (i = 1; i <= dwGroupTagCount; i++)
//         {
//             nTagOffset = pdwGroupTags[i] - dwTagUsedBase;
//             if (nTagOffset >= 0 && nTagOffset < TAG_ARRAY_SIZE)
//                 TagUsed[nTagOffset] = TRUE;
//         }
//
//         /* mark tags in service list as used */
//         ServiceEntry = lpService->ServiceListEntry.Flink;
//         while (ServiceEntry != &lpService->ServiceListEntry)
//         {
//             ASSERT(ServiceEntry != NULL);
//             CurrentService = CONTAINING_RECORD(ServiceEntry, SERVICE, ServiceListEntry);
//             if (CurrentService->lpGroup == lpService->lpGroup)
//             {
//                 nTagOffset = CurrentService->dwTag - dwTagUsedBase;
//                 if (nTagOffset >= 0 && nTagOffset < TAG_ARRAY_SIZE)
//                     TagUsed[nTagOffset] = TRUE;
//             }
//
//             ServiceEntry = ServiceEntry->Flink;
//         }
//
//         /* find unused tag, if any */
//         for (i = 0; i < TAG_ARRAY_SIZE; i++)
//         {
//             if (!TagUsed[i])
//             {
//                 dwFreeTag = dwTagUsedBase + i;
//                 break;
//             }
//         }
//
//         dwTagUsedBase += TAG_ARRAY_SIZE;
//     } while (!dwFreeTag);
//
// cleanup:
//     if (pdwGroupTags)
//         HeapFree(GetProcessHeap(), 0, pdwGroupTags);
//
//     if (hKey)
//         RegCloseKey(hKey);
//
//     if (dwFreeTag)
//     {
//         lpService->dwTag = dwFreeTag;
//         DPRINT("Assigning new tag %lu to service %S in group %S\n",
//                lpService->dwTag, lpService->lpServiceName, lpService->lpGroup->szGroupName);
//         dwError = ERROR_SUCCESS;
//     }
//     else
//     {
//         DPRINT1("Failed to assign new tag to service %S, error=%lu\n",
//                 lpService->lpServiceName, dwError);
//     }
//
//     return dwError;
// }
//
//
// /* Create a path suitable for the bootloader out of the full path */
// DWORD
// ScmConvertToBootPathName(wchar_t *CanonName, wchar_t **RelativeName)
// {
//     SIZE_T ServiceNameLen, ExpandedLen;
//     DWORD BufferSize;
//     WCHAR Dest;
//     WCHAR *Expanded;
//     UNICODE_STRING NtPathName, SystemRoot, LinkTarget;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE SymbolicLinkHandle;
//
//     DPRINT("ScmConvertToBootPathName %S\n", CanonName);
//
//     if (!RelativeName)
//         return ERROR_INVALID_PARAMETER;
//
//     *RelativeName = NULL;
//
//     ServiceNameLen = wcslen(CanonName);
//
//     /* First check, if it's already good */
//     if (ServiceNameLen > 12 &&
//         !_wcsnicmp(L"\\SystemRoot\\", CanonName, 12))
//     {
//         *RelativeName = HeapAlloc(GetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   (ServiceNameLen + 1) * sizeof(WCHAR));
//         if (*RelativeName == NULL)
//         {
//             DPRINT("Error allocating memory for boot driver name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         /* Copy it */
//         wcscpy(*RelativeName, CanonName);
//
//         DPRINT("Bootdriver name %S\n", *RelativeName);
//         return ERROR_SUCCESS;
//     }
//
//     /* If it has %SystemRoot% prefix, substitute it to \System*/
//     if (ServiceNameLen > 13 &&
//         !_wcsnicmp(L"%SystemRoot%\\", CanonName, 13))
//     {
//         /* There is no +sizeof(wchar_t) because the name is less by 1 wchar */
//         *RelativeName = HeapAlloc(GetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   ServiceNameLen * sizeof(WCHAR));
//
//         if (*RelativeName == NULL)
//         {
//             DPRINT("Error allocating memory for boot driver name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         /* Copy it */
//         wcscpy(*RelativeName, L"\\SystemRoot\\");
//         wcscat(*RelativeName, CanonName + 13);
//
//         DPRINT("Bootdriver name %S\n", *RelativeName);
//         return ERROR_SUCCESS;
//     }
//
//     /* Get buffer size needed for expanding env strings */
//     BufferSize = ExpandEnvironmentStringsW(L"%SystemRoot%\\", &Dest, 1);
//     if (BufferSize <= 1)
//     {
//         DPRINT("Error during a call to ExpandEnvironmentStringsW()\n");
//         return ERROR_INVALID_ENVIRONMENT;
//     }
//
//     /* Allocate memory, since the size is known now */
//     Expanded = HeapAlloc(GetProcessHeap(),
//                          HEAP_ZERO_MEMORY,
//                          (BufferSize + 1) * sizeof(WCHAR));
//     if (!Expanded)
//     {
//         DPRINT("Error allocating memory for boot driver name\n");
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     /* Expand it */
//     if (ExpandEnvironmentStringsW(L"%SystemRoot%\\", Expanded, BufferSize) >
//         BufferSize)
//     {
//         DPRINT("Error during a call to ExpandEnvironmentStringsW()\n");
//         HeapFree(GetProcessHeap(), 0, Expanded);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     /* Convert to NT-style path */
//     if (!RtlDosPathNameToNtPathName_U(Expanded, &NtPathName, NULL, NULL))
//     {
//         DPRINT("Error during a call to RtlDosPathNameToNtPathName_U()\n");
//         return ERROR_INVALID_ENVIRONMENT;
//     }
//
//     DPRINT("Converted to NT-style %wZ\n", &NtPathName);
//
//     /* No need to keep the dos-path anymore */
//     HeapFree(GetProcessHeap(), 0, Expanded);
//
//     /* Copy it to the allocated place */
//     Expanded = HeapAlloc(GetProcessHeap(),
//                          HEAP_ZERO_MEMORY,
//                          NtPathName.Length + sizeof(UNICODE_NULL));
//     if (!Expanded)
//     {
//         DPRINT("Error allocating memory for boot driver name\n");
//         RtlFreeUnicodeString(&NtPathName);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     ExpandedLen = NtPathName.Length / sizeof(WCHAR);
//     wcsncpy(Expanded, NtPathName.Buffer, ExpandedLen);
//     Expanded[ExpandedLen] = UNICODE_NULL;
//     RtlFreeUnicodeString(&NtPathName);
//
//     if (ServiceNameLen > ExpandedLen &&
//         !_wcsnicmp(Expanded, CanonName, ExpandedLen))
//     {
//         HeapFree(GetProcessHeap(), 0, Expanded);
//
//         /* Only \SystemRoot\ is missing */
//         *RelativeName = HeapAlloc(GetProcessHeap(),
//                                   HEAP_ZERO_MEMORY,
//                                   (ServiceNameLen - ExpandedLen) * sizeof(WCHAR) + 13*sizeof(WCHAR));
//         if (*RelativeName == NULL)
//         {
//             DPRINT("Error allocating memory for boot driver name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         wcscpy(*RelativeName, L"\\SystemRoot\\");
//         wcscat(*RelativeName, CanonName + ExpandedLen);
//
//         return ERROR_SUCCESS;
//     }
//
//     /* No longer need this */
//     HeapFree(GetProcessHeap(), 0, Expanded);
//
//     /* The most complex case starts here */
//     RtlInitUnicodeString(&SystemRoot, L"\\SystemRoot");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SystemRoot,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open this symlink */
//     Status = NtOpenSymbolicLinkObject(&SymbolicLinkHandle, SYMBOLIC_LINK_QUERY, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         DPRINT("Opened symbolic link object\n");
//
//         RtlInitEmptyUnicodeString(&LinkTarget, NULL, 0);
//         Status = NtQuerySymbolicLinkObject(SymbolicLinkHandle, &LinkTarget, &BufferSize);
//         if (NT_SUCCESS(Status) || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             /* Check if required buffer size is sane */
//             if (BufferSize > UNICODE_STRING_MAX_BYTES - sizeof(UNICODE_NULL))
//             {
//                 DPRINT("Too large buffer required\n");
//
//                 NtClose(SymbolicLinkHandle);
//                 return ERROR_NOT_ENOUGH_MEMORY;
//             }
//
//             /* Alloc the string */
//             LinkTarget.Length = (USHORT)BufferSize;
//             LinkTarget.MaximumLength = LinkTarget.Length + sizeof(UNICODE_NULL);
//             LinkTarget.Buffer = HeapAlloc(GetProcessHeap(),
//                                           HEAP_ZERO_MEMORY,
//                                           LinkTarget.MaximumLength);
//             if (!LinkTarget.Buffer)
//             {
//                 DPRINT("Unable to alloc buffer\n");
//                 NtClose(SymbolicLinkHandle);
//                 return ERROR_NOT_ENOUGH_MEMORY;
//             }
//
//             /* Do a real query now */
//             Status = NtQuerySymbolicLinkObject(SymbolicLinkHandle, &LinkTarget, &BufferSize);
//             NtClose(SymbolicLinkHandle);
//             if (NT_SUCCESS(Status))
//             {
//                 DPRINT("LinkTarget: %wZ\n", &LinkTarget);
//
//                 ExpandedLen = LinkTarget.Length / sizeof(WCHAR);
//                 if ((ServiceNameLen > ExpandedLen) &&
//                     !_wcsnicmp(LinkTarget.Buffer, CanonName, ExpandedLen))
//                 {
//                     *RelativeName = HeapAlloc(GetProcessHeap(),
//                                               HEAP_ZERO_MEMORY,
//                                               (ServiceNameLen - ExpandedLen) * sizeof(WCHAR) + 13*sizeof(WCHAR));
//
//                     if (*RelativeName == NULL)
//                     {
//                         DPRINT("Unable to alloc buffer\n");
//                         return ERROR_NOT_ENOUGH_MEMORY;
//                     }
//
//                     /* Copy it over, substituting the first part
//                        with SystemRoot */
//                     wcscpy(*RelativeName, L"\\SystemRoot\\");
//                     wcscat(*RelativeName, CanonName+ExpandedLen+1);
//
//                     /* Return success */
//                     return ERROR_SUCCESS;
//                 }
//                 else
//                 {
//                     return ERROR_INVALID_PARAMETER;
//                 }
//             }
//             else
//             {
//                 DPRINT("Error, Status = %08X\n", Status);
//                 return ERROR_INVALID_PARAMETER;
//             }
//         }
//         else
//         {
//             DPRINT("Error, Status = %08X\n", Status);
//             NtClose(SymbolicLinkHandle);
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//     else
//     {
//         /* Failure */
//         DPRINT("Error, Status = %08X\n", Status);
//         return ERROR_INVALID_PARAMETER;
//     }
// }
//
//
// DWORD
// ScmCanonDriverImagePath(DWORD dwStartType,
//                         const wchar_t *lpServiceName,
//                         wchar_t **lpCanonName)
// {
//     DWORD Result;
//     SIZE_T ServiceNameLen;
//     UNICODE_STRING NtServiceName;
//     WCHAR *RelativeName;
//     const WCHAR *SourceName = lpServiceName;
//
//     /* Calculate the length of the service's name */
//     ServiceNameLen = wcslen(lpServiceName);
//
//     /* 12 is wcslen(L"\\SystemRoot\\") */
//     if (ServiceNameLen > 12 &&
//         !_wcsnicmp(L"\\SystemRoot\\", lpServiceName, 12))
//     {
//         /* SystemRoot prefix is already included */
//         *lpCanonName = HeapAlloc(GetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  (ServiceNameLen + 1) * sizeof(WCHAR));
//
//         if (*lpCanonName == NULL)
//         {
//             DPRINT("Error allocating memory for canonized service name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         /* If it's a boot-time driver, it must be systemroot relative */
//         if (dwStartType == SERVICE_BOOT_START)
//             SourceName += 12;
//
//         /* Copy it */
//         wcscpy(*lpCanonName, SourceName);
//
//         DPRINT("Canonicalized name %S\n", *lpCanonName);
//         return NO_ERROR;
//     }
//
//     /* Check if it has %SystemRoot% (len=13) */
//     if (ServiceNameLen > 13 &&
//         !_wcsnicmp(L"%SystemRoot%\\", lpServiceName, 13))
//     {
//         /* Substitute %SystemRoot% with \\SystemRoot\\ */
//         *lpCanonName = HeapAlloc(GetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  (ServiceNameLen + 1) * sizeof(WCHAR));
//
//         if (*lpCanonName == NULL)
//         {
//             DPRINT("Error allocating memory for canonized service name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         /* If it's a boot-time driver, it must be systemroot relative */
//         if (dwStartType == SERVICE_BOOT_START)
//             wcscpy(*lpCanonName, L"\\SystemRoot\\");
//
//         wcscat(*lpCanonName, lpServiceName + 13);
//
//         DPRINT("Canonicalized name %S\n", *lpCanonName);
//         return NO_ERROR;
//     }
//
//     /* Check if it's a relative path name */
//     if (lpServiceName[0] != L'\\' && lpServiceName[1] != L':')
//     {
//         *lpCanonName = HeapAlloc(GetProcessHeap(),
//                                  HEAP_ZERO_MEMORY,
//                                  (ServiceNameLen + 1) * sizeof(WCHAR));
//
//         if (*lpCanonName == NULL)
//         {
//             DPRINT("Error allocating memory for canonized service name\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//
//         /* Just copy it over without changing */
//         wcscpy(*lpCanonName, lpServiceName);
//
//         return NO_ERROR;
//     }
//
//     /* It seems to be a DOS path, convert it */
//     if (!RtlDosPathNameToNtPathName_U(lpServiceName, &NtServiceName, NULL, NULL))
//     {
//         DPRINT("RtlDosPathNameToNtPathName_U() failed\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     *lpCanonName = HeapAlloc(GetProcessHeap(),
//                              HEAP_ZERO_MEMORY,
//                              NtServiceName.Length + sizeof(WCHAR));
//
//     if (*lpCanonName == NULL)
//     {
//         DPRINT("Error allocating memory for canonized service name\n");
//         RtlFreeUnicodeString(&NtServiceName);
//         return ERROR_NOT_ENOUGH_MEMORY;
//     }
//
//     /* Copy the string */
//     wcsncpy(*lpCanonName, NtServiceName.Buffer, NtServiceName.Length / sizeof(WCHAR));
//
//     /* The unicode string is not needed anymore */
//     RtlFreeUnicodeString(&NtServiceName);
//
//     if (dwStartType != SERVICE_BOOT_START)
//     {
//         DPRINT("Canonicalized name %S\n", *lpCanonName);
//         return NO_ERROR;
//     }
//
//     /* The service is boot-started, so must be relative */
//     Result = ScmConvertToBootPathName(*lpCanonName, &RelativeName);
//     if (Result)
//     {
//         /* There is a problem, free name and return */
//         HeapFree(GetProcessHeap(), 0, *lpCanonName);
//         DPRINT("Error converting named\n");
//         return Result;
//     }
//
//     ASSERT(RelativeName);
//
//     /* Copy that string */
//     wcscpy(*lpCanonName, RelativeName + 12);
//
//     /* Free the allocated buffer */
//     HeapFree(GetProcessHeap(), 0, RelativeName);
//
//     DPRINT("Canonicalized name %S\n", *lpCanonName);
//
//     /* Success */
//     return NO_ERROR;
// }
//
//
// /* Recursively search for every dependency on every service */
// DWORD
// Int_EnumDependentServicesW(HKEY hServicesKey,
//                            PSERVICE lpService,
//                            DWORD dwServiceState,
//                            PSERVICE *lpServices,
//                            LPDWORD pcbBytesNeeded,
//                            LPDWORD lpServicesReturned)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     WCHAR szNameBuf[MAX_PATH];
//     WCHAR szValueBuf[MAX_PATH];
//     WCHAR *lpszNameBuf = szNameBuf;
//     WCHAR *lpszValueBuf = szValueBuf;
//     DWORD dwSize;
//     DWORD dwNumSubKeys;
//     DWORD dwIteration;
//     PSERVICE lpCurrentService;
//     HKEY hServiceEnumKey;
//     DWORD dwCurrentServiceState = SERVICE_ACTIVE;
//     DWORD dwDependServiceStrPtr = 0;
//     DWORD dwRequiredSize = 0;
//
//     /* Get the number of service keys */
//     dwError = RegQueryInfoKeyW(hServicesKey,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &dwNumSubKeys,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("ERROR! Unable to get number of services keys\n");
//         return dwError;
//     }
//
//     /* Iterate the service keys to see if another service depends on the this service */
//     for (dwIteration = 0; dwIteration < dwNumSubKeys; dwIteration++)
//     {
//         dwSize = MAX_PATH;
//         dwError = RegEnumKeyExW(hServicesKey,
//                                 dwIteration,
//                                 lpszNameBuf,
//                                 &dwSize,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 NULL) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             return dwError;
//
//         /* Open the Service key */
//         dwError = RegOpenKeyExW(hServicesKey,
//                                 lpszNameBuf,
//                                 0,
//                                 KEY_READ,
//                                 &hServiceEnumKey) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             return dwError;
//
//         dwSize = MAX_PATH * sizeof(WCHAR);
//
//         /* Check for the DependOnService Value */
//         dwError = RegQueryValueExW(hServiceEnumKey,
//                                    L"DependOnService",
//                                    NULL,
//                                    NULL,
//                                    (LPBYTE)lpszValueBuf,
//                                    &dwSize) as DWORD;
//
//         /* FIXME: Handle load order. */
//
//         /* If the service found has a DependOnService value */
//         if (dwError == ERROR_SUCCESS)
//         {
//             dwDependServiceStrPtr = 0;
//
//             /* Can be more than one Dependencies in the DependOnService string */
//             while (wcslen(lpszValueBuf + dwDependServiceStrPtr) > 0)
//             {
//                 if (_wcsicmp(lpszValueBuf + dwDependServiceStrPtr, lpService->lpServiceName) == 0)
//                 {
//                     /* Get the current enumed service pointer */
//                     lpCurrentService = ScmGetServiceEntryByName(lpszNameBuf);
//
//                     /* Check for valid Service */
//                     if (!lpCurrentService)
//                     {
//                         /* This should never happen! */
//                         DPRINT("This should not happen at this point, report to Developer\n");
//                         return ERROR_NOT_FOUND;
//                     }
//
//                     /* Determine state the service is in */
//                     if (lpCurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//                         dwCurrentServiceState = SERVICE_INACTIVE;
//
//                     /* If the ServiceState matches that requested or searching for SERVICE_STATE_ALL */
//                     if ((dwCurrentServiceState == dwServiceState) ||
//                         (dwServiceState == SERVICE_STATE_ALL))
//                     {
//                         /* Calculate the required size */
//                         dwRequiredSize += sizeof(SERVICE_STATUS);
//                         dwRequiredSize += (DWORD)((wcslen(lpCurrentService->lpServiceName) + 1) * sizeof(WCHAR));
//                         dwRequiredSize += (DWORD)((wcslen(lpCurrentService->lpDisplayName) + 1) * sizeof(WCHAR));
//
//                         /* Add the size for service name and display name pointers */
//                         dwRequiredSize += (2 * sizeof(PVOID));
//
//                         /* increase the BytesNeeded size */
//                         *pcbBytesNeeded = *pcbBytesNeeded + dwRequiredSize;
//
//                         /* Don't fill callers buffer yet, as MSDN read that the last service with dependency
//                            comes first */
//
//                         /* Recursive call to check for its dependencies */
//                         Int_EnumDependentServicesW(hServicesKey,
//                                                    lpCurrentService,
//                                                    dwServiceState,
//                                                    lpServices,
//                                                    pcbBytesNeeded,
//                                                    lpServicesReturned);
//
//                         /* If the lpServices is valid set the service pointer */
//                         if (lpServices)
//                             lpServices[*lpServicesReturned] = lpCurrentService;
//
//                         *lpServicesReturned = *lpServicesReturned + 1;
//                     }
//                 }
//
//                 dwDependServiceStrPtr += (DWORD)(wcslen(lpszValueBuf + dwDependServiceStrPtr) + 1);
//             }
//         }
//         else if (*pcbBytesNeeded)
//         {
//             dwError = ERROR_SUCCESS;
//         }
//
//         RegCloseKey(hServiceEnumKey);
//     }
//
//     return dwError;
// }
//
//
// /* Function 0 */
// DWORD
// WINAPI
// RCloseServiceHandle(
//     LPSC_RPC_HANDLE hSCObject)
// {
//     PMANAGER_HANDLE hManager;
//     PSERVICE_HANDLE hService;
//     PSERVICE lpService;
//
//     DPRINT("RCloseServiceHandle() called\n");
//
//     DPRINT("hSCObject = %p\n", *hSCObject);
//
//     if (*hSCObject == 0)
//         return ERROR_INVALID_HANDLE;
//
//     hManager = ScmGetServiceManagerFromHandle(*hSCObject);
//     hService = ScmGetServiceFromHandle(*hSCObject);
//
//     if (hManager != NULL)
//     {
//         DPRINT("Found manager handle\n");
//
//         /* Make sure we don't access stale memory if someone tries to use this handle again. */
//         hManager->Handle.Tag = INVALID_TAG;
//
//         HeapFree(GetProcessHeap(), 0, hManager);
//         hManager = NULL;
//
//         *hSCObject = NULL;
//
//         DPRINT("RCloseServiceHandle() done\n");
//         return ERROR_SUCCESS;
//     }
//     else if (hService != NULL)
//     {
//         DPRINT("Found service handle\n");
//
//         /* Lock the service database exclusively */
//         ScmLockDatabaseExclusive();
//
//         /* Get the pointer to the service record */
//         lpService = hService->ServiceEntry;
//
//         /* Make sure we don't access stale memory if someone tries to use this handle again. */
//         hService->Handle.Tag = INVALID_TAG;
//
//         /* Free the handle */
//         HeapFree(GetProcessHeap(), 0, hService);
//         hService = NULL;
//
//
//         DPRINT("Closing service %S with %d references\n", lpService->lpServiceName, lpService->RefCount);
//         ScmDereferenceService(lpService as *mut c_void);
//
//         ScmUnlockDatabase();
//
//         *hSCObject = NULL;
//
//         DPRINT("RCloseServiceHandle() done\n");
//         return ERROR_SUCCESS;
//     }
//
//     DPRINT("Invalid handle tag (Tag %lx)\n", hManager->Handle.Tag);
//
//     return ERROR_INVALID_HANDLE;
// }
//
//
// /* Function 1 */
// DWORD
// WINAPI
// RControlService(
//     SC_RPC_HANDLE hService,
//     DWORD dwControl,
//     LPSERVICE_STATUS lpServiceStatus)
// {
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//     ACCESS_MASK DesiredAccess;
//     DWORD dwError = ERROR_SUCCESS;
//     DWORD pcbBytesNeeded = 0;
//     DWORD dwServicesReturned = 0;
//     DWORD dwControlsAccepted;
//     DWORD dwCurrentState;
//     HKEY hServicesKey = NULL;
//     LPCWSTR lpLogStrings[2];
//     WCHAR szLogBuffer[80];
//     UINT uID;
//
//     DPRINT("RControlService() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     /* Check the service handle */
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Check the service entry point */
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT1("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Check access rights */
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_STOP:
//             DesiredAccess = SERVICE_STOP;
//             break;
//
//         case SERVICE_CONTROL_PAUSE:
//         case SERVICE_CONTROL_CONTINUE:
//         case SERVICE_CONTROL_PARAMCHANGE:
//         case SERVICE_CONTROL_NETBINDADD:
//         case SERVICE_CONTROL_NETBINDREMOVE:
//         case SERVICE_CONTROL_NETBINDENABLE:
//         case SERVICE_CONTROL_NETBINDDISABLE:
//             DesiredAccess = SERVICE_PAUSE_CONTINUE;
//             break;
//
//         case SERVICE_CONTROL_INTERROGATE:
//             DesiredAccess = SERVICE_INTERROGATE;
//             break;
//
//         default:
//             if (dwControl >= 128 && dwControl <= 255)
//                 DesiredAccess = SERVICE_USER_DEFINED_CONTROL;
//             else
//                 return ERROR_INVALID_PARAMETER;
//             break;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   DesiredAccess))
//         return ERROR_ACCESS_DENIED;
//
//     /* Return the current service status information */
//     RtlCopyMemory(lpServiceStatus,
//                   &lpService->Status,
//                   sizeof(SERVICE_STATUS));
//
//     if (dwControl == SERVICE_CONTROL_STOP)
//     {
//         /* Check if the service has dependencies running as windows
//            doesn't stop a service that does */
//
//         /* Open the Services Reg key */
//         dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                                 L"System\\CurrentControlSet\\Services",
//                                 0,
//                                 KEY_READ,
//                                 &hServicesKey) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//         {
//             DPRINT("Failed to open services key\n");
//             return dwError;
//         }
//
//         /* Call the internal function with NULL, just to get bytes we need */
//         Int_EnumDependentServicesW(hServicesKey,
//                                    lpService,
//                                    SERVICE_ACTIVE,
//                                    NULL,
//                                    &pcbBytesNeeded,
//                                    &dwServicesReturned);
//
//         RegCloseKey(hServicesKey);
//
//         /* If pcbBytesNeeded is not zero then there are services running that
//            are dependent on this service */
//         if (pcbBytesNeeded != 0)
//         {
//             DPRINT("Service has running dependencies. Failed to stop service\n");
//             return ERROR_DEPENDENT_SERVICES_RUNNING;
//         }
//     }
//
//     if (lpService->Status.dwServiceType & SERVICE_DRIVER)
//     {
//         /* Send control code to the driver */
//         dwError = ScmControlDriver(lpService,
//                                    dwControl,
//                                    lpServiceStatus);
//     }
//     else
//     {
//         dwControlsAccepted = lpService->Status.dwControlsAccepted;
//         dwCurrentState = lpService->Status.dwCurrentState;
//
//         /* Return ERROR_SERVICE_NOT_ACTIVE if the service has not been started */
//         if (lpService->lpImage == NULL || dwCurrentState == SERVICE_STOPPED)
//             return ERROR_SERVICE_NOT_ACTIVE;
//
//         /* Check the current state before sending a control request */
//         switch (dwCurrentState)
//         {
//             case SERVICE_STOP_PENDING:
//             case SERVICE_STOPPED:
//                 return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
//
//             case SERVICE_START_PENDING:
//                 switch (dwControl)
//                 {
//                     case SERVICE_CONTROL_STOP:
//                         break;
//
//                     case SERVICE_CONTROL_INTERROGATE:
//                         RtlCopyMemory(lpServiceStatus,
//                                       &lpService->Status,
//                                       sizeof(SERVICE_STATUS));
//                         return ERROR_SUCCESS;
//
//                     default:
//                         return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
//                 }
//                 break;
//         }
//
//         /* Check if the control code is acceptable to the service */
//         switch (dwControl)
//         {
//             case SERVICE_CONTROL_STOP:
//                 if ((dwControlsAccepted & SERVICE_ACCEPT_STOP) == 0)
//                     return ERROR_INVALID_SERVICE_CONTROL;
//                 break;
//
//             case SERVICE_CONTROL_PAUSE:
//             case SERVICE_CONTROL_CONTINUE:
//                 if ((dwControlsAccepted & SERVICE_ACCEPT_PAUSE_CONTINUE) == 0)
//                     return ERROR_INVALID_SERVICE_CONTROL;
//                 break;
//
//             case SERVICE_CONTROL_PARAMCHANGE:
//                 if ((dwControlsAccepted & SERVICE_ACCEPT_PARAMCHANGE) == 0)
//                     return ERROR_INVALID_SERVICE_CONTROL;
//                 break;
//
//             case SERVICE_CONTROL_NETBINDADD:
//             case SERVICE_CONTROL_NETBINDREMOVE:
//             case SERVICE_CONTROL_NETBINDENABLE:
//             case SERVICE_CONTROL_NETBINDDISABLE:
//                 if ((dwControlsAccepted & SERVICE_ACCEPT_NETBINDCHANGE) == 0)
//                     return ERROR_INVALID_SERVICE_CONTROL;
//                 break;
//         }
//
//         /* Send control code to the service */
//         dwError = ScmControlService(lpService->lpImage->hControlPipe,
//                                     lpService->lpServiceName,
//                                     dwControl,
//                                     (SERVICE_STATUS_HANDLE)lpService);
//
//         /* Return service status information */
//         RtlCopyMemory(lpServiceStatus,
//                       &lpService->Status,
//                       sizeof(SERVICE_STATUS));
//     }
//
//     if (dwError == ERROR_SUCCESS)
//     {
//         if (dwControl == SERVICE_CONTROL_STOP ||
//             dwControl == SERVICE_CONTROL_PAUSE ||
//             dwControl == SERVICE_CONTROL_CONTINUE)
//         {
//             /* Log a successful send control */
//
//             switch (dwControl)
//             {
//                 case SERVICE_CONTROL_STOP:
//                     uID = IDS_SERVICE_STOP;
//                     break;
//
//                 case SERVICE_CONTROL_PAUSE:
//                     uID = IDS_SERVICE_PAUSE;
//                     break;
//
//                 case SERVICE_CONTROL_CONTINUE:
//                     uID = IDS_SERVICE_RESUME;
//                     break;
//             }
//             LoadStringW(GetModuleHandleW(NULL), uID, szLogBuffer, ARRAYSIZE(szLogBuffer));
//
//             lpLogStrings[0] = lpService->lpDisplayName;
//             lpLogStrings[1] = szLogBuffer;
//
//             ScmLogEvent(EVENT_SERVICE_CONTROL_SUCCESS,
//                         EVENTLOG_INFORMATION_TYPE as u16,
//                         2,
//                         lpLogStrings);
//         }
//     }
//
//     return dwError;
// }
//
//
// /* Function 2 */
// DWORD
// WINAPI
// RDeleteService(
//     SC_RPC_HANDLE hService)
// {
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//     DWORD dwError;
//
//     DPRINT("RDeleteService() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   DELETE))
//         return ERROR_ACCESS_DENIED;
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     if (lpService->bDeleted)
//     {
//         DPRINT("Service has already been marked for delete\n");
//         dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
//         goto Done;
//     }
//
//     /* Mark service for delete */
//     lpService->bDeleted = TRUE;
//
//     dwError = ScmMarkServiceForDelete(lpService);
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("RDeleteService() done\n");
//
//     return dwError;
// }
//
//
// /* Function 3 */
// DWORD
// WINAPI
// RLockServiceDatabase(
//     SC_RPC_HANDLE hSCManager,
//     LPSC_RPC_LOCK lpLock)
// {
//     PMANAGER_HANDLE hMgr;
//
//     DPRINT("RLockServiceDatabase() called\n");
//
//     *lpLock = NULL;
//
//     hMgr = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hMgr == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hMgr->Handle.DesiredAccess,
//                                   SC_MANAGER_LOCK))
//         return ERROR_ACCESS_DENIED;
//
//     return ScmAcquireServiceStartLock(FALSE, lpLock);
// }
//
//
// /* Function 4 */
// DWORD
// WINAPI
// RQueryServiceObjectSecurity(
//     SC_RPC_HANDLE hService,
//     SECURITY_INFORMATION dwSecurityInformation,
//     LPBYTE lpSecurityDescriptor,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded)
// {
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//     ULONG DesiredAccess = 0;
//     NTSTATUS Status;
//     DWORD dwBytesNeeded;
//     DWORD dwError;
//
//     DPRINT("RQueryServiceObjectSecurity() called\n");
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (dwSecurityInformation & (DACL_SECURITY_INFORMATION |
//                                  GROUP_SECURITY_INFORMATION |
//                                  OWNER_SECURITY_INFORMATION))
//         DesiredAccess |= READ_CONTROL;
//
//     if (dwSecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   DesiredAccess))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database */
//     ScmLockDatabaseShared();
//
//     /* Retrieve the security descriptor */
//     Status = RtlQuerySecurityObject(lpService->pSecurityDescriptor,
//                                     dwSecurityInformation,
//                                     (PSECURITY_DESCRIPTOR)lpSecurityDescriptor,
//                                     cbBufSize,
//                                     &dwBytesNeeded);
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (NT_SUCCESS(Status))
//     {
//         *pcbBytesNeeded = dwBytesNeeded;
//         dwError = STATUS_SUCCESS;
//     }
//     else if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         *pcbBytesNeeded = dwBytesNeeded;
//         dwError = ERROR_INSUFFICIENT_BUFFER;
//     }
//     else if (Status == STATUS_BAD_DESCRIPTOR_FORMAT)
//     {
//         dwError = ERROR_GEN_FAILURE;
//     }
//     else
//     {
//         dwError = RtlNtStatusToDosError(Status);
//     }
//
//     return dwError;
// }
//
//
// /* Function 5 */
// DWORD
// WINAPI
// RSetServiceObjectSecurity(
//     SC_RPC_HANDLE hService,
//     DWORD dwSecurityInformation,
//     LPBYTE lpSecurityDescriptor,
//     DWORD dwSecurityDescriptorSize)
// {
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//     ACCESS_MASK DesiredAccess = 0;
//     HANDLE hToken = NULL;
//     HKEY hServiceKey = NULL;
//     BOOL bDatabaseLocked = FALSE;
//     NTSTATUS Status;
//     DWORD dwError;
//
//     DPRINT("RSetServiceObjectSecurity() called\n");
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (dwSecurityInformation == 0 ||
//         dwSecurityInformation & ~(OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION
//         | DACL_SECURITY_INFORMATION | SACL_SECURITY_INFORMATION))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if (!RtlValidSecurityDescriptor((PSECURITY_DESCRIPTOR)lpSecurityDescriptor))
//         return ERROR_INVALID_PARAMETER;
//
//     if (dwSecurityInformation & SACL_SECURITY_INFORMATION)
//         DesiredAccess |= ACCESS_SYSTEM_SECURITY;
//
//     if (dwSecurityInformation & DACL_SECURITY_INFORMATION)
//         DesiredAccess |= WRITE_DAC;
//
//     if (dwSecurityInformation & (OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION))
//         DesiredAccess |= WRITE_OWNER;
//
//     if ((dwSecurityInformation & OWNER_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)lpSecurityDescriptor)->Owner == NULL))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if ((dwSecurityInformation & GROUP_SECURITY_INFORMATION) &&
//         (((PISECURITY_DESCRIPTOR)lpSecurityDescriptor)->Group == NULL))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   DesiredAccess))
//     {
//         DPRINT1("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT1("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (lpService->bDeleted)
//         return ERROR_SERVICE_MARKED_FOR_DELETE;
//
//
//     /* Build the new security descriptor */
//     Status = RtlSetSecurityObject(dwSecurityInformation,
//                                   (PSECURITY_DESCRIPTOR)lpSecurityDescriptor,
//                                   &lpService->pSecurityDescriptor,
//                                   &ScmServiceMapping,
//                                   hToken);
//     if (!NT_SUCCESS(Status))
//     {
//         dwError = RtlNtStatusToDosError(Status);
//         goto Done;
//     }
//
//     /* Lock the service database exclusive */
//     ScmLockDatabaseExclusive();
//     bDatabaseLocked = TRUE;
//
//     /* Open the service key */
//     dwError = ScmOpenServiceKey(lpService->lpServiceName,
//                                 READ_CONTROL | KEY_CREATE_SUB_KEY | KEY_SET_VALUE,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* Store the new security descriptor */
//     dwError = ScmWriteSecurityDescriptor(hServiceKey,
//                                          lpService->pSecurityDescriptor);
//
//     RegFlushKey(hServiceKey);
//
// Done:
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     /* Unlock service database */
//     if (bDatabaseLocked == TRUE)
//         ScmUnlockDatabase();
//
//     if (hToken != NULL)
//         NtClose(hToken);
//
//     DPRINT("RSetServiceObjectSecurity() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 6 */
// DWORD
// WINAPI
// RQueryServiceStatus(
//     SC_RPC_HANDLE hService,
//     LPSERVICE_STATUS lpServiceStatus)
// {
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//
//     DPRINT("RQueryServiceStatus() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_STATUS))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     /* Return service status information */
//     RtlCopyMemory(lpServiceStatus,
//                   &lpService->Status,
//                   sizeof(SERVICE_STATUS));
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     return ERROR_SUCCESS;
// }
//
//
// static BOOL
// ScmIsValidServiceState(DWORD dwCurrentState)
// {
//     switch (dwCurrentState)
//     {
//         case SERVICE_STOPPED:
//         case SERVICE_START_PENDING:
//         case SERVICE_STOP_PENDING:
//         case SERVICE_RUNNING:
//         case SERVICE_CONTINUE_PENDING:
//         case SERVICE_PAUSE_PENDING:
//         case SERVICE_PAUSED:
//             return TRUE;
//
//         default:
//             return FALSE;
//     }
// }
//
// static
// DWORD
// WINAPI
// ScmStopThread(PVOID pParam)
// {
//     PSERVICE lpService = (PSERVICE)pParam;
//     WCHAR szLogBuffer[80];
//     LPCWSTR lpLogStrings[2];
//
//     /* Check if we are about to stop this service */
//     if (lpService->lpImage->dwImageRunCount == 1)
//     {
//         /* Stop the dispatcher thread.
//          * We must not send a control message while holding the database lock, otherwise it can cause timeouts
//          * We are sure that the service won't be deleted in the meantime because we still have a reference to it. */
//         DPRINT("Stopping the dispatcher thread for service %S\n", lpService->lpServiceName);
//         ScmControlService(lpService->lpImage->hControlPipe,
//                           L"",
//                           SERVICE_CONTROL_STOP,
//                           (SERVICE_STATUS_HANDLE)lpService);
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     DPRINT("Service %S image count:%d\n", lpService->lpServiceName, lpService->lpImage->dwImageRunCount);
//
//     /* Decrement the image run counter */
//     lpService->lpImage->dwImageRunCount--;
//
//     /* If we just stopped the last running service... */
//     if (lpService->lpImage->dwImageRunCount == 0)
//     {
//         /* Remove the service image */
//         DPRINT("Removing service image for %S\n", lpService->lpServiceName);
//         ScmRemoveServiceImage(lpService->lpImage);
//         lpService->lpImage = NULL;
//     }
//
//     /* Report the results of the status change here */
//     if (lpService->Status.dwWin32ExitCode != ERROR_SUCCESS)
//     {
//         /* Log a failed service stop */
//         StringCchPrintfW(szLogBuffer, ARRAYSIZE(szLogBuffer),
//                          L"%lu", lpService->Status.dwWin32ExitCode);
//         lpLogStrings[0] = lpService->lpDisplayName;
//         lpLogStrings[1] = szLogBuffer;
//
//         ScmLogEvent(EVENT_SERVICE_EXIT_FAILED,
//                     EVENTLOG_ERROR_TYPE as u16,
//                     ARRAYSIZE(lpLogStrings),
//                     lpLogStrings);
//     }
//     else
//     {
//         /* Log a successful service status change */
//         LoadStringW(GetModuleHandleW(NULL), IDS_SERVICE_STOPPED, szLogBuffer, ARRAYSIZE(szLogBuffer));
//         lpLogStrings[0] = lpService->lpDisplayName;
//         lpLogStrings[1] = szLogBuffer;
//
//         ScmLogEvent(EVENT_SERVICE_STATUS_SUCCESS,
//                     EVENTLOG_INFORMATION_TYPE as u16,
//                     ARRAYSIZE(lpLogStrings),
//                     lpLogStrings);
//     }
//
//     /* Remove the reference that was added when the service started */
//     DPRINT("Service %S has %d references while stoping\n", lpService->lpServiceName, lpService->RefCount);
//     ScmDereferenceService(lpService as *mut c_void);
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     return 0;
// }
//
// /* Function 7 */
// DWORD
// WINAPI
// RSetServiceStatus(
//     RPC_SERVICE_STATUS_HANDLE hServiceStatus,
//     LPSERVICE_STATUS lpServiceStatus)
// {
//     PSERVICE lpService;
//     DWORD dwPreviousState;
//     DWORD dwPreviousType;
//     LPCWSTR lpLogStrings[2];
//     WCHAR szLogBuffer[80];
//     UINT uID;
//
//     DPRINT("RSetServiceStatus() called\n");
//     DPRINT("hServiceStatus = %lu\n", hServiceStatus);
//     DPRINT("dwServiceType = 0x%lx\n", lpServiceStatus->dwServiceType);
//     DPRINT("dwCurrentState = %lu\n", lpServiceStatus->dwCurrentState);
//     DPRINT("dwControlsAccepted = %lu\n", lpServiceStatus->dwControlsAccepted);
//     DPRINT("dwWin32ExitCode = %lu\n", lpServiceStatus->dwWin32ExitCode);
//     DPRINT("dwServiceSpecificExitCode = %lu\n", lpServiceStatus->dwServiceSpecificExitCode);
//     DPRINT("dwCheckPoint = %lu\n", lpServiceStatus->dwCheckPoint);
//     DPRINT("dwWaitHint = %lu\n", lpServiceStatus->dwWaitHint);
//
//     if (hServiceStatus == 0)
//     {
//         DPRINT("hServiceStatus == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     lpService = (PSERVICE)hServiceStatus;
//
//     /* Check current state */
//     if (!ScmIsValidServiceState(lpServiceStatus->dwCurrentState))
//     {
//         DPRINT("Invalid service state\n");
//         return ERROR_INVALID_DATA;
//     }
//
//     /* Check service type */
//     if (!(lpServiceStatus->dwServiceType & SERVICE_WIN32) &&
//          (lpServiceStatus->dwServiceType & SERVICE_DRIVER))
//     {
//         DPRINT("Invalid service type\n");
//         return ERROR_INVALID_DATA;
//     }
//
//     /* Check accepted controls */
//     if (lpServiceStatus->dwControlsAccepted & ~0xFF)
//     {
//         DPRINT("Invalid controls accepted\n");
//         return ERROR_INVALID_DATA;
//     }
//
//     /* Set the wait hint and check point only if the service is in a pending state,
//        otherwise they should be 0 */
//     if (lpServiceStatus->dwCurrentState == SERVICE_STOPPED ||
//         lpServiceStatus->dwCurrentState == SERVICE_PAUSED ||
//         lpServiceStatus->dwCurrentState == SERVICE_RUNNING)
//     {
//         lpServiceStatus->dwWaitHint = 0;
//         lpServiceStatus->dwCheckPoint = 0;
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     /* Save the current service state */
//     dwPreviousState = lpService->Status.dwCurrentState;
//
//     /* Save the current service type */
//     dwPreviousType = lpService->Status.dwServiceType;
//
//     /* Update the service status */
//     RtlCopyMemory(&lpService->Status,
//                   lpServiceStatus,
//                   sizeof(SERVICE_STATUS));
//
//     /* Restore the previous service type */
//     lpService->Status.dwServiceType = dwPreviousType;
//
//     DPRINT("Service %S changed state %d to %d\n", lpService->lpServiceName, dwPreviousState, lpServiceStatus->dwCurrentState);
//
//     if (lpServiceStatus->dwCurrentState != SERVICE_STOPPED &&
//         dwPreviousState == SERVICE_STOPPED)
//     {
//         /* Keep a reference on all non stopped services */
//         ScmReferenceService(lpService);
//         DPRINT("Service %S has %d references after starting\n", lpService->lpServiceName, lpService->RefCount);
//     }
//
//     /* Check if the service just stopped */
//     if (lpServiceStatus->dwCurrentState == SERVICE_STOPPED &&
//         dwPreviousState != SERVICE_STOPPED)
//     {
//         HANDLE hStopThread;
//         DWORD dwStopThreadId;
//         DPRINT("Service %S, currentstate: %d, prev: %d\n", lpService->lpServiceName, lpServiceStatus->dwCurrentState, dwPreviousState);
//
//         /*
//          * The service just changed its status to stopped.
//          * Create a thread that will complete the stop sequence.
//          * This thread will remove the reference that was added when the service started.
//          * This will ensure that the service will remain valid as long as this reference is still held.
//          */
//         hStopThread = CreateThread(NULL,
//                                    0,
//                                    ScmStopThread,
//                                    (LPVOID)lpService,
//                                    0,
//                                    &dwStopThreadId);
//         if (hStopThread == NULL)
//         {
//             DPRINT1("Failed to create thread to complete service stop\n");
//             /* We can't leave without releasing the reference.
//              * We also can't remove it without holding the lock. */
//             ScmDereferenceService(lpService as *mut c_void);
//             DPRINT1("Service %S has %d references after stop\n", lpService->lpServiceName, lpService->RefCount);
//         }
//         else
//         {
//             CloseHandle(hStopThread);
//         }
//     }
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     /* Don't log any events here regarding a service stop as it can become invalid at any time */
//
//     if (lpServiceStatus->dwCurrentState != dwPreviousState &&
//         (lpServiceStatus->dwCurrentState == SERVICE_RUNNING ||
//          lpServiceStatus->dwCurrentState == SERVICE_PAUSED))
//     {
//         /* Log a successful service status change */
//         switch(lpServiceStatus->dwCurrentState)
//         {
//             case SERVICE_RUNNING:
//                 uID = IDS_SERVICE_RUNNING;
//                 break;
//
//             case SERVICE_PAUSED:
//                 uID = IDS_SERVICE_PAUSED;
//                 break;
//         }
//
//         LoadStringW(GetModuleHandleW(NULL), uID, szLogBuffer, ARRAYSIZE(szLogBuffer));
//         lpLogStrings[0] = lpService->lpDisplayName;
//         lpLogStrings[1] = szLogBuffer;
//
//         ScmLogEvent(EVENT_SERVICE_STATUS_SUCCESS,
//                     EVENTLOG_INFORMATION_TYPE as u16,
//                     2,
//                     lpLogStrings);
//     }
//
//     DPRINT("Set %S to %lu\n", lpService->lpDisplayName, lpService->Status.dwCurrentState);
//     DPRINT("RSetServiceStatus() done\n");
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 8 */
// DWORD
// WINAPI
// RUnlockServiceDatabase(
//     LPSC_RPC_LOCK Lock)
// {
//     DPRINT("RUnlockServiceDatabase(%p)\n", Lock);
//     return ScmReleaseServiceStartLock(Lock);
// }
//
//
// /* Function 9 */
// DWORD
// WINAPI
// RNotifyBootConfigStatus(
//     SVCCTL_HANDLEW lpMachineName,
//     DWORD BootAcceptable)
// {
//     DPRINT("RNotifyBootConfigStatus(%p %lu)\n",
//            lpMachineName, BootAcceptable);
//
//     if (BootAcceptable)
//         return ScmAcceptBoot();
//
//     return ScmRunLastKnownGood();
// }
//
//
// /* Function 10 */
// DWORD
// WINAPI
// RI_ScSetServiceBitsW(
//     RPC_SERVICE_STATUS_HANDLE hServiceStatus,
//     DWORD dwServiceBits,
//     int bSetBitsOn,
//     int bUpdateImmediately,
//     wchar_t *lpString)
// {
//     PSERVICE pService;
//
//     DPRINT("RI_ScSetServiceBitsW(%p %lx %d %d %S)\n",
//            hServiceStatus, dwServiceBits, bSetBitsOn,
//            bUpdateImmediately, lpString);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if (lpString != NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     if (hServiceStatus == 0)
//     {
//         DPRINT("hServiceStatus == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     // FIXME: Validate the status handle
//     pService = (PSERVICE)hServiceStatus;
//
//     if (bSetBitsOn)
//     {
//         DPRINT("Old service bits: %08lx\n", pService->dwServiceBits);
//         DPRINT("Old global service bits: %08lx\n", g_dwServiceBits);
//         pService->dwServiceBits |= dwServiceBits;
//         g_dwServiceBits |= dwServiceBits;
//         DPRINT("New service bits: %08lx\n", pService->dwServiceBits);
//         DPRINT("New global service bits: %08lx\n", g_dwServiceBits);
//     }
//     else
//     {
//         DPRINT("Old service bits: %08lx\n", pService->dwServiceBits);
//         DPRINT("Old global service bits: %08lx\n", g_dwServiceBits);
//         pService->dwServiceBits &= ~dwServiceBits;
//         g_dwServiceBits &= ~dwServiceBits;
//         DPRINT("New service bits: %08lx\n", pService->dwServiceBits);
//         DPRINT("New global service bits: %08lx\n", g_dwServiceBits);
//     }
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 11 */
// DWORD
// WINAPI
// RChangeServiceConfigW(
//     SC_RPC_HANDLE hService,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPWSTR lpBinaryPathName,
//     LPWSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPWSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPWSTR lpDisplayName)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     LPWSTR lpDisplayNameW = NULL;
//     LPWSTR lpImagePathW = NULL;
//     LPWSTR lpClearTextPassword = NULL;
//
//     DPRINT("RChangeServiceConfigW() called\n");
//     DPRINT("dwServiceType = 0x%lx\n", dwServiceType);
//     DPRINT("dwStartType = %lu\n", dwStartType);
//     DPRINT("dwErrorControl = %lu\n", dwErrorControl);
//     DPRINT("lpBinaryPathName = %S\n", lpBinaryPathName);
//     DPRINT("lpLoadOrderGroup = %S\n", lpLoadOrderGroup);
//     DPRINT("lpServiceStartName = %S\n", lpServiceStartName);
//     DPRINT("lpPassword = %p\n", lpPassword);
//     DPRINT("dwPwSite = %lu\n", dwPwSize);
//     DPRINT("lpDisplayName = %S\n", lpDisplayName);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_CHANGE_CONFIG))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* Check for invalid service type value */
//     if ((dwServiceType != SERVICE_NO_CHANGE) &&
//         (dwServiceType != SERVICE_KERNEL_DRIVER) &&
//         (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER) &&
//         ((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS) &&
//         ((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check for invalid start type value */
//     if ((dwStartType != SERVICE_NO_CHANGE) &&
//         (dwStartType != SERVICE_BOOT_START) &&
//         (dwStartType != SERVICE_SYSTEM_START) &&
//         (dwStartType != SERVICE_AUTO_START) &&
//         (dwStartType != SERVICE_DEMAND_START) &&
//         (dwStartType != SERVICE_DISABLED))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Only drivers can be boot start or system start services */
//     if ((dwStartType == SERVICE_BOOT_START) ||
//         (dwStartType == SERVICE_SYSTEM_START))
//     {
//         if ((dwServiceType != SERVICE_KERNEL_DRIVER) &&
//             (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER))
//             return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check for invalid error control value */
//     if ((dwErrorControl != SERVICE_NO_CHANGE) &&
//         (dwErrorControl != SERVICE_ERROR_IGNORE) &&
//         (dwErrorControl != SERVICE_ERROR_NORMAL) &&
//         (dwErrorControl != SERVICE_ERROR_SEVERE) &&
//         (dwErrorControl != SERVICE_ERROR_CRITICAL))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if (lpdwTagId && (!lpLoadOrderGroup || !*lpLoadOrderGroup))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     if (lpService->bDeleted)
//     {
//         DPRINT("Service has already been marked for delete\n");
//         dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
//         goto done;
//     }
//
//     /* Open the service key */
//     dwError = ScmOpenServiceKey(lpService->szServiceName,
//                                 KEY_SET_VALUE,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Write service data to the registry */
//
//     /* Set the display name */
//     if (lpDisplayName != NULL && *lpDisplayName != 0)
//     {
//         RegSetValueExW(hServiceKey,
//                        L"DisplayName",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)lpDisplayName,
//                        (DWORD)((wcslen(lpDisplayName) + 1) * sizeof(WCHAR)));
//
//         /* Update the display name */
//         lpDisplayNameW = HeapAlloc(GetProcessHeap(),
//                                    HEAP_ZERO_MEMORY,
//                                    (wcslen(lpDisplayName) + 1) * sizeof(WCHAR));
//         if (lpDisplayNameW == NULL)
//         {
//             dwError = ERROR_NOT_ENOUGH_MEMORY;
//             goto done;
//         }
//
//         wcscpy(lpDisplayNameW, lpDisplayName);
//         if (lpService->lpDisplayName != lpService->lpServiceName)
//             HeapFree(GetProcessHeap(), 0, lpService->lpDisplayName);
//
//         lpService->lpDisplayName = lpDisplayNameW;
//     }
//
//     if (dwServiceType != SERVICE_NO_CHANGE)
//     {
//         /* Set the service type */
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Type",
//                                  0,
//                                  REG_DWORD,
//                                  (LPBYTE)&dwServiceType,
//                                  sizeof(DWORD)) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         lpService->Status.dwServiceType = dwServiceType;
//     }
//
//     if (dwStartType != SERVICE_NO_CHANGE)
//     {
//         /* Set the start value */
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Start",
//                                  0,
//                                  REG_DWORD,
//                                  (LPBYTE)&dwStartType,
//                                  sizeof(DWORD)) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         lpService->dwStartType = dwStartType;
//     }
//
//     if (dwErrorControl != SERVICE_NO_CHANGE)
//     {
//         /* Set the error control value */
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"ErrorControl",
//                                  0,
//                                  REG_DWORD,
//                                  (LPBYTE)&dwErrorControl,
//                                  sizeof(DWORD)) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         lpService->dwErrorControl = dwErrorControl;
//     }
//
//     if (lpBinaryPathName != NULL && *lpBinaryPathName != 0)
//     {
//         /* Set the image path */
//         lpImagePathW = lpBinaryPathName;
//
//         if (lpService->Status.dwServiceType & SERVICE_DRIVER)
//         {
//             dwError = ScmCanonDriverImagePath(lpService->dwStartType,
//                                               lpBinaryPathName,
//                                               &lpImagePathW);
//
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//         }
//
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"ImagePath",
//                                  0,
//                                  REG_EXPAND_SZ,
//                                  (LPBYTE)lpImagePathW,
//                                  (DWORD)((wcslen(lpImagePathW) + 1) * sizeof(WCHAR))) as DWORD;
//
//         if (lpImagePathW != lpBinaryPathName)
//             HeapFree(GetProcessHeap(), 0, lpImagePathW);
//
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Set the group name */
//     if (lpLoadOrderGroup != NULL && *lpLoadOrderGroup != 0)
//     {
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Group",
//                                  0,
//                                  REG_SZ,
//                                  (LPBYTE)lpLoadOrderGroup,
//                                  (DWORD)((wcslen(lpLoadOrderGroup) + 1) * sizeof(WCHAR))) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         dwError = ScmSetServiceGroup(lpService,
//                                      lpLoadOrderGroup);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Set the tag */
//     if (lpdwTagId != NULL)
//     {
//         dwError = ScmAssignNewTag(lpService);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Tag",
//                                  0,
//                                  REG_DWORD,
//                                  (LPBYTE)&lpService->dwTag,
//                                  sizeof(DWORD)) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         *lpdwTagId = lpService->dwTag;
//     }
//
//     /* Write dependencies */
//     if (lpDependencies != NULL && *lpDependencies != 0)
//     {
//         dwError = ScmWriteDependencies(hServiceKey,
//                                        (LPWSTR)lpDependencies,
//                                        dwDependSize);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Start name and password are only used by Win32 services */
//     if (lpService->Status.dwServiceType & SERVICE_WIN32)
//     {
//         /* Write service start name */
//         if (lpServiceStartName != NULL && *lpServiceStartName != 0)
//         {
//             dwError = RegSetValueExW(hServiceKey,
//                                      L"ObjectName",
//                                      0,
//                                      REG_SZ,
//                                      (LPBYTE)lpServiceStartName,
//                                      (DWORD)((wcslen(lpServiceStartName) + 1) * sizeof(WCHAR))) as DWORD;
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//         }
//
//         if (lpPassword != NULL)
//         {
//             if (*(LPWSTR)lpPassword != 0)
//             {
//                 /* Decrypt the password */
//                 dwError = ScmDecryptPassword(hService,
//                                              lpPassword,
//                                              dwPwSize,
//                                              &lpClearTextPassword);
//                 if (dwError != ERROR_SUCCESS)
//                 {
//                     DPRINT1("ScmDecryptPassword failed (Error %lu)\n", dwError);
//                     goto done;
//                 }
//                 DPRINT("Clear text password: %S\n", lpClearTextPassword);
//
//                 /* Write the password */
//                 dwError = ScmSetServicePassword(lpService->szServiceName,
//                                                 lpClearTextPassword);
//                 if (dwError != ERROR_SUCCESS)
//                 {
//                     DPRINT1("ScmSetServicePassword failed (Error %lu)\n", dwError);
//                     goto done;
//                 }
//             }
//             else
//             {
//                 /* Delete the password */
//                 dwError = ScmSetServicePassword(lpService->szServiceName,
//                                                 NULL);
//                 if (dwError == ERROR_FILE_NOT_FOUND)
//                     dwError = ERROR_SUCCESS;
//
//                 if (dwError != ERROR_SUCCESS)
//                 {
//                     DPRINT1("ScmSetServicePassword failed (Error %lu)\n", dwError);
//                     goto done;
//                 }
//             }
//         }
//     }
//
// done:
//     if (lpClearTextPassword != NULL)
//     {
//         /* Wipe and release the password buffer */
//         SecureZeroMemory(lpClearTextPassword,
//                          (wcslen(lpClearTextPassword) + 1) * sizeof(WCHAR));
//         HeapFree(GetProcessHeap(), 0, lpClearTextPassword);
//     }
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("RChangeServiceConfigW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 12 */
// DWORD
// WINAPI
// RCreateServiceW(
//     SC_RPC_HANDLE hSCManager,
//     LPCWSTR lpServiceName,
//     LPCWSTR lpDisplayName,
//     DWORD dwDesiredAccess,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPCWSTR lpBinaryPathName,
//     LPCWSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPCWSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     PMANAGER_HANDLE hManager;
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE lpService = NULL;
//     SC_HANDLE hServiceHandle = NULL;
//     LPWSTR lpImagePath = NULL;
//     LPWSTR lpClearTextPassword = NULL;
//     HKEY hServiceKey = NULL;
//     LPWSTR lpObjectName;
//
//     DPRINT("RCreateServiceW() called\n");
//     DPRINT("lpServiceName = %S\n", lpServiceName);
//     DPRINT("lpDisplayName = %S\n", lpDisplayName);
//     DPRINT("dwDesiredAccess = %lx\n", dwDesiredAccess);
//     DPRINT("dwServiceType = 0x%lx\n", dwServiceType);
//     DPRINT("dwStartType = %lu\n", dwStartType);
//     DPRINT("dwErrorControl = %lu\n", dwErrorControl);
//     DPRINT("lpBinaryPathName = %S\n", lpBinaryPathName);
//     DPRINT("lpLoadOrderGroup = %S\n", lpLoadOrderGroup);
//     DPRINT("lpdwTagId = %p\n", lpdwTagId);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_CREATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n",
//                hManager->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     if (*lpServiceName == 0)
//         return ERROR_INVALID_NAME;
//
//     if (*lpBinaryPathName == 0)
//         return ERROR_INVALID_PARAMETER;
//
//     /* Check for invalid service type value */
//     if ((dwServiceType != SERVICE_KERNEL_DRIVER) &&
//         (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER) &&
//         ((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_OWN_PROCESS) &&
//         ((dwServiceType & ~SERVICE_INTERACTIVE_PROCESS) != SERVICE_WIN32_SHARE_PROCESS))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check for invalid start type value */
//     if ((dwStartType != SERVICE_BOOT_START) &&
//         (dwStartType != SERVICE_SYSTEM_START) &&
//         (dwStartType != SERVICE_AUTO_START) &&
//         (dwStartType != SERVICE_DEMAND_START) &&
//         (dwStartType != SERVICE_DISABLED))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Only drivers can be boot start or system start services */
//     if ((dwStartType == SERVICE_BOOT_START) ||
//         (dwStartType == SERVICE_SYSTEM_START))
//     {
//         if ((dwServiceType != SERVICE_KERNEL_DRIVER) &&
//             (dwServiceType != SERVICE_FILE_SYSTEM_DRIVER))
//         {
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//
//     /* Check for invalid error control value */
//     if ((dwErrorControl != SERVICE_ERROR_IGNORE) &&
//         (dwErrorControl != SERVICE_ERROR_NORMAL) &&
//         (dwErrorControl != SERVICE_ERROR_SEVERE) &&
//         (dwErrorControl != SERVICE_ERROR_CRITICAL))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if ((dwServiceType == (SERVICE_WIN32_OWN_PROCESS | SERVICE_INTERACTIVE_PROCESS)) &&
//         (lpServiceStartName))
//     {
//         /* We allow LocalSystem to run interactive. */
//         if (_wcsicmp(lpServiceStartName, L"LocalSystem"))
//         {
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//
//     if (lpdwTagId && (!lpLoadOrderGroup || !*lpLoadOrderGroup))
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     lpService = ScmGetServiceEntryByName(lpServiceName);
//     if (lpService)
//     {
//         /* Unlock the service database */
//         ScmUnlockDatabase();
//
//         /* Check if it is marked for deletion */
//         if (lpService->bDeleted)
//             return ERROR_SERVICE_MARKED_FOR_DELETE;
//
//         /* Return service-exists error */
//         return ERROR_SERVICE_EXISTS;
//     }
//
//     if (lpDisplayName != NULL &&
//         ScmGetServiceEntryByDisplayName(lpDisplayName) != NULL)
//     {
//         /* Unlock the service database */
//         ScmUnlockDatabase();
//
//         return ERROR_DUPLICATE_SERVICE_NAME;
//     }
//
//     if (dwServiceType & SERVICE_DRIVER)
//     {
//         dwError = ScmCanonDriverImagePath(dwStartType,
//                                           lpBinaryPathName,
//                                           &lpImagePath);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//     else
//     {
//         if (dwStartType == SERVICE_BOOT_START ||
//             dwStartType == SERVICE_SYSTEM_START)
//         {
//             /* Unlock the service database */
//             ScmUnlockDatabase();
//
//             return ERROR_INVALID_PARAMETER;
//         }
//     }
//
//     /* Allocate a new service entry */
//     dwError = ScmCreateNewServiceRecord(lpServiceName,
//                                         &lpService,
//                                         dwServiceType,
//                                         dwStartType);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Fill the new service entry */
//     lpService->dwErrorControl = dwErrorControl;
//
//     /* Fill the display name */
//     if (lpDisplayName != NULL &&
//         *lpDisplayName != 0 &&
//         _wcsicmp(lpService->lpDisplayName, lpDisplayName) != 0)
//     {
//         lpService->lpDisplayName = HeapAlloc(GetProcessHeap(),
//                                              HEAP_ZERO_MEMORY,
//                                              (wcslen(lpDisplayName) + 1) * sizeof(WCHAR));
//         if (lpService->lpDisplayName == NULL)
//         {
//             dwError = ERROR_NOT_ENOUGH_MEMORY;
//             goto done;
//         }
//         wcscpy(lpService->lpDisplayName, lpDisplayName);
//     }
//
//     /* Assign the service to a group */
//     if (lpLoadOrderGroup != NULL && *lpLoadOrderGroup != 0)
//     {
//         dwError = ScmSetServiceGroup(lpService,
//                                      lpLoadOrderGroup);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Assign a new tag */
//     if (lpdwTagId != NULL)
//     {
//         dwError = ScmAssignNewTag(lpService);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Assign the default security descriptor */
//     if (dwServiceType & SERVICE_WIN32)
//     {
//         dwError = ScmCreateDefaultServiceSD(&lpService->pSecurityDescriptor);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Write service data to the registry */
//     /* Create the service key */
//     dwError = ScmCreateServiceKey(lpServiceName,
//                                   KEY_WRITE,
//                                   &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Set the display name */
//     if (lpDisplayName != NULL && *lpDisplayName != 0)
//     {
//         RegSetValueExW(hServiceKey,
//                        L"DisplayName",
//                        0,
//                        REG_SZ,
//                        (LPBYTE)lpDisplayName,
//                        (DWORD)((wcslen(lpDisplayName) + 1) * sizeof(WCHAR)));
//     }
//
//     /* Set the service type */
//     dwError = RegSetValueExW(hServiceKey,
//                              L"Type",
//                              0,
//                              REG_DWORD,
//                              (LPBYTE)&dwServiceType,
//                              sizeof(DWORD)) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Set the start value */
//     dwError = RegSetValueExW(hServiceKey,
//                              L"Start",
//                              0,
//                              REG_DWORD,
//                              (LPBYTE)&dwStartType,
//                              sizeof(DWORD)) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Set the error control value */
//     dwError = RegSetValueExW(hServiceKey,
//                              L"ErrorControl",
//                              0,
//                              REG_DWORD,
//                              (LPBYTE)&dwErrorControl,
//                              sizeof(DWORD)) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     /* Set the image path */
//     if (dwServiceType & SERVICE_WIN32)
//     {
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"ImagePath",
//                                  0,
//                                  REG_EXPAND_SZ,
//                                  (LPBYTE)lpBinaryPathName,
//                                  (DWORD)((wcslen(lpBinaryPathName) + 1) * sizeof(WCHAR))) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//     else if (dwServiceType & SERVICE_DRIVER)
//     {
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"ImagePath",
//                                  0,
//                                  REG_EXPAND_SZ,
//                                  (LPBYTE)lpImagePath,
//                                  (DWORD)((wcslen(lpImagePath) + 1) * sizeof(WCHAR))) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Set the group name */
//     if (lpLoadOrderGroup != NULL && *lpLoadOrderGroup != 0)
//     {
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Group",
//                                  0,
//                                  REG_SZ,
//                                  (LPBYTE)lpLoadOrderGroup,
//                                  (DWORD)((wcslen(lpLoadOrderGroup) + 1) * sizeof(WCHAR))) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Set the service tag */
//     if (lpdwTagId != NULL)
//     {
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"Tag",
//                                  0,
//                                  REG_DWORD,
//                                  (LPBYTE)&lpService->dwTag,
//                                  sizeof(DWORD)) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Write dependencies */
//     if (lpDependencies != NULL && *lpDependencies != 0)
//     {
//         dwError = ScmWriteDependencies(hServiceKey,
//                                        (LPCWSTR)lpDependencies,
//                                        dwDependSize);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     /* Start name and password are only used by Win32 services */
//     if (dwServiceType & SERVICE_WIN32)
//     {
//         /* Write service start name */
//         lpObjectName = (lpServiceStartName != NULL) ? (LPWSTR)lpServiceStartName : L"LocalSystem";
//         dwError = RegSetValueExW(hServiceKey,
//                                  L"ObjectName",
//                                  0,
//                                  REG_SZ,
//                                  (LPBYTE)lpObjectName,
//                                  (DWORD)((wcslen(lpObjectName) + 1) * sizeof(WCHAR))) as DWORD;
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//
//         if (lpPassword != NULL && *(LPWSTR)lpPassword != 0)
//         {
//             /* Decrypt the password */
//             dwError = ScmDecryptPassword(hSCManager,
//                                          lpPassword,
//                                          dwPwSize,
//                                          &lpClearTextPassword);
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//
//             /* Write the password */
//             dwError = ScmSetServicePassword(lpServiceName,
//                                             lpClearTextPassword);
//             if (dwError != ERROR_SUCCESS)
//                 goto done;
//         }
//
//         /* Write the security descriptor */
//         dwError = ScmWriteSecurityDescriptor(hServiceKey,
//                                              lpService->pSecurityDescriptor);
//         if (dwError != ERROR_SUCCESS)
//             goto done;
//     }
//
//     dwError = ScmCreateServiceHandle(lpService,
//                                      &hServiceHandle);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     dwError = ScmCheckAccess(hServiceHandle,
//                              dwDesiredAccess);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     ScmReferenceService(lpService);
//
//     /* Get the service tag (if Win32) */
//     ScmGenerateServiceTag(lpService);
//
//     DPRINT("CreateService - lpService->RefCount %u\n", lpService->RefCount);
//
// done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     if (lpClearTextPassword != NULL)
//     {
//         /* Wipe and release the password buffer */
//         SecureZeroMemory(lpClearTextPassword,
//                          (wcslen(lpClearTextPassword) + 1) * sizeof(WCHAR));
//         HeapFree(GetProcessHeap(), 0, lpClearTextPassword);
//     }
//
//     if (dwError == ERROR_SUCCESS)
//     {
//         DPRINT("hService %p\n", hServiceHandle);
//         *lpServiceHandle = (SC_RPC_HANDLE)hServiceHandle;
//
//         if (lpdwTagId != NULL)
//             *lpdwTagId = lpService->dwTag;
//     }
//     else
//     {
//         if (lpService != NULL &&
//             lpService->lpServiceName != NULL)
//         {
//             /* Release the display name buffer */
//             HeapFree(GetProcessHeap(), 0, lpService->lpDisplayName);
//         }
//
//         if (hServiceHandle)
//         {
//             /* Remove the service handle */
//             HeapFree(GetProcessHeap(), 0, hServiceHandle);
//         }
//
//         if (lpService != NULL)
//         {
//             /* FIXME: remove the service entry */
//         }
//     }
//
//     if (lpImagePath != NULL)
//         HeapFree(GetProcessHeap(), 0, lpImagePath);
//
//     DPRINT("RCreateServiceW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 13 */
// DWORD
// WINAPI
// REnumDependentServicesW(
//     SC_RPC_HANDLE hService,
//     DWORD dwServiceState,
//     LPBYTE lpServices,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     DWORD dwServicesReturned = 0;
//     DWORD dwServiceCount;
//     HKEY hServicesKey = NULL;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     PSERVICE *lpServicesArray = NULL;
//     LPENUM_SERVICE_STATUSW lpServicesPtr = NULL;
//     LPWSTR lpStr;
//
//     *pcbBytesNeeded = 0;
//     *lpServicesReturned = 0;
//
//     DPRINT("REnumDependentServicesW() called\n");
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     lpService = hSvc->ServiceEntry;
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n",
//                hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* Open the Services Reg key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_READ,
//                             &hServicesKey) as DWORD;
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     /* First determine the bytes needed and get the number of dependent services */
//     dwError = Int_EnumDependentServicesW(hServicesKey,
//                                          lpService,
//                                          dwServiceState,
//                                          NULL,
//                                          pcbBytesNeeded,
//                                          &dwServicesReturned);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* If buffer size is less than the bytes needed or pointer is null */
//     if ((!lpServices) || (cbBufSize < *pcbBytesNeeded))
//     {
//         dwError = ERROR_MORE_DATA;
//         goto Done;
//     }
//
//     /* Allocate memory for array of service pointers */
//     lpServicesArray = HeapAlloc(GetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 (dwServicesReturned + 1) * sizeof(PSERVICE));
//     if (!lpServicesArray)
//     {
//         DPRINT1("Could not allocate buffer\n");
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto Done;
//     }
//
//     dwServicesReturned = 0;
//     *pcbBytesNeeded = 0;
//
//     dwError = Int_EnumDependentServicesW(hServicesKey,
//                                          lpService,
//                                          dwServiceState,
//                                          lpServicesArray,
//                                          pcbBytesNeeded,
//                                          &dwServicesReturned);
//     if (dwError != ERROR_SUCCESS)
//     {
//         goto Done;
//     }
//
//     lpServicesPtr = (LPENUM_SERVICE_STATUSW)lpServices;
//     lpStr = (LPWSTR)(lpServices + (dwServicesReturned * sizeof(ENUM_SERVICE_STATUSW)));
//
//     /* Copy EnumDepenedentService to Buffer */
//     for (dwServiceCount = 0; dwServiceCount < dwServicesReturned; dwServiceCount++)
//     {
//         lpService = lpServicesArray[dwServiceCount];
//
//         /* Copy status info */
//         memcpy(&lpServicesPtr->ServiceStatus,
//                &lpService->Status,
//                sizeof(SERVICE_STATUS));
//
//         /* Copy display name */
//         wcscpy(lpStr, lpService->lpDisplayName);
//         lpServicesPtr->lpDisplayName = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServices);
//         lpStr += (wcslen(lpService->lpDisplayName) + 1);
//
//         /* Copy service name */
//         wcscpy(lpStr, lpService->lpServiceName);
//         lpServicesPtr->lpServiceName = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServices);
//         lpStr += (wcslen(lpService->lpServiceName) + 1);
//
//         lpServicesPtr++;
//     }
//
//     *lpServicesReturned = dwServicesReturned;
//
// Done:
//     if (lpServicesArray != NULL)
//         HeapFree(GetProcessHeap(), 0, lpServicesArray);
//
//     RegCloseKey(hServicesKey);
//
//     DPRINT("REnumDependentServicesW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 14 */
// DWORD
// WINAPI
// REnumServicesStatusW(
//     SC_RPC_HANDLE hSCManager,
//     DWORD dwServiceType,
//     DWORD dwServiceState,
//     LPBYTE lpBuffer,
//     DWORD dwBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned,
//     LPBOUNDED_DWORD_256K lpResumeHandle)
// {
//     /* Enumerate all the services, not regarding of their group */
//     return REnumServiceGroupW(hSCManager,
//                               dwServiceType,
//                               dwServiceState,
//                               lpBuffer,
//                               dwBufSize,
//                               pcbBytesNeeded,
//                               lpServicesReturned,
//                               lpResumeHandle,
//                               NULL);
// }
//
//
// /* Function 15 */
// DWORD
// WINAPI
// ROpenSCManagerW(
//     LPWSTR lpMachineName,
//     LPWSTR lpDatabaseName,
//     DWORD dwDesiredAccess,
//     LPSC_RPC_HANDLE lpScHandle)
// {
//     DWORD dwError;
//     SC_HANDLE hHandle;
//
//     DPRINT("ROpenSCManagerW() called\n");
//     DPRINT("lpMachineName = %p\n", lpMachineName);
//     DPRINT("lpMachineName: %S\n", lpMachineName);
//     DPRINT("lpDataBaseName = %p\n", lpDatabaseName);
//     DPRINT("lpDataBaseName: %S\n", lpDatabaseName);
//     DPRINT("dwDesiredAccess = %x\n", dwDesiredAccess);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if (!lpScHandle)
//         return ERROR_INVALID_PARAMETER;
//
//     dwError = ScmCreateManagerHandle(lpDatabaseName,
//                                      &hHandle);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("ScmCreateManagerHandle() failed (Error %lu)\n", dwError);
//         return dwError;
//     }
//
//     /* Check the desired access */
//     dwError = ScmCheckAccess(hHandle,
//                              dwDesiredAccess | SC_MANAGER_CONNECT);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("ScmCheckAccess() failed (Error %lu)\n", dwError);
//         HeapFree(GetProcessHeap(), 0, hHandle);
//         return dwError;
//     }
//
//     *lpScHandle = (SC_RPC_HANDLE)hHandle;
//     DPRINT("*hScm = %p\n", *lpScHandle);
//
//     DPRINT("ROpenSCManagerW() done\n");
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 16 */
// DWORD
// WINAPI
// ROpenServiceW(
//     SC_RPC_HANDLE hSCManager,
//     LPWSTR lpServiceName,
//     DWORD dwDesiredAccess,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     PSERVICE lpService;
//     PMANAGER_HANDLE hManager;
//     SC_HANDLE hHandle;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("ROpenServiceW() called\n");
//     DPRINT("hSCManager = %p\n", hSCManager);
//     DPRINT("lpServiceName = %p\n", lpServiceName);
//     DPRINT("lpServiceName: %S\n", lpServiceName);
//     DPRINT("dwDesiredAccess = %x\n", dwDesiredAccess);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!lpServiceHandle)
//         return ERROR_INVALID_PARAMETER;
//
//     if (!lpServiceName)
//         return ERROR_INVALID_ADDRESS;
//
//     /* Lock the service database exclusive */
//     ScmLockDatabaseExclusive();
//
//     /* Get service database entry */
//     lpService = ScmGetServiceEntryByName(lpServiceName);
//     if (lpService == NULL)
//     {
//         DPRINT("Could not find service\n");
//         dwError = ERROR_SERVICE_DOES_NOT_EXIST;
//         goto Done;
//     }
//
//     /* Create a service handle */
//     dwError = ScmCreateServiceHandle(lpService,
//                                      &hHandle);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("ScmCreateServiceHandle() failed (Error %lu)\n", dwError);
//         goto Done;
//     }
//
//     /* Check the desired access */
//     dwError = ScmCheckAccess(hHandle,
//                              dwDesiredAccess);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("ScmCheckAccess() failed (Error %lu)\n", dwError);
//         HeapFree(GetProcessHeap(), 0, hHandle);
//         goto Done;
//     }
//
//     ScmReferenceService(lpService);
//     DPRINT("OpenService %S - lpService->RefCount %u\n", lpService->lpServiceName, lpService->RefCount);
//
//     *lpServiceHandle = (SC_RPC_HANDLE)hHandle;
//     DPRINT("*hService = %p\n", *lpServiceHandle);
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("ROpenServiceW() done\n");
//
//     return dwError;
// }
//
//
// /* Function 17 */
// DWORD
// WINAPI
// RQueryServiceConfigW(
//     SC_RPC_HANDLE hService,
//     LPBYTE lpBuf, //LPQUERY_SERVICE_CONFIGW lpServiceConfig,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_8K pcbBytesNeeded)
// {
//     LPQUERY_SERVICE_CONFIGW lpServiceConfig = (LPQUERY_SERVICE_CONFIGW)lpBuf;
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     LPWSTR lpImagePath = NULL;
//     LPWSTR lpServiceStartName = NULL;
//     LPWSTR lpDependencies = NULL;
//     DWORD dwDependenciesLength = 0;
//     DWORD dwRequiredSize;
//     LPWSTR lpStr;
//
//     DPRINT("RQueryServiceConfigW() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_CONFIG))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     dwError = ScmOpenServiceKey(lpService->lpServiceName,
//                                 KEY_READ,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* Read the image path */
//     dwError = ScmReadString(hServiceKey,
//                             L"ImagePath",
//                             &lpImagePath);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* Read the service start name */
//     ScmReadString(hServiceKey,
//                   L"ObjectName",
//                   &lpServiceStartName);
//
//     /* Read the dependencies */
//     ScmReadDependencies(hServiceKey,
//                         &lpDependencies,
//                         &dwDependenciesLength);
//
//     dwRequiredSize = sizeof(QUERY_SERVICE_CONFIGW);
//
//     if (lpImagePath != NULL)
//         dwRequiredSize += (DWORD)((wcslen(lpImagePath) + 1) * sizeof(WCHAR));
//     else
//         dwRequiredSize += 2 * sizeof(WCHAR);
//
//     if ((lpService->lpGroup != NULL) && (lpService->lpGroup->lpGroupName != NULL))
//         dwRequiredSize += (DWORD)((wcslen(lpService->lpGroup->lpGroupName) + 1) * sizeof(WCHAR));
//     else
//         dwRequiredSize += 2 * sizeof(WCHAR);
//
//     if (lpDependencies != NULL)
//         dwRequiredSize += dwDependenciesLength * sizeof(WCHAR);
//     else
//         dwRequiredSize += 2 * sizeof(WCHAR);
//
//     if (lpServiceStartName != NULL)
//         dwRequiredSize += (DWORD)((wcslen(lpServiceStartName) + 1) * sizeof(WCHAR));
//     else
//         dwRequiredSize += 2 * sizeof(WCHAR);
//
//     if (lpService->lpDisplayName != NULL)
//         dwRequiredSize += (DWORD)((wcslen(lpService->lpDisplayName) + 1) * sizeof(WCHAR));
//     else
//         dwRequiredSize += 2 * sizeof(WCHAR);
//
//     if (lpServiceConfig == NULL || cbBufSize < dwRequiredSize)
//     {
//         dwError = ERROR_INSUFFICIENT_BUFFER;
//     }
//     else
//     {
//         lpServiceConfig->dwServiceType = lpService->Status.dwServiceType;
//         lpServiceConfig->dwStartType = lpService->dwStartType;
//         lpServiceConfig->dwErrorControl = lpService->dwErrorControl;
//         lpServiceConfig->dwTagId = lpService->dwTag;
//
//         lpStr = (LPWSTR)(lpServiceConfig + 1);
//
//         /* Append the image path */
//         if (lpImagePath != NULL)
//         {
//             wcscpy(lpStr, lpImagePath);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpBinaryPathName = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (wcslen(lpStr) + 1);
//
//         /* Append the group name */
//         if ((lpService->lpGroup != NULL) && (lpService->lpGroup->lpGroupName != NULL))
//         {
//             wcscpy(lpStr, lpService->lpGroup->lpGroupName);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpLoadOrderGroup = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (wcslen(lpStr) + 1);
//
//         /* Append Dependencies */
//         if (lpDependencies != NULL)
//         {
//             memcpy(lpStr,
//                    lpDependencies,
//                    dwDependenciesLength * sizeof(WCHAR));
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpDependencies = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         if (lpDependencies != NULL)
//             lpStr += dwDependenciesLength;
//         else
//             lpStr += (wcslen(lpStr) + 1);
//
//         /* Append the service start name */
//         if (lpServiceStartName != NULL)
//         {
//             wcscpy(lpStr, lpServiceStartName);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpServiceStartName = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (wcslen(lpStr) + 1);
//
//         /* Append the display name */
//         if (lpService->lpDisplayName != NULL)
//         {
//             wcscpy(lpStr, lpService->lpDisplayName);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpDisplayName = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//     }
//
//     if (pcbBytesNeeded != NULL)
//         *pcbBytesNeeded = dwRequiredSize;
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (lpImagePath != NULL)
//         HeapFree(GetProcessHeap(), 0, lpImagePath);
//
//     if (lpServiceStartName != NULL)
//         HeapFree(GetProcessHeap(), 0, lpServiceStartName);
//
//     if (lpDependencies != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDependencies);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     DPRINT("RQueryServiceConfigW() done\n");
//
//     return dwError;
// }
//
//
// /* Function 18 */
// DWORD
// WINAPI
// RQueryServiceLockStatusW(
//     SC_RPC_HANDLE hSCManager,
//     LPBYTE lpBuf, // LPQUERY_SERVICE_LOCK_STATUSW lpLockStatus,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_4K pcbBytesNeeded)
// {
//     LPQUERY_SERVICE_LOCK_STATUSW lpLockStatus = (LPQUERY_SERVICE_LOCK_STATUSW)lpBuf;
//     PMANAGER_HANDLE hMgr;
//     DWORD dwRequiredSize;
//
//     if (!lpLockStatus || !pcbBytesNeeded)
//         return ERROR_INVALID_PARAMETER;
//
//     hMgr = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hMgr == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hMgr->Handle.DesiredAccess,
//                                   SC_MANAGER_QUERY_LOCK_STATUS))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hMgr->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* FIXME: we need to compute instead the real length of the owner name */
//     dwRequiredSize = sizeof(QUERY_SERVICE_LOCK_STATUSW) + sizeof(WCHAR);
//     *pcbBytesNeeded = dwRequiredSize;
//
//     if (cbBufSize < dwRequiredSize)
//         return ERROR_INSUFFICIENT_BUFFER;
//
//     ScmQueryServiceLockStatusW(lpLockStatus);
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 19 */
// DWORD
// WINAPI
// RStartServiceW(
//     SC_RPC_HANDLE hService,
//     DWORD argc,
//     LPSTRING_PTRSW argv)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//
// #ifndef NDEBUG
//     DWORD i;
//
//     DPRINT("RStartServiceW(%p %lu %p) called\n", hService, argc, argv);
//     DPRINT("  argc: %lu\n", argc);
//     if (argv != NULL)
//     {
//         for (i = 0; i < argc; i++)
//         {
//             DPRINT("  argv[%lu]: %S\n", i, argv[i].StringPtr);
//         }
//     }
// #endif
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_START))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (lpService->dwStartType == SERVICE_DISABLED)
//         return ERROR_SERVICE_DISABLED;
//
//     if (lpService->bDeleted)
//         return ERROR_SERVICE_MARKED_FOR_DELETE;
//
//     /* Start the service */
//     dwError = ScmStartService(lpService, argc, (PCWSTR*)argv);
//
//     return dwError;
// }
//
//
// /* Function 20 */
// DWORD
// WINAPI
// RGetServiceDisplayNameW(
//     SC_RPC_HANDLE hSCManager,
//     LPCWSTR lpServiceName,
//     LPWSTR lpDisplayName,
//     DWORD *lpcchBuffer)
// {
//     // PMANAGER_HANDLE hManager;
//     PSERVICE lpService;
//     LPCWSTR lpSvcDisplayName;
//     DWORD dwLength;
//     DWORD dwError;
//
//     DPRINT("RGetServiceDisplayNameW() called\n");
//     DPRINT("hSCManager = %p\n", hSCManager);
//     DPRINT("lpServiceName: %S\n", lpServiceName);
//     DPRINT("lpDisplayName: %p\n", lpDisplayName);
//     DPRINT("*lpcchBuffer: %lu\n", *lpcchBuffer);
//
//
//     /* Get service database entry */
//     lpService = ScmGetServiceEntryByName(lpServiceName);
//     if (lpService == NULL)
//     {
//         DPRINT("Could not find service\n");
//         return ERROR_SERVICE_DOES_NOT_EXIST;
//     }
//
//     if (lpService->lpDisplayName)
//         lpSvcDisplayName = lpService->lpDisplayName;
//     else
//         lpSvcDisplayName = lpService->lpServiceName;
//
//     dwLength = (DWORD)wcslen(lpSvcDisplayName);
//
//     if (*lpcchBuffer > dwLength)
//     {
//         if (lpDisplayName != NULL)
//             wcscpy(lpDisplayName, lpSvcDisplayName);
//
//         dwError = ERROR_SUCCESS;
//     }
//     else
//     {
//         dwError = ERROR_INSUFFICIENT_BUFFER;
//     }
//
//     *lpcchBuffer = dwLength;
//
//     return dwError;
// }
//
//
// /* Function 21 */
// DWORD
// WINAPI
// RGetServiceKeyNameW(
//     SC_RPC_HANDLE hSCManager,
//     LPCWSTR lpDisplayName,
//     LPWSTR lpServiceName,
//     DWORD *lpcchBuffer)
// {
//     // PMANAGER_HANDLE hManager;
//     PSERVICE lpService;
//     DWORD dwLength;
//     DWORD dwError;
//
//     DPRINT("RGetServiceKeyNameW() called\n");
//     DPRINT("hSCManager = %p\n", hSCManager);
//     DPRINT("lpDisplayName: %S\n", lpDisplayName);
//     DPRINT("lpServiceName: %p\n", lpServiceName);
//     DPRINT("*lpcchBuffer: %lu\n", *lpcchBuffer);
//
//
//     /* Get service database entry */
//     lpService = ScmGetServiceEntryByDisplayName(lpDisplayName);
//     if (lpService == NULL)
//     {
//         DPRINT("Could not find service\n");
//         return ERROR_SERVICE_DOES_NOT_EXIST;
//     }
//
//     dwLength = (DWORD)wcslen(lpService->lpServiceName);
//
//     if (*lpcchBuffer > dwLength)
//     {
//         if (lpServiceName != NULL)
//             wcscpy(lpServiceName, lpService->lpServiceName);
//
//         dwError = ERROR_SUCCESS;
//     }
//     else
//     {
//         dwError = ERROR_INSUFFICIENT_BUFFER;
//     }
//
//     *lpcchBuffer = dwLength;
//
//     return dwError;
// }
//
//
// /* Function 22 */
// DWORD
// WINAPI
// RI_ScSetServiceBitsA(
//     RPC_SERVICE_STATUS_HANDLE hServiceStatus,
//     DWORD dwServiceBits,
//     int bSetBitsOn,
//     int bUpdateImmediately,
//     char *lpString)
// {
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if (lpString != NULL)
//         return ERROR_INVALID_PARAMETER;
//
//     return RI_ScSetServiceBitsW(hServiceStatus,
//                                 dwServiceBits,
//                                 bSetBitsOn,
//                                 bUpdateImmediately,
//                                 NULL);
// }
//
//
// /* Function 23 */
// DWORD
// WINAPI
// RChangeServiceConfigA(
//     SC_RPC_HANDLE hService,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPSTR lpBinaryPathName,
//     LPSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPSTR lpDisplayName)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     LPWSTR lpBinaryPathNameW = NULL;
//     LPWSTR lpLoadOrderGroupW = NULL;
//     LPWSTR lpDependenciesW = NULL;
//     LPWSTR lpServiceStartNameW = NULL;
//     LPWSTR lpDisplayNameW = NULL;
//     DWORD dwDependenciesLength = 0;
//     SIZE_T cchLength;
//     int len;
//     LPCSTR lpStr;
//
//     if (lpBinaryPathName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, NULL, 0);
//         lpBinaryPathNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpBinaryPathNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, lpBinaryPathNameW, len);
//     }
//
//     if (lpLoadOrderGroup)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, NULL, 0);
//         lpLoadOrderGroupW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpLoadOrderGroupW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, lpLoadOrderGroupW, len);
//     }
//
//     if (lpDependencies)
//     {
//         lpStr = (LPCSTR)lpDependencies;
//         while (*lpStr)
//         {
//             cchLength = strlen(lpStr) + 1;
//             dwDependenciesLength += (DWORD)cchLength;
//             lpStr = lpStr + cchLength;
//         }
//         dwDependenciesLength++;
//
//         lpDependenciesW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwDependenciesLength * sizeof(WCHAR));
//         if (!lpDependenciesW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, (LPCSTR)lpDependencies, dwDependenciesLength, lpDependenciesW, dwDependenciesLength);
//     }
//
//     if (lpServiceStartName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, NULL, 0);
//         lpServiceStartNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpServiceStartNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, lpServiceStartNameW, len);
//     }
//
//     if (lpDisplayName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, NULL, 0);
//         lpDisplayNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpDisplayNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, lpDisplayNameW, len);
//     }
//
//     dwError = RChangeServiceConfigW(hService,
//                                     dwServiceType,
//                                     dwStartType,
//                                     dwErrorControl,
//                                     lpBinaryPathNameW,
//                                     lpLoadOrderGroupW,
//                                     lpdwTagId,
//                                     (LPBYTE)lpDependenciesW,
//                                     dwDependenciesLength,
//                                     lpServiceStartNameW,
//                                     lpPassword,
//                                     dwPwSize,
//                                     lpDisplayNameW);
//
// cleanup:
//     if (lpBinaryPathNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpBinaryPathNameW);
//
//     if (lpLoadOrderGroupW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpLoadOrderGroupW);
//
//     if (lpDependenciesW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDependenciesW);
//
//     if (lpServiceStartNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpServiceStartNameW);
//
//     if (lpDisplayNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDisplayNameW);
//
//     return dwError;
// }
//
//
// /* Function 24 */
// DWORD
// WINAPI
// RCreateServiceA(
//     SC_RPC_HANDLE hSCManager,
//     LPSTR lpServiceName,
//     LPSTR lpDisplayName,
//     DWORD dwDesiredAccess,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPSTR lpBinaryPathName,
//     LPSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     LPWSTR lpServiceNameW = NULL;
//     LPWSTR lpDisplayNameW = NULL;
//     LPWSTR lpBinaryPathNameW = NULL;
//     LPWSTR lpLoadOrderGroupW = NULL;
//     LPWSTR lpDependenciesW = NULL;
//     LPWSTR lpServiceStartNameW = NULL;
//     DWORD dwDependenciesLength = 0;
//     SIZE_T cchLength;
//     int len;
//     LPCSTR lpStr;
//
//     if (lpServiceName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpServiceName, -1, NULL, 0);
//         lpServiceNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpServiceNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpServiceName, -1, lpServiceNameW, len);
//     }
//
//     if (lpDisplayName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, NULL, 0);
//         lpDisplayNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpDisplayNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpDisplayName, -1, lpDisplayNameW, len);
//     }
//
//     if (lpBinaryPathName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, NULL, 0);
//         lpBinaryPathNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpBinaryPathNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpBinaryPathName, -1, lpBinaryPathNameW, len);
//     }
//
//     if (lpLoadOrderGroup)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, NULL, 0);
//         lpLoadOrderGroupW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpLoadOrderGroupW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpLoadOrderGroup, -1, lpLoadOrderGroupW, len);
//     }
//
//     if (lpDependencies)
//     {
//         lpStr = (LPCSTR)lpDependencies;
//         while (*lpStr)
//         {
//             cchLength = strlen(lpStr) + 1;
//             dwDependenciesLength += (DWORD)cchLength;
//             lpStr = lpStr + cchLength;
//         }
//         dwDependenciesLength++;
//
//         lpDependenciesW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwDependenciesLength * sizeof(WCHAR));
//         if (!lpDependenciesW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, (LPCSTR)lpDependencies, dwDependenciesLength, lpDependenciesW, dwDependenciesLength);
//     }
//
//     if (lpServiceStartName)
//     {
//         len = MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, NULL, 0);
//         lpServiceStartNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len * sizeof(WCHAR));
//         if (!lpServiceStartNameW)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         MultiByteToWideChar(CP_ACP, 0, lpServiceStartName, -1, lpServiceStartNameW, len);
//     }
//
//     dwError = RCreateServiceW(hSCManager,
//                               lpServiceNameW,
//                               lpDisplayNameW,
//                               dwDesiredAccess,
//                               dwServiceType,
//                               dwStartType,
//                               dwErrorControl,
//                               lpBinaryPathNameW,
//                               lpLoadOrderGroupW,
//                               lpdwTagId,
//                               (LPBYTE)lpDependenciesW,
//                               dwDependenciesLength,
//                               lpServiceStartNameW,
//                               lpPassword,
//                               dwPwSize,
//                               lpServiceHandle);
//
// cleanup:
//     if (lpServiceNameW !=NULL)
//         HeapFree(GetProcessHeap(), 0, lpServiceNameW);
//
//     if (lpDisplayNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDisplayNameW);
//
//     if (lpBinaryPathNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpBinaryPathNameW);
//
//     if (lpLoadOrderGroupW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpLoadOrderGroupW);
//
//     if (lpDependenciesW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDependenciesW);
//
//     if (lpServiceStartNameW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpServiceStartNameW);
//
//     return dwError;
// }
//
//
// /* Function 25 */
// DWORD
// WINAPI
// REnumDependentServicesA(
//     SC_RPC_HANDLE hService,
//     DWORD dwServiceState,
//     LPBYTE lpServices,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     DWORD dwServicesReturned = 0;
//     DWORD dwServiceCount;
//     HKEY hServicesKey = NULL;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     PSERVICE *lpServicesArray = NULL;
//     LPENUM_SERVICE_STATUSA lpServicesPtr = NULL;
//     LPSTR lpStr;
//
//     *pcbBytesNeeded = 0;
//     *lpServicesReturned = 0;
//
//     DPRINT("REnumDependentServicesA() called\n");
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     lpService = hSvc->ServiceEntry;
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n",
//                hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* Open the Services Reg key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_READ,
//                             &hServicesKey) as DWORD;
//
//     if (dwError != ERROR_SUCCESS)
//         return dwError;
//
//     /* NOTE: Windows calculates the pcbBytesNeeded based on WCHAR strings for
//              both EnumDependentServicesA and EnumDependentServicesW. So returned pcbBytesNeeded
//              are the same for both. Verified in WINXP. */
//
//     /* First determine the bytes needed and get the number of dependent services*/
//     dwError = Int_EnumDependentServicesW(hServicesKey,
//                                          lpService,
//                                          dwServiceState,
//                                          NULL,
//                                          pcbBytesNeeded,
//                                          &dwServicesReturned);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* If buffer size is less than the bytes needed or pointer is null*/
//     if ((!lpServices) || (cbBufSize < *pcbBytesNeeded))
//     {
//         dwError = ERROR_MORE_DATA;
//         goto Done;
//     }
//
//     /* Allocate memory for array of service pointers */
//     lpServicesArray = HeapAlloc(GetProcessHeap(),
//                                 HEAP_ZERO_MEMORY,
//                                 (dwServicesReturned + 1) * sizeof(PSERVICE));
//     if (!lpServicesArray)
//     {
//         DPRINT("Could not allocate buffer\n");
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto Done;
//     }
//
//     dwServicesReturned = 0;
//     *pcbBytesNeeded = 0;
//
//     dwError = Int_EnumDependentServicesW(hServicesKey,
//                                          lpService,
//                                          dwServiceState,
//                                          lpServicesArray,
//                                          pcbBytesNeeded,
//                                          &dwServicesReturned);
//     if (dwError != ERROR_SUCCESS)
//     {
//         goto Done;
//     }
//
//     lpServicesPtr = (LPENUM_SERVICE_STATUSA)lpServices;
//     lpStr = (LPSTR)(lpServices + (dwServicesReturned * sizeof(ENUM_SERVICE_STATUSA)));
//
//     /* Copy EnumDepenedentService to Buffer */
//     for (dwServiceCount = 0; dwServiceCount < dwServicesReturned; dwServiceCount++)
//     {
//         lpService = lpServicesArray[dwServiceCount];
//
//         /* Copy the status info */
//         memcpy(&lpServicesPtr->ServiceStatus,
//                &lpService->Status,
//                sizeof(SERVICE_STATUS));
//
//         /* Copy display name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpService->lpDisplayName,
//                             -1,
//                             lpStr,
//                             (int)wcslen(lpService->lpDisplayName),
//                             0,
//                             0);
//         lpServicesPtr->lpDisplayName = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServices);
//         lpStr += strlen(lpStr) + 1;
//
//         /* Copy service name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpService->lpServiceName,
//                             -1,
//                             lpStr,
//                             (int)wcslen(lpService->lpServiceName),
//                             0,
//                             0);
//         lpServicesPtr->lpServiceName = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServices);
//         lpStr += strlen(lpStr) + 1;
//
//         lpServicesPtr++;
//     }
//
//     *lpServicesReturned = dwServicesReturned;
//
// Done:
//     if (lpServicesArray)
//         HeapFree(GetProcessHeap(), 0, lpServicesArray);
//
//     RegCloseKey(hServicesKey);
//
//     DPRINT("REnumDependentServicesA() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 26 */
// DWORD
// WINAPI
// REnumServicesStatusA(
//     SC_RPC_HANDLE hSCManager,
//     DWORD dwServiceType,
//     DWORD dwServiceState,
//     LPBYTE lpBuffer,
//     DWORD dwBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned,
//     LPBOUNDED_DWORD_256K lpResumeHandle)
// {
//     LPENUM_SERVICE_STATUSW lpStatusPtrW = NULL;
//     LPENUM_SERVICE_STATUSW lpStatusPtrIncrW;
//     LPENUM_SERVICE_STATUSA lpStatusPtrA = NULL;
//     LPWSTR lpStringPtrW;
//     LPSTR lpStringPtrA;
//     DWORD dwError;
//     DWORD dwServiceCount;
//
//     DPRINT("REnumServicesStatusA() called\n");
//
//     if (pcbBytesNeeded == NULL || lpServicesReturned == NULL)
//     {
//         return ERROR_INVALID_ADDRESS;
//     }
//
//     if ((dwBufSize > 0) && (lpBuffer))
//     {
//         lpStatusPtrW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwBufSize);
//         if (!lpStatusPtrW)
//         {
//             DPRINT("Failed to allocate buffer\n");
//             return ERROR_NOT_ENOUGH_MEMORY;
//         }
//     }
//
//     dwError = REnumServicesStatusW(hSCManager,
//                                    dwServiceType,
//                                    dwServiceState,
//                                    (LPBYTE)lpStatusPtrW,
//                                    dwBufSize,
//                                    pcbBytesNeeded,
//                                    lpServicesReturned,
//                                    lpResumeHandle);
//
//     /* if no services were returned then we are Done */
//     if (*lpServicesReturned == 0)
//         goto Done;
//
//     lpStatusPtrIncrW = lpStatusPtrW;
//     lpStatusPtrA = (LPENUM_SERVICE_STATUSA)lpBuffer;
//     lpStringPtrA = (LPSTR)((ULONG_PTR)lpBuffer +
//                   *lpServicesReturned * sizeof(ENUM_SERVICE_STATUSA));
//     lpStringPtrW = (LPWSTR)((ULONG_PTR)lpStatusPtrW +
//                   *lpServicesReturned * sizeof(ENUM_SERVICE_STATUSW));
//
//     for (dwServiceCount = 0; dwServiceCount < *lpServicesReturned; dwServiceCount++)
//     {
//         /* Copy the service name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpStringPtrW,
//                             -1,
//                             lpStringPtrA,
//                             (int)wcslen(lpStringPtrW),
//                             0,
//                             0);
//
//         lpStatusPtrA->lpServiceName = (LPSTR)((ULONG_PTR)lpStringPtrA - (ULONG_PTR)lpBuffer);
//         lpStringPtrA += wcslen(lpStringPtrW) + 1;
//         lpStringPtrW += wcslen(lpStringPtrW) + 1;
//
//         /* Copy the display name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpStringPtrW,
//                             -1,
//                             lpStringPtrA,
//                             (int)wcslen(lpStringPtrW),
//                             0,
//                             0);
//
//         lpStatusPtrA->lpDisplayName = (LPSTR)((ULONG_PTR)lpStringPtrA - (ULONG_PTR)lpBuffer);
//         lpStringPtrA += wcslen(lpStringPtrW) + 1;
//         lpStringPtrW += wcslen(lpStringPtrW) + 1;
//
//         /* Copy the status information */
//         memcpy(&lpStatusPtrA->ServiceStatus,
//                &lpStatusPtrIncrW->ServiceStatus,
//                sizeof(SERVICE_STATUS));
//
//         lpStatusPtrIncrW++;
//         lpStatusPtrA++;
//     }
//
// Done:
//     if (lpStatusPtrW)
//         HeapFree(GetProcessHeap(), 0, lpStatusPtrW);
//
//     DPRINT("REnumServicesStatusA() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 27 */
// DWORD
// WINAPI
// ROpenSCManagerA(
//     LPSTR lpMachineName,
//     LPSTR lpDatabaseName,
//     DWORD dwDesiredAccess,
//     LPSC_RPC_HANDLE lpScHandle)
// {
//     UNICODE_STRING MachineName;
//     UNICODE_STRING DatabaseName;
//     DWORD dwError;
//
//     DPRINT("ROpenSCManagerA() called\n");
//
//     if (lpMachineName)
//         RtlCreateUnicodeStringFromAsciiz(&MachineName,
//                                          lpMachineName);
//
//     if (lpDatabaseName)
//         RtlCreateUnicodeStringFromAsciiz(&DatabaseName,
//                                          lpDatabaseName);
//
//     dwError = ROpenSCManagerW(lpMachineName ? MachineName.Buffer : NULL,
//                               lpDatabaseName ? DatabaseName.Buffer : NULL,
//                               dwDesiredAccess,
//                               lpScHandle);
//
//     if (lpMachineName)
//         RtlFreeUnicodeString(&MachineName);
//
//     if (lpDatabaseName)
//         RtlFreeUnicodeString(&DatabaseName);
//
//     return dwError;
// }
//
//
// /* Function 28 */
// DWORD
// WINAPI
// ROpenServiceA(
//     SC_RPC_HANDLE hSCManager,
//     LPSTR lpServiceName,
//     DWORD dwDesiredAccess,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     UNICODE_STRING ServiceName;
//     DWORD dwError;
//
//     DPRINT("ROpenServiceA() called\n");
//
//     if (lpServiceName)
//         RtlCreateUnicodeStringFromAsciiz(&ServiceName,
//                                          lpServiceName);
//
//     dwError = ROpenServiceW(hSCManager,
//                             lpServiceName ? ServiceName.Buffer : NULL,
//                             dwDesiredAccess,
//                             lpServiceHandle);
//
//     if (lpServiceName)
//         RtlFreeUnicodeString(&ServiceName);
//
//     return dwError;
// }
//
//
// /* Function 29 */
// DWORD
// WINAPI
// RQueryServiceConfigA(
//     SC_RPC_HANDLE hService,
//     LPBYTE lpBuf, //LPQUERY_SERVICE_CONFIGA lpServiceConfig,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_8K pcbBytesNeeded)
// {
//     LPQUERY_SERVICE_CONFIGA lpServiceConfig = (LPQUERY_SERVICE_CONFIGA)lpBuf;
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     LPWSTR lpImagePath = NULL;
//     LPWSTR lpServiceStartName = NULL;
//     LPWSTR lpDependencies = NULL;
//     DWORD dwDependenciesLength = 0;
//     DWORD dwRequiredSize;
//     LPSTR lpStr;
//
//     DPRINT("RQueryServiceConfigA() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_CONFIG))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     dwError = ScmOpenServiceKey(lpService->lpServiceName,
//                                 KEY_READ,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* Read the image path */
//     dwError = ScmReadString(hServiceKey,
//                             L"ImagePath",
//                             &lpImagePath);
//     if (dwError != ERROR_SUCCESS)
//         goto Done;
//
//     /* Read the service start name */
//     ScmReadString(hServiceKey,
//                   L"ObjectName",
//                   &lpServiceStartName);
//
//     /* Read the dependencies */
//     ScmReadDependencies(hServiceKey,
//                         &lpDependencies,
//                         &dwDependenciesLength);
//
//     dwRequiredSize = sizeof(QUERY_SERVICE_CONFIGA);
//
//     if (lpImagePath != NULL)
//         dwRequiredSize += (DWORD)(wcslen(lpImagePath) + 1);
//     else
//         dwRequiredSize += 2 * sizeof(CHAR);
//
//     if ((lpService->lpGroup != NULL) && (lpService->lpGroup->lpGroupName != NULL))
//         dwRequiredSize += (DWORD)(wcslen(lpService->lpGroup->lpGroupName) + 1);
//     else
//         dwRequiredSize += 2 * sizeof(CHAR);
//
//     /* Add Dependencies length */
//     if (lpDependencies != NULL)
//         dwRequiredSize += dwDependenciesLength;
//     else
//         dwRequiredSize += 2 * sizeof(CHAR);
//
//     if (lpServiceStartName != NULL)
//         dwRequiredSize += (DWORD)(wcslen(lpServiceStartName) + 1);
//     else
//         dwRequiredSize += 2 * sizeof(CHAR);
//
//     if (lpService->lpDisplayName != NULL)
//         dwRequiredSize += (DWORD)(wcslen(lpService->lpDisplayName) + 1);
//     else
//         dwRequiredSize += 2 * sizeof(CHAR);
//
//     if (lpServiceConfig == NULL || cbBufSize < dwRequiredSize)
//     {
//         dwError = ERROR_INSUFFICIENT_BUFFER;
//     }
//     else
//     {
//         lpServiceConfig->dwServiceType = lpService->Status.dwServiceType;
//         lpServiceConfig->dwStartType = lpService->dwStartType;
//         lpServiceConfig->dwErrorControl = lpService->dwErrorControl;
//         lpServiceConfig->dwTagId = lpService->dwTag;
//
//         lpStr = (LPSTR)(lpServiceConfig + 1);
//
//         /* NOTE: Strings that are NULL for QUERY_SERVICE_CONFIG are pointers to empty strings.
//            Verified in WINXP */
//
//         if (lpImagePath)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpImagePath,
//                                 -1,
//                                 lpStr,
//                                 (int)(wcslen(lpImagePath) + 1),
//                                 0,
//                                 0);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpBinaryPathName = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (strlen((LPSTR)lpStr) + 1);
//
//         if (lpService->lpGroup && lpService->lpGroup->lpGroupName)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpService->lpGroup->lpGroupName,
//                                 -1,
//                                 lpStr,
//                                 (int)(wcslen(lpService->lpGroup->lpGroupName) + 1),
//                                 0,
//                                 0);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpLoadOrderGroup = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (strlen(lpStr) + 1);
//
//         /* Append Dependencies */
//         if (lpDependencies)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpDependencies,
//                                 dwDependenciesLength,
//                                 lpStr,
//                                 dwDependenciesLength,
//                                 0,
//                                 0);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpDependencies = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         if (lpDependencies)
//             lpStr += dwDependenciesLength;
//         else
//             lpStr += (strlen(lpStr) + 1);
//
//         if (lpServiceStartName)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpServiceStartName,
//                                 -1,
//                                 lpStr,
//                                 (int)(wcslen(lpServiceStartName) + 1),
//                                 0,
//                                 0);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpServiceStartName = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//         lpStr += (strlen(lpStr) + 1);
//
//         if (lpService->lpDisplayName)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpService->lpDisplayName,
//                                 -1,
//                                 lpStr,
//                                 (int)(wcslen(lpService->lpDisplayName) + 1),
//                                 0,
//                                 0);
//         }
//         else
//         {
//             *lpStr = 0;
//         }
//
//         lpServiceConfig->lpDisplayName = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceConfig);
//     }
//
//     if (pcbBytesNeeded != NULL)
//         *pcbBytesNeeded = dwRequiredSize;
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (lpImagePath != NULL)
//         HeapFree(GetProcessHeap(), 0, lpImagePath);
//
//     if (lpServiceStartName != NULL)
//         HeapFree(GetProcessHeap(), 0, lpServiceStartName);
//
//     if (lpDependencies != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDependencies);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     DPRINT("RQueryServiceConfigA() done\n");
//
//     return dwError;
// }
//
//
// /* Function 30 */
// DWORD
// WINAPI
// RQueryServiceLockStatusA(
//     SC_RPC_HANDLE hSCManager,
//     LPBYTE lpBuf, // LPQUERY_SERVICE_LOCK_STATUSA lpLockStatus,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_4K pcbBytesNeeded)
// {
//     LPQUERY_SERVICE_LOCK_STATUSA lpLockStatus = (LPQUERY_SERVICE_LOCK_STATUSA)lpBuf;
//     PMANAGER_HANDLE hMgr;
//     DWORD dwRequiredSize;
//
//     if (!lpLockStatus || !pcbBytesNeeded)
//         return ERROR_INVALID_PARAMETER;
//
//     hMgr = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hMgr == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hMgr->Handle.DesiredAccess,
//                                   SC_MANAGER_QUERY_LOCK_STATUS))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hMgr->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* FIXME: we need to compute instead the real length of the owner name */
//     dwRequiredSize = sizeof(QUERY_SERVICE_LOCK_STATUSA) + sizeof(CHAR);
//     *pcbBytesNeeded = dwRequiredSize;
//
//     if (cbBufSize < dwRequiredSize)
//         return ERROR_INSUFFICIENT_BUFFER;
//
//     ScmQueryServiceLockStatusA(lpLockStatus);
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 31 */
// DWORD
// WINAPI
// RStartServiceA(
//     SC_RPC_HANDLE hService,
//     DWORD argc,
//     LPSTRING_PTRSA argv)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     PWSTR* pVector = NULL;
//     DWORD i;
//     DWORD dwLength;
//
//     DPRINT("RStartServiceA() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_START))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (lpService->dwStartType == SERVICE_DISABLED)
//         return ERROR_SERVICE_DISABLED;
//
//     if (lpService->bDeleted)
//         return ERROR_SERVICE_MARKED_FOR_DELETE;
//
//     /* Build a Unicode argument vector */
//     if (argc > 0)
//     {
//         pVector = HeapAlloc(GetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             argc * sizeof(PWSTR));
//         if (!pVector)
//             return ERROR_NOT_ENOUGH_MEMORY;
//
//         for (i = 0; i < argc; i++)
//         {
//             dwLength = MultiByteToWideChar(CP_ACP,
//                                            0,
//                                            argv[i].StringPtr,
//                                            -1,
//                                            NULL,
//                                            0);
//
//             pVector[i] = HeapAlloc(GetProcessHeap(),
//                                    HEAP_ZERO_MEMORY,
//                                    dwLength * sizeof(WCHAR));
//             if (!pVector[i])
//             {
//                 dwError = ERROR_NOT_ENOUGH_MEMORY;
//                 goto done;
//             }
//
//             MultiByteToWideChar(CP_ACP,
//                                 0,
//                                 argv[i].StringPtr,
//                                 -1,
//                                 pVector[i],
//                                 dwLength);
//         }
//     }
//
//     /* Start the service */
//     dwError = ScmStartService(lpService, argc, (PCWSTR*)pVector);
//
// done:
//     /* Free the Unicode argument vector */
//     if (pVector)
//     {
//         for (i = 0; i < argc; i++)
//         {
//             if (pVector[i])
//                 HeapFree(GetProcessHeap(), 0, pVector[i]);
//         }
//         HeapFree(GetProcessHeap(), 0, pVector);
//     }
//
//     return dwError;
// }
//
//
// /* Function 32 */
// DWORD
// WINAPI
// RGetServiceDisplayNameA(
//     SC_RPC_HANDLE hSCManager,
//     LPCSTR lpServiceName,
//     LPSTR lpDisplayName,
//     LPBOUNDED_DWORD_4K lpcchBuffer)
// {
//     // PMANAGER_HANDLE hManager;
//     PSERVICE lpService = NULL;
//     LPCWSTR lpSvcDisplayName;
//     LPWSTR lpServiceNameW;
//     DWORD dwLength;
//
//     DPRINT("RGetServiceDisplayNameA() called\n");
//     DPRINT("hSCManager = %p\n", hSCManager);
//     DPRINT("lpServiceName: %s\n", lpServiceName);
//     DPRINT("lpDisplayName: %p\n", lpDisplayName);
//     DPRINT("*lpcchBuffer: %lu\n", *lpcchBuffer);
//
//
//     /* Get service database entry */
//     if (lpServiceName != NULL)
//     {
//         dwLength = (DWORD)(strlen(lpServiceName) + 1);
//         lpServiceNameW = HeapAlloc(GetProcessHeap(),
//                                    HEAP_ZERO_MEMORY,
//                                    dwLength * sizeof(WCHAR));
//         if (!lpServiceNameW)
//             return ERROR_NOT_ENOUGH_MEMORY;
//
//         MultiByteToWideChar(CP_ACP,
//                             0,
//                             lpServiceName,
//                             -1,
//                             lpServiceNameW,
//                             dwLength);
//
//         lpService = ScmGetServiceEntryByName(lpServiceNameW);
//
//         HeapFree(GetProcessHeap(), 0, lpServiceNameW);
//     }
//
//     if (lpService == NULL)
//     {
//         DPRINT("Could not find service\n");
//         return ERROR_SERVICE_DOES_NOT_EXIST;
//     }
//
//     if (lpService->lpDisplayName)
//         lpSvcDisplayName = lpService->lpDisplayName;
//     else
//         lpSvcDisplayName = lpService->lpServiceName;
//
//     /*
//      * NOTE: On Windows the comparison on *lpcchBuffer is made against
//      * the number of (wide) characters of the UNICODE display name, and
//      * not against the number of bytes needed to store the ANSI string.
//      */
//     dwLength = (DWORD)wcslen(lpSvcDisplayName);
//
//     if (*lpcchBuffer > dwLength)
//     {
//         if (lpDisplayName != NULL &&
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpSvcDisplayName,
//                                 -1,
//                                 lpDisplayName,
//                                 (int)*lpcchBuffer,
//                                 NULL,
//                                 NULL) == 0)
//         {
//             /*
//              * But then, if *lpcchBuffer was greater than the number of
//              * (wide) characters of the UNICODE display name, yet smaller
//              * than the number of bytes needed due to the possible presence
//              * of DBCS characters, the *exact* number of bytes is returned
//              * (without the NULL terminator).
//              */
//             dwLength = (DWORD)WideCharToMultiByte(CP_ACP,
//                                                   0,
//                                                   lpSvcDisplayName,
//                                                   (int)dwLength,
//                                                   NULL,
//                                                   0,
//                                                   NULL,
//                                                   NULL);
//             *lpDisplayName = 0;
//             *lpcchBuffer = dwLength;
//             return ERROR_INSUFFICIENT_BUFFER;
//         }
//
//         /*
//          * NOTE: On Windows, RGetServiceDisplayNameA() does not update
//          * *lpcchBuffer on success, contrary to RGetServiceDisplayNameW().
//          */
//         return ERROR_SUCCESS;
//     }
//     else
//     {
//         /*
//          * NOTE: On Windows, if *lpcchBuffer is smaller than the number of
//          * (wide) characters of the UNICODE display name, only an upper
//          * estimation is returned by doubling the string length, to account
//          * for the presence of any possible DBCS characters.
//          */
//         *lpcchBuffer = dwLength * sizeof(WCHAR);
//         return ERROR_INSUFFICIENT_BUFFER;
//     }
// }
//
//
// /* Function 33 */
// DWORD
// WINAPI
// RGetServiceKeyNameA(
//     SC_RPC_HANDLE hSCManager,
//     LPCSTR lpDisplayName,
//     LPSTR lpServiceName,
//     LPBOUNDED_DWORD_4K lpcchBuffer)
// {
//     // PMANAGER_HANDLE hManager;
//     PSERVICE lpService;
//     LPWSTR lpDisplayNameW;
//     DWORD dwLength;
//
//     DPRINT("RGetServiceKeyNameA() called\n");
//     DPRINT("hSCManager = %p\n", hSCManager);
//     DPRINT("lpDisplayName: %s\n", lpDisplayName);
//     DPRINT("lpServiceName: %p\n", lpServiceName);
//     DPRINT("*lpcchBuffer: %lu\n", *lpcchBuffer);
//
//
//     /* Get service database entry */
//
//     dwLength = (DWORD)(strlen(lpDisplayName) + 1);
//     lpDisplayNameW = HeapAlloc(GetProcessHeap(),
//                                HEAP_ZERO_MEMORY,
//                                dwLength * sizeof(WCHAR));
//     if (!lpDisplayNameW)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     MultiByteToWideChar(CP_ACP,
//                         0,
//                         lpDisplayName,
//                         -1,
//                         lpDisplayNameW,
//                         dwLength);
//
//     lpService = ScmGetServiceEntryByDisplayName(lpDisplayNameW);
//
//     HeapFree(GetProcessHeap(), 0, lpDisplayNameW);
//
//     if (lpService == NULL)
//     {
//         DPRINT("Could not find service\n");
//         return ERROR_SERVICE_DOES_NOT_EXIST;
//     }
//
//     /*
//      * NOTE: On Windows the comparison on *lpcchBuffer is made against
//      * the number of (wide) characters of the UNICODE service name, and
//      * not against the number of bytes needed to store the ANSI string.
//      */
//     dwLength = (DWORD)wcslen(lpService->lpServiceName);
//
//     if (*lpcchBuffer > dwLength)
//     {
//         if (lpServiceName != NULL &&
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpService->lpServiceName,
//                                 -1,
//                                 lpServiceName,
//                                 (int)*lpcchBuffer,
//                                 NULL,
//                                 NULL) == 0)
//         {
//             /*
//              * But then, if *lpcchBuffer was greater than the number of
//              * (wide) characters of the UNICODE service name, yet smaller
//              * than the number of bytes needed due to the possible presence
//              * of DBCS characters, the *exact* number of bytes is returned
//              * (without the NULL terminator).
//              */
//             dwLength = (DWORD)WideCharToMultiByte(CP_ACP,
//                                                   0,
//                                                   lpService->lpServiceName,
//                                                   (int)dwLength,
//                                                   NULL,
//                                                   0,
//                                                   NULL,
//                                                   NULL);
//             *lpServiceName = 0;
//             *lpcchBuffer = dwLength;
//             return ERROR_INSUFFICIENT_BUFFER;
//         }
//
//         /*
//          * NOTE: On Windows, RGetServiceKeyNameA() does not update
//          * *lpcchBuffer on success, contrary to RGetServiceKeyNameW().
//          */
//         return ERROR_SUCCESS;
//     }
//     else
//     {
//         /*
//          * NOTE: On Windows, if *lpcchBuffer is smaller than the number of
//          * (wide) characters of the UNICODE service name, only an upper
//          * estimation is returned by doubling the string length, to account
//          * for the presence of any possible DBCS characters.
//          */
//         *lpcchBuffer = dwLength * sizeof(WCHAR);
//         return ERROR_INSUFFICIENT_BUFFER;
//     }
// }
//
//
// /* Function 34 */
// DWORD
// WINAPI
// RI_ScGetCurrentGroupStateW(
//     SC_RPC_HANDLE hSCManager,
//     LPWSTR lpLoadOrderGroup,
//     LPDWORD lpState)
// {
//     PMANAGER_HANDLE hManager;
//     PSERVICE_GROUP pServiceGroup;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("RI_ScGetCurrentGroupStateW() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Check for SC_MANAGER_ENUMERATE_SERVICE access right */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n",
//                 hManager->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     /* Get the group list entry */
//     pServiceGroup = ScmGetServiceGroupByName(lpLoadOrderGroup);
//     if (pServiceGroup == NULL)
//     {
//         dwError = ERROR_SERVICE_DOES_NOT_EXIST;
//         goto done;
//     }
//
//     /* FIXME: Return the group state */
//     *lpState = 0;
//
// done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("RI_ScGetCurrentGroupStateW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 35 */
// DWORD
// WINAPI
// REnumServiceGroupW(
//     SC_RPC_HANDLE hSCManager,
//     DWORD dwServiceType,
//     DWORD dwServiceState,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned,
//     LPBOUNDED_DWORD_256K lpResumeIndex,
//     LPCWSTR pszGroupName)
// {
//     PMANAGER_HANDLE hManager;
//     PSERVICE lpService;
//     DWORD dwError = ERROR_SUCCESS;
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//     DWORD dwState;
//     DWORD dwRequiredSize;
//     DWORD dwServiceCount;
//     DWORD dwSize;
//     DWORD dwLastResumeCount = 0;
//     LPENUM_SERVICE_STATUSW lpStatusPtr;
//     LPWSTR lpStringPtr;
//
//     DPRINT("REnumServiceGroupW() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (pcbBytesNeeded == NULL || lpServicesReturned == NULL)
//     {
//         return ERROR_INVALID_ADDRESS;
//     }
//
//     *pcbBytesNeeded = 0;
//     *lpServicesReturned = 0;
//
//     if ((dwServiceType == 0) ||
//         ((dwServiceType & ~SERVICE_TYPE_ALL) != 0))
//     {
//         DPRINT("Invalid Service Type\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if ((dwServiceState == 0) ||
//         ((dwServiceState & ~SERVICE_STATE_ALL) != 0))
//     {
//         DPRINT("Invalid Service State\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n",
//                 hManager->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     if (lpResumeIndex)
//         dwLastResumeCount = *lpResumeIndex;
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     lpService = ScmGetServiceEntryByResumeCount(dwLastResumeCount);
//     if (lpService == NULL)
//     {
//         dwError = ERROR_SUCCESS;
//         goto Done;
//     }
//
//     dwRequiredSize = 0;
//     dwServiceCount = 0;
//
//     for (ServiceEntry = &lpService->ServiceListEntry;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwSize = sizeof(ENUM_SERVICE_STATUSW) +
//                  (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                  (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR));
//
//         if (dwRequiredSize + dwSize > cbBufSize)
//         {
//             DPRINT("Service name: %S  no fit\n", CurrentService->lpServiceName);
//             break;
//         }
//
//         DPRINT("Service name: %S  fit\n", CurrentService->lpServiceName);
//         dwRequiredSize += dwSize;
//         dwServiceCount++;
//         dwLastResumeCount = CurrentService->dwResumeCount;
//     }
//
//     DPRINT("dwRequiredSize: %lu\n", dwRequiredSize);
//     DPRINT("dwServiceCount: %lu\n", dwServiceCount);
//
//     for (;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwRequiredSize += (sizeof(ENUM_SERVICE_STATUSW) +
//                            (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                            (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR)));
//
//         dwError = ERROR_MORE_DATA;
//     }
//
//     DPRINT("*pcbBytesNeeded: %lu\n", dwRequiredSize);
//
//     if (lpResumeIndex)
//         *lpResumeIndex = dwLastResumeCount;
//
//     *lpServicesReturned = dwServiceCount;
//     *pcbBytesNeeded = dwRequiredSize;
//
//     lpStatusPtr = (LPENUM_SERVICE_STATUSW)lpBuffer;
//     lpStringPtr = (LPWSTR)((ULONG_PTR)lpBuffer +
//                            dwServiceCount * sizeof(ENUM_SERVICE_STATUSW));
//
//     dwRequiredSize = 0;
//     for (ServiceEntry = &lpService->ServiceListEntry;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwSize = sizeof(ENUM_SERVICE_STATUSW) +
//                  (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                  (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR));
//
//         if (dwRequiredSize + dwSize > cbBufSize)
//             break;
//
//         /* Copy the service name */
//         wcscpy(lpStringPtr, CurrentService->lpServiceName);
//         lpStatusPtr->lpServiceName = (LPWSTR)((ULONG_PTR)lpStringPtr - (ULONG_PTR)lpBuffer);
//         lpStringPtr += (wcslen(CurrentService->lpServiceName) + 1);
//
//         /* Copy the display name */
//         wcscpy(lpStringPtr, CurrentService->lpDisplayName);
//         lpStatusPtr->lpDisplayName = (LPWSTR)((ULONG_PTR)lpStringPtr - (ULONG_PTR)lpBuffer);
//         lpStringPtr += (wcslen(CurrentService->lpDisplayName) + 1);
//
//         /* Copy the status information */
//         memcpy(&lpStatusPtr->ServiceStatus,
//                &CurrentService->Status,
//                sizeof(SERVICE_STATUS));
//
//         lpStatusPtr++;
//         dwRequiredSize += dwSize;
//     }
//
//     if (dwError == ERROR_SUCCESS)
//     {
//         *pcbBytesNeeded = 0;
//         if (lpResumeIndex) *lpResumeIndex = 0;
//     }
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("REnumServiceGroupW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 36 */
// DWORD
// WINAPI
// RChangeServiceConfig2A(
//     SC_RPC_HANDLE hService,
//     SC_RPC_CONFIG_INFOA Info)
// {
//     SC_RPC_CONFIG_INFOW InfoW = { 0 };
//     DWORD dwRet, dwLength;
//     PVOID ptr = NULL;
//
//     DPRINT("RChangeServiceConfig2A() called\n");
//     DPRINT("dwInfoLevel = %lu\n", Info.dwInfoLevel);
//
//     if ((Info.dwInfoLevel < SERVICE_CONFIG_DESCRIPTION) ||
//         (Info.dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS))
//     {
//         return ERROR_INVALID_LEVEL;
//     }
//
//     InfoW.dwInfoLevel = Info.dwInfoLevel;
//
//     if (InfoW.dwInfoLevel == SERVICE_CONFIG_DESCRIPTION)
//     {
//         LPSERVICE_DESCRIPTIONW lpServiceDescriptionW;
//         LPSERVICE_DESCRIPTIONA lpServiceDescriptionA;
//
//         lpServiceDescriptionA = Info.psd;
//
//         if (lpServiceDescriptionA &&
//             lpServiceDescriptionA->lpDescription)
//         {
//             dwLength = (DWORD)((strlen(lpServiceDescriptionA->lpDescription) + 1) * sizeof(WCHAR));
//
//             lpServiceDescriptionW = HeapAlloc(GetProcessHeap(),
//                                               HEAP_ZERO_MEMORY,
//                                               dwLength + sizeof(SERVICE_DESCRIPTIONW));
//             if (!lpServiceDescriptionW)
//             {
//                 return ERROR_NOT_ENOUGH_MEMORY;
//             }
//
//             lpServiceDescriptionW->lpDescription = (LPWSTR)(lpServiceDescriptionW + 1);
//
//             MultiByteToWideChar(CP_ACP,
//                                 0,
//                                 lpServiceDescriptionA->lpDescription,
//                                 -1,
//                                 lpServiceDescriptionW->lpDescription,
//                                 dwLength);
//
//             ptr = lpServiceDescriptionW;
//             InfoW.psd = lpServiceDescriptionW;
//         }
//     }
//     else if (Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         LPSERVICE_FAILURE_ACTIONSW lpServiceFailureActionsW;
//         LPSERVICE_FAILURE_ACTIONSA lpServiceFailureActionsA;
//         DWORD dwRebootLen  = 0;
//         DWORD dwCommandLen = 0;
//         DWORD dwActionArrayLen = 0;
//         LPWSTR lpStr = NULL;
//
//         lpServiceFailureActionsA = Info.psfa;
//
//         if (lpServiceFailureActionsA)
//         {
//             /*
//              * The following code is inspired by the
//              * SERVICE_CONFIG_FAILURE_ACTIONS case of
//              * the RQueryServiceConfig2W function.
//              */
//
//             /* Retrieve the needed length for the two data strings */
//             if (lpServiceFailureActionsA->lpRebootMsg)
//             {
//                 dwRebootLen = (DWORD)((strlen(lpServiceFailureActionsA->lpRebootMsg) + 1) * sizeof(WCHAR));
//             }
//             if (lpServiceFailureActionsA->lpCommand)
//             {
//                 dwCommandLen = (DWORD)((strlen(lpServiceFailureActionsA->lpCommand) + 1) * sizeof(WCHAR));
//             }
//
//             /*
//              * Retrieve the size of the lpsaActions array if needed.
//              * We will copy the lpsaActions array only if there is at
//              * least one action AND that the original array is valid.
//              */
//             if (lpServiceFailureActionsA->cActions > 0 && lpServiceFailureActionsA->lpsaActions)
//             {
//                 dwActionArrayLen = lpServiceFailureActionsA->cActions * sizeof(SC_ACTION);
//             }
//
//             /* Compute the total length for the UNICODE structure, including data */
//             dwLength = sizeof(SERVICE_FAILURE_ACTIONSW) +
//                        dwActionArrayLen + dwRebootLen + dwCommandLen;
//
//             /* Allocate the structure */
//             lpServiceFailureActionsW = HeapAlloc(GetProcessHeap(),
//                                                  HEAP_ZERO_MEMORY,
//                                                  dwLength);
//             if (!lpServiceFailureActionsW)
//             {
//                 return ERROR_NOT_ENOUGH_MEMORY;
//             }
//
//             /* Copy the members */
//             lpServiceFailureActionsW->dwResetPeriod = lpServiceFailureActionsA->dwResetPeriod;
//             lpServiceFailureActionsW->cActions      = lpServiceFailureActionsA->cActions;
//
//             /* Copy the lpsaActions array if needed */
//             if (dwActionArrayLen > 0)
//             {
//                 /* The storage zone is just after the end of the SERVICE_FAILURE_ACTIONSW structure */
//                 lpServiceFailureActionsW->lpsaActions = (LPSC_ACTION)((ULONG_PTR)(lpServiceFailureActionsW + 1));
//
//                 /* dwActionArrayLen == lpServiceFailureActionsW->cActions * sizeof(SC_ACTION) */
//                 RtlCopyMemory(lpServiceFailureActionsW->lpsaActions,
//                               lpServiceFailureActionsA->lpsaActions,
//                               dwActionArrayLen);
//             }
//             else
//             {
//                 /* No lpsaActions array */
//                 lpServiceFailureActionsW->lpsaActions = NULL;
//             }
//             /* The data strings are stored just after the lpsaActions array */
//             lpStr = (LPWSTR)((ULONG_PTR)(lpServiceFailureActionsW + 1) + dwActionArrayLen);
//
//             /*
//              * Convert the data strings to UNICODE
//              */
//
//             lpServiceFailureActionsW->lpRebootMsg = NULL;
//             lpServiceFailureActionsW->lpCommand   = NULL;
//
//             if (dwRebootLen)
//             {
//                 /* lpRebootMsg points just after the lpsaActions array */
//                 lpServiceFailureActionsW->lpRebootMsg = lpStr;
//
//                 MultiByteToWideChar(CP_ACP,
//                                     0,
//                                     lpServiceFailureActionsA->lpRebootMsg,
//                                     -1,
//                                     lpServiceFailureActionsW->lpRebootMsg,
//                                     dwRebootLen);
//
//                 lpStr += dwRebootLen / sizeof(WCHAR);
//             }
//
//             if (dwCommandLen)
//             {
//                 /* lpRebootMsg points just after the lpRebootMsg data string */
//                 lpServiceFailureActionsW->lpCommand = lpStr;
//
//                 MultiByteToWideChar(CP_ACP,
//                                     0,
//                                     lpServiceFailureActionsA->lpCommand,
//                                     -1,
//                                     lpServiceFailureActionsW->lpCommand,
//                                     dwCommandLen);
//             }
//
//             /* Set the pointers */
//             ptr = lpServiceFailureActionsW;
//             InfoW.psfa = lpServiceFailureActionsW;
//         }
//     }
//
//     dwRet = RChangeServiceConfig2W(hService, InfoW);
//
//     HeapFree(GetProcessHeap(), 0, ptr);
//
//     return dwRet;
// }
//
//
// static DWORD
// ScmSetFailureActions(HKEY hServiceKey,
//                      LPSERVICE_FAILURE_ACTIONSW lpFailureActions)
// {
//     LPSERVICE_FAILURE_ACTIONSW lpReadBuffer = NULL;
//     LPSERVICE_FAILURE_ACTIONSW lpWriteBuffer = NULL;
//     DWORD dwRequiredSize = 0;
//     DWORD dwType = 0;
//     DWORD dwError;
//
//     /* There is nothing to be done if we have no failure actions */
//     if (lpFailureActions == NULL)
//         return ERROR_SUCCESS;
//
//     /*
//      * 1- Retrieve the original value of FailureActions.
//      */
//
//     /* Query value length */
//     dwError = RegQueryValueExW(hServiceKey,
//                                L"FailureActions",
//                                NULL,
//                                &dwType,
//                                NULL,
//                                &dwRequiredSize) as DWORD;
//     if (dwError != ERROR_SUCCESS &&
//         dwError != ERROR_MORE_DATA &&
//         dwError != ERROR_FILE_NOT_FOUND)
//     {
//         return dwError;
//     }
//
//     dwRequiredSize = (dwType == REG_BINARY) ? max(sizeof(SERVICE_FAILURE_ACTIONSW), dwRequiredSize)
//                                             : sizeof(SERVICE_FAILURE_ACTIONSW);
//
//     /* Initialize the read buffer */
//     lpReadBuffer = HeapAlloc(GetProcessHeap(),
//                              HEAP_ZERO_MEMORY,
//                              dwRequiredSize);
//     if (lpReadBuffer == NULL)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     /* Now we can fill the read buffer */
//     if (dwError != ERROR_FILE_NOT_FOUND &&
//         dwType == REG_BINARY)
//     {
//         dwError = RegQueryValueExW(hServiceKey,
//                                    L"FailureActions",
//                                    NULL,
//                                    NULL,
//                                    (LPBYTE)lpReadBuffer,
//                                    &dwRequiredSize) as DWORD;
//         if (dwError != ERROR_SUCCESS &&
//             dwError != ERROR_FILE_NOT_FOUND)
//             goto done;
//
//         if (dwRequiredSize < sizeof(SERVICE_FAILURE_ACTIONSW))
//             dwRequiredSize = sizeof(SERVICE_FAILURE_ACTIONSW);
//     }
//     else
//     {
//         /*
//          * The value of the error doesn't really matter, the only
//          * important thing is that it must be != ERROR_SUCCESS.
//          */
//         dwError = ERROR_INVALID_DATA;
//     }
//
//     if (dwError == ERROR_SUCCESS)
//     {
//         lpReadBuffer->cActions = min(lpReadBuffer->cActions, (dwRequiredSize - sizeof(SERVICE_FAILURE_ACTIONSW)) / sizeof(SC_ACTION));
//         lpReadBuffer->lpsaActions = (lpReadBuffer->cActions > 0 ? (LPSC_ACTION)(lpReadBuffer + 1) : NULL);
//     }
//     else
//     {
//         lpReadBuffer->dwResetPeriod = 0;
//         lpReadBuffer->cActions = 0;
//         lpReadBuffer->lpsaActions = NULL;
//     }
//
//     lpReadBuffer->lpRebootMsg = NULL;
//     lpReadBuffer->lpCommand = NULL;
//
//     /*
//      * 2- Initialize the new value to set.
//      */
//
//     dwRequiredSize = sizeof(SERVICE_FAILURE_ACTIONSW);
//
//     if (lpFailureActions->lpsaActions == NULL)
//     {
//         /*
//          * lpFailureActions->cActions is ignored.
//          * Therefore we use the original values
//          * of cActions and lpsaActions.
//          */
//         dwRequiredSize += lpReadBuffer->cActions * sizeof(SC_ACTION);
//     }
//     else
//     {
//         /*
//          * The reset period and array of failure actions
//          * are deleted if lpFailureActions->cActions == 0 .
//          */
//         dwRequiredSize += lpFailureActions->cActions * sizeof(SC_ACTION);
//     }
//
//     lpWriteBuffer = HeapAlloc(GetProcessHeap(),
//                               HEAP_ZERO_MEMORY,
//                               dwRequiredSize);
//     if (lpWriteBuffer == NULL)
//     {
//         dwError = ERROR_NOT_ENOUGH_MEMORY;
//         goto done;
//     }
//
//     /* Clean the pointers as they have no meaning when the structure is stored in the registry */
//     lpWriteBuffer->lpRebootMsg = NULL;
//     lpWriteBuffer->lpCommand = NULL;
//     lpWriteBuffer->lpsaActions = NULL;
//
//     /* Set the members */
//     if (lpFailureActions->lpsaActions == NULL)
//     {
//         /*
//          * lpFailureActions->dwResetPeriod and lpFailureActions->cActions are ignored.
//          * Therefore we use the original values of dwResetPeriod, cActions and lpsaActions.
//          */
//         lpWriteBuffer->dwResetPeriod = lpReadBuffer->dwResetPeriod;
//         lpWriteBuffer->cActions = lpReadBuffer->cActions;
//
//         if (lpReadBuffer->lpsaActions != NULL)
//         {
//             memmove(lpWriteBuffer + 1,
//                     lpReadBuffer->lpsaActions,
//                     lpReadBuffer->cActions * sizeof(SC_ACTION));
//         }
//     }
//     else
//     {
//         if (lpFailureActions->cActions > 0)
//         {
//             lpWriteBuffer->dwResetPeriod = lpFailureActions->dwResetPeriod;
//             lpWriteBuffer->cActions = lpFailureActions->cActions;
//
//             memmove(lpWriteBuffer + 1,
//                     lpFailureActions->lpsaActions,
//                     lpFailureActions->cActions * sizeof(SC_ACTION));
//         }
//         else
//         {
//             /* The reset period and array of failure actions are deleted */
//             lpWriteBuffer->dwResetPeriod = 0;
//             lpWriteBuffer->cActions = 0;
//         }
//     }
//
//     /* Save the new failure actions into the registry */
//     dwError = RegSetValueExW(hServiceKey,
//                              L"FailureActions",
//                              0,
//                              REG_BINARY,
//                              (LPBYTE)lpWriteBuffer,
//                              dwRequiredSize) as DWORD;
//
//     /* We modify the strings only in case of success.*/
//     if (dwError == ERROR_SUCCESS)
//     {
//         /* Modify the Reboot Message value, if specified */
//         if (lpFailureActions->lpRebootMsg != NULL)
//         {
//             /* If the Reboot Message is "" then we delete it */
//             if (*lpFailureActions->lpRebootMsg == 0)
//             {
//                 DPRINT("Delete Reboot Message value\n");
//                 RegDeleteValueW(hServiceKey, L"RebootMessage");
//             }
//             else
//             {
//                 DPRINT("Setting Reboot Message value %S\n", lpFailureActions->lpRebootMsg);
//                 RegSetValueExW(hServiceKey,
//                                L"RebootMessage",
//                                0,
//                                REG_SZ,
//                                (LPBYTE)lpFailureActions->lpRebootMsg,
//                                (DWORD)((wcslen(lpFailureActions->lpRebootMsg) + 1) * sizeof(WCHAR)));
//             }
//         }
//
//         /* Modify the Failure Command value, if specified */
//         if (lpFailureActions->lpCommand != NULL)
//         {
//             /* If the FailureCommand string is an empty string, delete the value */
//             if (*lpFailureActions->lpCommand == 0)
//             {
//                 DPRINT("Delete Failure Command value\n");
//                 RegDeleteValueW(hServiceKey, L"FailureCommand");
//             }
//             else
//             {
//                 DPRINT("Setting Failure Command value %S\n", lpFailureActions->lpCommand);
//                 RegSetValueExW(hServiceKey,
//                                L"FailureCommand",
//                                0,
//                                REG_SZ,
//                                (LPBYTE)lpFailureActions->lpCommand,
//                                (DWORD)((wcslen(lpFailureActions->lpCommand) + 1) * sizeof(WCHAR)));
//             }
//         }
//     }
//
// done:
//     if (lpWriteBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, lpWriteBuffer);
//
//     if (lpReadBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, lpReadBuffer);
//
//     return dwError;
// }
//
//
// /* Function 37 */
// DWORD
// WINAPI
// RChangeServiceConfig2W(
//     SC_RPC_HANDLE hService,
//     SC_RPC_CONFIG_INFOW Info)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     ACCESS_MASK RequiredAccess = SERVICE_CHANGE_CONFIG;
//
//     DPRINT("RChangeServiceConfig2W() called\n");
//     DPRINT("dwInfoLevel = %lu\n", Info.dwInfoLevel);
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if ((Info.dwInfoLevel < SERVICE_CONFIG_DESCRIPTION) ||
//         (Info.dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS))
//     {
//         return ERROR_INVALID_LEVEL;
//     }
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//         RequiredAccess |= SERVICE_START;
//
//     /* Check the access rights */
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   RequiredAccess))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     if (Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         /* FIXME: Check if the caller has the SE_SHUTDOWN_NAME privilege */
//
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Failure actions can only be set for Win32 services, not for drivers */
//     if (Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         if (lpService->Status.dwServiceType & SERVICE_DRIVER)
//             return ERROR_CANNOT_DETECT_DRIVER_FAILURE;
//     }
//
//     /* Lock the service database exclusively */
//     ScmLockDatabaseExclusive();
//
//     if (lpService->bDeleted)
//     {
//         DPRINT("Service has already been marked for delete\n");
//         dwError = ERROR_SERVICE_MARKED_FOR_DELETE;
//         goto done;
//     }
//
//     /* Open the service key */
//     dwError = ScmOpenServiceKey(lpService->szServiceName,
//                                 KEY_READ | KEY_SET_VALUE,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     if (Info.dwInfoLevel == SERVICE_CONFIG_DESCRIPTION)
//     {
//         LPSERVICE_DESCRIPTIONW lpServiceDescription = (LPSERVICE_DESCRIPTIONW)Info.psd;
//
//         /* Modify the service description, if specified */
//         if (lpServiceDescription != NULL &&
//             lpServiceDescription->lpDescription != NULL)
//         {
//             /* If the description is "" then we delete it */
//             if (*lpServiceDescription->lpDescription == 0)
//             {
//                 DPRINT("Delete service description\n");
//                 dwError = RegDeleteValueW(hServiceKey, L"Description") as DWORD;
//
//                 if (dwError == ERROR_FILE_NOT_FOUND)
//                     dwError = ERROR_SUCCESS;
//             }
//             else
//             {
//                 DPRINT("Setting service description value %S\n", lpServiceDescription->lpDescription);
//                 dwError = RegSetValueExW(hServiceKey,
//                                          L"Description",
//                                          0,
//                                          REG_SZ,
//                                          (LPBYTE)lpServiceDescription->lpDescription,
//                                          (DWORD)((wcslen(lpServiceDescription->lpDescription) + 1) * sizeof(WCHAR))) as DWORD;
//             }
//         }
//         else
//         {
//             dwError = ERROR_SUCCESS;
//         }
//     }
//     else if (Info.dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         dwError = ScmSetFailureActions(hServiceKey,
//                                        (LPSERVICE_FAILURE_ACTIONSW)Info.psfa);
//     }
//
// done:
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("RChangeServiceConfig2W() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 38 */
// DWORD
// WINAPI
// RQueryServiceConfig2A(
//     SC_RPC_HANDLE hService,
//     DWORD dwInfoLevel,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_8K pcbBytesNeeded)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     DWORD dwRequiredSize = 0;
//     DWORD dwType = 0;
//     LPWSTR lpDescriptionW = NULL;
//     LPWSTR lpRebootMessageW = NULL;
//     LPWSTR lpFailureCommandW = NULL;
//
//     DPRINT("RQueryServiceConfig2A() called hService %p dwInfoLevel %u, lpBuffer %p cbBufSize %u pcbBytesNeeded %p\n",
//            hService, dwInfoLevel, lpBuffer, cbBufSize, pcbBytesNeeded);
//
//     if (!lpBuffer)
//         return ERROR_INVALID_ADDRESS;
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if ((dwInfoLevel < SERVICE_CONFIG_DESCRIPTION) ||
//         (dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS))
//     {
//         return ERROR_INVALID_LEVEL;
//     }
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_CONFIG))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     dwError = ScmOpenServiceKey(lpService->lpServiceName,
//                                 KEY_READ,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     if (dwInfoLevel == SERVICE_CONFIG_DESCRIPTION)
//     {
//         LPSERVICE_DESCRIPTIONA lpServiceDescription = (LPSERVICE_DESCRIPTIONA)lpBuffer;
//         LPSTR lpStr;
//
//         dwError = ScmReadString(hServiceKey,
//                                 L"Description",
//                                 &lpDescriptionW);
//         if (dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND)
//             goto done;
//
//         *pcbBytesNeeded = sizeof(SERVICE_DESCRIPTIONA);
//         if (dwError == ERROR_SUCCESS)
//             *pcbBytesNeeded += (DWORD)((wcslen(lpDescriptionW) + 1) * sizeof(WCHAR));
//
//         if (cbBufSize < *pcbBytesNeeded)
//         {
//             dwError = ERROR_INSUFFICIENT_BUFFER;
//             goto done;
//         }
//
//         if (dwError == ERROR_SUCCESS)
//         {
//             lpStr = (LPSTR)(lpServiceDescription + 1);
//
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpDescriptionW,
//                                 -1,
//                                 lpStr,
//                                 (int)wcslen(lpDescriptionW),
//                                 NULL,
//                                 NULL);
//             lpServiceDescription->lpDescription = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceDescription);
//         }
//         else
//         {
//             lpServiceDescription->lpDescription = NULL;
//             dwError = ERROR_SUCCESS;
//         }
//     }
//     else if (dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         LPSERVICE_FAILURE_ACTIONSA lpFailureActions = (LPSERVICE_FAILURE_ACTIONSA)lpBuffer;
//         LPSTR lpStr = NULL;
//
//         /* Query value length */
//         dwError = RegQueryValueExW(hServiceKey,
//                                    L"FailureActions",
//                                    NULL,
//                                    &dwType,
//                                    NULL,
//                                    &dwRequiredSize) as DWORD;
//         if (dwError != ERROR_SUCCESS &&
//             dwError != ERROR_MORE_DATA &&
//             dwError != ERROR_FILE_NOT_FOUND)
//         {
//             goto done;
//         }
//
//         dwRequiredSize = (dwType == REG_BINARY) ? max(sizeof(SERVICE_FAILURE_ACTIONSA), dwRequiredSize)
//                                                 : sizeof(SERVICE_FAILURE_ACTIONSA);
//
//         /* Get the strings */
//         ScmReadString(hServiceKey,
//                       L"FailureCommand",
//                       &lpFailureCommandW);
//
//         ScmReadString(hServiceKey,
//                       L"RebootMessage",
//                       &lpRebootMessageW);
//
//         if (lpRebootMessageW)
//             dwRequiredSize += (DWORD)((wcslen(lpRebootMessageW) + 1) * sizeof(WCHAR));
//
//         if (lpFailureCommandW)
//             dwRequiredSize += (DWORD)((wcslen(lpFailureCommandW) + 1) * sizeof(WCHAR));
//
//         if (cbBufSize < dwRequiredSize)
//         {
//             *pcbBytesNeeded = dwRequiredSize;
//             dwError = ERROR_INSUFFICIENT_BUFFER;
//             goto done;
//         }
//
//         /* Now we can fill the buffer */
//         if (dwError != ERROR_FILE_NOT_FOUND && dwType == REG_BINARY)
//         {
//             dwError = RegQueryValueExW(hServiceKey,
//                                        L"FailureActions",
//                                        NULL,
//                                        NULL,
//                                        (LPBYTE)lpFailureActions,
//                                        &dwRequiredSize) as DWORD;
//             if (dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND)
//                 goto done;
//
//             if (dwRequiredSize < sizeof(SERVICE_FAILURE_ACTIONSA))
//                 dwRequiredSize = sizeof(SERVICE_FAILURE_ACTIONSA);
//         }
//         else
//         {
//             /*
//              * The value of the error doesn't really matter, the only
//              * important thing is that it must be != ERROR_SUCCESS .
//              */
//             dwError = ERROR_INVALID_DATA;
//         }
//
//         if (dwError == ERROR_SUCCESS)
//         {
//             lpFailureActions->cActions = min(lpFailureActions->cActions, (dwRequiredSize - sizeof(SERVICE_FAILURE_ACTIONSA)) / sizeof(SC_ACTION));
//
//             /* Here lpFailureActions->lpsaActions contains an offset. The conversion is done by the caller. */
//             lpFailureActions->lpsaActions = (lpFailureActions->cActions > 0 ? (LPSC_ACTION)(ULONG_PTR)sizeof(SERVICE_FAILURE_ACTIONSA) : NULL);
//
//             lpStr = (LPSTR)((ULONG_PTR)(lpFailureActions + 1) + lpFailureActions->cActions * sizeof(SC_ACTION));
//         }
//         else
//         {
//             lpFailureActions->dwResetPeriod = 0;
//             lpFailureActions->cActions = 0;
//             lpFailureActions->lpsaActions = NULL;
//             lpStr = (LPSTR)(lpFailureActions + 1);
//         }
//
//         lpFailureActions->lpRebootMsg = NULL;
//         lpFailureActions->lpCommand = NULL;
//
//         if (lpRebootMessageW)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpRebootMessageW,
//                                 -1,
//                                 lpStr,
//                                 (int)wcslen(lpRebootMessageW),
//                                 NULL,
//                                 NULL);
//             lpFailureActions->lpRebootMsg = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpFailureActions);
//             lpStr += strlen(lpStr) + 1;
//         }
//
//         if (lpFailureCommandW)
//         {
//             WideCharToMultiByte(CP_ACP,
//                                 0,
//                                 lpFailureCommandW,
//                                 -1,
//                                 lpStr,
//                                 (int)wcslen(lpFailureCommandW),
//                                 NULL,
//                                 NULL);
//             lpFailureActions->lpCommand = (LPSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpFailureActions);
//             /* lpStr += strlen(lpStr) + 1; */
//         }
//
//         dwError = ERROR_SUCCESS;
//     }
//
// done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (lpDescriptionW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDescriptionW);
//
//     if (lpRebootMessageW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpRebootMessageW);
//
//     if (lpFailureCommandW != NULL)
//         HeapFree(GetProcessHeap(), 0, lpFailureCommandW);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     DPRINT("RQueryServiceConfig2A() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 39 */
// DWORD
// WINAPI
// RQueryServiceConfig2W(
//     SC_RPC_HANDLE hService,
//     DWORD dwInfoLevel,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_8K pcbBytesNeeded)
// {
//     DWORD dwError = ERROR_SUCCESS;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService = NULL;
//     HKEY hServiceKey = NULL;
//     DWORD dwRequiredSize = 0;
//     DWORD dwType = 0;
//     LPWSTR lpDescription = NULL;
//     LPWSTR lpRebootMessage = NULL;
//     LPWSTR lpFailureCommand = NULL;
//
//     DPRINT("RQueryServiceConfig2W() called\n");
//
//     if (!lpBuffer)
//         return ERROR_INVALID_ADDRESS;
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if ((dwInfoLevel < SERVICE_CONFIG_DESCRIPTION) ||
//         (dwInfoLevel > SERVICE_CONFIG_FAILURE_ACTIONS))
//     {
//         return ERROR_INVALID_LEVEL;
//     }
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_CONFIG))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     dwError = ScmOpenServiceKey(lpService->lpServiceName,
//                                 KEY_READ,
//                                 &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//         goto done;
//
//     if (dwInfoLevel == SERVICE_CONFIG_DESCRIPTION)
//     {
//         LPSERVICE_DESCRIPTIONW lpServiceDescription = (LPSERVICE_DESCRIPTIONW)lpBuffer;
//         LPWSTR lpStr;
//
//         dwError = ScmReadString(hServiceKey,
//                                 L"Description",
//                                 &lpDescription);
//         if (dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND)
//             goto done;
//
//         *pcbBytesNeeded = sizeof(SERVICE_DESCRIPTIONW);
//         if (dwError == ERROR_SUCCESS)
//             *pcbBytesNeeded += (DWORD)((wcslen(lpDescription) + 1) * sizeof(WCHAR));
//
//         if (cbBufSize < *pcbBytesNeeded)
//         {
//             dwError = ERROR_INSUFFICIENT_BUFFER;
//             goto done;
//         }
//
//         if (dwError == ERROR_SUCCESS)
//         {
//             lpStr = (LPWSTR)(lpServiceDescription + 1);
//             wcscpy(lpStr, lpDescription);
//             lpServiceDescription->lpDescription = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpServiceDescription);
//         }
//         else
//         {
//             lpServiceDescription->lpDescription = NULL;
//             dwError = ERROR_SUCCESS;
//         }
//     }
//     else if (dwInfoLevel == SERVICE_CONFIG_FAILURE_ACTIONS)
//     {
//         LPSERVICE_FAILURE_ACTIONSW lpFailureActions = (LPSERVICE_FAILURE_ACTIONSW)lpBuffer;
//         LPWSTR lpStr = NULL;
//
//         /* Query value length */
//         dwError = RegQueryValueExW(hServiceKey,
//                                    L"FailureActions",
//                                    NULL,
//                                    &dwType,
//                                    NULL,
//                                    &dwRequiredSize) as DWORD;
//         if (dwError != ERROR_SUCCESS &&
//             dwError != ERROR_MORE_DATA &&
//             dwError != ERROR_FILE_NOT_FOUND)
//         {
//             goto done;
//         }
//
//         dwRequiredSize = (dwType == REG_BINARY) ? max(sizeof(SERVICE_FAILURE_ACTIONSW), dwRequiredSize)
//                                                 : sizeof(SERVICE_FAILURE_ACTIONSW);
//
//         /* Get the strings */
//         ScmReadString(hServiceKey,
//                       L"FailureCommand",
//                       &lpFailureCommand);
//
//         ScmReadString(hServiceKey,
//                       L"RebootMessage",
//                       &lpRebootMessage);
//
//         if (lpRebootMessage)
//             dwRequiredSize += (DWORD)((wcslen(lpRebootMessage) + 1) * sizeof(WCHAR));
//
//         if (lpFailureCommand)
//             dwRequiredSize += (DWORD)((wcslen(lpFailureCommand) + 1) * sizeof(WCHAR));
//
//         if (cbBufSize < dwRequiredSize)
//         {
//             *pcbBytesNeeded = dwRequiredSize;
//             dwError = ERROR_INSUFFICIENT_BUFFER;
//             goto done;
//         }
//
//         /* Now we can fill the buffer */
//         if (dwError != ERROR_FILE_NOT_FOUND && dwType == REG_BINARY)
//         {
//             dwError = RegQueryValueExW(hServiceKey,
//                                        L"FailureActions",
//                                        NULL,
//                                        NULL,
//                                        (LPBYTE)lpFailureActions,
//                                        &dwRequiredSize) as DWORD;
//             if (dwError != ERROR_SUCCESS && dwError != ERROR_FILE_NOT_FOUND)
//                 goto done;
//
//             if (dwRequiredSize < sizeof(SERVICE_FAILURE_ACTIONSW))
//                 dwRequiredSize = sizeof(SERVICE_FAILURE_ACTIONSW);
//         }
//         else
//         {
//             /*
//              * The value of the error doesn't really matter, the only
//              * important thing is that it must be != ERROR_SUCCESS .
//              */
//             dwError = ERROR_INVALID_DATA;
//         }
//
//         if (dwError == ERROR_SUCCESS)
//         {
//             lpFailureActions->cActions = min(lpFailureActions->cActions, (dwRequiredSize - sizeof(SERVICE_FAILURE_ACTIONSW)) / sizeof(SC_ACTION));
//
//             /* Here lpFailureActions->lpsaActions contains an offset. The conversion is done by the caller. */
//             lpFailureActions->lpsaActions = (lpFailureActions->cActions > 0 ? (LPSC_ACTION)(ULONG_PTR)sizeof(SERVICE_FAILURE_ACTIONSW) : NULL);
//
//             lpStr = (LPWSTR)((ULONG_PTR)(lpFailureActions + 1) + lpFailureActions->cActions * sizeof(SC_ACTION));
//         }
//         else
//         {
//             lpFailureActions->dwResetPeriod = 0;
//             lpFailureActions->cActions = 0;
//             lpFailureActions->lpsaActions = NULL;
//             lpStr = (LPWSTR)(lpFailureActions + 1);
//         }
//
//         lpFailureActions->lpRebootMsg = NULL;
//         lpFailureActions->lpCommand   = NULL;
//
//         if (lpRebootMessage)
//         {
//             wcscpy(lpStr, lpRebootMessage);
//             lpFailureActions->lpRebootMsg = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpFailureActions);
//             lpStr += wcslen(lpStr) + 1;
//         }
//
//         if (lpFailureCommand)
//         {
//             wcscpy(lpStr, lpFailureCommand);
//             lpFailureActions->lpCommand = (LPWSTR)((ULONG_PTR)lpStr - (ULONG_PTR)lpFailureActions);
//             /* lpStr += wcslen(lpStr) + 1; */
//         }
//
//         dwError = ERROR_SUCCESS;
//     }
//
// done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     if (lpDescription != NULL)
//         HeapFree(GetProcessHeap(), 0, lpDescription);
//
//     if (lpRebootMessage != NULL)
//         HeapFree(GetProcessHeap(), 0, lpRebootMessage);
//
//     if (lpFailureCommand != NULL)
//         HeapFree(GetProcessHeap(), 0, lpFailureCommand);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     DPRINT("RQueryServiceConfig2W() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 40 */
// DWORD
// WINAPI
// RQueryServiceStatusEx(
//     SC_RPC_HANDLE hService,
//     SC_STATUS_TYPE InfoLevel,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_8K pcbBytesNeeded)
// {
//     LPSERVICE_STATUS_PROCESS lpStatus;
//     PSERVICE_HANDLE hSvc;
//     PSERVICE lpService;
//
//     DPRINT("RQueryServiceStatusEx() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if (InfoLevel != SC_STATUS_PROCESS_INFO)
//         return ERROR_INVALID_LEVEL;
//
//     *pcbBytesNeeded = sizeof(SERVICE_STATUS_PROCESS);
//
//     if (cbBufSize < sizeof(SERVICE_STATUS_PROCESS))
//         return ERROR_INSUFFICIENT_BUFFER;
//
//     hSvc = ScmGetServiceFromHandle(hService);
//     if (hSvc == NULL)
//     {
//         DPRINT1("Invalid service handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (!RtlAreAllAccessesGranted(hSvc->Handle.DesiredAccess,
//                                   SERVICE_QUERY_STATUS))
//     {
//         DPRINT("Insufficient access rights! 0x%lx\n", hSvc->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     lpService = hSvc->ServiceEntry;
//     if (lpService == NULL)
//     {
//         DPRINT("lpService == NULL\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     lpStatus = (LPSERVICE_STATUS_PROCESS)lpBuffer;
//
//     /* Return service status information */
//     RtlCopyMemory(lpStatus,
//                   &lpService->Status,
//                   sizeof(SERVICE_STATUS));
//
//     /* Copy the service process ID */
//     if ((lpService->Status.dwCurrentState == SERVICE_STOPPED) || (lpService->lpImage == NULL))
//         lpStatus->dwProcessId = 0;
//     else
//         lpStatus->dwProcessId = lpService->lpImage->dwProcessId;
//
//     lpStatus->dwServiceFlags = 0;   /* FIXME */
//
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 41 */
// DWORD
// WINAPI
// REnumServicesStatusExA(
//     SC_RPC_HANDLE hSCManager,
//     SC_ENUM_TYPE InfoLevel,
//     DWORD dwServiceType,
//     DWORD dwServiceState,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned,
//     LPBOUNDED_DWORD_256K lpResumeIndex,
//     LPCSTR pszGroupName)
// {
//     LPENUM_SERVICE_STATUS_PROCESSW lpStatusPtrW = NULL;
//     LPENUM_SERVICE_STATUS_PROCESSW lpStatusPtrIncrW;
//     LPENUM_SERVICE_STATUS_PROCESSA lpStatusPtrA = NULL;
//     LPWSTR lpStringPtrW;
//     LPSTR lpStringPtrA;
//     LPWSTR pszGroupNameW = NULL;
//     DWORD dwError;
//     DWORD dwServiceCount;
//
//     DPRINT("REnumServicesStatusExA() called\n");
//
//     if (pcbBytesNeeded == NULL || lpServicesReturned == NULL)
//     {
//         return ERROR_INVALID_ADDRESS;
//     }
//
//     if (pszGroupName)
//     {
//         pszGroupNameW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (strlen(pszGroupName) + 1) * sizeof(WCHAR));
//         if (!pszGroupNameW)
//         {
//              DPRINT("Failed to allocate buffer\n");
//              dwError = ERROR_NOT_ENOUGH_MEMORY;
//              goto Done;
//         }
//
//         MultiByteToWideChar(CP_ACP,
//                             0,
//                             pszGroupName,
//                             -1,
//                             pszGroupNameW,
//                             (int)(strlen(pszGroupName) + 1));
//     }
//
//     if ((cbBufSize > 0) && (lpBuffer))
//     {
//         lpStatusPtrW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, cbBufSize);
//         if (!lpStatusPtrW)
//         {
//             DPRINT("Failed to allocate buffer\n");
//             dwError = ERROR_NOT_ENOUGH_MEMORY;
//             goto Done;
//         }
//     }
//
//     dwError = REnumServicesStatusExW(hSCManager,
//                                      InfoLevel,
//                                      dwServiceType,
//                                      dwServiceState,
//                                      (LPBYTE)lpStatusPtrW,
//                                      cbBufSize,
//                                      pcbBytesNeeded,
//                                      lpServicesReturned,
//                                      lpResumeIndex,
//                                      pszGroupNameW);
//
//     /* if no services were returned then we are Done */
//     if (*lpServicesReturned == 0)
//         goto Done;
//
//     lpStatusPtrIncrW = lpStatusPtrW;
//     lpStatusPtrA = (LPENUM_SERVICE_STATUS_PROCESSA)lpBuffer;
//     lpStringPtrA = (LPSTR)((ULONG_PTR)lpBuffer +
//                   *lpServicesReturned * sizeof(ENUM_SERVICE_STATUS_PROCESSA));
//     lpStringPtrW = (LPWSTR)((ULONG_PTR)lpStatusPtrW +
//                   *lpServicesReturned * sizeof(ENUM_SERVICE_STATUS_PROCESSW));
//
//     for (dwServiceCount = 0; dwServiceCount < *lpServicesReturned; dwServiceCount++)
//     {
//         /* Copy the service name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpStringPtrW,
//                             -1,
//                             lpStringPtrA,
//                             (int)wcslen(lpStringPtrW),
//                             0,
//                             0);
//
//         lpStatusPtrA->lpServiceName = (LPSTR)((ULONG_PTR)lpStringPtrA - (ULONG_PTR)lpBuffer);
//         lpStringPtrA += wcslen(lpStringPtrW) + 1;
//         lpStringPtrW += wcslen(lpStringPtrW) + 1;
//
//         /* Copy the display name */
//         WideCharToMultiByte(CP_ACP,
//                             0,
//                             lpStringPtrW,
//                             -1,
//                             lpStringPtrA,
//                             (int)wcslen(lpStringPtrW),
//                             0,
//                             0);
//
//         lpStatusPtrA->lpDisplayName = (LPSTR)((ULONG_PTR)lpStringPtrA - (ULONG_PTR)lpBuffer);
//         lpStringPtrA += wcslen(lpStringPtrW) + 1;
//         lpStringPtrW += wcslen(lpStringPtrW) + 1;
//
//         /* Copy the status information */
//         memcpy(&lpStatusPtrA->ServiceStatusProcess,
//                &lpStatusPtrIncrW->ServiceStatusProcess,
//                sizeof(SERVICE_STATUS));
//
//         /* Copy the service process ID */
//         lpStatusPtrA->ServiceStatusProcess.dwProcessId = lpStatusPtrIncrW->ServiceStatusProcess.dwProcessId;
//
//         lpStatusPtrA->ServiceStatusProcess.dwServiceFlags = 0; /* FIXME */
//
//         lpStatusPtrIncrW++;
//         lpStatusPtrA++;
//     }
//
// Done:
//     if (pszGroupNameW)
//         HeapFree(GetProcessHeap(), 0, pszGroupNameW);
//
//     if (lpStatusPtrW)
//         HeapFree(GetProcessHeap(), 0, lpStatusPtrW);
//
//     DPRINT("REnumServicesStatusExA() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 42 */
// DWORD
// WINAPI
// REnumServicesStatusExW(
//     SC_RPC_HANDLE hSCManager,
//     SC_ENUM_TYPE InfoLevel,
//     DWORD dwServiceType,
//     DWORD dwServiceState,
//     LPBYTE lpBuffer,
//     DWORD cbBufSize,
//     LPBOUNDED_DWORD_256K pcbBytesNeeded,
//     LPBOUNDED_DWORD_256K lpServicesReturned,
//     LPBOUNDED_DWORD_256K lpResumeIndex,
//     LPCWSTR pszGroupName)
// {
//     PMANAGER_HANDLE hManager;
//     PSERVICE lpService;
//     DWORD dwError = ERROR_SUCCESS;
//     PLIST_ENTRY ServiceEntry;
//     PSERVICE CurrentService;
//     DWORD dwState;
//     DWORD dwRequiredSize;
//     DWORD dwServiceCount;
//     DWORD dwSize;
//     DWORD dwLastResumeCount = 0;
//     LPENUM_SERVICE_STATUS_PROCESSW lpStatusPtr;
//     LPWSTR lpStringPtr;
//
//     DPRINT("REnumServicesStatusExW() called\n");
//
//     if (ScmShutdown)
//         return ERROR_SHUTDOWN_IN_PROGRESS;
//
//     if (InfoLevel != SC_ENUM_PROCESS_INFO)
//         return ERROR_INVALID_LEVEL;
//
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid service manager handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     if (pcbBytesNeeded == NULL || lpServicesReturned == NULL)
//     {
//         return ERROR_INVALID_ADDRESS;
//     }
//
//     *pcbBytesNeeded = 0;
//     *lpServicesReturned = 0;
//
//     if ((dwServiceType == 0) ||
//         ((dwServiceType & ~SERVICE_TYPE_ALL) != 0))
//     {
//         DPRINT("Invalid Service Type\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     if ((dwServiceState == 0) ||
//         ((dwServiceState & ~SERVICE_STATE_ALL) != 0))
//     {
//         DPRINT("Invalid Service State\n");
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         DPRINT("Insufficient access rights 0x%lx\n",
//                hManager->Handle.DesiredAccess);
//         return ERROR_ACCESS_DENIED;
//     }
//
//     if (lpResumeIndex)
//         dwLastResumeCount = *lpResumeIndex;
//
//     /* Lock the service database shared */
//     ScmLockDatabaseShared();
//
//     lpService = ScmGetServiceEntryByResumeCount(dwLastResumeCount);
//     if (lpService == NULL)
//     {
//         dwError = ERROR_SUCCESS;
//         goto Done;
//     }
//
//     dwRequiredSize = 0;
//     dwServiceCount = 0;
//
//     for (ServiceEntry = &lpService->ServiceListEntry;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwSize = sizeof(ENUM_SERVICE_STATUS_PROCESSW) +
//                  (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                  (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR));
//
//         if (dwRequiredSize + dwSize <= cbBufSize)
//         {
//             DPRINT("Service name: %S  fit\n", CurrentService->lpServiceName);
//             dwRequiredSize += dwSize;
//             dwServiceCount++;
//             dwLastResumeCount = CurrentService->dwResumeCount;
//         }
//         else
//         {
//             DPRINT("Service name: %S  no fit\n", CurrentService->lpServiceName);
//             break;
//         }
//
//     }
//
//     DPRINT("dwRequiredSize: %lu\n", dwRequiredSize);
//     DPRINT("dwServiceCount: %lu\n", dwServiceCount);
//
//     for (;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwRequiredSize += (sizeof(ENUM_SERVICE_STATUS_PROCESSW) +
//                            (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                            (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR)));
//
//         dwError = ERROR_MORE_DATA;
//     }
//
//     DPRINT("*pcbBytesNeeded: %lu\n", dwRequiredSize);
//
//     if (lpResumeIndex)
//         *lpResumeIndex = dwLastResumeCount;
//
//     *lpServicesReturned = dwServiceCount;
//     *pcbBytesNeeded = dwRequiredSize;
//
//     /* If there was no services that matched */
//     if ((!dwServiceCount) && (dwError != ERROR_MORE_DATA))
//     {
//         dwError = ERROR_SERVICE_DOES_NOT_EXIST;
//         goto Done;
//     }
//
//     lpStatusPtr = (LPENUM_SERVICE_STATUS_PROCESSW)lpBuffer;
//     lpStringPtr = (LPWSTR)((ULONG_PTR)lpBuffer +
//                            dwServiceCount * sizeof(ENUM_SERVICE_STATUS_PROCESSW));
//
//     dwRequiredSize = 0;
//     for (ServiceEntry = &lpService->ServiceListEntry;
//          ServiceEntry != &ServiceListHead;
//          ServiceEntry = ServiceEntry->Flink)
//     {
//         CurrentService = CONTAINING_RECORD(ServiceEntry,
//                                            SERVICE,
//                                            ServiceListEntry);
//
//         if ((CurrentService->Status.dwServiceType & dwServiceType) == 0)
//             continue;
//
//         dwState = SERVICE_ACTIVE;
//         if (CurrentService->Status.dwCurrentState == SERVICE_STOPPED)
//             dwState = SERVICE_INACTIVE;
//
//         if ((dwState & dwServiceState) == 0)
//             continue;
//
//         if (pszGroupName)
//         {
//             if (*pszGroupName == 0)
//             {
//                 if (CurrentService->lpGroup != NULL)
//                     continue;
//             }
//             else
//             {
//                 if ((CurrentService->lpGroup == NULL) ||
//                     _wcsicmp(pszGroupName, CurrentService->lpGroup->lpGroupName) != 0)
//                     continue;
//             }
//         }
//
//         dwSize = sizeof(ENUM_SERVICE_STATUS_PROCESSW) +
//                  (DWORD)((wcslen(CurrentService->lpServiceName) + 1) * sizeof(WCHAR)) +
//                  (DWORD)((wcslen(CurrentService->lpDisplayName) + 1) * sizeof(WCHAR));
//
//         if (dwRequiredSize + dwSize <= cbBufSize)
//         {
//             /* Copy the service name */
//             wcscpy(lpStringPtr,
//                    CurrentService->lpServiceName);
//             lpStatusPtr->lpServiceName = (LPWSTR)((ULONG_PTR)lpStringPtr - (ULONG_PTR)lpBuffer);
//             lpStringPtr += (wcslen(CurrentService->lpServiceName) + 1);
//
//             /* Copy the display name */
//             wcscpy(lpStringPtr,
//                    CurrentService->lpDisplayName);
//             lpStatusPtr->lpDisplayName = (LPWSTR)((ULONG_PTR)lpStringPtr - (ULONG_PTR)lpBuffer);
//             lpStringPtr += (wcslen(CurrentService->lpDisplayName) + 1);
//
//             /* Copy the status information */
//             memcpy(&lpStatusPtr->ServiceStatusProcess,
//                    &CurrentService->Status,
//                    sizeof(SERVICE_STATUS));
//
//             /* Copy the service process ID */
//             if ((CurrentService->Status.dwCurrentState == SERVICE_STOPPED) || (CurrentService->lpImage == NULL))
//                 lpStatusPtr->ServiceStatusProcess.dwProcessId = 0;
//             else
//                 lpStatusPtr->ServiceStatusProcess.dwProcessId = CurrentService->lpImage->dwProcessId;
//
//             lpStatusPtr->ServiceStatusProcess.dwServiceFlags = 0; /* FIXME */
//
//             lpStatusPtr++;
//             dwRequiredSize += dwSize;
//         }
//         else
//         {
//             break;
//         }
//     }
//
//     if (dwError == 0)
//     {
//         *pcbBytesNeeded = 0;
//         if (lpResumeIndex)
//             *lpResumeIndex = 0;
//     }
//
// Done:
//     /* Unlock the service database */
//     ScmUnlockDatabase();
//
//     DPRINT("REnumServicesStatusExW() done (Error %lu)\n", dwError);
//
//     return dwError;
// }
//
//
// /* Function 43 */
// DWORD
// WINAPI
// RSendTSMessage(
//     handle_t BindingHandle)  /* FIXME */
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 44 */
// DWORD
// WINAPI
// RCreateServiceWOW64A(
//     handle_t BindingHandle,
//     LPSTR lpServiceName,
//     LPSTR lpDisplayName,
//     DWORD dwDesiredAccess,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPSTR lpBinaryPathName,
//     LPSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 45 */
// DWORD
// WINAPI
// RCreateServiceWOW64W(
//     handle_t BindingHandle,
//     LPWSTR lpServiceName,
//     LPWSTR lpDisplayName,
//     DWORD dwDesiredAccess,
//     DWORD dwServiceType,
//     DWORD dwStartType,
//     DWORD dwErrorControl,
//     LPWSTR lpBinaryPathName,
//     LPWSTR lpLoadOrderGroup,
//     LPDWORD lpdwTagId,
//     LPBYTE lpDependencies,
//     DWORD dwDependSize,
//     LPWSTR lpServiceStartName,
//     LPBYTE lpPassword,
//     DWORD dwPwSize,
//     LPSC_RPC_HANDLE lpServiceHandle)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 46 */
// DWORD
// WINAPI
// RI_ScQueryServiceTagInfo(
//     SC_RPC_HANDLE hSCManager,
//     TAG_INFO_LEVEL dwInfoLevel,
//     PTAG_INFO_NAME_FROM_TAG_IN_PARAMS * lpInParams,
//     PTAG_INFO_NAME_FROM_TAG_OUT_PARAMS * lpOutParams)
// {
//     PMANAGER_HANDLE hManager;
//
//     /* Validate handle */
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* FIXME: should check whether client is local */
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_ENUMERATE_SERVICE))
//     {
//         return ERROR_ACCESS_DENIED;
//     }
//
//     /* Check parameters */
//     if (lpInParams == NULL || lpOutParams == NULL)
//     {
//         return ERROR_INVALID_PARAMETER;
//     }
//
//     /* Check info level */
//     if (dwInfoLevel != TagInfoLevelNameFromTag)
//     {
//         return ERROR_RETRY;
//     }
//
//     /* Call internal helper */
//     return ScmGetServiceNameFromTag(*lpInParams, lpOutParams);
// }
//
//
// /* Function 47 */
// DWORD
// WINAPI
// RNotifyServiceStatusChange(
//     SC_RPC_HANDLE hService,
//     SC_RPC_NOTIFY_PARAMS NotifyParams,
//     GUID *pClientProcessGuid,
//     GUID *pSCMProcessGuid,
//     PBOOL pfCreateRemoteQueue,
//     LPSC_NOTIFY_RPC_HANDLE phNotify)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 48 */
// DWORD
// WINAPI
// RGetNotifyResults(
//     SC_NOTIFY_RPC_HANDLE hNotify,
//     PSC_RPC_NOTIFY_PARAMS_LIST *ppNotifyParams)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 49 */
// DWORD
// WINAPI
// RCloseNotifyHandle(
//     LPSC_NOTIFY_RPC_HANDLE phNotify,
//     PBOOL pfApcFired)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 50 */
// DWORD
// WINAPI
// RControlServiceExA(
//     SC_RPC_HANDLE hService,
//     DWORD dwControl,
//     DWORD dwInfoLevel)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 51 */
// DWORD
// WINAPI
// RControlServiceExW(
//     SC_RPC_HANDLE hService,
//     DWORD dwControl,
//     DWORD dwInfoLevel)
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 52 */
// DWORD
// WINAPI
// RI_ScSendPnPMessage(
//      RPC_SERVICE_STATUS_HANDLE hServiceStatus,
//      DWORD dwControl,
//      DWORD dwEventType,
//      DWORD dwEventSize,
//      LPBYTE pEventData)
// {
//     PSCM_CONTROL_PACKET ControlPacket;
//     DWORD PacketSize;
//     PSERVICE pService;
//     ULONG_PTR Ptr;
//     DWORD dwControlsAccepted, dwCurrentState;
//     DWORD dwError = ERROR_SUCCESS;
//
//     DPRINT("RI_ScSendPnPMessage(%p %lx %lu %lu %p)\n",
//            hServiceStatus, dwControl, dwEventType, dwEventSize, pEventData);
//
//     /* FIXME: Verify the status handle */
//     pService = (PSERVICE)hServiceStatus;
//
//     /* Fail, if the service is a driver */
//     if (pService->Status.dwServiceType & SERVICE_DRIVER)
//         return ERROR_INVALID_SERVICE_CONTROL;
//
//     dwControlsAccepted = pService->Status.dwControlsAccepted;
//     dwCurrentState = pService->Status.dwCurrentState;
//
//     /* Return ERROR_SERVICE_NOT_ACTIVE if the service has not been started */
//     if (pService->lpImage == NULL || dwCurrentState == SERVICE_STOPPED)
//         return ERROR_SERVICE_NOT_ACTIVE;
//
//     /* The service cannot accept a control code if it is not running */
//     if (dwCurrentState != SERVICE_RUNNING)
//         return ERROR_SERVICE_CANNOT_ACCEPT_CTRL;
//
//     /* Check if the control code is acceptable to the service */
//     switch (dwControl)
//     {
//         case SERVICE_CONTROL_DEVICEEVENT:
//             break;
//
//         case SERVICE_CONTROL_HARDWAREPROFILECHANGE:
//             if ((dwControlsAccepted & SERVICE_ACCEPT_HARDWAREPROFILECHANGE) == 0)
//                 return ERROR_INVALID_SERVICE_CONTROL;
//             break;
//
//         case SERVICE_CONTROL_POWEREVENT:
//             if ((dwControlsAccepted & SERVICE_ACCEPT_POWEREVENT) == 0)
//                 return ERROR_INVALID_SERVICE_CONTROL;
//             break;
//
//         case SERVICE_CONTROL_SESSIONCHANGE:
//             if ((dwControlsAccepted & SERVICE_ACCEPT_SESSIONCHANGE) == 0)
//                 return ERROR_INVALID_SERVICE_CONTROL;
//             break;
//
//         default:
//             return ERROR_INVALID_SERVICE_CONTROL;
//     }
//
//     /* Calculate the total size of the control packet:
//      * initial structure, event type and event data */
//     PacketSize = sizeof(SCM_CONTROL_PACKET) + sizeof(DWORD) + dwEventSize;
//
//     /* Allocate the control packet */
//     ControlPacket = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, PacketSize);
//     if (!ControlPacket)
//         return ERROR_NOT_ENOUGH_MEMORY;
//
//     ControlPacket->dwSize = PacketSize;
//     ControlPacket->dwControl = dwControl;
//
//     /* Copy the event type and event data into the control packet */
//     Ptr = ((ULONG_PTR)ControlPacket + sizeof(SCM_CONTROL_PACKET));
//     RtlCopyMemory((PULONG)Ptr, &dwEventType, sizeof(dwEventType));
//     if (dwEventSize != 0)
//     {
//         Ptr = Ptr + sizeof(dwEventType);
//         RtlCopyMemory((PULONG)Ptr, pEventData, dwEventSize);
//     }
//
//     /* Send the control packet */
//     dwError = ScmSendControlPacket(pService->lpImage->hControlPipe,
//                                    pService->lpServiceName,
//                                    dwControl,
//                                    PacketSize,
//                                    ControlPacket);
//
//     /* Free the control packet */
//     HeapFree(GetProcessHeap(), 0, ControlPacket);
//
//     return dwError;
// }
//
//
// /* Function 53 */
// DWORD
// WINAPI
// RI_ScValidatePnPService(
//      SC_RPC_HANDLE hSCManager,
//      LPWSTR pszServiceName,
//      RPC_SERVICE_STATUS_HANDLE *phServiceStatus)
// {
//     PMANAGER_HANDLE hManager;
//     PSERVICE pService;
//
//     DPRINT("RI_ScValidatePnPService(%p %S %p)\n", hSCManager, pszServiceName, phServiceStatus);
//
//     /* Validate handle */
//     hManager = ScmGetServiceManagerFromHandle(hSCManager);
//     if (hManager == NULL)
//     {
//         DPRINT1("Invalid handle\n");
//         return ERROR_INVALID_HANDLE;
//     }
//
//     /* FIXME: should check whether client is local */
//
//     /* Check access rights */
//     if (!RtlAreAllAccessesGranted(hManager->Handle.DesiredAccess,
//                                   SC_MANAGER_CONNECT))
//     {
//         DPRINT1("No SC_MANAGER_CONNECT access\n");
//         return ERROR_ACCESS_DENIED;
//     }
//
//     pService = ScmGetServiceEntryByName(pszServiceName);
//     DPRINT("pService: %p\n", pService);
//     if (pService == NULL)
//         return ERROR_SERVICE_DOES_NOT_EXIST;
//
//     *phServiceStatus = (RPC_SERVICE_STATUS_HANDLE)pService;
//
//     return ERROR_SUCCESS;
// }
//
//
// /* Function 54 */
// DWORD
// WINAPI
// ROpenServiceStatusHandle(
//     handle_t BindingHandle)  /* FIXME */
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 55 */
// DWORD
// WINAPI
// RFunction55(
//     handle_t BindingHandle)  /* FIXME */
// {
//     UNIMPLEMENTED;
//     return ERROR_CALL_NOT_IMPLEMENTED;
// }
//
//
// void __RPC_FAR * __RPC_USER MIDL_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER MIDL_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// void __RPC_USER SC_RPC_HANDLE_rundown(SC_RPC_HANDLE hSCObject)
// {
//     /* Close the handle */
//     RCloseServiceHandle(&hSCObject);
// }
//
//
// void __RPC_USER SC_RPC_LOCK_rundown(SC_RPC_LOCK Lock)
// {
//     /* Unlock the database */
//     RUnlockServiceDatabase(&Lock);
// }
//
//
// void __RPC_USER SC_NOTIFY_RPC_HANDLE_rundown(SC_NOTIFY_RPC_HANDLE hNotify)
// {
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
