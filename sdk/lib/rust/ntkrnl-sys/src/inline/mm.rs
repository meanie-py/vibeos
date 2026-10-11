//! MM subsystem inline functions.
//!
//! Working set locks, PTE operations, expansion lock, MDL helpers,
//! and various FORCEINLINE/macro wrappers from miarm.h, mm.h, mmfuncs.h.

use crate::base::*;
use crate::ddk;

// MDL flags (from wdm.h #defines, not in bindgen)
const MDL_MAPPED_TO_SYSTEM_VA: i16 = 0x0001;
const MDL_SOURCE_IS_NONPAGED_POOL: i16 = 0x0004;

// ============================================================================
// MiIsMemoryTypeInvisible (FORCEINLINE from mm/ARM3/miarm.h)
// ============================================================================

/// MiIsMemoryTypeInvisible — memory the loader describes but MM must not count
/// or map as RAM: firmware-reserved ranges and the like. This was once ported as
/// `>= LoaderMaximum`, which no descriptor is, so every reserved range (the PCI
/// hole below 4 GiB, the VGA hole) was counted as physical memory: the OptiPlex's
/// 2 GiB below 4 GiB read as 4095 MiB, and with RAM above 4 GiB the hole's pages
/// sat in the physical memory runs.
#[inline]
pub unsafe fn MiIsMemoryTypeInvisible(mem_type: ddk::TYPE_OF_MEMORY) -> BOOLEAN {
    (mem_type == ddk::_TYPE_OF_MEMORY_LoaderFirmwarePermanent
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderSpecialMemory
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderHALCachedMemory
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderBBTMemory) as BOOLEAN
}

// ============================================================================
// MiIsMemoryTypeFree (FORCEINLINE from mm/ARM3/miarm.h)
// ============================================================================

/// MiIsMemoryTypeFree — check if memory type is free/reusable.
#[inline]
pub unsafe fn MiIsMemoryTypeFree(mem_type: ddk::TYPE_OF_MEMORY) -> BOOLEAN {
    if mem_type == ddk::_TYPE_OF_MEMORY_LoaderFree
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderLoadedProgram
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderFirmwareTemporary
        || mem_type == ddk::_TYPE_OF_MEMORY_LoaderOsloaderStack
    {
        1
    } else {
        0
    }
}

// ============================================================================
// MmGetMdlPfnArray (macro from wdm.h)
// ============================================================================

/// MmGetMdlPfnArray — return pointer to PFN array following MDL struct.
/// C: `(PPFN_NUMBER)((PCHAR)(Mdl) + sizeof(MDL))`
#[inline]
pub unsafe fn MmGetMdlPfnArray(mdl: ddk::PMDL) -> *mut ULONG_PTR {
    (mdl as *mut u8).add(core::mem::size_of::<ddk::_MDL>()) as *mut ULONG_PTR
}

// ============================================================================
// MmGetSystemAddressForMdl (macro from wdm.h)
// ============================================================================

/// MmGetSystemAddressForMdl — return system VA for MDL, mapping if necessary.
#[inline]
pub unsafe fn MmGetSystemAddressForMdl(mdl: ddk::PMDL) -> PVOID {
    if (*mdl).MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
        (*mdl).MappedSystemVa
    } else {
        ddk::MmMapLockedPages(mdl, 0) // 0 = KernelMode
    }
}

/// `MmPrepareMdlForReuse(Mdl)` (wdm.h macro): a partial MDL that has been mapped gets its
/// system mapping torn down before the MDL is rebuilt; `MmUnmapLockedPages` also clears
/// MDL_PARTIAL_HAS_BEEN_MAPPED. The ONE copy every driver should call: four crates had
/// hand-written it, and classpnp's was an empty body, so every split disk transfer leaked
/// the PTEs scsiport mapped it with (NO_MORE_SYSTEM_PTES on i386 during setup's file copy).
#[inline]
pub unsafe fn MmPrepareMdlForReuse(mdl: ddk::PMDL) {
    const MDL_PARTIAL_HAS_BEEN_MAPPED: i16 = 0x0020;
    if (*mdl).MdlFlags & MDL_PARTIAL_HAS_BEEN_MAPPED != 0 {
        ddk::MmUnmapLockedPages((*mdl).MappedSystemVa, mdl);
    }
}

// ============================================================================
// MDL accessors (macros from mmfuncs.h)
// ============================================================================

/// `MmGetMdlVirtualAddress(Mdl)` = `(PVOID)((PCHAR)(Mdl)->StartVa + (Mdl)->ByteOffset)`.
#[inline]
pub unsafe fn MmGetMdlVirtualAddress(mdl: ddk::PMDL) -> PVOID {
    ((*mdl).StartVa as *mut u8).wrapping_add((*mdl).ByteOffset as usize) as PVOID
}

/// `MmGetMdlByteCount(Mdl)` = `(Mdl)->ByteCount`.
#[inline]
pub unsafe fn MmGetMdlByteCount(mdl: ddk::PMDL) -> ULONG {
    (*mdl).ByteCount
}

/// `MmGetMdlByteOffset(Mdl)` = `(Mdl)->ByteOffset`.
#[inline]
pub unsafe fn MmGetMdlByteOffset(mdl: ddk::PMDL) -> ULONG {
    (*mdl).ByteOffset
}

// ============================================================================
// MmGetSystemAddressForMdlSafe (macro from mmfuncs.h)
// ============================================================================

/// MmGetSystemAddressForMdlSafe — safe version with priority parameter.
#[inline]
pub unsafe fn MmGetSystemAddressForMdlSafe(mdl: ddk::PMDL, priority: ULONG) -> PVOID {
    if (*mdl).MdlFlags & (MDL_MAPPED_TO_SYSTEM_VA | MDL_SOURCE_IS_NONPAGED_POOL) != 0 {
        (*mdl).MappedSystemVa
    } else {
        ddk::MmMapLockedPagesSpecifyCache(
            mdl,
            0, // KernelMode
            ddk::_MEMORY_CACHING_TYPE_MmCached,
            core::ptr::null_mut(), // RequestedAddress
            0,                     // FALSE (BugCheckOnFailure)
            priority,
        )
    }
}

// ============================================================================
// PTE portable macros (MI_MAKE_DIRTY_PAGE etc. from miarm.h / arch mm.h)
// ============================================================================

/// MiMakeDirtyPage — set Dirty bit in PTE.
#[inline]
pub unsafe fn MiMakeDirtyPage(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_Dirty(1); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_NotDirty(0); }
}

/// MiMakeCleanPage — clear Dirty bit in PTE.
#[inline]
pub unsafe fn MiMakeCleanPage(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_Dirty(0); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_NotDirty(1); }
}

/// MiMakeAccessedPage — set Accessed bit in PTE.
#[inline]
pub unsafe fn MiMakeAccessedPage(pte: ddk::PMMPTE) {
    (*pte).u.Hard.set_Accessed(1);
}

/// MiPageDisableCache — disable caching in PTE.
#[inline]
pub unsafe fn MiPageDisableCache(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_CacheDisable(1); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_CacheType(0); }
}

/// MiPageWriteThrough — set write-through caching in PTE.
#[inline]
pub unsafe fn MiPageWriteThrough(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_WriteThrough(1); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_CacheType(1); }
}

