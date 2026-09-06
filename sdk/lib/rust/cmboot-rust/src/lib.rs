//! cmboot-rust — Configuration Manager boot initialization, shared by the
//! kernel (kernel-rust) and the FreeLDR loaders (freeldr-rust uefi/rosload).
//! Translation of `ntoskrnl/config/cmboot.c`. See Cargo.toml for the
//! shared-object rationale.
#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_imports, unused_unsafe, dead_code, static_mut_refs)]

use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{BOOT_DRIVER_NODE, CmpCompressedNameSize, CmpCopyCompressedName, CmpFindSubKeyByName, CmpFindSubKeyByNumber, CmpFindValueByName, CmpGetValueData, CmpValueToData, HCELL_INDEX, PBOOT_DRIVER_LIST_ENTRY, PBOOT_DRIVER_NODE, PCM_KEY_NODE, PCM_KEY_VALUE, PCUNICODE_STRING, PHHIVE, PUNICODE_STRING, RtlAppendUnicodeStringToString, RtlAppendUnicodeToString, RtlEqualUnicodeString, RtlInitEmptyUnicodeString, RtlInitUnicodeString, _BOOT_DRIVER_LIST_ENTRY, _LIST_ENTRY, _UNICODE_STRING, SERVICE_LOAD_TYPE};
use core::ffi::c_void;

// HACK: non-NT-compatible SafeBoot support, exactly as in cmboot.c. The C reads
// it via `extern ULONG InitSafeBootMode;` regardless of context; the symbol is
// defined by the consumer — ex/init.rs (`#[no_mangle]`) on the kernel link, and
// freeldr-rust's `cmboot_shims` (the loader-local `ULONG InitSafeBootMode = 0;`
// that used to live in cmboot.c itself) on the loader links.
extern "C" {
    static mut InitSafeBootMode: ULONG;
}

use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}

// Local constants — values match cmtypes.h / ntoskrnl C definitions.
const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
const SERVICE_ERROR_CRITICAL: u32 = 0x00000003;

// RTL_CONSTANT_STRING — C macro constructs a UNICODE_STRING from a wide
// literal whose length is known at compile time. Here we accept a NUL-terminated
// wide-pointer (the LLM translated the literal as `b"...\\0\\0".as_ptr()`) and
// derive Length / MaximumLength via wcslen.
#[inline]
unsafe fn RTL_CONSTANT_STRING(buf: *mut u16) -> _UNICODE_STRING {
    let chars = wcslen(buf);
    let len = (chars * 2) as u16;
    _UNICODE_STRING { Length: len, MaximumLength: len + 2, Buffer: buf }
}

// Self-contained `RtlStringCbPrintfW(buf, sizeof(buf), L"ControlSet%03lu", id)`.
// cmboot.c's only formatting need. The kernel translation routed this through a
// ntoskrnl-rust-internal printf helper (rust_rtl_string_cb_printf_w), which is
// unreachable from a shared leaf crate without a circular dep — so we format the
// single fixed pattern directly. Writes "ControlSet" + zero-padded (>=3-digit)
// decimal of `id`, NUL-terminated, into `buf`. Returns STATUS_SUCCESS, or
// STATUS_BUFFER_OVERFLOW if `buf` can't hold it (matches the macro contract).
unsafe fn format_control_set(buf: &mut [u16], id: u32) -> i32 {
    const PREFIX: &[u16] = &[
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16,
        b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
    ];
    // %03lu: at least 3 digits, zero-padded; full value if longer.
    let mut digits = [0u8; 10]; // u32 max = 10 digits
    let mut n = id;
    let mut ndigits = 0usize;
    loop {
        digits[ndigits] = (n % 10) as u8;
        ndigits += 1;
        n /= 10;
        if n == 0 { break; }
    }
    let width = if ndigits < 3 { 3 } else { ndigits };
    let total = PREFIX.len() + width;
    if buf.len() < total + 1 {
        return STATUS_BUFFER_OVERFLOW;
    }
    let mut i = 0usize;
    for &c in PREFIX { buf[i] = c; i += 1; }
    for _ in ndigits..width { buf[i] = b'0' as u16; i += 1; } // leading zeros
    for k in (0..ndigits).rev() { buf[i] = (b'0' + digits[k]) as u16; i += 1; }
    buf[i] = 0;
    STATUS_SUCCESS
}

extern "C" {
    fn CMTRACE(level: u32, fmt: *const u8, ...);
    fn IS_NULL_TERMINATED(Buffer: *const u16, Length: u32) -> u8;
    fn wcsrchr(s: *const u16, c: u16) -> *mut u16;
    fn wcslen(s: *const u16) -> usize;
}

