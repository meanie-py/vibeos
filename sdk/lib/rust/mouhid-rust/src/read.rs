// read.rs — MouHid_InitiateRead, MouHid_ReadCompletion,
//            MouHid_CreateCompletion, MouHid_StartDeviceCompletion,
//            MouHid_GetButtonMove, MouHid_GetButtonFlags,
//            MouHid_DispatchInputData
// Translated from drivers/hid/mouhid/mouhid.c

use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{IoGetNextIrpStackLocation, IoSetCompletionRoutine};
use ntkrnl_sys::ddk::{
    PDEVICE_OBJECT, PIRP, PRKEVENT,
    IoReuseIrp,
    KeSetEvent,
    _EVENT_TYPE_NotificationEvent as NotificationEvent,
    _IO_STACK_LOCATION,
};
use crate::glue::{KeRaiseIrqlInternal as KeRaiseIrql, KeLowerIrqlInternal as KeLowerIrql};
use ntkrnl_sys::base::KIRQL;
use hid_sys::{
    HIDP_STATUS_SUCCESS,
    HIDP_STATUS_BAD_LOG_PHY_VALUES,
    HidP_Input,
};

macro_rules! ASSERT {
    ($e:expr) => { ntkrnl_sys::ASSERT!($e) };
}

use crate::types::{PMOUHID_DEVICE_EXTENSION, PUSAGE, USAGE};
use crate::glue::*;
use crate::glue::{MOUSE_LEFT_BUTTON_DOWN, MOUSE_LEFT_BUTTON_UP, MOUSE_RIGHT_BUTTON_DOWN, MOUSE_RIGHT_BUTTON_UP, MOUSE_MIDDLE_BUTTON_DOWN, MOUSE_MIDDLE_BUTTON_UP, MOUSE_BUTTON_4_DOWN, MOUSE_BUTTON_4_UP, MOUSE_BUTTON_5_DOWN, MOUSE_BUTTON_5_UP, MOUSE_WHEEL, MOUSE_MOVE_RELATIVE, MOUSE_MOVE_ABSOLUTE};

// MouHid_ButtonUpFlags[1..5] — "up" flags indexed by button number (1-based).
// Index 0 is 0xFF (unused), index 1 = MOUSE_LEFT_BUTTON_DOWN, etc.
// Note: the C naming is confusing — "ButtonUpFlags" is used for BreakUsageList
// (buttons released), which maps to DOWN flags (the "up" event clears the
// down state). This matches the C array exactly.
static MouHid_ButtonUpFlags: [u16; 6] = [
    0xFF,                    /* unused */
    MOUSE_LEFT_BUTTON_DOWN,
    MOUSE_RIGHT_BUTTON_DOWN,
    MOUSE_MIDDLE_BUTTON_DOWN,
    MOUSE_BUTTON_4_DOWN,
    MOUSE_BUTTON_5_DOWN,
];

// MouHid_ButtonDownFlags[1..5] — used for MakeUsageList (buttons pressed).
static MouHid_ButtonDownFlags: [u16; 6] = [
    0xFF,                   /* unused */
    MOUSE_LEFT_BUTTON_UP,
    MOUSE_RIGHT_BUTTON_UP,
    MOUSE_MIDDLE_BUTTON_UP,
    MOUSE_BUTTON_4_UP,
    MOUSE_BUTTON_5_UP,
];

