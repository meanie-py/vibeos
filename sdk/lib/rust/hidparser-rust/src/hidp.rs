use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use core::ffi::c_void;

use hid_sys::{
    HIDP_DEVICE_DESC, PHIDP_DEVICE_DESC,
    HIDP_CAPS,
    HIDP_REPORT_TYPE,
    HIDP_VALUE_CAPS, PHIDP_VALUE_CAPS,
    HIDP_BUTTON_CAPS,
    HIDP_DATA, PHIDP_DATA,
    HIDP_KEYBOARD_DIRECTION,
    HIDP_KEYBOARD_MODIFIER_STATE, PHIDP_KEYBOARD_MODIFIER_STATE,
    PHIDP_INSERT_SCANCODES,
    HIDP_LINK_COLLECTION_NODE, PHIDP_LINK_COLLECTION_NODE,
    HIDP_EXTENDED_ATTRIBUTES, PHIDP_EXTENDED_ATTRIBUTES,
    PHIDP_REPORT_DESCRIPTOR, PHIDP_PREPARSED_DATA, _HIDP_PREPARSED_DATA,
    USAGE,
    HidP_Input, HidP_Output, HidP_Feature,
    HIDP_STATUS_SUCCESS, HIDP_STATUS_I8042_TRANS_UNKNOWN,
    HIDP_STATUS_INTERNAL_ERROR, HIDP_STATUS_INVALID_REPORT_TYPE,
    HIDP_STATUS_USAGE_NOT_FOUND, HIDP_STATUS_BUFFER_TOO_SMALL,
};

// Local consts/types — not in substrate
type PUSAGE = *mut USAGE;
type HIDP_REPORT_DESCRIPTOR = PUCHAR;
const HID_USAGE_PAGE_UNDEFINED: u16 = 0;
const HIDP_LINK_COLLECTION_UNSPECIFIED: u16 = 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct USAGE_AND_PAGE {
    pub Usage: USAGE,
    pub UsagePage: USAGE,
}

pub type HIDP_PREPARSED_DATA = _HIDP_PREPARSED_DATA;

// The public HidParser_* API is NTAPI (HIDAPI NTSTATUS NTAPI ...) → extern "system",
// matching their definitions in hidparser.rs. RtlAssert is also NTAPI. DebugFunction
// is __cdecl (separate extern "C" block below).
extern "system" {
    fn HidParser_FreeCollectionDescription(DeviceDescription: PHIDP_DEVICE_DESC);
    fn HidParser_GetCaps(PreparsedData: *mut core::ffi::c_void, Capabilities: *mut HIDP_CAPS) -> NTSTATUS;
    fn RtlAssert( FailedAssertion: *const core::ffi::c_void, FileName: *const core::ffi::c_void, LineNumber: ULONG, Message: *mut core::ffi::c_void, );
    fn HidParser_MaxUsageListLength( PreparsedData: PVOID, ReportType: HIDP_REPORT_TYPE, UsagePage: USAGE, ) -> ULONG;
    fn HidParser_GetSpecificValueCaps( preparsed_data: *mut core::ffi::c_void, report_type: HIDP_REPORT_TYPE, usage_page: USAGE, link_collection: USHORT, usage: USAGE, value_caps: PHIDP_VALUE_CAPS, value_caps_length: PUSHORT, ) -> NTSTATUS;
    fn HidParser_GetUsages( PreparsedData: *mut core::ffi::c_void, ReportType: HIDP_REPORT_TYPE, UsagePage: USAGE, LinkCollection: USHORT, UsageList: *mut USAGE, UsageLength: *mut ULONG, Report: *mut i8, ReportLength: ULONG, ) -> NTSTATUS;
    fn HidParser_UsageListDifference( PreviousUsageList: PUSAGE, CurrentUsageList: PUSAGE, BreakUsageList: PUSAGE, MakeUsageList: PUSAGE, UsageListLength: ULONG, ) -> NTSTATUS;
    fn HidParser_UsageAndPageListDifference( PreviousUsageList: *mut USAGE_AND_PAGE, CurrentUsageList: *mut USAGE_AND_PAGE, BreakUsageList: *mut USAGE_AND_PAGE, MakeUsageList: *mut USAGE_AND_PAGE, UsageListLength: u32, ) -> i32;
    fn HidParser_GetScaledUsageValue( preparsedData: *mut core::ffi::c_void, reportType: HIDP_REPORT_TYPE, usagePage: USAGE, linkCollection: USHORT, usage: USAGE, usageValue: *mut i32, report: *mut i8, reportLength: ULONG, ) -> NTSTATUS;
    fn HidParser_GetUsageValue( PreparsedData: PVOID, ReportType: HIDP_REPORT_TYPE, UsagePage: USAGE, LinkCollection: USHORT, Usage: USAGE, UsageValue: PULONG, Report: PCHAR, ReportLength: ULONG, ) -> NTSTATUS;
    fn HidParser_TranslateUsageAndPagesToI8042ScanCodes( ChangedUsageList: *mut USAGE_AND_PAGE, UsageListLength: u32, KeyAction: HIDP_KEYBOARD_DIRECTION, ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE, InsertCodesProcedure: PHIDP_INSERT_SCANCODES, InsertCodesContext: *mut core::ffi::c_void, ) -> NTSTATUS;
    fn HidParser_GetCollectionDescription( ReportDesc: *mut HIDP_REPORT_DESCRIPTOR, DescLength: u32, PoolType: POOL_TYPE, DeviceDescription: *mut HIDP_DEVICE_DESC, ) -> NTSTATUS;
}
extern "C" {
    fn DebugFunction(format: *const i8, ...);
}

