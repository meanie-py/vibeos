use ntdll_sys::nt::*;
// RtlGetProcessHeap is a C macro (ndk/rtlfuncs.h:1127:
// `#define RtlGetProcessHeap() (NtCurrentPeb()->ProcessHeap)`), NOT an
// ntdll export — declaring it `extern` produces undefined `_RtlGetProcessHeap@0`.
#[inline]
unsafe fn RtlGetProcessHeap() -> HANDLE {
    (*NtCurrentPeb()).ProcessHeap as HANDLE
}
#[no_mangle]
pub static mut CsrDebug: ULONG = 0;
#[no_mangle]
pub static mut CsrHeap: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrObjectDirectory: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrDirectoryName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CsrSbApiPortName: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut CsrSbApiPort: HANDLE = core::ptr::null_mut();
static mut CsrSbApiRequestThreadPtr: PCSR_THREAD = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrSmApiPort: HANDLE = core::ptr::null_mut();
static mut hSbApiPort: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrApiPort: HANDLE = core::ptr::null_mut();
#[cfg(target_os = "windows")]
static mut CsrWindowsControl: BOOLEAN = 0;
#[cfg(target_os = "windows")]
pub static mut CsrSubSystemType: ULONG = 0;
#[no_mangle]
pub static mut CsrMaxApiRequestThreads: ULONG = 0;
#[no_mangle]
pub static mut CsrTotalPerProcessDataLength: ULONG = 0;
static mut SessionId: ULONG = 0;
static mut BNOLinksDirectory: HANDLE = core::ptr::null_mut();
static mut SessionObjectDirectory: HANDLE = core::ptr::null_mut();
static mut DosDevicesDirectory: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrNtSysInfo: SYSTEM_BASIC_INFORMATION = unsafe { core::mem::zeroed() };

unsafe extern "system" fn CsrSetProcessSecurity() -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let hProcess = NtCurrentProcess();
    let mut Length: ULONG = 0;
    let mut TokenInfo: PTOKEN_USER = core::ptr::null_mut();
    let mut ProcSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let Dacl: PACL;
    let UserSid: PSID;

    Status = NtOpenProcessToken(hProcess, TOKEN_QUERY, &mut hToken);
    if Status < 0 {
        return Status;
    }

    Status = NtQueryInformationToken(hToken, TokenUser, core::ptr::null_mut(), 0, &mut Length);
    if Status != STATUS_BUFFER_TOO_SMALL {
        NtClose(hToken);
        return Status;
    }

    TokenInfo = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        (Length as usize).try_into().unwrap(),
    ) as PTOKEN_USER;
    if TokenInfo.is_null() {
        NtClose(hToken);
        Status = STATUS_NO_MEMORY;
        return Status;
    }

    Status = NtQueryInformationToken(hToken, TokenUser, TokenInfo as PVOID, Length, &mut Length);
    NtClose(hToken);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    UserSid = (*TokenInfo).User.Sid;
    Length = RtlLengthSid(UserSid)
        + core::mem::size_of::<ACL>() as u32
        + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32;

    ProcSd = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        ((SECURITY_DESCRIPTOR_MIN_LENGTH + Length) as usize)
            .try_into()
            .unwrap(),
    ) as PSECURITY_DESCRIPTOR;
    if ProcSd.is_null() {
        Status = STATUS_NO_MEMORY;
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    Dacl = (ProcSd as usize + SECURITY_DESCRIPTOR_MIN_LENGTH as usize) as PACL;

    Status = RtlCreateSecurityDescriptor(ProcSd, SECURITY_DESCRIPTOR_REVISION);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, ProcSd as PVOID);
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    Status = RtlCreateAcl(Dacl, Length, ACL_REVISION2);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, ProcSd as PVOID);
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    Status = RtlAddAccessAllowedAce(
        Dacl,
        ACL_REVISION,
        PROCESS_VM_READ
            | PROCESS_VM_WRITE
            | PROCESS_VM_OPERATION
            | PROCESS_DUP_HANDLE
            | PROCESS_TERMINATE
            | PROCESS_SUSPEND_RESUME
            | PROCESS_QUERY_INFORMATION
            | READ_CONTROL,
        UserSid,
    );
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, ProcSd as PVOID);
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    Status = RtlSetDaclSecurityDescriptor(ProcSd, 1, Dacl, 0);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, ProcSd as PVOID);
        RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
        return Status;
    }

    Status = NtSetSecurityObject(hProcess, DACL_SECURITY_INFORMATION, ProcSd);
    RtlFreeHeap(CsrHeap, 0, ProcSd as PVOID);
    RtlFreeHeap(CsrHeap, 0, TokenInfo as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrSetDirectorySecurity(_ObjectDirectory: HANDLE) -> NTSTATUS {
    STATUS_SUCCESS
}

unsafe extern "system" fn GetDosDevicesProtection(DosDevicesSd: PSECURITY_DESCRIPTOR) -> NTSTATUS {
    let WorldAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };
    let CreatorAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 3],
    };
    let NtSidAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut WorldSid: PSID = core::ptr::null_mut();
    let mut CreatorSid: PSID = core::ptr::null_mut();
    let mut AdminSid: PSID = core::ptr::null_mut();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut KeyValueBuffer: [u8; 0x40] = [0; 0x40];
    let KeyValuePartialInfoLocal: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ProtectionModeLocal: ULONG = 0;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let Dacl: PACL;
    let mut Ace: PACCESS_ALLOWED_ACE = core::ptr::null_mut();
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ResultLength: ULONG = 0;
    let SidLength: ULONG;
    let AclLength: ULONG;

    static SM_REG_KEY: [u16; 34] = [
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
        0,
    ];

    static PROTECTION_MODE_KEY: [u16; 15] = [
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

    Status = RtlCreateSecurityDescriptor(DosDevicesSd, SECURITY_DESCRIPTOR_REVISION);

    Status = RtlAllocateAndInitializeSid(
        &NtSidAuthority as *const _ as *mut _,
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

    Status = RtlAllocateAndInitializeSid(
        &NtSidAuthority as *const _ as *mut _,
        2,
        SECURITY_BUILTIN_DOMAIN_RID,
        DOMAIN_ALIAS_RID_ADMINS,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut AdminSid,
    );

    Status = RtlAllocateAndInitializeSid(
        &CreatorAuthority as *const _ as *mut _,
        1,
        SECURITY_CREATOR_OWNER_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut CreatorSid,
    );

    RtlInitUnicodeString(&mut KeyName, SM_REG_KEY.as_ptr());
    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut hKey, KEY_READ, &mut ObjectAttributes);
    if Status >= 0 {
        RtlInitUnicodeString(&mut KeyName, PROTECTION_MODE_KEY.as_ptr());
        Status = NtQueryValueKey(
            hKey,
            &mut KeyName,
            KeyValuePartialInformation,
            KeyValueBuffer.as_mut_ptr() as PVOID,
            core::mem::size_of_val(&KeyValueBuffer) as ULONG,
            &mut ResultLength,
        );

        KeyValuePartialInfoLocal = KeyValueBuffer.as_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
        if Status >= 0
            && (*KeyValuePartialInfoLocal).Type == REG_DWORD
            && *((*KeyValuePartialInfoLocal).Data.as_ptr() as *const ULONG) != 0
        {
            ProtectionModeLocal = *((*KeyValuePartialInfoLocal).Data.as_ptr() as *const ULONG);
        }

        NtClose(hKey);
    }

    if (ProtectionModeLocal & 3) != 0 {
        SidLength = RtlLengthSid(CreatorSid) + RtlLengthSid(SystemSid) + RtlLengthSid(AdminSid);
        AclLength = core::mem::size_of::<ACL>() as u32
            + 3 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
            + SidLength;

        Dacl = RtlAllocateHeap(
            CsrHeap,
            HEAP_ZERO_MEMORY,
            (AclLength as usize).try_into().unwrap(),
        ) as PACL;

        Status = RtlCreateAcl(Dacl, AclLength, ACL_REVISION2);
        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SystemSid);
        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, AdminSid);
        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, CreatorSid);

        Status = RtlGetAce(Dacl, 0, &mut Ace as *mut _ as *mut PVOID);
        (*Ace).Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as u8;
        Status = RtlGetAce(Dacl, 1, &mut Ace as *mut _ as *mut PVOID);
        (*Ace).Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as u8;
        Status = RtlGetAce(Dacl, 2, &mut Ace as *mut _ as *mut PVOID);
        (*Ace).Header.AceFlags |=
            (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;

        Status = RtlSetDaclSecurityDescriptor(DosDevicesSd, 1, Dacl, 0);
    } else {
        SidLength = RtlLengthSid(WorldSid) + RtlLengthSid(SystemSid);
        AclLength = core::mem::size_of::<ACL>() as u32
            + 3 * core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32
            + SidLength;

        Dacl = RtlAllocateHeap(
            CsrHeap,
            HEAP_ZERO_MEMORY,
            (AclLength as usize).try_into().unwrap(),
        ) as PACL;

        Status = RtlCreateAcl(Dacl, AclLength, ACL_REVISION2);
        Status = RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE,
            WorldSid,
        );
        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SystemSid);
        Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, WorldSid);

        Status = RtlGetAce(Dacl, 2, &mut Ace as *mut _ as *mut PVOID);
        (*Ace).Header.AceFlags |=
            (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;

        Status = RtlSetDaclSecurityDescriptor(DosDevicesSd, 1, Dacl, 0);
    }

    RtlFreeSid(CreatorSid);
    RtlFreeSid(AdminSid);
    RtlFreeSid(WorldSid);
    RtlFreeSid(SystemSid);

    Status
}

