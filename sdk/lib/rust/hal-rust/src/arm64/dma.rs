//! ARM64 DMA adapter — minimal coherent-DMA implementation for QEMU `virt`.
//!
//! The GPEX PCIe host bridge on `-M virt` is cache-coherent (`dma-coherent` in
//! the DT/ACPI description) and there is no IOMMU in our configuration, so DMA
//! is simply "device sees physical memory": no map registers, no bounce
//! buffers, no cache maintenance. That collapses the NT DMA model to:
//!
//!   * `AllocateCommonBuffer`  = contiguous cached allocation + its physical
//!     address (same pattern netstack-rust uses for its virtio/e1000 rings).
//!   * `MapTransfer`           = walk the MDL PFN array and return the physical
//!     run starting at `CurrentVa`, clamping `*Length` to the physically
//!     contiguous extent (callers like usbport loop, building an SG list).
//!   * `AllocateAdapterChannel`= call the execution routine immediately at
//!     DISPATCH_LEVEL with a dummy MapRegisterBase (there is nothing to queue
//!     for — map registers are infinite/free here).
//!   * `FlushAdapterBuffers`   = TRUE no-op (coherent).
//!
//! Consumers: usbport-rust reaches this through
//! `IoGetDmaAdapter` → pci.sys `BUS_INTERFACE_STANDARD.GetDmaAdapter` →
//! `HalGetAdapter` (this file). The x86 equivalent is the full map-register
//! implementation in `x86/dma.rs`; this one is deliberately virt-shaped.
//! (HalDispatchTable.HalGetDmaAdapter is NOT registered yet — the PnP
//! bus-interface path above doesn't need it; add it if a non-PnP legacy caller
//! ever shows up on arm64.)

#![cfg(target_arch = "aarch64")]
#![allow(unused_variables)]

use core::sync::atomic::{AtomicBool, Ordering};
use ntkrnl_sys::ddk;

const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const DISPATCH_LEVEL: ddk::KIRQL = 2;
const STATUS_SUCCESS: ddk::NTSTATUS = 0;
const STATUS_INSUFFICIENT_RESOURCES: ddk::NTSTATUS = 0xC000009Au32 as ddk::NTSTATUS;
const HAL_DMA_TAG: u32 = u32::from_le_bytes(*b"amDH"); // 'HDma'

// KfRaiseIrql/KfLowerIrql are #[no_mangle] exports elsewhere in this HAL
// (halstubs.rs, PMR-backed). Reach them by symbol.
extern "C" {
    fn KfRaiseIrql(NewIrql: ddk::KIRQL) -> ddk::KIRQL;
    fn KfLowerIrql(OldIrql: ddk::KIRQL);
}

/// The adapter object we hand out. NT layout requires the DMA_ADAPTER header
/// first (`ADAPTER_OBJECT.DmaHeader`) — pci.sys casts PADAPTER_OBJECT to
/// PDMA_ADAPTER and callers only ever dereference the header.
#[repr(C)]
struct Arm64AdapterObject {
    dma_header: ddk::DMA_ADAPTER,
}

// One DMA_OPERATIONS table shared by every adapter. Function-pointer fields are
// installed on first use (bindgen models them as distinct fn-pointer types per
// slot; `transmute` from our `extern "system"` fns, same trick as x86/dma.rs).
static mut ARM64_DMA_OPERATIONS: ddk::DMA_OPERATIONS = unsafe { core::mem::zeroed() };
static ARM64_DMA_OPS_READY: AtomicBool = AtomicBool::new(false);

