use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{LARGE_INTEGER, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

extern "system" {
    fn READ_PORT_USHORT(Register: *mut USHORT) -> USHORT;
}
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}
extern "system" {
    fn READ_PORT_UCHAR(Port: *mut UCHAR) -> UCHAR;
    fn WRITE_PORT_UCHAR(Port: *mut UCHAR, Value: UCHAR);
    fn WRITE_PORT_USHORT(Port: *mut USHORT, Value: USHORT);
    fn USBPORT_RegisterUSBPortDriver( DriverObject: PDRIVER_OBJECT, Version: ULONG, RegPacket: *mut USBPORT_REGISTRATION_PACKET, ) -> NTSTATUS;
    fn RtlAssert( VoidFailedAssertion: *mut core::ffi::c_void, VoidFileName: *mut core::ffi::c_void, LineNumber: u32, MutableMessage: *mut i8, );
    fn WRITE_PORT_ULONG(Register: PULONG, Value: ULONG);
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

const UHCI_FRNUM_FRAME_MASK: ULONG = 0x07FF;
const UHCI_FRNUM_OVERFLOW_LIST: ULONG = 0x0400;
const UHCI_FRAME_LIST_MAX_ENTRIES: ULONG = 1024;

const MP_STATUS_SUCCESS: MPSTATUS = 0;

#[no_mangle]
pub unsafe extern "system" fn UhciDumpHcdQH(mut QH: PUHCI_HCD_QH) {
}

#[no_mangle]
pub unsafe extern "system" fn UhciDumpHcdTD(TD: PUHCI_HCD_TD) {
    if (*TD).HwTD.Buffer != 0 {
    }

    if (*TD).SetupPacket.bmRequestType.B != 0 {
    }
    if (*TD).SetupPacket.bRequest != 0 {
    }
    if (*TD).SetupPacket.wValue.W != 0 {
    }
    if (*TD).SetupPacket.wIndex.W != 0 {
    }
    if (*TD).SetupPacket.wLength != 0 {
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciFixDataToggle(
    mut UhciExtension: PUHCI_EXTENSION,
    mut UhciEndpoint: PUHCI_ENDPOINT,
    mut TD: PUHCI_HCD_TD,
    mut DataToggle: BOOL,
) {
    DPRINT_UHCI!(
        b"UhciFixDataToggle: UhciExtension - %p, UhciEndpoint - %p, DataToggle - %X\n\0".as_ptr()
            as *const i8,
        UhciExtension,
        UhciEndpoint,
        DataToggle,
    );

    while !TD.is_null() {
        (*TD)
            .HwTD
            .Token
            .c2rust_unnamed
            .set_DataToggle(((*TD).HwTD.Token.c2rust_unnamed.DataToggle() == 0) as ULONG);
        DataToggle = (DataToggle == 0) as BOOL;
        TD = (*TD).NextHcdTD as PUHCI_HCD_TD;
    }

    (*UhciEndpoint).DataToggle = DataToggle;
}

#[no_mangle]
pub unsafe extern "system" fn UhciCleanupFrameListEntry(
    UhciExtension: PUHCI_EXTENSION,
    Index: ULONG,
) {
    let mut UhciResources: PUHCI_HC_RESOURCES;
    let mut PhysicalAddress: ULONG;
    let mut HeadIdx: ULONG;

    UhciResources = (*UhciExtension).HcResourcesVA;

    if Index == 0 {
        PhysicalAddress = (*(*UhciExtension).StaticTD).PhysicalAddress;

        (*UhciResources).FrameList[0] = PhysicalAddress | UHCI_FRAME_LIST_POINTER_TD;
    } else {
        HeadIdx = ((INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) as ULONG)
            .wrapping_add(Index & (ENDPOINT_INTERRUPT_32ms - 1) as ULONG);

        PhysicalAddress = (*(*UhciExtension).IntQH[HeadIdx as usize]).PhysicalAddress;

        (*UhciResources).FrameList[Index as usize] = PhysicalAddress | UHCI_FRAME_LIST_POINTER_QH;
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciCleanupFrameList(
    UhciExtension: PUHCI_EXTENSION,
    IsAllEntries: BOOLEAN,
) {
    let mut NewFrameNumber: ULONG;
    let OldFrameNumber: ULONG;
    let mut ix: ULONG;

    DPRINT_UHCI!(
        b"UhciCleanupFrameList: [%p] All - %x\n\0".as_ptr() as *const i8,
        UhciExtension,
        IsAllEntries as core::ffi::c_int,
    );

    // FIXME: using UhciExtension->LockFrameList after supporting ISOs.

    NewFrameNumber = UhciGet32BitFrameNumber(UhciExtension as PVOID);
    OldFrameNumber = (*UhciExtension).FrameNumber;

    if NewFrameNumber.wrapping_sub(OldFrameNumber) < UHCI_FRAME_LIST_MAX_ENTRIES as ULONG
        && IsAllEntries as i32 == (FALSE as i32)
    {
        ix = OldFrameNumber & UHCI_FRAME_LIST_INDEX_MASK as ULONG;
        while ix != (NewFrameNumber & UHCI_FRAME_LIST_INDEX_MASK as ULONG) {
            UhciCleanupFrameListEntry(UhciExtension, ix);
            ix = (ix.wrapping_add(1)) & UHCI_FRAME_LIST_INDEX_MASK as ULONG;
        }
    } else {
        ix = 0 as ULONG;
        while ix < UHCI_FRAME_LIST_MAX_ENTRIES as ULONG {
            UhciCleanupFrameListEntry(UhciExtension, ix);
            ix = ix.wrapping_add(1);
        }
    }

    (*UhciExtension).FrameNumber = NewFrameNumber;
}

#[no_mangle]
pub unsafe extern "system" fn UhciUpdateCounter(UhciExtension: PUHCI_EXTENSION) {
    let mut FrameNumber: ULONG;

    FrameNumber = READ_PORT_USHORT(core::ptr::addr_of_mut!((*(*UhciExtension).BaseRegister).FrameNumber)) as ULONG;
    FrameNumber &= UHCI_FRNUM_FRAME_MASK;

    if (FrameNumber ^ (*UhciExtension).FrameHighPart) & UHCI_FRNUM_OVERFLOW_LIST != 0 {
        (*UhciExtension).FrameHighPart = ((*UhciExtension).FrameHighPart).wrapping_add(UHCI_FRAME_LIST_MAX_ENTRIES);

        // DPRINT_UHCI!("UhciUpdateCounter: UhciExtension->FrameHighPart - %lX\n", UhciExtension->FrameHighPart);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciSetNextQH(QH: PUHCI_HCD_QH, NextQH: PUHCI_HCD_QH) {
    DPRINT_UHCI!(b"UhciSetNextQH: QH - %p, NextQH - %p\n\0".as_ptr() as *const i8, QH, NextQH);

    (*QH).HwQH.NextQH = ((*NextQH).PhysicalAddress | UHCI_QH_HEAD_LINK_PTR_QH) as ULONG;
    (*QH).NextHcdQH = NextQH;

    (*NextQH).PrevHcdQH = QH;
    (*NextQH).QhFlags |= UHCI_HCD_QH_FLAG_ACTIVE as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn UhciOpenEndpoint(
    uhciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    uhciEndpoint: PVOID,
) -> MPSTATUS {
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let TransferType: ULONG;
    let mut BufferVA: ULONG_PTR;
    let mut BufferPA: ULONG;
    let mut ix: ULONG;
    let TdCount: ULONG;
    let mut TD: PUHCI_HCD_TD;
    let mut BufferLength: SIZE_T;
    let QH: PUHCI_HCD_QH;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*UhciEndpoint).EndpointProperties) as *mut core::ffi::c_void,
        EndpointProperties as *const core::ffi::c_void,
        core::mem::size_of::<USBPORT_ENDPOINT_PROPERTIES>() as SIZE_T,
    );

    InitializeListHead(core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs));

    (*UhciEndpoint).EndpointLock = 0;
    (*UhciEndpoint).DataToggle = UHCI_TD_PID_DATA0 as BOOL;
    (*UhciEndpoint).Flags = 0;

    TransferType = (*EndpointProperties).TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG
    {
        (*UhciEndpoint).Flags |= UHCI_ENDPOINT_FLAG_CONTROL_OR_ISO as ULONG;
    }

    BufferVA = ((*EndpointProperties).BufferVA as usize);
    BufferPA = (*EndpointProperties).BufferPA;

    BufferLength = (*EndpointProperties).BufferLength as SIZE_T;

    if (*EndpointProperties).TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        (*UhciEndpoint).QH = BufferVA as PUHCI_HCD_QH;

        QH = (*UhciEndpoint).QH;

        (*QH).HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
        (*QH).PhysicalAddress = BufferPA;

        (*QH).NextHcdQH = QH;
        (*QH).PrevHcdQH = QH;
        (*QH).UhciEndpoint = UhciEndpoint;

        BufferVA = BufferVA.wrapping_add(core::mem::size_of::<UHCI_HCD_QH>() as ULONG_PTR);
        BufferPA = BufferPA.wrapping_add(core::mem::size_of::<UHCI_HCD_QH>() as ULONG);

        BufferLength = BufferLength.wrapping_sub(core::mem::size_of::<UHCI_HCD_QH>() as _);
    }

    TdCount = (BufferLength as usize).wrapping_div(core::mem::size_of::<UHCI_HCD_TD>()) as ULONG;
    (*UhciEndpoint).MaxTDs = TdCount;

    (*UhciEndpoint).FirstTD = BufferVA as PUHCI_HCD_TD;
    (*UhciEndpoint).AllocatedTDs = 0;

    RtlZeroMemory(
        (*UhciEndpoint).FirstTD as *mut core::ffi::c_void,
        (TdCount as usize).wrapping_mul(core::mem::size_of::<UHCI_HCD_TD>()),
    );

    ix = 0;
    while ix < (*UhciEndpoint).MaxTDs {
        TD = (*UhciEndpoint).FirstTD.offset(ix as isize);
        (*TD).PhysicalAddress = BufferPA;
        BufferPA = BufferPA.wrapping_add(core::mem::size_of::<UHCI_HCD_TD>() as ULONG);
        ix = ix.wrapping_add(1);
    }

    (*UhciEndpoint).TailTD = core::ptr::null_mut();
    (*UhciEndpoint).HeadTD = core::ptr::null_mut();

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciReopenEndpoint(
    uhciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    uhciEndpoint: PVOID,
) -> MPSTATUS {
    DPRINT_IMPL!(b"Uhci: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciQueryEndpointRequirements(
    uhciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EndpointRequirements: PUSBPORT_ENDPOINT_REQUIREMENTS,
) {
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0;
    const USBPORT_TRANSFER_TYPE_CONTROL: ULONG = 1;
    const USBPORT_TRANSFER_TYPE_BULK: ULONG = 2;
    const USBPORT_TRANSFER_TYPE_INTERRUPT: ULONG = 3;

    let mut TransferType: ULONG;
    let mut TdCount: ULONG;

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
            TdCount = (2 * UHCI_MAX_ISO_TD_COUNT) as ULONG;

            (*EndpointRequirements).HeaderBufferSize = (0usize).wrapping_add(
                (TdCount as usize).wrapping_mul(core::mem::size_of::<UHCI_HCD_TD>()),
            ) as ULONG;

            (*EndpointRequirements).MaxTransferSize = UHCI_MAX_ISO_TRANSFER_SIZE as ULONG;
        }

        USBPORT_TRANSFER_TYPE_CONTROL => {
            TdCount = ((*EndpointProperties).MaxTransferSize as core::ffi::c_ulong)
                .wrapping_div((*EndpointProperties).TotalMaxPacketSize as core::ffi::c_ulong)
                as ULONG;

            TdCount = (TdCount as core::ffi::c_ulong).wrapping_add(2 as core::ffi::c_ulong) as ULONG;

            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<UHCI_HCD_QH>()).wrapping_add(
                    (TdCount as usize).wrapping_mul(core::mem::size_of::<UHCI_HCD_TD>()),
                ) as ULONG;

            (*EndpointRequirements).MaxTransferSize = (*EndpointProperties).MaxTransferSize;
        }

        USBPORT_TRANSFER_TYPE_BULK => {
            TdCount = (2 * UHCI_MAX_BULK_TRANSFER_SIZE
                / ((*EndpointProperties).TotalMaxPacketSize as core::ffi::c_int as u32))
                as ULONG;

            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<UHCI_HCD_QH>()).wrapping_add(
                    (TdCount as usize).wrapping_mul(core::mem::size_of::<UHCI_HCD_TD>()),
                ) as ULONG;

            (*EndpointRequirements).MaxTransferSize = UHCI_MAX_BULK_TRANSFER_SIZE as ULONG;
        }

        USBPORT_TRANSFER_TYPE_INTERRUPT => {
            TdCount = (2 * UHCI_MAX_INTERRUPT_TD_COUNT) as ULONG;

            (*EndpointRequirements).HeaderBufferSize =
                (core::mem::size_of::<UHCI_HCD_QH>()).wrapping_add(
                    (TdCount as usize).wrapping_mul(core::mem::size_of::<UHCI_HCD_TD>()),
                ) as ULONG;

            (*EndpointRequirements).MaxTransferSize = (UHCI_MAX_INTERRUPT_TD_COUNT
                * ((*EndpointProperties).TotalMaxPacketSize as core::ffi::c_int as u32))
                as ULONG;
        }

        _ => {
            if DbgPrint(
                b"(%s:%d) UhciQueryEndpointRequirements: Unknown TransferType - %x\n\0" as *const u8
                    as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const core::ffi::c_char,
                343 as core::ffi::c_int,
                TransferType,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"usbuhci.c\0" as *const u8 as *const core::ffi::c_char,
                    343 as core::ffi::c_int,
                );
            }
            DbgBreakPoint();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciCloseEndpoint(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    IsDoDisablePeriodic: BOOLEAN,
) {
    DPRINT_IMPL!(b"UhciCloseEndpoint: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciTakeControlHC(
    UhciExtension: PUHCI_EXTENSION,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {

    const USBPORT_RESOURCES_PORT: u32 = 0x01;
    const USBPORT_RESOURCES_INTERRUPT: u32 = 0x02;
    const PCI_LEGSUP: u32 = 0xC0;
    const UHCI_USB_STATUS_MASK: u32 = 0x003F;

    let mut EndTime: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let ResourcesTypes: ULONG;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let StatusRegister: PUSHORT;
    let mut LegacySupport: UHCI_PCI_LEGSUP = core::mem::zeroed();
    let mut LegacyMask: UHCI_PCI_LEGSUP = core::mem::zeroed();
    let mut Command: UHCI_USB_COMMAND = core::mem::zeroed();
    let mut HcStatus: UHCI_USB_STATUS = core::mem::zeroed();
    let mut MpStatus: MPSTATUS = MP_STATUS_SUCCESS as MPSTATUS;

    ResourcesTypes = (*Resources).ResourcesTypes;

    if (ResourcesTypes & (USBPORT_RESOURCES_PORT | USBPORT_RESOURCES_INTERRUPT))
        != (USBPORT_RESOURCES_PORT | USBPORT_RESOURCES_INTERRUPT)
    {
        MpStatus = MP_STATUS_ERROR as MPSTATUS;
    }

    BaseRegister = (*UhciExtension).BaseRegister;
    StatusRegister = core::ptr::addr_of_mut!((*BaseRegister).HcStatus.AsUSHORT) as PUSHORT;

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        UhciExtension as PVOID,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
        PCI_LEGSUP as ULONG,
        core::mem::size_of::<USHORT>() as ULONG,
    );

    UhciDisableInterrupts(UhciExtension as PVOID);

    Command.AsUSHORT = READ_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT));

    Command.set_Run(0);
    Command.set_GlobalReset(0);
    Command.set_ConfigureFlag(0);

    WRITE_PORT_USHORT(
        core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT),
        Command.AsUSHORT,
    );

    KeQuerySystemTime(core::ptr::addr_of_mut!(EndTime));
    EndTime.QuadPart += (100 * 10000) as i64; // 100 ms

    HcStatus.AsUSHORT = READ_PORT_USHORT(StatusRegister);

    while HcStatus.HcHalted() == 0 {
        HcStatus.AsUSHORT = READ_PORT_USHORT(StatusRegister);
        KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

        if CurrentTime.QuadPart >= EndTime.QuadPart {
            break;
        }
    }

    WRITE_PORT_USHORT(StatusRegister, UHCI_USB_STATUS_MASK as USHORT);

    LegacyMask.AsUSHORT = 0;
    LegacyMask.set_Smi60Read(1);
    LegacyMask.set_Smi60Write(1);
    LegacyMask.set_Smi64Read(1);
    LegacyMask.set_Smi64Write(1);
    LegacyMask.set_SmiIrq(1);
    LegacyMask.set_A20Gate(1);
    LegacyMask.set_SmiEndPassThrough(1);

    if (LegacySupport.AsUSHORT & LegacyMask.AsUSHORT) != 0 {
        (*Resources).LegacySupport = 1 as UCHAR;

        (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
            UhciExtension as PVOID,
            TRUE as BOOLEAN,
            core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
            PCI_LEGSUP as ULONG,
            core::mem::size_of::<USHORT>() as ULONG,
        );

        LegacySupport.AsUSHORT = 0;

        (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
            UhciExtension as PVOID,
            FALSE as BOOLEAN,
            core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
            PCI_LEGSUP as ULONG,
            core::mem::size_of::<USHORT>() as ULONG,
        );
    }

    return MpStatus;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInitializeHardware(UhciExtension: PUHCI_EXTENSION) -> MPSTATUS {

    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut Command: UHCI_USB_COMMAND = core::mem::zeroed();
    let mut StatusMask: UHCI_USB_STATUS;

    BaseRegister = (*UhciExtension).BaseRegister;

    /* Save SOF Timing Value */
    (*UhciExtension).SOF_Modify = READ_PORT_UCHAR(core::ptr::addr_of_mut!((*BaseRegister).SOF_Modify));

    (RegPacket.UsbPortWait.expect("non-null function pointer"))(UhciExtension as PVOID, 20 as ULONG);

    Command.AsUSHORT = READ_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT));

    /* Global Reset */
    Command.AsUSHORT = 0;
    Command.set_GlobalReset(1);
    WRITE_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT), Command.AsUSHORT);

    (RegPacket.UsbPortWait.expect("non-null function pointer"))(UhciExtension as PVOID, 20 as ULONG);

    Command.AsUSHORT = 0;
    WRITE_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT), Command.AsUSHORT);

    /* Set MaxPacket for full speed bandwidth reclamation */
    Command.AsUSHORT = 0;
    Command.set_MaxPacket(1);
    WRITE_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT), Command.AsUSHORT);

    /* Restore SOF Timing Value */
    WRITE_PORT_UCHAR(core::ptr::addr_of_mut!((*BaseRegister).SOF_Modify), (*UhciExtension).SOF_Modify);

    StatusMask = (*UhciExtension).StatusMask;

    StatusMask.set_Interrupt(1);
    StatusMask.set_ErrorInterrupt(1);
    StatusMask.set_ResumeDetect(1);
    StatusMask.set_HostSystemError(1);

    (*UhciExtension).StatusMask = StatusMask;

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInitializeSchedule(
    UhciExtension: PUHCI_EXTENSION,
    HcResourcesVA: PUHCI_HC_RESOURCES,
    hcResourcesPA: ULONG,
) -> MPSTATUS {
    let mut IntQH: PUHCI_HCD_QH;
    let mut IntQhPA: ULONG;
    let StaticControlHead: PUHCI_HCD_QH;
    let StaticControlHeadPA: ULONG;
    let StaticBulkHead: PUHCI_HCD_QH;
    let StaticBulkHeadPA: ULONG;
    let StaticBulkTD: PUHCI_HCD_TD;
    let StaticBulkTdPA: ULONG;
    let StaticTD: PUHCI_HCD_TD;
    let StaticTdPA: ULONG;
    let mut StaticSofTD: PUHCI_HCD_TD;
    let mut StaticSofTdPA: ULONG;
    let mut PhysicalAddress: ULONG;
    let mut Idx: ULONG;
    let mut HeadIdx: ULONG;
    let mut FrameIdx: UCHAR;

    /* Build structure (tree) of static QHs
       for interrupt and isochronous transfers */
    FrameIdx = 0;
    while (FrameIdx as i32) < (INTERRUPT_ENDPOINTs as i32) {
        IntQH = (core::ptr::addr_of_mut!((*HcResourcesVA).StaticIntHead) as *mut UHCI_HCD_QH)
            .offset(FrameIdx as isize) as PUHCI_HCD_QH;
        IntQhPA = hcResourcesPA
            .wrapping_add(
                core::mem::offset_of!(UHCI_HC_RESOURCES, StaticIntHead) as ULONG
                    + (FrameIdx as ULONG) * (core::mem::size_of::<UHCI_HCD_QH>() as ULONG),
            );

        RtlZeroMemory(IntQH as *mut core::ffi::c_void, core::mem::size_of::<UHCI_HCD_QH>() as usize);

        (*IntQH).HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
        (*IntQH).PhysicalAddress = IntQhPA;

        (*UhciExtension).IntQH[FrameIdx as usize] = IntQH;

        if FrameIdx as i32 == 0 {
            UhciSetNextQH(IntQH, (*UhciExtension).IntQH[0]);
        } else {
            UhciSetNextQH(
                IntQH,
                (*UhciExtension).IntQH[((FrameIdx as i32 - 1) / 2) as usize],
            );
        }

        FrameIdx = FrameIdx.wrapping_add(1);
    }

    /* Initialize static QH for control transfers */
    StaticControlHead = core::ptr::addr_of_mut!((*HcResourcesVA).StaticControlHead) as PUHCI_HCD_QH;
    StaticControlHeadPA = hcResourcesPA
        .wrapping_add(core::mem::offset_of!(UHCI_HC_RESOURCES, StaticControlHead) as ULONG);

    RtlZeroMemory(
        StaticControlHead as *mut core::ffi::c_void,
        core::mem::size_of::<UHCI_HCD_QH>() as usize,
    );

    (*StaticControlHead).HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
    (*StaticControlHead).PhysicalAddress = StaticControlHeadPA;

    UhciSetNextQH((*UhciExtension).IntQH[0], StaticControlHead);

    (*UhciExtension).ControlQH = StaticControlHead;

    /* Initialize static QH for bulk transfers */
    StaticBulkHead = core::ptr::addr_of_mut!((*HcResourcesVA).StaticBulkHead) as PUHCI_HCD_QH;
    StaticBulkHeadPA = hcResourcesPA
        .wrapping_add(core::mem::offset_of!(UHCI_HC_RESOURCES, StaticBulkHead) as ULONG);

    RtlZeroMemory(
        StaticBulkHead as *mut core::ffi::c_void,
        core::mem::size_of::<UHCI_HCD_QH>() as usize,
    );

    (*StaticBulkHead).PhysicalAddress = StaticBulkHeadPA;
    PhysicalAddress = StaticBulkHeadPA | UHCI_QH_ELEMENT_LINK_PTR_QH as ULONG;
    PhysicalAddress |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
    (*StaticBulkHead).HwQH.NextQH = PhysicalAddress;

    UhciSetNextQH(StaticControlHead, StaticBulkHead);

    (*UhciExtension).BulkQH = StaticBulkHead;
    (*UhciExtension).BulkTailQH = StaticBulkHead;

    /* Initialize static TD for bulk transfers */
    StaticBulkTD = core::ptr::addr_of_mut!((*HcResourcesVA).StaticBulkTD) as PUHCI_HCD_TD;
    StaticBulkTdPA = hcResourcesPA
        .wrapping_add(core::mem::offset_of!(UHCI_HC_RESOURCES, StaticBulkTD) as ULONG);

    (*StaticBulkTD).HwTD.NextElement = StaticBulkTdPA | UHCI_TD_LINK_PTR_TD as ULONG;

    (*StaticBulkTD).HwTD.ControlStatus.AsULONG = 0;
    (*StaticBulkTD).HwTD.ControlStatus.set_IsochronousType(1);

    (*StaticBulkTD).HwTD.Token.AsULONG = 0;
    (*StaticBulkTD).HwTD.Token.set_Endpoint(1);
    (*StaticBulkTD).HwTD.Token.set_MaximumLength(UHCI_TD_LENGTH_NULL as ULONG);
    (*StaticBulkTD).HwTD.Token.set_PIDCode(UHCI_TD_PID_OUT as ULONG);

    (*StaticBulkTD).HwTD.Buffer = 0;

    (*StaticBulkTD).PhysicalAddress = StaticBulkTdPA;
    (*StaticBulkTD).NextHcdTD = core::ptr::null_mut();
    (*StaticBulkTD).Flags = UHCI_HCD_TD_FLAG_PROCESSED as ULONG;

    PhysicalAddress = StaticBulkTdPA | UHCI_QH_ELEMENT_LINK_PTR_TD as ULONG;
    (*(*UhciExtension).BulkQH).HwQH.NextElement = PhysicalAddress;

    /* Set Frame List pointers */
    Idx = 0;
    while Idx < UHCI_FRAME_LIST_MAX_ENTRIES as ULONG {
        HeadIdx = ((INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) as ULONG)
            .wrapping_add(Idx & (ENDPOINT_INTERRUPT_32ms - 1) as ULONG);

        PhysicalAddress = (*(*UhciExtension).IntQH[HeadIdx as usize]).PhysicalAddress;
        PhysicalAddress |= UHCI_FRAME_LIST_POINTER_QH as ULONG;
        (*HcResourcesVA).FrameList[Idx as usize] = PhysicalAddress;

        Idx = Idx.wrapping_add(1);
    }

    /* Initialize static TD for first frame */
    StaticTD = core::ptr::addr_of_mut!((*HcResourcesVA).StaticTD) as PUHCI_HCD_TD;
    StaticTdPA = hcResourcesPA
        .wrapping_add(core::mem::offset_of!(UHCI_HC_RESOURCES, StaticTD) as ULONG);

    RtlZeroMemory(
        StaticTD as *mut core::ffi::c_void,
        core::mem::size_of::<UHCI_HCD_TD>() as usize,
    );

    (*StaticTD).PhysicalAddress = StaticTdPA;

    HeadIdx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) as ULONG;
    PhysicalAddress = (*(*UhciExtension).IntQH[HeadIdx as usize]).PhysicalAddress;
    (*StaticTD).HwTD.NextElement = PhysicalAddress | UHCI_TD_LINK_PTR_QH as ULONG;

    (*StaticTD).HwTD.ControlStatus.set_InterruptOnComplete(1);
    (*StaticTD).HwTD.Token.set_PIDCode(UHCI_TD_PID_IN as ULONG);

    (*UhciExtension).StaticTD = StaticTD;

    /* Initialize StaticSofTDs for UhciInterruptNextSOF() */
    (*UhciExtension).SOF_HcdTDs = (core::ptr::addr_of_mut!((*HcResourcesVA).StaticSofTD)
        as *mut UHCI_HCD_TD)
        .offset(0) as PUHCI_HCD_TD;
    StaticSofTdPA = hcResourcesPA
        .wrapping_add(core::mem::offset_of!(UHCI_HC_RESOURCES, StaticSofTD) as ULONG);

    Idx = 0;
    while Idx < UHCI_MAX_STATIC_SOF_TDS as ULONG {
        StaticSofTD = (*UhciExtension).SOF_HcdTDs.offset(Idx as isize);

        RtlZeroMemory(
            StaticSofTD as *mut core::ffi::c_void,
            core::mem::size_of::<UHCI_HCD_TD>() as usize,
        );

        PhysicalAddress = (*(*UhciExtension).IntQH[HeadIdx as usize]).PhysicalAddress;
        (*StaticSofTD).HwTD.NextElement = PhysicalAddress | UHCI_TD_LINK_PTR_QH as ULONG;

        (*StaticSofTD).HwTD.ControlStatus.set_InterruptOnComplete(1);
        (*StaticSofTD).PhysicalAddress = StaticSofTdPA;

        StaticSofTdPA = StaticSofTdPA.wrapping_add(core::mem::size_of::<UHCI_HCD_TD>() as ULONG);

        Idx = Idx.wrapping_add(1);
    }

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciStartController(
    uhciExtension: PVOID,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let mut MpStatus: MPSTATUS;
    let mut PortControlRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut Command: UHCI_USB_COMMAND = core::mem::zeroed();
    let mut Port: USHORT;

    (*UhciExtension).Flags &= !(UHCI_EXTENSION_FLAG_SUSPENDED as ULONG);
    (*UhciExtension).BaseRegister = (*Resources).ResourceBase as PUHCI_HW_REGISTERS;
    BaseRegister = (*UhciExtension).BaseRegister;

    (*UhciExtension).HcFlavor = (*Resources).HcFlavor;

    MpStatus = UhciTakeControlHC(UhciExtension, Resources);

    if MpStatus == MP_STATUS_SUCCESS {
        MpStatus = UhciInitializeHardware(UhciExtension);

        if MpStatus == MP_STATUS_SUCCESS {
            (*UhciExtension).HcResourcesVA = (*Resources).StartVA as PUHCI_HC_RESOURCES;
            (*UhciExtension).HcResourcesPA = (*Resources).StartPA;

            MpStatus = UhciInitializeSchedule(
                UhciExtension,
                (*UhciExtension).HcResourcesVA,
                (*UhciExtension).HcResourcesPA,
            );

            (*UhciExtension).LockFrameList = 0;
        }
    }

    WRITE_PORT_ULONG(
        core::ptr::addr_of_mut!((*BaseRegister).FrameAddress),
        (*UhciExtension).HcResourcesPA
            + core::mem::offset_of!(UHCI_HC_RESOURCES, FrameList) as ULONG,
    );

    if MpStatus == MP_STATUS_SUCCESS {
        Command.AsUSHORT = READ_PORT_USHORT(core::ptr::addr_of_mut!(
            (*BaseRegister).HcCommand.AsUSHORT
        ));
        Command.set_Run(1);
        WRITE_PORT_USHORT(
            core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT),
            Command.AsUSHORT,
        );

        Port = 0;
        while (Port as core::ffi::c_int) < (UHCI_NUM_ROOT_HUB_PORTS as i32) {
            let pc = (core::ptr::addr_of_mut!((*BaseRegister).PortControl)
                as *mut UHCI_PORT_STATUS_CONTROL)
                .offset(Port as isize);
            PortControlRegister = core::ptr::addr_of_mut!((*pc).AsUSHORT) as PUSHORT;
            PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);

            PortControl.set_ConnectStatusChange(0);
            PortControl.set_Suspend(0);

            WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);

            Port = Port.wrapping_add(1);
        }

        (*(*UhciExtension).HcResourcesVA).FrameList[0] =
            (*(*UhciExtension).StaticTD).PhysicalAddress;
    }

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciStopController(
    uhciExtension: PVOID,
    IsDoDisableInterrupts: BOOLEAN,
) {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let CommandReg: PUSHORT;
    let mut Command: UHCI_USB_COMMAND = core::mem::zeroed();
    let mut EndTime: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();

    BaseRegister = (*UhciExtension).BaseRegister;
    CommandReg = core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT) as PUSHORT;

    Command.AsUSHORT = READ_PORT_USHORT(CommandReg);

    if Command.AsUSHORT == 0xFFFF {
        return;
    }

    if Command.GlobalReset() != 0 {
        Command.set_GlobalReset(0);
        WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);
    }

    Command.set_HcReset(1);

    WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);

    // KeQuerySystemTime(&EndTime)
    *(core::ptr::addr_of_mut!(EndTime) as *mut u64) = *(SharedSystemTimePtr() as *mut u64);
    EndTime.QuadPart += 100 * 1000;

    loop {
        Command.AsUSHORT = READ_PORT_USHORT(CommandReg);
        if !(Command.HcReset() == 1) {
            break;
        }

        // KeQuerySystemTime(&CurrentTime)
        *(core::ptr::addr_of_mut!(CurrentTime) as *mut u64) = *(SharedSystemTimePtr() as *mut u64);

        if CurrentTime.QuadPart >= EndTime.QuadPart {
            DbgBreakPoint();
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciSuspendController(uhciExtension: PVOID) {
    DPRINT_IMPL!(b"UhciSuspendController: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciResumeController(uhciExtension: PVOID) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciResumeController: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciHardwarePresent(UhciExtension: PUHCI_EXTENSION) -> BOOLEAN {

    let mut UhciStatus: UHCI_USB_STATUS = core::mem::zeroed();
    let mut StatusReg: PUSHORT;

    StatusReg = core::ptr::addr_of_mut!((*(*UhciExtension).BaseRegister).HcStatus.AsUSHORT) as PUSHORT;
    UhciStatus.AsUSHORT = READ_PORT_USHORT(StatusReg);

    if UhciStatus.AsUSHORT as i32 == MAXUSHORT {
        // DPRINT_UHCI!("UhciHardwarePresent: HW not present\n");
    }

    return (UhciStatus.AsUSHORT as i32 != MAXUSHORT) as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInterruptService(uhciExtension: PVOID) -> BOOLEAN {

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let BaseRegister: PUHCI_HW_REGISTERS;
    let CommandReg: PUSHORT;
    let mut Command: UHCI_USB_COMMAND = core::mem::zeroed();
    let StatusReg: PUSHORT;
    let mut HcStatus: UHCI_USB_STATUS = core::mem::zeroed();
    let InterruptEnableReg: PUSHORT;
    let mut QH: PUHCI_HCD_QH;
    let BulkTailQH: PUHCI_HCD_QH;
    let ScheduleError: ULONG;
    let mut Result: BOOLEAN = 0;

    BaseRegister = (*UhciExtension).BaseRegister;
    StatusReg = core::ptr::addr_of_mut!((*BaseRegister).HcStatus.AsUSHORT) as PUSHORT;
    InterruptEnableReg = core::ptr::addr_of_mut!((*BaseRegister).HcInterruptEnable.AsUSHORT) as PUSHORT;
    CommandReg = core::ptr::addr_of_mut!((*BaseRegister).HcCommand.AsUSHORT) as PUSHORT;

    if UhciHardwarePresent(UhciExtension) == 0 {
        DbgPrint(b"UhciInterruptService: return FALSE\n\0".as_ptr() as PCSTR);
        return 0;
    }

    HcStatus.AsUSHORT = (READ_PORT_USHORT(StatusReg) as i32 & (UHCI_USB_STATUS_MASK as i32)) as USHORT;

    if HcStatus.HostSystemError() != 0 || HcStatus.HcProcessError() != 0 {
        DbgPrint(b"UhciInterruptService: Error [%p] HcStatus %X\n\0".as_ptr() as PCSTR, UhciExtension as *mut c_void, HcStatus.AsUSHORT as ULONG);
    } else if HcStatus.AsUSHORT != 0 {
        (*UhciExtension).HcScheduleError = 0;
    }

    if HcStatus.HcProcessError() != 0 {
        let fn_0: USHORT = READ_PORT_USHORT(core::ptr::addr_of_mut!((*BaseRegister).FrameNumber));
        let intr: USHORT = READ_PORT_USHORT(InterruptEnableReg);
        let cmd: USHORT = READ_PORT_USHORT(CommandReg);

        DbgPrint(b"UhciInterruptService: HC ProcessError!\n\0".as_ptr() as PCSTR);
        DbgPrint(b"UhciExtension %p, frame %X\n\0".as_ptr() as PCSTR, UhciExtension as *mut c_void, fn_0 as ULONG);
        DbgPrint(b"HcCommand %X\n\0".as_ptr() as PCSTR, cmd as ULONG);
        DbgPrint(b"HcStatus %X\n\0".as_ptr() as PCSTR, HcStatus.AsUSHORT as ULONG);
        DbgPrint(b"HcInterruptEnable %X\n\0".as_ptr() as PCSTR, intr as ULONG);

        DbgBreakPoint();
    }

    if HcStatus.HcHalted() != 0 {
        DPRINT_UHCI!(
            b"UhciInterruptService: Hc Halted [%p] HcStatus %X\n\0".as_ptr() as *const i8,
            UhciExtension,
            HcStatus.AsUSHORT as i32
        );
    } else if HcStatus.AsUSHORT != 0 {
        (*UhciExtension).HcStatus.AsUSHORT = HcStatus.AsUSHORT;

        WRITE_PORT_USHORT(StatusReg, HcStatus.AsUSHORT);
        WRITE_PORT_USHORT(InterruptEnableReg, 0);

        if HcStatus.HostSystemError() != 0 {
            DbgPrint(b"UhciInterruptService: HostSystemError! HcStatus - %X\n\0".as_ptr() as PCSTR, HcStatus.AsUSHORT as ULONG);

            DbgBreakPoint();
        }

        Result = 1;
    }

    'NextProcess: loop {
        if HcStatus.Interrupt() == 0 {
            break 'NextProcess;
        }

        UhciUpdateCounter(UhciExtension);

        BulkTailQH = (*UhciExtension).BulkTailQH;

        if (*BulkTailQH).HwQH.NextQH as core::ffi::c_ulong
            & UHCI_QH_HEAD_LINK_PTR_TERMINATE as core::ffi::c_ulong
            != 0
        {
            break 'NextProcess;
        }

        QH = (*UhciExtension).BulkQH;

        loop {
            QH = (*QH).NextHcdQH as PUHCI_HCD_QH;

            if QH.is_null() {
                (*BulkTailQH).HwQH.NextQH |= UHCI_QH_HEAD_LINK_PTR_TERMINATE as core::ffi::c_ulong;
                break 'NextProcess;
            }

            if !((*QH).HwQH.NextElement as core::ffi::c_ulong
                & UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as core::ffi::c_ulong
                != 0)
            {
                break;
            }
        }

        break 'NextProcess;
    }

    if HcStatus.HcProcessError() != 0 {
        UhciCleanupFrameList(UhciExtension, 1);

        ScheduleError = (*UhciExtension).HcScheduleError;
        (*UhciExtension).HcScheduleError = (ScheduleError as core::ffi::c_ulong)
            .wrapping_add(1) as ULONG;

        DbgPrint(b"UhciInterruptService: [%p] ScheduleError %X\n\0".as_ptr() as PCSTR, UhciExtension as *mut c_void, ScheduleError as ULONG);

        if ScheduleError < UHCI_MAX_HC_SCHEDULE_ERRORS as core::ffi::c_ulong {
            Command.AsUSHORT = READ_PORT_USHORT(CommandReg);
            Command.set_Run(1);
            WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);
        }
    } else if HcStatus.Interrupt() != 0 && (*UhciExtension).ExtensionLock != 0 {
        DbgPrint(b"UhciInterruptService: [%p] HcStatus %X\n\0".as_ptr() as PCSTR, UhciExtension as *mut c_void, HcStatus.AsUSHORT as ULONG);

        UhciCleanupFrameList(UhciExtension, 0);
    }

    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInterruptDpc(
    uhciExtension: PVOID,
    IsDoEnableInterrupts: BOOLEAN,
) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let HcStatus: UHCI_USB_STATUS;

    DPRINT_UHCI!(
        b"UhciInterruptDpc: [%p] EnableInt %x, HcStatus %X\n\0".as_ptr() as *const i8,
        uhciExtension,
        IsDoEnableInterrupts as core::ffi::c_int,
        (*UhciExtension).HcStatus,
    );

    BaseRegister = (*UhciExtension).BaseRegister;

    HcStatus = (*UhciExtension).HcStatus;
    (*UhciExtension).HcStatus.AsUSHORT = 0 as USHORT;

    if (HcStatus.Interrupt() as i32 | HcStatus.ErrorInterrupt() as i32) != 0 {
        (RegPacket.UsbPortInvalidateEndpoint.expect("non-null function pointer"))(
            UhciExtension as PVOID,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    if IsDoEnableInterrupts != 0 {
        WRITE_PORT_USHORT(
            core::ptr::addr_of_mut!((*BaseRegister).HcInterruptEnable.AsUSHORT),
            (*UhciExtension).StatusMask.AsUSHORT,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciQueueTransfer(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
    FirstTD: PUHCI_HCD_TD,
    LastTD: PUHCI_HCD_TD,
) {
    let mut QH: PUHCI_HCD_QH;
    let BulkTailQH: PUHCI_HCD_QH;
    let TailTD: PUHCI_HCD_TD;
    let mut PhysicalAddress: ULONG;

    TailTD = (*UhciEndpoint).TailTD;
    QH = (*UhciEndpoint).QH;

    if !(*UhciEndpoint).HeadTD.is_null() {
        (*TailTD).NextHcdTD = FirstTD;

        (*TailTD).HwTD.NextElement = (*FirstTD).PhysicalAddress;
        (*TailTD).HwTD.NextElement |= UHCI_TD_LINK_PTR_TD as ULONG;

        PhysicalAddress = (*QH).HwQH.NextElement;

        PhysicalAddress &= !(UHCI_TD_LINK_PTR_TERMINATE
            | UHCI_TD_LINK_PTR_QH
            | UHCI_TD_LINK_PTR_DEPTH_FIRST) as ULONG;

        if (*FirstTD).HwTD.ControlStatus.c2rust_unnamed.Status() as i32 & (UHCI_TD_STS_ACTIVE as i32) != 0 {
            if PhysicalAddress == (*TailTD).PhysicalAddress
                && (*TailTD).HwTD.ControlStatus.c2rust_unnamed.Status() as i32 & (UHCI_TD_STS_ACTIVE as i32)
                    == 0
            {
                (*QH).HwQH.NextElement = (*FirstTD).PhysicalAddress;

                (*QH).HwQH.NextElement &=
                    !(UHCI_QH_ELEMENT_LINK_PTR_TERMINATE | UHCI_QH_ELEMENT_LINK_PTR_QH) as ULONG;
            }
        }
    } else {
        if !FirstTD.is_null() {
            (*UhciEndpoint).HeadTD = FirstTD;
        } else {
            (*UhciEndpoint).TailTD = core::ptr::null_mut();
            (*UhciEndpoint).HeadTD = core::ptr::null_mut();
        }

        if FirstTD.is_null()
            || (*UhciEndpoint).Flags as ULONG & UHCI_ENDPOINT_FLAG_HALTED as ULONG != 0
        {
            PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
        } else {
            PhysicalAddress = (*FirstTD).PhysicalAddress;
            PhysicalAddress &= !UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
        }

        (*QH).HwQH.NextElement =
            (PhysicalAddress as ULONG & !UHCI_QH_ELEMENT_LINK_PTR_QH as ULONG) as ULONG;
    }

    if (*UhciEndpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG {
        BulkTailQH = (*UhciExtension).BulkTailQH;
        (*BulkTailQH).HwQH.NextQH &= !UHCI_QH_HEAD_LINK_PTR_TERMINATE as ULONG;
    }

    (*UhciEndpoint).TailTD = LastTD;
}

#[no_mangle]
pub unsafe extern "system" fn UhciAllocateTD(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
) -> PUHCI_HCD_TD {
    let mut TD: PUHCI_HCD_TD;
    let mut AllocTdCounter: ULONG;
    let mut ix: ULONG;

    DPRINT_UHCI!(b"UhciAllocateTD: ...\n\0".as_ptr() as *const i8);

    AllocTdCounter = (*UhciEndpoint).AllocTdCounter;

    ix = 0;
    while ix < (*UhciEndpoint).MaxTDs {
        TD = (*UhciEndpoint).FirstTD.offset(AllocTdCounter as isize);

        if (*TD).Flags & UHCI_HCD_TD_FLAG_ALLOCATED == 0 {
            (*TD).Flags |= UHCI_HCD_TD_FLAG_ALLOCATED;

            (*UhciEndpoint).AllocatedTDs += 1;
            (*UhciEndpoint).AllocTdCounter = AllocTdCounter;

            return TD;
        }

        if AllocTdCounter < (*UhciEndpoint).MaxTDs - 1 {
            AllocTdCounter += 1;
        } else {
            AllocTdCounter = 0;
        }

        ix += 1;
    }

    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn UhciMapAsyncTransferToTDs(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
    UhciTransfer: PUHCI_TRANSFER,
    OutFirstTD: *mut PUHCI_HCD_TD,
    OutLastTD: *mut PUHCI_HCD_TD,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) {

    let mut TD: PUHCI_HCD_TD;
    let mut LastTD: PUHCI_HCD_TD = core::ptr::null_mut();
    let mut PhysicalAddress: ULONG;
    let TotalMaxPacketSize: USHORT;
    let DeviceSpeed: USHORT;
    let EndpointAddress: USHORT;
    let DeviceAddress: USHORT;
    let TransferType: ULONG;
    let mut TransferLength: SIZE_T = 0 as SIZE_T;
    let mut LengthMapped: SIZE_T = 0 as SIZE_T;
    let mut BytesRemaining: SIZE_T;
    let mut LengthThisTD: SIZE_T = 0;
    let mut ix: ULONG;
    let mut DataToggle: BOOL;
    let mut PIDCode: UCHAR;
    let mut IsLastTd: BOOLEAN = TRUE as BOOLEAN;
    let mut ZeroLengthTransfer: BOOLEAN = TRUE as BOOLEAN;

    TotalMaxPacketSize = (*UhciEndpoint).EndpointProperties.TotalMaxPacketSize;
    DeviceSpeed = (*UhciEndpoint).EndpointProperties.DeviceSpeed as USHORT;
    EndpointAddress = (*UhciEndpoint).EndpointProperties.EndpointAddress;
    DeviceAddress = (*UhciEndpoint).EndpointProperties.DeviceAddress;
    TransferType = (*UhciEndpoint).EndpointProperties.TransferType;

    if (*SgList).SgElementCount != 0 || TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        ZeroLengthTransfer = FALSE as BOOLEAN;
    }

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        if (*(*UhciTransfer).TransferParameters).TransferFlags as ULONG
            & USBD_TRANSFER_DIRECTION_IN as ULONG
            != 0
        {
            PIDCode = UHCI_TD_PID_IN as UCHAR;
        } else {
            PIDCode = UHCI_TD_PID_OUT as UCHAR;
        }
        DataToggle = UHCI_TD_PID_DATA1 as BOOL;
    } else {
        if EndpointAddress as i32 & (USB_ENDPOINT_DIRECTION_MASK as i32) == 0 {
            PIDCode = UHCI_TD_PID_OUT as UCHAR;
        } else {
            PIDCode = UHCI_TD_PID_IN as UCHAR;
        }
        DataToggle = (*UhciEndpoint).DataToggle;
    }

    ix = 0 as ULONG;
    while ix < (*SgList).SgElementCount || ZeroLengthTransfer as i32 != 0 {
        BytesRemaining = (*SgList).SgElement[ix as usize].SgTransferLength as SIZE_T;
        PhysicalAddress = (*SgList).SgElement[ix as usize].SgPhysicalAddress.__bindgen_anon_1.LowPart;

        if IsLastTd == 0 {
            PhysicalAddress = (PhysicalAddress as u64).wrapping_add(TransferLength as u64) as ULONG;
            BytesRemaining = (BytesRemaining as u64).wrapping_sub(TransferLength as u64) as SIZE_T;
        }

        IsLastTd = TRUE as BOOLEAN;
        TransferLength = 0 as SIZE_T;

        while BytesRemaining != 0 || ZeroLengthTransfer as i32 != 0 {
            ZeroLengthTransfer = FALSE as BOOLEAN;

            if BytesRemaining >= TotalMaxPacketSize as SIZE_T {
                LengthThisTD = TotalMaxPacketSize as SIZE_T;
                BytesRemaining =
                    (BytesRemaining as u64).wrapping_sub(TotalMaxPacketSize as u64) as SIZE_T;
            } else {
                if ix >= ((*SgList).SgElementCount as ULONG).wrapping_sub(1 as ULONG) {
                    LengthThisTD = BytesRemaining;
                } else {
                    IsLastTd = FALSE as BOOLEAN;

                    if DbgPrint(
                        b"(%s:%d) UhciMapAsyncTransferToTds: IsLastTd = FALSE. FIXME\n\0".as_ptr()
                            as *const i8,
                        b"usbuhci.c\0".as_ptr() as *const i8,
                        1175 as i32,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                            b"usbuhci.c\0".as_ptr() as *const i8,
                            1175 as i32,
                        );
                    }
                    if 0 as i32 == 0 {
                        RtlAssert(
                            b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                            b"usbuhci.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                            1176 as ULONG,
                            core::ptr::null_mut(),
                        );
                    } else {
                    };
                }

                BytesRemaining = 0 as SIZE_T;
            }

            (*UhciTransfer).PendingTds = (*UhciTransfer).PendingTds.wrapping_add(1);
            TD = UhciAllocateTD(UhciExtension, UhciEndpoint);
            (*TD).Flags |= UHCI_HCD_TD_FLAG_PROCESSED as ULONG;

            (*TD).HwTD.NextElement = 0 as ULONG;
            (*TD).HwTD.Buffer = PhysicalAddress;

            (*TD).HwTD.ControlStatus.AsULONG = 0 as ULONG;
            (*TD).HwTD.ControlStatus.set_LowSpeedDevice(
                (DeviceSpeed as i32 == UsbLowSpeed as i32) as i32 as ULONG,
            );
            (*TD)
                .HwTD
                .ControlStatus
                .set_Status(UHCI_TD_STS_ACTIVE as ULONG);
            (*TD).HwTD.ControlStatus.set_ErrorCounter(3 as ULONG);
            (*TD)
                .HwTD
                .ControlStatus
                .set_ActualLength(UHCI_TD_LENGTH_NULL as ULONG);
            (*TD).HwTD.ControlStatus.set_ShortPacketDetect(1 as ULONG);

            (*TD).HwTD.Token.AsULONG = 0 as ULONG;
            (*TD).HwTD.Token.set_Endpoint(EndpointAddress as ULONG);
            (*TD).HwTD.Token.set_DeviceAddress(DeviceAddress as ULONG);
            (*TD).HwTD.Token.set_PIDCode(PIDCode as ULONG);

            if LengthThisTD == 0 as SIZE_T {
                (*TD)
                    .HwTD
                    .Token
                    .set_MaximumLength(UHCI_TD_LENGTH_NULL as ULONG);
            } else {
                (*TD)
                    .HwTD
                    .Token
                    .set_MaximumLength((LengthThisTD as u64).wrapping_sub(1 as u64) as ULONG);
            }

            (*TD)
                .HwTD
                .Token
                .set_DataToggle((DataToggle == (UHCI_TD_PID_DATA1 as i32)) as i32 as ULONG);

            (*TD).NextHcdTD = core::ptr::null_mut();
            (*TD).__bindgen_anon_1.UhciTransfer = UhciTransfer as PUHCI_TRANSFER;

            if IsLastTd == 0 {
                if 0 as i32 == 0 {
                    RtlAssert(
                        b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                        b"usbuhci.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                        1212 as ULONG,
                        core::ptr::null_mut(),
                    );
                } else {
                };
            }

            PhysicalAddress =
                (PhysicalAddress as u64).wrapping_add(LengthThisTD as u64) as ULONG;
            LengthMapped = (LengthMapped as u64).wrapping_add(LengthThisTD as u64) as SIZE_T;

            if !LastTD.is_null() {
                (*LastTD).HwTD.NextElement =
                    ((*TD).PhysicalAddress as ULONG & UHCI_TD_LINK_POINTER_MASK as ULONG) as ULONG;
                (*LastTD).NextHcdTD = TD;
            } else {
                *OutFirstTD = TD;
            }

            LastTD = TD;
            DataToggle = (DataToggle == (UHCI_TD_PID_DATA0 as i32)) as i32 as BOOL;
        }

        ix = ix.wrapping_add(1);
    }

    (*UhciEndpoint).DataToggle = DataToggle;

    *OutLastTD = LastTD;
}

#[no_mangle]
pub unsafe extern "system" fn UhciControlTransfer(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    UhciTransfer: PUHCI_TRANSFER,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut FirstTD: PUHCI_HCD_TD;
    let LastTD: PUHCI_HCD_TD;
    let mut DataFirstTD: PUHCI_HCD_TD;
    let mut DataLastTD: PUHCI_HCD_TD;
    let mut ControlStatus: UHCI_CONTROL_STATUS = core::mem::zeroed();
    let DeviceSpeed: USB_DEVICE_SPEED;
    let EndpointAddress: USHORT;
    let DeviceAddress: USHORT;
    let mut PhysicalAddress: ULONG;

    if (*UhciEndpoint).EndpointLock > 1 {
        InterlockedDecrement(core::ptr::addr_of_mut!((*UhciEndpoint).EndpointLock));

        if (*UhciEndpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
            InterlockedDecrement(core::ptr::addr_of_mut!((*UhciExtension).ExtensionLock));
        }

        return MP_STATUS_FAILURE;
    }

    DeviceSpeed = (*UhciEndpoint).EndpointProperties.DeviceSpeed;
    EndpointAddress = (*UhciEndpoint).EndpointProperties.EndpointAddress;
    DeviceAddress = (*UhciEndpoint).EndpointProperties.DeviceAddress;

    /* Allocate and setup first TD */
    (*UhciTransfer).PendingTds += 1;
    FirstTD = UhciAllocateTD(UhciExtension, UhciEndpoint);
    (*FirstTD).Flags |= UHCI_HCD_TD_FLAG_PROCESSED;

    (*FirstTD).HwTD.NextElement = 0;

    ControlStatus.AsULONG = 0;
    ControlStatus.set_LowSpeedDevice((DeviceSpeed == UsbLowSpeed) as ULONG);
    ControlStatus.set_Status(ControlStatus.__bindgen_anon_1.Status() | UHCI_TD_STS_ACTIVE);
    ControlStatus.set_ErrorCounter(3);
    (*FirstTD).HwTD.ControlStatus = ControlStatus;

    (*FirstTD).HwTD.Token.AsULONG = 0;
    (*FirstTD).HwTD.Token.set_Endpoint(EndpointAddress as ULONG);
    (*FirstTD).HwTD.Token.set_DeviceAddress(DeviceAddress as ULONG);

    (*FirstTD)
        .HwTD
        .Token
        .set_MaximumLength(core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as ULONG);
    (*FirstTD)
        .HwTD
        .Token
        .set_MaximumLength((*FirstTD).HwTD.Token.MaximumLength() - 1);
    (*FirstTD).HwTD.Token.set_PIDCode(UHCI_TD_PID_SETUP);
    (*FirstTD).HwTD.Token.set_DataToggle(UHCI_TD_PID_DATA0);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*FirstTD).SetupPacket) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*TransferParameters).SetupPacket) as *const core::ffi::c_void,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
    );

    (*FirstTD).HwTD.Buffer = ((*FirstTD).PhysicalAddress as core::ffi::c_ulong)
        .wrapping_add(core::mem::offset_of!(UHCI_HCD_TD, SetupPacket) as core::ffi::c_ulong)
        as ULONG;

    (*FirstTD).NextHcdTD = core::ptr::null_mut();
    (*FirstTD).__bindgen_anon_1.UhciTransfer = UhciTransfer;

    /* Allocate and setup last TD */
    (*UhciTransfer).PendingTds += 1;
    LastTD = UhciAllocateTD(UhciExtension, UhciEndpoint);
    (*LastTD).Flags |= UHCI_HCD_TD_FLAG_PROCESSED;

    (*LastTD).HwTD.NextElement = 0;

    (*LastTD).HwTD.ControlStatus.AsULONG = 0;
    (*LastTD)
        .HwTD
        .ControlStatus
        .set_LowSpeedDevice((DeviceSpeed == UsbLowSpeed) as ULONG);
    (*LastTD)
        .HwTD
        .ControlStatus
        .set_Status((*LastTD).HwTD.ControlStatus.__bindgen_anon_1.Status() | UHCI_TD_STS_ACTIVE);
    (*LastTD).HwTD.ControlStatus.set_ErrorCounter(3);

    (*LastTD).HwTD.Token.AsULONG = 0;
    (*LastTD).HwTD.Token.set_Endpoint(EndpointAddress as ULONG);
    (*LastTD).HwTD.Token.set_DeviceAddress(DeviceAddress as ULONG);

    (*LastTD).__bindgen_anon_1.UhciTransfer = UhciTransfer;
    (*LastTD).NextHcdTD = core::ptr::null_mut();

    /* Allocate and setup TDs for data */
    DataFirstTD = core::ptr::null_mut();
    DataLastTD = core::ptr::null_mut();

    UhciMapAsyncTransferToTDs(
        UhciExtension,
        UhciEndpoint,
        UhciTransfer,
        core::ptr::addr_of_mut!(DataFirstTD),
        core::ptr::addr_of_mut!(DataLastTD),
        SgList,
    );

    if !DataFirstTD.is_null() {
        PhysicalAddress = (*DataFirstTD).PhysicalAddress;
        PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
        (*FirstTD).HwTD.NextElement = PhysicalAddress;
        (*FirstTD).NextHcdTD = DataFirstTD;

        PhysicalAddress = (*LastTD).PhysicalAddress;
        PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
        (*DataLastTD).HwTD.NextElement = PhysicalAddress;
        (*DataLastTD).NextHcdTD = LastTD;
    } else {
        PhysicalAddress = (*LastTD).PhysicalAddress;
        PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
        (*FirstTD).HwTD.NextElement = PhysicalAddress;
        (*FirstTD).NextHcdTD = LastTD;
    }

    (*LastTD).HwTD.Buffer = 0;
    (*LastTD).HwTD.ControlStatus.set_InterruptOnComplete(1);

    (*LastTD).HwTD.Token.set_DataToggle(UHCI_TD_PID_DATA1);
    (*LastTD).HwTD.Token.set_MaximumLength(UHCI_TD_LENGTH_NULL);

    if (*(*UhciTransfer).TransferParameters).TransferFlags & USBD_TRANSFER_DIRECTION_IN != 0 {
        (*LastTD).HwTD.Token.set_PIDCode(UHCI_TD_PID_OUT);
    } else {
        (*LastTD).HwTD.Token.set_PIDCode(UHCI_TD_PID_IN);
    }

    (*LastTD).HwTD.NextElement = UHCI_TD_LINK_PTR_TERMINATE;

    (*LastTD).Flags |= UHCI_HCD_TD_FLAG_CONTROLL;
    (*LastTD).NextHcdTD = core::ptr::null_mut();

    /* Link this transfer to queue */
    UhciQueueTransfer(UhciExtension, UhciEndpoint, FirstTD, LastTD);

    return MP_STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciBulkOrInterruptTransfer(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    UhciTransfer: PUHCI_TRANSFER,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {

    let mut DataFirstTD: PUHCI_HCD_TD;
    let mut DataLastTD: PUHCI_HCD_TD;
    let TotalMaxPacketSize: ULONG;
    let SgCount: ULONG;
    let mut TransferLength: ULONG;
    let mut TDs: ULONG;
    let mut ix: ULONG;

    TotalMaxPacketSize = (*UhciEndpoint).EndpointProperties.TotalMaxPacketSize as ULONG;

    SgCount = (*SgList).SgElementCount;

    if SgCount == 0 {
        TDs = 1;
    } else {
        TransferLength = 0;

        ix = 0;
        while ix < SgCount {
            TransferLength = TransferLength
                .wrapping_add((*SgList).SgElement[ix as usize].SgTransferLength);
            ix = ix.wrapping_add(1);
        }

        if TransferLength != 0 {
            TDs = TransferLength.wrapping_add(TotalMaxPacketSize.wrapping_sub(1));
            TDs = TDs.wrapping_div(TotalMaxPacketSize);
        } else {
            TDs = 1;
        }
    }

    if (*UhciEndpoint)
        .MaxTDs
        .wrapping_sub((*UhciEndpoint).AllocatedTDs)
        < TDs
    {
        if DbgPrint(
            b"(%s:%d) UhciBulkOrInterruptTransfer: Not enough TDs \n\0".as_ptr() as PCSTR,
            b"usbuhci.c\0".as_ptr() as *const ::core::ffi::c_char,
            1440 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"usbuhci.c\0".as_ptr() as *const ::core::ffi::c_char,
                1440 as ::core::ffi::c_int,
            );
        }
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    DataFirstTD = core::ptr::null_mut();
    DataLastTD = core::ptr::null_mut();

    UhciMapAsyncTransferToTDs(
        UhciExtension,
        UhciEndpoint,
        UhciTransfer,
        core::ptr::addr_of_mut!(DataFirstTD),
        core::ptr::addr_of_mut!(DataLastTD),
        SgList,
    );

    if DataLastTD.is_null() || DataFirstTD.is_null() {
        if DbgPrint(
            b"(%s:%d) UhciBulkOrInterruptTransfer: !DataLastTD || !DataFirstTD\n\0".as_ptr()
                as PCSTR,
            b"usbuhci.c\0".as_ptr() as *const ::core::ffi::c_char,
            1456 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"usbuhci.c\0".as_ptr() as *const ::core::ffi::c_char,
                1456 as ::core::ffi::c_int,
            );
        }
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*DataLastTD).HwTD.NextElement = UHCI_TD_LINK_PTR_TERMINATE as ULONG;
    (*DataLastTD)
        .HwTD
        .ControlStatus
        .c2rust_unnamed
        .set_InterruptOnComplete(1 as ULONG);
    (*DataLastTD).NextHcdTD = core::ptr::null_mut();

    UhciQueueTransfer(UhciExtension, UhciEndpoint, DataFirstTD, DataLastTD);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciSubmitTransfer(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    uhciTransfer: PVOID,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let UhciTransfer: PUHCI_TRANSFER = uhciTransfer as PUHCI_TRANSFER;
    let mut TransferType: ULONG;

    InterlockedIncrement(core::ptr::addr_of_mut!((*UhciEndpoint).EndpointLock));

    TransferType = (*UhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG
        && InterlockedIncrement(core::ptr::addr_of_mut!((*UhciExtension).ExtensionLock)) == 1
    {
        (*UhciExtension).FrameNumber = UhciGet32BitFrameNumber(UhciExtension as PVOID);
    }

    RtlZeroMemory(
        UhciTransfer as PVOID,
        (core::mem::size_of::<UHCI_TRANSFER>() as ULONG as usize),
    );

    (*UhciTransfer).TransferParameters = TransferParameters;
    (*UhciTransfer).UhciEndpoint = UhciEndpoint;
    (*UhciTransfer).USBDStatus = USBD_STATUS_SUCCESS;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        return UhciControlTransfer(
            UhciExtension,
            UhciEndpoint,
            TransferParameters,
            UhciTransfer,
            SgList,
        );
    }

    if TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        return UhciBulkOrInterruptTransfer(
            UhciExtension,
            UhciEndpoint,
            TransferParameters,
            UhciTransfer,
            SgList,
        );
    }

    DbgPrint(
        b"UhciSubmitTransfer: Error TransferType - %x\n\0".as_ptr() as PCSTR,
        TransferType,
    );

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciGetErrorFromTD(
    UhciExtension: PUHCI_EXTENSION,
    TD: PUHCI_HCD_TD,
) -> USBD_STATUS {
    let mut USBDStatus: USBD_STATUS;
    let TdStatus: UCHAR;

    TdStatus = (*TD).HwTD.ControlStatus.__bindgen_anon_1.Status() as UCHAR;

    if TdStatus as i32 == (UHCI_TD_STS_ACTIVE as i32) {
        if (*TD).HwTD.Token.MaximumLength() as i32 == (UHCI_TD_LENGTH_NULL as i32) {
            return USBD_STATUS_SUCCESS;
        }

        if (*TD).HwTD.ControlStatus.ActualLength() as i32 + 1
            >= (*TD).HwTD.Token.MaximumLength() as i32 + 1
        {
            return USBD_STATUS_SUCCESS;
        }

        if (*TD).HwTD.ControlStatus.InterruptOnComplete() as i32 == 1 {
            return USBD_STATUS_SUCCESS;
        }

        return USBD_STATUS_ERROR_SHORT_TRANSFER;
    }

    if TdStatus as i32 & (UHCI_TD_STS_BABBLE_DETECTED as i32) != 0
        && TdStatus as i32 & (UHCI_TD_STS_STALLED as i32) != 0
    {
        if DbgPrint(
            b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_BUFFER_OVERRUN, TD - %p\n\0" as *const u8
                as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
            1559,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1559,
            );
        }
        return USBD_STATUS_BUFFER_OVERRUN;
    }

    if TdStatus as i32 & (UHCI_TD_STS_TIMEOUT_CRC_ERROR as i32) != 0
        && TdStatus as i32 & (UHCI_TD_STS_STALLED as i32) != 0
    {
        if DbgPrint(
            b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_DEV_NOT_RESPONDING, TD - %p\n\0" as *const u8
                as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
            1566,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1566,
            );
        }
        return USBD_STATUS_DEV_NOT_RESPONDING;
    }

    if TdStatus as i32 & (UHCI_TD_STS_TIMEOUT_CRC_ERROR as i32) != 0 {
        if (*TD).HwTD.ControlStatus.ActualLength() as i32 == (UHCI_TD_LENGTH_NULL as i32) {
            if DbgPrint(
                b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_DEV_NOT_RESPONDING, TD - %p\n\0"
                    as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1574,
                TD,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                    1574,
                );
            }
            return USBD_STATUS_DEV_NOT_RESPONDING;
        } else {
            if DbgPrint(
                b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_CRC, TD - %p\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1579,
                TD,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                    1579,
                );
            }
            return USBD_STATUS_CRC;
        }
    } else if TdStatus as i32 & (UHCI_TD_STS_DATA_BUFFER_ERROR as i32) != 0 {
        if DbgPrint(
            b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_DATA_OVERRUN, TD - %p\n\0" as *const u8
                as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
            1585,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1585,
            );
        }
        USBDStatus = USBD_STATUS_DATA_OVERRUN;
    } else if TdStatus as i32 & (UHCI_TD_STS_STALLED as i32) != 0 {
        if DbgPrint(
            b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_STALL_PID, TD - %p\n\0" as *const u8 as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
            1590,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1590,
            );
        }
        USBDStatus = USBD_STATUS_STALL_PID;
    } else {
        if DbgPrint(
            b"(%s:%d) UhciGetErrorFromTD: USBD_STATUS_INTERNAL_HC_ERROR, TD - %p\n\0" as *const u8
                as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
            1595,
            TD,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const ::core::ffi::c_char,
                1595,
            );
        }
        USBDStatus = USBD_STATUS_INTERNAL_HC_ERROR;
    }

    return USBDStatus;
}