/// MiPageWriteCombined — set write-combined caching in PTE.
#[inline]
pub unsafe fn MiPageWriteCombined(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_WriteThrough(0); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_CacheType(3); }
}

/// MiIsPageWriteable — check if PTE is writable.
#[inline]
pub unsafe fn MiIsPageWriteable(pte: ddk::PMMPTE) -> BOOLEAN {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { if (*pte).u.Hard.Write() == 1 { 1 } else { 0 } }
    #[cfg(target_arch = "aarch64")]
    { if (*pte).u.Hard.Writable() == 1 { 1 } else { 0 } }
}

/// MiIsPageDirty — check if PTE dirty bit is set.
#[inline]
pub unsafe fn MiIsPageDirty(pte: ddk::PMMPTE) -> BOOLEAN {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { if (*pte).u.Hard.Dirty() == 1 { 1 } else { 0 } }
    #[cfg(target_arch = "aarch64")]
    { if (*pte).u.Hard.NotDirty() == 0 { 1 } else { 0 } }
}

/// MiMakeOwnerPage — set Owner bit (user-mode accessible).
#[inline]
pub unsafe fn MiMakeOwnerPage(pte: ddk::PMMPTE) {
    (*pte).u.Hard.set_Owner(1);
}

/// MiMakeWritePage — set Write/Writable bit.
#[inline]
pub unsafe fn MiMakeWritePage(pte: ddk::PMMPTE) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    { (*pte).u.Hard.set_Write(1); }
    #[cfg(target_arch = "aarch64")]
    { (*pte).u.Hard.set_Writable(1); }
}

// ============================================================================
// MI_IS_MAPPED_PTE — arch-specific
// ============================================================================

/// MiIsMappedPte — check if PTE is mapped (valid, prototype, transition, or has PFN).
#[inline]
pub unsafe fn MiIsMappedPte(pte: ddk::PMMPTE) -> BOOLEAN {
    #[cfg(target_arch = "x86")]
    {
        // i386: single bitmask check on 32-bit Long
        if ((*pte).u.Long & 0xFFFFFC01) != 0 { 1 } else { 0 }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        // amd64/arm64: check individual fields
        if (*pte).u.Hard.Valid() != 0
            || (*pte).u.Proto.Prototype() != 0
            || (*pte).u.Trans.Transition() != 0
            || (*pte).u.Hard.PageFrameNumber() != 0
        {
            1
        } else {
            0
        }
    }
}

// ============================================================================
// InterlockedCompareExchangePte / InterlockedExchangePte — arch-specific atomics
// ============================================================================

/// MiInterlockedCompareExchangePte — atomic CAS on PTE.
#[inline]
pub unsafe fn MiInterlockedCompareExchangePte(
    pte: ddk::PMMPTE,
    exchange: LONG_PTR,
    comperand: LONG_PTR,
) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        use core::sync::atomic::{AtomicI32, Ordering};
        let atomic = &*(pte as *const AtomicI32);
        match atomic.compare_exchange(
            comperand as i32,
            exchange as i32,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(v) => v as LONG_PTR,
            Err(v) => v as LONG_PTR,
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        use core::sync::atomic::{AtomicI64, Ordering};
        let atomic = &*(pte as *const AtomicI64);
        match atomic.compare_exchange(
            comperand as i64,
            exchange as i64,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ) {
            Ok(v) => v as LONG_PTR,
            Err(v) => v as LONG_PTR,
        }
    }
}

/// MiInterlockedExchangePte — atomic exchange on PTE.
#[inline]
pub unsafe fn MiInterlockedExchangePte(
    pte: ddk::PMMPTE,
    value: LONG_PTR,
) -> LONG_PTR {
    #[cfg(target_arch = "x86")]
    {
        use core::sync::atomic::{AtomicI32, Ordering};
        let atomic = &*(pte as *const AtomicI32);
        atomic.swap(value as i32, Ordering::SeqCst) as LONG_PTR
    }
    #[cfg(not(target_arch = "x86"))]
    {
        use core::sync::atomic::{AtomicI64, Ordering};
        let atomic = &*(pte as *const AtomicI64);
        atomic.swap(value as i64, Ordering::SeqCst) as LONG_PTR
    }
}

// ============================================================================
// KeInvalidateTlbEntry — arch-specific TLB invalidation
// ============================================================================

/// KeInvalidateTlbEntry — invalidate a single TLB entry.
#[inline]
pub unsafe fn KeInvalidateTlbEntry(address: PVOID) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!("invlpg [{}]", in(reg) address, options(nostack, preserves_flags));
    }
    // M3 SMP (x86 both flavors): targeted remote shootdown — remotes invlpg
    // exactly this VA instead of a full cr3 reload (see KeFlushProcessTb note
    // for why the remote round-trip is load-bearing at -smp 2). Same symbol,
    // two transports: amd64 = the 0xE1 descriptor protocol (ke/amd64/ipi.rs),
    // i386 = the classic NT KiIpiSendPacket path (ke/i386/cpu.rs).
    #[cfg(all(feature = "smp", any(target_arch = "x86", target_arch = "x86_64")))]
    {
        unsafe extern "C" { fn KiTbShootdownSingle(Address: u64); }
        KiTbShootdownSingle(address as u64);
    }
    // arm64: `vae1is` is INNER-SHAREABLE — the invalidate broadcasts to every
    // core in hardware, so this single sequence IS the SMP shootdown (no IPI
    // protocol needed; the amd64 KiTbShootdownSingle machinery has no arm64
    // twin BY DESIGN — smp-bringup-quest.md M5d). The leading `dsb ishst`
    // publishes the caller's PTE store to remote table walkers BEFORE the
    // invalidate lands — without it a remote walk can refill from the stale
    // PTE (TCG forgives this; real hardware does not). Use VAE, not the
    // last-level-only VALE: this generic x86-semantic primitive is also used
    // while page-table levels cascade out of existence, so cached intermediate
    // descriptors must be invalidated too. The trailing `dsb ish` completes
    // only when every core has dropped the entry.
    #[cfg(target_arch = "aarch64")]
    {
        // TLBI's register operand is NOT a raw shifted pointer. It is three
        // fields: bits 43:0 = VA[55:12], bits 47:44 = the FEAT_TTL level hint,
        // bits 63:48 = ASID. A sign-extended kernel VA (0xFFFFF800_00000000)
        // shifted right by 12 is 0x000FFFFF_80000000, which lands 0xF in BOTH
        // the TTL field and the ASID field. Mask to VA bits only.
        //
        // Neither stray field can bite on TODAY's target, and it is worth
        // spelling out why so this mask does not get "optimized" away later:
        //   - ASID 0xF is ignored because every PTE in this port is GLOBAL
        //     (HARDWARE_PTE.NonGlobal is never set anywhere in the tree), and
        //     TLBI VAE1IS invalidates a global entry whatever the ASID operand.
        //   - TTL 0xF claims "64KB granule, level 3" — wrong for our 4KB
        //     granule, and a TTL inconsistent with the real entry makes it
        //     CONSTRAINED UNPREDICTABLE whether the entry is invalidated at
        //     all. It is inert only because QEMU's `-cpu cortex-a72` is
        //     ARMv8.0 and has no FEAT_TTL, so bits 47:44 are RES0.
        // Both go live the moment we run on an ARMv8.4+ core or start setting
        // NonGlobal for per-process ASIDs. Mask unconditionally. See 9.114.
        let tlbi_arg = ((address as u64) >> 12) & ((1u64 << 44) - 1);
        core::arch::asm!(
            "dsb ishst",
            "tlbi vae1is, {}",
            "dsb ish",
            "isb",
            in(reg) tlbi_arg,
            options(nostack),
        );
    }
}

