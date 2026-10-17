use crate::prelude_rpc::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::rpc_stubs::NetrServerStatisticsGet;
use crate::rpc_stubs::NetrWorkstationStatisticsGet;
use core::ffi::c_void;

// ---- NTSTATUS codes from sdk/include/psdk/ntstatus.h that the netapi32 substrate
// does not re-export. Without them these four arms were catch-all bindings, so every
// NetpNtStatusToApiStatus() input returned the STATUS_INVALID_ACCOUNT_NAME mapping. ----
pub const STATUS_INVALID_ACCOUNT_NAME: NTSTATUS = 0xC0000062u32 as NTSTATUS;
pub const STATUS_USER_EXISTS: NTSTATUS = 0xC0000063u32 as NTSTATUS;
pub const STATUS_GROUP_EXISTS: NTSTATUS = 0xC0000065u32 as NTSTATUS;
pub const STATUS_PASSWORD_RESTRICTION: NTSTATUS = 0xC000006Cu32 as NTSTATUS;

// ---- file-scope preamble (standard netapi32 wine-debug channel; TRACE/FIXME reference it) ----
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};


// ---- NetStatisticsGet ----
#[no_mangle]
pub unsafe extern "system" fn NetStatisticsGet(
    server: *mut u16,
    service: *mut u16,
    level: u32,
    options: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    extern "C" {
        fn _wcsicmp(a: *const u16, b: *const u16) -> i32;
    }
    extern "system" {
    }

    let mut status = ERROR_NOT_SUPPORTED;
    *bufptr = core::ptr::null_mut();

    const LANMAN_WORKSTATION: [u16; 18] = [
        0x004C, 0x0061, 0x006E, 0x006D, 0x0061, 0x006E, 0x0057, 0x006F, 0x0072, 0x006B, 0x0073,
        0x0074, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
    ];
    const LANMAN_SERVER: [u16; 13] = [
        0x004C, 0x0061, 0x006E, 0x006D, 0x0061, 0x006E, 0x0053, 0x0065, 0x0072, 0x0076, 0x0065,
        0x0072, 0x0000,
    ];

    if _wcsicmp(service, LANMAN_WORKSTATION.as_ptr()) == 0 {
        if level != 0 {
            return ERROR_INVALID_LEVEL;
        }
        if options != 0 {
            return ERROR_INVALID_PARAMETER;
        }
        status = NetrWorkstationStatisticsGet(server, LANMAN_WORKSTATION.as_ptr().cast_mut(), level, options, (bufptr).cast());
    } else if _wcsicmp(service, LANMAN_SERVER.as_ptr()) == 0 {
        if level != 0 {
            return ERROR_INVALID_LEVEL;
        }
        if options != 0 {
            return ERROR_INVALID_PARAMETER;
        }
        status = NetrServerStatisticsGet(server, LANMAN_SERVER.as_ptr().cast_mut(), level, options, (bufptr).cast());
    }

    status
}