#[no_mangle]
pub unsafe extern "system" fn UhciProcessDoneNonIsoTD(
    UhciExtension: PUHCI_EXTENSION,
    TD: PUHCI_HCD_TD,
) {
    let mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS = core::ptr::null_mut();
    let mut UhciEndpoint: PUHCI_ENDPOINT = core::ptr::null_mut();
    let mut UhciTransfer: PUHCI_TRANSFER = core::ptr::null_mut();
    let mut USBDStatus: USBD_STATUS = USBD_STATUS_SUCCESS;
    let mut TransferedLen: SIZE_T;

    DPRINT_UHCI!(b"UhciProcessDoneNonIsoTD: TD - %p\n\0".as_ptr() as *const i8, TD);

    UhciTransfer = (*TD).__bindgen_anon_1.UhciTransfer as PUHCI_TRANSFER;
    (*UhciTransfer).PendingTds = (*UhciTransfer).PendingTds.wrapping_sub(1);

    TransferParameters = (*UhciTransfer).TransferParameters;
    UhciEndpoint = (*UhciTransfer).UhciEndpoint;

    if (*TD).Flags & UHCI_HCD_TD_FLAG_NOT_ACCESSED == 0 {
        if (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_HALTED != 0 {
            USBDStatus = UhciGetErrorFromTD(UhciExtension, TD);
        }

        if USBDStatus != USBD_STATUS_SUCCESS
            || ((*TD).HwTD.ControlStatus.ActualLength() as i32 == (UHCI_TD_LENGTH_NULL as i32))
        {
            TransferedLen = 0 as SIZE_T;
        } else {
            TransferedLen = ((*TD).HwTD.ControlStatus.ActualLength() as i32 + 1) as SIZE_T;
        }

        if (*TD).HwTD.Token.PIDCode() as i32 != (UHCI_TD_PID_SETUP as i32) {
            (*UhciTransfer).TransferLen =
                (*UhciTransfer).TransferLen.wrapping_add(((TransferedLen as u32) as _));
        }

        if (*TD).HwTD.Token.PIDCode() as i32 == (UHCI_TD_PID_IN as i32)
            && (*TD).Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER != 0
        {
            DPRINT_IMPL!(b"UhciProcessDoneNonIsoTD: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
        }

        if USBDStatus != USBD_STATUS_SUCCESS {
            (*UhciTransfer).USBDStatus = USBDStatus;
        }
    }

    if (*TD).Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER != 0 {
        DPRINT_IMPL!(b"UhciProcessDoneNonIsoTD: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    }

    (*UhciEndpoint).AllocatedTDs = (*UhciEndpoint).AllocatedTDs.wrapping_sub(1);

    (*TD).HwTD.NextElement = 0 as ULONG;
    (*TD).__bindgen_anon_1.UhciTransfer = core::ptr::null_mut();
    (*TD).Flags = 0 as ULONG;

    if (*UhciTransfer).PendingTds == 0 {
        InterlockedDecrement(core::ptr::addr_of_mut!((*UhciEndpoint).EndpointLock));

        if (*UhciEndpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS {
            InterlockedDecrement(core::ptr::addr_of_mut!((*UhciExtension).ExtensionLock));
        }

        (RegPacket.UsbPortCompleteTransfer.expect("non-null function pointer"))(
            UhciExtension as PVOID,
            UhciEndpoint as PVOID,
            TransferParameters as PVOID,
            (*UhciTransfer).USBDStatus,
            (*UhciTransfer).TransferLen as ULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciIsochTransfer(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    ehciTransfer: PVOID,
    isoParameters: PVOID,
) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciIsochTransfer: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciAbortIsoTransfer(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
    UhciTransfer: PUHCI_TRANSFER,
) {
    DPRINT_IMPL!(b"UhciAbortIsoTransfer: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciAbortNonIsoTransfer(
    mut UhciExtension: PUHCI_EXTENSION,
    mut UhciEndpoint: PUHCI_ENDPOINT,
    mut UhciTransfer: PUHCI_TRANSFER,
    mut CompletedLength: PULONG,
) {
    let mut TD: PUHCI_HCD_TD;
    let mut PrevTD: PUHCI_HCD_TD = core::ptr::null_mut();
    let mut PhysicalAddress: ULONG;
    let mut DataToggle: BOOL;
    let mut IsHeadTD: BOOLEAN = FALSE as BOOLEAN;

    TD = (*UhciEndpoint).HeadTD;
    while !TD.is_null() && (*TD).__bindgen_anon_1.UhciTransfer != UhciTransfer {
        PrevTD = TD;
        TD = (*TD).NextHcdTD as PUHCI_HCD_TD;
    }

    DataToggle = (*TD).HwTD.Token.DataToggle() as BOOL;

    if TD == (*UhciEndpoint).HeadTD {
        IsHeadTD = TRUE as BOOLEAN;
    }

    while !TD.is_null() && (*TD).__bindgen_anon_1.UhciTransfer == UhciTransfer {
        if (*TD).HwTD.ControlStatus.__bindgen_anon_1.Status() as i32 & (UHCI_TD_STS_ACTIVE as i32) != 0 {
            if (*TD).Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER != 0 {
                // DPRINT_IMPL!("UhciAbortNonIsoTransfer: UNIMPLEMENTED. FIXME\n");
            }

            (*UhciEndpoint).AllocatedTDs = (*UhciEndpoint).AllocatedTDs.wrapping_sub(1);

            (*TD).HwTD.NextElement = 0 as ULONG;
            (*TD).Flags = 0 as ULONG;
            (*TD).__bindgen_anon_1.UhciTransfer = core::ptr::null_mut();
        } else {
            UhciProcessDoneNonIsoTD(UhciExtension, TD);
        }

        TD = (*TD).NextHcdTD as PUHCI_HCD_TD;
    }

    UhciFixDataToggle(UhciExtension, UhciEndpoint, TD, DataToggle);

    if IsHeadTD != 0 {
        if !TD.is_null() {
            (*UhciEndpoint).HeadTD = TD;
        } else {
            (*UhciEndpoint).HeadTD = core::ptr::null_mut();
            (*UhciEndpoint).TailTD = core::ptr::null_mut();
        }

        if TD.is_null()
            || (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_HALTED != 0
        {
            PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
        } else {
            PhysicalAddress = (*TD).PhysicalAddress;
            PhysicalAddress &= !(UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG);
        }

        (*(*UhciEndpoint).QH).HwQH.NextElement = PhysicalAddress;
        (*(*UhciEndpoint).QH).HwQH.NextElement &= !(UHCI_QH_ELEMENT_LINK_PTR_QH as ULONG);
    } else if !TD.is_null() {
        (*PrevTD).HwTD.NextElement =
            (*TD).PhysicalAddress & UHCI_TD_LINK_POINTER_MASK as ULONG;
        (*PrevTD).NextHcdTD = TD;
    } else {
        (*PrevTD).NextHcdTD = core::ptr::null_mut();
        (*PrevTD).HwTD.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;

        (*UhciEndpoint).TailTD = PrevTD;
    }

    *CompletedLength = ((*UhciTransfer).TransferLen as u32);
}

#[no_mangle]
pub unsafe extern "system" fn UhciAbortTransfer(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    uhciTransfer: PVOID,
    CompletedLength: PULONG,
) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let UhciTransfer: PUHCI_TRANSFER = uhciTransfer as PUHCI_TRANSFER;
    let mut TransferType: ULONG;

    InterlockedDecrement(
        core::ptr::addr_of_mut!((*UhciEndpoint).EndpointLock) as *mut core::ffi::c_long,
    );

    TransferType = (*UhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        InterlockedDecrement(
            core::ptr::addr_of_mut!((*UhciExtension).ExtensionLock) as *mut core::ffi::c_long,
        );

        UhciAbortIsoTransfer(UhciExtension, UhciEndpoint, UhciTransfer);
    }

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        UhciAbortNonIsoTransfer(UhciExtension, UhciEndpoint, UhciTransfer, CompletedLength);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciGetEndpointState(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
) -> ULONG {
    DPRINT_IMPL!(b"UhciGetEndpointState: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInsertQH(
    UhciExtension: PUHCI_EXTENSION,
    StaticQH: PUHCI_HCD_QH,
    QH: PUHCI_HCD_QH,
) {
    let mut NextHcdQH: PUHCI_HCD_QH;

    (*QH).HwQH.NextQH = (*StaticQH).HwQH.NextQH;
    NextHcdQH = (*StaticQH).NextHcdQH as PUHCI_HCD_QH;

    (*QH).PrevHcdQH = StaticQH;
    (*QH).NextHcdQH = NextHcdQH;

    if !NextHcdQH.is_null() {
        (*NextHcdQH).PrevHcdQH = QH;
    } else {
        (*UhciExtension).BulkTailQH = QH;
    }

    (*StaticQH).HwQH.NextQH = ((*QH).PhysicalAddress as ULONG
        | UHCI_QH_HEAD_LINK_PTR_QH as ULONG) as ULONG;
    (*StaticQH).NextHcdQH = QH;

    (*QH).QhFlags |= UHCI_HCD_QH_FLAG_ACTIVE as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn UhciUnlinkQH(UhciExtension: PUHCI_EXTENSION, QH: PUHCI_HCD_QH) {
    let mut NextHcdQH: PUHCI_HCD_QH;
    let mut PrevHcdQH: PUHCI_HCD_QH;
    let mut BulkQH: PUHCI_HCD_QH;

    NextHcdQH = (*QH).NextHcdQH;
    PrevHcdQH = (*QH).PrevHcdQH;

    if (*UhciExtension).BulkTailQH == QH {
        (*UhciExtension).BulkTailQH = PrevHcdQH;
    }

    (*PrevHcdQH).HwQH.NextQH = (*QH).HwQH.NextQH;
    (*PrevHcdQH).NextHcdQH = NextHcdQH;

    if !NextHcdQH.is_null() {
        (*NextHcdQH).PrevHcdQH = PrevHcdQH;
    }

    (*QH).PrevHcdQH = QH;
    (*QH).NextHcdQH = QH;

    if !((*(*QH).UhciEndpoint).EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_BULK as u32) {
        (*QH).QhFlags &= !(UHCI_HCD_QH_FLAG_ACTIVE as u32);
        return;
    }

    if (*(*UhciExtension).BulkTailQH).HwQH.NextQH & UHCI_QH_HEAD_LINK_PTR_TERMINATE as u32
        == UHCI_QH_HEAD_LINK_PTR_TERMINATE as u32
    {
        (*QH).QhFlags &= !(UHCI_HCD_QH_FLAG_ACTIVE as u32);
        return;
    }

    BulkQH = (*UhciExtension).BulkQH;

    loop {
        BulkQH = (*BulkQH).NextHcdQH;

        if BulkQH.is_null() {
            break;
        }

        if (*BulkQH).HwQH.NextElement & UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as u32 == 0 {
            (*QH).QhFlags &= !(UHCI_HCD_QH_FLAG_ACTIVE as u32);
            return;
        }
    }

    (*(*UhciExtension).BulkTailQH).HwQH.NextQH |= UHCI_QH_HEAD_LINK_PTR_TERMINATE as u32;

    (*QH).QhFlags &= !(UHCI_HCD_QH_FLAG_ACTIVE as u32);
}

#[no_mangle]
pub unsafe extern "system" fn UhciSetEndpointState(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    EndpointState: ULONG,
) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let TransferType: ULONG;
    let QH: PUHCI_HCD_QH;
    let Idx: ULONG;

    TransferType = (*UhciEndpoint).EndpointProperties.TransferType;
    QH = (*UhciEndpoint).QH;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        return;
    }

    if TransferType != USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        && TransferType != USBPORT_TRANSFER_TYPE_BULK as ULONG
        && TransferType != USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
    }

    match EndpointState {
        2 => {
            UhciUnlinkQH(UhciExtension, QH);
            return;
        }

        3 => {
            match TransferType {
                1 => {
                    UhciInsertQH(
                        UhciExtension,
                        (*UhciExtension).ControlQH,
                        (*UhciEndpoint).QH,
                    );
                }
                2 => {
                    UhciInsertQH(UhciExtension, (*UhciExtension).BulkQH, (*UhciEndpoint).QH);
                }
                3 => {
                    Idx = ((*UhciEndpoint).EndpointProperties.Period as ULONG)
                        .wrapping_add((*UhciEndpoint).EndpointProperties.ScheduleOffset);
                    UhciInsertQH(
                        UhciExtension,
                        (*UhciExtension).IntQH[(Idx as ULONG).wrapping_sub(1) as usize],
                        (*UhciEndpoint).QH,
                    );
                }
                _ => {
                    if 0 == 0 {
                        RtlAssert(
                            b"FALSE\0".as_ptr() as *const i8 as PVOID,
                            b"drivers/usb/usbuhci/usbuhci.c\0".as_ptr() as *const i8 as PVOID,
                            1995 as ULONG,
                            core::ptr::null_mut::<CHAR>(),
                        );
                    } else {
                    }
                }
            }
        }

        4 => {
            (*QH).QhFlags |= UHCI_HCD_QH_FLAG_REMOVE as ULONG;
            UhciUnlinkQH(UhciExtension, QH);
        }

        _ => {
            if 0 == 0 {
                RtlAssert(
                    b"FALSE\0".as_ptr() as *const i8 as PVOID,
                    b"drivers/usb/usbuhci/usbuhci.c\0".as_ptr() as *const i8 as PVOID,
                    2007 as ULONG,
                    core::ptr::null_mut::<CHAR>(),
                );
            } else {
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciGetEndpointStatus(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
) -> ULONG {
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let EndpointStatus: ULONG;

    DPRINT_UHCI!(b"UhciGetEndpointStatus: ...\n\0".as_ptr() as *const i8);

    if (*UhciEndpoint).Flags as ULONG & UHCI_ENDPOINT_FLAG_HALTED as ULONG != 0 {
        EndpointStatus = USBPORT_ENDPOINT_HALT as ULONG;
    } else {
        EndpointStatus = USBPORT_ENDPOINT_RUN as ULONG;
    }

    return EndpointStatus;
}

#[no_mangle]
pub unsafe extern "system" fn UhciSetEndpointStatus(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    EndpointStatus: ULONG,
) {
    let UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let mut PhysicalAddress: ULONG;

    if EndpointStatus != USBPORT_ENDPOINT_RUN as ULONG {
        return;
    }

    if (*UhciEndpoint).Flags as ULONG & UHCI_ENDPOINT_FLAG_HALTED as ULONG == 0 {
        return;
    }

    (*UhciEndpoint).Flags &= !UHCI_ENDPOINT_FLAG_HALTED as ULONG;

    if (*UhciEndpoint).HeadTD.is_null() {
        (*UhciEndpoint).TailTD = core::ptr::null_mut();
    }

    if !(*UhciEndpoint).HeadTD.is_null() {
        PhysicalAddress = (*(*UhciEndpoint).HeadTD).PhysicalAddress;
        PhysicalAddress &= !UHCI_TD_LINK_PTR_TERMINATE as ULONG;
        (*(*UhciEndpoint).QH).HwQH.NextElement = PhysicalAddress;
        (*(*UhciEndpoint).QH).HwQH.NextElement &= !UHCI_QH_ELEMENT_LINK_PTR_QH as ULONG;
    } else {
        (*(*UhciEndpoint).QH).HwQH.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE as ULONG;
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciPollIsoEndpoint(
    mut UhciExtension: PUHCI_EXTENSION,
    mut UhciEndpoint: PUHCI_ENDPOINT,
) {
    DPRINT_IMPL!(b"UhciPollIsoEndpoint: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciPollNonIsoEndpoint(
    UhciExtension: PUHCI_EXTENSION,
    UhciEndpoint: PUHCI_ENDPOINT,
) {
    let mut current_block: u64;
    let mut QH: PUHCI_HCD_QH;
    let mut NextTD: PUHCI_HCD_TD;
    let mut TD: PUHCI_HCD_TD;
    let mut NextTdPA: ULONG;
    let mut PhysicalAddress: ULONG;
    let mut TransferedLen: SIZE_T;
    let mut ListTDs: PLIST_ENTRY;
    let mut TdStatus: UCHAR;

    if (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_HALTED != 0 {
        return;
    }

    QH = (*UhciEndpoint).QH;

    NextTdPA = (*QH).HwQH.NextElement & UHCI_QH_ELEMENT_LINK_POINTER_MASK;

    if NextTdPA != 0 {
        NextTD = RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer")(
            NextTdPA,
            UhciExtension as PVOID,
            UhciEndpoint as PVOID,
        ) as PUHCI_HCD_TD;
    } else {
        NextTD = core::ptr::null_mut();
    }

    TD = (*UhciEndpoint).HeadTD;
    loop {
        if !(TD != NextTD && !TD.is_null()) {
            current_block = 13763002826403452995;
            break;
        }

        (*TD).Flags |= UHCI_HCD_TD_FLAG_DONE;
        InsertTailList(
            core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
            core::ptr::addr_of_mut!((*TD).TdLink),
        );

        if !(*TD).NextHcdTD.is_null()
            && (*(*TD).NextHcdTD).HwTD.ControlStatus.__bindgen_anon_1.Status() & UHCI_TD_STS_ACTIVE != 0
        {
            if NextTdPA == 0 {
                TD = (*TD).NextHcdTD;
                current_block = 15751724822426609728;
                break;
            } else if NextTdPA != (*(*TD).NextHcdTD).PhysicalAddress {
                ASSERT!(0);
            }
        } else {
            if (*TD).NextHcdTD.is_null() {
            } else {
            }
        }

        TD = (*TD).NextHcdTD;
    }

    match current_block {
        13763002826403452995 => {
            (*UhciEndpoint).HeadTD = NextTD;
            if NextTD.is_null() {
                (*UhciEndpoint).HeadTD = core::ptr::null_mut();
                (*UhciEndpoint).TailTD = core::ptr::null_mut();
                (*QH).HwQH.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
                current_block = 16549844112612896618;
            } else {
                TdStatus = (*NextTD).HwTD.ControlStatus.__bindgen_anon_1.Status() as UCHAR;

                if TdStatus as i32 & (UHCI_TD_STS_ACTIVE as i32) != 0 {
                    current_block = 16549844112612896618;
                } else {
                    if (*NextTD).HwTD.Token.PIDCode() as i32 == (UHCI_TD_PID_IN as i32)
                        && TdStatus as i32 & (UHCI_TD_STS_STALLED as i32) != 0
                        && TdStatus as i32 & (UHCI_TD_STS_TIMEOUT_CRC_ERROR as i32) != 0
                        && TdStatus as i32 & (UHCI_TD_STS_NAK_RECEIVED as i32) == 0
                        && TdStatus as i32 & (UHCI_TD_STS_BABBLE_DETECTED as i32) == 0
                        && TdStatus as i32 & (UHCI_TD_STS_BITSTUFF_ERROR as i32) == 0
                    {
                        UhciDumpHcdTD(NextTD);

                        if (*NextTD).Flags & UHCI_HCD_TD_FLAG_STALLED_SETUP == 0 {
                            (*NextTD).HwTD.ControlStatus.set_ErrorCounter(3);

                            (*NextTD).HwTD.ControlStatus.set_Status(
                                (*NextTD).HwTD.ControlStatus.__bindgen_anon_1.Status()
                                    & !((UHCI_TD_STS_STALLED | UHCI_TD_STS_TIMEOUT_CRC_ERROR) as ULONG),
                            );

                            (*NextTD).HwTD.ControlStatus.set_Status(
                                (*NextTD).HwTD.ControlStatus.__bindgen_anon_1.Status() | UHCI_TD_STS_ACTIVE as ULONG,
                            );

                            (*NextTD).Flags = (*NextTD).Flags | UHCI_HCD_TD_FLAG_STALLED_SETUP;

                            current_block = 16549844112612896618;
                        } else {
                            current_block = 18325745679564279244;
                        }
                    } else {
                        current_block = 18325745679564279244;
                    }

                    match current_block {
                        16549844112612896618 => {}
                        _ => {
                            if TdStatus as i32
                                & ((UHCI_TD_STS_STALLED
                                    | UHCI_TD_STS_DATA_BUFFER_ERROR
                                    | UHCI_TD_STS_BABBLE_DETECTED
                                    | UHCI_TD_STS_TIMEOUT_CRC_ERROR
                                    | UHCI_TD_STS_BITSTUFF_ERROR) as i32)
                                != 0
                            {
                                UhciDumpHcdTD(NextTD);

                                (*UhciEndpoint).Flags |= UHCI_ENDPOINT_FLAG_HALTED;
                                (*NextTD).Flags |= UHCI_HCD_TD_FLAG_DONE;

                                InsertTailList(
                                    core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
                                    core::ptr::addr_of_mut!((*NextTD).TdLink),
                                );

                                if (*TD).__bindgen_anon_1.UhciTransfer != (*NextTD).__bindgen_anon_1.UhciTransfer {
                                    ASSERT!(((*TD).__bindgen_anon_1.UhciTransfer == (*NextTD).__bindgen_anon_1.UhciTransfer) as i32);
                                }

                                while !TD.is_null()
                                    && (*(*(*TD).__bindgen_anon_1.UhciTransfer).TransferParameters).TransferCounter
                                        == (*(*(*NextTD).__bindgen_anon_1.UhciTransfer).TransferParameters)
                                            .TransferCounter
                                {
                                    if (*TD).Flags & UHCI_HCD_TD_FLAG_DONE == 0 {
                                        (*TD).Flags |= UHCI_HCD_TD_FLAG_DONE;
                                        (*TD).Flags |= UHCI_HCD_TD_FLAG_NOT_ACCESSED;

                                        InsertTailList(
                                            core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
                                            core::ptr::addr_of_mut!((*TD).TdLink),
                                        );
                                    }

                                    TD = (*TD).NextHcdTD;
                                }

                                if (*UhciEndpoint).EndpointProperties.TransferType
                                    != USBPORT_TRANSFER_TYPE_CONTROL
                                {
                                    UhciFixDataToggle(
                                        UhciExtension,
                                        UhciEndpoint,
                                        TD,
                                        (*NextTD).HwTD.Token.DataToggle() as BOOL,
                                    );
                                }

                                current_block = 15751724822426609728;
                            } else {
                                TransferedLen =
                                    (*NextTD).HwTD.ControlStatus.ActualLength() as SIZE_T;

                                if TransferedLen == UHCI_TD_LENGTH_NULL as SIZE_T {
                                    TransferedLen = 0;
                                } else {
                                    TransferedLen += 1;
                                }

                                if (*NextTD).HwTD.Token.MaximumLength() as i32 == (UHCI_TD_LENGTH_NULL as i32)
                                    || TransferedLen
                                        >= ((*NextTD).HwTD.Token.MaximumLength() as i32 + 1) as SIZE_T
                                {
                                    if NextTdPA
                                        == ((*QH).HwQH.NextElement
                                            & UHCI_QH_ELEMENT_LINK_POINTER_MASK)
                                    {
                                        (*NextTD).Flags |= UHCI_HCD_TD_FLAG_DONE;
                                        InsertTailList(
                                            core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
                                            core::ptr::addr_of_mut!((*NextTD).TdLink),
                                        );

                                        (*UhciEndpoint).HeadTD = (*NextTD).NextHcdTD;

                                        (*QH).HwQH.NextElement = (*NextTD).HwTD.NextElement;
                                        (*QH).HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TD;
                                    }

                                    current_block = 16549844112612896618;
                                } else {
                                    (*NextTD).Flags |= UHCI_HCD_TD_FLAG_DONE;
                                    InsertTailList(
                                        core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
                                        core::ptr::addr_of_mut!((*NextTD).TdLink),
                                    );

                                    while !TD.is_null()
                                        && (*(*(*TD).__bindgen_anon_1.UhciTransfer).TransferParameters)
                                            .TransferCounter
                                            == (*(*(*NextTD).__bindgen_anon_1.UhciTransfer).TransferParameters)
                                                .TransferCounter
                                    {
                                        if (*TD).Flags & UHCI_HCD_TD_FLAG_CONTROLL != 0
                                            && (*(*(*NextTD).__bindgen_anon_1.UhciTransfer).TransferParameters)
                                                .TransferFlags
                                                & USBD_SHORT_TRANSFER_OK
                                                != 0
                                        {
                                            break;
                                        }

                                        if (*TD).Flags & UHCI_HCD_TD_FLAG_DONE == 0 {
                                            (*TD).Flags |= UHCI_HCD_TD_FLAG_DONE
                                                | UHCI_HCD_TD_FLAG_NOT_ACCESSED;

                                            InsertTailList(
                                                core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs),
                                                core::ptr::addr_of_mut!((*TD).TdLink),
                                            );
                                        }

                                        TD = (*TD).NextHcdTD;
                                    }

                                    if !(*NextTD).NextHcdTD.is_null()
                                        && (*UhciEndpoint).EndpointProperties.TransferType
                                            != USBPORT_TRANSFER_TYPE_CONTROL
                                    {
                                        UhciFixDataToggle(
                                            UhciExtension,
                                            UhciEndpoint,
                                            TD,
                                            (*(*NextTD).NextHcdTD).HwTD.Token.DataToggle() as BOOL,
                                        );
                                    }

                                    if (*(*(*NextTD).__bindgen_anon_1.UhciTransfer).TransferParameters).TransferFlags
                                        & USBD_SHORT_TRANSFER_OK
                                        == 0
                                    {
                                        (*UhciEndpoint).Flags |= UHCI_ENDPOINT_FLAG_HALTED;
                                    }

                                    current_block = 15751724822426609728;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    match current_block {
        15751724822426609728 => {
            if !TD.is_null() {
                (*UhciEndpoint).HeadTD = TD;
            } else {
                (*UhciEndpoint).HeadTD = core::ptr::null_mut();
                (*UhciEndpoint).TailTD = core::ptr::null_mut();
            }

            if TD.is_null() || (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_HALTED != 0 {
                PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
            } else {
                PhysicalAddress = (*TD).PhysicalAddress;
                PhysicalAddress &= !UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
            }

            (*QH).HwQH.NextElement = PhysicalAddress;
            (*QH).HwQH.NextElement &= !UHCI_QH_ELEMENT_LINK_PTR_QH;
        }
        _ => {}
    }

    ListTDs = core::ptr::addr_of_mut!((*UhciEndpoint).ListTDs);

    while !IsListEmpty(ListTDs) {
        TD = ((*ListTDs).Flink as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(UHCI_HCD_TD, TdLink.Flink) as ULONG_PTR)
            as *mut UHCI_HCD_TD;

        RemoveHeadList(ListTDs);

        if (*TD).Flags & (UHCI_HCD_TD_FLAG_PROCESSED | UHCI_HCD_TD_FLAG_DONE)
            == (UHCI_HCD_TD_FLAG_PROCESSED | UHCI_HCD_TD_FLAG_DONE)
        {
            UhciProcessDoneNonIsoTD(UhciExtension, TD);
        }
    }

    if (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_CONTROL_OR_ISO != 0
        && (*UhciEndpoint).Flags & UHCI_ENDPOINT_FLAG_HALTED != 0
    {
        UhciSetEndpointStatus(
            UhciExtension as PVOID,
            UhciEndpoint as PVOID,
            USBPORT_ENDPOINT_RUN,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciPollEndpoint(
    mut uhciExtension: PVOID,
    mut uhciEndpoint: PVOID,
) {
    let mut UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut UhciEndpoint: PUHCI_ENDPOINT = uhciEndpoint as PUHCI_ENDPOINT;
    let mut TransferType: ULONG;

    DPRINT_UHCI!(b"UhciPollEndpoint: ...\n\0".as_ptr() as *const i8);

    TransferType = (*UhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        UhciPollIsoEndpoint(UhciExtension, UhciEndpoint);
        return;
    }

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        UhciPollNonIsoEndpoint(UhciExtension, UhciEndpoint);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciCheckController(uhciExtension: PVOID) {
    const UHCI_MAX_HC_SCHEDULE_ERRORS: ULONG = 16; // FIXED 2026-07-05: translation invented 50; verified vs C header
    const USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE: ULONG = 0x02;

    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;

    if UhciHardwarePresent(UhciExtension) == 0
        || (*UhciExtension).HcScheduleError >= UHCI_MAX_HC_SCHEDULE_ERRORS
    {
        if DbgPrint(
            b"(%s:%d) UhciCheckController: INVALIDATE_CONTROLLER_SURPRISE_REMOVE !!!\n\0"
                as *const u8 as PCSTR,
            b"usbuhci.c\0" as *const u8 as *const i8,
            2431 as i32,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"usbuhci.c\0" as *const u8 as *const i8,
                2431 as i32,
            );
        }

        (RegPacket
            .UsbPortInvalidateController
            .expect("non-null function pointer"))(
            UhciExtension as PVOID,
            USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciGet32BitFrameNumber(uhciExtension: PVOID) -> ULONG {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut Uhci32BitFrame: ULONG;
    let mut Fn: USHORT;
    let Hp: ULONG;

    Fn = READ_PORT_USHORT(core::ptr::addr_of_mut!((*(*UhciExtension).BaseRegister).FrameNumber));
    Fn = (Fn as i32 & (UHCI_FRNUM_FRAME_MASK as i32)) as USHORT;
    Hp = (*UhciExtension).FrameHighPart;

    Uhci32BitFrame = Hp;
    Uhci32BitFrame = (Uhci32BitFrame as core::ffi::c_ulong).wrapping_add(
        ((Hp as USHORT as i32 ^ Fn as i32) & (UHCI_FRNUM_OVERFLOW_LIST as i32)) as core::ffi::c_ulong,
    ) as ULONG;
    Uhci32BitFrame |= Fn as core::ffi::c_ulong as ULONG;

    DPRINT_UHCI!(
        b"UhciGet32BitFrameNumber: Uhci32BitFrame - %lX\n\0".as_ptr() as *const i8,
        Uhci32BitFrame,
    );

    return Uhci32BitFrame;
}

#[no_mangle]
pub unsafe extern "system" fn UhciInterruptNextSOF(uhciExtension: PVOID) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut UhciResources: PUHCI_HC_RESOURCES;
    let CurrentFrame: ULONG;
    let mut SOF_HcdTDs: PUHCI_HCD_TD;
    let mut ix: ULONG;
    let NextFrame: ULONG;
    let mut SofFrame: ULONG;
    let mut Idx: ULONG;

    DPRINT_UHCI!(b"UhciInterruptNextSOF: ...\n\0".as_ptr() as *const i8);

    CurrentFrame = UhciGet32BitFrameNumber(UhciExtension as PVOID);

    SOF_HcdTDs = (*UhciExtension).SOF_HcdTDs;
    NextFrame = CurrentFrame.wrapping_add(2);

    ix = 0;
    while ix < UHCI_MAX_STATIC_SOF_TDS as ULONG {
        SofFrame = (*SOF_HcdTDs).__bindgen_anon_1.Frame;

        if SofFrame == NextFrame {
            break;
        }

        if SofFrame < CurrentFrame {
            (*SOF_HcdTDs).__bindgen_anon_1.Frame = NextFrame;
            (*SOF_HcdTDs).Flags |= UHCI_HCD_TD_FLAG_GOOD_FRAME as ULONG;

            /* Insert SOF_HcdTD (InterruptOnComplete = TRUE) in Frame List */
            UhciResources = (*UhciExtension).HcResourcesVA;
            Idx = (*SOF_HcdTDs).__bindgen_anon_1.Frame & UHCI_FRAME_LIST_INDEX_MASK as ULONG;

            InterlockedExchange(
                core::ptr::addr_of_mut!((*SOF_HcdTDs).HwTD.NextElement) as PLONG,
                (*UhciResources).FrameList[Idx as usize] as LONG,
            );

            (*UhciResources).FrameList[Idx as usize] = (*SOF_HcdTDs).PhysicalAddress;
            break;
        }

        /* Go to next SOF_HcdTD */
        SOF_HcdTDs = SOF_HcdTDs.offset(1);
        ix = ix.wrapping_add(1);
    }

    ix = 0;
    while ix < UHCI_MAX_STATIC_SOF_TDS as ULONG {
        SOF_HcdTDs = core::ptr::addr_of_mut!((*(*UhciExtension).SOF_HcdTDs.offset(ix as isize)));

        if (*SOF_HcdTDs).__bindgen_anon_1.Frame != 0
            && ((*SOF_HcdTDs).__bindgen_anon_1.Frame < CurrentFrame
                || (*SOF_HcdTDs).__bindgen_anon_1.Frame.wrapping_sub(CurrentFrame)
                    > UHCI_FRAME_LIST_MAX_ENTRIES as ULONG)
        {
            (*SOF_HcdTDs).__bindgen_anon_1.Frame = 0;
        }

        ix = ix.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciEnableInterrupts(uhciExtension: PVOID) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut LegacySupport: UHCI_PCI_LEGSUP = core::mem::zeroed();

    BaseRegister = (*UhciExtension).BaseRegister;

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        UhciExtension as PVOID,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
        PCI_LEGSUP as ULONG,
        core::mem::size_of::<USHORT>() as ULONG,
    );

    LegacySupport.set_UsbPIRQ(1 as USHORT);

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        UhciExtension as PVOID,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
        PCI_LEGSUP as ULONG,
        core::mem::size_of::<USHORT>() as ULONG,
    );

    WRITE_PORT_USHORT(
        core::ptr::addr_of_mut!((*BaseRegister).HcInterruptEnable.AsUSHORT),
        (*UhciExtension).StatusMask.AsUSHORT,
    );
}

#[no_mangle]
pub unsafe extern "system" fn UhciDisableInterrupts(uhciExtension: PVOID) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let HcFlavor: USB_CONTROLLER_FLAVOR;
    let mut LegacySupport: UHCI_PCI_LEGSUP = core::mem::zeroed();

    BaseRegister = (*UhciExtension).BaseRegister;
    WRITE_PORT_USHORT(
        core::ptr::addr_of_mut!((*BaseRegister).HcInterruptEnable.AsUSHORT),
        0 as USHORT,
    );

    HcFlavor = (*UhciExtension).HcFlavor;

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        UhciExtension as PVOID,
        1 as BOOLEAN,
        core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
        PCI_LEGSUP as ULONG,
        core::mem::size_of::<USHORT>() as ULONG,
    );

    LegacySupport.c2rust_unnamed.set_UsbPIRQ(0 as USHORT);

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        UhciExtension as PVOID,
        0 as BOOLEAN,
        core::ptr::addr_of_mut!(LegacySupport.AsUSHORT) as PVOID,
        PCI_LEGSUP as ULONG,
        core::mem::size_of::<USHORT>() as ULONG,
    );

    let _ = HcFlavor;
}

#[no_mangle]
pub unsafe extern "system" fn UhciPollController(uhciExtension: PVOID) {
    let UhciExtension: PUHCI_EXTENSION = uhciExtension as PUHCI_EXTENSION;
    let mut BaseRegister: PUHCI_HW_REGISTERS;
    let mut PortRegister: PUSHORT;
    let mut PortControl: UHCI_PORT_STATUS_CONTROL = core::mem::zeroed();
    let mut Port: USHORT;

    DPRINT_UHCI!(
        b"UhciPollController: UhciExtension - %p\n\0".as_ptr() as *const i8,
        UhciExtension,
    );

    BaseRegister = (*UhciExtension).BaseRegister;

    if (*UhciExtension).Flags as u32 & UHCI_EXTENSION_FLAG_SUSPENDED as u32 == 0 {
        UhciCleanupFrameList(UhciExtension, FALSE as BOOLEAN);
        UhciUpdateCounter(UhciExtension);
        (RegPacket.UsbPortInvalidateRootHub).expect("non-null function pointer")(
            UhciExtension as PVOID,
        );
        return;
    }

    Port = 0;
    while (Port as i32) < (UHCI_NUM_ROOT_HUB_PORTS as i32) {
        PortRegister = (core::ptr::addr_of_mut!((*BaseRegister).PortControl)
            as *mut UHCI_PORT_STATUS_CONTROL)
            .offset(Port as isize) as *mut UHCI_PORT_STATUS_CONTROL
            as PUSHORT;
        PortControl.AsUSHORT = READ_PORT_USHORT(PortRegister);

        if PortControl.ConnectStatusChange() as i32 == 1 {
            (RegPacket.UsbPortInvalidateRootHub).expect("non-null function pointer")(
                UhciExtension as PVOID,
            );
        }

        Port = Port.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn UhciSetEndpointDataToggle(
    uhciExtension: PVOID,
    uhciEndpoint: PVOID,
    DataToggle: ULONG,
) {
    DPRINT_IMPL!(b"UhciSetEndpointDataToggle: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciResetController(uhciExtension: PVOID) {
    DPRINT_IMPL!(b"UhciResetController: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciStartSendOnePacket(
    uhciExtension: PVOID,
    PacketParameters: PVOID,
    Data: PVOID,
    pDataLength: PULONG,
    BufferVA: PVOID,
    BufferPA: PVOID,
    BufferLength: ULONG,
    pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciStartSendOnePacket: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciEndSendOnePacket(
    uhciExtension: PVOID,
    PacketParameters: PVOID,
    Data: PVOID,
    pDataLength: PULONG,
    BufferVA: PVOID,
    BufferPA: PVOID,
    BufferLength: ULONG,
    pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciEndSendOnePacket: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciPassThru(
    uhciExtension: PVOID,
    passThruParameters: PVOID,
    ParameterLength: ULONG,
    pParameters: PVOID,
) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciPassThru: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn UhciFlushInterrupts(uhciExtension: PVOID) {
    DPRINT_IMPL!(b"UhciFlushInterrupts: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
}

#[no_mangle]
pub unsafe extern "system" fn UhciUnload(uhciExtension: PVOID) -> MPSTATUS {
    DPRINT_IMPL!(b"UhciUnload: UNIMPLEMENTED. FIXME\n\0".as_ptr() as *const i8);
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {

    RtlZeroMemory(
        core::ptr::addr_of_mut!(RegPacket) as PVOID,
        core::mem::size_of::<USBPORT_REGISTRATION_PACKET>(),
    );

    RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_UHCI as ULONG;

    RegPacket.MiniPortFlags = (USB_MINIPORT_FLAGS_INTERRUPT
        | USB_MINIPORT_FLAGS_PORT_IO
        | USB_MINIPORT_FLAGS_NOT_LOCK_INT
        | USB_MINIPORT_FLAGS_POLLING
        | USB_MINIPORT_FLAGS_WAKE_SUPPORT) as ULONG;

    RegPacket.MiniPortBusBandwidth = TOTAL_USB11_BUS_BANDWIDTH as ULONG;

    RegPacket.MiniPortExtensionSize = ((core::mem::size_of::<UHCI_EXTENSION>() as SIZE_T as u32) as _);
    RegPacket.MiniPortEndpointSize = ((core::mem::size_of::<UHCI_ENDPOINT>() as SIZE_T as u32) as _);
    RegPacket.MiniPortTransferSize = ((core::mem::size_of::<UHCI_TRANSFER>() as SIZE_T as u32) as _);
    RegPacket.MiniPortResourcesSize = ((core::mem::size_of::<UHCI_HC_RESOURCES>() as SIZE_T as u32) as _);

    RegPacket.OpenEndpoint = Some(UhciOpenEndpoint);
    RegPacket.ReopenEndpoint = Some(UhciReopenEndpoint);
    RegPacket.QueryEndpointRequirements = Some(UhciQueryEndpointRequirements);
    RegPacket.CloseEndpoint = Some(UhciCloseEndpoint);
    RegPacket.StartController = Some(UhciStartController);
    RegPacket.StopController = Some(UhciStopController);
    RegPacket.SuspendController = Some(UhciSuspendController);
    RegPacket.ResumeController = Some(UhciResumeController);
    RegPacket.InterruptService = Some(UhciInterruptService);
    RegPacket.InterruptDpc = Some(UhciInterruptDpc);
    RegPacket.SubmitTransfer = Some(UhciSubmitTransfer);
    RegPacket.SubmitIsoTransfer = Some(UhciIsochTransfer);
    RegPacket.AbortTransfer = Some(UhciAbortTransfer);
    RegPacket.GetEndpointState = Some(UhciGetEndpointState);
    RegPacket.SetEndpointState = Some(UhciSetEndpointState);
    RegPacket.PollEndpoint = Some(UhciPollEndpoint);
    RegPacket.CheckController = Some(UhciCheckController);
    RegPacket.Get32BitFrameNumber = Some(UhciGet32BitFrameNumber);
    RegPacket.InterruptNextSOF = Some(UhciInterruptNextSOF);
    RegPacket.EnableInterrupts = Some(UhciEnableInterrupts);
    RegPacket.DisableInterrupts = Some(UhciDisableInterrupts);
    RegPacket.PollController = Some(UhciPollController);
    RegPacket.SetEndpointDataToggle = Some(UhciSetEndpointDataToggle);
    RegPacket.GetEndpointStatus = Some(UhciGetEndpointStatus);
    RegPacket.SetEndpointStatus = Some(UhciSetEndpointStatus);
    RegPacket.RH_GetRootHubData = Some(UhciRHGetRootHubData);
    RegPacket.RH_GetStatus = Some(UhciRHGetStatus);
    RegPacket.RH_GetPortStatus = Some(UhciRHGetPortStatus);
    RegPacket.RH_GetHubStatus = Some(UhciRHGetHubStatus);
    RegPacket.RH_SetFeaturePortReset = Some(UhciRHSetFeaturePortReset);
    RegPacket.RH_SetFeaturePortPower = Some(UhciRHSetFeaturePortPower);
    RegPacket.RH_SetFeaturePortEnable = Some(UhciRHSetFeaturePortEnable);
    RegPacket.RH_SetFeaturePortSuspend = Some(UhciRHSetFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnable = Some(UhciRHClearFeaturePortEnable);
    RegPacket.RH_ClearFeaturePortPower = Some(UhciRHClearFeaturePortPower);
    RegPacket.RH_ClearFeaturePortSuspend = Some(UhciRHClearFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnableChange = Some(UhciRHClearFeaturePortEnableChange);
    RegPacket.RH_ClearFeaturePortConnectChange = Some(UhciRHClearFeaturePortConnectChange);
    RegPacket.RH_ClearFeaturePortResetChange = Some(UhciRHClearFeaturePortResetChange);
    RegPacket.RH_ClearFeaturePortSuspendChange = Some(UhciRHClearFeaturePortSuspendChange);
    RegPacket.RH_ClearFeaturePortOvercurrentChange = Some(UhciRHClearFeaturePortOvercurrentChange);
    RegPacket.RH_DisableIrq = Some(UhciRHDisableIrq);
    RegPacket.RH_EnableIrq = Some(UhciRHEnableIrq);
    RegPacket.StartSendOnePacket = Some(UhciStartSendOnePacket);
    RegPacket.EndSendOnePacket = Some(UhciEndSendOnePacket);
    RegPacket.PassThru = Some(UhciPassThru);
    RegPacket.FlushInterrupts = Some(UhciFlushInterrupts);

    (*DriverObject).DriverUnload = None;

    return USBPORT_RegisterUSBPortDriver(
        DriverObject,
        USB10_MINIPORT_INTERFACE_VERSION as ULONG,
        core::ptr::addr_of_mut!(RegPacket),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB UHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBUHCI main driver functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbuhci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_UHCI_TRACE
// #define NDEBUG_UHCI_IMPLEMENT
// #include "dbg_uhci.h"
//
// USBPORT_REGISTRATION_PACKET RegPacket;
//
// VOID
// NTAPI
// UhciDumpHcdQH(PUHCI_HCD_QH QH)
// {
//     DPRINT("QH              - %p\n", QH);
//     DPRINT("NextQH          - %p\n", QH->HwQH.NextQH);
//     DPRINT("NextElement     - %p\n", QH->HwQH.NextElement);
//
//     DPRINT("PhysicalAddress - %p\n", QH->PhysicalAddress);
//     DPRINT("QhFlags         - %X\n", QH->QhFlags);
//     DPRINT("NextHcdQH       - %X\n", QH->NextHcdQH);
//     DPRINT("PrevHcdQH       - %X\n", QH->PrevHcdQH);
//     DPRINT("UhciEndpoint    - %X\n", QH->UhciEndpoint);
// }
//
// VOID
// NTAPI
// UhciDumpHcdTD(PUHCI_HCD_TD TD)
// {
//     DPRINT("TD              - %p\n", TD);
//     DPRINT("NextElement     - %p\n", TD->HwTD.NextElement);
//     DPRINT("ControlStatus   - %08X\n", TD->HwTD.ControlStatus.AsULONG);
//     DPRINT("Token           - %p\n", TD->HwTD.Token.AsULONG);
//     if (TD->HwTD.Buffer)
//         DPRINT("Buffer          - %p\n", TD->HwTD.Buffer);
//
//     if (TD->SetupPacket.bmRequestType.B)
//         DPRINT("bmRequestType   - %02X\n", TD->SetupPacket.bmRequestType.B);
//     if (TD->SetupPacket.bRequest)
//         DPRINT("bRequest        - %02X\n", TD->SetupPacket.bRequest);
//     if (TD->SetupPacket.wValue.W)
//         DPRINT("wValue          - %04X\n", TD->SetupPacket.wValue.W);
//     if (TD->SetupPacket.wIndex.W)
//         DPRINT("wIndex          - %04X\n", TD->SetupPacket.wIndex.W);
//     if (TD->SetupPacket.wLength)
//         DPRINT("wLength         - %04X\n", TD->SetupPacket.wLength);
//
//     DPRINT("PhysicalAddress - %p\n", TD->PhysicalAddress);
//     DPRINT("Flags           - %X\n", TD->Flags);
//     DPRINT("NextHcdTD       - %p\n", TD->NextHcdTD);
//     DPRINT("UhciTransfer    - %p\n", TD->UhciTransfer);
// }
//
// VOID
// NTAPI
// UhciFixDataToggle(IN PUHCI_EXTENSION UhciExtension,
//                   IN PUHCI_ENDPOINT UhciEndpoint,
//                   IN PUHCI_HCD_TD TD,
//                   IN BOOL DataToggle)
// {
//     DPRINT_UHCI!("UhciFixDataToggle: UhciExtension - %p, UhciEndpoint - %p, DataToggle - %X\n",
//                 UhciExtension,
//                 UhciEndpoint,
//                 DataToggle);
//
//     while (TD)
//     {
//         TD->HwTD.Token.DataToggle = !TD->HwTD.Token.DataToggle;
//         DataToggle = !DataToggle;
//
//         TD = TD->NextHcdTD;
//     }
//
//     UhciEndpoint->DataToggle = DataToggle;
// }
//
// VOID
// NTAPI
// UhciCleanupFrameListEntry(IN PUHCI_EXTENSION UhciExtension,
//                           IN ULONG Index)
// {
//     PUHCI_HC_RESOURCES UhciResources;
//     ULONG PhysicalAddress;
//     ULONG HeadIdx;
//
//     UhciResources = UhciExtension->HcResourcesVA;
//
//     if (Index == 0)
//     {
//         PhysicalAddress = UhciExtension->StaticTD->PhysicalAddress;
//
//         UhciResources->FrameList[0] = PhysicalAddress |
//                                       UHCI_FRAME_LIST_POINTER_TD;
//     }
//     else
//     {
//         HeadIdx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) +
//                   (Index & (ENDPOINT_INTERRUPT_32ms - 1));
//
//         PhysicalAddress = UhciExtension->IntQH[HeadIdx]->PhysicalAddress;
//
//         UhciResources->FrameList[Index] = PhysicalAddress |
//                                           UHCI_FRAME_LIST_POINTER_QH;
//     }
// }
//
// VOID
// NTAPI
// UhciCleanupFrameList(IN PUHCI_EXTENSION UhciExtension,
//                      IN BOOLEAN IsAllEntries)
// {
//     ULONG NewFrameNumber;
//     ULONG OldFrameNumber;
//     ULONG ix;
//
//     DPRINT_UHCI!("UhciCleanupFrameList: [%p] All - %x\n",
//                 UhciExtension, IsAllEntries);
//
//     // FIXME: using UhciExtension->LockFrameList after supporting ISOs.
//
//     NewFrameNumber = UhciGet32BitFrameNumber(UhciExtension);
//     OldFrameNumber = UhciExtension->FrameNumber;
//
//     if ((NewFrameNumber - OldFrameNumber) < UHCI_FRAME_LIST_MAX_ENTRIES &&
//         IsAllEntries == FALSE)
//     {
//         for (ix = OldFrameNumber & UHCI_FRAME_LIST_INDEX_MASK;
//              ix != (NewFrameNumber & UHCI_FRAME_LIST_INDEX_MASK);
//              ix = (ix + 1) & UHCI_FRAME_LIST_INDEX_MASK)
//         {
//             UhciCleanupFrameListEntry(UhciExtension, ix);
//         }
//     }
//     else
//     {
//         for (ix = 0; ix < UHCI_FRAME_LIST_MAX_ENTRIES; ++ix)
//         {
//             UhciCleanupFrameListEntry(UhciExtension, ix);
//         }
//     }
//
//     UhciExtension->FrameNumber = NewFrameNumber;
// }
//
// VOID
// NTAPI
// UhciUpdateCounter(IN PUHCI_EXTENSION UhciExtension)
// {
//     ULONG FrameNumber;
//
//     FrameNumber = READ_PORT_USHORT(&UhciExtension->BaseRegister->FrameNumber);
//     FrameNumber &= UHCI_FRNUM_FRAME_MASK;
//
//     if ((FrameNumber ^ UhciExtension->FrameHighPart) & UHCI_FRNUM_OVERFLOW_LIST)
//     {
//         UhciExtension->FrameHighPart += UHCI_FRAME_LIST_MAX_ENTRIES;
//
//         DPRINT_UHCI!("UhciUpdateCounter: UhciExtension->FrameHighPart - %lX\n",
//                     UhciExtension->FrameHighPart);
//     }
// }
//
// VOID
// NTAPI
// UhciSetNextQH(IN PUHCI_HCD_QH QH,
//               IN PUHCI_HCD_QH NextQH)
// {
//     DPRINT_UHCI!("UhciSetNextQH: QH - %p, NextQH - %p\n", QH, NextQH);
//
//     QH->HwQH.NextQH = NextQH->PhysicalAddress | UHCI_QH_HEAD_LINK_PTR_QH;
//     QH->NextHcdQH = NextQH;
//
//     NextQH->PrevHcdQH = QH;
//     NextQH->QhFlags |= UHCI_HCD_QH_FLAG_ACTIVE;
// }
//
// MPSTATUS
// NTAPI
// UhciOpenEndpoint(IN PVOID uhciExtension,
//                  IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                  IN PVOID uhciEndpoint)
// {
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     ULONG TransferType;
//     ULONG_PTR BufferVA;
//     ULONG BufferPA;
//     ULONG ix;
//     ULONG TdCount;
//     PUHCI_HCD_TD TD;
//     SIZE_T BufferLength;
//     PUHCI_HCD_QH QH;
//
//     RtlCopyMemory(&UhciEndpoint->EndpointProperties,
//                   EndpointProperties,
//                   sizeof(UhciEndpoint->EndpointProperties));
//
//     InitializeListHead(&UhciEndpoint->ListTDs);
//
//     UhciEndpoint->EndpointLock = 0;
//     UhciEndpoint->DataToggle = UHCI_TD_PID_DATA0;
//     UhciEndpoint->Flags = 0;
//
//     TransferType = EndpointProperties->TransferType;
//
//     DPRINT("UhciOpenEndpoint: UhciEndpoint - %p, TransferType - %x\n",
//            UhciEndpoint,
//            TransferType);
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         UhciEndpoint->Flags |= UHCI_ENDPOINT_FLAG_CONTROL_OR_ISO;
//     }
//
//     BufferVA = EndpointProperties->BufferVA;
//     BufferPA = EndpointProperties->BufferPA;
//
//     BufferLength = EndpointProperties->BufferLength;
//
//     /* For Isochronous transfers not used QHs (only TDs) */
//     if (EndpointProperties->TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         /* Initialize HCD Queue Head */
//         UhciEndpoint->QH = (PUHCI_HCD_QH)BufferVA;
//
//         QH = UhciEndpoint->QH;
//
//         QH->HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         QH->PhysicalAddress = BufferPA;
//
//         QH->NextHcdQH = QH;
//         QH->PrevHcdQH = QH;
//         QH->UhciEndpoint = UhciEndpoint;
//
//         BufferVA += sizeof(UHCI_HCD_QH);
//         BufferPA += sizeof(UHCI_HCD_QH);
//
//         BufferLength -= sizeof(UHCI_HCD_QH);
//     }
//
//     /* Initialize HCD Transfer Descriptors */
//     TdCount = BufferLength / sizeof(UHCI_HCD_TD);
//     UhciEndpoint->MaxTDs = TdCount;
//
//     UhciEndpoint->FirstTD = (PUHCI_HCD_TD)BufferVA;
//     UhciEndpoint->AllocatedTDs = 0;
//
//     RtlZeroMemory(UhciEndpoint->FirstTD, TdCount * sizeof(UHCI_HCD_TD));
//
//     for (ix = 0; ix < UhciEndpoint->MaxTDs; ix++)
//     {
//         TD = &UhciEndpoint->FirstTD[ix];
//         TD->PhysicalAddress = BufferPA;
//         BufferPA += sizeof(UHCI_HCD_TD);
//     }
//
//     UhciEndpoint->TailTD = NULL;
//     UhciEndpoint->HeadTD = NULL;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciReopenEndpoint(IN PVOID uhciExtension,
//                    IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                    IN PVOID uhciEndpoint)
// {
//     DPRINT_IMPL!("Uhci: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciQueryEndpointRequirements(IN PVOID uhciExtension,
//                               IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                               IN PUSBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements)
// {
//     ULONG TransferType;
//     ULONG TdCount;
//
//     DPRINT("UhciQueryEndpointRequirements: ... \n");
//
//     TransferType = EndpointProperties->TransferType;
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             DPRINT("UhciQueryEndpointRequirements: IsoTransfer\n");
//             TdCount = 2 * UHCI_MAX_ISO_TD_COUNT;
//
//             EndpointRequirements->HeaderBufferSize = 0 + // Iso queue is have not Queue Heads
//                                                      TdCount * sizeof(UHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = UHCI_MAX_ISO_TRANSFER_SIZE;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             DPRINT("UhciQueryEndpointRequirements: ControlTransfer\n");
//             TdCount = EndpointProperties->MaxTransferSize /
//                       EndpointProperties->TotalMaxPacketSize;
//
//             TdCount += 2; // First + Last TDs
//
//             EndpointRequirements->HeaderBufferSize = sizeof(UHCI_HCD_QH) +
//                                                      TdCount * sizeof(UHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = EndpointProperties->MaxTransferSize;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             DPRINT("UhciQueryEndpointRequirements: BulkTransfer\n");
//             TdCount = 2 * UHCI_MAX_BULK_TRANSFER_SIZE /
//                       EndpointProperties->TotalMaxPacketSize;
//
//             EndpointRequirements->HeaderBufferSize = sizeof(UHCI_HCD_QH) +
//                                                      TdCount * sizeof(UHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = UHCI_MAX_BULK_TRANSFER_SIZE;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             DPRINT("UhciQueryEndpointRequirements: InterruptTransfer\n");
//             TdCount = 2 * UHCI_MAX_INTERRUPT_TD_COUNT;
//
//             EndpointRequirements->HeaderBufferSize = sizeof(UHCI_HCD_QH) +
//                                                      TdCount * sizeof(UHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = UHCI_MAX_INTERRUPT_TD_COUNT *
//                                                     EndpointProperties->TotalMaxPacketSize;
//             break;
//
//         default:
//             DPRINT1("UhciQueryEndpointRequirements: Unknown TransferType - %x\n",
//                     TransferType);
//             DbgBreakPoint();
//             break;
//     }
// }
//
// VOID
// NTAPI
// UhciCloseEndpoint(IN PVOID uhciExtension,
//                   IN PVOID uhciEndpoint,
//                   IN BOOLEAN IsDoDisablePeriodic)
// {
//     DPRINT_IMPL!("UhciCloseEndpoint: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// UhciTakeControlHC(IN PUHCI_EXTENSION UhciExtension,
//                   IN PUSBPORT_RESOURCES Resources)
// {
//     LARGE_INTEGER EndTime;
//     LARGE_INTEGER CurrentTime;
//     ULONG ResourcesTypes;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT StatusRegister;
//     UHCI_PCI_LEGSUP LegacySupport;
//     UHCI_PCI_LEGSUP LegacyMask;
//     UHCI_USB_COMMAND Command;
//     UHCI_USB_STATUS HcStatus;
//     MPSTATUS MpStatus = MP_STATUS_SUCCESS;
//
//     DPRINT("UhciTakeControlHC: Resources->ResourcesTypes - %x\n",
//            Resources->ResourcesTypes);
//
//     ResourcesTypes = Resources->ResourcesTypes;
//
//     if ((ResourcesTypes & (USBPORT_RESOURCES_PORT | USBPORT_RESOURCES_INTERRUPT)) !=
//                           (USBPORT_RESOURCES_PORT | USBPORT_RESOURCES_INTERRUPT))
//     {
//         DPRINT1("UhciTakeControlHC: MP_STATUS_ERROR\n");
//         MpStatus = MP_STATUS_ERROR;
//     }
//
//     BaseRegister = UhciExtension->BaseRegister;
//     StatusRegister = &BaseRegister->HcStatus.AsUSHORT;
//
//     RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                           TRUE,
//                                           &LegacySupport.AsUSHORT,
//                                           PCI_LEGSUP,
//                                           sizeof(USHORT));
//
//     UhciDisableInterrupts(UhciExtension);
//
//     Command.AsUSHORT = READ_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT);
//
//     Command.Run = 0;
//     Command.GlobalReset = 0;
//     Command.ConfigureFlag = 0;
//
//     WRITE_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT, Command.AsUSHORT);
//
//     KeQuerySystemTime(&EndTime);
//     EndTime.QuadPart += 100 * 10000; // 100 ms
//
//     HcStatus.AsUSHORT = READ_PORT_USHORT(StatusRegister);
//     DPRINT("UhciTakeControlHC: HcStatus.AsUSHORT - %04X\n", HcStatus.AsUSHORT);
//
//     while (HcStatus.HcHalted == 0)
//     {
//         HcStatus.AsUSHORT = READ_PORT_USHORT(StatusRegister);
//         KeQuerySystemTime(&CurrentTime);
//
//         if (CurrentTime.QuadPart >= EndTime.QuadPart)
//             break;
//     }
//
//     WRITE_PORT_USHORT(StatusRegister, UHCI_USB_STATUS_MASK);
//
//     LegacyMask.AsUSHORT = 0;
//     LegacyMask.Smi60Read = 1;
//     LegacyMask.Smi60Write = 1;
//     LegacyMask.Smi64Read = 1;
//     LegacyMask.Smi64Write = 1;
//     LegacyMask.SmiIrq = 1;
//     LegacyMask.A20Gate = 1;
//     LegacyMask.SmiEndPassThrough = 1;
//
//     if (LegacySupport.AsUSHORT & LegacyMask.AsUSHORT)
//     {
//         DPRINT("UhciTakeControlHC: LegacySupport.AsUSHORT - %04X\n",
//                LegacySupport.AsUSHORT);
//
//         Resources->LegacySupport = 1;
//
//         RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                               TRUE,
//                                               &LegacySupport.AsUSHORT,
//                                               PCI_LEGSUP,
//                                               sizeof(USHORT));
//         LegacySupport.AsUSHORT = 0;
//
//         RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                               FALSE,
//                                               &LegacySupport.AsUSHORT,
//                                               PCI_LEGSUP,
//                                               sizeof(USHORT));
//     }
//
//     return MpStatus;
// }
//
// MPSTATUS
// NTAPI
// UhciInitializeHardware(IN PUHCI_EXTENSION UhciExtension)
// {
//     PUHCI_HW_REGISTERS BaseRegister;
//     UHCI_USB_COMMAND Command;
//     UHCI_USB_STATUS StatusMask;
//
//     DPRINT("UhciInitializeHardware: UhciExtension - %p\n", UhciExtension);
//     DPRINT("UhciInitializeHardware: VIA HW FIXME\n"); // after supporting HcFlavor in usbport
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     /* Save SOF Timing Value */
//     UhciExtension->SOF_Modify = READ_PORT_UCHAR(&BaseRegister->SOF_Modify);
//
//     RegPacket.UsbPortWait(UhciExtension, 20);
//
//     Command.AsUSHORT = READ_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT);
//
//     /* Global Reset */
//     Command.AsUSHORT = 0;
//     Command.GlobalReset = 1;
//     WRITE_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT, Command.AsUSHORT);
//
//     RegPacket.UsbPortWait(UhciExtension, 20);
//
//     Command.AsUSHORT = 0;
//     WRITE_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT, Command.AsUSHORT);
//
//     /* Set MaxPacket for full speed bandwidth reclamation */
//     Command.AsUSHORT = 0;
//     Command.MaxPacket = 1; // 64 bytes
//     WRITE_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT, Command.AsUSHORT);
//
//     /* Restore SOF Timing Value */
//     WRITE_PORT_UCHAR(&BaseRegister->SOF_Modify, UhciExtension->SOF_Modify);
//
//     StatusMask = UhciExtension->StatusMask;
//
//     StatusMask.Interrupt = 1;
//     StatusMask.ErrorInterrupt = 1;
//     StatusMask.ResumeDetect = 1;
//     StatusMask.HostSystemError = 1;
//
//     UhciExtension->StatusMask = StatusMask;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciInitializeSchedule(IN PUHCI_EXTENSION UhciExtension,
//                        IN PUHCI_HC_RESOURCES HcResourcesVA,
//                        IN ULONG hcResourcesPA)
// {
//     PUHCI_HCD_QH IntQH;
//     ULONG IntQhPA;
//     PUHCI_HCD_QH StaticControlHead;
//     ULONG StaticControlHeadPA;
//     PUHCI_HCD_QH StaticBulkHead;
//     ULONG StaticBulkHeadPA;
//     PUHCI_HCD_TD StaticBulkTD;
//     ULONG StaticBulkTdPA;
//     PUHCI_HCD_TD StaticTD;
//     ULONG StaticTdPA;
//     PUHCI_HCD_TD StaticSofTD;
//     ULONG StaticSofTdPA;
//     ULONG PhysicalAddress;
//     ULONG Idx;
//     ULONG HeadIdx;
//     UCHAR FrameIdx;
//
//     DPRINT("UhciInitializeSchedule: Ext[%p], VA - %p, PA - %lx\n",
//            UhciExtension,
//            HcResourcesVA,
//            hcResourcesPA);
//
//     /* Build structure (tree) of static QHs
//        for interrupt and isochronous transfers */
//     for (FrameIdx = 0; FrameIdx < INTERRUPT_ENDPOINTs; FrameIdx++)
//     {
//         IntQH = &HcResourcesVA->StaticIntHead[FrameIdx];
//         IntQhPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticIntHead[FrameIdx]);
//
//         RtlZeroMemory(IntQH, sizeof(UHCI_HCD_QH));
//
//         IntQH->HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         IntQH->PhysicalAddress = IntQhPA;
//
//         UhciExtension->IntQH[FrameIdx] = IntQH;
//
//         if (FrameIdx == 0)
//             UhciSetNextQH(IntQH, UhciExtension->IntQH[0]);
//         else
//             UhciSetNextQH(IntQH, UhciExtension->IntQH[(FrameIdx - 1) / 2]);
//     }
//
//     /* Initialize static QH for control transfers */
//     StaticControlHead = &HcResourcesVA->StaticControlHead;
//     StaticControlHeadPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticControlHead);
//
//     RtlZeroMemory(StaticControlHead, sizeof(UHCI_HCD_QH));
//
//     StaticControlHead->HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//     StaticControlHead->PhysicalAddress = StaticControlHeadPA;
//
//     UhciSetNextQH(UhciExtension->IntQH[0],StaticControlHead);
//
//     UhciExtension->ControlQH = StaticControlHead;
//
//     /* Initialize static QH for bulk transfers */
//     StaticBulkHead = &HcResourcesVA->StaticBulkHead;
//     StaticBulkHeadPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticBulkHead);
//
//     RtlZeroMemory(StaticBulkHead, sizeof(UHCI_HCD_QH));
//
//     StaticBulkHead->PhysicalAddress = StaticBulkHeadPA;
//     PhysicalAddress = StaticBulkHeadPA | UHCI_QH_ELEMENT_LINK_PTR_QH;
//     PhysicalAddress |= UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//     StaticBulkHead->HwQH.NextQH = PhysicalAddress;
//
//     UhciSetNextQH(StaticControlHead, StaticBulkHead);
//
//     UhciExtension->BulkQH = StaticBulkHead;
//     UhciExtension->BulkTailQH = StaticBulkHead;
//
//     /* Initialize static TD for bulk transfers */
//     StaticBulkTD = &HcResourcesVA->StaticBulkTD;
//     StaticBulkTdPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticBulkTD);
//
//     StaticBulkTD->HwTD.NextElement = StaticBulkTdPA | UHCI_TD_LINK_PTR_TD;
//
//     StaticBulkTD->HwTD.ControlStatus.AsULONG = 0;
//     StaticBulkTD->HwTD.ControlStatus.IsochronousType = 1;
//
//     StaticBulkTD->HwTD.Token.AsULONG = 0;
//     StaticBulkTD->HwTD.Token.Endpoint = 1;
//     StaticBulkTD->HwTD.Token.MaximumLength = UHCI_TD_LENGTH_NULL;
//     StaticBulkTD->HwTD.Token.PIDCode = UHCI_TD_PID_OUT;
//
//     StaticBulkTD->HwTD.Buffer = 0;
//
//     StaticBulkTD->PhysicalAddress = StaticBulkTdPA;
//     StaticBulkTD->NextHcdTD = NULL;
//     StaticBulkTD->Flags = UHCI_HCD_TD_FLAG_PROCESSED;
//
//     PhysicalAddress = StaticBulkTdPA | UHCI_QH_ELEMENT_LINK_PTR_TD;
//     UhciExtension->BulkQH->HwQH.NextElement = PhysicalAddress;
//
//     /* Set Frame List pointers */
//     for (Idx = 0; Idx < UHCI_FRAME_LIST_MAX_ENTRIES; Idx++)
//     {
//         HeadIdx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms) +
//                   (Idx & (ENDPOINT_INTERRUPT_32ms - 1));
//
//         PhysicalAddress = UhciExtension->IntQH[HeadIdx]->PhysicalAddress;
//         PhysicalAddress |= UHCI_FRAME_LIST_POINTER_QH;
//         HcResourcesVA->FrameList[Idx] = PhysicalAddress;
//     }
//
//     /* Initialize static TD for first frame */
//     StaticTD = &HcResourcesVA->StaticTD;
//     StaticTdPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticTD);
//
//     RtlZeroMemory(StaticTD, sizeof(UHCI_HCD_TD));
//
//     StaticTD->PhysicalAddress = StaticTdPA;
//
//     HeadIdx = (INTERRUPT_ENDPOINTs - ENDPOINT_INTERRUPT_32ms);
//     PhysicalAddress = UhciExtension->IntQH[HeadIdx]->PhysicalAddress;
//     StaticTD->HwTD.NextElement = PhysicalAddress | UHCI_TD_LINK_PTR_QH;
//
//     StaticTD->HwTD.ControlStatus.InterruptOnComplete = 1;
//     StaticTD->HwTD.Token.PIDCode = UHCI_TD_PID_IN;
//
//     UhciExtension->StaticTD = StaticTD;
//
//     /* Initialize StaticSofTDs for UhciInterruptNextSOF() */
//     UhciExtension->SOF_HcdTDs = &HcResourcesVA->StaticSofTD[0];
//     StaticSofTdPA = hcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, StaticSofTD[0]);
//
//     for (Idx = 0; Idx < UHCI_MAX_STATIC_SOF_TDS; Idx++)
//     {
//         StaticSofTD = UhciExtension->SOF_HcdTDs + Idx;
//
//         RtlZeroMemory(StaticSofTD, sizeof(UHCI_HCD_TD));
//
//         PhysicalAddress = UhciExtension->IntQH[HeadIdx]->PhysicalAddress;
//         StaticSofTD->HwTD.NextElement = PhysicalAddress | UHCI_TD_LINK_PTR_QH;
//
//         StaticSofTD->HwTD.ControlStatus.InterruptOnComplete = 1;
//         StaticSofTD->PhysicalAddress = StaticSofTdPA;
//
//         StaticSofTdPA += sizeof(UHCI_HCD_TD);
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciStartController(IN PVOID uhciExtension,
//                     IN PUSBPORT_RESOURCES Resources)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     MPSTATUS MpStatus;
//     PUSHORT PortControlRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//     UHCI_USB_COMMAND Command;
//     USHORT Port;
//
//     UhciExtension->Flags &= ~UHCI_EXTENSION_FLAG_SUSPENDED;
//     UhciExtension->BaseRegister = Resources->ResourceBase;
//     BaseRegister = UhciExtension->BaseRegister;
//     DPRINT("UhciStartController: UhciExtension - %p, BaseRegister - %p\n", UhciExtension, BaseRegister);
//
//     UhciExtension->HcFlavor = Resources->HcFlavor;
//
//     MpStatus = UhciTakeControlHC(UhciExtension, Resources);
//
//     if (MpStatus == MP_STATUS_SUCCESS)
//     {
//         MpStatus = UhciInitializeHardware(UhciExtension);
//
//         if (MpStatus == MP_STATUS_SUCCESS)
//         {
//             UhciExtension->HcResourcesVA = (PUHCI_HC_RESOURCES)Resources->StartVA;
//             UhciExtension->HcResourcesPA = Resources->StartPA;
//
//             MpStatus = UhciInitializeSchedule(UhciExtension,
//                                               UhciExtension->HcResourcesVA,
//                                               UhciExtension->HcResourcesPA);
//
//             UhciExtension->LockFrameList = 0;
//         }
//     }
//
//     WRITE_PORT_ULONG(&BaseRegister->FrameAddress,
//                      UhciExtension->HcResourcesPA + FIELD_OFFSET(UHCI_HC_RESOURCES, FrameList));
//
//     if (MpStatus == MP_STATUS_SUCCESS)
//     {
//         Command.AsUSHORT = READ_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT);
//         Command.Run = 1;
//         WRITE_PORT_USHORT(&BaseRegister->HcCommand.AsUSHORT, Command.AsUSHORT);
//
//         for (Port = 0; Port < UHCI_NUM_ROOT_HUB_PORTS; Port++)
//         {
//             PortControlRegister = &BaseRegister->PortControl[Port].AsUSHORT;
//             PortControl.AsUSHORT = READ_PORT_USHORT(PortControlRegister);
//
//             PortControl.ConnectStatusChange = 0;
//             PortControl.Suspend = 0;
//
//             WRITE_PORT_USHORT(PortControlRegister, PortControl.AsUSHORT);
//         }
//
//         UhciExtension->HcResourcesVA->FrameList[0] =
//             UhciExtension->StaticTD->PhysicalAddress;
//     }
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciStopController(IN PVOID uhciExtension,
//                    IN BOOLEAN IsDoDisableInterrupts)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT CommandReg;
//     UHCI_USB_COMMAND Command;
//     LARGE_INTEGER EndTime;
//     LARGE_INTEGER CurrentTime;
//
//     DPRINT("UhciStopController: UhciExtension - %p\n", UhciExtension);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     CommandReg = &BaseRegister->HcCommand.AsUSHORT;
//
//     Command.AsUSHORT = READ_PORT_USHORT(CommandReg);
//
//     if (Command.AsUSHORT == 0xFFFF)
//     {
//         DPRINT("UhciStopController: Command == -1\n");
//         return;
//     }
//
//     DPRINT("UhciStopController: Command.AsUSHORT - %p\n", Command.AsUSHORT);
//
//     if (Command.GlobalReset)
//     {
//         Command.GlobalReset = 0;
//         WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);
//     }
//
//     Command.HcReset = 1;
//
//     WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);
//
//     KeQuerySystemTime(&EndTime);
//     EndTime.QuadPart += 100 * 1000;
//
//     while (Command.AsUSHORT = READ_PORT_USHORT(CommandReg),
//            Command.HcReset == 1)
//     {
//         KeQuerySystemTime(&CurrentTime);
//
//         if (CurrentTime.QuadPart >= EndTime.QuadPart)
//         {
//             DPRINT1("UhciStopController: Failed to reset\n");
//             DbgBreakPoint();
//             break;
//         }
//     }
// }
//
// VOID
// NTAPI
// UhciSuspendController(IN PVOID uhciExtension)
// {
//     DPRINT_IMPL!("UhciSuspendController: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// UhciResumeController(IN PVOID uhciExtension)
// {
//     DPRINT_IMPL!("UhciResumeController: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// UhciHardwarePresent(IN PUHCI_EXTENSION UhciExtension)
// {
//     UHCI_USB_STATUS UhciStatus;
//     PUSHORT StatusReg;
//
//     StatusReg = &UhciExtension->BaseRegister->HcStatus.AsUSHORT;
//     UhciStatus.AsUSHORT = READ_PORT_USHORT(StatusReg);
//
//     if (UhciStatus.AsUSHORT == MAXUSHORT)
//         DPRINT_UHCI!("UhciHardwarePresent: HW not present\n");
//
//     return UhciStatus.AsUSHORT != MAXUSHORT;
// }
//
// BOOLEAN
// NTAPI
// UhciInterruptService(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT CommandReg;
//     UHCI_USB_COMMAND Command;
//     PUSHORT StatusReg;
//     UHCI_USB_STATUS HcStatus;
//     PUSHORT InterruptEnableReg;
//     PUHCI_HCD_QH QH;
//     PUHCI_HCD_QH BulkTailQH;
//     ULONG ScheduleError;
//     BOOLEAN Result = FALSE;
//
//     BaseRegister = UhciExtension->BaseRegister;
//     StatusReg = &BaseRegister->HcStatus.AsUSHORT;
//     InterruptEnableReg = &BaseRegister->HcInterruptEnable.AsUSHORT;
//     CommandReg = &BaseRegister->HcCommand.AsUSHORT;
//
//     if (!UhciHardwarePresent(UhciExtension))
//     {
//         DPRINT1("UhciInterruptService: return FALSE\n");
//         return FALSE;
//     }
//
//     HcStatus.AsUSHORT = READ_PORT_USHORT(StatusReg) & UHCI_USB_STATUS_MASK;
//
//     if (HcStatus.HostSystemError || HcStatus.HcProcessError)
//     {
//         DPRINT1("UhciInterruptService: Error [%p] HcStatus %X\n",
//                 UhciExtension,
//                 HcStatus.AsUSHORT);
//     }
//     else if (HcStatus.AsUSHORT)
//     {
//         UhciExtension->HcScheduleError = 0;
//     }
//
//     if (HcStatus.HcProcessError)
//     {
//         USHORT fn = READ_PORT_USHORT(&BaseRegister->FrameNumber);
//         USHORT intr = READ_PORT_USHORT(InterruptEnableReg);
//         USHORT cmd = READ_PORT_USHORT(CommandReg);
//
//         DPRINT1("UhciInterruptService: HC ProcessError!\n");
//         DPRINT1("UhciExtension %p, frame %X\n", UhciExtension, fn);
//         DPRINT1("HcCommand %X\n", cmd);
//         DPRINT1("HcStatus %X\n", HcStatus.AsUSHORT);
//         DPRINT1("HcInterruptEnable %X\n", intr);
//
//         DbgBreakPoint();
//     }
//
//     if (HcStatus.HcHalted)
//     {
//         DPRINT_UHCI!("UhciInterruptService: Hc Halted [%p] HcStatus %X\n",
//                     UhciExtension,
//                     HcStatus.AsUSHORT);
//     }
//     else if (HcStatus.AsUSHORT)
//     {
//         UhciExtension->HcStatus.AsUSHORT = HcStatus.AsUSHORT;
//
//         WRITE_PORT_USHORT(StatusReg, HcStatus.AsUSHORT);
//         WRITE_PORT_USHORT(InterruptEnableReg, 0);
//
//         if (HcStatus.HostSystemError)
//         {
//             DPRINT1("UhciInterruptService: HostSystemError! HcStatus - %X\n",
//                     HcStatus.AsUSHORT);
//
//             DbgBreakPoint();
//         }
//
//         Result = TRUE;
//     }
//
//     if (!HcStatus.Interrupt)
//         goto NextProcess;
//
//     UhciUpdateCounter(UhciExtension);
//
//     BulkTailQH = UhciExtension->BulkTailQH;
//
//     if (BulkTailQH->HwQH.NextQH & UHCI_QH_HEAD_LINK_PTR_TERMINATE)
//         goto NextProcess;
//
//     QH = UhciExtension->BulkQH;
//
//     do
//     {
//         QH = QH->NextHcdQH;
//
//         if (!QH)
//         {
//             BulkTailQH->HwQH.NextQH |= UHCI_QH_HEAD_LINK_PTR_TERMINATE;
//             goto NextProcess;
//         }
//     }
//     while (QH->HwQH.NextElement & UHCI_QH_ELEMENT_LINK_PTR_TERMINATE);
//
// NextProcess:
//
//     if (HcStatus.HcProcessError)
//     {
//         UhciCleanupFrameList(UhciExtension, TRUE);
//
//         ScheduleError = UhciExtension->HcScheduleError;
//         UhciExtension->HcScheduleError = ScheduleError + 1;
//
//         DPRINT1("UhciInterruptService: [%p] ScheduleError %X\n",
//                 UhciExtension,
//                 ScheduleError);
//
//         if (ScheduleError < UHCI_MAX_HC_SCHEDULE_ERRORS)
//         {
//             Command.AsUSHORT = READ_PORT_USHORT(CommandReg);
//             Command.Run = 1;
//             WRITE_PORT_USHORT(CommandReg, Command.AsUSHORT);
//         }
//     }
//     else if (HcStatus.Interrupt && UhciExtension->ExtensionLock)
//     {
//         DPRINT1("UhciInterruptService: [%p] HcStatus %X\n",
//                 UhciExtension,
//                 HcStatus.AsUSHORT);
//
//         UhciCleanupFrameList(UhciExtension, FALSE);
//     }
//
//     return Result;
// }
//
// VOID
// NTAPI
// UhciInterruptDpc(IN PVOID uhciExtension,
//                  IN BOOLEAN IsDoEnableInterrupts)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     UHCI_USB_STATUS HcStatus;
//
//     DPRINT_UHCI!("UhciInterruptDpc: [%p] EnableInt %x, HcStatus %X\n",
//                 uhciExtension, IsDoEnableInterrupts, UhciExtension->HcStatus);
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     HcStatus = UhciExtension->HcStatus;
//     UhciExtension->HcStatus.AsUSHORT = 0;
//
//     if ((HcStatus.Interrupt | HcStatus.ErrorInterrupt) != 0)
//         RegPacket.UsbPortInvalidateEndpoint(UhciExtension, 0);
//
//     if (IsDoEnableInterrupts)
//     {
//         WRITE_PORT_USHORT(&BaseRegister->HcInterruptEnable.AsUSHORT,
//                           UhciExtension->StatusMask.AsUSHORT);
//     }
// }
//
// VOID
// NTAPI
// UhciQueueTransfer(IN PUHCI_EXTENSION UhciExtension,
//                   IN PUHCI_ENDPOINT UhciEndpoint,
//                   IN PUHCI_HCD_TD FirstTD,
//                   IN PUHCI_HCD_TD LastTD)
// {
//     PUHCI_HCD_QH QH;
//     PUHCI_HCD_QH BulkTailQH;
//     PUHCI_HCD_TD TailTD;
//     ULONG PhysicalAddress;
//
//     DPRINT("UhciQueueTransfer: FirstTD - %p, LastTD - %p\n", FirstTD, LastTD);
//
//     TailTD = UhciEndpoint->TailTD;
//     QH = UhciEndpoint->QH;
//
//     if (UhciEndpoint->HeadTD)
//     {
//         TailTD->NextHcdTD = FirstTD;
//
//         TailTD->HwTD.NextElement = FirstTD->PhysicalAddress;
//         TailTD->HwTD.NextElement |= UHCI_TD_LINK_PTR_TD;
//
//         PhysicalAddress = QH->HwQH.NextElement;
//
//         PhysicalAddress &= ~(UHCI_TD_LINK_PTR_TERMINATE |
//                              UHCI_TD_LINK_PTR_QH |
//                              UHCI_TD_LINK_PTR_DEPTH_FIRST);
//
//         if (FirstTD->HwTD.ControlStatus.Status & UHCI_TD_STS_ACTIVE)
//         {
//             if (PhysicalAddress == TailTD->PhysicalAddress &&
//                 !(TailTD->HwTD.ControlStatus.Status & UHCI_TD_STS_ACTIVE))
//             {
//                 QH->HwQH.NextElement = FirstTD->PhysicalAddress;
//
//                 QH->HwQH.NextElement &= ~(UHCI_QH_ELEMENT_LINK_PTR_TERMINATE |
//                                           UHCI_QH_ELEMENT_LINK_PTR_QH);
//             }
//         }
//     }
//     else
//     {
//         if (FirstTD)
//         {
//             UhciEndpoint->HeadTD = FirstTD;
//         }
//         else
//         {
//             UhciEndpoint->TailTD = NULL;
//             UhciEndpoint->HeadTD = NULL;
//         }
//
//         if (FirstTD == NULL || UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//         {
//             PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         }
//         else
//         {
//             PhysicalAddress = FirstTD->PhysicalAddress;
//             PhysicalAddress &= ~UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         }
//
//         QH->HwQH.NextElement = PhysicalAddress & ~UHCI_QH_ELEMENT_LINK_PTR_QH;
//     }
//
//     if (UhciEndpoint->EndpointProperties.TransferType == USBPORT_TRANSFER_TYPE_BULK)
//     {
//         BulkTailQH = UhciExtension->BulkTailQH;
//         BulkTailQH->HwQH.NextQH &= ~UHCI_QH_HEAD_LINK_PTR_TERMINATE;
//     }
//
//     UhciEndpoint->TailTD = LastTD;
// }
//
// PUHCI_HCD_TD
// NTAPI
// UhciAllocateTD(IN PUHCI_EXTENSION UhciExtension,
//                IN PUHCI_ENDPOINT UhciEndpoint)
// {
//     PUHCI_HCD_TD TD;
//     ULONG AllocTdCounter;
//     ULONG ix;
//
//     DPRINT_UHCI!("UhciAllocateTD: ...\n");
//
//     AllocTdCounter = UhciEndpoint->AllocTdCounter;
//
//     for (ix = 0; ix < UhciEndpoint->MaxTDs; ++ix)
//     {
//         TD = &UhciEndpoint->FirstTD[AllocTdCounter];
//
//         if (!(TD->Flags & UHCI_HCD_TD_FLAG_ALLOCATED))
//         {
//             TD->Flags |= UHCI_HCD_TD_FLAG_ALLOCATED;
//
//             UhciEndpoint->AllocatedTDs++;
//             UhciEndpoint->AllocTdCounter = AllocTdCounter;
//
//             return TD;
//         }
//
//         if (AllocTdCounter < UhciEndpoint->MaxTDs - 1)
//             AllocTdCounter++;
//         else
//             AllocTdCounter = 0;
//     }
//
//     return NULL;
// }
//
// VOID
// NTAPI
// UhciMapAsyncTransferToTDs(IN PUHCI_EXTENSION UhciExtension,
//                           IN PUHCI_ENDPOINT UhciEndpoint,
//                           IN PUHCI_TRANSFER UhciTransfer,
//                           OUT PUHCI_HCD_TD * OutFirstTD,
//                           OUT PUHCI_HCD_TD * OutLastTD,
//                           IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PUHCI_HCD_TD TD;
//     PUHCI_HCD_TD LastTD = NULL;
//     ULONG PhysicalAddress;
//     USHORT TotalMaxPacketSize;
//     USHORT DeviceSpeed;
//     USHORT EndpointAddress;
//     USHORT DeviceAddress;
//     ULONG TransferType;
//     SIZE_T TransferLength = 0;
//     SIZE_T LengthMapped = 0;
//     SIZE_T BytesRemaining;
//     SIZE_T LengthThisTD;
//     ULONG ix;
//     BOOL DataToggle;
//     UCHAR PIDCode;
//     BOOLEAN IsLastTd = TRUE;
//     BOOLEAN ZeroLengthTransfer = TRUE;
//
//     DPRINT_UHCI!("UhciMapAsyncTransferToTDs: ...\n");
//
//     TotalMaxPacketSize = UhciEndpoint->EndpointProperties.TotalMaxPacketSize;
//     DeviceSpeed = UhciEndpoint->EndpointProperties.DeviceSpeed;
//     EndpointAddress = UhciEndpoint->EndpointProperties.EndpointAddress;
//     DeviceAddress = UhciEndpoint->EndpointProperties.DeviceAddress;
//     TransferType = UhciEndpoint->EndpointProperties.TransferType;
//
//     if (SgList->SgElementCount || TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//         ZeroLengthTransfer = FALSE;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         if (UhciTransfer->TransferParameters->TransferFlags &
//             USBD_TRANSFER_DIRECTION_IN)
//         {
//             PIDCode = UHCI_TD_PID_IN;
//         }
//         else
//         {
//             PIDCode = UHCI_TD_PID_OUT;
//         }
//
//         DataToggle = UHCI_TD_PID_DATA1;
//     }
//     else
//     {
//         if (USB_ENDPOINT_DIRECTION_OUT(EndpointAddress))
//             PIDCode = UHCI_TD_PID_OUT;
//         else
//             PIDCode = UHCI_TD_PID_IN;
//
//         DataToggle = UhciEndpoint->DataToggle;
//     }
//
//     for (ix = 0; ix < SgList->SgElementCount || ZeroLengthTransfer; ix++)
//     {
//         BytesRemaining = SgList->SgElement[ix].SgTransferLength;
//         PhysicalAddress = SgList->SgElement[ix].SgPhysicalAddress.LowPart;
//
//         if (!IsLastTd)
//         {
//             PhysicalAddress += TransferLength;
//             BytesRemaining -= TransferLength;
//         }
//
//         IsLastTd = TRUE;
//         TransferLength = 0;
//
//         while (BytesRemaining || ZeroLengthTransfer)
//         {
//             ZeroLengthTransfer = FALSE;
//
//             if (BytesRemaining >= TotalMaxPacketSize)
//             {
//                 LengthThisTD = TotalMaxPacketSize;
//                 BytesRemaining -= TotalMaxPacketSize;
//             }
//             else
//             {
//                 if (ix >= SgList->SgElementCount - 1)
//                 {
//                     LengthThisTD = BytesRemaining;
//                 }
//                 else
//                 {
//                     IsLastTd = FALSE;
//
//                     DPRINT1("UhciMapAsyncTransferToTds: IsLastTd = FALSE. FIXME\n");
//                     ASSERT(FALSE);
//                 }
//
//                 BytesRemaining = 0;
//             }
//
//             UhciTransfer->PendingTds++;
//             TD = UhciAllocateTD(UhciExtension, UhciEndpoint);
//             TD->Flags |= UHCI_HCD_TD_FLAG_PROCESSED;
//
//             TD->HwTD.NextElement = 0;
//             TD->HwTD.Buffer = PhysicalAddress;
//
//             TD->HwTD.ControlStatus.AsULONG = 0;
//             TD->HwTD.ControlStatus.LowSpeedDevice = (DeviceSpeed == UsbLowSpeed);
//             TD->HwTD.ControlStatus.Status = UHCI_TD_STS_ACTIVE;
//             TD->HwTD.ControlStatus.ErrorCounter = 3;
//             TD->HwTD.ControlStatus.ActualLength = UHCI_TD_LENGTH_NULL;
//             TD->HwTD.ControlStatus.ShortPacketDetect = 1;
//
//             TD->HwTD.Token.AsULONG = 0;
//             TD->HwTD.Token.Endpoint = EndpointAddress;
//             TD->HwTD.Token.DeviceAddress = DeviceAddress;
//             TD->HwTD.Token.PIDCode = PIDCode;
//
//             if (LengthThisTD == 0)
//                 TD->HwTD.Token.MaximumLength = UHCI_TD_LENGTH_NULL;
//             else
//                 TD->HwTD.Token.MaximumLength = LengthThisTD - 1;
//
//             TD->HwTD.Token.DataToggle = (DataToggle == UHCI_TD_PID_DATA1);
//
//             TD->NextHcdTD = 0;
//             TD->UhciTransfer = UhciTransfer;
//
//             if (!IsLastTd)
//                 ASSERT(FALSE);
//
//             PhysicalAddress += LengthThisTD;
//             LengthMapped += LengthThisTD;
//
//             if (LastTD)
//             {
//                 LastTD->HwTD.NextElement = TD->PhysicalAddress &
//                                            UHCI_TD_LINK_POINTER_MASK;
//                 LastTD->NextHcdTD = TD;
//             }
//             else
//             {
//                 *OutFirstTD = TD;
//             }
//
//             LastTD = TD;
//             DataToggle = DataToggle == UHCI_TD_PID_DATA0;
//         }
//     }
//
//     UhciEndpoint->DataToggle = DataToggle;
//
//     *OutLastTD = LastTD;
// }
//
// MPSTATUS
// NTAPI
// UhciControlTransfer(IN PUHCI_EXTENSION UhciExtension,
//                     IN PUHCI_ENDPOINT UhciEndpoint,
//                     IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                     IN PUHCI_TRANSFER UhciTransfer,
//                     IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PUHCI_HCD_TD FirstTD;
//     PUHCI_HCD_TD LastTD;
//     PUHCI_HCD_TD DataFirstTD;
//     PUHCI_HCD_TD DataLastTD;
//     UHCI_CONTROL_STATUS ControlStatus;
//     USB_DEVICE_SPEED DeviceSpeed;
//     USHORT EndpointAddress;
//     USHORT DeviceAddress;
//     ULONG PhysicalAddress;
//
//     DPRINT_UHCI!("UhciControlTransfer: UhciTransfer - %p\n", UhciTransfer);
//
//     if (UhciEndpoint->EndpointLock > 1)
//     {
//         InterlockedDecrement(&UhciEndpoint->EndpointLock);
//
//         if (UhciEndpoint->EndpointProperties.TransferType ==
//             USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             InterlockedDecrement(&UhciExtension->ExtensionLock);
//         }
//
//         DPRINT("UhciControlTransfer: end MP_STATUS_FAILURE\n");
//         return MP_STATUS_FAILURE;
//     }
//
//     DeviceSpeed = UhciEndpoint->EndpointProperties.DeviceSpeed;
//     EndpointAddress = UhciEndpoint->EndpointProperties.EndpointAddress;
//     DeviceAddress = UhciEndpoint->EndpointProperties.DeviceAddress;
//
//     /* Allocate and setup first TD */
//     UhciTransfer->PendingTds++;
//     FirstTD = UhciAllocateTD(UhciExtension, UhciEndpoint);
//     FirstTD->Flags |= UHCI_HCD_TD_FLAG_PROCESSED;
//     DPRINT_UHCI!("UhciControlTransfer: FirstTD - %p\n", FirstTD);
//
//     FirstTD->HwTD.NextElement = 0;
//
//     ControlStatus.AsULONG = 0;
//     ControlStatus.LowSpeedDevice = (DeviceSpeed == UsbLowSpeed);
//     ControlStatus.Status |= UHCI_TD_STS_ACTIVE;
//     ControlStatus.ErrorCounter = 3;
//     FirstTD->HwTD.ControlStatus = ControlStatus;
//
//     FirstTD->HwTD.Token.AsULONG = 0;
//     FirstTD->HwTD.Token.Endpoint = EndpointAddress;
//     FirstTD->HwTD.Token.DeviceAddress = DeviceAddress;
//
//     FirstTD->HwTD.Token.MaximumLength = sizeof(USB_DEFAULT_PIPE_SETUP_PACKET);
//     FirstTD->HwTD.Token.MaximumLength--;
//     FirstTD->HwTD.Token.PIDCode = UHCI_TD_PID_SETUP;
//     FirstTD->HwTD.Token.DataToggle = UHCI_TD_PID_DATA0;
//
//     RtlCopyMemory(&FirstTD->SetupPacket,
//                   &TransferParameters->SetupPacket,
//                   sizeof(USB_DEFAULT_PIPE_SETUP_PACKET));
//
//     FirstTD->HwTD.Buffer = FirstTD->PhysicalAddress + FIELD_OFFSET(UHCI_HCD_TD, SetupPacket);
//
//     FirstTD->NextHcdTD = NULL;
//     FirstTD->UhciTransfer = UhciTransfer;
//
//     /* Allocate and setup last TD */
//     UhciTransfer->PendingTds++;
//     LastTD = UhciAllocateTD(UhciExtension, UhciEndpoint);
//     LastTD->Flags |= UHCI_HCD_TD_FLAG_PROCESSED;
//     DPRINT_UHCI!("UhciControlTransfer: LastTD - %p\n", LastTD);
//
//     LastTD->HwTD.NextElement = 0;
//
//     LastTD->HwTD.ControlStatus.AsULONG = 0;
//     LastTD->HwTD.ControlStatus.LowSpeedDevice = (DeviceSpeed == UsbLowSpeed);
//     LastTD->HwTD.ControlStatus.Status |= UHCI_TD_STS_ACTIVE;
//     LastTD->HwTD.ControlStatus.ErrorCounter = 3;
//
//     LastTD->HwTD.Token.AsULONG = 0;
//     LastTD->HwTD.Token.Endpoint = EndpointAddress;
//     LastTD->HwTD.Token.DeviceAddress = DeviceAddress;
//
//     LastTD->UhciTransfer = UhciTransfer;
//     LastTD->NextHcdTD = NULL;
//
//     /* Allocate and setup TDs for data */
//     DataFirstTD = NULL;
//     DataLastTD = NULL;
//
//     UhciMapAsyncTransferToTDs(UhciExtension,
//                               UhciEndpoint,
//                               UhciTransfer,
//                               &DataFirstTD,
//                               &DataLastTD,
//                               SgList);
//
//     if (DataFirstTD)
//     {
//         PhysicalAddress = DataFirstTD->PhysicalAddress;
//         PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
//         FirstTD->HwTD.NextElement = PhysicalAddress;
//         FirstTD->NextHcdTD = DataFirstTD;
//
//         PhysicalAddress = LastTD->PhysicalAddress;
//         PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
//         DataLastTD->HwTD.NextElement = PhysicalAddress;
//         DataLastTD->NextHcdTD = LastTD;
//     }
//     else
//     {
//         PhysicalAddress = LastTD->PhysicalAddress;
//         PhysicalAddress &= UHCI_TD_LINK_POINTER_MASK;
//         FirstTD->HwTD.NextElement = PhysicalAddress;
//         FirstTD->NextHcdTD = LastTD;
//     }
//
//     LastTD->HwTD.Buffer = 0;
//     LastTD->HwTD.ControlStatus.InterruptOnComplete = 1;
//
//     LastTD->HwTD.Token.DataToggle = UHCI_TD_PID_DATA1;
//     LastTD->HwTD.Token.MaximumLength = UHCI_TD_LENGTH_NULL;
//
//     if (UhciTransfer->TransferParameters->TransferFlags &
//         USBD_TRANSFER_DIRECTION_IN)
//     {
//         LastTD->HwTD.Token.PIDCode = UHCI_TD_PID_OUT;
//     }
//     else
//     {
//         LastTD->HwTD.Token.PIDCode = UHCI_TD_PID_IN;
//     }
//
//     LastTD->HwTD.NextElement = UHCI_TD_LINK_PTR_TERMINATE;
//
//     LastTD->Flags |= UHCI_HCD_TD_FLAG_CONTROLL;
//     LastTD->NextHcdTD = NULL;
//
//     /* Link this transfer to queue */
//     UhciQueueTransfer(UhciExtension, UhciEndpoint, FirstTD, LastTD);
//
//     DPRINT_UHCI!("UhciControlTransfer: end MP_STATUS_SUCCESS\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciBulkOrInterruptTransfer(IN PUHCI_EXTENSION UhciExtension,
//                             IN PUHCI_ENDPOINT UhciEndpoint,
//                             IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                             IN PUHCI_TRANSFER UhciTransfer,
//                             IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PUHCI_HCD_TD DataFirstTD;
//     PUHCI_HCD_TD DataLastTD;
//     ULONG TotalMaxPacketSize;
//     ULONG SgCount;
//     ULONG TransferLength;
//     ULONG TDs;
//     ULONG ix;
//
//     DPRINT_UHCI!("UhciBulkOrInterruptTransfer: ...\n");
//
//     TotalMaxPacketSize = UhciEndpoint->EndpointProperties.TotalMaxPacketSize;
//
//     SgCount = SgList->SgElementCount;
//
//     if (SgCount == 0)
//     {
//         DPRINT("UhciBulkOrInterruptTransfer: SgCount == 0 \n");
//         TDs = 1;
//     }
//     else
//     {
//         TransferLength = 0;
//
//         for (ix = 0; ix < SgCount; ++ix)
//         {
//             TransferLength += SgList->SgElement[ix].SgTransferLength;
//         }
//
//         DPRINT("UhciBulkOrInterruptTransfer: SgCount - %X, TransferLength - %X\n",
//                SgList->SgElementCount,
//                TransferLength);
//
//         if (TransferLength)
//         {
//             TDs = TransferLength + (TotalMaxPacketSize - 1);
//             TDs /= TotalMaxPacketSize;
//         }
//         else
//         {
//             TDs = 1;
//         }
//     }
//
//     if ((UhciEndpoint->MaxTDs - UhciEndpoint->AllocatedTDs) < TDs)
//     {
//         DPRINT1("UhciBulkOrInterruptTransfer: Not enough TDs \n");
//         return MP_STATUS_FAILURE;
//     }
//
//     DataFirstTD = NULL;
//     DataLastTD = NULL;
//
//     UhciMapAsyncTransferToTDs(UhciExtension,
//                               UhciEndpoint,
//                               UhciTransfer,
//                               &DataFirstTD,
//                               &DataLastTD,
//                               SgList);
//
//     if (DataLastTD == NULL || DataFirstTD == NULL)
//     {
//         DPRINT1("UhciBulkOrInterruptTransfer: !DataLastTD || !DataFirstTD\n");
//         return MP_STATUS_FAILURE;
//     }
//
//     DataLastTD->HwTD.NextElement = UHCI_TD_LINK_PTR_TERMINATE;
//     DataLastTD->HwTD.ControlStatus.InterruptOnComplete = 1;
//     DataLastTD->NextHcdTD = NULL;
//
//     UhciQueueTransfer(UhciExtension,
//                       UhciEndpoint,
//                       DataFirstTD,
//                       DataLastTD);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciSubmitTransfer(IN PVOID uhciExtension,
//                    IN PVOID uhciEndpoint,
//                    IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                    IN PVOID uhciTransfer,
//                    IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     PUHCI_TRANSFER UhciTransfer = uhciTransfer;
//     ULONG TransferType;
//
//     DPRINT_UHCI!("UhciSubmitTransfer: ...\n");
//
//     InterlockedIncrement(&UhciEndpoint->EndpointLock);
//
//     TransferType = UhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS &&
//         InterlockedIncrement(&UhciExtension->ExtensionLock) == 1)
//     {
//         UhciExtension->FrameNumber = UhciGet32BitFrameNumber(UhciExtension);
//     }
//
//     RtlZeroMemory(UhciTransfer, sizeof(UHCI_TRANSFER));
//
//     UhciTransfer->TransferParameters = TransferParameters;
//     UhciTransfer->UhciEndpoint = UhciEndpoint;
//     UhciTransfer->USBDStatus = USBD_STATUS_SUCCESS;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//     {
//         return UhciControlTransfer(UhciExtension,
//                                    UhciEndpoint,
//                                    TransferParameters,
//                                    UhciTransfer,
//                                    SgList);
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         return UhciBulkOrInterruptTransfer(UhciExtension,
//                                            UhciEndpoint,
//                                            TransferParameters,
//                                            UhciTransfer,
//                                            SgList);
//     }
//
//     DPRINT1("UhciSubmitTransfer: Error TransferType - %x\n", TransferType);
//
//     return MP_STATUS_SUCCESS;
// }
//
// USBD_STATUS
// NTAPI
// UhciGetErrorFromTD(IN PUHCI_EXTENSION UhciExtension,
//                    IN PUHCI_HCD_TD TD)
// {
//     USBD_STATUS USBDStatus;
//     UCHAR TdStatus;
//
//     //DPRINT("UhciGetErrorFromTD: ...\n");
//
//     TdStatus = TD->HwTD.ControlStatus.Status;
//
//     if (TdStatus == UHCI_TD_STS_ACTIVE)
//     {
//         if (TD->HwTD.Token.MaximumLength == UHCI_TD_LENGTH_NULL)
//             return USBD_STATUS_SUCCESS;
//
//         if (TD->HwTD.ControlStatus.ActualLength + 1 >=
//             TD->HwTD.Token.MaximumLength + 1)
//         {
//             return USBD_STATUS_SUCCESS;
//         }
//
//         if (TD->HwTD.ControlStatus.InterruptOnComplete == 1)
//             return USBD_STATUS_SUCCESS;
//
//         return USBD_STATUS_ERROR_SHORT_TRANSFER;
//     }
//
//     if (TdStatus & UHCI_TD_STS_BABBLE_DETECTED &&
//         TdStatus & UHCI_TD_STS_STALLED)
//     {
//         DPRINT1("UhciGetErrorFromTD: USBD_STATUS_BUFFER_OVERRUN, TD - %p\n", TD);
//         return USBD_STATUS_BUFFER_OVERRUN;
//     }
//
//     if (TdStatus & UHCI_TD_STS_TIMEOUT_CRC_ERROR &&
//         TdStatus & UHCI_TD_STS_STALLED)
//     {
//         DPRINT1("UhciGetErrorFromTD: USBD_STATUS_DEV_NOT_RESPONDING, TD - %p\n", TD);
//         return USBD_STATUS_DEV_NOT_RESPONDING;
//     }
//
//     if (TdStatus & UHCI_TD_STS_TIMEOUT_CRC_ERROR)
//     {
//         if (TD->HwTD.ControlStatus.ActualLength == UHCI_TD_LENGTH_NULL)
//         {
//             DPRINT1("UhciGetErrorFromTD: USBD_STATUS_DEV_NOT_RESPONDING, TD - %p\n", TD);
//             return USBD_STATUS_DEV_NOT_RESPONDING;
//         }
//         else
//         {
//             DPRINT1("UhciGetErrorFromTD: USBD_STATUS_CRC, TD - %p\n", TD);
//             return USBD_STATUS_CRC;
//         }
//     }
//     else if (TdStatus & UHCI_TD_STS_DATA_BUFFER_ERROR)
//     {
//         DPRINT1("UhciGetErrorFromTD: USBD_STATUS_DATA_OVERRUN, TD - %p\n", TD);
//         USBDStatus = USBD_STATUS_DATA_OVERRUN;
//     }
//     else if (TdStatus & UHCI_TD_STS_STALLED)
//     {
//         DPRINT1("UhciGetErrorFromTD: USBD_STATUS_STALL_PID, TD - %p\n", TD);
//         USBDStatus = USBD_STATUS_STALL_PID;
//     }
//     else
//     {
//         DPRINT1("UhciGetErrorFromTD: USBD_STATUS_INTERNAL_HC_ERROR, TD - %p\n", TD);
//         USBDStatus = USBD_STATUS_INTERNAL_HC_ERROR;
//     }
//
//     return USBDStatus;
// }
//
// VOID
// NTAPI
// UhciProcessDoneNonIsoTD(IN PUHCI_EXTENSION UhciExtension,
//                         IN PUHCI_HCD_TD TD)
// {
//     PUSBPORT_TRANSFER_PARAMETERS TransferParameters;
//     PUHCI_ENDPOINT UhciEndpoint;
//     PUHCI_TRANSFER UhciTransfer;
//     USBD_STATUS USBDStatus = USBD_STATUS_SUCCESS;
//     SIZE_T TransferedLen;
//
//     DPRINT_UHCI!("UhciProcessDoneNonIsoTD: TD - %p\n", TD);
//
//     UhciTransfer = TD->UhciTransfer;
//     UhciTransfer->PendingTds--;
//
//     TransferParameters = UhciTransfer->TransferParameters;
//     UhciEndpoint = UhciTransfer->UhciEndpoint;
//
//     if (!(TD->Flags & UHCI_HCD_TD_FLAG_NOT_ACCESSED))
//     {
//         if (UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//             USBDStatus = UhciGetErrorFromTD(UhciExtension, TD);
//
//         if (USBDStatus != USBD_STATUS_SUCCESS ||
//             (TD->HwTD.ControlStatus.ActualLength == UHCI_TD_LENGTH_NULL))
//         {
//             TransferedLen = 0;
//         }
//         else
//         {
//             TransferedLen = TD->HwTD.ControlStatus.ActualLength + 1;
//         }
//
//         if (TD->HwTD.Token.PIDCode != UHCI_TD_PID_SETUP)
//             UhciTransfer->TransferLen += TransferedLen;
//
//         if (TD->HwTD.Token.PIDCode == UHCI_TD_PID_IN &&
//             TD->Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER)
//         {
//             DPRINT_IMPL!("UhciProcessDoneNonIsoTD: UNIMPLEMENTED. FIXME\n");
//         }
//
//         if (USBDStatus != USBD_STATUS_SUCCESS)
//             UhciTransfer->USBDStatus = USBDStatus;
//     }
//
//     if (TD->Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER)
//         DPRINT_IMPL!("UhciProcessDoneNonIsoTD: UNIMPLEMENTED. FIXME\n");
//
//     UhciEndpoint->AllocatedTDs--;
//
//     TD->HwTD.NextElement = 0;
//     TD->UhciTransfer = NULL;
//     TD->Flags = 0;
//
//     if (UhciTransfer->PendingTds == 0)
//     {
//         InterlockedDecrement(&UhciEndpoint->EndpointLock);
//
//         if (UhciEndpoint->EndpointProperties.TransferType ==
//             USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         {
//             InterlockedDecrement(&UhciExtension->ExtensionLock);
//         }
//
//         RegPacket.UsbPortCompleteTransfer(UhciExtension,
//                                           UhciEndpoint,
//                                           TransferParameters,
//                                           UhciTransfer->USBDStatus,
//                                           UhciTransfer->TransferLen);
//     }
// }
//
// MPSTATUS
// NTAPI
// UhciIsochTransfer(IN PVOID ehciExtension,
//                   IN PVOID ehciEndpoint,
//                   IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                   IN PVOID ehciTransfer,
//                   IN PVOID isoParameters)
// {
//     DPRINT_IMPL!("UhciIsochTransfer: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciAbortIsoTransfer(IN PUHCI_EXTENSION UhciExtension,
//                      IN PUHCI_ENDPOINT UhciEndpoint,
//                      IN PUHCI_TRANSFER UhciTransfer)
// {
//     DPRINT_IMPL!("UhciAbortIsoTransfer: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// UhciAbortNonIsoTransfer(IN PUHCI_EXTENSION UhciExtension,
//                         IN PUHCI_ENDPOINT UhciEndpoint,
//                         IN PUHCI_TRANSFER UhciTransfer,
//                         IN PULONG CompletedLength)
// {
//     PUHCI_HCD_TD TD;
//     PUHCI_HCD_TD PrevTD = NULL;
//     ULONG PhysicalAddress;
//     BOOL DataToggle;
//     BOOLEAN IsHeadTD = FALSE;
//
//     DPRINT("UhciAbortNonIsoTransfer: UhciExtension - %p, QH - %p, UhciTransfer - %p\n",
//            UhciExtension,
//            UhciEndpoint->QH,
//            UhciTransfer);
//
//     for (TD = UhciEndpoint->HeadTD;
//          TD && TD->UhciTransfer != UhciTransfer;
//          TD = TD->NextHcdTD)
//     {
//         PrevTD = TD;
//     }
//
//     DataToggle = TD->HwTD.Token.DataToggle;
//
//     if (TD == UhciEndpoint->HeadTD)
//         IsHeadTD = TRUE;
//
//     while (TD && TD->UhciTransfer == UhciTransfer)
//     {
//         DPRINT_UHCI!("UhciAbortNonIsoTransfer: TD - %p\n", TD);
//
//         if (TD->HwTD.ControlStatus.Status & UHCI_TD_STS_ACTIVE)
//         {
//             if (TD->Flags & UHCI_HCD_TD_FLAG_DATA_BUFFER)
//                 DPRINT_IMPL!("UhciAbortNonIsoTransfer: UNIMPLEMENTED. FIXME\n");
//
//             UhciEndpoint->AllocatedTDs--;
//
//             DPRINT_UHCI!("UhciAbortNonIsoTransfer: Active TD - %p\n", TD);
//
//             TD->HwTD.NextElement = 0;
//             TD->Flags = 0;
//             TD->UhciTransfer = NULL;
//         }
//         else
//         {
//             UhciProcessDoneNonIsoTD(UhciExtension, TD);
//         }
//
//         TD = TD->NextHcdTD;
//     }
//
//     UhciFixDataToggle(UhciExtension,
//                       UhciEndpoint,
//                       TD,
//                       DataToggle);
//
//     if (IsHeadTD)
//     {
//         if (TD)
//         {
//             UhciEndpoint->HeadTD = TD;
//         }
//         else
//         {
//             UhciEndpoint->HeadTD = NULL;
//             UhciEndpoint->TailTD = NULL;
//         }
//
//         if (TD == NULL || UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//         {
//             PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         }
//         else
//         {
//             PhysicalAddress = TD->PhysicalAddress;
//             PhysicalAddress &= ~UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//         }
//
//         DPRINT_UHCI!("UhciAbortNonIsoTransfer: TD - %p\n", TD);
//
//         UhciEndpoint->QH->HwQH.NextElement = PhysicalAddress;
//         UhciEndpoint->QH->HwQH.NextElement &= ~UHCI_QH_ELEMENT_LINK_PTR_QH;
//     }
//     else if (TD)
//     {
//         PrevTD->HwTD.NextElement = TD->PhysicalAddress & UHCI_TD_LINK_POINTER_MASK;
//         PrevTD->NextHcdTD = TD;
//     }
//     else
//     {
//         PrevTD->NextHcdTD = NULL;
//         PrevTD->HwTD.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//
//         UhciEndpoint->TailTD = PrevTD;
//     }
//
//     *CompletedLength = UhciTransfer->TransferLen;
// }
//
// VOID
// NTAPI
// UhciAbortTransfer(IN PVOID uhciExtension,
//                   IN PVOID uhciEndpoint,
//                   IN PVOID uhciTransfer,
//                   IN PULONG CompletedLength)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     PUHCI_TRANSFER UhciTransfer = uhciTransfer;
//     ULONG TransferType;
//
//     DPRINT("UhciAbortTransfer: ...\n");
//
//     InterlockedDecrement(&UhciEndpoint->EndpointLock);
//
//     TransferType = UhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         InterlockedDecrement(&UhciExtension->ExtensionLock);
//
//         UhciAbortIsoTransfer(UhciExtension,
//                              UhciEndpoint,
//                              UhciTransfer);
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         UhciAbortNonIsoTransfer(UhciExtension,
//                                 UhciEndpoint,
//                                 UhciTransfer,
//                                 CompletedLength);
//     }
// }
//
// ULONG
// NTAPI
// UhciGetEndpointState(IN PVOID uhciExtension,
//                      IN PVOID uhciEndpoint)
// {
//     DPRINT_IMPL!("UhciGetEndpointState: UNIMPLEMENTED. FIXME\n");
//     return 0;
// }
//
// VOID
// NTAPI
// UhciInsertQH(IN PUHCI_EXTENSION UhciExtension,
//              IN PUHCI_HCD_QH StaticQH,
//              IN PUHCI_HCD_QH QH)
// {
//     PUHCI_HCD_QH NextHcdQH;
//
//     DPRINT("UhciInsertQH: UhciExtension - %p, StaticQH - %p, QH - %p\n", UhciExtension, StaticQH, QH);
//
//     QH->HwQH.NextQH = StaticQH->HwQH.NextQH;
//     NextHcdQH = StaticQH->NextHcdQH;
//
//     QH->PrevHcdQH = StaticQH;
//     QH->NextHcdQH = NextHcdQH;
//
//     if (NextHcdQH)
//         NextHcdQH->PrevHcdQH = QH;
//     else
//         UhciExtension->BulkTailQH = QH;
//
//     StaticQH->HwQH.NextQH = QH->PhysicalAddress | UHCI_QH_HEAD_LINK_PTR_QH;
//     StaticQH->NextHcdQH = QH;
//
//     QH->QhFlags |= UHCI_HCD_QH_FLAG_ACTIVE;
// }
//
// VOID
// NTAPI
// UhciUnlinkQH(IN PUHCI_EXTENSION UhciExtension,
//              IN PUHCI_HCD_QH QH)
// {
//     PUHCI_HCD_QH NextHcdQH;
//     PUHCI_HCD_QH PrevHcdQH;
//     PUHCI_HCD_QH BulkQH;
//
//     DPRINT("UhciUnlinkQH: ... \n");
//
//     NextHcdQH = QH->NextHcdQH;
//     PrevHcdQH = QH->PrevHcdQH;
//
//     if (UhciExtension->BulkTailQH == QH)
//         UhciExtension->BulkTailQH = PrevHcdQH;
//
//     PrevHcdQH->HwQH.NextQH = QH->HwQH.NextQH;
//     PrevHcdQH->NextHcdQH = NextHcdQH;
//
//     if (NextHcdQH)
//         NextHcdQH->PrevHcdQH = PrevHcdQH;
//
//     QH->PrevHcdQH = QH;
//     QH->NextHcdQH = QH;
//
//     if (!(QH->UhciEndpoint->EndpointProperties.TransferType ==
//           USBPORT_TRANSFER_TYPE_BULK))
//     {
//         QH->QhFlags &= ~UHCI_HCD_QH_FLAG_ACTIVE;
//         return;
//     }
//
//     if ((UhciExtension->BulkTailQH->HwQH.NextQH & UHCI_QH_HEAD_LINK_PTR_TERMINATE)
//                                                == UHCI_QH_HEAD_LINK_PTR_TERMINATE)
//     {
//         QH->QhFlags &= ~UHCI_HCD_QH_FLAG_ACTIVE;
//         return;
//     }
//
//     BulkQH = UhciExtension->BulkQH;
//
//     while (TRUE)
//     {
//         BulkQH = BulkQH->NextHcdQH;
//
//         if (!BulkQH)
//             break;
//
//         if (!(BulkQH->HwQH.NextElement & UHCI_QH_ELEMENT_LINK_PTR_TERMINATE))
//         {
//             QH->QhFlags &= ~UHCI_HCD_QH_FLAG_ACTIVE;
//             return;
//         }
//     }
//
//     UhciExtension->BulkTailQH->HwQH.NextQH |= UHCI_QH_HEAD_LINK_PTR_TERMINATE;
//
//     QH->QhFlags &= ~UHCI_HCD_QH_FLAG_ACTIVE;
// }
//
// VOID
// NTAPI
// UhciSetEndpointState(IN PVOID uhciExtension,
//                      IN PVOID uhciEndpoint,
//                      IN ULONG EndpointState)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     ULONG TransferType;
//     PUHCI_HCD_QH QH;
//     ULONG Idx;
//
//     TransferType = UhciEndpoint->EndpointProperties.TransferType;
//     QH = UhciEndpoint->QH;
//
//     DPRINT("UhciSetEndpointState: EndpointState - %x, TransferType - %x\n",
//            EndpointState,
//            TransferType);
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         return;
//
//     if (TransferType != USBPORT_TRANSFER_TYPE_CONTROL &&
//         TransferType != USBPORT_TRANSFER_TYPE_BULK &&
//         TransferType != USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         DPRINT("UhciSetEndpointState: Unknown TransferType - %x\n",
//                TransferType);
//     }
//
//     switch (EndpointState)
//     {
//         case USBPORT_ENDPOINT_PAUSED:
//             UhciUnlinkQH(UhciExtension, QH);
//             return;
//
//         case USBPORT_ENDPOINT_ACTIVE:
//             switch (TransferType)
//             {
//                 case USBPORT_TRANSFER_TYPE_CONTROL:
//                     UhciInsertQH(UhciExtension,
//                                  UhciExtension->ControlQH,
//                                  UhciEndpoint->QH);
//                     break;
//
//                 case USBPORT_TRANSFER_TYPE_BULK:
//                     UhciInsertQH(UhciExtension,
//                                  UhciExtension->BulkQH,
//                                  UhciEndpoint->QH);
//                     break;
//
//                 case USBPORT_TRANSFER_TYPE_INTERRUPT:
//                     Idx = UhciEndpoint->EndpointProperties.Period +
//                           UhciEndpoint->EndpointProperties.ScheduleOffset;
//
//                     UhciInsertQH(UhciExtension,
//                                  UhciExtension->IntQH[Idx - 1],
//                                  UhciEndpoint->QH);
//                     break;
//                 default:
//                     ASSERT(FALSE);
//                     break;
//             }
//
//             break;
//
//         case USBPORT_ENDPOINT_REMOVE:
//             QH->QhFlags |= UHCI_HCD_QH_FLAG_REMOVE;
//             UhciUnlinkQH(UhciExtension, QH);
//             break;
//
//         default:
//             ASSERT(FALSE);
//             break;
//     }
// }
//
// ULONG
// NTAPI
// UhciGetEndpointStatus(IN PVOID uhciExtension,
//                       IN PVOID uhciEndpoint)
// {
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     ULONG EndpointStatus;
//
//     DPRINT_UHCI!("UhciGetEndpointStatus: ...\n");
//
//     if (UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//         EndpointStatus = USBPORT_ENDPOINT_HALT;
//     else
//         EndpointStatus = USBPORT_ENDPOINT_RUN;
//
//     return EndpointStatus;
// }
//
// VOID
// NTAPI
// UhciSetEndpointStatus(IN PVOID uhciExtension,
//                       IN PVOID uhciEndpoint,
//                       IN ULONG EndpointStatus)
// {
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     ULONG PhysicalAddress;
//
//     DPRINT("UhciSetEndpointStatus: uhciEndpoint - %p, EndpointStatus - %X\n",
//            uhciEndpoint,
//            EndpointStatus);
//
//     if (EndpointStatus != USBPORT_ENDPOINT_RUN)
//         return;
//
//     if (!(UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED))
//         return;
//
//     UhciEndpoint->Flags &= ~UHCI_ENDPOINT_FLAG_HALTED;
//
//     if (UhciEndpoint->HeadTD == NULL)
//         UhciEndpoint->TailTD = NULL;
//
//     if (UhciEndpoint->HeadTD)
//     {
//         PhysicalAddress = UhciEndpoint->HeadTD->PhysicalAddress;
//         PhysicalAddress &= ~UHCI_TD_LINK_PTR_TERMINATE;
//         UhciEndpoint->QH->HwQH.NextElement = PhysicalAddress;
//         UhciEndpoint->QH->HwQH.NextElement &= ~UHCI_QH_ELEMENT_LINK_PTR_QH;
//     }
//     else
//     {
//         UhciEndpoint->QH->HwQH.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//     }
// }
//
// VOID
// NTAPI
// UhciPollIsoEndpoint(IN PUHCI_EXTENSION UhciExtension,
//                     IN PUHCI_ENDPOINT UhciEndpoint)
// {
//     DPRINT_IMPL!("UhciPollIsoEndpoint: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// UhciPollNonIsoEndpoint(IN PUHCI_EXTENSION UhciExtension,
//                        IN PUHCI_ENDPOINT UhciEndpoint)
// {
//     PUHCI_HCD_QH QH;
//     PUHCI_HCD_TD NextTD;
//     PUHCI_HCD_TD TD;
//     ULONG NextTdPA;
//     ULONG PhysicalAddress;
//     SIZE_T TransferedLen;
//     PLIST_ENTRY ListTDs;
//     UCHAR TdStatus;
//
//     DPRINT_UHCI!("UhciPollNonIsoEndpoint: UhciExtension - %p, UhciEndpoint - %p\n",
//                 UhciExtension,
//                 UhciEndpoint);
//
//     if (UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//     {
//         DPRINT("UhciPollNonIsoEndpoint: Ep->Flags & UHCI_ENDPOINT_FLAG_HALTED \n");
//         return;
//     }
//
//     QH = UhciEndpoint->QH;
//
//     NextTdPA = QH->HwQH.NextElement & UHCI_QH_ELEMENT_LINK_POINTER_MASK;
//
//     if (NextTdPA)
//     {
//         NextTD = RegPacket.UsbPortGetMappedVirtualAddress(NextTdPA,
//                                                           UhciExtension,
//                                                           UhciEndpoint);
//     }
//     else
//     {
//         NextTD = NULL;
//     }
//
//     DPRINT_UHCI!("UhciPollNonIsoEndpoint: NextTD - %p, NextTdPA - %p\n",
//                 NextTD,
//                 NextTdPA);
//
//     for (TD = UhciEndpoint->HeadTD; TD != NextTD && TD != NULL; TD = TD->NextHcdTD)
//     {
//         DPRINT_UHCI!("UhciPollNonIsoEndpoint: TD - %p, TD->NextHcdTD - %p\n",
//                     TD,
//                     TD->NextHcdTD);
//
//         TD->Flags |= UHCI_HCD_TD_FLAG_DONE;
//         InsertTailList(&UhciEndpoint->ListTDs, &TD->TdLink);
//
//         if (TD->NextHcdTD &&
//             TD->NextHcdTD->HwTD.ControlStatus.Status & UHCI_TD_STS_ACTIVE)
//         {
//             if (NextTdPA == 0)
//             {
//                 TD = TD->NextHcdTD;
//                 goto EnqueueTD;
//             }
//
//             if (NextTdPA != TD->NextHcdTD->PhysicalAddress)
//             {
//                 DPRINT("UhciPollNonIsoEndpoint: TD->NextHcdTD->PhysicalAddress - %p\n",
//                        TD->NextHcdTD->PhysicalAddress);
//                 ASSERT(FALSE);
//             }
//         }
//         else
//         {
//             if (TD->NextHcdTD == NULL)
//             {
//                 DPRINT_UHCI!("UhciPollNonIsoEndpoint: TD->NextHcdTD == NULL\n");
//             }
//             else
//             {
//                 DPRINT_UHCI!("UhciPollNonIsoEndpoint: ControlStatus - %X\n",
//                             TD->NextHcdTD->HwTD.ControlStatus.AsULONG);
//             }
//         }
//     }
//
//     UhciEndpoint->HeadTD = NextTD;
//
//     if (NextTD == NULL)
//     {
//         DPRINT_UHCI!("UhciPollNonIsoEndpoint: NextTD == NULL\n");
//
//         UhciEndpoint->HeadTD = NULL;
//         UhciEndpoint->TailTD = NULL;
//
//         QH->HwQH.NextElement = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//
//         goto ProcessListTDs;
//     }
//
//     DPRINT_UHCI!("UhciPollNonIsoEndpoint: NextTD - %p, NextTdPA - %p\n",
//                 NextTD,
//                 NextTdPA);
//
//     TdStatus = NextTD->HwTD.ControlStatus.Status;
//
//     if (TdStatus & UHCI_TD_STS_ACTIVE)
//     {
//         DPRINT_UHCI!("UhciPollNonIsoEndpoint: UHCI_TD_STS_ACTIVE \n");
//         goto ProcessListTDs;
//     }
//
//     if (NextTD->HwTD.Token.PIDCode == UHCI_TD_PID_IN &&
//         TdStatus & UHCI_TD_STS_STALLED &&
//         TdStatus & UHCI_TD_STS_TIMEOUT_CRC_ERROR &&
//         !(TdStatus & UHCI_TD_STS_NAK_RECEIVED) &&
//         !(TdStatus & UHCI_TD_STS_BABBLE_DETECTED) &&
//         !(TdStatus & UHCI_TD_STS_BITSTUFF_ERROR))
//     {
//         DPRINT("UhciPollNonIsoEndpoint: USBD_STATUS_DEV_NOT_RESPONDING\n");
//
//         UhciDumpHcdTD(NextTD);
//
//         if (!(NextTD->Flags & UHCI_HCD_TD_FLAG_STALLED_SETUP))
//         {
//             NextTD->HwTD.ControlStatus.ErrorCounter = 3;
//
//             NextTD->HwTD.ControlStatus.Status &= ~(UHCI_TD_STS_STALLED |
//                                                    UHCI_TD_STS_TIMEOUT_CRC_ERROR);
//
//             NextTD->HwTD.ControlStatus.Status |= UHCI_TD_STS_ACTIVE;
//
//             NextTD->Flags = NextTD->Flags | UHCI_HCD_TD_FLAG_STALLED_SETUP;
//
//             goto ProcessListTDs;
//         }
//     }
//
//     if (TdStatus & (UHCI_TD_STS_STALLED |
//                     UHCI_TD_STS_DATA_BUFFER_ERROR |
//                     UHCI_TD_STS_BABBLE_DETECTED |
//                     UHCI_TD_STS_TIMEOUT_CRC_ERROR |
//                     UHCI_TD_STS_BITSTUFF_ERROR))
//     {
//         DPRINT("UhciPollNonIsoEndpoint: NextTD UHCI_TD_STS_ - %02X, PIDCode - %02X\n",
//                NextTD->HwTD.ControlStatus.Status,
//                NextTD->HwTD.Token.PIDCode);
//
//         UhciDumpHcdTD(NextTD);
//
//         UhciEndpoint->Flags |= UHCI_ENDPOINT_FLAG_HALTED;
//         NextTD->Flags |= UHCI_HCD_TD_FLAG_DONE;
//
//         InsertTailList(&UhciEndpoint->ListTDs, &NextTD->TdLink);
//
//         if (TD->UhciTransfer != NextTD->UhciTransfer)
//             ASSERT(TD->UhciTransfer == NextTD->UhciTransfer);
//
//         while (TD &&
//                TD->UhciTransfer->TransferParameters->TransferCounter ==
//                NextTD->UhciTransfer->TransferParameters->TransferCounter)
//         {
//             DPRINT("UhciPollNonIsoEndpoint: Bad TD - %p\n", TD);
//
//             if (!(TD->Flags & UHCI_HCD_TD_FLAG_DONE))
//             {
//                 TD->Flags |= UHCI_HCD_TD_FLAG_DONE;
//                 TD->Flags |= UHCI_HCD_TD_FLAG_NOT_ACCESSED;
//
//                 InsertTailList(&UhciEndpoint->ListTDs, &TD->TdLink);
//             }
//
//             TD = TD->NextHcdTD;
//         }
//
//         if (UhciEndpoint->EndpointProperties.TransferType !=
//             USBPORT_TRANSFER_TYPE_CONTROL)
//         {
//             UhciFixDataToggle(UhciExtension,
//                               UhciEndpoint,
//                               TD,
//                               NextTD->HwTD.Token.DataToggle);
//         }
//     }
//     else
//     {
//         TransferedLen = NextTD->HwTD.ControlStatus.ActualLength;
//
//         if (TransferedLen == UHCI_TD_LENGTH_NULL)
//             TransferedLen = 0;
//         else
//             TransferedLen += 1;
//
//         if (NextTD->HwTD.Token.MaximumLength == UHCI_TD_LENGTH_NULL ||
//             TransferedLen >= (NextTD->HwTD.Token.MaximumLength + 1))
//         {
//             DPRINT_UHCI!("UhciPollNonIsoEndpoint: NextTD - %p, TransferedLen - %X\n",
//                         NextTD,
//                         TransferedLen);
//
//             if (NextTdPA ==
//                 (QH->HwQH.NextElement & UHCI_QH_ELEMENT_LINK_POINTER_MASK))
//             {
//                 NextTD->Flags |= UHCI_HCD_TD_FLAG_DONE;
//                 InsertTailList(&UhciEndpoint->ListTDs, &NextTD->TdLink);
//
//                 UhciEndpoint->HeadTD = NextTD->NextHcdTD;
//
//                 QH->HwQH.NextElement = NextTD->HwTD.NextElement;
//                 QH->HwQH.NextElement |= UHCI_QH_ELEMENT_LINK_PTR_TD;
//
//                 DPRINT_UHCI!("UhciPollNonIsoEndpoint: NextTD - %p, TD - %p\n",
//                             NextTD,
//                             TD);
//             }
//
//             goto ProcessListTDs;
//         }
//
//         DPRINT_UHCI!("UhciPollNonIsoEndpoint: ShortPacket. ControlStatus - %X\n",
//                     NextTD->HwTD.ControlStatus.AsULONG);
//
//         NextTD->Flags |= UHCI_HCD_TD_FLAG_DONE;
//         InsertTailList(&UhciEndpoint->ListTDs, &NextTD->TdLink);
//
//         while (TD &&
//                TD->UhciTransfer->TransferParameters->TransferCounter ==
//                NextTD->UhciTransfer->TransferParameters->TransferCounter)
//         {
//             if (TD->Flags & UHCI_HCD_TD_FLAG_CONTROLL &&
//                 NextTD->UhciTransfer->TransferParameters->TransferFlags &
//                 USBD_SHORT_TRANSFER_OK)
//             {
//                 break;
//             }
//
//             if (!(TD->Flags & UHCI_HCD_TD_FLAG_DONE))
//             {
//                 DPRINT_UHCI!("UhciPollNonIsoEndpoint: TD - %p\n", TD);
//
//                 TD->Flags |= (UHCI_HCD_TD_FLAG_DONE |
//                               UHCI_HCD_TD_FLAG_NOT_ACCESSED);
//
//                 InsertTailList(&UhciEndpoint->ListTDs, &TD->TdLink);
//             }
//
//             TD = TD->NextHcdTD;
//         }
//
//         if (NextTD->NextHcdTD &&
//             (UhciEndpoint->EndpointProperties.TransferType !=
//              USBPORT_TRANSFER_TYPE_CONTROL))
//         {
//             UhciFixDataToggle(UhciExtension,
//                               UhciEndpoint,
//                               TD,
//                               NextTD->NextHcdTD->HwTD.Token.DataToggle);
//         }
//
//         if (!(NextTD->UhciTransfer->TransferParameters->TransferFlags &
//               USBD_SHORT_TRANSFER_OK))
//         {
//             UhciEndpoint->Flags |= UHCI_ENDPOINT_FLAG_HALTED;
//         }
//     }
//
// EnqueueTD:
//
//     if (TD)
//     {
//         UhciEndpoint->HeadTD = TD;
//     }
//     else
//     {
//         UhciEndpoint->HeadTD = NULL;
//         UhciEndpoint->TailTD = NULL;
//     }
//
//     if (TD == NULL || UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//     {
//         PhysicalAddress = UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//     }
//     else
//     {
//         PhysicalAddress = TD->PhysicalAddress;
//         PhysicalAddress &= ~UHCI_QH_ELEMENT_LINK_PTR_TERMINATE;
//     }
//
//     DPRINT_UHCI!("UhciPollNonIsoEndpoint: TD - %p\n", TD);
//
//     QH->HwQH.NextElement = PhysicalAddress;
//     QH->HwQH.NextElement &= ~UHCI_QH_ELEMENT_LINK_PTR_QH;
//
// ProcessListTDs:
//
//     ListTDs = &UhciEndpoint->ListTDs;
//
//     while (!IsListEmpty(ListTDs))
//     {
//         TD = CONTAINING_RECORD(ListTDs->Flink,
//                                UHCI_HCD_TD,
//                                TdLink.Flink);
//
//         RemoveHeadList(ListTDs);
//
//         if ((TD->Flags & (UHCI_HCD_TD_FLAG_PROCESSED | UHCI_HCD_TD_FLAG_DONE)) ==
//                          (UHCI_HCD_TD_FLAG_PROCESSED | UHCI_HCD_TD_FLAG_DONE))
//         {
//             UhciProcessDoneNonIsoTD(UhciExtension, TD);
//         }
//     }
//
//     if (UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_CONTROL_OR_ISO &&
//         UhciEndpoint->Flags & UHCI_ENDPOINT_FLAG_HALTED)
//     {
//         DPRINT_UHCI!("UhciPollNonIsoEndpoint: Halted periodic EP - %p\n",
//                     UhciEndpoint);
//
//         UhciSetEndpointStatus(UhciExtension,
//                               UhciEndpoint,
//                               USBPORT_ENDPOINT_RUN);
//     }
// }
//
// VOID
// NTAPI
// UhciPollEndpoint(IN PVOID uhciExtension,
//                  IN PVOID uhciEndpoint)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_ENDPOINT UhciEndpoint = uhciEndpoint;
//     ULONG TransferType;
//
//     DPRINT_UHCI!("UhciPollEndpoint: ...\n");
//
//     TransferType = UhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         UhciPollIsoEndpoint(UhciExtension, UhciEndpoint);
//         return;
//     }
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         UhciPollNonIsoEndpoint(UhciExtension, UhciEndpoint);
//     }
// }
//
// VOID
// NTAPI
// UhciCheckController(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//
//     if (!UhciHardwarePresent(UhciExtension) ||
//         UhciExtension->HcScheduleError >= UHCI_MAX_HC_SCHEDULE_ERRORS)
//     {
//        DPRINT1("UhciCheckController: INVALIDATE_CONTROLLER_SURPRISE_REMOVE !!!\n");
//
//        RegPacket.UsbPortInvalidateController(UhciExtension,
//                                              USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE);
//     }
// }
//
// ULONG
// NTAPI
// UhciGet32BitFrameNumber(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     ULONG Uhci32BitFrame;
//     USHORT Fn; // FrameNumber
//     ULONG Hp; // FrameHighPart
//
//     Fn = READ_PORT_USHORT(&UhciExtension->BaseRegister->FrameNumber);
//     Fn &= UHCI_FRNUM_FRAME_MASK;
//     Hp = UhciExtension->FrameHighPart;
//
//     Uhci32BitFrame = Hp;
//     Uhci32BitFrame += ((USHORT)Hp ^ Fn) & UHCI_FRNUM_OVERFLOW_LIST;
//     Uhci32BitFrame |= Fn;
//
//     DPRINT_UHCI!("UhciGet32BitFrameNumber: Uhci32BitFrame - %lX\n",
//                 Uhci32BitFrame);
//
//     return Uhci32BitFrame;
// }
//
// VOID
// NTAPI
// UhciInterruptNextSOF(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HC_RESOURCES UhciResources;
//     ULONG CurrentFrame;
//     PUHCI_HCD_TD SOF_HcdTDs;
//     ULONG ix;
//     ULONG NextFrame;
//     ULONG SofFrame;
//     ULONG Idx;
//
//     DPRINT_UHCI!("UhciInterruptNextSOF: ...\n");
//
//     CurrentFrame = UhciGet32BitFrameNumber(UhciExtension);
//
//     SOF_HcdTDs = UhciExtension->SOF_HcdTDs;
//     NextFrame = CurrentFrame + 2;
//
//     for (ix = 0; ix < UHCI_MAX_STATIC_SOF_TDS; ++ix)
//     {
//         SofFrame = SOF_HcdTDs->Frame;
//
//         if (SofFrame == NextFrame)
//             break;
//
//         if (SofFrame < CurrentFrame)
//         {
//             SOF_HcdTDs->Frame = NextFrame;
//             SOF_HcdTDs->Flags |= UHCI_HCD_TD_FLAG_GOOD_FRAME;
//
//             /* Insert SOF_HcdTD (InterruptOnComplete = TRUE) in Frame List */
//             UhciResources = UhciExtension->HcResourcesVA;
//             Idx = SOF_HcdTDs->Frame & UHCI_FRAME_LIST_INDEX_MASK;
//
//             InterlockedExchange((PLONG)&SOF_HcdTDs->HwTD.NextElement,
//                                 UhciResources->FrameList[Idx]);
//
//             UhciResources->FrameList[Idx] = SOF_HcdTDs->PhysicalAddress;
//             break;
//         }
//
//         /* Go to next SOF_HcdTD */
//         SOF_HcdTDs += 1;
//     }
//
//     for (ix = 0; ix < UHCI_MAX_STATIC_SOF_TDS; ++ix)
//     {
//         SOF_HcdTDs = &UhciExtension->SOF_HcdTDs[ix];
//
//         if (SOF_HcdTDs->Frame &&
//             (SOF_HcdTDs->Frame < CurrentFrame ||
//              (SOF_HcdTDs->Frame - CurrentFrame) > UHCI_FRAME_LIST_MAX_ENTRIES))
//         {
//             SOF_HcdTDs->Frame = 0;
//         }
//     }
// }
//
// VOID
// NTAPI
// UhciEnableInterrupts(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     UHCI_PCI_LEGSUP LegacySupport;
//
//     DPRINT("UhciEnableInterrupts: UhciExtension - %p\n", UhciExtension);
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                           TRUE,
//                                           &LegacySupport.AsUSHORT,
//                                           PCI_LEGSUP,
//                                           sizeof(USHORT));
//
//     LegacySupport.UsbPIRQ = 1;
//
//     RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                           FALSE,
//                                           &LegacySupport.AsUSHORT,
//                                           PCI_LEGSUP,
//                                           sizeof(USHORT));
//
//     WRITE_PORT_USHORT(&BaseRegister->HcInterruptEnable.AsUSHORT,
//                       UhciExtension->StatusMask.AsUSHORT);
// }
//
// VOID
// NTAPI
// UhciDisableInterrupts(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     USB_CONTROLLER_FLAVOR HcFlavor;
//     UHCI_PCI_LEGSUP LegacySupport;
//
//     DPRINT("UhciDisableInterrupts: UhciExtension - %p\n", UhciExtension);
//
//     BaseRegister = UhciExtension->BaseRegister;
//     WRITE_PORT_USHORT(&BaseRegister->HcInterruptEnable.AsUSHORT, 0);
//
//     HcFlavor = UhciExtension->HcFlavor;
//     DPRINT("UhciDisableInterrupts: FIXME HcFlavor - %lx\n", HcFlavor);
//
//     RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                           TRUE,
//                                           &LegacySupport.AsUSHORT,
//                                           PCI_LEGSUP,
//                                           sizeof(USHORT));
//
//     LegacySupport.UsbPIRQ = 0;
//
//     RegPacket.UsbPortReadWriteConfigSpace(UhciExtension,
//                                           FALSE,
//                                           &LegacySupport.AsUSHORT,
//                                           PCI_LEGSUP,
//                                           sizeof(USHORT));
// }
//
// VOID
// NTAPI
// UhciPollController(IN PVOID uhciExtension)
// {
//     PUHCI_EXTENSION UhciExtension = uhciExtension;
//     PUHCI_HW_REGISTERS BaseRegister;
//     PUSHORT PortRegister;
//     UHCI_PORT_STATUS_CONTROL PortControl;
//     USHORT Port;
//
//     DPRINT_UHCI!("UhciPollController: UhciExtension - %p\n", UhciExtension);
//
//     BaseRegister = UhciExtension->BaseRegister;
//
//     if (!(UhciExtension->Flags & UHCI_EXTENSION_FLAG_SUSPENDED))
//     {
//         UhciCleanupFrameList(UhciExtension, FALSE);
//         UhciUpdateCounter(UhciExtension);
//         RegPacket.UsbPortInvalidateRootHub(UhciExtension);
//         return;
//     }
//
//     for (Port = 0; Port < UHCI_NUM_ROOT_HUB_PORTS; Port++)
//     {
//         PortRegister = (PUSHORT)&BaseRegister->PortControl[Port];
//         PortControl.AsUSHORT = READ_PORT_USHORT(PortRegister);
//
//         if (PortControl.ConnectStatusChange == 1)
//             RegPacket.UsbPortInvalidateRootHub(UhciExtension);
//     }
// }
//
// VOID
// NTAPI
// UhciSetEndpointDataToggle(IN PVOID uhciExtension,
//                           IN PVOID uhciEndpoint,
//                           IN ULONG DataToggle)
// {
//     DPRINT_IMPL!("UhciSetEndpointDataToggle: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// UhciResetController(IN PVOID uhciExtension)
// {
//     DPRINT_IMPL!("UhciResetController: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// UhciStartSendOnePacket(IN PVOID uhciExtension,
//                        IN PVOID PacketParameters,
//                        IN PVOID Data,
//                        IN PULONG pDataLength,
//                        IN PVOID BufferVA,
//                        IN PVOID BufferPA,
//                        IN ULONG BufferLength,
//                        IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT_IMPL!("UhciStartSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciEndSendOnePacket(IN PVOID uhciExtension,
//                      IN PVOID PacketParameters,
//                      IN PVOID Data,
//                      IN PULONG pDataLength,
//                      IN PVOID BufferVA,
//                      IN PVOID BufferPA,
//                      IN ULONG BufferLength,
//                      IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT_IMPL!("UhciEndSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// UhciPassThru(IN PVOID uhciExtension,
//              IN PVOID passThruParameters,
//              IN ULONG ParameterLength,
//              IN PVOID pParameters)
// {
//     DPRINT_IMPL!("UhciPassThru: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// UhciFlushInterrupts(IN PVOID uhciExtension)
// {
//     DPRINT_IMPL!("UhciFlushInterrupts: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// UhciUnload(IN PVOID uhciExtension)
// {
//     DPRINT_IMPL!("UhciUnload: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     DPRINT("DriverEntry: DriverObject - %p, RegistryPath - %p\n", DriverObject, RegistryPath);
//
//     RtlZeroMemory(&RegPacket, sizeof(USBPORT_REGISTRATION_PACKET));
//
//     RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_UHCI;
//
//     RegPacket.MiniPortFlags = USB_MINIPORT_FLAGS_INTERRUPT |
//                               USB_MINIPORT_FLAGS_PORT_IO |
//                               USB_MINIPORT_FLAGS_NOT_LOCK_INT |
//                               USB_MINIPORT_FLAGS_POLLING |
//                               USB_MINIPORT_FLAGS_WAKE_SUPPORT;
//
//     RegPacket.MiniPortBusBandwidth = TOTAL_USB11_BUS_BANDWIDTH;
//
//     RegPacket.MiniPortExtensionSize = sizeof(UHCI_EXTENSION);
//     RegPacket.MiniPortEndpointSize = sizeof(UHCI_ENDPOINT);
//     RegPacket.MiniPortTransferSize = sizeof(UHCI_TRANSFER);
//     RegPacket.MiniPortResourcesSize = sizeof(UHCI_HC_RESOURCES);
//
//     RegPacket.OpenEndpoint = UhciOpenEndpoint;
//     RegPacket.ReopenEndpoint = UhciReopenEndpoint;
//     RegPacket.QueryEndpointRequirements = UhciQueryEndpointRequirements;
//     RegPacket.CloseEndpoint = UhciCloseEndpoint;
//     RegPacket.StartController = UhciStartController;
//     RegPacket.StopController = UhciStopController;
//     RegPacket.SuspendController = UhciSuspendController;
//     RegPacket.ResumeController = UhciResumeController;
//     RegPacket.InterruptService = UhciInterruptService;
//     RegPacket.InterruptDpc = UhciInterruptDpc;
//     RegPacket.SubmitTransfer = UhciSubmitTransfer;
//     RegPacket.SubmitIsoTransfer = UhciIsochTransfer;
//     RegPacket.AbortTransfer = UhciAbortTransfer;
//     RegPacket.GetEndpointState = UhciGetEndpointState;
//     RegPacket.SetEndpointState = UhciSetEndpointState;
//     RegPacket.PollEndpoint = UhciPollEndpoint;
//     RegPacket.CheckController = UhciCheckController;
//     RegPacket.Get32BitFrameNumber = UhciGet32BitFrameNumber;
//     RegPacket.InterruptNextSOF = UhciInterruptNextSOF;
//     RegPacket.EnableInterrupts = UhciEnableInterrupts;
//     RegPacket.DisableInterrupts = UhciDisableInterrupts;
//     RegPacket.PollController = UhciPollController;
//     RegPacket.SetEndpointDataToggle = UhciSetEndpointDataToggle;
//     RegPacket.GetEndpointStatus = UhciGetEndpointStatus;
//     RegPacket.SetEndpointStatus = UhciSetEndpointStatus;
//     RegPacket.RH_GetRootHubData = UhciRHGetRootHubData;
//     RegPacket.RH_GetStatus = UhciRHGetStatus;
//     RegPacket.RH_GetPortStatus = UhciRHGetPortStatus;
//     RegPacket.RH_GetHubStatus = UhciRHGetHubStatus;
//     RegPacket.RH_SetFeaturePortReset = UhciRHSetFeaturePortReset;
//     RegPacket.RH_SetFeaturePortPower = UhciRHSetFeaturePortPower;
//     RegPacket.RH_SetFeaturePortEnable = UhciRHSetFeaturePortEnable;
//     RegPacket.RH_SetFeaturePortSuspend = UhciRHSetFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnable = UhciRHClearFeaturePortEnable;
//     RegPacket.RH_ClearFeaturePortPower = UhciRHClearFeaturePortPower;
//     RegPacket.RH_ClearFeaturePortSuspend = UhciRHClearFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnableChange = UhciRHClearFeaturePortEnableChange;
//     RegPacket.RH_ClearFeaturePortConnectChange = UhciRHClearFeaturePortConnectChange;
//     RegPacket.RH_ClearFeaturePortResetChange = UhciRHClearFeaturePortResetChange;
//     RegPacket.RH_ClearFeaturePortSuspendChange = UhciRHClearFeaturePortSuspendChange;
//     RegPacket.RH_ClearFeaturePortOvercurrentChange = UhciRHClearFeaturePortOvercurrentChange;
//     RegPacket.RH_DisableIrq = UhciRHDisableIrq;
//     RegPacket.RH_EnableIrq = UhciRHEnableIrq;
//     RegPacket.StartSendOnePacket = UhciStartSendOnePacket;
//     RegPacket.EndSendOnePacket = UhciEndSendOnePacket;
//     RegPacket.PassThru = UhciPassThru;
//     RegPacket.FlushInterrupts = UhciFlushInterrupts;
//
//     DriverObject->DriverUnload = NULL;
//
//     return USBPORT_RegisterUSBPortDriver(DriverObject,
//                                          USB10_MINIPORT_INTERFACE_VERSION,
//                                          &RegPacket);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