// ============================================================================
// KeFlushProcessTb — arch-specific full TLB flush
// ============================================================================

/// KeFlushProcessTb — flush entire TLB.
#[inline]
pub unsafe fn KeFlushProcessTb() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let cr3: usize;
        core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, nomem));
        core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack));
    }
    // M3 SMP (x86 both flavors): the Rust callers of this primitive rely on
    // it for mappings other CPUs may have cached (verified: removing the
    // broadcast brings back the 0xF4/wild-RIP user corruption at -smp 2).
    // amd64's lazy-idle protocol (ke/amd64/ipi.rs) makes this cheap while the
    // other CPU idles; i386 pays the packet round-trip (ke/i386/cpu.rs).
    #[cfg(all(feature = "smp", any(target_arch = "x86", target_arch = "x86_64")))]
    {
        unsafe extern "C" { fn KiTbShootdownBroadcast(); }
        KiTbShootdownBroadcast();
    }
    // arm64: hardware-broadcast full flush (see KeInvalidateTlbEntry above for
    // the barrier + no-IPI-needed rationale).
    #[cfg(target_arch = "aarch64")]
    {
        core::arch::asm!(
            "dsb ishst",
            "tlbi vmalle1is",
            "dsb ish",
            "isb",
            options(nostack),
        );
    }
}

// ============================================================================
// MM_ANY_WS_LOCK_HELD — check if any working set lock is held
// ============================================================================

/// MM_ANY_WS_LOCK_HELD — returns TRUE if any working set lock is held by thread.
#[inline]
pub unsafe fn MM_ANY_WS_LOCK_HELD(thread: ddk::PETHREAD) -> BOOLEAN {
    let flags = &(*thread).__bindgen_anon_9.__bindgen_anon_1;
    if flags.OwnsProcessWorkingSetExclusive() != 0
        || flags.OwnsProcessWorkingSetShared() != 0
        || flags.OwnsSystemWorkingSetExclusive() != 0
        || flags.OwnsSystemWorkingSetShared() != 0
        || flags.OwnsSessionWorkingSetExclusive() != 0
        || flags.OwnsSessionWorkingSetShared() != 0
    {
        1
    } else {
        0
    }
}

// ============================================================================
// MiIsUserPde / MiIsUserPte — address range checks
// ============================================================================

extern "C" {
    static MiHighestUserPte: ddk::PMMPTE;
    #[cfg(target_arch = "x86")]
    static MiHighestUserPde: ddk::PMMPDE;
    static MmProtectToPteMask: [ULONG_PTR; 32];
    static MmKernelAddressSpace: ddk::PMMSUPPORT;
    static MiSessionBasePte: ddk::PMMPTE;
    static MiSessionLastPte: ddk::PMMPTE;
}

/// MiIsUserPte — check if address is in user PTE range.
#[inline]
pub unsafe fn MiIsUserPte(address: PVOID) -> BOOLEAN {
    #[cfg(target_arch = "x86")]
    {
        let pte_base = 0xC0000000usize;
        if (address as usize) >= pte_base && (address as usize) <= (MiHighestUserPte as usize) {
            1
        } else {
            0
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        // On 64-bit, MiAddressToPte(0) is the lowest user PTE
        let lowest_user_pte = crate::mm::MiAddressToPte(core::ptr::null_mut());
        if (address as usize) >= (lowest_user_pte as usize)
            && (address as usize) <= (MiHighestUserPte as usize)
        {
            1
        } else {
            0
        }
    }
}

/// MiIsUserPde — check if address is in user PDE range.
#[inline]
pub unsafe fn MiIsUserPde(address: PVOID) -> BOOLEAN {
    #[cfg(target_arch = "x86")]
    {
        let pde_base = crate::mm::MiAddressToPte(0xC0000000usize as PVOID) as usize;
        let lowest_user_pde = pde_base;
        if (address as usize) >= lowest_user_pde
            && (address as usize) <= (MiHighestUserPde as usize)
        {
            1
        } else {
            0
        }
    }
    #[cfg(not(target_arch = "x86"))]
    {
        let lowest_user_pde = crate::mm::MiAddressToPde(core::ptr::null_mut());
        // MiHighestUserPde not directly available on 64-bit, use MiAddressToPde(MiHighestUserPte)
        let highest_user_pde = crate::mm::MiAddressToPde(MiHighestUserPte as PVOID);
        if (address as usize) >= (lowest_user_pde as usize)
            && (address as usize) <= (highest_user_pde as usize)
        {
            1
        } else {
            0
        }
    }
}

// ============================================================================
// MiDetermineUserGlobalPteMask (FORCEINLINE from miarm.h)
// ============================================================================

/// MiIsUserPxe — top-level (PML4) entry check, 4-level paging only.
/// Mirrors `((ULONG_PTR)Address >> 11) == 0x1FFFFEDF6FB7DAULL` from miarm.h.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn MiIsUserPxe(address: PVOID) -> BOOLEAN {
    if ((address as u64) >> 11) == 0x1FFFFEDF6FB7DAu64 { 1 } else { 0 }
}

/// MiIsUserPpe — PDPT entry check, 3+-level paging only.
/// Mirrors `((ULONG_PTR)Address >> 20) == 0xFFFFF6FB7DAULL` from miarm.h.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn MiIsUserPpe(address: PVOID) -> BOOLEAN {
    if ((address as u64) >> 20) == 0xFFFFF6FB7DAu64 { 1 } else { 0 }
}

/// MiDetermineUserGlobalPteMask — compute the PTE mask for user/global pages.
/// Returns a PTE.u.Long value with Valid=1, Accessed=1, and Owner=1 for user PTEs.
#[inline]
pub unsafe fn MiDetermineUserGlobalPteMask(pointer_pte: ddk::PMMPTE) -> ULONG_PTR {
    let pte = core::mem::MaybeUninit::<ddk::MMPTE>::zeroed();
    let mut pte = pte.assume_init();
    pte.u.Hard.set_Valid(1);
    pte.u.Hard.set_Accessed(1);

    // Mirror the C predicate in `mm/ARM3/miarm.h:MiDetermineUserGlobalPteMask`.
    // On amd64/arm64 (4-level paging) the upper paging-structure entries
    // (PXE, PPE) are also user-mode mappings and must get Owner=1, otherwise
    // user-mode access through that hierarchy faults.
    let is_user;
    #[cfg(target_arch = "x86")]
    {
        is_user = MiIsUserPde(pointer_pte as PVOID) != 0
            || MiIsUserPte(pointer_pte as PVOID) != 0;
    }
    #[cfg(not(target_arch = "x86"))]
    {
        is_user = MiIsUserPxe(pointer_pte as PVOID) != 0
            || MiIsUserPpe(pointer_pte as PVOID) != 0
            || MiIsUserPde(pointer_pte as PVOID) != 0
            || MiIsUserPte(pointer_pte as PVOID) != 0;
    }
    if is_user {
        pte.u.Hard.set_Owner(1);
    }

    pte.u.Long as ULONG_PTR
}

