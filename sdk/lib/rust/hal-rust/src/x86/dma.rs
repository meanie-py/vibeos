//! generic/dma.c — HAL DMA functions (ISA/EISA + bus-master).
//!
//! Map-register allocator, scatter-gather list building, DMA-controller
//! programming, MDL chunking, and the `HalGetAdapter` public API.
//! Compiled for both i386 and amd64.
//!
//! _MINIHAL_ guards dropped — Rust HAL never builds for minihal.
//! SARCH_PC98 dropped (we never build for PC-98).

use core::mem::offset_of;
use core::ptr;

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, ExFreePoolWithTag, ExQueueWorkItem, KeBugCheckEx,
    KeInitializeDeviceQueue, KeInitializeEvent, KeInsertDeviceQueue, KeRemoveDeviceQueue,
    KeSetEvent, KeWaitForSingleObject, MmAllocateContiguousMemorySpecifyCache,
    MmFreeContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmMapLockedPagesSpecifyCache,
    ObCreateObject, ObInsertObject, ObfDereferenceObject, ObfReferenceObject, RtlClearBit,
    RtlClearBits, RtlFindClearBitsAndSet, RtlInitializeBitMap, RtlSetAllBits, RtlSetBits, ZwClose,
};

// IoAllocateAdapterChannel is in ntoskrnl.spec.  Pull via the kernel
// import library (libntoskrnl.a — dlltool puts bare names in the IAT,
// which matches what ntoskrnl.exe actually exports).  Rust raw-dylib
// would generate `Name@N` import names instead, which don't resolve.
unsafe extern "system" {
    fn IoAllocateAdapterChannel(
        AdapterObject: PADAPTER_OBJECT,
        DeviceObject: PDEVICE_OBJECT,
        NumberOfMapRegisters: ULONG,
        ExecutionRoutine: PDRIVER_CONTROL,
        Context: PVOID,
    ) -> NTSTATUS;
}

// KeInitializeSpinLock is FORCEINLINE in NDK — no exported symbol.  Zero the lock.
#[allow(non_snake_case)]
#[inline]
unsafe fn KeInitializeSpinLock(sl: *mut KSPIN_LOCK) {
    unsafe { *sl = 0 }
}

// KeAcquireSpinLock / KeReleaseSpinLock are macros over
// KeAcquireSpinLockRaiseToDpc / KeReleaseSpinLock: raise to DISPATCH_LEVEL,
// then take the lock (release: drop the lock, then lower). These once only
// moved the IRQL, which is all the UP hal.dll needs, but this file is also
// built into halmp.dll (`smp` feature), where that left the DMA adapter list
// and every master adapter's map-register state with no mutual exclusion
// between CPUs. The lock half goes through spinlock.rs's funnel, the same one
// the exported Kf*SpinLock bodies use (and which stays a fence in mini_hal).
#[inline]
unsafe fn ke_acquire_spinlock(lock: *mut KSPIN_LOCK, old_irql: *mut KIRQL) {
    unsafe {
        *old_irql = kf_raise_irql(DISPATCH_LEVEL as KIRQL);
        crate::x86::spinlock::kx_acquire_spinlock(lock);
    }
}

#[inline]
unsafe fn ke_release_spinlock(lock: *mut KSPIN_LOCK, old_irql: KIRQL) {
    unsafe {
        crate::x86::spinlock::kx_release_spinlock(lock);
        kf_lower_irql(old_irql);
    }
}
use ntkrnl_sys::hal_ddk::{
    BOOLEAN, DEVICE_DESCRIPTION_VERSION1, DEVICE_DESCRIPTION_VERSION2, DISPATCH_LEVEL, DMA_OPERATIONS,
    DMA_PAGE, FILE_READ_DATA, FILE_WRITE_DATA, INTERFACE_TYPE, IoAdapterObjectType, KIRQL,
    LIST_ENTRY, MACHINE_TYPE_EISA, MAXLONG, MAXULONG, NTSTATUS, OBJ_KERNEL_HANDLE,
    OBJ_PERMANENT, OBJECT_ATTRIBUTES, PADAPTER_OBJECT, PALLOCATE_ADAPTER_CHANNEL,
    PALLOCATE_COMMON_BUFFER, PBUILD_MDL_FROM_SCATTER_GATHER_LIST, PBUILD_SCATTER_GATHER_LIST,
    PCALCULATE_SCATTER_GATHER_LIST_SIZE, PDEVICE_DESCRIPTION, PDEVICE_OBJECT, PDMA_ADAPTER,
    PDRIVER_CONTROL, PDRIVER_LIST_CONTROL, PFLUSH_ADAPTER_BUFFERS, PFREE_ADAPTER_CHANNEL,
    PFREE_COMMON_BUFFER, PFREE_MAP_REGISTERS, PGET_DMA_ALIGNMENT, PGET_SCATTER_GATHER_LIST,
    PHYSICAL_ADDRESS, PIRP, PKDEVICE_QUEUE_ENTRY, PMAP_TRANSFER, PMDL, PPFN_NUMBER,
    PPHYSICAL_ADDRESS, PPUT_DMA_ADAPTER, PPUT_SCATTER_GATHER_LIST, PREAD_DMA_COUNTER,
    PROS_MAP_REGISTER_ENTRY, PSCATTER_GATHER_LIST, PUCHAR, PVOID, PWAIT_CONTEXT_BLOCK,
    ROS_MAP_REGISTER_ENTRY, SCATTER_GATHER_ELEMENT, SCATTER_GATHER_LIST, SIZE_T, ULONG, ULONG64,
    ULONG_PTR, USHORT, WAIT_CONTEXT_BLOCK, _ADAPTER_OBJECT, _DMA1_CONTROL, _DMA2_CONTROL,
    _EISA_CONTROL, _KDEVICE_QUEUE_ENTRY, _MDL, _SCATTER_GATHER_LIST, _WAIT_CONTEXT_BLOCK,
    _DMA_MODE, _DMA_EXTENDED_MODE, _RTL_BITMAP as RTL_BITMAP,
    _DMA_SPEED_Compatible, _DMA_SPEED_TypeA, _DMA_SPEED_TypeB, _DMA_SPEED_TypeC,
    _DMA_WIDTH_Width16Bits, _DMA_WIDTH_Width32Bits, _DMA_WIDTH_Width8Bits, _EVENT_TYPE_NotificationEvent,
    _INTERFACE_TYPE_Eisa, _INTERFACE_TYPE_Isa, _INTERFACE_TYPE_PCIBus,
    _IO_ALLOCATION_ACTION_DeallocateObject, _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters,
    _IO_ALLOCATION_ACTION_KeepObject, _KWAIT_REASON_Executive, _MEMORY_CACHING_TYPE_MmCached,
    _MEMORY_CACHING_TYPE_MmNonCached, _MM_PAGE_PRIORITY_HighPagePriority,
    _POOL_TYPE_NonPagedPool, _WORK_QUEUE_TYPE_DelayedWorkQueue,
    B_16BITS, B_32BITS, B_8BITS, BURST_TIMING, CASCADE_REQUEST_MODE, COMPATIBLE_TIMING,
    DEMAND_REQUEST_MODE, DMA_CLEARMASK, DMA_SETMASK, MAP_BASE_SW_SG, READ_TRANSFER,
    SINGLE_REQUEST_MODE, TYPE_A_TIMING, TYPE_B_TIMING, WRITE_TRANSFER,
    UCHAR, KEVENT, KSPIN_LOCK, HANDLE,
};

// HalpBusType / HalpActiveProcessors / HalDispatchTable etc come from kernel exports;
// HalpBusType is hal-internal (defined in halinit.rs).
use ntkrnl_sys::hal_ddk::HalpBusType;

// HalDispatchTable lives in ntoskrnl; access via the bindgen raw-dylib decl.
use ntkrnl_sys::hal_ddk::HalDispatchTable;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PAGE_SIZE: ULONG = 4096;
const PAGE_SHIFT: ULONG = 12;
const TAG_DMA: ULONG = u32::from_le_bytes(*b"DMA ");
const MAX_MAP_REGISTERS: ULONG = 64;
// Upstream 442892d14d5 (#8910) raised this from 0x10 (0x20 in 5418987cb89):
// at 0x10 any S/G transfer spanning more than 16 pages (64 KiB) failed.
const MAX_SG_ELEMENTS: usize = 0x30;
const MDL_MAPPED_TO_SYSTEM_VA: USHORT = 0x0001;
const MDL_SOURCE_IS_NONPAGED_POOL: USHORT = 0x0004;
const HAL_MEMORY_ALLOCATION: ULONG = 0x0000_00AC;

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

#[inline]
fn bytes_to_pages(b: ULONG) -> ULONG {
    (b + PAGE_SIZE - 1) >> PAGE_SHIFT
}

#[inline]
fn byte_offset(va: ULONG_PTR) -> ULONG {
    (va & (PAGE_SIZE as ULONG_PTR - 1)) as ULONG
}

#[inline]
fn page_round_up(x: ULONG) -> ULONG {
    (x + PAGE_SIZE - 1) & !(PAGE_SIZE - 1)
}

// ---------------------------------------------------------------------------
// Local port-I/O — dma.c uses WRITE_PORT_UCHAR/READ_PORT_UCHAR which are
// NTAPI exports only on i386 (portio.rs).  amd64 expects inline intrinsics.
// Provide local cross-arch helpers (single `in`/`out`).
// ---------------------------------------------------------------------------

#[inline(always)]
unsafe fn write_port_uchar(port: *mut u8, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port as u16, in("al") value,
            options(nostack, preserves_flags),
        );
    }
}

#[inline(always)]
unsafe fn read_port_uchar(port: *const u8) -> u8 {
    let v: u8;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            in("dx") port as u16, out("al") v,
            options(nostack, preserves_flags),
        );
    }
    v
}

#[inline(always)]
unsafe fn ulong_to_ptr(v: ULONG) -> PVOID {
    v as usize as PVOID
}

// ---------------------------------------------------------------------------
// LIST_ENTRY helpers (FORCEINLINE in NDK — no exported symbol)
// ---------------------------------------------------------------------------

#[inline]
unsafe fn initialize_list_head(head: *mut LIST_ENTRY) {
    unsafe {
        (*head).Flink = head;
        (*head).Blink = head;
    }
}

