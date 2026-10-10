use crate::prelude::*;
use ntkrnl_sys::ddk::{KeCancelTimer, KeInitializeDpc, KeInitializeTimer, KeSetTimer, KeSetTimerEx, LARGE_INTEGER, PLARGE_INTEGER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// The preamble contains only `#undef` directives, which have no Rust equivalent and define no declarations, so there is nothing to emit.
#[no_mangle]
pub unsafe extern "system" fn NdisCancelTimer(Timer: PNDIS_TIMER, TimerCancelled: PBOOLEAN) {
    ASSERT_IRQL!(DISPATCH_LEVEL);

    *TimerCancelled = KeCancelTimer(core::ptr::addr_of_mut!((*Timer).Timer));
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetCurrentSystemTime(mut pSystemTime: PLARGE_INTEGER) {
    if pSystemTime.is_null() {
        RtlAssert(
            b"pSystemTime\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/time.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            61 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    // The C calls KeQuerySystemTime (a macro over KUSER_SHARED_DATA.SystemTime on amd64/
    // arm64, an export on i386). The translation had expanded it through a `SharedSystemTime`
    // that resolve copied from storport (0x7FFE0008: InterruptTime, a user-mode address), so
    // this returned uptime instead of the time of day.
    KeQuerySystemTime(pSystemTime);
}
#[no_mangle]
pub unsafe extern "system" fn NdisInitializeTimer(
    Timer: PNDIS_TIMER,
    TimerFunction: PNDIS_TIMER_FUNCTION,
    FunctionContext: PVOID,
) {
    ASSERT_IRQL!(DISPATCH_LEVEL);

    KeInitializeTimer(core::ptr::addr_of_mut!((*Timer).Timer));

    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Timer).Dpc),
        core::mem::transmute::<PNDIS_TIMER_FUNCTION, PKDEFERRED_ROUTINE>(TimerFunction),
        FunctionContext,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DequeueMiniportTimer(mut Timer: PNDIS_MINIPORT_TIMER) -> BOOLEAN {
    let mut CurrentTimer: PNDIS_MINIPORT_TIMER = ::core::ptr::null_mut::<_NDIS_MINIPORT_TIMER>();
    if !(KeGetCurrentIrql() as i32 == DISPATCH_LEVEL as i32) {
        RtlAssert(
            b"KeGetCurrentIrql() == DISPATCH_LEVEL\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/network/ndis/ndis/time.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            98 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if (*(*Timer).Miniport).TimerQueue.is_null() {
        return FALSE as BOOLEAN;
    }
    if (*(*Timer).Miniport).TimerQueue == Timer {
        (*(*Timer).Miniport).TimerQueue = (*Timer).NextDeferredTimer as PNDIS_MINIPORT_TIMER;
        (*Timer).NextDeferredTimer = ::core::ptr::null_mut::<_NDIS_MINIPORT_TIMER>();
        return TRUE as BOOLEAN;
    } else {
        CurrentTimer = (*(*Timer).Miniport).TimerQueue;
        while !(*CurrentTimer).NextDeferredTimer.is_null() {
            if (*CurrentTimer).NextDeferredTimer == Timer {
                (*CurrentTimer).NextDeferredTimer = (*Timer).NextDeferredTimer;
                (*Timer).NextDeferredTimer = ::core::ptr::null_mut::<_NDIS_MINIPORT_TIMER>();
                return TRUE as BOOLEAN;
            }
            CurrentTimer = (*CurrentTimer).NextDeferredTimer as PNDIS_MINIPORT_TIMER;
        }
        return FALSE as BOOLEAN;
    };
}
#[no_mangle]
pub unsafe extern "system" fn NdisMCancelTimer(
    Timer: PNDIS_MINIPORT_TIMER,
    TimerCancelled: PBOOLEAN,
) {
    ASSERT_IRQL!(DISPATCH_LEVEL);

    *TimerCancelled = KeCancelTimer(core::ptr::addr_of_mut!((*Timer).Timer)) as u8;
}
#[no_mangle]
pub unsafe extern "system" fn MiniTimerDpcFunction(Dpc: PKDPC, DeferredContext: PVOID, SystemArgument1: PVOID, SystemArgument2: PVOID) {
    let mut Timer: PNDIS_MINIPORT_TIMER = DeferredContext as PNDIS_MINIPORT_TIMER;
    (*Timer).MiniportTimerFunction.unwrap_unchecked()(
        Dpc as PVOID,
        (*Timer).MiniportTimerContext,
        SystemArgument1,
        SystemArgument2,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMInitializeTimer(
    Timer: PNDIS_MINIPORT_TIMER,
    MiniportAdapterHandle: NDIS_HANDLE,
    TimerFunction: PNDIS_TIMER_FUNCTION,
    FunctionContext: PVOID,
) {
    ASSERT_IRQL!(DISPATCH_LEVEL);

    KeInitializeTimer(core::ptr::addr_of_mut!((*Timer).Timer));
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Timer).Dpc),
        Some(MiniTimerDpcFunction),
        Timer as PVOID,
    );

    (*Timer).MiniportTimerFunction = TimerFunction;
    (*Timer).MiniportTimerContext = FunctionContext;
    (*Timer).Miniport = core::ptr::addr_of_mut!(
        (*(MiniportAdapterHandle as PLOGICAL_ADAPTER)).NdisMiniportBlock
    ) as PNDIS_MINIPORT_BLOCK;
    (*Timer).NextDeferredTimer = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetPeriodicTimer(Timer: PNDIS_MINIPORT_TIMER, MillisecondsPeriod: UINT) {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    ASSERT_IRQL!(DISPATCH_LEVEL);

    /* relative delays are negative, absolute are positive; resolution is 100ns */
    Timeout.QuadPart = Int32x32To64!(MillisecondsPeriod as LONG, -10000);

    KeSetTimerEx(
        core::ptr::addr_of_mut!((*Timer).Timer),
        Timeout,
        MillisecondsPeriod as LONG,
        core::ptr::addr_of_mut!((*Timer).Dpc),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisMSetTimer(
    Timer: PNDIS_MINIPORT_TIMER,
    MillisecondsToDelay: UINT,
) {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    ASSERT_IRQL!(DISPATCH_LEVEL);

    /* relative delays are negative, absolute are positive; resolution is 100ns */
    Timeout.QuadPart = Int32x32To64!(MillisecondsToDelay, -10000);

    KeSetTimer(
        core::ptr::addr_of_mut!((*Timer).Timer),
        Timeout,
        core::ptr::addr_of_mut!((*Timer).Dpc),
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisSetTimer(Timer: PNDIS_TIMER, MillisecondsToDelay: UINT) {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();

    ASSERT_IRQL!(DISPATCH_LEVEL);

    NDIS_DbgPrint!(MAX_TRACE, b"Called. Timer is: 0x%x, Timeout is: %ld\n\0".as_ptr() as *const i8, Timer, MillisecondsToDelay);

    /* relative delays are negative, absolute are positive; resolution is 100ns */
    Timeout.QuadPart = Int32x32To64!(MillisecondsToDelay as LONG, -(10000) as LONG);

    KeSetTimer(core::ptr::addr_of_mut!((*Timer).Timer), Timeout, core::ptr::addr_of_mut!((*Timer).Dpc));
}
#[no_mangle]
pub unsafe extern "system" fn NdisSetTimerEx(
    Timer: PNDIS_TIMER,
    MillisecondsToDelay: UINT,
    FunctionContext: PVOID,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called. Timer is: 0x%x, Timeout is: %ld, FunctionContext is: 0x%x\n\0".as_ptr() as PCSTR, Timer, MillisecondsToDelay, FunctionContext);

    (*Timer).Dpc.DeferredContext = FunctionContext;

    NdisSetTimer(Timer, MillisecondsToDelay);
}
