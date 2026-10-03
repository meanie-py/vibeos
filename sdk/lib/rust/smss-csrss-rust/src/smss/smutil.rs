use ntdll_sys::nt::*;

// RtlConvertUlongToLuid is FORCEINLINE in ndk/rtlfuncs.h.
#[inline(always)]
unsafe fn RtlConvertUlongToLuid(u: ULONG) -> LUID {
    LUID {
        LowPart: u,
        HighPart: 0,
    }
}

// InitializeObjectAttributes is a C macro — inline.
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

const VALUE_BUFFER_SIZE: usize = core::mem::size_of::<_KEY_VALUE_PARTIAL_INFORMATION>() + 512;

#[repr(C)]
struct SMP_PRIVILEGE_STATE {
    TokenHandle: HANDLE,
    OldPrivileges: PTOKEN_PRIVILEGES,
    NewPrivileges: PTOKEN_PRIVILEGES,
    OldBuffer: [u8; 1024],
    NewBuffer: TOKEN_PRIVILEGES,
}

#[no_mangle]
pub static mut SmpDebugKeyword: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpASyncKeyword: _UNICODE_STRING = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut SmpAutoChkKeyword: _UNICODE_STRING = unsafe { core::mem::zeroed() };

// NtCurrentProcess is a macro: pseudo-handle `(HANDLE)-1`.
#[inline(always)]
unsafe fn NtCurrentProcess() -> HANDLE {
    -1isize as HANDLE
}

// KUSER_SHARED_DATA lives at a fixed user-mode VA on Windows / ReactOS.
// Expose as a const pointer so call sites that did `(*SharedUserData).Field`
// or `SharedUserData->Field` (translated to Rust `(*SharedUserData).Field`)
// continue to work — the const is the address itself, deref reaches the
// struct mapped by the kernel at boot.
#[allow(non_upper_case_globals)]
pub(crate) const SharedUserData: *mut _KUSER_SHARED_DATA = 0x7FFE0000 as *mut _KUSER_SHARED_DATA;

// Data globals live in sminit.c (still C) — convention-free import.
extern "C" {
    static mut SmpHeap: PVOID;
    static SmBaseTag: ULONG;
    static mut SmpDefaultLibPath: _UNICODE_STRING;
    static mut SmpDefaultEnvironment: PVOID;
}
// SmpSystemRoot is defined in this crate (smss/smss.rs) as a no_mangle
// pub static — use it directly via `crate::smss::smss::SmpSystemRoot`.
use crate::smss::smss::SmpSystemRoot;

const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const TOKEN_QUERY: u32 = 0x0008;
const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;
const STATUS_NOT_ALL_ASSIGNED: NTSTATUS = 0x00000106;
const STATUS_PRIVILEGE_NOT_HELD: NTSTATUS = 0xC0000061u32 as NTSTATUS;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;
const STATUS_OBJECT_NAME_NOT_FOUND: NTSTATUS = 0xC0000034u32 as NTSTATUS;
const STATUS_OBJECT_PATH_INVALID: NTSTATUS = 0xC0000039u32 as NTSTATUS;
const KEY_READ: u32 = 0x20019;
const OBJ_CASE_INSENSITIVE: u32 = 0x00000040;
const REG_SZ: u32 = 1;
const REG_EXPAND_SZ: u32 = 2;
const UNICODE_NULL: u16 = 0;
const MAX_PATH: usize = 260;

// SMP_*_FLAG come from `ntdll_sys::nt::*` (bindgen of smss.h). The local
// SMP_INVALID_PATH here was 0x08 — WRONG (the real value is 0x10; 0x08 is
// SMP_SUBSYSTEM_FLAG). A stray *Flags |= SMP_INVALID_PATH would have set the
// subsystem bit instead. See the note in smss.rs (task #18).

