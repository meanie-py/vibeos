#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(clippy::missing_safety_doc)]

//! usbxhci.sys — USB 3 xHCI host-controller miniport under usbport.
//!
//! Written in Rust from the start against the xHCI 1.2 specification; there is no C
//! body and no translation. Design, milestones and the Raspberry Pi 4 specifics:
//! `docs/engineering/usbxhci-design.md`.
//!
//! Layout: `hw` is the hardware vocabulary (registers, TRBs, barriers, cache
//! maintenance); `controller` owns bring-up, the command ring, the event ring and
//! interrupts; `roothub` maps usbport's USB 2.0 root hub onto the controller's
//! USB2-protocol ports; `endpoint` turns usbport's endpoints and transfers into
//! slots, contexts and transfer rings.

pub mod types;
pub mod hw;
pub mod controller;
pub mod roothub;
pub mod endpoint;

use crate::types::*;
use ntkrnl_sys::ddk::{NTSTATUS, PDRIVER_OBJECT, PUNICODE_STRING, ULONG};

extern "C" {
    pub fn DbgPrint(Format: *const i8, ...) -> u32;
}

extern "system" {
    fn USBPORT_GetHciMn() -> ULONG;
    fn USBPORT_RegisterUSBPortDriver(
        DriverObject: PDRIVER_OBJECT,
        Version: ULONG,
        RegPacket: *mut USBPORT_REGISTRATION_PACKET,
    ) -> NTSTATUS;
}

/// Always-on log line. Kept to state changes (start, reset, attach, errors) so
/// it never floods the serial console.
#[macro_export]
macro_rules! xlog {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        unsafe { $crate::DbgPrint(concat!("[XHCI] ", $fmt, "\n\0").as_ptr() as *const i8 $(, $arg)*) }
    };
}

/// The registration packet usbport copies at DriverEntry; the `UsbPort*` callbacks
/// it fills in are read back from here.
pub static mut RegPacket: USBPORT_REGISTRATION_PACKET =
    unsafe { core::mem::MaybeUninit::zeroed().assume_init() };