// ============================================================================
// MI_MAKE_HARDWARE_PTE_KERNEL / USER / generic (FORCEINLINE from miarm.h)
// ============================================================================

/// MI_MAKE_HARDWARE_PTE_KERNEL — create kernel hardware PTE.
#[inline]
pub unsafe fn MI_MAKE_HARDWARE_PTE_KERNEL(
    new_pte: ddk::PMMPTE,
    _mapping_pte: ddk::PMMPTE,
    protection_mask: ULONG_PTR,
    page_frame_number: ULONG_PTR,
) {
    // Access PTE as native-width word (ULONG on i386, ULONG_PTR on 64-bit)
    let pte_word = new_pte as *mut ULONG_PTR;
    *pte_word = 0;
    (*new_pte).u.Hard.set_PageFrameNumber(page_frame_number as _);
    *pte_word |= MmProtectToPteMask[protection_mask as usize];
    (*new_pte).u.Hard.set_Valid(1);
}

/// MI_MAKE_HARDWARE_PTE_USER — create user hardware PTE.
#[inline]
#[track_caller]
pub unsafe fn MI_MAKE_HARDWARE_PTE_USER(
    new_pte: ddk::PMMPTE,
    _mapping_pte: ddk::PMMPTE,
    protection_mask: ULONG_PTR,
    page_frame_number: ULONG_PTR,
) {
    let pte_word = new_pte as *mut ULONG_PTR;
    *pte_word = 0;
    // The C original's three ASSERTs, restored: a mask granting no access must
    // never be made valid. See mi_user_pte_may_be_valid.
    if !crate::mm::mi_user_pte_may_be_valid(protection_mask as usize) {
        return;
    }
    (*new_pte).u.Hard.set_Valid(1);
    (*new_pte).u.Hard.set_Owner(1);
    (*new_pte).u.Hard.set_PageFrameNumber(page_frame_number as _);
    *pte_word |= MmProtectToPteMask[protection_mask as usize];
}

/// MI_MAKE_HARDWARE_PTE — create hardware PTE with user/global mask.
#[inline]
pub unsafe fn MI_MAKE_HARDWARE_PTE(
    new_pte: ddk::PMMPTE,
    mapping_pte: ddk::PMMPTE,
    protection_mask: ULONG_PTR,
    page_frame_number: ULONG_PTR,
) {
    let pte_word = new_pte as *mut ULONG_PTR;
    *pte_word = MiDetermineUserGlobalPteMask(mapping_pte);
    *pte_word |= MmProtectToPteMask[protection_mask as usize];
    (*new_pte).u.Hard.set_PageFrameNumber(page_frame_number as _);
}

/// MI_UPDATE_VALID_PTE — update an already-valid PTE (volatile write).
#[inline]
pub unsafe fn MI_UPDATE_VALID_PTE(pte: ddk::PMMPTE, temp_pte: ddk::MMPTE) {
    core::ptr::write_volatile(pte, temp_pte);
}

// ============================================================================
// MiIsPdeForAddressValid — PDE hierarchy check (64-bit only)
// ============================================================================

/// MiIsPdeForAddressValid — check if all PDE levels are valid for an address.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn MiIsPdeForAddressValid(address: PVOID) -> BOOLEAN {
    if (*(crate::mm::MiAddressToPxe(address) as ddk::PMMPTE)).u.Hard.Valid() == 0 {
        return 0;
    }
    if (*(crate::mm::MiAddressToPpe(address) as ddk::PMMPTE)).u.Hard.Valid() == 0 {
        return 0;
    }
    if (*(crate::mm::MiAddressToPde(address) as ddk::PMMPTE)).u.Hard.Valid() == 0 {
        return 0;
    }
    1
}

// ============================================================================
// Working set lock functions (FORCEINLINE from miarm.h)
// ============================================================================

// Flight-recorder tap (sync-instrumentation §1) — resolved at link time to
// ntoskrnl-rust's exported VibeosFlightrecLog (this crate sits BELOW it in the
// dep graph). EXCLUSIVE WS lock/unlock only: cheap, and a leaked WS lock that
// strands a shared waiter is by definition exclusive. Ops must match
// flightrec.rs OP_WS_LOCK_EXCL/OP_WS_UNLOCK_EXCL.
extern "C" {
    fn VibeosFlightrecLog(op: u32, object: usize, thread: usize, data: u32);
}
const OP_WS_LOCK_EXCL: u32 = 16;
const OP_WS_UNLOCK_EXCL: u32 = 17;

/// MiLockProcessWorkingSet — acquire exclusive working set lock.
#[inline]
pub unsafe fn MiLockProcessWorkingSet(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    super::ke::KeEnterGuardedRegion();
    ddk::ExAcquirePushLockExclusive(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(1);
    VibeosFlightrecLog(
        OP_WS_LOCK_EXCL,
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex) as usize,
        thread as usize,
        0,
    );
}

/// MiUnlockProcessWorkingSet — release exclusive working set lock.
#[inline]
pub unsafe fn MiUnlockProcessWorkingSet(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    VibeosFlightrecLog(
        OP_WS_UNLOCK_EXCL,
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex) as usize,
        thread as usize,
        0,
    );
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(0);
    ddk::ExReleasePushLockExclusive(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
    super::ke::KeLeaveGuardedRegion();
}

/// MiLockProcessWorkingSetShared — acquire shared working set lock.
#[inline]
pub unsafe fn MiLockProcessWorkingSetShared(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    super::ke::KeEnterGuardedRegion();
    ddk::ExAcquirePushLockShared(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetShared(1);
}

/// MiUnlockProcessWorkingSetShared — release shared working set lock.
#[inline]
pub unsafe fn MiUnlockProcessWorkingSetShared(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetShared(0);
    ddk::ExReleasePushLockShared(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
    super::ke::KeLeaveGuardedRegion();
}

/// MiLockProcessWorkingSetUnsafe — acquire exclusive working set lock (unsafe variant, no guarded region).
/// Matches C miarm.h:1219 — caller must already have APCs disabled.
#[inline]
pub unsafe fn MiLockProcessWorkingSetUnsafe(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    ddk::ExAcquirePushLockExclusive(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
    (*process).Vm.Flags.set_AcquiredUnsafe(1);
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(1);
    VibeosFlightrecLog(
        OP_WS_LOCK_EXCL,
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex) as usize,
        thread as usize,
        1,
    );
}

/// MiUnlockProcessWorkingSetUnsafe — release exclusive working set lock (unsafe variant).
/// Matches C miarm.h:1289 — does NOT touch APC state (caller manages it).
#[inline]
pub unsafe fn MiUnlockProcessWorkingSetUnsafe(process: ddk::PEPROCESS, thread: ddk::PETHREAD) {
    VibeosFlightrecLog(
        OP_WS_UNLOCK_EXCL,
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex) as usize,
        thread as usize,
        1,
    );
    (*process).Vm.Flags.set_AcquiredUnsafe(0);
    (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(0);
    ddk::ExReleasePushLockExclusive(
        core::ptr::addr_of_mut!((*process).Vm.WorkingSetMutex),
    );
}

extern "C" {
    static mut MmSystemCacheWs: ddk::_MMSUPPORT;
}

#[inline]
unsafe fn _ws_set_owns(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT, exclusive: bool) {
    if ws == core::ptr::addr_of_mut!(MmSystemCacheWs) {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSystemWorkingSetExclusive(1);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSystemWorkingSetShared(1);
        }
    } else if (*ws).Flags.SessionSpace() != 0 {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSessionWorkingSetExclusive(1);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSessionWorkingSetShared(1);
        }
    } else {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(1);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetShared(1);
        }
    }
}

