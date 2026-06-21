// glue.rs — Tag constants, IOCTL codes, HidP_* extern block, misc externs.
// Translated from drivers/hid/kbdhid/kbdhid.h + hidpi.h + hidclass.h + ntddkbd.h + kbdmou.h

use ntkrnl_sys::base::{NTSTATUS, ULONG, UCHAR, PVOID, BOOLEAN};
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP, PMDL};
use ntkrnl_sys::driver_ddk::{PCONNECT_DATA, ctl_code, FILE_DEVICE_KEYBOARD, METHOD_BUFFERED, METHOD_NEITHER, FILE_ANY_ACCESS};
use hid_sys::{
    PHIDP_PREPARSED_DATA, HIDP_CAPS, HIDP_REPORT_TYPE,
    HIDP_KEYBOARD_DIRECTION, PHIDP_KEYBOARD_MODIFIER_STATE,
};
use crate::types::{PUSAGE_AND_PAGE, USAGE, PUSAGE};

// ============================================================================
// Pool tag — 'diHK' = 0x4B486964
// 'd' = 0x64, 'i' = 0x69, 'H' = 0x48, 'K' = 0x4B → little-endian u32
// ============================================================================
pub const KBDHID_TAG: ULONG = u32::from_le_bytes(*b"diHK");

// ============================================================================
// Keyboard typematic defaults (from kbdmou.h / kbfiltr.h)
// ============================================================================
pub const KEYBOARD_TYPEMATIC_RATE_MINIMUM: u16 = 2;
pub const KEYBOARD_TYPEMATIC_RATE_MAXIMUM: u16 = 30;
pub const KEYBOARD_TYPEMATIC_RATE_DEFAULT: u16 = 30;
pub const KEYBOARD_TYPEMATIC_DELAY_MINIMUM: u16 = 250;
pub const KEYBOARD_TYPEMATIC_DELAY_MAXIMUM: u16 = 1000;
pub const KEYBOARD_TYPEMATIC_DELAY_DEFAULT: u16 = 250;

// ============================================================================
// Keyboard type constants (from kbdhid.h "FIXME: write kbd.h")
// ============================================================================
pub const MICROSOFT_KBD_FUNC: u16      = 12;
pub const KEYBOARD_TYPE_UNKNOWN: u8    = 0x51;
pub const MICROSOFT_KBD_101_TYPE: u8   = 0;

// ============================================================================
// NTSTATUS values not yet in ntkrnl_sys::constants
// ============================================================================
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016_u32 as i32;

// ============================================================================
// LED indicator flags (ntddkbd.h)
// ============================================================================
pub const KEYBOARD_CAPS_LOCK_ON: u16   = 4;
pub const KEYBOARD_NUM_LOCK_ON: u16    = 2;
pub const KEYBOARD_SCROLL_LOCK_ON: u16 = 1;

// ============================================================================
// KEYBOARD_INPUT_DATA.Flags constants (ntddkbd.h)
// ============================================================================
pub const KEY_MAKE: u16  = 0;
pub const KEY_BREAK: u16 = 1;
pub const KEY_E0: u16    = 2;
pub const KEY_E1: u16    = 4;

// ============================================================================
// HID usage page for keyboard (hidusage.h)
// ============================================================================
pub const HID_USAGE_PAGE_KEYBOARD: USAGE = 0x07;

// ============================================================================
// IOCTL codes — computed via CTL_CODE macro from ntddkbd.h + kbdmou.h + hidclass.h
// FILE_DEVICE_KEYBOARD = 0x0B
// ============================================================================

// ntddkbd.h public IOCTLs (METHOD_BUFFERED)
pub const IOCTL_KEYBOARD_QUERY_ATTRIBUTES: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0000, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_INDICATORS: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0010, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_INDICATOR_TRANSLATION: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0020, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_QUERY_TYPEMATIC: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0008, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_SET_TYPEMATIC: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0001, METHOD_BUFFERED, FILE_ANY_ACCESS);
pub const IOCTL_KEYBOARD_SET_INDICATORS: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0002, METHOD_BUFFERED, FILE_ANY_ACCESS);

