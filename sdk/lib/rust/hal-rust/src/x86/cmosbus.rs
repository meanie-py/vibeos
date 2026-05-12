//! Tiny CMOS pseudo-bus handler — adapts the bus-handler interface
//! (Get/SetBusData) to the underlying `HalpGetCmosData` /
//! `HalpSetCmosData` primitives from cmos.rs.
//!
//! Translated from `hal/halx86/legacy/bus/cmosbus.c`.

use ntkrnl_sys::hal_ddk::{PBUS_HANDLER, PVOID, ULONG};
use crate::x86::cmos::{HalpGetCmosData, HalpSetCmosData};

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpcGetCmosData(
    BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    // CMOS reads do not support offsets — anything other than 0 means the
    // caller is using the wrong API.
    if Offset != 0 {
        return 0;
    }
    unsafe { HalpGetCmosData((*BusHandler).BusNumber, SlotNumber, Buffer, Length) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpcSetCmosData(
    BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    SlotNumber: ULONG,
    Buffer: PVOID,
    Offset: ULONG,
    Length: ULONG,
) -> ULONG {
    // CMOS writes do not support offsets.
    if Offset != 0 {
        return 0;
    }
    unsafe { HalpSetCmosData((*BusHandler).BusNumber, SlotNumber, Buffer, Length) }
}
