//! ARM64 DMA adapter — minimal coherent-DMA implementation for QEMU `virt`.
//!
//! The GPEX PCIe host bridge on `-M virt` is cache-coherent (`dma-coherent` in
//! the DT/ACPI description), and an SMMU, when one translates, maps the identity,
//! so DMA is "device sees physical memory": no map registers, no bounce buffers,
//! no cache maintenance. That collapses the NT DMA model to:
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
//! Behind a translating SMMU, a device whose driver is DMA-remapping compatible has
//! a domain of its own (smmuv3.rs), which reaches only what its adapter maps: every
//! common buffer while it is allocated, every MapTransfer run until the flush, and
//! every scatter/gather list until it is put back. Addresses stay the identity, so
//! nothing here changes for any other device. Each adapter knows its device's
//! requester ID (HalVibeosGetDmaAdapter, from pci.sys); HalGetAdapter's know none.
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
    /// The device's PCI segment over its requester ID, or NO_REQUESTER.
    requester: u32,
}

const NO_REQUESTER: u32 = u32::MAX;

unsafe fn requester_of(adapter: *const ddk::DMA_ADAPTER) -> u32 {
    if adapter.is_null() { NO_REQUESTER } else { (*(adapter as *const Arm64AdapterObject)).requester }
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
        // ndis's ProSend calls GetScatterGatherList for every bus-master miniport (e1000)
        // through this table from C, so a None here was a call to address 0 -- a 0x1E
        // naming nothing -- not a loud unwrap.
        (*ops).GetScatterGatherList =
            Some(core::mem::transmute(Arm64DmaGetScatterGatherList as usize));
        (*ops).PutScatterGatherList =
            Some(core::mem::transmute(Arm64DmaPutScatterGatherList as usize));
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
    new_adapter(NO_REQUESTER, DeviceDescription, NumberOfMapRegisters)
}

/// pci.sys's BUS_INTERFACE_STANDARD.GetDmaAdapter: an adapter for the device with
/// this requester ID (its PCI segment above it), so what it maps reaches that
/// device's DMA-remapping domain if it has one.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosGetDmaAdapter(
    RequesterId: ddk::ULONG,
    DeviceDescription: ddk::PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: ddk::PULONG,
) -> ddk::PDMA_ADAPTER {
    new_adapter(RequesterId, DeviceDescription, NumberOfMapRegisters) as ddk::PDMA_ADAPTER
}

