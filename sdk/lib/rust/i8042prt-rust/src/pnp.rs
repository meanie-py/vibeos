use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    _DEVICE_RELATION_TYPE_BusRelations as BusRelations,
    _DEVICE_RELATION_TYPE_RemovalRelations as RemovalRelations,
    _CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared,
    _KINTERRUPT_MODE_Latched as Latched,
    _KINTERRUPT_MODE_LevelSensitive as LevelSensitive,
    IoConnectInterrupt, IoDeleteDevice, IoDetachDevice, IoDisconnectInterrupt,
    IoForwardIrpSynchronously, IoGetDriverObjectExtension,
    KeAcquireInterruptSpinLock, KeReleaseInterruptSpinLock, KeStallExecutionProcessor,
    PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT,
    PIO_STACK_LOCATION, PIRP,
};
// FORCEINLINE accessor (no exported symbol — use the Rust inline impl).
use ntkrnl_sys::inline::tier1::IoGetCurrentIrpStackLocation;
use core::ffi::c_void;
use crate::i8042prt_h::*;
use crate::readwrite::{
    i8042Write, i8042ReadDataWait, i8042IsrWritePort, i8042SynchWritePort, i8042Flush,
};
use crate::misc::ForwardIrpAndForget;
// i8042HwFlags is a `pub static mut u32` in hwhacks (read-only here).
use crate::hwhacks::i8042HwFlags;
use crate::keyboard::i8042KbdInterruptService;
use crate::mouse::{i8042MouInterruptService, i8042MouInitialize};

// (i8042Flush now imported from readwrite by path above.)

unsafe fn i8042ChangeMode(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    FlagsToDisable: UCHAR,
    FlagsToEnable: UCHAR,
) -> BOOLEAN {
    let mut Value: UCHAR = 0;
    let Status: NTSTATUS;

    if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, KBD_READ_MODE as UCHAR) == 0 {
        return 0u8;
    }

    Status = i8042ReadDataWait(DeviceExtension, &mut Value);
    if !NT_SUCCESS(Status) {
        return 0u8;
    }

    Value &= !FlagsToDisable;
    Value |= FlagsToEnable;

    if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, KBD_WRITE_MODE as UCHAR) == 0 {
        return 0u8;
    }

    if i8042Write(DeviceExtension, (*DeviceExtension).DataPort, Value) == 0 {
        return 0u8;
    }

    1u8
}

unsafe fn i8042BasicDetect(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ResendIterations: ULONG;
    let mut Value: UCHAR = 0;

    i8042Flush(DeviceExtension);
    if i8042ChangeMode(
        DeviceExtension,
        (CCB_KBD_INT_ENAB | CCB_MOUSE_INT_ENAB) as UCHAR,
        (CCB_KBD_DISAB | CCB_MOUSE_DISAB) as UCHAR,
    ) == 0
    {
        return STATUS_IO_DEVICE_ERROR;
    }

    i8042Flush(DeviceExtension);

    ResendIterations = (*DeviceExtension).Settings.ResendIterations + 1;
    while ResendIterations != 0 {
        ResendIterations -= 1;

        if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, CTRL_SELF_TEST as UCHAR) == 0 {
            return STATUS_IO_TIMEOUT;
        }

        Status = i8042ReadDataWait(DeviceExtension, &mut Value);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if Value == KBD_SELF_TEST_OK as UCHAR {
            break;
        } else if Value == KBD_RESEND as UCHAR {
            KeStallExecutionProcessor(50);
        } else {
            return STATUS_IO_DEVICE_ERROR;
        }
    }

    STATUS_SUCCESS
}

unsafe fn i8042DetectKeyboard(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
) {
    let mut Status: NTSTATUS;

    Status = i8042SynchWritePort(DeviceExtension, 0, KBD_CMD_SET_LEDS as UCHAR, 1u8);
    if NT_SUCCESS(Status) {
        Status = i8042SynchWritePort(DeviceExtension, 0, 0, 1u8);
        if !NT_SUCCESS(Status) {
            return;
        }
    }

    if i8042ChangeMode(DeviceExtension, 0, (CCB_TRANSLATE | CCB_SYSTEM_FLAG) as UCHAR) == 0 {
        return;
    }

    (*DeviceExtension).Flags |= KEYBOARD_PRESENT;
}

unsafe fn i8042DetectMouse(DeviceExtension: PPORT_DEVICE_EXTENSION) {
    let mut Status: NTSTATUS;
    let mut Value: UCHAR = 0;
    let ExpectedReply: [UCHAR; 2] = [MOUSE_ACK as UCHAR, 0xAA];
    let mut ReplyByte: UCHAR;

    'failure: {
        /* First do a mouse line test */
        if i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, MOUSE_LINE_TEST as UCHAR) != 0 {
            Status = i8042ReadDataWait(DeviceExtension, &mut Value);

            if !NT_SUCCESS(Status) || Value != 0 {
                break 'failure;
            }
        }

        /* Now reset the mouse */
        i8042Flush(DeviceExtension);

        if i8042IsrWritePort(DeviceExtension, MOU_CMD_RESET as UCHAR, CTRL_WRITE_MOUSE as UCHAR) == 0 {
            break 'failure;
        }

        ReplyByte = 0;
        while (ReplyByte as usize) < ExpectedReply.len() {
            let mut Counter: ULONG = 500;

            loop {
                Status = i8042ReadDataWait(DeviceExtension, &mut Value);

                if !NT_SUCCESS(Status) {
                    /* Wait some time before trying again */
                    KeStallExecutionProcessor(50);
                }

                if !(Status == STATUS_IO_TIMEOUT && {
                    let old = Counter;
                    Counter = Counter.wrapping_sub(1);
                    old != 0
                }) {
                    break;
                }
            }

            if !NT_SUCCESS(Status) {
                break 'failure;
            } else if Value != ExpectedReply[ReplyByte as usize] {
                break 'failure;
            }

            ReplyByte += 1;
        }

        /* Finally get the third byte, but only try it one time */
        Status = i8042ReadDataWait(DeviceExtension, &mut Value);

        if !NT_SUCCESS(Status) {
            break 'failure;
        } else if Value != 0x00 {
            break 'failure;
        }

        (*DeviceExtension).Flags |= MOUSE_PRESENT;
        return;
    }

    /* failure: */
    i8042Flush(DeviceExtension);
    i8042Write(DeviceExtension, (*DeviceExtension).ControlPort, CTRL_SELF_TEST as UCHAR);
    i8042ReadDataWait(DeviceExtension, &mut Value);
    i8042Flush(DeviceExtension);
}

