use crate::setuplib::partlist::GetDiskBySCSI;
use crate::setuplib::partlist::GetDiskBySignature;
use crate::setuplib::partlist::GetPartition;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_18CA6AF5ACA2: [u16; 33] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0025,
    0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_25C59D0378B0: [u16; 10] = [0x0070u16, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000];
pub static RUSTCONV_WSTR_524C5CAE6CCF: [u16; 18] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_62AF54F8206D: [u16; 22] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0053, 0x0063,
    0x0073, 0x0069, 0x005C, 0x0043, 0x0064, 0x0052, 0x006F, 0x006D, 0x0025, 0x006C,
    0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_7730A3AD01A7: [u16; 19] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0052, 0x0061,
    0x006D, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_7BC572EA8FA4: [u16; 10] = [0x0073u16, 0x0069, 0x0067, 0x006E, 0x0061, 0x0074, 0x0075, 0x0072, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_8CCE59A651CE: [u16; 9] = [0x005Cu16, 0x0041, 0x0072, 0x0063, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_A969AFDFA6D5: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0043, 0x0064,
    0x0052, 0x006F, 0x006D, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_D6DFBCB9D927: [u16; 3] = [0x0028u16, 0x0029, 0x0000];
pub static RUSTCONV_WSTR_E96BA1AACA82: [u16; 4] = [0x0028u16, 0x0030, 0x0029, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_433B71C45E79: [u16; 8] = [0x0072u16, 0x0061, 0x006D, 0x0064, 0x0069, 0x0073, 0x006B, 0x0000];
pub static RUSTCONV_C2R_WSTR_6A5906DEFECB: [u16; 5] = [0x0073u16, 0x0063, 0x0073, 0x0069, 0x0000];
pub static RUSTCONV_C2R_WSTR_7E66718A76FA: [u16; 6] = [0x006Du16, 0x0075, 0x006C, 0x0074, 0x0069, 0x0000];
pub static RUSTCONV_C2R_WSTR_8443F98E8C2D: [u16; 6] = [0x0063u16, 0x0064, 0x0072, 0x006F, 0x006D, 0x0000];
pub static RUSTCONV_C2R_WSTR_97EA2F6038B7: [u16; 6] = [0x0072u16, 0x0064, 0x0069, 0x0073, 0x006B, 0x0000];
pub static RUSTCONV_C2R_WSTR_A2458B8BF19F: [u16; 5] = [0x0065u16, 0x0069, 0x0073, 0x0061, 0x0000];
pub static RUSTCONV_C2R_WSTR_B4D2E732F061: [u16; 4] = [0x006Eu16, 0x0065, 0x0074, 0x0000];
pub static RUSTCONV_C2R_WSTR_C2498467770F: [u16; 6] = [0x0066u16, 0x0064, 0x0069, 0x0073, 0x006B, 0x0000];
pub static RUSTCONV_C2R_WSTR_C34406FE9984: [u16; 5] = [0x0064u16, 0x0069, 0x0073, 0x006B, 0x0000];

pub static mut AdapterTypes_A: [PCSTR; 6] = [
    b"eisa\0" as *const u8 as *const ::core::ffi::c_char,
    b"scsi\0" as *const u8 as *const ::core::ffi::c_char,
    b"multi\0" as *const u8 as *const ::core::ffi::c_char,
    b"net\0" as *const u8 as *const ::core::ffi::c_char,
    b"ramdisk\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<CHAR>(),
];

pub static mut AdapterTypes_U: [PCWSTR; 6] = unsafe {
    [
        RUSTCONV_C2R_WSTR_A2458B8BF19F.as_ptr(),
        RUSTCONV_C2R_WSTR_6A5906DEFECB.as_ptr(),
        RUSTCONV_C2R_WSTR_7E66718A76FA.as_ptr(),
        RUSTCONV_C2R_WSTR_B4D2E732F061.as_ptr(),
        RUSTCONV_C2R_WSTR_433B71C45E79.as_ptr(),
        ::core::ptr::null::<WCHAR>(),
    ]
};

pub static mut ControllerTypes_A: [PCSTR; 3] = [
    b"disk\0" as *const u8 as *const ::core::ffi::c_char,
    b"cdrom\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<CHAR>(),
];

pub static mut ControllerTypes_U: [PCWSTR; 3] = unsafe {
    [
        RUSTCONV_C2R_WSTR_C34406FE9984.as_ptr(),
        RUSTCONV_C2R_WSTR_8443F98E8C2D.as_ptr(),
        ::core::ptr::null::<WCHAR>(),
    ]
};

pub static mut PeripheralTypes_A: [PCSTR; 4] = [
    b"rdisk\0" as *const u8 as *const ::core::ffi::c_char,
    b"fdisk\0" as *const u8 as *const ::core::ffi::c_char,
    b"cdrom\0" as *const u8 as *const ::core::ffi::c_char,
    ::core::ptr::null::<CHAR>(),
];

pub static mut PeripheralTypes_U: [PCWSTR; 4] = unsafe {
    [
        RUSTCONV_C2R_WSTR_97EA2F6038B7.as_ptr(),
        RUSTCONV_C2R_WSTR_C2498467770F.as_ptr(),
        RUSTCONV_C2R_WSTR_8443F98E8C2D.as_ptr(),
        ::core::ptr::null::<WCHAR>(),
    ]
};

// ADAPTER_TYPE / CONTROLLER_TYPE / PERIPHERAL_TYPE are arcname.c's OWN local enums
// (base/setup/lib/utils/arcname.c:37-93), numbered from 0 and used as indexes into the
// AdapterTypes_* / ControllerTypes_* / PeripheralTypes_* string tables above. They are
// NOT the CONFIGURATION_TYPE members of the same names (EisaAdapter = 8 there), so the
// 0-based values here are the C's values.
pub type _ADAPTER_TYPE = ::core::ffi::c_uint;
pub const EisaAdapter: _ADAPTER_TYPE = 0; // nolint: const-contradiction (module-private enum; the island's EisaAdapter is a different enum)
pub const ScsiAdapter: _ADAPTER_TYPE = 1; // nolint: const-contradiction (module-private enum; the island's ScsiAdapter is a different enum)
pub const MultiAdapter: _ADAPTER_TYPE = 2;
pub const NetAdapter: _ADAPTER_TYPE = 3;
pub const RamdiskAdapter: _ADAPTER_TYPE = 4;
pub const AdapterTypeMax: _ADAPTER_TYPE = 5;
pub type ADAPTER_TYPE = _ADAPTER_TYPE;
pub type PADAPTER_TYPE = *mut _ADAPTER_TYPE;

pub type _CONTROLLER_TYPE = ::core::ffi::c_uint;
pub const DiskController: _CONTROLLER_TYPE = 0; // nolint: const-contradiction (module-private enum; the island's DiskController is a different enum)
pub const CdRomController: _CONTROLLER_TYPE = 1;
pub const ControllerTypeMax: _CONTROLLER_TYPE = 2;
pub type CONTROLLER_TYPE = _CONTROLLER_TYPE;
pub type PCONTROLLER_TYPE = *mut _CONTROLLER_TYPE;

pub type _PERIPHERAL_TYPE = ::core::ffi::c_uint;
pub const RDiskPeripheral: _PERIPHERAL_TYPE = 0;
pub const FDiskPeripheral: _PERIPHERAL_TYPE = 1;
pub const CdRomPeripheral: _PERIPHERAL_TYPE = 2;
pub const PeripheralTypeMax: _PERIPHERAL_TYPE = 3;
pub type PERIPHERAL_TYPE = _PERIPHERAL_TYPE;
pub type PPERIPHERAL_TYPE = *mut _PERIPHERAL_TYPE;
#[no_mangle]
pub unsafe extern "C" fn ArcGetNextTokenA(
    ArcPath: PCSTR,
    TokenSpecifier: PANSI_STRING,
    Key: PULONG,
) -> PCSTR {
    let mut Status: NTSTATUS;
    let mut p: PCSTR = ArcPath;
    let mut SpecifierLength: usize;
    let mut KeyValue: u32;

    p = strchr(p, '(' as i32);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    if p == ArcPath {
        return core::ptr::null_mut();
    }

    SpecifierLength = (p.offset_from(ArcPath) as usize) * core::mem::size_of::<CHAR>();
    if SpecifierLength > (MAXUSHORT) as _ {
        return core::ptr::null_mut();
    }

    KeyValue = strtoul(p, &mut p, 10);

    while isspace(*p as i32) != 0 {
        p = p.add(1);
    }

    if *p as i32 != ')' as i32 {
        return core::ptr::null_mut();
    }

    Status = RtlStringCbCopyNA(
        (*TokenSpecifier).Buffer,
        ((*TokenSpecifier).MaximumLength) as _,
        ArcPath,
        SpecifierLength,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return core::ptr::null_mut();
    }

    (*TokenSpecifier).Length = SpecifierLength as u16;

    *Key = KeyValue;

    p = p.add(1);
    p
}

unsafe extern "C" fn ArcGetNextTokenU(
    mut ArcPath: PCWSTR,
    mut TokenSpecifier: PUNICODE_STRING,
    mut Key: PULONG,
) -> PCWSTR {
    let mut Status: NTSTATUS = 0;
    let mut p: PCWSTR = ArcPath;
    let mut SpecifierLength: usize = 0;
    let mut KeyValue: u32 = 0;

    p = wcschr(p, 0x0028u16);
    if p.is_null() {
        return core::ptr::null_mut();
    }
    if p == ArcPath {
        return core::ptr::null_mut();
    }

    SpecifierLength = (p.offset_from(ArcPath) as usize)
        .wrapping_mul(core::mem::size_of::<u16>());
    if SpecifierLength > (UNICODE_STRING_MAX_BYTES) as _ {
        return core::ptr::null_mut();
    }

    p = p.add(1);

    KeyValue = wcstoul(p, &mut p as *mut PCWSTR as *mut *mut u16, 10);

    while iswspace(*p) != 0 {
        p = p.add(1);
    }

    if *p != 0x0029u16 {
        return core::ptr::null_mut();
    }

    Status = RtlStringCbCopyNW(
        (*TokenSpecifier).Buffer,
        ((*TokenSpecifier).MaximumLength) as _,
        ArcPath,
        SpecifierLength,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return core::ptr::null_mut();
    }

    (*TokenSpecifier).Length = SpecifierLength as u16;

    *Key = KeyValue;

    p = p.add(1);
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn ArcMatchTokenA(
    CandidateToken: PCSTR,
    TokenTable: *const PCSTR,
) -> u32 {
    let mut Index: u32 = 0;

    while !(*TokenTable.offset(Index as isize)).is_null()
        && _stricmp(
            CandidateToken as *const core::ffi::c_char,
            *TokenTable.offset(Index as isize) as *const core::ffi::c_char,
        ) != 0
    {
        Index = Index.wrapping_add(1);
    }

    return Index;
}
#[no_mangle]
pub unsafe extern "C" fn ArcMatchTokenU(
    CandidateToken: PCWSTR,
    TokenTable: *const PCWSTR,
) -> u32 {
    let mut Index: u32 = 0;

    while !(*TokenTable.offset(Index as isize)).is_null()
        && _wcsicmp(CandidateToken, *TokenTable.offset(Index as isize)) != 0
    {
        Index = Index.wrapping_add(1);
    }

    return Index;
}

unsafe extern "C" fn ArcMatchToken_UStr(
    CandidateToken: PCUNICODE_STRING,
    TokenTable: *const PCWSTR,
) -> u32 {
    let mut Index: u32 = 0;
    let mut Token: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    while !(*TokenTable.offset(Index as isize)).is_null() {
        RtlInitUnicodeString(&mut Token, *TokenTable.offset(Index as isize));
        if RtlEqualUnicodeString(CandidateToken, &Token, (TRUE) as BOOLEAN) != 0 {
            break;
        }
        Index = Index.wrapping_add(1);
    }

    Index
}
#[no_mangle]
pub unsafe extern "C" fn ArcPathNormalize(
    NormalizedArcPath: PUNICODE_STRING,
    mut ArcPath: PCWSTR,
) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut EndOfArcName: PCWSTR;
    let mut p: PCWSTR;
    let mut PathLength: usize;

    if ((*NormalizedArcPath).MaximumLength as usize) < core::mem::size_of_val(&UNICODE_NULL) {
        return (FALSE) as u8;
    }

    *(*NormalizedArcPath).Buffer = UNICODE_NULL;
    (*NormalizedArcPath).Length = 0;

    EndOfArcName = wcschr(ArcPath, (OBJ_NAME_PATH_SEPARATOR) as _);
    if EndOfArcName.is_null() {
        EndOfArcName = ArcPath.add(wcslen(ArcPath));
    }

    loop {
        p = wcsstr(ArcPath, RUSTCONV_WSTR_D6DFBCB9D927.as_ptr());
        if p.is_null() || p >= EndOfArcName {
            break;
        }

        Status = RtlStringCbCatNW(
            (*NormalizedArcPath).Buffer,
            ((*NormalizedArcPath).MaximumLength) as _,
            ArcPath,
            (p.offset_from(ArcPath) as usize) * core::mem::size_of::<u16>(),
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            return (FALSE) as u8;
        }

        Status = RtlStringCbCatW(
            (*NormalizedArcPath).Buffer,
            ((*NormalizedArcPath).MaximumLength) as _,
            RUSTCONV_WSTR_E96BA1AACA82.as_ptr(),
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            return (FALSE) as u8;
        }
        ArcPath = p.add(2);
    }

    Status = RtlStringCbCatW(
        (*NormalizedArcPath).Buffer,
        ((*NormalizedArcPath).MaximumLength) as _,
        ArcPath,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return (FALSE) as u8;
    }

    PathLength = wcslen((*NormalizedArcPath).Buffer);
    if PathLength > (UNICODE_STRING_MAX_CHARS) as _ {
        return (FALSE) as u8;
    }

    (*NormalizedArcPath).Length = (PathLength as u16) * core::mem::size_of::<u16>() as u16;
    return (TRUE) as u8;
}

unsafe extern "C" fn ParseArcName(
    mut ArcNamePath: *mut PCWSTR,
    mut pAdapterKey: PULONG,
    mut pControllerKey: PULONG,
    mut pPeripheralKey: PULONG,
    mut pPartitionNumber: PULONG,
    mut pAdapterType: PADAPTER_TYPE,
    mut pControllerType: PCONTROLLER_TYPE,
    mut pPeripheralType: PPERIPHERAL_TYPE,
    mut pUseSignature: PBOOLEAN,
) -> NTSTATUS {
    let mut TokenBuffer: [u16; 50] = [0; 50];
    let mut Token: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut p: PCWSTR = core::ptr::null();
    let mut q: PCWSTR = core::ptr::null();
    let mut AdapterKey: u32 = 0;
    let mut ControllerKey: u32 = 0;
    let mut PeripheralKey: u32 = 0;
    let mut PartitionNumber: u32 = 0;
    let mut AdapterType_local: ADAPTER_TYPE = AdapterTypeMax;
    let mut ControllerType: CONTROLLER_TYPE = ControllerTypeMax;
    let mut PeripheralType: PERIPHERAL_TYPE = PeripheralTypeMax;
    let mut UseSignature: u8 = (FALSE) as u8;

    RtlInitEmptyUnicodeString(
        &mut Token,
        TokenBuffer.as_mut_ptr(),
        core::mem::size_of::<[u16; 50]>() as u16,
    );

    p = *ArcNamePath;

    p = ArcGetNextTokenU(p, &mut Token, &mut AdapterKey);
    if p.is_null() {
        DPRINT1!("No adapter specified!\n");
        return STATUS_OBJECT_PATH_SYNTAX_BAD;
    }

    'quit: {
        if _wcsicmp(Token.Buffer, RUSTCONV_WSTR_7BC572EA8FA4.as_ptr()) == 0 {
            UseSignature = (TRUE) as u8;
            AdapterType_local = ScsiAdapter;
        } else {
            AdapterType_local = ArcMatchToken_UStr(&Token, AdapterTypes_U.as_ptr()) as ADAPTER_TYPE;
            if AdapterType_local as u32 >= AdapterTypeMax as u32 {
                DPRINT1!("Invalid adapter type %wZ\n", &Token);
                return STATUS_OBJECT_NAME_INVALID;
            }

            if AdapterType_local == NetAdapter || AdapterType_local == RamdiskAdapter {
                if AdapterType_local == NetAdapter {
                    DPRINT1!(
                        "%S(%lu) path is not supported!\n",
                        AdapterTypes_U[AdapterType_local as usize],
                        AdapterKey,
                    );
                    return STATUS_NOT_SUPPORTED;
                }

                // C: `goto Quit;` -- no controller or peripheral node follows.
                break 'quit;
            }
        }

        p = ArcGetNextTokenU(p, &mut Token, &mut ControllerKey);
        if p.is_null() {
            DPRINT1!(
                "%S(%lu) adapter doesn't have a controller!\n",
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
            );
            return STATUS_OBJECT_PATH_SYNTAX_BAD;
        }

        ControllerType = ArcMatchToken_UStr(&Token, ControllerTypes_U.as_ptr()) as CONTROLLER_TYPE;
        if ControllerType as u32 >= ControllerTypeMax as u32 {
            DPRINT1!("Invalid controller type %wZ\n", &Token);
            return STATUS_OBJECT_NAME_INVALID;
        }

        if AdapterType_local == MultiAdapter && ControllerKey != 0 {
            DPRINT1!(
                "%S(%lu) adapter with %S(%lu non-zero), ignored!\n",
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
                ControllerTypes_U[ControllerType as usize],
                ControllerKey,
            );
            ControllerKey = 0;
        }

        if AdapterType_local != ScsiAdapter && ControllerType == CdRomController {
            DPRINT1!(
                "%S(%lu) adapter cannot have a CDROM controller!\n",
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
            );
            return STATUS_OBJECT_PATH_INVALID;
        }

        p = ArcGetNextTokenU(p, &mut Token, &mut PeripheralKey);
        if p.is_null() {
            DPRINT1!(
                "%S(%lu)%S(%lu) adapter-controller doesn't have a peripheral!\n",
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
                ControllerTypes_U[ControllerType as usize],
                ControllerKey,
            );
            return STATUS_OBJECT_PATH_SYNTAX_BAD;
        }

        PeripheralType = ArcMatchToken_UStr(&Token, PeripheralTypes_U.as_ptr()) as PERIPHERAL_TYPE;
        if PeripheralType as u32 >= PeripheralTypeMax as u32 {
            DPRINT1!("Invalid peripheral type %wZ\n", &Token);
            return STATUS_OBJECT_NAME_INVALID;
        }

        if ControllerType == CdRomController && PeripheralType != FDiskPeripheral {
            DPRINT1!(
                "%S(%lu) controller cannot have a %S(%lu) peripheral! (note that we haven't check whether the adapter was SCSI or not)\n",
                ControllerTypes_U[ControllerType as usize],
                ControllerKey,
                PeripheralTypes_U[PeripheralType as usize],
                PeripheralKey,
            );
            return STATUS_OBJECT_PATH_INVALID;
        }

        if AdapterType_local == ScsiAdapter
            && !(PeripheralType == RDiskPeripheral || PeripheralType == FDiskPeripheral)
        {
            DPRINT1!(
                "%S(%lu)%S(%lu) SCSI adapter-controller has an invalid peripheral %S(%lu) !\n",
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
                ControllerTypes_U[ControllerType as usize],
                ControllerKey,
                PeripheralTypes_U[PeripheralType as usize],
                PeripheralKey,
            );
            return STATUS_OBJECT_PATH_INVALID;
        }

        q = ArcGetNextTokenU(p, &mut Token, &mut PartitionNumber);
        if !q.is_null() && _wcsicmp(Token.Buffer, RUSTCONV_WSTR_25C59D0378B0.as_ptr()) == 0 {
            p = q;
        } else {
            PartitionNumber = 0;
        }
    }

    *ArcNamePath = p;
    *pAdapterKey = AdapterKey;
    *pControllerKey = ControllerKey;
    *pPeripheralKey = PeripheralKey;
    *pPartitionNumber = PartitionNumber;
    *pAdapterType = AdapterType_local;
    *pControllerType = ControllerType;
    *pPeripheralType = PeripheralType;
    *pUseSignature = UseSignature;

    return STATUS_SUCCESS;
}

unsafe extern "C" fn ResolveArcNameNtSymLink(
    NtName: PUNICODE_STRING,
    ArcName: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut DirectoryHandle: HANDLE = core::mem::zeroed();
    let mut LinkHandle: HANDLE = core::mem::zeroed();
    let mut ArcNameDir: _UNICODE_STRING = core::mem::zeroed();

    if (*NtName).MaximumLength < core::mem::size_of_val(&UNICODE_NULL) as u16 {
        return STATUS_BUFFER_TOO_SMALL;
    }

    RtlInitUnicodeString(&mut ArcNameDir, RUSTCONV_WSTR_8CCE59A651CE.as_ptr());
    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut ArcNameDir,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = NtOpenDirectoryObject(
        &mut DirectoryHandle,
        DIRECTORY_ALL_ACCESS,
        &mut ObjectAttributes,
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtOpenDirectoryObject(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as PCSTR,
            &mut ArcNameDir,
            Status,
        );
        return Status;
    }

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        ArcName,
        OBJ_CASE_INSENSITIVE,
        DirectoryHandle,
        core::ptr::null_mut(),
    );
    Status = NtOpenSymbolicLinkObject(
        &mut LinkHandle,
        SYMBOLIC_LINK_QUERY,
        &mut ObjectAttributes,
    );

    NtClose(DirectoryHandle);

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtOpenSymbolicLinkObject(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as PCSTR,
            ArcName,
            Status,
        );
        return Status;
    }

    (*NtName).MaximumLength -= core::mem::size_of_val(&UNICODE_NULL) as u16;

    Status = NtQuerySymbolicLinkObject(LinkHandle, NtName, core::ptr::null_mut());
    NtClose(LinkHandle);

    (*NtName).MaximumLength += core::mem::size_of_val(&UNICODE_NULL) as u16;

    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtQuerySymbolicLinkObject(%wZ) failed, Status 0x%08lx\n\0".as_ptr() as PCSTR,
            ArcName,
            Status,
        );
    } else {
        *(*NtName).Buffer.offset(((*NtName).Length / core::mem::size_of::<u16>() as u16) as isize) =
            UNICODE_NULL;
    }

    Status
}

