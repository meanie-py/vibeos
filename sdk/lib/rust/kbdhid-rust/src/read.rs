// read.rs — KbdHid_InitiateRead, KbdHid_ReadCompletion, KbdHid_InsertScanCodes,
//            KbdHid_DispatchInputData.
// Translated from drivers/hid/kbdhid/kbdhid.c

use core::mem::size_of;
use ntkrnl_sys::base::{NTSTATUS, BOOLEAN, ULONG, PVOID, KIRQL, TRUE, FALSE};
use ntkrnl_sys::ddk::{PDEVICE_OBJECT, PIRP, KEYBOARD_INPUT_DATA, PIO_COMPLETION_ROUTINE};
use ntkrnl_sys::ke::KeSetEvent;
use ntkrnl_sys::constants::{
    STATUS_PRIVILEGE_NOT_HELD, STATUS_DEVICE_NOT_CONNECTED,
    STATUS_CANCELLED, IRP_MJ_READ, IO_NO_INCREMENT,
};
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::driver_ddk::IoCallDriver;
use ntkrnl_sys::rtl::RtlZeroMemory;
use ntkrnl_sys::inline::tier1::{IoGetNextIrpStackLocation, IoSetCompletionRoutine};
use crate::glue::{KeRaiseIrqlInternal, KeLowerIrqlInternal};
use ntkrnl_sys::base::DISPATCH_LEVEL;
use hid_sys::HIDP_KEYBOARD_DIRECTION;
use crate::types::{PKBDHID_DEVICE_EXTENSION, USAGE_AND_PAGE};
use crate::glue::{
    STATUS_MORE_PROCESSING_REQUIRED,
    KEY_BREAK, KEY_E0, KEY_E1,
    HidP_GetUsagesEx, HidP_UsageAndPageListDifference,
    HidP_TranslateUsageAndPagesToI8042ScanCodes,
    RtlMoveMemory,
};

// HIDP_KEYBOARD_DIRECTION values (hidpi.h enum):
//   HidP_Keyboard_Break = 0, HidP_Keyboard_Make = 1
const HIDP_KEYBOARD_BREAK: HIDP_KEYBOARD_DIRECTION = 0;
const HIDP_KEYBOARD_MAKE:  HIDP_KEYBOARD_DIRECTION = 1;

// PSERVICE_CALLBACK_ROUTINE — the class driver callback type (kbdmou.h).
// typedef VOID (STDAPICALLTYPE *PSERVICE_CALLBACK_ROUTINE)(
//   IN PVOID NormalContext, IN PVOID Arg1, IN PVOID Arg2, IN OUT PVOID Arg3);
// STDAPICALLTYPE == stdcall on x86, cdecl on x64/arm64.
// "extern "system"" gives stdcall on i386, C elsewhere — exactly right.
type PSERVICE_CALLBACK_ROUTINE = unsafe extern "system" fn(
    NormalContext: PVOID,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
    SystemArgument3: *mut PVOID,
);

/// KbdHid_DispatchInputData — raise to DISPATCH_LEVEL, call the class service
/// callback with one KEYBOARD_INPUT_DATA record, lower IRQL.
pub unsafe fn KbdHid_DispatchInputData(
    DeviceExtension: PKBDHID_DEVICE_EXTENSION,
    InputData: *mut KEYBOARD_INPUT_DATA,
) {
    if (*DeviceExtension).ClassService.is_null() {
        return;
    }

    /* sanity check */
    debug_assert!(!(*DeviceExtension).ClassService.is_null());
    debug_assert!(!(*DeviceExtension).ClassDeviceObject.is_null());

    let mut OldIrql: KIRQL = 0;
    let mut InputDataConsumed: ULONG = 0;

    /* raise irql */
    KeRaiseIrqlInternal(DISPATCH_LEVEL, &mut OldIrql);

    /* dispatch input data */
    // C: (*(PSERVICE_CALLBACK_ROUTINE)DeviceExtension->ClassService)(
    //        ClassDeviceObject, InputData, InputData + 1, &InputDataConsumed)
    let callback: PSERVICE_CALLBACK_ROUTINE =
        core::mem::transmute((*DeviceExtension).ClassService);
    callback(
        (*DeviceExtension).ClassDeviceObject as PVOID,
        InputData as PVOID,
        InputData.offset(1) as PVOID,
        &mut InputDataConsumed as *mut ULONG as *mut PVOID,
    );

    /* lower irql to previous level */
    KeLowerIrqlInternal(OldIrql);
}

