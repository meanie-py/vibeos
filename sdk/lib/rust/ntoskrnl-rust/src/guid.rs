//! Materializes the GUID storage that the rest of the kernel references via
//! `extern { static FOO: GUID; }`. Replaces the preprocessor-emitted storage
//! that used to live in ntoskrnl/guid.c (which abused <initguid.h> to make
//! DEFINE_GUID(...) lines emit storage instead of declarations).
//!
//! GUIDs are taken from sdk/include/psdk/{batclass,diskguid}.h and
//! sdk/include/ddk/poclass.h. If anyone ever needs more GUIDs from those
//! headers, add them here — there is no auto-pull mechanism replacing the
//! initguid.h trick.

use ntkrnl_sys::ddk::GUID;

// ---------------------------------------------------------------------------
// diskguid.h — GPT partition type GUIDs.
// (PARTITION_BASIC_DATA_GUID also has a separate Rust-mangled copy in
// fstub/disksup.rs which is intentionally left alone for now.)
// ---------------------------------------------------------------------------

#[no_mangle]
pub static PARTITION_ENTRY_UNUSED_GUID: GUID = GUID {
    Data1: 0x00000000, Data2: 0x0000, Data3: 0x0000,
    Data4: [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
};

#[no_mangle]
pub static PARTITION_SYSTEM_GUID: GUID = GUID {
    Data1: 0xC12A7328, Data2: 0xF81F, Data3: 0x11D2,
    Data4: [0xBA, 0x4B, 0x00, 0xA0, 0xC9, 0x3E, 0xC9, 0x3B],
};

#[no_mangle]
pub static PARTITION_MSFT_RESERVED_GUID: GUID = GUID {
    Data1: 0xE3C9E316, Data2: 0x0B5C, Data3: 0x4DB8,
    Data4: [0x81, 0x7D, 0xF9, 0x2D, 0xF0, 0x02, 0x15, 0xAE],
};

#[no_mangle]
pub static PARTITION_BASIC_DATA_GUID: GUID = GUID {
    Data1: 0xEBD0A0A2, Data2: 0xB9E5, Data3: 0x4433,
    Data4: [0x87, 0xC0, 0x68, 0xB6, 0xB7, 0x26, 0x99, 0xC7],
};

#[no_mangle]
pub static PARTITION_LDM_METADATA_GUID: GUID = GUID {
    Data1: 0x5808C8AA, Data2: 0x7E8F, Data3: 0x42E0,
    Data4: [0x85, 0xD2, 0xE1, 0xE9, 0x04, 0x34, 0xCF, 0xB3],
};

#[no_mangle]
pub static PARTITION_LDM_DATA_GUID: GUID = GUID {
    Data1: 0xAF9B60A0, Data2: 0x1431, Data3: 0x4F62,
    Data4: [0xBC, 0x68, 0x33, 0x11, 0x71, 0x4A, 0x69, 0xAD],
};

// ---------------------------------------------------------------------------
// poclass.h — power management class GUIDs.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static GUID_CLASS_INPUT: GUID = GUID {
    Data1: 0x4D1E55B2, Data2: 0xF16F, Data3: 0x11CF,
    Data4: [0x88, 0xCB, 0x00, 0x11, 0x11, 0x00, 0x00, 0x30],
};

#[no_mangle]
pub static GUID_DEVICE_LID: GUID = GUID {
    Data1: 0x4AFA3D52, Data2: 0x74A7, Data3: 0x11d0,
    Data4: [0xbe, 0x5e, 0x00, 0xA0, 0xC9, 0x06, 0x28, 0x57],
};

#[no_mangle]
pub static GUID_DEVICE_MEMORY: GUID = GUID {
    Data1: 0x3fd0f03d, Data2: 0x92e0, Data3: 0x45fb,
    Data4: [0xb7, 0x5c, 0x5e, 0xd8, 0xff, 0xb0, 0x10, 0x21],
};

#[no_mangle]
pub static GUID_DEVICE_MESSAGE_INDICATOR: GUID = GUID {
    Data1: 0xCD48A365, Data2: 0xfa94, Data3: 0x4ce2,
    Data4: [0xa2, 0x32, 0xa1, 0xb7, 0x64, 0xe5, 0xd8, 0xb4],
};

#[no_mangle]
pub static GUID_DEVICE_PROCESSOR: GUID = GUID {
    Data1: 0x97fadb10, Data2: 0x4e33, Data3: 0x40ae,
    Data4: [0x35, 0x9c, 0x8b, 0xef, 0x02, 0x9d, 0xbd, 0xd0],
};

#[no_mangle]
pub static GUID_DEVICE_SYS_BUTTON: GUID = GUID {
    Data1: 0x4AFA3D53, Data2: 0x74A7, Data3: 0x11d0,
    Data4: [0xbe, 0x5e, 0x00, 0xA0, 0xC9, 0x06, 0x28, 0x57],
};

#[no_mangle]
pub static GUID_DEVICE_THERMAL_ZONE: GUID = GUID {
    Data1: 0x4AFA3D51, Data2: 0x74A7, Data3: 0x11d0,
    Data4: [0xbe, 0x5e, 0x00, 0xA0, 0xC9, 0x06, 0x28, 0x57],
};

#[no_mangle]
pub static GUID_DEVICE_FAN: GUID = GUID {
    Data1: 0x05ecd13d, Data2: 0x81da, Data3: 0x4a2a,
    Data4: [0x8a, 0x4c, 0x52, 0x4f, 0x23, 0xdd, 0x4d, 0xc9],
};

