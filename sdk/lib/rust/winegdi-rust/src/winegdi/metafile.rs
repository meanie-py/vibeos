use crate::winegdi::metadc::CloseMetaFile;
use crate::winegdi::metadc::CreateMetaFileW;
use crate::winegdi::rosglue::GDI_GetObjPtr;
use crate::winegdi::rosglue::GDI_ReleaseObj;
use crate::winegdi::rosglue::alloc_gdi_handle;
use crate::winegdi::rosglue::bitmap_info_size;
use crate::winegdi::rosglue::free_gdi_handle;
use crate::prelude::*;
use core::ffi::c_void;

// WINE_DEFAULT_DEBUG_CHANNEL(metafile): shadows the prelude's (enhmetafile) default channel.
static mut __wine_dbch___default: *mut __wine_debug_channel =
    &raw mut __wine_dbch_metafile;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(metafile): debug-channel registration macro, no Rust equivalent emitted.
// Forward prototypes of MF_Play_MetaCreateRegion / MF_Play_MetaExtTextOut: not declarations in Rust, the function bodies carry the signatures.

// metafile.c declares it inside #include <pshpack1.h> / <poppack.h>: 270 bytes, not 272.
#[repr(C, packed)]
pub struct METAHEADERDISK {
    pub dw1: DWORD,
    pub dw2: DWORD,
    pub dw3: DWORD,
    pub w4: WORD,
    pub filename: [CHAR; 0x100],
}

pub type PMETAHEADERDISK = *mut METAHEADERDISK;

unsafe extern "C" fn MF_AddHandle(mut ht: *mut HANDLETABLE, mut htlen: UINT, mut hobj: HGDIOBJ) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    while (i as UINT) < htlen {
        if *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).offset(i as isize) == core::ptr::null_mut() {
            *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).offset(i as isize) = hobj;
            return i;
        }
        i += 1;
    }
    return -1;
}
#[no_mangle]
pub unsafe extern "C" fn MF_Create_HMETAFILE(mh: *mut METAHEADER) -> HMETAFILE {
    return alloc_gdi_handle(
        mh as *mut core::ffi::c_void,
        OBJ_METAFILE as WORD,
        core::ptr::null::<gdi_obj_funcs>(),
    ) as HMETAFILE;
}

unsafe extern "C" fn convert_points(count: UINT, pts: *const POINTS) -> *mut POINT {
    let mut i: UINT;
    let mut ret: *mut POINT = HeapAlloc(GetProcessHeap(), 0, (count as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T)) as *mut POINT;
    if !ret.is_null() {
        i = 0;
        while i < count {
            (*ret.add(i as usize)).x = (*pts.add(i as usize)).x as LONG;
            (*ret.add(i as usize)).y = (*pts.add(i as usize)).y as LONG;
            i = i.wrapping_add(1);
        }
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn DeleteMetaFile(mut hmf: HMETAFILE) -> BOOL {
    let mut mh: *mut METAHEADER = free_gdi_handle(hmf as HGDIOBJ) as *mut METAHEADER;
    if mh.is_null() {
        return (FALSE) as BOOL;
    }
    return HeapFree(GetProcessHeap(), 0, mh as PVOID);
}

unsafe extern "C" fn MF_ReadMetaFile(hfile: HANDLE) -> *mut METAHEADER {
    let mut mh: *mut METAHEADER;
    let mut BytesRead: DWORD = 0;
    let mut size: DWORD = 0;

    size = ::core::mem::size_of::<METAHEADER>() as DWORD;
    mh = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut METAHEADER;
    if mh.is_null() {
        return core::ptr::null_mut();
    }
    if ReadFile(
        hfile,
        mh as LPVOID,
        size,
        core::ptr::addr_of_mut!(BytesRead),
        core::ptr::null_mut(),
    ) == 0
        || BytesRead != size
    {
        HeapFree(GetProcessHeap(), 0, mh as LPVOID);
        return core::ptr::null_mut();
    }
    if (*mh).mtType != (METAFILE_MEMORY as u16)
        || (*mh).mtVersion != (MFVERSION as u16)
        || (*mh).mtHeaderSize as DWORD != size / 2
    {
        HeapFree(GetProcessHeap(), 0, mh as LPVOID);
        return core::ptr::null_mut();
    }
    size = (*mh).mtSize * 2;
    mh = HeapReAlloc(GetProcessHeap(), 0, mh as LPVOID, size as SIZE_T) as *mut METAHEADER;
    if mh.is_null() {
        return core::ptr::null_mut();
    }
    size -= ::core::mem::size_of::<METAHEADER>() as DWORD;
    if ReadFile(
        hfile,
        (mh as *mut u8).add(::core::mem::size_of::<METAHEADER>()) as LPVOID,
        size,
        core::ptr::addr_of_mut!(BytesRead),
        core::ptr::null_mut(),
    ) == 0
        || BytesRead != size
    {
        HeapFree(GetProcessHeap(), 0, mh as LPVOID);
        return core::ptr::null_mut();
    }

    if (*mh).mtType != (METAFILE_MEMORY as u16) {
        (*mh).mtType = (METAFILE_MEMORY as u16);
    }
    return mh;
}
#[no_mangle]
pub unsafe extern "system" fn GetMetaFileA(mut lpFilename: LPCSTR) -> HMETAFILE {
    let mut mh: *mut METAHEADER = ::core::ptr::null_mut::<METAHEADER>();
    let mut hFile: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/metafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetMetaFileA\0" as *const u8 as *const ::core::ffi::c_char,
            199 as ::core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const ::core::ffi::c_char,
            lpFilename,
        );
    }
    if lpFilename.is_null() {
        return core::ptr::null_mut();
    }
    hFile = CreateFileA(
        lpFilename,
        GENERIC_READ as DWORD,
        FILE_SHARE_READ as DWORD,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING as DWORD,
        0 as DWORD,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut();
    }
    mh = MF_ReadMetaFile(hFile);
    CloseHandle(hFile);
    if mh.is_null() {
        return core::ptr::null_mut();
    }
    return MF_Create_HMETAFILE(mh);
}
#[no_mangle]
pub unsafe extern "system" fn GetMetaFileW(lpFilename: LPCWSTR) -> HMETAFILE {
    let mut mh: *mut METAHEADER = core::ptr::null_mut();
    let mut hFile: HANDLE = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/metafile.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"GetMetaFileW\0" as *const u8 as *const core::ffi::c_char,
            222 as core::ffi::c_int,
            b"%s\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(lpFilename as *const WCHAR),
        );
    }
    if lpFilename.is_null() {
        return core::ptr::null_mut();
    }
    hFile = CreateFileW(
        lpFilename,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut();
    }
    mh = MF_ReadMetaFile(hFile);
    CloseHandle(hFile);
    if mh.is_null() {
        return core::ptr::null_mut();
    }
    return MF_Create_HMETAFILE(mh);
}

