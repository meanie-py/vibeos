//! SMBIOS/DMI table parsing — Rust port of `sdk/lib/dmilib/dmilib.c`
//! (ReactOS DMI library, Timo Kreuzer). Inlined into i8042prt-rust so the
//! cargo-driven i8042prt.sys link no longer reuses cmake's `libdmilib.a`
//! (`_ParseSMBiosTables` was the sole symbol that pulled it in). The host tool
//! `udmihelp` keeps the C copy; this is the kernel-side translation.
//!
//! The walk is a faithful, byte-exact mirror of the C: it strides the WMI
//! `MSSmBios_RawSMBiosTables` blob (a 4-byte version header + ULONG `Size` +
//! the raw SMBIOS data), reading each `DMI_HEADER` and harvesting the indexed
//! strings into the caller's `Strings` slot table.

use ntkrnl_sys::base::{PCHAR, PVOID, ULONG};

// ---- dmilib.h _ID_STRINGS slot indices --------------------------------------
// The positions ParseSMBiosTables fills in the caller's string table. The
// i8042prt hwhacks matcher reads the SAME slots (see hwhacks::SYS_VENDOR /
// SYS_PRODUCT) — they must agree, or quirk matching silently reads garbage.
const BIOS_VENDOR: usize = 1;
const BIOS_VERSION: usize = 2;
const BIOS_DATE: usize = 3;
const SYS_VENDOR: usize = 4;
const SYS_PRODUCT: usize = 5;
const SYS_VERSION: usize = 6;
const SYS_SERIAL: usize = 7;
const SYS_SKU: usize = 8;
const SYS_FAMILY: usize = 9;
const BOARD_VENDOR: usize = 10;
const BOARD_NAME: usize = 11;
const BOARD_VERSION: usize = 12;
const BOARD_SERIAL: usize = 13;
const BOARD_ASSET_TAG: usize = 14;

// ---- dmi.h DMI_ENTRY_TYPE (only the types we harvest + the terminator) ------
const DMI_ENTRY_BIOS: u8 = 0;
const DMI_ENTRY_SYSTEM: u8 = 1;
const DMI_ENTRY_BASEBOARD: u8 = 2;
const DMI_ENTRY_END_OF_TABLE: u8 = 127;

// ---- dmi.h _DMI_FIELD_OFFSETS (byte offset of each string-index field) ------
const DMI_BIOS_VENDOR: usize = 0x04;
const DMI_BIOS_VERSION: usize = 0x05;
const DMI_BIOS_DATE: usize = 0x08;
const DMI_BIOS_SIZE: u32 = 0x14;

const DMI_SYS_VENDOR: usize = 0x04;
const DMI_SYS_PRODUCT: usize = 0x05;
const DMI_SYS_VERSION: usize = 0x06;
const DMI_SYS_SERIAL: usize = 0x07;
const DMI_SYS_SKU: usize = 0x19;
const DMI_SYS_FAMILY: usize = 0x1a;
const DMI_SYS_SIZE: u32 = 0x1b;

const DMI_BOARD_VENDOR: usize = 0x04;
const DMI_BOARD_NAME: usize = 0x05;
const DMI_BOARD_VERSION: usize = 0x06;
const DMI_BOARD_SERIAL: usize = 0x07;
const DMI_BOARD_ASSET_TAG: usize = 0x08;
const DMI_BOARD_SIZE: u32 = 0x10;

/// `DMI_HEADER` is `{ UCHAR Type; UCHAR Length; USHORT Handle; }` — `Type` at
/// byte 0, `Length` at byte 1. `sizeof(DMI_HEADER) == 4`.
const DMI_HEADER_SIZE: u32 = 4;

/// Port of `GetDmiString`: the byte at `field_offset` is a 1-based string index
/// into the string set that follows the fixed part of this structure (which
/// starts at `header + Length`). Index 0 means "no string".
unsafe fn get_dmi_string(header: *const u8, field_offset: usize) -> PCHAR {
    let mut string_index = *header.add(field_offset);
    if string_index == 0 {
        return core::ptr::null_mut();
    }

    // String set begins right after the formatted area (header + Length).
    let mut s = header.add(*header.add(1) as usize);

    // Walk forward (index - 1) NUL-terminated strings to reach the target.
    loop {
        string_index -= 1;
        if string_index == 0 {
            break;
        }
        while *s != 0 {
            s = s.add(1);
        }
        s = s.add(1);
    }

    s as PCHAR
}

