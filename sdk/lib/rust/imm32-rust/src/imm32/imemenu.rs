use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::ime::ImmUnlockImeDpi;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const IMEMENUINFO_MAPPING_NAME: [::core::ffi::c_ushort; 12] = unsafe {
    ::core::mem::transmute::<[u8; 24], [::core::ffi::c_ushort; 12]>(
        *b"I\0m\0m\0M\0e\0n\0u\0I\0n\0f\0o\0\0\0",
    )
};


pub const IMEMENUINFO_BUFFER_SIZE: u32 = 0x20000;

#[repr(C)]
pub struct tagIMEMENUINFO {
    pub dwVersion: DWORD,
    pub dwBufferSize: DWORD,
    pub dwFlags: DWORD,
    pub dwType: DWORD,
    pub dwParentOffset: ULONG_PTR,
    pub dwItemsOffset: ULONG_PTR,
    pub dwCount: DWORD,
    pub dwBitmapListOffset: ULONG_PTR,
    pub dwEndOffset: ULONG_PTR,
}
pub type IMEMENUINFO = tagIMEMENUINFO;
pub type PIMEMENUINFO = *mut tagIMEMENUINFO;

#[repr(C)]
pub struct tagBITMAPNODE {
    pub dwNext: ULONG_PTR,
    pub hbmpCached: HBITMAP,
    pub dibBitsPtr: ULONG_PTR,
    pub bmih: BITMAPINFOHEADER,
}
pub type BITMAPNODE = tagBITMAPNODE;
pub type PBITMAPNODE = *mut tagBITMAPNODE;

unsafe extern "C" fn Imm32ImeMenuAnsiToWide(
    mut pItemA: *const IMEMENUITEMINFOA,
    mut pItemW: PIMEMENUITEMINFOW,
    mut uCodePage: UINT,
    mut bBitmap: BOOL,
) -> BOOL {
    let mut ret: INT = 0;
    (*pItemW).cbSize = (*pItemA).cbSize;
    (*pItemW).fType = (*pItemA).fType;
    (*pItemW).fState = (*pItemA).fState;
    (*pItemW).wID = (*pItemA).wID;
    if bBitmap != 0 {
        (*pItemW).hbmpChecked = (*pItemA).hbmpChecked;
        (*pItemW).hbmpUnchecked = (*pItemA).hbmpUnchecked;
        (*pItemW).hbmpItem = (*pItemA).hbmpItem;
    }
    (*pItemW).dwItemData = (*pItemA).dwItemData;
    ret = MultiByteToWideChar(
        uCodePage,
        0,
        core::ptr::addr_of!((*pItemA).szString) as LPCSTR,
        -1,
        core::ptr::addr_of_mut!((*pItemW).szString) as LPWSTR,
        _countof!((*pItemW).szString) as ::core::ffi::c_int,
    ) as INT;
    (*pItemW).szString[(_countof!((*pItemW).szString) - 1) as usize] =
        UNICODE_NULL; // Avoid buffer overrun
    return (ret != 0) as ::core::ffi::c_int;
}

unsafe extern "C" fn Imm32ImeMenuWideToAnsi(
    pItemW: *const IMEMENUITEMINFOW,
    pItemA: PIMEMENUITEMINFOA,
    uCodePage: UINT,
) -> BOOL {
    let mut ret: INT;
    (*pItemA).cbSize = (*pItemW).cbSize;
    (*pItemA).fType = (*pItemW).fType;
    (*pItemA).fState = (*pItemW).fState;
    (*pItemA).wID = (*pItemW).wID;
    (*pItemA).hbmpChecked = (*pItemW).hbmpChecked;
    (*pItemA).hbmpUnchecked = (*pItemW).hbmpUnchecked;
    (*pItemA).dwItemData = (*pItemW).dwItemData;
    (*pItemA).hbmpItem = (*pItemW).hbmpItem;
    ret = WideCharToMultiByte(
        uCodePage,
        0 as DWORD,
        core::ptr::addr_of!((*pItemW).szString) as LPCWSTR,
        -(1 as INT),
        core::ptr::addr_of_mut!((*pItemA).szString) as LPSTR,
        _countof!((*pItemA).szString) as INT,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    ) as INT;
    (*pItemA).szString[((_countof!((*pItemA).szString) as usize)
        - (1 as usize))
        as usize] = ANSI_NULL as CHAR;
    return (ret != 0) as BOOL;
}