// HvGetCell(Hive, Cell) is the `cmlib.h` macro `(Hive)->GetCellRoutine(Hive, Cell)`.
// It is a MACRO, not a real symbol: the kernel exposes one via config/inlines.rs,
// but the bootloader's blcmlib only provides HvGetCellSize (the C loader used the
// macro inline). Inlining it here — identical to inlines.rs — means cmboot-rust
// needs NO external HvGetCell on either the kernel OR the loader links, the only
// way to satisfy both contexts from one shared crate.
#[inline]
unsafe fn HvGetCell(hive: PHHIVE, cell: HCELL_INDEX) -> *mut core::ffi::c_void {
    if let Some(get) = (*hive).GetCellRoutine {
        get(hive, cell) as *mut core::ffi::c_void
    } else {
        core::ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindControlSet(
    SystemHive: PHHIVE,
    RootCell: HCELL_INDEX,
    SelectKeyName: PCUNICODE_STRING,
    AutoSelect: PBOOLEAN
) -> HCELL_INDEX {
    let SystemHive = opaque_ptr(SystemHive);
    let SelectKeyName = opaque_ptr_const(SelectKeyName);
    let AutoSelect = opaque_ptr(AutoSelect);
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Node: PCM_KEY_NODE;
    let SelectCell: HCELL_INDEX;
    let AutoSelectCell: HCELL_INDEX;
    let SelectValueCell: HCELL_INDEX;
    let ControlSetCell: HCELL_INDEX;
    let CurrentValueCell: HCELL_INDEX;
    let mut Value: PCM_KEY_VALUE;
    let mut Length: u32 = 0;
    let Status: i32;
    let mut CurrentData: *mut u32;
    let ControlSetId: *mut u32;
    let mut Buffer: [u16; 128] = [0; 128];

    RtlInitUnicodeString(&mut Name, w!("select").as_ptr() as *const u16);
    Node = HvGetCell(SystemHive, RootCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xffffffff;
    }
    SelectCell = CmpFindSubKeyByName(SystemHive, Node, &Name);
    if SelectCell == 0xffffffff {
        return 0xffffffff;
    }

    RtlInitUnicodeString(&mut Name, w!("AutoSelect").as_ptr() as *const u16);
    Node = HvGetCell(SystemHive, SelectCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xffffffff;
    }
    AutoSelectCell = CmpFindValueByName(SystemHive, Node, &Name);
    if AutoSelectCell == 0xffffffff {
        *AutoSelect = 1u8;
    } else {
        Value = HvGetCell(SystemHive, AutoSelectCell) as PCM_KEY_VALUE;
        if Value.is_null() {
            return 0xffffffff;
        }

        CurrentData = CmpValueToData(SystemHive, Value, &mut Length) as *mut u32;
        if CurrentData.is_null() {
            return 0xffffffff;
        }

        *AutoSelect = *(CurrentData as *mut u8);
    }

    Node = HvGetCell(SystemHive, SelectCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xffffffff;
    }
    SelectValueCell = CmpFindValueByName(SystemHive, Node, SelectKeyName);
    if SelectValueCell == 0xffffffff {
        return 0xffffffff;
    }

    Value = HvGetCell(SystemHive, SelectValueCell) as PCM_KEY_VALUE;
    if Value.is_null() {
        return 0xffffffff;
    }
    if (*Value).Type != 4 {
        return 0xffffffff;
    }
    ControlSetId = CmpValueToData(SystemHive, Value, &mut Length) as *mut u32;
    if ControlSetId.is_null() {
        return 0xffffffff;
    }
    if Length < 4 {
        return 0xffffffff;
    }

    Status = format_control_set(&mut Buffer, *ControlSetId);
    if !NT_SUCCESS(Status) {
        return 0xffffffff;
    }
    RtlInitUnicodeString(&mut Name, Buffer.as_ptr());

    Node = HvGetCell(SystemHive, RootCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xffffffff;
    }
    ControlSetCell = CmpFindSubKeyByName(SystemHive, Node, &Name);
    if ControlSetCell == 0xffffffff {
        return 0xffffffff;
    }

    RtlInitUnicodeString(&mut Name, w!("Current").as_ptr() as *const u16);
    Node = HvGetCell(SystemHive, SelectCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xffffffff;
    }
    CurrentValueCell = CmpFindValueByName(SystemHive, Node, &Name);

    if CurrentValueCell != 0xffffffff {
        Value = HvGetCell(SystemHive, CurrentValueCell) as PCM_KEY_VALUE;
        if Value.is_null() {
            return 0xffffffff;
        }
        if (*Value).Type == 4 {
            CurrentData = CmpValueToData(SystemHive, Value, &mut Length) as *mut u32;
            if CurrentData.is_null() {
                return 0xffffffff;
            }
            if Length < 4 {
                return 0xffffffff;
            }

            *CurrentData = *ControlSetId;
        }
    }

    ControlSetCell
}

unsafe fn CmpFindTagIndex(
    Hive: PHHIVE,
    TagCell: HCELL_INDEX,
    GroupOrderCell: HCELL_INDEX,
    GroupName: PCUNICODE_STRING,
) -> u32 {
    let TagValue: PCM_KEY_VALUE;
    let Value: PCM_KEY_VALUE;
    let Node: PCM_KEY_NODE;
    let mut OrderCell: HCELL_INDEX;
    let DriverTag: *mut u32;
    let mut TagOrder: *mut u32 = core::ptr::null_mut();
    let mut CurrentTag: u32;
    let mut Length: u32 = 0;
    let mut BufferAllocated: u8 = 0;

    Value = HvGetCell(Hive, TagCell) as PCM_KEY_VALUE;
    if Value.is_null() {
        return 0xFFFFFFFE;
    }
    if (*Value).Type != 4 {
        return 0xFFFFFFFE;
    }
    DriverTag = CmpValueToData(Hive, Value, core::ptr::addr_of_mut!(Length)) as *mut u32;
    if DriverTag.is_null() {
        return 0xFFFFFFFE;
    }
    if Length < 4 {
        return 0xFFFFFFFE;
    }

    Node = HvGetCell(Hive, GroupOrderCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0xFFFFFFFE;
    }
    OrderCell = CmpFindValueByName(Hive, Node, GroupName);
    if OrderCell == 0xFFFFFFFF {
        return 0xFFFFFFFE;
    }

    TagValue = HvGetCell(Hive, OrderCell) as PCM_KEY_VALUE;
    if TagValue.is_null() {
        return 0xFFFFFFFE;
    }
    if CmpGetValueData(
        Hive,
        TagValue,
        core::ptr::addr_of_mut!(Length),
        core::ptr::addr_of_mut!(TagOrder) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(BufferAllocated),
        core::ptr::addr_of_mut!(OrderCell),
    ) == 0
        || TagOrder.is_null()
    {
        return 0xFFFFFFFE;
    }

    CurrentTag = 1;
    while CurrentTag <= *TagOrder.offset(0) {
        if *TagOrder.offset(CurrentTag as isize) == *DriverTag {
            if BufferAllocated != 0 {
                ((*Hive).Free.unwrap())(TagOrder as *mut core::ffi::c_void, Length);
            }
            return CurrentTag;
        }
        CurrentTag += 1;
    }

    if BufferAllocated != 0 {
        ((*Hive).Free.unwrap())(TagOrder as *mut core::ffi::c_void, Length);
    }
    return 0xFFFFFFFE;
}

// Exported (NTAPI / stdcall on i386 → `_CmpIsDriverInList@12`): the C cmboot.c
// exposed this as a real symbol that the bootloader's wlregistry.c calls (now
// freeldr-rust/src/wlregistry.rs externs it). The kernel doesn't reference it
// externally, but exporting is harmless there. Was a private `fn` — the loader
// link only "resolved" it via a stale cmboot.c.obj archive member.
#[no_mangle]
pub unsafe extern "system" fn CmpIsDriverInList(
    DriverListHead: PLIST_ENTRY,
    DriverName: PCUNICODE_STRING,
    FoundDriver: *mut PBOOT_DRIVER_NODE,
) -> u8 {
    let mut Entry: PLIST_ENTRY;
    let mut DriverNode: PBOOT_DRIVER_NODE;

    Entry = (*DriverListHead).Flink;
    while Entry != DriverListHead {
        DriverNode = (Entry as *mut u8).offset(
            -(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize)
            -(core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)
        ) as PBOOT_DRIVER_NODE;

        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!((*DriverNode).Name),
            DriverName,
            1u8,
        ) != 0
        {
            if !FoundDriver.is_null() {
                *FoundDriver = DriverNode;
            }
            return 1u8;
        }

        Entry = (*Entry).Flink;
    }

    if !FoundDriver.is_null() {
        *FoundDriver = core::ptr::null_mut();
    }
    return 0u8;
}

unsafe fn CmpAddDriverToList(
    Hive: PHHIVE,
    DriverCell: HCELL_INDEX,
    GroupOrderCell: HCELL_INDEX,
    RegistryPath: PCUNICODE_STRING,
    DriverListHead: PLIST_ENTRY,
) -> u8 {
    let DriverNode: PBOOT_DRIVER_NODE;
    let DriverEntry: PBOOT_DRIVER_LIST_ENTRY;
    let Node: PCM_KEY_NODE;
    let mut Value: PCM_KEY_VALUE;
    let mut Length: u32 = 0;
    let NameLength: u16;
    let mut ValueCell: HCELL_INDEX;
    let TagCell: HCELL_INDEX;
    let FilePath: PUNICODE_STRING;
    let RegistryString: PUNICODE_STRING;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let ErrorControl: *mut u32;
    let mut Buffer: *mut u16;

    const CM_BOOT_DEBUG: u32 = 0;
    // C: cmlib.h `#define TAG_CM '  MC'`. A multi-char literal is BIG-endian. cmboot.c:387
    const TAG_CM: u32 = u32::from_be_bytes(*b"  MC");
    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;
    const REG_DWORD: u32 = 4;
    const KEY_COMP_NAME: u16 = 0x20;
    const NormalError: u32 = 1;

    DriverNode = ((*Hive).Allocate.unwrap())(core::mem::size_of::<BOOT_DRIVER_NODE>() as _, 0u8, TAG_CM) as PBOOT_DRIVER_NODE;
    if DriverNode.is_null() {
        return 0u8;
    }

    core::ptr::write_bytes(DriverNode, 0, 1);
    DriverEntry = core::ptr::addr_of_mut!((*DriverNode).ListEntry);

    Node = HvGetCell(Hive, DriverCell) as PCM_KEY_NODE;
    if Node.is_null() {
        'Failure: {
            if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).RegistryPath.MaximumLength as u32,
                );
            }
            if !(*DriverEntry).FilePath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).FilePath.MaximumLength as u32,
                );
            }
            if !(*DriverNode).Name.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                    (*DriverNode).Name.MaximumLength as u32,
                );
            }
            ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
            return 0u8;
        }
    }

    NameLength = if (*Node).Flags & KEY_COMP_NAME != 0 {
        CmpCompressedNameSize(core::ptr::addr_of_mut!((*Node).Name) as *mut u16, (*Node).NameLength as u32)
    } else {
        (*Node).NameLength
    };
    if (NameLength as usize) < core::mem::size_of::<u16>() {
        'Failure: {
            if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).RegistryPath.MaximumLength as u32,
                );
            }
            if !(*DriverEntry).FilePath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).FilePath.MaximumLength as u32,
                );
            }
            if !(*DriverNode).Name.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                    (*DriverNode).Name.MaximumLength as u32,
                );
            }
            ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
            return 0u8;
        }
    }

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!((*DriverNode).Name),
        ((*Hive).Allocate.unwrap())(NameLength as _, 0u8, TAG_CM) as *mut u16,
        NameLength,
    );
    if (*DriverNode).Name.Buffer.is_null() {
        'Failure: {
            if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).RegistryPath.MaximumLength as u32,
                );
            }
            if !(*DriverEntry).FilePath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).FilePath.MaximumLength as u32,
                );
            }
            if !(*DriverNode).Name.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                    (*DriverNode).Name.MaximumLength as u32,
                );
            }
            ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
            return 0u8;
        }
    }

    (*DriverNode).Name.Length = NameLength;
    if (*Node).Flags & KEY_COMP_NAME != 0 {
        CmpCopyCompressedName(
            (*DriverNode).Name.Buffer,
            (*DriverNode).Name.Length as u32,
            core::ptr::addr_of_mut!((*Node).Name) as *mut u16,
            (*Node).NameLength as u32,
        );
    } else {
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*Node).Name) as *const u8,
            (*DriverNode).Name.Buffer as *mut u8,
            (*Node).NameLength as usize,
        );
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("ImagePath").as_ptr() as *const u16);
    ValueCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of!(Name));
    if ValueCell == HCELL_NIL {
        Length = (core::mem::size_of::<u16>() * 18) as u32 + NameLength as u32 + (core::mem::size_of::<u16>() * 4) as u32;

        FilePath = core::ptr::addr_of_mut!((*DriverEntry).FilePath);
        RtlInitEmptyUnicodeString(
            FilePath,
            ((*Hive).Allocate.unwrap())(Length as _, 0u8, TAG_CM) as *mut u16,
            Length as u16,
        );
        if (*FilePath).Buffer.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }

        if !NT_SUCCESS(RtlAppendUnicodeToString(FilePath, w!("System32\\Drivers\\").as_ptr())) ||
           !NT_SUCCESS(RtlAppendUnicodeStringToString(FilePath, core::ptr::addr_of!((*DriverNode).Name))) ||
           !NT_SUCCESS(RtlAppendUnicodeToString(FilePath, w!(".sys").as_ptr() as *const u16))
        {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
    } else {
        Value = HvGetCell(Hive, ValueCell) as PCM_KEY_VALUE;
        if Value.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if (*Value).Type != REG_SZ && (*Value).Type != REG_EXPAND_SZ {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        Buffer = CmpValueToData(Hive, Value, core::ptr::addr_of_mut!(Length)) as *mut u16;
        if Buffer.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if IS_NULL_TERMINATED(Buffer, Length) != 0 {
            Length -= core::mem::size_of::<u16>() as u32;
        }
        if (Length as usize) < core::mem::size_of::<u16>() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }

        FilePath = core::ptr::addr_of_mut!((*DriverEntry).FilePath);
        RtlInitEmptyUnicodeString(
            FilePath,
            ((*Hive).Allocate.unwrap())(Length as _, 0u8, TAG_CM) as *mut u16,
            Length as u16,
        );
        if (*FilePath).Buffer.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }

        core::ptr::copy_nonoverlapping(Buffer as *const u8, (*FilePath).Buffer as *mut u8, Length as usize);
        (*FilePath).Length = Length as u16;
    }

    RegistryString = core::ptr::addr_of_mut!((*DriverEntry).RegistryPath);
    Length = (*RegistryPath).Length as u32 + NameLength as u32;
    RtlInitEmptyUnicodeString(
        RegistryString,
        ((*Hive).Allocate.unwrap())(Length as _, 0u8, TAG_CM) as *mut u16,
        Length as u16,
    );
    if (*RegistryString).Buffer.is_null() {
        'Failure: {
            if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).RegistryPath.MaximumLength as u32,
                );
            }
            if !(*DriverEntry).FilePath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).FilePath.MaximumLength as u32,
                );
            }
            if !(*DriverNode).Name.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                    (*DriverNode).Name.MaximumLength as u32,
                );
            }
            ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
            return 0u8;
        }
    }

    if !NT_SUCCESS(RtlAppendUnicodeStringToString(RegistryString, RegistryPath)) ||
       !NT_SUCCESS(RtlAppendUnicodeStringToString(RegistryString, core::ptr::addr_of!((*DriverNode).Name)))
    {
        'Failure: {
            if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).RegistryPath.MaximumLength as u32,
                );
            }
            if !(*DriverEntry).FilePath.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                    (*DriverEntry).FilePath.MaximumLength as u32,
                );
            }
            if !(*DriverNode).Name.Buffer.is_null() {
                ((*Hive).Free.unwrap())(
                    (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                    (*DriverNode).Name.MaximumLength as u32,
                );
            }
            ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
            return 0u8;
        }
    }

    InsertHeadList(DriverListHead, core::ptr::addr_of_mut!((*DriverEntry).Link));

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("ErrorControl").as_ptr() as *const u16);
    ValueCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of!(Name));
    if ValueCell == HCELL_NIL {
        (*DriverNode).ErrorControl = NormalError;
    } else {
        Value = HvGetCell(Hive, ValueCell) as PCM_KEY_VALUE;
        if Value.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if (*Value).Type != REG_DWORD {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        ErrorControl = CmpValueToData(Hive, Value, core::ptr::addr_of_mut!(Length)) as *mut u32;
        if ErrorControl.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if (Length as usize) < core::mem::size_of::<u32>() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }

        (*DriverNode).ErrorControl = *ErrorControl;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("group").as_ptr() as *const u16);
    ValueCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of!(Name));
    if ValueCell == HCELL_NIL {
        RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!((*DriverNode).Group), core::ptr::null_mut(), 0);
    } else {
        Value = HvGetCell(Hive, ValueCell) as PCM_KEY_VALUE;
        if Value.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if (*Value).Type != REG_SZ {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }

        Buffer = CmpValueToData(Hive, Value, core::ptr::addr_of_mut!(Length)) as *mut u16;
        if Buffer.is_null() {
            'Failure: {
                if !(*DriverEntry).RegistryPath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).RegistryPath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).RegistryPath.MaximumLength as u32,
                    );
                }
                if !(*DriverEntry).FilePath.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverEntry).FilePath.Buffer as *mut core::ffi::c_void,
                        (*DriverEntry).FilePath.MaximumLength as u32,
                    );
                }
                if !(*DriverNode).Name.Buffer.is_null() {
                    ((*Hive).Free.unwrap())(
                        (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                        (*DriverNode).Name.MaximumLength as u32,
                    );
                }
                ((*Hive).Free.unwrap())(DriverNode as *mut core::ffi::c_void, core::mem::size_of::<BOOT_DRIVER_NODE>() as _);
                return 0u8;
            }
        }
        if IS_NULL_TERMINATED(Buffer, Length) != 0 {
            Length -= core::mem::size_of::<u16>() as u32;
        }

        (*DriverNode).Group.Buffer = Buffer;
        (*DriverNode).Group.Length = Length as u16;
        (*DriverNode).Group.MaximumLength = (*DriverNode).Group.Length;
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("Tag").as_ptr() as *const u16);
    TagCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of!(Name));
    if TagCell == HCELL_NIL {
        (*DriverNode).Tag = 0xFFFFFFFFu32;
    } else {
        (*DriverNode).Tag = CmpFindTagIndex(
            Hive,
            TagCell,
            GroupOrderCell,
            core::ptr::addr_of!((*DriverNode).Group),
        );
    }

    CMTRACE(
        CM_BOOT_DEBUG,
        b"Adding boot driver: '%wZ', '%wZ'\n\0".as_ptr(),
        core::ptr::addr_of!((*DriverNode).Name),
        core::ptr::addr_of!((*DriverEntry).FilePath),
    );

    1u8
}

