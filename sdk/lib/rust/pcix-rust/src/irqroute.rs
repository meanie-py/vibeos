//! Where a function's INTx pin really lands (VibeOS; no C original).
//!
//! The Interrupt Line register is the 8259 number the BIOS picked. Under an I/O APIC
//! the same pin arrives on whatever input the board wires it to (16-23 on an Intel
//! PCH), and on arm64 on a GIC SPI the register never names at all; only the firmware's
//! tables know. On Windows acpi.sys's interrupt arbiter applies `_PRT` to pcix's
//! open-ended request. We have no such arbiter, so pcix asks the root bridge's PDO for
//! the private routing interface pci.sys also uses (acpi-rust `pciroute.rs` owns the
//! layout; the arm64 HAL answers it for its own root) and requests exactly the line it
//! names.
//!
//! A root FDO asks once, when it is added. A function on any bus is resolved by handing
//! that interface its device/function path from the root down; the answer is cached in
//! the PDO extension. Without the interface (the 8259 HAL, no ACPI) the Interrupt Line
//! stays the truth, as it is on such machines.
#![allow(non_snake_case)]

use core::ffi::c_void;

use crate::types::*;
use ntkrnl_sys::ddk::{
    IoBuildSynchronousFsdRequest, IofCallDriver, KeInitializeEvent, KeWaitForSingleObject, GUID, KEVENT, NTSTATUS,
    PCI_SLOT_NUMBER, PINTERFACE, PVOID, UCHAR, ULONG, USHORT, _EVENT_TYPE_SynchronizationEvent, _IO_STATUS_BLOCK,
    _KWAIT_REASON_Executive,
};
use ntkrnl_sys::driver_ddk::{DbgPrint, IRP_MJ_PNP, IRP_MN_QUERY_INTERFACE};
use ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation;

const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC000_00BBu32 as NTSTATUS;
const STATUS_PENDING: NTSTATUS = 0x0000_0103;

/// `{9c5c2f4e-6d0b-4a8e-b0f1-5e7d3a2c8b41}`, acpi-rust's `GUID_VIBEOS_PCI_IRQ_ROUTING`.
static GUID_VIBEOS_PCI_IRQ_ROUTING: GUID = GUID {
    Data1: 0x9c5c_2f4e,
    Data2: 0x6d0b,
    Data3: 0x4a8e,
    Data4: [0xb0, 0xf1, 0x5e, 0x7d, 0x3a, 0x2c, 0x8b, 0x41],
};

/// acpi-rust's `VIBEOS_PCI_IRQ_ROUTING_INTERFACE`: an `INTERFACE` header and one routine.
#[repr(C)]
struct IrqRoutingInterface {
    size: USHORT,
    version: USHORT,
    context: PVOID,
    reference: Option<unsafe extern "system" fn(PVOID)>,
    dereference: Option<unsafe extern "system" fn(PVOID)>,
    route: PVIBEOS_ROUTE_INTERRUPT,
}

/// `PCI_PDO_EXTENSION::IrqRouteState`: not asked yet, routed (`IrqRoute` holds it), none.
const ROUTE_UNKNOWN: UCHAR = 0;
const ROUTE_GSI: UCHAR = 1;
const ROUTE_NONE: UCHAR = 2;

/// The deepest path the routing interface accepts.
const MAX_DEPTH: usize = 16;

/// When a root FDO is added (PASSIVE_LEVEL): ask the PDO below it for routing.
pub unsafe fn fdo_query_routing(fdo: PPCI_FDO_EXTENSION) {
    let lower = (*fdo).AttachedDeviceObject;
    if lower.is_null() || (*fdo).BusRootFdoExtension != fdo {
        return;
    }

    let mut interface: IrqRoutingInterface = core::mem::zeroed();
    let mut event: KEVENT = core::mem::zeroed();
    let mut iosb: _IO_STATUS_BLOCK = core::mem::zeroed();
    KeInitializeEvent(&mut event, _EVENT_TYPE_SynchronizationEvent, 0);
    let irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        lower,
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
    (*stack).Parameters.QueryInterface.Size = core::mem::size_of::<IrqRoutingInterface>() as USHORT;
    (*stack).Parameters.QueryInterface.Version = 1;
    (*stack).Parameters.QueryInterface.Interface = &raw mut interface as PINTERFACE;
    (*stack).Parameters.QueryInterface.InterfaceSpecificData = core::ptr::null_mut();
    let mut status = IofCallDriver(lower, irp);
    if status == STATUS_PENDING {
        KeWaitForSingleObject(&raw mut event as *mut c_void, _KWAIT_REASON_Executive, 0, 0, core::ptr::null_mut());
        status = iosb.__bindgen_anon_1.Status;
    }

    if status >= 0 && interface.route.is_some() {
        (*fdo).IrqRoutingContext = interface.context;
        (*fdo).IrqRouteInterrupt = interface.route;
        DbgPrint(
            b"PCI   Bus %u: INTx routed by the firmware's tables.\n\0".as_ptr() as *const i8,
            (*fdo).BaseBus as u32,
        );
    } else {
        DbgPrint(
            b"PCI   Bus %u: no interrupt routing (%#lx): INTx uses each Interrupt Line.\n\0".as_ptr() as *const i8,
            (*fdo).BaseBus as u32,
            status as u32,
        );
    }
}