// ---------------------------------------------------------------------------
// MouHid_GetButtonMove
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_GetButtonMove(
    DeviceExtension: PMOUHID_DEVICE_EXTENSION,
    LastX: *mut LONG,
    LastY: *mut LONG,
) {
    let mut Status: NTSTATUS;
    let mut ValueX: ULONG = 0;
    let mut ValueY: ULONG = 0;

    // init result
    *LastX = 0;
    *LastY = 0;

    let ext = &mut *DeviceExtension;

    if ext.MouseAbsolute == 0 {
        // get scaled usage value x
        Status = HidP_GetScaledUsageValue(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_X,
            LastX,
            ext.PreparsedData as _,
            ext.Report,
            ext.ReportLength,
        );

        if Status != HIDP_STATUS_SUCCESS {
            // FIXME: handle more errors
            if Status == HIDP_STATUS_BAD_LOG_PHY_VALUES {
                // FIXME: assume it operates in absolute mode
                ext.MouseAbsolute = TRUE as UCHAR;

                // get unscaled value
                Status = HidP_GetUsageValue(
                    HidP_Input,
                    HID_USAGE_PAGE_GENERIC,
                    HIDP_LINK_COLLECTION_UNSPECIFIED,
                    HID_USAGE_GENERIC_X,
                    &mut ValueX,
                    ext.PreparsedData as _,
                    ext.Report,
                    ext.ReportLength,
                );

                // FIXME handle error
                ASSERT!(Status == HIDP_STATUS_SUCCESS);

                // absolute pointing devices values need be in range 0 - 0xffff
                ASSERT!((*DeviceExtension).ValueCapsX.LogicalMax > 0);
                ASSERT!((*DeviceExtension).ValueCapsX.LogicalMax > (*DeviceExtension).ValueCapsX.LogicalMin);

                // convert to logical range
                *LastX = (ValueX * VIRTUAL_SCREEN_SIZE_X) as LONG
                    / (*DeviceExtension).ValueCapsX.LogicalMax;
            }
        }
    } else {
        // get unscaled value
        Status = HidP_GetUsageValue(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_X,
            &mut ValueX,
            ext.PreparsedData as _,
            ext.Report,
            ext.ReportLength,
        );

        // FIXME handle error
        ASSERT!(Status == HIDP_STATUS_SUCCESS);

        // absolute pointing devices values need be in range 0 - 0xffff
        ASSERT!((*DeviceExtension).ValueCapsX.LogicalMax > 0);
        ASSERT!((*DeviceExtension).ValueCapsX.LogicalMax > (*DeviceExtension).ValueCapsX.LogicalMin);

        // convert to logical range
        *LastX = (ValueX * VIRTUAL_SCREEN_SIZE_X) as LONG
            / (*DeviceExtension).ValueCapsX.LogicalMax;
    }

    // Re-read after potential MouseAbsolute change (same as C re-checks the flag)
    let ext = &mut *DeviceExtension;

    if ext.MouseAbsolute == 0 {
        // get scaled usage value y
        Status = HidP_GetScaledUsageValue(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_Y,
            LastY,
            ext.PreparsedData as _,
            ext.Report,
            ext.ReportLength,
        );

        if Status != HIDP_STATUS_SUCCESS {
            // FIXME: handle more errors
            if Status == HIDP_STATUS_BAD_LOG_PHY_VALUES {
                // assume it operates in absolute mode
                ext.MouseAbsolute = TRUE as UCHAR;

                // get unscaled value
                Status = HidP_GetUsageValue(
                    HidP_Input,
                    HID_USAGE_PAGE_GENERIC,
                    HIDP_LINK_COLLECTION_UNSPECIFIED,
                    HID_USAGE_GENERIC_Y,
                    &mut ValueY,
                    ext.PreparsedData as _,
                    ext.Report,
                    ext.ReportLength,
                );

                // FIXME handle error
                ASSERT!(Status == HIDP_STATUS_SUCCESS);

                // absolute pointing devices values need be in range 0 - 0xffff
                ASSERT!((*DeviceExtension).ValueCapsY.LogicalMax > 0);
                ASSERT!((*DeviceExtension).ValueCapsY.LogicalMax > (*DeviceExtension).ValueCapsY.LogicalMin);

                // convert to logical range
                *LastY = (ValueY * VIRTUAL_SCREEN_SIZE_Y) as LONG
                    / (*DeviceExtension).ValueCapsY.LogicalMax;
            }
        }
    } else {
        // get unscaled value
        Status = HidP_GetUsageValue(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_Y,
            &mut ValueY,
            ext.PreparsedData as _,
            ext.Report,
            ext.ReportLength,
        );

        // FIXME handle error
        ASSERT!(Status == HIDP_STATUS_SUCCESS);

        // absolute pointing devices values need be in range 0 - 0xffff
        ASSERT!((*DeviceExtension).ValueCapsY.LogicalMax > 0);
        ASSERT!((*DeviceExtension).ValueCapsY.LogicalMax > (*DeviceExtension).ValueCapsY.LogicalMin);

        // convert to logical range
        *LastY = (ValueY * VIRTUAL_SCREEN_SIZE_Y) as LONG
            / (*DeviceExtension).ValueCapsY.LogicalMax;
    }
}

