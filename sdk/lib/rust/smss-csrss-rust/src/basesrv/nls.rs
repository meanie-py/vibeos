use ntdll_sys::nt::*;

// RtlCopyMemory is a memmove macro (C). Re-translate as memcpy.
#[inline]
unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// InitializeObjectAttributes + NtCurrentProcess are C macros, not
// exports. Inline equivalents.
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    ObjectName: *mut _UNICODE_STRING,
    Attributes: ULONG,
    RootDirectory: HANDLE,
    SecurityDescriptor: PVOID,
) {
    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = RootDirectory;
    (*ObjectAttributes).Attributes = Attributes;
    (*ObjectAttributes).ObjectName = ObjectName;
    (*ObjectAttributes).SecurityDescriptor = SecurityDescriptor;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
}

#[inline(always)]
unsafe fn NtCurrentProcess() -> HANDLE {
    -1isize as HANDLE
}

extern "system" {
    fn RtlFillMemory(Destination: PVOID, Length: SIZE_T, Fill: UCHAR);
    fn LdrLoadDll(
        SearchPath: ULONG,
        LoadFlags: ULONG,
        DllName: *mut _UNICODE_STRING,
        DllHandle: *mut HANDLE,
    ) -> NTSTATUS;
    fn CsrValidateMessageBuffer(
        ApiMessage: PVOID,
        Buffer: *mut PVOID,
        Count: ULONG,
        ElementSize: ULONG,
    ) -> BOOLEAN;
}

const NLS_SECTION_SECURITY_DESCRIPTOR_SIZE: usize = 256;
const SEC_COMMIT: ULONG = 0x8000000;
const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
const OBJ_PERMANENT: ULONG = 0x00000010;
const OBJ_OPENIF: ULONG = 0x00000080;

#[repr(C)]
pub struct NLS_USER_INFO {
    pub UserLocaleId: ULONG,
    pub ulCacheUpdateCount: ULONG,
}

// NOTE: a `BASE_STATIC_SERVER_DATA` mirror with NlsUserInfo @offset 0 used to
// live here and was fed to BaseSrvNLSInit — it clobbered WindowsDirectory.
// BaseSrvNLSInit now receives the correctly-offset NlsUserInfo pointer directly
// (see init.rs), so no struct mirror is needed in this module. (task #18)