#[no_mangle]
pub unsafe extern "system" fn SmpAcquirePrivilege(
    Privilege: ULONG,
    PrivilegeState: *mut PVOID,
) -> NTSTATUS {
    let State: *mut SMP_PRIVILEGE_STATE;
    let mut Size: ULONG;
    let mut Status: NTSTATUS;

    *PrivilegeState = core::ptr::null_mut();

    State = RtlAllocateHeap(
        SmpHeap,
        0,
        (core::mem::size_of::<SMP_PRIVILEGE_STATE>()
            + core::mem::size_of::<TOKEN_PRIVILEGES>()
            + core::mem::size_of::<LUID_AND_ATTRIBUTES>())
        .try_into()
        .unwrap(),
    ) as *mut SMP_PRIVILEGE_STATE;
    if State.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = NtOpenProcessToken(
        NtCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
        core::ptr::addr_of_mut!((*State).TokenHandle),
    );
    if Status < 0 {
        RtlFreeHeap(SmpHeap, 0, State as PVOID);
        return Status;
    }

    (*State).NewPrivileges = core::ptr::addr_of_mut!((*State).NewBuffer);
    (*State).OldPrivileges = (*State).OldBuffer.as_mut_ptr() as PTOKEN_PRIVILEGES;
    (*(*State).NewPrivileges).PrivilegeCount = 1;
    (*(*State).NewPrivileges).Privileges[0].Luid = RtlConvertUlongToLuid(Privilege);
    (*(*State).NewPrivileges).Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

    Size = core::mem::size_of_val(&(*State).OldBuffer) as ULONG;
    Status = NtAdjustPrivilegesToken(
        (*State).TokenHandle,
        0u8,
        (*State).NewPrivileges,
        Size,
        (*State).OldPrivileges,
        core::ptr::addr_of_mut!(Size),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        (*State).OldPrivileges =
            RtlAllocateHeap(SmpHeap, 0, (Size as usize).try_into().unwrap()) as PTOKEN_PRIVILEGES;
        if (*State).OldPrivileges.is_null() {
            Status = STATUS_NO_MEMORY;
        } else {
            Status = NtAdjustPrivilegesToken(
                (*State).TokenHandle,
                0u8,
                (*State).NewPrivileges,
                Size,
                (*State).OldPrivileges,
                core::ptr::addr_of_mut!(Size),
            );
        }
        if Status < 0 {
            if (*State).OldPrivileges != (*State).OldBuffer.as_mut_ptr() as PTOKEN_PRIVILEGES {
                RtlFreeHeap(SmpHeap, 0, (*State).OldPrivileges as PVOID);
            }
            NtClose((*State).TokenHandle);
            RtlFreeHeap(SmpHeap, 0, State as PVOID);
            return Status;
        }
    }

    if Status == STATUS_NOT_ALL_ASSIGNED {
        Status = STATUS_PRIVILEGE_NOT_HELD;
    }
    if Status >= 0 {
        *PrivilegeState = State as PVOID;
        return STATUS_SUCCESS;
    }

    if (*State).OldPrivileges != (*State).OldBuffer.as_mut_ptr() as PTOKEN_PRIVILEGES {
        RtlFreeHeap(SmpHeap, 0, (*State).OldPrivileges as PVOID);
    }

    NtClose((*State).TokenHandle);
    RtlFreeHeap(SmpHeap, 0, State as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn SmpReleasePrivilege(PrivState: PVOID) {
    let State = PrivState as *mut SMP_PRIVILEGE_STATE;

    NtAdjustPrivilegesToken(
        (*State).TokenHandle,
        0u8,
        (*State).OldPrivileges,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if (*State).OldPrivileges != (*State).OldBuffer.as_mut_ptr() as PTOKEN_PRIVILEGES {
        RtlFreeHeap(SmpHeap, 0, (*State).OldPrivileges as PVOID);
    }

    NtClose((*State).TokenHandle);
    RtlFreeHeap(SmpHeap, 0, State as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SmpParseToken(
    Input: PUNICODE_STRING,
    SecondPass: BOOLEAN,
    Token: PUNICODE_STRING,
) -> NTSTATUS {
    let mut p: PWSTR;
    let mut pp: PWSTR;
    let mut Length: ULONG;
    let TokenLength: ULONG;
    let InputLength: ULONG;

    RtlInitUnicodeString(Token, core::ptr::null_mut());

    InputLength = (*Input).Length as ULONG;

    if InputLength == 0 {
        return STATUS_SUCCESS;
    }

    p = (*Input).Buffer;
    Length = 0;
    while Length < InputLength {
        if *p > b' ' as u16 {
            break;
        }
        p = p.add(1);
        Length += core::mem::size_of::<u16>() as ULONG;
    }

    if SecondPass != 0 {
        TokenLength = InputLength - Length;
        pp = (p as usize + TokenLength as usize) as PWSTR;
    } else {
        pp = p;
        while Length < InputLength {
            if *pp <= b' ' as u16 {
                break;
            }
            pp = pp.add(1);
            Length += core::mem::size_of::<u16>() as ULONG;
        }

        TokenLength = (pp as usize - p as usize) as ULONG;
        while Length < InputLength {
            if *pp > b' ' as u16 {
                break;
            }
            pp = pp.add(1);
            Length += core::mem::size_of::<u16>() as ULONG;
        }
    }

    if TokenLength != 0 {
        (*Token).Buffer = RtlAllocateHeap(
            SmpHeap,
            SmBaseTag,
            ((TokenLength as usize + core::mem::size_of::<u16>()) as usize)
                .try_into()
                .unwrap(),
        ) as PWSTR;
        if (*Token).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        (*Token).MaximumLength = (TokenLength as usize + core::mem::size_of::<u16>()) as u16;
        (*Token).Length = TokenLength as u16;
        core::ptr::copy_nonoverlapping(
            p as *const u8,
            (*Token).Buffer as *mut u8,
            TokenLength as usize,
        );
        *(*Token)
            .Buffer
            .add(TokenLength as usize / core::mem::size_of::<u16>()) = UNICODE_NULL;
    }

    (*Input).Length = ((*Input).Length as usize - (pp as usize - (*Input).Buffer as usize)) as u16;
    (*Input).Buffer = pp;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SmpParseCommandLine(
    CommandLine: PUNICODE_STRING,
    Flags: *mut ULONG,
    FileName: PUNICODE_STRING,
    Directory: PUNICODE_STRING,
    Arguments: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Length: ULONG;
    let mut EnvString: _UNICODE_STRING = core::mem::zeroed();
    let mut PathString: _UNICODE_STRING = core::mem::zeroed();
    let mut CmdLineCopy: _UNICODE_STRING;
    let mut Token: _UNICODE_STRING = core::mem::zeroed();
    let mut PathBuffer: [u16; MAX_PATH] = [0; MAX_PATH];
    let mut FilePart: PWSTR = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut FullPathString: _UNICODE_STRING;

    RtlInitUnicodeString(FileName, core::ptr::null_mut());
    RtlInitUnicodeString(Arguments, core::ptr::null_mut());
    if !Directory.is_null() {
        RtlInitUnicodeString(Directory, core::ptr::null_mut());
    }

    if SmpSystemRoot.Length == 0 {
        RtlInitUnicodeString(
            core::ptr::addr_of_mut!(SmpSystemRoot),
            (*SharedUserData).NtSystemRoot.as_ptr(),
        );

        Length = SmpDefaultLibPath.MaximumLength as ULONG
            + SmpSystemRoot.MaximumLength as ULONG
            + (core::mem::size_of::<u16>() * 11) as ULONG;

        FullPathString = _UNICODE_STRING {
            Length: 0,
            MaximumLength: Length as u16,
            Buffer: RtlAllocateHeap(SmpHeap, SmBaseTag, (Length as usize).try_into().unwrap())
                as PWSTR,
        };

        if !FullPathString.Buffer.is_null() {
            static SYSTEM32: [u16; 11] = [
                b'\\' as u16,
                b's' as u16,
                b'y' as u16,
                b's' as u16,
                b't' as u16,
                b'e' as u16,
                b'm' as u16,
                b'3' as u16,
                b'2' as u16,
                b';' as u16,
                0,
            ];

            RtlAppendUnicodeStringToString(
                core::ptr::addr_of_mut!(FullPathString),
                core::ptr::addr_of_mut!(SmpSystemRoot),
            );
            RtlAppendUnicodeToString(core::ptr::addr_of_mut!(FullPathString), SYSTEM32.as_ptr());
            RtlAppendUnicodeStringToString(
                core::ptr::addr_of_mut!(FullPathString),
                core::ptr::addr_of_mut!(SmpDefaultLibPath),
            );
            RtlFreeHeap(SmpHeap, 0, SmpDefaultLibPath.Buffer as PVOID);
            SmpDefaultLibPath = FullPathString;
        }
    }

    CmdLineCopy = *CommandLine;
    loop {
        Status = SmpParseToken(
            core::ptr::addr_of_mut!(CmdLineCopy),
            0u8,
            core::ptr::addr_of_mut!(Token),
        );
        if Status < 0 || Token.Buffer.is_null() {
            return STATUS_UNSUCCESSFUL;
        }
        if Flags.is_null() {
            break;
        }

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(Token),
            core::ptr::addr_of_mut!(SmpDebugKeyword),
            1u8,
        ) != 0
        {
            *Flags |= SMP_DEBUG_FLAG;
        } else if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(Token),
            core::ptr::addr_of_mut!(SmpASyncKeyword),
            1u8,
        ) != 0
        {
            *Flags |= SMP_ASYNC_FLAG;
        } else if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(Token),
            core::ptr::addr_of_mut!(SmpAutoChkKeyword),
            1u8,
        ) != 0
        {
            *Flags |= SMP_AUTOCHK_FLAG;
        } else {
            break;
        }

        RtlFreeHeap(SmpHeap, 0, Token.Buffer as PVOID);
    }

    static PATH_STR: [u16; 5] = [b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(EnvString), PATH_STR.as_ptr());
    Length = PAGE_SIZE as ULONG;
    PathString = _UNICODE_STRING {
        Length: 0,
        MaximumLength: Length as u16,
        Buffer: RtlAllocateHeap(SmpHeap, SmBaseTag, (Length as usize).try_into().unwrap()) as PWSTR,
    };
    if PathString.Buffer.is_null() {
        RtlFreeHeap(SmpHeap, 0, Token.Buffer as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = RtlQueryEnvironmentVariable_U(
        SmpDefaultEnvironment as PWSTR,
        core::ptr::addr_of_mut!(EnvString),
        core::ptr::addr_of_mut!(PathString),
    );
    if Status == STATUS_BUFFER_TOO_SMALL {
        RtlFreeHeap(SmpHeap, 0, PathString.Buffer as PVOID);

        Length = PathString.Length as ULONG + core::mem::size_of::<u16>() as ULONG;
        PathString = _UNICODE_STRING {
            Length: 0,
            MaximumLength: Length as u16,
            Buffer: RtlAllocateHeap(SmpHeap, SmBaseTag, (Length as usize).try_into().unwrap())
                as PWSTR,
        };
        if PathString.Buffer.is_null() {
            RtlFreeHeap(SmpHeap, 0, Token.Buffer as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = RtlQueryEnvironmentVariable_U(
            SmpDefaultEnvironment as PWSTR,
            core::ptr::addr_of_mut!(EnvString),
            core::ptr::addr_of_mut!(PathString),
        );
    }

    if Status < 0 {
        Status = STATUS_OBJECT_NAME_NOT_FOUND;
    } else {
        if !Flags.is_null() {
            static EXE_EXT: [u16; 5] = [b'.' as u16, b'e' as u16, b'x' as u16, b'e' as u16, 0];
            if RtlDosSearchPath_U(
                PathString.Buffer,
                Token.Buffer,
                EXE_EXT.as_ptr(),
                core::mem::size_of_val(&PathBuffer) as ULONG,
                PathBuffer.as_mut_ptr(),
                core::ptr::addr_of_mut!(FilePart),
            ) == 0
                && RtlDosSearchPath_U(
                    SmpDefaultLibPath.Buffer,
                    Token.Buffer,
                    EXE_EXT.as_ptr(),
                    core::mem::size_of_val(&PathBuffer) as ULONG,
                    PathBuffer.as_mut_ptr(),
                    core::ptr::addr_of_mut!(FilePart),
                ) == 0
            {
                *Flags |= SMP_INVALID_PATH;
                *FileName = Token;
                RtlFreeHeap(SmpHeap, 0, PathString.Buffer as PVOID);
                return STATUS_SUCCESS;
            }
        } else {
            let len = Token.Length as usize / core::mem::size_of::<u16>();
            let copy_len = len.min(PathBuffer.len() - 1);
            core::ptr::copy_nonoverlapping(Token.Buffer, PathBuffer.as_mut_ptr(), copy_len);
            PathBuffer[copy_len] = 0;
        }
    }

    RtlFreeHeap(SmpHeap, 0, Token.Buffer as PVOID);
    RtlFreeHeap(SmpHeap, 0, PathString.Buffer as PVOID);
    if Status < 0 {
        return Status;
    }

    if RtlDosPathNameToNtPathName_U(
        PathBuffer.as_ptr(),
        FileName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) == 0
    {
        Status = STATUS_OBJECT_PATH_INVALID;
    }
    if Status < 0 {
        return Status;
    }

    if !Directory.is_null() {
        if FilePart as usize <= PathBuffer.as_ptr() as usize {
            RtlInitUnicodeString(Directory, core::ptr::null_mut());
        } else {
            FilePart = FilePart.sub(1);
            *FilePart = UNICODE_NULL;
            RtlCreateUnicodeString(Directory, PathBuffer.as_ptr());
        }
    }

    SmpParseToken(core::ptr::addr_of_mut!(CmdLineCopy), 1u8, Arguments)
}

#[no_mangle]
pub unsafe extern "system" fn SmpQueryRegistrySosOption() -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    // The C's `WCHAR ValueBuffer[VALUE_BUFFER_SIZE]`: VALUE_BUFFER_SIZE *WCHARs*, not bytes,
    // and aligned for the KEY_VALUE_PARTIAL_INFORMATION read out of it (a bare u8 array is
    // 1-aligned, which makes every field access UB; upstream aa8d624253a's class).
    #[repr(C, align(8))]
    struct ValueBufferT([u16; VALUE_BUFFER_SIZE]);
    let mut ValueBuffer = ValueBufferT([0; VALUE_BUFFER_SIZE]);
    let PartialInfo = ValueBuffer.0.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;

    static KEY_PATH: [u16; 65] = [
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
        b'C' as u16,
        b'o' as u16,
        b'n' as u16,
        b't' as u16,
        b'r' as u16,
        b'o' as u16,
        b'l' as u16,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
    ];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyName), KEY_PATH.as_ptr());
    InitializeObjectAttributes(
        core::ptr::addr_of_mut!(ObjectAttributes),
        core::ptr::addr_of_mut!(KeyName),
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        KEY_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if Status < 0 {
        return 0u8;
    }

    static VALUE_NAME: [u16; 19] = [
        b'S' as u16,
        b'y' as u16,
        b's' as u16,
        b't' as u16,
        b'e' as u16,
        b'm' as u16,
        b'S' as u16,
        b't' as u16,
        b'a' as u16,
        b'r' as u16,
        b't' as u16,
        b'O' as u16,
        b'p' as u16,
        b't' as u16,
        b'i' as u16,
        b'o' as u16,
        b'n' as u16,
        b's' as u16,
        0,
    ];
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ValueName), VALUE_NAME.as_ptr());
    Status = NtQueryValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        KeyValuePartialInformation,
        PartialInfo as PVOID,
        core::mem::size_of_val(&ValueBuffer) as ULONG,
        core::ptr::addr_of_mut!(Length),
    );
    NtClose(KeyHandle);
    if Status < 0 || ((*PartialInfo).Type != REG_SZ && (*PartialInfo).Type != REG_EXPAND_SZ) {
        return 0u8;
    }

    let data_ptr = (*PartialInfo).Data.as_ptr() as *const u16;
    let data_len = ((*PartialInfo).DataLength as usize) / core::mem::size_of::<u16>();

    static SOS_UPPER: [u16; 3] = [b'S' as u16, b'O' as u16, b'S' as u16];
    static SOS_LOWER: [u16; 3] = [b's' as u16, b'o' as u16, b's' as u16];

    let mut found = false;
    for i in 0..data_len.saturating_sub(2) {
        if *data_ptr.add(i) == SOS_UPPER[0]
            && *data_ptr.add(i + 1) == SOS_UPPER[1]
            && *data_ptr.add(i + 2) == SOS_UPPER[2]
        {
            found = true;
            break;
        }
        if *data_ptr.add(i) == SOS_LOWER[0]
            && *data_ptr.add(i + 1) == SOS_LOWER[1]
            && *data_ptr.add(i + 2) == SOS_LOWER[2]
        {
            found = true;
            break;
        }
    }

    if !found {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SmpSaveAndClearBootStatusData(
    BootOkay: PBOOLEAN,
    ShutdownOkay: PBOOLEAN,
) -> BOOLEAN {
    let Status: NTSTATUS;
    let Value: BOOLEAN = 1u8;
    let mut BootStatusDataHandle: PVOID = core::ptr::null_mut();

    *BootOkay = 0u8;
    *ShutdownOkay = 0u8;

    Status = RtlLockBootStatusData(core::ptr::addr_of_mut!(BootStatusDataHandle));
    if Status < 0 {
        return 0u8;
    }

    RtlGetSetBootStatusData(
        BootStatusDataHandle,
        1u8,
        RtlBsdItemBootGood,
        BootOkay as PVOID,
        core::mem::size_of::<BOOLEAN>() as ULONG,
        core::ptr::null_mut(),
    );
    RtlGetSetBootStatusData(
        BootStatusDataHandle,
        1u8,
        RtlBsdItemBootShutdown,
        ShutdownOkay as PVOID,
        core::mem::size_of::<BOOLEAN>() as ULONG,
        core::ptr::null_mut(),
    );

    RtlGetSetBootStatusData(
        BootStatusDataHandle,
        0u8,
        RtlBsdItemBootGood,
        core::ptr::addr_of!(Value) as PVOID,
        core::mem::size_of_val(&Value) as ULONG,
        core::ptr::null_mut(),
    );
    RtlGetSetBootStatusData(
        BootStatusDataHandle,
        0u8,
        RtlBsdItemBootShutdown,
        core::ptr::addr_of!(Value) as PVOID,
        core::mem::size_of_val(&Value) as ULONG,
        core::ptr::null_mut(),
    );

    RtlUnlockBootStatusData(BootStatusDataHandle);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn SmpRestoreBootStatusData(BootOkay: BOOLEAN, ShutdownOkay: BOOLEAN) {
    let Status: NTSTATUS;
    let mut BootState: PVOID = core::ptr::null_mut();

    Status = RtlLockBootStatusData(core::ptr::addr_of_mut!(BootState));
    if Status >= 0 {
        RtlGetSetBootStatusData(
            BootState,
            0u8,
            RtlBsdItemBootGood,
            core::ptr::addr_of!(BootOkay) as PVOID,
            core::mem::size_of::<BOOLEAN>() as ULONG,
            core::ptr::null_mut(),
        );
        RtlGetSetBootStatusData(
            BootState,
            0u8,
            RtlBsdItemBootShutdown,
            core::ptr::addr_of!(ShutdownOkay) as PVOID,
            core::mem::size_of::<BOOLEAN>() as ULONG,
            core::ptr::null_mut(),
        );

        RtlUnlockBootStatusData(BootState);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Windows-Compatible Session Manager
//  * LICENSE:         BSD 2-Clause License
//  * FILE:            base/system/smss/smutil.c
//  * PURPOSE:         Main SMSS Code
//  * PROGRAMMERS:     Alex Ionescu
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "smss.h"
//
// #include <ndk/sefuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// //
// // Taken from an ASSERT
// //
// #define VALUE_BUFFER_SIZE (sizeof(KEY_VALUE_PARTIAL_INFORMATION) + 512)
//
// typedef struct _SMP_PRIVILEGE_STATE
// {
//     HANDLE TokenHandle;
//     PTOKEN_PRIVILEGES OldPrivileges;
//     PTOKEN_PRIVILEGES NewPrivileges;
//     UCHAR OldBuffer[1024];
//     TOKEN_PRIVILEGES NewBuffer;
// } SMP_PRIVILEGE_STATE, *PSMP_PRIVILEGE_STATE;
//
// UNICODE_STRING SmpDebugKeyword, SmpASyncKeyword, SmpAutoChkKeyword;
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// SmpAcquirePrivilege(IN ULONG Privilege,
//                     OUT PVOID *PrivilegeState)
// {
//     PSMP_PRIVILEGE_STATE State;
//     ULONG Size;
//     NTSTATUS Status;
//
//     /* Assume failure */
//     *PrivilegeState = NULL;
//
//     /* Acquire the state structure to hold everything we need */
//     State = RtlAllocateHeap(SmpHeap,
//                             0,
//                             sizeof(SMP_PRIVILEGE_STATE) +
//                             sizeof(TOKEN_PRIVILEGES) +
//                             sizeof(LUID_AND_ATTRIBUTES));
//     if (!State) return STATUS_NO_MEMORY;
//
//     /* Open our token */
//     Status = NtOpenProcessToken(NtCurrentProcess(),
//                                 TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
//                                 &State->TokenHandle);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         RtlFreeHeap(SmpHeap, 0, State);
//         return Status;
//     }
//
//     /* Set one privilege in the enabled state */
//     State->NewPrivileges = &State->NewBuffer;
//     State->OldPrivileges = (PTOKEN_PRIVILEGES)&State->OldBuffer;
//     State->NewPrivileges->PrivilegeCount = 1;
//     State->NewPrivileges->Privileges[0].Luid = RtlConvertUlongToLuid(Privilege);
//     State->NewPrivileges->Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
//
//     /* Adjust the privileges in the token */
//     Size = sizeof(State->OldBuffer);
//     Status = NtAdjustPrivilegesToken(State->TokenHandle,
//                                      FALSE,
//                                      State->NewPrivileges,
//                                      Size,
//                                      State->OldPrivileges,
//                                      &Size);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* Our static buffer is not big enough, allocate a bigger one */
//         State->OldPrivileges = RtlAllocateHeap(SmpHeap, 0, Size);
//         if (!State->OldPrivileges)
//         {
//             /* Out of memory, fail */
//             Status = STATUS_NO_MEMORY;
//             goto Quickie;
//         }
//
//         /* Now try again */
//         Status = NtAdjustPrivilegesToken(State->TokenHandle,
//                                          FALSE,
//                                          State->NewPrivileges,
//                                          Size,
//                                          State->OldPrivileges,
//                                          &Size);
//     }
//
//     /* Normalize failure code and check for success */
//     if (Status == STATUS_NOT_ALL_ASSIGNED) Status = STATUS_PRIVILEGE_NOT_HELD;
//     if (NT_SUCCESS(Status))
//     {
//         /* We got the privilege, return */
//         *PrivilegeState = State;
//         return STATUS_SUCCESS;
//     }
//
// Quickie:
//     /* Check if we used a dynamic buffer */
//     if (State->OldPrivileges != (PTOKEN_PRIVILEGES)&State->OldBuffer)
//     {
//         /* Free it */
//         RtlFreeHeap(SmpHeap, 0, State->OldPrivileges);
//     }
//
//     /* Close the token handle and free the state structure */
//     NtClose(State->TokenHandle);
//     RtlFreeHeap(SmpHeap, 0, State);
//     return Status;
// }
//
// VOID
// NTAPI
// SmpReleasePrivilege(IN PVOID PrivState)
// {
//     PSMP_PRIVILEGE_STATE State = (PSMP_PRIVILEGE_STATE)PrivState;
//
//     /* Adjust the privileges in the token */
//     NtAdjustPrivilegesToken(State->TokenHandle,
//                             FALSE,
//                             State->OldPrivileges,
//                             0,
//                             NULL,
//                             NULL);
//
//     /* Check if we used a dynamic buffer */
//     if (State->OldPrivileges != (PTOKEN_PRIVILEGES)&State->OldBuffer)
//     {
//         /* Free it */
//         RtlFreeHeap(SmpHeap, 0, State->OldPrivileges);
//     }
//
//     /* Close the token handle and free the state structure */
//     NtClose(State->TokenHandle);
//     RtlFreeHeap(SmpHeap, 0, State);
// }
//
// NTSTATUS
// NTAPI
// SmpParseToken(IN PUNICODE_STRING Input,
//               IN BOOLEAN SecondPass,
//               OUT PUNICODE_STRING Token)
// {
//     PWCHAR p, pp;
//     ULONG Length, TokenLength, InputLength;
//
//     /* Initialize to NULL to start with */
//     RtlInitUnicodeString(Token, NULL);
//
//     /* Save the input length */
//     InputLength = Input->Length;
//
//     /* If the input string is empty, just return */
//     if (InputLength == 0) return STATUS_SUCCESS;
//
//     /* Parse the buffer until the first character */
//     p = Input->Buffer;
//     Length = 0;
//     while (Length < InputLength)
//     {
//         if (*p > L' ') break;
//         ++p;
//         Length += sizeof(WCHAR);
//     }
//
//     /* Are we being called for argument pick-up? */
//     if (SecondPass)
//     {
//         /* Then assume everything else is an argument */
//         TokenLength = InputLength - Length * sizeof(WCHAR);
//         pp = (PWSTR)((ULONG_PTR)p + TokenLength);
//     }
//     else
//     {
//         /* No -- so loop until the next space */
//         pp = p;
//         while (Length < InputLength)
//         {
//             if (*pp <= L' ') break;
//             ++pp;
//             Length += sizeof(WCHAR);
//         }
//
//         /* Now compute how long this token is, and loop until the next char */
//         TokenLength = (ULONG_PTR)pp - (ULONG_PTR)p;
//         while (Length < InputLength)
//         {
//             if (*pp > L' ') break;
//             ++pp;
//             Length += sizeof(WCHAR);
//         }
//     }
//
//     /* Did we find a token? */
//     if (TokenLength)
//     {
//         /* Allocate a buffer for it */
//         Token->Buffer = RtlAllocateHeap(SmpHeap,
//                                         SmBaseTag,
//                                         TokenLength + sizeof(UNICODE_NULL));
//         if (!Token->Buffer) return STATUS_NO_MEMORY;
//
//         /* Fill in the unicode string to hold it */
//         Token->MaximumLength = (USHORT)(TokenLength + sizeof(UNICODE_NULL));
//         Token->Length = (USHORT)TokenLength;
//         RtlCopyMemory(Token->Buffer, p, TokenLength);
//         Token->Buffer[TokenLength / sizeof(WCHAR)] = UNICODE_NULL;
//     }
//
//     /* Modify the input string with the position of where the next token begins */
//     Input->Length -= (USHORT)((ULONG_PTR)pp - (ULONG_PTR)Input->Buffer);
//     Input->Buffer = pp;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// SmpParseCommandLine(IN PUNICODE_STRING CommandLine,
//                     OUT PULONG Flags,
//                     OUT PUNICODE_STRING FileName,
//                     OUT PUNICODE_STRING Directory,
//                     OUT PUNICODE_STRING Arguments)
// {
//     ULONG Length;
//     UNICODE_STRING EnvString, PathString, CmdLineCopy, Token;
//     WCHAR PathBuffer[MAX_PATH];
//     PWCHAR FilePart;
//     NTSTATUS Status;
//     UNICODE_STRING FullPathString;
//
//     /* Initialize output arguments to NULL */
//     RtlInitUnicodeString(FileName, NULL);
//     RtlInitUnicodeString(Arguments, NULL);
//     if (Directory) RtlInitUnicodeString(Directory, NULL);
//
//     /* Check if we haven't yet built a full default path or system root yet */
//     if (!SmpSystemRoot.Length)
//     {
//         /* Initialize it based on shared user data string */
//         RtlInitUnicodeString(&SmpSystemRoot, SharedUserData->NtSystemRoot);
//
//         /* Allocate an empty string for the path */
//         Length = SmpDefaultLibPath.MaximumLength + SmpSystemRoot.MaximumLength +
//                  sizeof(L"\\system32;");
//         RtlInitEmptyUnicodeString(&FullPathString,
//                                   RtlAllocateHeap(SmpHeap, SmBaseTag, Length),
//                                   (USHORT)Length);
//         if (FullPathString.Buffer)
//         {
//             /* Append the root, system32;, and then the current library path */
//             RtlAppendUnicodeStringToString(&FullPathString, &SmpSystemRoot);
//             RtlAppendUnicodeToString(&FullPathString, L"\\system32;");
//             RtlAppendUnicodeStringToString(&FullPathString, &SmpDefaultLibPath);
//             RtlFreeHeap(SmpHeap, 0, SmpDefaultLibPath.Buffer);
//             SmpDefaultLibPath = FullPathString;
//         }
//     }
//
//     /* Consume the command line */
//     CmdLineCopy = *CommandLine;
//     while (TRUE)
//     {
//         /* Parse the first token and check for modifiers/specifiers */
//         Status = SmpParseToken(&CmdLineCopy, FALSE, &Token);
//         if (!(NT_SUCCESS(Status)) || !(Token.Buffer)) return STATUS_UNSUCCESSFUL;
//         if (!Flags) break;
//
//         /* Debug requested? */
//         if (RtlEqualUnicodeString(&Token, &SmpDebugKeyword, TRUE))
//         {
//             /* Convert into a flag */
//             *Flags |= SMP_DEBUG_FLAG;
//         }
//         else if (RtlEqualUnicodeString(&Token, &SmpASyncKeyword, TRUE))
//         {
//             /* Asynch requested, convert into a flag */
//             *Flags |= SMP_ASYNC_FLAG;
//         }
//         else if (RtlEqualUnicodeString(&Token, &SmpAutoChkKeyword, TRUE))
//         {
//             /* Autochk requested, convert into a flag */
//             *Flags |= SMP_AUTOCHK_FLAG;
//         }
//         else
//         {
//             /* No specifier found, keep going */
//             break;
//         }
//
//         /* Get rid of this token and get the next */
//         RtlFreeHeap(SmpHeap, 0, Token.Buffer);
//     }
//
//     /* Initialize a string to hold the current path */
//     RtlInitUnicodeString(&EnvString, L"Path");
//     Length = PAGE_SIZE;
//     RtlInitEmptyUnicodeString(&PathString,
//                               RtlAllocateHeap(SmpHeap, SmBaseTag, Length),
//                               (USHORT)Length);
//     if (!PathString.Buffer)
//     {
//         /* Fail if we have no memory for this */
//         RtlFreeHeap(SmpHeap, 0, Token.Buffer);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Query the path from the environment */
//     Status = RtlQueryEnvironmentVariable_U(SmpDefaultEnvironment,
//                                            &EnvString,
//                                            &PathString);
//     if (Status == STATUS_BUFFER_TOO_SMALL)
//     {
//         /* Our buffer was too small, free it */
//         RtlFreeHeap(SmpHeap, 0, PathString.Buffer);
//
//         /* And allocate one big enough */
//         Length = PathString.Length + sizeof(UNICODE_NULL);
//         RtlInitEmptyUnicodeString(&PathString,
//                                   RtlAllocateHeap(SmpHeap, SmBaseTag, Length),
//                                   (USHORT)Length);
//         if (!PathString.Buffer)
//         {
//             /* Fail if we have no memory for this */
//             RtlFreeHeap(SmpHeap, 0, Token.Buffer);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Now try again, this should work */
//         Status = RtlQueryEnvironmentVariable_U(SmpDefaultEnvironment,
//                                                &EnvString,
//                                                &PathString);
//     }
//     if (!NT_SUCCESS(Status))
//     {
//         /* Another failure means that the kernel hasn't passed the path correctly */
//         DPRINT1("SMSS: %wZ environment variable not defined.\n", &EnvString);
//         Status = STATUS_OBJECT_NAME_NOT_FOUND;
//     }
//     else
//     {
//         /* Check if the caller expects any flags out of here */
//         if (Flags)
//         {
//             /* We can return the image not found flag -- so does the image exist */
//             if (!(RtlDosSearchPath_U(PathString.Buffer,
//                                      Token.Buffer,
//                                      L".exe",
//                                      sizeof(PathBuffer),
//                                      PathBuffer,
//                                      &FilePart)) &&
//                 !(RtlDosSearchPath_U(SmpDefaultLibPath.Buffer,
//                                      Token.Buffer,
//                                      L".exe",
//                                      sizeof(PathBuffer),
//                                      PathBuffer,
//                                      &FilePart)))
//             {
//                 /* It doesn't, let the caller know about it and exit */
//                 *Flags |= SMP_INVALID_PATH;
//                 *FileName = Token;
//                 RtlFreeHeap(SmpHeap, 0, PathString.Buffer);
//                 return STATUS_SUCCESS;
//             }
//         }
//         else
//         {
//             /* Caller doesn't want flags, probably wants the image itself */
//             RtlStringCbCopyW(PathBuffer, sizeof(PathBuffer), Token.Buffer);
//         }
//     }
//
//     /* Free tokens and such, all that's left is to convert the image name */
//     RtlFreeHeap(SmpHeap, 0, Token.Buffer);
//     RtlFreeHeap(SmpHeap, 0, PathString.Buffer);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Convert it and bail out if this failed */
//     if (!RtlDosPathNameToNtPathName_U(PathBuffer, FileName, NULL, NULL))
//     {
//         DPRINT1("SMSS: Unable to translate %ws into an NT File Name\n",
//                 &PathBuffer);
//         Status = STATUS_OBJECT_PATH_INVALID;
//     }
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Finally, check if the caller also wanted the directory */
//     if (Directory)
//     {
//         /* Is the file a relative name with no directory? */
//         if (FilePart <= PathBuffer)
//         {
//             /* Clear it */
//             RtlInitUnicodeString(Directory, NULL);
//         }
//         else
//         {
//             /* There is a directory, and a filename -- separate those two */
//             *--FilePart = UNICODE_NULL;
//             RtlCreateUnicodeString(Directory, PathBuffer);
//         }
//     }
//
//     /* We are done -- move on to the second pass to get the arguments */
//     return SmpParseToken(&CmdLineCopy, TRUE, Arguments);
// }
//
// BOOLEAN
// NTAPI
// SmpQueryRegistrySosOption(VOID)
// {
//     NTSTATUS Status;
//     UNICODE_STRING KeyName, ValueName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle;
//     ULONG Length;
//     WCHAR ValueBuffer[VALUE_BUFFER_SIZE];
//     PKEY_VALUE_PARTIAL_INFORMATION PartialInfo = (PVOID)ValueBuffer;
//
//     /* Open the key */
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\Control");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("SMSS: Cannot open control key (Status 0x%x)\n", Status);
//         return FALSE;
//     }
//
//     /* Query the value */
//     RtlInitUnicodeString(&ValueName, L"SystemStartOptions");
//     Status = NtQueryValueKey(KeyHandle,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              PartialInfo,
//                              sizeof(ValueBuffer),
//                              &Length);
//     ASSERT(Length < VALUE_BUFFER_SIZE);
//     NtClose(KeyHandle);
//     if (!NT_SUCCESS(Status) ||
//         ((PartialInfo->Type != REG_SZ) && (PartialInfo->Type != REG_EXPAND_SZ)))
//     {
//         DPRINT1("SMSS: Cannot query value key (Type %lu, Status 0x%x)\n",
//                 PartialInfo->Type, Status);
//         return FALSE;
//     }
//
//     /* Check if it's set to SOS or sos */
//     if (!(wcsstr((PWCHAR)PartialInfo->Data, L"SOS")) ||
//          (wcsstr((PWCHAR)PartialInfo->Data, L"sos")))
//     {
//         /* It's not set, return FALSE */
//         return FALSE;
//     }
//
//     /* It's set, return TRUE */
//     return TRUE;
// }
//
// BOOLEAN
// NTAPI
// SmpSaveAndClearBootStatusData(OUT PBOOLEAN BootOkay,
//                               OUT PBOOLEAN ShutdownOkay)
// {
//     NTSTATUS Status;
//     BOOLEAN Value = TRUE;
//     PVOID BootStatusDataHandle;
//
//     /* Assume failure */
//     *BootOkay = FALSE;
//     *ShutdownOkay = FALSE;
//
//     /* Lock the BSD and fail if we couldn't */
//     Status = RtlLockBootStatusData(&BootStatusDataHandle);
//     if (!NT_SUCCESS(Status)) return FALSE;
//
//     /* Read the old settings */
//     RtlGetSetBootStatusData(BootStatusDataHandle,
//                             TRUE,
//                             RtlBsdItemBootGood,
//                             BootOkay,
//                             sizeof(BOOLEAN),
//                             NULL);
//     RtlGetSetBootStatusData(BootStatusDataHandle,
//                             TRUE,
//                             RtlBsdItemBootShutdown,
//                             ShutdownOkay,
//                             sizeof(BOOLEAN),
//                             NULL);
//
//     /* Set new ones indicating we got at least this far */
//     RtlGetSetBootStatusData(BootStatusDataHandle,
//                             FALSE,
//                             RtlBsdItemBootGood,
//                             &Value,
//                             sizeof(Value),
//                             NULL);
//     RtlGetSetBootStatusData(BootStatusDataHandle,
//                             FALSE,
//                             RtlBsdItemBootShutdown,
//                             &Value,
//                             sizeof(Value),
//                             NULL);
//
//     /* Unlock the BSD and return */
//     RtlUnlockBootStatusData(BootStatusDataHandle);
//     return TRUE;
// }
//
// VOID
// NTAPI
// SmpRestoreBootStatusData(IN BOOLEAN BootOkay,
//                          IN BOOLEAN ShutdownOkay)
// {
//     NTSTATUS Status;
//     PVOID BootState;
//
//     /* Lock the BSD */
//     Status = RtlLockBootStatusData(&BootState);
//     if (NT_SUCCESS(Status))
//     {
//         /* Write the bootokay and bootshutdown values */
//         RtlGetSetBootStatusData(BootState,
//                                 FALSE,
//                                 RtlBsdItemBootGood,
//                                 &BootOkay,
//                                 sizeof(BootOkay),
//                                 NULL);
//         RtlGetSetBootStatusData(BootState,
//                                 FALSE,
//                                 RtlBsdItemBootShutdown,
//                                 &ShutdownOkay,
//                                 sizeof(ShutdownOkay),
//                                 NULL);
//
//         /* Unlock the BSD and return */
//         RtlUnlockBootStatusData(BootState);
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
