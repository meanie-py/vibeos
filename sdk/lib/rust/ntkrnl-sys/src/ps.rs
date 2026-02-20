//! Process subsystem types and extern declarations.

use crate::base::*;
use crate::mm::POOL_TYPE;

// ============================================================================
// EPROCESS — opaque process object
// ============================================================================

/// EPROCESS is extremely large and architecture-dependent.
/// We only use pointers to it (PEPROCESS).
/// The pool allocator accesses Process->Pcb.Header.Type to verify
/// the object is indeed a process (must equal ProcessObject = 3).
#[repr(C)]
pub struct EPROCESS {
    _opaque: [u8; 0],
}

pub type PEPROCESS = *mut EPROCESS;

/// Read the object type byte from an EPROCESS.
/// EPROCESS starts with KPROCESS, which starts with DISPATCHER_HEADER,
/// which starts with Type (UCHAR at offset 0).
#[inline]
pub unsafe fn EprocessGetType(process: PEPROCESS) -> u8 {
    *(process as *const u8)
}

// ============================================================================
// Extern declarations — Ps functions
// ============================================================================

extern "system" {
    /// PsGetCurrentProcess — macro forwarding to IoGetCurrentProcess in the kernel.
    #[link_name = "IoGetCurrentProcess"]
    pub fn PsGetCurrentProcess() -> PEPROCESS;

    pub fn PsChargeProcessPoolQuota(
        Process: PEPROCESS,
        PoolType: POOL_TYPE,
        Amount: SIZE_T,
    ) -> NTSTATUS;

    pub fn PsReturnPoolQuota(
        Process: PEPROCESS,
        PoolType: POOL_TYPE,
        Amount: SIZE_T,
    );
}