#[repr(C)]
pub struct CSR_PROCESS {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct BASE_API_MESSAGE {
    pub Header: CSR_API_MESSAGE_HEADER,
    pub Data: BASE_API_MESSAGE_DATA,
}

#[repr(C)]
pub struct CSR_API_MESSAGE_HEADER {
    pub ClientId: _CLIENT_ID,
}

#[repr(C)]
pub struct _CLIENT_ID {
    pub UniqueProcess: HANDLE,
    pub UniqueThread: HANDLE,
}

#[repr(C)]
pub union BASE_API_MESSAGE_DATA {
    pub NlsCreateSection: core::mem::ManuallyDrop<BASE_NLS_CREATE_SECTION>,
    pub NlsGetUserInfo: core::mem::ManuallyDrop<BASE_NLS_GET_USER_INFO>,
}

#[repr(C)]
pub struct BASE_NLS_CREATE_SECTION {
    pub Type: ULONG,
    pub LocaleId: ULONG,
    pub SectionHandle: HANDLE,
}

#[repr(C)]
pub struct BASE_NLS_GET_USER_INFO {
    pub NlsUserInfo: *mut NLS_USER_INFO,
    pub Size: ULONG,
}

type POPEN_DATA_FILE = Option<unsafe extern "system" fn(*mut HANDLE, *const WCHAR) -> NTSTATUS>;
type PVALIDATE_LOCALE = Option<unsafe extern "system" fn(ULONG) -> BOOLEAN>;
type PGET_NLS_SECTION_NAME = Option<
    unsafe extern "system" fn(ULONG, ULONG, ULONG, *const WCHAR, *mut WCHAR, ULONG) -> NTSTATUS,
>;
type PGET_CP_FILE_NAME_FROM_REGISTRY =
    Option<unsafe extern "system" fn(ULONG, *mut WCHAR, ULONG) -> BOOLEAN>;
type PCREATE_NLS_SECURITY_DESCRIPTOR =
    Option<unsafe extern "system" fn(PSECURITY_DESCRIPTOR, SIZE_T, ULONG) -> NTSTATUS>;

#[repr(C)]
struct BASESRV_KERNEL_IMPORTS {
    FunctionName: *const i8,
    FunctionPointer: *mut PVOID,
}

static mut NlsCacheCriticalSection: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
static mut pNlsRegUserInfo: *mut NLS_USER_INFO = core::ptr::null_mut();

static mut BaseSrvKernel32DelayLoadComplete: BOOLEAN = 0u8;
static mut BaseSrvKernel32DllHandle: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut BaseSrvKernel32DllPath: _UNICODE_STRING = unsafe { core::mem::zeroed() };

static mut pOpenDataFile: POPEN_DATA_FILE = None;
static mut pGetDefaultSortkeySize: PVOID = core::ptr::null_mut();
static mut pGetLinguistLangSize: PVOID = core::ptr::null_mut();
static mut pNlsConvertIntegerToString: PVOID = core::ptr::null_mut();
static mut pValidateLCType: PVOID = core::ptr::null_mut();
static mut pValidateLocale: PVALIDATE_LOCALE = None;
static mut pGetNlsSectionName: PGET_NLS_SECTION_NAME = None;
static mut pGetUserDefaultLangID: PVOID = core::ptr::null_mut();
static mut pGetCPFileNameFromRegistry: PGET_CP_FILE_NAME_FROM_REGISTRY = None;
static mut pCreateNlsSecurityDescriptor: PCREATE_NLS_SECURITY_DESCRIPTOR = None;

static mut BaseSrvKernel32Imports: [BASESRV_KERNEL_IMPORTS; 10] = [
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"OpenDataFile\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pOpenDataFile) as *mut PVOID },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"GetDefaultSortkeySize\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pGetDefaultSortkeySize) },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"GetLinguistLangSize\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pGetLinguistLangSize) },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"NlsConvertIntegerToString\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pNlsConvertIntegerToString) },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"ValidateLCType\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pValidateLCType) },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"ValidateLocale\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pValidateLocale) as *mut PVOID },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"GetNlsSectionName\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pGetNlsSectionName) as *mut PVOID },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"GetUserDefaultLangID\0".as_ptr() as *const i8,
        FunctionPointer: unsafe { core::ptr::addr_of_mut!(pGetUserDefaultLangID) },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"GetCPFileNameFromRegistry\0".as_ptr() as *const i8,
        FunctionPointer: unsafe {
            core::ptr::addr_of_mut!(pGetCPFileNameFromRegistry) as *mut PVOID
        },
    },
    BASESRV_KERNEL_IMPORTS {
        FunctionName: b"CreateNlsSecurityDescriptor\0".as_ptr() as *const i8,
        FunctionPointer: unsafe {
            core::ptr::addr_of_mut!(pCreateNlsSecurityDescriptor) as *mut PVOID
        },
    },
];

#[no_mangle]
pub unsafe extern "system" fn BaseSrvDelayLoadKernel32() -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut i: ULONG;
    let mut ProcedureName: ANSI_STRING = core::mem::zeroed();

    if BaseSrvKernel32DelayLoadComplete != 0 {
        return STATUS_SUCCESS;
    }

    i = 0;
    while i < 10 {
        if (*(BaseSrvKernel32Imports[i as usize].FunctionPointer as *mut PVOID)).is_null() {
            if BaseSrvKernel32DllHandle.is_null() {
                Status = LdrLoadDll(
                    0,
                    0,
                    core::ptr::addr_of_mut!(BaseSrvKernel32DllPath),
                    core::ptr::addr_of_mut!(BaseSrvKernel32DllHandle),
                );
                if Status < 0 {
                    return Status;
                }
            }

            RtlInitAnsiString(
                core::ptr::addr_of_mut!(ProcedureName),
                BaseSrvKernel32Imports[i as usize].FunctionName,
            );
            Status = LdrGetProcedureAddress(
                BaseSrvKernel32DllHandle,
                core::ptr::addr_of_mut!(ProcedureName),
                0,
                BaseSrvKernel32Imports[i as usize].FunctionPointer,
            );
            if Status < 0 {
                break;
            }
        }
        i += 1;
    }

    if i == 10 {
        BaseSrvKernel32DelayLoadComplete = 1u8;
        return STATUS_SUCCESS;
    }

    Status
}