fn devfn(slot: PCI_SLOT_NUMBER) -> UCHAR {
    let s = unsafe { slot.u.AsULONG };
    (((s & 0x1F) << 3) | ((s >> 5) & 7)) as UCHAR
}

/// Where this function's INTx pin is routed, when its root bus has routing. Resolved
/// once per function (the HAL learns the line's polarity as a side effect) and cached.
pub unsafe fn routed(pdo: PPCI_PDO_EXTENSION) -> Option<VIBEOS_PCI_IRQ_ROUTE> {
    match (*pdo).IrqRouteState {
        ROUTE_GSI => return Some(core::ptr::read(&(*pdo).IrqRoute)),
        ROUTE_NONE => return None,
        ROUTE_UNKNOWN | _ => {}
    }
    let found = resolve(pdo);
    match found {
        Some(ref r) => {
            (*pdo).IrqRoute = core::ptr::read(r);
            (*pdo).IrqRouteState = ROUTE_GSI;
        }
        None => (*pdo).IrqRouteState = ROUTE_NONE,
    }
    found
}

/// The interrupt this function's resources name: its routed line, else the Interrupt
/// Line the firmware wrote (0 and 0xFF mean none), with whether it is level-triggered.
pub unsafe fn interrupt_line(pdo: PPCI_PDO_EXTENSION) -> Option<(ULONG, bool)> {
    if (*pdo).InterruptPin == 0 {
        return None;
    }
    if let Some(r) = routed(pdo) {
        return Some((r.Gsi, r.LevelSensitive != 0));
    }
    match (*pdo).AdjustedInterruptLine {
        0 | 0xFF => None,
        line => Some((line as ULONG, true)),
    }
}

unsafe fn resolve(pdo: PPCI_PDO_EXTENSION) -> Option<VIBEOS_PCI_IRQ_ROUTE> {
    let pin = (*pdo).InterruptPin as ULONG;
    if !(1..=4).contains(&pin) {
        return None;
    }
    let mut path = [0 as UCHAR; MAX_DEPTH];
    let mut depth = 0;
    let mut at = pdo;
    loop {
        if depth == MAX_DEPTH {
            return None;
        }
        path[depth] = devfn((*at).Slot);
        depth += 1;
        let fdo = (*at).ParentFdoExtension;
        if (*fdo).BusRootFdoExtension == fdo {
            let route = (*fdo).IrqRouteInterrupt?;
            path[..depth].reverse();
            let mut answer: VIBEOS_PCI_IRQ_ROUTE = core::mem::zeroed();
            let status = route((*fdo).IrqRoutingContext, path.as_ptr(), depth as ULONG, pin, &mut answer);
            let slot = (*pdo).Slot.u.AsULONG;
            if status < 0 {
                // The firmware's tables do not route it: the Interrupt Line is a guess
                // that is right on few machines, so say so.
                DbgPrint(
                    b"PCI   %02x:%02x.%x INT%c: not routed (%#lx); using Interrupt Line %u\n\0".as_ptr() as *const i8,
                    (*(*pdo).ParentFdoExtension).BaseBus as u32,
                    slot & 0x1F,
                    (slot >> 5) & 7,
                    (b'@' as ULONG + pin) as i32,
                    status as u32,
                    (*pdo).AdjustedInterruptLine as u32,
                );
                return None;
            }
            DbgPrint(
                b"PCI   %02x:%02x.%x INT%c -> GSI %u (Interrupt Line %u)\n\0".as_ptr() as *const i8,
                (*(*pdo).ParentFdoExtension).BaseBus as u32,
                slot & 0x1F,
                (slot >> 5) & 7,
                (b'@' as ULONG + pin) as i32,
                answer.Gsi,
                (*pdo).AdjustedInterruptLine as u32,
            );
            return Some(answer);
        }
        // A secondary bus: its FDO sits on the bridge's PDO, which is one of ours.
        at = (*(*fdo).PhysicalDeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
    }
}
