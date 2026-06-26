use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor};
use core::ffi::c_void;

extern "C" {
    fn READ_PORT_USHORT(Register: PUSHORT) -> USHORT;
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
    fn WRITE_PORT_USHORT(Port: PUSHORT, Value: USHORT);
    fn UhciHardwarePresent(UhciExtension: PUHCI_EXTENSION) -> BOOLEAN;
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}

// ---- file-scope preamble (translated) ----
// (no declarations to translate; `NDEBUG` is a function-like/empty object macro skipped)

#[no_mangle]
pub unsafe extern "system" fn UhciRHGetRootHubData(uhciExtension: PVOID, rootHubData: PVOID) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let RootHubData: PUSBPORT_ROOT_HUB_DATA = rootHubData as PUSBPORT_ROOT_HUB_DATA;
    let mut HubCharacteristics: USBPORT_HUB_11_CHARACTERISTICS = core::mem::zeroed();

    HubCharacteristics.AsUSHORT = 0;
    HubCharacteristics.set_PowerControlMode(1);
    HubCharacteristics.set_NoPowerSwitching(1);
    HubCharacteristics.set_OverCurrentProtectionMode(1);

    if (*UhciExtension).HcFlavor as u32 != UHCI_Piix4 as u32 {
        HubCharacteristics.set_NoOverCurrentProtection(1);
    }

    (*RootHubData).NumberOfPorts = UHCI_NUM_ROOT_HUB_PORTS as ULONG;
    (*RootHubData).HubCharacteristics.Usb11HubCharacteristics = HubCharacteristics;
    (*RootHubData).PowerOnToPowerGood = 1 as ULONG;
    (*RootHubData).HubControlCurrent = 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHGetStatus(
    uhciExtension: PVOID,
    Status: PUSHORT,
) -> MPSTATUS {
    // DPRINT("UhciRHGetStatus: ...\n");
    *Status = USB_GETSTATUS_SELF_POWERED as core::ffi::c_ushort;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHGetPortStatus(
    uhciExtension: PVOID,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut PortBit: ULONG;
    let mut portStatus: USB_20_PORT_STATUS = core::mem::zeroed();
    let mut portChange: USB_20_PORT_CHANGE = core::mem::zeroed();

    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            64 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    BaseRegister = (*UhciExtension).BaseRegister;
    PortControlRegister = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*BaseRegister).PortControl) as *mut UHCI_PORT_STATUS_CONTROL)
            .offset((Port as core::ffi::c_int - 1) as isize))
        .AsUSHORT
    ) as PUSHORT;
    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    portStatus.AsUshort16 = 0 as USHORT;
    portChange.AsUshort16 = 0 as USHORT;

    portStatus.set_CurrentConnectStatus(PortControl.CurrentConnectStatus() as USHORT);
    portStatus.set_PortEnabledDisabled(PortControl.PortEnabledDisabled() as USHORT);

    if PortControl.Suspend() as core::ffi::c_int == 1
        && PortControl.PortEnabledDisabled() as core::ffi::c_int == 1
    {
        portStatus.set_Suspend(1 as USHORT);
    } else {
        portStatus.set_Suspend(0 as USHORT);
    }

    if true {
        portStatus.set_OverCurrent(
            (PortControl.Reserved2() as core::ffi::c_int & 1) as USHORT,
        );
        portStatus.set_PortPower(
            (!(PortControl.Reserved2() as core::ffi::c_int) & 1) as USHORT,
        );
        portChange.set_OverCurrentIndicatorChange(
            ((PortControl.Reserved2() as core::ffi::c_int & 2) != 0) as core::ffi::c_int as USHORT,
        );
    } else {
        portStatus.set_OverCurrent(0 as USHORT);
        portStatus.set_PortPower(1 as USHORT);
        portChange.set_OverCurrentIndicatorChange(0 as USHORT);
    }

    portStatus.set_HighSpeedDeviceAttached(0 as USHORT);

    portStatus.set_Reset(PortControl.PortReset() as USHORT);
    portStatus.set_LowSpeedDeviceAttached(PortControl.LowSpeedDevice() as USHORT);
    portChange.set_ConnectStatusChange(PortControl.ConnectStatusChange() as USHORT);

    PortBit = ((1 as core::ffi::c_int) << (Port as core::ffi::c_int - 1)) as ULONG;

    if (*UhciExtension).ResetPortMask & PortBit != 0 {
        portChange.set_ConnectStatusChange(0 as USHORT);
        portChange.set_PortEnableDisableChange(0 as USHORT);
    } else {
        portChange.set_PortEnableDisableChange(PortControl.PortEnableDisableChange() as USHORT);
    }

    if (*UhciExtension).SuspendChangePortMask & PortBit != 0 {
        portChange.set_SuspendChange(1 as USHORT);
    }

    if (*UhciExtension).ResetChangePortMask & PortBit != 0 {
        portChange.set_ResetChange(1 as USHORT);
    }

    (*PortStatus).PortStatus.Usb20PortStatus = portStatus;
    (*PortStatus).PortChange.Usb20PortChange = portChange;

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHGetHubStatus(
    uhciExtension: PVOID,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
) -> MPSTATUS {
    (*HubStatus).AsUlong32 = 0 as ULONG;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHPortResetComplete(uhciExtension: PVOID, pPort: PVOID) {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut ix: ULONG;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let Port: USHORT;

    BaseRegister = (*UhciExtension).BaseRegister;

    Port = *(pPort as PUSHORT);

    PortControlRegister = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*BaseRegister).PortControl) as *mut UHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1) as isize))
        .AsUSHORT
    ) as PUSHORT;
    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    PortControl.set_ConnectStatusChange(0);
    PortControl.set_PortEnableDisableChange(0);
    PortControl.set_PortReset(0);

    WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

    while UhciHardwarePresent(UhciExtension) != 0 {
        PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

        if PortControl.PortReset() == 0 {
            break;
        }
    }

    ix = 0;
    while ix < 10 {
        KeStallExecutionProcessor(50);

        PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

        if PortControl.PortEnabledDisabled() == 1 {
            break;
        }

        PortControl.set_PortEnabledDisabled(1);
        WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

        ix = ix.wrapping_add(1);
    }

    PortControl.set_ConnectStatusChange(1);
    PortControl.set_PortEnableDisableChange(1);
    WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

    if (*UhciExtension).HcFlavor as u32 == UHCI_VIA as u32
        || (*UhciExtension).HcFlavor as u32 == UHCI_VIA_x01 as u32
        || (*UhciExtension).HcFlavor as u32 == UHCI_VIA_x02 as u32
        || (*UhciExtension).HcFlavor as u32 == UHCI_VIA_x03 as u32
        || (*UhciExtension).HcFlavor as u32 == UHCI_VIA_x04 as u32
    {
        DbgBreakPoint();
        return;
    }

    (*UhciExtension).ResetChangePortMask |= (1i32 << (Port as i32 - 1)) as ULONG;
    (*UhciExtension).ResetPortMask &= !((1i32 << (Port as i32 - 1)) as ULONG);

    (RegPacket.UsbPortInvalidateRootHub.expect("non-null function pointer"))(UhciExtension as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHSetFeaturePortResetWorker(
    UhciExtension: PUHCI_EXTENSION,
    pPort: PUSHORT,
) {
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut Port: USHORT;

    BaseRegister = (*UhciExtension).BaseRegister;

    Port = *(pPort as PUSHORT);
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const i8 as PVOID,
            230 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    PortControlRegister = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*BaseRegister).PortControl) as *mut UHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1) as isize))
        .AsUSHORT
    ) as PUSHORT;
    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    PortControl.set_ConnectStatusChange(0);
    PortControl.set_PortEnableDisableChange(0);
    PortControl.set_PortReset(1);

    WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

    RegPacket.UsbPortRequestAsyncCallback.expect("non-null function pointer")(
        UhciExtension as PVOID,
        10 as ULONG,
        pPort as PVOID,
        ((core::mem::size_of::<u16>() as SIZE_T as u32) as _),
        Some(UhciRHPortResetComplete as unsafe extern "system" fn(PVOID, PVOID) -> ()),
    );
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHSetFeaturePortReset(
    uhciExtension: PVOID,
    mut Port: USHORT,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let ResetPortMask: ULONG;
    let PortBit: ULONG;

    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            259 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    ResetPortMask = (*UhciExtension).ResetPortMask;
    PortBit = ((1 as core::ffi::c_int) << (Port as core::ffi::c_int - 1 as core::ffi::c_int)) as ULONG;

    if ResetPortMask & PortBit != 0 {
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*UhciExtension).ResetPortMask = ResetPortMask | PortBit;

    if (*UhciExtension).HcFlavor as core::ffi::c_uint
        == UHCI_VIA as core::ffi::c_int as core::ffi::c_uint
        || (*UhciExtension).HcFlavor as core::ffi::c_uint
            == UHCI_VIA_x01 as core::ffi::c_int as core::ffi::c_uint
        || (*UhciExtension).HcFlavor as core::ffi::c_uint
            == UHCI_VIA_x02 as core::ffi::c_int as core::ffi::c_uint
        || (*UhciExtension).HcFlavor as core::ffi::c_uint
            == UHCI_VIA_x03 as core::ffi::c_int as core::ffi::c_uint
        || (*UhciExtension).HcFlavor as core::ffi::c_uint
            == UHCI_VIA_x04 as core::ffi::c_int as core::ffi::c_uint
    {
        if DbgPrint(
            b"(%s:%d) UhciRHSetFeaturePortReset: Via chip. FIXME\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char,
            275 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"drivers/usb/usbuhci/roothub.c\0"
                    as *const u8 as *const core::ffi::c_char,
                275 as core::ffi::c_int,
            );
        }
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    UhciRHSetFeaturePortResetWorker(UhciExtension, core::ptr::addr_of_mut!(Port));

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHSetFeaturePortPower(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            290 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHPortEnable(
    uhciExtension: PVOID,
    Port: USHORT,
    IsSet: BOOLEAN,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    if Port == 0 {
        RtlAssert(
            b"Port\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            307 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    BaseRegister = (*UhciExtension).BaseRegister;
    PortControlRegister = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*BaseRegister).PortControl) as *mut UHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1) as isize))
        .AsUSHORT
    ) as PUSHORT;

    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    PortControl.set_ConnectStatusChange(0);
    PortControl.set_PortEnableDisableChange(0);

    if IsSet != 0 {
        PortControl.set_PortEnabledDisabled(1);
    } else {
        PortControl.set_PortEnabledDisabled(0);
    }

    WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHSetFeaturePortEnable(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            334 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    return UhciRHPortEnable(UhciExtension as PVOID, Port, TRUE as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHSetFeaturePortSuspend(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            344 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortEnable(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            355 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return UhciRHPortEnable(UhciExtension as PVOID, Port, 0 as BOOLEAN);
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortPower(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            365 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortSuspend(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            375 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortEnableChange(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    if Port == 0 {
        RtlAssert(
            b"Port\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                .as_ptr() as *const i8 as PVOID,
            391 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    BaseRegister = (*UhciExtension).BaseRegister;
    PortControlRegister = (core::ptr::addr_of_mut!((*BaseRegister).PortControl)
        as *mut UHCI_PORT_STATUS_CONTROL)
        .offset((Port as i32 - 1 as i32) as isize) as *mut UHCI_PORT_STATUS_CONTROL
        as PUSHORT;
    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    PortControl.set_ConnectStatusChange(0 as USHORT);
    PortControl.set_PortEnableDisableChange(1 as USHORT);
    WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortConnectChange(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    if Port == 0 {
        RtlAssert(
            b"Port\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            416 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    BaseRegister = (*UhciExtension).BaseRegister;
    PortControlRegister = (core::ptr::addr_of_mut!((*BaseRegister).PortControl)
        as *mut UHCI_PORT_STATUS_CONTROL)
        .offset((Port as core::ffi::c_int - 1) as isize)
        as *mut UHCI_PORT_STATUS_CONTROL as PUSHORT;
    PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

    if PortControl.ConnectStatusChange() as core::ffi::c_int == 1 {
        PortControl.set_PortEnableDisableChange(0 as USHORT);
        PortControl.set_ConnectStatusChange(1 as USHORT);
        WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortResetChange(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            440 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    (*UhciExtension).ResetChangePortMask &=
        !((1 as core::ffi::c_int) << Port as core::ffi::c_int - 1 as core::ffi::c_int)
            as core::ffi::c_ulong;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortSuspendChange(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            451 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHClearFeaturePortOvercurrentChange(
    uhciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    if Port == 0 {
        RtlAssert(
            b"Port\0" as *const u8 as *const i8 as PVOID,
            b"drivers/usb/usbuhci/roothub.c\0"
                as *const u8 as *const i8 as PVOID,
            461 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHDisableIrq(uhciExtension: PVOID) {
    return;
}

#[no_mangle]
pub unsafe extern "system" fn UhciRHEnableIrq(uhciExtension: PVOID) {
    /* Do nothing */
    return;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB UHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBUHCI root hub functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbuhci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// VOID
// NTAPI
// UhciRHGetRootHubData(IN PVOID uhciExtension,
//                      IN PVOID rootHubData)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUSBPORT_ROOT_HUB_DATA RootHubData = rootHubData;
//     USBPORT_HUB_11_CHARACTERISTICS HubCharacteristics;
//
//     DPRINT("UhciRHGetRootHubData: ...\n");
//
//     HubCharacteristics.AsUSHORT = 0;
//     HubCharacteristics.PowerControlMode = 1;
//     HubCharacteristics.NoPowerSwitching = 1;
//     HubCharacteristics.OverCurrentProtectionMode = 1;
//
//     if (UhciExtension->HcFlavor != UHCI_Piix4)
//         HubCharacteristics.NoOverCurrentProtection = 1;
//
//     RootHubData->NumberOfPorts = UHCI_NUM_ROOT_HUB_PORTS;
//     RootHubData->HubCharacteristics.Usb11HubCharacteristics = HubCharacteristics;
//     RootHubData->PowerOnToPowerGood = 1;
//     RootHubData->HubControlCurrent = 0;
// }
//
// MPSTATUS
// NTAPI
// UhciRHGetStatus(IN PVOID uhciExtension,
//                 IN PUSHORT Status)
// {
//     DPRINT("UhciRHGetStatus: ...\n");
//     *Status = USB_GETSTATUS_SELF_POWERED;
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHGetPortStatus(IN PVOID uhciExtension,
//                     IN USHORT Port,
//                     IN PUSB_PORT_STATUS_AND_CHANGE PortStatus)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//     ULONG PortBit;
//     USB_20_PORT_STATUS portStatus;
//     USB_20_PORT_CHANGE portChange;
//
//     //DPRINT("UhciRHGetPortStatus: ...\n");
//
//     ASSERT(Port);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     PortControlRegister = &BaseRegister->PortControl[Port-1].AsUSHORT;
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     portStatus.AsUshort16 = 0;
//     portChange.AsUshort16 = 0;
//
//     portStatus.CurrentConnectStatus = PortControl.CurrentConnectStatus;
//     portStatus.PortEnabledDisabled = PortControl.PortEnabledDisabled;
//
//     if (PortControl.Suspend == 1 &&
//         PortControl.PortEnabledDisabled == 1)
//     {
//         portStatus.Suspend = 1;
//     }
//     else
//     {
//         portStatus.Suspend = 0;
//     }
//
//     //if (UhciExtension->HcFlavor == UHCI_Piix4) // check will work after supporting HcFlavor in usbport.
//     if (TRUE)
//     {
//         portStatus.OverCurrent = PortControl.Reserved2 & 1;
//         portStatus.PortPower = (~PortControl.Reserved2 & 1);
//         portChange.OverCurrentIndicatorChange = (PortControl.Reserved2 & 2) != 0;
//     }
//     else
//     {
//         portStatus.OverCurrent = 0;
//         portStatus.PortPower = 1;
//         portChange.OverCurrentIndicatorChange = 0;
//     }
//
//     portStatus.HighSpeedDeviceAttached = 0;
//
//     portStatus.Reset = PortControl.PortReset;
//     portStatus.LowSpeedDeviceAttached = PortControl.LowSpeedDevice;
//     portChange.ConnectStatusChange = PortControl.ConnectStatusChange;
//
//     PortBit = 1 << (Port - 1);
//
//     if (UhciExtension->ResetPortMask & PortBit)
//     {
//         portChange.ConnectStatusChange = 0;
//         portChange.PortEnableDisableChange = 0;
//     }
//     else
//     {
//         portChange.PortEnableDisableChange = PortControl.PortEnableDisableChange;
//     }
//
//     if (UhciExtension->SuspendChangePortMask & PortBit)
//         portChange.SuspendChange = 1;
//
//     if (UhciExtension->ResetChangePortMask & PortBit)
//         portChange.ResetChange = 1;
//
//     PortStatus->PortStatus.Usb20PortStatus = portStatus;
//     PortStatus->PortChange.Usb20PortChange = portChange;
//
//     //DPRINT("UhciRHGetPortStatus: PortControl.AsUSHORT[%x] - %X, PortStatus - %X\n",
//     //       Port,
//     //       PortControl.AsUSHORT,
//     //       PortStatus->AsUlong32);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHGetHubStatus(IN PVOID uhciExtension,
//                    IN PUSB_HUB_STATUS_AND_CHANGE HubStatus)
// {
//     //DPRINT("UhciRHGetHubStatus: ...\n");
//     HubStatus->AsUlong32 = 0;
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciRHPortResetComplete(IN PVOID uhciExtension,
//                         IN PVOID pPort)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     ULONG ix;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//     USHORT Port;
//
//     DPRINT("UhciRHPortResetComplete: ...\n");
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     Port = *(PUSHORT)pPort;
//     ASSERT(Port);
//
//     PortControlRegister = &BaseRegister->PortControl[Port - 1].AsUSHORT;
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     PortControl.ConnectStatusChange = 0;
//     PortControl.PortEnableDisableChange = 0;
//     PortControl.PortReset = 0;
//
//     WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//
//     while (UhciHardwarePresent(UhciExtension))
//     {
//         PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//         if (PortControl.PortReset == 0)
//             break;
//     }
//
//     for (ix = 0; ix < 10; ++ix)
//     {
//         KeStallExecutionProcessor(50);
//
//         PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//         if (PortControl.PortEnabledDisabled == 1)
//             break;
//
//         PortControl.PortEnabledDisabled = 1;
//         WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//     }
//
//     PortControl.ConnectStatusChange = 1;
//     PortControl.PortEnableDisableChange = 1;
//     WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//
//     if (UhciExtension->HcFlavor == UHCI_VIA ||
//         UhciExtension->HcFlavor == UHCI_VIA_x01 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x02 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x03 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x04)
//     {
//         DPRINT1("UhciRHPortResetComplete: Via chip. FIXME\n");
//         DbgBreakPoint();
//         return;
//     }
//
//     UhciExtension->ResetChangePortMask |= (1 << (Port - 1));
//     UhciExtension->ResetPortMask &= ~(1 << (Port - 1));
//
//     RegPacket.UsbPortInvalidateRootHub(UhciExtension);
// }
//
// VOID
// NTAPI
// UhciRHSetFeaturePortResetWorker(IN PUHCI_EXTENSION UhciExtension,
//                                 IN PUSHORT pPort)
// {
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//     USHORT Port;
//
//     DPRINT("UhciRHSetFeaturePortResetWorker: ...\n");
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     Port = *(PUSHORT)pPort;
//     ASSERT(Port);
//
//     PortControlRegister = &BaseRegister->PortControl[Port - 1].AsUSHORT;
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     PortControl.ConnectStatusChange = 0;
//     PortControl.PortEnableDisableChange = 0;
//     PortControl.PortReset = 1;
//
//     WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//
//     RegPacket.UsbPortRequestAsyncCallback(UhciExtension,
//                                           10, // TimerValue
//                                           pPort,
//                                           sizeof(*pPort),
//                                           UhciRHPortResetComplete);
// }
//
// MPSTATUS
// NTAPI
// UhciRHSetFeaturePortReset(IN PVOID uhciExtension,
//                           IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     ULONG ResetPortMask;
//     ULONG PortBit;
//
//     DPRINT("UhciRHSetFeaturePortReset: ...\n");
//
//     ASSERT(Port);
//
//     ResetPortMask = UhciExtension->ResetPortMask;
//     PortBit = 1 << (Port - 1);
//
//     if (ResetPortMask & PortBit)
//         return MP_STATUS_FAILURE;
//
//     UhciExtension->ResetPortMask = ResetPortMask | PortBit;
//
//     if (UhciExtension->HcFlavor == UHCI_VIA ||
//         UhciExtension->HcFlavor == UHCI_VIA_x01 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x02 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x03 ||
//         UhciExtension->HcFlavor == UHCI_VIA_x04)
//     {
//         DPRINT1("UhciRHSetFeaturePortReset: Via chip. FIXME\n");
//         return MP_STATUS_SUCCESS;
//     }
//
//     UhciRHSetFeaturePortResetWorker(UhciExtension, &Port);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHSetFeaturePortPower(IN PVOID uhciExtension,
//                           IN USHORT Port)
// {
//     DPRINT("UhciRHSetFeaturePortPower: ...\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHPortEnable(IN PVOID uhciExtension,
//                  IN USHORT Port,
//                  IN BOOLEAN IsSet)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//
//     DPRINT("UhciRHPortEnable: ...\n");
//
//     ASSERT(Port);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     PortControlRegister = &BaseRegister->PortControl[Port-1].AsUSHORT;
//
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     PortControl.ConnectStatusChange = 0;
//     PortControl.PortEnableDisableChange = 0;
//
//     if (IsSet)
//         PortControl.PortEnabledDisabled = 1;
//     else
//         PortControl.PortEnabledDisabled = 0;
//
//     WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHSetFeaturePortEnable(IN PVOID uhciExtension,
//                            IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     DPRINT("UhciRHSetFeaturePortEnable: ...\n");
//     ASSERT(Port);
//     return UhciRHPortEnable(UhciExtension, Port, TRUE);
// }
//
// MPSTATUS
// NTAPI
// UhciRHSetFeaturePortSuspend(IN PVOID uhciExtension,
//                             IN USHORT Port)
// {
//     DPRINT("UhciRHSetFeaturePortSuspend: UNIMPLEMENTED. FIXME\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortEnable(IN PVOID uhciExtension,
//                              IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     DPRINT("UhciRHClearFeaturePortEnable: ...\n");
//     ASSERT(Port);
//     return UhciRHPortEnable(UhciExtension, Port, FALSE);
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortPower(IN PVOID uhciExtension,
//                             IN USHORT Port)
// {
//     DPRINT("UhciRHClearFeaturePortPower: UNIMPLEMENTED. FIXME\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortSuspend(IN PVOID uhciExtension,
//                               IN USHORT Port)
// {
//     DPRINT("UhciRHClearFeaturePortSuspend: UNIMPLEMENTED. FIXME\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortEnableChange(IN PVOID uhciExtension,
//                                    IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//
//     DPRINT("UhciRHClearFeaturePortEnableChange: ...\n");
//
//     ASSERT(Port);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     PortControlRegister = (PUSHORT)&BaseRegister->PortControl[Port - 1];
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     PortControl.ConnectStatusChange = 0;
//     PortControl.PortEnableDisableChange = 1;
//     WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortConnectChange(IN PVOID uhciExtension,
//                                     IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//
//     DPRINT("UhciRHClearFeaturePortConnectChange: Port - %04X\n", Port);
//
//     ASSERT(Port);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     PortControlRegister = (PUSHORT)&BaseRegister->PortControl[Port - 1];
//     PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//     if (PortControl.ConnectStatusChange == 1)
//     {
//         /* WC (Write Clear) bits */
//         PortControl.PortEnableDisableChange = 0;
//         PortControl.ConnectStatusChange = 1;
//         WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortResetChange(IN PVOID uhciExtension,
//                                   IN USHORT Port)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     DPRINT("UhciRHClearFeaturePortResetChange: ...\n");
//     ASSERT(Port);
//     UhciExtension->ResetChangePortMask &= ~(1 << (Port - 1));
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortSuspendChange(IN PVOID uhciExtension,
//                                     IN USHORT Port)
// {
//     DPRINT("UhciRHClearFeaturePortSuspendChange: UNIMPLEMENTED. FIXME\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciRHClearFeaturePortOvercurrentChange(IN PVOID uhciExtension,
//                                         IN USHORT Port)
// {
//     DPRINT("UhciRHClearFeaturePortOvercurrentChange: UNIMPLEMENTED. FIXME\n");
//     ASSERT(Port);
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciRHDisableIrq(IN PVOID uhciExtension)
// {
//     /* Do nothing */
//     return;
// }
//
// VOID
// NTAPI
// UhciRHEnableIrq(IN PVOID uhciExtension)
// {
//     /* Do nothing */
//     return;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