// ---------------------------------------------------------------------------
// batclass.h — battery class GUIDs.
// ---------------------------------------------------------------------------

#[no_mangle]
pub static GUID_DEVICE_BATTERY: GUID = GUID {
    Data1: 0x72631e54, Data2: 0x78A4, Data3: 0x11d0,
    Data4: [0xbc, 0xf7, 0x00, 0xaa, 0x00, 0xb7, 0xb3, 0x2a],
};

#[no_mangle]
pub static BATTERY_STATUS_WMI_GUID: GUID = GUID {
    Data1: 0xfc4670d1, Data2: 0xebbf, Data3: 0x416e,
    Data4: [0x87, 0xce, 0x37, 0x4a, 0x4e, 0xbc, 0x11, 0x1a],
};

#[no_mangle]
pub static BATTERY_RUNTIME_WMI_GUID: GUID = GUID {
    Data1: 0x535a3767, Data2: 0x1ac2, Data3: 0x49bc,
    Data4: [0xa0, 0x77, 0x3f, 0x7a, 0x02, 0xe4, 0x0a, 0xec],
};

#[no_mangle]
pub static BATTERY_TEMPERATURE_WMI_GUID: GUID = GUID {
    Data1: 0x1a52a14d, Data2: 0xadce, Data3: 0x4a44,
    Data4: [0x9a, 0x3e, 0xc8, 0xd8, 0xf1, 0x5f, 0xf2, 0xc2],
};

#[no_mangle]
pub static BATTERY_FULL_CHARGED_CAPACITY_WMI_GUID: GUID = GUID {
    Data1: 0x40b40565, Data2: 0x96f7, Data3: 0x4435,
    Data4: [0x86, 0x94, 0x97, 0xe0, 0xe4, 0x39, 0x59, 0x05],
};

#[no_mangle]
pub static BATTERY_CYCLE_COUNT_WMI_GUID: GUID = GUID {
    Data1: 0xef98db24, Data2: 0x0014, Data3: 0x4c25,
    Data4: [0xa5, 0x0b, 0xc7, 0x24, 0xae, 0x5c, 0xd3, 0x71],
};

#[no_mangle]
pub static BATTERY_STATIC_DATA_WMI_GUID: GUID = GUID {
    Data1: 0x05e1e463, Data2: 0xe4e2, Data3: 0x4ea9,
    Data4: [0x80, 0xcb, 0x9b, 0xd4, 0xb3, 0xca, 0x06, 0x55],
};

#[no_mangle]
pub static BATTERY_STATUS_CHANGE_WMI_GUID: GUID = GUID {
    Data1: 0xcddfa0c3, Data2: 0x7c5b, Data3: 0x4e43,
    Data4: [0xa0, 0x34, 0x05, 0x9f, 0xa5, 0xb8, 0x43, 0x64],
};

#[no_mangle]
pub static BATTERY_TAG_CHANGE_WMI_GUID: GUID = GUID {
    Data1: 0x5e1f6e19, Data2: 0x8786, Data3: 0x4d23,
    Data4: [0x94, 0xfc, 0x9e, 0x74, 0x6b, 0xd5, 0xd8, 0x88],
};

// Force the linker to pull this CGU out of the staticlib archive even though
// the only references come from C-side `extern GUID` declarations. Without
// this, the GUID-defining CGU may not get pulled.
mod _force_guid_symbols {
    use super::*;
    #[repr(transparent)]
    struct SyncPtr(*const GUID);
    unsafe impl Sync for SyncPtr {}

    #[used]
    static REFS: [SyncPtr; 23] = [
        SyncPtr(&PARTITION_ENTRY_UNUSED_GUID),
        SyncPtr(&PARTITION_SYSTEM_GUID),
        SyncPtr(&PARTITION_MSFT_RESERVED_GUID),
        SyncPtr(&PARTITION_BASIC_DATA_GUID),
        SyncPtr(&PARTITION_LDM_METADATA_GUID),
        SyncPtr(&PARTITION_LDM_DATA_GUID),
        SyncPtr(&GUID_CLASS_INPUT),
        SyncPtr(&GUID_DEVICE_LID),
        SyncPtr(&GUID_DEVICE_MEMORY),
        SyncPtr(&GUID_DEVICE_MESSAGE_INDICATOR),
        SyncPtr(&GUID_DEVICE_PROCESSOR),
        SyncPtr(&GUID_DEVICE_SYS_BUTTON),
        SyncPtr(&GUID_DEVICE_THERMAL_ZONE),
        SyncPtr(&GUID_DEVICE_FAN),
        SyncPtr(&GUID_DEVICE_BATTERY),
        SyncPtr(&BATTERY_STATUS_WMI_GUID),
        SyncPtr(&BATTERY_RUNTIME_WMI_GUID),
        SyncPtr(&BATTERY_TEMPERATURE_WMI_GUID),
        SyncPtr(&BATTERY_FULL_CHARGED_CAPACITY_WMI_GUID),
        SyncPtr(&BATTERY_CYCLE_COUNT_WMI_GUID),
        SyncPtr(&BATTERY_STATIC_DATA_WMI_GUID),
        SyncPtr(&BATTERY_STATUS_CHANGE_WMI_GUID),
        SyncPtr(&BATTERY_TAG_CHANGE_WMI_GUID),
    ];
}