// kbdmou.h internal IOCTLs (METHOD_NEITHER)
pub const IOCTL_INTERNAL_KEYBOARD_CONNECT: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0080, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_KEYBOARD_DISCONNECT: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0100, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_KEYBOARD_ENABLE: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0200, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_INTERNAL_KEYBOARD_DISABLE: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 0x0400, METHOD_NEITHER, FILE_ANY_ACCESS);

// hidclass.h HID IOCTLs
// HID_CTL_CODE(id)        = CTL_CODE(FILE_DEVICE_KEYBOARD, id, METHOD_NEITHER,  FILE_ANY_ACCESS)
// HID_BUFFER_CTL_CODE(id) = CTL_CODE(FILE_DEVICE_KEYBOARD, id, METHOD_BUFFERED, FILE_ANY_ACCESS)
pub const IOCTL_HID_FLUSH_QUEUE: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 101, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_HID_GET_COLLECTION_DESCRIPTOR: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 100, METHOD_NEITHER, FILE_ANY_ACCESS);
pub const IOCTL_HID_GET_COLLECTION_INFORMATION: ULONG =
    ctl_code(FILE_DEVICE_KEYBOARD, 106, METHOD_BUFFERED, FILE_ANY_ACCESS);

// ============================================================================
// HID_COLLECTION_INFORMATION (hidclass.h) — not in hid_sys substrate
// ============================================================================
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct HID_COLLECTION_INFORMATION {
    pub DescriptorSize: ULONG,
    pub Polled: BOOLEAN,
    pub Reserved1: [UCHAR; 1],
    pub VendorID: u16,
    pub ProductID: u16,
    pub VersionNumber: u16,
}
pub type PHID_COLLECTION_INFORMATION = *mut HID_COLLECTION_INFORMATION;

// ============================================================================
// PoSetPowerState — exported by ntoskrnl, absent from ntkrnl_sys bindgen.
// _POWER_STATE is a union; for DevicePowerState the DeviceState field is used.
// We pass the union as a usize (pointer-width) since on all arches the union
// is pointer-sized (enum value lives in lowest word).
// ============================================================================
extern "system" {
    pub fn PoSetPowerState(
        DeviceObject: PDEVICE_OBJECT,
        Type: u32,          // POWER_STATE_TYPE — DevicePowerState = 1
        State: usize,       // POWER_STATE union passed by value as usize
    ) -> usize;             // returns POWER_STATE (unused)
}

// POWER_STATE_TYPE enum values (ntpoapi.h)
pub const DevicePowerState: u32 = 1;
// DEVICE_POWER_STATE enum values
pub const PowerDeviceD0: usize = 1;

// ============================================================================
// RtlMoveMemory — exported by ntoskrnl, absent from ntkrnl_sys::rtl.
// ============================================================================
extern "system" {
    pub fn RtlMoveMemory(Destination: PVOID, Source: *const core::ffi::c_void, Length: usize);
}

