use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PSTR};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[repr(C)]
pub enum VF_OBJECT_TYPE {
    VfObjectTypeUnknown = 0,
}

#[repr(C)]
pub enum VF_FAILURE_CLASS {
    VfFailureClassUnknown = 0,
}

#[no_mangle]
pub unsafe extern "system" fn VfIsVerificationEnabled(
    _vf_object_type: VF_OBJECT_TYPE,
    _object: PVOID,
) -> BOOLEAN {
    let _object = opaque_ptr(_object);
    0u8
}

#[no_mangle]
pub unsafe extern "C" fn VfFailDeviceNode(
    _physical_device_object: PDEVICE_OBJECT,
    _bug_check_major_code: ULONG,
    _bug_check_minor_code: ULONG,
    _failure_class: VF_FAILURE_CLASS,
    _assertion_control: *mut ULONG,
    _debugger_message_text: PSTR,
    _parameter_format_string: PSTR,
) {
    let _physical_device_object = opaque_ptr(_physical_device_object);
    let _assertion_control = opaque_ptr(_assertion_control);
    let _debugger_message_text = opaque_ptr(_debugger_message_text);
    let _parameter_format_string = opaque_ptr(_parameter_format_string);
}

#[no_mangle]
pub unsafe extern "C" fn VfFailSystemBIOS(
    _bug_check_major_code: ULONG,
    _bug_check_minor_code: ULONG,
    _failure_class: VF_FAILURE_CLASS,
    _assertion_control: *mut ULONG,
    _debugger_message_text: PSTR,
    _parameter_format_string: PSTR,
) {
    let _assertion_control = opaque_ptr(_assertion_control);
    let _debugger_message_text = opaque_ptr(_debugger_message_text);
    let _parameter_format_string = opaque_ptr(_parameter_format_string);
}

#[no_mangle]
pub unsafe extern "C" fn VfFailDriver(
    _bug_check_major_code: ULONG,
    _bug_check_minor_code: ULONG,
    _failure_class: VF_FAILURE_CLASS,
    _assertion_control: *mut ULONG,
    _debugger_message_text: PSTR,
    _parameter_format_string: PSTR,
) {
    let _assertion_control = opaque_ptr(_assertion_control);
    let _debugger_message_text = opaque_ptr(_debugger_message_text);
    let _parameter_format_string = opaque_ptr(_parameter_format_string);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            ntoskrnl/vf/driver.c
//  * PURPOSE:         Driver Verifier Device Driver Interface
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @unimplemented
//  */
// BOOLEAN
// NTAPI
// VfIsVerificationEnabled(IN VF_OBJECT_TYPE VfObjectType,
//                         IN PVOID Object OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// __cdecl
// VfFailDeviceNode(IN PDEVICE_OBJECT PhysicalDeviceObject,
//                  IN ULONG BugCheckMajorCode,
//                  IN ULONG BugCheckMinorCode,
//                  IN VF_FAILURE_CLASS FailureClass,
//                  IN OUT PULONG AssertionControl,
//                  IN PSTR DebuggerMessageText,
//                  IN PSTR ParameterFormatString,
//                  ...)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// __cdecl
// VfFailSystemBIOS(IN ULONG BugCheckMajorCode,
//                  IN ULONG BugCheckMinorCode,
//                  IN VF_FAILURE_CLASS FailureClass,
//                  IN OUT PULONG AssertionControl,
//                  IN PSTR DebuggerMessageText,
//                  IN PSTR ParameterFormatString,
//                  ...)
// {
//     UNIMPLEMENTED;
// }
//
// VOID
// __cdecl
// VfFailDriver(IN ULONG BugCheckMajorCode,
//              IN ULONG BugCheckMinorCode,
//              IN VF_FAILURE_CLASS FailureClass,
//              IN OUT PULONG AssertionControl,
//              IN PSTR DebuggerMessageText,
//              IN PSTR ParameterFormatString,
//              ...)
// {
//     UNIMPLEMENTED;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
