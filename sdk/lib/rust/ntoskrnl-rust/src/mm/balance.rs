use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _EPROCESS,
    _KAPC_STATE,
    _KEVENT,
    _KTIMER,
    _CLIENT_ID,
    _LIST_ENTRY,
    ExAcquireRundownProtection,
    ExReleaseRundownProtection,
    KEVENT,
    KPRIORITY,
    KTIMER,
    KeBugCheck,
    KeDelayExecutionThread,
    KeInitializeEvent,
    KeInitializeTimerEx,
    KeResetEvent,
    KeSetTimerEx,
    KeStackAttachProcess,
    KeUnstackDetachProcess,
    KeWaitForMultipleObjects,
    KeWaitForSingleObject,
    LARGE_INTEGER,
    MiLockProcessWorkingSet,
    MiUnlockProcessWorkingSet,
    MmAllocPage,
    MmDereferencePage,
    MmGetLRUFirstUserPage,
    MmGetLRUNextUserPage,
    MmGetRmapListHeadPage,
    MmIsAddressValid,
    MmPageOutPhysicalAddress,
    NtSetInformationThread,
    PEPROCESS,
    PETHREAD,
    PMM_RMAP_ENTRY,
    PPFN_NUMBER,
    PsCreateSystemThread,
    PsGetCurrentThread,
};

extern "C" {
    fn UpdateTotalCommittedPages(delta: LONG);
    static mut MmAvailablePages: PFN_NUMBER;
    fn CcRosTrimCache(target: ULONG, nr_freed: *mut ULONG);
    fn MM_ANY_WS_LOCK_HELD(thread: PETHREAD) -> BOOLEAN;
}

extern "C" {
    fn InterlockedDecrementUL(addend: *mut ULONG) -> ULONG;
    fn InterlockedIncrementUL(addend: *mut ULONG) -> ULONG;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedCompareExchange(destination: *mut LONG, exchange: LONG, comparand: LONG) -> LONG;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG;
}
#[cfg(not(target_arch = "x86"))]
extern "system" {
    fn InterlockedExchange(target: *mut LONG, value: LONG) -> LONG;
}

#[repr(C)]
struct MM_ALLOCATION_REQUEST {
    Page: PFN_NUMBER,
    ListEntry: _LIST_ENTRY,
    Event: _KEVENT,
}

type PMM_ALLOCATION_REQUEST = *mut MM_ALLOCATION_REQUEST;

#[no_mangle]
pub static mut MiMemoryConsumers: [MM_MEMORY_CONSUMER; MC_MAXIMUM as usize] = unsafe { core::mem::zeroed() };
static mut MiMinimumAvailablePages: ULONG = 0;
static mut MiMinimumPagesPerRun: ULONG = 0;
static mut MiBalancerThreadId: _CLIENT_ID = unsafe { core::mem::zeroed() };
static mut MiBalancerThreadHandle: PVOID = core::ptr::null_mut();
static mut MiBalancerEvent: _KEVENT = unsafe { core::mem::zeroed() };
static mut MiBalancerDoneEvent: _KEVENT = unsafe { core::mem::zeroed() };
static mut MiBalancerTimer: _KTIMER = unsafe { core::mem::zeroed() };

static mut PageOutThreadActive: LONG = 0;

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MmInitializeBalancer(
    NrAvailablePages: ULONG,
    _NrSystemPages: ULONG,
) {
    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(MiMemoryConsumers) as *mut u8,
        0,
        core::mem::size_of_val(&MiMemoryConsumers),
    );

    MiMinimumAvailablePages = 256;
    MiMinimumPagesPerRun = 256;
    MiMemoryConsumers[MC_USER as usize].PagesTarget = NrAvailablePages / 2;
}

// CODE_SEG("INIT")
// The Trim callback has no NTAPI in C (cdecl) — extern "C" on i386.
#[no_mangle]
pub unsafe extern "system" fn MmInitializeMemoryConsumer(
    Consumer: ULONG,
    Trim: Option<unsafe extern "C" fn(ULONG, ULONG, *mut ULONG) -> NTSTATUS>,
) {
    MiMemoryConsumers[Consumer as usize].Trim = Trim;
}

