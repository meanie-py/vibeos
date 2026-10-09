use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KeQueryTimeIncrement, KeStallExecutionProcessor, LARGE_INTEGER, PDRIVER_OBJECT, PUNICODE_STRING, RtlSecureZeroMemory, UINT32};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}
extern "system" {
    fn RtlAssert( VoidFailedAssertion: PVOID, VoidFileName: PVOID, LineNumber: ULONG, MutableMessage: *mut CHAR, );
    fn OHCI_ReadRhDescriptorA(OhciExtension: POHCI_EXTENSION) -> OHCI_REG_RH_DESCRIPTORA;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut RegPacket: USBPORT_REGISTRATION_PACKET = USBPORT_REGISTRATION_PACKET {
    MiniPortVersion: 0,
    MiniPortFlags: 0,
    MiniPortBusBandwidth: 0,
    Reserved1: 0,
    MiniPortExtensionSize: 0,
    MiniPortEndpointSize: 0,
    MiniPortTransferSize: 0,
    Reserved2: 0,
    Reserved3: 0,
    MiniPortResourcesSize: 0,
    OpenEndpoint: None,
    ReopenEndpoint: None,
    QueryEndpointRequirements: None,
    CloseEndpoint: None,
    StartController: None,
    StopController: None,
    SuspendController: None,
    ResumeController: None,
    InterruptService: None,
    InterruptDpc: None,
    SubmitTransfer: None,
    SubmitIsoTransfer: None,
    AbortTransfer: None,
    GetEndpointState: None,
    SetEndpointState: None,
    PollEndpoint: None,
    CheckController: None,
    Get32BitFrameNumber: None,
    InterruptNextSOF: None,
    EnableInterrupts: None,
    DisableInterrupts: None,
    PollController: None,
    SetEndpointDataToggle: None,
    GetEndpointStatus: None,
    SetEndpointStatus: None,
    ResetController: None,
    RH_GetRootHubData: None,
    RH_GetStatus: None,
    RH_GetPortStatus: None,
    RH_GetHubStatus: None,
    RH_SetFeaturePortReset: None,
    RH_SetFeaturePortPower: None,
    RH_SetFeaturePortEnable: None,
    RH_SetFeaturePortSuspend: None,
    RH_ClearFeaturePortEnable: None,
    RH_ClearFeaturePortPower: None,
    RH_ClearFeaturePortSuspend: None,
    RH_ClearFeaturePortEnableChange: None,
    RH_ClearFeaturePortConnectChange: None,
    RH_ClearFeaturePortResetChange: None,
    RH_ClearFeaturePortSuspendChange: None,
    RH_ClearFeaturePortOvercurrentChange: None,
    RH_DisableIrq: None,
    RH_EnableIrq: None,
    StartSendOnePacket: None,
    EndSendOnePacket: None,
    PassThru: None,
    UsbPortDbgPrint: None,
    UsbPortTestDebugBreak: None,
    UsbPortAssertFailure: None,
    UsbPortGetMiniportRegistryKeyValue: None,
    UsbPortInvalidateRootHub: None,
    UsbPortInvalidateEndpoint: None,
    UsbPortCompleteTransfer: None,
    UsbPortCompleteIsoTransfer: None,
    UsbPortLogEntry: None,
    UsbPortGetMappedVirtualAddress: None,
    UsbPortRequestAsyncCallback: None,
    UsbPortReadWriteConfigSpace: None,
    UsbPortWait: None,
    UsbPortInvalidateController: None,
    UsbPortBugCheck: None,
    UsbPortNotifyDoubleBuffer: None,
    RebalanceEndpoint: None,
    FlushInterrupts: None,
    RH_ChirpRootPort: None,
    TakePortControl: None,
    Reserved4: 0,
    Reserved5: 0,
};

static mut Index: [UCHAR; 8] = [
    (ENDPOINT_INTERRUPT_1ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_2ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_4ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_8ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_16ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - (1 as ::core::ffi::c_int as u32)) as UCHAR,
];

static mut Balance: [UCHAR; 32] = [
    0 as ::core::ffi::c_int as UCHAR,
    16 as ::core::ffi::c_int as UCHAR,
    8 as ::core::ffi::c_int as UCHAR,
    24 as ::core::ffi::c_int as UCHAR,
    4 as ::core::ffi::c_int as UCHAR,
    20 as ::core::ffi::c_int as UCHAR,
    12 as ::core::ffi::c_int as UCHAR,
    28 as ::core::ffi::c_int as UCHAR,
    2 as ::core::ffi::c_int as UCHAR,
    18 as ::core::ffi::c_int as UCHAR,
    10 as ::core::ffi::c_int as UCHAR,
    26 as ::core::ffi::c_int as UCHAR,
    6 as ::core::ffi::c_int as UCHAR,
    22 as ::core::ffi::c_int as UCHAR,
    14 as ::core::ffi::c_int as UCHAR,
    30 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
    17 as ::core::ffi::c_int as UCHAR,
    9 as ::core::ffi::c_int as UCHAR,
    25 as ::core::ffi::c_int as UCHAR,
    5 as ::core::ffi::c_int as UCHAR,
    21 as ::core::ffi::c_int as UCHAR,
    13 as ::core::ffi::c_int as UCHAR,
    29 as ::core::ffi::c_int as UCHAR,
    3 as ::core::ffi::c_int as UCHAR,
    19 as ::core::ffi::c_int as UCHAR,
    11 as ::core::ffi::c_int as UCHAR,
    27 as ::core::ffi::c_int as UCHAR,
    7 as ::core::ffi::c_int as UCHAR,
    23 as ::core::ffi::c_int as UCHAR,
    15 as ::core::ffi::c_int as UCHAR,
    31 as ::core::ffi::c_int as UCHAR,
];

pub const NDEBUG: () = ();

pub const NDEBUG_OHCI_TRACE: () = ();

#[no_mangle]
pub unsafe extern "system" fn OHCI_DumpHcdED(ED: POHCI_HCD_ED) {
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_DumpHcdTD(TD: POHCI_HCD_TD) {
    if (*TD).HwTD.c2rust_unnamed.gTD.CurrentBuffer != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.gTD.NextTD != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.gTD.BufferEnd != 0 {
    }

    if (*TD).HwTD.c2rust_unnamed.SetupPacket.bmRequestType.B != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.SetupPacket.bRequest != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.SetupPacket.wValue.W != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.SetupPacket.wIndex.W != 0 {
    }
    if (*TD).HwTD.c2rust_unnamed.SetupPacket.wLength != 0 {
    }

    if (*TD).TransferLen != 0 {
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_EnableList(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
) {
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let CommandStatusReg: PULONG;
    let TransferType: ULONG;
    let mut CommandStatus: OHCI_REG_COMMAND_STATUS = core::mem::zeroed();

    DPRINT_OHCI!(b"OHCI_EnableList: ... \n\0".as_ptr() as *const i8);

    OperationalRegs = (*OhciExtension).OperationalRegs;
    CommandStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcCommandStatus) as PULONG;

    CommandStatus.AsULONG = 0;

    if READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcControlHeadED) as PULONG as *mut ULONG) != 0 {
        CommandStatus.set_ControlListFilled(1);
    }

    if READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcBulkHeadED) as PULONG as *mut ULONG) != 0 {
        CommandStatus.set_BulkListFilled(1);
    }

    TransferType = (*OhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG {
        CommandStatus.set_BulkListFilled(1);
    } else if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        CommandStatus.set_ControlListFilled(1);
    }

    WRITE_REGISTER_ULONG(CommandStatusReg as *mut ULONG, CommandStatus.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InsertEndpointInSchedule(OhciEndpoint: POHCI_ENDPOINT) {
    let HeadED: POHCI_STATIC_ED;
    let ED: POHCI_HCD_ED;
    let PrevED: POHCI_HCD_ED;
    let HeadLink: PLIST_ENTRY;

    ED = (*OhciEndpoint).HcdED;

    HeadED = (*OhciEndpoint).HeadED;
    HeadLink = core::ptr::addr_of_mut!((*HeadED).Link) as PLIST_ENTRY;

    if IsListEmpty(HeadLink) {
        InsertHeadList(HeadLink, core::ptr::addr_of_mut!((*ED).HcdEDLink));

        if (*HeadED).Type == OHCI_STATIC_ED_TYPE_CONTROL as ULONG
            || (*HeadED).Type == OHCI_STATIC_ED_TYPE_BULK as ULONG
        {
            (*ED).HwED.NextED = READ_REGISTER_ULONG((*HeadED).pNextED as *mut ULONG);
            WRITE_REGISTER_ULONG((*HeadED).pNextED as *mut ULONG, (*ED).PhysicalAddress);
        } else if (*HeadED).Type == OHCI_STATIC_ED_TYPE_INTERRUPT as ULONG {
            (*ED).HwED.NextED = *(*HeadED).pNextED as ULONG;
            *(*HeadED).pNextED = (*ED).PhysicalAddress as ULONG;
        } else {
            DbgBreakPoint();
        }
    } else {
        PrevED = ((*HeadLink).Blink as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(OHCI_HCD_ED, HcdEDLink) as ULONG_PTR)
            as *mut OHCI_HCD_ED;

        InsertTailList(HeadLink, core::ptr::addr_of_mut!((*ED).HcdEDLink));

        (*ED).HwED.NextED = 0 as ULONG;
        (*PrevED).HwED.NextED = (*ED).PhysicalAddress;
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InitializeED(
    OhciEndpoint: POHCI_ENDPOINT,
    ED: POHCI_HCD_ED,
    FirstTD: POHCI_HCD_TD,
    EdPA: ULONG_PTR,
) -> POHCI_HCD_ED {
    let mut EndpointControl: OHCI_ENDPOINT_CONTROL = core::mem::zeroed();
    let EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;

    DPRINT_OHCI!(
        b"OHCI_InitializeED: OhciEndpoint - %p, ED - %p, FirstTD - %p, EdPA - %p\n\0".as_ptr()
            as *const i8,
        OhciEndpoint,
        ED,
        FirstTD,
        EdPA,
    );

    RtlZeroMemory(ED as *mut core::ffi::c_void, core::mem::size_of::<OHCI_HCD_ED>());

    (*ED).PhysicalAddress = EdPA as ULONG;

    EndpointProperties =
        core::ptr::addr_of_mut!((*OhciEndpoint).EndpointProperties) as PUSBPORT_ENDPOINT_PROPERTIES;

    EndpointControl = (*ED).HwED.EndpointControl;

    EndpointControl.set_FunctionAddress((*EndpointProperties).DeviceAddress as ULONG);
    EndpointControl.set_EndpointNumber((*EndpointProperties).EndpointAddress as ULONG);
    EndpointControl.set_MaximumPacketSize((*EndpointProperties).TotalMaxPacketSize as ULONG);

    if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        EndpointControl.set_Direction(OHCI_ED_DATA_FLOW_DIRECTION_FROM_TD as ULONG);
    } else if (*EndpointProperties).Direction != 0 {
        EndpointControl.set_Direction(OHCI_ED_DATA_FLOW_DIRECTION_OUT as ULONG);
    } else {
        EndpointControl.set_Direction(OHCI_ED_DATA_FLOW_DIRECTION_IN as ULONG);
    }

    if (*EndpointProperties).DeviceSpeed as u32 == UsbLowSpeed as u32 {
        EndpointControl.set_Speed(OHCI_ENDPOINT_LOW_SPEED as ULONG);
    }

    if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        EndpointControl.set_Format(OHCI_ENDPOINT_ISOCHRONOUS_FORMAT as ULONG);
    } else {
        EndpointControl.set_sKip(1 as ULONG);
    }

    (*ED).HwED.EndpointControl = EndpointControl;

    (*ED).HwED.TailPointer = (*FirstTD).PhysicalAddress;
    (*ED).HwED.HeadPointer = (*FirstTD).PhysicalAddress;

    (*FirstTD).Flags |= OHCI_HCD_TD_FLAG_ALLOCATED as ULONG;

    (*OhciEndpoint).HcdTailP = FirstTD;
    (*OhciEndpoint).HcdHeadP = FirstTD;

    return ED;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InitializeTDs(
    OhciEndpoint: POHCI_ENDPOINT,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
) {
    let mut TdVA: POHCI_HCD_TD;
    let mut TdPA: ULONG;
    let TdCount: ULONG;
    let mut ix: ULONG;

    // ASSERT(EndpointProperties->BufferLength > sizeof(OHCI_HCD_ED));

    TdCount = (((*EndpointProperties).BufferLength as usize)
        .wrapping_sub(core::mem::size_of::<OHCI_HCD_ED>())
        .wrapping_div(core::mem::size_of::<OHCI_HCD_TD>())) as ULONG;

    (*OhciEndpoint).MaxTransferDescriptors = TdCount;

    DPRINT_OHCI!(
        b"OHCI_InitializeTDs: TdCount - %x\n\0".as_ptr() as *const i8,
        TdCount,
    );

    // ASSERT(TdCount > 0);

    TdVA = (*OhciEndpoint).FirstTD;

    TdPA = (((*EndpointProperties).BufferPA as usize)
        .wrapping_add(core::mem::size_of::<OHCI_HCD_ED>())) as ULONG;

    ix = 0;
    while ix < TdCount {
        DPRINT_OHCI!(
            b"OHCI_InitializeTDs: TdVA - %p, TdPA - %08X\n\0".as_ptr() as *const i8,
            TdVA,
            TdPA,
        );

        RtlZeroMemory(TdVA as *mut core::ffi::c_void, core::mem::size_of::<OHCI_HCD_TD>());

        (*TdVA).PhysicalAddress = TdPA;
        (*TdVA).Flags = 0;
        (*TdVA).OhciTransfer = core::ptr::null_mut();

        TdVA = TdVA.offset(1);
        TdPA = (TdPA as usize).wrapping_add(core::mem::size_of::<OHCI_HCD_TD>()) as ULONG;
        ix = ix.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_OpenControlEndpoint(
    OhciExtension: POHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    OhciEndpoint: POHCI_ENDPOINT,
) -> MPSTATUS {
    let mut ED: POHCI_HCD_ED;

    DPRINT_OHCI!(b"OHCI_OpenControlEndpoint: ... \n\0".as_ptr() as *const i8);

    ED = (*EndpointProperties).BufferVA as POHCI_HCD_ED;

    (*OhciEndpoint).FirstTD =
        (ED as ULONG_PTR).wrapping_add(core::mem::size_of::<OHCI_HCD_ED>() as ULONG_PTR)
            as POHCI_HCD_TD;
    (*OhciEndpoint).HeadED = core::ptr::addr_of_mut!((*OhciExtension).ControlStaticED);

    OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);

    (*OhciEndpoint).HcdED = OHCI_InitializeED(
        OhciEndpoint,
        ED,
        (*OhciEndpoint).FirstTD,
        (*EndpointProperties).BufferPA as ULONG_PTR,
    );

    (*(*OhciEndpoint).HcdED).Flags =
        (OHCI_HCD_ED_FLAG_CONTROL | OHCI_HCD_ED_FLAG_RESET_ON_HALT) as ULONG;

    OHCI_InsertEndpointInSchedule(OhciEndpoint);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_OpenBulkEndpoint(
    OhciExtension: POHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    OhciEndpoint: POHCI_ENDPOINT,
) -> MPSTATUS {
    let mut ED: POHCI_HCD_ED;

    DPRINT_OHCI!(b"OHCI_OpenBulkEndpoint: ... \n\0".as_ptr() as *const i8);

    ED = (*EndpointProperties).BufferVA as POHCI_HCD_ED;

    (*OhciEndpoint).FirstTD =
        ((ED as ULONG_PTR).wrapping_add(core::mem::size_of::<OHCI_HCD_ED>() as ULONG_PTR))
            as POHCI_HCD_TD;
    (*OhciEndpoint).HeadED = core::ptr::addr_of_mut!((*OhciExtension).BulkStaticED) as POHCI_STATIC_ED;

    OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);

    (*OhciEndpoint).HcdED = OHCI_InitializeED(
        OhciEndpoint,
        ED,
        (*OhciEndpoint).FirstTD,
        (*EndpointProperties).BufferPA as ULONG_PTR,
    );

    OHCI_InsertEndpointInSchedule(OhciEndpoint);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_OpenInterruptEndpoint(
    OhciExtension: POHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    OhciEndpoint: POHCI_ENDPOINT,
) -> MPSTATUS {
    let mut Period: UCHAR;
    let mut PeriodIdx: ULONG = 0;
    let ED: POHCI_HCD_ED;
    let ScheduleOffset: ULONG;

    DPRINT_OHCI!(b"OHCI_OpenInterruptEndpoint: ... \n\0".as_ptr() as *const i8);

    ED = (*EndpointProperties).BufferVA as POHCI_HCD_ED;

    (*OhciEndpoint).FirstTD =
        ((ED as ULONG_PTR) + core::mem::size_of::<OHCI_HCD_ED>() as ULONG_PTR) as POHCI_HCD_TD;

    Period = (*EndpointProperties).Period;

    while (Period & 1) == 0 {
        PeriodIdx += 1;
        Period >>= 1;
    }

    ScheduleOffset = (*EndpointProperties).ScheduleOffset;
    DPRINT_OHCI!(
        b"OHCI_OpenInterruptEndpoint: InitTD. Index[PeriodIdx] - %x, ScheduleOffset - %x\n\0"
            .as_ptr() as *const i8,
        *(&raw const Index).cast::<UCHAR>().offset(PeriodIdx as isize) as ULONG,
        ScheduleOffset,
    );

    (*OhciEndpoint).HeadED = (core::ptr::addr_of_mut!((*OhciExtension).IntStaticED)
        as *mut OHCI_STATIC_ED)
        .offset(
            (*(&raw const Index).cast::<UCHAR>().offset(PeriodIdx as isize) as ULONG).wrapping_add(ScheduleOffset)
                as isize,
        ) as POHCI_STATIC_ED;

    //OhciEndpoint->HeadED->UsbBandwidth += EndpointProperties->UsbBandwidth;

    OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);

    (*OhciEndpoint).HcdED = OHCI_InitializeED(
        OhciEndpoint,
        ED,
        (*OhciEndpoint).FirstTD,
        (*EndpointProperties).BufferPA as ULONG_PTR,
    );

    OHCI_InsertEndpointInSchedule(OhciEndpoint);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_OpenIsoEndpoint(
    OhciExtension: POHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    OhciEndpoint: POHCI_ENDPOINT,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) OHCI_OpenIsoEndpoint: UNIMPLEMENTED. FIXME\n\0".as_ptr() as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0".as_ptr()
            as *const core::ffi::c_char,
        380 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0".as_ptr()
                as *const core::ffi::c_char,
            380 as core::ffi::c_int,
        );
    }
    return MP_STATUS_NOT_SUPPORTED as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_OpenEndpoint(
    ohciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    ohciEndpoint: PVOID,
) -> MPSTATUS {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let TransferType: ULONG;
    let MPStatus: MPSTATUS;

    DPRINT_OHCI!(b"OHCI_OpenEndpoint: ... \n\0".as_ptr() as *const i8);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*OhciEndpoint).EndpointProperties) as *mut core::ffi::c_void,
        EndpointProperties as *const core::ffi::c_void,
        core::mem::size_of::<USBPORT_ENDPOINT_PROPERTIES>(),
    );

    InitializeListHead(core::ptr::addr_of_mut!((*OhciEndpoint).TDList));

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
            MPStatus = OHCI_OpenIsoEndpoint(OhciExtension, EndpointProperties, OhciEndpoint);
        }
        USBPORT_TRANSFER_TYPE_CONTROL => {
            MPStatus = OHCI_OpenControlEndpoint(OhciExtension, EndpointProperties, OhciEndpoint);
        }
        USBPORT_TRANSFER_TYPE_BULK => {
            MPStatus = OHCI_OpenBulkEndpoint(OhciExtension, EndpointProperties, OhciEndpoint);
        }
        USBPORT_TRANSFER_TYPE_INTERRUPT => {
            MPStatus = OHCI_OpenInterruptEndpoint(OhciExtension, EndpointProperties, OhciEndpoint);
        }
        _ => {
            MPStatus = MP_STATUS_NOT_SUPPORTED as MPSTATUS;
        }
    }

    return MPStatus;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_ReopenEndpoint(
    ohciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    ohciEndpoint: PVOID,
) -> MPSTATUS {
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let ED: POHCI_HCD_ED;

    DPRINT_OHCI!(b"OHCI_ReopenEndpoint: ... \n\0".as_ptr() as *const i8);

    ED = (*OhciEndpoint).HcdED;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*OhciEndpoint).EndpointProperties) as *mut core::ffi::c_void,
        EndpointProperties as *const core::ffi::c_void,
        core::mem::size_of_val(&(*OhciEndpoint).EndpointProperties),
    );

    (*ED).HwED.EndpointControl.set_FunctionAddress(
        (*OhciEndpoint).EndpointProperties.DeviceAddress as ULONG,
    );

    (*ED).HwED.EndpointControl.set_MaximumPacketSize(
        (*OhciEndpoint).EndpointProperties.TotalMaxPacketSize as ULONG,
    );

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_QueryEndpointRequirements(
    ohciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EndpointRequirements: PUSBPORT_ENDPOINT_REQUIREMENTS,
) {
    let mut TransferType: ULONG;

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        0 => {
            (*EndpointRequirements).MaxTransferSize = OHCI_MAX_ISO_TRANSFER_SIZE as ULONG;
            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<OHCI_HCD_ED>())
                    .wrapping_add(
                        (OHCI_MAX_ISO_TD_COUNT as usize)
                            .wrapping_mul(core::mem::size_of::<OHCI_HCD_TD>()),
                    ) as ULONG;
        }
        1 => {
            (*EndpointRequirements).MaxTransferSize = OHCI_MAX_CONTROL_TRANSFER_SIZE as ULONG;
            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<OHCI_HCD_ED>())
                    .wrapping_add(
                        (OHCI_MAX_CONTROL_TD_COUNT as usize)
                            .wrapping_mul(core::mem::size_of::<OHCI_HCD_TD>()),
                    ) as ULONG;
        }
        2 => {
            (*EndpointRequirements).MaxTransferSize = OHCI_MAX_BULK_TRANSFER_SIZE as ULONG;
            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<OHCI_HCD_ED>())
                    .wrapping_add(
                        (OHCI_MAX_BULK_TD_COUNT as usize)
                            .wrapping_mul(core::mem::size_of::<OHCI_HCD_TD>()),
                    ) as ULONG;
        }
        3 => {
            (*EndpointRequirements).MaxTransferSize = OHCI_MAX_INTERRUPT_TRANSFER_SIZE as ULONG;
            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<OHCI_HCD_ED>())
                    .wrapping_add(
                        (OHCI_MAX_INTERRUPT_TD_COUNT as usize)
                            .wrapping_mul(core::mem::size_of::<OHCI_HCD_TD>()),
                    ) as ULONG;
        }
        _ => {
            if DbgPrint(
                b"(%s:%d) OHCI_QueryEndpointRequirements: Unknown TransferType - %x\n\0" as *const u8 as PCSTR,
                b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
                509 as core::ffi::c_int,
                TransferType,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
                    509 as core::ffi::c_int,
                );
            }
            DbgBreakPoint();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_CloseEndpoint(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
    IsDoDisablePeriodic: BOOLEAN,
) {
    if DbgPrint(
        b"(%s:%d) OHCI_CloseEndpoint: Not supported\n\0".as_ptr() as *const i8,
        b"drivers/usb/usbohci/usbohci.c\0".as_ptr()
            as *const i8,
        522 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"drivers/usb/usbohci/usbohci.c\0".as_ptr()
                as *const i8,
            522 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_TakeControlHC(
    OhciExtension: POHCI_EXTENSION,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let ControlReg: PULONG;
    let InterruptEnableReg: PULONG;
    let InterruptDisableReg: PULONG;
    let CommandStatusReg: PULONG;
    let InterruptStatusReg: PULONG;
    let mut Control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut IntEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut IntDisable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut CommandStatus: OHCI_REG_COMMAND_STATUS = core::mem::zeroed();
    let mut IntStatus: OHCI_REG_INTERRUPT_STATUS = core::mem::zeroed();
    let mut StartTicks: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTicks: LARGE_INTEGER = core::mem::zeroed();
    let TicksDiff: UINT32;

    OperationalRegs = (*OhciExtension).OperationalRegs;

    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;
    InterruptDisableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as PULONG;
    CommandStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcCommandStatus) as PULONG;
    InterruptStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptStatus) as PULONG;

    /* 5.1.1.3 Take Control of Host Controller */
    Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);

    if Control.c2rust_unnamed.InterruptRouting() == 0 {
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    IntEnable.AsULONG = READ_REGISTER_ULONG(InterruptEnableReg as *mut ULONG);

    if Control.c2rust_unnamed.HostControllerFunctionalState() == OHCI_HC_STATE_RESET as ULONG
        && IntEnable.AsULONG == 0
    {
        Control.AsULONG = 0;
        WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    /* Enable interrupt generations */
    IntEnable.AsULONG = 0;
    IntEnable.c2rust_unnamed.set_MasterInterruptEnable(1);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, IntEnable.AsULONG);

    /* Request a change of control of the HC */
    CommandStatus.AsULONG = 0;
    CommandStatus.c2rust_unnamed.set_OwnershipChangeRequest(1);

    WRITE_REGISTER_ULONG(CommandStatusReg as *mut ULONG, CommandStatus.AsULONG);

    /* Disable interrupt generation due to Root Hub Status Change */
    IntDisable.AsULONG = 0;
    IntDisable.c2rust_unnamed.set_RootHubStatusChange(1);

    WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, IntDisable.AsULONG);

    /* Monitoring the InterruptRouting bit
       to determine when the ownership change has taken effect. */

    TicksDiff = ((500 * 10000) as ULONG).wrapping_div(KeQueryTimeIncrement()) as UINT32; // 500 ms
    KeQueryTickCount(core::ptr::addr_of_mut!(StartTicks) as *mut i64);

    loop {
        Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);

        if Control.c2rust_unnamed.InterruptRouting() == 0 {
            /* Clear all bits in register */
            IntStatus.AsULONG = 0xFFFFFFFF;
            WRITE_REGISTER_ULONG(InterruptStatusReg as *mut ULONG, IntStatus.AsULONG);

            /* Disable interrupt generations */
            IntDisable.AsULONG = 0;
            IntDisable.c2rust_unnamed.set_MasterInterruptEnable(1);

            WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, IntDisable.AsULONG);

            return MP_STATUS_SUCCESS as MPSTATUS;
        }

        KeQueryTickCount(core::ptr::addr_of_mut!(CurrentTicks) as *mut i64);

        if !(CurrentTicks.QuadPart - StartTicks.QuadPart < TicksDiff as i64) {
            break;
        }
    }

    return MP_STATUS_HW_ERROR as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_StartController(
    ohciExtension: PVOID,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let CommandStatusReg: PULONG;
    let FmIntervalReg: PULONG;
    let ControlReg: PULONG;
    let InterruptEnableReg: PULONG;
    let RhStatusReg: PULONG;
    let mut CommandStatus: OHCI_REG_COMMAND_STATUS = core::mem::zeroed();
    let mut Interrupts: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut RhStatus: OHCI_REG_RH_STATUS = core::mem::zeroed();
    let mut FrameInterval: OHCI_REG_FRAME_INTERVAL = core::mem::zeroed();
    let MaxFrameIntervalAdjusting: ULONG;
    let mut Control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut HeadIndex: UCHAR;
    let mut IntED: POHCI_ENDPOINT_DESCRIPTOR;
    let mut IntEdPA: ULONG_PTR;
    let OhciHCCA: POHCI_HCCA;
    let mut StartTicks: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTicks: LARGE_INTEGER = core::mem::zeroed();
    let TicksDiff: UINT32;
    let mut ix: ULONG;
    let mut jx: ULONG;
    let mut MPStatus: MPSTATUS = MP_STATUS_SUCCESS as MPSTATUS;

    DPRINT_OHCI!(
        b"OHCI_StartController: ohciExtension - %p, Resources - %p\n\0".as_ptr() as *const i8,
        ohciExtension,
        Resources,
    );

    /* HC on-chip operational registers */
    OperationalRegs = (*Resources).ResourceBase as POHCI_OPERATIONAL_REGISTERS;
    (*OhciExtension).OperationalRegs = OperationalRegs;

    CommandStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcCommandStatus) as PULONG;
    FmIntervalReg = core::ptr::addr_of_mut!((*OperationalRegs).HcFmInterval) as PULONG;
    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;
    RhStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhStatus) as PULONG;

    /* 5.1.1 Initialization */

    MPStatus = OHCI_TakeControlHC(OhciExtension, Resources);

    if MPStatus != MP_STATUS_SUCCESS as MPSTATUS {
        // DPRINT removed

        return MPStatus;
    }

    (*OhciExtension).HcResourcesVA = (*Resources).StartVA as POHCI_HC_RESOURCES;
    (*OhciExtension).HcResourcesPA = (*Resources).StartPA;

    DPRINT_OHCI!(
        b"OHCI_StartController: HcResourcesVA - %p, HcResourcesPA - %lx\n\0".as_ptr() as *const i8,
        (*OhciExtension).HcResourcesVA,
        (*OhciExtension).HcResourcesPA,
    );

    /* 5.2.7.2 Interrupt */

    /* Build structure of interrupt static EDs */
    ix = 0 as ULONG;
    while ix < INTERRUPT_ENDPOINTs as ULONG {
        IntED = (core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).InterrruptHeadED)
            as *mut OHCI_ENDPOINT_DESCRIPTOR)
            .offset(ix as isize) as POHCI_ENDPOINT_DESCRIPTOR;
        IntEdPA = ((*OhciExtension).HcResourcesPA as ULONG).wrapping_add(
            core::mem::offset_of!(OHCI_HC_RESOURCES, InterrruptHeadED) as ULONG
                + (ix * core::mem::size_of::<OHCI_ENDPOINT_DESCRIPTOR>() as ULONG),
        ) as ULONG_PTR;

        if ix == (ENDPOINT_INTERRUPT_1ms - 1) as ULONG {
            HeadIndex = ED_EOF as UCHAR;
            (*IntED).NextED = 0 as ULONG;
        } else {
            HeadIndex = (ix.wrapping_sub(1) / 2) as UCHAR;

            if !((HeadIndex as i32 >= ((ENDPOINT_INTERRUPT_1ms - 1) as i32))
                && ((HeadIndex as i32)
                    < ((INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) as i32)))
            {
                DbgBreakPoint();
            }

            (*IntED).NextED =
                (*OhciExtension).IntStaticED[HeadIndex as usize].PhysicalAddress;
        }

        (*IntED).EndpointControl.set_sKip(1);

        (*IntED).TailPointer = 0 as ULONG;
        (*IntED).HeadPointer = 0 as ULONG;

        (*OhciExtension).IntStaticED[ix as usize].HwED = IntED;
        (*OhciExtension).IntStaticED[ix as usize].PhysicalAddress = IntEdPA as ULONG;
        (*OhciExtension).IntStaticED[ix as usize].HeadIndex = HeadIndex;
        (*OhciExtension).IntStaticED[ix as usize].pNextED =
            core::ptr::addr_of_mut!((*IntED).NextED) as PULONG;
        (*OhciExtension).IntStaticED[ix as usize].Type = OHCI_STATIC_ED_TYPE_INTERRUPT as ULONG;

        InitializeListHead(core::ptr::addr_of_mut!(
            (*OhciExtension).IntStaticED[ix as usize].Link
        ));

        ix = ix.wrapping_add(1);
    }

    OhciHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;
    DPRINT_OHCI!(
        b"OHCI_InitializeSchedule: OhciHCCA - %p\n\0".as_ptr() as *const i8,
        OhciHCCA,
    );

    /* Set head pointers which start from HCCA */
    ix = 0 as ULONG;
    jx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) as ULONG;
    while ix < OHCI_NUMBER_OF_INTERRUPTS as ULONG {
        (*OhciHCCA).InterrruptTable[Balance[ix as usize] as usize] =
            (*OhciExtension).IntStaticED[jx as usize].PhysicalAddress;

        (*OhciExtension).IntStaticED[jx as usize].pNextED =
            core::ptr::addr_of_mut!((*OhciHCCA).InterrruptTable[Balance[ix as usize] as usize])
                as PULONG;

        (*OhciExtension).IntStaticED[jx as usize].HccaIndex = Balance[ix as usize] as ULONG;

        ix = ix.wrapping_add(1);
        jx = jx.wrapping_add(1);
    }

    DPRINT_OHCI!(b"OHCI_InitializeSchedule: ix - %x\n\0".as_ptr() as *const i8, ix);

    /* Init static Control and Bulk EDs head pointers which start from HCCA */
    InitializeListHead(core::ptr::addr_of_mut!((*OhciExtension).ControlStaticED.Link));

    (*OhciExtension).ControlStaticED.HeadIndex = ED_EOF as UCHAR;
    (*OhciExtension).ControlStaticED.Type = OHCI_STATIC_ED_TYPE_CONTROL as ULONG;
    (*OhciExtension).ControlStaticED.pNextED =
        core::ptr::addr_of_mut!((*OperationalRegs).HcControlHeadED) as PULONG;

    InitializeListHead(core::ptr::addr_of_mut!((*OhciExtension).BulkStaticED.Link));

    (*OhciExtension).BulkStaticED.HeadIndex = ED_EOF as UCHAR;
    (*OhciExtension).BulkStaticED.Type = OHCI_STATIC_ED_TYPE_BULK as ULONG;
    (*OhciExtension).BulkStaticED.pNextED =
        core::ptr::addr_of_mut!((*OperationalRegs).HcBulkHeadED) as PULONG;

    /* 6.3.1 Frame Timing */
    FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg as *mut ULONG);

    MaxFrameIntervalAdjusting = (OHCI_DEFAULT_FRAME_INTERVAL / 10) as ULONG; // 10%

    if FrameInterval.FrameInterval()
        < (OHCI_DEFAULT_FRAME_INTERVAL as ULONG).wrapping_sub(MaxFrameIntervalAdjusting)
        || FrameInterval.FrameInterval()
            > (OHCI_DEFAULT_FRAME_INTERVAL as ULONG).wrapping_add(MaxFrameIntervalAdjusting)
    {
        FrameInterval.set_FrameInterval(OHCI_DEFAULT_FRAME_INTERVAL as ULONG);
    }

    /* 5.4 FrameInterval Counter */
    FrameInterval.set_FrameIntervalToggle(1);

    /* OHCI_MAXIMUM_OVERHEAD is the maximum overhead per frame */
    FrameInterval.set_FSLargestDataPacket(
        ((FrameInterval.FrameInterval() as i32 - (OHCI_MAXIMUM_OVERHEAD as i32)) * 6 / 7) as ULONG,
    );

    (*OhciExtension).FrameInterval = FrameInterval;

    DPRINT_OHCI!(
        b"OHCI_StartController: FrameInterval - %lX\n\0".as_ptr() as *const i8,
        FrameInterval.AsULONG,
    );

    /* Reset HostController */
    CommandStatus.AsULONG = 0 as ULONG;
    CommandStatus.set_HostControllerReset(1);

    WRITE_REGISTER_ULONG(CommandStatusReg as *mut ULONG, CommandStatus.AsULONG);

    KeStallExecutionProcessor(25 as ULONG);

    Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);
    Control.set_HostControllerFunctionalState(OHCI_HC_STATE_RESET as ULONG);

    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);

    TicksDiff = ((500 * 10000) as ULONG).wrapping_div(KeQueryTimeIncrement()) as UINT32; // 500 ms
    KeQueryTickCount(core::ptr::addr_of_mut!(StartTicks) as *mut i64);

    loop {
        WRITE_REGISTER_ULONG(
            FmIntervalReg as *mut ULONG,
            (*OhciExtension).FrameInterval.AsULONG,
        );
        FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg as *mut ULONG);

        KeQueryTickCount(core::ptr::addr_of_mut!(CurrentTicks) as *mut i64);

        if CurrentTicks.QuadPart - StartTicks.QuadPart >= TicksDiff as i64 {
            MPStatus = MP_STATUS_HW_ERROR as MPSTATUS;
            break;
        }

        if FrameInterval.AsULONG == (*OhciExtension).FrameInterval.AsULONG {
            MPStatus = MP_STATUS_SUCCESS as MPSTATUS;
            break;
        }
    }

    if MPStatus != MP_STATUS_SUCCESS as MPSTATUS {
        DPRINT_OHCI!(
            b"OHCI_StartController: frame interval not set\n\0".as_ptr() as *const i8,
        );
        return MPStatus;
    }

    /* Setup HcPeriodicStart register */
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcPeriodicStart) as *mut ULONG,
        (((*OhciExtension).FrameInterval.FrameInterval() as i32 * 9) / 10) as ULONG,
    ); // 90%

    /* Setup HcHCCA register */
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcHCCA) as *mut ULONG,
        (*OhciExtension)
            .HcResourcesPA
            .wrapping_add(core::mem::offset_of!(OHCI_HC_RESOURCES, HcHCCA) as ULONG),
    );

    /* Setup HcInterruptEnable register */
    Interrupts.AsULONG = 0 as ULONG;

    Interrupts.set_SchedulingOverrun(1);
    Interrupts.set_WritebackDoneHead(1);
    Interrupts.set_UnrecoverableError(1);
    Interrupts.set_FrameNumberOverflow(1);
    Interrupts.set_OwnershipChange(1);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, Interrupts.AsULONG);

    /* Setup HcControl register */
    Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);

    Control.set_ControlBulkServiceRatio(0); // FIXME (1 : 1)
    Control.set_PeriodicListEnable(1);
    Control.set_IsochronousEnable(1);
    Control.set_ControlListEnable(1);
    Control.set_BulkListEnable(1);
    Control.set_HostControllerFunctionalState(OHCI_HC_STATE_OPERATIONAL as ULONG);

    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);

    /* Setup HcRhStatus register */
    RhStatus.AsULONG = 0 as ULONG;
    RhStatus.set_SetGlobalPower(1);

    WRITE_REGISTER_ULONG(RhStatusReg as *mut ULONG, RhStatus.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_StopController(
    ohciExtension: PVOID,
    IsDoDisableInterrupts: BOOLEAN,
) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let ControlReg: PULONG;
    let InterruptDisableReg: PULONG;
    let InterruptStatusReg: PULONG;
    let mut Control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut IntDisable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut IntStatus: OHCI_REG_INTERRUPT_STATUS = core::mem::zeroed();

    OperationalRegs = (*OhciExtension).OperationalRegs;

    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    InterruptDisableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as PULONG;
    InterruptStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptStatus) as PULONG;

    /* Setup HcControl register */
    Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);

    Control.set_PeriodicListEnable(0);
    Control.set_IsochronousEnable(0);
    Control.set_ControlListEnable(0);
    Control.set_BulkListEnable(0);
    Control.set_HostControllerFunctionalState(OHCI_HC_STATE_SUSPEND as ULONG);
    Control.set_RemoteWakeupEnable(0);

    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);

    /* Disable interrupt generations */
    IntDisable.AsULONG = 0xFFFFFFFF;
    WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, IntDisable.AsULONG);

    /* Clear all bits in HcInterruptStatus register */
    IntStatus.AsULONG = 0xFFFFFFFF;
    WRITE_REGISTER_ULONG(InterruptStatusReg as *mut ULONG, IntStatus.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SuspendController(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let ControlReg: PULONG;
    let InterruptEnableReg: PULONG;
    let mut Control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut InterruptReg: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    OperationalRegs = (*OhciExtension).OperationalRegs;
    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;

    /* Disable all interrupt generations */
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable.AsULONG) as *mut ULONG,
        0xFFFFFFFF as ULONG,
    );

    /* Clear all bits in HcInterruptStatus register */
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptStatus.AsULONG) as *mut ULONG,
        0xFFFFFFFF as ULONG,
    );

    /* Setup HcControl register */
    Control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);
    Control.set_HostControllerFunctionalState(OHCI_HC_STATE_SUSPEND as ULONG);
    Control.set_RemoteWakeupEnable(1 as ULONG);

    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);

    /* Setup HcInterruptEnable register */
    InterruptReg.AsULONG = 0 as ULONG;
    InterruptReg.set_ResumeDetected(1 as ULONG);
    InterruptReg.set_UnrecoverableError(1 as ULONG);
    InterruptReg.set_RootHubStatusChange(1 as ULONG);
    InterruptReg.set_MasterInterruptEnable(1 as ULONG);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, InterruptReg.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_ResumeController(ohciExtension: PVOID) -> MPSTATUS {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let ControlReg: PULONG;
    let InterruptEnableReg: PULONG;
    let HcHCCA: POHCI_HCCA;
    let mut control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut InterruptReg: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    OperationalRegs = (*OhciExtension).OperationalRegs;
    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;

    control.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);

    if control.c2rust_unnamed.HostControllerFunctionalState() != OHCI_HC_STATE_SUSPEND as ULONG {
        return MP_STATUS_HW_ERROR as MPSTATUS;
    }

    HcHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;
    (*HcHCCA).Pad1 = 0 as USHORT;

    control.c2rust_unnamed.set_HostControllerFunctionalState(OHCI_HC_STATE_OPERATIONAL as ULONG);
    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, control.AsULONG);

    InterruptReg.AsULONG = 0 as ULONG;
    InterruptReg.c2rust_unnamed.set_SchedulingOverrun(1 as ULONG);
    InterruptReg.c2rust_unnamed.set_WritebackDoneHead(1 as ULONG);
    InterruptReg.c2rust_unnamed.set_UnrecoverableError(1 as ULONG);
    InterruptReg.c2rust_unnamed.set_FrameNumberOverflow(1 as ULONG);
    InterruptReg.c2rust_unnamed.set_OwnershipChange(1 as ULONG);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, InterruptReg.AsULONG);
    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, control.AsULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_HardwarePresent(
    OhciExtension: POHCI_EXTENSION,
    IsInvalidateController: BOOLEAN,
) -> BOOLEAN {
    const USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE: u32 = 0x02;

    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let mut CommandStatusReg: PULONG;

    OperationalRegs = (*OhciExtension).OperationalRegs;
    CommandStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcCommandStatus) as PULONG;

    if READ_REGISTER_ULONG(CommandStatusReg as *mut ULONG) != 0xFFFFFFFF {
        return 1 as BOOLEAN;
    }

    if DbgPrint(
        b"(%s:%d) OHCI_HardwarePresent: IsInvalidateController - %x\n\0" as *const u8 as PCSTR,
        b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
        1007 as core::ffi::c_int,
        IsInvalidateController as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
            1007 as core::ffi::c_int,
        );
    }

    if IsInvalidateController != 0 {
        RegPacket
            .UsbPortInvalidateController
            .expect("non-null function pointer")(
            OhciExtension as PVOID,
            USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE as ULONG,
        );
    }

    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InterruptService(ohciExtension: PVOID) -> BOOLEAN {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS = core::ptr::null_mut();
    let mut IntStatus: OHCI_REG_INTERRUPT_STATUS = core::mem::zeroed();
    let mut IntEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut IntDisable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut HardwarePresent: BOOLEAN = 0;

    DPRINT_OHCI!(
        b"OHCI_Interrupt: Ext %p\n\0".as_ptr() as *const i8,
        OhciExtension,
    );

    OperationalRegs = (*OhciExtension).OperationalRegs;

    HardwarePresent = OHCI_HardwarePresent(OhciExtension, 0);

    if HardwarePresent == 0 {
        return 0;
    }

    IntEnable.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as *mut ULONG);
    IntStatus.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptStatus) as *mut ULONG)
            & IntEnable.AsULONG;

    if IntStatus.AsULONG == 0 || IntEnable.MasterInterruptEnable() == 0 {
        return 0;
    }

    if IntStatus.UnrecoverableError() != 0 {
        // DPRINT removed
    }

    if IntStatus.FrameNumberOverflow() != 0 {
        let HcHCCA: POHCI_HCCA;
        let fm: ULONG;
        let hp: ULONG;

        HcHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;

        // DPRINT removed

        hp = (*OhciExtension).FrameHighPart;
        fm = (*HcHCCA).FrameNumber as ULONG;

        /* Increment value of FrameHighPart */
        (*OhciExtension).FrameHighPart = ((*OhciExtension).FrameHighPart as core::ffi::c_ulong)
            .wrapping_add(
                ((1 * (1 << 16)) as core::ffi::c_ulong).wrapping_sub(
                    (hp as core::ffi::c_ulong ^ fm as core::ffi::c_ulong) & 0x8000,
                ),
            ) as ULONG;
    }

    /* Disable interrupt generation */
    IntDisable.AsULONG = 0;
    IntDisable.set_MasterInterruptEnable(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as *mut ULONG,
        IntDisable.AsULONG,
    );

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InterruptDpc(
    ohciExtension: PVOID,
    IsDoEnableInterrupts: BOOLEAN,
) {

    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS = core::ptr::null_mut();
    let mut InterruptDisableReg: PULONG = core::ptr::null_mut();
    let mut InterruptEnableReg: PULONG = core::ptr::null_mut();
    let mut InterruptStatusReg: PULONG = core::ptr::null_mut();
    let mut IntStatus: OHCI_REG_INTERRUPT_STATUS = core::mem::zeroed();
    let mut InterruptBits: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let mut HcHCCA: POHCI_HCCA;

    OperationalRegs = (*OhciExtension).OperationalRegs;

    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;
    InterruptDisableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as PULONG;
    InterruptStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptStatus) as PULONG;

    IntStatus.AsULONG = READ_REGISTER_ULONG(InterruptStatusReg as *mut ULONG);

    if IntStatus.c2rust_unnamed.RootHubStatusChange() != 0 {
        RegPacket.UsbPortInvalidateRootHub.expect("non-null function pointer")(
            OhciExtension as PVOID,
        );
    }

    if IntStatus.c2rust_unnamed.WritebackDoneHead() != 0 {
        HcHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;
        (*HcHCCA).DoneHead = 0 as ULONG;

        RegPacket.UsbPortInvalidateEndpoint.expect("non-null function pointer")(
            OhciExtension as PVOID,
            core::ptr::null_mut(),
        );
    }

    if IntStatus.c2rust_unnamed.StartofFrame() != 0 {
        InterruptBits.AsULONG = 0 as ULONG;
        InterruptBits.c2rust_unnamed.set_StartofFrame(1 as ULONG);

        WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, InterruptBits.AsULONG);
    }

    if IntStatus.c2rust_unnamed.ResumeDetected() != 0 {
        if DbgPrint(
            b"(%s:%d) OHCI_IntDpc: ResumeDetected\n\0" as *const u8 as PCSTR,
            b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
            1125 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
                1125 as core::ffi::c_int,
            );
        }
    }

    if IntStatus.c2rust_unnamed.UnrecoverableError() != 0 {
        if DbgPrint(
            b"(%s:%d) OHCI_IntDpc: UnrecoverableError\n\0" as *const u8 as PCSTR,
            b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
            1129 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbohci.c\0" as *const u8 as *const core::ffi::c_char,
                1129 as core::ffi::c_int,
            );
        }
    }

    WRITE_REGISTER_ULONG(InterruptStatusReg as *mut ULONG, IntStatus.AsULONG);

    if IsDoEnableInterrupts != 0 {
        InterruptBits.AsULONG = 0 as ULONG;
        InterruptBits.c2rust_unnamed.set_MasterInterruptEnable(1 as ULONG);

        WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, InterruptBits.AsULONG);
    }
}
unsafe fn OHCI_MapTransferToTD(
    OhciExtension: POHCI_EXTENSION,
    TransferedLen: ULONG,
    OhciTransfer: POHCI_TRANSFER,
    TD: POHCI_HCD_TD,
    SGList: PUSBPORT_SCATTER_GATHER_LIST,
) -> ULONG {
    let mut SgElement: PUSBPORT_SCATTER_GATHER_ELEMENT = core::ptr::null_mut();
    let mut SgIdx: ULONG;
    let mut CurrentTransferLen: ULONG;
    let mut BufferEnd: ULONG;
    let CurrentBuffer: ULONG;
    let mut TransferDataLeft: ULONG = (*(*OhciTransfer).TransferParameters)
        .TransferBufferLength
        .wrapping_sub(TransferedLen);

    DPRINT_OHCI!(
        b"OHCI_MapTransferToTD: TransferedLen - %x\n\0".as_ptr() as *const i8,
        TransferedLen,
    );

    SgIdx = 0;
    while SgIdx < (*SGList).SgElementCount {
        SgElement = (&raw mut (*SGList).SgElement as *mut USBPORT_SCATTER_GATHER_ELEMENT)
            .offset(SgIdx as isize);

        if TransferedLen >= (*SgElement).SgOffset
            && TransferedLen
                < (*SgElement)
                    .SgOffset
                    .wrapping_add((*SgElement).SgTransferLength)
        {
            break;
        }

        SgIdx = SgIdx.wrapping_add(1);
    }

    DPRINT_OHCI!(
        b"OHCI_MapTransferToTD: SgIdx - %x, SgCount - %x\n\0".as_ptr() as *const i8,
        SgIdx,
        (*SGList).SgElementCount,
    );

    /* ASSERT(SgIdx < SGList->SgElementCount); */
    /* ASSERT(TransferedLen == SgElement->SgOffset); */

    CurrentBuffer = (*SgElement).SgPhysicalAddress.__bindgen_anon_1.LowPart;

    if TransferDataLeft <= (*SgElement).SgTransferLength {
        CurrentTransferLen = TransferDataLeft;
        BufferEnd = (*SgElement)
            .SgPhysicalAddress
            .u.LowPart
            .wrapping_add(CurrentTransferLen)
            .wrapping_sub(1);
    } else {
        let SgNextElement: PUSBPORT_SCATTER_GATHER_ELEMENT;
        /* ASSERT(SGList->SgElementCount - SgIdx > 1); */

        SgNextElement = (&raw mut (*SGList).SgElement as *mut USBPORT_SCATTER_GATHER_ELEMENT)
            .offset(SgIdx.wrapping_add(1) as isize);

        TransferDataLeft =
            TransferDataLeft.wrapping_sub((*SgElement).SgTransferLength);
        CurrentTransferLen = (*SgElement).SgTransferLength;

        if TransferDataLeft <= (*SgNextElement).SgTransferLength {
            CurrentTransferLen = CurrentTransferLen.wrapping_add(TransferDataLeft);
            BufferEnd = (*SgNextElement)
                .SgPhysicalAddress
                .u.LowPart
                .wrapping_add(TransferDataLeft)
                .wrapping_sub(1);
        } else {
            CurrentTransferLen =
                CurrentTransferLen.wrapping_add((*SgNextElement).SgTransferLength);
            BufferEnd = (*SgNextElement)
                .SgPhysicalAddress
                .u.LowPart
                .wrapping_add((*SgNextElement).SgTransferLength)
                .wrapping_sub(1);
        }
    }

    (*TD).HwTD.c2rust_unnamed.gTD.CurrentBuffer = CurrentBuffer;
    (*TD).HwTD.c2rust_unnamed.gTD.BufferEnd = BufferEnd;
    (*TD).TransferLen = CurrentTransferLen;

    return TransferedLen.wrapping_add(CurrentTransferLen);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_AllocateTD(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
) -> POHCI_HCD_TD {
    let mut TD: POHCI_HCD_TD;

    DPRINT_OHCI!(b"OHCI_AllocateTD: ... \n\0".as_ptr() as *const i8);

    TD = (*OhciEndpoint).FirstTD;

    while (*TD).Flags & OHCI_HCD_TD_FLAG_ALLOCATED != 0 {
        TD = TD.offset(1);
    }

    (*TD).Flags |= OHCI_HCD_TD_FLAG_ALLOCATED;

    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!((*TD).HwTD) as PVOID,
        ((core::mem::size_of::<OHCI_HW_TRANSFER_DESCRIPTOR>() as SIZE_T as u32) as _),
    );

    return TD;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RemainTDs(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
) -> ULONG {
    let mut TD: POHCI_HCD_TD;
    let mut MaxTDs: ULONG;
    let mut RemainTDs: ULONG;
    let mut ix: ULONG;

    DPRINT_OHCI!(b"OHCI_RemainTDs: ... \n\0".as_ptr() as *const i8);

    MaxTDs = (*OhciEndpoint).MaxTransferDescriptors;
    TD = (*OhciEndpoint).FirstTD;

    RemainTDs = 0;

    ix = 0;
    while ix < MaxTDs {
        if ((*TD).Flags as u32 & OHCI_HCD_TD_FLAG_ALLOCATED as u32) == 0 {
            RemainTDs = RemainTDs.wrapping_add(1);
        }

        TD = TD.offset(1);
        ix = ix.wrapping_add(1);
    }

    return RemainTDs;
}
unsafe fn OHCI_ControlTransfer(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    OhciTransfer: POHCI_TRANSFER,
    SGList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut SetupTD: POHCI_HCD_TD;
    let mut TD: POHCI_HCD_TD;
    let mut PrevTD: POHCI_HCD_TD;
    let MaxTDs: ULONG;
    let mut TransferedLen: ULONG;
    let mut DataToggle: UCHAR;

    DPRINT_OHCI!(
        b"OHCI_ControlTransfer: Ext %p, Endpoint %p\n\0".as_ptr() as *const i8,
        OhciExtension,
        OhciEndpoint,
    );

    MaxTDs = OHCI_RemainTDs(OhciExtension, OhciEndpoint);

    if ((*SGList).SgElementCount as ULONG).wrapping_add(OHCI_NON_DATA_CONTROL_TDS as ULONG) > MaxTDs
    {
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    /* Form a setup packet first */
    SetupTD = (*OhciEndpoint).HcdTailP;
    RtlSecureZeroMemory(
        core::ptr::addr_of_mut!((*SetupTD).HwTD) as PVOID,
        ((core::mem::size_of::<OHCI_HW_TRANSFER_DESCRIPTOR>() as SIZE_T as u32) as _),
    );

    (*SetupTD).Flags |= OHCI_HCD_TD_FLAG_PROCESSED as ULONG;
    (*SetupTD).OhciTransfer = OhciTransfer;

    (*OhciTransfer).PendingTDs = (*OhciTransfer).PendingTDs.wrapping_add(1);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*SetupTD).HwTD.c2rust_unnamed.SetupPacket) as PVOID,
        core::ptr::addr_of!((*TransferParameters).SetupPacket) as PVOID,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as SIZE_T,
    );

    (*SetupTD).HwTD.c2rust_unnamed.gTD.CurrentBuffer = ((*SetupTD).PhysicalAddress as ULONG)
        .wrapping_add(core::mem::offset_of!(OHCI_HCD_TD, HwTD.c2rust_unnamed.SetupPacket) as ULONG)
        as ULONG;
    (*SetupTD).HwTD.c2rust_unnamed.gTD.BufferEnd = (((*SetupTD).PhysicalAddress as ULONG)
        .wrapping_add(core::mem::offset_of!(OHCI_HCD_TD, HwTD.c2rust_unnamed.SetupPacket) as ULONG) as usize)
        .wrapping_add(core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as usize)
        .wrapping_sub(1 as usize) as ULONG;
    (*SetupTD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_DelayInterrupt(OHCI_TD_INTERRUPT_NONE as ULONG);
    (*SetupTD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_ConditionCode(OHCI_TD_CONDITION_NOT_ACCESSED as ULONG);
    (*SetupTD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_DataToggle(OHCI_TD_DATA_TOGGLE_DATA0 as ULONG);

    PrevTD = SetupTD;

    /* Data packets follow a setup packet (if any) */
    TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
    (*TD).Flags |= OHCI_HCD_TD_FLAG_PROCESSED as ULONG;
    (*TD).OhciTransfer = OhciTransfer;

    (*PrevTD).HwTD.c2rust_unnamed.gTD.NextTD = (*TD).PhysicalAddress;
    (*PrevTD).NextTDVa = TD;

    /* The first data packet should use DATA1, subsequent ones use DATA0 (OpenHCI spec, 4.3.1.3.4) */
    DataToggle = OHCI_TD_DATA_TOGGLE_DATA1 as UCHAR;
    TransferedLen = 0 as ULONG;

    while TransferedLen < (*TransferParameters).TransferBufferLength {
        (*OhciTransfer).PendingTDs = (*OhciTransfer).PendingTDs.wrapping_add(1);

        if (*TransferParameters).TransferFlags as ULONG & USBD_TRANSFER_DIRECTION_IN as ULONG != 0 {
            (*TD)
                .HwTD
                .c2rust_unnamed.gTD
                .Control
                .set_DirectionPID(OHCI_TD_DIRECTION_PID_IN as ULONG);
        } else {
            (*TD)
                .HwTD
                .c2rust_unnamed.gTD
                .Control
                .set_DirectionPID(OHCI_TD_DIRECTION_PID_OUT as ULONG);
        }

        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_DelayInterrupt(OHCI_TD_INTERRUPT_NONE as ULONG);
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_ConditionCode(OHCI_TD_CONDITION_NOT_ACCESSED as ULONG);
        (*TD).HwTD.c2rust_unnamed.gTD.Control.set_DataToggle(DataToggle as ULONG);

        TransferedLen =
            OHCI_MapTransferToTD(OhciExtension, TransferedLen, OhciTransfer, TD, SGList);

        PrevTD = TD;

        TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_DelayInterrupt(OHCI_TD_INTERRUPT_NONE as ULONG);
        (*TD).Flags |= OHCI_HCD_TD_FLAG_PROCESSED as ULONG;
        (*TD).OhciTransfer = OhciTransfer;

        (*PrevTD).HwTD.c2rust_unnamed.gTD.NextTD = (*TD).PhysicalAddress;
        (*PrevTD).NextTDVa = TD;

        DataToggle = OHCI_TD_DATA_TOGGLE_DATA0 as UCHAR;
    }

    if (*TransferParameters).TransferFlags as ULONG & USBD_SHORT_TRANSFER_OK as ULONG != 0 {
        (*PrevTD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_BufferRounding(TRUE as ULONG);
        (*OhciTransfer).Flags |= OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK as ULONG;
    }

    /* After data packets, goes a status packet */

    (*TD).Flags |= OHCI_HCD_TD_FLAG_CONTROL_STATUS as ULONG;
    (*TD).TransferLen = 0 as ULONG;

    if (*TransferParameters).TransferFlags as ULONG & USBD_TRANSFER_DIRECTION_IN as ULONG != 0 {
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_BufferRounding(FALSE as ULONG);
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_DirectionPID(OHCI_TD_DIRECTION_PID_OUT as ULONG);
    } else {
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_BufferRounding(TRUE as ULONG);
        (*TD)
            .HwTD
            .c2rust_unnamed.gTD
            .Control
            .set_DirectionPID(OHCI_TD_DIRECTION_PID_IN as ULONG);
    }

    /* OpenHCI spec, 4.3.1.3.4 */
    (*TD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_DataToggle(OHCI_TD_DATA_TOGGLE_DATA1 as ULONG);
    (*TD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_ConditionCode(OHCI_TD_CONDITION_NOT_ACCESSED as ULONG);
    (*TD)
        .HwTD
        .c2rust_unnamed.gTD
        .Control
        .set_DelayInterrupt(OHCI_TD_INTERRUPT_IMMEDIATE as ULONG);

    (*OhciTransfer).PendingTDs = (*OhciTransfer).PendingTDs.wrapping_add(1);
    (*OhciTransfer).ControlStatusTD = TD;

    PrevTD = TD;

    /* And the last descriptor, which is not used in the current transfer (OpenHCI spec, 4.6) */
    TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);

    (*PrevTD).HwTD.c2rust_unnamed.gTD.NextTD = (*TD).PhysicalAddress;
    (*PrevTD).NextTDVa = TD;

    (*TD).NextTDVa = core::ptr::null_mut();
    /* TD->HwTD.c2rust_unnamed.gTD.NextTD = 0; */

    (*OhciTransfer).NextTD = TD;
    (*OhciEndpoint).HcdTailP = TD;

    (*(*OhciEndpoint).HcdED).HwED.TailPointer = (*TD).PhysicalAddress;

    OHCI_EnableList(OhciExtension, OhciEndpoint);

    return MP_STATUS_SUCCESS as MPSTATUS;
}
unsafe fn OHCI_BulkOrInterruptTransfer(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    OhciTransfer: POHCI_TRANSFER,
    SGList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut TD: POHCI_HCD_TD;
    let mut PrevTD: POHCI_HCD_TD;
    let mut TransferedLen: ULONG;
    let mut MaxTDs: ULONG;

    DPRINT_OHCI!(b"OHCI_BulkOrInterruptTransfer: ... \n\0".as_ptr() as *const i8);

    MaxTDs = OHCI_RemainTDs(OhciExtension, OhciEndpoint);

    if (*SGList).SgElementCount > MaxTDs {
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    TD = (*OhciEndpoint).HcdTailP;

    TransferedLen = 0 as ULONG;

    PrevTD = core::ptr::null_mut();

    loop {
        (*TD).HwTD.c2rust_unnamed.gTD.Control.set_DelayInterrupt(OHCI_TD_INTERRUPT_NONE as ULONG);
        (*TD).HwTD.c2rust_unnamed.gTD.Control.set_ConditionCode(OHCI_TD_CONDITION_NOT_ACCESSED as ULONG);

        if (*TransferParameters).TransferFlags & USBD_TRANSFER_DIRECTION_IN != 0 {
            (*TD).HwTD.c2rust_unnamed.gTD.Control.set_BufferRounding(FALSE as ULONG);
            (*TD).HwTD.c2rust_unnamed.gTD.Control.set_DirectionPID(OHCI_TD_DIRECTION_PID_IN as ULONG);
        } else {
            (*TD).HwTD.c2rust_unnamed.gTD.Control.set_BufferRounding(TRUE as ULONG);
            (*TD).HwTD.c2rust_unnamed.gTD.Control.set_DirectionPID(OHCI_TD_DIRECTION_PID_OUT as ULONG);
        }

        (*TD).Flags |= OHCI_HCD_TD_FLAG_PROCESSED as ULONG;
        (*TD).OhciTransfer = OhciTransfer;

        if (*TransferParameters).TransferBufferLength != 0 {
            TransferedLen = OHCI_MapTransferToTD(
                OhciExtension,
                TransferedLen,
                OhciTransfer,
                TD,
                SGList,
            );
        } else {
            // ASSERT(SGList->SgElementCount == 0);
            (*TD).TransferLen = 0 as ULONG;
        }

        PrevTD = TD;

        TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
        (*OhciTransfer).PendingTDs = (*OhciTransfer).PendingTDs.wrapping_add(1);

        (*PrevTD).HwTD.c2rust_unnamed.gTD.NextTD = (*TD).PhysicalAddress;
        (*PrevTD).NextTDVa = TD;

        if !(TransferedLen < (*TransferParameters).TransferBufferLength) {
            break;
        }
    }

    if (*TransferParameters).TransferFlags & USBD_SHORT_TRANSFER_OK != 0 {
        (*PrevTD).HwTD.c2rust_unnamed.gTD.Control.set_BufferRounding(TRUE as ULONG);
        (*OhciTransfer).Flags |= OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK as ULONG;
    }

    (*PrevTD).HwTD.c2rust_unnamed.gTD.Control.set_DelayInterrupt(OHCI_TD_INTERRUPT_IMMEDIATE as ULONG);

    /* The last TD in a chain is not used in a transfer. The controller does not access it
     * so it will be used for chaining a next transfer to it (OpenHCI spec, 4.6)
     */
    /* TD->HwTD.c2rust_unnamed.gTD.NextTD = 0; */
    (*TD).NextTDVa = core::ptr::null_mut();

    (*OhciTransfer).NextTD = TD;
    (*OhciEndpoint).HcdTailP = TD;

    (*(*OhciEndpoint).HcdED).HwED.TailPointer = (*TD).PhysicalAddress;

    OHCI_EnableList(OhciExtension, OhciEndpoint);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SubmitTransfer(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    ohciTransfer: PVOID,
    SGList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let OhciTransfer: POHCI_TRANSFER = ohciTransfer as POHCI_TRANSFER;
    let TransferType: ULONG;

    DPRINT_OHCI!(b"OHCI_SubmitTransfer: ... \n\0".as_ptr() as *const i8);

    RtlZeroMemory(
        OhciTransfer as *mut core::ffi::c_void,
        core::mem::size_of::<OHCI_TRANSFER>(),
    );

    (*OhciTransfer).TransferParameters = TransferParameters;
    (*OhciTransfer).OhciEndpoint = OhciEndpoint;

    TransferType = (*OhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        return OHCI_ControlTransfer(
            OhciExtension,
            OhciEndpoint,
            TransferParameters,
            OhciTransfer,
            SGList,
        );
    }

    if TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        return OHCI_BulkOrInterruptTransfer(
            OhciExtension,
            OhciEndpoint,
            TransferParameters,
            OhciTransfer,
            SGList,
        );
    }

    return MP_STATUS_FAILURE as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SubmitIsoTransfer(
    mut ohciExtension: PVOID,
    mut ohciEndpoint: PVOID,
    mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    mut ohciTransfer: PVOID,
    mut isoParameters: PVOID,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) OHCI_SubmitIsoTransfer: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1579 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1579 as ::core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_ProcessDoneTD(
    OhciExtension: POHCI_EXTENSION,
    TD: POHCI_HCD_TD,
    IsPortComplete: BOOLEAN,
) {
    let mut OhciTransfer: POHCI_TRANSFER;
    let mut OhciEndpoint: POHCI_ENDPOINT;
    let mut Buffer: ULONG;
    let mut BufferEnd: ULONG;
    let mut Length: ULONG;

    DPRINT_OHCI!(b"OHCI_ProcessDoneTD: ... \n\0".as_ptr() as *const i8);

    OhciTransfer = (*TD).OhciTransfer as POHCI_TRANSFER;
    OhciEndpoint = (*OhciTransfer).OhciEndpoint;

    (*OhciTransfer).PendingTDs -= 1;

    Buffer = (*TD).HwTD.c2rust_unnamed.gTD.CurrentBuffer;
    BufferEnd = (*TD).HwTD.c2rust_unnamed.gTD.BufferEnd;

    if (*TD).Flags & OHCI_HCD_TD_FLAG_NOT_ACCESSED != 0 {
        (*TD).HwTD.c2rust_unnamed.gTD.Control.set_ConditionCode(OHCI_TD_CONDITION_NO_ERROR as ULONG);
    } else {
        if (*TD).HwTD.c2rust_unnamed.gTD.CurrentBuffer != 0 {
            if (*TD).TransferLen != 0 {
                Length = (BufferEnd & (PAGE_SIZE - 1) as ULONG)
                    .wrapping_sub(Buffer & (PAGE_SIZE - 1) as ULONG);

                Length = Length.wrapping_add(1);

                if Buffer >> PAGE_SHIFT != BufferEnd >> PAGE_SHIFT {
                    Length = Length.wrapping_add(PAGE_SIZE as ULONG);
                }

                (*TD).TransferLen = (*TD).TransferLen.wrapping_sub(Length);
            }
        }

        if (*TD).HwTD.c2rust_unnamed.gTD.Control.DirectionPID() != OHCI_TD_DIRECTION_PID_SETUP {
            (*OhciTransfer).TransferLen = (*OhciTransfer).TransferLen.wrapping_add((*TD).TransferLen);
        }

        if (*TD).HwTD.c2rust_unnamed.gTD.Control.ConditionCode() != 0 {
            (*OhciTransfer).USBDStatus =
                (USBD_STATUS_HALTED | (*TD).HwTD.c2rust_unnamed.gTD.Control.ConditionCode() as i32) as USBD_STATUS;
        }
    }

    (*TD).Flags = 0;
    (*TD).HwTD.c2rust_unnamed.gTD.NextTD = 0;
    (*TD).OhciTransfer = core::ptr::null_mut();

    (*TD).DoneLink.Flink = core::ptr::null_mut();
    (*TD).DoneLink.Blink = core::ptr::null_mut();

    if IsPortComplete != 0 && (*OhciTransfer).PendingTDs == 0 {
        (RegPacket.UsbPortCompleteTransfer.expect("non-null function pointer"))(
            OhciExtension as PVOID,
            OhciEndpoint as PVOID,
            (*OhciTransfer).TransferParameters as PVOID,
            (*OhciTransfer).USBDStatus,
            (*OhciTransfer).TransferLen,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_ProcessDoneIsoTD(
    mut OhciExtension: POHCI_EXTENSION,
    mut TD: POHCI_HCD_TD,
    mut IsPortComplete: BOOLEAN,
) {
    if DbgPrint(
        b"(%s:%d) OHCI_ProcessDoneIsoTD: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const core::ffi::c_char,
        1660 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char,
            1660 as core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_AbortTransfer(
    mut ohciExtension: PVOID,
    mut ohciEndpoint: PVOID,
    mut ohciTransfer: PVOID,
    mut CompletedLength: PULONG,
) {
    let mut OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut OhciTransfer: POHCI_TRANSFER = ohciTransfer as POHCI_TRANSFER;
    let mut ED: POHCI_HCD_ED = (*OhciEndpoint).HcdED;
    let mut TD: POHCI_HCD_TD;
    let mut NextTD: POHCI_HCD_TD;
    let mut LastTD: POHCI_HCD_TD = core::ptr::null_mut();
    let mut ix: ULONG;
    let mut IsIsoEndpoint: BOOLEAN;
    let mut IsProcessed: BOOLEAN = 0 as BOOLEAN;

    IsIsoEndpoint = ((*OhciEndpoint).EndpointProperties.TransferType
        == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG) as i32 as BOOLEAN;

    NextTD = (RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer"))(
        (*ED).HwED.HeadPointer & OHCI_ED_HEAD_POINTER_MASK as ULONG,
        OhciExtension as PVOID,
        OhciEndpoint as PVOID,
    ) as POHCI_HCD_TD;

    if (*NextTD).OhciTransfer == OhciTransfer {
        LastTD = (*OhciTransfer).NextTD;

        (*ED).HwED.HeadPointer = (*LastTD).PhysicalAddress
            | ((*ED).HwED.HeadPointer & OHCI_ED_HEAD_POINTER_CARRY as ULONG);

        (*OhciEndpoint).HcdHeadP = LastTD;

        ix = 0 as ULONG;
        while ix < (*OhciEndpoint).MaxTransferDescriptors {
            TD = (*OhciEndpoint).FirstTD.offset(ix as isize) as POHCI_HCD_TD;

            if (*TD).OhciTransfer != OhciTransfer {
                ix = ix.wrapping_add(1);
                continue;
            }

            if IsIsoEndpoint != 0 {
                OHCI_ProcessDoneIsoTD(OhciExtension, TD, 0 as BOOLEAN);
            } else {
                OHCI_ProcessDoneTD(OhciExtension, TD, 0 as BOOLEAN);
            }

            ix = ix.wrapping_add(1);
        }

        *CompletedLength = (*OhciTransfer).TransferLen as ULONG;
        return;
    }

    if NextTD == (*OhciEndpoint).HcdHeadP {
        IsProcessed = 1 as BOOLEAN;
    }

    TD = (*OhciEndpoint).HcdHeadP;
    while TD != NextTD {
        if (*TD).OhciTransfer != OhciTransfer {
            TD = (*TD).NextTDVa as POHCI_HCD_TD;
            continue;
        }

        if (*OhciEndpoint).HcdHeadP == TD {
            (*OhciEndpoint).HcdHeadP = (*TD).NextTDVa as POHCI_HCD_TD;
        }

        if IsIsoEndpoint != 0 {
            OHCI_ProcessDoneIsoTD(OhciExtension, TD, 0 as BOOLEAN);
        } else {
            OHCI_ProcessDoneTD(OhciExtension, TD, 0 as BOOLEAN);
        }

        IsProcessed = 1 as BOOLEAN;

        TD = (*TD).NextTDVa as POHCI_HCD_TD;
    }

    if IsProcessed == 0 {
        TD = (*OhciEndpoint).HcdHeadP;
        while (*TD).OhciTransfer != OhciTransfer {
            if TD == (*OhciEndpoint).HcdTailP {
                TD = core::ptr::null_mut();
                break;
            }
            LastTD = TD;
            TD = (*TD).NextTDVa as POHCI_HCD_TD;
        }

        while (*TD).OhciTransfer == OhciTransfer {
            if TD == (*OhciEndpoint).HcdTailP {
                break;
            }

            if IsIsoEndpoint != 0 {
                OHCI_ProcessDoneIsoTD(OhciExtension, TD, 0 as BOOLEAN);
            } else {
                OHCI_ProcessDoneTD(OhciExtension, TD, 0 as BOOLEAN);
            }

            TD = (*TD).NextTDVa as POHCI_HCD_TD;
        }

        (*(*LastTD).OhciTransfer).NextTD = TD;

        (*LastTD).NextTDVa = TD as *mut _OHCI_HCD_TD;
        (*LastTD).HwTD.c2rust_unnamed.gTD.NextTD = (*TD).PhysicalAddress;
    }

    *CompletedLength = (*OhciTransfer).TransferLen as ULONG;

    if (*OhciTransfer).TransferLen != 0 {
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_GetEndpointState(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
) -> ULONG {
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut ED: POHCI_HCD_ED;

    DPRINT_OHCI!(b"OHCI_GetEndpointState: ... \n\0".as_ptr() as *const i8);

    ED = (*OhciEndpoint).HcdED;

    if (*ED).Flags as u32 & OHCI_HCD_TD_FLAG_NOT_ACCESSED as u32 != 0 {
        return USBPORT_ENDPOINT_REMOVE as ULONG;
    }

    if (*ED).HwED.EndpointControl.c2rust_unnamed.sKip() != 0 {
        return USBPORT_ENDPOINT_PAUSED as ULONG;
    }

    return USBPORT_ENDPOINT_ACTIVE as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_RemoveEndpointFromSchedule(OhciEndpoint: POHCI_ENDPOINT) {
    let mut ED: POHCI_HCD_ED;
    let mut PreviousED: POHCI_HCD_ED;
    let HeadED: POHCI_STATIC_ED;

    DPRINT_OHCI!(b"OHCI_RemoveEndpointFromSchedule \n\0".as_ptr() as *const i8);

    ED = (*OhciEndpoint).HcdED;
    HeadED = (*OhciEndpoint).HeadED;

    if core::ptr::addr_of_mut!((*HeadED).Link) == (*ED).HcdEDLink.Blink {
        if (*HeadED).Type == OHCI_STATIC_ED_TYPE_CONTROL as ULONG
            || (*HeadED).Type == OHCI_STATIC_ED_TYPE_BULK as ULONG
        {
            WRITE_REGISTER_ULONG((*HeadED).pNextED as *mut ULONG, (*ED).HwED.NextED);
        } else if (*HeadED).Type == OHCI_STATIC_ED_TYPE_INTERRUPT as ULONG {
            *(*HeadED).pNextED = (*ED).HwED.NextED;
        } else {
            // DPRINT removed
            DbgBreakPoint();
        }
    } else {
        PreviousED = ((*ED).HcdEDLink.Blink as usize)
            .wrapping_sub(core::mem::offset_of!(OHCI_HCD_ED, HcdEDLink))
            as *mut OHCI_HCD_ED;

        (*PreviousED).HwED.NextED = (*ED).HwED.NextED;
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*ED).HcdEDLink));

    (*OhciEndpoint).HeadED = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SetEndpointState(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
    EndpointState: ULONG,
) {

    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut ED: POHCI_HCD_ED = core::ptr::null_mut();

    // DPRINT_OHCI!("OHCI_SetEndpointState: EndpointState - %x\n", EndpointState);

    ED = (*OhciEndpoint).HcdED;

    match EndpointState {
        2 => {
            (*ED).HwED.EndpointControl.set_sKip(1 as ULONG);
        }
        3 => {
            (*ED).HwED.EndpointControl.set_sKip(0 as ULONG);
            OHCI_EnableList(OhciExtension, OhciEndpoint);
        }
        4 => {
            (*ED).HwED.EndpointControl.set_sKip(1 as ULONG);
            (*ED).Flags |= OHCI_HCD_ED_FLAG_NOT_ACCESSED as core::ffi::c_ulong;
            OHCI_RemoveEndpointFromSchedule(OhciEndpoint);
        }
        _ => {
            if 0 as core::ffi::c_int == 0 {
                RtlAssert(
                    b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
                    b"drivers/usb/usbohci/usbohci.c\0"
                        as *const u8 as *const core::ffi::c_char as PVOID,
                    1878 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            };
        }
    };
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_PollAsyncEndpoint(
    OhciExtension: POHCI_EXTENSION,
    OhciEndpoint: POHCI_ENDPOINT,
) {
    let mut current_block: u64;
    let mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS = core::ptr::null_mut();
    let ED: POHCI_HCD_ED;
    let mut NextTdPA: ULONG_PTR;
    let mut NextTD: POHCI_HCD_TD;
    let mut TD: POHCI_HCD_TD;
    let DoneList: PLIST_ENTRY;
    let mut OhciTransfer: POHCI_TRANSFER;
    let mut ControlStatusTD: POHCI_HCD_TD;
    let mut PhysicalAddress: ULONG_PTR;
    let mut TransferNumber: ULONG;
    let mut transfer: POHCI_TRANSFER;
    let mut ConditionCode: UCHAR;
    let mut IsResetOnHalt: BOOLEAN = 0;

    ED = (*OhciEndpoint).HcdED;
    NextTdPA = ((((*ED).HwED.HeadPointer & (OHCI_ED_HEAD_POINTER_MASK as usize as usize) as u32)) as usize);

    if NextTdPA == 0 {
        OHCI_DumpHcdED(ED);
        DbgBreakPoint();
    }

    NextTD = RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer")(
        NextTdPA as ULONG,
        OhciExtension as PVOID,
        OhciEndpoint as PVOID,
    ) as POHCI_HCD_TD;

    if (*ED).HwED.HeadPointer & OHCI_ED_HEAD_POINTER_HALT == 0 {
        // ProcessListTDs
        TD = (*OhciEndpoint).HcdHeadP;
        while TD != NextTD {
            OHCI_DumpHcdTD(TD);
            (*TD).Flags |= OHCI_HCD_TD_FLAG_DONE;
            InsertTailList(core::ptr::addr_of_mut!((*OhciEndpoint).TDList), core::ptr::addr_of_mut!((*TD).DoneLink));
            TD = (*TD).NextTDVa;
        }
    } else {
        OHCI_DumpHcdED(ED);

        IsResetOnHalt = (((*ED).Flags & OHCI_HCD_ED_FLAG_RESET_ON_HALT) != 0) as BOOLEAN;

        TD = (*OhciEndpoint).HcdHeadP;
        loop {
            if TD.is_null() {
                OHCI_DumpHcdED(ED);
                DbgBreakPoint();
            }

            if TD == NextTD {
                break;
            }

            OhciTransfer = (*TD).OhciTransfer;
            ConditionCode = (*TD).HwTD.c2rust_unnamed.gTD.Control.ConditionCode() as UCHAR;

            OHCI_DumpHcdTD(TD);

            match ConditionCode as ULONG {
                OHCI_TD_CONDITION_NO_ERROR => {
                    (*TD).Flags |= OHCI_HCD_TD_FLAG_DONE;
                    InsertTailList(core::ptr::addr_of_mut!((*OhciEndpoint).TDList), core::ptr::addr_of_mut!((*TD).DoneLink));
                    current_block = 12147880666119273379;
                }
                OHCI_TD_CONDITION_NOT_ACCESSED => {
                    (*TD).Flags |= OHCI_HCD_TD_FLAG_DONE | OHCI_HCD_TD_FLAG_NOT_ACCESSED;
                    InsertTailList(core::ptr::addr_of_mut!((*OhciEndpoint).TDList), core::ptr::addr_of_mut!((*TD).DoneLink));
                    current_block = 12147880666119273379;
                }
                OHCI_TD_CONDITION_DATA_UNDERRUN => {
                    if (*OhciTransfer).Flags & OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK != 0 {
                        IsResetOnHalt = 1;
                        (*TD).HwTD.c2rust_unnamed.gTD.Control.set_ConditionCode(OHCI_TD_CONDITION_NO_ERROR as ULONG);

                        ControlStatusTD = (*OhciTransfer).ControlStatusTD;

                        if (*TD).Flags & OHCI_HCD_TD_FLAG_CONTROL_STATUS == 0
                            && !ControlStatusTD.is_null()
                        {
                            PhysicalAddress = ((*ControlStatusTD).PhysicalAddress as usize);
                            PhysicalAddress |= ((((*ED).HwED.HeadPointer & (OHCI_ED_HEAD_POINTER_FLAGS_MASK as usize as usize) as u32)) as usize);

                            (*ED).HwED.HeadPointer = (PhysicalAddress as u32);

                            NextTD = (*OhciTransfer).ControlStatusTD;
                        } else {
                            TransferParameters = (*OhciTransfer).TransferParameters;

                            if (*TransferParameters).IsTransferSplited != 0 {
                                TransferNumber = (*TransferParameters).TransferCounter;
                                transfer = OhciTransfer;

                                loop {
                                    transfer = (*(*transfer).NextTD).OhciTransfer;
                                    NextTD = (*transfer).NextTD;
                                    if !(!transfer.is_null()
                                        && TransferNumber == (*(*transfer).TransferParameters).TransferCounter)
                                    {
                                        break;
                                    }
                                }

                                PhysicalAddress = ((*NextTD).PhysicalAddress as usize);
                                PhysicalAddress |= ((((*ED).HwED.HeadPointer & (OHCI_ED_HEAD_POINTER_FLAGS_MASK as usize as usize) as u32)) as usize);

                                (*ED).HwED.HeadPointer = (PhysicalAddress as u32);
                            } else {
                                PhysicalAddress = ((*(*OhciTransfer).NextTD).PhysicalAddress as usize);
                                PhysicalAddress |= ((((*ED).HwED.HeadPointer & (OHCI_ED_HEAD_POINTER_FLAGS_MASK as usize as usize) as u32)) as usize);

                                (*ED).HwED.HeadPointer = (PhysicalAddress as u32);

                                NextTD = (*OhciTransfer).NextTD;
                            }
                        }

                        (*TD).Flags |= OHCI_HCD_TD_FLAG_DONE;
                        InsertTailList(core::ptr::addr_of_mut!((*OhciEndpoint).TDList), core::ptr::addr_of_mut!((*TD).DoneLink));
                        current_block = 12147880666119273379;
                    } else {
                        current_block = 6643962143486670607;
                    }
                }
                _ => {
                    current_block = 6643962143486670607;
                }
            }

            match current_block {
                6643962143486670607 => {
                    (*TD).Flags |= OHCI_HCD_TD_FLAG_DONE;
                    InsertTailList(core::ptr::addr_of_mut!((*OhciEndpoint).TDList), core::ptr::addr_of_mut!((*TD).DoneLink));

                    (*ED).HwED.HeadPointer = (*(*OhciTransfer).NextTD).PhysicalAddress
                        | ((*ED).HwED.HeadPointer & OHCI_ED_HEAD_POINTER_FLAGS_MASK);

                    NextTD = (*OhciTransfer).NextTD;
                }
                _ => {}
            }

            TD = (*TD).NextTDVa;
        }
    }

    // HandleDoneList
    TD = NextTD;
    (*OhciEndpoint).HcdHeadP = NextTD;

    DoneList = core::ptr::addr_of_mut!((*OhciEndpoint).TDList);

    while !IsListEmpty(DoneList) {
        TD = ((*DoneList).Flink as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(OHCI_HCD_TD, DoneLink) as ULONG_PTR)
            as POHCI_HCD_TD;

        RemoveHeadList(DoneList);

        if (*TD).Flags & OHCI_HCD_TD_FLAG_DONE != 0
            && (*TD).Flags & OHCI_HCD_TD_FLAG_PROCESSED != 0
        {
            OHCI_ProcessDoneTD(OhciExtension, TD, 1);
        }
    }

    if IsResetOnHalt != 0 {
        (*ED).HwED.HeadPointer &= !OHCI_ED_HEAD_POINTER_HALT;
    }

    let _ = current_block;
    let _ = TransferParameters;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_PollIsoEndpoint(
    mut OhciExtension: POHCI_EXTENSION,
    mut OhciEndpoint: POHCI_ENDPOINT,
) {
    if DbgPrint(
        b"(%s:%d) OHCI_PollAsyncEndpoint: UNIMPLEMENTED. FIXME \n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const core::ffi::c_char,
        2083 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char,
            2083 as core::ffi::c_int,
        );
    }
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char as PVOID,
            2084 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_PollEndpoint(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let TransferType: ULONG;

    DPRINT_OHCI!(
        b"OHCI_PollEndpoint: OhciExtension - %p, Endpoint - %p\n\0".as_ptr() as *const i8,
        OhciExtension,
        OhciEndpoint,
    );

    TransferType = (*OhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        OHCI_PollIsoEndpoint(OhciExtension, OhciEndpoint);
        return;
    }

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        OHCI_PollAsyncEndpoint(OhciExtension, OhciEndpoint);
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_CheckController(ohciExtension: PVOID) {
    const OHCI_HC_STATE_OPERATIONAL: i32 = 2;
    const USBPORT_INVALIDATE_CONTROLLER_RESET: u32 = 1;

    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let HcControlReg: PULONG;
    let mut HcControl: OHCI_REG_CONTROL = core::mem::zeroed();
    let FmNumber: ULONG;
    let FmDiff: USHORT;
    let HcHCCA: POHCI_HCCA;

    OperationalRegs = (*OhciExtension).OperationalRegs;

    if OHCI_HardwarePresent(OhciExtension, 1 as BOOLEAN) == 0 {
        return;
    }

    HcControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    HcControl.AsULONG = READ_REGISTER_ULONG(HcControlReg as *mut ULONG);

    if HcControl.HostControllerFunctionalState() as i32 != OHCI_HC_STATE_OPERATIONAL {
        return;
    }

    FmNumber = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcFmNumber) as *mut ULONG);
    FmDiff = FmNumber.wrapping_sub((*OhciExtension).HcdFmNumber) as USHORT;

    if FmNumber == 0 || (FmDiff as i32) < 5 {
        return;
    }

    HcHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;
    (*OhciExtension).HcdFmNumber = FmNumber;

    if (*HcHCCA).Pad1 as i32 == 0 {
        (*HcHCCA).Pad1 = 0xBAD1 as USHORT;
        return;
    }

    DbgPrint(
        b"OHCI_CheckController: HcHCCA->Pad1 - %x\n\0".as_ptr() as PCSTR,
        (*HcHCCA).Pad1 as i32,
    );

    if (*HcHCCA).Pad1 as i32 == 0xBAD1 {
        (*HcHCCA).Pad1 = 0xBAD2 as USHORT;
    } else if (*HcHCCA).Pad1 as i32 == 0xBAD2 {
        (*HcHCCA).Pad1 = 0xBAD3 as USHORT;

        RegPacket.UsbPortInvalidateController.expect("non-null function pointer")(
            OhciExtension as PVOID,
            USBPORT_INVALIDATE_CONTROLLER_RESET as ULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_Get32BitFrameNumber(ohciExtension: PVOID) -> ULONG {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let HcHCCA: POHCI_HCCA;
    let fm: ULONG;
    let hp: ULONG;

    HcHCCA = core::ptr::addr_of_mut!((*(*OhciExtension).HcResourcesVA).HcHCCA) as POHCI_HCCA;

    hp = (*OhciExtension).FrameHighPart;
    fm = (*HcHCCA).FrameNumber as ULONG;

    DPRINT_OHCI!(
        b"OHCI_Get32BitFrameNumber: hp - %lX, fm - %lX\n\0".as_ptr() as *const i8,
        hp,
        fm,
    );

    return (fm & 0x7FFF as ULONG | hp).wrapping_add((fm ^ hp) & 0x8000 as ULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_InterruptNextSOF(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let InterruptEnableReg: PULONG;
    let mut IntEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    DPRINT_OHCI!(
        b"OHCI_InterruptNextSOF: OhciExtension - %p\n\0".as_ptr() as *const i8,
        OhciExtension,
    );

    OperationalRegs = (*OhciExtension).OperationalRegs;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;

    /* Enable interrupt generation due to Start of Frame */
    IntEnable.AsULONG = 0;
    IntEnable.set_StartofFrame(1);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, IntEnable.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_EnableInterrupts(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let InterruptEnableReg: PULONG;
    let mut IntEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    DPRINT_OHCI!(
        b"OHCI_EnableInterrupts: OhciExtension - %p\n\0".as_ptr() as *const i8,
        OhciExtension,
    );

    OperationalRegs = (*OhciExtension).OperationalRegs;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;

    /*  Enable interrupt generation */
    IntEnable.AsULONG = 0;
    IntEnable.set_MasterInterruptEnable(1);

    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, IntEnable.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_DisableInterrupts(ohciExtension: PVOID) {
    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let InterruptDisableReg: PULONG;
    let mut IntDisable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();

    DPRINT_OHCI!(b"OHCI_DisableInterrupts\n\0".as_ptr() as *const i8);

    OperationalRegs = (*OhciExtension).OperationalRegs;
    InterruptDisableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptDisable) as PULONG;

    /*  Disable interrupt generation */
    IntDisable.AsULONG = 0;
    IntDisable.set_MasterInterruptEnable(1);

    WRITE_REGISTER_ULONG(InterruptDisableReg as *mut ULONG, IntDisable.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_PollController(ohciExtension: PVOID) {
    if DbgPrint(
        b"(%s:%d) OHCI_PollController: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8,
        b"drivers/usb/usbohci/usbohci.c\0".as_ptr() as *const i8,
        2261 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"drivers/usb/usbohci/usbohci.c\0".as_ptr() as *const i8,
            2261 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SetEndpointDataToggle(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
    DataToggle: ULONG,
) {
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut ED: POHCI_HCD_ED;

    DPRINT_OHCI!(
        b"OHCI_SetEndpointDataToggle: Endpoint - %p, DataToggle - %x\n\0".as_ptr() as *const i8,
        OhciEndpoint,
        DataToggle,
    );

    ED = (*OhciEndpoint).HcdED;

    if DataToggle != 0 {
        (*ED).HwED.HeadPointer |= OHCI_ED_HEAD_POINTER_CARRY as u32;
    } else {
        (*ED).HwED.HeadPointer &= !(OHCI_ED_HEAD_POINTER_CARRY as u32);
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_GetEndpointStatus(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
) -> ULONG {
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut ED: POHCI_HCD_ED;
    let mut EndpointStatus: ULONG = USBPORT_ENDPOINT_RUN as ULONG;

    DPRINT_OHCI!(b"OHCI_GetEndpointStatus: ... \n\0".as_ptr() as *const i8);

    ED = (*OhciEndpoint).HcdED;

    if ((*ED).HwED.HeadPointer as u32 & OHCI_ED_HEAD_POINTER_HALT as u32) != 0
        && ((*ED).Flags as u32 & OHCI_HCD_ED_FLAG_RESET_ON_HALT as u32) == 0
    {
        EndpointStatus = USBPORT_ENDPOINT_HALT as ULONG;
    }

    return EndpointStatus;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_SetEndpointStatus(
    ohciExtension: PVOID,
    ohciEndpoint: PVOID,
    EndpointStatus: ULONG,
) {
    const USBPORT_ENDPOINT_RUN: u32 = 0x00;
    const USBPORT_ENDPOINT_HALT: u32 = 0x01;
    const OHCI_ED_HEAD_POINTER_HALT: u32 = 1;

    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let OhciEndpoint: POHCI_ENDPOINT = ohciEndpoint as POHCI_ENDPOINT;
    let mut ED: POHCI_HCD_ED = core::ptr::null_mut();

    DPRINT_OHCI!(
        b"OHCI_SetEndpointStatus: Endpoint - %p, EndpointStatus - %lX\n\0".as_ptr() as *const i8,
        OhciEndpoint,
        EndpointStatus,
    );

    if EndpointStatus == USBPORT_ENDPOINT_RUN as ULONG {
        ED = (*OhciEndpoint).HcdED;
        (*ED).HwED.HeadPointer &= !(OHCI_ED_HEAD_POINTER_HALT as ULONG);

        OHCI_EnableList(OhciExtension, OhciEndpoint);
    } else if EndpointStatus == USBPORT_ENDPOINT_HALT as ULONG {
        if 0 == 0 {
            RtlAssert(
                b"FALSE\0".as_ptr() as *const i8 as PVOID,
                b"drivers/usb/usbohci/usbohci.c\0".as_ptr() as *const i8 as PVOID,
                2330 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_ResetController(ohciExtension: PVOID) {

    let OhciExtension: POHCI_EXTENSION = ohciExtension as POHCI_EXTENSION;
    let mut OperationalRegs: POHCI_OPERATIONAL_REGISTERS;
    let FrameNumber: ULONG;
    let ControlReg: PULONG;
    let CommandStatusReg: PULONG;
    let InterruptEnableReg: PULONG;
    let FmIntervalReg: PULONG;
    let RhStatusReg: PULONG;
    let mut PortStatusReg: PULONG;
    let mut ControlBak: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut Control: OHCI_REG_CONTROL = core::mem::zeroed();
    let mut CommandStatus: OHCI_REG_COMMAND_STATUS = core::mem::zeroed();
    let mut IntEnable: OHCI_REG_INTERRUPT_ENABLE_DISABLE = core::mem::zeroed();
    let HCCA: ULONG_PTR;
    let ControlHeadED: ULONG_PTR;
    let BulkHeadED: ULONG_PTR;
    let mut FrameInterval: OHCI_REG_FRAME_INTERVAL = core::mem::zeroed();
    let PeriodicStart: ULONG_PTR;
    let LSThreshold: ULONG_PTR;
    let mut RhStatus: OHCI_REG_RH_STATUS = core::mem::zeroed();
    let RhDescriptorA: OHCI_REG_RH_DESCRIPTORA;
    let mut PortStatus: OHCI_REG_RH_PORT_STATUS = core::mem::zeroed();
    let NumPorts: ULONG;
    let mut ix: ULONG;

    OperationalRegs = (*OhciExtension).OperationalRegs;

    ControlReg = core::ptr::addr_of_mut!((*OperationalRegs).HcControl) as PULONG;
    CommandStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcCommandStatus) as PULONG;
    InterruptEnableReg = core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable) as PULONG;
    FmIntervalReg = core::ptr::addr_of_mut!((*OperationalRegs).HcFmInterval) as PULONG;
    RhStatusReg = core::ptr::addr_of_mut!((*OperationalRegs).HcRhStatus) as PULONG;

    /* Backup FrameNumber from HcHCCA */
    FrameNumber = (*(*OhciExtension).HcResourcesVA).HcHCCA.FrameNumber as ULONG;

    /* Backup registers */
    ControlBak.AsULONG = READ_REGISTER_ULONG(ControlReg as *mut ULONG);
    HCCA = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcHCCA) as *mut ULONG) as ULONG_PTR;
    ControlHeadED = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcControlHeadED) as *mut ULONG) as ULONG_PTR;
    BulkHeadED = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcBulkHeadED) as *mut ULONG) as ULONG_PTR;
    FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg as *mut ULONG);
    PeriodicStart = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcPeriodicStart) as *mut ULONG) as ULONG_PTR;
    LSThreshold = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcLSThreshold) as *mut ULONG) as ULONG_PTR;

    /* Reset HostController */
    CommandStatus.AsULONG = 0;
    CommandStatus.set_HostControllerReset(1);
    WRITE_REGISTER_ULONG(CommandStatusReg as *mut ULONG, CommandStatus.AsULONG);

    KeStallExecutionProcessor(10);

    /* Restore registers */
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcHCCA) as *mut ULONG, HCCA as ULONG);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcControlHeadED) as *mut ULONG, ControlHeadED as ULONG);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcBulkHeadED) as *mut ULONG, BulkHeadED as ULONG);

    /* Set OPERATIONAL state for HC */
    Control.AsULONG = 0;
    Control.set_HostControllerFunctionalState(OHCI_HC_STATE_OPERATIONAL as ULONG);
    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, Control.AsULONG);

    /* Set Toggle bit for FmInterval register */
    FrameInterval.set_FrameIntervalToggle(1);
    WRITE_REGISTER_ULONG(FmIntervalReg as *mut ULONG, FrameInterval.AsULONG);

    /* Restore registers */
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcFmNumber) as *mut ULONG, FrameNumber);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcPeriodicStart) as *mut ULONG, PeriodicStart as ULONG);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcLSThreshold) as *mut ULONG, LSThreshold as ULONG);

    /* Setup RhStatus register */
    RhStatus.AsULONG = 0;
    RhStatus.set_SetRemoteWakeupEnable(1);
    RhStatus.set_SetGlobalPower(1);
    WRITE_REGISTER_ULONG(RhStatusReg as *mut ULONG, RhStatus.AsULONG);

    /* Setup RH PortStatus registers */
    RhDescriptorA = OHCI_ReadRhDescriptorA(OhciExtension);
    NumPorts = RhDescriptorA.NumberDownstreamPorts() as ULONG;

    PortStatus.AsULONG = 0;
    PortStatus.set_SetPortPower(1);

    ix = 0;
    while ix < NumPorts {
        PortStatusReg = (core::ptr::addr_of_mut!((*OperationalRegs).HcRhPortStatus) as *mut OHCI_REG_RH_PORT_STATUS).offset(ix as isize) as PULONG;
        WRITE_REGISTER_ULONG(PortStatusReg as *mut ULONG, PortStatus.AsULONG);
        ix = ix.wrapping_add(1);
    }

    /* Restore HcControl register */
    ControlBak.set_HostControllerFunctionalState(OHCI_HC_STATE_OPERATIONAL as ULONG);
    WRITE_REGISTER_ULONG(ControlReg as *mut ULONG, ControlBak.AsULONG);

    /* Setup HcInterruptEnable register */
    IntEnable.AsULONG = 0xFFFFFFFF;
    IntEnable.set_Reserved1(0);
    WRITE_REGISTER_ULONG(InterruptEnableReg as *mut ULONG, IntEnable.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_StartSendOnePacket(
    ohciExtension: PVOID,
    PacketParameters: PVOID,
    Data: PVOID,
    pDataLength: PULONG,
    BufferVA: PVOID,
    BufferPA: PVOID,
    BufferLength: ULONG,
    pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) OHCI_StartSendOnePacket: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const core::ffi::c_char,
        2451 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char,
            2451 as core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_EndSendOnePacket(
    ohciExtension: PVOID,
    PacketParameters: PVOID,
    Data: PVOID,
    pDataLength: PULONG,
    BufferVA: PVOID,
    BufferPA: PVOID,
    BufferLength: ULONG,
    pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    const MP_STATUS_SUCCESS: MPSTATUS = 0;
    if DbgPrint(
        b"(%s:%d) OHCI_EndSendOnePacket: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const core::ffi::c_char,
        2466 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char,
            2466 as core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_PassThru(
    mut ohciExtension: PVOID,
    mut passThruParameters: PVOID,
    mut ParameterLength: ULONG,
    mut pParameters: PVOID,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) OHCI_PassThru: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const core::ffi::c_char,
        2477 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const core::ffi::c_char,
            2477 as core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_Unload(mut DriverObject: PDRIVER_OBJECT) {
    if DbgPrint(
        b"(%s:%d) OHCI_Unload: Not supported\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        2486 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            2486 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn OHCI_FlushInterrupts(mut uhciExtension: PVOID) {
    if DbgPrint(
        b"(%s:%d) OHCI_FlushInterrupts: Not supported\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbohci/usbohci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        2496 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbohci/usbohci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            2496 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    DPRINT_OHCI!(
        b"DriverEntry: DriverObject - %p, RegistryPath - %wZ\n\0".as_ptr() as *const i8,
        DriverObject,
        RegistryPath,
    );

    RtlZeroMemory(
        core::ptr::addr_of_mut!(RegPacket) as *mut core::ffi::c_void,
        core::mem::size_of::<USBPORT_REGISTRATION_PACKET>(),
    );

    RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_OHCI as ULONG;

    RegPacket.MiniPortFlags =
        (USB_MINIPORT_FLAGS_INTERRUPT | USB_MINIPORT_FLAGS_MEMORY_IO) as ULONG;

    RegPacket.MiniPortBusBandwidth = TOTAL_USB11_BUS_BANDWIDTH as ULONG;

    RegPacket.MiniPortExtensionSize = ((core::mem::size_of::<OHCI_EXTENSION>() as SIZE_T as u32) as _);
    RegPacket.MiniPortEndpointSize = ((core::mem::size_of::<OHCI_ENDPOINT>() as SIZE_T as u32) as _);
    RegPacket.MiniPortTransferSize = ((core::mem::size_of::<OHCI_TRANSFER>() as SIZE_T as u32) as _);
    RegPacket.MiniPortResourcesSize = ((core::mem::size_of::<OHCI_HC_RESOURCES>() as SIZE_T as u32) as _);

    RegPacket.OpenEndpoint = Some(OHCI_OpenEndpoint);
    RegPacket.ReopenEndpoint = Some(OHCI_ReopenEndpoint);
    RegPacket.QueryEndpointRequirements = Some(OHCI_QueryEndpointRequirements);
    RegPacket.CloseEndpoint = Some(OHCI_CloseEndpoint);
    RegPacket.StartController = Some(OHCI_StartController);
    RegPacket.StopController = Some(OHCI_StopController);
    RegPacket.SuspendController = Some(OHCI_SuspendController);
    RegPacket.ResumeController = Some(OHCI_ResumeController);
    RegPacket.InterruptService = Some(OHCI_InterruptService);
    RegPacket.InterruptDpc = Some(OHCI_InterruptDpc);
    RegPacket.SubmitTransfer = Some(OHCI_SubmitTransfer);
    RegPacket.SubmitIsoTransfer = Some(OHCI_SubmitIsoTransfer);
    RegPacket.AbortTransfer = Some(OHCI_AbortTransfer);
    RegPacket.GetEndpointState = Some(OHCI_GetEndpointState);
    RegPacket.SetEndpointState = Some(OHCI_SetEndpointState);
    RegPacket.PollEndpoint = Some(OHCI_PollEndpoint);
    RegPacket.CheckController = Some(OHCI_CheckController);
    RegPacket.Get32BitFrameNumber = Some(OHCI_Get32BitFrameNumber);
    RegPacket.InterruptNextSOF = Some(OHCI_InterruptNextSOF);
    RegPacket.EnableInterrupts = Some(OHCI_EnableInterrupts);
    RegPacket.DisableInterrupts = Some(OHCI_DisableInterrupts);
    RegPacket.PollController = Some(OHCI_PollController);
    RegPacket.SetEndpointDataToggle = Some(OHCI_SetEndpointDataToggle);
    RegPacket.GetEndpointStatus = Some(OHCI_GetEndpointStatus);
    RegPacket.SetEndpointStatus = Some(OHCI_SetEndpointStatus);
    RegPacket.ResetController = Some(OHCI_ResetController);
    RegPacket.RH_GetRootHubData = Some(OHCI_RH_GetRootHubData);
    RegPacket.RH_GetStatus = Some(OHCI_RH_GetStatus);
    RegPacket.RH_GetPortStatus = Some(OHCI_RH_GetPortStatus);
    RegPacket.RH_GetHubStatus = Some(OHCI_RH_GetHubStatus);
    RegPacket.RH_SetFeaturePortReset = Some(OHCI_RH_SetFeaturePortReset);
    RegPacket.RH_SetFeaturePortPower = Some(OHCI_RH_SetFeaturePortPower);
    RegPacket.RH_SetFeaturePortEnable = Some(OHCI_RH_SetFeaturePortEnable);
    RegPacket.RH_SetFeaturePortSuspend = Some(OHCI_RH_SetFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnable = Some(OHCI_RH_ClearFeaturePortEnable);
    RegPacket.RH_ClearFeaturePortPower = Some(OHCI_RH_ClearFeaturePortPower);
    RegPacket.RH_ClearFeaturePortSuspend = Some(OHCI_RH_ClearFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnableChange = Some(OHCI_RH_ClearFeaturePortEnableChange);
    RegPacket.RH_ClearFeaturePortConnectChange = Some(OHCI_RH_ClearFeaturePortConnectChange);
    RegPacket.RH_ClearFeaturePortResetChange = Some(OHCI_RH_ClearFeaturePortResetChange);
    RegPacket.RH_ClearFeaturePortSuspendChange = Some(OHCI_RH_ClearFeaturePortSuspendChange);
    RegPacket.RH_ClearFeaturePortOvercurrentChange =
        Some(OHCI_RH_ClearFeaturePortOvercurrentChange);
    RegPacket.RH_DisableIrq = Some(OHCI_RH_DisableIrq);
    RegPacket.RH_EnableIrq = Some(OHCI_RH_EnableIrq);
    RegPacket.StartSendOnePacket = Some(OHCI_StartSendOnePacket);
    RegPacket.EndSendOnePacket = Some(OHCI_EndSendOnePacket);
    RegPacket.PassThru = Some(OHCI_PassThru);
    RegPacket.FlushInterrupts = Some(OHCI_FlushInterrupts);

    (*DriverObject).DriverUnload = core::mem::transmute(Some(OHCI_Unload as unsafe extern "system" fn(PDRIVER_OBJECT)));

    Status = USBPORT_RegisterUSBPortDriver(
        DriverObject,
        USB10_MINIPORT_INTERFACE_VERSION as ULONG,
        core::ptr::addr_of_mut!(RegPacket),
    );

    DPRINT_OHCI!(
        b"DriverEntry: USBPORT_RegisterUSBPortDriver return Status - %x\n\0".as_ptr() as *const i8,
        Status,
    );

    return Status;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB OHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBOHCI main driver functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbohci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_OHCI_TRACE
// #include "dbg_ohci.h"
//
// USBPORT_REGISTRATION_PACKET RegPacket;
//
// static const UCHAR Index[8] =
// {
//     ENDPOINT_INTERRUPT_1ms - 1,
//     ENDPOINT_INTERRUPT_2ms - 1,
//     ENDPOINT_INTERRUPT_4ms - 1,
//     ENDPOINT_INTERRUPT_8ms - 1,
//     ENDPOINT_INTERRUPT_16ms - 1,
//     ENDPOINT_INTERRUPT_32ms - 1,
//     ENDPOINT_INTERRUPT_32ms - 1,
//     ENDPOINT_INTERRUPT_32ms - 1
// };
//
// static const UCHAR Balance[OHCI_NUMBER_OF_INTERRUPTS] =
// {
//     0, 16, 8, 24, 4, 20, 12, 28, 2, 18, 10, 26, 6, 22, 14, 30,
//     1, 17, 9, 25, 5, 21, 13, 29, 3, 19, 11, 27, 7, 23, 15, 31
// };
//
// VOID
// NTAPI
// OHCI_DumpHcdED(POHCI_HCD_ED ED)
// {
//     DPRINT("ED                - %p\n", ED);
//     DPRINT("EndpointControl   - %X\n", ED->HwED.EndpointControl.AsULONG);
//     DPRINT("TailPointer       - %08X\n", ED->HwED.TailPointer);
//     DPRINT("HeadPointer       - %08X\n", ED->HwED.HeadPointer);
//     DPRINT("NextED            - %08X\n", ED->HwED.NextED);
// }
//
// VOID
// NTAPI
// OHCI_DumpHcdTD(POHCI_HCD_TD TD)
// {
//     DPRINT("TD                - %p\n", TD);
//     DPRINT("gTD.Control       - %08X\n", TD->HwTD.c2rust_unnamed.gTD.Control.AsULONG);
//     if (TD->HwTD.c2rust_unnamed.gTD.CurrentBuffer)
//         DPRINT("gTD.CurrentBuffer - %08X\n", TD->HwTD.c2rust_unnamed.gTD.CurrentBuffer);
//     if (TD->HwTD.c2rust_unnamed.gTD.NextTD)
//         DPRINT("gTD.NextTD        - %08X\n", TD->HwTD.c2rust_unnamed.gTD.NextTD);
//     if (TD->HwTD.c2rust_unnamed.gTD.BufferEnd)
//         DPRINT("gTD.BufferEnd     - %08X\n", TD->HwTD.c2rust_unnamed.gTD.BufferEnd);
//
//     if (TD->HwTD.c2rust_unnamed.SetupPacket.bmRequestType.B)
//         DPRINT("bmRequestType     - %02X\n", TD->HwTD.c2rust_unnamed.SetupPacket.bmRequestType.B);
//     if (TD->HwTD.c2rust_unnamed.SetupPacket.bRequest)
//         DPRINT("bRequest          - %02X\n", TD->HwTD.c2rust_unnamed.SetupPacket.bRequest);
//     if (TD->HwTD.c2rust_unnamed.SetupPacket.wValue.W)
//         DPRINT("wValue            - %04X\n", TD->HwTD.c2rust_unnamed.SetupPacket.wValue.W);
//     if (TD->HwTD.c2rust_unnamed.SetupPacket.wIndex.W)
//         DPRINT("wIndex            - %04X\n", TD->HwTD.c2rust_unnamed.SetupPacket.wIndex.W);
//     if (TD->HwTD.c2rust_unnamed.SetupPacket.wLength)
//         DPRINT("wLength           - %04X\n", TD->HwTD.c2rust_unnamed.SetupPacket.wLength);
//
//     DPRINT("PhysicalAddress   - %p\n", TD->PhysicalAddress);
//     DPRINT("Flags             - %X\n", TD->Flags);
//     DPRINT("OhciTransfer      - %08X\n", TD->OhciTransfer);
//     DPRINT("NextTDVa          - %08X\n", TD->NextTDVa);
//     if (TD->TransferLen)
//         DPRINT("TransferLen       - %X\n", TD->TransferLen);
// }
//
// VOID
// NTAPI
// OHCI_EnableList(IN POHCI_EXTENSION OhciExtension,
//                 IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG CommandStatusReg;
//     ULONG TransferType;
//     OHCI_REG_COMMAND_STATUS CommandStatus;
//
//     DPRINT_OHCI!("OHCI_EnableList: ... \n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     CommandStatusReg = (PULONG)&OperationalRegs->HcCommandStatus;
//
//     CommandStatus.AsULONG = 0;
//
//     if (READ_REGISTER_ULONG((PULONG)&OperationalRegs->HcControlHeadED))
//         CommandStatus.ControlListFilled = 1;
//
//     if (READ_REGISTER_ULONG((PULONG)&OperationalRegs->HcBulkHeadED))
//         CommandStatus.BulkListFilled = 1;
//
//     TransferType = OhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK)
//         CommandStatus.BulkListFilled = 1;
//     else if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//         CommandStatus.ControlListFilled = 1;
//
//     WRITE_REGISTER_ULONG(CommandStatusReg, CommandStatus.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_InsertEndpointInSchedule(IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_STATIC_ED HeadED;
//     POHCI_HCD_ED ED;
//     POHCI_HCD_ED PrevED;
//     PLIST_ENTRY HeadLink;
//
//     DPRINT_OHCI!("OHCI_InsertEndpointInSchedule: OhciEndpoint - %p\n",
//                 OhciEndpoint);
//
//     ED = OhciEndpoint->HcdED;
//
//     HeadED = OhciEndpoint->HeadED;
//     HeadLink = &HeadED->Link;
//
//     if (IsListEmpty(HeadLink))
//     {
//         InsertHeadList(HeadLink, &ED->HcdEDLink);
//
//         if (HeadED->Type == OHCI_STATIC_ED_TYPE_CONTROL ||
//             HeadED->Type == OHCI_STATIC_ED_TYPE_BULK)
//         {
//             ED->HwED.NextED = READ_REGISTER_ULONG(HeadED->pNextED);
//             WRITE_REGISTER_ULONG(HeadED->pNextED, ED->PhysicalAddress);
//         }
//         else if (HeadED->Type == OHCI_STATIC_ED_TYPE_INTERRUPT)
//         {
//             ED->HwED.NextED = *HeadED->pNextED;
//             *HeadED->pNextED = ED->PhysicalAddress;
//         }
//         else
//         {
//             DPRINT1("OHCI_InsertEndpointInSchedule: Unknown HeadED->Type - %x\n",
//                     HeadED->Type);
//             DbgBreakPoint();
//         }
//     }
//     else
//     {
//         PrevED = CONTAINING_RECORD(HeadLink->Blink,
//                                    OHCI_HCD_ED,
//                                    HcdEDLink);
//
//         InsertTailList(HeadLink, &ED->HcdEDLink);
//
//         ED->HwED.NextED = 0;
//         PrevED->HwED.NextED = ED->PhysicalAddress;
//     }
// }
//
// POHCI_HCD_ED
// NTAPI
// OHCI_InitializeED(IN POHCI_ENDPOINT OhciEndpoint,
//                   IN POHCI_HCD_ED ED,
//                   IN POHCI_HCD_TD FirstTD,
//                   IN ULONG_PTR EdPA)
// {
//     OHCI_ENDPOINT_CONTROL EndpointControl;
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//
//     DPRINT_OHCI!("OHCI_InitializeED: OhciEndpoint - %p, ED - %p, FirstTD - %p, EdPA - %p\n",
//                 OhciEndpoint,
//                 ED,
//                 FirstTD,
//                 EdPA);
//
//     RtlZeroMemory(ED, sizeof(OHCI_HCD_ED));
//
//     ED->PhysicalAddress = EdPA;
//
//     EndpointProperties = &OhciEndpoint->EndpointProperties;
//
//     EndpointControl = ED->HwED.EndpointControl;
//
//     EndpointControl.FunctionAddress = EndpointProperties->DeviceAddress;
//     EndpointControl.EndpointNumber = EndpointProperties->EndpointAddress;
//     EndpointControl.MaximumPacketSize = EndpointProperties->TotalMaxPacketSize;
//
//     if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         EndpointControl.Direction = OHCI_ED_DATA_FLOW_DIRECTION_FROM_TD;
//     }
//     else if (EndpointProperties->Direction)
//     {
//         EndpointControl.Direction = OHCI_ED_DATA_FLOW_DIRECTION_OUT;
//     }
//     else
//     {
//         EndpointControl.Direction = OHCI_ED_DATA_FLOW_DIRECTION_IN;
//     }
//
//     if (EndpointProperties->DeviceSpeed == UsbLowSpeed)
//         EndpointControl.Speed = OHCI_ENDPOINT_LOW_SPEED;
//
//     if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         EndpointControl.Format = OHCI_ENDPOINT_ISOCHRONOUS_FORMAT;
//     else
//         EndpointControl.sKip = 1;
//
//     ED->HwED.EndpointControl = EndpointControl;
//
//     ED->HwED.TailPointer = FirstTD->PhysicalAddress;
//     ED->HwED.HeadPointer = FirstTD->PhysicalAddress;
//
//     FirstTD->Flags |= OHCI_HCD_TD_FLAG_ALLOCATED;
//
//     OhciEndpoint->HcdTailP = FirstTD;
//     OhciEndpoint->HcdHeadP = FirstTD;
//
//     return ED;
// }
//
// VOID
// NTAPI
// OHCI_InitializeTDs(IN POHCI_ENDPOINT OhciEndpoint,
//                    IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties)
// {
//     POHCI_HCD_TD TdVA;
//     ULONG TdPA;
//     ULONG TdCount;
//     ULONG ix;
//
//     ASSERT(EndpointProperties->BufferLength > sizeof(OHCI_HCD_ED));
//
//     TdCount = (EndpointProperties->BufferLength - sizeof(OHCI_HCD_ED)) /
//               sizeof(OHCI_HCD_TD);
//
//     OhciEndpoint->MaxTransferDescriptors = TdCount;
//
//     DPRINT_OHCI!("OHCI_InitializeTDs: TdCount - %x\n", TdCount);
//
//     ASSERT(TdCount > 0);
//
//     TdVA = OhciEndpoint->FirstTD;
//
//     TdPA = EndpointProperties->BufferPA + sizeof(OHCI_HCD_ED);
//
//     for (ix = 0; ix < TdCount; ix++)
//     {
//         DPRINT_OHCI!("OHCI_InitializeTDs: TdVA - %p, TdPA - %08X\n", TdVA, TdPA);
//
//         RtlZeroMemory(TdVA, sizeof(OHCI_HCD_TD));
//
//         TdVA->PhysicalAddress = TdPA;
//         TdVA->Flags = 0;
//         TdVA->OhciTransfer = 0;
//
//         TdVA++;
//         TdPA += sizeof(OHCI_HCD_TD);
//     }
// }
//
// MPSTATUS
// NTAPI
// OHCI_OpenControlEndpoint(IN POHCI_EXTENSION OhciExtension,
//                          IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                          IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_OpenControlEndpoint: ... \n");
//
//     ED = (POHCI_HCD_ED)EndpointProperties->BufferVA;
//
//     OhciEndpoint->FirstTD = (POHCI_HCD_TD)((ULONG_PTR)ED + sizeof(OHCI_HCD_ED));
//     OhciEndpoint->HeadED = &OhciExtension->ControlStaticED;
//
//     OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);
//
//     OhciEndpoint->HcdED = OHCI_InitializeED(OhciEndpoint,
//                                             ED,
//                                             OhciEndpoint->FirstTD,
//                                             EndpointProperties->BufferPA);
//
//     OhciEndpoint->HcdED->Flags = OHCI_HCD_ED_FLAG_CONTROL |
//                                  OHCI_HCD_ED_FLAG_RESET_ON_HALT;
//
//     OHCI_InsertEndpointInSchedule(OhciEndpoint);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_OpenBulkEndpoint(IN POHCI_EXTENSION OhciExtension,
//                       IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                       IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_OpenBulkEndpoint: ... \n");
//
//     ED = (POHCI_HCD_ED)EndpointProperties->BufferVA;
//
//     OhciEndpoint->FirstTD = (POHCI_HCD_TD)((ULONG_PTR)ED + sizeof(OHCI_HCD_ED));
//     OhciEndpoint->HeadED = &OhciExtension->BulkStaticED;
//
//     OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);
//
//     OhciEndpoint->HcdED = OHCI_InitializeED(OhciEndpoint,
//                                             ED,
//                                             OhciEndpoint->FirstTD,
//                                             EndpointProperties->BufferPA);
//
//     OHCI_InsertEndpointInSchedule(OhciEndpoint);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_OpenInterruptEndpoint(IN POHCI_EXTENSION OhciExtension,
//                            IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                            IN POHCI_ENDPOINT OhciEndpoint)
// {
//     UCHAR Period;
//     ULONG PeriodIdx = 0;
//     POHCI_HCD_ED ED;
//     ULONG ScheduleOffset;
//
//     DPRINT_OHCI!("OHCI_OpenInterruptEndpoint: ... \n");
//
//     ED = (POHCI_HCD_ED)EndpointProperties->BufferVA;
//
//     OhciEndpoint->FirstTD = (POHCI_HCD_TD)((ULONG_PTR)ED + sizeof(OHCI_HCD_ED));
//
//     Period = EndpointProperties->Period;
//
//     ASSERT(Period != 0);
//
//     while (!(Period & 1))
//     {
//         PeriodIdx++;
//         Period >>= 1;
//     }
//
//     ASSERT(PeriodIdx < ARRAYSIZE(Index));
//
//     ScheduleOffset = EndpointProperties->ScheduleOffset;
//     DPRINT_OHCI!("OHCI_OpenInterruptEndpoint: InitTD. Index[PeriodIdx] - %x, ScheduleOffset - %x\n",
//                 Index[PeriodIdx],
//                 ScheduleOffset);
//
//     OhciEndpoint->HeadED = &OhciExtension->IntStaticED[Index[PeriodIdx] +
//                                                        ScheduleOffset];
//
//     //OhciEndpoint->HeadED->UsbBandwidth += EndpointProperties->UsbBandwidth;
//
//     OHCI_InitializeTDs(OhciEndpoint, EndpointProperties);
//
//     OhciEndpoint->HcdED = OHCI_InitializeED(OhciEndpoint,
//                                             ED,
//                                             OhciEndpoint->FirstTD,
//                                             EndpointProperties->BufferPA);
//
//     OHCI_InsertEndpointInSchedule(OhciEndpoint);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_OpenIsoEndpoint(IN POHCI_EXTENSION OhciExtension,
//                      IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                      IN POHCI_ENDPOINT OhciEndpoint)
// {
//     DPRINT1("OHCI_OpenIsoEndpoint: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_NOT_SUPPORTED;
// }
//
// MPSTATUS
// NTAPI
// OHCI_OpenEndpoint(IN PVOID ohciExtension,
//                   IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                   IN PVOID ohciEndpoint)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     ULONG TransferType;
//     MPSTATUS MPStatus;
//
//     DPRINT_OHCI!("OHCI_OpenEndpoint: ... \n");
//
//     RtlCopyMemory(&OhciEndpoint->EndpointProperties,
//                   EndpointProperties,
//                   sizeof(OhciEndpoint->EndpointProperties));
//
//     InitializeListHead(&OhciEndpoint->TDList);
//
//     TransferType = EndpointProperties->TransferType;
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             MPStatus = OHCI_OpenIsoEndpoint(OhciExtension,
//                                             EndpointProperties,
//                                             OhciEndpoint);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             MPStatus = OHCI_OpenControlEndpoint(OhciExtension,
//                                                 EndpointProperties,
//                                                 OhciEndpoint);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             MPStatus = OHCI_OpenBulkEndpoint(OhciExtension,
//                                              EndpointProperties,
//                                              OhciEndpoint);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             MPStatus = OHCI_OpenInterruptEndpoint(OhciExtension,
//                                                   EndpointProperties,
//                                                   OhciEndpoint);
//             break;
//
//         default:
//             MPStatus = MP_STATUS_NOT_SUPPORTED;
//             break;
//     }
//
//     return MPStatus;
// }
//
// MPSTATUS
// NTAPI
// OHCI_ReopenEndpoint(IN PVOID ohciExtension,
//                     IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                     IN PVOID ohciEndpoint)
// {
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_ReopenEndpoint: ... \n");
//
//     ED = OhciEndpoint->HcdED;
//
//     RtlCopyMemory(&OhciEndpoint->EndpointProperties,
//                   EndpointProperties,
//                   sizeof(OhciEndpoint->EndpointProperties));
//
//     ED->HwED.EndpointControl.FunctionAddress =
//         OhciEndpoint->EndpointProperties.DeviceAddress;
//
//     ED->HwED.EndpointControl.MaximumPacketSize =
//         OhciEndpoint->EndpointProperties.TotalMaxPacketSize;
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// OHCI_QueryEndpointRequirements(IN PVOID ohciExtension,
//                                IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                                IN PUSBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements)
// {
//     ULONG TransferType;
//
//     DPRINT_OHCI!("OHCI_QueryEndpointRequirements: ... \n");
//
//     TransferType = EndpointProperties->TransferType;
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             DPRINT_OHCI!("OHCI_QueryEndpointRequirements: IsoTransfer\n");
//             EndpointRequirements->MaxTransferSize = OHCI_MAX_ISO_TRANSFER_SIZE;
//             EndpointRequirements->HeaderBufferSize =
//             sizeof(OHCI_HCD_ED) + OHCI_MAX_ISO_TD_COUNT * sizeof(OHCI_HCD_TD);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             DPRINT_OHCI!("OHCI_QueryEndpointRequirements: ControlTransfer\n");
//             EndpointRequirements->MaxTransferSize = OHCI_MAX_CONTROL_TRANSFER_SIZE;
//             EndpointRequirements->HeaderBufferSize =
//             sizeof(OHCI_HCD_ED) + OHCI_MAX_CONTROL_TD_COUNT * sizeof(OHCI_HCD_TD);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             DPRINT_OHCI!("OHCI_QueryEndpointRequirements: BulkTransfer\n");
//             EndpointRequirements->MaxTransferSize = OHCI_MAX_BULK_TRANSFER_SIZE;
//             EndpointRequirements->HeaderBufferSize =
//             sizeof(OHCI_HCD_ED) + OHCI_MAX_BULK_TD_COUNT * sizeof(OHCI_HCD_TD);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             DPRINT_OHCI!("OHCI_QueryEndpointRequirements: InterruptTransfer\n");
//             EndpointRequirements->MaxTransferSize = OHCI_MAX_INTERRUPT_TRANSFER_SIZE;
//             EndpointRequirements->HeaderBufferSize =
//             sizeof(OHCI_HCD_ED) + OHCI_MAX_INTERRUPT_TD_COUNT * sizeof(OHCI_HCD_TD);
//             break;
//
//         default:
//             DPRINT1("OHCI_QueryEndpointRequirements: Unknown TransferType - %x\n",
//                     TransferType);
//             DbgBreakPoint();
//             break;
//     }
// }
//
// VOID
// NTAPI
// OHCI_CloseEndpoint(IN PVOID ohciExtension,
//                    IN PVOID ohciEndpoint,
//                    IN BOOLEAN IsDoDisablePeriodic)
// {
//     DPRINT1("OHCI_CloseEndpoint: Not supported\n");
//     return;
// }
//
// MPSTATUS
// NTAPI
// OHCI_TakeControlHC(IN POHCI_EXTENSION OhciExtension,
//                    IN PUSBPORT_RESOURCES Resources)
// {
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG ControlReg;
//     PULONG InterruptEnableReg;
//     PULONG InterruptDisableReg;
//     PULONG CommandStatusReg;
//     PULONG InterruptStatusReg;
//     OHCI_REG_CONTROL Control;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntEnable;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntDisable;
//     OHCI_REG_COMMAND_STATUS CommandStatus;
//     OHCI_REG_INTERRUPT_STATUS IntStatus;
//     LARGE_INTEGER StartTicks, CurrentTicks;
//     UINT32 TicksDiff;
//
//     DPRINT("OHCI_TakeControlHC: ...\n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//     InterruptDisableReg = (PULONG)&OperationalRegs->HcInterruptDisable;
//     CommandStatusReg = (PULONG)&OperationalRegs->HcCommandStatus;
//     InterruptStatusReg = (PULONG)&OperationalRegs->HcInterruptStatus;
//
//     /* 5.1.1.3 Take Control of Host Controller */
//     Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//
//     if (Control.InterruptRouting == 0)
//         return MP_STATUS_SUCCESS;
//
//     DPRINT1("OHCI_TakeControlHC: detected Legacy BIOS\n");
//
//     IntEnable.AsULONG = READ_REGISTER_ULONG(InterruptEnableReg);
//
//     DPRINT("OHCI_TakeControlHC: Control - %lX, IntEnable - %lX\n",
//            Control.AsULONG,
//            IntEnable.AsULONG);
//
//     if (Control.HostControllerFunctionalState == OHCI_HC_STATE_RESET &&
//         IntEnable.AsULONG == 0)
//     {
//         Control.AsULONG = 0;
//         WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//         return MP_STATUS_SUCCESS;
//     }
//
//     /* Enable interrupt generations */
//     IntEnable.AsULONG = 0;
//     IntEnable.MasterInterruptEnable = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, IntEnable.AsULONG);
//
//     /* Request a change of control of the HC */
//     CommandStatus.AsULONG = 0;
//     CommandStatus.OwnershipChangeRequest = 1;
//
//     WRITE_REGISTER_ULONG(CommandStatusReg, CommandStatus.AsULONG);
//
//     /* Disable interrupt generation due to Root Hub Status Change */
//     IntDisable.AsULONG = 0;
//     IntDisable.RootHubStatusChange = 1;
//
//     WRITE_REGISTER_ULONG(InterruptDisableReg, IntDisable.AsULONG);
//
//     /* Monitoring the InterruptRouting bit
//        to determine when the ownership change has taken effect. */
//
//     TicksDiff = (500 * 10000) / KeQueryTimeIncrement(); // 500 ms
//     KeQueryTickCount(&StartTicks);
//
//     do
//     {
//         Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//
//         if (Control.InterruptRouting == 0)
//         {
//             /* Clear all bits in register */
//             IntStatus.AsULONG = 0xFFFFFFFF;
//             WRITE_REGISTER_ULONG(InterruptStatusReg, IntStatus.AsULONG);
//
//             /* Disable interrupt generations */
//             IntDisable.AsULONG = 0;
//             IntDisable.MasterInterruptEnable = 1;
//
//             WRITE_REGISTER_ULONG(InterruptDisableReg, IntDisable.AsULONG);
//
//             return MP_STATUS_SUCCESS;
//         }
//
//         KeQueryTickCount(&CurrentTicks);
//     }
//     while (CurrentTicks.QuadPart - StartTicks.QuadPart < TicksDiff);
//
//     return MP_STATUS_HW_ERROR;
// }
//
// MPSTATUS
// NTAPI
// OHCI_StartController(IN PVOID ohciExtension,
//                      IN PUSBPORT_RESOURCES Resources)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG CommandStatusReg;
//     PULONG FmIntervalReg;
//     PULONG ControlReg;
//     PULONG InterruptEnableReg;
//     PULONG RhStatusReg;
//     OHCI_REG_COMMAND_STATUS CommandStatus;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE Interrupts;
//     OHCI_REG_RH_STATUS RhStatus;
//     OHCI_REG_FRAME_INTERVAL FrameInterval;
//     ULONG MaxFrameIntervalAdjusting;
//     OHCI_REG_CONTROL Control;
//     UCHAR HeadIndex;
//     POHCI_ENDPOINT_DESCRIPTOR IntED;
//     ULONG_PTR IntEdPA;
//     POHCI_HCCA OhciHCCA;
//     LARGE_INTEGER StartTicks, CurrentTicks;
//     UINT32 TicksDiff;
//     ULONG ix;
//     ULONG jx;
//     MPSTATUS MPStatus = MP_STATUS_SUCCESS;
//
//     DPRINT_OHCI!("OHCI_StartController: ohciExtension - %p, Resources - %p\n",
//                 ohciExtension,
//                 Resources);
//
//     /* HC on-chip operational registers */
//     OperationalRegs = (POHCI_OPERATIONAL_REGISTERS)Resources->ResourceBase;
//     OhciExtension->OperationalRegs = OperationalRegs;
//
//     CommandStatusReg = (PULONG)&OperationalRegs->HcCommandStatus;
//     FmIntervalReg = (PULONG)&OperationalRegs->HcFmInterval;
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//     RhStatusReg = (PULONG)&OperationalRegs->HcRhStatus;
//
//     /* 5.1.1 Initialization */
//
//     MPStatus = OHCI_TakeControlHC(OhciExtension, Resources);
//
//     if (MPStatus != MP_STATUS_SUCCESS)
//     {
//         DPRINT1("OHCI_StartController: OHCI_TakeControlHC return MPStatus - %x\n",
//                 MPStatus);
//
//         return MPStatus;
//     }
//
//     OhciExtension->HcResourcesVA = (POHCI_HC_RESOURCES)Resources->StartVA;
//     OhciExtension->HcResourcesPA = Resources->StartPA;
//
//     DPRINT_OHCI!("OHCI_StartController: HcResourcesVA - %p, HcResourcesPA - %lx\n",
//                 OhciExtension->HcResourcesVA,
//                 OhciExtension->HcResourcesPA);
//
//     /* 5.2.7.2 Interrupt */
//
//     /* Build structure of interrupt static EDs */
//     for (ix = 0; ix < INTERRUPT_ENDPOINTs; ix++)
//     {
//         IntED = &OhciExtension->HcResourcesVA->InterrruptHeadED[ix];
//         IntEdPA = OhciExtension->HcResourcesPA + core::mem::offset_of!(OHCI_HC_RESOURCES, InterrruptHeadED[ix]);
//
//         if (ix == (ENDPOINT_INTERRUPT_1ms - 1))
//         {
//             HeadIndex = ED_EOF;
//             IntED->NextED = 0;
//         }
//         else
//         {
//             HeadIndex = ((ix - 1) / 2);
//
//             ASSERT(HeadIndex >= (ENDPOINT_INTERRUPT_1ms - 1) &&
//                    HeadIndex < (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms));
//
//             IntED->NextED = OhciExtension->IntStaticED[HeadIndex].PhysicalAddress;
//         }
//
//         IntED->EndpointControl.sKip = 1;
//
//         IntED->TailPointer = 0;
//         IntED->HeadPointer = 0;
//
//         OhciExtension->IntStaticED[ix].HwED = IntED;
//         OhciExtension->IntStaticED[ix].PhysicalAddress = IntEdPA;
//         OhciExtension->IntStaticED[ix].HeadIndex = HeadIndex;
//         OhciExtension->IntStaticED[ix].pNextED = &IntED->NextED;
//         OhciExtension->IntStaticED[ix].Type = OHCI_STATIC_ED_TYPE_INTERRUPT;
//
//         InitializeListHead(&OhciExtension->IntStaticED[ix].Link);
//     }
//
//     OhciHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//     DPRINT_OHCI!("OHCI_InitializeSchedule: OhciHCCA - %p\n", OhciHCCA);
//
//     /* Set head pointers which start from HCCA */
//     for (ix = 0, jx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms);
//          ix < OHCI_NUMBER_OF_INTERRUPTS;
//          ix++, jx++)
//     {
//         OhciHCCA->InterrruptTable[Balance[ix]] =
//             OhciExtension->IntStaticED[jx].PhysicalAddress;
//
//         OhciExtension->IntStaticED[jx].pNextED =
//             (PULONG)&OhciHCCA->InterrruptTable[Balance[ix]];
//
//         OhciExtension->IntStaticED[jx].HccaIndex = Balance[ix];
//     }
//
//     DPRINT_OHCI!("OHCI_InitializeSchedule: ix - %x\n", ix);
//
//     /* Init static Control and Bulk EDs head pointers which start from HCCA */
//     InitializeListHead(&OhciExtension->ControlStaticED.Link);
//
//     OhciExtension->ControlStaticED.HeadIndex = ED_EOF;
//     OhciExtension->ControlStaticED.Type = OHCI_STATIC_ED_TYPE_CONTROL;
//     OhciExtension->ControlStaticED.pNextED = &OperationalRegs->HcControlHeadED;
//
//     InitializeListHead(&OhciExtension->BulkStaticED.Link);
//
//     OhciExtension->BulkStaticED.HeadIndex = ED_EOF;
//     OhciExtension->BulkStaticED.Type = OHCI_STATIC_ED_TYPE_BULK;
//     OhciExtension->BulkStaticED.pNextED = &OperationalRegs->HcBulkHeadED;
//
//     /* 6.3.1 Frame Timing */
//     FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg);
//
//     MaxFrameIntervalAdjusting = OHCI_DEFAULT_FRAME_INTERVAL / 10; // 10%
//
//     if ((FrameInterval.FrameInterval) < (OHCI_DEFAULT_FRAME_INTERVAL - MaxFrameIntervalAdjusting) ||
//         (FrameInterval.FrameInterval) > (OHCI_DEFAULT_FRAME_INTERVAL + MaxFrameIntervalAdjusting))
//     {
//         FrameInterval.FrameInterval = OHCI_DEFAULT_FRAME_INTERVAL;
//     }
//
//     /* 5.4 FrameInterval Counter */
//     FrameInterval.FrameIntervalToggle = 1;
//
//     /* OHCI_MAXIMUM_OVERHEAD is the maximum overhead per frame */
//     FrameInterval.FSLargestDataPacket =
//         ((FrameInterval.FrameInterval - OHCI_MAXIMUM_OVERHEAD) * 6) / 7;
//
//     OhciExtension->FrameInterval = FrameInterval;
//
//     DPRINT_OHCI!("OHCI_StartController: FrameInterval - %lX\n",
//                 FrameInterval.AsULONG);
//
//     /* Reset HostController */
//     CommandStatus.AsULONG = 0;
//     CommandStatus.HostControllerReset = 1;
//
//     WRITE_REGISTER_ULONG(CommandStatusReg, CommandStatus.AsULONG);
//
//     KeStallExecutionProcessor(25);
//
//     Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//     Control.HostControllerFunctionalState = OHCI_HC_STATE_RESET;
//
//     WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//
//     TicksDiff = (500 * 10000) / KeQueryTimeIncrement(); // 500 ms
//     KeQueryTickCount(&StartTicks);
//
//     while (TRUE)
//     {
//         WRITE_REGISTER_ULONG(FmIntervalReg, OhciExtension->FrameInterval.AsULONG);
//         FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg);
//
//         KeQueryTickCount(&CurrentTicks);
//
//         if (CurrentTicks.QuadPart - StartTicks.QuadPart >= TicksDiff)
//         {
//             MPStatus = MP_STATUS_HW_ERROR;
//             break;
//         }
//
//         if (FrameInterval.AsULONG == OhciExtension->FrameInterval.AsULONG)
//         {
//             MPStatus = MP_STATUS_SUCCESS;
//             break;
//         }
//     }
//
//     if (MPStatus != MP_STATUS_SUCCESS)
//     {
//         DPRINT_OHCI!("OHCI_StartController: frame interval not set\n");
//         return MPStatus;
//     }
//
//     /* Setup HcPeriodicStart register */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcPeriodicStart,
//                         (OhciExtension->FrameInterval.FrameInterval * 9) / 10); //90%
//
//     /* Setup HcHCCA register */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcHCCA,
//                          OhciExtension->HcResourcesPA + core::mem::offset_of!(OHCI_HC_RESOURCES, HcHCCA));
//
//     /* Setup HcInterruptEnable register */
//     Interrupts.AsULONG = 0;
//
//     Interrupts.SchedulingOverrun = 1;
//     Interrupts.WritebackDoneHead = 1;
//     Interrupts.UnrecoverableError = 1;
//     Interrupts.FrameNumberOverflow = 1;
//     Interrupts.OwnershipChange = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, Interrupts.AsULONG);
//
//     /* Setup HcControl register */
//     Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//
//     Control.ControlBulkServiceRatio = 0; // FIXME (1 : 1)
//     Control.PeriodicListEnable = 1;
//     Control.IsochronousEnable = 1;
//     Control.ControlListEnable = 1;
//     Control.BulkListEnable = 1;
//     Control.HostControllerFunctionalState = OHCI_HC_STATE_OPERATIONAL;
//
//     WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//
//     /* Setup HcRhStatus register */
//     RhStatus.AsULONG = 0;
//     RhStatus.SetGlobalPower = 1;
//
//     WRITE_REGISTER_ULONG(RhStatusReg, RhStatus.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// OHCI_StopController(IN PVOID ohciExtension,
//                     IN BOOLEAN IsDoDisableInterrupts)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG ControlReg;
//     PULONG InterruptDisableReg;
//     PULONG InterruptStatusReg;
//     OHCI_REG_CONTROL Control;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntDisable;
//     OHCI_REG_INTERRUPT_STATUS IntStatus;
//
//     DPRINT("OHCI_StopController: ... \n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     InterruptDisableReg = (PULONG)&OperationalRegs->HcInterruptDisable;
//     InterruptStatusReg = (PULONG)&OperationalRegs->HcInterruptStatus;
//
//     /* Setup HcControl register */
//     Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//
//     Control.PeriodicListEnable = 0;
//     Control.IsochronousEnable = 0;
//     Control.ControlListEnable = 0;
//     Control.BulkListEnable = 0;
//     Control.HostControllerFunctionalState = OHCI_HC_STATE_SUSPEND;
//     Control.RemoteWakeupEnable = 0;
//
//     WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//
//     /* Disable interrupt generations */
//     IntDisable.AsULONG = 0xFFFFFFFF;
//     WRITE_REGISTER_ULONG(InterruptDisableReg, IntDisable.AsULONG);
//
//     /* Clear all bits in HcInterruptStatus register */
//     IntStatus.AsULONG = 0xFFFFFFFF;
//     WRITE_REGISTER_ULONG(InterruptStatusReg, IntStatus.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_SuspendController(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG ControlReg;
//     PULONG InterruptEnableReg;
//     OHCI_REG_CONTROL Control;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE InterruptReg;
//
//     DPRINT("OHCI_SuspendController: ... \n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//
//     /* Disable all interrupt generations */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptDisable.AsULONG,
//                          0xFFFFFFFF);
//
//     /* Clear all bits in HcInterruptStatus register */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptStatus.AsULONG,
//                          0xFFFFFFFF);
//
//     /* Setup HcControl register */
//     Control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//     Control.HostControllerFunctionalState = OHCI_HC_STATE_SUSPEND;
//     Control.RemoteWakeupEnable =  1;
//
//     WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//
//     /* Setup HcInterruptEnable register */
//     InterruptReg.AsULONG = 0;
//     InterruptReg.ResumeDetected = 1;
//     InterruptReg.UnrecoverableError = 1;
//     InterruptReg.RootHubStatusChange = 1;
//     InterruptReg.MasterInterruptEnable = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, InterruptReg.AsULONG);
// }
//
// MPSTATUS
// NTAPI
// OHCI_ResumeController(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG ControlReg;
//     PULONG InterruptEnableReg;
//     POHCI_HCCA HcHCCA;
//     OHCI_REG_CONTROL control;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE InterruptReg;
//
//     DPRINT("OHCI_ResumeController \n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//
//     control.AsULONG = READ_REGISTER_ULONG(ControlReg);
//
//     if (control.HostControllerFunctionalState != OHCI_HC_STATE_SUSPEND)
//         return MP_STATUS_HW_ERROR;
//
//     HcHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//     HcHCCA->Pad1 = 0;
//
//     /* Setup HcControl register */
//     control.HostControllerFunctionalState = OHCI_HC_STATE_OPERATIONAL;
//     WRITE_REGISTER_ULONG(ControlReg, control.AsULONG);
//
//     /* Setup HcInterruptEnable register */
//     InterruptReg.AsULONG = 0;
//     InterruptReg.SchedulingOverrun = 1;
//     InterruptReg.WritebackDoneHead = 1;
//     InterruptReg.UnrecoverableError = 1;
//     InterruptReg.FrameNumberOverflow = 1;
//     InterruptReg.OwnershipChange = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, InterruptReg.AsULONG);
//     WRITE_REGISTER_ULONG(ControlReg, control.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// OHCI_HardwarePresent(IN POHCI_EXTENSION OhciExtension,
//                      IN BOOLEAN IsInvalidateController)
// {
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG CommandStatusReg;
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     CommandStatusReg = (PULONG)&OperationalRegs->HcCommandStatus;
//
//     if (READ_REGISTER_ULONG(CommandStatusReg) != 0xFFFFFFFF)
//         return TRUE;
//
//     DPRINT1("OHCI_HardwarePresent: IsInvalidateController - %x\n",
//             IsInvalidateController);
//
//     if (IsInvalidateController)
//     {
//         RegPacket.UsbPortInvalidateController(OhciExtension,
//                                               USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE);
//     }
//
//     return FALSE;
// }
//
// BOOLEAN
// NTAPI
// OHCI_InterruptService(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     OHCI_REG_INTERRUPT_STATUS IntStatus;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntEnable;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntDisable;
//     BOOLEAN HardwarePresent = FALSE;
//
//     DPRINT_OHCI!("OHCI_Interrupt: Ext %p\n", OhciExtension);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     HardwarePresent = OHCI_HardwarePresent(OhciExtension, FALSE);
//
//     if (!HardwarePresent)
//         return FALSE;
//
//     IntEnable.AsULONG = READ_REGISTER_ULONG((PULONG)&OperationalRegs->HcInterruptEnable);
//     IntStatus.AsULONG = READ_REGISTER_ULONG((PULONG)&OperationalRegs->HcInterruptStatus) & IntEnable.AsULONG;
//
//     if ((IntStatus.AsULONG == 0) || (IntEnable.MasterInterruptEnable == 0))
//         return FALSE;
//
//     if (IntStatus.UnrecoverableError)
//         DPRINT1("OHCI_InterruptService: IntStatus.UnrecoverableError\n");
//
//     if (IntStatus.FrameNumberOverflow)
//     {
//         POHCI_HCCA HcHCCA;
//         ULONG fm;
//         ULONG hp;
//
//         HcHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//
//         DPRINT("FrameNumberOverflow %lX\n", HcHCCA->FrameNumber);
//
//         hp = OhciExtension->FrameHighPart;
//         fm = HcHCCA->FrameNumber;
//
//         /* Increment value of FrameHighPart */
//         OhciExtension->FrameHighPart += 1 * (1 << 16) - ((hp ^ fm) & 0x8000);
//     }
//
//     /* Disable interrupt generation */
//     IntDisable.AsULONG = 0;
//     IntDisable.MasterInterruptEnable = 1;
//     WRITE_REGISTER_ULONG((PULONG)&OperationalRegs->HcInterruptDisable,
//                          IntDisable.AsULONG);
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// OHCI_InterruptDpc(IN PVOID ohciExtension,
//                   IN BOOLEAN IsDoEnableInterrupts)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptDisableReg;
//     PULONG InterruptEnableReg;
//     PULONG InterruptStatusReg;
//     OHCI_REG_INTERRUPT_STATUS IntStatus;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE InterruptBits;
//     POHCI_HCCA HcHCCA;
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//     InterruptDisableReg = (PULONG)&OperationalRegs->HcInterruptDisable;
//     InterruptStatusReg = (PULONG)&OperationalRegs->HcInterruptStatus;
//
//     DPRINT_OHCI!("OHCI_InterruptDpc: OhciExtension - %p, IsDoEnableInterrupts - %x\n",
//                 OhciExtension,
//                 IsDoEnableInterrupts);
//
//     IntStatus.AsULONG = READ_REGISTER_ULONG(InterruptStatusReg);
//
//     if (IntStatus.RootHubStatusChange)
//     {
//         DPRINT_OHCI!("OHCI_InterruptDpc: RootHubStatusChange\n");
//         RegPacket.UsbPortInvalidateRootHub(OhciExtension);
//     }
//
//     if (IntStatus.WritebackDoneHead)
//     {
//         DPRINT_OHCI!("OHCI_InterruptDpc: WritebackDoneHead\n");
//
//         HcHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//         HcHCCA->DoneHead = 0;
//
//         RegPacket.UsbPortInvalidateEndpoint(OhciExtension, NULL);
//     }
//
//     if (IntStatus.StartofFrame)
//     {
//         /* Disable interrupt generation due to Start of Frame */
//         InterruptBits.AsULONG = 0;
//         InterruptBits.StartofFrame = 1;
//
//         WRITE_REGISTER_ULONG(InterruptDisableReg, InterruptBits.AsULONG);
//     }
//
//     if (IntStatus.ResumeDetected)
//         DPRINT1("OHCI_IntDpc: ResumeDetected\n");
//
//     if (IntStatus.UnrecoverableError)
//     {
//         DPRINT1("OHCI_IntDpc: UnrecoverableError\n");
//     }
//
//     WRITE_REGISTER_ULONG(InterruptStatusReg, IntStatus.AsULONG);
//
//     if (IsDoEnableInterrupts)
//     {
//         /*  Enable interrupt generation */
//         InterruptBits.AsULONG = 0;
//         InterruptBits.MasterInterruptEnable = 1;
//
//         WRITE_REGISTER_ULONG(InterruptEnableReg, InterruptBits.AsULONG);
//     }
// }
//
// /**
//  * @brief      Forms the next General Transfer Descriptor for the current transfer
//  *
//  * @param[in]  OhciExtension  The ohci extension
//  * @param[in]  TransferedLen  The consolidated length of all previous descriptors' buffers
//  * @param[in]  OhciTransfer   The ohci transfer
//  * @param[out] TD             The transfer descriptor we are forming
//  * @param[in]  SGList         The scatter/gather list
//  *
//  * @return     The length of all previous buffers summed with the length of the current buffer
//  */
// static
// ULONG
// OHCI_MapTransferToTD(IN POHCI_EXTENSION OhciExtension,
//                      IN ULONG TransferedLen,
//                      IN POHCI_TRANSFER OhciTransfer,
//                      OUT POHCI_HCD_TD TD,
//                      IN PUSBPORT_SCATTER_GATHER_LIST SGList)
// {
//     PUSBPORT_SCATTER_GATHER_ELEMENT SgElement;
//     ULONG SgIdx, CurrentTransferLen, BufferEnd, CurrentBuffer;
//     ULONG TransferDataLeft = OhciTransfer->TransferParameters->TransferBufferLength - TransferedLen;
//
//     DPRINT_OHCI!("OHCI_MapTransferToTD: TransferedLen - %x\n", TransferedLen);
//
//     for (SgIdx = 0; SgIdx < SGList->SgElementCount; SgIdx++)
//     {
//         SgElement = &SGList->SgElement[SgIdx];
//
//         if (TransferedLen >= SgElement->SgOffset &&
//             TransferedLen < SgElement->SgOffset + SgElement->SgTransferLength)
//         {
//             break;
//         }
//     }
//
//     DPRINT_OHCI!("OHCI_MapTransferToTD: SgIdx - %x, SgCount - %x\n",
//                 SgIdx,
//                 SGList->SgElementCount);
//
//     ASSERT(SgIdx < SGList->SgElementCount);
//     ASSERT(TransferedLen == SgElement->SgOffset);
//
//     /* The buffer for a TD can be 0 to 8192 bytes long,
//      * and can span within mo more than two 4k pages (see OpenHCI spec 3.3.2)
//      * CurrentBuffer - the (physical) address of the first byte in the buffer
//      * BufferEnd - the address of the last byte in the buffer. It can be on a different physical 4k page
//      * when a controller will reach the end of a page from CurrentBuffer, it will take the first 20 bits
//      * of the BufferEnd as a next address (OpenHCI spec, 4.3.1.3.1)
//      */
//
//     CurrentBuffer = SgElement->SgPhysicalAddress.LowPart;
//
//     if (TransferDataLeft <= SgElement->SgTransferLength)
//     {
//         CurrentTransferLen = TransferDataLeft;
//         BufferEnd = SgElement->SgPhysicalAddress.LowPart + CurrentTransferLen - 1;
//     }
//     else
//     {
//         PUSBPORT_SCATTER_GATHER_ELEMENT SgNextElement;
//         ASSERT(SGList->SgElementCount - SgIdx > 1);
//
//         SgNextElement = &SGList->SgElement[SgIdx + 1];
//
//         TransferDataLeft -= SgElement->SgTransferLength;
//         CurrentTransferLen = SgElement->SgTransferLength;
//
//         if (TransferDataLeft <= SgNextElement->SgTransferLength)
//         {
//             CurrentTransferLen += TransferDataLeft;
//             BufferEnd = SgNextElement->SgPhysicalAddress.LowPart + TransferDataLeft - 1;
//         }
//         else
//         {
//             CurrentTransferLen += SgNextElement->SgTransferLength;
//             BufferEnd = SgNextElement->SgPhysicalAddress.LowPart + SgNextElement->SgTransferLength - 1;
//         }
//     }
//
//     TD->HwTD.c2rust_unnamed.gTD.CurrentBuffer = CurrentBuffer;
//     TD->HwTD.c2rust_unnamed.gTD.BufferEnd = BufferEnd;
//     TD->TransferLen = CurrentTransferLen;
//
//     return TransferedLen + CurrentTransferLen;
// }
//
// POHCI_HCD_TD
// NTAPI
// OHCI_AllocateTD(IN POHCI_EXTENSION OhciExtension,
//                 IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_HCD_TD TD;
//
//     DPRINT_OHCI!("OHCI_AllocateTD: ... \n");
//
//     TD = OhciEndpoint->FirstTD;
//
//     while (TD->Flags & OHCI_HCD_TD_FLAG_ALLOCATED)
//     {
//         TD += 1;
//     }
//
//     TD->Flags |= OHCI_HCD_TD_FLAG_ALLOCATED;
//
//     RtlSecureZeroMemory(&TD->HwTD, sizeof(TD->HwTD));
//
//     return TD;
// }
//
// ULONG
// NTAPI
// OHCI_RemainTDs(IN POHCI_EXTENSION OhciExtension,
//                IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_HCD_TD TD;
//     ULONG MaxTDs;
//     ULONG RemainTDs;
//     ULONG ix;
//
//     DPRINT_OHCI!("OHCI_RemainTDs: ... \n");
//
//     MaxTDs = OhciEndpoint->MaxTransferDescriptors;
//     TD = OhciEndpoint->FirstTD;
//
//     RemainTDs = 0;
//
//     for (ix = 0; ix < MaxTDs; ix++)
//     {
//         if (!(TD->Flags & OHCI_HCD_TD_FLAG_ALLOCATED))
//             RemainTDs++;
//
//         TD += 1;
//     }
//
//     return RemainTDs;
// }
//
// static
// MPSTATUS
// OHCI_ControlTransfer(IN POHCI_EXTENSION OhciExtension,
//                      IN POHCI_ENDPOINT OhciEndpoint,
//                      IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                      IN POHCI_TRANSFER OhciTransfer,
//                      IN PUSBPORT_SCATTER_GATHER_LIST SGList)
// {
//     POHCI_HCD_TD SetupTD;
//     POHCI_HCD_TD TD;
//     POHCI_HCD_TD PrevTD;
//     ULONG MaxTDs;
//     ULONG TransferedLen;
//     UCHAR DataToggle;
//
//     DPRINT_OHCI!("OHCI_ControlTransfer: Ext %p, Endpoint %p\n",
//                 OhciExtension,
//                 OhciEndpoint);
//
//     MaxTDs = OHCI_RemainTDs(OhciExtension, OhciEndpoint);
//
//     if ((SGList->SgElementCount + OHCI_NON_DATA_CONTROL_TDS) > MaxTDs)
//         return MP_STATUS_FAILURE;
//
//     /* Form a setup packet first */
//     SetupTD = OhciEndpoint->HcdTailP;
//     RtlSecureZeroMemory(&SetupTD->HwTD, sizeof(SetupTD->HwTD));
//
//     SetupTD->Flags |= OHCI_HCD_TD_FLAG_PROCESSED;
//     SetupTD->OhciTransfer = OhciTransfer;
//
//     OhciTransfer->PendingTDs++;
//
//     RtlCopyMemory(&SetupTD->HwTD.c2rust_unnamed.SetupPacket,
//                   &TransferParameters->SetupPacket,
//                   sizeof(SetupTD->HwTD.c2rust_unnamed.SetupPacket));
//
//     SetupTD->HwTD.c2rust_unnamed.gTD.CurrentBuffer = SetupTD->PhysicalAddress + core::mem::offset_of!(OHCI_HCD_TD, HwTD.c2rust_unnamed.SetupPacket);
//     SetupTD->HwTD.c2rust_unnamed.gTD.BufferEnd = SetupTD->PhysicalAddress + core::mem::offset_of!(OHCI_HCD_TD, HwTD.c2rust_unnamed.SetupPacket) +
//                                   sizeof(USB_DEFAULT_PIPE_SETUP_PACKET) - 1;
//     SetupTD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_NONE;
//     SetupTD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NOT_ACCESSED;
//     SetupTD->HwTD.c2rust_unnamed.gTD.Control.DataToggle = OHCI_TD_DATA_TOGGLE_DATA0;
//
//     PrevTD = SetupTD;
//
//     /* Data packets follow a setup packet (if any) */
//     TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
//     TD->Flags |= OHCI_HCD_TD_FLAG_PROCESSED;
//     TD->OhciTransfer = OhciTransfer;
//
//     PrevTD->HwTD.c2rust_unnamed.gTD.NextTD = TD->PhysicalAddress;
//     PrevTD->NextTDVa = TD;
//
//     /* The first data packet should use DATA1, subsequent ones use DATA0 (OpenHCI spec, 4.3.1.3.4) */
//     DataToggle = OHCI_TD_DATA_TOGGLE_DATA1;
//     TransferedLen = 0;
//
//     while (TransferedLen < TransferParameters->TransferBufferLength)
//     {
//         OhciTransfer->PendingTDs++;
//
//         if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//             TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_IN;
//         else
//             TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_OUT;
//
//         TD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_NONE;
//         TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NOT_ACCESSED;
//         TD->HwTD.c2rust_unnamed.gTD.Control.DataToggle = DataToggle;
//
//         TransferedLen = OHCI_MapTransferToTD(OhciExtension,
//                                              TransferedLen,
//                                              OhciTransfer,
//                                              TD,
//                                              SGList);
//
//         PrevTD = TD;
//
//         TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
//         TD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_NONE;
//         TD->Flags |= OHCI_HCD_TD_FLAG_PROCESSED;
//         TD->OhciTransfer = OhciTransfer;
//
//         PrevTD->HwTD.c2rust_unnamed.gTD.NextTD = TD->PhysicalAddress;
//         PrevTD->NextTDVa = TD;
//
//         DataToggle = OHCI_TD_DATA_TOGGLE_DATA0;
//     }
//
//     if (TransferParameters->TransferFlags & USBD_SHORT_TRANSFER_OK)
//     {
//         PrevTD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = TRUE;
//         OhciTransfer->Flags |= OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK;
//     }
//
//     /* After data packets, goes a status packet */
//
//     TD->Flags |= OHCI_HCD_TD_FLAG_CONTROL_STATUS;
//     TD->TransferLen = 0;
//
//     if ((TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN) != 0)
//     {
//         TD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = FALSE;
//         TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_OUT;
//     }
//     else
//     {
//         TD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = TRUE;
//         TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_IN;
//     }
//
//     /* OpenHCI spec, 4.3.1.3.4 */
//     TD->HwTD.c2rust_unnamed.gTD.Control.DataToggle = OHCI_TD_DATA_TOGGLE_DATA1;
//     TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NOT_ACCESSED;
//     TD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_IMMEDIATE;
//
//     OhciTransfer->PendingTDs++;
//     OhciTransfer->ControlStatusTD = TD;
//
//     PrevTD = TD;
//
//     /* And the last descriptor, which is not used in the current transfer (OpenHCI spec, 4.6) */
//     TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
//
//     PrevTD->HwTD.c2rust_unnamed.gTD.NextTD = TD->PhysicalAddress;
//     PrevTD->NextTDVa = TD;
//
//     TD->NextTDVa = NULL;
//     /* TD->HwTD.c2rust_unnamed.gTD.NextTD = 0; */
//
//     OhciTransfer->NextTD = TD;
//     OhciEndpoint->HcdTailP = TD;
//
//     OhciEndpoint->HcdED->HwED.TailPointer = TD->PhysicalAddress;
//
//     OHCI_EnableList(OhciExtension, OhciEndpoint);
//
//     return MP_STATUS_SUCCESS;
// }
//
// static
// MPSTATUS
// OHCI_BulkOrInterruptTransfer(IN POHCI_EXTENSION OhciExtension,
//                              IN POHCI_ENDPOINT OhciEndpoint,
//                              IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                              IN POHCI_TRANSFER OhciTransfer,
//                              IN PUSBPORT_SCATTER_GATHER_LIST SGList)
// {
//     POHCI_HCD_TD TD;
//     POHCI_HCD_TD PrevTD;
//     ULONG TransferedLen;
//     ULONG MaxTDs;
//
//     DPRINT_OHCI!("OHCI_BulkOrInterruptTransfer: ... \n");
//
//     MaxTDs = OHCI_RemainTDs(OhciExtension, OhciEndpoint);
//
//     if (SGList->SgElementCount > MaxTDs)
//         return MP_STATUS_FAILURE;
//
//     TD = OhciEndpoint->HcdTailP;
//
//     TransferedLen = 0;
//
//     do
//     {
//         TD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_NONE;
//         TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NOT_ACCESSED;
//
//         if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//         {
//             TD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = FALSE;
//             TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_IN;
//         }
//         else
//         {
//             TD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = TRUE;
//             TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID = OHCI_TD_DIRECTION_PID_OUT;
//         }
//
//         TD->Flags |= OHCI_HCD_TD_FLAG_PROCESSED;
//         TD->OhciTransfer = OhciTransfer;
//
//         if (TransferParameters->TransferBufferLength)
//         {
//             TransferedLen = OHCI_MapTransferToTD(OhciExtension,
//                                                  TransferedLen,
//                                                  OhciTransfer,
//                                                  TD,
//                                                  SGList);
//         }
//         else
//         {
//             ASSERT(SGList->SgElementCount == 0);
//             TD->TransferLen = 0;
//         }
//
//         PrevTD = TD;
//
//         TD = OHCI_AllocateTD(OhciExtension, OhciEndpoint);
//         OhciTransfer->PendingTDs++;
//
//         PrevTD->HwTD.c2rust_unnamed.gTD.NextTD = TD->PhysicalAddress;
//         PrevTD->NextTDVa = TD;
//     }
//     while (TransferedLen < TransferParameters->TransferBufferLength);
//
//     if (TransferParameters->TransferFlags & USBD_SHORT_TRANSFER_OK)
//     {
//         PrevTD->HwTD.c2rust_unnamed.gTD.Control.BufferRounding = TRUE;
//         OhciTransfer->Flags |= OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK;
//     }
//
//     PrevTD->HwTD.c2rust_unnamed.gTD.Control.DelayInterrupt = OHCI_TD_INTERRUPT_IMMEDIATE;
//
//     /* The last TD in a chain is not used in a transfer. The controller does not access it
//      * so it will be used for chaining a next transfer to it (OpenHCI spec, 4.6)
//      */
//     /* TD->HwTD.c2rust_unnamed.gTD.NextTD = 0; */
//     TD->NextTDVa = NULL;
//
//     OhciTransfer->NextTD = TD;
//     OhciEndpoint->HcdTailP = TD;
//
//     OhciEndpoint->HcdED->HwED.TailPointer = TD->PhysicalAddress;
//
//     OHCI_EnableList(OhciExtension, OhciEndpoint);
//
//     return MP_STATUS_SUCCESS;
// }
//
// /**
//  * @brief      Creates the transfer descriptor chain for the given transfer's buffer
//  *             and attaches it to a given endpoint (for control, bulk or interrupt transfers)
//  *
//  * @param[in]  OhciExtension       The ohci extension
//  * @param[in]  OhciEndpoint        The ohci endpoint
//  * @param[in]  TransferParameters  The transfer parameters
//  * @param[in]  OhciTransfer        The ohci transfer
//  * @param[in]  SGList              The scatter/gather list
//  *
//  * @return     MP_STATUS_SUCCESS or MP_STATUS_FAILURE if there are not enough TDs left
//  *             or wrong transfer type given
//  */
// MPSTATUS
// NTAPI
// OHCI_SubmitTransfer(IN PVOID ohciExtension,
//                     IN PVOID ohciEndpoint,
//                     IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                     IN PVOID ohciTransfer,
//                     IN PUSBPORT_SCATTER_GATHER_LIST SGList)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_TRANSFER OhciTransfer = ohciTransfer;
//     ULONG TransferType;
//
//     DPRINT_OHCI!("OHCI_SubmitTransfer: ... \n");
//
//     RtlZeroMemory(OhciTransfer, sizeof(OHCI_TRANSFER));
//
//     OhciTransfer->TransferParameters = TransferParameters;
//     OhciTransfer->OhciEndpoint = OhciEndpoint;
//
//     TransferType = OhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         return OHCI_ControlTransfer(OhciExtension,
//                                     OhciEndpoint,
//                                     TransferParameters,
//                                     OhciTransfer,
//                                     SGList);
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         return OHCI_BulkOrInterruptTransfer(OhciExtension,
//                                             OhciEndpoint,
//                                             TransferParameters,
//                                             OhciTransfer,
//                                             SGList);
//     }
//
//     return MP_STATUS_FAILURE;
// }
//
// MPSTATUS
// NTAPI
// OHCI_SubmitIsoTransfer(IN PVOID ohciExtension,
//                        IN PVOID ohciEndpoint,
//                        IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                        IN PVOID ohciTransfer,
//                        IN PVOID isoParameters)
// {
//     DPRINT1("OHCI_SubmitIsoTransfer: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// OHCI_ProcessDoneTD(IN POHCI_EXTENSION OhciExtension,
//                    IN POHCI_HCD_TD TD,
//                    IN BOOLEAN IsPortComplete)
// {
//     POHCI_TRANSFER OhciTransfer;
//     POHCI_ENDPOINT OhciEndpoint;
//     ULONG Buffer;
//     ULONG BufferEnd;
//     ULONG Length;
//
//     DPRINT_OHCI!("OHCI_ProcessDoneTD: ... \n");
//
//     OhciTransfer = TD->OhciTransfer;
//     OhciEndpoint = OhciTransfer->OhciEndpoint;
//
//     OhciTransfer->PendingTDs--;
//
//     Buffer = TD->HwTD.c2rust_unnamed.gTD.CurrentBuffer;
//     BufferEnd = TD->HwTD.c2rust_unnamed.gTD.BufferEnd;
//
//     if (TD->Flags & OHCI_HCD_TD_FLAG_NOT_ACCESSED)
//     {
//         TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NO_ERROR;
//     }
//     else
//     {
//         if (TD->HwTD.c2rust_unnamed.gTD.CurrentBuffer)
//         {
//             if (TD->TransferLen)
//             {
//                 Length = (BufferEnd & (PAGE_SIZE - 1)) -
//                          (Buffer & (PAGE_SIZE - 1));
//
//                 Length++;
//
//                 if (Buffer >> PAGE_SHIFT != BufferEnd >> PAGE_SHIFT)
//                     Length += PAGE_SIZE;
//
//                 TD->TransferLen -= Length;
//             }
//         }
//
//         if (TD->HwTD.c2rust_unnamed.gTD.Control.DirectionPID != OHCI_TD_DIRECTION_PID_SETUP)
//             OhciTransfer->TransferLen += TD->TransferLen;
//
//         if (TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode)
//         {
//             OhciTransfer->USBDStatus = USBD_STATUS_HALTED |
//                                        TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode;
//         }
//     }
//
//     TD->Flags = 0;
//     TD->HwTD.c2rust_unnamed.gTD.NextTD = 0;
//     TD->OhciTransfer = 0;
//
//     TD->DoneLink.Flink = NULL;
//     TD->DoneLink.Blink = NULL;
//
//     if (IsPortComplete && (OhciTransfer->PendingTDs == 0))
//     {
//         RegPacket.UsbPortCompleteTransfer(OhciExtension,
//                                           OhciEndpoint,
//                                           OhciTransfer->TransferParameters,
//                                           OhciTransfer->USBDStatus,
//                                           OhciTransfer->TransferLen);
//     }
// }
//
// VOID
// NTAPI
// OHCI_ProcessDoneIsoTD(IN POHCI_EXTENSION OhciExtension,
//                       IN POHCI_HCD_TD TD,
//                       IN BOOLEAN IsPortComplete)
// {
//     DPRINT1("OHCI_ProcessDoneIsoTD: UNIMPLEMENTED. FIXME\n");
// }
//
// /**
//  * @brief      Aborts the transfer descriptor chain in a given endpoint
//  *
//  * @param[in]  ohciExtension   The ohci extension
//  * @param[in]  ohciEndpoint    The ohci endpoint
//  * @param[in]  ohciTransfer    The ohci transfer
//  * @param[out] CompletedLength
//  */
// VOID
// NTAPI
// OHCI_AbortTransfer(IN PVOID ohciExtension,
//                    IN PVOID ohciEndpoint,
//                    IN PVOID ohciTransfer,
//                    IN OUT PULONG CompletedLength)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_TRANSFER OhciTransfer = ohciTransfer;
//     POHCI_HCD_ED ED = OhciEndpoint->HcdED;
//     POHCI_HCD_TD TD, NextTD, LastTD;
//     ULONG ix;
//     BOOLEAN IsIsoEndpoint;
//     BOOLEAN IsProcessed = FALSE;
//
//     DPRINT("OHCI_AbortTransfer: ohciEndpoint - %p, ohciTransfer - %p\n",
//            OhciEndpoint,
//            OhciTransfer);
//
//     IsIsoEndpoint = (OhciEndpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS);
//     NextTD = RegPacket.UsbPortGetMappedVirtualAddress(ED->HwED.HeadPointer & OHCI_ED_HEAD_POINTER_MASK,
//                                                       OhciExtension,
//                                                       OhciEndpoint);
//
//     if (NextTD->OhciTransfer == OhciTransfer)
//     {
//         LastTD = OhciTransfer->NextTD;
//
//         /* Keeping the carry bit from previous pointer value */
//         ED->HwED.HeadPointer = LastTD->PhysicalAddress |
//                                (ED->HwED.HeadPointer & OHCI_ED_HEAD_POINTER_CARRY);
//
//         OhciEndpoint->HcdHeadP = LastTD;
//
//         for (ix = 0; ix < OhciEndpoint->MaxTransferDescriptors; ix++)
//         {
//             TD = &OhciEndpoint->FirstTD[ix];
//
//             if (TD->OhciTransfer != OhciTransfer)
//                 continue;
//
//             if (IsIsoEndpoint)
//                 OHCI_ProcessDoneIsoTD(OhciExtension, TD, FALSE);
//             else
//                 OHCI_ProcessDoneTD(OhciExtension, TD, FALSE);
//         }
//
//         *CompletedLength = OhciTransfer->TransferLen;
//         return;
//     }
//
//     if (NextTD == OhciEndpoint->HcdHeadP)
//         IsProcessed = TRUE;
//
//     for (TD = OhciEndpoint->HcdHeadP; TD != NextTD; TD = TD->NextTDVa)
//     {
//         if (TD->OhciTransfer != OhciTransfer)
//             continue;
//
//         if (OhciEndpoint->HcdHeadP == TD)
//             OhciEndpoint->HcdHeadP = TD->NextTDVa;
//
//         if (IsIsoEndpoint)
//             OHCI_ProcessDoneIsoTD(OhciExtension, TD, FALSE);
//         else
//             OHCI_ProcessDoneTD(OhciExtension, TD, FALSE);
//
//         IsProcessed = TRUE;
//     }
//
//     if (!IsProcessed)
//     {
//         for (TD = OhciEndpoint->HcdHeadP; TD->OhciTransfer != OhciTransfer; TD = TD->NextTDVa)
//         {
//             if (TD == OhciEndpoint->HcdTailP)
//             {
//                 TD = NULL;
//                 break;
//             }
//             LastTD = TD;
//         }
//
//         for (; TD->OhciTransfer == OhciTransfer; TD = TD->NextTDVa)
//         {
//             if (TD == OhciEndpoint->HcdTailP)
//                 break;
//
//             if (IsIsoEndpoint)
//                 OHCI_ProcessDoneIsoTD(OhciExtension, TD, FALSE);
//             else
//                 OHCI_ProcessDoneTD(OhciExtension, TD, FALSE);
//         }
//
//         LastTD->OhciTransfer->NextTD = TD;
//
//         LastTD->NextTDVa = TD;
//         LastTD->HwTD.c2rust_unnamed.gTD.NextTD = TD->PhysicalAddress;
//     }
//
//     *CompletedLength = OhciTransfer->TransferLen;
//
//     if (OhciTransfer->TransferLen)
//     {
//         DPRINT("OHCI_AbortTransfer: *CompletedLength - %x\n", *CompletedLength);
//     }
// }
//
// ULONG
// NTAPI
// OHCI_GetEndpointState(IN PVOID ohciExtension,
//                       IN PVOID ohciEndpoint)
// {
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_GetEndpointState: ... \n");
//
//     ED = OhciEndpoint->HcdED;
//
//     if (ED->Flags & OHCI_HCD_TD_FLAG_NOT_ACCESSED)
//         return USBPORT_ENDPOINT_REMOVE;
//
//     if (ED->HwED.EndpointControl.sKip)
//         return USBPORT_ENDPOINT_PAUSED;
//
//     return USBPORT_ENDPOINT_ACTIVE;
// }
//
// VOID
// NTAPI
// OHCI_RemoveEndpointFromSchedule(IN POHCI_ENDPOINT OhciEndpoint)
// {
//     POHCI_HCD_ED ED;
//     POHCI_HCD_ED PreviousED;
//     POHCI_STATIC_ED HeadED;
//
//     DPRINT_OHCI!("OHCI_RemoveEndpointFromSchedule \n");
//
//     ED = OhciEndpoint->HcdED;
//     HeadED = OhciEndpoint->HeadED;
//
//     if (&HeadED->Link == ED->HcdEDLink.Blink)
//     {
//         if (HeadED->Type == OHCI_STATIC_ED_TYPE_CONTROL ||
//             HeadED->Type == OHCI_STATIC_ED_TYPE_BULK)
//         {
//             WRITE_REGISTER_ULONG(HeadED->pNextED, ED->HwED.NextED);
//         }
//         else if (HeadED->Type == OHCI_STATIC_ED_TYPE_INTERRUPT)
//         {
//             *HeadED->pNextED = ED->HwED.NextED;
//         }
//         else
//         {
//             DPRINT1("OHCI_RemoveEndpointFromSchedule: Unknown HeadED->Type - %x\n",
//                     HeadED->Type);
//             DbgBreakPoint();
//         }
//     }
//     else
//     {
//         PreviousED = CONTAINING_RECORD(ED->HcdEDLink.Blink,
//                                        OHCI_HCD_ED,
//                                        HcdEDLink);
//
//         PreviousED->HwED.NextED = ED->HwED.NextED;
//     }
//
//     RemoveEntryList(&ED->HcdEDLink);
//
//     OhciEndpoint->HeadED = NULL;
// }
//
// VOID
// NTAPI
// OHCI_SetEndpointState(IN PVOID ohciExtension,
//                       IN PVOID ohciEndpoint,
//                       IN ULONG EndpointState)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_SetEndpointState: EndpointState - %x\n",
//                 EndpointState);
//
//     ED = OhciEndpoint->HcdED;
//
//     switch (EndpointState)
//     {
//         case USBPORT_ENDPOINT_PAUSED:
//             ED->HwED.EndpointControl.sKip = 1;
//             break;
//
//         case USBPORT_ENDPOINT_ACTIVE:
//             ED->HwED.EndpointControl.sKip = 0;
//             OHCI_EnableList(OhciExtension, OhciEndpoint);
//             break;
//
//         case USBPORT_ENDPOINT_REMOVE:
//             ED->HwED.EndpointControl.sKip = 1;
//             ED->Flags |= OHCI_HCD_ED_FLAG_NOT_ACCESSED;
//             OHCI_RemoveEndpointFromSchedule(OhciEndpoint);
//             break;
//
//         default:
//             ASSERT(FALSE);
//             break;
//     }
// }
//
// VOID
// NTAPI
// OHCI_PollAsyncEndpoint(IN POHCI_EXTENSION OhciExtension,
//                        IN POHCI_ENDPOINT OhciEndpoint)
// {
//     PUSBPORT_TRANSFER_PARAMETERS TransferParameters;
//     POHCI_HCD_ED ED;
//     ULONG_PTR NextTdPA;
//     POHCI_HCD_TD NextTD;
//     POHCI_HCD_TD TD;
//     PLIST_ENTRY DoneList;
//     POHCI_TRANSFER OhciTransfer;
//     POHCI_HCD_TD ControlStatusTD;
//     ULONG_PTR PhysicalAddress;
//     ULONG TransferNumber;
//     POHCI_TRANSFER transfer;
//     UCHAR ConditionCode;
//     BOOLEAN IsResetOnHalt = FALSE;
//
//     //DPRINT_OHCI!("OHCI_PollAsyncEndpoint: Endpoint - %p\n", OhciEndpoint);
//
//     ED = OhciEndpoint->HcdED;
//     NextTdPA = ED->HwED.HeadPointer & OHCI_ED_HEAD_POINTER_MASK;
//
//     if (!NextTdPA)
//     {
//         OHCI_DumpHcdED(ED);
//         DbgBreakPoint();
//     }
//
//     NextTD = RegPacket.UsbPortGetMappedVirtualAddress(NextTdPA,
//                                                       OhciExtension,
//                                                       OhciEndpoint);
//     DPRINT_OHCI!("NextTD - %p\n", NextTD);
//
//     if ((ED->HwED.HeadPointer & OHCI_ED_HEAD_POINTER_HALT) == 0)
//         goto ProcessListTDs;
//
//     OHCI_DumpHcdED(ED);
//
//     IsResetOnHalt = (ED->Flags & OHCI_HCD_ED_FLAG_RESET_ON_HALT) != 0;
//     DPRINT1("PollAsyncEndpoint: IsResetOnHalt %x\n", IsResetOnHalt);
//
//     for (TD = OhciEndpoint->HcdHeadP; ; TD = TD->NextTDVa)
//     {
//         if (!TD)
//         {
//             OHCI_DumpHcdED(ED);
//             DbgBreakPoint();
//         }
//
//         if (TD == NextTD)
//         {
//             DPRINT("TD == NextTD - %p\n", TD);
//             goto HandleDoneList;
//         }
//
//         OhciTransfer = TD->OhciTransfer;
//         ConditionCode = TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode;
//
//         DPRINT("TD - %p, ConditionCode - %X\n", TD, ConditionCode);
//         OHCI_DumpHcdTD(TD);
//
//         switch (ConditionCode)
//         {
//             case OHCI_TD_CONDITION_NO_ERROR:
//                 TD->Flags |= OHCI_HCD_TD_FLAG_DONE;
//                 InsertTailList(&OhciEndpoint->TDList, &TD->DoneLink);
//                 continue;
//
//             case OHCI_TD_CONDITION_NOT_ACCESSED:
//                 TD->Flags |= (OHCI_HCD_TD_FLAG_DONE | OHCI_HCD_TD_FLAG_NOT_ACCESSED);
//                 InsertTailList(&OhciEndpoint->TDList, &TD->DoneLink);
//                 continue;
//
//             case OHCI_TD_CONDITION_DATA_UNDERRUN:
//                 DPRINT1("DATA_UNDERRUN. Transfer->Flags - %X\n", OhciTransfer->Flags);
//
//                 if (OhciTransfer->Flags & OHCI_TRANSFER_FLAGS_SHORT_TRANSFER_OK)
//                 {
//                     IsResetOnHalt = TRUE;
//                     TD->HwTD.c2rust_unnamed.gTD.Control.ConditionCode = OHCI_TD_CONDITION_NO_ERROR;
//
//                     ControlStatusTD = OhciTransfer->ControlStatusTD;
//
//                     if ((TD->Flags & OHCI_HCD_TD_FLAG_CONTROL_STATUS) == 0 &&
//                         ControlStatusTD)
//                     {
//                         PhysicalAddress = ControlStatusTD->PhysicalAddress;
//                         PhysicalAddress |= (ED->HwED.HeadPointer &
//                                             OHCI_ED_HEAD_POINTER_FLAGS_MASK);
//
//                         ED->HwED.HeadPointer = PhysicalAddress;
//
//                         NextTD = OhciTransfer->ControlStatusTD;
//                         DPRINT("PhysicalAddress - %p, NextTD - %p\n", PhysicalAddress, NextTD);
//                     }
//                     else
//                     {
//                         TransferParameters = OhciTransfer->TransferParameters;
//
//                         if (TransferParameters->IsTransferSplited)
//                         {
//                             TransferNumber = TransferParameters->TransferCounter;
//                             transfer = OhciTransfer;
//
//                             do
//                             {
//                                 transfer = transfer->NextTD->OhciTransfer;
//                                 NextTD = transfer->NextTD;
//                             }
//                             while (transfer && TransferNumber ==
//                                    transfer->TransferParameters->TransferCounter);
//
//                             PhysicalAddress = NextTD->PhysicalAddress;
//                             PhysicalAddress |= (ED->HwED.HeadPointer &
//                                                 OHCI_ED_HEAD_POINTER_FLAGS_MASK);
//
//                             ED->HwED.HeadPointer = PhysicalAddress;
//                             DPRINT("PhysicalAddress - %p, NextTD - %p\n", PhysicalAddress, NextTD);
//                         }
//                         else
//                         {
//                             PhysicalAddress = OhciTransfer->NextTD->PhysicalAddress;
//                             PhysicalAddress |= (ED->HwED.HeadPointer &
//                                                 OHCI_ED_HEAD_POINTER_FLAGS_MASK);
//
//                             ED->HwED.HeadPointer = PhysicalAddress;
//
//                             NextTD = OhciTransfer->NextTD;
//                             DPRINT("PhysicalAddress - %p, NextTD - %p\n", PhysicalAddress, NextTD);
//                         }
//                     }
//
//                     TD->Flags |= OHCI_HCD_TD_FLAG_DONE;
//                     InsertTailList(&OhciEndpoint->TDList, &TD->DoneLink);
//                     continue;
//                 }
//
//                 /* fall through */
//
//             default:
//                 TD->Flags |= OHCI_HCD_TD_FLAG_DONE;
//                 InsertTailList(&OhciEndpoint->TDList, &TD->DoneLink);
//
//                 ED->HwED.HeadPointer = OhciTransfer->NextTD->PhysicalAddress |
//                                        (ED->HwED.HeadPointer &
//                                         OHCI_ED_HEAD_POINTER_FLAGS_MASK);
//
//                 NextTD = OhciTransfer->NextTD;
//                 break;
//         }
//     }
//
// ProcessListTDs:
//
//     TD = OhciEndpoint->HcdHeadP;
//
//     while (TD != NextTD)
//     {
//         OHCI_DumpHcdTD(TD);
//         TD->Flags |= OHCI_HCD_TD_FLAG_DONE;
//         InsertTailList(&OhciEndpoint->TDList, &TD->DoneLink);
//         TD = TD->NextTDVa;
//     }
//
// HandleDoneList:
//
//     TD = NextTD;
//     OhciEndpoint->HcdHeadP = NextTD;
//
//     DoneList = &OhciEndpoint->TDList;
//
//     while (!IsListEmpty(DoneList))
//     {
//         TD = CONTAINING_RECORD(DoneList->Flink,
//                                OHCI_HCD_TD,
//                                DoneLink);
//
//         RemoveHeadList(DoneList);
//
//         if (TD->Flags & OHCI_HCD_TD_FLAG_DONE &&
//             TD->Flags & OHCI_HCD_TD_FLAG_PROCESSED)
//         {
//             OHCI_ProcessDoneTD(OhciExtension, TD, TRUE);
//         }
//     }
//
//     if (IsResetOnHalt)
//     {
//         ED->HwED.HeadPointer &= ~OHCI_ED_HEAD_POINTER_HALT;
//         DPRINT("ED->HwED.HeadPointer - %p\n", ED->HwED.HeadPointer);
//     }
// }
//
// VOID
// NTAPI
// OHCI_PollIsoEndpoint(IN POHCI_EXTENSION OhciExtension,
//                      IN POHCI_ENDPOINT OhciEndpoint)
// {
//     DPRINT1("OHCI_PollAsyncEndpoint: UNIMPLEMENTED. FIXME \n");
//     ASSERT(FALSE);
// }
//
// VOID
// NTAPI
// OHCI_PollEndpoint(IN PVOID ohciExtension,
//                   IN PVOID ohciEndpoint)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     ULONG TransferType;
//
//     DPRINT_OHCI!("OHCI_PollEndpoint: OhciExtension - %p, Endpoint - %p\n",
//                 OhciExtension,
//                 OhciEndpoint);
//
//     TransferType = OhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         OHCI_PollIsoEndpoint(OhciExtension, OhciEndpoint);
//         return;
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         OHCI_PollAsyncEndpoint(OhciExtension, OhciEndpoint);
//     }
// }
//
// VOID
// NTAPI
// OHCI_CheckController(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG HcControlReg;
//     OHCI_REG_CONTROL HcControl;
//     ULONG FmNumber;
//     USHORT FmDiff;
//     POHCI_HCCA HcHCCA;
//
//     //DPRINT_OHCI!("OHCI_CheckController: ...\n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     if (!OHCI_HardwarePresent(OhciExtension, TRUE))
//         return;
//
//     HcControlReg = (PULONG)&OperationalRegs->HcControl;
//     HcControl.AsULONG = READ_REGISTER_ULONG(HcControlReg);
//
//     if (HcControl.HostControllerFunctionalState != OHCI_HC_STATE_OPERATIONAL)
//         return;
//
//     FmNumber = READ_REGISTER_ULONG(&OperationalRegs->HcFmNumber);
//     FmDiff = (USHORT)(FmNumber - OhciExtension->HcdFmNumber);
//
//     if (FmNumber == 0 || FmDiff < 5)
//         return;
//
//     HcHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//     OhciExtension->HcdFmNumber = FmNumber;
//
//     if (HcHCCA->Pad1 == 0)
//     {
//         HcHCCA->Pad1 = 0xBAD1;
//         return;
//     }
//
//     DPRINT1("OHCI_CheckController: HcHCCA->Pad1 - %x\n", HcHCCA->Pad1);
//
//     if (HcHCCA->Pad1 == 0xBAD1)
//     {
//         HcHCCA->Pad1 = 0xBAD2;
//     }
//     else if (HcHCCA->Pad1 == 0xBAD2)
//     {
//         HcHCCA->Pad1 = 0xBAD3;
//
//         RegPacket.UsbPortInvalidateController(OhciExtension,
//                                               USBPORT_INVALIDATE_CONTROLLER_RESET);
//     }
// }
//
// ULONG
// NTAPI
// OHCI_Get32BitFrameNumber(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_HCCA HcHCCA;
//     ULONG fm;
//     ULONG hp;
//
//     HcHCCA = &OhciExtension->HcResourcesVA->HcHCCA;
//
//     /* 5.4 FrameInterval Counter: Get32BitFrameNumber() */
//
//     hp = OhciExtension->FrameHighPart;
//     fm = HcHCCA->FrameNumber;
//
//     DPRINT_OHCI!("OHCI_Get32BitFrameNumber: hp - %lX, fm - %lX\n", hp, fm);
//
//     return ((fm & 0x7FFF) | hp) + ((fm ^ hp) & 0x8000);
// }
//
// VOID
// NTAPI
// OHCI_InterruptNextSOF(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptEnableReg;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntEnable;
//
//     DPRINT_OHCI!("OHCI_InterruptNextSOF: OhciExtension - %p\n",
//                 OhciExtension);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//
//     /* Enable interrupt generation due to Start of Frame */
//     IntEnable.AsULONG = 0;
//     IntEnable.StartofFrame = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, IntEnable.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_EnableInterrupts(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptEnableReg;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntEnable;
//
//     DPRINT_OHCI!("OHCI_EnableInterrupts: OhciExtension - %p\n",
//                 OhciExtension);
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//
//     /*  Enable interrupt generation */
//     IntEnable.AsULONG = 0;
//     IntEnable.MasterInterruptEnable = 1;
//
//     WRITE_REGISTER_ULONG(InterruptEnableReg, IntEnable.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_DisableInterrupts(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION  OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     PULONG InterruptDisableReg;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntDisable;
//
//     DPRINT_OHCI!("OHCI_DisableInterrupts\n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//     InterruptDisableReg = (PULONG)&OperationalRegs->HcInterruptDisable;
//
//     /*  Disable interrupt generation */
//     IntDisable.AsULONG = 0;
//     IntDisable.MasterInterruptEnable = 1;
//
//     WRITE_REGISTER_ULONG(InterruptDisableReg, IntDisable.AsULONG);
// }
//
// VOID
// NTAPI
// OHCI_PollController(IN PVOID ohciExtension)
// {
//     DPRINT1("OHCI_PollController: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// OHCI_SetEndpointDataToggle(IN PVOID ohciExtension,
//                            IN PVOID ohciEndpoint,
//                            IN ULONG DataToggle)
// {
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_SetEndpointDataToggle: Endpoint - %p, DataToggle - %x\n",
//                 OhciEndpoint,
//                 DataToggle);
//
//     ED = OhciEndpoint->HcdED;
//
//     if (DataToggle)
//         ED->HwED.HeadPointer |= OHCI_ED_HEAD_POINTER_CARRY;
//     else
//         ED->HwED.HeadPointer &= ~OHCI_ED_HEAD_POINTER_CARRY;
// }
//
// ULONG
// NTAPI
// OHCI_GetEndpointStatus(IN PVOID ohciExtension,
//                        IN PVOID ohciEndpoint)
// {
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//     ULONG EndpointStatus = USBPORT_ENDPOINT_RUN;
//
//     DPRINT_OHCI!("OHCI_GetEndpointStatus: ... \n");
//
//     ED = OhciEndpoint->HcdED;
//
//     if ((ED->HwED.HeadPointer & OHCI_ED_HEAD_POINTER_HALT) &&
//         !(ED->Flags & OHCI_HCD_ED_FLAG_RESET_ON_HALT))
//     {
//         EndpointStatus = USBPORT_ENDPOINT_HALT;
//     }
//
//     return EndpointStatus;
// }
//
// VOID
// NTAPI
// OHCI_SetEndpointStatus(IN PVOID ohciExtension,
//                        IN PVOID ohciEndpoint,
//                        IN ULONG EndpointStatus)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_ENDPOINT OhciEndpoint = ohciEndpoint;
//     POHCI_HCD_ED ED;
//
//     DPRINT_OHCI!("OHCI_SetEndpointStatus: Endpoint - %p, EndpointStatus - %lX\n",
//                 OhciEndpoint,
//                 EndpointStatus);
//
//     if (EndpointStatus == USBPORT_ENDPOINT_RUN)
//     {
//         ED = OhciEndpoint->HcdED;
//         ED->HwED.HeadPointer &= ~OHCI_ED_HEAD_POINTER_HALT;
//
//         OHCI_EnableList(OhciExtension, OhciEndpoint);
//     }
//     else if (EndpointStatus == USBPORT_ENDPOINT_HALT)
//     {
//         ASSERT(FALSE);
//     }
// }
//
// VOID
// NTAPI
// OHCI_ResetController(IN PVOID ohciExtension)
// {
//     POHCI_EXTENSION OhciExtension = ohciExtension;
//     POHCI_OPERATIONAL_REGISTERS OperationalRegs;
//     ULONG FrameNumber;
//     PULONG ControlReg;
//     PULONG CommandStatusReg;
//     PULONG InterruptEnableReg;
//     PULONG FmIntervalReg;
//     PULONG RhStatusReg;
//     PULONG PortStatusReg;
//     OHCI_REG_CONTROL ControlBak;
//     OHCI_REG_CONTROL Control;
//     OHCI_REG_COMMAND_STATUS CommandStatus;
//     OHCI_REG_INTERRUPT_ENABLE_DISABLE IntEnable;
//     ULONG_PTR HCCA;
//     ULONG_PTR ControlHeadED;
//     ULONG_PTR BulkHeadED;
//     OHCI_REG_FRAME_INTERVAL FrameInterval;
//     ULONG_PTR PeriodicStart;
//     ULONG_PTR LSThreshold;
//     OHCI_REG_RH_STATUS RhStatus;
//     OHCI_REG_RH_DESCRIPTORA RhDescriptorA;
//     OHCI_REG_RH_PORT_STATUS PortStatus;
//     ULONG NumPorts;
//     ULONG ix;
//
//     DPRINT("OHCI_ResetController: ... \n");
//
//     OperationalRegs = OhciExtension->OperationalRegs;
//
//     ControlReg = (PULONG)&OperationalRegs->HcControl;
//     CommandStatusReg = (PULONG)&OperationalRegs->HcCommandStatus;
//     InterruptEnableReg = (PULONG)&OperationalRegs->HcInterruptEnable;
//     FmIntervalReg = (PULONG)&OperationalRegs->HcFmInterval;
//     RhStatusReg = (PULONG)&OperationalRegs->HcRhStatus;
//
//     /* Backup FrameNumber from HcHCCA */
//     FrameNumber = OhciExtension->HcResourcesVA->HcHCCA.FrameNumber;
//
//     /* Backup registers */
//     ControlBak.AsULONG = READ_REGISTER_ULONG(ControlReg);
//     HCCA = READ_REGISTER_ULONG(&OperationalRegs->HcHCCA);
//     ControlHeadED = READ_REGISTER_ULONG(&OperationalRegs->HcControlHeadED);
//     BulkHeadED = READ_REGISTER_ULONG(&OperationalRegs->HcBulkHeadED);
//     FrameInterval.AsULONG = READ_REGISTER_ULONG(FmIntervalReg);
//     PeriodicStart = READ_REGISTER_ULONG(&OperationalRegs->HcPeriodicStart);
//     LSThreshold = READ_REGISTER_ULONG(&OperationalRegs->HcLSThreshold);
//
//     /* Reset HostController */
//     CommandStatus.AsULONG = 0;
//     CommandStatus.HostControllerReset = 1;
//     WRITE_REGISTER_ULONG(CommandStatusReg, CommandStatus.AsULONG);
//
//     KeStallExecutionProcessor(10);
//
//     /* Restore registers */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcHCCA, HCCA);
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcControlHeadED, ControlHeadED);
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcBulkHeadED, BulkHeadED);
//
//     /* Set OPERATIONAL state for HC */
//     Control.AsULONG = 0;
//     Control.HostControllerFunctionalState = OHCI_HC_STATE_OPERATIONAL;
//     WRITE_REGISTER_ULONG(ControlReg, Control.AsULONG);
//
//     /* Set Toggle bit for FmInterval register */
//     FrameInterval.FrameIntervalToggle = 1;
//     WRITE_REGISTER_ULONG(FmIntervalReg, FrameInterval.AsULONG);
//
//     /* Restore registers */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcFmNumber, FrameNumber);
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcPeriodicStart, PeriodicStart);
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcLSThreshold, LSThreshold);
//
//     /* Setup RhStatus register */
//     RhStatus.AsULONG = 0;
//     RhStatus.SetRemoteWakeupEnable = 1;
//     RhStatus.SetGlobalPower = 1;
//     WRITE_REGISTER_ULONG(RhStatusReg, RhStatus.AsULONG);
//
//     /* Setup RH PortStatus registers */
//     RhDescriptorA = OHCI_ReadRhDescriptorA(OhciExtension);
//     NumPorts = RhDescriptorA.NumberDownstreamPorts;
//
//     PortStatus.AsULONG = 0;
//     PortStatus.SetPortPower = 1;
//
//     for (ix = 0; ix < NumPorts; ix++)
//     {
//         PortStatusReg = (PULONG)&OperationalRegs->HcRhPortStatus[ix];
//         WRITE_REGISTER_ULONG(PortStatusReg, PortStatus.AsULONG);
//     }
//
//     /* Restore HcControl register */
//     ControlBak.HostControllerFunctionalState = OHCI_HC_STATE_OPERATIONAL;
//     WRITE_REGISTER_ULONG(ControlReg, ControlBak.AsULONG);
//
//     /* Setup HcInterruptEnable register */
//     IntEnable.AsULONG = 0xFFFFFFFF;
//     IntEnable.Reserved1 = 0;
//     WRITE_REGISTER_ULONG(InterruptEnableReg, IntEnable.AsULONG);
// }
//
// MPSTATUS
// NTAPI
// OHCI_StartSendOnePacket(IN PVOID ohciExtension,
//                         IN PVOID PacketParameters,
//                         IN PVOID Data,
//                         IN PULONG pDataLength,
//                         IN PVOID BufferVA,
//                         IN PVOID BufferPA,
//                         IN ULONG BufferLength,
//                         IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT1("OHCI_StartSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_EndSendOnePacket(IN PVOID ohciExtension,
//                       IN PVOID PacketParameters,
//                       IN PVOID Data,
//                       IN PULONG pDataLength,
//                       IN PVOID BufferVA,
//                       IN PVOID BufferPA,
//                       IN ULONG BufferLength,
//                       IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT1("OHCI_EndSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// OHCI_PassThru(IN PVOID ohciExtension,
//               IN PVOID passThruParameters,
//               IN ULONG ParameterLength,
//               IN PVOID pParameters)
// {
//     DPRINT1("OHCI_PassThru: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// OHCI_Unload(IN PDRIVER_OBJECT DriverObject)
// {
//     DPRINT1("OHCI_Unload: Not supported\n");
//     return;
// }
//
// VOID
// NTAPI
// OHCI_FlushInterrupts(IN PVOID uhciExtension)
// {
//     DPRINT1("OHCI_FlushInterrupts: Not supported\n");
//     return;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     NTSTATUS Status;
//
//     DPRINT_OHCI!("DriverEntry: DriverObject - %p, RegistryPath - %wZ\n",
//                 DriverObject,
//                 RegistryPath);
//
//     RtlZeroMemory(&RegPacket, sizeof(USBPORT_REGISTRATION_PACKET));
//
//     RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_OHCI;
//
//     RegPacket.MiniPortFlags = USB_MINIPORT_FLAGS_INTERRUPT |
//                               USB_MINIPORT_FLAGS_MEMORY_IO;
//
//     RegPacket.MiniPortBusBandwidth = TOTAL_USB11_BUS_BANDWIDTH;
//
//     RegPacket.MiniPortExtensionSize = sizeof(OHCI_EXTENSION);
//     RegPacket.MiniPortEndpointSize = sizeof(OHCI_ENDPOINT);
//     RegPacket.MiniPortTransferSize = sizeof(OHCI_TRANSFER);
//     RegPacket.MiniPortResourcesSize = sizeof(OHCI_HC_RESOURCES);
//
//     RegPacket.OpenEndpoint = OHCI_OpenEndpoint;
//     RegPacket.ReopenEndpoint = OHCI_ReopenEndpoint;
//     RegPacket.QueryEndpointRequirements = OHCI_QueryEndpointRequirements;
//     RegPacket.CloseEndpoint = OHCI_CloseEndpoint;
//     RegPacket.StartController = OHCI_StartController;
//     RegPacket.StopController = OHCI_StopController;
//     RegPacket.SuspendController = OHCI_SuspendController;
//     RegPacket.ResumeController = OHCI_ResumeController;
//     RegPacket.InterruptService = OHCI_InterruptService;
//     RegPacket.InterruptDpc = OHCI_InterruptDpc;
//     RegPacket.SubmitTransfer = OHCI_SubmitTransfer;
//     RegPacket.SubmitIsoTransfer = OHCI_SubmitIsoTransfer;
//     RegPacket.AbortTransfer = OHCI_AbortTransfer;
//     RegPacket.GetEndpointState = OHCI_GetEndpointState;
//     RegPacket.SetEndpointState = OHCI_SetEndpointState;
//     RegPacket.PollEndpoint = OHCI_PollEndpoint;
//     RegPacket.CheckController = OHCI_CheckController;
//     RegPacket.Get32BitFrameNumber = OHCI_Get32BitFrameNumber;
//     RegPacket.InterruptNextSOF = OHCI_InterruptNextSOF;
//     RegPacket.EnableInterrupts = OHCI_EnableInterrupts;
//     RegPacket.DisableInterrupts = OHCI_DisableInterrupts;
//     RegPacket.PollController = OHCI_PollController;
//     RegPacket.SetEndpointDataToggle = OHCI_SetEndpointDataToggle;
//     RegPacket.GetEndpointStatus = OHCI_GetEndpointStatus;
//     RegPacket.SetEndpointStatus = OHCI_SetEndpointStatus;
//     RegPacket.ResetController = OHCI_ResetController;
//     RegPacket.RH_GetRootHubData = OHCI_RH_GetRootHubData;
//     RegPacket.RH_GetStatus = OHCI_RH_GetStatus;
//     RegPacket.RH_GetPortStatus = OHCI_RH_GetPortStatus;
//     RegPacket.RH_GetHubStatus = OHCI_RH_GetHubStatus;
//     RegPacket.RH_SetFeaturePortReset = OHCI_RH_SetFeaturePortReset;
//     RegPacket.RH_SetFeaturePortPower = OHCI_RH_SetFeaturePortPower;
//     RegPacket.RH_SetFeaturePortEnable = OHCI_RH_SetFeaturePortEnable;
//     RegPacket.RH_SetFeaturePortSuspend = OHCI_RH_SetFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnable = OHCI_RH_ClearFeaturePortEnable;
//     RegPacket.RH_ClearFeaturePortPower = OHCI_RH_ClearFeaturePortPower;
//     RegPacket.RH_ClearFeaturePortSuspend = OHCI_RH_ClearFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnableChange = OHCI_RH_ClearFeaturePortEnableChange;
//     RegPacket.RH_ClearFeaturePortConnectChange = OHCI_RH_ClearFeaturePortConnectChange;
//     RegPacket.RH_ClearFeaturePortResetChange = OHCI_RH_ClearFeaturePortResetChange;
//     RegPacket.RH_ClearFeaturePortSuspendChange = OHCI_RH_ClearFeaturePortSuspendChange;
//     RegPacket.RH_ClearFeaturePortOvercurrentChange = OHCI_RH_ClearFeaturePortOvercurrentChange;
//     RegPacket.RH_DisableIrq = OHCI_RH_DisableIrq;
//     RegPacket.RH_EnableIrq = OHCI_RH_EnableIrq;
//     RegPacket.StartSendOnePacket = OHCI_StartSendOnePacket;
//     RegPacket.EndSendOnePacket = OHCI_EndSendOnePacket;
//     RegPacket.PassThru = OHCI_PassThru;
//     RegPacket.FlushInterrupts = OHCI_FlushInterrupts;
//
//     DriverObject->DriverUnload = OHCI_Unload;
//
//     Status = USBPORT_RegisterUSBPortDriver(DriverObject,
//                                            USB10_MINIPORT_INTERFACE_VERSION,
//                                            &RegPacket);
//
//     DPRINT_OHCI!("DriverEntry: USBPORT_RegisterUSBPortDriver return Status - %x\n",
//                 Status);
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
