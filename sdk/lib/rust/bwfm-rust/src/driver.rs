//! bwfm.sys's device stack: our FDO on the HAL's `VIBEOS\CYW43455` PDO (the Pi
//! 5's SDC1, hal-rust/src/arm64/pnpdd.rs), bound through the critical device
//! database.
//!
//! So far START powers the chip, brings the SDIO bus up, and reads the chip id
//! and its cores:
//! the first board milestone in docs/engineering/pi-wifi-design.md. The chip
//! stays attached in `CHIP` for the firmware download that comes next. Every
//! other IRP passes down untouched.

use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

use ntkrnl_sys::ddk::{
    IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice, IoForwardIrpSynchronously, KeDelayExecutionThread,
    KeQueryPerformanceCounter, KeStallExecutionProcessor, MmMapIoSpace, _MEMORY_CACHING_TYPE_MmNonCached, DEVICE_OBJECT,
    LARGE_INTEGER, NTSTATUS, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, ULONG,
};
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest, PoCallDriver, PoStartNextPowerIrp};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use sdhci_rust::Platform;

use crate::chip::Chip;
use crate::print::bwprint;
use crate::sdio::Sdio;

const IRP_MJ_POWER: u8 = 0x16;
const IRP_MJ_PNP: u8 = 0x1B;
const IRP_MN_START_DEVICE: u8 = 0x00;
const CM_RESOURCE_TYPE_MEMORY: u8 = 3;
const FILE_DEVICE_PHYSICAL_NETCARD: ULONG = 0x17;
const FILE_DEVICE_SECURE_OPEN: ULONG = 0x100;
const DO_DEVICE_INITIALIZING: ULONG = 0x80;
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000_000Eu32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
const STATUS_DEVICE_CONFIGURATION_ERROR: NTSTATUS = 0xC000_0182u32 as NTSTATUS;

/// The Pi 5's Wi-Fi SDIO host (BCM2712 `sdio2`), as SDC1's _CRS gives it.
const PI5_SDIO2: u64 = 0x10_0110_0000;
/// The Pi 5's `brcm,brcmstb-gpio` block. Bank 0: ODEN +0, DATA +4, IODIR +8
/// (a set IODIR bit is an input). WL_REG_ON is its line 28, active high, and
/// the chip wants 150 ms after it rises (the Pi 5 device tree's `wl_on_reg`).
/// ACPI does not describe it, so the board is known by its SDIO host's address.
const PI5_GIO: u64 = 0x10_7D50_8500;
const GIO_DATA: usize = 0x04;
const GIO_IODIR: usize = 0x08;
const PI5_WL_ON: u32 = 1 << 28;
const WL_OFF_MS: i64 = 20;
const WL_ON_MS: i64 = 150;

static FDO: AtomicPtr<DEVICE_OBJECT> = AtomicPtr::new(null_mut());
static LOWER: AtomicPtr<DEVICE_OBJECT> = AtomicPtr::new(null_mut());
/// Written once in START.
static mut CHIP: Option<Chip<Kernel>> = None;

/// The mapped SDHCI window and the ARM generic timer. No interrupt yet: the
/// bring-up polls.
pub struct Kernel {
    regs: *mut u32,
}

impl Platform for Kernel {
    fn rd(&self, off: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.regs.add(off / 4)) }
    }
    fn wr(&self, off: usize, v: u32) {
        unsafe { core::ptr::write_volatile(self.regs.add(off / 4), v) }
    }
    fn now_us(&self) -> u64 {
        let mut freq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        let count = unsafe { KeQueryPerformanceCounter(&mut freq) };
        let f = unsafe { freq.QuadPart }.max(1) as u128;
        (unsafe { count.QuadPart } as u128 * 1_000_000 / f) as u64
    }
    fn stall_us(&self, us: u32) {
        unsafe { KeStallExecutionProcessor(us) }
    }
}

fn sleep_ms(ms: i64) {
    let mut t: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    t.QuadPart = -(ms * 10_000);
    // KernelMode (0), not alertable.
    unsafe { KeDelayExecutionThread(0, 0, &mut t) };
}

unsafe fn map(pa: u64, len: usize) -> *mut u8 {
    let mut p: PHYSICAL_ADDRESS = core::mem::zeroed();
    p.QuadPart = pa as i64;
    MmMapIoSpace(p, len as _, _MEMORY_CACHING_TYPE_MmNonCached) as *mut u8
}

unsafe fn complete(irp: PIRP, status: NTSTATUS) -> NTSTATUS {
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    IoCompleteRequest(irp, 0);
    status
}

pub unsafe extern "system" fn add_device(driver: PDRIVER_OBJECT, pdo: PDEVICE_OBJECT) -> NTSTATUS {
    if !FDO.load(Ordering::Acquire).is_null() {
        bwprint!("a second Wi-Fi chip was ignored\n");
        return STATUS_NO_SUCH_DEVICE;
    }
    let mut fdo: PDEVICE_OBJECT = null_mut();
    let status = IoCreateDevice(driver, 0, null_mut(), FILE_DEVICE_PHYSICAL_NETCARD, FILE_DEVICE_SECURE_OPEN, 0, &mut fdo);
    if status < 0 {
        bwprint!("IoCreateDevice failed {:#010x}\n", status as u32);
        return status;
    }
    let lower = IoAttachDeviceToDeviceStack(fdo, pdo);
    if lower.is_null() {
        IoDeleteDevice(fdo);
        return STATUS_NO_SUCH_DEVICE;
    }
    LOWER.store(lower, Ordering::Release);
    FDO.store(fdo, Ordering::Release);
    (*fdo).Flags &= !DO_DEVICE_INITIALIZING;
    STATUS_SUCCESS
}

