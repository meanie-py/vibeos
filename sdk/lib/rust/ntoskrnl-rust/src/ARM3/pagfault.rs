use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{KEVENT, KPROCESSOR_MODE, KeAreAllApcsDisabled, KeInitializeEvent, KeIsAttachedProcess, KeWaitForSingleObject, KeZeroPages, KiAcquireProcessLockRaiseToSynch, KiReleaseProcessLock, MiDeletePte, MiDereferencePfnAndDropLockCount, MiDetermineUserGlobalPteMask, MiDropLockCount, MiIncrementPageTableReferences, MiInitializePfn, MiIsUserPde, MiIsUserPte, MiLocateAddress, MiLockProcessWorkingSet, MiLockWorkingSet, MiReadPageFile, MiReferenceUnusedPageAndBumpLockCount, MiReferenceUsedPageAndBumpLockCount, MiRemoveZeroPageSafe, MiUnlinkPageFromList, MiUnlockProcessWorkingSet, MiUnlockWorkingSet, PEPROCESS, PETHREAD, PKEVENT, PKPROCESS, PKTRAP_FRAME, PMM_AVL_TABLE, PMMSUPPORT, PMMVAD, PTEB, PsGetCurrentThread, ZwAllocateVirtualMemory, _EPROCESS, _ETHREAD, _HANDLE_TABLE_ENTRY, _IMAGE_NT_HEADERS, _KEVENT, _KLOCK_QUEUE_HANDLE, _KPRCB, _KTRAP_FRAME, _MMPFN, _MMPTE, _OBJECT_HEADER, _MM_SESSION_SPACE};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use super::helpers::*;

extern "system" {
    fn MmRebalanceMemoryConsumersAndWait();
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedExchange(Target: *mut LONG, Value: LONG) -> LONG;
}

const HYDRA_PROCESS: PEPROCESS = 1 as PEPROCESS;

#[cfg(feature = "MI_TRACE_PFNS")]
static mut UserPdeFault: BOOLEAN = 0;

/// Faulting PC out of a trap frame, or 0 when there is none.
///
/// The only reason this exists is the stack-runaway report below: a thread that
/// eats its whole stack is identified by WHERE it was when it did, and the trap
/// frame is the only thing at this depth that knows.
/// The sentinel several Mm paths pass INSTEAD of a trap frame, to tell the
/// fault handler they are running unlocked (mdlsup.rs, virtual.rs). It is not
/// a pointer and must never be dereferenced -- reading `.Rip` off it yields a
/// plausible-looking address, which is worse than none: it cost this probe its
/// first set of results, where 2 of 4 reported "PCs" were garbage and one was
/// page-aligned across three samples.
///
/// Spelled as a u64 cast to usize, exactly as the CALLERS spell it
/// (`0xBADBADA3BADBADA3u64 as PVOID`), so the value truncates the same way they
/// do and the comparison holds on both widths. Writing the literal directly as
/// a `usize` does not merely mismatch on i386 -- it does not COMPILE there
/// ("literal out of range for usize"), which is how the merge queue caught it
/// after a green `check`: `check` does not build.
const MI_TRAP_INFO_UNLOCKED: usize = 0xBADBADA3BADBADA3u64 as usize;

#[inline]
unsafe fn MiFaultingPc(TrapInformation: PVOID) -> ULONG_PTR {
    if TrapInformation.is_null() || TrapInformation as usize == MI_TRAP_INFO_UNLOCKED {
        return 0;
    }
    let tf = TrapInformation as PKTRAP_FRAME;
    #[cfg(target_arch = "x86_64")]
    {
        (*tf).Rip as ULONG_PTR
    }
    #[cfg(target_arch = "aarch64")]
    {
        (*tf).Pc as ULONG_PTR
    }
    #[cfg(target_arch = "x86")]
    {
        (*tf).Eip as ULONG_PTR
    }
}

/// Code selector out of a trap frame, or 0. A sane amd64 value is 0x33 (user)
/// or 0x10 (kernel); anything else convicts the offsets rather than the PC.
#[inline]
unsafe fn MiFaultingCs(TrapInformation: PVOID) -> u16 {
    if TrapInformation.is_null() || TrapInformation as usize == MI_TRAP_INFO_UNLOCKED {
        return 0;
    }
    let tf = TrapInformation as PKTRAP_FRAME;
    #[cfg(target_arch = "x86_64")]
    {
        (*tf).SegCs
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        let _ = tf;
        0
    }
}

/// Outcome of trying to grow ONE stack descriptor.
///
/// `new_limit` is the `NtTib.StackLimit` the caller must publish, in whichever
/// TEB owns the descriptor that matched — that is the whole reason this is
/// split out rather than writing the TEB inline.
struct MiStackGrowth {
    status: NTSTATUS,
    new_limit: Option<ULONG_PTR>,
}

/// Faithful `MiCheckForUserStackOverflow` body for a single [dealloc, base)
/// stack, returning `None` when `Address` is not in this stack at all so the
/// caller can try another descriptor.
unsafe fn MiGrowOneUserStack(
    Address: ULONG_PTR,
    StackBase: ULONG_PTR,
    DeallocationStack: ULONG_PTR,
) -> Option<MiStackGrowth> {
    if StackBase == 0 || DeallocationStack == 0 || DeallocationStack >= StackBase {
        return None;
    }
    if Address >= StackBase || Address < DeallocationStack {
        return None;
    }

    // GuaranteedStackBytes is asserted zero upstream; one page is the guarantee.
    let mut GuaranteedSize: SIZE_T = PAGE_SIZE as SIZE_T;
    let mut NextStackAddress: PVOID = (PAGE_ALIGN(Address) - GuaranteedSize) as PVOID;

    if ((NextStackAddress as ULONG_PTR) - PAGE_SIZE as ULONG_PTR) <= DeallocationStack {
        NextStackAddress = (PAGE_ALIGN(DeallocationStack) + GuaranteedSize) as PVOID;

        crate::dbg_print!(b"[MM] Close to our death...\n\0");
        let Status = ZwAllocateVirtualMemory(
            NtCurrentProcess(),
            &mut NextStackAddress,
            0,
            &mut GuaranteedSize as *mut SIZE_T as *mut _,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if Status < 0 {
            crate::dbg_print!(b"[MM] Failed to allocate memory: %lx\n\0", Status as u32);
        }
        return Some(MiStackGrowth {
            status: STATUS_STACK_OVERFLOW,
            new_limit: if Status >= 0 {
                Some(NextStackAddress as ULONG_PTR)
            } else {
                None
            },
        });
    }

    // ASSERT: NtGlobalFlag FLG_DISABLE_STACK_EXTENSION not set
    let PublishedLimit = (NextStackAddress as ULONG_PTR) + GuaranteedSize;

    let Status = ZwAllocateVirtualMemory(
        NtCurrentProcess(),
        &mut NextStackAddress,
        0,
        &mut GuaranteedSize as *mut SIZE_T as *mut _,
        MEM_COMMIT,
        PAGE_READWRITE | 0x100,
    );
    if (Status >= 0) || (Status == STATUS_ALREADY_COMMITTED) {
        return Some(MiStackGrowth {
            status: STATUS_PAGE_FAULT_GUARD_PAGE,
            new_limit: Some(PublishedLimit),
        });
    }

    crate::dbg_print!(b"[MM] Guard page failure: %lx\n\0", Status as u32);
    ASSERT!(false);
    Some(MiStackGrowth {
        status: STATUS_STACK_OVERFLOW,
        new_limit: Some(PublishedLimit),
    })
}

unsafe fn MiCheckForUserStackOverflow(
    Address: PVOID,
    TrapInformation: PVOID,
) -> NTSTATUS {
    let CurrentThread = PsGetCurrentThread();
    let Teb: PTEB = (*(CurrentThread as *mut _ETHREAD)).Tcb.Teb as PTEB;

    if false /* elided _ETHREAD bitfield */ {
        return STATUS_GUARD_PAGE_VIOLATION;
    }

    if KeIsAttachedProcess() != 0 {
        return STATUS_GUARD_PAGE_VIOLATION;
    }
    if Teb.is_null() {
        return STATUS_GUARD_PAGE_VIOLATION;
    }

    // Kept from the original body: the guaranteed-bytes path is unimplemented,
    // and this assert is what says so out loud. MiGrowOneUserStack hardcodes one
    // page as the guarantee, which is only correct while this holds.
    ASSERT!((*Teb).GuaranteedStackBytes == 0);

    let Addr = Address as ULONG_PTR;

    // The NATIVE stack first, and unconditionally. A WoW64 thread owns TWO
    // stacks and the 64-bit one is the busier of them -- it carries the whole
    // loader, every thunk and every unwind -- so anything that makes this test
    // conditional breaks host growth for every guest process. Redirecting it
    // wholesale to TEB32 did exactly that: 0 PASS / 16 EXITED, dying in
    // RtlpUnwindInternal, which is the 64-bit unwinder failing to get a frame.
    if let Some(g) = MiGrowOneUserStack(
        Addr,
        (*Teb).NtTib.StackBase as ULONG_PTR,
        (*Teb).DeallocationStack as ULONG_PTR,
    ) {
        if let Some(limit) = g.new_limit {
            (*Teb).NtTib.StackLimit = limit as PVOID;

            // A thread that has eaten HALF its stack reservation is a runaway
            // until proven otherwise, and by the time it faults at the bottom
            // the evidence is gone -- the PC there is whoever drew the short
            // straw, not the cause. Report the midpoint crossing instead, with
            // the faulting PC, which names the code doing the consuming.
            //
            // Stateless one-shot: growth is monotonic and page-at-a-time, so
            // the midpoint is crossed exactly once per thread.
            let base = (*Teb).NtTib.StackBase as ULONG_PTR;
            let dealloc = (*Teb).DeallocationStack as ULONG_PTR;
            if base > dealloc {
                let midpoint = dealloc + (base - dealloc) / 2;
                if limit <= midpoint && limit + PAGE_SIZE as ULONG_PTR > midpoint {
                    // TrapInformation is printed beside the PC on purpose: a
                    // trap frame lives on the KERNEL STACK, so if the "PC"
                    // tracks the frame pointer then MiFaultingPc is reading
                    // the wrong displacement and the address is furniture, not
                    // a call site. Two of the values this probe reported look
                    // exactly like that -- page-aligned and repeated -- and
                    // there is no way to tell from the PC alone.
                    crate::dbg_print!(
                        b"[MM] stack runaway: thread ate half its stack (base=%p limit=%p dealloc=%p pc=%p tf=%p cs=%lx)\n\0",
                        base,
                        limit,
                        dealloc,
                        MiFaultingPc(TrapInformation),
                        TrapInformation as usize,
                        MiFaultingCs(TrapInformation) as u32,
                    );
                }
            }
        }
        return g.status;
    }

    // Not the native stack. On a WoW64 thread the fault may still be the GUEST
    // stack, which only TEB32 describes -- TEB64's bounds never cover it, so
    // the test above rejects every guest stack-growth fault and the guest dies
    // on its first page of growth. Measured: eventvwr's worker needed the page
    // at 0x1fae000 with TEB64 covering [0x1fb0000, 0x20b0000).
    #[cfg(not(target_arch = "x86"))]
    if let Some(teb32) = ntkrnl_sys::wow64::teb32_of_current(Teb) {
        if let Some(g) = MiGrowOneUserStack(
            Addr,
            (*teb32).NtTib.StackBase as ULONG_PTR,
            (*teb32).DeallocationStack as ULONG_PTR,
        ) {
            if let Some(limit) = g.new_limit {
                (*teb32).NtTib.StackLimit = limit as u32;
            }
            return g.status;
        }
    }

    // The C prints this whenever the fault is not in the stack at all. Here it
    // belongs at the END rather than at the first bounds test, because a WoW64
    // thread has TWO descriptors and missing the native one is the NORMAL way
    // a guest stack fault reaches TEB32 -- printing there would fire on every
    // healthy guest stack growth.
    crate::dbg_print!(
        b"[MM] Faulting address outside of stack bounds. Address=%p, StackBase=%p, DeallocationStack=%p\n\0",
        Addr,
        (*Teb).NtTib.StackBase as usize,
        (*Teb).DeallocationStack as usize,
    );
    STATUS_GUARD_PAGE_VIOLATION
}

#[inline]
unsafe fn MiIsAccessAllowed(
    ProtectionMask: ULONG,
    Write: BOOLEAN,
    Execute: BOOLEAN,
) -> BOOLEAN {
    const ACCESS_ALLOWED_MASK: [[u8; 2]; 2] = [
        [0b01111110, 0b00110110],
        [0b00111000, 0b00000011],
    ];

    let prot = (ProtectionMask & MM_PROTECT_ACCESS) as usize;
    (((ACCESS_ALLOWED_MASK[(Write != 0) as usize][(Execute != 0) as usize] >> prot) & 1) != 0) as u8
}

unsafe fn MiAccessCheck(
    PointerPte: PMMPTE,
    StoreInstruction: BOOLEAN,
    PreviousMode: KPROCESSOR_MODE,
    ProtectionMask: ULONG_PTR,
    TrapFrame: PVOID,
    LockHeld: BOOLEAN,
) -> NTSTATUS {
    if (PreviousMode == 1) && (PointerPte > MiHighestUserPte) {
        return STATUS_ACCESS_VIOLATION;
    }

    let TempPte = *PointerPte;
    if (TempPte.valid() as usize) != 0 {
        if StoreInstruction != 0 {
            if MI_IS_PAGE_WRITEABLE(&TempPte) != 0 || MI_IS_PAGE_COPY_ON_WRITE(&TempPte) != 0 {
                return STATUS_SUCCESS;
            }
            return STATUS_ACCESS_VIOLATION;
        }
        return STATUS_SUCCESS;
    }

    if MiIsAccessAllowed(ProtectionMask as ULONG, StoreInstruction, 0) == 0 {
        return STATUS_ACCESS_VIOLATION;
    }

    if ((ProtectionMask as ULONG) & MM_PROTECT_SPECIAL) == MM_GUARDPAGE {
        ASSERT!((ProtectionMask as ULONG) != MM_DECOMMIT);

        if KeIsAttachedProcess() != 0 {
            return STATUS_ACCESS_VIOLATION;
        }

        ASSERT!((TempPte.prototype() as usize) == 0);

        let mut NewPte = TempPte;
        NewPte.set_soft_protection(((ProtectionMask as ULONG & !MM_GUARDPAGE) as u32) as ULONG);
        ASSERT!(NewPte.Long != 0);
        MI_WRITE_INVALID_PTE(PointerPte, NewPte);
        return STATUS_GUARD_PAGE_VIOLATION;
    }

    STATUS_SUCCESS
}

unsafe fn MiCheckVirtualAddress(
    VirtualAddress: PVOID,
    ProtectCode: *mut ULONG,
    ProtoVad: *mut PMMVAD,
) -> PMMPTE {
    *ProtoVad = core::ptr::null_mut();

    if VirtualAddress <= MM_HIGHEST_USER_ADDRESS as PVOID {
        if PAGE_ALIGN(VirtualAddress as ULONG_PTR) == MM_SHARED_USER_DATA_VA as ULONG_PTR {
            *ProtectCode = MM_READONLY;
            return MmSharedUserDataPte;
        }

        let Vad = MiLocateAddress(VirtualAddress);
        if Vad.is_null() {
            *ProtectCode = MM_NOACCESS;
            return core::ptr::null_mut();
        }

        ASSERT!((*Vad).u.VadFlags.VadType() != 2);

        if (*Vad).u.VadFlags.PrivateMemory() != 0 {
            ASSERT!((*Vad).u.VadFlags.VadType() != 3);

            if (*Vad).u.VadFlags.MemCommit() != 0 {
                *ProtectCode = (*Vad).u.VadFlags.Protection() as ULONG;
            } else {
                *ProtectCode = MM_NOACCESS;
            }
            return core::ptr::null_mut();
        } else {
            ASSERT!((*Vad).u.VadFlags.VadType() != 0);
            ASSERT!((*Vad).u2.VadFlags2.ExtendableFile() == 0);

            *ProtoVad = Vad;

            // C: MiGetProtoPteAddress(Vad, MI_VA_TO_VPN(VirtualAddress))
            // = (char *)FirstPrototypePte + ((VPN - StartingVpn) << PTE_SHIFT)
            let page_offset = ((VirtualAddress as ULONG_PTR) >> PAGE_SHIFT) - (*Vad).StartingVpn as ULONG_PTR;
            let PointerPte = ((*Vad).FirstPrototypePte as PMMPTE).add(page_offset);
            ASSERT!(!PointerPte.is_null());
            ASSERT!(PointerPte <= (*Vad).LastContiguousPte as *mut MMPTE);

            *ProtectCode = (*Vad).u.VadFlags.Protection() as ULONG;
            return PointerPte;
        }
    } else if MI_IS_PAGE_TABLE_ADDRESS(VirtualAddress) {
        if ((VirtualAddress as PMMPTE) >= MiAddressToPte(MmPagedPoolStart))
            && ((VirtualAddress as PMMPTE) <= (*core::ptr::addr_of!(MmPagedPoolInfo)).LastPteForPagedPool as *mut MMPTE)
        {
            *ProtectCode = MM_NOACCESS;
            return core::ptr::null_mut();
        }

        *ProtectCode = MM_EXECUTE_READWRITE;
        return core::ptr::null_mut();
    } else if MI_IS_SESSION_ADDRESS(VirtualAddress) {
        // ASSERT!(IsListEmpty(core::ptr::addr_of_mut!((*MmSessionSpace).ImageList) as *mut LIST_ENTRY)); // elided (MmSessionSpace typed as PVOID)
    }

    *ProtectCode = MM_NOACCESS;
    core::ptr::null_mut()
}

#[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
unsafe fn MiCheckPdeForSessionSpace(Address: PVOID) -> NTSTATUS {
    let mut TempPde: MMPTE;
    let mut PointerPde: PMMPTE;
    let SessionAddress: PVOID;
    let Index: ULONG;

    if MI_IS_SESSION_PTE(Address as PMMPTE) {
        PointerPde = MiAddressToPde(MmSessionSpace as PVOID);
        if ((*PointerPde).valid() as usize) == 0 {
            DbgBreakPoint();
            return STATUS_ACCESS_VIOLATION;
        }

        SessionAddress = MiPteToAddress(Address as PMMPTE);
        PointerPde = MiAddressToPte(Address) as PMMPDE;
        if ((*PointerPde).valid() as usize) != 0 {
            return STATUS_WAIT_1;
        }

        Index = (((SessionAddress as ULONG_PTR) - (MmSessionBase as ULONG_PTR)) >> 22) as ULONG;
        TempPde = core::mem::transmute(*(*(MmSessionSpace as *mut _MM_SESSION_SPACE)).PageTables.add(Index as usize));
        if (TempPde.valid() as usize) != 0 {
            InterlockedExchange(
                core::ptr::addr_of_mut!((*PointerPde).Long) as *mut LONG,
                TempPde.Long as LONG,
            );
            return STATUS_WAIT_1;
        }

        DbgBreakPoint();
        return STATUS_ACCESS_VIOLATION;
    }

    if !MI_IS_SESSION_ADDRESS(Address) {
        return STATUS_SUCCESS;
    }

    PointerPde = MiAddressToPde(MmSessionSpace as PVOID);
    if ((*PointerPde).valid() as usize) == 0 {
        DbgBreakPoint();
        return STATUS_ACCESS_VIOLATION;
    }

    PointerPde = MiAddressToPde(Address);
    if ((*PointerPde).valid() as usize) == 0 {
        Index = (((Address as ULONG_PTR) - (MmSessionBase as ULONG_PTR)) >> 22) as ULONG;
        *PointerPde = core::mem::transmute(*(*(MmSessionSpace as *mut _MM_SESSION_SPACE)).PageTables.add(Index as usize));
        if ((*PointerPde).valid() as usize) != 0 {
            return STATUS_WAIT_1;
        }

        DbgBreakPoint();
        return STATUS_ACCESS_VIOLATION;
    }

    STATUS_SUCCESS
}

#[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn MiCheckPdeForPagedPool(Address: PVOID) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    let PointerPde: PMMPTE;
    let mut Status = STATUS_SUCCESS;

    if MI_IS_SESSION_ADDRESS(Address) {
        return MiCheckPdeForSessionSpace(Address);
    }
    if MI_IS_SESSION_PTE(Address as PMMPTE) {
        return MiCheckPdeForSessionSpace(Address);
    }

    if MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address) {
        PointerPde = MiAddressToPte(Address) as PMMPDE;
        Status = STATUS_WAIT_1;
    } else if Address < MmSystemRangeStart {
        return STATUS_ACCESS_VIOLATION;
    } else {
        PointerPde = MiAddressToPde(Address);
    }

    if ((*PointerPde).valid() as usize) == 0 {
        let idx = ((PointerPde as ULONG_PTR) & (SYSTEM_PD_SIZE as ULONG_PTR - 1)) / core::mem::size_of::<MMPTE>();
        let system_pte = (*MmSystemPagePtes.add(idx)).Long;
        InterlockedExchange(
            core::ptr::addr_of_mut!((*PointerPde).Long) as *mut LONG,
            system_pte as LONG,
        );
    }

    Status
}

#[cfg(not(all(target_arch = "x86", not(target_pointer_width = "64"))))]
#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn MiCheckPdeForPagedPool(Address: PVOID) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    STATUS_ACCESS_VIOLATION
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MiCheckPdeForPagedPool(Address: PVOID) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    STATUS_ACCESS_VIOLATION
}

unsafe fn MiZeroPfn(PageFrameNumber: PFN_NUMBER) {
    let ZeroPte: PMMPTE;
    let mut TempPte: MMPTE;
    let Pfn1: PMMPFN;
    let ZeroAddress: PVOID;

    Pfn1 = MiGetPfnEntry(PageFrameNumber);
    ASSERT!(!Pfn1.is_null());

    ZeroPte = MiReserveSystemPtes(1, SystemPteSpace);
    ASSERT!(!ZeroPte.is_null());

    TempPte = ValidKernelPte;
    TempPte.set_page_frame_number((PageFrameNumber as usize) as PFN_NUMBER);

    if (*Pfn1).cache_attribute() == (MiWriteCombined) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_COMBINED(&mut TempPte);
    } else if (*Pfn1).cache_attribute() == (MiNonCached) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_THROUGH(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(ZeroPte, TempPte);

    ZeroAddress = MiPteToAddress(ZeroPte);
    KeZeroPages(ZeroAddress, (PAGE_SIZE as usize) as u32);

    MiReleaseSystemPtes(ZeroPte, 1, SystemPteSpace);
}

unsafe fn MiCopyPfn(DestPage: PFN_NUMBER, SrcPage: PFN_NUMBER) {
    let SysPtes: PMMPTE;
    let mut TempPte: MMPTE;
    let DestPfn: PMMPFN;
    let SrcPfn: PMMPFN;
    let DestAddress: PVOID;
    let SrcAddress: PVOID;

    DestPfn = MiGetPfnEntry(DestPage);
    ASSERT!(!DestPfn.is_null());
    SrcPfn = MiGetPfnEntry(SrcPage);
    ASSERT!(!SrcPfn.is_null());

    SysPtes = MiReserveSystemPtes(2, SystemPteSpace);
    ASSERT!(!SysPtes.is_null());

    TempPte = ValidKernelPte;
    TempPte.set_page_frame_number((DestPage as usize) as PFN_NUMBER);

    if (*DestPfn).cache_attribute() == (MiWriteCombined) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_COMBINED(&mut TempPte);
    } else if (*DestPfn).cache_attribute() == (MiNonCached) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_THROUGH(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(&mut (*SysPtes.add(0)), TempPte);

    TempPte = ValidKernelPte;
    TempPte.set_page_frame_number((SrcPage as usize) as PFN_NUMBER);

    if (*SrcPfn).cache_attribute() == (MiNonCached) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(&mut (*SysPtes.add(1)), TempPte);

    DestAddress = MiPteToAddress(&mut (*SysPtes.add(0)));
    SrcAddress = MiPteToAddress(&mut (*SysPtes.add(1)));
    RtlCopyMemory(DestAddress, SrcAddress, PAGE_SIZE as SIZE_T);

    MiReleaseSystemPtes(SysPtes, 2, SystemPteSpace);
}

