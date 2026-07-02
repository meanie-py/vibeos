//! usbaudio-rust glue: bare-named enum-member aliases (bindgen names them `<EnumTag>_<member>`)
//! + type aliases + #define consts the DeepSeek translation references bare. Arch-independent.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code)]
use crate::types::*;
use ntkrnl_sys::base::{PVOID, ULONG, NTSTATUS, BOOLEAN};

pub const NULL: PVOID = core::ptr::null_mut();

// kernel/Windows types referenced bare (not in the per-file ddk imports)
pub use ntkrnl_sys::ddk::{_UNICODE_STRING, IO_STACK_LOCATION};
pub use ntkrnl_sys::ddk::_DEVICE_POWER_STATE as DEVICE_POWER_STATE;
pub use ntkrnl_sys::ddk::_SYSTEM_POWER_STATE as SYSTEM_POWER_STATE;
pub use crate::types::_URB_CONTROL_DESCRIPTOR_REQUEST as URB_CONTROL_DESCRIPTOR_REQUEST;
pub type DWORD = u32;
pub type WORD = u16;
pub type PCSTR = *const i8;

// #define constants not emitted as bare bindgen consts (values from sdk/include; verify_constants confirms)
pub const IOCTL_INTERNAL_USB_SUBMIT_URB: ULONG = 0x220003;
pub const KEY_ALL_ACCESS: ULONG = 0xF003F;
pub const KSFILTER_DESCRIPTOR_VERSION: ULONG = 0xFFFFFFFF;
pub const KSFILTER_NODE: ULONG = 0xFFFFFFFF;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as i32;
pub const STATUS_NO_MATCH: NTSTATUS = 0xC0000272u32 as i32;
pub const USBD_STATUS_INSUFFICIENT_RESOURCES: ULONG = 0xC0001000;
pub const WAVE_FORMAT_PCM: ULONG = 1;
pub const USBAUDIO_TAG: ULONG = 0x41627355;

// ---- auto-generated enum-member aliases ----
pub use crate::types::KSINTERFACE_STANDARD_KSINTERFACE_STANDARD_STREAMING as KSINTERFACE_STANDARD_STREAMING;
pub use crate::types::KSPIN_COMMUNICATION_KSPIN_COMMUNICATION_BOTH as KSPIN_COMMUNICATION_BOTH;
pub use crate::types::KSPIN_COMMUNICATION_KSPIN_COMMUNICATION_BRIDGE as KSPIN_COMMUNICATION_BRIDGE;
pub use crate::types::KSPIN_COMMUNICATION_KSPIN_COMMUNICATION_SINK as KSPIN_COMMUNICATION_SINK;
pub use crate::types::KSPIN_DATAFLOW_KSPIN_DATAFLOW_IN as KSPIN_DATAFLOW_IN;
pub use crate::types::KSPIN_DATAFLOW_KSPIN_DATAFLOW_OUT as KSPIN_DATAFLOW_OUT;
pub use crate::types::KSPROPERTY_AUDIO_KSPROPERTY_AUDIO_MUTE as KSPROPERTY_AUDIO_MUTE;
pub use crate::types::KSPROPERTY_AUDIO_KSPROPERTY_AUDIO_VOLUMELEVEL as KSPROPERTY_AUDIO_VOLUMELEVEL;
pub use crate::types::KSSTATE_KSSTATE_PAUSE as KSSTATE_PAUSE;
pub use crate::types::KSSTATE_KSSTATE_RUN as KSSTATE_RUN;
pub use crate::types::KSSTREAM_POINTER_STATE_KSSTREAM_POINTER_STATE_LOCKED as KSSTREAM_POINTER_STATE_LOCKED;

// FORCEINLINE DDK helpers (no real export) referenced as externs by the translation —
// #[no_mangle] wrappers over the ntkrnl-sys inline impls (mirrors usbport-rust glue).
#[no_mangle]
pub unsafe extern "C" fn KeInitializeSpinLock(SpinLock: *mut usize) {
    *SpinLock = 0;
}
#[no_mangle]
pub unsafe extern "C" fn IoGetNextIrpStackLocation(
    Irp: ntkrnl_sys::ddk::PIRP,
) -> ntkrnl_sys::ddk::PIO_STACK_LOCATION {
    ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation(Irp)
}
#[no_mangle]
pub unsafe extern "C" fn IoSetCompletionRoutine(
    Irp: ntkrnl_sys::ddk::PIRP,
    CompletionRoutine: ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    InvokeOnSuccess: BOOLEAN,
    InvokeOnError: BOOLEAN,
    InvokeOnCancel: BOOLEAN,
) {
    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        Irp, CompletionRoutine, Context, InvokeOnSuccess, InvokeOnError, InvokeOnCancel,
    )
}
