//! prefxsup.rs — prefxsup.c: Pipes Prefixes Support (prefix-table lookup helpers).
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::ptr::{addr_of, addr_of_mut, null_mut};
use ntkrnl_sys::base::{NT_SUCCESS, InitializeListHead, InsertHeadList, InsertTailList, RemoveEntryList, RemoveHeadList, RemoveTailList, IsListEmpty};
use ntkrnl_sys::ddk::*;
use crate::glue::*;
use crate::globals::*;
use crate::types::*;
use crate::helpers::*;
use crate::CONTAINING_RECORD;

use ntkrnl_sys::rtl::RtlCopyMemory;
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
use ntkrnl_sys::ke::KeBugCheckEx;
use ntkrnl_sys::constants::OBJ_NAME_PATH_SEPARATOR;
use ntkrnl_sys::driver_ddk::UNICODE_NULL;

// File ID number for NPFS bugchecking support: NPFS_BUGCHECK_FILE_ID == NPFS_BUGCHECK_PREFXSUP.

// sizeof(WCHAR) — UTF-16 code unit (xdk: WCHAR == wchar_t == 2 bytes on Windows).
const SIZEOF_WCHAR: USHORT = core::mem::size_of::<u16>() as USHORT;

/// PNP_FCB NpFindPrefix(IN PUNICODE_STRING Name, IN ULONG CaseInsensitiveIndex,
///                      IN PUNICODE_STRING Prefix)
#[no_mangle]
pub unsafe extern "system" fn NpFindPrefix(
    Name: PUNICODE_STRING,
    CaseInsensitiveIndex: ULONG,
    Prefix: PUNICODE_STRING,
) -> PNP_FCB {
    let mut Entry: PUNICODE_PREFIX_TABLE_ENTRY;
    let mut Fcb: PNP_FCB;
    // PAGED_CODE();

    Entry = RtlFindUnicodePrefix(
        addr_of_mut!((*NpVcb).PrefixTable),
        Name,
        CaseInsensitiveIndex,
    );
    if Entry.is_null() {
        // NpBugCheck(0, 0, 0); — prefxsup.c line 31
        KeBugCheckEx(
            NPFS_FILE_SYSTEM,
            ((NPFS_BUGCHECK_PREFXSUP << 16) | 31u32) as _,
            0,
            0,
            0,
        );
    }

    Fcb = CONTAINING_RECORD!(Entry, NP_FCB, PrefixTableEntry);

    (*Prefix).Length = (*Name).Length - (*Fcb).FullName.Length;
    (*Prefix).MaximumLength = (*Prefix).Length;
    (*Prefix).Buffer = &mut *(*Name)
        .Buffer
        .add(((*Fcb).FullName.Length / SIZEOF_WCHAR) as usize);

    if ((*Prefix).Length != 0)
        && (*(*Prefix).Buffer == OBJ_NAME_PATH_SEPARATOR)
    {
        (*Prefix).Length -= SIZEOF_WCHAR;
        (*Prefix).MaximumLength -= SIZEOF_WCHAR;
        (*Prefix).Buffer = (*Prefix).Buffer.add(1);
    }

    Fcb
}

/// NTSTATUS NpFindRelativePrefix(IN PNP_DCB Dcb, IN PUNICODE_STRING Name,
///     IN ULONG CaseInsensitiveIndex, IN PUNICODE_STRING Prefix, OUT PNP_FCB *FoundFcb)
#[no_mangle]
pub unsafe extern "system" fn NpFindRelativePrefix(
    Dcb: PNP_DCB,
    Name: PUNICODE_STRING,
    CaseInsensitiveIndex: ULONG,
    Prefix: PUNICODE_STRING,
    FoundFcb: *mut PNP_FCB,
) -> NTSTATUS {
    let mut Buffer: PWCHAR;
    let mut Fcb: PNP_FCB;
    let mut RootName: UNICODE_STRING = core::mem::zeroed();
    let Length: USHORT;
    let MaximumLength: USHORT;
    // PAGED_CODE();

    Length = (*Name).Length;
    // sizeof(OBJ_NAME_PATH_SEPARATOR) + sizeof(UNICODE_NULL) — both WCHAR-typed in C.
    MaximumLength = Length + SIZEOF_WCHAR + SIZEOF_WCHAR;
    if MaximumLength < Length {
        return STATUS_INVALID_PARAMETER;
    }

    // ASSERT(Dcb->NodeType == NPFS_NTC_ROOT_DCB);
    debug_assert!((*Dcb).NodeType == NPFS_NTC_ROOT_DCB);

    Buffer = ExAllocatePoolWithTag(
        PagedPool,
        MaximumLength as _,
        NPFS_NAME_BLOCK_TAG,
    ) as PWCHAR;
    if Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *Buffer = OBJ_NAME_PATH_SEPARATOR;
    RtlCopyMemory(
        Buffer.add(1) as PVOID,
        (*Name).Buffer as PCVOID,
        Length as _,
    );
    *Buffer.add(((Length / SIZEOF_WCHAR) + 1) as usize) = UNICODE_NULL;

    RootName.Length = Length + SIZEOF_WCHAR; // + sizeof(OBJ_NAME_PATH_SEPARATOR)
    RootName.MaximumLength = MaximumLength;
    RootName.Buffer = Buffer;

    Fcb = NpFindPrefix(addr_of_mut!(RootName), CaseInsensitiveIndex, Prefix);

    ExFreePool(Buffer as PVOID);

    (*Prefix).Buffer = &mut *(*Name)
        .Buffer
        .add(((Length - (*Prefix).Length) / SIZEOF_WCHAR) as usize);
    *FoundFcb = Fcb;

    STATUS_SUCCESS
}

/* EOF */
