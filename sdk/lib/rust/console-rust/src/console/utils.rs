use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn BisectListSortedByValueEx(
    mut ListCtl: PLIST_CTL,
    mut Value: ULONG_PTR,
    mut itemStart: UINT,
    mut itemEnd: UINT,
    mut pValueItem: PUINT,
    mut BisectRightOrLeft: BOOL,
) -> UINT {
    let mut iItemStart: UINT = 0;
    let mut iItemEnd: UINT = 0;
    let mut iItem: UINT = 0;
    let mut itemData: ULONG_PTR = 0;
    if itemStart > itemEnd {
        return CB_ERR as UINT;
    }
    iItemStart = itemStart;
    iItemEnd = itemEnd;
    iItem = iItemStart;
    if !pValueItem.is_null() {
        *pValueItem = CB_ERR as ::core::ffi::c_uint;
    }
    while iItemStart <= iItemEnd {
        iItem = iItemStart
            .wrapping_add(iItemEnd)
            .wrapping_div(2 as UINT);
        itemData = (*ListCtl).GetData.unwrap_unchecked()(
            ListCtl as *mut _LIST_CTL,
            iItem as INT,
        );
        if itemData == CB_ERR as ULONG_PTR {
            return CB_ERR as UINT;
        }
        if Value == itemData {
            if !pValueItem.is_null() {
                *pValueItem = iItem as ::core::ffi::c_uint;
            }
            if BisectRightOrLeft != 0 {
                iItemStart = iItem.wrapping_add(1 as UINT);
            } else {
                if iItem <= itemStart {
                    break;
                }
                iItemEnd = iItem.wrapping_sub(1 as UINT);
            }
        } else if Value < itemData {
            if iItem <= itemStart {
                break;
            }
            iItemEnd = iItem.wrapping_sub(1 as UINT);
        } else {
            iItemStart = iItem.wrapping_add(1 as UINT);
        }
    }
    return iItemStart;
}
#[no_mangle]
pub unsafe extern "C" fn BisectListSortedByValue(ListCtl: PLIST_CTL, Value: ULONG_PTR, pValueItem: PUINT, BisectRightOrLeft: BOOL) -> UINT {
    let mut iItemEnd: INT = ((*ListCtl).GetCount.unwrap_unchecked())(ListCtl as *mut _LIST_CTL);
    if iItemEnd == CB_ERR || iItemEnd <= 0 {
        return (CB_ERR) as UINT;
    }
    return BisectListSortedByValueEx(
        ListCtl,
        Value,
        0,
        (iItemEnd - 1) as UINT,
        pValueItem,
        BisectRightOrLeft,
    );
}