unsafe extern "C" fn Imm32WriteHBitmapToNode(mut hDC: HDC, mut hbmp: HBITMAP, mut pNode: PBITMAPNODE, mut pView: PIMEMENUINFO) -> PVOID {
    let mut pBmih: *mut BITMAPINFOHEADER = core::ptr::null_mut::<BITMAPINFOHEADER>();
    let mut nodeExtraSize: DWORD = 0;
    let mut dibBitsPtr: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut hbmpTemp: HBITMAP = core::ptr::null_mut::<HBITMAP__>();
    let mut hbmpOld: HGDIOBJ = core::ptr::null_mut::<core::ffi::c_void>();
    let mut totalNeeded: DWORD = 0;
    let mut scanlines: core::ffi::c_int = 0;

    if hbmp.is_null() {
        return core::ptr::null_mut();
    }

    pBmih = core::ptr::addr_of_mut!((*pNode).bmih);
    (*pBmih).biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*pBmih).biBitCount = 0 as WORD;
    if GetDIBits(
        hDC,
        hbmp,
        0 as UINT,
        0 as UINT,
        core::ptr::null_mut(),
        pBmih as LPBITMAPINFO,
        DIB_RGB_COLORS as UINT,
    ) == 0
    {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32WriteHBitmapToNode\0" as *const u8 as *const core::ffi::c_char,
                116 as core::ffi::c_int,
                b"Invalid hbmp: %p\n\0" as *const u8 as *const core::ffi::c_char,
                hbmp,
            );
        }
        return core::ptr::null_mut();
    }

    match (*pBmih).biBitCount as core::ffi::c_int {
        16 | 32 => {
            nodeExtraSize = (core::mem::size_of::<BITMAPINFOHEADER>() as DWORD).wrapping_add(
                (3 as DWORD).wrapping_mul(core::mem::size_of::<DWORD>() as DWORD),
            );
        }
        24 => {
            nodeExtraSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
        }
        _ => {
            nodeExtraSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            nodeExtraSize = nodeExtraSize.wrapping_add(
                ((1 << (*pBmih).biBitCount) as DWORD)
                    .wrapping_mul(core::mem::size_of::<RGBQUAD>() as DWORD),
            );
        }
    }

    totalNeeded = nodeExtraSize.wrapping_add((*pBmih).biSizeImage);
    if (pBmih as PBYTE)
        .offset(totalNeeded as isize)
        .offset(core::mem::size_of::<ULONG_PTR>() as isize)
        > (pView as PBYTE).offset((*pView).dwBufferSize as isize)
    {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32WriteHBitmapToNode\0" as *const u8 as *const core::ffi::c_char,
                137 as core::ffi::c_int,
                b"No more space\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }

    dibBitsPtr = (pBmih as PBYTE).offset(nodeExtraSize as isize) as PVOID;
    (*pNode).dibBitsPtr = dibBitsPtr as ULONG_PTR;

    hbmpTemp = CreateCompatibleBitmap(
        hDC,
        (*pBmih).biWidth as INT,
        labs((*pBmih).biHeight as core::ffi::c_long) as INT,
    );
    if hbmpTemp.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32WriteHBitmapToNode\0" as *const u8 as *const core::ffi::c_char,
                147 as core::ffi::c_int,
                b"Out of memory\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }

    hbmpOld = SelectObject(hDC, hbmpTemp as HGDIOBJ);
    scanlines = GetDIBits(
        hDC,
        hbmp,
        0 as UINT,
        labs((*pBmih).biHeight as core::ffi::c_long) as UINT,
        dibBitsPtr as LPVOID,
        pBmih as LPBITMAPINFO,
        DIB_RGB_COLORS as UINT,
    );
    SelectObject(hDC, hbmpOld);
    DeleteObject(hbmpTemp as HGDIOBJ);

    if scanlines == 0 {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32WriteHBitmapToNode\0" as *const u8 as *const core::ffi::c_char,
                159 as core::ffi::c_int,
                b"GetDIBits failed\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return core::ptr::null_mut();
    }

    return (pBmih as PBYTE)
        .offset(nodeExtraSize as isize)
        .offset((*pBmih).biSizeImage as isize) as PVOID;
}

unsafe extern "C" fn Imm32InitImeMenuView(
    pView: PIMEMENUINFO,
    dwFlags: DWORD,
    dwType: DWORD,
    lpImeParentMenu: PIMEMENUITEMINFO,
    lpImeMenuItems: PIMEMENUITEMINFO,
    dwSize: DWORD,
) -> BOOL {
    ZeroMemory!(
        pView as *mut core::ffi::c_void,
        core::mem::size_of::<tagIMEMENUINFO>(),
    );
    (*pView).dwVersion = 1 as DWORD;
    (*pView).dwBufferSize = IMEMENUINFO_BUFFER_SIZE as DWORD;
    (*pView).dwFlags = dwFlags;
    (*pView).dwType = dwType;
    (*pView).dwCount = (dwSize as usize).wrapping_div(core::mem::size_of::<IMEMENUITEMINFOW>() as usize) as DWORD;

    if !((dwSize == 0 as DWORD || lpImeMenuItems.is_null()) && lpImeParentMenu.is_null()) {
        // empty branch in C: no parents nor items
    }
    if !((dwSize == 0 as DWORD || lpImeMenuItems.is_null()) && lpImeParentMenu.is_null()) {
        if !lpImeParentMenu.is_null() {
            // There is parent
            let pParentSlot: PIMEMENUITEMINFOW =
                pView.offset(1 as isize) as PIMEMENUITEMINFOW;
            RtlCopyMemory!(
                pParentSlot as *mut core::ffi::c_void,
                lpImeParentMenu as *const core::ffi::c_void,
                core::mem::size_of::<IMEMENUITEMINFOW>() as usize,
            );
            // Clear parent bitmaps (unused)
            (*pParentSlot).hbmpChecked = core::ptr::null_mut();
            (*pParentSlot).hbmpUnchecked = core::ptr::null_mut();
            (*pParentSlot).hbmpItem = core::ptr::null_mut();
            // Offset (relative)
            (*pView).dwParentOffset = core::mem::size_of::<tagIMEMENUINFO>() as usize as ULONG_PTR;
            (*pView).dwItemsOffset = (core::mem::size_of::<tagIMEMENUINFO>() as usize)
                .wrapping_add(core::mem::size_of::<IMEMENUITEMINFOW>() as usize)
                as ULONG_PTR;
        } else {
            // No parent but some items
            (*pView).dwParentOffset = 0 as ULONG_PTR;
            (*pView).dwItemsOffset = core::mem::size_of::<tagIMEMENUINFO>() as usize as ULONG_PTR;
        }
    }

    // SECURITY: Validate dwCount (ReactOS only)
    return (((*pView).dwItemsOffset as usize).wrapping_add(
        ((*pView).dwCount as usize)
            .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>() as usize),
    ) <= (*pView).dwBufferSize as usize) as ::core::ffi::c_int;
}

