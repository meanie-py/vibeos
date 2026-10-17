//! bwfm.sys's device stack: our FDO on the HAL's `VIBEOS\CYW43455` PDO (the Pi
//! 5's SDC1, hal-rust/src/arm64/pnpdd.rs), bound through the critical device
//! database.
//!
//! START powers the chip, brings the SDIO bus up, and reads the chip id and its
//! cores. A worker thread then loads the firmware from
//! `\SystemRoot\System32\drivers\cyw43455\` (staged only by a build with
//! `VIBEOS_PAYLOAD=piwifi`: VibeOS does not ship it) and starts it; function 2
//! coming ready is the firmware running. docs/engineering/pi-wifi-design.md.
//! Every other IRP passes down untouched.

use core::ptr::null_mut;
use core::sync::atomic::{AtomicPtr, Ordering};

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePoolWithTag, IoAttachDeviceToDeviceStack, IoCreateDevice, IoDeleteDevice,
    IoForwardIrpSynchronously, KeDelayExecutionThread, KeQueryPerformanceCounter, KeStallExecutionProcessor,
    MmMapIoSpace, PsCreateSystemThread, ZwClose, ZwCreateFile, ZwQueryInformationFile, ZwReadFile,
    _FILE_INFORMATION_CLASS_FileStandardInformation, _MEMORY_CACHING_TYPE_MmNonCached, DEVICE_OBJECT,
    FILE_STANDARD_INFORMATION, HANDLE, IO_STATUS_BLOCK, LARGE_INTEGER,
    NTSTATUS, OBJECT_ATTRIBUTES, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIO_STACK_LOCATION, PIRP, PVOID,
    ULONG, UNICODE_STRING,
};
use ntkrnl_sys::driver_ddk::{IoCallDriver, IoCompleteRequest, PoCallDriver, PoStartNextPowerIrp};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use sdhci_rust::Platform;

use crate::bus::{Buffers, Bus};
use crate::chip::Chip;
use crate::join::{JoinEnd, JoinLog, WifiConfig};
use crate::scan::Bss;
use crate::nvram;
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
/// Written once in START, then owned by the firmware worker, which moves it into
/// BUS once the firmware runs.
static mut CHIP: Option<Chip<Kernel>> = None;
static mut BUS: Option<Bus<Kernel>> = None;
static mut BUFFERS: Buffers = Buffers::new();

const BWFM_POOL_TAG: ULONG = u32::from_le_bytes(*b"Bwfm");
const NON_PAGED_POOL: i32 = 0;
const THREAD_ALL_ACCESS: ULONG = 0x001F_03FF;
/// The files, under \SystemRoot\System32\drivers\cyw43455\, as
/// tools/fetch_pi_wifi_firmware.py names them.
const FIRMWARE: &str = "\\SystemRoot\\System32\\drivers\\cyw43455\\cyfmac43455-sdio.bin";
const NVRAM: &str = "\\SystemRoot\\System32\\drivers\\cyw43455\\brcmfmac43455-sdio.txt";
const CLM: &str = "\\SystemRoot\\System32\\drivers\\cyw43455\\cyfmac43455-sdio.clm_blob";
/// The network to join, staged by VIBEOS_WIFI_CONFIG (the WDI path reads the same file).
const WIFI_CONFIG: &str = "\\SystemRoot\\bin\\wifi.txt";
/// The firmware is ~600 KiB and the NVRAM ~2 KiB; anything far past that is not them.
const FILE_MAX: usize = 1 << 20;
/// How long the worker waits for the system volume to answer for the files.
const FILE_WAIT_S: u32 = 120;

/// The mapped SDHCI window and the ARM generic timer. No interrupt yet: the
/// bring-up polls.
pub struct Kernel {
    regs: *mut u32,
}

impl Kernel {
    fn now_us_static() -> u64 {
        let mut freq: LARGE_INTEGER = unsafe { core::mem::zeroed() };
        let count = unsafe { KeQueryPerformanceCounter(&mut freq) };
        let f = unsafe { freq.QuadPart }.max(1) as u128;
        (unsafe { count.QuadPart } as u128 * 1_000_000 / f) as u64
    }
}

