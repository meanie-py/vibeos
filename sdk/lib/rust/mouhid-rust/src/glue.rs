// glue.rs — tag constants, IOCTL codes, HID usage constants,
//           HidP_* extern declarations (link from hidparse.sys at final link),
//           and HID_COLLECTION_INFORMATION local struct.
// Translated from drivers/hid/mouhid/mouhid.h + hidclass.h + hidusage.h + kbdmou.h + ntddmou.h

use ntkrnl_sys::base::*;
pub const STATUS_MORE_PROCESSING_REQUIRED: NTSTATUS = 0xC0000016u32 as i32;
use ntkrnl_sys::driver_ddk::ctl_code;
use ntkrnl_sys::driver_ddk::{
    FILE_DEVICE_KEYBOARD, FILE_DEVICE_MOUSE,
    METHOD_NEITHER, METHOD_BUFFERED, METHOD_OUT_DIRECT, FILE_ANY_ACCESS,
};
use hid_sys::{HIDP_CAPS, HIDP_VALUE_CAPS, PHIDP_PREPARSED_DATA};
use hid_sys::{HidP_Input, HidP_Output, HidP_Feature};

use crate::types::{USAGE, PUSAGE};

// ---------------------------------------------------------------------------
// Pool tag — mouhid.h: #define MOUHID_TAG 'diHM'
// Character literals in little-endian: 'M'=0x4D 'H'=0x48 'i'=0x69 'd'=0x64
// The C literal 'diHM' is a multi-char constant stored as 0x6469484D
// ---------------------------------------------------------------------------
pub const MOUHID_TAG: u32 = 0x6469_484D; // 'diHM'

// ---------------------------------------------------------------------------
// Mouse geometry constants — mouhid.h
// ---------------------------------------------------------------------------
pub const WHEEL_DELTA: i32 = 120;
pub const VIRTUAL_SCREEN_SIZE_X: u32 = 65536;
pub const VIRTUAL_SCREEN_SIZE_Y: u32 = 65536;

// ---------------------------------------------------------------------------
// MouseIdentifier constants — ntddmou.h
// ---------------------------------------------------------------------------
pub const MOUSE_HID_HARDWARE: u16      = 0x0080;
pub const WHEELMOUSE_HID_HARDWARE: u16 = 0x0100;

// ---------------------------------------------------------------------------
// MOUSE_INPUT_DATA.ButtonFlags — ntddmou.h
// ---------------------------------------------------------------------------
pub const MOUSE_LEFT_BUTTON_DOWN:   u16 = 0x0001;
pub const MOUSE_LEFT_BUTTON_UP:     u16 = 0x0002;
pub const MOUSE_RIGHT_BUTTON_DOWN:  u16 = 0x0004;
pub const MOUSE_RIGHT_BUTTON_UP:    u16 = 0x0008;
pub const MOUSE_MIDDLE_BUTTON_DOWN: u16 = 0x0010;
pub const MOUSE_MIDDLE_BUTTON_UP:   u16 = 0x0020;
pub const MOUSE_BUTTON_4_DOWN:      u16 = 0x0040;
pub const MOUSE_BUTTON_4_UP:        u16 = 0x0080;
pub const MOUSE_BUTTON_5_DOWN:      u16 = 0x0100;
pub const MOUSE_BUTTON_5_UP:        u16 = 0x0200;
pub const MOUSE_WHEEL:              u16 = 0x0400;

// ---------------------------------------------------------------------------
// MOUSE_INPUT_DATA.Flags — ntddmou.h
// ---------------------------------------------------------------------------
pub const MOUSE_MOVE_RELATIVE: u16 = 0;
pub const MOUSE_MOVE_ABSOLUTE: u16 = 1;

// ---------------------------------------------------------------------------
// HID usage pages — hidusage.h
// #define HID_USAGE_PAGE_GENERIC  ((USAGE) 0x01)
// #define HID_USAGE_PAGE_BUTTON   ((USAGE) 0x09)
// ---------------------------------------------------------------------------
pub const HID_USAGE_PAGE_GENERIC: USAGE = 0x01;
pub const HID_USAGE_PAGE_BUTTON:  USAGE = 0x09;

// ---------------------------------------------------------------------------
// HID generic desktop usages — hidusage.h
// ---------------------------------------------------------------------------
pub const HID_USAGE_GENERIC_POINTER: USAGE = 0x01;
pub const HID_USAGE_GENERIC_MOUSE:   USAGE = 0x02;
pub const HID_USAGE_GENERIC_X:       USAGE = 0x30;
pub const HID_USAGE_GENERIC_Y:       USAGE = 0x31;
pub const HID_USAGE_GENERIC_Z:       USAGE = 0x32;
pub const HID_USAGE_GENERIC_WHEEL:   USAGE = 0x38;