#[inline]
unsafe fn is_list_empty(head: *const LIST_ENTRY) -> bool {
    unsafe { (*head).Flink == head as *mut _ }
}

#[inline]
unsafe fn insert_tail_list(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    unsafe {
        let blink = (*head).Blink;
        (*entry).Flink = head;
        (*entry).Blink = blink;
        (*blink).Flink = entry;
        (*head).Blink = entry;
    }
}

#[inline]
unsafe fn insert_head_list(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    unsafe {
        let flink = (*head).Flink;
        (*entry).Flink = flink;
        (*entry).Blink = head;
        (*flink).Blink = entry;
        (*head).Flink = entry;
    }
}

#[inline]
unsafe fn remove_entry_list(entry: *mut LIST_ENTRY) {
    unsafe {
        let flink = (*entry).Flink;
        let blink = (*entry).Blink;
        (*blink).Flink = flink;
        (*flink).Blink = blink;
    }
}

#[inline]
unsafe fn remove_head_list(head: *mut LIST_ENTRY) -> *mut LIST_ENTRY {
    unsafe {
        let entry = (*head).Flink;
        let flink = (*entry).Flink;
        (*head).Flink = flink;
        (*flink).Blink = head;
        entry
    }
}

// ---------------------------------------------------------------------------
// MDL helpers — FORCEINLINE in NDK (mmfuncs.h).
// ---------------------------------------------------------------------------

#[inline]
unsafe fn mm_get_mdl_pfn_array(mdl: PMDL) -> PPFN_NUMBER {
    unsafe { mdl.add(1) as PPFN_NUMBER }
}

#[inline]
unsafe fn mm_get_mdl_virtual_address(mdl: PMDL) -> PVOID {
    unsafe {
        ((*mdl).StartVa as ULONG_PTR + (*mdl).ByteOffset as ULONG_PTR) as PVOID
    }
}

#[inline]
unsafe fn mm_get_system_address_for_mdl_safe(mdl: PMDL, priority: u32) -> PVOID {
    unsafe {
        let flags = (*mdl).MdlFlags as u16;
        if flags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
            (*mdl).MappedSystemVa
        } else {
            MmMapLockedPagesSpecifyCache(
                mdl as *mut _,
                0u32 as _,
                _MEMORY_CACHING_TYPE_MmCached as _,
                ptr::null_mut(),
                0,
                priority as _,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Globals — symbol layout matches the C definitions in dma.c.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 82-87) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpDmaLock: KEVENT = unsafe { core::mem::zeroed() };

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpDmaAdapterListLock: KSPIN_LOCK = 0;

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpDmaAdapterList: LIST_ENTRY = LIST_ENTRY {
    Flink: ptr::null_mut(),
    Blink: ptr::null_mut(),
};

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpEisaAdapter: [PADAPTER_OBJECT; 8] = [ptr::null_mut(); 8];

// dma.c: NOT gated (line 88) — present under -D_MINIHAL_.
#[unsafe(no_mangle)]
pub static mut HalpEisaDma: BOOLEAN = 0;

// dma.c: inside `#ifndef _MINIHAL_` (lines 89-91) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpMasterAdapter: PADAPTER_OBJECT = ptr::null_mut();

// FIELD_OFFSET(DMA_PAGE, ChannelN) for channels 0..7 (4 = unused in DMA_PAGE).
const HALP_EISA_PORT_PAGE: [ULONG_PTR; 8] = [
    offset_of!(DMA_PAGE, Channel0) as ULONG_PTR,
    offset_of!(DMA_PAGE, Channel1) as ULONG_PTR,
    offset_of!(DMA_PAGE, Channel2) as ULONG_PTR,
    offset_of!(DMA_PAGE, Channel3) as ULONG_PTR,
    0,
    offset_of!(DMA_PAGE, Channel5) as ULONG_PTR,
    offset_of!(DMA_PAGE, Channel6) as ULONG_PTR,
    offset_of!(DMA_PAGE, Channel7) as ULONG_PTR,
];

// HalpDmaOperations — DMA_OPERATIONS dispatch table.  At static-init time
// just stamp Size; HalpInitDma installs every fn-pointer field via
// `core::mem::transmute(X as usize)` since bindgen models them as
// Option<extern "stdcall" fn(...)> but our exports are extern "system"
// (identical ABI on i386 but distinct Rust types).
// dma.c: inside `#ifndef _MINIHAL_` (lines 104-156) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub static mut HalpDmaOperations: DMA_OPERATIONS = unsafe {
    let mut ops: DMA_OPERATIONS = core::mem::zeroed();
    ops.Size = core::mem::size_of::<DMA_OPERATIONS>() as ULONG;
    ops
};

// ---------------------------------------------------------------------------
// HalpInitDma — boot-time setup.  Probes EISA DMA, allocates the master
// adapter, registers the PnP DMA-adapter dispatch in HalDispatchTable.
// ---------------------------------------------------------------------------

// C decl: VOID HalpInitDma(VOID) — bare cdecl (no NTAPI).  Bindgen declares it
// `extern "C"`, so callers (pic/apic_halinit.rs) expect the bare `_HalpInitDma`
// symbol on i386.  Must match cdecl here or LLVM IPO folds the call into ud2
// at the i386 call site (see feedback_ud2_abi_vs_captures.md).
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn HalpInitDma() {
    unsafe {
        HalpDmaOperations.PutDmaAdapter = Some(core::mem::transmute(HalPutDmaAdapter as usize));
        HalpDmaOperations.AllocateCommonBuffer =
            Some(core::mem::transmute(HalAllocateCommonBuffer as usize));
        HalpDmaOperations.FreeCommonBuffer =
            Some(core::mem::transmute(HalFreeCommonBuffer as usize));
        HalpDmaOperations.AllocateAdapterChannel =
            Some(core::mem::transmute(IoAllocateAdapterChannel as usize));
        HalpDmaOperations.FlushAdapterBuffers =
            Some(core::mem::transmute(IoFlushAdapterBuffers as usize));
        HalpDmaOperations.FreeAdapterChannel =
            Some(core::mem::transmute(IoFreeAdapterChannel as usize));
        HalpDmaOperations.FreeMapRegisters =
            Some(core::mem::transmute(IoFreeMapRegisters as usize));
        HalpDmaOperations.MapTransfer = Some(core::mem::transmute(IoMapTransfer as usize));
        HalpDmaOperations.GetDmaAlignment =
            Some(core::mem::transmute(HalpDmaGetDmaAlignment as usize));
        HalpDmaOperations.ReadDmaCounter =
            Some(core::mem::transmute(HalReadDmaCounter as usize));
        HalpDmaOperations.GetScatterGatherList =
            Some(core::mem::transmute(HalGetScatterGatherList as usize));
        HalpDmaOperations.PutScatterGatherList =
            Some(core::mem::transmute(HalPutScatterGatherList as usize));
        HalpDmaOperations.CalculateScatterGatherList =
            Some(core::mem::transmute(HalCalculateScatterGatherListSize as usize));
        HalpDmaOperations.BuildScatterGatherList =
            Some(core::mem::transmute(HalBuildScatterGatherList as usize));
        HalpDmaOperations.BuildMdlFromScatterGatherList =
            Some(core::mem::transmute(HalBuildMdlFromScatterGatherList as usize));

        if HalpBusType == MACHINE_TYPE_EISA as ULONG {
            // EISA presence probe — write+read DMA controller 2 page register.
            let port = ulong_to_ptr(
                offset_of!(_EISA_CONTROL, DmaController2Pages) as ULONG
                    + offset_of!(DMA_PAGE, Channel2) as ULONG,
            ) as *mut u8;
            write_port_uchar(port, 0x2A);
            if read_port_uchar(port) == 0x2A {
                HalpEisaDma = 1;
            }
        }

        initialize_list_head(&raw mut HalpDmaAdapterList);
        KeInitializeSpinLock(&raw mut HalpDmaAdapterListLock);
        KeInitializeEvent(
            &raw mut HalpDmaLock as *mut _,
            _EVENT_TYPE_NotificationEvent as _,
            1,
        );
        HalpMasterAdapter = HalpDmaAllocateMasterAdapter();

        // Register the PnP DMA-adapter dispatch (used by IoGetDmaAdapter).
        HalDispatchTable.HalGetDmaAdapter =
            Some(core::mem::transmute(HalpGetDmaAdapter as usize));
    }
}

// ---------------------------------------------------------------------------
// HalpGetAdapterMaximumPhysicalAddress
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetAdapterMaximumPhysicalAddress(
    AdapterObject: PADAPTER_OBJECT,
) -> PHYSICAL_ADDRESS {
    unsafe {
        let mut hi: PHYSICAL_ADDRESS = core::mem::zeroed();
        if (*AdapterObject).MasterDevice != 0 {
            if (*AdapterObject).Dma64BitAddresses != 0 {
                hi.QuadPart = -1; // 0xFFFFFFFFFFFFFFFF as i64
                return hi;
            } else if (*AdapterObject).Dma32BitAddresses != 0 {
                hi.QuadPart = 0xFFFF_FFFF;
                return hi;
            }
        }
        hi.QuadPart = 0xFF_FFFF; // 24-bit
        hi
    }
}

// ---------------------------------------------------------------------------
// HalpGrowMapBuffers — extend the master adapter's map-register array.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 240-488) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGrowMapBuffers(
    AdapterObject: PADAPTER_OBJECT,
    SizeOfMapBuffers: ULONG,
) -> BOOLEAN {
    unsafe {
        let mut map_register_count = bytes_to_pages(SizeOfMapBuffers);
        if map_register_count + (*AdapterObject).NumberOfMapRegisters > MAX_MAP_REGISTERS {
            return 0;
        }

        let mut highest = HalpGetAdapterMaximumPhysicalAddress(AdapterObject);
        let mut lowest: PHYSICAL_ADDRESS = core::mem::zeroed();
        // LowPart = HighestAcceptableAddress.LowPart == 0xFFFFFFFF ? 0x1000000 : 0
        let hi_low = (highest.QuadPart & 0xFFFF_FFFF) as ULONG;
        lowest.QuadPart = if hi_low == 0xFFFF_FFFF { 0x100_0000 } else { 0 };
        let boundary: PHYSICAL_ADDRESS = core::mem::zeroed();

        let mut va = MmAllocateContiguousMemorySpecifyCache(
            (map_register_count << PAGE_SHIFT) as SIZE_T,
            core::mem::transmute(lowest),
            core::mem::transmute(highest),
            core::mem::transmute(boundary),
            _MEMORY_CACHING_TYPE_MmNonCached as _,
        );
        if va.is_null() && (lowest.QuadPart & 0xFFFF_FFFF) != 0 {
            lowest.QuadPart = 0;
            va = MmAllocateContiguousMemorySpecifyCache(
                (map_register_count << PAGE_SHIFT) as SIZE_T,
                core::mem::transmute(lowest),
                core::mem::transmute(highest),
                core::mem::transmute(boundary),
                _MEMORY_CACHING_TYPE_MmNonCached as _,
            );
        }
        let _ = highest; // suppress unused mut

        if va.is_null() {
            return 0;
        }

        let phys_ddk = MmGetPhysicalAddress(va);
        let mut phys: PHYSICAL_ADDRESS = core::mem::transmute(phys_ddk);

        let mut old_irql: KIRQL = 0;
        ke_acquire_spinlock(&raw mut (*AdapterObject).SpinLock, &mut old_irql);

        if map_register_count > 0 {
            let mut current_entry: PROS_MAP_REGISTER_ENTRY = (*AdapterObject)
                .MapRegisterBase
                .add((*AdapterObject).NumberOfMapRegisters as usize);
            loop {
                // Leave one free entry for non-contiguous boundaries (so RtlFindClearBits
                // can find contiguous runs).  Non-EISA also leaves one per 64K boundary.
                if current_entry != (*AdapterObject).MapRegisterBase {
                    let previous_entry = current_entry.sub(1);
                    let prev_low = ((*previous_entry).PhysicalAddress.QuadPart & 0xFFFF_FFFF) as ULONG;
                    let cur_low = (phys.QuadPart & 0xFFFF_FFFF) as ULONG;
                    if prev_low.wrapping_add(PAGE_SIZE) == cur_low {
                        if HalpEisaDma == 0 && ((prev_low ^ cur_low) & 0xFFFF_0000) != 0 {
                            current_entry = current_entry.add(1);
                            (*AdapterObject).NumberOfMapRegisters += 1;
                        }
                    } else {
                        current_entry = current_entry.add(1);
                        (*AdapterObject).NumberOfMapRegisters += 1;
                    }
                }

                let index = current_entry.offset_from((*AdapterObject).MapRegisterBase) as ULONG;
                RtlClearBit((*AdapterObject).MapRegisters as *mut _, index);
                (*current_entry).VirtualAddress = va;
                (*current_entry).PhysicalAddress = phys;

                phys.QuadPart = phys.QuadPart.wrapping_add(PAGE_SIZE as i64);
                va = (va as ULONG_PTR + PAGE_SIZE as ULONG_PTR) as PVOID;

                current_entry = current_entry.add(1);
                (*AdapterObject).NumberOfMapRegisters += 1;
                map_register_count -= 1;
                if map_register_count == 0 {
                    break;
                }
            }
        }

        ke_release_spinlock(&raw mut (*AdapterObject).SpinLock, old_irql);

        1
    }
}