#[inline]
unsafe fn _ws_clear_owns(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT, exclusive: bool) {
    if ws == core::ptr::addr_of_mut!(MmSystemCacheWs) {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSystemWorkingSetExclusive(0);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSystemWorkingSetShared(0);
        }
    } else if (*ws).Flags.SessionSpace() != 0 {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSessionWorkingSetExclusive(0);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsSessionWorkingSetShared(0);
        }
    } else {
        if exclusive {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(0);
        } else {
            (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetShared(0);
        }
    }
}

/// MiLockWorkingSet — acquire exclusive working set lock on MMSUPPORT.
#[inline]
pub unsafe fn MiLockWorkingSet(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT) {
    super::ke::KeEnterGuardedRegion();
    ddk::ExAcquirePushLockExclusive(core::ptr::addr_of_mut!((*ws).WorkingSetMutex));
    _ws_set_owns(thread, ws, true);
    VibeosFlightrecLog(
        OP_WS_LOCK_EXCL,
        core::ptr::addr_of_mut!((*ws).WorkingSetMutex) as usize,
        thread as usize,
        2,
    );
}

/// MiUnlockWorkingSet — release exclusive working set lock on MMSUPPORT.
#[inline]
pub unsafe fn MiUnlockWorkingSet(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT) {
    VibeosFlightrecLog(
        OP_WS_UNLOCK_EXCL,
        core::ptr::addr_of_mut!((*ws).WorkingSetMutex) as usize,
        thread as usize,
        2,
    );
    _ws_clear_owns(thread, ws, true);
    ddk::ExReleasePushLockExclusive(core::ptr::addr_of_mut!((*ws).WorkingSetMutex));
    super::ke::KeLeaveGuardedRegion();
}

/// MiLockWorkingSetShared — acquire shared working set lock on MMSUPPORT.
#[inline]
pub unsafe fn MiLockWorkingSetShared(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT) {
    super::ke::KeEnterGuardedRegion();
    ddk::ExAcquirePushLockShared(core::ptr::addr_of_mut!((*ws).WorkingSetMutex));
    _ws_set_owns(thread, ws, false);
}

/// MiUnlockWorkingSetShared — release shared working set lock on MMSUPPORT.
#[inline]
pub unsafe fn MiUnlockWorkingSetShared(thread: ddk::PETHREAD, ws: ddk::PMMSUPPORT) {
    _ws_clear_owns(thread, ws, false);
    ddk::ExReleasePushLockShared(core::ptr::addr_of_mut!((*ws).WorkingSetMutex));
    super::ke::KeLeaveGuardedRegion();
}

// ============================================================================
// MiLockProcessWorkingSetForFault / MiUnlockProcessWorkingSetForFault
// ============================================================================

/// MiLockProcessWorkingSetForFault — lock working set for page fault handling.
/// Matches C miarm.h:1551 — `Safe` selects safe vs unsafe lock; `Shared` only valid if `Safe`.
#[inline]
pub unsafe fn MiLockProcessWorkingSetForFault(
    process: ddk::PEPROCESS,
    thread: ddk::PETHREAD,
    safe: BOOLEAN,
    shared: BOOLEAN,
) {
    if safe != 0 {
        if shared != 0 {
            MiLockProcessWorkingSetShared(process, thread);
        } else {
            MiLockProcessWorkingSet(process, thread);
        }
    } else {
        // Unsafe lock cannot be shared (C asserts this).
        MiLockProcessWorkingSetUnsafe(process, thread);
    }
}

/// MiUnlockProcessWorkingSetForFault — unlock working set after page fault handling.
/// Matches C miarm.h:1518 — checks `MI_IS_WS_UNSAFE` to pick the right release path
/// and writes the (Safe, Shared) pair so the caller can re-lock with the same flavor.
#[inline]
pub unsafe fn MiUnlockProcessWorkingSetForFault(
    process: ddk::PEPROCESS,
    thread: ddk::PETHREAD,
    safe: *mut BOOLEAN,
    shared: *mut BOOLEAN,
) {
    if (*process).Vm.Flags.AcquiredUnsafe() != 0 {
        // Was an unsafe acquisition — release unsafely (no GuardedRegion leave).
        MiUnlockProcessWorkingSetUnsafe(process, thread);
        *safe = 0;
        *shared = 0;
    } else if (*thread).__bindgen_anon_9.__bindgen_anon_1.OwnsProcessWorkingSetExclusive() != 0 {
        // Safe + exclusive.
        MiUnlockProcessWorkingSet(process, thread);
        *safe = 1;
        *shared = 0;
    } else {
        // Safe + shared (implied — owner is shared).
        MiUnlockProcessWorkingSetShared(process, thread);
        *safe = 1;
        *shared = 1;
    }
}

// ============================================================================
// MiConvertSharedWorkingSetLockToExclusive
// ============================================================================