// ---------------------------------------------------------------------------
// HIDP link collection unspecified — hidpi.h
// #define HIDP_LINK_COLLECTION_UNSPECIFIED  ((USHORT) 0)
// ---------------------------------------------------------------------------
pub const HIDP_LINK_COLLECTION_UNSPECIFIED: USHORT = 0;

// ---------------------------------------------------------------------------
// IOCTL codes
//
// CTL_CODE(DeviceType, Function, Method, Access):
//   (DeviceType << 16) | (Access << 14) | (Function << 2) | Method
//
// IOCTL_MOUSE_QUERY_ATTRIBUTES — ntddmou.h:
//   CTL_CODE(FILE_DEVICE_MOUSE, 0, METHOD_BUFFERED, FILE_ANY_ACCESS)
//   = (0xF << 16) | (0 << 14) | (0 << 2) | 0 = 0x000F_0000
//
// IOCTL_INTERNAL_MOUSE_CONNECT — kbdmou.h:
//   CTL_CODE(FILE_DEVICE_MOUSE, 0x0080, METHOD_NEITHER, FILE_ANY_ACCESS)
//
// IOCTL_INTERNAL_MOUSE_DISCONNECT — kbdmou.h:
//   CTL_CODE(FILE_DEVICE_MOUSE, 0x0100, METHOD_NEITHER, FILE_ANY_ACCESS)
//
// IOCTL_INTERNAL_MOUSE_ENABLE — kbdmou.h:
//   CTL_CODE(FILE_DEVICE_MOUSE, 0x0200, METHOD_NEITHER, FILE_ANY_ACCESS)
//
// IOCTL_INTERNAL_MOUSE_DISABLE — kbdmou.h:
//   CTL_CODE(FILE_DEVICE_MOUSE, 0x0400, METHOD_NEITHER, FILE_ANY_ACCESS)
//
// HID_CTL_CODE(id)        = CTL_CODE(FILE_DEVICE_KEYBOARD, id, METHOD_NEITHER, FILE_ANY_ACCESS)
// HID_BUFFER_CTL_CODE(id) = CTL_CODE(FILE_DEVICE_KEYBOARD, id, METHOD_BUFFERED, FILE_ANY_ACCESS)
// HID_OUT_CTL_CODE(id)    = CTL_CODE(FILE_DEVICE_KEYBOARD, id, METHOD_OUT_DIRECT, FILE_ANY_ACCESS)
//
// IOCTL_HID_GET_COLLECTION_DESCRIPTOR = HID_CTL_CODE(100)
// IOCTL_HID_GET_COLLECTION_INFORMATION = HID_BUFFER_CTL_CODE(106)
// IOCTL_HID_FLUSH_QUEUE               = HID_CTL_CODE(101)
// ---------------------------------------------------------------------------

pub const IOCTL_MOUSE_QUERY_ATTRIBUTES: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0, METHOD_BUFFERED, FILE_ANY_ACCESS);

pub const IOCTL_INTERNAL_MOUSE_CONNECT: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0080, METHOD_NEITHER, FILE_ANY_ACCESS);

pub const IOCTL_INTERNAL_MOUSE_DISCONNECT: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0100, METHOD_NEITHER, FILE_ANY_ACCESS);

pub const IOCTL_INTERNAL_MOUSE_ENABLE: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0200, METHOD_NEITHER, FILE_ANY_ACCESS);

pub const IOCTL_INTERNAL_MOUSE_DISABLE: u32 =
    ctl_code(FILE_DEVICE_MOUSE, 0x0400, METHOD_NEITHER, FILE_ANY_ACCESS);

pub const IOCTL_HID_GET_COLLECTION_DESCRIPTOR: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 100, METHOD_NEITHER, FILE_ANY_ACCESS);

pub const IOCTL_HID_GET_COLLECTION_INFORMATION: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 106, METHOD_BUFFERED, FILE_ANY_ACCESS);

pub const IOCTL_HID_FLUSH_QUEUE: u32 =
    ctl_code(FILE_DEVICE_KEYBOARD, 101, METHOD_NEITHER, FILE_ANY_ACCESS);

// ---------------------------------------------------------------------------
// HID_COLLECTION_INFORMATION — hidclass.h
// Not present in hid-sys bindgen (hidclass.h not included in hid-sys generation).
// ---------------------------------------------------------------------------
#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct HID_COLLECTION_INFORMATION {
    pub DescriptorSize: ULONG,
    pub Polled:         BOOLEAN,
    pub Reserved1:      [UCHAR; 1],
    pub VendorID:       USHORT,
    pub ProductID:      USHORT,
    pub VersionNumber:  USHORT,
}
pub type PHID_COLLECTION_INFORMATION = *mut HID_COLLECTION_INFORMATION;

