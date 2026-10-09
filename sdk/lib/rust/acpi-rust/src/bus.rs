use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ExAcquireFastMutex, ExReleaseFastMutex, _KWAIT_REASON_Executive, FAST_MUTEX, KEVENT, KDPC, KeInitializeDpc, KeInitializeEvent, KeInsertQueueDpc, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, UINT32, UINT8};
// KeInitializeSpinLock + ExInitializeFastMutex are FORCEINLINE in the DDK (no
// exported symbol) — use the ntkrnl-sys inline impls, not a ddk extern, or the
// cohab link fails with undefined _KeInitializeSpinLock/_ExInitializeFastMutex.
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::inline::ex::ExInitializeFastMutex;
use core::ffi::{c_char, c_int, c_ulonglong, c_void};
use core::ptr::addr_of_mut;
// Driver-private + ACPICA types from the bindgen substrate (acpi_device, the
// acpi_device_* bitfield structs, ACPI_HANDLE/ACPI_STATUS, AE_* codes, list_head,
// LIST_HEAD, ACPI_BUS_COMPONENT, ACPI_ROOT_OBJECT). Replaces the LLM preamble whose
// `acpi_device` was truncated ("// other fields omitted for brevity").
use crate::shared::*;

// ---- ACPICA API functions + globals (imported by name; NO glob, which would
// collide with the ddk glob on shared primitives like BOOLEAN/UINT8 → E0659) ----
use acpica_sys::{
    AcpiAttachData, AcpiEnableSubsystem, AcpiEvaluateObject, AcpiGetData, AcpiGetHandle,
    AcpiGetName, AcpiGetNextObject, AcpiGetObjectInfo, AcpiGetParent, AcpiGetType,
    AcpiInitializeObjects, AcpiInstallNotifyHandler, AcpiOsFree, AcpiRemoveNotifyHandler,
    AcpiTerminate, AcpiWriteBitRegister,
};
// ACPICA constants that bindgen *does* emit (acpica-sys).
use acpica_sys::{
    ACPI_BITREG_POWER_BUTTON_ENABLE, ACPI_BITREG_SLEEP_BUTTON_ENABLE, ACPI_FADT_POWER_BUTTON,
    ACPI_FADT_SLEEP_BUTTON, ACPI_FULL_INITIALIZATION, ACPI_SINGLE_NAME,
    ACPI_STA_DEVICE_ENABLED, ACPI_STA_DEVICE_FUNCTIONING, ACPI_STA_DEVICE_PRESENT,
    ACPI_STA_DEVICE_UI, ACPI_SYSTEM_NOTIFY, ACPI_TYPE_ANY, ACPI_TYPE_LOCAL_SCOPE,
    ACPI_VALID_ADR, ACPI_VALID_CID, ACPI_VALID_HID, ACPI_VALID_UID,
};
// ACPICA types used by signatures/casts in this file.
use acpica_sys::{
    ACPI_BUFFER, ACPI_DEVICE_INFO, ACPI_OBJECT_TYPE, ACPI_PNP_DEVICE_ID_LIST, ACPI_SIZE,
    ACPI_STRING, ACPI_TABLE_FADT,
};

// ---- ACPICA #defines / cast-macros bindgen can't emit. Values verified against
//      vendor/acpica/include/actypes.h. ----
pub const ACPI_STATE_D0: u32 = 0; // (UINT8) 0   actypes.h:633
pub const ACPI_STATE_D1: u32 = 1; // (UINT8) 1   actypes.h:634
pub const ACPI_STATE_D2: u32 = 2; // (UINT8) 2   actypes.h:635
pub const ACPI_STATE_D3: u32 = 3; // (UINT8) 3   actypes.h:636
pub const ACPI_STATE_UNKNOWN: u32 = 0xFF; // (UINT8) 0xFF  actypes.h:622

pub const ACPI_NOTIFY_BUS_CHECK: u32 = 0x00; // actypes.h:656
pub const ACPI_NOTIFY_DEVICE_CHECK: u32 = 0x01;
pub const ACPI_NOTIFY_DEVICE_WAKE: u32 = 0x02;
pub const ACPI_NOTIFY_EJECT_REQUEST: u32 = 0x03;
pub const ACPI_NOTIFY_DEVICE_CHECK_LIGHT: u32 = 0x04;
pub const ACPI_NOTIFY_FREQUENCY_MISMATCH: u32 = 0x05;
pub const ACPI_NOTIFY_BUS_MODE_MISMATCH: u32 = 0x06;
pub const ACPI_NOTIFY_POWER_FAULT: u32 = 0x07;

// enum acpi_bus_device_type (acpi_bus.h:70) — sequential from 0.
pub const ACPI_BUS_TYPE_DEVICE: u32 = 0;
pub const ACPI_BUS_TYPE_POWER: u32 = 1;
pub const ACPI_BUS_TYPE_PROCESSOR: u32 = 2;
pub const ACPI_BUS_TYPE_THERMAL: u32 = 3;
pub const ACPI_BUS_TYPE_SYSTEM: u32 = 4;
pub const ACPI_BUS_TYPE_POWER_BUTTON: u32 = 5;
pub const ACPI_BUS_TYPE_SLEEP_BUTTON: u32 = 6;
pub const ACPI_BUS_TYPE_POWER_BUTTONF: u32 = 7;
pub const ACPI_BUS_TYPE_SLEEP_BUTTONF: u32 = 8;

// enum acpi_bus_removal_type (acpi_bus.h:63) — NORMAL = 0.
pub const ACPI_BUS_REMOVAL_NORMAL: u32 = 0;

// acpi_bus.h:119
pub const ACPI_DRIVER_ALL_NOTIFY_EVENTS: u32 = 0x1;

// ---- ACPICA result macros (acexcep.h:94-95): SUCCESS(a)=!(a), FAILURE(a)=(a).
//      AE_OK == 0, so these are exact. ----
#[inline]
fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}
#[inline]
fn ACPI_FAILURE(s: ACPI_STATUS) -> bool {
    s != AE_OK
}

// ---- ACPICA debug macros — no-ops under the driver's NDEBUG build. ----
macro_rules! ACPI_DEBUG_PRINT {
    ($($t:tt)*) => {};
}
pub const ACPI_DB_INFO: u32 = 0;
pub const ACPI_DB_ERROR: u32 = 0;

// ACPI_FREE(a) = AcpiOsFree(a)  (actypes.h:386, NDEBUG build).
#[inline]
unsafe fn ACPI_FREE(a: *mut c_void) {
    AcpiOsFree(a);
}

// ---- ACPICA exported global (linked from libacpica.a). ----
extern "C" {
    static AcpiGbl_FADT: ACPI_TABLE_FADT;
}

// ---- Linux list.h helpers (drivers/bus/acpi/include/list.h), faithful. ----
#[inline]
unsafe fn INIT_LIST_HEAD(ptr: *mut list_head) {
    (*ptr).next = ptr;
    (*ptr).prev = ptr;
}
#[inline]
unsafe fn __list_add(new: *mut list_head, prev: *mut list_head, next: *mut list_head) {
    (*next).prev = new;
    (*new).next = next;
    (*new).prev = prev;
    (*prev).next = new;
}
#[inline]
unsafe fn list_add_tail(new: *mut list_head, head: *mut list_head) {
    __list_add(new, (*head).prev, head);
}
#[inline]
unsafe fn __list_del(prev: *mut list_head, next: *mut list_head) {
    (*next).prev = prev;
    (*prev).next = next;
}
#[inline]
unsafe fn list_del(entry: *mut list_head) {
    __list_del((*entry).prev, (*entry).next);
    (*entry).next = core::ptr::null_mut();
    (*entry).prev = core::ptr::null_mut();
}
#[inline]
unsafe fn list_empty(head: *mut list_head) -> c_int {
    ((*head).next == head) as c_int
}
// list_entry(ptr, type, member) = (type*)((char*)ptr - offsetof(type, member))
macro_rules! list_entry {
    ($ptr:expr, $ty:ty, $member:ident) => {
        ($ptr as *mut u8).wrapping_sub(core::mem::offset_of!($ty, $member)) as *mut $ty
    };
}

// ---- Driver macros (acpi_bus.h). ----
// STRUCT_TO_INT(s) = (*((int*)&s)) — reinterpret first 4 bytes of a bitfield
// struct as int. Takes a *mut to the struct (matches the translated call sites).
#[inline]
unsafe fn STRUCT_TO_INT<T>(s: *mut T) -> i32 {
    *(s as *mut i32)
}
// HAS_CHILDREN(d) = ((d)->children.next != &((d)->children))
#[inline]
unsafe fn HAS_CHILDREN(d: *mut acpi_device) -> bool {
    (*d).children.next != core::ptr::addr_of_mut!((*d).children)
}
// HAS_SIBLINGS(d) = ((d)->parent && ((d)->node.next != &(d)->parent->children))
#[inline]
unsafe fn HAS_SIBLINGS(d: *mut acpi_device) -> bool {
    !(*d).parent.is_null()
        && (*d).node.next != core::ptr::addr_of_mut!((*(*d).parent).children)
}
// NODE_TO_DEVICE(n) = list_entry(n, struct acpi_device, node)
#[inline]
unsafe fn NODE_TO_DEVICE(n: *mut list_head) -> *mut acpi_device {
    (n as *mut u8).wrapping_sub(core::mem::offset_of!(acpi_device, node)) as *mut acpi_device
}

// ---- C string routines (link against CRT/ntoskrnl). ----
extern "C" {
    fn _snprintf(buf: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;
    fn sprintf(buf: *mut c_char, fmt: *const c_char, ...) -> c_int;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
}

// ---- Sibling acpi_* glue (power.c / eval.c / scan.c / drivers stay C in
//      acpi.sys; declare, never stub). Prototypes from acpi_bus.h / acpi_drivers.h.
//      NOTE: acpi_evaluate_integer / acpi_evaluate_reference are already declared
//      in crate::types and reach this file via `use crate::shared::*`. ----
extern "C" {
    fn acpi_power_get_inferred_state(device: *mut acpi_device) -> c_int;
    fn acpi_power_transition(device: *mut acpi_device, state: c_int) -> c_int;
    fn acpi_power_init() -> c_int;
    fn acpi_system_init() -> c_int;
    fn acpi_button_init() -> c_int;
    // acpi_pci_root_exit/acpi_pci_link_exit/acpi_ec_exit omitted — CONFIG_ACPI_PCI/EC
    // are undefined, so the C never calls or defines them (see acpi_bus_exit).
    fn acpi_system_exit();
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const ACPI_BUS_CLASS: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"system_bus\0") };

pub const ACPI_BUS_HID: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"ACPI_BUS\0") };

pub const ACPI_BUS_DEVICE_NAME: [::core::ffi::c_char; 11] =
    unsafe { ::core::mem::transmute::<[u8; 11], [::core::ffi::c_char; 11]>(*b"System Bus\0") };

pub const ACPI_BUTTON_HID_POWER: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"PNP0C0C\0") };

pub const ACPI_BUTTON_HID_POWERF: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"ACPI_FPB\0") };

pub const ACPI_BUTTON_HID_SLEEP: [::core::ffi::c_char; 8] =
    unsafe { ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"PNP0C0E\0") };

pub const ACPI_BUTTON_HID_SLEEPF: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"ACPI_FSB\0") };

pub const ACPI_POWER_HID: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"ACPI_PWR\0") };

pub const ACPI_PROCESSOR_HID: [::core::ffi::c_char; 10] =
    unsafe { ::core::mem::transmute::<[u8; 10], [::core::ffi::c_char; 10]>(*b"Processor\0") };

pub const ACPI_SYSTEM_HID: [::core::ffi::c_char; 9] =
    unsafe { ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"ACPI_SYS\0") };

pub const ACPI_THERMAL_HID: [::core::ffi::c_char; 12] =
    unsafe { ::core::mem::transmute::<[u8; 12], [::core::ffi::c_char; 12]>(*b"ThermalZone\0") };

pub static mut event_is_open: ::core::ffi::c_int = 0;

// #[no_mangle]: acpienum.c (still C) reads `acpi_root` as an extern global, so the
// cohab archive must export the unmangled symbol.
#[no_mangle]
pub static mut acpi_root: *mut acpi_device = ::core::ptr::null::<acpi_device>() as *mut acpi_device;

pub static mut acpi_bus_event_lock: KSPIN_LOCK = 0;

pub static mut acpi_bus_event_list: list_head = unsafe {
    list_head {
        next: &raw const acpi_bus_event_list as *mut list_head,
        prev: &raw const acpi_bus_event_list as *mut list_head,
    }
};

// Zero-init kernel objects (initialized at runtime by KeInitializeEvent/Dpc). The
// c2rust output spelled out DISPATCHER_HEADER's anonymous unions with undefined
// C2RustUnnamed_* names; a zeroed transmute is the exact same all-zero value and
// sidesteps the bindgen anon field paths entirely.
pub static mut AcpiEventQueue: KEVENT =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<KEVENT>()]) };

pub static mut event_dpc: KDPC =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<KDPC>()]) };

pub static mut PowerDeviceCount: ::core::ffi::c_int = 0;

pub static mut FixedPowerButtonCount: ::core::ffi::c_int = 0;

pub static mut PowerButtonCount: ::core::ffi::c_int = 0;

pub static mut ProcessorCount: ::core::ffi::c_int = 0;

pub static mut ThermalZoneCount: ::core::ffi::c_int = 0;

pub static mut FixedSleepButtonCount: ::core::ffi::c_int = 0;

pub static mut SleepButtonCount: ::core::ffi::c_int = 0;

static mut acpi_bus_drivers: list_head = unsafe {
    list_head {
        next: &raw const acpi_bus_drivers as *mut list_head,
        prev: &raw const acpi_bus_drivers as *mut list_head,
    }
};

static mut acpi_bus_drivers_lock: FAST_MUTEX =
    unsafe { core::mem::transmute([0u8; core::mem::size_of::<FAST_MUTEX>()]) };

pub const NDEBUG: u32 = 0;
pub const _COMPONENT: u32 = ACPI_BUS_COMPONENT;
pub const WALK_UP: i32 = 0;
pub const WALK_DOWN: i32 = 1;

pub type acpi_bus_walk_callback = Option<unsafe extern "C" fn(*mut acpi_device, i32, *mut core::ffi::c_void) -> i32>;
// NOTE: `acpi_device` comes from crate::types (full acpisys.h/acpi_bus.h layout) via
// `use crate::shared::*` — the LLM's local struct here was truncated and is removed.

unsafe fn acpi_device_register(
    device: *mut acpi_device,
    _parent: *mut acpi_device,
) -> i32 {
    let result: i32 = 0;

    if device.is_null() {
        return (0x1 | 0x1000) as i32;
    }

    result
}

