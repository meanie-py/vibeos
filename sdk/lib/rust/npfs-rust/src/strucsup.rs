//! strucsup.rs — strucsup.c: NPFS structure (VCB/DCB/FCB/CCB + event-table) lifecycle —
//! create/delete/initialize. The global NpVcb lives in crate::globals; the RTL_GENERIC_TABLE
//! event-table callbacks (compare/allocate/deallocate) are NTAPI fns transmuted into the
//! bindgen `Option<extern "C" fn>` slots (contract §9). No SEH in this file.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::mem::{size_of, transmute};
use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{
    BOOLEAN, InitializeListHead, InsertTailList, NTSTATUS, NT_SUCCESS, PCVOID, PVOID,
    RemoveEntryList, SIZE_T, UCHAR, ULONG, USHORT,
};
use ntkrnl_sys::ddk::{
    CLONG, ExFreePool, ExInitializeResourceLite, ExDeleteResourceLite, LARGE_INTEGER, LIST_ENTRY,
    ObDereferenceSecurityDescriptor, PFILE_OBJECT, PRTL_GENERIC_TABLE,
    PRTL_GENERIC_ALLOCATE_ROUTINE, PRTL_GENERIC_COMPARE_ROUTINE, PRTL_GENERIC_FREE_ROUTINE,
    RTL_GENERIC_COMPARE_RESULTS, RTL_GENERIC_TABLE, RtlDeleteElementGenericTable,
    RtlInitializeGenericTable, RtlInitializeUnicodePrefix, RtlInsertUnicodePrefix,
    RtlRemoveUnicodePrefix, UNICODE_STRING,
};
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
use ntkrnl_sys::ke::KeBugCheckEx;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

use crate::glue::*;
use crate::globals::NpVcb;
use crate::types::*;

// Cross-module helpers (declared NTAPI in npfs.h):
use crate::create::NpCheckForNotify;
use crate::datasup::{NpInitializeDataQueue, NpUninitializeDataQueue};
use crate::secursup::NpUninitializeSecurity;
use crate::waitsup::{NpCancelWaiter, NpInitializeWaitQueue};

// File ID number for NPFS bugchecking support: NPFS_BUGCHECK_STRUCSUP (glue.rs).
// NpBugCheck(p1,p2,p3) -> KeBugCheckEx(NPFS_FILE_SYSTEM, (FILEID<<16)|line, p1,p2,p3).

// RTL_GENERIC_COMPARE_RESULTS::GenericEqual (xdk/rtltypes.h: enum value 2).
const GenericEqual: RTL_GENERIC_COMPARE_RESULTS = 2;

// UNICODE_NULL (xdk/ntbasedef.h: #define UNICODE_NULL ((WCHAR)0)) — sizeof = sizeof(WCHAR) = 2.
const UNICODE_NULL: u16 = 0;
const SIZEOF_UNICODE_NULL: USHORT = 2;
// OBJ_NAME_PATH_SEPARATOR (xdk/obtypes.h: ((WCHAR)L'\\')) — sizeof = sizeof(WCHAR) = 2.
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;
const SIZEOF_OBJ_NAME_PATH_SEPARATOR: USHORT = 2;
const SIZEOF_WCHAR: USHORT = 2;

// C: WCHAR NpRootDCBName[] = L"\\";  (2 WCHARs: '\\', '\0')
static mut NpRootDCBName: [u16; 2] = [b'\\' as u16, 0];