// ---------------------------------------------------------------------------
// PSERVICE_CALLBACK_ROUTINE function type — kbdmou.h
// typedef VOID (STDAPICALLTYPE *PSERVICE_CALLBACK_ROUTINE)(
//   IN PVOID NormalContext,
//   IN PVOID SystemArgument1,
//   IN PVOID SystemArgument2,
//   IN OUT PVOID SystemArgument3);
//
// STDAPICALLTYPE is __stdcall on i386, __cdecl on amd64/arm64.
// "system" resolves correctly per arch.
// ---------------------------------------------------------------------------
pub type PSERVICE_CALLBACK_ROUTINE = unsafe extern "system" fn(
    NormalContext:   PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
    SystemArgument3: *mut PVOID,
);

// ---------------------------------------------------------------------------
// HidP_* extern declarations — link from hidparse.sys at final link.
// These are NOT in any crate; declared with extern "system" for correct ABI.
// ---------------------------------------------------------------------------
extern "system" {
    pub fn HidP_GetCaps(
        PreparsedData: PHIDP_PREPARSED_DATA,
        Capabilities:  *mut HIDP_CAPS,
    ) -> NTSTATUS;

    pub fn HidP_GetUsages(
        ReportType:            u32,  // HIDP_REPORT_TYPE
        UsagePage:             USAGE,
        LinkCollection:        USHORT,
        UsageList:             PUSAGE,
        UsageLength:           *mut ULONG,
        PreparsedData:         PHIDP_PREPARSED_DATA,
        Report:                *mut CHAR,
        ReportLength:          ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetUsageValue(
        ReportType:     u32,
        UsagePage:      USAGE,
        LinkCollection: USHORT,
        Usage:          USAGE,
        UsageValue:     *mut ULONG,
        PreparsedData:  PHIDP_PREPARSED_DATA,
        Report:         *mut CHAR,
        ReportLength:   ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetScaledUsageValue(
        ReportType:     u32,
        UsagePage:      USAGE,
        LinkCollection: USHORT,
        Usage:          USAGE,
        UsageValue:     *mut LONG,
        PreparsedData:  PHIDP_PREPARSED_DATA,
        Report:         *mut CHAR,
        ReportLength:   ULONG,
    ) -> NTSTATUS;

    pub fn HidP_GetSpecificValueCaps(
        ReportType:     u32,
        UsagePage:      USAGE,
        LinkCollection: USHORT,
        Usage:          USAGE,
        ValueCaps:      *mut HIDP_VALUE_CAPS,
        ValueCapsLength: *mut USHORT,
        PreparsedData:  PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS;

    pub fn HidP_GetValueCaps(
        ReportType:     u32,
        ValueCaps:      *mut HIDP_VALUE_CAPS,
        ValueCapsLength: *mut USHORT,
        PreparsedData:  PHIDP_PREPARSED_DATA,
    ) -> NTSTATUS;

    pub fn HidP_UsageListDifference(
        PreviousUsageList: PUSAGE,
        CurrentUsageList:  PUSAGE,
        BreakUsageList:    PUSAGE,
        MakeUsageList:     PUSAGE,
        UsageListLength:   ULONG,
    ) -> NTSTATUS;

    pub fn HidP_MaxUsageListLength(
        ReportType:    u32,
        UsagePage:     USAGE,
        PreparsedData: PHIDP_PREPARSED_DATA,
    ) -> ULONG;

    // Additional HidP_* functions per CONTRACT.md (not used directly by mouhid
    // but required in the extern block per the contract's function list):
    pub fn HidP_GetUsagesEx(
        ReportType:     u32,
        LinkCollection: USHORT,
        ButtonList:     *mut crate::types::USAGE_AND_PAGE,
        UsageLength:    *mut ULONG,
        PreparsedData:  PHIDP_PREPARSED_DATA,
        Report:         *mut CHAR,
        ReportLength:   ULONG,
    ) -> NTSTATUS;

    pub fn HidP_UsageAndPageListDifference(
        PreviousUsageList: *mut crate::types::USAGE_AND_PAGE,
        CurrentUsageList:  *mut crate::types::USAGE_AND_PAGE,
        BreakUsageList:    *mut crate::types::USAGE_AND_PAGE,
        MakeUsageList:     *mut crate::types::USAGE_AND_PAGE,
        UsageListLength:   ULONG,
    ) -> NTSTATUS;

    pub fn HidP_TranslateUsageAndPagesToI8042ScanCodes(
        ChangedUsageList: *mut crate::types::USAGE_AND_PAGE,
        UsageListLength:  ULONG,
        KeyAction:        u32,  // HIDP_KEYBOARD_DIRECTION
        ModifierState:    *mut hid_sys::HIDP_KEYBOARD_MODIFIER_STATE,
        InsertCodesProcedure: Option<unsafe extern "system" fn(
            Context: PVOID,
            NewScanCodes: *mut u8,
            Length: *mut ULONG,
        ) -> BOOLEAN>,
        InsertCodesContext: PVOID,
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