unsafe extern "C" fn MF_LoadDiskBasedMetaFile(mut mh: *mut METAHEADER) -> *mut METAHEADER {
    let mut mhd: *mut METAHEADERDISK = ::core::ptr::null_mut::<METAHEADERDISK>();
    let mut hfile: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut mh2: *mut METAHEADER = ::core::ptr::null_mut::<METAHEADER>();
    if (*mh).mtType as ::core::ffi::c_int != METAFILE_DISK as ::core::ffi::c_int {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MF_LoadDiskBasedMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                250 as ::core::ffi::c_int,
                b"Not a disk based metafile\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return ::core::ptr::null_mut::<METAHEADER>();
    }
    mhd = (mh as *mut ::core::ffi::c_char)
        .offset(core::mem::size_of::<METAHEADER>() as usize as isize)
        as *mut METAHEADERDISK;
    hfile = CreateFileA(
        &raw mut (*mhd).filename as *mut CHAR as LPCSTR,
        GENERIC_READ,
        FILE_SHARE_READ as DWORD,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING as DWORD,
        0 as DWORD,
        ::core::ptr::null_mut::<::core::ffi::c_void>(),
    );
    if hfile == INVALID_HANDLE_VALUE {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MF_LoadDiskBasedMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                257 as ::core::ffi::c_int,
                b"Can't open file of disk based metafile\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return ::core::ptr::null_mut::<METAHEADER>();
    }
    mh2 = MF_ReadMetaFile(hfile);
    CloseHandle(hfile);
    return mh2;
}
#[no_mangle]
pub unsafe extern "C" fn MF_CreateMetaHeaderDisk(mut mh: *mut METAHEADER, mut filename: LPCVOID, mut uni: BOOL) -> *mut METAHEADER {
    let mut mhd: *mut METAHEADERDISK = core::ptr::null_mut::<METAHEADERDISK>();
    mh = HeapReAlloc(
        GetProcessHeap(),
        0 as DWORD,
        mh as PVOID,
        (core::mem::size_of::<METAHEADER>() as SIZE_T)
            .wrapping_add(core::mem::size_of::<METAHEADERDISK>() as SIZE_T),
    ) as *mut METAHEADER;
    (*mh).mtType = METAFILE_DISK as WORD;
    mhd = (mh as *mut core::ffi::c_char)
        .offset(core::mem::size_of::<METAHEADER>() as usize as isize)
        as *mut METAHEADERDISK;

    if uni != 0 {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            filename as LPCWSTR,
            -(1 as core::ffi::c_int),
            core::ptr::addr_of_mut!((*mhd).filename) as LPSTR,
            core::mem::size_of::<[CHAR; 256]>() as core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    } else {
        lstrcpynA(
            core::ptr::addr_of_mut!((*mhd).filename) as LPSTR,
            filename as LPCSTR,
            core::mem::size_of::<[CHAR; 256]>() as core::ffi::c_int,
        );
    }
    return mh;
}

unsafe extern "C" fn get_metafile_bits(mut hmf: HMETAFILE) -> *mut METAHEADER {
    let mut ret: *mut METAHEADER = ::core::ptr::null_mut::<METAHEADER>();
    let mut mh: *mut METAHEADER =
        GDI_GetObjPtr(hmf as HGDIOBJ, OBJ_METAFILE as WORD) as *mut METAHEADER;
    if mh.is_null() {
        return ::core::ptr::null_mut::<METAHEADER>();
    }
    if (*mh).mtType as ::core::ffi::c_int != (METAFILE_DISK as i32) {
        ret = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (*mh).mtSize.wrapping_mul(2 as DWORD) as SIZE_T,
        ) as *mut METAHEADER;
        if !ret.is_null() {
            memcpy(
                ret as *mut ::core::ffi::c_void,
                mh as *const ::core::ffi::c_void,
                (*mh).mtSize.wrapping_mul(2 as DWORD) as size_t,
            );
        }
    } else {
        ret = MF_LoadDiskBasedMetaFile(mh);
    }
    GDI_ReleaseObj(hmf as HGDIOBJ);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CopyMetaFileW(hSrcMetaFile: HMETAFILE, lpFilename: LPCWSTR) -> HMETAFILE {
    let mut mh: *mut METAHEADER = get_metafile_bits(hSrcMetaFile);
    let mut hFile: HANDLE = core::ptr::null_mut();

    if mh.is_null() {
        return core::ptr::null_mut();
    }

    if !lpFilename.is_null() {
        let mut w: DWORD = 0;
        hFile = CreateFileW(
            lpFilename,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            0,
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            HeapFree(GetProcessHeap(), 0, mh as PVOID);
            return core::ptr::null_mut();
        }
        WriteFile(
            hFile,
            mh as LPCVOID,
            (*mh).mtSize * 2,
            core::ptr::addr_of_mut!(w),
            core::ptr::null_mut(),
        );
        CloseHandle(hFile);
    }

    return MF_Create_HMETAFILE(mh);
}

