//! types.rs — the npfs.h driver-private node graph, `#[repr(C)]` with exact C field order.
//!
//! C uses anonymous struct members (`NP_CB_HEADER;` / `NP_CB_FOOTER;`) to embed the shared
//! header/footer into NP_DCB and NP_FCB; the C code then accesses the header/footer fields
//! *flattened* (e.g. `Dcb->NodeType`, `Fcb->FullName`). We replicate that by inlining the
//! header and footer fields directly into NP_DCB / NP_FCB (same layout, direct field access).
//! The arch-conditional Pad fields (#ifndef/#ifdef _WIN64) keep PrefixTableEntry at the same
//! offset in both (the npfs.h C_ASSERT) — kept via cfg(target_arch).
#![allow(non_snake_case, non_camel_case_types, dead_code)]

use ntkrnl_sys::base::{PVOID, UCHAR, ULONG, USHORT};
use ntkrnl_sys::ddk::{
    ERESOURCE, KDPC, KTIMER, LARGE_INTEGER, LIST_ENTRY, PEPROCESS, PFILE_OBJECT, PIRP,
    PKEVENT, PSECURITY_CLIENT_CONTEXT, PSECURITY_DESCRIPTOR, RTL_GENERIC_TABLE,
    SECURITY_QUALITY_OF_SERVICE, KSPIN_LOCK, UNICODE_PREFIX_TABLE,
    UNICODE_PREFIX_TABLE_ENTRY, UNICODE_STRING,
};

pub type NODE_TYPE_CODE = USHORT;
pub type PNODE_TYPE_CODE = *mut USHORT;

pub type PNP_DATA_QUEUE = *mut NP_DATA_QUEUE;
pub type PNP_DATA_QUEUE_ENTRY = *mut NP_DATA_QUEUE_ENTRY;
pub type PNP_WAIT_QUEUE = *mut NP_WAIT_QUEUE;
pub type PNP_WAIT_QUEUE_ENTRY = *mut NP_WAIT_QUEUE_ENTRY;
pub type PNP_EVENT_BUFFER = *mut NP_EVENT_BUFFER;
pub type PNP_ROOT_DCB_FCB = *mut NP_ROOT_DCB_CCB;
pub type PNP_CB_HEADER = *mut NP_CB_HEADER;
pub type PNP_DCB = *mut NP_DCB;
pub type PNP_FCB = *mut NP_FCB;
pub type PNP_NONPAGED_CCB = *mut NP_NONPAGED_CCB;
pub type PNP_CCB = *mut NP_CCB;
pub type PNP_VCB = *mut NP_VCB;
pub type PNPFS_ALIAS = *mut NPFS_ALIAS;
pub type PNPFS_QUERY_VALUE_CONTEXT = *mut NPFS_QUERY_VALUE_CONTEXT;
pub type PNP_CLIENT_PROCESS = *mut NP_CLIENT_PROCESS;

// NP_DATA_QUEUE_STATE
pub const ReadEntries: ULONG = 0;
pub const WriteEntries: ULONG = 1;
pub const Empty: ULONG = 2;

// NP_DATA_QUEUE_ENTRY_TYPE
pub const Buffered: ULONG = 0;
pub const Unbuffered: ULONG = 1;

#[repr(C)]
pub struct NP_DATA_QUEUE {
    pub Queue: LIST_ENTRY,
    pub QueueState: ULONG,
    pub BytesInQueue: ULONG,
    pub EntriesInQueue: ULONG,
    pub QuotaUsed: ULONG,
    pub ByteOffset: ULONG,
    pub Quota: ULONG,
}

#[repr(C)]
pub struct NP_DATA_QUEUE_ENTRY {
    pub QueueEntry: LIST_ENTRY,
    pub DataEntryType: ULONG,
    pub Irp: PIRP,
    pub QuotaInEntry: ULONG,
    pub ClientSecurityContext: PSECURITY_CLIENT_CONTEXT,
    pub DataSize: ULONG,
}

#[repr(C)]
pub struct NP_WAIT_QUEUE {
    pub WaitList: LIST_ENTRY,
    pub WaitLock: KSPIN_LOCK,
}

#[repr(C)]
pub struct NP_WAIT_QUEUE_ENTRY {
    pub Irp: PIRP,
    pub Dpc: KDPC,
    pub Timer: KTIMER,
    pub WaitQueue: PNP_WAIT_QUEUE,
    pub AliasName: UNICODE_STRING,
    pub FileObject: PFILE_OBJECT,
}

#[repr(C)]
pub struct NP_EVENT_BUFFER {
    pub Event: PKEVENT,
}

#[repr(C)]
pub struct NP_ROOT_DCB_CCB {
    pub NodeType: NODE_TYPE_CODE,
    pub Unknown: PVOID,
    pub Unknown2: ULONG,
}