unsafe fn i8042ConnectKeyboardInterrupt(
    DeviceExtension: PI8042_KEYBOARD_EXTENSION,
) -> NTSTATUS {
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let DirqlMax: KIRQL;
    let Status: NTSTATUS;

    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;

    // Enable keyboard clock line
    i8042Write(
        PortDeviceExtension,
        (*PortDeviceExtension).ControlPort,
        KBD_CLK_ENABLE as UCHAR,
    );

    DirqlMax = if (*PortDeviceExtension).KeyboardInterrupt.Dirql
        >= (*PortDeviceExtension).MouseInterrupt.Dirql
    {
        (*PortDeviceExtension).KeyboardInterrupt.Dirql
    } else {
        (*PortDeviceExtension).MouseInterrupt.Dirql
    };

    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*PortDeviceExtension).KeyboardInterrupt.Object),
        core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_KINTERRUPT, *mut c_void) -> u8>, ntkrnl_sys::ddk::PKSERVICE_ROUTINE>(Some(i8042KbdInterruptService as unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_KINTERRUPT, *mut c_void) -> u8)),
        DeviceExtension as PVOID,
        core::ptr::addr_of_mut!((*PortDeviceExtension).SpinLock),
        (*PortDeviceExtension).KeyboardInterrupt.Vector,
        (*PortDeviceExtension).KeyboardInterrupt.Dirql,
        DirqlMax,
        (*PortDeviceExtension).KeyboardInterrupt.InterruptMode,
        (*PortDeviceExtension).KeyboardInterrupt.ShareInterrupt,
        (*PortDeviceExtension).KeyboardInterrupt.Affinity,
        0u8,
    );

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[I8042PRT] kbd IoConnectInterrupt=0x%x vec=0x%x dirql=%u\n\0".as_ptr() as *const i8,
        Status,
        (*PortDeviceExtension).KeyboardInterrupt.Vector,
        (*PortDeviceExtension).KeyboardInterrupt.Dirql as u32,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if DirqlMax == (*PortDeviceExtension).KeyboardInterrupt.Dirql {
        (*PortDeviceExtension).HighestDIRQLInterrupt =
            (*PortDeviceExtension).KeyboardInterrupt.Object;
    }
    (*PortDeviceExtension).Flags |= KEYBOARD_INITIALIZED;
    STATUS_SUCCESS
}

unsafe fn i8042ConnectMouseInterrupt(
    DeviceExtension: PI8042_MOUSE_EXTENSION,
) -> NTSTATUS {
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let DirqlMax: KIRQL;
    let mut Status: NTSTATUS;

    Status = i8042MouInitialize(DeviceExtension);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    PortDeviceExtension = (*DeviceExtension).Common.PortDeviceExtension;
    DirqlMax = core::cmp::max(
        (*PortDeviceExtension).KeyboardInterrupt.Dirql,
        (*PortDeviceExtension).MouseInterrupt.Dirql,
    );

    Status = IoConnectInterrupt(
        core::ptr::addr_of_mut!((*PortDeviceExtension).MouseInterrupt.Object),
        core::mem::transmute::<Option<unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_KINTERRUPT, *mut c_void) -> u8>, ntkrnl_sys::ddk::PKSERVICE_ROUTINE>(Some(i8042MouInterruptService as unsafe extern "system" fn(*mut ntkrnl_sys::ddk::_KINTERRUPT, *mut c_void) -> u8)),
        DeviceExtension as *mut _,
        core::ptr::addr_of_mut!((*PortDeviceExtension).SpinLock),
        (*PortDeviceExtension).MouseInterrupt.Vector,
        (*PortDeviceExtension).MouseInterrupt.Dirql,
        DirqlMax,
        (*PortDeviceExtension).MouseInterrupt.InterruptMode,
        (*PortDeviceExtension).MouseInterrupt.ShareInterrupt,
        (*PortDeviceExtension).MouseInterrupt.Affinity,
        0u8,
    );

    ntkrnl_sys::driver_ddk::DbgPrint(
        b"[I8042PRT] mou IoConnectInterrupt=0x%x vec=0x%x dirql=%u\n\0".as_ptr() as *const i8,
        Status,
        (*PortDeviceExtension).MouseInterrupt.Vector,
        (*PortDeviceExtension).MouseInterrupt.Dirql as u32,
    );
    'cleanup: {
        if !NT_SUCCESS(Status) {
            break 'cleanup;
        }

        if DirqlMax == (*PortDeviceExtension).MouseInterrupt.Dirql {
            (*PortDeviceExtension).HighestDIRQLInterrupt =
                (*PortDeviceExtension).MouseInterrupt.Object;
        }

        (*PortDeviceExtension).Flags |= MOUSE_INITIALIZED;
        Status = STATUS_SUCCESS;
    }

    if !NT_SUCCESS(Status) {
        (*PortDeviceExtension).Flags &= !MOUSE_INITIALIZED;
        if !(*PortDeviceExtension).MouseInterrupt.Object.is_null() {
            IoDisconnectInterrupt((*PortDeviceExtension).MouseInterrupt.Object);
            (*PortDeviceExtension).HighestDIRQLInterrupt =
                (*PortDeviceExtension).KeyboardInterrupt.Object;
        }
    }

    Status
}

unsafe fn EnableInterrupts(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
    FlagsToDisable: UCHAR,
    FlagsToEnable: UCHAR,
) -> NTSTATUS {
    i8042Flush(DeviceExtension);

    if i8042ChangeMode(DeviceExtension, FlagsToDisable, FlagsToEnable) == 0 {
        return STATUS_UNSUCCESSFUL;
    }

    STATUS_SUCCESS
}

unsafe fn StartProcedure(
    DeviceExtension: PPORT_DEVICE_EXTENSION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;
    let mut FlagsToDisable: UCHAR = 0;
    let mut FlagsToEnable: UCHAR = 0;
    let Irql: KIRQL;

    if (*DeviceExtension).DataPort.is_null() {
        return STATUS_SUCCESS;
    }

    if ((*DeviceExtension).Flags & (KEYBOARD_PRESENT | MOUSE_PRESENT)) == 0 {
        Status = i8042BasicDetect(DeviceExtension);
        if !NT_SUCCESS(Status) {
            return STATUS_UNSUCCESSFUL;
        }

        i8042DetectMouse(DeviceExtension);
        i8042DetectKeyboard(DeviceExtension);

        if (*DeviceExtension).Flags & KEYBOARD_PRESENT != 0 {
            FlagsToDisable |= CCB_KBD_DISAB as UCHAR;
            FlagsToEnable |= CCB_KBD_INT_ENAB as UCHAR;
        }
        if (*DeviceExtension).Flags & MOUSE_PRESENT != 0 {
            FlagsToDisable |= CCB_MOUSE_DISAB as UCHAR;
            FlagsToEnable |= CCB_MOUSE_INT_ENAB as UCHAR;
        }

        Status = EnableInterrupts(DeviceExtension, FlagsToDisable, FlagsToEnable);
        if !NT_SUCCESS(Status) {
            (*DeviceExtension).Flags &= !(KEYBOARD_PRESENT | MOUSE_PRESENT);
            return Status;
        }
    }

    if (*DeviceExtension).Flags & KEYBOARD_PRESENT != 0
        && (*DeviceExtension).Flags & KEYBOARD_CONNECTED != 0
        && (*DeviceExtension).Flags & KEYBOARD_STARTED != 0
        && (*DeviceExtension).Flags & KEYBOARD_INITIALIZED == 0
    {
        Status = i8042ConnectKeyboardInterrupt((*DeviceExtension).KeyboardExtension);
        if NT_SUCCESS(Status) {
            (*DeviceExtension).Flags |= KEYBOARD_INITIALIZED;
        }
    }

    if (*DeviceExtension).Flags & MOUSE_PRESENT != 0
        && (*DeviceExtension).Flags & MOUSE_CONNECTED != 0
        && (*DeviceExtension).Flags & MOUSE_STARTED != 0
        && (*DeviceExtension).Flags & MOUSE_INITIALIZED == 0
    {
        Status = i8042ConnectMouseInterrupt((*DeviceExtension).MouseExtension);
        if NT_SUCCESS(Status) {
            (*DeviceExtension).Flags |= MOUSE_INITIALIZED;
        }

        Irql = KeAcquireInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt);
        if (i8042HwFlags & FL_INITHACK) == 0 {
            i8042IsrWritePort(DeviceExtension, MOU_CMD_RESET as UCHAR, CTRL_WRITE_MOUSE as UCHAR);
        }
        KeReleaseInterruptSpinLock((*DeviceExtension).HighestDIRQLInterrupt, Irql);
    }

    Status
}