pub unsafe extern "system" fn dispatch(device: PDEVICE_OBJECT, irp: PIRP) -> NTSTATUS {
    let lower = LOWER.load(Ordering::Acquire);
    if device != FDO.load(Ordering::Acquire) || lower.is_null() {
        return complete(irp, STATUS_NO_SUCH_DEVICE);
    }
    let io = IoGetCurrentIrpStackLocation(irp);
    match ((*io).MajorFunction, (*io).MinorFunction) {
        (IRP_MJ_PNP, IRP_MN_START_DEVICE) => {
            let status = if IoForwardIrpSynchronously(lower, irp) == 0 {
                STATUS_NO_SUCH_DEVICE
            } else if (*irp).IoStatus.__bindgen_anon_1.Status < 0 {
                (*irp).IoStatus.__bindgen_anon_1.Status
            } else {
                start(io)
            };
            complete(irp, status)
        }
        (IRP_MJ_POWER, _) => {
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

/// Raise WL_REG_ON from low, so the chip starts from its power-on reset
/// whatever the firmware left it in. False when this board's line is unknown.
unsafe fn power_on(host: u64) -> bool {
    if host != PI5_SDIO2 {
        return false;
    }
    let gio = map(PI5_GIO, 0x20) as *mut u32;
    if gio.is_null() {
        bwprint!("could not map the GPIO block at {:#x}\n", PI5_GIO);
        return false;
    }
    let rd = |off: usize| core::ptr::read_volatile(gio.add(off / 4));
    let wr = |off: usize, v: u32| core::ptr::write_volatile(gio.add(off / 4), v);
    let (data, dir) = (rd(GIO_DATA), rd(GIO_IODIR));
    bwprint!(
        "WL_ON (GPIO 28) as the firmware left it: {} {} (DATA {:#010x} IODIR {:#010x})\n",
        if dir & PI5_WL_ON != 0 { "input" } else { "output" },
        if data & PI5_WL_ON != 0 { "high" } else { "low" },
        data,
        dir
    );
    wr(GIO_DATA, data & !PI5_WL_ON);
    wr(GIO_IODIR, dir & !PI5_WL_ON);
    sleep_ms(WL_OFF_MS);
    wr(GIO_DATA, rd(GIO_DATA) | PI5_WL_ON);
    sleep_ms(WL_ON_MS);
    bwprint!("WL_ON raised: DATA {:#010x} IODIR {:#010x}\n", rd(GIO_DATA), rd(GIO_IODIR));
    true
}

unsafe fn start(io: PIO_STACK_LOCATION) -> NTSTATUS {
    let list = (*io).Parameters.StartDevice.AllocatedResourcesTranslated;
    if list.is_null() || (*list).Count == 0 {
        bwprint!("START_DEVICE carried no resources\n");
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    }
    let partial = &(*(*list).List.as_ptr()).PartialResourceList;
    let descs = core::slice::from_raw_parts(partial.PartialDescriptors.as_ptr(), partial.Count as usize);
    let Some(mem) = descs.iter().find(|d| d.Type == CM_RESOURCE_TYPE_MEMORY) else {
        bwprint!("START_DEVICE carried no register window\n");
        return STATUS_DEVICE_CONFIGURATION_ERROR;
    };
    let pa = mem.u.Memory.Start.QuadPart as u64;
    let len = mem.u.Memory.Length as usize;
    let regs = map(pa, len);
    if regs.is_null() {
        bwprint!("could not map the SDIO host at {:#x}\n", pa);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let p = Kernel { regs: regs.cast() };
    bwprint!(
        "SDIO host {:#x}+{:#x}: version {:#x}, caps {:#010x}, firmware left clock {:#06x} host {:#010x}\n",
        pa,
        len,
        p.rd(sdhci_rust::REG_VERSION) >> 16,
        p.rd(sdhci_rust::REG_CAPS),
        p.rd(sdhci_rust::REG_CLOCK) & 0xFFFF,
        p.rd(sdhci_rust::REG_HOST_CTRL)
    );
    // Nothing may signal: this driver polls.
    p.wr(sdhci_rust::REG_INT_SIGNAL, 0);
    if !power_on(pa) {
        bwprint!("no known power line on this board: trying the chip as it is\n");
    }

    let mut sdio = Sdio::new(p);
    let card = match sdio.init() {
        Ok(c) => c,
        Err(e) => {
            // No chip answering: the device stays started, with no network.
            bwprint!("no SDIO card: {:?}\n", e);
            return STATUS_SUCCESS;
        }
    };
    bwprint!(
        "SDIO card {:04x}:{:04x}, RCA {:#06x}, {} functions, OCR {:#08x}, CCCR rev {:#04x}, {} kHz 4-bit\n",
        card.vendor,
        card.device,
        card.rca,
        card.functions,
        card.ocr,
        card.cccr_rev,
        card.clock_hz / 1000
    );
    match Chip::attach(sdio) {
        Ok(chip) => {
            let id = chip.id;
            bwprint!(
                "chip {:#06x} rev {} package {} cores {} (chip id {:#010x}, {})\n",
                id.chip,
                id.rev,
                id.package,
                id.cores,
                id.raw,
                if id.chip == 0x4345 { "CYW43455" } else { "not a CYW43455" }
            );
            for c in &chip.cores[..chip.ncores] {
                bwprint!("  core {:#05x} rev {:2} base {:#010x} wrap {:#010x}\n", c.id, c.rev, c.base, c.wrap);
            }
            CHIP = Some(chip);
        }
        Err((_, e)) => bwprint!("chip did not attach: {:?}\n", e),
    }
    STATUS_SUCCESS
}
