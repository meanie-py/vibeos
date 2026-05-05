//! Rust wrappers for `ob_x.h` FORCEINLINE functions whose bodies live in
//! `ntkrnl_sys::inline::ob`. This rlib used to live as a section of
//! `ntoskrnl-rust::shim_exports`; extracted into an isolated rlib (same
//! shape as `ke-inlines` / `cm-inlines`) to dodge the LLVM captures(none)
//! cross-CGU contagion that killed the first Phase 1 cm pilot.
//!
//! The wrappers are intentionally minimal: each one is a `#[no_mangle]
//! #[inline(never)]` extern-C entry that launders pointer arguments through
//! `opaque_ptr` (defeats LLVM's captures(none) attribute deduction on
//! noalias/readnone) and delegates to the corresponding
//! `ntkrnl_sys::inline::ob::*` body. Behaviour is identical to the previous
//! shim_exports placement.
//!
//! 22 functions total — every `ob_x.h` FORCEINLINE that's not classified as
//! "common" / "tier1" (those stay in `shim_exports`).

#![no_std]

use ntkrnl_sys::ddk::*;
use ntkrnl_sys::opaque_ptr;

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpSelectObjectLockSlot(header: POBJECT_HEADER) -> ULONG {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ob::ObpSelectObjectLockSlot(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpValidateAttributes(
    attributes: ULONG,
    previous_mode: KPROCESSOR_MODE,
) -> ULONG {
    ntkrnl_sys::inline::ob::ObpValidateAttributes(attributes, previous_mode)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireObjectLock(header: POBJECT_HEADER) {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ob::ObpAcquireObjectLock(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseObjectLock(header: POBJECT_HEADER) {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ob::ObpReleaseObjectLock(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireObjectLockShared(header: POBJECT_HEADER) {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ob::ObpAcquireObjectLockShared(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpEnterObjectTypeMutex(object_type: POBJECT_TYPE) {
    let object_type = opaque_ptr(object_type);
    ntkrnl_sys::inline::ob::ObpEnterObjectTypeMutex(object_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpLeaveObjectTypeMutex(object_type: POBJECT_TYPE) {
    let object_type = opaque_ptr(object_type);
    ntkrnl_sys::inline::ob::ObpLeaveObjectTypeMutex(object_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpCalloutStart(callout_irql: *mut u8) {
    let callout_irql = opaque_ptr(callout_irql);
    ntkrnl_sys::inline::ob::ObpCalloutStart(callout_irql)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpCalloutEnd(
    callout_irql: u8,
    procedure: *const u8,
    object_type: POBJECT_TYPE,
    object: PVOID,
) {
    let object_type = opaque_ptr(object_type);
    let object = opaque_ptr(object);
    ntkrnl_sys::inline::ob::ObpCalloutEnd(callout_irql, procedure, object_type, object)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReferenceNameInfo(
    header: POBJECT_HEADER,
) -> POBJECT_HEADER_NAME_INFO {
    let header = opaque_ptr(header);
    ntkrnl_sys::inline::ob::ObpReferenceNameInfo(header)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpDereferenceNameInfo(name_info: POBJECT_HEADER_NAME_INFO) {
    ntkrnl_sys::inline::ob::ObpDereferenceNameInfo(name_info)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpInitializeLookupContext(context: POBP_LOOKUP_CONTEXT) {
    ntkrnl_sys::inline::ob::ObpInitializeLookupContext(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireDirectoryLockShared(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpAcquireDirectoryLockShared(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireDirectoryLockExclusive(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpAcquireDirectoryLockExclusive(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseDirectoryLock(
    directory: POBJECT_DIRECTORY,
    context: POBP_LOOKUP_CONTEXT,
) {
    ntkrnl_sys::inline::ob::ObpReleaseDirectoryLock(directory, context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAcquireLookupContextLock(
    context: POBP_LOOKUP_CONTEXT,
    directory: POBJECT_DIRECTORY,
) {
    ntkrnl_sys::inline::ob::ObpAcquireLookupContextLock(context, directory)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseLookupContextObject(context: POBP_LOOKUP_CONTEXT) {
    ntkrnl_sys::inline::ob::ObpReleaseLookupContextObject(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseLookupContext(context: POBP_LOOKUP_CONTEXT) {
    ntkrnl_sys::inline::ob::ObpReleaseLookupContext(context)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpReleaseObjectCreateInformation(
    create_info: POBJECT_CREATE_INFORMATION,
) {
    let create_info = opaque_ptr(create_info);
    ntkrnl_sys::inline::ob::ObpReleaseObjectCreateInformation(create_info)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpAllocateObjectCreateInfoBuffer(
    lookaside_type: PP_NPAGED_LOOKASIDE_NUMBER,
) -> PVOID {
    ntkrnl_sys::inline::ob::ObpAllocateObjectCreateInfoBuffer(lookaside_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpFreeCapturedAttributes(
    buffer: PVOID,
    lookaside_type: PP_NPAGED_LOOKASIDE_NUMBER,
) {
    let buffer = opaque_ptr(buffer);
    ntkrnl_sys::inline::ob::ObpFreeCapturedAttributes(buffer, lookaside_type)
}

#[no_mangle]
#[inline(never)]
pub unsafe extern "C" fn ObpFreeObjectCreateInformation(
    create_info: POBJECT_CREATE_INFORMATION,
) {
    let create_info = opaque_ptr(create_info);
    ntkrnl_sys::inline::ob::ObpFreeObjectCreateInformation(create_info)
}
