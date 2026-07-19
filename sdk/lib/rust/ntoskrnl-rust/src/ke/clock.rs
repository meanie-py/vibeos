use super::ke_defs::*;
use crate::CONTAINING_RECORD;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;

use ntkrnl_sys::ddk::{
    HalSetRealTimeClock, KSYSTEM_TIME, KTIMER, LARGE_INTEGER, LIST_ENTRY, PKSPIN_LOCK_QUEUE,
    PKTIMER, PLARGE_INTEGER, TIME_FIELDS, KeRevertToUserAffinityThread, KeSetSystemAffinityThread,
    KiAcquireDispatcherLock, KiAcquireTimerLock, KiComputeTimerTableIndex, KiInsertTimerTable,
    KiReleaseTimerLock, KiRemoveEntryTimer, KiTimerListExpire, RtlTimeToTimeFields,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub static mut KeBootTime: LARGE_INTEGER = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut KeBootTimeBias: u64 = 0;
#[no_mangle]
pub static mut KeTickCount: KSYSTEM_TIME = KSYSTEM_TIME {
    LowPart: 0,
    High1Time: 0,
    High2Time: 0,
};
#[no_mangle]
pub static mut KeMaximumIncrement: ULONG = 0;
#[no_mangle]
pub static mut KeMinimumIncrement: ULONG = 0;
#[no_mangle]
pub static mut KeTimeIncrement: ULONG = 0;

extern "C" {
    static mut SharedUserData: *mut KUSER_SHARED_DATA;
    static mut ExCmosClockIsSane: BOOLEAN;
    fn YieldProcessor();
}
// Use actual definitions from sibling modules, not extern declarations (lto=false splits them)
use super::timerobj::KiTimerTableListHead;
use super::time::KeTimeAdjustment;

#[no_mangle]
pub static mut KiTickOffset: LONG = 0;

#[repr(C)]
struct KUSER_SHARED_DATA {
    _padding1: [u8; 0x8],          // TickCountLowDeprecated + TickCountMultiplier
    InterruptTime: KSYSTEM_TIME,   // offset 0x08
    SystemTime: KSYSTEM_TIME,      // offset 0x14
}

#[repr(C)]
struct KTIMER_TABLE_ENTRY {
    Entry: LIST_ENTRY,
}

const TIMER_TABLE_SIZE: usize = 512;

#[no_mangle]
pub unsafe extern "system" fn KeSetSystemTime(
    NewTime: PLARGE_INTEGER,
    OldTime: PLARGE_INTEGER,
    FixInterruptTime: BOOLEAN,
    HalTime: PLARGE_INTEGER,
) {
    let NewTime = opaque_ptr(NewTime);
    let OldTime = opaque_ptr(OldTime);
    let HalTime = opaque_ptr(HalTime);
    let mut TimeFields: TIME_FIELDS = core::mem::zeroed();
    let mut OldIrql: KIRQL = 0;
    let mut OldIrql2: KIRQL = 0;
    let mut DeltaTime: LARGE_INTEGER = core::mem::zeroed();
    let mut TempList: LIST_ENTRY = core::mem::zeroed();
    let mut TempList2: LIST_ENTRY = core::mem::zeroed();

    if !HalTime.is_null() {
        RtlTimeToTimeFields(HalTime, &mut TimeFields);
    }

    KeSetSystemAffinityThread(1);
    OldIrql = KiAcquireDispatcherLock();
    // [guard] KeRaiseIrql(HIGH_LEVEL, &mut OldIrql2);
    let irql_guard = unsafe { ntkrnl_sys::guards::IrqlGuard::raise(HIGH_LEVEL) };

    KeQuerySystemTime(OldTime as *mut _);

    // Volatile stores in protocol order (High2, Low, High1): plain stores to
    // disjoint fields are reorderable, which breaks readers' torn-read guard.
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).SystemTime.High2Time), (*NewTime).__bindgen_anon_1.HighPart);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).SystemTime.LowPart), (*NewTime).__bindgen_anon_1.LowPart);
    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).SystemTime.High1Time), (*NewTime).__bindgen_anon_1.HighPart);

    if !HalTime.is_null() {
        ExCmosClockIsSane = HalSetRealTimeClock(&mut TimeFields);
    }

    DeltaTime.QuadPart = (*NewTime).QuadPart - (*OldTime).QuadPart;

    KeBootTime.QuadPart += DeltaTime.QuadPart;
    KeBootTimeBias = KeBootTimeBias + DeltaTime.QuadPart as u64;

    drop(irql_guard);

    if FixInterruptTime != 0 {
        DbgBreakPoint();
    }

    InitializeListHead(&mut TempList);

    for i in 0..TIMER_TABLE_SIZE {
        let ListHead = core::ptr::addr_of_mut!(KiTimerTableListHead[i].Entry) as PLIST_ENTRY;
        let LockQueue = KiAcquireTimerLock(i as ULONG);
        let mut NextEntry = (*ListHead).Flink;
        while NextEntry != ListHead {
            let Timer = CONTAINING_RECORD!(NextEntry, KTIMER, TimerListEntry) as PKTIMER;
            NextEntry = (*NextEntry).Flink;

            if (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.Absolute() != 0 {
                KiRemoveEntryTimer(Timer);
                InsertTailList(
                    &mut TempList as PLIST_ENTRY,
                    core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY,
                );
            }
        }

        KiReleaseTimerLock(LockQueue);
    }

    InitializeListHead(&mut TempList2);

    while TempList.Flink != &mut TempList as PLIST_ENTRY {
        let Timer = CONTAINING_RECORD!(TempList.Flink, KTIMER, TimerListEntry) as PKTIMER;
        RemoveEntryList(core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY);

        (*Timer).DueTime.QuadPart = ((*Timer).DueTime.QuadPart as i64 - DeltaTime.QuadPart) as u64;
        let Hand = KiComputeTimerTableIndex((*Timer).DueTime.QuadPart);
        (*Timer).Header.__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_2.Hand = Hand as u8;

        let LockQueue = KiAcquireTimerLock(Hand);
        if KiInsertTimerTable(Timer, Hand) != 0 {
            KiRemoveEntryTimer(Timer);
            InsertTailList(
                &mut TempList2 as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*Timer).TimerListEntry) as PLIST_ENTRY,
            );
        }

        KiReleaseTimerLock(LockQueue);
    }

    KiTimerListExpire(&mut TempList2 as PLIST_ENTRY, OldIrql);

    KeRevertToUserAffinityThread();
}

