use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const CLIST_ID_CONTAINER: u32 = !0u32;

unsafe extern "C" fn NextItem(lpList: LPDBLIST) -> LPDATABLOCK_HEADER {
    let mut address: *mut i8 = lpList as *mut i8;
    address = address.offset((*lpList).cbSize as isize);
    return address as LPDATABLOCK_HEADER;
}

#[no_mangle]
pub unsafe extern "system" fn SHAddDataBlock(lppList: *mut LPDBLIST, lpNewItem: *const DATABLOCK_HEADER) -> BOOL {
    let mut lpInsertAt: LPDATABLOCK_HEADER = core::ptr::null_mut();
    let mut ulSize: ULONG = 0;

    if lppList.is_null() || lpNewItem.is_null() {
        return (FALSE) as BOOL;
    }

    if ((*lpNewItem).cbSize as usize) < core::mem::size_of::<DATABLOCK_HEADER>()
        || (*lpNewItem).dwSignature == CLIST_ID_CONTAINER as DWORD
    {
        return (FALSE) as BOOL;
    }

    ulSize = (*lpNewItem).cbSize as ULONG;

    if (ulSize & 0x3 as ULONG) != 0 {
        ulSize = ((ulSize.wrapping_add(0x3 as ULONG)) & 0xFFFFFFFC as ULONG)
            .wrapping_add(core::mem::size_of::<DATABLOCK_HEADER>() as ULONG);
    }

    if (*lppList).is_null() {
        *lppList = LocalAlloc(
            LMEM_ZEROINIT as UINT,
            (ulSize as SIZE_T).wrapping_add(core::mem::size_of::<ULONG>() as SIZE_T),
        ) as LPDBLIST;
        lpInsertAt = *lppList as LPDATABLOCK_HEADER;
    } else {
        let mut ulTotalSize: ULONG = 0 as ULONG;
        let mut lpIter: LPDATABLOCK_HEADER = *lppList;

        while (*lpIter).cbSize != 0 {
            ulTotalSize = ulTotalSize.wrapping_add((*lpIter).cbSize as ULONG);
            lpIter = NextItem(lpIter as LPDBLIST) as LPDATABLOCK_HEADER;
        }

        lpIter = LocalReAlloc(
            *lppList as HLOCAL,
            (ulTotalSize.wrapping_add(ulSize) as SIZE_T).wrapping_add(core::mem::size_of::<ULONG>() as SIZE_T),
            (LMEM_ZEROINIT | LMEM_MOVEABLE) as UINT,
        ) as LPDATABLOCK_HEADER;

        if !lpIter.is_null() {
            *lppList = lpIter as LPDBLIST;
            lpInsertAt = (lpIter as *mut core::ffi::c_char).offset(ulTotalSize as isize) as LPDATABLOCK_HEADER;
        }
    }

    if !lpInsertAt.is_null() {
        let mut lpDest: LPDATABLOCK_HEADER = lpInsertAt;

        if ulSize != (*lpNewItem).cbSize as ULONG {
            (*lpInsertAt).cbSize = ulSize as DWORD;
            (*lpInsertAt).dwSignature = CLIST_ID_CONTAINER as DWORD;
            lpDest = lpDest.offset(1);
        }
        memcpy(
            lpDest as *mut core::ffi::c_void,
            lpNewItem as *const core::ffi::c_void,
            (*lpNewItem).cbSize as usize,
        );

        lpInsertAt = NextItem(lpInsertAt as LPDBLIST) as LPDATABLOCK_HEADER;
        (*lpInsertAt).cbSize = 0 as DWORD;

        return (TRUE) as BOOL;
    }
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SHWriteDataBlockList(
    lpStream: *mut IStream,
    mut lpList: LPDBLIST,
) -> HRESULT {
    let mut ulSize: ULONG = 0;
    let mut hRet: HRESULT = S_OK;
    WINE_TRACE!("(%p,%p)\n", lpStream, lpList);
    if !lpList.is_null() {
        while (*lpList).cbSize != 0 {
            let mut lpItem: LPDATABLOCK_HEADER = lpList as LPDATABLOCK_HEADER;
            if (*lpList).dwSignature == CLIST_ID_CONTAINER as DWORD {
                lpItem = lpItem.offset(1);
            }
            hRet = IStream_Write(
                lpStream,
                lpItem as *const core::ffi::c_void,
                (*lpItem).cbSize as ULONG,
                core::ptr::addr_of_mut!(ulSize),
            );
            if hRet < 0 {
                return hRet;
            }
            if (*lpItem).cbSize != ulSize {
                return STG_E_MEDIUMFULL;
            }
            lpList = NextItem(lpList);
        }
    }
    if hRet >= 0 {
        let mut ulDummy: ULONG = 0;
        ulSize = 0;

        /* Write a terminating list entry with zero size */
        hRet = IStream_Write(
            lpStream,
            core::ptr::addr_of_mut!(ulSize) as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::addr_of_mut!(ulDummy),
        );
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHReadDataBlockList(lpStream: *mut IStream, lppList: *mut LPDBLIST) -> HRESULT {
    let mut bBuff: [DATABLOCK_HEADER; 128] = core::mem::zeroed();
    let mut ulBuffSize: ULONG = core::mem::size_of::<[DATABLOCK_HEADER; 128]>() as ULONG;
    let mut pItem: LPDATABLOCK_HEADER = core::ptr::addr_of_mut!(bBuff) as LPDATABLOCK_HEADER;
    let mut ulRead: ULONG = 0;
    let mut ulSize: ULONG = 0;
    let mut hRet: HRESULT = S_OK;

    if !(*lppList).is_null() {
        /* Free any existing list */
        LocalFree(*lppList as HLOCAL);
        *lppList = core::ptr::null_mut();
    }

    loop {
        /* Read the size of the next item */
        hRet = IStream_Read(
            lpStream,
            core::ptr::addr_of_mut!(ulSize) as *mut core::ffi::c_void,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::addr_of_mut!(ulRead),
        );

        if hRet < 0
            || ulRead as usize != core::mem::size_of::<ULONG>() as usize
            || ulSize == 0
        {
            break; /* Read failed or read zero size (the end of the list) */
        }

        if ulSize > 0xFFFF {
            let mut liZero: LARGE_INTEGER = core::mem::zeroed();
            let mut ulPos: ULARGE_INTEGER = core::mem::zeroed();

            liZero.QuadPart = 0 as LONGLONG;

            /* Back the stream up; this object is too big for the list */
            if IStream_Seek(
                lpStream,
                liZero,
                STREAM_SEEK_CUR as DWORD,
                core::ptr::addr_of_mut!(ulPos),
            ) >= 0
            {
                liZero.QuadPart = (ulPos.QuadPart as usize)
                    .wrapping_sub(core::mem::size_of::<ULONG>() as usize) as LONGLONG;
                IStream_Seek(
                    lpStream,
                    liZero,
                    STREAM_SEEK_SET as DWORD,
                    core::ptr::null_mut(),
                );
            }
            break;
        } else if ulSize as usize >= core::mem::size_of::<DATABLOCK_HEADER>() as usize {
            /* Add this new item to the list */
            if ulSize > ulBuffSize {
                /* We need more buffer space, allocate it */
                let mut lpTemp: LPDATABLOCK_HEADER = core::ptr::null_mut();

                if pItem == core::ptr::addr_of_mut!(bBuff) as LPDATABLOCK_HEADER {
                    lpTemp = LocalAlloc(
                        LMEM_ZEROINIT as UINT,
                        ulSize as SIZE_T,
                    ) as LPDATABLOCK_HEADER;
                } else {
                    lpTemp = LocalReAlloc(
                        pItem as HLOCAL,
                        ulSize as SIZE_T,
                        (LMEM_ZEROINIT | LMEM_MOVEABLE) as UINT,
                    ) as LPDATABLOCK_HEADER;
                }

                if lpTemp.is_null() {
                    hRet = E_OUTOFMEMORY;
                    break;
                }
                ulBuffSize = ulSize;
                pItem = lpTemp;
            }

            (*pItem).cbSize = ulSize as DWORD;
            ulSize -= core::mem::size_of::<DWORD>() as ULONG; /* already read this member */

            /* Read the item Id and data */
            hRet = IStream_Read(
                lpStream,
                core::ptr::addr_of_mut!((*pItem).dwSignature) as *mut core::ffi::c_void,
                ulSize,
                core::ptr::addr_of_mut!(ulRead),
            );

            if hRet < 0 || ulRead != ulSize {
                break;
            }

            SHAddDataBlock(lppList, pItem as *const DATABLOCK_HEADER); /* Insert Item */
        }

        if !(1 != 0) {
            break;
        }
    }

    /* If we allocated space, free it */
    if pItem != core::ptr::addr_of_mut!(bBuff) as LPDATABLOCK_HEADER {
        LocalFree(pItem as HLOCAL);
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHFreeDataBlockList(lpList: LPDBLIST) {
    if !lpList.is_null() {
        LocalFree(lpList as HLOCAL);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHRemoveDataBlock(lppList: *mut LPDBLIST, dwSignature: DWORD) -> BOOL {
    let mut lpList: LPDATABLOCK_HEADER = core::ptr::null_mut();
    let mut lpItem: LPDATABLOCK_HEADER = core::ptr::null_mut();
    let mut lpNext: LPDATABLOCK_HEADER;
    let mut ulNewSize: ULONG;

    if !lppList.is_null() && {
        lpList = *lppList as LPDATABLOCK_HEADER;
        !lpList.is_null()
    } {
        /* Search for item in list */
        while (*lpList).cbSize != 0 {
            if (*lpList).dwSignature == dwSignature
                || ((*lpList).dwSignature == CLIST_ID_CONTAINER as DWORD
                    && (*lpList.offset(1 as isize)).dwSignature == dwSignature)
            {
                lpItem = lpList; /* Found */
                break;
            }
            lpList = NextItem(lpList as LPDBLIST) as LPDATABLOCK_HEADER;
        }
    }

    if lpItem.is_null() {
        return FALSE as BOOL;
    }

    lpNext = NextItem(lpItem as LPDBLIST) as LPDATABLOCK_HEADER;
    lpList = lpNext;

    /* Locate the end of the list */
    while (*lpList).cbSize != 0 {
        lpList = NextItem(lpList as LPDBLIST) as LPDATABLOCK_HEADER;
    }

    /* Resize the list */
    ulNewSize = LocalSize(*lppList as HLOCAL).wrapping_sub((*lpItem).cbSize as SIZE_T) as ULONG;

    /* Copy following elements over lpItem */
    memmove(
        lpItem as *mut core::ffi::c_void,
        lpNext as *const core::ffi::c_void,
        (((lpList as *mut core::ffi::c_char).offset_from(lpNext as *mut core::ffi::c_char)
            as core::ffi::c_longlong as SIZE_T)
            .wrapping_add(core::mem::size_of::<ULONG>() as SIZE_T)) as _,
    );

    if ulNewSize as usize <= core::mem::size_of::<ULONG>() as usize {
        LocalFree(*lppList as HLOCAL);
        *lppList = core::ptr::null_mut(); /* Removed the last element */
    } else {
        lpList = LocalReAlloc(
            *lppList as HLOCAL,
            ulNewSize as SIZE_T,
            (LMEM_ZEROINIT | LMEM_MOVEABLE) as UINT,
        ) as LPDATABLOCK_HEADER;
        if !lpList.is_null() {
            *lppList = lpList as LPDBLIST;
        }
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHFindDataBlock(
    mut lpList: LPDBLIST,
    mut dwSignature: DWORD,
) -> *mut DATABLOCK_HEADER {
    if !lpList.is_null() {
        while (*lpList).cbSize != 0 {
            if (*lpList).dwSignature == dwSignature {
                return lpList as *mut DATABLOCK_HEADER; /* Matched */
            } else if (*lpList).dwSignature == CLIST_ID_CONTAINER as DWORD
                && (*lpList.offset(1 as isize)).dwSignature == dwSignature
            {
                return lpList.offset(1 as isize) as *mut DATABLOCK_HEADER; /* Contained item matches */
            }

            lpList = NextItem(lpList) as LPDBLIST;
        }
    }
    return ::core::ptr::null_mut::<DATABLOCK_HEADER>();
}