#[track_caller]
unsafe fn MiResolveDemandZeroFault(
    Address: PVOID,
    PointerPte: PMMPTE,
    Protection: ULONG,
    Process: PEPROCESS,
    mut OldIrql: KIRQL,
) -> NTSTATUS {
    // #39: a Protection with no access bits reaching here becomes a PTE the
    // arm64 MMU rejects while software reads it valid (see
    // ntkrnl_sys::mm::mi_user_pte_may_be_valid). The guard downstream refuses
    // it, but the interesting question is who asked. Of the six live callers
    // only two pass a value that can be zero -- both pass
    // TempPte.soft_protection() -- so name the caller and dump the software PTE
    // it read it out of. One line per boot; this is a hot path.
    if Protection & (ntkrnl_sys::mm::MM_PROTECT_ACCESS as ULONG) == 0 {
        static mut DZ_REPORTED: bool = false;
        if !DZ_REPORTED {
            DZ_REPORTED = true;
            let loc = core::panic::Location::caller();
            let mut buf = [0u8; 64];
            let src = loc.file().as_bytes();
            let take = if src.len() > 63 { 63 } else { src.len() };
            let start = src.len() - take;
            let mut i = 0;
            while i < take {
                buf[i] = src[start + i];
                i += 1;
            }
            buf[take] = 0;
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[DZPROT] demand-zero with no-access Protection=%u from %s:%u                   Address=%p Pte=%p PteLong=0x%016llX\n\0".as_ptr() as *const i8,
                Protection,
                buf.as_ptr() as *const i8,
                loc.line(),
                Address,
                PointerPte,
                (*PointerPte).Long as u64,
            );
        }
    }
    let mut PageFrameNumber: PFN_NUMBER = 0;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut NeedZero = false;
    let mut HaveLock = false;
    let mut Color: ULONG;
    let Pfn1: PMMPFN;

    if (Process > HYDRA_PROCESS) && (OldIrql == MM_NOIRQL) {
        ASSERT!(MI_IS_PAGE_TABLE_ADDRESS(PointerPte as PVOID));
        ASSERT!((*(Process as *mut _EPROCESS)).ForkInProgress.is_null());

        Color = MI_GET_NEXT_PROCESS_COLOR(Process as *mut _EPROCESS as *mut c_void);
        ASSERT!(Color != 0xFFFFFFFF);
        NeedZero = true;
    } else {
        NeedZero = (OldIrql != MM_NOIRQL);

        if (Process == HYDRA_PROCESS)
            && ((MI_IS_SESSION_IMAGE_ADDRESS(Address))
                || ((Address >= MiSessionViewStart) && (Address < MiSessionSpaceWs)))
        {
            NeedZero = true;
        }

        Color = 0xFFFFFFFF;
    }

    if OldIrql == MM_NOIRQL {
        let irql = MiAcquirePfnLock();
        HaveLock = true;
        OldIrql = irql;
    }

    MI_ASSERT_PFN_LOCK_HELD();
    ASSERT!(((*PointerPte).valid() as usize) == 0);

    #[cfg(feature = "MI_TRACE_PFNS")]
    {
        if UserPdeFault != 0 {
            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_TABLE);
        }
        if UserPdeFault == 0 {
            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_DEMAND_ZERO);
        }
    }
    if Process == HYDRA_PROCESS {
        ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
    } else if !Process.is_null() {
        ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
    } else {
        ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
    }

    if Color != 0xFFFFFFFF {
        PageFrameNumber = (MiRemoveZeroPageSafe(Color)) as usize;
        if PageFrameNumber == 0 {
            PageFrameNumber = MiRemoveAnyPage(Color);
            NeedZero = true;
        }
    } else {
        Color = MI_GET_NEXT_COLOR();
        if !NeedZero {
            PageFrameNumber = MiRemoveAnyPage(Color);
        } else {
            PageFrameNumber = MiRemoveZeroPage(Color);
        }
    }

    if PageFrameNumber == 0 {
        MiReleasePfnLock(OldIrql);
        return STATUS_NO_MEMORY;
    }

    MiInitializePfn((PageFrameNumber) as _, PointerPte as *mut _MMPTE, 1);

    /* MmDemandZeroCount (no-op) */;

    if HaveLock {
        MiReleasePfnLock(OldIrql);

        if Process > HYDRA_PROCESS {
            (*(Process as *mut _EPROCESS)).NumberOfPrivatePages += 1;
        }
    }

    if NeedZero {
        MiZeroPfn(PageFrameNumber);
    }

    if PointerPte <= MiHighestUserPte {
        MI_MAKE_HARDWARE_PTE_USER(&mut TempPte, PointerPte, (Protection) as usize, PageFrameNumber);
    } else {
        MI_MAKE_HARDWARE_PTE(&mut TempPte, PointerPte, (Protection) as usize, PageFrameNumber);
    }

    if MI_IS_PAGE_WRITEABLE(&TempPte) != 0 {
        MI_MAKE_DIRTY_PAGE(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(PointerPte, TempPte);

    if HaveLock {
        Pfn1 = MI_PFN_ELEMENT(PageFrameNumber);
        ASSERT!((*Pfn1).flink() == 0);
        ASSERT!(((*Pfn1).prototype_pte() as u16) != 0);
    }

    STATUS_PAGE_FAULT_DEMAND_ZERO
}

unsafe fn MiCompleteProtoPteFault(
    mut StoreInstruction: BOOLEAN,
    Address: PVOID,
    PointerPte: PMMPTE,
    PointerProtoPte: PMMPTE,
    OldIrql: KIRQL,
    LockedProtoPfn: *mut PMMPFN,
) -> NTSTATUS {
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let OriginalPte: PMMPTE;
    let PageTablePte: PMMPTE;
    let mut Protection: ULONG_PTR;
    let PageFrameIndex: PFN_NUMBER;
    let Pfn1: PMMPFN;
    let Pfn2: PMMPFN;
    let mut OriginalProtection: BOOLEAN;
    let mut DirtyPage: BOOLEAN;

    MI_ASSERT_PFN_LOCK_HELD();
    ASSERT!(((*PointerProtoPte).valid() as usize) == 1);

    PageFrameIndex = PFN_FROM_PTE(PointerProtoPte);

    Pfn1 = MiGetPfnEntry(PageFrameIndex);
    (*Pfn1).set_prototype_pte((1) != 0);

    PageTablePte = MiAddressToPte(PointerPte as PVOID);
    Pfn2 = MiGetPfnEntry((*PageTablePte).page_frame_number() as PFN_NUMBER);
    (*Pfn2).set_share_count((*Pfn2).share_count().wrapping_add((1) as ULONG_PTR));

    if (*PointerPte).soft_page_file_high() == MI_PTE_LOOKUP_NEEDED {
        Protection = (*PointerPte).soft_protection() as ULONG_PTR;
        OriginalProtection = (false) as u8;
    } else {
        OriginalPte = mmpfn_original_pte_ptr(Pfn1);
        OriginalProtection = (true) as u8;

        if MI_IS_ROS_PFN(Pfn1) {
            // ROS PFN (MmAllocPage — legacy/ROS section pages, e.g. the win32k GDI handle
            // table section): the OriginalPte union slot holds RmapListHead, NOT a PTE —
            // never read protection from it and never write it (a write would corrupt a
            // live rmap chain). Derive the real protection from the authoritative (valid)
            // prototype PTE instead. Without this the kernel session view of a read-write
            // ROS section maps read-only (garbage/zero protection bits) and the first
            // write BUGCHECKs 0xBE (win32k GdiHandleTable->DevCaps).
            // (field-manual §9.13 — devcaps-0xBE)
            Protection = if MI_IS_PAGE_WRITEABLE(&*PointerProtoPte) != 0 {
                MM_READWRITE as ULONG_PTR
            } else {
                MM_READONLY as ULONG_PTR
            };
        } else {
            Protection = (*OriginalPte).soft_protection() as ULONG_PTR;

            // Defensive heal for a genuine ARM3 PFN whose OriginalPte was never
            // initialized (protection reads MM_ZERO_ACCESS, Long stays 0 from the
            // free-list link) while its prototype PTE is validly mapped: derive the real
            // protection from the proto PTE and write it back so every later consumer
            // (MiDecrementShareCount transition-PTE build, re-fault, the valid-write RO
            // check) sees the section's true protection. Gated on kernel addresses so
            // user views are untouched. (field-manual §9.13 — devcaps-0xBE)
            if (Protection as ULONG) == MM_ZERO_ACCESS
                && (Address as u64) >= (MmSystemRangeStart as u64)
                && ((*PointerProtoPte).valid() as usize) == 1
            {
                Protection = if MI_IS_PAGE_WRITEABLE(&*PointerProtoPte) != 0 {
                    MM_READWRITE as ULONG_PTR
                } else {
                    MM_READONLY as ULONG_PTR
                };
                (*OriginalPte).set_soft_protection(Protection as ULONG);
            }
        }

        if (StoreInstruction != 0) && !((Protection as ULONG) & MM_READWRITE != 0) {
            StoreInstruction = 0;
        }
    }

    DirtyPage = (false) as u8;
    if (StoreInstruction != 0) && ((Protection as ULONG & MM_WRITECOPY) != MM_WRITECOPY) {
        DirtyPage = (true) as u8;
        let original_pte = mmpfn_original_pte(Pfn1);
        ASSERT!(mmpte_software_prototype(&original_pte));
    }

    if !(*LockedProtoPfn).is_null() {
        ASSERT!((**LockedProtoPfn).reference_count() >= 1);
        MiDereferencePfnAndDropLockCount(*LockedProtoPfn as *mut _MMPFN);
        *LockedProtoPfn = core::ptr::null_mut();
    }

    MiReleasePfnLock(OldIrql);

    Protection &= !(MM_PROTECT_SPECIAL as ULONG_PTR);

    if (*Pfn1).cache_attribute() == (MiWriteCombined) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_COMBINED(&mut TempPte);
    } else if (*Pfn1).cache_attribute() == (MiNonCached) as u16 {
        MI_PAGE_DISABLE_CACHE(&mut TempPte);
        MI_PAGE_WRITE_THROUGH(&mut TempPte);
    }

    if Address < MmSystemRangeStart {
        MI_MAKE_HARDWARE_PTE_USER(&mut TempPte, PointerPte, (Protection as ULONG) as usize, PageFrameIndex);
    } else {
        MI_MAKE_HARDWARE_PTE(&mut TempPte, PointerPte, (Protection as ULONG) as usize, PageFrameIndex);
    }

    if (DirtyPage) != 0 {
        MI_MAKE_DIRTY_PAGE(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(PointerPte, TempPte);

    if (OriginalProtection) != 0 {
        Protection = MM_ZERO_ACCESS as ULONG_PTR;
    }

    ASSERT!(PointerPte == MiAddressToPte(Address));
    STATUS_SUCCESS
}

unsafe fn MiResolvePageFileFault(
    StoreInstruction: BOOLEAN,
    FaultingAddress: PVOID,
    PointerPte: PMMPTE,
    CurrentProcess: PEPROCESS,
    OldIrql: *mut KIRQL,
) -> NTSTATUS {
    let mut Color: ULONG;
    let mut Page: PFN_NUMBER;
    let mut Status: NTSTATUS;
    let TempPte = *PointerPte;
    let Pfn1: PMMPFN;
    let PageFileIndex = TempPte.soft_protection();
    let PageFileOffset = TempPte.soft_page_file_high() as ULONG_PTR;
    let Protection = TempPte.soft_protection();

    ASSERT!(CurrentProcess > HYDRA_PROCESS);
    ASSERT!(*OldIrql != MM_NOIRQL);

    ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PAGE_FILE);
    ntkrnl_sys::MI_SET_PROCESS!(CurrentProcess as *mut c_void);

    MI_ASSERT_PFN_LOCK_HELD();

    ASSERT!((TempPte.valid() as usize) == 0);
    ASSERT!(TempPte.soft_page_file_high() != 0);
    ASSERT!(TempPte.soft_page_file_high() != MI_PTE_LOOKUP_NEEDED);

    Color = MI_GET_NEXT_PROCESS_COLOR(CurrentProcess as *mut _EPROCESS as *mut c_void);
    Page = MiRemoveAnyPage(Color);
    if Page == 0 {
        return STATUS_NO_MEMORY;
    }

    MiInitializePfn((Page) as _, PointerPte as *mut _MMPTE, StoreInstruction as BOOLEAN);

    Pfn1 = MI_PFN_ELEMENT(Page);
    ASSERT!((*Pfn1).flink() == 0);
    ASSERT!(((*Pfn1).start_of_allocation() as u16) == 0);
    ASSERT!(((*Pfn1).end_of_allocation() as u16) == 0);
    (*Pfn1).set_start_of_allocation((1) != 0);

    let mut NewPte: MMPTE = MMPTE { Long: 0 };
    MI_MAKE_TRANSITION_PTE(&mut NewPte, Page, Protection);
    MI_WRITE_INVALID_PTE(PointerPte, NewPte);

    MiReleasePfnLock(*OldIrql);

    Status = MiReadPageFile(Page as _, PageFileIndex, PageFileOffset as _);

    *OldIrql = MiAcquirePfnLock();

    ASSERT!(((*Pfn1).start_of_allocation() as u16) == 1);
    ASSERT!(((*Pfn1).end_of_allocation() as u16) == 0);

    if Status < 0 {
        ASSERT!(false);
        mmpfn_set_in_page_error_u4(Pfn1, (1) != 0);
        (*Pfn1).set_flink(Status as PFN_NUMBER);
    }

    MI_MAKE_HARDWARE_PTE(&mut NewPte, PointerPte, (Protection) as usize, Page);
    MI_WRITE_VALID_PTE(PointerPte, NewPte);

    (*Pfn1).set_start_of_allocation((0) != 0);
    if !((*Pfn1).flink() as PVOID).is_null() {
        KeSetEvent(((*Pfn1).flink() as PVOID), 0, 0);
    }

    Status
}

unsafe fn MiResolveTransitionFault(
    mut StoreInstruction: BOOLEAN,
    FaultingAddress: PVOID,
    PointerPte: PMMPTE,
    CurrentProcess: PEPROCESS,
    OldIrql: KIRQL,
    InPageBlock: *mut *mut PKEVENT,
) -> NTSTATUS {
    let PageFrameIndex: PFN_NUMBER;
    let Pfn1: PMMPFN;
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let PointerToPteForProtoPage: PMMPTE;

    ASSERT!((*InPageBlock).is_null());

    ASSERT!(OldIrql != MM_NOIRQL);

    TempPte = *PointerPte;
    ASSERT!(((TempPte.valid() as usize) == 0) && ((TempPte.prototype() as usize) == 0) && ((TempPte.transition() as usize) == 1));

    PageFrameIndex = TempPte.page_frame_number() as PFN_NUMBER;
    Pfn1 = MiGetPfnEntry(PageFrameIndex);

    // DIAGNOSTIC: Catch NULL PFN entry (PageFrameIndex out of range). Every arch: the
    // next line dereferences Pfn1, so the alternative is an anonymous NULL fault.
    {
        if Pfn1.is_null() {
            KeBugCheckEx( // nolint: bugcheck-code (0xDEAD0040 = deliberate NULL-PFN diagnostic marker, not a stop code)
                0xDEAD0040,
                FaultingAddress as ULONG_PTR,  // P1: what VA caused the fault
                PointerPte as ULONG_PTR,       // P2: PTE address (self-map vs proto)
                TempPte.Long as ULONG_PTR,     // P3: PTE contents (garbage)
                PageFrameIndex as ULONG_PTR,   // P4: extracted bogus PFN
            );
        }
    }

    // MmTransitionCount increment elided (not in bindgen KPRCB)

    ASSERT!((mmpfn_in_page_error_u4(Pfn1) as u16) == 0);

    if ((*Pfn1).start_of_allocation() as u16) == 1 || (((*Pfn1).end_of_allocation() as u16) == 1 && StoreInstruction != 0) {
        *InPageBlock = (*Pfn1).flink() as *mut PKEVENT;
        if PointerPte == (*Pfn1).pte_address() {
            return STATUS_SUCCESS;
        }
    }

    ASSERT!(MmAvailablePages > 0);
    ASSERT!((mmpfn_in_page_error_u4(Pfn1) as u16) == 0);

    ASSERT!(MmAvailablePages > 32);

    if (*Pfn1).page_location() == ActiveAndValid {
        ASSERT!((*Pfn1).pte_address() >= MiAddressToPte(MmPagedPoolStart)
            && (*Pfn1).pte_address() <= MiAddressToPte(MmPagedPoolEnd));
        ASSERT!((*Pfn1).share_count() != 0);
        ASSERT!((*Pfn1).reference_count() != 0);
    } else {
        MiUnlinkPageFromList(Pfn1 as *mut _MMPFN);
        MiReferenceUnusedPageAndBumpLockCount(Pfn1 as *mut _MMPFN);
    }

    ASSERT!((mmpfn_in_page_error_u4(Pfn1) as u16) == 0);

    if (*Pfn1).share_count() == 0 {
        MiDropLockCount(Pfn1 as *mut _MMPFN);
    }

    (*Pfn1).set_share_count((*Pfn1).share_count().wrapping_add((1) as ULONG_PTR));
    (*Pfn1).set_page_location((ActiveAndValid) as u16);

    if FaultingAddress >= MmSystemRangeStart {
        PointerToPteForProtoPage = MiAddressToPte(PointerPte as PVOID);

        // DIAGNOSTIC: Verify self-map is accessible before reading through it. Both
        // 4-level arches (the PXE lesson: arm64 is 4-level too); i386 has no PXE/PPE.
        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
        {
            let DiagPde = MiAddressToPde(PointerToPteForProtoPage as PVOID);
            let DiagPpe = MiAddressToPpe(PointerToPteForProtoPage as PVOID);
            let DiagPxe = MiAddressToPxe(PointerToPteForProtoPage as PVOID);
            if ((*DiagPxe).valid() as usize) == 0
                || ((*DiagPpe).valid() as usize) == 0
                || ((*DiagPde).valid() as usize) == 0
            {
                KeBugCheckEx( // nolint: bugcheck-code (0xDEAD0030 = deliberate proto-PTE diagnostic marker, not a stop code)
                    0xDEAD0030,
                    PointerToPteForProtoPage as ULONG_PTR,
                    PointerPte as ULONG_PTR,
                    FaultingAddress as ULONG_PTR,
                    (((*DiagPxe).valid() as ULONG_PTR) << 16)
                        | (((*DiagPpe).valid() as ULONG_PTR) << 8)
                        | ((*DiagPde).valid() as ULONG_PTR),
                );
            }
        }

        TempPte = *PointerToPteForProtoPage;
        if ((TempPte.valid() as usize) == 0) && ((TempPte.transition() as usize) == 1) {
            ASSERT!(false);
        }
    }

    ASSERT!(((*PointerPte).valid() as usize) == 0);
    ASSERT!((*PointerPte).prototype() == false);
    ASSERT!((*PointerPte).transition() == true);
    TempPte.Long = ((*PointerPte).Long & !0xFFF)
        | MmProtectToPteMask[(*PointerPte).soft_protection() as usize]
        | (MiDetermineUserGlobalPteMask(PointerPte as *mut c_void) as usize);

    if (((*Pfn1).modified() as u16) != 0)
        && (MI_IS_PAGE_WRITEABLE(&TempPte) != 0)
        && (MI_IS_PAGE_COPY_ON_WRITE(&TempPte) == 0)
    {
        MI_MAKE_DIRTY_PAGE(&mut TempPte);
    } else {
        MI_MAKE_CLEAN_PAGE(&mut TempPte);
    }

    MI_WRITE_VALID_PTE(PointerPte, TempPte);

    STATUS_PAGE_FAULT_TRANSITION
}

unsafe fn MiResolveProtoPteFault(
    StoreInstruction: BOOLEAN,
    Address: PVOID,
    PointerPte: PMMPTE,
    PointerProtoPte: PMMPTE,
    OutPfn: *mut PMMPFN,
    PageFileData: *mut PVOID,
    PteValue: PMMPTE,
    Process: PEPROCESS,
    mut OldIrql: KIRQL,
    TrapInformation: PVOID,
) -> NTSTATUS {
    let mut TempPte: MMPTE = MMPTE { Long: 0 };
    let mut PteContents: MMPTE;
    let Pfn1: PMMPFN;
    let mut PageFrameIndex: PFN_NUMBER;
    let mut Status: NTSTATUS;
    let mut InPageBlock: *mut PKEVENT = core::ptr::null_mut();
    let mut Protection: ULONG;

    MI_ASSERT_PFN_LOCK_HELD();
    ASSERT!(((*PointerPte).valid() as usize) == 0);
    ASSERT!(((*PointerPte).prototype() as usize) == 1);

    TempPte = *PointerProtoPte;
    if (TempPte.valid() as usize) == 1 {
        PageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
        Pfn1 = MiGetPfnEntry(PageFrameIndex);
        (*Pfn1).set_share_count((*Pfn1).share_count().wrapping_add((1) as ULONG_PTR));

        // MmTransitionCount increment elided (not in bindgen KPRCB)

        return MiCompleteProtoPteFault(StoreInstruction, Address, PointerPte, PointerProtoPte, OldIrql, OutPfn);
    }

    if TempPte.Long == 0 {
        MiReleasePfnLock(OldIrql);
        return STATUS_ACCESS_VIOLATION;
    }

    ASSERT!(TempPte.Long != (*core::ptr::addr_of!(MmDecommittedPte)).Long);

    PteContents = *PointerPte;
    if PteContents.soft_page_file_high() != MI_PTE_LOOKUP_NEEDED {
        if !PteContents.proto_read_only() {
            Protection = TempPte.soft_protection();
        } else {
            Protection = MM_READONLY;
        }
        Status = MiAccessCheck(
            PointerProtoPte,
            StoreInstruction,
            0,
            TempPte.soft_protection() as ULONG_PTR,
            TrapInformation,
            1,
        );
        ASSERT!(Status == STATUS_SUCCESS);
    } else {
        Protection = PteContents.soft_protection();
    }

    if ((Protection & MM_WRITECOPY) == MM_WRITECOPY) && (StoreInstruction != 0) {
        let mut ProtoPageFrameIndex: PFN_NUMBER;

        Status = MiResolveProtoPteFault(
            0,
            Address,
            PointerPte,
            PointerProtoPte,
            OutPfn,
            PageFileData,
            PteValue,
            Process,
            OldIrql,
            TrapInformation,
        );

        if Status < 0 {
            return Status;
        }

        OldIrql = MiAcquirePfnLock();

        TempPte = *PointerProtoPte;
        ASSERT!((TempPte.valid() as usize) == 1);
        ProtoPageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);

        ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_COW);
        ntkrnl_sys::MI_SET_PROCESS!(Process as *mut c_void);

        let Color: ULONG;
        if Process > HYDRA_PROCESS {
            Color = MI_GET_NEXT_PROCESS_COLOR(Process as *mut _EPROCESS as *mut c_void);
        } else {
            Color = MI_GET_NEXT_COLOR();
        }

        PageFrameIndex = MiRemoveAnyPage(Color);
        if PageFrameIndex == 0 {
            MiReleasePfnLock(OldIrql);
            return STATUS_NO_MEMORY;
        }

        MiCopyPfn(PageFrameIndex, ProtoPageFrameIndex);

        MiDeletePte(PointerPte as *mut _MMPTE, Address, Process, PointerProtoPte as *mut _MMPTE);

        Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
        MiInitializePfn((PageFrameIndex) as _, PointerPte as *mut _MMPTE, 1);

        Protection &= !MM_WRITECOPY;
        Protection |= MM_READWRITE;
        if Address < MmSystemRangeStart {
            MI_MAKE_HARDWARE_PTE_USER(&mut PteContents, PointerPte, (Protection) as usize, PageFrameIndex);
        } else {
            MI_MAKE_HARDWARE_PTE(&mut PteContents, PointerPte, (Protection) as usize, PageFrameIndex);
        }

        MI_WRITE_VALID_PTE(PointerPte, PteContents);

        MiReleasePfnLock(OldIrql);
        return Status;
    }

    if PointerPte <= MiHighestUserPte {
        ASSERT!((*(Process as *mut _EPROCESS)).CloneRoot.is_null());
    }

    ASSERT!((TempPte.prototype() as usize) == 0);

    if (TempPte.prototype() as usize) == 0 && (TempPte.transition() as usize) == 1 {
        ASSERT!(OldIrql != MM_NOIRQL);
        Status = MiResolveTransitionFault(
            StoreInstruction,
            Address,
            PointerProtoPte,
            Process,
            OldIrql,
            &mut InPageBlock,
        );
        ASSERT!(Status >= 0);
    } else {
        ASSERT!(TempPte.soft_page_file_high() == 0);

        Status = MiResolveDemandZeroFault(
            Address,
            PointerProtoPte,
            TempPte.soft_protection() as ULONG,
            Process,
            OldIrql,
        );
        #[cfg(feature = "MI_TRACE_PFNS")]
        {
            if !TrapInformation.is_null() {
                MiGetPfnEntry((*PointerProtoPte).page_frame_number() as PFN_NUMBER).CallSite =
                    (*(TrapInformation as PKTRAP_FRAME)).Eip as PVOID;
            }
        }

        ASSERT!(Status >= 0);
    }

    ASSERT!(((*PointerPte).valid() as usize) == 0);
    MiCompleteProtoPteFault(StoreInstruction, Address, PointerPte, PointerProtoPte, OldIrql, OutPfn)
}