#[no_mangle]
pub unsafe extern "system" fn KeQueryTimeIncrement() -> ULONG {
    KeMaximumIncrement
}

#[cfg(any(target_arch = "x86", target_arch = "arm"))]
#[no_mangle]
pub unsafe extern "system" fn KeQueryTickCount(TickCount: PLARGE_INTEGER) {
    let TickCount = opaque_ptr(TickCount);
    loop {
        (*TickCount).__bindgen_anon_1.HighPart = core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.High1Time));
        (*TickCount).__bindgen_anon_1.LowPart = core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.LowPart));
        if (*TickCount).__bindgen_anon_1.HighPart == core::ptr::read_volatile(core::ptr::addr_of!(KeTickCount.High2Time)) {
            break;
        }
        YieldProcessor();
    }
}

// On 64-bit (amd64/arm64), KeQueryTickCount is a macro: *(x) = KeTickCount
// Provide a callable function for Rust code that references it.
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
#[no_mangle]
pub unsafe extern "system" fn KeQueryTickCount(TickCount: PLARGE_INTEGER) {
    let TickCount = opaque_ptr(TickCount);
    *TickCount = *(core::ptr::addr_of!(KeTickCount) as *const LARGE_INTEGER);
}

#[cfg(not(target_arch = "x86_64"))]
#[cfg(not(target_arch = "aarch64"))]
#[cfg(any(target_arch = "x86", target_arch = "arm"))]
#[no_mangle]
pub unsafe extern "system" fn KeQuerySystemTime(CurrentTime: PLARGE_INTEGER) {
    let CurrentTime = opaque_ptr(CurrentTime);
    loop {
        (*CurrentTime).__bindgen_anon_1.HighPart = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High1Time));
        (*CurrentTime).__bindgen_anon_1.LowPart = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.LowPart));
        if (*CurrentTime).__bindgen_anon_1.HighPart == core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High2Time)) {
            break;
        }
        YieldProcessor();
    }
}

