#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{PDRIVER_NOTIFICATION_CALLBACK_ROUTINE as DRIVER_NOTIFICATION_CALLBACK_ROUTINE, IoRegisterPlugPlayNotification, PDRIVER_OBJECT, VF_FAILURE_CLASS as VFFAILURE_FLAGS};
use ntkrnl_sys::ddk::{
    _VF_FAILURE_CLASS_VFFAILURE_FAIL_LOGO as VFFAILURE_FAIL_LOGO,
    _VF_FAILURE_CLASS_VFFAILURE_FAIL_UNDER_DEBUGGER as VFFAILURE_FAIL_UNDER_DEBUGGER,
    _VF_OBJECT_TYPE_VFOBJTYPE_SYSTEM_BIOS as VFOBJTYPE_SYSTEM_BIOS,
    PDEVICE_OBJECT, PULONG, PSTR,
};

extern "system" {
    // Driver Verifier interface (ndk/vffuncs.h; not in ntkrnl_sys bindgen)
    fn VfIsVerificationEnabled(VfObjectType: u32, Object: PVOID) -> BOOLEAN;
}
extern "C" {
    // variadic → must be cdecl (NTAPI variadics are cdecl on all arches)
    fn VfFailDeviceNode(
        PhysicalDeviceObject: PDEVICE_OBJECT,
        BugCheckMajorCode: ULONG,
        BugCheckMinorCode: ULONG,
        FailureClass: VFFAILURE_FLAGS,
        AssertionControl: PULONG,
        DebuggerMessageText: PSTR,
        ParameterFormatString: PSTR,
        ...
    );
}
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{PCSTR, RtlAssert, _IO_NOTIFICATION_EVENT_CATEGORY_EventCategoryHardwareProfileChange as EventCategoryHardwareProfileChange};
use ntkrnl_sys::driver_ddk::{DbgPrint};
use ntkrnl_sys::ke::{DbgBreakPoint};

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciVerifierRegistered: BOOLEAN = 0;

pub static mut PciVerifierNotificationHandle: PVOID =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

pub static mut PciVerifierFailureTable: [PCI_VERIFIER_DATA; 4] = [
    _PCI_VERIFIER_DATA {
        FailureCode: 1 as ULONG,
        FailureClass: VFFAILURE_FAIL_LOGO,
        AssertionControl: 0 as ULONG,
        DebuggerMessageText: b"The BIOS has reprogrammed the bus numbers of an active PCI device (!devstack %DevObj) during a dock or undock!\0"
            as *const u8 as *const ::core::ffi::c_char as PCHAR,
    },
    _PCI_VERIFIER_DATA {
        FailureCode: 2 as ULONG,
        FailureClass: VFFAILURE_FAIL_LOGO,
        AssertionControl: 0 as ULONG,
        DebuggerMessageText: b"A device in the system did not update it's PMCSR register in the spec mandated time (!devstack %DevObj, Power state D%Ulong)\0"
            as *const u8 as *const ::core::ffi::c_char as PCHAR,
    },
    _PCI_VERIFIER_DATA {
        FailureCode: 3 as ULONG,
        FailureClass: VFFAILURE_FAIL_LOGO,
        AssertionControl: 0 as ULONG,
        DebuggerMessageText: b"A driver controlling a PCI device has tried to access OS controlled configuration space registers (!devstack %DevObj, Offset 0x%Ulong1, Length 0x%Ulong2)\0"
            as *const u8 as *const ::core::ffi::c_char as PCHAR,
    },
    _PCI_VERIFIER_DATA {
        FailureCode: 4 as ULONG,
        FailureClass: VFFAILURE_FAIL_UNDER_DEBUGGER,
        AssertionControl: 0 as ULONG,
        DebuggerMessageText: b"A driver controlling a PCI device has tried to read or write from an invalid space using IRP_MN_READ/WRITE_CONFIG or via BUS_INTERFACE_STANDARD.  NB: These functions take WhichSpace parameters of the form PCI_WHICHSPACE_* and not a BUS_DATA_TYPE (!devstack %DevObj, WhichSpace 0x%Ulong1)\0"
            as *const u8 as *const ::core::ffi::c_char as PCHAR,
    },
];

// (PCI_VERIFIER_DATA is defined in crate::types as _PCI_VERIFIER_DATA with the
//  FailureCode/FailureClass/AssertionControl/DebuggerMessageText layout used above;
//  the earlier local redefinition was a bogus duplicate and has been removed.)

// (C line 77 `DRIVER_NOTIFICATION_CALLBACK_ROUTINE PciVerifierProfileChangeCallback;`
//  is a function forward-prototype, not a variable — the fn is defined below.)

#[no_mangle]
pub unsafe extern "system" fn PciVerifierRetrieveFailureData(
    FailureCode: ULONG,
) -> PPCI_VERIFIER_DATA {
    let mut verifier_data = core::ptr::addr_of_mut!(PciVerifierFailureTable[0]) as PPCI_VERIFIER_DATA;
    while (*verifier_data).FailureCode != FailureCode {
        verifier_data = verifier_data.offset(1);
        let end_ptr = (core::ptr::addr_of_mut!(PciVerifierFailureTable[0]) as *mut PCI_VERIFIER_DATA)
            .offset(PCI_VERIFIER_CODES as isize);
        if !(verifier_data < (end_ptr).cast()) {
            RtlAssert(
                b"VerifierData < &PciVerifierFailureTable[PCI_VERIFIER_CODES]\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/pcivrify.c\0".as_ptr() as PVOID,
                70 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    verifier_data
}

#[no_mangle]
pub unsafe extern "system" fn PciVerifierProfileChangeCallback(
    NotificationStructure: *mut core::ffi::c_void,
    Context: *mut core::ffi::c_void,
) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"PciVerifierProfileChangeCallback\0" as *const u8 as *const i8,
        b"drivers/bus/pcix/pcivrify.c\0" as *const u8
            as *const i8,
        88i32,
    );
    DbgPrint(b"\0" as *const u8 as *const i8);
    DbgBreakPoint();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciVerifierInit(DriverObject: PDRIVER_OBJECT) {
    let mut Status: NTSTATUS;
    if VfIsVerificationEnabled(VFOBJTYPE_SYSTEM_BIOS, core::ptr::null_mut()) != 0 {
        Status = IoRegisterPlugPlayNotification(
            EventCategoryHardwareProfileChange,
            0,
            core::ptr::null_mut(),
            DriverObject,
            // extern "system" == stdcall on i386; transmute to the bindgen
            // Option<extern "system" fn(..)> callback type (same ABI per arch).
            core::mem::transmute::<_, DRIVER_NOTIFICATION_CALLBACK_ROUTINE>(Some(
                PciVerifierProfileChangeCallback
                    as unsafe extern "system" fn(*mut core::ffi::c_void, *mut core::ffi::c_void) -> NTSTATUS,
            )),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(PciVerifierNotificationHandle),
        );
        if NT_SUCCESS(Status) {
            PciVerifierRegistered = 1u8;
        }
    }
}

/// Undo PciVerifierInit's notification registration (pcix's unload; not in the C).
pub unsafe fn PciVerifierUnload() {
    if PciVerifierRegistered != 0 {
        ntkrnl_sys::ddk::IoUnregisterPlugPlayNotification(PciVerifierNotificationHandle);
        PciVerifierNotificationHandle = core::ptr::null_mut();
        PciVerifierRegistered = 0;
    }
}