unsafe fn new_adapter(
    requester: u32,
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
    (*adapter).requester = requester;

    if !NumberOfMapRegisters.is_null() {
        // "Map registers" are free here, but the count is also the caller's answer to
        // "how many pages can one transfer of MaximumLength span", and drivers size
        // per-transfer tables from it. So report what x86/dma.rs reports for a bus
        // master, BYTES_TO_PAGES(MaximumLength) + 1, and 1 otherwise. A flat 256 made
        // pciidex's AHCI command table (128 + 16 bytes per entry) larger than the page
        // it packs tables into: zero tables per page, a 0-byte common buffer, and every
        // AHCI port failed to start.
        let master = !DeviceDescription.is_null() && (*DeviceDescription).Master != 0;
        *NumberOfMapRegisters = if master {
            let length = (*DeviceDescription).MaximumLength & 0x7FFF_FFFF;
            ((length as usize + PAGE_SIZE - 1) >> PAGE_SHIFT) as ddk::ULONG + 1
        } else {
            1
        };
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
    if !super::smmuv3::map(requester_of(AdapterObject as _), base_phys.QuadPart as u64, Length as u64) {
        ddk::MmFreeContiguousMemory(va);
        return core::ptr::null_mut();
    }
    if !LogicalAddress.is_null() {
        core::ptr::write_unaligned(LogicalAddress, base_phys);
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
        // The device stops reaching it before anything else can have it.
        super::smmuv3::unmap(requester_of(AdapterObject as _), LogicalAddress.QuadPart as u64, Length as u64);
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
    // Coherent, so nothing to flush; but the runs MapTransfer mapped for this
    // transfer are given back.
    let requester = requester_of(DmaAdapter);
    if !Mdl.is_null() && super::smmuv3::is_strict(requester) {
        for_each_run(Mdl, CurrentVa, Length, |pa, bytes| super::smmuv3::unmap(requester, pa, bytes));
    }
    1
}

/// Call `f` with each physically contiguous run of `Length` bytes of `Mdl`'s buffer
/// from `CurrentVa`, from the MDL's page list. A chain of MDLs continues into the
/// next one.
unsafe fn for_each_run(mut mdl: ddk::PMDL, current_va: ddk::PVOID, length: ddk::ULONG, mut f: impl FnMut(u64, u64)) {
    let mut left = length as usize;
    let mut va = current_va as usize;
    while !mdl.is_null() && left != 0 {
        let start = (*mdl).StartVa as usize + (*mdl).ByteOffset as usize;
        let end = start + (*mdl).ByteCount as usize;
        if va < start || va >= end {
            // Not in this MDL: a chained transfer starts each next MDL at its start.
            va = start;
        }
        let pfns = mdl.offset(1) as *const ddk::PFN_NUMBER;
        while va < end && left != 0 {
            let offset = va - (*mdl).StartVa as usize;
            let pa = ((*pfns.add(offset >> PAGE_SHIFT)) as u64) << PAGE_SHIFT | (offset & (PAGE_SIZE - 1)) as u64;
            // Extend while the next page follows physically.
            let mut run = (PAGE_SIZE - (offset & (PAGE_SIZE - 1))).min(end - va).min(left);
            while run < left && va + run < end {
                let next = (va + run - (*mdl).StartVa as usize) >> PAGE_SHIFT;
                if *pfns.add(next) != *pfns.add(next - 1) + 1 {
                    break;
                }
                run = (run + PAGE_SIZE).min(end - va).min(left);
            }
            f(pa, run as u64);
            va += run;
            left -= run;
        }
        mdl = (*mdl).Next;
    }
}

/// GetScatterGatherList: the buffer's physical runs, mapped into the device's domain
/// if it has one, handed to `ExecutionRoutine` at DISPATCH_LEVEL right away (there
/// are no map registers to wait for). PutScatterGatherList gives them back.
unsafe extern "system" fn Arm64DmaGetScatterGatherList(
    DmaAdapter: ddk::PDMA_ADAPTER,
    DeviceObject: ddk::PDEVICE_OBJECT,
    Mdl: ddk::PMDL,
    CurrentVa: ddk::PVOID,
    Length: ddk::ULONG,
    ExecutionRoutine: ddk::PDRIVER_LIST_CONTROL,
    Context: ddk::PVOID,
    WriteToDevice: ddk::BOOLEAN,
) -> ddk::NTSTATUS {
    type ListControl = unsafe extern "system" fn(ddk::PDEVICE_OBJECT, ddk::PIRP, ddk::PSCATTER_GATHER_LIST, ddk::PVOID);
    let Some(routine) = ExecutionRoutine else {
        return ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
    };
    let routine: ListControl = core::mem::transmute(routine);
    if Mdl.is_null() || Length == 0 {
        return ntkrnl_sys::constants::STATUS_INVALID_PARAMETER;
    }
    let mut runs = 0usize;
    for_each_run(Mdl, CurrentVa, Length, |_, _| runs += 1);
    let bytes = core::mem::size_of::<ddk::SCATTER_GATHER_LIST>() + runs * core::mem::size_of::<ddk::SCATTER_GATHER_ELEMENT>();
    let list = ddk::ExAllocatePoolWithTag(ddk::_POOL_TYPE_NonPagedPool, bytes as ddk::SIZE_T, HAL_DMA_TAG) as ddk::PSCATTER_GATHER_LIST;
    if list.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let requester = requester_of(DmaAdapter);
    let elements = (*list).Elements.as_mut_ptr();
    let (mut n, mut mapped) = (0usize, true);
    for_each_run(Mdl, CurrentVa, Length, |pa, len| {
        let e = elements.add(n);
        (*e).Address.QuadPart = pa as i64;
        (*e).Length = len as ddk::ULONG;
        (*e).Reserved = 0;
        n += 1;
        mapped &= super::smmuv3::map(requester, pa, len);
    });
    (*list).NumberOfElements = n as ddk::ULONG;
    (*list).Reserved = 0;
    if !mapped {
        Arm64DmaPutScatterGatherList(DmaAdapter, list, WriteToDevice);
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    let old_irql = KfRaiseIrql(DISPATCH_LEVEL);
    let irp = if DeviceObject.is_null() { core::ptr::null_mut() } else { (*DeviceObject).CurrentIrp };
    routine(DeviceObject, irp, list, Context);
    KfLowerIrql(old_irql);
    STATUS_SUCCESS
}

unsafe extern "system" fn Arm64DmaPutScatterGatherList(
    DmaAdapter: ddk::PDMA_ADAPTER,
    ScatterGather: ddk::PSCATTER_GATHER_LIST,
    WriteToDevice: ddk::BOOLEAN,
) {
    if ScatterGather.is_null() {
        return;
    }
    let requester = requester_of(DmaAdapter);
    let elements = (*ScatterGather).Elements.as_ptr();
    for i in 0..(*ScatterGather).NumberOfElements as usize {
        let e = elements.add(i);
        super::smmuv3::unmap(requester, (*e).Address.QuadPart as u64, (*e).Length as u64);
    }
    ddk::ExFreePoolWithTag(ScatterGather as ddk::PVOID, HAL_DMA_TAG);
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
    ntkrnl_sys::ASSERT!(offset >= byte_offset && offset < byte_offset + byte_count);

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
    // Until FlushAdapterBuffers gives it back.
    super::smmuv3::map(requester_of(DmaAdapter), phys.QuadPart as u64, *Length as u64);
    phys
}

unsafe extern "system" fn Arm64DmaGetDmaAlignment(DmaAdapter: ddk::PDMA_ADAPTER) -> ddk::ULONG {
    1
}

unsafe extern "system" fn Arm64DmaReadDmaCounter(DmaAdapter: ddk::PDMA_ADAPTER) -> ddk::ULONG {
    0
}