/// The shared header (used standalone where the C casts to PNP_CB_HEADER). NP_DCB/NP_FCB
/// inline these fields directly (see below) so their layout matches a leading NP_CB_HEADER.
#[repr(C)]
pub struct NP_CB_HEADER {
    pub NodeType: NODE_TYPE_CODE,
    pub DcbEntry: LIST_ENTRY,
    pub ParentDcb: PVOID,
    pub CurrentInstances: ULONG,
    pub ServerOpenCount: ULONG,
    pub SecurityDescriptor: PSECURITY_DESCRIPTOR,
}

/// A Directory Control Block (DCB). Header fields inlined, then DCB-specific data, then the
/// inlined footer. On i386 (!_WIN64) a ULONG Pad precedes the footer.
#[repr(C)]
pub struct NP_DCB {
    // --- NP_CB_HEADER (inlined) ---
    pub NodeType: NODE_TYPE_CODE,
    pub DcbEntry: LIST_ENTRY,
    pub ParentDcb: PVOID,
    pub CurrentInstances: ULONG,
    pub ServerOpenCount: ULONG,
    pub SecurityDescriptor: PSECURITY_DESCRIPTOR,
    // --- DCB-specific ---
    pub NotifyList: LIST_ENTRY,
    pub NotifyList2: LIST_ENTRY,
    pub FcbList: LIST_ENTRY,
    #[cfg(target_arch = "x86")]
    pub Pad: ULONG,
    // --- NP_CB_FOOTER (inlined) ---
    pub FullName: UNICODE_STRING,
    pub ShortName: UNICODE_STRING,
    pub PrefixTableEntry: UNICODE_PREFIX_TABLE_ENTRY,
}

/// A File Control Block (FCB). Header fields inlined, then FCB-specific data, then the inlined
/// footer. On _WIN64 (amd64/arm64) a PVOID Pad[2] precedes the footer.
#[repr(C)]
pub struct NP_FCB {
    // --- NP_CB_HEADER (inlined) ---
    pub NodeType: NODE_TYPE_CODE,
    pub DcbEntry: LIST_ENTRY,
    pub ParentDcb: PVOID,
    pub CurrentInstances: ULONG,
    pub ServerOpenCount: ULONG,
    pub SecurityDescriptor: PSECURITY_DESCRIPTOR,
    // --- FCB-specific ---
    pub MaximumInstances: ULONG,
    pub NamedPipeConfiguration: USHORT,
    pub NamedPipeType: USHORT,
    pub Timeout: LARGE_INTEGER,
    pub CcbList: LIST_ENTRY,
    #[cfg(not(target_arch = "x86"))]
    pub Pad: [PVOID; 2],
    // --- NP_CB_FOOTER (inlined) ---
    pub FullName: UNICODE_STRING,
    pub ShortName: UNICODE_STRING,
    pub PrefixTableEntry: UNICODE_PREFIX_TABLE_ENTRY,
}

#[repr(C)]
pub struct NP_NONPAGED_CCB {
    pub NodeType: NODE_TYPE_CODE,
    pub EventBuffer: [PNP_EVENT_BUFFER; 2],
    pub Lock: ERESOURCE,
}

#[repr(C)]
pub struct NP_CCB {
    pub NodeType: NODE_TYPE_CODE,
    pub NamedPipeState: UCHAR,
    pub ReadMode: [UCHAR; 2],
    pub CompletionMode: [UCHAR; 2],
    pub ClientQos: SECURITY_QUALITY_OF_SERVICE,
    pub CcbEntry: LIST_ENTRY,
    pub Fcb: PNP_FCB,
    pub FileObject: [PFILE_OBJECT; 2],
    pub Process: PEPROCESS,
    pub ClientSession: PVOID,
    pub NonPagedCcb: PNP_NONPAGED_CCB,
    pub DataQueue: [NP_DATA_QUEUE; 2],
    pub ClientContext: PSECURITY_CLIENT_CONTEXT,
    pub IrpList: LIST_ENTRY,
}

#[repr(C)]
pub struct NP_VCB {
    pub NodeType: NODE_TYPE_CODE,
    pub ReferenceCount: ULONG,
    pub RootDcb: PNP_DCB,
    pub PrefixTable: UNICODE_PREFIX_TABLE,
    pub Lock: ERESOURCE,
    pub EventTable: RTL_GENERIC_TABLE,
    pub WaitQueue: NP_WAIT_QUEUE,
}

#[repr(C)]
pub struct NPFS_ALIAS {
    pub Next: *mut NPFS_ALIAS,
    pub TargetName: *mut UNICODE_STRING,
    pub Name: UNICODE_STRING,
}

#[repr(C)]
pub struct NPFS_QUERY_VALUE_CONTEXT {
    pub SizeOnly: ntkrnl_sys::base::BOOLEAN,
    pub FullSize: ntkrnl_sys::base::SIZE_T,
    pub NumberOfAliases: ULONG,
    pub NumberOfEntries: ULONG,
    pub CurrentAlias: PNPFS_ALIAS,
    pub CurrentTargetName: *mut UNICODE_STRING,
    pub CurrentStringPointer: *mut u16,
}

#[repr(C)]
pub struct NP_CLIENT_PROCESS {
    pub Unknown: PVOID,
    pub Process: PVOID,
    pub DataLength: USHORT,
    pub Buffer: [u16; 17],
}