/// Port of `ParseSMBiosTables`. `smbios_tables` points at a WMI
/// `MSSmBios_RawSMBiosTables` blob: `{ UCHAR Used20CallingMethod; UCHAR Major;
/// UCHAR Minor; UCHAR DmiRevision; ULONG Size; UCHAR SMBiosData[]; }` — `Size`
/// at byte 4, `SMBiosData` at byte 8. `strings` must point at a table of at
/// least `ID_STRINGS_MAX` `PCHAR` slots. `table_size` is unused by the C and
/// kept only for signature parity.
pub unsafe fn parse_smbios_tables(smbios_tables: PVOID, _table_size: ULONG, strings: *mut PCHAR) {
    let base = smbios_tables as *const u8;
    let mut header = base.add(8); // &SMBiosData
    let mut remaining = *(base.add(4) as *const u32); // Size

    while remaining >= DMI_HEADER_SIZE {
        let entry_type = *header; // DMI_HEADER.Type
        if entry_type == DMI_ENTRY_END_OF_TABLE {
            break;
        }

        if entry_type == DMI_ENTRY_BIOS {
            if remaining < DMI_BIOS_SIZE {
                return;
            }
            *strings.add(BIOS_VENDOR) = get_dmi_string(header, DMI_BIOS_VENDOR);
            *strings.add(BIOS_VERSION) = get_dmi_string(header, DMI_BIOS_VERSION);
            *strings.add(BIOS_DATE) = get_dmi_string(header, DMI_BIOS_DATE);
        } else if entry_type == DMI_ENTRY_SYSTEM {
            if remaining < DMI_SYS_SIZE {
                return;
            }
            *strings.add(SYS_VENDOR) = get_dmi_string(header, DMI_SYS_VENDOR);
            *strings.add(SYS_PRODUCT) = get_dmi_string(header, DMI_SYS_PRODUCT);
            *strings.add(SYS_VERSION) = get_dmi_string(header, DMI_SYS_VERSION);
            *strings.add(SYS_SERIAL) = get_dmi_string(header, DMI_SYS_SERIAL);
            *strings.add(SYS_SKU) = get_dmi_string(header, DMI_SYS_SKU);
            *strings.add(SYS_FAMILY) = get_dmi_string(header, DMI_SYS_FAMILY);
        } else if entry_type == DMI_ENTRY_BASEBOARD {
            if remaining < DMI_BOARD_SIZE {
                return;
            }
            *strings.add(BOARD_VENDOR) = get_dmi_string(header, DMI_BOARD_VENDOR);
            *strings.add(BOARD_NAME) = get_dmi_string(header, DMI_BOARD_NAME);
            *strings.add(BOARD_VERSION) = get_dmi_string(header, DMI_BOARD_VERSION);
            *strings.add(BOARD_SERIAL) = get_dmi_string(header, DMI_BOARD_SERIAL);
            *strings.add(BOARD_ASSET_TAG) = get_dmi_string(header, DMI_BOARD_ASSET_TAG);
        }
        // DMI_ENTRY_CHASSIS / _ONBOARD_DEVICE / _OEMSTRINGS and everything else
        // are skipped (the C has empty cases) — the string-set walk below still
        // advances past them.

        // Advance past the formatted area (matches C: ULONG wraps on malformed
        // tables, same as the source — the size guards above protect the reads).
        let length = *header.add(1) as u32; // DMI_HEADER.Length
        remaining = remaining.wrapping_sub(length);
        let mut data = header.add(length as usize);

        // Skip the trailing string set: scan to a double-NUL.
        while remaining >= 2 && (*data != 0 || *data.add(1) != 0) {
            data = data.add(1);
            remaining -= 1;
        }

        if remaining < 2 {
            break;
        }
        remaining -= 2;
        header = data.add(2); // next DMI_HEADER
    }
}