#[no_mangle]
pub unsafe extern "system" fn CopyMetaFileA( hSrcMetaFile: HMETAFILE, lpFilename: LPCSTR ) -> HMETAFILE {
    let mut lpFilenameW: _UNICODE_STRING = core::mem::zeroed();
    let mut ret: HMETAFILE = core::ptr::null_mut();

    if !lpFilename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(lpFilenameW), lpFilename);
    } else {
        lpFilenameW.Buffer = core::ptr::null_mut();
    }

    ret = CopyMetaFileW( hSrcMetaFile, lpFilenameW.Buffer as LPCWSTR );
    if !lpFilenameW.Buffer.is_null() {
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(lpFilenameW));
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn PlayMetaFile(hdc: HDC, hmf: HMETAFILE) -> BOOL {
    let mut mh: *mut METAHEADER = get_metafile_bits(hmf);
    let mut mr: *mut METARECORD = core::ptr::null_mut();
    let mut ht: *mut HANDLETABLE = core::ptr::null_mut();
    let mut offset: u32 = 0 as u32;
    let mut i: WORD = 0;
    let mut hPen: HPEN = core::ptr::null_mut();
    let mut hBrush: HBRUSH = core::ptr::null_mut();
    let mut hPal: HPALETTE = core::ptr::null_mut();
    let mut hRgn: HRGN = core::ptr::null_mut();

    if mh.is_null() {
        return (FALSE) as BOOL;
    }

    /* save DC */
    hPen = GetCurrentObject(hdc, OBJ_PEN as UINT) as HPEN;
    hBrush = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
    hPal = GetCurrentObject(hdc, OBJ_PAL as UINT) as HPALETTE;

    hRgn = CreateRectRgn(
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    ) as HRGN;
    if GetClipRgn(hdc, hRgn) == 0 {
        DeleteObject(hRgn as HGDIOBJ);
        hRgn = core::ptr::null_mut();
    }

    /* create the handle table */
    ht = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (::core::mem::size_of::<HANDLETABLE>() as SIZE_T).wrapping_mul((*mh).mtNoObjects as SIZE_T),
    ) as *mut HANDLETABLE;
    if ht.is_null() {
        HeapFree(GetProcessHeap(), 0 as DWORD, mh as PVOID);
        return (FALSE) as BOOL;
    }

    /* loop through metafile playing records */
    offset = ((*mh).mtHeaderSize as u32).wrapping_mul(2 as u32);
    while (offset as DWORD) < (*mh).mtSize.wrapping_mul(2 as DWORD) {
        mr = (mh as *mut ::core::ffi::c_char).offset(offset as isize) as *mut METARECORD;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"PlayMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                421 as ::core::ffi::c_int,
                b"offset=%04x,size=%08x\n\0" as *const u8 as *const ::core::ffi::c_char,
                offset,
                (*mr).rdSize,
            );
        }
        if (*mr).rdSize < 3 as DWORD {
            /* catch illegal record sizes */
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                    424 as ::core::ffi::c_int,
                    b"Entry got size %d at offset %d, total mf length is %d\n\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    (*mr).rdSize,
                    offset,
                    (*mh).mtSize.wrapping_mul(2 as DWORD),
                );
            }
            break;
        } else {
            offset = (offset as DWORD)
                .wrapping_add((*mr).rdSize.wrapping_mul(2 as DWORD))
                as ::core::ffi::c_uint as ::core::ffi::c_uint;
            if (*mr).rdFunction as ::core::ffi::c_int == (META_EOF as i32) {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                    ros_dbg_log(
                        __dbcl_1,
                        __dbch_1,
                        b"win32ss/gdi/gdi32/wine/metafile.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"PlayMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                        430 as ::core::ffi::c_int,
                        b"Got META_EOF so stopping\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                    );
                }
                break;
            } else {
                PlayMetaFileRecord(hdc, ht, mr, (*mh).mtNoObjects as UINT);
            }
        }
    }

    /* restore DC */
    SelectObject(hdc, hPen as HGDIOBJ);
    SelectObject(hdc, hBrush as HGDIOBJ);
    SelectPalette(hdc, hPal, FALSE as BOOL);
    ExtSelectClipRgn(hdc, hRgn, RGN_COPY as ::core::ffi::c_int);
    DeleteObject(hRgn as HGDIOBJ);

    /* free objects in handle table */
    i = 0 as WORD;
    while (i as ::core::ffi::c_int) < (*mh).mtNoObjects as ::core::ffi::c_int {
        if ((*(&raw mut (*ht).objectHandle as *mut HGDIOBJ)
            .offset(i as ::core::ffi::c_int as isize)) as usize)
            != (0 as usize)
        {
            DeleteObject(
                *(&raw mut (*ht).objectHandle as *mut HGDIOBJ)
                    .offset(i as ::core::ffi::c_int as isize),
            );
        }
        i = i.wrapping_add(1);
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ht as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mh as PVOID,
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn EnumMetaFile(
    mut hdc: HDC,
    mut hmf: HMETAFILE,
    mut lpEnumFunc: MFENUMPROC,
    mut lpData: LPARAM,
) -> BOOL {
    let mut mh: *mut METAHEADER = get_metafile_bits(hmf);
    let mut mr: *mut METARECORD = ::core::ptr::null_mut::<METARECORD>();
    let mut ht: *mut HANDLETABLE = ::core::ptr::null_mut::<HANDLETABLE>();
    let mut result: BOOL = TRUE as BOOL;
    let mut i: ::core::ffi::c_int = 0;
    let mut offset: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut hPen: HPEN = ::core::ptr::null_mut::<HPEN__>();
    let mut hBrush: HBRUSH = ::core::ptr::null_mut::<HBRUSH__>();
    let mut hFont: HFONT = ::core::ptr::null_mut::<HFONT__>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/metafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"EnumMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
            476 as ::core::ffi::c_int,
            b"(%p,%p,%p,%lx)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdc,
            hmf,
            lpEnumFunc,
            lpData,
        );
    }
    if mh.is_null() {
        return FALSE as BOOL;
    }
    hPen = GetCurrentObject(hdc, OBJ_PEN as UINT) as HPEN;
    hBrush = GetCurrentObject(hdc, OBJ_BRUSH as UINT) as HBRUSH;
    hFont = GetCurrentObject(hdc, OBJ_FONT as UINT) as HFONT;
    ht = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (::core::mem::size_of::<HANDLETABLE>() as SIZE_T).wrapping_mul((*mh).mtNoObjects as SIZE_T),
    ) as *mut HANDLETABLE;
    offset = ((*mh).mtHeaderSize as ::core::ffi::c_int * 2 as ::core::ffi::c_int)
        as ::core::ffi::c_uint;
    while (offset as DWORD) < (*mh).mtSize.wrapping_mul(2 as DWORD) {
        mr = (mh as *mut ::core::ffi::c_char).offset(offset as isize) as *mut METARECORD;
        if (*mr).rdFunction as ::core::ffi::c_int == META_EOF as ::core::ffi::c_int {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EnumMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                    495 as ::core::ffi::c_int,
                    b"Got META_EOF so stopping\n\0" as *const u8 as *const ::core::ffi::c_char,
                );
            }
            break;
        } else {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"EnumMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                    499 as ::core::ffi::c_int,
                    b"Calling EnumFunc with record type %x\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    (*mr).rdFunction as ::core::ffi::c_int,
                );
            }
            if lpEnumFunc.unwrap_unchecked()(
                hdc,
                ht,
                mr,
                (*mh).mtNoObjects as ::core::ffi::c_int,
                lpData,
            ) == 0
            {
                result = FALSE as BOOL;
                break;
            } else {
                offset = (offset as DWORD)
                    .wrapping_add((*mr).rdSize.wrapping_mul(2 as DWORD))
                    as ::core::ffi::c_uint as ::core::ffi::c_uint;
            }
        }
    }
    SelectObject(hdc, hBrush as HGDIOBJ);
    SelectObject(hdc, hPen as HGDIOBJ);
    SelectObject(hdc, hFont as HGDIOBJ);
    i = 0 as ::core::ffi::c_int;
    while i < (*mh).mtNoObjects as ::core::ffi::c_int {
        if (*(&raw mut (*ht).objectHandle as *mut HGDIOBJ).offset(i as isize))
            != 0 as HGDIOBJ
        {
            DeleteObject(*(&raw mut (*ht).objectHandle as *mut HGDIOBJ).offset(i as isize));
        }
        i += 1;
    }
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ht as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mh as PVOID,
    );
    return result;
}