// ---------------------------------------------------------------------------
// MouHid_GetButtonFlags
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_GetButtonFlags(
    DeviceExtension: PMOUHID_DEVICE_EXTENSION,
    ButtonFlags: *mut USHORT,
    Flags: *mut USHORT,
) {
    let mut Status: NTSTATUS;
    let mut Usage: USAGE;
    let mut Index: ULONG;
    let mut TempList: PUSAGE;
    let mut CurrentUsageListLength: ULONG;

    // init flags
    *ButtonFlags = 0;
    *Flags = 0;

    let ext = &mut *DeviceExtension;

    // get usages
    CurrentUsageListLength = ext.UsageListLength as ULONG;
    Status = HidP_GetUsages(
        HidP_Input,
        HID_USAGE_PAGE_BUTTON,
        HIDP_LINK_COLLECTION_UNSPECIFIED,
        ext.CurrentUsageList,
        &mut CurrentUsageListLength,
        ext.PreparsedData as _,
        ext.Report,
        ext.ReportLength,
    );
    if Status != HIDP_STATUS_SUCCESS {
        // DPRINT1
        return;
    }

    // extract usage list difference
    Status = HidP_UsageListDifference(
        ext.PreviousUsageList,
        ext.CurrentUsageList,
        ext.BreakUsageList,
        ext.MakeUsageList,
        ext.UsageListLength as ULONG,
    );
    if Status != HIDP_STATUS_SUCCESS {
        // DPRINT1
        return;
    }

    if ext.UsageListLength != 0 {
        Index = 0;
        loop {
            // get usage
            Usage = *ext.BreakUsageList.add(Index as usize);
            if Usage == 0 {
                break;
            }

            if Usage <= 5 {
                // max 5 buttons supported
                *ButtonFlags |= MouHid_ButtonDownFlags[Usage as usize];
            }

            // move to next index
            Index += 1;
            if Index >= ext.UsageListLength as ULONG {
                break;
            }
        }
    }

    if ext.UsageListLength != 0 {
        Index = 0;
        loop {
            // get usage
            Usage = *ext.MakeUsageList.add(Index as usize);
            if Usage == 0 {
                break;
            }

            if Usage <= 5 {
                // max 5 buttons supported
                *ButtonFlags |= MouHid_ButtonUpFlags[Usage as usize];
            }

            // move to next index
            Index += 1;
            if Index >= ext.UsageListLength as ULONG {
                break;
            }
        }
    }

    // now switch the previous list with current list
    TempList = ext.CurrentUsageList;
    ext.CurrentUsageList = ext.PreviousUsageList;
    ext.PreviousUsageList = TempList;

    if ext.MouseAbsolute != 0 {
        // mouse operates absolute
        *Flags |= MOUSE_MOVE_ABSOLUTE;
    }
}