/// KbdHid_InsertScanCodes — PHIDP_INSERT_SCANCODES callback invoked by
/// HidP_TranslateUsageAndPagesToI8042ScanCodes for each batch of scan codes.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_InsertScanCodes(
    Context: PVOID,
    NewScanCodes: *mut i8,
    Length: ULONG,
) -> BOOLEAN {
    let DeviceExtension = Context as PKBDHID_DEVICE_EXTENSION;
    let mut InputData: KEYBOARD_INPUT_DATA = core::mem::zeroed();
    let mut Prefix: i8 = 0;

    let mut Index: ULONG = 0;
    while Index < Length {
        let sc: i8 = *NewScanCodes.offset(Index as isize);

        /* check if this is E0 or E1 prefix */
        if sc == 0xE0u8 as i8 || sc == 0xE1u8 as i8 {
            Prefix = sc;
            Index += 1;
            continue;
        }

        /* init input data */
        RtlZeroMemory(
            &mut InputData as *mut _ as PVOID,
            size_of::<KEYBOARD_INPUT_DATA>(),
        );

        /* use keyboard unit id */
        InputData.UnitId = (*DeviceExtension).KeyboardTypematic.UnitId;

        if (sc as u8) & 0x80 != 0 {
            /* scan codes with 0x80 flag are a key break */
            InputData.Flags |= KEY_BREAK;
        }

        /* set a prefix if needed */
        if Prefix != 0 {
            InputData.Flags |= if Prefix == 0xE0u8 as i8 { KEY_E0 } else { KEY_E1 };
            Prefix = 0;
        }

        /* store key code */
        InputData.MakeCode = (sc as u8 & 0x7F) as u16;

        /* dispatch scan codes */
        KbdHid_DispatchInputData(Context as PKBDHID_DEVICE_EXTENSION, &mut InputData);

        Index += 1;
    }

    /* done */
    TRUE
}

