use core::ffi::c_void;
use ntdll_sys::nt::*;

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


// RtlCopyMemory is a memmove macro (C). Re-translate as memcpy.
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// InitializeObjectAttributes is a C macro, not a function — rustconv
// emitted an extern decl. Inline equivalent.
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PSECURITY_DESCRIPTOR,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

// RtlInitEmptyUnicodeString is a C macro — inline equivalent.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(s: *mut _UNICODE_STRING, buf: *mut u16, len: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = len;
    (*s).Buffer = buf;
}

#[no_mangle]
pub static mut BaseSrvDllInstance: HANDLE = core::ptr::null_mut();

extern "C" {
    static mut BaseSrvKernel32DllPath: _UNICODE_STRING;
}

#[no_mangle]
pub static mut BaseSrvHeap: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut BaseSrvSharedHeap: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut BaseStaticServerData: PBASE_STATIC_SERVER_DATA = core::ptr::null_mut();

// Exported so the C dosdev.c (still cohabited) can read them via
// `extern ULONG SessionId, ProtectionMode;` references.
#[no_mangle]
pub static mut SessionId: ULONG = 0;
#[no_mangle]
pub static mut ProtectionMode: ULONG = 0;

static mut BaseSrvIniFileMapping: PINIFILE_MAPPING = core::ptr::null_mut();

