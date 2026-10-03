//! ARM64 HAL stubs translated from `hal/halarm64/generic/halstubs.c`.
//!
//! These are intentionally minimal. They preserve the C stub behavior used by
//! the current QEMU ARM64 boot path: enough HAL surface to link, basic IRQL
//! bookkeeping, software APC dispatch, spinlock/fast-mutex helpers, port
//! MMIO shims, and the ARM64 virtual counter performance clock.
//!
//! "Minimal" stopped being free once `halmp.dll` shipped: this file is built
//! twice, one `smp` feature apart, so a stub that was adequate for a UP boot is
//! now also the MP implementation. The spinlock helpers below were exactly that
//! and had to be made real. Prefer a `#[cfg(feature = "smp")]` over a comment
//! asserting what the build is — see the i386 twin of that mistake in the field
//! manual, under carrying a build-configuration `#ifdef` into the Rust.

#![cfg(target_arch = "aarch64")]
#![allow(unused_variables)]

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

const TRUE: ddk::BOOLEAN = 1;
const FALSE: ddk::BOOLEAN = 0;
const STATUS_SUCCESS: ddk::NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: ddk::NTSTATUS = 0xC0000002u32 as ddk::NTSTATUS;
const ESUCCESS: ddk::ARC_STATUS = ddk::_ARC_CODES_ESUCCESS as ddk::ARC_STATUS;

const PASSIVE_LEVEL: ddk::KIRQL = 0;
const APC_LEVEL: ddk::KIRQL = 1;
const DISPATCH_LEVEL: ddk::KIRQL = 2;
/// IRQL the HAL synchronizes at — must agree with the kernel-side SYNCH_LEVEL
/// (ntoskrnl-rust ke/ke_defs.rs, ntkrnl-sys inline/ke.rs) and with
/// `ndk/arm64/ketypes.h:29`. UP: DISPATCH. MP: IPI_LEVEL(14) - 2 = 12, which
/// sits above every DIRQL (max 11) and below CLOCK_LEVEL(13) — device lines
/// masked, scheduler tick and IPIs still delivered. See arm64/clock.rs's
/// IRQL↔GIC priority map; 12 is exactly the gap it already leaves free.
#[cfg(not(feature = "smp"))]
const SYNCH_LEVEL: ddk::KIRQL = DISPATCH_LEVEL;
#[cfg(feature = "smp")]
const SYNCH_LEVEL: ddk::KIRQL = 14 - 2;

static mut HalpCurrentTimeIncrement: ddk::ULONG = 100000;
static mut HalpNextTimeIncrement: ddk::ULONG = 100000;
static mut HalpNextIntervalCount: ddk::ULONG = 0;

// ---------------------------------------------------------------------------
// Per-CPU IRQL + software-interrupt state (SMP — smp-bringup-quest.md M5b).
//
// The soft IRQL lives in the KPCR (KPCR.CurrentIrql via tpidr_el1) — the same
// slot KiInitializePcr initializes for every CPU — with a boot-global fallback
// for the pre-PCR window (KiSystemStartup before `msr tpidr_el1`: single CPU,
// IRQs hard-masked). The softint pending/in-progress masks and the exception-
// window depth are banked per CPU: with one global, CPU0's clock tick would
// gate CPU1's softint dispatch, and CPU1's KfLowerIrql drain would consume
// (and clear) a DISPATCH request CPU0 posted for itself — a lost-DPC hang.
// ---------------------------------------------------------------------------

/// Pre-PCR boot IRQL slot (BSP only, before tpidr_el1 is programmed).
static mut HalpBootIrql: ddk::KIRQL = PASSIVE_LEVEL;

const HALP_MAX_CPUS: usize = 64;

/// One CPU's softint/exception-window state, padded to its own cache lines.
#[repr(C, align(128))]
pub(crate) struct HalpCpuState {
    /// Pending software-interrupt bits (1 << APC_LEVEL / 1 << DISPATCH_LEVEL).
    pub(crate) soft_int_pending: AtomicU32,
    /// Reentrancy latch for HalpDispatchSoftwareInterrupts on this CPU.
    pub(crate) soft_int_in_progress: AtomicI32,
    /// Nesting depth of exception-context interrupt dispatch. Bumped for the
    /// whole IRQ-vector body (HalClockInterruptArm64) and again by each
    /// HalpEnterInterruptIrql/HalpExitInterruptIrql bracket (ISR dispatch,
    /// IRQ-exit DPC retire). Non-zero means this CPU is inside an IRQ
    /// exception window — software-interrupt delivery that can wake/swap
    /// threads (DISPATCH work) must NOT run there (§9.19 DAIF-inheritance
    /// class). Notably the clock-tick branch runs KeUpdateSystemTime with NO
    /// IRQL bracket at all, so without the vector-wide bump a raise/lower
    /// pair inside the tick path delivers DISPATCH work in exception context
    /// (BUGCHECK 0x1E, EC=0x0E illegal-execution-state).
    pub(crate) interrupt_context_depth: AtomicU32,
    /// M5c tail: this CPU's virtual timer is armed (banked PPI-27 enabled +
    /// CNTV deadline programmed). APs arm lazily on their first IRQ after
    /// clock init publishes the interval (clock.rs vtimer_percpu_arm).
    pub(crate) vtimer_armed: AtomicU32,
}

#[allow(clippy::declare_interior_mutable_const)]
const HALP_CPU_STATE_INIT: HalpCpuState = HalpCpuState {
    soft_int_pending: AtomicU32::new(0),
    soft_int_in_progress: AtomicI32::new(0),
    interrupt_context_depth: AtomicU32::new(0),
    vtimer_armed: AtomicU32::new(0),
};
static HalpCpuStates: [HalpCpuState; HALP_MAX_CPUS] = [HALP_CPU_STATE_INIT; HALP_MAX_CPUS];

#[inline(always)]
unsafe fn halp_pcr() -> *mut ddk::_KIPCR {
    let pcr: u64;
    core::arch::asm!("mrs {}, tpidr_el1", out(reg) pcr, options(nomem, nostack, preserves_flags));
    pcr as *mut ddk::_KIPCR
}

/// This CPU's banked softint/depth state (index 0 pre-PCR: BSP only).
#[inline(always)]
pub(crate) unsafe fn halp_cpu_state() -> &'static HalpCpuState {
    let pcr = halp_pcr();
    let n = if pcr.is_null() { 0 } else { (*pcr).Prcb.Number as usize };
    &HalpCpuStates[n % HALP_MAX_CPUS]
}

/// This CPU's processor number (0 pre-PCR: BSP only).
#[inline(always)]
pub(crate) unsafe fn halp_cpu_number() -> u32 {
    let pcr = halp_pcr();
    if pcr.is_null() { 0 } else { (*pcr).Prcb.Number as u32 }
}

#[inline(always)]
unsafe fn halp_irql_slot() -> *mut ddk::KIRQL {
    let pcr = halp_pcr();
    if pcr.is_null() {
        core::ptr::addr_of_mut!(HalpBootIrql)
    } else {
        core::ptr::addr_of_mut!((*pcr).CurrentIrql)
    }
}

