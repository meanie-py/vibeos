//! busmgr/system.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! ACPI System Driver (S-state sleep/suspend/resume). Linux-style acpi_* glue.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*;
use core::ffi::{c_char, c_int, c_void};

// Kernel types (UINT8, UINT32, SIZE_T, PVOID, NTSTATUS) via the ddk glob. ACPI_*
// types come from crate::shared (acpica-sys) — ddk has no ACPI_HANDLE, no collision.
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::NonPagedPool;

use acpica_sys::{
    AcpiEnterSleepState, AcpiEnterSleepStatePrep, AcpiGetSleepTypeData, AcpiLeaveSleepState,
    AcpiSetFirmwareWakingVector,
};

// ---- ACPICA exported global FADT table (acpica). We read AcpiGbl_FADT.SmiCommand. ----
extern "C" {
    static AcpiGbl_FADT: acpica_sys::ACPI_TABLE_FADT;
}

// ---- ACPICA tracing macros — no-ops under NDEBUG. return_VOID/return_VALUE/
//      return_ACPI_STATUS reduce to plain returns (NDEBUG build). ----
macro_rules! ACPI_FUNCTION_TRACE {
    ($($t:tt)*) => {};
}
macro_rules! ACPI_MODULE_NAME {
    ($($t:tt)*) => {};
}
macro_rules! ACPI_DEBUG_PRINT {
    ($($t:tt)*) => {};
}

// ACPI_FLUSH_CPU_CACHE() — no-op (the C macro expands to a WBINVD only on builds
// that define it; under this driver it is a no-op macro).
macro_rules! ACPI_FLUSH_CPU_CACHE {
    () => {};
}

// ---- ACPICA result macros (acexcep.h): SUCCESS(a)=!(a), FAILURE(a)=(a). AE_OK==0. ----
#[inline]
fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}
#[inline]
fn ACPI_FAILURE(s: ACPI_STATUS) -> bool {
    s != AE_OK
}

// ---- CLI/STI intrinsics (_disable()/_enable() in the C source). x86 only; arm64
//      is a no-op (FIXME(integrator): arm64 has no global CLI/STI equivalent here;
//      interrupt masking on arm64 would need DAIF manipulation — left as a no-op,
//      matching the fact that this S-state path is x86-centric). ----
#[inline]
unsafe fn _disable() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("cli", options(nomem, nostack, preserves_flags));
    // arm64: no-op (FIXME(integrator) if real masking is required)
}
#[inline]
unsafe fn _enable() {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    core::arch::asm!("sti", options(nomem, nostack, preserves_flags));
    // arm64: no-op (FIXME(integrator) if real masking is required)
}

// AE_ERROR (acexcep.h: EXCEP_ENV(0x0001)). shared.rs has AE_OK=0x0000 (env base)
// and AE_NOT_FOUND=0x0005=EXCEP_ENV(0x0005), so EXCEP_ENV(n)=n here.
// FIXME(integrator): confirm AE_ERROR == 0x0001 (acexcep.h).
const AE_ERROR: ACPI_STATUS = 0x0001;

// Pool tag 'IPCA' = ('I'<<24)|('P'<<16)|('C'<<8)|'A'.
const TAG_IPCA: u32 = u32::from_be_bytes([b'I', b'P', b'C', b'A']);

// ---- ACPI_S_STATE_COUNT (actypes.h) = ACPI_S_STATE_COUNT, S0..S5 = 6. ----
// FIXME(integrator): confirm ACPI_S_STATE_COUNT == 6 (actypes.h).
const ACPI_S_STATE_COUNT: usize = 6;

// ---- ACPI sleep-state ids (actypes.h: ACPI_STATE_S0..S5). ----
// FIXME(integrator): confirm ACPI_STATE_Sn values (actypes.h: S0=0..S5=5,
// ACPI_STATE_COUNT after S5).
const ACPI_STATE_S0: UINT32 = 0;
const ACPI_STATE_S1: UINT32 = 1;
const ACPI_STATE_S2: UINT32 = 2;
const ACPI_STATE_S3: UINT32 = 3;
const ACPI_STATE_S4: UINT32 = 4;
const ACPI_STATE_S5: UINT32 = 5;
const ACPI_STATE_COUNT: UINT32 = 6;

// ---- Driver identity strings (acpi_drivers.h / acpisys.h). NUL-terminated: these
//      are C string literals, used both by pad_name() (stops at NUL) and as %s args
//      to sprintf (which reads until NUL). ----
// FIXME(integrator): confirm exact ACPI_SYSTEM_* strings.
const ACPI_SYSTEM_DRIVER_NAME: &[u8] = b"ACPI System Driver\0";
const ACPI_SYSTEM_CLASS: &[u8] = b"system\0";
const ACPI_SYSTEM_HID: &[u8] = b"ACPI_SYS\0";
const ACPI_SYSTEM_DEVICE_NAME: &[u8] = b"System\0";

// ---- Sibling acpi-rust functions (extern "C"/cdecl; #[no_mangle] elsewhere). ----
extern "C" {
    fn acpi_bus_register_driver(driver: *mut acpi_driver) -> c_int;
    fn acpi_bus_unregister_driver(driver: *mut acpi_driver);
}

