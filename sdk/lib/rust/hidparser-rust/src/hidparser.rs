use hid_sys::{
    HIDP_DEVICE_DESC, PHIDP_DEVICE_DESC,
    HIDP_COLLECTION_DESC, PHIDP_COLLECTION_DESC,
    HIDP_REPORT_IDS, PHIDP_REPORT_IDS,
    HIDP_CAPS,
    HIDP_REPORT_TYPE,
    PHIDP_REPORT_DESCRIPTOR, PHIDP_VALUE_CAPS,
    HIDP_BUTTON_CAPS,
    HIDP_DATA,
    HIDP_KEYBOARD_DIRECTION,
    HIDP_KEYBOARD_MODIFIER_STATE, PHIDP_KEYBOARD_MODIFIER_STATE,
    PHIDP_INSERT_SCANCODES,
    HIDP_LINK_COLLECTION_NODE,
    HIDP_EXTENDED_ATTRIBUTES,
    USAGE,
    HidP_Input, HidP_Output, HidP_Feature,
    HIDP_STATUS_INVALID_REPORT_TYPE, HIDP_STATUS_I8042_TRANS_UNKNOWN,
    HIDP_STATUS_SUCCESS,
};
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;


// Local consts — not in substrate. HID_REPORT_TYPE_* is the parser-private bitmask
// (parser.h: INPUT=0x01/OUTPUT=0x02/FEATURE=0x04/ANY=0x07) — distinct from the DDK
// HidP_Input/Output/Feature enum (0/1/2, imported from hid_sys). ITEM_TAG_MAIN_* are
// raw 4-bit nibbles (ITEM_PREFIX.Tag:4), not byte-shifted.
const HID_REPORT_TYPE_INPUT: u8 = 0x01;
const HID_REPORT_TYPE_OUTPUT: u8 = 0x02;
const HID_REPORT_TYPE_FEATURE: u8 = 0x04;
const HID_USAGE_PAGE_UNDEFINED: u16 = 0;
const HIDP_LINK_COLLECTION_UNSPECIFIED: u16 = 0;
// hidusage.h: HID_USAGE_PAGE_KEYBOARD = 0x07. Was invented as 1, which sent every
// keyboard-page usage down the "unknown page" branch of
// HidParser_TranslateUsageAndPagesToI8042ScanCodes — zero USB keys ever dispatched.
const HID_USAGE_PAGE_KEYBOARD: u16 = 0x07;
const HID_USAGE_PAGE_CONSUMER: u16 = 0x0C;
const TRUE: u8 = 1;
const FALSE: u8 = 0;
const HID_REPORT_TYPE_ANY: u8 = 0x07;
const ITEM_TAG_MAIN_COLLECTION: u32 = 0xa;
const ITEM_TAG_MAIN_END_COLLECTION: u32 = 0xc;
const ITEM_TAG_MAIN_INPUT: u32 = 0x8;
const ITEM_TAG_MAIN_OUTPUT: u32 = 0x9;
const ITEM_TAG_MAIN_FEATURE: u32 = 0xb;
const ITEM_TYPE_MAIN: u32 = 0;
const ITEM_TYPE_GLOBAL: u32 = 1;
const ITEM_TYPE_LOCAL: u32 = 2;
const ITEM_TYPE_LONG: u32 = 3;
type PUSAGE = *mut USAGE;
type USAGE_PAGE = USHORT;
type LPCSTR = *const i8;
type PCSTR = *const i8;
type HIDP_REPORT_DESCRIPTOR = PUCHAR;
// Win32 BOOL is `int` (4 bytes), and api.rs:383 DEFINES HidParser_UsesReportId
// as `-> u32`. Declaring it u8 here made the decl and the definition disagree at
// 6 live call sites; it only ever behaved because the returned values are 0/1,
// so the low byte happened to be the whole answer.
type BOOL = i32;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct USAGE_AND_PAGE {
    pub Usage: USAGE,
    pub UsagePage: USAGE,
}
type PUSAGE_AND_PAGE = *mut USAGE_AND_PAGE;

