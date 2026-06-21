// types.rs — MOUHID_DEVICE_EXTENSION + local type aliases
// Translated from drivers/hid/mouhid/mouhid.h
//
// USAGE_AND_PAGE is missing from hid-sys (bindgen can't see it); defined here
// per CONTRACT.md §"Substrate available".

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, KEVENT, PMDL, PFILE_OBJECT,
};
use hid_sys::HIDP_VALUE_CAPS;

// ---------------------------------------------------------------------------
// USAGE_AND_PAGE — hidpi.h typedef struct (missing from hid-sys bindgen)
// ---------------------------------------------------------------------------
pub type USAGE = u16;
pub type PUSAGE = *mut USAGE;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct USAGE_AND_PAGE {
    pub Usage:     USAGE,
    pub UsagePage: USAGE,
}
pub type PUSAGE_AND_PAGE = *mut USAGE_AND_PAGE;

// ---------------------------------------------------------------------------
// MOUHID_DEVICE_EXTENSION — mouhid.h typedef struct
// ---------------------------------------------------------------------------
#[repr(C)]
pub struct MOUHID_DEVICE_EXTENSION {
    /// lower device object
    pub NextDeviceObject: PDEVICE_OBJECT,

    /// irp which is used for reading input reports
    pub Irp: PIRP,

    /// event (signals when read loop has terminated)
    pub ReadCompletionEvent: KEVENT,

    /// device object for class callback
    pub ClassDeviceObject: PDEVICE_OBJECT,

    /// class callback (PSERVICE_CALLBACK_ROUTINE stored as PVOID)
    pub ClassService: PVOID,

    /// mouse type (MOUSE_HID_HARDWARE or WHEELMOUSE_HID_HARDWARE)
    pub MouseIdentifier: USHORT,

    /// wheel usage page
    pub WheelUsagePage: USHORT,

    /// buffer for the four usage lists below (single allocation)
    pub UsageListBuffer: PVOID,

    /// usage list length (max buttons)
    pub UsageListLength: USHORT,

    /// current usage list
    pub CurrentUsageList: PUSAGE,

    /// previous usage list
    pub PreviousUsageList: PUSAGE,

    /// removed usage item list
    pub BreakUsageList: PUSAGE,

    /// new item usage list
    pub MakeUsageList: PUSAGE,

    /// preparsed HID data (PHIDP_PREPARSED_DATA stored as PVOID)
    pub PreparsedData: PVOID,

    /// MDL for reading input report
    pub ReportMDL: PMDL,

    /// input report buffer
    pub Report: *mut CHAR,

    /// input report length
    pub ReportLength: ULONG,

    /// file object the device is reading reports from
    pub FileObject: PFILE_OBJECT,

    /// read is active flag
    pub ReadReportActive: UCHAR,

    /// stop reading flag
    pub StopReadReport: UCHAR,

    /// mouse operates in absolute mode
    pub MouseAbsolute: UCHAR,

    /// value caps for X axis
    pub ValueCapsX: HIDP_VALUE_CAPS,

    /// value caps for Y axis
    pub ValueCapsY: HIDP_VALUE_CAPS,
}

pub type PMOUHID_DEVICE_EXTENSION = *mut MOUHID_DEVICE_EXTENSION;