/// KbdHid_ReadCompletion — IRP completion routine for the continuous read loop.
#[no_mangle]
pub unsafe extern "system" fn KbdHid_ReadCompletion(
    _DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let DeviceExtension = Context as PKBDHID_DEVICE_EXTENSION;
    let mut ButtonLength: ULONG;

    let irp_status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    if irp_status == STATUS_PRIVILEGE_NOT_HELD
        || irp_status == STATUS_DEVICE_NOT_CONNECTED
        || irp_status == STATUS_CANCELLED
        || (*DeviceExtension).StopReadReport != 0
    {
        /* failed to read or should be stopped */
        /* report no longer active */
        (*DeviceExtension).ReadReportActive = FALSE as BOOLEAN;

        /* request stopping of the report cycle */
        (*DeviceExtension).StopReadReport = FALSE as BOOLEAN;

        /* signal completion event */
        KeSetEvent(
            &mut (*DeviceExtension).ReadCompletionEvent as *mut _ as _,
            0,
            FALSE,
        );
        return STATUS_MORE_PROCESSING_REQUIRED;
    }

    // print out raw report (DPRINT — omitted, debug only)
    debug_assert!((*DeviceExtension).ReportLength >= 9);

    /* get current usages */
    ButtonLength = (*DeviceExtension).UsageListLength as ULONG;
    let Status = HidP_GetUsagesEx(
        hid_sys::HidP_Input,
        0, // HIDP_LINK_COLLECTION_UNSPECIFIED = 0
        (*DeviceExtension).CurrentUsageList,
        &mut ButtonLength,
        (*DeviceExtension).PreparsedData,
        (*DeviceExtension).Report,
        (*DeviceExtension).ReportLength,
    );
    debug_assert!(Status == hid_sys::HIDP_STATUS_SUCCESS);

    /* FIXME check if needs mapping */

    /* get usage difference */
    let _s = HidP_UsageAndPageListDifference(
        (*DeviceExtension).PreviousUsageList,
        (*DeviceExtension).CurrentUsageList,
        (*DeviceExtension).BreakUsageList,
        (*DeviceExtension).MakeUsageList,
        (*DeviceExtension).UsageListLength as ULONG,
    );
    debug_assert!(_s == hid_sys::HIDP_STATUS_SUCCESS);

    /* replace previous usage list with current list */
    RtlMoveMemory(
        (*DeviceExtension).PreviousUsageList as PVOID,
        (*DeviceExtension).CurrentUsageList as *const core::ffi::c_void,
        size_of::<USAGE_AND_PAGE>() * (*DeviceExtension).UsageListLength as usize,
    );

    /* translate break usage list */
    let _s = HidP_TranslateUsageAndPagesToI8042ScanCodes(
        (*DeviceExtension).BreakUsageList,
        (*DeviceExtension).UsageListLength as ULONG,
        HIDP_KEYBOARD_BREAK,
        &mut (*DeviceExtension).ModifierState,
        Some(KbdHid_InsertScanCodes),
        DeviceExtension as PVOID,
    );
    debug_assert!(_s == hid_sys::HIDP_STATUS_SUCCESS);

    /* translate new usage list */
    let _s = HidP_TranslateUsageAndPagesToI8042ScanCodes(
        (*DeviceExtension).MakeUsageList,
        (*DeviceExtension).UsageListLength as ULONG,
        HIDP_KEYBOARD_MAKE,
        &mut (*DeviceExtension).ModifierState,
        Some(KbdHid_InsertScanCodes),
        DeviceExtension as PVOID,
    );
    debug_assert!(_s == hid_sys::HIDP_STATUS_SUCCESS);

    /* re-init read */
    KbdHid_InitiateRead(DeviceExtension);

    /* stop completion */
    STATUS_MORE_PROCESSING_REQUIRED
}

/// KbdHid_InitiateRead — arm (or re-arm) the continuous read IRP.
pub unsafe fn KbdHid_InitiateRead(
    DeviceExtension: PKBDHID_DEVICE_EXTENSION,
) -> NTSTATUS {
    /* re-use irp */
    ntkrnl_sys::ddk::IoReuseIrp((*DeviceExtension).Irp, STATUS_SUCCESS);

    /* init irp */
    (*(*DeviceExtension).Irp).MdlAddress = (*DeviceExtension).ReportMDL;

    /* get next stack location */
    let IoStack = IoGetNextIrpStackLocation((*DeviceExtension).Irp);

    /* init stack location */
    (*IoStack).Parameters.Read.Length = (*DeviceExtension).ReportLength;
    (*IoStack).Parameters.Read.Key = 0;
    (*IoStack).Parameters.Read.ByteOffset.QuadPart = 0;
    (*IoStack).MajorFunction = IRP_MJ_READ;
    (*IoStack).FileObject = (*DeviceExtension).FileObject;

    /* set completion routine */
    IoSetCompletionRoutine(
        (*DeviceExtension).Irp,
        core::mem::transmute::<unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS, PIO_COMPLETION_ROUTINE>(KbdHid_ReadCompletion),
        DeviceExtension as PVOID,
        TRUE,
        TRUE,
        TRUE,
    );

    /* read is active */
    (*DeviceExtension).ReadReportActive = TRUE as BOOLEAN;

    /* start the read */
    IoCallDriver((*DeviceExtension).NextDeviceObject, (*DeviceExtension).Irp)
}