unsafe fn acpi_device_unregister(device: *mut acpi_device) -> i32 {
    if device.is_null() {
        return (0x1 | 0x1000) as ACPI_STATUS as i32;
    }
    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_data_handler(
    _handle: ACPI_HANDLE,
    _context: *mut core::ffi::c_void,
) {
    // DPRINT removed
}

#[no_mangle]
pub unsafe extern "C" fn acpi_bus_get_device(
    handle: ACPI_HANDLE,
    device: *mut *mut acpi_device,
) -> c_int {
    let status: ACPI_STATUS;

    if device.is_null() {
        return AE_BAD_PARAMETER as c_int;
    }

    status = AcpiGetData(
        handle,
        Some(acpi_bus_data_handler),
        device as *mut *mut c_void,
    );

    if status != 0 || (*device).is_null() {
        return AE_NOT_FOUND as c_int;
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_get_status_handle(
    handle: ACPI_HANDLE,
    sta: *mut u64,
) -> ACPI_STATUS {
    let status = acpi_evaluate_integer(
        handle,
        b"_STA\0".as_ptr() as ACPI_STRING as ACPI_STRING,
        core::ptr::null_mut(),
        sta,
    );
    if ACPI_SUCCESS(status) {
        return AE_OK;
    }
    if status == AE_NOT_FOUND {
        core::ptr::write_volatile(sta, (ACPI_STA_DEVICE_PRESENT
            | ACPI_STA_DEVICE_ENABLED
            | ACPI_STA_DEVICE_UI
            | ACPI_STA_DEVICE_FUNCTIONING) as u64);
        return AE_OK;
    }
    status
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_get_status(device: *mut acpi_device) -> i32 {
    let status: ACPI_STATUS;
    let mut sta: u64 = 0;

    status = acpi_bus_get_status_handle((*device).handle, &mut sta);
    if ACPI_FAILURE(status) {
        return -1;
    }

    *(&mut (*device).status as *mut _ as *mut i32) = sta as i32;

    if (*device).status.functional() != 0 && (*device).status.present() == 0 {
        ACPI_DEBUG_PRINT!(
            ACPI_DB_INFO,
            b"Device [%s] status [%08x]: functional but not present;\n\0".as_ptr() as *const u8,
            (*device).pnp.bus_id.as_ptr(),
            *(&mut (*device).status as *mut _ as *mut u32),
        );
    }

    ACPI_DEBUG_PRINT!(
        ACPI_DB_INFO,
        b"Device [%s] status [%08x]\n\0".as_ptr() as *const u8,
        (*device).pnp.bus_id.as_ptr(),
        *(&mut (*device).status as *mut _ as *mut u32),
    );

    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_private_data_handler(
    _handle: ACPI_HANDLE,
    _context: *mut core::ffi::c_void,
) {
    return;
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_get_private_data(
    handle: ACPI_HANDLE,
    data: *mut *mut core::ffi::c_void,
) -> core::ffi::c_int {
    let status: ACPI_STATUS;

    if (*data).is_null() {
        return -1;
    }

    status = AcpiGetData(
        handle,
        Some(acpi_bus_private_data_handler),
        data,
    );

    if status != 0 || (*data).is_null() {
        return -1;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_bus_get_power(
    handle: ACPI_HANDLE,
    state: *mut c_int,
) -> c_int {
    let mut result: c_int;
    let status: ACPI_STATUS;
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let mut psc: c_ulonglong = 0;

    result = acpi_bus_get_device(handle, &mut device);
    if result != 0 {
        return result;
    }

    core::ptr::write_volatile(state, ACPI_STATE_UNKNOWN as c_int);

    if (*device).flags.power_manageable() == 0 {
        if !(*device).parent.is_null() {
            core::ptr::write_volatile(state, (*(*device).parent).power.state);
        } else {
            core::ptr::write_volatile(state, ACPI_STATE_D0 as c_int);
        }
    } else {
        if (*device).power.flags.explicit_get() != 0 {
            status = acpi_evaluate_integer(
                (*device).handle,
                b"_PSC\0".as_ptr() as ACPI_STRING,
                core::ptr::null_mut(),
                &mut psc,
            );
            if ACPI_FAILURE(status) {
                return AE_NOT_FOUND as c_int;
            }
            (*device).power.state = psc as c_int;
        } else if (*device).power.flags.power_resources() != 0 {
            result = acpi_power_get_inferred_state(device);
            if result != 0 {
                return result;
            }
        }

        core::ptr::write_volatile(state, (*device).power.state);
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_bus_set_power(
    handle: ACPI_HANDLE,
    state: c_int,
) -> c_int {
    let mut result: c_int;
    let status: ACPI_STATUS;
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let mut object_name: [c_char; 5] = [
        '_' as c_char,
        'P' as c_char,
        'S' as c_char,
        ('0' as c_int + state) as c_char,
        '\0' as c_char,
    ];

    result = acpi_bus_get_device(handle, &mut device);
    if result != 0 {
        return result;
    }

    if state < ACPI_STATE_D0 as c_int || state > ACPI_STATE_D3 as c_int {
        return AE_BAD_PARAMETER as c_int;
    }

    if (*device).flags.power_manageable() == 0 {
        // DPRINT removed
        return AE_NOT_FOUND as c_int;
    }

    acpi_bus_get_power((*device).handle, &mut (*device).power.state);

    if state == (*device).power.state && (*device).flags.force_power_state() == 0 {
        // DPRINT removed
        return 0;
    }

    if (*device).power.states[state as usize].flags.valid() == 0 {
        // DPRINT removed
        return AE_NOT_FOUND as c_int;
    }

    if !(*device).parent.is_null() && state < (*(*device).parent).power.state {
        // DPRINT removed
        return AE_NOT_FOUND as c_int;
    }

    if state < (*device).power.state {
        if (*device).power.flags.power_resources() != 0 {
            result = acpi_power_transition(device, state);
            if result != 0 {
                // goto end
            } else {
                if (*device).power.states[state as usize].flags.explicit_set() != 0 {
                    status = AcpiEvaluateObject(
                        (*device).handle,
                        object_name.as_mut_ptr() as ACPI_STRING,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                    );
                    if ACPI_FAILURE(status) {
                        result = AE_NOT_FOUND as c_int;
                    }
                }
            }
        } else {
            if (*device).power.states[state as usize].flags.explicit_set() != 0 {
                status = AcpiEvaluateObject(
                    (*device).handle,
                    object_name.as_mut_ptr() as ACPI_STRING,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                if ACPI_FAILURE(status) {
                    result = AE_NOT_FOUND as c_int;
                }
            }
        }
    } else {
        if (*device).power.states[state as usize].flags.explicit_set() != 0 {
            status = AcpiEvaluateObject(
                (*device).handle,
                object_name.as_mut_ptr() as ACPI_STRING,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ACPI_FAILURE(status) {
                result = AE_NOT_FOUND as c_int;
            } else {
                if (*device).power.flags.power_resources() != 0 {
                    result = acpi_power_transition(device, state);
                    if result != 0 {
                        // goto end
                    }
                }
            }
        } else {
            if (*device).power.flags.power_resources() != 0 {
                result = acpi_power_transition(device, state);
                if result != 0 {
                    // goto end
                }
            }
        }
    }

    if result != 0 {
        // DPRINT removed
    } else {
        // DPRINT removed
    }

    result
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_power_manageable(handle: ACPI_HANDLE) -> BOOLEAN {
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let result = acpi_bus_get_device(handle, &mut device);
    if result != 0 {
        0u8
    } else {
        (*device).flags.power_manageable() as u8
    }
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_can_wakeup(handle: ACPI_HANDLE) -> BOOLEAN {
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let result = acpi_bus_get_device(handle, &mut device);
    if result != 0 {
        0u8
    } else {
        (*device).wakeup.flags.valid() as u8
    }
}

unsafe fn acpi_bus_get_power_flags(device: *mut acpi_device) -> i32 {
    let mut status: ACPI_STATUS;
    let mut handle: ACPI_HANDLE = core::ptr::null_mut();
    let mut i: UINT32;

    if device.is_null() {
        return AE_NOT_FOUND as i32;
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_PSC\0".as_ptr() as ACPI_STRING,
        &mut handle,
    );
    if status == 0 {
        (*device).power.flags.set_explicit_get(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_IRC\0".as_ptr() as ACPI_STRING,
        &mut handle,
    );
    if status == 0 {
        (*device).power.flags.set_inrush_current(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_PRW\0".as_ptr() as ACPI_STRING,
        &mut handle,
    );
    if status == 0 {
        (*device).flags.set_wake_capable(1);
    }

    i = ACPI_STATE_D0 as UINT32;
    while i <= ACPI_STATE_D3 as UINT32 {
        let ps: *mut acpi_device_power_state = &mut (*device).power.states[i as usize];
        let mut object_name: [i8; 5] = [
            '_' as i8,
            'P' as i8,
            'R' as i8,
            ('0' as i32 + i as i32) as i8,
            '\0' as i8,
        ];

        status = acpi_evaluate_reference(
            (*device).handle,
            object_name.as_mut_ptr() as ACPI_STRING,
            core::ptr::null_mut(),
            &mut (*ps).resources,
        );
        if status == 0 && (*ps).resources.count != 0 {
            (*device).power.flags.set_power_resources(1);
            (*ps).flags.set_valid(1);
        }

        object_name[2] = 'S' as i8;
        status = AcpiGetHandle(
            (*device).handle,
            object_name.as_mut_ptr() as ACPI_STRING,
            &mut handle,
        );
        if status == 0 {
            (*ps).flags.set_explicit_set(1);
            (*ps).flags.set_valid(1);
        }

        if (*ps).resources.count != 0 || (*ps).flags.explicit_set() != 0 {
            (*ps).flags.set_valid(1);
        }

        (*ps).power = -1;
        (*ps).latency = -1;

        i += 1;
    }

    (*device).power.states[ACPI_STATE_D0 as usize].flags.set_valid(1);
    (*device).power.states[ACPI_STATE_D0 as usize].power = 100;
    (*device).power.states[ACPI_STATE_D3 as usize].flags.set_valid(1);
    (*device).power.states[ACPI_STATE_D3 as usize].power = 0;

    (*device).power.state = ACPI_STATE_UNKNOWN as i32;

    0
}

unsafe fn acpi_bus_get_perf_flags(device: *mut acpi_device) -> i32 {
    if device.is_null() {
        return AE_NOT_FOUND as i32;
    }

    (*device).performance.state = ACPI_STATE_UNKNOWN as i32;

    0
}

// NTAPI DPC routine. The bindgen PKDEFERRED_ROUTINE type uses `extern "stdcall"`
// on i386 and `extern "C"` on amd64/arm64; declare the matching ABI per arch so
// `Some(acpi_bus_generate_event_dpc)` unifies with KeInitializeDpc's parameter
// without a transmute (the ABIs are identical to "system", only the nominal type
// differs).
// Use the ddk `_KDPC` (the type KeInitializeDpc's PKDEFERRED_ROUTINE expects);
// `use ntkrnl_sys::ke::*` also pulls an opaque ke::KDPC whose PKDPC would not
// unify with the ddk binding.
#[no_mangle]
pub unsafe extern "system" fn acpi_bus_generate_event_dpc(
    Dpc: *mut ntkrnl_sys::ddk::_KDPC,
    DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    acpi_bus_generate_event_dpc_impl(Dpc, DeferredContext, SystemArgument1, SystemArgument2)
}

#[inline(always)]
unsafe fn acpi_bus_generate_event_dpc_impl(
    _Dpc: *mut ntkrnl_sys::ddk::_KDPC,
    _DeferredContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let device = SystemArgument1 as *mut acpi_device;
    let TypeData = SystemArgument2 as ULONG_PTR;
    let mut OldIrql: KIRQL = 0;

    let event = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<acpi_bus_event>(),
        0x65706341u32 /* 'epcA' */,
    ) as *mut acpi_bus_event;

    if event.is_null() {
        return;
    }

    sprintf(
        (*event).device_class.as_mut_ptr(),
        b"%s\0".as_ptr() as *const i8,
        (*device).pnp.device_class.as_mut_ptr(),
    );
    sprintf(
        (*event).bus_id.as_mut_ptr(),
        b"%s\0".as_ptr() as *const i8,
        (*device).pnp.bus_id.as_mut_ptr(),
    );
    (*event).type_ = ((TypeData & 0xFF000000) >> 24) as UINT32;
    (*event).data = (TypeData & 0x00FFFFFF) as UINT32;

    KeAcquireSpinLock(&raw mut acpi_bus_event_lock, &mut OldIrql);
    list_add_tail(&mut (*event).node, &raw mut acpi_bus_event_list);
    KeReleaseSpinLock(&raw mut acpi_bus_event_lock, OldIrql);

    KeSetEvent(&raw mut AcpiEventQueue as *mut _ as *mut core::ffi::c_void, IO_NO_INCREMENT as i32, FALSE as BOOLEAN);
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_generate_event(
    device: *mut acpi_device,
    type_: UINT8,
    data: i32,
) -> i32 {
    let mut TypeData: usize;

    if device.is_null() {
        return (0x1 | 0x1000) as i32;
    }

    if event_is_open == 0 {
        return 0;
    }

    TypeData = data as usize;
    TypeData |= (type_ as usize) << 24;

    KeInsertQueueDpc(&raw mut event_dpc, device as *mut _, TypeData as *mut _);

    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_receive_event(
    event: *mut acpi_bus_event,
) -> i32 {
    let entry: *mut acpi_bus_event;
    let mut old_irql: KIRQL = 0;

    if event.is_null() {
        return AE_BAD_PARAMETER as i32;
    }

    event_is_open += 1;
    KeWaitForSingleObject(
        &raw mut AcpiEventQueue as *mut _ as *mut core::ffi::c_void,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
    event_is_open -= 1;
    KeClearEvent(&raw mut AcpiEventQueue as *mut _ as *mut core::ffi::c_void);

    if list_empty(&raw mut acpi_bus_event_list) != 0 {
        return AE_NOT_FOUND as i32;
    }

    KeAcquireSpinLock(&raw mut acpi_bus_event_lock, &mut old_irql);
    entry = list_entry!(acpi_bus_event_list.next, acpi_bus_event, node);
    if !entry.is_null() {
        list_del(&mut (*entry).node);
    }
    KeReleaseSpinLock(&raw mut acpi_bus_event_lock, old_irql);

    if entry.is_null() {
        return AE_NOT_FOUND as i32;
    }

    core::ptr::copy_nonoverlapping(
        entry as *const u8,
        event as *mut u8,
        core::mem::size_of::<acpi_bus_event>(),
    );

    ExFreePoolWithTag(entry as *mut core::ffi::c_void, 0x65706341u32 /* 'epcA' */);
    0
}

unsafe fn acpi_bus_walk(
    start: *mut acpi_device,
    callback: acpi_bus_walk_callback,
    direction: i32,
    data: *mut core::ffi::c_void,
) -> i32 {
    let result: i32 = 0;
    let mut level: i32 = 0;
    let mut device: *mut acpi_device;

    if start.is_null() || callback.is_none() {
        return AE_BAD_PARAMETER as i32;
    }

    device = start;

    while !device.is_null() {
        if direction == WALK_DOWN {
            if -249 == callback.unwrap()(device, level, data) {
                break;
            }
        }

        if HAS_CHILDREN(device) {
            device = NODE_TO_DEVICE((*device).children.next);
            level += 1;
            continue;
        }

        if direction == WALK_UP {
            if -249 == callback.unwrap()(device, level, data) {
                break;
            }
        }

        if HAS_SIBLINGS(device) {
            device = NODE_TO_DEVICE((*device).node.next);
            continue;
        }

        loop {
            device = (*device).parent;
            if device.is_null() {
                break;
            }
            level -= 1;
            if HAS_SIBLINGS(device) {
                device = NODE_TO_DEVICE((*device).node.next);
                break;
            }
        }
    }

    if direction == WALK_UP && result == 0 {
        callback.unwrap()(start, level, data);
    }

    result
}

unsafe fn acpi_bus_check_device(handle: ACPI_HANDLE) {
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let status: ACPI_STATUS;
    let mut old_status: acpi_device_status ;

    if acpi_bus_get_device(handle, &mut device) != 0 {
        return;
    }
    if device.is_null() {
        return;
    }

    old_status = core::ptr::read(addr_of_mut!((*device).status));

    if !(*device).parent.is_null() && (*(*device).parent).status.present() == 0 {
        core::ptr::write(
            addr_of_mut!((*device).status),
            core::ptr::read(addr_of_mut!((*(*device).parent).status)),
        );
        return;
    }

    status = acpi_bus_get_status(device) as ACPI_STATUS;
    if ACPI_FAILURE(status) {
        return;
    }

    if STRUCT_TO_INT(addr_of_mut!(old_status)) == STRUCT_TO_INT(addr_of_mut!((*device).status)) {
        return;
    }

    if (*device).status.present() != 0 && old_status.present() == 0 {
        // DPRINT("Device insertion detected\n");
        /* TBD: Handle device insertion */
    } else if (*device).status.present() == 0 && old_status.present() != 0 {
        // DPRINT("Device removal detected\n");
        /* TBD: Handle device removal */
    }
}

unsafe fn acpi_bus_check_scope(handle: ACPI_HANDLE) {
    acpi_bus_check_device(handle);
}

unsafe extern "C" fn acpi_bus_notify(
    handle: ACPI_HANDLE,
    type_: UINT32,
    _data: *mut core::ffi::c_void,
) {
    let mut device: *mut acpi_device = core::ptr::null_mut();
    let driver: *mut acpi_driver;

    // DPRINT removed

    acpi_bus_get_device(handle, &mut device);

    match type_ {
        ACPI_NOTIFY_BUS_CHECK => {
            // DPRINT removed
            acpi_bus_check_scope(handle);
        }
        ACPI_NOTIFY_DEVICE_CHECK => {
            // DPRINT removed
            acpi_bus_check_device(handle);
        }
        ACPI_NOTIFY_DEVICE_WAKE => {
            // DPRINT removed
            acpi_bus_check_device(handle);
        }
        ACPI_NOTIFY_EJECT_REQUEST => {
            // DPRINT removed
        }
        ACPI_NOTIFY_DEVICE_CHECK_LIGHT => {
            // DPRINT removed
        }
        ACPI_NOTIFY_FREQUENCY_MISMATCH => {
            // DPRINT removed
        }
        ACPI_NOTIFY_BUS_MODE_MISMATCH => {
            // DPRINT removed
        }
        ACPI_NOTIFY_POWER_FAULT => {
            // DPRINT removed
        }
        _ => {
            // DPRINT removed
        }
    }

    if !device.is_null() {
        driver = (*device).driver;
        if !driver.is_null()
            && (*driver).ops.notify.is_some()
            && ((*driver).flags & ACPI_DRIVER_ALL_NOTIFY_EVENTS) != 0
        {
            ((*driver).ops.notify.unwrap())(device, type_);
        }
    }
}

unsafe fn acpi_bus_match(
    device: *mut acpi_device,
    driver: *mut acpi_driver,
) -> i32 {
    let mut error = 0i32;

    if (*device).flags.hardware_id() != 0 {
        if !strstr(
            (*driver).ids,
            (*device).pnp.hardware_id as *const i8,
        )
        .is_null()
        {
            return error;
        }
    }

    if (*device).flags.compatible_ids() != 0 {
        let cid_list: *mut ACPI_PNP_DEVICE_ID_LIST = (*device).pnp.cid_list;
        let mut i: i32 = 0;
        while (i as u32) < (*cid_list).Count {
            if !strstr(
                (*driver).ids,
                (*(*cid_list).Ids.as_mut_ptr().offset(i as isize)).String,
            )
            .is_null()
            {
                return error;
            }
            i += 1;
        }
    }

    error = -2;
    error
}

unsafe fn acpi_bus_driver_init(
    device: *mut acpi_device,
    driver: *mut acpi_driver,
) -> c_int {
    let mut result: c_int;

    if device.is_null() || driver.is_null() {
        return (0x1 | 0x1000) as ACPI_STATUS as c_int;
    }

    if (*driver).ops.add.is_none() {
        return -38;
    }

    result = (*driver).ops.add.expect("non-null function pointer")(device);
    if result != 0 {
        (*device).driver = core::ptr::null_mut();
        return result;
    }

    (*device).driver = driver;

    if (*driver).ops.start.is_some() {
        result = (*driver).ops.start.expect("non-null function pointer")(device);
        if result != 0 && (*driver).ops.remove.is_some() {
            (*driver).ops.remove.expect("non-null function pointer")(
                device,
                ACPI_BUS_REMOVAL_NORMAL as c_int,
            );
        }
        return result;
    }

    if (*driver).ops.scan.is_some() {
        (*driver).ops.scan.expect("non-null function pointer")(device);
    }

    0
}

unsafe extern "C" fn acpi_bus_attach(
    device: *mut acpi_device,
    _level: c_int,
    data: *mut c_void,
) -> c_int {
    let mut result: c_int;
    let driver: *mut acpi_driver;

    if device.is_null() || data.is_null() {
        return (0x1 | 0x1000) as ACPI_STATUS as c_int;
    }

    driver = data as *mut acpi_driver;

    if !(*device).driver.is_null() {
        return -9;
    }

    if (*device).status.present() == 0 {
        return (0x5 | 0x0) as ACPI_STATUS as c_int;
    }

    result = acpi_bus_match(device, driver);
    if result != 0 {
        return result;
    }

    result = acpi_bus_driver_init(device, driver);
    if result != 0 {
        return result;
    }

    ExAcquireFastMutex(&raw mut acpi_bus_drivers_lock);
    (*driver).references += 1;
    ExReleaseFastMutex(&raw mut acpi_bus_drivers_lock);

    return 0;
}

unsafe extern "C" fn acpi_bus_unattach(
    device: *mut acpi_device,
    _level: i32,
    data: *mut core::ffi::c_void,
) -> i32 {
    let result: i32;
    let driver: *mut acpi_driver = data as *mut acpi_driver;

    if device.is_null() || driver.is_null() {
        return AE_BAD_PARAMETER as i32;
    }

    if (*device).driver != driver {
        return -6;
    }

    if (*driver).ops.remove.is_none() {
        return -23;
    }

    result = (*driver).ops.remove.expect("non-null function pointer")(
        device,
        ACPI_BUS_REMOVAL_NORMAL as i32,
    );
    if result != 0 {
        return result;
    }

    (*device).driver = core::ptr::null_mut();
    (*device).driver_data = core::ptr::null_mut();

    ExAcquireFastMutex(&raw mut acpi_bus_drivers_lock);
    (*driver).references -= 1;
    ExReleaseFastMutex(&raw mut acpi_bus_drivers_lock);

    0
}

unsafe fn acpi_bus_find_driver(device: *mut acpi_device) -> c_int {
    let mut result: c_int = AE_NOT_FOUND as c_int;
    let mut entry: *mut list_head;
    let mut driver: *mut acpi_driver;

    if device.is_null() || !(*device).driver.is_null() {
        return AE_BAD_PARAMETER as c_int;
    }

    ExAcquireFastMutex(&raw mut acpi_bus_drivers_lock);

    entry = acpi_bus_drivers.next;
    while entry != &raw mut acpi_bus_drivers {
        driver = list_entry!(entry, acpi_driver, node);

        if acpi_bus_match(device, driver) != 0 {
            entry = (*entry).next;
        } else {
            result = acpi_bus_driver_init(device, driver);
            if result == 0 {
                (*driver).references += 1;
            }
            break;
        }
    }

    ExReleaseFastMutex(&raw mut acpi_bus_drivers_lock);

    result
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_register_driver(
    driver: *mut acpi_driver,
) -> i32 {
    if driver.is_null() {
        return (0x1 | 0x1000) as i32;
    }

    ExAcquireFastMutex(&raw mut acpi_bus_drivers_lock);
    list_add_tail(&mut (*driver).node, &raw mut acpi_bus_drivers);
    ExReleaseFastMutex(&raw mut acpi_bus_drivers_lock);

    acpi_bus_walk(
        acpi_root,
        Some(acpi_bus_attach),
        WALK_DOWN,
        driver as *mut core::ffi::c_void,
    );

    (*driver).references
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_unregister_driver(
    driver: *mut acpi_driver,
) {
    if driver.is_null() {
        return;
    }

    acpi_bus_walk(
        acpi_root,
        Some(acpi_bus_unattach),
        WALK_UP,
        driver as *mut core::ffi::c_void,
    );

    if (*driver).references != 0 {
        return;
    }

    ExAcquireFastMutex(&raw mut acpi_bus_drivers_lock);
    list_del(&raw mut (*driver).node);
    ExReleaseFastMutex(&raw mut acpi_bus_drivers_lock);
}

unsafe fn acpi_bus_get_flags(device: *mut acpi_device) -> i32 {
    let mut status: ACPI_STATUS;
    let mut temp: ACPI_HANDLE = core::ptr::null_mut();

    status = AcpiGetHandle(
        (*device).handle,
        b"_STA\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_SUCCESS(status) {
        (*device).flags.set_dynamic_status(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_CID\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_SUCCESS(status) {
        (*device).flags.set_compatible_ids(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_RMV\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_SUCCESS(status) {
        (*device).flags.set_removable(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_EJD\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_SUCCESS(status) {
        (*device).flags.set_ejectable(1);
    } else {
        status = AcpiGetHandle(
            (*device).handle,
            b"_EJ0\0".as_ptr() as ACPI_STRING,
            &mut temp,
        );
        if ACPI_SUCCESS(status) {
            (*device).flags.set_ejectable(1);
        }
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_LCK\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_SUCCESS(status) {
        (*device).flags.set_lockable(1);
    }

    status = AcpiGetHandle(
        (*device).handle,
        b"_PS0\0".as_ptr() as ACPI_STRING,
        &mut temp,
    );
    if ACPI_FAILURE(status) {
        status = AcpiGetHandle(
            (*device).handle,
            b"_PR0\0".as_ptr() as ACPI_STRING,
            &mut temp,
        );
    }
    if ACPI_SUCCESS(status) {
        (*device).flags.set_power_manageable(1);
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_add(
    child: *mut *mut acpi_device,
    parent: *mut acpi_device,
    handle: ACPI_HANDLE,
    type_: c_int,
) -> c_int {
    let mut result: c_int;
    let mut status: ACPI_STATUS = AE_OK;
    let device: *mut acpi_device;
    let mut bus_id: [c_char; 5] = ['?' as c_char, 0, 0, 0, 0];
    let mut buffer: ACPI_BUFFER = ACPI_BUFFER {
        Length: 0,
        Pointer: core::ptr::null_mut(),
    };
    let mut info: *mut ACPI_DEVICE_INFO = core::ptr::null_mut();
    let mut hid: *mut c_char = core::ptr::null_mut();
    let mut uid: *mut c_char = core::ptr::null_mut();
    let cid_list: *mut ACPI_PNP_DEVICE_ID_LIST;
    let mut i: c_int;
    let mut static_uid_buffer: acpi_unique_id = [0; 9];

    if child.is_null() {
        return AE_BAD_PARAMETER as c_int;
    }

    device = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<acpi_device>(),
        0x44706341u32 as ULONG  /* 'DpcA' */,
    ) as *mut acpi_device;

    if device.is_null() {
        // DPRINT removed
        return -12;
    }

    core::ptr::write_bytes(device as *mut u8, 0, core::mem::size_of::<acpi_device>());

    (*device).handle = handle;
    (*device).parent = parent;

    match type_ {
        4 => {
            _snprintf(
                (*device).pnp.bus_id.as_mut_ptr(),
                core::mem::size_of::<acpi_bus_id>(),
                b"%s\0".as_ptr() as PCHAR,
                b"ACPI\0".as_ptr() as PCHAR,
            );
        }
        7 | 5 => {
            _snprintf(
                (*device).pnp.bus_id.as_mut_ptr(),
                core::mem::size_of::<acpi_bus_id>(),
                b"%s\0".as_ptr() as PCHAR,
                b"PWRF\0".as_ptr() as PCHAR,
            );
        }
        8 | 6 => {
            _snprintf(
                (*device).pnp.bus_id.as_mut_ptr(),
                core::mem::size_of::<acpi_bus_id>(),
                b"%s\0".as_ptr() as PCHAR,
                b"SLPF\0".as_ptr() as PCHAR,
            );
        }
        _ => {
            buffer.Length = core::mem::size_of::<[c_char; 5]>() as ACPI_SIZE;
            buffer.Pointer = bus_id.as_mut_ptr() as *mut c_void;
            AcpiGetName(handle, ACPI_SINGLE_NAME as UINT32, &mut buffer);

            i = 3;
            while i > 1 {
                if bus_id[i as usize] != '_' as c_char {
                    break;
                }
                bus_id[i as usize] = 0;
                i -= 1;
            }

            _snprintf(
                (*device).pnp.bus_id.as_mut_ptr(),
                core::mem::size_of::<acpi_bus_id>(),
                b"%s\0".as_ptr() as PCHAR,
                bus_id.as_mut_ptr(),
            );
        }
    }

    // C `goto end` cleanup chain → labeled block; each `goto end` becomes
    // `break 'end`. The cleanup that followed `end:` runs after the block.
    'end: {
    result = acpi_bus_get_flags(device);
    if result != 0 {
        break 'end;
    }

    match type_ {
        0 => {
            result = acpi_bus_get_status(device);
            if result != 0 {
                break 'end;
            }
        }
        _ => {
            // STRUCT_TO_INT(device->status) = 0x0F  (write first int of bitfield)
            *(addr_of_mut!((*device).status) as *mut i32) = 0x0F;
        }
    }

    if (*device).status.present() == 0 {
        result = -2;
        break 'end;
    }

    match type_ {
        0 => {
            status = AcpiGetObjectInfo(handle, &mut info);
            if ACPI_FAILURE(status) {
                ACPI_DEBUG_PRINT!(ACPI_DB_ERROR, b"Error reading device info\n\0".as_ptr() as PCHAR);
                result = AE_NOT_FOUND as c_int;
                info = core::ptr::null_mut();
                break 'end;
            }

            if (*info).Valid as c_int & ACPI_VALID_HID as c_int != 0 {
                hid = (*info).HardwareId.String;
            }
            if (*info).Valid as c_int & ACPI_VALID_UID as c_int != 0 {
                uid = (*info).UniqueId.String;
            }
            if (*info).Valid as c_int & ACPI_VALID_CID as c_int != 0 {
                cid_list = addr_of_mut!((*info).CompatibleIdList);
                (*device).pnp.cid_list = ExAllocatePoolWithTag(
                    NonPagedPool,
                    (*cid_list).ListSize as SIZE_T,
                    0x44706341u32 as ULONG  /* 'DpcA' */,
                ) as *mut ACPI_PNP_DEVICE_ID_LIST;

                if !(*device).pnp.cid_list.is_null() {
                    let mut p: *mut c_char = (*(*device).pnp.cid_list).Ids.as_mut_ptr()
                        .offset((*cid_list).Count as isize) as *mut c_char;

                    (*(*device).pnp.cid_list).Count = (*cid_list).Count;
                    (*(*device).pnp.cid_list).ListSize = (*cid_list).ListSize;

                    i = 0;
                    while (i as UINT32) < (*cid_list).Count {
                        (*((*(*device).pnp.cid_list).Ids.as_mut_ptr().offset(i as isize))).Length =
                            (*((*cid_list).Ids.as_mut_ptr().offset(i as isize))).Length;

                        (*((*(*device).pnp.cid_list).Ids.as_mut_ptr().offset(i as isize))).String = p;

                        core::ptr::copy_nonoverlapping(
                            (*((*cid_list).Ids.as_mut_ptr().offset(i as isize))).String,
                            (*((*(*device).pnp.cid_list).Ids.as_mut_ptr().offset(i as isize))).String,
                            (*((*cid_list).Ids.as_mut_ptr().offset(i as isize))).Length as usize,
                        );

                        p = p.offset(
                            (*((*cid_list).Ids.as_mut_ptr().offset(i as isize))).Length as isize,
                        );
                        i += 1;
                    }
                } else {
                    // DPRINT removed
                }
            }

            if (*info).Valid as c_int & ACPI_VALID_ADR as c_int != 0 {
                (*device).pnp.bus_address = (*info).Address as acpi_bus_address;
                (*device).flags.set_bus_address(1);
            }
        }
        1 => {
            hid = ACPI_POWER_HID.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = PowerDeviceCount;
            PowerDeviceCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        2 => {
            hid = ACPI_PROCESSOR_HID.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = ProcessorCount;
            ProcessorCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"_%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        4 => {
            hid = ACPI_SYSTEM_HID.as_ptr() as *mut c_char;
        }
        3 => {
            hid = ACPI_THERMAL_HID.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = ThermalZoneCount;
            ThermalZoneCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        5 => {
            hid = ACPI_BUTTON_HID_POWER.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = PowerButtonCount;
            PowerButtonCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        7 => {
            hid = ACPI_BUTTON_HID_POWERF.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = FixedPowerButtonCount;
            FixedPowerButtonCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        6 => {
            hid = ACPI_BUTTON_HID_SLEEP.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = SleepButtonCount;
            SleepButtonCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        8 => {
            hid = ACPI_BUTTON_HID_SLEEPF.as_ptr() as *mut c_char;
            uid = static_uid_buffer.as_mut_ptr();
            let fresh = FixedSleepButtonCount;
            FixedSleepButtonCount += 1;
            _snprintf(
                uid,
                core::mem::size_of::<acpi_unique_id>(),
                b"%d\0".as_ptr() as PCHAR,
                fresh,
            );
        }
        _ => {}
    }

    if parent as ACPI_HANDLE == ACPI_ROOT_OBJECT && type_ == 0 {
        hid = ACPI_BUS_HID.as_ptr() as *mut c_char;
        _snprintf(
            (*device).pnp.device_name.as_mut_ptr(),
            core::mem::size_of::<acpi_device_name>(),
            b"%s\0".as_ptr() as PCHAR,
            ACPI_BUS_DEVICE_NAME.as_ptr(),
        );
        _snprintf(
            (*device).pnp.device_class.as_mut_ptr(),
            core::mem::size_of::<acpi_device_class>(),
            b"%s\0".as_ptr() as PCHAR,
            ACPI_BUS_CLASS.as_ptr(),
        );
    }

    if !hid.is_null() {
        (*device).pnp.hardware_id = ExAllocatePoolWithTag(
            NonPagedPool,
            (strlen(hid) + 1) as SIZE_T,
            0x44706341u32 as ULONG  /* 'DpcA' */,
        ) as acpi_hardware_id;

        if !(*device).pnp.hardware_id.is_null() {
            _snprintf(
                (*device).pnp.hardware_id as *mut c_char,
                strlen(hid) + 1,
                b"%s\0".as_ptr() as PCHAR,
                hid,
            );
            (*device).flags.set_hardware_id(1);
        }
    }

    if !uid.is_null() {
        _snprintf(
            (*device).pnp.unique_id.as_mut_ptr(),
            core::mem::size_of::<acpi_unique_id>(),
            b"%s\0".as_ptr() as PCHAR,
            uid,
        );
        (*device).flags.set_unique_id(1);
    }

    if (*device).flags.power_manageable() != 0 {
        result = acpi_bus_get_power_flags(device);
        if result != 0 {
            break 'end;
        }
    }

    if (*device).flags.performance_manageable() != 0 {
        result = acpi_bus_get_perf_flags(device);
        if result != 0 {
            break 'end;
        }
    }

    match type_ {
        5 | 7 | 6 | 8 => {}
        _ => {
            status = AcpiAttachData(
                (*device).handle,
                Some(acpi_bus_data_handler),
                device as *mut c_void,
            );
        }
    }

    if ACPI_FAILURE(status) {
        ACPI_DEBUG_PRINT!(ACPI_DB_ERROR, b"Error attaching device data\n\0".as_ptr() as PCHAR);
        result = AE_NOT_FOUND as c_int;
        break 'end;
    }

    INIT_LIST_HEAD(addr_of_mut!((*device).children));
    if (*device).parent.is_null() {
        INIT_LIST_HEAD(addr_of_mut!((*device).node));
    } else {
        list_add_tail(addr_of_mut!((*device).node), addr_of_mut!((*(*device).parent).children));
    }

    acpi_device_register(device, parent);

    if (*device).flags.bus_address() != 0 {
        if !(*device).parent.is_null() && (*(*device).parent).ops.bind.is_some() {
            (*(*device).parent).ops.bind.expect("non-null function pointer")(device);
        }
    }

    if (*device).flags.hardware_id() != 0 || (*device).flags.compatible_ids() != 0 {
        acpi_bus_find_driver(device);
    }
    } // 'end

    if !info.is_null() {
        ACPI_FREE(info as *mut c_void);
    }

    if result != 0 {
        if !(*device).pnp.cid_list.is_null() {
            ExFreePoolWithTag((*device).pnp.cid_list as PVOID, 0x44706341u32 as ULONG  /* 'DpcA' */);
        }
        if !(*device).pnp.hardware_id.is_null() {
            ExFreePoolWithTag((*device).pnp.hardware_id as PVOID, 0x44706341u32 as ULONG  /* 'DpcA' */);
        }
        ExFreePoolWithTag(device as PVOID, 0x44706341u32 as ULONG  /* 'DpcA' */);
        return result;
    }

    core::ptr::write_volatile(child, device);
    0
}

unsafe fn acpi_bus_remove(
    device: *mut acpi_device,
    _type_: i32,
) -> i32 {
    if device.is_null() {
        return 0x5 | 0x0;
    }

    acpi_device_unregister(device);

    if !(*device).pnp.cid_list.is_null() {
        ExFreePoolWithTag((*device).pnp.cid_list as *mut core::ffi::c_void, 0x44706341u32 /* 'DpcA' */);
    }

    if !(*device).pnp.hardware_id.is_null() {
        ExFreePoolWithTag((*device).pnp.hardware_id as *mut core::ffi::c_void, 0x44706341u32 /* 'DpcA' */);
    }

    if !device.is_null() {
        ExFreePoolWithTag(device as *mut core::ffi::c_void, 0x44706341u32 /* 'DpcA' */);
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_scan(start: *mut acpi_device) -> c_int {
    let mut status: ACPI_STATUS;
    let mut parent: *mut acpi_device;
    let mut child: *mut acpi_device = core::ptr::null_mut();
    let mut phandle: ACPI_HANDLE;
    let mut chandle: ACPI_HANDLE = core::ptr::null_mut();
    let mut type_: ACPI_OBJECT_TYPE = 0;
    let mut level: UINT32 = 1;

    if start.is_null() {
        return AE_BAD_PARAMETER as c_int;
    }

    parent = start;
    phandle = (*start).handle;

    while level > 0 && !parent.is_null() {
        status = AcpiGetNextObject(ACPI_TYPE_ANY, phandle, chandle, &mut chandle);

        if ACPI_FAILURE(status) {
            level = level.wrapping_sub(1);
            chandle = phandle;
            AcpiGetParent(phandle, &mut phandle);
            if !(*parent).parent.is_null() {
                parent = (*parent).parent;
            }
            continue;
        }

        status = AcpiGetType(chandle, &mut type_);
        if ACPI_FAILURE(status) {
            continue;
        }

        if type_ == ACPI_TYPE_LOCAL_SCOPE as ACPI_OBJECT_TYPE {
            level = level.wrapping_add(1);
            phandle = chandle;
            chandle = core::ptr::null_mut();
            continue;
        }

        match type_ {
            6 => {
                type_ = ACPI_BUS_TYPE_DEVICE as ACPI_OBJECT_TYPE;
            }
            12 => {
                type_ = ACPI_BUS_TYPE_PROCESSOR as ACPI_OBJECT_TYPE;
            }
            13 => {
                type_ = ACPI_BUS_TYPE_THERMAL as ACPI_OBJECT_TYPE;
            }
            11 => {
                type_ = ACPI_BUS_TYPE_POWER as ACPI_OBJECT_TYPE;
            }
            _ => {
                continue;
            }
        }

        status = acpi_bus_add(&mut child, parent, chandle, type_ as c_int) as ACPI_STATUS;
        if ACPI_FAILURE(status) {
            continue;
        }

        if (*child).status.present() != 0 {
            status = AcpiGetNextObject(ACPI_TYPE_ANY, chandle, core::ptr::null_mut(), core::ptr::null_mut());
            if ACPI_SUCCESS(status) {
                level = level.wrapping_add(1);
                phandle = chandle;
                chandle = core::ptr::null_mut();
                parent = child;
            }
        }
    }

    0
}

unsafe fn acpi_bus_scan_fixed(root: *mut acpi_device) -> c_int {
    let result: c_int;
    let mut device: *mut acpi_device = core::ptr::null_mut();

    if root.is_null() {
        return AE_NOT_FOUND as c_int;
    }

    if AcpiGbl_FADT.Flags as u32 & ACPI_FADT_POWER_BUTTON as u32 != 0 {
        acpi_bus_add(&mut device, acpi_root, core::ptr::null_mut(), ACPI_BUS_TYPE_POWER_BUTTON as c_int);
    } else {
        AcpiWriteBitRegister(ACPI_BITREG_POWER_BUTTON_ENABLE as UINT32, 1 as UINT32);
        acpi_bus_add(&mut device, acpi_root, core::ptr::null_mut(), ACPI_BUS_TYPE_POWER_BUTTONF as c_int);
    }

    if AcpiGbl_FADT.Flags as u32 & ACPI_FADT_SLEEP_BUTTON as u32 != 0 {
        result = acpi_bus_add(&mut device, acpi_root, core::ptr::null_mut(), ACPI_BUS_TYPE_SLEEP_BUTTON as c_int);
    } else {
        AcpiWriteBitRegister(ACPI_BITREG_SLEEP_BUTTON_ENABLE as UINT32, 1 as UINT32);
        result = acpi_bus_add(&mut device, acpi_root, core::ptr::null_mut(), ACPI_BUS_TYPE_SLEEP_BUTTONF as c_int);
    }

    result
}
#[no_mangle]
pub unsafe extern "C" fn acpi_bus_init() -> c_int {
    let mut result: c_int;
    let mut status: ACPI_STATUS;

    KeInitializeDpc(&raw mut event_dpc, Some(acpi_bus_generate_event_dpc), core::ptr::null_mut());

    status = AcpiEnableSubsystem(ACPI_FULL_INITIALIZATION);
    if ACPI_FAILURE(status) {
        // DPRINT removed
        result = AE_NOT_FOUND as c_int;
        AcpiTerminate();
        return result;
    }

    status = AcpiInitializeObjects(ACPI_FULL_INITIALIZATION);
    if ACPI_FAILURE(status) {
        // DPRINT removed
        result = AE_NOT_FOUND as c_int;
        AcpiTerminate();
        return result;
    }

    status = AcpiInstallNotifyHandler(ACPI_ROOT_OBJECT, ACPI_SYSTEM_NOTIFY, Some(acpi_bus_notify), core::ptr::null_mut());
    if ACPI_FAILURE(status) {
        // DPRINT removed
        result = AE_NOT_FOUND as c_int;
        AcpiTerminate();
        return result;
    }

    result = acpi_bus_add(&raw mut acpi_root, core::ptr::null_mut(), ACPI_ROOT_OBJECT, ACPI_BUS_TYPE_SYSTEM as c_int);
    if result != 0 {
        AcpiRemoveNotifyHandler(ACPI_ROOT_OBJECT, ACPI_SYSTEM_NOTIFY, Some(acpi_bus_notify));
        AcpiTerminate();
        return result;
    }

    result = acpi_bus_scan_fixed(acpi_root);
    if result != 0 {
        // DPRINT removed
    }

    result = acpi_bus_scan(acpi_root);
    if result != 0 {
        // DPRINT removed
    }

    0
}

unsafe fn acpi_bus_exit() {
    let mut status: ACPI_STATUS;

    // DPRINT removed

    status = AcpiRemoveNotifyHandler(
        ACPI_ROOT_OBJECT,
        ACPI_SYSTEM_NOTIFY,
        Some(acpi_bus_notify),
    );
    if ACPI_FAILURE(status) {
        // DPRINT removed
    }

    // acpi_pci_root_exit()/acpi_pci_link_exit()/acpi_ec_exit() are guarded by
    // #ifdef CONFIG_ACPI_PCI / CONFIG_ACPI_EC in bus.c — undefined in this build, so
    // the C omits them (and the functions are never defined). The LLM translated the
    // disabled code as live calls; removed to match the compiled C.
    acpi_system_exit();

    acpi_bus_remove(acpi_root, ACPI_BUS_REMOVAL_NORMAL as i32);

    status = AcpiTerminate();
    if ACPI_FAILURE(status) {
        // DPRINT removed
    } else {
        // DPRINT removed
    }
}
#[no_mangle]
pub unsafe extern "C" fn acpi_init() -> c_int {
    let result: c_int;

    // ke::KSPIN_LOCK (usize) vs ddk::KeInitializeSpinLock's ULONG_PTR pointer —
    // same width per arch, distinct nominal type → cast the pointer.
    KeInitializeSpinLock(&raw mut acpi_bus_event_lock as *mut _ as *mut _);
    KeInitializeEvent(&raw mut AcpiEventQueue, _EVENT_TYPE_NotificationEvent, 0u8);
    ExInitializeFastMutex(&raw mut acpi_bus_drivers_lock);

    result = acpi_bus_init();

    acpi_system_init();
    acpi_power_init();
    acpi_button_init();

    return result;
}
#[no_mangle]
pub unsafe extern "C" fn acpi_exit() {
    acpi_bus_exit();
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  acpi_bus.c - ACPI Bus Driver ($Revision: 80 $)
//  *
//  *  Copyright (C) 2001, 2002 Paul Diefenbaugh <paul.s.diefenbaugh@intel.com>
//  *
//  * ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or (at
//  *  your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful, but
//  *  WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  *  General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  59 Temple Place, Suite 330, Boston, MA 02111-1307 USA.
//  *
//  * ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//  */
//
//  /*
//   * Modified for ReactOS and latest ACPICA
//   * Copyright (C)2009  Samuel Serapion
//   */
//
// #include <precomp.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define _COMPONENT		ACPI_BUS_COMPONENT
// ACPI_MODULE_NAME		("acpi_bus")
//
// #define WALK_UP			0
// #define WALK_DOWN		1
//
// #define STRUCT_TO_INT(s)	(*((int*)&s))
// #define HAS_CHILDREN(d)		((d)->children.next != &((d)->children))
// #define HAS_SIBLINGS(d)		(((d)->parent) && ((d)->node.next != &(d)->parent->children))
// #define NODE_TO_DEVICE(n)	(list_entry(n, struct acpi_device, node))
//
// int			event_is_open;
// extern void acpi_pic_sci_set_trigger(unsigned int irq, UINT16 trigger);
//
// typedef int (*acpi_bus_walk_callback)(struct acpi_device*, int, void*);
//
// struct acpi_device		*acpi_root;
// KSPIN_LOCK	acpi_bus_event_lock;
// LIST_HEAD(acpi_bus_event_list);
// //DECLARE_WAIT_QUEUE_HEAD(acpi_bus_event_queue);
// KEVENT AcpiEventQueue;
// KDPC event_dpc;
//
// int ProcessorCount, PowerDeviceCount, PowerButtonCount, FixedPowerButtonCount;
// int FixedSleepButtonCount, SleepButtonCount, ThermalZoneCount;
//
// static int
// acpi_device_register (
// 	struct acpi_device	*device,
// 	struct acpi_device	*parent)
// {
// 	int			result = 0;
//
// 	if (!device)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	return_VALUE(result);
// }
//
//
// static int
// acpi_device_unregister (
// 	struct acpi_device	*device)
// {
// 	if (!device)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// #ifdef CONFIG_LDM
// 	put_device(&device->dev);
// #endif /*CONFIG_LDM*/
//
// 	return_VALUE(0);
// }
//
//
// /* --------------------------------------------------------------------------
//                                 Device Management
//    -------------------------------------------------------------------------- */
//
// void
// acpi_bus_data_handler (
// 	ACPI_HANDLE		handle,
// 	void			*context)
// {
// 	DPRINT1("acpi_bus_data_handler not implemented\n");
//
// 	/* TBD */
//
// 	return;
// }
//
//
// int
// acpi_bus_get_device (
// 	ACPI_HANDLE		handle,
// 	struct acpi_device	**device)
// {
// 	ACPI_STATUS		status = AE_OK;
//
// 	if (!device)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	/* TBD: Support fixed-feature devices */
//
// 	status = AcpiGetData(handle, acpi_bus_data_handler, (void**)device);
// 	if (ACPI_FAILURE(status) || !*device) {
// 		DPRINT( "Error getting context for object [%p]\n",
// 			handle);
// 		return_VALUE(AE_NOT_FOUND);
// 	}
//
// 	return 0;
// }
//
// ACPI_STATUS acpi_bus_get_status_handle(ACPI_HANDLE handle,
// 				       unsigned long long *sta)
// {
// 	ACPI_STATUS status;
//
// 	status = acpi_evaluate_integer(handle, "_STA", NULL, sta);
// 	if (ACPI_SUCCESS(status))
// 		return AE_OK;
//
// 	if (status == AE_NOT_FOUND) {
// 		*sta = ACPI_STA_DEVICE_PRESENT | ACPI_STA_DEVICE_ENABLED |
// 		       ACPI_STA_DEVICE_UI      | ACPI_STA_DEVICE_FUNCTIONING;
// 		return AE_OK;
// 	}
// 	return status;
// }
//
// int
// acpi_bus_get_status (
// 	struct acpi_device	*device)
// {
// 	ACPI_STATUS status;
// 	unsigned long long sta;
//
// 	status = acpi_bus_get_status_handle(device->handle, &sta);
// 	if (ACPI_FAILURE(status))
// 		return -1;
//
// 	STRUCT_TO_INT(device->status) = (int) sta;
//
// 	if (device->status.functional && !device->status.present) {
// 		ACPI_DEBUG_PRINT((ACPI_DB_INFO, "Device [%s] status [%08x]: "
// 		       "functional but not present;\n",
// 			device->pnp.bus_id,
// 			(UINT32) STRUCT_TO_INT(device->status)));
// 	}
//
// 	ACPI_DEBUG_PRINT((ACPI_DB_INFO, "Device [%s] status [%08x]\n",
// 			  device->pnp.bus_id,
// 			  (UINT32) STRUCT_TO_INT(device->status)));
// 	return 0;
// }
//
// void acpi_bus_private_data_handler(ACPI_HANDLE handle,
// 				   void *context)
// {
// 	return;
// }
//
// int acpi_bus_get_private_data(ACPI_HANDLE handle, void **data)
// {
// 	ACPI_STATUS status = AE_OK;
//
// 	if (!*data)
// 		return -1;
//
// 	status = AcpiGetData(handle, acpi_bus_private_data_handler, data);
// 	if (ACPI_FAILURE(status) || !*data) {
// 		DPRINT("No context for object [%p]\n", handle);
// 		return -1;
// 	}
//
// 	return 0;
// }
// /* --------------------------------------------------------------------------
//                                  Power Management
//    -------------------------------------------------------------------------- */
//
// int
// acpi_bus_get_power (
// 	ACPI_HANDLE		handle,
// 	int			*state)
// {
// 	int			result = 0;
// 	ACPI_STATUS             status = 0;
// 	struct acpi_device	*device = NULL;
// 	unsigned long long		psc = 0;
//
// 	result = acpi_bus_get_device(handle, &device);
// 	if (result)
// 		return_VALUE(result);
//
// 	*state = ACPI_STATE_UNKNOWN;
//
// 	if (!device->flags.power_manageable) {
// 		/* TBD: Non-recursive algorithm for walking up hierarchy */
// 		if (device->parent)
// 			*state = device->parent->power.state;
// 		else
// 			*state = ACPI_STATE_D0;
// 	}
// 	else {
// 		/*
// 		 * Get the device's power state either directly (via _PSC) or
// 		 * indirectly (via power resources).
// 		 */
// 		if (device->power.flags.explicit_get) {
// 			status = acpi_evaluate_integer(device->handle, "_PSC",
// 				NULL, &psc);
// 			if (ACPI_FAILURE(status))
// 				return_VALUE(AE_NOT_FOUND);
// 			device->power.state = (int) psc;
// 		}
// 		else if (device->power.flags.power_resources) {
// 			result = acpi_power_get_inferred_state(device);
// 			if (result)
// 				return_VALUE(result);
// 		}
//
// 		*state = device->power.state;
// 	}
//
// 	DPRINT("Device [%s] power state is D%d\n",
// 		device->pnp.bus_id, device->power.state);
//
// 	return_VALUE(0);
// }
//
//
// int
// acpi_bus_set_power (
// 	ACPI_HANDLE		handle,
// 	int			state)
// {
// 	int			result = 0;
// 	ACPI_STATUS		status = AE_OK;
// 	struct acpi_device	*device = NULL;
// 	char			object_name[5] = {'_','P','S','0'+state,'\0'};
//
//
// 	result = acpi_bus_get_device(handle, &device);
// 	if (result)
// 		return_VALUE(result);
//
// 	if ((state < ACPI_STATE_D0) || (state > ACPI_STATE_D3))
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	/* Make sure this is a valid target state */
//
// 	if (!device->flags.power_manageable) {
// 		DPRINT1( "Device is not power manageable\n");
// 		return_VALUE(AE_NOT_FOUND);
// 	}
// 	/*
// 	 * Get device's current power state
// 	 */
// 	//if (!acpi_power_nocheck) {
// 		/*
// 		 * Maybe the incorrect power state is returned on the bogus
// 		 * bios, which is different with the real power state.
// 		 * For example: the bios returns D0 state and the real power
// 		 * state is D3. OS expects to set the device to D0 state. In
// 		 * such case if OS uses the power state returned by the BIOS,
// 		 * the device can't be transisted to the correct power state.
// 		 * So if the acpi_power_nocheck is set, it is unnecessary to
// 		 * get the power state by calling acpi_bus_get_power.
// 		 */
// 		acpi_bus_get_power(device->handle, &device->power.state);
// 	//}
//
// 	if ((state == device->power.state) && !device->flags.force_power_state) {
// 		DPRINT1("Device is already at D%d\n", state);
// 		return 0;
// 	}
// 	if (!device->power.states[state].flags.valid) {
// 		DPRINT1( "Device does not support D%d\n", state);
// 		return AE_NOT_FOUND;
// 	}
// 	if (device->parent && (state < device->parent->power.state)) {
// 		DPRINT1( "Cannot set device to a higher-powered state than parent\n");
// 		return AE_NOT_FOUND;
// 	}
//
// 	/*
// 	 * Transition Power
// 	 * ----------------
// 	 * On transitions to a high-powered state we first apply power (via
// 	 * power resources) then evalute _PSx.  Conversely for transitions to
// 	 * a lower-powered state.
// 	 */
// 	if (state < device->power.state) {
// 		if (device->power.flags.power_resources) {
// 			result = acpi_power_transition(device, state);
// 			if (result)
// 				goto end;
// 		}
// 		if (device->power.states[state].flags.explicit_set) {
// 			status = AcpiEvaluateObject(device->handle,
// 				object_name, NULL, NULL);
// 			if (ACPI_FAILURE(status)) {
// 				result = AE_NOT_FOUND;
// 				goto end;
// 			}
// 		}
// 	}
// 	else {
// 		if (device->power.states[state].flags.explicit_set) {
// 			status = AcpiEvaluateObject(device->handle,
// 				object_name, NULL, NULL);
// 			if (ACPI_FAILURE(status)) {
// 				result = AE_NOT_FOUND;
// 				goto end;
// 			}
// 		}
// 		if (device->power.flags.power_resources) {
// 			result = acpi_power_transition(device, state);
// 			if (result)
// 				goto end;
// 		}
// 	}
//
// end:
// 	if (result)
// 		DPRINT( "Error transitioning device [%s] to D%d\n",
// 			device->pnp.bus_id, state);
// 	else
// 		DPRINT("Device [%s] transitioned to D%d\n",
// 			device->pnp.bus_id, state);
//
// 	return result;
// }
//
// BOOLEAN acpi_bus_power_manageable(ACPI_HANDLE handle)
// {
// 	struct acpi_device *device;
// 	int result;
//
// 	result = acpi_bus_get_device(handle, &device);
// 	return result ? 0 : device->flags.power_manageable;
// }
//
// BOOLEAN acpi_bus_can_wakeup(ACPI_HANDLE handle)
// {
// 	struct acpi_device *device;
// 	int result;
//
// 	result = acpi_bus_get_device(handle, &device);
// 	return result ? 0 : device->wakeup.flags.valid;
// }
//
// static int
// acpi_bus_get_power_flags (
// 	struct acpi_device	*device)
// {
// 	ACPI_STATUS             status = 0;
// 	ACPI_HANDLE		handle = 0;
// 	UINT32                     i = 0;
//
// 	if (!device)
// 		return AE_NOT_FOUND;
//
// 	/*
// 	 * Power Management Flags
// 	 */
// 	status = AcpiGetHandle(device->handle, "_PSC", &handle);
// 	if (ACPI_SUCCESS(status))
// 		device->power.flags.explicit_get = 1;
// 	status = AcpiGetHandle(device->handle, "_IRC", &handle);
// 	if (ACPI_SUCCESS(status))
// 		device->power.flags.inrush_current = 1;
// 	status = AcpiGetHandle(device->handle, "_PRW", &handle);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.wake_capable = 1;
//
// 	/*
// 	 * Enumerate supported power management states
// 	 */
// 	for (i = ACPI_STATE_D0; i <= ACPI_STATE_D3; i++) {
// 		struct acpi_device_power_state *ps = &device->power.states[i];
// 		char		object_name[5] = {'_','P','R','0'+i,'\0'};
//
// 		/* Evaluate "_PRx" to se if power resources are referenced */
// 		status = acpi_evaluate_reference(device->handle, object_name, NULL,
// 			&ps->resources);
// 		if (ACPI_SUCCESS(status) && ps->resources.count) {
// 			device->power.flags.power_resources = 1;
// 			ps->flags.valid = 1;
// 		}
//
// 		/* Evaluate "_PSx" to see if we can do explicit sets */
// 		object_name[2] = 'S';
// 		status = AcpiGetHandle(device->handle, object_name, &handle);
// 		if (ACPI_SUCCESS(status)) {
// 			ps->flags.explicit_set = 1;
// 			ps->flags.valid = 1;
// 		}
//
// 		/* State is valid if we have some power control */
// 		if (ps->resources.count || ps->flags.explicit_set)
// 			ps->flags.valid = 1;
//
// 		ps->power = -1;		/* Unknown - driver assigned */
// 		ps->latency = -1;	/* Unknown - driver assigned */
// 	}
//
// 	/* Set defaults for D0 and D3 states (always valid) */
// 	device->power.states[ACPI_STATE_D0].flags.valid = 1;
// 	device->power.states[ACPI_STATE_D0].power = 100;
// 	device->power.states[ACPI_STATE_D3].flags.valid = 1;
// 	device->power.states[ACPI_STATE_D3].power = 0;
//
// 	device->power.state = ACPI_STATE_UNKNOWN;
//
// 	return 0;
// }
//
// /* --------------------------------------------------------------------------
//                               Performance Management
//    -------------------------------------------------------------------------- */
//
// static int
// acpi_bus_get_perf_flags (
// 	struct acpi_device	*device)
// {
// 	if (!device)
// 		return AE_NOT_FOUND;
//
// 	device->performance.state = ACPI_STATE_UNKNOWN;
//
// 	return 0;
// }
//
//
// /* --------------------------------------------------------------------------
//                                 Event Management
//    -------------------------------------------------------------------------- */
//
// void
// NTAPI
// acpi_bus_generate_event_dpc(PKDPC Dpc,
//                             PVOID DeferredContext,
//                             PVOID SystemArgument1,
//                             PVOID SystemArgument2)
// {
// 	struct acpi_bus_event *event;
//     struct acpi_device *device = SystemArgument1;
//     ULONG_PTR TypeData = (ULONG_PTR)SystemArgument2;
// 	KIRQL OldIrql;
//
//     event = ExAllocatePoolWithTag(NonPagedPool,sizeof(struct acpi_bus_event), 'epcA');
// 	if (!event)
// 		return;
//
// 	sprintf(event->device_class, "%s", device->pnp.device_class);
// 	sprintf(event->bus_id, "%s", device->pnp.bus_id);
// 	event->type = (TypeData & 0xFF000000) >> 24;
// 	event->data = (TypeData & 0x00FFFFFF);
//
// 	KeAcquireSpinLock(&acpi_bus_event_lock, &OldIrql);
// 	list_add_tail(&event->node, &acpi_bus_event_list);
// 	KeReleaseSpinLock(&acpi_bus_event_lock, OldIrql);
//
// 	KeSetEvent(&AcpiEventQueue, IO_NO_INCREMENT, FALSE);
// }
//
// int
// acpi_bus_generate_event (
// 	struct acpi_device	*device,
// 	UINT8			type,
// 	int			data)
// {
//     ULONG_PTR TypeData = 0;
//
// 	DPRINT("acpi_bus_generate_event\n");
//
// 	if (!device)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	/* drop event on the floor if no one's listening */
// 	if (!event_is_open)
// 		return_VALUE(0);
//
//     /* Data shouldn't even get near 24 bits */
//     ASSERT(!(data & 0xFF000000));
//
//     TypeData = data;
//     TypeData |= type << 24;
//
// 	KeInsertQueueDpc(&event_dpc, device, (PVOID)TypeData);
//
// 	return_VALUE(0);
// }
//
// int
// acpi_bus_receive_event (
// 	struct acpi_bus_event	*event)
// {
// //	unsigned long		flags = 0;
// 	struct acpi_bus_event	*entry = NULL;
// 	KIRQL OldIrql;
//
// 	//DECLARE_WAITQUEUE(wait, current);
//
// 	DPRINT("acpi_bus_receive_event\n");
//
// 	if (!event)
// 		return AE_BAD_PARAMETER;
//
// 	event_is_open++;
// 	KeWaitForSingleObject(&AcpiEventQueue,
// 			      Executive,
// 			      KernelMode,
// 			      FALSE,
// 			      NULL);
// 	event_is_open--;
// 	KeClearEvent(&AcpiEventQueue);
//
// 	if (list_empty(&acpi_bus_event_list))
// 		return_VALUE(AE_NOT_FOUND);
//
// //	spin_lock_irqsave(&acpi_bus_event_lock, flags);
// 	KeAcquireSpinLock(&acpi_bus_event_lock, &OldIrql);
// 	entry = list_entry(acpi_bus_event_list.next, struct acpi_bus_event, node);
// 	if (entry)
// 		list_del(&entry->node);
// 	KeReleaseSpinLock(&acpi_bus_event_lock, OldIrql);
// //	spin_unlock_irqrestore(&acpi_bus_event_lock, flags);
//
// 	if (!entry)
// 		return_VALUE(AE_NOT_FOUND);
//
// 	memcpy(event, entry, sizeof(struct acpi_bus_event));
//
// 	ExFreePoolWithTag(entry, 'epcA');
// 	return_VALUE(0);
// }
//
//
// /* --------------------------------------------------------------------------
//                                Namespace Management
//    -------------------------------------------------------------------------- */
//
//
// /**
//  * acpi_bus_walk
//  * -------------
//  * Used to walk the ACPI Bus's device namespace.  Can walk down (depth-first)
//  * or up.  Able to parse starting at any node in the namespace.  Note that a
//  * callback return value of -249 will terminate the walk.
//  *
//  * @start:	starting point
//  * callback:	function to call for every device encountered while parsing
//  * direction:	direction to parse (up or down)
//  * @data:	context for this search operation
//  */
// static int
// acpi_bus_walk (
// 	struct acpi_device	*start,
// 	acpi_bus_walk_callback	callback,
// 	int			direction,
// 	void			*data)
// {
// 	int			result = 0;
// 	int			level = 0;
// 	struct acpi_device	*device = NULL;
//
// 	if (!start || !callback)
// 		return AE_BAD_PARAMETER;
//
// 	device = start;
//
// 	/*
// 	 * Parse Namespace
// 	 * ---------------
// 	 * Parse a given subtree (specified by start) in the given direction.
// 	 * Walking 'up' simply means that we execute the callback on leaf
// 	 * devices prior to their parents (useful for things like removing
// 	 * or powering down a subtree).
// 	 */
//
// 	while (device) {
//
// 		if (direction == WALK_DOWN)
// 			if (-249 == callback(device, level, data))
// 				break;
//
// 		/* Depth First */
//
// 		if (HAS_CHILDREN(device)) {
// 			device = NODE_TO_DEVICE(device->children.next);
// 			++level;
// 			continue;
// 		}
//
// 		if (direction == WALK_UP)
// 			if (-249 == callback(device, level, data))
// 				break;
//
// 		/* Now Breadth */
//
// 		if (HAS_SIBLINGS(device)) {
// 			device = NODE_TO_DEVICE(device->node.next);
// 			continue;
// 		}
//
// 		/* Scope Exhausted - Find Next */
//
// 		while ((device = device->parent)) {
// 			--level;
// 			if (HAS_SIBLINGS(device)) {
// 				device = NODE_TO_DEVICE(device->node.next);
// 				break;
// 			}
// 		}
// 	}
//
// 	if ((direction == WALK_UP) && (result == 0))
// 		callback(start, level, data);
//
// 	return result;
// }
//
//
// /* --------------------------------------------------------------------------
//                              Notification Handling
//    -------------------------------------------------------------------------- */
//
// static void
// acpi_bus_check_device (ACPI_HANDLE handle)
// {
// 	struct acpi_device *device;
// 	ACPI_STATUS status = 0;
// 	struct acpi_device_status old_status;
//
// 	if (acpi_bus_get_device(handle, &device))
// 		return;
// 	if (!device)
// 		return;
//
// 	old_status = device->status;
//
// 	/*
// 	 * Make sure this device's parent is present before we go about
// 	 * messing with the device.
// 	 */
// 	if (device->parent && !device->parent->status.present) {
// 		device->status = device->parent->status;
// 		return;
// 	}
//
// 	status = acpi_bus_get_status(device);
// 	if (ACPI_FAILURE(status))
// 		return;
//
// 	if (STRUCT_TO_INT(old_status) == STRUCT_TO_INT(device->status))
// 		return;
//
//
// 	/*
// 	 * Device Insertion/Removal
// 	 */
// 	if ((device->status.present) && !(old_status.present)) {
// 		DPRINT("Device insertion detected\n");
// 		/* TBD: Handle device insertion */
// 	}
// 	else if (!(device->status.present) && (old_status.present)) {
// 		DPRINT("Device removal detected\n");
// 		/* TBD: Handle device removal */
// 	}
//
// }
//
//
// static void
// acpi_bus_check_scope (ACPI_HANDLE handle)
// {
// 	/* Status Change? */
// 	acpi_bus_check_device(handle);
//
// 	/*
// 	 * TBD: Enumerate child devices within this device's scope and
// 	 *       run acpi_bus_check_device()'s on them.
// 	 */
// }
//
//
// /**
//  * acpi_bus_notify
//  * ---------------
//  * Callback for all 'system-level' device notifications (values 0x00-0x7F).
//  */
// static void
// acpi_bus_notify (
// 	ACPI_HANDLE             handle,
// 	UINT32                     type,
// 	void                    *data)
// {
// 	struct acpi_device *device = NULL;
// 	struct acpi_driver *driver;
//
// 	DPRINT1("Notification %#02x to handle %p\n", type, handle);
//
// 	//blocking_notifier_call_chain(&acpi_bus_notify_list,
// 	//	type, (void *)handle);
//
// 	acpi_bus_get_device(handle, &device);
//
// 	switch (type) {
//
// 	case ACPI_NOTIFY_BUS_CHECK:
// 		DPRINT("Received BUS CHECK notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		acpi_bus_check_scope(handle);
// 		/*
// 		 * TBD: We'll need to outsource certain events to non-ACPI
// 		 *	drivers via the device manager (device.c).
// 		 */
// 		break;
//
// 	case ACPI_NOTIFY_DEVICE_CHECK:
// 		DPRINT("Received DEVICE CHECK notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		acpi_bus_check_device(handle);
// 		/*
// 		 * TBD: We'll need to outsource certain events to non-ACPI
// 		 *	drivers via the device manager (device.c).
// 		 */
// 		break;
//
// 	case ACPI_NOTIFY_DEVICE_WAKE:
// 		DPRINT("Received DEVICE WAKE notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		acpi_bus_check_device(handle);
// 		/*
// 		 * TBD: We'll need to outsource certain events to non-ACPI
// 		 *      drivers via the device manager (device.c).
// 		 */
// 		break;
//
// 	case ACPI_NOTIFY_EJECT_REQUEST:
// 		DPRINT1("Received EJECT REQUEST notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		/* TBD */
// 		break;
//
// 	case ACPI_NOTIFY_DEVICE_CHECK_LIGHT:
// 		DPRINT1("Received DEVICE CHECK LIGHT notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		/* TBD: Exactly what does 'light' mean? */
// 		break;
//
// 	case ACPI_NOTIFY_FREQUENCY_MISMATCH:
// 		DPRINT1("Received FREQUENCY MISMATCH notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		/* TBD */
// 		break;
//
// 	case ACPI_NOTIFY_BUS_MODE_MISMATCH:
// 		DPRINT1("Received BUS MODE MISMATCH notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		/* TBD */
// 		break;
//
// 	case ACPI_NOTIFY_POWER_FAULT:
// 		DPRINT1("Received POWER FAULT notification for device [%s]\n",
// 			device ? device->pnp.bus_id : "n/a");
// 		/* TBD */
// 		break;
//
// 	default:
// 		DPRINT1("Received unknown/unsupported notification [%08x] for device [%s]\n",
// 			type, device ? device->pnp.bus_id : "n/a");
// 		break;
// 	}
//
// 	if (device) {
// 		driver = device->driver;
// 		if (driver && driver->ops.notify &&
// 		    (driver->flags & ACPI_DRIVER_ALL_NOTIFY_EVENTS))
// 			driver->ops.notify(device, type);
// 	}
// }
//
//
// /* --------------------------------------------------------------------------
//                                  Driver Management
//    -------------------------------------------------------------------------- */
//
//
// static LIST_HEAD(acpi_bus_drivers);
// //static DECLARE_MUTEX(acpi_bus_drivers_lock);
// static FAST_MUTEX acpi_bus_drivers_lock;
//
//
// /**
//  * acpi_bus_match
//  * --------------
//  * Checks the device's hardware (_HID) or compatible (_CID) ids to see if it
//  * matches the specified driver's criteria.
//  */
// static int
// acpi_bus_match (
// 	struct acpi_device	*device,
// 	struct acpi_driver	*driver)
// {
// 	int error = 0;
//
// 	if (device->flags.hardware_id)
// 		if (strstr(driver->ids, device->pnp.hardware_id))
// 			goto Done;
//
// 	if (device->flags.compatible_ids) {
// 		ACPI_PNP_DEVICE_ID_LIST *cid_list = device->pnp.cid_list;
// 		int i;
//
// 		/* compare multiple _CID entries against driver ids */
// 		for (i = 0; i < cid_list->Count; i++)
// 		{
// 			if (strstr(driver->ids, cid_list->Ids[i].String))
// 				goto Done;
// 		}
// 	}
// 	error = -2;
//
//  Done:
//
// 	return error;
// }
//
//
// /**
//  * acpi_bus_driver_init
//  * --------------------
//  * Used to initialize a device via its device driver.  Called whenever a
//  * driver is bound to a device.  Invokes the driver's add() and start() ops.
//  */
// static int
// acpi_bus_driver_init (
// 	struct acpi_device	*device,
// 	struct acpi_driver	*driver)
// {
// 	int			result = 0;
//
// 	if (!device || !driver)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	if (!driver->ops.add)
// 		return_VALUE(-38);
//
// 	result = driver->ops.add(device);
// 	if (result) {
// 		device->driver = NULL;
// 		//acpi_driver_data(device) = NULL;
// 		return_VALUE(result);
// 	}
//
// 	device->driver = driver;
//
// 	/*
// 	 * TBD - Configuration Management: Assign resources to device based
// 	 * upon possible configuration and currently allocated resources.
// 	 */
//
// 	if (driver->ops.start) {
// 		result = driver->ops.start(device);
// 		if (result && driver->ops.remove)
// 			driver->ops.remove(device, ACPI_BUS_REMOVAL_NORMAL);
// 		return_VALUE(result);
// 	}
//
// 	DPRINT("Driver successfully bound to device\n");
//
// 	if (driver->ops.scan) {
// 		driver->ops.scan(device);
// 	}
//
// 	return_VALUE(0);
// }
//
//
// /**
//  * acpi_bus_attach
//  * -------------
//  * Callback for acpi_bus_walk() used to find devices that match a specific
//  * driver's criteria and then attach the driver.
//  */
// static int
// acpi_bus_attach (
// 	struct acpi_device	*device,
// 	int			level,
// 	void			*data)
// {
// 	int			result = 0;
// 	struct acpi_driver	*driver = NULL;
//
// 	if (!device || !data)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	driver = (struct acpi_driver *) data;
//
// 	if (device->driver)
// 		return_VALUE(-9);
//
// 	if (!device->status.present)
// 		return_VALUE(AE_NOT_FOUND);
//
// 	result = acpi_bus_match(device, driver);
// 	if (result)
// 		return_VALUE(result);
//
// 	DPRINT("Found driver [%s] for device [%s]\n",
// 		driver->name, device->pnp.bus_id);
//
// 	result = acpi_bus_driver_init(device, driver);
// 	if (result)
// 		return_VALUE(result);
//
// 	down(&acpi_bus_drivers_lock);
// 	++driver->references;
// 	up(&acpi_bus_drivers_lock);
//
// 	return_VALUE(0);
// }
//
//
// /**
//  * acpi_bus_unattach
//  * -----------------
//  * Callback for acpi_bus_walk() used to find devices that match a specific
//  * driver's criteria and unattach the driver.
//  */
// static int
// acpi_bus_unattach (
// 	struct acpi_device	*device,
// 	int			level,
// 	void			*data)
// {
// 	int			result = 0;
// 	struct acpi_driver	*driver = (struct acpi_driver *) data;
//
// 	if (!device || !driver)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	if (device->driver != driver)
// 		return_VALUE(-6);
//
// 	if (!driver->ops.remove)
// 		return_VALUE(-23);
//
// 	result = driver->ops.remove(device, ACPI_BUS_REMOVAL_NORMAL);
// 	if (result)
// 		return_VALUE(result);
//
// 	device->driver = NULL;
// 	acpi_driver_data(device) = NULL;
//
// 	down(&acpi_bus_drivers_lock);
// 	driver->references--;
// 	up(&acpi_bus_drivers_lock);
//
// 	return_VALUE(0);
// }
//
//
// /**
//  * acpi_bus_find_driver
//  * --------------------
//  * Parses the list of registered drivers looking for a driver applicable for
//  * the specified device.
//  */
// static int
// acpi_bus_find_driver (
// 	struct acpi_device	*device)
// {
// 	int			result = AE_NOT_FOUND;
// 	struct list_head	*entry = NULL;
// 	struct acpi_driver	*driver = NULL;
//
// 	if (!device || device->driver)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	down(&acpi_bus_drivers_lock);
//
// 	list_for_each(entry, &acpi_bus_drivers) {
//
// 		driver = list_entry(entry, struct acpi_driver, node);
//
// 		if (acpi_bus_match(device, driver))
// 			continue;
//
// 		result = acpi_bus_driver_init(device, driver);
// 		if (!result)
// 			++driver->references;
//
// 		break;
// 	}
//
// 	up(&acpi_bus_drivers_lock);
//
// 	return_VALUE(result);
// }
//
//
// /**
//  * acpi_bus_register_driver
//  * ------------------------
//  * Registers a driver with the ACPI bus.  Searches the namespace for all
//  * devices that match the driver's criteria and binds.
//  */
// int
// acpi_bus_register_driver (
// 	struct acpi_driver	*driver)
// {
// 	if (!driver)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	//if (acpi_disabled)
// 	//	return_VALUE(AE_NOT_FOUND);
//
// 	down(&acpi_bus_drivers_lock);
// 	list_add_tail(&driver->node, &acpi_bus_drivers);
// 	up(&acpi_bus_drivers_lock);
//
// 	acpi_bus_walk(acpi_root, acpi_bus_attach,
// 		WALK_DOWN, driver);
//
// 	return_VALUE(driver->references);
// }
//
//
// /**
//  * acpi_bus_unregister_driver
//  * --------------------------
//  * Unregisters a driver with the ACPI bus.  Searches the namespace for all
//  * devices that match the driver's criteria and unbinds.
//  */
// void
// acpi_bus_unregister_driver (
// 	struct acpi_driver	*driver)
// {
// 	if (!driver)
// 		return;
//
// 	acpi_bus_walk(acpi_root, acpi_bus_unattach, WALK_UP, driver);
//
// 	if (driver->references)
// 		return;
//
// 	down(&acpi_bus_drivers_lock);
// 	list_del(&driver->node);
// 	up(&acpi_bus_drivers_lock);
//
// 	return;
// }
//
//
// /* --------------------------------------------------------------------------
//                                  Device Enumeration
//    -------------------------------------------------------------------------- */
//
// static int
// acpi_bus_get_flags (
// 	struct acpi_device	*device)
// {
// 	ACPI_STATUS		status = AE_OK;
// 	ACPI_HANDLE		temp = NULL;
//
// 	/* Presence of _STA indicates 'dynamic_status' */
// 	status = AcpiGetHandle(device->handle, "_STA", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.dynamic_status = 1;
//
// 	/* Presence of _CID indicates 'compatible_ids' */
// 	status = AcpiGetHandle(device->handle, "_CID", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.compatible_ids = 1;
//
// 	/* Presence of _RMV indicates 'removable' */
// 	status = AcpiGetHandle(device->handle, "_RMV", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.removable = 1;
//
// 	/* Presence of _EJD|_EJ0 indicates 'ejectable' */
// 	status = AcpiGetHandle(device->handle, "_EJD", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.ejectable = 1;
// 	else {
// 		status = AcpiGetHandle(device->handle, "_EJ0", &temp);
// 		if (ACPI_SUCCESS(status))
// 			device->flags.ejectable = 1;
// 	}
//
// 	/* Presence of _LCK indicates 'lockable' */
// 	status = AcpiGetHandle(device->handle, "_LCK", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.lockable = 1;
//
// 	/* Presence of _PS0|_PR0 indicates 'power manageable' */
// 	status = AcpiGetHandle(device->handle, "_PS0", &temp);
// 	if (ACPI_FAILURE(status))
// 		status = AcpiGetHandle(device->handle, "_PR0", &temp);
// 	if (ACPI_SUCCESS(status))
// 		device->flags.power_manageable = 1;
//
// 	/* TBD: Performance management */
//
// 	return_VALUE(0);
// }
//
//
// int
// acpi_bus_add (
// 	struct acpi_device	**child,
// 	struct acpi_device	*parent,
// 	ACPI_HANDLE		handle,
// 	int			type)
// {
// 	int			result = 0;
// 	ACPI_STATUS		status = AE_OK;
// 	struct acpi_device	*device = NULL;
// 	char			bus_id[5] = {'?',0};
// 	ACPI_BUFFER	buffer;
// 	ACPI_DEVICE_INFO	*info = NULL;
// 	char			*hid = NULL;
// 	char			*uid = NULL;
// 	ACPI_PNP_DEVICE_ID_LIST *cid_list = NULL;
// 	int			i = 0;
// 	acpi_unique_id		static_uid_buffer;
//
// 	if (!child)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	device = ExAllocatePoolWithTag(NonPagedPool,sizeof(struct acpi_device), 'DpcA');
// 	if (!device) {
// 		DPRINT1("Memory allocation error\n");
// 		return_VALUE(-12);
// 	}
// 	memset(device, 0, sizeof(struct acpi_device));
//
// 	device->handle = handle;
// 	device->parent = parent;
//
// 	/*
// 	 * Bus ID
// 	 * ------
// 	 * The device's Bus ID is simply the object name.
// 	 * TBD: Shouldn't this value be unique (within the ACPI namespace)?
// 	 */
// 	switch (type) {
// 	case ACPI_BUS_TYPE_SYSTEM:
// 		snprintf(device->pnp.bus_id, sizeof(device->pnp.bus_id), "%s", "ACPI");
// 		break;
// 	case ACPI_BUS_TYPE_POWER_BUTTONF:
// 	case ACPI_BUS_TYPE_POWER_BUTTON:
// 		snprintf(device->pnp.bus_id, sizeof(device->pnp.bus_id), "%s", "PWRF");
// 		break;
// 	case ACPI_BUS_TYPE_SLEEP_BUTTONF:
// 	case ACPI_BUS_TYPE_SLEEP_BUTTON:
// 		snprintf(device->pnp.bus_id, sizeof(device->pnp.bus_id), "%s", "SLPF");
// 		break;
// 	default:
// 		buffer.Length = sizeof(bus_id);
// 		buffer.Pointer = bus_id;
// 		AcpiGetName(handle, ACPI_SINGLE_NAME, &buffer);
//
//
// 		/* Clean up trailing underscores (if any) */
// 		for (i = 3; i > 1; i--) {
// 			if (bus_id[i] == '_')
// 				bus_id[i] = '\0';
// 			else
// 				break;
// 		}
// 		snprintf(device->pnp.bus_id, sizeof(device->pnp.bus_id), "%s", bus_id);
// 		buffer.Pointer = NULL;
//
// 		break;
// 	}
//
// 	/*
// 	 * Flags
// 	 * -----
// 	 * Get prior to calling acpi_bus_get_status() so we know whether
// 	 * or not _STA is present.  Note that we only look for object
// 	 * handles -- cannot evaluate objects until we know the device is
// 	 * present and properly initialized.
// 	 */
// 	result = acpi_bus_get_flags(device);
// 	if (result)
// 		goto end;
//
// 	/*
// 	 * Status
// 	 * ------
// 	 * See if the device is present.  We always assume that non-Device()
// 	 * objects (e.g. thermal zones, power resources, processors, etc.) are
// 	 * present, functioning, etc. (at least when parent object is present).
// 	 * Note that _STA has a different meaning for some objects (e.g.
// 	 * power resources) so we need to be careful how we use it.
// 	 */
// 	switch (type) {
// 	case ACPI_BUS_TYPE_DEVICE:
// 		result = acpi_bus_get_status(device);
// 		if (result)
// 			goto end;
// 		break;
// 	default:
// 		STRUCT_TO_INT(device->status) = 0x0F;
// 		break;
// 	}
// 	if (!device->status.present) {
// 		result = -2;
// 		goto end;
// 	}
//
// 	/*
// 	 * Initialize Device
// 	 * -----------------
// 	 * TBD: Synch with Core's enumeration/initialization process.
// 	 */
//
// 	/*
// 	 * Hardware ID, Unique ID, & Bus Address
// 	 * -------------------------------------
// 	 */
// 	switch (type) {
// 	case ACPI_BUS_TYPE_DEVICE:
// 		status = AcpiGetObjectInfo(handle,&info);
// 		if (ACPI_FAILURE(status)) {
// 			ACPI_DEBUG_PRINT((ACPI_DB_ERROR,
// 				"Error reading device info\n"));
// 			result = AE_NOT_FOUND;
// 			info = NULL;
// 			goto end;
// 		}
// 		if (info->Valid & ACPI_VALID_HID)
// 			hid = info->HardwareId.String;
// 		if (info->Valid & ACPI_VALID_UID)
// 			uid = info->UniqueId.String;
// 		if (info->Valid & ACPI_VALID_CID) {
// 			cid_list = &info->CompatibleIdList;
// 			device->pnp.cid_list = ExAllocatePoolWithTag(NonPagedPool,cid_list->ListSize, 'DpcA');
// 			if (device->pnp.cid_list) {
// 				char *p = (char *)&device->pnp.cid_list->Ids[cid_list->Count];
// 				device->pnp.cid_list->Count = cid_list->Count;
// 				device->pnp.cid_list->ListSize = cid_list->ListSize;
// 				for (i = 0; i < cid_list->Count; i++) {
// 					device->pnp.cid_list->Ids[i].Length = cid_list->Ids[i].Length;
// 					device->pnp.cid_list->Ids[i].String = p;
// 					ASSERT(p + cid_list->Ids[i].Length <= (char *)device->pnp.cid_list + cid_list->ListSize);
// 					memcpy(device->pnp.cid_list->Ids[i].String,
// 						cid_list->Ids[i].String, cid_list->Ids[i].Length);
// 					p += cid_list->Ids[i].Length;
// 				}
// 			}
// 			else
// 				DPRINT("Memory allocation error\n");
// 		}
// 		if (info->Valid & ACPI_VALID_ADR) {
// 			device->pnp.bus_address = info->Address;
// 			device->flags.bus_address = 1;
// 		}
// 		break;
// 	case ACPI_BUS_TYPE_POWER:
// 		hid = ACPI_POWER_HID;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (PowerDeviceCount++));
// 		break;
// 	case ACPI_BUS_TYPE_PROCESSOR:
// 		hid = ACPI_PROCESSOR_HID;
// 		uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "_%d", (ProcessorCount++));
// 		break;
// 	case ACPI_BUS_TYPE_SYSTEM:
// 		hid = ACPI_SYSTEM_HID;
// 		break;
// 	case ACPI_BUS_TYPE_THERMAL:
// 		hid = ACPI_THERMAL_HID;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (ThermalZoneCount++));
// 		break;
// 	case ACPI_BUS_TYPE_POWER_BUTTON:
// 		hid = ACPI_BUTTON_HID_POWER;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (PowerButtonCount++));
// 		break;
// 	case ACPI_BUS_TYPE_POWER_BUTTONF:
// 		hid = ACPI_BUTTON_HID_POWERF;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (FixedPowerButtonCount++));
// 		break;
// 	case ACPI_BUS_TYPE_SLEEP_BUTTON:
// 		hid = ACPI_BUTTON_HID_SLEEP;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (SleepButtonCount++));
// 		break;
// 	case ACPI_BUS_TYPE_SLEEP_BUTTONF:
// 		hid = ACPI_BUTTON_HID_SLEEPF;
//         uid = static_uid_buffer;
// 		snprintf(uid, sizeof(static_uid_buffer), "%d", (FixedSleepButtonCount++));
// 		break;
// 	}
//
// 	/*
// 	 * \_SB
// 	 * ----
// 	 * Fix for the system root bus device -- the only root-level device.
// 	 */
// 	if (((ACPI_HANDLE)parent == ACPI_ROOT_OBJECT) && (type == ACPI_BUS_TYPE_DEVICE)) {
// 		hid = ACPI_BUS_HID;
// 		snprintf(device->pnp.device_name, sizeof(device->pnp.device_name), "%s", ACPI_BUS_DEVICE_NAME);
// 		snprintf(device->pnp.device_class, sizeof(device->pnp.device_class), "%s", ACPI_BUS_CLASS);
// 	}
//
// 	if (hid) {
//         device->pnp.hardware_id = ExAllocatePoolWithTag(NonPagedPool, strlen(hid) + 1, 'DpcA');
//         if (device->pnp.hardware_id) {
//             snprintf(device->pnp.hardware_id, strlen(hid) + 1, "%s", hid);
//             device->flags.hardware_id = 1;
//         }
// 	}
// 	if (uid) {
// 		snprintf(device->pnp.unique_id, sizeof(device->pnp.unique_id), "%s", uid);
// 		device->flags.unique_id = 1;
// 	}
//
// 	/*
// 	 * If we called get_object_info, we now are finished with the buffer,
// 	 * so we can free it.
// 	 */
// 	//if (buffer.Pointer)
// 		//AcpiOsFree(buffer.Pointer);
//
// 	/*
// 	 * Power Management
// 	 * ----------------
// 	 */
// 	if (device->flags.power_manageable) {
// 		result = acpi_bus_get_power_flags(device);
// 		if (result)
// 			goto end;
// 	}
//
// 	/*
// 	 * Performance Management
// 	 * ----------------------
// 	 */
// 	if (device->flags.performance_manageable) {
// 		result = acpi_bus_get_perf_flags(device);
// 		if (result)
// 			goto end;
// 	}
//
// 	/*
// 	 * Context
// 	 * -------
// 	 * Attach this 'struct acpi_device' to the ACPI object.  This makes
// 	 * resolutions from handle->device very efficient.  Note that we need
// 	 * to be careful with fixed-feature devices as they all attach to the
// 	 * root object.
// 	 */
// 	switch (type) {
// 	case ACPI_BUS_TYPE_POWER_BUTTON:
// 	case ACPI_BUS_TYPE_POWER_BUTTONF:
// 	case ACPI_BUS_TYPE_SLEEP_BUTTON:
// 	case ACPI_BUS_TYPE_SLEEP_BUTTONF:
// 		break;
// 	default:
// 		status = AcpiAttachData(device->handle,
// 			acpi_bus_data_handler, device);
// 		break;
// 	}
// 	if (ACPI_FAILURE(status)) {
// 		ACPI_DEBUG_PRINT((ACPI_DB_ERROR,
// 			"Error attaching device data\n"));
// 		result = AE_NOT_FOUND;
// 		goto end;
// 	}
//
// 	/*
// 	 * Linkage
// 	 * -------
// 	 * Link this device to its parent and siblings.
// 	 */
// 	INIT_LIST_HEAD(&device->children);
// 	if (!device->parent)
// 		INIT_LIST_HEAD(&device->node);
// 	else
// 		list_add_tail(&device->node, &device->parent->children);
//
// 	/*
// 	 * Global Device Hierarchy:
// 	 * ------------------------
// 	 * Register this device with the global device hierarchy.
// 	 */
// 	acpi_device_register(device, parent);
//
// 	/*
// 	 * Bind _ADR-Based Devices
// 	 * -----------------------
// 	 * If there's a a bus address (_ADR) then we utilize the parent's
// 	 * 'bind' function (if exists) to bind the ACPI- and natively-
// 	 * enumerated device representations.
// 	 */
// 	if (device->flags.bus_address) {
// 		if (device->parent && device->parent->ops.bind)
// 			device->parent->ops.bind(device);
// 	}
//
// 	/*
// 	 * Locate & Attach Driver
// 	 * ----------------------
// 	 * If there's a hardware id (_HID) or compatible ids (_CID) we check
// 	 * to see if there's a driver installed for this kind of device.  Note
// 	 * that drivers can install before or after a device is enumerated.
// 	 *
// 	 * TBD: Assumes LDM provides driver hot-plug capability.
// 	 */
// 	if (device->flags.hardware_id || device->flags.compatible_ids)
// 		acpi_bus_find_driver(device);
//
// end:
// 	if (info != NULL)
// 		ACPI_FREE(info);
// 	if (result) {
// 		if (device->pnp.cid_list) {
// 			ExFreePoolWithTag(device->pnp.cid_list, 'DpcA');
// 		}
//         if (device->pnp.hardware_id) {
//             ExFreePoolWithTag(device->pnp.hardware_id, 'DpcA');
//         }
// 		ExFreePoolWithTag(device, 'DpcA');
// 		return_VALUE(result);
// 	}
// 	*child = device;
//
// 	return_VALUE(0);
// }
//
//
// static int
// acpi_bus_remove (
// 	struct acpi_device	*device,
// 	int			type)
// {
//
// 	if (!device)
// 		return_VALUE(AE_NOT_FOUND);
//
// 	acpi_device_unregister(device);
//
// 	if (device->pnp.cid_list)
// 		ExFreePoolWithTag(device->pnp.cid_list, 'DpcA');
//
// 	if (device->pnp.hardware_id)
// 		ExFreePoolWithTag(device->pnp.hardware_id, 'DpcA');
//
// 	if (device)
// 		ExFreePoolWithTag(device, 'DpcA');
//
// 	return_VALUE(0);
// }
//
//
// int
// acpi_bus_scan (
// 	struct acpi_device	*start)
// {
// 	ACPI_STATUS		status = AE_OK;
// 	struct acpi_device	*parent = NULL;
// 	struct acpi_device	*child = NULL;
// 	ACPI_HANDLE		phandle = 0;
// 	ACPI_HANDLE		chandle = 0;
// 	ACPI_OBJECT_TYPE	type = 0;
// 	UINT32			level = 1;
//
// 	if (!start)
// 		return_VALUE(AE_BAD_PARAMETER);
//
// 	parent = start;
// 	phandle = start->handle;
//
// 	/*
// 	 * Parse through the ACPI namespace, identify all 'devices', and
// 	 * create a new 'struct acpi_device' for each.
// 	 */
// 	while ((level > 0) && parent) {
//
// 		status = AcpiGetNextObject(ACPI_TYPE_ANY, phandle,
// 			chandle, &chandle);
//
// 		/*
// 		 * If this scope is exhausted then move our way back up.
// 		 */
// 		if (ACPI_FAILURE(status)) {
// 			level--;
// 			chandle = phandle;
// 			AcpiGetParent(phandle, &phandle);
// 			if (parent->parent)
// 				parent = parent->parent;
// 			continue;
// 		}
//
// 		status = AcpiGetType(chandle, &type);
// 		if (ACPI_FAILURE(status))
// 			continue;
//
// 		/*
// 		 * If this is a scope object then parse it (depth-first).
// 		 */
// 		if (type == ACPI_TYPE_LOCAL_SCOPE) {
// 			level++;
// 			phandle = chandle;
// 			chandle = 0;
// 			continue;
// 		}
//
// 		/*
// 		 * We're only interested in objects that we consider 'devices'.
// 		 */
// 		switch (type) {
// 		case ACPI_TYPE_DEVICE:
// 			type = ACPI_BUS_TYPE_DEVICE;
// 			break;
// 		case ACPI_TYPE_PROCESSOR:
// 			type = ACPI_BUS_TYPE_PROCESSOR;
// 			break;
// 		case ACPI_TYPE_THERMAL:
// 			type = ACPI_BUS_TYPE_THERMAL;
// 			break;
// 		case ACPI_TYPE_POWER:
// 			type = ACPI_BUS_TYPE_POWER;
// 			break;
// 		default:
// 			continue;
// 		}
//
// 		status = acpi_bus_add(&child, parent, chandle, type);
// 		if (ACPI_FAILURE(status))
// 			continue;
//
// 		/*
// 		 * If the device is present, enabled, and functioning then
// 		 * parse its scope (depth-first).  Note that we need to
// 		 * represent absent devices to facilitate PnP notifications
// 		 * -- but only the subtree head (not all of its children,
// 		 * which will be enumerated when the parent is inserted).
// 		 *
// 		 * TBD: Need notifications and other detection mechanisms
// 		 *	in place before we can fully implement this.
// 		 */
// 		if (child->status.present) {
// 			status = AcpiGetNextObject(ACPI_TYPE_ANY, chandle,
// 				0, NULL);
// 			if (ACPI_SUCCESS(status)) {
// 				level++;
// 				phandle = chandle;
// 				chandle = 0;
// 				parent = child;
// 			}
// 		}
// 	}
//
// 	return_VALUE(0);
// }
//
//
// static int
// acpi_bus_scan_fixed (
// 	struct acpi_device	*root)
// {
// 	int			result = 0;
// 	struct acpi_device	*device = NULL;
//
// 	if (!root)
// 		return_VALUE(AE_NOT_FOUND);
//
// 	/* If ACPI_FADT_POWER_BUTTON is set, then a control
// 	 * method power button is present. Otherwise, a fixed
// 	 * power button is present.
// 	 */
// 	if (AcpiGbl_FADT.Flags & ACPI_FADT_POWER_BUTTON)
// 		result = acpi_bus_add(&device, acpi_root,
// 			NULL, ACPI_BUS_TYPE_POWER_BUTTON);
// 	else
// 	{
// 		/* Enable the fixed power button so we get notified if it is pressed */
// 		AcpiWriteBitRegister(ACPI_BITREG_POWER_BUTTON_ENABLE, 1);
//
// 		result = acpi_bus_add(&device, acpi_root,
// 			NULL, ACPI_BUS_TYPE_POWER_BUTTONF);
// 	}
//
// 	/* This one is a bit more complicated and we do it wrong
// 	 * right now. If ACPI_FADT_SLEEP_BUTTON is set but no
// 	 * device object is present then no sleep button is present, but
// 	 * if the flags is clear and there is no device object then it is
// 	 * a fixed sleep button. If the flag is set and there is a device object
// 	 * the we have a control method button just like above.
// 	 */
// 	if (AcpiGbl_FADT.Flags & ACPI_FADT_SLEEP_BUTTON)
// 		result = acpi_bus_add(&device, acpi_root,
// 			NULL, ACPI_BUS_TYPE_SLEEP_BUTTON);
// 	else
// 	{
// 		/* Enable the fixed sleep button so we get notified if it is pressed */
// 		AcpiWriteBitRegister(ACPI_BITREG_SLEEP_BUTTON_ENABLE, 1);
//
// 		result = acpi_bus_add(&device, acpi_root,
// 			NULL, ACPI_BUS_TYPE_SLEEP_BUTTONF);
// 	}
//
// 	return_VALUE(result);
// }
//
//
// /* --------------------------------------------------------------------------
//                              Initialization/Cleanup
//    -------------------------------------------------------------------------- */
//
// int
// acpi_bus_init (void)
// {
// 	int			result = 0;
// 	ACPI_STATUS		status = AE_OK;
//
// 	DPRINT("acpi_bus_init\n");
//
//         KeInitializeDpc(&event_dpc, acpi_bus_generate_event_dpc, NULL);
//
// 	status = AcpiEnableSubsystem(ACPI_FULL_INITIALIZATION);
// 	if (ACPI_FAILURE(status)) {
// 		DPRINT1("Unable to start the ACPI Interpreter\n");
// 		goto error1;
// 	}
//
// 	/*
// 	 * ACPI 2.0 requires the EC driver to be loaded and work before
// 	 * the EC device is found in the namespace. This is accomplished
// 	 * by looking for the ECDT table, and getting the EC parameters out
// 	 * of that.
// 	 */
// 	//result = acpi_ec_ecdt_probe();
// 	/* Ignore result. Not having an ECDT is not fatal. */
//
// 	status = AcpiInitializeObjects(ACPI_FULL_INITIALIZATION);
// 	if (ACPI_FAILURE(status)) {
// 		DPRINT1("Unable to initialize ACPI objects\n");
// 		goto error1;
// 	}
//
// 	/*
// 	 * Maybe EC region is required at bus_scan/acpi_get_devices. So it
// 	 * is necessary to enable it as early as possible.
// 	 */
// 	//acpi_boot_ec_enable();
//
// 	/* Initialize sleep structures */
// 	//acpi_sleep_init();
//
// 	/*
// 	 * Register the for all standard device notifications.
// 	 */
// 	status = AcpiInstallNotifyHandler(ACPI_ROOT_OBJECT, ACPI_SYSTEM_NOTIFY, acpi_bus_notify, NULL);
// 	if (ACPI_FAILURE(status)) {
// 		DPRINT1("Unable to register for device notifications\n");
// 		result = AE_NOT_FOUND;
// 		goto error1;
// 	}
//
// 	/*
// 	 * Create the root device in the bus's device tree
// 	 */
// 	result = acpi_bus_add(&acpi_root, NULL, ACPI_ROOT_OBJECT,
// 		ACPI_BUS_TYPE_SYSTEM);
// 	if (result)
// 		goto error2;
//
//
// 	/*
// 	 * Enumerate devices in the ACPI namespace.
// 	 */
// 	result = acpi_bus_scan_fixed(acpi_root);
// 	if (result)
// 		DPRINT1("acpi_bus_scan_fixed failed\n");
// 	result = acpi_bus_scan(acpi_root);
// 	if (result)
// 		DPRINT1("acpi_bus_scan failed\n");
//
// 	return_VALUE(0);
//
// 	/* Mimic structured exception handling */
// error2:
// 	AcpiRemoveNotifyHandler(ACPI_ROOT_OBJECT,
// 		ACPI_SYSTEM_NOTIFY, acpi_bus_notify);
// error1:
// 	AcpiTerminate();
// 	return_VALUE(AE_NOT_FOUND);
// }
//
// static void
// acpi_bus_exit (void)
// {
// 	ACPI_STATUS		status = AE_OK;
//
// 	DPRINT1("acpi_bus_exit\n");
//
// 	status = AcpiRemoveNotifyHandler(ACPI_ROOT_OBJECT,
// 		ACPI_SYSTEM_NOTIFY, acpi_bus_notify);
// 	if (ACPI_FAILURE(status))
// 		DPRINT1("Error removing notify handler\n");
//
// #ifdef CONFIG_ACPI_PCI
// 	acpi_pci_root_exit();
// 	acpi_pci_link_exit();
// #endif
// #ifdef CONFIG_ACPI_EC
// 	acpi_ec_exit();
// #endif
// 	//acpi_power_exit();
// 	acpi_system_exit();
//
// 	acpi_bus_remove(acpi_root, ACPI_BUS_REMOVAL_NORMAL);
//
// 	status = AcpiTerminate();
// 	if (ACPI_FAILURE(status))
// 		DPRINT1("Unable to terminate the ACPI Interpreter\n");
// 	else
// 		DPRINT1("Interpreter disabled\n");
//
// 	return_VOID;
// }
//
//
// int
// acpi_init (void)
// {
// 	int			result = 0;
//
// 	DPRINT("acpi_init\n");
//
// 	DPRINT("Subsystem revision %08x\n",ACPI_CA_VERSION);
//
// 	KeInitializeSpinLock(&acpi_bus_event_lock);
// 	KeInitializeEvent(&AcpiEventQueue, NotificationEvent, FALSE);
// 	ExInitializeFastMutex(&acpi_bus_drivers_lock);
//
// 	result = acpi_bus_init();
//
// 	//if (!result) {
// 		//pci_mmcfg_late_init();
// 		//if (!(pm_flags & PM_APM))
// 		//	pm_flags |= PM_ACPI;
// 		//else {
// 			//DPRINT1("APM is already active, exiting\n");
// 			//disable_acpi();
// 			//result = -ENODEV;
// 		//}
// 	//} else
// 	//	disable_acpi();
//
// 	/*
// 	 * If the laptop falls into the DMI check table, the power state check
// 	 * will be disabled in the course of device power transistion.
// 	 */
// 	//dmi_check_system(power_nocheck_dmi_table);
//
// 	/*
// 	 * Install drivers required for proper enumeration of the
// 	 * ACPI namespace.
// 	 */
// 	acpi_system_init();	/* ACPI System */
// 	acpi_power_init();	/* ACPI Bus Power Management */
// 	acpi_button_init();
// 	//acpi_ec_init();		/* ACPI Embedded Controller */
// #ifdef CONFIG_ACPI_PCI
// 	if (!acpi_pci_disabled) {
// 		acpi_pci_link_init();	/* ACPI PCI Interrupt Link */
// 		acpi_pci_root_init();	/* ACPI PCI Root Bridge */
// 	}
// #endif
//
// 	//acpi_scan_init();
// 	//acpi_ec_init();
// 	//acpi_power_init();
// 	//acpi_system_init();
// 	//acpi_debug_init();
// 	//acpi_sleep_proc_init();
// 	//acpi_wakeup_device_init();
//
// 	return result;
// }
//
//
// void
// acpi_exit (void)
// {
// 	DPRINT("acpi_exit\n");
//
// #ifdef CONFIG_PM
// 	pm_active = 0;
// #endif
//
// 	acpi_bus_exit();
//
// 	return_VOID;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