unsafe fn arm64_dma_ops() -> *mut ddk::DMA_OPERATIONS {
    let ops = core::ptr::addr_of_mut!(ARM64_DMA_OPERATIONS);
    if !ARM64_DMA_OPS_READY.load(Ordering::Acquire) {
        (*ops).Size = core::mem::size_of::<ddk::DMA_OPERATIONS>() as ddk::ULONG;
        (*ops).PutDmaAdapter = Some(core::mem::transmute(Arm64DmaPutDmaAdapter as usize));
        (*ops).AllocateCommonBuffer =
            Some(core::mem::transmute(Arm64DmaAllocateCommonBuffer as usize));
        (*ops).FreeCommonBuffer = Some(core::mem::transmute(Arm64DmaFreeCommonBuffer as usize));
        (*ops).AllocateAdapterChannel =
            Some(core::mem::transmute(Arm64DmaAllocateAdapterChannel as usize));
        (*ops).FlushAdapterBuffers =
            Some(core::mem::transmute(Arm64DmaFlushAdapterBuffers as usize));
        (*ops).FreeAdapterChannel = Some(core::mem::transmute(Arm64DmaFreeAdapterChannel as usize));
        (*ops).FreeMapRegisters = Some(core::mem::transmute(Arm64DmaFreeMapRegisters as usize));
        (*ops).MapTransfer = Some(core::mem::transmute(Arm64DmaMapTransfer as usize));
        (*ops).GetDmaAlignment = Some(core::mem::transmute(Arm64DmaGetDmaAlignment as usize));
        (*ops).ReadDmaCounter = Some(core::mem::transmute(Arm64DmaReadDmaCounter as usize));
        // Scatter-gather ops stay None — usbport/usbstor don't use them; a
        // future consumer that does should get a loud None-unwrap, not a
        // silently-wrong implementation.
        ARM64_DMA_OPS_READY.store(true, Ordering::Release);
    }
    ops
}

/// HalGetAdapter — the legacy/NT4 export; also what pci.sys's
/// `BUS_INTERFACE_STANDARD.GetDmaAdapter` calls. Allocates a fresh adapter
/// object (freed by PutDmaAdapter).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetAdapter(
    DeviceDescription: ddk::PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: ddk::PULONG,
) -> ddk::PADAPTER_OBJECT {
    let adapter = ddk::ExAllocatePoolWithTag(
        ddk::_POOL_TYPE_NonPagedPool,
        core::mem::size_of::<Arm64AdapterObject>() as ddk::SIZE_T,
        HAL_DMA_TAG,
    ) as *mut Arm64AdapterObject;
    if adapter.is_null() {
        return core::ptr::null_mut();
    }
    (*adapter).dma_header.Version = 1;
    (*adapter).dma_header.Size = core::mem::size_of::<ddk::DMA_ADAPTER>() as ddk::USHORT;
    (*adapter).dma_header.DmaOperations = arm64_dma_ops();

    if !NumberOfMapRegisters.is_null() {
        // "Map registers" are free here; report a generous per-transfer cap so
        // callers size their SG machinery sanely (usbport asks and remembers).
        *NumberOfMapRegisters = 256;
    }
    adapter as ddk::PADAPTER_OBJECT
}

unsafe extern "system" fn Arm64DmaPutDmaAdapter(DmaAdapter: ddk::PDMA_ADAPTER) {
    if !DmaAdapter.is_null() {
        ddk::ExFreePoolWithTag(DmaAdapter as ddk::PVOID, HAL_DMA_TAG);
    }
}

