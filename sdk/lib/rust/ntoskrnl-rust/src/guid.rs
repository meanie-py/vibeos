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
// poclass.h / batclass.h — power-management + battery class GUIDs.
//
// REMOVED (crate-split dup dedup, 2026-07-06): the 17 poclass/batclass GUIDs
// (GUID_CLASS_INPUT, GUID_DEVICE_*, GUID_DEVICE_BATTERY, BATTERY_*_WMI_GUID)
// used to be defined here AND — byte-identically — in guids-rust/src/defs.rs
// (the auto-generated wdmguid/poguid/ioevent replacement, which is in the
// kernel link and marks each `#[no_mangle] #[used]`). Two definitions of the
// same symbol were only tolerated by `--allow-multiple-definition`. guids-rust
// is the single canonical owner now; consumers still reference the symbol via
// `extern { static FOO: GUID; }` and resolve to it. Only the diskguid.h
// PARTITION_* GUIDs (NOT provided by guids-rust) remain here.
// ---------------------------------------------------------------------------

// Force the linker to pull this CGU out of the staticlib archive even though
// the only references come from C-side `extern GUID` declarations. Without
// this, the GUID-defining CGU may not get pulled.
mod _force_guid_symbols {
    use super::*;
    #[repr(transparent)]
    struct SyncPtr(*const GUID);
    unsafe impl Sync for SyncPtr {}

    #[used]
    static REFS: [SyncPtr; 6] = [
        SyncPtr(&PARTITION_ENTRY_UNUSED_GUID),
        SyncPtr(&PARTITION_SYSTEM_GUID),
        SyncPtr(&PARTITION_MSFT_RESERVED_GUID),
        SyncPtr(&PARTITION_BASIC_DATA_GUID),
        SyncPtr(&PARTITION_LDM_METADATA_GUID),
        SyncPtr(&PARTITION_LDM_DATA_GUID),
    ];
}