// --- generic-table callback slot aliases (contract §9: our fns are extern "system",
//     the slots are Option<extern "C" fn>; define matching extern "system" aliases and
//     transmute the fn items into the bindgen slot types). ---
type EventTableCompareFn = unsafe extern "system" fn(
    PRTL_GENERIC_TABLE,
    PVOID,
    PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS;
type EventTableAllocateFn = unsafe extern "system" fn(PRTL_GENERIC_TABLE, CLONG) -> PVOID;
type EventTableDeallocateFn = unsafe extern "system" fn(PRTL_GENERIC_TABLE, PVOID);

pub unsafe extern "system" fn NpEventTableCompareRoutine(
    Table: PRTL_GENERIC_TABLE,
    FirstStruct: PVOID,
    SecondStruct: PVOID,
) -> RTL_GENERIC_COMPARE_RESULTS {
    // UNIMPLEMENTED;
    GenericEqual
}

pub unsafe extern "system" fn NpEventTableAllocate(
    Table: PRTL_GENERIC_TABLE,
    ByteSize: CLONG,
) -> PVOID {
    // UNIMPLEMENTED;
    null_mut()
}

pub unsafe extern "system" fn NpEventTableDeallocate(
    Table: PRTL_GENERIC_TABLE,
    Buffer: PVOID,
) {
    // UNIMPLEMENTED;
}

pub unsafe extern "system" fn NpDeleteEventTableEntry(
    Table: PRTL_GENERIC_TABLE,
    Buffer: PVOID,
) -> BOOLEAN {
    if Buffer.is_null() {
        return FALSE;
    }

    ObDereferenceObject((*(Buffer as PNP_EVENT_BUFFER)).Event as PVOID);
    RtlDeleteElementGenericTable(Table, Buffer)
}

pub unsafe extern "system" fn NpDeleteFcb(Fcb: PNP_FCB, ListEntry: *mut LIST_ENTRY) {
    let Dcb: PNP_DCB;
    // PAGED_CODE();

    Dcb = (*Fcb).ParentDcb as PNP_DCB;
    if (*Fcb).CurrentInstances != 0 {
        // NpBugCheck(0, 0, 0);  __LINE__ == 70
        KeBugCheckEx(
            NPFS_FILE_SYSTEM,
            ((NPFS_BUGCHECK_STRUCSUP << 16) | 70) as _,
            0,
            0,
            0,
        );
    }

    NpCancelWaiter(
        addr_of_mut!((*NpVcb).WaitQueue),
        addr_of_mut!((*Fcb).FullName),
        STATUS_OBJECT_NAME_NOT_FOUND,
        ListEntry,
    );

    RemoveEntryList(addr_of_mut!((*Fcb).DcbEntry));

    if !(*Fcb).SecurityDescriptor.is_null() {
        ObDereferenceSecurityDescriptor((*Fcb).SecurityDescriptor, 1);
    }

    RtlRemoveUnicodePrefix(
        addr_of_mut!((*NpVcb).PrefixTable),
        addr_of_mut!((*Fcb).PrefixTableEntry),
    );
    ExFreePool((*Fcb).FullName.Buffer as PVOID);
    ExFreePool(Fcb as PVOID);
    NpCheckForNotify(Dcb, TRUE, ListEntry);
}

pub unsafe extern "system" fn NpDeleteCcb(Ccb: PNP_CCB, ListEntry: *mut LIST_ENTRY) {
    let RootDcbCcb: PNP_ROOT_DCB_FCB;
    // PAGED_CODE();

    RootDcbCcb = Ccb as PNP_ROOT_DCB_FCB;
    if (*Ccb).NodeType == NPFS_NTC_CCB {
        RemoveEntryList(addr_of_mut!((*Ccb).CcbEntry));
        (*(*Ccb).Fcb).CurrentInstances -= 1;

        NpDeleteEventTableEntry(
            addr_of_mut!((*NpVcb).EventTable),
            (*(*Ccb).NonPagedCcb).EventBuffer[FILE_PIPE_CLIENT_END as usize] as PVOID,
        );
        NpDeleteEventTableEntry(
            addr_of_mut!((*NpVcb).EventTable),
            (*(*Ccb).NonPagedCcb).EventBuffer[FILE_PIPE_SERVER_END as usize] as PVOID,
        );
        NpUninitializeDataQueue(addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]));
        NpUninitializeDataQueue(addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]));
        NpCheckForNotify((*(*Ccb).Fcb).ParentDcb as PNP_DCB, FALSE, ListEntry);
        ExDeleteResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
        NpUninitializeSecurity(Ccb);
        if !(*Ccb).ClientSession.is_null() {
            ExFreePool((*Ccb).ClientSession);
            (*Ccb).ClientSession = null_mut();
        }
        ExFreePool((*Ccb).NonPagedCcb as PVOID);
    } else if (*RootDcbCcb).NodeType == NPFS_NTC_ROOT_DCB_CCB && !(*RootDcbCcb).Unknown.is_null() {
        ExFreePool((*RootDcbCcb).Unknown);
    }

    ExFreePool(Ccb as PVOID);
}