/// MiConvertSharedWorkingSetLockToExclusive — try to upgrade shared to exclusive.
#[inline]
pub unsafe fn MiConvertSharedWorkingSetLockToExclusive(
    thread: ddk::PETHREAD,
    vm: ddk::PMMSUPPORT,
) -> BOOLEAN {
    // Try to convert the push lock
    let result = super::ex::ExConvertPushLockSharedToExclusive(
        core::ptr::addr_of_mut!((*vm).WorkingSetMutex),
    );

    if result != 0 {
        // Successfully converted
        (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetShared(0);
        (*thread).__bindgen_anon_9.__bindgen_anon_1.set_OwnsProcessWorkingSetExclusive(1);
        1
    } else {
        0
    }
}

// ============================================================================
// Expansion lock (FORCEINLINE from miarm.h)
// ============================================================================

extern "C" {
    static mut MmExpansionLock: crate::ke::KSPIN_LOCK;
}

/// MiAcquireExpansionLock — acquire MmExpansionLock at DISPATCH_LEVEL.
#[inline]
pub unsafe fn MiAcquireExpansionLock() -> KIRQL {
    let mut old_irql: KIRQL = 0;
    crate::ke::KeAcquireSpinLock(core::ptr::addr_of_mut!(MmExpansionLock), &mut old_irql);
    old_irql
}

/// MiReleaseExpansionLock — release MmExpansionLock.
#[inline]
pub unsafe fn MiReleaseExpansionLock(old_irql: KIRQL) {
    crate::ke::KeReleaseSpinLock(core::ptr::addr_of_mut!(MmExpansionLock), old_irql);
}

// ============================================================================
// Simple macro wrappers
// ============================================================================

/// RMAP_IS_SEGMENT — check if address has segment flag bits set.
/// C macro: #define RMAP_SEGMENT_MASK (~((ULONG_PTR)0xff))
/// Check: ((ULONG_PTR)(x) & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK
#[inline]
pub unsafe fn RMAP_IS_SEGMENT(address: PVOID) -> BOOLEAN {
    // ~0xFF on EVERY arch — the low 8 bits carry the segment page index, so the
    // mask is not pointer-width dependent. The old 64-bit `!0xFFF` happened to
    // accept the same inputs (a constructed segment address has bits 8..63 set,
    // so bits 12..63 are set too) but encodes the wrong invariant.
    const RMAP_SEGMENT_MASK: usize = !0xFF;

    if ((address as usize) & RMAP_SEGMENT_MASK) == RMAP_SEGMENT_MASK {
        1
    } else {
        0
    }
}

/// PAGE_ROUND_DOWN — round address down to page boundary.
#[inline]
pub unsafe fn PAGE_ROUND_DOWN(address: PVOID) -> ULONG_PTR {
    (address as ULONG_PTR) & !((0x1000usize - 1) as ULONG_PTR)
}

/// PFN_FROM_SSE — extract PFN from section segment entry.
#[inline]
pub unsafe fn PFN_FROM_SSE(entry: ULONG_PTR) -> ULONG_PTR {
    entry >> 12 // PAGE_SHIFT = 12
}

/// MM_IS_WAIT_PTE — check if entry is a wait PTE.
#[inline]
pub unsafe fn MM_IS_WAIT_PTE(entry: ULONG_PTR) -> BOOLEAN {
    // MM_WAIT_ENTRY defined in ntoskrnl/include/internal/mm.h — pointer-width
    // dependent: 0x7ffffc00 (32-bit) / 0x7FFFFFFFFFFFFC00 (64-bit). The prior
    // 0x7FFFFFFF was a placeholder wrong on BOTH widths, so MM_IS_WAIT_PTE never
    // recognized a wait-PTE written by MAKE_SWAP_SSE(MM_WAIT_ENTRY) → collided
    // section faults never waited → page-in livelock (arm64 fastfat, amd64 stall).
    #[cfg(target_pointer_width = "64")]
    const MM_WAIT_ENTRY: ULONG_PTR = 0x7FFFFFFFFFFFFC00;
    #[cfg(target_pointer_width = "32")]
    const MM_WAIT_ENTRY: ULONG_PTR = 0x7ffffc00;
    if (entry & 0x1) != 0 && (entry >> 1) == MM_WAIT_ENTRY {
        1
    } else {
        0
    }
}

/// IS_SWAP_FROM_SSE — check if section segment entry is a swap entry.
#[inline]
pub unsafe fn IS_SWAP_FROM_SSE(entry: ULONG_PTR) -> BOOLEAN {
    // C macro (mm.h): ((E) & 0x00000001). The swap bit (0x1) and dirty bit (0x2)
    // are orthogonal — a dirty swap entry has both set, and callers test
    // IS_SWAP_FROM_SSE / IS_DIRTY_SSE independently. The prior extra
    // `&& (E & 0x2) == 0` wrongly declassified dirty swap entries.
    if (entry & 0x00000001) != 0 {
        1
    } else {
        0
    }
}

/// SWAPENTRY_FROM_SSE — extract swap entry from section segment entry.
#[inline]
pub unsafe fn SWAPENTRY_FROM_SSE(entry: ULONG_PTR) -> ULONG_PTR {
    entry >> 1
}

/// MmSetDirtyPage — set dirty bit for a page (calls MmSetDirtyBit).
#[inline]
pub unsafe fn MmSetDirtyPage(process: ddk::PEPROCESS, address: PVOID) {
    ddk::MmSetDirtyBit(process, address, 1);
}

extern "C" {
    static mut MmTotalCommittedPages: ULONG_PTR;
    static mut MmPeakCommitment: ULONG_PTR;
}

/// UpdateTotalCommittedPages — atomically update committed pages with CAS loop.
#[inline]
pub unsafe fn UpdateTotalCommittedPages(delta: i32) {
    use core::sync::atomic::{AtomicUsize, Ordering};

    let total = &*(&raw const MmTotalCommittedPages as *const _ as *const AtomicUsize);
    let peak = &*(&raw const MmPeakCommitment as *const _ as *const AtomicUsize);

    let new_total = if delta >= 0 {
        total.fetch_add(delta as usize, Ordering::SeqCst).wrapping_add(delta as usize)
    } else {
        total.fetch_sub((-delta) as usize, Ordering::SeqCst).wrapping_sub((-delta) as usize)
    };

    // Update peak if new total exceeds it
    let mut current_peak = peak.load(Ordering::SeqCst);
    while new_total > current_peak {
        match peak.compare_exchange(current_peak, new_total, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => break,
            Err(actual) => current_peak = actual,
        }
    }
}

// ============================================================================
// InterlockedIncrementUL / InterlockedExchangeAddUL
// ============================================================================

/// InterlockedIncrementUL — atomic increment of ULONG.
#[inline]
pub unsafe fn InterlockedIncrementUL(addend: *mut ULONG) -> ULONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(addend as *const AtomicI32);
    (atomic.fetch_add(1, Ordering::SeqCst) + 1) as ULONG
}

/// InterlockedExchangeAddUL — atomic exchange-add on ULONG.
#[inline]
pub unsafe fn InterlockedExchangeAddUL(addend: *mut ULONG, value: ULONG) -> ULONG {
    use core::sync::atomic::{AtomicI32, Ordering};
    let atomic = &*(addend as *const AtomicI32);
    atomic.fetch_add(value as i32, Ordering::SeqCst) as ULONG
}

// ============================================================================
// MmDereferenceSegment, ExAllocatePoolZero, MmGetAddressSpaceOwner, etc.
// ============================================================================

/// MmDereferenceSegment — calls MmDereferenceSegmentWithLock(Segment, MM_NOIRQL).
#[inline]
pub unsafe fn MmDereferenceSegment(segment: ddk::PMM_SECTION_SEGMENT) {
    const MM_NOIRQL: KIRQL = 0xFF; // Sentinel value meaning "no IRQL"
    ddk::MmDereferenceSegmentWithLock(segment, MM_NOIRQL);
}

/// ExAllocatePoolZero — allocate pool memory and zero it.
#[inline]
pub unsafe fn ExAllocatePoolZero(
    pool_type: ddk::POOL_TYPE,
    number_of_bytes: ULONG_PTR,
    tag: ULONG,
) -> PVOID {
    let alloc = ddk::ExAllocatePoolWithTag(pool_type, number_of_bytes as _, tag);
    if !alloc.is_null() {
        core::ptr::write_bytes(alloc as *mut u8, 0, number_of_bytes as usize);
    }
    alloc
}

/// ExAllocatePoolUninitialized — exfuncs.h FORCEINLINE: `ExAllocatePoolWithTag`, nothing more.
#[inline]
pub unsafe fn ExAllocatePoolUninitialized(
    pool_type: ddk::POOL_TYPE,
    number_of_bytes: ULONG_PTR,
    tag: ULONG,
) -> PVOID {
    ddk::ExAllocatePoolWithTag(pool_type, number_of_bytes as _, tag)
}