unsafe extern "C" fn Imm32SerializeBitmap(mut pView: PIMEMENUINFO, mut hbmp: HBITMAP) -> PBITMAPNODE {
    let mut pListHead: PBITMAPNODE = core::ptr::null_mut::<tagBITMAPNODE>();
    let mut pNode: PBITMAPNODE = core::ptr::null_mut::<tagBITMAPNODE>();
    let mut pNewEnd: PVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut hwndDesktop: HWND = core::ptr::null_mut::<HWND__>();
    let mut hDC: HDC = core::ptr::null_mut::<HDC__>();
    let mut hMemDC: HDC = core::ptr::null_mut::<HDC__>();

    // Check bitmap caches
    pNode = (*pView).dwBitmapListOffset as PBITMAPNODE;
    while !pNode.is_null() {
        if (*pNode).hbmpCached == hbmp {
            return pNode;
        }
        pNode = (*pNode).dwNext as PBITMAPNODE;
    }

    // Boundary check
    pNode = (*pView).dwEndOffset as PBITMAPNODE;
    if pNode.is_null()
        || (pNode as PBYTE) < pView as PBYTE
        || (pNode as PBYTE) >= (pView as PBYTE).offset((*pView).dwBufferSize as isize)
        || (pNode as PBYTE).offset(core::mem::size_of::<BITMAPNODE>() as isize)
            > (pView as PBYTE).offset((*pView).dwBufferSize as isize)
    {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32SerializeBitmap\0" as *const u8 as *const core::ffi::c_char,
                237 as core::ffi::c_int,
                b"Insufficient or incorrect space\n\0" as *const u8
                    as *const core::ffi::c_char,
            );
        }
        return core::ptr::null_mut::<tagBITMAPNODE>();
    }

    hwndDesktop = GetDesktopWindow();
    hDC = GetDC(hwndDesktop);
    if hDC.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32SerializeBitmap\0" as *const u8 as *const core::ffi::c_char,
                245 as core::ffi::c_int,
                b"GetDC failed\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        return core::ptr::null_mut::<tagBITMAPNODE>();
    }

    hMemDC = CreateCompatibleDC(hDC);
    if hMemDC.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32SerializeBitmap\0" as *const u8 as *const core::ffi::c_char,
                252 as core::ffi::c_int,
                b"CreateCompatibleDC failed\n\0" as *const u8
                    as *const core::ffi::c_char,
            );
        }
        ReleaseDC(hwndDesktop, hDC);
        return core::ptr::null_mut::<tagBITMAPNODE>();
    }

    pListHead = (*pView).dwBitmapListOffset as PBITMAPNODE;
    (*pNode).dwNext = pListHead as ULONG_PTR;

    pNewEnd = Imm32WriteHBitmapToNode(hMemDC, hbmp, pNode, pView);

    DeleteDC(hMemDC);
    ReleaseDC(hwndDesktop, hDC);

    if pNewEnd.is_null() {
        if (*__wine_dbch___default).flags as core::ffi::c_int
            & (1 as core::ffi::c_int) << __WINE_DBCL_ERR as core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const core::ffi::c_char,
                b"Imm32SerializeBitmap\0" as *const u8 as *const core::ffi::c_char,
                266 as core::ffi::c_int,
                b"No more space\n\0" as *const u8 as *const core::ffi::c_char,
            );
        }
        (*pView).dwEndOffset = pNode as ULONG_PTR;
        (*pView).dwBitmapListOffset = (*pNode).dwNext;
        return core::ptr::null_mut::<tagBITMAPNODE>();
    }

    (*pNode).hbmpCached = hbmp;
    (*pNode).dwNext = pListHead as ULONG_PTR;
    (*pView).dwBitmapListOffset = pNode as ULONG_PTR;
    (*pView).dwEndOffset = pNewEnd as ULONG_PTR;

    return pNode;
}

unsafe extern "C" fn Imm32DeserializeBitmap(pNode: PBITMAPNODE) -> HBITMAP {
    if pNode.is_null() || (*pNode).dibBitsPtr == 0 {
        return core::ptr::null_mut();
    }

    if !(*pNode).hbmpCached.is_null() {
        return (*pNode).hbmpCached;
    }

    let hDC: HDC = GetDC(GetDesktopWindow());
    if hDC.is_null() {
        return core::ptr::null_mut();
    }

    let hCompatDC: HDC = CreateCompatibleDC(hDC);
    if hCompatDC.is_null() {
        ReleaseDC(GetDesktopWindow(), hDC);
        return core::ptr::null_mut();
    }

    let hbmp: HBITMAP = CreateDIBitmap(
        hCompatDC,
        core::ptr::addr_of_mut!((*pNode).bmih),
        CBM_INIT as DWORD,
        (*pNode).dibBitsPtr as PVOID,
        core::ptr::addr_of_mut!((*pNode).bmih) as PBITMAPINFO,
        DIB_RGB_COLORS as UINT,
    );
    (*pNode).hbmpCached = hbmp;

    DeleteDC(hCompatDC);
    ReleaseDC(GetDesktopWindow(), hDC);

    hbmp
}