// ---------------------------------------------------------------------------
// HalpDmaAllocateMasterAdapter — allocate the system-wide master adapter.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 240-488) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpDmaAllocateMasterAdapter() -> PADAPTER_OBJECT {
    unsafe {
        let size_of_bitmap = MAX_MAP_REGISTERS;
        let mut size: ULONG = core::mem::size_of::<_ADAPTER_OBJECT>() as ULONG;
        size += core::mem::size_of::<RTL_BITMAP>() as ULONG;
        size += (size_of_bitmap + 7) >> 3;

        let master =
            ExAllocatePoolWithTag(_POOL_TYPE_NonPagedPool as _, size as SIZE_T, TAG_DMA)
                as PADAPTER_OBJECT;
        if master.is_null() {
            return ptr::null_mut();
        }
        ptr::write_bytes(master as *mut u8, 0, size as usize);

        KeInitializeSpinLock(&raw mut (*master).SpinLock);
        initialize_list_head(&raw mut (*master).AdapterQueue);

        let after_master = master.add(1) as *mut u8;
        (*master).MapRegisters = after_master as *mut RTL_BITMAP;
        RtlInitializeBitMap(
            (*master).MapRegisters as *mut _,
            after_master.add(core::mem::size_of::<RTL_BITMAP>()) as *mut ULONG,
            size_of_bitmap,
        );
        RtlSetAllBits((*master).MapRegisters as *mut _);
        (*master).NumberOfMapRegisters = 0;
        (*master).CommittedMapRegisters = 0;

        (*master).MapRegisterBase = ExAllocatePoolWithTag(
            _POOL_TYPE_NonPagedPool as _,
            (size_of_bitmap as SIZE_T) * core::mem::size_of::<ROS_MAP_REGISTER_ENTRY>() as SIZE_T,
            TAG_DMA,
        ) as PROS_MAP_REGISTER_ENTRY;
        if (*master).MapRegisterBase.is_null() {
            ExFreePool(master as PVOID);
            return ptr::null_mut();
        }

        ptr::write_bytes(
            (*master).MapRegisterBase as *mut u8,
            0,
            (size_of_bitmap as usize) * core::mem::size_of::<ROS_MAP_REGISTER_ENTRY>(),
        );
        if HalpGrowMapBuffers(master, 0x1_0000) == 0 {
            ExFreePool(master as PVOID);
            return ptr::null_mut();
        }

        master
    }
}

// ---------------------------------------------------------------------------
// HalpDmaAllocateChildAdapter — allocate a per-device adapter object.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 240-488) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpDmaAllocateChildAdapter(
    _NumberOfMapRegisters: ULONG,
    DeviceDescription: PDEVICE_DESCRIPTION,
) -> PADAPTER_OBJECT {
    unsafe {
        let mut adapter: PADAPTER_OBJECT = ptr::null_mut();
        let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
        oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        oa.RootDirectory = ptr::null_mut();
        oa.ObjectName = ptr::null_mut();
        oa.Attributes = OBJ_KERNEL_HANDLE | OBJ_PERMANENT;
        oa.SecurityDescriptor = ptr::null_mut();
        oa.SecurityQualityOfService = ptr::null_mut();

        let status = ObCreateObject(
            0u32 as _,
            IoAdapterObjectType as *mut _,
            &mut oa as *mut _ as _,
            0u32 as _,
            ptr::null_mut(),
            core::mem::size_of::<_ADAPTER_OBJECT>() as ULONG,
            0,
            0,
            &mut adapter as *mut PADAPTER_OBJECT as *mut PVOID,
        );
        if !nt_success(status) {
            return ptr::null_mut();
        }

        ptr::write_bytes(adapter as *mut u8, 0, core::mem::size_of::<_ADAPTER_OBJECT>());

        let mut handle: HANDLE = ptr::null_mut();
        let status = ObInsertObject(
            adapter as PVOID,
            ptr::null_mut(),
            FILE_READ_DATA | FILE_WRITE_DATA,
            0,
            ptr::null_mut(),
            &mut handle,
        );
        if !nt_success(status) {
            return ptr::null_mut();
        }

        ObfReferenceObject(adapter as PVOID);
        ZwClose(handle);

        (*adapter).DmaHeader.Version = (*DeviceDescription).Version as USHORT;
        (*adapter).DmaHeader.Size = core::mem::size_of::<_ADAPTER_OBJECT>() as USHORT;
        (*adapter).DmaHeader.DmaOperations = &raw mut HalpDmaOperations;
        (*adapter).MapRegistersPerChannel = 1;
        (*adapter).Dma32BitAddresses = (*DeviceDescription).Dma32BitAddresses;
        (*adapter).ChannelNumber = 0xFF;
        (*adapter).MasterAdapter = HalpMasterAdapter;
        KeInitializeDeviceQueue(&raw mut (*adapter).ChannelWaitQueue as *mut _);

        adapter
    }
}

// ---------------------------------------------------------------------------
// HalpDmaInitializeEisaAdapter — set up (E)ISA DMA mode + extended mode.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpDmaInitializeEisaAdapter(
    AdapterObject: PADAPTER_OBJECT,
    DeviceDescription: PDEVICE_DESCRIPTION,
) -> BOOLEAN {
    unsafe {
        let mut dma_mode: _DMA_MODE = core::mem::zeroed();
        let mut ext_mode: _DMA_EXTENDED_MODE = core::mem::zeroed();

        let controller: UCHAR = if (*DeviceDescription).DmaChannel & 4 != 0 { 2 } else { 1 };
        let base_va: PVOID = if controller == 1 {
            ulong_to_ptr(offset_of!(_EISA_CONTROL, DmaController1) as ULONG)
        } else {
            ulong_to_ptr(offset_of!(_EISA_CONTROL, DmaController2) as ULONG)
        };

        (*AdapterObject).AdapterNumber = controller;
        (*AdapterObject).ChannelNumber = ((*DeviceDescription).DmaChannel & 3) as UCHAR;
        (*AdapterObject).PagePort =
            HALP_EISA_PORT_PAGE[(*DeviceDescription).DmaChannel as usize] as PUCHAR;
        (*AdapterObject).Width16Bits = 0;
        (*AdapterObject).AdapterBaseVa = base_va;

        if HalpEisaDma != 0 {
            ext_mode.__bindgen_anon_1.set_ChannelNumber((*AdapterObject).ChannelNumber);

            let speed = (*DeviceDescription).DmaSpeed;
            let timing: UCHAR = if speed == _DMA_SPEED_Compatible {
                COMPATIBLE_TIMING as UCHAR
            } else if speed == _DMA_SPEED_TypeA {
                TYPE_A_TIMING as UCHAR
            } else if speed == _DMA_SPEED_TypeB {
                TYPE_B_TIMING as UCHAR
            } else if speed == _DMA_SPEED_TypeC {
                BURST_TIMING as UCHAR
            } else {
                return 0;
            };
            ext_mode.__bindgen_anon_1.set_TimingMode(timing);

            let width = (*DeviceDescription).DmaWidth;
            let xfer: UCHAR = if width == _DMA_WIDTH_Width8Bits {
                B_8BITS as UCHAR
            } else if width == _DMA_WIDTH_Width16Bits {
                B_16BITS as UCHAR
            } else if width == _DMA_WIDTH_Width32Bits {
                B_32BITS as UCHAR
            } else {
                return 0;
            };
            ext_mode.__bindgen_anon_1.set_TransferSize(xfer);

            let port = if controller == 1 {
                ulong_to_ptr(offset_of!(_EISA_CONTROL, DmaExtendedMode1) as ULONG) as *mut u8
            } else {
                ulong_to_ptr(offset_of!(_EISA_CONTROL, DmaExtendedMode2) as ULONG) as *mut u8
            };
            write_port_uchar(port, ext_mode.Byte);
        } else if (*DeviceDescription).Master == 0 {
            if controller == 2 && (*DeviceDescription).DmaWidth == _DMA_WIDTH_Width16Bits {
                (*AdapterObject).Width16Bits = 1;
            } else if controller != 1 || (*DeviceDescription).DmaWidth != _DMA_WIDTH_Width8Bits {
                return 0;
            }
        }

        dma_mode.__bindgen_anon_1.set_Channel((*AdapterObject).ChannelNumber);
        dma_mode.__bindgen_anon_1.set_AutoInitialize((*DeviceDescription).AutoInitialize);

        if (*DeviceDescription).Master != 0 {
            dma_mode.__bindgen_anon_1.set_RequestMode(CASCADE_REQUEST_MODE as UCHAR);
            if controller == 1 {
                let ctrl = base_va as *mut _DMA1_CONTROL;
                write_port_uchar(&raw mut (*ctrl).Mode, dma_mode.Byte);
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_CLEARMASK as UCHAR,
                );
            } else {
                let ctrl = base_va as *mut _DMA2_CONTROL;
                write_port_uchar(&raw mut (*ctrl).Mode, dma_mode.Byte);
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_CLEARMASK as UCHAR,
                );
            }
        } else if (*DeviceDescription).DemandMode != 0 {
            dma_mode.__bindgen_anon_1.set_RequestMode(DEMAND_REQUEST_MODE as UCHAR);
        } else {
            dma_mode.__bindgen_anon_1.set_RequestMode(SINGLE_REQUEST_MODE as UCHAR);
        }

        (*AdapterObject).AdapterMode = dma_mode;
        1
    }
}