unsafe fn i8042PnpStartDevice(
    DeviceObject: PDEVICE_OBJECT,
    AllocatedResources: PCM_RESOURCE_LIST,
    AllocatedResourcesTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let DeviceExtension: PFDO_DEVICE_EXTENSION;
    let PortDeviceExtension: PPORT_DEVICE_EXTENSION;
    let mut ResourceDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut ResourceDescriptorTranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut InterruptData: INTERRUPT_DATA = core::mem::zeroed();
    let mut FoundDataPort: BOOLEAN = 0u8;
    let mut FoundControlPort: BOOLEAN = 0u8;
    let mut FoundIrq: BOOLEAN = 0u8;
    let mut DataPort: *mut u8 = core::ptr::null_mut();
    let mut ControlPort: *mut u8 = core::ptr::null_mut();
    let mut i: ULONG;
    let Status: NTSTATUS;

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    PortDeviceExtension = (*DeviceExtension).PortDeviceExtension;

    if AllocatedResources.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if (*AllocatedResources).Count != 1 {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    if (*AllocatedResources).List[0].PartialResourceList.Version != 1
        || (*AllocatedResources).List[0].PartialResourceList.Revision != 1
        || (*AllocatedResourcesTranslated).List[0].PartialResourceList.Version != 1
        || (*AllocatedResourcesTranslated).List[0].PartialResourceList.Revision != 1
    {
        return STATUS_REVISION_MISMATCH;
    }

    i = 0;
    while i < (*AllocatedResources).List[0].PartialResourceList.Count {
        // PartialDescriptors is a C ANYSIZE_ARRAY (bindgen renders it as `[T; 1]`).
        // Indexing `[i]` past 0 triggers Rust's bounds-check panic; the C source
        // treats it as a flexible array. Use pointer arithmetic to match C.
        ResourceDescriptor = (*AllocatedResources)
            .List[0]
            .PartialResourceList
            .PartialDescriptors
            .as_mut_ptr()
            .add(i as usize);
        ResourceDescriptorTranslated = (*AllocatedResourcesTranslated)
            .List[0]
            .PartialResourceList
            .PartialDescriptors
            .as_mut_ptr()
            .add(i as usize);
        match (*ResourceDescriptor).Type as u32 {
            CmResourceTypePort => {
                if (*ResourceDescriptor).u.Port.Length == 1 {
                    if FoundDataPort == 0 {
                        DataPort = ULongToPtr((*ResourceDescriptor).u.Port.Start.u.LowPart) as _;
                        FoundDataPort = 1u8;
                    } else if FoundControlPort == 0 {
                        ControlPort = ULongToPtr((*ResourceDescriptor).u.Port.Start.u.LowPart) as _;
                        FoundControlPort = 1u8;
                    } else {
                        // FIXME: implement PS/2 Active Multiplexing
                    }
                }
            }
            CmResourceTypeInterrupt => {
                if FoundIrq != 0 {
                    return STATUS_INVALID_PARAMETER;
                }
                InterruptData.Dirql = (*ResourceDescriptorTranslated).u.Interrupt.Level as KIRQL;
                InterruptData.Vector = (*ResourceDescriptorTranslated).u.Interrupt.Vector;
                InterruptData.Affinity = (*ResourceDescriptorTranslated).u.Interrupt.Affinity;
                if ((*ResourceDescriptorTranslated).Flags as u32 & CM_RESOURCE_INTERRUPT_LATCHED as u32) != 0 {
                    InterruptData.InterruptMode = Latched;
                } else {
                    InterruptData.InterruptMode = LevelSensitive;
                }
                InterruptData.ShareInterrupt =
                    if (*ResourceDescriptorTranslated).ShareDisposition as i32 == CmResourceShareShared as i32 {
                        1u8
                    } else {
                        0u8
                    };
                FoundIrq = 1u8;
            }
            _ => {}
        }
        i += 1;
    }

    if FoundIrq == 0 {
        return STATUS_INSUFFICIENT_RESOURCES;
    } else if (*DeviceExtension).Type == Keyboard
        && (FoundDataPort == 0 || FoundControlPort == 0)
    {
        return STATUS_INSUFFICIENT_RESOURCES;
    } else if (*DeviceExtension).Type == Mouse && FoundDataPort != FoundControlPort {
        // Mouse was given only one of the controller ports
        return STATUS_INVALID_PARAMETER;
    }

    /* Firmware gives the controller ports to the mouse when it reports no keyboard */
    if FoundDataPort != 0
        && FoundControlPort != 0
        && ((*DeviceExtension).Type == Keyboard || (*PortDeviceExtension).DataPort.is_null())
    {
        (*PortDeviceExtension).DataPort = DataPort;
        (*PortDeviceExtension).ControlPort = ControlPort;
    }

    match (*DeviceExtension).Type {
        Keyboard => {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*PortDeviceExtension).KeyboardInterrupt) as _,
                core::ptr::addr_of!(InterruptData) as _,
                core::mem::size_of::<INTERRUPT_DATA>() as _,
            );
            (*PortDeviceExtension).Flags |= KEYBOARD_STARTED;
            Status = StartProcedure(PortDeviceExtension);
        }
        Mouse => {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*PortDeviceExtension).MouseInterrupt) as _,
                core::ptr::addr_of!(InterruptData) as _,
                core::mem::size_of::<INTERRUPT_DATA>() as _,
            );
            (*PortDeviceExtension).Flags |= MOUSE_STARTED;
            Status = StartProcedure(PortDeviceExtension);
        }
        _ => {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    if NT_SUCCESS(Status) {
        (*DeviceExtension).PnpState = dsStarted;
    }

    // Delivery-chain boot marker (one line per device): pairs with
    // [KBDCLASS] first key delivered / [I8042] first kbd IRQ so a smoke log
    // shows exactly where the input chain breaks.
    DbgPrint(
        b"[I8042] StartDevice type=%d -> 0x%08x (irq vec=%lu dirql=%u portflags=0x%lx)\n\0"
            .as_ptr() as *const i8,
        (*DeviceExtension).Type as u32,
        Status as u32,
        InterruptData.Vector,
        InterruptData.Dirql as u32,
        (*PortDeviceExtension).Flags,
    );

    Status
}

