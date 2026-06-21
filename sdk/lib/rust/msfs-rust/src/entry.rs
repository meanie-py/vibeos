//! entry.rs — msfs.c: DriverEntry. Creates \Device\MailSlot (FILE_DEVICE_MAILSLOT,
//! DO_DIRECT_IO), wires the dispatch table, and initializes the device-extension FCB list.
#![allow(non_snake_case)]

use core::mem::{size_of, transmute, zeroed};
use core::ptr::{addr_of_mut, null_mut};

use ntkrnl_sys::base::{InitializeListHead, NTSTATUS, NT_SUCCESS, ULONG};
use ntkrnl_sys::ddk::{
    IoCreateDevice, KeInitializeMutex, PDEVICE_OBJECT, PDRIVER_DISPATCH, PDRIVER_OBJECT,
    PUNICODE_STRING, RtlInitUnicodeString, UNICODE_STRING,
};

use crate::create::{MsfsClose, MsfsCreate, MsfsCreateMailslot};
use crate::finfo::{MsfsQueryInformation, MsfsSetInformation};
use crate::fsctrl::MsfsFileSystemControl;
use crate::glue::*;
use crate::helpers::wsz;
use crate::rw::{MsfsRead, MsfsWrite};
use crate::types::{MSFS_DEVICE_EXTENSION, PMSFS_DEVICE_EXTENSION};

/// All msfs dispatch routines are NTAPI `NTSTATUS (PDEVICE_OBJECT, PIRP)`; `extern "system"`
/// is the right ABI per arch. The MajorFunction slot type (PDRIVER_DISPATCH) is the bindgen
/// `Option<extern "C" fn>`, so we transmute the fn item to it (null-niche compatible).
type DispatchFn = unsafe extern "system" fn(PDEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS;

// L"\\Device\\MailSlot" — 16 wide chars + NUL.
static MAILSLOT_DEVICE_NAME: [u16; 17] = wsz(b"\\Device\\MailSlot");

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let _ = RegistryPath; // UNREFERENCED_PARAMETER

    (*DriverObject).Flags = 0;
    (*DriverObject).MajorFunction[IRP_MJ_CREATE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsCreate);
    (*DriverObject).MajorFunction[IRP_MJ_CREATE_MAILSLOT] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsCreateMailslot);
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsClose);
    (*DriverObject).MajorFunction[IRP_MJ_READ] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsRead);
    (*DriverObject).MajorFunction[IRP_MJ_WRITE] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsWrite);
    (*DriverObject).MajorFunction[IRP_MJ_QUERY_INFORMATION] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsQueryInformation);
    (*DriverObject).MajorFunction[IRP_MJ_SET_INFORMATION] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsSetInformation);
    (*DriverObject).MajorFunction[IRP_MJ_FILE_SYSTEM_CONTROL] =
        transmute::<DispatchFn, PDRIVER_DISPATCH>(MsfsFileSystemControl);

    (*DriverObject).DriverUnload = None;

    let mut DeviceName: UNICODE_STRING = zeroed();
    RtlInitUnicodeString(addr_of_mut!(DeviceName), MAILSLOT_DEVICE_NAME.as_ptr());

    let mut DeviceObject: PDEVICE_OBJECT = null_mut();
    let Status = IoCreateDevice(
        DriverObject,
        size_of::<MSFS_DEVICE_EXTENSION>() as ULONG,
        addr_of_mut!(DeviceName),
        FILE_DEVICE_MAILSLOT,
        0,
        FALSE,
        addr_of_mut!(DeviceObject),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* initialize the device object */
    (*DeviceObject).Flags |= DO_DIRECT_IO;

    /* initialize device extension */
    let DeviceExtension = (*DeviceObject).DeviceExtension as PMSFS_DEVICE_EXTENSION;
    InitializeListHead(addr_of_mut!((*DeviceExtension).FcbListHead));
    KeInitializeMutex(addr_of_mut!((*DeviceExtension).FcbListLock), 0);

    STATUS_SUCCESS
}
