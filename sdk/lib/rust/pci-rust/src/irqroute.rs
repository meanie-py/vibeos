//! Where a function's INTx pin really lands, from ACPI's `_PRT` (VibeOS; no C
//! original).
//!
//! The Interrupt Line register is the 8259 number the BIOS picked; under an I/O
//! APIC the same pin arrives on whatever input the board wires it to (16-23 on an
//! Intel PCH), which only ACPI knows. acpi.sys offers a private routing interface
//! on the root bridge's PDO (acpi-rust `pciroute.rs`, which owns the layout below;
//! change both together). A root bus FDO asks for it at start; a function on any
//! bus is then resolved by handing acpi.sys its device/function path up to that
//! root, and the answer, a GSI, becomes the interrupt the PDO reports.
//!
//! Without the interface (the 8259 HAL, arm64, no ACPI) nothing changes: the
//! Interrupt Line is used as before.

use core::ptr::addr_of_mut;

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::constants::{STATUS_NOT_SUPPORTED, STATUS_PENDING};
use ntkrnl_sys::driver_ddk::{IRP_MJ_PNP, IRP_MN_QUERY_INTERFACE};
use ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation;

use crate::shared::{PCOMMON_DEVICE_EXTENSION, PFDO_DEVICE_EXTENSION, PPDO_DEVICE_EXTENSION};

/// `{9c5c2f4e-6d0b-4a8e-b0f1-5e7d3a2c8b41}`, acpi-rust's `GUID_VIBEOS_PCI_IRQ_ROUTING`.
static GUID_VIBEOS_PCI_IRQ_ROUTING: GUID = GUID {
    Data1: 0x9c5c_2f4e,
    Data2: 0x6d0b,
    Data3: 0x4a8e,
    Data4: [0xb0, 0xf1, 0x5e, 0x7d, 0x3a, 0x2c, 0x8b, 0x41],
};

#[repr(C)]
struct IrqRoute {
    gsi: ULONG,
    level_sensitive: BOOLEAN,
    active_low: BOOLEAN,
}

type RouteInterrupt =
    unsafe extern "system" fn(Context: PVOID, Path: *const UCHAR, Depth: ULONG, Pin: ULONG, Route: *mut IrqRoute) -> NTSTATUS;

/// acpi-rust's `VIBEOS_PCI_IRQ_ROUTING_INTERFACE`: an `INTERFACE` header and one routine.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IrqRoutingInterface {
    size: USHORT,
    version: USHORT,
    context: PVOID,
    reference: Option<unsafe extern "system" fn(PVOID)>,
    dereference: Option<unsafe extern "system" fn(PVOID)>,
    route: Option<RouteInterrupt>,
}

/// `PCI_DEVICE::IrqRouteState`: not asked yet, routed (`IrqRouteGsi` holds it), none.
pub const ROUTE_UNKNOWN: UCHAR = 0;
const ROUTE_GSI: UCHAR = 1;
const ROUTE_NONE: UCHAR = 2;

/// The deepest path acpi.sys accepts.
const MAX_DEPTH: usize = 16;

/// A pci.sys PDO, i.e. the bridge under a secondary bus's FDO, rather than the
/// PDO acpi.sys made for a root bridge.
unsafe fn is_our_pdo(fdo: PDEVICE_OBJECT, device: PDEVICE_OBJECT) -> bool {
    (*device).DriverObject == (*fdo).DriverObject
        && (*((*device).DeviceExtension as PCOMMON_DEVICE_EXTENSION)).IsFDO == 0
}

