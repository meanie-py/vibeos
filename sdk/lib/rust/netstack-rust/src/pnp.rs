//! netstack as a PnP function driver for NICs that only ACPI describes.
//!
//! netstack finds its PCI NICs (virtio-net, e1000) itself, as a non-PnP
//! system-start driver. A platform NIC has no PCI header to scan: the arm64
//! HAL's PnP root (hal-rust/src/arm64/pnpdd.rs) reads it from the ACPI tables
//! and publishes it as a device, `ACPI\BCM6E4E` for the Raspberry Pi 4's GENET
//! or `VIBEOS\RP1GEM` for the Pi 5's Cadence GEM (which the HAL synthesizes,
//! the firmware's tables leaving it out), and the critical device database
//! names netstack as its service. So the PnP
//! manager calls `add_device` here, then sends IRP_MN_START_DEVICE with the
//! register window as a translated resource; this module records it and wakes
//! the engine worker, which waits for it when no PCI NIC exists (`wait`).
//!
//! The device stack is just our FDO on the HAL's PDO. Everything but START
//! passes down untouched, and the TDI devices that share this driver object
//! never see PnP or power IRPs: tdidev's dispatcher routes those here.

use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, AtomicU64, AtomicU8, Ordering};

use ntkrnl_sys::ddk::{
    IoAttachDeviceToDeviceStack, IoCreateDevice, IoForwardIrpSynchronously, IoGetDeviceProperty, KeInitializeEvent,
    KeSetEvent, KeWaitForSingleObject, DRIVER_OBJECT, KEVENT, LARGE_INTEGER, NTSTATUS,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, ULONG,
};
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest, PoCallDriver, PoStartNextPowerIrp};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};

use crate::serial::nsprint;

const IRP_MJ_POWER: u8 = 0x16;
const IRP_MJ_PNP: u8 = 0x1B;
const IRP_MN_START_DEVICE: u8 = 0x00;
const IRP_MN_REMOVE_DEVICE: u8 = 0x02;
const CM_RESOURCE_TYPE_MEMORY: u8 = 3;
const FILE_DEVICE_PHYSICAL_NETCARD: ULONG = 0x17;
const DO_DEVICE_INITIALIZING: ULONG = 0x80;
const NOTIFICATION_EVENT: u32 = 0; // _EVENT_TYPE_NotificationEvent
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_TIMEOUT: NTSTATUS = 0x0000_0102;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000_000Eu32 as NTSTATUS;
const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC000_0182u32 as NTSTATUS;
const DEVICE_PROPERTY_HARDWARE_ID: i32 = 1; // DevicePropertyHardwareID

/// The hardware IDs the engine has a backend for: genet.rs and gem.rs.
const BACKENDS: [(&str, NicKind); 2] = [("ACPI\\BCM6E4E", NicKind::Genet), ("VIBEOS\\RP1GEM", NicKind::Gem)];

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum NicKind {
    /// The Raspberry Pi 4's Broadcom GENET v5.
    Genet = 1,
    /// The Raspberry Pi 5's Cadence GEM, inside RP1.
    Gem = 2,
}

/// The platform NIC's register window, as the PnP manager assigned it.
#[derive(Clone, Copy)]
pub struct PlatformNic {
    pub kind: NicKind,
    pub mmio: u64,
    pub mmio_len: u64,
}

