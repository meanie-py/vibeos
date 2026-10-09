use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KeStallExecutionProcessor, LARGE_INTEGER, PDRIVER_OBJECT, PUNICODE_STRING};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
}
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: *mut CHAR);
    fn USBPORT_GetHciMn() -> ULONG;
    fn USBPORT_RegisterUSBPortDriver( DriverObject: PDRIVER_OBJECT, Version: ULONG, RegPacket: *mut USBPORT_REGISTRATION_PACKET, ) -> NTSTATUS;
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

static mut ClassicPeriod: [UCHAR; 8] = [
    (ENDPOINT_INTERRUPT_1ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_2ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_4ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_8ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_16ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - 1 as ::core::ffi::c_int) as UCHAR,
    (ENDPOINT_INTERRUPT_32ms - 1 as ::core::ffi::c_int) as UCHAR,
];

static mut pTable: [EHCI_PERIOD; 64] = [
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_1ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0xff as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_2ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x55 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_2ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0xaa as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_4ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x11 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_4ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x44 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_4ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x22 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_4ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x88 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_8ms as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x1 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_16ms as UCHAR,
        PeriodIdx: 0x2 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x1 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x10 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x4 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x40 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x2 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x20 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x8 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x3 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x5 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x4 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: ENDPOINT_INTERRUPT_32ms as UCHAR,
        PeriodIdx: 0x6 as UCHAR,
        ScheduleMask: 0x80 as UCHAR,
    },
    _EHCI_PERIOD {
        Period: 0 as UCHAR,
        PeriodIdx: 0 as UCHAR,
        ScheduleMask: 0 as UCHAR,
    },
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

static mut LinkTable: [UCHAR; 64] = [
    255 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
    1 as ::core::ffi::c_int as UCHAR,
    2 as ::core::ffi::c_int as UCHAR,
    2 as ::core::ffi::c_int as UCHAR,
    3 as ::core::ffi::c_int as UCHAR,
    3 as ::core::ffi::c_int as UCHAR,
    4 as ::core::ffi::c_int as UCHAR,
    4 as ::core::ffi::c_int as UCHAR,
    5 as ::core::ffi::c_int as UCHAR,
    5 as ::core::ffi::c_int as UCHAR,
    6 as ::core::ffi::c_int as UCHAR,
    6 as ::core::ffi::c_int as UCHAR,
    7 as ::core::ffi::c_int as UCHAR,
    7 as ::core::ffi::c_int as UCHAR,
    8 as ::core::ffi::c_int as UCHAR,
    8 as ::core::ffi::c_int as UCHAR,
    9 as ::core::ffi::c_int as UCHAR,
    9 as ::core::ffi::c_int as UCHAR,
    10 as ::core::ffi::c_int as UCHAR,
    10 as ::core::ffi::c_int as UCHAR,
    11 as ::core::ffi::c_int as UCHAR,
    11 as ::core::ffi::c_int as UCHAR,
    12 as ::core::ffi::c_int as UCHAR,
    12 as ::core::ffi::c_int as UCHAR,
    13 as ::core::ffi::c_int as UCHAR,
    13 as ::core::ffi::c_int as UCHAR,
    14 as ::core::ffi::c_int as UCHAR,
    14 as ::core::ffi::c_int as UCHAR,
    15 as ::core::ffi::c_int as UCHAR,
    15 as ::core::ffi::c_int as UCHAR,
    16 as ::core::ffi::c_int as UCHAR,
    16 as ::core::ffi::c_int as UCHAR,
    17 as ::core::ffi::c_int as UCHAR,
    17 as ::core::ffi::c_int as UCHAR,
    18 as ::core::ffi::c_int as UCHAR,
    18 as ::core::ffi::c_int as UCHAR,
    19 as ::core::ffi::c_int as UCHAR,
    19 as ::core::ffi::c_int as UCHAR,
    20 as ::core::ffi::c_int as UCHAR,
    20 as ::core::ffi::c_int as UCHAR,
    21 as ::core::ffi::c_int as UCHAR,
    21 as ::core::ffi::c_int as UCHAR,
    22 as ::core::ffi::c_int as UCHAR,
    22 as ::core::ffi::c_int as UCHAR,
    23 as ::core::ffi::c_int as UCHAR,
    23 as ::core::ffi::c_int as UCHAR,
    24 as ::core::ffi::c_int as UCHAR,
    24 as ::core::ffi::c_int as UCHAR,
    25 as ::core::ffi::c_int as UCHAR,
    25 as ::core::ffi::c_int as UCHAR,
    26 as ::core::ffi::c_int as UCHAR,
    26 as ::core::ffi::c_int as UCHAR,
    27 as ::core::ffi::c_int as UCHAR,
    27 as ::core::ffi::c_int as UCHAR,
    28 as ::core::ffi::c_int as UCHAR,
    28 as ::core::ffi::c_int as UCHAR,
    29 as ::core::ffi::c_int as UCHAR,
    29 as ::core::ffi::c_int as UCHAR,
    30 as ::core::ffi::c_int as UCHAR,
    30 as ::core::ffi::c_int as UCHAR,
    0 as ::core::ffi::c_int as UCHAR,
];

const EHCI_TOKEN_STATUS_HALTED: i32 = 1 << 6;
const EHCI_TOKEN_STATUS_TRANSACTION_ERROR: i32 = 1 << 3;
const EHCI_TOKEN_STATUS_BABBLE_DETECTED: i32 = 1 << 4;
const EHCI_TOKEN_STATUS_DATA_BUFFER_ERROR: i32 = 1 << 5;
const EHCI_TOKEN_STATUS_MISSED_MICROFRAME: i32 = 1 << 2;