// ---------------------------------------------------------------------------
// HalGetAdapter — public DMA-adapter factory.
// ---------------------------------------------------------------------------

/// pci.sys's BUS_INTERFACE_STANDARD.GetDmaAdapter, naming the device (its PCI
/// segment over its requester ID). Only the arm64 HAL drives an IOMMU that needs to
/// know whose buffers it maps; here it is HalGetAdapter.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalVibeosGetDmaAdapter(
    _RequesterId: ULONG,
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PDMA_ADAPTER {
    unsafe { HalGetAdapter(DeviceDescription, NumberOfMapRegisters) as PDMA_ADAPTER }
}

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetAdapter(
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PADAPTER_OBJECT {
    unsafe {
        let mut adapter: PADAPTER_OBJECT = ptr::null_mut();

        if (*DeviceDescription).Version > DEVICE_DESCRIPTION_VERSION2 {
            return ptr::null_mut();
        }

        let iface = (*DeviceDescription).InterfaceType;
        let mut eisa_adapter = (iface == _INTERFACE_TYPE_Eisa
            || iface == _INTERFACE_TYPE_Isa
            || (*DeviceDescription).Master == 0)
            && !((iface == _INTERFACE_TYPE_Isa || iface == _INTERFACE_TYPE_Eisa)
                && (*DeviceDescription).DmaChannel >= 8);

        // The C predicate falls through differently: outer EisaAdapter
        // is FALSE if (Eisa or Isa) AND DmaChannel >= 8; else if any of
        // (Eisa, Isa, !Master) is true, EisaAdapter is TRUE; else FALSE.
        // Reproduce exactly:
        let eisa_adapter = if iface == _INTERFACE_TYPE_Eisa
            || iface == _INTERFACE_TYPE_Isa
            || (*DeviceDescription).Master == 0
        {
            if (iface == _INTERFACE_TYPE_Isa || iface == _INTERFACE_TYPE_Eisa)
                && (*DeviceDescription).DmaChannel >= 8
            {
                false
            } else {
                true
            }
        } else {
            false
        };
        let _ = eisa_adapter; // shadowing — first read keeps compiler happy

        // Disallow ISA/EISA channel 4 (used for cascading).
        if eisa_adapter && (*DeviceDescription).DmaChannel == 4 {
            return ptr::null_mut();
        }

        // Compute number of map registers.
        let maximum_length = (*DeviceDescription).MaximumLength & MAXLONG;
        let map_registers: ULONG = if (*DeviceDescription).ScatterGather != 0
            && (iface == _INTERFACE_TYPE_Eisa || iface == _INTERFACE_TYPE_PCIBus)
        {
            0
        } else if (*DeviceDescription).ScatterGather != 0 && (*DeviceDescription).Master == 0 {
            1
        } else {
            let mut mr = bytes_to_pages(maximum_length) + 1;
            if mr > 16 {
                mr = 16;
            }
            mr
        };

        KeWaitForSingleObject(
            &raw mut HalpDmaLock as *mut _,
            _KWAIT_REASON_Executive as _,
            0u32 as _,
            0,
            ptr::null_mut(),
        );

        if eisa_adapter {
            adapter = HalpEisaAdapter[(*DeviceDescription).DmaChannel as usize];
            if !adapter.is_null()
                && (*adapter).NeedsMapRegisters != 0
                && map_registers > (*adapter).MapRegistersPerChannel
            {
                (*adapter).MapRegistersPerChannel = map_registers;
            }
        }

        if adapter.is_null() {
            adapter = HalpDmaAllocateChildAdapter(map_registers, DeviceDescription);
            if adapter.is_null() {
                KeSetEvent(&raw mut HalpDmaLock as *mut _, 0, 0);
                return ptr::null_mut();
            }

            if eisa_adapter {
                HalpEisaAdapter[(*DeviceDescription).DmaChannel as usize] = adapter;
            }

            if map_registers > 0 {
                (*adapter).NeedsMapRegisters = 1;
                (*adapter).MapRegistersPerChannel = map_registers;
            } else {
                (*adapter).NeedsMapRegisters = 0;
                (*adapter).MapRegistersPerChannel = if (*DeviceDescription).Master != 0 {
                    bytes_to_pages(maximum_length) + 1
                } else {
                    1
                };
            }
        }

        KeSetEvent(&raw mut HalpDmaLock as *mut _, 0, 0);

        if !eisa_adapter {
            let mut old_irql: KIRQL = 0;
            ke_acquire_spinlock(&raw mut HalpDmaAdapterListLock, &mut old_irql);
            insert_tail_list(
                &raw mut HalpDmaAdapterList,
                &raw mut (*adapter).AdapterList,
            );
            ke_release_spinlock(&raw mut HalpDmaAdapterListLock, old_irql);
        }

        if (*DeviceDescription).Version >= DEVICE_DESCRIPTION_VERSION1 {
            (*adapter).IgnoreCount = (*DeviceDescription).IgnoreCount;
        } else {
            (*adapter).IgnoreCount = 0;
        }

        (*adapter).Dma32BitAddresses = (*DeviceDescription).Dma32BitAddresses;
        (*adapter).Dma64BitAddresses = (*DeviceDescription).Dma64BitAddresses;
        (*adapter).ScatterGather = (*DeviceDescription).ScatterGather;
        (*adapter).MasterDevice = (*DeviceDescription).Master;
        *NumberOfMapRegisters = (*adapter).MapRegistersPerChannel;

        if eisa_adapter && HalpDmaInitializeEisaAdapter(adapter, DeviceDescription) == 0 {
            ObfDereferenceObject(adapter as PVOID);
            return ptr::null_mut();
        }

        adapter
    }
}

// ---------------------------------------------------------------------------
// HalpGetDmaAdapter — exported via HalDispatchTable for IoGetDmaAdapter.
// ---------------------------------------------------------------------------

#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetDmaAdapter(
    _Context: PVOID,
    DeviceDescription: PDEVICE_DESCRIPTION,
    NumberOfMapRegisters: *mut ULONG,
) -> PDMA_ADAPTER {
    unsafe {
        let adapter = HalGetAdapter(DeviceDescription, NumberOfMapRegisters);
        if adapter.is_null() {
            ptr::null_mut()
        } else {
            &raw mut (*adapter).DmaHeader
        }
    }
}

// ---------------------------------------------------------------------------
// HalPutDmaAdapter
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalPutDmaAdapter(AdapterObject: PADAPTER_OBJECT) {
    unsafe {
        if (*AdapterObject).ChannelNumber == 0xFF {
            let mut old_irql: KIRQL = 0;
            ke_acquire_spinlock(&raw mut HalpDmaAdapterListLock, &mut old_irql);
            remove_entry_list(&raw mut (*AdapterObject).AdapterList);
            ke_release_spinlock(&raw mut HalpDmaAdapterListLock, old_irql);
        }
        ObfDereferenceObject(AdapterObject as PVOID);
    }
}

// ---------------------------------------------------------------------------
// HalAllocateCommonBuffer
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateCommonBuffer(
    AdapterObject: PDMA_ADAPTER,
    Length: ULONG,
    LogicalAddress: PPHYSICAL_ADDRESS,
    CacheEnabled: BOOLEAN,
) -> PVOID {
    unsafe {
        // PDMA_ADAPTER -> PADAPTER_OBJECT (DmaHeader is the first field).
        let adapter = AdapterObject as PADAPTER_OBJECT;

        let mut lowest: PHYSICAL_ADDRESS = core::mem::zeroed();
        lowest.QuadPart = 0;
        let highest = HalpGetAdapterMaximumPhysicalAddress(adapter);
        let mut boundary: PHYSICAL_ADDRESS = core::mem::zeroed();

        // Bus-master => 4Gb; slave => 64Kb.
        if (*adapter).MasterDevice != 0 {
            boundary.QuadPart = 0x1_0000_0000;
        } else {
            boundary.QuadPart = 0x10000;
        }

        let cache = if CacheEnabled != 0 {
            _MEMORY_CACHING_TYPE_MmCached
        } else {
            _MEMORY_CACHING_TYPE_MmNonCached
        };
        let va = MmAllocateContiguousMemorySpecifyCache(
            Length as SIZE_T,
            core::mem::transmute(lowest),
            core::mem::transmute(highest),
            core::mem::transmute(boundary),
            cache as _,
        );
        if va.is_null() {
            return ptr::null_mut();
        }
        core::ptr::write_unaligned(LogicalAddress, core::mem::transmute(MmGetPhysicalAddress(va)));
        va
    }
}

