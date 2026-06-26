use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}
extern "system" {
    fn RtlAssert( FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR, );
}

// ---- file-scope preamble (translated) ----
// #define NDEBUG  (no value; skipped — function-like/marker macro)

const OHCI_MAX_PORT_COUNT: ::core::ffi::c_int = 15;

const USB_GETSTATUS_SELF_POWERED: u32 = 0x0001;

const MP_STATUS_SUCCESS: MPSTATUS = 0;

#[no_mangle]
pub unsafe extern "system" fn OHCI_ReadRhDescriptorA(
    mut OhciExtension: POHCI_EXTENSION,
) -> OHCI_REG_RH_DESCRIPTORA {
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut DescriptorA: OHCI_REG_RH_DESCRIPTORA = core::mem::zeroed();
    let mut DescriptorAReg: PULONG;
    let mut ix: ULONG;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    DescriptorAReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhDescriptorA) as PULONG;

    ix = 0;
    while ix < 10 {
        DescriptorA.AsULONG = READ_REGISTER_ULONG(DescriptorAReg as *mut ULONG);

        if DescriptorA.AsULONG != 0
            && DescriptorA.c2rust_unnamed.Reserved() as ::core::ffi::c_int == 0
            && DescriptorA.c2rust_unnamed.NumberDownstreamPorts() as ::core::ffi::c_int
                <= OHCI_MAX_PORT_COUNT
        {
            break;
        }

        if DbgPrint(
            b"(%s:%d) OHCI_ReadRhDescriptorA: DescriptorA - %lX, ix - %d\n\0" as *const u8
                as PCSTR,
            b"roothub.c\0" as *const u8 as *const ::core::ffi::c_char,
            39 as ::core::ffi::c_int,
            DescriptorA.AsULONG,
            ix,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"roothub.c\0" as *const u8 as *const ::core::ffi::c_char,
                39 as ::core::ffi::c_int,
            );
        }

        KeStallExecutionProcessor(5 as ULONG);
        ix = ix.wrapping_add(1);
    }

    return DescriptorA;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_GetRootHubData(ohciExtension: PVOID, rootHubData: PVOID) {
    let OhciExtension: POHCI_EXTENSION;
    let RootHubData: PUSBPORT_ROOT_HUB_DATA;
    let DescriptorA: OHCI_REG_RH_DESCRIPTORA;
    let mut PowerOnToPowerGoodTime: UCHAR;
    let mut HubCharacteristics: USBPORT_HUB_11_CHARACTERISTICS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    RootHubData = rootHubData as PUSBPORT_ROOT_HUB_DATA;
    DescriptorA = OHCI_ReadRhDescriptorA(OhciExtension);

    (*RootHubData).NumberOfPorts = DescriptorA.c2rust_unnamed.NumberDownstreamPorts();

    /* Waiting time (in 2 ms intervals) */
    PowerOnToPowerGoodTime = DescriptorA.c2rust_unnamed.PowerOnToPowerGoodTime() as UCHAR;
    if PowerOnToPowerGoodTime as i32 <= (OHCI_MINIMAL_POTPGT as i32) {
        PowerOnToPowerGoodTime = OHCI_MINIMAL_POTPGT as UCHAR;
    }
    (*RootHubData).PowerOnToPowerGood = PowerOnToPowerGoodTime as ULONG;

    HubCharacteristics.AsUSHORT = 0 as USHORT;

    if DescriptorA.c2rust_unnamed.PowerSwitchingMode() != 0 {
        /* Individual port power switching */
        HubCharacteristics
            .c2rust_unnamed
            .set_PowerControlMode(1 as USHORT);
    } else {
        /* Ganged power switching */
        HubCharacteristics
            .c2rust_unnamed
            .set_PowerControlMode(0 as USHORT);
    }

    HubCharacteristics
        .c2rust_unnamed
        .set_NoPowerSwitching(0 as USHORT);

    /* always 0 (OHCI RH is not a compound device) */
    if !(DescriptorA.c2rust_unnamed.DeviceType() as i32 == 0 as i32) {
        RtlAssert(
            b"DescriptorA.DeviceType == 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbohci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            91 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    HubCharacteristics
        .c2rust_unnamed
        .set_PartOfCompoundDevice(DescriptorA.c2rust_unnamed.DeviceType() as USHORT);

    HubCharacteristics
        .c2rust_unnamed
        .set_OverCurrentProtectionMode(
            DescriptorA.c2rust_unnamed.OverCurrentProtectionMode() as USHORT,
        );
    HubCharacteristics
        .c2rust_unnamed
        .set_NoOverCurrentProtection(
            DescriptorA.c2rust_unnamed.NoOverCurrentProtection() as USHORT,
        );

    (*RootHubData).HubCharacteristics.Usb11HubCharacteristics = HubCharacteristics;
    (*RootHubData).HubControlCurrent = 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_GetStatus(
    ohciExtension: PVOID,
    Status: PUSHORT,
) -> MPSTATUS {
    *Status = USB_GETSTATUS_SELF_POWERED as core::ffi::c_ushort;
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_GetPortStatus(
    ohciExtension: PVOID,
    Port: USHORT,
    PortStatus: PUSB_PORT_STATUS_AND_CHANGE,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut OhciPortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();
    let mut ix: ULONG;
    let mut Reserved: ULONG;

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (&raw mut (*OperationalRegs).HcRhPortStatus
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1) as isize) as PULONG;

    ix = 0;
    while ix < 10 {
        OhciPortStatus.AsULONG = READ_REGISTER_ULONG(PortStatusReg as *mut ULONG);

        Reserved = (OhciPortStatus.Reserved1r() as i32
            | OhciPortStatus.Reserved2r() as i32
            | OhciPortStatus.c2rust_unnamed.Reserved3() as i32) as ULONG;

        if OhciPortStatus.AsULONG != 0 && Reserved == 0 {
            break;
        }

        KeStallExecutionProcessor(5);

        ix = ix.wrapping_add(1);
    }

    (*PortStatus).AsUlong32 = OhciPortStatus.AsULONG;

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_GetHubStatus(
    ohciExtension: PVOID,
    HubStatus: PUSB_HUB_STATUS_AND_CHANGE,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut RhStatusReg: PULONG;
    let mut HcRhStatus: OHCI_REG_RH_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    RhStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhStatus) as PULONG;

    HcRhStatus.AsULONG = READ_REGISTER_ULONG(RhStatusReg as *mut ULONG);

    (*HubStatus).HubStatus.set_LocalPowerLost(HcRhStatus.LocalPowerStatus() as USHORT);
    (*HubStatus).HubChange.set_LocalPowerChange(HcRhStatus.LocalPowerStatusChange() as USHORT);

    (*HubStatus).HubStatus.set_OverCurrent(HcRhStatus.OverCurrentIndicator() as USHORT);
    (*HubStatus).HubChange.set_OverCurrentChange(HcRhStatus.OverCurrentIndicatorChangeR() as USHORT);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_SetFeaturePortReset(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as i32 > 0 as i32) {
        RtlAssert(
            b"Port > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            203 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1 as i32) as isize) as *mut OHCI_REG_RH_PORT_STATUS
        as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_SetPortReset(1 as USHORT as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_SetFeaturePortPower(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as i32 > 0) {
        RtlAssert(
            b"Port > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            232 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1) as isize) as *mut OHCI_REG_RH_PORT_STATUS
        as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_SetPortPower(1 as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_SetFeaturePortEnable(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    // DPRINT("OHCI_RH_SetFeaturePortEnable: OhciExtension - %p, Port - %x\n", OhciExtension, Port);

    if !(Port as i32 > 0 as i32) {
        RtlAssert(
            b"Port > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            261 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1 as i32) as isize) as *mut OHCI_REG_RH_PORT_STATUS
        as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_SetPortEnable(1 as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_SetFeaturePortSuspend(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as i32 > 0 as i32) {
        RtlAssert(
            b"Port > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            290 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1 as i32) as isize)
        as *mut OHCI_REG_RH_PORT_STATUS as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_SetPortSuspend(1 as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortEnable(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as i32 > 0 as i32) {
        RtlAssert(
            b"Port > 0\0" as *const u8 as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0"
                as *const u8 as *const i8 as PVOID,
            319 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1 as i32) as isize) as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_ClearPortEnable(1 as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortPower(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {

    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    // DPRINT("OHCI_RH_ClearFeaturePortPower: OhciExtension - %p, Port - %x\n", OhciExtension, Port);

    if !(Port as i32 > 0 as i32) {
        RtlAssert(
            b"Port > 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbohci/roothub.c\0".as_ptr() as *const i8 as PVOID,
            348 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1 as i32) as isize)
        as *mut OHCI_REG_RH_PORT_STATUS as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus.set_ClearPortPower(1 as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortSuspend(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus[(Port - 1) as usize]) as PULONG;

    PortStatus.AsULONG = 0;
    PortStatus.set_ClearSuspendStatus(1);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortEnableChange(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as core::ffi::c_int > 0 as core::ffi::c_int) {
        RtlAssert(
            b"Port > 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbohci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            406 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as core::ffi::c_int - 1 as core::ffi::c_int) as isize)
        as *mut OHCI_REG_RH_PORT_STATUS as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus
        .c2rust_unnamed
        .set_PortEnableStatusChange(1 as USHORT as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortConnectChange(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1) as isize) as PULONG;

    PortStatus.AsULONG = 0;
    PortStatus.set_ConnectStatusChange(1);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortResetChange(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    // DPRINT("OHCI_RH_ClearFeaturePortResetChange: OhciExtension - %p, Port - %x\n", OhciExtension, Port);

    // ASSERT(Port > 0);

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as i32 - 1) as isize) as PULONG;

    PortStatus.AsULONG = 0;
    PortStatus.set_PortResetStatusChange(1);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortSuspendChange(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut PortStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    if !(Port as core::ffi::c_int > 0 as core::ffi::c_int) {
        RtlAssert(
            b"Port > 0\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbohci/roothub.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            493 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    }

    OperationalRegs = (*OhciExtension).OperationalRegs;
    PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus)
        as *mut OHCI_REG_RH_PORT_STATUS)
        .offset((Port as core::ffi::c_int - 1 as core::ffi::c_int) as isize)
        as *mut OHCI_REG_RH_PORT_STATUS as PULONG;

    PortStatus.AsULONG = 0 as ULONG;
    PortStatus
        .c2rust_unnamed
        .set_PortSuspendStatusChange(1 as USHORT as USHORT);

    WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_ClearFeaturePortOvercurrentChange(
    ohciExtension: PVOID,
    Port: USHORT,
) -> MPSTATUS {
    let mut OhciExtension: POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let PortStatusReg: PULONG;
    let RhStatusReg: PULONG;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();
    let mut RhStatus: OHCI_REG_RH_STATUS = core::mem::zeroed();

    OhciExtension = ohciExtension as POHCI_EXTENSION;

    OperationalRegs = (*OhciExtension).OperationalRegs;

    if Port != 0 {
        /* USBPORT_RECIPIENT_PORT */
        PortStatus.AsULONG = 0;
        PortStatus.set_PortOverCurrentIndicatorChange(1);

        PortStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus[(Port - 1) as usize]) as PULONG;
        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);
    } else {
        /* USBPORT_RECIPIENT_HUB */
        RhStatus.AsULONG = 0;
        RhStatus.set_OverCurrentIndicatorChangeW(1);

        RhStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhStatus) as PULONG;
        WRITE_REGISTER_ULONG(RhStatusReg as *mut ULONG, RhStatus.AsULONG);
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_DisableIrq(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let InterruptDisableReg: PULONG;
    let mut InterruptDisable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    OperationalRegs = (*OhciExtension).OperationalRegs;
    InterruptDisableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as PULONG;

    InterruptDisable.AsULONG = 0 as ULONG;
    InterruptDisable.c2rust_unnamed.set_RootHubStatusChange(1 as ULONG);

    WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, InterruptDisable.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RH_EnableIrq(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let InterruptEnableReg: PULONG;
    let mut InterruptEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    OperationalRegs = (*OhciExtension).OperationalRegs;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;

    InterruptEnable.AsULONG = 0 as ULONG;
    InterruptEnable.set_RootHubStatusChange(1 as ULONG);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, InterruptEnable.AsULONG);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB OHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBOHCI root hub functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbohci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// OHCI_REG_RH_DESCRIPTORA
// NTAPI
// OHCI_ReadRhDescriptorA(IN POHCI_EXTENSION OhciExtension)
// {
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     OHCI_REG_RH_DESCRIPTORA DescriptorA;
//     PULONG DescriptorAReg;
//     ULONG ix;
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     DescriptorAReg = (PULONG)&OperationalRegs->HcRhDescriptorA;
//
//     DPRINT("OHCI_ReadRhDescriptorA: OhciExtension - %p\n", OhciExtension);
//
//     for (ix = 0; ix < 10; ix++)
//     {
//         DescriptorA.AsULONG = READ_REGISTER_ULONG(DescriptorAReg);
//
//         if (DescriptorA.AsULONG != 0 &&
//             DescriptorA.Reserved == 0 &&
//             DescriptorA.NumberDownstreamPorts <= OHCI_MAX_PORT_COUNT)
//         {
//             break;
//         }
//
//         DPRINT1("OHCI_ReadRhDescriptorA: DescriptorA - %lX, ix - %d\n",
//                 DescriptorA.AsULONG, ix);
//
//         KeStallExecutionProcessor(5);
//     }
//
//     return DescriptorA;
// }
//
// VOID
// NTAPI
// OHCI_RH_GetRootHubData(IN PVOID ohciExtension,
//                        IN PVOID rootHubData)
// {
//     POHCI_EXTENSION OhciExtension;
//     PUSBPORT_ROOT_HUB_DATA RootHubData;
//     OHCI_REG_RH_DESCRIPTORA DescriptorA;
//     UCHAR PowerOnToPowerGoodTime;
//     USBPORT_HUB_11_CHARACTERISTICS HubCharacteristics;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_GetRootHubData: OhciExtension - %p, rootHubData - %p\n",
//            OhciExtension,
//            rootHubData);
//
//     RootHubData = rootHubData;
//     DescriptorA = OHCI_ReadRhDescriptorA(OhciExtension);
//
//     RootHubData->NumberOfPorts = DescriptorA.NumberDownstreamPorts;
//
//     /* Waiting time (in 2 ms intervals) */
//     PowerOnToPowerGoodTime = DescriptorA.PowerOnToPowerGoodTime;
//     if (PowerOnToPowerGoodTime <= OHCI_MINIMAL_POTPGT)
//         PowerOnToPowerGoodTime = OHCI_MINIMAL_POTPGT;
//     RootHubData->PowerOnToPowerGood = PowerOnToPowerGoodTime;
//
//     HubCharacteristics.AsUSHORT = 0;
//
//     if (DescriptorA.PowerSwitchingMode)
//     {
//         /* Individual port power switching */
//         HubCharacteristics.PowerControlMode = 1;
//     }
//     else
//     {
//         /* Ganged power switching */
//         HubCharacteristics.PowerControlMode = 0;
//     }
//
//     HubCharacteristics.NoPowerSwitching = 0;
//
//     /* always 0 (OHCI RH is not a compound device) */
//     ASSERT(DescriptorA.DeviceType == 0);
//     HubCharacteristics.PartOfCompoundDevice = DescriptorA.DeviceType;
//
//     HubCharacteristics.OverCurrentProtectionMode = DescriptorA.OverCurrentProtectionMode;
//     HubCharacteristics.NoOverCurrentProtection = DescriptorA.NoOverCurrentProtection;
//
//     RootHubData->HubCharacteristics.Usb11HubCharacteristics = HubCharacteristics;
//     RootHubData->HubControlCurrent = 0;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_GetStatus(IN PVOID ohciExtension,
//                   IN PUSHORT Status)
// {
//     DPRINT("OHCI_RH_GetStatus: \n");
//     *Status = USB_GETSTATUS_SELF_POWERED;
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_GetPortStatus(IN PVOID ohciExtension,
//                       IN USHORT Port,
//                       IN PUSB_PORT_STATUS_AND_CHANGE PortStatus)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS OhciPortStatus;
//     ULONG ix;
//     ULONG Reserved;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_GetPortStatus: OhciExtension - %p, Port - %x, PortStatus - %lX\n",
//            OhciExtension,
//            Port,
//            PortStatus->AsUlong32);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     for (ix = 0; ix < 10; ix++)
//     {
//         OhciPortStatus.AsULONG = READ_REGISTER_ULONG(PortStatusReg);
//
//         Reserved = OhciPortStatus.Reserved1r |
//                    OhciPortStatus.Reserved2r |
//                    OhciPortStatus.Reserved3;
//
//         if (OhciPortStatus.AsULONG && !Reserved)
//             break;
//
//         DPRINT("OHCI_RH_GetPortStatus: OhciPortStatus - %X\n", OhciPortStatus.AsULONG);
//
//         KeStallExecutionProcessor(5);
//     }
//
//     PortStatus->AsUlong32 = OhciPortStatus.AsULONG;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_GetHubStatus(IN PVOID ohciExtension,
//                      IN PUSB_HUB_STATUS_AND_CHANGE HubStatus)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG RhStatusReg;
//     OHCI_REG_RH_STATUS HcRhStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_GetHubStatus: ohciExtension - %p, HubStatus - %lX\n",
//            ohciExtension,
//            HubStatus->AsUlong32);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     RhStatusReg = (PULONG)&OperationalRegs->HcRhStatus;
//
//     HcRhStatus.AsULONG = READ_REGISTER_ULONG(RhStatusReg);
//
//     HubStatus->HubStatus.LocalPowerLost = HcRhStatus.LocalPowerStatus;
//     HubStatus->HubChange.LocalPowerChange = HcRhStatus.LocalPowerStatusChange;
//
//     HubStatus->HubStatus.OverCurrent = HcRhStatus.OverCurrentIndicator;
//     HubStatus->HubChange.OverCurrentChange = HcRhStatus.OverCurrentIndicatorChangeR;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_SetFeaturePortReset(IN PVOID ohciExtension,
//                             IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_SetFeaturePortReset: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.SetPortReset = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_SetFeaturePortPower(IN PVOID ohciExtension,
//                             IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_SetFeaturePortPower: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.SetPortPower = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_SetFeaturePortEnable(IN PVOID ohciExtension,
//                              IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_SetFeaturePortEnable: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.SetPortEnable = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_SetFeaturePortSuspend(IN PVOID ohciExtension,
//                               IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_SetFeaturePortSuspend: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.SetPortSuspend = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortEnable(IN PVOID ohciExtension,
//                                IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortEnable: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.ClearPortEnable = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortPower(IN PVOID ohciExtension,
//                               IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortPower: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.ClearPortPower = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortSuspend(IN PVOID ohciExtension,
//                                 IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortSuspend: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.ClearSuspendStatus = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortEnableChange(IN PVOID ohciExtension,
//                                      IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortEnableChange: ohciExtension - %p, Port - %x\n",
//            ohciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.PortEnableStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortConnectChange(IN PVOID ohciExtension,
//                                       IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortConnectChange: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.ConnectStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortResetChange(IN PVOID ohciExtension,
//                                     IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortResetChange: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.PortResetStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortSuspendChange(IN PVOID ohciExtension,
//                                       IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortSuspendChange: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     ASSERT(Port > 0);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//
//     PortStatus.AsULONG = 0;
//     PortStatus.PortSuspendStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_RH_ClearFeaturePortOvercurrentChange(IN PVOID ohciExtension,
//                                           IN USHORT Port)
// {
//     POHCI_EXTENSION OhciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG PortStatusReg;
//     PULONG RhStatusReg;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//     OHCI_REG_RH_STATUS RhStatus;
//
//     OhciExtension = ohciExtension;
//
//     DPRINT("OHCI_RH_ClearFeaturePortOvercurrentChange: OhciExtension - %p, Port - %x\n",
//            OhciExtension,
//            Port);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     if (Port)
//     {
//         /* USBPORT_RECIPIENT_PORT */
//         PortStatus.AsULONG = 0;
//         PortStatus.PortOverCurrentIndicatorChange = 1;
//
//         PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[Port-1];
//         WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//     }
//     else
//     {
//         /* USBPORT_RECIPIENT_HUB */
//         RhStatus.AsULONG = 0;
//         RhStatus.OverCurrentIndicatorChangeW = 1;
//
//         RhStatusReg = (PULONG)&OperationalRegs->HcRhStatus;
//         WRITE_REGISTER_ULONG(RhStatusReg, RhStatus.AsULONG);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// OHCI_RH_DisableIrq(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION  OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptDisableReg;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE InterruptDisable;
//
//     DPRINT("OHCI_RH_DisableIrq: OhciExtension - %p\n", OhciExtension);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     InterruptDisableReg = (PULONG)&OperationalRegs->HcInterruptDisable;
//
//     InterruptDisable.AsULONG = 0;
//     InterruptDisable.RootHubStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(InterruptDisableReg, InterruptDisable.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_RH_EnableIrq(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptEnableReg;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE InterruptEnable;
//
//     DPRINT("OHCI_RH_EnableIrq: OhciExtension - %p\n", OhciExtension);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//
//     InterruptEnable.AsULONG = 0;
//     InterruptEnable.RootHubStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, InterruptEnable.AsULONG);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