/// Contiguous cached allocation below 4 GB (EHCI programs 32-bit physical
/// addresses unless the 64-bit CTRLDSSEGMENT path is used — stay low to be
/// safe; virt RAM starts at 1 GB so this is trivially satisfiable).
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateCommonBuffer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PPHYSICAL_ADDRESS,
    CacheEnabled: ddk::BOOLEAN,
) -> ddk::PVOID {
    let mut highest: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    highest.QuadPart = 0xFFFF_FFFF;
    let lowest: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    let boundary: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();

    let va = ddk::MmAllocateContiguousMemorySpecifyCache(
        Length as ddk::SIZE_T,
        lowest,
        highest,
        boundary,
        ddk::_MEMORY_CACHING_TYPE_MmCached,
    );
    if va.is_null() {
        return core::ptr::null_mut();
    }
    core::ptr::write_bytes(va as *mut u8, 0, Length as usize);
    let base_phys = ddk::MmGetPhysicalAddress(va);
    if !LogicalAddress.is_null() {
        *LogicalAddress = base_phys;
    }

    // TEMP Wall-2 instrumentation (task #18) — remove before landing.
    // Verify the allocation is REALLY physically contiguous: every page's true
    // phys must equal base_phys + page offset, else DMA descriptors carved at
    // offsets point at the wrong physical memory.
    {
        extern "C" {
            fn DbgPrint(fmt: *const u8, ...) -> u32;
        }
        let pages = ((Length as usize) + 0xFFF) >> 12;
        let mut bad = 0u32;
        let mut i = 0usize;
        while i < pages {
            let pva = (va as usize + (i << 12)) as ddk::PVOID;
            let p = ddk::MmGetPhysicalAddress(pva);
            let expect = base_phys.QuadPart as u64 + ((i as u64) << 12);
            if p.QuadPart as u64 != expect {
                bad += 1;
                DbgPrint(
                    b"[W2-CB] NONCONTIG page %d: va=%p phys=%llx expect=%llx\n\0".as_ptr(),
                    i as u32,
                    pva,
                    p.QuadPart as u64,
                    expect,
                );
            }
            i += 1;
        }
        DbgPrint(
            b"[W2-CB] alloc va=%p len=%x base_phys=%llx pages=%d noncontig=%d\n\0".as_ptr(),
            va,
            Length,
            base_phys.QuadPart as u64,
            pages as u32,
            bad,
        );
    }
    va
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFreeCommonBuffer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PHYSICAL_ADDRESS,
    VirtualAddress: ddk::PVOID,
    CacheEnabled: ddk::BOOLEAN,
) {
    if !VirtualAddress.is_null() {
        ddk::MmFreeContiguousMemory(VirtualAddress);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFlushCommonBuffer(
    AdapterObject: ddk::PADAPTER_OBJECT,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PHYSICAL_ADDRESS,
    VirtualAddress: ddk::PVOID,
) -> ddk::BOOLEAN {
    1 // coherent
}

// DMA_OPERATIONS-shaped wrappers (first param is PDMA_ADAPTER, not
// PADAPTER_OBJECT — same pointer, different nominal type).

unsafe extern "system" fn Arm64DmaAllocateCommonBuffer(
    DmaAdapter: ddk::PDMA_ADAPTER,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PPHYSICAL_ADDRESS,
    CacheEnabled: ddk::BOOLEAN,
) -> ddk::PVOID {
    HalAllocateCommonBuffer(
        DmaAdapter as ddk::PADAPTER_OBJECT,
        Length,
        LogicalAddress,
        CacheEnabled,
    )
}

unsafe extern "system" fn Arm64DmaFreeCommonBuffer(
    DmaAdapter: ddk::PDMA_ADAPTER,
    Length: ddk::ULONG,
    LogicalAddress: ddk::PHYSICAL_ADDRESS,
    VirtualAddress: ddk::PVOID,
    CacheEnabled: ddk::BOOLEAN,
) {
    HalFreeCommonBuffer(
        DmaAdapter as ddk::PADAPTER_OBJECT,
        Length,
        LogicalAddress,
        VirtualAddress,
        CacheEnabled,
    );
}

/// Map registers are free — run the execution routine right now, at
/// DISPATCH_LEVEL, as the NT contract requires. MapRegisterBase is a dummy
/// non-null cookie (nothing dereferences it; it only round-trips into
/// MapTransfer/FreeMapRegisters).
type DriverControlFn = unsafe extern "system" fn(
    DeviceObject: ddk::PDEVICE_OBJECT,
    Irp: ddk::PIRP,
    MapRegisterBase: ddk::PVOID,
    Context: ddk::PVOID,
) -> ddk::IO_ALLOCATION_ACTION;

unsafe extern "system" fn Arm64DmaAllocateAdapterChannel(
    DmaAdapter: ddk::PDMA_ADAPTER,
    DeviceObject: ddk::PDEVICE_OBJECT,
    NumberOfMapRegisters: ddk::ULONG,
    ExecutionRoutine: ddk::PDRIVER_CONTROL,
    Context: ddk::PVOID,
) -> ddk::NTSTATUS {
    let routine: DriverControlFn = match ExecutionRoutine {
        Some(f) => core::mem::transmute(f),
        None => return STATUS_INSUFFICIENT_RESOURCES,
    };
    let old_irql = KfRaiseIrql(DISPATCH_LEVEL);
    let current_irp = if DeviceObject.is_null() {
        core::ptr::null_mut()
    } else {
        (*DeviceObject).CurrentIrp
    };
    // MapRegisterBase: the adapter pointer itself — non-null, opaque.
    routine(DeviceObject, current_irp, DmaAdapter as ddk::PVOID, Context);
    KfLowerIrql(old_irql);
    STATUS_SUCCESS
}

unsafe extern "system" fn Arm64DmaFlushAdapterBuffers(
    DmaAdapter: ddk::PDMA_ADAPTER,
    Mdl: ddk::PMDL,
    MapRegisterBase: ddk::PVOID,
    CurrentVa: ddk::PVOID,
    Length: ddk::ULONG,
    WriteToDevice: ddk::BOOLEAN,
) -> ddk::BOOLEAN {
    1 // coherent — nothing to flush
}

unsafe extern "system" fn Arm64DmaFreeAdapterChannel(DmaAdapter: ddk::PDMA_ADAPTER) {}

unsafe extern "system" fn Arm64DmaFreeMapRegisters(
    DmaAdapter: ddk::PDMA_ADAPTER,
    MapRegisterBase: ddk::PVOID,
    NumberOfMapRegisters: ddk::ULONG,
) {
}

/// Return the physical address of `CurrentVa` within the MDL's buffer and
/// clamp `*Length` to the physically contiguous run starting there. Walks the
/// PFN array (which immediately follows the MDL header) rather than trusting
/// `MmGetPhysicalAddress(CurrentVa)` — CurrentVa may be a user VA from another
/// context.
unsafe extern "system" fn Arm64DmaMapTransfer(
    DmaAdapter: ddk::PDMA_ADAPTER,
    Mdl: ddk::PMDL,
    MapRegisterBase: ddk::PVOID,
    CurrentVa: ddk::PVOID,
    Length: ddk::PULONG,
    WriteToDevice: ddk::BOOLEAN,
) -> ddk::PHYSICAL_ADDRESS {
    let mut phys: ddk::PHYSICAL_ADDRESS = core::mem::zeroed();
    if Mdl.is_null() || Length.is_null() || (*Length) == 0 {
        return phys;
    }

    // Offset of CurrentVa from the MDL's page-aligned StartVa. The PFN array
    // is indexed from StartVa; the buffer itself begins at ByteOffset into
    // page 0.
    let start_va = (*Mdl).StartVa as usize;
    let offset = (CurrentVa as usize).wrapping_sub(start_va);
    let byte_count = (*Mdl).ByteCount as usize;
    let byte_offset = (*Mdl).ByteOffset as usize;
    debug_assert!(offset >= byte_offset && offset < byte_offset + byte_count);

    let pfn_array = Mdl.offset(1) as *const ddk::PFN_NUMBER;
    let mut idx = offset >> PAGE_SHIFT;
    let page_off = offset & (PAGE_SIZE - 1);

    phys.QuadPart =
        (((*pfn_array.add(idx)) as u64) << PAGE_SHIFT) as i64 + page_off as i64;

    // Clamp to (a) the end of the MDL buffer, (b) the physically contiguous
    // page run.
    let mut max_len = (byte_offset + byte_count).saturating_sub(offset);
    let mut contiguous = PAGE_SIZE - page_off;
    let last_idx = (byte_offset + byte_count - 1) >> PAGE_SHIFT;
    while contiguous < max_len && idx < last_idx {
        if *pfn_array.add(idx + 1) != (*pfn_array.add(idx)) + 1 {
            break;
        }
        idx += 1;
        contiguous += PAGE_SIZE;
    }
    if max_len > contiguous {
        max_len = contiguous;
    }
    if (*Length) as usize > max_len {
        *Length = max_len as ddk::ULONG;
    }
    phys
}

unsafe extern "system" fn Arm64DmaGetDmaAlignment(DmaAdapter: ddk::PDMA_ADAPTER) -> ddk::ULONG {
    1
}

unsafe extern "system" fn Arm64DmaReadDmaCounter(DmaAdapter: ddk::PDMA_ADAPTER) -> ddk::ULONG {
    0
}
