//! PCI interrupt routing from the firmware's `_PRT` (VibeOS; no C original).
//!
//! In APIC mode a PCI function's INTx pin is not wired to the input its Interrupt
//! Line register names. That byte is the 8259 number the BIOS chose for DOS-era
//! drivers; an Intel PCH delivers the same pin on I/O APIC inputs 16-23, and only
//! the ACPI `_PRT` says which. Treating the Interrupt Line as an I/O APIC input
//! works on QEMU's i440fx (its PIRQ links do land on inputs 10/11) and nowhere
//! else: on the OptiPlex 9020 EHCI's interrupt never arrived and USB enumeration
//! hung on its first control transfer.
//!
//! So, as Windows does:
//! - when the HAL routes through an I/O APIC, `acpi_select_apic_model` evaluates
//!   `\_PIC(1)` once the namespace is up, which makes the board's `_PRT` methods
//!   return their APIC tables;
//! - the PCI bus driver asks the root bridge's PDO for this private interface and
//!   resolves each function through it: walk from the function up through its
//!   bridges to the nearest bus that has a `_PRT` (swizzling the pin per bridge
//!   on the way up, PCI-to-PCI bridge spec 9.1), then read the entry. A direct
//!   entry names a GSI (level, active-low by the ACPI spec); a link device's
//!   `_CRS` names one with its own trigger and polarity;
//! - the polarity goes to the HAL (`HalVibeosSetInterruptLine`) before the
//!   device's interrupt resource is translated, so the I/O APIC entry is
//!   programmed for the wire it actually is.
//!
//! The interface layout is shared with pci-rust (`pdo.rs`); change both together.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicBool, Ordering};

use acpica_sys::{
    AcpiEvaluateObject, AcpiGetCurrentResources, AcpiGetHandle, AcpiGetIrqRoutingTable, AcpiGetNextObject,
    ACPI_BUFFER, ACPI_HANDLE, ACPI_OBJECT, ACPI_OBJECT_LIST, ACPI_PCI_ROUTING_TABLE, ACPI_RESOURCE,
    ACPI_RESOURCE_EXTENDED_IRQ, ACPI_RESOURCE_IRQ, ACPI_SIZE, ACPI_STATUS,
};
use ntkrnl_sys::ddk::{BOOLEAN, GUID, NTSTATUS, PVOID, UCHAR, ULONG, USHORT};

use crate::osl::AcpiOsFree;

const AE_OK: ACPI_STATUS = 0;
const AE_NOT_FOUND: ACPI_STATUS = 0x0005;
const ACPI_ALLOCATE_BUFFER: ACPI_SIZE = ACPI_SIZE::MAX; // (ACPI_SIZE)(-1)
const ACPI_TYPE_INTEGER: u32 = 1;
const ACPI_TYPE_DEVICE: u32 = 6;
const ACPI_RESOURCE_TYPE_IRQ: u32 = 0;
const ACPI_RESOURCE_TYPE_END_TAG: u32 = 7;
const ACPI_RESOURCE_TYPE_EXTENDED_IRQ: u32 = 15;
const ACPI_LEVEL_SENSITIVE: u8 = 0;
const ACPI_ACTIVE_LOW: u8 = 1;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_FOUND: NTSTATUS = 0xC000_0225u32 as NTSTATUS;
const STATUS_INVALID_PARAMETER: NTSTATUS = 0xC000_000Du32 as NTSTATUS;
const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC000_00BBu32 as NTSTATUS;

/// `{9c5c2f4e-6d0b-4a8e-b0f1-5e7d3a2c8b41}`: VibeOS's PCI interrupt routing interface.
pub static GUID_VIBEOS_PCI_IRQ_ROUTING: GUID = GUID {
    Data1: 0x9c5c_2f4e,
    Data2: 0x6d0b,
    Data3: 0x4a8e,
    Data4: [0xb0, 0xf1, 0x5e, 0x7d, 0x3a, 0x2c, 0x8b, 0x41],
};

/// Where a PCI function's INTx pin lands.
#[repr(C)]
pub struct VIBEOS_PCI_IRQ_ROUTE {
    pub Gsi: ULONG,
    pub LevelSensitive: BOOLEAN,
    pub ActiveLow: BOOLEAN,
}