/// Our FDO, and the PDO below it (the HAL's ACPI child).
static FDO: AtomicPtr<ntkrnl_sys::ddk::DEVICE_OBJECT> = AtomicPtr::new(null_mut());
static LOWER: AtomicPtr<ntkrnl_sys::ddk::DEVICE_OBJECT> = AtomicPtr::new(null_mut());
static MMIO: AtomicU64 = AtomicU64::new(0);
static MMIO_LEN: AtomicU64 = AtomicU64::new(0);
static KIND: AtomicU8 = AtomicU8::new(0);
/// Signalled when AddDevice attaches, and when START_DEVICE has recorded the
/// resources.
static mut ATTACHED: KEVENT = unsafe { core::mem::zeroed() };
static mut STARTED: KEVENT = unsafe { core::mem::zeroed() };
/// A board with a platform NIC loads netstack FOR it, so AddDevice follows
/// DriverEntry within moments; the PnP manager starts the device right after.
/// Without one (any PCI-NIC or NIC-less machine), the engine gives up after
/// ATTACH_WAIT_MS, and TDI requests fail fast instead of pending meanwhile.
const ATTACH_WAIT_MS: i64 = 1_000;
const START_WAIT_MS: i64 = 10_000;

/// Hook netstack into PnP. Called from DriverEntry before anything can send us
/// an IRP; tdidev::init then installs the dispatcher that routes PnP and power
/// IRPs for our FDO to `dispatch`.
pub unsafe fn init(driver: *mut DRIVER_OBJECT) {
    KeInitializeEvent(&raw mut ATTACHED, NOTIFICATION_EVENT as _, 0);
    KeInitializeEvent(&raw mut STARTED, NOTIFICATION_EVENT as _, 0);
    (*(*driver).DriverExtension).AddDevice = Some(core::mem::transmute::<_, _>(add_device as *const ()));
}

/// Is `device` the platform NIC's FDO (as opposed to a TDI device)?
pub fn owns(device: PDEVICE_OBJECT) -> bool {
    !device.is_null() && device == FDO.load(Ordering::Acquire)
}

/// Wait for a platform NIC to attach and start (see ATTACH_WAIT_MS).
pub fn wait() -> Option<PlatformNic> {
    unsafe fn wait_for(event: *mut KEVENT, ms: i64) -> bool {
        let mut timeout: LARGE_INTEGER = core::mem::zeroed();
        timeout.QuadPart = -(ms * 10_000);
        // Executive (0), KernelMode (0), not alertable.
        KeWaitForSingleObject(event.cast(), 0, 0, 0, &mut timeout) != STATUS_TIMEOUT
    }
    if !unsafe { wait_for(&raw mut ATTACHED, ATTACH_WAIT_MS) } {
        return None;
    }
    if !unsafe { wait_for(&raw mut STARTED, START_WAIT_MS) } {
        nsprint!("[NETSTACK] PnP: the platform NIC attached but was never started\n");
        return None;
    }
    let mmio = MMIO.load(Ordering::Acquire);
    let kind = match KIND.load(Ordering::Acquire) {
        k if k == NicKind::Gem as u8 => NicKind::Gem,
        _ => NicKind::Genet,
    };
    (mmio != 0).then(|| PlatformNic { kind, mmio, mmio_len: MMIO_LEN.load(Ordering::Acquire) })
}

unsafe extern "system" fn add_device(driver: PDRIVER_OBJECT, pdo: PDEVICE_OBJECT) -> NTSTATUS {
    if !FDO.load(Ordering::Acquire).is_null() {
        // One platform NIC per machine is all the engine drives.
        nsprint!("[NETSTACK] PnP: a second platform NIC was ignored\n");
        return STATUS_NO_SUCH_DEVICE;
    }
    // The critical device database is what routes devices here, but refuse
    // anything the engine has no backend for.
    let mut ids = [0u16; 128];
    let mut got: ULONG = 0;
    let status = IoGetDeviceProperty(
        pdo,
        DEVICE_PROPERTY_HARDWARE_ID as _,
        (ids.len() * 2) as ULONG,
        ids.as_mut_ptr().cast(),
        &mut got,
    );
    let first = || ids.iter().take_while(|&&c| c != 0).map(|&c| c as u8);
    let Some(&(_, kind)) = BACKENDS.iter().find(|(id, _)| status >= 0 && first().eq(id.bytes())) else {
        nsprint!("[NETSTACK] PnP: no backend for this device (hardware ID query {:#010x}); not attaching\n", status as u32);
        return STATUS_NO_SUCH_DEVICE;
    };
    KIND.store(kind as u8, Ordering::Release);
    let mut fdo: PDEVICE_OBJECT = null_mut();
    let status = IoCreateDevice(driver, 0, null_mut(), FILE_DEVICE_PHYSICAL_NETCARD, 0, 0, &mut fdo);
    if status < 0 {
        nsprint!("[NETSTACK] PnP: IoCreateDevice failed {:#010x}\n", status as u32);
        return status;
    }
    let lower = IoAttachDeviceToDeviceStack(fdo, pdo);
    if lower.is_null() {
        ntkrnl_sys::ddk::IoDeleteDevice(fdo);
        return STATUS_NO_SUCH_DEVICE;
    }
    LOWER.store(lower, Ordering::Release);
    FDO.store(fdo, Ordering::Release);
    (*fdo).Flags &= !DO_DEVICE_INITIALIZING;
    nsprint!("[NETSTACK] PnP: platform NIC attached\n");
    KeSetEvent(&raw mut ATTACHED, 0, 0);
    STATUS_SUCCESS
}