pub unsafe extern "system" fn NpInitializeVcb() {
    // PAGED_CODE();

    RtlZeroMemory(NpVcb as PVOID, size_of::<NP_VCB>() as _);

    (*NpVcb).NodeType = NPFS_NTC_VCB;
    RtlInitializeUnicodePrefix(addr_of_mut!((*NpVcb).PrefixTable));
    ExInitializeResourceLite(addr_of_mut!((*NpVcb).Lock));
    RtlInitializeGenericTable(
        addr_of_mut!((*NpVcb).EventTable),
        transmute::<EventTableCompareFn, PRTL_GENERIC_COMPARE_ROUTINE>(NpEventTableCompareRoutine),
        transmute::<EventTableAllocateFn, PRTL_GENERIC_ALLOCATE_ROUTINE>(NpEventTableAllocate),
        transmute::<EventTableDeallocateFn, PRTL_GENERIC_FREE_ROUTINE>(NpEventTableDeallocate),
        null_mut(),
    );
    NpInitializeWaitQueue(addr_of_mut!((*NpVcb).WaitQueue));
}

pub unsafe extern "system" fn NpCreateRootDcbCcb(NewRootCcb: *mut PNP_ROOT_DCB_FCB) -> NTSTATUS {
    let RootCcb: PNP_ROOT_DCB_FCB;
    // PAGED_CODE();

    RootCcb = ExAllocatePoolWithTag(
        PagedPool,
        size_of::<NP_ROOT_DCB_CCB>() as _,
        NPFS_ROOT_DCB_CCB_TAG,
    ) as PNP_ROOT_DCB_FCB;
    if RootCcb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(RootCcb as PVOID, size_of::<NP_ROOT_DCB_CCB>() as _);
    (*RootCcb).NodeType = NPFS_NTC_ROOT_DCB_CCB;
    *NewRootCcb = RootCcb;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCreateRootDcb() -> NTSTATUS {
    let Dcb: PNP_DCB;
    // PAGED_CODE();

    if !(*NpVcb).RootDcb.is_null() {
        // NpBugCheck(0, 0, 0);  __LINE__ == 172
        KeBugCheckEx(
            NPFS_FILE_SYSTEM,
            ((NPFS_BUGCHECK_STRUCSUP << 16) | 172) as _,
            0,
            0,
            0,
        );
    }

    (*NpVcb).RootDcb =
        ExAllocatePoolWithTag(PagedPool, size_of::<NP_DCB>() as _, NPFS_DCB_TAG) as PNP_DCB;
    if (*NpVcb).RootDcb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Dcb = (*NpVcb).RootDcb;
    RtlZeroMemory(Dcb as PVOID, size_of::<NP_DCB>() as _);
    (*Dcb).NodeType = NPFS_NTC_ROOT_DCB;

    InitializeListHead(addr_of_mut!((*Dcb).DcbEntry));
    InitializeListHead(addr_of_mut!((*Dcb).NotifyList));
    InitializeListHead(addr_of_mut!((*Dcb).NotifyList2));
    InitializeListHead(addr_of_mut!((*Dcb).FcbList));

    (*Dcb).FullName.Buffer = addr_of_mut!(NpRootDCBName) as *mut u16;
    // sizeof(NpRootDCBName) == 2 WCHARs == 4 bytes; - sizeof(UNICODE_NULL) == 2.
    (*Dcb).FullName.Length = (size_of::<[u16; 2]>() as USHORT) - SIZEOF_UNICODE_NULL;
    (*Dcb).FullName.MaximumLength = size_of::<[u16; 2]>() as USHORT;

    (*Dcb).ShortName.Length = (*Dcb).FullName.Length;
    (*Dcb).ShortName.MaximumLength = (*Dcb).FullName.MaximumLength;
    (*Dcb).ShortName.Buffer = (*Dcb).FullName.Buffer;

    if RtlInsertUnicodePrefix(
        addr_of_mut!((*NpVcb).PrefixTable),
        addr_of_mut!((*Dcb).FullName),
        addr_of_mut!((*Dcb).PrefixTableEntry),
    ) == FALSE
    {
        // NpBugCheck(0, 0, 0);  __LINE__ == 202
        KeBugCheckEx(
            NPFS_FILE_SYSTEM,
            ((NPFS_BUGCHECK_STRUCSUP << 16) | 202) as _,
            0,
            0,
            0,
        );
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCreateFcb(
    Dcb: PNP_DCB,
    PipeName: *mut UNICODE_STRING,
    MaximumInstances: ULONG,
    Timeout: LARGE_INTEGER,
    NamedPipeConfiguration: USHORT,
    NamedPipeType: USHORT,
    NewFcb: *mut PNP_FCB,
) -> NTSTATUS {
    let Fcb: PNP_FCB;
    let mut RootPipe: BOOLEAN;
    let NameBuffer: *mut u16;
    let mut Length: USHORT;
    let mut MaximumLength: USHORT;
    // PAGED_CODE();

    Length = (*PipeName).Length;
    MaximumLength = Length + SIZEOF_UNICODE_NULL;

    if (Length < SIZEOF_WCHAR) || (MaximumLength < Length) {
        return STATUS_INVALID_PARAMETER;
    }

    RootPipe = FALSE;
    if *(*PipeName).Buffer.offset(0) != OBJ_NAME_PATH_SEPARATOR {
        Length += SIZEOF_OBJ_NAME_PATH_SEPARATOR;
        MaximumLength += SIZEOF_OBJ_NAME_PATH_SEPARATOR;
        RootPipe = TRUE;
        if MaximumLength < SIZEOF_WCHAR {
            return STATUS_INVALID_PARAMETER;
        }
    }

    Fcb = ExAllocatePoolWithTag(PagedPool, size_of::<NP_FCB>() as _, NPFS_FCB_TAG) as PNP_FCB;
    if Fcb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(Fcb as PVOID, size_of::<NP_FCB>() as _);
    (*Fcb).MaximumInstances = MaximumInstances;
    (*Fcb).Timeout = Timeout;
    (*Fcb).NodeType = NPFS_NTC_FCB;
    (*Fcb).ParentDcb = Dcb as PVOID;
    InitializeListHead(addr_of_mut!((*Fcb).CcbList));

    NameBuffer =
        ExAllocatePoolWithTag(PagedPool, MaximumLength as _, NPFS_NAME_BLOCK_TAG) as *mut u16;
    if NameBuffer.is_null() {
        ExFreePool(Fcb as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InsertTailList(addr_of_mut!((*Dcb).FcbList), addr_of_mut!((*Fcb).DcbEntry));

    if RootPipe != FALSE {
        *NameBuffer.offset(0) = OBJ_NAME_PATH_SEPARATOR;
        RtlCopyMemory(
            NameBuffer.offset(1) as PVOID,
            (*PipeName).Buffer as PCVOID,
            (*PipeName).Length as _,
        );
    } else {
        RtlCopyMemory(
            NameBuffer as PVOID,
            (*PipeName).Buffer as PCVOID,
            (*PipeName).Length as _,
        );
    }

    *NameBuffer.offset((Length / SIZEOF_WCHAR) as isize) = UNICODE_NULL;

    (*Fcb).FullName.Length = Length;
    (*Fcb).FullName.MaximumLength = MaximumLength;
    (*Fcb).FullName.Buffer = NameBuffer;

    (*Fcb).ShortName.MaximumLength = Length;
    (*Fcb).ShortName.Length = Length - SIZEOF_OBJ_NAME_PATH_SEPARATOR;
    (*Fcb).ShortName.Buffer = NameBuffer.offset(1);

    if RtlInsertUnicodePrefix(
        addr_of_mut!((*NpVcb).PrefixTable),
        addr_of_mut!((*Fcb).FullName),
        addr_of_mut!((*Fcb).PrefixTableEntry),
    ) == FALSE
    {
        // NpBugCheck(0, 0, 0);  __LINE__ == 293
        KeBugCheckEx(
            NPFS_FILE_SYSTEM,
            ((NPFS_BUGCHECK_STRUCSUP << 16) | 293) as _,
            0,
            0,
            0,
        );
    }

    (*Fcb).NamedPipeConfiguration = NamedPipeConfiguration;
    (*Fcb).NamedPipeType = NamedPipeType;
    *NewFcb = Fcb;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpCreateCcb(
    Fcb: PNP_FCB,
    FileObject: PFILE_OBJECT,
    State: UCHAR,
    ReadMode: UCHAR,
    CompletionMode: UCHAR,
    InQuota: ULONG,
    OutQuota: ULONG,
    NewCcb: *mut PNP_CCB,
) -> NTSTATUS {
    let Ccb: PNP_CCB;
    let CcbNonPaged: PNP_NONPAGED_CCB;
    let mut Status: NTSTATUS;
    // PAGED_CODE();

    Ccb = ExAllocatePoolWithTag(PagedPool, size_of::<NP_CCB>() as _, NPFS_CCB_TAG) as PNP_CCB;
    if Ccb.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    CcbNonPaged = ExAllocatePoolWithTag(
        NonPagedPool,
        size_of::<NP_NONPAGED_CCB>() as _,
        NPFS_CCB_TAG,
    ) as PNP_NONPAGED_CCB;
    if CcbNonPaged.is_null() {
        ExFreePool(Ccb as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(CcbNonPaged as PVOID, size_of::<NP_NONPAGED_CCB>() as _);
    (*CcbNonPaged).NodeType = NPFS_NTC_NONPAGED_CCB;

    RtlZeroMemory(Ccb as PVOID, size_of::<NP_CCB>() as _);
    (*Ccb).NodeType = NPFS_NTC_CCB;
    (*Ccb).NonPagedCcb = CcbNonPaged;
    (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = FileObject;
    (*Ccb).Fcb = Fcb;
    (*Ccb).NamedPipeState = State;
    (*Ccb).ReadMode[FILE_PIPE_SERVER_END as usize] = ReadMode;
    (*Ccb).CompletionMode[FILE_PIPE_SERVER_END as usize] = CompletionMode;

    Status = NpInitializeDataQueue(
        addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]),
        InQuota,
    );
    if !NT_SUCCESS(Status) {
        ExFreePool(CcbNonPaged as PVOID);
        ExFreePool(Ccb as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = NpInitializeDataQueue(
        addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_OUTBOUND as usize]),
        OutQuota,
    );
    if !NT_SUCCESS(Status) {
        NpUninitializeDataQueue(addr_of_mut!((*Ccb).DataQueue[FILE_PIPE_INBOUND as usize]));
        ExFreePool(CcbNonPaged as PVOID);
        ExFreePool(Ccb as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InsertTailList(addr_of_mut!((*Fcb).CcbList), addr_of_mut!((*Ccb).CcbEntry));

    (*Fcb).CurrentInstances += 1;
    (*Fcb).ServerOpenCount += 1;
    InitializeListHead(addr_of_mut!((*Ccb).IrpList));
    ExInitializeResourceLite(addr_of_mut!((*(*Ccb).NonPagedCcb).Lock));
    *NewCcb = Ccb;
    STATUS_SUCCESS
}
