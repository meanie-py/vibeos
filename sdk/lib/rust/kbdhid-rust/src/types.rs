// types.rs — KBDHID device-extension struct + local USAGE_AND_PAGE type.
// Translated from drivers/hid/kbdhid/kbdhid.h

use ntkrnl_sys::base::{ULONG, USHORT, UCHAR, PVOID, PCHAR};
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP, PFILE_OBJECT, PMDL, KEVENT};
use ntkrnl_sys::ddk::{
    KEYBOARD_ATTRIBUTES, KEYBOARD_INDICATOR_PARAMETERS, KEYBOARD_TYPEMATIC_PARAMETERS,
};
use hid_sys::{PHIDP_PREPARSED_DATA, HIDP_KEYBOARD_MODIFIER_STATE};

// ============================================================================
// USAGE / USAGE_AND_PAGE — defined in hidpi.h but absent from hid_sys bindgen.
// CONTRACT: define locally.
// ============================================================================
pub type USAGE = u16;
pub type PUSAGE = *mut USAGE;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct USAGE_AND_PAGE {
    pub Usage: USAGE,
    pub UsagePage: USAGE,
}
pub type PUSAGE_AND_PAGE = *mut USAGE_AND_PAGE;

// ============================================================================
// LOCAL_KEYBOARD_INDICATOR_TRANSLATION — defined at file scope in kbdhid.c.
// "This structure starts with the same layout as KEYBOARD_INDICATOR_TRANSLATION"
// ============================================================================
#[repr(C)]
#[derive(Clone, Copy)]
pub struct INDICATOR_LIST {
    pub MakeCode: USHORT,
    pub IndicatorFlags: USHORT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LOCAL_KEYBOARD_INDICATOR_TRANSLATION {
    pub NumberOfIndicatorKeys: USHORT,
    pub IndicatorList: [INDICATOR_LIST; 3],
}

// ============================================================================
// KBDHID_DEVICE_EXTENSION
// ============================================================================
#[repr(C)]
pub struct KBDHID_DEVICE_EXTENSION {
    /// lower device object
    pub NextDeviceObject: PDEVICE_OBJECT,
    /// irp which is used for reading input reports
    pub Irp: PIRP,
    /// event
    pub ReadCompletionEvent: KEVENT,
    /// device object for class callback
    pub ClassDeviceObject: PDEVICE_OBJECT,
    /// class callback
    pub ClassService: PVOID,
    /// buffer for the four usage lists below
    pub UsageListBuffer: PVOID,
    /// usage list length
    pub UsageListLength: USHORT,
    // (C struct padding between USHORT and next pointer reproduced by repr(C))
    /// current usage list
    pub CurrentUsageList: PUSAGE_AND_PAGE,
    /// previous usage list
    pub PreviousUsageList: PUSAGE_AND_PAGE,
    /// removed usage item list
    pub BreakUsageList: PUSAGE_AND_PAGE,
    /// new item usage list
    pub MakeUsageList: PUSAGE_AND_PAGE,
    /// preparsed data
    pub PreparsedData: PHIDP_PREPARSED_DATA,
    /// mdl for reading input report
    pub ReportMDL: PMDL,
    /// input report buffer
    pub Report: PCHAR,
    /// input report length
    pub ReportLength: ULONG,
    /// file object the device is reading reports from
    pub FileObject: PFILE_OBJECT,
    /// report read is active
    pub ReadReportActive: UCHAR,
    /// stop reading flag
    pub StopReadReport: UCHAR,
    /// keyboard attributes
    pub Attributes: KEYBOARD_ATTRIBUTES,
    /// keyboard modifier state
    pub ModifierState: HIDP_KEYBOARD_MODIFIER_STATE,
    /// keyboard indicator state
    pub KeyboardIndicator: KEYBOARD_INDICATOR_PARAMETERS,
    /// keyboard type matic
    pub KeyboardTypematic: KEYBOARD_TYPEMATIC_PARAMETERS,
}
pub type PKBDHID_DEVICE_EXTENSION = *mut KBDHID_DEVICE_EXTENSION;