#[no_mangle]
pub unsafe extern "system" fn PlayMetaFileRecord(
    hdc: HDC,
    ht: *mut HANDLETABLE,
    mr: *mut METARECORD,
    handles: UINT,
) -> BOOL {
    let mut s1: ::core::ffi::c_short = 0;
    let mut pt: *mut POINT = ::core::ptr::null_mut::<POINT>();
    let mut infohdr: *mut BITMAPINFOHEADER = ::core::ptr::null_mut::<BITMAPINFOHEADER>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
            546 as ::core::ffi::c_int,
            b"(%p %p %p %u) function %04x\n\0" as *const u8 as *const ::core::ffi::c_char,
            hdc,
            ht,
            mr,
            handles,
            (*mr).rdFunction as ::core::ffi::c_int,
        );
    }
    match ((*mr).rdFunction as ::core::ffi::c_int) as u32 {
        META_EOF => {}

        META_DELETEOBJECT => {
            DeleteObject(*(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize));
            *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) = 0 as HGDIOBJ;
        }

        META_SETBKCOLOR => {
            SetBkColor(hdc, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as COLORREF);
        }

        META_SETBKMODE => {
            SetBkMode(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETMAPMODE => {
            SetMapMode(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETROP2 => {
            SetROP2(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETRELABS => {
            SetRelAbs(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETPOLYFILLMODE => {
            SetPolyFillMode(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETSTRETCHBLTMODE => {
            SetStretchBltMode(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
        }

        META_SETTEXTCOLOR => {
            SetTextColor(hdc, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as COLORREF);
        }

        META_SETWINDOWORG => {
            SetWindowOrgEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_SETWINDOWEXT => {
            SetWindowExtEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_SETVIEWPORTORG => {
            SetViewportOrgEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_SETVIEWPORTEXT => {
            SetViewportExtEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_OFFSETWINDOWORG => {
            OffsetWindowOrgEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_SCALEWINDOWEXT => {
            ScaleWindowExtEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_OFFSETVIEWPORTORG => {
            OffsetViewportOrgEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_SCALEVIEWPORTEXT => {
            ScaleViewportExtEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_LINETO => {
            LineTo(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_MOVETO => {
            MoveToEx(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int, ::core::ptr::null_mut());
        }

        META_EXCLUDECLIPRECT => {
            ExcludeClipRect(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_INTERSECTCLIPRECT => {
            IntersectClipRect(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_ARC => {
            Arc(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_ELLIPSE => {
            Ellipse(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_FLOODFILL => {
            FloodFill(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as COLORREF);
        }

        META_PIE => {
            Pie(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_RECTANGLE => {
            Rectangle(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_ROUNDRECT => {
            RoundRect(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_PATBLT => {
            PatBlt(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
        }

        META_SAVEDC => {
            SaveDC(hdc);
        }

        META_SETPIXEL => {
            SetPixel(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as COLORREF);
        }

        META_OFFSETCLIPRGN => {
            OffsetClipRgn(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_TEXTOUT => {
            s1 = (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_short;
            TextOutA(hdc,
                (*(&raw mut (*mr).rdParm as *mut WORD).add((((s1 as ::core::ffi::c_int) + (1 as ::core::ffi::c_int)) >> (1 as ::core::ffi::c_int)) as usize + (2 as usize))) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add((((s1 as ::core::ffi::c_int) + (1 as ::core::ffi::c_int)) >> (1 as ::core::ffi::c_int)) as usize + (1 as usize))) as SHORT as ::core::ffi::c_int,
                (&raw mut (*mr).rdParm as *mut WORD).add(1 as usize) as *mut ::core::ffi::c_char as LPCSTR,
                s1 as ::core::ffi::c_int);
        }

        META_POLYGON => {
            pt = convert_points((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT, (&raw mut (*mr).rdParm as *mut WORD).add(1 as usize) as *mut POINTS);
            if !pt.is_null() {
                Polygon(hdc, pt, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
                HeapFree(GetProcessHeap(), 0 as DWORD, pt as PVOID);
            }
        }

        META_POLYPOLYGON => {
            let mut i: UINT = 0;
            let mut total: UINT = 0;
            let mut counts: *mut SHORT = (&raw mut (*mr).rdParm as *mut WORD).add(1 as usize) as *mut SHORT;

            total = 0 as UINT;
            i = total;
            while i < (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT {
                total = total.wrapping_add(*counts.add(i as usize) as UINT);
                i = i.wrapping_add(1);
            }
            pt = convert_points(total, counts.add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) as *mut POINTS);
            if !pt.is_null() {
                let mut cnt32: *mut INT = HeapAlloc(GetProcessHeap(), 0 as DWORD, ((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SIZE_T) * (::core::mem::size_of::<INT>() as SIZE_T)) as *mut INT;
                if !cnt32.is_null() {
                    i = 0 as UINT;
                    while i < (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT {
                        *cnt32.add(i as usize) = *counts.add(i as usize) as INT;
                        i = i.wrapping_add(1);
                    }
                    PolyPolygon(hdc, pt, cnt32, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
                    HeapFree(GetProcessHeap(), 0 as DWORD, cnt32 as PVOID);
                }
            }
            HeapFree(GetProcessHeap(), 0 as DWORD, pt as PVOID);
        }

        META_POLYLINE => {
            pt = convert_points((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT, (&raw mut (*mr).rdParm as *mut WORD).add(1 as usize) as *mut POINTS);
            if !pt.is_null() {
                Polyline(hdc, pt, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int);
                HeapFree(GetProcessHeap(), 0 as DWORD, pt as PVOID);
            }
        }

        META_RESTOREDC => {
            RestoreDC(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_SELECTOBJECT => {
            SelectObject(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize));
        }

        META_CHORD => {
            Chord(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_CREATEPATTERNBRUSH => {
            match ((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int) as u32 {
                BS_PATTERN => {
                    infohdr = (&raw mut (*mr).rdParm as *mut WORD).add(2 as usize) as *mut BITMAPINFOHEADER;
                    MF_AddHandle(ht, handles,
                        (CreatePatternBrush(CreateBitmap((*infohdr).biWidth as INT,
                            (*infohdr).biHeight as INT,
                            (*infohdr).biPlanes as UINT,
                            (*infohdr).biBitCount as UINT,
                            (&raw mut (*mr).rdParm as *mut WORD).add((::core::mem::size_of::<BITMAPINFOHEADER>() / (2 as usize)) + (4 as usize)) as *const ::core::ffi::c_void))).cast());
                }

                BS_DIBPATTERN => {
                    infohdr = (&raw mut (*mr).rdParm as *mut WORD).add(2 as usize) as *mut BITMAPINFOHEADER;
                    MF_AddHandle(ht, handles, (CreateDIBPatternBrushPt(infohdr as *const ::core::ffi::c_void, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as UINT)).cast());
                }

                _ => {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_ERR,
                            __wine_dbch___default,
                            b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                            b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                            776 as ::core::ffi::c_int,
                            b"META_CREATEPATTERNBRUSH: Unknown pattern type %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                            (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int,
                        );
                    }
                }
            }
        }

        META_CREATEPENINDIRECT => {
            let mut pen: LOGPEN = ::core::mem::zeroed();
            pen.lopnStyle = (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT;
            pen.lopnWidth.x = (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as LONG;
            pen.lopnWidth.y = (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as LONG;
            pen.lopnColor = MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize))) as COLORREF;
            MF_AddHandle(ht, handles, (CreatePenIndirect(&raw mut pen)).cast());
        }

        META_CREATEFONTINDIRECT => {
            let mut font: LOGFONTA = ::core::mem::zeroed();
            font.lfHeight = (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as LONG;
            font.lfWidth = (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as LONG;
            font.lfEscapement = (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as LONG;
            font.lfOrientation = (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as LONG;
            font.lfWeight = (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as LONG;
            font.lfItalic = LOBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)));
            font.lfUnderline = HIBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)));
            font.lfStrikeOut = LOBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)));
            font.lfCharSet = HIBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)));
            font.lfOutPrecision = LOBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)));
            font.lfClipPrecision = HIBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)));
            font.lfQuality = LOBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)));
            font.lfPitchAndFamily = HIBYTE!((*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)));
            memcpy(&raw mut font.lfFaceName as *mut CHAR as *mut ::core::ffi::c_void, (&raw mut (*mr).rdParm as *mut WORD).add(9 as usize) as *const ::core::ffi::c_void, LF_FACESIZE as usize);
            MF_AddHandle(ht, handles, (CreateFontIndirectA(&raw mut font)).cast());
        }

        META_CREATEBRUSHINDIRECT => {
            let mut brush: LOGBRUSH = ::core::mem::zeroed();
            brush.lbStyle = (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT;
            brush.lbColor = MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize))) as COLORREF;
            brush.lbHatch = (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as ULONG_PTR;
            MF_AddHandle(ht, handles, CreateBrushIndirect(&raw mut brush) as HGDIOBJ);
        }

        META_CREATEPALETTE => {
            MF_AddHandle(ht, handles, CreatePalette((&raw mut (*mr).rdParm as *mut WORD as LPLOGPALETTE as *const LOGPALETTE)) as HGDIOBJ);
        }

        META_SETTEXTALIGN => {
            SetTextAlign(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT);
        }

        META_SELECTPALETTE => {
            GDISelectPalette(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as usize) as HPALETTE, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as BOOL);
        }

        META_SETMAPPERFLAGS => {
            SetMapperFlags(hdc, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
        }

        META_REALIZEPALETTE => {
            GDIRealizePalette(hdc);
        }

        META_ESCAPE => {
            match ((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int) as u32 {
                GETSCALINGFACTOR | GETPHYSPAGESIZE | GETPRINTINGOFFSET => {
                    return (FALSE) as BOOL;
                }
                SETABORTPROC => {
                    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                        ros_dbg_log(
                            __WINE_DBCL_FIXME,
                            __wine_dbch___default,
                            b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                            b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                            850 as ::core::ffi::c_int,
                            b"Filtering Escape(SETABORTPROC), possible virus?\n\0" as *const u8 as *const ::core::ffi::c_char,
                        );
                    }
                    return (FALSE) as BOOL;
                }
                _ => {}
            }
            Escape(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as ::core::ffi::c_int, (&raw mut (*mr).rdParm as *mut WORD).add(2 as usize) as LPCSTR, ::core::ptr::null_mut());
        }

        META_EXTTEXTOUT => {
            MF_Play_MetaExtTextOut(hdc, mr);
        }

        META_STRETCHDIB => {
            let mut info: LPBITMAPINFO = (&raw mut (*mr).rdParm as *mut WORD).add(11 as usize) as *mut WORD as LPBITMAPINFO;
            let mut bits: LPSTR = (info as LPSTR).offset(bitmap_info_size(info as *const BITMAPINFO, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) ) as isize);
            StretchDIBits(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(10 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(9 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, bits as *const ::core::ffi::c_void, info as *const BITMAPINFO,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as UINT, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
        }

        META_DIBSTRETCHBLT => {
            let mut info: LPBITMAPINFO = (&raw mut (*mr).rdParm as *mut WORD).add(10 as usize) as *mut WORD as LPBITMAPINFO;
            let mut bits: LPSTR = (info as LPSTR).offset(bitmap_info_size(info as *const BITMAPINFO, DIB_RGB_COLORS as WORD) as isize);
            StretchDIBits(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(9 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int, bits as *const ::core::ffi::c_void, info as *const BITMAPINFO,
                DIB_RGB_COLORS as UINT, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
        }

        META_STRETCHBLT => {
            let mut hdcSrc: HDC = CreateCompatibleDC(hdc);
            let mut hbitmap: HBITMAP = CreateBitmap((*(&raw mut (*mr).rdParm as *mut WORD).add(10 as usize)) as INT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(11 as usize)) as INT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(13 as usize)) as UINT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(14 as usize)) as UINT,
                (&raw mut (*mr).rdParm as *mut WORD).add(15 as usize) as *const ::core::ffi::c_void);
            SelectObject(hdcSrc, hbitmap as HGDIOBJ);
            StretchBlt(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(9 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int,
                hdcSrc, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
            DeleteDC(hdcSrc);
        }

        META_BITBLT => {
            let mut hdcSrc: HDC = CreateCompatibleDC(hdc);
            let mut hbitmap: HBITMAP = CreateBitmap((*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as INT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as INT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(10 as usize)) as UINT,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(11 as usize)) as UINT,
                (&raw mut (*mr).rdParm as *mut WORD).add(12 as usize) as *const ::core::ffi::c_void);
            SelectObject(hdcSrc, hbitmap as HGDIOBJ);
            BitBlt(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int,
                hdcSrc, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!(0 as WORD, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize))) as DWORD);
            DeleteDC(hdcSrc);
        }

        META_CREATEREGION => {
            let mut hrgn: HRGN = CreateRectRgn(0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int, 0 as ::core::ffi::c_int);

            MF_Play_MetaCreateRegion(mr, hrgn);
            MF_AddHandle(ht, handles, hrgn as HGDIOBJ);
        }

        META_FILLREGION => {
            FillRgn(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as usize) as HRGN,
                *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) as HBRUSH);
        }

        META_FRAMEREGION => {
            FrameRgn(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as usize) as HRGN,
                *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as usize) as HBRUSH,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_INVERTREGION => {
            InvertRgn(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) as HRGN);
        }

        META_PAINTREGION => {
            PaintRgn(hdc, *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) as HRGN);
        }

        META_SELECTCLIPREGION => {
            let mut hrgn: HRGN = 0 as HRGN;

            if (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) != 0 {
                hrgn = *(&raw mut (*ht).objectHandle as *mut HGDIOBJ).add((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as usize) as HRGN;
            }
            SelectClipRgn(hdc, hrgn);
        }

        META_DIBCREATEPATTERNBRUSH => {
            MF_AddHandle(ht, handles, CreateDIBPatternBrushPt((&raw mut (*mr).rdParm as *mut WORD).add(2 as usize) as *const ::core::ffi::c_void, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as UINT) as HGDIOBJ);
        }

        META_DIBBITBLT => {
            if (*mr).rdSize > 12 as DWORD {
                let mut info: LPBITMAPINFO = (&raw mut (*mr).rdParm as *mut WORD).add(8 as usize) as *mut WORD as LPBITMAPINFO;
                let mut bits: LPSTR = (info as LPSTR).offset(bitmap_info_size(info as *const BITMAPINFO, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize))) as isize);

                StretchDIBits(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int,
                    (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as SHORT as ::core::ffi::c_int,
                    (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, bits as *const ::core::ffi::c_void, info as *const BITMAPINFO,
                    DIB_RGB_COLORS as UINT, MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
            } else {
                PatBlt(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int,
                    (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as ::core::ffi::c_int,
                    MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize))) as DWORD);
            }
        }

        META_SETTEXTCHAREXTRA => {
            SetTextCharacterExtra(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_SETTEXTJUSTIFICATION => {
            SetTextJustification(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as SHORT as ::core::ffi::c_int);
        }

        META_EXTFLOODFILL => {
            ExtFloodFill(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int,
                MAKELONG!((*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)), (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize))) as COLORREF,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT);
        }

        META_SETDIBTODEV => {
            let mut info: *mut BITMAPINFO = (&raw mut (*mr).rdParm as *mut WORD).add(9 as usize) as *mut WORD as *mut BITMAPINFO;
            let mut bits: *mut ::core::ffi::c_char = (info as *mut ::core::ffi::c_char).offset(bitmap_info_size(info, (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize))) as isize);
            SetDIBitsToDevice(hdc, (*(&raw mut (*mr).rdParm as *mut WORD).add(8 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(7 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(6 as usize)) as SHORT as DWORD, (*(&raw mut (*mr).rdParm as *mut WORD).add(5 as usize)) as SHORT as DWORD,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(4 as usize)) as SHORT as ::core::ffi::c_int, (*(&raw mut (*mr).rdParm as *mut WORD).add(3 as usize)) as SHORT as ::core::ffi::c_int,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(2 as usize)) as UINT, (*(&raw mut (*mr).rdParm as *mut WORD).add(1 as usize)) as UINT, bits as *const ::core::ffi::c_void, info as *const BITMAPINFO,
                (*(&raw mut (*mr).rdParm as *mut WORD).add(0 as usize)) as UINT);
        }

        META_DRAWTEXT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1013 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_DRAWTEXT as ::core::ffi::c_int);
            }
        }
        META_ANIMATEPALETTE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1014 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_ANIMATEPALETTE as ::core::ffi::c_int);
            }
        }
        META_SETPALENTRIES => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1015 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_SETPALENTRIES as ::core::ffi::c_int);
            }
        }
        META_RESIZEPALETTE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1016 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_RESIZEPALETTE as ::core::ffi::c_int);
            }
        }
        META_RESETDC => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1017 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_RESETDC as ::core::ffi::c_int);
            }
        }
        META_STARTDOC => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1018 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_STARTDOC as ::core::ffi::c_int);
            }
        }
        META_STARTPAGE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1019 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_STARTPAGE as ::core::ffi::c_int);
            }
        }
        META_ENDPAGE => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1020 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_ENDPAGE as ::core::ffi::c_int);
            }
        }
        META_ABORTDOC => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1021 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_ABORTDOC as ::core::ffi::c_int);
            }
        }
        META_ENDDOC => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1022 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_ENDDOC as ::core::ffi::c_int);
            }
        }
        META_CREATEBRUSH => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1023 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_CREATEBRUSH as ::core::ffi::c_int);
            }
        }
        META_CREATEBITMAPINDIRECT => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1024 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_CREATEBITMAPINDIRECT as ::core::ffi::c_int);
            }
        }
        META_CREATEBITMAP => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int != 0 {
                ros_dbg_log(__WINE_DBCL_FIXME, __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1025 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord:record type %d not implemented.\n\0" as *const u8 as *const ::core::ffi::c_char,
                    META_CREATEBITMAP as ::core::ffi::c_int);
            }
        }

        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int != 0 {
                ros_dbg_log(
                    __WINE_DBCL_WARN,
                    __wine_dbch___default,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0" as *const u8 as *const ::core::ffi::c_char,
                    b"PlayMetaFileRecord\0" as *const u8 as *const ::core::ffi::c_char,
                    1029 as ::core::ffi::c_int,
                    b"PlayMetaFileRecord: Unknown record type %x\n\0" as *const u8 as *const ::core::ffi::c_char,
                    (*mr).rdFunction as ::core::ffi::c_int,
                );
            }
            return (FALSE) as BOOL;
        }
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn SetMetaFileBitsEx(size: UINT, lpData: *const BYTE) -> HMETAFILE {
    let mh_in: *const METAHEADER = lpData as *const METAHEADER;
    let mut mh_out: *mut METAHEADER = core::ptr::null_mut::<METAHEADER>();

    if (size & 1) != 0 {
        return core::ptr::null_mut();
    }

    if size == 0
        || (*mh_in).mtType as i32 != (METAFILE_MEMORY as i32)
        || (*mh_in).mtVersion as i32 != (MFVERSION as i32)
        || (*mh_in).mtHeaderSize as usize
            != (core::mem::size_of::<METAHEADER>() as usize) / (2 as usize)
    {
        SetLastError(ERROR_INVALID_DATA as DWORD);
        return core::ptr::null_mut();
    }

    mh_out = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        size as SIZE_T,
    ) as *mut METAHEADER;
    if mh_out.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return core::ptr::null_mut();
    }

    memcpy(
        mh_out as *mut core::ffi::c_void,
        mh_in as *const core::ffi::c_void,
        size as usize,
    );
    (*mh_out).mtSize = (size / (2 as UINT)) as DWORD;
    return MF_Create_HMETAFILE(mh_out);
}

