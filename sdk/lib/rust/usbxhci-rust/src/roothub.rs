//! The root hub usbport emulates on top of the controller's ports.
//!
//! usbport and usbhub speak USB 2.0 hub semantics (wPortStatus / wPortChange), so
//! the root hub exposes the controller's USB2-protocol ports only: keyboards,
//! mice and the Pico bridge are low/full/high-speed devices and always attach
//! there. SuperSpeed ports (and SuperSpeed devices, which attach only to them) are
//! a later milestone; see docs/engineering/usbxhci-design.md.

use crate::controller::{ext_of, XhciExtension};
use crate::hw::*;
use crate::types::*;
use ntkrnl_sys::ddk::{PUSHORT, PVOID, USHORT};

// USB 2.0 hub port status and change bits (USB 2.0 11.24.2.7).
const PS_CONNECTION: u32 = 1 << 0;
const PS_ENABLE: u32 = 1 << 1;
const PS_OVER_CURRENT: u32 = 1 << 3;
const PS_RESET: u32 = 1 << 4;
const PS_POWER: u32 = 1 << 8;
const PS_LOW_SPEED: u32 = 1 << 9;
const PS_HIGH_SPEED: u32 = 1 << 10;
const PC_CONNECTION: u32 = 1 << 16;
const PC_ENABLE: u32 = 1 << 17;
const PC_OVER_CURRENT: u32 = 1 << 19;
const PC_RESET: u32 = 1 << 20;

/// PORTSC offset for usbport's root port `port` (1-based), or None if out of range.
fn portsc(x: &XhciExtension, port: USHORT) -> Option<usize> {
    let port = port as u32;
    if port == 0 || port > x.usb2_count {
        return None;
    }
    let hw = x.usb2_first + port - 1;
    Some(PORTSC_BASE + 0x10 * (hw as usize - 1))
}

/// The xHCI port number behind usbport's root port `port`.
pub fn hw_port(x: &XhciExtension, port: USHORT) -> u32 {
    x.usb2_first + port as u32 - 1
}

/// Read-modify-write PORTSC: keep the preserved state, add `set`.
unsafe fn portsc_write(x: &XhciExtension, port: USHORT, set: u32) -> MPSTATUS {
    match portsc(x, port) {
        Some(off) => {
            let v = rd32(x.op, off);
            wr32(x.op, off, (v & PORTSC_PRESERVE) | set);
            MP_STATUS_SUCCESS
        }
        None => MP_STATUS_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_GetRootHubData(xhciExtension: PVOID, rootHubData: PVOID) {
    let x = ext_of(xhciExtension);
    let d = &mut *(rootHubData as *mut USBPORT_ROOT_HUB_DATA);
    d.NumberOfPorts = x.usb2_count;
    // wHubCharacteristics: individual port power switching (01b), individual
    // over-current reporting (bit 3 = 1).
    core::ptr::write_unaligned(core::ptr::addr_of_mut!(d.HubCharacteristics) as *mut u16, 0x0009);
    d.PowerOnToPowerGood = 10; // in 2 ms units
    d.HubControlCurrent = 0;
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_GetStatus(_xhciExtension: PVOID, Status: PUSHORT) -> MPSTATUS {
    *Status = 1; // self-powered
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_GetPortStatus(
    xhciExtension: PVOID,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    let Some(off) = portsc(x, Port) else {
        return MP_STATUS_FAILURE;
    };
    let v = rd32(x.op, off);
    let mut s = 0u32;
    if v & PORTSC_CCS != 0 {
        s |= PS_CONNECTION;
    }
    if v & PORTSC_PED != 0 {
        s |= PS_ENABLE;
    }
    if v & PORTSC_OCA != 0 {
        s |= PS_OVER_CURRENT;
    }
    if v & PORTSC_PR != 0 {
        s |= PS_RESET;
    }
    if v & PORTSC_PP != 0 {
        s |= PS_POWER;
    }
    if v & PORTSC_CCS != 0 {
        match (v & PORTSC_SPEED_MASK) >> PORTSC_SPEED_SHIFT {
            SPEED_LOW => s |= PS_LOW_SPEED,
            SPEED_HIGH => s |= PS_HIGH_SPEED,
            _ => {}
        }
    }
    if v & PORTSC_CSC != 0 {
        s |= PC_CONNECTION;
    }
    if v & PORTSC_PEC != 0 {
        s |= PC_ENABLE;
    }
    if v & PORTSC_OCC != 0 {
        s |= PC_OVER_CURRENT;
    }
    if v & PORTSC_PRC != 0 {
        s |= PC_RESET;
    }
    core::ptr::write_unaligned(PortStatus as *mut u32, s);
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_GetHubStatus(
    _xhciExtension: PVOID,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
) -> MPSTATUS {
    core::ptr::write_unaligned(HubStatus as *mut u32, 0);
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_SetFeaturePortReset(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    crate::xlog!("port %u (hw %u): reset", Port as u32, hw_port(x, Port));
    portsc_write(x, Port, PORTSC_PR)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_SetFeaturePortPower(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    portsc_write(ext_of(xhciExtension), Port, PORTSC_PP)
}

/// USB2 ports are enabled by a reset, never by software directly (4.19.1.1).
#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_SetFeaturePortEnable(_xhciExtension: PVOID, _Port: USHORT) -> MPSTATUS {
    MP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_SetFeaturePortSuspend(_xhciExtension: PVOID, _Port: USHORT) -> MPSTATUS {
    // USB2 resume requires a timed Resume -> U0 transition. Until the paired
    // power-management path exists, don't put a working keyboard into U3.
    MP_STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortEnable(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    // PED is RW1CS: writing 1 disables the port.
    portsc_write(ext_of(xhciExtension), Port, PORTSC_PED)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortPower(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    let x = ext_of(xhciExtension);
    match portsc(x, Port) {
        Some(off) => {
            let v = rd32(x.op, off);
            wr32(x.op, off, v & PORTSC_PRESERVE & !PORTSC_PP);
            MP_STATUS_SUCCESS
        }
        None => MP_STATUS_FAILURE,
    }
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortSuspend(_xhciExtension: PVOID, _Port: USHORT) -> MPSTATUS {
    MP_STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortEnableChange(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    portsc_write(ext_of(xhciExtension), Port, PORTSC_PEC)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortConnectChange(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    portsc_write(ext_of(xhciExtension), Port, PORTSC_CSC)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortResetChange(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    // A reset also raises the other "noise" changes on some controllers; clear
    // warm-reset and link-state changes with it so they cannot pile up unseen.
    portsc_write(ext_of(xhciExtension), Port, PORTSC_PRC | PORTSC_WRC | PORTSC_PLC)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortSuspendChange(xhciExtension: PVOID, Port: USHORT) -> MPSTATUS {
    portsc_write(ext_of(xhciExtension), Port, PORTSC_PLC)
}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ClearFeaturePortOvercurrentChange(
    xhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    portsc_write(ext_of(xhciExtension), Port, PORTSC_OCC)
}

/// Port changes arrive as Port Status Change events on the event ring, which the
/// root hub cannot mask separately; usbport's disable/enable pair is a no-op.
#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_DisableIrq(_xhciExtension: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_EnableIrq(_xhciExtension: PVOID) {}

/// Chirp is the EHCI companion-controller handshake; xHCI has no companions.
#[no_mangle]
pub unsafe extern "system" fn XHCI_RH_ChirpRootPort(_xhciExtension: PVOID, _Port: USHORT) -> MPSTATUS {
    MP_STATUS_SUCCESS
}