const STATUS_NOT_IMPLEMENTED: i32 = 0xC0000002u32 as i32;
const STATUS_NO_DATA_DETECTED: i32 = 0x80000022u32 as i32;
const STATUS_INSUFFICIENT_RESOURCES: i32 = 0xC000009Au32 as i32;
const STATUS_SUCCESS: i32 = 0;
// HIDP_STATUS_SUCCESS / INTERNAL_ERROR / USAGE_NOT_FOUND / REPORT_DOES_NOT_EXIST
// imported from hid_sys.

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> u32;
}
// NTAPI (stdcall on i386) externals: RtlAssert + hidparse.sys util callbacks.
extern "system" {
    fn AllocFunction(Size: ULONG) -> PVOID;
    fn FreeFunction(Memory: PVOID);
    fn ZeroFunction( Destination: PVOID, Length: ULONG, );
    fn RtlAssert( FailedAssertion: *const core::ffi::c_void, FileName: *const core::ffi::c_void, LineNumber: ULONG, Message: *mut core::ffi::c_void, );
}
extern "C" {
    fn DebugFunction(Format: LPCSTR, ...);
    fn HidParser_BuildContext( ParserContext: PVOID, Index: ULONG, PreparsedDataLength: ULONG, PreparsedData: *mut PVOID, ) -> NTSTATUS;
    fn HidParser_GetCollectionUsagePage( PreparsedData: PVOID, Usage: *mut USAGE, UsagePage: *mut USAGE_PAGE, ) -> NTSTATUS;
    fn HidParser_GetContextSize( ParserContext: PVOID, Index: ULONG, ) -> USHORT;
    fn HidParser_GetReportLength( PreparsedData: PVOID, ReportType: UCHAR, ) -> USHORT;
    fn HidParser_NumberOfTopCollections( ParserContext: PVOID, ) -> ULONG;
    fn HidParser_ParseReportDescriptor( ReportDesc: PUCHAR, DescLength: ULONG, ParserContext: *mut PVOID, ) -> NTSTATUS;
    fn HidParser_UsesReportId( PreparsedData: PVOID, ReportType: UCHAR, ) -> BOOL;
    fn HidParser_GetMaxUsageListLengthWithReportAndPage(CollectionContext: PVOID, ReportType: u8, UsagePage: USAGE) -> ULONG;
    fn HidParser_GetSpecificValueCapsWithReport( CollectionContext: PVOID, ReportType: u8, UsagePage: USHORT, Usage: USHORT, ValueCaps: PHIDP_VALUE_CAPS, ValueCapsLength: PUSHORT, ) -> NTSTATUS;
    fn HidParser_GetUsagesWithReport( CollectionContext: *mut ::core::ffi::c_void, ReportType: u8, UsagePage: u16, UsageList: *mut u16, UsageLength: *mut u32, Report: *mut i8, ReportLength: u32, ) -> i32;
    fn HidParser_GetScaledUsageValueWithReport( collection_context: *mut core::ffi::c_void, report_type: u8, usage_page: u16, usage: u16, usage_value: *mut i32, report: *mut i8, report_length: u32, ) -> i32;
    fn HidParser_TranslateKbdUsage( Usage: u16, KeyAction: HIDP_KEYBOARD_DIRECTION, ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE, InsertCodesProcedure: PHIDP_INSERT_SCANCODES, InsertCodesContext: *mut ::core::ffi::c_void, ) -> NTSTATUS;
    fn HidParser_TranslateCustUsage( Usage: u16, KeyAction: HIDP_KEYBOARD_DIRECTION, ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE, InsertCodesProcedure: PHIDP_INSERT_SCANCODES, InsertCodesContext: *mut ::core::ffi::c_void, ) -> NTSTATUS;
    fn HidParser_GetUsageValueWithReport( CollectionContext: *mut core::ffi::c_void, ReportType: u8, UsagePage: u16, Usage: u16, UsageValue: *mut u32, Report: *mut i8, ReportLength: u32, ) -> i32;
    fn HidParser_GetTotalCollectionCount(Context: *mut core::ffi::c_void) -> u32;
    fn HidParser_GetReportItemTypeCountFromReportType(CollectionContext: *mut core::ffi::c_void, ReportType: u8, bData: u32) -> u32;
    fn HidParser_GetReportItemCountFromReportType(CollectionContext: *mut core::ffi::c_void, ReportType: u8) -> u32;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetCollectionDescription(
    ReportDesc: PHIDP_REPORT_DESCRIPTOR,
    DescLength: ULONG,
    _PoolType: POOL_TYPE,
    DeviceDescription: PHIDP_DEVICE_DESC,
) -> NTSTATUS {
    let mut ParserStatus: NTSTATUS;
    let CollectionCount: ULONG;
    let mut Index: ULONG;
    let mut ParserContext: PVOID = core::ptr::null_mut();

    // Windows zeroes the description before parsing, so a failure leaves NULL pointers and
    // zero lengths (HidPDescription.c:145-148). The C zeroed it only after a successful
    // parse, leaving the caller's garbage on every failure path.
    ZeroFunction(
        DeviceDescription as PVOID,
        core::mem::size_of::<HIDP_DEVICE_DESC>() as ULONG,
    );

    //
    // first parse the report descriptor
    //
    ParserStatus = HidParser_ParseReportDescriptor(
        ReportDesc as PUCHAR,
        DescLength,
        core::ptr::addr_of_mut!(ParserContext),
    );
    if ParserStatus != HIDP_STATUS_SUCCESS {
        //
        // failed to parse report descriptor
        //
        DebugFunction(
            b"[HIDPARSER] Failed to parse report descriptor with %x\n\0".as_ptr() as LPCSTR,
            ParserStatus,
        );
        return ParserStatus;
    }

    //
    // get collection count
    //
    CollectionCount = HidParser_NumberOfTopCollections(ParserContext);
    if CollectionCount == 0 {
        //
        // no top level collections found
        //
        return STATUS_NO_DATA_DETECTED;
    }

    //
    // zero description
    //
    ZeroFunction(
        DeviceDescription as PVOID,
        core::mem::size_of::<HIDP_DEVICE_DESC>() as ULONG,
    );

    //
    // allocate collection
    //
    (*DeviceDescription).CollectionDesc = AllocFunction(
        (core::mem::size_of::<HIDP_COLLECTION_DESC>() as usize)
            .wrapping_mul(CollectionCount as usize) as ULONG,
    ) as PHIDP_COLLECTION_DESC;
    if (*DeviceDescription).CollectionDesc.is_null() {
        //
        // no memory
        //
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // allocate report description
    //
    (*DeviceDescription).ReportIDs = AllocFunction(
        (core::mem::size_of::<HIDP_REPORT_IDS>() as usize)
            .wrapping_mul(CollectionCount as usize) as ULONG,
    ) as PHIDP_REPORT_IDS;
    if (*DeviceDescription).ReportIDs.is_null() {
        //
        // no memory
        //
        FreeFunction((*DeviceDescription).CollectionDesc as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Index = 0;
    while Index < CollectionCount {
        //
        // set preparsed data length
        //
        let coll_desc: *mut HIDP_COLLECTION_DESC =
            (*DeviceDescription).CollectionDesc.offset(Index as isize);
        (*coll_desc).PreparsedDataLength =
            HidParser_GetContextSize(ParserContext, Index) as USHORT;

        ParserStatus = HidParser_BuildContext(
            ParserContext,
            Index,
            (*coll_desc).PreparsedDataLength as ULONG,
            core::ptr::addr_of_mut!((*coll_desc).PreparsedData) as *mut PVOID,
        );
        if ParserStatus != HIDP_STATUS_SUCCESS {
            //
            // no memory
            //
            FreeFunction((*DeviceDescription).CollectionDesc as PVOID);
            FreeFunction((*DeviceDescription).ReportIDs as PVOID);
            return ParserStatus;
        }

        //
        // init report description
        //
        let rep_ids: *mut HIDP_REPORT_IDS =
            (*DeviceDescription).ReportIDs.offset(Index as isize);
        (*rep_ids).CollectionNumber = Index.wrapping_add(1) as UCHAR;
        (*rep_ids).ReportID = Index as UCHAR;
        (*rep_ids).InputLength = HidParser_GetReportLength(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_INPUT,
        ) as USHORT;
        (*rep_ids).OutputLength = HidParser_GetReportLength(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_OUTPUT,
        ) as USHORT;
        (*rep_ids).FeatureLength = HidParser_GetReportLength(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_FEATURE,
        ) as USHORT;

        // Adjust lengths for report ID usage (ReportIDs)
        let uses_input = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_INPUT,
        );
        let input_len_ptr = core::ptr::addr_of_mut!((*rep_ids).InputLength);
        *input_len_ptr = (*input_len_ptr as i32
            + if uses_input != 0 { 1 } else { 0 }) as USHORT;

        let uses_output = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_OUTPUT,
        );
        let output_len_ptr = core::ptr::addr_of_mut!((*rep_ids).OutputLength);
        *output_len_ptr = (*output_len_ptr as i32
            + if uses_output != 0 { 1 } else { 0 }) as USHORT;

        let uses_feature = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_FEATURE,
        );
        let feature_len_ptr = core::ptr::addr_of_mut!((*rep_ids).FeatureLength);
        *feature_len_ptr = (*feature_len_ptr as i32
            + if uses_feature != 0 { 1 } else { 0 }) as USHORT;

        //
        // init collection description
        //
        (*coll_desc).CollectionNumber = Index.wrapping_add(1) as UCHAR;

        //
        // get collection usage page
        //
        ParserStatus = HidParser_GetCollectionUsagePage(
            (*coll_desc).PreparsedData as PVOID,
            core::ptr::addr_of_mut!((*coll_desc).Usage),
            core::ptr::addr_of_mut!((*coll_desc).UsagePage),
        );
        if ParserStatus != HIDP_STATUS_SUCCESS {
            // collection not found
            FreeFunction((*DeviceDescription).CollectionDesc as PVOID);
            FreeFunction((*DeviceDescription).ReportIDs as PVOID);
            return ParserStatus;
        }

        //
        // windows seems to prepend the report id, regardless if it is required
        //
        (*coll_desc).CollectionNumber = Index.wrapping_add(1) as UCHAR;
        (*coll_desc).InputLength = (*rep_ids).InputLength;
        (*coll_desc).OutputLength = (*rep_ids).OutputLength;
        (*coll_desc).FeatureLength = (*rep_ids).FeatureLength;

        // Adjust lengths for collection descriptor (inverse logic). Only a report type the
        // collection HAS gets the prepended ID byte: Windows reports FeatureLength 0 for a
        // keyboard with no feature report, where the C said 1 (HidPDescription.c:168).
        let uses_input_coll = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_INPUT,
        );
        let input_len_coll_ptr = core::ptr::addr_of_mut!((*coll_desc).InputLength);
        *input_len_coll_ptr = (*input_len_coll_ptr as i32
            + if uses_input_coll == 0 && (*rep_ids).InputLength != 0 { 1 } else { 0 }) as USHORT;

        let uses_output_coll = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_OUTPUT,
        );
        let output_len_coll_ptr = core::ptr::addr_of_mut!((*coll_desc).OutputLength);
        *output_len_coll_ptr = (*output_len_coll_ptr as i32
            + if uses_output_coll == 0 && (*rep_ids).OutputLength != 0 { 1 } else { 0 }) as USHORT;

        let uses_feature_coll = HidParser_UsesReportId(
            (*coll_desc).PreparsedData as PVOID,
            HID_REPORT_TYPE_FEATURE,
        );
        let feature_len_coll_ptr = core::ptr::addr_of_mut!((*coll_desc).FeatureLength);
        *feature_len_coll_ptr = (*feature_len_coll_ptr as i32
            + if uses_feature_coll == 0 && (*rep_ids).FeatureLength != 0 { 1 } else { 0 }) as USHORT;

        Index = Index.wrapping_add(1);
    }

    //
    // store collection & report count
    //
    (*DeviceDescription).CollectionDescLength = CollectionCount;
    (*DeviceDescription).ReportIDsLength = CollectionCount;

    //
    // done
    //
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_FreeCollectionDescription(
    DeviceDescription: *mut HIDP_DEVICE_DESC,
) {
    let _Index: u32;
    for Index in 0..(*DeviceDescription).CollectionDescLength {
        FreeFunction(
            (*(*DeviceDescription).CollectionDesc.add(Index as usize)).PreparsedData
                as *mut core::ffi::c_void,
        );
    }
    FreeFunction((*DeviceDescription).CollectionDesc as *mut core::ffi::c_void);
    FreeFunction((*DeviceDescription).ReportIDs as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn HidParser_GetCaps(
    CollectionContext: *mut core::ffi::c_void,
    Capabilities: *mut HIDP_CAPS,
) -> i32 {
    unsafe {
        ZeroFunction(
            Capabilities as *mut core::ffi::c_void,
            core::mem::size_of::<HIDP_CAPS>() as u32,
        );
        HidParser_GetCollectionUsagePage(
            CollectionContext,
            core::ptr::addr_of_mut!((*Capabilities).Usage),
            core::ptr::addr_of_mut!((*Capabilities).UsagePage),
        );
        (*Capabilities).InputReportByteLength =
            HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_INPUT as u8) as u16;
        (*Capabilities).OutputReportByteLength =
            HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_OUTPUT as u8) as u16;
        (*Capabilities).FeatureReportByteLength =
            HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_FEATURE as u8) as u16;
        (*Capabilities).InputReportByteLength =
            (if (*Capabilities).InputReportByteLength as i32 > 0 {
                (*Capabilities).InputReportByteLength as i32 + 1
            } else {
                0
            }) as u16;
        (*Capabilities).OutputReportByteLength =
            (if (*Capabilities).OutputReportByteLength as i32 > 0 {
                (*Capabilities).OutputReportByteLength as i32 + 1
            } else {
                0
            }) as u16;
        (*Capabilities).FeatureReportByteLength =
            (if (*Capabilities).FeatureReportByteLength as i32 > 0 {
                (*Capabilities).FeatureReportByteLength as i32 + 1
            } else {
                0
            }) as u16;
        (*Capabilities).NumberLinkCollectionNodes =
            HidParser_GetTotalCollectionCount(CollectionContext) as u16;
        (*Capabilities).NumberInputDataIndices = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
            TRUE as u32,
        ) as u16;
        (*Capabilities).NumberOutputDataIndices = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
            TRUE as u32,
        ) as u16;
        (*Capabilities).NumberFeatureDataIndices = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
            TRUE as u32,
        ) as u16;
        (*Capabilities).NumberInputValueCaps = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
            FALSE as u32,
        ) as u16;
        (*Capabilities).NumberOutputValueCaps = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
            FALSE as u32,
        ) as u16;
        (*Capabilities).NumberFeatureValueCaps = HidParser_GetReportItemTypeCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
            FALSE as u32,
        ) as u16;
        (*Capabilities).NumberInputButtonCaps = HidParser_GetReportItemCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
        ) as u16;
        (*Capabilities).NumberOutputButtonCaps = HidParser_GetReportItemCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
        ) as u16;
        (*Capabilities).NumberFeatureButtonCaps = HidParser_GetReportItemCountFromReportType(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
        ) as u16;
        HIDP_STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_MaxUsageListLength(
    CollectionContext: PVOID,
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
) -> ULONG {
    if UsagePage as i32 == HID_USAGE_PAGE_UNDEFINED as i32 {
        DbgPrint(
            b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
            b"HidParser_MaxUsageListLength\0".as_ptr() as *const ::core::ffi::c_char,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const ::core::ffi::c_char,
            266 as ::core::ffi::c_int,
        );
        return 0u32;
    }
    if ReportType as u32 == HidP_Input {
        return HidParser_GetMaxUsageListLengthWithReportAndPage(
            CollectionContext,
            HID_REPORT_TYPE_INPUT,
            UsagePage,
        );
    } else if ReportType as u32 == HidP_Output {
        return HidParser_GetMaxUsageListLengthWithReportAndPage(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT,
            UsagePage,
        );
    } else if ReportType as u32 == HidP_Feature {
        return HidParser_GetMaxUsageListLengthWithReportAndPage(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE,
            UsagePage,
        );
    } else {
        return 0u32;
    }
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetButtonCaps(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: i32,
    ButtonCaps: *mut HIDP_BUTTON_CAPS,
    ButtonCapsLength: *mut u16,
) -> i32 {
    HidParser_GetSpecificButtonCaps(
        CollectionContext,
        ReportType,
        0u16, // HID_USAGE_PAGE_UNDEFINED
        0u16, // HIDP_LINK_COLLECTION_UNSPECIFIED
        0u16, // HID_USAGE_PAGE_UNDEFINED
        ButtonCaps,
        ButtonCapsLength as *mut u32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetSpecificValueCaps(
    CollectionContext: PVOID,
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: USAGE,
    LinkCollection: USHORT,
    Usage: USAGE,
    ValueCaps: PHIDP_VALUE_CAPS,
    ValueCapsLength: PUSHORT,
) -> NTSTATUS {
    // HID_REPORT_TYPE_* / HidP_* / HIDP_STATUS_INVALID_REPORT_TYPE imported at module scope.
    let mut ParserStatus: NTSTATUS = 0;
    if LinkCollection != 0 {
        RtlAssert(
            b"LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const core::ffi::c_void,
            335u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if ReportType == HidP_Input {
        ParserStatus = HidParser_GetSpecificValueCapsWithReport(
            CollectionContext,
            HID_REPORT_TYPE_INPUT,
            UsagePage,
            Usage,
            ValueCaps,
            ValueCapsLength,
        );
    } else if ReportType == HidP_Output {
        ParserStatus = HidParser_GetSpecificValueCapsWithReport(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT,
            UsagePage,
            Usage,
            ValueCaps,
            ValueCapsLength,
        );
    } else if ReportType == HidP_Feature {
        ParserStatus = HidParser_GetSpecificValueCapsWithReport(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE,
            UsagePage,
            Usage,
            ValueCaps,
            ValueCapsLength,
        );
    } else {
        return HIDP_STATUS_INVALID_REPORT_TYPE;
    }
    return ParserStatus;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_UsageListDifference(
    PreviousUsageList: PUSAGE,
    CurrentUsageList: PUSAGE,
    BreakUsageList: PUSAGE,
    MakeUsageList: PUSAGE,
    UsageListLength: ULONG,
) -> NTSTATUS {
    let mut Index: ULONG = 0;
    let mut SubIndex: ULONG = 0;
    let mut bFound: ULONG = 0;
    let mut BreakUsageIndex: ULONG = 0;
    let mut MakeUsageIndex: ULONG = 0;
    let mut CurrentUsage: USAGE = 0;
    let mut Usage: USAGE = 0;

    if UsageListLength != 0 {
        Index = 0;
        loop {
            CurrentUsage = *PreviousUsageList.add(Index as usize);
            if CurrentUsage == 0 {
                break;
            }
            SubIndex = 0;
            bFound = 0; // FALSE
            loop {
                Usage = *CurrentUsageList.add(SubIndex as usize);
                if Usage == 0 {
                    break;
                }
                if CurrentUsage == Usage {
                    bFound = 1; // TRUE
                    break;
                } else {
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(SubIndex < UsageListLength) {
                        break;
                    }
                }
            }
            if bFound == 0 {
                *BreakUsageList.add(BreakUsageIndex as usize) = CurrentUsage;
                BreakUsageIndex = BreakUsageIndex.wrapping_add(1);
            }
            Index = Index.wrapping_add(1);
            if !(Index < UsageListLength) {
                break;
            }
        }

        Index = 0;
        loop {
            CurrentUsage = *CurrentUsageList.add(Index as usize);
            if CurrentUsage == 0 {
                break;
            }
            SubIndex = 0;
            bFound = 0; // FALSE
            loop {
                Usage = *PreviousUsageList.add(SubIndex as usize);
                if Usage == 0 {
                    break;
                }
                if CurrentUsage == Usage {
                    bFound = 1; // TRUE
                    break;
                } else {
                    SubIndex = SubIndex.wrapping_add(1);
                    if !(SubIndex < UsageListLength) {
                        break;
                    }
                }
            }
            if bFound == 0 {
                *MakeUsageList.add(MakeUsageIndex as usize) = CurrentUsage;
                MakeUsageIndex = MakeUsageIndex.wrapping_add(1);
            }
            Index = Index.wrapping_add(1);
            if !(Index < UsageListLength) {
                break;
            }
        }
    }

    if BreakUsageIndex < UsageListLength {
        let break_zero_count = (UsageListLength - BreakUsageIndex) as usize 
            * core::mem::size_of::<USAGE>();
        core::ptr::write_bytes(
            BreakUsageList.add(BreakUsageIndex as usize).cast::<u8>(),
            0,
            break_zero_count,
        );
    }

    if MakeUsageIndex < UsageListLength {
        let make_zero_count = (UsageListLength - MakeUsageIndex) as usize 
            * core::mem::size_of::<USAGE>();
        core::ptr::write_bytes(
            MakeUsageList.add(MakeUsageIndex as usize).cast::<u8>(),
            0,
            make_zero_count,
        );
    }

    HIDP_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetUsages(
    CollectionContext: *mut ::core::ffi::c_void,
    ReportType: HIDP_REPORT_TYPE,
    UsagePage: u16,
    LinkCollection: u16,
    UsageList: *mut u16,
    UsageLength: *mut u32,
    Report: *mut i8,
    ReportLength: u32,
) -> i32 {
    let mut ParserStatus: i32 = 0;

    // FIXME: implement searching in specific collection
    if LinkCollection as i32 != 0 {
        RtlAssert(
            b"LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED\0".as_ptr() as *mut ::core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *mut ::core::ffi::c_void,
            519 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {}

    if ReportType as u32 == HidP_Input as u32 {
        // input report
        ParserStatus = HidParser_GetUsagesWithReport(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
            UsagePage,
            UsageList,
            UsageLength,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Output as u32 {
        // input report
        ParserStatus = HidParser_GetUsagesWithReport(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
            UsagePage,
            UsageList,
            UsageLength,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Feature as u32 {
        // input report
        ParserStatus = HidParser_GetUsagesWithReport(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
            UsagePage,
            UsageList,
            UsageLength,
            Report,
            ReportLength,
        );
    } else {
        // invalid report type
        return HIDP_STATUS_INVALID_REPORT_TYPE;
    }

    // return status
    return ParserStatus;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetScaledUsageValue(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u32,
    UsagePage: u16,
    LinkCollection: u16,
    Usage: u16,
    UsageValue: *mut i32,
    Report: *mut i8,
    ReportLength: u32,
) -> i32 {
    let mut ParserStatus: i32 = 0;

    if LinkCollection != HIDP_LINK_COLLECTION_UNSPECIFIED {
        RtlAssert(
            b"LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED\0"
                .as_ptr()
                .cast::<i8>()
                .cast::<core::ffi::c_void>(),
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                .as_ptr()
                .cast::<i8>()
                .cast::<core::ffi::c_void>(),
            574u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }

    if ReportType as u32 == HidP_Input as u32 {
        ParserStatus = HidParser_GetScaledUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Output as u32 {
        ParserStatus = HidParser_GetScaledUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Feature as u32 {
        ParserStatus = HidParser_GetScaledUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else {
        return HIDP_STATUS_INVALID_REPORT_TYPE;
    }

    return ParserStatus;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_TranslateUsageAndPagesToI8042ScanCodes(
    ChangedUsageList: *const USAGE_AND_PAGE,
    UsageListLength: u32,
    KeyAction: HIDP_KEYBOARD_DIRECTION,
    ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE,
    InsertCodesProcedure: PHIDP_INSERT_SCANCODES,
    InsertCodesContext: *mut ::core::ffi::c_void,
) -> NTSTATUS {
    let mut Index: u32 = 0;
    let mut Status: NTSTATUS = HIDP_STATUS_SUCCESS;
    while Index < UsageListLength {
        if (*ChangedUsageList.offset(Index as isize)).UsagePage as ::core::ffi::c_int
            == HID_USAGE_PAGE_KEYBOARD as ::core::ffi::c_int
        {
            Status = HidParser_TranslateKbdUsage(
                (*ChangedUsageList.offset(Index as isize)).Usage,
                KeyAction,
                ModifierState,
                InsertCodesProcedure,
                InsertCodesContext,
            );
        } else if (*ChangedUsageList.offset(Index as isize)).UsagePage as ::core::ffi::c_int
            == HID_USAGE_PAGE_CONSUMER as ::core::ffi::c_int
        {
            Status = HidParser_TranslateCustUsage(
                (*ChangedUsageList.offset(Index as isize)).Usage,
                KeyAction,
                ModifierState,
                InsertCodesProcedure,
                InsertCodesContext,
            );
        } else {
            return HIDP_STATUS_I8042_TRANS_UNKNOWN;
        }
        if Status != HIDP_STATUS_SUCCESS {
            return Status;
        }
        Index = Index.wrapping_add(1);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetUsagesEx(
    CollectionContext: PVOID,
    ReportType: HIDP_REPORT_TYPE,
    LinkCollection: USHORT,
    ButtonList: PUSAGE_AND_PAGE,
    UsageLength: *mut ULONG,
    Report: PCHAR,
    ReportLength: ULONG,
) -> NTSTATUS {
    return HidParser_GetUsages(
        CollectionContext,
        ReportType,
        HID_USAGE_PAGE_UNDEFINED,
        LinkCollection,
        ButtonList as *mut USAGE,
        UsageLength as *mut ULONG,
        Report,
        ReportLength,
    );
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_UsageAndPageListDifference(
    PreviousUsageList: *mut USAGE_AND_PAGE,
    CurrentUsageList: *mut USAGE_AND_PAGE,
    BreakUsageList: *mut USAGE_AND_PAGE,
    MakeUsageList: *mut USAGE_AND_PAGE,
    UsageListLength: u32,
) -> i32 {
    let mut Index: u32 = 0;
    let mut SubIndex: u32 = 0;
    let mut BreakUsageListIndex: u32 = 0;
    let mut MakeUsageListIndex: u32 = 0;
    let mut bFound: u32 = 0;
    let mut CurrentUsage: *mut USAGE_AND_PAGE = core::ptr::null_mut();
    let mut Usage: *mut USAGE_AND_PAGE = core::ptr::null_mut();

    if UsageListLength != 0 {
        Index = 0;
        loop {
            CurrentUsage = PreviousUsageList.offset(Index as isize);
            if (*CurrentUsage).Usage == 0 && (*CurrentUsage).UsagePage == 0 {
                break;
            }
            SubIndex = 0;
            bFound = 0;
            loop {
                Usage = CurrentUsageList.offset(SubIndex as isize);
                if (*Usage).Usage == 0 && (*Usage).UsagePage == 0 {
                    break;
                }
                if (*Usage).Usage == (*CurrentUsage).Usage && (*Usage).UsagePage == (*CurrentUsage).UsagePage {
                    bFound = 1;
                }
                SubIndex = SubIndex.wrapping_add(1);
                if !(SubIndex < UsageListLength) {
                    break;
                }
            }
            if bFound == 0 {
                (*BreakUsageList.offset(BreakUsageListIndex as isize)).Usage = (*CurrentUsage).Usage;
                (*BreakUsageList.offset(BreakUsageListIndex as isize)).UsagePage = (*CurrentUsage).UsagePage;
                BreakUsageListIndex = BreakUsageListIndex.wrapping_add(1);
            }
            Index = Index.wrapping_add(1);
            if !(Index < UsageListLength) {
                break;
            }
        }

        Index = 0;
        loop {
            CurrentUsage = CurrentUsageList.offset(Index as isize);
            if (*CurrentUsage).Usage == 0 && (*CurrentUsage).UsagePage == 0 {
                break;
            }
            SubIndex = 0;
            bFound = 0;
            loop {
                Usage = PreviousUsageList.offset(SubIndex as isize);
                if (*Usage).Usage == 0 && (*Usage).UsagePage == 0 {
                    break;
                }
                if (*Usage).Usage == (*CurrentUsage).Usage && (*Usage).UsagePage == (*CurrentUsage).UsagePage {
                    bFound = 1;
                }
                SubIndex = SubIndex.wrapping_add(1);
                if !(SubIndex < UsageListLength) {
                    break;
                }
            }
            if bFound == 0 {
                (*MakeUsageList.offset(MakeUsageListIndex as isize)).Usage = (*CurrentUsage).Usage;
                (*MakeUsageList.offset(MakeUsageListIndex as isize)).UsagePage = (*CurrentUsage).UsagePage;
                MakeUsageListIndex = MakeUsageListIndex.wrapping_add(1);
            }
            Index = Index.wrapping_add(1);
            if !(Index < UsageListLength) {
                break;
            }
        }
    }

    if BreakUsageListIndex < UsageListLength {
        let byte_len: usize = (UsageListLength - BreakUsageListIndex) as usize * core::mem::size_of::<USAGE_AND_PAGE>();
        core::ptr::write_bytes(
            BreakUsageList.add(BreakUsageListIndex as usize) as *mut u8,
            0,
            byte_len,
        );
    }

    if MakeUsageListIndex < UsageListLength {
        let byte_len: usize = (UsageListLength - MakeUsageListIndex) as usize * core::mem::size_of::<USAGE_AND_PAGE>();
        core::ptr::write_bytes(
            MakeUsageList.add(MakeUsageListIndex as usize) as *mut u8,
            0,
            byte_len,
        );
    }

    // HIDP_STATUS_SUCCESS (0x00110000), as the C returns, not STATUS_SUCCESS:
    // kbdhid checks for exactly that code, and the bare 0 translated here made
    // it assert on every keyboard report.
    HIDP_STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn HidParser_GetSpecificButtonCaps(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: i32,
    _UsagePage: u16,
    _LinkCollection: u16,
    _Usage: u16,
    _ButtonCaps: *mut HIDP_BUTTON_CAPS,
    _ButtonCapsLength: *mut u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetSpecificButtonCaps\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        827 as core::ffi::c_int,
    );
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            828 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetData(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: HIDP_REPORT_TYPE,
    _DataList: *mut HIDP_DATA,
    _DataLength: *mut u32,
    _Report: *mut i8,
    _ReportLength: u32,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetData\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        844i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            845u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetExtendedAttributes(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: HIDP_REPORT_TYPE,
    _DataIndex: u16,
    _Attributes: *mut HIDP_EXTENDED_ATTRIBUTES,
    _LengthAttributes: *mut u32,
) -> NTSTATUS {

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetExtendedAttributes\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
            .as_ptr() as *const i8,
        859i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            860u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetLinkCollectionNodes(
    _CollectionContext: *mut core::ffi::c_void,
    _LinkCollectionNodes: *mut HIDP_LINK_COLLECTION_NODE,
    _LinkCollectionNodesLength: *mut u32,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetLinkCollectionNodes\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        872i32,
    );
    if 0i32 == 0i32 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8 as *const core::ffi::c_void,
            873u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetUsageValue(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: i32,
    UsagePage: u16,
    LinkCollection: u16,
    Usage: u16,
    UsageValue: *mut u32,
    Report: *mut i8,
    ReportLength: u32,
) -> i32 {
    let mut ParserStatus: i32 = 0;
    if !(LinkCollection as i32 == (0 as u16) as i32) {
        RtlAssert(
            b"LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            895u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    if ReportType as u32 == HidP_Input as i32 as u32 {
        ParserStatus = HidParser_GetUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_INPUT as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Output as i32 as u32 {
        ParserStatus = HidParser_GetUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_OUTPUT as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else if ReportType as u32 == HidP_Feature as i32 as u32 {
        ParserStatus = HidParser_GetUsageValueWithReport(
            CollectionContext,
            HID_REPORT_TYPE_FEATURE as u8,
            UsagePage,
            Usage,
            UsageValue,
            Report,
            ReportLength,
        );
    } else {
        return HIDP_STATUS_INVALID_REPORT_TYPE;
    }
    return ParserStatus;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SysPowerEvent(
    _CollectionContext: *mut core::ffi::c_void,
    _HidPacket: *mut i8,
    _HidPacketLength: u16,
    _OutputBuffer: *mut u32,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SysPowerEvent\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
           .as_ptr() as *const i8,
        940i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                as *const u8 as *const i8 as *mut core::ffi::c_void,
            941u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SysPowerCaps(
    _CollectionContext: PVOID,
    _OutputBuffer: PULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SysPowerCaps\0" as *const u8 as *const ::core::ffi::c_char,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        951 as ::core::ffi::c_int,
    );
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *mut core::ffi::c_void,
            952 as ULONG,
            ::core::ptr::null_mut(),
        );
    } else {
    };
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetUsageValueArray(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: i32,
    _UsagePage: u16,
    _LinkCollection: u16,
    _Usage: u16,
    _UsageValue: *mut i8,
    _UsageValueByteLength: u16,
    _Report: *mut i8,
    _ReportLength: u32,
) -> i32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetUsageValueArray\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
            .as_ptr() as *const i8,
        970i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                .as_ptr() as *mut core::ffi::c_void,
            971u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_UnsetUsages(
    _CollectionContext: PVOID,
    _ReportType: HIDP_REPORT_TYPE,
    _UsagePage: USAGE,
    _LinkCollection: USHORT,
    _UsageList: PUSAGE,
    _UsageLength: PULONG,
    _Report: PCHAR,
    _ReportLength: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_UnsetUsages\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
           .as_ptr() as *const i8,
        988i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void as PVOID,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *mut core::ffi::c_void,
            989u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_TranslateUsagesToI8042ScanCodes(
    _ChangedUsageList: PUSAGE,
    _UsageListLength: ULONG,
    _KeyAction: HIDP_KEYBOARD_DIRECTION,
    _ModifierState: PHIDP_KEYBOARD_MODIFIER_STATE,
    _InsertCodesProcedure: PHIDP_INSERT_SCANCODES,
    _InsertCodesContext: PVOID,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_TranslateUsagesToI8042ScanCodes\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        1004_i32,
    );
    if 0 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as PVOID,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as PVOID,
            1005u32,
            core::ptr::null_mut(),
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SetUsages(
    _CollectionContext: PVOID,
    _ReportType: HIDP_REPORT_TYPE,
    _UsagePage: USAGE,
    _LinkCollection: USHORT,
    _UsageList: PUSAGE,
    _UsageLength: PULONG,
    _Report: PCHAR,
    _ReportLength: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SetUsages\0" as *const u8 as *const ::core::ffi::c_char,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1022 as ::core::ffi::c_int,
    );
    if 0 as ::core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *mut core::ffi::c_void,
            1023 as ULONG,
            ::core::ptr::null_mut(),
        );
    } else {
    };
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SetUsageValueArray(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: HIDP_REPORT_TYPE,
    _UsagePage: USAGE,
    _LinkCollection: USHORT,
    _Usage: USAGE,
    _UsageValue: *mut i8,
    _UsageValueByteLength: USHORT,
    _Report: *mut i8,
    _ReportLength: u32,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SetUsageValueArray\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        1041i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const core::ffi::c_void,
            1042u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SetUsageValue(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: HIDP_REPORT_TYPE,
    _UsagePage: USAGE,
    _LinkCollection: USHORT,
    _Usage: USAGE,
    _UsageValue: ULONG,
    _Report: PCHAR,
    _ReportLength: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SetUsageValue\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
            .as_ptr() as *const i8,
        1059 as core::ffi::c_int,
    );
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                .as_ptr() as *const core::ffi::c_char as *mut core::ffi::c_void,
            1060 as core::ffi::c_ulong,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SetScaledUsageValue(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: HIDP_REPORT_TYPE,
    _UsagePage: USAGE,
    _LinkCollection: USHORT,
    _Usage: USAGE,
    _UsageValue: LONG,
    _Report: *mut i8,
    _ReportLength: ULONG,
) -> NTSTATUS {

    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_SetScaledUsageValue\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        1077i32,
    );
    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const core::ffi::c_void,
            1078u32,
            core::ptr::null_mut(),
        );
    } else {
    }
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_SetData(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: u32,
    _DataList: *mut HIDP_DATA,
    _DataLength: *mut u32,
    _Report: *mut i8,
    _ReportLength: u32,
) -> i32 {
    DbgPrint(
        c"WARNING:  %s at %s:%d is unimplemented!()\n".as_ptr() as *const i8,
        c"HidParser_SetData".as_ptr() as *const i8,
        c"sdk/lib/drivers/hidparser/hidparser.c".as_ptr()
            as *const core::ffi::c_void,
        1093i32,
    );
    if 0i32 == 0i32 {
        RtlAssert(
            c"FALSE".as_ptr() as *const i8 as *mut core::ffi::c_void,
            c"sdk/lib/drivers/hidparser/hidparser.c"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            1094u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_MaxDataListLength(
    _CollectionContext: *mut core::ffi::c_void,
    _ReportType: i32,
) -> u32 {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_MaxDataListLength\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr()
            as *const core::ffi::c_void,
        1105 as core::ffi::c_int,
    );
    if 0 as core::ffi::c_int == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr()
                as *const i8 as *const core::ffi::c_void,
            1106 as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    return 0 as u32;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_InitializeReportForID(
    _CollectionContext: PVOID,
    _ReportType: HIDP_REPORT_TYPE,
    _ReportID: UCHAR,
    _Report: PCHAR,
    _ReportLength: ULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_InitializeReportForID\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0"
           .as_ptr() as *const i8,
        1120 as i32,
    );
    if 0 as i32 == 0 {
        RtlAssert(
            b"FALSE\0" as *const u8 as *const i8 as PVOID,
            b"sdk/lib/drivers/hidparser/hidparser.c\0"
                as *const u8 as *const i8 as PVOID,
            1121 as ULONG,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn HidParser_GetValueCaps(
    _CollectionContext: PVOID,
    _ReportType: HIDP_REPORT_TYPE,
    _ValueCaps: PHIDP_VALUE_CAPS,
    _ValueCapsLength: PULONG,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"HidParser_GetValueCaps\0".as_ptr() as *const i8,
        b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8,
        1136i32,
    );

    if 0i32 == 0 {
        RtlAssert(
            b"FALSE\0".as_ptr() as *const i8 as PVOID,
            b"sdk/lib/drivers/hidparser/hidparser.c\0".as_ptr() as *const i8 as PVOID,
            1137u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    }

    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS HID Parser Library
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        lib/drivers/hidparser/hidparser.c
//  * PURPOSE:     HID Parser
//  * PROGRAMMERS:
//  *              Michael Martin (michael.martin@reactos.org)
//  *              Johannes Anderwald (johannes.anderwald@reactos.org)
//  */
//
// #include "parser.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// HidParser_GetCollectionDescription(
//     IN PHIDP_REPORT_DESCRIPTOR ReportDesc,
//     IN ULONG DescLength,
//     IN POOL_TYPE PoolType,
//     OUT PHIDP_DEVICE_DESC DeviceDescription)
// {
//     NTSTATUS ParserStatus;
//     ULONG CollectionCount;
//     ULONG Index;
//     PVOID ParserContext;
//
//     //
//     // first parse the report descriptor
//     //
//     ParserStatus = HidParser_ParseReportDescriptor(ReportDesc, DescLength, &ParserContext);
//     if (ParserStatus != HIDP_STATUS_SUCCESS)
//     {
//         //
//         // failed to parse report descriptor
//         //
//         DebugFunction("[HIDPARSER] Failed to parse report descriptor with %x\n", ParserStatus);
//         return ParserStatus;
//     }
//
//     //
//     // get collection count
//     //
//     CollectionCount = HidParser_NumberOfTopCollections(ParserContext);
//     if (CollectionCount == 0)
//     {
//         //
//         // no top level collections found
//         //
//         ASSERT(FALSE);
//         return STATUS_NO_DATA_DETECTED;
//     }
//
//     //
//     // zero description
//     //
//     ZeroFunction(DeviceDescription, sizeof(HIDP_DEVICE_DESC));
//
//     //
//     // allocate collection
//     //
//     DeviceDescription->CollectionDesc = (PHIDP_COLLECTION_DESC)AllocFunction(sizeof(HIDP_COLLECTION_DESC) * CollectionCount);
//     if (!DeviceDescription->CollectionDesc)
//     {
//         //
//         // no memory
//         //
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     //
//     // allocate report description
//     //
//     DeviceDescription->ReportIDs = (PHIDP_REPORT_IDS)AllocFunction(sizeof(HIDP_REPORT_IDS) * CollectionCount);
//     if (!DeviceDescription->ReportIDs)
//     {
//         //
//         // no memory
//         //
//         FreeFunction(DeviceDescription->CollectionDesc);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     for(Index = 0; Index < CollectionCount; Index++)
//     {
//         //
//         // set preparsed data length
//         //
//         DeviceDescription->CollectionDesc[Index].PreparsedDataLength = HidParser_GetContextSize(ParserContext, Index);
//         ParserStatus = HidParser_BuildContext(ParserContext, Index, DeviceDescription->CollectionDesc[Index].PreparsedDataLength, (PVOID*)&DeviceDescription->CollectionDesc[Index].PreparsedData);
//         if (ParserStatus != HIDP_STATUS_SUCCESS)
//         {
//             //
//             // no memory
//             //
//             FreeFunction(DeviceDescription->CollectionDesc);
//             FreeFunction(DeviceDescription->ReportIDs);
//             return ParserStatus;
//         }
//
//         //
//         // init report description
//         //
//         DeviceDescription->ReportIDs[Index].CollectionNumber = Index + 1;
//         DeviceDescription->ReportIDs[Index].ReportID = Index; //FIXME
//         DeviceDescription->ReportIDs[Index].InputLength = HidParser_GetReportLength((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_INPUT);
//         DeviceDescription->ReportIDs[Index].OutputLength = HidParser_GetReportLength((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_OUTPUT);
//         DeviceDescription->ReportIDs[Index].FeatureLength = HidParser_GetReportLength((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_FEATURE);
//
//
//         DeviceDescription->ReportIDs[Index].InputLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_INPUT) ? 1 : 0);
//         DeviceDescription->ReportIDs[Index].OutputLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_OUTPUT) ? 1 : 0);
//         DeviceDescription->ReportIDs[Index].FeatureLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_FEATURE) ? 1 : 0);
//
//
//         //
//         // init collection description
//         //
//         DeviceDescription->CollectionDesc[Index].CollectionNumber = Index + 1;
//
//         //
//         // get collection usage page
//         //
//         ParserStatus = HidParser_GetCollectionUsagePage((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, &DeviceDescription->CollectionDesc[Index].Usage, &DeviceDescription->CollectionDesc[Index].UsagePage);
//         if (ParserStatus != HIDP_STATUS_SUCCESS)
//         {
//             // collection not found
//             FreeFunction(DeviceDescription->CollectionDesc);
//             FreeFunction(DeviceDescription->ReportIDs);
//             return ParserStatus;
//         }
//
//         //
//         // windows seems to prepend the report id, regardless if it is required
//         //
//         DeviceDescription->CollectionDesc[Index].CollectionNumber = Index + 1;
//         DeviceDescription->CollectionDesc[Index].InputLength = DeviceDescription->ReportIDs[Index].InputLength;
//         DeviceDescription->CollectionDesc[Index].OutputLength = DeviceDescription->ReportIDs[Index].OutputLength;
//         DeviceDescription->CollectionDesc[Index].FeatureLength = DeviceDescription->ReportIDs[Index].FeatureLength;
//
//         DeviceDescription->CollectionDesc[Index].InputLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_INPUT) == FALSE ? 1 : 0);
//         DeviceDescription->CollectionDesc[Index].OutputLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_OUTPUT) == FALSE ? 1 : 0);
//         DeviceDescription->CollectionDesc[Index].FeatureLength += (HidParser_UsesReportId((PVOID)DeviceDescription->CollectionDesc[Index].PreparsedData, HID_REPORT_TYPE_FEATURE) == FALSE ? 1 : 0);
//
//
//     }
//
//     //
//     // store collection & report count
//     //
//     DeviceDescription->CollectionDescLength = CollectionCount;
//     DeviceDescription->ReportIDsLength = CollectionCount;
//
//     //
//     // done
//     //
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// HidParser_FreeCollectionDescription(
//     IN PHIDP_DEVICE_DESC   DeviceDescription)
// {
//     ULONG Index;
//
//     //
//     // first free all context
//     //
//     for(Index = 0; Index < DeviceDescription->CollectionDescLength; Index++)
//     {
//         //
//         // free collection context
//         //
//         FreeFunction(DeviceDescription->CollectionDesc[Index].PreparsedData);
//     }
//
//     //
//     // now free collection description
//     //
//     FreeFunction(DeviceDescription->CollectionDesc);
//
//     //
//     // free report description
//     //
//     FreeFunction(DeviceDescription->ReportIDs);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetCaps(
//     IN PVOID CollectionContext,
//     OUT PHIDP_CAPS  Capabilities)
// {
//     //
//     // zero capabilities
//     //
//     ZeroFunction(Capabilities, sizeof(HIDP_CAPS));
//
//     //
//     // init capabilities
//     //
//     HidParser_GetCollectionUsagePage(CollectionContext, &Capabilities->Usage, &Capabilities->UsagePage);
//     Capabilities->InputReportByteLength = HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_INPUT);
//     Capabilities->OutputReportByteLength = HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_OUTPUT);
//     Capabilities->FeatureReportByteLength = HidParser_GetReportLength(CollectionContext, HID_REPORT_TYPE_FEATURE);
//
//     //
//     // always pre-prend report id
//     //
//     Capabilities->InputReportByteLength = (Capabilities->InputReportByteLength > 0 ? Capabilities->InputReportByteLength + 1 : 0);
//     Capabilities->OutputReportByteLength = (Capabilities->OutputReportByteLength > 0 ? Capabilities->OutputReportByteLength + 1 : 0);
//     Capabilities->FeatureReportByteLength = (Capabilities->FeatureReportByteLength > 0 ? Capabilities->FeatureReportByteLength + 1 : 0);
//
//     //
//     // get number of link collection nodes
//     //
//     Capabilities->NumberLinkCollectionNodes = HidParser_GetTotalCollectionCount(CollectionContext);
//
//     //
//     // get data indices
//     //
//     Capabilities->NumberInputDataIndices = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_INPUT, TRUE);
//     Capabilities->NumberOutputDataIndices = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_OUTPUT, TRUE);
//     Capabilities->NumberFeatureDataIndices = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_FEATURE, TRUE);
//
//     //
//     // get value caps
//     //
//     Capabilities->NumberInputValueCaps = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_INPUT, FALSE);
//     Capabilities->NumberOutputValueCaps = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_OUTPUT, FALSE);
//     Capabilities->NumberFeatureValueCaps = HidParser_GetReportItemTypeCountFromReportType(CollectionContext, HID_REPORT_TYPE_FEATURE, FALSE);
//
//
//     //
//     // get button caps
//     //
//     Capabilities->NumberInputButtonCaps = HidParser_GetReportItemCountFromReportType(CollectionContext, HID_REPORT_TYPE_INPUT);
//     Capabilities->NumberOutputButtonCaps = HidParser_GetReportItemCountFromReportType(CollectionContext, HID_REPORT_TYPE_OUTPUT);
//     Capabilities->NumberFeatureButtonCaps = HidParser_GetReportItemCountFromReportType(CollectionContext, HID_REPORT_TYPE_FEATURE);
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// HIDAPI
// ULONG
// NTAPI
// HidParser_MaxUsageListLength(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage  OPTIONAL)
// {
//     //
//     // FIXME test what should be returned when usage page is not defined
//     //
//     if (UsagePage == HID_USAGE_PAGE_UNDEFINED)
//     {
//         //
//         // implement me
//         //
//         UNIMPLEMENTED;
//
//         //
//         // invalid report
//         //
//         return 0;
//     }
//
//     if (ReportType == HidP_Input)
//     {
//         //
//         // input report
//         //
//         return HidParser_GetMaxUsageListLengthWithReportAndPage(CollectionContext, HID_REPORT_TYPE_INPUT, UsagePage);
//     }
//     else if (ReportType == HidP_Output)
//     {
//         //
//         // input report
//         //
//         return HidParser_GetMaxUsageListLengthWithReportAndPage(CollectionContext, HID_REPORT_TYPE_OUTPUT, UsagePage);
//     }
//     else if (ReportType == HidP_Feature)
//     {
//         //
//         // input report
//         //
//         return HidParser_GetMaxUsageListLengthWithReportAndPage(CollectionContext, HID_REPORT_TYPE_FEATURE, UsagePage);
//     }
//     else
//     {
//         //
//         // invalid report type
//         //
//         return 0;
//     }
// }
//
// #undef HidParser_GetButtonCaps
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetButtonCaps(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE ReportType,
//     IN PHIDP_BUTTON_CAPS ButtonCaps,
//     IN PUSHORT ButtonCapsLength)
// {
//     return HidParser_GetSpecificButtonCaps(CollectionContext, ReportType, HID_USAGE_PAGE_UNDEFINED, HIDP_LINK_COLLECTION_UNSPECIFIED, HID_USAGE_PAGE_UNDEFINED, ButtonCaps, (PULONG)ButtonCapsLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetSpecificValueCaps(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     OUT PHIDP_VALUE_CAPS  ValueCaps,
//     IN OUT PUSHORT  ValueCapsLength)
// {
//     NTSTATUS ParserStatus;
//
//     //
//     // FIXME: implement searching in specific collection
//     //
//     ASSERT(LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED);
//
//     if (ReportType == HidP_Input)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetSpecificValueCapsWithReport(CollectionContext, HID_REPORT_TYPE_INPUT, UsagePage, Usage, ValueCaps, ValueCapsLength);
//     }
//     else if (ReportType == HidP_Output)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetSpecificValueCapsWithReport(CollectionContext, HID_REPORT_TYPE_OUTPUT, UsagePage, Usage, ValueCaps, ValueCapsLength);
//     }
//     else if (ReportType == HidP_Feature)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetSpecificValueCapsWithReport(CollectionContext, HID_REPORT_TYPE_FEATURE, UsagePage, Usage, ValueCaps, ValueCapsLength);
//     }
//     else
//     {
//         //
//         // invalid report type
//         //
//         return HIDP_STATUS_INVALID_REPORT_TYPE;
//     }
//
//     //
//     // return status
//     //
//     return ParserStatus;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_UsageListDifference(
//   IN PUSAGE  PreviousUsageList,
//   IN PUSAGE  CurrentUsageList,
//   OUT PUSAGE  BreakUsageList,
//   OUT PUSAGE  MakeUsageList,
//   IN ULONG  UsageListLength)
// {
//     ULONG Index, SubIndex, bFound, BreakUsageIndex = 0, MakeUsageIndex = 0;
//     USAGE CurrentUsage, Usage;
//
//     if (UsageListLength)
//     {
//         Index = 0;
//         do
//         {
//             /* get current usage */
//             CurrentUsage = PreviousUsageList[Index];
//
//             /* is the end of list reached? */
//             if (!CurrentUsage)
//                 break;
//
//             /* start searching in current usage list */
//             SubIndex = 0;
//             bFound = FALSE;
//             do
//             {
//                 /* get usage of current list */
//                 Usage = CurrentUsageList[SubIndex];
//
//                 /* end of list reached? */
//                 if (!Usage)
//                     break;
//
//                 /* check if it matches the current one */
//                 if (CurrentUsage == Usage)
//                 {
//                     /* it does */
//                     bFound = TRUE;
//                     break;
//                 }
//
//                 /* move to next usage */
//                 SubIndex++;
//             }while(SubIndex < UsageListLength);
//
//             /* was the usage found ?*/
//             if (!bFound)
//             {
//                 /* store it in the break usage list */
//                 BreakUsageList[BreakUsageIndex] = CurrentUsage;
//                 BreakUsageIndex++;
//             }
//
//             /* move to next usage */
//             Index++;
//
//         }while(Index < UsageListLength);
//
//         /* now process the new items */
//         Index = 0;
//         do
//         {
//             /* get current usage */
//             CurrentUsage = CurrentUsageList[Index];
//
//             /* is the end of list reached? */
//             if (!CurrentUsage)
//                 break;
//
//             /* start searching in current usage list */
//             SubIndex = 0;
//             bFound = FALSE;
//             do
//             {
//                 /* get usage of previous list */
//                 Usage = PreviousUsageList[SubIndex];
//
//                 /* end of list reached? */
//                 if (!Usage)
//                     break;
//
//                 /* check if it matches the current one */
//                 if (CurrentUsage == Usage)
//                 {
//                     /* it does */
//                     bFound = TRUE;
//                     break;
//                 }
//
//                 /* move to next usage */
//                 SubIndex++;
//             }while(SubIndex < UsageListLength);
//
//             /* was the usage found ?*/
//             if (!bFound)
//             {
//                 /* store it in the make usage list */
//                 MakeUsageList[MakeUsageIndex] = CurrentUsage;
//                 MakeUsageIndex++;
//             }
//
//             /* move to next usage */
//             Index++;
//
//         }while(Index < UsageListLength);
//     }
//
//     /* does the break list contain empty entries */
//     if (BreakUsageIndex < UsageListLength)
//     {
//         /* zeroize entries */
//         RtlZeroMemory(&BreakUsageList[BreakUsageIndex], sizeof(USAGE) * (UsageListLength - BreakUsageIndex));
//     }
//
//     /* does the make usage list contain empty entries */
//     if (MakeUsageIndex < UsageListLength)
//     {
//         /* zeroize entries */
//         RtlZeroMemory(&MakeUsageList[MakeUsageIndex], sizeof(USAGE) * (UsageListLength - MakeUsageIndex));
//     }
//
//     /* done */
//     return HIDP_STATUS_SUCCESS;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetUsages(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     OUT USAGE  *UsageList,
//     IN OUT PULONG UsageLength,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     NTSTATUS ParserStatus;
//
//     //
//     // FIXME: implement searching in specific collection
//     //
//     ASSERT(LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED);
//
//     if (ReportType == HidP_Input)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsagesWithReport(CollectionContext, HID_REPORT_TYPE_INPUT, UsagePage, UsageList, UsageLength, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Output)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsagesWithReport(CollectionContext, HID_REPORT_TYPE_OUTPUT, UsagePage, UsageList, UsageLength, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Feature)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsagesWithReport(CollectionContext, HID_REPORT_TYPE_FEATURE, UsagePage, UsageList, UsageLength, Report, ReportLength);
//     }
//     else
//     {
//         //
//         // invalid report type
//         //
//         return HIDP_STATUS_INVALID_REPORT_TYPE;
//     }
//
//     //
//     // return status
//     //
//     return ParserStatus;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetScaledUsageValue(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     OUT PLONG  UsageValue,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     NTSTATUS ParserStatus;
//
//     //
//     // FIXME: implement searching in specific collection
//     //
//     ASSERT(LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED);
//
//     if (ReportType == HidP_Input)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetScaledUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_INPUT, UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Output)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetScaledUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_OUTPUT, UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Feature)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetScaledUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_FEATURE,  UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else
//     {
//         //
//         // invalid report type
//         //
//         return HIDP_STATUS_INVALID_REPORT_TYPE;
//     }
//
//     //
//     // return status
//     //
//     return ParserStatus;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_TranslateUsageAndPagesToI8042ScanCodes(
//    IN PUSAGE_AND_PAGE  ChangedUsageList,
//    IN ULONG  UsageListLength,
//    IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//    IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//    IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//    IN PVOID  InsertCodesContext)
// {
//     ULONG Index;
//     NTSTATUS Status = HIDP_STATUS_SUCCESS;
//
//     for(Index = 0; Index < UsageListLength; Index++)
//     {
//         //
//         // check current usage
//         //
//         if (ChangedUsageList[Index].UsagePage == HID_USAGE_PAGE_KEYBOARD)
//         {
//             //
//             // process keyboard usage
//             //
//             Status = HidParser_TranslateKbdUsage(ChangedUsageList[Index].Usage, KeyAction, ModifierState, InsertCodesProcedure, InsertCodesContext);
//         }
//         else if (ChangedUsageList[Index].UsagePage == HID_USAGE_PAGE_CONSUMER)
//         {
//             //
//             // process consumer usage
//             //
//             Status = HidParser_TranslateCustUsage(ChangedUsageList[Index].Usage, KeyAction, ModifierState, InsertCodesProcedure, InsertCodesContext);
//         }
//         else
//         {
//             //
//             // invalid page / end of usage list page
//             //
//             return HIDP_STATUS_I8042_TRANS_UNKNOWN;
//         }
//
//         //
//         // check status
//         //
//         if (Status != HIDP_STATUS_SUCCESS)
//         {
//             //
//             // failed
//             //
//             return Status;
//         }
//     }
//
//     //
//     // return status
//     //
//     return Status;
// }
//
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetUsagesEx(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USHORT  LinkCollection,
//     OUT PUSAGE_AND_PAGE  ButtonList,
//     IN OUT ULONG  *UsageLength,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     return HidParser_GetUsages(CollectionContext, ReportType, HID_USAGE_PAGE_UNDEFINED, LinkCollection, (PUSAGE)ButtonList, UsageLength, Report, ReportLength);
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_UsageAndPageListDifference(
//    IN PUSAGE_AND_PAGE  PreviousUsageList,
//    IN PUSAGE_AND_PAGE  CurrentUsageList,
//    OUT PUSAGE_AND_PAGE  BreakUsageList,
//    OUT PUSAGE_AND_PAGE  MakeUsageList,
//    IN ULONG  UsageListLength)
// {
//     ULONG Index, SubIndex, BreakUsageListIndex = 0, MakeUsageListIndex = 0, bFound;
//     PUSAGE_AND_PAGE CurrentUsage, Usage;
//
//     if (UsageListLength)
//     {
//         /* process removed usages */
//         Index = 0;
//         do
//         {
//             /* get usage from current index */
//             CurrentUsage = &PreviousUsageList[Index];
//
//             /* end of list reached? */
//             if (CurrentUsage->Usage == 0 && CurrentUsage->UsagePage == 0)
//                 break;
//
//             /* search in current list */
//             SubIndex = 0;
//             bFound = FALSE;
//             do
//             {
//                 /* get usage */
//                 Usage = &CurrentUsageList[SubIndex];
//
//                 /* end of list reached? */
//                 if (Usage->Usage == 0 && Usage->UsagePage == 0)
//                     break;
//
//                 /* does it match */
//                 if (Usage->Usage == CurrentUsage->Usage && Usage->UsagePage == CurrentUsage->UsagePage)
//                 {
//                     /* found match */
//                     bFound = TRUE;
//                 }
//
//                 /* move to next index */
//                 SubIndex++;
//
//             }while(SubIndex < UsageListLength);
//
//             if (!bFound)
//             {
//                 /* store it in break usage list */
//                 BreakUsageList[BreakUsageListIndex].Usage = CurrentUsage->Usage;
//                 BreakUsageList[BreakUsageListIndex].UsagePage = CurrentUsage->UsagePage;
//                 BreakUsageListIndex++;
//             }
//
//             /* move to next index */
//             Index++;
//
//         }while(Index < UsageListLength);
//
//         /* process new usages */
//         Index = 0;
//         do
//         {
//             /* get usage from current index */
//             CurrentUsage = &CurrentUsageList[Index];
//
//             /* end of list reached? */
//             if (CurrentUsage->Usage == 0 && CurrentUsage->UsagePage == 0)
//                 break;
//
//             /* search in current list */
//             SubIndex = 0;
//             bFound = FALSE;
//             do
//             {
//                 /* get usage */
//                 Usage = &PreviousUsageList[SubIndex];
//
//                 /* end of list reached? */
//                 if (Usage->Usage == 0 && Usage->UsagePage == 0)
//                     break;
//
//                 /* does it match */
//                 if (Usage->Usage == CurrentUsage->Usage && Usage->UsagePage == CurrentUsage->UsagePage)
//                 {
//                     /* found match */
//                     bFound = TRUE;
//                 }
//
//                 /* move to next index */
//                 SubIndex++;
//
//             }while(SubIndex < UsageListLength);
//
//             if (!bFound)
//             {
//                 /* store it in break usage list */
//                 MakeUsageList[MakeUsageListIndex].Usage = CurrentUsage->Usage;
//                 MakeUsageList[MakeUsageListIndex].UsagePage = CurrentUsage->UsagePage;
//                 MakeUsageListIndex++;
//             }
//
//             /* move to next index */
//             Index++;
//         }while(Index < UsageListLength);
//     }
//
//     /* are there remaining free list entries */
//     if (BreakUsageListIndex < UsageListLength)
//     {
//         /* zero them */
//         RtlZeroMemory(&BreakUsageList[BreakUsageListIndex], (UsageListLength - BreakUsageListIndex) * sizeof(USAGE_AND_PAGE));
//     }
//
//     /* are there remaining free list entries */
//     if (MakeUsageListIndex < UsageListLength)
//     {
//         /* zero them */
//         RtlZeroMemory(&MakeUsageList[MakeUsageListIndex], (UsageListLength - MakeUsageListIndex) * sizeof(USAGE_AND_PAGE));
//     }
//
//     /* done */
//     return HIDP_STATUS_SUCCESS;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetSpecificButtonCaps(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     OUT PHIDP_BUTTON_CAPS  ButtonCaps,
//     IN OUT PULONG  ButtonCapsLength)
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
// HidParser_GetData(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     OUT PHIDP_DATA  DataList,
//     IN OUT PULONG  DataLength,
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
// HidParser_GetExtendedAttributes(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USHORT  DataIndex,
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
// HidParser_GetLinkCollectionNodes(
//     IN PVOID CollectionContext,
//     OUT PHIDP_LINK_COLLECTION_NODE  LinkCollectionNodes,
//     IN OUT PULONG  LinkCollectionNodesLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetUsageValue(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     OUT PULONG  UsageValue,
//     IN PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     NTSTATUS ParserStatus;
//
//     //
//     // FIXME: implement searching in specific collection
//     //
//     ASSERT(LinkCollection == HIDP_LINK_COLLECTION_UNSPECIFIED);
//
//     if (ReportType == HidP_Input)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_INPUT, UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Output)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_OUTPUT, UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else if (ReportType == HidP_Feature)
//     {
//         //
//         // input report
//         //
//         ParserStatus = HidParser_GetUsageValueWithReport(CollectionContext, HID_REPORT_TYPE_FEATURE,  UsagePage, Usage, UsageValue, Report, ReportLength);
//     }
//     else
//     {
//         //
//         // invalid report type
//         //
//         return HIDP_STATUS_INVALID_REPORT_TYPE;
//     }
//
//     //
//     // return status
//     //
//     return ParserStatus;
// }
//
// NTSTATUS
// NTAPI
// HidParser_SysPowerEvent(
//     IN PVOID CollectionContext,
//     IN PCHAR HidPacket,
//     IN USHORT HidPacketLength,
//     OUT PULONG OutputBuffer)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// HidParser_SysPowerCaps (
//     IN PVOID CollectionContext,
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
// HidParser_GetUsageValueArray(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     OUT PCHAR  UsageValue,
//     IN USHORT  UsageValueByteLength,
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
// HidParser_UnsetUsages(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN PUSAGE  UsageList,
//     IN OUT PULONG  UsageLength,
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
// HidParser_TranslateUsagesToI8042ScanCodes(
//   IN PUSAGE  ChangedUsageList,
//   IN ULONG  UsageListLength,
//   IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//   IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//   IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//   IN PVOID  InsertCodesContext)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_SetUsages(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN PUSAGE  UsageList,
//     IN OUT PULONG  UsageLength,
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
// HidParser_SetUsageValueArray(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     IN PCHAR  UsageValue,
//     IN USHORT  UsageValueByteLength,
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
// HidParser_SetUsageValue(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection,
//     IN USAGE  Usage,
//     IN ULONG  UsageValue,
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
// HidParser_SetScaledUsageValue(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN USAGE  UsagePage,
//     IN USHORT  LinkCollection  OPTIONAL,
//     IN USAGE  Usage,
//     IN LONG  UsageValue,
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
// HidParser_SetData(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN PHIDP_DATA  DataList,
//     IN OUT PULONG  DataLength,
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
// HidParser_MaxDataListLength(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return 0;
// }
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_InitializeReportForID(
//     IN PVOID CollectionContext,
//     IN HIDP_REPORT_TYPE  ReportType,
//     IN UCHAR  ReportID,
//     IN OUT PCHAR  Report,
//     IN ULONG  ReportLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// #undef HidParser_GetValueCaps
//
// HIDAPI
// NTSTATUS
// NTAPI
// HidParser_GetValueCaps(
//     IN PVOID CollectionContext,
//     HIDP_REPORT_TYPE ReportType,
//     PHIDP_VALUE_CAPS ValueCaps,
//     PULONG ValueCapsLength)
// {
//     UNIMPLEMENTED;
//     ASSERT(FALSE);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
