use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::PagedPool;
use ntkrnl_sys::ddk::{GUID, IoWMIOpenBlock, IoWMIQueryAllData};

// HANDLE = PVOID (ntdef.h). PVOID comes from base::* above.
type HANDLE = PVOID;
// Only FL_INITHACK is used here (FL_NOLOOP entries in the C table are commented
// out). Glob-importing the i8042 header would re-introduce UNICODE_STRING (also
// in base) → E0659, so import the one constant by name.
use crate::i8042prt_h::FL_INITHACK;

// PCSTR is not defined in ntkrnl-sys; PCHAR is (base.rs). Trivial char ptr.
type PCSTR = *const i8;

// Zw* registry syscalls take this file's local UNICODE_STRING/OBJECT_ATTRIBUTES
// (layout-compatible with the kernel's). The bindgen ddk Zw* use the bindgen
// _UNICODE_STRING/_OBJECT_ATTRIBUTES types, which would mismatch our locals;
// declare our own extern "system" against ntoskrnl's exported symbols instead.
extern "system" {
    fn ZwCreateKey(
        KeyHandle: *mut HANDLE,
        DesiredAccess: ULONG,
        ObjectAttributes: *mut OBJECT_ATTRIBUTES,
        TitleIndex: ULONG,
        Class: *mut UNICODE_STRING,
        CreateOptions: ULONG,
        Disposition: *mut ULONG,
    ) -> NTSTATUS;
    fn ZwSetValueKey(
        KeyHandle: HANDLE,
        ValueName: *mut UNICODE_STRING,
        TitleIndex: ULONG,
        Type: ULONG,
        Data: PVOID,
        DataSize: ULONG,
    ) -> NTSTATUS;
    fn ZwClose(Handle: HANDLE) -> NTSTATUS;
}

// ---- dmilib.h constants (NOT in ntoskrnl.h, so not in bindgen) ----
// Index slots in the parsed DMI string table; ID_NONE marks an unused match
// entry. From sdk/include/reactos/libs/dmilib/dmilib.h.
const ID_STRINGS_MAX: usize = 19;
const ID_NONE: ULONG = 0xFFFFFFFF;

// wmistr.h
const WMIGUID_QUERY: ULONG = 0x0001;

// wdm.h registry/object constants (define locally to stay self-contained).
const KEY_WRITE: ULONG = 0x20006;
const REG_OPTION_VOLATILE: ULONG = 0x00000001;
const REG_BINARY: ULONG = 3;
const OBJ_KERNEL_HANDLE: ULONG = 0x00000200;
const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;

// ParseSMBiosTables is now a Rust port in crate::dmi (was cmake's libdmilib.a —
// dropping the extern lets the i8042prt cargo link shed that last .a). strcmp
// from CRT is still external.
use crate::dmi::parse_smbios_tables;
extern "C" {
    fn strcmp(s1: PCSTR, s2: PCSTR) -> i32;
}

// WNODE_ALL_DATA / OBJECT_ATTRIBUTES minimal mirrors (wmistr.h / ntdef.h). Only
// the fields hwhacks.c touches are needed; layout matches the C structs.
#[repr(C)]
struct WNODE_HEADER {
    BufferSize: ULONG,
    ProviderId: ULONG,
    HistoricalContext: u64,
    KernelHandle_TimeStamp: i64,
    Guid: GUID,
    Version: ULONG,
    Linkage: ULONG,
}
#[repr(C)]
struct WNODE_ALL_DATA {
    WnodeHeader: WNODE_HEADER,
    DataBlockOffset: ULONG,
    InstanceCount: ULONG,
    OffsetInstanceNameOffsets: ULONG,
    FixedInstanceSize: ULONG, // union with OffsetInstanceDataAndLength
}
type PWNODE_ALL_DATA = *mut WNODE_ALL_DATA;

#[repr(C)]
struct OBJECT_ATTRIBUTES {
    Length: ULONG,
    RootDirectory: HANDLE,
    ObjectName: *mut UNICODE_STRING,
    Attributes: ULONG,
    SecurityDescriptor: PVOID,
    SecurityQualityOfService: PVOID,
}

