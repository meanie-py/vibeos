//! Hand-ported FORCEINLINE helpers from `drivers/base/kdgdb/kdgdb.h`.
//!
//! These live only in the header (no `.c` file), so rustconv never translated
//! them. Ported faithfully here.

use ntkrnl_sys::ddk::{DBGKD_ANY_WAIT_STATE_CHANGE, DBGKD_MANIPULATE_STATE64, HANDLE};
use ntkrnl_sys::base::{ULONG, ULONG_PTR};

// C `UINT_PTR` is pointer-sized; model it with the pointer-sized `ULONG_PTR`
// (usize) so the kdgdb id arithmetic is one consistent integer type.

/* GDB doesn't like pid - tid 0, so +1 them */

#[inline]
pub unsafe fn gdb_tid_to_handle(Tid: ULONG_PTR) -> HANDLE {
    (Tid.wrapping_sub(1)) as HANDLE
}

/* #define gdb_pid_to_handle gdb_tid_to_handle */
#[inline]
pub unsafe fn gdb_pid_to_handle(Pid: ULONG_PTR) -> HANDLE {
    gdb_tid_to_handle(Pid)
}

#[inline]
pub unsafe fn handle_to_gdb_tid(Handle: HANDLE) -> ULONG_PTR {
    (Handle as ULONG_PTR).wrapping_add(1)
}

/* #define handle_to_gdb_pid handle_to_gdb_tid */
#[inline]
pub unsafe fn handle_to_gdb_pid(Handle: HANDLE) -> ULONG_PTR {
    handle_to_gdb_tid(Handle)
}

#[inline]
pub unsafe fn InitManipulateFromStateChange(
    ApiNumber: ULONG,
    StateChange: *const DBGKD_ANY_WAIT_STATE_CHANGE,
    Manipulate: *mut DBGKD_MANIPULATE_STATE64,
) {
    (*Manipulate).ApiNumber = ApiNumber;
    (*Manipulate).Processor = (*StateChange).Processor;
    (*Manipulate).ProcessorLevel = (*StateChange).ProcessorLevel;
}
