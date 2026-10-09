use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_dpa);
// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct DPA {
    pub nItemCount: INT,
    pub ptrs: *mut LPVOID,
    pub hHeap: HANDLE,
    pub nGrow: INT,
    pub nMaxCount: INT,
}

pub type PDPA = *mut DPA;

#[repr(C)]
pub struct STREAMDATA {
    pub dwSize: DWORD,
    pub dwData2: DWORD,
    pub dwItems: DWORD,
}

pub type PSTREAMDATA = *mut STREAMDATA;

#[no_mangle]
pub unsafe extern "system" fn DPA_LoadStream(
    phDpa: *mut HDPA,
    loadProc: PFNDPASTREAM,
    pStream: *mut IStream,
    pData: LPVOID,
) -> HRESULT {
    let mut errCode: HRESULT = 0;
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut initial_pos: ULARGE_INTEGER = core::mem::zeroed();
    let mut streamData: STREAMDATA = core::mem::zeroed();
    let mut streamInfo: DPASTREAMINFO = core::mem::zeroed();
    let mut ulRead: ULONG = 0;
    let mut hDpa: HDPA = core::ptr::null_mut();
    let mut ptr: *mut PVOID = core::ptr::null_mut();

    if phDpa.is_null() || loadProc.is_none() || pStream.is_null() {
        return E_INVALIDARG;
    }

    *phDpa = core::ptr::null_mut();

    position.QuadPart = 0 as LONGLONG;

    errCode = IStream_Seek(
        pStream,
        position,
        STREAM_SEEK_CUR,
        core::ptr::addr_of_mut!(initial_pos),
    );
    if errCode != S_OK {
        return errCode;
    }

    memset(
        core::ptr::addr_of_mut!(streamData) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<STREAMDATA>() as size_t,
    );
    errCode = IStream_Read(
        pStream,
        core::ptr::addr_of_mut!(streamData) as *mut core::ffi::c_void,
        core::mem::size_of::<STREAMDATA>() as ULONG,
        core::ptr::addr_of_mut!(ulRead),
    );
    if errCode != S_OK {
        return errCode;
    }

    if (ulRead as usize) < core::mem::size_of::<STREAMDATA>()
        || (streamData.dwSize as usize) < core::mem::size_of::<STREAMDATA>()
        || streamData.dwData2 != 1
    {
        /* back to initial position */
        position.QuadPart = initial_pos.QuadPart as LONGLONG;
        IStream_Seek(
            pStream,
            position,
            STREAM_SEEK_SET,
            core::ptr::null_mut(),
        );
        return E_FAIL;
    }

    if (streamData.dwItems as usize)
        > ((UINT_MAX as usize / 2 as usize)
            / core::mem::size_of::<*mut VOID>())
    {
        return E_OUTOFMEMORY;
    }

    /* create the dpa */
    hDpa = DPA_Create(streamData.dwItems as INT);
    if hDpa.is_null() {
        return E_OUTOFMEMORY;
    }

    if DPA_Grow(hDpa, streamData.dwItems as INT) == 0 {
        DPA_Destroy(hDpa);
        return E_OUTOFMEMORY;
    }

    /* load data from the stream into the dpa */
    ptr = (*hDpa).ptrs as *mut PVOID;
    streamInfo.iPos = 0 as core::ffi::c_int;
    while (streamInfo.iPos as DWORD) < streamData.dwItems {
        errCode = loadProc.unwrap_unchecked()(
            core::ptr::addr_of_mut!(streamInfo),
            pStream as *mut IStream,
            pData as *mut core::ffi::c_void,
        );
        if errCode != S_OK {
            errCode = S_FALSE;
            break;
        }

        *ptr = streamInfo.pvItem as PVOID;
        ptr = ptr.offset(1);
        streamInfo.iPos += 1;
    }

    /* set the number of items */
    (*hDpa).nItemCount = streamInfo.iPos as INT;

    /* store the handle to the dpa */
    *phDpa = hDpa;

    return errCode;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_SaveStream(
    hDpa: HDPA,
    saveProc: PFNDPASTREAM,
    pStream: *mut IStream,
    pData: LPVOID,
) -> HRESULT {
    let mut position: LARGE_INTEGER = core::mem::zeroed();
    let mut initial_pos: ULARGE_INTEGER = core::mem::zeroed();
    let mut curr_pos: ULARGE_INTEGER = core::mem::zeroed();
    let mut streamData: STREAMDATA = core::mem::zeroed();
    let mut streamInfo: DPASTREAMINFO = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut ptr: *mut PVOID = core::ptr::null_mut();

    if hDpa.is_null() || saveProc.is_none() || pStream.is_null() {
        return E_INVALIDARG;
    }

    /* save initial position to write header after completion */
    position.QuadPart = 0;
    hr = IStream_Seek(
        pStream,
        position,
        STREAM_SEEK_CUR,
        core::ptr::addr_of_mut!(initial_pos),
    );
    if hr != S_OK {
        return hr;
    }

    /* write empty header */
    streamData.dwSize = core::mem::size_of::<STREAMDATA>() as DWORD;
    streamData.dwData2 = 1;
    streamData.dwItems = 0;

    hr = IStream_Write(
        pStream,
        core::ptr::addr_of_mut!(streamData) as *const core::ffi::c_void,
        core::mem::size_of::<STREAMDATA>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );
    if hr != S_OK {
        position.QuadPart = initial_pos.QuadPart as LONGLONG;
        IStream_Seek(
            pStream,
            position,
            STREAM_SEEK_SET,
            core::ptr::null_mut::<ULARGE_INTEGER>(),
        );
        return hr;
    }

    /* no items - we're done */
    if (*hDpa).nItemCount == 0 {
        return S_OK;
    }

    ptr = (*hDpa).ptrs as *mut PVOID;
    streamInfo.iPos = 0;
    while streamInfo.iPos < (*hDpa).nItemCount {
        streamInfo.pvItem = *ptr as *mut core::ffi::c_void;
        hr = saveProc.unwrap_unchecked()(
            core::ptr::addr_of_mut!(streamInfo),
            pStream,
            pData,
        );
        if hr != S_OK {
            hr = S_FALSE;
            break;
        }
        ptr = ptr.offset(1);
        streamInfo.iPos += 1;
    }

    /* write updated header */
    position.QuadPart = 0;
    IStream_Seek(
        pStream,
        position,
        STREAM_SEEK_CUR,
        core::ptr::addr_of_mut!(curr_pos),
    );

    streamData.dwSize = curr_pos.QuadPart.wrapping_sub(initial_pos.QuadPart) as DWORD;
    streamData.dwData2 = 1;
    streamData.dwItems = streamInfo.iPos as DWORD;

    position.QuadPart = initial_pos.QuadPart as LONGLONG;
    IStream_Seek(
        pStream,
        position,
        STREAM_SEEK_SET,
        core::ptr::null_mut::<ULARGE_INTEGER>(),
    );
    IStream_Write(
        pStream,
        core::ptr::addr_of_mut!(streamData) as *const core::ffi::c_void,
        core::mem::size_of::<STREAMDATA>() as ULONG,
        core::ptr::null_mut::<ULONG>(),
    );

    position.QuadPart = curr_pos.QuadPart as LONGLONG;
    IStream_Seek(
        pStream,
        position,
        STREAM_SEEK_SET,
        core::ptr::null_mut::<ULARGE_INTEGER>(),
    );

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Merge(
    mut hdpa1: HDPA,
    mut hdpa2: HDPA,
    mut dwFlags: DWORD,
    mut pfnCompare: PFNDPACOMPARE,
    mut pfnMerge: PFNDPAMERGE,
    mut lParam: LPARAM,
) -> BOOL {
    let mut nCount: INT = 0;
    let mut pWork1: *mut LPVOID = core::ptr::null_mut::<LPVOID>();
    let mut pWork2: *mut LPVOID = core::ptr::null_mut::<LPVOID>();
    let mut nResult: INT = 0;
    let mut i: INT = 0;
    let mut nIndex: INT = 0;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
            275 as ::core::ffi::c_int,
            b"(%p %p %08x %p %p %08lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdpa1,
            hdpa2,
            dwFlags,
            pfnCompare,
            pfnMerge,
            lParam,
        );
    }

    if IsBadWritePtr(hdpa1 as PVOID, core::mem::size_of::<DPA>() as UINT_PTR) != 0 {
        return FALSE;
    }

    if IsBadWritePtr(hdpa2 as PVOID, core::mem::size_of::<DPA>() as UINT_PTR) != 0 {
        return FALSE;
    }

    if IsBadCodePtr(::core::mem::transmute::<PFNDPACOMPARE, FARPROC>(pfnCompare)) != 0 {
        return FALSE;
    }

    if IsBadCodePtr(::core::mem::transmute::<PFNDPAMERGE, FARPROC>(pfnMerge)) != 0 {
        return FALSE;
    }

    if dwFlags & DPAM_SORTED as DWORD == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/dpa.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
                290 as ::core::ffi::c_int,
                b"sorting dpa's.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*hdpa1).nItemCount > 0 as INT {
            DPA_Sort(hdpa1, pfnCompare, lParam);
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/dpa.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
                293 as ::core::ffi::c_int,
                b"dpa 1 sorted.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        if (*hdpa2).nItemCount > 0 as INT {
            DPA_Sort(hdpa2, pfnCompare, lParam);
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/dpa.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
                296 as ::core::ffi::c_int,
                b"dpa 2 sorted.\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
    }

    if (*hdpa2).nItemCount < 1 as INT {
        return TRUE;
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_3,
            __dbch_3,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
            303 as ::core::ffi::c_int,
            b"hdpa1->nItemCount=%d hdpa2->nItemCount=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            (*hdpa1).nItemCount,
            (*hdpa2).nItemCount,
        );
    }

    nIndex = ((*hdpa1).nItemCount as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;
    nCount = ((*hdpa2).nItemCount as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as INT;

    loop {
        pWork1 = (*hdpa1).ptrs.offset(nIndex as isize) as *mut LPVOID;
        pWork2 = (*hdpa2).ptrs.offset(nCount as isize) as *mut LPVOID;

        if nIndex < 0 as INT {
            if nCount >= 0 as INT && dwFlags & DPAM_UNION as DWORD != 0 {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_4,
                        __dbch_4,
                        b"dll/win32/comctl32/dpa.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
                        318 as ::core::ffi::c_int,
                        b"%d items to be inserted at start of DPA 1\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        nCount as ::core::ffi::c_int + 1 as ::core::ffi::c_int,
                    );
                }
                i = nCount;
                while i >= 0 as INT {
                    let mut ptr: PVOID = core::ptr::null_mut::<::core::ffi::c_void>();
                    ptr = pfnMerge.unwrap_unchecked()(
                        DPAMM_INSERT as UINT,
                        *pWork2,
                        NULL,
                        lParam,
                    ) as PVOID;
                    if ptr.is_null() {
                        return FALSE;
                    }
                    DPA_InsertPtr(hdpa1, 0 as INT, ptr as LPVOID);
                    pWork2 = pWork2.offset(-1);
                    i -= 1;
                }
            }
            break;
        }

        nResult = pfnCompare.unwrap_unchecked()(*pWork1, *pWork2, lParam) as INT;

        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_5: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_5,
                __dbch_5,
                b"dll/win32/comctl32/dpa.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"DPA_Merge\0" as *const u8 as *const ::core::ffi::c_char,
                333 as ::core::ffi::c_int,
                b"compare result=%d, dpa1.cnt=%d, dpa2.cnt=%d\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                nResult,
                nIndex,
                nCount,
            );
        }

        if nResult == 0 as INT {
            let mut ptr_0: PVOID = core::ptr::null_mut::<::core::ffi::c_void>();
            ptr_0 = pfnMerge.unwrap_unchecked()(
                DPAMM_MERGE as UINT,
                *pWork1,
                *pWork2,
                lParam,
            ) as PVOID;
            if ptr_0.is_null() {
                return FALSE;
            }
            nCount -= 1;
            *pWork1 = ptr_0 as LPVOID;
            nIndex -= 1;
        } else if nResult > 0 as INT {
            if dwFlags & DPAM_INTERSECT as DWORD != 0 {
                let mut ptr_1: PVOID = core::ptr::null_mut::<::core::ffi::c_void>();
                ptr_1 = DPA_DeletePtr(hdpa1, nIndex);
                pfnMerge.unwrap_unchecked()(
                    DPAMM_DELETE as UINT,
                    ptr_1 as *mut ::core::ffi::c_void,
                    NULL,
                    lParam,
                );
            }
            nIndex -= 1;
        } else {
            if dwFlags & DPAM_UNION as DWORD != 0 {
                let mut ptr_2: PVOID = core::ptr::null_mut::<::core::ffi::c_void>();
                ptr_2 = pfnMerge.unwrap_unchecked()(
                    DPAMM_INSERT as UINT,
                    *pWork2,
                    NULL,
                    lParam,
                ) as PVOID;
                if ptr_2.is_null() {
                    return FALSE;
                }
                DPA_InsertPtr(hdpa1, nIndex + 1 as INT, ptr_2 as LPVOID);
            }
            nCount -= 1;
        }

        if !(nCount >= 0 as INT) {
            break;
        }
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Destroy(hdpa: HDPA) -> BOOL {
    // TRACE("(%p)\n", hdpa);

    if hdpa.is_null() {
        return FALSE;
    }

    if !(*hdpa).ptrs.is_null() && HeapFree((*hdpa).hHeap, 0, (*hdpa).ptrs as LPVOID) == 0 {
        return FALSE;
    }

    return HeapFree((*hdpa).hHeap, 0, hdpa as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Grow(hdpa: HDPA, mut nGrow: INT) -> BOOL {
    let mut items: INT;

    if hdpa.is_null() {
        return FALSE;
    }

    nGrow = max(8, nGrow);
    items = nGrow * (((*hdpa).nMaxCount - 1) / nGrow + 1);
    if items > (*hdpa).nMaxCount {
        let mut ptr: LPVOID;

        if !(*hdpa).ptrs.is_null() {
            ptr = HeapReAlloc(
                (*hdpa).hHeap,
                HEAP_ZERO_MEMORY as DWORD,
                (*hdpa).ptrs as LPVOID,
                (items as SIZE_T).wrapping_mul(core::mem::size_of::<LPVOID>() as SIZE_T),
            ) as LPVOID;
        } else {
            ptr = HeapAlloc(
                (*hdpa).hHeap,
                HEAP_ZERO_MEMORY as DWORD,
                (items as SIZE_T).wrapping_mul(core::mem::size_of::<LPVOID>() as SIZE_T),
            ) as LPVOID;
        }
        if ptr.is_null() {
            return FALSE;
        }
        (*hdpa).nMaxCount = items;
        (*hdpa).ptrs = ptr as *mut LPVOID;
    }
    (*hdpa).nGrow = nGrow;

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Clone(hdpa: HDPA, mut hdpaNew: HDPA) -> HDPA {
    let mut nNewItems: INT = 0;
    let mut nSize: INT = 0;
    let mut hdpaTemp: HDPA = core::ptr::null_mut::<_DPA>();
    if hdpa.is_null() {
        return core::ptr::null_mut::<_DPA>();
    }
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"DPA_Clone\0" as *const u8 as *const core::ffi::c_char,
            478 as core::ffi::c_int,
            b"(%p %p)\n\0" as *const u8 as *const core::ffi::c_char,
            hdpa,
            hdpaNew,
        );
    }
    if hdpaNew.is_null() {
        hdpaTemp = HeapAlloc(
            (*hdpa).hHeap,
            HEAP_ZERO_MEMORY as DWORD,
            core::mem::size_of::<_DPA>() as SIZE_T,
        ) as HDPA;
        (*hdpaTemp).hHeap = (*hdpa).hHeap;
        (*hdpaTemp).nGrow = (*hdpa).nGrow;
    } else {
        hdpaTemp = hdpaNew;
    }
    if !(*hdpaTemp).ptrs.is_null() {
        HeapFree(
            (*hdpaTemp).hHeap,
            0 as DWORD,
            (*hdpaTemp).ptrs as PVOID,
        );
        (*hdpaTemp).ptrs = core::ptr::null_mut::<LPVOID>();
        (*hdpaTemp).nItemCount = 0 as INT;
        (*hdpaTemp).nMaxCount = 0 as INT;
    }
    nNewItems = ((*hdpaTemp).nGrow as core::ffi::c_int
        * (((*hdpa).nItemCount as core::ffi::c_int - 1 as core::ffi::c_int)
            / (*hdpaTemp).nGrow as core::ffi::c_int
            + 1 as core::ffi::c_int)) as INT;
    nSize = (nNewItems as usize).wrapping_mul(core::mem::size_of::<LPVOID>() as usize) as INT;
    (*hdpaTemp).ptrs = HeapAlloc(
        (*hdpaTemp).hHeap,
        HEAP_ZERO_MEMORY as DWORD,
        nSize as SIZE_T,
    ) as *mut LPVOID;
    (*hdpaTemp).nMaxCount = nNewItems;
    (*hdpaTemp).nItemCount = (*hdpa).nItemCount;
    memmove(
        (*hdpaTemp).ptrs as *mut core::ffi::c_void,
        (*hdpa).ptrs as *const core::ffi::c_void,
        ((*hdpaTemp).nItemCount as usize)
            .wrapping_mul(core::mem::size_of::<LPVOID>() as usize),
    );
    return hdpaTemp;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_GetPtr(hdpa: HDPA, nIndex: INT) -> LPVOID {
    if hdpa.is_null() {
        return core::ptr::null_mut();
    }
    if (*hdpa).ptrs.is_null() {
        return core::ptr::null_mut();
    }
    if (nIndex < 0) || (nIndex >= (*hdpa).nItemCount) {
        return core::ptr::null_mut();
    }

    return *(*hdpa).ptrs.offset(nIndex as isize);
}

#[no_mangle]
pub unsafe extern "system" fn DPA_GetPtrIndex(hdpa: HDPA, p: LPCVOID) -> INT {
    let mut i: INT;

    if hdpa.is_null() || (*hdpa).ptrs.is_null() {
        return -1;
    }

    i = 0;
    while i < (*hdpa).nItemCount {
        if *(*hdpa).ptrs.offset(i as isize) == p as LPVOID {
            return i;
        }
        i += 1;
    }

    return -1;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_InsertPtr(hdpa: HDPA, mut i: INT, p: LPVOID) -> INT {
    if hdpa.is_null() || i < 0 {
        return -1;
    }

    /* append item if index is out of bounds */
    i = min((*hdpa).nItemCount, i);

    /* create empty spot at the end */
    if DPA_SetPtr(hdpa, (*hdpa).nItemCount, 0 as LPVOID) == 0 {
        return -1;
    }

    if i != (*hdpa).nItemCount - 1 {
        memmove(
            (*hdpa).ptrs.offset((i + 1) as isize) as *mut core::ffi::c_void,
            (*hdpa).ptrs.offset(i as isize) as *const core::ffi::c_void,
            (((*hdpa).nItemCount - i - 1) as usize)
                .wrapping_mul(core::mem::size_of::<LPVOID>()),
        );
    }

    *(*hdpa).ptrs.offset(i as isize) = p;
    return i;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_SetPtr(hdpa: HDPA, i: INT, p: LPVOID) -> BOOL {
    let mut lpTemp: *mut LPVOID = core::ptr::null_mut();

    if hdpa.is_null() || i < 0 {
        return FALSE;
    }

    if (*hdpa).nItemCount <= i {
        /* within the old array */
        if (*hdpa).nMaxCount <= i {
            /* resize the block of memory */
            let mut nNewItems: INT = (*hdpa).nGrow
                * ((((i + 1) - 1) / (*hdpa).nGrow)
                    + 1);
            let mut nSize: INT = nNewItems
                * (core::mem::size_of::<LPVOID>() as INT);

            if !(*hdpa).ptrs.is_null() {
                lpTemp = HeapReAlloc(
                    (*hdpa).hHeap,
                    HEAP_ZERO_MEMORY as DWORD,
                    (*hdpa).ptrs as LPVOID,
                    nSize as SIZE_T,
                ) as *mut LPVOID;
            } else {
                lpTemp = HeapAlloc(
                    (*hdpa).hHeap,
                    HEAP_ZERO_MEMORY as DWORD,
                    nSize as SIZE_T,
                ) as *mut LPVOID;
            }

            if lpTemp.is_null() {
                return FALSE;
            }

            (*hdpa).nMaxCount = nNewItems;
            (*hdpa).ptrs = lpTemp;
        }
        (*hdpa).nItemCount = i + 1;
    }

    /* put the new entry in */
    *((*hdpa).ptrs.offset(i as isize)) = p;

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_DeletePtr(hdpa: HDPA, i: INT) -> LPVOID {
    let mut lpDest: *mut LPVOID = core::ptr::null_mut();
    let mut lpSrc: *mut LPVOID = core::ptr::null_mut();
    let mut lpTemp: LPVOID = core::ptr::null_mut();
    let mut nSize: INT = 0;

    if hdpa.is_null() || i < 0 || i >= (*hdpa).nItemCount {
        return core::ptr::null_mut();
    }

    lpTemp = *(*hdpa).ptrs.offset(i as isize);

    /* do we need to move ?*/
    if i < (*hdpa).nItemCount - 1 {
        lpDest = (*hdpa).ptrs.offset(i as isize);
        lpSrc = lpDest.offset(1 as isize);
        nSize = (((*hdpa).nItemCount - i - 1) as usize)
            .wrapping_mul(core::mem::size_of::<LPVOID>() as usize) as INT;
        memmove(
            lpDest as *mut core::ffi::c_void,
            lpSrc as *const core::ffi::c_void,
            nSize as usize,
        );
    }

    (*hdpa).nItemCount -= 1;

    /* free memory ?*/
    if ((*hdpa).nMaxCount - (*hdpa).nItemCount) >= (*hdpa).nGrow {
        let nNewItems: INT = max((*hdpa).nGrow * 2, (*hdpa).nItemCount);
        nSize = (nNewItems as usize).wrapping_mul(core::mem::size_of::<LPVOID>() as usize) as INT;
        lpDest = HeapReAlloc(
            (*hdpa).hHeap,
            HEAP_ZERO_MEMORY,
            (*hdpa).ptrs as LPVOID,
            nSize as SIZE_T,
        ) as *mut LPVOID;
        if lpDest.is_null() {
            return core::ptr::null_mut();
        }

        (*hdpa).nMaxCount = nNewItems;
        (*hdpa).ptrs = lpDest;
    }

    lpTemp
}

#[no_mangle]
pub unsafe extern "system" fn DPA_DeleteAllPtrs(hdpa: HDPA) -> BOOL {
    if hdpa.is_null() {
        return FALSE;
    }

    if !(*hdpa).ptrs.is_null() && HeapFree((*hdpa).hHeap, 0, (*hdpa).ptrs as LPVOID) == 0 {
        return FALSE;
    }

    (*hdpa).nItemCount = 0;
    (*hdpa).nMaxCount = (*hdpa).nGrow * 2;
    (*hdpa).ptrs = HeapAlloc(
        (*hdpa).hHeap,
        HEAP_ZERO_MEMORY,
        ((*hdpa).nMaxCount * (core::mem::size_of::<LPVOID>() as INT)) as SIZE_T,
    ) as *mut LPVOID;

    return TRUE;
}

unsafe extern "C" fn DPA_QuickSort(
    mut lpPtrs: *mut LPVOID,
    mut l: INT,
    mut r: INT,
    mut pfnCompare: PFNDPACOMPARE,
    mut lParam: LPARAM,
) {
    let mut m: INT = 0;
    let mut t: LPVOID = core::ptr::null_mut();

    if l == r {
        return;
    }
    if r < l {
        DPA_QuickSort(lpPtrs, r, l, pfnCompare, lParam);
        return;
    }
    m = (l + r) / 2;
    DPA_QuickSort(lpPtrs, l, m, pfnCompare, lParam);
    DPA_QuickSort(lpPtrs, m + 1, r, pfnCompare, lParam);

    /* join the two sides */
    while l <= m && m < r {
        if pfnCompare.unwrap_unchecked()(
            *lpPtrs.offset(l as isize) as *mut core::ffi::c_void,
            *lpPtrs.offset((m + 1) as isize) as *mut core::ffi::c_void,
            lParam,
        ) > 0
        {
            t = *lpPtrs.offset((m + 1) as isize);
            memmove(
                lpPtrs.offset((l + 1) as isize) as *mut LPVOID
                    as *mut core::ffi::c_void,
                lpPtrs.offset(l as isize) as *mut LPVOID as *const core::ffi::c_void,
                ((m - l + 1) as usize)
                    .wrapping_mul(core::mem::size_of::<LPVOID>()),
            );
            *lpPtrs.offset(l as isize) = t;

            m += 1;
        }
        l += 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Sort(
    hdpa: HDPA,
    pfnCompare: PFNDPACOMPARE,
    lParam: LPARAM,
) -> BOOL {
    if hdpa.is_null() || pfnCompare.is_none() {
        return FALSE;
    }
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DPA_Sort\0" as *const u8 as *const ::core::ffi::c_char,
            818 as ::core::ffi::c_int,
            b"(%p %p 0x%lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdpa,
            pfnCompare,
            lParam,
        );
    }
    if (*hdpa).nItemCount > 1 && !(*hdpa).ptrs.is_null() {
        DPA_QuickSort(
            (*hdpa).ptrs,
            0,
            (*hdpa).nItemCount - 1,
            pfnCompare,
            lParam,
        );
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Search(
    hdpa: HDPA,
    pFind: LPVOID,
    nStart: INT,
    pfnCompare: PFNDPACOMPARE,
    lParam: LPARAM,
    uOptions: UINT,
) -> INT {
    if hdpa.is_null() || pfnCompare.is_none() || pFind.is_null() {
        return -1;
    }

    if (uOptions & DPAS_SORTED as UINT) != 0 {
        /* array is sorted --> use binary search */
        let mut l: INT;
        let mut r: INT;
        let mut x: INT;
        let mut n: INT;
        let mut lpPtr: *mut LPVOID;

        /* for binary search ignore start index */
        l = 0;
        r = (*hdpa).nItemCount - 1;
        lpPtr = (*hdpa).ptrs;
        while r >= l {
            x = (l + r) / 2;
            n = pfnCompare.unwrap_unchecked()(
                pFind,
                *lpPtr.offset(x as isize),
                lParam,
            );
            if n == 0 {
                return x;
            } else if n < 0 {
                r = x - 1;
            } else {
                /* (n > 0) */
                l = x + 1;
            }
        }
        if (uOptions & ((DPAS_INSERTBEFORE | DPAS_INSERTAFTER) as UINT)) != 0 {
            return l;
        }
    } else {
        /* array is not sorted --> use linear search */
        let mut lpPtr: *mut LPVOID;
        let mut nIndex: INT;

        nIndex = if nStart == -1 {
            0
        } else {
            nStart
        };
        lpPtr = (*hdpa).ptrs;
        while nIndex < (*hdpa).nItemCount {
            if pfnCompare.unwrap_unchecked()(
                pFind,
                *lpPtr.offset(nIndex as isize),
                lParam,
            ) == 0
            {
                return nIndex;
            }
            nIndex += 1;
        }
    }

    return -1;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_CreateEx(mut nGrow: INT, mut hHeap: HANDLE) -> HDPA {
    let mut hdpa: HDPA = core::ptr::null_mut();

    if !hHeap.is_null() {
        hdpa = HeapAlloc(hHeap, HEAP_ZERO_MEMORY, core::mem::size_of::<DPA>() as SIZE_T) as HDPA;
    } else {
        hdpa = Alloc(core::mem::size_of::<DPA>() as DWORD) as HDPA;
    }

    if !hdpa.is_null() {
        (*hdpa).nGrow = max(8, nGrow);
        (*hdpa).hHeap = if !hHeap.is_null() { hHeap } else { GetProcessHeap() };
        (*hdpa).nMaxCount = (*hdpa).nGrow * 2;
        (*hdpa).ptrs = HeapAlloc(
            (*hdpa).hHeap,
            HEAP_ZERO_MEMORY,
            ((*hdpa).nMaxCount as SIZE_T).wrapping_mul(core::mem::size_of::<LPVOID>() as SIZE_T),
        ) as *mut LPVOID;
    }

    return hdpa;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_Create(nGrow: INT) -> HDPA {
    return DPA_CreateEx(nGrow, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn DPA_EnumCallback(hdpa: HDPA, enumProc: PFNDPAENUMCALLBACK, lParam: LPVOID) {
    let mut i: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DPA_EnumCallback\0" as *const u8 as *const ::core::ffi::c_char,
            974 as ::core::ffi::c_int,
            b"(%p %p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdpa,
            enumProc,
            lParam,
        );
    }
    if hdpa.is_null() {
        return;
    }
    if (*hdpa).nItemCount <= 0 {
        return;
    }
    i = 0 as INT;
    while i < (*hdpa).nItemCount {
        if enumProc.unwrap_unchecked()(
            *(*hdpa).ptrs.offset(i as isize) as *mut ::core::ffi::c_void,
            lParam as *mut ::core::ffi::c_void,
        ) == 0
        {
            return;
        }
        i += 1;
    }
    return;
}

#[no_mangle]
pub unsafe extern "system" fn DPA_DestroyCallback(
    hdpa: HDPA,
    enumProc: PFNDPAENUMCALLBACK,
    lParam: LPVOID,
) {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0".as_ptr() as *const i8,
            b"DPA_DestroyCallback\0".as_ptr() as *const i8,
            1006,
            b"(%p %p %p)\n\0".as_ptr() as *const i8,
            hdpa,
            enumProc,
            lParam,
        );
    }
    DPA_EnumCallback(hdpa, enumProc, lParam);
    DPA_Destroy(hdpa);
}
#[no_mangle]
pub unsafe extern "system" fn DPA_GetSize(mut hdpa: HDPA) -> ULONGLONG {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/dpa.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DPA_GetSize\0" as *const u8 as *const ::core::ffi::c_char,
            1025 as ::core::ffi::c_int,
            b"(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdpa,
        );
    }
    if hdpa.is_null() {
        return 0 as ULONGLONG;
    }
    return (::core::mem::size_of::<DPA>() as ULONGLONG).wrapping_add(
        ((*hdpa).nMaxCount as ULONGLONG)
            .wrapping_mul(::core::mem::size_of::<PVOID>() as ULONGLONG),
    );
}