unsafe fn CmpIsLoadType(
    Hive: PHHIVE,
    Cell: HCELL_INDEX,
    LoadType: SERVICE_LOAD_TYPE,
) -> BOOLEAN {
    let Node: PCM_KEY_NODE;
    let Value: PCM_KEY_VALUE;
    let mut Name: _UNICODE_STRING = RTL_CONSTANT_STRING(w!("Start").as_ptr() as *mut u16);
    let ValueCell: HCELL_INDEX;
    let mut Length: ULONG = 0;
    let Data: PULONG;

    Node = HvGetCell(Hive, Cell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0u8;
    }
    ValueCell = CmpFindValueByName(Hive, Node, &mut Name);
    if ValueCell == HCELL_NIL {
        return 0u8;
    }

    Value = HvGetCell(Hive, ValueCell) as PCM_KEY_VALUE;
    if Value.is_null() {
        return 0u8;
    }
    if (*Value).Type != REG_DWORD {
        return 0u8;
    }
    Data = CmpValueToData(Hive, Value, &mut Length) as PULONG;
    if Data.is_null() {
        return 0u8;
    }
    if Length < core::mem::size_of::<ULONG>() as u32 {
        return 0u8;
    }

    if *Data == LoadType {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn CmpFindDrivers(
    Hive: PHHIVE,
    ControlSet: HCELL_INDEX,
    LoadType: SERVICE_LOAD_TYPE,
    BootFileSystem: PCWSTR,
    DriverListHead: PLIST_ENTRY,
) -> u8 {
    let Hive = opaque_ptr(Hive);
    let BootFileSystem = opaque_ptr_const(BootFileSystem);
    let DriverListHead = opaque_ptr(DriverListHead);
    let ServicesCell: HCELL_INDEX;
    let ControlCell: HCELL_INDEX;
    let GroupOrderCell: HCELL_INDEX;
    let mut DriverCell: HCELL_INDEX;
    let mut SafeBootCell: HCELL_INDEX = HCELL_NIL;
    let mut i: u32;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut KeyPath: _UNICODE_STRING = core::mem::zeroed();
    let ControlNode: PCM_KEY_NODE;
    let ServicesNode: PCM_KEY_NODE;
    let mut Node: PCM_KEY_NODE;
    let FsNode: PBOOT_DRIVER_NODE;

    ControlNode = HvGetCell(Hive, ControlSet) as PCM_KEY_NODE;
    if ControlNode.is_null() {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, w!("Services").as_ptr() as *mut u16);
    ServicesCell = CmpFindSubKeyByName(Hive, ControlNode, &Name);
    if ServicesCell == HCELL_NIL {
        return 0u8;
    }

    ServicesNode = HvGetCell(Hive, ServicesCell) as PCM_KEY_NODE;
    if ServicesNode.is_null() {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, w!("Control").as_ptr() as *mut u16);
    ControlCell = CmpFindSubKeyByName(Hive, ControlNode, &Name);
    if ControlCell == HCELL_NIL {
        return 0u8;
    }

    Node = HvGetCell(Hive, ControlCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0u8;
    }
    RtlInitUnicodeString(&mut Name, w!("GroupOrderList").as_ptr() as *mut u16);
    GroupOrderCell = CmpFindSubKeyByName(Hive, Node, &Name);
    if GroupOrderCell == HCELL_NIL {
        return 0u8;
    }

    if InitSafeBootMode != 0 {
        RtlInitUnicodeString(&mut Name, w!("SafeBoot").as_ptr() as *mut u16);
        Node = HvGetCell(Hive, ControlCell) as PCM_KEY_NODE;
        if Node.is_null() {
            return 0u8;
        }
        SafeBootCell = CmpFindSubKeyByName(Hive, Node, &Name);
        if SafeBootCell == HCELL_NIL {
            return 0u8;
        }

        Node = HvGetCell(Hive, SafeBootCell) as PCM_KEY_NODE;
        if Node.is_null() {
            return 0u8;
        }
        match InitSafeBootMode {
            1 | 3 => RtlInitUnicodeString(&mut Name, w!("Minimal").as_ptr() as *mut u16),
            2 => RtlInitUnicodeString(&mut Name, w!("Network").as_ptr() as *mut u16),
            _ => return 0u8,
        }
        SafeBootCell = CmpFindSubKeyByName(Hive, Node, &Name);
        if SafeBootCell == HCELL_NIL {
            return 0u8;
        }
    }

    RtlInitUnicodeString(&mut KeyPath, w!("\\Registry\\Machine\\System\\CurrentControlSet\\Services\\").as_ptr() as *mut u16);

    i = 0;
    DriverCell = CmpFindSubKeyByNumber(Hive, ServicesNode, i);
    while DriverCell != HCELL_NIL {
        if CmpIsLoadType(Hive, DriverCell, LoadType) != 0 &&
           CmpIsSafe(Hive, SafeBootCell, DriverCell) != 0 {
            if CmpAddDriverToList(Hive,
                                  DriverCell,
                                  GroupOrderCell,
                                  &KeyPath,
                                  DriverListHead) == 0 {
            }
        }

        i = i.wrapping_add(1);
        DriverCell = CmpFindSubKeyByNumber(Hive, ServicesNode, i);
    }

    if !BootFileSystem.is_null() {
        RtlInitUnicodeString(&mut Name, BootFileSystem);
        DriverCell = CmpFindSubKeyByName(Hive, ServicesNode, &Name);
        if DriverCell != HCELL_NIL {
            if CmpAddDriverToList(Hive,
                                  DriverCell,
                                  GroupOrderCell,
                                  &KeyPath,
                                  DriverListHead) == 0 {
            } else {
                FsNode = ((*DriverListHead).Flink as *mut u8).offset(-(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize) - (core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)) as PBOOT_DRIVER_NODE;
                (*FsNode).ErrorControl = SERVICE_ERROR_CRITICAL;
            }
        }
    }

    return 1u8;
}

unsafe fn CmpDoSort(
    DriverListHead: PLIST_ENTRY,
    OrderList: PCUNICODE_STRING
) -> u8 {
    let mut Current: *mut u16;
    let mut End: *mut u16 = core::ptr::null_mut();
    let mut GroupName: _UNICODE_STRING = core::mem::zeroed();
    let mut NextEntry: PLIST_ENTRY;
    let mut CurrentNode: PBOOT_DRIVER_NODE;

    Current = (*OrderList).Buffer.add((*OrderList).Length as usize / core::mem::size_of::<u16>());
    while Current > (*OrderList).Buffer {
        loop {
            if *Current == 0u16 {
                End = Current;
            }
            Current = Current.sub(1);
            if Current == (*OrderList).Buffer || *Current.sub(1) == 0u16 {
                break;
            }
        }

        GroupName.Length = (End.offset_from(Current) as u16) * core::mem::size_of::<u16>() as u16;
        GroupName.MaximumLength = GroupName.Length;
        GroupName.Buffer = Current;

        NextEntry = (*DriverListHead).Flink;
        while NextEntry != DriverListHead {
            CurrentNode = (NextEntry as *const u8).sub(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) + core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_NODE;

            NextEntry = (*NextEntry).Flink;

            if !(*CurrentNode).Group.Buffer.is_null() &&
                RtlEqualUnicodeString(&GroupName, &(*CurrentNode).Group, 1u8) != 0 {
                RemoveEntryList(&mut (*CurrentNode).ListEntry.Link);
                InsertHeadList(DriverListHead, &mut (*CurrentNode).ListEntry.Link);
            }
        }

        Current = Current.sub(1);
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpSortDriverList(
    Hive: PHHIVE,
    ControlSet: HCELL_INDEX,
    DriverListHead: PLIST_ENTRY
) -> u8 {
    let Hive = opaque_ptr(Hive);
    let DriverListHead = opaque_ptr(DriverListHead);
    let mut Node: PCM_KEY_NODE;
    let ListValue: PCM_KEY_VALUE;
    let ControlCell: HCELL_INDEX;
    let GroupOrder: HCELL_INDEX;
    let ListCell: HCELL_INDEX;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut OrderList: _UNICODE_STRING = core::mem::zeroed();
    let mut Length: u32 = 0;

    Node = HvGetCell(Hive, ControlSet) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0u8;
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("Control").as_ptr() as *const u16);
    ControlCell = CmpFindSubKeyByName(Hive, Node, core::ptr::addr_of_mut!(Name));
    if ControlCell == 0xFFFFFFFF {
        return 0u8;
    }

    Node = HvGetCell(Hive, ControlCell) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0u8;
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("ServiceGroupOrder").as_ptr() as *const u16);
    GroupOrder = CmpFindSubKeyByName(Hive, Node, core::ptr::addr_of_mut!(Name));
    if GroupOrder == 0xFFFFFFFF {
        return 0u8;
    }

    Node = HvGetCell(Hive, GroupOrder) as PCM_KEY_NODE;
    if Node.is_null() {
        return 0u8;
    }
    RtlInitUnicodeString(core::ptr::addr_of_mut!(Name), w!("list").as_ptr() as *const u16);
    ListCell = CmpFindValueByName(Hive, Node, core::ptr::addr_of_mut!(Name));
    if ListCell == 0xFFFFFFFF {
        return 0u8;
    }

    ListValue = HvGetCell(Hive, ListCell) as PCM_KEY_VALUE;
    if ListValue.is_null() {
        return 0u8;
    }
    if (*ListValue).Type != 7 {
        return 0u8;
    }

    OrderList.Buffer = CmpValueToData(Hive, ListValue, core::ptr::addr_of_mut!(Length)) as *mut u16;
    if OrderList.Buffer.is_null() {
        return 0u8;
    }
    if IS_NULL_TERMINATED(OrderList.Buffer, Length) == 0 {
        return 0u8;
    }
    OrderList.Length = Length as u16 - 2;
    OrderList.MaximumLength = OrderList.Length;

    CmpDoSort(DriverListHead, core::ptr::addr_of_mut!(OrderList))
}

unsafe fn CmpOrderGroup(
    mut StartNode: PBOOT_DRIVER_NODE,
    mut EndNode: PBOOT_DRIVER_NODE,
) -> u8 {
    let mut CurrentNode: PBOOT_DRIVER_NODE;
    let mut PreviousNode: PBOOT_DRIVER_NODE;
    let mut ListEntry: PLIST_ENTRY;

    if StartNode == EndNode {
        return 1u8;
    }

    CurrentNode = StartNode;
    loop {
        PreviousNode = CurrentNode;

        ListEntry = (*CurrentNode).ListEntry.Link.Flink;
        CurrentNode = (ListEntry as *mut u8).offset(
            -(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize)
            - (core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)
        ) as PBOOT_DRIVER_NODE;

        if (*PreviousNode).Tag > (*CurrentNode).Tag {
            if CurrentNode == EndNode {
                ListEntry = (*CurrentNode).ListEntry.Link.Blink;
                EndNode = (ListEntry as *mut u8).offset(
                    -(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize)
                    - (core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)
                ) as PBOOT_DRIVER_NODE;
            }

            RemoveEntryList(core::ptr::addr_of_mut!((*CurrentNode).ListEntry.Link));

            while (*PreviousNode).Tag > (*CurrentNode).Tag && PreviousNode != StartNode {
                ListEntry = (*PreviousNode).ListEntry.Link.Blink;
                PreviousNode = (ListEntry as *mut u8).offset(
                    -(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) as isize)
                    - (core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link) as isize)
                ) as PBOOT_DRIVER_NODE;
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*PreviousNode).ListEntry.Link),
                core::ptr::addr_of_mut!((*CurrentNode).ListEntry.Link),
            );

            if PreviousNode == StartNode {
                StartNode = CurrentNode;
            }
        }

        if CurrentNode == EndNode {
            break;
        }
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn CmpResolveDriverDependencies(
    DriverListHead: PLIST_ENTRY,
) -> u8 {
    let DriverListHead = opaque_ptr(DriverListHead);
    let mut NextEntry: PLIST_ENTRY;
    let mut StartNode: PBOOT_DRIVER_NODE;
    let mut EndNode: PBOOT_DRIVER_NODE;
    let mut CurrentNode: PBOOT_DRIVER_NODE;

    NextEntry = (*DriverListHead).Flink;
    while NextEntry != DriverListHead {
        StartNode = (NextEntry as *mut u8).sub(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) + core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_NODE;
        loop {
            EndNode = (NextEntry as *mut u8).sub(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) + core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_NODE;

            NextEntry = (*NextEntry).Flink;
            CurrentNode = (NextEntry as *mut u8).sub(core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) + core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_NODE;

            if NextEntry == DriverListHead {
                break;
            }

            if RtlEqualUnicodeString(
                core::ptr::addr_of_mut!((*StartNode).Group),
                core::ptr::addr_of_mut!((*CurrentNode).Group),
                1u8,
            ) == 0
            {
                break;
            }
        }

        CmpOrderGroup(StartNode, EndNode);
    }

    1u8
}

