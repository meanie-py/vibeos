use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor};
use core::ffi::c_void;

extern "C" {
}
extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG
// #define NDEBUG_EHCI_ROOT_HUB

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ChirpRootPort(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let mut PortStatusReg: PULONG = core::ptr::null_mut::<ULONG>();
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut PortBit: ULONG;
    let mut ix: ULONG;

    // DPRINT_RH!("EHCI_RH_ChirpRootPort: Port - %x\n", Port);
    if !(Port as i32 != 0 as i32) {
        RtlAssert(
            b"Port != 0\0" as *const u8 as *const i8 as PVOID,
            b"drivers/usb/usbehci/roothub.c\0"
                as *const u8 as *const i8 as PVOID,
            28 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1 as i32) as isize))
        .AsULONG
    ) as PULONG;
    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);
    // DPRINT_RH!("EHCI_RH_ChirpRootPort: PortSC - %X\n", PortSC.AsULONG);

    PortBit = ((1 as i32) << (Port as i32 - 1 as i32)) as ULONG;

    if PortBit & (*EhciExtension).ResetPortBits != 0 {
        // DPRINT_RH!("EHCI_RH_ChirpRootPort: Skip port - %x\n", Port);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    if PortSC.c2rust_unnamed.PortPower() as i32 == 0 as i32 {
        // DPRINT_RH!("EHCI_RH_ChirpRootPort: Skip port - %x\n", Port);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    if PortSC.c2rust_unnamed.CurrentConnectStatus() as i32 == 0 as i32
        || PortSC.c2rust_unnamed.PortEnabledDisabled() as i32 == 1 as i32
        || PortSC.c2rust_unnamed.PortOwner() as i32 == EHCI_PORT_OWNER_COMPANION_CONTROLLER
    {
        // DPRINT_RH!("EHCI_RH_ChirpRootPort: No port - %x\n", Port);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    if PortSC.c2rust_unnamed.LineStatus() as i32 == EHCI_LINE_STATUS_K_STATE_LOW_SPEED
        && PortSC.c2rust_unnamed.Suspend() as i32 == 0 as i32
        && PortSC.c2rust_unnamed.CurrentConnectStatus() as i32 == 1 as i32
    {
        /* Attached device is not a high-speed device.
           Release ownership of the port to a selected HC.
           Companion HC owns and controls the port. Section 4.2 */
        PortSC
            .c2rust_unnamed
            .set_PortOwner(EHCI_PORT_OWNER_COMPANION_CONTROLLER as ULONG);
        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

        // DPRINT_RH!("EHCI_RH_ChirpRootPort: Companion HC port - %x\n", Port);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    // DPRINT("EHCI_RH_ChirpRootPort: EhciExtension - %p, Port - %x\n", EhciExtension, Port);

    PortSC.c2rust_unnamed.set_PortEnabledDisabled(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortReset(1 as ULONG);
    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    RegPacket.UsbPortWait.expect("non-null function pointer")(EhciExtension as PVOID, 10 as ULONG);

    loop {
        PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

        PortSC.c2rust_unnamed.set_ConnectStatusChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_PortEnableDisableChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_OverCurrentChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_PortReset(0 as ULONG);

        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

        ix = 0 as ULONG;
        while ix <= 500 as ULONG {
            KeStallExecutionProcessor(20 as ULONG);
            PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

            // DPRINT_RH!("EHCI_RH_ChirpRootPort: Reset port - %x\n", Port);

            if PortSC.c2rust_unnamed.PortReset() as i32 == 0 as i32 {
                break;
            }
            ix = (ix as ULONG).wrapping_add(20 as ULONG) as ULONG;
        }

        if !(PortSC.c2rust_unnamed.PortReset() as i32 == 1 as i32) {
            break;
        }
    }

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    if PortSC.c2rust_unnamed.PortEnabledDisabled() as i32 == 1 as i32 {
        PortSC.c2rust_unnamed.set_ConnectStatusChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_PortEnabledDisabled(0 as ULONG);
        PortSC.c2rust_unnamed.set_PortEnableDisableChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_OverCurrentChange(0 as ULONG);

        RegPacket.UsbPortWait.expect("non-null function pointer")(
            EhciExtension as PVOID,
            10 as ULONG,
        );

        (*EhciExtension).ResetPortBits |= PortBit;

        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
        // DPRINT_RH!("EHCI_RH_ChirpRootPort: Disable port - %x\n", Port);
    } else {
        PortSC
            .c2rust_unnamed
            .set_PortOwner(EHCI_PORT_OWNER_COMPANION_CONTROLLER as ULONG);
        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
        // DPRINT_RH!("EHCI_RH_ChirpRootPort: Companion HC port - %x\n", Port);
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_GetRootHubData(
    ehciExtension: PVOID,
    rootHubData: PVOID,
) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let RootHubData: PUSBPORT_ROOT_HUB_DATA;
    let mut HubCharacteristics: USBPORT_HUB_20_CHARACTERISTICS = core::mem::zeroed();

    DPRINT_RH!(
        b"EHCI_RH_GetRootHubData: EhciExtension - %p, rootHubData - %p\n\0".as_ptr() as *const i8,
        EhciExtension,
        rootHubData,
    );

    RootHubData = rootHubData as PUSBPORT_ROOT_HUB_DATA;

    (*RootHubData).NumberOfPorts = (*EhciExtension).NumberOfPorts as ULONG;

    HubCharacteristics.AsUSHORT = 0 as USHORT;

    /* Logical Power Switching Mode */
    if (*EhciExtension).PortPowerControl as i32 == 1 {
        /* Individual port power switching */
        HubCharacteristics.set_PowerControlMode(1 as USHORT);
    } else {
        /* Ganged power switching (all ports' power at once) */
        HubCharacteristics.set_PowerControlMode(0 as USHORT);
    }

    HubCharacteristics.set_NoPowerSwitching(0 as USHORT);

    /* EHCI RH is not part of a compound device */
    HubCharacteristics.set_PartOfCompoundDevice(0 as USHORT);

    /* Global Over-current Protection */
    HubCharacteristics.set_OverCurrentProtectionMode(0 as USHORT);

    (*RootHubData).HubCharacteristics.Usb20HubCharacteristics = HubCharacteristics;

    (*RootHubData).PowerOnToPowerGood = 2 as ULONG; // Time (in 2 ms intervals)
    (*RootHubData).HubControlCurrent = 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_GetStatus(
    ehciExtension: PVOID,
    Status: PUSHORT,
) -> MPSTATUS {
    DPRINT_RH!(b"EHCI_RH_GetStatus: ... \n\0".as_ptr() as *const i8);
    *Status = USB_GETSTATUS_SELF_POWERED as ::core::ffi::c_ushort;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_GetPortStatus(
    ehciExtension: PVOID,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut status: USB_PORT_STATUS_AND_CHANGE = core::mem::zeroed();
    let PortMaskBits: ULONG;

    // ASSERT(Port != 0);

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(*EhciExtension).OperationalRegs).PortControl[(Port - 1) as usize].AsULONG
    ) as PULONG;
    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    if PortSC.CurrentConnectStatus() != 0 {
        DPRINT_RH!(
            b"EHCI_RH_GetPortStatus: Port - %x, PortSC.AsULONG - %X\n\0".as_ptr() as *const i8,
            Port as core::ffi::c_int,
            PortSC.AsULONG,
        );
    }

    (*PortStatus).AsUlong32 = 0;

    if PortSC.LineStatus() as core::ffi::c_int == EHCI_LINE_STATUS_K_STATE_LOW_SPEED
        && PortSC.PortOwner() as core::ffi::c_int != EHCI_PORT_OWNER_COMPANION_CONTROLLER
        && (PortSC.PortEnabledDisabled() as core::ffi::c_int
            | PortSC.Suspend() as core::ffi::c_int)
            != 0
        && PortSC.CurrentConnectStatus() as core::ffi::c_int == 1
    {
        // DPRINT removed
        PortSC.set_PortOwner(EHCI_PORT_OWNER_COMPANION_CONTROLLER as ULONG);
        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    status.AsUlong32 = 0;

    status
        .PortStatus
        .Usb20PortStatus
        .set_CurrentConnectStatus(PortSC.CurrentConnectStatus() as USHORT);
    status
        .PortStatus
        .Usb20PortStatus
        .set_PortEnabledDisabled(PortSC.PortEnabledDisabled() as USHORT);
    status
        .PortStatus
        .Usb20PortStatus
        .set_Suspend(PortSC.Suspend() as USHORT);
    status
        .PortStatus
        .Usb20PortStatus
        .set_OverCurrent(PortSC.OverCurrentActive() as USHORT);
    status
        .PortStatus
        .Usb20PortStatus
        .set_Reset(PortSC.PortReset() as USHORT);
    status
        .PortStatus
        .Usb20PortStatus
        .set_PortPower(PortSC.PortPower() as USHORT);
    if PortSC.PortOwner() as core::ffi::c_int == EHCI_PORT_OWNER_COMPANION_CONTROLLER {
        status
            .PortStatus
            .Usb20PortStatus
            .set_Reserved1(USB20_PORT_STATUS_RESERVED1_OWNED_BY_COMPANION as USHORT);
    }

    status
        .PortChange
        .Usb20PortChange
        .set_PortEnableDisableChange(PortSC.PortEnableDisableChange() as USHORT);
    status
        .PortChange
        .Usb20PortChange
        .set_OverCurrentIndicatorChange(PortSC.OverCurrentChange() as USHORT);

    PortMaskBits = (1 << (Port as core::ffi::c_int - 1)) as ULONG;

    if status.PortStatus.Usb20PortStatus.CurrentConnectStatus() != 0 {
        status
            .PortStatus
            .Usb20PortStatus
            .set_LowSpeedDeviceAttached(0 as USHORT);
    }

    status
        .PortStatus
        .Usb20PortStatus
        .set_HighSpeedDeviceAttached(1 as USHORT);

    if PortSC.ConnectStatusChange() != 0 {
        (*EhciExtension).ConnectPortBits |= PortMaskBits;
    }

    if (*EhciExtension).FinishResetPortBits & PortMaskBits != 0 {
        status
            .PortChange
            .Usb20PortChange
            .set_ResetChange(1 as USHORT);
    }

    if (*EhciExtension).ConnectPortBits & PortMaskBits != 0 {
        status
            .PortChange
            .Usb20PortChange
            .set_ConnectStatusChange(1 as USHORT);
    }

    if (*EhciExtension).SuspendPortBits & PortMaskBits != 0 {
        status
            .PortChange
            .Usb20PortChange
            .set_SuspendChange(1 as USHORT);
    }

    *PortStatus = status;

    if status.PortStatus.Usb20PortStatus.CurrentConnectStatus() != 0 {
        DPRINT_RH!(
            b"EHCI_RH_GetPortStatus: Port - %x, status.AsULONG - %X\n\0".as_ptr() as *const i8,
            Port as core::ffi::c_int,
            status.AsUlong32,
        );
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_GetHubStatus(
    ehciExtension: PVOID,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
) -> MPSTATUS {
    DPRINT_RH!(b"EHCI_RH_GetHubStatus: ... \n\0".as_ptr() as *const i8);
    (*HubStatus).AsUlong32 = 0 as ULONG;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_FinishReset(ehciExtension: PVOID, Context: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let Port: PUSHORT = Context as PUSHORT;

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((*Port as i32 - 1) as isize))
        .AsULONG
    ) as PULONG;
    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    if PortSC.AsULONG != -1i32 as ULONG {
        if PortSC.CurrentConnectStatus() == 0 {
            // DPRINT("EHCI_RH_FinishReset: PortSC.AsULONG - %X\n", PortSC.AsULONG);
        }

        if PortSC.PortEnabledDisabled() != 0
            || PortSC.CurrentConnectStatus() == 0
            || PortSC.ConnectStatusChange() != 0
        {
            (*EhciExtension).FinishResetPortBits |= (1i32 << (*Port as i32 - 1)) as ULONG;
            (RegPacket.UsbPortInvalidateRootHub).expect("non-null function pointer")(
                EhciExtension as PVOID,
            );
        } else {
            PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);
            PortSC.set_PortOwner(EHCI_PORT_OWNER_COMPANION_CONTROLLER as ULONG);
            WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
            (*EhciExtension).FinishResetPortBits |= (1i32 << (*Port as i32 - 1)) as ULONG;
        }

        (*EhciExtension).ResetPortBits &= !((1i32 << (*Port as i32 - 1)) as ULONG);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_PortResetComplete(ehciExtension: PVOID, Context: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let mut PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut ix: ULONG;
    let Port: PUSHORT = Context as PUSHORT;

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((*Port as i32 - 1) as isize))
        .AsULONG
    ) as PULONG;

    loop {
        PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

        PortSC.set_ConnectStatusChange(0);
        PortSC.set_PortEnableDisableChange(0);
        PortSC.set_OverCurrentChange(0);
        PortSC.set_PortReset(0);

        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

        ix = 0;
        while ix <= 500 {
            KeStallExecutionProcessor(20);
            PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

            if PortSC.PortReset() == 0 {
                break;
            }

            ix = ix.wrapping_add(20);
        }

        if !(PortSC.PortReset() == 1 && PortSC.AsULONG != (-1i32) as ULONG) {
            break;
        }
    }

    RegPacket.UsbPortRequestAsyncCallback.expect("non-null function pointer")(
        EhciExtension as PVOID,
        50,
        Port as PVOID,
        (core::mem::size_of::<u16>() as _),
        Some(EHCI_RH_FinishReset as unsafe extern "system" fn(PVOID, PVOID) -> ()),
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_SetFeaturePortReset(
    ehciExtension: PVOID,
    mut Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    // DPRINT("EHCI_RH_SetFeaturePortReset: Port - %x\n", Port);
    if !(Port as i32 != 0) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const i8 as PVOID,
            b"roothub.c\0".as_ptr() as *const i8 as PVOID,
            372 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*((core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1) as isize)))
        .AsULONG
    ) as PULONG;

    (*EhciExtension).ResetPortBits |= (1i32 << (Port as i32 - 1)) as ULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.c2rust_unnamed.set_ConnectStatusChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortEnabledDisabled(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortEnableDisableChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_OverCurrentChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortReset(1 as ULONG);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    (RegPacket.UsbPortRequestAsyncCallback).expect("non-null function pointer")(
        EhciExtension as PVOID,
        50 as ULONG,
        core::ptr::addr_of_mut!(Port) as PVOID,
        (core::mem::size_of::<USHORT>() as _),
        Some(EHCI_RH_PortResetComplete as unsafe extern "system" fn(PVOID, PVOID) -> ()),
    );

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_SetFeaturePortPower(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    DPRINT_RH!(
        b"EHCI_RH_SetFeaturePortPower: Port - %x\n\0".as_ptr() as *const ::core::ffi::c_char,
        Port as ::core::ffi::c_int,
    );

    if !(Port as ::core::ffi::c_int != 0 as ::core::ffi::c_int) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/roothub.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            407 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    PortStatusReg = &raw mut (*(&raw mut (*(*EhciExtension).OperationalRegs).PortControl
        as *mut EHCI_PORT_STATUS_CONTROL)
        .offset((Port as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as isize))
    .AsULONG as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.c2rust_unnamed.set_ConnectStatusChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortEnableDisableChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_OverCurrentChange(0 as ULONG);
    PortSC.c2rust_unnamed.set_PortPower(1 as ULONG);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_SetFeaturePortEnable(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    DPRINT_RH!(b"EHCI_RH_SetFeaturePortEnable: Not supported\n\0".as_ptr() as *const ::core::ffi::c_char);
    if !(Port as ::core::ffi::c_int != 0 as ::core::ffi::c_int) {
        RtlAssert(
            b"Port != 0\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/roothub.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            429 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_SetFeaturePortSuspend(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    // DPRINT("EHCI_RH_SetFeaturePortSuspend: Port - %x\n", Port);
    // ASSERT(Port != 0);

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(*(*EhciExtension).OperationalRegs)
            .PortControl
            .as_mut_ptr()
            .offset((Port as i32 - 1) as isize))
        .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.set_ConnectStatusChange(0);
    PortSC.set_PortEnableDisableChange(0);
    PortSC.set_OverCurrentChange(0);
    PortSC.set_Suspend(1);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
    KeStallExecutionProcessor(125);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortEnable(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(*EhciExtension).OperationalRegs).PortControl[(Port as i32 - 1) as usize].AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.set_ConnectStatusChange(0);
    PortSC.set_PortEnabledDisabled(0);
    PortSC.set_PortEnableDisableChange(0);
    PortSC.set_OverCurrentChange(0);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortPower(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    // DPRINT("EHCI_RH_ClearFeaturePortPower: Port - %x\n", Port);
    // ASSERT(Port != 0);

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(*EhciExtension).OperationalRegs)
            .PortControl[(Port as i32 - 1) as usize]
            .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);
    PortSC.set_PortPower(0);
    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_PortResumeComplete(ehciExtension: PVOID, Context: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let Port: PUSHORT = Context as PUSHORT;

    if Port.is_null() {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbehci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            518 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(&raw mut (*(*EhciExtension).OperationalRegs).PortControl as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((*Port as i32 - 1) as isize))
            .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.set_ConnectStatusChange(0 as ULONG);
    PortSC.set_PortEnableDisableChange(0 as ULONG);
    PortSC.set_OverCurrentChange(0 as ULONG);
    PortSC.set_ForcePortResume(0 as ULONG);
    PortSC.set_Suspend(0 as ULONG);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
    READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    (*EhciExtension).SuspendPortBits |= ((1 as i32) << (*Port as i32 - 1)) as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortSuspend(
    ehciExtension: PVOID,
    mut Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    PortStatusReg = core::ptr::addr_of_mut!(
        (*((core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as core::ffi::c_int - 1) as isize)))
        .AsULONG
    ) as PULONG;

    (*EhciExtension).ResetPortBits |= (1 << (Port as core::ffi::c_int - 1)) as ULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);
    PortSC.c2rust_unnamed.set_ForcePortResume(1);
    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    RegPacket.UsbPortRequestAsyncCallback.expect("non-null function pointer")(
        EhciExtension as PVOID,
        50,
        core::ptr::addr_of_mut!(Port) as PVOID,
        (core::mem::size_of::<USHORT>() as _),
        Some(EHCI_RH_PortResumeComplete as unsafe extern "system" fn(PVOID, PVOID) -> ()),
    );

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortEnableChange(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    // DPRINT("EHCI_RH_ClearFeaturePortEnableChange: Port - %p\n", Port);
    // ASSERT(Port != 0);
    if !(Port as core::ffi::c_int != 0) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"roothub.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            574 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as core::ffi::c_int - 1) as isize))
        .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.set_ConnectStatusChange(0 as ULONG);
    PortSC.set_OverCurrentChange(0 as ULONG);
    PortSC.set_PortEnableDisableChange(1 as ULONG);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortConnectChange(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let mut PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    DPRINT_RH!(
        b"EHCI_RH_ClearFeaturePortConnectChange: Port - %x\n\0".as_ptr() as *const core::ffi::c_char,
        Port as core::ffi::c_int,
    );

    if !(Port as core::ffi::c_int != 0 as core::ffi::c_int) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/roothub.c\0".as_ptr()
                as *const core::ffi::c_char as PVOID,
            599 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*(core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as core::ffi::c_int - 1 as core::ffi::c_int) as isize))
        .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    if PortSC.c2rust_unnamed.ConnectStatusChange() != 0 {
        PortSC.c2rust_unnamed.set_ConnectStatusChange(1 as ULONG);
        PortSC.c2rust_unnamed.set_PortEnableDisableChange(0 as ULONG);
        PortSC.c2rust_unnamed.set_OverCurrentChange(0 as ULONG);

        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);
    }

    (*EhciExtension).ConnectPortBits &= !((1 as core::ffi::c_int)
        << (Port as core::ffi::c_int - 1 as core::ffi::c_int))
        as core::ffi::c_ulong;

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortResetChange(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    if !(Port as core::ffi::c_int != 0 as core::ffi::c_int) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/roothub.c\0"
                .as_ptr() as *const core::ffi::c_char as PVOID,
            627 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    (*EhciExtension).FinishResetPortBits &= !((1 as core::ffi::c_int)
        << Port as core::ffi::c_int - 1 as core::ffi::c_int)
        as core::ffi::c_ulong;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortSuspendChange(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;

    if !(Port as core::ffi::c_int != 0 as core::ffi::c_int) {
        RtlAssert(
            b"Port != 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            641 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    (*EhciExtension).SuspendPortBits &= !((1 as core::ffi::c_int)
        << Port as core::ffi::c_int - 1 as core::ffi::c_int)
        as core::ffi::c_ulong;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_ClearFeaturePortOvercurrentChange(
    ehciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let PortStatusReg: PULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    DPRINT_RH!(
        b"EHCI_RH_ClearFeaturePortOvercurrentChange: Port - %x\n\0".as_ptr() as *const i8,
        Port as core::ffi::c_int,
    );
    if !(Port as i32 != 0 as i32) {
        RtlAssert(
            b"Port != 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbehci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            657 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    PortStatusReg = core::ptr::addr_of_mut!(
        (*((core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).PortControl)
            as *mut EHCI_PORT_STATUS_CONTROL)
            .offset((Port as i32 - 1 as i32) as isize)))
        .AsULONG
    ) as PULONG;

    PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

    PortSC.set_ConnectStatusChange(0 as ULONG);
    PortSC.set_PortEnableDisableChange(0 as ULONG);
    PortSC.set_OverCurrentChange(1 as ULONG);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortSC.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_DisableIrq(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let IntrStsReg: PULONG;
    let mut IntrSts: EHCI_INTERRUPT_ENABLE = core::mem::zeroed();

    DPRINT_RH!(b"EHCI_RH_DisableIrq: ... \n\0".as_ptr() as *const i8);

    IntrStsReg = core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).HcInterruptEnable.AsULONG) as PULONG;
    IntrSts.AsULONG = READ_REGISTER_ULONG(IntrStsReg as *mut ULONG);

    (*EhciExtension).InterruptMask.set_PortChangeInterrupt(0);
    IntrSts.set_PortChangeInterrupt(0);

    if IntrSts.Interrupt() != 0 {
        WRITE_REGISTER_ULONG(IntrStsReg as *mut ULONG, IntrSts.AsULONG);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RH_EnableIrq(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let IntrStsReg: PULONG;
    let mut IntrSts: EHCI_INTERRUPT_ENABLE = core::mem::zeroed();

    DPRINT_RH!(b"EHCI_RH_EnableIrq: ... \n\0".as_ptr() as *const i8);

    IntrStsReg = core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).HcInterruptEnable.AsULONG) as PULONG;
    IntrSts.AsULONG = READ_REGISTER_ULONG(IntrStsReg as *mut ULONG);

    (*EhciExtension).InterruptMask.set_PortChangeInterrupt(1);
    IntrSts.set_PortChangeInterrupt(1);

    if IntrSts.Interrupt() != 0 {
        WRITE_REGISTER_ULONG(IntrStsReg as *mut ULONG, IntrSts.AsULONG);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB EHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBEHCI root hub functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbehci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_EHCI_ROOT_HUB
// #include "dbg_ehci.h"
//
// MPSTATUS
// NTAPI
// EHCI_RH_ChirpRootPort(IN PVOID ehciExtension,
//                       IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//     ULONG PortBit;
//     ULONG ix;
//
//     DPRINT_RH!("EHCI_RH_ChirpRootPort: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//     DPRINT_RH!("EHCI_RH_ChirpRootPort: PortSC - %X\n", PortSC.AsULONG);
//
//     PortBit = 1 << (Port - 1);
//
//     if (PortBit & EhciExtension->ResetPortBits)
//     {
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: Skip port - %x\n", Port);
//         return MP_STATUS_SUCCESS;
//     }
//
//     if (PortSC.PortPower == 0)
//     {
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: Skip port - %x\n", Port);
//         return MP_STATUS_SUCCESS;
//     }
//
//     if (PortSC.CurrentConnectStatus == 0 ||
//         PortSC.PortEnabledDisabled == 1 ||
//         PortSC.PortOwner == EHCI_PORT_OWNER_COMPANION_CONTROLLER)
//     {
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: No port - %x\n", Port);
//         return MP_STATUS_SUCCESS;
//     }
//
//     if (PortSC.LineStatus == EHCI_LINE_STATUS_K_STATE_LOW_SPEED &&
//         PortSC.Suspend == 0 &&
//         PortSC.CurrentConnectStatus == 1)
//     {
//         /* Attached device is not a high-speed device.
//            Release ownership of the port to a selected HC.
//            Companion HC owns and controls the port. Section 4.2 */
//         PortSC.PortOwner = EHCI_PORT_OWNER_COMPANION_CONTROLLER;
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: Companion HC port - %x\n", Port);
//         return MP_STATUS_SUCCESS;
//     }
//
//     DPRINT("EHCI_RH_ChirpRootPort: EhciExtension - %p, Port - %x\n",
//            EhciExtension,
//            Port);
//
//     PortSC.PortEnabledDisabled = 0;
//     PortSC.PortReset = 1;
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     RegPacket.UsbPortWait(EhciExtension, 10);
//
//     do
//     {
//         PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//         PortSC.ConnectStatusChange = 0;
//         PortSC.PortEnableDisableChange = 0;
//         PortSC.OverCurrentChange = 0;
//         PortSC.PortReset = 0;
//
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//         for (ix = 0; ix <= 500; ix += 20)
//         {
//              KeStallExecutionProcessor(20);
//              PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//              DPRINT_RH!("EHCI_RH_ChirpRootPort: Reset port - %x\n", Port);
//
//              if (PortSC.PortReset == 0)
//                  break;
//         }
//     }
//     while (PortSC.PortReset == 1);
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     if (PortSC.PortEnabledDisabled == 1)
//     {
//         PortSC.ConnectStatusChange = 0;
//         PortSC.PortEnabledDisabled = 0;
//         PortSC.PortEnableDisableChange = 0;
//         PortSC.OverCurrentChange = 0;
//
//         RegPacket.UsbPortWait(EhciExtension, 10);
//
//         EhciExtension->ResetPortBits |= PortBit;
//
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: Disable port - %x\n", Port);
//     }
//     else
//     {
//         PortSC.PortOwner = EHCI_PORT_OWNER_COMPANION_CONTROLLER;
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//         DPRINT_RH!("EHCI_RH_ChirpRootPort: Companion HC port - %x\n", Port);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_RH_GetRootHubData(IN PVOID ehciExtension,
//                        IN PVOID rootHubData)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PUSBPORT_ROOT_HUB_DATA RootHubData;
//     USBPORT_HUB_20_CHARACTERISTICS HubCharacteristics;
//
//     DPRINT_RH!("EHCI_RH_GetRootHubData: EhciExtension - %p, rootHubData - %p\n",
//               EhciExtension,
//               rootHubData);
//
//     RootHubData = rootHubData;
//
//     RootHubData->NumberOfPorts = EhciExtension->NumberOfPorts;
//
//     HubCharacteristics.AsUSHORT = 0;
//
//     /* Logical Power Switching Mode */
//     if (EhciExtension->PortPowerControl == 1)
//     {
//         /* Individual port power switching */
//         HubCharacteristics.PowerControlMode = 1;
//     }
//     else
//     {
//         /* Ganged power switching (all ports' power at once) */
//         HubCharacteristics.PowerControlMode = 0;
//     }
//
//     HubCharacteristics.NoPowerSwitching = 0;
//
//     /* EHCI RH is not part of a compound device */
//     HubCharacteristics.PartOfCompoundDevice = 0;
//
//     /* Global Over-current Protection */
//     HubCharacteristics.OverCurrentProtectionMode = 0;
//
//     RootHubData->HubCharacteristics.Usb20HubCharacteristics = HubCharacteristics;
//
//     RootHubData->PowerOnToPowerGood = 2; // Time (in 2 ms intervals)
//     RootHubData->HubControlCurrent = 0;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_GetStatus(IN PVOID ehciExtension,
//                   IN PUSHORT Status)
// {
//     DPRINT_RH!("EHCI_RH_GetStatus: ... \n");
//     *Status = USB_GETSTATUS_SELF_POWERED;
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_GetPortStatus(IN PVOID ehciExtension,
//                       IN USHORT Port,
//                       IN PUSB_PORT_STATUS_AND_CHANGE PortStatus)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//     USB_PORT_STATUS_AND_CHANGE status;
//     ULONG PortMaskBits;
//
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     if (PortSC.CurrentConnectStatus)
//     {
//         DPRINT_RH!("EHCI_RH_GetPortStatus: Port - %x, PortSC.AsULONG - %X\n",
//                   Port,
//                   PortSC.AsULONG);
//     }
//
//     PortStatus->AsUlong32 = 0;
//
//     if (PortSC.LineStatus == EHCI_LINE_STATUS_K_STATE_LOW_SPEED &&
//         PortSC.PortOwner != EHCI_PORT_OWNER_COMPANION_CONTROLLER &&
//         (PortSC.PortEnabledDisabled | PortSC.Suspend) && // Enable or Suspend
//         PortSC.CurrentConnectStatus == 1) // Device is present
//     {
//         DPRINT("EHCI_RH_GetPortStatus: LowSpeed device detected\n");
//         PortSC.PortOwner = EHCI_PORT_OWNER_COMPANION_CONTROLLER; // release ownership
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//         return MP_STATUS_SUCCESS;
//     }
//
//     status.AsUlong32 = 0;
//
//     status.PortStatus.Usb20PortStatus.CurrentConnectStatus = PortSC.CurrentConnectStatus;
//     status.PortStatus.Usb20PortStatus.PortEnabledDisabled = PortSC.PortEnabledDisabled;
//     status.PortStatus.Usb20PortStatus.Suspend = PortSC.Suspend;
//     status.PortStatus.Usb20PortStatus.OverCurrent = PortSC.OverCurrentActive;
//     status.PortStatus.Usb20PortStatus.Reset = PortSC.PortReset;
//     status.PortStatus.Usb20PortStatus.PortPower = PortSC.PortPower;
//     if (PortSC.PortOwner == EHCI_PORT_OWNER_COMPANION_CONTROLLER)
//         status.PortStatus.Usb20PortStatus.Reserved1 = USB20_PORT_STATUS_RESERVED1_OWNED_BY_COMPANION;
//
//     status.PortChange.Usb20PortChange.PortEnableDisableChange = PortSC.PortEnableDisableChange;
//     status.PortChange.Usb20PortChange.OverCurrentIndicatorChange = PortSC.OverCurrentChange;
//
//     PortMaskBits = 1 << (Port - 1);
//
//     if (status.PortStatus.Usb20PortStatus.CurrentConnectStatus)
//         status.PortStatus.Usb20PortStatus.LowSpeedDeviceAttached = 0;
//
//     status.PortStatus.Usb20PortStatus.HighSpeedDeviceAttached = 1;
//
//     if (PortSC.ConnectStatusChange)
//         EhciExtension->ConnectPortBits |= PortMaskBits;
//
//     if (EhciExtension->FinishResetPortBits & PortMaskBits)
//         status.PortChange.Usb20PortChange.ResetChange = 1;
//
//     if (EhciExtension->ConnectPortBits & PortMaskBits)
//         status.PortChange.Usb20PortChange.ConnectStatusChange = 1;
//
//     if (EhciExtension->SuspendPortBits & PortMaskBits)
//         status.PortChange.Usb20PortChange.SuspendChange = 1;
//
//     *PortStatus = status;
//
//     if (status.PortStatus.Usb20PortStatus.CurrentConnectStatus)
//     {
//         DPRINT_RH!("EHCI_RH_GetPortStatus: Port - %x, status.AsULONG - %X\n",
//                   Port,
//                   status.AsUlong32);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_GetHubStatus(IN PVOID ehciExtension,
//                      IN PUSB_HUB_STATUS_AND_CHANGE HubStatus)
// {
//     DPRINT_RH!("EHCI_RH_GetHubStatus: ... \n");
//     HubStatus->AsUlong32 = 0;
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_RH_FinishReset(IN PVOID ehciExtension,
//                     IN PVOID Context)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//     PUSHORT Port = Context;
//
//     DPRINT("EHCI_RH_FinishReset: *Port - %x\n", *Port);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[*Port - 1].AsULONG;
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     if (PortSC.AsULONG != -1)
//     {
//         if (!PortSC.CurrentConnectStatus)
//             DPRINT("EHCI_RH_FinishReset: PortSC.AsULONG - %X\n", PortSC.AsULONG);
//
//         if (PortSC.PortEnabledDisabled ||
//             !PortSC.CurrentConnectStatus ||
//             PortSC.ConnectStatusChange)
//         {
//             EhciExtension->FinishResetPortBits |= (1 << (*Port - 1));
//             RegPacket.UsbPortInvalidateRootHub(EhciExtension);
//         }
//         else
//         {
//             PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//             PortSC.PortOwner = EHCI_PORT_OWNER_COMPANION_CONTROLLER;
//             WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//             EhciExtension->FinishResetPortBits |= (1 << (*Port - 1));
//         }
//
//         EhciExtension->ResetPortBits &= ~(1 << (*Port - 1));
//     }
// }
//
// VOID
// NTAPI
// EHCI_RH_PortResetComplete(IN PVOID ehciExtension,
//                           IN PVOID Context)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//     ULONG ix;
//     PUSHORT Port = Context;
//
//     DPRINT("EHCI_RH_PortResetComplete: *Port - %x\n", *Port);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[*Port - 1].AsULONG;
//
//     do
//     {
//         PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//         PortSC.ConnectStatusChange = 0;
//         PortSC.PortEnableDisableChange = 0;
//         PortSC.OverCurrentChange = 0;
//         PortSC.PortReset = 0;
//
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//         for (ix = 0; ix <= 500; ix += 20)
//         {
//              KeStallExecutionProcessor(20);
//              PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//              DPRINT("EHCI_RH_PortResetComplete: Reset port - %x\n", Port);
//
//              if (PortSC.PortReset == 0)
//                  break;
//         }
//     }
//     while (PortSC.PortReset == 1 && (PortSC.AsULONG != -1));
//
//     RegPacket.UsbPortRequestAsyncCallback(EhciExtension,
//                                           50, // TimerValue
//                                           Port,
//                                           sizeof(*Port),
//                                           EHCI_RH_FinishReset);
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_SetFeaturePortReset(IN PVOID ehciExtension,
//                             IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_SetFeaturePortReset: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     EhciExtension->ResetPortBits |= 1 << (Port - 1);
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnabledDisabled = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 0;
//     PortSC.PortReset = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     RegPacket.UsbPortRequestAsyncCallback(EhciExtension,
//                                           50, // TimerValue
//                                           &Port,
//                                           sizeof(Port),
//                                           EHCI_RH_PortResetComplete);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_SetFeaturePortPower(IN PVOID ehciExtension,
//                             IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT_RH!("EHCI_RH_SetFeaturePortPower: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 0;
//     PortSC.PortPower = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_SetFeaturePortEnable(IN PVOID ehciExtension,
//                              IN USHORT Port)
// {
//     DPRINT_RH!("EHCI_RH_SetFeaturePortEnable: Not supported\n");
//     ASSERT(Port != 0);
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_SetFeaturePortSuspend(IN PVOID ehciExtension,
//                               IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_SetFeaturePortSuspend: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 0;
//     PortSC.Suspend = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//     KeStallExecutionProcessor(125);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortEnable(IN PVOID ehciExtension,
//                                IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_ClearFeaturePortEnable: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnabledDisabled = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 0;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortPower(IN PVOID ehciExtension,
//                               IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_ClearFeaturePortPower: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//     PortSC.PortPower = 0;
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_RH_PortResumeComplete(IN PVOID ehciExtension,
//                            IN PVOID Context)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//     PUSHORT Port = Context;
//
//     DPRINT("EHCI_RH_PortResumeComplete: *Port - %x\n", *Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[*Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 0;
//     PortSC.ForcePortResume = 0;
//     PortSC.Suspend = 0;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//     READ_REGISTER_ULONG(PortStatusReg);
//
//     EhciExtension->SuspendPortBits |= 1 << (*Port - 1);
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortSuspend(IN PVOID ehciExtension,
//                                 IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_ClearFeaturePortSuspend: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//     EhciExtension->ResetPortBits |= 1 << (Port - 1);
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//     PortSC.ForcePortResume = 1;
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     RegPacket.UsbPortRequestAsyncCallback(EhciExtension,
//                                           50, // TimerValue
//                                           &Port,
//                                           sizeof(Port),
//                                           EHCI_RH_PortResumeComplete);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortEnableChange(IN PVOID ehciExtension,
//                                      IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT("EHCI_RH_ClearFeaturePortEnableChange: Port - %p\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.OverCurrentChange = 0;
//     PortSC.PortEnableDisableChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortConnectChange(IN PVOID ehciExtension,
//                                       IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT_RH!("EHCI_RH_ClearFeaturePortConnectChange: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     if (PortSC.ConnectStatusChange)
//     {
//         PortSC.ConnectStatusChange = 1;
//         PortSC.PortEnableDisableChange = 0;
//         PortSC.OverCurrentChange = 0;
//
//         WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//     }
//
//     EhciExtension->ConnectPortBits &= ~(1 << (Port - 1));
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortResetChange(IN PVOID ehciExtension,
//                                     IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     DPRINT("EHCI_RH_ClearFeaturePortConnectChange: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     EhciExtension->FinishResetPortBits &= ~(1 << (Port - 1));
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortSuspendChange(IN PVOID ehciExtension,
//                                       IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     DPRINT("EHCI_RH_ClearFeaturePortSuspendChange: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     EhciExtension->SuspendPortBits &= ~(1 << (Port - 1));
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_RH_ClearFeaturePortOvercurrentChange(IN PVOID ehciExtension,
//                                           IN USHORT Port)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG PortStatusReg;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT_RH!("EHCI_RH_ClearFeaturePortOvercurrentChange: Port - %x\n", Port);
//     ASSERT(Port != 0);
//
//     PortStatusReg = &EhciExtension->OperationalRegs->PortControl[Port - 1].AsULONG;
//
//     PortSC.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//     PortSC.ConnectStatusChange = 0;
//     PortSC.PortEnableDisableChange = 0;
//     PortSC.OverCurrentChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortSC.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_RH_DisableIrq(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG IntrStsReg;
//     EHCI_INTERRUPT_ENABLE IntrSts;
//
//     DPRINT_RH!("EHCI_RH_DisableIrq: ... \n");
//
//     IntrStsReg = &EhciExtension->OperationalRegs->HcInterruptEnable.AsULONG;
//     IntrSts.AsULONG = READ_REGISTER_ULONG(IntrStsReg);
//
//     EhciExtension->InterruptMask.PortChangeInterrupt = 0;
//     IntrSts.PortChangeInterrupt = 0;
//
//     if (IntrSts.Interrupt)
//         WRITE_REGISTER_ULONG(IntrStsReg, IntrSts.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_RH_EnableIrq(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PULONG IntrStsReg;
//     EHCI_INTERRUPT_ENABLE IntrSts;
//
//     DPRINT_RH!("EHCI_RH_EnableIrq: ... \n");
//
//     IntrStsReg = &EhciExtension->OperationalRegs->HcInterruptEnable.AsULONG;
//     IntrSts.AsULONG = READ_REGISTER_ULONG(IntrStsReg);
//
//     EhciExtension->InterruptMask.PortChangeInterrupt = 1;
//     IntrSts.PortChangeInterrupt = 1;
//
//     if (IntrSts.Interrupt)
//         WRITE_REGISTER_ULONG(IntrStsReg, IntrSts.AsULONG);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