// ---------------------------------------------------------------------------
// HalFreeCommonBuffer
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFreeCommonBuffer(
    _AdapterObject: PDMA_ADAPTER,
    Length: ULONG,
    _LogicalAddress: PHYSICAL_ADDRESS,
    VirtualAddress: PVOID,
    CacheEnabled: BOOLEAN,
) {
    unsafe {
        let cache = if CacheEnabled != 0 {
            _MEMORY_CACHING_TYPE_MmCached
        } else {
            _MEMORY_CACHING_TYPE_MmNonCached
        };
        MmFreeContiguousMemorySpecifyCache(VirtualAddress, Length as SIZE_T, cache as _);
    }
}

// ---------------------------------------------------------------------------
// SCATTER_GATHER_CONTEXT — private layout used during S/G DMA bookkeeping.
// ---------------------------------------------------------------------------

#[repr(C)]
struct ScatterGatherContext {
    using_user_buffer: BOOLEAN,
    adapter_object: PADAPTER_OBJECT,
    mdl: PMDL,
    current_va: PUCHAR,
    length: ULONG,
    adapter_list_control_routine: PDRIVER_LIST_CONTROL,
    adapter_list_control_context: PVOID,
    map_register_base: PVOID,
    map_register_count: ULONG,
    write_to_device: BOOLEAN,
    wcb: _WAIT_CONTEXT_BLOCK,
}

// ---------------------------------------------------------------------------
// HalpScatterGatherAdapterControl — adapter-control callback used during S/G.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpScatterGatherAdapterControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    MapRegisterBase: PVOID,
    Context: PVOID,
) -> u32 {
    unsafe {
        let ctx = Context as *mut ScatterGatherContext;
        let adapter = (*ctx).adapter_object;
        let mut element_count = 0usize;
        let mut remaining = (*ctx).length;
        let current_va = (*ctx).current_va;
        // RemainingLength / PAGE_SIZE + 1 for the remainder, + 1 cushion
        // (upstream 442892d14d5, #8910): a buffer of that length spans at most
        // that many pages, and IoMapTransfer never returns less than one page
        // run per element, so this bounds the element count.
        let est_sg_elements =
            core::cmp::min((remaining / PAGE_SIZE + 2) as usize, MAX_SG_ELEMENTS);

        (*ctx).map_register_base = MapRegisterBase;

        // Upstream 5418987cb89 moved this scratch array off the stack into pool
        // (it is 0x30 elements now).
        let elements = ExAllocatePoolWithTag(
            _POOL_TYPE_NonPagedPool as _,
            (core::mem::size_of::<SCATTER_GATHER_ELEMENT>() * est_sg_elements) as SIZE_T,
            TAG_DMA,
        ) as *mut SCATTER_GATHER_ELEMENT;
        if elements.is_null() {
            return _IO_ALLOCATION_ACTION_DeallocateObject as u32;
        }
        // Zeroed so the slice below never views uninitialised memory.
        ptr::write_bytes(elements, 0, est_sg_elements);
        let elements = core::slice::from_raw_parts_mut(elements, est_sg_elements);

        // Bounded by the allocation, not MAX_SG_ELEMENTS as in the C: the two
        // only agree while the estimate above holds, and if it ever does not,
        // running out of elements fails the transfer below instead of
        // overrunning the pool block.
        while remaining > 0 && element_count < est_sg_elements {
            elements[element_count].Length = remaining;
            elements[element_count].Reserved = 0;
            let mut len_local = elements[element_count].Length;
            elements[element_count].Address = IoMapTransfer(
                adapter,
                (*ctx).mdl,
                MapRegisterBase,
                current_va.add(((*ctx).length - remaining) as usize) as PVOID,
                &mut len_local,
                (*ctx).write_to_device,
            );
            elements[element_count].Length = len_local;
            if elements[element_count].Length == 0 {
                break;
            }
            ntkrnl_sys::ASSERT!(elements[element_count].Length <= remaining);
            remaining -= elements[element_count].Length;
            element_count += 1;
        }

        if remaining > 0 {
            ExFreePoolWithTag(elements.as_mut_ptr() as PVOID, TAG_DMA);
            return _IO_ALLOCATION_ACTION_DeallocateObject as u32;
        }

        let sg_size = core::mem::size_of::<SCATTER_GATHER_LIST>()
            + core::mem::size_of::<SCATTER_GATHER_ELEMENT>() * element_count;
        let sg = ExAllocatePoolWithTag(_POOL_TYPE_NonPagedPool as _, sg_size as SIZE_T, TAG_DMA)
            as PSCATTER_GATHER_LIST;
        ntkrnl_sys::ASSERT!(!sg.is_null());

        (*sg).NumberOfElements = element_count as ULONG;
        (*sg).Reserved = ctx as ULONG_PTR;
        ptr::copy_nonoverlapping(
            elements.as_ptr(),
            (*sg).Elements.as_mut_ptr(),
            element_count,
        );
        ExFreePoolWithTag(elements.as_mut_ptr() as PVOID, TAG_DMA);

        if let Some(routine) = (*ctx).adapter_list_control_routine {
            routine(
                DeviceObject,
                Irp,
                sg,
                (*ctx).adapter_list_control_context,
            );
        }

        _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as u32
    }
}

// ---------------------------------------------------------------------------
// HalGetScatterGatherList — convenience wrapper around HalBuildScatterGatherList.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalGetScatterGatherList(
    AdapterObject: PDMA_ADAPTER,
    DeviceObject: PDEVICE_OBJECT,
    Mdl: PMDL,
    CurrentVa: PVOID,
    Length: ULONG,
    ExecutionRoutine: PDRIVER_LIST_CONTROL,
    Context: PVOID,
    WriteToDevice: BOOLEAN,
) -> NTSTATUS {
    unsafe {
        HalBuildScatterGatherList(
            AdapterObject,
            DeviceObject,
            Mdl,
            CurrentVa,
            Length,
            ExecutionRoutine,
            Context,
            WriteToDevice,
            ptr::null_mut(),
            0,
        )
    }
}

// ---------------------------------------------------------------------------
// HalPutScatterGatherList
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalPutScatterGatherList(
    AdapterObject: PDMA_ADAPTER,
    ScatterGather: PSCATTER_GATHER_LIST,
    _WriteToDevice: BOOLEAN,
) {
    unsafe {
        let ctx = (*ScatterGather).Reserved as *mut ScatterGatherContext;
        let adapter = AdapterObject as PADAPTER_OBJECT;
        for i in 0..(*ScatterGather).NumberOfElements {
            let elem = (*ScatterGather).Elements.as_ptr().add(i as usize);
            IoFlushAdapterBuffers(
                adapter,
                (*ctx).mdl,
                (*ctx).map_register_base,
                (*ctx).current_va as PVOID,
                (*elem).Length,
                (*ctx).write_to_device,
            );
            (*ctx).current_va = (*ctx).current_va.add((*elem).Length as usize);
        }

        IoFreeMapRegisters(adapter, (*ctx).map_register_base, (*ctx).map_register_count);
        ExFreePoolWithTag(ScatterGather as PVOID, TAG_DMA);
        if (*ctx).using_user_buffer == 0 {
            ExFreePoolWithTag(ctx as PVOID, TAG_DMA);
        }
    }
}

// ---------------------------------------------------------------------------
// HalCalculateScatterGatherListSize
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalCalculateScatterGatherListSize(
    _AdapterObject: PDMA_ADAPTER,
    _Mdl: PMDL,
    _CurrentVa: PVOID,
    Length: ULONG,
    ScatterGatherListSize: *mut ULONG,
    pNumberOfMapRegisters: *mut ULONG,
) -> NTSTATUS {
    unsafe {
        let number_of_map_registers = page_round_up(Length) >> PAGE_SHIFT;
        let sg_size = core::mem::size_of::<ScatterGatherContext>() as ULONG;
        *ScatterGatherListSize = sg_size;
        if !pNumberOfMapRegisters.is_null() {
            *pNumberOfMapRegisters = number_of_map_registers;
        }
        0 // STATUS_SUCCESS
    }
}

