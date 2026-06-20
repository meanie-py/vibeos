//! ARM64 HAL stubs translated from `hal/halarm64/generic/halstubs.c`.
//!
//! These are intentionally minimal. They preserve the C stub behavior used by
//! the current QEMU ARM64 boot path: enough HAL surface to link, basic IRQL
//! bookkeeping, software APC dispatch, UP spinlock/fast-mutex helpers, port
//! MMIO shims, and the ARM64 virtual counter performance clock.

#![cfg(target_arch = "aarch64")]
#![allow(unused_variables)]

use core::sync::atomic::{AtomicI32, AtomicU32, AtomicUsize, Ordering};
use ntkrnl_sys::ddk;

const TRUE: ddk::BOOLEAN = 1;
const FALSE: ddk::BOOLEAN = 0;
const STATUS_SUCCESS: ddk::NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: ddk::NTSTATUS = 0xC0000002u32 as ddk::NTSTATUS;
const ESUCCESS: ddk::ARC_STATUS = ddk::_ARC_CODES_ESUCCESS as ddk::ARC_STATUS;

const PASSIVE_LEVEL: ddk::KIRQL = 0;
const APC_LEVEL: ddk::KIRQL = 1;
const DISPATCH_LEVEL: ddk::KIRQL = 2;
const SYNCH_LEVEL: ddk::KIRQL = DISPATCH_LEVEL;

static mut HalpCurrentTimeIncrement: ddk::ULONG = 100000;
static mut HalpNextTimeIncrement: ddk::ULONG = 100000;
static mut HalpNextIntervalCount: ddk::ULONG = 0;
static mut HalpCurrentIrql: ddk::KIRQL = PASSIVE_LEVEL;