#[cfg(not(target_arch = "x86_64"))]
#[cfg(not(target_arch = "aarch64"))]
#[cfg(any(target_arch = "x86", target_arch = "arm"))]
#[no_mangle]
pub unsafe extern "system" fn KeQueryInterruptTime() -> u64 {
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();

    loop {
        CurrentTime.__bindgen_anon_1.HighPart = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.High1Time));
        CurrentTime.__bindgen_anon_1.LowPart = core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.LowPart));
        if CurrentTime.__bindgen_anon_1.HighPart == core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).InterruptTime.High2Time)) {
            break;
        }
        YieldProcessor();
    }

    CurrentTime.QuadPart as u64
}

#[no_mangle]
pub unsafe extern "system" fn KeSetTimeIncrement(MaxIncrement: ULONG, MinIncrement: ULONG) {
    KeMaximumIncrement = MaxIncrement;
    KeMinimumIncrement = if MinIncrement > 10000 {
        MinIncrement
    } else {
        10000
    };
    KeTimeAdjustment = MaxIncrement;
    KeTimeIncrement = MaxIncrement;
    KiTickOffset = MaxIncrement as LONG;

    // amd64: establish the 0-based system-time base HERE, at clock-init. This runs in
    // HAL clock setup (HalpInitializeClock -> KeSetTimeIncrement, Phase-0) which is
    // *before* IoInitSystem (Phase-1) arms any periodic timer. On amd64 the
    // KUSER_SHARED_DATA InterruptTime/Tick (and KeTickCount) are loader-left page
    // garbage (~2.8e14) — never explicitly zeroed (i386 starts at 0). The old fix
    // (ke/time.rs CLOCK_BASE_RESET) zeroed them on the *first clock tick*, which fires
    // AFTER IoInitSystem: by then the scsiport SRB-timeout IopTimer (1s periodic) has
    // already baked its DueTime against the garbage base, so the reset stranded it
    // forever (DueTime never <= the new 0-based InterruptTime) — the watchdog died
    // after ~4 fires, storage stalls never self-recovered, and amd64 hung ~30% of boots.
    // Zeroing here — coherent with KeMaximumIncrement/KiTickOffset/KiTickOffset above —
    // means every Phase-1 timer arms against a 0 base and KeUpdateSystemTime just
    // accumulates from 0 (exactly like i386). KeSetTimeIncrement is boot-only (runtime
    // resolution changes go through HalSetTimeIncrement), so this never rewinds time at
    // runtime. Root-caused by arm64-love + kernel32 (debugcon ISR/timer instrumentation);
    // supersedes the racy first-tick reset in 38b63f15b74 + the KiTickOffset patch 3262dfd1018.
    #[cfg(target_arch = "x86_64")]
    {
        let shared_interrupt_time = (SharedUserData as usize
            + core::mem::offset_of!(ntkrnl_sys::ddk::_KUSER_SHARED_DATA, InterruptTime)) as *mut u64;
        *shared_interrupt_time = 0;
        let shared_tick = (SharedUserData as usize
            + core::mem::offset_of!(ntkrnl_sys::ddk::_KUSER_SHARED_DATA, __bindgen_anon_2)) as *mut KSYSTEM_TIME;
        (*shared_tick).LowPart = 0;
        (*shared_tick).High1Time = 0;
        (*shared_tick).High2Time = 0;
        KeTickCount.LowPart = 0;
        KeTickCount.High1Time = 0;
        KeTickCount.High2Time = 0;
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ke/clock.c
//  * PURPOSE:         System Clock Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// LARGE_INTEGER KeBootTime;
// ULONGLONG KeBootTimeBias;
// volatile KSYSTEM_TIME KeTickCount = { 0, 0, 0 };
// ULONG KeMaximumIncrement;
// ULONG KeMinimumIncrement;
// ULONG KeTimeIncrement;
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// KeSetSystemTime(IN PLARGE_INTEGER NewTime,
//                 OUT PLARGE_INTEGER OldTime,
//                 IN BOOLEAN FixInterruptTime,
//                 IN PLARGE_INTEGER HalTime OPTIONAL)
// {
//     TIME_FIELDS TimeFields;
//     KIRQL OldIrql, OldIrql2;
//     LARGE_INTEGER DeltaTime;
//     PLIST_ENTRY ListHead, NextEntry;
//     PKTIMER Timer;
//     PKSPIN_LOCK_QUEUE LockQueue;
//     LIST_ENTRY TempList, TempList2;
//     ULONG Hand, i;
//
//     /* Sanity checks */
//     ASSERT((NewTime->HighPart & 0xF0000000) == 0);
//     ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL);
//
//     /* Check if this is for the HAL */
//     if (HalTime) RtlTimeToTimeFields(HalTime, &TimeFields);
//
//     /* Set affinity to this CPU, lock the dispatcher, and raise IRQL */
//     KeSetSystemAffinityThread(1);
//     OldIrql = KiAcquireDispatcherLock();
//     KeRaiseIrql(HIGH_LEVEL, &OldIrql2);
//
//     /* Query the system time now */
//     KeQuerySystemTime(OldTime);
//
//     /* Set the new system time (ordering of these operations is critical) */
//     SharedUserData->SystemTime.High2Time = NewTime->HighPart;
//     SharedUserData->SystemTime.LowPart = NewTime->LowPart;
//     SharedUserData->SystemTime.High1Time = NewTime->HighPart;
//
//     /* Check if this was for the HAL and set the RTC time */
//     if (HalTime) ExCmosClockIsSane = HalSetRealTimeClock(&TimeFields);
//
//     /* Calculate the difference between the new and the old time */
//     DeltaTime.QuadPart = NewTime->QuadPart - OldTime->QuadPart;
//
//     /* Update system boot time */
//     KeBootTime.QuadPart += DeltaTime.QuadPart;
//     KeBootTimeBias = KeBootTimeBias + DeltaTime.QuadPart;
//
//     /* Lower IRQL back */
//     KeLowerIrql(OldIrql2);
//
//     /* Check if we need to adjust interrupt time */
//     if (FixInterruptTime) ASSERT(FALSE);
//
//     /* Setup a temporary list of absolute timers */
//     InitializeListHead(&TempList);
//
//     /* Loop current timers */
//     for (i = 0; i < TIMER_TABLE_SIZE; i++)
//     {
//         /* Loop the entries in this table and lock the timers */
//         ListHead = &KiTimerTableListHead[i].Entry;
//         LockQueue = KiAcquireTimerLock(i);
//         NextEntry = ListHead->Flink;
//         while (NextEntry != ListHead)
//         {
//             /* Get the timer */
//             Timer = CONTAINING_RECORD(NextEntry, KTIMER, TimerListEntry);
//             NextEntry = NextEntry->Flink;
//
//             /* Is it absolute? */
//             if (Timer->Header.Absolute)
//             {
//                 /* Remove it from the timer list */
//                 KiRemoveEntryTimer(Timer);
//
//                 /* Insert it into our temporary list */
//                 InsertTailList(&TempList, &Timer->TimerListEntry);
//             }
//         }
//
//         /* Release the lock */
//         KiReleaseTimerLock(LockQueue);
//     }
//
//     /* Setup a temporary list of expired timers */
//     InitializeListHead(&TempList2);
//
//     /* Loop absolute timers */
//     while (TempList.Flink != &TempList)
//     {
//         /* Get the timer */
//         Timer = CONTAINING_RECORD(TempList.Flink, KTIMER, TimerListEntry);
//         RemoveEntryList(&Timer->TimerListEntry);
//
//         /* Update the due time and handle */
//         Timer->DueTime.QuadPart -= DeltaTime.QuadPart;
//         Hand = KiComputeTimerTableIndex(Timer->DueTime.QuadPart);
//         Timer->Header.Hand = (UCHAR)Hand;
//
//         /* Lock the timer and re-insert it */
//         LockQueue = KiAcquireTimerLock(Hand);
//         if (KiInsertTimerTable(Timer, Hand))
//         {
//             /* Remove it from the timer list */
//             KiRemoveEntryTimer(Timer);
//
//             /* Insert it into our temporary list */
//             InsertTailList(&TempList2, &Timer->TimerListEntry);
//         }
//
//         /* Release the lock */
//         KiReleaseTimerLock(LockQueue);
//     }
//
//     /* Process expired timers. This releases the dispatcher lock. */
//     KiTimerListExpire(&TempList2, OldIrql);
//
//     /* Revert affinity */
//     KeRevertToUserAffinityThread();
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// ULONG
// NTAPI
// KeQueryTimeIncrement(VOID)
// {
//     /* Return the increment */
//     return KeMaximumIncrement;
// }
//
// /*
//  * @implemented
//  */
// #undef KeQueryTickCount
// VOID
// NTAPI
// KeQueryTickCount(IN PLARGE_INTEGER TickCount)
// {
//     /* Loop until we get a perfect match */
//     for (;;)
//     {
//         /* Read the tick count value */
//         TickCount->HighPart = KeTickCount.High1Time;
//         TickCount->LowPart = KeTickCount.LowPart;
//         if (TickCount->HighPart == KeTickCount.High2Time) break;
//         YieldProcessor();
//     }
// }
//
// #if !defined(_M_AMD64) && !defined(_M_ARM64)
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeQuerySystemTime(OUT PLARGE_INTEGER CurrentTime)
// {
//     /* Loop until we get a perfect match */
//     for (;;)
//     {
//         /* Read the time value */
//         CurrentTime->HighPart = SharedUserData->SystemTime.High1Time;
//         CurrentTime->LowPart = SharedUserData->SystemTime.LowPart;
//         if (CurrentTime->HighPart ==
//             SharedUserData->SystemTime.High2Time) break;
//         YieldProcessor();
//     }
// }
//
// /*
//  * @implemented
//  */
// ULONGLONG
// NTAPI
// KeQueryInterruptTime(VOID)
// {
//     LARGE_INTEGER CurrentTime;
//
//     /* Loop until we get a perfect match */
//     for (;;)
//     {
//         /* Read the time value */
//         CurrentTime.HighPart = SharedUserData->InterruptTime.High1Time;
//         CurrentTime.LowPart = SharedUserData->InterruptTime.LowPart;
//         if (CurrentTime.HighPart ==
//             SharedUserData->InterruptTime.High2Time) break;
//         YieldProcessor();
//     }
//
//     /* Return the time value */
//     return CurrentTime.QuadPart;
// }
// #endif
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// KeSetTimeIncrement(IN ULONG MaxIncrement,
//                    IN ULONG MinIncrement)
// {
//     /* Set some Internal Variables */
//     KeMaximumIncrement = MaxIncrement;
//     KeMinimumIncrement = max(MinIncrement, 10000);
//     KeTimeAdjustment = MaxIncrement;
//     KeTimeIncrement = MaxIncrement;
//     KiTickOffset = MaxIncrement;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