// ---------------------------------------------------------------------------
// HalBuildScatterGatherList
// ---------------------------------------------------------------------------

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC000_0023u32 as NTSTATUS;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000_009Au32 as NTSTATUS;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC000_0002u32 as NTSTATUS;

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalBuildScatterGatherList(
    AdapterObject: PDMA_ADAPTER,
    DeviceObject: PDEVICE_OBJECT,
    Mdl: PMDL,
    CurrentVa: PVOID,
    Length: ULONG,
    ExecutionRoutine: PDRIVER_LIST_CONTROL,
    Context: PVOID,
    WriteToDevice: BOOLEAN,
    mut ScatterGatherBuffer: PVOID,
    ScatterGatherBufferLength: ULONG,
) -> NTSTATUS {
    unsafe {
        let adapter = AdapterObject as PADAPTER_OBJECT;
        let mut sg_size: ULONG = 0;
        let mut num_map_regs: ULONG = 0;
        let status = HalCalculateScatterGatherListSize(
            AdapterObject,
            Mdl,
            CurrentVa,
            Length,
            &mut sg_size,
            &mut num_map_regs,
        );
        if !nt_success(status) {
            return status;
        }

        let using_user_buffer = !ScatterGatherBuffer.is_null();
        if using_user_buffer {
            if ScatterGatherBufferLength < sg_size {
                return STATUS_BUFFER_TOO_SMALL;
            }
        } else {
            ScatterGatherBuffer = ExAllocatePoolWithTag(
                _POOL_TYPE_NonPagedPool as _,
                sg_size as SIZE_T,
                TAG_DMA,
            );
            if ScatterGatherBuffer.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        let ctx = ScatterGatherBuffer as *mut ScatterGatherContext;
        (*ctx).using_user_buffer = if using_user_buffer { 1 } else { 0 };
        (*ctx).adapter_object = adapter;
        (*ctx).mdl = Mdl;
        (*ctx).current_va = CurrentVa as PUCHAR;
        (*ctx).length = Length;
        (*ctx).map_register_count = num_map_regs;
        (*ctx).adapter_list_control_routine = ExecutionRoutine;
        (*ctx).adapter_list_control_context = Context;
        (*ctx).write_to_device = WriteToDevice;

        (*ctx).wcb.DeviceObject = DeviceObject as PVOID;
        (*ctx).wcb.DeviceContext = ctx as PVOID;
        (*ctx).wcb.CurrentIrp = (*DeviceObject).CurrentIrp as PVOID;

        let status = HalAllocateAdapterChannel(
            adapter,
            &raw mut (*ctx).wcb,
            num_map_regs,
            Some(core::mem::transmute(HalpScatterGatherAdapterControl as usize)),
        );
        if !nt_success(status) {
            if !using_user_buffer {
                ExFreePoolWithTag(ScatterGatherBuffer, TAG_DMA);
            }
            return status;
        }

        STATUS_SUCCESS
    }
}

// ---------------------------------------------------------------------------
// HalBuildMdlFromScatterGatherList — unimplemented in C.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 626-1287) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalBuildMdlFromScatterGatherList(
    _DmaAdapter: PDMA_ADAPTER,
    _ScatterGather: PSCATTER_GATHER_LIST,
    _OriginalMdl: PMDL,
    _TargetMdl: *mut PMDL,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// ---------------------------------------------------------------------------
// HalpDmaGetDmaAlignment
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpDmaGetDmaAlignment(_AdapterObject: PDMA_ADAPTER) -> ULONG {
    1
}

// ---------------------------------------------------------------------------
// HalReadDmaCounter
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalReadDmaCounter(AdapterObject: PADAPTER_OBJECT) -> ULONG {
    unsafe {
        ntkrnl_sys::ASSERT!((*AdapterObject).MasterDevice == 0);

        let master = (*AdapterObject).MasterAdapter;
        let mut old_irql: KIRQL = 0;
        ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);

        let mut count: ULONG = 0xFFFF00;
        if (*AdapterObject).AdapterNumber == 1 {
            let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA1_CONTROL;
            loop {
                let old = count;
                write_port_uchar(&raw mut (*ctrl).ClearBytePointer, 0);
                let ch = (*AdapterObject).ChannelNumber as usize;
                let lo = read_port_uchar(&raw const (*ctrl).DmaAddressCount[ch].DmaBaseCount);
                let hi = read_port_uchar(&raw const (*ctrl).DmaAddressCount[ch].DmaBaseCount);
                count = lo as ULONG | ((hi as ULONG) << 8);
                if 0xFFFF00 & (old ^ count) == 0 {
                    break;
                }
            }
        } else {
            let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA2_CONTROL;
            loop {
                let old = count;
                write_port_uchar(&raw mut (*ctrl).ClearBytePointer, 0);
                let ch = (*AdapterObject).ChannelNumber as usize;
                let lo = read_port_uchar(&raw const (*ctrl).DmaAddressCount[ch].DmaBaseCount);
                let hi = read_port_uchar(&raw const (*ctrl).DmaAddressCount[ch].DmaBaseCount);
                count = lo as ULONG | ((hi as ULONG) << 8);
                if 0xFFFF00 & (old ^ count) == 0 {
                    break;
                }
            }
        }

        ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);

        count = count.wrapping_add(1) & 0xFFFF;
        if (*AdapterObject).Width16Bits != 0 {
            count *= 2;
        }
        count
    }
}

// ---------------------------------------------------------------------------
// HalpGrowMapBufferWorker — PASSIVE_LEVEL work item to extend map registers.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGrowMapBufferWorker(DeferredContext: PVOID) {
    unsafe {
        let work_item = DeferredContext as *mut _GROW_WORK_ITEM_LOCAL;
        KeWaitForSingleObject(
            &raw mut HalpDmaLock as *mut _,
            _KWAIT_REASON_Executive as _,
            0u32 as _,
            0,
            ptr::null_mut(),
        );
        let succeeded = HalpGrowMapBuffers(
            (*(*work_item).adapter_object).MasterAdapter,
            (*work_item).number_of_map_registers << PAGE_SHIFT,
        );
        KeSetEvent(&raw mut HalpDmaLock as *mut _, 0, 0);

        if succeeded != 0 {
            // Flush adapter queue: trick is to call IoFreeMapRegisters with
            // a sentinel base + zero count so it just runs the queue.
            let old_irql = kf_raise_irql(DISPATCH_LEVEL as KIRQL);
            IoFreeMapRegisters((*work_item).adapter_object, 2usize as PVOID, 0);
            kf_lower_irql(old_irql);
        }

        ExFreePool(work_item as PVOID);
    }
}

// _GROW_WORK_ITEM bindgen layout: WORK_QUEUE_ITEM (12 bytes i386 / 24 bytes amd64)
// + PADAPTER_OBJECT + ULONG.  Define a local mirror so we can fill the work-item
// fields without touching bindgen accessors.
#[repr(C)]
struct _GROW_WORK_ITEM_LOCAL {
    work_queue_item: ntkrnl_sys::hal_ddk::WORK_QUEUE_ITEM,
    adapter_object: PADAPTER_OBJECT,
    number_of_map_registers: ULONG,
}

// KfRaiseIrql/KfLowerIrql — fastcall on i386, "C" on amd64 (per ke-inlines).
#[cfg(target_arch = "x86")]
unsafe fn kf_raise_irql(new_irql: KIRQL) -> KIRQL {
    extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
    }
    unsafe { KfRaiseIrql(new_irql) }
}
#[cfg(target_arch = "x86_64")]
unsafe fn kf_raise_irql(_new_irql: KIRQL) -> KIRQL {
    // amd64 doesn't export KfRaiseIrql — use KeRaiseIrqlToDpcLevel via
    // raw-dylib (so the linker resolves it via the ntoskrnl IAT instead of
    // expecting a local strong symbol).  Caller always passes DISPATCH_LEVEL.
    #[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
    unsafe extern "system" {
        fn KeRaiseIrqlToDpcLevel() -> KIRQL;
    }
    unsafe { KeRaiseIrqlToDpcLevel() }
}

#[cfg(target_arch = "x86")]
unsafe fn kf_lower_irql(old_irql: KIRQL) {
    extern "fastcall" {
        fn KfLowerIrql(NewIrql: KIRQL);
    }
    unsafe { KfLowerIrql(old_irql) }
}
#[cfg(target_arch = "x86_64")]
unsafe fn kf_lower_irql(old_irql: KIRQL) {
    #[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
    unsafe extern "C" {
        fn KeLowerIrql(NewIrql: KIRQL);
    }
    unsafe { KeLowerIrql(old_irql) }
}

// ExInitializeWorkItem — FORCEINLINE in NDK.
#[inline]
unsafe fn ex_initialize_work_item(
    item: *mut ntkrnl_sys::hal_ddk::WORK_QUEUE_ITEM,
    routine: unsafe extern "system" fn(PVOID),
    context: PVOID,
) {
    unsafe {
        (*item).List.Flink = ptr::null_mut();
        (*item).List.Blink = ptr::null_mut();
        (*item).WorkerRoutine = Some(core::mem::transmute(routine as usize));
        (*item).Parameter = context;
    }
}

// ---------------------------------------------------------------------------
// HalAllocateAdapterChannel — reserve map registers and dispatch to caller.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateAdapterChannel(
    AdapterObject: PADAPTER_OBJECT,
    WaitContextBlock: PWAIT_CONTEXT_BLOCK,
    NumberOfMapRegisters: ULONG,
    ExecutionRoutine: PDRIVER_CONTROL,
) -> NTSTATUS {
    unsafe {
        // ASSERT(KeGetCurrentIrql() == DISPATCH_LEVEL) — skip.
        (*WaitContextBlock).DeviceRoutine = ExecutionRoutine;
        (*WaitContextBlock).NumberOfMapRegisters = NumberOfMapRegisters;

        // KeInsertDeviceQueue returns TRUE if queued (busy), FALSE if marked busy.
        if KeInsertDeviceQueue(
            &raw mut (*AdapterObject).ChannelWaitQueue as *mut _,
            &raw mut (*WaitContextBlock).WaitQueueEntry as *mut _,
        ) != 0
        {
            return STATUS_SUCCESS;
        }

        let master = (*AdapterObject).MasterAdapter;
        (*AdapterObject).NumberOfMapRegisters = NumberOfMapRegisters;
        (*AdapterObject).CurrentWcb = WaitContextBlock;

        let mut index: ULONG = MAXULONG;
        if NumberOfMapRegisters != 0 && (*AdapterObject).NeedsMapRegisters != 0 {
            if NumberOfMapRegisters > (*AdapterObject).MapRegistersPerChannel {
                (*AdapterObject).NumberOfMapRegisters = 0;
                IoFreeAdapterChannel(AdapterObject);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            let mut old_irql: KIRQL = 0;
            ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);

            if is_list_empty(&raw const (*master).AdapterQueue) {
                index =
                    RtlFindClearBitsAndSet((*master).MapRegisters as *mut _, NumberOfMapRegisters, 0);
                if index != MAXULONG {
                    let mut base = (*master).MapRegisterBase.add(index as usize);
                    if (*AdapterObject).ScatterGather == 0 {
                        base = ((base as ULONG_PTR) | MAP_BASE_SW_SG as ULONG_PTR)
                            as PROS_MAP_REGISTER_ENTRY;
                    }
                    (*AdapterObject).MapRegisterBase = base;
                }
            }

            if index == MAXULONG {
                insert_tail_list(
                    &raw mut (*master).AdapterQueue,
                    &raw mut (*AdapterObject).AdapterQueue,
                );

                let work_item = ExAllocatePoolWithTag(
                    _POOL_TYPE_NonPagedPool as _,
                    core::mem::size_of::<_GROW_WORK_ITEM_LOCAL>() as SIZE_T,
                    TAG_DMA,
                ) as *mut _GROW_WORK_ITEM_LOCAL;
                if !work_item.is_null() {
                    ex_initialize_work_item(
                        &raw mut (*work_item).work_queue_item,
                        HalpGrowMapBufferWorker,
                        work_item as PVOID,
                    );
                    (*work_item).adapter_object = AdapterObject;
                    (*work_item).number_of_map_registers = NumberOfMapRegisters;
                    ExQueueWorkItem(
                        &raw mut (*work_item).work_queue_item as *mut _,
                        _WORK_QUEUE_TYPE_DelayedWorkQueue as _,
                    );
                }

                ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
                return STATUS_SUCCESS;
            }

            ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
        } else {
            (*AdapterObject).MapRegisterBase = ptr::null_mut();
            (*AdapterObject).NumberOfMapRegisters = 0;
        }

        (*AdapterObject).CurrentWcb = WaitContextBlock;

        if let Some(routine) = ExecutionRoutine {
            let result = routine(
                (*WaitContextBlock).DeviceObject as PDEVICE_OBJECT,
                (*WaitContextBlock).CurrentIrp as *mut _,
                (*AdapterObject).MapRegisterBase as PVOID,
                (*WaitContextBlock).DeviceContext,
            );
            if result == _IO_ALLOCATION_ACTION_DeallocateObject as u32 {
                IoFreeAdapterChannel(AdapterObject);
            } else if result == _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as u32 {
                (*AdapterObject).NumberOfMapRegisters = 0;
                IoFreeAdapterChannel(AdapterObject);
            }
        }

        STATUS_SUCCESS
    }
}

