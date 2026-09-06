//! busmgr/power.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! This is busmgr/power.c — ACPI **power-RESOURCE** management (Power Resource
//! Control), NOT the top-level power.c. Module file is named busmgr_power.rs but
//! every #[no_mangle] symbol keeps its original C name.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*;
use core::ffi::{c_char, c_int, c_void};

// Driver-private + ACPICA types (acpi_device, acpi_driver, acpi_handle_list,
// acpi_object/ACPI_OBJECT, ACPI_BUFFER, acpi_object_list, ACPI_HANDLE,
// ACPI_STATUS, ACPI_STRING, AE_OK, AE_NOT_FOUND, list_head, acpi_bus_id, and the
// sibling extern "C" fns acpi_bus_get_device / acpi_bus_register_driver /
// acpi_evaluate_integer) all come in via crate::shared::* (re-export of crate::types).
// ExAllocatePoolWithTag from ntkrnl_sys::ex (POOL_TYPE/SIZE_T = mm/base flavor,
// matching the bus.rs allocation pattern). ExFreePool is bindgen-only (ddk).
use ntkrnl_sys::ex::ExAllocatePoolWithTag;
use ntkrnl_sys::ddk::ExFreePool;
// NonPagedPool (POOL_TYPE) lives in ntkrnl_sys::mm, not in crate::shared.
use ntkrnl_sys::mm::NonPagedPool;

// ACPICA outbound calls (libacpica.a).
use acpica_sys::{AcpiEvaluateObject, AcpiGetName};
// ACPICA constants bindgen emits.
use acpica_sys::{ACPI_SINGLE_NAME, ACPI_TYPE_INTEGER};

// ACPI_DEBUG_PRINT / ACPI_FUNCTION_TRACE / ACPI_MODULE_NAME are no-ops (NDEBUG).
macro_rules! ACPI_DEBUG_PRINT { ($($t:tt)*) => {}; }

// strcpy from the CRT (used for resource.name and acpi_device_name/class).
extern "C" {
    fn strcpy(dst: *mut c_char, src: *const c_char) -> *mut c_char;
}

// ---- ACPICA result macros (acexcep.h): AE_OK == 0. ----
#[inline]
fn ACPI_SUCCESS(s: ACPI_STATUS) -> bool {
    s == AE_OK
}
#[inline]
fn ACPI_FAILURE(s: ACPI_STATUS) -> bool {
    s != AE_OK
}

// ---- ACPI D-state #defines (actypes.h), same values bus.rs uses. ----
const ACPI_STATE_D0: c_int = 0; // (UINT8) 0     actypes.h:633
const ACPI_STATE_D3: c_int = 3; // (UINT8) 3     actypes.h:636
const ACPI_STATE_UNKNOWN: c_int = 0xFF; // (UINT8) 0xFF  actypes.h:622

// ---- Power-resource state #defines (defined in busmgr/power.c itself, kept). ----
const ACPI_POWER_RESOURCE_STATE_OFF: c_int = 0x00;
const ACPI_POWER_RESOURCE_STATE_ON: c_int = 0x01;
const ACPI_POWER_RESOURCE_STATE_UNKNOWN: c_int = 0xFF;

// ---- Pool tag 'IPCA'  (C multichar = ('I'<<24)|('P'<<16)|('C'<<8)|'A'). ----
const ACPI_POWER_TAG: u32 = u32::from_be_bytes([b'I', b'P', b'C', b'A']);

// ---- Driver-descriptor string #defines (drivers/bus/acpi/include/acpi_drivers.h). ----
//   ACPI_POWER_DRIVER_NAME = "ACPI Power Resource Driver"
//   ACPI_POWER_CLASS       = "power_resource"
//   ACPI_POWER_HID         = "ACPI_PWR"
//   ACPI_POWER_DEVICE_NAME = "Power Resource"
const ACPI_POWER_CLASS: &[u8] = b"power_resource\0";
const ACPI_POWER_DEVICE_NAME: &[u8] = b"Power Resource\0";

// ---- Linux list.h helpers (drivers/bus/acpi/include/list.h), faithful;
//      copied from bus.rs so this leaf is self-contained. ----
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