// KeStallExecutionProcessor comes from ntkrnl_sys::ddk::* (no local decl needed).

// ---- CRT (libcntpr). ----
extern "C" {
    fn sprintf(buffer: *mut c_char, format: *const c_char, ...) -> c_int;
}

// ---- file-private struct acpi_system. ----
#[repr(C)]
struct acpi_system {
    handle: ACPI_HANDLE,
    states: [UINT8; ACPI_S_STATE_COUNT],
}

// ---- acpi_driver_data(device) = (*device).driver_data (*mut c_void) ----
#[inline]
unsafe fn acpi_driver_data(device: *mut acpi_device) -> *mut c_void {
    (*device).driver_data
}
// ---- acpi_device_name(d) = (*d).pnp.device_name ([c_char;40]) ----
#[inline]
unsafe fn acpi_device_name(d: *mut acpi_device) -> *mut c_char {
    (*d).pnp.device_name.as_mut_ptr()
}
// ---- acpi_device_class(d) = (*d).pnp.device_class ([c_char;20]) ----
#[inline]
unsafe fn acpi_device_class(d: *mut acpi_device) -> *mut c_char {
    (*d).pnp.device_class.as_mut_ptr()
}
// ---- acpi_device_bid(d) = (*d).pnp.bus_id ([c_char;8]) — used only by DPRINT (no-op). ----
#[inline]
unsafe fn acpi_device_bid(d: *mut acpi_device) -> *mut c_char {
    (*d).pnp.bus_id.as_mut_ptr()
}

// ---- helper: pad a byte string into a fixed [c_char; N] name/class array ----
const fn pad_name<const N: usize>(s: &[u8]) -> [c_char; N] {
    let mut out = [0 as c_char; N];
    let mut i = 0;
    while i < s.len() && i < N {
        out[i] = s[i] as c_char;
        i += 1;
    }
    out
}

// ---- static struct acpi_driver acpi_system_driver ----
// C initializer: {{0,0}, NAME, CLASS, 0, 0, HID, {add, remove}}.
// name/class are [c_char;80] arrays, ids is *mut c_char (the HID string).
static mut acpi_system_driver: acpi_driver = acpi_driver {
    node: list_head {
        next: core::ptr::null_mut(),
        prev: core::ptr::null_mut(),
    },
    name: pad_name::<80>(ACPI_SYSTEM_DRIVER_NAME),
    class: pad_name::<80>(ACPI_SYSTEM_CLASS),
    references: 0,
    flags: 0,
    ids: ACPI_SYSTEM_HID.as_ptr() as *mut c_char,
    ops: acpi_device_ops {
        add: Some(acpi_system_add),
        remove: Some(acpi_system_remove),
        start: None,
        suspend: None,
        resume: None,
        bind: None,
        unbind: None,
        notify: None,
        scan: None,
    },
};

// `static` C fn stored in acpi_driver.ops dispatch table → extern "C".
unsafe extern "C" fn acpi_system_add(device: *mut acpi_device) -> c_int {
    let mut status: ACPI_STATUS = AE_OK;
    let mut system: *mut acpi_system = core::ptr::null_mut();
    let mut i: UINT8 = 0;

    ACPI_FUNCTION_TRACE!("acpi_system_add");

    if device.is_null() {
        return -1;
    }

    system = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<acpi_system>() as usize,
        TAG_IPCA,
    ) as *mut acpi_system;
    if system.is_null() {
        return -14;
    }
    core::ptr::write_bytes(system as *mut u8, 0, core::mem::size_of::<acpi_system>());

    (*system).handle = (*device).handle;
    sprintf(
        acpi_device_name(device),
        b"%s\0".as_ptr() as *const c_char,
        ACPI_SYSTEM_DEVICE_NAME.as_ptr(),
    );
    sprintf(
        acpi_device_class(device),
        b"%s\0".as_ptr() as *const c_char,
        ACPI_SYSTEM_CLASS.as_ptr(),
    );
    (*device).driver_data = system as *mut c_void;

    DPRINT!();
    i = 0;
    while (i as usize) < ACPI_S_STATE_COUNT {
        let mut type_a: UINT8 = 0;
        let mut type_b: UINT8 = 0;
        status = AcpiGetSleepTypeData(i, &mut type_a, &mut type_b);
        match i as UINT32 {
            ACPI_STATE_S4 => {
                if /* AcpiGbl_FACS->S4bios_f && */ AcpiGbl_FADT.SmiCommand != 0 {
                    DPRINT!();
                    (*system).states[i as usize] = 1;
                }
                // no break — fall through to default
                if ACPI_SUCCESS(status) {
                    (*system).states[i as usize] = 1;
                    DPRINT!();
                }
            }
            _ => {
                if ACPI_SUCCESS(status) {
                    (*system).states[i as usize] = 1;
                    DPRINT!();
                }
            }
        }
        i = i.wrapping_add(1);
    }

    //#ifdef CONFIG_PM  — S5 soft-off handler install: not compiled in.

    0
}