// ---------------------------------------------------------------------------
// IoFreeAdapterChannel
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFreeAdapterChannel(mut AdapterObject: PADAPTER_OBJECT) {
    unsafe {
        let master = (*AdapterObject).MasterAdapter;
        let mut old_irql: KIRQL = 0;

        loop {
            if (*AdapterObject).NumberOfMapRegisters != 0 {
                IoFreeMapRegisters(
                    AdapterObject,
                    (*AdapterObject).MapRegisterBase as PVOID,
                    (*AdapterObject).NumberOfMapRegisters,
                );
            }

            let queue_entry = KeRemoveDeviceQueue(&raw mut (*AdapterObject).ChannelWaitQueue as *mut _);
            if queue_entry.is_null() {
                break;
            }

            // CONTAINING_RECORD(queue_entry, WAIT_CONTEXT_BLOCK, WaitQueueEntry)
            let off = offset_of!(WAIT_CONTEXT_BLOCK, WaitQueueEntry) as ULONG_PTR;
            let wcb = ((queue_entry as ULONG_PTR) - off) as PWAIT_CONTEXT_BLOCK;

            (*AdapterObject).CurrentWcb = wcb;
            (*AdapterObject).NumberOfMapRegisters = (*wcb).NumberOfMapRegisters;

            let mut index: ULONG = MAXULONG;
            if (*wcb).NumberOfMapRegisters != 0 && !(*AdapterObject).MasterAdapter.is_null() {
                ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);

                if is_list_empty(&raw const (*master).AdapterQueue) {
                    index = RtlFindClearBitsAndSet(
                        (*master).MapRegisters as *mut _,
                        (*wcb).NumberOfMapRegisters,
                        0,
                    );
                    if index != MAXULONG {
                        let mut base = (*master).MapRegisterBase.add(index as usize);
                        if (*AdapterObject).ScatterGather == 0 {
                            base = ((base as ULONG_PTR) | MAP_BASE_SW_SG as ULONG_PTR)
                                as PROS_MAP_REGISTER_ENTRY;
                        }
                        (*AdapterObject).MapRegisterBase = base;
                    }
                }

                if index == MAXULONG {
                    insert_tail_list(
                        &raw mut (*master).AdapterQueue,
                        &raw mut (*AdapterObject).AdapterQueue,
                    );
                    ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
                    break;
                }
                ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
            } else {
                (*AdapterObject).MapRegisterBase = ptr::null_mut();
                (*AdapterObject).NumberOfMapRegisters = 0;
            }

            let device_routine: PDRIVER_CONTROL =
                core::mem::transmute((*wcb).DeviceRoutine);
            if let Some(routine) = device_routine {
                let result = routine(
                    (*wcb).DeviceObject as PDEVICE_OBJECT,
                    (*wcb).CurrentIrp as *mut _,
                    (*AdapterObject).MapRegisterBase as PVOID,
                    (*wcb).DeviceContext,
                );
                if result == _IO_ALLOCATION_ACTION_KeepObject as u32 {
                    return;
                } else if result == _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as u32 {
                    (*AdapterObject).NumberOfMapRegisters = 0;
                }
            }
        }
        let _ = AdapterObject;
    }
}

// ---------------------------------------------------------------------------
// IoFreeMapRegisters
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFreeMapRegisters(
    AdapterObject: PADAPTER_OBJECT,
    MapRegisterBase: PVOID,
    NumberOfMapRegisters: ULONG,
) {
    unsafe {
        let master = (*AdapterObject).MasterAdapter;
        if master.is_null() || MapRegisterBase.is_null() {
            return;
        }

        let mut old_irql: KIRQL = 0;
        ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);

        if NumberOfMapRegisters != 0 {
            let real_base = ((MapRegisterBase as ULONG_PTR) & !(MAP_BASE_SW_SG as ULONG_PTR))
                as PROS_MAP_REGISTER_ENTRY;
            let idx = real_base.offset_from((*master).MapRegisterBase) as ULONG;
            RtlClearBits((*master).MapRegisters as *mut _, idx, NumberOfMapRegisters);
        }

        // Walk waiting adapters, satisfy as many as possible.
        while !is_list_empty(&raw const (*master).AdapterQueue) {
            let entry = remove_head_list(&raw mut (*master).AdapterQueue);
            let off = offset_of!(_ADAPTER_OBJECT, AdapterQueue) as ULONG_PTR;
            let next_adapter = ((entry as ULONG_PTR) - off) as PADAPTER_OBJECT;

            let index = RtlFindClearBitsAndSet(
                (*master).MapRegisters as *mut _,
                (*next_adapter).NumberOfMapRegisters,
                0,
            );
            if index == MAXULONG {
                insert_head_list(&raw mut (*master).AdapterQueue, entry);
                break;
            }

            ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);

            let mut base = (*master).MapRegisterBase.add(index as usize);
            if (*next_adapter).ScatterGather == 0 {
                base = ((base as ULONG_PTR) | MAP_BASE_SW_SG as ULONG_PTR)
                    as PROS_MAP_REGISTER_ENTRY;
            }
            (*next_adapter).MapRegisterBase = base;

            let wcb = (*next_adapter).CurrentWcb;
            let device_routine: PDRIVER_CONTROL =
                core::mem::transmute((*wcb).DeviceRoutine);
            if let Some(routine) = device_routine {
                let result = routine(
                    (*wcb).DeviceObject as PDEVICE_OBJECT,
                    (*wcb).CurrentIrp as *mut _,
                    (*next_adapter).MapRegisterBase as PVOID,
                    (*wcb).DeviceContext,
                );
                if result == _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as u32 {
                    (*next_adapter).NumberOfMapRegisters = 0;
                }
                if result == _IO_ALLOCATION_ACTION_DeallocateObject as u32
                    || result == _IO_ALLOCATION_ACTION_DeallocateObjectKeepRegisters as u32
                {
                    if (*next_adapter).NumberOfMapRegisters != 0 {
                        ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);
                        let idx = ((*next_adapter).MapRegisterBase as ULONG_PTR
                            - (*master).MapRegisterBase as ULONG_PTR)
                            as ULONG
                            / core::mem::size_of::<ROS_MAP_REGISTER_ENTRY>() as ULONG;
                        RtlClearBits(
                            (*master).MapRegisters as *mut _,
                            idx,
                            (*next_adapter).NumberOfMapRegisters,
                        );
                        ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
                    }
                    IoFreeAdapterChannel(next_adapter);
                }
            }

            ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);
        }

        ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
    }
}

// ---------------------------------------------------------------------------
// HalpCopyBufferMap — copy data between MDL and map-register buffers.
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpCopyBufferMap(
    Mdl: PMDL,
    mut MapRegisterBase: PROS_MAP_REGISTER_ENTRY,
    CurrentVa: PVOID,
    mut Length: ULONG,
    WriteToDevice: BOOLEAN,
) {
    unsafe {
        let virtual_address =
            mm_get_system_address_for_mdl_safe(Mdl, _MM_PAGE_PRIORITY_HighPagePriority);
        if virtual_address.is_null() {
            KeBugCheckEx(HAL_MEMORY_ALLOCATION, PAGE_SIZE as ULONG_PTR, 0, 0, 0);
        }

        let mdl_va = mm_get_mdl_virtual_address(Mdl) as ULONG_PTR;
        let mut current_address =
            virtual_address as ULONG_PTR + CurrentVa as ULONG_PTR - mdl_va;

        while Length > 0 {
            let byte_off = byte_offset(current_address);
            let mut current_length = PAGE_SIZE - byte_off;
            if current_length > Length {
                current_length = Length;
            }

            let map_va =
                (*MapRegisterBase).VirtualAddress as ULONG_PTR + byte_off as ULONG_PTR;
            if WriteToDevice != 0 {
                ptr::copy_nonoverlapping(
                    current_address as *const u8,
                    map_va as *mut u8,
                    current_length as usize,
                );
            } else {
                ptr::copy_nonoverlapping(
                    map_va as *const u8,
                    current_address as *mut u8,
                    current_length as usize,
                );
            }

            Length -= current_length;
            current_address += current_length as ULONG_PTR;
            MapRegisterBase = MapRegisterBase.add(1);
        }
    }
}

// ---------------------------------------------------------------------------
// IoFlushAdapterBuffers
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoFlushAdapterBuffers(
    AdapterObject: PADAPTER_OBJECT,
    Mdl: PMDL,
    MapRegisterBase: PVOID,
    CurrentVa: PVOID,
    mut Length: ULONG,
    WriteToDevice: BOOLEAN,
) -> BOOLEAN {
    unsafe {
        let mut slave_dma = false;

        if (*AdapterObject).MasterDevice == 0 {
            // Mask out (disable) the DMA channel.
            if (*AdapterObject).AdapterNumber == 1 {
                let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA1_CONTROL;
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_SETMASK as UCHAR,
                );
            } else {
                let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA2_CONTROL;
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_SETMASK as UCHAR,
                );
            }
            slave_dma = true;
        }

        if MapRegisterBase.is_null() {
            return 1;
        }

        let real_base = ((MapRegisterBase as ULONG_PTR) & !(MAP_BASE_SW_SG as ULONG_PTR))
            as PROS_MAP_REGISTER_ENTRY;

        if WriteToDevice == 0 {
            if (MapRegisterBase as ULONG_PTR) & (MAP_BASE_SW_SG as ULONG_PTR) != 0 {
                if (*real_base).Counter != MAXULONG {
                    if slave_dma && (*AdapterObject).IgnoreCount == 0 {
                        Length -= HalReadDmaCounter(AdapterObject);
                    }
                }
                HalpCopyBufferMap(Mdl, real_base, CurrentVa, Length, 0);
            } else {
                let mut pages = mm_get_mdl_pfn_array(Mdl);
                pages =
                    pages.add(((CurrentVa as ULONG_PTR - (*Mdl).StartVa as ULONG_PTR) >> PAGE_SHIFT) as usize);

                let mut phys: PHYSICAL_ADDRESS = core::mem::zeroed();
                phys.QuadPart = ((*pages as i64) << PAGE_SHIFT) + byte_offset(CurrentVa as ULONG_PTR) as i64;

                let high = HalpGetAdapterMaximumPhysicalAddress(AdapterObject);
                if phys.QuadPart + Length as i64 > high.QuadPart {
                    HalpCopyBufferMap(Mdl, real_base, CurrentVa, Length, 0);
                }
            }
        }

        (*real_base).Counter = 0;
        1
    }
}