// container_of(ptr, acpi_power_reference, node)
macro_rules! container_of_ref {
    ($ptr:expr) => {
        ($ptr as *mut u8)
            .wrapping_sub(core::mem::offset_of!(acpi_power_reference, node))
            as *mut acpi_power_reference
    };
}

// ---- File-private structs (busmgr/power.c). ----
#[repr(C)]
struct acpi_power_reference {
    node: list_head,
    device: *mut acpi_device,
}

#[repr(C)]
struct acpi_power_resource {
    device: *mut acpi_device,
    name: acpi_bus_id, // [c_char; 8]
    system_level: u32,
    order: u32,
    reference: list_head,
}

// ---- Global with external linkage: `int acpi_power_nocheck;`. ----
#[no_mangle]
pub static mut acpi_power_nocheck: c_int = 0;

// ---- File-private list head (zero-init; INIT_LIST_HEAD'd in acpi_power_init). ----
static mut acpi_power_resource_list: list_head = list_head {
    next: core::ptr::null_mut(),
    prev: core::ptr::null_mut(),
};

// ---- Driver descriptor. ops fields order = {add, remove, start, suspend, resume,
//      bind, unbind, notify, scan}; C init {add, remove, NULL, NULL, resume}. ----
// Built lazily-by-pointer at registration time (name/class are arrays, ids is a
// *mut c_char; we cast the const string literals — acpi_bus_register_driver only
// reads them). Use a static mut initialized once in acpi_power_init.
static mut acpi_power_driver: acpi_driver = acpi_driver {
    node: list_head { next: core::ptr::null_mut(), prev: core::ptr::null_mut() },
    name: [0; 80],
    class: [0; 80],
    references: 0,
    flags: 0,
    ids: core::ptr::null_mut(),
    ops: acpi_device_ops {
        add: Some(acpi_power_add),
        remove: Some(acpi_power_remove),
        start: None,
        suspend: None,
        resume: Some(acpi_power_resume),
        bind: None,
        unbind: None,
        notify: None,
        scan: None,
    },
};

// acpi_driver_data(device) == (*device).driver_data
#[inline]
unsafe fn acpi_driver_data(device: *mut acpi_device) -> *mut acpi_power_resource {
    (*device).driver_data as *mut acpi_power_resource
}
// acpi_device_name(device) == (*device).pnp.device_name (array → ptr)
#[inline]
unsafe fn acpi_device_name(device: *mut acpi_device) -> *mut c_char {
    (*device).pnp.device_name.as_mut_ptr()
}
// acpi_device_class(device) == (*device).pnp.device_class (array → ptr)
#[inline]
unsafe fn acpi_device_class(device: *mut acpi_device) -> *mut c_char {
    (*device).pnp.device_class.as_mut_ptr()
}

/* --------------------------------------------------------------------------
                         Power Resource Management
   -------------------------------------------------------------------------- */

unsafe fn acpi_power_get_context(
    handle: ACPI_HANDLE,
    resource: *mut *mut acpi_power_resource,
) -> c_int {
    let mut result: c_int = 0;
    let mut device: *mut acpi_device = core::ptr::null_mut();

    if resource.is_null() {
        return -15;
    }

    result = acpi_bus_get_device(handle, &mut device);
    if result != 0 {
        ACPI_DEBUG_PRINT!();
        return result;
    }

    *resource = acpi_driver_data(device);
    if (*resource).is_null() {
        return -15;
    }

    0
}

unsafe fn acpi_power_get_state(handle: ACPI_HANDLE, state: *mut c_int) -> c_int {
    let mut status: ACPI_STATUS = AE_OK;
    let mut sta: u64 = 0;
    let mut node_name: [c_char; 5] = [0; 5];
    let mut buffer = ACPI_BUFFER {
        Length: core::mem::size_of::<[c_char; 5]>() as ACPI_SIZE,
        Pointer: node_name.as_mut_ptr() as *mut c_void,
    };

    if handle.is_null() || state.is_null() {
        return -1;
    }

    // acpi_evaluate_integer(handle, "_STA", NULL, &sta) — sibling extern "C".
    status = acpi_evaluate_integer(
        handle,
        c"_STA".as_ptr() as ACPI_STRING,
        core::ptr::null_mut(),
        &mut sta,
    );
    if ACPI_FAILURE(status) {
        return -15;
    }

    *state = if (sta & 0x01) != 0 {
        ACPI_POWER_RESOURCE_STATE_ON
    } else {
        ACPI_POWER_RESOURCE_STATE_OFF
    };

    AcpiGetName(handle, ACPI_SINGLE_NAME, &mut buffer);

    ACPI_DEBUG_PRINT!();

    0
}

