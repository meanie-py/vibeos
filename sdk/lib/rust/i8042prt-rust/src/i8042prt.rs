use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, _IO_STATUS_BLOCK, IoAllocateDriverObjectExtension, IoAttachDeviceToDeviceStackSafe, IoBuildDeviceIoControlRequest, IoCreateDevice, IoDeleteDevice, IoDetachDevice, IoFreeWorkItem, IoGetAttachedDeviceReference, IoGetDriverObjectExtension, KEVENT, KeAcquireInterruptSpinLock, KeInitializeEvent, KeReleaseInterruptSpinLock, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent, PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PUNICODE_STRING};
// FORCEINLINE accessors (no exported symbol — must use the Rust inline impls).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use crate::i8042prt_h::*;

// pub(crate) siblings (importable by path)
use crate::misc::DuplicateUnicodeString;
use crate::registry::ReadRegistryEntries;
use crate::readwrite::i8042Write;
// i8042PacketDpc lives in keyboard.rs; i8042StartPacket/i8042PacketIsr are
// defined locally in this module.

// #[no_mangle] siblings: not importable by module path (they live at crate root
// for codegen), so link them by symbol via extern blocks.
extern "system" {
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042SynchReadPort(Context: PVOID, Value: PUCHAR, WaitForAck: BOOLEAN) -> NTSTATUS;
    fn i8042SynchWritePortKbd(Context: PVOID, Value: UCHAR, WaitForAck: BOOLEAN) -> NTSTATUS;
    fn i8042InitializeHwHacks();
    fn i8042KbdStartIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
    fn i8042KbdDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042KbdInternalDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042MouInternalDeviceControl(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042Create(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042Cleanup(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042Close(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn i8042Pnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// File-scope preamble: only forward declarations of dispatch/startio routines.
// No types, constants, or tables to translate.

#[no_mangle]
pub unsafe extern "system" fn i8042AddDevice(
    DriverObject: PDRIVER_OBJECT,
    Pdo: PDEVICE_OBJECT,
) -> NTSTATUS {
    let DriverExtension: PI8042_DRIVER_EXTENSION;
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let DeviceExtensionSize: ULONG;
    let mut Status: NTSTATUS;

    DriverExtension =
        IoGetDriverObjectExtension(DriverObject, DriverObject as *mut _) as PI8042_DRIVER_EXTENSION;

    if Pdo.is_null() {
        return STATUS_SUCCESS;
    }

    DeviceExtensionSize = {
        let a = core::mem::size_of::<I8042_KEYBOARD_EXTENSION>();
        let b = core::mem::size_of::<I8042_MOUSE_EXTENSION>();
        if a > b { a } else { b }
    } as ULONG;

    'cleanup: loop {
        Status = IoCreateDevice(
            DriverObject,
            DeviceExtensionSize,
            core::ptr::null_mut(),
            (*Pdo).DeviceType,
            FILE_DEVICE_SECURE_OPEN,
            1u8,
            core::ptr::addr_of_mut!(Fdo),
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        DeviceExtension = (*Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;
        RtlZeroMemory(DeviceExtension as *mut _, DeviceExtensionSize as usize);
        (*DeviceExtension).Type = Unknown;
        (*DeviceExtension).Fdo = Fdo;
        (*DeviceExtension).Pdo = Pdo;
        (*DeviceExtension).PortDeviceExtension = core::ptr::addr_of_mut!((*DriverExtension).Port);
        Status = IoAttachDeviceToDeviceStackSafe(
            Fdo,
            Pdo,
            core::ptr::addr_of_mut!((*DeviceExtension).LowerDevice),
        );
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        ExInterlockedInsertTailList(
            core::ptr::addr_of_mut!((*DriverExtension).DeviceListHead),
            core::ptr::addr_of_mut!((*DeviceExtension).ListEntry),
            core::ptr::addr_of_mut!((*DriverExtension).DeviceListLock),
        );

        (*Fdo).Flags &= !DO_DEVICE_INITIALIZING;
        return STATUS_SUCCESS;
    }

    if !DeviceExtension.is_null() && !(*DeviceExtension).LowerDevice.is_null() {
        IoDetachDevice((*DeviceExtension).LowerDevice);
    }
    if !Fdo.is_null() {
        IoDeleteDevice(Fdo);
    }
    Status
}

#[no_mangle]
pub unsafe extern "system" fn i8042SendHookWorkItem(
    DeviceObject: PDEVICE_OBJECT,
    Context: PVOID,
) {
    let WorkItemData: PI8042_HOOK_WORKITEM = Context as PI8042_HOOK_WORKITEM;
    let FdoDeviceExtension: PFDO_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION = (*FdoDeviceExtension).PortDeviceExtension;
    let mut TopOfStack: PDEVICE_OBJECT = core::ptr::null_mut();
    let IoControlCode: ULONG;
    let InputBuffer: PVOID;
    let InputBufferLength: ULONG;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Event: KEVENT = core::mem::zeroed();
    let NewIrp: PIRP;
    let mut Status: NTSTATUS;

    'cleanup: {
        match (*FdoDeviceExtension).Type {
            Keyboard => {
                let DeviceExtension: PI8042_KEYBOARD_EXTENSION =
                    FdoDeviceExtension as PI8042_KEYBOARD_EXTENSION;
                IoControlCode = IOCTL_INTERNAL_I8042_HOOK_KEYBOARD;
                InputBuffer =
                    core::ptr::addr_of_mut!((*DeviceExtension).KeyboardHook) as PVOID;
                InputBufferLength =
                    core::mem::size_of::<INTERNAL_I8042_HOOK_KEYBOARD>() as ULONG;
            }
            Mouse => {
                let DeviceExtension: PI8042_MOUSE_EXTENSION =
                    FdoDeviceExtension as PI8042_MOUSE_EXTENSION;
                IoControlCode = IOCTL_INTERNAL_I8042_HOOK_MOUSE;
                InputBuffer = core::ptr::addr_of_mut!((*DeviceExtension).MouseHook) as PVOID;
                InputBufferLength =
                    core::mem::size_of::<INTERNAL_I8042_HOOK_MOUSE>() as ULONG;
            }
            _ => {
                (*(*WorkItemData).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INTERNAL_ERROR;
                break 'cleanup;
            }
        }

        KeInitializeEvent(&mut Event, _EVENT_TYPE_NotificationEvent, 0u8);
        TopOfStack = IoGetAttachedDeviceReference(DeviceObject);

        NewIrp = IoBuildDeviceIoControlRequest(
            IoControlCode,
            TopOfStack,
            InputBuffer,
            InputBufferLength,
            core::ptr::null_mut(),
            0,
            1u8,
            &mut Event,
            &mut IoStatus,
        );

        if NewIrp.is_null() {
            (*(*WorkItemData).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'cleanup;
        }

        Status = IoCallDriver(TopOfStack, NewIrp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                &mut Event as *mut _ as PVOID,
                _KWAIT_REASON_Executive,
                0i32 as i8,
                0u8,
                core::ptr::null_mut(),
            );
            Status = IoStatus.__bindgen_anon_1.Status;
        }
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        if (*FdoDeviceExtension).Type == Keyboard {
            let DeviceExtension: PI8042_KEYBOARD_EXTENSION =
                FdoDeviceExtension as PI8042_KEYBOARD_EXTENSION;
            if (*DeviceExtension).KeyboardHook.InitializationRoutine.is_some() {
                // C passes &TurnTranslationOn (PBOOLEAN). rustconv dropped the
                // local; restore it so the hook can request 8042 translation.
                let mut TurnTranslationOn: u8 = 0;
                Status = (*DeviceExtension).KeyboardHook.InitializationRoutine.unwrap()(
                    (*DeviceExtension).KeyboardHook.Context,
                    PortDeviceExtension as PVOID,
                    i8042SynchReadPort as *mut core::ffi::c_void,
                    i8042SynchWritePortKbd as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(TurnTranslationOn),
                );
                if !NT_SUCCESS(Status) {
                    (*(*WorkItemData).Irp).IoStatus.__bindgen_anon_1.Status = Status;
                    break 'cleanup;
                }
            }
        }

        (*(*WorkItemData).Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    }

    if !TopOfStack.is_null() {
        ObDereferenceObject(TopOfStack as PVOID);
    }
    (*(*WorkItemData).Irp).IoStatus.Information = 0;
    IoCompleteRequest((*WorkItemData).Irp, IO_NO_INCREMENT as i8);

    IoFreeWorkItem((*WorkItemData).WorkItem);
    ExFreePoolWithTag(WorkItemData as PVOID, I8042PRT_TAG);
}

unsafe extern "system" fn i8042StartIo(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    match (*DeviceExtension).Type {
        Keyboard => {
            i8042KbdStartIo(DeviceObject, Irp);
        }
        _ => {}
    }
}

unsafe fn i8042PacketWrite(DeviceExtension: PPORT_DEVICE_EXTENSION) -> BOOLEAN {
    let Port: UCHAR = (*DeviceExtension).PacketPort;

    if Port != 0 {
        if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, Port) == 0 {
            return 0u8;
        }
    }

    i8042Write(
        DeviceExtension,
        (*DeviceExtension).DataPort,
        *(*DeviceExtension).Packet.Bytes.add((*DeviceExtension).Packet.CurrentByte as usize),
    )
}

pub(crate) unsafe fn i8042PacketIsr(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    Output: UCHAR,
) -> BOOLEAN {
    if (*DeviceExtension).Packet.State == Idle {
        return 0u8;
    }

    match Output {
        KBD_RESEND => {
            (*DeviceExtension).PacketResends += 1;
            if (*DeviceExtension).PacketResends > (*DeviceExtension).Settings.ResendIterations {
                (*DeviceExtension).Packet.State = Idle;
                (*DeviceExtension).PacketComplete = 1u8;
                (*DeviceExtension).PacketResult = STATUS_IO_TIMEOUT;
                (*DeviceExtension).PacketResends = 0;
                return 1u8;
            }
            (*DeviceExtension).Packet.CurrentByte -= 1;
        }
        KBD_NACK => {
            (*DeviceExtension).Packet.State = Idle;
            (*DeviceExtension).PacketComplete = 1u8;
            (*DeviceExtension).PacketResult = STATUS_UNEXPECTED_IO_ERROR;
            (*DeviceExtension).PacketResends = 0;
            return 1u8;
        }
        _ => {
            (*DeviceExtension).PacketResends = 0;
        }
    }

    if (*DeviceExtension).Packet.CurrentByte >= (*DeviceExtension).Packet.ByteCount {
        (*DeviceExtension).Packet.State = Idle;
        (*DeviceExtension).PacketComplete = 1u8;
        (*DeviceExtension).PacketResult = STATUS_SUCCESS;
        return 1u8;
    }

    if i8042PacketWrite(DeviceExtension) == 0 {
        (*DeviceExtension).Packet.State = Idle;
        (*DeviceExtension).PacketComplete = 1u8;
        (*DeviceExtension).PacketResult = STATUS_IO_TIMEOUT;
        return 1u8;
    }
    (*DeviceExtension).Packet.CurrentByte += 1;

    1u8
}

pub(crate) unsafe fn i8042StartPacket(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    FdoDeviceExtension: PFDO_DEVICE_EXTENSION,
    Bytes: PUCHAR,
    ByteCount: ULONG,
    Irp: PIRP,
) -> NTSTATUS {
    let Irql: KIRQL;
    let mut Status: NTSTATUS;

    Irql = KeAcquireInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt);

    'done: {
        if (*DeviceExtension).Packet.State != _MOUSE_TIMEOUT_STATE_Idle {
            Status = STATUS_DEVICE_BUSY;
            break 'done;
        }

        match (*FdoDeviceExtension).Type {
            _I8042_DEVICE_TYPE_Keyboard => {
                (*DeviceExtension).PacketPort = 0;
            }
            _I8042_DEVICE_TYPE_Mouse => {
                (*DeviceExtension).PacketPort = CTRL_WRITE_MOUSE as UCHAR;
            }
            _ => {
                Status = STATUS_INTERNAL_ERROR;
                break 'done;
            }
        }

        (*DeviceExtension).Packet.Bytes = Bytes;
        (*DeviceExtension).Packet.CurrentByte = 0;
        (*DeviceExtension).Packet.ByteCount = ByteCount;
        (*DeviceExtension).Packet.State = _MOUSE_TIMEOUT_STATE_SendingBytes;
        Status = STATUS_PENDING;
        (*DeviceExtension).PacketResult = Status;
        (*DeviceExtension).CurrentIrp = Irp;
        (*DeviceExtension).CurrentIrpDevice = (*FdoDeviceExtension).Fdo;

        if i8042PacketWrite(DeviceExtension) == 0 {
            Status = STATUS_IO_TIMEOUT;
            (*DeviceExtension).Packet.State = _MOUSE_TIMEOUT_STATE_Idle;
            (*DeviceExtension).PacketResult = STATUS_ABANDONED;
            break 'done;
        }

        (*DeviceExtension).Packet.CurrentByte += 1;
    }

    KeReleaseInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt, Irql);

    if Status != STATUS_PENDING {
        (*DeviceExtension).CurrentIrp = core::ptr::null_mut();
        (*DeviceExtension).CurrentIrpDevice = core::ptr::null_mut();
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    Status
}

unsafe extern "system" fn i8042DeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PFDO_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    match (*DeviceExtension).Type {
        Keyboard => i8042KbdDeviceControl(DeviceObject, Irp),
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INVALID_DEVICE_REQUEST;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            STATUS_INVALID_DEVICE_REQUEST
        }
    }
}

unsafe extern "system" fn i8042InternalDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension: PFDO_DEVICE_EXTENSION =
        (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    let ControlCode: ULONG;
    let Status: NTSTATUS;

    match (*DeviceExtension).Type {
        Unknown => {
            ControlCode = (*IoGetCurrentIrpStackLocation(Irp))
                .Parameters
                .DeviceIoControl
                .IoControlCode;
            match ControlCode {
                IOCTL_INTERNAL_KEYBOARD_CONNECT => {
                    Status = i8042KbdInternalDeviceControl(DeviceObject, Irp);
                }
                IOCTL_INTERNAL_MOUSE_CONNECT => {
                    Status = i8042MouInternalDeviceControl(DeviceObject, Irp);
                }
                _ => {
                    Status = STATUS_INVALID_DEVICE_REQUEST;
                }
            }
        }
        Keyboard => {
            Status = i8042KbdInternalDeviceControl(DeviceObject, Irp);
        }
        Mouse => {
            Status = i8042MouInternalDeviceControl(DeviceObject, Irp);
        }
        _ => {
            Status = STATUS_INTERNAL_ERROR;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
        }
    }

    Status
}

unsafe extern "system" fn i8042Power(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    let LowerDevice = (*DeviceExtension).LowerDevice;

    PoStartNextPowerIrp(Irp);
    IoSkipCurrentIrpStackLocation(Irp);
    PoCallDriver(LowerDevice, Irp)
}

unsafe extern "system" fn i8042SystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    ForwardIrpAndForget(DeviceObject, Irp)
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut DriverExtension: PI8042_DRIVER_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = IoAllocateDriverObjectExtension(
        DriverObject,
        DriverObject as *mut _,
        core::mem::size_of::<I8042_DRIVER_EXTENSION>() as u32,
        &mut DriverExtension as *mut _ as *mut PVOID,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    RtlZeroMemory(
        DriverExtension as *mut _,
        core::mem::size_of::<I8042_DRIVER_EXTENSION>(),
    );
    // KeInitializeSpinLock inline takes *mut ULONG_PTR; the KSPIN_LOCK field
    // type identity differs across the glob, so cast to the expected pointer.
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DriverExtension).Port.SpinLock) as *mut usize);
    InitializeListHead(core::ptr::addr_of_mut!((*DriverExtension).DeviceListHead));
    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DriverExtension).DeviceListLock) as *mut usize);

    Status = DuplicateUnicodeString(
        RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
        RegistryPath,
        core::ptr::addr_of_mut!((*DriverExtension).RegistryPath),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ReadRegistryEntries(
        core::ptr::addr_of_mut!((*DriverExtension).RegistryPath),
        core::ptr::addr_of_mut!((*DriverExtension).Port.Settings),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*(*DriverObject).DriverExtension).AddDevice = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DRIVER_OBJECT, *mut ntkrnl_sys::ddk::_DEVICE_OBJECT) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_ADD_DEVICE>(Some(i8042AddDevice));
    (*DriverObject).DriverStartIo = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP)>, ntkrnl_sys::ddk::PDRIVER_STARTIO>(Some(i8042StartIo));

    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042Create));
    (*DriverObject).MajorFunction[IRP_MJ_CLEANUP as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042Cleanup));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042Close));
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042DeviceControl));
    (*DriverObject).MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042InternalDeviceControl));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042Power));
    (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042SystemControl));
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_DEVICE_OBJECT, *mut ntkrnl_sys::ddk::_IRP) -> NTSTATUS>, ntkrnl_sys::ddk::PDRIVER_DISPATCH>(Some(i8042Pnp));

    i8042InitializeHwHacks();

    STATUS_SUCCESS
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/i8042prt.c
//  * PURPOSE:     Driver entry function
//  * PROGRAMMERS: Copyright Victor Kirhenshtein (sauros@iname.com)
//                 Copyright Jason Filby (jasonfilby@yahoo.com)
//                 Copyright Martijn Vernooij (o112w8r02@sneakemail.com)
//                 Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include "i8042prt.h"
//
// #include <debug.h>
//
// /* FUNCTIONS *****************************************************************/
//
// static DRIVER_STARTIO i8042StartIo;
// _Dispatch_type_(IRP_MJ_DEVICE_CONTROL)
// static DRIVER_DISPATCH i8042DeviceControl;
// _Dispatch_type_(IRP_MJ_INTERNAL_DEVICE_CONTROL)
// static DRIVER_DISPATCH i8042InternalDeviceControl;
// _Dispatch_type_(IRP_MJ_SYSTEM_CONTROL)
// static DRIVER_DISPATCH i8042SystemControl;
// _Dispatch_type_(IRP_MJ_POWER)
// static DRIVER_DISPATCH i8042Power;
// DRIVER_INITIALIZE DriverEntry;
//
// NTSTATUS NTAPI
// i8042AddDevice(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PDEVICE_OBJECT Pdo)
// {
// 	PI8042_DRIVER_EXTENSION DriverExtension;
// 	PFDO_DEVICE_EXTENSION DeviceExtension = NULL;
// 	PDEVICE_OBJECT Fdo = NULL;
// 	ULONG DeviceExtensionSize;
// 	NTSTATUS Status;
//
// 	TRACE_(I8042PRT, "i8042AddDevice(%p %p)\n", DriverObject, Pdo);
// 	DPRINT1("[[I8K]] AddDevice Pdo=%p\n", Pdo);
//
// 	DriverExtension = (PI8042_DRIVER_EXTENSION)IoGetDriverObjectExtension(DriverObject, DriverObject);
//
// 	if (Pdo == NULL)
// 	{
// 		/* We're getting a NULL Pdo at the first call as
// 		 * we are a legacy driver. Ignore it */
// 		return STATUS_SUCCESS;
// 	}
//
// 	/* Create new device object. As we don't know if the device would be a keyboard
// 	 * or a mouse, we have to allocate the biggest device extension. */
// 	DeviceExtensionSize = MAX(sizeof(I8042_KEYBOARD_EXTENSION), sizeof(I8042_MOUSE_EXTENSION));
// 	Status = IoCreateDevice(
// 		DriverObject,
// 		DeviceExtensionSize,
// 		NULL,
// 		Pdo->DeviceType,
// 		FILE_DEVICE_SECURE_OPEN,
// 		TRUE,
// 		&Fdo);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "IoCreateDevice() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
//
// 	DeviceExtension = (PFDO_DEVICE_EXTENSION)Fdo->DeviceExtension;
// 	RtlZeroMemory(DeviceExtension, DeviceExtensionSize);
// 	DeviceExtension->Type = Unknown;
// 	DeviceExtension->Fdo = Fdo;
// 	DeviceExtension->Pdo = Pdo;
// 	DeviceExtension->PortDeviceExtension = &DriverExtension->Port;
// 	Status = IoAttachDeviceToDeviceStackSafe(Fdo, Pdo, &DeviceExtension->LowerDevice);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "IoAttachDeviceToDeviceStackSafe() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
//
// 	ExInterlockedInsertTailList(
// 		&DriverExtension->DeviceListHead,
// 		&DeviceExtension->ListEntry,
// 		&DriverExtension->DeviceListLock);
//
// 	Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
// 	return STATUS_SUCCESS;
//
// cleanup:
// 	if (DeviceExtension && DeviceExtension->LowerDevice)
// 		IoDetachDevice(DeviceExtension->LowerDevice);
// 	if (Fdo)
// 		IoDeleteDevice(Fdo);
// 	return Status;
// }
//
// VOID NTAPI
// i8042SendHookWorkItem(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PVOID Context)
// {
// 	PI8042_HOOK_WORKITEM WorkItemData;
// 	PFDO_DEVICE_EXTENSION FdoDeviceExtension;
// 	PPORT_DEVICE_EXTENSION PortDeviceExtension;
// 	PDEVICE_OBJECT TopOfStack = NULL;
// 	ULONG IoControlCode;
// 	PVOID InputBuffer;
// 	ULONG InputBufferLength;
// 	IO_STATUS_BLOCK IoStatus;
// 	KEVENT Event;
// 	PIRP NewIrp;
// 	NTSTATUS Status;
//
// 	TRACE_(I8042PRT, "i8042SendHookWorkItem(%p %p)\n", DeviceObject, Context);
//
// 	WorkItemData = (PI8042_HOOK_WORKITEM)Context;
// 	FdoDeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	PortDeviceExtension = FdoDeviceExtension->PortDeviceExtension;
//
// 	switch (FdoDeviceExtension->Type)
// 	{
// 		case Keyboard:
// 		{
// 			PI8042_KEYBOARD_EXTENSION DeviceExtension;
// 			DeviceExtension = (PI8042_KEYBOARD_EXTENSION)FdoDeviceExtension;
// 			IoControlCode = IOCTL_INTERNAL_I8042_HOOK_KEYBOARD;
// 			InputBuffer = &DeviceExtension->KeyboardHook;
// 			InputBufferLength = sizeof(INTERNAL_I8042_HOOK_KEYBOARD);
// 			break;
// 		}
// 		case Mouse:
// 		{
// 			PI8042_MOUSE_EXTENSION DeviceExtension;
// 			DeviceExtension = (PI8042_MOUSE_EXTENSION)FdoDeviceExtension;
// 			IoControlCode = IOCTL_INTERNAL_I8042_HOOK_MOUSE;
// 			InputBuffer = &DeviceExtension->MouseHook;
// 			InputBufferLength = sizeof(INTERNAL_I8042_HOOK_MOUSE);
// 			break;
// 		}
// 		default:
// 		{
// 			ERR_(I8042PRT, "Unknown FDO type %u\n", FdoDeviceExtension->Type);
// 			ASSERT(FALSE);
// 			WorkItemData->Irp->IoStatus.Status = STATUS_INTERNAL_ERROR;
// 			goto cleanup;
// 		}
// 	}
//
// 	KeInitializeEvent(&Event, NotificationEvent, FALSE);
// 	TopOfStack = IoGetAttachedDeviceReference(DeviceObject);
//
// 	NewIrp = IoBuildDeviceIoControlRequest(
// 		IoControlCode,
// 		TopOfStack,
// 		InputBuffer,
// 		InputBufferLength,
// 		NULL,
// 		0,
// 		TRUE,
// 		&Event,
// 		&IoStatus);
//
// 	if (!NewIrp)
// 	{
// 		WARN_(I8042PRT, "IoBuildDeviceIoControlRequest() failed\n");
// 		WorkItemData->Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
// 		goto cleanup;
// 	}
//
// 	Status = IoCallDriver(TopOfStack, NewIrp);
// 	if (Status == STATUS_PENDING)
// 	{
// 		KeWaitForSingleObject(
// 			&Event,
// 			Executive,
// 			KernelMode,
// 			FALSE,
// 			NULL);
// 		Status = IoStatus.Status;
// 	}
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "IoCallDriver() failed with status 0x%08lx\n", Status);
// 		goto cleanup;
// 	}
//
// 	if (FdoDeviceExtension->Type == Keyboard)
// 	{
// 		PI8042_KEYBOARD_EXTENSION DeviceExtension;
//
// 		DeviceExtension = (PI8042_KEYBOARD_EXTENSION)FdoDeviceExtension;
// 		/* Call the hooked initialization if it exists */
// 		if (DeviceExtension->KeyboardHook.InitializationRoutine)
// 		{
// 			Status = DeviceExtension->KeyboardHook.InitializationRoutine(
// 				DeviceExtension->KeyboardHook.Context,
// 				PortDeviceExtension,
// 				i8042SynchReadPort,
// 				i8042SynchWritePortKbd,
// 				FALSE);
// 			if (!NT_SUCCESS(Status))
// 			{
// 				WARN_(I8042PRT, "KeyboardHook.InitializationRoutine() failed with status 0x%08lx\n", Status);
// 				WorkItemData->Irp->IoStatus.Status = Status;
// 				goto cleanup;
// 			}
// 		}
// 	}
//
// 	WorkItemData->Irp->IoStatus.Status = STATUS_SUCCESS;
//
// cleanup:
// 	if (TopOfStack != NULL)
// 		ObDereferenceObject(TopOfStack);
// 	WorkItemData->Irp->IoStatus.Information = 0;
// 	IoCompleteRequest(WorkItemData->Irp, IO_NO_INCREMENT);
//
// 	IoFreeWorkItem(WorkItemData->WorkItem);
// 	ExFreePoolWithTag(WorkItemData, I8042PRT_TAG);
// }
//
// static VOID NTAPI
// i8042StartIo(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PFDO_DEVICE_EXTENSION DeviceExtension;
//
// 	DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
// 	switch (DeviceExtension->Type)
// 	{
// 		case Keyboard:
// 			i8042KbdStartIo(DeviceObject, Irp);
// 			break;
// 		default:
// 			ERR_(I8042PRT, "Unknown FDO type %u\n", DeviceExtension->Type);
// 			ASSERT(FALSE);
// 			break;
// 	}
// }
//
// /* Write the current byte of the packet. Returns FALSE in case
//  * of problems.
//  */
// static BOOLEAN
// i8042PacketWrite(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
// 	UCHAR Port = DeviceExtension->PacketPort;
//
// 	if (Port)
// 	{
// 		if (!i8042Write(DeviceExtension,
// 		                DeviceExtension->ControlPort,
// 		                Port))
// 		{
// 			/* something is really wrong! */
// 			WARN_(I8042PRT, "Failed to send packet byte!\n");
// 			return FALSE;
// 		}
// 	}
//
// 	return i8042Write(DeviceExtension,
// 	                  DeviceExtension->DataPort,
// 	                  DeviceExtension->Packet.Bytes[DeviceExtension->Packet.CurrentByte]);
// }
//
// BOOLEAN
// i8042PacketIsr(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN UCHAR Output)
// {
// 	if (DeviceExtension->Packet.State == Idle)
// 		return FALSE;
//
// 	switch (Output)
// 	{
// 		case KBD_RESEND:
// 			DeviceExtension->PacketResends++;
// 			if (DeviceExtension->PacketResends > DeviceExtension->Settings.ResendIterations)
// 			{
// 				DeviceExtension->Packet.State = Idle;
// 				DeviceExtension->PacketComplete = TRUE;
// 				DeviceExtension->PacketResult = STATUS_IO_TIMEOUT;
// 				DeviceExtension->PacketResends = 0;
// 				return TRUE;
// 			}
// 			DeviceExtension->Packet.CurrentByte--;
// 			break;
//
// 		case KBD_NACK:
// 			DeviceExtension->Packet.State = Idle;
// 			DeviceExtension->PacketComplete = TRUE;
// 			DeviceExtension->PacketResult = STATUS_UNEXPECTED_IO_ERROR;
// 			DeviceExtension->PacketResends = 0;
// 			return TRUE;
//
// 		default:
// 			DeviceExtension->PacketResends = 0;
// 	}
//
// 	if (DeviceExtension->Packet.CurrentByte >= DeviceExtension->Packet.ByteCount)
// 	{
// 		DeviceExtension->Packet.State = Idle;
// 		DeviceExtension->PacketComplete = TRUE;
// 		DeviceExtension->PacketResult = STATUS_SUCCESS;
// 		return TRUE;
// 	}
//
// 	if (!i8042PacketWrite(DeviceExtension))
// 	{
// 		DeviceExtension->Packet.State = Idle;
// 		DeviceExtension->PacketComplete = TRUE;
// 		DeviceExtension->PacketResult = STATUS_IO_TIMEOUT;
// 		return TRUE;
// 	}
// 	DeviceExtension->Packet.CurrentByte++;
//
// 	return TRUE;
// }
//
// /*
//  * This function starts a packet. It must be called with the
//  * correct DIRQL.
//  */
// NTSTATUS
// i8042StartPacket(
// 	IN PPORT_DEVICE_EXTENSION DeviceExtension,
// 	IN PFDO_DEVICE_EXTENSION FdoDeviceExtension,
// 	IN PUCHAR Bytes,
// 	IN ULONG ByteCount,
// 	IN PIRP Irp)
// {
// 	KIRQL Irql;
// 	NTSTATUS Status;
//
// 	Irql = KeAcquireInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt);
//
// 	if (DeviceExtension->Packet.State != Idle)
// 	{
// 		Status = STATUS_DEVICE_BUSY;
// 		goto done;
// 	}
//
// 	switch (FdoDeviceExtension->Type)
// 	{
// 		case Keyboard: DeviceExtension->PacketPort = 0; break;
// 		case Mouse: DeviceExtension->PacketPort = CTRL_WRITE_MOUSE; break;
// 		default:
// 			ERR_(I8042PRT, "Unknown FDO type %u\n", FdoDeviceExtension->Type);
// 			ASSERT(FALSE);
// 			Status = STATUS_INTERNAL_ERROR;
// 			goto done;
// 	}
//
// 	DeviceExtension->Packet.Bytes = Bytes;
// 	DeviceExtension->Packet.CurrentByte = 0;
// 	DeviceExtension->Packet.ByteCount = ByteCount;
// 	DeviceExtension->Packet.State = SendingBytes;
// 	DeviceExtension->PacketResult = Status = STATUS_PENDING;
// 	DeviceExtension->CurrentIrp = Irp;
// 	DeviceExtension->CurrentIrpDevice = FdoDeviceExtension->Fdo;
//
// 	if (!i8042PacketWrite(DeviceExtension))
// 	{
// 		Status = STATUS_IO_TIMEOUT;
// 		DeviceExtension->Packet.State = Idle;
// 		DeviceExtension->PacketResult = STATUS_ABANDONED;
// 		goto done;
// 	}
//
// 	DeviceExtension->Packet.CurrentByte++;
//
// done:
// 	KeReleaseInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt, Irql);
//
// 	if (Status != STATUS_PENDING)
// 	{
// 		DeviceExtension->CurrentIrp = NULL;
// 		DeviceExtension->CurrentIrpDevice = NULL;
// 		Irp->IoStatus.Status = Status;
// 		IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 	}
// 	return Status;
// }
//
// static NTSTATUS NTAPI
// i8042DeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PFDO_DEVICE_EXTENSION DeviceExtension;
//
// 	TRACE_(I8042PRT, "i8042DeviceControl(%p %p)\n", DeviceObject, Irp);
// 	DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (DeviceExtension->Type)
// 	{
// 		case Keyboard:
// 			return i8042KbdDeviceControl(DeviceObject, Irp);
// 		default:
// 			Irp->IoStatus.Status = STATUS_INVALID_DEVICE_REQUEST;
// 			IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 			return STATUS_INVALID_DEVICE_REQUEST;
// 	}
// }
//
// static NTSTATUS NTAPI
// i8042InternalDeviceControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PFDO_DEVICE_EXTENSION DeviceExtension;
// 	ULONG ControlCode;
// 	NTSTATUS Status;
//
// 	TRACE_(I8042PRT, "i8042InternalDeviceControl(%p %p)\n", DeviceObject, Irp);
// 	DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
// 	switch (DeviceExtension->Type)
// 	{
// 		case Unknown:
// 		{
// 			ControlCode = IoGetCurrentIrpStackLocation(Irp)->Parameters.DeviceIoControl.IoControlCode;
// 			switch (ControlCode)
// 			{
// 				case IOCTL_INTERNAL_KEYBOARD_CONNECT:
// 					Status = i8042KbdInternalDeviceControl(DeviceObject, Irp);
// 					break;
// 				case IOCTL_INTERNAL_MOUSE_CONNECT:
// 					Status = i8042MouInternalDeviceControl(DeviceObject, Irp);
// 					break;
// 				default:
// 					ERR_(I8042PRT, "Unknown IO control code 0x%lx\n", ControlCode);
// 					ASSERT(FALSE);
// 					Status = STATUS_INVALID_DEVICE_REQUEST;
// 					break;
// 			}
// 			break;
// 		}
// 		case Keyboard:
// 			Status = i8042KbdInternalDeviceControl(DeviceObject, Irp);
// 			break;
// 		case Mouse:
// 			Status = i8042MouInternalDeviceControl(DeviceObject, Irp);
// 			break;
// 		default:
// 			ERR_(I8042PRT, "Unknown FDO type %u\n", DeviceExtension->Type);
// 			ASSERT(FALSE);
// 			Status = STATUS_INTERNAL_ERROR;
// 			IoCompleteRequest(Irp, IO_NO_INCREMENT);
// 			break;
// 	}
//
// 	return Status;
// }
//
// static NTSTATUS NTAPI
// i8042Power(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	PFDO_DEVICE_EXTENSION DeviceExtension = DeviceObject->DeviceExtension;
// 	PDEVICE_OBJECT LowerDevice = DeviceExtension->LowerDevice;
//
// 	PoStartNextPowerIrp(Irp);
// 	IoSkipCurrentIrpStackLocation(Irp);
// 	return PoCallDriver(LowerDevice, Irp);
// }
//
// static NTSTATUS NTAPI
// i8042SystemControl(
// 	IN PDEVICE_OBJECT DeviceObject,
// 	IN PIRP Irp)
// {
// 	return ForwardIrpAndForget(DeviceObject, Irp);
// }
//
// NTSTATUS NTAPI
// DriverEntry(
// 	IN PDRIVER_OBJECT DriverObject,
// 	IN PUNICODE_STRING RegistryPath)
// {
// 	PI8042_DRIVER_EXTENSION DriverExtension;
// 	NTSTATUS Status;
//
// 	Status = IoAllocateDriverObjectExtension(
// 		DriverObject,
// 		DriverObject,
// 		sizeof(I8042_DRIVER_EXTENSION),
// 		(PVOID*)&DriverExtension);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "IoAllocateDriverObjectExtension() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
// 	RtlZeroMemory(DriverExtension, sizeof(I8042_DRIVER_EXTENSION));
// 	KeInitializeSpinLock(&DriverExtension->Port.SpinLock);
// 	InitializeListHead(&DriverExtension->DeviceListHead);
// 	KeInitializeSpinLock(&DriverExtension->DeviceListLock);
//
// 	Status = DuplicateUnicodeString(
// 		RTL_DUPLICATE_UNICODE_STRING_NULL_TERMINATE,
// 		RegistryPath,
// 		&DriverExtension->RegistryPath);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "DuplicateUnicodeString() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
//
// 	Status = ReadRegistryEntries(&DriverExtension->RegistryPath, &DriverExtension->Port.Settings);
// 	if (!NT_SUCCESS(Status))
// 	{
// 		WARN_(I8042PRT, "ReadRegistryEntries() failed with status 0x%08lx\n", Status);
// 		return Status;
// 	}
//
// 	DriverObject->DriverExtension->AddDevice = i8042AddDevice;
// 	DriverObject->DriverStartIo = i8042StartIo;
//
// 	DriverObject->MajorFunction[IRP_MJ_CREATE]  = i8042Create;
// 	DriverObject->MajorFunction[IRP_MJ_CLEANUP] = i8042Cleanup;
// 	DriverObject->MajorFunction[IRP_MJ_CLOSE]   = i8042Close;
// 	DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = i8042DeviceControl;
// 	DriverObject->MajorFunction[IRP_MJ_INTERNAL_DEVICE_CONTROL] = i8042InternalDeviceControl;
// 	DriverObject->MajorFunction[IRP_MJ_POWER]   = i8042Power;
// 	DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = i8042SystemControl;
// 	DriverObject->MajorFunction[IRP_MJ_PNP]     = i8042Pnp;
//
//     i8042InitializeHwHacks();
//
// 	return STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