unsafe extern "C" fn Imm32SerializeImeMenu(hIMC: HIMC, pView: PIMEMENUINFO) -> BOOL {
    let mut pParent: PIMEMENUITEMINFOW = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
    let mut pItems: PIMEMENUITEMINFOW = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
    let mut pTempBuf: PIMEMENUITEMINFOW = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
    let mut i: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut dwTempBufSize: DWORD = 0;
    let mut pNode: PBITMAPNODE = core::ptr::null_mut::<tagBITMAPNODE>();
    let mut ret: BOOL = (FALSE) as i32;

    if (*pView).dwVersion != 1 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"Imm32SerializeImeMenu\0" as *const u8 as *const ::core::ffi::c_char,
                321 as ::core::ffi::c_int,
                b"dwVersion mismatch: 0x%08lX\n\0" as *const u8
                    as *const ::core::ffi::c_char,
                (*pView).dwVersion,
            );
        }
        return (FALSE) as BOOL;
    }

    if (*pView).dwParentOffset != 0 {
        pParent = (pView as PBYTE).offset((*pView).dwParentOffset as isize) as PIMEMENUITEMINFOW;
        if (pParent as PBYTE) < pView as PBYTE
            || pParent as PBYTE >= (pView as PBYTE).offset((*pView).dwBufferSize as isize)
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/user/imm32/imemenu.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32SerializeImeMenu\0" as *const u8 as *const ::core::ffi::c_char,
                    331 as ::core::ffi::c_int,
                    b"Invalid dwParentOffset\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
        (*pView).dwParentOffset = pParent as ULONG_PTR;
    } else {
        pParent = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
    }

    if (*pView).dwItemsOffset != 0 {
        pItems = (pView as PBYTE).offset((*pView).dwItemsOffset as isize) as PIMEMENUITEMINFOW;
        if (pItems as PBYTE) < pView as PBYTE
            || pItems as PBYTE >= (pView as PBYTE).offset((*pView).dwBufferSize as isize)
        {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/user/imm32/imemenu.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32SerializeImeMenu\0" as *const u8 as *const ::core::ffi::c_char,
                    347 as ::core::ffi::c_int,
                    b"Invalid dwItemsOffset\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
        (*pView).dwItemsOffset = pItems as ULONG_PTR;
    } else {
        pItems = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
    }

    // Allocate items buffer
    if (*pView).dwCount > 0 {
        dwTempBufSize = ((*pView).dwCount as usize)
            .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>() as usize)
            as DWORD;
        pTempBuf = ImmLocalAlloc(0, dwTempBufSize) as PIMEMENUITEMINFOW;
        if pTempBuf.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"win32ss/user/imm32/imemenu.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"Imm32SerializeImeMenu\0" as *const u8 as *const ::core::ffi::c_char,
                    364 as ::core::ffi::c_int,
                    b"Out of memory\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            return (FALSE) as BOOL;
        }
    } else {
        pTempBuf = core::ptr::null_mut::<tagIMEMENUITEMINFOW>();
        dwTempBufSize = 0;
    }

    dwCount = ImmGetImeMenuItemsW(
        hIMC,
        (*pView).dwFlags,
        (*pView).dwType,
        pParent,
        pTempBuf,
        dwTempBufSize,
    );
    (*pView).dwCount = dwCount;

    'convert_back: {
        'free_and_cleanup: {
            if dwCount == 0 {
                if !pTempBuf.is_null() {
                    ImmLocalFree!(pTempBuf as PVOID);
                }
                ret = (TRUE) as i32;
                break 'convert_back;
            }

            if pTempBuf.is_null() {
                ret = (TRUE) as i32;
                break 'convert_back;
            }

            (*pView).dwBitmapListOffset = 0;

            {
                let mut pBufferEnd: PBYTE = (pView as PBYTE).offset((*pView).dwBufferSize as isize);
                let mut pItemsEnd: PBYTE = (pItems as PBYTE).offset(
                    (dwCount as usize)
                        .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>() as usize)
                        as isize,
                );
                if pItemsEnd > pBufferEnd {
                    ret = (FALSE) as i32;
                    break 'free_and_cleanup;
                }
                (*pView).dwEndOffset = pItemsEnd as ULONG_PTR;
            }

            // Serialize items
            while i < dwCount {
                let mut pSrc: PIMEMENUITEMINFOW = pTempBuf.offset(i as isize) as PIMEMENUITEMINFOW;
                let mut pDest: PIMEMENUITEMINFOW = pItems.offset(i as isize);
                *pDest = *pSrc;

                if !(*pSrc).hbmpChecked.is_null() {
                    pNode = Imm32SerializeBitmap(pView, (*pSrc).hbmpChecked);
                    if pNode.is_null() {
                        break 'free_and_cleanup;
                    }
                    (*pDest).hbmpChecked = pNode as ULONG_PTR as HBITMAP;
                } else {
                    (*pDest).hbmpChecked = core::ptr::null_mut::<HBITMAP__>();
                }

                if !(*pSrc).hbmpUnchecked.is_null() {
                    pNode = Imm32SerializeBitmap(pView, (*pSrc).hbmpUnchecked);
                    if pNode.is_null() {
                        break 'free_and_cleanup;
                    }
                    (*pDest).hbmpUnchecked = pNode as ULONG_PTR as HBITMAP;
                } else {
                    (*pDest).hbmpUnchecked = core::ptr::null_mut::<HBITMAP__>();
                }

                if !(*pSrc).hbmpItem.is_null() {
                    pNode = Imm32SerializeBitmap(pView, (*pSrc).hbmpItem);
                    if pNode.is_null() {
                        break 'free_and_cleanup;
                    }
                    (*pDest).hbmpItem = pNode as ULONG_PTR as HBITMAP;
                } else {
                    (*pDest).hbmpItem = core::ptr::null_mut::<HBITMAP__>();
                }

                i = i.wrapping_add(1);
            }

            ret = (TRUE) as i32;
        }

        // FreeAndCleanup:
        ImmLocalFree!(pTempBuf as PVOID);
    }

    // ConvertBack:
    // Convert dwItemsOffset to relative
    if (*pView).dwItemsOffset != 0 {
        (*pView).dwItemsOffset = (*pView).dwItemsOffset.wrapping_sub(pView as ULONG_PTR);
    }

    if (*pView).dwCount > 0 && !pItems.is_null() {
        i = 0;
        while i < (*pView).dwCount {
            let mut pItem: PIMEMENUITEMINFOW = pItems.offset(i as isize);
            if !(*pItem).hbmpChecked.is_null() {
                (*pItem).hbmpChecked = ((*pItem).hbmpChecked as PBYTE).offset_from(pView as PBYTE)
                    as ::core::ffi::c_longlong as HBITMAP;
            }
            if !(*pItem).hbmpUnchecked.is_null() {
                (*pItem).hbmpUnchecked = ((*pItem).hbmpUnchecked as PBYTE)
                    .offset_from(pView as PBYTE)
                    as ::core::ffi::c_longlong as HBITMAP;
            }
            if !(*pItem).hbmpItem.is_null() {
                (*pItem).hbmpItem = ((*pItem).hbmpItem as PBYTE).offset_from(pView as PBYTE)
                    as ::core::ffi::c_longlong as HBITMAP;
            }
            i = i.wrapping_add(1);
        }
    }

    // Convert dwBitmapListOffset to relative
    if (*pView).dwBitmapListOffset != 0 {
        let mut pCur: PBITMAPNODE = (*pView).dwBitmapListOffset as PBITMAPNODE;
        (*pView).dwBitmapListOffset = (pCur as ULONG_PTR).wrapping_sub(pView as ULONG_PTR);

        while !pCur.is_null() {
            let mut pNext: PBITMAPNODE = (*pCur).dwNext as PBITMAPNODE;

            if (*pCur).dibBitsPtr != 0 {
                (*pCur).dibBitsPtr = (*pCur).dibBitsPtr.wrapping_sub(pView as ULONG_PTR);
            }
            if (*pCur).dwNext != 0 {
                (*pCur).dwNext = (pNext as ULONG_PTR).wrapping_sub(pView as ULONG_PTR);
            }

            pCur = pNext;
        }
    }

    // Convert dwParentOffset to relative
    if (*pView).dwParentOffset != 0 && (*pView).dwParentOffset as PBYTE >= pView as PBYTE {
        (*pView).dwParentOffset = (*pView).dwParentOffset.wrapping_sub(pView as ULONG_PTR);
    }

    return ret;
}

