//! Executive subsystem types and declarations.

use crate::base::*;
use crate::mm::POOL_TYPE;

// ============================================================================
// EX_POOL_PRIORITY
// ============================================================================

pub type EX_POOL_PRIORITY = ULONG;

pub const LowPoolPriority: EX_POOL_PRIORITY = 0;
pub const LowPoolPrioritySpecialPoolOverrun: EX_POOL_PRIORITY = 8;
pub const NormalPoolPriority: EX_POOL_PRIORITY = 16;
pub const NormalPoolPrioritySpecialPoolOverrun: EX_POOL_PRIORITY = 24;
pub const HighPoolPriority: EX_POOL_PRIORITY = 32;
pub const HighPoolPrioritySpecialPoolOverrun: EX_POOL_PRIORITY = 40;

// ============================================================================
// SYSTEM_POOLTAG_INFORMATION — used by ExGetPoolTagInfo
// ============================================================================

#[repr(C)]
pub struct SYSTEM_POOLTAG {
    pub Tag: [u8; 4],
    pub PagedAllocs: ULONG,
    pub PagedFrees: ULONG,
    pub PagedUsed: SIZE_T,
    pub NonPagedAllocs: ULONG,
    pub NonPagedFrees: ULONG,
    pub NonPagedUsed: SIZE_T,
}

#[repr(C)]
pub struct SYSTEM_POOLTAG_INFORMATION {
    pub Count: ULONG,
    pub TagInfo: [SYSTEM_POOLTAG; 1], // variable-length array
}

pub type PSYSTEM_POOLTAG_INFORMATION = *mut SYSTEM_POOLTAG_INFORMATION;

// ============================================================================
// Extern declarations
// ============================================================================

extern "system" {
    /// ExRaiseStatus — forwards to RtlRaiseStatus in the kernel.
    #[link_name = "RtlRaiseStatus"]
    pub fn ExRaiseStatus(Status: NTSTATUS) -> !;

    pub fn ExAllocatePoolWithTag(
        PoolType: POOL_TYPE,
        NumberOfBytes: SIZE_T,
        Tag: ULONG,
    ) -> PVOID;

    pub fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}