#[no_mangle]
pub unsafe extern "system" fn GetMetaFileBitsEx(
    hmf: HMETAFILE,
    nSize: UINT,
    buf: LPVOID,
) -> UINT {
    let mut mh: *mut METAHEADER =
        GDI_GetObjPtr(hmf as HGDIOBJ, OBJ_METAFILE as WORD) as *mut METAHEADER;
    let mut mfSize: UINT;
    let mut mf_copy: BOOL = (FALSE) as i32;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/metafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetMetaFileBitsEx\0" as *const u8 as *const ::core::ffi::c_char,
            1097 as ::core::ffi::c_int,
            b"(%p,%d,%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            hmf,
            nSize,
            buf,
        );
    }
    if mh.is_null() {
        return 0;
    }
    if (*mh).mtType as ::core::ffi::c_int == (METAFILE_DISK as i32) {
        mh = MF_LoadDiskBasedMetaFile(mh);
        if mh.is_null() {
            GDI_ReleaseObj(hmf as HGDIOBJ);
            return 0;
        }
        mf_copy = (TRUE) as i32;
    }
    mfSize = (*mh).mtSize.wrapping_mul(2) as UINT;
    if !buf.is_null() {
        if mfSize > nSize {
            mfSize = nSize;
        }
        memmove(
            buf as *mut ::core::ffi::c_void,
            mh as *const ::core::ffi::c_void,
            mfSize as usize,
        );
    }
    if mf_copy != 0 {
        HeapFree(
            GetProcessHeap(),
            0,
            mh as PVOID,
        );
    }
    GDI_ReleaseObj(hmf as HGDIOBJ);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/gdi/gdi32/wine/metafile.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetMetaFileBitsEx\0" as *const u8 as *const ::core::ffi::c_char,
            1117 as ::core::ffi::c_int,
            b"returning size %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            mfSize,
        );
    }
    return mfSize;
}

