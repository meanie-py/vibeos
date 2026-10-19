use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _TABLE_SLOT {
    pub pString: LPWSTR,
    pub pData: LPVOID,
    pub dwSize: DWORD,
}
pub type TABLE_SLOT = _TABLE_SLOT;
pub type PTABLE_SLOT = *mut _TABLE_SLOT;

#[repr(C)]
pub struct _STRING_TABLE {
    pub pSlots: PTABLE_SLOT,
    pub dwUsedSlots: DWORD,
    pub dwMaxSlots: DWORD,
    pub dwMaxDataSize: DWORD,
}
pub type STRING_TABLE = _STRING_TABLE;
pub type PSTRING_TABLE = *mut _STRING_TABLE;

pub const TABLE_DEFAULT_SIZE: usize = 256;

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableInitialize() -> HSTRING_TABLE {
    let mut pStringTable: PSTRING_TABLE;

    pStringTable = MyMalloc(core::mem::size_of::<STRING_TABLE>() as DWORD) as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return core::ptr::null_mut();
    }

    memset(pStringTable as *mut core::ffi::c_void, 0, core::mem::size_of::<STRING_TABLE>() as usize);

    (*pStringTable).pSlots = MyMalloc(core::mem::size_of::<TABLE_SLOT>().wrapping_mul(TABLE_DEFAULT_SIZE as usize) as DWORD) as PTABLE_SLOT;
    if (*pStringTable).pSlots.is_null() {
        MyFree(pStringTable as PVOID);
        return core::ptr::null_mut();
    }

    memset((*pStringTable).pSlots as *mut core::ffi::c_void, 0, core::mem::size_of::<TABLE_SLOT>().wrapping_mul(TABLE_DEFAULT_SIZE as usize));

    (*pStringTable).dwUsedSlots = 0;
    (*pStringTable).dwMaxSlots = (TABLE_DEFAULT_SIZE as u32);
    (*pStringTable).dwMaxDataSize = 0;

    return pStringTable as HSTRING_TABLE;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableInitializeEx(dwMaxExtraDataSize: DWORD, dwReserved: DWORD) -> HSTRING_TABLE {
    let mut pStringTable: PSTRING_TABLE;

    pStringTable = MyMalloc(core::mem::size_of::<STRING_TABLE>() as DWORD) as PSTRING_TABLE;
    if pStringTable.is_null() {
        return core::ptr::null_mut();
    }

    memset(pStringTable as *mut core::ffi::c_void, 0 as i32, core::mem::size_of::<STRING_TABLE>() as usize);

    (*pStringTable).pSlots = MyMalloc((core::mem::size_of::<TABLE_SLOT>() * (TABLE_DEFAULT_SIZE as usize)) as DWORD) as PTABLE_SLOT;
    if (*pStringTable).pSlots.is_null() {
        MyFree(pStringTable as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    memset((*pStringTable).pSlots as *mut core::ffi::c_void, 0 as i32, core::mem::size_of::<TABLE_SLOT>() * (TABLE_DEFAULT_SIZE as usize));

    (*pStringTable).dwUsedSlots = 0 as DWORD;
    (*pStringTable).dwMaxSlots = TABLE_DEFAULT_SIZE as DWORD;
    (*pStringTable).dwMaxDataSize = dwMaxExtraDataSize;

    return pStringTable as HSTRING_TABLE;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableDestroy(hStringTable: HSTRING_TABLE) {
    let mut pStringTable: PSTRING_TABLE;
    let mut i: DWORD;

    // TRACE("%p\n", hStringTable);
    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        return;
    }

    if !(*pStringTable).pSlots.is_null() {
        i = 0;
        while i < (*pStringTable).dwMaxSlots {
            MyFree((*(*pStringTable).pSlots.add(i as usize)).pString as PVOID);
            (*(*pStringTable).pSlots.add(i as usize)).pString = core::ptr::null_mut();

            MyFree((*(*pStringTable).pSlots.add(i as usize)).pData as PVOID);
            (*(*pStringTable).pSlots.add(i as usize)).pData = core::ptr::null_mut();
            (*(*pStringTable).pSlots.add(i as usize)).dwSize = 0;
            i = i.wrapping_add(1);
        }

        MyFree((*pStringTable).pSlots as PVOID);
    }

    MyFree(pStringTable as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableAddString(
    hStringTable: HSTRING_TABLE,
    lpString: LPWSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut pStringTable: PSTRING_TABLE;
    let mut i: DWORD;

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return (0 as DWORD).wrapping_sub(1 as DWORD);
    }

    /* Search for existing string in the string table */
    i = 0 as DWORD;
    while i < (*pStringTable).dwMaxSlots {
        if !(*(*pStringTable).pSlots.offset(i as isize)).pString.is_null() {
            if (dwFlags & (1 as DWORD)) != 0 {
                if lstrcmpW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as LPCWSTR,
                    lpString as LPCWSTR,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            } else {
                if lstrcmpiW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as LPCWSTR,
                    lpString as LPCWSTR,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    /* Check for filled slot table */
    if (*pStringTable).dwUsedSlots == (*pStringTable).dwMaxSlots {
        let mut pNewSlots: PTABLE_SLOT;
        let mut dwNewMaxSlots: DWORD;

        /* FIXME: not thread safe */
        dwNewMaxSlots = (*pStringTable).dwMaxSlots.wrapping_mul(2 as DWORD);
        pNewSlots = MyMalloc(
            (core::mem::size_of::<TABLE_SLOT>() as usize).wrapping_mul(dwNewMaxSlots as usize)
                as DWORD,
        ) as PTABLE_SLOT;
        if pNewSlots.is_null() {
            return (0 as DWORD).wrapping_sub(1 as DWORD);
        }
        memset(
            pNewSlots.offset((*pStringTable).dwMaxSlots as isize) as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (core::mem::size_of::<TABLE_SLOT>() as usize)
                .wrapping_mul(dwNewMaxSlots.wrapping_sub((*pStringTable).dwMaxSlots) as usize),
        );
        memcpy(
            pNewSlots as *mut ::core::ffi::c_void,
            (*pStringTable).pSlots as *const ::core::ffi::c_void,
            (core::mem::size_of::<TABLE_SLOT>() as usize)
                .wrapping_mul((*pStringTable).dwMaxSlots as usize),
        );
        pNewSlots = InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*pStringTable).pSlots) as *mut PVOID,
            pNewSlots as PVOID,
        ) as PTABLE_SLOT;
        MyFree(pNewSlots as PVOID);
        (*pStringTable).dwMaxSlots = dwNewMaxSlots;

        return pSetupStringTableAddString(hStringTable, lpString, dwFlags);
    }

    /* Search for an empty slot */
    i = 0 as DWORD;
    while i < (*pStringTable).dwMaxSlots {
        if (*(*pStringTable).pSlots.offset(i as isize)).pString.is_null() {
            (*(*pStringTable).pSlots.offset(i as isize)).pString = MyMalloc(
                ((lstrlenW(lpString as LPCWSTR) + 1 as i32)
                    as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as DWORD,
            ) as LPWSTR;
            if (*(*pStringTable).pSlots.offset(i as isize)).pString.is_null() {
                TRACE!("Couldn't allocate memory for a new string!\n");
                return (0 as DWORD).wrapping_sub(1 as DWORD);
            }

            lstrcpyW(
                (*(*pStringTable).pSlots.offset(i as isize)).pString,
                lpString as LPCWSTR,
            );

            (*pStringTable).dwUsedSlots = (*pStringTable).dwUsedSlots.wrapping_add(1);

            return i.wrapping_add(1 as DWORD);
        }
        i = i.wrapping_add(1);
    }

    TRACE!("Couldn't find an empty slot!\n");

    return (0 as DWORD).wrapping_sub(1 as DWORD);
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableAddStringEx(
    hStringTable: HSTRING_TABLE,
    lpString: LPWSTR,
    dwFlags: DWORD,
    lpExtraData: LPVOID,
    dwExtraDataSize: DWORD,
) -> DWORD {
    let mut pStringTable: PSTRING_TABLE;
    let mut i: DWORD = 0;

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return (-(1 as i32)) as DWORD;
    }

    /* Search for existing string in the string table */
    i = 0 as DWORD;
    while i < (*pStringTable).dwMaxSlots {
        if !(*(*pStringTable).pSlots.add(i as usize)).pString.is_null() {
            if dwFlags & (1 as DWORD) != 0 {
                if lstrcmpW(
                    (*(*pStringTable).pSlots.add(i as usize)).pString as *const u16,
                    lpString as *const u16,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            } else {
                if lstrcmpiW(
                    (*(*pStringTable).pSlots.add(i as usize)).pString as *const u16,
                    lpString as *const u16,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    /* Check for filled slot table */
    if (*pStringTable).dwUsedSlots == (*pStringTable).dwMaxSlots {
        return (-(1 as i32)) as DWORD;
    }

    /* Search for an empty slot */
    i = 0 as DWORD;
    while i < (*pStringTable).dwMaxSlots {
        if (*(*pStringTable).pSlots.add(i as usize)).pString.is_null() {
            (*(*pStringTable).pSlots.add(i as usize)).pString = MyMalloc(
                (((lstrlenW(lpString as *const u16) + 1 as i32)
                    as usize)
                    * core::mem::size_of::<u16>()) as DWORD,
            ) as LPWSTR;
            if (*(*pStringTable).pSlots.add(i as usize)).pString.is_null() {
                return (-(1 as i32)) as DWORD;
            }

            lstrcpyW(
                (*(*pStringTable).pSlots.add(i as usize)).pString,
                lpString as *const u16,
            );

            (*(*pStringTable).pSlots.add(i as usize)).pData =
                MyMalloc(dwExtraDataSize) as LPVOID;
            if (*(*pStringTable).pSlots.add(i as usize)).pData.is_null() {
                MyFree(
                    (*(*pStringTable).pSlots.add(i as usize)).pString
                        as *mut core::ffi::c_void,
                );
                (*(*pStringTable).pSlots.add(i as usize)).pString = core::ptr::null_mut();
                return (-(1 as i32)) as DWORD;
            }

            memcpy(
                (*(*pStringTable).pSlots.add(i as usize)).pData as *mut core::ffi::c_void,
                lpExtraData as *const core::ffi::c_void,
                dwExtraDataSize as usize,
            );
            (*(*pStringTable).pSlots.add(i as usize)).dwSize = dwExtraDataSize;

            (*pStringTable).dwUsedSlots = (*pStringTable).dwUsedSlots.wrapping_add(1);

            return i.wrapping_add(1 as DWORD);
        }
        i = i.wrapping_add(1);
    }

    return (-(1 as i32)) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableDuplicate(hStringTable: HSTRING_TABLE) -> HSTRING_TABLE {
    let mut pSourceTable: PSTRING_TABLE;
    let mut pDestinationTable: PSTRING_TABLE;
    let mut i: DWORD;
    let mut length: DWORD;

    pSourceTable = hStringTable as PSTRING_TABLE;
    if pSourceTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return core::ptr::null_mut();
    }

    pDestinationTable = MyMalloc(::core::mem::size_of::<STRING_TABLE>() as DWORD) as PSTRING_TABLE;
    if pDestinationTable.is_null() {
        ERR!("Could not allocate a new string table!\n");
        return core::ptr::null_mut();
    }

    memset(
        pDestinationTable as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        ::core::mem::size_of::<STRING_TABLE>() as usize,
    );

    (*pDestinationTable).pSlots = MyMalloc(
        (::core::mem::size_of::<TABLE_SLOT>() as DWORD).wrapping_mul((*pSourceTable).dwMaxSlots),
    ) as PTABLE_SLOT;
    if (*pDestinationTable).pSlots.is_null() {
        MyFree(pDestinationTable as PVOID);
        return core::ptr::null_mut();
    }

    memset(
        (*pDestinationTable).pSlots as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (::core::mem::size_of::<TABLE_SLOT>() as usize).wrapping_mul((*pSourceTable).dwMaxSlots as usize),
    );

    (*pDestinationTable).dwUsedSlots = 0;
    (*pDestinationTable).dwMaxSlots = (*pSourceTable).dwMaxSlots;

    i = 0;
    while i < (*pSourceTable).dwMaxSlots {
        if !(*(*pSourceTable).pSlots.offset(i as isize)).pString.is_null() {
            length = ((lstrlenW((*(*pSourceTable).pSlots.offset(i as isize)).pString as LPCWSTR)
                + 1 as ::core::ffi::c_int) as usize)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
            (*(*pDestinationTable).pSlots.offset(i as isize)).pString =
                MyMalloc(length) as LPWSTR;
            if !(*(*pDestinationTable).pSlots.offset(i as isize)).pString.is_null() {
                memcpy(
                    (*(*pDestinationTable).pSlots.offset(i as isize)).pString as *mut ::core::ffi::c_void,
                    (*(*pSourceTable).pSlots.offset(i as isize)).pString as *const ::core::ffi::c_void,
                    length as usize,
                );
                (*pDestinationTable).dwUsedSlots = (*pDestinationTable).dwUsedSlots.wrapping_add(1);
            }

            if !(*(*pSourceTable).pSlots.offset(i as isize)).pData.is_null() {
                length = (*(*pSourceTable).pSlots.offset(i as isize)).dwSize;
                (*(*pDestinationTable).pSlots.offset(i as isize)).pData =
                    MyMalloc(length) as LPVOID;
                if !(*(*pDestinationTable).pSlots.offset(i as isize)).pData.is_null() {
                    memcpy(
                        (*(*pDestinationTable).pSlots.offset(i as isize)).pData as *mut ::core::ffi::c_void,
                        (*(*pSourceTable).pSlots.offset(i as isize)).pData as *const ::core::ffi::c_void,
                        length as usize,
                    );
                    (*(*pDestinationTable).pSlots.offset(i as isize)).dwSize = length;
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return pDestinationTable as HSTRING_TABLE;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableGetExtraData(
    hStringTable: HSTRING_TABLE,
    dwId: DWORD,
    lpExtraData: LPVOID,
    dwExtraDataSize: DWORD,
) -> BOOL {
    let mut pStringTable: PSTRING_TABLE;

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return (FALSE) as BOOL;
    }

    if dwId == 0 || dwId > (*pStringTable).dwMaxSlots {
        ERR!("Invalid Slot id!\n");
        return (FALSE) as BOOL;
    }

    if (*(*pStringTable).pSlots.offset(dwId.wrapping_sub(1) as isize)).dwSize
        < dwExtraDataSize
    {
        ERR!("Data size is too large!\n");
        return (FALSE) as BOOL;
    }

    memcpy(
        lpExtraData as *mut core::ffi::c_void,
        (*(*pStringTable).pSlots.offset(dwId.wrapping_sub(1) as isize)).pData
            as *const core::ffi::c_void,
        dwExtraDataSize as usize,
    );

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableLookUpString(
    hStringTable: HSTRING_TABLE,
    lpString: LPWSTR,
    dwFlags: DWORD,
) -> DWORD {
    let mut pStringTable: PSTRING_TABLE;
    let mut i: DWORD;

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return (-(1 as i32)) as DWORD;
    }

    /* Search for existing string in the string table */
    i = 0 as DWORD;
    while i < (*pStringTable).dwMaxSlots {
        if !(*(*pStringTable).pSlots.offset(i as isize)).pString.is_null() {
            if (dwFlags & (1 as DWORD)) != 0 {
                if lstrcmpW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as LPCWSTR,
                    lpString as LPCWSTR,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            } else {
                if lstrcmpiW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as LPCWSTR,
                    lpString as LPCWSTR,
                ) == 0
                {
                    return i.wrapping_add(1 as DWORD);
                }
            }
        }
        i = i.wrapping_add(1);
    }

    return (-(1 as i32)) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableLookUpStringEx(
    hStringTable: HSTRING_TABLE,
    lpString: LPWSTR,
    dwFlags: DWORD,
    lpExtraData: LPVOID,
    dwReserved: DWORD,
) -> DWORD {
    let mut pStringTable: PSTRING_TABLE = core::ptr::null_mut();
    let mut i: DWORD = 0;

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return !(0);
    }

    /* Search for existing string in the string table */
    i = 0;
    while i < (*pStringTable).dwMaxSlots {
        if !(*(*pStringTable).pSlots.offset(i as isize)).pString.is_null() {
            if (dwFlags & 1) != 0 {
                if lstrcmpW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as *const u16,
                    lpString as *const u16,
                ) == 0
                {
                    if !lpExtraData.is_null() {
                        memcpy(
                            lpExtraData as *mut core::ffi::c_void,
                            (*(*pStringTable).pSlots.offset(i as isize)).pData
                                as *const core::ffi::c_void,
                            dwReserved as usize,
                        );
                    }
                    return i.wrapping_add(1);
                }
            } else {
                if lstrcmpiW(
                    (*(*pStringTable).pSlots.offset(i as isize)).pString as *const u16,
                    lpString as *const u16,
                ) == 0
                {
                    if !lpExtraData.is_null() {
                        memcpy(
                            lpExtraData as *mut core::ffi::c_void,
                            (*(*pStringTable).pSlots.offset(i as isize)).pData
                                as *const core::ffi::c_void,
                            dwReserved as usize,
                        );
                    }
                    return i.wrapping_add(1);
                }
            }
        }
        i = i.wrapping_add(1);
    }
    return !(0);
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableSetExtraData(
    hStringTable: HSTRING_TABLE,
    dwId: DWORD,
    lpExtraData: LPVOID,
    dwExtraDataSize: DWORD,
) -> BOOL {
    let mut pStringTable: PSTRING_TABLE = core::ptr::null_mut();

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return (FALSE) as BOOL;
    }

    if dwId == 0 || dwId > (*pStringTable).dwMaxSlots {
        ERR!("Invalid Slot id!\n");
        return (FALSE) as BOOL;
    }

    if (*pStringTable).dwMaxDataSize < dwExtraDataSize {
        ERR!("Data size is too large!\n");
        return (FALSE) as BOOL;
    }

    // Divergence: the C overwrites a previous SetExtraData's block without freeing it.
    let old = (*(*pStringTable).pSlots.offset(dwId.wrapping_sub(1) as isize)).pData;
    if !old.is_null() {
        MyFree(old);
    }
    (*(*pStringTable)
        .pSlots
        .offset(dwId.wrapping_sub(1) as isize))
    .pData = MyMalloc(dwExtraDataSize) as LPVOID;
    if (*(*pStringTable)
        .pSlots
        .offset(dwId.wrapping_sub(1) as isize))
    .pData
    .is_null()
    {
        ERR!("\n");
        return (FALSE) as BOOL;
    }

    memcpy(
        (*(*pStringTable)
            .pSlots
            .offset(dwId.wrapping_sub(1) as isize))
        .pData as *mut core::ffi::c_void,
        lpExtraData as *const core::ffi::c_void,
        dwExtraDataSize as usize,
    );
    (*(*pStringTable)
        .pSlots
        .offset(dwId.wrapping_sub(1) as isize))
    .dwSize = dwExtraDataSize;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableStringFromId(
    hStringTable: HSTRING_TABLE,
    dwId: DWORD,
) -> LPWSTR {
    let mut pStringTable: PSTRING_TABLE;
    static mut empty: [WCHAR; 1] = [0 as WCHAR];

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        return core::ptr::null_mut();
    }

    if dwId == 0 || dwId > (*pStringTable).dwMaxSlots {
        return core::ptr::addr_of_mut!(empty) as LPWSTR;
    }

    return (*(*pStringTable)
        .pSlots
        .offset(dwId.wrapping_sub(1) as isize))
    .pString;
}

#[no_mangle]
pub unsafe extern "system" fn pSetupStringTableStringFromIdEx(
    hStringTable: HSTRING_TABLE,
    dwId: DWORD,
    lpBuffer: LPWSTR,
    lpBufferLength: LPDWORD,
) -> BOOL {
    let mut pStringTable: PSTRING_TABLE;
    let mut dwLength: DWORD;
    let mut bResult: BOOL = (FALSE as i32);

    pStringTable = hStringTable as PSTRING_TABLE;
    if pStringTable.is_null() {
        ERR!("Invalid hStringTable!\n");
        *lpBufferLength = 0;
        return (FALSE) as BOOL;
    }

    if dwId == 0
        || dwId > (*pStringTable).dwMaxSlots
        || (*(*pStringTable)
            .pSlots
            .offset(dwId.wrapping_sub(1) as isize))
        .pString
        .is_null()
    {
        WARN!("Invalid string ID!\n");
        *lpBufferLength = 0;
        return (FALSE) as BOOL;
    }

    dwLength = (lstrlenW(
        (*(*pStringTable)
            .pSlots
            .offset(dwId.wrapping_sub(1) as isize))
        .pString as LPCWSTR,
    ) + 1) as DWORD;
    if dwLength <= *lpBufferLength {
        lstrcpyW(
            lpBuffer,
            (*(*pStringTable)
                .pSlots
                .offset(dwId.wrapping_sub(1) as isize))
            .pString as LPCWSTR,
        );
        bResult = (TRUE as i32);
    }

    *lpBufferLength = dwLength;

    return bResult;
}