#[no_mangle]
pub unsafe extern "system" fn EHCI_AllocTd(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) -> PEHCI_HCD_TD {
    let mut TD: PEHCI_HCD_TD;
    let mut ix: ULONG;

    DPRINT_EHCI!(b"EHCI_AllocTd: ... \n\0".as_ptr() as *const i8);

    if (*EhciEndpoint).MaxTDs == 0 {
        (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(EhciExtension as PVOID);
        return core::ptr::null_mut();
    }

    TD = (*EhciEndpoint).FirstTD;

    ix = 1 as ULONG;
    while (*TD).TdFlags as core::ffi::c_ulong & EHCI_HCD_TD_FLAG_ALLOCATED as core::ffi::c_ulong != 0
    {
        TD = TD.offset(1);

        if ix >= (*EhciEndpoint).MaxTDs {
            (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(EhciExtension as PVOID);
            return core::ptr::null_mut();
        }

        ix = ix.wrapping_add(1);
    }

    (*TD).TdFlags |= EHCI_HCD_TD_FLAG_ALLOCATED as core::ffi::c_ulong;

    (*EhciEndpoint).RemainTDs = (*EhciEndpoint).RemainTDs.wrapping_sub(1);

    return TD;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InitializeQH(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    QH: PEHCI_HCD_QH,
    QhPA: ULONG,
) -> PEHCI_HCD_QH {
    let EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES;
    let DeviceSpeed: ULONG;

    DPRINT_EHCI!(
        b"EHCI_InitializeQH: EhciEndpoint - %p, QH - %p, QhPA - %p\n\0".as_ptr() as *const i8,
        EhciEndpoint,
        QH,
        QhPA,
    );

    EndpointProperties =
        core::ptr::addr_of_mut!((*EhciEndpoint).EndpointProperties) as PUSBPORT_ENDPOINT_PROPERTIES;

    RtlZeroMemory(QH as *mut core::ffi::c_void, core::mem::size_of::<EHCI_HCD_QH>());

    // ASSERT((QhPA & ~LINK_POINTER_MASK) == 0);

    (*QH).sqh.PhysicalAddress = QhPA;

    (*QH).sqh.HwQH.EndpointParams.set_DeviceAddress((*EndpointProperties).DeviceAddress as ULONG);
    (*QH).sqh.HwQH.EndpointParams.set_EndpointNumber((*EndpointProperties).EndpointAddress as ULONG);

    DeviceSpeed = (*EndpointProperties).DeviceSpeed as ULONG;

    match DeviceSpeed {
        UsbLowSpeed => {
            (*QH).sqh.HwQH.EndpointParams.set_EndpointSpeed(EHCI_QH_EP_LOW_SPEED as ULONG);
        }
        UsbFullSpeed => {
            (*QH).sqh.HwQH.EndpointParams.set_EndpointSpeed(EHCI_QH_EP_FULL_SPEED as ULONG);
        }
        UsbHighSpeed => {
            (*QH).sqh.HwQH.EndpointParams.set_EndpointSpeed(EHCI_QH_EP_HIGH_SPEED as ULONG);
        }
        _ => {
            // DPRINT removed
            // ASSERT(FALSE);
        }
    }

    (*QH).sqh.HwQH.EndpointParams.set_MaximumPacketLength((*EndpointProperties).MaxPacketSize as ULONG);
    (*QH).sqh.HwQH.EndpointCaps.set_PipeMultiplier(1 as ULONG);

    if DeviceSpeed == UsbHighSpeed as ULONG {
        (*QH).sqh.HwQH.EndpointCaps.set_HubAddr(0 as ULONG);
        (*QH).sqh.HwQH.EndpointCaps.set_PortNumber(0 as ULONG);
    } else {
        (*QH).sqh.HwQH.EndpointCaps.set_HubAddr((*EndpointProperties).HubAddr as ULONG);
        (*QH).sqh.HwQH.EndpointCaps.set_PortNumber((*EndpointProperties).PortNumber as ULONG);

        if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
            (*QH).sqh.HwQH.EndpointParams.set_ControlEndpointFlag(1 as ULONG);
        }
    }

    (*QH).sqh.HwQH.NextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*QH).sqh.HwQH.Token.set_Status(
        (*QH).sqh.HwQH.Token.__bindgen_anon_1.Status()
            & !(EHCI_TOKEN_STATUS_ACTIVE | EHCI_TOKEN_STATUS_HALTED) as UCHAR as ULONG,
    );

    return QH;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_OpenBulkOrControlEndpoint(
    EhciExtension: PEHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EhciEndpoint: PEHCI_ENDPOINT,
    IsControl: BOOLEAN,
) -> MPSTATUS {
    let mut QH: PEHCI_HCD_QH;
    let mut QhPA: ULONG;
    let mut TdVA: PEHCI_HCD_TD;
    let mut TdPA: ULONG;
    let mut TD: PEHCI_HCD_TD;
    let mut TdCount: ULONG;
    let mut ix: ULONG;

    InitializeListHead(core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs));

    (*EhciEndpoint).DmaBufferVA = (*EndpointProperties).BufferVA as PVOID;
    (*EhciEndpoint).DmaBufferPA = (*EndpointProperties).BufferPA;

    RtlZeroMemory(
        (*EhciEndpoint).DmaBufferVA,
        core::mem::size_of::<EHCI_HCD_TD>() as SIZE_T,
    );

    QH = ((*EhciEndpoint).DmaBufferVA as PEHCI_HCD_QH).offset(1);
    QhPA = (*EhciEndpoint).DmaBufferPA + core::mem::size_of::<EHCI_HCD_TD>() as ULONG;

    (*EhciEndpoint).FirstTD = QH.offset(1) as PEHCI_HCD_TD;

    TdCount = ((*EndpointProperties).BufferLength
        - (core::mem::size_of::<EHCI_HCD_TD>() as ULONG
            + core::mem::size_of::<EHCI_HCD_QH>() as ULONG))
        / core::mem::size_of::<EHCI_HCD_TD>() as ULONG;

    if (*EndpointProperties).TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG {
        (*EhciEndpoint).EndpointStatus |= USBPORT_ENDPOINT_CONTROL as ULONG;
    }

    (*EhciEndpoint).MaxTDs = TdCount;
    (*EhciEndpoint).RemainTDs = TdCount;

    TdVA = (*EhciEndpoint).FirstTD;
    TdPA = QhPA + core::mem::size_of::<EHCI_HCD_QH>() as ULONG;

    ix = 0;
    while ix < TdCount {
        RtlZeroMemory(TdVA as PVOID, core::mem::size_of::<EHCI_HCD_TD>() as _);

        // ASSERT((TdPA & ~LINK_POINTER_MASK) == 0);

        (*TdVA).PhysicalAddress = TdPA;
        (*TdVA).EhciEndpoint = EhciEndpoint;
        (*TdVA).EhciTransfer = core::ptr::null_mut();

        TdPA += core::mem::size_of::<EHCI_HCD_TD>() as ULONG;
        TdVA = TdVA.offset(1);

        ix = ix.wrapping_add(1);
    }

    (*EhciEndpoint).QH = EHCI_InitializeQH(EhciExtension, EhciEndpoint, QH, QhPA);

    if IsControl != 0 {
        (*QH).sqh.HwQH.EndpointParams.set_DataToggleControl(1);
        (*EhciEndpoint).HcdHeadP = core::ptr::null_mut();
    } else {
        (*QH).sqh.HwQH.EndpointParams.set_DataToggleControl(0);
    }

    TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

    if TD.is_null() {
        return MP_STATUS_NO_RESOURCES as MPSTATUS;
    }

    (*TD).TdFlags |= EHCI_HCD_TD_FLAG_DUMMY as ULONG;
    (*TD).HwTD.Token.set_Status(
        (*TD).HwTD.Token.__bindgen_anon_1.Status() & !(EHCI_TOKEN_STATUS_ACTIVE as UCHAR) as ULONG,
    );

    (*TD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
    (*TD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*TD).NextHcdTD = core::ptr::null_mut();
    (*TD).AltNextHcdTD = core::ptr::null_mut();

    (*EhciEndpoint).HcdTailP = TD;
    (*EhciEndpoint).HcdHeadP = TD;

    (*QH).sqh.HwQH.CurrentTD = (*TD).PhysicalAddress;
    (*QH).sqh.HwQH.NextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*QH).sqh.HwQH.Token.set_Status(
        (*QH).sqh.HwQH.Token.__bindgen_anon_1.Status() & !(EHCI_TOKEN_STATUS_ACTIVE as UCHAR) as ULONG,
    );
    (*QH).sqh.HwQH.Token.set_TransferBytes(0);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_OpenInterruptEndpoint(
    EhciExtension: PEHCI_EXTENSION,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EhciEndpoint: PEHCI_ENDPOINT,
) -> MPSTATUS {
    let mut QH: PEHCI_HCD_QH = core::ptr::null_mut();
    let mut QhPA: ULONG;
    let mut FirstTD: PEHCI_HCD_TD;
    let mut FirstTdPA: ULONG;
    let mut TD: PEHCI_HCD_TD;
    let mut DummyTD: PEHCI_HCD_TD;
    let mut TdCount: ULONG;
    let mut ix: ULONG;
    let mut PeriodTable: *const EHCI_PERIOD = core::ptr::null();
    let mut ScheduleOffset: ULONG;
    let mut Idx: ULONG = 0;
    let mut Period: UCHAR;

    Period = (*EndpointProperties).Period;
    ScheduleOffset = (*EndpointProperties).ScheduleOffset;

    while Period as i32 & 1 == 0 {
        Idx = Idx.wrapping_add(1);
        Period = (Period as i32 >> 1) as UCHAR;
    }

    InitializeListHead(core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs));

    if (*EhciEndpoint).EndpointProperties.DeviceSpeed as u32 == UsbHighSpeed as u32 {
        PeriodTable = (core::ptr::addr_of!(pTable) as *const EHCI_PERIOD).offset(
            (*(core::ptr::addr_of!(ClassicPeriod) as *const UCHAR).offset(Idx as isize) as ULONG)
                .wrapping_add(ScheduleOffset) as isize,
        ) as *const EHCI_PERIOD;
        (*EhciEndpoint).PeriodTable = PeriodTable;

        (*EhciEndpoint).StaticQH = (*EhciExtension).PeriodicHead[(*PeriodTable).PeriodIdx as usize];
    } else {
        (*EhciEndpoint).PeriodTable = core::ptr::null();

        (*EhciEndpoint).StaticQH = (*EhciExtension).PeriodicHead
            [(ClassicPeriod[Idx as usize] as ULONG).wrapping_add(ScheduleOffset) as usize];
    }

    (*EhciEndpoint).DmaBufferVA = (*EndpointProperties).BufferVA as PVOID;
    (*EhciEndpoint).DmaBufferPA = (*EndpointProperties).BufferPA;

    RtlZeroMemory(
        (*EndpointProperties).BufferVA as PVOID,
        core::mem::size_of::<EHCI_HCD_TD>(),
    );

    QH = ((*EndpointProperties).BufferVA as usize)
        .wrapping_add(core::mem::size_of::<EHCI_HCD_TD>()) as PEHCI_HCD_QH;
    QhPA = ((*EndpointProperties).BufferPA as usize)
        .wrapping_add(core::mem::size_of::<EHCI_HCD_TD>()) as ULONG;

    FirstTD = ((*EndpointProperties).BufferVA as usize)
        .wrapping_add(core::mem::size_of::<EHCI_HCD_TD>())
        .wrapping_add(core::mem::size_of::<EHCI_HCD_QH>()) as PEHCI_HCD_TD;

    FirstTdPA = ((*EndpointProperties).BufferPA as usize)
        .wrapping_add(core::mem::size_of::<EHCI_HCD_TD>())
        .wrapping_add(core::mem::size_of::<EHCI_HCD_QH>()) as ULONG;

    TdCount = ((*EndpointProperties).BufferLength as usize)
        .wrapping_sub(
            (core::mem::size_of::<EHCI_HCD_TD>()).wrapping_add(core::mem::size_of::<EHCI_HCD_QH>()),
        )
        .wrapping_div(core::mem::size_of::<EHCI_HCD_TD>()) as ULONG;

    (*EhciEndpoint).FirstTD = FirstTD;
    (*EhciEndpoint).MaxTDs = TdCount;

    ix = 0;
    while ix < TdCount {
        TD = (*EhciEndpoint).FirstTD.offset(ix as isize);

        RtlZeroMemory(TD as PVOID, core::mem::size_of::<EHCI_HCD_TD>());

        (*TD).PhysicalAddress = FirstTdPA;
        (*TD).EhciEndpoint = EhciEndpoint;
        (*TD).EhciTransfer = core::ptr::null_mut();

        FirstTdPA = (FirstTdPA as u64)
            .wrapping_add(core::mem::size_of::<EHCI_HCD_TD>() as u64) as ULONG;

        ix = ix.wrapping_add(1);
    }

    (*EhciEndpoint).RemainTDs = TdCount;

    (*EhciEndpoint).QH = EHCI_InitializeQH(EhciExtension, EhciEndpoint, QH, QhPA);

    if (*EhciEndpoint).EndpointProperties.DeviceSpeed as u32 == UsbHighSpeed as u32 {
        (*QH)
            .sqh
            .HwQH
            .EndpointCaps
            .set_InterruptMask((*PeriodTable).ScheduleMask as ULONG);
    } else {
        (*QH)
            .sqh
            .HwQH
            .EndpointCaps
            .set_InterruptMask((*EndpointProperties).InterruptScheduleMask as ULONG);

        (*QH)
            .sqh
            .HwQH
            .EndpointCaps
            .set_SplitCompletionMask((*EndpointProperties).SplitCompletionMask as ULONG);
    }

    DummyTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

    (*DummyTD).TdFlags |= EHCI_HCD_TD_FLAG_DUMMY as ULONG;
    (*DummyTD).NextHcdTD = core::ptr::null_mut();
    (*DummyTD).AltNextHcdTD = core::ptr::null_mut();

    (*DummyTD).HwTD.Token.set_Status(
        (*DummyTD).HwTD.Token.__bindgen_anon_1.Status() & !(EHCI_TOKEN_STATUS_ACTIVE as ULONG),
    );

    (*DummyTD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
    (*DummyTD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*EhciEndpoint).HcdTailP = DummyTD;
    (*EhciEndpoint).HcdHeadP = DummyTD;

    (*QH).sqh.HwQH.CurrentTD = (*DummyTD).PhysicalAddress;
    (*QH).sqh.HwQH.NextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*QH).sqh.HwQH.Token.set_Status(
        (*QH).sqh.HwQH.Token.__bindgen_anon_1.Status() & !(EHCI_TOKEN_STATUS_ACTIVE as ULONG),
    );
    (*QH).sqh.HwQH.Token.set_TransferBytes(0 as ULONG);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_OpenHsIsoEndpoint(
    mut EhciExtension: PEHCI_EXTENSION,
    mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    mut EhciEndpoint: PEHCI_ENDPOINT,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_OpenHsIsoEndpoint: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        478 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            478 as ::core::ffi::c_int,
        );
    }
    return MP_STATUS_NOT_SUPPORTED as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_OpenIsoEndpoint(
    mut EhciExtension: PEHCI_EXTENSION,
    mut EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    mut EhciEndpoint: PEHCI_ENDPOINT,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_OpenIsoEndpoint: UNIMPLEMENTED. FIXME\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        488 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            488 as ::core::ffi::c_int,
        );
    }
    return MP_STATUS_NOT_SUPPORTED as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_OpenEndpoint(
    ehciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    ehciEndpoint: PVOID,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let EhciEndpoint: PEHCI_ENDPOINT = ehciEndpoint as PEHCI_ENDPOINT;
    let TransferType: ULONG;
    let MPStatus: MPSTATUS;

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*EhciEndpoint).EndpointProperties) as *mut core::ffi::c_void,
        EndpointProperties as *const core::ffi::c_void,
        core::mem::size_of::<USBPORT_ENDPOINT_PROPERTIES>() as usize,
    );

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
            if (*EndpointProperties).DeviceSpeed as u32 == UsbHighSpeed as u32 {
                MPStatus =
                    EHCI_OpenHsIsoEndpoint(EhciExtension, EndpointProperties, EhciEndpoint);
            } else {
                MPStatus =
                    EHCI_OpenIsoEndpoint(EhciExtension, EndpointProperties, EhciEndpoint);
            }
        }
        USBPORT_TRANSFER_TYPE_CONTROL => {
            MPStatus = EHCI_OpenBulkOrControlEndpoint(
                EhciExtension,
                EndpointProperties,
                EhciEndpoint,
                TRUE as BOOLEAN,
            );
        }
        USBPORT_TRANSFER_TYPE_BULK => {
            MPStatus = EHCI_OpenBulkOrControlEndpoint(
                EhciExtension,
                EndpointProperties,
                EhciEndpoint,
                FALSE as BOOLEAN,
            );
        }
        USBPORT_TRANSFER_TYPE_INTERRUPT => {
            MPStatus =
                EHCI_OpenInterruptEndpoint(EhciExtension, EndpointProperties, EhciEndpoint);
        }
        _ => {
            MPStatus = MP_STATUS_NOT_SUPPORTED as MPSTATUS;
        }
    }

    return MPStatus;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_ReopenEndpoint(
    ehciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    ehciEndpoint: PVOID,
) -> MPSTATUS {
    let EhciEndpoint: PEHCI_ENDPOINT;
    let TransferType: ULONG;
    let QH: PEHCI_HCD_QH;
    let MPStatus: MPSTATUS;

    EhciEndpoint = ehciEndpoint as PEHCI_ENDPOINT;

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        USBPORT_TRANSFER_TYPE_ISOCHRONOUS => {
            if (*EndpointProperties).DeviceSpeed as u32 == UsbHighSpeed as u32 {
                MPStatus = MP_STATUS_NOT_SUPPORTED as MPSTATUS;
            } else {
                MPStatus = MP_STATUS_NOT_SUPPORTED as MPSTATUS;
            }
        }
        USBPORT_TRANSFER_TYPE_CONTROL
        | USBPORT_TRANSFER_TYPE_BULK
        | USBPORT_TRANSFER_TYPE_INTERRUPT => {
            RtlCopyMemory(
                core::ptr::addr_of_mut!((*EhciEndpoint).EndpointProperties) as *mut core::ffi::c_void,
                EndpointProperties as *const core::ffi::c_void,
                core::mem::size_of::<USBPORT_ENDPOINT_PROPERTIES>() as usize,
            );

            QH = (*EhciEndpoint).QH;

            (*QH).sqh.HwQH.EndpointParams.set_DeviceAddress((*EndpointProperties).DeviceAddress as ULONG);
            (*QH).sqh.HwQH.EndpointParams.set_MaximumPacketLength((*EndpointProperties).MaxPacketSize as ULONG);

            (*QH).sqh.HwQH.EndpointCaps.set_HubAddr((*EndpointProperties).HubAddr as ULONG);

            MPStatus = MP_STATUS_SUCCESS as MPSTATUS;
        }
        _ => {
            MPStatus = MP_STATUS_SUCCESS as MPSTATUS;
        }
    }

    return MPStatus;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_QueryEndpointRequirements(
    ehciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    EndpointRequirements: PUSBPORT_ENDPOINT_REQUIREMENTS,
) {
    let mut TransferType: ULONG;

    TransferType = (*EndpointProperties).TransferType;

    match TransferType {
        0 => {
            if (*EndpointProperties).DeviceSpeed as u32 == UsbHighSpeed as u32 {
                (*EndpointRequirements).HeaderBufferSize =
                    EHCI_MAX_HS_ISO_HEADER_BUFFER_SIZE as ULONG;
                (*EndpointRequirements).MaxTransferSize = EHCI_MAX_HS_ISO_TRANSFER_SIZE as ULONG;
            } else {
                (*EndpointRequirements).HeaderBufferSize =
                    EHCI_MAX_FS_ISO_HEADER_BUFFER_SIZE as ULONG;
                (*EndpointRequirements).MaxTransferSize = EHCI_MAX_FS_ISO_TRANSFER_SIZE as ULONG;
            }
        }
        1 => {
            (*EndpointRequirements).HeaderBufferSize = (core::mem::size_of::<EHCI_HCD_TD>()
                .wrapping_add(core::mem::size_of::<EHCI_HCD_QH>())
                .wrapping_add(
                    (EHCI_MAX_CONTROL_TD_COUNT as usize)
                        .wrapping_mul(core::mem::size_of::<EHCI_HCD_TD>()),
                )) as ULONG;
            (*EndpointRequirements).MaxTransferSize = EHCI_MAX_CONTROL_TRANSFER_SIZE as ULONG;
        }
        2 => {
            (*EndpointRequirements).HeaderBufferSize = (core::mem::size_of::<EHCI_HCD_TD>()
                .wrapping_add(core::mem::size_of::<EHCI_HCD_QH>())
                .wrapping_add(
                    (EHCI_MAX_BULK_TD_COUNT as usize)
                        .wrapping_mul(core::mem::size_of::<EHCI_HCD_TD>()),
                )) as ULONG;
            (*EndpointRequirements).MaxTransferSize = EHCI_MAX_BULK_TRANSFER_SIZE as ULONG;
        }
        3 => {
            (*EndpointRequirements).HeaderBufferSize = (core::mem::size_of::<EHCI_HCD_TD>()
                .wrapping_add(core::mem::size_of::<EHCI_HCD_QH>())
                .wrapping_add(
                    (EHCI_MAX_INTERRUPT_TD_COUNT as usize)
                        .wrapping_mul(core::mem::size_of::<EHCI_HCD_TD>()),
                )) as ULONG;
            (*EndpointRequirements).MaxTransferSize = EHCI_MAX_INTERRUPT_TRANSFER_SIZE as ULONG;
        }
        _ => {
            DbgBreakPoint();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_DisablePeriodicList(EhciExtension: PEHCI_EXTENSION) {
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();

    if ((*EhciExtension).Flags as u32 & EHCI_FLAGS_IDLE_SUPPORT as u32) != 0 {
        OperationalRegs = (*EhciExtension).OperationalRegs;

        Command.AsULONG = READ_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        );
        Command.set_PeriodicEnable(0);
        WRITE_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
            Command.AsULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_CloseEndpoint(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    DisablePeriodic: BOOLEAN,
) {
    const USBPORT_TRANSFER_TYPE_ISOCHRONOUS: ULONG = 0;
    const USBPORT_TRANSFER_TYPE_INTERRUPT: ULONG = 0x03;

    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let EhciEndpoint: PEHCI_ENDPOINT = ehciEndpoint as PEHCI_ENDPOINT;
    let mut TransferType: ULONG;

    if DbgPrint(
        b"(%s:%d) EHCI_CloseEndpoint: EhciEndpoint - %p, DisablePeriodic - %X\n\0" as *const u8
            as PCSTR,
        b"usbehci.c\0" as *const u8 as *const ::core::ffi::c_char,
        713 as ::core::ffi::c_int,
        EhciEndpoint,
        DisablePeriodic as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"usbehci.c\0" as *const u8 as *const ::core::ffi::c_char,
            713 as ::core::ffi::c_int,
        );
    }

    if DisablePeriodic != 0 {
        TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

        if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS
            || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT
        {
            EHCI_DisablePeriodicList(EhciExtension);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetQhForFrame(
    EhciExtension: PEHCI_EXTENSION,
    FrameIdx: ULONG,
) -> PEHCI_STATIC_QH {
    return (*EhciExtension).PeriodicHead[Balance[(FrameIdx & (EHCI_FRAMES - 1) as ULONG) as usize] as usize];
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetDummyQhForFrame(
    EhciExtension: PEHCI_EXTENSION,
    Idx: ULONG,
) -> PEHCI_HCD_QH {
    return ((*EhciExtension).IsoDummyQHListVA as usize)
        .wrapping_add((Idx as usize).wrapping_mul(core::mem::size_of::<EHCI_HCD_QH>()))
        as PEHCI_HCD_QH;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_AlignHwStructure(
    EhciExtension: PEHCI_EXTENSION,
    PhysicalAddress: PULONG,
    VirtualAddress: PULONG_PTR,
    Alignment: ULONG,
) {
    let mut PAddress: ULONG;
    let NewPAddress: ULONG;
    let mut VAddress: ULONG_PTR;

    PAddress = *PhysicalAddress as ULONG;
    VAddress = *VirtualAddress as ULONG_PTR;

    NewPAddress = (((*PhysicalAddress).wrapping_add(Alignment).wrapping_sub(1) as usize
        & !(PAGE_SIZE as usize - 1)) as PVOID as ULONG_PTR) as ULONG;

    if NewPAddress
        != (((*PhysicalAddress) as usize & !(PAGE_SIZE as usize - 1)) as PVOID as ULONG_PTR)
            as ULONG
    {
        VAddress = (VAddress as usize)
            .wrapping_add(NewPAddress.wrapping_sub(PAddress) as usize) as ULONG_PTR;
        PAddress = NewPAddress;
    }

    *VirtualAddress = VAddress;
    *PhysicalAddress = PAddress;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_AddDummyQHs(EhciExtension: PEHCI_EXTENSION) {
    let mut HcResourcesVA: PEHCI_HC_RESOURCES;
    let mut DummyQH: PEHCI_HCD_QH;
    let mut DummyQhPA: ULONG;
    let mut EndpointParams: EHCI_QH_EP_PARAMS;
    let mut PAddress: EHCI_LINK_POINTER = core::mem::zeroed();
    let mut Frame: ULONG;

    HcResourcesVA = (*EhciExtension).HcResourcesVA;

    DummyQH = (*EhciExtension).IsoDummyQHListVA;
    DummyQhPA = (*EhciExtension).IsoDummyQHListPA;

    Frame = 0;
    while Frame < EHCI_FRAME_LIST_MAX_ENTRIES as ULONG {
        core::ptr::write_bytes(DummyQH as *mut u8, 0, core::mem::size_of::<EHCI_HCD_QH>());

        PAddress.AsULONG = (*HcResourcesVA).PeriodicFrameList[Frame as usize];

        (*DummyQH).sqh.HwQH.HorizontalLink.AsULONG = PAddress.AsULONG;
        (*DummyQH).sqh.HwQH.CurrentTD = 0;
        (*DummyQH).sqh.HwQH.NextTD = TERMINATE_POINTER as ULONG;
        (*DummyQH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;

        EndpointParams = (*DummyQH).sqh.HwQH.EndpointParams;
        EndpointParams.set_DeviceAddress(0);
        EndpointParams.set_EndpointSpeed(0);
        EndpointParams.set_MaximumPacketLength(EHCI_DUMMYQH_MAX_PACKET_LENGTH as ULONG);
        (*DummyQH).sqh.HwQH.EndpointParams = EndpointParams;

        (*DummyQH).sqh.HwQH.EndpointCaps.AsULONG = 0;
        (*DummyQH).sqh.HwQH.EndpointCaps.set_InterruptMask(0);
        (*DummyQH).sqh.HwQH.EndpointCaps.set_SplitCompletionMask(0);
        (*DummyQH).sqh.HwQH.EndpointCaps.set_PipeMultiplier(1);

        (*DummyQH).sqh.HwQH.Token.set_Status(
            (*DummyQH).sqh.HwQH.Token.__bindgen_anon_1.Status() & !(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG),
        );

        (*DummyQH).sqh.PhysicalAddress = DummyQhPA;
        (*DummyQH).sqh.StaticQH = EHCI_GetQhForFrame(EhciExtension, Frame);

        PAddress.AsULONG = DummyQhPA;
        PAddress.set_Reserved(0);
        PAddress.set_Type(EHCI_LINK_TYPE_QH as ULONG);

        (*HcResourcesVA).PeriodicFrameList[Frame as usize] = PAddress.AsULONG;

        DummyQH = DummyQH.offset(1);
        DummyQhPA = DummyQhPA.wrapping_add(core::mem::size_of::<EHCI_HCD_QH>() as ULONG);
        Frame = Frame.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InitializeInterruptSchedule(EhciExtension: PEHCI_EXTENSION) {
    let mut StaticQH: PEHCI_STATIC_QH;
    let mut ix: ULONG;

    ix = 0;
    while ix < INTERRUPT_ENDPOINTs as ULONG {
        StaticQH = (*EhciExtension).PeriodicHead[ix as usize];

        (*StaticQH).HwQH.EndpointParams.set_HeadReclamationListFlag(0);
        (*StaticQH).HwQH.NextTD |= TERMINATE_POINTER as ULONG;
        (*StaticQH).HwQH.Token.set_Status(
            (*StaticQH).HwQH.Token.__bindgen_anon_1.Status() | EHCI_TOKEN_STATUS_HALTED as UCHAR as ULONG,
        );

        ix = ix.wrapping_add(1);
    }

    ix = 1;
    while ix < INTERRUPT_ENDPOINTs as ULONG {
        StaticQH = (*EhciExtension).PeriodicHead[ix as usize];

        (*StaticQH).PrevHead = core::ptr::null_mut();
        (*StaticQH).NextHead =
            (*EhciExtension).PeriodicHead[LinkTable[ix as usize] as usize] as PEHCI_HCD_QH;

        (*StaticQH).HwQH.HorizontalLink.AsULONG =
            (*(*EhciExtension).PeriodicHead[LinkTable[ix as usize] as usize]).PhysicalAddress;

        (*StaticQH).HwQH.HorizontalLink.set_Type(EHCI_LINK_TYPE_QH as ULONG);
        (*StaticQH).HwQH.EndpointCaps.set_InterruptMask(0xFF as ULONG);

        (*StaticQH).QhFlags |= EHCI_QH_FLAG_STATIC as ULONG;

        if ix < (ENDPOINT_INTERRUPT_8ms - 1) as ULONG {
            (*StaticQH).QhFlags |= EHCI_QH_FLAG_STATIC_FAST as ULONG;
        }

        ix = ix.wrapping_add(1);
    }

    (*(*EhciExtension).PeriodicHead[0]).HwQH.HorizontalLink.set_Terminate(1 as ULONG);

    (*(*EhciExtension).PeriodicHead[0]).QhFlags |=
        (EHCI_QH_FLAG_STATIC | EHCI_QH_FLAG_STATIC_FAST) as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InitializeSchedule(
    EhciExtension: PEHCI_EXTENSION,
    BaseVA: ULONG_PTR,
    BasePA: ULONG,
) -> MPSTATUS {
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut HcResourcesVA: PEHCI_HC_RESOURCES;
    let mut HcResourcesPA: ULONG;
    let mut AsyncHead: PEHCI_STATIC_QH;
    let mut AsyncHeadPA: ULONG;
    let mut PeriodicHead: PEHCI_STATIC_QH;
    let mut PeriodicHeadPA: ULONG;
    let mut StaticQH: PEHCI_STATIC_QH;
    let mut NextLink: EHCI_LINK_POINTER = core::mem::zeroed();
    let mut StaticHeadPA: EHCI_LINK_POINTER = core::mem::zeroed();
    let mut Frame: ULONG;
    let mut ix: ULONG;

    OperationalRegs = (*EhciExtension).OperationalRegs;

    HcResourcesVA = BaseVA as PEHCI_HC_RESOURCES;
    HcResourcesPA = BasePA;

    (*EhciExtension).HcResourcesVA = HcResourcesVA;
    (*EhciExtension).HcResourcesPA = BasePA;

    /* Asynchronous Schedule */

    AsyncHead = core::ptr::addr_of_mut!((*HcResourcesVA).AsyncHead) as PEHCI_STATIC_QH;
    AsyncHeadPA = HcResourcesPA
        .wrapping_add(core::mem::offset_of!(EHCI_HC_RESOURCES, AsyncHead) as ULONG);

    RtlZeroMemory(
        AsyncHead as *mut core::ffi::c_void,
        core::mem::size_of::<EHCI_STATIC_QH>() as SIZE_T,
    );

    NextLink.AsULONG = AsyncHeadPA;
    NextLink.set_Type(EHCI_LINK_TYPE_QH as ULONG);

    (*AsyncHead).HwQH.HorizontalLink = NextLink;
    (*AsyncHead).HwQH.EndpointParams.set_HeadReclamationListFlag(1);
    (*AsyncHead).HwQH.EndpointCaps.set_PipeMultiplier(1);
    (*AsyncHead).HwQH.NextTD |= TERMINATE_POINTER;
    (*AsyncHead).HwQH.Token.set_Status(EHCI_TOKEN_STATUS_HALTED as UCHAR as ULONG);

    (*AsyncHead).PhysicalAddress = AsyncHeadPA;
    (*AsyncHead).NextHead = AsyncHead as PEHCI_HCD_QH;
    (*AsyncHead).PrevHead = (*AsyncHead).NextHead;

    (*EhciExtension).AsyncHead = AsyncHead;

    /* Periodic Schedule */

    PeriodicHead = core::ptr::addr_of_mut!((*HcResourcesVA).PeriodicHead[0]) as PEHCI_STATIC_QH;
    PeriodicHeadPA = HcResourcesPA
        .wrapping_add(core::mem::offset_of!(EHCI_HC_RESOURCES, PeriodicHead) as ULONG);

    ix = 0;
    while ix < (INTERRUPT_ENDPOINTs + 1) as ULONG {
        EHCI_AlignHwStructure(
            EhciExtension,
            core::ptr::addr_of_mut!(PeriodicHeadPA),
            core::ptr::addr_of_mut!(PeriodicHead) as PULONG_PTR,
            80,
        );

        (*EhciExtension).PeriodicHead[ix as usize] = PeriodicHead;
        (*(*EhciExtension).PeriodicHead[ix as usize]).PhysicalAddress = PeriodicHeadPA;

        PeriodicHead = PeriodicHead.offset(1);
        PeriodicHeadPA = PeriodicHeadPA
            .wrapping_add(core::mem::size_of::<EHCI_STATIC_QH>() as ULONG);

        ix = ix.wrapping_add(1);
    }

    EHCI_InitializeInterruptSchedule(EhciExtension);

    Frame = 0;
    while Frame < EHCI_FRAME_LIST_MAX_ENTRIES as ULONG {
        StaticQH = EHCI_GetQhForFrame(EhciExtension, Frame);

        StaticHeadPA.AsULONG = (*StaticQH).PhysicalAddress;
        StaticHeadPA.set_Type(EHCI_LINK_TYPE_QH as ULONG);

        (*HcResourcesVA).PeriodicFrameList[Frame as usize] = StaticHeadPA.AsULONG;

        Frame = Frame.wrapping_add(1);
    }

    (*EhciExtension).IsoDummyQHListVA =
        core::ptr::addr_of_mut!((*HcResourcesVA).IsoDummyQH[0]) as PEHCI_HCD_QH;
    (*EhciExtension).IsoDummyQHListPA = HcResourcesPA
        .wrapping_add(core::mem::offset_of!(EHCI_HC_RESOURCES, IsoDummyQH) as ULONG);

    EHCI_AddDummyQHs(EhciExtension);

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).PeriodicListBase) as *mut ULONG,
        (*EhciExtension).HcResourcesPA
            .wrapping_add(core::mem::offset_of!(EHCI_HC_RESOURCES, PeriodicFrameList) as ULONG),
    );

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).AsyncListBase) as *mut ULONG,
        NextLink.AsULONG,
    );

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InitializeHardware(EhciExtension: PEHCI_EXTENSION) -> MPSTATUS {
    let mut CapabilityRegisters: PEHCI_HC_CAPABILITY_REGISTERS;
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();
    let mut EndTime: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut StructuralParams: EHCI_HC_STRUCTURAL_PARAMS = core::mem::zeroed();

    OperationalRegs = (*EhciExtension).OperationalRegs;
    CapabilityRegisters = (*EhciExtension).CapabilityRegisters;

    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_Reset(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    KeQuerySystemTime(core::ptr::addr_of_mut!(EndTime));
    EndTime.QuadPart += (100 * 10000) as i64;

    loop {
        KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));
        Command.AsULONG =
            READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);

        if Command.Reset() != 1 {
            break;
        }

        if CurrentTime.QuadPart >= EndTime.QuadPart {
            if Command.Reset() == 1 {
                return MP_STATUS_HW_ERROR as MPSTATUS;
            }

            break;
        }
    }

    StructuralParams.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*CapabilityRegisters).StructParameters.AsULONG) as *mut ULONG);

    (*EhciExtension).NumberOfPorts = StructuralParams.PortCount() as USHORT;
    (*EhciExtension).PortPowerControl = StructuralParams.PortPowerControl() as USHORT;

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).PeriodicListBase) as *mut ULONG,
        0 as ULONG,
    );
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).AsyncListBase) as *mut ULONG,
        0 as ULONG,
    );

    (*EhciExtension).InterruptMask.AsULONG = 0 as ULONG;
    (*EhciExtension).InterruptMask.set_Interrupt(1);
    (*EhciExtension).InterruptMask.set_ErrorInterrupt(1);
    (*EhciExtension).InterruptMask.set_PortChangeInterrupt(0);
    (*EhciExtension).InterruptMask.set_FrameListRollover(1);
    (*EhciExtension).InterruptMask.set_HostSystemError(1);
    (*EhciExtension).InterruptMask.set_InterruptOnAsyncAdvance(1);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetOffsetEECP(
    EhciExtension: PEHCI_EXTENSION,
    CapabilityID: UCHAR,
) -> UCHAR {
    let mut LegacyCapability: EHCI_LEGACY_EXTENDED_CAPABILITY = core::mem::zeroed();
    let CapParameters: EHCI_HC_CAPABILITY_PARAMS;
    let mut OffsetEECP: UCHAR;

    CapParameters = (*(*EhciExtension).CapabilityRegisters).CapParameters;

    OffsetEECP = CapParameters.ExtCapabilitiesPointer() as UCHAR;

    if OffsetEECP == 0 {
        return 0;
    }

    loop {
        (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
            EhciExtension as PVOID,
            1 as BOOLEAN,
            core::ptr::addr_of_mut!(LegacyCapability.AsULONG) as PVOID,
            OffsetEECP as ULONG,
            core::mem::size_of::<EHCI_LEGACY_EXTENDED_CAPABILITY>() as ULONG,
        );

        if LegacyCapability.CapabilityID() as i32 == CapabilityID as i32 {
            break;
        }

        OffsetEECP = LegacyCapability.NextCapabilityPointer() as UCHAR;

        if OffsetEECP == 0 {
            return 0;
        }
    }

    return OffsetEECP;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_TakeControlHC(EhciExtension: PEHCI_EXTENSION) -> MPSTATUS {
    let mut EndTime: LARGE_INTEGER = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut LegacyCapability: EHCI_LEGACY_EXTENDED_CAPABILITY = core::mem::zeroed();
    let OffsetEECP: UCHAR;

    OffsetEECP = EHCI_GetOffsetEECP(EhciExtension, 1 as UCHAR);

    if OffsetEECP as i32 == 0 {
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    (RegPacket.UsbPortReadWriteConfigSpace).expect("non-null function pointer")(
        EhciExtension as PVOID,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(LegacyCapability.AsULONG) as PVOID,
        OffsetEECP as ULONG,
        core::mem::size_of::<EHCI_LEGACY_EXTENDED_CAPABILITY>() as ULONG,
    );

    if LegacyCapability.c2rust_unnamed.BiosOwnedSemaphore() as i32 == 0 {
        return MP_STATUS_SUCCESS as MPSTATUS;
    }

    LegacyCapability.c2rust_unnamed.set_OsOwnedSemaphore(1 as ULONG);

    (RegPacket.UsbPortReadWriteConfigSpace).expect("non-null function pointer")(
        EhciExtension as PVOID,
        FALSE as BOOLEAN,
        core::ptr::addr_of_mut!(LegacyCapability.AsULONG) as PVOID,
        OffsetEECP as ULONG,
        core::mem::size_of::<EHCI_LEGACY_EXTENDED_CAPABILITY>() as ULONG,
    );

    *(core::ptr::addr_of_mut!(EndTime) as *mut ULONG64) = *(SharedSystemTimePtr());
    EndTime.QuadPart += (100 * 10000) as i64;

    loop {
        (RegPacket.UsbPortReadWriteConfigSpace).expect("non-null function pointer")(
            EhciExtension as PVOID,
            TRUE as BOOLEAN,
            core::ptr::addr_of_mut!(LegacyCapability.AsULONG) as PVOID,
            OffsetEECP as ULONG,
            core::mem::size_of::<EHCI_LEGACY_EXTENDED_CAPABILITY>() as ULONG,
        );
        *(core::ptr::addr_of_mut!(CurrentTime) as *mut ULONG64) = *(SharedSystemTimePtr());

        if LegacyCapability.c2rust_unnamed.BiosOwnedSemaphore() != 0 {
            break;
        }

        if !(CurrentTime.QuadPart <= EndTime.QuadPart) {
            break;
        }
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}
#[no_mangle]
pub unsafe extern "system" fn EHCI_GetRegistryParameters(mut EhciExtension: PEHCI_EXTENSION) {
    if DbgPrint(
        b"(%s:%d) EHCI_GetRegistryParameters: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1155 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1155 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_StartController(
    ehciExtension: PVOID,
    Resources: PUSBPORT_RESOURCES,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let mut CapabilityRegisters: PEHCI_HC_CAPABILITY_REGISTERS = core::ptr::null_mut();
    let mut OperationalRegs: PEHCI_HW_REGISTERS = core::ptr::null_mut();
    let mut MPStatus: MPSTATUS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();
    let mut CapabilityRegLength: UCHAR;
    let mut Fladj: UCHAR = 0;

    if ((*Resources).ResourcesTypes & (USBPORT_RESOURCES_MEMORY | USBPORT_RESOURCES_INTERRUPT))
        != (USBPORT_RESOURCES_MEMORY | USBPORT_RESOURCES_INTERRUPT)
    {
        return MP_STATUS_ERROR as MPSTATUS;
    }

    CapabilityRegisters = (*Resources).ResourceBase as PEHCI_HC_CAPABILITY_REGISTERS;
    (*EhciExtension).CapabilityRegisters = CapabilityRegisters;

    CapabilityRegLength =
        READ_REGISTER_UCHAR(core::ptr::addr_of_mut!((*CapabilityRegisters).RegistersLength) as *mut UCHAR);

    OperationalRegs = ((CapabilityRegisters as ULONG_PTR)
        .wrapping_add(CapabilityRegLength as ULONG_PTR)) as PEHCI_HW_REGISTERS;

    (*EhciExtension).OperationalRegs = OperationalRegs;

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        EhciExtension as PVOID,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(Fladj) as PVOID,
        EHCI_FLADJ_PCI_CONFIG_OFFSET as ULONG,
        core::mem::size_of::<UCHAR>() as ULONG,
    );

    (*EhciExtension).FrameLengthAdjustment = Fladj;

    EHCI_GetRegistryParameters(EhciExtension);

    MPStatus = EHCI_TakeControlHC(EhciExtension);

    if MPStatus != 0 {
        return MPStatus;
    }

    MPStatus = EHCI_InitializeHardware(EhciExtension);

    if MPStatus != 0 {
        return MPStatus;
    }

    MPStatus = EHCI_InitializeSchedule(EhciExtension, (*Resources).StartVA as ULONG_PTR, (*Resources).StartPA);

    if MPStatus != 0 {
        return MPStatus;
    }

    (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
        EhciExtension as PVOID,
        TRUE as BOOLEAN,
        core::ptr::addr_of_mut!(Fladj) as PVOID,
        EHCI_FLADJ_PCI_CONFIG_OFFSET as ULONG,
        core::mem::size_of::<UCHAR>() as ULONG,
    );

    if Fladj != (*EhciExtension).FrameLengthAdjustment {
        Fladj = (*EhciExtension).FrameLengthAdjustment;

        (RegPacket.UsbPortReadWriteConfigSpace.expect("non-null function pointer"))(
            EhciExtension as PVOID,
            FALSE as BOOLEAN,
            core::ptr::addr_of_mut!(Fladj) as PVOID,
            EHCI_FLADJ_PCI_CONFIG_OFFSET as ULONG,
            core::mem::size_of::<UCHAR>() as ULONG,
        );
    }

    (*EhciExtension).PortRoutingControl = EHCI_CONFIG_FLAG_CONFIGURED as ULONG;
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).ConfigFlag) as *mut ULONG,
        (*EhciExtension).PortRoutingControl,
    );

    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_InterruptThreshold(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_Run(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    (*EhciExtension).IsStarted = TRUE as BOOLEAN;

    if (*Resources).IsChirpHandled != 0 {
        let mut Port: ULONG;

        Port = 1;
        while Port <= (*EhciExtension).NumberOfPorts as ULONG {
            EHCI_RH_SetFeaturePortPower(EhciExtension as PVOID, Port as USHORT);
            Port = Port.wrapping_add(1);
        }

        (RegPacket.UsbPortWait.expect("non-null function pointer"))(EhciExtension as PVOID, 200 as ULONG);

        Port = 1;
        while Port <= (*EhciExtension).NumberOfPorts as ULONG {
            let fresh0 = Port;
            Port = Port.wrapping_add(1);
            EHCI_RH_ChirpRootPort(EhciExtension as PVOID, fresh0 as USHORT);
            Port = Port.wrapping_add(1);
        }
    }

    return MPStatus;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_StopController(
    ehciExtension: PVOID,
    DisableInterrupts: BOOLEAN,
) {
    if DbgPrint(
        b"(%s:%d) EHCI_StopController: UNIMPLEMENTED. FIXME\n\0".as_ptr() as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char,
        1288 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char,
            1288 as core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SuspendController(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();
    let mut Status: EHCI_USB_STATUS = core::mem::zeroed();
    let mut IntrEn: EHCI_INTERRUPT_ENABLE = core::mem::zeroed();
    let mut ix: ULONG;

    OperationalRegs = (*EhciExtension).OperationalRegs;

    (*EhciExtension).BackupPeriodiclistbase =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).PeriodicListBase) as *mut ULONG);
    (*EhciExtension).BackupAsynclistaddr =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).AsyncListBase) as *mut ULONG);
    (*EhciExtension).BackupCtrlDSSegment =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).SegmentSelector) as *mut ULONG);
    (*EhciExtension).BackupUSBCmd =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);

    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_InterruptAdvanceDoorbell(0);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_Run(0);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    KeStallExecutionProcessor(125);

    Status.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG) as *mut ULONG);

    Status.set_HCHalted(0);
    Status.set_Reclamation(0);
    Status.set_PeriodicStatus(0);
    Status.set_AsynchronousStatus(0);

    if Status.AsULONG != 0 {
        WRITE_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG) as *mut ULONG,
            Status.AsULONG,
        );
    }

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG,
        0,
    );

    ix = 0;
    while ix < 10 {
        Status.AsULONG =
            READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG) as *mut ULONG);

        if Status.HCHalted() != 0 {
            break;
        }

        (RegPacket.UsbPortWait.expect("non-null function pointer"))(EhciExtension as PVOID, 1);

        ix = ix.wrapping_add(1);
    }

    if Status.HCHalted() == 0 {
        DbgBreakPoint();
    }

    IntrEn.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG);
    IntrEn.set_PortChangeInterrupt(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG,
        IntrEn.AsULONG,
    );

    (*EhciExtension).Flags |= EHCI_FLAGS_CONTROLLER_SUSPEND as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_ResumeController(ehciExtension: PVOID) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let RoutingControl: ULONG;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();

    OperationalRegs = (*EhciExtension).OperationalRegs;

    RoutingControl = (*EhciExtension).PortRoutingControl;

    if RoutingControl & EHCI_CONFIG_FLAG_CONFIGURED == 0 {
        (*EhciExtension).PortRoutingControl = RoutingControl | EHCI_CONFIG_FLAG_CONFIGURED;
        WRITE_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*OperationalRegs).ConfigFlag) as *mut ULONG,
            (*EhciExtension).PortRoutingControl,
        );

        return MP_STATUS_HW_ERROR as MPSTATUS;
    }

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).SegmentSelector) as *mut ULONG,
        (*EhciExtension).BackupCtrlDSSegment,
    );

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).PeriodicListBase) as *mut ULONG,
        (*EhciExtension).BackupPeriodiclistbase,
    );

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).AsyncListBase) as *mut ULONG,
        (*EhciExtension).BackupAsynclistaddr,
    );

    Command.AsULONG = READ_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
    );

    Command.AsULONG = Command.AsULONG ^ (*EhciExtension).BackupUSBCmd;

    Command.set_Reset(0);
    Command.set_FrameListSize(0);
    Command.set_InterruptAdvanceDoorbell(0);
    Command.set_LightResetHC(0);
    Command.set_AsynchronousParkModeCount(0);
    Command.set_AsynchronousParkModeEnable(0);

    Command.set_Run(1);

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG,
        (*EhciExtension).InterruptMask.AsULONG,
    );

    (*EhciExtension).Flags &= !EHCI_FLAGS_CONTROLLER_SUSPEND;

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_HardwarePresent(
    EhciExtension: PEHCI_EXTENSION,
    IsInvalidateController: BOOLEAN,
) -> BOOLEAN {
    const USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE: u32 = 2;
    let OperationalRegs: PEHCI_HW_REGISTERS = (*EhciExtension).OperationalRegs;
    if READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG)
        != -(1 as core::ffi::c_int) as core::ffi::c_ulong
    {
        return TRUE as BOOLEAN;
    }
    if DbgPrint(
        b"(%s:%d) EHCI_HardwarePresent: IsInvalidateController - %x\n\0" as *const u8 as PCSTR,
        b"usbehci.c\0" as *const u8 as *const core::ffi::c_char,
        1421 as core::ffi::c_int,
        IsInvalidateController as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"usbehci.c\0" as *const u8 as *const core::ffi::c_char,
            1421 as core::ffi::c_int,
        );
    }
    if IsInvalidateController == 0 {
        return FALSE as BOOLEAN;
    }
    RegPacket.UsbPortInvalidateController.expect("non-null function pointer")(
        EhciExtension as PVOID,
        USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE as ULONG,
    );
    return FALSE as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InterruptService(ehciExtension: PVOID) -> BOOLEAN {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Result: BOOLEAN;
    let mut IntrSts: EHCI_USB_STATUS = core::mem::zeroed();
    let mut IntrEn: EHCI_INTERRUPT_ENABLE = core::mem::zeroed();
    let mut iStatus: EHCI_INTERRUPT_ENABLE = core::mem::zeroed();
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();
    let mut FrameIndex: ULONG;

    OperationalRegs = (*EhciExtension).OperationalRegs;

    DPRINT_EHCI!(b"EHCI_InterruptService: ... \n\0".as_ptr() as *const i8);

    Result = EHCI_HardwarePresent(EhciExtension, 0 as BOOLEAN);

    if Result == 0 {
        return 0 as BOOLEAN;
    }

    IntrEn.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG));
    IntrSts.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG));

    iStatus.AsULONG = (IntrEn.AsULONG & IntrSts.AsULONG) & EHCI_INTERRUPT_MASK;

    if iStatus.AsULONG == 0 {
        return 0 as BOOLEAN;
    }

    (*EhciExtension).InterruptStatus.AsULONG |= iStatus.AsULONG;

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG),
        iStatus.AsULONG,
    );

    if iStatus.HostSystemError() != 0 {
        (*EhciExtension).HcSystemErrors = (*EhciExtension).HcSystemErrors.wrapping_add(1);

        if ((*EhciExtension).HcSystemErrors as i32) < EHCI_MAX_HC_SYSTEM_ERRORS as i32 {
            Command.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
            Command.set_Run(1 as ULONG);
            WRITE_REGISTER_ULONG(
                core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG),
                Command.AsULONG,
            );
        }
    }

    FrameIndex = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).FrameIndex))
        / EHCI_MICROFRAMES;
    FrameIndex &= EHCI_FRINDEX_FRAME_MASK;

    if (FrameIndex ^ (*EhciExtension).FrameIndex) & EHCI_FRAME_LIST_MAX_ENTRIES != 0 {
        (*EhciExtension).FrameHighPart = (*EhciExtension)
            .FrameHighPart
            .wrapping_add(2 * EHCI_FRAME_LIST_MAX_ENTRIES);

        (*EhciExtension).FrameHighPart = (*EhciExtension).FrameHighPart.wrapping_sub(
            (FrameIndex ^ (*EhciExtension).FrameHighPart) & EHCI_FRAME_LIST_MAX_ENTRIES,
        );
    }

    (*EhciExtension).FrameIndex = FrameIndex;

    if iStatus.Interrupt() != 0
        || iStatus.ErrorInterrupt() != 0
        || iStatus.PortChangeInterrupt() != 0
        || iStatus.InterruptOnAsyncAdvance() != 0
    {
        return 1 as BOOLEAN;
    }

    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InterruptDpc(
    ehciExtension: PVOID,
    EnableInterrupts: BOOLEAN,
) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let iStatus: EHCI_INTERRUPT_ENABLE;

    OperationalRegs = (*EhciExtension).OperationalRegs;

    DPRINT_EHCI!(
        b"EHCI_InterruptDpc: [%p] EnableInterrupts - %x\n\0".as_ptr() as *const i8,
        EhciExtension,
        EnableInterrupts as core::ffi::c_int,
    );

    iStatus = (*EhciExtension).InterruptStatus;
    (*EhciExtension).InterruptStatus.AsULONG = 0;

    if iStatus.Interrupt() == 1
        || iStatus.ErrorInterrupt() == 1
        || iStatus.InterruptOnAsyncAdvance() == 1
    {
        DPRINT_EHCI!(
            b"EHCI_InterruptDpc: [%p] InterruptStatus - %X\n\0".as_ptr() as *const i8,
            EhciExtension,
            iStatus.AsULONG,
        );
        (RegPacket.UsbPortInvalidateEndpoint).expect("non-null function pointer")(
            EhciExtension as PVOID,
            core::ptr::null_mut(),
        );
    }

    if iStatus.PortChangeInterrupt() == 1 {
        DPRINT_EHCI!(
            b"EHCI_InterruptDpc: [%p] PortChangeInterrupt\n\0".as_ptr() as *const i8,
            EhciExtension,
        );
        (RegPacket.UsbPortInvalidateRootHub).expect("non-null function pointer")(
            EhciExtension as PVOID,
        );
    }

    if EnableInterrupts != 0 {
        WRITE_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG,
            (*EhciExtension).InterruptMask.AsULONG,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_MapAsyncTransferToTd(
    EhciExtension: PEHCI_EXTENSION,
    MaxPacketSize: ULONG,
    TransferedLen: ULONG,
    DataToggle: PULONG,
    EhciTransfer: PEHCI_TRANSFER,
    TD: PEHCI_HCD_TD,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> ULONG {
    let mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS;
    let mut SgElement: PUSBPORT_SCATTER_GATHER_ELEMENT;
    let mut SgIdx: ULONG;
    let mut LengthThisTD: ULONG;
    let mut ix: ULONG;
    let SgRemain: ULONG;
    let DiffLength: ULONG;
    let NumPackets: ULONG;

    DPRINT_EHCI!(
        b"EHCI_MapAsyncTransferToTd: EhciTransfer - %p, TD - %p, TransferedLen - %x, MaxPacketSize - %x, DataToggle - %x\n\0".as_ptr() as *const i8,
        EhciTransfer,
        TD,
        TransferedLen,
        MaxPacketSize,
        DataToggle,
    );

    TransferParameters = (*EhciTransfer).TransferParameters;

    SgElement = core::ptr::addr_of_mut!((*SgList).SgElement[0]);

    SgIdx = 0;
    while SgIdx < (*SgList).SgElementCount {
        if TransferedLen >= (*SgElement).SgOffset
            && TransferedLen < (*SgElement).SgOffset + (*SgElement).SgTransferLength
        {
            break;
        }

        SgElement = SgElement.offset(1);
        SgIdx = SgIdx.wrapping_add(1);
    }

    SgRemain = (*SgList).SgElementCount - SgIdx;

    if SgRemain > EHCI_MAX_QTD_BUFFER_PAGES {
        (*TD).HwTD.Buffer[0] = (*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add(SgIdx as usize))
            .SgPhysicalAddress
            .u.LowPart
            .wrapping_sub((*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add(SgIdx as usize)).SgOffset)
            .wrapping_add(TransferedLen);

        LengthThisTD = (EHCI_MAX_QTD_BUFFER_PAGES * PAGE_SIZE as ULONG)
            .wrapping_sub((*TD).HwTD.Buffer[0] & (PAGE_SIZE as ULONG - 1));

        ix = 1;
        while ix < EHCI_MAX_QTD_BUFFER_PAGES {
            (*TD).HwTD.Buffer[ix as usize] =
                (*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add((SgIdx + ix) as usize)).SgPhysicalAddress.__bindgen_anon_1.LowPart;
            ix = ix.wrapping_add(1);
        }

        NumPackets = LengthThisTD / MaxPacketSize;
        DiffLength = LengthThisTD - MaxPacketSize * (LengthThisTD / MaxPacketSize);

        if LengthThisTD != MaxPacketSize * (LengthThisTD / MaxPacketSize) {
            LengthThisTD -= DiffLength;
        }

        if !DataToggle.is_null() && (NumPackets & 1) != 0 {
            *DataToggle = (*DataToggle == 0) as ULONG;
        }
    } else {
        LengthThisTD = (*TransferParameters).TransferBufferLength - TransferedLen;

        (*TD).HwTD.Buffer[0] = TransferedLen
            .wrapping_add((*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add(SgIdx as usize)).SgPhysicalAddress.__bindgen_anon_1.LowPart)
            .wrapping_sub((*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add(SgIdx as usize)).SgOffset);

        ix = 1;
        while ix < EHCI_MAX_QTD_BUFFER_PAGES {
            if (SgIdx + ix) >= (*SgList).SgElementCount {
                break;
            }

            (*TD).HwTD.Buffer[ix as usize] =
                (*(core::ptr::addr_of!((*SgList).SgElement) as *const USBPORT_SCATTER_GATHER_ELEMENT).add((SgIdx + ix) as usize)).SgPhysicalAddress.__bindgen_anon_1.LowPart;
            ix = ix.wrapping_add(1);
        }
    }

    (*TD).HwTD.Token.set_TransferBytes(LengthThisTD);
    (*TD).LengthThisTD = LengthThisTD;

    return LengthThisTD + TransferedLen;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_EnableAsyncList(EhciExtension: PEHCI_EXTENSION) {
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut UsbCmd: EHCI_USB_COMMAND = core::mem::zeroed();

    DPRINT_EHCI!(b"EHCI_EnableAsyncList: ... \n\0".as_ptr() as *const i8);

    OperationalRegs = (*EhciExtension).OperationalRegs;

    UsbCmd.AsULONG = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
    UsbCmd.set_AsynchronousEnable(1);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG), UsbCmd.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_DisableAsyncList(EhciExtension: PEHCI_EXTENSION) {
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut UsbCmd: EHCI_USB_COMMAND = core::mem::zeroed();

    OperationalRegs = (*EhciExtension).OperationalRegs;

    UsbCmd.AsULONG = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
    UsbCmd.set_AsynchronousEnable(0);
    WRITE_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG), UsbCmd.AsULONG);
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_EnablePeriodicList(EhciExtension: PEHCI_EXTENSION) {
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();

    OperationalRegs = (*EhciExtension).OperationalRegs;

    Command.AsULONG = READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG);
    Command.set_PeriodicEnable(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
        Command.AsULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_FlushAsyncCache(EhciExtension: PEHCI_EXTENSION) {
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();
    let mut Status: EHCI_USB_STATUS = core::mem::zeroed();
    let mut CurrentTime: LARGE_INTEGER = core::mem::zeroed();
    let mut EndTime: LARGE_INTEGER = core::mem::zeroed();
    let mut Cmd: EHCI_USB_COMMAND = core::mem::zeroed();

    DPRINT_EHCI!(
        b"EHCI_FlushAsyncCache: EhciExtension - %p\n\0".as_ptr() as *const i8,
        EhciExtension,
    );

    OperationalRegs = (*EhciExtension).OperationalRegs;
    Command.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
    Status.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG));

    if Status.AsynchronousStatus() == 0 && Command.AsynchronousEnable() == 0 {
        return;
    }

    if Status.AsynchronousStatus() != 0 && Command.AsynchronousEnable() == 0 {
        KeQuerySystemTime(core::ptr::addr_of_mut!(EndTime));
        EndTime.QuadPart += (100 * 10000) as i64; //100 ms

        loop {
            Status.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG));
            Command.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
            KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

            if CurrentTime.QuadPart > EndTime.QuadPart {
                (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(
                    EhciExtension as PVOID,
                );
            }

            if !(Status.AsynchronousStatus() != 0
                && Command.AsULONG != (-1i32) as ULONG
                && Command.Run() != 0)
            {
                break;
            }
        }

        return;
    }

    if Status.AsynchronousStatus() == 0 && Command.AsynchronousEnable() != 0 {
        KeQuerySystemTime(core::ptr::addr_of_mut!(EndTime));
        EndTime.QuadPart += (100 * 10000) as i64; //100 ms

        loop {
            Status.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG));
            Command.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
            KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

            if !(Status.AsynchronousStatus() == 0
                && Command.AsULONG != (-1i32) as ULONG
                && Command.Run() != 0)
            {
                break;
            }
        }
    }

    Command.set_InterruptAdvanceDoorbell(1);
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG),
        Command.AsULONG,
    );

    KeQuerySystemTime(core::ptr::addr_of_mut!(EndTime));
    EndTime.QuadPart += (100 * 10000) as i64; //100 ms

    Cmd.AsULONG =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));

    if Cmd.InterruptAdvanceDoorbell() != 0 {
        while Cmd.Run() != 0 {
            if Cmd.AsULONG == (-1i32) as ULONG {
                break;
            }

            KeStallExecutionProcessor(1);
            Command.AsULONG =
                READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG));
            KeQuerySystemTime(core::ptr::addr_of_mut!(CurrentTime));

            if Command.InterruptAdvanceDoorbell() == 0 {
                break;
            }

            Cmd = Command;
        }
    }

    /* InterruptOnAsyncAdvance */
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG),
        0x20,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_LockQH(
    EhciExtension: PEHCI_EXTENSION,
    QH: PEHCI_HCD_QH,
    TransferType: ULONG,
) {
    let mut PrevQH: PEHCI_HCD_QH;
    let mut NextQH: PEHCI_HCD_QH;
    let mut QhPA: ULONG;
    let FrameIndexReg: ULONG;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();

    DPRINT_EHCI!(
        b"EHCI_LockQH: QH - %p, TransferType - %x\n\0".as_ptr() as *const i8,
        QH,
        TransferType,
    );

    OperationalRegs = (*EhciExtension).OperationalRegs;

    // ASSERT((QH->sqh.QhFlags & EHCI_QH_FLAG_UPDATING) == 0);
    // ASSERT(EhciExtension->LockQH == NULL);

    PrevQH = (*QH).sqh.PrevHead;
    (*QH).sqh.QhFlags |= EHCI_QH_FLAG_UPDATING as ULONG;

    // ASSERT(PrevQH);

    NextQH = (*QH).sqh.NextHead;

    (*EhciExtension).PrevQH = PrevQH;
    (*EhciExtension).NextQH = NextQH;
    (*EhciExtension).LockQH = QH;

    if !NextQH.is_null() {
        QhPA = (*NextQH).sqh.PhysicalAddress;
        QhPA &= (LINK_POINTER_MASK + TERMINATE_POINTER) as ULONG;
        QhPA |= (EHCI_LINK_TYPE_QH << 1) as ULONG;
    } else {
        QhPA = TERMINATE_POINTER as ULONG;
    }

    (*PrevQH).sqh.HwQH.HorizontalLink.AsULONG = QhPA;

    FrameIndexReg =
        READ_REGISTER_ULONG(core::ptr::addr_of_mut!((*OperationalRegs).FrameIndex) as *mut ULONG);

    if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
        loop {
            Command.AsULONG = READ_REGISTER_ULONG(
                core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
            );
            if !(READ_REGISTER_ULONG(
                core::ptr::addr_of_mut!((*OperationalRegs).FrameIndex) as *mut ULONG,
            ) == FrameIndexReg
                && Command.AsULONG != -(1i32) as ULONG
                && Command.Run() != 0)
            {
                break;
            }
        }
    } else {
        EHCI_FlushAsyncCache(EhciExtension);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_UnlockQH(
    EhciExtension: PEHCI_EXTENSION,
    QH: PEHCI_HCD_QH,
) {

    let mut QhPA: ULONG;

    DPRINT_EHCI!(b"EHCI_UnlockQH: QH - %p\n\0".as_ptr() as *const i8, QH);

    if (*QH).sqh.QhFlags as core::ffi::c_ulong & EHCI_QH_FLAG_UPDATING as core::ffi::c_ulong == 0 {
        RtlAssert(
            b"QH->sqh.QhFlags & EHCI_QH_FLAG_UPDATING\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            1823 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    if (*EhciExtension).LockQH.is_null() {
        RtlAssert(
            b"EhciExtension->LockQH\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            1824 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    if !((*EhciExtension).LockQH == QH) {
        RtlAssert(
            b"EhciExtension->LockQH == QH\0".as_ptr() as *const core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char as PVOID,
            1825 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    (*QH).sqh.QhFlags &= !EHCI_QH_FLAG_UPDATING as core::ffi::c_ulong;

    (*EhciExtension).LockQH = core::ptr::null_mut();

    QhPA = (*QH).sqh.PhysicalAddress;
    QhPA &= LINK_POINTER_MASK.wrapping_add(TERMINATE_POINTER as core::ffi::c_uint) as core::ffi::c_ulong;
    QhPA |= (EHCI_LINK_TYPE_QH << 1) as core::ffi::c_ulong;

    (*(*EhciExtension).PrevQH).sqh.HwQH.HorizontalLink.AsULONG = QhPA;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_LinkTransferToQueue(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    NextTD: PEHCI_HCD_TD,
) {
    let mut QH: PEHCI_HCD_QH;
    let mut TD: PEHCI_HCD_TD;
    let Transfer: PEHCI_TRANSFER;
    let mut LinkTD: PEHCI_HCD_TD;
    let IsPresent: BOOLEAN;
    let mut ix: ULONG;

    DPRINT_EHCI!(
        b"EHCI_LinkTransferToQueue: EhciEndpoint - %p, NextTD - %p\n\0".as_ptr() as *const i8,
        EhciEndpoint,
        NextTD,
    );

    if (*EhciEndpoint).HcdHeadP.is_null() {
        RtlAssert(
            b"EhciEndpoint->HcdHeadP != NULL\0".as_ptr() as *const i8 as PVOID,
            b"usbehci.c\0".as_ptr() as *const i8 as PVOID,
            1855 as ULONG,
            core::ptr::null_mut(),
        );
    }

    IsPresent = EHCI_HardwarePresent(EhciExtension, 0 as BOOLEAN);

    QH = (*EhciEndpoint).QH;
    TD = (*EhciEndpoint).HcdHeadP;

    if TD == (*EhciEndpoint).HcdTailP {
        if IsPresent != 0 {
            EHCI_LockQH(
                EhciExtension,
                QH,
                (*EhciEndpoint).EndpointProperties.TransferType,
            );
        }

        (*QH).sqh.HwQH.CurrentTD = (*EhciEndpoint).DmaBufferPA;
        (*QH).sqh.HwQH.NextTD = (*NextTD).PhysicalAddress;
        (*QH).sqh.HwQH.AlternateNextTD = (*NextTD).HwTD.AlternateNextTD;

        (*QH).sqh.HwQH.Token.set_Status(
            !(EHCI_TOKEN_STATUS_ACTIVE | EHCI_TOKEN_STATUS_HALTED) as UCHAR as ULONG,
        );

        (*QH).sqh.HwQH.Token.set_TransferBytes(0 as ULONG);

        if IsPresent != 0 {
            EHCI_UnlockQH(EhciExtension, QH);
        }

        (*EhciEndpoint).HcdHeadP = NextTD;
    } else {
        DPRINT_EHCI!(
            b"EHCI_LinkTransferToQueue: TD - %p, HcdTailP - %p\n\0".as_ptr() as *const i8,
            (*EhciEndpoint).HcdHeadP,
            (*EhciEndpoint).HcdTailP,
        );

        LinkTD = (*EhciEndpoint).HcdHeadP;

        while TD != (*EhciEndpoint).HcdTailP {
            LinkTD = TD;
            TD = (*TD).NextHcdTD as PEHCI_HCD_TD;
        }

        if !(LinkTD != (*EhciEndpoint).HcdTailP) {
            RtlAssert(
                b"LinkTD != EhciEndpoint->HcdTailP\0".as_ptr() as *const i8 as PVOID,
                b"usbehci.c\0".as_ptr() as *const i8 as PVOID,
                1898 as ULONG,
                core::ptr::null_mut(),
            );
        }

        Transfer = (*LinkTD).EhciTransfer as PEHCI_TRANSFER;

        TD = (*EhciEndpoint).FirstTD;

        ix = 0 as ULONG;
        while ix < (*EhciEndpoint).MaxTDs {
            if (*TD).EhciTransfer == Transfer {
                (*TD).AltNextHcdTD = NextTD;
                (*TD).HwTD.AlternateNextTD = (*NextTD).PhysicalAddress;
            }

            TD = TD.offset(1);
            ix = ix.wrapping_add(1);
        }

        (*LinkTD).HwTD.NextTD = (*NextTD).PhysicalAddress;
        (*LinkTD).NextHcdTD = NextTD;

        if (*QH).sqh.HwQH.CurrentTD == (*LinkTD).PhysicalAddress {
            (*QH).sqh.HwQH.NextTD = (*NextTD).PhysicalAddress;
            (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_ControlTransfer(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    EhciTransfer: PEHCI_TRANSFER,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut FirstTD: PEHCI_HCD_TD;
    let mut LastTD: PEHCI_HCD_TD;
    let mut TD: PEHCI_HCD_TD;
    let mut PrevTD: PEHCI_HCD_TD;
    let mut LinkTD: PEHCI_HCD_TD;
    let mut TransferedLen: ULONG = 0;
    let mut Token: EHCI_TD_TOKEN = core::mem::zeroed();
    let mut DataToggle: ULONG = 1;

    DPRINT_EHCI!(
        b"EHCI_ControlTransfer: EhciEndpoint - %p, EhciTransfer - %p\n\0".as_ptr() as *const i8,
        EhciEndpoint,
        EhciTransfer,
    );

    if (*EhciEndpoint).RemainTDs < EHCI_MAX_CONTROL_TD_COUNT as ULONG {
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*EhciExtension).PendingTransfers = (*EhciExtension).PendingTransfers.wrapping_add(1);
    (*EhciEndpoint).PendingTDs = (*EhciEndpoint).PendingTDs.wrapping_add(1);

    (*EhciTransfer).TransferOnAsyncList = 1;

    FirstTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

    if FirstTD.is_null() {
        (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(EhciExtension as PVOID);
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*EhciTransfer).PendingTDs = (*EhciTransfer).PendingTDs.wrapping_add(1);

    (*FirstTD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
    (*FirstTD).EhciTransfer = EhciTransfer;

    (*FirstTD).HwTD.Buffer[0] = ((*FirstTD).PhysicalAddress as ULONG)
        .wrapping_add(core::mem::offset_of!(EHCI_HCD_TD, SetupPacket) as ULONG);
    (*FirstTD).HwTD.Buffer[1] = 0;
    (*FirstTD).HwTD.Buffer[2] = 0;
    (*FirstTD).HwTD.Buffer[3] = 0;
    (*FirstTD).HwTD.Buffer[4] = 0;

    (*FirstTD).NextHcdTD = core::ptr::null_mut();

    (*FirstTD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
    (*FirstTD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*FirstTD).HwTD.Token.AsULONG = 0;
    (*FirstTD).HwTD.Token.set_ErrorCounter(3);
    (*FirstTD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_SETUP as ULONG);
    (*FirstTD).HwTD.Token.set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);
    (*FirstTD)
        .HwTD
        .Token
        .set_TransferBytes(core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>() as ULONG);

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*FirstTD).SetupPacket) as PVOID,
        core::ptr::addr_of!((*TransferParameters).SetupPacket) as PVOID,
        core::mem::size_of::<USB_DEFAULT_PIPE_SETUP_PACKET>(),
    );

    LastTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

    if LastTD.is_null() {
        (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(EhciExtension as PVOID);
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*EhciTransfer).PendingTDs = (*EhciTransfer).PendingTDs.wrapping_add(1);

    (*LastTD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
    (*LastTD).EhciTransfer = EhciTransfer;

    (*LastTD).HwTD.Buffer[0] = 0;
    (*LastTD).HwTD.Buffer[1] = 0;
    (*LastTD).HwTD.Buffer[2] = 0;
    (*LastTD).HwTD.Buffer[3] = 0;
    (*LastTD).HwTD.Buffer[4] = 0;

    (*LastTD).NextHcdTD = core::ptr::null_mut();
    (*LastTD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
    (*LastTD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

    (*LastTD).HwTD.Token.AsULONG = 0;
    (*LastTD).HwTD.Token.set_ErrorCounter(3);

    (*FirstTD).AltNextHcdTD = LastTD;
    (*FirstTD).HwTD.AlternateNextTD = (*LastTD).PhysicalAddress;

    PrevTD = FirstTD;
    LinkTD = FirstTD;

    while TransferedLen < (*TransferParameters).TransferBufferLength {
        TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

        if TD.is_null() {
            (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(EhciExtension as PVOID);
            return MP_STATUS_FAILURE as MPSTATUS;
        }

        LinkTD = TD;

        (*EhciTransfer).PendingTDs = (*EhciTransfer).PendingTDs.wrapping_add(1);

        (*TD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
        (*TD).EhciTransfer = EhciTransfer;

        (*TD).HwTD.Buffer[0] = 0;
        (*TD).HwTD.Buffer[1] = 0;
        (*TD).HwTD.Buffer[2] = 0;
        (*TD).HwTD.Buffer[3] = 0;
        (*TD).HwTD.Buffer[4] = 0;

        (*TD).NextHcdTD = core::ptr::null_mut();

        (*TD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
        (*TD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

        (*TD).HwTD.Token.AsULONG = 0;
        (*TD).HwTD.Token.set_ErrorCounter(3);

        (*PrevTD).NextHcdTD = TD;
        (*PrevTD).HwTD.NextTD = (*TD).PhysicalAddress;

        if (*TransferParameters).TransferFlags as ULONG & USBD_TRANSFER_DIRECTION_IN as ULONG != 0 {
            (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_IN as ULONG);
        } else {
            (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_OUT as ULONG);
        }

        (*TD).HwTD.Token.set_DataToggle(DataToggle);
        (*TD).HwTD.Token.set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);

        if DataToggle != 0 {
            (*TD).HwTD.Token.set_DataToggle(1);
        } else {
            (*TD).HwTD.Token.set_DataToggle(0);
        }

        (*TD).AltNextHcdTD = LastTD;
        (*TD).HwTD.AlternateNextTD = (*LastTD).PhysicalAddress;

        TransferedLen = EHCI_MapAsyncTransferToTd(
            EhciExtension,
            (*EhciEndpoint).EndpointProperties.MaxPacketSize,
            TransferedLen,
            core::ptr::addr_of_mut!(DataToggle),
            EhciTransfer,
            TD,
            SgList,
        );

        PrevTD = TD;
    }

    (*LinkTD).NextHcdTD = LastTD;
    (*LinkTD).HwTD.NextTD = (*LastTD).PhysicalAddress;

    (*LastTD).HwTD.Buffer[0] = 0;
    (*LastTD).LengthThisTD = 0;

    Token.AsULONG = 0;
    Token.set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);
    Token.set_InterruptOnComplete(1);
    Token.set_DataToggle(1);

    if (*TransferParameters).TransferFlags as ULONG & USBD_TRANSFER_DIRECTION_IN as ULONG != 0 {
        Token.set_PIDCode(EHCI_TD_TOKEN_PID_OUT as ULONG);
    } else {
        Token.set_PIDCode(EHCI_TD_TOKEN_PID_IN as ULONG);
    }

    (*LastTD).HwTD.Token = Token;

    (*LastTD).NextHcdTD = (*EhciEndpoint).HcdTailP;
    (*LastTD).HwTD.NextTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;

    EHCI_EnableAsyncList(EhciExtension);
    EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);

    if !(*(*EhciEndpoint).HcdTailP).NextHcdTD.is_null() {
        RtlAssert(
            b"EhciEndpoint->HcdTailP->NextHcdTD == NULL\0".as_ptr() as PVOID,
            b"usbehci.c\0".as_ptr() as PVOID,
            2106,
            core::ptr::null_mut(),
        );
    }
    if !(*(*EhciEndpoint).HcdTailP).AltNextHcdTD.is_null() {
        RtlAssert(
            b"EhciEndpoint->HcdTailP->AltNextHcdTD == NULL\0".as_ptr() as PVOID,
            b"usbehci.c\0".as_ptr() as PVOID,
            2107,
            core::ptr::null_mut(),
        );
    }

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_BulkTransfer(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    EhciTransfer: PEHCI_TRANSFER,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut PrevTD: PEHCI_HCD_TD;
    let mut FirstTD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut TD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut TransferedLen: ULONG;

    DPRINT_EHCI!(
        b"EHCI_BulkTransfer: EhciEndpoint - %p, EhciTransfer - %p\n\0".as_ptr() as *const i8,
        EhciEndpoint,
        EhciTransfer
    );

    if ((*TransferParameters).TransferBufferLength
        / (((EHCI_MAX_QTD_BUFFER_PAGES - 1) * PAGE_SIZE as ULONG) as ULONG))
        + 1
        > (*EhciEndpoint).RemainTDs
    {
        // DPRINT removed
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*EhciExtension).PendingTransfers += 1;
    (*EhciEndpoint).PendingTDs += 1;

    (*EhciTransfer).TransferOnAsyncList = 1;

    TransferedLen = 0;
    PrevTD = core::ptr::null_mut();

    if (*TransferParameters).TransferBufferLength != 0 {
        while TransferedLen < (*TransferParameters).TransferBufferLength {
            TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

            if TD.is_null() {
                (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(
                    EhciExtension as PVOID,
                );
                return MP_STATUS_FAILURE as MPSTATUS;
            }

            (*EhciTransfer).PendingTDs += 1;

            (*TD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
            (*TD).EhciTransfer = EhciTransfer;

            (*TD).HwTD.Buffer[0] = 0;
            (*TD).HwTD.Buffer[1] = 0;
            (*TD).HwTD.Buffer[2] = 0;
            (*TD).HwTD.Buffer[3] = 0;
            (*TD).HwTD.Buffer[4] = 0;

            (*TD).NextHcdTD = core::ptr::null_mut();
            (*TD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
            (*TD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

            (*TD).HwTD.Token.AsULONG = 0;
            (*TD).HwTD.Token.set_ErrorCounter(3);

            if (*EhciTransfer).PendingTDs == 1 {
                FirstTD = TD;
            } else {
                (*PrevTD).HwTD.NextTD = (*TD).PhysicalAddress;
                (*PrevTD).NextHcdTD = TD;
            }

            (*TD).HwTD.AlternateNextTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;
            (*TD).AltNextHcdTD = (*EhciEndpoint).HcdTailP;

            (*TD).HwTD.Token.set_InterruptOnComplete(1);

            if (*TransferParameters).TransferFlags & USBD_TRANSFER_DIRECTION_IN != 0 {
                (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_IN as ULONG);
            } else {
                (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_OUT as ULONG);
            }

            (*TD)
                .HwTD
                .Token
                .set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);
            (*TD).HwTD.Token.set_DataToggle(1);

            TransferedLen = EHCI_MapAsyncTransferToTd(
                EhciExtension,
                (*EhciEndpoint).EndpointProperties.MaxPacketSize,
                TransferedLen,
                core::ptr::null_mut(),
                EhciTransfer,
                TD,
                SgList,
            );

            PrevTD = TD;
        }
    } else {
        TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

        if TD.is_null() {
            (RegPacket.UsbPortBugCheck.expect("non-null function pointer"))(
                EhciExtension as PVOID,
            );
            return MP_STATUS_FAILURE as MPSTATUS;
        }

        (*EhciTransfer).PendingTDs += 1;

        (*TD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
        (*TD).EhciTransfer = EhciTransfer;

        (*TD).HwTD.Buffer[0] = 0;
        (*TD).HwTD.Buffer[1] = 0;
        (*TD).HwTD.Buffer[2] = 0;
        (*TD).HwTD.Buffer[3] = 0;
        (*TD).HwTD.Buffer[4] = 0;

        (*TD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
        (*TD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

        (*TD).HwTD.Token.AsULONG = 0;
        (*TD).HwTD.Token.set_ErrorCounter(3);

        (*TD).NextHcdTD = core::ptr::null_mut();

        // ASSERT(EhciTransfer->PendingTDs == 1);

        FirstTD = TD;

        (*TD).HwTD.AlternateNextTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;
        (*TD).AltNextHcdTD = (*EhciEndpoint).HcdTailP;

        (*TD).HwTD.Token.set_InterruptOnComplete(1);

        if (*TransferParameters).TransferFlags & USBD_TRANSFER_DIRECTION_IN != 0 {
            (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_IN as ULONG);
        } else {
            (*TD).HwTD.Token.set_PIDCode(EHCI_TD_TOKEN_PID_OUT as ULONG);
        }

        (*TD).HwTD.Buffer[0] = (*TD).PhysicalAddress;

        (*TD)
            .HwTD
            .Token
            .set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);
        (*TD).HwTD.Token.set_DataToggle(1);

        (*TD).LengthThisTD = 0;
    }

    (*TD).HwTD.NextTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;
    (*TD).NextHcdTD = (*EhciEndpoint).HcdTailP;

    EHCI_EnableAsyncList(EhciExtension);
    EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);

    // ASSERT(EhciEndpoint->HcdTailP->NextHcdTD == 0);
    // ASSERT(EhciEndpoint->HcdTailP->AltNextHcdTD == 0);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InterruptTransfer(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    EhciTransfer: PEHCI_TRANSFER,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let mut TD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut FirstTD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut PrevTD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut TransferedLen: ULONG = 0;

    if (*EhciEndpoint).RemainTDs == 0 {
        DbgBreakPoint();
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    (*EhciEndpoint).PendingTDs = (*EhciEndpoint).PendingTDs.wrapping_add(1);

    if (*TransferParameters).TransferBufferLength == 0 {
        DbgBreakPoint();
        return MP_STATUS_FAILURE as MPSTATUS;
    }

    while TransferedLen < (*TransferParameters).TransferBufferLength {
        TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);

        if TD.is_null() {
            RegPacket.UsbPortBugCheck.expect("non-null function pointer")(EhciExtension as PVOID);
            return MP_STATUS_FAILURE as MPSTATUS;
        }

        (*EhciTransfer).PendingTDs = (*EhciTransfer).PendingTDs.wrapping_add(1);

        (*TD).TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED as ULONG;
        (*TD).EhciTransfer = EhciTransfer;

        (*TD).HwTD.Buffer[0] = 0;
        (*TD).HwTD.Buffer[1] = 0;
        (*TD).HwTD.Buffer[2] = 0;
        (*TD).HwTD.Buffer[3] = 0;
        (*TD).HwTD.Buffer[4] = 0;

        (*TD).HwTD.NextTD = TERMINATE_POINTER as ULONG;
        (*TD).HwTD.AlternateNextTD = TERMINATE_POINTER as ULONG;

        (*TD).HwTD.Token.AsULONG = 0;
        (*TD).HwTD.Token.c2rust_unnamed.set_ErrorCounter(3);

        (*TD).NextHcdTD = core::ptr::null_mut();

        if (*EhciTransfer).PendingTDs == 1 {
            FirstTD = TD;
        } else if !PrevTD.is_null() {
            (*PrevTD).HwTD.NextTD = (*TD).PhysicalAddress;
            (*PrevTD).NextHcdTD = TD;
        }

        if (*TransferParameters).TransferFlags as ULONG & USBD_TRANSFER_DIRECTION_IN as ULONG != 0 {
            (*TD).HwTD.Token.c2rust_unnamed.set_PIDCode(EHCI_TD_TOKEN_PID_IN as ULONG);
        } else {
            (*TD).HwTD.Token.c2rust_unnamed.set_PIDCode(EHCI_TD_TOKEN_PID_OUT as ULONG);
        }

        (*TD).HwTD.Token.c2rust_unnamed.set_Status(EHCI_TOKEN_STATUS_ACTIVE as UCHAR as ULONG);
        (*TD).HwTD.Token.c2rust_unnamed.set_DataToggle(1);

        TransferedLen = EHCI_MapAsyncTransferToTd(
            EhciExtension,
            (*EhciEndpoint).EndpointProperties.TotalMaxPacketSize as ULONG,
            TransferedLen,
            core::ptr::null_mut(),
            EhciTransfer,
            TD,
            SgList,
        );

        PrevTD = TD;
    }

    (*TD).HwTD.Token.c2rust_unnamed.set_InterruptOnComplete(1);

    (*TD).HwTD.NextTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;
    (*TD).NextHcdTD = (*EhciEndpoint).HcdTailP;

    EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);

    EHCI_EnablePeriodicList(EhciExtension);

    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SubmitTransfer(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    ehciTransfer: PVOID,
    SgList: PUSBPORT_SCATTER_GATHER_LIST,
) -> MPSTATUS {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let EhciEndpoint: PEHCI_ENDPOINT = ehciEndpoint as PEHCI_ENDPOINT;
    let EhciTransfer: PEHCI_TRANSFER = ehciTransfer as PEHCI_TRANSFER;
    let MPStatus: MPSTATUS;

    RtlZeroMemory(
        EhciTransfer as *mut core::ffi::c_void,
        core::mem::size_of::<EHCI_TRANSFER>(),
    );

    (*EhciTransfer).TransferParameters = TransferParameters;
    (*EhciTransfer).USBDStatus = USBD_STATUS_SUCCESS as ULONG;
    (*EhciTransfer).EhciEndpoint = EhciEndpoint;

    match (*EhciEndpoint).EndpointProperties.TransferType {
        USBPORT_TRANSFER_TYPE_CONTROL => {
            MPStatus = EHCI_ControlTransfer(
                EhciExtension,
                EhciEndpoint,
                TransferParameters,
                EhciTransfer,
                SgList,
            );
        }
        USBPORT_TRANSFER_TYPE_BULK => {
            MPStatus = EHCI_BulkTransfer(
                EhciExtension,
                EhciEndpoint,
                TransferParameters,
                EhciTransfer,
                SgList,
            );
        }
        USBPORT_TRANSFER_TYPE_INTERRUPT => {
            MPStatus = EHCI_InterruptTransfer(
                EhciExtension,
                EhciEndpoint,
                TransferParameters,
                EhciTransfer,
                SgList,
            );
        }
        _ => {
            DbgBreakPoint();
            MPStatus = MP_STATUS_NOT_SUPPORTED as MPSTATUS;
        }
    }

    return MPStatus;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SubmitIsoTransfer(
    mut ehciExtension: PVOID,
    mut ehciEndpoint: PVOID,
    mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS,
    mut ehciTransfer: PVOID,
    mut isoParameters: PVOID,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_SubmitIsoTransfer: UNIMPLEMENTED. FIXME\n\0".as_ptr() as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const ::core::ffi::c_char,
        2449 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const ::core::ffi::c_char,
            2449 as ::core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_AbortIsoTransfer(
    mut EhciExtension: PEHCI_EXTENSION,
    mut EhciEndpoint: PEHCI_ENDPOINT,
    mut EhciTransfer: PEHCI_TRANSFER,
) {
    if DbgPrint(
        b"(%s:%d) EHCI_AbortIsoTransfer: UNIMPLEMENTED. FIXME\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        2459 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            2459 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_AbortAsyncTransfer(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    EhciTransfer: PEHCI_TRANSFER,
) {
    let mut QH: PEHCI_HCD_QH;
    let mut TD: PEHCI_HCD_TD;
    let mut TransferLength: ULONG;
    let mut CurrentTD: PEHCI_HCD_TD;
    let mut CurrentTransfer: PEHCI_TRANSFER;
    let mut FirstTdPA: ULONG;
    let mut LastTD: PEHCI_HCD_TD;
    let mut PrevTD: PEHCI_HCD_TD = core::ptr::null_mut();
    let mut NextTD: ULONG;

    QH = (*EhciEndpoint).QH;
    TD = (*EhciEndpoint).HcdHeadP;

    (*EhciEndpoint).PendingTDs -= 1;

    if (*TD).EhciTransfer == EhciTransfer {
        TransferLength = 0;

        while TD != (*EhciEndpoint).HcdTailP && (*TD).EhciTransfer == EhciTransfer {
            TransferLength += (*TD).LengthThisTD - (*TD).HwTD.Token.TransferBytes();

            (*TD).HwTD.NextTD = 0;
            (*TD).HwTD.AlternateNextTD = 0;

            (*TD).TdFlags = 0;
            (*TD).EhciTransfer = core::ptr::null_mut();

            (*EhciEndpoint).RemainTDs += 1;

            TD = (*TD).NextHcdTD;
        }

        if TransferLength != 0 {
            (*EhciTransfer).TransferLen += TransferLength;
        }

        (*QH).sqh.HwQH.CurrentTD = (*EhciEndpoint).DmaBufferPA;
        (*QH).sqh.HwQH.NextTD = (*TD).PhysicalAddress;
        (*QH).sqh.HwQH.AlternateNextTD = (*TD).HwTD.AlternateNextTD;

        (*QH).sqh.HwQH.Token.set_TransferBytes(0);
        (*QH).sqh.HwQH.Token.set_Status(
            !((EHCI_TOKEN_STATUS_ACTIVE | EHCI_TOKEN_STATUS_HALTED) as UCHAR) as ULONG,
        );

        (*EhciEndpoint).HcdHeadP = TD;
    } else {
        CurrentTD = (RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer"))(
            (*QH).sqh.HwQH.CurrentTD,
            EhciExtension as PVOID,
            EhciEndpoint as PVOID,
        ) as PEHCI_HCD_TD;

        CurrentTransfer = (*CurrentTD).EhciTransfer;
        TD = (*EhciEndpoint).HcdHeadP;

        while !TD.is_null() && (*TD).EhciTransfer != EhciTransfer {
            PrevTD = TD;
            TD = (*TD).NextHcdTD;
        }

        FirstTdPA = (*TD).PhysicalAddress;

        while !TD.is_null() && (*TD).EhciTransfer == EhciTransfer {
            (*TD).HwTD.NextTD = 0;
            (*TD).HwTD.AlternateNextTD = 0;

            (*TD).TdFlags = 0;
            (*TD).EhciTransfer = core::ptr::null_mut();

            (*EhciEndpoint).RemainTDs += 1;

            TD = (*TD).NextHcdTD;
        }

        LastTD = TD;
        NextTD = (*LastTD).PhysicalAddress + core::mem::offset_of!(EHCI_HCD_TD, HwTD) as ULONG + core::mem::offset_of!(EHCI_HW_TD, NextTD) as ULONG;

        (*PrevTD).HwTD.NextTD = (*LastTD).PhysicalAddress;
        (*PrevTD).HwTD.AlternateNextTD = (*LastTD).PhysicalAddress;

        (*PrevTD).NextHcdTD = LastTD;
        (*PrevTD).AltNextHcdTD = LastTD;

        if CurrentTransfer == EhciTransfer {
            (*QH).sqh.HwQH.CurrentTD = (*EhciEndpoint).DmaBufferPA;

            (*QH).sqh.HwQH.Token.set_Status(!(EHCI_TOKEN_STATUS_ACTIVE as UCHAR) as ULONG);
            (*QH).sqh.HwQH.Token.set_TransferBytes(0);

            (*QH).sqh.HwQH.NextTD = NextTD;
            (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;

            return;
        }

        if (*PrevTD).EhciTransfer == CurrentTransfer {
            if (*QH).sqh.HwQH.NextTD == FirstTdPA {
                (*QH).sqh.HwQH.NextTD = NextTD;
            }

            if (*QH).sqh.HwQH.AlternateNextTD == FirstTdPA {
                (*QH).sqh.HwQH.AlternateNextTD = NextTD;
            }

            TD = (*EhciEndpoint).HcdHeadP;
            while !TD.is_null() {
                if (*TD).EhciTransfer == CurrentTransfer {
                    (*TD).HwTD.AlternateNextTD = NextTD;
                    (*TD).AltNextHcdTD = LastTD;
                }
                TD = (*TD).NextHcdTD;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_AbortTransfer(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    ehciTransfer: PVOID,
    CompletedLength: PULONG,
) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let EhciEndpoint: PEHCI_ENDPOINT = ehciEndpoint as PEHCI_ENDPOINT;
    let EhciTransfer: PEHCI_TRANSFER = ehciTransfer as PEHCI_TRANSFER;
    let TransferType: ULONG;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        EHCI_AbortIsoTransfer(EhciExtension, EhciEndpoint, EhciTransfer);
    } else {
        EHCI_AbortAsyncTransfer(EhciExtension, EhciEndpoint, EhciTransfer);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetEndpointState(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
) -> ULONG {
    if DbgPrint(
        b"(%s:%d) EHCI_GetEndpointState: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const core::ffi::c_char,
        2626 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const core::ffi::c_char,
            2626 as core::ffi::c_int,
        );
    }
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RemoveQhFromPeriodicList(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) {
    let mut QH: PEHCI_HCD_QH;
    let mut NextHead: PEHCI_HCD_QH;
    let mut NextQhPA: ULONG;
    let mut PrevHead: PEHCI_HCD_QH;

    QH = (*EhciEndpoint).QH;

    if (*QH).sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE == 0 {
        return;
    }

    NextHead = (*QH).sqh.NextHead;
    PrevHead = (*QH).sqh.PrevHead;

    (*PrevHead).sqh.NextHead = NextHead;

    if !NextHead.is_null() {
        if (*NextHead).sqh.QhFlags & EHCI_QH_FLAG_STATIC == 0 {
            (*NextHead).sqh.PrevHead = PrevHead;
        }

        NextQhPA = (*NextHead).sqh.PhysicalAddress;
        NextQhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
        NextQhPA |= (EHCI_LINK_TYPE_QH << 1) as ULONG;

        (*PrevHead).sqh.HwQH.HorizontalLink.AsULONG = NextQhPA;
    } else {
        (*PrevHead).sqh.HwQH.HorizontalLink.set_Terminate(1);
    }

    (*QH).sqh.QhFlags &= !EHCI_QH_FLAG_IN_SCHEDULE;

    (*QH).sqh.NextHead = core::ptr::null_mut();
    (*QH).sqh.PrevHead = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RemoveQhFromAsyncList(
    EhciExtension: PEHCI_EXTENSION,
    QH: PEHCI_HCD_QH,
) {
    let mut NextHead: PEHCI_HCD_QH;
    let mut NextHeadPA: ULONG;
    let mut PrevHead: PEHCI_HCD_QH;
    let mut AsyncHead: PEHCI_STATIC_QH;
    let mut AsyncHeadPA: ULONG;

    if (*QH).sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE != 0 {
        NextHead = (*QH).sqh.NextHead;
        PrevHead = (*QH).sqh.PrevHead;

        AsyncHead = (*EhciExtension).AsyncHead;

        AsyncHeadPA = (*AsyncHead).PhysicalAddress;
        AsyncHeadPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
        AsyncHeadPA |= EHCI_LINK_TYPE_QH << 1;

        NextHeadPA = (*NextHead).sqh.PhysicalAddress;
        NextHeadPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
        NextHeadPA |= EHCI_LINK_TYPE_QH << 1;

        (*PrevHead).sqh.HwQH.HorizontalLink.AsULONG = NextHeadPA;

        (*PrevHead).sqh.NextHead = NextHead;
        (*NextHead).sqh.PrevHead = PrevHead;

        EHCI_FlushAsyncCache(EhciExtension);

        if READ_REGISTER_ULONG(
            core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).AsyncListBase) as *mut ULONG,
        ) == (*QH).sqh.PhysicalAddress
        {
            WRITE_REGISTER_ULONG(
                core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).AsyncListBase)
                    as *mut ULONG,
                AsyncHeadPA,
            );
        }

        (*QH).sqh.QhFlags &= !EHCI_QH_FLAG_IN_SCHEDULE;
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InsertQhInPeriodicList(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) {
    let mut StaticQH: PEHCI_STATIC_QH;
    let mut QH: PEHCI_HCD_QH;
    let mut QhPA: ULONG;
    let mut NextHead: PEHCI_HCD_QH;
    let mut PrevHead: PEHCI_HCD_QH;

    QH = (*EhciEndpoint).QH;
    StaticQH = (*EhciEndpoint).StaticQH;

    NextHead = (*StaticQH).NextHead as PEHCI_HCD_QH;

    (*QH).sqh.Period = (*EhciEndpoint).EndpointProperties.Period as ULONG;
    (*QH).sqh.Ordinal = (*EhciEndpoint).EndpointProperties.Reserved6;

    PrevHead = StaticQH as PEHCI_HCD_QH;

    if (*StaticQH).QhFlags as u32 & EHCI_QH_FLAG_STATIC as u32 != 0
        && (NextHead.is_null()
            || (*NextHead).sqh.QhFlags as u32 & EHCI_QH_FLAG_STATIC as u32 != 0)
    {
    } else {
        while !NextHead.is_null()
            && (*NextHead).sqh.QhFlags as u32 & EHCI_QH_FLAG_STATIC as u32 == 0
            && (*QH).sqh.Ordinal > (*NextHead).sqh.Ordinal
        {
            PrevHead = NextHead;
            NextHead = (*NextHead).sqh.NextHead as PEHCI_HCD_QH;
        }
    }

    (*QH).sqh.NextHead = NextHead as *mut _EHCI_HCD_QH;
    (*QH).sqh.PrevHead = PrevHead as *mut _EHCI_HCD_QH;

    if !NextHead.is_null() && (*NextHead).sqh.QhFlags as u32 & EHCI_QH_FLAG_STATIC as u32 == 0 {
        (*NextHead).sqh.PrevHead = QH as *mut _EHCI_HCD_QH;
    }

    (*QH).sqh.QhFlags |= EHCI_QH_FLAG_IN_SCHEDULE as u32;
    (*QH).sqh.HwQH.HorizontalLink = (*PrevHead).sqh.HwQH.HorizontalLink;

    (*PrevHead).sqh.NextHead = QH as *mut _EHCI_HCD_QH;

    QhPA = (*QH).sqh.PhysicalAddress;
    QhPA &= (LINK_POINTER_MASK.wrapping_add(TERMINATE_POINTER as u32)) as ULONG;
    QhPA |= (EHCI_LINK_TYPE_QH << 1) as ULONG;

    (*PrevHead).sqh.HwQH.HorizontalLink.AsULONG = QhPA;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InsertQhInAsyncList(
    EhciExtension: PEHCI_EXTENSION,
    QH: PEHCI_HCD_QH,
) {

    let mut AsyncHead: PEHCI_STATIC_QH;
    let mut QhPA: ULONG;
    let mut NextHead: PEHCI_HCD_QH;

    if !((*QH).sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE == 0) {
        RtlAssert(
            b"(QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE) == 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const i8 as PVOID,
            2799 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    if !((*QH).sqh.QhFlags & EHCI_QH_FLAG_NUKED == 0) {
        RtlAssert(
            b"(QH->sqh.QhFlags & EHCI_QH_FLAG_NUKED) == 0\0".as_ptr() as *const i8 as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const i8 as PVOID,
            2800 as ULONG,
            core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    AsyncHead = (*EhciExtension).AsyncHead;
    NextHead = (*AsyncHead).NextHead as PEHCI_HCD_QH;

    (*QH).sqh.HwQH.HorizontalLink = (*AsyncHead).HwQH.HorizontalLink;
    (*QH).sqh.QhFlags |= EHCI_QH_FLAG_IN_SCHEDULE;
    (*QH).sqh.NextHead = NextHead;
    (*QH).sqh.PrevHead = AsyncHead as PEHCI_HCD_QH;

    (*NextHead).sqh.PrevHead = QH;

    QhPA = (*QH).sqh.PhysicalAddress;
    QhPA &= (LINK_POINTER_MASK.wrapping_add(TERMINATE_POINTER as u32)) as ULONG;
    QhPA |= (EHCI_LINK_TYPE_QH << 1) as ULONG;

    (*AsyncHead).HwQH.HorizontalLink.AsULONG = QhPA;

    (*AsyncHead).NextHead = QH;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SetIsoEndpointState(
    mut EhciExtension: PEHCI_EXTENSION,
    mut EhciEndpoint: PEHCI_ENDPOINT,
    mut EndpointState: ULONG,
) {
    if DbgPrint(
        b"(%s:%d) EHCI_SetIsoEndpointState: UNIMPLEMENTED. FIXME\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        2827 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            2827 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SetAsyncEndpointState(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
    EndpointState: ULONG,
) {
    let QH: PEHCI_HCD_QH;
    let TransferType: ULONG;

    QH = (*EhciEndpoint).QH;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    match EndpointState {
        USBPORT_ENDPOINT_PAUSED => {
            if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
                EHCI_RemoveQhFromPeriodicList(EhciExtension, EhciEndpoint);
            } else {
                EHCI_RemoveQhFromAsyncList(EhciExtension, (*EhciEndpoint).QH);
            }
        }
        USBPORT_ENDPOINT_ACTIVE => {
            if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
                EHCI_InsertQhInPeriodicList(EhciExtension, EhciEndpoint);
            } else {
                EHCI_InsertQhInAsyncList(EhciExtension, (*EhciEndpoint).QH);
            }
        }
        USBPORT_ENDPOINT_REMOVE => {
            (*QH).sqh.QhFlags |= EHCI_QH_FLAG_CLOSED as ULONG;

            if TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG {
                EHCI_RemoveQhFromPeriodicList(EhciExtension, EhciEndpoint);
            } else {
                EHCI_RemoveQhFromAsyncList(EhciExtension, (*EhciEndpoint).QH);
            }
        }
        _ => {
            DbgBreakPoint();
        }
    }

    (*EhciEndpoint).EndpointState = EndpointState;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SetEndpointState(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    EndpointState: ULONG,
) {
    let mut EhciEndpoint: PEHCI_ENDPOINT;
    let mut TransferType: ULONG;

    EhciEndpoint = ehciEndpoint as PEHCI_ENDPOINT;
    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_CONTROL as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        EHCI_SetAsyncEndpointState(
            ehciExtension as PEHCI_EXTENSION,
            EhciEndpoint,
            EndpointState,
        );
    } else if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        EHCI_SetIsoEndpointState(
            ehciExtension as PEHCI_EXTENSION,
            EhciEndpoint,
            EndpointState,
        );
    } else {
        RegPacket.UsbPortBugCheck.expect("non-null function pointer")(ehciExtension);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_InterruptNextSOF(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;

    DPRINT_EHCI!(b"EHCI_InterruptNextSOF: ... \n\0".as_ptr() as *const i8);

    (RegPacket.UsbPortInvalidateController.expect("non-null function pointer"))(
        EhciExtension as PVOID,
        USBPORT_INVALIDATE_CONTROLLER_SOFT_INTERRUPT as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetErrorFromTD(TD: PEHCI_HCD_TD) -> USBD_STATUS {
    let mut Token: EHCI_TD_TOKEN;

    DPRINT_EHCI!(b"EHCI_GetErrorFromTD: ... \n\0".as_ptr() as *const i8);

    if (*TD).HwTD.Token.c2rust_unnamed.Status() as ::core::ffi::c_int
        & EHCI_TOKEN_STATUS_HALTED
        == 0
    {
        RtlAssert(
            b"TD->HwTD.Token.__bindgen_anon_1.Status & EHCI_TOKEN_STATUS_HALTED\0" as *const u8
                as *const ::core::ffi::c_char as PVOID,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            2937 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };

    Token = (*TD).HwTD.Token;

    if Token.c2rust_unnamed.Status() as ::core::ffi::c_int & EHCI_TOKEN_STATUS_TRANSACTION_ERROR
        != 0
    {
        return USBD_STATUS_XACT_ERROR;
    }

    if Token.c2rust_unnamed.Status() as ::core::ffi::c_int & EHCI_TOKEN_STATUS_BABBLE_DETECTED
        != 0
    {
        return USBD_STATUS_BABBLE_DETECTED;
    }

    if Token.c2rust_unnamed.Status() as ::core::ffi::c_int & EHCI_TOKEN_STATUS_DATA_BUFFER_ERROR
        != 0
    {
        return USBD_STATUS_DATA_BUFFER_ERROR;
    }

    if Token.c2rust_unnamed.Status() as ::core::ffi::c_int & EHCI_TOKEN_STATUS_MISSED_MICROFRAME
        != 0
    {
        return USBD_STATUS_XACT_ERROR;
    }

    return USBD_STATUS_STALL_PID;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_ProcessDoneAsyncTd(
    EhciExtension: PEHCI_EXTENSION,
    TD: PEHCI_HCD_TD,
) {
    let mut EhciTransfer: PEHCI_TRANSFER;
    let mut TransferParameters: PUSBPORT_TRANSFER_PARAMETERS;
    let mut TransferType: ULONG;
    let mut EhciEndpoint: PEHCI_ENDPOINT;
    let mut LengthTransfered: ULONG;
    let mut USBDStatus: USBD_STATUS;
    let mut OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Command: EHCI_USB_COMMAND = core::mem::zeroed();

    DPRINT_EHCI!(b"EHCI_ProcessDoneAsyncTd: TD - %p\n\0".as_ptr() as *const i8, TD);

    EhciTransfer = (*TD).EhciTransfer;

    TransferParameters = (*EhciTransfer).TransferParameters;
    (*EhciTransfer).PendingTDs = (*EhciTransfer).PendingTDs.wrapping_sub(1);

    EhciEndpoint = (*EhciTransfer).EhciEndpoint;

    if (*TD).TdFlags & EHCI_HCD_TD_FLAG_ACTIVE == 0 {
        if (*TD).HwTD.Token.__bindgen_anon_1.Status() as i32 & EHCI_TOKEN_STATUS_HALTED != 0 {
            USBDStatus = EHCI_GetErrorFromTD(TD);
        } else {
            USBDStatus = USBD_STATUS_SUCCESS;
        }

        LengthTransfered =
            ((*TD).LengthThisTD as ULONG).wrapping_sub((*TD).HwTD.Token.TransferBytes() as ULONG);

        if (*TD).HwTD.Token.PIDCode() as i32 != EHCI_TD_TOKEN_PID_SETUP {
            (*EhciTransfer).TransferLen =
                (*EhciTransfer).TransferLen.wrapping_add(LengthTransfered);
        }

        if USBDStatus != USBD_STATUS_SUCCESS {
            (*EhciTransfer).USBDStatus = USBDStatus as ULONG;
        }
    }

    (*TD).HwTD.NextTD = 0;
    (*TD).HwTD.AlternateNextTD = 0;

    (*TD).TdFlags = 0;
    (*TD).EhciTransfer = core::ptr::null_mut();

    (*EhciEndpoint).RemainTDs = (*EhciEndpoint).RemainTDs.wrapping_add(1);

    if (*EhciTransfer).PendingTDs == 0 {
        (*EhciEndpoint).PendingTDs = (*EhciEndpoint).PendingTDs.wrapping_sub(1);

        TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

        if TransferType == USBPORT_TRANSFER_TYPE_CONTROL
            || TransferType == USBPORT_TRANSFER_TYPE_BULK
        {
            (*EhciExtension).PendingTransfers =
                (*EhciExtension).PendingTransfers.wrapping_sub(1);

            if (*EhciExtension).PendingTransfers == 0 {
                OperationalRegs = (*EhciExtension).OperationalRegs;
                Command.AsULONG = READ_REGISTER_ULONG(
                    core::ptr::addr_of_mut!((*OperationalRegs).HcCommand.AsULONG) as *mut ULONG,
                );

                if Command.InterruptAdvanceDoorbell() == 0
                    && (*EhciExtension).Flags & EHCI_FLAGS_IDLE_SUPPORT != 0
                {
                    EHCI_DisableAsyncList(EhciExtension);
                }
            }
        }

        (RegPacket.UsbPortCompleteTransfer.expect("non-null function pointer"))(
            EhciExtension as PVOID,
            EhciEndpoint as PVOID,
            TransferParameters as PVOID,
            (*EhciTransfer).USBDStatus as USBD_STATUS,
            (*EhciTransfer).TransferLen,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollActiveAsyncEndpoint(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) {
    let mut QH: PEHCI_HCD_QH;
    let mut TD: PEHCI_HCD_TD;
    let CurrentTD: PEHCI_HCD_TD;
    let CurrentTDPhys: ULONG;
    let mut IsScheduled: BOOLEAN;

    DPRINT_EHCI!(b"EHCI_PollActiveAsyncEndpoint: ... \n\0".as_ptr() as *const i8);

    QH = (*EhciEndpoint).QH;

    CurrentTDPhys = ((*QH).sqh.HwQH.CurrentTD & LINK_POINTER_MASK as ULONG) as ULONG;
    // ASSERT(CurrentTDPhys);

    CurrentTD = (RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer"))(
        CurrentTDPhys,
        EhciExtension as PVOID,
        EhciEndpoint as PVOID,
    ) as PEHCI_HCD_TD;

    if CurrentTD == (*EhciEndpoint).DmaBufferVA as PEHCI_HCD_TD {
        return;
    }

    IsScheduled = ((*QH).sqh.QhFlags as ULONG & EHCI_QH_FLAG_IN_SCHEDULE as ULONG) as BOOLEAN;

    if EHCI_HardwarePresent(EhciExtension, 0 as BOOLEAN) == 0 {
        IsScheduled = 0 as BOOLEAN;
    }

    TD = (*EhciEndpoint).HcdHeadP;

    if TD == CurrentTD {
        if TD != (*EhciEndpoint).HcdTailP
            && (*TD).HwTD.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE == 0
        {
            if !(*TD).NextHcdTD.is_null()
                && (*TD).HwTD.NextTD != (*(*TD).NextHcdTD).PhysicalAddress
            {
                (*TD).HwTD.NextTD = (*(*TD).NextHcdTD).PhysicalAddress;
            }

            if !(*TD).AltNextHcdTD.is_null()
                && (*TD).HwTD.AlternateNextTD != (*(*TD).AltNextHcdTD).PhysicalAddress
            {
                (*TD).HwTD.AlternateNextTD = (*(*TD).AltNextHcdTD).PhysicalAddress;
            }

            if (*QH).sqh.HwQH.CurrentTD == (*TD).PhysicalAddress
                && (*TD).HwTD.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE == 0
                && ((*QH).sqh.HwQH.NextTD != (*TD).HwTD.NextTD
                    || (*QH).sqh.HwQH.AlternateNextTD != (*TD).HwTD.AlternateNextTD)
            {
                (*QH).sqh.HwQH.NextTD = (*TD).HwTD.NextTD;
                (*QH).sqh.HwQH.AlternateNextTD = (*TD).HwTD.AlternateNextTD;
            }

            EHCI_InterruptNextSOF(EhciExtension as PVOID);
        }
    } else {
        while TD != CurrentTD {
            // ASSERT((TD->TdFlags & EHCI_HCD_TD_FLAG_DUMMY) == 0);

            (*TD).TdFlags |= EHCI_HCD_TD_FLAG_DONE as ULONG;

            if (*TD).HwTD.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE != 0 {
                (*TD).TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE as ULONG;
            }

            InsertTailList(
                core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs),
                core::ptr::addr_of_mut!((*TD).DoneLink),
            );
            TD = (*TD).NextHcdTD as PEHCI_HCD_TD;
        }
    }

    if (*CurrentTD).HwTD.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE != 0 {
        // ASSERT(TD != NULL);
        (*EhciEndpoint).HcdHeadP = TD;
        return;
    }

    if (*CurrentTD).NextHcdTD != (*EhciEndpoint).HcdTailP
        && ((*CurrentTD).AltNextHcdTD != (*EhciEndpoint).HcdTailP
            || (*CurrentTD).HwTD.Token.c2rust_unnamed.TransferBytes() as i32 == 0)
    {
        // ASSERT(TD != NULL);
        (*EhciEndpoint).HcdHeadP = TD;
        return;
    }

    if IsScheduled != 0 {
        EHCI_LockQH(
            EhciExtension,
            QH,
            (*EhciEndpoint).EndpointProperties.TransferType,
        );
    }

    (*QH).sqh.HwQH.CurrentTD = (*EhciEndpoint).DmaBufferPA;

    (*CurrentTD).TdFlags |= EHCI_HCD_TD_FLAG_DONE as ULONG;
    InsertTailList(
        core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs),
        core::ptr::addr_of_mut!((*CurrentTD).DoneLink),
    );

    if (*CurrentTD).HwTD.Token.c2rust_unnamed.TransferBytes() as i32 != 0
        && (*CurrentTD).AltNextHcdTD == (*EhciEndpoint).HcdTailP
    {
        TD = (*CurrentTD).NextHcdTD as PEHCI_HCD_TD;

        while TD != (*EhciEndpoint).HcdTailP {
            (*TD).TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE as ULONG;
            InsertTailList(
                core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs),
                core::ptr::addr_of_mut!((*TD).DoneLink),
            );
            TD = (*TD).NextHcdTD as PEHCI_HCD_TD;
        }
    }

    (*QH).sqh.HwQH.CurrentTD = (*(*EhciEndpoint).HcdTailP).PhysicalAddress;
    (*QH).sqh.HwQH.NextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.Token.c2rust_unnamed.set_TransferBytes(0 as ULONG);

    (*EhciEndpoint).HcdHeadP = (*EhciEndpoint).HcdTailP;

    if IsScheduled != 0 {
        EHCI_UnlockQH(EhciExtension, QH);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollHaltedAsyncEndpoint(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) {
    let mut QH: PEHCI_HCD_QH;
    let mut CurrentTD: PEHCI_HCD_TD;
    let mut CurrentTdPA: ULONG;
    let mut TD: PEHCI_HCD_TD;
    let Transfer: PEHCI_TRANSFER;
    let mut IsScheduled: BOOLEAN;

    QH = (*EhciEndpoint).QH;
    EHCI_DumpHwQH(QH);

    CurrentTdPA = ((*QH).sqh.HwQH.CurrentTD & LINK_POINTER_MASK as ULONG) as ULONG;

    IsScheduled = ((*QH).sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE) as BOOLEAN;

    if EHCI_HardwarePresent(EhciExtension, 0) == 0 {
        IsScheduled = 0;
    }

    CurrentTD = RegPacket.UsbPortGetMappedVirtualAddress.expect("non-null function pointer")(
        CurrentTdPA,
        EhciExtension as PVOID,
        EhciEndpoint as PVOID,
    ) as PEHCI_HCD_TD;

    if CurrentTD == (*EhciEndpoint).DmaBufferVA as PEHCI_HCD_TD {
        return;
    }

    if IsScheduled != 0 {
        EHCI_LockQH(
            EhciExtension,
            QH,
            (*EhciEndpoint).EndpointProperties.TransferType,
        );
    }

    TD = (*EhciEndpoint).HcdHeadP;

    while TD != CurrentTD {
        if (*TD).HwTD.Token.__bindgen_anon_1.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE != 0 {
            (*TD).TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
        }

        (*TD).TdFlags |= EHCI_HCD_TD_FLAG_DONE;

        InsertTailList(
            core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs),
            core::ptr::addr_of_mut!((*TD).DoneLink),
        );

        TD = (*TD).NextHcdTD as PEHCI_HCD_TD;
    }

    TD = CurrentTD;

    Transfer = (*CurrentTD).EhciTransfer as PEHCI_TRANSFER;

    loop {
        if (*TD).HwTD.Token.__bindgen_anon_1.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE != 0 {
            (*TD).TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
        }

        (*TD).TdFlags |= EHCI_HCD_TD_FLAG_DONE;

        InsertTailList(
            core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs),
            core::ptr::addr_of_mut!((*TD).DoneLink),
        );

        TD = (*TD).NextHcdTD as PEHCI_HCD_TD;

        if !((*TD).EhciTransfer == Transfer) {
            break;
        }
    }

    (*EhciEndpoint).HcdHeadP = TD;

    (*QH).sqh.HwQH.CurrentTD = (*EhciEndpoint).DmaBufferPA;
    (*QH).sqh.HwQH.NextTD = (*TD).PhysicalAddress;
    (*QH).sqh.HwQH.AlternateNextTD = TERMINATE_POINTER as ULONG;
    (*QH).sqh.HwQH.Token.set_TransferBytes(0);

    if IsScheduled != 0 {
        EHCI_UnlockQH(EhciExtension, QH);
    }

    if (*EhciEndpoint).EndpointStatus & USBPORT_ENDPOINT_CONTROL != 0 {
        (*EhciEndpoint).EndpointStatus &= !USBPORT_ENDPOINT_HALT;
        (*QH).sqh.HwQH.Token.set_ErrorCounter(0);
        (*QH).sqh.HwQH.Token.set_Status(
            (*QH).sqh.HwQH.Token.__bindgen_anon_1.Status()
                & !((EHCI_TOKEN_STATUS_ACTIVE | EHCI_TOKEN_STATUS_HALTED) as UCHAR as ULONG),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollAsyncEndpoint(
    EhciExtension: PEHCI_EXTENSION,
    EhciEndpoint: PEHCI_ENDPOINT,
) {
    let mut QH: PEHCI_HCD_QH;
    let DoneList: PLIST_ENTRY;
    let mut TD: PEHCI_HCD_TD;

    if (*EhciEndpoint).PendingTDs == 0 {
        return;
    }

    QH = (*EhciEndpoint).QH;

    if (*QH).sqh.QhFlags as u32 & EHCI_QH_FLAG_CLOSED as u32 != 0 {
        return;
    }

    if (*QH).sqh.HwQH.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_ACTIVE != 0
        || (*QH).sqh.HwQH.Token.c2rust_unnamed.Status() as i32 & EHCI_TOKEN_STATUS_HALTED == 0
    {
        EHCI_PollActiveAsyncEndpoint(EhciExtension, EhciEndpoint);
    } else {
        (*EhciEndpoint).EndpointStatus |= USBPORT_ENDPOINT_HALT as u32;
        EHCI_PollHaltedAsyncEndpoint(EhciExtension, EhciEndpoint);
    }

    DoneList = core::ptr::addr_of_mut!((*EhciEndpoint).ListTDs) as PLIST_ENTRY;

    while !IsListEmpty(DoneList as *const LIST_ENTRY) {
        TD = ((*DoneList).Flink as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(EHCI_HCD_TD, DoneLink) as ULONG_PTR,
        ) as *mut EHCI_HCD_TD as PEHCI_HCD_TD;

        RemoveHeadList(DoneList);

        // ASSERT((TD->TdFlags & (EHCI_HCD_TD_FLAG_PROCESSED | EHCI_HCD_TD_FLAG_DONE)));

        EHCI_ProcessDoneAsyncTd(EhciExtension, TD);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollIsoEndpoint(
    mut EhciExtension: PEHCI_EXTENSION,
    mut EhciEndpoint: PEHCI_ENDPOINT,
) {
    if DbgPrint(
        b"(%s:%d) EHCI_PollIsoEndpoint: UNIMPLEMENTED. FIXME\n\0" as *const u8 as *const i8,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const i8,
        3327 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const i8,
            3327 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollEndpoint(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let EhciEndpoint: PEHCI_ENDPOINT = ehciEndpoint as PEHCI_ENDPOINT;
    let TransferType: ULONG;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        EHCI_PollIsoEndpoint(EhciExtension, EhciEndpoint);
    } else {
        EHCI_PollAsyncEndpoint(EhciExtension, EhciEndpoint);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_CheckController(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;

    if (*EhciExtension).IsStarted != 0 {
        EHCI_HardwarePresent(EhciExtension, 1 as BOOLEAN);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_Get32BitFrameNumber(ehciExtension: PVOID) -> ULONG {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let HighPart: ULONG;
    let mut Frame: ULONG;

    HighPart = (*EhciExtension).FrameHighPart;
    Frame = READ_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).FrameIndex) as *mut ULONG,
    ) / EHCI_MICROFRAMES;
    Frame &= EHCI_FRINDEX_FRAME_MASK;

    /* The ISR keeps bit 10 of HighPart in step with the frame list half */
    return ((Frame & EHCI_FRINDEX_INDEX_MASK) | HighPart)
        .wrapping_add((Frame ^ HighPart) & EHCI_FRAME_LIST_MAX_ENTRIES);
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_EnableInterrupts(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).HcInterruptEnable.AsULONG) as *mut ULONG,
        (*EhciExtension).InterruptMask.AsULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_DisableInterrupts(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;

    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*(*EhciExtension).OperationalRegs).HcInterruptEnable.AsULONG)
            as *mut ULONG,
        0 as ULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PollController(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Port: ULONG;
    let mut PortSC: EHCI_PORT_STATUS_CONTROL = core::mem::zeroed();

    DPRINT_EHCI!(b"EHCI_PollController: ... \n\0".as_ptr() as *const i8);

    OperationalRegs = (*EhciExtension).OperationalRegs;

    if (*EhciExtension).Flags & EHCI_FLAGS_CONTROLLER_SUSPEND == 0 {
        (RegPacket.UsbPortInvalidateRootHub.expect("non-null function pointer"))(EhciExtension as PVOID);
        return;
    }

    if (*EhciExtension).NumberOfPorts != 0 {
        Port = 0;
        while Port < (*EhciExtension).NumberOfPorts as ULONG {
            PortSC.AsULONG = READ_REGISTER_ULONG(
                core::ptr::addr_of_mut!(
                    (*(core::ptr::addr_of_mut!((*OperationalRegs).PortControl)
                        as *mut EHCI_PORT_STATUS_CONTROL)
                        .offset(Port as isize))
                    .AsULONG
                ) as *mut ULONG,
            );

            if PortSC.ConnectStatusChange() != 0 {
                (RegPacket.UsbPortInvalidateRootHub.expect("non-null function pointer"))(EhciExtension as PVOID);
            }

            Port = Port.wrapping_add(1);
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SetEndpointDataToggle(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    DataToggle: ULONG,
) {
    let mut EhciEndpoint: PEHCI_ENDPOINT;
    let TransferType: ULONG;

    EhciEndpoint = ehciEndpoint as PEHCI_ENDPOINT;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_BULK as ULONG
        || TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT as ULONG
    {
        (*(*EhciEndpoint).QH).sqh.HwQH.Token.set_DataToggle(DataToggle as ULONG);
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_GetEndpointStatus(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
) -> ULONG {
    let mut EhciEndpoint: PEHCI_ENDPOINT;
    let TransferType: ULONG;
    let mut EndpointStatus: ULONG = USBPORT_ENDPOINT_RUN as ULONG;

    EhciEndpoint = ehciEndpoint as PEHCI_ENDPOINT;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        return EndpointStatus;
    }

    if (*EhciEndpoint).EndpointStatus as ULONG & USBPORT_ENDPOINT_HALT as ULONG != 0 {
        EndpointStatus = USBPORT_ENDPOINT_HALT as ULONG;
    }

    return EndpointStatus;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_SetEndpointStatus(
    ehciExtension: PVOID,
    ehciEndpoint: PVOID,
    EndpointStatus: ULONG,
) {
    let mut EhciEndpoint: PEHCI_ENDPOINT;
    let mut TransferType: ULONG;
    let mut QH: PEHCI_HCD_QH;

    EhciEndpoint = ehciEndpoint as PEHCI_ENDPOINT;

    TransferType = (*EhciEndpoint).EndpointProperties.TransferType;

    if TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS as ULONG {
        if EndpointStatus == USBPORT_ENDPOINT_RUN as ULONG {
            (*EhciEndpoint).EndpointStatus &= !(USBPORT_ENDPOINT_HALT as ULONG);

            QH = (*EhciEndpoint).QH;
            { let __t = &mut (*QH).sqh.HwQH.Token.__bindgen_anon_1; let __v = __t.Status() & !(EHCI_TOKEN_STATUS_HALTED as UCHAR) as ULONG; __t.set_Status(__v); }

            return;
        }

        if EndpointStatus == USBPORT_ENDPOINT_HALT as ULONG {
            DbgBreakPoint();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_ResetController(mut ehciExtension: PVOID) {
    if DbgPrint(
        b"(%s:%d) EHCI_ResetController: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const core::ffi::c_char,
        3526 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const core::ffi::c_char,
            3526 as core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_StartSendOnePacket(
    mut ehciExtension: PVOID,
    mut PacketParameters: PVOID,
    mut Data: PVOID,
    mut pDataLength: PULONG,
    mut BufferVA: PVOID,
    mut BufferPA: PVOID,
    mut BufferLength: ULONG,
    mut pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_StartSendOnePacket: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        3540 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            3540 as ::core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_EndSendOnePacket(
    ehciExtension: PVOID,
    PacketParameters: PVOID,
    Data: PVOID,
    pDataLength: PULONG,
    BufferVA: PVOID,
    BufferPA: PVOID,
    BufferLength: ULONG,
    pUSBDStatus: *mut USBD_STATUS,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_EndSendOnePacket: UNIMPLEMENTED. FIXME\n\0".as_ptr() as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char,
        3555 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0".as_ptr() as *const core::ffi::c_char,
            3555 as core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_PassThru(
    ehciExtension: PVOID,
    passThruParameters: PVOID,
    ParameterLength: ULONG,
    pParameters: PVOID,
) -> MPSTATUS {
    if DbgPrint(
        b"(%s:%d) EHCI_PassThru: UNIMPLEMENTED. FIXME\n\0" as *const u8 as PCSTR,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const core::ffi::c_char,
        3566 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const core::ffi::c_char,
            3566 as core::ffi::c_int,
        );
    }
    return MP_STATUS_SUCCESS as MPSTATUS;
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_RebalanceEndpoint(
    ohciExtension: PVOID,
    EndpointProperties: PUSBPORT_ENDPOINT_PROPERTIES,
    ohciEndpoint: PVOID,
) {
    if DbgPrint(
        b"(%s:%d) EHCI_RebalanceEndpoint: UNIMPLEMENTED. FIXME\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        3576 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            3576 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_FlushInterrupts(ehciExtension: PVOID) {
    let EhciExtension: PEHCI_EXTENSION = ehciExtension as PEHCI_EXTENSION;
    let OperationalRegs: PEHCI_HW_REGISTERS;
    let mut Status: EHCI_USB_STATUS = core::mem::zeroed();

    OperationalRegs = (*EhciExtension).OperationalRegs;

    Status.AsULONG = READ_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG) as *mut ULONG,
    );
    WRITE_REGISTER_ULONG(
        core::ptr::addr_of_mut!((*OperationalRegs).HcStatus.AsULONG) as *mut ULONG,
        Status.AsULONG,
    );
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_TakePortControl(mut ohciExtension: PVOID) {
    if DbgPrint(
        b"(%s:%d) EHCI_TakePortControl: UNIMPLEMENTED. FIXME\n\0" as *const u8 as *const i8,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const i8,
        3599 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const i8,
            3599 as i32,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn EHCI_Unload(mut DriverObject: PDRIVER_OBJECT) {
    if DbgPrint(
        b"(%s:%d) EHCI_Unload: Not supported\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/usb/usbehci/usbehci.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        3607 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/usb/usbehci/usbehci.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            3607 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {

    if USBPORT_GetHciMn() != USBPORT_HCI_MN as ULONG {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(RegPacket) as *mut u8,
        0,
        core::mem::size_of::<USBPORT_REGISTRATION_PACKET>(),
    );

    RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_EHCI as ULONG;

    RegPacket.MiniPortFlags = (USB_MINIPORT_FLAGS_INTERRUPT
        | USB_MINIPORT_FLAGS_MEMORY_IO
        | USB_MINIPORT_FLAGS_USB2
        | USB_MINIPORT_FLAGS_POLLING
        | USB_MINIPORT_FLAGS_WAKE_SUPPORT) as ULONG;

    RegPacket.MiniPortBusBandwidth = TOTAL_USB20_BUS_BANDWIDTH as ULONG;

    RegPacket.MiniPortExtensionSize = (core::mem::size_of::<EHCI_EXTENSION>() as _);
    RegPacket.MiniPortEndpointSize = (core::mem::size_of::<EHCI_ENDPOINT>() as _);
    RegPacket.MiniPortTransferSize = (core::mem::size_of::<EHCI_TRANSFER>() as _);
    RegPacket.MiniPortResourcesSize = (core::mem::size_of::<EHCI_HC_RESOURCES>() as _);

    RegPacket.OpenEndpoint = Some(EHCI_OpenEndpoint);
    RegPacket.ReopenEndpoint = Some(EHCI_ReopenEndpoint);
    RegPacket.QueryEndpointRequirements = Some(EHCI_QueryEndpointRequirements);
    RegPacket.CloseEndpoint = Some(EHCI_CloseEndpoint);
    RegPacket.StartController = Some(EHCI_StartController);
    RegPacket.StopController = Some(EHCI_StopController);
    RegPacket.SuspendController = Some(EHCI_SuspendController);
    RegPacket.ResumeController = Some(EHCI_ResumeController);
    RegPacket.InterruptService = Some(EHCI_InterruptService);
    RegPacket.InterruptDpc = Some(EHCI_InterruptDpc);
    RegPacket.SubmitTransfer = Some(EHCI_SubmitTransfer);
    RegPacket.SubmitIsoTransfer = Some(EHCI_SubmitIsoTransfer);
    RegPacket.AbortTransfer = Some(EHCI_AbortTransfer);
    RegPacket.GetEndpointState = Some(EHCI_GetEndpointState);
    RegPacket.SetEndpointState = Some(EHCI_SetEndpointState);
    RegPacket.PollEndpoint = Some(EHCI_PollEndpoint);
    RegPacket.CheckController = Some(EHCI_CheckController);
    RegPacket.Get32BitFrameNumber = Some(EHCI_Get32BitFrameNumber);
    RegPacket.InterruptNextSOF = Some(EHCI_InterruptNextSOF);
    RegPacket.EnableInterrupts = Some(EHCI_EnableInterrupts);
    RegPacket.DisableInterrupts = Some(EHCI_DisableInterrupts);
    RegPacket.PollController = Some(EHCI_PollController);
    RegPacket.SetEndpointDataToggle = Some(EHCI_SetEndpointDataToggle);
    RegPacket.GetEndpointStatus = Some(EHCI_GetEndpointStatus);
    RegPacket.SetEndpointStatus = Some(EHCI_SetEndpointStatus);
    RegPacket.RH_GetRootHubData = Some(EHCI_RH_GetRootHubData);
    RegPacket.RH_GetStatus = Some(EHCI_RH_GetStatus);
    RegPacket.RH_GetPortStatus = Some(EHCI_RH_GetPortStatus);
    RegPacket.RH_GetHubStatus = Some(EHCI_RH_GetHubStatus);
    RegPacket.RH_SetFeaturePortReset = Some(EHCI_RH_SetFeaturePortReset);
    RegPacket.RH_SetFeaturePortPower = Some(EHCI_RH_SetFeaturePortPower);
    RegPacket.RH_SetFeaturePortEnable = Some(EHCI_RH_SetFeaturePortEnable);
    RegPacket.RH_SetFeaturePortSuspend = Some(EHCI_RH_SetFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnable = Some(EHCI_RH_ClearFeaturePortEnable);
    RegPacket.RH_ClearFeaturePortPower = Some(EHCI_RH_ClearFeaturePortPower);
    RegPacket.RH_ClearFeaturePortSuspend = Some(EHCI_RH_ClearFeaturePortSuspend);
    RegPacket.RH_ClearFeaturePortEnableChange = Some(EHCI_RH_ClearFeaturePortEnableChange);
    RegPacket.RH_ClearFeaturePortConnectChange = Some(EHCI_RH_ClearFeaturePortConnectChange);
    RegPacket.RH_ClearFeaturePortResetChange = Some(EHCI_RH_ClearFeaturePortResetChange);
    RegPacket.RH_ClearFeaturePortSuspendChange = Some(EHCI_RH_ClearFeaturePortSuspendChange);
    RegPacket.RH_ClearFeaturePortOvercurrentChange =
        Some(EHCI_RH_ClearFeaturePortOvercurrentChange);
    RegPacket.RH_DisableIrq = Some(EHCI_RH_DisableIrq);
    RegPacket.RH_EnableIrq = Some(EHCI_RH_EnableIrq);
    RegPacket.StartSendOnePacket = Some(EHCI_StartSendOnePacket);
    RegPacket.EndSendOnePacket = Some(EHCI_EndSendOnePacket);
    RegPacket.PassThru = Some(EHCI_PassThru);
    RegPacket.RebalanceEndpoint = Some(EHCI_RebalanceEndpoint);
    RegPacket.FlushInterrupts = Some(EHCI_FlushInterrupts);
    RegPacket.RH_ChirpRootPort = Some(EHCI_RH_ChirpRootPort);
    RegPacket.TakePortControl = Some(EHCI_TakePortControl);

    (*DriverObject).DriverUnload = core::mem::transmute(Some(EHCI_Unload as unsafe extern "system" fn(PDRIVER_OBJECT)));

    return USBPORT_RegisterUSBPortDriver(
        DriverObject,
        USB20_MINIPORT_INTERFACE_VERSION as ULONG,
        core::ptr::addr_of_mut!(RegPacket),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS USB EHCI Miniport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     USBEHCI main driver functions
//  * COPYRIGHT:   Copyright 2017-2018 Vadim Galyant <vgal@rambler.ru>
//  */
//
// #include "usbehci.h"
//
// #define NDEBUG
// #include <debug.h>
//
// #define NDEBUG_EHCI_TRACE
// #include "dbg_ehci.h"
//
// USBPORT_REGISTRATION_PACKET RegPacket;
//
// static const UCHAR ClassicPeriod[8] = {
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
// static const EHCI_PERIOD pTable[] = {
//     { ENDPOINT_INTERRUPT_1ms, 0x00, 0xFF },
//     { ENDPOINT_INTERRUPT_2ms, 0x00, 0x55 },
//     { ENDPOINT_INTERRUPT_2ms, 0x00, 0xAA },
//     { ENDPOINT_INTERRUPT_4ms, 0x00, 0x11 },
//     { ENDPOINT_INTERRUPT_4ms, 0x00, 0x44 },
//     { ENDPOINT_INTERRUPT_4ms, 0x00, 0x22 },
//     { ENDPOINT_INTERRUPT_4ms, 0x00, 0x88 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x01 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x10 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x04 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x40 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x02 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x20 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x08 },
//     { ENDPOINT_INTERRUPT_8ms, 0x00, 0x80 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x01 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x01 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x10 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x10 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x04 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x04 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x40 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x40 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x02 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x02 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x20 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x20 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x08 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x08 },
//     { ENDPOINT_INTERRUPT_16ms, 0x01, 0x80 },
//     { ENDPOINT_INTERRUPT_16ms, 0x02, 0x80 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x01 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x01 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x01 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x01 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x10 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x10 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x10 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x10 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x04 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x04 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x04 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x04 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x40 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x40 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x40 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x40 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x02 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x02 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x02 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x02 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x20 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x20 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x20 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x20 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x08 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x08 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x08 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x08 },
//     { ENDPOINT_INTERRUPT_32ms, 0x03, 0x80 },
//     { ENDPOINT_INTERRUPT_32ms, 0x05, 0x80 },
//     { ENDPOINT_INTERRUPT_32ms, 0x04, 0x80 },
//     { ENDPOINT_INTERRUPT_32ms, 0x06, 0x80 },
//     { 0x00, 0x00, 0x00 }
// };
// C_ASSERT(RTL_NUMBER_OF(pTable) == INTERRUPT_ENDPOINTs + 1);
//
// static const UCHAR Balance[] = {
//     0, 16, 8, 24, 4, 20, 12, 28, 2, 18, 10, 26, 6, 22, 14, 30,
//     1, 17, 9, 25, 5, 21, 13, 29, 3, 19, 11, 27, 7, 23, 15, 31
// };
// C_ASSERT(RTL_NUMBER_OF(Balance) == EHCI_FRAMES);
//
// static const UCHAR LinkTable[] = {
//     255, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8,  9, 9,
//     10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19,
//     20, 20, 21, 21, 22, 22, 23, 23, 24, 24, 25, 25, 26, 26, 27, 27, 28, 28, 29, 29,
//     30, 30, 0
// };
// C_ASSERT(RTL_NUMBER_OF(LinkTable) == INTERRUPT_ENDPOINTs + 1);
//
// PEHCI_HCD_TD
// NTAPI
// EHCI_AllocTd(IN PEHCI_EXTENSION EhciExtension,
//              IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_TD TD;
//     ULONG ix;
//
//     DPRINT_EHCI!("EHCI_AllocTd: ... \n");
//
//     if (EhciEndpoint->MaxTDs == 0)
//     {
//         RegPacket.UsbPortBugCheck(EhciExtension);
//         return NULL;
//     }
//
//     TD = EhciEndpoint->FirstTD;
//
//     for (ix = 1; TD->TdFlags & EHCI_HCD_TD_FLAG_ALLOCATED; ix++)
//     {
//         TD += 1;
//
//         if (ix >= EhciEndpoint->MaxTDs)
//         {
//             RegPacket.UsbPortBugCheck(EhciExtension);
//             return NULL;
//         }
//     }
//
//     TD->TdFlags |= EHCI_HCD_TD_FLAG_ALLOCATED;
//
//     EhciEndpoint->RemainTDs--;
//
//     return TD;
// }
//
// PEHCI_HCD_QH
// NTAPI
// EHCI_InitializeQH(IN PEHCI_EXTENSION EhciExtension,
//                   IN PEHCI_ENDPOINT EhciEndpoint,
//                   IN PEHCI_HCD_QH QH,
//                   IN ULONG QhPA)
// {
//     PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties;
//     ULONG DeviceSpeed;
//
//     DPRINT_EHCI!("EHCI_InitializeQH: EhciEndpoint - %p, QH - %p, QhPA - %p\n",
//                 EhciEndpoint,
//                 QH,
//                 QhPA);
//
//     EndpointProperties = &EhciEndpoint->EndpointProperties;
//
//     RtlZeroMemory(QH, sizeof(EHCI_HCD_QH));
//
//     ASSERT((QhPA & ~LINK_POINTER_MASK) == 0);
//
//     QH->sqh.PhysicalAddress = QhPA;
//
//     QH->sqh.HwQH.EndpointParams.DeviceAddress = EndpointProperties->DeviceAddress;
//     QH->sqh.HwQH.EndpointParams.EndpointNumber = EndpointProperties->EndpointAddress;
//
//     DeviceSpeed = EndpointProperties->DeviceSpeed;
//
//     switch (DeviceSpeed)
//     {
//         case UsbLowSpeed:
//             QH->sqh.HwQH.EndpointParams.EndpointSpeed = EHCI_QH_EP_LOW_SPEED;
//             break;
//
//         case UsbFullSpeed:
//             QH->sqh.HwQH.EndpointParams.EndpointSpeed = EHCI_QH_EP_FULL_SPEED;
//             break;
//
//         case UsbHighSpeed:
//             QH->sqh.HwQH.EndpointParams.EndpointSpeed = EHCI_QH_EP_HIGH_SPEED;
//             break;
//
//         default:
//             DPRINT1("EHCI_InitializeQH: Unknown DeviceSpeed - %x\n", DeviceSpeed);
//             ASSERT(FALSE);
//             break;
//     }
//
//     QH->sqh.HwQH.EndpointParams.MaximumPacketLength = EndpointProperties->MaxPacketSize;
//     QH->sqh.HwQH.EndpointCaps.PipeMultiplier = 1;
//
//     if (DeviceSpeed == UsbHighSpeed)
//     {
//         QH->sqh.HwQH.EndpointCaps.HubAddr = 0;
//         QH->sqh.HwQH.EndpointCaps.PortNumber = 0;
//     }
//     else
//     {
//         QH->sqh.HwQH.EndpointCaps.HubAddr = EndpointProperties->HubAddr;
//         QH->sqh.HwQH.EndpointCaps.PortNumber = EndpointProperties->PortNumber;
//
//         if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//             QH->sqh.HwQH.EndpointParams.ControlEndpointFlag = 1;
//     }
//
//     QH->sqh.HwQH.NextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//
//     QH->sqh.HwQH.Token.Status &= (UCHAR)~(EHCI_TOKEN_STATUS_ACTIVE |
//                                           EHCI_TOKEN_STATUS_HALTED);
//
//     return QH;
// }
//
// MPSTATUS
// NTAPI
// EHCI_OpenBulkOrControlEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                                IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                                IN PEHCI_ENDPOINT EhciEndpoint,
//                                IN BOOLEAN IsControl)
// {
//     PEHCI_HCD_QH QH;
//     ULONG QhPA;
//     PEHCI_HCD_TD TdVA;
//     ULONG TdPA;
//     PEHCI_HCD_TD TD;
//     ULONG TdCount;
//     ULONG ix;
//
//     DPRINT("EHCI_OpenBulkOrControlEndpoint: EhciEndpoint - %p, IsControl - %x\n",
//            EhciEndpoint,
//            IsControl);
//
//     InitializeListHead(&EhciEndpoint->ListTDs);
//
//     EhciEndpoint->DmaBufferVA = (PVOID)EndpointProperties->BufferVA;
//     EhciEndpoint->DmaBufferPA = EndpointProperties->BufferPA;
//
//     RtlZeroMemory(EhciEndpoint->DmaBufferVA, sizeof(EHCI_HCD_TD));
//
//     QH = (PEHCI_HCD_QH)EhciEndpoint->DmaBufferVA + 1;
//     QhPA = EhciEndpoint->DmaBufferPA + sizeof(EHCI_HCD_TD);
//
//     EhciEndpoint->FirstTD = (PEHCI_HCD_TD)(QH + 1);
//
//     TdCount = (EndpointProperties->BufferLength -
//                (sizeof(EHCI_HCD_TD) + sizeof(EHCI_HCD_QH))) /
//                sizeof(EHCI_HCD_TD);
//
//     if (EndpointProperties->TransferType == USBPORT_TRANSFER_TYPE_CONTROL)
//         EhciEndpoint->EndpointStatus |= USBPORT_ENDPOINT_CONTROL;
//
//     EhciEndpoint->MaxTDs = TdCount;
//     EhciEndpoint->RemainTDs = TdCount;
//
//     TdVA = EhciEndpoint->FirstTD;
//     TdPA = QhPA + sizeof(EHCI_HCD_QH);
//
//     for (ix = 0; ix < TdCount; ix++)
//     {
//         DPRINT_EHCI!("EHCI_OpenBulkOrControlEndpoint: TdVA - %p, TdPA - %p\n",
//                     TdVA,
//                     TdPA);
//
//         RtlZeroMemory(TdVA, sizeof(EHCI_HCD_TD));
//
//         ASSERT((TdPA & ~LINK_POINTER_MASK) == 0);
//
//         TdVA->PhysicalAddress = TdPA;
//         TdVA->EhciEndpoint = EhciEndpoint;
//         TdVA->EhciTransfer = NULL;
//
//         TdPA += sizeof(EHCI_HCD_TD);
//         TdVA += 1;
//     }
//
//     EhciEndpoint->QH = EHCI_InitializeQH(EhciExtension,
//                                          EhciEndpoint,
//                                          QH,
//                                          QhPA);
//
//     if (IsControl)
//     {
//         QH->sqh.HwQH.EndpointParams.DataToggleControl = 1;
//         EhciEndpoint->HcdHeadP = NULL;
//     }
//     else
//     {
//         QH->sqh.HwQH.EndpointParams.DataToggleControl = 0;
//     }
//
//     TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//     if (!TD)
//         return MP_STATUS_NO_RESOURCES;
//
//     TD->TdFlags |= EHCI_HCD_TD_FLAG_DUMMY;
//     TD->HwTD.Token.Status &= (UCHAR)~EHCI_TOKEN_STATUS_ACTIVE;
//
//     TD->HwTD.NextTD = TERMINATE_POINTER;
//     TD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//     TD->NextHcdTD = NULL;
//     TD->AltNextHcdTD = NULL;
//
//     EhciEndpoint->HcdTailP = TD;
//     EhciEndpoint->HcdHeadP = TD;
//
//     QH->sqh.HwQH.CurrentTD = TD->PhysicalAddress;
//     QH->sqh.HwQH.NextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//
//     QH->sqh.HwQH.Token.Status &= (UCHAR)~EHCI_TOKEN_STATUS_ACTIVE;
//     QH->sqh.HwQH.Token.TransferBytes = 0;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_OpenInterruptEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                            IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                            IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_QH QH;
//     ULONG QhPA;
//     PEHCI_HCD_TD FirstTD;
//     ULONG FirstTdPA;
//     PEHCI_HCD_TD TD;
//     PEHCI_HCD_TD DummyTD;
//     ULONG TdCount;
//     ULONG ix;
//     const EHCI_PERIOD * PeriodTable = NULL;
//     ULONG ScheduleOffset;
//     ULONG Idx = 0;
//     UCHAR Period;
//
//     DPRINT("EHCI_OpenInterruptEndpoint: EhciExtension - %p, EndpointProperties - %p, EhciEndpoint - %p\n",
//            EhciExtension,
//            EndpointProperties,
//            EhciEndpoint);
//
//     Period = EndpointProperties->Period;
//     ScheduleOffset = EndpointProperties->ScheduleOffset;
//
//     ASSERT(Period < (INTERRUPT_ENDPOINTs + 1));
//
//     while (!(Period & 1))
//     {
//         Idx++;
//         Period >>= 1;
//     }
//
//     ASSERT(Idx < 8);
//
//     InitializeListHead(&EhciEndpoint->ListTDs);
//
//     if (EhciEndpoint->EndpointProperties.DeviceSpeed == UsbHighSpeed)
//     {
//         PeriodTable = &pTable[ClassicPeriod[Idx] + ScheduleOffset];
//         EhciEndpoint->PeriodTable = PeriodTable;
//
//         DPRINT("EHCI_OpenInterruptEndpoint: EhciEndpoint - %p, ScheduleMask - %X, Index - %X\n",
//                EhciEndpoint,
//                PeriodTable->ScheduleMask,
//                ClassicPeriod[Idx]);
//
//         EhciEndpoint->StaticQH = EhciExtension->PeriodicHead[PeriodTable->PeriodIdx];
//     }
//     else
//     {
//         EhciEndpoint->PeriodTable = NULL;
//
//         DPRINT("EHCI_OpenInterruptEndpoint: EhciEndpoint - %p, Index - %X\n",
//                EhciEndpoint,
//                ClassicPeriod[Idx]);
//
//         EhciEndpoint->StaticQH = EhciExtension->PeriodicHead[ClassicPeriod[Idx] +
//                                                              ScheduleOffset];
//     }
//
//     EhciEndpoint->DmaBufferVA = (PVOID)EndpointProperties->BufferVA;
//     EhciEndpoint->DmaBufferPA = EndpointProperties->BufferPA;
//
//     RtlZeroMemory((PVOID)EndpointProperties->BufferVA, sizeof(EHCI_HCD_TD));
//
//     QH = (PEHCI_HCD_QH)(EndpointProperties->BufferVA + sizeof(EHCI_HCD_TD));
//     QhPA = EndpointProperties->BufferPA + sizeof(EHCI_HCD_TD);
//
//     FirstTD = (PEHCI_HCD_TD)(EndpointProperties->BufferVA +
//                              sizeof(EHCI_HCD_TD) +
//                              sizeof(EHCI_HCD_QH));
//
//     FirstTdPA = EndpointProperties->BufferPA +
//                 sizeof(EHCI_HCD_TD) +
//                 sizeof(EHCI_HCD_QH);
//
//     TdCount = (EndpointProperties->BufferLength -
//                (sizeof(EHCI_HCD_TD) + sizeof(EHCI_HCD_QH))) /
//                sizeof(EHCI_HCD_TD);
//
//     ASSERT(TdCount >= EHCI_MAX_INTERRUPT_TD_COUNT + 1);
//
//     EhciEndpoint->FirstTD = FirstTD;
//     EhciEndpoint->MaxTDs = TdCount;
//
//     for (ix = 0; ix < TdCount; ix++)
//     {
//         TD = EhciEndpoint->FirstTD + ix;
//
//         RtlZeroMemory(TD, sizeof(EHCI_HCD_TD));
//
//         ASSERT((FirstTdPA & ~LINK_POINTER_MASK) == 0);
//
//         TD->PhysicalAddress = FirstTdPA;
//         TD->EhciEndpoint = EhciEndpoint;
//         TD->EhciTransfer = NULL;
//
//         FirstTdPA += sizeof(EHCI_HCD_TD);
//     }
//
//     EhciEndpoint->RemainTDs = TdCount;
//
//     EhciEndpoint->QH = EHCI_InitializeQH(EhciExtension,
//                                          EhciEndpoint,
//                                          QH,
//                                          QhPA);
//
//     if (EhciEndpoint->EndpointProperties.DeviceSpeed == UsbHighSpeed)
//     {
//         QH->sqh.HwQH.EndpointCaps.InterruptMask = PeriodTable->ScheduleMask;
//     }
//     else
//     {
//         QH->sqh.HwQH.EndpointCaps.InterruptMask =
//         EndpointProperties->InterruptScheduleMask;
//
//         QH->sqh.HwQH.EndpointCaps.SplitCompletionMask =
//         EndpointProperties->SplitCompletionMask;
//     }
//
//     DummyTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//     DummyTD->TdFlags |= EHCI_HCD_TD_FLAG_DUMMY;
//     DummyTD->NextHcdTD = NULL;
//     DummyTD->AltNextHcdTD = NULL;
//
//     DummyTD->HwTD.Token.Status &= ~EHCI_TOKEN_STATUS_ACTIVE;
//
//     DummyTD->HwTD.NextTD = TERMINATE_POINTER;
//     DummyTD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//     EhciEndpoint->HcdTailP = DummyTD;
//     EhciEndpoint->HcdHeadP = DummyTD;
//
//     QH->sqh.HwQH.CurrentTD = DummyTD->PhysicalAddress;
//     QH->sqh.HwQH.NextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//
//     QH->sqh.HwQH.Token.Status &= ~EHCI_TOKEN_STATUS_ACTIVE;
//     QH->sqh.HwQH.Token.TransferBytes = 0;
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_OpenHsIsoEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                        IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                        IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     DPRINT1("EHCI_OpenHsIsoEndpoint: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_NOT_SUPPORTED;
// }
//
// MPSTATUS
// NTAPI
// EHCI_OpenIsoEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                      IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                      IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     DPRINT1("EHCI_OpenIsoEndpoint: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_NOT_SUPPORTED;
// }
//
// MPSTATUS
// NTAPI
// EHCI_OpenEndpoint(IN PVOID ehciExtension,
//                   IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                   IN PVOID ehciEndpoint)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_ENDPOINT EhciEndpoint = ehciEndpoint;
//     ULONG TransferType;
//     MPSTATUS MPStatus;
//
//     DPRINT("EHCI_OpenEndpoint: ... \n");
//
//     RtlCopyMemory(&EhciEndpoint->EndpointProperties,
//                   EndpointProperties,
//                   sizeof(EhciEndpoint->EndpointProperties));
//
//     TransferType = EndpointProperties->TransferType;
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             if (EndpointProperties->DeviceSpeed == UsbHighSpeed)
//             {
//                 MPStatus = EHCI_OpenHsIsoEndpoint(EhciExtension,
//                                                   EndpointProperties,
//                                                   EhciEndpoint);
//             }
//             else
//             {
//                 MPStatus = EHCI_OpenIsoEndpoint(EhciExtension,
//                                                 EndpointProperties,
//                                                 EhciEndpoint);
//             }
//
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             MPStatus = EHCI_OpenBulkOrControlEndpoint(EhciExtension,
//                                                       EndpointProperties,
//                                                       EhciEndpoint,
//                                                       TRUE);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             MPStatus = EHCI_OpenBulkOrControlEndpoint(EhciExtension,
//                                                       EndpointProperties,
//                                                       EhciEndpoint,
//                                                       FALSE);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             MPStatus = EHCI_OpenInterruptEndpoint(EhciExtension,
//                                                   EndpointProperties,
//                                                   EhciEndpoint);
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
// EHCI_ReopenEndpoint(IN PVOID ehciExtension,
//                     IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                     IN PVOID ehciEndpoint)
// {
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG TransferType;
//     PEHCI_HCD_QH QH;
//     MPSTATUS MPStatus;
//
//     EhciEndpoint = ehciEndpoint;
//
//     TransferType = EndpointProperties->TransferType;
//
//     DPRINT("EHCI_ReopenEndpoint: EhciEndpoint - %p, TransferType - %x\n",
//            EhciEndpoint,
//            TransferType);
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             if (EndpointProperties->DeviceSpeed == UsbHighSpeed)
//             {
//                 DPRINT1("EHCI_ReopenEndpoint: HS Iso. UNIMPLEMENTED. FIXME\n");
//                 MPStatus = MP_STATUS_NOT_SUPPORTED;
//             }
//             else
//             {
//                 DPRINT1("EHCI_ReopenEndpoint: Iso. UNIMPLEMENTED. FIXME\n");
//                 MPStatus = MP_STATUS_NOT_SUPPORTED;
//             }
//
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//         case USBPORT_TRANSFER_TYPE_BULK:
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             RtlCopyMemory(&EhciEndpoint->EndpointProperties,
//                           EndpointProperties,
//                           sizeof(EhciEndpoint->EndpointProperties));
//
//             QH = EhciEndpoint->QH;
//
//             QH->sqh.HwQH.EndpointParams.DeviceAddress = EndpointProperties->DeviceAddress;
//             QH->sqh.HwQH.EndpointParams.MaximumPacketLength = EndpointProperties->MaxPacketSize;
//
//             QH->sqh.HwQH.EndpointCaps.HubAddr = EndpointProperties->HubAddr;
//
//             MPStatus = MP_STATUS_SUCCESS;
//             break;
//
//         default:
//             DPRINT1("EHCI_ReopenEndpoint: Unknown TransferType\n");
//             MPStatus = MP_STATUS_SUCCESS;
//             break;
//     }
//
//     return MPStatus;
// }
//
// VOID
// NTAPI
// EHCI_QueryEndpointRequirements(IN PVOID ehciExtension,
//                                IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                                IN PUSBPORT_ENDPOINT_REQUIREMENTS EndpointRequirements)
// {
//     ULONG TransferType;
//
//     DPRINT("EHCI_QueryEndpointRequirements: ... \n");
//
//     TransferType = EndpointProperties->TransferType;
//
//     switch (TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_ISOCHRONOUS:
//             DPRINT("EHCI_QueryEndpointRequirements: IsoTransfer\n");
//
//             if (EndpointProperties->DeviceSpeed == UsbHighSpeed)
//             {
//                 EndpointRequirements->HeaderBufferSize = EHCI_MAX_HS_ISO_HEADER_BUFFER_SIZE;
//                 EndpointRequirements->MaxTransferSize = EHCI_MAX_HS_ISO_TRANSFER_SIZE;
//             }
//             else
//             {
//                 EndpointRequirements->HeaderBufferSize = EHCI_MAX_FS_ISO_HEADER_BUFFER_SIZE;
//                 EndpointRequirements->MaxTransferSize = EHCI_MAX_FS_ISO_TRANSFER_SIZE;
//             }
//             break;
//
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             DPRINT("EHCI_QueryEndpointRequirements: ControlTransfer\n");
//             EndpointRequirements->HeaderBufferSize = sizeof(EHCI_HCD_TD) +
//                                                      sizeof(EHCI_HCD_QH) +
//                                                      EHCI_MAX_CONTROL_TD_COUNT * sizeof(EHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = EHCI_MAX_CONTROL_TRANSFER_SIZE;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             DPRINT("EHCI_QueryEndpointRequirements: BulkTransfer\n");
//             EndpointRequirements->HeaderBufferSize = sizeof(EHCI_HCD_TD) +
//                                                      sizeof(EHCI_HCD_QH) +
//                                                      EHCI_MAX_BULK_TD_COUNT * sizeof(EHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = EHCI_MAX_BULK_TRANSFER_SIZE;
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             DPRINT("EHCI_QueryEndpointRequirements: InterruptTransfer\n");
//             EndpointRequirements->HeaderBufferSize = sizeof(EHCI_HCD_TD) +
//                                                      sizeof(EHCI_HCD_QH) +
//                                                      EHCI_MAX_INTERRUPT_TD_COUNT * sizeof(EHCI_HCD_TD);
//
//             EndpointRequirements->MaxTransferSize = EHCI_MAX_INTERRUPT_TRANSFER_SIZE;
//             break;
//
//         default:
//             DPRINT1("EHCI_QueryEndpointRequirements: Unknown TransferType - %x\n",
//                     TransferType);
//             DbgBreakPoint();
//             break;
//     }
// }
//
// VOID
// NTAPI
// EHCI_DisablePeriodicList(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//
//     DPRINT("EHCI_DisablePeriodicList: ... \n");
//
//     if (EhciExtension->Flags & EHCI_FLAGS_IDLE_SUPPORT)
//     {
//         OperationalRegs = EhciExtension->OperationalRegs;
//
//         Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//         Command.PeriodicEnable = 0;
//         WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//     }
// }
//
// VOID
// NTAPI
// EHCI_CloseEndpoint(IN PVOID ehciExtension,
//                    IN PVOID ehciEndpoint,
//                    IN BOOLEAN DisablePeriodic)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_ENDPOINT EhciEndpoint = ehciEndpoint;
//     ULONG TransferType;
//
//     DPRINT1("EHCI_CloseEndpoint: EhciEndpoint - %p, DisablePeriodic - %X\n",
//             EhciEndpoint,
//             DisablePeriodic);
//
//     if (DisablePeriodic)
//     {
//         TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS ||
//             TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//         {
//             EHCI_DisablePeriodicList(EhciExtension);
//         }
//     }
// }
//
// PEHCI_STATIC_QH
// NTAPI
// EHCI_GetQhForFrame(IN PEHCI_EXTENSION EhciExtension,
//                    IN ULONG FrameIdx)
// {
//     //DPRINT_EHCI!("EHCI_GetQhForFrame: FrameIdx - %x, Balance[FrameIdx] - %x\n",
//     //            FrameIdx,
//     //            Balance[FrameIdx & 0x1F]);
//
//     return EhciExtension->PeriodicHead[Balance[FrameIdx & (EHCI_FRAMES - 1)]];
// }
//
// PEHCI_HCD_QH
// NTAPI
// EHCI_GetDummyQhForFrame(IN PEHCI_EXTENSION EhciExtension,
//                         IN ULONG Idx)
// {
//     return (PEHCI_HCD_QH)((ULONG_PTR)EhciExtension->IsoDummyQHListVA +
//                           Idx * sizeof(EHCI_HCD_QH));
// }
//
// VOID
// NTAPI
// EHCI_AlignHwStructure(IN PEHCI_EXTENSION EhciExtension,
//                       IN PULONG PhysicalAddress,
//                       IN PULONG_PTR VirtualAddress,
//                       IN ULONG Alignment)
// {
//     ULONG PAddress;
//     ULONG NewPAddress;
//     ULONG_PTR VAddress;
//
//     //DPRINT_EHCI!("EHCI_AlignHwStructure: *PhysicalAddress - %X, *VirtualAddress - %X, Alignment - %x\n",
//     //             *PhysicalAddress,
//     //             *VirtualAddress,
//     //             Alignment);
//
//     PAddress = *PhysicalAddress;
//     VAddress = *VirtualAddress;
//
//     NewPAddress = (ULONG)(ULONG_PTR)PAGE_ALIGN(*PhysicalAddress + Alignment - 1);
//
//     if (NewPAddress != (ULONG)(ULONG_PTR)PAGE_ALIGN(*PhysicalAddress))
//     {
//         VAddress += NewPAddress - PAddress;
//         PAddress = NewPAddress;
//
//         DPRINT("EHCI_AlignHwStructure: VAddress - %X, PAddress - %X\n",
//                VAddress,
//                PAddress);
//     }
//
//     *VirtualAddress = VAddress;
//     *PhysicalAddress = PAddress;
// }
//
// VOID
// NTAPI
// EHCI_AddDummyQHs(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HC_RESOURCES HcResourcesVA;
//     PEHCI_HCD_QH DummyQH;
//     ULONG DummyQhPA;
//     EHCI_QH_EP_PARAMS EndpointParams;
//     EHCI_LINK_POINTER PAddress;
//     ULONG Frame;
//
//     DPRINT("EHCI_AddDummyQueueHeads: EhciExtension - %p\n", EhciExtension);
//
//     HcResourcesVA = EhciExtension->HcResourcesVA;
//
//     DummyQH = EhciExtension->IsoDummyQHListVA;
//     DummyQhPA = EhciExtension->IsoDummyQHListPA;
//
//     for (Frame = 0; Frame < EHCI_FRAME_LIST_MAX_ENTRIES; Frame++)
//     {
//         RtlZeroMemory(DummyQH, sizeof(EHCI_HCD_QH));
//
//         PAddress.AsULONG = HcResourcesVA->PeriodicFrameList[Frame];
//
//         DummyQH->sqh.HwQH.HorizontalLink.AsULONG = PAddress.AsULONG;
//         DummyQH->sqh.HwQH.CurrentTD = 0;
//         DummyQH->sqh.HwQH.NextTD = TERMINATE_POINTER;
//         DummyQH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//
//         EndpointParams = DummyQH->sqh.HwQH.EndpointParams;
//         EndpointParams.DeviceAddress = 0;
//         EndpointParams.EndpointSpeed = 0;
//         EndpointParams.MaximumPacketLength = EHCI_DUMMYQH_MAX_PACKET_LENGTH;
//         DummyQH->sqh.HwQH.EndpointParams = EndpointParams;
//
//         DummyQH->sqh.HwQH.EndpointCaps.AsULONG = 0;
//         DummyQH->sqh.HwQH.EndpointCaps.InterruptMask = 0;
//         DummyQH->sqh.HwQH.EndpointCaps.SplitCompletionMask = 0;
//         DummyQH->sqh.HwQH.EndpointCaps.PipeMultiplier = 1;
//
//         DummyQH->sqh.HwQH.Token.Status &= (UCHAR)~EHCI_TOKEN_STATUS_ACTIVE;
//
//         DummyQH->sqh.PhysicalAddress = DummyQhPA;
//         DummyQH->sqh.StaticQH = EHCI_GetQhForFrame(EhciExtension, Frame);
//
//         PAddress.AsULONG = DummyQhPA;
//         PAddress.Reserved = 0;
//         PAddress.Type = EHCI_LINK_TYPE_QH;
//
//         HcResourcesVA->PeriodicFrameList[Frame] = PAddress.AsULONG;
//
//         DummyQH++;
//         DummyQhPA += sizeof(EHCI_HCD_QH);
//     }
// }
//
// VOID
// NTAPI
// EHCI_InitializeInterruptSchedule(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_STATIC_QH StaticQH;
//     ULONG ix;
//
//     DPRINT("EHCI_InitializeInterruptSchedule: ... \n");
//
//     for (ix = 0; ix < INTERRUPT_ENDPOINTs; ix++)
//     {
//         StaticQH = EhciExtension->PeriodicHead[ix];
//
//         StaticQH->HwQH.EndpointParams.HeadReclamationListFlag = 0;
//         StaticQH->HwQH.NextTD |= TERMINATE_POINTER;
//         StaticQH->HwQH.Token.Status |= (UCHAR)EHCI_TOKEN_STATUS_HALTED;
//     }
//
//     for (ix = 1; ix < INTERRUPT_ENDPOINTs; ix++)
//     {
//         StaticQH = EhciExtension->PeriodicHead[ix];
//
//         StaticQH->PrevHead = NULL;
//         StaticQH->NextHead = (PEHCI_HCD_QH)EhciExtension->PeriodicHead[LinkTable[ix]];
//
//         StaticQH->HwQH.HorizontalLink.AsULONG =
//             EhciExtension->PeriodicHead[LinkTable[ix]]->PhysicalAddress;
//
//         StaticQH->HwQH.HorizontalLink.Type = EHCI_LINK_TYPE_QH;
//         StaticQH->HwQH.EndpointCaps.InterruptMask = 0xFF;
//
//         StaticQH->QhFlags |= EHCI_QH_FLAG_STATIC;
//
//         if (ix < (ENDPOINT_INTERRUPT_8ms - 1))
//             StaticQH->QhFlags |= EHCI_QH_FLAG_STATIC_FAST;
//     }
//
//     EhciExtension->PeriodicHead[0]->HwQH.HorizontalLink.Terminate = 1;
//
//     EhciExtension->PeriodicHead[0]->QhFlags |= (EHCI_QH_FLAG_STATIC |
//                                                 EHCI_QH_FLAG_STATIC_FAST);
// }
//
// MPSTATUS
// NTAPI
// EHCI_InitializeSchedule(IN PEHCI_EXTENSION EhciExtension,
//                         IN ULONG_PTR BaseVA,
//                         IN ULONG BasePA)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     PEHCI_HC_RESOURCES HcResourcesVA;
//     ULONG HcResourcesPA;
//     PEHCI_STATIC_QH AsyncHead;
//     ULONG AsyncHeadPA;
//     PEHCI_STATIC_QH PeriodicHead;
//     ULONG PeriodicHeadPA;
//     PEHCI_STATIC_QH StaticQH;
//     EHCI_LINK_POINTER NextLink;
//     EHCI_LINK_POINTER StaticHeadPA;
//     ULONG Frame;
//     ULONG ix;
//
//     DPRINT("EHCI_InitializeSchedule: BaseVA - %p, BasePA - %p\n",
//            BaseVA,
//            BasePA);
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     HcResourcesVA = (PEHCI_HC_RESOURCES)BaseVA;
//     HcResourcesPA = BasePA;
//
//     EhciExtension->HcResourcesVA = HcResourcesVA;
//     EhciExtension->HcResourcesPA = BasePA;
//
//     /* Asynchronous Schedule */
//
//     AsyncHead = &HcResourcesVA->AsyncHead;
//     AsyncHeadPA = HcResourcesPA + FIELD_OFFSET(EHCI_HC_RESOURCES, AsyncHead);
//
//     RtlZeroMemory(AsyncHead, sizeof(EHCI_STATIC_QH));
//
//     NextLink.AsULONG = AsyncHeadPA;
//     NextLink.Type = EHCI_LINK_TYPE_QH;
//
//     AsyncHead->HwQH.HorizontalLink = NextLink;
//     AsyncHead->HwQH.EndpointParams.HeadReclamationListFlag = 1;
//     AsyncHead->HwQH.EndpointCaps.PipeMultiplier = 1;
//     AsyncHead->HwQH.NextTD |= TERMINATE_POINTER;
//     AsyncHead->HwQH.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_HALTED;
//
//     AsyncHead->PhysicalAddress = AsyncHeadPA;
//     AsyncHead->PrevHead = AsyncHead->NextHead = (PEHCI_HCD_QH)AsyncHead;
//
//     EhciExtension->AsyncHead = AsyncHead;
//
//     /* Periodic Schedule */
//
//     PeriodicHead = &HcResourcesVA->PeriodicHead[0];
//     PeriodicHeadPA = HcResourcesPA + FIELD_OFFSET(EHCI_HC_RESOURCES, PeriodicHead[0]);
//
//     for (ix = 0; ix < (INTERRUPT_ENDPOINTs + 1); ix++)
//     {
//         EHCI_AlignHwStructure(EhciExtension,
//                               &PeriodicHeadPA,
//                               (PULONG_PTR)&PeriodicHead,
//                               80);
//
//         EhciExtension->PeriodicHead[ix] = PeriodicHead;
//         EhciExtension->PeriodicHead[ix]->PhysicalAddress = PeriodicHeadPA;
//
//         PeriodicHead += 1;
//         PeriodicHeadPA += sizeof(EHCI_STATIC_QH);
//     }
//
//     EHCI_InitializeInterruptSchedule(EhciExtension);
//
//     for (Frame = 0; Frame < EHCI_FRAME_LIST_MAX_ENTRIES; Frame++)
//     {
//         StaticQH = EHCI_GetQhForFrame(EhciExtension, Frame);
//
//         StaticHeadPA.AsULONG = StaticQH->PhysicalAddress;
//         StaticHeadPA.Type = EHCI_LINK_TYPE_QH;
//
//         //DPRINT_EHCI!("EHCI_InitializeSchedule: StaticHeadPA[%x] - %X\n",
//         //            Frame,
//         //            StaticHeadPA);
//
//         HcResourcesVA->PeriodicFrameList[Frame] = StaticHeadPA.AsULONG;
//     }
//
//     EhciExtension->IsoDummyQHListVA = &HcResourcesVA->IsoDummyQH[0];
//     EhciExtension->IsoDummyQHListPA = HcResourcesPA + FIELD_OFFSET(EHCI_HC_RESOURCES, IsoDummyQH[0]);
//
//     EHCI_AddDummyQHs(EhciExtension);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->PeriodicListBase,
//                          EhciExtension->HcResourcesPA + FIELD_OFFSET(EHCI_HC_RESOURCES, PeriodicFrameList));
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->AsyncListBase,
//                          NextLink.AsULONG);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_InitializeHardware(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HC_CAPABILITY_REGISTERS CapabilityRegisters;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//     LARGE_INTEGER EndTime;
//     LARGE_INTEGER CurrentTime;
//     EHCI_HC_STRUCTURAL_PARAMS StructuralParams;
//
//     DPRINT("EHCI_InitializeHardware: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//     CapabilityRegisters = EhciExtension->CapabilityRegisters;
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.Reset = 1;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     KeQuerySystemTime(&EndTime);
//     EndTime.QuadPart += 100 * 10000; // 100 msec
//
//     DPRINT("EHCI_InitializeHardware: Start reset ... \n");
//
//     while (TRUE)
//     {
//         KeQuerySystemTime(&CurrentTime);
//         Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//
//         if (Command.Reset != 1)
//             break;
//
//         if (CurrentTime.QuadPart >= EndTime.QuadPart)
//         {
//             if (Command.Reset == 1)
//             {
//                 DPRINT1("EHCI_InitializeHardware: Reset failed!\n");
//                 return MP_STATUS_HW_ERROR;
//             }
//
//             break;
//         }
//     }
//
//     DPRINT("EHCI_InitializeHardware: Reset - OK\n");
//
//     StructuralParams.AsULONG = READ_REGISTER_ULONG(&CapabilityRegisters->StructParameters.AsULONG);
//
//     EhciExtension->NumberOfPorts = StructuralParams.PortCount;
//     EhciExtension->PortPowerControl = StructuralParams.PortPowerControl;
//
//     DPRINT("EHCI_InitializeHardware: StructuralParams - %X\n", StructuralParams.AsULONG);
//     DPRINT("EHCI_InitializeHardware: PortPowerControl - %x\n", EhciExtension->PortPowerControl);
//     DPRINT("EHCI_InitializeHardware: N_PORTS          - %x\n", EhciExtension->NumberOfPorts);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->PeriodicListBase, 0);
//     WRITE_REGISTER_ULONG(&OperationalRegs->AsyncListBase, 0);
//
//     EhciExtension->InterruptMask.AsULONG = 0;
//     EhciExtension->InterruptMask.Interrupt = 1;
//     EhciExtension->InterruptMask.ErrorInterrupt = 1;
//     EhciExtension->InterruptMask.PortChangeInterrupt = 0;
//     EhciExtension->InterruptMask.FrameListRollover = 1;
//     EhciExtension->InterruptMask.HostSystemError = 1;
//     EhciExtension->InterruptMask.InterruptOnAsyncAdvance = 1;
//
//     return MP_STATUS_SUCCESS;
// }
//
// UCHAR
// NTAPI
// EHCI_GetOffsetEECP(IN PEHCI_EXTENSION EhciExtension,
//                    IN UCHAR CapabilityID)
// {
//     EHCI_LEGACY_EXTENDED_CAPABILITY LegacyCapability;
//     EHCI_HC_CAPABILITY_PARAMS CapParameters;
//     UCHAR OffsetEECP;
//
//     DPRINT("EHCI_GetOffsetEECP: CapabilityID - %x\n", CapabilityID);
//
//     CapParameters = EhciExtension->CapabilityRegisters->CapParameters;
//
//     OffsetEECP = CapParameters.ExtCapabilitiesPointer;
//
//     if (!OffsetEECP)
//         return 0;
//
//     while (TRUE)
//     {
//         RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                               TRUE,
//                                               &LegacyCapability.AsULONG,
//                                               OffsetEECP,
//                                               sizeof(LegacyCapability));
//
//         DPRINT("EHCI_GetOffsetEECP: OffsetEECP - %x\n", OffsetEECP);
//
//         if (LegacyCapability.CapabilityID == CapabilityID)
//             break;
//
//         OffsetEECP = LegacyCapability.NextCapabilityPointer;
//
//         if (!OffsetEECP)
//             return 0;
//     }
//
//     return OffsetEECP;
// }
//
// MPSTATUS
// NTAPI
// EHCI_TakeControlHC(IN PEHCI_EXTENSION EhciExtension)
// {
//     LARGE_INTEGER EndTime;
//     LARGE_INTEGER CurrentTime;
//     EHCI_LEGACY_EXTENDED_CAPABILITY LegacyCapability;
//     UCHAR OffsetEECP;
//
//     DPRINT("EHCI_TakeControlHC: EhciExtension - %p\n", EhciExtension);
//
//     OffsetEECP = EHCI_GetOffsetEECP(EhciExtension, 1);
//
//     if (OffsetEECP == 0)
//         return MP_STATUS_SUCCESS;
//
//     DPRINT("EHCI_TakeControlHC: OffsetEECP - %X\n", OffsetEECP);
//
//     RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                           TRUE,
//                                           &LegacyCapability.AsULONG,
//                                           OffsetEECP,
//                                           sizeof(LegacyCapability));
//
//     if (LegacyCapability.BiosOwnedSemaphore == 0)
//         return MP_STATUS_SUCCESS;
//
//     LegacyCapability.OsOwnedSemaphore = 1;
//
//     RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                           FALSE,
//                                           &LegacyCapability.AsULONG,
//                                           OffsetEECP,
//                                           sizeof(LegacyCapability));
//
//     KeQuerySystemTime(&EndTime);
//     EndTime.QuadPart += 100 * 10000;
//
//     do
//     {
//         RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                               TRUE,
//                                               &LegacyCapability.AsULONG,
//                                               OffsetEECP,
//                                               sizeof(LegacyCapability));
//         KeQuerySystemTime(&CurrentTime);
//
//         if (LegacyCapability.BiosOwnedSemaphore)
//         {
//             DPRINT("EHCI_TakeControlHC: Ownership is ok\n");
//             break;
//         }
//     }
//     while (CurrentTime.QuadPart <= EndTime.QuadPart);
//
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_GetRegistryParameters(IN PEHCI_EXTENSION EhciExtension)
// {
//     DPRINT1("EHCI_GetRegistryParameters: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// EHCI_StartController(IN PVOID ehciExtension,
//                      IN PUSBPORT_RESOURCES Resources)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HC_CAPABILITY_REGISTERS CapabilityRegisters;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     MPSTATUS MPStatus;
//     EHCI_USB_COMMAND Command;
//     UCHAR CapabilityRegLength;
//     UCHAR Fladj;
//
//     DPRINT("EHCI_StartController: ... \n");
//
//     if ((Resources->ResourcesTypes & (USBPORT_RESOURCES_MEMORY | USBPORT_RESOURCES_INTERRUPT)) !=
//                                      (USBPORT_RESOURCES_MEMORY | USBPORT_RESOURCES_INTERRUPT))
//     {
//         DPRINT1("EHCI_StartController: Resources->ResourcesTypes - %x\n",
//                 Resources->ResourcesTypes);
//
//         return MP_STATUS_ERROR;
//     }
//
//     CapabilityRegisters = (PEHCI_HC_CAPABILITY_REGISTERS)Resources->ResourceBase;
//     EhciExtension->CapabilityRegisters = CapabilityRegisters;
//
//     CapabilityRegLength = READ_REGISTER_UCHAR(&CapabilityRegisters->RegistersLength);
//
//     OperationalRegs = (PEHCI_HW_REGISTERS)((ULONG_PTR)CapabilityRegisters +
//                                                       CapabilityRegLength);
//
//     EhciExtension->OperationalRegs = OperationalRegs;
//
//     DPRINT("EHCI_StartController: CapabilityRegisters - %p\n", CapabilityRegisters);
//     DPRINT("EHCI_StartController: OperationalRegs     - %p\n", OperationalRegs);
//
//     RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                           TRUE,
//                                           &Fladj,
//                                           EHCI_FLADJ_PCI_CONFIG_OFFSET,
//                                           sizeof(Fladj));
//
//     EhciExtension->FrameLengthAdjustment = Fladj;
//
//     EHCI_GetRegistryParameters(EhciExtension);
//
//     MPStatus = EHCI_TakeControlHC(EhciExtension);
//
//     if (MPStatus)
//     {
//         DPRINT1("EHCI_StartController: Unsuccessful TakeControlHC()\n");
//         return MPStatus;
//     }
//
//     MPStatus = EHCI_InitializeHardware(EhciExtension);
//
//     if (MPStatus)
//     {
//         DPRINT1("EHCI_StartController: Unsuccessful InitializeHardware()\n");
//         return MPStatus;
//     }
//
//     MPStatus = EHCI_InitializeSchedule(EhciExtension,
//                                        Resources->StartVA,
//                                        Resources->StartPA);
//
//     if (MPStatus)
//     {
//         DPRINT1("EHCI_StartController: Unsuccessful InitializeSchedule()\n");
//         return MPStatus;
//     }
//
//     RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                           TRUE,
//                                           &Fladj,
//                                           EHCI_FLADJ_PCI_CONFIG_OFFSET,
//                                           sizeof(Fladj));
//
//     if (Fladj != EhciExtension->FrameLengthAdjustment)
//     {
//         Fladj = EhciExtension->FrameLengthAdjustment;
//
//         RegPacket.UsbPortReadWriteConfigSpace(EhciExtension,
//                                               FALSE, // write
//                                               &Fladj,
//                                               EHCI_FLADJ_PCI_CONFIG_OFFSET,
//                                               sizeof(Fladj));
//     }
//
//     /* Port routing control logic default-routes all ports to this HC */
//     EhciExtension->PortRoutingControl = EHCI_CONFIG_FLAG_CONFIGURED;
//     WRITE_REGISTER_ULONG(&OperationalRegs->ConfigFlag,
//                          EhciExtension->PortRoutingControl);
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.InterruptThreshold = 1; // one micro-frame
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.Run = 1; // execution of the schedule
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     EhciExtension->IsStarted = TRUE;
//
//     if (Resources->IsChirpHandled)
//     {
//         ULONG Port;
//
//         for (Port = 1; Port <= EhciExtension->NumberOfPorts; Port++)
//         {
//             EHCI_RH_SetFeaturePortPower(EhciExtension, Port);
//         }
//
//         RegPacket.UsbPortWait(EhciExtension, 200);
//
//         for (Port = 1; Port <= EhciExtension->NumberOfPorts; Port++)
//         {
//             EHCI_RH_ChirpRootPort(EhciExtension, Port++);
//         }
//     }
//
//     return MPStatus;
// }
//
// VOID
// NTAPI
// EHCI_StopController(IN PVOID ehciExtension,
//                     IN BOOLEAN DisableInterrupts)
// {
//     DPRINT1("EHCI_StopController: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_SuspendController(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//     EHCI_USB_STATUS Status;
//     EHCI_INTERRUPT_ENABLE IntrEn;
//     ULONG ix;
//
//     DPRINT("EHCI_SuspendController: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     EhciExtension->BackupPeriodiclistbase = READ_REGISTER_ULONG(&OperationalRegs->PeriodicListBase);
//     EhciExtension->BackupAsynclistaddr = READ_REGISTER_ULONG(&OperationalRegs->AsyncListBase);
//     EhciExtension->BackupCtrlDSSegment = READ_REGISTER_ULONG(&OperationalRegs->SegmentSelector);
//     EhciExtension->BackupUSBCmd = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.InterruptAdvanceDoorbell = 0;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.Run = 0;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     KeStallExecutionProcessor(125);
//
//     Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//
//     Status.HCHalted = 0;
//     Status.Reclamation = 0;
//     Status.PeriodicStatus = 0;
//     Status.AsynchronousStatus = 0;
//
//     if (Status.AsULONG)
//         WRITE_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG, Status.AsULONG);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG, 0);
//
//     for (ix = 0; ix < 10; ix++)
//     {
//         Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//
//         if (Status.HCHalted)
//             break;
//
//         RegPacket.UsbPortWait(EhciExtension, 1);
//     }
//
//     if (!Status.HCHalted)
//         DbgBreakPoint();
//
//     IntrEn.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG);
//     IntrEn.PortChangeInterrupt = 1;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG, IntrEn.AsULONG);
//
//     EhciExtension->Flags |= EHCI_FLAGS_CONTROLLER_SUSPEND;
// }
//
// MPSTATUS
// NTAPI
// EHCI_ResumeController(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     ULONG RoutingControl;
//     EHCI_USB_COMMAND Command;
//
//     DPRINT("EHCI_ResumeController: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     RoutingControl = EhciExtension->PortRoutingControl;
//
//     if (!(RoutingControl & EHCI_CONFIG_FLAG_CONFIGURED))
//     {
//         EhciExtension->PortRoutingControl = RoutingControl | EHCI_CONFIG_FLAG_CONFIGURED;
//         WRITE_REGISTER_ULONG(&OperationalRegs->ConfigFlag,
//                              EhciExtension->PortRoutingControl);
//
//         return MP_STATUS_HW_ERROR;
//     }
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->SegmentSelector,
//                          EhciExtension->BackupCtrlDSSegment);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->PeriodicListBase,
//                          EhciExtension->BackupPeriodiclistbase);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->AsyncListBase,
//                          EhciExtension->BackupAsynclistaddr);
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//
//     Command.AsULONG = Command.AsULONG ^ EhciExtension->BackupUSBCmd;
//
//     Command.Reset = 0;
//     Command.FrameListSize = 0;
//     Command.InterruptAdvanceDoorbell = 0;
//     Command.LightResetHC = 0;
//     Command.AsynchronousParkModeCount = 0;
//     Command.AsynchronousParkModeEnable = 0;
//
//     Command.Run = 1;
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG,
//                          Command.AsULONG);
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG,
//                          EhciExtension->InterruptMask.AsULONG);
//
//     EhciExtension->Flags &= ~EHCI_FLAGS_CONTROLLER_SUSPEND;
//
//     return MP_STATUS_SUCCESS;
// }
//
// BOOLEAN
// NTAPI
// EHCI_HardwarePresent(IN PEHCI_EXTENSION EhciExtension,
//                      IN BOOLEAN IsInvalidateController)
// {
//     PEHCI_HW_REGISTERS OperationalRegs = EhciExtension->OperationalRegs;
//
//     if (READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG) != -1)
//         return TRUE;
//
//     DPRINT1("EHCI_HardwarePresent: IsInvalidateController - %x\n",
//             IsInvalidateController);
//
//     if (!IsInvalidateController)
//         return FALSE;
//
//     RegPacket.UsbPortInvalidateController(EhciExtension,
//                                           USBPORT_INVALIDATE_CONTROLLER_SURPRISE_REMOVE);
//     return FALSE;
// }
//
// BOOLEAN
// NTAPI
// EHCI_InterruptService(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     BOOLEAN Result = FALSE;
//     EHCI_USB_STATUS IntrSts;
//     EHCI_INTERRUPT_ENABLE IntrEn;
//     EHCI_INTERRUPT_ENABLE iStatus;
//     EHCI_USB_COMMAND Command;
//     ULONG FrameIndex;
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     DPRINT_EHCI!("EHCI_InterruptService: ... \n");
//
//     Result = EHCI_HardwarePresent(EhciExtension, FALSE);
//
//     if (!Result)
//         return FALSE;
//
//     IntrEn.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG);
//     IntrSts.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//
//     iStatus.AsULONG = (IntrEn.AsULONG & IntrSts.AsULONG) & EHCI_INTERRUPT_MASK;
//
//     if (!iStatus.AsULONG)
//         return FALSE;
//
//     EhciExtension->InterruptStatus = iStatus;
//
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG, iStatus.AsULONG);
//
//     if (iStatus.HostSystemError)
//     {
//         EhciExtension->HcSystemErrors++;
//
//         if (EhciExtension->HcSystemErrors < EHCI_MAX_HC_SYSTEM_ERRORS)
//         {
//             Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//             Command.Run = 1;
//             WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//         }
//     }
//
//     FrameIndex = READ_REGISTER_ULONG(&OperationalRegs->FrameIndex) / EHCI_MICROFRAMES;
//     FrameIndex &= EHCI_FRINDEX_FRAME_MASK;
//
//     if ((FrameIndex ^ EhciExtension->FrameIndex) & EHCI_FRAME_LIST_MAX_ENTRIES)
//     {
//         EhciExtension->FrameHighPart += 2 * EHCI_FRAME_LIST_MAX_ENTRIES;
//
//         EhciExtension->FrameHighPart -= (FrameIndex ^ EhciExtension->FrameHighPart) &
//                                         EHCI_FRAME_LIST_MAX_ENTRIES;
//     }
//
//     EhciExtension->FrameIndex = FrameIndex;
//
//     return TRUE;
// }
//
// VOID
// NTAPI
// EHCI_InterruptDpc(IN PVOID ehciExtension,
//                   IN BOOLEAN EnableInterrupts)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_INTERRUPT_ENABLE iStatus;
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     DPRINT_EHCI!("EHCI_InterruptDpc: [%p] EnableInterrupts - %x\n",
//                 EhciExtension, EnableInterrupts);
//
//     iStatus = EhciExtension->InterruptStatus;
//     EhciExtension->InterruptStatus.AsULONG = 0;
//
//     if (iStatus.Interrupt == 1 ||
//         iStatus.ErrorInterrupt == 1 ||
//         iStatus.InterruptOnAsyncAdvance == 1)
//     {
//         DPRINT_EHCI!("EHCI_InterruptDpc: [%p] InterruptStatus - %X\n", EhciExtension, iStatus.AsULONG);
//         RegPacket.UsbPortInvalidateEndpoint(EhciExtension, NULL);
//     }
//
//     if (iStatus.PortChangeInterrupt == 1)
//     {
//         DPRINT_EHCI!("EHCI_InterruptDpc: [%p] PortChangeInterrupt\n", EhciExtension);
//         RegPacket.UsbPortInvalidateRootHub(EhciExtension);
//     }
//
//     if (EnableInterrupts)
//     {
//         WRITE_REGISTER_ULONG(&OperationalRegs->HcInterruptEnable.AsULONG,
//                              EhciExtension->InterruptMask.AsULONG);
//     }
// }
//
// ULONG
// NTAPI
// EHCI_MapAsyncTransferToTd(IN PEHCI_EXTENSION EhciExtension,
//                           IN ULONG MaxPacketSize,
//                           IN ULONG TransferedLen,
//                           IN PULONG DataToggle,
//                           IN PEHCI_TRANSFER EhciTransfer,
//                           IN PEHCI_HCD_TD TD,
//                           IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PUSBPORT_TRANSFER_PARAMETERS TransferParameters;
//     PUSBPORT_SCATTER_GATHER_ELEMENT SgElement;
//     ULONG SgIdx;
//     ULONG LengthThisTD;
//     ULONG ix;
//     ULONG SgRemain;
//     ULONG DiffLength;
//     ULONG NumPackets;
//
//     DPRINT_EHCI!("EHCI_MapAsyncTransferToTd: EhciTransfer - %p, TD - %p, TransferedLen - %x, MaxPacketSize - %x, DataToggle - %x\n",
//                 EhciTransfer,
//                 TD,
//                 TransferedLen,
//                 MaxPacketSize,
//                 DataToggle);
//
//     TransferParameters = EhciTransfer->TransferParameters;
//
//     SgElement = &SgList->SgElement[0];
//
//     for (SgIdx = 0; SgIdx < SgList->SgElementCount; SgIdx++)
//     {
//         if (TransferedLen >= SgElement->SgOffset &&
//             TransferedLen < SgElement->SgOffset + SgElement->SgTransferLength)
//         {
//             break;
//         }
//
//         SgElement += 1;
//     }
//
//     SgRemain = SgList->SgElementCount - SgIdx;
//
//     if (SgRemain > EHCI_MAX_QTD_BUFFER_PAGES)
//     {
//         TD->HwTD.Buffer[0] = SgList->SgElement[SgIdx].SgPhysicalAddress.LowPart -
//                              SgList->SgElement[SgIdx].SgOffset +
//                              TransferedLen;
//
//         LengthThisTD = EHCI_MAX_QTD_BUFFER_PAGES * PAGE_SIZE as ULONG -
//                        (TD->HwTD.Buffer[0] & (PAGE_SIZE as ULONG - 1));
//
//         for (ix = 1; ix < EHCI_MAX_QTD_BUFFER_PAGES; ix++)
//         {
//             TD->HwTD.Buffer[ix] = SgList->SgElement[SgIdx + ix].SgPhysicalAddress.LowPart;
//         }
//
//         NumPackets = LengthThisTD / MaxPacketSize;
//         DiffLength = LengthThisTD - MaxPacketSize * (LengthThisTD / MaxPacketSize);
//
//         if (LengthThisTD != MaxPacketSize * (LengthThisTD / MaxPacketSize))
//             LengthThisTD -= DiffLength;
//
//         if (DataToggle && (NumPackets & 1))
//             *DataToggle = !(*DataToggle);
//     }
//     else
//     {
//         LengthThisTD = TransferParameters->TransferBufferLength - TransferedLen;
//
//         TD->HwTD.Buffer[0] = TransferedLen +
//                              SgList->SgElement[SgIdx].SgPhysicalAddress.LowPart -
//                              SgList->SgElement[SgIdx].SgOffset;
//
//         for (ix = 1; ix < EHCI_MAX_QTD_BUFFER_PAGES; ix++)
//         {
//             if ((SgIdx + ix) >= SgList->SgElementCount)
//                 break;
//
//             TD->HwTD.Buffer[ix] = SgList->SgElement[SgIdx + ix].SgPhysicalAddress.LowPart;
//         }
//     }
//
//     TD->HwTD.Token.TransferBytes = LengthThisTD;
//     TD->LengthThisTD = LengthThisTD;
//
//     return LengthThisTD + TransferedLen;
// }
//
// VOID
// NTAPI
// EHCI_EnableAsyncList(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND UsbCmd;
//
//     DPRINT_EHCI!("EHCI_EnableAsyncList: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     UsbCmd.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     UsbCmd.AsynchronousEnable = 1;
//     WRITE_REGISTER_ULONG((&OperationalRegs->HcCommand.AsULONG), UsbCmd.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_DisableAsyncList(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND UsbCmd;
//
//     DPRINT("EHCI_DisableAsyncList: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     UsbCmd.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     UsbCmd.AsynchronousEnable = 0;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, UsbCmd.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_EnablePeriodicList(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//
//     DPRINT("EHCI_EnablePeriodicList: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Command.PeriodicEnable = 1;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_FlushAsyncCache(IN PEHCI_EXTENSION EhciExtension)
// {
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//     EHCI_USB_STATUS Status;
//     LARGE_INTEGER CurrentTime;
//     LARGE_INTEGER EndTime;
//     EHCI_USB_COMMAND Cmd;
//
//     DPRINT_EHCI!("EHCI_FlushAsyncCache: EhciExtension - %p\n", EhciExtension);
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//     Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//     Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//
//     if (!Status.AsynchronousStatus && !Command.AsynchronousEnable)
//         return;
//
//     if (Status.AsynchronousStatus && !Command.AsynchronousEnable)
//     {
//         KeQuerySystemTime(&EndTime);
//         EndTime.QuadPart += 100 * 10000;  //100 ms
//
//         do
//         {
//             Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//             Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//             KeQuerySystemTime(&CurrentTime);
//
//             if (CurrentTime.QuadPart > EndTime.QuadPart)
//                 RegPacket.UsbPortBugCheck(EhciExtension);
//         }
//         while (Status.AsynchronousStatus && Command.AsULONG != -1 && Command.Run);
//
//         return;
//     }
//
//     if (!Status.AsynchronousStatus && Command.AsynchronousEnable)
//     {
//         KeQuerySystemTime(&EndTime);
//         EndTime.QuadPart += 100 * 10000;  //100 ms
//
//         do
//         {
//             Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//             Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//             KeQuerySystemTime(&CurrentTime);
//         }
//         while (!Status.AsynchronousStatus && Command.AsULONG != -1 && Command.Run);
//     }
//
//     Command.InterruptAdvanceDoorbell = 1;
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG, Command.AsULONG);
//
//     KeQuerySystemTime(&EndTime);
//     EndTime.QuadPart += 100 * 10000;  //100 ms
//
//     Cmd.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//
//     if (Cmd.InterruptAdvanceDoorbell)
//     {
//         while (Cmd.Run)
//         {
//             if (Cmd.AsULONG == -1)
//                 break;
//
//             KeStallExecutionProcessor(1);
//             Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//             KeQuerySystemTime(&CurrentTime);
//
//             if (!Command.InterruptAdvanceDoorbell)
//                 break;
//
//             Cmd = Command;
//         }
//     }
//
//     /* InterruptOnAsyncAdvance */
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG, 0x20);
// }
//
// VOID
// NTAPI
// EHCI_LockQH(IN PEHCI_EXTENSION EhciExtension,
//             IN PEHCI_HCD_QH QH,
//             IN ULONG TransferType)
// {
//     PEHCI_HCD_QH PrevQH;
//     PEHCI_HCD_QH NextQH;
//     ULONG QhPA;
//     ULONG FrameIndexReg;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//
//     DPRINT_EHCI!("EHCI_LockQH: QH - %p, TransferType - %x\n",
//                 QH,
//                 TransferType);
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     ASSERT((QH->sqh.QhFlags & EHCI_QH_FLAG_UPDATING) == 0);
//     ASSERT(EhciExtension->LockQH == NULL);
//
//     PrevQH = QH->sqh.PrevHead;
//     QH->sqh.QhFlags |= EHCI_QH_FLAG_UPDATING;
//
//     ASSERT(PrevQH);
//
//     NextQH = QH->sqh.NextHead;
//
//     EhciExtension->PrevQH = PrevQH;
//     EhciExtension->NextQH = NextQH;
//     EhciExtension->LockQH = QH;
//
//     if (NextQH)
//     {
//         QhPA = NextQH->sqh.PhysicalAddress;
//         QhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//         QhPA |= (EHCI_LINK_TYPE_QH << 1);
//     }
//     else
//     {
//         QhPA = TERMINATE_POINTER;
//     }
//
//     PrevQH->sqh.HwQH.HorizontalLink.AsULONG = QhPA;
//
//     FrameIndexReg = READ_REGISTER_ULONG(&OperationalRegs->FrameIndex);
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         do
//         {
//             Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//         }
//         while (READ_REGISTER_ULONG(&OperationalRegs->FrameIndex) ==
//                FrameIndexReg && (Command.AsULONG != -1) && Command.Run);
//     }
//     else
//     {
//         EHCI_FlushAsyncCache(EhciExtension);
//     }
// }
//
// VOID
// NTAPI
// EHCI_UnlockQH(IN PEHCI_EXTENSION EhciExtension,
//               IN PEHCI_HCD_QH QH)
// {
//     ULONG QhPA;
//
//     DPRINT_EHCI!("EHCI_UnlockQH: QH - %p\n", QH);
//
//     ASSERT(QH->sqh.QhFlags & EHCI_QH_FLAG_UPDATING);
//     ASSERT(EhciExtension->LockQH);
//     ASSERT(EhciExtension->LockQH == QH);
//
//     QH->sqh.QhFlags &= ~EHCI_QH_FLAG_UPDATING;
//
//     EhciExtension->LockQH = NULL;
//
//     QhPA = QH->sqh.PhysicalAddress;
//     QhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//     QhPA |= (EHCI_LINK_TYPE_QH << 1);
//
//     EhciExtension->PrevQH->sqh.HwQH.HorizontalLink.AsULONG = QhPA;
// }
//
// VOID
// NTAPI
// EHCI_LinkTransferToQueue(IN PEHCI_EXTENSION EhciExtension,
//                          IN PEHCI_ENDPOINT EhciEndpoint,
//                          IN PEHCI_HCD_TD NextTD)
// {
//     PEHCI_HCD_QH QH;
//     PEHCI_HCD_TD TD;
//     PEHCI_TRANSFER Transfer;
//     PEHCI_HCD_TD LinkTD;
//     BOOLEAN IsPresent;
//     ULONG ix;
//
//     DPRINT_EHCI!("EHCI_LinkTransferToQueue: EhciEndpoint - %p, NextTD - %p\n",
//                 EhciEndpoint,
//                 NextTD);
//
//     ASSERT(EhciEndpoint->HcdHeadP != NULL);
//     IsPresent = EHCI_HardwarePresent(EhciExtension, 0);
//
//     QH = EhciEndpoint->QH;
//     TD = EhciEndpoint->HcdHeadP;
//
//     if (TD == EhciEndpoint->HcdTailP)
//     {
//         if (IsPresent)
//         {
//             EHCI_LockQH(EhciExtension,
//                         QH,
//                         EhciEndpoint->EndpointProperties.TransferType);
//         }
//
//         QH->sqh.HwQH.CurrentTD = EhciEndpoint->DmaBufferPA;
//         QH->sqh.HwQH.NextTD = NextTD->PhysicalAddress;
//         QH->sqh.HwQH.AlternateNextTD = NextTD->HwTD.AlternateNextTD;
//
//         QH->sqh.HwQH.Token.Status = (UCHAR)~(EHCI_TOKEN_STATUS_ACTIVE |
//                                              EHCI_TOKEN_STATUS_HALTED);
//
//         QH->sqh.HwQH.Token.TransferBytes = 0;
//
//         if (IsPresent)
//             EHCI_UnlockQH(EhciExtension, QH);
//
//         EhciEndpoint->HcdHeadP = NextTD;
//     }
//     else
//     {
//         DPRINT_EHCI!("EHCI_LinkTransferToQueue: TD - %p, HcdTailP - %p\n",
//                     EhciEndpoint->HcdHeadP,
//                     EhciEndpoint->HcdTailP);
//
//         LinkTD = EhciEndpoint->HcdHeadP;
//
//         while (TD != EhciEndpoint->HcdTailP)
//         {
//             LinkTD = TD;
//             TD = TD->NextHcdTD;
//         }
//
//         ASSERT(LinkTD != EhciEndpoint->HcdTailP);
//
//         Transfer = LinkTD->EhciTransfer;
//
//         TD = EhciEndpoint->FirstTD;
//
//         for (ix = 0; ix < EhciEndpoint->MaxTDs; ix++)
//         {
//             if (TD->EhciTransfer == Transfer)
//             {
//                 TD->AltNextHcdTD = NextTD;
//                 TD->HwTD.AlternateNextTD = NextTD->PhysicalAddress;
//             }
//
//             TD += 1;
//         }
//
//         LinkTD->HwTD.NextTD = NextTD->PhysicalAddress;
//         LinkTD->NextHcdTD = NextTD;
//
//         if (QH->sqh.HwQH.CurrentTD == LinkTD->PhysicalAddress)
//         {
//             QH->sqh.HwQH.NextTD = NextTD->PhysicalAddress;
//             QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//         }
//     }
// }
//
// MPSTATUS
// NTAPI
// EHCI_ControlTransfer(IN PEHCI_EXTENSION EhciExtension,
//                      IN PEHCI_ENDPOINT EhciEndpoint,
//                      IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                      IN PEHCI_TRANSFER EhciTransfer,
//                      IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PEHCI_HCD_TD FirstTD;
//     PEHCI_HCD_TD LastTD;
//     PEHCI_HCD_TD TD;
//     PEHCI_HCD_TD PrevTD;
//     PEHCI_HCD_TD LinkTD;
//     ULONG TransferedLen = 0;
//     EHCI_TD_TOKEN Token;
//     ULONG DataToggle = 1;
//
//     DPRINT_EHCI!("EHCI_ControlTransfer: EhciEndpoint - %p, EhciTransfer - %p\n",
//                 EhciEndpoint,
//                 EhciTransfer);
//
//     if (EhciEndpoint->RemainTDs < EHCI_MAX_CONTROL_TD_COUNT)
//         return MP_STATUS_FAILURE;
//
//     EhciExtension->PendingTransfers++;
//     EhciEndpoint->PendingTDs++;
//
//     EhciTransfer->TransferOnAsyncList = 1;
//
//     FirstTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//     if (!FirstTD)
//     {
//         RegPacket.UsbPortBugCheck(EhciExtension);
//         return MP_STATUS_FAILURE;
//     }
//
//     EhciTransfer->PendingTDs++;
//
//     FirstTD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//     FirstTD->EhciTransfer = EhciTransfer;
//
//     FirstTD->HwTD.Buffer[0] = FirstTD->PhysicalAddress + FIELD_OFFSET(EHCI_HCD_TD, SetupPacket);
//     FirstTD->HwTD.Buffer[1] = 0;
//     FirstTD->HwTD.Buffer[2] = 0;
//     FirstTD->HwTD.Buffer[3] = 0;
//     FirstTD->HwTD.Buffer[4] = 0;
//
//     FirstTD->NextHcdTD = NULL;
//
//     FirstTD->HwTD.NextTD = TERMINATE_POINTER;
//     FirstTD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//     FirstTD->HwTD.Token.AsULONG = 0;
//     FirstTD->HwTD.Token.ErrorCounter = 3;
//     FirstTD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_SETUP;
//     FirstTD->HwTD.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//     FirstTD->HwTD.Token.TransferBytes = sizeof(FirstTD->SetupPacket);
//
//     RtlCopyMemory(&FirstTD->SetupPacket,
//                   &TransferParameters->SetupPacket,
//                   sizeof(FirstTD->SetupPacket));
//
//     LastTD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//     if (!LastTD)
//     {
//         RegPacket.UsbPortBugCheck(EhciExtension);
//         return MP_STATUS_FAILURE;
//     }
//
//     EhciTransfer->PendingTDs++;
//
//     LastTD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//     LastTD->EhciTransfer = EhciTransfer;
//
//     LastTD->HwTD.Buffer[0] = 0;
//     LastTD->HwTD.Buffer[1] = 0;
//     LastTD->HwTD.Buffer[2] = 0;
//     LastTD->HwTD.Buffer[3] = 0;
//     LastTD->HwTD.Buffer[4] = 0;
//
//     LastTD->NextHcdTD = NULL;
//     LastTD->HwTD.NextTD = TERMINATE_POINTER;
//     LastTD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//     LastTD->HwTD.Token.AsULONG = 0;
//     LastTD->HwTD.Token.ErrorCounter = 3;
//
//     FirstTD->AltNextHcdTD = LastTD;
//     FirstTD->HwTD.AlternateNextTD = LastTD->PhysicalAddress;
//
//     PrevTD = FirstTD;
//     LinkTD = FirstTD;
//
//     while (TransferedLen < TransferParameters->TransferBufferLength)
//     {
//         TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//         if (!TD)
//         {
//             RegPacket.UsbPortBugCheck(EhciExtension);
//             return MP_STATUS_FAILURE;
//         }
//
//         LinkTD = TD;
//
//         EhciTransfer->PendingTDs++;
//
//         TD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//         TD->EhciTransfer = EhciTransfer;
//
//         TD->HwTD.Buffer[0] = 0;
//         TD->HwTD.Buffer[1] = 0;
//         TD->HwTD.Buffer[2] = 0;
//         TD->HwTD.Buffer[3] = 0;
//         TD->HwTD.Buffer[4] = 0;
//
//         TD->NextHcdTD = NULL;
//
//         TD->HwTD.NextTD = TERMINATE_POINTER;
//         TD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//         TD->HwTD.Token.AsULONG = 0;
//         TD->HwTD.Token.ErrorCounter = 3;
//
//         PrevTD->NextHcdTD = TD;
//         PrevTD->HwTD.NextTD = TD->PhysicalAddress;
//
//         if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_IN;
//         else
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_OUT;
//
//         TD->HwTD.Token.DataToggle = DataToggle;
//         TD->HwTD.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//
//         if (DataToggle)
//             TD->HwTD.Token.DataToggle = 1;
//         else
//             TD->HwTD.Token.DataToggle = 0;
//
//         TD->AltNextHcdTD = LastTD;
//         TD->HwTD.AlternateNextTD = LastTD->PhysicalAddress;
//
//         TransferedLen = EHCI_MapAsyncTransferToTd(EhciExtension,
//                                                   EhciEndpoint->EndpointProperties.MaxPacketSize,
//                                                   TransferedLen,
//                                                   &DataToggle,
//                                                   EhciTransfer,
//                                                   TD,
//                                                   SgList);
//
//         PrevTD = TD;
//     }
//
//     LinkTD->NextHcdTD = LastTD;
//     LinkTD->HwTD.NextTD = LastTD->PhysicalAddress;
//
//     LastTD->HwTD.Buffer[0] = 0;
//     LastTD->LengthThisTD = 0;
//
//     Token.AsULONG = 0;
//     Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//     Token.InterruptOnComplete = 1;
//     Token.DataToggle = 1;
//
//     if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//         Token.PIDCode = EHCI_TD_TOKEN_PID_OUT;
//     else
//         Token.PIDCode = EHCI_TD_TOKEN_PID_IN;
//
//     LastTD->HwTD.Token = Token;
//
//     LastTD->NextHcdTD = EhciEndpoint->HcdTailP;
//     LastTD->HwTD.NextTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//
//     EHCI_EnableAsyncList(EhciExtension);
//     EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);
//
//     ASSERT(EhciEndpoint->HcdTailP->NextHcdTD == NULL);
//     ASSERT(EhciEndpoint->HcdTailP->AltNextHcdTD == NULL);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_BulkTransfer(IN PEHCI_EXTENSION EhciExtension,
//                   IN PEHCI_ENDPOINT EhciEndpoint,
//                   IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                   IN PEHCI_TRANSFER EhciTransfer,
//                   IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PEHCI_HCD_TD PrevTD;
//     PEHCI_HCD_TD FirstTD;
//     PEHCI_HCD_TD TD;
//     ULONG TransferedLen;
//
//     DPRINT_EHCI!("EHCI_BulkTransfer: EhciEndpoint - %p, EhciTransfer - %p\n",
//                 EhciEndpoint,
//                 EhciTransfer);
//
//     if (((TransferParameters->TransferBufferLength /
//         ((EHCI_MAX_QTD_BUFFER_PAGES - 1) * PAGE_SIZE)) + 1) > EhciEndpoint->RemainTDs)
//     {
//         DPRINT1("EHCI_BulkTransfer: return MP_STATUS_FAILURE\n");
//         return MP_STATUS_FAILURE;
//     }
//
//     EhciExtension->PendingTransfers++;
//     EhciEndpoint->PendingTDs++;
//
//     EhciTransfer->TransferOnAsyncList = 1;
//
//     TransferedLen = 0;
//     PrevTD = NULL;
//
//     if (TransferParameters->TransferBufferLength)
//     {
//         while (TransferedLen < TransferParameters->TransferBufferLength)
//         {
//             TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//             if (!TD)
//             {
//                 RegPacket.UsbPortBugCheck(EhciExtension);
//                 return MP_STATUS_FAILURE;
//             }
//
//             EhciTransfer->PendingTDs++;
//
//             TD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//             TD->EhciTransfer = EhciTransfer;
//
//             TD->HwTD.Buffer[0] = 0;
//             TD->HwTD.Buffer[1] = 0;
//             TD->HwTD.Buffer[2] = 0;
//             TD->HwTD.Buffer[3] = 0;
//             TD->HwTD.Buffer[4] = 0;
//
//             TD->NextHcdTD = NULL;
//             TD->HwTD.NextTD = TERMINATE_POINTER;
//             TD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//             TD->HwTD.Token.AsULONG = 0;
//             TD->HwTD.Token.ErrorCounter = 3;
//
//             if (EhciTransfer->PendingTDs == 1)
//             {
//                 FirstTD = TD;
//             }
//             else
//             {
//                 PrevTD->HwTD.NextTD = TD->PhysicalAddress;
//                 PrevTD->NextHcdTD = TD;
//             }
//
//             TD->HwTD.AlternateNextTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//             TD->AltNextHcdTD = EhciEndpoint->HcdTailP;
//
//             TD->HwTD.Token.InterruptOnComplete = 1;
//
//             if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//                 TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_IN;
//             else
//                 TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_OUT;
//
//             TD->HwTD.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//             TD->HwTD.Token.DataToggle = 1;
//
//             TransferedLen = EHCI_MapAsyncTransferToTd(EhciExtension,
//                                                       EhciEndpoint->EndpointProperties.MaxPacketSize,
//                                                       TransferedLen,
//                                                       0,
//                                                       EhciTransfer,
//                                                       TD,
//                                                       SgList);
//
//             PrevTD = TD;
//         }
//     }
//     else
//     {
//         TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//         if (!TD)
//         {
//             RegPacket.UsbPortBugCheck(EhciExtension);
//             return MP_STATUS_FAILURE;
//         }
//
//         EhciTransfer->PendingTDs++;
//
//         TD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//         TD->EhciTransfer = EhciTransfer;
//
//         TD->HwTD.Buffer[0] = 0;
//         TD->HwTD.Buffer[1] = 0;
//         TD->HwTD.Buffer[2] = 0;
//         TD->HwTD.Buffer[3] = 0;
//         TD->HwTD.Buffer[4] = 0;
//
//         TD->HwTD.NextTD = TERMINATE_POINTER;
//         TD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//         TD->HwTD.Token.AsULONG = 0;
//         TD->HwTD.Token.ErrorCounter = 3;
//
//         TD->NextHcdTD = NULL;
//
//         ASSERT(EhciTransfer->PendingTDs == 1);
//
//         FirstTD = TD;
//
//         TD->HwTD.AlternateNextTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//         TD->AltNextHcdTD = EhciEndpoint->HcdTailP;
//
//         TD->HwTD.Token.InterruptOnComplete = 1;
//
//         if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_IN;
//         else
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_OUT;
//
//         TD->HwTD.Buffer[0] = TD->PhysicalAddress;
//
//         TD->HwTD.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//         TD->HwTD.Token.DataToggle = 1;
//
//         TD->LengthThisTD = 0;
//     }
//
//     TD->HwTD.NextTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//     TD->NextHcdTD = EhciEndpoint->HcdTailP;
//
//     EHCI_EnableAsyncList(EhciExtension);
//     EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);
//
//     ASSERT(EhciEndpoint->HcdTailP->NextHcdTD == 0);
//     ASSERT(EhciEndpoint->HcdTailP->AltNextHcdTD == 0);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_InterruptTransfer(IN PEHCI_EXTENSION EhciExtension,
//                        IN PEHCI_ENDPOINT EhciEndpoint,
//                        IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                        IN PEHCI_TRANSFER EhciTransfer,
//                        IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PEHCI_HCD_TD TD;
//     PEHCI_HCD_TD FirstTD;
//     PEHCI_HCD_TD PrevTD = NULL;
//     ULONG TransferedLen = 0;
//
//     DPRINT_EHCI!("EHCI_InterruptTransfer: EhciEndpoint - %p, EhciTransfer - %p\n",
//                 EhciEndpoint,
//                 EhciTransfer);
//
//     if (!EhciEndpoint->RemainTDs)
//     {
//         DPRINT1("EHCI_InterruptTransfer: EhciEndpoint - %p\n", EhciEndpoint);
//         DbgBreakPoint();
//         return MP_STATUS_FAILURE;
//     }
//
//     EhciEndpoint->PendingTDs++;
//
//     if (!TransferParameters->TransferBufferLength)
//     {
//         DPRINT1("EHCI_InterruptTransfer: EhciEndpoint - %p\n", EhciEndpoint);
//         DbgBreakPoint();
//         return MP_STATUS_FAILURE;
//     }
//
//     while (TransferedLen < TransferParameters->TransferBufferLength)
//     {
//         TD = EHCI_AllocTd(EhciExtension, EhciEndpoint);
//
//         if (!TD)
//         {
//             DPRINT1("EHCI_InterruptTransfer: EhciEndpoint - %p\n", EhciEndpoint);
//             RegPacket.UsbPortBugCheck(EhciExtension);
//             return MP_STATUS_FAILURE;
//         }
//
//         EhciTransfer->PendingTDs++;
//
//         TD->TdFlags |= EHCI_HCD_TD_FLAG_PROCESSED;
//         TD->EhciTransfer = EhciTransfer;
//
//         TD->HwTD.Buffer[0] = 0;
//         TD->HwTD.Buffer[1] = 0;
//         TD->HwTD.Buffer[2] = 0;
//         TD->HwTD.Buffer[3] = 0;
//         TD->HwTD.Buffer[4] = 0;
//
//         TD->HwTD.NextTD = TERMINATE_POINTER;
//         TD->HwTD.AlternateNextTD = TERMINATE_POINTER;
//
//         TD->HwTD.Token.AsULONG = 0;
//         TD->HwTD.Token.ErrorCounter = 3;
//
//         TD->NextHcdTD = NULL;
//
//         if (EhciTransfer->PendingTDs == 1)
//         {
//             FirstTD = TD;
//         }
//         else if (PrevTD)
//         {
//             PrevTD->HwTD.NextTD = TD->PhysicalAddress;
//             PrevTD->NextHcdTD = TD;
//         }
//
//         if (TransferParameters->TransferFlags & USBD_TRANSFER_DIRECTION_IN)
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_IN;
//         else
//             TD->HwTD.Token.PIDCode = EHCI_TD_TOKEN_PID_OUT;
//
//         TD->HwTD.Token.Status = (UCHAR)EHCI_TOKEN_STATUS_ACTIVE;
//         TD->HwTD.Token.DataToggle = 1;
//
//         TransferedLen = EHCI_MapAsyncTransferToTd(EhciExtension,
//                                                   EhciEndpoint->EndpointProperties.TotalMaxPacketSize,
//                                                   TransferedLen,
//                                                   NULL,
//                                                   EhciTransfer,
//                                                   TD,
//                                                   SgList);
//
//         PrevTD = TD;
//     }
//
//     TD->HwTD.Token.InterruptOnComplete = 1;
//
//     DPRINT_EHCI!("EHCI_InterruptTransfer: PendingTDs - %x, TD->PhysicalAddress - %p, FirstTD - %p\n",
//                 EhciTransfer->PendingTDs,
//                 TD->PhysicalAddress,
//                 FirstTD);
//
//     TD->HwTD.NextTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//     TD->NextHcdTD = EhciEndpoint->HcdTailP;
//
//     EHCI_LinkTransferToQueue(EhciExtension, EhciEndpoint, FirstTD);
//
//     ASSERT(EhciEndpoint->HcdTailP->NextHcdTD == NULL);
//     ASSERT(EhciEndpoint->HcdTailP->AltNextHcdTD == NULL);
//
//     EHCI_EnablePeriodicList(EhciExtension);
//
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_SubmitTransfer(IN PVOID ehciExtension,
//                     IN PVOID ehciEndpoint,
//                     IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                     IN PVOID ehciTransfer,
//                     IN PUSBPORT_SCATTER_GATHER_LIST SgList)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_ENDPOINT EhciEndpoint = ehciEndpoint;
//     PEHCI_TRANSFER EhciTransfer = ehciTransfer;
//     MPSTATUS MPStatus;
//
//     DPRINT_EHCI!("EHCI_SubmitTransfer: EhciEndpoint - %p, EhciTransfer - %p\n",
//                 EhciEndpoint,
//                 EhciTransfer);
//
//     RtlZeroMemory(EhciTransfer, sizeof(EHCI_TRANSFER));
//
//     EhciTransfer->TransferParameters = TransferParameters;
//     EhciTransfer->USBDStatus = USBD_STATUS_SUCCESS;
//     EhciTransfer->EhciEndpoint = EhciEndpoint;
//
//     switch (EhciEndpoint->EndpointProperties.TransferType)
//     {
//         case USBPORT_TRANSFER_TYPE_CONTROL:
//             MPStatus = EHCI_ControlTransfer(EhciExtension,
//                                             EhciEndpoint,
//                                             TransferParameters,
//                                             EhciTransfer,
//                                             SgList);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_BULK:
//             MPStatus = EHCI_BulkTransfer(EhciExtension,
//                                          EhciEndpoint,
//                                          TransferParameters,
//                                          EhciTransfer,
//                                          SgList);
//             break;
//
//         case USBPORT_TRANSFER_TYPE_INTERRUPT:
//             MPStatus = EHCI_InterruptTransfer(EhciExtension,
//                                               EhciEndpoint,
//                                               TransferParameters,
//                                               EhciTransfer,
//                                               SgList);
//             break;
//
//         default:
//             DbgBreakPoint();
//             MPStatus = MP_STATUS_NOT_SUPPORTED;
//             break;
//     }
//
//     return MPStatus;
// }
//
// MPSTATUS
// NTAPI
// EHCI_SubmitIsoTransfer(IN PVOID ehciExtension,
//                        IN PVOID ehciEndpoint,
//                        IN PUSBPORT_TRANSFER_PARAMETERS TransferParameters,
//                        IN PVOID ehciTransfer,
//                        IN PVOID isoParameters)
// {
//     DPRINT1("EHCI_SubmitIsoTransfer: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_AbortIsoTransfer(IN PEHCI_EXTENSION EhciExtension,
//                       IN PEHCI_ENDPOINT EhciEndpoint,
//                       IN PEHCI_TRANSFER EhciTransfer)
// {
//     DPRINT1("EHCI_AbortIsoTransfer: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_AbortAsyncTransfer(IN PEHCI_EXTENSION EhciExtension,
//                         IN PEHCI_ENDPOINT EhciEndpoint,
//                         IN PEHCI_TRANSFER EhciTransfer)
// {
//     PEHCI_HCD_QH QH;
//     PEHCI_HCD_TD TD;
//     ULONG TransferLength;
//     PEHCI_HCD_TD CurrentTD;
//     PEHCI_TRANSFER CurrentTransfer;
//     ULONG FirstTdPA;
//     PEHCI_HCD_TD LastTD;
//     PEHCI_HCD_TD PrevTD;
//     ULONG NextTD;
//
//     DPRINT("EHCI_AbortAsyncTransfer: EhciEndpoint - %p, EhciTransfer - %p\n",
//            EhciEndpoint,
//            EhciTransfer);
//
//     QH = EhciEndpoint->QH;
//     TD = EhciEndpoint->HcdHeadP;
//
//     ASSERT(EhciEndpoint->PendingTDs);
//     EhciEndpoint->PendingTDs--;
//
//     if (TD->EhciTransfer == EhciTransfer)
//     {
//         TransferLength = 0;
//
//         while (TD != EhciEndpoint->HcdTailP &&
//                TD->EhciTransfer == EhciTransfer)
//         {
//             TransferLength += TD->LengthThisTD - TD->HwTD.Token.TransferBytes;
//
//             TD->HwTD.NextTD = 0;
//             TD->HwTD.AlternateNextTD = 0;
//
//             TD->TdFlags = 0;
//             TD->EhciTransfer = NULL;
//
//             EhciEndpoint->RemainTDs++;
//
//             TD = TD->NextHcdTD;
//         }
//
//         if (TransferLength)
//             EhciTransfer->TransferLen += TransferLength;
//
//         QH->sqh.HwQH.CurrentTD = EhciEndpoint->DmaBufferPA;
//         QH->sqh.HwQH.NextTD = TD->PhysicalAddress;
//         QH->sqh.HwQH.AlternateNextTD = TD->HwTD.AlternateNextTD;
//
//         QH->sqh.HwQH.Token.TransferBytes = 0;
//         QH->sqh.HwQH.Token.Status = (UCHAR)~(EHCI_TOKEN_STATUS_ACTIVE |
//                                              EHCI_TOKEN_STATUS_HALTED);
//
//         EhciEndpoint->HcdHeadP = TD;
//     }
//     else
//     {
//         DPRINT("EHCI_AbortAsyncTransfer: TD->EhciTransfer - %p\n", TD->EhciTransfer);
//
//         CurrentTD = RegPacket.UsbPortGetMappedVirtualAddress(QH->sqh.HwQH.CurrentTD,
//                                                              EhciExtension,
//                                                              EhciEndpoint);
//
//         CurrentTransfer = CurrentTD->EhciTransfer;
//         TD = EhciEndpoint->HcdHeadP;
//
//         while (TD && TD->EhciTransfer != EhciTransfer)
//         {
//             PrevTD = TD;
//             TD = TD->NextHcdTD;
//         }
//
//         FirstTdPA = TD->PhysicalAddress;
//
//         while (TD && TD->EhciTransfer == EhciTransfer)
//         {
//             TD->HwTD.NextTD = 0;
//             TD->HwTD.AlternateNextTD = 0;
//
//             TD->TdFlags = 0;
//             TD->EhciTransfer = NULL;
//
//             EhciEndpoint->RemainTDs++;
//
//             TD = TD->NextHcdTD;
//         }
//
//         LastTD = TD;
//         NextTD = LastTD->PhysicalAddress + FIELD_OFFSET(EHCI_HCD_TD, HwTD.NextTD);
//
//         PrevTD->HwTD.NextTD = LastTD->PhysicalAddress;
//         PrevTD->HwTD.AlternateNextTD = LastTD->PhysicalAddress;
//
//         PrevTD->NextHcdTD = LastTD;
//         PrevTD->AltNextHcdTD = LastTD;
//
//         if (CurrentTransfer == EhciTransfer)
//         {
//             QH->sqh.HwQH.CurrentTD = EhciEndpoint->DmaBufferPA;
//
//             QH->sqh.HwQH.Token.Status = (UCHAR)~EHCI_TOKEN_STATUS_ACTIVE;
//             QH->sqh.HwQH.Token.TransferBytes = 0;
//
//             QH->sqh.HwQH.NextTD = NextTD;
//             QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//
//             return;
//         }
//
//         if (PrevTD->EhciTransfer == CurrentTransfer)
//         {
//             if (QH->sqh.HwQH.NextTD == FirstTdPA)
//                 QH->sqh.HwQH.NextTD = NextTD;
//
//             if (QH->sqh.HwQH.AlternateNextTD == FirstTdPA)
//                 QH->sqh.HwQH.AlternateNextTD = NextTD;
//
//             for (TD = EhciEndpoint->HcdHeadP;
//                  TD;
//                  TD = TD->NextHcdTD)
//             {
//                 if (TD->EhciTransfer == CurrentTransfer)
//                 {
//                     TD->HwTD.AlternateNextTD = NextTD;
//                     TD->AltNextHcdTD = LastTD;
//                 }
//             }
//         }
//     }
// }
//
// VOID
// NTAPI
// EHCI_AbortTransfer(IN PVOID ehciExtension,
//                    IN PVOID ehciEndpoint,
//                    IN PVOID ehciTransfer,
//                    IN PULONG CompletedLength)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_ENDPOINT EhciEndpoint = ehciEndpoint;
//     PEHCI_TRANSFER EhciTransfer = ehciTransfer;
//     ULONG TransferType;
//
//     DPRINT("EHCI_AbortTransfer: EhciTransfer - %p, CompletedLength - %x\n",
//            EhciTransfer,
//            CompletedLength);
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         EHCI_AbortIsoTransfer(EhciExtension, EhciEndpoint, EhciTransfer);
//     else
//         EHCI_AbortAsyncTransfer(EhciExtension, EhciEndpoint, EhciTransfer);
// }
//
// ULONG
// NTAPI
// EHCI_GetEndpointState(IN PVOID ehciExtension,
//                       IN PVOID ehciEndpoint)
// {
//     DPRINT1("EHCI_GetEndpointState: UNIMPLEMENTED. FIXME\n");
//     return 0;
// }
//
// VOID
// NTAPI
// EHCI_RemoveQhFromPeriodicList(IN PEHCI_EXTENSION EhciExtension,
//                               IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_QH QH;
//     PEHCI_HCD_QH NextHead;
//     ULONG NextQhPA;
//     PEHCI_HCD_QH PrevHead;
//
//     QH = EhciEndpoint->QH;
//
//     if (!(QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE))
//         return;
//
//     DPRINT("EHCI_RemoveQhFromPeriodicList: EhciEndpoint - %p, QH - %X, EhciEndpoint->StaticQH - %p\n",
//            EhciEndpoint,
//            QH,
//            EhciEndpoint->StaticQH);
//
//     NextHead = QH->sqh.NextHead;
//     PrevHead = QH->sqh.PrevHead;
//
//     PrevHead->sqh.NextHead = NextHead;
//
//     if (NextHead)
//     {
//         if (!(NextHead->sqh.QhFlags & EHCI_QH_FLAG_STATIC))
//             NextHead->sqh.PrevHead = PrevHead;
//
//         NextQhPA = NextHead->sqh.PhysicalAddress;
//         NextQhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//         NextQhPA |= (EHCI_LINK_TYPE_QH << 1);
//
//         PrevHead->sqh.HwQH.HorizontalLink.AsULONG = NextQhPA;
//     }
//     else
//     {
//         PrevHead->sqh.HwQH.HorizontalLink.Terminate = 1;
//     }
//
//     QH->sqh.QhFlags &= ~EHCI_QH_FLAG_IN_SCHEDULE;
//
//     QH->sqh.NextHead = NULL;
//     QH->sqh.PrevHead = NULL;
// }
//
// VOID
// NTAPI
// EHCI_RemoveQhFromAsyncList(IN PEHCI_EXTENSION EhciExtension,
//                            IN PEHCI_HCD_QH QH)
// {
//     PEHCI_HCD_QH NextHead;
//     ULONG NextHeadPA;
//     PEHCI_HCD_QH PrevHead;
//     PEHCI_STATIC_QH AsyncHead;
//     ULONG AsyncHeadPA;
//
//     DPRINT("EHCI_RemoveQhFromAsyncList: QH - %p\n", QH);
//
//     if (QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE)
//     {
//         NextHead = QH->sqh.NextHead;
//         PrevHead = QH->sqh.PrevHead;
//
//         AsyncHead = EhciExtension->AsyncHead;
//
//         AsyncHeadPA = AsyncHead->PhysicalAddress;
//         AsyncHeadPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//         AsyncHeadPA |= (EHCI_LINK_TYPE_QH << 1);
//
//         NextHeadPA = NextHead->sqh.PhysicalAddress;
//         NextHeadPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//         NextHeadPA |= (EHCI_LINK_TYPE_QH << 1);
//
//         PrevHead->sqh.HwQH.HorizontalLink.AsULONG = NextHeadPA;
//
//         PrevHead->sqh.NextHead = NextHead;
//         NextHead->sqh.PrevHead = PrevHead;
//
//         EHCI_FlushAsyncCache(EhciExtension);
//
//         if (READ_REGISTER_ULONG(&EhciExtension->OperationalRegs->AsyncListBase) ==
//             QH->sqh.PhysicalAddress)
//         {
//             WRITE_REGISTER_ULONG(&EhciExtension->OperationalRegs->AsyncListBase,
//                                  AsyncHeadPA);
//         }
//
//         QH->sqh.QhFlags &= ~EHCI_QH_FLAG_IN_SCHEDULE;
//     }
// }
//
// VOID
// NTAPI
// EHCI_InsertQhInPeriodicList(IN PEHCI_EXTENSION EhciExtension,
//                             IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_STATIC_QH StaticQH;
//     PEHCI_HCD_QH QH;
//     ULONG QhPA;
//     PEHCI_HCD_QH NextHead;
//     PEHCI_HCD_QH PrevHead;
//
//     QH = EhciEndpoint->QH;
//     StaticQH = EhciEndpoint->StaticQH;
//
//     ASSERT((QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE) == 0);
//     ASSERT(StaticQH->QhFlags & EHCI_QH_FLAG_STATIC);
//
//     NextHead = StaticQH->NextHead;
//
//     QH->sqh.Period = EhciEndpoint->EndpointProperties.Period;
//     QH->sqh.Ordinal = EhciEndpoint->EndpointProperties.Reserved6;
//
//     DPRINT("EHCI_InsertQhInPeriodicList: EhciEndpoint - %p, QH - %X, EhciEndpoint->StaticQH - %p\n",
//            EhciEndpoint,
//            QH,
//            EhciEndpoint->StaticQH);
//
//     PrevHead = (PEHCI_HCD_QH)StaticQH;
//
//     if ((StaticQH->QhFlags & EHCI_QH_FLAG_STATIC) &&
//         (!NextHead || (NextHead->sqh.QhFlags & EHCI_QH_FLAG_STATIC)))
//     {
//         DPRINT("EHCI_InsertQhInPeriodicList: StaticQH - %p, StaticQH->NextHead - %p\n",
//                StaticQH,
//                StaticQH->NextHead);
//     }
//     else
//     {
//         while (NextHead &&
//                !(NextHead->sqh.QhFlags & EHCI_QH_FLAG_STATIC) &&
//                QH->sqh.Ordinal > NextHead->sqh.Ordinal)
//         {
//             PrevHead = NextHead;
//             NextHead = NextHead->sqh.NextHead;
//         }
//     }
//
//     QH->sqh.NextHead = NextHead;
//     QH->sqh.PrevHead = PrevHead;
//
//     if (NextHead && !(NextHead->sqh.QhFlags & EHCI_QH_FLAG_STATIC))
//         NextHead->sqh.PrevHead = QH;
//
//     QH->sqh.QhFlags |= EHCI_QH_FLAG_IN_SCHEDULE;
//     QH->sqh.HwQH.HorizontalLink = PrevHead->sqh.HwQH.HorizontalLink;
//
//     PrevHead->sqh.NextHead = QH;
//
//     QhPA = QH->sqh.PhysicalAddress;
//     QhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//     QhPA |= (EHCI_LINK_TYPE_QH << 1);
//
//     PrevHead->sqh.HwQH.HorizontalLink.AsULONG = QhPA;
// }
//
// VOID
// NTAPI
// EHCI_InsertQhInAsyncList(IN PEHCI_EXTENSION EhciExtension,
//                          IN PEHCI_HCD_QH QH)
// {
//     PEHCI_STATIC_QH AsyncHead;
//     ULONG QhPA;
//     PEHCI_HCD_QH NextHead;
//
//     DPRINT("EHCI_InsertQhInAsyncList: QH - %p\n", QH);
//
//     ASSERT((QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE) == 0);
//     ASSERT((QH->sqh.QhFlags & EHCI_QH_FLAG_NUKED) == 0);
//
//     AsyncHead = EhciExtension->AsyncHead;
//     NextHead = AsyncHead->NextHead;
//
//     QH->sqh.HwQH.HorizontalLink = AsyncHead->HwQH.HorizontalLink;
//     QH->sqh.QhFlags |= EHCI_QH_FLAG_IN_SCHEDULE;
//     QH->sqh.NextHead = NextHead;
//     QH->sqh.PrevHead = (PEHCI_HCD_QH)AsyncHead;
//
//     NextHead->sqh.PrevHead = QH;
//
//     QhPA = QH->sqh.PhysicalAddress;
//     QhPA &= LINK_POINTER_MASK + TERMINATE_POINTER;
//     QhPA |= (EHCI_LINK_TYPE_QH << 1);
//
//     AsyncHead->HwQH.HorizontalLink.AsULONG = QhPA;
//
//     AsyncHead->NextHead = QH;
// }
//
// VOID
// NTAPI
// EHCI_SetIsoEndpointState(IN PEHCI_EXTENSION EhciExtension,
//                          IN PEHCI_ENDPOINT EhciEndpoint,
//                          IN ULONG EndpointState)
// {
//     DPRINT1("EHCI_SetIsoEndpointState: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_SetAsyncEndpointState(IN PEHCI_EXTENSION EhciExtension,
//                            IN PEHCI_ENDPOINT EhciEndpoint,
//                            IN ULONG EndpointState)
// {
//     PEHCI_HCD_QH QH;
//     ULONG TransferType;
//
//     DPRINT("EHCI_SetAsyncEndpointState: EhciEndpoint - %p, EndpointState - %x\n",
//             EhciEndpoint,
//             EndpointState);
//
//     QH = EhciEndpoint->QH;
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     switch (EndpointState)
//     {
//         case USBPORT_ENDPOINT_PAUSED:
//             if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//                 EHCI_RemoveQhFromPeriodicList(EhciExtension, EhciEndpoint);
//             else
//                 EHCI_RemoveQhFromAsyncList(EhciExtension, EhciEndpoint->QH);
//
//             break;
//
//         case USBPORT_ENDPOINT_ACTIVE:
//             if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//                 EHCI_InsertQhInPeriodicList(EhciExtension, EhciEndpoint);
//             else
//                 EHCI_InsertQhInAsyncList(EhciExtension, EhciEndpoint->QH);
//
//             break;
//
//         case USBPORT_ENDPOINT_REMOVE:
//             QH->sqh.QhFlags |= EHCI_QH_FLAG_CLOSED;
//
//             if (TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//                 EHCI_RemoveQhFromPeriodicList(EhciExtension, EhciEndpoint);
//             else
//                 EHCI_RemoveQhFromAsyncList(EhciExtension, EhciEndpoint->QH);
//
//             break;
//
//         default:
//             DbgBreakPoint();
//             break;
//     }
//
//     EhciEndpoint->EndpointState = EndpointState;
// }
//
// VOID
// NTAPI
// EHCI_SetEndpointState(IN PVOID ehciExtension,
//                       IN PVOID ehciEndpoint,
//                       IN ULONG EndpointState)
// {
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG TransferType;
//
//     DPRINT("EHCI_SetEndpointState: ... \n");
//
//     EhciEndpoint = ehciEndpoint;
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//         TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//          EHCI_SetAsyncEndpointState((PEHCI_EXTENSION)ehciExtension,
//                                     EhciEndpoint,
//                                     EndpointState);
//     }
//     else if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//         EHCI_SetIsoEndpointState((PEHCI_EXTENSION)ehciExtension,
//                                  EhciEndpoint,
//                                  EndpointState);
//     }
//     else
//     {
//         RegPacket.UsbPortBugCheck(ehciExtension);
//     }
// }
//
// VOID
// NTAPI
// EHCI_InterruptNextSOF(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     DPRINT_EHCI!("EHCI_InterruptNextSOF: ... \n");
//
//     RegPacket.UsbPortInvalidateController(EhciExtension,
//                                           USBPORT_INVALIDATE_CONTROLLER_SOFT_INTERRUPT);
// }
//
// USBD_STATUS
// NTAPI
// EHCI_GetErrorFromTD(IN PEHCI_HCD_TD TD)
// {
//     EHCI_TD_TOKEN Token;
//
//     DPRINT_EHCI!("EHCI_GetErrorFromTD: ... \n");
//
//     ASSERT(TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_HALTED);
//
//     Token = TD->HwTD.Token;
//
//     if (Token.Status & EHCI_TOKEN_STATUS_TRANSACTION_ERROR)
//     {
//         DPRINT("EHCI_GetErrorFromTD: TD - %p, TRANSACTION_ERROR\n", TD);
//         return USBD_STATUS_XACT_ERROR;
//     }
//
//     if (Token.Status & EHCI_TOKEN_STATUS_BABBLE_DETECTED)
//     {
//         DPRINT("EHCI_GetErrorFromTD: TD - %p, BABBLE_DETECTED\n", TD);
//         return USBD_STATUS_BABBLE_DETECTED;
//     }
//
//     if (Token.Status & EHCI_TOKEN_STATUS_DATA_BUFFER_ERROR)
//     {
//         DPRINT("EHCI_GetErrorFromTD: TD - %p, DATA_BUFFER_ERROR\n", TD);
//         return USBD_STATUS_DATA_BUFFER_ERROR;
//     }
//
//     if (Token.Status & EHCI_TOKEN_STATUS_MISSED_MICROFRAME)
//     {
//         DPRINT("EHCI_GetErrorFromTD: TD - %p, MISSED_MICROFRAME\n", TD);
//         return USBD_STATUS_XACT_ERROR;
//     }
//
//     DPRINT("EHCI_GetErrorFromTD: TD - %p, STALL_PID\n", TD);
//     return USBD_STATUS_STALL_PID;
// }
//
// VOID
// NTAPI
// EHCI_ProcessDoneAsyncTd(IN PEHCI_EXTENSION EhciExtension,
//                         IN PEHCI_HCD_TD TD)
// {
//     PEHCI_TRANSFER EhciTransfer;
//     PUSBPORT_TRANSFER_PARAMETERS TransferParameters;
//     ULONG TransferType;
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG LengthTransfered;
//     USBD_STATUS USBDStatus;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_COMMAND Command;
//
//     DPRINT_EHCI!("EHCI_ProcessDoneAsyncTd: TD - %p\n", TD);
//
//     EhciTransfer = TD->EhciTransfer;
//
//     TransferParameters = EhciTransfer->TransferParameters;
//     EhciTransfer->PendingTDs--;
//
//     EhciEndpoint = EhciTransfer->EhciEndpoint;
//
//     if (!(TD->TdFlags & EHCI_HCD_TD_FLAG_ACTIVE))
//     {
//
//         if (TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_HALTED)
//             USBDStatus = EHCI_GetErrorFromTD(TD);
//         else
//             USBDStatus = USBD_STATUS_SUCCESS;
//
//         LengthTransfered = TD->LengthThisTD - TD->HwTD.Token.TransferBytes;
//
//         if (TD->HwTD.Token.PIDCode != EHCI_TD_TOKEN_PID_SETUP)
//             EhciTransfer->TransferLen += LengthTransfered;
//
//         if (USBDStatus != USBD_STATUS_SUCCESS)
//             EhciTransfer->USBDStatus = USBDStatus;
//     }
//
//     TD->HwTD.NextTD = 0;
//     TD->HwTD.AlternateNextTD = 0;
//
//     TD->TdFlags = 0;
//     TD->EhciTransfer = NULL;
//
//     EhciEndpoint->RemainTDs++;
//
//     if (EhciTransfer->PendingTDs == 0)
//     {
//         EhciEndpoint->PendingTDs--;
//
//         TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//         if (TransferType == USBPORT_TRANSFER_TYPE_CONTROL ||
//             TransferType == USBPORT_TRANSFER_TYPE_BULK)
//         {
//             EhciExtension->PendingTransfers--;
//
//             if (EhciExtension->PendingTransfers == 0)
//             {
//                 OperationalRegs = EhciExtension->OperationalRegs;
//                 Command.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcCommand.AsULONG);
//
//                 if (!Command.InterruptAdvanceDoorbell &&
//                     (EhciExtension->Flags & EHCI_FLAGS_IDLE_SUPPORT))
//                 {
//                     EHCI_DisableAsyncList(EhciExtension);
//                 }
//             }
//         }
//
//         RegPacket.UsbPortCompleteTransfer(EhciExtension,
//                                           EhciEndpoint,
//                                           TransferParameters,
//                                           EhciTransfer->USBDStatus,
//                                           EhciTransfer->TransferLen);
//     }
// }
//
// VOID
// NTAPI
// EHCI_PollActiveAsyncEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                              IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_QH QH;
//     PEHCI_HCD_TD TD;
//     PEHCI_HCD_TD CurrentTD;
//     ULONG CurrentTDPhys;
//     BOOLEAN IsScheduled;
//
//     DPRINT_EHCI!("EHCI_PollActiveAsyncEndpoint: ... \n");
//
//     QH = EhciEndpoint->QH;
//
//     CurrentTDPhys = QH->sqh.HwQH.CurrentTD & LINK_POINTER_MASK;
//     ASSERT(CurrentTDPhys);
//
//     CurrentTD = RegPacket.UsbPortGetMappedVirtualAddress(CurrentTDPhys,
//                                                          EhciExtension,
//                                                          EhciEndpoint);
//
//     if (CurrentTD == EhciEndpoint->DmaBufferVA)
//         return;
//
//     IsScheduled = QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE;
//
//     if (!EHCI_HardwarePresent(EhciExtension, 0))
//         IsScheduled = 0;
//
//     TD = EhciEndpoint->HcdHeadP;
//
//     if (TD == CurrentTD)
//     {
//         if (TD != EhciEndpoint->HcdTailP &&
//             !(TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE))
//         {
//             if (TD->NextHcdTD && TD->HwTD.NextTD != TD->NextHcdTD->PhysicalAddress)
//                 TD->HwTD.NextTD = TD->NextHcdTD->PhysicalAddress;
//
//             if (TD->AltNextHcdTD &&
//                 TD->HwTD.AlternateNextTD != TD->AltNextHcdTD->PhysicalAddress)
//             {
//                 TD->HwTD.AlternateNextTD = TD->AltNextHcdTD->PhysicalAddress;
//             }
//
//             if (QH->sqh.HwQH.CurrentTD == TD->PhysicalAddress &&
//                 !(TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE) &&
//                 (QH->sqh.HwQH.NextTD != TD->HwTD.NextTD ||
//                  QH->sqh.HwQH.AlternateNextTD != TD->HwTD.AlternateNextTD))
//             {
//                 QH->sqh.HwQH.NextTD = TD->HwTD.NextTD;
//                 QH->sqh.HwQH.AlternateNextTD = TD->HwTD.AlternateNextTD;
//             }
//
//             EHCI_InterruptNextSOF(EhciExtension);
//         }
//     }
//     else
//     {
//         while (TD != CurrentTD)
//         {
//             ASSERT((TD->TdFlags & EHCI_HCD_TD_FLAG_DUMMY) == 0);
//
//             TD->TdFlags |= EHCI_HCD_TD_FLAG_DONE;
//
//             if (TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE)
//                 TD->TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
//
//             InsertTailList(&EhciEndpoint->ListTDs, &TD->DoneLink);
//             TD = TD->NextHcdTD;
//         }
//     }
//
//     if (CurrentTD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE)
//     {
//         ASSERT(TD != NULL);
//         EhciEndpoint->HcdHeadP = TD;
//         return;
//     }
//
//     if ((CurrentTD->NextHcdTD != EhciEndpoint->HcdTailP) &&
//         (CurrentTD->AltNextHcdTD != EhciEndpoint->HcdTailP ||
//          CurrentTD->HwTD.Token.TransferBytes == 0))
//     {
//         ASSERT(TD != NULL);
//         EhciEndpoint->HcdHeadP = TD;
//         return;
//     }
//
//     if (IsScheduled)
//     {
//         EHCI_LockQH(EhciExtension,
//                     QH,
//                     EhciEndpoint->EndpointProperties.TransferType);
//     }
//
//     QH->sqh.HwQH.CurrentTD = EhciEndpoint->DmaBufferPA;
//
//     CurrentTD->TdFlags |= EHCI_HCD_TD_FLAG_DONE;
//     InsertTailList(&EhciEndpoint->ListTDs, &CurrentTD->DoneLink);
//
//     if (CurrentTD->HwTD.Token.TransferBytes &&
//         CurrentTD->AltNextHcdTD == EhciEndpoint->HcdTailP)
//     {
//         TD = CurrentTD->NextHcdTD;
//
//         while (TD != EhciEndpoint->HcdTailP)
//         {
//             TD->TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
//             InsertTailList(&EhciEndpoint->ListTDs, &TD->DoneLink);
//             TD = TD->NextHcdTD;
//         }
//     }
//
//     QH->sqh.HwQH.CurrentTD = EhciEndpoint->HcdTailP->PhysicalAddress;
//     QH->sqh.HwQH.NextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.Token.TransferBytes = 0;
//
//     EhciEndpoint->HcdHeadP = EhciEndpoint->HcdTailP;
//
//     if (IsScheduled)
//         EHCI_UnlockQH(EhciExtension, QH);
// }
//
// VOID
// NTAPI
// EHCI_PollHaltedAsyncEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                              IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_QH QH;
//     PEHCI_HCD_TD CurrentTD;
//     ULONG CurrentTdPA;
//     PEHCI_HCD_TD TD;
//     PEHCI_TRANSFER Transfer;
//     BOOLEAN IsScheduled;
//
//     DPRINT("EHCI_PollHaltedAsyncEndpoint: EhciEndpoint - %p\n", EhciEndpoint);
//
//     QH = EhciEndpoint->QH;
//     EHCI_DumpHwQH(QH);
//
//     CurrentTdPA = QH->sqh.HwQH.CurrentTD & LINK_POINTER_MASK;
//     ASSERT(CurrentTdPA);
//
//     IsScheduled = QH->sqh.QhFlags & EHCI_QH_FLAG_IN_SCHEDULE;
//
//     if (!EHCI_HardwarePresent(EhciExtension, 0))
//         IsScheduled = 0;
//
//     CurrentTD = RegPacket.UsbPortGetMappedVirtualAddress(CurrentTdPA,
//                                                          EhciExtension,
//                                                          EhciEndpoint);
//
//     DPRINT("EHCI_PollHaltedAsyncEndpoint: CurrentTD - %p\n", CurrentTD);
//
//     if (CurrentTD == EhciEndpoint->DmaBufferVA)
//         return;
//
//     ASSERT(EhciEndpoint->HcdTailP != CurrentTD);
//
//     if (IsScheduled)
//     {
//         EHCI_LockQH(EhciExtension,
//                     QH,
//                     EhciEndpoint->EndpointProperties.TransferType);
//     }
//
//     TD = EhciEndpoint->HcdHeadP;
//
//     while (TD != CurrentTD)
//     {
//         DPRINT("EHCI_PollHaltedAsyncEndpoint: TD - %p\n", TD);
//
//         ASSERT((TD->TdFlags & EHCI_HCD_TD_FLAG_DUMMY) == 0);
//
//         if (TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE)
//             TD->TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
//
//         TD->TdFlags |= EHCI_HCD_TD_FLAG_DONE;
//
//         InsertTailList(&EhciEndpoint->ListTDs, &TD->DoneLink);
//
//         TD = TD->NextHcdTD;
//     }
//
//     TD = CurrentTD;
//
//     Transfer = CurrentTD->EhciTransfer;
//
//     do
//     {
//         DPRINT("EHCI_PollHaltedAsyncEndpoint: TD - %p\n", TD);
//
//         if (TD->HwTD.Token.Status & EHCI_TOKEN_STATUS_ACTIVE)
//             TD->TdFlags |= EHCI_HCD_TD_FLAG_ACTIVE;
//
//         TD->TdFlags |= EHCI_HCD_TD_FLAG_DONE;
//
//         InsertTailList(&EhciEndpoint->ListTDs, &TD->DoneLink);
//
//         TD = TD->NextHcdTD;
//     }
//     while (TD->EhciTransfer == Transfer);
//
//     EhciEndpoint->HcdHeadP = TD;
//
//     QH->sqh.HwQH.CurrentTD = EhciEndpoint->DmaBufferPA;
//     QH->sqh.HwQH.NextTD = TD->PhysicalAddress;
//     QH->sqh.HwQH.AlternateNextTD = TERMINATE_POINTER;
//     QH->sqh.HwQH.Token.TransferBytes = 0;
//
//     if (IsScheduled)
//         EHCI_UnlockQH(EhciExtension, QH);
//
//     if (EhciEndpoint->EndpointStatus & USBPORT_ENDPOINT_CONTROL)
//     {
//         EhciEndpoint->EndpointStatus &= ~USBPORT_ENDPOINT_HALT;
//         QH->sqh.HwQH.Token.ErrorCounter = 0;
//         QH->sqh.HwQH.Token.Status &= (UCHAR)~(EHCI_TOKEN_STATUS_ACTIVE |
//                                               EHCI_TOKEN_STATUS_HALTED);
//
//     }
// }
//
// VOID
// NTAPI
// EHCI_PollAsyncEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                        IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     PEHCI_HCD_QH QH;
//     PLIST_ENTRY DoneList;
//     PEHCI_HCD_TD TD;
//
//     //DPRINT_EHCI!("EHCI_PollAsyncEndpoint: EhciEndpoint - %p\n", EhciEndpoint);
//
//     if (!EhciEndpoint->PendingTDs)
//         return;
//
//     QH = EhciEndpoint->QH;
//
//     if (QH->sqh.QhFlags & EHCI_QH_FLAG_CLOSED)
//         return;
//
//     if (QH->sqh.HwQH.Token.Status & EHCI_TOKEN_STATUS_ACTIVE ||
//         !(QH->sqh.HwQH.Token.Status & EHCI_TOKEN_STATUS_HALTED))
//     {
//         EHCI_PollActiveAsyncEndpoint(EhciExtension, EhciEndpoint);
//     }
//     else
//     {
//         EhciEndpoint->EndpointStatus |= USBPORT_ENDPOINT_HALT;
//         EHCI_PollHaltedAsyncEndpoint(EhciExtension, EhciEndpoint);
//     }
//
//     DoneList = &EhciEndpoint->ListTDs;
//
//     while (!IsListEmpty(DoneList))
//     {
//         TD = CONTAINING_RECORD(DoneList->Flink,
//                                EHCI_HCD_TD,
//                                DoneLink);
//
//         RemoveHeadList(DoneList);
//
//         ASSERT((TD->TdFlags & (EHCI_HCD_TD_FLAG_PROCESSED |
//                                EHCI_HCD_TD_FLAG_DONE)));
//
//         EHCI_ProcessDoneAsyncTd(EhciExtension, TD);
//     }
// }
//
// VOID
// NTAPI
// EHCI_PollIsoEndpoint(IN PEHCI_EXTENSION EhciExtension,
//                      IN PEHCI_ENDPOINT EhciEndpoint)
// {
//     DPRINT1("EHCI_PollIsoEndpoint: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_PollEndpoint(IN PVOID ehciExtension,
//                   IN PVOID ehciEndpoint)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_ENDPOINT EhciEndpoint = ehciEndpoint;
//     ULONG TransferType;
//
//     //DPRINT_EHCI!("EHCI_PollEndpoint: EhciEndpoint - %p\n", EhciEndpoint);
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         EHCI_PollIsoEndpoint(EhciExtension, EhciEndpoint);
//     else
//         EHCI_PollAsyncEndpoint(EhciExtension, EhciEndpoint);
// }
//
// VOID
// NTAPI
// EHCI_CheckController(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     //DPRINT_EHCI!("EHCI_CheckController: ... \n");
//
//     if (EhciExtension->IsStarted)
//         EHCI_HardwarePresent(EhciExtension, TRUE);
// }
//
// ULONG
// NTAPI
// EHCI_Get32BitFrameNumber(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     ULONG FrameIdx;
//     ULONG FrameIndex;
//     ULONG FrameNumber;
//
//     //DPRINT_EHCI!("EHCI_Get32BitFrameNumber: EhciExtension - %p\n", EhciExtension);
//
//     FrameIdx = EhciExtension->FrameIndex;
//     FrameIndex = READ_REGISTER_ULONG(&EhciExtension->OperationalRegs->FrameIndex);
//
//     FrameNumber = (USHORT)FrameIdx ^ ((FrameIndex / EHCI_MICROFRAMES) & EHCI_FRINDEX_FRAME_MASK);
//     FrameNumber &= EHCI_FRAME_LIST_MAX_ENTRIES;
//     FrameNumber += FrameIndex | ((FrameIndex / EHCI_MICROFRAMES) & EHCI_FRINDEX_INDEX_MASK);
//
//     return FrameNumber;
// }
//
// VOID
// NTAPI
// EHCI_EnableInterrupts(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     DPRINT("EHCI_EnableInterrupts: EhciExtension->InterruptMask - %x\n",
//            EhciExtension->InterruptMask.AsULONG);
//
//     WRITE_REGISTER_ULONG(&EhciExtension->OperationalRegs->HcInterruptEnable.AsULONG,
//                          EhciExtension->InterruptMask.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_DisableInterrupts(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//
//     DPRINT("EHCI_DisableInterrupts: ... \n");
//
//     WRITE_REGISTER_ULONG(&EhciExtension->OperationalRegs->HcInterruptEnable.AsULONG,
//                          0);
// }
//
// VOID
// NTAPI
// EHCI_PollController(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     ULONG Port;
//     EHCI_PORT_STATUS_CONTROL PortSC;
//
//     DPRINT_EHCI!("EHCI_PollController: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     if (!(EhciExtension->Flags & EHCI_FLAGS_CONTROLLER_SUSPEND))
//     {
//         RegPacket.UsbPortInvalidateRootHub(EhciExtension);
//         return;
//     }
//
//     if (EhciExtension->NumberOfPorts)
//     {
//         for (Port = 0; Port < EhciExtension->NumberOfPorts; Port++)
//         {
//             PortSC.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->PortControl[Port].AsULONG);
//
//             if (PortSC.ConnectStatusChange)
//                 RegPacket.UsbPortInvalidateRootHub(EhciExtension);
//         }
//     }
// }
//
// VOID
// NTAPI
// EHCI_SetEndpointDataToggle(IN PVOID ehciExtension,
//                            IN PVOID ehciEndpoint,
//                            IN ULONG DataToggle)
// {
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG TransferType;
//
//     EhciEndpoint = ehciEndpoint;
//
//     DPRINT("EHCI_SetEndpointDataToggle: EhciEndpoint - %p, DataToggle - %x\n",
//                 EhciEndpoint,
//                 DataToggle);
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_BULK ||
//         TransferType == USBPORT_TRANSFER_TYPE_INTERRUPT)
//     {
//         EhciEndpoint->QH->sqh.HwQH.Token.DataToggle = DataToggle;
//     }
// }
//
// ULONG
// NTAPI
// EHCI_GetEndpointStatus(IN PVOID ehciExtension,
//                        IN PVOID ehciEndpoint)
// {
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG TransferType;
//     ULONG EndpointStatus = USBPORT_ENDPOINT_RUN;
//
//     EhciEndpoint = ehciEndpoint;
//
//     DPRINT("EHCI_GetEndpointStatus: EhciEndpoint - %p\n", EhciEndpoint);
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType == USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//         return EndpointStatus;
//
//     if (EhciEndpoint->EndpointStatus & USBPORT_ENDPOINT_HALT)
//         EndpointStatus = USBPORT_ENDPOINT_HALT;
//
//     return EndpointStatus;
// }
//
// VOID
// NTAPI
// EHCI_SetEndpointStatus(IN PVOID ehciExtension,
//                        IN PVOID ehciEndpoint,
//                        IN ULONG EndpointStatus)
// {
//     PEHCI_ENDPOINT EhciEndpoint;
//     ULONG TransferType;
//     PEHCI_HCD_QH QH;
//
//     EhciEndpoint = ehciEndpoint;
//
//     DPRINT("EHCI_SetEndpointStatus: EhciEndpoint - %p, EndpointStatus - %x\n",
//                 EhciEndpoint,
//                 EndpointStatus);
//
//     TransferType = EhciEndpoint->EndpointProperties.TransferType;
//
//     if (TransferType != USBPORT_TRANSFER_TYPE_ISOCHRONOUS)
//     {
//
//         if (EndpointStatus == USBPORT_ENDPOINT_RUN)
//         {
//             EhciEndpoint->EndpointStatus &= ~USBPORT_ENDPOINT_HALT;
//
//             QH = EhciEndpoint->QH;
//             QH->sqh.HwQH.Token.Status &= (UCHAR)~EHCI_TOKEN_STATUS_HALTED;
//
//             return;
//         }
//
//         if (EndpointStatus == USBPORT_ENDPOINT_HALT)
//             DbgBreakPoint();
//     }
// }
//
// VOID
// NTAPI
// EHCI_ResetController(IN PVOID ehciExtension)
// {
//     DPRINT1("EHCI_ResetController: UNIMPLEMENTED. FIXME\n");
// }
//
// MPSTATUS
// NTAPI
// EHCI_StartSendOnePacket(IN PVOID ehciExtension,
//                         IN PVOID PacketParameters,
//                         IN PVOID Data,
//                         IN PULONG pDataLength,
//                         IN PVOID BufferVA,
//                         IN PVOID BufferPA,
//                         IN ULONG BufferLength,
//                         IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT1("EHCI_StartSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_EndSendOnePacket(IN PVOID ehciExtension,
//                       IN PVOID PacketParameters,
//                       IN PVOID Data,
//                       IN PULONG pDataLength,
//                       IN PVOID BufferVA,
//                       IN PVOID BufferPA,
//                       IN ULONG BufferLength,
//                       IN USBD_STATUS * pUSBDStatus)
// {
//     DPRINT1("EHCI_EndSendOnePacket: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// MPSTATUS
// NTAPI
// EHCI_PassThru(IN PVOID ehciExtension,
//               IN PVOID passThruParameters,
//               IN ULONG ParameterLength,
//               IN PVOID pParameters)
// {
//     DPRINT1("EHCI_PassThru: UNIMPLEMENTED. FIXME\n");
//     return MP_STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// EHCI_RebalanceEndpoint(IN PVOID ohciExtension,
//                        IN PUSBPORT_ENDPOINT_PROPERTIES EndpointProperties,
//                        IN PVOID ohciEndpoint)
// {
//     DPRINT1("EHCI_RebalanceEndpoint: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_FlushInterrupts(IN PVOID ehciExtension)
// {
//     PEHCI_EXTENSION EhciExtension = ehciExtension;
//     PEHCI_HW_REGISTERS OperationalRegs;
//     EHCI_USB_STATUS Status;
//
//     DPRINT("EHCI_FlushInterrupts: ... \n");
//
//     OperationalRegs = EhciExtension->OperationalRegs;
//
//     Status.AsULONG = READ_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG);
//     WRITE_REGISTER_ULONG(&OperationalRegs->HcStatus.AsULONG, Status.AsULONG);
// }
//
// VOID
// NTAPI
// EHCI_TakePortControl(IN PVOID ohciExtension)
// {
//     DPRINT1("EHCI_TakePortControl: UNIMPLEMENTED. FIXME\n");
// }
//
// VOID
// NTAPI
// EHCI_Unload(IN PDRIVER_OBJECT DriverObject)
// {
//     DPRINT1("EHCI_Unload: Not supported\n");
//     return;
// }
//
// NTSTATUS
// NTAPI
// DriverEntry(IN PDRIVER_OBJECT DriverObject,
//             IN PUNICODE_STRING RegistryPath)
// {
//     DPRINT("DriverEntry: DriverObject - %p, RegistryPath - %wZ\n",
//            DriverObject,
//            RegistryPath);
//
//     if (USBPORT_GetHciMn() != USBPORT_HCI_MN)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     RtlZeroMemory(&RegPacket, sizeof(USBPORT_REGISTRATION_PACKET));
//
//     RegPacket.MiniPortVersion = USB_MINIPORT_VERSION_EHCI;
//
//     RegPacket.MiniPortFlags = USB_MINIPORT_FLAGS_INTERRUPT |
//                               USB_MINIPORT_FLAGS_MEMORY_IO |
//                               USB_MINIPORT_FLAGS_USB2 |
//                               USB_MINIPORT_FLAGS_POLLING |
//                               USB_MINIPORT_FLAGS_WAKE_SUPPORT;
//
//     RegPacket.MiniPortBusBandwidth = TOTAL_USB20_BUS_BANDWIDTH;
//
//     RegPacket.MiniPortExtensionSize = sizeof(EHCI_EXTENSION);
//     RegPacket.MiniPortEndpointSize = sizeof(EHCI_ENDPOINT);
//     RegPacket.MiniPortTransferSize = sizeof(EHCI_TRANSFER);
//     RegPacket.MiniPortResourcesSize = sizeof(EHCI_HC_RESOURCES);
//
//     RegPacket.OpenEndpoint = EHCI_OpenEndpoint;
//     RegPacket.ReopenEndpoint = EHCI_ReopenEndpoint;
//     RegPacket.QueryEndpointRequirements = EHCI_QueryEndpointRequirements;
//     RegPacket.CloseEndpoint = EHCI_CloseEndpoint;
//     RegPacket.StartController = EHCI_StartController;
//     RegPacket.StopController = EHCI_StopController;
//     RegPacket.SuspendController = EHCI_SuspendController;
//     RegPacket.ResumeController = EHCI_ResumeController;
//     RegPacket.InterruptService = EHCI_InterruptService;
//     RegPacket.InterruptDpc = EHCI_InterruptDpc;
//     RegPacket.SubmitTransfer = EHCI_SubmitTransfer;
//     RegPacket.SubmitIsoTransfer = EHCI_SubmitIsoTransfer;
//     RegPacket.AbortTransfer = EHCI_AbortTransfer;
//     RegPacket.GetEndpointState = EHCI_GetEndpointState;
//     RegPacket.SetEndpointState = EHCI_SetEndpointState;
//     RegPacket.PollEndpoint = EHCI_PollEndpoint;
//     RegPacket.CheckController = EHCI_CheckController;
//     RegPacket.Get32BitFrameNumber = EHCI_Get32BitFrameNumber;
//     RegPacket.InterruptNextSOF = EHCI_InterruptNextSOF;
//     RegPacket.EnableInterrupts = EHCI_EnableInterrupts;
//     RegPacket.DisableInterrupts = EHCI_DisableInterrupts;
//     RegPacket.PollController = EHCI_PollController;
//     RegPacket.SetEndpointDataToggle = EHCI_SetEndpointDataToggle;
//     RegPacket.GetEndpointStatus = EHCI_GetEndpointStatus;
//     RegPacket.SetEndpointStatus = EHCI_SetEndpointStatus;
//     RegPacket.RH_GetRootHubData = EHCI_RH_GetRootHubData;
//     RegPacket.RH_GetStatus = EHCI_RH_GetStatus;
//     RegPacket.RH_GetPortStatus = EHCI_RH_GetPortStatus;
//     RegPacket.RH_GetHubStatus = EHCI_RH_GetHubStatus;
//     RegPacket.RH_SetFeaturePortReset = EHCI_RH_SetFeaturePortReset;
//     RegPacket.RH_SetFeaturePortPower = EHCI_RH_SetFeaturePortPower;
//     RegPacket.RH_SetFeaturePortEnable = EHCI_RH_SetFeaturePortEnable;
//     RegPacket.RH_SetFeaturePortSuspend = EHCI_RH_SetFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnable = EHCI_RH_ClearFeaturePortEnable;
//     RegPacket.RH_ClearFeaturePortPower = EHCI_RH_ClearFeaturePortPower;
//     RegPacket.RH_ClearFeaturePortSuspend = EHCI_RH_ClearFeaturePortSuspend;
//     RegPacket.RH_ClearFeaturePortEnableChange = EHCI_RH_ClearFeaturePortEnableChange;
//     RegPacket.RH_ClearFeaturePortConnectChange = EHCI_RH_ClearFeaturePortConnectChange;
//     RegPacket.RH_ClearFeaturePortResetChange = EHCI_RH_ClearFeaturePortResetChange;
//     RegPacket.RH_ClearFeaturePortSuspendChange = EHCI_RH_ClearFeaturePortSuspendChange;
//     RegPacket.RH_ClearFeaturePortOvercurrentChange = EHCI_RH_ClearFeaturePortOvercurrentChange;
//     RegPacket.RH_DisableIrq = EHCI_RH_DisableIrq;
//     RegPacket.RH_EnableIrq = EHCI_RH_EnableIrq;
//     RegPacket.StartSendOnePacket = EHCI_StartSendOnePacket;
//     RegPacket.EndSendOnePacket = EHCI_EndSendOnePacket;
//     RegPacket.PassThru = EHCI_PassThru;
//     RegPacket.RebalanceEndpoint = EHCI_RebalanceEndpoint;
//     RegPacket.FlushInterrupts = EHCI_FlushInterrupts;
//     RegPacket.RH_ChirpRootPort = EHCI_RH_ChirpRootPort;
//     RegPacket.TakePortControl = EHCI_TakePortControl;
//
//     DriverObject->DriverUnload = EHCI_Unload;
//
//     return USBPORT_RegisterUSBPortDriver(DriverObject,
//                                          USB20_MINIPORT_INTERFACE_VERSION,
//                                          &RegPacket);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
