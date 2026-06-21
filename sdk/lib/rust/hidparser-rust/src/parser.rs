use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use core::ffi::c_void;

use crate::parser_h::{
    HID_COLLECTION, PHID_COLLECTION, __HID_COLLECTION__,
    HID_REPORT, PHID_REPORT, _HID_REPORT,
    HID_REPORT_ITEM, PHID_REPORT_ITEM,
    GLOBAL_ITEM_STATE, PGLOBAL_ITEM_STATE, __GLOBAL_ITEM_STATE__, __GLOBAL_ITEM_STATE_,
    LOCAL_ITEM_STATE, PLOCAL_ITEM_STATE,
    USAGE_VALUE, usage_value, PUSAGE_VALUE,
    ITEM_PREFIX, PITEM_PREFIX,
    SHORT_ITEM, PSHORT_ITEM,
    LONG_ITEM, PLONG_ITEM,
    MAIN_ITEM_DATA, PMAIN_ITEM_DATA,
    HID_PARSER_CONTEXT, PHID_PARSER_CONTEXT,
};
use hid_sys::{
    BOOLEAN,
    HIDP_STATUS_INTERNAL_ERROR, HIDP_STATUS_SUCCESS,
    HIDP_STATUS_REPORT_DOES_NOT_EXIST, HIDP_STATUS_USAGE_NOT_FOUND,
};

// Local consts/types — not in substrate
#[allow(non_camel_case_types)]
type BOOL = u8;
type C2RustUnnamed = SHORT_ITEM;
type C2RustUnnamed_1 = LONG_ITEM;
type C2RustUnnamed_2 = MAIN_ITEM_DATA;

extern "C" {
    fn HidParser_CalculateContextSize( Collection: PHID_COLLECTION, ) -> ULONG;
}
extern "system" {
    fn RtlAssert(msg: *const core::ffi::c_void, file: *const core::ffi::c_void, line: ULONG, p: *mut core::ffi::c_void);
    fn AllocFunction(size: u32) -> *mut core::ffi::c_void;
    fn CopyFunction(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, size: u32);
    fn FreeFunction(ptr: *mut core::ffi::c_void);
    fn HidParser_BuildCollectionContext( Collection: PHID_COLLECTION, Context: PVOID, ContextSize: ULONG, ) -> NTSTATUS;
    fn ZeroFunction(mem: *mut core::ffi::c_void, size: u32);
}
extern "C" {
    fn DebugFunction(msg: *const i8, ...);
}

// ---- file-scope preamble (translated) ----
static ItemSize: [u8; 4] = [0, 1, 2, 4];

// Translated from parser.c (grinder left it as an extern stub — body dropped).
// NOTE: faithful to the C, including its quirk — when the loop finds the sign bit
// it mutates the local Minimum/Maximum and returns WITHOUT writing the out params;
// only the no-sign-bit fall-through path writes *NewMinimum/*NewMaximum.
#[no_mangle]
pub unsafe extern "C" fn HidParser_SignRange(
    mut Minimum: ULONG,
    mut Maximum: ULONG,
    NewMinimum: *mut ULONG,
    NewMaximum: *mut ULONG,
) {
    let mut Mask: ULONG = 0x8000_0000;
    let mut Index: ULONG = 0;
    while Index < 4 {
        if (Minimum & Mask) != 0 {
            Minimum |= Mask;
            if (Maximum & Mask) != 0 {
                Maximum |= Mask;
            }
            return;
        }
        Mask >>= 8;
        Mask |= 0xff00_0000;
        Index = Index.wrapping_add(1);
    }
    core::ptr::write_volatile(NewMinimum, Minimum);
    core::ptr::write_volatile(NewMaximum, Maximum);
}

#[allow(unused)]
#[allow(non_camel_case_types)]
// Values from parser.h. Tag is a 4-bit bitfield (ITEM_PREFIX.Tag:4), so tags are
// raw nibbles (0x8/0x9/0xb/0xa/0xc), NOT byte-shifted. HID_REPORT_TYPE_* is the
// parser-private bitmask (INPUT=0x01/OUTPUT=0x02/FEATURE=0x04/ANY=0x07).
const ITEM_TYPE_MAIN: i32 = 0x0;
const ITEM_TYPE_GLOBAL: i32 = 0x1;
const ITEM_TYPE_LOCAL: i32 = 0x2;
const ITEM_TYPE_LONG: i32 = 0x3;
const ITEM_TAG_MAIN_COLLECTION: i32 = 0xa;
const ITEM_TAG_MAIN_END_COLLECTION: i32 = 0xc;
const ITEM_TAG_MAIN_INPUT: i32 = 0x8;
const ITEM_TAG_MAIN_OUTPUT: i32 = 0x9;
const ITEM_TAG_MAIN_FEATURE: i32 = 0xb;
// Global item tags (parser.h lines 31-42) — used as match arms on Tag() (4-bit
// nibble). The grinder dropped these defs, which silently turned each match arm
// into a catch-all binding ("unreachable pattern" warnings) — breaking global
// item parsing entirely. Must be defined for the arms to be real const patterns.
const ITEM_TAG_GLOBAL_USAGE_PAGE: i32 = 0x0;
const ITEM_TAG_GLOBAL_LOGICAL_MINIMUM: i32 = 0x1;
const ITEM_TAG_GLOBAL_LOGICAL_MAXIMUM: i32 = 0x2;
const ITEM_TAG_GLOBAL_PHYSICAL_MINIMUM: i32 = 0x3;
const ITEM_TAG_GLOBAL_PHYSICAL_MAXIMUM: i32 = 0x4;
const ITEM_TAG_GLOBAL_UNIT_EXPONENT: i32 = 0x5;
const ITEM_TAG_GLOBAL_UNIT: i32 = 0x6;
const ITEM_TAG_GLOBAL_REPORT_SIZE: i32 = 0x7;
const ITEM_TAG_GLOBAL_REPORT_ID: i32 = 0x8;
const ITEM_TAG_GLOBAL_REPORT_COUNT: i32 = 0x9;
const ITEM_TAG_GLOBAL_PUSH: i32 = 0xa;
const ITEM_TAG_GLOBAL_POP: i32 = 0xb;
// Local item tags (parser.h lines 44-55).
const ITEM_TAG_LOCAL_USAGE: i32 = 0x0;
const ITEM_TAG_LOCAL_USAGE_MINIMUM: i32 = 0x1;
const ITEM_TAG_LOCAL_USAGE_MAXIMUM: i32 = 0x2;
const ITEM_TAG_LOCAL_DESIGNATOR_INDEX: i32 = 0x3;
const ITEM_TAG_LOCAL_DESIGNATOR_MINIMUM: i32 = 0x4;
const ITEM_TAG_LOCAL_DESIGNATOR_MAXIMUM: i32 = 0x5;
const ITEM_TAG_LOCAL_STRING_INDEX: i32 = 0x7;
const ITEM_TAG_LOCAL_STRING_MINIMUM: i32 = 0x8;
const ITEM_TAG_LOCAL_STRING_MAXIMUM: i32 = 0x9;
const ITEM_TAG_LOCAL_DELIMITER: i32 = 0xa;
const ITEM_TAG_LONG: i32 = 0xf;
const HID_REPORT_TYPE_ANY: u8 = 0x07;
const HID_REPORT_TYPE_INPUT: u8 = 0x01;
const HID_REPORT_TYPE_OUTPUT: u8 = 0x02;
const HID_REPORT_TYPE_FEATURE: u8 = 0x04;
const TRUE: u8 = 1;
const FALSE: u8 = 0;
const COLLECTION_LOGICAL: u8 = 1;