#[no_mangle]
pub unsafe extern "system" fn MmReleasePageMemoryConsumer(
    Consumer: ULONG,
    Page: PFN_NUMBER,
) -> NTSTATUS {
    if Page == 0 {
        KeBugCheck(MEMORY_MANAGEMENT as u32);
    }

    InterlockedDecrementUL(
        core::ptr::addr_of_mut!(MiMemoryConsumers[Consumer as usize].PagesUsed)
    );
    UpdateTotalCommittedPages(-1);

    // [guard] let old_irql = MiAcquirePfnLock();
    let pfn_guard = unsafe { crate::ARM3::helpers::PfnLockGuard::acquire() };

    MmDereferencePage(Page as _);

    drop(pfn_guard);

    STATUS_SUCCESS
}

unsafe fn MiTrimMemoryConsumer(Consumer: ULONG, InitialTarget: ULONG) -> ULONG {
    let mut target = InitialTarget;
    let mut nr_freed_pages: ULONG = 0;

    if MiMemoryConsumers[Consumer as usize].Trim.is_none() {
        return InitialTarget;
    }

    if (MmAvailablePages as ULONG) < MiMinimumAvailablePages {
        target = core::cmp::max(
            target,
            MiMinimumAvailablePages - MmAvailablePages as ULONG,
        );
    } else if MiMemoryConsumers[Consumer as usize].PagesUsed
        > MiMemoryConsumers[Consumer as usize].PagesTarget
    {
        target = core::cmp::max(
            target,
            MiMemoryConsumers[Consumer as usize].PagesUsed
                - MiMemoryConsumers[Consumer as usize].PagesTarget,
        );
    }

    if target != 0 {
        let status = (MiMemoryConsumers[Consumer as usize].Trim.unwrap())(
            target,
            if (MmAvailablePages as ULONG) < MiMinimumAvailablePages { 1 } else { 0 },
            &mut nr_freed_pages,
        );

        if status < 0 {
            KeBugCheck(MEMORY_MANAGEMENT as u32);
        }
    }

    if InitialTarget > nr_freed_pages {
        InitialTarget - nr_freed_pages
    } else {
        0
    }
}