// ============================================================================
// HidP_* extern block — link from hidparse.sys at final link.
// Prototypes from hidpi.h / hidpddi.h.
// ============================================================================
extern "system" {
    pub fn HidP_GetCaps(
        PreparsedData: PHIDP_PREPARSED_DATA,
        Capabilities: *mut HIDP_CAPS,
    ) -> NTSTATUS;

    pub fn HidP_MaxUsageListLength(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> ULONG;

    pub fn HidP_GetUsagesEx(
        ReportType: HIDP_REPORT_TYPE,
        LinkCollection: u16,
        ButtonList: PUSAGE_AND_PAGE,
        UsageLength: *mut ULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: *mut i8,
        ReportLength: ULONG,
    ) -> NTSTATUS;

    pub fn HidP_UsageAndPageListDifference(
        PreviousUsageList: PUSAGE_AND_PAGE,
        CurrentUsageList: PUSAGE_AND_PAGE,
        BreakUsageList: PUSAGE_AND_PAGE,
        MakeUsageList: PUSAGE_AND_PAGE,
        UsageListLength: ULONG,
    ) -> NTSTATUS;

    pub fn HidP_TranslateUsageAndPagesToI8042ScanCodes(
        ChangedUsageList: PUSAGE_AND_PAGE,
        UsageListLength: ULONG,
        KeyAction: HIDP_KEYBOARD_DIRECTION,
        ModifierState: PHIDP_KEYBOARD_MODIFIER_STATE,
        InsertCodesProcedure: Option<
            unsafe extern "system" fn(
                Context: PVOID,
                NewScanCodes: *mut i8,
                Length: ULONG,
            ) -> BOOLEAN
        >,
        InsertCodesContext: PVOID,
    ) -> NTSTATUS;

    // Listed in CONTRACT for completeness; not called by kbdhid.c directly
    // but declared per contract requirement.
    pub fn HidP_GetUsages(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: u16,
        UsageList: PUSAGE,
        UsageLength: *mut ULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: *mut i8,
        ReportLength: ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetScaledUsageValue(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: u16,
        Usage: USAGE,
        UsageValue: *mut i32,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: *mut i8,
        ReportLength: ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetUsageValue(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: u16,
        Usage: USAGE,
        UsageValue: *mut ULONG,
        PreparsedData: PHIDP_PREPARSED_DATA,
        Report: *mut i8,
        ReportLength: ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetSpecificValueCaps(
        ReportType: HIDP_REPORT_TYPE,
        UsagePage: USAGE,
        LinkCollection: u16,
        Usage: USAGE,
        ValueCaps: *mut hid_sys::HIDP_VALUE_CAPS,
        ValueCapsLength: *mut u16,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS;

    pub fn HidP_UsageListDifference(
        PreviousUsageList: PUSAGE,
        CurrentUsageList: PUSAGE,
        BreakUsageList: PUSAGE,
        MakeUsageList: PUSAGE,
        UsageListLength: ULONG,
    ) -> NTSTATUS;
}


// --- driver-safe IRQL raise-to-DISPATCH / lower (arch-dispatched) ---
// ntoskrnl/hal export the raise/lower primitives to *drivers* on i386 (ntoskrnl fastcall
// KfRaiseIrql) and arm64 (hal KeRaiseIrql/KeLowerIrql), but NOT on amd64 — there neither
// ntoskrnl nor hal exports them, so a driver must use the self-contained CR8 asm. The tier1
// KeRaiseIrqlInternal/KeLowerIrqlInternal reference kernel-internal KeRaiseIrql/KeLowerIrql
// that only resolve when linked *into* ntoskrnl, so amd64 cohab drivers must not use them.
#[inline]
pub unsafe fn KeRaiseIrqlInternal(new_irql: ntkrnl_sys::base::KIRQL, old_irql: *mut ntkrnl_sys::base::KIRQL) {
    #[cfg(not(target_arch = "x86_64"))]
    { ntkrnl_sys::inline::tier1::KeRaiseIrqlInternal(new_irql, old_irql); }
    #[cfg(target_arch = "x86_64")]
    { *old_irql = ntkrnl_sys::inline::ke::KfRaiseIrql_amd64(new_irql); }
}
#[inline]
pub unsafe fn KeLowerIrqlInternal(new_irql: ntkrnl_sys::base::KIRQL) {
    #[cfg(not(target_arch = "x86_64"))]
    { ntkrnl_sys::inline::tier1::KeLowerIrqlInternal(new_irql); }
    #[cfg(target_arch = "x86_64")]
    { ntkrnl_sys::inline::ke::KeLowerIrql_amd64(new_irql); }
}
