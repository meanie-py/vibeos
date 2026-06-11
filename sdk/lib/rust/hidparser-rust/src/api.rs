use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;

use core::ffi::c_void;

use crate::parser_h::{
    HID_COLLECTION, PHID_COLLECTION,
    HID_REPORT, PHID_REPORT, _HID_REPORT,
    HID_REPORT_ITEM, PHID_REPORT_ITEM,
    USAGE_VALUE, usage_value,
};
use hid_sys::{
    HIDP_VALUE_CAPS, PHIDP_VALUE_CAPS,
    HIDP_CAPS,
    HIDP_KEYBOARD_MODIFIER_STATE, PHIDP_KEYBOARD_MODIFIER_STATE,
    HIDP_KEYBOARD_DIRECTION,
    PHIDP_INSERT_SCANCODES,
    HIDP_DEVICE_DESC,
    USAGE,
    HidP_Input, HidP_Output, HidP_Feature,
    HIDP_STATUS_SUCCESS, HIDP_STATUS_REPORT_DOES_NOT_EXIST,
    HIDP_STATUS_INVALID_REPORT_LENGTH, HIDP_STATUS_BUFFER_TOO_SMALL,
    HIDP_STATUS_USAGE_NOT_FOUND, HIDP_STATUS_BAD_LOG_PHY_VALUES,
    HIDP_STATUS_I8042_TRANS_UNKNOWN,
};

// Local consts/types — not in substrate
const HID_USAGE_PAGE_UNDEFINED: u16 = 0;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct USAGE_AND_PAGE {
    pub Usage: USAGE,
    pub UsagePage: USAGE,
}
type PUSAGE_AND_PAGE = *mut USAGE_AND_PAGE;