unsafe extern "C" fn add_mf_comment(hdc: HDC, emf: HENHMETAFILE) -> BOOL {
    let mut size: DWORD = GetEnhMetaFileBits(emf, 0, core::ptr::null_mut()) as DWORD;
    let mut i: DWORD;
    let mut bits: *mut BYTE;
    let mut chunk_data: *mut BYTE;
    let mut chunk: *mut emf_in_wmf_comment = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;
    let max_chunk_size: DWORD = 0x2000;

    if size == 0 {
        return (FALSE) as BOOL;
    }
    bits = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut BYTE;
    chunk_data = bits;
    if bits.is_null() {
        return (FALSE) as BOOL;
    }
    'end: {
        if !(GetEnhMetaFileBits(emf, size as UINT, bits as LPBYTE) == 0) {
            break 'end;
        }

        chunk = HeapAlloc(
            GetProcessHeap(),
            0,
            (FIELD_OFFSET!(emf_in_wmf_comment, emf_data[max_chunk_size as usize]) as usize) as SIZE_T,
        ) as *mut emf_in_wmf_comment;
        if chunk.is_null() {
            break 'end;
        }

        (*chunk).magic = WMFC_MAGIC;
        (*chunk).comment_type = 0x1;
        (*chunk).version = 0x00010000;
        (*chunk).checksum = 0; /* We fixup the first chunk's checksum before returning from GetWinMetaFileBits */
        (*chunk).flags = 0;
        (*chunk).num_chunks = (size + max_chunk_size - 1) / max_chunk_size;
        (*chunk).chunk_size = max_chunk_size;
        (*chunk).remaining_size = size;
        (*chunk).emf_size = size;

        i = 0;
        while i < (*chunk).num_chunks {
            if i == (*chunk).num_chunks - 1 {
                /* last chunk */
                (*chunk).chunk_size = (*chunk).remaining_size;
            }

            (*chunk).remaining_size -= (*chunk).chunk_size;
            memcpy(
                core::ptr::addr_of_mut!((*chunk).emf_data) as *mut core::ffi::c_void,
                chunk_data as *const core::ffi::c_void,
                (*chunk).chunk_size as usize,
            );
            chunk_data = chunk_data.offset((*chunk).chunk_size as isize);

            if Escape(
                hdc,
                (MFCOMMENT as i32),
                (FIELD_OFFSET!(emf_in_wmf_comment, emf_data[(*chunk).chunk_size as usize]) as usize) as LONG as core::ffi::c_int,
                chunk as *mut core::ffi::c_char as LPCSTR,
                core::ptr::null_mut(),
            ) == 0
            {
                break 'end;
            }
            i += 1;
        }
        ret = (TRUE) as i32;
    }
    HeapFree(GetProcessHeap(), 0, chunk as PVOID);
    HeapFree(GetProcessHeap(), 0, bits as PVOID);
    return ret;
}