// Item size array (external static, defined in file scope)
#[no_mangle]
pub unsafe extern "C" fn HidParser_DeleteReport(Report: PHID_REPORT) {}
#[no_mangle]
pub unsafe extern "C" fn HidParser_FreeCollection(mut Collection: PHID_COLLECTION) {}
#[no_mangle]
pub unsafe extern "C" fn HidParser_AllocateCollection(
    ParentCollection: *mut HID_COLLECTION,
    Type: u8,
    LocalItemState: *mut LOCAL_ITEM_STATE,
    OutCollection: *mut *mut HID_COLLECTION,
) -> NTSTATUS {
    let mut Collection: *mut HID_COLLECTION = core::ptr::null_mut();
    let mut UsageValue: USAGE_VALUE = unsafe { core::mem::zeroed() };

    Collection = AllocFunction(core::mem::size_of::<HID_COLLECTION>() as u32) as *mut HID_COLLECTION;
    if Collection.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }

    (*Collection).Root = ParentCollection;
    (*Collection).Type = Type;
    (*Collection).StringID = (*LocalItemState).StringIndex;
    (*Collection).PhysicalID = (*LocalItemState).DesignatorIndex as u8;

    if LocalItemState.is_null() {
        RtlAssert(
            b"LocalItemState\0".as_ptr() as *mut core::ffi::c_void,
            b"parser.c\0".as_ptr() as *mut core::ffi::c_void,
            69,
            core::ptr::null_mut(),
        );
    }
    if (*LocalItemState).UsageStack.is_null() {
        RtlAssert(
            b"LocalItemState->UsageStack\0".as_ptr() as *mut core::ffi::c_void,
            b"parser.c\0".as_ptr() as *mut core::ffi::c_void,
            70,
            core::ptr::null_mut(),
        );
    }

    if (*LocalItemState).UsageStackUsed > 0 {
        UsageValue.u.Extended = (*(*LocalItemState).UsageStack).u.Extended;
    } else if (*LocalItemState).UsageMinimumSet != 0 {
        UsageValue.u.Extended = (*LocalItemState).UsageMinimum.u.Extended;
    } else if (*LocalItemState).UsageMaximumSet != 0 {
        UsageValue.u.Extended = (*LocalItemState).UsageMaximum.u.Extended;
    } else if Type as i32 == COLLECTION_LOGICAL as i32 as i32 {
        UsageValue.u.Extended = 0;
    } else {
        DebugFunction(b"HIDPARSE] No usage set\n\0".as_ptr() as *const i8);
        UsageValue.u.Extended = 0;
    }

    (*Collection).Usage = UsageValue.u.Extended;
    core::ptr::write_volatile(OutCollection, Collection);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_AddCollection(
    mut CurrentCollection: PHID_COLLECTION,
    mut NewCollection: PHID_COLLECTION,
) -> NTSTATUS {
    let mut NewAllocCollection: *mut PHID_COLLECTION = core::ptr::null_mut();
    let mut CollectionCount: u32 = 0;

    CollectionCount = ((*CurrentCollection).NodeCount).wrapping_add(1);
    NewAllocCollection = AllocFunction(
        (core::mem::size_of::<PHID_COLLECTION>() as u32).wrapping_mul(CollectionCount),
    ) as *mut PHID_COLLECTION;
    if NewAllocCollection.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }
    if (*CurrentCollection).NodeCount != 0 {
        CopyFunction(
            NewAllocCollection as *mut core::ffi::c_void,
            (*CurrentCollection).Nodes as *const core::ffi::c_void,
            ((*CurrentCollection).NodeCount as u32)
                .wrapping_mul(core::mem::size_of::<PHID_COLLECTION>() as u32),
        );
        FreeFunction((*CurrentCollection).Nodes as *mut core::ffi::c_void);
    }
    let fresh2 = &mut *NewAllocCollection.offset((*CurrentCollection).NodeCount as isize);
    *fresh2 = NewCollection as *mut __HID_COLLECTION__ as PHID_COLLECTION;
    (*CurrentCollection).Nodes = NewAllocCollection as *mut *mut __HID_COLLECTION__;
    (*CurrentCollection).NodeCount = (*CurrentCollection).NodeCount.wrapping_add(1);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_FindReportInCollection(
    mut Collection: *mut HID_COLLECTION,
    mut ReportType: u8,
    mut ReportID: u8,
    mut OutReport: *mut *mut HID_REPORT,
) -> NTSTATUS {
    let mut Index: u32 = 0;
    let mut Status: NTSTATUS = 0;
    Index = 0u32;
    while Index < (*Collection).ReportCount {
        if (**(*Collection).Reports.offset(Index as isize)).Type as core::ffi::c_int
            == ReportType as core::ffi::c_int
            && (**(*Collection).Reports.offset(Index as isize)).ReportID as core::ffi::c_int
                == ReportID as core::ffi::c_int
        {
            *OutReport = *(*Collection).Reports.offset(Index as isize) as *mut HID_REPORT;
            return HIDP_STATUS_SUCCESS;
        }
        Index = Index.wrapping_add(1);
    }
    Index = 0u32;
    while Index < (*Collection).NodeCount {
        Status = HidParser_FindReportInCollection(
            *(*Collection).Nodes.offset(Index as isize) as *mut HID_COLLECTION,
            ReportType,
            ReportID,
            OutReport,
        );
        if Status == HIDP_STATUS_SUCCESS {
            return Status;
        }
        Index = Index.wrapping_add(1);
    }
    *OutReport = core::ptr::null_mut::<HID_REPORT>();
    return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_FindReport(
    ParserContext: *mut HID_PARSER_CONTEXT,
    ReportType: u8,
    ReportID: u8,
    OutReport: *mut PHID_REPORT,
) -> NTSTATUS {
    return HidParser_FindReportInCollection(
        *((*(*ParserContext).RootCollection).Nodes.offset(
            ((*(*ParserContext).RootCollection).NodeCount as u32)
                .wrapping_sub(1u32) as isize,
        )) as PHID_COLLECTION,
        ReportType,
        ReportID,
        OutReport,
    );
}

#[no_mangle]
pub unsafe extern "C" fn HidParser_AllocateReport(
    mut ReportType: u8,
    mut ReportID: u8,
    mut OutReport: *mut PHID_REPORT,
) -> NTSTATUS {
    // HIDP_STATUS_SUCCESS / HIDP_STATUS_INTERNAL_ERROR imported from hid_sys.
    let mut Report: PHID_REPORT = core::ptr::null_mut::<HID_REPORT>();
    Report = AllocFunction(core::mem::size_of::<HID_REPORT>() as u32) as PHID_REPORT;
    if Report.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }
    (*Report).ReportID = ReportID;
    (*Report).Type = ReportType;
    core::ptr::write_volatile(OutReport, Report);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_AddReportToCollection(
    ParserContext: *mut HID_PARSER_CONTEXT,
    CurrentCollection: *mut HID_COLLECTION,
    NewReport: *mut HID_REPORT,
) -> NTSTATUS {
    let mut NewReportArray: *mut *mut HID_REPORT = core::ptr::null_mut::<*mut HID_REPORT>();
    NewReportArray = AllocFunction(
        (core::mem::size_of::<*mut HID_REPORT>() as u32)
            .wrapping_mul((*CurrentCollection).ReportCount.wrapping_add(1)),
    ) as *mut *mut HID_REPORT;
    if NewReportArray.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }
    if (*CurrentCollection).ReportCount != 0 {
        CopyFunction(
            NewReportArray as *mut core::ffi::c_void,
            (*CurrentCollection).Reports as *const core::ffi::c_void,
            (core::mem::size_of::<*mut HID_REPORT>() as u32)
                .wrapping_mul((*CurrentCollection).ReportCount),
        );
        FreeFunction((*CurrentCollection).Reports as *mut core::ffi::c_void);
    }
    let ref mut fresh1 = *NewReportArray.offset((*CurrentCollection).ReportCount as isize);
    *fresh1 = NewReport;
    (*CurrentCollection).Reports = NewReportArray as *mut *mut _HID_REPORT;
    (*CurrentCollection).ReportCount = (*CurrentCollection).ReportCount.wrapping_add(1);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetReport(
    mut ParserContext: PHID_PARSER_CONTEXT,
    mut Collection: PHID_COLLECTION,
    mut ReportType: u8,
    mut ReportID: u8,
    mut CreateIfNotExists: u8,
    mut OutReport: *mut PHID_REPORT,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    Status = HidParser_FindReport(ParserContext, ReportType, ReportID, OutReport);
    if Status == HIDP_STATUS_SUCCESS || CreateIfNotExists as i32 == 0 {
        return Status;
    }
    Status = HidParser_AllocateReport(ReportType, ReportID, OutReport);
    if Status != HIDP_STATUS_SUCCESS {
        return Status;
    }
    Status = HidParser_AddReportToCollection(ParserContext, Collection, *OutReport);
    if Status != HIDP_STATUS_SUCCESS {
        FreeFunction(*OutReport as *mut core::ffi::c_void);
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_ReserveReportItems(
    Report: *mut HID_REPORT,
    ReportCount: u32,
    OutReport: *mut *mut HID_REPORT,
) -> i32 {
    let mut NewReport: *mut HID_REPORT = core::ptr::null_mut();
    let mut OldSize: u32 = 0;
    let mut Size: u32 = 0;
    if (*Report).ItemCount.wrapping_add(ReportCount) <= (*Report).ItemAllocated {
        core::ptr::write_volatile(OutReport, Report);
        return HIDP_STATUS_SUCCESS;
    }
    OldSize = (core::mem::size_of::<HID_REPORT>() as u32)
        .wrapping_add(((*Report).ItemCount as u32).wrapping_mul(core::mem::size_of::<HID_REPORT_ITEM>() as u32));
    Size = (ReportCount as u32).wrapping_mul(core::mem::size_of::<HID_REPORT_ITEM>() as u32);
    NewReport = AllocFunction(Size.wrapping_add(OldSize)) as *mut HID_REPORT;
    if NewReport.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }
    CopyFunction(NewReport as *mut core::ffi::c_void, Report as *const core::ffi::c_void, OldSize);
    (*NewReport).ItemAllocated = (*NewReport).ItemAllocated.wrapping_add(ReportCount);
    core::ptr::write_volatile(OutReport, NewReport);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_InitReportItem(
    mut Report: PHID_REPORT,
    mut ReportItem: PHID_REPORT_ITEM,
    mut GlobalItemState: PGLOBAL_ITEM_STATE,
    mut LocalItemState: PLOCAL_ITEM_STATE,
    mut ItemData: PMAIN_ITEM_DATA,
    mut ReportItemIndex: ULONG,
) -> NTSTATUS {
    let mut LogicalMinimum: ULONG;
    let mut LogicalMaximum: ULONG;
    let mut PhysicalMinimum: ULONG;
    let mut PhysicalMaximum: ULONG;
    let mut UsageMinimum: ULONG;
    let mut UsageMaximum: ULONG;
    let mut UsageValue: USAGE_VALUE;

    LogicalMinimum = (*GlobalItemState).LogicalMinimum;
    LogicalMaximum = (*GlobalItemState).LogicialMaximum;
    if LogicalMinimum > LogicalMaximum {
        HidParser_SignRange(
            LogicalMinimum,
            LogicalMaximum,
            &raw mut LogicalMinimum,
            &raw mut LogicalMaximum,
        );
    }

    PhysicalMinimum = (*GlobalItemState).PhysicalMinimum;
    PhysicalMaximum = (*GlobalItemState).PhysicalMaximum;
    if PhysicalMinimum > PhysicalMaximum {
        HidParser_SignRange(
            PhysicalMinimum,
            PhysicalMaximum,
            &raw mut PhysicalMinimum,
            &raw mut PhysicalMaximum,
        );
    }

    UsageMinimum = 0 as ULONG;
    UsageMaximum = 0 as ULONG;
    if (*ItemData).ArrayVariable() as i32 == 0 {
        UsageMinimum = (*LocalItemState).UsageMinimum.u.Extended;
        UsageMaximum = (*LocalItemState).UsageMaximum.u.Extended;
    } else {
        if ReportItemIndex < (*LocalItemState).UsageStackUsed {
            UsageValue = *((*LocalItemState).UsageStack.offset(ReportItemIndex as isize)) as USAGE_VALUE;
        } else {
            UsageValue = (*LocalItemState).UsageMinimum;
            UsageValue.u.Extended = (UsageValue.u.Extended as u32).wrapping_add(ReportItemIndex as u32) as ULONG;
            if (*LocalItemState).UsageMaximumSet != 0 {
                if UsageValue.u.Extended > (*LocalItemState).UsageMaximum.u.Extended {
                    UsageValue.u.Extended = (*LocalItemState).UsageMaximum.u.Extended;
                }
            }
        }
        UsageMaximum = UsageValue.u.Extended;
        UsageMinimum = UsageMaximum;
    }

    (*ReportItem).ByteOffset = ((*Report).ReportSize / 8) as ULONG;
    (*ReportItem).Shift = ((*Report).ReportSize % 8) as UCHAR;
    (*ReportItem).Mask = !(0xFFFFFFFFu32 << (*GlobalItemState).ReportSize) as ULONG;
    (*ReportItem).BitCount = (*GlobalItemState).ReportSize as UCHAR;
    (*ReportItem).HasData = ((*ItemData).DataConstant() as i32 == 0) as UCHAR;
    (*ReportItem).Array = ((*ItemData).ArrayVariable() as i32 == 0) as UCHAR;
    (*ReportItem).Relative = ((*ItemData).Relative() as i32 != 0) as UCHAR;
    (*ReportItem).Minimum = LogicalMinimum;
    (*ReportItem).Maximum = LogicalMaximum;
    (*ReportItem).UsageMinimum = UsageMinimum;
    (*ReportItem).UsageMaximum = UsageMaximum;

    (*Report).ReportSize = ((*Report).ReportSize as u32).wrapping_add((*GlobalItemState).ReportSize as u32) as ULONG;

    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_UpdateCurrentCollectionReport(
    Collection: PHID_COLLECTION,
    Report: PHID_REPORT,
    NewReport: PHID_REPORT,
) -> i32 {
    let mut Index: u32;
    let mut Found: i32 = 0;
    let mut TempFound: i32;
    Index = 0u32;
    while Index < (*Collection).ReportCount {
        if *(*Collection).Reports.offset(Index as isize) == Report {
            let ref mut fresh0 = *(*Collection).Reports.offset(Index as isize);
            *fresh0 = NewReport;
            Found = 1i32;
        }
        Index = Index.wrapping_add(1);
    }
    Index = 0u32;
    while Index < (*Collection).NodeCount {
        TempFound = HidParser_UpdateCurrentCollectionReport(
            *(*Collection).Nodes.offset(Index as isize) as PHID_COLLECTION,
            Report,
            NewReport,
        );
        if TempFound != 0 {
            if !(Found as i32 == 0) {
                RtlAssert(
                    b"Found == FALSE\0".as_ptr() as *const ::core::ffi::c_void,
                    b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const ::core::ffi::c_void,
                    619u32,
                    ::core::ptr::null_mut::<core::ffi::c_void>(),
                );
            }
            Found = 1i32;
        }
        Index = Index.wrapping_add(1);
    }
    return Found;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_UpdateCollectionReport(
    mut ParserContext: PHID_PARSER_CONTEXT,
    mut Report: PHID_REPORT,
    mut NewReport: PHID_REPORT,
) -> BOOLEAN {
    return HidParser_UpdateCurrentCollectionReport(
        (*(*(*ParserContext).RootCollection).Nodes.offset(
            ((*(*ParserContext).RootCollection).NodeCount as usize).wrapping_sub(1) as isize
        )) as PHID_COLLECTION,
        Report,
        NewReport,
    ) as BOOLEAN; // inner returns i32 status; this wrapper is BOOLEAN (C: BOOLEAN)
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_AddMainItem(
    ParserContext: PHID_PARSER_CONTEXT,
    Report: PHID_REPORT,
    GlobalItemState: PGLOBAL_ITEM_STATE,
    LocalItemState: PLOCAL_ITEM_STATE,
    ItemData: PMAIN_ITEM_DATA,
    Collection: PHID_COLLECTION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Index: ULONG = 0;
    let mut NewReport: PHID_REPORT = core::ptr::null_mut();
    let mut Found: BOOLEAN = 0;

    Status = HidParser_ReserveReportItems(Report, (*GlobalItemState).ReportCount, &raw mut NewReport);
    if Status != HIDP_STATUS_SUCCESS {
        return Status;
    }
    if NewReport != Report {
        Found = HidParser_UpdateCollectionReport(ParserContext, Report, NewReport);
        if Found == 0 {
            RtlAssert(
                b"Found\0".as_ptr() as *const core::ffi::c_void as PVOID,
                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *mut core::ffi::c_void,
                675 as ULONG,
                core::ptr::null_mut(),
            );
        } else {
        };
    }
    if !((*NewReport).ItemCount.wrapping_add((*GlobalItemState).ReportCount) <= (*NewReport).ItemAllocated) {
        RtlAssert(
            b"NewReport->ItemCount + GlobalItemState->ReportCount <= NewReport->ItemAllocated\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *mut core::ffi::c_void,
            681 as ULONG,
            core::ptr::null_mut(),
        );
    } else {
    };

    Index = 0 as ULONG;
    while Index < (*GlobalItemState).ReportCount {
        Status = HidParser_InitReportItem(
            NewReport,
            (&raw mut (*NewReport).Items as *mut HID_REPORT_ITEM)
                .offset((*NewReport).ItemCount as isize) as PHID_REPORT_ITEM,
            GlobalItemState,
            LocalItemState,
            ItemData,
            Index,
        );
        if Status != HIDP_STATUS_SUCCESS {
            return Status;
        }
        (*NewReport).ItemCount = (*NewReport).ItemCount.wrapping_add(1);
        Index = Index.wrapping_add(1);
    }

    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_ParseReportDescriptor(
    ReportDescriptor: *mut u8,
    ReportLength: u32,
    OutParser: *mut *mut core::ffi::c_void,
) -> i32 {
    let mut LinkedGlobalItemState: *mut GLOBAL_ITEM_STATE = core::ptr::null_mut();
    let mut NextLinkedGlobalItemState: *mut GLOBAL_ITEM_STATE = core::ptr::null_mut();
    let mut Index: u32 = 0;
    let mut NewUsageStack: *mut usage_value = core::ptr::null_mut();
    let mut UsageValue: *mut usage_value = core::ptr::null_mut();
    let mut Status: i32 = 0;
    let mut CurrentCollection: *mut __HID_COLLECTION__ = core::ptr::null_mut();
    let mut NewCollection: *mut __HID_COLLECTION__ = core::ptr::null_mut();
    let mut CurrentOffset: *mut u8 = core::ptr::null_mut();
    let mut ReportEnd: *mut u8 = core::ptr::null_mut();
    let mut CurrentItem: *mut ITEM_PREFIX = core::ptr::null_mut();
    let mut CurrentItemSize: u32 = 0;
    let mut CurrentLongItem: *mut C2RustUnnamed_1 = core::ptr::null_mut();
    let mut CurrentShortItem: *mut C2RustUnnamed = core::ptr::null_mut();
    let mut Data: u32 = 0;
    let mut ReportType: u8 = 0;
    let mut Report: *mut _HID_REPORT = core::ptr::null_mut();
    let mut MainItemData: *mut C2RustUnnamed_2 = core::ptr::null_mut();
    let mut ParserContext: *mut HID_PARSER_CONTEXT = core::ptr::null_mut();

    CurrentOffset = ReportDescriptor;
    ReportEnd = ReportDescriptor.offset(ReportLength as isize);

    if ReportDescriptor >= ReportEnd {
        return HIDP_STATUS_USAGE_NOT_FOUND;
    }

    ParserContext = AllocFunction(core::mem::size_of::<HID_PARSER_CONTEXT>() as u32) as *mut HID_PARSER_CONTEXT;
    if ParserContext.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }

    (*ParserContext).LocalItemState.UsageStackAllocated = 10 as u32;
    (*ParserContext).LocalItemState.UsageStack = AllocFunction(
        ((*ParserContext).LocalItemState.UsageStackAllocated as usize)
            .wrapping_mul(core::mem::size_of::<USAGE_VALUE>() as usize) as u32,
    ) as *mut usage_value;
    if (*ParserContext).LocalItemState.UsageStack.is_null() {
        FreeFunction(ParserContext as *mut core::ffi::c_void);
        return HIDP_STATUS_INTERNAL_ERROR;
    }

    Status = HidParser_AllocateCollection(
        core::ptr::null_mut::<__HID_COLLECTION__>(),
        COLLECTION_LOGICAL,
        core::ptr::addr_of_mut!((*ParserContext).LocalItemState),
        core::ptr::addr_of_mut!((*ParserContext).RootCollection),
    );
    if Status != HIDP_STATUS_SUCCESS {
        FreeFunction((*ParserContext).LocalItemState.UsageStack as *mut core::ffi::c_void);
        (*ParserContext).LocalItemState.UsageStack = core::ptr::null_mut::<usage_value>();
        FreeFunction(ParserContext as *mut core::ffi::c_void);
        return HIDP_STATUS_INTERNAL_ERROR;
    }

    CurrentCollection = (*ParserContext).RootCollection;

    loop {
        CurrentItem = CurrentOffset as *mut ITEM_PREFIX;
        CurrentItemSize = ItemSize[(*CurrentItem).Size() as usize] as u32;
        Data = 0u32;

        if (*CurrentItem).Type() as i32 == ITEM_TYPE_LONG {
            CurrentLongItem = CurrentItem as *mut C2RustUnnamed_1;
            CurrentItemSize = (CurrentItemSize as u32).wrapping_add((*CurrentLongItem).DataSize as u32);
        } else {
            CurrentShortItem = CurrentItem as *mut C2RustUnnamed;
            if CurrentItemSize == 1u32 {
                Data = (*CurrentShortItem).Data.UData8[0usize] as u32;
            } else if CurrentItemSize == 2u32 {
                Data = (*CurrentShortItem).Data.UData16[0usize] as u32;
            } else if CurrentItemSize == 4u32 {
                Data = (*CurrentShortItem).Data.UData32;
            }
        }

        DebugFunction(
            b"Tag %x Type %x Size %x Offset %lu Length %lu\n\0".as_ptr() as *const i8,
            (*CurrentItem).Tag() as i32,
            (*CurrentItem).Type() as i32,
            (*CurrentItem).Size() as i32,
            (CurrentItem as usize).wrapping_sub(ReportDescriptor as usize) as u32,
            ReportLength,
        );

        // ASSERT equivalent
        if !((*CurrentItem).Type() as i32 >= ITEM_TYPE_MAIN && (*CurrentItem).Type() as i32 <= ITEM_TYPE_LONG) {
            RtlAssert(
                b"CurrentItem->Type >= ITEM_TYPE_MAIN && CurrentItem->Type <= ITEM_TYPE_LONG\0".as_ptr() as *const core::ffi::c_void,
                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                827u32,
                core::ptr::null_mut(),
            );
        }

        let mut current_block_152: u64 = 0;
        match (*CurrentItem).Type() as i32 {
            ITEM_TYPE_MAIN => {
                if (*CurrentItem).Tag() as i32 != ITEM_TAG_MAIN_END_COLLECTION {
                    Index = 0u32;
                    while Index < (*ParserContext).LocalItemState.UsageStackUsed {
                        if !((*(*ParserContext).LocalItemState.UsageStack.offset(Index as isize)).IsExtended != 0) {
                            (*(*ParserContext).LocalItemState.UsageStack.offset(Index as isize)).u.s.UsagePage = (*ParserContext).GlobalItemState.UsagePage;
                            (*(*ParserContext).LocalItemState.UsageStack.offset(Index as isize)).IsExtended = TRUE;
                        }
                        Index = Index.wrapping_add(1);
                    }
                    if (*ParserContext).LocalItemState.UsageMinimum.IsExtended == 0 {
                        (*ParserContext).LocalItemState.UsageMaximum.u.s.UsagePage = (*ParserContext).GlobalItemState.UsagePage;
                        (*ParserContext).LocalItemState.UsageMinimum.u.s.UsagePage = (*ParserContext).LocalItemState.UsageMaximum.u.s.UsagePage;
                        (*ParserContext).LocalItemState.UsageMaximum.IsExtended = TRUE;
                        (*ParserContext).LocalItemState.UsageMinimum.IsExtended = (*ParserContext).LocalItemState.UsageMaximum.IsExtended;
                    }
                }
                if (*CurrentItem).Tag() as i32 == ITEM_TAG_MAIN_COLLECTION {
                    Status = HidParser_AllocateCollection(
                        CurrentCollection,
                        Data as u8,
                        core::ptr::addr_of_mut!((*ParserContext).LocalItemState),
                        core::ptr::addr_of_mut!(NewCollection),
                    );
                    if Status != HIDP_STATUS_SUCCESS {
                        RtlAssert(
                            b"Status == HIDP_STATUS_SUCCESS\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            873u32,
                            core::ptr::null_mut(),
                        );
                    }
                    Status = HidParser_AddCollection(CurrentCollection, NewCollection);
                    if Status != HIDP_STATUS_SUCCESS {
                        RtlAssert(
                            b"Status == HIDP_STATUS_SUCCESS\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            879u32,
                            core::ptr::null_mut(),
                        );
                    }
                    CurrentCollection = NewCollection;
                    current_block_152 = 7189308829251266000;
                } else if (*CurrentItem).Tag() as i32 == ITEM_TAG_MAIN_END_COLLECTION {
                    if !(CurrentCollection != (*ParserContext).RootCollection) {
                        RtlAssert(
                            b"CurrentCollection != ParserContext->RootCollection\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            891u32,
                            core::ptr::null_mut(),
                        );
                    }
                    CurrentCollection = (*CurrentCollection).Root as *mut __HID_COLLECTION__;
                    if CurrentCollection.is_null() {
                        RtlAssert(
                            b"CurrentCollection\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            897u32,
                            core::ptr::null_mut(),
                        );
                    }
                    current_block_152 = 7189308829251266000;
                } else {
                    ReportType = HID_REPORT_TYPE_ANY;
                    match (*CurrentItem).Tag() as i32 {
                        ITEM_TAG_MAIN_INPUT => {
                            ReportType = HID_REPORT_TYPE_INPUT;
                        }
                        ITEM_TAG_MAIN_OUTPUT => {
                            ReportType = HID_REPORT_TYPE_OUTPUT;
                        }
                        ITEM_TAG_MAIN_FEATURE => {
                            ReportType = HID_REPORT_TYPE_FEATURE;
                        }
                        _ => {
                            DebugFunction(
                                b"[HIDPARSE] Unknown ReportType Tag %x Type %x Size %x CurrentItemSize %x\n\0".as_ptr() as *const i8,
                                (*CurrentItem).Tag() as i32,
                                (*CurrentItem).Type() as i32,
                                (*CurrentItem).Size() as i32,
                                CurrentItemSize,
                            );
                            if FALSE == 0 {
                                RtlAssert(
                                    b"FALSE\0".as_ptr() as *const core::ffi::c_void,
                                    b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                                    918u32,
                                    core::ptr::null_mut(),
                                );
                            }
                        }
                    }
                    if ReportType == HID_REPORT_TYPE_ANY {
                        current_block_152 = 7315983924538012637;
                    } else {
                        Status = HidParser_GetReport(
                            ParserContext,
                            CurrentCollection,
                            ReportType,
                            (*ParserContext).GlobalItemState.ReportId,
                            TRUE,
                            core::ptr::addr_of_mut!(Report),
                        );
                        if Status != HIDP_STATUS_SUCCESS {
                            RtlAssert(
                                b"Status == HIDP_STATUS_SUCCESS\0".as_ptr() as *const core::ffi::c_void,
                                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                                929u32,
                                core::ptr::null_mut(),
                            );
                        }
                        if (*ParserContext).LocalItemState.DesignatorIndexSet == 0 {
                            (*ParserContext).LocalItemState.DesignatorIndex = (*ParserContext).LocalItemState.DesignatorMinimum;
                        }
                        if (*ParserContext).LocalItemState.StringIndexSet == 0 {
                            (*ParserContext).LocalItemState.StringIndex = (*ParserContext).LocalItemState.StringMinimum;
                        }
                        MainItemData = core::ptr::addr_of_mut!(Data) as *mut C2RustUnnamed_2;
                        Status = HidParser_AddMainItem(
                            ParserContext,
                            Report,
                            core::ptr::addr_of_mut!((*ParserContext).GlobalItemState),
                            core::ptr::addr_of_mut!((*ParserContext).LocalItemState),
                            MainItemData,
                            CurrentCollection,
                        );
                        if Status != HIDP_STATUS_SUCCESS {
                            RtlAssert(
                                b"Status == HIDP_STATUS_SUCCESS\0".as_ptr() as *const core::ffi::c_void,
                                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                                949u32,
                                core::ptr::null_mut(),
                            );
                        }
                        current_block_152 = 7189308829251266000;
                    }
                }
                if current_block_152 != 7315983924538012637 {
                    Index = (*ParserContext).LocalItemState.UsageStackAllocated;
                    NewUsageStack = (*ParserContext).LocalItemState.UsageStack;
                    ZeroFunction(
                        core::ptr::addr_of_mut!((*ParserContext).LocalItemState) as *mut core::ffi::c_void,
                        core::mem::size_of::<LOCAL_ITEM_STATE>() as u32,
                    );
                    (*ParserContext).LocalItemState.UsageStack = NewUsageStack;
                    (*ParserContext).LocalItemState.UsageStackAllocated = Index;
                }
            }
            ITEM_TYPE_GLOBAL => match (*CurrentItem).Tag() as i32 {
                ITEM_TAG_GLOBAL_USAGE_PAGE => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_USAGE_PAGE %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.UsagePage = Data as u16;
                }
                ITEM_TAG_GLOBAL_LOGICAL_MINIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_LOGICAL_MINIMUM %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.LogicalMinimum = Data;
                }
                ITEM_TAG_GLOBAL_LOGICAL_MAXIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_LOCAL_MAXIMUM %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.LogicialMaximum = Data;
                }
                ITEM_TAG_GLOBAL_PHYSICAL_MINIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_PHYSICAL_MINIMUM %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.PhysicalMinimum = Data;
                }
                ITEM_TAG_GLOBAL_PHYSICAL_MAXIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_PHYSICAL_MAXIMUM %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.PhysicalMaximum = Data;
                }
                ITEM_TAG_GLOBAL_UNIT_EXPONENT => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_UNIT_EXPONENT %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.UnitExponent = Data as u8;
                }
                ITEM_TAG_GLOBAL_UNIT => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_UNIT %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.Unit = Data as u8;
                }
                ITEM_TAG_GLOBAL_REPORT_SIZE => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_SIZE %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.ReportSize = Data;
                }
                ITEM_TAG_GLOBAL_REPORT_ID => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_ID %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.ReportId = Data as u8;
                    (*ParserContext).UseReportIDs = TRUE;
                }
                ITEM_TAG_GLOBAL_REPORT_COUNT => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_COUNT %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).GlobalItemState.ReportCount = Data;
                }
                ITEM_TAG_GLOBAL_PUSH => {
                    DebugFunction(b"[HIDPARSE] ITEM_TAG_GLOBAL_PUSH\n\0".as_ptr() as *const i8);
                    LinkedGlobalItemState = AllocFunction(core::mem::size_of::<GLOBAL_ITEM_STATE>() as u32) as *mut GLOBAL_ITEM_STATE;
                    if LinkedGlobalItemState.is_null() {
                        RtlAssert(
                            b"LinkedGlobalItemState\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            1030u32,
                            core::ptr::null_mut(),
                        );
                    }
                    CopyFunction(
                        LinkedGlobalItemState as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!((*ParserContext).GlobalItemState) as *mut core::ffi::c_void,
                        core::mem::size_of::<GLOBAL_ITEM_STATE>() as u32,
                    );
                    (*ParserContext).GlobalItemState.Next = LinkedGlobalItemState as *mut __GLOBAL_ITEM_STATE__;
                }
                ITEM_TAG_GLOBAL_POP => {
                    DebugFunction(b"[HIDPARSE] ITEM_TAG_GLOBAL_POP\n\0".as_ptr() as *const i8);
                    if (*ParserContext).GlobalItemState.Next.is_null() {
                        if FALSE == 0 {
                            RtlAssert(
                                b"FALSE\0".as_ptr() as *const core::ffi::c_void,
                                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                                1051u32,
                                core::ptr::null_mut(),
                            );
                        }
                    } else {
                        LinkedGlobalItemState = (*ParserContext).GlobalItemState.Next as *mut GLOBAL_ITEM_STATE;
                        CopyFunction(
                            core::ptr::addr_of_mut!((*ParserContext).GlobalItemState) as *mut core::ffi::c_void,
                            LinkedGlobalItemState as *mut core::ffi::c_void,
                            core::mem::size_of::<GLOBAL_ITEM_STATE>() as u32,
                        );
                        FreeFunction(LinkedGlobalItemState as *mut core::ffi::c_void);
                    }
                }
                _ => {
                    if FALSE == 0 {
                        RtlAssert(
                            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            1076u32,
                            core::ptr::null_mut(),
                        );
                    }
                }
            },
            ITEM_TYPE_LOCAL => match (*CurrentItem).Tag() as i32 {
                ITEM_TAG_LOCAL_USAGE => {
                    if (*ParserContext).LocalItemState.UsageStackUsed >= (*ParserContext).LocalItemState.UsageStackAllocated {
                        (*ParserContext).LocalItemState.UsageStackAllocated = ((*ParserContext).LocalItemState.UsageStackAllocated as u32).wrapping_add(10u32);
                        NewUsageStack = AllocFunction(
                            (core::mem::size_of::<USAGE_VALUE>() as usize)
                                .wrapping_mul((*ParserContext).LocalItemState.UsageStackAllocated as usize) as u32,
                        ) as *mut usage_value;
                        if NewUsageStack.is_null() {
                            RtlAssert(
                                b"NewUsageStack\0".as_ptr() as *const core::ffi::c_void,
                                b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                                1099u32,
                                core::ptr::null_mut(),
                            );
                        }
                        CopyFunction(
                            NewUsageStack as *mut core::ffi::c_void,
                            (*ParserContext).LocalItemState.UsageStack as *mut core::ffi::c_void,
                            (core::mem::size_of::<USAGE_VALUE>() as usize)
                                .wrapping_mul(((*ParserContext).LocalItemState.UsageStackAllocated as u32).wrapping_sub(10u32) as usize) as u32,
                        );
                        FreeFunction((*ParserContext).LocalItemState.UsageStack as *mut core::ffi::c_void);
                        (*ParserContext).LocalItemState.UsageStack = NewUsageStack;
                    }
                    UsageValue = (*ParserContext)
                        .LocalItemState
                        .UsageStack
                        .offset((*ParserContext).LocalItemState.UsageStackUsed as isize) as *mut usage_value;
                    (*UsageValue).IsExtended = (CurrentItemSize as usize == core::mem::size_of::<u32>() as usize) as u8;
                    (*UsageValue).u.Extended = Data;
                    (*ParserContext).LocalItemState.UsageStackUsed = (*ParserContext)
                        .LocalItemState
                        .UsageStackUsed
                        .wrapping_add(1);
                }
                ITEM_TAG_LOCAL_USAGE_MINIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_USAGE_MINIMUM Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.UsageMinimum.u.Extended = Data;
                    (*ParserContext).LocalItemState.UsageMinimum.IsExtended =
                        (CurrentItemSize as usize == core::mem::size_of::<u32>() as usize) as u8;
                    (*ParserContext).LocalItemState.UsageMinimumSet = TRUE;
                }
                ITEM_TAG_LOCAL_USAGE_MAXIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_USAGE_MAXIMUM Data %x ItemSize %x %x\n\0".as_ptr() as *const i8,
                        Data,
                        CurrentItemSize,
                        (*CurrentItem).Size() as i32,
                    );
                    (*ParserContext).LocalItemState.UsageMaximum.u.Extended = Data;
                    (*ParserContext).LocalItemState.UsageMaximum.IsExtended =
                        (CurrentItemSize as usize == core::mem::size_of::<u32>() as usize) as u8;
                    (*ParserContext).LocalItemState.UsageMaximumSet = TRUE;
                }
                ITEM_TAG_LOCAL_DESIGNATOR_INDEX => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_INDEX Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.DesignatorIndex = Data;
                    (*ParserContext).LocalItemState.DesignatorIndexSet = TRUE;
                }
                ITEM_TAG_LOCAL_DESIGNATOR_MINIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_MINIMUM Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.DesignatorMinimum = Data;
                }
                ITEM_TAG_LOCAL_DESIGNATOR_MAXIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_MAXIMUM Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.DesignatorMaximum = Data;
                }
                ITEM_TAG_LOCAL_STRING_INDEX => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_STRING_INDEX Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.StringIndex = Data as u8;
                    (*ParserContext).LocalItemState.StringIndexSet = TRUE;
                }
                ITEM_TAG_LOCAL_STRING_MINIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_STRING_MINIMUM Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.StringMinimum = Data as u8;
                }
                ITEM_TAG_LOCAL_STRING_MAXIMUM => {
                    DebugFunction(
                        b"[HIDPARSE] ITEM_TAG_LOCAL_STRING_MAXIMUM Data %x\n\0".as_ptr() as *const i8,
                        Data,
                    );
                    (*ParserContext).LocalItemState.StringMaximum = Data as u8;
                }
                _ => {
                    DebugFunction(
                        b"Unknown Local Item Tag %x\n\0".as_ptr() as *const i8,
                        (*CurrentItem).Tag() as i32,
                    );
                    if FALSE == 0 {
                        RtlAssert(
                            b"FALSE\0".as_ptr() as *const core::ffi::c_void,
                            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *const core::ffi::c_void,
                            1185u32,
                            core::ptr::null_mut(),
                        );
                    }
                }
            },
            ITEM_TYPE_LONG => {
                CurrentLongItem = CurrentItem as *mut C2RustUnnamed_1;
                DebugFunction(
                    b"Unsupported ITEM_TYPE_LONG Tag %x\n\0".as_ptr() as *const i8,
                    (*CurrentLongItem).LongItemTag as i32,
                );
            }
            _ => {}
        }

        CurrentOffset = CurrentOffset.offset(
            (CurrentItemSize as usize).wrapping_add(core::mem::size_of::<ITEM_PREFIX>() as usize) as isize,
        );
        if !(CurrentOffset < ReportEnd) {
            break;
        }
    }

    LinkedGlobalItemState = (*ParserContext).GlobalItemState.Next as *mut GLOBAL_ITEM_STATE;
    while !LinkedGlobalItemState.is_null() {
        DebugFunction(
            b"[HIDPARSE] Freeing GlobalState %p\n\0".as_ptr() as *const i8,
            LinkedGlobalItemState,
        );
        NextLinkedGlobalItemState = (*LinkedGlobalItemState).Next as *mut GLOBAL_ITEM_STATE;
        FreeFunction(LinkedGlobalItemState as *mut core::ffi::c_void);
        LinkedGlobalItemState = NextLinkedGlobalItemState;
    }

    FreeFunction((*ParserContext).LocalItemState.UsageStack as *mut core::ffi::c_void);
    (*ParserContext).LocalItemState.UsageStack = core::ptr::null_mut::<usage_value>();

    core::ptr::write_volatile(OutParser, ParserContext as *mut core::ffi::c_void);

    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetCollection(
    ParserContext: *mut HID_PARSER_CONTEXT,
    CollectionNumber: u32,
) -> *mut HID_COLLECTION {
    if ParserContext.is_null() {
        RtlAssert(
            b"ParserContext\0" as *const u8 as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0"
                as *const u8 as *const core::ffi::c_void,
            1255,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if (*ParserContext).RootCollection.is_null() {
        RtlAssert(
            b"ParserContext->RootCollection\0" as *const u8 as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0"
                as *const u8 as *const core::ffi::c_void,
            1256,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if (*(*ParserContext).RootCollection).NodeCount == 0 {
        RtlAssert(
            b"ParserContext->RootCollection->NodeCount\0" as *const u8 as *const core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0"
                as *const u8 as *const core::ffi::c_void,
            1257,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    }
    if CollectionNumber < (*(*ParserContext).RootCollection).NodeCount {
        return *(*(*ParserContext).RootCollection).Nodes.add(CollectionNumber as usize);
    }
    DebugFunction(
        b"HIDPARSE] No such collection %lu\n\0".as_ptr() as *const i8,
        CollectionNumber,
    );
    return core::ptr::null_mut::<HID_COLLECTION>();
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_NumberOfTopCollections(ParserCtx: *mut core::ffi::c_void) -> u32 {
    let mut ParserContext: *mut HID_PARSER_CONTEXT = core::ptr::null_mut::<HID_PARSER_CONTEXT>();
    ParserContext = ParserCtx as *mut HID_PARSER_CONTEXT;
    if ParserContext.is_null() {
        RtlAssert(
            b"ParserContext\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *mut core::ffi::c_void,
            1292u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    if (*ParserContext).RootCollection.is_null() {
        RtlAssert(
            b"ParserContext->RootCollection\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *mut core::ffi::c_void,
            1293u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    if (*(*ParserContext).RootCollection).NodeCount == 0 {
        RtlAssert(
            b"ParserContext->RootCollection->NodeCount\0".as_ptr() as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0".as_ptr() as *mut core::ffi::c_void,
            1294u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    return (*(*ParserContext).RootCollection).NodeCount;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_BuildContext(
    mut ParserContext: PVOID,
    mut CollectionIndex: ULONG,
    mut ContextSize: ULONG,
    mut CollectionContext: *mut PVOID,
) -> NTSTATUS {
    let mut Collection: PHID_COLLECTION = core::ptr::null_mut::<__HID_COLLECTION__>();
    let mut Context: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Status: NTSTATUS = 0;
    Collection = HidParser_GetCollection(ParserContext as PHID_PARSER_CONTEXT, CollectionIndex);
    if Collection.is_null() {
        RtlAssert(
            b"Collection\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
            b"sdk/lib/drivers/hidparser/parser.c\0"
                .as_ptr() as *const i8 as *mut core::ffi::c_void,
            1317 as ULONG,
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
    } else {
    };
    Context = AllocFunction(ContextSize);
    if Context.is_null() {
        return HIDP_STATUS_INTERNAL_ERROR;
    }
    Status = HidParser_BuildCollectionContext(Collection, Context, ContextSize);
    if Status == HIDP_STATUS_SUCCESS {
        *CollectionContext = Context;
    }
    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetContextSize(
    ParserContext: PVOID,
    CollectionIndex: ULONG,
) -> ULONG {
    let mut Collection: PHID_COLLECTION = core::ptr::null_mut::<HID_COLLECTION>();
    let mut Size: ULONG = 0;
    Collection = HidParser_GetCollection(ParserContext as PHID_PARSER_CONTEXT, CollectionIndex);
    Size = HidParser_CalculateContextSize(Collection);
    return Size;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS HID Parser Library
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        lib/drivers/hidparser/parser.c
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
// static UCHAR ItemSize[4] = { 0, 1, 2, 4 };
//
// VOID
// HidParser_DeleteReport(
//     IN PHID_REPORT Report)
// {
//     //
//     // not implemented
//     //
// }
//
// VOID
// HidParser_FreeCollection(
//     IN PHID_COLLECTION Collection)
// {
//     //
//     // not implemented
//     //
// }
//
// NTSTATUS
// HidParser_AllocateCollection(
//     IN PHID_COLLECTION ParentCollection,
//     IN UCHAR Type,
//     IN PLOCAL_ITEM_STATE LocalItemState,
//     OUT PHID_COLLECTION * OutCollection)
// {
//     PHID_COLLECTION Collection;
//     USAGE_VALUE UsageValue;
//
//     //
//     // first allocate the collection
//     //
//     Collection = (PHID_COLLECTION)AllocFunction(sizeof(HID_COLLECTION));
//     if (!Collection)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     //
//     // init collection
//     //
//     Collection->Root = ParentCollection;
//     Collection->Type = Type;
//     Collection->StringID = LocalItemState->StringIndex;
//     Collection->PhysicalID = LocalItemState->DesignatorIndex;
//
//     //
//     // set Usage
//     //
//     ASSERT(LocalItemState);
//     ASSERT(LocalItemState->UsageStack);
//
//     if (LocalItemState->UsageStackUsed > 0)
//     {
//         //
//         // usage value from first local stack item
//         //
//         UsageValue.u.Extended = LocalItemState->UsageStack[0].u.Extended;
//     }
//     else if (LocalItemState->UsageMinimumSet)
//     {
//         //
//         // use value from minimum
//         //
//         UsageValue.u.Extended = LocalItemState->UsageMinimum.u.Extended;
//     }
//     else if (LocalItemState->UsageMaximumSet)
//     {
//         //
//         // use value from maximum
//         //
//         UsageValue.u.Extended = LocalItemState->UsageMaximum.u.Extended;
//     }
//     else if (Type == COLLECTION_LOGICAL)
//     {
//         //
//         // root collection
//         //
//         UsageValue.u.Extended = 0;
//     }
//     else
//     {
//         //
//         // no usage set
//         //
//         DebugFunction("HIDPARSE] No usage set\n");
//         UsageValue.u.Extended = 0;
//     }
//
//     //
//     // store usage
//     //
//     Collection->Usage = UsageValue.u.Extended;
//
//     //
//     // store result
//     //
//     *OutCollection = Collection;
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_AddCollection(
//     IN PHID_COLLECTION CurrentCollection,
//     IN PHID_COLLECTION NewCollection)
// {
//     PHID_COLLECTION * NewAllocCollection;
//     ULONG CollectionCount;
//
//     //
//     // increment collection array
//     //
//     CollectionCount = CurrentCollection->NodeCount + 1;
//
//     //
//     // allocate new collection
//     //
//     NewAllocCollection = (PHID_COLLECTION*)AllocFunction(sizeof(PHID_COLLECTION) * CollectionCount);
//     if (!NewAllocCollection)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     if (CurrentCollection->NodeCount)
//     {
//         //
//         // copy old array
//         //
//         CopyFunction(NewAllocCollection, CurrentCollection->Nodes, CurrentCollection->NodeCount * sizeof(PHID_COLLECTION));
//
//         //
//         // delete old array
//         //
//         FreeFunction(CurrentCollection->Nodes);
//     }
//
//     //
//     // insert new item
//     //
//     NewAllocCollection[CurrentCollection->NodeCount] = (struct __HID_COLLECTION__*)NewCollection;
//
//
//     //
//     // store new array
//     //
//     CurrentCollection->Nodes = NewAllocCollection;
//     CurrentCollection->NodeCount++;
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_FindReportInCollection(
//     IN PHID_COLLECTION Collection,
//     IN UCHAR ReportType,
//     IN UCHAR ReportID,
//     OUT PHID_REPORT *OutReport)
// {
//     ULONG Index;
//     NTSTATUS Status;
//
//     //
//     // search in local list
//     //
//     for(Index = 0; Index < Collection->ReportCount; Index++)
//     {
//         if (Collection->Reports[Index]->Type == ReportType && Collection->Reports[Index]->ReportID == ReportID)
//         {
//             //
//             // found report
//             //
//             *OutReport = Collection->Reports[Index];
//             return HIDP_STATUS_SUCCESS;
//         }
//     }
//
//     //
//     // search in sub collections
//     //
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         Status = HidParser_FindReportInCollection(Collection->Nodes[Index], ReportType, ReportID, OutReport);
//         if (Status == HIDP_STATUS_SUCCESS)
//             return Status;
//     }
//
//     //
//     // no such report found
//     //
//     *OutReport = NULL;
//     return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
// }
//
//
// NTSTATUS
// HidParser_FindReport(
//     IN PHID_PARSER_CONTEXT ParserContext,
//     IN UCHAR ReportType,
//     IN UCHAR ReportID,
//     OUT PHID_REPORT *OutReport)
// {
//     //
//     // search in current top level collection
//     //
//     return HidParser_FindReportInCollection(ParserContext->RootCollection->Nodes[ParserContext->RootCollection->NodeCount-1], ReportType, ReportID, OutReport);
// }
//
// NTSTATUS
// HidParser_AllocateReport(
//     IN UCHAR ReportType,
//     IN UCHAR ReportID,
//     OUT PHID_REPORT *OutReport)
// {
//     PHID_REPORT Report;
//
//     //
//     // allocate report
//     //
//     Report = (PHID_REPORT)AllocFunction(sizeof(HID_REPORT));
//     if (!Report)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     //
//     // init report
//     //
//     Report->ReportID = ReportID;
//     Report->Type = ReportType;
//
//     //
//     // done
//     //
//     *OutReport = Report;
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_AddReportToCollection(
//     IN PHID_PARSER_CONTEXT ParserContext,
//     IN PHID_COLLECTION CurrentCollection,
//     IN PHID_REPORT NewReport)
// {
//     PHID_REPORT * NewReportArray;
//
//     //
//     // allocate new report array
//     //
//     NewReportArray = (PHID_REPORT*)AllocFunction(sizeof(PHID_REPORT) * (CurrentCollection->ReportCount + 1));
//     if (!NewReportArray)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     if (CurrentCollection->ReportCount)
//     {
//         //
//         // copy old array contents
//         //
//         CopyFunction(NewReportArray, CurrentCollection->Reports, sizeof(PHID_REPORT) * CurrentCollection->ReportCount);
//
//         //
//         // free old array
//         //
//         FreeFunction(CurrentCollection->Reports);
//     }
//
//     //
//     // store result
//     //
//     NewReportArray[CurrentCollection->ReportCount] = NewReport;
//     CurrentCollection->Reports = NewReportArray;
//     CurrentCollection->ReportCount++;
//
//     //
//     // completed successfully
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_GetReport(
//     IN PHID_PARSER_CONTEXT ParserContext,
//     IN PHID_COLLECTION Collection,
//     IN UCHAR ReportType,
//     IN UCHAR ReportID,
//     IN UCHAR CreateIfNotExists,
//     OUT PHID_REPORT *OutReport)
// {
//     NTSTATUS Status;
//
//     //
//     // try finding existing report
//     //
//     Status = HidParser_FindReport(ParserContext, ReportType, ReportID, OutReport);
//     if (Status == HIDP_STATUS_SUCCESS || CreateIfNotExists == FALSE)
//     {
//         //
//         // founed report
//         //
//         return Status;
//     }
//
//     //
//     // allocate new report
//     //
//     Status = HidParser_AllocateReport(ReportType, ReportID, OutReport);
//     if (Status != HIDP_STATUS_SUCCESS)
//     {
//         //
//         // failed to allocate report
//         //
//         return Status;
//     }
//
//     //
//     // add report
//     //
//     Status = HidParser_AddReportToCollection(ParserContext, Collection, *OutReport);
//     if (Status != HIDP_STATUS_SUCCESS)
//     {
//         //
//         // failed to allocate report
//         //
//         FreeFunction(*OutReport);
//     }
//
//     //
//     // done
//     //
//     return Status;
// }
//
// NTSTATUS
// HidParser_ReserveReportItems(
//     IN PHID_REPORT Report,
//     IN ULONG ReportCount,
//     OUT PHID_REPORT *OutReport)
// {
//     PHID_REPORT NewReport;
//     ULONG OldSize, Size;
//
//     if (Report->ItemCount + ReportCount <= Report->ItemAllocated)
//     {
//         //
//         // space is already allocated
//         //
//         *OutReport = Report;
//         return HIDP_STATUS_SUCCESS;
//     }
//
//     //
//     //calculate new size
//     //
//     OldSize = sizeof(HID_REPORT) + (Report->ItemCount) * sizeof(HID_REPORT_ITEM);
//     Size =  ReportCount * sizeof(HID_REPORT_ITEM);
//
//     //
//     // allocate memory
//     //
//     NewReport = (PHID_REPORT)AllocFunction(Size + OldSize);
//     if (!NewReport)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//
//     //
//     // copy old report
//     //
//     CopyFunction(NewReport, Report, OldSize);
//
//     //
//     // increase array size
//     //
//     NewReport->ItemAllocated += ReportCount;
//
//     //
//     // store result
//     //
//     *OutReport = NewReport;
//
//     //
//     // completed sucessfully
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// VOID
// HidParser_SignRange(
//     IN ULONG Minimum,
//     IN ULONG Maximum,
//     OUT PULONG NewMinimum,
//     OUT PULONG NewMaximum)
// {
//     ULONG Mask = 0x80000000;
//     ULONG Index;
//
//     for (Index = 0; Index < 4; Index++)
//     {
//         if (Minimum & Mask)
//         {
//             Minimum |= Mask;
//             if (Maximum & Mask)
//                 Maximum |= Mask;
//             return;
//         }
//
//         Mask >>= 8;
//         Mask |= 0xff000000;
//     }
//
//     *NewMinimum = Minimum;
//     *NewMaximum = Maximum;
// }
//
// NTSTATUS
// HidParser_InitReportItem(
//     IN PHID_REPORT Report,
//     IN PHID_REPORT_ITEM ReportItem,
//     IN PGLOBAL_ITEM_STATE GlobalItemState,
//     IN PLOCAL_ITEM_STATE LocalItemState,
//     IN PMAIN_ITEM_DATA ItemData,
//     IN ULONG ReportItemIndex)
// {
//     ULONG LogicalMinimum;
//     ULONG LogicalMaximum;
//     ULONG PhysicalMinimum;
//     ULONG PhysicalMaximum;
//     ULONG UsageMinimum;
//     ULONG UsageMaximum;
//     USAGE_VALUE UsageValue;
//
//     //
//     // get logical bounds
//     //
//     LogicalMinimum = GlobalItemState->LogicalMinimum;
//     LogicalMaximum = GlobalItemState->LogicialMaximum;
//     if (LogicalMinimum > LogicalMaximum)
//     {
//         //
//         // make them signed
//         //
//         HidParser_SignRange(LogicalMinimum, LogicalMaximum, &LogicalMinimum, &LogicalMaximum);
//     }
//     //ASSERT(LogicalMinimum <= LogicalMaximum);
//
//     //
//     // get physical bounds
//     //
//     PhysicalMinimum = GlobalItemState->PhysicalMinimum;
//     PhysicalMaximum = GlobalItemState->PhysicalMaximum;
//     if (PhysicalMinimum > PhysicalMaximum)
//     {
//         //
//         // make them signed
//         //
//         HidParser_SignRange(PhysicalMinimum, PhysicalMaximum, &PhysicalMinimum, &PhysicalMaximum);
//     }
//     //ASSERT(PhysicalMinimum <= PhysicalMaximum);
//
//     //
//     // get usage bounds
//     //
//     UsageMinimum = 0;
//     UsageMaximum = 0;
//     if (ItemData->ArrayVariable == FALSE)
//     {
//         //
//         // get usage bounds
//         //
//         UsageMinimum = LocalItemState->UsageMinimum.u.Extended;
//         UsageMaximum = LocalItemState->UsageMaximum.u.Extended;
//     }
//     else
//     {
//         //
//         // get usage value from stack
//         //
//         if (ReportItemIndex < LocalItemState->UsageStackUsed)
//         {
//             //
//             // use stack item
//             //
//             UsageValue = LocalItemState->UsageStack[ReportItemIndex];
//         }
//         else
//         {
//             //
//             // get usage minimum from local state
//             //
//             UsageValue = LocalItemState->UsageMinimum;
//
//             //
//             // append item index
//             //
//             UsageValue.u.Extended += ReportItemIndex;
//
//             if (LocalItemState->UsageMaximumSet)
//             {
//                 if (UsageValue.u.Extended > LocalItemState->UsageMaximum.u.Extended)
//                 {
//                     //
//                     // maximum reached
//                     //
//                     UsageValue.u.Extended = LocalItemState->UsageMaximum.u.Extended;
//                 }
//             }
//         }
//
//         //
//         // usage usage bounds
//         //
//         UsageMinimum = UsageMaximum = UsageValue.u.Extended;
//     }
//
//     //
//     // now store all values
//     //
//     ReportItem->ByteOffset = (Report->ReportSize / 8);
//     ReportItem->Shift = (Report->ReportSize % 8);
//     ReportItem->Mask = ~(0xFFFFFFFF << GlobalItemState->ReportSize);
//     ReportItem->BitCount = GlobalItemState->ReportSize;
//     ReportItem->HasData = (ItemData->DataConstant == FALSE);
//     ReportItem->Array = (ItemData->ArrayVariable == 0);
//     ReportItem->Relative = (ItemData->Relative != FALSE);
//     ReportItem->Minimum = LogicalMinimum;
//     ReportItem->Maximum = LogicalMaximum;
//     ReportItem->UsageMinimum = UsageMinimum;
//     ReportItem->UsageMaximum = UsageMaximum;
//
//     //
//     // increment report size
//     //
//     Report->ReportSize += GlobalItemState->ReportSize;
//
//     //
//     // completed successfully
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// BOOLEAN
// HidParser_UpdateCurrentCollectionReport(
//     IN PHID_COLLECTION Collection,
//     IN PHID_REPORT Report,
//     IN PHID_REPORT NewReport)
// {
//     ULONG Index;
//     BOOLEAN Found = FALSE, TempFound;
//
//     //
//     // search in local list
//     //
//     for(Index = 0; Index < Collection->ReportCount; Index++)
//     {
//         if (Collection->Reports[Index] == Report)
//         {
//             //
//             // update report
//             //
//             Collection->Reports[Index] = NewReport;
//             Found = TRUE;
//         }
//     }
//
//     //
//     // search in sub collections
//     //
//     for(Index = 0; Index < Collection->NodeCount; Index++)
//     {
//         //
//         // was it found
//         //
//         TempFound = HidParser_UpdateCurrentCollectionReport(Collection->Nodes[Index], Report, NewReport);
//         if (TempFound)
//         {
//             //
//             // the same report should not be found in different collections
//             //
//             ASSERT(Found == FALSE);
//             Found = TRUE;
//         }
//     }
//
//     //
//     // done
//     //
//     return Found;
// }
//
// BOOLEAN
// HidParser_UpdateCollectionReport(
//     IN PHID_PARSER_CONTEXT ParserContext,
//     IN PHID_REPORT Report,
//     IN PHID_REPORT NewReport)
// {
//     //
//     // update in current collection
//     //
//     return HidParser_UpdateCurrentCollectionReport(ParserContext->RootCollection->Nodes[ParserContext->RootCollection->NodeCount-1], Report, NewReport) as u8;
// }
//
//
// NTSTATUS
// HidParser_AddMainItem(
//     IN PHID_PARSER_CONTEXT ParserContext,
//     IN PHID_REPORT Report,
//     IN PGLOBAL_ITEM_STATE GlobalItemState,
//     IN PLOCAL_ITEM_STATE LocalItemState,
//     IN PMAIN_ITEM_DATA ItemData,
//     IN PHID_COLLECTION Collection)
// {
//     NTSTATUS Status;
//     ULONG Index;
//     PHID_REPORT NewReport;
//     BOOLEAN Found;
//
//     //
//     // first grow report item array
//     //
//     Status = HidParser_ReserveReportItems(Report, GlobalItemState->ReportCount, &NewReport);
//     if (Status != HIDP_STATUS_SUCCESS)
//     {
//         //
//         // failed to allocate memory
//         //
//         return Status;
//     }
//
//     if (NewReport != Report)
//     {
//         //
//         // update current top level collection
//         //
//         Found = HidParser_UpdateCollectionReport(ParserContext, Report, NewReport);
//         ASSERT(Found);
//     }
//
//     //
//     // sanity check
//     //
//     ASSERT(NewReport->ItemCount + GlobalItemState->ReportCount <= NewReport->ItemAllocated);
//
//     for(Index = 0; Index < GlobalItemState->ReportCount; Index++)
//     {
//         Status = HidParser_InitReportItem(NewReport, &NewReport->Items[NewReport->ItemCount], GlobalItemState, LocalItemState, ItemData, Index);
//         if (Status != HIDP_STATUS_SUCCESS)
//         {
//             //
//             // failed to init report item
//             //
//             return Status;
//         }
//
//         //
//         // increment report item count
//         //
//         NewReport->ItemCount++;
//     }
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_ParseReportDescriptor(
//     IN PUCHAR ReportDescriptor,
//     IN ULONG ReportLength,
//     OUT PVOID *OutParser)
// {
//     PGLOBAL_ITEM_STATE LinkedGlobalItemState, NextLinkedGlobalItemState;
//     ULONG Index;
//     PUSAGE_VALUE NewUsageStack, UsageValue;
//     NTSTATUS Status;
//     PHID_COLLECTION CurrentCollection, NewCollection;
//     PUCHAR CurrentOffset, ReportEnd;
//     PITEM_PREFIX CurrentItem;
//     ULONG CurrentItemSize;
//     PLONG_ITEM CurrentLongItem;
//     PSHORT_ITEM CurrentShortItem;
//     ULONG Data;
//     UCHAR ReportType;
//     PHID_REPORT Report;
//     PMAIN_ITEM_DATA MainItemData;
//     PHID_PARSER_CONTEXT ParserContext;
//
//     CurrentOffset = ReportDescriptor;
//     ReportEnd = ReportDescriptor + ReportLength;
//
//     if (ReportDescriptor >= ReportEnd)
//         return HIDP_STATUS_USAGE_NOT_FOUND;
//
//     //
//     // allocate parser
//     //
//     ParserContext = AllocFunction(sizeof(HID_PARSER_CONTEXT));
//     if (!ParserContext)
//         return HIDP_STATUS_INTERNAL_ERROR;
//
//
//     //
//     // allocate usage stack
//     //
//     ParserContext->LocalItemState.UsageStackAllocated = 10;
//     ParserContext->LocalItemState.UsageStack = (PUSAGE_VALUE)AllocFunction(ParserContext->LocalItemState.UsageStackAllocated * sizeof(USAGE_VALUE));
//     if (!ParserContext->LocalItemState.UsageStack)
//     {
//         //
//         // no memory
//         //
//         FreeFunction(ParserContext);
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     //
//     // now allocate root collection
//     //
//     Status = HidParser_AllocateCollection(NULL, COLLECTION_LOGICAL, &ParserContext->LocalItemState, &ParserContext->RootCollection);
//     if (Status != HIDP_STATUS_SUCCESS)
//     {
//         //
//         // no memory
//         //
//         FreeFunction(ParserContext->LocalItemState.UsageStack);
//         ParserContext->LocalItemState.UsageStack = NULL;
//         FreeFunction(ParserContext);
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     //
//     // start parsing
//     //
//     CurrentCollection = ParserContext->RootCollection;
//
//     do
//     {
//         //
//         // get current item
//         //
//         CurrentItem = (PITEM_PREFIX)CurrentOffset;
//
//         //
//         // get item size
//         //
//         CurrentItemSize = ItemSize[CurrentItem->Size];
//         Data = 0;
//
//         if (CurrentItem->Type == ITEM_TYPE_LONG)
//         {
//             //
//             // increment item size with size of data item
//             //
//             CurrentLongItem = (PLONG_ITEM)CurrentItem;
//             CurrentItemSize += CurrentLongItem->DataSize;
//         }
//         else
//         {
//             //
//             // get short item
//             //
//             CurrentShortItem = (PSHORT_ITEM)CurrentItem;
//
//             //
//             // get associated data
//             //
//             //ASSERT(CurrentItemSize == 1 || CurrentItemSize == 2 || CurrentItemSize == 4);
//             if (CurrentItemSize == 1)
//                 Data = CurrentShortItem->Data.UData8[0];
//             else if (CurrentItemSize == 2)
//                 Data = CurrentShortItem->Data.UData16[0];
//             else if (CurrentItemSize == 4)
//                 Data = CurrentShortItem->Data.UData32;
//             else
//             {
//                 //
//                 // invalid item size
//                 //
//                 //DebugFunction("CurrentItem invalid item size %lu\n", CurrentItemSize);
//             }
//
//         }
//         DebugFunction("Tag %x Type %x Size %x Offset %lu Length %lu\n", CurrentItem->Tag, CurrentItem->Type, CurrentItem->Size,  ((ULONG_PTR)CurrentItem - (ULONG_PTR)ReportDescriptor), ReportLength);
//         //
//         // handle items
//         //
//         ASSERT(CurrentItem->Type >= ITEM_TYPE_MAIN && CurrentItem->Type <= ITEM_TYPE_LONG);
//         switch(CurrentItem->Type)
//         {
//             case ITEM_TYPE_MAIN:
//             {
//                 // preprocess the local state if relevant (usages for
//                 // collections and report items)
//                 if (CurrentItem->Tag != ITEM_TAG_MAIN_END_COLLECTION)
//                 {
//                     // make all usages extended for easier later processing
//                     for (Index = 0; Index < ParserContext->LocalItemState.UsageStackUsed; Index++)
//                     {
//                         //
//                         // is it already extended
//                         //
//                         if (ParserContext->LocalItemState.UsageStack[Index].IsExtended)
//                             continue;
//
//                         //
//                         // extend usage item
//                         //
//                         ParserContext->LocalItemState.UsageStack[Index].u.s.UsagePage = ParserContext->GlobalItemState.UsagePage;
//                         ParserContext->LocalItemState.UsageStack[Index].IsExtended = TRUE;
//                     }
//
//                     if (!ParserContext->LocalItemState.UsageMinimum.IsExtended) {
//                         // the specs say if one of them is extended they must
//                         // both be extended, so if the minimum isn't, the
//                         // maximum mustn't either.
//                         ParserContext->LocalItemState.UsageMinimum.u.s.UsagePage
//                             = ParserContext->LocalItemState.UsageMaximum.u.s.UsagePage
//                                 = ParserContext->GlobalItemState.UsagePage;
//                         ParserContext->LocalItemState.UsageMinimum.IsExtended
//                             = ParserContext->LocalItemState.UsageMaximum.IsExtended = TRUE;
//                     }
//
//                     //LocalItemState.usage_stack = usageStack;
//                     //ParserContext->LocalItemState.UsageStackUsed = UsageStackUsed;
//                 }
//
//                 if (CurrentItem->Tag == ITEM_TAG_MAIN_COLLECTION) {
//
//                     //
//                     // allocate new collection
//                     //
//                     Status = HidParser_AllocateCollection(CurrentCollection, (UCHAR)Data, &ParserContext->LocalItemState, &NewCollection);
//                     ASSERT(Status == HIDP_STATUS_SUCCESS);
//
//                     //
//                     // add new collection to current collection
//                     //
//                     Status = HidParser_AddCollection(CurrentCollection, NewCollection);
//                     ASSERT(Status == HIDP_STATUS_SUCCESS);
//
//                     //
//                     // make new collection current
//                     //
//                     CurrentCollection = NewCollection;
//                 }
//                 else if (CurrentItem->Tag == ITEM_TAG_MAIN_END_COLLECTION)
//                 {
//                     //
//                     // assert on ending the root collection
//                     //
//                     ASSERT(CurrentCollection != ParserContext->RootCollection);
//
//                     //
//                     // use parent of current collection
//                     //
//                     CurrentCollection = CurrentCollection->Root;
//                     ASSERT(CurrentCollection);
//                 }
//                 else
//                 {
//                     ReportType = HID_REPORT_TYPE_ANY;
//
//                     switch (CurrentItem->Tag) {
//                         case ITEM_TAG_MAIN_INPUT:
//                             ReportType = HID_REPORT_TYPE_INPUT;
//                             break;
//
//                         case ITEM_TAG_MAIN_OUTPUT:
//                             ReportType = HID_REPORT_TYPE_OUTPUT;
//                             break;
//
//                         case ITEM_TAG_MAIN_FEATURE:
//                             ReportType = HID_REPORT_TYPE_FEATURE;
//                             break;
//
//                         default:
//                             DebugFunction("[HIDPARSE] Unknown ReportType Tag %x Type %x Size %x CurrentItemSize %x\n", CurrentItem->Tag, CurrentItem->Type, CurrentItem->Size, CurrentItemSize);
//                             ASSERT(FALSE);
//                             break;
//                     }
//
//                     if (ReportType == HID_REPORT_TYPE_ANY)
//                         break;
//
//                     //
//                     // get report
//                     //
//                     Status = HidParser_GetReport(ParserContext, CurrentCollection, ReportType, ParserContext->GlobalItemState.ReportId, TRUE, &Report);
//                     ASSERT(Status == HIDP_STATUS_SUCCESS);
//
//                     // fill in a sensible default if the index isn't set
//                     if (!ParserContext->LocalItemState.DesignatorIndexSet) {
//                         ParserContext->LocalItemState.DesignatorIndex
//                             = ParserContext->LocalItemState.DesignatorMinimum;
//                     }
//
//                     if (!ParserContext->LocalItemState.StringIndexSet)
//                         ParserContext->LocalItemState.StringIndex = ParserContext->LocalItemState.StringMinimum;
//
//                     //
//                     // get main item data
//                     //
//                     MainItemData = (PMAIN_ITEM_DATA)&Data;
//
//                     //
//                     // add states & data to the report
//                     //
//                     Status = HidParser_AddMainItem(ParserContext, Report, &ParserContext->GlobalItemState, &ParserContext->LocalItemState, MainItemData, CurrentCollection);
//                     ASSERT(Status == HIDP_STATUS_SUCCESS);
//                 }
//
//                 //
//                 // backup stack
//                 //
//                 Index = ParserContext->LocalItemState.UsageStackAllocated;
//                 NewUsageStack = ParserContext->LocalItemState.UsageStack;
//
//                 //
//                 // reset the local item state and clear the usage stack
//                 //
//                 ZeroFunction(&ParserContext->LocalItemState, sizeof(LOCAL_ITEM_STATE));
//
//                 //
//                 // restore stack
//                 //
//                 ParserContext->LocalItemState.UsageStack = NewUsageStack;
//                 ParserContext->LocalItemState.UsageStackAllocated = Index;
//                 break;
//             }
//             case ITEM_TYPE_GLOBAL:
//             {
//                 switch (CurrentItem->Tag) {
//                     case ITEM_TAG_GLOBAL_USAGE_PAGE:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_USAGE_PAGE %x\n", Data);
//                         ParserContext->GlobalItemState.UsagePage = Data;
//                         break;
//                     case ITEM_TAG_GLOBAL_LOGICAL_MINIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_LOGICAL_MINIMUM %x\n", Data);
//                         ParserContext->GlobalItemState.LogicalMinimum = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_LOGICAL_MAXIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_LOCAL_MAXIMUM %x\n", Data);
//                         ParserContext->GlobalItemState.LogicialMaximum = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_PHYSICAL_MINIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_PHYSICAL_MINIMUM %x\n", Data);
//                         ParserContext->GlobalItemState.PhysicalMinimum = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_PHYSICAL_MAXIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_PHYSICAL_MAXIMUM %x\n", Data);
//                         ParserContext->GlobalItemState.PhysicalMaximum = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_UNIT_EXPONENT:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_UNIT_EXPONENT %x\n", Data);
//                         ParserContext->GlobalItemState.UnitExponent = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_UNIT:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_UNIT %x\n", Data);
//                         ParserContext->GlobalItemState.Unit = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_REPORT_SIZE:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_SIZE %x\n", Data);
//                         ParserContext->GlobalItemState.ReportSize = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_REPORT_ID:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_ID %x\n", Data);
//                         ParserContext->GlobalItemState.ReportId = Data;
//                         ParserContext->UseReportIDs = TRUE;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_REPORT_COUNT:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_REPORT_COUNT %x\n", Data);
//                         ParserContext->GlobalItemState.ReportCount = Data;
//                         break;
//
//                     case ITEM_TAG_GLOBAL_PUSH:
//                     {
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_PUSH\n");
//                         //
//                         // allocate global item state
//                         //
//                         LinkedGlobalItemState = (PGLOBAL_ITEM_STATE)AllocFunction(sizeof(GLOBAL_ITEM_STATE));
//                         ASSERT(LinkedGlobalItemState);
//
//                         //
//                         // copy global item state
//                         //
//                         CopyFunction(LinkedGlobalItemState, &ParserContext->GlobalItemState, sizeof(GLOBAL_ITEM_STATE));
//
//                         //
//                         // store pushed item in link member
//                         //
//                         ParserContext->GlobalItemState.Next = (struct __GLOBAL_ITEM_STATE__*)LinkedGlobalItemState;
//                         break;
//                     }
//                     case ITEM_TAG_GLOBAL_POP:
//                     {
//                         DebugFunction("[HIDPARSE] ITEM_TAG_GLOBAL_POP\n");
//                         if (ParserContext->GlobalItemState.Next == NULL)
//                         {
//                             //
//                             // pop without push
//                             //
//                             ASSERT(FALSE);
//                             break;
//                         }
//
//                         //
//                         // get link
//                         //
//                         LinkedGlobalItemState = (PGLOBAL_ITEM_STATE)ParserContext->GlobalItemState.Next;
//
//                         //
//                         // replace current item with linked one
//                         //
//                         CopyFunction(&ParserContext->GlobalItemState, LinkedGlobalItemState, sizeof(GLOBAL_ITEM_STATE));
//
//                         //
//                         // free item
//                         //
//                         FreeFunction(LinkedGlobalItemState);
//                         break;
//                     }
//
//                     default:
//                         //
//                         // unknown  / unsupported tag
//                         //
//                         ASSERT(FALSE);
//                         break;
//                 }
//
//                 break;
//             }
//             case ITEM_TYPE_LOCAL:
//             {
//                 switch (CurrentItem->Tag)
//                 {
//                     case ITEM_TAG_LOCAL_USAGE:
//                     {
//                         if (ParserContext->LocalItemState.UsageStackUsed >= ParserContext->LocalItemState.UsageStackAllocated)
//                         {
//                             //
//                             // increment stack size
//                             //
//                             ParserContext->LocalItemState.UsageStackAllocated += 10;
//
//                             //
//                             // build new usage stack
//                             //
//                             NewUsageStack = (PUSAGE_VALUE)AllocFunction(sizeof(USAGE_VALUE) * ParserContext->LocalItemState.UsageStackAllocated);
//                             ASSERT(NewUsageStack);
//
//                             //
//                             // copy old usage stack
//                             //
//                             CopyFunction(NewUsageStack, ParserContext->LocalItemState.UsageStack, sizeof(USAGE_VALUE) * (ParserContext->LocalItemState.UsageStackAllocated - 10));
//
//                             //
//                             // free old usage stack
//                             //
//                             FreeFunction(ParserContext->LocalItemState.UsageStack);
//
//                             //
//                             // replace with new usage stack
//                             //
//                             ParserContext->LocalItemState.UsageStack = NewUsageStack;
//                         }
//
//                         //
//                         // get fresh usage value
//                         //
//                         UsageValue = &ParserContext->LocalItemState.UsageStack[ParserContext->LocalItemState.UsageStackUsed];
//
//                         //
//                         // init usage stack
//                         //
//                         UsageValue->IsExtended = CurrentItemSize == sizeof(ULONG);
//                         UsageValue->u.Extended = Data;
//
//                         //
//                         // increment usage stack usage count
//                         //
//                         ParserContext->LocalItemState.UsageStackUsed++;
//                         break;
//                     }
//
//                     case ITEM_TAG_LOCAL_USAGE_MINIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_USAGE_MINIMUM Data %x\n", Data);
//                         ParserContext->LocalItemState.UsageMinimum.u.Extended = Data;
//                         ParserContext->LocalItemState.UsageMinimum.IsExtended
//                             = CurrentItemSize == sizeof(ULONG);
//                         ParserContext->LocalItemState.UsageMinimumSet = TRUE;
//                         break;
//
//                     case ITEM_TAG_LOCAL_USAGE_MAXIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_USAGE_MAXIMUM Data %x ItemSize %x %x\n", Data, CurrentItemSize, CurrentItem->Size);
//                         ParserContext->LocalItemState.UsageMaximum.u.Extended = Data;
//                         ParserContext->LocalItemState.UsageMaximum.IsExtended
//                             = CurrentItemSize == sizeof(ULONG);
//                         ParserContext->LocalItemState.UsageMaximumSet = TRUE;
//                         break;
//
//                     case ITEM_TAG_LOCAL_DESIGNATOR_INDEX:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_INDEX Data %x\n", Data);
//                         ParserContext->LocalItemState.DesignatorIndex = Data;
//                         ParserContext->LocalItemState.DesignatorIndexSet = TRUE;
//                         break;
//
//                     case ITEM_TAG_LOCAL_DESIGNATOR_MINIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_MINIMUM Data %x\n", Data);
//                         ParserContext->LocalItemState.DesignatorMinimum = Data;
//                         break;
//
//                     case ITEM_TAG_LOCAL_DESIGNATOR_MAXIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_DESIGNATOR_MAXIMUM Data %x\n", Data);
//                         ParserContext->LocalItemState.DesignatorMaximum = Data;
//                         break;
//
//                     case ITEM_TAG_LOCAL_STRING_INDEX:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_STRING_INDEX Data %x\n", Data);
//                         ParserContext->LocalItemState.StringIndex = Data;
//                         ParserContext->LocalItemState.StringIndexSet = TRUE;
//                         break;
//
//                     case ITEM_TAG_LOCAL_STRING_MINIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_STRING_MINIMUM Data %x\n", Data);
//                         ParserContext->LocalItemState.StringMinimum = Data;
//                         break;
//
//                     case ITEM_TAG_LOCAL_STRING_MAXIMUM:
//                         DebugFunction("[HIDPARSE] ITEM_TAG_LOCAL_STRING_MAXIMUM Data %x\n", Data);
//                         ParserContext->LocalItemState.StringMaximum = Data;
//                         break;
//
//                     default:
//                         DebugFunction("Unknown Local Item Tag %x\n", CurrentItem->Tag);
//                         ASSERT(FALSE);
//                         break;
//                 }
//                 break;
//             }
//
//             case ITEM_TYPE_LONG:
//             {
//                 CurrentLongItem = (PLONG_ITEM)CurrentItem;
//                 DebugFunction("Unsupported ITEM_TYPE_LONG Tag %x\n", CurrentLongItem->LongItemTag);
//                 break;
//             }
//         }
//
//         //
//         // move to next item
//         //
//         CurrentOffset += CurrentItemSize + sizeof(ITEM_PREFIX);
//
//     }while (CurrentOffset < ReportEnd);
//
//
//     //
//     // cleanup global stack
//     //
//     LinkedGlobalItemState = (PGLOBAL_ITEM_STATE)ParserContext->GlobalItemState.Next;
//     while(LinkedGlobalItemState != NULL)
//     {
//         DebugFunction("[HIDPARSE] Freeing GlobalState %p\n", LinkedGlobalItemState);
//         //
//         // free global item state
//         //
//         NextLinkedGlobalItemState = (PGLOBAL_ITEM_STATE)LinkedGlobalItemState->Next;
//
//         //
//         // free state
//         //
//         FreeFunction(LinkedGlobalItemState);
//
//         //
//         // move to next global state
//         //
//         LinkedGlobalItemState = NextLinkedGlobalItemState;
//     }
//
//     //
//     // free usage stack
//     //
//     FreeFunction(ParserContext->LocalItemState.UsageStack);
//     ParserContext->LocalItemState.UsageStack = NULL;
//
//     //
//     // store result
//     //
//     *OutParser = ParserContext;
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// PHID_COLLECTION
// HidParser_GetCollection(
//     PHID_PARSER_CONTEXT ParserContext,
//     IN ULONG CollectionNumber)
// {
//     //
//     // sanity checks
//     //
//     ASSERT(ParserContext);
//     ASSERT(ParserContext->RootCollection);
//     ASSERT(ParserContext->RootCollection->NodeCount);
//
//     //
//     // is collection index out of bounds
//     //
//     if (CollectionNumber < ParserContext->RootCollection->NodeCount)
//     {
//         //
//         // valid collection
//         //
//         return ParserContext->RootCollection->Nodes[CollectionNumber];
//     }
//
//     //
//     // no such collection
//     //
//     DebugFunction("HIDPARSE] No such collection %lu\n", CollectionNumber);
//     return NULL;
// }
//
//
// ULONG
// HidParser_NumberOfTopCollections(
//     IN PVOID ParserCtx)
// {
//     PHID_PARSER_CONTEXT ParserContext;
//
//     //
//     // get parser context
//     //
//     ParserContext = (PHID_PARSER_CONTEXT)ParserCtx;
//
//     //
//     // sanity checks
//     //
//     ASSERT(ParserContext);
//     ASSERT(ParserContext->RootCollection);
//     ASSERT(ParserContext->RootCollection->NodeCount);
//
//     //
//     // number of top collections
//     //
//     return ParserContext->RootCollection->NodeCount;
// }
//
// NTSTATUS
// HidParser_BuildContext(
//     IN PVOID ParserContext,
//     IN ULONG CollectionIndex,
//     IN ULONG ContextSize,
//     OUT PVOID *CollectionContext)
// {
//     PHID_COLLECTION Collection;
//     PVOID Context;
//     NTSTATUS Status;
//
//     //
//     // lets get the collection
//     //
//     Collection = HidParser_GetCollection((PHID_PARSER_CONTEXT)ParserContext, CollectionIndex);
//     ASSERT(Collection);
//
//     //
//     // lets allocate the context
//     //
//     Context = AllocFunction(ContextSize);
//     if (Context == NULL)
//     {
//         //
//         // no memory
//         //
//         return HIDP_STATUS_INTERNAL_ERROR;
//     }
//
//     //
//     // lets build the context
//     //
//     Status = HidParser_BuildCollectionContext(Collection, Context, ContextSize);
//     if (Status == HIDP_STATUS_SUCCESS)
//     {
//         //
//         // store context
//         //
//         *CollectionContext = Context;
//     }
//
//     //
//     // done
//     //
//     return Status;
// }
//
//
// ULONG
// HidParser_GetContextSize(
//     IN PVOID ParserContext,
//     IN ULONG CollectionIndex)
// {
//     PHID_COLLECTION Collection;
//     ULONG Size;
//
//     //
//     // lets get the collection
//     //
//     Collection = HidParser_GetCollection((PHID_PARSER_CONTEXT)ParserContext, CollectionIndex);
//
//     //
//     // calculate size
//     //
//     Size = HidParser_CalculateContextSize(Collection);
//     return Size;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