#[inline(always)]
pub(crate) unsafe fn halp_get_irql() -> ddk::KIRQL {
    core::ptr::read_volatile(halp_irql_slot())
}

#[inline(always)]
unsafe fn halp_set_irql(irql: ddk::KIRQL) {
    core::ptr::write_volatile(halp_irql_slot(), irql)
}

#[unsafe(no_mangle)]
pub static mut KdComPortInUse: ddk::PUCHAR = core::ptr::null_mut();

#[inline]
unsafe fn interlocked_increment(addend: *mut ddk::LONG) -> ddk::LONG {
    let atomic = unsafe { &*(addend as *const AtomicI32) };
    atomic.fetch_add(1, Ordering::SeqCst).wrapping_add(1) as ddk::LONG
}

#[inline]
unsafe fn interlocked_decrement(addend: *mut ddk::LONG) -> ddk::LONG {
    let atomic = unsafe { &*(addend as *const AtomicI32) };
    atomic.fetch_sub(1, Ordering::SeqCst).wrapping_sub(1) as ddk::LONG
}

#[inline]
unsafe fn interlocked_compare_exchange(
    addend: *mut ddk::LONG,
    exchange: ddk::LONG,
    comparand: ddk::LONG,
) -> ddk::LONG {
    let atomic = unsafe { &*(addend as *const AtomicI32) };
    match atomic.compare_exchange(
        comparand as i32,
        exchange as i32,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(old) | Err(old) => old as ddk::LONG,
    }
}

#[inline]
fn software_interrupt_bit(request: ddk::KIRQL) -> u32 {
    1u32 << (request as u32)
}