unsafe extern "C" fn muldiv(mut m1: INT, mut m2: INT, mut d: INT) -> INT {
    let mut ret: LONGLONG = 0;
    ret = ((m1 as LONGLONG) * (m2 as LONGLONG)
        + ((d / 2) as LONGLONG))
        / (d as LONGLONG); /* Always add d/2 even if ret will be -ve */

    if (m1 as LONGLONG) * (m2 as LONGLONG) * (2 as LONGLONG)
        == ((2 as LONGLONG) * ret - (1 as LONGLONG))
            * (d as LONGLONG)
    /* If the answer is exactly n.5 round towards zero */
    {
        if ret > (0 as LONGLONG) {
            ret -= 1;
        } else {
            ret += 1;
        }
    }
    return ret as INT;
}

unsafe extern "C" fn set_window(mut hdc: HDC, mut emf: HENHMETAFILE, mut ref_dc: HDC, mut map_mode: INT) -> BOOL {
    let mut header: ENHMETAHEADER = tagENHMETAHEADER {
        iType: 0,
        nSize: 0,
        rclBounds: _RECTL { left: 0, top: 0, right: 0, bottom: 0 },
        rclFrame: _RECTL { left: 0, top: 0, right: 0, bottom: 0 },
        dSignature: 0,
        nVersion: 0,
        nBytes: 0,
        nRecords: 0,
        nHandles: 0,
        sReserved: 0,
        nDescription: 0,
        offDescription: 0,
        nPalEntries: 0,
        szlDevice: tagSIZE { cx: 0, cy: 0 },
        szlMillimeters: tagSIZE { cx: 0, cy: 0 },
        cbPixelFormat: 0,
        offPixelFormat: 0,
        bOpenGL: 0,
        szlMicrometers: tagSIZE { cx: 0, cy: 0 },
    };
    let mut horz_res: INT = 0;
    let mut vert_res: INT = 0;
    let mut horz_size: INT = 0;
    let mut vert_size: INT = 0;
    let mut pt: POINT = tagPOINT { x: 0, y: 0 };

    if GetEnhMetaFileHeader(
        emf,
        core::mem::size_of::<ENHMETAHEADER>() as UINT,
        core::ptr::addr_of_mut!(header),
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    horz_res = GetDeviceCaps(ref_dc, (HORZRES as i32)) as INT;
    vert_res = GetDeviceCaps(ref_dc, (VERTRES as i32)) as INT;
    horz_size = GetDeviceCaps(ref_dc, (HORZSIZE as i32)) as INT;
    vert_size = GetDeviceCaps(ref_dc, (VERTSIZE as i32)) as INT;

    match (map_mode) as u32 {
        MM_TEXT | MM_ISOTROPIC | MM_ANISOTROPIC => {
            pt.y = muldiv(
                header.rclFrame.top as INT,
                vert_res,
                vert_size * 100 as INT,
            ) as LONG;
            pt.x = muldiv(
                header.rclFrame.left as INT,
                horz_res,
                horz_size * 100 as INT,
            ) as LONG;
        }
        MM_LOMETRIC => {
            pt.y = (muldiv(
                -header.rclFrame.top as INT,
                1 as INT,
                10 as INT,
            ) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as LONG;
            pt.x = muldiv(
                header.rclFrame.left as INT,
                1 as INT,
                10 as INT,
            ) as LONG;
        }
        MM_HIMETRIC => {
            pt.y = -header.rclFrame.top + 1 as LONG;
            pt.x = if header.rclFrame.left >= 0 as LONG {
                header.rclFrame.left
            } else {
                header.rclFrame.left + 1 as LONG
            };
        }
        MM_LOENGLISH => {
            pt.y = (muldiv(
                -header.rclFrame.top as INT,
                10 as INT,
                254 as INT,
            ) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as LONG;
            pt.x = muldiv(
                header.rclFrame.left as INT,
                10 as INT,
                254 as INT,
            ) as LONG;
        }
        MM_HIENGLISH => {
            pt.y = (muldiv(
                -header.rclFrame.top as INT,
                100 as INT,
                254 as INT,
            ) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as LONG;
            pt.x = muldiv(
                header.rclFrame.left as INT,
                100 as INT,
                254 as INT,
            ) as LONG;
        }
        MM_TWIPS => {
            pt.y = (muldiv(
                -header.rclFrame.top as INT,
                72 as INT * 20 as INT,
                2540 as INT,
            ) as ::core::ffi::c_int
                + 1 as ::core::ffi::c_int) as LONG;
            pt.x = muldiv(
                header.rclFrame.left as INT,
                72 as INT * 20 as INT,
                2540 as INT,
            ) as LONG;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"set_window\0" as *const u8 as *const ::core::ffi::c_char,
                    1243 as ::core::ffi::c_int,
                    b"Unknown map mode %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                    map_mode,
                );
            }
            return (FALSE) as BOOL;
        }
    }

    SetWindowOrgEx(
        hdc,
        pt.x as ::core::ffi::c_int,
        pt.y as ::core::ffi::c_int,
        core::ptr::null_mut::<tagPOINT>(),
    );

    pt.x = muldiv(
        (header.rclFrame.right - header.rclFrame.left) as INT,
        horz_res,
        horz_size * 100 as INT,
    ) as LONG;
    pt.y = muldiv(
        (header.rclFrame.bottom - header.rclFrame.top) as INT,
        vert_res,
        vert_size * 100 as INT,
    ) as LONG;
    SetWindowExtEx(
        hdc,
        pt.x as ::core::ffi::c_int,
        pt.y as ::core::ffi::c_int,
        core::ptr::null_mut::<tagSIZE>(),
    );
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn GetWinMetaFileBits(
    hemf: HENHMETAFILE,
    cbBuffer: UINT,
    lpbBuffer: LPBYTE,
    map_mode: INT,
    hdcRef: HDC,
) -> UINT {
    let mut hdcmf: HDC = core::ptr::null_mut();
    let mut hmf: HMETAFILE = core::ptr::null_mut();
    let mut ret: UINT = 0;
    let mut full_size: UINT = 0;
    let mut rc: RECT = core::mem::zeroed();

    GetClipBox(hdcRef, core::ptr::addr_of_mut!(rc));

    hdcmf = CreateMetaFileW(core::ptr::null());

    add_mf_comment(hdcmf, hemf);
    SetMapMode(hdcmf, map_mode);
    'error: {
        if set_window(hdcmf, hemf, hdcRef, map_mode) == 0 {
            break 'error;
        }

        PlayEnhMetaFile(hdcmf, hemf, core::ptr::addr_of_mut!(rc));
        hmf = CloseMetaFile(hdcmf);
        full_size = GetMetaFileBitsEx(hmf, 0, core::ptr::null_mut());
        ret = GetMetaFileBitsEx(hmf, cbBuffer, lpbBuffer as LPVOID);
        DeleteMetaFile(hmf);

        if ret != 0 && ret == full_size && !lpbBuffer.is_null() {
            let mut checksum: WORD = 0;
            let mut comment_rec: *mut METARECORD = lpbBuffer
                .offset(core::mem::size_of::<METAHEADER>() as isize)
                as *mut METARECORD;
            let mut i: UINT = 0;

            i = 0;
            while i < full_size / 2 {
                checksum = (checksum as ::core::ffi::c_int
                    + *(lpbBuffer as *mut WORD).offset(i as isize) as ::core::ffi::c_int)
                    as WORD;
                i = i.wrapping_add(1);
            }
            (*core::ptr::addr_of_mut!((*comment_rec).rdParm[0]).add(8 as usize)) =
                ((!(checksum as ::core::ffi::c_int)) + 1) as WORD;
        }
        return ret;
    }

    DeleteMetaFile(CloseMetaFile(hdcmf));
    return 0;
}

unsafe extern "C" fn MF_Play_MetaCreateRegion(mr: *mut METARECORD, hrgn: HRGN) -> BOOL {
    let mut band: WORD = 0;
    let mut pair: WORD = 0;
    let mut start: *mut WORD = core::ptr::null_mut();
    let mut end: *mut WORD = core::ptr::null_mut();
    let mut y0: INT16 = 0;
    let mut y1: INT16 = 0;
    let hrgn2: HRGN = CreateRectRgn(
        0 as i32,
        0 as i32,
        0 as i32,
        0 as i32,
    );

    band = 0 as WORD;
    start = (core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
        .offset(11 as isize);

    while (band as i32)
        < (*(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(5 as isize)) as i32
    {
        if (*start as i32) / (2 as i32)
            != ((*start as i32) + (1 as i32)) / (2 as i32)
        {
            if (*__wine_dbch___default).flags as i32
                & (1i32) << (__WINE_DBCL_WARN as i32)
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"MF_Play_MetaCreateRegion\0" as *const u8 as *const core::ffi::c_char,
                    1341i32,
                    b"Delimiter not even.\n\0" as *const u8 as *const core::ffi::c_char,
                );
            }
            DeleteObject(hrgn2 as HGDIOBJ);
            return FALSE as BOOL;
        }

        end = start
            .offset((*start as i32) as isize)
            .offset(3 as isize);
        if end > (mr as *mut WORD).offset((*mr).rdSize as isize) {
            if (*__wine_dbch___default).flags as i32
                & (1i32) << (__WINE_DBCL_WARN as i32)
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"MF_Play_MetaCreateRegion\0" as *const u8 as *const core::ffi::c_char,
                    1348i32,
                    b"End points outside record.\n\0" as *const u8 as *const core::ffi::c_char,
                );
            }
            DeleteObject(hrgn2 as HGDIOBJ);
            return FALSE as BOOL;
        }

        if (*start as i32) != (*end as i32) {
            if (*__wine_dbch___default).flags as i32
                & (1i32) << (__WINE_DBCL_WARN as i32)
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/gdi/gdi32/wine/metafile.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"MF_Play_MetaCreateRegion\0" as *const u8 as *const core::ffi::c_char,
                    1354i32,
                    b"Mismatched delimiters.\n\0" as *const u8 as *const core::ffi::c_char,
                );
            }
            DeleteObject(hrgn2 as HGDIOBJ);
            return FALSE as BOOL;
        }

        y0 = *(start.offset(1 as isize) as *mut INT16);
        y1 = *(start.offset(2 as isize) as *mut INT16);
        pair = 0 as WORD;
        while (pair as i32) < (*start as i32) / (2 as i32) {
            SetRectRgn(
                hrgn2,
                *(start
                    .offset(3 as isize)
                    .offset((2 as i32 * pair as i32) as isize)
                    as *mut INT16) as i32,
                y0 as i32,
                *(start
                    .offset(4 as isize)
                    .offset((2 as i32 * pair as i32) as isize)
                    as *mut INT16) as i32,
                y1 as i32,
            );
            CombineRgn(hrgn, hrgn, hrgn2, RGN_OR as i32);
            pair = pair.wrapping_add(1);
        }

        band = band.wrapping_add(1);
        start = end.offset(1 as isize);
    }

    DeleteObject(hrgn2 as HGDIOBJ);
    return TRUE as BOOL;
}