// MmTrimUserMemory is defined without NTAPI in C (cdecl) — use extern "C".
// On i386, "system" = stdcall (_MmTrimUserMemory@12) but C expects cdecl (_MmTrimUserMemory).
#[no_mangle]
pub unsafe extern "C" fn MmTrimUserMemory(
    Target: ULONG,
    Priority: ULONG,
    NrFreedPages: *mut ULONG,
) -> NTSTATUS {
    let NrFreedPages = opaque_ptr(NrFreedPages);
    let mut target = Target;
    *NrFreedPages = 0;

    let mut first_page: PFN_NUMBER = MmGetLRUFirstUserPage() as PFN_NUMBER;
    let mut current_page: PFN_NUMBER = first_page;

    while current_page != 0 && target > 0 {
        if Priority != 0 {
            let status = MmPageOutPhysicalAddress(current_page as _);
            if status >= 0 {
                target -= 1;
                *NrFreedPages += 1;
                if current_page == first_page {
                    first_page = 0;
                }
            }
        } else {
            let mut process: PEPROCESS = core::ptr::null_mut();
            let mut address: PVOID = core::ptr::null_mut();
            let mut accessed: BOOLEAN = 0;

            loop {
                let mut apc_state: _KAPC_STATE = core::mem::zeroed();
                let old_irql = MiAcquirePfnLock();
                let mut entry = MmGetRmapListHeadPage(current_page as _);

                loop {
                    if entry.is_null() {
                        break;
                    }

                    // C: `if (RMAP_IS_SEGMENT(Entry->Address)) { ...; continue; }`.
                    // This was inlined as `Address & 1`, which skipped only the
                    // odd-indexed segment entries; the even ones fell through and
                    // were handled as process mappings, so `Entry->Process` — a
                    // PCACHE_SECTION_PAGE_TABLE, not an EPROCESS — was passed to
                    // ObReferenceObject/ExAcquireRundownProtection/KeStackAttachProcess
                    // and the segment pseudo-address to MiAddressToPte.
                    if RMAP_IS_SEGMENT((*entry).Address) {
                        entry = (*entry).Next;
                        continue;
                    }

                    if (*entry).Address < address {
                        entry = (*entry).Next;
                        continue;
                    }

                    if (*entry).Address == address && (*entry).Process <= process {
                        entry = (*entry).Next;
                        continue;
                    }

                    break;
                }

                if entry.is_null() {
                    MiReleasePfnLock(old_irql);
                    break;
                }

                process = (*entry).Process;
                address = (*entry).Address;

                ObReferenceObject(process as PVOID);

                if ExAcquireRundownProtection(
                    core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect)
                ) == 0
                {
                    ObDereferenceObject(process as PVOID);
                    MiReleasePfnLock(old_irql);
                    continue;
                }

                MiReleasePfnLock(old_irql);

                KeStackAttachProcess(
                    core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
                    &mut apc_state,
                );
                MiLockProcessWorkingSet(process, PsGetCurrentThread());

                if MmIsAddressValid(address) != 0 {
                    let pte = MiAddressToPte(address);
                    accessed = if accessed != 0 || (*pte).accessed() { 1 } else { 0 };
                    // Clear accessed bit
                    (*pte).Long &= !PTE_ACCESSED;
                }

                MiUnlockProcessWorkingSet(process, PsGetCurrentThread());

                KeUnstackDetachProcess(&mut apc_state);
                ExReleaseRundownProtection(
                    core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).RundownProtect)
                );
                ObDereferenceObject(process as PVOID);
            }

            if accessed == 0 {
                let status = MmPageOutPhysicalAddress(current_page as _);
                if status >= 0 {
                    if current_page == first_page {
                        first_page = 0;
                    }
                }
            }

            target -= 1;
        }

        current_page = MmGetLRUNextUserPage(current_page as _, 1) as PFN_NUMBER;
        if first_page == 0 {
            first_page = current_page;
        } else if current_page == first_page {
            return STATUS_SUCCESS;
        }
    }

    if current_page != 0 {
        let old_irql = MiAcquirePfnLock();
        MmDereferencePage(current_page as _);
        MiReleasePfnLock(old_irql);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmRebalanceMemoryConsumers() {
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!(PageOutThreadActive),
        1,
        0,
    ) == 0
    {
        KeSetEvent(
            core::ptr::addr_of_mut!(MiBalancerEvent) as *mut _,
            IO_NO_INCREMENT as i32,
            0u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn MmRebalanceMemoryConsumersAndWait() {
    KeResetEvent(core::ptr::addr_of_mut!(MiBalancerDoneEvent) as *mut _);
    MmRebalanceMemoryConsumers();
    KeWaitForSingleObject(
        core::ptr::addr_of_mut!(MiBalancerDoneEvent) as PVOID,
        0,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn MmRequestPageMemoryConsumer(
    Consumer: ULONG,
    _CanWait: BOOLEAN,
    AllocatedPage: PPFN_NUMBER,
) -> NTSTATUS {
    static mut I: i32 = 0;
    #[cfg(target_arch = "x86")]
    static mut TINY_TIME: LARGE_INTEGER = unsafe { core::mem::transmute(-1i64) };

    I += 1;
    if I >= 100 {
        // Not drift: balance.c waits on every arch, but a relative sub-tick wait blocked
        // ~6.3s on amd64 and stalled boot (97d678e72ef); only i386 services it promptly.
        // Yield is the throttle's intent. Revisit when relative waits are sound off-i386.
        #[cfg(target_arch = "x86")]
        KeDelayExecutionThread(0i8, 0u8, core::ptr::addr_of_mut!(TINY_TIME));
        #[cfg(not(target_arch = "x86"))]
        ntkrnl_sys::ddk::NtYieldExecution();
        I = 0;
    }

    let page = MmAllocPage(Consumer) as PFN_NUMBER;
    if page == 0 {
        *AllocatedPage = 0;
        return STATUS_NO_MEMORY;
    }
    *AllocatedPage = page as _;

    InterlockedIncrementUL(
        core::ptr::addr_of_mut!(MiMemoryConsumers[Consumer as usize].PagesUsed)
    );
    UpdateTotalCommittedPages(1);

    STATUS_SUCCESS
}

unsafe extern "system" fn MiBalancerThread(_Unused: PVOID) {
    MiBalancerThreadInner();
}

unsafe fn MiBalancerThreadInner() {
    let mut wait_objects: [PVOID; 2] = [
        core::ptr::addr_of_mut!(MiBalancerEvent) as PVOID,
        core::ptr::addr_of_mut!(MiBalancerTimer) as PVOID,
    ];

    loop {
        KeSetEvent(
            core::ptr::addr_of_mut!(MiBalancerDoneEvent) as *mut _,
            IO_NO_INCREMENT as i32,
            0u8,
        );

        let status = KeWaitForMultipleObjects(
            2,
            wait_objects.as_mut_ptr(),
            0,
            0,
            0i8,
            0u8,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        if status == STATUS_WAIT_0 || status == STATUS_WAIT_1 {
            let mut initial_target: ULONG = 0;

            loop {
                let old_target = initial_target;

                for i in 0..(MC_MAXIMUM as usize) {
                    initial_target = MiTrimMemoryConsumer(i as ULONG, initial_target);
                }

                let target = core::cmp::max(
                    initial_target,
                    (MiMinimumAvailablePages as i32 - MmAvailablePages as i32).unsigned_abs(),
                );
                if target != 0 {
                    let mut nr_freed_pages: ULONG = 0;
                    CcRosTrimCache(target, &mut nr_freed_pages);
                    initial_target = initial_target.saturating_sub(nr_freed_pages);
                }

                if initial_target != 0 && initial_target == old_target {
                    // The balancer made a full pass and freed nothing, so it is
                    // wedged against a target it cannot meet: game over, on every
                    // arch. arm64 used to `break` here instead ("no pagefile"),
                    // but no arch has a pagefile on the livecd, and carrying on
                    // short of pages only turned this into a later, anonymous
                    // timeout. The guard never fired in 530 arm64 boots.
                    mm_guard_trip!(
                        "balance.MiBalancerThread no-progress", 4,
                        "target" => initial_target,
                    );
                    KeBugCheck(NO_PAGES_AVAILABLE as u32);
                }

                if initial_target == 0 {
                    break;
                }
            }

            if status == STATUS_WAIT_0 {
                InterlockedExchange(core::ptr::addr_of_mut!(PageOutThreadActive), 0);
            }
        } else {
            KeBugCheck(MEMORY_MANAGEMENT as u32);
        }
    }
}

// CODE_SEG("INIT")
#[no_mangle]
pub unsafe extern "system" fn MiInitBalancerThread() {
    KeInitializeEvent(
        core::ptr::addr_of_mut!(MiBalancerEvent) as *mut _,
        1,
        0u8,
    );
    KeInitializeEvent(
        core::ptr::addr_of_mut!(MiBalancerDoneEvent) as *mut _,
        1,
        0u8,
    );
    KeInitializeTimerEx(
        core::ptr::addr_of_mut!(MiBalancerTimer) as *mut _,
        1,
    );

    let timeout: LARGE_INTEGER = core::mem::transmute(-20000000i64);
    KeSetTimerEx(
        core::ptr::addr_of_mut!(MiBalancerTimer) as *mut _,
        timeout,
        2000,
        core::ptr::null_mut(),
    );

    let status = PsCreateSystemThread(
        core::ptr::addr_of_mut!(MiBalancerThreadHandle) as *mut _,
        0x1F03FF,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(MiBalancerThreadId),
        Some(MiBalancerThread),
        core::ptr::null_mut(),
    );
    if status < 0 {
        KeBugCheck(MEMORY_MANAGEMENT as u32);
    }

    let mut priority: KPRIORITY = LOW_REALTIME_PRIORITY as KPRIORITY + 1;
    NtSetInformationThread(
        MiBalancerThreadHandle,
        0,
        &mut priority as *mut _ as PVOID,
        core::mem::size_of::<KPRIORITY>() as ULONG,
    );
}