unsafe fn MiDispatchFault(
    FaultCode: ULONG,
    Address: PVOID,
    PointerPte: PMMPTE,
    PointerProtoPte: PMMPTE,
    Recursive: BOOLEAN,
    Process: PEPROCESS,
    TrapInformation: PVOID,
    Vad: PMMVAD,
) -> NTSTATUS {
    let mut TempPte: MMPTE;
    let mut OldIrql: KIRQL;
    let mut LockIrql: KIRQL;
    let mut Status: NTSTATUS;
    let SuperProtoPte: PMMPTE;
    let mut Pfn1: PMMPFN;
    let mut OutPfn: PMMPFN = core::ptr::null_mut();
    let mut PageFrameIndex: PFN_NUMBER;
    let mut PteCount: PFN_COUNT;
    let mut ProcessedPtes: PFN_COUNT;

    ASSERT!(PointerPte == MiAddressToPte(Address));

    OldIrql = KeGetCurrentIrql();
    ASSERT!(OldIrql <= APC_LEVEL);
    ASSERT!(KeAreAllApcsDisabled() != 0);

    TempPte = *PointerPte;

    if !PointerProtoPte.is_null() {
        ASSERT!(MI_IS_PHYSICAL_ADDRESS(PointerProtoPte as PVOID) == 0);

        SuperProtoPte = MiAddressToPte(PointerProtoPte as PVOID);
        if Address >= MmSystemRangeStart {
            LockIrql = MiAcquirePfnLock();

            if ((*SuperProtoPte).valid() as usize) == 0 {
                ASSERT!(false);
            } else if ((*PointerPte).valid() as usize) == 1 {
                ASSERT!(false);
            }

            Status = MiResolveProtoPteFault(
                ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
                Address,
                PointerPte,
                PointerProtoPte,
                &mut OutPfn,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                Process,
                LockIrql,
                TrapInformation,
            );
            ASSERT!(Status == STATUS_SUCCESS);

            ASSERT!(OldIrql == KeGetCurrentIrql());
            ASSERT!(OldIrql <= APC_LEVEL);
            ASSERT!(KeAreAllApcsDisabled() != 0);
            return Status;
        } else {
            ASSERT!((*PointerPte).soft_page_file_high() == MI_PTE_LOOKUP_NEEDED as ULONG_PTR);

            if (!Vad.is_null())
                && ((*Vad).u.VadFlags.VadType() != 0)
                && ((*Vad).u2.VadFlags2.ExtendableFile() == 0)
            {
                ASSERT!(Address <= MM_HIGHEST_USER_ADDRESS as PVOID);
            }

            PteCount = 1;
            ProcessedPtes = 0;

            LockIrql = MiAcquirePfnLock();

            ASSERT!(((*SuperProtoPte).valid() as usize) == 1);

            TempPte = *PointerProtoPte;

            loop {
                if (TempPte.valid() as usize) == 1 {
                    PageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
                    Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
                    (*Pfn1).set_share_count((*Pfn1).share_count().wrapping_add((1) as ULONG_PTR));
                } else if ((TempPte.prototype() as usize) == 0) && ((TempPte.transition() as usize) == 1) {
                    PageFrameIndex = TempPte.page_frame_number() as PFN_NUMBER;
                    Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
                    ASSERT!((*Pfn1).page_location() != ActiveAndValid);

                    ASSERT!(((*Pfn1).start_of_allocation() as u16) == 0);
                    ASSERT!((mmpfn_in_page_error_u4(Pfn1) as u16) == 0);

                    MiUnlinkPageFromList(Pfn1 as *mut _MMPFN);

                    ASSERT!((*Pfn1).share_count() == 0 as ULONG_PTR);
                    mmpfn_increment_reference_count(Pfn1);
                    (*Pfn1).set_share_count((*Pfn1).share_count().wrapping_add((1) as ULONG_PTR));

                    (*Pfn1).set_page_location((ActiveAndValid) as u16);
                    ASSERT!(((*PointerProtoPte).valid() as usize) == 0);
                    ASSERT!(((*PointerProtoPte).prototype() as usize) == 0);
                    ASSERT!(((*PointerProtoPte).transition() as usize) == 1);
                    TempPte.Long = ((*PointerProtoPte).Long & !0xFFF)
                        | MmProtectToPteMask[(*PointerProtoPte).soft_protection() as usize];
                    TempPte.set_valid(true);
                    MI_MAKE_ACCESSED_PAGE(&mut TempPte);

                    if (((*Pfn1).modified() as u16) != 0)
                        && (MI_IS_PAGE_WRITEABLE(&TempPte) != 0)
                        && (MI_IS_PAGE_COPY_ON_WRITE(&TempPte) == 0)
                    {
                        MI_MAKE_DIRTY_PAGE(&mut TempPte);
                    } else {
                        MI_MAKE_CLEAN_PAGE(&mut TempPte);
                    }

                    MI_WRITE_VALID_PTE(PointerProtoPte, TempPte);
                    ASSERT!(((*PointerPte).valid() as usize) == 0);
                } else {
                    break;
                }

                ProcessedPtes += 1;
                if ProcessedPtes == PteCount {
                    MiCompleteProtoPteFault(
                        ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
                        Address,
                        PointerPte,
                        PointerProtoPte,
                        LockIrql,
                        &mut OutPfn,
                    );
                    break;
                }

                ASSERT!(false);
            }

            if ProcessedPtes != 0 {
                // MmTransitionCount InterlockedExchangeAddSizeT elided
                ProcessedPtes -= 1;

                ASSERT!(ProcessedPtes == 0);

                ASSERT!(OldIrql == KeGetCurrentIrql());
                ASSERT!(OldIrql <= APC_LEVEL);
                ASSERT!(KeAreAllApcsDisabled() != 0);
                return STATUS_PAGE_FAULT_TRANSITION;
            }

            OutPfn = MI_PFN_ELEMENT((*SuperProtoPte).page_frame_number() as PFN_NUMBER);
            MiReferenceUsedPageAndBumpLockCount(OutPfn as *mut _MMPFN);
            ASSERT!((*OutPfn).reference_count() > 1);
            ASSERT!(((*PointerPte).valid() as usize) == 0);

            Status = MiResolveProtoPteFault(
                ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
                Address,
                PointerPte,
                PointerProtoPte,
                &mut OutPfn,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                Process,
                LockIrql,
                TrapInformation,
            );

            if !OutPfn.is_null() {
                ASSERT!(!PointerProtoPte.is_null());
                OldIrql = MiAcquirePfnLock();

                MiDereferencePfnAndDropLockCount(OutPfn as *mut _MMPFN);
                ASSERT!((*OutPfn).reference_count() >= 1);

                MiReleasePfnLock(OldIrql);
            }

            ASSERT!(OldIrql == KeGetCurrentIrql());
            ASSERT!(OldIrql <= APC_LEVEL);
            ASSERT!(KeAreAllApcsDisabled() != 0);
            return Status;
        }
    }

    if (TempPte.prototype() as usize) == 0 && (TempPte.transition() as usize) != 0 {
        let mut InPageBlock: *mut PKEVENT = core::ptr::null_mut();
        let mut PreviousPageEvent: PKEVENT = core::ptr::null_mut();
        let mut CurrentPageEvent: KEVENT = core::mem::zeroed();

        LockIrql = MiAcquirePfnLock();

        Status = MiResolveTransitionFault(
            ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
            Address,
            PointerPte,
            Process,
            LockIrql,
            &mut InPageBlock,
        );

        ASSERT!(Status >= 0);

        if !InPageBlock.is_null() {
            KeInitializeEvent(&mut CurrentPageEvent as *mut KEVENT, 0, 0);
            PreviousPageEvent = *InPageBlock;
            *InPageBlock = &mut CurrentPageEvent as *mut KEVENT;
        }

        MiReleasePfnLock(LockIrql);

        if !InPageBlock.is_null() {
            KeWaitForSingleObject(
                &mut CurrentPageEvent as *mut KEVENT as *mut _,
                ntkrnl_sys::ddk::_KWAIT_REASON_WrPageIn as _,
                0,
                0,
                core::ptr::null_mut(),
            );

            if !PreviousPageEvent.is_null() {
                KeSetEvent(PreviousPageEvent as *mut c_void, 0, 0);
            }
        }

        ASSERT!(OldIrql == KeGetCurrentIrql());
        ASSERT!(OldIrql <= APC_LEVEL);
        ASSERT!(KeAreAllApcsDisabled() != 0);
        return Status;
    }

    if TempPte.soft_page_file_high() != 0 {
        LockIrql = MiAcquirePfnLock();

        Status = MiResolvePageFileFault(
            ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
            Address,
            PointerPte,
            Process,
            &mut LockIrql,
        );

        MiReleasePfnLock(LockIrql);

        ASSERT!(OldIrql == KeGetCurrentIrql());
        ASSERT!(OldIrql <= APC_LEVEL);
        ASSERT!(KeAreAllApcsDisabled() != 0);
        return Status;
    }

    ASSERT!((TempPte.valid() as usize) == 0);
    ASSERT!((TempPte.prototype() as usize) == 0);
    ASSERT!((TempPte.transition() as usize) == 0);
    ASSERT!(TempPte.soft_page_file_high() == 0);
    ASSERT!(TempPte.Long != 0);

    // #39: the decision above is made on TempPte, a SNAPSHOT, while the PTE the
    // resolver then writes is read live. On the wedge boot those disagreed --
    // the snapshot's Prototype was clear (its ASSERT did not fire, and that
    // mechanism demonstrably works: 86 boots print ASSERT FAILED) yet the live
    // entry read 0x400, Prototype set and nothing else. Print both side by side
    // so the disagreement is measured rather than inferred. One line per boot.
    if TempPte.soft_protection() & (ntkrnl_sys::mm::MM_PROTECT_ACCESS as ULONG) == 0
        || TempPte.Long != (*PointerPte).Long
    {
        static mut SNAP_REPORTED: bool = false;
        if !SNAP_REPORTED {
            SNAP_REPORTED = true;
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[DZSNAP] Address=%p Pte=%p snapshot=0x%016llX live=0x%016llX                   snap_prot=%u live_prot=%u snap_proto=%u live_proto=%u\n\0"
                    .as_ptr() as *const i8,
                Address,
                PointerPte,
                TempPte.Long as u64,
                (*PointerPte).Long as u64,
                TempPte.soft_protection(),
                (*PointerPte).soft_protection(),
                TempPte.prototype() as u32,
                (*PointerPte).prototype() as u32,
            );
        }
    }

    Status = MiResolveDemandZeroFault(
        Address,
        PointerPte,
        TempPte.soft_protection() as ULONG,
        Process,
        MM_NOIRQL,
    );
    ASSERT!(KeAreAllApcsDisabled() != 0);
    if Status >= 0 {
        #[cfg(feature = "MI_TRACE_PFNS")]
        {
            if !TrapInformation.is_null() {
                MiGetPfnEntry((*PointerPte).page_frame_number() as PFN_NUMBER).CallSite =
                    (*(TrapInformation as PKTRAP_FRAME)).Eip as PVOID;
            }
        }

        ASSERT!(OldIrql == KeGetCurrentIrql());
        ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);
        return Status;
    }

    Status
}

/// [KSTACK] at a PAGE_FAULT_IN_NONPAGED_AREA — the coverage gap the §9.97 soak found.
///
/// The `[KSTACK]` probe added to `ke/arm64/stubs.rs` sits in the `[KEXC]`
/// kernel-mode-*unhandled-exception* path, which ends in BUGCHECK 0x1E. A 0x50 is
/// raised straight out of MmArmAccessFault and never reaches it — so the first
/// reproduction the soak caught (round 2: sweep died at test 69/89 on
/// `ZwWaitForMultipleObjects`, right after the ZwCreateSection/ZwMapViewOfSection
/// canaries) printed nothing at all. Same question, second door.
///
/// It is worth asking here specifically because of what that bugcheck's operands
/// already say: `0x50 (0xffff8000050f4030, 2, 0xffff8000050f3e20, 0)`. P3 is
/// `TrapInformation`, a KTRAP_FRAME — which lives ON the kernel stack — and the
/// faulting address P1 is only 0x210 ABOVE it, across the ...f4000 page boundary.
/// Stacks grow down, so that is a reference to ALREADY-USED stack whose page is no
/// longer present. Printing StackBase/StackLimit turns that inference into a fact:
/// if P1 falls inside (StackLimit, StackBase] then a live kernel stack lost a
/// backing page, which is the same corruption as the 0x1E face and NOT generic
/// page reuse.
///
/// Note for whoever reads the silence: `[FREEMAPPED]` watching `MmDereferencePage`
/// cannot see this. That is the rmap-tracked *user* page path; kernel stacks come
/// from `MmCreateKernelStack` via system PTEs and are released by a different
/// route, so a stack page freed while mapped never passes that tripwire.
#[cfg(target_arch = "aarch64")]
unsafe fn arm64_dump_fault_stack(Address: PVOID, TrapInformation: PVOID) {
    extern "C" {
        fn ARM64_UART_TRACE_HEX(prefix: *const u8, val: ULONG_PTR);
    }
    let thread = crate::ke::ke_defs::KeGetCurrentThread();
    // SP first: kstack_verdict.py splits blocks on it.
    let sp = if TrapInformation.is_null() {
        0
    } else {
        (*(TrapInformation as *const ntkrnl_sys::ddk::_KTRAP_FRAME)).Sp as ULONG_PTR
    };
    ARM64_UART_TRACE_HEX(b"[KSTACK] SP=\0".as_ptr(), sp);
    ARM64_UART_TRACE_HEX(b"[KSTACK] TrapFrame=\0".as_ptr(), TrapInformation as ULONG_PTR);
    if !thread.is_null() {
        ARM64_UART_TRACE_HEX(
            b"[KSTACK] StackBase=\0".as_ptr(),
            (*thread).StackBase as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[KSTACK] StackLimit=\0".as_ptr(),
            (*thread).StackLimit as ULONG_PTR,
        );
        ARM64_UART_TRACE_HEX(
            b"[KSTACK] InitialStack=\0".as_ptr(),
            (*thread).InitialStack as ULONG_PTR,
        );
    }
    ARM64_UART_TRACE_HEX(b"[KSTACK] FaultVA=\0".as_ptr(), Address as ULONG_PTR);
    // The faulting PC/LR, so a bugcheck names the code rather than only the frame
    // address (P3). A 0x50 at smss startup otherwise gave no symbol at all.
    if !TrapInformation.is_null() {
        let tf = TrapInformation as *const ntkrnl_sys::ddk::_KTRAP_FRAME;
        ARM64_UART_TRACE_HEX(b"[KSTACK] FaultPC=\0".as_ptr(), (*tf).Pc as ULONG_PTR);
        ARM64_UART_TRACE_HEX(b"[KSTACK] FaultLR=\0".as_ptr(), (*tf).Lr as ULONG_PTR);
        // Name the module each lives in: a boot driver's address is otherwise
        // unsymbolizable from the log (no load bases are printed).
        for (tag, pc) in [(b"PC\0", (*tf).Pc as usize), (b"LR\0", (*tf).Lr as usize)] {
            let mut entry: ntkrnl_sys::ddk::PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
            let mut in_kernel: u8 = 0;
            let base = crate::ke::bug::KiPcToFileHeader(pc as _, &mut entry as *mut _ as _, 0, &mut in_kernel);
            if !base.is_null() && !entry.is_null() {
                crate::dbg_print!(
                    b"[KSTACK] Fault%s in %wZ+0x%x\n\0",
                    tag.as_ptr(),
                    core::ptr::addr_of!((*entry).BaseDllName),
                    (pc - (*entry).DllBase as usize) as u32,
                );
            }
        }
    }
}

#[cfg(not(target_arch = "aarch64"))]
unsafe fn arm64_dump_fault_stack(_Address: PVOID, _TrapInformation: PVOID) {}