unsafe fn CmpIsSafe(
    Hive: PHHIVE,
    SafeBootCell: HCELL_INDEX,
    DriverCell: HCELL_INDEX,
) -> u8 {
    let SafeBootNode: PCM_KEY_NODE;
    let DriverNode: PCM_KEY_NODE;
    let mut KeyValue: PCM_KEY_VALUE;
    let mut CellIndex: HCELL_INDEX;
    let mut Length: u32 = 0;
    let mut Name: _UNICODE_STRING = core::mem::zeroed();
    let mut Buffer: *mut u16;

    const HCELL_NIL: HCELL_INDEX = 0xFFFFFFFF;
    const REG_SZ: u32 = 1;
    const REG_EXPAND_SZ: u32 = 2;
    const KEY_COMP_NAME: u16 = 0x0020;
    const OBJ_NAME_PATH_SEPARATOR: u16 = '\\' as u16;
    // C: cmlib.h `#define TAG_CM '  MC'`. A multi-char literal is BIG-endian. cmboot.c:387
    const TAG_CM: u32 = u32::from_be_bytes(*b"  MC");

    DriverNode = HvGetCell(Hive, DriverCell) as PCM_KEY_NODE;
    if DriverNode.is_null() {
        return 0u8;
    }

    if SafeBootCell == HCELL_NIL {
        return 1u8;
    }
    SafeBootNode = HvGetCell(Hive, SafeBootCell) as PCM_KEY_NODE;
    if SafeBootNode.is_null() {
        return 0u8;
    }

    RtlInitUnicodeString(&mut Name, w!("Group").as_ptr() as *const u16);
    CellIndex = CmpFindValueByName(Hive, DriverNode, &Name);
    if CellIndex != HCELL_NIL {
        KeyValue = HvGetCell(Hive, CellIndex) as PCM_KEY_VALUE;
        if KeyValue.is_null() {
            return 0u8;
        }

        if (*KeyValue).Type == REG_SZ {
            Buffer = CmpValueToData(Hive, KeyValue, &mut Length) as *mut u16;
            if Buffer.is_null() {
                return 0u8;
            }
            if IS_NULL_TERMINATED(Buffer, Length) != 0 {
                Length -= core::mem::size_of::<u16>() as u32;
            }

            Name.Buffer = Buffer;
            Name.Length = Length as u16;
            Name.MaximumLength = Name.Length;

            CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
            if CellIndex != HCELL_NIL {
                return 1u8;
            }
        }
    }

    Length = if (*DriverNode).Flags & KEY_COMP_NAME != 0 {
        CmpCompressedNameSize(core::ptr::addr_of_mut!((*DriverNode).Name) as *mut u16, (*DriverNode).NameLength as u32) as u32
    } else {
        (*DriverNode).NameLength as u32
    };
    if Length < core::mem::size_of::<u16>() as u32 {
        return 0u8;
    }

    RtlInitEmptyUnicodeString(
        &mut Name,
        ((*Hive).Allocate.unwrap())(Length as _, 0u8, TAG_CM) as *mut u16,
        Length as u16,
    );
    if Name.Buffer.is_null() {
        return 0u8;
    }

    Name.Length = Length as u16;
    if (*DriverNode).Flags & KEY_COMP_NAME != 0 {
        CmpCopyCompressedName(
            Name.Buffer,
            Name.Length as u32,
            core::ptr::addr_of_mut!((*DriverNode).Name) as *mut u16,
            (*DriverNode).NameLength as u32,
        );
    } else {
        RtlCopyMemory(
            Name.Buffer as *mut core::ffi::c_void,
            core::ptr::addr_of!((*DriverNode).Name) as *const core::ffi::c_void,
            (*DriverNode).NameLength as usize,
        );
    }

    CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
    ((*Hive).Free.unwrap())(Name.Buffer as *mut core::ffi::c_void, Name.MaximumLength as u32);
    if CellIndex != HCELL_NIL {
        return 1u8;
    }

    RtlInitUnicodeString(&mut Name, w!("ImagePath").as_ptr() as *const u16);
    CellIndex = CmpFindValueByName(Hive, DriverNode, &Name);
    if CellIndex != HCELL_NIL {
        KeyValue = HvGetCell(Hive, CellIndex) as PCM_KEY_VALUE;
        if KeyValue.is_null() {
            return 0u8;
        }

        if ((*KeyValue).Type == REG_SZ) || ((*KeyValue).Type == REG_EXPAND_SZ) {
            Buffer = CmpValueToData(Hive, KeyValue, &mut Length) as *mut u16;
            if Buffer.is_null() {
                return 0u8;
            }
            if Length < core::mem::size_of::<u16>() as u32 {
                return 0u8;
            }

            Name.Buffer = wcsrchr(Buffer, OBJ_NAME_PATH_SEPARATOR);
            if Name.Buffer.is_null() {
                return 0u8;
            }
            Name.Buffer = Name.Buffer.add(1);

            if ((Name.Buffer as usize) - (Buffer as usize)) >= Length as usize {
                return 0u8;
            }
            Length -= ((Name.Buffer as usize) - (Buffer as usize)) as u32;
            if IS_NULL_TERMINATED(Name.Buffer, Length) != 0 {
                Length -= core::mem::size_of::<u16>() as u32;
            }
            if Length < core::mem::size_of::<u16>() as u32 {
                return 0u8;
            }

            Name.Length = Length as u16;
            Name.MaximumLength = Name.Length;

            CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
            if CellIndex != HCELL_NIL {
                return 1u8;
            }
        }
    }

    return 0u8;
}

