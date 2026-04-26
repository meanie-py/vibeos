//! Internal pool types not exported through ntkrnl-sys.

use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;

/// POOL_DPC_CONTEXT — used for DPC-based atomic pool tag table snapshots.
#[repr(C)]
pub struct PoolDpcContext {
    pub pool_track_table: PPOOL_TRACKER_TABLE,
    pub pool_track_table_size: SIZE_T,
    pub pool_track_table_expansion: PPOOL_TRACKER_TABLE,
    pub pool_track_table_size_expansion: SIZE_T,
}