#[no_mangle]
pub unsafe extern "system" fn MmArmAccessFault(
    FaultCode: ULONG,
    Address: PVOID,
    Mode: KPROCESSOR_MODE,
    TrapInformation: PVOID,
) -> NTSTATUS {
    let Address = opaque_ptr(Address);
    let TrapInformation = opaque_ptr(TrapInformation);
    let mut OldIrql = KeGetCurrentIrql();
    let mut LockIrql: KIRQL = 0;
    let mut ProtoPte: PMMPTE = core::ptr::null_mut();
    let PointerPte = MiAddressToPte(Address);
    let PointerPde = MiAddressToPde(Address);
    #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
    let PointerPpe = MiAddressToPpe(Address);
    #[cfg(target_pointer_width = "64")]
    let PointerPxe = MiAddressToPxe(Address);
    let mut TempPte: MMPTE;
    let CurrentThread: PETHREAD;
    let mut CurrentProcess: PEPROCESS;
    let mut Status: NTSTATUS;
    let mut WorkingSet: PMMSUPPORT = core::ptr::null_mut();
    let mut ProtectionCode: ULONG = 0;
    let mut Vad: PMMVAD = core::ptr::null_mut();
    let mut PageFrameIndex: PFN_NUMBER;
    let mut Color: ULONG;
    let IsSessionAddress: BOOLEAN;
    let Pfn1: PMMPFN;

    if OldIrql > APC_LEVEL {
        #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
        {
            if MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address) {
                MiSynchronizeSystemPde(PointerPte as PMMPDE);
            }
            MiCheckPdeForPagedPool(Address);
        }

        #[cfg(target_pointer_width = "64")]
        let pxe_valid = ((*PointerPxe).valid() as usize);
        #[cfg(not(target_pointer_width = "64"))]
        let pxe_valid = 1;

        #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
        let ppe_valid = ((*PointerPpe).valid() as usize);
        #[cfg(not(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae"))))]
        let ppe_valid = 1;

        if (pxe_valid == 0) || (ppe_valid == 0) || (((*PointerPde).valid() as usize) == 0) || (((*PointerPte).valid() as usize) == 0)
        {
            // The C prints the Va, IRQL and faulting registers here (pagfault.c:1762).
            // The translation had emptied every one of those blocks, so this status --
            // STATUS_IN_PAGE_ERROR|0x10000000, which the smoke logs showed as
            // `st=d0000006` killing csrss on amd64-bios -- reached the log with nothing
            // naming the code that faulted at DISPATCH_LEVEL. Keep the C's diagnostic.
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"MM:***PAGE FAULT AT IRQL > 1  Va %p, IRQL %lx\n\0".as_ptr() as *const i8,
                Address,
                OldIrql as u32,
            );
            if !TrapInformation.is_null() {
                let TrapFrame = TrapInformation as PKTRAP_FRAME;
                #[cfg(target_arch = "x86")]
                {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***EIP %p, EFL %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Eip as usize as PVOID,
                        (*TrapFrame).EFlags as usize as PVOID,
                    );
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***EAX %p, ECX %p EDX %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Eax as usize as PVOID,
                        (*TrapFrame).Ecx as usize as PVOID,
                        (*TrapFrame).Edx as usize as PVOID,
                    );
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***EBX %p, ESI %p EDI %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Ebx as usize as PVOID,
                        (*TrapFrame).Esi as usize as PVOID,
                        (*TrapFrame).Edi as usize as PVOID,
                    );
                }
                #[cfg(target_arch = "x86_64")]
                {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***RIP %p, EFL %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Rip as usize as PVOID,
                        (*TrapFrame).EFlags as usize as PVOID,
                    );
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***RAX %p, RCX %p RDX %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Rax as usize as PVOID,
                        (*TrapFrame).Rcx as usize as PVOID,
                        (*TrapFrame).Rdx as usize as PVOID,
                    );
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***RBX %p, RSI %p RDI %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Rbx as usize as PVOID,
                        (*TrapFrame).Rsi as usize as PVOID,
                        (*TrapFrame).Rdi as usize as PVOID,
                    );
                    crate::ke::bug::KiPrintPcAndStack(TrapFrame as *const _);
                }
                #[cfg(target_arch = "aarch64")]
                {
                    ntkrnl_sys::driver_ddk::DbgPrint(
                        b"MM:***PC %p, LR %p, SP %p\n\0".as_ptr() as *const i8,
                        (*TrapFrame).Pc as usize as PVOID,
                        (*TrapFrame).Lr as usize as PVOID,
                        (*TrapFrame).Sp as usize as PVOID,
                    );
                }
            }

            /* Tell the trap handler to fail */
            return STATUS_IN_PAGE_ERROR | 0x10000000;
        }

        ASSERT!(MI_IS_PAGE_LARGE(PointerPde as PMMPDE) == 0);
        ASSERT!(!((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize)) && (MI_IS_PAGE_COPY_ON_WRITE(PointerPte) != 0)));

        if MI_IS_WRITE_ACCESS((FaultCode) as usize) {
            Pfn1 = MI_PFN_ELEMENT((*PointerPte).page_frame_number() as PFN_NUMBER);
            let original_pte = mmpfn_original_pte(Pfn1);
            if ((*PointerPte).Long & PTE_READWRITE) == 0
                && (mmpte_software_protection(&original_pte) & MM_READWRITE) == 0
            {
                KeBugCheckEx(
                    ATTEMPTED_WRITE_TO_READONLY_MEMORY,
                    Address as ULONG_PTR,
                    (*PointerPte).Long as ULONG_PTR,
                    TrapInformation as ULONG_PTR,
                    10,
                );
            }
        }

        return STATUS_SUCCESS;
    }

    if Address >= MmSystemRangeStart {
        if Mode == 1 {
            return STATUS_ACCESS_VIOLATION;
        }

        #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
        {
            if MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address) {
                MiSynchronizeSystemPde(PointerPte as PMMPDE);
            }
            MiCheckPdeForPagedPool(Address);
        }

        #[cfg(target_pointer_width = "64")]
        {
            if MI_IS_PAGE_TABLE_OR_HYPER_ADDRESS(Address) {
                let mut UnderlyingVa: PVOID = core::ptr::null_mut();
                let UserPxe: PMMPTE;
                let UserPpe: PMMPTE;
                let mut NewPage: PFN_NUMBER;
                let mut NewPte: MMPTE = MMPTE { Long: 0 };
                let mut PfnIrql: KIRQL;

                let mut _goto_user_fault = false;
                if (Address as ULONG_PTR) >= PXE_BASE as ULONG_PTR
                    && (Address as ULONG_PTR) <= PXE_TOP as ULONG_PTR
                {
                    _goto_user_fault = true;
                } else if (Address as ULONG_PTR) >= PPE_BASE as ULONG_PTR {
                    UnderlyingVa = MiPpeToAddress(Address as PMMPTE);
                } else if (Address as ULONG_PTR) >= PDE_BASE as ULONG_PTR {
                    UnderlyingVa = MiPdeToAddress(Address as PMMPTE);
                } else {
                    UnderlyingVa = MiPteToAddress(Address as PMMPTE);
                }

                if !_goto_user_fault {

                UserPxe = MiAddressToPxe(UnderlyingVa);
                if ((*UserPxe).valid() as usize) == 0 {
                    PfnIrql = MiAcquirePfnLock();
                    NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
                    if NewPage == 0 {
                        MiReleasePfnLock(PfnIrql);
                        return STATUS_NO_MEMORY;
                    }
                    MiInitializePfn(NewPage as _, UserPxe as *mut _MMPTE, 1);
                    MiReleasePfnLock(PfnIrql);

                    MI_MAKE_HARDWARE_PTE(&mut NewPte, UserPxe, MM_EXECUTE_READWRITE as usize, NewPage);
                    if MI_IS_PAGE_WRITEABLE(&NewPte) != 0 {
                        MI_MAKE_DIRTY_PAGE(&mut NewPte);
                    }
                    MI_WRITE_VALID_PTE(UserPxe, NewPte);

                    // Force-zero the PPE page through the self-map.
                    // MiRemoveZeroPage SHOULD return zeroed pages, but on amd64
                    // we've observed stale PTE data (e.g., PFN=0 entries) in
                    // supposedly-zeroed page table pages. Belt-and-suspenders.
                    {
                        let ppe_page = (MiAddressToPpe(UnderlyingVa) as usize) & !(PAGE_SIZE as usize - 1);
                        core::ptr::write_bytes(ppe_page as *mut u8, 0u8, PAGE_SIZE as usize);
                    }
                }

                UserPpe = MiAddressToPpe(UnderlyingVa);
                if ((*UserPpe).valid() as usize) == 0 {
                    PfnIrql = MiAcquirePfnLock();
                    NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
                    if NewPage == 0 {
                        MiReleasePfnLock(PfnIrql);
                        return STATUS_NO_MEMORY;
                    }
                    MiInitializePfn(NewPage as _, UserPpe as *mut _MMPTE, 1);

                    {
                        let L1Pfn = MiGetPfnEntry(PFN_FROM_PXE(UserPxe));
                        mmpfn_set_used_page_table_entries(L1Pfn, mmpfn_used_page_table_entries(L1Pfn).wrapping_add(1));
                    }
                    MiReleasePfnLock(PfnIrql);

                    MI_MAKE_HARDWARE_PTE(&mut NewPte, UserPpe, MM_EXECUTE_READWRITE as usize, NewPage);
                    if MI_IS_PAGE_WRITEABLE(&NewPte) != 0 {
                        MI_MAKE_DIRTY_PAGE(&mut NewPte);
                    }
                    MI_WRITE_VALID_PTE(UserPpe, NewPte);

                    // Force-zero the PDE page through the self-map.
                    {
                        let pde_page = (MiAddressToPde(UnderlyingVa) as usize) & !(PAGE_SIZE as usize - 1);
                        core::ptr::write_bytes(pde_page as *mut u8, 0u8, PAGE_SIZE as usize);
                    }
                }

                {
                    let UserPde = MiAddressToPde(UnderlyingVa);
                    if ((*UserPde).valid() as usize) == 0 {
                        PfnIrql = MiAcquirePfnLock();
                        NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
                        if NewPage == 0 {
                            MiReleasePfnLock(PfnIrql);
                            return STATUS_NO_MEMORY;
                        }
                        MiInitializePfn(NewPage as _, UserPde as *mut _MMPTE, 1);

                        {
                            let L2Pfn = MiGetPfnEntry(PFN_FROM_PPE(UserPpe));
                            mmpfn_set_used_page_table_entries(L2Pfn, mmpfn_used_page_table_entries(L2Pfn).wrapping_add(1));
                        }
                        MiReleasePfnLock(PfnIrql);

                        MI_MAKE_HARDWARE_PTE(&mut NewPte, UserPde, MM_EXECUTE_READWRITE as usize, NewPage);
                        if MI_IS_PAGE_WRITEABLE(&NewPte) != 0 {
                            MI_MAKE_DIRTY_PAGE(&mut NewPte);
                        }
                        MI_WRITE_VALID_PTE(UserPde, NewPte);

                        // Force-zero the PTE page through the self-map.
                        {
                            let pte_page = (MiAddressToPte(UnderlyingVa) as usize) & !(PAGE_SIZE as usize - 1);
                            core::ptr::write_bytes(pte_page as *mut u8, 0u8, PAGE_SIZE as usize);
                        }
                    }
                }

                return STATUS_SUCCESS;
            }
        }
        } // end #[cfg(target_pointer_width = "64")]

        #[cfg(target_pointer_width = "64")]
        let pxe_valid = ((*PointerPxe).valid() as usize);
        #[cfg(not(target_pointer_width = "64"))]
        let pxe_valid = 1;

        #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
        let ppe_valid = ((*PointerPpe).valid() as usize);
        #[cfg(not(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae"))))]
        let ppe_valid = 1;

        if (pxe_valid == 0) || (ppe_valid == 0) || (((*PointerPde).valid() as usize) == 0) {
            arm64_dump_fault_stack(Address, TrapInformation);
            KeBugCheckEx(
                PAGE_FAULT_IN_NONPAGED_AREA,
                Address as ULONG_PTR,
                FaultCode as ULONG_PTR,
                TrapInformation as ULONG_PTR,
                2,
            );
        }

        IsSessionAddress = (MI_IS_SESSION_ADDRESS(Address)) as u8;

        TempPte = *PointerPte;
        if (TempPte.valid() as usize) == 1 {
            if IsSessionAddress == 0 {
                OldIrql = MiAcquirePfnLock();
                TempPte = *PointerPte;
                if (TempPte.valid() as usize) != 0 {
                    if MI_IS_WRITE_ACCESS((FaultCode) as usize) {
                        Pfn1 = MI_PFN_ELEMENT((*PointerPte).page_frame_number() as PFN_NUMBER);
                        let original_pte = mmpfn_original_pte(Pfn1);
                        if ((*PointerPte).Long & PTE_READWRITE) == 0
                            && (mmpte_software_protection(&original_pte) & MM_READWRITE) == 0
                        {
                            KeBugCheckEx(
                                ATTEMPTED_WRITE_TO_READONLY_MEMORY,
                                Address as ULONG_PTR,
                                (*PointerPte).Long as ULONG_PTR,
                                TrapInformation as ULONG_PTR,
                                11,
                            );
                        }
                    }

                    if (MI_IS_INSTRUCTION_FETCH((FaultCode) as usize)) && (MI_IS_PAGE_EXECUTABLE(&TempPte) == 0) {
                        KeBugCheckEx(
                            ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY,
                            Address as ULONG_PTR,
                            TempPte.Long as ULONG_PTR,
                            TrapInformation as ULONG_PTR,
                            1,
                        );
                    }
                }

                MiReleasePfnLock(OldIrql);
                return STATUS_SUCCESS;
            }
        }

        #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
        {
            if MI_IS_SESSION_PTE(Address as PMMPTE) {
                Status = MiCheckPdeForSessionSpace(Address);
                if Status < 0 {
                    arm64_dump_fault_stack(Address, TrapInformation);
                    KeBugCheckEx(
                        PAGE_FAULT_IN_NONPAGED_AREA,
                        Address as ULONG_PTR,
                        FaultCode as ULONG_PTR,
                        TrapInformation as ULONG_PTR,
                        6,
                    );
                }
            }
        }

        if MI_IS_PAGE_TABLE_OR_HYPER_ADDRESS(Address) {
            #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
            {
                ASSERT!(MiCheckPdeForPagedPool(Address) != STATUS_WAIT_1);
            }
            // goto UserFault - fall through to user fault handling below
        } else
        {

        CurrentThread = PsGetCurrentThread();

        if IsSessionAddress == 0 {
            WorkingSet = core::ptr::addr_of_mut!(MmSystemCacheWs) as PMMSUPPORT;
            CurrentProcess = core::ptr::null_mut();

            // Elided bitfield checks (always false with & 0)
        } else {
            CurrentProcess = HYDRA_PROCESS;
            WorkingSet = core::ptr::addr_of_mut!((*(*(MmSessionSpace as *mut _MM_SESSION_SPACE)).GlobalVirtualAddress).Vm) as PMMSUPPORT;
        }
'RetryKernel: loop {
        // [guard] KeRaiseIrql(APC_LEVEL, &mut LockIrql);
        let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(APC_LEVEL) };
        MiLockWorkingSet(CurrentThread, WorkingSet);

        TempPte = *PointerPte;
        if (TempPte.valid() as usize) == 1 {
            if MI_IS_WRITE_ACCESS((FaultCode) as usize) {
                Pfn1 = MI_PFN_ELEMENT((*PointerPte).page_frame_number() as PFN_NUMBER);
                let original_pte = mmpfn_original_pte(Pfn1);
                if !(TempPte.Long & PTE_READWRITE != 0)
                    && !(mmpte_software_protection(&original_pte) & MM_READWRITE != 0)
                    && (MI_IS_PAGE_COPY_ON_WRITE(&TempPte) == 0)
                {
                    ASSERT!(IsSessionAddress == 0);
                    KeBugCheckEx(
                        ATTEMPTED_WRITE_TO_READONLY_MEMORY,
                        Address as ULONG_PTR,
                        TempPte.Long as ULONG_PTR,
                        TrapInformation as ULONG_PTR,
                        12,
                    );
                }
            }

            if (MI_IS_INSTRUCTION_FETCH((FaultCode) as usize)) && (MI_IS_PAGE_EXECUTABLE(&TempPte) == 0) {
                KeBugCheckEx(
                    ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY,
                    Address as ULONG_PTR,
                    TempPte.Long as ULONG_PTR,
                    TrapInformation as ULONG_PTR,
                    2,
                );
            }

            if (IsSessionAddress != 0)
                && (MI_IS_WRITE_ACCESS((FaultCode) as usize))
                && (MI_IS_PAGE_WRITEABLE(&TempPte) == 0)
            {
                ASSERT!(MI_IS_SESSION_IMAGE_ADDRESS(Address));

                if MI_IS_PAGE_COPY_ON_WRITE(&TempPte) == 0 {
                    KeBugCheckEx(
                        ATTEMPTED_WRITE_TO_READONLY_MEMORY,
                        Address as ULONG_PTR,
                        TempPte.Long as ULONG_PTR,
                        TrapInformation as ULONG_PTR,
                        13,
                    );
                }

                ASSERT!(false);
            }

            MiUnlockWorkingSet(CurrentThread, WorkingSet);
            // [guard-drop] KeLowerIrql(LockIrql);
            return STATUS_SUCCESS;
        }

        if (TempPte.prototype() as usize) != 0 {
            if (MmProtectFreedNonPagedPool != 0)
                && (((Address >= MmNonPagedPoolStart)
                    && (Address
                        < ((MmNonPagedPoolStart as ULONG_PTR + MmSizeOfNonPagedPoolInBytes as ULONG_PTR) as PVOID)))
                    || ((Address >= MmNonPagedPoolExpansionStart) && (Address < MmNonPagedPoolEnd)))
            {
                KeBugCheckEx(
                    DRIVER_CAUGHT_MODIFYING_FREED_POOL,
                    Address as ULONG_PTR,
                    FaultCode as ULONG_PTR,
                    Mode as ULONG_PTR,
                    4,
                );
            }

            ProtoPte = MiProtoPteToPte(&mut TempPte as PMMPTE);

            if (IsSessionAddress != 0) && (TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED) {
                ProtoPte = MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
                ASSERT!(!ProtoPte.is_null());
            }
        } else {
            ASSERT!((TempPte.transition() as usize) == 0);

            if TempPte.soft_protection() == MM_NOACCESS as ULONG {
                arm64_dump_fault_stack(Address, TrapInformation);
                KeBugCheckEx(
                    PAGE_FAULT_IN_NONPAGED_AREA,
                    Address as ULONG_PTR,
                    FaultCode as ULONG_PTR,
                    TrapInformation as ULONG_PTR,
                    1,
                );
            }

            if TempPte.soft_protection() == MM_ZERO_ACCESS as ULONG {
                // Debugcon trace: dump EIP from trap frame to identify faulting instruction
                #[cfg(target_arch = "x86")]
                {
                    if !TrapInformation.is_null() {
                        let eip = *((TrapInformation as *const u8)
                            .add(core::mem::offset_of!(ntkrnl_sys::ddk::_KTRAP_FRAME, Eip))
                            as *const u32);
                        let mut s: i32 = 28;
                        while s >= 0 {
                            let d = ((eip >> s) & 0xF) as u8;
                            let ch = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                            s -= 4;
                        }
                        // Also dump the faulting address
                        let mut s2: i32 = 28;
                        while s2 >= 0 {
                            let d = ((Address as u32 >> s2) & 0xF) as u8;
                            let ch = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                            s2 -= 4;
                        }
                        // Also dump TempPte.Long for diagnosis
                        let mut s3: i32 = 28;
                        while s3 >= 0 {
                            let d = ((TempPte.Long as u32 >> s3) & 0xF) as u8;
                            let ch = if d < 10 { b'0' + d } else { b'a' + d - 10 };
                            s3 -= 4;
                        }
                    }
                }
                arm64_dump_fault_stack(Address, TrapInformation);
                KeBugCheckEx(
                    PAGE_FAULT_IN_NONPAGED_AREA,
                    Address as ULONG_PTR,
                    FaultCode as ULONG_PTR,
                    TrapInformation as ULONG_PTR,
                    0,
                );
            }
        }

        if (MI_IS_WRITE_ACCESS((FaultCode) as usize))
            && (ProtoPte.is_null())
            && (IsSessionAddress == 0)
            && ((TempPte.valid() as usize) == 0)
        {
            ASSERT!((TempPte.transition() as usize) == 0);
            if (TempPte.soft_protection() & MM_READWRITE) == 0 {
                KeBugCheckEx(
                    ATTEMPTED_WRITE_TO_READONLY_MEMORY,
                    Address as ULONG_PTR,
                    TempPte.Long as ULONG_PTR,
                    TrapInformation as ULONG_PTR,
                    14,
                );
            }
        }

        Status = MiDispatchFault(
            FaultCode,
            Address,
            PointerPte,
            ProtoPte,
            0,
            CurrentProcess,
            TrapInformation,
            core::ptr::null_mut(),
        );

        ASSERT!(KeAreAllApcsDisabled() != 0);
        MiUnlockWorkingSet(CurrentThread, WorkingSet);
        // [guard-drop] KeLowerIrql(LockIrql);

        if Status == STATUS_NO_MEMORY {
            MmRebalanceMemoryConsumersAndWait();
            continue 'RetryKernel;
        }

        return Status;
        } // end RetryKernel loop
    } // end else (not page table/hyper address)
    } // end if Address >= MmSystemRangeStart

    // UserFault:
    CurrentThread = PsGetCurrentThread();
    CurrentProcess = (*(CurrentThread as *mut _ETHREAD)).Tcb.__bindgen_anon_1.ApcState.Process as PEPROCESS;

    MiLockProcessWorkingSet(CurrentProcess, CurrentThread);

    ProtectionCode = MM_INVALID_PROTECTION;

    'exit_user: loop {

    #[cfg(target_pointer_width = "64")]
    {
        if ((*PointerPxe).valid() as usize) == 0 {
            ASSERT!((*PointerPxe).Long == 0 as usize);
            ASSERT!(PointerPte <= MiHighestUserPte);

            MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
            if ProtectionCode == MM_NOACCESS {
                if MiLocateAddress(Address).is_null() {
                    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                    return STATUS_ACCESS_VIOLATION;
                }
                ProtectionCode = MM_EXECUTE_READWRITE;
            }

            Status = MiResolveDemandZeroFault(
                PointerPpe as PVOID,
                PointerPxe,
                MM_EXECUTE_READWRITE,
                CurrentProcess,
                MM_NOIRQL,
            );
            if Status < 0 {
                break 'exit_user;
            }

            ASSERT!(((*PointerPxe).valid() as usize) == 1);
        }
    }

    #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
    {
        if ((*PointerPpe).valid() as usize) == 0 {
            ASSERT!((*PointerPpe).Long == 0 as usize);
            ASSERT!(PointerPte <= MiHighestUserPte);

            if ProtectionCode == MM_INVALID_PROTECTION {
                MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
            }

            if ProtectionCode == MM_NOACCESS {
                if MiLocateAddress(Address).is_null() {
                    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                    return STATUS_ACCESS_VIOLATION;
                }
                ProtectionCode = MM_EXECUTE_READWRITE;
            }

            Status = MiResolveDemandZeroFault(
                PointerPde as *mut c_void,
                PointerPpe,
                MM_EXECUTE_READWRITE,
                CurrentProcess,
                MM_NOIRQL,
            );
            if Status < 0 {
                break 'exit_user;
            }

            ASSERT!(((*PointerPpe).valid() as usize) == 1);
            MiIncrementPageTableReferences(PointerPde as PVOID);
        }
    }

    if ((*PointerPde).valid() as usize) == 0 {
        ASSERT!((*PointerPde).Long == 0 as usize);

        #[cfg(feature = "MI_TRACE_PFNS")]
        {
            UserPdeFault = 1;
        }

        if ProtectionCode == MM_INVALID_PROTECTION {
            MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
        }

        if ProtectionCode == MM_NOACCESS {
            #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
            {
                if !MiLocateAddress(Address).is_null() {
                    ProtectionCode = MM_EXECUTE_READWRITE;
                } else {
                    #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
                    {
                        MiCheckPdeForPagedPool(Address);
                    }
                    Status = if ((*PointerPde).valid() as usize) == 1 {
                        STATUS_SUCCESS
                    } else {
                        STATUS_ACCESS_VIOLATION
                    };

                    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                    return Status;
                }
            }
            #[cfg(not(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae"))))]
            {
                #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
                {
                    MiCheckPdeForPagedPool(Address);
                }
                Status = if ((*PointerPde).valid() as usize) == 1 {
                    STATUS_SUCCESS
                } else {
                    STATUS_ACCESS_VIOLATION
                };

                MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                return Status;
            }
        }

        Status = MiResolveDemandZeroFault(
            PointerPte as *mut c_void,
            PointerPde,
            MM_EXECUTE_READWRITE,
            CurrentProcess,
            MM_NOIRQL,
        );
        if Status < 0 {
            break 'exit_user;
        }

        #[cfg(any(target_pointer_width = "64", all(target_arch = "x86", feature = "pae")))]
        {
            MiIncrementPageTableReferences(PointerPte as PVOID);
        }

        #[cfg(feature = "MI_TRACE_PFNS")]
        {
            UserPdeFault = 0;
            if !TrapInformation.is_null() {
                MiGetPfnEntry((*PointerPde).page_frame_number() as PFN_NUMBER).CallSite =
                    (*(TrapInformation as PKTRAP_FRAME)).Eip as PVOID;
            }
        }

        ASSERT!(KeAreAllApcsDisabled() != 0);
        ASSERT!(((*PointerPde).valid() as usize) == 1);
    } else {
        ASSERT!(MI_IS_PAGE_LARGE(PointerPde as PMMPDE) == 0);
    }

    TempPte = *PointerPte;

    if (TempPte.valid() as usize) != 0 {
        if MI_IS_WRITE_ACCESS((FaultCode) as usize) {
            if MI_IS_PAGE_COPY_ON_WRITE(&TempPte) != 0 {
                let mut OldPageFrameIndex: PFN_NUMBER;

                LockIrql = MiAcquirePfnLock();

                ASSERT!(MmAvailablePages > 0);

                ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_COW);
                ntkrnl_sys::MI_SET_PROCESS!(CurrentProcess as *mut c_void);

                PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_PROCESS_COLOR(CurrentProcess as *mut _EPROCESS as *mut c_void));
                if PageFrameIndex == 0 {
                    MiReleasePfnLock(LockIrql);
                    Status = STATUS_NO_MEMORY;
                    break 'exit_user;
                }
                OldPageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);

                MiCopyPfn(PageFrameIndex, OldPageFrameIndex);

                Pfn1 = MI_PFN_ELEMENT(OldPageFrameIndex);
                ASSERT!(((*Pfn1).prototype_pte() as u16) == 1);
                ASSERT!(!MI_IS_PFN_DELETED(Pfn1));
                ProtoPte = (*Pfn1).pte_address();
                MiDeletePte(PointerPte as *mut _MMPTE, Address, CurrentProcess, ProtoPte as *mut _MMPTE);

                MiInitializePfn((PageFrameIndex) as _, PointerPte as *mut _MMPTE, 1);
                TempPte.set_page_frame_number((PageFrameIndex as usize) as PFN_NUMBER);
                MI_MAKE_WRITE_PAGE(&mut TempPte);
                MI_MAKE_DIRTY_PAGE(&mut TempPte);
                mi_clear_copy_on_write(&mut TempPte);

                MI_WRITE_VALID_PTE(PointerPte, TempPte);

                MiReleasePfnLock(LockIrql);

                MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                return STATUS_PAGE_FAULT_COPY_ON_WRITE;
            }

            if MI_IS_PAGE_WRITEABLE(&TempPte) == 0 {
                MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                return STATUS_ACCESS_VIOLATION;
            }
        }

        #[cfg(feature = "_MI_HAS_NO_EXECUTE")]
        {
            if (MI_IS_INSTRUCTION_FETCH(FaultCode)) && (MI_IS_PAGE_EXECUTABLE(&TempPte) == 0) {
                if (*(CurrentProcess as *mut _EPROCESS)).Pcb.__bindgen_anon_2.Flags.ExecuteEnable() != 0 {
                    mi_make_executable_page(&mut TempPte);
                    MI_UPDATE_VALID_PTE(PointerPte, TempPte);
                    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                    return STATUS_SUCCESS;
                }

                MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                return STATUS_ACCESS_VIOLATION;
            }
        }

        MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
        return STATUS_SUCCESS;
    }

    // Demand-zero soft PTE check: only Valid bit clear and only Protection field set.
    // We compare via the bitfield accessor to avoid hardcoding a per-arch shift —
    // ARM64's struct puts Protection at bit 1 while amd64/i386 put it at bit 5.
    {
        let prot = TempPte.soft_protection() as ULONG;
        let mut bare = MMPTE { Long: 0 };
        bare.set_soft_protection(prot);
        let is_demand_zero = TempPte.Long == bare.Long
            && (prot == MM_READWRITE || prot == MM_EXECUTE_READWRITE);
        if is_demand_zero {
        Status = MiResolveDemandZeroFault(Address, PointerPte, prot, CurrentProcess, MM_NOIRQL);
        if Status < 0 {
            break 'exit_user;
        }

        #[cfg(feature = "MI_TRACE_PFNS")]
        {
            if !TrapInformation.is_null() {
                MiGetPfnEntry((*PointerPte).page_frame_number() as PFN_NUMBER).CallSite =
                    (*(TrapInformation as PKTRAP_FRAME)).Eip as PVOID;
            }
        }

        MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
        return STATUS_PAGE_FAULT_DEMAND_ZERO;
        }
    }

    if TempPte.Long == 0 {
        ProtoPte = MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
        if ProtectionCode == MM_NOACCESS {
            #[cfg(all(target_arch = "x86", not(target_pointer_width = "64")))]
            {
                MiCheckPdeForPagedPool(Address);
            }
            Status = if ((*PointerPte).valid() as usize) == 1 {
                STATUS_SUCCESS
            } else {
                STATUS_ACCESS_VIOLATION
            };

            MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
            return Status;
        }

        let _is_user_addr = Address <= MM_HIGHEST_USER_ADDRESS as PVOID
            || MiIsUserPte(Address as PMMPTE as *mut c_void) != 0;
        #[cfg(target_pointer_width = "64")]
        let _is_user_addr = _is_user_addr || MiIsUserPde(Address as PMMPDE as *mut c_void) != 0;
        if _is_user_addr {
            MiIncrementPageTableReferences(Address);
        }

        if (ProtectionCode & MM_PROTECT_SPECIAL) == MM_GUARDPAGE {
            ASSERT!(ProtectionCode != MM_DECOMMIT);

            TempPte.set_soft_protection(((ProtectionCode & !MM_GUARDPAGE) as u32) as ULONG);
            MI_WRITE_INVALID_PTE(PointerPte, TempPte);

            ASSERT!(ProtoPte.is_null());
            // ASSERT: elided bitfield check (always true with & 0 == 0)

            MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
            ASSERT!(KeGetCurrentIrql() == OldIrql);

            return MiCheckForUserStackOverflow(Address, TrapInformation);
        }

        if ProtoPte.is_null() {
            if PointerPde == MiAddressToPde(PTE_BASE as PVOID) {
                #[cfg(target_arch = "arm")]
                {
                    MI_WRITE_INVALID_PDE(PointerPte as PMMPDE, DemandZeroPde);
                }
                #[cfg(not(target_arch = "arm"))]
                {
                    MI_WRITE_INVALID_PDE(PointerPte, DemandZeroPde);
                }
            } else {
                TempPte.set_soft_protection((ProtectionCode as u32) as ULONG);
                MI_WRITE_INVALID_PTE(PointerPte, TempPte);
            }

            OldIrql = MiAcquirePfnLock();

            ntkrnl_sys::MI_SET_USAGE!(MI_USAGE_PEB_TEB);
            ntkrnl_sys::MI_SET_PROCESS2!(core::ptr::null_mut());
            Color = MI_GET_NEXT_PROCESS_COLOR(CurrentProcess as *mut _EPROCESS as *mut c_void);
            PageFrameIndex = (MiRemoveZeroPageSafe(Color)) as usize;
            if PageFrameIndex == 0 {
                PageFrameIndex = MiRemoveAnyPage(Color);
                MiReleasePfnLock(OldIrql);

                if PageFrameIndex == 0 {
                    Status = STATUS_NO_MEMORY;
                    break 'exit_user;
                }

                MiZeroPfn(PageFrameIndex);

                OldIrql = MiAcquirePfnLock();
            }

            MiInitializePfn((PageFrameIndex) as _, PointerPte as *mut _MMPTE, 1);

            (*(CurrentProcess as *mut _EPROCESS)).NumberOfPrivatePages += 1;

            /* MmDemandZeroCount (no-op) */;

            MiReleasePfnLock(OldIrql);

            if PointerPte <= MiHighestUserPte {
                MI_MAKE_HARDWARE_PTE_USER(
                    &mut TempPte,
                    PointerPte,
                    ((*PointerPte).soft_protection()) as usize,
                    PageFrameIndex,
                );
            } else {
                MI_MAKE_HARDWARE_PTE(
                    &mut TempPte,
                    PointerPte,
                    ((*PointerPte).soft_protection()) as usize,
                    PageFrameIndex,
                );
            }

            if MI_IS_PAGE_WRITEABLE(&TempPte) != 0 {
                MI_MAKE_DIRTY_PAGE(&mut TempPte);
            }

            MI_WRITE_VALID_PTE(PointerPte, TempPte);
            Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
            ASSERT!((*Pfn1).flink() == 0);

            ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);
            MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
            return STATUS_PAGE_FAULT_DEMAND_ZERO;
        }

        ASSERT!(ProtectionCode != 0x100);

        TempPte = MMPTE { Long: PrototypePte as ULONG_PTR };
        TempPte.set_soft_protection((ProtectionCode as u32) as ULONG);
        ASSERT!(TempPte.Long != 0);
        MI_WRITE_INVALID_PTE(PointerPte, TempPte);
    } else {
        ProtectionCode = TempPte.soft_protection() as ULONG;
        if (TempPte.prototype() as usize) != 0 {
            if TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED {
                ProtoPte = MiCheckVirtualAddress(Address, &mut ProtectionCode, &mut Vad);
                if ProtoPte.is_null() {
                    ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);
                    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
                    return STATUS_ACCESS_VIOLATION;
                }
            } else {
                ProtoPte = MiProtoPteToPte(&mut TempPte as PMMPTE);

                if TempPte.proto_read_only() {
                    ProtectionCode = MM_READONLY;
                } else {
                    ProtectionCode = 0x100;
                    ASSERT!(!(*(CurrentProcess as *mut _EPROCESS)).CloneRoot.is_null());
                }
            }
        }
    }

    if ProtectionCode != 0x100 {
        Status = MiAccessCheck(
            PointerPte,
            ((!MI_IS_NOT_PRESENT_FAULT((FaultCode) as usize))) as BOOLEAN,
            Mode,
            ProtectionCode as ULONG_PTR,
            TrapInformation,
            0,
        );
        if Status != STATUS_SUCCESS {
            // ASSERT: elided bitfield check (always true with & 0 == 0)

            MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
            ASSERT!(KeGetCurrentIrql() == OldIrql);

            if Status == STATUS_GUARD_PAGE_VIOLATION {
                return MiCheckForUserStackOverflow(Address, TrapInformation);
            }

            return Status;
        }
    }

    Status = MiDispatchFault(
        FaultCode,
        Address,
        PointerPte,
        ProtoPte,
        0,
        CurrentProcess,
        TrapInformation,
        Vad,
    );

    break 'exit_user;
    } // end 'exit_user loop

// ExitUser:
    ASSERT!(KeGetCurrentIrql() <= APC_LEVEL);
    MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);

    if Status == STATUS_NO_MEMORY {
        // The C retries here (goto UserFault). Deliberately not: with no commit
        // limit and no pagefile, out of pages means out for good until a process
        // exits, and a retrying fault never lets one. Retrying every user fault
        // turned exhaustion into a whole-machine stall (appstorm and explorer
        // parked in faults for 2h at 768M, 20261010-054207-p5m0hi2a). Failing the
        // fault kills only the starving app, whose exit frees the pages that
        // kernel-internal waiters (MiMakeSystemAddressValid) retry for.
        MmRebalanceMemoryConsumersAndWait();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn MmGetExecuteOptions(ExecuteOptions: *mut ULONG) -> NTSTATUS {
    let ExecuteOptions = opaque_ptr(ExecuteOptions);
    let CurrentProcess = core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb);
    ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);

    *ExecuteOptions = 0;

    if (*CurrentProcess).__bindgen_anon_2.Flags.ExecuteDisable() != 0 {
        *ExecuteOptions |= 0x1;
    }

    if (*CurrentProcess).__bindgen_anon_2.Flags.ExecuteEnable() != 0 {
        *ExecuteOptions |= 0x2;
    }

    if (*CurrentProcess).__bindgen_anon_2.Flags.DisableThunkEmulation() != 0 {
        *ExecuteOptions |= 0x4;
    }

    if (*CurrentProcess).__bindgen_anon_2.Flags.Permanent() != 0 {
        *ExecuteOptions |= 0x8;
    }

    if (*CurrentProcess).__bindgen_anon_2.Flags.ExecuteDispatchEnable() != 0 {
        *ExecuteOptions |= 0x10;
    }

    if (*CurrentProcess).__bindgen_anon_2.Flags.ImageDispatchEnable() != 0 {
        *ExecuteOptions |= 0x20;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmSetExecuteOptions(ExecuteOptions: ULONG) -> NTSTATUS {
    let CurrentProcess = core::ptr::addr_of_mut!((*(PsGetCurrentProcess() as *mut _EPROCESS)).Pcb);
    let mut ProcessLock: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Status = STATUS_ACCESS_DENIED;
    ASSERT!(KeGetCurrentIrql() == PASSIVE_LEVEL);

    // ndk/mmtypes.h: MEM_EXECUTE_OPTION_VALID_FLAGS is 0x3F.
    const MEM_EXECUTE_OPTION_VALID_FLAGS: u32 = 0x3F;
    if (ExecuteOptions & !MEM_EXECUTE_OPTION_VALID_FLAGS) != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    KiAcquireProcessLockRaiseToSynch(CurrentProcess, &mut ProcessLock as *mut _ as *mut _KLOCK_QUEUE_HANDLE);

    if (*CurrentProcess).__bindgen_anon_2.Flags.Permanent() == 0 {
        (*CurrentProcess).__bindgen_anon_2.Flags.set_ExecuteDisable(0);

        if (ExecuteOptions & 0x1) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ExecuteDisable(1);
        }
        if (ExecuteOptions & 0x2) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ExecuteEnable(1);
        }
        if (ExecuteOptions & 0x4) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_DisableThunkEmulation(1);
        }
        if (ExecuteOptions & 0x8) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_Permanent(1);
        }
        if (ExecuteOptions & 0x10) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ExecuteDispatchEnable(1);
        }
        if (ExecuteOptions & 0x20) != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ImageDispatchEnable(1);
        }

        if (*CurrentProcess).__bindgen_anon_2.Flags.ExecuteEnable() != 0 {
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ExecuteDispatchEnable(1);
            (*CurrentProcess).__bindgen_anon_2.Flags.set_ImageDispatchEnable(1);
        }

        Status = STATUS_SUCCESS;
    }

    KiReleaseProcessLock(&mut ProcessLock as *mut _ as *mut _KLOCK_QUEUE_HANDLE);
    Status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/mm/ARM3/pagfault.c