unsafe fn acpi_power_get_list_state(list: *mut acpi_handle_list, state: *mut c_int) -> c_int {
    let mut result: c_int = 0;
    let mut state1: c_int = 0;
    let mut i: u32 = 0;

    if list.is_null() || state.is_null() {
        return -1;
    }

    /* The state of the list is 'on' IFF all resources are 'on'. */
    i = 0;
    while i < (*list).count {
        result = acpi_power_get_state((*list).handles[i as usize], &mut state1);
        if result != 0 {
            return result;
        }

        *state = state1;

        if *state != ACPI_POWER_RESOURCE_STATE_ON {
            break;
        }
        i += 1;
    }

    ACPI_DEBUG_PRINT!();

    result
}

unsafe fn acpi_power_on(handle: ACPI_HANDLE, dev: *mut acpi_device) -> c_int {
    let mut result: c_int = 0;
    let mut found: c_int = 0;
    let mut status: ACPI_STATUS = AE_OK;
    let mut resource: *mut acpi_power_resource = core::ptr::null_mut();

    result = acpi_power_get_context(handle, &mut resource);
    if result != 0 {
        return result;
    }

    // list_for_each_safe(node, next, &resource->reference)
    let head: *mut list_head = &mut (*resource).reference;
    let mut node: *mut list_head = (*head).next;
    let mut next: *mut list_head = (*node).next;
    while node != head {
        let r#ref = container_of_ref!(node);
        if (*dev).handle == (*(*r#ref).device).handle {
            ACPI_DEBUG_PRINT!();
            found = 1;
            break;
        }
        node = next;
        next = (*node).next;
    }

    if found == 0 {
        let r#ref = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<acpi_power_reference>(),
            ACPI_POWER_TAG,
        ) as *mut acpi_power_reference;
        if r#ref.is_null() {
            ACPI_DEBUG_PRINT!();
            return -1; //-ENOMEM;
        }
        list_add_tail(&mut (*r#ref).node, &mut (*resource).reference);
        (*r#ref).device = dev;
        ACPI_DEBUG_PRINT!();
    }

    status = AcpiEvaluateObject(
        (*(*resource).device).handle,
        c"_ON".as_ptr() as ACPI_STRING,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ACPI_FAILURE(status) {
        return -15;
    }

    /* Update the power resource's _device_ power state */
    (*(*resource).device).power.state = ACPI_STATE_D0;

    0
}

unsafe fn acpi_power_off_device(handle: ACPI_HANDLE, dev: *mut acpi_device) -> c_int {
    let mut result: c_int = 0;
    let mut status: ACPI_STATUS = AE_OK;
    let mut resource: *mut acpi_power_resource = core::ptr::null_mut();

    result = acpi_power_get_context(handle, &mut resource);
    if result != 0 {
        return result;
    }

    // list_for_each_safe(node, next, &resource->reference)
    let head: *mut list_head = &mut (*resource).reference;
    let mut node: *mut list_head = (*head).next;
    let mut next: *mut list_head = (*node).next;
    while node != head {
        let r#ref = container_of_ref!(node);
        if (*dev).handle == (*(*r#ref).device).handle {
            list_del(&mut (*r#ref).node);
            ExFreePool(r#ref as *mut c_void);
            ACPI_DEBUG_PRINT!();
            break;
        }
        node = next;
        next = (*node).next;
    }

    if list_empty(&mut (*resource).reference) == 0 {
        ACPI_DEBUG_PRINT!();
        return 0;
    }

    status = AcpiEvaluateObject(
        (*(*resource).device).handle,
        c"_OFF".as_ptr() as ACPI_STRING,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if ACPI_FAILURE(status) {
        return -1;
    }

    /* Update the power resource's _device_ power state */
    (*(*resource).device).power.state = ACPI_STATE_D3;

    ACPI_DEBUG_PRINT!();

    0
}

/// acpi_device_sleep_wake - execute _DSW (Device Sleep Wake) or (deprecated in
/// ACPI 3.0) _PSW (Power State Wake).
#[no_mangle]
pub unsafe extern "C" fn acpi_device_sleep_wake(
    dev: *mut acpi_device,
    enable: c_int,
    sleep_state: c_int,
    dev_state: c_int,
) -> c_int {
    let mut in_arg: [ACPI_OBJECT; 3] = core::mem::zeroed();
    let mut arg_list = acpi_object_list {
        Count: 3,
        Pointer: in_arg.as_mut_ptr(),
    };
    let mut status: ACPI_STATUS = AE_OK;

    /*
     * Try to execute _DSW first.  Three arguments are needed.
     */
    in_arg[0].Integer.Type = ACPI_TYPE_INTEGER;
    in_arg[0].Integer.Value = enable as u64;
    in_arg[1].Integer.Type = ACPI_TYPE_INTEGER;
    in_arg[1].Integer.Value = sleep_state as u64;
    in_arg[2].Integer.Type = ACPI_TYPE_INTEGER;
    in_arg[2].Integer.Value = dev_state as u64;
    status = AcpiEvaluateObject(
        (*dev).handle,
        c"_DSW".as_ptr() as ACPI_STRING,
        &mut arg_list,
        core::ptr::null_mut(),
    );
    if ACPI_SUCCESS(status) {
        return 0;
    } else if status != AE_NOT_FOUND {
        DPRINT1!("_DSW execution failed\n");
        (*dev).wakeup.flags.set_valid(0);
        return -1;
    }

    /* Execute _PSW */
    arg_list.Count = 1;
    in_arg[0].Integer.Value = enable as u64;
    status = AcpiEvaluateObject(
        (*dev).handle,
        c"_PSW".as_ptr() as ACPI_STRING,
        &mut arg_list,
        core::ptr::null_mut(),
    );
    if ACPI_FAILURE(status) && (status != AE_NOT_FOUND) {
        DPRINT1!("_PSW execution failed\n");
        (*dev).wakeup.flags.set_valid(0);
        return -1;
    }

    0
}

/// Prepare a wakeup device.
#[no_mangle]
pub unsafe extern "C" fn acpi_enable_wakeup_device_power(
    dev: *mut acpi_device,
    sleep_state: c_int,
) -> c_int {
    let mut i: u32;
    let mut err: c_int = 0;

    if dev.is_null() || (*dev).wakeup.flags.valid() == 0 {
        return -1;
    }

    // if (dev->wakeup.prepare_count++) goto out;
    let prev_count = (*dev).wakeup.prepare_count;
    (*dev).wakeup.prepare_count = prev_count + 1;
    if prev_count != 0 {
        // out:
        return err;
    }

    /* Open power resource */
    let mut hit_err_out = false;
    i = 0;
    while i < (*dev).wakeup.resources.count {
        let ret = acpi_power_on((*dev).wakeup.resources.handles[i as usize], dev);
        if ret != 0 {
            DPRINT!();
            (*dev).wakeup.flags.set_valid(0);
            err = -1;
            hit_err_out = true;
            break;
        }
        i += 1;
    }

    if !hit_err_out {
        /*
         * Passing 3 as the third argument below means the device may be placed
         * in arbitrary power state afterwards.
         */
        err = acpi_device_sleep_wake(dev, 1, sleep_state, 3);
    }

    // err_out:
    if err != 0 {
        (*dev).wakeup.prepare_count = 0;
    }

    // out:
    err
}

/// Shutdown a wakeup device, counterpart of acpi_enable_wakeup_device_power.
#[no_mangle]
pub unsafe extern "C" fn acpi_disable_wakeup_device_power(dev: *mut acpi_device) -> c_int {
    let mut i: u32;
    let mut err: c_int = 0;

    if dev.is_null() || (*dev).wakeup.flags.valid() == 0 {
        return -1;
    }

    // if (--dev->wakeup.prepare_count > 0) goto out;
    (*dev).wakeup.prepare_count -= 1;
    if (*dev).wakeup.prepare_count > 0 {
        return err;
    }

    /*
     * Executing the code below even if prepare_count is already zero when
     * the function is called may be useful, for example for initialisation.
     */
    if (*dev).wakeup.prepare_count < 0 {
        (*dev).wakeup.prepare_count = 0;
    }

    err = acpi_device_sleep_wake(dev, 0, 0, 0);
    if err != 0 {
        return err; // goto out;
    }

    /* Close power resource */
    i = 0;
    while i < (*dev).wakeup.resources.count {
        let ret =
            acpi_power_off_device((*dev).wakeup.resources.handles[i as usize], dev);
        if ret != 0 {
            DPRINT!();
            (*dev).wakeup.flags.set_valid(0);
            err = -1;
            return err; // goto out;
        }
        i += 1;
    }

    // out:
    err
}

/* --------------------------------------------------------------------------
                         Device Power Management
   -------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn acpi_power_get_inferred_state(device: *mut acpi_device) -> c_int {
    let mut result: c_int = 0;
    let mut list: *mut acpi_handle_list = core::ptr::null_mut();
    let mut list_state: c_int = 0;
    let mut i: c_int = 0;

    if device.is_null() {
        return -1;
    }

    (*device).power.state = ACPI_STATE_UNKNOWN;

    /*
     * We know a device's inferred power state when all the resources
     * required for a given D-state are 'on'.
     */
    i = ACPI_STATE_D0;
    while i < ACPI_STATE_D3 {
        list = &mut (*device).power.states[i as usize].resources;
        if (*list).count < 1 {
            i += 1;
            continue;
        }

        result = acpi_power_get_list_state(list, &mut list_state);
        if result != 0 {
            return result;
        }

        if list_state == ACPI_POWER_RESOURCE_STATE_ON {
            (*device).power.state = i;
            return 0;
        }
        i += 1;
    }

    (*device).power.state = ACPI_STATE_D3;

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_power_transition(device: *mut acpi_device, state: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut cl: *mut acpi_handle_list = core::ptr::null_mut(); /* Current Resources */
    let mut tl: *mut acpi_handle_list = core::ptr::null_mut(); /* Target Resources */
    let mut i: u32 = 0;

    if device.is_null() || (state < ACPI_STATE_D0) || (state > ACPI_STATE_D3) {
        return -1;
    }

    if ((*device).power.state < ACPI_STATE_D0) || ((*device).power.state > ACPI_STATE_D3) {
        return -15;
    }

    cl = &mut (*device).power.states[(*device).power.state as usize].resources;
    tl = &mut (*device).power.states[state as usize].resources;

    /* TBD: Resources must be ordered. */

    /*
     * First we reference all power resources required in the target list.
     */
    i = 0;
    while i < (*tl).count {
        result = acpi_power_on((*tl).handles[i as usize], device);
        if result != 0 {
            break; // goto end
        }
        i += 1;
    }

    if result == 0 && (*device).power.state == state {
        // goto end (state unchanged path)
    } else if result == 0 {
        /*
         * Then we dereference all power resources used in the current list.
         */
        i = 0;
        while i < (*cl).count {
            result = acpi_power_off_device((*cl).handles[i as usize], device);
            if result != 0 {
                break; // goto end
            }
            i += 1;
        }
    }

    // end:
    if result != 0 {
        (*device).power.state = ACPI_STATE_UNKNOWN;
    } else {
        /* We shouldn't change the state till all above operations succeed */
        (*device).power.state = state;
    }

    result
}

/* --------------------------------------------------------------------------
                            Driver Interface
   -------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "C" fn acpi_power_add(device: *mut acpi_device) -> c_int {
    let mut result: c_int = 0;
    let mut state: c_int = 0;
    let mut status: ACPI_STATUS = AE_OK;
    let mut resource: *mut acpi_power_resource = core::ptr::null_mut();
    let mut acpi_object: ACPI_OBJECT = core::mem::zeroed();
    let mut buffer = ACPI_BUFFER {
        Length: core::mem::size_of::<ACPI_OBJECT>() as ACPI_SIZE,
        Pointer: &mut acpi_object as *mut ACPI_OBJECT as *mut c_void,
    };

    if device.is_null() {
        return -1;
    }

    resource = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<acpi_power_resource>(),
        ACPI_POWER_TAG,
    ) as *mut acpi_power_resource;
    if resource.is_null() {
        return -4;
    }

    (*resource).device = device;
    INIT_LIST_HEAD(&mut (*resource).reference);

    strcpy(
        (*resource).name.as_mut_ptr(),
        (*device).pnp.bus_id.as_ptr(),
    );
    strcpy(acpi_device_name(device), ACPI_POWER_DEVICE_NAME.as_ptr() as *const c_char);
    strcpy(acpi_device_class(device), ACPI_POWER_CLASS.as_ptr() as *const c_char);
    (*device).driver_data = resource as *mut c_void;

    /* Evalute the object to get the system level and resource order. */
    status = AcpiEvaluateObject(
        (*device).handle,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut buffer,
    );
    if ACPI_FAILURE(status) {
        result = -15;
        // goto end
        if result != 0 {
            ExFreePool(resource as *mut c_void);
        }
        return result;
    }
    (*resource).system_level = acpi_object.PowerResource.SystemLevel;
    (*resource).order = acpi_object.PowerResource.ResourceOrder;

    result = acpi_power_get_state((*device).handle, &mut state);
    if result != 0 {
        // goto end
        if result != 0 {
            ExFreePool(resource as *mut c_void);
        }
        return result;
    }

    match state {
        ACPI_POWER_RESOURCE_STATE_ON => {
            (*device).power.state = ACPI_STATE_D0;
        }
        ACPI_POWER_RESOURCE_STATE_OFF => {
            (*device).power.state = ACPI_STATE_D3;
        }
        _ => {
            (*device).power.state = ACPI_STATE_UNKNOWN;
        }
    }

    DPRINT!();

    // end:
    if result != 0 {
        ExFreePool(resource as *mut c_void);
    }

    result
}

#[no_mangle]
pub unsafe extern "C" fn acpi_power_remove(device: *mut acpi_device, type_: c_int) -> c_int {
    let mut resource: *mut acpi_power_resource = core::ptr::null_mut();

    if device.is_null() || acpi_driver_data(device).is_null() {
        return -1;
    }

    resource = acpi_driver_data(device);

    // list_for_each_safe(node, next, &resource->reference)
    let head: *mut list_head = &mut (*resource).reference;
    let mut node: *mut list_head = (*head).next;
    let mut next: *mut list_head = (*node).next;
    while node != head {
        let r#ref = container_of_ref!(node);
        list_del(&mut (*r#ref).node);
        ExFreePool(r#ref as *mut c_void);
        node = next;
        next = (*node).next;
    }
    ExFreePool(resource as *mut c_void);

    0
}

unsafe extern "C" fn acpi_power_resume(device: *mut acpi_device, mut state: c_int) -> c_int {
    let mut result: c_int = 0;
    let mut resource: *mut acpi_power_resource = core::ptr::null_mut();

    if device.is_null() || acpi_driver_data(device).is_null() {
        return -1;
    }

    resource = acpi_driver_data(device);

    result = acpi_power_get_state((*device).handle, &mut state);
    if result != 0 {
        return result;
    }

    if state == ACPI_POWER_RESOURCE_STATE_OFF
        && list_empty(&mut (*resource).reference) == 0
    {
        let r#ref = container_of_ref!((*resource).reference.next);
        result = acpi_power_on((*device).handle, (*r#ref).device);
        return result;
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn acpi_power_init() -> c_int {
    let mut result: c_int = 0;

    DPRINT!();

    INIT_LIST_HEAD(core::ptr::addr_of_mut!(acpi_power_resource_list));

    // Populate the driver descriptor's name/class/ids (static mut arrays + ptr).
    // ACPI_POWER_DRIVER_NAME = "ACPI Power Resource Driver"
    strcpy(
        core::ptr::addr_of_mut!(acpi_power_driver.name) as *mut c_char,
        c"ACPI Power Resource Driver".as_ptr() as *const c_char,
    );
    // ACPI_POWER_CLASS = "power_resource"
    strcpy(
        core::ptr::addr_of_mut!(acpi_power_driver.class) as *mut c_char,
        ACPI_POWER_CLASS.as_ptr() as *const c_char,
    );
    // ACPI_POWER_HID = "ACPI_PWR"  (ids is *mut c_char → point at a static literal)
    acpi_power_driver.ids = c"ACPI_PWR".as_ptr() as *mut c_char;

    result = acpi_bus_register_driver(core::ptr::addr_of_mut!(acpi_power_driver));
    if result < 0 {
        return -15;
    }

    0
}