// `static` C fn stored in acpi_driver.ops dispatch table → extern "C".
unsafe extern "C" fn acpi_system_remove(device: *mut acpi_device, type_: c_int) -> c_int {
    let system: *mut acpi_system;

    ACPI_FUNCTION_TRACE!("acpi_system_remove");

    if device.is_null() || acpi_driver_data(device).is_null() {
        return -1;
    }

    system = acpi_driver_data(device) as *mut acpi_system;

    //#ifdef CONFIG_PM — S5 soft-off handler remove: not compiled in.

    ExFreePoolWithTag(system as PVOID, TAG_IPCA);

    0
}

/**
 * acpi_system_restore_state - OS-specific restoration of state
 */
#[no_mangle]
pub unsafe extern "C" fn acpi_system_restore_state(state: UINT32) -> ACPI_STATUS {
    if state > ACPI_STATE_S1 {
        //acpi_restore_state_mem();
        // _X86_: //init_8259A(0);  (comment-only block — ignored)
        /* wait for power to come back */
        KeStallExecutionProcessor(100);
    }

    /* Be really sure that irqs are disabled. */
    //ACPI_DISABLE_IRQS();

    /* Wait a little again, just in case... */
    KeStallExecutionProcessor(10);

    /* enable interrupts once again */
    //ACPI_ENABLE_IRQS();

    /* turn all the devices back on */
    //if (state > ACPI_STATE_S1) pm_send_all(PM_RESUME, ...);

    AE_OK
}

/**
 * acpi_system_save_state - save OS specific state and power down devices
 */
#[no_mangle]
pub unsafe extern "C" fn acpi_system_save_state(state: UINT32) -> ACPI_STATUS {
    let mut error: c_int = 0;

    /* Send notification ... (pm_send_all paths commented out in C) */

    /* flush caches */
    ACPI_FLUSH_CPU_CACHE!();

    /* Do arch specific saving of state. */
    if state > ACPI_STATE_S1 {
        error = 0; //acpi_save_state_mem();
                   // #if 0 acpi_save_state_disk() — not compiled.
    }
    /* disable interrupts — done by acpi_suspend on return (comment in C). */

    /* Unconditionally turn off devices (pm_send_all path commented out in C). */

    if error != 0 {
        AE_ERROR
    } else {
        AE_OK
    }
}

/****************************************************************************
 *
 * FUNCTION:    acpi_system_suspend
 *
 * DESCRIPTION: Perform OS-specific action to enter sleep state.
 *
 ****************************************************************************/
#[no_mangle]
pub unsafe extern "C" fn acpi_system_suspend(state: UINT32) -> ACPI_STATUS {
    let mut status: ACPI_STATUS = AE_ERROR;

    //local_irq_save(flags);

    match state {
        ACPI_STATE_S1 | ACPI_STATE_S5 => {
            //barrier();
            status = AcpiEnterSleepState(state as UINT8);
        }
        ACPI_STATE_S4 => {
            //do_suspend_lowlevel_s4bios(0);
        }
        _ => {}
    }

    //local_irq_restore(flags);

    status
}

/**
 * acpi_suspend - OS-agnostic system suspend/resume support (S? states)
 */
#[no_mangle]
pub unsafe extern "C" fn acpi_suspend(state: UINT32) -> ACPI_STATUS {
    let mut status: ACPI_STATUS;

    /* only support S1 and S5 on kernel 2.4 (S4 handling commented out in C) */

    AcpiEnterSleepStatePrep(state as UINT8);

    status = AcpiEnterSleepState(state as UINT8);
    if !ACPI_SUCCESS(status) && state != ACPI_STATE_S5 {
        return status;
    }

    /* disable interrupts and flush caches */
    _disable();
    ACPI_FLUSH_CPU_CACHE!();

    /* perform OS-specific sleep actions */
    status = acpi_system_suspend(state);

    /* Even if we failed to go to sleep, run these unconditionally. */
    AcpiLeaveSleepState(state as UINT8);
    acpi_system_restore_state(state);

    /* make sure interrupts are enabled */
    _enable();

    /* reset firmware waking vector */
    // NOTE: faithful to C — this call site passes 2 args.
    // FIXME(integrator): confirm acpica_sys::AcpiSetFirmwareWakingVector arity/types
    // (this ReactOS ACPICA build's signature takes 2 args here).
    AcpiSetFirmwareWakingVector(0 as acpica_sys::ACPI_PHYSICAL_ADDRESS, 0 as acpica_sys::ACPI_PHYSICAL_ADDRESS);

    status
}

#[no_mangle]
pub unsafe extern "C" fn acpi_system_init() -> c_int {
    let result: c_int;

    ACPI_FUNCTION_TRACE!("acpi_system_init");

    result = acpi_bus_register_driver(core::ptr::addr_of_mut!(acpi_system_driver));
    if result < 0 {
        return AE_NOT_FOUND as c_int;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_system_exit() {
    ACPI_FUNCTION_TRACE!("acpi_system_exit");
    acpi_bus_unregister_driver(core::ptr::addr_of_mut!(acpi_system_driver));
}
