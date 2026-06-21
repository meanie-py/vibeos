use ntkrnl_sys::globals::*;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}
// KsecInitializeEncryptionSupport (crypt.rs) is defined NTAPI = `extern "system"`
// in this crate; import with matching ABI. KsecDdDispatch (dispatch.rs) is called
// via crate::dispatch::KsecDdDispatch in the dispatch-slot transmute below.
extern "system" {
    fn KsecInitializeEncryptionSupport();
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut KsecDeviceObject: PDEVICE_OBJECT =
    ::core::ptr::null::<_DEVICE_OBJECT>() as *mut _DEVICE_OBJECT;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    const FILE_DEVICE_KSEC: u32 = 0x00000039;
    const IRP_MJ_QUERY_VOLUME_INFORMATION: u32 = 0x0a;

    // RTL_CONSTANT_STRING(L"\\Device\\KsecDD") — bind the wide-string buffer to a
    // local so `addr_of!` has a non-temporary place to point at (faithful: same
    // bytes, same length arithmetic as the c2rust temporary).
    static mut DeviceNameBuf: [core::ffi::c_ushort; 15] = unsafe {
        core::mem::transmute::<[u8; 30], [core::ffi::c_ushort; 15]>(
            *b"\\\0D\0e\0v\0i\0c\0e\0\\\0K\0s\0e\0c\0D\0D\0\0\0",
        )
    };
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: (core::mem::size_of::<[core::ffi::c_ushort; 15]>() as usize)
            .wrapping_sub(core::mem::size_of::<core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: (core::mem::size_of::<[core::ffi::c_ushort; 15]>() as usize)
            .wrapping_div(core::mem::size_of::<core::ffi::c_char>() as usize)
            as USHORT,
        Buffer: core::ptr::addr_of!(DeviceNameBuf) as *const core::ffi::c_ushort as PWCH,
    };
    let mut Status: NTSTATUS;

    Status = IoCreateDevice(
        DriverObject,
        0 as ULONG,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_KSEC as ULONG,
        0x100 as ULONG,
        0 as BOOLEAN,
        core::ptr::addr_of_mut!(KsecDeviceObject),
    );

    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) Failed to create KsecDD device: 0x%lx\n\0" as *const u8 as PCSTR,
            b"ksecdd.c\0" as *const u8 as *const core::ffi::c_char,
            52 as core::ffi::c_int,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"ksecdd.c\0" as *const u8 as *const core::ffi::c_char,
                52 as core::ffi::c_int,
            );
        }
        return Status;
    }

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_READ as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        core::mem::transmute::<
            unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS,
            PDRIVER_DISPATCH,
        >(crate::dispatch::KsecDdDispatch);

    KsecInitializeEncryptionSupport();

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Drivers
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PURPOSE:         Kernel Security Support Provider Interface Driver
//  *
//  * PROGRAMMERS:     Timo Kreuzer (timo.kreuzer@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "ksecdd.h"
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// PDEVICE_OBJECT KsecDeviceObject;
//
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath)
// {
//     UNICODE_STRING DeviceName = RTL_CONSTANT_STRING(L"\\Device\\KsecDD");
//     NTSTATUS Status;
//
//     /* Debugcon traces for i386 boot debugging */
// #ifdef _M_IX86
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'0'), "Nd"((unsigned short)0xE9));
// #endif
//
//     /* Create the KsecDD device */
//     Status = IoCreateDevice(DriverObject,
//                             0,
//                             &DeviceName,
//                             FILE_DEVICE_KSEC,
//                             0x100u,
//                             FALSE,
//                             &KsecDeviceObject);
// #ifdef _M_IX86
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'1'), "Nd"((unsigned short)0xE9));
// #endif
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create KsecDD device: 0x%lx\n", Status);
//         return Status;
//     }
//
//     /* Set up dispatch table */
//     DriverObject->MajorFunction[IRP_MJ_CREATE] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_CLOSE] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_READ] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_WRITE] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_INFORMATION] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_QUERY_VOLUME_INFORMATION] = KsecDdDispatch;
//     DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = KsecDdDispatch;
//
// #ifdef _M_IX86
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'2'), "Nd"((unsigned short)0xE9));
// #endif
//
//     /* Initialize */
//     KsecInitializeEncryptionSupport();
//
// #ifdef _M_IX86
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'K'), "Nd"((unsigned short)0xE9));
//     __asm__ __volatile__("outb %b0, %w1" : : "a"((unsigned char)'3'), "Nd"((unsigned short)0xE9));
// #endif
//
//     return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