static HalpSoftwareInterruptPending: AtomicU32 = AtomicU32::new(0);
static HalpSoftIntDeliveryInProgress: AtomicI32 = AtomicI32::new(0);

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

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateCommonBuffer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PPHYSICAL_ADDRESS,
    CacheEnabled: ddk::BOOLEAN,
) -> ddk::PVOID {
    core::ptr::null_mut()
}

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
pub unsafe extern "system" fn HalDisableSystemInterrupt(Vector: ddk::ULONG, Irql: ddk::KIRQL) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalDisplayString(String: ddk::PCSTR) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalEnableSystemInterrupt(
    Vector: ddk::ULONG,
    Irql: ddk::KIRQL,
    InterruptMode: ddk::KINTERRUPT_MODE,
) -> ddk::BOOLEAN {
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFlushCommonBuffer(
    Unknown1: ddk::ULONG,
    Unknown2: ddk::ULONG,
    Unknown3: ddk::ULONG,
    Unknown4: ddk::ULONG,
    Unknown5: ddk::ULONG,
) -> ddk::BOOLEAN {
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFreeCommonBuffer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PHYSICAL_ADDRESS,
    VirtualAddress: ddk::PVOID,
    CacheEnabled: ddk::BOOLEAN,
) {
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetAdapter(
    DeviceDescription: ddk::PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: ddk::PULONG,
) -> ddk::PADAPTER_OBJECT {
    core::ptr::null_mut()
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
    0
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

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalQueryRealTimeClock(Time: ddk::PTIME_FIELDS) -> ddk::BOOLEAN {
    unsafe {
        if !Time.is_null() {
            (*Time).Year = 2025;
            (*Time).Month = 1;
            (*Time).Day = 1;
            (*Time).Hour = 0;
            (*Time).Minute = 0;
            (*Time).Second = 0;
            (*Time).Milliseconds = 0;
            (*Time).Weekday = 0;
        }
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
pub unsafe extern "system" fn HalRequestIpi(TargetProcessors: ddk::KAFFINITY) {}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalRequestSoftwareInterrupt(Request: ddk::KIRQL) {
    HalpSoftwareInterruptPending.fetch_or(software_interrupt_bit(Request), Ordering::SeqCst);

    unsafe {
        if HalpCurrentIrql < APC_LEVEL && Request == APC_LEVEL {
            HalpDispatchSoftwareInterrupts(APC_LEVEL, HalpCurrentIrql);
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

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalStartNextProcessor(
    LoaderBlock: ddk::PLOADER_PARAMETER_BLOCK,
    ProcessorState: ddk::PKPROCESSOR_STATE,
) -> ddk::BOOLEAN {
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
    unsafe { HalpCurrentIrql }
}

unsafe fn HalpDispatchSoftwareInterrupts(OldIrql: ddk::KIRQL, NewIrql: ddk::KIRQL) {
    if HalpSoftIntDeliveryInProgress
        .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let apc_bit = software_interrupt_bit(APC_LEVEL);
    if NewIrql < APC_LEVEL && (HalpSoftwareInterruptPending.load(Ordering::SeqCst) & apc_bit) != 0 {
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
                    HalpSoftwareInterruptPending.fetch_and(!apc_bit, Ordering::SeqCst);
                    HalpCurrentIrql = APC_LEVEL;
                    ddk::KiDeliverApc(
                        ddk::_MODE_KernelMode as ddk::KPROCESSOR_MODE,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    HalpCurrentIrql = NewIrql;
                }
            }
        }
    }

    HalpSoftIntDeliveryInProgress.store(0, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfLowerIrql(NewIrql: ddk::KIRQL) {
    unsafe {
        let OldIrql = HalpCurrentIrql;
        HalpCurrentIrql = NewIrql;

        if OldIrql > NewIrql && HalpSoftwareInterruptPending.load(Ordering::SeqCst) != 0 {
            HalpDispatchSoftwareInterrupts(OldIrql, NewIrql);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KfRaiseIrql(NewIrql: ddk::KIRQL) -> ddk::KIRQL {
    unsafe {
        let OldIrql = HalpCurrentIrql;
        HalpCurrentIrql = NewIrql;
        OldIrql
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

#[inline]
unsafe fn halp_acquire_spin_lock_dbg(spin_lock: ddk::PKSPIN_LOCK) {
    unsafe {
        if !spin_lock.is_null() {
            *spin_lock = (ddk::KeGetCurrentThread() as ddk::ULONG_PTR) | 1;
        }
    }
}

#[inline]
unsafe fn halp_release_spin_lock_dbg(spin_lock: ddk::PKSPIN_LOCK) {
    unsafe {
        if !spin_lock.is_null() {
            *spin_lock = 0;
        }
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
    unsafe { KfRaiseIrql(DISPATCH_LEVEL) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ddk::KSPIN_LOCK_QUEUE_NUMBER,
) -> ddk::KIRQL {
    unsafe { KfRaiseIrql(SYNCH_LEVEL) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeReleaseQueuedSpinLock(
    Number: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::KIRQL,
) {
    unsafe { KfLowerIrql(OldIrql) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeTryToAcquireQueuedSpinLock(
    Number: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::PKIRQL,
) -> ddk::LOGICAL {
    unsafe {
        *OldIrql = KfRaiseIrql(DISPATCH_LEVEL);
    }
    1
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeTryToAcquireQueuedSpinLockRaiseToSynch(
    LockNumber: ddk::KSPIN_LOCK_QUEUE_NUMBER,
    OldIrql: ddk::PKIRQL,
) -> ddk::BOOLEAN {
    unsafe {
        *OldIrql = KfRaiseIrql(SYNCH_LEVEL);
    }
    TRUE
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX) {
    unsafe {
        (*FastMutex).OldIrql = KfRaiseIrql(APC_LEVEL) as ddk::ULONG;
        interlocked_decrement(&raw mut (*FastMutex).Count);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExReleaseFastMutex(FastMutex: ddk::PFAST_MUTEX) {
    unsafe {
        let OldIrql = (*FastMutex).OldIrql as ddk::KIRQL;
        interlocked_increment(&raw mut (*FastMutex).Count);
        KfLowerIrql(OldIrql);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn ExTryToAcquireFastMutex(FastMutex: ddk::PFAST_MUTEX) -> ddk::BOOLEAN {
    unsafe {
        (*FastMutex).OldIrql = KfRaiseIrql(APC_LEVEL) as ddk::ULONG;
        if interlocked_compare_exchange(&raw mut (*FastMutex).Count, 0, 1) == 1 {
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