/// `Path` lists the device/function (`dev << 3 | fn`) of each hop from the root
/// bus down: the bridges in order, the function itself last. `Pin` is the
/// function's Interrupt Pin register (1 = INTA).
pub type PVIBEOS_ROUTE_INTERRUPT = unsafe extern "system" fn(
    Context: PVOID,
    Path: *const UCHAR,
    Depth: ULONG,
    Pin: ULONG,
    Route: *mut VIBEOS_PCI_IRQ_ROUTE,
) -> NTSTATUS;

/// An `INTERFACE` header followed by the one routine.
#[repr(C)]
pub struct VIBEOS_PCI_IRQ_ROUTING_INTERFACE {
    pub Size: USHORT,
    pub Version: USHORT,
    pub Context: PVOID,
    pub InterfaceReference: Option<unsafe extern "system" fn(PVOID)>,
    pub InterfaceDereference: Option<unsafe extern "system" fn(PVOID)>,
    pub RouteInterrupt: Option<PVIBEOS_ROUTE_INTERRUPT>,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
extern "system" {
    fn HalVibeosIoApicInputs() -> ULONG;
    fn HalVibeosSetInterruptLine(Gsi: ULONG, LevelSensitive: BOOLEAN, ActiveLow: BOOLEAN) -> BOOLEAN;
}

// No I/O APIC outside x86: `acpi_select_apic_model` never selects the model,
// so routing is never offered and this is never reached.
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
unsafe fn HalVibeosSetInterruptLine(_Gsi: ULONG, _LevelSensitive: BOOLEAN, _ActiveLow: BOOLEAN) -> BOOLEAN {
    0
}

/// Set once `\_PIC(1)` has run: the namespace's `_PRT`s describe APIC inputs.
static APIC_MODEL: AtomicBool = AtomicBool::new(false);

/// Select the APIC interrupt model if the HAL has an I/O APIC. Runs once, after
/// `AcpiInitializeObjects` and before anything reads a `_PRT`.
pub unsafe fn acpi_select_apic_model() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if HalVibeosIoApicInputs() == 0 {
            return; // 8259 HAL: the Interrupt Line registers are the truth
        }
        let mut arg: ACPI_OBJECT = core::mem::zeroed();
        arg.Integer.Type = ACPI_TYPE_INTEGER as _;
        arg.Integer.Value = 1; // APIC mode
        let mut args = ACPI_OBJECT_LIST { Count: 1, Pointer: &mut arg };
        let status = AcpiEvaluateObject(
            core::ptr::null_mut(),
            b"\\_PIC\0".as_ptr() as *mut c_char,
            &mut args,
            core::ptr::null_mut(),
        );
        // _PIC is optional: a board without it has one _PRT for both models.
        if status != AE_OK && status != AE_NOT_FOUND {
            DPRINT1!("\\_PIC(1) failed %#x: PCI interrupts stay on their Interrupt Line\n", status);
            return;
        }
        APIC_MODEL.store(true, Ordering::Release);
        DPRINT1!("APIC interrupt model selected (\\_PIC %s)\n", if status == AE_OK { b"ran\0".as_ptr() } else { b"absent\0".as_ptr() });
    }
}

/// Fill `Interface` for a PDO whose ACPI node may carry a `_PRT` (a root bridge).
pub unsafe fn query_routing_interface(handle: ACPI_HANDLE, interface: *mut c_void, size: USHORT) -> NTSTATUS {
    if handle.is_null() || !APIC_MODEL.load(Ordering::Acquire) {
        return STATUS_NOT_SUPPORTED;
    }
    if (size as usize) < core::mem::size_of::<VIBEOS_PCI_IRQ_ROUTING_INTERFACE>() {
        return STATUS_INVALID_PARAMETER;
    }
    let out = interface as *mut VIBEOS_PCI_IRQ_ROUTING_INTERFACE;
    (*out).Size = core::mem::size_of::<VIBEOS_PCI_IRQ_ROUTING_INTERFACE>() as USHORT;
    (*out).Version = 1;
    (*out).Context = handle;
    (*out).InterfaceReference = Some(interface_noop);
    (*out).InterfaceDereference = Some(interface_noop);
    (*out).RouteInterrupt = Some(route_interrupt);
    STATUS_SUCCESS
}

unsafe extern "system" fn interface_noop(_context: PVOID) {}

/// The deepest bus a route can name; PCI hierarchies in practice are a few deep.
const MAX_DEPTH: usize = 16;

