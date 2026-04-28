//! kdbg-private types — translated from ntoskrnl/kdbg/kdb.h.
//!
//! These types are local to the kernel debugger subsystem and are not
//! exported via ntkrnl-sys.

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{CONTEXT, PEPROCESS};

pub type KDB_KTRAP_FRAME = CONTEXT;
pub type PKDB_KTRAP_FRAME = *mut CONTEXT;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KDB_BREAKPOINT_TYPE {
    KdbBreakPointNone = 0,
    KdbBreakPointSoftware = 1,
    KdbBreakPointHardware = 2,
    KdbBreakPointTemporary = 3,
}
pub use KDB_BREAKPOINT_TYPE::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KDB_ACCESS_TYPE {
    KdbAccessRead = 0,
    KdbAccessWrite = 1,
    KdbAccessReadWrite = 2,
    KdbAccessExec = 3,
}
pub use KDB_ACCESS_TYPE::*;

/// Hardware breakpoint sub-fields. C uses bitfields; we use full bytes
/// since this struct is internal to ntoskrnl and never marshalled.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct KDB_BREAKPOINT_HW {
    pub DebugReg: UCHAR,
    pub Size: UCHAR,
    pub AccessType: KDB_ACCESS_TYPE,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union KDB_BREAKPOINT_DATA {
    pub SavedInstruction: UCHAR,
    pub Hw: KDB_BREAKPOINT_HW,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct KDB_BREAKPOINT {
    pub Type: KDB_BREAKPOINT_TYPE,
    pub Enabled: BOOLEAN,
    pub Address: ULONG_PTR,
    pub Global: BOOLEAN,
    pub Process: PEPROCESS,
    pub ConditionExpression: PCHAR,
    pub Condition: PVOID,
    pub Data: KDB_BREAKPOINT_DATA,
}
pub type PKDB_BREAKPOINT = *mut KDB_BREAKPOINT;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KDB_ENTER_CONDITION {
    KdbDoNotEnter = 0,
    KdbEnterAlways = 1,
    KdbEnterFromKmode = 2,
    KdbEnterFromUmode = 3,
}
pub use KDB_ENTER_CONDITION::*;

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KD_CONTINUE_TYPE {
    kdContinue = 0,
    kdDoNotHandleException = 1,
    kdHandleException = 2,
}
pub use KD_CONTINUE_TYPE::*;