extern "C" {
    fn HidParser_GetReportInCollection( CollectionContext: *mut core::ffi::c_void, ReportType: u8, ) -> PHID_REPORT;
}
extern "C" {
    fn HidParser_GetCollectionFromContext(CollectionContext: PVOID) -> PHID_COLLECTION;
    fn DbgPrint(format: *const i8, ...) -> u32;
}
// NTAPI (stdcall on i386) externals: RtlAssert (kernel export) + the hidparse.sys
// util callbacks (defined extern "system" in hidparse-rust).
extern "system" {
    fn RtlAssert(pCondition: *const core::ffi::c_void, pFileName: *const core::ffi::c_void, LineNumber: u32, pMessage: *mut core::ffi::c_void);
    fn ZeroFunction(p: PVOID, size: ULONG);
    fn CopyFunction(dest: PVOID, src: PVOID, size: ULONG);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct CustomerScanCode {
    pub Usage: USAGE,
    pub ScanCode: ULONG,
}

pub static KeyboardScanCodes: [ULONG; 256] = [
/* 0 */ 0x0000, 0x0000, 0x0000, 0x0000, 0x001e, 0x0030, 0x002e, 0x0020, 0x0012, 0x0021, 0x0022, 0x0023, 0x0017, 0x0024, 0x0025, 0x0026,
/* 1 */ 0x0032, 0x0031, 0x0018, 0x0019, 0x0010, 0x0013, 0x001f, 0x0014, 0x0016, 0x002f, 0x0011, 0x002d, 0x0015, 0x002c, 0x0002, 0x0003,
/* 2 */ 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000a, 0x000b, 0x001c, 0x0001, 0x000e, 0x000f, 0x0039, 0x000c, 0x000d, 0x001a,
/* 3 */ 0x001b, 0x002b, 0x002b, 0x0027, 0x0028, 0x0029, 0x0033, 0x0034, 0x0035, 0x003a, 0x003b, 0x003c, 0x003d, 0x003e, 0x003f, 0x0040,
/* 4 */ 0x0041, 0x0042, 0x0043, 0x0044, 0x0057, 0x0058, 0xE037, 0x0046, 0x0045, 0xE052, 0xE047, 0xE049, 0xE053, 0xE04F, 0xE051, 0xE04D,
/* 5 */ 0xE04B, 0xE050, 0xE048, 0x0045, 0xE035, 0x0037, 0x004a, 0x004e, 0xE01C, 0x004f, 0x0050, 0x0051, 0x004b, 0x004c, 0x004d, 0x0047,
/* 6 */ 0x0048, 0x0049, 0x0052, 0x0053, 0x0056, 0xE05D, 0xE05E, 0x0075, 0x00b7, 0x00b8, 0x00b9, 0x00ba, 0x00bb, 0x00bc, 0x00bd, 0x00be,
/* 7 */ 0x00bf, 0x00c0, 0x00c1, 0x00c2, 0x0086, 0x008a, 0x0082, 0x0084, 0x0080, 0x0081, 0x0083, 0x0089, 0x0085, 0x0087, 0x0088, 0x0071,
/* 8 */ 0x0073, 0x0072, 0x0000, 0x0000, 0x0000, 0x0079, 0x0000, 0x0059, 0x005d, 0x007c, 0x005c, 0x005e, 0x005f, 0x0000, 0x0000, 0x0000,
/* 9 */ 0x007a, 0x007b, 0x005a, 0x005b, 0x0055, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
/* A */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
/* B */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
/* C */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
/* D */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
/* E */ 0x001D, 0x002A, 0x0038, 0xE05B, 0xE01D, 0x0036, 0xE038, 0xE05C, 0x00a4, 0x00a6, 0x00a5, 0x00a3, 0x00a1, 0x0073, 0x0072, 0x0071,
/* F */ 0x0096, 0x009e, 0x009f, 0x0080, 0x0088, 0x00b1, 0x00b2, 0x00b0, 0x008e, 0x0098, 0x00ad, 0x008c, 0x0000, 0x0000, 0x0000, 0x0000,
];

pub static CustomerScanCodes: [CustomerScanCode; 18] = [
    CustomerScanCode { Usage: 0x00B5, ScanCode: 0xE019 },
    CustomerScanCode { Usage: 0x00B6, ScanCode: 0xE010 },
    CustomerScanCode { Usage: 0x00B7, ScanCode: 0xE024 },
    CustomerScanCode { Usage: 0x00CD, ScanCode: 0xE022 },
    CustomerScanCode { Usage: 0x00E2, ScanCode: 0xE020 },
    CustomerScanCode { Usage: 0x00E9, ScanCode: 0xE030 },
    CustomerScanCode { Usage: 0x00EA, ScanCode: 0xE02E },
    CustomerScanCode { Usage: 0x0183, ScanCode: 0xE06D },
    CustomerScanCode { Usage: 0x018A, ScanCode: 0xE06C },
    CustomerScanCode { Usage: 0x0192, ScanCode: 0xE021 },
    CustomerScanCode { Usage: 0x0194, ScanCode: 0xE06B },
    CustomerScanCode { Usage: 0x0221, ScanCode: 0xE065 },
    CustomerScanCode { Usage: 0x0223, ScanCode: 0xE032 },
    CustomerScanCode { Usage: 0x0224, ScanCode: 0xE06A },
    CustomerScanCode { Usage: 0x0225, ScanCode: 0xE069 },
    CustomerScanCode { Usage: 0x0226, ScanCode: 0xE068 },
    CustomerScanCode { Usage: 0x0227, ScanCode: 0xE067 },
    CustomerScanCode { Usage: 0x022A, ScanCode: 0xE066 },
];


#[no_mangle]
pub unsafe extern "C" fn HidParser_GetCollectionUsagePage(
    CollectionContext: PVOID,
    Usage: PUSHORT,
    UsagePage: PUSHORT,
) -> NTSTATUS {
    let mut Collection: PHID_COLLECTION = core::ptr::null_mut();
    Collection = HidParser_GetCollectionFromContext(CollectionContext);
    if Collection.is_null() {
        return HIDP_STATUS_USAGE_NOT_FOUND;
    }
    core::ptr::write_volatile(UsagePage, ((*Collection).Usage >> 16) as u16);
    core::ptr::write_volatile(Usage, ((*Collection).Usage & 0xFFFF) as u16);
    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetReportLength(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
) -> u32 {
    let mut Report: *mut HID_REPORT = core::ptr::null_mut::<HID_REPORT>();
    let mut ReportLength: u32 = 0u32;
    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return 0u32;
    }
    ReportLength = (*Report).ReportSize;
    if ReportLength != 0u32 {
        if (ReportLength as u32).wrapping_rem(8u32) != 0u32 {
            RtlAssert(
                b"ReportLength % 8 == 0\0".as_ptr() as *const core::ffi::c_void,
                b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0".as_ptr() as *const core::ffi::c_void,
                125u32,
                core::ptr::null_mut::<core::ffi::c_void>(),
            );
        }
        return ReportLength.wrapping_div(8u32);
    }
    return ReportLength;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetReportItemCountFromReportType(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
) -> u32 {
    let mut Report: *mut _HID_REPORT = core::ptr::null_mut();
    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return 0u32;
    }
    return (*Report).ItemCount;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetReportItemTypeCountFromReportType(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
    bData: u32,
) -> u32 {
    let mut Index: u32 = 0;
    let mut Report: *mut HID_REPORT = core::ptr::null_mut::<HID_REPORT>();
    let mut ItemCount: u32 = 0;

    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return 0;
    }

    Index = 0;
    while Index < (*Report).ItemCount {
        let item_ptr = core::ptr::addr_of_mut!((*Report).Items) as *mut HID_REPORT_ITEM;
        let current_item = item_ptr.offset(Index as isize);

        if (*current_item).HasData != 0 && bData != 0 {
            ItemCount = ItemCount.wrapping_add(1);
        } else if (*current_item).HasData == 0 && bData == 0 {
            ItemCount = ItemCount.wrapping_add(1);
        }

        Index = Index.wrapping_add(1);
    }

    ItemCount
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetMaxUsageListLengthWithReportAndPage(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
    UsagePage: u16,
) -> u32 {
    let mut Index: u32;
    let mut Report: *mut HID_REPORT = core::ptr::null_mut();
    let mut ItemCount: u32 = 0u32;
    let mut CurrentUsagePage: u16;
    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return 0u32;
    }
    Index = 0u32;
    while Index < (*Report).ItemCount {
        CurrentUsagePage = ((*(&raw mut (*Report).Items as *mut HID_REPORT_ITEM).offset(Index as isize)).UsageMinimum >> 16i32) as u16;
        if CurrentUsagePage as i32 == UsagePage as i32
            && (*(&raw mut (*Report).Items as *mut HID_REPORT_ITEM).offset(Index as isize)).HasData as i32 != 0
        {
            ItemCount = ItemCount.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }
    return ItemCount;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetSpecificValueCapsWithReport(
    CollectionContext: PVOID,
    ReportType: UCHAR,
    UsagePage: USHORT,
    Usage: USHORT,
    ValueCaps: PHIDP_VALUE_CAPS,
    ValueCapsLength: *mut USHORT,
) -> NTSTATUS {
    let mut Index: ULONG = 0;
    let mut Report: PHID_REPORT = core::ptr::null_mut();
    let mut ItemCount: USHORT = 0;
    let mut CurrentUsagePage: USHORT = 0;
    let mut CurrentUsage: USHORT = 0;

    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
    }

    Index = 0;
    let items_ptr = &raw mut (*Report).Items as *mut HID_REPORT_ITEM;
    while Index < (*Report).ItemCount {
        let item_ptr = items_ptr.add(Index as usize);
        CurrentUsagePage = ((*item_ptr).UsageMinimum >> 16) as USHORT;
        CurrentUsage = ((*item_ptr).UsageMinimum & 0xFFFF) as USHORT;

        if (Usage as i32 == CurrentUsage as i32 && UsagePage as i32 == CurrentUsagePage as i32)
            || (Usage as i32 == 0 && UsagePage as i32 == CurrentUsagePage as i32)
            || (Usage as i32 == CurrentUsage as i32 && UsagePage as i32 == 0)
            || (Usage as i32 == 0 && UsagePage as i32 == 0)
        {
            if (ItemCount as i32) < (*ValueCapsLength as i32) {
                ZeroFunction(
                    ValueCaps.add(ItemCount as usize) as PVOID,
                    core::mem::size_of::<HIDP_VALUE_CAPS>() as ULONG,
                );

                let caps_ptr = ValueCaps.add(ItemCount as usize);
                (*caps_ptr).UsagePage = CurrentUsagePage as USAGE;
                (*caps_ptr).ReportID = (*Report).ReportID;
                (*caps_ptr).LogicalMin = (*item_ptr).Minimum as LONG;
                (*caps_ptr).LogicalMax = (*item_ptr).Maximum as LONG;
                (*caps_ptr).IsAbsolute = if (*item_ptr).Relative == 0 { 1 } else { 0 };
                (*caps_ptr).BitSize = (*item_ptr).BitCount as USHORT;
            }
            ItemCount = ItemCount.wrapping_add(1);
        }
        Index = Index.wrapping_add(1);
    }

    core::ptr::write_volatile(ValueCapsLength, ItemCount as USHORT);

    if ItemCount != 0 {
        return HIDP_STATUS_SUCCESS;
    }
    return HIDP_STATUS_USAGE_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetUsagesWithReport(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
    UsagePage: u16,
    UsageList: *mut u16,
    UsageLength: *mut u32,
    ReportDescriptor: *mut i8,
    ReportDescriptorLength: u32,
) -> i32 {
    let mut Index: u32;
    let Report: *mut HID_REPORT;
    let mut ItemCount: u32 = 0;
    let mut CurrentUsagePage: u16;
    let mut ReportItem: *mut HID_REPORT_ITEM;
    let mut Activated: u8;
    let mut Data: u32;
    let mut UsageAndPage: *mut USAGE_AND_PAGE = core::ptr::null_mut();

    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
    }

    if ((*Report).ReportSize as u32).wrapping_div(8) != (ReportDescriptorLength).wrapping_sub(1) {
        return HIDP_STATUS_INVALID_REPORT_LENGTH;
    }

    if UsagePage == HID_USAGE_PAGE_UNDEFINED {
        UsageAndPage = UsageList as *mut USAGE_AND_PAGE;
    }

    Index = 0;
    while Index < (*Report).ItemCount {
        ReportItem = (*Report).Items.as_mut_ptr().offset(Index as isize) as *mut HID_REPORT_ITEM;

        if (*ReportItem).HasData == 0 {
            Index = Index.wrapping_add(1);
            continue;
        }

        CurrentUsagePage = ((*ReportItem).UsageMinimum >> 16) as u16;

        if UsagePage != HID_USAGE_PAGE_UNDEFINED && UsagePage != CurrentUsagePage {
            Index = Index.wrapping_add(1);
            continue;
        }

        // assertion checks from anchor
        if !((*ReportItem).ByteOffset < ReportDescriptorLength) {
            RtlAssert(
                b"ReportItem->ByteOffset < ReportDescriptorLength\0".as_ptr() as *mut core::ffi::c_void,
                b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0".as_ptr() as *mut core::ffi::c_void,
                422,
                core::ptr::null_mut(),
            );
        }
        if !((*ReportItem).BitCount as i32 <= 8) {
            RtlAssert(
                b"ReportItem->BitCount <= 8\0".as_ptr() as *mut core::ffi::c_void,
                b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0".as_ptr() as *mut core::ffi::c_void,
                423,
                core::ptr::null_mut(),
            );
        }

        Data = *ReportDescriptor.offset(
            ((*ReportItem).ByteOffset as u32).wrapping_add(1) as isize,
        ) as u32;

        Data >>= (*ReportItem).Shift as u32;
        Data &= (*ReportItem).Mask as u32;

        Activated = if Data != 0 { 1 } else { 0 };

        if Activated == 0 {
            Index = Index.wrapping_add(1);
            continue;
        }

        if ItemCount >= *UsageLength {
            ItemCount = ItemCount.wrapping_add(1);
        } else {
            if UsagePage != HID_USAGE_PAGE_UNDEFINED {
                *UsageList.offset(ItemCount as isize) =
                    ((*ReportItem).UsageMinimum & 0xFFFF) as u16;
            } else {
                if (*ReportItem).BitCount == 1 {
                    (*UsageAndPage.offset(ItemCount as isize)).Usage =
                        ((*ReportItem).UsageMinimum & 0xFFFF) as u16;
                } else {
                    (*UsageAndPage.offset(ItemCount as isize)).Usage = Data as u16;
                }
                (*UsageAndPage.offset(ItemCount as isize)).UsagePage = CurrentUsagePage;
            }
            ItemCount = ItemCount.wrapping_add(1);
        }

        Index = Index.wrapping_add(1);
    }

    if ItemCount > *UsageLength {
        return HIDP_STATUS_BUFFER_TOO_SMALL;
    }

    if UsagePage == HID_USAGE_PAGE_UNDEFINED {
        ZeroFunction(
            UsageAndPage.offset(ItemCount as isize) as *mut USAGE_AND_PAGE as *mut core::ffi::c_void,
            (*UsageLength).wrapping_sub(ItemCount).wrapping_mul(core::mem::size_of::<USAGE_AND_PAGE>() as u32),
        );
    } else {
        ZeroFunction(
            UsageList.offset(ItemCount as isize) as *mut u16 as *mut core::ffi::c_void,
            (*UsageLength).wrapping_sub(ItemCount).wrapping_mul(core::mem::size_of::<u16>() as u32),
        );
    }

    core::ptr::write_volatile(UsageLength, ItemCount);

    HIDP_STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_UsesReportId(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
) -> u32 {
    let Report: *mut _HID_REPORT = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return 0u32;
    }
    ((*Report).ReportID as i32 != 0i32) as i32 as u32
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetUsageValueWithReport(
    CollectionContext: PVOID,
    ReportType: UCHAR,
    UsagePage: USAGE,
    Usage: USAGE,
    UsageValue: PULONG,
    ReportDescriptor: PCHAR,
    ReportDescriptorLength: ULONG,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut Report: PHID_REPORT;
    let mut CurrentUsagePage: USHORT;
    let mut ReportItem: PHID_REPORT_ITEM;
    let mut Data: ULONG;

    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
    }
    if ((*Report).ReportSize as ULONG) / 8 != (ReportDescriptorLength.wrapping_sub(1)) {
        return HIDP_STATUS_INVALID_REPORT_LENGTH;
    }
    Index = 0;
    while Index < (*Report).ItemCount {
        ReportItem = ((&raw mut (*Report).Items as *mut HID_REPORT_ITEM).add(Index as usize)) as PHID_REPORT_ITEM;
        CurrentUsagePage = ((*ReportItem).UsageMinimum >> 16) as USHORT;
        if !(UsagePage as i32 != CurrentUsagePage as i32) {
            if !(Usage as u32 != ((*ReportItem).UsageMinimum as u32 & 0xffff)) {
                if !((*ReportItem).ByteOffset < ReportDescriptorLength) {
                    RtlAssert(
                        b"ReportItem->ByteOffset < ReportDescriptorLength\0" as *const u8 as *const i8 as PVOID,
                        b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0" as *const u8 as *const i8 as PVOID,
                        612,
                        core::ptr::null_mut::<core::ffi::c_void>() as PVOID,
                    );
                }
                Data = 0;
                let offset_bytes = ((*ReportItem).ByteOffset as ULONG).wrapping_add(1);
                CopyFunction(
                    core::ptr::addr_of_mut!(Data) as PVOID,
                    ReportDescriptor.add(offset_bytes as usize) as PVOID,
                    {
                        let size_of_ulong = core::mem::size_of::<ULONG>() as ULONG;
                        let available = ReportDescriptorLength.wrapping_sub(offset_bytes);
                        if size_of_ulong < available { size_of_ulong } else { available }
                    },
                );
                Data >>= (*ReportItem).Shift;
                Data &= (*ReportItem).Mask;
                core::ptr::write_volatile(UsageValue, Data);
                return HIDP_STATUS_SUCCESS;
            }
        }
        Index = Index.wrapping_add(1);
    }
    return HIDP_STATUS_USAGE_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetScaledUsageValueWithReport(
    CollectionContext: *mut core::ffi::c_void,
    ReportType: u8,
    UsagePage: u16,
    Usage: u16,
    UsageValue: *mut i32,
    ReportDescriptor: *mut i8,
    ReportDescriptorLength: u32,
) -> i32 {
    let mut Index: u32 = 0;
    let mut Report: *mut HID_REPORT = core::ptr::null_mut();
    let mut CurrentUsagePage: u16 = 0;
    let mut ReportItem: *mut HID_REPORT_ITEM = core::ptr::null_mut();
    let mut Data: u32 = 0;

    Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
    if Report.is_null() {
        return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
    }
    if ((*Report).ReportSize as u32).wrapping_div(8) != ReportDescriptorLength.wrapping_sub(1) {
        return HIDP_STATUS_INVALID_REPORT_LENGTH;
    }
    Index = 0;
    while Index < (*Report).ItemCount {
        ReportItem = (core::ptr::addr_of_mut!((*Report).Items) as *mut HID_REPORT_ITEM)
            .offset(Index as isize);
        CurrentUsagePage = ((*ReportItem).UsageMinimum >> 16) as u16;
        if UsagePage as i32 == CurrentUsagePage as i32 {
            if Usage as u32 == ((*ReportItem).UsageMinimum as u32 & 0xFFFF) {
                if !((*ReportItem).ByteOffset < ReportDescriptorLength) {
                    RtlAssert(
                        b"ReportItem->ByteOffset < ReportDescriptorLength\0".as_ptr()
                            as *const core::ffi::c_void,
                        b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0"
                            .as_ptr() as *const core::ffi::c_void,
                        708u32,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                    );
                }
                Data = 0;
                let copy_size = {
                    let size_of_ulong = core::mem::size_of::<u32>() as u32;
                    let available = ReportDescriptorLength
                        .wrapping_sub((*ReportItem).ByteOffset as u32 + 1);
                    if size_of_ulong < available {
                        size_of_ulong
                    } else {
                        available
                    }
                };
                CopyFunction(
                    &raw mut Data as *mut core::ffi::c_void,
                    ReportDescriptor.offset((*ReportItem).ByteOffset as isize + 1)
                        as *mut i8 as *mut core::ffi::c_void,
                    copy_size,
                );
                Data >>= (*ReportItem).Shift;
                Data &= (*ReportItem).Mask as u32;
                if (*ReportItem).Minimum > (*ReportItem).Maximum {
                    if Data & !((*ReportItem).Mask >> 1) != 0 {
                        Data |= !(*ReportItem).Mask as u32;
                    }
                } else {
                    return HIDP_STATUS_BAD_LOG_PHY_VALUES;
                }
                core::ptr::write_volatile(UsageValue, Data as i32);
                return HIDP_STATUS_SUCCESS;
            }
        }
        Index = Index.wrapping_add(1);
    }
    return HIDP_STATUS_USAGE_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetScanCodeFromKbdUsage(mut Usage: USAGE) -> ULONG {
    if (Usage as usize)
        < (::core::mem::size_of::<[ULONG; 256]>() as usize)
            .wrapping_div(::core::mem::size_of::<ULONG>() as usize)
    {
        return KeyboardScanCodes[Usage as usize];
    }
    return 0 as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_GetScanCodeFromCustUsage(Usage: USAGE) -> ULONG {
    let mut i: ULONG = 0;
    i = 0 as ULONG;
    while (i as usize)
        < (core::mem::size_of_val(&CustomerScanCodes)
            / core::mem::size_of_val(&CustomerScanCodes[0]))
    {
        if CustomerScanCodes[i as usize].Usage == Usage {
            return CustomerScanCodes[i as usize].ScanCode;
        }
        i = i.wrapping_add(1);
    }
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn HidParser_DispatchKey(
    ScanCodes: *mut i8,
    KeyAction: i32, // HIDP_KEYBOARD_DIRECTION (enum)
    InsertCodesProcedure: unsafe extern "system" fn(*mut core::ffi::c_void, *mut i8, u32),
    InsertCodesContext: *mut core::ffi::c_void,
) {
    let mut Length: u32 = 0;
    for Index in 0..core::mem::size_of::<u32>() {
        if *ScanCodes.add(Index) as i32 == 0 {
            break;
        }
        if KeyAction == 0 /* HidP_Keyboard_Break */ {
            let p = ScanCodes.add(Index);
            *p = (*p as u8 | 0x80) as i8;
        }
        Length += 1;
    }
    if Length > 0 {
        InsertCodesProcedure(InsertCodesContext, ScanCodes, Length);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_TranslateKbdUsage(
    Usage: u16,
    KeyAction: i32,
    ModifierState: *mut HIDP_KEYBOARD_MODIFIER_STATE,
    InsertCodesProcedure: unsafe extern "system" fn(
        *mut core::ffi::c_void,
        *mut i8,
        u32,
    ),
    InsertCodesContext: *mut core::ffi::c_void,
) -> i32 {
    // HIDP_STATUS_* come from hid_sys (imported at module scope). HIDP_KEYBOARD_DIRECTION
    // is `Break, Make` in hidpi.h → Break=0, Make=1.
    const HidP_Keyboard_Break: i32 = 0;
    const HidP_Keyboard_Make: i32 = 1;

    let mut ScanCode: u32;
    let mut FakeShift: [i8; 3] = [0xe0u8 as i8, 0x2au8 as i8, 0i8];
    let mut FakeCtrl: [i8; 3] = [0xe1u8 as i8, 0x1du8 as i8, 0i8];

    ScanCode = HidParser_GetScanCodeFromKbdUsage(Usage);
    if ScanCode == 0 {
        let dbg_str: *const i8 =
            b"(%s:%d) No Scan code for Usage %x\n\0".as_ptr() as *const i8;
        let dbg_file: *const i8 =
            b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0"
                .as_ptr() as *const i8;
        if DbgPrint(dbg_str, dbg_file, 871i32, Usage as i32) != 0 {
            let dbg_err: *const i8 =
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8;
            DbgPrint(dbg_err, dbg_file, 871i32);
        }
        return HIDP_STATUS_I8042_TRANS_UNKNOWN;
    }

    if ScanCode & 0xFF00u32 != 0 {
        ScanCode = ((ScanCode as u16 & 0xFF) << 8 | (ScanCode as u16 & 0xFF00) >> 8) as u32;
    }

    if Usage as i32 == 0x46 && KeyAction == HidP_Keyboard_Make {
        HidParser_DispatchKey(
            core::ptr::addr_of_mut!(FakeShift) as *mut i8,
            KeyAction,
            InsertCodesProcedure,
            InsertCodesContext,
        );
    }

    if Usage as i32 == 0x48 {
        HidParser_DispatchKey(
            core::ptr::addr_of_mut!(FakeCtrl) as *mut i8,
            KeyAction,
            InsertCodesProcedure,
            InsertCodesContext,
        );
    }

    HidParser_DispatchKey(
        core::ptr::addr_of_mut!(ScanCode) as *mut i8,
        KeyAction,
        InsertCodesProcedure,
        InsertCodesContext,
    );

    if Usage as i32 == 0x46 && KeyAction == HidP_Keyboard_Break {
        HidParser_DispatchKey(
            core::ptr::addr_of_mut!(FakeShift) as *mut i8,
            KeyAction,
            InsertCodesProcedure,
            InsertCodesContext,
        );
    }

    return HIDP_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn HidParser_TranslateCustUsage(
    mut Usage: USAGE,
    mut KeyAction: HIDP_KEYBOARD_DIRECTION,
    mut ModifierState: PHIDP_KEYBOARD_MODIFIER_STATE,
    mut InsertCodesProcedure: PHIDP_INSERT_SCANCODES,
    mut InsertCodesContext: PVOID,
) -> NTSTATUS {
    let mut ScanCode: ULONG = 0;
    ScanCode = HidParser_GetScanCodeFromCustUsage(Usage);
    if ScanCode == 0 {
        if DbgPrint(
            b"(%s:%d) No Scan code for Usage %x\n\0".as_ptr() as *const i8,
            b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0".as_ptr() as *const i8,
            931i32,
            Usage as i32,
        ) != 0 {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"/home/curi/Projects/os-playground/vibeos/sdk/lib/drivers/hidparser/api.c\0".as_ptr() as *const i8,
                931i32,
            );
        }
        return HIDP_STATUS_I8042_TRANS_UNKNOWN;
    }
    if (ScanCode & 0xff00u32) != 0 {
        ScanCode = (((ScanCode as u16 as i32 & 0xff) << 8) | ((ScanCode as u16 as i32 & 0xff00) >> 8)) as ULONG;
    }
    HidParser_DispatchKey(
        &raw mut ScanCode as *mut i8,
        KeyAction as i32, // HIDP_KEYBOARD_DIRECTION (enum)
        // PHIDP_INSERT_SCANCODES is extern "stdcall" on i386; DispatchKey's param
        // is extern "system" — same ABI per arch, nominally distinct fn types.
        core::mem::transmute(InsertCodesProcedure.unwrap()),
        InsertCodesContext,
    );
    return HIDP_STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS HID Parser Library
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        lib/drivers/hidparser/api.c
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
// static ULONG KeyboardScanCodes[256] =
// { /*    0       1       2       3       4       5       6       7       8       9       A       B       C       D       E       F */
// /* 0 */ 0x0000, 0x0000, 0x0000, 0x0000, 0x001e, 0x0030, 0x002e, 0x0020, 0x0012, 0x0021, 0x0022, 0x0023, 0x0017, 0x0024, 0x0025, 0x0026,
// /* 1 */ 0x0032, 0x0031, 0x0018, 0x0019, 0x0010, 0x0013, 0x001f, 0x0014, 0x0016, 0x002f, 0x0011, 0x002d, 0x0015, 0x002c, 0x0002, 0x0003,
// /* 2 */ 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000a, 0x000b, 0x001c, 0x0001, 0x000e, 0x000f, 0x0039, 0x000c, 0x000d, 0x001a,
// /* 3 */ 0x001b, 0x002b, 0x002b, 0x0027, 0x0028, 0x0029, 0x0033, 0x0034, 0x0035, 0x003a, 0x003b, 0x003c, 0x003d, 0x003e, 0x003f, 0x0040,
// /* 4 */ 0x0041, 0x0042, 0x0043, 0x0044, 0x0057, 0x0058, 0xE037, 0x0046, 0x0045, 0xE052, 0xE047, 0xE049, 0xE053, 0xE04F, 0xE051, 0xE04D,
// /* 5 */ 0xE04B, 0xE050, 0xE048, 0x0045, 0xE035, 0x0037, 0x004a, 0x004e, 0xE01C, 0x004f, 0x0050, 0x0051, 0x004b, 0x004c, 0x004d, 0x0047,
// /* 6 */ 0x0048, 0x0049, 0x0052, 0x0053, 0x0056, 0xE05D, 0xE05E, 0x0075, 0x00b7, 0x00b8, 0x00b9, 0x00ba, 0x00bb, 0x00bc, 0x00bd, 0x00be,
// /* 7 */ 0x00bf, 0x00c0, 0x00c1, 0x00c2, 0x0086, 0x008a, 0x0082, 0x0084, 0x0080, 0x0081, 0x0083, 0x0089, 0x0085, 0x0087, 0x0088, 0x0071,
// /* 8 */ 0x0073, 0x0072, 0x0000, 0x0000, 0x0000, 0x0079, 0x0000, 0x0059, 0x005d, 0x007c, 0x005c, 0x005e, 0x005f, 0x0000, 0x0000, 0x0000,
// /* 9 */ 0x007a, 0x007b, 0x005a, 0x005b, 0x0055, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
// /* A */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
// /* B */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
// /* C */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
// /* D */ 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
// /* E */ 0x001D, 0x002A, 0x0038, 0xE05B, 0xE01D, 0x0036, 0xE038, 0xE05C, 0x00a4, 0x00a6, 0x00a5, 0x00a3, 0x00a1, 0x0073, 0x0072, 0x0071,
// /* F */ 0x0096, 0x009e, 0x009f, 0x0080, 0x0088, 0x00b1, 0x00b2, 0x00b0, 0x008e, 0x0098, 0x00ad, 0x008c, 0x0000, 0x0000, 0x0000, 0x0000,
// };
//
// static struct
// {
//     USAGE Usage;
//     ULONG ScanCode;
// } CustomerScanCodes[] =
// {
//     { 0x00B5, 0xE019 },
//     { 0x00B6, 0xE010 },
//     { 0x00B7, 0xE024 },
//     { 0x00CD, 0xE022 },
//     { 0x00E2, 0xE020 },
//     { 0x00E9, 0xE030 },
//     { 0x00EA, 0xE02E },
//     { 0x0183, 0xE06D },
//     { 0x018A, 0xE06C },
//     { 0x0192, 0xE021 },
//     { 0x0194, 0xE06B },
//     { 0x0221, 0xE065 },
//     { 0x0223, 0xE032 },
//     { 0x0224, 0xE06A },
//     { 0x0225, 0xE069 },
//     { 0x0226, 0xE068 },
//     { 0x0227, 0xE067 },
//     { 0x022A, 0xE066 },
// };
//
// #define NTOHS(n) (((((unsigned short)(n) & 0xFF)) << 8) | (((unsigned short)(n) & 0xFF00) >> 8))
//
// NTSTATUS
// HidParser_GetCollectionUsagePage(
//     IN PVOID CollectionContext,
//     OUT PUSHORT Usage,
//     OUT PUSHORT UsagePage)
// {
//     PHID_COLLECTION Collection;
//
//     //
//     // find collection
//     //
//     Collection = HidParser_GetCollectionFromContext(CollectionContext);
//     if (!Collection)
//     {
//         //
//         // collection not found
//         //
//         return HIDP_STATUS_USAGE_NOT_FOUND;
//     }
//
//     //
//     // store result
//     //
//     *UsagePage = (Collection->Usage >> 16);
//     *Usage = (Collection->Usage & 0xFFFF);
//     return HIDP_STATUS_SUCCESS;
// }
//
// ULONG
// HidParser_GetReportLength(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType)
// {
//     PHID_REPORT Report;
//     ULONG ReportLength;
//
//     //
//     // get first report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no report found
//         //
//         return 0;
//     }
//
//     //
//     // get report length
//     //
//     ReportLength = Report->ReportSize;
//
//     //
//     // done
//     //
//     if (ReportLength)
//     {
//         //
//         // byte aligned length
//         //
//         ASSERT(ReportLength % 8 == 0);
//         return ReportLength / 8;
//     }
//     return ReportLength;
// }
//
// ULONG
// HidParser_GetReportItemCountFromReportType(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType)
// {
//     PHID_REPORT Report;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return 0;
//     }
//
//     //
//     // return report item count
//     //
//     return Report->ItemCount;
// }
//
//
// ULONG
// HidParser_GetReportItemTypeCountFromReportType(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType,
//     IN ULONG bData)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     ULONG ItemCount = 0;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return 0;
//     }
//
//     //
//     // enumerate all items
//     //
//     for(Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // check item type
//         //
//         if (Report->Items[Index].HasData && bData)
//         {
//             //
//             // found data item
//             //
//             ItemCount++;
//         }
//         else if (Report->Items[Index].HasData == FALSE && bData == FALSE)
//         {
//             //
//             // found value item
//             //
//             ItemCount++;
//         }
//     }
//
//     //
//     // no report items
//     //
//     return ItemCount;
// }
//
// ULONG
// HidParser_GetMaxUsageListLengthWithReportAndPage(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType,
//     IN USAGE  UsagePage  OPTIONAL)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     ULONG ItemCount = 0;
//     USHORT CurrentUsagePage;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return 0;
//     }
//
//     for(Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // check usage page
//         //
//         CurrentUsagePage = (Report->Items[Index].UsageMinimum >> 16);
//         if (CurrentUsagePage == UsagePage && Report->Items[Index].HasData)
//         {
//             //
//             // found item
//             //
//             ItemCount++;
//         }
//     }
//
//     //
//     // done
//     //
//     return ItemCount;
// }
//
// NTSTATUS
// HidParser_GetSpecificValueCapsWithReport(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType,
//     IN USHORT UsagePage,
//     IN USHORT Usage,
//     OUT PHIDP_VALUE_CAPS  ValueCaps,
//     IN OUT PUSHORT  ValueCapsLength)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     USHORT ItemCount = 0;
//     USHORT CurrentUsagePage;
//     USHORT CurrentUsage;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
//     }
//
//     for(Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // check usage page
//         //
//         CurrentUsagePage = (Report->Items[Index].UsageMinimum >> 16);
//         CurrentUsage = (Report->Items[Index].UsageMinimum & 0xFFFF);
//
//         if ((Usage == CurrentUsage && UsagePage == CurrentUsagePage) || (Usage == 0 && UsagePage == CurrentUsagePage) || (Usage == CurrentUsage && UsagePage == 0) || (Usage == 0 && UsagePage == 0))
//         {
//             //
//             // check if there is enough place for the caps
//             //
//             if (ItemCount < *ValueCapsLength)
//             {
//                 //
//                 // zero caps
//                 //
//                 ZeroFunction(&ValueCaps[ItemCount], sizeof(HIDP_VALUE_CAPS));
//
//                 //
//                 // init caps
//                 //
//                 ValueCaps[ItemCount].UsagePage = CurrentUsagePage;
//                 ValueCaps[ItemCount].ReportID = Report->ReportID;
//                 ValueCaps[ItemCount].LogicalMin = Report->Items[Index].Minimum;
//                 ValueCaps[ItemCount].LogicalMax = Report->Items[Index].Maximum;
//                 ValueCaps[ItemCount].IsAbsolute = !Report->Items[Index].Relative;
//                 ValueCaps[ItemCount].BitSize = Report->Items[Index].BitCount;
//
//                 //
//                 // FIXME: FILLMEIN
//                 //
//             }
//
//
//             //
//             // found item
//             //
//             ItemCount++;
//         }
//     }
//
//     //
//     // store result
//     //
//     *ValueCapsLength = ItemCount;
//
//     if (ItemCount)
//     {
//         //
//         // success
//         //
//         return HIDP_STATUS_SUCCESS;
//     }
//
//     //
//     // item not found
//     //
//     return HIDP_STATUS_USAGE_NOT_FOUND;
// }
//
// NTSTATUS
// HidParser_GetUsagesWithReport(
//     IN PVOID CollectionContext,
//     IN UCHAR  ReportType,
//     IN USAGE  UsagePage,
//     OUT USAGE  *UsageList,
//     IN OUT PULONG UsageLength,
//     IN PCHAR  ReportDescriptor,
//     IN ULONG  ReportDescriptorLength)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     ULONG ItemCount = 0;
//     USHORT CurrentUsagePage;
//     PHID_REPORT_ITEM ReportItem;
//     UCHAR Activated;
//     ULONG Data;
//     PUSAGE_AND_PAGE UsageAndPage = NULL;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
//     }
//
//     if (Report->ReportSize / 8 != (ReportDescriptorLength - 1))
//     {
//         //
//         // invalid report descriptor length
//         //
//         return HIDP_STATUS_INVALID_REPORT_LENGTH;
//     }
//
//     //
//     // cast to usage and page
//     //
//     if (UsagePage == HID_USAGE_PAGE_UNDEFINED)
//     {
//         //
//         // the caller requested any set usages
//         //
//         UsageAndPage = (PUSAGE_AND_PAGE)UsageList;
//     }
//
//     for(Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // get report item
//         //
//         ReportItem = &Report->Items[Index];
//
//         //
//         // does it have data
//         //
//         if (!ReportItem->HasData)
//             continue;
//
//         //
//         // check usage page
//         //
//         CurrentUsagePage = (ReportItem->UsageMinimum >> 16);
//
//         if (UsagePage != HID_USAGE_PAGE_UNDEFINED)
//         {
//             //
//             // does usage match
//             //
//             if (UsagePage != CurrentUsagePage)
//                 continue;
//         }
//
//         //
//         // check if the specified usage is activated
//         //
//         ASSERT(ReportItem->ByteOffset < ReportDescriptorLength);
//         ASSERT(ReportItem->BitCount <= 8);
//
//         //
//         // one extra shift for skipping the prepended report id
//         //
//         Data = ReportDescriptor[ReportItem->ByteOffset + 1];
//
//         //
//         // shift data
//         //
//         Data >>= ReportItem->Shift;
//
//         //
//         // clear unwanted bits
//         //
//         Data &= ReportItem->Mask;
//
//         //
//         // is it activated
//         //
//         Activated = (Data != 0);
//
//         if (!Activated)
//             continue;
//
//         //
//         // is there enough space for the usage
//         //
//         if (ItemCount >= *UsageLength)
//         {
//             ItemCount++;
//             continue;
//         }
//
//         if (UsagePage != HID_USAGE_PAGE_UNDEFINED)
//         {
//             //
//             // store item
//             //
//             UsageList[ItemCount] = (ReportItem->UsageMinimum & 0xFFFF);
//         }
//         else
//         {
//             //
//             // store usage and page
//             //
//             if (ReportItem->BitCount == 1)
//             {
//                 //
//                 // use usage minimum
//                 //
//                 UsageAndPage[ItemCount].Usage =(ReportItem->UsageMinimum & 0xFFFF);
//             }
//             else
//             {
//                 //
//                 // use value from control
//                 //
//                 UsageAndPage[ItemCount].Usage = (USHORT)Data;
//             }
//             UsageAndPage[ItemCount].UsagePage = CurrentUsagePage;
//         }
//         ItemCount++;
//     }
//
//     if (ItemCount > *UsageLength)
//     {
//         //
//         // list too small
//         //
//         return HIDP_STATUS_BUFFER_TOO_SMALL;
//     }
//
//     if (UsagePage == HID_USAGE_PAGE_UNDEFINED)
//     {
//         //
//         // success, clear rest of array
//         //
//         ZeroFunction(&UsageAndPage[ItemCount], (*UsageLength - ItemCount) * sizeof(USAGE_AND_PAGE));
//     }
//     else
//     {
//         //
//         // success, clear rest of array
//         //
//         ZeroFunction(&UsageList[ItemCount], (*UsageLength - ItemCount) * sizeof(USAGE));
//     }
//
//
//     //
//     // store result size
//     //
//     *UsageLength = ItemCount;
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// ULONG
// HidParser_UsesReportId(
//     IN PVOID CollectionContext,
//     IN UCHAR  ReportType)
// {
//     PHID_REPORT Report;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return 0;
//     }
//
//     //
//     // returns true when report id != 0
//     //
//     return (Report->ReportID != 0);
//
// }
//
// NTSTATUS
// HidParser_GetUsageValueWithReport(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType,
//     IN USAGE UsagePage,
//     IN USAGE  Usage,
//     OUT PULONG UsageValue,
//     IN PCHAR ReportDescriptor,
//     IN ULONG ReportDescriptorLength)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     USHORT CurrentUsagePage;
//     PHID_REPORT_ITEM ReportItem;
//     ULONG Data;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
//     }
//
//     if (Report->ReportSize / 8 != (ReportDescriptorLength - 1))
//     {
//         //
//         // invalid report descriptor length
//         //
//         return HIDP_STATUS_INVALID_REPORT_LENGTH;
//     }
//
//     for (Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // get report item
//         //
//         ReportItem = &Report->Items[Index];
//
//         //
//         // check usage page
//         //
//         CurrentUsagePage = (ReportItem->UsageMinimum >> 16);
//
//         //
//         // does usage page match
//         //
//         if (UsagePage != CurrentUsagePage)
//             continue;
//
//         //
//         // does the usage match
//         //
//         if (Usage != (ReportItem->UsageMinimum & 0xFFFF))
//             continue;
//
//         //
//         // check if the specified usage is activated
//         //
//         ASSERT(ReportItem->ByteOffset < ReportDescriptorLength);
//
//         //
//         // one extra shift for skipping the prepended report id
//         //
//         Data = 0;
//         CopyFunction(&Data, &ReportDescriptor[ReportItem->ByteOffset + 1], min(sizeof(ULONG), ReportDescriptorLength - (ReportItem->ByteOffset + 1)));
//
//         //
//         // shift data
//         //
//         Data >>= ReportItem->Shift;
//
//         //
//         // clear unwanted bits
//         //
//         Data &= ReportItem->Mask;
//
//         //
//         // store result
//         //
//         *UsageValue = Data;
//         return HIDP_STATUS_SUCCESS;
//     }
//
//     //
//     // usage not found
//     //
//     return HIDP_STATUS_USAGE_NOT_FOUND;
// }
//
//
//
// NTSTATUS
// HidParser_GetScaledUsageValueWithReport(
//     IN PVOID CollectionContext,
//     IN UCHAR ReportType,
//     IN USAGE UsagePage,
//     IN USAGE  Usage,
//     OUT PLONG UsageValue,
//     IN PCHAR ReportDescriptor,
//     IN ULONG ReportDescriptorLength)
// {
//     ULONG Index;
//     PHID_REPORT Report;
//     USHORT CurrentUsagePage;
//     PHID_REPORT_ITEM ReportItem;
//     ULONG Data;
//
//     //
//     // get report
//     //
//     Report = HidParser_GetReportInCollection(CollectionContext, ReportType);
//     if (!Report)
//     {
//         //
//         // no such report
//         //
//         return HIDP_STATUS_REPORT_DOES_NOT_EXIST;
//     }
//
//     if (Report->ReportSize / 8 != (ReportDescriptorLength - 1))
//     {
//         //
//         // invalid report descriptor length
//         //
//         return HIDP_STATUS_INVALID_REPORT_LENGTH;
//     }
//
//     for (Index = 0; Index < Report->ItemCount; Index++)
//     {
//         //
//         // get report item
//         //
//         ReportItem = &Report->Items[Index];
//
//         //
//         // check usage page
//         //
//         CurrentUsagePage = (ReportItem->UsageMinimum >> 16);
//
//         //
//         // does usage page match
//         //
//         if (UsagePage != CurrentUsagePage)
//             continue;
//
//         //
//         // does the usage match
//         //
//         if (Usage != (ReportItem->UsageMinimum & 0xFFFF))
//             continue;
//
//         //
//         // check if the specified usage is activated
//         //
//         ASSERT(ReportItem->ByteOffset < ReportDescriptorLength);
//
//         //
//         // one extra shift for skipping the prepended report id
//         //
//         Data = 0;
//         CopyFunction(&Data, &ReportDescriptor[ReportItem->ByteOffset + 1], min(sizeof(ULONG), ReportDescriptorLength - (ReportItem->ByteOffset + 1)));
//
//         //
//         // shift data
//         //
//         Data >>= ReportItem->Shift;
//
//         //
//         // clear unwanted bits
//         //
//         Data &= ReportItem->Mask;
//
//         if (ReportItem->Minimum > ReportItem->Maximum)
//         {
//             //
//             // logical boundaries are signed values
//             //
//
//             // FIXME: scale with physical min/max
//             if ((Data & ~(ReportItem->Mask >> 1)) != 0)
//             {
//                 Data |= ~ReportItem->Mask;
//             }
//         }
//         else
//         {
//             // logical boundaries are absolute values
//             return HIDP_STATUS_BAD_LOG_PHY_VALUES;
//         }
//
//         //
//         // store result
//         //
//         *UsageValue = Data;
//         return HIDP_STATUS_SUCCESS;
//     }
//
//     //
//     // usage not found
//     //
//     return HIDP_STATUS_USAGE_NOT_FOUND;
// }
//
// ULONG
// HidParser_GetScanCodeFromKbdUsage(
//     IN USAGE Usage)
// {
//     if (Usage < sizeof(KeyboardScanCodes) / sizeof(KeyboardScanCodes[0]))
//     {
//         //
//         // valid usage
//         //
//         return KeyboardScanCodes[Usage];
//     }
//
//     //
//     // invalid usage
//     //
//     return 0;
// }
//
// ULONG
// HidParser_GetScanCodeFromCustUsage(
//     IN USAGE Usage)
// {
//     ULONG i;
//
//     //
//     // find usage in array
//     //
//     for (i = 0; i < sizeof(CustomerScanCodes) / sizeof(CustomerScanCodes[0]); ++i)
//     {
//         if (CustomerScanCodes[i].Usage == Usage)
//         {
//             //
//             // valid usage
//             //
//             return CustomerScanCodes[i].ScanCode;
//         }
//     }
//
//     //
//     // invalid usage
//     //
//     return 0;
// }
//
// VOID
// HidParser_DispatchKey(
//     IN PCHAR ScanCodes,
//     IN HIDP_KEYBOARD_DIRECTION KeyAction,
//     IN PHIDP_INSERT_SCANCODES InsertCodesProcedure,
//     IN PVOID InsertCodesContext)
// {
//     ULONG Index;
//     ULONG Length = 0;
//
//     //
//     // count code length
//     //
//     for(Index = 0; Index < sizeof(ULONG); Index++)
//     {
//         if (ScanCodes[Index] == 0)
//         {
//             //
//             // last scan code
//             //
//             break;
//         }
//
//         //
//         // is this a key break
//         //
//         if (KeyAction == HidP_Keyboard_Break)
//         {
//             //
//             // add break - see USB HID to PS/2 Scan Code Translation Table
//             //
//             ScanCodes[Index] |= 0x80;
//         }
//
//         //
//         // more scan counts
//         //
//         Length++;
//     }
//
//     if (Length > 0)
//     {
//          //
//          // dispatch scan codes
//          //
//          InsertCodesProcedure(InsertCodesContext, ScanCodes, Length);
//     }
// }
//
// NTSTATUS
// HidParser_TranslateKbdUsage(
//     IN USAGE Usage,
//     IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//     IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//     IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//     IN PVOID  InsertCodesContext)
// {
//     ULONG ScanCode;
//     CHAR FakeShift[] = {0xE0, 0x2A, 0x00};
//     CHAR FakeCtrl[] = {0xE1, 0x1D, 0x00};
//
//     //
//     // get scan code
//     //
//     ScanCode = HidParser_GetScanCodeFromKbdUsage(Usage);
//     if (!ScanCode)
//     {
//         //
//         // invalid lookup or no scan code available
//         //
//         DPRINT1("No Scan code for Usage %x\n", Usage);
//         return HIDP_STATUS_I8042_TRANS_UNKNOWN;
//     }
//
//     if (ScanCode & 0xFF00)
//     {
//         //
//         // swap scan code
//         //
//         ScanCode = NTOHS(ScanCode);
//     }
//
//     if (Usage == 0x46 && KeyAction == HidP_Keyboard_Make)
//     {
//         // Print Screen generates additional FakeShift
//         HidParser_DispatchKey(FakeShift, KeyAction, InsertCodesProcedure, InsertCodesContext);
//     }
//
//     if (Usage == 0x48)
//     {
//         // Pause/Break generates additional FakeCtrl. Note: it's always before key press/release.
//         HidParser_DispatchKey(FakeCtrl, KeyAction, InsertCodesProcedure, InsertCodesContext);
//     }
//
//     //
//     // FIXME: translate modifier states
//     //
//     HidParser_DispatchKey((PCHAR)&ScanCode, KeyAction, InsertCodesProcedure, InsertCodesContext);
//
//     if (Usage == 0x46 && KeyAction == HidP_Keyboard_Break)
//     {
//         // Print Screen generates additional FakeShift
//         HidParser_DispatchKey(FakeShift, KeyAction, InsertCodesProcedure, InsertCodesContext);
//     }
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// NTSTATUS
// HidParser_TranslateCustUsage(
//     IN USAGE Usage,
//     IN HIDP_KEYBOARD_DIRECTION  KeyAction,
//     IN OUT PHIDP_KEYBOARD_MODIFIER_STATE  ModifierState,
//     IN PHIDP_INSERT_SCANCODES  InsertCodesProcedure,
//     IN PVOID  InsertCodesContext)
// {
//     ULONG ScanCode;
//
//     //
//     // get scan code
//     //
//     ScanCode = HidParser_GetScanCodeFromCustUsage(Usage);
//     if (!ScanCode)
//     {
//         //
//         // invalid lookup or no scan code available
//         //
//         DPRINT1("No Scan code for Usage %x\n", Usage);
//         return HIDP_STATUS_I8042_TRANS_UNKNOWN;
//     }
//
//     if (ScanCode & 0xFF00)
//     {
//         //
//         // swap scan code
//         //
//         ScanCode = NTOHS(ScanCode);
//     }
//
//     //
//     // FIXME: translate modifier states
//     //
//     HidParser_DispatchKey((PCHAR)&ScanCode, KeyAction, InsertCodesProcedure, InsertCodesContext);
//
//     //
//     // done
//     //
//     return HIDP_STATUS_SUCCESS;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