unsafe extern "C" fn Imm32DeserializeImeMenu(
    mut pView: PIMEMENUINFO,
    mut lpImeMenuItems: PIMEMENUITEMINFOW,
    mut dwSize: DWORD,
) -> DWORD {
    let mut i: DWORD = 0;
    let mut dwCount: DWORD = 0;
    let mut pViewBase: PBYTE = pView as PBYTE;
    let mut pNode: PBITMAPNODE = core::ptr::null_mut();
    let mut pItemsBase: PIMEMENUITEMINFOW = core::ptr::null_mut();

    dwCount = (*pView).dwCount;

    if lpImeMenuItems.is_null() {
        return dwCount;
    }

    if dwCount == 0 {
        return 0;
    }

    // SECURITY: Validate dwSize (ReactOS only)
    dwCount = min(
        dwCount as usize,
        (dwSize as usize) / core::mem::size_of::<IMEMENUITEMINFOW>(),
    ) as DWORD;

    if (*pView).dwBitmapListOffset != 0 {
        let mut pCur: PBITMAPNODE =
            pViewBase.add((*pView).dwBitmapListOffset as usize) as PBITMAPNODE;
        (*pView).dwBitmapListOffset = pCur as ULONG_PTR;

        while !pCur.is_null() {
            let mut pNextCur: PBITMAPNODE = core::ptr::null_mut();

            (*pCur).hbmpCached = core::ptr::null_mut();

            // dibBitsPtr: relative to absolute
            if (*pCur).dibBitsPtr != 0 {
                (*pCur).dibBitsPtr = pViewBase.add((*pCur).dibBitsPtr as usize) as ULONG_PTR;
            }

            // dwNext: relative to absolute
            if (*pCur).dwNext != 0 {
                pNextCur = pViewBase.add((*pCur).dwNext as usize) as PBITMAPNODE;
                if (pNextCur as PBYTE) < pViewBase
                    || (pNextCur as PBYTE) >= pViewBase.add((*pView).dwBufferSize as usize)
                {
                    return 0;
                }
                (*pCur).dwNext = pNextCur as ULONG_PTR;
                pCur = pNextCur;
            } else {
                (*pCur).dwNext = 0;
                pCur = core::ptr::null_mut();
            }
        }
    }

    // dwItemsOffset: relative to absolute
    if (*pView).dwItemsOffset != 0 {
        pItemsBase = pViewBase.add((*pView).dwItemsOffset as usize) as PIMEMENUITEMINFOW;
        // Boundary check: base pointer must be inside the view
        if (pItemsBase as PBYTE) < pViewBase
            || (pItemsBase as PBYTE) >= pViewBase.add((*pView).dwBufferSize as usize)
        {
            return 0;
        }

        // Boundary check (ReactOS only): ensure the whole items array fits inside the view
        if dwCount > 0 {
            let offsetFromBase: SIZE_T =
                (pItemsBase as usize).wrapping_sub(pViewBase as usize) as SIZE_T;
            let available: SIZE_T =
                ((*pView).dwBufferSize as SIZE_T).wrapping_sub(offsetFromBase);
            let needed: SIZE_T = (dwCount as SIZE_T)
                .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>() as SIZE_T);

            if available < needed {
                return 0;
            }
        }

        (*pView).dwItemsOffset = pItemsBase as ULONG_PTR;
    } else {
        return 0;
    }

    // Bitmap offsets: relative to absolute
    i = 0;
    while i < dwCount {
        let pItem: PIMEMENUITEMINFOW = ((*pView).dwItemsOffset as PBYTE).add(
            (i as usize).wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>()),
        ) as PIMEMENUITEMINFOW;

        // hbmpChecked
        if !(*pItem).hbmpChecked.is_null() {
            let pN: PBITMAPNODE =
                pViewBase.add((*pItem).hbmpChecked as ULONG_PTR as usize) as PBITMAPNODE;
            if (pN as PBYTE) < pViewBase
                || (pN as PBYTE) >= pViewBase.add((*pView).dwBufferSize as usize)
            {
                return 0;
            }
            (*pItem).hbmpChecked = pN as ULONG_PTR as HBITMAP;
        }

        // hbmpUnchecked
        if !(*pItem).hbmpUnchecked.is_null() {
            let pN_0: PBITMAPNODE =
                pViewBase.add((*pItem).hbmpUnchecked as ULONG_PTR as usize) as PBITMAPNODE;
            if (pN_0 as PBYTE) < pViewBase
                || (pN_0 as PBYTE) >= pViewBase.add((*pView).dwBufferSize as usize)
            {
                return 0;
            }
            (*pItem).hbmpUnchecked = pN_0 as ULONG_PTR as HBITMAP;
        }

        // hbmpItem
        if !(*pItem).hbmpItem.is_null() {
            let pN_1: PBITMAPNODE =
                pViewBase.add((*pItem).hbmpItem as ULONG_PTR as usize) as PBITMAPNODE;
            if (pN_1 as PBYTE) < pViewBase
                || (pN_1 as PBYTE) >= pViewBase.add((*pView).dwBufferSize as usize)
            {
                return 0;
            }
            (*pItem).hbmpItem = pN_1 as ULONG_PTR as HBITMAP;
        }

        i = i.wrapping_add(1);
    }

    // De-serialize items
    i = 0;
    while i < dwCount {
        let pSrc: PIMEMENUITEMINFOW = ((*pView).dwItemsOffset as PBYTE).add(
            (i as usize).wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>()),
        ) as PIMEMENUITEMINFOW;
        let pDst: PIMEMENUITEMINFOW = lpImeMenuItems.add(i as usize) as PIMEMENUITEMINFOW;

        // Copy scalar fields (excluding HBITMAP fields)
        (*pDst).cbSize = (*pSrc).cbSize;
        (*pDst).fType = (*pSrc).fType;
        (*pDst).fState = (*pSrc).fState;
        (*pDst).wID = (*pSrc).wID;
        (*pDst).dwItemData = (*pSrc).dwItemData;

        // Copy szString
        StringCbCopyW(
            core::ptr::addr_of_mut!((*pDst).szString) as STRSAFE_LPWSTR,
            core::mem::size_of::<[WCHAR; 80]>() as size_t,
            core::ptr::addr_of!((*pSrc).szString) as *mut WCHAR as STRSAFE_LPCWSTR,
        );

        // De-serialize hbmpChecked
        pNode = (*pSrc).hbmpChecked as ULONG_PTR as PBITMAPNODE;
        (*pDst).hbmpChecked = Imm32DeserializeBitmap(pNode);

        // De-serialize hbmpUnchecked
        pNode = (*pSrc).hbmpUnchecked as ULONG_PTR as PBITMAPNODE;
        (*pDst).hbmpUnchecked = Imm32DeserializeBitmap(pNode);

        // De-serialize hbmpItem
        pNode = (*pSrc).hbmpItem as ULONG_PTR as PBITMAPNODE;
        (*pDst).hbmpItem = Imm32DeserializeBitmap(pNode);

        i = i.wrapping_add(1);
    }

    return dwCount;
}