/// MmGetAddressSpaceOwner — return the EPROCESS that owns this address space.
#[inline]
pub unsafe fn MmGetAddressSpaceOwner(address_space: ddk::PMMSUPPORT) -> ddk::PEPROCESS {
    if address_space == MmKernelAddressSpace {
        core::ptr::null_mut()
    } else {
        // CONTAINING_RECORD(address_space, EPROCESS, Vm)
        let vm_offset = core::mem::offset_of!(ddk::_EPROCESS, Vm);
        ((address_space as *mut u8).sub(vm_offset)) as ddk::PEPROCESS
    }
}

/// MmGetKernelAddressSpace — return pointer to kernel address space.
#[inline]
pub unsafe fn MmGetKernelAddressSpace() -> ddk::PMMSUPPORT {
    MmKernelAddressSpace
}

/// MmLockAddressSpace — acquire the guarded mutex for address creation.
#[inline]
pub unsafe fn MmLockAddressSpace(address_space: ddk::PMMSUPPORT) {
    let vm_offset = core::mem::offset_of!(ddk::_EPROCESS, Vm);
    let process = ((address_space as *mut u8).sub(vm_offset)) as ddk::PEPROCESS;
    ddk::KeAcquireGuardedMutex(core::ptr::addr_of_mut!((*process).AddressCreationLock));
}

/// MmUnlockAddressSpace — release the guarded mutex for address creation.
#[inline]
pub unsafe fn MmUnlockAddressSpace(address_space: ddk::PMMSUPPORT) {
    let vm_offset = core::mem::offset_of!(ddk::_EPROCESS, Vm);
    let process = ((address_space as *mut u8).sub(vm_offset)) as ddk::PEPROCESS;
    ddk::KeReleaseGuardedMutex(core::ptr::addr_of_mut!((*process).AddressCreationLock));
}

/// MmGetCurrentAddressSpace — return current process's address space.
#[inline]
pub unsafe fn MmGetCurrentAddressSpace() -> ddk::PMMSUPPORT {
    let thread = crate::mm::KeGetCurrentThread() as ddk::PKTHREAD;
    let process = (*thread).__bindgen_anon_1.ApcState.Process as ddk::PEPROCESS;
    core::ptr::addr_of_mut!((*process).Vm)
}

/// MiIsRosSectionObject — check if section is a ReactOS section object.
#[inline]
pub unsafe fn MiIsRosSectionObject(section: ddk::PSECTION) -> BOOLEAN {
    (*section).u.Flags.filler() as BOOLEAN
}

// ============================================================================
// MmInitializeMdl (macro from mmfuncs.h)
// ============================================================================

/// MmInitializeMdl — initialize MDL fields.
#[inline]
pub unsafe fn MmInitializeMdl(mdl: ddk::PMDL, base_va: PVOID, length: ULONG) {
    (*mdl).Next = core::ptr::null_mut();

    // ADDRESS_AND_SIZE_TO_SPAN_PAGES
    let start_page = (base_va as usize) >> 12;
    let end_page = ((base_va as usize) + length as usize + 0xFFF) >> 12;
    let span_pages = end_page - start_page;

    (*mdl).Size = (core::mem::size_of::<ddk::_MDL>()
        + core::mem::size_of::<ULONG_PTR>() * span_pages) as i16;
    (*mdl).MdlFlags = 0;
    (*mdl).StartVa = ((base_va as usize) & !(0xFFF)) as PVOID; // PAGE_ALIGN
    (*mdl).ByteOffset = (base_va as ULONG) & 0xFFF; // BYTE_OFFSET
    (*mdl).ByteCount = length;
}

// ============================================================================
// PFN reference counting (FORCEINLINE from miarm.h)
// ============================================================================

extern "C" {
    static mut MmWorkingSetList: ddk::PMMWSL;
}

/// MiIncrementPageTableReferences — increment page table reference count.
#[inline]
pub unsafe fn MiIncrementPageTableReferences(address: PVOID) -> u16 {
    #[cfg(target_arch = "x86")]
    {
        // i386 (2-level paging): index into MmWorkingSetList->UsedPageTableEntries
        // MiGetPdeOffset(Va) = (ULONG_PTR)(Va) / PDE_MAPPED_VA (4MB = 0x400000)
        // NOTE: C code intentionally indexes past the array bounds (768 entries)
        // for kernel VAs (index >= 768). Use raw pointer arithmetic to avoid
        // Rust bounds checking panic.
        let pde_offset = (address as usize) / 0x400000;
        let ref_count = (*MmWorkingSetList).UsedPageTableEntries.as_mut_ptr().add(pde_offset);
        *ref_count += 1;
        *ref_count
    }
    #[cfg(not(target_arch = "x86"))]
    {
        // 64-bit (3+ level paging): PFN from PDE
        let pde = crate::mm::MiAddressToPde(address) as ddk::PMMPTE;
        let pfn = &mut *(crate::mm::MiGetPfnEntry((*pde).u.Hard.PageFrameNumber() as _) as ddk::PMMPFN);
        let count = pfn.__bindgen_anon_1.OriginalPte.u.Soft.UsedPageTableEntries() + 1;
        pfn.__bindgen_anon_1.OriginalPte.u.Soft.set_UsedPageTableEntries(count);
        count as u16
    }
}

extern "C" {
    static mut MmSystemLockPagesCount: ULONG_PTR;
}
// MiDecrementReferenceCount is stdcall on i386, cdecl on others
extern "system" {
    fn MiDecrementReferenceCount(Pfn1: ddk::PMMPFN, PageFrameIndex: ULONG_PTR);
}

/// Atomic decrement of ULONG_PTR (InterlockedDecrementSizeT).
#[inline]
unsafe fn InterlockedDecrementSizeT(addend: *mut ULONG_PTR) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(addend as *const AtomicUsize);
    atomic.fetch_sub(1, Ordering::SeqCst);
}

/// Atomic increment of ULONG_PTR (InterlockedIncrementSizeT).
#[inline]
unsafe fn InterlockedIncrementSizeT(addend: *mut ULONG_PTR) {
    use core::sync::atomic::{AtomicUsize, Ordering};
    let atomic = &*(addend as *const AtomicUsize);
    atomic.fetch_add(1, Ordering::SeqCst);
}

/// Atomic compare-exchange on i16.
#[inline]
unsafe fn InterlockedCompareExchange16(dest: *mut i16, exchange: i16, comperand: i16) -> i16 {
    use core::sync::atomic::{AtomicI16, Ordering};
    let atomic = &*(dest as *const AtomicI16);
    match atomic.compare_exchange(comperand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(v) | Err(v) => v,
    }
}

/// MiDropLockCount — drop a locked page without dereferencing it.
#[inline]
pub unsafe fn MiDropLockCount(pfn: ddk::PMMPFN) {
    // If last reference, update MmSystemLockPagesCount
    if (*pfn).u3.e2.ReferenceCount == 1 {
        InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
    }
}