/// At a root bus's start (PASSIVE_LEVEL): ask the PDO below for routing. A
/// secondary bus resolves through its parent instead.
pub unsafe fn fdo_query_routing(fdo: PDEVICE_OBJECT) {
    let ext = (*fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;
    let ldo = (*ext).Ldo;
    if ldo.is_null() || is_our_pdo(fdo, ldo) {
        return;
    }

    let mut interface: IrqRoutingInterface = core::mem::zeroed();
    let mut event: KEVENT = core::mem::zeroed();
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0);
    let irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        ldo,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        &mut event,
        &mut iosb,
    );
    if irp.is_null() {
        return;
    }
    (*irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;
    let stack = IoGetNextIrpStackLocation(irp);
    (*stack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*stack).Parameters.QueryInterface.InterfaceType = &GUID_VIBEOS_PCI_IRQ_ROUTING;
    (*stack).Parameters.QueryInterface.Size = size_of_val(&interface) as USHORT;
    (*stack).Parameters.QueryInterface.Version = 1;
    (*stack).Parameters.QueryInterface.Interface = addr_of_mut!(interface) as PINTERFACE;
    (*stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();
    let mut status = IofCallDriver(ldo, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(addr_of_mut!(event).cast(), _KWAIT_REASON_Executive, 0, 0, core::ptr::null_mut());
        status = iosb.__bindgen_anon_1.Status;
    }

    if status >= 0 && interface.route.is_some() {
        (*ext).IrqRouting = interface;
        DPRINT1!("bus %lu: INTx routed by ACPI _PRT\n", (*ext).BusNumber);
    } else {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        DPRINT1!(
            "bus %lu: no ACPI interrupt routing (%#lx): INTx uses each Interrupt Line\n",
            (*ext).BusNumber,
            status as ULONG
        );
    }
}

fn devfn(slot: &PCI_SLOT_NUMBER) -> UCHAR {
    let s = unsafe { slot.u.AsULONG };
    (((s & 0x1F) << 3) | ((s >> 5) & 7)) as UCHAR
}

/// The GSI this function's INTx pin is routed to, when ACPI routes its bus.
/// Resolved once per function and cached (the HAL is told the line's polarity
/// as a side effect of the first resolution).
pub unsafe fn routed_line(pdo: PPDO_DEVICE_EXTENSION, config: *const PCI_COMMON_HEADER) -> Option<ULONG> {
    let device = (*pdo).PciDevice;
    match (*device).IrqRouteState {
        ROUTE_GSI => return Some((*device).IrqRouteGsi),
        ROUTE_NONE => return None,
        _ => {}
    }
    let found = resolve(pdo, config);
    (*device).IrqRouteState = if found.is_some() { ROUTE_GSI } else { ROUTE_NONE };
    (*device).IrqRouteGsi = found.unwrap_or(0);
    found
}

unsafe fn resolve(pdo: PPDO_DEVICE_EXTENSION, config: *const PCI_COMMON_HEADER) -> Option<ULONG> {
    let pin = (*config).u.type0.InterruptPin as ULONG;
    if !(1..=4).contains(&pin) {
        return None;
    }
    let device = (*pdo).PciDevice;
    let mut path = [0 as UCHAR; MAX_DEPTH];
    let mut depth = 0;
    let mut at = pdo;
    loop {
        if depth == MAX_DEPTH {
            return None;
        }
        path[depth] = devfn(&(*(*at).PciDevice).SlotNumber);
        depth += 1;
        let fdo = (*at).Fdo;
        let ext = (*fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;
        if let Some(route) = (*ext).IrqRouting.route {
            path[..depth].reverse();
            let mut answer: IrqRoute = core::mem::zeroed();
            let status = route((*ext).IrqRouting.context, path.as_ptr(), depth as ULONG, pin, &mut answer);
            let slot = &(*device).SlotNumber;
            let line = (*config).u.type0.InterruptLine as ULONG;
            if status < 0 {
                // The board's tables do not route it: the Interrupt Line is a guess
                // that is right on few machines, so say so.
                DPRINT1!(
                    "%02lx:%02lx.%lx INT%c: ACPI cannot route it (%#lx); falling back to Interrupt Line %lu\n",
                    (*device).BusNumber,
                    slot.u.AsULONG & 0x1F,
                    (slot.u.AsULONG >> 5) & 7,
                    (b'@' as ULONG + pin) as i32,
                    status as ULONG,
                    line
                );
                return None;
            }
            DPRINT1!(
                "%02lx:%02lx.%lx INT%c -> GSI %lu (Interrupt Line %lu)\n",
                (*device).BusNumber,
                slot.u.AsULONG & 0x1F,
                (slot.u.AsULONG >> 5) & 7,
                (b'@' as ULONG + pin) as i32,
                answer.gsi,
                line
            );
            return Some(answer.gsi);
        }
        let ldo = (*ext).Ldo;
        if ldo.is_null() || !is_our_pdo(fdo, ldo) {
            return None; // a root bus without routing
        }
        at = (*ldo).DeviceExtension as PPDO_DEVICE_EXTENSION;
    }
}