// ---------------------------------------------------------------------------
// MouHid_DispatchInputData
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_DispatchInputData(
    DeviceExtension: PMOUHID_DEVICE_EXTENSION,
    InputData: *mut ntkrnl_sys::ddk::MOUSE_INPUT_DATA,
) {
    let mut OldIrql: KIRQL = 0;
    let mut InputDataConsumed: ULONG = 0;

    let ext = &*DeviceExtension;

    if ext.ClassService.is_null() {
        return;
    }

    // sanity check
    ASSERT!(!ext.ClassService.is_null());
    ASSERT!(!ext.ClassDeviceObject.is_null());

    // raise irql
    KeRaiseIrql(ntkrnl_sys::base::DISPATCH_LEVEL, &mut OldIrql);

    // dispatch input data
    let callback: PSERVICE_CALLBACK_ROUTINE =
        core::mem::transmute(ext.ClassService);
    callback(
        ext.ClassDeviceObject as PVOID,
        InputData as PVOID,
        InputData.add(1) as PVOID,
        &mut InputDataConsumed as *mut ULONG as *mut PVOID,
    );
    // One report accepted by mouclass proves the USB mouse input path; useful
    // on the Pi, where the framebuffer cursor alone cannot prove HID delivery.
    if InputDataConsumed != 0 {
        static DELIVERED: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);
        if !DELIVERED.swap(true, core::sync::atomic::Ordering::Relaxed) {
            ntkrnl_sys::driver_ddk::DbgPrint(
                b"[MOUHID] report delivered dx=%ld dy=%ld\n\0".as_ptr() as *const i8,
                (*InputData).LastX,
                (*InputData).LastY,
            );
        }
    }

    // lower irql to previous level
    KeLowerIrql(OldIrql);
}

// ---------------------------------------------------------------------------
// MouHid_ReadCompletion
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_ReadCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: *mut ntkrnl_sys::ddk::_IRP,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceExtension: PMOUHID_DEVICE_EXTENSION = Context as _;
    let mut ButtonFlags: USHORT;
    let mut UsageValue: LONG;
    let mut Status: NTSTATUS;
    let mut LastX: LONG = 0;
    let mut LastY: LONG = 0;
    let mut MouseInputData: ntkrnl_sys::ddk::MOUSE_INPUT_DATA = core::mem::zeroed();
    let mut Flags: USHORT;

    let ext = &mut *DeviceExtension;

    let irp_status = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if irp_status == STATUS_PRIVILEGE_NOT_HELD
        || irp_status == STATUS_DEVICE_NOT_CONNECTED
        || irp_status == STATUS_CANCELLED
        || ext.StopReadReport != 0
    {
        // failed to read or should be stopped
        // DPRINT1("[MOUHID] ReadCompletion terminating read Status %x\n", ...)

        // report no longer active
        ext.ReadReportActive = FALSE as UCHAR;

        // request stopping of the report cycle
        ext.StopReadReport = FALSE as UCHAR;

        // signal completion event
        KeSetEvent(&mut ext.ReadCompletionEvent as *mut _ as PRKEVENT, 0, FALSE);
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    // get mouse change
    MouHid_GetButtonMove(DeviceExtension, &mut LastX, &mut LastY);

    // get mouse change flags
    ButtonFlags = 0;
    Flags = 0;
    MouHid_GetButtonFlags(DeviceExtension, &mut ButtonFlags, &mut Flags);

    // init input data
    RtlZeroMemory(
        &mut MouseInputData as *mut _ as PVOID,
        core::mem::size_of::<ntkrnl_sys::ddk::MOUSE_INPUT_DATA>(),
    );

    // init input data fields
    MouseInputData.__bindgen_anon_1.__bindgen_anon_1.ButtonFlags = ButtonFlags;
    MouseInputData.Flags = Flags;
    MouseInputData.LastX = LastX;
    MouseInputData.LastY = LastY;

    // detect mouse wheel change
    let ext = &*DeviceExtension;
    if ext.MouseIdentifier == WHEELMOUSE_HID_HARDWARE {
        // get usage
        UsageValue = 0;
        Status = HidP_GetScaledUsageValue(
            HidP_Input,
            HID_USAGE_PAGE_GENERIC,
            HIDP_LINK_COLLECTION_UNSPECIFIED,
            HID_USAGE_GENERIC_WHEEL,
            &mut UsageValue,
            ext.PreparsedData as _,
            ext.Report,
            ext.ReportLength,
        );
        if Status == HIDP_STATUS_SUCCESS && UsageValue != 0 {
            // store wheel status
            MouseInputData.__bindgen_anon_1.__bindgen_anon_1.ButtonFlags |= MOUSE_WHEEL;
            MouseInputData.__bindgen_anon_1.__bindgen_anon_1.ButtonData =
                (UsageValue * WHEEL_DELTA) as USHORT;
        }
        // else DPRINT("[MOUHID] failed to get wheel status with %x\n", Status)
    }

    // DPRINT("[MOUHID] ReportData ...")
    // DPRINT("[MOUHID] LastX %ld LastY %ld Flags ...")

    // dispatch mouse action
    MouHid_DispatchInputData(DeviceExtension, &mut MouseInputData);

    // re-init read
    MouHid_InitiateRead(DeviceExtension);

    // stop completion
    STATUS_MORE_PROCESSING_REQUIRED
}