unsafe extern "C" fn MF_Play_MetaExtTextOut(mut hdc: HDC, mut mr: *mut METARECORD) -> BOOL {
    let mut dx: *mut INT = ::core::ptr::null_mut::<INT>();
    let mut i: ::core::ffi::c_int = 0;
    let mut dxx: *mut SHORT = ::core::ptr::null_mut::<SHORT>();
    let mut sot: LPSTR = ::core::ptr::null_mut::<CHAR>();
    let mut len: DWORD = 0;
    let mut s1: WORD = 0;
    let mut rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut isrect: BOOL = (*(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
        .offset(3 as isize) as BOOL)
        & ((ETO_OPAQUE | ETO_CLIPPED)) as i32;
    s1 = *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
        .offset(2 as isize);
    len = (::core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add(
            (((s1 as ::core::ffi::c_int + 1) >> 1)
                * 2) as usize,
        )
        .wrapping_add(
            (2 as usize)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as usize),
        )
        .wrapping_add(::core::mem::size_of::<UINT16>() as usize)
        .wrapping_add(if isrect != 0 {
            (4 as usize).wrapping_mul(::core::mem::size_of::<SHORT>() as usize)
        } else {
            0 as usize
        }) as DWORD;
    sot = (core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
        .offset(4 as isize) as *mut WORD as LPSTR;
    if isrect != 0 {
        rect.left = *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(4 as isize) as SHORT as LONG;
        rect.top = *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(5 as isize) as SHORT as LONG;
        rect.right = *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(6 as isize) as SHORT as LONG;
        rect.bottom = *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(7 as isize) as SHORT as LONG;
        sot = sot.offset(
            (4 as usize)
                .wrapping_mul(::core::mem::size_of::<SHORT>() as usize)
                as isize,
        );
    }
    if (*mr).rdSize == len.wrapping_div(2 as DWORD) {
        dxx = ::core::ptr::null_mut::<SHORT>();
    } else if (*mr).rdSize as usize
        == (len as usize)
            .wrapping_add(
                (s1 as usize).wrapping_mul(::core::mem::size_of::<INT16>() as usize),
            )
            .wrapping_div(2 as usize)
    {
        dxx = sot.offset(
            (((s1 as ::core::ffi::c_int + 1) >> 1)
                * 2) as isize,
        ) as *mut SHORT;
        dx = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (s1 as SIZE_T).wrapping_mul(::core::mem::size_of::<INT>() as SIZE_T),
        ) as *mut INT;
        if !dx.is_null() {
            i = 0 as ::core::ffi::c_int;
            while i < s1 as ::core::ffi::c_int {
                *dx.offset(i as isize) = *dxx.offset(i as isize) as INT;
                i += 1;
            }
        }
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MF_Play_MetaExtTextOut\0" as *const u8 as *const ::core::ffi::c_char,
                1414 as ::core::ffi::c_int,
                b"%s  len: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                sot,
                (*mr).rdSize,
            );
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MF_Play_MetaExtTextOut\0" as *const u8 as *const ::core::ffi::c_char,
                1416 as ::core::ffi::c_int,
                b"Please report: ExtTextOut len=%d slen=%d rdSize=%d opt=%04x\n\0"
                    as *const u8 as *const ::core::ffi::c_char,
                len,
                s1 as ::core::ffi::c_int,
                (*mr).rdSize,
                *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
                    .offset(0 as isize) as ::core::ffi::c_int,
            );
        }
        dxx = ::core::ptr::null_mut::<SHORT>();
    }
    ExtTextOutA(
        hdc,
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(1 as isize) as SHORT as ::core::ffi::c_int,
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(0 as isize) as SHORT as ::core::ffi::c_int,
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(3 as isize) as UINT,
        core::ptr::addr_of_mut!(rect),
        sot as LPCSTR,
        s1 as UINT,
        dx,
    );
    if !dx.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/gdi/gdi32/wine/metafile.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"MF_Play_MetaExtTextOut\0" as *const u8 as *const ::core::ffi::c_char,
                1428 as ::core::ffi::c_int,
                b"%s  len: %d  dx0: %d\n\0" as *const u8 as *const ::core::ffi::c_char,
                sot,
                (*mr).rdSize,
                *dx.offset(0 as ::core::ffi::c_int as isize),
            );
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            dx as PVOID,
        );
    }
    return (TRUE) as BOOL;
}
