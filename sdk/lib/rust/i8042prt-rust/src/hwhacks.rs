use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{GUID, IoWMIOpenBlock, IoWMIQueryAllData, _OBJECT_ATTRIBUTES, Type, ZwClose, ZwCreateKey, ZwSetValueKey};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_MATCH_ENTRIES: usize = 3;

pub const SMBIOS_DATA_GUID: GUID = GUID {
    Data1: 0x8F680850,
    Data2: 0xA584,
    Data3: 0x11d1,
    Data4: [0xBF, 0x38, 0x00, 0xA0, 0xC9, 0x06, 0x29, 0x10],
};

pub const SYS_VENDOR: ULONG = 0x01;
pub const SYS_PRODUCT: ULONG = 0x02;

pub const FL_INITHACK: ULONG = 0x01;
pub const FL_NOLOOP: ULONG = 0x02;

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
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Strings: [PCHAR; ID_STRINGS_MAX as usize] = [core::ptr::null_mut(); ID_STRINGS_MAX as usize];

    ParseSMBiosTables(SMBiosTables, TableSize, Strings.as_mut_ptr());

    let table_size = (core::mem::size_of_val(&i8042HardwareTable) / core::mem::size_of::<HARDWARE_TABLE>()) as ULONG;

    i = 0;
    while i < table_size {
        j = 0;
        while j < MAX_MATCH_ENTRIES {
            let Type = i8042HardwareTable[i as usize].MatchEntries[j as usize].Type;

            if Type != ID_NONE {
                if Strings[Type as usize].is_null()
                    || strcmp(
                        i8042HardwareTable[i as usize].MatchEntries[j as usize].String,
                        Strings[i8042HardwareTable[i as usize].MatchEntries[j as usize].Type as usize],
                    ) != 0
                {
                    break;
                }
            }
            j += 1;
        }

        if j == MAX_MATCH_ENTRIES {
            i8042HwFlags = i8042HardwareTable[i as usize].Flags;
            break;
        }
        i += 1;
    }
}