#[no_mangle]
pub unsafe extern "system" fn CmpFreeDriverList(
    Hive: PHHIVE,
    DriverListHead: PLIST_ENTRY,
) {
    let Hive = opaque_ptr(Hive);
    let DriverListHead = opaque_ptr(DriverListHead);
    let mut Entry: PLIST_ENTRY;
    let mut DriverNode: PBOOT_DRIVER_NODE;

    while !IsListEmpty(DriverListHead) {
        Entry = RemoveHeadList(DriverListHead);
        DriverNode = ((Entry as usize) - core::mem::offset_of!(BOOT_DRIVER_NODE, ListEntry) - core::mem::offset_of!(_BOOT_DRIVER_LIST_ENTRY, Link)) as PBOOT_DRIVER_NODE;

        if !(*DriverNode).ListEntry.RegistryPath.Buffer.is_null() {
            ((*Hive).Free.unwrap())(
                (*DriverNode).ListEntry.RegistryPath.Buffer as *mut core::ffi::c_void,
                (*DriverNode).ListEntry.RegistryPath.MaximumLength as u32,
            );
        }
        if !(*DriverNode).ListEntry.FilePath.Buffer.is_null() {
            ((*Hive).Free.unwrap())(
                (*DriverNode).ListEntry.FilePath.Buffer as *mut core::ffi::c_void,
                (*DriverNode).ListEntry.FilePath.MaximumLength as u32,
            );
        }
        if !(*DriverNode).Name.Buffer.is_null() {
            ((*Hive).Free.unwrap())(
                (*DriverNode).Name.Buffer as *mut core::ffi::c_void,
                (*DriverNode).Name.MaximumLength as u32,
            );
        }
        ((*Hive).Free.unwrap())(
            DriverNode as *mut core::ffi::c_void,
            core::mem::size_of::<BOOT_DRIVER_NODE>() as u32,
        );
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     BSD - See COPYING.ARM in the top level directory
//  * PURPOSE:     Configuration Manager - Boot Initialization
//  * COPYRIGHT:   Copyright 2007 Alex Ionescu (alex.ionescu@reactos.org)
//  *              Copyright 2010 ReactOS Portable Systems Group
//  *              Copyright 2022 Hermès Bélusca-Maïto
//  *
//  * NOTE: This module is shared by both the kernel and the bootloader.
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #ifdef _BLDR_
//
// #undef CODE_SEG
// #define CODE_SEG(...)
//
// #include <ntstrsafe.h>
// #include <cmlib.h>
// #include "internal/cmboot.h"
//
// // HACK: This is part of non-NT-compatible SafeBoot support in kernel.
// ULONG InitSafeBootMode = 0;
//
// DBG_DEFAULT_CHANNEL(REGISTRY);
// #define CMTRACE(x, fmt, ...) TRACE(fmt, ##__VA_ARGS__) // DPRINT
//
// #endif /* _BLDR_ */
//
//
// /* DEFINES ********************************************************************/
//
// #define CM_BOOT_DEBUG   0x20
//
// #define IS_NULL_TERMINATED(Buffer, Size) \
//     (((Size) >= sizeof(WCHAR)) && ((Buffer)[(Size) / sizeof(WCHAR) - 1] == UNICODE_NULL))
//
//
// /* FUNCTIONS ******************************************************************/
//
// // HACK: This is part of non-NT-compatible SafeBoot support in kernel.
// extern ULONG InitSafeBootMode;
//
// CODE_SEG("INIT")
// static
// BOOLEAN
// CmpIsSafe(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX SafeBootCell,
//     _In_ HCELL_INDEX DriverCell);
//
// /**
//  * @brief
//  * Finds the corresponding "HKLM\SYSTEM\ControlSetXXX" system control set
//  * registry key, according to the "Current", "Default", or "LastKnownGood"
//  * values in the "HKLM\SYSTEM\Select" registry key.
//  *
//  * @param[in]   SystemHive
//  * The SYSTEM hive.
//  *
//  * @param[in]   RootCell
//  * The root cell of the SYSTEM hive.
//  *
//  * @param[in]   SelectKeyName
//  * The control set to check for: either "Current", "Default", or
//  * "LastKnownGood", the value of which selects the corresponding
//  * "HKLM\SYSTEM\ControlSetXXX" control set registry key.
//  *
//  * @param[out]  AutoSelect
//  * Value of the "AutoSelect" registry value (unused).
//  *
//  * @return
//  * The control set registry key's hive cell (if found), or HCELL_NIL.
//  **/
// CODE_SEG("INIT")
// HCELL_INDEX
// NTAPI
// CmpFindControlSet(
//     _In_ PHHIVE SystemHive,
//     _In_ HCELL_INDEX RootCell,
//     _In_ PCUNICODE_STRING SelectKeyName,
//     _Out_ PBOOLEAN AutoSelect)
// {
//     UNICODE_STRING Name;
//     PCM_KEY_NODE Node;
//     HCELL_INDEX SelectCell, AutoSelectCell, SelectValueCell, ControlSetCell;
//     HCELL_INDEX CurrentValueCell;
//     PCM_KEY_VALUE Value;
//     ULONG Length;
//     NTSTATUS Status;
//     PULONG CurrentData;
//     PULONG ControlSetId;
//     WCHAR Buffer[128];
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(SystemHive->ReleaseCellRoutine == NULL);
//
//     /* Get the Select key */
//     RtlInitUnicodeString(&Name, L"select");
//     Node = (PCM_KEY_NODE)HvGetCell(SystemHive, RootCell);
//     if (!Node) return HCELL_NIL;
//     SelectCell = CmpFindSubKeyByName(SystemHive, Node, &Name);
//     if (SelectCell == HCELL_NIL) return HCELL_NIL;
//
//     /* Get AutoSelect value */
//     RtlInitUnicodeString(&Name, L"AutoSelect");
//     Node = (PCM_KEY_NODE)HvGetCell(SystemHive, SelectCell);
//     if (!Node) return HCELL_NIL;
//     AutoSelectCell = CmpFindValueByName(SystemHive, Node, &Name);
//     if (AutoSelectCell == HCELL_NIL)
//     {
//         /* Assume TRUE if the value is missing */
//         *AutoSelect = TRUE;
//     }
//     else
//     {
//         /* Read the value */
//         Value = (PCM_KEY_VALUE)HvGetCell(SystemHive, AutoSelectCell);
//         if (!Value) return HCELL_NIL;
//         // if (Value->Type != REG_DWORD) return HCELL_NIL;
//
//         /* Convert it to a boolean */
//         CurrentData = (PULONG)CmpValueToData(SystemHive, Value, &Length);
//         if (!CurrentData) return HCELL_NIL;
//         // if (Length < sizeof(ULONG)) return HCELL_NIL;
//
//         *AutoSelect = *(PBOOLEAN)CurrentData;
//     }
//
//     /* Now find the control set being looked up */
//     Node = (PCM_KEY_NODE)HvGetCell(SystemHive, SelectCell);
//     if (!Node) return HCELL_NIL;
//     SelectValueCell = CmpFindValueByName(SystemHive, Node, SelectKeyName);
//     if (SelectValueCell == HCELL_NIL) return HCELL_NIL;
//
//     /* Read the value (corresponding to the CCS ID) */
//     Value = (PCM_KEY_VALUE)HvGetCell(SystemHive, SelectValueCell);
//     if (!Value) return HCELL_NIL;
//     if (Value->Type != REG_DWORD) return HCELL_NIL;
//     ControlSetId = (PULONG)CmpValueToData(SystemHive, Value, &Length);
//     if (!ControlSetId) return HCELL_NIL;
//     if (Length < sizeof(ULONG)) return HCELL_NIL;
//
//     /* Now build the CCS's Name */
//     Status = RtlStringCbPrintfW(Buffer, sizeof(Buffer),
//                                 L"ControlSet%03lu", *ControlSetId);
//     if (!NT_SUCCESS(Status)) return HCELL_NIL;
//     /* RtlStringCbPrintfW ensures the buffer to be NULL-terminated */
//     RtlInitUnicodeString(&Name, Buffer);
//
//     /* Now open it */
//     Node = (PCM_KEY_NODE)HvGetCell(SystemHive, RootCell);
//     if (!Node) return HCELL_NIL;
//     ControlSetCell = CmpFindSubKeyByName(SystemHive, Node, &Name);
//     if (ControlSetCell == HCELL_NIL) return HCELL_NIL;
//
//     /* Get the value of the "Current" CCS */
//     RtlInitUnicodeString(&Name, L"Current");
//     Node =  (PCM_KEY_NODE)HvGetCell(SystemHive, SelectCell);
//     if (!Node) return HCELL_NIL;
//     CurrentValueCell = CmpFindValueByName(SystemHive, Node, &Name);
//
//     /* Make sure it exists */
//     if (CurrentValueCell != HCELL_NIL)
//     {
//         /* Get the current value and make sure it's a ULONG */
//         Value = (PCM_KEY_VALUE)HvGetCell(SystemHive, CurrentValueCell);
//         if (!Value) return HCELL_NIL;
//         if (Value->Type == REG_DWORD)
//         {
//             /* Get the data and update it */
//             CurrentData = (PULONG)CmpValueToData(SystemHive, Value, &Length);
//             if (!CurrentData) return HCELL_NIL;
//             if (Length < sizeof(ULONG)) return HCELL_NIL;
//
//             *CurrentData = *ControlSetId;
//         }
//     }
//
//     /* Return the CCS cell */
//     return ControlSetCell;
// }
//
// /**
//  * @brief
//  * Finds the index of the driver's "Tag" value
//  * in its corresponding group ordering list.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   TagCell
//  * The driver's "Tag" registry value's hive cell.
//  *
//  * @param[in]   GroupOrderCell
//  * The hive cell of the "Control\GroupOrderList" registry key
//  * inside the currently selected control set.
//  *
//  * @param[in]   GroupName
//  * The driver's group name.
//  *
//  * @return
//  * The corresponding tag index, or -1 (last position),
//  * or -2 (next-to-last position).
//  **/
// CODE_SEG("INIT")
// static
// ULONG
// CmpFindTagIndex(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX TagCell,
//     _In_ HCELL_INDEX GroupOrderCell,
//     _In_ PCUNICODE_STRING GroupName)
// {
//     PCM_KEY_VALUE TagValue, Value;
//     PCM_KEY_NODE Node;
//     HCELL_INDEX OrderCell;
//     PULONG DriverTag, TagOrder;
//     ULONG CurrentTag, Length;
//     BOOLEAN BufferAllocated;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Get the tag */
//     Value = (PCM_KEY_VALUE)HvGetCell(Hive, TagCell);
//     if (!Value) return -2;
//     if (Value->Type != REG_DWORD) return -2;
//     DriverTag = (PULONG)CmpValueToData(Hive, Value, &Length);
//     if (!DriverTag) return -2;
//     if (Length < sizeof(ULONG)) return -2;
//
//     /* Get the order array */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, GroupOrderCell);
//     if (!Node) return -2;
//     OrderCell = CmpFindValueByName(Hive, Node, GroupName);
//     if (OrderCell == HCELL_NIL) return -2;
//
//     /* And read it */
//     TagValue = (PCM_KEY_VALUE)HvGetCell(Hive, OrderCell);
//     if (!TagValue) return -2;
//     if (!CmpGetValueData(Hive,
//                          TagValue,
//                          &Length,
//                          (PVOID*)&TagOrder,
//                          &BufferAllocated,
//                          &OrderCell)
//         || !TagOrder)
//     {
//         return -2;
//     }
//
//     /* Parse each tag */
//     for (CurrentTag = 1; CurrentTag <= TagOrder[0]; CurrentTag++)
//     {
//         /* Find a match */
//         if (TagOrder[CurrentTag] == *DriverTag)
//         {
//             /* Found it -- return the tag */
//             if (BufferAllocated) Hive->Free(TagOrder, Length);
//             return CurrentTag;
//         }
//     }
//
//     /* No matches, so assume next to last ordering */
//     if (BufferAllocated) Hive->Free(TagOrder, Length);
//     return -2;
// }
//
// #ifdef _BLDR_
//
// /**
//  * @brief
//  * Checks whether the specified named driver is already in the driver list.
//  * Optionally returns its corresponding driver node.
//  *
//  * @remarks Used in bootloader only.
//  *
//  * @param[in]   DriverListHead
//  * The driver list.
//  *
//  * @param[in]   DriverName
//  * The name of the driver to search for.
//  *
//  * @param[out]  FoundDriver
//  * Optional pointer that receives in output the address of the
//  * matching driver node, if any, or NULL if none has been found.
//  *
//  * @return
//  * TRUE if the driver has been found, FALSE if not.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpIsDriverInList(
//     _In_ PLIST_ENTRY DriverListHead,
//     _In_ PCUNICODE_STRING DriverName,
//     _Out_opt_ PBOOT_DRIVER_NODE* FoundDriver)
// {
//     PLIST_ENTRY Entry;
//     PBOOT_DRIVER_NODE DriverNode;
//
//     for (Entry = DriverListHead->Flink;
//          Entry != DriverListHead;
//          Entry = Entry->Flink)
//     {
//         DriverNode = CONTAINING_RECORD(Entry,
//                                        BOOT_DRIVER_NODE,
//                                        ListEntry.Link);
//
//         if (RtlEqualUnicodeString(&DriverNode->Name,
//                                   DriverName,
//                                   TRUE))
//         {
//             /* The driver node has been found */
//             if (FoundDriver)
//                 *FoundDriver = DriverNode;
//             return TRUE;
//         }
//     }
//
//     /* None has been found */
//     if (FoundDriver)
//         *FoundDriver = NULL;
//     return FALSE;
// }
//
// #endif /* _BLDR_ */
//
// /**
//  * @brief
//  * Inserts the specified driver entry into the driver list.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   DriverCell
//  * The registry key's hive cell of the driver to be added, inside
//  * the "Services" sub-key of the currently selected control set.
//  *
//  * @param[in]   GroupOrderCell
//  * The hive cell of the "Control\GroupOrderList" registry key
//  * inside the currently selected control set.
//  *
//  * @param[in]   RegistryPath
//  * Constant UNICODE_STRING pointing to
//  * "\\Registry\\Machine\\System\\CurrentControlSet\\Services\\".
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list where to insert the driver entry.
//  *
//  * @return
//  * TRUE if the driver has been inserted into the list, FALSE if not.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpAddDriverToList(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX DriverCell,
//     _In_ HCELL_INDEX GroupOrderCell,
//     _In_ PCUNICODE_STRING RegistryPath,
//     _Inout_ PLIST_ENTRY DriverListHead)
// {
//     PBOOT_DRIVER_NODE DriverNode;
//     PBOOT_DRIVER_LIST_ENTRY DriverEntry;
//     PCM_KEY_NODE Node;
//     PCM_KEY_VALUE Value;
//     ULONG Length;
//     USHORT NameLength;
//     HCELL_INDEX ValueCell, TagCell;
//     PUNICODE_STRING FilePath, RegistryString;
//     UNICODE_STRING Name;
//     PULONG ErrorControl;
//     PWCHAR Buffer;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Allocate a driver node and initialize it */
//     DriverNode = Hive->Allocate(sizeof(BOOT_DRIVER_NODE), FALSE, TAG_CM);
//     if (!DriverNode)
//         return FALSE;
//
//     RtlZeroMemory(DriverNode, sizeof(BOOT_DRIVER_NODE));
//     DriverEntry = &DriverNode->ListEntry;
//
//     /* Get the driver cell */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, DriverCell);
//     if (!Node)
//         goto Failure;
//
//     /* Get the name from the cell */
//     NameLength = (Node->Flags & KEY_COMP_NAME) ?
//                  CmpCompressedNameSize(Node->Name, Node->NameLength) :
//                  Node->NameLength;
//     if (NameLength < sizeof(WCHAR))
//         goto Failure;
//
//     /* Now allocate the buffer for it and copy the name */
//     RtlInitEmptyUnicodeString(&DriverNode->Name,
//                               Hive->Allocate(NameLength, FALSE, TAG_CM),
//                               NameLength);
//     if (!DriverNode->Name.Buffer)
//         goto Failure;
//
//     DriverNode->Name.Length = NameLength;
//     if (Node->Flags & KEY_COMP_NAME)
//     {
//         /* Compressed name */
//         CmpCopyCompressedName(DriverNode->Name.Buffer,
//                               DriverNode->Name.Length,
//                               Node->Name,
//                               Node->NameLength);
//     }
//     else
//     {
//         /* Normal name */
//         RtlCopyMemory(DriverNode->Name.Buffer, Node->Name, Node->NameLength);
//     }
//
//     /* Now find the image path */
//     RtlInitUnicodeString(&Name, L"ImagePath");
//     ValueCell = CmpFindValueByName(Hive, Node, &Name);
//     if (ValueCell == HCELL_NIL)
//     {
//         /* Could not find it, so assume the drivers path */
//         Length = sizeof(L"System32\\Drivers\\") + NameLength + sizeof(L".sys");
//
//         /* Allocate the path name */
//         FilePath = &DriverEntry->FilePath;
//         RtlInitEmptyUnicodeString(FilePath,
//                                   Hive->Allocate(Length, FALSE, TAG_CM),
//                                   (USHORT)Length);
//         if (!FilePath->Buffer)
//             goto Failure;
//
//         /* Write the path name */
//         if (!NT_SUCCESS(RtlAppendUnicodeToString(FilePath, L"System32\\Drivers\\"))  ||
//             !NT_SUCCESS(RtlAppendUnicodeStringToString(FilePath, &DriverNode->Name)) ||
//             !NT_SUCCESS(RtlAppendUnicodeToString(FilePath, L".sys")))
//         {
//             goto Failure;
//         }
//     }
//     else
//     {
//         /* Path name exists, so grab it */
//         Value = (PCM_KEY_VALUE)HvGetCell(Hive, ValueCell);
//         if (!Value)
//             goto Failure;
//         if ((Value->Type != REG_SZ) && (Value->Type != REG_EXPAND_SZ))
//             goto Failure;
//         Buffer = (PWCHAR)CmpValueToData(Hive, Value, &Length);
//         if (!Buffer)
//             goto Failure;
//         if (IS_NULL_TERMINATED(Buffer, Length))
//             Length -= sizeof(UNICODE_NULL);
//         if (Length < sizeof(WCHAR))
//             goto Failure;
//
//         /* Allocate and setup the path name */
//         FilePath = &DriverEntry->FilePath;
//         RtlInitEmptyUnicodeString(FilePath,
//                                   Hive->Allocate(Length, FALSE, TAG_CM),
//                                   (USHORT)Length);
//         if (!FilePath->Buffer)
//             goto Failure;
//
//         /* Transfer the data */
//         RtlCopyMemory(FilePath->Buffer, Buffer, Length);
//         FilePath->Length = (USHORT)Length;
//     }
//
//     /* Now build the registry path */
//     RegistryString = &DriverEntry->RegistryPath;
//     Length = RegistryPath->Length + NameLength;
//     RtlInitEmptyUnicodeString(RegistryString,
//                               Hive->Allocate(Length, FALSE, TAG_CM),
//                               (USHORT)Length);
//     if (!RegistryString->Buffer)
//         goto Failure;
//
//     /* Add the driver name to it */
//     if (!NT_SUCCESS(RtlAppendUnicodeStringToString(RegistryString, RegistryPath)) ||
//         !NT_SUCCESS(RtlAppendUnicodeStringToString(RegistryString, &DriverNode->Name)))
//     {
//         goto Failure;
//     }
//
//     /* The entry is done, add it */
//     InsertHeadList(DriverListHead, &DriverEntry->Link);
//
//     /* Now find error control settings */
//     RtlInitUnicodeString(&Name, L"ErrorControl");
//     ValueCell = CmpFindValueByName(Hive, Node, &Name);
//     if (ValueCell == HCELL_NIL)
//     {
//         /* Could not find it, so assume default */
//         DriverNode->ErrorControl = NormalError;
//     }
//     else
//     {
//         /* Otherwise, read whatever the data says */
//         Value = (PCM_KEY_VALUE)HvGetCell(Hive, ValueCell);
//         if (!Value)
//             goto Failure;
//         if (Value->Type != REG_DWORD)
//             goto Failure;
//         ErrorControl = (PULONG)CmpValueToData(Hive, Value, &Length);
//         if (!ErrorControl)
//             goto Failure;
//         if (Length < sizeof(ULONG))
//             goto Failure;
//
//         DriverNode->ErrorControl = *ErrorControl;
//     }
//
//     /* Next, get the group cell */
//     RtlInitUnicodeString(&Name, L"group");
//     ValueCell = CmpFindValueByName(Hive, Node, &Name);
//     if (ValueCell == HCELL_NIL)
//     {
//         /* Could not find it, so set an empty string */
//         RtlInitEmptyUnicodeString(&DriverNode->Group, NULL, 0);
//     }
//     else
//     {
//         /* Found it, read the group value */
//         Value = (PCM_KEY_VALUE)HvGetCell(Hive, ValueCell);
//         if (!Value)
//             goto Failure;
//         if (Value->Type != REG_SZ) // REG_EXPAND_SZ not really allowed there.
//             goto Failure;
//
//         /* Copy it into the node */
//         Buffer = (PWCHAR)CmpValueToData(Hive, Value, &Length);
//         if (!Buffer)
//             goto Failure;
//         if (IS_NULL_TERMINATED(Buffer, Length))
//             Length -= sizeof(UNICODE_NULL);
//
//         DriverNode->Group.Buffer = Buffer;
//         DriverNode->Group.Length = (USHORT)Length;
//         DriverNode->Group.MaximumLength = DriverNode->Group.Length;
//     }
//
//     /* Finally, find the tag */
//     RtlInitUnicodeString(&Name, L"Tag");
//     TagCell = CmpFindValueByName(Hive, Node, &Name);
//     if (TagCell == HCELL_NIL)
//     {
//         /* No tag, so load last */
//         DriverNode->Tag = -1;
//     }
//     else
//     {
//         /* Otherwise, decode it based on tag order */
//         DriverNode->Tag = CmpFindTagIndex(Hive,
//                                           TagCell,
//                                           GroupOrderCell,
//                                           &DriverNode->Group);
//     }
//
//     CMTRACE(CM_BOOT_DEBUG, "Adding boot driver: '%wZ', '%wZ'\n",
//             &DriverNode->Name, &DriverEntry->FilePath);
//
//     /* All done! */
//     return TRUE;
//
// Failure:
//     if (DriverEntry->RegistryPath.Buffer)
//     {
//         Hive->Free(DriverEntry->RegistryPath.Buffer,
//                    DriverEntry->RegistryPath.MaximumLength);
//     }
//     if (DriverEntry->FilePath.Buffer)
//     {
//         Hive->Free(DriverEntry->FilePath.Buffer,
//                    DriverEntry->FilePath.MaximumLength);
//     }
//     if (DriverNode->Name.Buffer)
//     {
//         Hive->Free(DriverNode->Name.Buffer,
//                    DriverNode->Name.MaximumLength);
//     }
//     Hive->Free(DriverNode, sizeof(BOOT_DRIVER_NODE));
//
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Checks whether the specified driver has the expected load type.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   DriverCell
//  * The registry key's hive cell of the driver, inside the
//  * "Services" sub-key of the currently selected control set.
//  *
//  * @param[in]   LoadType
//  * The load type the driver should match.
//  *
//  * @return
//  * TRUE if the driver's load type matches, FALSE if not.
//  **/
// CODE_SEG("INIT")
// static
// BOOLEAN
// CmpIsLoadType(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX Cell,
//     _In_ SERVICE_LOAD_TYPE LoadType)
// {
//     PCM_KEY_NODE Node;
//     PCM_KEY_VALUE Value;
//     UNICODE_STRING Name = RTL_CONSTANT_STRING(L"Start");
//     HCELL_INDEX ValueCell;
//     ULONG Length;
//     PULONG Data;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Open the start cell */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, Cell);
//     if (!Node) return FALSE;
//     ValueCell = CmpFindValueByName(Hive, Node, &Name);
//     if (ValueCell == HCELL_NIL) return FALSE;
//
//     /* Read the start value */
//     Value = (PCM_KEY_VALUE)HvGetCell(Hive, ValueCell);
//     if (!Value) return FALSE;
//     if (Value->Type != REG_DWORD) return FALSE;
//     Data = (PULONG)CmpValueToData(Hive, Value, &Length);
//     if (!Data) return FALSE;
//     if (Length < sizeof(ULONG)) return FALSE;
//
//     /* Return if the type matches */
//     return (*Data == LoadType);
// }
//
// /**
//  * @brief
//  * Enumerates all drivers within the given control set and load type,
//  * present in the "Services" sub-key, and inserts them into the driver list.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   ControlSet
//  * The control set registry key's hive cell.
//  *
//  * @param[in]   LoadType
//  * The load type the driver should match.
//  *
//  * @param[in]   BootFileSystem
//  * Optional name of the boot file system, for which to insert
//  * its corresponding driver.
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list where to insert the enumerated drivers.
//  *
//  * @return
//  * TRUE if the drivers have been successfully enumerated and inserted,
//  * FALSE if not.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpFindDrivers(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX ControlSet,
//     _In_ SERVICE_LOAD_TYPE LoadType,
//     _In_opt_ PCWSTR BootFileSystem,
//     _Inout_ PLIST_ENTRY DriverListHead)
// {
//     HCELL_INDEX ServicesCell, ControlCell, GroupOrderCell, DriverCell;
//     HCELL_INDEX SafeBootCell = HCELL_NIL;
//     ULONG i;
//     UNICODE_STRING Name;
//     UNICODE_STRING KeyPath;
//     PCM_KEY_NODE ControlNode, ServicesNode, Node;
//     PBOOT_DRIVER_NODE FsNode;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Open the control set key */
//     ControlNode = (PCM_KEY_NODE)HvGetCell(Hive, ControlSet);
//     if (!ControlNode) return FALSE;
//
//     /* Get services cell */
//     RtlInitUnicodeString(&Name, L"Services");
//     ServicesCell = CmpFindSubKeyByName(Hive, ControlNode, &Name);
//     if (ServicesCell == HCELL_NIL) return FALSE;
//
//     /* Open services key */
//     ServicesNode = (PCM_KEY_NODE)HvGetCell(Hive, ServicesCell);
//     if (!ServicesNode) return FALSE;
//
//     /* Get control cell */
//     RtlInitUnicodeString(&Name, L"Control");
//     ControlCell = CmpFindSubKeyByName(Hive, ControlNode, &Name);
//     if (ControlCell == HCELL_NIL) return FALSE;
//
//     /* Get the group order cell and read it */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, ControlCell);
//     if (!Node) return FALSE;
//     RtlInitUnicodeString(&Name, L"GroupOrderList");
//     GroupOrderCell = CmpFindSubKeyByName(Hive, Node, &Name);
//     if (GroupOrderCell == HCELL_NIL) return FALSE;
//
//     /* Get Safe Boot cell */
//     if (InitSafeBootMode)
//     {
//         /* Open the Safe Boot key */
//         RtlInitUnicodeString(&Name, L"SafeBoot");
//         Node = (PCM_KEY_NODE)HvGetCell(Hive, ControlCell);
//         if (!Node) return FALSE;
//         SafeBootCell = CmpFindSubKeyByName(Hive, Node, &Name);
//         if (SafeBootCell == HCELL_NIL) return FALSE;
//
//         /* Open the correct start key (depending on the mode) */
//         Node = (PCM_KEY_NODE)HvGetCell(Hive, SafeBootCell);
//         if (!Node) return FALSE;
//         switch (InitSafeBootMode)
//         {
//             /* NOTE: Assumes MINIMAL (1) and DSREPAIR (3) load same items */
//             case 1:
//             case 3: RtlInitUnicodeString(&Name, L"Minimal"); break;
//             case 2: RtlInitUnicodeString(&Name, L"Network"); break;
//             default: return FALSE;
//         }
//         SafeBootCell = CmpFindSubKeyByName(Hive, Node, &Name);
//         if (SafeBootCell == HCELL_NIL) return FALSE;
//     }
//
//     /* Build the root registry path */
//     RtlInitUnicodeString(&KeyPath, L"\\Registry\\Machine\\System\\CurrentControlSet\\Services\\");
//
//     /* Enumerate each sub-key */
//     i = 0;
//     DriverCell = CmpFindSubKeyByNumber(Hive, ServicesNode, i);
//     while (DriverCell != HCELL_NIL)
//     {
//         /* Make sure it's a driver of this start type AND is "safe" to load */
//         if (CmpIsLoadType(Hive, DriverCell, LoadType) &&
//             CmpIsSafe(Hive, SafeBootCell, DriverCell))
//         {
//             /* Add it to the list */
//             if (!CmpAddDriverToList(Hive,
//                                     DriverCell,
//                                     GroupOrderCell,
//                                     &KeyPath,
//                                     DriverListHead))
//             {
//                 CMTRACE(CM_BOOT_DEBUG, "  Failed to add boot driver\n");
//             }
//         }
//
//         /* Go to the next sub-key */
//         DriverCell = CmpFindSubKeyByNumber(Hive, ServicesNode, ++i);
//     }
//
//     /* Check if we have a boot file system */
//     if (BootFileSystem)
//     {
//         /* Find it */
//         RtlInitUnicodeString(&Name, BootFileSystem);
//         DriverCell = CmpFindSubKeyByName(Hive, ServicesNode, &Name);
//         if (DriverCell != HCELL_NIL)
//         {
//             CMTRACE(CM_BOOT_DEBUG, "Adding Boot FileSystem '%S'\n",
//                     BootFileSystem);
//
//             /* Always add it to the list */
//             if (!CmpAddDriverToList(Hive,
//                                     DriverCell,
//                                     GroupOrderCell,
//                                     &KeyPath,
//                                     DriverListHead))
//             {
//                 CMTRACE(CM_BOOT_DEBUG, "  Failed to add boot driver\n");
//             }
//             else
//             {
//                 /* Mark it as critical so it always loads */
//                 FsNode = CONTAINING_RECORD(DriverListHead->Flink,
//                                            BOOT_DRIVER_NODE,
//                                            ListEntry.Link);
//                 FsNode->ErrorControl = SERVICE_ERROR_CRITICAL;
//             }
//         }
//     }
//
//     /* We're done! */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Performs the driver list sorting, according to the ordering list.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   ControlSet
//  * The control set registry key's hive cell.
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list to sort.
//  *
//  * @return
//  * TRUE if sorting has been successfully done, FALSE if not.
//  **/
// CODE_SEG("INIT")
// static
// BOOLEAN
// CmpDoSort(
//     _Inout_ PLIST_ENTRY DriverListHead,
//     _In_ PCUNICODE_STRING OrderList)
// {
//     PWCHAR Current, End = NULL;
//     UNICODE_STRING GroupName;
//     PLIST_ENTRY NextEntry;
//     PBOOT_DRIVER_NODE CurrentNode;
//
//     /* We're going from end to start, so get to the last group and keep going */
//     Current = &OrderList->Buffer[OrderList->Length / sizeof(WCHAR)];
//     while (Current > OrderList->Buffer)
//     {
//         /* Scan the current string */
//         do
//         {
//             if (*Current == UNICODE_NULL) End = Current;
//         } while ((*(--Current - 1) != UNICODE_NULL) && (Current != OrderList->Buffer));
//
//         /* This is our cleaned up string for this specific group */
//         ASSERT(End != NULL);
//         GroupName.Length = (USHORT)(End - Current) * sizeof(WCHAR);
//         GroupName.MaximumLength = GroupName.Length;
//         GroupName.Buffer = Current;
//
//         /* Now loop the driver list */
//         NextEntry = DriverListHead->Flink;
//         while (NextEntry != DriverListHead)
//         {
//             /* Get this node */
//             CurrentNode = CONTAINING_RECORD(NextEntry,
//                                             BOOT_DRIVER_NODE,
//                                             ListEntry.Link);
//
//             /* Get the next entry now since we'll do a relink */
//             NextEntry = NextEntry->Flink;
//
//             /* Is there a group name and does it match the current group? */
//             if (CurrentNode->Group.Buffer &&
//                 RtlEqualUnicodeString(&GroupName, &CurrentNode->Group, TRUE))
//             {
//                 /* Remove from this location and re-link in the new one */
//                 RemoveEntryList(&CurrentNode->ListEntry.Link);
//                 InsertHeadList(DriverListHead, &CurrentNode->ListEntry.Link);
//             }
//         }
//
//         /* Move on */
//         --Current;
//     }
//
//     /* All done */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Sorts the driver list, according to the drivers' group load ordering.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive.
//  *
//  * @param[in]   ControlSet
//  * The control set registry key's hive cell.
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list to sort.
//  *
//  * @return
//  * TRUE if sorting has been successfully done, FALSE if not.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpSortDriverList(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX ControlSet,
//     _Inout_ PLIST_ENTRY DriverListHead)
// {
//     PCM_KEY_NODE Node;
//     PCM_KEY_VALUE ListValue;
//     HCELL_INDEX ControlCell, GroupOrder, ListCell;
//     UNICODE_STRING Name, OrderList;
//     ULONG Length;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Open the control key */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, ControlSet);
//     if (!Node) return FALSE;
//     RtlInitUnicodeString(&Name, L"Control");
//     ControlCell = CmpFindSubKeyByName(Hive, Node, &Name);
//     if (ControlCell == HCELL_NIL) return FALSE;
//
//     /* Open the service group order */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, ControlCell);
//     if (!Node) return FALSE;
//     RtlInitUnicodeString(&Name, L"ServiceGroupOrder");
//     GroupOrder = CmpFindSubKeyByName(Hive, Node, &Name);
//     if (GroupOrder == HCELL_NIL) return FALSE;
//
//     /* Open the list key */
//     Node = (PCM_KEY_NODE)HvGetCell(Hive, GroupOrder);
//     if (!Node) return FALSE;
//     RtlInitUnicodeString(&Name, L"list");
//     ListCell = CmpFindValueByName(Hive, Node, &Name);
//     if (ListCell == HCELL_NIL) return FALSE;
//
//     /* Read the actual list */
//     ListValue = (PCM_KEY_VALUE)HvGetCell(Hive, ListCell);
//     if (!ListValue) return FALSE;
//     if (ListValue->Type != REG_MULTI_SZ) return FALSE;
//
//     /* Copy it into a buffer */
//     OrderList.Buffer = (PWCHAR)CmpValueToData(Hive, ListValue, &Length);
//     if (!OrderList.Buffer) return FALSE;
//     if (!IS_NULL_TERMINATED(OrderList.Buffer, Length)) return FALSE;
//     OrderList.Length = (USHORT)Length - sizeof(UNICODE_NULL);
//     OrderList.MaximumLength = OrderList.Length;
//
//     /* And start the sort algorithm */
//     return CmpDoSort(DriverListHead, &OrderList);
// }
//
// CODE_SEG("INIT")
// static
// BOOLEAN
// CmpOrderGroup(
//     _In_ PBOOT_DRIVER_NODE StartNode,
//     _In_ PBOOT_DRIVER_NODE EndNode)
// {
//     PBOOT_DRIVER_NODE CurrentNode, PreviousNode;
//     PLIST_ENTRY ListEntry;
//
//     /* Base case, nothing to do */
//     if (StartNode == EndNode) return TRUE;
//
//     /* Loop the nodes */
//     CurrentNode = StartNode;
//     do
//     {
//         /* Save this as the previous node */
//         PreviousNode = CurrentNode;
//
//         /* And move to the next one */
//         ListEntry = CurrentNode->ListEntry.Link.Flink;
//         CurrentNode = CONTAINING_RECORD(ListEntry,
//                                         BOOT_DRIVER_NODE,
//                                         ListEntry.Link);
//
//         /* Check if the previous driver had a bigger tag */
//         if (PreviousNode->Tag > CurrentNode->Tag)
//         {
//             /* Check if we need to update the tail */
//             if (CurrentNode == EndNode)
//             {
//                 /* Update the tail */
//                 ListEntry = CurrentNode->ListEntry.Link.Blink;
//                 EndNode = CONTAINING_RECORD(ListEntry,
//                                             BOOT_DRIVER_NODE,
//                                             ListEntry.Link);
//             }
//
//             /* Remove this driver since we need to move it */
//             RemoveEntryList(&CurrentNode->ListEntry.Link);
//
//             /* Keep looping until we find a driver with a lower tag than ours */
//             while ((PreviousNode->Tag > CurrentNode->Tag) && (PreviousNode != StartNode))
//             {
//                 /* We'll be re-inserted at this spot */
//                 ListEntry = PreviousNode->ListEntry.Link.Blink;
//                 PreviousNode = CONTAINING_RECORD(ListEntry,
//                                                  BOOT_DRIVER_NODE,
//                                                  ListEntry.Link);
//             }
//
//             /* Do the insert in the new location */
//             InsertTailList(&PreviousNode->ListEntry.Link, &CurrentNode->ListEntry.Link);
//
//             /* Update the head, if needed */
//             if (PreviousNode == StartNode) StartNode = CurrentNode;
//         }
//     } while (CurrentNode != EndNode);
//
//     /* All done */
//     return TRUE;
// }
//
// /**
//  * @brief
//  * Removes potential circular dependencies (cycles) and sorts the driver list.
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list to sort.
//  *
//  * @return
//  * Always TRUE.
//  **/
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// CmpResolveDriverDependencies(
//     _Inout_ PLIST_ENTRY DriverListHead)
// {
//     PLIST_ENTRY NextEntry;
//     PBOOT_DRIVER_NODE StartNode, EndNode, CurrentNode;
//
//     /* Loop the list */
//     NextEntry = DriverListHead->Flink;
//     while (NextEntry != DriverListHead)
//     {
//         /* Find the first entry */
//         StartNode = CONTAINING_RECORD(NextEntry,
//                                       BOOT_DRIVER_NODE,
//                                       ListEntry.Link);
//         do
//         {
//             /* Find the last entry */
//             EndNode = CONTAINING_RECORD(NextEntry,
//                                         BOOT_DRIVER_NODE,
//                                         ListEntry.Link);
//
//             /* Get the next entry */
//             NextEntry = NextEntry->Flink;
//             CurrentNode = CONTAINING_RECORD(NextEntry,
//                                             BOOT_DRIVER_NODE,
//                                             ListEntry.Link);
//
//             /* If the next entry is back to the top, break out */
//             if (NextEntry == DriverListHead) break;
//
//             /* Otherwise, check if this entry is equal */
//             if (!RtlEqualUnicodeString(&StartNode->Group,
//                                        &CurrentNode->Group,
//                                        TRUE))
//             {
//                 /* It is, so we've detected a cycle, break out */
//                 break;
//             }
//         } while (NextEntry != DriverListHead);
//
//         /* Now we have the correct start and end pointers, so do the sort */
//         CmpOrderGroup(StartNode, EndNode);
//     }
//
//     /* We're done */
//     return TRUE;
// }
//
// CODE_SEG("INIT")
// static
// BOOLEAN
// CmpIsSafe(
//     _In_ PHHIVE Hive,
//     _In_ HCELL_INDEX SafeBootCell,
//     _In_ HCELL_INDEX DriverCell)
// {
//     PCM_KEY_NODE SafeBootNode;
//     PCM_KEY_NODE DriverNode;
//     PCM_KEY_VALUE KeyValue;
//     HCELL_INDEX CellIndex;
//     ULONG Length;
//     UNICODE_STRING Name;
//     PWCHAR Buffer;
//
//     /* Sanity check: We shouldn't need to release any acquired cells */
//     ASSERT(Hive->ReleaseCellRoutine == NULL);
//
//     /* Driver key node (mandatory) */
//     ASSERT(DriverCell != HCELL_NIL);
//     DriverNode = (PCM_KEY_NODE)HvGetCell(Hive, DriverCell);
//     if (!DriverNode) return FALSE;
//
//     /* Safe boot key node (optional but return TRUE if not present) */
//     if (SafeBootCell == HCELL_NIL) return TRUE;
//     SafeBootNode = (PCM_KEY_NODE)HvGetCell(Hive, SafeBootCell);
//     if (!SafeBootNode) return FALSE;
//
//     /* Search by the name from the group */
//     RtlInitUnicodeString(&Name, L"Group");
//     CellIndex = CmpFindValueByName(Hive, DriverNode, &Name);
//     if (CellIndex != HCELL_NIL)
//     {
//         KeyValue = (PCM_KEY_VALUE)HvGetCell(Hive, CellIndex);
//         if (!KeyValue) return FALSE;
//
//         if (KeyValue->Type == REG_SZ) // REG_EXPAND_SZ not really allowed there.
//         {
//             /* Compose the search 'key' */
//             Buffer = (PWCHAR)CmpValueToData(Hive, KeyValue, &Length);
//             if (!Buffer)
//                 return FALSE;
//             if (IS_NULL_TERMINATED(Buffer, Length))
//                 Length -= sizeof(UNICODE_NULL);
//
//             Name.Buffer = Buffer;
//             Name.Length = (USHORT)Length;
//             Name.MaximumLength = Name.Length;
//
//             /* Search for corresponding key in the Safe Boot key */
//             CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
//             if (CellIndex != HCELL_NIL) return TRUE;
//         }
//     }
//
//     /* Group has not been found - find driver name */
//     Length = (DriverNode->Flags & KEY_COMP_NAME) ?
//              CmpCompressedNameSize(DriverNode->Name, DriverNode->NameLength) :
//              DriverNode->NameLength;
//     if (Length < sizeof(WCHAR))
//         return FALSE;
//
//     /* Now allocate the buffer for it and copy the name */
//     RtlInitEmptyUnicodeString(&Name,
//                               Hive->Allocate(Length, FALSE, TAG_CM),
//                               (USHORT)Length);
//     if (!Name.Buffer)
//         return FALSE;
//
//     Name.Length = (USHORT)Length;
//     if (DriverNode->Flags & KEY_COMP_NAME)
//     {
//         /* Compressed name */
//         CmpCopyCompressedName(Name.Buffer,
//                               Name.Length,
//                               DriverNode->Name,
//                               DriverNode->NameLength);
//     }
//     else
//     {
//         /* Normal name */
//         RtlCopyMemory(Name.Buffer, DriverNode->Name, DriverNode->NameLength);
//     }
//
//     CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
//     Hive->Free(Name.Buffer, Name.MaximumLength);
//     if (CellIndex != HCELL_NIL) return TRUE;
//
//     /* Not group or driver name - search by image name */
//     RtlInitUnicodeString(&Name, L"ImagePath");
//     CellIndex = CmpFindValueByName(Hive, DriverNode, &Name);
//     if (CellIndex != HCELL_NIL)
//     {
//         KeyValue = (PCM_KEY_VALUE)HvGetCell(Hive, CellIndex);
//         if (!KeyValue) return FALSE;
//
//         if ((KeyValue->Type == REG_SZ) || (KeyValue->Type == REG_EXPAND_SZ))
//         {
//             /* Compose the search 'key' */
//             Buffer = (PWCHAR)CmpValueToData(Hive, KeyValue, &Length);
//             if (!Buffer) return FALSE;
//             if (Length < sizeof(WCHAR)) return FALSE;
//
//             /* Get the base image file name */
//             // FIXME: wcsrchr() may fail if Buffer is *not* NULL-terminated!
//             Name.Buffer = wcsrchr(Buffer, OBJ_NAME_PATH_SEPARATOR);
//             if (!Name.Buffer) return FALSE;
//             ++Name.Buffer;
//
//             /* Length of the base name must be >=1 WCHAR */
//             if (((ULONG_PTR)Name.Buffer - (ULONG_PTR)Buffer) >= Length)
//                 return FALSE;
//             Length -= ((ULONG_PTR)Name.Buffer - (ULONG_PTR)Buffer);
//             if (IS_NULL_TERMINATED(Name.Buffer, Length))
//                 Length -= sizeof(UNICODE_NULL);
//             if (Length < sizeof(WCHAR)) return FALSE;
//
//             Name.Length = (USHORT)Length;
//             Name.MaximumLength = Name.Length;
//
//             /* Search for corresponding key in the Safe Boot key */
//             CellIndex = CmpFindSubKeyByName(Hive, SafeBootNode, &Name);
//             if (CellIndex != HCELL_NIL) return TRUE;
//         }
//     }
//
//     /* Nothing found - nothing else to search */
//     return FALSE;
// }
//
// /**
//  * @brief
//  * Empties the driver list and frees all allocated driver nodes in it.
//  *
//  * @param[in]   Hive
//  * The SYSTEM hive (used only for the Hive->Free() memory deallocator).
//  *
//  * @param[in,out]   DriverListHead
//  * The driver list to free.
//  *
//  * @return  None
//  **/
// CODE_SEG("INIT")
// VOID
// NTAPI
// CmpFreeDriverList(
//     _In_ PHHIVE Hive,
//     _Inout_ PLIST_ENTRY DriverListHead)
// {
//     PLIST_ENTRY Entry;
//     PBOOT_DRIVER_NODE DriverNode;
//
//     /* Loop through the list and remove each driver node */
//     while (!IsListEmpty(DriverListHead))
//     {
//         /* Get the driver node */
//         Entry = RemoveHeadList(DriverListHead);
//         DriverNode = CONTAINING_RECORD(Entry,
//                                        BOOT_DRIVER_NODE,
//                                        ListEntry.Link);
//
//         /* Free any allocated string buffers, then the node */
//         if (DriverNode->ListEntry.RegistryPath.Buffer)
//         {
//             Hive->Free(DriverNode->ListEntry.RegistryPath.Buffer,
//                        DriverNode->ListEntry.RegistryPath.MaximumLength);
//         }
//         if (DriverNode->ListEntry.FilePath.Buffer)
//         {
//             Hive->Free(DriverNode->ListEntry.FilePath.Buffer,
//                        DriverNode->ListEntry.FilePath.MaximumLength);
//         }
//         if (DriverNode->Name.Buffer)
//         {
//             Hive->Free(DriverNode->Name.Buffer,
//                        DriverNode->Name.MaximumLength);
//         }
//         Hive->Free(DriverNode, sizeof(BOOT_DRIVER_NODE));
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