// ---------------------------------------------------------------------------
// IoMapTransfer
// ---------------------------------------------------------------------------

// dma.c: inside `#ifndef _MINIHAL_` (lines 1376-2281) — absent under -D_MINIHAL_.
#[cfg(not(feature = "minihal"))]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn IoMapTransfer(
    AdapterObject: PADAPTER_OBJECT,
    Mdl: PMDL,
    MapRegisterBase: PVOID,
    CurrentVa: PVOID,
    Length: *mut ULONG,
    WriteToDevice: BOOLEAN,
) -> PHYSICAL_ADDRESS {
    unsafe {
        let byte_off = byte_offset(CurrentVa as ULONG_PTR);
        let mut pages = mm_get_mdl_pfn_array(Mdl);
        pages = pages.add(((CurrentVa as ULONG_PTR - (*Mdl).StartVa as ULONG_PTR) >> PAGE_SHIFT) as usize);

        let mut phys: PHYSICAL_ADDRESS = core::mem::zeroed();
        phys.QuadPart = ((*pages as i64) << PAGE_SHIFT) + byte_off as i64;
        let mut transfer_length = PAGE_SIZE - byte_off;

        // Bus-master path: walk physically-contiguous pages from MDL.
        if MapRegisterBase.is_null() {
            while transfer_length < *Length {
                let p1 = *pages;
                let p2 = *(pages.add(1));
                if p1 + 1 != p2 {
                    break;
                }
                if ((p1 ^ p2) as ULONG_PTR) & !0xFFFFFu32 as ULONG_PTR != 0 {
                    break;
                }
                transfer_length += PAGE_SIZE;
                pages = pages.add(1);
            }
            if transfer_length < *Length {
                *Length = transfer_length;
            }
            return phys;
        }

        // Slave + bus-master without S/G — copyover via map registers if needed.
        let real_base = ((MapRegisterBase as ULONG_PTR) & !(MAP_BASE_SW_SG as ULONG_PTR))
            as PROS_MAP_REGISTER_ENTRY;

        while transfer_length < *Length {
            let p1 = *pages;
            let p2 = *(pages.add(1));
            if p1 + 1 != p2 {
                break;
            }
            if HalpEisaDma == 0 && ((p1 ^ p2) as ULONG_PTR) & !0xFu32 as ULONG_PTR != 0 {
                break;
            }
            transfer_length += PAGE_SIZE;
            pages = pages.add(1);
        }
        if transfer_length > *Length {
            transfer_length = *Length;
        }

        let mut use_map_registers;
        let mut counter: ULONG;
        if (MapRegisterBase as ULONG_PTR) & (MAP_BASE_SW_SG as ULONG_PTR) != 0
            && transfer_length < *Length
        {
            use_map_registers = true;
            phys = (*real_base).PhysicalAddress;
            phys.QuadPart += byte_off as i64;
            transfer_length = *Length;
            (*real_base).Counter = MAXULONG;
            counter = 0;
        } else {
            use_map_registers = false;
            counter = (*real_base).Counter;
            (*real_base).Counter += bytes_to_pages(byte_off + transfer_length);

            let high = HalpGetAdapterMaximumPhysicalAddress(AdapterObject);
            if phys.QuadPart + transfer_length as i64 > high.QuadPart {
                use_map_registers = true;
                phys = (*real_base.add(counter as usize)).PhysicalAddress;
                phys.QuadPart += byte_off as i64;
                if (MapRegisterBase as ULONG_PTR) & (MAP_BASE_SW_SG as ULONG_PTR) != 0 {
                    (*real_base).Counter = MAXULONG;
                    counter = 0;
                }
            }
        }

        if use_map_registers && WriteToDevice != 0 {
            HalpCopyBufferMap(
                Mdl,
                real_base.add(counter as usize),
                CurrentVa,
                transfer_length,
                WriteToDevice,
            );
        }

        *Length = transfer_length;

        // Slave DMA: program the controller.
        if !AdapterObject.is_null() && (*AdapterObject).MasterDevice == 0 {
            let mut adapter_mode = (*AdapterObject).AdapterMode;

            if WriteToDevice != 0 {
                adapter_mode.__bindgen_anon_1.set_TransferType(WRITE_TRANSFER as UCHAR);
            } else {
                adapter_mode.__bindgen_anon_1.set_TransferType(READ_TRANSFER as UCHAR);
                if (*AdapterObject).IgnoreCount != 0 {
                    let dst = (*real_base.add(counter as usize)).VirtualAddress as ULONG_PTR
                        + byte_off as ULONG_PTR;
                    ptr::write_bytes(dst as *mut u8, 0, transfer_length as usize);
                }
            }

            let mut transfer_offset = (phys.QuadPart as ULONG) & 0xFFFF;
            let mut transfer_length_local = transfer_length;
            if (*AdapterObject).Width16Bits != 0 {
                transfer_length_local >>= 1;
                transfer_offset >>= 1;
            }

            let master = (*AdapterObject).MasterAdapter;
            let mut old_irql: KIRQL = 0;
            ke_acquire_spinlock(&raw mut (*master).SpinLock, &mut old_irql);

            let channel = (*AdapterObject).ChannelNumber as usize;
            let page_byte = ((phys.QuadPart as ULONG) >> 16) as UCHAR;

            if (*AdapterObject).AdapterNumber == 1 {
                let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA1_CONTROL;
                write_port_uchar(&raw mut (*ctrl).ClearBytePointer, 0);
                write_port_uchar(&raw mut (*ctrl).Mode, adapter_mode.Byte);
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseAddress,
                    transfer_offset as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseAddress,
                    (transfer_offset >> 8) as UCHAR,
                );
                write_port_uchar(
                    (*AdapterObject)
                        .PagePort
                        .add(offset_of!(_EISA_CONTROL, DmaController1Pages)),
                    page_byte,
                );
                if HalpEisaDma != 0 {
                    write_port_uchar(
                        (*AdapterObject)
                            .PagePort
                            .add(offset_of!(_EISA_CONTROL, DmaController2Pages)),
                        0,
                    );
                }
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseCount,
                    (transfer_length_local - 1) as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseCount,
                    ((transfer_length_local - 1) >> 8) as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_CLEARMASK as UCHAR,
                );
            } else {
                let ctrl = (*AdapterObject).AdapterBaseVa as *mut _DMA2_CONTROL;
                write_port_uchar(&raw mut (*ctrl).ClearBytePointer, 0);
                write_port_uchar(&raw mut (*ctrl).Mode, adapter_mode.Byte);
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseAddress,
                    transfer_offset as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseAddress,
                    (transfer_offset >> 8) as UCHAR,
                );
                write_port_uchar(
                    (*AdapterObject)
                        .PagePort
                        .add(offset_of!(_EISA_CONTROL, DmaController1Pages)),
                    page_byte,
                );
                if HalpEisaDma != 0 {
                    write_port_uchar(
                        (*AdapterObject)
                            .PagePort
                            .add(offset_of!(_EISA_CONTROL, DmaController2Pages)),
                        0,
                    );
                }
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseCount,
                    (transfer_length_local - 1) as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).DmaAddressCount[channel].DmaBaseCount,
                    ((transfer_length_local - 1) >> 8) as UCHAR,
                );
                write_port_uchar(
                    &raw mut (*ctrl).SingleMask,
                    (*AdapterObject).ChannelNumber | DMA_CLEARMASK as UCHAR,
                );
            }

            ke_release_spinlock(&raw mut (*master).SpinLock, old_irql);
        }

        phys
    }
}

// ---------------------------------------------------------------------------
// HalFlushCommonBuffer — always returns true.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalFlushCommonBuffer(
    _AdapterObject: PDMA_ADAPTER,
    _Length: ULONG,
    _LogicalAddress: PHYSICAL_ADDRESS,
    _VirtualAddress: PVOID,
) -> BOOLEAN {
    1
}

// ---------------------------------------------------------------------------
// HalAllocateCrashDumpRegisters
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalAllocateCrashDumpRegisters(
    AdapterObject: PADAPTER_OBJECT,
    NumberOfMapRegisters: *mut ULONG,
) -> PVOID {
    unsafe {
        let master = (*AdapterObject).MasterAdapter;
        if (*AdapterObject).NeedsMapRegisters != 0 {
            if *NumberOfMapRegisters > (*AdapterObject).MapRegistersPerChannel {
                (*AdapterObject).NumberOfMapRegisters = 0;
                return ptr::null_mut();
            }

            let mut map_register_number =
                RtlFindClearBitsAndSet((*master).MapRegisters as *mut _, *NumberOfMapRegisters, 0);
            if map_register_number == MAXULONG {
                RtlSetBits((*master).MapRegisters as *mut _, 0, *NumberOfMapRegisters);
                map_register_number = 0;
            }

            let mut base = (*master).MapRegisterBase.add(map_register_number as usize);
            if (*AdapterObject).ScatterGather == 0 {
                base = ((base as ULONG_PTR) | MAP_BASE_SW_SG as ULONG_PTR)
                    as PROS_MAP_REGISTER_ENTRY;
            }
            (*AdapterObject).MapRegisterBase = base;
        } else {
            (*AdapterObject).MapRegisterBase = ptr::null_mut();
            (*AdapterObject).NumberOfMapRegisters = 0;
        }
        (*AdapterObject).MapRegisterBase as PVOID
    }
}