unsafe fn i8042StoreSMBiosTables(
    SMBiosTables: PVOID,
    TableSize: ULONG,
) {
    let mssmbiosKeyName_buffer: &[u16] = &[
        0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079,
        0x005C, 0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065,
        0x005C, 0x0053, 0x0059, 0x0053, 0x0054, 0x0045, 0x004D,
        0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053, 0x0065, 0x0074,
        0x005C, 0x0053, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0073,
        0x005C, 0x006D, 0x0073, 0x0073, 0x006D, 0x0062, 0x0069, 0x006F, 0x0073,
        0x0000,
    ];
    let mut mssmbiosKeyName: _UNICODE_STRING = core::mem::zeroed();
    mssmbiosKeyName.Length = ((mssmbiosKeyName_buffer.len() - 1) * 2) as u16;
    mssmbiosKeyName.MaximumLength = (mssmbiosKeyName_buffer.len() * 2) as u16;
    mssmbiosKeyName.Buffer = mssmbiosKeyName_buffer.as_ptr() as *mut u16;

    let DataName_buffer: &[u16] = &[0x0044, 0x0061, 0x0074, 0x0061, 0x0000];
    let mut DataName: _UNICODE_STRING = core::mem::zeroed();
    DataName.Length = ((DataName_buffer.len() - 1) * 2) as u16;
    DataName.MaximumLength = (DataName_buffer.len() * 2) as u16;
    DataName.Buffer = DataName_buffer.as_ptr() as *mut u16;

    let ValueName_buffer: &[u16] = &[
        0x0053, 0x004D, 0x0042, 0x0069, 0x006F, 0x0073, 0x0044, 0x0061, 0x0074, 0x0061, 0x0000,
    ];
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    ValueName.Length = ((ValueName_buffer.len() - 1) * 2) as u16;
    ValueName.MaximumLength = (ValueName_buffer.len() * 2) as u16;
    ValueName.Buffer = ValueName_buffer.as_ptr() as *mut u16;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let Status;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(mssmbiosKeyName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        &mut KeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_VOLATILE,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(DataName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = ZwCreateKey(
        &mut SubKeyHandle,
        KEY_WRITE,
        &mut ObjectAttributes,
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
        &mut ValueName,
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

    AllData = ExAllocatePoolWithTag(PagedPool, BufferSize, u32::from_le_bytes(*b"SMTB")) as PWNODE_ALL_DATA;
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

    i8042StoreSMBiosTables(
        AllData.add(1) as PVOID,
        (*AllData).FixedInstanceSize,
    );

    i8042ParseSMBiosTables(
        AllData.add(1) as PVOID,
        (*AllData).WnodeHeader.BufferSize,
    );

    ExFreePoolWithTag(AllData as PVOID, u32::from_le_bytes(*b"SMTB"));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/hwhacks.c
//  * PURPOSE:     Mouse specific functions
//  * PROGRAMMERS: Timo Kreuzer (timo.kreuzer@reactos.org)
//  * REFERENCES:  - http://www.dmtf.org/sites/default/files/standards/documents/DSP0134_3.0.0.pdf
//  *              -
//  */
//
// #include "i8042prt.h"
// #include <wmiguid.h>
// #include <wmidata.h>
// #include <wmistr.h>
// #include <dmilib.h>
//
// #define NDEBUG
// #include <debug.h>
//
// const GUID MSSmBios_RawSMBiosTables_GUID = SMBIOS_DATA_GUID;
// PVOID i8042SMBiosTables;
// ULONG i8042HwFlags;
//
// typedef struct _MATCHENTRY
// {
//     ULONG Type;
//     PCHAR String;
// } MATCHENTRY;
//
// #define MAX_MATCH_ENTRIES 3
// typedef struct _HARDWARE_TABLE
// {
//     MATCHENTRY MatchEntries[MAX_MATCH_ENTRIES];
//     ULONG Flags;
// } HARDWARE_TABLE;
//
// const HARDWARE_TABLE i8042HardwareTable[] =
// {
// //    { {{BOARD_VENDOR, "RIOWORKS"}, {BOARD_NAME, "HDAMB"}, {BOARD_VERSION, "Rev E"}}, FL_NOLOOP },
// //    { {{BOARD_VENDOR, "ASUSTeK Computer Inc."}, {BOARD_NAME, "G1S"}, {BOARD_VERSION, "1.0"}}, FL_NOLOOP },
//
//     { {{SYS_VENDOR, "Microsoft Corporation"}, {SYS_PRODUCT, "Virtual Machine"}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Inspiron 6000                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D410                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D430                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D530                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D531                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D600                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D610                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D620                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D630                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D810                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude D820                   "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude E4300                  "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude E4310                  "}}, FL_INITHACK },
//     { {{SYS_VENDOR, "Dell Inc."}, {SYS_PRODUCT, "Latitude E6400                  "}}, FL_INITHACK },
//
// };
//
//
//
// static
// VOID
// i8042ParseSMBiosTables(
//     _In_reads_bytes_(TableSize) PVOID SMBiosTables,
//     _In_ ULONG TableSize)
// {
//     ULONG i, j;
//     PCHAR Strings[ID_STRINGS_MAX] = { 0 };
//
//     ParseSMBiosTables(SMBiosTables, TableSize, Strings);
//
// #if 0 // DBG
//     DbgPrint("i8042prt: Dumping DMI data:\n");
//     DbgPrint("BIOS_VENDOR: %s\n", Strings[BIOS_VENDOR]);
//     DbgPrint("BIOS_VERSION: %s\n", Strings[BIOS_VERSION]);
//     DbgPrint("BIOS_DATE: %s\n", Strings[BIOS_DATE]);
//     DbgPrint("SYS_VENDOR: %s\n", Strings[SYS_VENDOR]);
//     DbgPrint("SYS_PRODUCT: %s\n", Strings[SYS_PRODUCT]);
//     DbgPrint("SYS_VERSION: %s\n", Strings[SYS_VERSION]);
//     DbgPrint("SYS_SERIAL: %s\n", Strings[SYS_SERIAL]);
//     DbgPrint("BOARD_VENDOR: %s\n", Strings[BOARD_VENDOR]);
//     DbgPrint("BOARD_NAME: %s\n", Strings[BOARD_NAME]);
//     DbgPrint("BOARD_VERSION: %s\n", Strings[BOARD_VERSION]);
//     DbgPrint("BOARD_SERIAL: %s\n", Strings[BOARD_SERIAL]);
//     DbgPrint("BOARD_ASSET_TAG: %s\n", Strings[BOARD_ASSET_TAG]);
// #endif
//
//     /* Now loop the hardware table to find a match */
//     for (i = 0; i < ARRAYSIZE(i8042HardwareTable); i++)
//     {
//         for (j = 0; j < MAX_MATCH_ENTRIES; j++)
//         {
//             ULONG Type = i8042HardwareTable[i].MatchEntries[j].Type;
//
//             if (Type != ID_NONE)
//             {
//                 /* Check for a match */
//                 if ((Strings[Type] == NULL) ||
//                     strcmp(i8042HardwareTable[i].MatchEntries[j].String,
//                            Strings[i8042HardwareTable[i].MatchEntries[j].Type]))
//                 {
//                     /* Does not match, try next entry */
//                     break;
//                 }
//             }
//         }
//
//         if (j == MAX_MATCH_ENTRIES)
//         {
//             /* All items matched! */
//             i8042HwFlags = i8042HardwareTable[i].Flags;
//             DPRINT("Found match for hw table index %u\n", i);
//             break;
//         }
//     }
// }
//
// static
// VOID
// i8042StoreSMBiosTables(
//     _In_reads_bytes_(TableSize) PVOID SMBiosTables,
//     _In_ ULONG TableSize)
// {
//     static UNICODE_STRING mssmbiosKeyName = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\SYSTEM\\CurrentControlSet\\Services\\mssmbios");
//     static UNICODE_STRING DataName = RTL_CONSTANT_STRING(L"Data");
//     static UNICODE_STRING ValueName = RTL_CONSTANT_STRING(L"SMBiosData");
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE KeyHandle = NULL, SubKeyHandle = NULL;
//     NTSTATUS Status;
//
//     /* Create registry key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &mssmbiosKeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwCreateKey(&KeyHandle,
//                          KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         return;
//     }
//
//     /* Create sub key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &DataName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                KeyHandle,
//                                NULL);
//     Status = ZwCreateKey(&SubKeyHandle,
//                          KEY_WRITE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//
//     if (!NT_SUCCESS(Status))
//     {
//         ZwClose(KeyHandle);
//         return;
//     }
//
//     /* Write value */
//     ZwSetValueKey(SubKeyHandle,
//                   &ValueName,
//                   0,
//                   REG_BINARY,
//                   SMBiosTables,
//                   TableSize);
//
//     ZwClose(SubKeyHandle);
//     ZwClose(KeyHandle);
// }
//
// VOID
// NTAPI
// i8042InitializeHwHacks(
//     VOID)
// {
//     NTSTATUS Status;
//     PVOID DataBlockObject;
//     PWNODE_ALL_DATA AllData;
//     ULONG BufferSize;
//
//     /* Open the data block object for the SMBIOS table */
//     Status = IoWMIOpenBlock(&MSSmBios_RawSMBiosTables_GUID,
//                             WMIGUID_QUERY,
//                             &DataBlockObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoWMIOpenBlock failed: 0x%08lx\n", Status);
//         return;
//     }
//
//     /* Query the required buffer size */
//     BufferSize = 0;
//     Status = IoWMIQueryAllData(DataBlockObject, &BufferSize, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoWMIOpenBlock failed: 0x%08lx\n", Status);
//         return;
//     }
//
//     AllData = ExAllocatePoolWithTag(PagedPool, BufferSize, 'BTMS');
//     if (AllData == NULL)
//     {
//         DPRINT1("Failed to allocate %lu bytes for SMBIOS tables\n", BufferSize);
//         return;
//     }
//
//     /* Query the buffer data */
//     Status = IoWMIQueryAllData(DataBlockObject, &BufferSize, AllData);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoWMIOpenBlock failed: 0x%08lx\n", Status);
//         ExFreePoolWithTag(AllData, 'BTMS');
//         return;
//     }
//
//     /* FIXME: This function should be removed once the mssmbios driver is implemented */
//     /* Store SMBios data in registry */
//     i8042StoreSMBiosTables(AllData + 1,
//                            AllData->FixedInstanceSize);
//     DPRINT1("SMBiosTables HACK, see CORE-14867\n");
//
//     /* Parse the table */
//     i8042ParseSMBiosTables(AllData + 1,
//                            AllData->WnodeHeader.BufferSize);
//
//     /* Free the buffer */
//     ExFreePoolWithTag(AllData, 'BTMS');
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