impl Platform for Kernel {
    fn rd(&self, off: usize) -> u32 {
        unsafe { core::ptr::read_volatile(self.regs.add(off / 4)) }
    }
    fn wr(&self, off: usize, v: u32) {
        unsafe { core::ptr::write_volatile(self.regs.add(off / 4), v) }
    }
    fn now_us(&self) -> u64 {
        Kernel::now_us_static()
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

/// A file read whole into pool (`path` from `\SystemRoot`), or the status that
/// stopped it. The caller frees it with BWFM_POOL_TAG.
unsafe fn read_driver_file(path: &str) -> Result<(*mut u8, usize), NTSTATUS> {
    let mut name = [0u16; 96];
    let mut len = 0;
    for c in path.encode_utf16() {
        if len == name.len() - 1 {
            return Err(STATUS_INSUFFICIENT_RESOURCES);
        }
        name[len] = c;
        len += 1;
    }
    let mut us: UNICODE_STRING = core::mem::zeroed();
    us.Length = (len * 2) as u16;
    us.MaximumLength = (len * 2 + 2) as u16;
    us.Buffer = name.as_mut_ptr();
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = &mut us;
    oa.Attributes = 0x40 | 0x200; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut h: HANDLE = null_mut();
    let s = ZwCreateFile(
        &mut h,
        0x8012_0089, // GENERIC_READ | SYNCHRONIZE
        &mut oa,
        &mut iosb,
        null_mut(),
        0x80, // FILE_ATTRIBUTE_NORMAL
        0x1,  // FILE_SHARE_READ
        0x1,  // FILE_OPEN
        0x20 | 0x40, // FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE
        null_mut(),
        0,
    );
    if s < 0 {
        return Err(s);
    }
    let mut info: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let s = ZwQueryInformationFile(
        h,
        &mut iosb,
        (&raw mut info).cast(),
        core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
        _FILE_INFORMATION_CLASS_FileStandardInformation,
    );
    let size = info.EndOfFile.QuadPart as usize;
    if s < 0 || size == 0 || size > FILE_MAX {
        ZwClose(h);
        return Err(if s < 0 { s } else { STATUS_DEVICE_CONFIGURATION_ERROR });
    }
    // Room for nvram::convert's padding and token after the text.
    let buf = ExAllocatePoolWithTag(NON_PAGED_POOL as _, (size + nvram::SLACK) as _, BWFM_POOL_TAG) as *mut u8;
    if buf.is_null() {
        ZwClose(h);
        return Err(STATUS_INSUFFICIENT_RESOURCES);
    }
    let s = ZwReadFile(h, null_mut(), None, null_mut(), &mut iosb, buf as PVOID, size as u32, null_mut(), null_mut());
    ZwClose(h);
    if s < 0 || iosb.Information as usize != size {
        ExFreePoolWithTag(buf.cast(), BWFM_POOL_TAG);
        return Err(if s < 0 { s } else { STATUS_DEVICE_CONFIGURATION_ERROR });
    }
    Ok((buf, size))
}

/// Wait for the files, convert the NVRAM, load the firmware and start it.
unsafe extern "system" fn firmware_worker(_context: PVOID) {
    let Some(chip) = (*(&raw mut CHIP)).as_mut() else { return };
    let mut waited = 0;
    let (fw, nv) = loop {
        match (read_driver_file(FIRMWARE), read_driver_file(NVRAM)) {
            (Ok(f), Ok(n)) => break (f, n),
            (f, n) => {
                // One may be readable before the other: free it, retry both.
                for r in [f, n] {
                    if let Ok((p, _)) = r {
                        ExFreePoolWithTag(p.cast(), BWFM_POOL_TAG);
                    }
                }
                if waited >= FILE_WAIT_S {
                    bwprint!(
                        "no firmware in \\SystemRoot\\System32\\drivers\\cyw43455 after {} s: VibeOS does not ship it; \
                         build with VIBEOS_PAYLOAD=piwifi to fetch and stage it\n",
                        FILE_WAIT_S
                    );
                    return;
                }
                sleep_ms(1000);
                waited += 1;
            }
        }
    };
    let firmware = core::slice::from_raw_parts(fw.0, fw.1);
    let text = core::slice::from_raw_parts(nv.0, nv.1);
    let mut blob = [0u8; 4096];
    let loaded = match nvram::convert(text, &mut blob) {
        None => {
            bwprint!("NVRAM file is {} bytes: too big to convert\n", nv.1);
            None
        }
        Some(n) => {
            let t0 = Kernel::now_us_static();
            let r = chip.load_firmware(firmware, &blob[..n]);
            Some((r, Kernel::now_us_static().wrapping_sub(t0)))
        }
    };
    ExFreePoolWithTag(fw.0.cast(), BWFM_POOL_TAG);
    ExFreePoolWithTag(nv.0.cast(), BWFM_POOL_TAG);
    match loaded {
        None => {}
        Some((Err(e), _)) => bwprint!("firmware load failed: {:?}\n", e),
        Some((Ok(l), us)) => {
            bwprint!(
                "firmware loaded and verified: {} bytes at {:#x}, NVRAM {} bytes at {:#x}, reset vector {:#010x}, {} ms\n",
                l.firmware,
                chip.rambase,
                l.nvram,
                l.nvram_at,
                l.reset_vector,
                us / 1000
            );
            let t0 = Kernel::now_us_static();
            match chip.start() {
                Ok(()) => {
                    bwprint!(
                        "firmware running: function 2 ready after {} ms\n",
                        Kernel::now_us_static().wrapping_sub(t0) / 1000
                    );
                    if let Some(chip) = (*(&raw mut CHIP)).take() {
                        talk(chip);
                    }
                }
                Err(e) => bwprint!("firmware did not come up: {:?}\n", e),
            }
        }
    }
}

/// The first exchange with the running firmware: its version string over BCDC.
unsafe fn talk(chip: Chip<Kernel>) {
    let bus = (*(&raw mut BUS)).insert(Bus::new(chip, &mut *(&raw mut BUFFERS)));
    let mut events = 0u32;
    let mut out = [0u8; 256];
    match bus.get_var(b"ver", &mut out, |_| events += 1) {
        Ok(n) => {
            let text = &out[..out[..n].iter().position(|&c| c == 0).unwrap_or(n)];
            // The firmware ends the string with a newline; one line in the log.
            let text = text.strip_suffix(b"\n").unwrap_or(text);
            bwprint!(
                "firmware says: {} ({} frames, {} other frames before the reply, credit window {})\n",
                core::str::from_utf8(text).unwrap_or("<not UTF-8>"),
                bus.frames,
                events,
                bus.credit.window()
            );
        }
        Err(e) => {
            bwprint!(
                "no answer to `ver`: {:?} ({} frames read, {} dropped, credit seq {} max {})\n",
                e,
                bus.frames,
                bus.dropped,
                bus.credit.seq,
                bus.credit.max_seq
            );
            return;
        }
    }
    match bus.read_mac(|_| events += 1) {
        Ok(mac) => bwprint!(
            "MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
            mac[0],
            mac[1],
            mac[2],
            mac[3],
            mac[4],
            mac[5]
        ),
        Err(e) => bwprint!("no MAC address: {:?}\n", e),
    }
    // The regulatory table: without it the firmware will not scan.
    match read_driver_file(CLM) {
        Err(s) => bwprint!("no CLM blob ({:#010x}): the firmware cannot scan\n", s as u32),
        Ok((p, n)) => {
            let r = bus.load_clm(core::slice::from_raw_parts(p, n), |_| events += 1);
            ExFreePoolWithTag(p.cast(), BWFM_POOL_TAG);
            let mut status = [0u8; 4];
            let st = bus.get_var(b"clmload_status", &mut status, |_| events += 1);
            match (r, st) {
                (Ok(chunks), Ok(_)) => bwprint!(
                    "CLM loaded: {} bytes in {} chunks, clmload_status {}\n",
                    n,
                    chunks,
                    u32::from_le_bytes(status)
                ),
                (Ok(chunks), Err(e)) => {
                    bwprint!("CLM sent ({} bytes, {} chunks), status unreadable: {:?}\n", n, chunks, e)
                }
                (Err(e), _) => bwprint!("CLM load failed: {:?}\n", e),
            }
        }
    }
    if let Err(e) = bus.bring_up(|_| events += 1) {
        bwprint!("could not bring the interface up: {:?}\n", e);
        return;
    }
    // The network to join, if this image carries one (VIBEOS_WIFI_CONFIG).
    let cfg = read_driver_file(WIFI_CONFIG).ok().and_then(|(p, n)| {
        let c = WifiConfig::parse(core::slice::from_raw_parts(p, n));
        core::ptr::write_bytes(p, 0, n); // the passphrase does not linger in pool
        ExFreePoolWithTag(p.cast(), BWFM_POOL_TAG);
        c
    });
    let mut best: Option<Bss> = None;
    // Each network once, however many times the scan reports it.
    let mut seen = [[0u8; 6]; 32];
    let (mut n, mut reports) = (0usize, 0u32);
    let t0 = Kernel::now_us_static();
    let end = bus.scan(|b| {
        reports += 1;
        if let Some(c) = &cfg {
            if b.ssid() == c.ssid() && best.is_none_or(|x| b.rssi > x.rssi) {
                best = Some(*b);
            }
        }
        if seen[..n].contains(&b.bssid) || n == seen.len() {
            return;
        }
        seen[n] = b.bssid;
        n += 1;
        let mut name = [0u8; 32];
        let ssid = b.ssid();
        for (d, &c) in name.iter_mut().zip(ssid) {
            *d = if (0x20..0x7F).contains(&c) { c } else { b'?' };
        }
        let name = core::str::from_utf8(&name[..ssid.len()]).unwrap_or("");
        bwprint!(
            "  {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} ch {:3} {:4} dBm  {}\n",
            b.bssid[0],
            b.bssid[1],
            b.bssid[2],
            b.bssid[3],
            b.bssid[4],
            b.bssid[5],
            b.channel,
            b.rssi,
            if ssid.is_empty() { "<hidden>" } else { name }
        );
    });
    match end {
        Ok(e) => bwprint!(
            "scan {:?} in {} ms: {} networks, {} reports\n",
            e,
            Kernel::now_us_static().wrapping_sub(t0) / 1000,
            n,
            reports
        ),
        Err(e) => bwprint!("scan failed after {} networks: {:?}\n", n, e),
    }
    let Some(cfg) = cfg else {
        bwprint!("no \\SystemRoot\\bin\\wifi.txt (ssid=, psk=): no network to join\n");
        bwprint!("{} frames other than replies so far\n", events);
        return;
    };
    let ssid = core::str::from_utf8(cfg.ssid()).unwrap_or("?");
    if best.is_none() {
        // A hidden network answers only a scan that names it.
        bwprint!("'{}' not heard{}: asking for it by name\n", ssid, if cfg.hidden { " (hidden=1)" } else { "" });
        let r = bus.scan_for(Some(cfg.ssid()), |b| {
            if b.ssid() == cfg.ssid() && best.is_none_or(|x| b.rssi > x.rssi) {
                best = Some(*b);
            }
        });
        if let Err(e) = r {
            bwprint!("directed scan failed: {:?}\n", e);
        }
    }
    let Some(target) = best else {
        bwprint!("'{}' is not in the scan: nothing to join\n", ssid);
        return;
    };
    bwprint!(
        "joining '{}' at {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} (ch {}, {} dBm)\n",
        ssid,
        target.bssid[0],
        target.bssid[1],
        target.bssid[2],
        target.bssid[3],
        target.bssid[4],
        target.bssid[5],
        target.channel,
        target.rssi
    );
    let mut log = JoinLog::default();
    let t0 = Kernel::now_us_static();
    let end = bus.join(&cfg, &target, &mut log, |_| events += 1);
    let ms = Kernel::now_us_static().wrapping_sub(t0) / 1000;
    if log.optional_refused != 0 {
        bwprint!(
            "firmware refused optional settings (bit 0 sup_wpa, bit 1 mfp): {:#x}, last {:?}\n",
            log.optional_refused,
            log.optional_error
        );
    }
    if log.assoc_rsn_len > 0 {
        let mut hex = [0u8; 128];
        for (i, b) in log.assoc_rsn[..log.assoc_rsn_len].iter().enumerate() {
            hex[2 * i] = b"0123456789abcdef"[(b >> 4) as usize];
            hex[2 * i + 1] = b"0123456789abcdef"[(b & 15) as usize];
        }
        bwprint!(
            "association request RSN {} ({})\n",
            core::str::from_utf8(&hex[..2 * log.assoc_rsn_len]).unwrap_or("?"),
            if log.assoc_rsn_differs { "not ours: message 2 used it" } else { "ours" }
        );
    }
    match end {
        Ok(JoinEnd::Up) => bwprint!(
            "4-way handshake complete in {} ms: link up, keys installed ({} EAPOL in, {} out)\n",
            ms,
            log.eapol_in,
            log.eapol_out
        ),
        Ok(e) => bwprint!(
            "join ended {:?} after {} ms (associated {}, {} EAPOL in, {} out, {} rejected, last {:?})\n",
            e,
            ms,
            log.associated,
            log.eapol_in,
            log.eapol_out,
            log.rejected,
            log.last_reject
        ),
        Err(e) => bwprint!("join failed: {:?}\n", e),
    }
    bwprint!("{} frames other than replies so far\n", events);
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
            bwprint!(
                "CR4 halted; RAM {} KiB at {:#x}; chip-common caps {:#010x}, PMU rev {}\n",
                chip.ramsize / 1024,
                chip.rambase,
                chip.cc_caps,
                chip.pmurev
            );
            CHIP = Some(chip);
            let mut thread: HANDLE = null_mut();
            let s = PsCreateSystemThread(
                &mut thread,
                THREAD_ALL_ACCESS,
                null_mut(),
                null_mut(),
                null_mut(),
                Some(firmware_worker),
                null_mut(),
            );
            if s < 0 {
                bwprint!("PsCreateSystemThread failed {:#010x}: no firmware\n", s as u32);
            } else {
                ZwClose(thread);
            }
        }
        Err((_, e)) => bwprint!("chip did not attach: {:?}\n", e),
    }
    STATUS_SUCCESS
}