// task #18: caller (init.rs) passes the *correctly-offset* NlsUserInfo pointer
// (`addr_of_mut!((*BaseStaticServerData).NlsUserInfo)`), NOT a pointer to a
// local `BASE_STATIC_SERVER_DATA` mirror. The old signature took
// `*mut BASE_STATIC_SERVER_DATA` where this module's mirror had NlsUserInfo at
// OFFSET 0 — so the 0xFF fill + NtQueryDefaultLocale wrote UserLocaleId (0x409)
// over WindowsDirectory{Length,MaxLength,Buffer} at offset 0, NULL-faulting
// kernel32 GetWindowsDirectoryW the first time any process read system.ini
// (msacm32 boot hang). NlsUserInfo truly lives at offset 0x184 (i386).
#[no_mangle]
pub unsafe extern "system" fn BaseSrvNLSInit(pNlsUserInfo: *mut NLS_USER_INFO) {
    RtlInitializeCriticalSection(core::ptr::addr_of_mut!(NlsCacheCriticalSection));

    pNlsRegUserInfo = pNlsUserInfo;
    RtlFillMemory(
        pNlsUserInfo as PVOID,
        core::mem::size_of::<NLS_USER_INFO>().try_into().unwrap(),
        0xFF,
    );

    (*pNlsRegUserInfo).UserLocaleId = 0;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(NlsCacheCriticalSection));
    (*pNlsRegUserInfo).ulCacheUpdateCount = 0;
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(NlsCacheCriticalSection));

    NtQueryDefaultLocale(0, core::ptr::addr_of_mut!((*pNlsRegUserInfo).UserLocaleId));
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsConnect(
    _CsrProcess: *mut CSR_PROCESS,
    _ConnectionInfo: PVOID,
    _ConnectionInfoLength: *mut ULONG,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsSetUserInfo(
    _ApiMessage: *mut BASE_API_MESSAGE,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsSetMultipleUserInfo(
    _ApiMessage: *mut BASE_API_MESSAGE,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsCreateSection(
    ApiMessage: *mut BASE_API_MESSAGE,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut SectionHandle: HANDLE = core::ptr::null_mut();
    let mut ProcessHandle: HANDLE = core::ptr::null_mut();
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let LocaleId: ULONG;
    let mut NlsSectionName: _UNICODE_STRING = core::mem::zeroed();
    let NlsFileName: *const WCHAR;
    let mut SecurityDescriptor: [u8; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE] =
        [0; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE];
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileNameBuffer: [WCHAR; 32] = [0; 32];
    let mut NlsSectionNameBuffer: [WCHAR; 32] = [0; 32];
    let NlsMsg: *mut BASE_NLS_CREATE_SECTION =
        core::ptr::addr_of_mut!((*ApiMessage).Data.NlsCreateSection)
            as *mut core::mem::ManuallyDrop<BASE_NLS_CREATE_SECTION>
            as *mut BASE_NLS_CREATE_SECTION;

    Status = BaseSrvDelayLoadKernel32();
    if Status < 0 {
        return Status;
    }

    (*NlsMsg).SectionHandle = core::ptr::null_mut();

    LocaleId = (*NlsMsg).LocaleId;
    if LocaleId != 0 {
        if (pValidateLocale.unwrap())(LocaleId) == 0 {
            return STATUS_INVALID_PARAMETER;
        }
    }

    static NLS_UNICODE: [u16; 23] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'U' as u16,
        b'n' as u16,
        b'i' as u16,
        b'c' as u16,
        b'o' as u16,
        b'd' as u16,
        b'e' as u16,
     0,];
    static UNICODE_NLS: [u16; 12] = [
        b'u' as u16,
        b'n' as u16,
        b'i' as u16,
        b'c' as u16,
        b'o' as u16,
        b'd' as u16,
        b'e' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_LOCALE: [u16; 22] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'L' as u16,
        b'o' as u16,
        b'c' as u16,
        b'a' as u16,
        b'l' as u16,
        b'e' as u16,
     0,];
    static LOCALE_NLS: [u16; 11] = [
        b'l' as u16,
        b'o' as u16,
        b'c' as u16,
        b'a' as u16,
        b'l' as u16,
        b'e' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_CTYPE: [u16; 21] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'C' as u16,
        b'T' as u16,
        b'y' as u16,
        b'p' as u16,
        b'e' as u16,
     0,];
    static CTYPE_NLS: [u16; 10] = [
        b'c' as u16,
        b't' as u16,
        b'y' as u16,
        b'p' as u16,
        b'e' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_SORTKEY: [u16; 23] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'S' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        b'k' as u16,
        b'e' as u16,
        b'y' as u16,
     0,];
    static SORTKEY_NLS: [u16; 12] = [
        b's' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        b'k' as u16,
        b'e' as u16,
        b'y' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_SORTTBLS: [u16; 24] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'S' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        b'T' as u16,
        b'b' as u16,
        b'l' as u16,
        b's' as u16,
     0,];
    static SORTTBLS_NLS: [u16; 13] = [
        b's' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        b't' as u16,
        b'b' as u16,
        b'l' as u16,
        b's' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_CP437: [u16; 21] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'C' as u16,
        b'P' as u16,
        b'4' as u16,
        b'3' as u16,
        b'7' as u16,
     0,];
    static C_437_NLS: [u16; 10] = [
        b'c' as u16,
        b'_' as u16,
        b'4' as u16,
        b'3' as u16,
        b'7' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_CP1252: [u16; 22] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'C' as u16,
        b'P' as u16,
        b'1' as u16,
        b'2' as u16,
        b'5' as u16,
        b'2' as u16,
     0,];
    static C_1252_NLS: [u16; 11] = [
        b'c' as u16,
        b'_' as u16,
        b'1' as u16,
        b'2' as u16,
        b'5' as u16,
        b'2' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];
    static NLS_LANG_EXCEPT: [u16; 27] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'L' as u16,
        b'A' as u16,
        b'N' as u16,
        b'G' as u16,
        b'_' as u16,
        b'E' as u16,
        b'X' as u16,
        b'C' as u16,
        b'E' as u16,
        b'P' as u16,
        b'T' as u16,
        0,
    ];
    static L_EXCEPT_NLS: [u16; 13] = [
        b'l' as u16,
        b'_' as u16,
        b'e' as u16,
        b'x' as u16,
        b'c' as u16,
        b'e' as u16,
        b'p' as u16,
        b't' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
     0,];
    static NLS_CP_PREFIX: [u16; 20] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'C' as u16,
        b'P' as u16,
        0,
        0,
        0,
    ];
    static NLS_GEO: [u16; 19] = [
        b'\\' as u16,
        b'N' as u16,
        b'L' as u16,
        b'S' as u16,
        b'\\' as u16,
        b'N' as u16,
        b'l' as u16,
        b's' as u16,
        b'S' as u16,
        b'e' as u16,
        b'c' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b'G' as u16,
        b'e' as u16,
        b'o' as u16,
     0,];
    static GEO_NLS: [u16; 8] = [
        b'g' as u16,
        b'e' as u16,
        b'o' as u16,
        b'.' as u16,
        b'n' as u16,
        b'l' as u16,
        b's' as u16,
        0,
    ];

    match (*NlsMsg).Type {
        1 => {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(NlsSectionName),
                NLS_UNICODE.as_ptr(),
            );
            NlsFileName = UNICODE_NLS.as_ptr();
        }
        2 => {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(NlsSectionName), NLS_LOCALE.as_ptr());
            NlsFileName = LOCALE_NLS.as_ptr();
        }
        3 => {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(NlsSectionName), NLS_CTYPE.as_ptr());
            NlsFileName = CTYPE_NLS.as_ptr();
        }
        4 => {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(NlsSectionName),
                NLS_SORTKEY.as_ptr(),
            );
            NlsFileName = SORTKEY_NLS.as_ptr();
        }
        5 => {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(NlsSectionName),
                NLS_SORTTBLS.as_ptr(),
            );
            NlsFileName = SORTTBLS_NLS.as_ptr();
        }
        6 => {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(NlsSectionName), NLS_CP437.as_ptr());
            NlsFileName = C_437_NLS.as_ptr();
        }
        7 => {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(NlsSectionName), NLS_CP1252.as_ptr());
            NlsFileName = C_1252_NLS.as_ptr();
        }
        8 => {
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(NlsSectionName),
                NLS_LANG_EXCEPT.as_ptr(),
            );
            NlsFileName = L_EXCEPT_NLS.as_ptr();
        }
        9 => {
            return STATUS_NOT_IMPLEMENTED;
        }
        10 => {
            return STATUS_NOT_IMPLEMENTED;
        }
        11 => {
            if (pGetCPFileNameFromRegistry.unwrap())(
                (*NlsMsg).LocaleId,
                FileNameBuffer.as_mut_ptr(),
                32,
            ) == 0
            {
                return STATUS_INVALID_PARAMETER;
            }

            Status = (pGetNlsSectionName.unwrap())(
                (*NlsMsg).LocaleId,
                10,
                0,
                NLS_CP_PREFIX.as_ptr(),
                NlsSectionNameBuffer.as_mut_ptr(),
                32,
            );
            if Status < 0 {
                return Status;
            }

            NlsFileName = FileNameBuffer.as_ptr();
            RtlInitUnicodeString(
                core::ptr::addr_of_mut!(NlsSectionName),
                NlsSectionNameBuffer.as_ptr(),
            );
        }
        12 => {
            RtlInitUnicodeString(core::ptr::addr_of_mut!(NlsSectionName), NLS_GEO.as_ptr());
            NlsFileName = GEO_NLS.as_ptr();
        }
        _ => {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Status = (pOpenDataFile.unwrap())(core::ptr::addr_of_mut!(FileHandle), NlsFileName);
    if Status != STATUS_SUCCESS {
        return Status;
    }

    Status = (pCreateNlsSecurityDescriptor.unwrap())(
        SecurityDescriptor.as_mut_ptr() as PSECURITY_DESCRIPTOR,
        NLS_SECTION_SECURITY_DESCRIPTOR_SIZE.try_into().unwrap(),
        SECTION_MAP_READ,
    );
    if Status < 0 {
        NtClose(FileHandle);
        return Status;
    }

    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(NlsSectionName),
        OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_OPENIF,
        core::ptr::null_mut(),
        SecurityDescriptor.as_mut_ptr() as PSECURITY_DESCRIPTOR,
    );
    Status = NtCreateSection(
        core::ptr::addr_of_mut!(SectionHandle),
        SECTION_MAP_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::null_mut(),
        PAGE_READONLY,
        SEC_COMMIT,
        FileHandle,
    );
    NtClose(FileHandle);
    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenProcess(
        core::ptr::addr_of_mut!(ProcessHandle),
        PROCESS_DUP_HANDLE,
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!((*ApiMessage).Header.ClientId) as *mut ntdll_sys::nt::_CLIENT_ID,
    );
    if Status < 0 {
        NtClose(SectionHandle);
        return Status;
    }

    Status = NtDuplicateObject(
        NtCurrentProcess(),
        SectionHandle,
        ProcessHandle,
        core::ptr::addr_of_mut!((*NlsMsg).SectionHandle),
        0,
        0,
        3,
    );
    NtClose(ProcessHandle);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsUpdateCacheCount(
    _ApiMessage: *mut BASE_API_MESSAGE,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsGetUserInfo(
    ApiMessage: *mut BASE_API_MESSAGE,
    _ReplyCode: *mut ULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let NlsMsg: *mut BASE_NLS_GET_USER_INFO =
        core::ptr::addr_of_mut!((*ApiMessage).Data.NlsGetUserInfo)
            as *mut core::mem::ManuallyDrop<BASE_NLS_GET_USER_INFO>
            as *mut BASE_NLS_GET_USER_INFO;

    if CsrValidateMessageBuffer(
        ApiMessage as PVOID,
        core::ptr::addr_of_mut!((*NlsMsg).NlsUserInfo) as *mut PVOID,
        (*NlsMsg).Size,
        core::mem::size_of::<UCHAR>() as ULONG,
    ) != 0
        && (*NlsMsg).Size == core::mem::size_of::<NLS_USER_INFO>() as ULONG
    {
        Status = RtlEnterCriticalSection(core::ptr::addr_of_mut!(NlsCacheCriticalSection));
        if Status >= 0 {
            RtlCopyMemory(
                (*NlsMsg).NlsUserInfo as PVOID,
                pNlsRegUserInfo as PVOID,
                (*NlsMsg).Size as usize,
            );
            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(NlsCacheCriticalSection));
        }
    } else {
        Status = STATUS_INVALID_PARAMETER;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsLogon(_Unknown: ULONG) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn BaseSrvNlsUpdateRegistryCache(
    _Unknown1: ULONG,
    _Unknown2: ULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Base API Server DLL
//  * FILE:            subsystems/win/basesrv/nls.c
//  * PURPOSE:         National Language Support (NLS)
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "basesrv.h"
//
// #include <ndk/mmfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// RTL_CRITICAL_SECTION NlsCacheCriticalSection;
// PNLS_USER_INFO pNlsRegUserInfo;
//
// BOOLEAN BaseSrvKernel32DelayLoadComplete;
// HANDLE BaseSrvKernel32DllHandle;
// UNICODE_STRING BaseSrvKernel32DllPath;
//
// POPEN_DATA_FILE pOpenDataFile;
// PVOID /*PGET_DEFAULT_SORTKEY_SIZE */ pGetDefaultSortkeySize;
// PVOID /*PGET_LINGUIST_LANG_SIZE*/ pGetLinguistLangSize;
// PVOID /*PNLS_CONVERT_INTEGER_TO_STRING*/ pNlsConvertIntegerToString;
// PVOID /*PVALIDATE_LCTYPE*/ pValidateLCType;
// PVALIDATE_LOCALE pValidateLocale;
// PGET_NLS_SECTION_NAME pGetNlsSectionName;
// PVOID /*PGET_USER_DEFAULT_LANGID*/ pGetUserDefaultLangID;
// PGET_CP_FILE_NAME_FROM_REGISTRY pGetCPFileNameFromRegistry;
//
// NTSTATUS
// (WINAPI *pCreateNlsSecurityDescriptor)(
//     _Out_ PSECURITY_DESCRIPTOR SecurityDescriptor,
//     _In_ SIZE_T DescriptorSize,
//     _In_ ULONG AccessMask);
//
// BASESRV_KERNEL_IMPORTS BaseSrvKernel32Imports[10] =
// {
//     { "OpenDataFile", (PVOID*) &pOpenDataFile },
//     { "GetDefaultSortkeySize", (PVOID*) &pGetDefaultSortkeySize },
//     { "GetLinguistLangSize", (PVOID*) &pGetLinguistLangSize },
//     { "NlsConvertIntegerToString", (PVOID*) &pNlsConvertIntegerToString },
//     { "ValidateLCType", (PVOID*) &pValidateLCType },
//     { "ValidateLocale", (PVOID*) &pValidateLocale },
//     { "GetNlsSectionName", (PVOID*) &pGetNlsSectionName },
//     { "GetUserDefaultLangID", (PVOID*) &pGetUserDefaultLangID },
//     { "GetCPFileNameFromRegistry", (PVOID*) &pGetCPFileNameFromRegistry },
//     { "CreateNlsSecurityDescriptor", (PVOID*) &pCreateNlsSecurityDescriptor },
// };
//
// /* FUNCTIONS *****************************************************************/
//
// NTSTATUS
// NTAPI
// BaseSrvDelayLoadKernel32(VOID)
// {
//     NTSTATUS Status;
//     ULONG i;
//     ANSI_STRING ProcedureName;
//
//     /* Only do this once */
//     if (BaseSrvKernel32DelayLoadComplete) return STATUS_SUCCESS;
//
//     /* Loop all imports */
//     for (i = 0; i < RTL_NUMBER_OF(BaseSrvKernel32Imports); i++)
//     {
//         /* Only look them up once */
//         if (!*BaseSrvKernel32Imports[i].FunctionPointer)
//         {
//             /* If we haven't loaded the DLL yet, do it now */
//             if (!BaseSrvKernel32DllHandle)
//             {
//                 Status = LdrLoadDll(0,
//                                     0,
//                                     &BaseSrvKernel32DllPath,
//                                     &BaseSrvKernel32DllHandle);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     DPRINT1("Failed to load %wZ\n", &BaseSrvKernel32DllPath);
//                     return Status;
//                 }
//             }
//
//             /* Get the address of the routine being looked up*/
//             RtlInitAnsiString(&ProcedureName, BaseSrvKernel32Imports[i].FunctionName);
//             Status = LdrGetProcedureAddress(BaseSrvKernel32DllHandle,
//                                             &ProcedureName,
//                                             0,
//                                             BaseSrvKernel32Imports[i].FunctionPointer);
//             DPRINT1("NLS: Found %Z at 0x%p\n",
//                     &ProcedureName,
//                     BaseSrvKernel32Imports[i].FunctionPointer);
//             if (!NT_SUCCESS(Status)) break;
//         }
//     }
//
//     /* Did we find them all? */
//     if (i == RTL_NUMBER_OF(BaseSrvKernel32Imports))
//     {
//         /* Excellent */
//         BaseSrvKernel32DelayLoadComplete = TRUE;
//         return STATUS_SUCCESS;
//     }
//
//     /* Nope, fail */
//     return Status;
// }
//
// VOID
// NTAPI
// BaseSrvNLSInit(IN PBASE_STATIC_SERVER_DATA StaticData)
// {
//     /* Initialize the lock */
//     RtlInitializeCriticalSection(&NlsCacheCriticalSection);
//
//     /* Initialize the data with all F's */
//     pNlsRegUserInfo = &StaticData->NlsUserInfo;
//     RtlFillMemory(&StaticData->NlsUserInfo, sizeof(StaticData->NlsUserInfo), 0xFF);
//
//     /* Set empty LCID */
//     pNlsRegUserInfo->UserLocaleId = 0;
//
//     /* Reset the cache update counter */
//     RtlEnterCriticalSection(&NlsCacheCriticalSection);
//     pNlsRegUserInfo->ulCacheUpdateCount = 0;
//     RtlLeaveCriticalSection(&NlsCacheCriticalSection);
//
//     /* Get the LCID */
//     NtQueryDefaultLocale(0, &pNlsRegUserInfo->UserLocaleId);
// }
//
// NTSTATUS
// NTAPI
// BaseSrvNlsConnect(IN PCSR_PROCESS CsrProcess,
//                   IN OUT PVOID  ConnectionInfo,
//                   IN OUT PULONG ConnectionInfoLength)
// {
//     /* Does nothing */
//     return STATUS_SUCCESS;
// }
//
// /* PUBLIC SERVER APIS *********************************************************/
//
// CSR_API(BaseSrvNlsSetUserInfo)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvNlsSetMultipleUserInfo)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvNlsCreateSection)
// {
//     NTSTATUS Status;
//     HANDLE SectionHandle, ProcessHandle, FileHandle;
//     ULONG LocaleId;
//     UNICODE_STRING NlsSectionName;
//     PWCHAR NlsFileName;
//     UCHAR SecurityDescriptor[NLS_SECTION_SECURITY_DESCRIPTOR_SIZE];
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     WCHAR FileNameBuffer[32];
//     WCHAR NlsSectionNameBuffer[32];
//     PBASE_NLS_CREATE_SECTION NlsMsg = &((PBASE_API_MESSAGE)ApiMessage)->Data.NlsCreateSection;
//
//     /* Load kernel32 first and import the NLS routines */
//     Status = BaseSrvDelayLoadKernel32();
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Assume failure */
//     NlsMsg->SectionHandle = NULL;
//
//     /* Check and validate the locale ID, if one is present */
//     LocaleId = NlsMsg->LocaleId;
//     DPRINT1("NLS: Create Section with LCID: %lx for Type: %d\n", LocaleId, NlsMsg->Type);
//     if (LocaleId)
//     {
//         if (!pValidateLocale(LocaleId)) return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check which NLS section is being created */
//     switch (NlsMsg->Type)
//     {
//         /* For each one, set the correct filename and object name */
//         case 1:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionUnicode");
//             NlsFileName = L"unicode.nls";
//             break;
//         case 2:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionLocale");
//             NlsFileName =  L"locale.nls";
//             break;
//         case 3:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionCType");
//             NlsFileName = L"ctype.nls";
//             break;
//         case 4:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionSortkey");
//             NlsFileName = L"sortkey.nls";
//             break;
//         case 5:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionSortTbls");
//             NlsFileName = L"sorttbls.nls";
//             break;
//         case 6:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionCP437");
//             NlsFileName = L"c_437.nls";
//             break;
//         case 7:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionCP1252");
//             NlsFileName = L"c_1252.nls";
//             break;
//         case 8:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionLANG_EXCEPT");
//             NlsFileName = L"l_except.nls";
//             break;
//         case 9:
//             DPRINT1("This type not yet supported\n");
//             return STATUS_NOT_IMPLEMENTED;
//         case 10:
//             DPRINT1("This type not yet supported\n");
//             return STATUS_NOT_IMPLEMENTED;
//         case 11:
//             /* Get the filename for this locale */
//             if (!pGetCPFileNameFromRegistry(NlsMsg->LocaleId,
//                                             FileNameBuffer,
//                                             RTL_NUMBER_OF(FileNameBuffer)))
//             {
//                 DPRINT1("File name query failed\n");
//                 return STATUS_INVALID_PARAMETER;
//             }
//
//             /* Get the name of the section for this locale */
//             DPRINT1("File name: %S\n", FileNameBuffer);
//             Status = pGetNlsSectionName(NlsMsg->LocaleId,
//                                         10,
//                                         0,
//                                         L"\\NLS\\NlsSectionCP",
//                                         NlsSectionNameBuffer,
//                                         RTL_NUMBER_OF(NlsSectionNameBuffer));
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("Section name query failed: %lx\n", Status);
//                 return Status;
//             }
//
//             /* Setup the name and go open it */
//             NlsFileName = FileNameBuffer;
//             DPRINT1("Section name: %S\n", NlsSectionNameBuffer);
//             RtlInitUnicodeString(&NlsSectionName, NlsSectionNameBuffer);
//             break;
//         case 12:
//             RtlInitUnicodeString(&NlsSectionName, L"\\NLS\\NlsSectionGeo");
//             NlsFileName = L"geo.nls";
//             break;
//         default:
//             DPRINT1("NLS: Invalid NLS type!\n");
//             return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Open the specified NLS file */
//     Status = pOpenDataFile(&FileHandle, NlsFileName);
//     if (Status != STATUS_SUCCESS)
//     {
//         DPRINT1("NLS: Failed to open file: %lx\n", Status);
//         return Status;
//     }
//
//     /* Create an SD for the section object */
//     Status = pCreateNlsSecurityDescriptor(&SecurityDescriptor,
//                                           sizeof(SecurityDescriptor),
//                                           SECTION_MAP_READ);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NLS: CreateNlsSecurityDescriptor FAILED!: %lx\n", Status);
//         NtClose(FileHandle);
//         return Status;
//     }
//
//     /* Create the section object proper */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NlsSectionName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT | OBJ_OPENIF,
//                                NULL,
//                                &SecurityDescriptor);
//     Status = NtCreateSection(&SectionHandle,
//                              SECTION_MAP_READ,
//                              &ObjectAttributes,
//                              0,
//                              PAGE_READONLY,
//                              SEC_COMMIT,
//                              FileHandle);
//     NtClose(FileHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NLS: Failed to create section! %lx\n", Status);
//         return Status;
//     }
//
//     /* Open a handle to the calling process */
//     InitializeObjectAttributes(&ObjectAttributes, NULL, 0, NULL, NULL);
//     Status = NtOpenProcess(&ProcessHandle,
//                            PROCESS_DUP_HANDLE,
//                            &ObjectAttributes,
//                            &ApiMessage->Header.ClientId);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("NLS: Failed to open process! %lx\n", Status);
//         NtClose(SectionHandle);
//         return Status;
//     }
//
//     /* Duplicate the handle to the section object into it */
//     Status = NtDuplicateObject(NtCurrentProcess(),
//                                SectionHandle,
//                                ProcessHandle,
//                                &NlsMsg->SectionHandle,
//                                0,
//                                0,
//                                3);
//     NtClose(ProcessHandle);
//     return Status;
// }
//
// CSR_API(BaseSrvNlsUpdateCacheCount)
// {
//     DPRINT1("%s not yet implemented\n", __FUNCTION__);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// CSR_API(BaseSrvNlsGetUserInfo)
// {
//     NTSTATUS Status;
//     PBASE_NLS_GET_USER_INFO NlsMsg = &((PBASE_API_MESSAGE)ApiMessage)->Data.NlsGetUserInfo;
//
//     /* Make sure the buffer is valid and of the right size */
//     if ((CsrValidateMessageBuffer(ApiMessage, &NlsMsg->NlsUserInfo, NlsMsg->Size, sizeof(BYTE))) &&
//         (NlsMsg->Size == sizeof(NLS_USER_INFO)))
//     {
//         /* Acquire the lock to prevent updates while we copy */
//         Status = RtlEnterCriticalSection(&NlsCacheCriticalSection);
//         if (NT_SUCCESS(Status))
//         {
//             /* Do the copy now, then drop the lock */
//             RtlCopyMemory(NlsMsg->NlsUserInfo, pNlsRegUserInfo, NlsMsg->Size);
//             DPRINT1("NLS Data copy complete\n");
//             RtlLeaveCriticalSection(&NlsCacheCriticalSection);
//         }
//     }
//     else
//     {
//         /* The data was invalid, bail out */
//         DPRINT1("NLS: Size of info is invalid: %lx vs %lx\n", NlsMsg->Size, sizeof(NLS_USER_INFO));
//         Status = STATUS_INVALID_PARAMETER;
//     }
//
//     /* All done */
//     return Status;
// }
//
// /* PUBLIC APIS ****************************************************************/
//
// NTSTATUS
// NTAPI
// BaseSrvNlsLogon(DWORD Unknown)
// {
//     DPRINT1("%s(%lu) not yet implemented\n", __FUNCTION__, Unknown);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// BaseSrvNlsUpdateRegistryCache(DWORD Unknown1,
//                               DWORD Unknown2)
// {
//     DPRINT1("%s(%lu, %lu) not yet implemented\n", __FUNCTION__, Unknown1, Unknown2);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
