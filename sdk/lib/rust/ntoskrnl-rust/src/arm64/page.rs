use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _MMPFN,
    PEPROCESS,
    PMMPFN as DDK_PMMPFN,
    PPFN_NUMBER,
    PULONG_PTR,
    SWAPENTRY,
    KPROCESSOR_MODE,
    KeBugCheck,
    KeBugCheckEx,
    KeInvalidateTlbEntry,
    MiDecrementPageTableReferences,
    MiDeletePde,
    MiIncrementPageTableReferences,
    MiIsPdeForAddressValid,
    MiLockProcessWorkingSetShared,
    MiLockProcessWorkingSetUnsafe,
    MiMakePdeExistAndMakeValid,
    MiMakeProtectionMask,
    MiRemoveZeroPageSafe,
    MiUnlockProcessWorkingSetShared,
    MiUnlockProcessWorkingSetUnsafe,
    MmIsPageInUse,
    PsGetCurrentThread,
};

use crate::ARM3::helpers::*;

extern "C" {
    static mut Arm64UartKernelBase: PVOID;
}

unsafe fn PagePutChar(c: u8) {
    let pl011_base = Arm64UartKernelBase as *mut ULONG;
    crate::arm64_dbg::uart_putc_default(c as u8);
}

unsafe fn PagePuts(s: &[u8]) {
    for &c in s {
        if c == 0 { break; }
        PagePutChar(c);
    }
}

unsafe fn PagePutHex64(val: u64) {
    PagePuts(b"0x");
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        PagePutChar(b"0123456789ABCDEF"[nibble]);
    }
}

#[cfg(feature = "arm64_uart_trace")]
macro_rules! PAGE_TRACE {
    ($s:expr) => { unsafe { PagePuts($s) } };
}
#[cfg(feature = "arm64_uart_trace")]
macro_rules! PAGE_TRACE_HEX {
    ($v:expr) => { unsafe { PagePutHex64($v) } };
}
#[cfg(not(feature = "arm64_uart_trace"))]
macro_rules! PAGE_TRACE {
    ($s:expr) => { () };
}
#[cfg(not(feature = "arm64_uart_trace"))]
macro_rules! PAGE_TRACE_HEX {
    ($v:expr) => { () };
}

/// Helper: cast hand-written MMPTE pointer to DDK _MMPTE pointer for DDK function calls.
#[inline(always)]
unsafe fn as_ddk_pde(p: *mut MMPTE) -> *mut ntkrnl_sys::ddk::_MMPTE {
    p as *mut ntkrnl_sys::ddk::_MMPTE
}

// PXE_SELFMAP: the self-referencing PXE entry address
// On ARM64 the PXE self-map entry is at the very last PXE slot (same as PXE_BASE + 0xFF8)
// PXE_BASE is 0xFFFFF6FB7DBED000 and the self-map is at PXE_BASE + 255*8 = PXE_BASE + 0x7F8
// but actually PXE_SELFMAP = PXE_BASE + PXE_SELFMAP_IDX * 8.  Matching the C code:
//   #define PXE_SELFMAP (PXE_BASE + MiAddressToPxeOffset(PXE_BASE) * sizeof(MMPTE))
// MiAddressToPxeOffset(PXE_BASE) = (PXE_BASE >> 39) & 0x1FF = 0x1ED
// So PXE_SELFMAP = PXE_BASE + 0x1ED * 8 = PXE_BASE + 0xF68
const PXE_SELFMAP: usize = PXE_BASE + ((PXE_BASE >> 39) & 0x1FF) * 8;

/// MiAddressToPxeOffset — (VA >> 39) & 0x1FF
#[inline]
fn MiAddressToPxeOffset(va: PVOID) -> ULONG {
    (((va as ULONG_PTR) >> 39) & 0x1FF) as ULONG
}

/// MiAddressToPteOffset — (VA >> 12) & 0x1FF
#[inline]
fn MiAddressToPteOffset(va: PVOID) -> ULONG {
    (((va as ULONG_PTR) >> 12) & 0x1FF) as ULONG
}