/// PnP and power IRPs for our FDO.
pub unsafe fn dispatch(_device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    let io = IoGetCurrentIrpStackLocation(irp);
    let lower = LOWER.load(Ordering::Acquire);
    match (*io).MajorFunction {
        IRP_MJ_PNP if (*io).MinorFunction == IRP_MN_START_DEVICE => {
            // The PDO starts first; then the resources are ours.
            let status = if IoForwardIrpSynchronously(lower, irp) == 0 {
                STATUS_NO_SUCH_DEVICE
            } else {
                let s = (*irp).IoStatus.__bindgen_anon_1.Status;
                if s < 0 { s } else { start(io) }
            };
            (*irp).IoStatus.__bindgen_anon_1.Status = status;
            IoCompleteRequest(irp, 0);
            status
        }
        IRP_MJ_PNP => {
            if (*io).MinorFunction == IRP_MN_REMOVE_DEVICE {
                // The engine owns the hardware for the life of the system; a
                // removal would have to stop it first. Not supported yet: say so.
                nsprint!("[NETSTACK] PnP: platform NIC removal is not supported\n");
            }
            IoSkipCurrentIrpStackLocation(irp);
            IoCallDriver(lower, irp)
        }
        IRP_MJ_POWER => {
            PoStartNextPowerIrp(irp);
            IoSkipCurrentIrpStackLocation(irp);
            PoCallDriver(lower, irp)
        }
        _ => {
            IoSkipCurrentIrpStackLocation(irp);
            IoCallDriver(lower, irp)
        }
    }
}

/// Take the first memory range from the translated resources and wake the engine.
unsafe fn start(io: ntkrnl_sys::ddk::PIO_STACK_LOCATION) -> NTSTATUS {
    let list = (*io).Parameters.StartDevice.AllocatedResourcesTranslated;
    if list.is_null() || (*list).Count == 0 {
        nsprint!("[NETSTACK] PnP: START_DEVICE carried no resources\n");
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }
    let partial = &(*(*list).List.as_ptr()).PartialResourceList;
    let descs = core::slice::from_raw_parts(partial.PartialDescriptors.as_ptr(), partial.Count as usize);
    let Some(mem) = descs.iter().find(|d| d.Type == CM_RESOURCE_TYPE_MEMORY) else {
        nsprint!("[NETSTACK] PnP: START_DEVICE carried no register window\n");
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    };
    let mmio = mem.u.Memory.Start.QuadPart as u64;
    let len = mem.u.Memory.Length as u64;
    nsprint!("[NETSTACK] PnP: platform NIC started, regs {:#x}+{:#x}\n", mmio, len);
    MMIO_LEN.store(len, Ordering::Release);
    MMIO.store(mmio, Ordering::Release);
    KeSetEvent(&raw mut STARTED, 0, 0);
    STATUS_SUCCESS
}