unsafe fn i8042RemoveDevice(DeviceObject: PDEVICE_OBJECT) {
    let DriverExtension = IoGetDriverObjectExtension(
        (*DeviceObject).DriverObject as *mut _,
        (*DeviceObject).DriverObject as *mut _,
    ) as PI8042_DRIVER_EXTENSION;
    let DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    let mut OldIrql: KIRQL = 0;
    KeAcquireSpinLock(
        core::ptr::addr_of_mut!((*DriverExtension).DeviceListLock) as *mut _,
        &mut OldIrql,
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*DeviceExtension).ListEntry));
    KeReleaseSpinLock(
        core::ptr::addr_of_mut!((*DriverExtension).DeviceListLock) as *mut _,
        OldIrql,
    );

    IoDetachDevice((*DeviceExtension).LowerDevice);

    IoDeleteDevice(DeviceObject);
}

#[no_mangle]
pub unsafe extern "system" fn i8042Pnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let FdoExtension: PFDO_DEVICE_EXTENSION;
    let Stack: PIO_STACK_LOCATION;
    // IRP_MN_* constants are UCHAR; keep MinorFunction the same width so the
    // match arms (which name those UCHAR consts) type-check.
    let MinorFunction: UCHAR;
    let DeviceType: I8042_DEVICE_TYPE;
    let Information: ULONG_PTR = 0;
    let mut Status: NTSTATUS;

    Stack = IoGetCurrentIrpStackLocation(Irp);
    MinorFunction = (*Stack).MinorFunction;
    FdoExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;
    DeviceType = (*FdoExtension).Type;

    match MinorFunction {
        IRP_MN_START_DEVICE => {
            if DeviceType != PhysicalDeviceObject {
                Status = STATUS_UNSUCCESSFUL;

                if IoForwardIrpSynchronously((*FdoExtension).LowerDevice, Irp) != 0 {
                    Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                    if NT_SUCCESS(Status) {
                        Status = i8042PnpStartDevice(
                            DeviceObject,
                            (*Stack).Parameters.StartDevice.AllocatedResources,
                            (*Stack).Parameters.StartDevice.AllocatedResourcesTranslated,
                        );
                    }
                }
            } else {
                Status = STATUS_SUCCESS;
            }
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            match (*Stack).Parameters.QueryDeviceRelations.Type {
                BusRelations => {
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
                RemovalRelations => {
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
                _ => {
                    return ForwardIrpAndForget(DeviceObject, Irp);
                }
            }
        }
        IRP_MN_QUERY_CAPABILITIES => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_QUERY_PNP_DEVICE_STATE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_QUERY_REMOVE_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            Status = ForwardIrpAndForget(DeviceObject, Irp);
            i8042RemoveDevice(DeviceObject);
            return Status;
        }
        _ => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information as _;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS i8042 (ps/2 keyboard-mouse controller) driver
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        drivers/input/i8042prt/pnp.c
//  * PURPOSE:     IRP_MJ_PNP operations
//  * PROGRAMMERS: Copyright 2006-2007 Herv� Poussineau (hpoussin@reactos.org)
//  *              Copyright 2008 Colin Finck (mail@colinfinck.de)
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
// /* This is all pretty confusing. There's more than one way to
//  * disable/enable the keyboard. You can send KBD_ENABLE to the
//  * keyboard, and it will start scanning keys. Sending KBD_DISABLE
//  * will disable the key scanning but also reset the parameters to
//  * defaults.
//  *
//  * You can also send 0xAE to the controller for enabling the
//  * keyboard clock line and 0xAD for disabling it. Then it'll
//  * automatically get turned on at the next command. The last
//  * way is by modifying the bit that drives the clock line in the
//  * 'command byte' of the controller. This is almost, but not quite,
//  * the same as the AE/AD thing. The difference can be used to detect
//  * some really old broken keyboard controllers which I hope won't be
//  * necessary.
//  *
//  * We change the command byte, sending KBD_ENABLE/DISABLE seems to confuse
//  * some kvm switches.
//  */
// BOOLEAN
// i8042ChangeMode(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension,
//     IN UCHAR FlagsToDisable,
//     IN UCHAR FlagsToEnable)
// {
//     UCHAR Value;
//     NTSTATUS Status;
//
//     if (!i8042Write(DeviceExtension, DeviceExtension->ControlPort, KBD_READ_MODE))
//     {
//         WARN_(I8042PRT, "Can't read i8042 mode\n");
//         return FALSE;
//     }
//
//     Status = i8042ReadDataWait(DeviceExtension, &Value);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN_(I8042PRT, "No response after read i8042 mode\n");
//         return FALSE;
//     }
//
//     Value &= ~FlagsToDisable;
//     Value |= FlagsToEnable;
//
//     if (!i8042Write(DeviceExtension, DeviceExtension->ControlPort, KBD_WRITE_MODE))
//     {
//         WARN_(I8042PRT, "Can't set i8042 mode\n");
//         return FALSE;
//     }
//
//     if (!i8042Write(DeviceExtension, DeviceExtension->DataPort, Value))
//     {
//         WARN_(I8042PRT, "Can't send i8042 mode\n");
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// static NTSTATUS
// i8042BasicDetect(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     ULONG ResendIterations;
//     UCHAR Value = 0;
//
//     /* Don't enable keyboard and mouse interrupts, disable keyboard/mouse */
//     i8042Flush(DeviceExtension);
//     if (!i8042ChangeMode(DeviceExtension, CCB_KBD_INT_ENAB | CCB_MOUSE_INT_ENAB, CCB_KBD_DISAB | CCB_MOUSE_DISAB))
//         return STATUS_IO_DEVICE_ERROR;
//
//     i8042Flush(DeviceExtension);
//
//     /* Issue a CTRL_SELF_TEST command to check if this is really an i8042 controller */
//     ResendIterations = DeviceExtension->Settings.ResendIterations + 1;
//     while (ResendIterations--)
//     {
//         if (!i8042Write(DeviceExtension, DeviceExtension->ControlPort, CTRL_SELF_TEST))
//         {
//             WARN_(I8042PRT, "Writing CTRL_SELF_TEST command failed\n");
//             return STATUS_IO_TIMEOUT;
//         }
//
//         Status = i8042ReadDataWait(DeviceExtension, &Value);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN_(I8042PRT, "Failed to read CTRL_SELF_TEST response, status 0x%08lx\n", Status);
//             return Status;
//         }
//
//         if (Value == KBD_SELF_TEST_OK)
//         {
//             INFO_(I8042PRT, "CTRL_SELF_TEST completed successfully!\n");
//             break;
//         }
//         else if (Value == KBD_RESEND)
//         {
//             TRACE_(I8042PRT, "Resending...\n");
//             KeStallExecutionProcessor(50);
//         }
//         else
//         {
//             WARN_(I8042PRT, "Got 0x%02x instead of 0x55\n", Value);
//             return STATUS_IO_DEVICE_ERROR;
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// static VOID
// i8042DetectKeyboard(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//
//     /* Set LEDs (that is not fatal if some error occurs) */
//     Status = i8042SynchWritePort(DeviceExtension, 0, KBD_CMD_SET_LEDS, TRUE);
//     if (NT_SUCCESS(Status))
//     {
//         Status = i8042SynchWritePort(DeviceExtension, 0, 0, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN_(I8042PRT, "Can't finish SET_LEDS (0x%08lx)\n", Status);
//             return;
//         }
//     }
//     else
//     {
//         WARN_(I8042PRT, "Warning: can't write SET_LEDS (0x%08lx)\n", Status);
//     }
//
//     /* Turn on translation and SF (Some machines don't reboot if SF is not set, see ReactOS bug CORE-1713) */
//     if (!i8042ChangeMode(DeviceExtension, 0, CCB_TRANSLATE | CCB_SYSTEM_FLAG))
//         return;
//
//     /*
//      * We used to send a KBD_LINE_TEST (0xAB) command, but on at least HP
//      * Pavilion notebooks the response to that command was incorrect.
//      * So now we just assume that a keyboard is attached.
//      */
//     DeviceExtension->Flags |= KEYBOARD_PRESENT;
//
//     INFO_(I8042PRT, "Keyboard detected\n");
// }
//
// static VOID
// i8042DetectMouse(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status;
//     UCHAR Value;
//     UCHAR ExpectedReply[] = { MOUSE_ACK, 0xAA };
//     UCHAR ReplyByte;
//
//     /* First do a mouse line test */
//     if (i8042Write(DeviceExtension, DeviceExtension->ControlPort, MOUSE_LINE_TEST))
//     {
//         Status = i8042ReadDataWait(DeviceExtension, &Value);
//
//         if (!NT_SUCCESS(Status) || Value != 0)
//         {
//             WARN_(I8042PRT, "Mouse line test failed\n");
//             goto failure;
//         }
//     }
//
//     /* Now reset the mouse */
//     i8042Flush(DeviceExtension);
//
//     if(!i8042IsrWritePort(DeviceExtension, MOU_CMD_RESET, CTRL_WRITE_MOUSE))
//     {
//         WARN_(I8042PRT, "Failed to write reset command to mouse\n");
//         goto failure;
//     }
//
//     /* The implementation of the "Mouse Reset" command differs much from chip to chip.
//
//        By default, the first byte is an ACK, when the mouse is plugged in and working and NACK when it's not.
//        On success, the next bytes are 0xAA and 0x00.
//
//        But on some systems (like ECS K7S5A Pro, SiS 735 chipset), we always get an ACK and 0xAA.
//        Only the last byte indicates, whether a mouse is plugged in.
//        It is either sent or not, so there is no byte, which indicates a failure here.
//
//        After the Mouse Reset command was issued, it usually takes some time until we get a response.
//        So get the first two bytes in a loop. */
//     for (ReplyByte = 0;
//          ReplyByte < sizeof(ExpectedReply) / sizeof(ExpectedReply[0]);
//          ReplyByte++)
//     {
//         ULONG Counter = 500;
//
//         do
//         {
//             Status = i8042ReadDataWait(DeviceExtension, &Value);
//
//             if(!NT_SUCCESS(Status))
//             {
//                 /* Wait some time before trying again */
//                 KeStallExecutionProcessor(50);
//             }
//         } while (Status == STATUS_IO_TIMEOUT && Counter--);
//
//         if (!NT_SUCCESS(Status))
//         {
//             WARN_(I8042PRT, "No ACK after mouse reset, status 0x%08lx\n", Status);
//             goto failure;
//         }
//         else if (Value != ExpectedReply[ReplyByte])
//         {
//             WARN_(I8042PRT, "Unexpected reply: 0x%02x (expected 0x%02x)\n", Value, ExpectedReply[ReplyByte]);
//             goto failure;
//         }
//     }
//
//     /* Finally get the third byte, but only try it one time (see above).
//        Otherwise this takes around 45 seconds on a K7S5A Pro, when no mouse is plugged in. */
//     Status = i8042ReadDataWait(DeviceExtension, &Value);
//
//     if(!NT_SUCCESS(Status))
//     {
//         WARN_(I8042PRT, "Last byte was not transmitted after mouse reset, status 0x%08lx\n", Status);
//         goto failure;
//     }
//     else if(Value != 0x00)
//     {
//         WARN_(I8042PRT, "Last byte after mouse reset was not 0x00, but 0x%02x\n", Value);
//         goto failure;
//     }
//
//     DeviceExtension->Flags |= MOUSE_PRESENT;
//     INFO_(I8042PRT, "Mouse detected\n");
//     return;
//
// failure:
//     /* There is probably no mouse present. On some systems,
//        the probe locks the entire keyboard controller. Let's
//        try to get access to the keyboard again by sending a
//        reset */
//     i8042Flush(DeviceExtension);
//     i8042Write(DeviceExtension, DeviceExtension->ControlPort, CTRL_SELF_TEST);
//     i8042ReadDataWait(DeviceExtension, &Value);
//     i8042Flush(DeviceExtension);
//
//     INFO_(I8042PRT, "Mouse not detected\n");
// }
//
// static NTSTATUS
// i8042ConnectKeyboardInterrupt(
//     IN PI8042_KEYBOARD_EXTENSION DeviceExtension)
// {
//     PPORT_DEVICE_EXTENSION PortDeviceExtension;
//     KIRQL DirqlMax;
//     NTSTATUS Status;
//
//     TRACE_(I8042PRT, "i8042ConnectKeyboardInterrupt()\n");
//
//     PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//
//     // Enable keyboard clock line
//     i8042Write(PortDeviceExtension, PortDeviceExtension->ControlPort, KBD_CLK_ENABLE);
//
//     DirqlMax = MAX(
//         PortDeviceExtension->KeyboardInterrupt.Dirql,
//         PortDeviceExtension->MouseInterrupt.Dirql);
//
//     INFO_(I8042PRT, "KeyboardInterrupt.Vector         %lu\n",
//         PortDeviceExtension->KeyboardInterrupt.Vector);
//     INFO_(I8042PRT, "KeyboardInterrupt.Dirql          %lu\n",
//         PortDeviceExtension->KeyboardInterrupt.Dirql);
//     INFO_(I8042PRT, "KeyboardInterrupt.DirqlMax       %lu\n",
//         DirqlMax);
//     INFO_(I8042PRT, "KeyboardInterrupt.InterruptMode  %s\n",
//         PortDeviceExtension->KeyboardInterrupt.InterruptMode == LevelSensitive ? "LevelSensitive" : "Latched");
//     INFO_(I8042PRT, "KeyboardInterrupt.ShareInterrupt %s\n",
//         PortDeviceExtension->KeyboardInterrupt.ShareInterrupt ? "yes" : "no");
//     INFO_(I8042PRT, "KeyboardInterrupt.Affinity       0x%lx\n",
//         PortDeviceExtension->KeyboardInterrupt.Affinity);
//     Status = IoConnectInterrupt(
//         &PortDeviceExtension->KeyboardInterrupt.Object,
//         i8042KbdInterruptService,
//         DeviceExtension, &PortDeviceExtension->SpinLock,
//         PortDeviceExtension->KeyboardInterrupt.Vector, PortDeviceExtension->KeyboardInterrupt.Dirql, DirqlMax,
//         PortDeviceExtension->KeyboardInterrupt.InterruptMode, PortDeviceExtension->KeyboardInterrupt.ShareInterrupt,
//         PortDeviceExtension->KeyboardInterrupt.Affinity, FALSE);
//     DPRINT1("[[I8K]] KBD IoConnectInterrupt -> 0x%08x (Vec=%lu Dirql=%u Obj=%p)\n",
//         Status, PortDeviceExtension->KeyboardInterrupt.Vector,
//         PortDeviceExtension->KeyboardInterrupt.Dirql,
//         PortDeviceExtension->KeyboardInterrupt.Object);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN_(I8042PRT, "IoConnectInterrupt() failed with status 0x%08x\n", Status);
//         return Status;
//     }
//
//     if (DirqlMax == PortDeviceExtension->KeyboardInterrupt.Dirql)
//         PortDeviceExtension->HighestDIRQLInterrupt = PortDeviceExtension->KeyboardInterrupt.Object;
//     PortDeviceExtension->Flags |= KEYBOARD_INITIALIZED;
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// i8042ConnectMouseInterrupt(
//     IN PI8042_MOUSE_EXTENSION DeviceExtension)
// {
//     PPORT_DEVICE_EXTENSION PortDeviceExtension;
//     KIRQL DirqlMax;
//     NTSTATUS Status;
//
//     TRACE_(I8042PRT, "i8042ConnectMouseInterrupt()\n");
//
//     Status = i8042MouInitialize(DeviceExtension);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     PortDeviceExtension = DeviceExtension->Common.PortDeviceExtension;
//     DirqlMax = MAX(
//         PortDeviceExtension->KeyboardInterrupt.Dirql,
//         PortDeviceExtension->MouseInterrupt.Dirql);
//
//     INFO_(I8042PRT, "MouseInterrupt.Vector         %lu\n",
//         PortDeviceExtension->MouseInterrupt.Vector);
//     INFO_(I8042PRT, "MouseInterrupt.Dirql          %lu\n",
//         PortDeviceExtension->MouseInterrupt.Dirql);
//     INFO_(I8042PRT, "MouseInterrupt.DirqlMax       %lu\n",
//         DirqlMax);
//     INFO_(I8042PRT, "MouseInterrupt.InterruptMode  %s\n",
//         PortDeviceExtension->MouseInterrupt.InterruptMode == LevelSensitive ? "LevelSensitive" : "Latched");
//     INFO_(I8042PRT, "MouseInterrupt.ShareInterrupt %s\n",
//         PortDeviceExtension->MouseInterrupt.ShareInterrupt ? "yes" : "no");
//     INFO_(I8042PRT, "MouseInterrupt.Affinity       0x%lx\n",
//         PortDeviceExtension->MouseInterrupt.Affinity);
//     Status = IoConnectInterrupt(
//         &PortDeviceExtension->MouseInterrupt.Object,
//         i8042MouInterruptService,
//         DeviceExtension, &PortDeviceExtension->SpinLock,
//         PortDeviceExtension->MouseInterrupt.Vector, PortDeviceExtension->MouseInterrupt.Dirql, DirqlMax,
//         PortDeviceExtension->MouseInterrupt.InterruptMode, PortDeviceExtension->MouseInterrupt.ShareInterrupt,
//         PortDeviceExtension->MouseInterrupt.Affinity, FALSE);
//     DPRINT1("[[I8K]] MOU IoConnectInterrupt -> 0x%08x (Vec=%lu Dirql=%u Obj=%p)\n",
//         Status, PortDeviceExtension->MouseInterrupt.Vector,
//         PortDeviceExtension->MouseInterrupt.Dirql,
//         PortDeviceExtension->MouseInterrupt.Object);
//     if (!NT_SUCCESS(Status))
//     {
//         WARN_(I8042PRT, "IoConnectInterrupt() failed with status 0x%08x\n", Status);
//         goto cleanup;
//     }
//
//     if (DirqlMax == PortDeviceExtension->MouseInterrupt.Dirql)
//         PortDeviceExtension->HighestDIRQLInterrupt = PortDeviceExtension->MouseInterrupt.Object;
//
//     PortDeviceExtension->Flags |= MOUSE_INITIALIZED;
//     Status = STATUS_SUCCESS;
//
// cleanup:
//     if (!NT_SUCCESS(Status))
//     {
//         PortDeviceExtension->Flags &= ~MOUSE_INITIALIZED;
//         if (PortDeviceExtension->MouseInterrupt.Object)
//         {
//             IoDisconnectInterrupt(PortDeviceExtension->MouseInterrupt.Object);
//             PortDeviceExtension->HighestDIRQLInterrupt = PortDeviceExtension->KeyboardInterrupt.Object;
//         }
//     }
//     return Status;
// }
//
// static NTSTATUS
// EnableInterrupts(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension,
//     IN UCHAR FlagsToDisable,
//     IN UCHAR FlagsToEnable)
// {
//     i8042Flush(DeviceExtension);
//
//     if (!i8042ChangeMode(DeviceExtension, FlagsToDisable, FlagsToEnable))
//         return STATUS_UNSUCCESSFUL;
//
//     return STATUS_SUCCESS;
// }
//
// static NTSTATUS
// StartProcedure(
//     IN PPORT_DEVICE_EXTENSION DeviceExtension)
// {
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     UCHAR FlagsToDisable = 0;
//     UCHAR FlagsToEnable = 0;
//     KIRQL Irql;
//
//     if (DeviceExtension->DataPort == 0)
//     {
//         /* Unable to do something at the moment */
//         return STATUS_SUCCESS;
//     }
//
//     if (!(DeviceExtension->Flags & (KEYBOARD_PRESENT | MOUSE_PRESENT)))
//     {
//         /* Try to detect them */
//         TRACE_(I8042PRT, "Check if the controller is really a i8042\n");
//         Status = i8042BasicDetect(DeviceExtension);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN_(I8042PRT, "i8042BasicDetect() failed with status 0x%08lx\n", Status);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* First detect the mouse and then the keyboard!
//            If we do it the other way round, some systems throw away settings like the keyboard translation, when detecting the mouse. */
//         TRACE_(I8042PRT, "Detecting mouse\n");
//         i8042DetectMouse(DeviceExtension);
//         TRACE_(I8042PRT, "Detecting keyboard\n");
//         i8042DetectKeyboard(DeviceExtension);
//
//         INFO_(I8042PRT, "Keyboard present: %s\n", DeviceExtension->Flags & KEYBOARD_PRESENT ? "YES" : "NO");
//         INFO_(I8042PRT, "Mouse present   : %s\n", DeviceExtension->Flags & MOUSE_PRESENT ? "YES" : "NO");
//
//         TRACE_(I8042PRT, "Enabling i8042 interrupts\n");
//         if (DeviceExtension->Flags & KEYBOARD_PRESENT)
//         {
//             FlagsToDisable |= CCB_KBD_DISAB;
//             FlagsToEnable |= CCB_KBD_INT_ENAB;
//         }
//         if (DeviceExtension->Flags & MOUSE_PRESENT)
//         {
//             FlagsToDisable |= CCB_MOUSE_DISAB;
//             FlagsToEnable |= CCB_MOUSE_INT_ENAB;
//         }
//
//         Status = EnableInterrupts(DeviceExtension, FlagsToDisable, FlagsToEnable);
//         if (!NT_SUCCESS(Status))
//         {
//             WARN_(I8042PRT, "EnableInterrupts failed: %lx\n", Status);
//             DeviceExtension->Flags &= ~(KEYBOARD_PRESENT | MOUSE_PRESENT);
//             return Status;
//         }
//     }
//
//     /* Connect interrupts */
//     if (DeviceExtension->Flags & KEYBOARD_PRESENT &&
//         DeviceExtension->Flags & KEYBOARD_CONNECTED &&
//         DeviceExtension->Flags & KEYBOARD_STARTED &&
//         !(DeviceExtension->Flags & KEYBOARD_INITIALIZED))
//     {
//         /* Keyboard is ready to be initialized */
//         Status = i8042ConnectKeyboardInterrupt(DeviceExtension->KeyboardExtension);
//         if (NT_SUCCESS(Status))
//         {
//             DeviceExtension->Flags |= KEYBOARD_INITIALIZED;
//         }
//         else
//         {
//             WARN_(I8042PRT, "i8042ConnectKeyboardInterrupt failed: %lx\n", Status);
//         }
//     }
//
//     if (DeviceExtension->Flags & MOUSE_PRESENT &&
//         DeviceExtension->Flags & MOUSE_CONNECTED &&
//         DeviceExtension->Flags & MOUSE_STARTED &&
//         !(DeviceExtension->Flags & MOUSE_INITIALIZED))
//     {
//         /* Mouse is ready to be initialized */
//         Status = i8042ConnectMouseInterrupt(DeviceExtension->MouseExtension);
//         if (NT_SUCCESS(Status))
//         {
//             DeviceExtension->Flags |= MOUSE_INITIALIZED;
//         }
//         else
//         {
//             WARN_(I8042PRT, "i8042ConnectMouseInterrupt failed: %lx\n", Status);
//         }
//
//         /* Start the mouse */
//         Irql = KeAcquireInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt);
//         /* HACK: the mouse has already been reset in i8042DetectMouse. This second
//            reset prevents some touchpads/mice from working (Dell D531, D600).
//            See CORE-6901 */
//         if (!(i8042HwFlags & FL_INITHACK))
//         {
//             i8042IsrWritePort(DeviceExtension, MOU_CMD_RESET, CTRL_WRITE_MOUSE);
//         }
//         KeReleaseInterruptSpinLock(DeviceExtension->HighestDIRQLInterrupt, Irql);
//     }
//
//     return Status;
// }
//
// static NTSTATUS
// i8042PnpStartDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PCM_RESOURCE_LIST AllocatedResources,
//     IN PCM_RESOURCE_LIST AllocatedResourcesTranslated)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     PPORT_DEVICE_EXTENSION PortDeviceExtension;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR ResourceDescriptor, ResourceDescriptorTranslated;
//     INTERRUPT_DATA InterruptData = { NULL };
//     BOOLEAN FoundDataPort = FALSE;
//     BOOLEAN FoundControlPort = FALSE;
//     BOOLEAN FoundIrq = FALSE;
//     ULONG i;
//     NTSTATUS Status;
//
//     TRACE_(I8042PRT, "i8042PnpStartDevice(%p)\n", DeviceObject);
//     DPRINT1("[[I8K]] StartDevice ENTER Dev=%p AllocRes=%p\n", DeviceObject, AllocatedResources);
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     PortDeviceExtension = DeviceExtension->PortDeviceExtension;
//
//     ASSERT(DeviceExtension->PnpState == dsStopped);
//
//     if (!AllocatedResources)
//     {
//         WARN_(I8042PRT, "No allocated resources sent to driver\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     if (AllocatedResources->Count != 1)
//     {
//         WARN_(I8042PRT, "Wrong number of allocated resources sent to driver\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     if (AllocatedResources->List[0].PartialResourceList.Version != 1
//      || AllocatedResources->List[0].PartialResourceList.Revision != 1
//      || AllocatedResourcesTranslated->List[0].PartialResourceList.Version != 1
//      || AllocatedResourcesTranslated->List[0].PartialResourceList.Revision != 1)
//     {
//         WARN_(I8042PRT, "Revision mismatch: %u.%u != 1.1 or %u.%u != 1.1\n",
//             AllocatedResources->List[0].PartialResourceList.Version,
//             AllocatedResources->List[0].PartialResourceList.Revision,
//             AllocatedResourcesTranslated->List[0].PartialResourceList.Version,
//             AllocatedResourcesTranslated->List[0].PartialResourceList.Revision);
//         return STATUS_REVISION_MISMATCH;
//     }
//
//     /* Get Irq and optionally control port and data port */
//     for (i = 0; i < AllocatedResources->List[0].PartialResourceList.Count; i++)
//     {
//         ResourceDescriptor = &AllocatedResources->List[0].PartialResourceList.PartialDescriptors[i];
//         ResourceDescriptorTranslated = &AllocatedResourcesTranslated->List[0].PartialResourceList.PartialDescriptors[i];
//         switch (ResourceDescriptor->Type)
//         {
//             case CmResourceTypePort:
//             {
//                 if (ResourceDescriptor->u.Port.Length == 1)
//                 {
//                     /* We assume that the first resource will
//                      * be the control port and the second one
//                      * will be the data port...
//                      */
//                     if (!FoundDataPort)
//                     {
//                         PortDeviceExtension->DataPort = ULongToPtr(ResourceDescriptor->u.Port.Start.u.LowPart);
//                         INFO_(I8042PRT, "Found data port: %p\n", PortDeviceExtension->DataPort);
//                         FoundDataPort = TRUE;
//                     }
//                     else if (!FoundControlPort)
//                     {
//                         PortDeviceExtension->ControlPort = ULongToPtr(ResourceDescriptor->u.Port.Start.u.LowPart);
//                         INFO_(I8042PRT, "Found control port: %p\n", PortDeviceExtension->ControlPort);
//                         FoundControlPort = TRUE;
//                     }
//                     else
//                     {
//                         /* FIXME: implement PS/2 Active Multiplexing */
//                         ERR_(I8042PRT, "Unhandled I/O ranges provided: 0x%lx\n", ResourceDescriptor->u.Port.Length);
//                     }
//                 }
//                 else
//                     WARN_(I8042PRT, "Invalid I/O range length: 0x%lx\n", ResourceDescriptor->u.Port.Length);
//                 break;
//             }
//             case CmResourceTypeInterrupt:
//             {
//                 if (FoundIrq)
//                     return STATUS_INVALID_PARAMETER;
//                 InterruptData.Dirql = (KIRQL)ResourceDescriptorTranslated->u.Interrupt.Level;
//                 InterruptData.Vector = ResourceDescriptorTranslated->u.Interrupt.Vector;
//                 InterruptData.Affinity = ResourceDescriptorTranslated->u.Interrupt.Affinity;
//                 if (ResourceDescriptorTranslated->Flags & CM_RESOURCE_INTERRUPT_LATCHED)
//                     InterruptData.InterruptMode = Latched;
//                 else
//                     InterruptData.InterruptMode = LevelSensitive;
//                 InterruptData.ShareInterrupt = (ResourceDescriptorTranslated->ShareDisposition == CmResourceShareShared);
//                 INFO_(I8042PRT, "Found irq resource: %lu\n", ResourceDescriptor->u.Interrupt.Level);
//                 FoundIrq = TRUE;
//                 break;
//             }
//             default:
//                 WARN_(I8042PRT, "Unknown resource descriptor type 0x%x\n", ResourceDescriptor->Type);
//         }
//     }
//
//     DPRINT1("[[I8K]] StartDevice res: Type=%d FoundIrq=%d Vec=%lu Dirql=%u Data=%d Ctrl=%d\n",
//         DeviceExtension->Type, FoundIrq, InterruptData.Vector, InterruptData.Dirql,
//         FoundDataPort, FoundControlPort);
//
//     if (!FoundIrq)
//     {
//         WARN_(I8042PRT, "Interrupt resource was not found in allocated resources list\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else if (DeviceExtension->Type == Keyboard && (!FoundDataPort || !FoundControlPort))
//     {
//         WARN_(I8042PRT, "Some required resources were not found in allocated resources list\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     else if (DeviceExtension->Type == Mouse && (FoundDataPort || FoundControlPort))
//     {
//         WARN_(I8042PRT, "Too much resources were provided in allocated resources list\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     switch (DeviceExtension->Type)
//     {
//         case Keyboard:
//         {
//             RtlCopyMemory(
//                 &PortDeviceExtension->KeyboardInterrupt,
//                 &InterruptData,
//                 sizeof(INTERRUPT_DATA));
//             PortDeviceExtension->Flags |= KEYBOARD_STARTED;
//             Status = StartProcedure(PortDeviceExtension);
//             break;
//         }
//         case Mouse:
//         {
//             RtlCopyMemory(
//                 &PortDeviceExtension->MouseInterrupt,
//                 &InterruptData,
//                 sizeof(INTERRUPT_DATA));
//             PortDeviceExtension->Flags |= MOUSE_STARTED;
//             Status = StartProcedure(PortDeviceExtension);
//             break;
//         }
//         default:
//         {
//             WARN_(I8042PRT, "Unknown FDO type %u\n", DeviceExtension->Type);
//             ASSERT(!(PortDeviceExtension->Flags & KEYBOARD_CONNECTED) || !(PortDeviceExtension->Flags & MOUSE_CONNECTED));
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//     }
//
//     if (NT_SUCCESS(Status))
//         DeviceExtension->PnpState = dsStarted;
//
//     return Status;
// }
//
// static VOID
// i8042RemoveDevice(
//     IN PDEVICE_OBJECT DeviceObject)
// {
//     PI8042_DRIVER_EXTENSION DriverExtension;
//     KIRQL OldIrql;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//
//     DriverExtension = (PI8042_DRIVER_EXTENSION)IoGetDriverObjectExtension(DeviceObject->DriverObject, DeviceObject->DriverObject);
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     KeAcquireSpinLock(&DriverExtension->DeviceListLock, &OldIrql);
//     RemoveEntryList(&DeviceExtension->ListEntry);
//     KeReleaseSpinLock(&DriverExtension->DeviceListLock, OldIrql);
//
//     IoDetachDevice(DeviceExtension->LowerDevice);
//
//     IoDeleteDevice(DeviceObject);
// }
//
// NTSTATUS NTAPI
// i8042Pnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION FdoExtension;
//     PIO_STACK_LOCATION Stack;
//     ULONG MinorFunction;
//     I8042_DEVICE_TYPE DeviceType;
//     ULONG_PTR Information = 0;
//     NTSTATUS Status;
//
//     Stack = IoGetCurrentIrpStackLocation(Irp);
//     MinorFunction = Stack->MinorFunction;
//     FdoExtension = DeviceObject->DeviceExtension;
//     DeviceType = FdoExtension->Type;
//
//     switch (MinorFunction)
//     {
//         case IRP_MN_START_DEVICE: /* 0x00 */
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_START_DEVICE\n");
//             DPRINT1("[[I8K]] Pnp START_DEVICE DeviceType=%d\n", DeviceType);
//
//             /* Call lower driver (if any) */
//             if (DeviceType != PhysicalDeviceObject)
//             {
//                 Status = STATUS_UNSUCCESSFUL;
//
//                 if (IoForwardIrpSynchronously(FdoExtension->LowerDevice, Irp))
//                 {
//                     Status = Irp->IoStatus.Status;
//                     if (NT_SUCCESS(Status))
//                     {
//                         Status = i8042PnpStartDevice(
//                             DeviceObject,
//                             Stack->Parameters.StartDevice.AllocatedResources,
//                             Stack->Parameters.StartDevice.AllocatedResourcesTranslated);
//                     }
//                 }
//             }
//             else
//                 Status = STATUS_SUCCESS;
//             break;
//         }
//         case IRP_MN_QUERY_DEVICE_RELATIONS: /* (optional) 0x07 */
//         {
//             switch (Stack->Parameters.QueryDeviceRelations.Type)
//             {
//                 case BusRelations:
//                 {
//                     TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n");
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//                 }
//                 case RemovalRelations:
//                 {
//                     TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n");
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//                 }
//                 default:
//                     ERR_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n",
//                         Stack->Parameters.QueryDeviceRelations.Type);
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//             }
//             break;
//         }
//         case IRP_MN_QUERY_CAPABILITIES: /* (optional) 0x09 */
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_CAPABILITIES\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS: /* (optional) 0x0d */
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_PNP_DEVICE_STATE: /* (optional) 0x14 */
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_PNP_DEVICE_STATE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_QUERY_REMOVE_DEVICE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//         case IRP_MN_REMOVE_DEVICE:
//         {
//             TRACE_(I8042PRT, "IRP_MJ_PNP / IRP_MN_REMOVE_DEVICE\n");
//             Status = ForwardIrpAndForget(DeviceObject, Irp);
//             i8042RemoveDevice(DeviceObject);
//             return Status;
//         }
//         default:
//         {
//             ERR_(I8042PRT, "IRP_MJ_PNP / unknown minor function 0x%x\n", MinorFunction);
//             return ForwardIrpAndForget(DeviceObject, Irp);
//         }
//     }
//
//     Irp->IoStatus.Information = Information;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