unsafe fn MiEnsureKernelPdeValid(Address: PVOID) {
    let PointerPxe = MiAddressToPxe(Address);
    let PointerPpe = MiAddressToPpe(Address);
    let PointerPde = MiAddressToPde(Address);
    let mut PageFrameIndex: PFN_NUMBER;
    let mut TempPte = MMPTE { Long: 0 };
    let mut OldIrql: KIRQL;

    if !(*PointerPxe).valid() {
        PagePuts(b"[PAGE] BUG: PXE invalid, cannot create\r\n");
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    if !(*PointerPpe).valid() {
        // [guard] OldIrql = MiAcquirePfnLock();
        let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        drop(pfn_guard);

        TempPte.Long = ValidKernelPde.Long;
        TempPte.set_page_frame_number(PageFrameIndex);
        MI_WRITE_VALID_PTE(PointerPpe, TempPte);
        core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
        core::ptr::write_bytes(MiPteToAddress(PointerPpe) as *mut u8, 0, PAGE_SIZE);
    }

    if !(*PointerPde).valid() {
        // [guard] OldIrql = MiAcquirePfnLock();
        let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_COLOR());
        drop(pfn_guard);

        TempPte.Long = ValidKernelPde.Long;
        TempPte.set_page_frame_number(PageFrameIndex);
        MI_WRITE_VALID_PTE(PointerPde, TempPte);
        core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));
        core::ptr::write_bytes(MiPteToAddress(PointerPde) as *mut u8, 0, PAGE_SIZE);
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmIsPagePresent(
    Process: PEPROCESS,
    Address: PVOID,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    if Address >= MmSystemRangeStart {
        if MiIsPdeForAddressValid(Address) == 0 {
            return 0u8;
        }
        return (*MiAddressToPte(Address)).valid() as BOOLEAN;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if !(*MiAddressToPxe(Address)).valid()
        || !(*MiAddressToPpe(Address)).valid()
        || !(*MiAddressToPde(Address)).valid()
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0u8;
    }

    let Ret = (*MiAddressToPte(Address)).valid() as BOOLEAN;
    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmIsDisabledPage(
    Process: PEPROCESS,
    Address: PVOID,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Ret: BOOLEAN;

    if Address >= MmSystemRangeStart {
        if MiIsPdeForAddressValid(Address) == 0 {
            return 0u8;
        }
    } else {
        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if !(*MiAddressToPxe(Address)).valid()
            || !(*MiAddressToPpe(Address)).valid()
            || !(*MiAddressToPde(Address)).valid()
        {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return 0u8;
        }
    }

    PointerPte = MiAddressToPte(Address);

    Ret = (!(*PointerPte).valid()
        && ((*PointerPte).Long & 0x800) == 0
        && (*PointerPte).page_frame_number() != 0) as BOOLEAN;

    if Address < MmSystemRangeStart {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmIsPageSwapEntry(
    Process: PEPROCESS,
    Address: PVOID,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Ret: BOOLEAN;

    if Address >= MmSystemRangeStart {
        return 0u8;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if !(*MiAddressToPxe(Address)).valid()
        || !(*MiAddressToPpe(Address)).valid()
        || !(*MiAddressToPde(Address)).valid()
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0u8;
    }

    PointerPte = MiAddressToPte(Address);
    Ret = (!(*PointerPte).valid() && ((*PointerPte).Long & 0x800) != 0) as BOOLEAN;

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPfnForProcess(
    Process: PEPROCESS,
    Address: PVOID,
) -> PFN_NUMBER {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Page: PFN_NUMBER;

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if !(*MiAddressToPxe(Address)).valid()
        || !(*MiAddressToPpe(Address)).valid()
        || !(*MiAddressToPde(Address)).valid()
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        return 0;
    }

    PointerPte = MiAddressToPte(Address);
    Page = if (*PointerPte).valid() {
        PFN_FROM_PTE(PointerPte)
    } else {
        0
    };

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    Page
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPageProtect(
    Process: PEPROCESS,
    Address: PVOID,
) -> ULONG {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let mut Protect: ULONG;

    if Address >= MmSystemRangeStart {
        if MiIsPdeForAddressValid(Address) == 0 {
            return PAGE_NOACCESS;
        }
    } else {
        MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

        if !(*MiAddressToPxe(Address)).valid()
            || !(*MiAddressToPpe(Address)).valid()
            || !(*MiAddressToPde(Address)).valid()
        {
            MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
            return PAGE_NOACCESS;
        }
    }

    PointerPte = MiAddressToPte(Address);
    let pte = &*PointerPte;

    if !pte.valid() {
        Protect = PAGE_NOACCESS;
    } else {
        // ARM64 hardware PTE bits:
        // CopyOnWrite = bit 56, Writable = NOT bit 54 (AP[2]==0 means writable)
        // UserNoExecute = bit 54
        // CacheType = bits [3:2]
        // We use the raw Long field for these ARM64-specific bits.
        let copy_on_write = (pte.Long >> 56) & 1;
        let writable = (pte.Long >> 55) & 1; // bit 55 = Writable in ARM64 _HARDWARE_PTE
        let user_no_execute = (pte.Long >> 54) & 1;
        let cache_type = (pte.Long >> 2) & 0x3;

        if copy_on_write != 0 {
            Protect = PAGE_WRITECOPY;
        } else if writable != 0 {
            Protect = PAGE_READWRITE;
        } else {
            Protect = PAGE_READONLY;
        }

        let mut final_protect = Protect;
        if user_no_execute == 0 {
            final_protect <<= 4;
        }

        // CacheType = AttrIndx into MAIR=0x004400FF: 0=Normal-WB (cached),
        // 1=Device (no-cache), 2=Normal-NC (write-combine). This was previously
        // `== 0` (the inverted scheme that mapped cached pages as Device); now
        // consistent with the corrected MmProtectToPteMask cache bits.
        if cache_type == 1 {
            final_protect |= PAGE_NOCACHE;
        } else if cache_type == 2 {
            final_protect |= PAGE_WRITECOMBINE;
        }

        Protect = final_protect;
    }

    if Address < MmSystemRangeStart {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
    }

    Protect
}

#[no_mangle]
pub unsafe extern "system" fn MmSetPageProtect(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let ProtectionMask: ULONG;
    let PointerPte: PMMPTE;
    let mut TempPte = MMPTE { Long: 0 };
    let mut OldPte = MMPTE { Long: 0 };

    ProtectionMask = MiMakeProtectionMask(flProtect);

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    PointerPte = MiAddressToPte(Address);

    TempPte.Long = 0;
    TempPte.set_page_frame_number((*PointerPte).page_frame_number());
    TempPte.Long |= MmProtectToPteMask[ProtectionMask as usize];
    // Owner bit (bit 6 on ARM64 = user accessible)
    TempPte.Long |= 1 << 6;

    if (ProtectionMask != MM_NOACCESS) && ((ProtectionMask & MM_GUARDPAGE) == 0) {
        TempPte.set_valid(true);
    }

    // Copy Accessed bit from old PTE
    TempPte.Long |= (*PointerPte).Long & PTE_ACCESSED;

    OldPte.Long = InterlockedExchangePte(PointerPte, TempPte.Long);

    if OldPte.Long != TempPte.Long {
        KeInvalidateTlbEntry(Address);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

#[no_mangle]
pub unsafe extern "system" fn MmSetDirtyBit(
    Process: PEPROCESS,
    Address: PVOID,
    Dirty: BOOLEAN,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    PointerPte = MiAddressToPte(Address);

    // ARM64: NotDirty is bit 7 (DBM bit inverted sense)
    if Dirty != 0 {
        (*PointerPte).Long &= !(1u64 << 7) as ULONG_PTR; // NotDirty = 0
    } else {
        (*PointerPte).Long |= (1u64 << 7) as ULONG_PTR;  // NotDirty = 1
    }

    if Dirty == 0 {
        KeInvalidateTlbEntry(Address);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
}

unsafe fn MmCreateVirtualMappingUnsafeEx(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
    IsPhysical: BOOLEAN,
) -> NTSTATUS {
    let ProtectionMask: ULONG;
    let PointerPte: PMMPTE;
    let mut TempPte = MMPTE { Long: 0 };
    let Pte: ULONG_PTR;

    PAGE_TRACE!(b"[PAGE] CreateVMap addr=");
    PAGE_TRACE_HEX!(Address as u64);
    PAGE_TRACE!(b" pfn=");
    PAGE_TRACE_HEX!(Page as u64);
    PAGE_TRACE!(b"\r\n");

    ProtectionMask = MiMakeProtectionMask(flProtect);
    PAGE_TRACE!(b"[PAGE] ProtMask=");
    PAGE_TRACE_HEX!(ProtectionMask as u64);
    PAGE_TRACE!(b"\r\n");

    if Process.is_null() {
        if Address < MmSystemRangeStart {
            PagePuts(b"[PAGE] BUG: kernel addr < MmSystemRangeStart\r\n");
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        if MiIsPdeForAddressValid(Address) == 0 {
            MiEnsureKernelPdeValid(Address);
        }
    } else {
        if Address >= MmSystemRangeStart || (Address as usize + PAGE_SIZE) >= MmSystemRangeStart as usize {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
    }

    PointerPte = MiAddressToPte(Address);

    MI_MAKE_HARDWARE_PTE(&mut TempPte, PointerPte, ProtectionMask as ULONG_PTR, Page);

    // bits[1:0]=0b01 is a reserved-invalid L3 descriptor on ARM64: the CPU
    // translation-faults on it while MMPTE.valid() (bit0) reports present, so
    // the fault path spins forever instead of bugchecking (§9.96). The only
    // way to compose it here is MmProtectToPteMask[mask]==0 — a NOACCESS-family
    // protection that should never reach a hardware-valid mapping.
    if TempPte.Long & 3 == 1 {
        PagePuts(b"[PAGE] BUG: malformed L3 descriptor (bits[1:0]=0b01) - NOACCESS protection reached MmCreateVirtualMapping va=");
        PagePutHex64(Address as u64);
        PagePuts(b" flProtect=");
        PagePutHex64(flProtect as u64);
        PagePuts(b" mask=");
        PagePutHex64(ProtectionMask as u64);
        PagePuts(b" pfn=");
        PagePutHex64(Page as u64);
        PagePuts(b"\r\n");
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    Pte = InterlockedExchangePte(PointerPte, TempPte.Long);

    crate::ke::flightrec::flightrec_log(
        crate::ke::flightrec::OP_PAGE_MAP,
        Address as usize & !(PAGE_SIZE - 1),
        PsGetCurrentThread() as usize,
        Page as u32,
    );

    if Pte != 0 {
        // A double-map. The bare "old PTE was non-zero!" this used to print names
        // no VA, no page and no owner, so 20-of-85 arm64 sweep rounds died here
        // with nothing to chase. The discriminator that costs nothing is the two
        // PFNs: SAME physical page means an idempotent re-map (someone mapped a VA
        // twice without unmapping), DIFFERENT means a genuine alias — one VA
        // pointing at two pages, i.e. a leaked mapping whose page went back to the
        // free list and got handed out again. Those are different bugs.
        PagePuts(b"[PAGE] BUG: old PTE was non-zero! va=");
        PagePutHex64(Address as u64);
        PagePuts(b" oldpte=");
        PagePutHex64(Pte as u64);
        PagePuts(b" newpte=");
        PagePutHex64(TempPte.Long as u64);
        PagePuts(b"\r\n[PAGE]   oldpfn=");
        PagePutHex64((Pte as u64 >> 12) & 0xF_FFFF_FFFF);
        PagePuts(b" newpfn=");
        PagePutHex64(Page as u64);
        PagePuts(b" same=");
        PagePuts(if ((Pte as u64 >> 12) & 0xF_FFFF_FFFF) == Page as u64 { b"YES" } else { b"NO " });
        PagePuts(b" proc=");
        PagePutHex64(Process as u64);
        PagePuts(b" thr=");
        PagePutHex64(PsGetCurrentThread() as u64);
        PagePuts(b"\r\n");
        // This VA's own map/unmap history, from the flight recorder. A whole-ring
        // tail dump is useless here — on the crash that motivated this the last 32
        // events were all working-set lock churn — but filtering by VA reads out
        // exactly the previous map that was never unmapped, and the thread that
        // did it.
        crate::ke::flightrec::flightrec_dump_object(Address as usize & !(PAGE_SIZE - 1), 16);
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    core::arch::asm!("dsb ish", "isb", options(nostack, preserves_flags));

    if IsPhysical == 0 {
        let Pfn1: PMMPFN;
        let OldIrql: KIRQL;

        // [guard] OldIrql = MiAcquirePfnLock();
        let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        Pfn1 = MiGetPfnEntry(TempPte.page_frame_number());
        (*Pfn1).set_share_count((*Pfn1).share_count() + 1);
        (*Pfn1).set_page_location(ActiveAndValid);
        drop(pfn_guard);
    }

    if Address < MmSystemRangeStart {
        MiIncrementPageTableReferences(Address);
        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateVirtualMappingUnsafe(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmCreateVirtualMapping(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    if MmIsPageInUse(Page as _) == 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    MmCreateVirtualMappingUnsafe(Process, Address, flProtect, Page)
}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePhysicalMapping(
    Process: PEPROCESS,
    Address: PVOID,
    flProtect: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    MmCreateVirtualMappingUnsafeEx(Process, Address, flProtect, Page, 1u8)
}

/// Zero any leftover VALID descriptors over a user VA range that is being
/// freshly reserved.
///
/// FreeLDR's identity-mapped page-table PAGES get recycled as user-space page
/// tables while still holding FreeLDR's own entries (VA==PA, global, EL1-only),
/// so a brand-new memory area can sit on top of live-looking descriptors. Two
/// consequences, both observed on the arm64 kmtest sweep: the fault that should
/// demand-page the range never fires, and teardown decodes the stale descriptor
/// into a PFN that names no frame (VA 0x50000 -> pte 0x50703 -> PFN 0x50, far
/// below the 0x40000000 DRAM base) — which MmDeleteRmap reports as
/// mapping-not-in-list, after which MmReleasePageMemoryConsumer hands the
/// allocator a frame that was never allocated.
///
/// amd64 carries the same guard per-path in ARM3/sysldr.rs, which is why this
/// reproduced on arm64 only. Doing it once at memory-area creation covers
/// VirtualAlloc and section views together instead of path by path.
pub(crate) unsafe fn MiPurgeStaleUserPtes(StartVa: ULONG_PTR, EndVa: ULONG_PTR) {
    // ARM64 descriptor bits[1:0]: 0b11 = table/page entry, 0b01 = 2MB block.
    // HARDWARE_PTE names bit 1 NotLargePage (sdk/include/ndk/arm64/mmtypes.h);
    // there is no shared accessor for it because bit 1 is `Write` on x86.
    const NOT_LARGE_PAGE: ULONG_PTR = 1 << 1;
    const BLOCK_MASK: ULONG_PTR = 0x1F_FFFF;

    let mut need_flush = false;
    let mut Va = StartVa;

    while Va < EndVa {
        let PointerPxe = MiAddressToPxe(Va as PVOID);
        let PointerPpe = MiAddressToPpe(Va as PVOID);
        let PointerPde = MiAddressToPde(Va as PVOID);

        // Nothing is mapped this far down — skip the whole 2MB instead of
        // walking it page by page. This runs on every user reservation,
        // including multi-megabyte section views, so the common case must not
        // be O(pages).
        if !(*PointerPxe).valid() || !(*PointerPpe).valid() || !(*PointerPde).valid() {
            Va = (Va | BLOCK_MASK) + 1;
            continue;
        }

        if ((*PointerPde).Long & NOT_LARGE_PAGE) == 0 {
            // A 2MB block descriptor: drop it and skip its whole span.
            (*PointerPde).Long = 0;
            need_flush = true;
            Va = (Va | BLOCK_MASK) + 1;
            continue;
        }

        let PointerPte = MiAddressToPte(Va as PVOID);
        if (*PointerPte).valid() {
            (*PointerPte).Long = 0;
            need_flush = true;
        }
        Va += PAGE_SIZE as ULONG_PTR;
    }

    if need_flush {
        crate::ke::arm64::stubs::KeFlushEntireTb(1, 1);
    }
}

unsafe fn MmDeleteVirtualMappingEx(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
    IsPhysical: BOOLEAN,
) -> BOOLEAN {
    let PointerPte: PMMPTE;
    let mut OldPte = MMPTE { Long: 0 };
    let ValidPde: BOOLEAN;

    if Process.is_null() {
        if Address < MmSystemRangeStart {
            KeBugCheck(MEMORY_MANAGEMENT);
        }
        ValidPde = MiIsPdeForAddressValid(Address);
    } else {
        if Address >= MmSystemRangeStart || (Address as usize + PAGE_SIZE) >= MmSystemRangeStart as usize {
            KeBugCheck(MEMORY_MANAGEMENT);
        }

        MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

        ValidPde = ((*MiAddressToPxe(Address)).valid()
            && (*MiAddressToPpe(Address)).valid()
            && (*MiAddressToPde(Address)).valid()) as BOOLEAN;

        if ValidPde != 0 {
            MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);
        }
    }

    if ValidPde != 0 {
        PointerPte = MiAddressToPte(Address);
        OldPte.Long = InterlockedExchangePte(PointerPte, 0);

        // Paired with OP_PAGE_MAP above: together they make flightrec_dump_object(va)
        // a per-VA ledger, so a double-map bugcheck can say whether the previous
        // mapping was ever torn down and by whom.
        crate::ke::flightrec::flightrec_log(
            crate::ke::flightrec::OP_PAGE_UNMAP,
            Address as usize & !(PAGE_SIZE - 1),
            PsGetCurrentThread() as usize,
            OldPte.page_frame_number() as u32,
        );
        // Beside it, the raw descriptor. A PFN that names no frame is ambiguous
        // on its own — a hardware descriptor pointing somewhere wrong, or a
        // software PTE decoded with the hardware layout? `valid()` cannot settle
        // it (bit 0 only). The low 32 bits carry the type/valid bits and the
        // whole address field whenever the PFN is small — the failing case.
        crate::ke::flightrec::flightrec_log(
            crate::ke::flightrec::OP_PAGE_PTE,
            Address as usize & !(PAGE_SIZE - 1),
            PsGetCurrentThread() as usize,
            OldPte.Long as u32,
        );

        KeInvalidateTlbEntry(Address);

        if OldPte.Long != 0 {
            if !WasDirty.is_null() {
                *WasDirty = if OldPte.valid() {
                    // ARM64: NotDirty bit is bit 7 — dirty when NotDirty==0
                    ((OldPte.Long >> 7) & 1 == 0) as BOOLEAN
                } else {
                    0u8
                };
            }
            if !Page.is_null() {
                *Page = OldPte.page_frame_number() as _;
            }
        }
    }

    if !Process.is_null() {
        if OldPte.Long != 0 {
            if MiDecrementPageTableReferences(Address) == 0 {
                // [guard] let OldIrql = MiAcquirePfnLock();
                let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
                MiDeletePde(as_ddk_pde(MiAddressToPde(Address)), Process);
                drop(pfn_guard);
            }
        }

        if IsPhysical == 0 && OldPte.valid() {
            let Pfn1: PMMPFN;
            let OldIrql: KIRQL;

            // [guard] OldIrql = MiAcquirePfnLock();
            let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
            Pfn1 = MiGetPfnEntry(OldPte.page_frame_number());
            let sc = (*Pfn1).share_count();
            (*Pfn1).set_share_count(sc - 1);
            if sc - 1 == 0 {
                (*Pfn1).set_page_location(TransitionPage);
            }
            drop(pfn_guard);
        }

        MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());
    }

    if !WasDirty.is_null() && OldPte.Long == 0 {
        *WasDirty = 0u8;
    }
    if !Page.is_null() && OldPte.Long == 0 {
        *Page = 0;
    }

    (OldPte.Long != 0) as BOOLEAN
}

// NTAPI/stdcall on x86 (extern "system"); no-op CC on arm64 but kept uniform with the i386 def
// and the marea.rs VirtFn transmute-call. Do NOT flip to extern "C" (§9.99 ESP desync on i386).
#[no_mangle]
pub unsafe extern "system" fn MmDeleteVirtualMapping(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let WasDirty = opaque_ptr(WasDirty);
    MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmDeletePhysicalMapping(
    Process: PEPROCESS,
    Address: PVOID,
    WasDirty: *mut BOOLEAN,
    Page: PPFN_NUMBER,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let WasDirty = opaque_ptr(WasDirty);
    MmDeleteVirtualMappingEx(Process, Address, WasDirty, Page, 1u8)
}

#[no_mangle]
pub unsafe extern "system" fn MmGetPageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: *mut SWAPENTRY,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let SwapEntry = opaque_ptr(SwapEntry);
    let PointerPte: PMMPTE;

    if Address >= MmSystemRangeStart {
        *SwapEntry = 0;
        return;
    }

    MiLockProcessWorkingSetShared(Process, PsGetCurrentThread());

    if !(*MiAddressToPxe(Address)).valid()
        || !(*MiAddressToPpe(Address)).valid()
        || !(*MiAddressToPde(Address)).valid()
    {
        MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
        *SwapEntry = 0;
        return;
    }

    PointerPte = MiAddressToPte(Address);
    if !(*PointerPte).valid() && ((*PointerPte).Long & 0x800) != 0 {
        *SwapEntry = ((*PointerPte).Long >> 1) as SWAPENTRY;
    } else {
        *SwapEntry = 0;
    }

    MiUnlockProcessWorkingSetShared(Process, PsGetCurrentThread());
}

#[no_mangle]
pub unsafe extern "system" fn MmCreatePageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: SWAPENTRY,
) -> NTSTATUS {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let PointerPte: PMMPTE;
    let Pte: ULONG_PTR;

    if (SwapEntry as ULONG_PTR) & (1usize << (core::mem::size_of::<SWAPENTRY>() * 8 - 1)) != 0 {
        KeBugCheck(MEMORY_MANAGEMENT);
    }

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    Pte = InterlockedExchangePte(PointerPte, (SwapEntry as ULONG_PTR) << 1);
    if Pte != 0 {
        KeBugCheckEx(MEMORY_MANAGEMENT, SwapEntry as _, Process as _, Address as _, 0);
    }

    MiIncrementPageTableReferences(Address);
    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmDeletePageFileMapping(
    Process: PEPROCESS,
    Address: PVOID,
    SwapEntry: *mut SWAPENTRY,
) {
    let Process = opaque_ptr(Process);
    let Address = opaque_ptr(Address);
    let SwapEntry = opaque_ptr(SwapEntry);
    let PointerPte: PMMPTE;
    let mut OldPte = MMPTE { Long: 0 };

    MiLockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    MiMakePdeExistAndMakeValid(as_ddk_pde(MiAddressToPde(Address)), Process, MM_NOIRQL);

    PointerPte = MiAddressToPte(Address);
    OldPte.Long = InterlockedExchangePte(PointerPte, 0);

    if (OldPte.Long & 0x800) == 0 || OldPte.valid() {
        KeBugCheckEx(MEMORY_MANAGEMENT, OldPte.Long as _, Process as _, Address as _, 0);
    }

    if MiDecrementPageTableReferences(Address) == 0 {
        // [guard] let OldIrql = MiAcquirePfnLock();
        let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
        MiDeletePde(as_ddk_pde(MiPteToPde(PointerPte)), Process);
        drop(pfn_guard);
    }

    MiUnlockProcessWorkingSetUnsafe(Process, PsGetCurrentThread());

    *SwapEntry = (OldPte.Long >> 1) as SWAPENTRY;
}

#[no_mangle]
pub unsafe extern "system" fn MiArchCreateProcessAddressSpace(
    Process: PEPROCESS,
    DirectoryTableBase: PULONG_PTR,
) -> BOOLEAN {
    let Process = opaque_ptr(Process);
    let DirectoryTableBase = opaque_ptr(DirectoryTableBase);
    let mut OldIrql: KIRQL;
    let TableBasePfn: PFN_NUMBER;
    let HyperPfn: PFN_NUMBER;
    let mut HyperPdPfn: PFN_NUMBER;
    let mut HyperPtPfn: PFN_NUMBER;
    let SystemPte: PMMPTE;
    let mut TempPte = MMPTE { Long: 0 };
    let mut PdePte = MMPTE { Long: 0 };
    let mut TableIndex: ULONG;
    let PageTablePointer: PMMPTE;
    let mut PageColor: ULONG;

    // Enable IRQL transition tracing for SMSS creation debugging

    PagePuts(b"[MM] MiArchCreateProcessAddressSpace\r\n");

    TableBasePfn = (*DirectoryTableBase.add(0) >> PAGE_SHIFT) as PFN_NUMBER;
    HyperPfn = (*DirectoryTableBase.add(1) >> PAGE_SHIFT) as PFN_NUMBER;

    // [guard] OldIrql = MiAcquirePfnLock();
    let mut pfn_guard_0 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process as PVOID);
    HyperPdPfn = MiRemoveZeroPageSafe(PageColor) as PFN_NUMBER;
    if HyperPdPfn == 0 {
        HyperPdPfn = MiRemoveAnyPage(PageColor);
        drop(pfn_guard_0);
        MiZeroPhysicalPage(HyperPdPfn);
        pfn_guard_0 = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };
    }
    PageColor = MI_GET_NEXT_PROCESS_COLOR(Process as PVOID);
    HyperPtPfn = MiRemoveZeroPageSafe(PageColor) as PFN_NUMBER;
    if HyperPtPfn == 0 {
        HyperPtPfn = MiRemoveAnyPage(PageColor);
        drop(pfn_guard_0);
        MiZeroPhysicalPage(HyperPtPfn);
    } else {
        drop(pfn_guard_0);
    }

    SystemPte = MiReserveSystemPtes(1, SystemPteSpace);
    if SystemPte.is_null() {
        return 0u8;
    }

    PageTablePointer = MiPteToAddress(SystemPte) as PMMPTE;

    MI_MAKE_HARDWARE_PTE_KERNEL(&mut PdePte, SystemPte, MM_READWRITE, TableBasePfn);
    MI_WRITE_VALID_PTE(SystemPte, PdePte);

    TableIndex = PXE_PER_PAGE as ULONG / 2;
    core::ptr::write_bytes(PageTablePointer as *mut u8, 0, (TableIndex as usize) * core::mem::size_of::<MMPTE>());
    core::ptr::copy_nonoverlapping(
        (MiAddressToPxe(core::ptr::null_mut()) as *const MMPTE).add(TableIndex as usize) as *const u8,
        PageTablePointer.add(TableIndex as usize) as *mut u8,
        PAGE_SIZE - (TableIndex as usize) * core::mem::size_of::<MMPTE>(),
    );

    TempPte = ValidKernelPte;

    TableIndex = MiAddressToPxeOffset(PXE_SELFMAP as PVOID);
    TempPte.set_page_frame_number(TableBasePfn);
    (*PageTablePointer.add(TableIndex as usize)) = TempPte;

    TableIndex = MiAddressToPxeOffset(HYPER_SPACE as PVOID);
    TempPte.set_page_frame_number(HyperPfn);
    (*PageTablePointer.add(TableIndex as usize)) = TempPte;

    PdePte.set_page_frame_number(HyperPfn);
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer as PVOID);

    TempPte.set_page_frame_number(HyperPdPfn);
    (*PageTablePointer.add(0)) = TempPte;

    PdePte.set_page_frame_number(HyperPdPfn);
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer as PVOID);

    TempPte.set_page_frame_number(HyperPtPfn);
    (*PageTablePointer.add(0)) = TempPte;

    PdePte.set_page_frame_number(HyperPtPfn);
    *SystemPte = PdePte;
    KeInvalidateTlbEntry(PageTablePointer as PVOID);

    TempPte.set_page_frame_number((*(Process as *const _EPROCESS)).WorkingSetPage as PFN_NUMBER);
    TableIndex = MiAddressToPteOffset(MmWorkingSetList as PVOID);
    (*PageTablePointer.add(TableIndex as usize)) = TempPte;

    MiReleaseSystemPtes(SystemPte, 1, SystemPteSpace);

    PagePuts(b"[MM] MiArchCreateProcessAddressSpace done\r\n");

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn MmInitGlobalKernelPageDirectory() {
}