#[no_mangle]
pub unsafe extern "system" fn HidP_FreeCollectionDescription(DeviceDescription: PHIDP_DEVICE_DESC) {
    HidParser_FreeCollectionDescription(DeviceDescription);
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetCaps(
    PreparsedData: *mut HIDP_PREPARSED_DATA,
    Capabilities: *mut HIDP_CAPS,
) -> NTSTATUS {
    return HidParser_GetCaps(PreparsedData as *mut core::ffi::c_void, Capabilities);
}
#[no_mangle]
pub unsafe extern "C" fn TranslateStatusForUpperLayer(Status: NTSTATUS) -> NTSTATUS {
    match Status {
        HIDP_STATUS_INTERNAL_ERROR => return STATUS_INSUFFICIENT_RESOURCES,
        HIDP_STATUS_INVALID_REPORT_TYPE => return HIDP_STATUS_INVALID_REPORT_TYPE,
        HIDP_STATUS_BUFFER_TOO_SMALL => return STATUS_BUFFER_TOO_SMALL,
        HIDP_STATUS_USAGE_NOT_FOUND => return STATUS_NO_DATA_DETECTED,
        _ => return Status,
    };
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetCollectionDescription(
    ReportDesc: *mut HIDP_REPORT_DESCRIPTOR,
    DescLength: u32,
    PoolType: POOL_TYPE,
    DeviceDescription: *mut HIDP_DEVICE_DESC,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    Status = HidParser_GetCollectionDescription(ReportDesc, DescLength, PoolType, DeviceDescription);
    return TranslateStatusForUpperLayer(Status);
}

#[no_mangle]
pub unsafe extern "system" fn HidP_MaxUsageListLength(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    PreparsedData: PHIDP_PREPARSED_DATA,
) -> ULONG {
    if !(ReportType as ::core::ffi::c_uint == HidP_Input as ::core::ffi::c_int as ::core::ffi::c_uint
        || ReportType as ::core::ffi::c_uint == HidP_Output as ::core::ffi::c_int as ::core::ffi::c_uint
        || ReportType as ::core::ffi::c_uint == HidP_Feature as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        RtlAssert(
            b"ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const ::core::ffi::c_char as PVOID,
            87 as ULONG,
            ::core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    return HidParser_MaxUsageListLength(PreparsedData as PVOID, ReportType, UsagePage);
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetSpecificValueCaps(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    ValueCaps: PHIDP_VALUE_CAPS,
    ValueCapsLength: PUSHORT,
    PreparsedData: PHIDP_PREPARSED_DATA,
) -> NTSTATUS {
    if !(ReportType as u32 == HidP_Input as u32
        || ReportType as u32 == HidP_Output as u32
        || ReportType as u32 == HidP_Feature as u32)
    {
        RtlAssert(
            b"ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature\0"
                .as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const core::ffi::c_void,
            110u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {};

    HidParser_GetSpecificValueCaps(
        PreparsedData as *mut core::ffi::c_void,
        ReportType,
        UsagePage,
        LinkCollection,
        Usage,
        ValueCaps,
        ValueCapsLength,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetUsages(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    UsageList: *mut USAGE,
    UsageLength: *mut ULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: *mut i8,
    ReportLength: ULONG,
) -> NTSTATUS {
    if !(ReportType as u32 == HidP_Input as i32 as u32
        || ReportType as u32 == HidP_Output as i32 as u32
        || ReportType as u32 == HidP_Feature as i32 as u32)
    {
        RtlAssert(
            b"ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            134u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {}

    return HidParser_GetUsages(
        PreparsedData as *mut core::ffi::c_void,
        ReportType,
        UsagePage,
        LinkCollection,
        UsageList as *mut USAGE,
        UsageLength as *mut ULONG,
        Report,
        ReportLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidP_UsageListDifference(
    PreviousUsageList: PUSAGE,
    CurrentUsageList: PUSAGE,
    BreakUsageList: PUSAGE,
    MakeUsageList: PUSAGE,
    UsageListLength: ULONG,
) -> NTSTATUS {
    return HidParser_UsageListDifference(
        PreviousUsageList,
        CurrentUsageList,
        BreakUsageList,
        MakeUsageList,
        UsageListLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetUsagesEx(
    ReportType: HIDP_REPORT_TYPE,
    LinkCollection: USHORT,
    ButtonList: *mut USAGE_AND_PAGE,
    UsageLength: *mut ULONG,
    PreparsedData: *mut HIDP_PREPARSED_DATA,
    Report: *mut i8,
    ReportLength: ULONG,
) -> NTSTATUS {
    HidP_GetUsages(
        ReportType,
        0u16, // HID_USAGE_PAGE_UNDEFINED
        LinkCollection,
        &raw mut (*ButtonList).Usage,
        UsageLength,
        PreparsedData,
        Report,
        ReportLength,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HidP_UsageAndPageListDifference(
    PreviousUsageList: *mut USAGE_AND_PAGE,
    CurrentUsageList: *mut USAGE_AND_PAGE,
    BreakUsageList: *mut USAGE_AND_PAGE,
    MakeUsageList: *mut USAGE_AND_PAGE,
    UsageListLength: u32,
) -> i32 {
    return HidParser_UsageAndPageListDifference(
        PreviousUsageList,
        CurrentUsageList,
        BreakUsageList,
        MakeUsageList,
        UsageListLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetScaledUsageValue(
    mut ReportType: HIDP_REPORT_TYPE,
    mut UsagePage: USAGE,
    mut LinkCollection: USHORT,
    mut Usage: USAGE,
    mut UsageValue: *mut i32,
    mut PreparsedData: *mut HIDP_PREPARSED_DATA,
    mut Report: *mut i8,
    mut ReportLength: ULONG,
) -> NTSTATUS {
    if !((ReportType as u32 == HidP_Input as i32 as u32)
        || (ReportType as u32 == HidP_Output as i32 as u32)
        || (ReportType as u32 == HidP_Feature as i32 as u32))
    {
        RtlAssert(
            (b"ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature\0"
                as *const u8 as *const core::ffi::c_char) as *mut core::ffi::c_void,
            (b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const core::ffi::c_char) as *mut core::ffi::c_void,
            202 as ULONG,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return HidParser_GetScaledUsageValue(
        PreparsedData as *mut core::ffi::c_void,
        ReportType,
        UsagePage,
        LinkCollection,
        Usage,
        UsageValue,
        Report,
        ReportLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetUsageValue(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    UsageValue: PULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    if !(ReportType as ::core::ffi::c_uint
        == HidP_Input as ::core::ffi::c_int as ::core::ffi::c_uint
        || ReportType as ::core::ffi::c_uint
            == HidP_Output as ::core::ffi::c_int as ::core::ffi::c_uint
        || ReportType as ::core::ffi::c_uint
            == HidP_Feature as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        RtlAssert(
            b"ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const ::core::ffi::c_char as PVOID,
            226 as ULONG,
            ::core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return HidParser_GetUsageValue(
        PreparsedData as PVOID,
        ReportType,
        UsagePage,
        LinkCollection,
        Usage,
        UsageValue,
        Report,
        ReportLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidP_TranslateUsageAndPagesToI8042ScanCodes(
    ChangedUsageList: *mut USAGE_AND_PAGE,
    UsageListLength: u32,
    KeyAction: HIDP_KEYBOARD_DIRECTION,
    ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE,
    InsertCodesProcedure: PHIDP_INSERT_SCANCODES,
    InsertCodesContext: *mut core::ffi::c_void,
) -> NTSTATUS {
    HidParser_TranslateUsageAndPagesToI8042ScanCodes(
        ChangedUsageList,
        UsageListLength,
        KeyAction,
        ModifierState,
        InsertCodesProcedure,
        InsertCodesContext,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetButtonCaps(
    ReportType: HIDP_REPORT_TYPE,
    ButtonCaps: *mut HIDP_BUTTON_CAPS,
    ButtonCapsLength: *mut u16,
    PreparsedData: *mut HIDP_PREPARSED_DATA,
) -> NTSTATUS {
    HidP_GetSpecificButtonCaps(
        ReportType,
        0u16, /* HID_USAGE_PAGE_UNDEFINED */
        0u16,
        0u16,
        ButtonCaps,
        ButtonCapsLength,
        PreparsedData,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetSpecificButtonCaps(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: u16,
    LinkCollection: u16,
    Usage: u16,
    ButtonCaps: *mut HIDP_BUTTON_CAPS,
    ButtonCapsLength: *mut u16,
    PreparsedData: *mut HIDP_PREPARSED_DATA,
) -> i32 {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_GetSpecificButtonCaps\0" as *const u8 as *const core::ffi::c_void,
    );
    if (0 as i32) == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            277u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetData(
    ReportType: HIDP_REPORT_TYPE,
    DataList: *mut HIDP_DATA,
    DataLength: *mut u32,
    PreparsedData: *mut HIDP_PREPARSED_DATA,
    Report: *mut core::ffi::c_char,
    ReportLength: u32,
) -> NTSTATUS {

    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const core::ffi::c_char,
        b"HidP_GetData\0".as_ptr() as *const core::ffi::c_char,
    );

    if 0 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0".as_ptr() as *mut core::ffi::c_void,
            293u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetExtendedAttributes(
    mut ReportType: HIDP_REPORT_TYPE,
    mut DataIndex: USHORT,
    mut PreparsedData: PHIDP_PREPARSED_DATA,
    mut Attributes: PHIDP_EXTENDED_ATTRIBUTES,
    mut LengthAttributes: PULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_GetExtendedAttributes\0" as *const u8 as *const core::ffi::c_char,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const core::ffi::c_char as *mut core::ffi::c_void,
            308u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetLinkCollectionNodes(
    LinkCollectionNodes: PHIDP_LINK_COLLECTION_NODE,
    LinkCollectionNodesLength: PULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_GetLinkCollectionNodes\0".as_ptr() as *const core::ffi::c_void,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            321u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SysPowerEvent(
    HidPacket: PCHAR,
    HidPacketLength: USHORT,
    Ppd: PHIDP_PREPARSED_DATA,
    OutputBuffer: PULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SysPowerEvent\0".as_ptr() as *const core::ffi::c_void,
    );
    if 0_i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as PVOID,
            b"sdk/lib/drivers/hidparser/hidp.c\0".as_ptr() as *const i8 as PVOID,
            334_u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SysPowerCaps(
    mut Ppd: PHIDP_PREPARSED_DATA,
    mut OutputBuffer: PULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SysPowerCaps\0" as *const u8 as *const core::ffi::c_void,
    );
    if 0_i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            345 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetUsageValueArray(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    UsageValue: PCHAR,
    UsageValueByteLength: USHORT,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_GetUsageValueArray\0" as *const u8 as *const core::ffi::c_void,
    );
    if 0 as i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as *mut ::core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const i8 as *mut ::core::ffi::c_void,
            364 as u32,
            ::core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_UnsetUsages(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    UsageList: PUSAGE,
    UsageLength: PULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_UnsetUsages\0".as_ptr() as *const core::ffi::c_void,
    );
    if 0 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0".as_ptr()
                as *const core::ffi::c_void,
            383,
            core::ptr::null_mut(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_TranslateUsagesToI8042ScanCodes(
    ChangedUsageList: PUSAGE,
    UsageListLength: ULONG,
    KeyAction: HIDP_KEYBOARD_DIRECTION,
    ModifierState: PHIDP_KEYBOARD_MODIFIER_STATE,
    InsertCodesProcedure: PHIDP_INSERT_SCANCODES,
    InsertCodesContext: PVOID,
) -> NTSTATUS {

    DebugFunction(
        c"%s is UNIMPLEMENTED\n".as_ptr() as *const core::ffi::c_char,
        c"HidP_TranslateUsagesToI8042ScanCodes".as_ptr() as *const core::ffi::c_char,
    );
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            c"FALSE".as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const core::ffi::c_char as *const core::ffi::c_void,
            399 as ULONG,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SetUsages(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    UsageList: PUSAGE,
    UsageLength: PULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {

    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SetUsages\0".as_ptr() as *const core::ffi::c_void,
    );

    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const core::ffi::c_void,
            417u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };

    return ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SetUsageValueArray(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    UsageValue: PCHAR,
    UsageValueByteLength: USHORT,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SetUsageValueArray\0".as_ptr() as *const core::ffi::c_void,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as PVOID,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const i8 as PVOID,
            436u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SetUsageValue(
    mut ReportType: HIDP_REPORT_TYPE,
    mut UsagePage: USAGE,
    mut LinkCollection: USHORT,
    mut Usage: USAGE,
    mut UsageValue: ULONG,
    mut PreparsedData: PHIDP_PREPARSED_DATA,
    mut Report: PCHAR,
    mut ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SetUsageValue\0" as *const u8 as *const core::ffi::c_void,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *mut core::ffi::c_void,
            454u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SetScaledUsageValue(
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    UsageValue: LONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SetScaledUsageValue\0".as_ptr() as *const core::ffi::c_char,
    );
    if 0 as i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0".as_ptr() as *const core::ffi::c_void,
            472 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_SetData(
    ReportType: HIDP_REPORT_TYPE,
    DataList: PHIDP_DATA,
    DataLength: PULONG,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_SetData\0" as *const u8 as *const ::core::ffi::c_char,
    );
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *mut core::ffi::c_void,
            488 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_MaxDataListLength(
    ReportType: HIDP_REPORT_TYPE,
    PreparsedData: PHIDP_PREPARSED_DATA,
) -> ULONG {
    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_MaxDataListLength\0".as_ptr() as *const core::ffi::c_char,
    );
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                .as_ptr() as *const core::ffi::c_void,
            500 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_InitializeReportForID(
    ReportType: HIDP_REPORT_TYPE,
    ReportID: UCHAR,
    PreparsedData: PHIDP_PREPARSED_DATA,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {

    DebugFunction(
        b"%s is UNIMPLEMENTED\n\0".as_ptr() as *const i8,
        b"HidP_InitializeReportForID\0" as *const u8 as *const core::ffi::c_void,
    );

    if 0 as i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidp.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            515 as ULONG,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidP_GetValueCaps(
    ReportType: HIDP_REPORT_TYPE,
    ValueCaps: PHIDP_VALUE_CAPS,
    ValueCapsLength: PUSHORT,
    PreparsedData: PHIDP_PREPARSED_DATA,
) -> NTSTATUS {
    HidP_GetSpecificValueCaps(
        ReportType,
        HID_USAGE_PAGE_UNDEFINED,
        HIDP_LINK_COLLECTION_UNSPECIFIED,
        0 as USAGE,
        ValueCaps,
        ValueCapsLength,
        PreparsedData,
    )
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #define _HIDPI_
// #define _HIDPI_NO_FUNCTION_MACROS_
// #include <ntddk.h>
// #include <hidpddi.h>
//
// #include "hidparser.h"
// #include "hidp.h"
//
// #define UNIMPLEMENTED DebugFunction("%s is UNIMPLEMENTED\n", __FUNCTION__)
//
// VOID
// NTAPI
// HidP_FreeCollectionDescription(
//     IN PHIDP_DEVICE_DESC   DeviceDescription)
// {
//     //
//     // free collection
//     //
//     HidParser_FreeCollectionDescription(DeviceDescription);
// }
//
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetCaps(
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     OUT PHIDP_CAPS  Capabilities)
// {
//     //
//     // get caps
//     //
//     return HidParser_GetCaps(PreparsedData, Capabilities);
// }
//
// NTSTATUS
// TranslateStatusForUpperLayer(
//     IN NTSTATUS Status)
// {
//     //
//     // now we are handling only this values, for others just return
//     // status as it is.
//     //
//     switch (Status)
//     {
//     case HIDP_STATUS_INTERNAL_ERROR:
//         return STATUS_INSUFFICIENT_RESOURCES;
//     case HIDP_STATUS_INVALID_REPORT_TYPE:
//         return HIDP_STATUS_INVALID_REPORT_TYPE;
//     case HIDP_STATUS_BUFFER_TOO_SMALL:
//         return STATUS_BUFFER_TOO_SMALL;
//     case HIDP_STATUS_USAGE_NOT_FOUND:
//         return STATUS_NO_DATA_DETECTED;
//     default:
//         return Status;
//     }
// }
//
// NTSTATUS
// NTAPI
// HidP_GetCollectionDescription(
//     IN PHIDP_REPORT_DESCRIPTOR ReportDesc,
//     IN ULONG DescLength,
//     IN POOL_TYPE PoolType,
//     OUT PHIDP_DEVICE_DESC DeviceDescription)
// {
//     NTSTATUS Status;
//
//     //
//     // get description;
//     //
//     Status = HidParser_GetCollectionDescription(ReportDesc, DescLength, PoolType, DeviceDescription);
//     return TranslateStatusForUpperLayer(Status);
// }
//
// HIDAPI
// ULONG
// NTAPI
// HidP_MaxUsageListLength(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage  OPTIONAL,
//     IN PHIDP_PREPARSED_DATA  PreparsedData)
// {
//     //
//     // sanity check
//     //
//     ASSERT(ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature);
//
//     //
//     // get usage length
//     //
//     return HidParser_MaxUsageListLength(PreparsedData, ReportType, UsagePage);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetSpecificValueCaps(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     OUT PHIDP_VALUE_CAPS  ValueCaps,
//     IN OUT PUSHORT  ValueCapsLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData)
// {
//     //
//     // sanity check
//     //
//     ASSERT(ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature);
//
//     //
//     // get value caps
//     //
//     return HidParser_GetSpecificValueCaps(PreparsedData, ReportType, UsagePage, LinkCollection, Usage, ValueCaps, ValueCapsLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetUsages(
//     IN HIDP_REPORT_TYPE ReportType,
//     IN USAGE UsagePage,
//     IN USHORT LinkCollection  OPTIONAL,
//     OUT PUSAGE UsageList,
//     IN OUT PULONG UsageLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR Report,
//     IN ULONG ReportLength)
// {
//     //
//     // sanity check
//     //
//     ASSERT(ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature);
//
//     //
//     // get usages
//     //
//     return HidParser_GetUsages(PreparsedData, ReportType, UsagePage, LinkCollection, UsageList, UsageLength, Report, ReportLength);
// }
//
//
// #undef HidP_GetButtonCaps
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_UsageListDifference(
//     IN PUSAGE  PreviousUsageList,
//     IN PUSAGE  CurrentUsageList,
//     OUT PUSAGE  BreakUsageList,
//     OUT PUSAGE  MakeUsageList,
//     IN ULONG  UsageListLength)
// {
//     return HidParser_UsageListDifference(PreviousUsageList, CurrentUsageList, BreakUsageList, MakeUsageList, UsageListLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetUsagesEx(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USHORT  LinkCollection,
//     OUT PUSAGE_AND_PAGE  ButtonList,
//     IN OUT ULONG  *UsageLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     return HidP_GetUsages(ReportType, HID_USAGE_PAGE_UNDEFINED, LinkCollection, &ButtonList->Usage, UsageLength, PreparsedData, Report, ReportLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_UsageAndPageListDifference(
//     IN PUSAGE_AND_PAGE  PreviousUsageList,
//     IN PUSAGE_AND_PAGE  CurrentUsageList,
//     OUT PUSAGE_AND_PAGE  BreakUsageList,
//     OUT PUSAGE_AND_PAGE  MakeUsageList,
//     IN ULONG  UsageListLength)
// {
//     return HidParser_UsageAndPageListDifference(PreviousUsageList, CurrentUsageList, BreakUsageList, MakeUsageList, UsageListLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetScaledUsageValue(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     OUT PLONG  UsageValue,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     //
//     // sanity check
//     //
//     ASSERT(ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature);
//
//     //
//     // get scaled usage value
//     //
//     return HidParser_GetScaledUsageValue(PreparsedData, ReportType, UsagePage, LinkCollection, Usage, UsageValue, Report, ReportLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetUsageValue(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     OUT PULONG  UsageValue,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     //
//     // sanity check
//     //
//     ASSERT(ReportType == HidP_Input || ReportType == HidP_Output || ReportType == HidP_Feature);
//
//     //
//     // get scaled usage value
//     //
//     return HidParser_GetUsageValue(PreparsedData, ReportType, UsagePage, LinkCollection, Usage, UsageValue, Report, ReportLength);
// }
//
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_TranslateUsageAndPagesToI8042ScanCodes(
//     IN PUSAGE_AND_PAGE  ChangedUsageList,
//     IN ULONG  UsageListLength,
//     IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//     IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//     IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//     IN PVOID  InsertCodesContext)
// {
//     //
//     // translate usage pages
//     //
//     return HidParser_TranslateUsageAndPagesToI8042ScanCodes(ChangedUsageList, UsageListLength, KeyAction, ModifierState, InsertCodesProcedure, InsertCodesContext);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetButtonCaps(
//     HIDP_REPORT_TYPE ReportType,
//     PHIDP_BUTTON_CAPS ButtonCaps,
//     PUSHORT ButtonCapsLength,
//     PHIDP_PREPARSED_DATA PreparsedData)
// {
//     return HidP_GetSpecificButtonCaps(ReportType, HID_USAGE_PAGE_UNDEFINED, 0, 0, ButtonCaps, ButtonCapsLength, PreparsedData);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetSpecificButtonCaps(
//     IN HIDP_REPORT_TYPE ReportType,
//     IN USAGE UsagePage,
//     IN USHORT LinkCollection,
//     IN USAGE Usage,
//     OUT PHIDP_BUTTON_CAPS ButtonCaps,
//     IN OUT PUSHORT ButtonCapsLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetData(
//     IN HIDP_REPORT_TYPE  ReportType,
//     OUT PHIDP_DATA  DataList,
//     IN OUT PULONG  DataLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetExtendedAttributes(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USHORT DataIndex,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     OUT PHIDP_EXTENDED_ATTRIBUTES  Attributes,
//     IN OUT PULONG  LengthAttributes)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetLinkCollectionNodes(
//     OUT PHIDP_LINK_COLLECTION_NODE  LinkCollectionNodes,
//     IN OUT PULONG  LinkCollectionNodesLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// HidP_SysPowerEvent(
//     IN PCHAR HidPacket,
//     IN USHORT HidPacketLength,
//     IN PHIDP_PREPARSED_DATA Ppd,
//     OUT PULONG OutputBuffer)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// HidP_SysPowerCaps(
//     IN PHIDP_PREPARSED_DATA Ppd,
//     OUT PULONG OutputBuffer)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetUsageValueArray(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     OUT PCHAR  UsageValue,
//     IN USHORT  UsageValueByteLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_UnsetUsages(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN PUSAGE  UsageList,
//     IN OUT PULONG  UsageLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_TranslateUsagesToI8042ScanCodes(
//     IN PUSAGE  ChangedUsageList,
//     IN ULONG  UsageListLength,
//     IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//     IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//     IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//     IN PVOID  InsertCodesContext)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_SetUsages(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN PUSAGE  UsageList,
//     IN OUT PULONG  UsageLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_SetUsageValueArray(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     IN PCHAR  UsageValue,
//     IN USHORT  UsageValueByteLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_SetUsageValue(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     IN ULONG  UsageValue,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_SetScaledUsageValue(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     IN LONG  UsageValue,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_SetData(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN PHIDP_DATA  DataList,
//     IN OUT PULONG  DataLength,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// ULONG
// NTAPI
// HidP_MaxDataListLength(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN PHIDP_PREPARSED_DATA  PreparsedData)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_InitializeReportForID(
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN UCHAR  ReportID,
//     IN PHIDP_PREPARSED_DATA  PreparsedData,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// #undef HidP_GetValueCaps
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidP_GetValueCaps(
//     HIDP_REPORT_TYPE ReportType,
//     PHIDP_VALUE_CAPS ValueCaps,
//     PUSHORT ValueCapsLength,
//     PHIDP_PREPARSED_DATA PreparsedData)
// {
//     return HidP_GetSpecificValueCaps(ReportType,
//                                      HID_USAGE_PAGE_UNDEFINED,
//                                      HIDP_LINK_COLLECTION_UNSPECIFIED,
//                                      0,
//                                      ValueCaps,
//                                      ValueCapsLength,
//                                      PreparsedData);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