unsafe extern "C" fn ImmGetImeMenuItemsInterProcess(
    hIMC: HIMC,
    dwFlags: DWORD,
    dwType: DWORD,
    lpImeParentMenu: PVOID,
    lpImeMenuItems: PVOID,
    dwSize: DWORD,
) -> DWORD {
    let mut hwndIme: HWND = core::ptr::null_mut();
    let mut hMapping: HANDLE = core::ptr::null_mut();
    let mut pView: PIMEMENUINFO = core::ptr::null_mut();
    let mut dwCount: DWORD = 0;

    'Cleanup: {
        // Get IME window
        hwndIme = NtUserQueryInputContext(hIMC, QIC_DEFAULTWINDOWIME) as HWND;
        if hwndIme.is_null() || IsWindow(hwndIme) == 0 {
            return 0;
        }

        RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

        // Create a file mapping
        hMapping = CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            core::ptr::null_mut(),
            PAGE_READWRITE,
            0,
            IMEMENUINFO_BUFFER_SIZE,
            (IMEMENUINFO_MAPPING_NAME).as_ptr(),
        );
        if hMapping.is_null() {
            break 'Cleanup;
        }

        // Map view of file mapping
        pView = MapViewOfFile(
            hMapping,
            FILE_MAP_READ | FILE_MAP_WRITE,
            0,
            0,
            0,
        ) as PIMEMENUINFO;
        if pView.is_null() {
            break 'Cleanup;
        }

        if Imm32InitImeMenuView(
            pView,
            dwFlags,
            dwType,
            lpImeParentMenu as PIMEMENUITEMINFO,
            lpImeMenuItems as PIMEMENUITEMINFO,
            dwSize,
        ) == 0
        {
            break 'Cleanup;
        }

        // WM_IME_SYSTEM:IMS_GETIMEMENU will invoke ImmPutImeMenuItemsIntoMappedFile to serialize
        if SendMessageW(
            hwndIme,
            WM_IME_SYSTEM,
            (IMS_GETIMEMENU) as WPARAM,
            hIMC as LPARAM,
        ) == 0
        {
            break 'Cleanup;
        }

        dwCount = (*pView).dwCount;

        if lpImeMenuItems.is_null() {
            break 'Cleanup;
        }

        // De-serialize IME menu
        dwCount = Imm32DeserializeImeMenu(pView, lpImeMenuItems as PIMEMENUITEMINFOW, dwSize);
    }

    // Cleanup:
    if !pView.is_null() {
        UnmapViewOfFile(pView as LPCVOID);
    }
    if !hMapping.is_null() {
        CloseHandle(hMapping);
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsImeDpi));

    return dwCount;
}

#[no_mangle]
pub unsafe extern "system" fn ImmPutImeMenuItemsIntoMappedFile(hIMC: HIMC) -> BOOL {
    let hMapping: HANDLE = OpenFileMappingW(
        FILE_MAP_ALL_ACCESS,
        (FALSE) as BOOL,
        (IMEMENUINFO_MAPPING_NAME).as_ptr(),
    );
    if hMapping.is_null() {
        return (FALSE) as BOOL;
    }

    let pView: PIMEMENUINFO = MapViewOfFile(
        hMapping,
        FILE_MAP_ALL_ACCESS,
        0,
        0,
        0,
    ) as PIMEMENUINFO;
    if pView.is_null() {
        CloseHandle(hMapping);
        return (FALSE) as BOOL;
    }

    let ret: BOOL = Imm32SerializeImeMenu(hIMC, pView);
    UnmapViewOfFile(pView as LPCVOID);
    CloseHandle(hMapping);
    return ret;
}

