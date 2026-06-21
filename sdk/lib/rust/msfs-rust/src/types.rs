//! types.rs — the msfs.h driver-private node structs. All `#[repr(C)]` with the exact C
//! field order. These are driver-private: the kernel only ever sees them through opaque
//! pointers (DeviceExtension, FsContext/FsContext2). The embedded kernel objects
//! (FSRTL_COMMON_FCB_HEADER, KMUTEX, KSPIN_LOCK, KTIMER, KDPC, KEVENT, IO_CSQ,
//! IO_CSQ_IRP_CONTEXT, UNICODE_STRING, LIST_ENTRY, LARGE_INTEGER) use the real ntkrnl-sys
//! ddk layouts so the kernel reads/writes them correctly.
#![allow(non_snake_case, non_camel_case_types, dead_code)]

use ntkrnl_sys::base::{UCHAR, ULONG};
use ntkrnl_sys::ddk::{
    FSRTL_COMMON_FCB_HEADER, IO_CSQ, IO_CSQ_IRP_CONTEXT, KDPC, KEVENT, KMUTEX, KSPIN_LOCK,
    KTIMER, LARGE_INTEGER, LIST_ENTRY, PIO_CSQ, UNICODE_STRING,
};

pub type PMSFS_DEVICE_EXTENSION = *mut MSFS_DEVICE_EXTENSION;
pub type PMSFS_FCB = *mut MSFS_FCB;
pub type PMSFS_CCB = *mut MSFS_CCB;
pub type PMSFS_MESSAGE = *mut MSFS_MESSAGE;
pub type PMSFS_DPC_CTX = *mut MSFS_DPC_CTX;

#[repr(C)]
pub struct MSFS_DEVICE_EXTENSION {
    pub FcbListHead: LIST_ENTRY,
    pub FcbListLock: KMUTEX,
}

#[repr(C)]
pub struct MSFS_FCB {
    pub RFCB: FSRTL_COMMON_FCB_HEADER,
    pub Name: UNICODE_STRING,
    pub FcbListEntry: LIST_ENTRY,
    pub CcbListLock: KSPIN_LOCK,
    pub CcbListHead: LIST_ENTRY,
    pub ServerCcb: PMSFS_CCB,
    pub ReferenceCount: ULONG,
    pub TimeOut: LARGE_INTEGER,
    pub MaxMessageSize: ULONG,
    pub MessageCount: ULONG,
    pub MessageListLock: KSPIN_LOCK,
    pub MessageListHead: LIST_ENTRY,
    pub CancelSafeQueue: IO_CSQ,
    pub QueueLock: KSPIN_LOCK,
    pub PendingIrpQueue: LIST_ENTRY,
}

#[repr(C)]
pub struct MSFS_DPC_CTX {
    pub Timer: KTIMER,
    pub Dpc: KDPC,
    pub Csq: PIO_CSQ,
    pub Event: KEVENT,
    pub CsqContext: IO_CSQ_IRP_CONTEXT,
}

#[repr(C)]
pub struct MSFS_CCB {
    pub CcbListEntry: LIST_ENTRY,
    pub Fcb: PMSFS_FCB,
}

#[repr(C)]
pub struct MSFS_MESSAGE {
    pub MessageListEntry: LIST_ENTRY,
    pub Size: ULONG,
    pub Buffer: [UCHAR; 1],
}