unsafe extern "system" fn route_interrupt(
    Context: PVOID,
    Path: *const UCHAR,
    Depth: ULONG,
    Pin: ULONG,
    Route: *mut VIBEOS_PCI_IRQ_ROUTE,
) -> NTSTATUS {
    let depth = Depth as usize;
    if Context.is_null() || Path.is_null() || Route.is_null() || depth == 0 || depth > MAX_DEPTH || !(1..=4).contains(&Pin) {
        return STATUS_INVALID_PARAMETER;
    }
    let path = core::slice::from_raw_parts(Path, depth);

    // bus[k] is the ACPI node of the bus path[k] sits on: the root bridge for
    // k = 0, else the node of the bridge path[k-1], when the namespace has one.
    let mut bus: [ACPI_HANDLE; MAX_DEPTH] = [core::ptr::null_mut(); MAX_DEPTH];
    bus[0] = Context;
    for k in 1..depth {
        bus[k] = if bus[k - 1].is_null() { core::ptr::null_mut() } else { child_at(bus[k - 1], path[k - 1]) };
    }

    // From the function upward: the nearest bus with a _PRT answers; every
    // bridge without one swizzles the pin by the device number below it.
    let mut pin = Pin - 1;
    let mut k = depth - 1;
    loop {
        let slot = (path[k] >> 3) as u64;
        if !bus[k].is_null() {
            match lookup_prt(bus[k], slot, pin) {
                Lookup::Found(gsi, level, low) => {
                    if HalVibeosSetInterruptLine(gsi, level as BOOLEAN, low as BOOLEAN) == 0 {
                        return STATUS_NOT_SUPPORTED;
                    }
                    (*Route).Gsi = gsi;
                    (*Route).LevelSensitive = level as BOOLEAN;
                    (*Route).ActiveLow = low as BOOLEAN;
                    return STATUS_SUCCESS;
                }
                Lookup::Failed => return STATUS_NOT_FOUND,
                Lookup::NoTable => {}
            }
        }
        if k == 0 {
            return STATUS_NOT_FOUND; // the root bridge has no _PRT
        }
        pin = (pin + slot as ULONG) % 4;
        k -= 1;
    }
}

/// The child device of `parent` whose `_ADR` is `devfn`'s device and function.
unsafe fn child_at(parent: ACPI_HANDLE, devfn: UCHAR) -> ACPI_HANDLE {
    let want = (((devfn >> 3) as u64) << 16) | (devfn & 7) as u64;
    let mut child: ACPI_HANDLE = core::ptr::null_mut();
    loop {
        let mut next: ACPI_HANDLE = core::ptr::null_mut();
        if AcpiGetNextObject(ACPI_TYPE_DEVICE as _, parent, child, &mut next) != AE_OK {
            return core::ptr::null_mut();
        }
        child = next;
        let mut obj: ACPI_OBJECT = core::mem::zeroed();
        let mut buf = ACPI_BUFFER { Length: core::mem::size_of::<ACPI_OBJECT>() as _, Pointer: (&raw mut obj).cast() };
        if AcpiEvaluateObject(child, b"_ADR\0".as_ptr() as *mut c_char, core::ptr::null_mut(), &mut buf) == AE_OK
            && obj.Type == ACPI_TYPE_INTEGER as _
            && obj.Integer.Value == want
        {
            return child;
        }
    }
}

enum Lookup {
    Found(ULONG, bool, bool),
    /// The bus has a `_PRT` but it does not route this pin usefully.
    Failed,
    /// No `_PRT` here: swizzle and ask the bus above.
    NoTable,
}