unsafe extern "C" fn ImmGetImeMenuItemsAW(mut hIMC: HIMC, mut dwFlags: DWORD, mut dwType: DWORD, mut lpImeParentMenu: PVOID, mut lpImeMenuItems: PVOID, mut dwSize: DWORD, mut bTargetIsAnsi: BOOL) -> DWORD {
    let mut ret: DWORD = 0;
    let mut iItem: DWORD;

    if hIMC.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                770 as ::core::ffi::c_int,
                b"!hIMC\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    /* Get input process ID */
    let mut dwProcessId: DWORD =
        NtUserQueryInputContext(hIMC, QIC_INPUTPROCESSID) as DWORD;
    if dwProcessId == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                778 as ::core::ffi::c_int,
                b"!dwProcessId\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    if dwProcessId != GetCurrentProcessId() {
        /* Cross process? */
        if bTargetIsAnsi != 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/user/imm32/imemenu.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                    786 as ::core::ffi::c_int,
                    b"ImmGetImeMenuItemsA cannot cross process boundary\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                );
            }
            return 0;
        }

        /* Transport the IME menu items, using file mapping */
        return ImmGetImeMenuItemsInterProcess(
            hIMC,
            dwFlags,
            dwType,
            lpImeParentMenu,
            lpImeMenuItems,
            dwSize,
        );
    }

    let mut pIC: PINPUTCONTEXT = ImmLockIMC(hIMC) as PINPUTCONTEXT;
    if pIC.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                798 as ::core::ffi::c_int,
                b"!pIC\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return 0;
    }

    /* Get input thread ID */
    let mut dwThreadId: DWORD =
        NtUserQueryInputContext(hIMC, QIC_INPUTTHREADID) as DWORD;
    if dwThreadId == 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_3: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_3,
                __dbch_3,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                806 as ::core::ffi::c_int,
                b"!dwThreadId\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        ImmUnlockIMC(hIMC);
        return 0;
    }

    /* Get IME interface */
    let mut hKL: HKL = GetKeyboardLayout(dwThreadId);
    let mut pImeDpi: PIMEDPI = ImmLockImeDpi(hKL);
    if pImeDpi.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_4: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_4,
                __dbch_4,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                816 as ::core::ffi::c_int,
                b"!pImeDpi\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        ImmUnlockIMC(hIMC);
        return 0;
    }

    /* ImeGetImeMenuItems is optional */
    if (*pImeDpi).ImeGetImeMenuItems.is_none() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_5: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_5: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_5,
                __dbch_5,
                b"win32ss/user/imm32/imemenu.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                824 as ::core::ffi::c_int,
                b"ImeGetImeMenuItems is not available (optional).\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        ImmUnlockImeDpi(pImeDpi);
        ImmUnlockIMC(hIMC);
        return 0;
    }

    /* Is the IME ANSI? */
    let mut bImcIsAnsi: BOOL = Imm32IsImcAnsi(hIMC);

    let mut ParentA: IMEMENUITEMINFOA = tagIMEMENUITEMINFOA {
        cbSize: 0,
        fType: 0,
        fState: 0,
        wID: 0,
        hbmpChecked: ::core::ptr::null_mut::<HBITMAP__>(),
        hbmpUnchecked: ::core::ptr::null_mut::<HBITMAP__>(),
        dwItemData: 0,
        szString: [0; 80],
        hbmpItem: ::core::ptr::null_mut::<HBITMAP__>(),
    };
    let mut pItemA: *mut IMEMENUITEMINFOA = ::core::ptr::null_mut::<IMEMENUITEMINFOA>();
    let mut ParentW: IMEMENUITEMINFOW = tagIMEMENUITEMINFOW {
        cbSize: 0,
        fType: 0,
        fState: 0,
        wID: 0,
        hbmpChecked: ::core::ptr::null_mut::<HBITMAP__>(),
        hbmpUnchecked: ::core::ptr::null_mut::<HBITMAP__>(),
        dwItemData: 0,
        szString: [0; 80],
        hbmpItem: ::core::ptr::null_mut::<HBITMAP__>(),
    };
    let mut pItemW: *mut IMEMENUITEMINFOW = ::core::ptr::null_mut::<IMEMENUITEMINFOW>();
    let mut pNewItems: PVOID = core::ptr::null_mut();
    let mut pNewParent: PVOID = core::ptr::null_mut();

    'quit: {
        /* Are text types (ANSI/Wide) different between IME and target? */
        if bImcIsAnsi != bTargetIsAnsi {
            let mut cbTotal: DWORD = 0;
            if bTargetIsAnsi != 0 {
                /* Convert the parent */
                if !lpImeParentMenu.is_null() {
                    Imm32ImeMenuAnsiToWide(
                        lpImeParentMenu as *const IMEMENUITEMINFOA,
                        core::ptr::addr_of_mut!(ParentW),
                        (*pImeDpi).uCodePage,
                        (TRUE) as BOOL,
                    );
                    pNewParent = core::ptr::addr_of_mut!(ParentW) as PVOID;
                }

                /* Allocate buffer for new items */
                if !lpImeMenuItems.is_null() {
                    cbTotal = ((dwSize as usize)
                        .wrapping_div(core::mem::size_of::<IMEMENUITEMINFOA>() as usize)
                        .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOW>() as usize))
                        as DWORD;
                    pNewItems = ImmLocalAlloc(0, cbTotal) as PVOID;
                    if pNewItems.is_null() {
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_6: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_6: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_6,
                                __dbch_6,
                                b"win32ss/user/imm32/imemenu.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                                857 as ::core::ffi::c_int,
                                b"!pNewItems\n\0" as *const u8 as *const ::core::ffi::c_char,
                            );
                        }
                        break 'quit;
                    }
                }
            } else {
                /* Convert the parent */
                if !lpImeParentMenu.is_null() {
                    Imm32ImeMenuWideToAnsi(
                        lpImeParentMenu as *const IMEMENUITEMINFOW,
                        core::ptr::addr_of_mut!(ParentA),
                        (*pImeDpi).uCodePage,
                    );
                    pNewParent = core::ptr::addr_of_mut!(ParentA) as PVOID;
                }

                /* Allocate buffer for new items */
                if !lpImeMenuItems.is_null() {
                    cbTotal = ((dwSize as usize)
                        .wrapping_div(core::mem::size_of::<IMEMENUITEMINFOW>() as usize)
                        .wrapping_mul(core::mem::size_of::<IMEMENUITEMINFOA>() as usize))
                        as DWORD;
                    pNewItems = ImmLocalAlloc(0, cbTotal) as PVOID;
                    if pNewItems.is_null() {
                        if (*__wine_dbch___default).flags as ::core::ffi::c_int
                            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                            != 0
                        {
                            let __dbch_7: *mut __wine_debug_channel = __wine_dbch___default;
                            let __dbcl_7: __wine_debug_class = __WINE_DBCL_ERR;
                            ros_dbg_log(
                                __dbcl_7,
                                __dbch_7,
                                b"win32ss/user/imm32/imemenu.c\0"
                                    as *const u8 as *const ::core::ffi::c_char,
                                b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                                878 as ::core::ffi::c_int,
                                b"!pNewItems\n\0" as *const u8 as *const ::core::ffi::c_char,
                            );
                        }
                        break 'quit;
                    }
                }
            }
        } else {
            /* Get the items directly */
            pNewItems = lpImeMenuItems;
            pNewParent = lpImeParentMenu;
        }

        /* Get IME menu items from the IME */
        ret = (*pImeDpi)
            .ImeGetImeMenuItems
            .unwrap_unchecked()(
            hIMC,
            dwFlags,
            dwType,
            pNewParent as LPIMEMENUITEMINFOW,
            pNewItems as LPIMEMENUITEMINFOW,
            dwSize,
        );
        if ret == 0 {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                != 0
            {
                let __dbch_8: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_8: __wine_debug_class = __WINE_DBCL_ERR;
                ros_dbg_log(
                    __dbcl_8,
                    __dbch_8,
                    b"win32ss/user/imm32/imemenu.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"ImmGetImeMenuItemsAW\0" as *const u8 as *const ::core::ffi::c_char,
                    895 as ::core::ffi::c_int,
                    b"ImeGetImeMenuItems failed\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            break 'quit;
        }

        if lpImeMenuItems.is_null() {
            break 'quit;
        }

        /* Are text types different? */
        if bImcIsAnsi != bTargetIsAnsi {
            if bTargetIsAnsi != 0 {
                /* Convert the parent */
                if !pNewParent.is_null() {
                    Imm32ImeMenuWideToAnsi(
                        pNewParent as *const IMEMENUITEMINFOW,
                        lpImeParentMenu as PIMEMENUITEMINFOA,
                        (*pImeDpi).uCodePage,
                    );
                }

                /* Convert the items */
                pItemW = pNewItems as *mut IMEMENUITEMINFOW;
                pItemA = lpImeMenuItems as *mut IMEMENUITEMINFOA;
                iItem = 0;
                while iItem < ret {
                    Imm32ImeMenuWideToAnsi(pItemW, pItemA, (*pImeDpi).uCodePage);
                    iItem = iItem.wrapping_add(1);
                    pItemW = pItemW.offset(1);
                    pItemA = pItemA.offset(1);
                }
            } else {
                /* Convert the parent */
                if !pNewParent.is_null() {
                    Imm32ImeMenuAnsiToWide(
                        pNewParent as *const IMEMENUITEMINFOA,
                        lpImeParentMenu as PIMEMENUITEMINFOW,
                        (*pImeDpi).uCodePage,
                        (TRUE) as BOOL,
                    );
                }

                /* Convert the items */
                pItemA = pNewItems as *mut IMEMENUITEMINFOA;
                pItemW = lpImeMenuItems as *mut IMEMENUITEMINFOW;
                iItem = 0;
                while iItem < dwSize {
                    Imm32ImeMenuAnsiToWide(
                        pItemA,
                        pItemW,
                        (*pImeDpi).uCodePage,
                        (TRUE) as BOOL,
                    );
                    iItem = iItem.wrapping_add(1);
                    pItemA = pItemA.offset(1);
                    pItemW = pItemW.offset(1);
                }
            }
        }
    }

    /* Quit: */
    if pNewItems != lpImeMenuItems {
        ImmLocalFree!(pNewItems);
    }
    ImmUnlockImeDpi(pImeDpi);
    ImmUnlockIMC(hIMC);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetImeMenuItemsA(hIMC: HIMC, dwFlags: DWORD, dwType: DWORD, lpImeParentMenu: PIMEMENUITEMINFOA, lpImeMenu: PIMEMENUITEMINFOA, dwSize: DWORD) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/imemenu.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmGetImeMenuItemsA\0" as *const u8 as *const ::core::ffi::c_char,
            955 as ::core::ffi::c_int,
            b"(%p, 0x%lX, 0x%lX, %p, %p, 0x%lX)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hIMC,
            dwFlags,
            dwType,
            lpImeParentMenu,
            lpImeMenu,
            dwSize,
        );
    }
    return ImmGetImeMenuItemsAW(
        hIMC,
        dwFlags,
        dwType,
        lpImeParentMenu as PVOID,
        lpImeMenu as PVOID,
        dwSize,
        (TRUE) as BOOL,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ImmGetImeMenuItemsW(mut hIMC: HIMC, mut dwFlags: DWORD, mut dwType: DWORD, mut lpImeParentMenu: PIMEMENUITEMINFOW, mut lpImeMenu: PIMEMENUITEMINFOW, mut dwSize: DWORD) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/imemenu.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImmGetImeMenuItemsW\0" as *const u8 as *const ::core::ffi::c_char,
            972 as ::core::ffi::c_int,
            b"(%p, 0x%lX, 0x%lX, %p, %p, 0x%lX)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hIMC,
            dwFlags,
            dwType,
            lpImeParentMenu,
            lpImeMenu,
            dwSize,
        );
    }
    return ImmGetImeMenuItemsAW(
        hIMC,
        dwFlags,
        dwType,
        lpImeParentMenu as PVOID,
        lpImeMenu as PVOID,
        dwSize,
        (FALSE) as BOOL,
    );
}