// ---- file-scope preamble ----
pub const MAX_MATCH_ENTRIES: usize = 3;

pub const SMBIOS_DATA_GUID: GUID = GUID {
    Data1: 0x8F680850,
    Data2: 0xA584,
    Data3: 0x11d1,
    Data4: [0xBF, 0x38, 0x00, 0xA0, 0xC9, 0x06, 0x29, 0x10],
};

// _ID_STRINGS slot indices (dmilib.h) — these MUST match crate::dmi's slots:
// ParseSMBiosTables writes the system vendor/product into slots 4/5, and this
// matcher reads them back. The original translation fabricated 0x01/0x02 here,
// so quirk matching read the BIOS_VERSION/BIOS_DATE slots instead and never
// fired (inert under QEMU's non-Dell SMBIOS, hence undetected). Fixed to 4/5.
pub const SYS_VENDOR: ULONG = 0x04;
pub const SYS_PRODUCT: ULONG = 0x05;

#[no_mangle]
pub static MSSmBios_RawSMBiosTables_GUID: GUID = SMBIOS_DATA_GUID;

#[no_mangle]
pub static mut i8042SMBiosTables: PVOID = core::ptr::null_mut();

#[no_mangle]
pub static mut i8042HwFlags: ULONG = 0;

#[repr(C)]
pub struct _MATCHENTRY {
    pub Type: ULONG,
    pub String: PCHAR,
}
pub type MATCHENTRY = _MATCHENTRY;
pub type PMATCHENTRY = *mut _MATCHENTRY;

#[repr(C)]
pub struct _HARDWARE_TABLE {
    pub MatchEntries: [MATCHENTRY; MAX_MATCH_ENTRIES],
    pub Flags: ULONG,
}
pub type HARDWARE_TABLE = _HARDWARE_TABLE;
pub type PHARDWARE_TABLE = *mut _HARDWARE_TABLE;

// The C table is `const HARDWARE_TABLE[]` of read-only string-literal pointers;
// it is never mutated. Raw pointers make the struct non-Sync, so assert Sync
// for the static (sound: read-only, points at 'static string literals).
unsafe impl Sync for _HARDWARE_TABLE {}

#[no_mangle]
pub static i8042HardwareTable: [HARDWARE_TABLE; 15] = [
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Microsoft Corporation\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Virtual Machine\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Inspiron 6000                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D410                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D430                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D530                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D531                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D600                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D610                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D620                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D630                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D810                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude D820                   \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude E4300                  \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude E4310                  \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
    HARDWARE_TABLE {
        MatchEntries: [
            MATCHENTRY { Type: SYS_VENDOR, String: b"Dell Inc.\0".as_ptr() as PCHAR },
            MATCHENTRY { Type: SYS_PRODUCT, String: b"Latitude E6400                  \0".as_ptr() as PCHAR },
            MATCHENTRY { Type: 0, String: core::ptr::null_mut() },
        ],
        Flags: FL_INITHACK,
    },
];

unsafe fn i8042ParseSMBiosTables(SMBiosTables: PVOID, TableSize: ULONG) {
    let mut Strings: [PCHAR; ID_STRINGS_MAX] = [core::ptr::null_mut(); ID_STRINGS_MAX];

    parse_smbios_tables(SMBiosTables, TableSize, Strings.as_mut_ptr());

    let table_size = i8042HardwareTable.len();

    let mut i = 0usize;
    while i < table_size {
        let mut j = 0usize;
        while j < MAX_MATCH_ENTRIES {
            let Type = i8042HardwareTable[i].MatchEntries[j].Type;

            if Type != ID_NONE {
                if Strings[Type as usize].is_null()
                    || strcmp(
                        i8042HardwareTable[i].MatchEntries[j].String,
                        Strings[i8042HardwareTable[i].MatchEntries[j].Type as usize],
                    ) != 0
                {
                    break;
                }
            }
            j += 1;
        }

        if j == MAX_MATCH_ENTRIES {
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!(i8042HwFlags),
                i8042HardwareTable[i].Flags,
            );
            break;
        }
        i += 1;
    }
}

