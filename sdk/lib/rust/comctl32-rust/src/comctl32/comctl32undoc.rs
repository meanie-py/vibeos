use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_commctrl);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut strMRUList: [WCHAR; 8] = [
    'M' as i32 as WCHAR,
    'R' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'L' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

// #define COBJMACROS (C preprocessor only; no Rust declaration)
// #define NONAMELESSUNION (C preprocessor only; no Rust declaration)
// WINE_DEFAULT_DEBUG_CHANNEL(commctrl); (debug-channel macro invocation; no direct Rust equivalent)

pub type MRUStringCmpFnA = unsafe extern "system" fn(lhs: LPCSTR, rhs: LPCSTR) -> INT;
pub type MRUStringCmpFnW = unsafe extern "system" fn(lhs: LPCWSTR, rhs: LPCWSTR) -> INT;
pub type MRUBinaryCmpFn = unsafe extern "system" fn(lhs: LPCVOID, rhs: LPCVOID, length: DWORD) -> INT;

#[repr(C)]
pub union MRUINFOA_U {
    pub string_cmpfn: MRUStringCmpFnA,
    pub binary_cmpfn: MRUBinaryCmpFn,
}

#[repr(C)]
pub struct tagMRUINFOA {
    pub cbSize: DWORD,
    pub uMax: UINT,
    pub fFlags: UINT,
    pub hKey: HKEY,
    pub lpszSubKey: LPSTR,
    pub u: MRUINFOA_U,
}
pub type MRUINFOA = tagMRUINFOA;
pub type LPMRUINFOA = *mut MRUINFOA;

#[repr(C)]
pub union MRUINFOW_U {
    pub string_cmpfn: MRUStringCmpFnW,
    pub binary_cmpfn: MRUBinaryCmpFn,
}

#[repr(C)]
pub struct tagMRUINFOW {
    pub cbSize: DWORD,
    pub uMax: UINT,
    pub fFlags: UINT,
    pub hKey: HKEY,
    pub lpszSubKey: LPWSTR,
    pub u: MRUINFOW_U,
}
pub type MRUINFOW = tagMRUINFOW;
pub type LPMRUINFOW = *mut MRUINFOW;

pub const MRU_STRING: i32 = 0; // list will contain strings
pub const MRU_BINARY: i32 = 1; // list will contain binary data
pub const MRU_CACHEWRITE: i32 = 2; // only save list order to reg. is FreeMRUList

#[repr(C)]
pub struct tagWINEMRUITEM {
    pub size: DWORD,        // size of data stored
    pub itemFlag: DWORD,    // flags
    pub datastart: BYTE,
}
pub type WINEMRUITEM = tagWINEMRUITEM;
pub type LPWINEMRUITEM = *mut WINEMRUITEM;

pub const WMRUIF_CHANGED: i32 = 0x0001; // this dataitem changed

#[repr(C)]
pub struct tagWINEMRULIST {
    pub extview: MRUINFOW,          // original create information
    pub isUnicode: BOOL,            // is compare fn Unicode
    pub wineFlags: DWORD,           // internal flags
    pub cursize: DWORD,             // current size of realMRU
    pub realMRU: LPWSTR,            // pointer to string of index names
    pub array: *mut LPWINEMRUITEM,  // array of pointers to data
                                    // in 'a' to 'z' order
}
pub type WINEMRULIST = tagWINEMRULIST;
pub type LPWINEMRULIST = *mut WINEMRULIST;

pub const WMRUF_CHANGED: i32 = 0x0001; // MRU list has changed

#[repr(C)]
pub struct tagNOTIFYDATA {
    pub hwndFrom: HWND,
    pub hwndTo: HWND,
    pub dwParam3: DWORD,
    pub dwParam4: DWORD,
    pub dwParam5: DWORD,
    pub dwParam6: DWORD,
}
pub type NOTIFYDATA = tagNOTIFYDATA;
pub type LPNOTIFYDATA = *mut NOTIFYDATA;

#[no_mangle]
pub unsafe extern "system" fn Alloc(dwSize: DWORD) -> LPVOID {
    return LocalAlloc(LMEM_ZEROINIT as UINT, dwSize as SIZE_T) as LPVOID;
}

#[no_mangle]
pub unsafe extern "system" fn ReAlloc(lpSrc: LPVOID, dwSize: DWORD) -> LPVOID {
    if !lpSrc.is_null() {
        return LocalReAlloc(
            lpSrc as HLOCAL,
            dwSize as SIZE_T,
            (LMEM_ZEROINIT | LMEM_MOVEABLE) as UINT,
        ) as LPVOID;
    } else {
        return LocalAlloc(LMEM_ZEROINIT as UINT, dwSize as SIZE_T) as LPVOID;
    };
}

#[no_mangle]
pub unsafe extern "system" fn Free(lpMem: LPVOID) -> BOOL {
    return LocalFree(lpMem).is_null() as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetSize(lpMem: LPVOID) -> DWORD {
    (LocalSize(lpMem)) as u32
}

unsafe extern "C" fn MRU_SaveChanged(mp: LPWINEMRULIST) {
    let mut i: UINT = 0;
    let mut err: UINT = 0;
    let mut newkey: HKEY = core::ptr::null_mut();
    let mut realname: [WCHAR; 2] = [0; 2];
    let mut witem: LPWINEMRUITEM = core::ptr::null_mut();

    /* open the sub key */
    err = RegOpenKeyExW(
        (*mp).extview.hKey,
        (*mp).extview.lpszSubKey as LPCWSTR,
        0 as DWORD,
        KEY_WRITE as REGSAM,
        core::ptr::addr_of_mut!(newkey),
    ) as UINT;
    if err != 0 {
        /* not present - what to do ??? */
        err = RegCreateKeyExW(
            (*mp).extview.hKey,
            (*mp).extview.lpszSubKey as LPCWSTR,
            0 as DWORD,
            core::ptr::null_mut::<WCHAR>(),
            REG_OPTION_NON_VOLATILE as DWORD,
            (KEY_READ | KEY_WRITE) as REGSAM,
            0 as *mut _SECURITY_ATTRIBUTES,
            core::ptr::addr_of_mut!(newkey),
            0 as *mut DWORD,
        ) as UINT;
        if err != 0 {
            return;
        }
    }
    if ((*mp).wineFlags & (WMRUF_CHANGED as DWORD)) != 0 {
        (*mp).wineFlags &= !(WMRUF_CHANGED as DWORD);
        err = RegSetValueExW(
            newkey,
            core::ptr::addr_of!(strMRUList) as LPCWSTR,
            0 as DWORD,
            REG_SZ as DWORD,
            (*mp).realMRU as LPBYTE,
            ((lstrlenW((*mp).realMRU as LPCWSTR) + 1 as i32) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD,
        ) as UINT;
        if err != 0 {
            /* error saving MRUList */
        }
    }
    realname[1 as usize] = 0 as WCHAR;
    i = 0 as UINT;
    while i < (*mp).cursize {
        witem = *(*mp).array.offset(i as isize);
        if ((*witem).itemFlag & (WMRUIF_CHANGED as DWORD)) != 0 {
            (*witem).itemFlag &= !(WMRUIF_CHANGED as DWORD);
            realname[0 as usize] = ('a' as UINT).wrapping_add(i) as WCHAR;
            err = RegSetValueExW(
                newkey,
                realname.as_ptr() as LPCWSTR,
                0 as DWORD,
                (if ((*mp).extview.fFlags & (MRU_BINARY as UINT)) != 0 {
                    REG_BINARY
                } else {
                    REG_SZ
                }) as DWORD,
                core::ptr::addr_of_mut!((*witem).datastart),
                (*witem).size,
            ) as UINT;
            if err != 0 {
                /* error saving value */
            }
        }
        i = i.wrapping_add(1);
    }
    RegCloseKey(newkey);
}

#[no_mangle]
pub unsafe extern "system" fn FreeMRUList(hMRUList: HANDLE) {
    let mut mp: LPWINEMRULIST = hMRUList as LPWINEMRULIST;
    let mut i: UINT = 0;
    if hMRUList.is_null() {
        return;
    }
    if (*mp).wineFlags & WMRUF_CHANGED as DWORD != 0 {
        /* need to open key and then save the info */
        MRU_SaveChanged(mp);
    }
    i = 0 as UINT;
    while i < (*mp).extview.uMax {
        Free(*(*mp).array.offset(i as isize) as LPVOID);
        i = i.wrapping_add(1);
    }
    Free((*mp).realMRU as LPVOID);
    Free((*mp).array as LPVOID);
    Free((*mp).extview.lpszSubKey as LPVOID);
    Free(mp as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn FindMRUData(
    mut hList: HANDLE,
    mut lpData: LPCVOID,
    mut cbData: DWORD,
    mut lpRegNum: LPINT,
) -> INT {
    let mut mp: *const WINEMRULIST = hList as *const WINEMRULIST;
    let mut ret: INT = 0;
    let mut i: UINT = 0 as UINT;
    let mut dataA: LPSTR = ::core::ptr::null_mut::<CHAR>();
    if mp.is_null() || (*mp).extview.u.string_cmpfn as usize == 0 {
        return -1 as INT;
    }
    if (*mp).extview.fFlags & MRU_BINARY as UINT == 0 && (*mp).isUnicode == 0 {
        let mut len: DWORD = WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            lpData as LPCWSTR,
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<CHAR>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) as DWORD;
        dataA = Alloc(len) as LPSTR;
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            lpData as LPCWSTR,
            -1 as ::core::ffi::c_int,
            dataA,
            len as ::core::ffi::c_int,
            ::core::ptr::null::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        );
    }
    while i < (*mp).cursize {
        if (*mp).extview.fFlags & MRU_BINARY as UINT != 0 {
            if ((*mp).extview.u.binary_cmpfn)(
                lpData,
                &raw mut (**(*mp).array.offset(i as isize)).datastart as LPCVOID,
                cbData,
            ) == 0
            {
                break;
            }
        } else if (*mp).isUnicode != 0 {
            if ((*mp).extview.u.string_cmpfn)(
                lpData as LPCWSTR,
                &raw mut (**(*mp).array.offset(i as isize)).datastart as LPWSTR as LPCWSTR,
            ) == 0
            {
                break;
            }
        } else {
            let mut len_0: DWORD = WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                &raw mut (**(*mp).array.offset(i as isize)).datastart as LPWSTR as LPCWSTR,
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<CHAR>(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) as DWORD;
            let mut itemA: LPSTR = Alloc(len_0) as LPSTR;
            let mut cmp: INT = 0;
            WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                &raw mut (**(*mp).array.offset(i as isize)).datastart as LPWSTR as LPCWSTR,
                -1 as ::core::ffi::c_int,
                itemA,
                len_0 as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            );
            cmp = ((*mp).extview.u.string_cmpfn)(
                dataA as LPWSTR as LPCWSTR,
                itemA as LPWSTR as LPCWSTR,
            );
            Free(itemA as LPVOID);
            if cmp == 0 {
                break;
            }
        }
        i = i.wrapping_add(1);
    }
    Free(dataA as LPVOID);
    if i < (*mp).cursize {
        ret = i as INT;
    } else {
        ret = -1 as INT;
    }
    if !lpRegNum.is_null() && ret != -1 as INT {
        *lpRegNum = ('a' as i32 as UINT).wrapping_add(i) as ::core::ffi::c_int;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"FindMRUData\0" as *const u8 as *const ::core::ffi::c_char,
            440 as ::core::ffi::c_int,
            b"(%p, %p, %d, %p) returning %d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hList,
            lpData,
            cbData,
            lpRegNum,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn AddMRUData(hList: HANDLE, lpData: LPCVOID, cbData: DWORD) -> INT {
    let mp: LPWINEMRULIST = hList as LPWINEMRULIST;
    let mut witem: LPWINEMRUITEM = core::ptr::null_mut();
    let mut i: INT = 0;
    let mut replace: INT = 0;

    replace = FindMRUData(hList, lpData, cbData, core::ptr::null_mut());
    if replace >= 0 {
        /* Item exists, just move it to the front */
        let mut pos: LPWSTR = wcschr((*mp).realMRU as *const WCHAR, (replace + 'a' as INT) as WCHAR) as LPWSTR;
        while pos > (*mp).realMRU {
            *pos.offset(0 as isize) = *pos.offset(-(1 as isize));
            pos = pos.offset(-1);
        }
    } else {
        /* either add a new entry or replace oldest */
        if (*mp).cursize < (*mp).extview.uMax {
            /* Add in a new item */
            replace = (*mp).cursize as INT;
            (*mp).cursize = (*mp).cursize.wrapping_add(1);
        } else {
            /* get the oldest entry and replace data */
            replace = ((*(*mp).realMRU.offset((*mp).cursize.wrapping_sub(1) as isize)) as INT - 'a' as INT) as INT;
            Free(*(*mp).array.offset(replace as isize) as LPVOID);
        }

        /* Allocate space for new item and move in the data */
        witem = Alloc((cbData as usize).wrapping_add(core::mem::size_of::<WINEMRUITEM>()) as DWORD) as LPWINEMRUITEM;
        *(*mp).array.offset(replace as isize) = witem;
        (*witem).itemFlag |= WMRUIF_CHANGED as DWORD;
        (*witem).size = cbData;
        memcpy(
            core::ptr::addr_of_mut!((*witem).datastart) as *mut core::ffi::c_void,
            lpData as *const core::ffi::c_void,
            cbData as usize,
        );

        /* now rotate MRU list */
        i = (*mp).cursize.wrapping_sub(1) as INT;
        while i >= 1 as INT {
            *(*mp).realMRU.offset(i as isize) = *(*mp).realMRU.offset((i - 1 as INT) as isize);
            i -= 1;
        }
    }

    /* The new item gets the front spot */
    (*mp).wineFlags |= WMRUF_CHANGED as DWORD;
    *(*mp).realMRU.offset(0 as isize) = (replace + 'a' as INT) as WCHAR;

    if ((*mp).extview.fFlags & (MRU_CACHEWRITE as UINT)) == 0 {
        /* save changed stuff right now */
        MRU_SaveChanged(mp);
    }

    return replace;
}

#[no_mangle]
pub unsafe extern "system" fn AddMRUStringW(hList: HANDLE, lpszString: LPCWSTR) -> INT {
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"AddMRUStringW\0" as *const u8 as *const ::core::ffi::c_char,
            543 as ::core::ffi::c_int,
            b"(%p,%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hList,
            debugstr_w(lpszString as *const WCHAR),
        );
    }

    if hList.is_null() {
        return -1;
    }

    if lpszString.is_null()
        || IsBadStringPtrW(lpszString, usize::MAX as UINT_PTR) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    return AddMRUData(
        hList,
        lpszString as LPCVOID,
        ((lstrlenW(lpszString) + 1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>()) as DWORD,
    );
}

#[no_mangle]
pub unsafe extern "system" fn AddMRUStringA(mut hList: HANDLE, mut lpszString: LPCSTR) -> INT {
    let mut len: DWORD = 0;
    let mut stringW: LPWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut ret: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"AddMRUStringA\0" as *const u8 as *const ::core::ffi::c_char,
            569 as ::core::ffi::c_int,
            b"(%p,%s)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hList,
            debugstr_a(lpszString as *const ::core::ffi::c_char),
        );
    }
    if hList.is_null() {
        return -(1 as INT);
    }
    if IsBadStringPtrA(lpszString, -(1 as ::core::ffi::c_int) as UINT_PTR) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0 as INT;
    }
    len = (MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszString,
        -(1 as ::core::ffi::c_int),
        ::core::ptr::null_mut::<WCHAR>(),
        0 as ::core::ffi::c_int,
    ) as usize)
        .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
    stringW = Alloc(len) as LPWSTR;
    if stringW.is_null() {
        return -(1 as INT);
    }
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszString,
        -(1 as ::core::ffi::c_int),
        stringW,
        (len as usize).wrapping_div(::core::mem::size_of::<WCHAR>() as usize) as ::core::ffi::c_int,
    );
    ret = AddMRUData(hList, stringW as LPCVOID, len);
    Free(stringW as LPVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn DelMRUString(hList: HANDLE, nItemPos: INT) -> BOOL {
    // FIXME("(%p, %d): stub\n", hList, nItemPos);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn FindMRUStringW(hList: HANDLE, lpszString: LPCWSTR, lpRegNum: LPINT) -> INT {
    return FindMRUData(hList, lpszString as LPCVOID, (((lstrlenW(lpszString) + 1) as usize).wrapping_mul(core::mem::size_of::<WCHAR>())) as DWORD, lpRegNum);
}

#[no_mangle]
pub unsafe extern "system" fn FindMRUStringA(hList: HANDLE, lpszString: LPCSTR, lpRegNum: LPINT) -> INT {
    let len: DWORD = MultiByteToWideChar(CP_ACP, 0, lpszString, -1, core::ptr::null_mut(), 0) as DWORD;
    let stringW: LPWSTR = Alloc(len * core::mem::size_of::<WCHAR>() as DWORD) as LPWSTR;
    let ret: INT;

    MultiByteToWideChar(CP_ACP, 0, lpszString, -1, stringW, len as core::ffi::c_int);
    ret = FindMRUData(hList, stringW as LPCVOID, len * core::mem::size_of::<WCHAR>() as DWORD, lpRegNum);
    Free(stringW as LPVOID);
    return ret;
}

unsafe extern "C" fn create_mru_list(mp: LPWINEMRULIST) -> HANDLE {
    let mut i: UINT = 0;
    let mut err: UINT = 0;
    let mut newkey: HKEY = core::ptr::null_mut();
    let mut datasize: DWORD = 0;
    let mut dwdisp: DWORD = 0;
    let mut realname: [WCHAR; 2] = [0; 2];
    let mut witem: LPWINEMRUITEM = core::ptr::null_mut();
    let mut type_0: DWORD = 0;

    /* get space to save indices that will turn into names
     * but in order of most to least recently used
     */
    (*mp).realMRU = Alloc(
        ((*mp).extview.uMax.wrapping_add(2) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
    ) as LPWSTR;

    /* get space to save pointers to actual data in order of
     * 'a' to 'z' (0 to n).
     */
    (*mp).array = Alloc(
        ((*mp).extview.uMax as usize)
            .wrapping_mul(::core::mem::size_of::<LPVOID>() as usize) as DWORD,
    ) as *mut LPWINEMRUITEM;

    /* open the sub key */
    err = RegCreateKeyExW(
        (*mp).extview.hKey,
        (*mp).extview.lpszSubKey as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE as DWORD,
        (KEY_READ | KEY_WRITE) as REGSAM,
        core::ptr::null_mut(),
        &raw mut newkey,
        &raw mut dwdisp,
    ) as UINT;
    if err != 0 {
        /* error - what to do ??? */
        return core::ptr::null_mut();
    }

    /* get values from key 'MRUList' */
    if !newkey.is_null() {
        datasize = ((*mp).extview.uMax.wrapping_add(1) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD;
        if RegQueryValueExW(
            newkey,
            core::ptr::addr_of!(strMRUList) as LPCWSTR,
            core::ptr::null_mut(),
            &raw mut type_0,
            (*mp).realMRU as LPBYTE,
            &raw mut datasize,
        ) != 0
        {
            /* not present - set size to 1 (will become 0 later) */
            datasize = 1;
            *(*mp).realMRU = 0;
        } else {
            datasize /= ::core::mem::size_of::<WCHAR>() as DWORD;
        }

        /* MRU list read; trace output omitted */

        (*mp).cursize = datasize.wrapping_sub(1);
        /* datasize now has number of items in the MRUList */

        /* get actual values for each entry */
        realname[1] = 0;
        i = 0;
        while i < (*mp).cursize {
            realname[0] = ('a' as UINT).wrapping_add(i) as WCHAR;
            if RegQueryValueExW(
                newkey,
                &raw mut realname as *mut WCHAR as LPCWSTR,
                core::ptr::null_mut(),
                &raw mut type_0,
                core::ptr::null_mut(),
                &raw mut datasize,
            ) != 0
            {
                /* not present - what to do ??? */
            }
            witem = Alloc(
                (datasize as usize).wrapping_add(::core::mem::size_of::<WINEMRUITEM>() as usize)
                    as DWORD,
            ) as LPWINEMRUITEM;
            *(*mp).array.offset(i as isize) = witem;
            (*witem).size = datasize;
            if RegQueryValueExW(
                newkey,
                &raw mut realname as *mut WCHAR as LPCWSTR,
                core::ptr::null_mut(),
                &raw mut type_0,
                &raw mut (*witem).datastart,
                &raw mut datasize,
            ) != 0
            {
                /* not present - what to do ??? */
            }
            i = i.wrapping_add(1);
        }
        RegCloseKey(newkey);
    } else {
        (*mp).cursize = 0;
    }

    /* Current size trace output omitted */
    return mp as HANDLE;
}

#[no_mangle]
pub unsafe extern "system" fn CreateMRUListLazyW(
    infoW: *const MRUINFOW,
    dwParam2: DWORD,
    dwParam3: DWORD,
    dwParam4: DWORD,
) -> HANDLE {
    let mut mp: LPWINEMRULIST = core::ptr::null_mut();

    /* Native does not check for a NULL lpcml */
    if (*infoW).hKey.is_null()
        || IsBadStringPtrW((*infoW).lpszSubKey as LPCWSTR, -(1 as ::core::ffi::c_int) as UINT_PTR) != 0
    {
        return core::ptr::null_mut();
    }

    mp = Alloc(core::mem::size_of::<WINEMRULIST>() as DWORD) as LPWINEMRULIST;
    memcpy(
        core::ptr::addr_of_mut!((*mp).extview) as *mut ::core::ffi::c_void,
        infoW as *const ::core::ffi::c_void,
        core::mem::size_of::<MRUINFOW>() as size_t,
    );
    (*mp).extview.lpszSubKey = Alloc(
        ((lstrlenW((*infoW).lpszSubKey as LPCWSTR) + 1 as ::core::ffi::c_int) as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
    ) as LPWSTR;
    lstrcpyW((*mp).extview.lpszSubKey, (*infoW).lpszSubKey as LPCWSTR);
    (*mp).isUnicode = TRUE as BOOL;

    return create_mru_list(mp);
}

#[no_mangle]
pub unsafe extern "system" fn CreateMRUListLazyA(
    lpcml: *const MRUINFOA,
    dwParam2: DWORD,
    dwParam3: DWORD,
    dwParam4: DWORD,
) -> HANDLE {
    let mut mp: LPWINEMRULIST = core::ptr::null_mut();
    let mut len: DWORD = 0;

    /* Native does not check for a NULL lpcml */

    if (*lpcml).hKey.is_null()
        || IsBadStringPtrA((*lpcml).lpszSubKey as LPCSTR, !0 as UINT_PTR) != 0
    {
        return core::ptr::null_mut();
    }

    mp = Alloc(core::mem::size_of::<WINEMRULIST>() as DWORD) as LPWINEMRULIST;
    memcpy(
        core::ptr::addr_of_mut!((*mp).extview) as *mut core::ffi::c_void,
        lpcml as *const core::ffi::c_void,
        core::mem::size_of::<MRUINFOA>() as usize,
    );
    len = MultiByteToWideChar(
        CP_ACP,
        0,
        (*lpcml).lpszSubKey as LPCSTR,
        -1,
        core::ptr::null_mut(),
        0,
    ) as DWORD;
    (*mp).extview.lpszSubKey =
        Alloc((len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        (*lpcml).lpszSubKey as LPCSTR,
        -1,
        (*mp).extview.lpszSubKey,
        len as core::ffi::c_int,
    );
    (*mp).isUnicode = FALSE;
    return create_mru_list(mp);
}

#[no_mangle]
pub unsafe extern "system" fn CreateMRUListW(infoW: *const MRUINFOW) -> HANDLE {
    return CreateMRUListLazyW(infoW, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn CreateMRUListA(lpcml: *const MRUINFOA) -> HANDLE {
    return CreateMRUListLazyA(lpcml, 0, 0, 0);
}

#[no_mangle]
pub unsafe extern "system" fn EnumMRUListW(
    hList: HANDLE,
    nItemPos: INT,
    lpBuffer: LPVOID,
    nBufferSize: DWORD,
) -> INT {
    let mp: *const WINEMRULIST = hList as *const WINEMRULIST;
    let mut witem: *const WINEMRUITEM = core::ptr::null::<WINEMRUITEM>();
    let mut desired: INT = 0;
    let mut datasize: INT = 0;
    if mp.is_null() {
        return -1;
    }
    if nItemPos < 0 || lpBuffer.is_null() {
        return (*mp).cursize as INT;
    }
    if nItemPos as DWORD >= (*mp).cursize {
        return -1;
    }
    desired = *(*mp).realMRU.offset(nItemPos as isize) as INT;
    desired -= 'a' as i32;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumMRUListW\0" as *const u8 as *const ::core::ffi::c_char,
            846 as ::core::ffi::c_int,
            b"nItemPos=%d, desired=%d\n\0" as *const u8 as *const ::core::ffi::c_char,
            nItemPos,
            desired,
        );
    }
    witem = *(*mp).array.offset(desired as isize) as *const WINEMRUITEM;
    datasize = min((*witem).size, nBufferSize) as INT;
    memcpy(
        lpBuffer as *mut ::core::ffi::c_void,
        core::ptr::addr_of!((*witem).datastart) as *const ::core::ffi::c_void,
        datasize as usize,
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumMRUListW\0" as *const u8 as *const ::core::ffi::c_char,
            851 as ::core::ffi::c_int,
            b"(%p, %d, %p, %d): returning len=%d\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hList,
            nItemPos,
            lpBuffer,
            nBufferSize,
            datasize,
        );
    }
    return datasize;
}

#[no_mangle]
pub unsafe extern "system" fn EnumMRUListA(
    hList: HANDLE,
    nItemPos: INT,
    lpBuffer: LPVOID,
    nBufferSize: DWORD,
) -> INT {
    let mp: *const WINEMRULIST = hList as *const WINEMRULIST;
    let mut witem: LPWINEMRUITEM = core::ptr::null_mut();
    let mut desired: INT;
    let mut datasize: INT;
    let mut lenA: DWORD;

    if mp.is_null() {
        return -1;
    }
    if (nItemPos < 0) || lpBuffer.is_null() {
        return (*mp).cursize as INT;
    }
    if nItemPos as DWORD >= (*mp).cursize {
        return -1;
    }
    desired = *(*mp).realMRU.offset(nItemPos as isize) as INT;
    desired -= 'a' as INT;

    witem = *(*mp).array.offset(desired as isize);
    if (*mp).extview.fFlags & (MRU_BINARY as u32) != 0 {
        datasize = min((*witem).size, nBufferSize) as INT;
        memcpy(
            lpBuffer,
            core::ptr::addr_of!((*witem).datastart) as *const core::ffi::c_void,
            datasize as usize,
        );
    } else {
        lenA = WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!((*witem).datastart) as LPWSTR,
            -1,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) as DWORD;
        datasize = min(lenA, nBufferSize) as INT;
        WideCharToMultiByte(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!((*witem).datastart) as LPWSTR,
            -1,
            lpBuffer as LPSTR,
            datasize,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        *(lpBuffer as *mut i8).offset((datasize - 1) as isize) = '\0' as i8;
        datasize = (lenA - 1) as INT;
    }
    return datasize;
}
#[no_mangle]
pub unsafe extern "C" fn Str_GetPtrWtoA(mut lpSrc: LPCWSTR, mut lpDest: LPSTR, mut nMaxLen: INT) -> INT {
    let mut len: INT = 0;
    if lpDest.is_null() && !lpSrc.is_null() {
        return WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            lpSrc,
            -(1 as INT),
            ::core::ptr::null_mut::<CHAR>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<CHAR>(),
            ::core::ptr::null_mut::<BOOL>(),
        ) as INT;
    }
    if nMaxLen == 0 as INT {
        return 0 as INT;
    }
    if lpSrc.is_null() {
        *lpDest.offset(0 as isize) = '\0' as i32 as CHAR;
        return 0 as INT;
    }
    len = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        lpSrc,
        -(1 as INT),
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as INT;
    if len >= nMaxLen {
        len = nMaxLen - 1 as INT;
    }
    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        lpSrc,
        -(1 as INT),
        lpDest,
        len as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    *lpDest.offset(len as isize) = '\0' as i32 as CHAR;
    return len;
}
#[no_mangle]
pub unsafe extern "C" fn Str_GetPtrAtoW(mut lpSrc: LPCSTR, mut lpDest: LPWSTR, mut nMaxLen: INT) -> INT {
    let mut len: INT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"Str_GetPtrAtoW\0" as *const u8 as *const ::core::ffi::c_char,
            951 as ::core::ffi::c_int,
            b"(%s %p %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_a(lpSrc as *const ::core::ffi::c_char),
            lpDest,
            nMaxLen,
        );
    }

    if !lpDest.is_null() == false && !lpSrc.is_null() {
        return MultiByteToWideChar(
            CP_ACP,
            0,
            lpSrc,
            -1,
            core::ptr::null_mut(),
            0,
        ) as INT;
    }

    if nMaxLen == 0 {
        return 0;
    }

    if lpSrc.is_null() {
        *lpDest.offset(0 as isize) = '\0' as u16;
        return 0;
    }

    len = MultiByteToWideChar(
        CP_ACP,
        0,
        lpSrc,
        -1,
        core::ptr::null_mut(),
        0,
    ) as INT;
    if len >= nMaxLen {
        len = nMaxLen - 1;
    }

    MultiByteToWideChar(
        CP_ACP,
        0,
        lpSrc,
        -1,
        lpDest,
        len,
    );
    *lpDest.offset(len as isize) = '\0' as u16;

    return len;
}
#[no_mangle]
pub unsafe extern "C" fn Str_SetPtrAtoW(lppDest: *mut LPWSTR, lpSrc: LPCSTR) -> BOOL {
    if !lpSrc.is_null() {
        let len: INT = MultiByteToWideChar(CP_ACP, 0, lpSrc, -1, core::ptr::null_mut(), 0) as INT;
        let ptr: LPWSTR = ReAlloc(*lppDest as LPVOID, (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD) as LPWSTR;

        if ptr.is_null() {
            return FALSE;
        }
        MultiByteToWideChar(CP_ACP, 0, lpSrc, -1, ptr, len as INT);
        *lppDest = ptr;
    } else {
        Free(*lppDest as LPVOID);
        *lppDest = core::ptr::null_mut();
    }

    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn Str_SetPtrWtoA(lppDest: *mut LPSTR, lpSrc: LPCWSTR) -> BOOL {
    if !lpSrc.is_null() {
        let len: INT = WideCharToMultiByte(CP_ACP, 0, lpSrc, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), FALSE as usize as *mut BOOL) as INT;
        let ptr: LPSTR = ReAlloc(*lppDest as LPVOID, (len as usize).wrapping_mul(core::mem::size_of::<CHAR>()) as DWORD) as LPSTR;

        if ptr.is_null() {
            return FALSE;
        }
        WideCharToMultiByte(CP_ACP, 0, lpSrc, -1, ptr, len, core::ptr::null_mut(), FALSE as usize as *mut BOOL);
        *lppDest = ptr;
    } else {
        Free(*lppDest as LPVOID);
        *lppDest = core::ptr::null_mut();
    }

    TRUE
}

unsafe extern "C" fn DoNotify(
    mut lpNotify: *const NOTIFYDATA,
    mut uCode: UINT,
    mut lpHdr: LPNMHDR,
) -> LRESULT {
    let mut nmhdr: NMHDR = core::mem::zeroed();
    let mut lpNmh: LPNMHDR = core::ptr::null_mut();
    let mut idFrom: UINT = 0 as UINT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/comctl32undoc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"DoNotify\0" as *const u8 as *const ::core::ffi::c_char,
            1079 as ::core::ffi::c_int,
            b"(%p %p %d %p 0x%08x)\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*lpNotify).hwndFrom,
            (*lpNotify).hwndTo,
            uCode,
            lpHdr,
            (*lpNotify).dwParam5,
        );
    }
    if (*lpNotify).hwndTo.is_null() {
        return 0 as LRESULT;
    }
    if (*lpNotify).hwndFrom == (-(1 as isize)) as HWND {
        lpNmh = lpHdr;
        idFrom = (*lpHdr).idFrom as UINT;
    } else {
        if !(*lpNotify).hwndFrom.is_null() {
            idFrom = GetDlgCtrlID((*lpNotify).hwndFrom) as UINT;
        }
        lpNmh = (if !lpHdr.is_null() {
            lpHdr
        } else {
            core::ptr::addr_of_mut!(nmhdr)
        }) as LPNMHDR;
        (*lpNmh).hwndFrom = (*lpNotify).hwndFrom;
        (*lpNmh).idFrom = idFrom as UINT_PTR;
        (*lpNmh).code = uCode;
    }
    return SendMessageW(
        (*lpNotify).hwndTo,
        WM_NOTIFY as UINT,
        idFrom as WPARAM,
        lpNmh as LPARAM,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SendNotify(hwndTo: HWND, hwndFrom: HWND, uCode: UINT, lpHdr: LPNMHDR) -> LRESULT {
    let mut notify: NOTIFYDATA = core::mem::zeroed();

    notify.hwndFrom = hwndFrom;
    notify.hwndTo = hwndTo;
    notify.dwParam5 = 0;
    notify.dwParam6 = 0;

    return DoNotify(core::ptr::addr_of_mut!(notify), uCode, lpHdr);
}

#[no_mangle]
pub unsafe extern "system" fn SendNotifyEx(
    hwndTo: HWND,
    hwndFrom: HWND,
    uCode: UINT,
    lpHdr: LPNMHDR,
    dwParam5: DWORD,
) -> LRESULT {
    let mut notify: NOTIFYDATA = core::mem::zeroed();
    let mut hwndNotify: HWND = core::ptr::null_mut();

    hwndNotify = hwndTo;
    if hwndTo.is_null() {
        if IsWindow(hwndFrom) != 0 {
            hwndNotify = GetParent(hwndFrom);
            if hwndNotify.is_null() {
                return 0;
            }
        }
    }

    notify.hwndFrom = hwndFrom;
    notify.hwndTo = hwndNotify;
    notify.dwParam5 = dwParam5;
    notify.dwParam6 = 0;

    return DoNotify(core::ptr::addr_of_mut!(notify), uCode, lpHdr);
}