const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009Au32 as NTSTATUS;

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    _RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    if USBPORT_GetHciMn() != USBPORT_HCI_MN {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let p = &mut *core::ptr::addr_of_mut!(RegPacket);
    core::ptr::write_bytes(p as *mut _ as *mut u8, 0, core::mem::size_of::<USBPORT_REGISTRATION_PACKET>());

    p.MiniPortVersion = USB_MINIPORT_VERSION_XHCI;
    // USB2: usbport presents a USB 2.0 root hub and accepts high-speed devices.
    // POLLING: usbport also calls PollController on a timer, a backstop for a
    // missed interrupt.
    p.MiniPortFlags = USB_MINIPORT_FLAGS_INTERRUPT
        | USB_MINIPORT_FLAGS_MEMORY_IO
        | USB_MINIPORT_FLAGS_USB2
        | USB_MINIPORT_FLAGS_POLLING;
    p.MiniPortBusBandwidth = TOTAL_USB20_BUS_BANDWIDTH;
    p.MiniPortExtensionSize = core::mem::size_of::<controller::XhciExtension>() as _;
    p.MiniPortEndpointSize = core::mem::size_of::<endpoint::XhciEndpoint>() as _;
    p.MiniPortTransferSize = core::mem::size_of::<endpoint::XhciTransfer>() as _;
    p.MiniPortResourcesSize = controller::COMMON_BUFFER_SIZE as _;

    p.OpenEndpoint = Some(endpoint::XHCI_OpenEndpoint);
    p.ReopenEndpoint = Some(endpoint::XHCI_ReopenEndpoint);
    p.QueryEndpointRequirements = Some(endpoint::XHCI_QueryEndpointRequirements);
    p.CloseEndpoint = Some(endpoint::XHCI_CloseEndpoint);
    p.StartController = Some(controller::XHCI_StartController);
    p.StopController = Some(controller::XHCI_StopController);
    p.SuspendController = Some(controller::XHCI_SuspendController);
    p.ResumeController = Some(controller::XHCI_ResumeController);
    p.InterruptService = Some(controller::XHCI_InterruptService);
    p.InterruptDpc = Some(controller::XHCI_InterruptDpc);
    p.SubmitTransfer = Some(endpoint::XHCI_SubmitTransfer);
    p.SubmitIsoTransfer = Some(endpoint::XHCI_SubmitIsoTransfer);
    p.AbortTransfer = Some(endpoint::XHCI_AbortTransfer);
    p.GetEndpointState = Some(endpoint::XHCI_GetEndpointState);
    p.SetEndpointState = Some(endpoint::XHCI_SetEndpointState);
    p.PollEndpoint = Some(endpoint::XHCI_PollEndpoint);
    p.CheckController = Some(controller::XHCI_CheckController);
    p.Get32BitFrameNumber = Some(controller::XHCI_Get32BitFrameNumber);
    p.InterruptNextSOF = Some(controller::XHCI_InterruptNextSOF);
    p.EnableInterrupts = Some(controller::XHCI_EnableInterrupts);
    p.DisableInterrupts = Some(controller::XHCI_DisableInterrupts);
    p.PollController = Some(controller::XHCI_PollController);
    p.SetEndpointDataToggle = Some(endpoint::XHCI_SetEndpointDataToggle);
    p.GetEndpointStatus = Some(endpoint::XHCI_GetEndpointStatus);
    p.SetEndpointStatus = Some(endpoint::XHCI_SetEndpointStatus);
    p.RH_GetRootHubData = Some(roothub::XHCI_RH_GetRootHubData);
    p.RH_GetStatus = Some(roothub::XHCI_RH_GetStatus);
    p.RH_GetPortStatus = Some(roothub::XHCI_RH_GetPortStatus);
    p.RH_GetHubStatus = Some(roothub::XHCI_RH_GetHubStatus);
    p.RH_SetFeaturePortReset = Some(roothub::XHCI_RH_SetFeaturePortReset);
    p.RH_SetFeaturePortPower = Some(roothub::XHCI_RH_SetFeaturePortPower);
    p.RH_SetFeaturePortEnable = Some(roothub::XHCI_RH_SetFeaturePortEnable);
    p.RH_SetFeaturePortSuspend = Some(roothub::XHCI_RH_SetFeaturePortSuspend);
    p.RH_ClearFeaturePortEnable = Some(roothub::XHCI_RH_ClearFeaturePortEnable);
    p.RH_ClearFeaturePortPower = Some(roothub::XHCI_RH_ClearFeaturePortPower);
    p.RH_ClearFeaturePortSuspend = Some(roothub::XHCI_RH_ClearFeaturePortSuspend);
    p.RH_ClearFeaturePortEnableChange = Some(roothub::XHCI_RH_ClearFeaturePortEnableChange);
    p.RH_ClearFeaturePortConnectChange = Some(roothub::XHCI_RH_ClearFeaturePortConnectChange);
    p.RH_ClearFeaturePortResetChange = Some(roothub::XHCI_RH_ClearFeaturePortResetChange);
    p.RH_ClearFeaturePortSuspendChange = Some(roothub::XHCI_RH_ClearFeaturePortSuspendChange);
    p.RH_ClearFeaturePortOvercurrentChange =
        Some(roothub::XHCI_RH_ClearFeaturePortOvercurrentChange);
    p.RH_DisableIrq = Some(roothub::XHCI_RH_DisableIrq);
    p.RH_EnableIrq = Some(roothub::XHCI_RH_EnableIrq);
    p.StartSendOnePacket = Some(controller::XHCI_StartSendOnePacket);
    p.EndSendOnePacket = Some(controller::XHCI_EndSendOnePacket);
    p.PassThru = Some(controller::XHCI_PassThru);
    p.RebalanceEndpoint = Some(endpoint::XHCI_RebalanceEndpoint);
    p.FlushInterrupts = Some(controller::XHCI_FlushInterrupts);
    p.RH_ChirpRootPort = Some(roothub::XHCI_RH_ChirpRootPort);
    p.TakePortControl = Some(controller::XHCI_TakePortControl);

    USBPORT_RegisterUSBPortDriver(DriverObject, USB20_MINIPORT_INTERFACE_VERSION, p)
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
