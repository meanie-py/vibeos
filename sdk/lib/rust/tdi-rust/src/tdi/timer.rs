use crate::prelude::*;
use ntkrnl_sys::ddk::{KeInitializeDpc, KeInitializeTimer, KeSetTimer, LARGE_INTEGER};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut CteTimeIncrement: LONG = 0;

#[repr(C)]
pub struct _CTE_DELAYED_EVENT {
    _private: [u8; 0],
}

pub type CTE_WORKER_ROUTINE =
    Option<unsafe extern "C" fn(*mut _CTE_DELAYED_EVENT, PVOID)>;

#[repr(C)]
pub struct _CTE_TIMER {
    pub Queued: BOOLEAN,
    pub Lock: KSPIN_LOCK,
    pub Callback: CTE_WORKER_ROUTINE,
    pub Context: PVOID,
    pub Dpc: KDPC,
    pub Timer: KTIMER,
}

pub type CTE_TIMER = _CTE_TIMER;
pub type PCTE_TIMER = *mut CTE_TIMER;

#[no_mangle]
pub unsafe extern "system" fn InternalDpcRoutine(
    Dpc: PKDPC,
    Context: *mut core::ffi::c_void,
    SystemArgument1: *mut core::ffi::c_void,
    SystemArgument2: *mut core::ffi::c_void,
) {
    let mut Timer: PCTE_TIMER = Context as PCTE_TIMER;

    /* Call our registered callback */
    (*Timer).Callback.unwrap_unchecked()(
        Timer as *mut _CTE_DELAYED_EVENT,
        (*Timer).Context,
    );
}

#[no_mangle]
pub unsafe extern "system" fn CTEInitTimer(Timer: PCTE_TIMER) {
    /* Zero all fields */
    RtlZeroMemory!(Timer as PVOID, core::mem::size_of::<CTE_TIMER>());

    /* Create a DPC and a timer */
    KeInitializeDpc(
        core::ptr::addr_of_mut!((*Timer).Dpc),
        Some(InternalDpcRoutine),
        Timer as PVOID,
    );
    KeInitializeTimer(core::ptr::addr_of_mut!((*Timer).Timer));
}

#[no_mangle]
pub unsafe extern "system" fn CTEStartTimer(
    Timer: PCTE_TIMER,
    DueTimeShort: ULONG,
    Callback: CTE_WORKER_ROUTINE,
    Context: PVOID,
) -> BOOLEAN {
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();

    /* Make sure a callback was provided */
    ASSERT!(Callback);

    /* We need to convert due time, because DueTimeShort is in ms,
       but NT timer expects 100s of ns, negative one */
    DueTime.QuadPart = -Int32x32To64!(DueTimeShort as LONG, 10000);

    /* Set other timer members */
    (*Timer).Callback = Callback;
    (*Timer).Context = Context;

    /* Set the timer */
    KeSetTimer(
        core::ptr::addr_of_mut!((*Timer).Timer),
        DueTime,
        core::ptr::addr_of_mut!((*Timer).Dpc),
    );

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn CTESystemUpTime() -> ULONG {
    let mut Ticks: LARGE_INTEGER = core::mem::zeroed();

    /* Get the tick count */
    KeQueryTickCount!(core::ptr::addr_of_mut!(Ticks) as *mut i64);

    /* Convert to 100s of ns and then to ms*/
    Ticks.QuadPart = (Ticks.QuadPart * CteTimeIncrement as i64) / 10000;

    return Ticks.__bindgen_anon_1.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn CTEInitialize() -> BOOLEAN {
    TRUE
}