// ---------------------------------------------------------------------------
// MouHid_InitiateRead
// ---------------------------------------------------------------------------
pub unsafe fn MouHid_InitiateRead(
    DeviceExtension: PMOUHID_DEVICE_EXTENSION,
) -> NTSTATUS {
    let ext = &mut *DeviceExtension;
    let mut IoStack: *mut _IO_STACK_LOCATION;
    let mut Status: NTSTATUS;

    // re-use irp
    IoReuseIrp(ext.Irp, STATUS_SUCCESS);

    // init irp
    (*ext.Irp).MdlAddress = ext.ReportMDL;

    // get next stack location
    IoStack = IoGetNextIrpStackLocation(ext.Irp);

    // init stack location
    (*IoStack).Parameters.Read.Length = ext.ReportLength;
    (*IoStack).Parameters.Read.Key = 0;
    (*IoStack).Parameters.Read.ByteOffset.QuadPart = 0i64;
    (*IoStack).MajorFunction = IRP_MJ_READ;
    (*IoStack).FileObject = ext.FileObject;

    // set completion routine
    type CompletionFn = unsafe extern "system" fn(
        PDEVICE_OBJECT,
        *mut ntkrnl_sys::ddk::_IRP,
        PVOID,
    ) -> NTSTATUS;
    IoSetCompletionRoutine(
        ext.Irp,
        core::mem::transmute::<CompletionFn, ntkrnl_sys::ddk::PIO_COMPLETION_ROUTINE>(
            MouHid_ReadCompletion,
        ),
        DeviceExtension as PVOID,
        TRUE as UCHAR,
        TRUE as UCHAR,
        TRUE as UCHAR,
    );

    // read is active
    ext.ReadReportActive = TRUE as UCHAR;

    // start the read
    Status = IoCallDriver(ext.NextDeviceObject, ext.Irp);

    // done
    Status
}

// ---------------------------------------------------------------------------
// MouHid_CreateCompletion — used by MouHid_Create
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_CreateCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: *mut ntkrnl_sys::ddk::_IRP,
    Context: PVOID,
) -> NTSTATUS {
    KeSetEvent(Context as PRKEVENT, 0, FALSE);
    STATUS_MORE_PROCESSING_REQUIRED
}

// ---------------------------------------------------------------------------
// MouHid_StartDeviceCompletion — used by MouHid_Pnp IRP_MN_START_DEVICE
// ---------------------------------------------------------------------------
pub unsafe extern "system" fn MouHid_StartDeviceCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    _Irp: *mut ntkrnl_sys::ddk::_IRP,
    Context: PVOID,
) -> NTSTATUS {
    KeSetEvent(Context as PRKEVENT, 0, FALSE);
    STATUS_MORE_PROCESSING_REQUIRED
}