unsafe extern "system" fn FreeDosDevicesProtection(DosDevicesSd: PSECURITY_DESCRIPTOR) {
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut Present: BOOLEAN = 0;
    let mut Default: BOOLEAN = 0;

    let Status = RtlGetDaclSecurityDescriptor(DosDevicesSd, &mut Present, &mut Dacl, &mut Default);
    if Status >= 0 && !Dacl.is_null() {
        RtlFreeHeap(CsrHeap, 0, Dacl as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CsrCreateSessionObjectDirectory(Session: ULONG) -> NTSTATUS {
    let mut SessionBuffer: [u16; 512] = [0; 512];
    let mut BnoBuffer: [u16; 512] = [0; 512];
    let mut SessionString: _UNICODE_STRING = core::mem::zeroed();
    let mut BnoString: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut BnoHandle: HANDLE = core::ptr::null_mut();
    let mut DosDevicesSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut Status: NTSTATUS;

    // srv.h SESSION_ROOT is L"\\Sessions" (plural). The pieces below are appended
    // with `wappend`, which stops at a NUL, so a terminator inside the buffer can
    // never cut a name short.
    static SESSION_ROOT: [u16; 10] = [
        b'\\' as u16,
        b'S' as u16,
        b'e' as u16,
        b's' as u16,
        b's' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b's' as u16,
        0,
    ];
    static BNOLINKS: [u16; 10] = [
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
    static BASE_NAMED_OBJECTS: [u16; 18] = [
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
    static DOS_DEVICES: [u16; 11] = [
        b'D' as u16,
        b'o' as u16,
        b's' as u16,
        b'D' as u16,
        b'e' as u16,
        b'v' as u16,
        b'i' as u16,
        b'c' as u16,
        b'e' as u16,
        b's' as u16,
        0,
    ];

    /* _swprintf(SessionBuffer, L"%ws\\BNOLINKS", SESSION_ROOT) */
    let mut idx = 0;
    wappend(&mut SessionBuffer, &mut idx, &SESSION_ROOT);
    wappend(&mut SessionBuffer, &mut idx, &BNOLINKS);
    RtlInitUnicodeString(&mut SessionString, SessionBuffer.as_ptr());

    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut SessionString;
    ObjectAttributes.Attributes = OBJ_OPENIF | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateDirectoryObject(
        &raw mut BNOLinksDirectory,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        return Status;
    }

    let mut session_str: [u16; 32] = [0; 32];
    let session_len = format_number(Session, &mut session_str);
    RtlInitUnicodeString(&mut SessionString, session_str.as_ptr());

    if Session != 0 {
        /* _swprintf(BnoBuffer, L"%ws\\%lu\\BaseNamedObjects", SESSION_ROOT, Session) */
        idx = 0;
        wappend(&mut BnoBuffer, &mut idx, &SESSION_ROOT);
        wappend(&mut BnoBuffer, &mut idx, &[b'\\' as u16]);
        wappend(&mut BnoBuffer, &mut idx, &session_str[..session_len]);
        wappend(&mut BnoBuffer, &mut idx, &BASE_NAMED_OBJECTS);
        RtlInitUnicodeString(&mut BnoString, BnoBuffer.as_ptr());
    } else {
        RtlInitUnicodeString(&mut BnoString, BASE_NAMED_OBJECTS.as_ptr());
    }

    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = BNOLinksDirectory;
    ObjectAttributes.ObjectName = &mut SessionString;
    ObjectAttributes.Attributes = OBJ_OPENIF | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateSymbolicLinkObject(
        &mut BnoHandle,
        SYMBOLIC_LINK_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut BnoString,
    );
    if Status < 0 {
        return Status;
    }

    Status = GetDosDevicesProtection(&mut DosDevicesSd as *mut _ as PVOID);
    if Status < 0 {
        return Status;
    }

    /* _swprintf(SessionBuffer, L"%ws\\%lu", SESSION_ROOT, Session) */
    idx = 0;
    wappend(&mut SessionBuffer, &mut idx, &SESSION_ROOT);
    wappend(&mut SessionBuffer, &mut idx, &[b'\\' as u16]);
    wappend(&mut SessionBuffer, &mut idx, &session_str[..session_len]);
    RtlInitUnicodeString(&mut SessionString, SessionBuffer.as_ptr());

    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &mut SessionString;
    ObjectAttributes.Attributes = OBJ_OPENIF | OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = &mut DosDevicesSd as *mut _ as PSECURITY_DESCRIPTOR;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateDirectoryObject(
        &raw mut SessionObjectDirectory,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if Status < 0 {
        FreeDosDevicesProtection(&mut DosDevicesSd as *mut _ as PVOID);
        return Status;
    }

    RtlInitUnicodeString(&mut SessionString, DOS_DEVICES.as_ptr());
    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = SessionObjectDirectory;
    ObjectAttributes.ObjectName = &mut SessionString;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = &mut DosDevicesSd as *mut _ as PSECURITY_DESCRIPTOR;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateDirectoryObject(
        &raw mut DosDevicesDirectory,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    FreeDosDevicesProtection(&mut DosDevicesSd as *mut _ as PVOID);
    Status
}

/// Append `src` (up to its first NUL, if any) to `buf` at `*idx`, keeping `buf`
/// NUL-terminated; silently stops at the buffer end like a bounded wcscat.
fn wappend(buf: &mut [u16], idx: &mut usize, src: &[u16]) {
    for &c in src {
        if c == 0 || *idx + 1 >= buf.len() {
            break;
        }
        buf[*idx] = c;
        *idx += 1;
    }
    buf[*idx] = 0;
}

unsafe fn format_number(mut n: ULONG, buf: &mut [u16]) -> usize {
    if n == 0 {
        buf[0] = b'0' as u16;
        buf[1] = 0;
        return 1;
    }
    let mut digits: [u16; 16] = [0; 16];
    let mut idx = 0;
    while n > 0 {
        digits[idx] = (b'0' + (n % 10) as u8) as u16;
        n /= 10;
        idx += 1;
    }
    for i in 0..idx {
        buf[i] = digits[idx - 1 - i];
    }
    buf[idx] = 0;
    idx
}

unsafe extern "system" fn CsrParseServerCommandLine(
    ArgumentCount: ULONG,
    Arguments: *mut PCHAR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ParameterName: PCHAR;
    let mut ParameterValue: PCHAR;
    let mut EntryPoint: PCHAR;
    let mut ServerString: PCHAR;
    let mut DllIndex: ULONG = 0;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    CsrTotalPerProcessDataLength = 0;
    CsrObjectDirectory = core::ptr::null_mut();
    #[cfg(target_os = "windows")]
    {
        CsrWindowsControl = 0;
        CsrSubSystemType = IMAGE_SUBSYSTEM_UNKNOWN;
    }
    CsrMaxApiRequestThreads = 16;

    SessionId = (*NtCurrentPeb()).SessionId;
    Status = CsrCreateSessionObjectDirectory(SessionId);
    if Status < 0 {
        if SessionId != 0 {
            return Status;
        }
    }

    for i in 1..ArgumentCount {
        ParameterName = *Arguments.add(i as usize);
        ParameterValue = core::ptr::null_mut();

        let mut ptr = ParameterName;
        while *ptr != 0 {
            if *ptr == b'=' as i8 {
                *ptr = 0;
                ParameterValue = ptr.add(1);
                break;
            }
            ptr = ptr.add(1);
        }

        if stricmp(ParameterName, b"ObjectDirectory\0".as_ptr() as PCHAR) == 0 {
            if SessionId != 0 {
                continue;
            }

            RtlInitAnsiString(&mut AnsiString, ParameterValue);
            Status = RtlAnsiStringToUnicodeString(&raw mut CsrDirectoryName, &mut AnsiString, 1);
            if Status < 0 {
                return Status;
            }

            core::ptr::write_bytes(
                &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
                0,
                core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
            );
            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = &raw mut CsrDirectoryName;
            ObjectAttributes.Attributes = OBJ_OPENIF | OBJ_CASE_INSENSITIVE | OBJ_PERMANENT;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            Status = NtCreateDirectoryObject(
                &raw mut CsrObjectDirectory,
                DIRECTORY_ALL_ACCESS,
                &mut ObjectAttributes,
            );
            if Status < 0 {
                return Status;
            }

            Status = CsrSetDirectorySecurity(CsrObjectDirectory);
            if Status < 0 {
                return Status;
            }
        } else if stricmp(ParameterName, b"SubSystemType\0".as_ptr() as PCHAR) == 0 {
            #[cfg(target_os = "windows")]
            {
                if stricmp(ParameterValue, b"windows\0".as_ptr() as PCHAR) == 0 {
                    if CsrWindowsControl != 0 {
                        CsrSubSystemType = IMAGE_SUBSYSTEM_WINDOWS_GUI;
                    } else {
                        CsrSubSystemType = IMAGE_SUBSYSTEM_WINDOWS_CUI;
                    }
                } else if stricmp(ParameterValue, b"posix\0".as_ptr() as PCHAR) == 0 {
                    CsrSubSystemType = IMAGE_SUBSYSTEM_POSIX_CUI;
                } else if stricmp(ParameterValue, b"os2\0".as_ptr() as PCHAR) == 0 {
                    CsrSubSystemType = IMAGE_SUBSYSTEM_OS2_CUI;
                } else if stricmp(ParameterValue, b"native\0".as_ptr() as PCHAR) == 0 {
                    CsrSubSystemType = IMAGE_SUBSYSTEM_NATIVE;
                } else {
                    Status = RtlCharToInteger(ParameterValue, 0, &raw mut CsrSubSystemType);
                }
            }
        } else if stricmp(ParameterName, b"MaxRequestThreads\0".as_ptr() as PCHAR) == 0 {
            Status = RtlCharToInteger(ParameterValue, 0, &raw mut CsrMaxApiRequestThreads);
        } else if stricmp(ParameterName, b"RequestThreads\0".as_ptr() as PCHAR) == 0 {
            Status = STATUS_SUCCESS;
        } else if stricmp(ParameterName, b"ProfileControl\0".as_ptr() as PCHAR) == 0 {
        } else if stricmp(ParameterName, b"SharedSection\0".as_ptr() as PCHAR) == 0 {
            Status = CsrSrvCreateSharedSection(ParameterValue);
            if Status < 0 {
                return Status;
            }

            Status = CsrLoadServerDll(
                b"CSRSS\0".as_ptr() as PCHAR,
                core::ptr::null_mut(),
                CSRSRV_SERVERDLL_INDEX,
            );
        } else if stricmp(ParameterName, b"ServerDll\0".as_ptr() as PCHAR) == 0 {
            EntryPoint = core::ptr::null_mut();
            Status = STATUS_INVALID_PARAMETER;
            ServerString = ParameterValue;
            while *ServerString != 0 {
                if *ServerString == b':' as i8 && EntryPoint.is_null() {
                    *ServerString = 0;
                    ServerString = ServerString.add(1);
                    EntryPoint = ServerString;
                }
                if *ServerString == b',' as i8 {
                    ServerString = ServerString.add(1);
                    break;
                }
                ServerString = ServerString.add(1);
            }

            if *ServerString == 0 {
                return Status;
            }

            Status = RtlCharToInteger(ServerString, 10, &mut DllIndex);
            if Status >= 0 {
                *ServerString.sub(1) = 0;
            }

            if CsrDebug & 1 != 0 {}
            crate::dbg_print!(b"[CSI] LoadServerDll '%s' idx=%u ...\n\0", ParameterValue, DllIndex);
            Status = CsrLoadServerDll(ParameterValue, EntryPoint, DllIndex);
            crate::dbg_print!(b"[CSI] LoadServerDll '%s' done st=%lx\n\0", ParameterValue, Status);
            if Status < 0 {
                return Status;
            }
        } else if stricmp(ParameterName, b"Windows\0".as_ptr() as PCHAR) == 0 {
            #[cfg(target_os = "windows")]
            {
                CsrWindowsControl = if stricmp(ParameterValue, b"On\0".as_ptr() as PCHAR) == 0 {
                    1
                } else {
                    0
                };
            }
        } else {
            Status = STATUS_INVALID_PARAMETER;
        }
    }

    Status
}

unsafe fn stricmp(a: PCHAR, b: PCHAR) -> i32 {
    let mut pa = a;
    let mut pb = b;
    loop {
        let ca = to_lower(*pa as u8);
        let cb = to_lower(*pb as u8);
        if ca != cb {
            return (ca as i32) - (cb as i32);
        }
        if ca == 0 {
            return 0;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
}

unsafe fn to_lower(c: u8) -> u8 {
    if c >= b'A' && c <= b'Z' {
        c + 32
    } else {
        c
    }
}

unsafe extern "system" fn CsrInitCsrRootProcess() -> NTSTATUS {
    let mut ProcessData: PVOID;
    let mut ServerDll: PCSR_SERVER_DLL;

    ProcessData = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        (CsrTotalPerProcessDataLength as usize).try_into().unwrap(),
    );
    if ProcessData.is_null() {
        return STATUS_NO_MEMORY;
    }

    // ServerData is a C flexible array (`PVOID ServerData[ANYSIZE_ARRAY]`);
    // bindgen emits it as `[PVOID; 1]`. Index past 0 must use raw
    // pointer arithmetic — Rust's bounds-checked indexing would
    // panic-abort the process.
    let server_data_ptr = (*CsrRootProcess).ServerData.as_mut_ptr();
    for i in 0..CSR_SERVER_DLL_MAX {
        ServerDll = *CsrLoadedServerDll.as_ptr().add(i as usize);
        if !ServerDll.is_null() && (*ServerDll).SizeOfProcessData != 0 {
            *server_data_ptr.add(i as usize) = ProcessData;
            ProcessData = (ProcessData as usize + (*ServerDll).SizeOfProcessData as usize) as PVOID;
        } else {
            *server_data_ptr.add(i as usize) = core::ptr::null_mut();
        }
    }

    for i in 0..CSR_SERVER_DLL_MAX {
        ServerDll = *CsrLoadedServerDll.as_ptr().add(i as usize);
        if !ServerDll.is_null() && (*ServerDll).NewProcessCallback.is_some() {
            ((*ServerDll).NewProcessCallback.unwrap())(core::ptr::null_mut(), CsrRootProcess);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrCreateLocalSystemSD(
    LocalSystemSd: *mut PSECURITY_DESCRIPTOR,
) -> NTSTATUS {
    let NtSidAuthority = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut SystemSid: PSID = core::ptr::null_mut();
    let Length: ULONG;
    let SystemSd: PSECURITY_DESCRIPTOR;
    let Dacl: PACL;
    let mut Status: NTSTATUS;

    RtlAllocateAndInitializeSid(
        &NtSidAuthority as *const _ as *mut _,
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

    Length = RtlLengthSid(SystemSid)
        + core::mem::size_of::<ACL>() as u32
        + core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32;

    SystemSd = RtlAllocateHeap(
        CsrHeap,
        0,
        ((SECURITY_DESCRIPTOR_MIN_LENGTH + Length) as usize)
            .try_into()
            .unwrap(),
    ) as PSECURITY_DESCRIPTOR;

    Dacl = (SystemSd as usize + SECURITY_DESCRIPTOR_MIN_LENGTH as usize) as PACL;

    Status = RtlCreateSecurityDescriptor(SystemSd, SECURITY_DESCRIPTOR_REVISION);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, SystemSd as PVOID);
        RtlFreeSid(SystemSid);
        *LocalSystemSd = core::ptr::null_mut();
        return Status;
    }

    RtlCreateAcl(Dacl, Length, ACL_REVISION2);

    Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, PORT_ALL_ACCESS, SystemSid);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, SystemSd as PVOID);
        RtlFreeSid(SystemSid);
        *LocalSystemSd = core::ptr::null_mut();
        return Status;
    }

    Status = RtlSetDaclSecurityDescriptor(SystemSd, 1, Dacl, 0);
    if Status < 0 {
        RtlFreeHeap(CsrHeap, 0, SystemSd as PVOID);
        RtlFreeSid(SystemSid);
        *LocalSystemSd = core::ptr::null_mut();
        return Status;
    }

    RtlFreeSid(SystemSid);
    *LocalSystemSd = SystemSd;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrSbApiPortInitialize() -> NTSTATUS {
    let Size: ULONG;
    let mut PortSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut hRequestThread: HANDLE = core::ptr::null_mut();
    let mut ClientId: _CLIENT_ID = core::mem::zeroed();

    static SB_PORT_NAME: [u16; 11] = [
        b'S' as u16,
        b'b' as u16,
        b'A' as u16,
        b'p' as u16,
        b'i' as u16,
        b'P' as u16,
        b'o' as u16,
        b'r' as u16,
        b't' as u16,
        0,
        0,
    ];

    Size = CsrDirectoryName.Length as u32
        + core::mem::size_of_val(&SB_PORT_NAME) as u32
        + core::mem::size_of::<u16>() as u32;

    CsrSbApiPortName.Buffer =
        RtlAllocateHeap(CsrHeap, 0, (Size as usize).try_into().unwrap()) as *mut u16;
    if CsrSbApiPortName.Buffer.is_null() {
        return STATUS_NO_MEMORY;
    }

    CsrSbApiPortName.Length = 0;
    CsrSbApiPortName.MaximumLength = Size as u16;

    RtlAppendUnicodeStringToString(&raw mut CsrSbApiPortName, &raw const CsrDirectoryName);
    static SEP: [u16; 2] = [b'\\' as u16, 0];
    let mut sep_str: _UNICODE_STRING = core::mem::zeroed();
    sep_str.Buffer = SEP.as_ptr() as *mut u16;
    sep_str.Length = 2;
    sep_str.MaximumLength = 2;
    RtlAppendUnicodeStringToString(&raw mut CsrSbApiPortName, &sep_str);
    let mut sb_name: _UNICODE_STRING = core::mem::zeroed();
    sb_name.Buffer = SB_PORT_NAME.as_ptr() as *mut u16;
    sb_name.Length = 18;
    sb_name.MaximumLength = 20;
    RtlAppendUnicodeStringToString(&raw mut CsrSbApiPortName, &sb_name);

    if CsrDebug & 2 != 0 {}

    Status = CsrCreateLocalSystemSD(&mut PortSd);
    if Status < 0 {
        return Status;
    }

    core::ptr::write_bytes(
        &mut ObjectAttributes as *mut _OBJECT_ATTRIBUTES as *mut u8,
        0,
        core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
    );
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = &raw mut CsrSbApiPortName;
    ObjectAttributes.Attributes = 0;
    ObjectAttributes.SecurityDescriptor = PortSd;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreatePort(
        &raw mut CsrSbApiPort,
        &mut ObjectAttributes,
        core::mem::size_of::<SB_CONNECTION_INFO>() as ULONG,
        core::mem::size_of::<SB_API_MSG>() as ULONG,
        (32 * core::mem::size_of::<SB_API_MSG>()) as ULONG,
    );
    if !PortSd.is_null() {
        RtlFreeHeap(CsrHeap, 0, PortSd as PVOID);
    }

    if Status >= 0 {
        Status = RtlCreateUserThread(
            NtCurrentProcess(),
            core::ptr::null_mut(),
            1,
            0,
            0,
            0,
            core::mem::transmute(CsrSbApiRequestThread as usize),
            core::ptr::null_mut(),
            &mut hRequestThread,
            &mut ClientId,
        );
        if Status >= 0 {
            CsrSbApiRequestThreadPtr = CsrAddStaticServerThread(hRequestThread, &mut ClientId, 0);
            Status = NtResumeThread(hRequestThread, core::ptr::null_mut());
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrServerInitialization(
    ArgumentCount: ULONG,
    Arguments: *mut PCHAR,
) -> NTSTATUS {
    let mut Status = STATUS_SUCCESS;

    crate::dbg_print!(b"[CSI] 1 pre-NtQuerySystemInformation\n\0");
    Status = NtQuerySystemInformation(
        SystemBasicInformation,
        &raw mut CsrNtSysInfo as *mut _ as PVOID,
        core::mem::size_of::<SYSTEM_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    CsrHeap = RtlGetProcessHeap();

    crate::dbg_print!(b"[CSI] 2 pre-CsrSetProcessSecurity\n\0");
    Status = CsrSetProcessSecurity();
    if Status < 0 {
        return Status;
    }

    crate::dbg_print!(b"[CSI] 3 pre-CsrInitializeNtSessionList\n\0");
    Status = CsrInitializeNtSessionList();
    if Status < 0 {
        return Status;
    }

    crate::dbg_print!(b"[CSI] 4 pre-CsrInitializeProcessStructure\n\0");
    Status = CsrInitializeProcessStructure();
    if Status < 0 {
        return Status;
    }

    crate::dbg_print!(b"[CSI] 5 pre-CsrParseServerCommandLine\n\0");
    Status = CsrParseServerCommandLine(ArgumentCount, Arguments);
    if Status < 0 {
        return Status;
    }

    crate::dbg_print!(b"[CSI] 6 pre-CsrInitCsrRootProcess\n\0");
    Status = CsrInitCsrRootProcess();
    if Status < 0 {
        return Status;
    }

    crate::dbg_print!(b"[CSI] 7 pre-CsrApiPortInitialize\n\0");
    Status = CsrApiPortInitialize();
    if Status < 0 {
        return Status;
    }
    crate::dbg_print!(b"[CSI] 8 pre-SbApiPort/SmConnect block\n\0");

    #[cfg(target_os = "windows")]
    {
        #[cfg(not(target_arch = "aarch64"))]
        {
            if CsrSubSystemType != IMAGE_SUBSYSTEM_UNKNOWN {
                Status = CsrSbApiPortInitialize();
                if Status < 0 {
                    return Status;
                }

                Status = SmConnectToSm(
                    &raw mut CsrSbApiPortName,
                    CsrSbApiPort,
                    CsrSubSystemType,
                    &raw mut CsrSmApiPort,
                );
                if Status < 0 {
                    return Status;
                }
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            Status = CsrSbApiPortInitialize();
            if Status < 0 {
                return Status;
            }

            Status = SmConnectToSm(
                &raw mut CsrSbApiPortName,
                CsrSbApiPort,
                IMAGE_SUBSYSTEM_WINDOWS_GUI,
                &raw mut CsrSmApiPort,
            );
            if Status < 0 {
                return Status;
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Status = CsrSbApiPortInitialize();
        if Status < 0 {
            return Status;
        }

        Status = SmConnectToSm(
            &mut CsrSbApiPortName,
            CsrSbApiPort,
            IMAGE_SUBSYSTEM_WINDOWS_GUI,
            &mut CsrSmApiPort,
        );
        if Status < 0 {
            return Status;
        }
    }

    Status = NtSetDefaultHardErrorPort(CsrApiPort);
    if Status < 0 {
        return Status;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrPopulateDosDevices() {}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _hInstanceDll: HINSTANCE,
    _dwReason: DWORD,
    _lpReserved: LPVOID,
) -> BOOL {
    1
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Client/Server Runtime SubSystem
//  * FILE:            subsystems/win32/csrsrv/init.c
//  * PURPOSE:         CSR Server DLL Initialization
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "srv.h"
//
// #include <winreg.h>
// #include <ndk/cmfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* ARM64: CsrUartPuts is a no-op — user-mode cannot access kernel UART MMIO.
//  * Tracing is done from the kernel side via LPC port creation/connection logs. */
// #define CsrUartPuts(s) ((void)0)
//
// /* DATA ***********************************************************************/
//
// // Debug Flag
// ULONG CsrDebug = 0; // 0xFFFFFFFF;
//
// HANDLE CsrHeap = NULL;
// HANDLE CsrObjectDirectory = NULL;
// UNICODE_STRING CsrDirectoryName;
// UNICODE_STRING CsrSbApiPortName;
// HANDLE CsrSbApiPort = NULL;
// PCSR_THREAD CsrSbApiRequestThreadPtr;
// HANDLE CsrSmApiPort = NULL;
// HANDLE hSbApiPort = NULL;
// HANDLE CsrApiPort = NULL;
// #ifdef __REACTOS__
// BOOLEAN CsrWindowsControl;
// ULONG CsrSubSystemType; // Known as SessionFirstProcessImageType in Windows 7+
// #endif
// ULONG CsrMaxApiRequestThreads;
// ULONG CsrTotalPerProcessDataLength;
// ULONG SessionId;
// HANDLE BNOLinksDirectory;
// HANDLE SessionObjectDirectory;
// HANDLE DosDevicesDirectory;
// SYSTEM_BASIC_INFORMATION CsrNtSysInfo;
//
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /* === INIT ROUTINES === */
//
// /*++
//  * @name CsrSetProcessSecurity
//  *
//  * The CsrSetProcessSecurity routine protects access to the CSRSS process
//  * from unauthorized tampering.
//  *
//  * @param None.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSetProcessSecurity(VOID)
// {
//     NTSTATUS Status;
//     HANDLE hToken, hProcess = NtCurrentProcess();
//     ULONG Length;
//     PTOKEN_USER TokenInfo = NULL;
//     PSECURITY_DESCRIPTOR ProcSd = NULL;
//     PACL Dacl;
//     PSID UserSid;
//
//     /* Open our token */
//     Status = NtOpenProcessToken(hProcess, TOKEN_QUERY, &hToken);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Get the Token User Length */
//     Status = NtQueryInformationToken(hToken, TokenUser, NULL, 0, &Length);
//     if (Status != STATUS_BUFFER_TOO_SMALL)
//     {
//         NtClose(hToken);
//         goto Quickie;
//     }
//
//     /* Allocate space for it */
//     TokenInfo = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, Length);
//     if (!TokenInfo)
//     {
//         NtClose(hToken);
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* Now query the data */
//     Status = NtQueryInformationToken(hToken, TokenUser, TokenInfo, Length, &Length);
//     NtClose(hToken);
//     if (!NT_SUCCESS(Status)) goto Quickie;
//
//     /* Now check the SID Length */
//     UserSid = TokenInfo->User.Sid;
//     Length = RtlLengthSid(UserSid) + sizeof(ACL) + sizeof(ACCESS_ALLOWED_ACE);
//
//     /* Allocate a buffer for the Security Descriptor, with SID and DACL */
//     ProcSd = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, SECURITY_DESCRIPTOR_MIN_LENGTH + Length);
//     if (!ProcSd)
//     {
//         Status = STATUS_NO_MEMORY;
//         goto Quickie;
//     }
//
//     /* Set the pointer to the DACL */
//     Dacl = (PACL)((ULONG_PTR)ProcSd + SECURITY_DESCRIPTOR_MIN_LENGTH as usize);
//
//     /* Now create the SD itself */
//     Status = RtlCreateSecurityDescriptor(ProcSd, SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: SD creation failed - status = %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Create the DACL for it*/
//     Status = RtlCreateAcl(Dacl, Length, ACL_REVISION2);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: DACL creation failed - status = %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Create the ACE */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     PROCESS_VM_READ | PROCESS_VM_WRITE |
//                                     PROCESS_VM_OPERATION | PROCESS_DUP_HANDLE |
//                                     PROCESS_TERMINATE | PROCESS_SUSPEND_RESUME |
//                                     PROCESS_QUERY_INFORMATION | READ_CONTROL,
//                                     UserSid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: ACE creation failed - status = %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Clear the DACL in the SD */
//     Status = RtlSetDaclSecurityDescriptor(ProcSd, TRUE, Dacl, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: set DACL failed - status = %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Write the SD into the Process */
//     Status = NtSetSecurityObject(hProcess, DACL_SECURITY_INFORMATION, ProcSd);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: set process DACL failed - status = %lx\n", Status);
//         goto Quickie;
//     }
//
//     /* Free the memory and return */
// Quickie:
//     if (ProcSd) RtlFreeHeap(CsrHeap, 0, ProcSd);
//     if (TokenInfo) RtlFreeHeap(CsrHeap, 0, TokenInfo);
//     return Status;
// }
//
// /*++
//  * @name CsrSetDirectorySecurity
//  *
//  * The CsrSetDirectorySecurity routine sets the security descriptor for the
//  * specified Object Directory.
//  *
//  * @param ObjectDirectory
//  *        Handle fo the Object Directory to protect.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSetDirectorySecurity(IN HANDLE ObjectDirectory)
// {
//     /* FIXME: Implement */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name GetDosDevicesProtection
//  *
//  * The GetDosDevicesProtection creates a security descriptor for the DOS Devices
//  * Object Directory.
//  *
//  * @param DosDevicesSd
//  *        Pointer to the Security Descriptor to return.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks Depending on the DOS Devices Protection Mode (set in the registry),
//  *          regular users may or may not have full access to the directory.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// GetDosDevicesProtection(OUT PSECURITY_DESCRIPTOR DosDevicesSd)
// {
//     SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY CreatorAuthority = {SECURITY_CREATOR_SID_AUTHORITY};
//     SID_IDENTIFIER_AUTHORITY NtSidAuthority = {SECURITY_NT_AUTHORITY};
//     PSID WorldSid, CreatorSid, AdminSid, SystemSid;
//     UCHAR KeyValueBuffer[0x40];
//     PKEY_VALUE_PARTIAL_INFORMATION KeyValuePartialInfoLocal;
//     UNICODE_STRING KeyName;
//     ULONG ProtectionModeLocal = 0;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PACL Dacl;
//     PACCESS_ALLOWED_ACE Ace;
//     HANDLE hKey;
//     NTSTATUS Status;
//     ULONG ResultLength, SidLength, AclLength;
//
//     /* Create the SD */
//     Status = RtlCreateSecurityDescriptor(DosDevicesSd, SECURITY_DESCRIPTOR_REVISION);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Initialize the System SID */
//     Status = RtlAllocateAndInitializeSid(&NtSidAuthority, 1,
//                                          SECURITY_LOCAL_SYSTEM_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &SystemSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Initialize the World SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority, 1,
//                                          SECURITY_WORLD_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &WorldSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Initialize the Admin SID */
//     Status = RtlAllocateAndInitializeSid(&NtSidAuthority, 2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0, 0, 0, 0, 0, 0,
//                                          &AdminSid);
//     ASSERT(NT_SUCCESS(Status));
//
//     /* Initialize the Creator SID */
//     Status = RtlAllocateAndInitializeSid(&CreatorAuthority, 1,
//                                          SECURITY_CREATOR_OWNER_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &CreatorSid);
//     ASSERT(NT_SUCCESS(Status));
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
//         RtlInitUnicodeString(&KeyName, L"ProtectionModeLocal");
//         Status = NtQueryValueKey(hKey,
//                                  &KeyName,
//                                  KeyValuePartialInformation,
//                                  KeyValueBuffer,
//                                  sizeof(KeyValueBuffer),
//                                  &ResultLength);
//
//         /* Make sure it's what we expect it to be */
//         KeyValuePartialInfoLocal = (PKEY_VALUE_PARTIAL_INFORMATION)KeyValueBuffer;
//         if ((NT_SUCCESS(Status)) && (KeyValuePartialInfoLocal->Type == REG_DWORD) &&
//             (*(PULONG)KeyValuePartialInfoLocal->Data))
//         {
//             /* Save the Protection Mode */
//             ProtectionModeLocal = *(PULONG)KeyValuePartialInfoLocal->Data;
//         }
//
//         /* Close the handle */
//         NtClose(hKey);
//     }
//
//     /* Check the Protection Mode */
//     if (ProtectionModeLocal & 3)
//     {
//         /* Calculate SID Lengths */
//         SidLength = RtlLengthSid(CreatorSid) + RtlLengthSid(SystemSid) +
//                     RtlLengthSid(AdminSid);
//         AclLength = sizeof(ACL) + 3 * sizeof(ACCESS_ALLOWED_ACE) + SidLength;
//
//         /* Allocate memory for the DACL */
//         Dacl = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, AclLength);
//         ASSERT(Dacl != NULL);
//
//         /* Build the ACL and add 3 ACEs */
//         Status = RtlCreateAcl(Dacl, AclLength, ACL_REVISION2);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SystemSid);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, AdminSid);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, CreatorSid);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Edit the ACEs to make them inheritable */
//         Status = RtlGetAce(Dacl, 0, (PVOID*)&Ace);
//         ASSERT(NT_SUCCESS(Status));
//         Ace->Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as u8;
//         Status = RtlGetAce(Dacl, 1, (PVOID*)&Ace);
//         ASSERT(NT_SUCCESS(Status));
//         Ace->Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE) as u8;
//         Status = RtlGetAce(Dacl, 2, (PVOID*)&Ace);
//         ASSERT(NT_SUCCESS(Status));
//         Ace->Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
//
//         /* Set this DACL with the SD */
//         Status = RtlSetDaclSecurityDescriptor(DosDevicesSd, TRUE, Dacl, FALSE);
//         ASSERT(NT_SUCCESS(Status));
//         goto Quickie;
//     }
//     else
//     {
//         /* Calculate SID Lengths */
//         SidLength = RtlLengthSid(WorldSid) + RtlLengthSid(SystemSid);
//         AclLength = sizeof(ACL) + 3 * sizeof(ACCESS_ALLOWED_ACE) + SidLength;
//
//         /* Allocate memory for the DACL */
//         Dacl = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, AclLength);
//         ASSERT(Dacl != NULL);
//
//         /* Build the ACL and add 3 ACEs */
//         Status = RtlCreateAcl(Dacl, AclLength, ACL_REVISION2);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_READ | GENERIC_WRITE | GENERIC_EXECUTE, WorldSid);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, SystemSid);
//         ASSERT(NT_SUCCESS(Status));
//         Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, GENERIC_ALL, WorldSid);
//         ASSERT(NT_SUCCESS(Status));
//
//         /* Edit the last ACE to make it inheritable */
//         Status = RtlGetAce(Dacl, 2, (PVOID*)&Ace);
//         ASSERT(NT_SUCCESS(Status));
//         Ace->Header.AceFlags |= (OBJECT_INHERIT_ACE | CONTAINER_INHERIT_ACE | INHERIT_ONLY_ACE) as u8;
//
//         /* Set this DACL with the SD */
//         Status = RtlSetDaclSecurityDescriptor(DosDevicesSd, TRUE, Dacl, FALSE);
//         ASSERT(NT_SUCCESS(Status));
//         goto Quickie;
//     }
//
// /* FIXME: failure cases! Fail: */
//     /* Free the memory */
//     RtlFreeHeap(CsrHeap, 0, Dacl);
//
// /* FIXME: semi-failure cases! Quickie: */
// Quickie:
//     /* Free the SIDs */
//     RtlFreeSid(CreatorSid);
//     RtlFreeSid(AdminSid);
//     RtlFreeSid(WorldSid);
//     RtlFreeSid(SystemSid);
//
//     /* Return */
//     return Status;
// }
//
// /*++
//  * @name FreeDosDevicesProtection
//  *
//  * The FreeDosDevicesProtection frees the security descriptor that was created
//  * by GetDosDevicesProtection
//  *
//  * @param DosDevicesSd
//  *        Pointer to the security descriptor to free.
//
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// FreeDosDevicesProtection(IN PSECURITY_DESCRIPTOR DosDevicesSd)
// {
//     PACL Dacl;
//     BOOLEAN Present, Default;
//     NTSTATUS Status;
//
//     /* Get the DACL corresponding to this SD */
//     Status = RtlGetDaclSecurityDescriptor(DosDevicesSd, &Present, &Dacl, &Default);
//     ASSERT(NT_SUCCESS(Status));
//     ASSERT(Present);
//     ASSERT(Dacl != NULL);
//
//     /* Free it */
//     if ((NT_SUCCESS(Status)) && (Dacl)) RtlFreeHeap(CsrHeap, 0, Dacl);
// }
//
// /*++
//  * @name CsrCreateSessionObjectDirectory
//  *
//  * The CsrCreateSessionObjectDirectory routine creates the BaseNamedObjects,
//  * Session and Dos Devices directories for the specified session.
//  *
//  * @param Session
//  *        Session ID for which to create the directories.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrCreateSessionObjectDirectory(IN ULONG Session)
// {
//     WCHAR SessionBuffer[512], BnoBuffer[512];
//     UNICODE_STRING SessionString, BnoString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE BnoHandle;
//     SECURITY_DESCRIPTOR DosDevicesSd;
//     NTSTATUS Status;
//
//     /* Generate the Session BNOLINKS Directory name */
//     swprintf(SessionBuffer, L"%ws\\BNOLINKS", SESSION_ROOT);
//     RtlInitUnicodeString(&SessionString, SessionBuffer);
//
//     /* Create it */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SessionString,
//                                OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtCreateDirectoryObject(&BNOLinksDirectory,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: NtCreateDirectoryObject failed in "
//                 "CsrCreateSessionObjectDirectory - status = %lx\n", Status);
//         return Status;
//     }
//
//     /* Now add the Session ID */
//     swprintf(SessionBuffer, L"%ld", Session);
//     RtlInitUnicodeString(&SessionString, SessionBuffer);
//
//     /* Check if this is the first Session */
//     if (Session)
//     {
//         /* Not the first, so the name will be slighly more complex */
//         swprintf(BnoBuffer, L"%ws\\%ld\\BaseNamedObjects", SESSION_ROOT, Session);
//         RtlInitUnicodeString(&BnoString, BnoBuffer);
//     }
//     else
//     {
//         /* Use the direct name */
//         RtlInitUnicodeString(&BnoString, L"\\BaseNamedObjects");
//     }
//
//     /* Create the symlink */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SessionString,
//                                OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
//                                BNOLinksDirectory,
//                                NULL);
//     Status = NtCreateSymbolicLinkObject(&BnoHandle,
//                                         SYMBOLIC_LINK_ALL_ACCESS,
//                                         &ObjectAttributes,
//                                         &BnoString);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: NtCreateSymbolicLinkObject failed in "
//                 "CsrCreateSessionObjectDirectory - status = %lx\n", Status);
//         return Status;
//     }
//
//     /* Create the \DosDevices Security Descriptor */
//     Status = GetDosDevicesProtection(&DosDevicesSd);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Now create a directory for this session */
//     swprintf(SessionBuffer, L"%ws\\%ld", SESSION_ROOT, Session);
//     RtlInitUnicodeString(&SessionString, SessionBuffer);
//
//     /* Create the directory */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SessionString,
//                                OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
//                                0,
//                                &DosDevicesSd);
//     Status = NtCreateDirectoryObject(&SessionObjectDirectory,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: NtCreateDirectoryObject failed in "
//                 "CsrCreateSessionObjectDirectory - status = %lx\n", Status);
//         FreeDosDevicesProtection(&DosDevicesSd);
//         return Status;
//     }
//
//     /* Next, create a directory for this session's DOS Devices */
//     RtlInitUnicodeString(&SessionString, L"DosDevices");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SessionString,
//                                OBJ_CASE_INSENSITIVE,
//                                SessionObjectDirectory,
//                                &DosDevicesSd);
//     Status = NtCreateDirectoryObject(&DosDevicesDirectory,
//                                      DIRECTORY_ALL_ACCESS,
//                                      &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: NtCreateDirectoryObject failed in "
//                 "CsrCreateSessionObjectDirectory - status = %lx\n", Status);
//     }
//
//     /* Release the Security Descriptor */
//     FreeDosDevicesProtection(&DosDevicesSd);
//
//     /* Return */
//     return Status;
// }
//
// /*++
//  * @name CsrParseServerCommandLine
//  *
//  * The CsrParseServerCommandLine routine parses the CSRSS command-line in the
//  * registry and performs operations for each entry found.
//  *
//  * @param ArgumentCount
//  *        Number of arguments on the command line.
//  *
//  * @param Arguments
//  *        Array of arguments.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrParseServerCommandLine(IN ULONG ArgumentCount,
//                           IN PCHAR Arguments[])
// {
//     NTSTATUS Status;
//     PCHAR ParameterName = NULL, ParameterValue = NULL, EntryPoint, ServerString;
//     ULONG i, DllIndex;
//     ANSI_STRING AnsiString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Set the Defaults */
//     CsrTotalPerProcessDataLength = 0;
//     CsrObjectDirectory = NULL;
// #ifdef __REACTOS__
//     CsrWindowsControl = FALSE;
//     CsrSubSystemType = IMAGE_SUBSYSTEM_UNKNOWN;
// #endif
//     CsrMaxApiRequestThreads = 16;
//
//     /* Save our Session ID, and create a Directory for it */
//     SessionId = NtCurrentPeb()->SessionId;
//     Status = CsrCreateSessionObjectDirectory(SessionId);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSS: CsrCreateSessionObjectDirectory failed (%lx)\n",
//                 Status);
//
//         /* It's not fatal if the session ID isn't zero */
//         if (SessionId != 0) return Status;
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Loop through every argument */
//     for (i = 1; i < ArgumentCount; i++)
//     {
//         /* Split Name and Value */
//         ParameterName = Arguments[i];
//         ParameterValue = NULL;
//         ParameterValue = strchr(ParameterName, '=');
//         if (ParameterValue) *ParameterValue++ = ANSI_NULL;
//         DPRINT("Name=%s, Value=%s\n", ParameterName, ParameterValue);
//
//         /* Check for Object Directory */
//         if (_stricmp(ParameterName, "ObjectDirectory") == 0)
//         {
//             /* Check if a session ID is specified */
//             if (SessionId != 0)
//             {
//                 DPRINT1("Sessions not yet implemented\n");
//                 ASSERT(SessionId);
//             }
//
//             /* Initialize the directory name */
//             RtlInitAnsiString(&AnsiString, ParameterValue);
//             Status = RtlAnsiStringToUnicodeString(&CsrDirectoryName,
//                                                   &AnsiString,
//                                                   TRUE);
//             ASSERT(NT_SUCCESS(Status) || SessionId != 0);
//             if (!NT_SUCCESS(Status)) return Status;
//
//             /* Create it */
//             InitializeObjectAttributes(&ObjectAttributes,
//                                        &CsrDirectoryName,
//                                        OBJ_OPENIF | OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                        NULL,
//                                        NULL);
//             Status = NtCreateDirectoryObject(&CsrObjectDirectory,
//                                              DIRECTORY_ALL_ACCESS,
//                                              &ObjectAttributes);
//             if (!NT_SUCCESS(Status)) return Status;
//
//             /* Secure it */
//             Status = CsrSetDirectorySecurity(CsrObjectDirectory);
//             if (!NT_SUCCESS(Status)) return Status;
//         }
//         else if (_stricmp(ParameterName, "SubSystemType") == 0)
//         {
// #ifdef __REACTOS__
//             /* Well-known subsystems, specified by names */
//             if (_stricmp(ParameterValue, "windows") == 0)
//             {
//                 /* Behaviour compatible with Windows 7+ */
//                 if (CsrWindowsControl)
//                     CsrSubSystemType = IMAGE_SUBSYSTEM_WINDOWS_GUI;
//                 else
//                     CsrSubSystemType = IMAGE_SUBSYSTEM_WINDOWS_CUI;
//             }
//             else if (_stricmp(ParameterValue, "posix") == 0)
//             {
//                 CsrSubSystemType = IMAGE_SUBSYSTEM_POSIX_CUI;
//             }
//             else if (_stricmp(ParameterValue, "os2") == 0)
//             {
//                 CsrSubSystemType = IMAGE_SUBSYSTEM_OS2_CUI;
//             }
//             else if (_stricmp(ParameterValue, "native") == 0)
//             {
//                 CsrSubSystemType = IMAGE_SUBSYSTEM_NATIVE;
//             }
//             else
//             {
//                 /* Another subsystem type, specified by a numerical value */
//                 Status = RtlCharToInteger(ParameterValue, 0, &CsrSubSystemType);
//             }
// #else
//             /* Ignored */
// #endif
//         }
//         else if (_stricmp(ParameterName, "MaxRequestThreads") == 0)
//         {
//             Status = RtlCharToInteger(ParameterValue,
//                                       0,
//                                       &CsrMaxApiRequestThreads);
//         }
//         else if (_stricmp(ParameterName, "RequestThreads") == 0)
//         {
//             /* Ignored */
//             Status = STATUS_SUCCESS;
//         }
//         else if (_stricmp(ParameterName, "ProfileControl") == 0)
//         {
//             /* Related functionality ignored since NT 3.5 */
//         }
//         else if (_stricmp(ParameterName, "SharedSection") == 0)
//         {
//             /* Create the Section */
//             Status = CsrSrvCreateSharedSection(ParameterValue);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("CSRSS: *** Invalid syntax for %s=%s (Status == %X)\n",
//                         ParameterName, ParameterValue, Status);
//                 return Status;
//             }
//
//             /* Load us */
//             Status = CsrLoadServerDll("CSRSS" /* "CSRSRV" */, NULL, CSRSRV_SERVERDLL_INDEX);
//         }
//         else if (_stricmp(ParameterName, "ServerDll") == 0)
//         {
//             /* Loop the command line */
//             EntryPoint = NULL;
//             Status = STATUS_INVALID_PARAMETER;
//             ServerString = ParameterValue;
//             while (*ServerString)
//             {
//                 /* Check for the Entry Point */
//                 if ((*ServerString == ':') && (!EntryPoint))
//                 {
//                     /* Found it. NULL-terminate and save it. */
//                     *ServerString++ = ANSI_NULL;
//                     EntryPoint = ServerString;
//                 }
//
//                 /* Check for the Dll Index */
//                 if (*ServerString++ == ',') break;
//             }
//
//             /* Did we find something to load? */
//             if (!*ServerString)
//             {
//                 DPRINT1("CSRSS: *** Invalid syntax for ServerDll=%s (Status == %X)\n",
//                         ParameterValue, Status);
//                 return Status;
//             }
//
//             /* Convert it to a ULONG */
//             Status = RtlCharToInteger(ServerString, 10, &DllIndex);
//
//             /* Add a null char if it was valid */
//             if (NT_SUCCESS(Status)) ServerString[-1] = ANSI_NULL;
//
//             /* Load it */
//             if (CsrDebug & 1) DPRINT1("CSRSS: Loading ServerDll=%s:%s\n", ParameterValue, EntryPoint);
//             Status = CsrLoadServerDll(ParameterValue, EntryPoint, DllIndex);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("CSRSS: *** Failed loading ServerDll=%s (Status == 0x%x)\n",
//                         ParameterValue, Status);
//                 return Status;
//             }
//         }
//         else if (_stricmp(ParameterName, "Windows") == 0)
//         {
// #ifdef __REACTOS__
//             CsrWindowsControl = (_stricmp(ParameterValue, "On") == 0);
// #else
//             /* Ignored */
// #endif
//         }
//         else
//         {
//             /* Invalid parameter on the command line */
//             Status = STATUS_INVALID_PARAMETER;
//         }
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name CsrInitCsrRootProcess
//  *
//  * The CsrInitCsrRootProcess routine further initializes the CSR Root Process
//  * created by CsrInitializeProcessStructure, by allocating and initializing
//  * per-process data for each Server DLL.
//  *
//  * @param None.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrInitCsrRootProcess(VOID)
// {
//     PVOID ProcessData;
//     PCSR_SERVER_DLL ServerDll;
//     ULONG i = 0;
//
//     /* All Server DLLs are now loaded, allocate a heap for the Root Process */
//     ProcessData = RtlAllocateHeap(CsrHeap,
//                                   HEAP_ZERO_MEMORY,
//                                   CsrTotalPerProcessDataLength);
//     if (!ProcessData)
//     {
//         DPRINT1("CSRSRV:%s: RtlAllocateHeap failed (Status=%08lx)\n",
//                 __FUNCTION__, STATUS_NO_MEMORY);
//         return STATUS_NO_MEMORY;
//     }
//
//     /*
//      * Our Root Process was never officially initialized,
//      * so write the data for each Server DLL manually.
//      */
//
//     /* Loop every DLL */
//     for (i = 0; i < CSR_SERVER_DLL_MAX; i++)
//     {
//         /* Get the current Server */
//         ServerDll = CsrLoadedServerDll[i];
//
//         /* Is it loaded, and does it have per process data? */
//         if (ServerDll && ServerDll->SizeOfProcessData)
//         {
//             /* It does, give it part of our allocated heap */
//             CsrRootProcess->ServerData[i] = ProcessData;
//
//             /* Move to the next heap position */
//             ProcessData = (PVOID)((ULONG_PTR)ProcessData +
//                                   ServerDll->SizeOfProcessData);
//         }
//         else
//         {
//             /* Nothing for this Server DLL */
//             CsrRootProcess->ServerData[i] = NULL;
//         }
//     }
//
//     /* Now initialize the Root Process manually as well */
//     for (i = 0; i < CSR_SERVER_DLL_MAX; i++)
//     {
//         /* Get the current Server */
//         ServerDll = CsrLoadedServerDll[i];
//
//         /* Is it loaded, and does it a callback for new processes? */
//         if (ServerDll && ServerDll->NewProcessCallback)
//         {
//             /* Call the callback */
//             ServerDll->NewProcessCallback(NULL, CsrRootProcess);
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrCreateLocalSystemSD
//  *
//  * The CsrCreateLocalSystemSD routine creates a Security Descriptor for
//  * the local account with PORT_ALL_ACCESS.
//  *
//  * @param LocalSystemSd
//  *        Pointer to a pointer to the security descriptor to create.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrCreateLocalSystemSD(OUT PSECURITY_DESCRIPTOR *LocalSystemSd)
// {
//     SID_IDENTIFIER_AUTHORITY NtSidAuthority = {SECURITY_NT_AUTHORITY};
//     PSID SystemSid;
//     ULONG Length;
//     PSECURITY_DESCRIPTOR SystemSd;
//     PACL Dacl;
//     NTSTATUS Status;
//
//     /* Initialize the System SID */
//     RtlAllocateAndInitializeSid(&NtSidAuthority, 1,
//                                 SECURITY_LOCAL_SYSTEM_RID,
//                                 0, 0, 0, 0, 0, 0, 0,
//                                 &SystemSid);
//
//     /* Get the length of the SID */
//     Length = RtlLengthSid(SystemSid) + sizeof(ACL) + sizeof(ACCESS_ALLOWED_ACE);
//
//     /* Allocate a buffer for the Security Descriptor, with SID and DACL */
//     SystemSd = RtlAllocateHeap(CsrHeap, 0, SECURITY_DESCRIPTOR_MIN_LENGTH + Length);
//
//     /* Set the pointer to the DACL */
//     Dacl = (PACL)((ULONG_PTR)SystemSd + SECURITY_DESCRIPTOR_MIN_LENGTH as usize);
//
//     /* Now create the SD itself */
//     Status = RtlCreateSecurityDescriptor(SystemSd, SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status)) goto Quit;
//
//     /* Create the DACL for it */
//     RtlCreateAcl(Dacl, Length, ACL_REVISION2);
//
//     /* Create the ACE */
//     Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, PORT_ALL_ACCESS, SystemSid);
//     if (!NT_SUCCESS(Status)) goto Quit;
//
//     /* Clear the DACL in the SD */
//     Status = RtlSetDaclSecurityDescriptor(SystemSd, TRUE, Dacl, FALSE);
//     if (!NT_SUCCESS(Status)) goto Quit;
//
// Quit:
//     if (!NT_SUCCESS(Status))
//     {
//         RtlFreeHeap(CsrHeap, 0, SystemSd);
//         SystemSd = NULL;
//     }
//
//     /* Free the SID and return*/
//     RtlFreeSid(SystemSid);
//     *LocalSystemSd = SystemSd;
//     return Status;
// }
//
// /*++
//  * @name CsrSbApiPortInitialize
//  *
//  * The CsrSbApiPortInitialize routine initializes the LPC Port used for
//  * communications with the Session Manager (SM) and initializes the static
//  * thread that will handle connection requests and APIs.
//  *
//  * @param None
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSbApiPortInitialize(VOID)
// {
//     ULONG Size;
//     PSECURITY_DESCRIPTOR PortSd;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE hRequestThread;
//     CLIENT_ID ClientId;
//
//     /* Calculate how much space we'll need for the Port Name */
//     Size = CsrDirectoryName.Length + sizeof(SB_PORT_NAME) + sizeof(WCHAR);
//
//     /* Create the buffer for it */
//     CsrSbApiPortName.Buffer = RtlAllocateHeap(CsrHeap, 0, Size);
//     if (!CsrSbApiPortName.Buffer) return STATUS_NO_MEMORY;
//
//     /* Setup the rest of the empty string */
//     CsrSbApiPortName.Length = 0;
//     CsrSbApiPortName.MaximumLength = (USHORT)Size;
//
//     /* Now append the full port name */
//     RtlAppendUnicodeStringToString(&CsrSbApiPortName, &CsrDirectoryName);
//     RtlAppendUnicodeToString(&CsrSbApiPortName, UNICODE_PATH_SEP);
//     RtlAppendUnicodeToString(&CsrSbApiPortName, SB_PORT_NAME);
//     if (CsrDebug & 2) DPRINT1("CSRSS: Creating %wZ port and associated thread\n", &CsrSbApiPortName);
//
//     /* Create Security Descriptor for this Port */
//     Status = CsrCreateLocalSystemSD(&PortSd);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Initialize the Attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &CsrSbApiPortName,
//                                0,
//                                NULL,
//                                PortSd);
//
//     /* Create the Port Object */
//     Status = NtCreatePort(&CsrSbApiPort,
//                           &ObjectAttributes,
//                           sizeof(SB_CONNECTION_INFO),
//                           sizeof(SB_API_MSG),
//                           32 * sizeof(SB_API_MSG));
//     if (PortSd) RtlFreeHeap(CsrHeap, 0, PortSd);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* Create the Thread to handle the API Requests */
//         Status = RtlCreateUserThread(NtCurrentProcess(),
//                                      NULL,
//                                      TRUE,
//                                      0,
//                                      0,
//                                      0,
//                                      (PVOID)CsrSbApiRequestThread,
//                                      NULL,
//                                      &hRequestThread,
//                                      &ClientId);
//         if (NT_SUCCESS(Status))
//         {
//             /* Add it as a Static Server Thread */
//             CsrSbApiRequestThreadPtr = CsrAddStaticServerThread(hRequestThread,
//                                                                 &ClientId,
//                                                                 0);
//
//             /* Activate it */
//             Status = NtResumeThread(hRequestThread, NULL);
//         }
//     }
//
//     return Status;
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*++
//  * @name CsrServerInitialization
//  * @implemented NT4
//  *
//  * The CsrServerInitialization routine is the native (not Server) entrypoint
//  * of this Server DLL. It serves as the entrypoint for CSRSS.
//  *
//  * @param ArgumentCount
//  *        Number of arguments on the command line.
//  *
//  * @param Arguments
//  *        Array of arguments from the command line.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrServerInitialization(IN ULONG ArgumentCount,
//                         IN PCHAR Arguments[])
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Cache System Basic Information so we don't always request it */
//     Status = NtQuerySystemInformation(SystemBasicInformation,
//                                       &CsrNtSysInfo,
//                                       sizeof(SYSTEM_BASIC_INFORMATION),
//                                       NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSRV:%s: NtQuerySystemInformation failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//
//     /* Save our Heap */
//     CsrHeap = RtlGetProcessHeap();
//
//     /* Set our Security Descriptor to protect the process */
//     Status = CsrSetProcessSecurity();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSRV:%s: CsrSetProcessSecurity failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//
//     /* Set up Session Support */
//     Status = CsrInitializeNtSessionList();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSRV:%s: CsrInitializeSessions failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//
//     /* Set up Process Support and allocate the CSR Root Process */
//     Status = CsrInitializeProcessStructure();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSRV:%s: CsrInitializeProcessStructure failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//
//     /* Parse the command line */
//     CsrUartPuts("[CSRSRV] CsrParseServerCommandLine...\r\n");
//     Status = CsrParseServerCommandLine(ArgumentCount, Arguments);
//     if (!NT_SUCCESS(Status))
//     {
//         CsrUartPuts("[CSRSRV] CsrParseServerCommandLine FAILED\r\n");
//         DPRINT1("CSRSRV:%s: CsrParseServerCommandLine failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//     CsrUartPuts("[CSRSRV] CsrParseServerCommandLine OK\r\n");
//
//     /* Finish to initialize the CSR Root Process */
//     Status = CsrInitCsrRootProcess();
//     if (!NT_SUCCESS(Status))
//     {
//         CsrUartPuts("[CSRSRV] CsrInitCsrRootProcess FAILED\r\n");
//         DPRINT1("CSRSRV:%s: CsrInitCsrRootProcess failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//     CsrUartPuts("[CSRSRV] CsrInitCsrRootProcess OK\r\n");
//
//     /* Now initialize our API Port */
//     Status = CsrApiPortInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         CsrUartPuts("[CSRSRV] CsrApiPortInitialize FAILED\r\n");
//         DPRINT1("CSRSRV:%s: CsrApiPortInitialize failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//     CsrUartPuts("[CSRSRV] CsrApiPortInitialize OK\r\n");
//
// #ifdef __REACTOS__
// #ifndef _M_ARM64
//     if (CsrSubSystemType != IMAGE_SUBSYSTEM_UNKNOWN)
//     {
// #endif
// #endif
//     /* Initialize the API Port for SM communication */
//     CsrUartPuts("[CSRSRV] CsrSbApiPortInitialize...\r\n");
//     Status = CsrSbApiPortInitialize();
//     if (!NT_SUCCESS(Status))
//     {
//         CsrUartPuts("[CSRSRV] CsrSbApiPortInitialize FAILED\r\n");
//         DPRINT1("CSRSRV:%s: CsrSbApiPortInitialize failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//     CsrUartPuts("[CSRSRV] CsrSbApiPortInitialize OK\r\n");
//
//     /* We're all set! Connect to SM! */
//     CsrUartPuts("[CSRSRV] SmConnectToSm...\r\n");
//     Status = SmConnectToSm(&CsrSbApiPortName,
//                            CsrSbApiPort,
// #ifdef __REACTOS__
// #ifdef _M_ARM64
//                            IMAGE_SUBSYSTEM_WINDOWS_GUI,
// #else
//                            CsrSubSystemType,
// #endif
// #else
//                            IMAGE_SUBSYSTEM_WINDOWS_GUI,
// #endif
//                            &CsrSmApiPort);
//     if (!NT_SUCCESS(Status))
//     {
//         CsrUartPuts("[CSRSRV] SmConnectToSm FAILED\r\n");
//         DPRINT1("CSRSRV:%s: SmConnectToSm failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
// #ifdef __REACTOS__
// #ifndef _M_ARM64
//     }
// #endif
// #endif
//
//     /* Have us handle Hard Errors */
//     Status = NtSetDefaultHardErrorPort(CsrApiPort);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CSRSRV:%s: NtSetDefaultHardErrorPort failed (Status=0x%08lx)\n",
//                 __FUNCTION__, Status);
//         return Status;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name CsrPopulateDosDevices
//  * @unimplemented NT5.1
//  *
//  * The CsrPopulateDosDevices routine uses the DOS Device Map from the Kernel
//  * to populate the Dos Devices Object Directory for the session.
//  *
//  * @param None.
//  *
//  * @return None.
//  *
//  * @remarks None.
//  *
//  *--*/
// VOID
// NTAPI
// CsrPopulateDosDevices(VOID)
// {
//     DPRINT1("Deprecated API in r55585.\n");
//     return;
// }
//
// BOOL
// NTAPI
// DllMain(IN HINSTANCE hInstanceDll,
//         IN DWORD dwReason,
//         IN LPVOID lpReserved)
// {
//     /* We don't do much */
//     UNREFERENCED_PARAMETER(hInstanceDll);
//     UNREFERENCED_PARAMETER(dwReason);
//     UNREFERENCED_PARAMETER(lpReserved);
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
