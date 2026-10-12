use crate::prelude::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    // UNIMPLEMENTED;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn WdAllocateWatchdog(p1: PVOID, p2: PVOID, p3: ULONG) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdAllocateDeferredWatchdog(p1: PVOID, p2: PVOID, p3: ULONG) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdFreeWatchdog(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdFreeDeferredWatchdog(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdStartWatch(p1: PVOID, p2: LARGE_INTEGER, p3: ULONG) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdStartDeferredWatch(p1: PVOID, p2: PVOID, p3: ULONG) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdStopWatch(p1: PVOID, p2: ULONG) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdStopDeferredWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdSuspendWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WdSuspendDeferredWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WdSuspendDeferredWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdResumeWatch(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WdResumeDeferredWatch(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WdResumeDeferredWatch(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdResetWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WdResetDeferredWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WdResetDeferredWatch(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WdEnterMonitoredSection(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WdEnterMonitoredSection(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WdExitMonitoredSection(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WdExitMonitoredSection(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdAttachContext(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdDetachContext(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdGetDeviceObject(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdGetLowestDeviceObject(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdGetLastEvent(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdCompleteEvent(p1: PVOID, p2: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdReferenceObject(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdDereferenceObject(p1: PVOID) {
    // UNIMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn WdMadeAnyProgress(p1: PVOID) -> BOOLEAN {
    // UNIMPLEMENTED;
    FALSE
}
