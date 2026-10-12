use crate::prelude::*;
use ntkrnl_sys::ddk::{KAFFINITY, KeQueryActiveProcessors, KeSetTimerEx, LARGE_INTEGER, PKAFFINITY, PKTIMER, RtlNumberOfSetBits};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations in this preamble. It contains only annotation-macro fragments (`NTKRNLVISTAAPI`, `_IRQL_requires_max_(DISPATCH_LEVEL)`) and the bare type names `ULONG` and `USHORT`, with no typedefs, `#define` constants, enums, or tables to emit.

#[no_mangle]
pub unsafe extern "system" fn KeQueryActiveProcessorCount(ActiveProcessors: PKAFFINITY) -> ULONG {
    let mut Bitmap: RTL_BITMAP = RTL_BITMAP {
        SizeOfBitMap: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ActiveMap: KAFFINITY = KeQueryActiveProcessors();

    if !ActiveProcessors.is_null() {
        *ActiveProcessors = ActiveMap;
    }

    RtlInitializeBitMap(
        core::ptr::addr_of_mut!(Bitmap),
        core::ptr::addr_of_mut!(ActiveMap) as PULONG,
        (core::mem::size_of::<KAFFINITY>() * 8) as ULONG,
    );
    return RtlNumberOfSetBits(core::ptr::addr_of_mut!(Bitmap));
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryHighestNodeNumber() -> USHORT {
    return 0 as USHORT;
}

#[no_mangle]
pub unsafe extern "system" fn KeGetCurrentNodeNumber() -> USHORT {
    return 0 as USHORT;
}
#[no_mangle]
pub unsafe extern "system" fn KeSetCoalescableTimer(
    Timer: PKTIMER,
    DueTime: LARGE_INTEGER,
    Period: ULONG,
    TolerableDelay: ULONG,
    Dpc: PKDPC,
) -> BOOLEAN {
    return KeSetTimerEx(Timer, DueTime, Period as LONG, Dpc);
}