// CSR_API_ROUTINE in C is NTAPI / stdcall on i386, so the BaseSrv*
// handlers are exported as `_BaseSrv*@8`. `extern "system"` selects
// the right convention per arch (stdcall on i386, cdecl elsewhere).
extern "system" {
    fn BaseSrvCreateProcess(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvCreateThread(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvGetTempFile(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvExitProcess(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvDebugProcess(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvCheckVDM(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvUpdateVDMEntry(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvGetNextVDMCommand(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvExitVDM(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvIsFirstVDM(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvGetVDMExitCode(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSetReenterCount(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSetProcessShutdownParam(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvGetProcessShutdownParam(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvNlsSetUserInfo(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvNlsSetMultipleUserInfo(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvNlsCreateSection(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSetVDMCurDirs(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvGetVDMCurDirs(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvBatNotification(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvRegisterWowExec(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSoundSentryNotification(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvRefreshIniFileMapping(ApiMessage: *mut c_void, ReplyMessage: *mut c_void)
        -> NTSTATUS;
    fn BaseSrvDefineDosDevice(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSetTermsrvAppInstallMode(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvNlsUpdateCacheCount(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvSetTermsrvClientTimeZone(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvSxsCreateActivationContext(
        ApiMessage: *mut c_void,
        ReplyMessage: *mut c_void,
    ) -> NTSTATUS;
    fn BaseSrvRegisterThread(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
    fn BaseSrvNlsGetUserInfo(ApiMessage: *mut c_void, ReplyMessage: *mut c_void) -> NTSTATUS;
}

type PCSR_API_ROUTINE = Option<unsafe extern "system" fn(*mut c_void, *mut c_void) -> NTSTATUS>;

#[no_mangle]
pub static mut BaseServerApiDispatchTable: [PCSR_API_ROUTINE; 31] = [
    Some(BaseSrvCreateProcess),
    Some(BaseSrvCreateThread),
    Some(BaseSrvGetTempFile),
    Some(BaseSrvExitProcess),
    Some(BaseSrvDebugProcess),
    Some(BaseSrvCheckVDM),
    Some(BaseSrvUpdateVDMEntry),
    Some(BaseSrvGetNextVDMCommand),
    Some(BaseSrvExitVDM),
    Some(BaseSrvIsFirstVDM),
    Some(BaseSrvGetVDMExitCode),
    Some(BaseSrvSetReenterCount),
    Some(BaseSrvSetProcessShutdownParam),
    Some(BaseSrvGetProcessShutdownParam),
    Some(BaseSrvNlsSetUserInfo),
    Some(BaseSrvNlsSetMultipleUserInfo),
    Some(BaseSrvNlsCreateSection),
    Some(BaseSrvSetVDMCurDirs),
    Some(BaseSrvGetVDMCurDirs),
    Some(BaseSrvBatNotification),
    Some(BaseSrvRegisterWowExec),
    Some(BaseSrvSoundSentryNotification),
    Some(BaseSrvRefreshIniFileMapping),
    Some(BaseSrvDefineDosDevice),
    Some(BaseSrvSetTermsrvAppInstallMode),
    Some(BaseSrvNlsUpdateCacheCount),
    Some(BaseSrvSetTermsrvClientTimeZone),
    Some(BaseSrvSxsCreateActivationContext),
    Some(BaseSrvDebugProcess),
    Some(BaseSrvRegisterThread),
    Some(BaseSrvNlsGetUserInfo),
];

#[no_mangle]
pub static BaseServerApiServerValidTable: [BOOLEAN; 31] = [
    1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1,
];

// Always defined — project's -DDBG=1 build enables CSR_DBG, so the
// NameTable slot in CSR_SERVER_DLL is real and csrsrv may read it.
// `static mut` because raw pointers aren't Sync; this table is read-only
// at runtime so the `mut` is a Rust hoop, not a hazard.
#[no_mangle]
pub static mut BaseServerApiNameTable: [*const i8; 31] = [
    b"BaseCreateProcess\0".as_ptr() as *const i8,
    b"BaseCreateThread\0".as_ptr() as *const i8,
    b"BaseGetTempFile\0".as_ptr() as *const i8,
    b"BaseExitProcess\0".as_ptr() as *const i8,
    b"BaseDebugProcess\0".as_ptr() as *const i8,
    b"BaseCheckVDM\0".as_ptr() as *const i8,
    b"BaseUpdateVDMEntry\0".as_ptr() as *const i8,
    b"BaseGetNextVDMCommand\0".as_ptr() as *const i8,
    b"BaseExitVDM\0".as_ptr() as *const i8,
    b"BaseIsFirstVDM\0".as_ptr() as *const i8,
    b"BaseGetVDMExitCode\0".as_ptr() as *const i8,
    b"BaseSetReenterCount\0".as_ptr() as *const i8,
    b"BaseSetProcessShutdownParam\0".as_ptr() as *const i8,
    b"BaseGetProcessShutdownParam\0".as_ptr() as *const i8,
    b"BaseNlsSetUserInfo\0".as_ptr() as *const i8,
    b"BaseNlsSetMultipleUserInfo\0".as_ptr() as *const i8,
    b"BaseNlsCreateSection\0".as_ptr() as *const i8,
    b"BaseSetVDMCurDirs\0".as_ptr() as *const i8,
    b"BaseGetVDMCurDirs\0".as_ptr() as *const i8,
    b"BaseBatNotification\0".as_ptr() as *const i8,
    b"BaseRegisterWowExec\0".as_ptr() as *const i8,
    b"BaseSoundSentryNotification\0".as_ptr() as *const i8,
    b"BaseRefreshIniFileMapping\0".as_ptr() as *const i8,
    b"BaseDefineDosDevice\0".as_ptr() as *const i8,
    b"BaseSetTermsrvAppInstallMode\0".as_ptr() as *const i8,
    b"BaseNlsUpdateCacheCount\0".as_ptr() as *const i8,
    b"BaseSetTermsrvClientTimeZone\0".as_ptr() as *const i8,
    b"BaseSxsCreateActivationContext\0".as_ptr() as *const i8,
    b"BaseSrvDebugProcessStop\0".as_ptr() as *const i8,
    b"BaseRegisterThread\0".as_ptr() as *const i8,
    b"BaseNlsGetUserInfo\0".as_ptr() as *const i8,
];

type PBASE_STATIC_SERVER_DATA = *mut BASE_STATIC_SERVER_DATA;
type PINIFILE_MAPPING = *mut INIFILE_MAPPING;
type PCSR_SERVER_DLL = *mut c_void;
type PCSR_PROCESS = *mut c_void;
type PBASESRV_API_CONNECTINFO = *mut c_void;

// Layout MUST match C `BASE_STATIC_SERVER_DATA` (sdk/include/reactos/subsys/
// win/base.h) field-for-field: basesrv.dll writes this struct into the shared
// section and kernel32 (+ ~7 user DLLs) read it back. The prior Rust mirror had
// a scrambled field order and opaque blobs → every BaseStaticServerData->X read
// from kernel32 got garbage and boot hung ~7 DLLs after kernel32 DllMain. Using
// the real bindgen nested types in exact C order makes repr(C) reproduce the C
// layout per-arch; the size assert below pins it to the C C_ASSERT (0x1AC8).
#[repr(C)]
struct BASE_STATIC_SERVER_DATA {
    WindowsDirectory: _UNICODE_STRING,
    WindowsSystemDirectory: _UNICODE_STRING,
    NamedObjectDirectory: _UNICODE_STRING,
    WindowsMajorVersion: USHORT,
    WindowsMinorVersion: USHORT,
    BuildNumber: USHORT,
    CSDNumber: USHORT,
    RCNumber: USHORT,
    CSDVersion: [WCHAR; 128],
    SysInfo: SYSTEM_BASIC_INFORMATION,
    TimeOfDay: SYSTEM_TIMEOFDAY_INFORMATION,
    IniFileMapping: PINIFILE_MAPPING, // C: PVOID — same layout
    NlsUserInfo: NLS_USER_INFO,
    DefaultSeparateVDM: BOOLEAN,
    IsWowTaskReady: BOOLEAN,
    WindowsSys32x86Directory: _UNICODE_STRING,
    fTermsrvAppInstallMode: BOOLEAN,
    tziTermsrvClientTimeZone: TIME_ZONE_INFORMATION,
    ktTermsrvClientBias: KSYSTEM_TIME,
    TermsrvClientTimeZoneId: ULONG,
    LUIDDeviceMapsEnabled: BOOLEAN,
    TermsrvClientTimeZoneChangeNum: ULONG,
}
// C pins the i386 size (base.h: `#if defined(_M_IX86) C_ASSERT(==0x1AC8)`).
// 64-bit has a different size (pointer-width fields); there the layout is
// guaranteed by repr(C) + the size-verified ntdll-sys nested types.
#[cfg(target_arch = "x86")]
const _: () = assert!(core::mem::size_of::<BASE_STATIC_SERVER_DATA>() == 0x1AC8);

// NtCurrentPeb is a TEB-inline (ntdll-sys::inlines), NOT an extern — the
// rustconv translation fabricated an extern decl that shadowed the real
// inline and broke field access on the returned PPEB. Use the inline.
// Data import is convention-free; the cdecl `extern "C"` is fine.
extern "C" {
    // BaseSrvDDDBSMCritSec lives in basesrv/dosdev.c (not yet translated) —
    // a real RTL_CRITICAL_SECTION export. Forward-declare with the right type
    // so callers see a typed reference.
    static mut BaseSrvDDDBSMCritSec: _RTL_CRITICAL_SECTION;
}

// Per-fn ABI matches the C declarations:
//   nls.c BaseSrvNLSInit / BaseSrvNlsConnect      — NTAPI = stdcall ("system")
//   vdm.c BaseSrvCleanupVDMResources              — plain VOID  = cdecl ("C")
//   dosdev.c BaseInitDefineDosDevice /
//            BaseCleanupDefineDosDevice           — plain VOID  = cdecl ("C")
//   vdm.c BaseInitializeVDM                       — plain VOID  = cdecl ("C")
extern "system" {
    // task #18: pass the correctly-offset NlsUserInfo pointer, not the whole
    // struct — nls.rs's BASE_STATIC_SERVER_DATA mirror had NlsUserInfo @0.
    fn BaseSrvNLSInit(pNlsUserInfo: PVOID);
    fn BaseSrvNlsConnect(
        CsrProcess: PCSR_PROCESS,
        ConnectionInfo: PVOID,
        ConnectionInfoLength: *mut ULONG,
    ) -> NTSTATUS;
}
extern "C" {
    fn BaseSrvCleanupVDMResources(CsrProcess: PCSR_PROCESS);
    fn BaseInitDefineDosDevice();
    fn BaseInitializeVDM();
    fn BaseCleanupDefineDosDevice();
}

static SM_REG_KEY: [u16; 44] = [
    b'\\' as u16,
    b'R' as u16,
    b'e' as u16,
    b'g' as u16,
    b'i' as u16,
    b's' as u16,
    b't' as u16,
    b'r' as u16,
    b'y' as u16,
    b'\\' as u16,
    b'M' as u16,
    b'a' as u16,
    b'c' as u16,
    b'h' as u16,
    b'i' as u16,
    b'n' as u16,
    b'e' as u16,
    b'\\' as u16,
    b'S' as u16,
    b'y' as u16,
    b's' as u16,
    b't' as u16,
    b'e' as u16,
    b'm' as u16,
    b'\\' as u16,
    b'C' as u16,
    b'u' as u16,
    b'r' as u16,
    b'r' as u16,
    b'e' as u16,
    b'n' as u16,
    b't' as u16,
    b'C' as u16,
    b'o' as u16,
    b'n' as u16,
    b't' as u16,
    b'r' as u16,
    b'o' as u16,
    b'l' as u16,
    b'S' as u16,
    b'e' as u16,
    b't' as u16,
    b'\\' as u16,
    0,
];

const HEAP_ZERO_MEMORY: ULONG = 0x00000008;
const TIME_ZONE_ID_INVALID: ULONG = 0xFFFFFFFF;
const MAX_PATH: usize = 260;
const UNICODE_NULL: u16 = 0;
const SECURITY_NT_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 5],
};
const SECURITY_WORLD_SID_AUTHORITY: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
    Value: [0, 0, 0, 0, 0, 1],
};
const SECURITY_LOCAL_SYSTEM_RID: ULONG = 0x00000012;
const SECURITY_WORLD_RID: ULONG = 0x00000000;
const SECURITY_RESTRICTED_CODE_RID: ULONG = 0x0000000C;
const ACL_REVISION2: ULONG = 2;
const SECURITY_DESCRIPTOR_REVISION: ULONG = 1;
const DIRECTORY_QUERY: ACCESS_MASK = 0x0001;
const DIRECTORY_TRAVERSE: ACCESS_MASK = 0x0002;
const DIRECTORY_CREATE_OBJECT: ACCESS_MASK = 0x0004;
const DIRECTORY_CREATE_SUBDIRECTORY: ACCESS_MASK = 0x0008;
const DIRECTORY_ALL_ACCESS: ACCESS_MASK = 0x000F000F; // STANDARD_RIGHTS_REQUIRED | 0xF (ndk/obtypes.h)
const READ_CONTROL: ACCESS_MASK = 0x00020000;
const SYMBOLIC_LINK_ALL_ACCESS: ACCESS_MASK = 0x000F0001;
const KEY_READ: ACCESS_MASK = 0x00020019;
const OBJ_OPENIF: ULONG = 0x00000080;
const OBJ_PERMANENT: ULONG = 0x00000010;
const REG_DWORD: ULONG = 4;
const RTL_QUERY_REGISTRY_DIRECT: ULONG = 0x00000020;
const REG_NONE: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn BaseSrvInitializeIniFileMappings(
    StaticServerData: PBASE_STATIC_SERVER_DATA,
) -> NTSTATUS {
    BaseSrvIniFileMapping = RtlAllocateHeap(
        BaseSrvSharedHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<INIFILE_MAPPING>().try_into().unwrap(),
    ) as PINIFILE_MAPPING;
    if BaseSrvIniFileMapping.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*StaticServerData).IniFileMapping = BaseSrvIniFileMapping;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CreateBaseAcls(
    Dacl: *mut PACL,
    RestrictedDacl: *mut PACL,
) -> NTSTATUS {
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut WorldSid: PSID = core::ptr::null_mut();
    let mut RestrictedSid: PSID = core::ptr::null_mut();
    let NtAuthority = SECURITY_NT_AUTHORITY;
    let WorldAuthority = SECURITY_WORLD_SID_AUTHORITY;
    let mut Status: NTSTATUS;
    let mut KeyValueBuffer: [u8; 0x40] = [0; 0x40];
    let KeyValuePartialInfo: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let AclLength: ULONG;
    let mut ResultLength: ULONG = 0;
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectSecurityMode: ULONG = 0;
    let WorldAccess: ACCESS_MASK;
    let RestrictedAccess: ACCESS_MASK;

    RtlInitUnicodeString(&mut KeyName, SM_REG_KEY.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(&mut hKey, KEY_READ, &mut ObjectAttributes);
    if Status >= 0 {
        static PROTECTION_MODE: [u16; 15] = [
            b'P' as u16,
            b'r' as u16,
            b'o' as u16,
            b't' as u16,
            b'e' as u16,
            b'c' as u16,
            b't' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            b'M' as u16,
            b'o' as u16,
            b'd' as u16,
            b'e' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut KeyName, PROTECTION_MODE.as_ptr());
        Status = NtQueryValueKey(
            hKey,
            &mut KeyName,
            KeyValuePartialInformation,
            KeyValueBuffer.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&KeyValueBuffer) as ULONG,
            &mut ResultLength,
        );

        KeyValuePartialInfo = KeyValueBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
        if Status >= 0
            && (*KeyValuePartialInfo).Type == REG_DWORD
            && *((*KeyValuePartialInfo).Data.as_ptr() as *const ULONG) != 0
        {
            ProtectionMode = *((*KeyValuePartialInfo).Data.as_ptr() as *const ULONG);
        }

        NtClose(hKey);
    }

    if SessionId == 0
        || NtQuerySystemInformation(
            SystemObjectSecurityMode,
            &mut ObjectSecurityMode as *mut ULONG as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
        ) < 0
    {
        ObjectSecurityMode = 0;
    }

    Status = RtlAllocateAndInitializeSid(
        &NtAuthority as *const _ as *mut _,
        1,
        SECURITY_LOCAL_SYSTEM_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut SystemSid,
    );
    if Status < 0 {
        return Status;
    }

    Status = RtlAllocateAndInitializeSid(
        &WorldAuthority as *const _ as *mut _,
        1,
        SECURITY_WORLD_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut WorldSid,
    );
    if Status < 0 {
        RtlFreeSid(SystemSid);
        return Status;
    }

    Status = RtlAllocateAndInitializeSid(
        &NtAuthority as *const _ as *mut _,
        1,
        SECURITY_RESTRICTED_CODE_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut RestrictedSid,
    );
    if Status < 0 {
        RtlFreeSid(WorldSid);
        RtlFreeSid(SystemSid);
        return Status;
    }

    AclLength = core::mem::size_of::<ACL>() as ULONG
        + 3 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as ULONG
        + RtlLengthSid(SystemSid)
        + RtlLengthSid(WorldSid)
        + RtlLengthSid(RestrictedSid);
    *Dacl = RtlAllocateHeap(BaseSrvHeap, 0, (AclLength as usize).try_into().unwrap()) as PACL;
    if (*Dacl).is_null() {
        Status = STATUS_NO_MEMORY;
        RtlFreeSid(RestrictedSid);
        RtlFreeSid(WorldSid);
        RtlFreeSid(SystemSid);
        return Status;
    }

    Status = RtlCreateAcl(*Dacl, AclLength, ACL_REVISION2);
    if Status < 0 {
        RtlFreeHeap(BaseSrvHeap, 0, *Dacl as PVOID);
        RtlFreeSid(RestrictedSid);
        RtlFreeSid(WorldSid);
        RtlFreeSid(SystemSid);
        return Status;
    }

    if ObjectSecurityMode != 0 {
        WorldAccess = DIRECTORY_QUERY | DIRECTORY_TRAVERSE;
    } else {
        WorldAccess = READ_CONTROL
            | DIRECTORY_QUERY
            | DIRECTORY_TRAVERSE
            | DIRECTORY_CREATE_OBJECT
            | DIRECTORY_CREATE_SUBDIRECTORY;
    }

    if RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, WorldAccess, WorldSid) >= 0
        && RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, DIRECTORY_ALL_ACCESS, SystemSid) >= 0
    {
        RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, DIRECTORY_TRAVERSE, RestrictedSid);
    }

    *RestrictedDacl =
        RtlAllocateHeap(BaseSrvHeap, 0, (AclLength as usize).try_into().unwrap()) as PACL;
    if (*RestrictedDacl).is_null() {
        Status = STATUS_NO_MEMORY;
        RtlFreeHeap(BaseSrvHeap, 0, *Dacl as PVOID);
        RtlFreeSid(RestrictedSid);
        RtlFreeSid(WorldSid);
        RtlFreeSid(SystemSid);
        return Status;
    }

    Status = RtlCreateAcl(*RestrictedDacl, AclLength, ACL_REVISION2);
    if Status < 0 {
        RtlFreeHeap(BaseSrvHeap, 0, *RestrictedDacl as PVOID);
        RtlFreeHeap(BaseSrvHeap, 0, *Dacl as PVOID);
        RtlFreeSid(RestrictedSid);
        RtlFreeSid(WorldSid);
        RtlFreeSid(SystemSid);
        return Status;
    }

    if SessionId == 0 || (ProtectionMode & 3) == 0 {
        RestrictedAccess = READ_CONTROL
            | DIRECTORY_QUERY
            | DIRECTORY_TRAVERSE
            | DIRECTORY_CREATE_OBJECT
            | DIRECTORY_CREATE_SUBDIRECTORY;
    } else {
        RestrictedAccess = READ_CONTROL | DIRECTORY_QUERY | DIRECTORY_TRAVERSE;
    }

    Status = RtlAddAccessAllowedAce(*RestrictedDacl, ACL_REVISION2, WorldAccess, WorldSid);
    if Status >= 0 {
        Status = RtlAddAccessAllowedAce(
            *RestrictedDacl,
            ACL_REVISION2,
            DIRECTORY_ALL_ACCESS,
            SystemSid,
        );
        if Status >= 0 {
            Status = RtlAddAccessAllowedAce(
                *RestrictedDacl,
                ACL_REVISION2,
                RestrictedAccess,
                RestrictedSid,
            );
        }
    }

    RtlFreeSid(RestrictedSid);
    RtlFreeSid(WorldSid);
    RtlFreeSid(SystemSid);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseInitializeStaticServerData(LoadedServerDll: PCSR_SERVER_DLL) {
    let mut Status: NTSTATUS;
    let mut Success: BOOLEAN;
    let mut BnoBuffer: [WCHAR; 100] = [0; 100];
    let mut Buffer: [WCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut HeapBuffer: *mut WCHAR;
    let mut SystemRootString: _UNICODE_STRING = core::mem::zeroed();
    static UNEXPANDED_SYSTEM_ROOT: [u16; 13] = [
        b'%' as u16,
        b'S' as u16,
        b'y' as u16,
        b's' as u16,
        b't' as u16,
        b'e' as u16,
        b'm' as u16,
        b'R' as u16,
        b'o' as u16,
        b'o' as u16,
        b't' as u16,
        b'%' as u16,
        0,
    ];
    let UnexpandedSystemRootString = _UNICODE_STRING {
        Length: 24,
        MaximumLength: 26,
        Buffer: UNEXPANDED_SYSTEM_ROOT.as_ptr() as *mut u16,
    };
    let mut BaseSrvCSDString: _UNICODE_STRING = core::mem::zeroed();
    let mut BaseSrvWindowsDirectory: _UNICODE_STRING = core::mem::zeroed();
    let mut BaseSrvWindowsSystemDirectory: _UNICODE_STRING = core::mem::zeroed();
    let mut BnoString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut BaseSrvNamedObjectDirectory: HANDLE = core::ptr::null_mut();
    let mut BaseSrvRestrictedObjectDirectory: HANDLE = core::ptr::null_mut();
    let mut BnoDacl: PACL = core::ptr::null_mut();
    let mut BnoRestrictedDacl: PACL = core::ptr::null_mut();
    let BnoSd: PSECURITY_DESCRIPTOR;
    let mut SymHandle: HANDLE = core::ptr::null_mut();
    let mut DirectoryName: _UNICODE_STRING = core::mem::zeroed();
    let mut SymlinkName: _UNICODE_STRING = core::mem::zeroed();
    let mut LuidEnabled: ULONG = 0;
    // MUST be a [u16] static (align 2), NOT a `b"..."` byte-string literal
    // (align 1). RtlQueryRegistryValues hands this to NtQueryValueKey as a
    // UNICODE_STRING value name; ProbeAndCaptureUnicodeString does
    // ProbeForRead(Buffer, Length, sizeof(WCHAR)) which raises
    // STATUS_DATATYPE_MISALIGNMENT on amd64 if the buffer is odd-addressed.
    // The old `w!("CSDVersion").as_ptr() as *mut u16` landed at an odd offset in
    // basesrv.dll on amd64 → BUGCHECK 0x1E. (i386/arm64 only got lucky.)
    static CSDVERSION_NAME: [u16; 11] = [
        b'C' as u16,
        b'S' as u16,
        b'D' as u16,
        b'V' as u16,
        b'e' as u16,
        b'r' as u16,
        b's' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        0,
    ];
    let mut BaseServerRegistryConfigurationTable: [RTL_QUERY_REGISTRY_TABLE; 2] = [
        RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: RTL_QUERY_REGISTRY_DIRECT,
            Name: CSDVERSION_NAME.as_ptr() as *mut u16,
            EntryContext: &mut BaseSrvCSDString as *mut _UNICODE_STRING as PVOID,
            DefaultType: REG_NONE,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        core::mem::zeroed(),
    ];

    // C: BaseSrvHeap = RtlGetProcessHeap()           (TEB inline)
    // C: BaseSrvSharedHeap = LoadedServerDll->SharedSection
    // The earlier rustconv translation invented a fake CSR_SERVER_DLL_STRUCT
    // with ProcessHeap at offset 0 — that read NULL (the C struct has
    // `Length` there) and zeroed BaseSrvHeap. See task #18 / memory
    // feedback_cohab_flip_runtime_traps.
    BaseSrvHeap = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).ProcessHeap;
    BaseSrvSharedHeap = (*(LoadedServerDll as *const CSR_SERVER_DLL_FULL)).SharedSection;

    SessionId = (*(NtCurrentPeb())).SessionId;

    RtlInitEmptyUnicodeString(
        &mut SystemRootString,
        Buffer.as_mut_ptr(),
        (core::mem::size_of_val(&Buffer)) as USHORT,
    );
    Status = RtlExpandEnvironmentStrings_U(
        core::ptr::null_mut(),
        &UnexpandedSystemRootString as *const _ as *mut _,
        &mut SystemRootString,
        core::ptr::null_mut(),
    );

    Buffer[SystemRootString.Length as usize / 2] = UNICODE_NULL;
    Success = RtlCreateUnicodeString(&mut BaseSrvWindowsDirectory, SystemRootString.Buffer);

    let system32 = [
        b'\\' as u16,
        b'S' as u16,
        b'y' as u16,
        b's' as u16,
        b't' as u16,
        b'e' as u16,
        b'm' as u16,
        b'3' as u16,
        b'2' as u16,
        0,
    ];
    let mut i = (SystemRootString.Length / 2) as usize;
    for &ch in &system32 {
        Buffer[i] = ch;
        i += 1;
        if ch == 0 {
            break;
        }
    }
    Success = RtlCreateUnicodeString(&mut BaseSrvWindowsSystemDirectory, Buffer.as_ptr());

    let kernel32_dll = [
        b'\\' as u16,
        b'k' as u16,
        b'e' as u16,
        b'r' as u16,
        b'n' as u16,
        b'e' as u16,
        b'l' as u16,
        b'3' as u16,
        b'2' as u16,
        b'.' as u16,
        b'd' as u16,
        b'l' as u16,
        b'l' as u16,
        0,
    ];
    i = (SystemRootString.Length / 2) as usize + 9;
    for &ch in &kernel32_dll {
        Buffer[i] = ch;
        i += 1;
        if ch == 0 {
            break;
        }
    }
    Success = RtlCreateUnicodeString(&raw mut BaseSrvKernel32DllPath, Buffer.as_ptr());

    if SessionId != 0 {
        let sessions = [
            b'\\' as u16,
            b'S' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            b's' as u16,
            b'\\' as u16,
            0,
        ];
        for (i, &ch) in sessions.iter().enumerate() {
            BnoBuffer[i] = ch;
        }
        let mut idx = 10;
        let mut sid = SessionId;
        if sid == 0 {
            BnoBuffer[idx] = b'0' as u16;
            idx += 1;
        } else {
            let mut digits = [0u16; 10];
            let mut digit_count = 0;
            while sid > 0 {
                digits[digit_count] = (b'0' as u16) + (sid % 10) as u16;
                sid /= 10;
                digit_count += 1;
            }
            for i in (0..digit_count).rev() {
                BnoBuffer[idx] = digits[i];
                idx += 1;
            }
        }
        let bno = [
            b'\\' as u16,
            b'B' as u16,
            b'a' as u16,
            b's' as u16,
            b'e' as u16,
            b'N' as u16,
            b'a' as u16,
            b'm' as u16,
            b'e' as u16,
            b'd' as u16,
            b'O' as u16,
            b'b' as u16,
            b'j' as u16,
            b'e' as u16,
            b'c' as u16,
            b't' as u16,
            b's' as u16,
            0,
        ];
        for &ch in &bno {
            BnoBuffer[idx] = ch;
            idx += 1;
            if ch == 0 {
                break;
            }
        }
    } else {
        let bno = [
            b'\\' as u16,
            b'B' as u16,
            b'a' as u16,
            b's' as u16,
            b'e' as u16,
            b'N' as u16,
            b'a' as u16,
            b'm' as u16,
            b'e' as u16,
            b'd' as u16,
            b'O' as u16,
            b'b' as u16,
            b'j' as u16,
            b'e' as u16,
            b'c' as u16,
            b't' as u16,
            b's' as u16,
            0,
        ];
        for (i, &ch) in bno.iter().enumerate() {
            BnoBuffer[i] = ch;
        }
    }
    RtlInitUnicodeString(&mut BnoString, BnoBuffer.as_ptr());

    BaseStaticServerData = RtlAllocateHeap(
        BaseSrvSharedHeap,
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<BASE_STATIC_SERVER_DATA>()
            .try_into()
            .unwrap(),
    ) as PBASE_STATIC_SERVER_DATA;

    (*BaseStaticServerData).TermsrvClientTimeZoneId = TIME_ZONE_ID_INVALID;
    (*BaseStaticServerData).TermsrvClientTimeZoneChangeNum = 0;
    Status = NtQuerySystemInformation(
        SystemTimeOfDayInformation,
        core::ptr::addr_of_mut!((*BaseStaticServerData).TimeOfDay) as PVOID,
        core::mem::size_of_val(&(*BaseStaticServerData).TimeOfDay) as ULONG,
        core::ptr::null_mut(),
    );

    (*BaseStaticServerData).WindowsDirectory = BaseSrvWindowsDirectory;
    HeapBuffer = RtlAllocateHeap(
        BaseSrvSharedHeap,
        0,
        (BaseSrvWindowsDirectory.MaximumLength as usize)
            .try_into()
            .unwrap(),
    ) as *mut WCHAR;
    RtlCopyMemory(
        HeapBuffer as PVOID,
        (*BaseStaticServerData).WindowsDirectory.Buffer as PVOID,
        BaseSrvWindowsDirectory.MaximumLength as usize,
    );
    (*BaseStaticServerData).WindowsDirectory.Buffer = HeapBuffer;

    (*BaseStaticServerData).WindowsSystemDirectory = BaseSrvWindowsSystemDirectory;
    HeapBuffer = RtlAllocateHeap(
        BaseSrvSharedHeap,
        0,
        (BaseSrvWindowsSystemDirectory.MaximumLength as usize)
            .try_into()
            .unwrap(),
    ) as *mut WCHAR;
    RtlCopyMemory(
        HeapBuffer as PVOID,
        (*BaseStaticServerData).WindowsSystemDirectory.Buffer as PVOID,
        BaseSrvWindowsSystemDirectory.MaximumLength as usize,
    );
    (*BaseStaticServerData).WindowsSystemDirectory.Buffer = HeapBuffer;

    RtlInitEmptyUnicodeString(
        &mut (*BaseStaticServerData).WindowsSys32x86Directory,
        core::ptr::null_mut(),
        0,
    );

    (*BaseStaticServerData).NamedObjectDirectory = BnoString;
    (*BaseStaticServerData).NamedObjectDirectory.MaximumLength =
        BnoString.Length + core::mem::size_of::<WCHAR>() as USHORT;
    HeapBuffer = RtlAllocateHeap(
        BaseSrvSharedHeap,
        0,
        ((*BaseStaticServerData).NamedObjectDirectory.MaximumLength as usize)
            .try_into()
            .unwrap(),
    ) as *mut WCHAR;
    RtlCopyMemory(
        HeapBuffer as PVOID,
        (*BaseStaticServerData).NamedObjectDirectory.Buffer as PVOID,
        (*BaseStaticServerData).NamedObjectDirectory.MaximumLength as usize,
    );
    (*BaseStaticServerData).NamedObjectDirectory.Buffer = HeapBuffer;

    (*BaseStaticServerData).CSDNumber = 0;
    (*BaseStaticServerData).RCNumber = 0;

    RtlInitEmptyUnicodeString(
        &mut BaseSrvCSDString,
        Buffer.as_mut_ptr(),
        core::mem::size_of_val(&Buffer) as USHORT,
    );
    Status = RtlQueryRegistryValues(
        1,
        core::ptr::null_mut(),
        BaseServerRegistryConfigurationTable.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status >= 0 {
        let len = (BaseSrvCSDString.Length / 2) as usize;
        for i in 0..len.min(127) {
            (*BaseStaticServerData).CSDVersion[i] = Buffer[i];
        }
        (*BaseStaticServerData).CSDVersion[len.min(127)] = UNICODE_NULL;
    } else {
        BaseSrvCSDString.Length = 0;
        (*BaseStaticServerData).CSDVersion[0] = UNICODE_NULL;
    }

    Status = NtQuerySystemInformation(
        SystemBasicInformation,
        core::ptr::addr_of_mut!((*BaseStaticServerData).SysInfo) as PVOID,
        core::mem::size_of_val(&(*BaseStaticServerData).SysInfo) as ULONG,
        core::ptr::null_mut(),
    );

    Status = BaseSrvInitializeIniFileMappings(BaseStaticServerData);

    (*BaseStaticServerData).DefaultSeparateVDM = 0u8;
    (*BaseStaticServerData).IsWowTaskReady = 0u8;

    BnoSd = RtlAllocateHeap(BaseSrvHeap, 0, 1024) as PSECURITY_DESCRIPTOR;
    Status = RtlCreateSecurityDescriptor(BnoSd, SECURITY_DESCRIPTOR_REVISION);

    Status = CreateBaseAcls(&mut BnoDacl, &mut BnoRestrictedDacl);

    Status = RtlSetDaclSecurityDescriptor(BnoSd, 1u8, BnoDacl, 0u8);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut BnoString,
        OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        BnoSd,
    );
    Status = NtCreateDirectoryObject(
        &mut BaseSrvNamedObjectDirectory,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );

    if SessionId == 0 {
        Status = NtSetInformationObject(
            BaseSrvNamedObjectDirectory,
            _OBJECT_INFORMATION_CLASS_ObjectSessionInformation,
            core::ptr::null_mut(),
            0,
        );
    }

    Status = NtQueryInformationProcess(
        -1isize as HANDLE,
        ProcessLUIDDeviceMapsEnabled,
        &mut LuidEnabled as *mut ULONG as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
        core::ptr::null_mut(),
    );
    (*BaseStaticServerData).LUIDDeviceMapsEnabled = LuidEnabled as BOOLEAN;

    if !(*BaseStaticServerData).LUIDDeviceMapsEnabled != 0
        || RtlInitializeCriticalSectionAndSpinCount(
            core::ptr::addr_of_mut!(BaseSrvDDDBSMCritSec),
            0x80000000,
        ) >= 0
    {
        static GLOBAL: [u16; 7] = [
            b'G' as u16,
            b'l' as u16,
            b'o' as u16,
            b'b' as u16,
            b'a' as u16,
            b'l' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut DirectoryName, GLOBAL.as_ptr());
        static BNO_SYMLINK: [u16; 18] = [
            b'\\' as u16,
            b'B' as u16,
            b'a' as u16,
            b's' as u16,
            b'e' as u16,
            b'N' as u16,
            b'a' as u16,
            b'm' as u16,
            b'e' as u16,
            b'd' as u16,
            b'O' as u16,
            b'b' as u16,
            b'j' as u16,
            b'e' as u16,
            b'c' as u16,
            b't' as u16,
            b's' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut SymlinkName, BNO_SYMLINK.as_ptr());
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut DirectoryName,
            OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
            BaseSrvNamedObjectDirectory,
            BnoSd,
        );
        Status = NtCreateSymbolicLinkObject(
            &mut SymHandle,
            SYMBOLIC_LINK_ALL_ACCESS,
            &mut ObjectAttributes,
            &mut SymlinkName,
        );
        if Status >= 0 && SessionId == 0 {
            NtClose(SymHandle);
        }

        static LOCAL: [u16; 6] = [
            b'L' as u16,
            b'o' as u16,
            b'c' as u16,
            b'a' as u16,
            b'l' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut DirectoryName, LOCAL.as_ptr());
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut DirectoryName,
            OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
            BaseSrvNamedObjectDirectory,
            BnoSd,
        );
        Status = NtCreateSymbolicLinkObject(
            &mut SymHandle,
            SYMBOLIC_LINK_ALL_ACCESS,
            &mut ObjectAttributes,
            &mut BnoString,
        );
        if Status >= 0 && SessionId == 0 {
            NtClose(SymHandle);
        }

        static SESSION: [u16; 8] = [
            b'S' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut DirectoryName, SESSION.as_ptr());
        static BNOLINKS: [u16; 19] = [
            b'\\' as u16,
            b'S' as u16,
            b'e' as u16,
            b's' as u16,
            b's' as u16,
            b'i' as u16,
            b'o' as u16,
            b'n' as u16,
            b's' as u16,
            b'\\' as u16,
            b'B' as u16,
            b'N' as u16,
            b'O' as u16,
            b'L' as u16,
            b'I' as u16,
            b'N' as u16,
            b'K' as u16,
            b'S' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut SymlinkName, BNOLINKS.as_ptr());
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut DirectoryName,
            OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
            BaseSrvNamedObjectDirectory,
            BnoSd,
        );
        Status = NtCreateSymbolicLinkObject(
            &mut SymHandle,
            SYMBOLIC_LINK_ALL_ACCESS,
            &mut ObjectAttributes,
            &mut SymlinkName,
        );
        if Status >= 0 && SessionId == 0 {
            NtClose(SymHandle);
        }

        static RESTRICTED: [u16; 11] = [
            b'R' as u16,
            b'e' as u16,
            b's' as u16,
            b't' as u16,
            b'r' as u16,
            b'i' as u16,
            b'c' as u16,
            b't' as u16,
            b'e' as u16,
            b'd' as u16,
            0,
        ];
        RtlInitUnicodeString(&mut DirectoryName, RESTRICTED.as_ptr());
        Status = RtlSetDaclSecurityDescriptor(BnoSd, 1u8, BnoRestrictedDacl, 0u8);
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut DirectoryName,
            OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
            BaseSrvNamedObjectDirectory,
            BnoSd,
        );
        Status = NtCreateDirectoryObject(
            &mut BaseSrvRestrictedObjectDirectory,
            DIRECTORY_ALL_ACCESS,
            &mut ObjectAttributes,
        );
    }

    BaseSrvNLSInit(core::ptr::addr_of_mut!((*BaseStaticServerData).NlsUserInfo) as PVOID);

    // C: LoadedServerDll->SharedSection = BaseStaticServerData.
    // Use the real struct layout (CSR_SERVER_DLL_FULL) — the old hack
    // CSR_SERVER_DLL_STRUCT had SharedSection at offset 4, but C has
    // it at 0x3C (CSR_DBG path). Writing to the wrong offset left
    // CsrSrvSharedStaticServerData[BASESRV_INDEX] NULL, which made
    // kernel32 client/dllmain.c:188 fail ASSERT(BaseStaticServerData).
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).SharedSection = BaseStaticServerData as PVOID;
}

#[repr(C)]
struct PEB_STRUCT {
    Reserved1: [u8; 2],
    BeingDebugged: u8,
    Reserved2: [u8; 1],
    Reserved3: [PVOID; 2],
    Ldr: PVOID,
    ProcessParameters: PVOID,
    Reserved4: [PVOID; 3],
    AtlThunkSListPtr: PVOID,
    Reserved5: PVOID,
    Reserved6: u32,
    Reserved7: PVOID,
    Reserved8: u32,
    AtlThunkSListPtr32: u32,
    Reserved9: [PVOID; 45],
    Reserved10: [u8; 96],
    PostProcessInitRoutine: PVOID,
    Reserved11: [u8; 128],
    Reserved12: [PVOID; 1],
    SessionId: u32,
}

#[no_mangle]
pub unsafe extern "system" fn BaseClientConnectRoutine(
    CsrProcess: PCSR_PROCESS,
    ConnectionInfo: PVOID,
    ConnectionInfoLength: *mut ULONG,
) -> NTSTATUS {
    let _ConnectInfo = ConnectionInfo as PBASESRV_API_CONNECTINFO;

    if ConnectionInfo.is_null()
        || ConnectionInfoLength.is_null()
        || *ConnectionInfoLength != core::mem::size_of::<BASESRV_API_CONNECTINFO>() as ULONG
    {
        return STATUS_INVALID_PARAMETER;
    }

    BaseSrvNlsConnect(CsrProcess, ConnectionInfo, ConnectionInfoLength)
}

#[no_mangle]
pub unsafe extern "system" fn BaseClientDisconnectRoutine(CsrProcess: PCSR_PROCESS) {
    BaseSrvCleanupVDMResources(CsrProcess);
}

// C `_BASESRV_API_CONNECTINFO` (basemsg.h): `{ ULONG DebugFlags; }`, with
// C_ASSERT(sizeof == 0x04). BaseClientConnectRoutine rejects a connect whose
// `*ConnectionInfoLength != size_of::<BASESRV_API_CONNECTINFO>()`. The old
// `= c_void` made that size 0, so every real client (4-byte connect info)
// was rejected with STATUS_INVALID_PARAMETER. Same class as INIFILE_MAPPING.
#[repr(C)]
#[allow(dead_code)]
struct BASESRV_API_CONNECTINFO {
    DebugFlags: ULONG,
}
const _: () = assert!(core::mem::size_of::<BASESRV_API_CONNECTINFO>() == 0x04);
// INIFILE_MAPPING: 3 pointers + ULONG → 16 on i386, 32 on LLP64 (repr(C) pad).
#[cfg(target_arch = "x86")]
const _: () = assert!(core::mem::size_of::<INIFILE_MAPPING>() == 16);
// C `_INIFILE_MAPPING` (sdk/include/reactos/subsys/win/base.h): 3 list-head
// pointers + a ULONG. The old `type INIFILE_MAPPING = c_void` made
// `size_of::<INIFILE_MAPPING>()` 0, so BaseSrvInitializeIniFileMappings
// allocated a ~0-byte block — kernel32's GetPrivateProfile* (via CSR) then
// read FileNames/DefaultFileNameMapping/WinIniFileMapping from adjacent heap
// garbage and walked a bogus mapping list → csrss wedged at the first
// system.ini lookup (msacm32 MSACM_RegisterAllDrivers). Real struct → correct
// HEAP_ZERO_MEMORY size → all three heads NULL → empty mapping (matches C).
#[repr(C)]
#[allow(dead_code)]
struct INIFILE_MAPPING {
    FileNames: PVOID,              // PINIFILE_MAPPING_FILENAME
    DefaultFileNameMapping: PVOID, // PINIFILE_MAPPING_FILENAME
    WinIniFileMapping: PVOID,      // PINIFILE_MAPPING_FILENAME
    Reserved: ULONG,
}

extern "C" {
    fn DbgPrint(Format: *const i8, ...) -> ULONG;
}

#[cfg_attr(feature = "basesrv_dll_only", no_mangle)]
#[allow(dead_code)]
pub unsafe extern "system" fn ServerDllInitialization(
    LoadedServerDll: PCSR_SERVER_DLL,
) -> NTSTATUS {
    const BASESRV_FIRST_API_NUMBER: ULONG = 0;
    const BasepMaxApiNumber: ULONG = 31;

    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).ApiBase = BASESRV_FIRST_API_NUMBER;
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).HighestApiSupported = BasepMaxApiNumber;
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).DispatchTable =
        BaseServerApiDispatchTable.as_ptr() as *const c_void;
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).ValidTable =
        BaseServerApiServerValidTable.as_ptr();
    // NameTable is always present in the struct layout (CSR_DBG path,
    // which the project's -DDBG=1 build selects). Fill it; csrsrv may
    // read it for diagnostic purposes.
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).NameTable =
        BaseServerApiNameTable.as_ptr() as *const *const i8;
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).SizeOfProcessData = 0;
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).ConnectCallback =
        Some(BaseClientConnectRoutine);
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).DisconnectCallback =
        Some(BaseClientDisconnectRoutine);
    (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).ShutdownProcessCallback = None;

    BaseSrvDllInstance = (*(LoadedServerDll as *mut CSR_SERVER_DLL_FULL)).ServerHandle;

    BaseInitializeStaticServerData(LoadedServerDll);

    BaseInitDefineDosDevice();

    BaseInitializeVDM();

    STATUS_SUCCESS
}

// Mirrors the C `_CSR_SERVER_DLL` struct in csrsrv.h. Fields BEFORE
// ApiBase (Length, Name, ServerHandle, ServerId, Unknown) MUST be
// present in the right order — earlier rustconv translations omitted
// the preamble and corrupted CsrLoadedServerDll[] entries by writing
// ApiBase at offset 0 (where C has `Length`). The C_ASSERT at the
// bottom of csrsrv.h pins SharedSection at offset 0x38 (no CSR_DBG).
#[repr(C)]
struct ANSI_STRING_REPR {
    Length: USHORT,
    MaximumLength: USHORT,
    Buffer: *mut i8,
}

#[repr(C)]
struct CSR_SERVER_DLL_FULL {
    Length: ULONG,                // +0x00
    Name: ANSI_STRING_REPR,       // +0x04 (8 bytes on i386)
    ServerHandle: HANDLE,         // +0x0C
    ServerId: ULONG,              // +0x10
    Unknown: ULONG,               // +0x14
    ApiBase: ULONG,               // +0x18
    HighestApiSupported: ULONG,   // +0x1C
    DispatchTable: *const c_void, // +0x20
    ValidTable: *const BOOLEAN,   // +0x24
    // NameTable is `#ifdef CSR_DBG` in csrsrv.h, which is set when
    // the project builds with -DDBG=1 (the default debug build).
    // The C_ASSERT at the bottom of csrsrv.h confirms SharedSection
    // is at 0x3C when CSR_DBG is on (0x38 without). bindgen i386 also
    // includes NameTable unconditionally, matching the DBG=1 layout.
    NameTable: *const *const i8, // +0x28 (CSR_DBG branch)
    SizeOfProcessData: ULONG,    // +0x2C
    ConnectCallback: Option<unsafe extern "system" fn(PCSR_PROCESS, PVOID, *mut ULONG) -> NTSTATUS>, // +0x30
    DisconnectCallback: Option<unsafe extern "system" fn(PCSR_PROCESS)>, // +0x34
    HardErrorCallback: PVOID,                                            // +0x38
    SharedSection: PVOID,      // +0x3C  ← C_ASSERT (CSR_DBG) verifies this
    NewProcessCallback: PVOID, // +0x40
    ShutdownProcessCallback:
        Option<unsafe extern "system" fn(PCSR_PROCESS, ULONG, BOOLEAN) -> NTSTATUS>, // +0x44
    Unknown2: [ULONG; 3],
}

// basesrv.dll DllMain — gated to basesrv_dll_only feature so csrsrv's
// `#[no_mangle] DllMain` doesn't collide with this one in the
// default-features archive. The lean basesrv archive (--features
// basesrv_dll_only) excludes csrsrv, leaving this DllMain as the sole
// owner. Cohabitation cmake flip is reverted (task #18) until
// BASE_STATIC_SERVER_DATA layout is repaired.
#[cfg_attr(feature = "basesrv_dll_only", no_mangle)]
#[allow(dead_code)]
pub unsafe extern "system" fn DllMain(
    _hInstanceDll: HINSTANCE,
    dwReason: ULONG,
    _lpReserved: PVOID,
) -> BOOLEAN {
    const DLL_PROCESS_DETACH: ULONG = 0;

    if dwReason == DLL_PROCESS_DETACH {
        BaseCleanupDefineDosDevice();
    }

    1u8
}

type HINSTANCE = HANDLE;

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/init.c
//  * PURPOSE:         Initialization
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
// #include "vdm.h"
//
// #include <winreg.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #include "api.h"
//
// /* GLOBALS ********************************************************************/
//
// HANDLE BaseSrvDllInstance = NULL;
// extern UNICODE_STRING BaseSrvKernel32DllPath;
//
// /* Memory */
// HANDLE BaseSrvHeap = NULL;          // Our own heap.
// HANDLE BaseSrvSharedHeap = NULL;    // Shared heap with CSR. (CsrSrvSharedSectionHeap)
// PBASE_STATIC_SERVER_DATA BaseStaticServerData = NULL;   // Data that we can share amongst processes. Initialized inside BaseSrvSharedHeap.
//
// ULONG SessionId = 0;
// ULONG ProtectionMode = 0;
//
// PINIFILE_MAPPING BaseSrvIniFileMapping;
//
// // Windows Server 2003 table from http://j00ru.vexillium.org/csrss_list/api_list.html#Windows_2k3
// PCSR_API_ROUTINE BaseServerApiDispatchTable[BasepMaxApiNumber - BASESRV_FIRST_API_NUMBER] =
// {
//     BaseSrvCreateProcess,
//     BaseSrvCreateThread,
//     BaseSrvGetTempFile,
//     BaseSrvExitProcess,
//     BaseSrvDebugProcess,
//     BaseSrvCheckVDM,
//     BaseSrvUpdateVDMEntry,
//     BaseSrvGetNextVDMCommand,
//     BaseSrvExitVDM,
//     BaseSrvIsFirstVDM,
//     BaseSrvGetVDMExitCode,
//     BaseSrvSetReenterCount,
//     BaseSrvSetProcessShutdownParam,
//     BaseSrvGetProcessShutdownParam,
//     BaseSrvNlsSetUserInfo,
//     BaseSrvNlsSetMultipleUserInfo,
//     BaseSrvNlsCreateSection,
//     BaseSrvSetVDMCurDirs,
//     BaseSrvGetVDMCurDirs,
//     BaseSrvBatNotification,
//     BaseSrvRegisterWowExec,
//     BaseSrvSoundSentryNotification,
//     BaseSrvRefreshIniFileMapping,
//     BaseSrvDefineDosDevice,
//     BaseSrvSetTermsrvAppInstallMode,
//     BaseSrvNlsUpdateCacheCount,
//     BaseSrvSetTermsrvClientTimeZone,
//     BaseSrvSxsCreateActivationContext,
//     BaseSrvDebugProcess,
//     BaseSrvRegisterThread,
//     BaseSrvNlsGetUserInfo,
// };
//
// BOOLEAN BaseServerApiServerValidTable[BasepMaxApiNumber - BASESRV_FIRST_API_NUMBER] =
// {
//     TRUE,   // BaseSrvCreateProcess
//     TRUE,   // BaseSrvCreateThread
//     TRUE,   // BaseSrvGetTempFile
//     FALSE,  // BaseSrvExitProcess
//     FALSE,  // BaseSrvDebugProcess
//     TRUE,   // BaseSrvCheckVDM
//     TRUE,   // BaseSrvUpdateVDMEntry
//     TRUE,   // BaseSrvGetNextVDMCommand
//     TRUE,   // BaseSrvExitVDM
//     TRUE,   // BaseSrvIsFirstVDM
//     TRUE,   // BaseSrvGetVDMExitCode
//     TRUE,   // BaseSrvSetReenterCount
//     TRUE,   // BaseSrvSetProcessShutdownParam
//     TRUE,   // BaseSrvGetProcessShutdownParam
//     TRUE,   // BaseSrvNlsSetUserInfo
//     TRUE,   // BaseSrvNlsSetMultipleUserInfo
//     TRUE,   // BaseSrvNlsCreateSection
//     TRUE,   // BaseSrvSetVDMCurDirs
//     TRUE,   // BaseSrvGetVDMCurDirs
//     TRUE,   // BaseSrvBatNotification
//     TRUE,   // BaseSrvRegisterWowExec
//     TRUE,   // BaseSrvSoundSentryNotification
//     TRUE,   // BaseSrvRefreshIniFileMapping
//     TRUE,   // BaseSrvDefineDosDevice
//     TRUE,   // BaseSrvSetTermsrvAppInstallMode
//     TRUE,   // BaseSrvNlsUpdateCacheCount
//     TRUE,   // BaseSrvSetTermsrvClientTimeZone
//     TRUE,   // BaseSrvSxsCreateActivationContext
//     FALSE,  // BaseSrvDebugProcess
//     TRUE,   // BaseSrvRegisterThread
//     TRUE,   // BaseSrvNlsGetUserInfo
// };
//
// /*
//  * On Windows Server 2003, CSR Servers contain
//  * the API Names Table only in Debug Builds.
//  */
// #ifdef CSR_DBG
// PCHAR BaseServerApiNameTable[BasepMaxApiNumber - BASESRV_FIRST_API_NUMBER] =
// {
//     "BaseCreateProcess",
//     "BaseCreateThread",
//     "BaseGetTempFile",
//     "BaseExitProcess",
//     "BaseDebugProcess",
//     "BaseCheckVDM",
//     "BaseUpdateVDMEntry",
//     "BaseGetNextVDMCommand",
//     "BaseExitVDM",
//     "BaseIsFirstVDM",
//     "BaseGetVDMExitCode",
//     "BaseSetReenterCount",
//     "BaseSetProcessShutdownParam",
//     "BaseGetProcessShutdownParam",
//     "BaseNlsSetUserInfo",
//     "BaseNlsSetMultipleUserInfo",
//     "BaseNlsCreateSection",
//     "BaseSetVDMCurDirs",
//     "BaseGetVDMCurDirs",
//     "BaseBatNotification",
//     "BaseRegisterWowExec",
//     "BaseSoundSentryNotification",
//     "BaseRefreshIniFileMapping",
//     "BaseDefineDosDevice",
//     "BaseSetTermsrvAppInstallMode",
//     "BaseNlsUpdateCacheCount",
//     "BaseSetTermsrvClientTimeZone",
//     "BaseSxsCreateActivationContext",
//     "BaseSrvDebugProcessStop",
//     "BaseRegisterThread",
//     "BaseNlsGetUserInfo",
// };
// #endif
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// BaseSrvInitializeIniFileMappings(IN PBASE_STATIC_SERVER_DATA StaticServerData)
// {
//     /* Allocate the mapping blob */
//     BaseSrvIniFileMapping = RtlAllocateHeap(BaseSrvSharedHeap,
//                                             HEAP_ZERO_MEMORY,
//                                             sizeof(*BaseSrvIniFileMapping));
//     if (BaseSrvIniFileMapping == NULL)
//     {
//         DPRINT1("BASESRV: Unable to allocate memory in shared heap for IniFileMapping\n");
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Set it*/
//     StaticServerData->IniFileMapping = BaseSrvIniFileMapping;
//
//     /* FIXME: Do the work to initialize the mappings */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// CreateBaseAcls(OUT PACL* Dacl,
//                OUT PACL* RestrictedDacl)
// {
//     PSID SystemSid, WorldSid, RestrictedSid;
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     NTSTATUS Status;
//     UCHAR KeyValueBuffer[0x40];
//     PKEY_VALUE_PARTIAL_INFORMATION KeyValuePartialInfo;
//     UNICODE_STRING KeyName;
//     ULONG AclLength;
//     ULONG ResultLength;
//     HANDLE hKey;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     ULONG ObjectSecurityMode;
//     ACCESS_MASK WorldAccess, RestrictedAccess;
//
//     /* Open the Session Manager Key */
//     RtlInitUnicodeString(&KeyName, SM_REG_KEY);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&hKey, KEY_READ, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Read the key value */
//         RtlInitUnicodeString(&KeyName, L"ProtectionMode");
//         Status = NtQueryValueKey(hKey,
//                                  &KeyName,
//                                  KeyValuePartialInformation,
//                                  KeyValueBuffer,
//                                  sizeof(KeyValueBuffer),
//                                  &ResultLength);
//
//         /* Make sure it's what we expect it to be */
//         KeyValuePartialInfo = (PKEY_VALUE_PARTIAL_INFORMATION)KeyValueBuffer;
//         if ((NT_SUCCESS(Status)) && (KeyValuePartialInfo->Type == REG_DWORD) &&
//             (*(PULONG)KeyValuePartialInfo->Data))
//         {
//             /* Save the Protection Mode */
//             ProtectionMode = *(PULONG)KeyValuePartialInfo->Data;
//         }
//
//         /* Close the handle */
//         NtClose(hKey);
//     }
//
//     /* Get object security mode */
//     if (SessionId == 0 ||
//         !NT_SUCCESS(NtQuerySystemInformation(SystemObjectSecurityMode, &ObjectSecurityMode, sizeof(ULONG), NULL)))
//     {
//         ObjectSecurityMode = 0;
//     }
//
//     /* Allocate the System SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1, SECURITY_LOCAL_SYSTEM_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &SystemSid);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Allocate the World SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
//                                          1, SECURITY_WORLD_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &WorldSid);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeSid(SystemSid);
//         goto Return;
//     }
//
//     /* Allocate the restricted SID */
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          1, SECURITY_RESTRICTED_CODE_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &RestrictedSid);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeSid(WorldSid);
//         RtlFreeSid(SystemSid);
//         goto Return;
//     }
//
//     /* Allocate one ACL with 3 ACEs each for one SID */
//     AclLength = sizeof(ACL) + 3 * sizeof(ACCESS_ALLOWED_ACE) +
//                     RtlLengthSid(SystemSid) +
//                     RtlLengthSid(WorldSid)  +
//                     RtlLengthSid(RestrictedSid);
//     *Dacl = RtlAllocateHeap(BaseSrvHeap, 0, AclLength);
//     if (*Dacl == NULL)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto FreeAndReturn;
//     }
//
//     /* Set the correct header fields */
//     Status = RtlCreateAcl(*Dacl, AclLength, ACL_REVISION2);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(BaseSrvHeap, 0, *Dacl);
//         goto FreeAndReturn;
//     }
//
//     /* Setup access for anyone depending on object security mode */
//     if (ObjectSecurityMode != 0)
//     {
//         /*
//          * If we have restrictions on security mode, make it read only
//          * it also means session ID is not 0
//          */
//         WorldAccess = DIRECTORY_QUERY | DIRECTORY_TRAVERSE;
//     }
//     else
//     {
//         /* Otherwise, open wide */
//         WorldAccess = READ_CONTROL | DIRECTORY_QUERY | DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT | DIRECTORY_CREATE_SUBDIRECTORY;
//     }
//
//     /* Give the appropriate rights to each SID */
//     if (NT_SUCCESS(RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, WorldAccess, WorldSid)) &&
//         NT_SUCCESS(RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, DIRECTORY_ALL_ACCESS, SystemSid)))
//     {
//         RtlAddAccessAllowedAce(*Dacl, ACL_REVISION2, DIRECTORY_TRAVERSE, RestrictedSid);
//     }
//
//     /* Now allocate the restricted DACL */
//     *RestrictedDacl = RtlAllocateHeap(BaseSrvHeap, 0, AclLength);
//     if (*RestrictedDacl == NULL)
//     {
//         Status = STATUS_NO_MEMORY;
//         RtlFreeHeap(BaseSrvHeap, 0, *Dacl);
//         goto FreeAndReturn;
//     }
//
//     /* Initialize it */
//     Status = RtlCreateAcl(*RestrictedDacl, AclLength, ACL_REVISION2);
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(BaseSrvHeap, 0, *RestrictedDacl);
//         RtlFreeHeap(BaseSrvHeap, 0, *Dacl);
//         goto FreeAndReturn;
//     }
//
//     /* Setup access for restricted sid depending on session id and protection mode */
//     if (SessionId == 0 || (ProtectionMode & 3) == 0)
//     {
//         /* If we have no session ID or if protection mode is not set, then open wide */
//         RestrictedAccess = READ_CONTROL | DIRECTORY_QUERY | DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT | DIRECTORY_CREATE_SUBDIRECTORY;
//     }
//     else
//     {
//         /* Otherwise, make read only */
//         RestrictedAccess = READ_CONTROL | DIRECTORY_QUERY | DIRECTORY_TRAVERSE;
//     }
//
//     /* And add the same ACEs as before */
//     Status = RtlAddAccessAllowedAce(*RestrictedDacl, ACL_REVISION2, WorldAccess, WorldSid);
//     if (NT_SUCCESS(Status))
//     {
//         Status = RtlAddAccessAllowedAce(*RestrictedDacl, ACL_REVISION2, DIRECTORY_ALL_ACCESS, SystemSid);
//         if (NT_SUCCESS(Status))
//         {
//             Status = RtlAddAccessAllowedAce(*RestrictedDacl, ACL_REVISION2, RestrictedAccess, RestrictedSid);
//         }
//     }
//
//     /* The SIDs are captured, can free them now */
// FreeAndReturn:
//     RtlFreeSid(RestrictedSid);
//     RtlFreeSid(WorldSid);
//     RtlFreeSid(SystemSid);
//
// Return:
//     return Status;
// }
//
// VOID
// NTAPI
// BaseInitializeStaticServerData(IN PCSR_SERVER_DLL LoadedServerDll)
// {
//     NTSTATUS Status;
//     BOOLEAN Success;
//     WCHAR BnoBuffer[100];
//     WCHAR Buffer[MAX_PATH];
//     PWCHAR HeapBuffer;
//     UNICODE_STRING SystemRootString;
//     UNICODE_STRING UnexpandedSystemRootString = RTL_CONSTANT_STRING(L"%SystemRoot%");
//     UNICODE_STRING BaseSrvCSDString;
//     UNICODE_STRING BaseSrvWindowsDirectory;
//     UNICODE_STRING BaseSrvWindowsSystemDirectory;
//     UNICODE_STRING BnoString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE BaseSrvNamedObjectDirectory;
//     HANDLE BaseSrvRestrictedObjectDirectory;
//     PACL BnoDacl, BnoRestrictedDacl;
//     PSECURITY_DESCRIPTOR BnoSd;
//     HANDLE SymHandle;
//     UNICODE_STRING DirectoryName, SymlinkName;
//     ULONG LuidEnabled;
//     RTL_QUERY_REGISTRY_TABLE BaseServerRegistryConfigurationTable[2] =
//     {
//         {
//             NULL,
//             RTL_QUERY_REGISTRY_DIRECT,
//             L"CSDVersion",
//             &BaseSrvCSDString,
//             REG_NONE, NULL, 0
//         },
//
//         {0}
//     };
//
//     /* Initialize the memory */
//     BaseSrvHeap = RtlGetProcessHeap();                  // Initialize our own heap.
//     BaseSrvSharedHeap = LoadedServerDll->SharedSection; // Get the CSR shared heap.
//
//     /* Get the session ID */
//     SessionId = NtCurrentPeb()->SessionId;
//
//     /* Get the Windows directory */
//     RtlInitEmptyUnicodeString(&SystemRootString, Buffer, sizeof(Buffer));
//     Status = RtlExpandEnvironmentStrings_U(NULL,
//                                            &UnexpandedSystemRootString,
//                                            &SystemRootString,
//                                            NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create the base directory */
//     Buffer[SystemRootString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//     Success = RtlCreateUnicodeString(&BaseSrvWindowsDirectory,
//                                      SystemRootString.Buffer);
//     ASSERT(Success);
//
//     /* Create the system directory */
//     wcscat(SystemRootString.Buffer, L"\\System32");
//     Success = RtlCreateUnicodeString(&BaseSrvWindowsSystemDirectory,
//                                      SystemRootString.Buffer);
//     ASSERT(Success);
//
//     /* Create the kernel32 path */
//     wcscat(SystemRootString.Buffer, L"\\kernel32.dll");
//     Success = RtlCreateUnicodeString(&BaseSrvKernel32DllPath,
//                                      SystemRootString.Buffer);
//     ASSERT(Success);
//
//     if (SessionId != 0)
//     {
//         swprintf(BnoBuffer, L"\\Sessions\\%ld\\BaseNamedObjects", SessionId);
//     }
//     else
//     {
//         wcscpy(BnoBuffer, L"\\BaseNamedObjects");
//     }
//     RtlInitUnicodeString(&BnoString, BnoBuffer);
//
//     /* Allocate the server data */
//     BaseStaticServerData = RtlAllocateHeap(BaseSrvSharedHeap,
//                                            HEAP_ZERO_MEMORY,
//                                            sizeof(BASE_STATIC_SERVER_DATA));
//     ASSERT(BaseStaticServerData != NULL);
//
//     /* Process timezone information */
//     BaseStaticServerData->TermsrvClientTimeZoneId = TIME_ZONE_ID_INVALID;
//     BaseStaticServerData->TermsrvClientTimeZoneChangeNum = 0;
//     Status = NtQuerySystemInformation(SystemTimeOfDayInformation,
//                                       &BaseStaticServerData->TimeOfDay,
//                                       sizeof(BaseStaticServerData->TimeOfDay),
//                                       NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Make a shared heap copy of the Windows directory */
//     BaseStaticServerData->WindowsDirectory = BaseSrvWindowsDirectory;
//     HeapBuffer = RtlAllocateHeap(BaseSrvSharedHeap,
//                                  0,
//                                  BaseSrvWindowsDirectory.MaximumLength);
//     ASSERT(HeapBuffer);
//     RtlCopyMemory(HeapBuffer,
//                   BaseStaticServerData->WindowsDirectory.Buffer,
//                   BaseSrvWindowsDirectory.MaximumLength);
//     BaseStaticServerData->WindowsDirectory.Buffer = HeapBuffer;
//
//     /* Make a shared heap copy of the System directory */
//     BaseStaticServerData->WindowsSystemDirectory = BaseSrvWindowsSystemDirectory;
//     HeapBuffer = RtlAllocateHeap(BaseSrvSharedHeap,
//                                  0,
//                                  BaseSrvWindowsSystemDirectory.MaximumLength);
//     ASSERT(HeapBuffer);
//     RtlCopyMemory(HeapBuffer,
//                   BaseStaticServerData->WindowsSystemDirectory.Buffer,
//                   BaseSrvWindowsSystemDirectory.MaximumLength);
//     BaseStaticServerData->WindowsSystemDirectory.Buffer = HeapBuffer;
//
//     /* This string is not used */
//     RtlInitEmptyUnicodeString(&BaseStaticServerData->WindowsSys32x86Directory,
//                               NULL,
//                               0);
//
//     /* Make a shared heap copy of the BNO directory */
//     BaseStaticServerData->NamedObjectDirectory = BnoString;
//     BaseStaticServerData->NamedObjectDirectory.MaximumLength = BnoString.Length +
//                                                                sizeof(UNICODE_NULL);
//     HeapBuffer = RtlAllocateHeap(BaseSrvSharedHeap,
//                                  0,
//                                  BaseStaticServerData->NamedObjectDirectory.MaximumLength);
//     ASSERT(HeapBuffer);
//     RtlCopyMemory(HeapBuffer,
//                   BaseStaticServerData->NamedObjectDirectory.Buffer,
//                   BaseStaticServerData->NamedObjectDirectory.MaximumLength);
//     BaseStaticServerData->NamedObjectDirectory.Buffer = HeapBuffer;
//
//     /*
//      * Confirmed that in Windows, CSDNumber and RCNumber are actually Length
//      * and MaximumLength of the CSD String, since the same UNICODE_STRING is
//      * being queried twice, the first time as a ULONG!
//      *
//      * Somehow, in Windows this doesn't cause a buffer overflow, but it might
//      * in ReactOS, so this code is disabled until someone figures out WTF.
//      */
//     BaseStaticServerData->CSDNumber = 0;
//     BaseStaticServerData->RCNumber = 0;
//
//     /* Initialize the CSD string and query its value from the registry */
//     RtlInitEmptyUnicodeString(&BaseSrvCSDString, Buffer, sizeof(Buffer));
//     Status = RtlQueryRegistryValues(RTL_REGISTRY_WINDOWS_NT,
//                                     L"",
//                                     BaseServerRegistryConfigurationTable,
//                                     NULL,
//                                     NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Copy into the shared buffer */
//         wcsncpy(BaseStaticServerData->CSDVersion,
//                 BaseSrvCSDString.Buffer,
//                 BaseSrvCSDString.Length / sizeof(WCHAR));
//     }
//     else
//     {
//         /* Indicate nothing is there */
//         BaseSrvCSDString.Length = 0;
//     }
//     /* NULL-terminate */
//     BaseStaticServerData->CSDVersion[BaseSrvCSDString.Length / sizeof(WCHAR)] = UNICODE_NULL;
//
//     /* Cache the system information */
//     Status = NtQuerySystemInformation(SystemBasicInformation,
//                                       &BaseStaticServerData->SysInfo,
//                                       sizeof(BaseStaticServerData->SysInfo),
//                                       NULL);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Setup the ini file mappings */
//     Status = BaseSrvInitializeIniFileMappings(BaseStaticServerData);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* FIXME: Should query the registry for these */
//     BaseStaticServerData->DefaultSeparateVDM = FALSE;
//     BaseStaticServerData->IsWowTaskReady = FALSE;
//
//     /* Allocate a security descriptor and create it */
//     BnoSd = RtlAllocateHeap(BaseSrvHeap, 0, 1024);
//     ASSERT(BnoSd);
//     Status = RtlCreateSecurityDescriptor(BnoSd, SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create the BNO and \Restricted DACLs */
//     Status = CreateBaseAcls(&BnoDacl, &BnoRestrictedDacl);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Set the BNO DACL as active for now */
//     Status = RtlSetDaclSecurityDescriptor(BnoSd, TRUE, BnoDacl, FALSE);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Create the BNO directory */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &BnoString,
//                                OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                BnoSd);
//     Status = NtCreateDirectoryObject(&BaseSrvNamedObjectDirectory,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Check if we are session 0 */
//     if (SessionId == 0)
//     {
//         /* Mark this as a session 0 directory */
//         Status = NtSetInformationObject(BaseSrvNamedObjectDirectory,
//                                         ObjectSessionInformation,
//                                         NULL,
//                                         0);
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Check if LUID device maps are enabled */
//     Status = NtQueryInformationProcess(NtCurrentProcess(),
//                                        ProcessLUIDDeviceMapsEnabled,
//                                        &LuidEnabled,
//                                        sizeof(LuidEnabled),
//                                        NULL);
//     ASSERT(NT_SUCCESS(Status));
//     BaseStaticServerData->LUIDDeviceMapsEnabled = (BOOLEAN)LuidEnabled;
//
//     /* Initialize Global */
//     if (!BaseStaticServerData->LUIDDeviceMapsEnabled ||
//         NT_SUCCESS(RtlInitializeCriticalSectionAndSpinCount(&BaseSrvDDDBSMCritSec, 0x80000000)))
//     {
//         /* Make Global point back to BNO */
//         RtlInitUnicodeString(&DirectoryName, L"Global");
//         RtlInitUnicodeString(&SymlinkName, L"\\BaseNamedObjects");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DirectoryName,
//                                    OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                    BaseSrvNamedObjectDirectory,
//                                    BnoSd);
//         Status = NtCreateSymbolicLinkObject(&SymHandle,
//                                             SYMBOLIC_LINK_ALL_ACCESS,
//                                             &ObjectAttributes,
//                                             &SymlinkName);
//         if ((NT_SUCCESS(Status)) && SessionId == 0) NtClose(SymHandle);
//
//         /* Make local point back to \Sessions\x\BNO */
//         RtlInitUnicodeString(&DirectoryName, L"Local");
//         ASSERT(SessionId == 0);
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DirectoryName,
//                                    OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                    BaseSrvNamedObjectDirectory,
//                                    BnoSd);
//         Status = NtCreateSymbolicLinkObject(&SymHandle,
//                                             SYMBOLIC_LINK_ALL_ACCESS,
//                                             &ObjectAttributes,
//                                             &BnoString);
//         if ((NT_SUCCESS(Status)) && SessionId == 0) NtClose(SymHandle);
//
//         /* Make Session point back to BNOLINKS */
//         RtlInitUnicodeString(&DirectoryName, L"Session");
//         RtlInitUnicodeString(&SymlinkName, L"\\Sessions\\BNOLINKS");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DirectoryName,
//                                    OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                    BaseSrvNamedObjectDirectory,
//                                    BnoSd);
//         Status = NtCreateSymbolicLinkObject(&SymHandle,
//                                             SYMBOLIC_LINK_ALL_ACCESS,
//                                             &ObjectAttributes,
//                                             &SymlinkName);
//         if ((NT_SUCCESS(Status)) && SessionId == 0) NtClose(SymHandle);
//
//         /* Create the BNO\Restricted directory and set the restricted DACL */
//         RtlInitUnicodeString(&DirectoryName, L"Restricted");
//         Status = RtlSetDaclSecurityDescriptor(BnoSd, TRUE, BnoRestrictedDacl, FALSE);
//         ASSERT(NT_SUCCESS(Status));
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &DirectoryName,
//                                    OBJ_OPENIF | OBJ_PERMANENT | OBJ_CASE_INSENSITIVE,
//                                    BaseSrvNamedObjectDirectory,
//                                    BnoSd);
//         Status = NtCreateDirectoryObject(&BaseSrvRestrictedObjectDirectory,
//                                          DIRECTORY_ALL_ACCESS,
//                                          &ObjectAttributes);
//         ASSERT(NT_SUCCESS(Status));
//     }
//     else
//     {
//         /* That should never happen */
//         ASSERT(FALSE);
//     }
//
//     /* Initialize NLS */
//     BaseSrvNLSInit(BaseStaticServerData);
//
//     /* Finally, set the pointer */
//     LoadedServerDll->SharedSection = BaseStaticServerData;
// }
//
// NTSTATUS
// NTAPI
// BaseClientConnectRoutine(IN PCSR_PROCESS CsrProcess,
//                          IN OUT PVOID  ConnectionInfo,
//                          IN OUT PULONG ConnectionInfoLength)
// {
//     PBASESRV_API_CONNECTINFO ConnectInfo = (PBASESRV_API_CONNECTINFO)ConnectionInfo;
//
//     if ( ConnectionInfo       == NULL ||
//          ConnectionInfoLength == NULL ||
//         *ConnectionInfoLength != sizeof(*ConnectInfo) )
//     {
//         DPRINT1("BASESRV: Connection failed - ConnectionInfo = 0x%p ; ConnectionInfoLength = 0x%p (%lu), expected %lu\n",
//                 ConnectionInfo,
//                 ConnectionInfoLength,
//                 ConnectionInfoLength ? *ConnectionInfoLength : (ULONG)-1,
//                 sizeof(*ConnectInfo));
//
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Do the NLS connection */
//     return BaseSrvNlsConnect(CsrProcess, ConnectionInfo, ConnectionInfoLength);
// }
//
// VOID
// NTAPI
// BaseClientDisconnectRoutine(IN PCSR_PROCESS CsrProcess)
// {
//     /* Cleanup VDM resources */
//     BaseSrvCleanupVDMResources(CsrProcess);
// }
//
// CSR_SERVER_DLL_INIT(ServerDllInitialization)
// {
//     /* Setup the DLL Object */
//     LoadedServerDll->ApiBase = BASESRV_FIRST_API_NUMBER;
//     LoadedServerDll->HighestApiSupported = BasepMaxApiNumber;
//     LoadedServerDll->DispatchTable = BaseServerApiDispatchTable;
//     LoadedServerDll->ValidTable = BaseServerApiServerValidTable;
// #ifdef CSR_DBG
//     LoadedServerDll->NameTable = BaseServerApiNameTable;
// #endif
//     LoadedServerDll->SizeOfProcessData = 0;
//     LoadedServerDll->ConnectCallback = BaseClientConnectRoutine;
//     LoadedServerDll->DisconnectCallback = BaseClientDisconnectRoutine;
//     LoadedServerDll->ShutdownProcessCallback = NULL;
//
//     BaseSrvDllInstance = LoadedServerDll->ServerHandle;
//
//     BaseInitializeStaticServerData(LoadedServerDll);
//
//     /* Initialize DOS devices management */
//     BaseInitDefineDosDevice();
//
//     /* Initialize VDM support */
//     BaseInitializeVDM();
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// BOOL
// NTAPI
// DllMain(IN HINSTANCE hInstanceDll,
//         IN DWORD dwReason,
//         IN LPVOID lpReserved)
// {
//     UNREFERENCED_PARAMETER(hInstanceDll);
//     UNREFERENCED_PARAMETER(dwReason);
//     UNREFERENCED_PARAMETER(lpReserved);
//
//     if (DLL_PROCESS_DETACH == dwReason)
//     {
//         BaseCleanupDefineDosDevice();
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