unsafe fn i8042StoreSMBiosTables(SMBiosTables: PVOID, TableSize: ULONG) {
    let mssmbiosKeyName_buffer: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
        0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065,
        0x005C, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
        0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074,
        0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073,
        0x005C, 0x006D, 0x0073, 0x0073, 0x006D, 0x0062, 0x0069, 0x006F, 0x0073,
        0x0000,
    ];
    let mut mssmbiosKeyName: UNICODE_STRING = core::mem::zeroed();
    mssmbiosKeyName.Length = ((mssmbiosKeyName_buffer.len() - 1) * 2) as u16;
    mssmbiosKeyName.MaximumLength = (mssmbiosKeyName_buffer.len() * 2) as u16;
    mssmbiosKeyName.Buffer = mssmbiosKeyName_buffer.as_ptr() as *mut u16;

    let DataName_buffer: &[u16] = &[0x0044, 0x0061, 0x0074, 0x0061, 0x0000];
    let mut DataName: UNICODE_STRING = core::mem::zeroed();
    DataName.Length = ((DataName_buffer.len() - 1) * 2) as u16;
    DataName.MaximumLength = (DataName_buffer.len() * 2) as u16;
    DataName.Buffer = DataName_buffer.as_ptr() as *mut u16;

    let ValueName_buffer: &[u16] = &[
        0x0053, 0x004D, 0x0042, 0x0069, 0x006F, 0x0073, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
    ];
    let mut ValueName: UNICODE_STRING = core::mem::zeroed();
    ValueName.Length = ((ValueName_buffer.len() - 1) * 2) as u16;
    ValueName.MaximumLength = (ValueName_buffer.len() * 2) as u16;
    ValueName.Buffer = ValueName_buffer.as_ptr() as *mut u16;

    let mut ObjectAttributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let Status;

    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(mssmbiosKeyName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &mut KeyHandle,
        KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut _,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return;
    }

    ObjectAttributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(DataName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwCreateKey(
        &mut SubKeyHandle,
        KEY_WRITE,
        core::ptr::addr_of_mut!(ObjectAttributes) as *mut _,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        ZwClose(KeyHandle);
        return;
    }

    ZwSetValueKey(
        SubKeyHandle,
        core::ptr::addr_of_mut!(ValueName),
        0,
        REG_BINARY,
        SMBiosTables,
        TableSize,
    );

    ZwClose(SubKeyHandle);
    ZwClose(KeyHandle);
}

#[no_mangle]
pub unsafe extern "system" fn i8042InitializeHwHacks() {
    let mut Status: NTSTATUS;
    let mut DataBlockObject: PVOID = core::ptr::null_mut();
    let AllData: PWNODE_ALL_DATA;
    let mut BufferSize: ULONG;

    Status = IoWMIOpenBlock(
        core::ptr::addr_of!(MSSmBios_RawSMBiosTables_GUID) as *mut GUID,
        WMIGUID_QUERY,
        core::ptr::addr_of_mut!(DataBlockObject),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    BufferSize = 0;
    Status = IoWMIQueryAllData(
        DataBlockObject,
        core::ptr::addr_of_mut!(BufferSize),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return;
    }

    AllData = ExAllocatePoolWithTag(PagedPool, BufferSize as usize, u32::from_le_bytes(*b"SMTB"))
        as PWNODE_ALL_DATA;
    if AllData.is_null() {
        return;
    }

    Status = IoWMIQueryAllData(
        DataBlockObject,
        core::ptr::addr_of_mut!(BufferSize),
        AllData as PVOID,
    );
    if !NT_SUCCESS(Status) {
        ExFreePoolWithTag(AllData as PVOID, u32::from_le_bytes(*b"SMTB"));
        return;
    }

    // C: AllData + 1 (pointer arithmetic in WNODE_ALL_DATA units).
    i8042StoreSMBiosTables(AllData.add(1) as PVOID, (*AllData).FixedInstanceSize);

    i8042ParseSMBiosTables(AllData.add(1) as PVOID, (*AllData).WnodeHeader.BufferSize);

    ExFreePoolWithTag(AllData as PVOID, u32::from_le_bytes(*b"SMTB"));
}