// ---- NetpAllocAnsiStrFromWStr ----
#[no_mangle]
pub unsafe extern "system" fn NetpAllocAnsiStrFromWStr(InString: PWSTR) -> PSTR {
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let Size: ULONG;
    let NetStatus: NET_API_STATUS;
    let Status: NTSTATUS;

    RtlInitUnicodeString(&mut UnicodeString, InString);

    Size = RtlUnicodeStringToAnsiSize(&UnicodeString);

    // NetApiBufferAllocate expects PVOID*, use a temporary
    let mut BufferPtr: *mut core::ffi::c_void = core::ptr::null_mut();
    NetStatus = NetApiBufferAllocate(Size, &mut BufferPtr);
    if NetStatus != 0 {
        return core::ptr::null_mut();
    }
    AnsiString.Buffer = BufferPtr as PSTR;

    Status = RtlUnicodeStringToAnsiString(&mut AnsiString, &UnicodeString, 0);
    if !NT_SUCCESS(Status) {
        NetApiBufferFree(AnsiString.Buffer as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    AnsiString.Buffer
}

// ---- NetpAllocStrFromWStr ----
#[no_mangle]
pub unsafe extern "system" fn NetpAllocStrFromWStr(
    InString: PWSTR,
) -> PSTR
{
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let mut OemString: OEM_STRING = core::mem::zeroed();
    let Size: ULONG;
    let NetStatus: NET_API_STATUS;
    let Status: NTSTATUS;

    RtlInitUnicodeString(&mut UnicodeString, InString);

    Size = RtlUnicodeStringToOemSize(&UnicodeString);
    NetStatus = NetApiBufferAllocate(
        Size,
        core::ptr::addr_of_mut!(OemString.Buffer) as *mut *mut core::ffi::c_void,
    );
    if NetStatus != 0 {
        return core::ptr::null_mut();
    }

    Status = RtlUnicodeStringToOemString(
        &mut OemString,
        &UnicodeString as *const UNICODE_STRING,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        NetApiBufferFree(OemString.Buffer as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    OemString.Buffer
}

// ---- NetpAllocWStrFromAnsiStr ----
// netapi32.h:79 declares this WINAPI (like its NetpAllocAnsiStrFromWStr sibling above) and
// netlogon.c calls it across TU boundaries, so it must be visible outside this module.
#[no_mangle]
pub unsafe extern "system" fn NetpAllocWStrFromAnsiStr(InString: PSTR) -> PWSTR {
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();

    RtlInitAnsiString(&mut AnsiString, InString);

    let Size: u32 = RtlAnsiStringToUnicodeSize(&AnsiString);
    let NetStatus: u32 = NetApiBufferAllocate(Size, &mut UnicodeString.Buffer as *mut PWSTR as *mut *mut core::ffi::c_void);
    if NetStatus != 0 {
        return core::ptr::null_mut();
    }

    // Same idiom as the NetpAllocWStrFromStr sibling below: bindgen types the SOURCE string as
    // `*mut _STRING` (the C prototype's PCANSI_STRING loses its const), and NetApiBufferFree
    // takes PVOID — so both arguments need an explicit cast.
    let Status: i32 = RtlAnsiStringToUnicodeString(
        &mut UnicodeString as *mut UNICODE_STRING,
        &mut AnsiString as *mut ANSI_STRING as *mut _,
        0u8,
    );
    if !NT_SUCCESS(Status) {
        NetApiBufferFree(UnicodeString.Buffer as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    UnicodeString.Buffer
}

// ---- NetpAllocWStrFromStr ----
#[no_mangle]
pub unsafe extern "system" fn NetpAllocWStrFromStr(InString: PSTR) -> PWSTR {
    let mut OemString: OEM_STRING = core::mem::zeroed();
    let mut UnicodeString: UNICODE_STRING = core::mem::zeroed();
    let Size: ULONG;
    let NetStatus: NET_API_STATUS;
    let Status: NTSTATUS;

    RtlInitAnsiString(
        &mut OemString as *mut OEM_STRING as PANSI_STRING,
        InString,
    );

    Size = RtlOemStringToUnicodeSize(&OemString as *const OEM_STRING as *const _);
    NetStatus = NetApiBufferAllocate(
        Size,
        core::ptr::addr_of_mut!(UnicodeString.Buffer) as *mut *mut core::ffi::c_void,
    );
    if NetStatus != NERR_Success {
        return core::ptr::null_mut();
    }

    Status = RtlOemStringToUnicodeString(
        &mut UnicodeString as *mut UNICODE_STRING,
        &OemString as *const OEM_STRING as *const _,
        0u8, // FALSE
    );
    if !NT_SUCCESS(Status) {
        NetApiBufferFree(UnicodeString.Buffer as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    UnicodeString.Buffer
}

// ---- NetpAllocWStrFromWStr ----
#[no_mangle]
pub unsafe extern "system" fn NetpAllocWStrFromWStr(InString: PWSTR) -> PWSTR {
    let mut OutString: PWSTR = core::ptr::null_mut();
    let Size: u32 = ((wcslen(InString) + 1) * core::mem::size_of::<u16>()) as u32;
    // NetApiBufferAllocate returns NET_API_STATUS (DWORD), not NTSTATUS — the C declares this
    // local `NET_API_STATUS NetStatus`.
    let Status: NET_API_STATUS = NetApiBufferAllocate(Size, &mut OutString as *mut PWSTR as *mut *mut core::ffi::c_void);
    if Status != 0 {
        return core::ptr::null_mut();
    }
    wcscpy(OutString, InString);
    OutString
}

// ---- NetpNtStatusToApiStatus ----
#[no_mangle]
pub unsafe extern "system" fn NetpNtStatusToApiStatus(Status: NTSTATUS) -> NET_API_STATUS {
    let ApiStatus: NET_API_STATUS;
    match Status {
        STATUS_SUCCESS => ApiStatus = NERR_Success,
        STATUS_INVALID_ACCOUNT_NAME => ApiStatus = NERR_BadUsername,
        STATUS_USER_EXISTS => ApiStatus = NERR_UserExists,
        STATUS_GROUP_EXISTS => ApiStatus = NERR_GroupExists,
        STATUS_PASSWORD_RESTRICTION => ApiStatus = NERR_PasswordTooShort,
        _ => ApiStatus = RtlNtStatusToDosError(Status) as NET_API_STATUS,
    }
    ApiStatus
}

// ---- NetRegisterDomainNameChangeNotification ----
#[no_mangle]
pub unsafe extern "system" fn NetRegisterDomainNameChangeNotification(
    NotificationEventHandle: PHANDLE,
) -> NET_API_STATUS {
    if NotificationEventHandle.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let EventHandle = CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
    if EventHandle.is_null() {
        return GetLastError();
    }

    // The info-class enum is a bindgen u32 const; the prototype takes a C `int`.
    // NOTE: misc.c:39-40 passes `NotificationEventHandle` — a PHANDLE — straight into the HANDLE
    // parameter (both are pointers, so C accepts it with at most a warning). Preserved verbatim
    // rather than "corrected" to *NotificationEventHandle: translation keeps the C's semantics.
    let Status = LsaRegisterPolicyChangeNotification(
        PolicyNotifyDnsDomainInformation as c_int,
        NotificationEventHandle as HANDLE,
    );
    if !NT_SUCCESS(Status) {
        CloseHandle(EventHandle);
        return NetpNtStatusToApiStatus(Status);
    }

    *NotificationEventHandle = EventHandle;
    NERR_Success
}

// ---- NetUnregisterDomainNameChangeNotification ----
#[no_mangle]
pub unsafe extern "system" fn NetUnregisterDomainNameChangeNotification(
    NotificationEventHandle: HANDLE,
) -> NET_API_STATUS {
    if NotificationEventHandle == core::ptr::null_mut() {
        return ERROR_INVALID_PARAMETER;
    }
    let Status = LsaUnregisterPolicyChangeNotification(
        PolicyNotifyDnsDomainInformation as c_int,
        NotificationEventHandle,
    );
    NetpNtStatusToApiStatus(Status)
}