unsafe extern "C" fn ResolveArcNameManually(
    mut NtName: PUNICODE_STRING,
    mut ArcNamePath: *mut PCWSTR,
    mut PartList: PPARTLIST,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut AdapterKey: ULONG = 0;
    let mut ControllerKey: ULONG = 0;
    let mut PeripheralKey: ULONG = 0;
    let mut PartitionNumber: ULONG = 0;
    let mut AdapterType_local: ADAPTER_TYPE = EisaAdapter;
    let mut ControllerType: CONTROLLER_TYPE = DiskController;
    let mut PeripheralType: PERIPHERAL_TYPE = RDiskPeripheral;
    let mut UseSignature: BOOLEAN = 0;
    let mut NameLength: SIZE_T = 0;

    if ((*NtName).MaximumLength as usize) < core::mem::size_of_val(&UNICODE_NULL) {
        return STATUS_BUFFER_TOO_SMALL;
    }

    Status = ParseArcName(
        ArcNamePath,
        core::ptr::addr_of_mut!(AdapterKey),
        core::ptr::addr_of_mut!(ControllerKey),
        core::ptr::addr_of_mut!(PeripheralKey),
        core::ptr::addr_of_mut!(PartitionNumber),
        core::ptr::addr_of_mut!(AdapterType_local),
        core::ptr::addr_of_mut!(ControllerType),
        core::ptr::addr_of_mut!(PeripheralType),
        core::ptr::addr_of_mut!(UseSignature),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        return Status;
    }

    if (AdapterType_local as u32) == (NetAdapter as u32) || (AdapterType_local as u32) == (RamdiskAdapter as u32) {
        if (AdapterType_local as u32) == (NetAdapter as u32) {
            DPRINT1!(
                b"%S(%lu) path is not supported!\n\0".as_ptr() as *const i8,
                AdapterTypes_U[AdapterType_local as usize],
                AdapterKey,
            );
            return STATUS_NOT_SUPPORTED;
        }

        Status = RtlStringCbPrintfW!(
            (*NtName).Buffer,
            (*NtName).MaximumLength as usize,
            RUSTCONV_WSTR_7730A3AD01A7.as_ptr(),
            AdapterKey,
        );
    } else if (ControllerType as u32) == (CdRomController as u32) {
        Status = RtlStringCbPrintfW!(
            (*NtName).Buffer,
            (*NtName).MaximumLength as usize,
            RUSTCONV_WSTR_62AF54F8206D.as_ptr(),
            ControllerKey,
        );
    } else if (PeripheralType as u32) == (CdRomPeripheral as u32) {
        Status = RtlStringCbPrintfW!(
            (*NtName).Buffer,
            (*NtName).MaximumLength as usize,
            RUSTCONV_WSTR_A969AFDFA6D5.as_ptr(),
            PeripheralKey,
        );
    } else if (PeripheralType as u32) == (FDiskPeripheral as u32) {
        Status = RtlStringCbPrintfW!(
            (*NtName).Buffer,
            (*NtName).MaximumLength as usize,
            RUSTCONV_WSTR_524C5CAE6CCF.as_ptr(),
            PeripheralKey,
        );
    } else if (PeripheralType as u32) == (RDiskPeripheral as u32) {
        let mut DiskEntry: PDISKENTRY = core::ptr::null_mut();
        let mut PartEntry: PPARTENTRY = core::ptr::null_mut();

        if UseSignature != 0 {
            DiskEntry = GetDiskBySignature(PartList, AdapterKey);
        } else {
            DiskEntry = GetDiskBySCSI(
                PartList,
                AdapterKey as u16,
                ControllerKey as u16,
                PeripheralKey as u16,
            );
        }
        if DiskEntry.is_null() {
            return STATUS_OBJECT_PATH_NOT_FOUND;
        }

        if PartitionNumber != 0 {
            PartEntry = GetPartition(DiskEntry, PartitionNumber);
            if PartEntry.is_null() {
                return STATUS_OBJECT_PATH_NOT_FOUND;
            }
        }

        Status = RtlStringCbPrintfW!(
            (*NtName).Buffer,
            (*NtName).MaximumLength as usize,
            RUSTCONV_WSTR_18CA6AF5ACA2.as_ptr(),
            (*DiskEntry).DiskNumber,
            PartitionNumber,
        );
    }

    if ((NT_SUCCESS!(Status)) == 0) {
        *(*NtName).Buffer = UNICODE_NULL as u16;
        (*NtName).Length = 0 as u16;
        return Status;
    }

    NameLength = (wcslen((*NtName).Buffer as *const u16)) as _;
    if NameLength > ((UNICODE_STRING_MAX_CHARS as usize)) as _ {
        return STATUS_NAME_TOO_LONG;
    }

    (*NtName).Length = ((NameLength as u16 as usize).wrapping_mul(core::mem::size_of::<u16>() as usize)) as u16;

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn ArcPathToNtPath(
    NtPath: PUNICODE_STRING,
    ArcPath: PCWSTR,
    PartList: PPARTLIST,
) -> BOOLEAN {
    let mut Status: NTSTATUS;
    let mut BeginOfPath: PCWSTR;
    let mut ArcName: _UNICODE_STRING = core::mem::zeroed();
    let mut PathLength: usize;

    if ((*NtPath).MaximumLength as usize) < core::mem::size_of_val(&UNICODE_NULL) {
        return (FALSE) as u8;
    }

    *(*NtPath).Buffer = UNICODE_NULL;
    (*NtPath).Length = 0;

    RtlInitUnicodeString(&mut ArcName, ArcPath);
    BeginOfPath = wcschr(ArcName.Buffer, (OBJ_NAME_PATH_SEPARATOR) as _);
    if !BeginOfPath.is_null() {
        ArcName.Length = (BeginOfPath as usize - ArcName.Buffer as usize) as u16;
    }

    Status = ResolveArcNameNtSymLink(NtPath, &mut ArcName);
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"ResolveArcNameNtSymLink(ArcName = '%wZ') for ArcPath = '%S' failed, Status 0x%08lx\n\0"
                .as_ptr() as PCSTR,
            &mut ArcName,
            ArcPath,
            Status,
        );

        if PartList.is_null() {
            DPRINT1!(b"PartList == NULL, cannot perform a manual resolution\n\0".as_ptr() as PCSTR);
            return (FALSE) as u8;
        }

        *(*NtPath).Buffer = UNICODE_NULL;
        (*NtPath).Length = 0;

        BeginOfPath = ArcPath;
        Status = ResolveArcNameManually(NtPath, &mut BeginOfPath, PartList);
        if ((NT_SUCCESS!(Status)) == 0) {
            DPRINT1!(
                b"ResolveArcNameManually(ArcPath = '%S') failed, Status 0x%08lx\n\0".as_ptr()
                    as PCSTR,
                ArcPath,
                Status,
            );
            return (FALSE) as u8;
        }
    }

    if !BeginOfPath.is_null() && *BeginOfPath != 0 {
        Status = ConcatPaths(
            (*NtPath).Buffer,
            (*NtPath).MaximumLength as usize / core::mem::size_of::<u16>(),
            1,
            BeginOfPath,
        );
        if ((NT_SUCCESS!(Status)) == 0) {
            return (FALSE) as u8;
        }
    }

    PathLength = wcslen((*NtPath).Buffer);
    if PathLength > (UNICODE_STRING_MAX_CHARS) as _ {
        return (FALSE) as u8;
    }

    (*NtPath).Length = (PathLength as u16) * core::mem::size_of::<u16>() as u16;

    return (TRUE) as u8;
}