/// MiDereferencePfnAndDropLockCount — drop a locked page and dereference it.
#[inline]
pub unsafe fn MiDereferencePfnAndDropLockCount(pfn: ddk::PMMPFN) {
    loop {
        let old_ref_count = (*pfn).u3.e2.ReferenceCount;

        // Are we the last one?
        if old_ref_count == 1 {
            InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
            let page_frame_index = crate::mm::MiGetPfnEntryIndex(pfn as crate::mm::PMMPFN) as ULONG_PTR;
            MiDecrementReferenceCount(pfn, page_frame_index);
            return;
        }

        // Try atomic decrement via CAS
        let ref_count = InterlockedCompareExchange16(
            &mut (*pfn).u3.e2.ReferenceCount as *mut u16 as *mut i16,
            (old_ref_count - 1) as i16,
            old_ref_count as i16,
        ) as u16;

        if old_ref_count == ref_count {
            // Success — check if we went from 2 to 1 with shares
            if ref_count == 2 && (*pfn).u2.ShareCount >= 1 {
                InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
            }
            return;
        }
    }
}

/// MiReferenceProbedPageAndBumpLockCount — reference a probed page via CAS loop.
#[inline]
pub unsafe fn MiReferenceProbedPageAndBumpLockCount(pfn: ddk::PMMPFN) {
    InterlockedIncrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));

    let old_ref_count;
    loop {
        let old = (*pfn).u3.e2.ReferenceCount;
        let actual = InterlockedCompareExchange16(
            &mut (*pfn).u3.e2.ReferenceCount as *mut u16 as *mut i16,
            (old + 1) as i16,
            old as i16,
        ) as u16;
        if old == actual {
            old_ref_count = old;
            break;
        }
    }

    // Was this the first lock attempt? If not, undo our bump.
    if old_ref_count != 1 {
        InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
    }
}

/// MiReferenceUnusedPageAndBumpLockCount — reference an unused page.
#[inline]
pub unsafe fn MiReferenceUnusedPageAndBumpLockCount(pfn: ddk::PMMPFN) {
    InterlockedIncrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));

    // InterlockedIncrement16
    let new_ref_count = {
        use core::sync::atomic::{AtomicI16, Ordering};
        let atomic = &*(&mut (*pfn).u3.e2.ReferenceCount as *mut u16 as *const AtomicI16);
        (atomic.fetch_add(1, Ordering::SeqCst) + 1) as u16
    };

    // If someone had already locked the page, undo our bump
    if new_ref_count != 1 {
        InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
    }
}

/// MiReferenceUsedPageAndBumpLockCount — reference a used page.
#[inline]
pub unsafe fn MiReferenceUsedPageAndBumpLockCount(pfn: ddk::PMMPFN) {
    InterlockedIncrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));

    // InterlockedIncrement16
    let new_ref_count = {
        use core::sync::atomic::{AtomicI16, Ordering};
        let atomic = &*(&mut (*pfn).u3.e2.ReferenceCount as *mut u16 as *const AtomicI16);
        (atomic.fetch_add(1, Ordering::SeqCst) + 1) as u16
    };

    if new_ref_count == 2 {
        // Is it locked or shared?
        if (*pfn).u2.ShareCount == 0 {
            // Locked, so undo
            InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
        }
    } else {
        // Someone had already locked the page, undo our bump
        InterlockedDecrementSizeT(core::ptr::addr_of_mut!(MmSystemLockPagesCount));
    }
}

extern "C" {
    // MmFreePagesByColor[FreePageList+1] = array of 2 pointers to MMCOLOR_TABLES
    static mut MmFreePagesByColor: [ddk::PMMCOLOR_TABLES; 2];
}

/// MiRemoveZeroPageSafe — remove a zero page from the free list if available.
#[inline]
pub unsafe fn MiRemoveZeroPageSafe(color: ULONG) -> ULONG_PTR {
    // ZeroedPageList = 0, LIST_HEAD = (PFN_NUMBER)-1
    let color_table = MmFreePagesByColor[0]; // PMMCOLOR_TABLES
    if (*color_table.add(color as usize)).Flink != !0u32 as _ {
        ddk::MiRemoveZeroPage(color) as ULONG_PTR
    } else {
        0
    }
}

// ============================================================================
// MiSynchronizeSystemPde — i386 only
// ============================================================================

/// MiSynchronizeSystemPde — synchronize system PDE on i386 (2-level paging).
/// Copies the PDE from MmSystemPagePtes (double-mapped system page directory).
#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn MiSynchronizeSystemPde(pde: ddk::PMMPDE) -> BOOLEAN {
    // SYSTEM_PD_SIZE = PAGE_SIZE = 0x1000 on i386
    const SYSTEM_PD_SIZE: usize = 0x1000;
    let index = ((pde as usize) & (SYSTEM_PD_SIZE - 1)) / core::mem::size_of::<ddk::MMPTE>();

    if (*pde).u.Hard.Valid() != 0 {
        return 1; // Already valid
    }

    // Read from double-mapped system page directory
    let source_pde = *(crate::mm::MmSystemPagePtes.add(index) as ddk::PMMPTE);
    if source_pde.u.Hard.Valid() == 0 {
        return 0; // Source not valid either
    }

    // MI_WRITE_VALID_PDE — volatile write
    core::ptr::write_volatile(pde, source_pde);
    // KeMemoryBarrierWithoutFence — compiler barrier
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    1
}

// ============================================================================
// KiAcquireProcessLockRaiseToSynch / KiReleaseProcessLock
// ============================================================================

/// KiAcquireProcessLockRaiseToSynch — acquire process lock, raise to synch level.
#[inline]
pub unsafe fn KiAcquireProcessLockRaiseToSynch(
    process: ddk::PKPROCESS,
    handle: ddk::PKLOCK_QUEUE_HANDLE,
) {
    ddk::KeAcquireInStackQueuedSpinLockRaiseToSynch(
        core::ptr::addr_of_mut!((*process).ProcessLock),
        handle,
    );
}

/// KiReleaseProcessLock — release process lock.
#[inline]
pub unsafe fn KiReleaseProcessLock(handle: ddk::PKLOCK_QUEUE_HANDLE) {
    ddk::KeReleaseInStackQueuedSpinLock(handle);
}

// ============================================================================
// Probe helpers
// ============================================================================

/// ProbeForWriteSize_t — probe a SIZE_T for write access.
#[inline]
pub unsafe fn ProbeForWriteSize_t(ptr: *mut ULONG_PTR) {
    crate::probe::probe_user_range(ptr);
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
}

/// ProbeForWriteIoStatusBlock — probe an IO_STATUS_BLOCK for write access.
#[inline]
pub unsafe fn ProbeForWriteIoStatusBlock(ptr: *mut ddk::IO_STATUS_BLOCK) {
    crate::probe::probe_user_range(ptr);
    // Read then write back to trigger page fault if not writeable
    let val = core::ptr::read_volatile(ptr);
    core::ptr::write_volatile(ptr, val);
}

// ============================================================================
// KfRaiseIrql / KfLowerIrql wrappers
// ============================================================================

/// KfRaiseIrql — amd64: read/write CR8.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KfRaiseIrql(new_irql: KIRQL) -> KIRQL {
    super::ke::KfRaiseIrql_amd64(new_irql)
}

/// KfLowerIrql — amd64: write CR8.
#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn KfLowerIrql(new_irql: KIRQL) {
    super::ke::KeLowerIrql_amd64(new_irql);
}
