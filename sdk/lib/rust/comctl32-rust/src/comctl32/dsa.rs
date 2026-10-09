use crate::comctl32::comctl32undoc::Alloc;
use crate::comctl32::comctl32undoc::Free;
use crate::comctl32::comctl32undoc::ReAlloc;
use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_dsa);
// ---- file-scope preamble (translated) ----
pub type DSA = _DSA;
pub type PDSA = *mut _DSA;

#[no_mangle]
pub unsafe extern "system" fn DSA_Create(nSize: INT, nGrow: INT) -> HDSA {
    let mut hdsa: HDSA = core::ptr::null_mut();

    hdsa = Alloc(core::mem::size_of::<_DSA>() as DWORD) as HDSA;
    if !hdsa.is_null() {
        (*hdsa).nItemCount = 0;
        (*hdsa).pData = core::ptr::null_mut();
        (*hdsa).nMaxCount = 0;
        (*hdsa).nItemSize = nSize;
        (*hdsa).nGrow = max(1, nGrow);
    }

    return hdsa;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_Destroy(hdsa: HDSA) -> BOOL {
    if hdsa.is_null() {
        return FALSE;
    }

    if !(*hdsa).pData.is_null() && Free((*hdsa).pData) == 0 {
        return FALSE;
    }

    return Free(hdsa as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn DSA_GetItem(hdsa: HDSA, nIndex: INT, pDest: LPVOID) -> BOOL {
    let mut pSrc: LPVOID;

    if hdsa.is_null() {
        return FALSE;
    }
    if (nIndex < 0) || (nIndex >= (*hdsa).nItemCount) {
        return FALSE;
    }

    pSrc = ((*hdsa).pData as *mut core::ffi::c_char)
        .offset(((*hdsa).nItemSize * nIndex) as isize) as LPVOID;
    memmove(
        pDest as *mut core::ffi::c_void,
        pSrc as *const core::ffi::c_void,
        (*hdsa).nItemSize as usize,
    );

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_GetItemPtr(hdsa: HDSA, nIndex: INT) -> LPVOID {
    let mut pSrc: LPVOID;

    if hdsa.is_null() {
        return core::ptr::null_mut();
    }
    if (nIndex < 0) || (nIndex >= (*hdsa).nItemCount) {
        return core::ptr::null_mut();
    }

    pSrc = ((*hdsa).pData as *mut core::ffi::c_char)
        .offset(((*hdsa).nItemSize * nIndex) as isize) as LPVOID;

    return pSrc;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_SetItem(hdsa: HDSA, nIndex: INT, pSrc: LPVOID) -> BOOL {
    let mut nSize: INT = 0;
    let mut nNewItems: INT = 0;
    let mut pDest: LPVOID = core::ptr::null_mut();
    let mut lpTemp: LPVOID = core::ptr::null_mut();

    if hdsa.is_null() || nIndex < 0 {
        return FALSE;
    }

    if (*hdsa).nItemCount <= nIndex {
        /* within the old array */
        if (*hdsa).nMaxCount > nIndex {
            /* within the allocated space, set a new boundary */
            (*hdsa).nItemCount = nIndex + 1;
        } else {
            /* resize the block of memory */
            nNewItems = (*hdsa).nGrow
                * ((((nIndex + 1) - 1) / (*hdsa).nGrow)
                    + 1);
            nSize = (*hdsa).nItemSize * nNewItems;

            lpTemp = ReAlloc((*hdsa).pData, nSize as DWORD);
            if lpTemp.is_null() {
                return FALSE;
            }

            (*hdsa).nMaxCount = nNewItems;
            (*hdsa).nItemCount = nIndex + 1;
            (*hdsa).pData = lpTemp;
        }
    }

    /* put the new entry in */
    pDest = ((*hdsa).pData as *mut core::ffi::c_char).offset(((*hdsa).nItemSize * nIndex) as isize)
        as LPVOID;
    memmove(pDest, pSrc, (*hdsa).nItemSize as usize);

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_InsertItem(hdsa: HDSA, mut nIndex: INT, mut pSrc: LPVOID) -> INT {
    let mut nNewItems: INT;
    let mut nSize: INT;
    let mut lpTemp: LPVOID;
    let mut lpDest: LPVOID;

    if hdsa.is_null() || nIndex < 0 {
        return -1;
    }

    /* when nIndex >= nItemCount then append */
    if nIndex >= (*hdsa).nItemCount {
        nIndex = (*hdsa).nItemCount;
    }

    /* do we need to resize ? */
    if (*hdsa).nItemCount >= (*hdsa).nMaxCount {
        nNewItems = (*hdsa).nMaxCount + (*hdsa).nGrow;
        nSize = (*hdsa).nItemSize * nNewItems;

        if nSize / (*hdsa).nItemSize != nNewItems {
            return -1;
        }

        lpTemp = ReAlloc((*hdsa).pData, nSize as DWORD);
        if lpTemp.is_null() {
            return -1;
        }

        (*hdsa).nMaxCount = nNewItems;
        (*hdsa).pData = lpTemp;
    }

    /* do we need to move elements ? */
    if nIndex < (*hdsa).nItemCount {
        lpTemp = ((*hdsa).pData as *mut core::ffi::c_char)
            .offset(((*hdsa).nItemSize * nIndex) as isize) as LPVOID;
        lpDest = (lpTemp as *mut core::ffi::c_char).offset((*hdsa).nItemSize as isize) as LPVOID;
        nSize = ((*hdsa).nItemCount - nIndex) * (*hdsa).nItemSize;

        memmove(
            lpDest as *mut core::ffi::c_void,
            lpTemp as *const core::ffi::c_void,
            nSize as usize,
        );
    }

    /* ok, we can put the new Item in */
    (*hdsa).nItemCount += 1;
    lpDest = ((*hdsa).pData as *mut core::ffi::c_char)
        .offset(((*hdsa).nItemSize * nIndex) as isize) as LPVOID;

    memmove(
        lpDest as *mut core::ffi::c_void,
        pSrc as *const core::ffi::c_void,
        (*hdsa).nItemSize as usize,
    );

    return nIndex;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_DeleteItem(hdsa: HDSA, nIndex: INT) -> INT {
    let mut lpDest: LPVOID = core::ptr::null_mut();
    let mut lpSrc: LPVOID = core::ptr::null_mut();
    let mut nSize: INT;

    if hdsa.is_null() {
        return -1;
    }
    if nIndex < 0 || nIndex >= (*hdsa).nItemCount {
        return -1;
    }

    /* do we need to move ? */
    if nIndex < (*hdsa).nItemCount - 1 {
        lpDest = ((*hdsa).pData as *mut core::ffi::c_char)
            .offset(((*hdsa).nItemSize * nIndex) as isize) as LPVOID;
        lpSrc = (lpDest as *mut core::ffi::c_char).offset((*hdsa).nItemSize as isize) as LPVOID;
        nSize = (*hdsa).nItemSize * ((*hdsa).nItemCount - nIndex - 1);
        memmove(
            lpDest as *mut core::ffi::c_void,
            lpSrc as *const core::ffi::c_void,
            nSize as usize,
        );
    }

    (*hdsa).nItemCount -= 1;

    /* free memory ? */
    if ((*hdsa).nMaxCount - (*hdsa).nItemCount) >= (*hdsa).nGrow {
        nSize = (*hdsa).nItemSize * (*hdsa).nItemCount;

        lpDest = ReAlloc((*hdsa).pData, nSize as DWORD);
        if lpDest.is_null() {
            return -1;
        }

        (*hdsa).nMaxCount = (*hdsa).nItemCount;
        (*hdsa).pData = lpDest;
    }

    return nIndex;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_DeleteAllItems(hdsa: HDSA) -> BOOL {
    if hdsa.is_null() {
        return FALSE;
    }
    if !(*hdsa).pData.is_null() && (Free((*hdsa).pData) == 0) {
        return FALSE;
    }

    (*hdsa).nItemCount = 0;
    (*hdsa).pData = core::ptr::null_mut();
    (*hdsa).nMaxCount = 0;

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_EnumCallback(hdsa: HDSA, enumProc: PFNDSAENUMCALLBACK, lParam: LPVOID) {
    let mut i: INT;

    if hdsa.is_null() {
        return;
    }
    if (*hdsa).nItemCount <= 0 {
        return;
    }

    i = 0;
    while i < (*hdsa).nItemCount {
        let lpItem: LPVOID = DSA_GetItemPtr(hdsa, i) as LPVOID;
        if enumProc.unwrap_unchecked()(lpItem, lParam) == 0 {
            return;
        }
        i += 1;
    }

    return;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_DestroyCallback(hdsa: HDSA, enumProc: PFNDSAENUMCALLBACK, lParam: LPVOID) {
    DSA_EnumCallback(hdsa, enumProc, lParam);
    DSA_Destroy(hdsa);
}
#[no_mangle]
pub unsafe extern "system" fn DSA_Clone(hdsa: HDSA) -> HDSA {
    let mut dest: HDSA;
    let mut i: INT;

    if hdsa.is_null() {
        return core::ptr::null_mut();
    }

    dest = DSA_Create((*hdsa).nItemSize, (*hdsa).nGrow);
    if dest.is_null() {
        return core::ptr::null_mut();
    }

    i = 0;
    while i < (*hdsa).nItemCount {
        let ptr: LPVOID = DSA_GetItemPtr(hdsa, i);
        if DSA_InsertItem(dest, (DA_LAST) as INT, ptr) == -1 {
            DSA_Destroy(dest);
            return core::ptr::null_mut();
        }
        i += 1;
    }

    return dest;
}

#[no_mangle]
pub unsafe extern "system" fn DSA_GetSize(hdsa: HDSA) -> ULONGLONG {
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dsa.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DSA_GetSize\0" as *const u8 as *const core::ffi::c_char,
            490 as core::ffi::c_int,
            b"(%p)\n\0" as *const u8 as *const core::ffi::c_char,
            hdsa,
        );
    }

    if hdsa.is_null() {
        return 0;
    }

    return (core::mem::size_of::<_DSA>() as ULONGLONG).wrapping_add(
        ((*hdsa).nMaxCount as ULONGLONG).wrapping_mul((*hdsa).nItemSize as ULONGLONG),
    );
}