//  * PURPOSE:         ARM Memory Manager Page Fault Handling
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// #define MODULE_INVOLVED_IN_ARM3
// #include <mm/ARM3/miarm.h>
//
// VOID
// NTAPI
// MmRebalanceMemoryConsumersAndWait(VOID);
//
// /* GLOBALS ********************************************************************/
//
// #define HYDRA_PROCESS (PEPROCESS)1
// #if MI_TRACE_PFNS
// BOOLEAN UserPdeFault = FALSE;
// #endif
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// static
// NTSTATUS
// NTAPI
// MiCheckForUserStackOverflow(IN PVOID Address,
//                             IN PVOID TrapInformation)
// {
//     PETHREAD CurrentThread = PsGetCurrentThread();
//     PTEB Teb = CurrentThread->Tcb.Teb;
//     PVOID StackBase, DeallocationStack, NextStackAddress;
//     SIZE_T GuaranteedSize;
//     NTSTATUS Status;
//
//     /* Do we own the address space lock? */
//     if (CurrentThread->AddressSpaceOwner == 1)
//     {
//         /* This isn't valid */
//         DPRINT1("Process owns address space lock\n");
//         ASSERT(KeAreAllApcsDisabled() == TRUE);
//         return STATUS_GUARD_PAGE_VIOLATION;
//     }
//
//     /* Are we attached? */
//     if (KeIsAttachedProcess())
//     {
//         /* This isn't valid */
//         DPRINT1("Process is attached\n");
//         return STATUS_GUARD_PAGE_VIOLATION;
//     }
//
//     /* Read the current settings */
//     StackBase = Teb->NtTib.StackBase;
//     DeallocationStack = Teb->DeallocationStack;
//     GuaranteedSize = Teb->GuaranteedStackBytes;
//     DPRINT("Handling guard page fault with Stacks Addresses 0x%p and 0x%p, guarantee: %lx\n",
//             StackBase, DeallocationStack, GuaranteedSize);
//
//     /* Guarantees make this code harder, for now, assume there aren't any */
//     ASSERT(GuaranteedSize == 0);
//
//     /* So allocate only the minimum guard page size */
//     GuaranteedSize = PAGE_SIZE;
//
//     /* Does this faulting stack address actually exist in the stack? */
//     if ((Address >= StackBase) || (Address < DeallocationStack))
//     {
//         /* That's odd... */
//         DPRINT1("Faulting address outside of stack bounds. Address=%p, StackBase=%p, DeallocationStack=%p\n",
//                 Address, StackBase, DeallocationStack);
//         return STATUS_GUARD_PAGE_VIOLATION;
//     }
//
//     /* This is where the stack will start now */
//     NextStackAddress = (PVOID)((ULONG_PTR)PAGE_ALIGN(Address) - GuaranteedSize);
//
//     /* Do we have at least one page between here and the end of the stack? */
//     if (((ULONG_PTR)NextStackAddress - PAGE_SIZE) <= (ULONG_PTR)DeallocationStack)
//     {
//         /* We don't -- Trying to make this guard page valid now */
//         DPRINT1("Close to our death...\n");
//
//         /* Calculate the next memory address */
//         NextStackAddress = (PVOID)((ULONG_PTR)PAGE_ALIGN(DeallocationStack) + GuaranteedSize);
//
//         /* Allocate the memory */
//         Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                          &NextStackAddress,
//                                          0,
//                                          &GuaranteedSize,
//                                          MEM_COMMIT,
//                                          PAGE_READWRITE);
//         if (NT_SUCCESS(Status))
//         {
//             /* Success! */
//             Teb->NtTib.StackLimit = NextStackAddress;
//         }
//         else
//         {
//             DPRINT1("Failed to allocate memory\n");
//         }
//
//         return STATUS_STACK_OVERFLOW;
//     }
//
//     /* Don't handle this flag yet */
//     ASSERT((PsGetCurrentProcess()->Peb->NtGlobalFlag & FLG_DISABLE_STACK_EXTENSION) == 0);
//
//     /* Update the stack limit */
//     Teb->NtTib.StackLimit = (PVOID)((ULONG_PTR)NextStackAddress + GuaranteedSize);
//
//     /* Now move the guard page to the next page */
//     Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                      &NextStackAddress,
//                                      0,
//                                      &GuaranteedSize,
//                                      MEM_COMMIT,
//                                      PAGE_READWRITE | PAGE_GUARD);
//     if ((NT_SUCCESS(Status) || (Status == STATUS_ALREADY_COMMITTED)))
//     {
//         /* We did it! */
//         DPRINT("Guard page handled successfully for %p\n", Address);
//         return STATUS_PAGE_FAULT_GUARD_PAGE;
//     }
//
//     /* Fail, we couldn't move the guard page */
//     DPRINT1("Guard page failure: %lx\n", Status);
//     ASSERT(FALSE);
//     return STATUS_STACK_OVERFLOW;
// }
//
// FORCEINLINE
// BOOLEAN
// MiIsAccessAllowed(
//     _In_ ULONG ProtectionMask,
//     _In_ BOOLEAN Write,
//     _In_ BOOLEAN Execute)
// {
//     #define _BYTE_MASK(Bit0, Bit1, Bit2, Bit3, Bit4, Bit5, Bit6, Bit7) \
//         (Bit0) | ((Bit1) << 1) | ((Bit2) << 2) | ((Bit3) << 3) | \
//         ((Bit4) << 4) | ((Bit5) << 5) | ((Bit6) << 6) | ((Bit7) << 7)
//     static const UCHAR AccessAllowedMask[2][2] =
//     {
//         {   // Protect 0  1  2  3  4  5  6  7
//             _BYTE_MASK(0, 1, 1, 1, 1, 1, 1, 1), // READ
//             _BYTE_MASK(0, 0, 1, 1, 0, 0, 1, 1), // EXECUTE READ
//         },
//         {
//             _BYTE_MASK(0, 0, 0, 0, 1, 1, 1, 1), // WRITE
//             _BYTE_MASK(0, 0, 0, 0, 0, 0, 1, 1), // EXECUTE WRITE
//         }
//     };
//
//     /* We want only the lower access bits */
//     ProtectionMask &= MM_PROTECT_ACCESS;
//
//     /* Look it up in the table */
//     return (AccessAllowedMask[Write != 0][Execute != 0] >> ProtectionMask) & 1;
// }
//
// static
// NTSTATUS
// NTAPI
// MiAccessCheck(IN PMMPTE PointerPte,
//               IN BOOLEAN StoreInstruction,
//               IN KPROCESSOR_MODE PreviousMode,
//               IN ULONG_PTR ProtectionMask,
//               IN PVOID TrapFrame,
//               IN BOOLEAN LockHeld)
// {
//     MMPTE TempPte;
//
//     /* Check for invalid user-mode access */
//     if ((PreviousMode == UserMode) && (PointerPte > MiHighestUserPte))
//     {
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Capture the PTE -- is it valid? */
//     TempPte = *PointerPte;
//     if ((TempPte.valid() as usize))
//     {
//         /* Was someone trying to write to it? */
//         if (StoreInstruction)
//         {
//             /* Is it writable?*/
//             if (MI_IS_PAGE_WRITEABLE(&TempPte) ||
//                 MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//             {
//                 /* Then there's nothing to worry about */
//                 return STATUS_SUCCESS;
//             }
//
//             /* Oops! This isn't allowed */
//             return STATUS_ACCESS_VIOLATION;
//         }
//
//         /* Someone was trying to read from a valid PTE, that's fine too */
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if the protection on the page allows what is being attempted */
//     if (!MiIsAccessAllowed(ProtectionMask, StoreInstruction, FALSE))
//     {
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Check if this is a guard page */
//     if ((ProtectionMask & MM_PROTECT_SPECIAL) == MM_GUARDPAGE)
//     {
//         ASSERT(ProtectionMask != MM_DECOMMIT);
//
//         /* Attached processes can't expand their stack */
//         if (KeIsAttachedProcess()) return STATUS_ACCESS_VIOLATION;
//
//         /* No support for prototype PTEs yet */
//         ASSERT((TempPte.prototype() as usize) == 0);
//
//         /* Remove the guard page bit, and return a guard page violation */
//         TempPte.set_soft_protection((ProtectionMask & ~MM_GUARDPAGE) as ULONG);
//         ASSERT(TempPte.Long != 0);
//         MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//         return STATUS_GUARD_PAGE_VIOLATION;
//     }
//
//     /* Nothing to do */
//     return STATUS_SUCCESS;
// }
//
// static
// PMMPTE
// NTAPI
// MiCheckVirtualAddress(IN PVOID VirtualAddress,
//                       OUT PULONG ProtectCode,
//                       OUT PMMVAD *ProtoVad)
// {
//     PMMVAD Vad;
//     PMMPTE PointerPte;
//
//     /* No prototype/section support for now */
//     *ProtoVad = NULL;
//
//     /* User or kernel fault? */
//     if (VirtualAddress <= MM_HIGHEST_USER_ADDRESS)
//     {
//         /* Special case for shared data */
//         if (PAGE_ALIGN(VirtualAddress) == (PVOID)MM_SHARED_USER_DATA_VA)
//         {
//             /* It's a read-only page */
//             *ProtectCode = MM_READONLY;
//             return MmSharedUserDataPte;
//         }
//
//         /* Find the VAD, it might not exist if the address is bogus */
//         Vad = MiLocateAddress(VirtualAddress);
//         if (!Vad)
//         {
//             /* Bogus virtual address */
//             *ProtectCode = MM_NOACCESS;
//             return NULL;
//         }
//
//         /* ReactOS does not handle physical memory VADs yet */
//         ASSERT(Vad->u.VadFlags.VadType() != VadDevicePhysicalMemory);
//
//         /* Check if it's a section, or just an allocation */
//         if (Vad->u.VadFlags.PrivateMemory())
//         {
//             /* ReactOS does not handle AWE VADs yet */
//             ASSERT(Vad->u.VadFlags.VadType() != VadAwe);
//
//             /* This must be a TEB/PEB VAD */
//             if (Vad->u.VadFlags.MemCommit())
//             {
//                 /* It's committed, so return the VAD protection */
//                 *ProtectCode = (ULONG)Vad->u.VadFlags.Protection();
//             }
//             else
//             {
//                 /* It has not yet been committed, so return no access */
//                 *ProtectCode = MM_NOACCESS;
//             }
//
//             /* In both cases, return no PTE */
//             return NULL;
//         }
//         else
//         {
//             /* ReactOS does not supoprt these VADs yet */
//             ASSERT(Vad->u.VadFlags.VadType() != VadImageMap);
//             ASSERT(Vad->u2.VadFlags2.ExtendableFile() == 0);
//
//             /* Return the proto VAD */
//             *ProtoVad = Vad;
//
//             /* Get the prototype PTE for this page */
//             PointerPte = (((ULONG_PTR)VirtualAddress >> PAGE_SHIFT) - Vad->StartingVpn) + Vad->FirstPrototypePte;
//             ASSERT(PointerPte != NULL);
//             ASSERT(PointerPte <= Vad->LastContiguousPte);
//
//             /* Return the Prototype PTE and the protection for the page mapping */
//             *ProtectCode = (ULONG)Vad->u.VadFlags.Protection();
//             return PointerPte;
//         }
//     }
//     else if (MI_IS_PAGE_TABLE_ADDRESS(VirtualAddress))
//     {
//         /* This should never happen, as these addresses are handled by the double-maping */
//         if (((PMMPTE)VirtualAddress >= MiAddressToPte(MmPagedPoolStart)) &&
//             ((PMMPTE)VirtualAddress <= MmPagedPoolInfo.LastPteForPagedPool))
//         {
//             /* Fail such access */
//             *ProtectCode = MM_NOACCESS;
//             return NULL;
//         }
//
//         /* Return full access rights */
//         *ProtectCode = MM_EXECUTE_READWRITE;
//         return NULL;
//     }
//     else if (MI_IS_SESSION_ADDRESS(VirtualAddress))
//     {
//         /* ReactOS does not have an image list yet, so bail out to failure case */
//         ASSERT(IsListEmpty(&MmSessionSpace->ImageList));
//     }
//
//     /* Default case -- failure */
//     *ProtectCode = MM_NOACCESS;
//     return NULL;
// }
//
// #if (_MI_PAGING_LEVELS == 2)
// static
// NTSTATUS
// FASTCALL
// MiCheckPdeForSessionSpace(IN PVOID Address)
// {
//     MMPTE TempPde;
//     PMMPDE PointerPde;
//     PVOID SessionAddress;
//     ULONG Index;
//
//     /* Is this a session PTE? */
//     if (MI_IS_SESSION_PTE(Address as PMMPTE))
//     {
//         /* Make sure the PDE for session space is valid */
//         PointerPde = MiAddressToPde(MmSessionSpace);
//         if (!PointerPde->u.Hard.Valid)
//         {
//             /* This means there's no valid session, bail out */
//             DbgPrint("MiCheckPdeForSessionSpace: No current session for PTE %p\n",
//                      Address);
//             DbgBreakPoint();
//             return STATUS_ACCESS_VIOLATION;
//         }
//
//         /* Now get the session-specific page table for this address */
//         SessionAddress = MiPteToAddress(Address);
//         PointerPde = MiAddressToPte(Address);
//         if (PointerPde->u.Hard.Valid) return STATUS_WAIT_1;
//
//         /* It's not valid, so find it in the page table array */
//         Index = ((ULONG_PTR)SessionAddress - (ULONG_PTR)MmSessionBase) >> 22;
//         TempPde.Long = MmSessionSpace->PageTables[Index].Long;
//         if ((TempPde.valid() as usize))
//         {
//             /* The copy is valid, so swap it in */
//             InterlockedExchange((PLONG)PointerPde, TempPde.Long);
//             return STATUS_WAIT_1;
//         }
//
//         /* We don't seem to have allocated a page table for this address yet? */
//         DbgPrint("MiCheckPdeForSessionSpace: No Session PDE for PTE %p, %p\n",
//                  PointerPde->u.Long, SessionAddress);
//         DbgBreakPoint();
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Is the address also a session address? If not, we're done */
//     if (!MI_IS_SESSION_ADDRESS(Address)) return STATUS_SUCCESS;
//
//     /* It is, so again get the PDE for session space */
//     PointerPde = MiAddressToPde(MmSessionSpace);
//     if (!PointerPde->u.Hard.Valid)
//     {
//         /* This means there's no valid session, bail out */
//         DbgPrint("MiCheckPdeForSessionSpace: No current session for VA %p\n",
//                     Address);
//         DbgBreakPoint();
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* Now get the PDE for the address itself */
//     PointerPde = MiAddressToPde(Address);
//     if (!PointerPde->u.Hard.Valid)
//     {
//         /* Do the swap, we should be good to go */
//         Index = ((ULONG_PTR)Address - (ULONG_PTR)MmSessionBase) >> 22;
//         PointerPde->u.Long = MmSessionSpace->PageTables[Index].Long;
//         if (PointerPde->u.Hard.Valid) return STATUS_WAIT_1;
//
//         /* We had not allocated a page table for this session address yet, fail! */
//         DbgPrint("MiCheckPdeForSessionSpace: No Session PDE for VA %p, %p\n",
//                  PointerPde->u.Long, Address);
//         DbgBreakPoint();
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* It's valid, so there's nothing to do */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// FASTCALL
// MiCheckPdeForPagedPool(IN PVOID Address)
// {
//     PMMPDE PointerPde;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* Check session PDE */
//     if (MI_IS_SESSION_ADDRESS(Address)) return MiCheckPdeForSessionSpace(Address);
//     if (MI_IS_SESSION_PTE(Address as PMMPTE)) return MiCheckPdeForSessionSpace(Address);
//
//     //
//     // Check if this is a fault while trying to access the page table itself
//     //
//     if (MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address))
//     {
//         //
//         // Send a hint to the page fault handler that this is only a valid fault
//         // if we already detected this was access within the page table range
//         //
//         PointerPde = (PMMPDE)MiAddressToPte(Address);
//         Status = STATUS_WAIT_1;
//     }
//     else if (Address < MmSystemRangeStart)
//     {
//         //
//         // This is totally illegal
//         //
//         return STATUS_ACCESS_VIOLATION;
//     }
//     else
//     {
//         //
//         // Get the PDE for the address
//         //
//         PointerPde = MiAddressToPde(Address);
//     }
//
//     //
//     // Check if it's not valid
//     //
//     if (PointerPde->u.Hard.Valid == 0)
//     {
//         //
//         // Copy it from our double-mapped system page directory
//         //
//         InterlockedExchangePte(PointerPde,
//                                MmSystemPagePtes[((ULONG_PTR)PointerPde & (SYSTEM_PD_SIZE - 1)) / sizeof(MMPTE)].Long);
//     }
//
//     //
//     // Return status
//     //
//     return Status;
// }
// #else
// NTSTATUS
// FASTCALL
// MiCheckPdeForPagedPool(IN PVOID Address)
// {
//     return STATUS_ACCESS_VIOLATION;
// }
// #endif
//
// VOID
// NTAPI
// MiZeroPfn(IN PFN_NUMBER PageFrameNumber)
// {
//     PMMPTE ZeroPte;
//     MMPTE TempPte;
//     PMMPFN Pfn1;
//     PVOID ZeroAddress;
//
//     /* Get the PFN for this page */
//     Pfn1 = MiGetPfnEntry(PageFrameNumber);
//     ASSERT(Pfn1);
//
//     /* Grab a system PTE we can use to zero the page */
//     ZeroPte = MiReserveSystemPtes(1, SystemPteSpace);
//     ASSERT(ZeroPte);
//
//     /* Initialize the PTE for it */
//     TempPte = ValidKernelPte;
//     TempPte.set_page_frame_number((PageFrameNumber) as PFN_NUMBER);
//
//     /* Setup caching */
//     if (Pfn1->u3.e1.CacheAttribute == MiWriteCombined)
//     {
//         /* Write combining, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_COMBINED(&TempPte);
//     }
//     else if (Pfn1->u3.e1.CacheAttribute == MiNonCached)
//     {
//         /* Write through, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_THROUGH(&TempPte);
//     }
//
//     /* Make the system PTE valid with our PFN */
//     MI_WRITE_VALID_PTE(ZeroPte, TempPte);
//
//     /* Get the address it maps to, and zero it out */
//     ZeroAddress = MiPteToAddress(ZeroPte);
//     KeZeroPages(ZeroAddress, PAGE_SIZE);
//
//     /* Now get rid of it */
//     MiReleaseSystemPtes(ZeroPte, 1, SystemPteSpace);
// }
//
// VOID
// NTAPI
// MiCopyPfn(
//     _In_ PFN_NUMBER DestPage,
//     _In_ PFN_NUMBER SrcPage)
// {
//     PMMPTE SysPtes;
//     MMPTE TempPte;
//     PMMPFN DestPfn, SrcPfn;
//     PVOID DestAddress;
//     const VOID* SrcAddress;
//
//     /* Get the PFNs */
//     DestPfn = MiGetPfnEntry(DestPage);
//     ASSERT(DestPfn);
//     SrcPfn = MiGetPfnEntry(SrcPage);
//     ASSERT(SrcPfn);
//
//     /* Grab 2 system PTEs */
//     SysPtes = MiReserveSystemPtes(2, SystemPteSpace);
//     ASSERT(SysPtes);
//
//     /* Initialize the destination PTE */
//     TempPte = ValidKernelPte;
//     TempPte.set_page_frame_number((DestPage) as PFN_NUMBER);
//
//     /* Setup caching */
//     if (DestPfn->u3.e1.CacheAttribute == MiWriteCombined)
//     {
//         /* Write combining, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_COMBINED(&TempPte);
//     }
//     else if (DestPfn->u3.e1.CacheAttribute == MiNonCached)
//     {
//         /* Write through, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_THROUGH(&TempPte);
//     }
//
//     /* Make the system PTE valid with our PFN */
//     MI_WRITE_VALID_PTE(&SysPtes[0], TempPte);
//
//     /* Initialize the source PTE */
//     TempPte = ValidKernelPte;
//     TempPte.set_page_frame_number((SrcPage) as PFN_NUMBER);
//
//     /* Setup caching */
//     if (SrcPfn->u3.e1.CacheAttribute == MiNonCached)
//     {
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//     }
//
//     /* Make the system PTE valid with our PFN */
//     MI_WRITE_VALID_PTE(&SysPtes[1], TempPte);
//
//     /* Get the addresses and perform the copy */
//     DestAddress = MiPteToAddress(&SysPtes[0]);
//     SrcAddress = MiPteToAddress(&SysPtes[1]);
//     RtlCopyMemory(DestAddress, SrcAddress, PAGE_SIZE);
//
//     /* Now get rid of it */
//     MiReleaseSystemPtes(SysPtes, 2, SystemPteSpace);
// }
//
// static
// NTSTATUS
// NTAPI
// MiResolveDemandZeroFault(IN PVOID Address,
//                          IN PMMPTE PointerPte,
//                          IN ULONG Protection,
//                          IN PEPROCESS Process,
//                          IN KIRQL OldIrql)
// {
//     PFN_NUMBER PageFrameNumber = 0;
//     MMPTE TempPte;
//     BOOLEAN NeedZero = FALSE, HaveLock = FALSE;
//     ULONG Color;
//     PMMPFN Pfn1;
//     DPRINT("ARM3 Demand Zero Page Fault Handler for address: %p in process: %p\n",
//             Address,
//             Process);
//
//     /* Must currently only be called by paging path */
//     if ((Process > HYDRA_PROCESS) && (OldIrql == MM_NOIRQL))
//     {
//         /* Sanity check */
//         ASSERT(MI_IS_PAGE_TABLE_ADDRESS(PointerPte));
//
//         /* No forking yet */
//         ASSERT(Process->ForkInProgress == NULL);
//
//         /* Get process color */
//         Color = MI_GET_NEXT_PROCESS_COLOR(Process);
//         ASSERT(Color != 0xFFFFFFFF);
//
//         /* We'll need a zero page */
//         NeedZero = TRUE;
//     }
//     else
//     {
//         /* Check if we need a zero page */
//         NeedZero = (OldIrql != MM_NOIRQL);
//
//         /* Session-backed image views must be zeroed */
//         if ((Process == HYDRA_PROCESS) &&
//             ((MI_IS_SESSION_IMAGE_ADDRESS(Address)) ||
//              ((Address >= MiSessionViewStart) && (Address < MiSessionSpaceWs))))
//         {
//             NeedZero = TRUE;
//         }
//
//         /* Hardcode unknown color */
//         Color = 0xFFFFFFFF;
//     }
//
//     /* Check if the PFN database should be acquired */
//     if (OldIrql == MM_NOIRQL)
//     {
//         /* Acquire it and remember we should release it after */
//         OldIrql = MiAcquirePfnLock();
//         HaveLock = TRUE;
//     }
//
//     /* We either manually locked the PFN DB, or already came with it locked */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//
//     /* Assert we have enough pages */
//     //ASSERT(MmAvailablePages >= 32);
//
// #if MI_TRACE_PFNS
//     if (UserPdeFault) MI_SET_USAGE(MI_USAGE_PAGE_TABLE);
//     if (!UserPdeFault) MI_SET_USAGE(MI_USAGE_DEMAND_ZERO);
// #endif
//     if (Process == HYDRA_PROCESS) super::helpers::MI_SET_PROCESS2("Hydra");
//     else if (Process) super::helpers::MI_SET_PROCESS2(Process->ImageFileName);
//     else super::helpers::MI_SET_PROCESS2("Kernel Demand 0");
//
//     /* Do we need a zero page? */
//     if (Color != 0xFFFFFFFF)
//     {
//         /* Try to get one, if we couldn't grab a free page and zero it */
//         PageFrameNumber = MiRemoveZeroPageSafe(Color);
//         if (!PageFrameNumber)
//         {
//             /* We'll need a free page and zero it manually */
//             PageFrameNumber = MiRemoveAnyPage(Color);
//             NeedZero = TRUE;
//         }
//         else
//         {
//             /* Page guaranteed to be zero-filled */
//             NeedZero = FALSE;
//         }
//     }
//     else
//     {
//         /* Get a color, and see if we should grab a zero or non-zero page */
//         Color = MI_GET_NEXT_COLOR();
//         if (!NeedZero)
//         {
//             /* Process or system doesn't want a zero page, grab anything */
//             PageFrameNumber = MiRemoveAnyPage(Color);
//         }
//         else
//         {
//             /* System wants a zero page, obtain one */
//             PageFrameNumber = MiRemoveZeroPage(Color);
//             /* No need to zero-fill it */
//             NeedZero = FALSE;
//         }
//     }
//
//     if (PageFrameNumber == 0)
//     {
//         MiReleasePfnLock(OldIrql);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Initialize it */
//     MiInitializePfn(PageFrameNumber, PointerPte, TRUE);
//
//     /* Increment demand zero faults */
//     KeGetCurrentPrcb()->MmDemandZeroCount++;
//
//     /* Do we have the lock? */
//     if (HaveLock)
//     {
//         /* Release it */
//         MiReleasePfnLock(OldIrql);
//
//         /* Update performance counters */
//         if (Process > HYDRA_PROCESS) Process->NumberOfPrivatePages++;
//     }
//
//     /* Zero the page if need be */
//     if (NeedZero) MiZeroPfn(PageFrameNumber);
//
//     /* Fault on user PDE, or fault on user PTE? */
//     if (PointerPte <= MiHighestUserPte)
//     {
//         /* User fault, build a user PTE */
//         MI_MAKE_HARDWARE_PTE_USER(&TempPte,
//                                   PointerPte,
//                                   Protection,
//                                   PageFrameNumber);
//     }
//     else
//     {
//         /* This is a user-mode PDE, create a kernel PTE for it */
//         MI_MAKE_HARDWARE_PTE(&TempPte,
//                              PointerPte,
//                              Protection,
//                              PageFrameNumber);
//     }
//
//     /* Set it dirty if it's a writable page */
//     if (MI_IS_PAGE_WRITEABLE(&TempPte)) MI_MAKE_DIRTY_PAGE(&TempPte);
//
//     /* Write it */
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     /* Did we manually acquire the lock */
//     if (HaveLock)
//     {
//         /* Get the PFN entry */
//         Pfn1 = MI_PFN_ELEMENT(PageFrameNumber);
//
//         /* Windows does these sanity checks */
//         ASSERT(Pfn1->u1.Event == 0);
//         ASSERT(Pfn1->u3.e1.PrototypePte == 0);
//     }
//
//     //
//     // It's all good now
//     //
//     DPRINT("Demand zero page has now been paged in\n");
//     return STATUS_PAGE_FAULT_DEMAND_ZERO;
// }
//
// static
// NTSTATUS
// NTAPI
// MiCompleteProtoPteFault(IN BOOLEAN StoreInstruction,
//                         IN PVOID Address,
//                         IN PMMPTE PointerPte,
//                         IN PMMPTE PointerProtoPte,
//                         IN KIRQL OldIrql,
//                         IN PMMPFN* LockedProtoPfn)
// {
//     MMPTE TempPte;
//     PMMPTE OriginalPte, PageTablePte;
//     ULONG_PTR Protection;
//     PFN_NUMBER PageFrameIndex;
//     PMMPFN Pfn1, Pfn2;
//     BOOLEAN OriginalProtection, DirtyPage;
//
//     /* Must be called with an valid prototype PTE, with the PFN lock held */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(PointerProtoPte->u.Hard.Valid == 1);
//
//     /* Get the page */
//     PageFrameIndex = PFN_FROM_PTE(PointerProtoPte);
//
//     /* Get the PFN entry and set it as a prototype PTE */
//     Pfn1 = MiGetPfnEntry(PageFrameIndex);
//     Pfn1->u3.e1.PrototypePte = 1;
//
//     /* Increment the share count for the page table */
//     PageTablePte = MiAddressToPte(PointerPte);
//     Pfn2 = MiGetPfnEntry(PageTablePte->u.Hard.PageFrameNumber);
//     Pfn2->u2.ShareCount++;
//
//     /* Check where we should be getting the protection information from */
//     if (PointerPte->u.Soft.PageFileHigh == MI_PTE_LOOKUP_NEEDED)
//     {
//         /* Get the protection from the PTE, there's no real Proto PTE data */
//         Protection = PointerPte->u.Soft.Protection();
//
//         /* Remember that we did not use the proto protection */
//         OriginalProtection = FALSE;
//     }
//     else
//     {
//         /* Get the protection from the original PTE link */
//         OriginalPte = &Pfn1->OriginalPte;
//         Protection = OriginalPte->u.Soft.Protection();
//
//         /* Remember that we used the original protection */
//         OriginalProtection = TRUE;
//
//         /* Check if this was a write on a read only proto */
//         if ((StoreInstruction) && !(Protection & MM_READWRITE))
//         {
//             /* Clear the flag */
//             StoreInstruction = 0;
//         }
//     }
//
//     /* Check if this was a write on a non-COW page */
//     DirtyPage = FALSE;
//     if ((StoreInstruction) && ((Protection & MM_WRITECOPY) != MM_WRITECOPY))
//     {
//         /* Then the page should be marked dirty */
//         DirtyPage = TRUE;
//
//         /* ReactOS check */
//         ASSERT(Pfn1->(OriginalPte.prototype() as usize) != 0);
//     }
//
//     /* Did we get a locked incoming PFN? */
//     if (*LockedProtoPfn)
//     {
//         /* Drop a reference */
//         ASSERT((*LockedProtoPfn)->u3.e2.ReferenceCount >= 1);
//         MiDereferencePfnAndDropLockCount(*LockedProtoPfn);
//         *LockedProtoPfn = NULL;
//     }
//
//     /* Release the PFN lock */
//     MiReleasePfnLock(OldIrql);
//
//     /* Remove special/caching bits */
//     Protection &= ~MM_PROTECT_SPECIAL;
//
//     /* Setup caching */
//     if (Pfn1->u3.e1.CacheAttribute == MiWriteCombined)
//     {
//         /* Write combining, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_COMBINED(&TempPte);
//     }
//     else if (Pfn1->u3.e1.CacheAttribute == MiNonCached)
//     {
//         /* Write through, no caching */
//         MI_PAGE_DISABLE_CACHE(&TempPte);
//         MI_PAGE_WRITE_THROUGH(&TempPte);
//     }
//
//     /* Check if this is a kernel or user address */
//     if (Address < MmSystemRangeStart)
//     {
//         /* Build the user PTE */
//         MI_MAKE_HARDWARE_PTE_USER(&TempPte, PointerPte, Protection, PageFrameIndex);
//     }
//     else
//     {
//         /* Build the kernel PTE */
//         MI_MAKE_HARDWARE_PTE(&TempPte, PointerPte, Protection, PageFrameIndex);
//     }
//
//     /* Set the dirty flag if needed */
//     if (DirtyPage) MI_MAKE_DIRTY_PAGE(&TempPte);
//
//     /* Write the PTE */
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     /* Reset the protection if needed */
//     if (OriginalProtection) Protection = MM_ZERO_ACCESS;
//
//     /* Return success */
//     ASSERT(PointerPte == MiAddressToPte(Address));
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// NTAPI
// MiResolvePageFileFault(_In_ BOOLEAN StoreInstruction,
//                        _In_ PVOID FaultingAddress,
//                        _In_ PMMPTE PointerPte,
//                        _In_ PEPROCESS CurrentProcess,
//                        _Inout_ KIRQL *OldIrql)
// {
//     ULONG Color;
//     PFN_NUMBER Page;
//     NTSTATUS Status;
//     MMPTE TempPte = *PointerPte;
//     PMMPFN Pfn1;
//     ULONG PageFileIndex = TempPte.soft_page_file_low();
//     ULONG_PTR PageFileOffset = TempPte.soft_page_file_high();
//     ULONG Protection = TempPte.soft_protection();
//
//     /* Things we don't support yet */
//     ASSERT(CurrentProcess > HYDRA_PROCESS);
//     ASSERT(*OldIrql != MM_NOIRQL);
//
//     MI_SET_USAGE(MI_USAGE_PAGE_FILE);
//     MI_SET_PROCESS(CurrentProcess);
//
//     /* We must hold the PFN lock */
//     MI_ASSERT_PFN_LOCK_HELD();
//
//     /* Some sanity checks */
//     ASSERT((TempPte.valid() as usize) == 0);
//     ASSERT(TempPte.soft_page_file_high() != 0);
//     ASSERT(TempPte.soft_page_file_high() != MI_PTE_LOOKUP_NEEDED);
//
//     /* Get any page, it will be overwritten */
//     Color = MI_GET_NEXT_PROCESS_COLOR(CurrentProcess);
//     Page = MiRemoveAnyPage(Color);
//     if (Page == 0)
//     {
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Initialize this PFN */
//     MiInitializePfn(Page, PointerPte, StoreInstruction);
//
//     /* Sets the PFN as being in IO operation */
//     Pfn1 = MI_PFN_ELEMENT(Page);
//     ASSERT(Pfn1->u1.Event == NULL);
//     ASSERT(Pfn1->u3.e1.ReadInProgress == 0);
//     ASSERT(Pfn1->u3.e1.WriteInProgress == 0);
//     Pfn1->u3.e1.ReadInProgress = 1;
//
//     /* We must write the PTE now as the PFN lock will be released while performing the IO operation */
//     MI_MAKE_TRANSITION_PTE(&TempPte, Page, Protection);
//
//     MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//
//     /* Release the PFN lock while we proceed */
//     MiReleasePfnLock(*OldIrql);
//
//     /* Do the paging IO */
//     Status = MiReadPageFile(Page, PageFileIndex, PageFileOffset);
//
//     /* Lock the PFN database again */
//     *OldIrql = MiAcquirePfnLock();
//
//     /* Nobody should have changed that while we were not looking */
//     ASSERT(Pfn1->u3.e1.ReadInProgress == 1);
//     ASSERT(Pfn1->u3.e1.WriteInProgress == 0);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Malheur! */
//         ASSERT(FALSE);
//         Pfn1->u4.InPageError = 1;
//         Pfn1->u1.ReadStatus = Status;
//     }
//
//     /* And the PTE can finally be valid */
//     MI_MAKE_HARDWARE_PTE(&TempPte, PointerPte, Protection, Page);
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     Pfn1->u3.e1.ReadInProgress = 0;
//     /* Did someone start to wait on us while we proceeded ? */
//     if (Pfn1->u1.Event)
//     {
//         /* Tell them we're done */
//         KeSetEvent(Pfn1->u1.Event, IO_NO_INCREMENT, FALSE);
//     }
//
//     return Status;
// }
//
// static
// NTSTATUS
// NTAPI
// MiResolveTransitionFault(IN BOOLEAN StoreInstruction,
//                          IN PVOID FaultingAddress,
//                          IN PMMPTE PointerPte,
//                          IN PEPROCESS CurrentProcess,
//                          IN KIRQL OldIrql,
//                          OUT PKEVENT **InPageBlock)
// {
//     PFN_NUMBER PageFrameIndex;
//     PMMPFN Pfn1;
//     MMPTE TempPte;
//     PMMPTE PointerToPteForProtoPage;
//     DPRINT("Transition fault on 0x%p with PTE 0x%p in process %s\n",
//             FaultingAddress, PointerPte, CurrentProcess->ImageFileName);
//
//     /* Windowss does this check */
//     ASSERT(*InPageBlock == NULL);
//
//     /* ARM3 doesn't support this path */
//     ASSERT(OldIrql != MM_NOIRQL);
//
//     /* Capture the PTE and make sure it's in transition format */
//     TempPte = *PointerPte;
//     ASSERT(((TempPte.valid() as usize) == 0) &&
//            ((TempPte.prototype() as usize) == 0) &&
//            ((TempPte.transition() as usize) == 1));
//
//     /* Get the PFN and the PFN entry */
//     PageFrameIndex = TempPte.page_frame_number();
//     DPRINT("Transition PFN: %lx\n", PageFrameIndex);
//     Pfn1 = MiGetPfnEntry(PageFrameIndex);
//
//     /* One more transition fault! */
//     InterlockedIncrement(&KeGetCurrentPrcb()->MmTransitionCount);
//
//     /* This is from ARM3 -- Windows normally handles this here */
//     ASSERT(Pfn1->u4.InPageError == 0);
//
//     /* See if we should wait before terminating the fault */
//     if ((Pfn1->u3.e1.ReadInProgress == 1)
//             || ((Pfn1->u3.e1.WriteInProgress == 1) && StoreInstruction))
//     {
//         DPRINT1("The page is currently in a page transition !\n");
//         *InPageBlock = &Pfn1->u1.Event;
//         if (PointerPte == Pfn1->PteAddress)
//         {
//             DPRINT1("And this if for this particular PTE.\n");
//             /* The PTE will be made valid by the thread serving the fault */
//             return STATUS_SUCCESS; // FIXME: Maybe something more descriptive
//         }
//     }
//
//     /* Windows checks there's some free pages and this isn't an in-page error */
//     ASSERT(MmAvailablePages > 0);
//     ASSERT(Pfn1->u4.InPageError == 0);
//
//     /* ReactOS checks for this */
//     ASSERT(MmAvailablePages > 32);
//
//     /* Was this a transition page in the valid list, or free/zero list? */
//     if (Pfn1->u3.e1.PageLocation == ActiveAndValid)
//     {
//         /* All Windows does here is a bunch of sanity checks */
//         DPRINT("Transition in active list\n");
//         ASSERT((Pfn1->PteAddress >= MiAddressToPte(MmPagedPoolStart)) &&
//                (Pfn1->PteAddress <= MiAddressToPte(MmPagedPoolEnd)));
//         ASSERT(Pfn1->u2.ShareCount != 0);
//         ASSERT(Pfn1->u3.e2.ReferenceCount != 0);
//     }
//     else
//     {
//         /* Otherwise, the page is removed from its list */
//         DPRINT("Transition page in free/zero list\n");
//         MiUnlinkPageFromList(Pfn1);
//         MiReferenceUnusedPageAndBumpLockCount(Pfn1);
//     }
//
//     /* At this point, there should no longer be any in-page errors */
//     ASSERT(Pfn1->u4.InPageError == 0);
//
//     /* Check if this was a PFN with no more share references */
//     if (Pfn1->u2.ShareCount == 0) MiDropLockCount(Pfn1);
//
//     /* Bump the share count and make the page valid */
//     Pfn1->u2.ShareCount++;
//     Pfn1->u3.e1.PageLocation = ActiveAndValid;
//
//     /* Prototype PTEs are in paged pool, which itself might be in transition */
//     if (FaultingAddress >= MmSystemRangeStart)
//     {
//         /* Check if this is a paged pool PTE in transition state */
//         PointerToPteForProtoPage = MiAddressToPte(PointerPte);
//         TempPte = *PointerToPteForProtoPage;
//         if (((TempPte.valid() as usize) == 0) && ((TempPte.transition() as usize) == 1))
//         {
//             /* This isn't yet supported */
//             DPRINT1("Double transition fault not yet supported\n");
//             ASSERT(FALSE);
//         }
//     }
//
//     /* Build the final PTE */
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//     ASSERT(PointerPte->u.Trans.Prototype == 0);
//     ASSERT(PointerPte->u.Trans.Transition == 1);
//     TempPte.Long = (PointerPte->u.Long & ~0xFFF) |
//                      (MmProtectToPteMask[PointerPte->u.Trans.Protection()]) |
//                      MiDetermineUserGlobalPteMask(PointerPte);
//
//     /* Is the PTE writeable? */
//     if ((Pfn1->u3.e1.Modified) &&
//         MI_IS_PAGE_WRITEABLE(&TempPte) &&
//         !MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//     {
//         /* Make it dirty */
//         MI_MAKE_DIRTY_PAGE(&TempPte);
//     }
//     else
//     {
//         /* Make it clean */
//         MI_MAKE_CLEAN_PAGE(&TempPte);
//     }
//
//     /* Write the valid PTE */
//     MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//     /* Return success */
//     return STATUS_PAGE_FAULT_TRANSITION;
// }
//
// static
// NTSTATUS
// NTAPI
// MiResolveProtoPteFault(IN BOOLEAN StoreInstruction,
//                        IN PVOID Address,
//                        IN PMMPTE PointerPte,
//                        IN PMMPTE PointerProtoPte,
//                        IN OUT PMMPFN *OutPfn,
//                        OUT PVOID *PageFileData,
//                        OUT PMMPTE PteValue,
//                        IN PEPROCESS Process,
//                        IN KIRQL OldIrql,
//                        IN PVOID TrapInformation)
// {
//     MMPTE TempPte, PteContents;
//     PMMPFN Pfn1;
//     PFN_NUMBER PageFrameIndex;
//     NTSTATUS Status;
//     PKEVENT* InPageBlock = NULL;
//     ULONG Protection;
//
//     /* Must be called with an invalid, prototype PTE, with the PFN lock held */
//     MI_ASSERT_PFN_LOCK_HELD();
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//     ASSERT(PointerPte->u.Soft.Prototype == 1);
//
//     /* Read the prototype PTE and check if it's valid */
//     TempPte = *PointerProtoPte;
//     if ((TempPte.valid() as usize) == 1)
//     {
//         /* One more user of this mapped page */
//         PageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
//         Pfn1 = MiGetPfnEntry(PageFrameIndex);
//         Pfn1->u2.ShareCount++;
//
//         /* Call it a transition */
//         InterlockedIncrement(&KeGetCurrentPrcb()->MmTransitionCount);
//
//         /* Complete the prototype PTE fault -- this will release the PFN lock */
//         return MiCompleteProtoPteFault(StoreInstruction,
//                                        Address,
//                                        PointerPte,
//                                        PointerProtoPte,
//                                        OldIrql,
//                                        OutPfn);
//     }
//
//     /* Make sure there's some protection mask */
//     if (TempPte.Long == 0)
//     {
//         /* Release the lock */
//         DPRINT1("Access on reserved section?\n");
//         MiReleasePfnLock(OldIrql);
//         return STATUS_ACCESS_VIOLATION;
//     }
//
//     /* There is no such thing as a decommitted prototype PTE */
//     ASSERT(TempPte.Long != MmDecommittedPte.Long);
//
//     /* Check for access rights on the PTE proper */
//     PteContents = *PointerPte;
//     if (PteContents.soft_page_file_high() != MI_PTE_LOOKUP_NEEDED)
//     {
//         if (!PteContents.u.Proto.ReadOnly)
//         {
//             Protection = TempPte.soft_protection();
//         }
//         else
//         {
//             Protection = MM_READONLY;
//         }
//         /* Check for page acess in software */
//         Status = MiAccessCheck(PointerProtoPte,
//                                StoreInstruction,
//                                KernelMode,
//                                TempPte.soft_protection(),
//                                TrapInformation,
//                                TRUE);
//         ASSERT(Status == STATUS_SUCCESS);
//     }
//     else
//     {
//         Protection = PteContents.soft_protection();
//     }
//
//     /* Check for writing copy on write page */
//     if (((Protection & MM_WRITECOPY) == MM_WRITECOPY) && StoreInstruction)
//     {
//         PFN_NUMBER PageFrameIndex, ProtoPageFrameIndex;
//         ULONG Color;
//
//         /* Resolve the proto fault as if it was a read operation */
//         Status = MiResolveProtoPteFault(FALSE,
//                                         Address,
//                                         PointerPte,
//                                         PointerProtoPte,
//                                         OutPfn,
//                                         PageFileData,
//                                         PteValue,
//                                         Process,
//                                         OldIrql,
//                                         TrapInformation);
//
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         /* Lock again the PFN lock, MiResolveProtoPteFault unlocked it */
//         OldIrql = MiAcquirePfnLock();
//
//         /* And re-read the proto PTE */
//         TempPte = *PointerProtoPte;
//         ASSERT((TempPte.valid() as usize) == 1);
//         ProtoPageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
//
//         MI_SET_USAGE(MI_USAGE_COW);
//         MI_SET_PROCESS(Process);
//
//         /* Get a new page for the private copy */
//         if (Process > HYDRA_PROCESS)
//             Color = MI_GET_NEXT_PROCESS_COLOR(Process);
//         else
//             Color = MI_GET_NEXT_COLOR();
//
//         PageFrameIndex = MiRemoveAnyPage(Color);
//         if (PageFrameIndex == 0)
//         {
//             MiReleasePfnLock(OldIrql);
//             return STATUS_NO_MEMORY;
//         }
//
//         /* Perform the copy */
//         MiCopyPfn(PageFrameIndex, ProtoPageFrameIndex);
//
//         /* This will drop everything MiResolveProtoPteFault referenced */
//         MiDeletePte(PointerPte as *mut _MMPTE, Address, Process, PointerProtoPte as *mut _MMPTE);
//
//         /* Because now we use this */
//         Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//         MiInitializePfn(PageFrameIndex, PointerPte, TRUE);
//
//         /* Fix the protection */
//         Protection &= ~MM_WRITECOPY;
//         Protection |= MM_READWRITE;
//         if (Address < MmSystemRangeStart)
//         {
//             /* Build the user PTE */
//             MI_MAKE_HARDWARE_PTE_USER(&PteContents, PointerPte, Protection, PageFrameIndex);
//         }
//         else
//         {
//             /* Build the kernel PTE */
//             MI_MAKE_HARDWARE_PTE(&PteContents, PointerPte, Protection, PageFrameIndex);
//         }
//
//         /* And finally, write the valid PTE */
//         MI_WRITE_VALID_PTE(PointerPte, PteContents);
//
//         /* The caller expects us to release the PFN lock */
//         MiReleasePfnLock(OldIrql);
//         return Status;
//     }
//
//     /* Check for clone PTEs */
//     if (PointerPte <= MiHighestUserPte) ASSERT(Process->CloneRoot == NULL);
//
//     /* We don't support mapped files yet */
//     ASSERT((TempPte.prototype() as usize) == 0);
//
//     /* We might however have transition PTEs */
//     if ((TempPte.transition() as usize) == 1)
//     {
//         /* Resolve the transition fault */
//         ASSERT(OldIrql != MM_NOIRQL);
//         Status = MiResolveTransitionFault(StoreInstruction,
//                                           Address,
//                                           PointerProtoPte,
//                                           Process,
//                                           OldIrql,
//                                           &InPageBlock);
//         ASSERT(NT_SUCCESS(Status));
//     }
//     else
//     {
//         /* We also don't support paged out pages */
//         ASSERT(TempPte.soft_page_file_high() == 0);
//
//         /* Resolve the demand zero fault */
//         Status = MiResolveDemandZeroFault(Address,
//                                           PointerProtoPte,
//                                           (ULONG)TempPte.soft_protection(),
//                                           Process,
//                                           OldIrql);
// #if MI_TRACE_PFNS
//         /* Update debug info */
//         if (TrapInformation)
//             MiGetPfnEntry(PointerProtoPte->u.Hard.PageFrameNumber)->CallSite = (PVOID)((PKTRAP_FRAME)TrapInformation)->Eip;
//         else
//             MiGetPfnEntry(PointerProtoPte->u.Hard.PageFrameNumber)->CallSite = _ReturnAddress();
// #endif
//                                       
//         ASSERT(NT_SUCCESS(Status));
//     }
//
//     /* Complete the prototype PTE fault -- this will release the PFN lock */
//     ASSERT(PointerPte->u.Hard.Valid == 0);
//     return MiCompleteProtoPteFault(StoreInstruction,
//                                    Address,
//                                    PointerPte,
//                                    PointerProtoPte,
//                                    OldIrql,
//                                    OutPfn);
// }
//
// NTSTATUS
// NTAPI
// MiDispatchFault(IN ULONG FaultCode,
//                 IN PVOID Address,
//                 IN PMMPTE PointerPte,
//                 IN PMMPTE PointerProtoPte,
//                 IN BOOLEAN Recursive,
//                 IN PEPROCESS Process,
//                 IN PVOID TrapInformation,
//                 IN PMMVAD Vad)
// {
//     MMPTE TempPte;
//     KIRQL OldIrql, LockIrql;
//     NTSTATUS Status;
//     PMMPTE SuperProtoPte;
//     PMMPFN Pfn1, OutPfn = NULL;
//     PFN_NUMBER PageFrameIndex;
//     PFN_COUNT PteCount, ProcessedPtes;
//     DPRINT("ARM3 Page Fault Dispatcher for address: %p in process: %p\n",
//              Address,
//              Process);
//
//     /* Make sure the addresses are ok */
//     ASSERT(PointerPte == MiAddressToPte(Address));
//
//     //
//     // Make sure APCs are off and we're not at dispatch
//     //
//     OldIrql = KeGetCurrentIrql();
//     ASSERT(OldIrql <= APC_LEVEL);
//     ASSERT(KeAreAllApcsDisabled() == TRUE);
//
//     //
//     // Grab a copy of the PTE
//     //
//     TempPte = *PointerPte;
//
//     /* Do we have a prototype PTE? */
//     if (PointerProtoPte)
//     {
//         /* This should never happen */
//         ASSERT(!MI_IS_PHYSICAL_ADDRESS(PointerProtoPte));
//
//         /* Check if this is a kernel-mode address */
//         SuperProtoPte = MiAddressToPte(PointerProtoPte);
//         if (Address >= MmSystemRangeStart)
//         {
//             /* Lock the PFN database */
//             LockIrql = MiAcquirePfnLock();
//
//             /* Has the PTE been made valid yet? */
//             if (!SuperProtoPte->u.Hard.Valid)
//             {
//                 ASSERT(FALSE);
//             }
//             else if (PointerPte->u.Hard.Valid == 1)
//             {
//                 ASSERT(FALSE);
//             }
//
//             /* Resolve the fault -- this will release the PFN lock */
//             Status = MiResolveProtoPteFault(!MI_IS_NOT_PRESENT_FAULT(FaultCode),
//                                             Address,
//                                             PointerPte,
//                                             PointerProtoPte,
//                                             &OutPfn,
//                                             NULL,
//                                             NULL,
//                                             Process,
//                                             LockIrql,
//                                             TrapInformation);
//             ASSERT(Status == STATUS_SUCCESS);
//
//             /* Complete this as a transition fault */
//             ASSERT(OldIrql == KeGetCurrentIrql());
//             ASSERT(OldIrql <= APC_LEVEL);
//             ASSERT(KeAreAllApcsDisabled() == TRUE);
//             return Status;
//         }
//         else
//         {
//             /* We only handle the lookup path */
//             ASSERT(PointerPte->u.Soft.PageFileHigh == MI_PTE_LOOKUP_NEEDED);
//
//             /* Is there a non-image VAD? */
//             if ((Vad) &&
//                 (Vad->u.VadFlags.VadType() != VadImageMap) &&
//                 !(Vad->u2.VadFlags2.ExtendableFile()))
//             {
//                 /* One day, ReactOS will cluster faults */
//                 ASSERT(Address <= MM_HIGHEST_USER_ADDRESS);
//                 DPRINT("Should cluster fault, but won't\n");
//             }
//
//             /* Only one PTE to handle for now */
//             PteCount = 1;
//             ProcessedPtes = 0;
//
//             /* Lock the PFN database */
//             LockIrql = MiAcquirePfnLock();
//
//             /* We only handle the valid path */
//             ASSERT(SuperProtoPte->u.Hard.Valid == 1);
//
//             /* Capture the PTE */
//             TempPte = *PointerProtoPte;
//
//             /* Loop to handle future case of clustered faults */
//             while (TRUE)
//             {
//                 /* For our current usage, this should be true */
//                 if ((TempPte.valid() as usize) == 1)
//                 {
//                     /* Bump the share count on the PTE */
//                     PageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
//                     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//                     Pfn1->u2.ShareCount++;
//                 }
//                 else if (((TempPte.prototype() as usize) == 0) &&
//                          ((TempPte.transition() as usize) == 1))
//                 {
//                     /* This is a standby page, bring it back from the cache */
//                     PageFrameIndex = TempPte.page_frame_number();
//                     DPRINT("oooh, shiny, a soft fault! 0x%lx\n", PageFrameIndex);
//                     Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//                     ASSERT(Pfn1->u3.e1.PageLocation != ActiveAndValid);
//
//                     /* Should not yet happen in ReactOS */
//                     ASSERT(Pfn1->u3.e1.ReadInProgress == 0);
//                     ASSERT(Pfn1->u4.InPageError == 0);
//
//                     /* Get the page */
//                     MiUnlinkPageFromList(Pfn1);
//
//                     /* Bump its reference count */
//                     ASSERT(Pfn1->u2.ShareCount == 0);
//                     InterlockedIncrement16((PSHORT)&Pfn1->u3.e2.ReferenceCount);
//                     Pfn1->u2.ShareCount++;
//
//                     /* Make it valid again */
//                     /* This looks like another macro.... */
//                     Pfn1->u3.e1.PageLocation = ActiveAndValid;
//                     ASSERT(PointerProtoPte->u.Hard.Valid == 0);
//                     ASSERT(PointerProtoPte->u.Trans.Prototype == 0);
//                     ASSERT(PointerProtoPte->u.Trans.Transition == 1);
//                     TempPte.Long = (PointerProtoPte->u.Long & ~0xFFF) |
//                                      MmProtectToPteMask[PointerProtoPte->u.Trans.Protection()];
//                     (TempPte.valid() as usize) = 1;
//                     MI_MAKE_ACCESSED_PAGE(&TempPte);
//
//                     /* Is the PTE writeable? */
//                     if ((Pfn1->u3.e1.Modified) &&
//                         MI_IS_PAGE_WRITEABLE(&TempPte) &&
//                         !MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//                     {
//                         /* Make it dirty */
//                         MI_MAKE_DIRTY_PAGE(&TempPte);
//                     }
//                     else
//                     {
//                         /* Make it clean */
//                         MI_MAKE_CLEAN_PAGE(&TempPte);
//                     }
//
//                     /* Write the valid PTE */
//                     MI_WRITE_VALID_PTE(PointerProtoPte, TempPte);
//                     ASSERT(PointerPte->u.Hard.Valid == 0);
//                 }
//                 else
//                 {
//                     /* Page is invalid, get out of the loop */
//                     break;
//                 }
//
//                 /* One more done, was it the last? */
//                 if (++ProcessedPtes == PteCount)
//                 {
//                     /* Complete the fault */
//                     MiCompleteProtoPteFault(!MI_IS_NOT_PRESENT_FAULT(FaultCode),
//                                             Address,
//                                             PointerPte,
//                                             PointerProtoPte,
//                                             LockIrql,
//                                             &OutPfn);
//
//                     /* THIS RELEASES THE PFN LOCK! */
//                     break;
//                 }
//
//                 /* No clustered faults yet */
//                 ASSERT(FALSE);
//             }
//
//             /* Did we resolve the fault? */
//             if (ProcessedPtes)
//             {
//                 /* Bump the transition count */
//                 InterlockedExchangeAddSizeT(&KeGetCurrentPrcb()->MmTransitionCount, ProcessedPtes);
//                 ProcessedPtes--;
//
//                 /* Loop all the processing we did */
//                 ASSERT(ProcessedPtes == 0);
//
//                 /* Complete this as a transition fault */
//                 ASSERT(OldIrql == KeGetCurrentIrql());
//                 ASSERT(OldIrql <= APC_LEVEL);
//                 ASSERT(KeAreAllApcsDisabled() == TRUE);
//                 return STATUS_PAGE_FAULT_TRANSITION;
//             }
//
//             /* We did not -- PFN lock is still held, prepare to resolve prototype PTE fault */
//             OutPfn = MI_PFN_ELEMENT(SuperProtoPte->u.Hard.PageFrameNumber);
//             MiReferenceUsedPageAndBumpLockCount(OutPfn);
//             ASSERT(OutPfn->u3.e2.ReferenceCount > 1);
//             ASSERT(PointerPte->u.Hard.Valid == 0);
//
//             /* Resolve the fault -- this will release the PFN lock */
//             Status = MiResolveProtoPteFault(!MI_IS_NOT_PRESENT_FAULT(FaultCode),
//                                             Address,
//                                             PointerPte,
//                                             PointerProtoPte,
//                                             &OutPfn,
//                                             NULL,
//                                             NULL,
//                                             Process,
//                                             LockIrql,
//                                             TrapInformation);
//             //ASSERT(Status != STATUS_ISSUE_PAGING_IO);
//             //ASSERT(Status != STATUS_REFAULT);
//             //ASSERT(Status != STATUS_PTE_CHANGED);
//
//             /* Did the routine clean out the PFN or should we? */
//             if (OutPfn)
//             {
//                 /* We had a locked PFN, so acquire the PFN lock to dereference it */
//                 ASSERT(PointerProtoPte != NULL);
//                 OldIrql = MiAcquirePfnLock();
//
//                 /* Dereference the locked PFN */
//                 MiDereferencePfnAndDropLockCount(OutPfn);
//                 ASSERT(OutPfn->u3.e2.ReferenceCount >= 1);
//
//                 /* And now release the lock */
//                 MiReleasePfnLock(OldIrql);
//             }
//
//             /* Complete this as a transition fault */
//             ASSERT(OldIrql == KeGetCurrentIrql());
//             ASSERT(OldIrql <= APC_LEVEL);
//             ASSERT(KeAreAllApcsDisabled() == TRUE);
//             return Status;
//         }
//     }
//
//     /* Is this a transition PTE */
//     if ((TempPte.transition() as usize))
//     {
//         PKEVENT* InPageBlock = NULL;
//         PKEVENT PreviousPageEvent;
//         KEVENT CurrentPageEvent;
//
//         /* Lock the PFN database */
//         LockIrql = MiAcquirePfnLock();
//
//         /* Resolve */
//         Status = MiResolveTransitionFault(!MI_IS_NOT_PRESENT_FAULT(FaultCode), Address, PointerPte, Process, LockIrql, &InPageBlock);
//
//         ASSERT(NT_SUCCESS(Status));
//
//         if (InPageBlock != NULL)
//         {
//             /* Another thread is reading or writing this page. Put us into the waiting queue. */
//             KeInitializeEvent(&CurrentPageEvent, NotificationEvent, FALSE);
//             PreviousPageEvent = *InPageBlock;
//             *InPageBlock = &CurrentPageEvent;
//         }
//
//         /* And now release the lock and leave*/
//         MiReleasePfnLock(LockIrql);
//
//         if (InPageBlock != NULL)
//         {
//             KeWaitForSingleObject(&CurrentPageEvent, WrPageIn, KernelMode, FALSE, NULL);
//
//             /* Let's the chain go on */
//             if (PreviousPageEvent)
//             {
//                 KeSetEvent(PreviousPageEvent, IO_NO_INCREMENT, FALSE);
//             }
//         }
//
//         ASSERT(OldIrql == KeGetCurrentIrql());
//         ASSERT(OldIrql <= APC_LEVEL);
//         ASSERT(KeAreAllApcsDisabled() == TRUE);
//         return Status;
//     }
//
//     /* Should we page the data back in ? */
//     if (TempPte.soft_page_file_high() != 0)
//     {
//         /* Lock the PFN database */
//         LockIrql = MiAcquirePfnLock();
//
//         /* Resolve */
//         Status = MiResolvePageFileFault(!MI_IS_NOT_PRESENT_FAULT(FaultCode), Address, PointerPte, Process, &LockIrql);
//
//         /* And now release the lock and leave*/
//         MiReleasePfnLock(LockIrql);
//
//         ASSERT(OldIrql == KeGetCurrentIrql());
//         ASSERT(OldIrql <= APC_LEVEL);
//         ASSERT(KeAreAllApcsDisabled() == TRUE);
//         return Status;
//     }
//
//     //
//     // The PTE must be invalid but not completely empty. It must also not be a
//     // prototype a transition or a paged-out PTE as those scenarii should've been handled above.
//     // These are all Windows checks
//     //
//     ASSERT((TempPte.valid() as usize) == 0);
//     ASSERT((TempPte.prototype() as usize) == 0);
//     ASSERT((TempPte.transition() as usize) == 0);
//     ASSERT(TempPte.soft_page_file_high() == 0);
//     ASSERT(TempPte.Long != 0);
//
//     //
//     // If we got this far, the PTE can only be a demand zero PTE, which is what
//     // we want. Go handle it!
//     //
//     Status = MiResolveDemandZeroFault(Address,
//                                       PointerPte,
//                                       (ULONG)TempPte.soft_protection(),
//                                       Process,
//                                       MM_NOIRQL);
//     ASSERT(KeAreAllApcsDisabled() == TRUE);
//     if (NT_SUCCESS(Status))
//     {
// #if MI_TRACE_PFNS
//         /* Update debug info */
//         if (TrapInformation)
//             MiGetPfnEntry(PointerPte->u.Hard.PageFrameNumber)->CallSite = (PVOID)((PKTRAP_FRAME)TrapInformation)->Eip;
//         else
//             MiGetPfnEntry(PointerPte->u.Hard.PageFrameNumber)->CallSite = _ReturnAddress();
// #endif
//
//         //
//         // Make sure we're returning in a sane state and pass the status down
//         //
//         ASSERT(OldIrql == KeGetCurrentIrql());
//         ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//         return Status;
//     }
//
//     //
//     // Return status
//     //
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// MmArmAccessFault(IN ULONG FaultCode,
//                  IN PVOID Address,
//                  IN KPROCESSOR_MODE Mode,
//                  IN PVOID TrapInformation)
// {
//     KIRQL OldIrql = KeGetCurrentIrql(), LockIrql;
//     PMMPTE ProtoPte = NULL;
//     PMMPTE PointerPte = MiAddressToPte(Address);
//     PMMPDE PointerPde = MiAddressToPde(Address);
// #if (_MI_PAGING_LEVELS >= 3)
//     PMMPDE PointerPpe = MiAddressToPpe(Address);
// #if (_MI_PAGING_LEVELS == 4)
//     PMMPDE PointerPxe = MiAddressToPxe(Address);
// #endif
// #endif
//     MMPTE TempPte;
//     PETHREAD CurrentThread;
//     PEPROCESS CurrentProcess;
//     NTSTATUS Status;
//     PMMSUPPORT WorkingSet;
//     ULONG ProtectionCode;
//     PMMVAD Vad = NULL;
//     PFN_NUMBER PageFrameIndex;
//     ULONG Color;
//     BOOLEAN IsSessionAddress;
//     PMMPFN Pfn1;
//     DPRINT("ARM3 FAULT AT: %p\n", Address);
//
//     /* Check for page fault on high IRQL */
//     if (OldIrql > APC_LEVEL)
//     {
// #if (_MI_PAGING_LEVELS < 3)
//         /* Could be a page table for paged pool, which we'll allow */
//         if (MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address)) MiSynchronizeSystemPde((PMMPDE)PointerPte);
//         MiCheckPdeForPagedPool(Address);
// #endif
//         /* Check if any of the top-level pages are invalid */
//         if (
// #if (_MI_PAGING_LEVELS == 4)
//             (PointerPxe->u.Hard.Valid == 0) ||
// #endif
// #if (_MI_PAGING_LEVELS >= 3)
//             (PointerPpe->u.Hard.Valid == 0) ||
// #endif
//             (PointerPde->u.Hard.Valid == 0) ||
//             (PointerPte->u.Hard.Valid == 0))
//         {
//             /* This fault is not valid, print out some debugging help */
//             DbgPrint("MM:***PAGE FAULT AT IRQL > 1  Va %p, IRQL %lx\n",
//                      Address,
//                      OldIrql);
//             if (TrapInformation)
//             {
//                 PKTRAP_FRAME TrapFrame = TrapInformation;
// #ifdef _M_IX86
//                 DbgPrint("MM:***EIP %p, EFL %p\n", TrapFrame->Eip, TrapFrame->EFlags);
//                 DbgPrint("MM:***EAX %p, ECX %p EDX %p\n", TrapFrame->Eax, TrapFrame->Ecx, TrapFrame->Edx);
//                 DbgPrint("MM:***EBX %p, ESI %p EDI %p\n", TrapFrame->Ebx, TrapFrame->Esi, TrapFrame->Edi);
// #elif defined(_M_AMD64)
//                 DbgPrint("MM:***RIP %p, EFL %p\n", TrapFrame->Rip, TrapFrame->EFlags);
//                 DbgPrint("MM:***RAX %p, RCX %p RDX %p\n", TrapFrame->Rax, TrapFrame->Rcx, TrapFrame->Rdx);
//                 DbgPrint("MM:***RBX %p, RSI %p RDI %p\n", TrapFrame->Rbx, TrapFrame->Rsi, TrapFrame->Rdi);
// #elif defined(_M_ARM)
//                 DbgPrint("MM:***PC %p\n", TrapFrame->Pc);
//                 DbgPrint("MM:***R0 %p, R1 %p R2 %p, R3 %p\n", TrapFrame->R0, TrapFrame->R1, TrapFrame->R2, TrapFrame->R3);
//                 DbgPrint("MM:***R11 %p, R12 %p SP %p, LR %p\n", TrapFrame->R11, TrapFrame->R12, TrapFrame->Sp, TrapFrame->Lr);
// #elif defined(_M_ARM64)
//                 DbgPrint("MM:***PC %p, LR %p, SP %p\n", (PVOID)TrapFrame->Pc, (PVOID)TrapFrame->Lr, (PVOID)TrapFrame->Sp);
//                 DbgPrint("MM:***X0 %p, X1 %p, X2 %p, X3 %p\n", (PVOID)TrapFrame->X[0], (PVOID)TrapFrame->X[1], (PVOID)TrapFrame->X[2], (PVOID)TrapFrame->X[3]);
// #endif
//             }
//
//             /* Tell the trap handler to fail */
//             return STATUS_IN_PAGE_ERROR | 0x10000000;
//         }
//
//         /* Not yet implemented in ReactOS */
//         ASSERT(MI_IS_PAGE_LARGE(PointerPde) == FALSE);
//         ASSERT((!MI_IS_NOT_PRESENT_FAULT(FaultCode) && MI_IS_PAGE_COPY_ON_WRITE(PointerPte)) == FALSE);
//
//         /* Check if this was a write */
//         if (MI_IS_WRITE_ACCESS(FaultCode))
//         {
//             /* Was it to a read-only page? */
//             Pfn1 = MI_PFN_ELEMENT(PointerPte->u.Hard.PageFrameNumber);
//             if (!(PointerPte->u.Long & PTE_READWRITE) &&
//                 !(Pfn1->OriginalPte.soft_protection() & MM_READWRITE))
//             {
//                 /* Crash with distinguished bugcheck code */
//                 KeBugCheckEx(ATTEMPTED_WRITE_TO_READONLY_MEMORY,
//                              (ULONG_PTR)Address,
//                              PointerPte->u.Long,
//                              (ULONG_PTR)TrapInformation,
//                              10);
//             }
//         }
//
//         /* Nothing is actually wrong */
//         DPRINT1("Fault at IRQL %u is ok (%p)\n", OldIrql, Address);
//         return STATUS_SUCCESS;
//     }
//
//     /* Check for kernel fault address */
//     if (Address >= MmSystemRangeStart)
//     {
//         /* Bail out, if the fault came from user mode */
//         if (Mode == UserMode) return STATUS_ACCESS_VIOLATION;
//
// #if (_MI_PAGING_LEVELS == 2)
//         if (MI_IS_SYSTEM_PAGE_TABLE_ADDRESS(Address)) MiSynchronizeSystemPde((PMMPDE)PointerPte);
//         MiCheckPdeForPagedPool(Address);
// #endif
//
//         /* Trace ALL kernel page faults for hang debugging */
// #ifdef _M_ARM64
//         ARM64_UART_TRACE_HEX("[KPF] ", (ULONG_PTR)Address);
// #endif
//
//         /* ARM64 4-level self-map fault handling.
//          *
//          * When kernel code reads a PDE/PPE/PTE through the self-map for a user
//          * VA whose PXE or PPE is zero, the hardware page table walk for the
//          * self-map address fails. The self-map walk uses PXE[0x1ED] as a
//          * recursive entry, so a PDE self-map access walks:
//          *   PML4[0x1ED] -> PML4[0x1ED] -> PML4[user_pxe] -> PPE[user_ppe] -> PDE
//          * If PML4[user_pxe] = 0, the walk faults here.
//          *
//          * We resolve this by directly allocating the missing page table page(s)
//          * and populating the PXE/PPE entry. This is safe without the WS lock
//          * because we only create the page table structure — no PTE/data pages. */
// #if (_MI_PAGING_LEVELS == 4)
//         if (MI_IS_PAGE_TABLE_OR_HYPER_ADDRESS(Address))
//         {
//             PVOID UnderlyingVa;
//             PMMPTE UserPxe, UserPpe;
//             PFN_NUMBER NewPage;
//             MMPTE NewPte;
//             KIRQL PfnIrql;
//
//             /* Decode self-map address to underlying VA.
//              * IMPORTANT: check innermost ranges first — they overlap
//              * (PDE_TOP > PPE_BASE > PXE_BASE). */
//             ARM64_UART_TRACE_HEX("[SMAP] ", (ULONG_PTR)Address);
//             if ((ULONG_PTR)Address >= PXE_BASE && (ULONG_PTR)Address <= PXE_TOP)
//             {
//                 /* PXE range is the PML4 page itself — shouldn't fault */
//                 goto SelfMapKernel;
//             }
//             else if ((ULONG_PTR)Address >= PPE_BASE)
//             {
//                 UnderlyingVa = MiPpeToAddress((PMMPTE)Address);
//             }
//             else if ((ULONG_PTR)Address >= PDE_BASE)
//             {
//                 UnderlyingVa = MiPdeToAddress((PMMPTE)Address);
//             }
//             else
//             {
//                 UnderlyingVa = MiPteToAddress((PMMPTE)Address);
//             }
//
//             if (UnderlyingVa >= MmSystemRangeStart)
//             {
// SelfMapKernel:
//                 /* Kernel page table fault — use existing handler */
//                 goto UserFault;
//             }
//
//             /* HACK: Directly allocate missing page table page(s) for user VA.
//              * This is a simplified version of what UserFault does for PXE/PPE
//              * allocation, but without requiring the working set lock (which may
//              * already be held by the caller). PFN entries are minimally
//              * initialized — proper tracking will be added later. */
//
//             /* PXE is always readable (PXE_BASE = PML4 page, 4x self-ref) */
//             UserPxe = MiAddressToPxe(UnderlyingVa);
//             if (UserPxe->u.Hard.Valid == 0)
//             {
//                 ARM64_UART_TRACE("[SMAP] +PXE\r\n");
//                 /* Allocate a zeroed page for the PPE table */
//                 PfnIrql = MiAcquirePfnLock();
//                 NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
//                 if (NewPage == 0)
//                 {
//                     MiReleasePfnLock(PfnIrql);
//                     return STATUS_NO_MEMORY;
//                 }
//                 /* Minimal PFN setup so the page isn't reused */
//                 MiInitializePfn(NewPage, UserPxe, TRUE);
//                 MiReleasePfnLock(PfnIrql);
//
//                 /* Build PXE entry: kernel PTE with RW for page table page */
//                 MI_MAKE_HARDWARE_PTE(&NewPte, UserPxe,
//                                      MM_EXECUTE_READWRITE, NewPage);
//                 if (MI_IS_PAGE_WRITEABLE(&NewPte)) MI_MAKE_DIRTY_PAGE(&NewPte);
//                 MI_WRITE_VALID_PTE(UserPxe, NewPte);
//             }
//
//             /* Now PXE is valid — PPE self-map walk succeeds */
//             UserPpe = MiAddressToPpe(UnderlyingVa);
//             if (UserPpe->u.Hard.Valid == 0)
//             {
//                 ARM64_UART_TRACE("[SMAP] +PPE\r\n");
//                 /* Allocate a zeroed page for the PDE table */
//                 PfnIrql = MiAcquirePfnLock();
//                 NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
//                 if (NewPage == 0)
//                 {
//                     MiReleasePfnLock(PfnIrql);
//                     return STATUS_NO_MEMORY;
//                 }
//                 MiInitializePfn(NewPage, UserPpe, TRUE);
//
//                 /* Increment L1 page (PXE table) refcount — this PPE entry
//                  * occupies a slot in the PXE-pointed L1 page. MiDeletePde's
//                  * cascade calls MiDecrementPageTableReferences(PointerPde)
//                  * which decrements this same refcount. */
//                 {
//                     PMMPFN L1Pfn = MiGetPfnEntry(PFN_FROM_PXE(UserPxe));
//                     L1Pfn->OriginalPte.u.Soft.UsedPageTableEntries++;
//                 }
//                 MiReleasePfnLock(PfnIrql);
//
//                 MI_MAKE_HARDWARE_PTE(&NewPte, UserPpe,
//                                      MM_EXECUTE_READWRITE, NewPage);
//                 if (MI_IS_PAGE_WRITEABLE(&NewPte)) MI_MAKE_DIRTY_PAGE(&NewPte);
//                 MI_WRITE_VALID_PTE(UserPpe, NewPte);
//             }
//
//             /* Now PXE and PPE are valid — PDE self-map walk succeeds.
//              * For PTE-range faults, we also need the PDE (PTE page). */
//             {
//                 PMMPTE UserPde = MiAddressToPde(UnderlyingVa);
//                 if (UserPde->u.Hard.Valid == 0)
//                 {
//                     ARM64_UART_TRACE("[SMAP] +PDE\r\n");
//                     /* Allocate a zeroed page for the PTE table */
//                     PfnIrql = MiAcquirePfnLock();
//                     NewPage = MiRemoveZeroPage(MI_GET_NEXT_COLOR());
//                     if (NewPage == 0)
//                     {
//                         MiReleasePfnLock(PfnIrql);
//                         return STATUS_NO_MEMORY;
//                     }
//                     MiInitializePfn(NewPage, UserPde, TRUE);
//
//                     /* Increment L2 page (PPE table) refcount — this PDE entry
//                      * occupies a slot in the PPE-pointed L2 page. MiDeletePde's
//                      * cascade calls MiDecrementPageTableReferences(MiPdeToPte(PDE))
//                      * which decrements this same refcount through the self-map. */
//                     {
//                         PMMPFN L2Pfn = MiGetPfnEntry(PFN_FROM_PPE(UserPpe));
//                         L2Pfn->OriginalPte.u.Soft.UsedPageTableEntries++;
//                     }
//                     MiReleasePfnLock(PfnIrql);
//
//                     MI_MAKE_HARDWARE_PTE(&NewPte, UserPde,
//                                          MM_EXECUTE_READWRITE, NewPage);
//                     if (MI_IS_PAGE_WRITEABLE(&NewPte)) MI_MAKE_DIRTY_PAGE(&NewPte);
//                     MI_WRITE_VALID_PTE(UserPde, NewPte);
//                 }
//             }
//
//             /* Self-map address should now be accessible */
//             ARM64_UART_TRACE("[SMAP] OK\r\n");
//             return STATUS_SUCCESS;
//         }
// #endif
//
//         /* Check if the higher page table entries are invalid */
//         if (
// #if (_MI_PAGING_LEVELS == 4)
//             /* AMD64 system, check if PXE is invalid */
//             (PointerPxe->u.Hard.Valid == 0) ||
// #endif
// #if (_MI_PAGING_LEVELS >= 3)
//             /* PAE/AMD64 system, check if PPE is invalid */
//             (PointerPpe->u.Hard.Valid == 0) ||
// #endif
//             /* Always check if the PDE is valid */
//             (PointerPde->u.Hard.Valid == 0))
//         {
//             /* PXE/PPE/PDE (still) not valid, kill the system */
//             KeBugCheckEx(PAGE_FAULT_IN_NONPAGED_AREA,
//                          (ULONG_PTR)Address,
//                          FaultCode,
//                          (ULONG_PTR)TrapInformation,
//                          2);
//         }
//
//         /* Not handling session faults yet */
//         IsSessionAddress = MI_IS_SESSION_ADDRESS(Address);
//
//         /* The PDE is valid, so read the PTE */
//         TempPte = *PointerPte;
//         if ((TempPte.valid() as usize) == 1)
//         {
//             /* Check if this was system space or session space */
//             if (!IsSessionAddress)
//             {
//                 /* Check if the PTE is still valid under PFN lock */
//                 OldIrql = MiAcquirePfnLock();
//                 TempPte = *PointerPte;
//                 if ((TempPte.valid() as usize))
//                 {
//                     /* Check if this was a write */
//                     if (MI_IS_WRITE_ACCESS(FaultCode))
//                     {
//                         /* Was it to a read-only page? */
//                         Pfn1 = MI_PFN_ELEMENT(PointerPte->u.Hard.PageFrameNumber);
//                         if (!(PointerPte->u.Long & PTE_READWRITE) &&
//                             !(Pfn1->OriginalPte.soft_protection() & MM_READWRITE))
//                         {
//                             /* Crash with distinguished bugcheck code */
//                             KeBugCheckEx(ATTEMPTED_WRITE_TO_READONLY_MEMORY,
//                                          (ULONG_PTR)Address,
//                                          PointerPte->u.Long,
//                                          (ULONG_PTR)TrapInformation,
//                                          11);
//                         }
//                     }
//
//                     /* Check for execution of non-executable memory */
//                     if (MI_IS_INSTRUCTION_FETCH(FaultCode) &&
//                         !MI_IS_PAGE_EXECUTABLE(&TempPte))
//                     {
//                         KeBugCheckEx(ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY,
//                                      (ULONG_PTR)Address,
//                                      (ULONG_PTR)TempPte.Long,
//                                      (ULONG_PTR)TrapInformation,
//                                      1);
//                     }
//                 }
//
//                 /* Release PFN lock and return all good */
//                 MiReleasePfnLock(OldIrql);
//                 return STATUS_SUCCESS;
//             }
//         }
// #if (_MI_PAGING_LEVELS == 2)
//         /* Check if this was a session PTE that needs to remap the session PDE */
//         if (MI_IS_SESSION_PTE(Address as PMMPTE))
//         {
//             /* Do the remapping */
//             Status = MiCheckPdeForSessionSpace(Address);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* It failed, this address is invalid */
//                 KeBugCheckEx(PAGE_FAULT_IN_NONPAGED_AREA,
//                              (ULONG_PTR)Address,
//                              FaultCode,
//                              (ULONG_PTR)TrapInformation,
//                              6);
//             }
//         }
// #else
//
// _WARN("Session space stuff is not implemented yet!")
//
// #endif
//
//         /* Check for a fault on the page table or hyperspace */
//         if (MI_IS_PAGE_TABLE_OR_HYPER_ADDRESS(Address))
//         {
// #if (_MI_PAGING_LEVELS < 3)
//             /* Windows does this check but I don't understand why -- it's done above! */
//             ASSERT(MiCheckPdeForPagedPool(Address) != STATUS_WAIT_1);
// #endif
//             /* Handle this as a user mode fault */
//             goto UserFault;
//         }
//
//         /* Get the current thread */
//         CurrentThread = PsGetCurrentThread();
//
//         /* What kind of address is this */
//         if (!IsSessionAddress)
//         {
//             /* Use the system working set */
//             WorkingSet = &MmSystemCacheWs;
//             CurrentProcess = NULL;
//
//             /* Make sure we don't have a recursive working set lock */
//             if ((CurrentThread->OwnsProcessWorkingSetExclusive) ||
//                 (CurrentThread->OwnsProcessWorkingSetShared) ||
//                 (CurrentThread->OwnsSystemWorkingSetExclusive) ||
//                 (CurrentThread->OwnsSystemWorkingSetShared) ||
//                 (CurrentThread->OwnsSessionWorkingSetExclusive) ||
//                 (CurrentThread->OwnsSessionWorkingSetShared))
//             {
//                 /* Fail */
//                 return STATUS_IN_PAGE_ERROR | 0x10000000;
//             }
//         }
//         else
//         {
//             /* Use the session process and working set */
//             CurrentProcess = HYDRA_PROCESS;
//             WorkingSet = &MmSessionSpace->GlobalVirtualAddress->Vm;
//
//             /* Make sure we don't have a recursive working set lock */
//             if ((CurrentThread->OwnsSessionWorkingSetExclusive) ||
//                 (CurrentThread->OwnsSessionWorkingSetShared))
//             {
//                 /* Fail */
//                 return STATUS_IN_PAGE_ERROR | 0x10000000;
//             }
//         }
// RetryKernel:
//         /* Acquire the working set lock */
//         KeRaiseIrql(APC_LEVEL, &LockIrql);
//         MiLockWorkingSet(CurrentThread, WorkingSet);
//
//         /* Re-read PTE now that we own the lock */
//         TempPte = *PointerPte;
//         if ((TempPte.valid() as usize) == 1)
//         {
//             /* Check if this was a write */
//             if (MI_IS_WRITE_ACCESS(FaultCode))
//             {
//                 /* Was it to a read-only page that is not copy on write? */
//                 Pfn1 = MI_PFN_ELEMENT(PointerPte->u.Hard.PageFrameNumber);
//                 if (!(TempPte.Long & PTE_READWRITE) &&
//                     !(Pfn1->OriginalPte.soft_protection() & MM_READWRITE) &&
//                     !MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//                 {
//                     /* Case not yet handled */
//                     ASSERT(!IsSessionAddress);
//
//                     /* Crash with distinguished bugcheck code */
//                     KeBugCheckEx(ATTEMPTED_WRITE_TO_READONLY_MEMORY,
//                                  (ULONG_PTR)Address,
//                                  TempPte.Long,
//                                  (ULONG_PTR)TrapInformation,
//                                  12);
//                 }
//             }
//
//             /* Check for execution of non-executable memory */
//             if (MI_IS_INSTRUCTION_FETCH(FaultCode) &&
//                 !MI_IS_PAGE_EXECUTABLE(&TempPte))
//             {
//                 KeBugCheckEx(ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY,
//                              (ULONG_PTR)Address,
//                              (ULONG_PTR)TempPte.Long,
//                              (ULONG_PTR)TrapInformation,
//                              2);
//             }
//
//             /* Check for read-only write in session space */
//             if ((IsSessionAddress) &&
//                 MI_IS_WRITE_ACCESS(FaultCode) &&
//                 !MI_IS_PAGE_WRITEABLE(&TempPte))
//             {
//                 /* Sanity check */
//                 ASSERT(MI_IS_SESSION_IMAGE_ADDRESS(Address));
//
//                 /* Was this COW? */
//                 if (!MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//                 {
//                     /* Then this is not allowed */
//                     KeBugCheckEx(ATTEMPTED_WRITE_TO_READONLY_MEMORY,
//                                  (ULONG_PTR)Address,
//                                  (ULONG_PTR)TempPte.Long,
//                                  (ULONG_PTR)TrapInformation,
//                                  13);
//                 }
//
//                 /* Otherwise, handle COW */
//                 ASSERT(FALSE);
//             }
//
//             /* Release the working set */
//             MiUnlockWorkingSet(CurrentThread, WorkingSet);
//             KeLowerIrql(LockIrql);
//
//             /* Otherwise, the PDE was probably invalid, and all is good now */
//             return STATUS_SUCCESS;
//         }
//
//         /* Check one kind of prototype PTE */
//         if ((TempPte.prototype() as usize))
//         {
//             /* Make sure protected pool is on, and that this is a pool address */
//             if ((MmProtectFreedNonPagedPool) &&
//                 (((Address >= MmNonPagedPoolStart) &&
//                   (Address < (PVOID)((ULONG_PTR)MmNonPagedPoolStart +
//                                      MmSizeOfNonPagedPoolInBytes))) ||
//                  ((Address >= MmNonPagedPoolExpansionStart) &&
//                   (Address < MmNonPagedPoolEnd))))
//             {
//                 /* Bad boy, bad boy, whatcha gonna do, whatcha gonna do when ARM3 comes for you! */
//                 KeBugCheckEx(DRIVER_CAUGHT_MODIFYING_FREED_POOL,
//                              (ULONG_PTR)Address,
//                              FaultCode,
//                              Mode,
//                              4);
//             }
//
//             /* Get the prototype PTE! */
//             ProtoPte = MiProtoPteToPte(&mut TempPte as PMMPTE);
//
//             /* Do we need to locate the prototype PTE in session space? */
//             if ((IsSessionAddress) &&
//                 (TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED)
//             {
//                 /* Yep, go find it as well as the VAD for it */
//                 ProtoPte = MiCheckVirtualAddress(Address,
//                                                  &ProtectionCode,
//                                                  &Vad)) as ULONG_PTR);
//                 ASSERT(ProtoPte != NULL);
//             }
//         }
//         else
//         {
//             /* We don't implement transition PTEs */
//             ASSERT((TempPte.transition() as usize) == 0);
//
//             /* Check for no-access PTE */
//             if (TempPte.soft_protection() == MM_NOACCESS
//             {
//                 /* Bugcheck the system! */
//                 KeBugCheckEx(PAGE_FAULT_IN_NONPAGED_AREA,
//                              (ULONG_PTR)Address,
//                              FaultCode,
//                              (ULONG_PTR)TrapInformation,
//                              1)) as ULONG);
//             }
//
//             /* Check for no protecton at all */
//             if (TempPte.soft_protection() == MM_ZERO_ACCESS
//             {
//                 /* Bugcheck the system! */
//                 KeBugCheckEx(PAGE_FAULT_IN_NONPAGED_AREA,
//                              (ULONG_PTR)Address,
//                              FaultCode,
//                              (ULONG_PTR)TrapInformation,
//                              0)) as ULONG);
//             }
//         }
//
//         /* Check for demand page */
//         if (MI_IS_WRITE_ACCESS(FaultCode) &&
//             !(ProtoPte) &&
//             !(IsSessionAddress) &&
//             !((TempPte.valid() as usize)))
//         {
//             /* Get the protection code */
//             ASSERT((TempPte.transition() as usize) == 0);
//             if (!(TempPte.soft_protection() & MM_READWRITE))
//             {
//                 /* Bugcheck the system! */
//                 KeBugCheckEx(ATTEMPTED_WRITE_TO_READONLY_MEMORY,
//                              (ULONG_PTR)Address,
//                              TempPte.Long,
//                              (ULONG_PTR)TrapInformation,
//                              14);
//             }
//         }
//
//         /* Now do the real fault handling */
//         Status = MiDispatchFault(FaultCode,
//                                  Address,
//                                  PointerPte,
//                                  ProtoPte,
//                                  FALSE,
//                                  CurrentProcess,
//                                  TrapInformation,
//                                  NULL);
//
//         /* Release the working set */
//         ASSERT(KeAreAllApcsDisabled() == TRUE);
//         MiUnlockWorkingSet(CurrentThread, WorkingSet);
//         KeLowerIrql(LockIrql);
//
//         if (Status == STATUS_NO_MEMORY)
//         {
//             MmRebalanceMemoryConsumersAndWait();
//             goto RetryKernel;
//         }
//
//         /* We are done! */
//         DPRINT("Fault resolved with status: %lx\n", Status);
//         return Status;
//     }
//
//     /* This is a user fault */
// UserFault:
//     CurrentThread = PsGetCurrentThread();
//     CurrentProcess = (PEPROCESS)CurrentThread->Tcb.ApcState.Process;
//
//     /* Lock the working set */
//     MiLockProcessWorkingSet(CurrentProcess, CurrentThread);
//
//     ProtectionCode = MM_INVALID_PROTECTION;
//
// #if (_MI_PAGING_LEVELS == 4)
//     /* Check if the PXE is valid */
//     if (PointerPxe->u.Hard.Valid == 0)
//     {
//         /* Right now, we only handle scenarios where the PXE is totally empty */
//         ASSERT(PointerPxe->u.Long == 0);
//
//         /* This is only possible for user mode addresses! */
//         ASSERT(PointerPte <= MiHighestUserPte);
//
//         /* Check if we have a VAD */
//         MiCheckVirtualAddress(Address, &ProtectionCode, &Vad);
//         if (ProtectionCode == MM_NOACCESS)
//         {
//             /* MM_NOACCESS may indicate a partially committed VAD (MemCommit=0,
//              * pages committed individually). Create page table if a VAD exists;
//              * the PTE-level check handles uncommitted pages correctly. */
//             if (!MiLocateAddress(Address))
//             {
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return STATUS_ACCESS_VIOLATION;
//             }
//             ProtectionCode = MM_EXECUTE_READWRITE;
//         }
//
//         /* Resolve a demand zero fault */
//         Status = MiResolveDemandZeroFault(PointerPpe,
//                                  PointerPxe,
//                                  MM_EXECUTE_READWRITE,
//                                  CurrentProcess,
//                                  MM_NOIRQL);
//         if (!NT_SUCCESS(Status))
//         {
//             goto ExitUser;
//         }
//
//         /* We should come back with a valid PXE */
//         ASSERT(PointerPxe->u.Hard.Valid == 1);
//     }
// #endif
//
// #if (_MI_PAGING_LEVELS >= 3)
//     /* Check if the PPE is valid */
//     if (PointerPpe->u.Hard.Valid == 0)
//     {
//         /* Right now, we only handle scenarios where the PPE is totally empty */
//         ASSERT(PointerPpe->u.Long == 0);
//
//         /* This is only possible for user mode addresses! */
//         ASSERT(PointerPte <= MiHighestUserPte);
//
//         /* Check if we have a VAD, unless we did this already */
//         if (ProtectionCode == MM_INVALID_PROTECTION)
//         {
//             MiCheckVirtualAddress(Address, &ProtectionCode, &Vad);
//         }
//
//         if (ProtectionCode == MM_NOACCESS)
//         {
//             /* MM_NOACCESS may indicate a partially committed VAD (MemCommit=0).
//              * Create page table if a VAD exists. */
//             if (!MiLocateAddress(Address))
//             {
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return STATUS_ACCESS_VIOLATION;
//             }
//             ProtectionCode = MM_EXECUTE_READWRITE;
//         }
//
//         /* Resolve a demand zero fault */
//         Status = MiResolveDemandZeroFault(PointerPde,
//                                  PointerPpe,
//                                  MM_EXECUTE_READWRITE,
//                                  CurrentProcess,
//                                  MM_NOIRQL);
//         if (!NT_SUCCESS(Status))
//         {
//             goto ExitUser;
//         }
//
//         /* We should come back with a valid PPE */
//         ASSERT(PointerPpe->u.Hard.Valid == 1);
//         MiIncrementPageTableReferences(PointerPde);
//     }
// #endif
//
//     /* Check if the PDE is invalid */
//     if (PointerPde->u.Hard.Valid == 0)
//     {
//         /* Right now, we only handle scenarios where the PDE is totally empty */
//         ASSERT(PointerPde->u.Long == 0);
//
//         /* And go dispatch the fault on the PDE. This should handle the demand-zero */
// #if MI_TRACE_PFNS
//         UserPdeFault = TRUE;
// #endif
//         /* Check if we have a VAD, unless we did this already */
//         if (ProtectionCode == MM_INVALID_PROTECTION)
//         {
//             MiCheckVirtualAddress(Address, &ProtectionCode, &Vad);
//         }
//
//         if (ProtectionCode == MM_NOACCESS)
//         {
// #if (_MI_PAGING_LEVELS >= 3)
//             /* MM_NOACCESS may indicate a partially committed VAD (MemCommit=0).
//              * Create page table if a VAD exists. */
//             if (MiLocateAddress(Address))
//             {
//                 ProtectionCode = MM_EXECUTE_READWRITE;
//             }
//             else
// #endif
//             {
// #if (_MI_PAGING_LEVELS == 2)
//                 /* Could be a page table for paged pool */
//                 MiCheckPdeForPagedPool(Address);
// #endif
//                 /* Has the code above changed anything -- is this now a valid PTE? */
//                 Status = (PointerPde->u.Hard.Valid == 1) ? STATUS_SUCCESS : STATUS_ACCESS_VIOLATION;
//
//                 /* Either this was a bogus VA or we've fixed up a paged pool PDE */
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return Status;
//             }
//         }
//
//         /* Resolve a demand zero fault */
//         Status = MiResolveDemandZeroFault(PointerPte,
//                                  PointerPde,
//                                  MM_EXECUTE_READWRITE,
//                                  CurrentProcess,
//                                  MM_NOIRQL);
//         if (!NT_SUCCESS(Status))
//         {
//             goto ExitUser;
//         }
//
// #if _MI_PAGING_LEVELS >= 3
//         MiIncrementPageTableReferences(PointerPte);
// #endif
//
// #if MI_TRACE_PFNS
//         UserPdeFault = FALSE;
//         /* Update debug info */
//         if (TrapInformation)
//             MiGetPfnEntry(PointerPde->u.Hard.PageFrameNumber)->CallSite = (PVOID)((PKTRAP_FRAME)TrapInformation)->Eip;
//         else
//             MiGetPfnEntry(PointerPde->u.Hard.PageFrameNumber)->CallSite = _ReturnAddress();
// #endif
//         /* We should come back with APCs enabled, and with a valid PDE */
//         ASSERT(KeAreAllApcsDisabled() == TRUE);
//         ASSERT(PointerPde->u.Hard.Valid == 1);
//     }
//     else
//     {
//         /* Not yet implemented in ReactOS */
//         ASSERT(MI_IS_PAGE_LARGE(PointerPde) == FALSE);
//     }
//
//     /* Now capture the PTE. */
//     TempPte = *PointerPte;
//
//     /* Check if the PTE is valid */
//     if ((TempPte.valid() as usize))
//     {
//         /* Check if this is a write on a readonly PTE */
//         if (MI_IS_WRITE_ACCESS(FaultCode))
//         {
//             /* Is this a copy on write PTE? */
//             if (MI_IS_PAGE_COPY_ON_WRITE(&TempPte))
//             {
//                 PFN_NUMBER PageFrameIndex, OldPageFrameIndex;
//                 PMMPFN Pfn1;
//
//                 LockIrql = MiAcquirePfnLock();
//
//                 ASSERT(MmAvailablePages > 0);
//
//                 MI_SET_USAGE(MI_USAGE_COW);
//                 MI_SET_PROCESS(CurrentProcess);
//
//                 /* Allocate a new page and copy it */
//                 PageFrameIndex = MiRemoveAnyPage(MI_GET_NEXT_PROCESS_COLOR(CurrentProcess));
//                 if (PageFrameIndex == 0)
//                 {
//                     MiReleasePfnLock(LockIrql);
//                     Status = STATUS_NO_MEMORY;
//                     goto ExitUser;
//                 }
//                 OldPageFrameIndex = PFN_FROM_PTE(&mut TempPte as PMMPTE);
//
//                 MiCopyPfn(PageFrameIndex, OldPageFrameIndex);
//
//                 /* Dereference whatever this PTE is referencing */
//                 Pfn1 = MI_PFN_ELEMENT(OldPageFrameIndex);
//                 ASSERT(Pfn1->u3.e1.PrototypePte == 1);
//                 ASSERT(!MI_IS_PFN_DELETED(Pfn1));
//                 ProtoPte = Pfn1->PteAddress;
//                 MiDeletePte(PointerPte as *mut _MMPTE, Address, CurrentProcess, ProtoPte as *mut _MMPTE);
//
//                 /* And make a new shiny one with our page */
//                 MiInitializePfn(PageFrameIndex, PointerPte, TRUE);
//                 TempPte.set_page_frame_number((PageFrameIndex) as PFN_NUMBER);
// #ifdef _M_ARM64
//                 TempPte.u.Hard.Writable = 1;
//                 TempPte.u.Hard.NotDirty = 0;  /* Make hardware-writable */
// #else
//                 TempPte.write() = 1;
// #endif
//                 TempPte.u.Hard.CopyOnWrite = 0;
//
//                 MI_WRITE_VALID_PTE(PointerPte, TempPte);
//
//                 MiReleasePfnLock(LockIrql);
//
//                 /* Return the status */
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return STATUS_PAGE_FAULT_COPY_ON_WRITE;
//             }
//
//             /* Is this a read-only PTE? */
//             if (!MI_IS_PAGE_WRITEABLE(&TempPte))
//             {
//                 /* Return the status */
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return STATUS_ACCESS_VIOLATION;
//             }
//         }
//
// #if _MI_HAS_NO_EXECUTE
//         /* Check for execution of non-executable memory */
//         if (MI_IS_INSTRUCTION_FETCH(FaultCode) &&
//             !MI_IS_PAGE_EXECUTABLE(&TempPte))
//         {
//             /* Check if execute enable was set */
//             if (CurrentProcess->Pcb.Flags.ExecuteEnable)
//             {
//                 /* Fix up the PTE to be executable */
// #ifdef _M_ARM64
//                 TempPte.u.Hard.UserNoExecute = 0;
// #else
//                 TempPte.u.Hard.NoExecute = 0;
// #endif
//                 MI_UPDATE_VALID_PTE(PointerPte, TempPte);
//                 MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                 return STATUS_SUCCESS;
//             }
//
//             /* Return the status */
//             MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//             return STATUS_ACCESS_VIOLATION;
//         }
// #endif
//
//         /* The fault has already been resolved by a different thread */
//         MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//         return STATUS_SUCCESS;
//     }
//
//     /* Quick check for demand-zero */
//     if ((TempPte.Long == (MM_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS)) ||
//         (TempPte.Long == (MM_EXECUTE_READWRITE << MM_PTE_SOFTWARE_PROTECTION_BITS)))
//     {
//         /* Resolve the fault */
//         Status = MiResolveDemandZeroFault(Address,
//                                  PointerPte,
//                                  TempPte.soft_protection(),
//                                  CurrentProcess,
//                                  MM_NOIRQL);
//         if (!NT_SUCCESS(Status))
//         {
//             goto ExitUser;
//         }
//
// #if MI_TRACE_PFNS
//         /* Update debug info */
//         if (TrapInformation)
//             MiGetPfnEntry(PointerPte->u.Hard.PageFrameNumber)->CallSite = (PVOID)((PKTRAP_FRAME)TrapInformation)->Eip;
//         else
//             MiGetPfnEntry(PointerPte->u.Hard.PageFrameNumber)->CallSite = _ReturnAddress();
// #endif
//
//         /* Return the status */
//         MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//         return STATUS_PAGE_FAULT_DEMAND_ZERO;
//     }
//
//     /* Check for zero PTE */
//     if (TempPte.Long == 0)
//     {
//         /* Check if this address range belongs to a valid allocation (VAD) */
//         ProtoPte = MiCheckVirtualAddress(Address, &ProtectionCode, &Vad);
//         if (ProtectionCode == MM_NOACCESS)
//         {
// #if (_MI_PAGING_LEVELS == 2)
//             /* Could be a page table for paged pool */
//             MiCheckPdeForPagedPool(Address);
// #endif
//             /* Has the code above changed anything -- is this now a valid PTE? */
//             Status = (PointerPte->u.Hard.Valid == 1) ? STATUS_SUCCESS : STATUS_ACCESS_VIOLATION;
//
//             /* Either this was a bogus VA or we've fixed up a paged pool PDE */
//             MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//             return Status;
//         }
//
//         /*
//          * Check if this is a real user-mode address or actually a kernel-mode
//          * page table for a user mode address
//          */
//         if (Address <= MM_HIGHEST_USER_ADDRESS
// #if _MI_PAGING_LEVELS >= 3
//             || MiIsUserPte(Address)
// #if _MI_PAGING_LEVELS == 4
//             || MiIsUserPde(Address)
// #endif
// #endif
//         )
//         {
//             /* Add an additional page table reference */
//             MiIncrementPageTableReferences(Address);
//         }
//
//         /* Is this a guard page? */
//         if ((ProtectionCode & MM_PROTECT_SPECIAL) == MM_GUARDPAGE)
//         {
//             /* The VAD protection cannot be MM_DECOMMIT! */
//             ASSERT(ProtectionCode != MM_DECOMMIT);
//
//             /* Remove the bit */
//             TempPte.set_soft_protection((ProtectionCode & ~MM_GUARDPAGE) as ULONG);
//             MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//
//             /* Not supported */
//             ASSERT(ProtoPte == NULL);
//             ASSERT(CurrentThread->ApcNeeded == 0);
//
//             /* Drop the working set lock */
//             MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//             ASSERT(KeGetCurrentIrql() == OldIrql);
//
//             /* Handle stack expansion */
//             return MiCheckForUserStackOverflow(Address, TrapInformation);
//         }
//
//         /* Did we get a prototype PTE back? */
//         if (!ProtoPte)
//         {
//             /* Is this PTE actually part of the PDE-PTE self-mapping directory? */
//             if (PointerPde == MiAddressToPde(PTE_BASE))
//             {
//                 /* Then it's really a demand-zero PDE (on behalf of user-mode) */
// #ifdef _M_ARM
//                 _WARN("This is probably completely broken!");
//                 MI_WRITE_INVALID_PDE((PMMPDE)PointerPte, DemandZeroPde);
// #else
//                 MI_WRITE_INVALID_PDE(PointerPte, DemandZeroPde);
// #endif
//             }
//             else
//             {
//                 /* No, create a new PTE. First, write the protection */
//                 TempPte.set_soft_protection((ProtectionCode) as ULONG);
//                 MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//             }
//
//             /* Lock the PFN database since we're going to grab a page */
//             OldIrql = MiAcquirePfnLock();
//
//             /* Make sure we have enough pages */
//             //ASSERT(MmAvailablePages >= 32);
//
//             /* Try to get a zero page */
//             MI_SET_USAGE(MI_USAGE_PEB_TEB);
//             super::helpers::MI_SET_PROCESS2(CurrentProcess->ImageFileName);
//             Color = MI_GET_NEXT_PROCESS_COLOR(CurrentProcess);
//             PageFrameIndex = MiRemoveZeroPageSafe(Color);
//             if (!PageFrameIndex)
//             {
//                 /* Grab a page out of there. Later we should grab a colored zero page */
//                 PageFrameIndex = MiRemoveAnyPage(Color);
//
//                 /* Release the lock since we need to do some zeroing */
//                 MiReleasePfnLock(OldIrql);
//
//                 if (PageFrameIndex == 0)
//                 {
//                     Status = STATUS_NO_MEMORY;
//                     goto ExitUser;
//                 }
//
//                 /* Zero out the page, since it's for user-mode */
//                 MiZeroPfn(PageFrameIndex);
//
//                 /* Grab the lock again so we can initialize the PFN entry */
//                 OldIrql = MiAcquirePfnLock();
//             }
//
//             /* Initialize the PFN entry now */
//             MiInitializePfn(PageFrameIndex, PointerPte as *mut _MMPTE, 1);
//
//             /* Increment the count of pages in the process */
//             CurrentProcess->NumberOfPrivatePages++;
//
//             /* One more demand-zero fault */
//             KeGetCurrentPrcb()->MmDemandZeroCount++;
//
//             /* And we're done with the lock */
//             MiReleasePfnLock(OldIrql);
//
//             /* Fault on user PDE, or fault on user PTE? */
//             if (PointerPte <= MiHighestUserPte)
//             {
//                 /* User fault, build a user PTE */
//                 MI_MAKE_HARDWARE_PTE_USER(&TempPte,
//                                           PointerPte,
//                                           PointerPte->u.Soft.Protection(),
//                                           PageFrameIndex);
//             }
//             else
//             {
//                 /* This is a user-mode PDE, create a kernel PTE for it */
//                 MI_MAKE_HARDWARE_PTE(&TempPte,
//                                      PointerPte,
//                                      PointerPte->u.Soft.Protection(),
//                                      PageFrameIndex);
//             }
//
//             /* Write the dirty bit for writeable pages */
//             if (MI_IS_PAGE_WRITEABLE(&TempPte)) MI_MAKE_DIRTY_PAGE(&TempPte);
//
//             /* And now write down the PTE, making the address valid */
//             MI_WRITE_VALID_PTE(PointerPte, TempPte);
//             Pfn1 = MI_PFN_ELEMENT(PageFrameIndex);
//             ASSERT(Pfn1->u1.Event == NULL);
//
//             /* Demand zero */
//             ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//             MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//             return STATUS_PAGE_FAULT_DEMAND_ZERO;
//         }
//
//         /* We should have a valid protection here */
//         ASSERT(ProtectionCode != 0x100);
//
//         /* Write the prototype PTE */
//         TempPte = MMPTE { Long: PrototypePte as ULONG_PTR };
//         TempPte.set_soft_protection((ProtectionCode) as ULONG);
//         ASSERT(TempPte.Long != 0);
//         MI_WRITE_INVALID_PTE(PointerPte, TempPte);
//     }
//     else
//     {
//         /* Get the protection code and check if this is a proto PTE */
//         ProtectionCode = (ULONG)TempPte.soft_protection();
//         if ((TempPte.prototype() as usize))
//         {
//             /* Do we need to go find the real PTE? */
//             if (TempPte.soft_page_file_high() == MI_PTE_LOOKUP_NEEDED
//             {
//                 /* Get the prototype pte and VAD for it */
//                 ProtoPte = MiCheckVirtualAddress(Address,
//                                                  &ProtectionCode,
//                                                  &Vad)) as ULONG_PTR);
//                 if (!ProtoPte)
//                 {
//                     ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//                     MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//                     return STATUS_ACCESS_VIOLATION;
//                 }
//             }
//             else
//             {
//                 /* Get the prototype PTE! */
//                 ProtoPte = MiProtoPteToPte(&mut TempPte as PMMPTE);
//
//                 /* Is it read-only */
//                 if (TempPte.u.Proto.ReadOnly)
//                 {
//                     /* Set read-only code */
//                     ProtectionCode = MM_READONLY;
//                 }
//                 else
//                 {
//                     /* Set unknown protection */
//                     ProtectionCode = 0x100;
//                     ASSERT(CurrentProcess->CloneRoot != NULL);
//                 }
//             }
//         }
//     }
//
//     /* Do we have a valid protection code? */
//     if (ProtectionCode != 0x100)
//     {
//         /* Run a software access check first, including to detect guard pages */
//         Status = MiAccessCheck(PointerPte,
//                                !MI_IS_NOT_PRESENT_FAULT(FaultCode),
//                                Mode,
//                                ProtectionCode,
//                                TrapInformation,
//                                FALSE);
//         if (Status != STATUS_SUCCESS)
//         {
//             /* Not supported */
//             ASSERT(CurrentThread->ApcNeeded == 0);
//
//             /* Drop the working set lock */
//             MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//             ASSERT(KeGetCurrentIrql() == OldIrql);
//
//             /* Did we hit a guard page? */
//             if (Status == STATUS_GUARD_PAGE_VIOLATION)
//             {
//                 /* Handle stack expansion */
//                 return MiCheckForUserStackOverflow(Address, TrapInformation);
//             }
//
//             /* Otherwise, fail back to the caller directly */
//             return Status;
//         }
//     }
//
//     /* Dispatch the fault */
//     Status = MiDispatchFault(FaultCode,
//                              Address,
//                              PointerPte,
//                              ProtoPte,
//                              FALSE,
//                              CurrentProcess,
//                              TrapInformation,
//                              Vad);
//
// ExitUser:
//
//     /* Return the status */
//     ASSERT(KeGetCurrentIrql() <= APC_LEVEL);
//     MiUnlockProcessWorkingSet(CurrentProcess, CurrentThread);
//
//     if (Status == STATUS_NO_MEMORY)
//     {
//         MmRebalanceMemoryConsumersAndWait();
//         goto UserFault;
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// MmGetExecuteOptions(IN PULONG ExecuteOptions)
// {
//     PKPROCESS CurrentProcess = &PsGetCurrentProcess()->Pcb;
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//
//     *ExecuteOptions = 0;
//
//     if (CurrentProcess->Flags.ExecuteDisable)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_DISABLE;
//     }
//
//     if (CurrentProcess->Flags.ExecuteEnable)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_ENABLE;
//     }
//
//     if (CurrentProcess->Flags.DisableThunkEmulation)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION;
//     }
//
//     if (CurrentProcess->Flags.Permanent)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_PERMANENT;
//     }
//
//     if (CurrentProcess->Flags.ExecuteDispatchEnable)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_EXECUTE_DISPATCH_ENABLE;
//     }
//
//     if (CurrentProcess->Flags.ImageDispatchEnable)
//     {
//         *ExecuteOptions |= MEM_EXECUTE_OPTION_IMAGE_DISPATCH_ENABLE;
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// MmSetExecuteOptions(IN ULONG ExecuteOptions)
// {
//     PKPROCESS CurrentProcess = &PsGetCurrentProcess()->Pcb;
//     KLOCK_QUEUE_HANDLE ProcessLock;
//     NTSTATUS Status = STATUS_ACCESS_DENIED;
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//
//     /* Only accept valid flags */
//     if (ExecuteOptions & ~MEM_EXECUTE_OPTION_VALID_FLAGS)
//     {
//         /* Fail */
//         DPRINT1("Invalid no-execute options\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Change the NX state in the process lock */
//     KiAcquireProcessLockRaiseToSynch(CurrentProcess, &ProcessLock);
//
//     /* Don't change anything if the permanent flag was set */
//     if (!CurrentProcess->Flags.Permanent)
//     {
//         /* Start by assuming it's not disabled */
//         CurrentProcess->Flags.ExecuteDisable = FALSE;
//
//         /* Now process each flag and turn the equivalent bit on */
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_DISABLE)
//         {
//             CurrentProcess->Flags.ExecuteDisable = TRUE;
//         }
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_ENABLE)
//         {
//             CurrentProcess->Flags.ExecuteEnable = TRUE;
//         }
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION)
//         {
//             CurrentProcess->Flags.DisableThunkEmulation = TRUE;
//         }
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_PERMANENT)
//         {
//             CurrentProcess->Flags.Permanent = TRUE;
//         }
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_EXECUTE_DISPATCH_ENABLE)
//         {
//             CurrentProcess->Flags.ExecuteDispatchEnable = TRUE;
//         }
//         if (ExecuteOptions & MEM_EXECUTE_OPTION_IMAGE_DISPATCH_ENABLE)
//         {
//             CurrentProcess->Flags.ImageDispatchEnable = TRUE;
//         }
//
//         /* These are turned on by default if no-execution is also enabled */
//         if (CurrentProcess->Flags.ExecuteEnable)
//         {
//             CurrentProcess->Flags.ExecuteDispatchEnable = TRUE;
//             CurrentProcess->Flags.ImageDispatchEnable = TRUE;
//         }
//
//         /* All good */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Release the lock and return status */
//     KiReleaseProcessLock(&ProcessLock);
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