unsafe fn lookup_prt(bus: ACPI_HANDLE, slot: u64, pin: ULONG) -> Lookup {
    let mut buf = ACPI_BUFFER { Length: ACPI_ALLOCATE_BUFFER, Pointer: core::ptr::null_mut() };
    let status = AcpiGetIrqRoutingTable(bus, &mut buf);
    if status != AE_OK || buf.Pointer.is_null() {
        if status != AE_NOT_FOUND {
            DPRINT1!("_PRT unreadable: %#x\n", status);
        }
        return Lookup::NoTable;
    }
    let mut result = Lookup::Failed;
    let mut entry = buf.Pointer as *const u8;
    loop {
        let e = entry as *const ACPI_PCI_ROUTING_TABLE;
        let length = core::ptr::addr_of!((*e).Length).read_unaligned();
        if length == 0 {
            DPRINT1!("_PRT has no entry for device %u INT%c\n", slot as ULONG, (b'A' as ULONG + pin) as i32);
            break;
        }
        let address = core::ptr::addr_of!((*e).Address).read_unaligned();
        if (address >> 16) & 0xFFFF == slot && core::ptr::addr_of!((*e).Pin).read_unaligned() == pin {
            let index = core::ptr::addr_of!((*e).SourceIndex).read_unaligned();
            let source = core::ptr::addr_of!((*e).Source) as *const c_char;
            result = if *source == 0 {
                // A direct entry: SourceIndex is the GSI, and the wire is a PCI
                // INTx, level-triggered and active-low (ACPI 6.5, 6.2.13).
                DPRINT1!("_PRT: device %u INT%c -> GSI %u\n", slot as ULONG, (b'A' as ULONG + pin) as i32, index);
                Lookup::Found(index, true, true)
            } else {
                link_irq(bus, source, slot, pin)
            };
            break;
        }
        entry = entry.add(length as usize);
    }
    AcpiOsFree(buf.Pointer);
    result
}

/// The interrupt a link device (`LNKA`, `GSIA`, ...) currently routes to.
unsafe fn link_irq(bus: ACPI_HANDLE, name: *const c_char, slot: u64, pin: ULONG) -> Lookup {
    let mut link: ACPI_HANDLE = core::ptr::null_mut();
    if AcpiGetHandle(bus, name as *mut c_char, &mut link) != AE_OK {
        DPRINT1!("_PRT: link %s not found\n", name);
        return Lookup::Failed;
    }
    let mut buf = ACPI_BUFFER { Length: ACPI_ALLOCATE_BUFFER, Pointer: core::ptr::null_mut() };
    if AcpiGetCurrentResources(link, &mut buf) != AE_OK || buf.Pointer.is_null() {
        DPRINT1!("_PRT: link %s has no _CRS\n", name);
        return Lookup::Failed;
    }
    let mut found = None;
    let mut res = buf.Pointer as *const ACPI_RESOURCE;
    let end = (buf.Pointer as *const u8).add(buf.Length as usize);
    while (res as *const u8) < end && (*res).Type != ACPI_RESOURCE_TYPE_END_TAG && (*res).Length != 0 {
        let data = core::ptr::addr_of!((*res).Data) as *const u8;
        if (*res).Type == ACPI_RESOURCE_TYPE_IRQ {
            let irq = data as *const ACPI_RESOURCE_IRQ;
            if (*irq).InterruptCount > 0 {
                let n = *core::ptr::addr_of!((*irq).Interrupts).cast::<u8>() as ULONG;
                found = Some((n, (*irq).Triggering == ACPI_LEVEL_SENSITIVE, (*irq).Polarity == ACPI_ACTIVE_LOW));
                break;
            }
        } else if (*res).Type == ACPI_RESOURCE_TYPE_EXTENDED_IRQ {
            let irq = data as *const ACPI_RESOURCE_EXTENDED_IRQ;
            if (*irq).InterruptCount > 0 {
                let n = core::ptr::addr_of!((*irq).Interrupts).cast::<u32>().read_unaligned();
                found = Some((n, (*irq).Triggering == ACPI_LEVEL_SENSITIVE, (*irq).Polarity == ACPI_ACTIVE_LOW));
                break;
            }
        }
        res = (res as *const u8).add((*res).Length as usize) as *const ACPI_RESOURCE;
    }
    AcpiOsFree(buf.Pointer);
    match found {
        // A link reporting interrupt 0 is disabled: nothing routed it.
        Some((n, level, low)) if n != 0 => {
            DPRINT1!(
                "_PRT: device %u INT%c -> %s -> GSI %u, %s, active-%s\n",
                slot as ULONG,
                (b'A' as ULONG + pin) as i32,
                name,
                n,
                if level { b"level\0".as_ptr() } else { b"edge\0".as_ptr() },
                if low { b"low\0".as_ptr() } else { b"high\0".as_ptr() },
            );
            Lookup::Found(n, level, low)
        }
        _ => {
            DPRINT1!("_PRT: link %s routes no interrupt\n", name);
            Lookup::Failed
        }
    }
}