#[inline]
unsafe fn zero_physical_address() -> ddk::PHYSICAL_ADDRESS {
    ddk::PHYSICAL_ADDRESS { QuadPart: 0 }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitSystem(
    BootPhase: ddk::ULONG,
    LoaderBlock: ddk::PLOADER_PARAMETER_BLOCK,
) -> ddk::BOOLEAN {
    unsafe {
        if BootPhase == 0 {
            super::pnpdd::register(LoaderBlock);
            HalpCurrentTimeIncrement = 100000;
            HalpNextTimeIncrement = 100000;
            HalpNextIntervalCount = 0;
            ddk::KeSetTimeIncrement(100000, 10000);
        }
    }

    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalInitializeProcessor(
    ProcessorNumber: ddk::ULONG,
    LoaderBlock: ddk::PLOADER_PARAMETER_BLOCK,
) {
    // BSP (P0): called in Phase 0, before the GIC matters — its per-CPU GIC
    // init happens in HalInitializeClockArm64 (Phase 1). An AP starts much
    // later (KeStartAllProcessors, Phase 1) and must wire its OWN banked GICv2
    // registers here — SGI enable/priority and the CPU interface are banked
    // per accessing core, so the BSP cannot do it on the AP's behalf. Until
    // the MmMapIoSpace remap (clock init) the GIC is reached through the boot
    // identity mapping (same pattern as Arm64UartKernelBase / fw_cfg; the
    // loader maps the GICD/GICC pages as Device — winldr_arm64.rs).
    if ProcessorNumber != 0 {
        super::clock::gic_ensure_early_bases(LoaderBlock);
        if !super::clock::gic_percpu_init() {
            // Fail loud: an AP with no GIC CPU interface can never receive an
            // IPI — every KeIpiGenericCall would hang the sender.
            DbgPrint(
                b"[SMP] arm64 HalInitializeProcessor: CPU %u GIC init FAILED - IPIs to this CPU are dead\n\0"
                    .as_ptr(),
                ProcessorNumber,
            );
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllProcessorsStarted() -> ddk::BOOLEAN {
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAcquireDisplayOwnership(
    ResetDisplayParameters: ddk::PHAL_RESET_DISPLAY_PARAMETERS,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAdjustResourceList(
    Requirements: *mut ddk::PIO_RESOURCE_REQUIREMENTS_LIST,
) -> ddk::NTSTATUS {
    STATUS_SUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateAdapterChannel(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Wcb: ddk::PWAIT_CONTEXT_BLOCK,
    NumberOfMapRegisters: ddk::ULONG,
    ExecutionRoutine: ddk::PDRIVER_CONTROL,
) -> ddk::NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// HalAllocateCommonBuffer / HalFreeCommonBuffer / HalFlushCommonBuffer /
// HalGetAdapter live in dma.rs now (real coherent-DMA implementations).

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateCrashDumpRegisters(
    AdapterObject: ddk::PADAPTER_OBJECT,
    NumberOfMapRegisters: ddk::PULONG,
) -> ddk::PVOID {
    core::ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAssignSlotResources(
    RegistryPath: ddk::PUNICODE_STRING,
    DriverClassName: ddk::PUNICODE_STRING,
    DriverObject: ddk::PDRIVER_OBJECT,
    DeviceObject: ddk::PDEVICE_OBJECT,
    BusType: ddk::INTERFACE_TYPE,
    BusNumber: ddk::ULONG,
    SlotNumber: ddk::ULONG,
    AllocatedResources: *mut ddk::PCM_RESOURCE_LIST,
) -> ddk::NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalCalibratePerformanceCounter(
    Count: ddk::PLONG,
    NewCount: ddk::ULONGLONG,
    Unused: ddk::ULONGLONG,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalClearSoftwareInterrupt(Request: ddk::KIRQL) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDisableSystemInterrupt(Vector: ddk::ULONG, Irql: ddk::KIRQL) {
    // Clear-enable the device's GIC line (Vector == GIC INTID).
    super::clock::disable_gic_interrupt(Vector as u32);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDisplayString(String: ddk::PCSTR) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ddk::ULONG,
    Irql: ddk::KIRQL,
    InterruptMode: ddk::KINTERRUPT_MODE,
) -> ddk::BOOLEAN {
    // Vector == GIC INTID. Enable the distributor line so the IRQ reaches the
    // CPU interface; the connected ISR is then dispatched by KiIrqHandler →
    // HalClockInterruptArm64 → KiDispatchInterruptArm64. Returns FALSE for an
    // out-of-range / special INTID so KeConnectInterrupt rolls back cleanly.
    if Vector as usize >= 1020 {
        return FALSE;
    }
    super::clock::enable_gic_interrupt(Vector as u32, Irql, InterruptMode == ddk::_KINTERRUPT_MODE_LevelSensitive);
    TRUE
}

// ============================================================================
// PCI configuration space (ECAM) — QEMU `virt` Generic PCIe (GPEX) host bridge.
// ============================================================================
//
// QEMU `-M virt` exposes PCIe config space as a flat ECAM (Enhanced
// Configuration Access Mechanism) MMIO window. Its base depends on the
// machine's `highmem-ecam` setting:
//   * high ECAM: 0x40_1000_0000, 256 MB (buses 0..=255)  — default on aarch64
//   * low  ECAM: 0x0000_3F00_0000, 16 MB (buses 0..=15)
// We probe both: map a bus-0 window and read the host-bridge VID/DID at 00:00.0;
// whichever base reports a present vendor (low 16 bits != 0xFFFF) is the live
// ECAM. Mapping a wrong/unbacked base reads back all-ones on QEMU, so the probe
// is side-effect free.
//
// The only HAL bus-data consumer on arm64 today is netstack's `find_virtio_net`,
// which scans bus 0 exclusively (QEMU keeps the NIC there) — so a single 1 MB
// bus-0 ECAM window is sufficient. Any access with BusNumber != 0, or when no
// ECAM is present, reads back as "device not present" (all-ones).
//
// ECAM address layout within the bus window:
//   offset = (device << 15) | (function << 12) | register   (4 KB per function)
// `SlotNumber` is the kernel PCI_SLOT_NUMBER packing used by the caller:
//   bits 0..4 = device, bits 5..7 = function.

const PCI_CONFIGURATION: ddk::BUS_DATA_TYPE = 4; // _BUS_DATA_TYPE_PCIConfiguration
const ECAM_HIGH_BASE: u64 = 0x40_1000_0000;
const ECAM_LOW_BASE: u64 = 0x0000_3F00_0000;
const ECAM_BUS_SIZE: usize = 0x10_0000; // 1 MB = 256 functions * 4 KB
const ECAM_FN_SIZE: u32 = 0x1000; // 4 KB per function (extended config space)

// Cached mapped VA of the bus-0 ECAM window:
//   0          = not yet probed
//   usize::MAX = probed, no ECAM present
//   otherwise  = mapped kernel VA
static ECAM_BUS0_VA: AtomicUsize = AtomicUsize::new(0);
const ECAM_VA_NONE: usize = usize::MAX;

#[inline]
fn phys(v: u64) -> ddk::PHYSICAL_ADDRESS {
    ddk::PHYSICAL_ADDRESS { QuadPart: v as i64 }
}

/// Lazily map + probe the bus-0 ECAM window. Returns the mapped VA, or null when
/// no PCIe ECAM is present (e.g. a non-virt machine).
unsafe fn ecam_bus0_va() -> *mut u8 {
    let cached = ECAM_BUS0_VA.load(Ordering::Acquire);
    if cached == ECAM_VA_NONE {
        return core::ptr::null_mut();
    }
    if cached != 0 {
        return cached as *mut u8;
    }
    for &base in &[ECAM_HIGH_BASE, ECAM_LOW_BASE] {
        let va = ddk::MmMapIoSpace(
            phys(base),
            ECAM_BUS_SIZE as _,
            ddk::_MEMORY_CACHING_TYPE_MmNonCached as _,
        ) as *mut u8;
        if va.is_null() {
            continue;
        }
        // Host bridge sits at 00:00.0 in either layout; a present vendor id
        // confirms this base is the live ECAM.
        let vid_did = core::ptr::read_volatile(va as *const u32);
        if (vid_did & 0xFFFF) != 0xFFFF {
            ECAM_BUS0_VA.store(va as usize, Ordering::Release);
            return va;
        }
        ddk::MmUnmapIoSpace(va as ddk::PVOID, ECAM_BUS_SIZE as _);
    }
    ECAM_BUS0_VA.store(ECAM_VA_NONE, Ordering::Release);
    core::ptr::null_mut()
}

/// Byte pointer to (device, function) config space within the bus-0 window.
#[inline]
unsafe fn ecam_fn_ptr(va: *mut u8, slot_number: ddk::ULONG) -> *mut u8 {
    let device = slot_number & 0x1F;
    let function = (slot_number >> 5) & 0x7;
    let win = (device << 15) | (function << 12);
    va.add(win as usize)
}

/// Shared read for HalGetBusData / HalGetBusDataByOffset. Copies `Length` bytes
/// of config space at `Offset` into `Buffer`; absent device / no ECAM yields
/// all-ones (the PCI "not present" convention).
unsafe fn ecam_read(
    bus_data_type: ddk::BUS_DATA_TYPE,
    bus_number: ddk::ULONG,
    slot_number: ddk::ULONG,
    buffer: ddk::PVOID,
    offset: ddk::ULONG,
    length: ddk::ULONG,
) -> ddk::ULONG {
    if bus_data_type != PCI_CONFIGURATION || buffer.is_null() || length == 0 {
        return 0;
    }
    let dst = buffer as *mut u8;
    let va = ecam_bus0_va();
    if va.is_null() || bus_number != 0 {
        core::ptr::write_bytes(dst, 0xFF, length as usize);
        return length;
    }
    let src = ecam_fn_ptr(va, slot_number);
    let mut copied = 0u32;
    for i in 0..length {
        let reg = offset + i;
        if reg >= ECAM_FN_SIZE {
            break;
        }
        *dst.add(i as usize) = core::ptr::read_volatile(src.add(reg as usize));
        copied += 1;
    }
    copied
}

/// Shared write for HalSetBusData / HalSetBusDataByOffset.
unsafe fn ecam_write(
    bus_data_type: ddk::BUS_DATA_TYPE,
    bus_number: ddk::ULONG,
    slot_number: ddk::ULONG,
    buffer: ddk::PVOID,
    offset: ddk::ULONG,
    length: ddk::ULONG,
) -> ddk::ULONG {
    if bus_data_type != PCI_CONFIGURATION || buffer.is_null() || length == 0 {
        return 0;
    }
    let va = ecam_bus0_va();
    if va.is_null() || bus_number != 0 {
        return 0;
    }
    let dst = ecam_fn_ptr(va, slot_number);
    let src = buffer as *const u8;
    let mut copied = 0u32;
    for i in 0..length {
        let reg = offset + i;
        if reg >= ECAM_FN_SIZE {
            break;
        }
        core::ptr::write_volatile(dst.add(reg as usize), *src.add(i as usize));
        copied += 1;
    }
    copied
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetBusData(
    BusDataType: ddk::BUS_DATA_TYPE,
    BusNumber: ddk::ULONG,
    SlotNumber: ddk::ULONG,
    Buffer: ddk::PVOID,
    Length: ddk::ULONG,
) -> ddk::ULONG {
    ecam_read(BusDataType, BusNumber, SlotNumber, Buffer, 0, Length)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetBusDataByOffset(
    BusDataType: ddk::BUS_DATA_TYPE,
    BusNumber: ddk::ULONG,
    SlotNumber: ddk::ULONG,
    Buffer: ddk::PVOID,
    Offset: ddk::ULONG,
    Length: ddk::ULONG,
) -> ddk::ULONG {
    ecam_read(BusDataType, BusNumber, SlotNumber, Buffer, Offset, Length)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetEnvironmentVariable(
    Name: ddk::PCH,
    ValueLength: ddk::USHORT,
    Value: ddk::PCH,
) -> ddk::ARC_STATUS {
    ESUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetInterruptVector(
    InterfaceType: ddk::INTERFACE_TYPE,
    BusNumber: ddk::ULONG,
    BusInterruptLevel: ddk::ULONG,
    BusInterruptVector: ddk::ULONG,
    Irql: ddk::PKIRQL,
    Affinity: ddk::PKAFFINITY,
) -> ddk::ULONG {
    // On arm64 the "vector" IS the GIC INTID: drivers pass the device's INTID
    // (QEMU virt: SPI n = INTID 32+n, e.g. UART=33, virtio-mmio slot i=48+i)
    // as the bus interrupt level and/or vector, and get it back verbatim plus
    // the DIRQL it will run at. Valid range: PPIs+SPIs (16..1019); SGIs (<16)
    // and the special INTIDs (>=1020) are not connectable device lines.
    let intid = if BusInterruptVector != 0 {
        BusInterruptVector
    } else {
        BusInterruptLevel
    };
    if !(16..1020).contains(&intid) {
        return 0;
    }
    unsafe {
        if !Irql.is_null() {
            // All device SPIs run at one DIRQL (UP port; per-line DIRQLs are a
            // later refinement) — above DISPATCH, below CLOCK_LEVEL, so a DIRQL
            // raise masks every device but never the system tick.
            *Irql = super::clock::DEVICE_IRQL;
        }
        if !Affinity.is_null() {
            *Affinity = 1;
        }
    }
    intid
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalMakeBeep(Frequency: ddk::ULONG) -> ddk::BOOLEAN {
    FALSE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalProcessorIdle() {
    unsafe {
        core::arch::asm!("wfi", options(nomem, nostack));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalQueryDisplayParameters(
    WidthInCharacters: ddk::PULONG,
    HeightInLines: ddk::PULONG,
    CursorColumn: ddk::PULONG,
    CursorRow: ddk::PULONG,
) {
    unsafe {
        if !WidthInCharacters.is_null() {
            *WidthInCharacters = 80;
        }
        if !HeightInLines.is_null() {
            *HeightInLines = 25;
        }
        if !CursorColumn.is_null() {
            *CursorColumn = 0;
        }
        if !CursorRow.is_null() {
            *CursorRow = 0;
        }
    }
}

/// QEMU virt PL031 RTC: DR register at +0x00 holds seconds since the Unix
/// epoch (UTC). Lazily mapped, same pattern as `ecam_bus0_va`.
const PL031_BASE: u64 = 0x0901_0000;
static PL031_VA: AtomicUsize = AtomicUsize::new(0);
const PL031_VA_NONE: usize = usize::MAX;

unsafe fn pl031_va() -> *mut u32 {
    let cached = PL031_VA.load(Ordering::Acquire);
    if cached == PL031_VA_NONE {
        return core::ptr::null_mut();
    }
    if cached != 0 {
        return cached as *mut u32;
    }
    let va = unsafe {
        ddk::MmMapIoSpace(
            phys(PL031_BASE),
            0x1000,
            ddk::_MEMORY_CACHING_TYPE_MmNonCached as _,
        )
    } as *mut u32;
    if va.is_null() {
        PL031_VA.store(PL031_VA_NONE, Ordering::Release);
        return core::ptr::null_mut();
    }
    PL031_VA.store(va as usize, Ordering::Release);
    va
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalQueryRealTimeClock(Time: ddk::PTIME_FIELDS) -> ddk::BOOLEAN {
    if Time.is_null() {
        return TRUE;
    }

    unsafe {
        let rtc = pl031_va();
        if rtc.is_null() {
            // No PL031 (or MmMapIoSpace not up yet) — be loud, then fall back
            // to a fixed date so callers still get *a* sane time.
            DbgPrint(b"[HAL] HalQueryRealTimeClock: PL031 unavailable, using fallback date\n\0".as_ptr());
            (*Time).Year = 2025;
            (*Time).Month = 1;
            (*Time).Day = 1;
            (*Time).Hour = 0;
            (*Time).Minute = 0;
            (*Time).Second = 0;
            (*Time).Milliseconds = 0;
            (*Time).Weekday = 0;
            return TRUE;
        }

        let secs = core::ptr::read_volatile(rtc) as u64; // seconds since 1970-01-01 UTC

        let days = secs / 86_400;
        let rem = secs % 86_400;
        (*Time).Hour = (rem / 3600) as i16;
        (*Time).Minute = ((rem % 3600) / 60) as i16;
        (*Time).Second = (rem % 60) as i16;
        (*Time).Milliseconds = 0;
        (*Time).Weekday = ((days + 4) % 7) as i16; // 1970-01-01 was a Thursday

        // Civil-date conversion (Howard Hinnant's algorithm), epoch 1970-01-01.
        let z = days as i64 + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097) as u64;          // [0, 146096]
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
        let y = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
        let mp = (5 * doy + 2) / 153;                      // [0, 11]
        let d = doy - (153 * mp + 2) / 5 + 1;              // [1, 31]
        let m = if mp < 10 { mp + 3 } else { mp - 9 };     // [1, 12]
        (*Time).Year = (if m <= 2 { y + 1 } else { y }) as i16;
        (*Time).Month = m as i16;
        (*Time).Day = d as i16;
    }

    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReadDmaCounter(AdapterObject: ddk::PADAPTER_OBJECT) -> ddk::ULONG {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReportResourceUsage() {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestIpi(TargetProcessors: ddk::KAFFINITY) {
    // NT IPI = SGI 0 to the target CPUs (GICv2 SGIR; KAFFINITY bit N == GIC
    // CPU interface N on QEMU virt, where MPIDR.Aff0 == the boot CPU index).
    super::clock::gic_send_sgi(TargetProcessors as u64);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestSoftwareInterrupt(Request: ddk::KIRQL) {
    // Per-CPU: a software interrupt targets the REQUESTING processor (the
    // cross-CPU flavor is an IPI — KiIpiServiceRoutine calls this on the
    // receiving CPU, so the bit lands on the right bank).
    halp_cpu_state()
        .soft_int_pending
        .fetch_or(software_interrupt_bit(Request), Ordering::SeqCst);

    unsafe {
        let irql = halp_get_irql();
        if irql < APC_LEVEL && Request == APC_LEVEL {
            HalpDispatchSoftwareInterrupts(APC_LEVEL, irql);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReturnToFirmware(Action: ddk::_FIRMWARE_REENTRY) -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi", options(nomem, nostack));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetBusData(
    BusDataType: ddk::BUS_DATA_TYPE,
    BusNumber: ddk::ULONG,
    SlotNumber: ddk::ULONG,
    Buffer: ddk::PVOID,
    Length: ddk::ULONG,
) -> ddk::ULONG {
    ecam_write(BusDataType, BusNumber, SlotNumber, Buffer, 0, Length)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetBusDataByOffset(
    BusDataType: ddk::BUS_DATA_TYPE,
    BusNumber: ddk::ULONG,
    SlotNumber: ddk::ULONG,
    Buffer: ddk::PVOID,
    Offset: ddk::ULONG,
    Length: ddk::ULONG,
) -> ddk::ULONG {
    ecam_write(BusDataType, BusNumber, SlotNumber, Buffer, Offset, Length)
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetDisplayParameters(
    CursorColumn: ddk::ULONG,
    CursorRow: ddk::ULONG,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetEnvironmentVariable(
    Name: ddk::PCH,
    Value: ddk::PCH,
) -> ddk::ARC_STATUS {
    ESUCCESS
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetProfileInterval(Interval: ddk::ULONG_PTR) -> ddk::ULONG_PTR {
    Interval
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetRealTimeClock(Time: ddk::PTIME_FIELDS) -> ddk::BOOLEAN {
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSetTimeIncrement(Increment: ddk::ULONG) -> ddk::ULONG {
    unsafe {
        HalpNextTimeIncrement = Increment;
        HalpCurrentTimeIncrement
    }
}

// SMP HAL (ntkrnlmp): the real PSCI CPU_ON body lives in mpboot.rs. UP HAL keeps
// this FALSE stub (no APs to start) — but a CALL here means an MP kernel is
// running on a UP hal.dll (the smp HAL flavor rides the VIBEOS_SMP_DEFAULT_BOOT
// cmake cache var, so a plain UP `build` reconfigure silently flips hal.dll
// back to UP and a later `ninja ntkrnlmp` inherits it; arm64 has no
// MISMATCHED_HAL 0x79 handshake to catch the pair). Fail loud, not silent
// 1-CPU degrade — this exact mismatch cost an M5c debug round (2026-07-14).
#[cfg(not(feature = "smp"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartNextProcessor(
    LoaderBlock: ddk::PLOADER_PARAMETER_BLOCK,
    ProcessorState: ddk::PKPROCESSOR_STATE,
) -> ddk::BOOLEAN {
    DbgPrint(
        b"[SMP] arm64 HalStartNextProcessor: UP HAL stub called - MP kernel on a UP hal.dll? Rebuild with --smp-kernel (VIBEOS_SMP_DEFAULT_BOOT)\n\0"
            .as_ptr(),
    );
    FALSE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartProfileInterrupt(ProfileSource: ddk::KPROFILE_SOURCE) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStopProfileInterrupt(ProfileSource: ddk::KPROFILE_SOURCE) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSystemVectorDispatchEntry(
    Vector: ddk::ULONG,
    FlatDispatch: *mut *mut ddk::PKINTERRUPT_ROUTINE,
    NoConnection: *mut ddk::PKINTERRUPT_ROUTINE,
) -> ddk::UCHAR {
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalTranslateBusAddress(
    InterfaceType: ddk::INTERFACE_TYPE,
    BusNumber: ddk::ULONG,
    BusAddress: ddk::PHYSICAL_ADDRESS,
    AddressSpace: ddk::PULONG,
    TranslatedAddress: ddk::PPHYSICAL_ADDRESS,
) -> ddk::BOOLEAN {
    unsafe {
        if !TranslatedAddress.is_null() {
            (*TranslatedAddress).QuadPart = BusAddress.QuadPart;
        }
    }

    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFlushAdapterBuffers(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Mdl: ddk::PMDL,
    MapRegisterBase: ddk::PVOID,
    CurrentVa: ddk::PVOID,
    Length: ddk::ULONG,
    WriteToDevice: ddk::BOOLEAN,
) -> ddk::BOOLEAN {
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFreeAdapterChannel(AdapterObject: ddk::PADAPTER_OBJECT) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFreeMapRegisters(
    AdapterObject: ddk::PADAPTER_OBJECT,
    MapRegisterBase: ddk::PVOID,
    NumberOfMapRegisters: ddk::ULONG,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoMapTransfer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Mdl: ddk::PMDL,
    MapRegisterBase: ddk::PVOID,
    CurrentVa: ddk::PVOID,
    Length: ddk::PULONG,
    WriteToDevice: ddk::BOOLEAN,
) -> ddk::PHYSICAL_ADDRESS {
    unsafe { zero_physical_address() }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeFlushWriteBuffer() {
    unsafe {
        core::arch::asm!("dsb sy", options(nostack, preserves_flags));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeQueryPerformanceCounter(
    PerformanceFrequency: ddk::PLARGE_INTEGER,
) -> ddk::LARGE_INTEGER {
    let mut count: u64;
    let mut freq: u64;

    unsafe {
        core::arch::asm!("mrs {0}, cntvct_el0", out(reg) count, options(nomem, nostack, preserves_flags));
        if !PerformanceFrequency.is_null() {
            core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) freq, options(nomem, nostack, preserves_flags));
            (*PerformanceFrequency).QuadPart = freq as ddk::LONGLONG;
        }
    }

    ddk::LARGE_INTEGER {
        QuadPart: count as ddk::LONGLONG,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeStallExecutionProcessor(MicroSeconds: ddk::ULONG) {
    let freq: u64;
    let start: u64;

    unsafe {
        core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) freq, options(nomem, nostack, preserves_flags));
        core::arch::asm!("mrs {0}, cntvct_el0", out(reg) start, options(nomem, nostack, preserves_flags));
    }

    let target = start.wrapping_add(freq.wrapping_mul(MicroSeconds as u64) / 1_000_000);
    loop {
        let current: u64;
        unsafe {
            core::arch::asm!("mrs {0}, cntvct_el0", out(reg) current, options(nomem, nostack, preserves_flags));
        }
        if current >= target {
            break;
        }
        unsafe {
            core::arch::asm!("yield", options(nomem, nostack, preserves_flags));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeGetCurrentIrql() -> ddk::KIRQL {
    unsafe { halp_get_irql() }
}

extern "system" {
    /// ntoskrnl export (ke/dpc.rs): the DISPATCH-level software-interrupt body
    /// — timer expiration (Prcb.TimerRequest → KiTimerExpiration) + device-DPC
    /// drain, in normal thread context. Called below when the soft IRQL
    /// crosses under DISPATCH_LEVEL with the DISPATCH bit pended.
    fn KiDispatchSoftwareIntArm64();
    /// ntoskrnl export (ke/dpc.rs): quantum end / switch to Prcb->NextThread,
    /// entered at DISPATCH on a crossing under it — x86's DPC-interrupt tail.
    fn KiDispatchPreemptArm64();
}

unsafe fn HalpDispatchSoftwareInterrupts(OldIrql: ddk::KIRQL, NewIrql: ddk::KIRQL) {
    let cpu = halp_cpu_state();
    if cpu
        .soft_int_in_progress
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    // DISPATCH delivery first (timers + DPCs), then APC — mirrors the NT
    // lowering order (higher software level drains before the lower one).
    // Without this, timer expiration ran ONLY from the idle loop, so one
    // ready-spinning thread froze every kernel timer on arm64 (landmine #4,
    // field manual §9.20) — a spinner still crosses DISPATCH in its own
    // context constantly (syscalls, spinlock releases), so delivery here keeps
    // timers flowing no matter how busy the ready queue is.
    //
    // GATED on interrupt-context depth == 0: KfLowerIrql is also reachable
    // from inside IRQ exception windows (raise/lower pairs in ISR- and
    // DPC-called kernel code), and running KiTimerExpiration's thread-wake
    // machinery from exception context is the §9.19 wedge/crash class
    // (first attempt BUGCHECKed 0x1E with an EC=0x0E illegal-execution-state
    // return to EL0). HalpEnter/ExitInterruptIrql bracket every exception-
    // context dispatch window, so depth 0 == genuine thread/SVC context.
    let dispatch_bit = software_interrupt_bit(DISPATCH_LEVEL);
    if NewIrql < DISPATCH_LEVEL
        && cpu.interrupt_context_depth.load(Ordering::SeqCst) == 0
        && (cpu.soft_int_pending.load(Ordering::SeqCst) & dispatch_bit) != 0
    {
        cpu.soft_int_pending.fetch_and(!dispatch_bit, Ordering::SeqCst);
        unsafe {
            halp_set_irql(DISPATCH_LEVEL);
            KiDispatchSoftwareIntArm64();
            halp_set_irql(NewIrql);
        }
    }

    let apc_bit = software_interrupt_bit(APC_LEVEL);
    if NewIrql < APC_LEVEL && (cpu.soft_int_pending.load(Ordering::SeqCst) & apc_bit) != 0 {
        unsafe {
            let thread = ddk::KeGetCurrentThread();
            if !thread.is_null() {
                let kernel_apc_pending = (*thread).__bindgen_anon_1.ApcState.KernelApcPending != 0;
                let special_apc_disable = (*thread)
                    .__bindgen_anon_5
                    .__bindgen_anon_8
                    .__bindgen_anon_1
                    .__bindgen_anon_1
                    .SpecialApcDisable;

                if kernel_apc_pending && special_apc_disable == 0 {
                    cpu.soft_int_pending.fetch_and(!apc_bit, Ordering::SeqCst);
                    halp_set_irql(APC_LEVEL);
                    ddk::KiDeliverApc(
                        ddk::_MODE_KernelMode as ddk::KPROCESSOR_MODE,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    halp_set_irql(NewIrql);
                }
            }
        }
    }

    cpu.soft_int_in_progress.store(0, Ordering::SeqCst);
}

// --- IRQL-violation tripwire (serial self-reporting) -----------------------
// Raising DOWN or lowering UP is the classic imbalance that ends in a masked
// clock or a re-entered spinlock; catch it at the violation site with the
// caller's address symbolicatable via addr2sym.sh. Reports the first few then
// goes quiet (a broken caller in a hot path must not flood the UART).

static HalpIrqlViolations: AtomicU32 = AtomicU32::new(0);
const IRQL_VIOLATION_REPORT_MAX: u32 = 8;

// The raise-DOWN/lower-UP violations (field manual §9.24) are the symptom of a
// LEAKED HIGH_LEVEL raise: current soft IRQL is stuck at 15 when a spinlock
// acquire raises to SYNCH(=DISPATCH=2), so the "raise" drops 15->2. The victim
// is easy to see; the leaker is not. Record the caller RA of the most recent
// raise UP TO HIGH_LEVEL so the violation report can name the site that raised
// and never lowered — turning a rare arm64-net flake's forensics from an
// anonymous later-module caller into the exact leaking raiser. Zero cost off the
// raise path; only stored on a genuine raise-to-HIGH.
static HalpLastHighRaiseRa: AtomicU64 = AtomicU64::new(0);

extern "C" {
    fn DbgPrint(Format: *const u8, ...) -> u32;
}

#[inline(never)]
unsafe fn irql_violation(kind: &'static [u8], old: ddk::KIRQL, new: ddk::KIRQL) {
    if HalpIrqlViolations.fetch_add(1, Ordering::Relaxed) < IRQL_VIOLATION_REPORT_MAX {
        // x30 here is our caller = the KfRaiseIrql/KfLowerIrql frame's caller
        // is already gone; the violating site is 1-2 frames up — print our
        // return address (inside Kf*Irql's caller) as the best cheap anchor.
        let ra: u64;
        core::arch::asm!("mov {0}, x30", out(reg) ra, options(nomem, nostack, preserves_flags));
        // Leaked-HIGH case (old IRQL already at HIGH_LEVEL): also print the last
        // raise-to-HIGH site — that's the raiser that never lowered (§9.24).
        let leaker = if old as u32 >= 15 {
            HalpLastHighRaiseRa.load(Ordering::Relaxed)
        } else {
            0
        };
        DbgPrint(
            b"[IRQL] VIOLATION: %s %u -> %u, caller near %p, last raise-to-HIGH @ %p\n\0".as_ptr(),
            kind.as_ptr(),
            old as u32,
            new as u32,
            ra as usize,
            leaker as usize,
        );
    }
}

/// One-way mute for the IRQL tripwire, called by the kernel at bugcheck entry.
/// The bugcheck path parks IRQL at HIGH_LEVEL by design while its display path
/// acquires DISPATCH-level locks, so every acquire would report raise-DOWN /
/// lower-UP — eight lines of noise wedged between the banner and the
/// diagnostics that read like the crash signal (they misled a 2026-07-11
/// triage). Burning the report budget mutes it without touching the hot path.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosQuiesceIrqlTripwire() {
    HalpIrqlViolations.store(IRQL_VIOLATION_REPORT_MAX, Ordering::Relaxed);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfLowerIrql(NewIrql: ddk::KIRQL) {
    unsafe {
        let OldIrql = halp_get_irql();
        if NewIrql > OldIrql {
            irql_violation(b"lower-UP\0", OldIrql, NewIrql);
        }
        halp_set_irql(NewIrql);
        // Hardware half: reopen the GIC priority mask for the new level (MMIO
        // skipped inside the software band) BEFORE draining soft interrupts,
        // so a device IRQ deliverable at the new level isn't held off by a
        // stale mask while we run APC delivery.
        super::clock::update_pmr(OldIrql, NewIrql);

        if OldIrql > NewIrql
            && halp_cpu_state().soft_int_pending.load(Ordering::SeqCst) != 0
        {
            HalpDispatchSoftwareInterrupts(OldIrql, NewIrql);
        }

        // Preemption point: on x86 this crossing is where the DPC interrupt
        // fires and switches to a readied higher-priority thread; arm64 had no
        // equivalent, so a thread that never blocked held the CPU forever
        // (the zero-page thread stalled every arm64 kmtest round ~95 s). Only
        // in thread context (not inside an IRQ window) and never while our own
        // softint drain is still running — a switch there would leave
        // soft_int_in_progress set under whichever thread runs next, and every
        // lower on this CPU would skip delivery until we were resumed. The
        // kernel side declines when IRQs are masked, on the idle thread, and
        // inside a DPC. Re-read the CPU state: after the switch we may resume
        // on another CPU.
        if OldIrql >= DISPATCH_LEVEL && NewIrql < DISPATCH_LEVEL {
            let cpu = halp_cpu_state();
            if cpu.interrupt_context_depth.load(Ordering::SeqCst) == 0
                && cpu.soft_int_in_progress.load(Ordering::SeqCst) == 0
            {
                halp_set_irql(DISPATCH_LEVEL);
                KiDispatchPreemptArm64();
                halp_set_irql(NewIrql);
            }
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfRaiseIrql(NewIrql: ddk::KIRQL) -> ddk::KIRQL {
    unsafe {
        let OldIrql = halp_get_irql();
        if NewIrql < OldIrql {
            irql_violation(b"raise-DOWN\0", OldIrql, NewIrql);
        }
        // Track raises to HIGH_LEVEL so a later leaked-HIGH violation (§9.24)
        // can name the raiser that never lowered.
        if NewIrql as u32 >= 15 {
            let ra: u64;
            core::arch::asm!("mov {0}, x30", out(reg) ra, options(nomem, nostack, preserves_flags));
            HalpLastHighRaiseRa.store(ra, Ordering::Relaxed);
        }
        halp_set_irql(NewIrql);
        // Hardware half: mask GIC lines at-or-below the new level (MMIO
        // skipped inside the software band, so spinlocks stay register-only).
        super::clock::update_pmr(OldIrql, NewIrql);
        OldIrql
    }
}

/// Enter interrupt-dispatch IRQL (called by ntoskrnl's KiDispatchInterruptArm64
/// around a device ISR). Same as KfRaiseIrql but exempt from the raise-DOWN
/// tripwire: an IRQ delivered while the soft IRQL was transiently higher than
/// the line's DIRQL (PMR write windows) must still dispatch at ITS level.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpEnterInterruptIrql(NewIrql: ddk::KIRQL) -> ddk::KIRQL {
    unsafe {
        halp_cpu_state()
            .interrupt_context_depth
            .fetch_add(1, Ordering::SeqCst);
        let OldIrql = halp_get_irql();
        halp_set_irql(NewIrql);
        super::clock::update_pmr(OldIrql, NewIrql);
        OldIrql
    }
}

/// Exit interrupt-dispatch IRQL: restore the interrupted level WITHOUT the
/// KfLowerIrql soft-interrupt drain — delivering APCs from inside the IRQ
/// exception would run them in interrupt context (see the clock ISR's note).
/// Pending DPC/APC work is picked up on the trap-exit path / next real lower.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpExitInterruptIrql(RestoreIrql: ddk::KIRQL) {
    unsafe {
        let OldIrql = halp_get_irql();
        halp_set_irql(RestoreIrql);
        super::clock::update_pmr(OldIrql, RestoreIrql);
        halp_cpu_state()
            .interrupt_context_depth
            .fetch_sub(1, Ordering::SeqCst);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeLowerIrql(NewIrql: ddk::KIRQL) {
    unsafe { KfLowerIrql(NewIrql) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeRaiseIrql(NewIrql: ddk::KIRQL, OldIrql: ddk::PKIRQL) {
    unsafe {
        *OldIrql = KfRaiseIrql(NewIrql);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeRaiseIrqlToDpcLevel() -> ddk::KIRQL {
    unsafe { KfRaiseIrql(DISPATCH_LEVEL) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeRaiseIrqlToSynchLevel() -> ddk::KIRQL {
    unsafe { KfRaiseIrql(SYNCH_LEVEL) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalBeginSystemInterrupt(
    Irql: ddk::KIRQL,
    Vector: ddk::ULONG,
    OldIrql: ddk::PKIRQL,
) -> ddk::BOOLEAN {
    unsafe {
        *OldIrql = KfRaiseIrql(Irql);
    }

    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalEndSystemInterrupt(
    Irql: ddk::KIRQL,
    TrapFrame: ddk::PKTRAP_FRAME,
) {
    unsafe { KfLowerIrql(Irql) }
}

// ---------------------------------------------------------------------------
// Spinlocks.
//
// hal.spec exports the whole `Ke*SpinLock*` surface `-arch=i386,arm,arm64`, so
// on ARM64 — exactly as on i386, and unlike amd64, which takes them from
// ntoskrnl.spec — THESE functions are the kernel's spinlocks. Every one of them
// used to be a plain store: no interlocked op, no spin, no barrier. That is a
// correct UP implementation and a non-implementation on MP, where two CPUs both
// "acquire" and both proceed. It went unnoticed because it is SELF-CONSISTENT —
// nothing stamps a lock that nothing else clears — so no kmtest row and no
// ratchet delta could see it, unlike the i386 twin of this bug, which was loud
// precisely because the HAL and the kernel disagreed.
//
// Both halves now funnel into ntkrnl_sys::inline::ke, the same code ntoskrnl
// uses, so the kernel and HAL halves of a lock cannot diverge and the `smp`
// feature reaches them (hal-rust/smp -> ntkrnl-sys/smp). On the UP HAL this is
// behaviour-preserving: `dbg` is in ntkrnl-sys's default feature set, so
// acquire still writes `KeGetCurrentThread() | 1` and release still clears —
// the same values as the plain stores, now with the barriers.
//
// The null guard is kept. A NULL spinlock is a caller bug, but tolerating it is
// this port's existing behaviour and changing that belongs in its own commit,
// not smuggled in under a locking fix.
#[inline]
unsafe fn halp_acquire_spin_lock_dbg(spin_lock: ddk::PKSPIN_LOCK) {
    unsafe {
        if !spin_lock.is_null() {
            ntkrnl_sys::inline::ke::KxAcquireSpinLock(spin_lock as *mut _);
        }
    }
}

#[inline]
unsafe fn halp_release_spin_lock_dbg(spin_lock: ddk::PKSPIN_LOCK) {
    unsafe {
        if !spin_lock.is_null() {
            ntkrnl_sys::inline::ke::KxReleaseSpinLock(spin_lock as *mut _);
        }
    }
}

/// Pull `KPRCB.LockQueue[N].Lock` for the by-number queued-spinlock functions.
/// The HACK is upstream's — a real queued lock walks a per-CPU queue — and this
/// mirrors `hal-rust/src/x86/spinlock.rs::queue_lock` so the two arches resolve
/// the same lock the same way.
///
/// Before this existed the by-number surface raised IRQL and returned, ignoring
/// `Number` entirely, which left `LockQueuePfnLock` and friends with no
/// cross-CPU protection at all on `halmp`.
#[inline(always)]
unsafe fn queue_lock(number: ddk::KSPIN_LOCK_QUEUE_NUMBER) -> ddk::PKSPIN_LOCK {
    unsafe {
        let prcb = ntkrnl_sys::inline::ke::KeGetCurrentPrcb();
        (*prcb).LockQueue[number as usize].Lock
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfAcquireSpinLock(SpinLock: ddk::PKSPIN_LOCK) -> ddk::KIRQL {
    unsafe {
        let OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
        halp_acquire_spin_lock_dbg(SpinLock);
        OldIrql
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfReleaseSpinLock(SpinLock: ddk::PKSPIN_LOCK, NewIrql: ddk::KIRQL) {
    unsafe {
        halp_release_spin_lock_dbg(SpinLock);
        KfLowerIrql(NewIrql);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireSpinLock(SpinLock: ddk::PKSPIN_LOCK, OldIrql: ddk::PKIRQL) {
    unsafe {
        *OldIrql = KfAcquireSpinLock(SpinLock);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeReleaseSpinLock(SpinLock: ddk::PKSPIN_LOCK, NewIrql: ddk::KIRQL) {
    unsafe { KfReleaseSpinLock(SpinLock, NewIrql) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireSpinLockRaiseToSynch(
    SpinLock: ddk::PKSPIN_LOCK,
) -> ddk::KIRQL {
    unsafe {
        let OldIrql = KfRaiseIrql(SYNCH_LEVEL);
        halp_acquire_spin_lock_dbg(SpinLock);
        OldIrql
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireInStackQueuedSpinLock(
    SpinLock: ddk::PKSPIN_LOCK,
    LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    unsafe {
        (*LockHandle).LockQueue.Next = core::ptr::null_mut();
        (*LockHandle).OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
        (*LockHandle).LockQueue.Lock = SpinLock;
        halp_acquire_spin_lock_dbg(SpinLock);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireInStackQueuedSpinLockRaiseToSynch(
    SpinLock: ddk::PKSPIN_LOCK,
    LockHandle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    unsafe {
        (*LockHandle).LockQueue.Next = core::ptr::null_mut();
        (*LockHandle).OldIrql = KfRaiseIrql(SYNCH_LEVEL);
        (*LockHandle).LockQueue.Lock = SpinLock;
        halp_acquire_spin_lock_dbg(SpinLock);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeReleaseInStackQueuedSpinLock(LockHandle: ddk::PKLOCK_QUEUE_HANDLE) {
    unsafe {
        halp_release_spin_lock_dbg((*LockHandle).LockQueue.Lock);
        KfLowerIrql((*LockHandle).OldIrql);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireQueuedSpinLock(
    Number: ddk::KSPIN_LOCK_QUEUE_NUMBER,
) -> ddk::KIRQL {
    unsafe {
        let OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
        halp_acquire_spin_lock_dbg(queue_lock(Number));
        OldIrql
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ddk::KSPIN_LOCK_QUEUE_NUMBER,
) -> ddk::KIRQL {
    unsafe {
        let OldIrql = KfRaiseIrql(SYNCH_LEVEL);
        halp_acquire_spin_lock_dbg(queue_lock(LockNumber));
        OldIrql
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeReleaseQueuedSpinLock(
    Number: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::KIRQL,
) {
    unsafe {
        halp_release_spin_lock_dbg(queue_lock(Number));
        KfLowerIrql(OldIrql);
    }
}

// The Try* pair raises IRQL even when the acquire fails — kmtest asserts that,
// and the x86 twin mirrors it too. Delegating to ntoskrnl's
// KeTryToAcquireSpinLockAtDpcLevel keeps one try-acquire implementation across
// kernel and HAL, the same reason the blocking pair funnels into ntkrnl-sys.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeTryToAcquireQueuedSpinLock(
    Number: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::PKIRQL,
) -> ddk::LOGICAL {
    unsafe {
        let lock = queue_lock(Number);
        *OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
        ddk::KeTryToAcquireSpinLockAtDpcLevel(lock) as ddk::LOGICAL
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeTryToAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::PKIRQL,
) -> ddk::BOOLEAN {
    unsafe {
        let lock = queue_lock(LockNumber);
        *OldIrql = KfRaiseIrql(SYNCH_LEVEL);
        ddk::KeTryToAcquireSpinLockAtDpcLevel(lock)
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX) {
    unsafe {
        (*FastMutex).OldIrql = KfRaiseIrql(APC_LEVEL) as ddk::ULONG;
        ddk::ExAcquireFastMutexUnsafe(FastMutex);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExReleaseFastMutex(FastMutex: ddk::PFAST_MUTEX) {
    unsafe {
        let OldIrql = (*FastMutex).OldIrql as ddk::KIRQL;
        ddk::ExReleaseFastMutexUnsafe(FastMutex);
        KfLowerIrql(OldIrql);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExTryToAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX) -> ddk::BOOLEAN {
    unsafe {
        (*FastMutex).OldIrql = KfRaiseIrql(APC_LEVEL) as ddk::ULONG;
        if interlocked_compare_exchange(&raw mut (*FastMutex).Count, 0, 1) == 1 {
            (*FastMutex).Owner = ddk::KeGetCurrentThread();
            return TRUE;
        }

        KfLowerIrql((*FastMutex).OldIrql as ddk::KIRQL);
    }

    FALSE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_UCHAR(Port: ddk::PUCHAR) -> ddk::UCHAR {
    unsafe { core::ptr::read_volatile(Port) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_USHORT(Port: ddk::PUSHORT) -> ddk::USHORT {
    unsafe { core::ptr::read_volatile(Port) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_ULONG(Port: ddk::PULONG) -> ddk::ULONG {
    unsafe { core::ptr::read_volatile(Port) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_UCHAR(
    Port: ddk::PUCHAR,
    Buffer: ddk::PUCHAR,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write(Buffer.add(i), core::ptr::read_volatile(Port));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_USHORT(
    Port: ddk::PUSHORT,
    Buffer: ddk::PUSHORT,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write(Buffer.add(i), core::ptr::read_volatile(Port));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn READ_PORT_BUFFER_ULONG(
    Port: ddk::PULONG,
    Buffer: ddk::PULONG,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write(Buffer.add(i), core::ptr::read_volatile(Port));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_UCHAR(Port: ddk::PUCHAR, Value: ddk::UCHAR) {
    unsafe {
        core::ptr::write_volatile(Port, Value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_USHORT(Port: ddk::PUSHORT, Value: ddk::USHORT) {
    unsafe {
        core::ptr::write_volatile(Port, Value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_ULONG(Port: ddk::PULONG, Value: ddk::ULONG) {
    unsafe {
        core::ptr::write_volatile(Port, Value);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_UCHAR(
    Port: ddk::PUCHAR,
    Buffer: ddk::PUCHAR,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write_volatile(Port, core::ptr::read(Buffer.add(i)));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_USHORT(
    Port: ddk::PUSHORT,
    Buffer: ddk::PUSHORT,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write_volatile(Port, core::ptr::read(Buffer.add(i)));
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn WRITE_PORT_BUFFER_ULONG(
    Port: ddk::PULONG,
    Buffer: ddk::PULONG,
    Count: ddk::ULONG,
) {
    unsafe {
        for i in 0..Count as usize {
            core::ptr::write_volatile(Port, core::ptr::read(Buffer.add(i)));
        }
    }
}
