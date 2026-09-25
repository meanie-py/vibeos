use win32k_sys::win32k::*;
// ntdll Rtl*/Zw* exports come from the user32-internal substrate, NOT ntkrnl_sys
// (which is the KERNEL FFI crate and is not a dependency here). The `CR` import
// the translator emitted was a bindgen mis-resolution of the local `#define CR 13`
// (carriage-return byte in the PNG signature) — reproduced inline as a local const
// in DIB_GetBitmapInfo, so no import is needed.
use crate::internal::{RtlInitUnicodeString, ZwCallbackReturn, LOWORD};
// NtUserx* multiplexer wrappers (inline forwards to NtUserCall*); the plain NtUser*
// syscalls + all the win32 types come from win32k_sys::win32k::*.
use crate::ntwrapper::{
    GetDC, NtUserxCreateEmptyCurObject, NtUserxGetCursorPos, NtUserxSetCursorPos,
    NtUserxShowCursor, SetCursor,
};
use core::ffi::c_void;

// ---- kernel32.dll exports (WINAPI = stdcall) ----
extern "system" {
    fn GetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: usize) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn SetLastError(dwErrCode: DWORD);
    fn GetLastError() -> DWORD;
}
// wsprintfW is __cdecl variadic (variadic + "system" is unstable) → _wsprintfW.
extern "C" {
    fn wsprintfW(lpOut: LPWSTR, lpFmt: LPCWSTR, ...) -> core::ffi::c_int;
}
extern "system" {
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD;
    fn GetModuleHandleExW(dwFlags: DWORD, lpModuleName: LPCWSTR, phModule: *mut HMODULE) -> BOOL;
    fn FindResourceW(hModule: HMODULE, lpName: LPCWSTR, lpType: LPCWSTR) -> HRSRC;
    fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    fn LockResource(hResData: HGLOBAL) -> LPVOID;
    fn FreeResource(hResData: HGLOBAL) -> BOOL;
    fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;
    fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn MultiByteToWideChar(CodePage: u32, dwFlags: DWORD, lpMultiByteStr: LPCSTR, cbMultiByte: core::ffi::c_int, lpWideCharStr: LPWSTR, cchWideChar: core::ffi::c_int) -> core::ffi::c_int;
    fn CreateFileW( lpFileName: LPCWSTR, dwDesiredAccess: u32, dwShareMode: u32, lpSecurityAttributes: *mut core::ffi::c_void, dwCreationDisposition: u32, dwFlagsAndAttributes: u32, hTemplateFile: *mut core::ffi::c_void, ) -> *mut core::ffi::c_void;
    fn CreateFileMappingW( hFile: *mut core::ffi::c_void, lpFileMappingAttributes: *mut core::ffi::c_void, flProtect: u32, dwMaximumSizeHigh: u32, dwMaximumSizeLow: u32, lpName: LPCWSTR, ) -> *mut core::ffi::c_void;
    fn MapViewOfFile( hFileMappingObject: *mut core::ffi::c_void, dwDesiredAccess: u32, dwFileOffsetHigh: u32, dwFileOffsetLow: u32, dwNumberOfBytesToMap: usize, ) -> *mut core::ffi::c_void;
    fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> i32;
    fn CloseHandle(hObject: *mut core::ffi::c_void) -> i32;
    fn GetFileSize(hFile: *mut core::ffi::c_void, lpFileSizeHigh: *mut u32) -> u32;
    static User32Instance: HINSTANCE;
}

// ---- gdi32.dll / user32.dll exports (WINAPI = stdcall) ----
// NOTE: CreateIconFromResourceEx is NOT imported — it's a real own-export DEFINED
// below (the translator dropped its body; reconstructed faithfully from the C).
extern "system" {
    fn CreateDCW(pwszDriver: LPCWSTR, pwszDevice: LPCWSTR, pszPort: LPCWSTR, pdm: *const core::ffi::c_void) -> HDC;
    fn CreateICW(pwszDriver: LPCWSTR, pwszDevice: LPCWSTR, pszPort: LPCWSTR, pdm: *const core::ffi::c_void) -> HDC;
    fn CreateCompatibleDC(hdc: HDC) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn DeleteObject(ho: HGDIOBJ) -> BOOL;
    fn GetObjectW(h: *mut core::ffi::c_void, c: i32, pv: LPVOID) -> i32;
    fn GetDIBits(hdc: HDC, hbm: HBITMAP, start: u32, cLines: u32, lpvBits: LPVOID, lpbmi: *mut BITMAPINFO, usage: u32) -> i32;
    fn CreateDIBitmap(hdc: HDC, pbmih: *const BITMAPINFOHEADER, flInit: u32, pjBits: *const core::ffi::c_void, pbmi: *const BITMAPINFO, iUsage: u32) -> HBITMAP;
    fn CreateDIBSection(hdc: HDC, pbmi: *const BITMAPINFO, usage: u32, ppvBits: *mut LPVOID, hSection: HANDLE, offset: DWORD) -> HBITMAP;
    fn SelectObject(hdc: HDC, h: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn StretchDIBits(hdc: HDC, xDest: i32, yDest: i32, DestWidth: i32, DestHeight: i32, xSrc: i32, ySrc: i32, SrcWidth: i32, SrcHeight: i32, lpBits: *const core::ffi::c_void, lpbmi: *const BITMAPINFO, iUsage: u32, rop: u32) -> i32;
    fn StretchBlt(hdcDest: HDC, xDest: i32, yDest: i32, wDest: i32, hDest: i32, hdcSrc: HDC, xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32, rop: u32) -> i32;
    fn CreateBitmap(nWidth: i32, nHeight: i32, nPlanes: u32, nBitCount: u32, lpBits: *const core::ffi::c_void) -> HBITMAP;
    fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
    fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
    fn GetSysColor(nIndex: core::ffi::c_int) -> COLORREF;
    // user32 own-export (defined in dc.rs) — regular import, not a forwarder.
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> INT;
}

// ---- CRT cdecl helpers (resolve via the C runtime) ----
extern "C" {
    fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> i32;
    fn memcpy(dst: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// ---- libpng.dll — plain cdecl exports, delay-imported on every arch (the arm64
//      NULL-stubs retired 2026-09-25 with delay-load). Opaque pointers per png.h.
type png_structp = *mut c_void;
type png_infop = *mut c_void;
type png_bytep = *mut u8;
type png_bytepp = *mut *mut u8;
#[allow(non_camel_case_types)]
type png_uint_32 = u32;
extern "C" {
    fn png_create_info_struct(png_ptr: png_structp) -> png_infop;
    fn png_create_read_struct(user_png_ver: *const core::ffi::c_char, error_ptr: *mut c_void, error_fn: *mut c_void, warn_fn: *mut c_void) -> png_structp;
    fn png_destroy_read_struct(png_ptr_ptr: *mut png_structp, info_ptr_ptr: *mut png_infop, end_info_ptr_ptr: *mut png_infop);
    fn png_error(png_ptr: png_structp, error_message: *const core::ffi::c_char);
    fn png_free(png_ptr: png_structp, ptr: *mut c_void);
    fn png_get_IHDR(png_ptr: png_structp, info_ptr: png_infop, width: *mut png_uint_32, height: *mut png_uint_32, bit_depth: *mut core::ffi::c_int, color_type: *mut core::ffi::c_int, interlace_method: *mut core::ffi::c_int, compression_method: *mut core::ffi::c_int, filter_method: *mut core::ffi::c_int) -> png_uint_32;
    fn png_get_channels(png_ptr: png_structp, info_ptr: png_infop) -> u8;
    fn png_get_color_type(png_ptr: png_structp, info_ptr: png_infop) -> u8;
    fn png_get_io_ptr(png_ptr: png_structp) -> *mut c_void;
    fn png_get_rowbytes(png_ptr: png_structp, info_ptr: png_infop) -> usize;
    fn png_malloc(png_ptr: png_structp, size: usize) -> *mut c_void;
    fn png_read_end(png_ptr: png_structp, info_ptr: png_infop);
    fn png_read_image(png_ptr: png_structp, image: png_bytepp);
    fn png_read_info(png_ptr: png_structp, info_ptr: png_infop);
    fn png_read_update_info(png_ptr: png_structp, info_ptr: png_infop);
    fn png_set_gray_to_rgb(png_ptr: png_structp);
    fn png_set_palette_to_rgb(png_ptr: png_structp);
    fn png_set_read_fn(png_ptr: png_structp, io_ptr: *mut c_void, read_data_fn: png_rw_ptr);
    fn png_set_rows(png_ptr: png_structp, info_ptr: png_infop, row_pointers: png_bytepp);
    fn png_set_scale_16(png_ptr: png_structp);
    fn png_set_sig_bytes(png_ptr: png_structp, num_bytes: core::ffi::c_int);
    // png_check_sig(sig,n) is a macro: !png_sig_cmp((sig),0,(n)) — declare the real fn.
    fn png_sig_cmp(sig: *const u8, start: usize, num_to_check: usize) -> core::ffi::c_int;
}

/// libpng `png_rw_ptr` read callback type (cdecl).
type png_rw_ptr = unsafe extern "C" fn(png_ptr: png_structp, data: png_bytep, length: usize);

/// `PNG_LIBPNG_VER_STRING` (#define "1.6.39") — first arg to png_create_read_struct.
const PNG_LIBPNG_VER_STRING: *const core::ffi::c_char =
    b"1.6.39\0".as_ptr() as *const core::ffi::c_char;

/// `png_check_sig(sig, n)` := `!png_sig_cmp(sig, 0, n)` — a wingdi-style macro.
#[inline]
unsafe fn png_check_sig(sig: *const u8, n: usize) -> BOOL {
    (png_sig_cmp(sig, 0, n) == 0) as BOOL
}

// ---- libpng PNG_COLOR_TYPE_* (png.h #defines; not in win32k-sys) ----
const PNG_COLOR_MASK_PALETTE: u8 = 1;
const PNG_COLOR_MASK_COLOR: u8 = 2;
const PNG_COLOR_MASK_ALPHA: u8 = 4;
const PNG_COLOR_TYPE_GRAY: u8 = 0;
const PNG_COLOR_TYPE_PALETTE: u8 = PNG_COLOR_MASK_COLOR | PNG_COLOR_MASK_PALETTE;
const PNG_COLOR_TYPE_GRAY_ALPHA: u8 = PNG_COLOR_MASK_ALPHA;

// ---- winuser.h / winnt.h constants not in win32k-sys ----
const STATUS_SUCCESS: NTSTATUS = 0;
const INVALID_HANDLE_VALUE: isize = -1;
const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x00000002;
const CP_ACP: u32 = 0;
/// winuser.h: valid CopyImage flags mask.
const COPYIMAGE_VALID_FLAGS: UINT = LR_DEFAULTCOLOR | LR_DEFAULTSIZE | LR_LOADFROMFILE
    | LR_CREATEDIBSECTION | LR_COPYRETURNORG | LR_COPYDELETEORG | LR_LOADTRANSPARENT
    | LR_LOADMAP3DCOLORS | LR_COLOR | LR_MONOCHROME | LR_COPYFROMRESOURCE | LR_SHARED;
const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: DWORD = 0x00000004;
const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: DWORD = 0x00000002;

// RT_* resource type ids — winuser.h MAKEINTRESOURCE(n) (pointer-as-int LPCWSTR).
const RT_BITMAP: LPCWSTR = 2usize as LPCWSTR;
const RT_ICON: LPCWSTR = 3usize as LPCWSTR;
const RT_CURSOR: LPCWSTR = 1usize as LPCWSTR;
const RT_GROUP_ICON: LPCWSTR = 14usize as LPCWSTR; // RT_ICON + 11 (DIFFERENCE)
const RT_GROUP_CURSOR: LPCWSTR = 12usize as LPCWSTR; // RT_CURSOR + 11 (DIFFERENCE)

// IDC_* standard cursor ids — winuser.h MAKEINTRESOURCEW(n).
const IDC_ARROW: LPCWSTR = 32512usize as LPCWSTR;
const IDC_IBEAM: LPCWSTR = 32513usize as LPCWSTR;
const IDC_WAIT: LPCWSTR = 32514usize as LPCWSTR;
const IDC_CROSS: LPCWSTR = 32515usize as LPCWSTR;
const IDC_UPARROW: LPCWSTR = 32516usize as LPCWSTR;
const IDC_SIZE: LPCWSTR = 32640usize as LPCWSTR;
const IDC_ICON: LPCWSTR = 32641usize as LPCWSTR;
const IDC_SIZENWSE: LPCWSTR = 32642usize as LPCWSTR;
const IDC_SIZENESW: LPCWSTR = 32643usize as LPCWSTR;
const IDC_SIZEWE: LPCWSTR = 32644usize as LPCWSTR;
const IDC_SIZENS: LPCWSTR = 32645usize as LPCWSTR;
const IDC_SIZEALL: LPCWSTR = 32646usize as LPCWSTR;
const IDC_NO: LPCWSTR = 32648usize as LPCWSTR;
const IDC_HAND: LPCWSTR = 32649usize as LPCWSTR;
const IDC_APPSTARTING: LPCWSTR = 32650usize as LPCWSTR;
const IDC_HELP: LPCWSTR = 32651usize as LPCWSTR;

// ---- wingdi.h color macros (not functions) ----
#[inline]
fn RGB(r: u8, g: u8, b: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16)
}
#[inline]
fn RGBA(r: u8, g: u8, b: u8, a: u8) -> COLORREF {
    (r as COLORREF) | ((g as COLORREF) << 8) | ((b as COLORREF) << 16) | ((a as COLORREF) << 24)
}
#[inline]
fn GetRValue(rgb: COLORREF) -> u8 {
    (rgb & 0xff) as u8
}
#[inline]
fn GetGValue(rgb: COLORREF) -> u8 {
    ((rgb >> 8) & 0xff) as u8
}
#[inline]
fn GetBValue(rgb: COLORREF) -> u8 {
    ((rgb >> 16) & 0xff) as u8
}

/// winuser.h `MAKEINTRESOURCEW(i)` — pack a resource id into a (const) LPWSTR.
/// Returns LPCWSTR since both call sites pass it to LPCWSTR params (FindResourceW,
/// LoadImageW); `*mut` would not coerce to `*const` at those args.
#[inline]
fn MAKEINTRESOURCEW(i: DWORD) -> LPCWSTR {
    (i & 0xffff) as usize as LPCWSTR
}

/// wine/winuser.h `IS_INTRESOURCE(p)` — pointer with high bits clear is an int-resource.
#[inline]
unsafe fn IS_INTRESOURCE(p: *const c_void) -> BOOL {
    ((p as usize) >> 16 == 0) as BOOL
}

/// `RtlCopyMemory`/`CopyMemory` (winnt.h macros over memcpy).
#[inline]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    memcpy(dst, src, n);
}
#[inline]
unsafe fn CopyMemory(dst: LPVOID, src: LPCVOID, n: usize) {
    memcpy(dst, src, n);
}

/// `abs()` — libc cdecl; used for icon size scoring.
#[inline]
fn abs(i: core::ffi::c_int) -> core::ffi::c_int {
    i.wrapping_abs()
}

// ReactOS WINE_*_DEBUG_CHANNEL trace macros — no-ops in the cohab build.

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _CURSORICONFILEDIRENTRY {
    pub bWidth: BYTE,
    pub bHeight: BYTE,
    pub bColorCount: BYTE,
    pub bReserved: BYTE,
    pub __bindgen_anon_1: _CURSORICONFILEDIRENTRY__bindgen_ty_1,
    pub __bindgen_anon_2: _CURSORICONFILEDIRENTRY__bindgen_ty_2,
    pub dwDIBSize: DWORD,
    pub dwDIBOffset: DWORD,
}

#[repr(C)]
pub union _CURSORICONFILEDIRENTRY__bindgen_ty_1 {
    pub wPlanes: WORD,
    pub xHotspot: WORD,
}

#[repr(C)]
pub union _CURSORICONFILEDIRENTRY__bindgen_ty_2 {
    pub wBitCount: WORD,
    pub yHotspot: WORD,
}

pub type CURSORICONFILEDIRENTRY = _CURSORICONFILEDIRENTRY;

#[repr(C)]
pub struct _CURSORICONFILEDIR {
    pub idReserved: WORD,
    pub idType: WORD,
    pub idCount: WORD,
    pub idEntries: [CURSORICONFILEDIRENTRY; 1],
}

pub type CURSORICONFILEDIR = _CURSORICONFILEDIR;

pub const PNG_CHECK_SIG_SIZE: u32 = 8;

#[repr(C)]
pub struct _PNG_READER_STATE {
    pub buffer: png_bytep,
    pub bufSize: usize,
    pub currentPos: usize,
}

pub type PNG_READER_STATE = _PNG_READER_STATE;

pub static DISPLAYW: [WCHAR; 8] = [
    b'D' as WCHAR,
    b'I' as WCHAR,
    b'S' as WCHAR,
    b'P' as WCHAR,
    b'L' as WCHAR,
    b'A' as WCHAR,
    b'Y' as WCHAR,
    0,
];

pub const ANI_RIFF_ID: DWORD = (b'R' as DWORD) | ((b'I' as DWORD) << 8) | ((b'F' as DWORD) << 16) | ((b'F' as DWORD) << 24);

pub const ANI_LIST_ID: DWORD = (b'L' as DWORD) | ((b'I' as DWORD) << 8) | ((b'S' as DWORD) << 16) | ((b'T' as DWORD) << 24);

pub const ANI_ACON_ID: DWORD = (b'A' as DWORD) | ((b'C' as DWORD) << 8) | ((b'O' as DWORD) << 16) | ((b'N' as DWORD) << 24);

pub const ANI_anih_ID: DWORD = (b'a' as DWORD) | ((b'n' as DWORD) << 8) | ((b'i' as DWORD) << 16) | ((b'h' as DWORD) << 24);

pub const ANI_seq__ID: DWORD = (b's' as DWORD) | ((b'e' as DWORD) << 8) | ((b'q' as DWORD) << 16) | ((b' ' as DWORD) << 24);

pub const ANI_fram_ID: DWORD = (b'f' as DWORD) | ((b'r' as DWORD) << 8) | ((b'a' as DWORD) << 16) | ((b'm' as DWORD) << 24);

pub const ANI_rate_ID: DWORD = (b'r' as DWORD) | ((b'a' as DWORD) << 8) | ((b't' as DWORD) << 16) | ((b'e' as DWORD) << 24);

pub const ANI_FLAG_ICON: u32 = 0x1;

pub const ANI_FLAG_SEQUENCE: u32 = 0x2;

#[repr(C)]
pub struct ani_header {
    pub header_size: DWORD,
    pub num_frames: DWORD,
    pub num_steps: DWORD,
    pub width: DWORD,
    pub height: DWORD,
    pub bpp: DWORD,
    pub num_planes: DWORD,
    pub display_rate: DWORD,
    pub flags: DWORD,
}

#[repr(C)]
pub struct riff_chunk_t {
    pub data_size: DWORD,
    pub data: *const ::core::ffi::c_uchar,
}

unsafe extern "C" fn ReadMemoryPng(
    png_ptr: png_structp,
    data: png_bytep,
    length: usize,
) {
    let state = png_get_io_ptr(png_ptr) as *mut PNG_READER_STATE;
    if ((*state).currentPos + length) > (*state).bufSize {
        png_error(png_ptr, b"read error in ReadMemoryPng\0".as_ptr() as *const _);
        return;
    }
    RtlCopyMemory(
        data as *mut core::ffi::c_void,
        (*state).buffer.add((*state).currentPos) as *const core::ffi::c_void,
        length,
    );
    (*state).currentPos += length;
}

unsafe fn LibPngExists() -> BOOL {
    static mut bLibPngFound: BOOL = -1;
    if bLibPngFound == -1 {
        let wszName: [u16; 11] = [
            b'l' as u16, b'i' as u16, b'b' as u16, b'p' as u16, b'n' as u16,
            b'g' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
        ];
        let hLibPng = LoadLibraryExW(wszName.as_ptr(), core::ptr::null_mut(), LOAD_LIBRARY_AS_DATAFILE);
        bLibPngFound = (!hLibPng.is_null()) as BOOL;
        FreeLibrary(hLibPng);
    }
    bLibPngFound
}

unsafe fn CURSORICON_ConvertPngToBmpIcon(
    pngBits: PBYTE,
    fileSize: DWORD,
    pBmpIconSize: PDWORD,
) -> PBYTE {
    if LibPngExists() == 0 {
        return core::ptr::null_mut();
    }

    if pngBits.is_null() || fileSize < PNG_CHECK_SIG_SIZE || png_check_sig(pngBits, PNG_CHECK_SIG_SIZE as usize) == 0 {
        return core::ptr::null_mut();
    }

    let mut png_ptr = png_create_read_struct(
        PNG_LIBPNG_VER_STRING,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if png_ptr.is_null() {
        return core::ptr::null_mut();
    }

    let mut info_ptr = png_create_info_struct(png_ptr);
    if info_ptr.is_null() {
        png_destroy_read_struct(&mut png_ptr, core::ptr::null_mut(), core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    let mut readerState = PNG_READER_STATE {
        buffer: pngBits,
        bufSize: fileSize as usize,
        currentPos: PNG_CHECK_SIG_SIZE as usize,
    };
    png_set_read_fn(
        png_ptr,
        &mut readerState as *mut _ as *mut core::ffi::c_void,
        ReadMemoryPng,
    );
    png_set_sig_bytes(png_ptr, PNG_CHECK_SIG_SIZE as i32);

    png_read_info(png_ptr, info_ptr);

    let mut colorType: i32 = png_get_color_type(png_ptr, info_ptr) as i32;
    if colorType == PNG_COLOR_TYPE_PALETTE as i32 {
        png_set_palette_to_rgb(png_ptr);
    } else if colorType == PNG_COLOR_TYPE_GRAY as i32 || colorType == PNG_COLOR_TYPE_GRAY_ALPHA as i32 {
        png_set_gray_to_rgb(png_ptr);
    }
    png_set_scale_16(png_ptr);
    png_read_update_info(png_ptr, info_ptr);

    let mut width: png_uint_32 = 0;
    let mut height: png_uint_32 = 0;
    let mut bitDepth: i32 = 0;
    png_get_IHDR(
        png_ptr,
        info_ptr,
        &mut width,
        &mut height,
        &mut bitDepth,
        &mut colorType,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let channels = png_get_channels(png_ptr, info_ptr) as i32;
    let rowbytes = png_get_rowbytes(png_ptr, info_ptr) as i32;
    let imageSize = height as i32 * rowbytes;

    let rows = png_malloc(
        png_ptr,
        (core::mem::size_of::<png_bytep>() * height as usize) as _,
    ) as png_bytepp;
    if rows.is_null() {
        png_destroy_read_struct(&mut png_ptr, &mut info_ptr, core::ptr::null_mut());
        return core::ptr::null_mut();
    }

    let mut i = 0i32;
    while i < height as i32 {
        let r = png_malloc(png_ptr, rowbytes as _) as png_bytep;
        *rows.offset(i as isize) = r;
        if r.is_null() {
            let mut k = i;
            loop {
                k -= 1;
                if k < 0 {
                    break;
                }
                png_free(png_ptr, *rows.offset(k as isize) as _);
            }
            png_free(png_ptr, rows as _);
            png_destroy_read_struct(&mut png_ptr, &mut info_ptr, core::ptr::null_mut());
            return core::ptr::null_mut();
        }
        i += 1;
    }

    png_set_rows(png_ptr, info_ptr, rows);
    png_read_image(png_ptr, rows);
    png_read_end(png_ptr, info_ptr);

    let imageBytes = HeapAlloc(GetProcessHeap(), 0, imageSize as _) as PBYTE;
    if !imageBytes.is_null() {
        let mut pb = imageBytes;
        let mut i = height as i32 - 1;
        while i >= 0 {
            let row = *rows.offset(i as isize);
            let mut j = 0i32;
            while j < channels * width as i32 {
                *pb = *row.offset((j + 2) as isize);
                pb = pb.offset(1);
                *pb = *row.offset((j + 1) as isize);
                pb = pb.offset(1);
                *pb = *row.offset((j + 0) as isize);
                pb = pb.offset(1);
                if channels == 4 {
                    *pb = *row.offset((j + 3) as isize);
                    pb = pb.offset(1);
                }
                j += channels;
            }
            pb = pb.offset(((channels * width as i32) % 4) as isize);
            i -= 1;
        }
    }

    for i in 0..height as i32 {
        png_free(png_ptr, *rows.offset(i as isize) as _);
    }
    png_free(png_ptr, rows as _);
    png_destroy_read_struct(&mut png_ptr, &mut info_ptr, core::ptr::null_mut());

    if imageBytes.is_null() {
        return core::ptr::null_mut();
    }

    let bpp = (bitDepth * channels) as WORD;

    let maskSize = get_dib_image_size(width as i32, height as i32, 1) as DWORD;

    let mut info: BITMAPINFOHEADER = core::mem::zeroed();
    info.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    info.biWidth = width as _;
    info.biHeight = 2 * height as i32;
    info.biPlanes = 1;
    info.biBitCount = bpp;
    info.biCompression = BI_RGB as _;

    let mut cifd: CURSORICONFILEDIR = core::mem::zeroed();
    cifd.idReserved = 0;
    cifd.idType = 1;
    cifd.idCount = 1;
    cifd.idEntries[0].bWidth = width as BYTE;
    cifd.idEntries[0].bHeight = height as BYTE;
    cifd.idEntries[0].bColorCount = 0;
    cifd.idEntries[0].__bindgen_anon_1.wPlanes = 1;
    cifd.idEntries[0].__bindgen_anon_2.wBitCount = bpp;
    cifd.idEntries[0].dwDIBSize =
        (core::mem::size_of::<BITMAPINFOHEADER>() as i32 + imageSize + maskSize as i32) as DWORD;
    cifd.idEntries[0].dwDIBOffset = core::mem::size_of::<CURSORICONFILEDIR>() as DWORD;

    *pBmpIconSize = (core::mem::size_of::<CURSORICONFILEDIR>() as i32
        + core::mem::size_of::<BITMAPINFOHEADER>() as i32
        + imageSize
        + maskSize as i32) as DWORD;
    let pbBmpIcon = HeapAlloc(GetProcessHeap(), 0, *pBmpIconSize as _) as PBYTE;
    if pbBmpIcon.is_null() {
        HeapFree(GetProcessHeap(), 0, imageBytes as _);
        return core::ptr::null_mut();
    }

    let mut pb = pbBmpIcon;
    RtlCopyMemory(
        pb as _,
        &cifd as *const _ as _,
        core::mem::size_of::<CURSORICONFILEDIR>() as _,
    );
    pb = pb.add(core::mem::size_of::<CURSORICONFILEDIR>());
    RtlCopyMemory(
        pb as _,
        &info as *const _ as _,
        core::mem::size_of::<BITMAPINFOHEADER>() as _,
    );
    pb = pb.add(core::mem::size_of::<BITMAPINFOHEADER>());
    RtlCopyMemory(pb as _, imageBytes as _, imageSize as _);
    pb = pb.add(imageSize as usize);
    core::ptr::write_bytes(pb, 0xFF, maskSize as usize);

    HeapFree(GetProcessHeap(), 0, imageBytes as _);
    pbBmpIcon
}

unsafe fn LoadSystemCursors() {
    if gpsi.is_null() || crate::wow64::psi::hicon_sm_windows().is_null() {
        ERR!("Loading System Cursors\n");
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_ARROW, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_NORMAL);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_IBEAM, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_IBEAM);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_WAIT, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_WAIT);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_CROSS, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_CROSS);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_UPARROW, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_UP);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_ICON, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_ICON);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZE, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZE);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZENWSE, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZENWSE);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZENESW, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZENESW);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZEWE, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZEWE);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZENS, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZENS);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_SIZEALL, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_SIZEALL);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_NO, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_NO);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_HAND, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_HAND);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_APPSTARTING, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_APPSTARTING);
        NtUserSetSystemCursor(LoadImageW(core::ptr::null_mut(), IDC_HELP, IMAGE_CURSOR, 0, 0, LR_DEFAULTSIZE) as HCURSOR, OCR_HELP);
    }
}
#[no_mangle]
pub unsafe extern "system" fn User32SetupDefaultCursors(
    Arguments: PVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let DefaultCursor: *mut BOOL = Arguments as *mut BOOL;
    let mut hCursor: HCURSOR;

    LoadSystemCursors();

    if *DefaultCursor != 0 {
        hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
        SetCursor(hCursor);
    } else {
        SetCursor(core::ptr::null_mut());
        hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
        SetCursor(hCursor);
    }

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(hCursor) as PVOID,
        core::mem::size_of::<HCURSOR>() as ULONG,
        STATUS_SUCCESS,
    )
}

// get_icon_size / CursorIconToCursor are cdecl C helpers (user32p.h, no WINAPI)
// referenced cross-TU by the now-Rust draw.rs/static_.rs and surviving C — must be
// `#[no_mangle] extern "C"` (plain `_get_icon_size`, no `@N`).
#[no_mangle]
pub unsafe extern "C" fn get_icon_size(hIcon: HICON, size: *mut SIZE) -> BOOL {
    NtUserGetIconSize(
        hIcon as HANDLE,
        0,
        core::ptr::addr_of_mut!((*size).cx),
        core::ptr::addr_of_mut!((*size).cy),
    )
}

#[no_mangle]
pub unsafe extern "C" fn CursorIconToCursor(_hIcon: HICON, _SemiTransparent: BOOL) -> HCURSOR {
    // UNIMPLEMENTED — faithful to the C stub.
    core::ptr::null_mut()
}

unsafe fn map_fileW(name: LPCWSTR, filesize: LPDWORD) -> *mut core::ffi::c_void {

    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_RANDOM_ACCESS: u32 = 0x10000000;
    const FILE_MAP_READ: u32 = 0x0004;

    let mut ptr: LPVOID = core::ptr::null_mut();

    let hFile = CreateFileW(
        name,
        GENERIC_READ,
        FILE_SHARE_READ,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_FLAG_RANDOM_ACCESS,
        core::ptr::null_mut(),
    );
    if hFile != INVALID_HANDLE_VALUE as *mut core::ffi::c_void {
        let hMapping = CreateFileMappingW(
            hFile,
            core::ptr::null_mut(),
            PAGE_READONLY,
            0,
            0,
            core::ptr::null_mut(),
        );
        if !hMapping.is_null() {
            ptr = MapViewOfFile(hMapping, FILE_MAP_READ, 0, 0, 0);
            CloseHandle(hMapping);
            if !filesize.is_null() {
                *filesize = GetFileSize(hFile, core::ptr::null_mut());
            }
        }
        CloseHandle(hFile);
    }
    ptr
}

unsafe fn get_dib_image_size(width: i32, height: i32, depth: i32) -> i32 {
    (((width * depth + 31) / 8) & !3) * height.abs()
}

unsafe fn is_dib_monochrome(info: *const BITMAPINFO) -> u8 {
    if (*info).bmiHeader.biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
        let core = info as *const BITMAPCOREINFO;
        let rgb = core::ptr::addr_of!((*core).bmciColors) as *const RGBTRIPLE;

        if (*core).bmciHeader.bcBitCount != 1 {
            return 0u8;
        }

        if RGB(
            (*rgb.add(0)).rgbtRed,
            (*rgb.add(0)).rgbtGreen,
            (*rgb.add(0)).rgbtBlue,
        ) == RGB(0, 0, 0)
        {
            return (RGB(
                (*rgb.add(1)).rgbtRed,
                (*rgb.add(1)).rgbtGreen,
                (*rgb.add(1)).rgbtBlue,
            ) == RGB(0xff, 0xff, 0xff)) as u8;
        }

        if RGB(
            (*rgb.add(0)).rgbtRed,
            (*rgb.add(0)).rgbtGreen,
            (*rgb.add(0)).rgbtBlue,
        ) == RGB(0xff, 0xff, 0xff)
        {
            return (RGB(
                (*rgb.add(1)).rgbtRed,
                (*rgb.add(1)).rgbtGreen,
                (*rgb.add(1)).rgbtBlue,
            ) == RGB(0, 0, 0)) as u8;
        }

        0u8
    } else {
        let rgb = core::ptr::addr_of!((*info).bmiColors) as *const RGBQUAD;

        if (*info).bmiHeader.biBitCount != 1 {
            return 0u8;
        }

        if RGBA(
            (*rgb.add(0)).rgbRed,
            (*rgb.add(0)).rgbGreen,
            (*rgb.add(0)).rgbBlue,
            (*rgb.add(0)).rgbReserved,
        ) == RGBA(0, 0, 0, 0)
        {
            return (RGBA(
                (*rgb.add(1)).rgbRed,
                (*rgb.add(1)).rgbGreen,
                (*rgb.add(1)).rgbBlue,
                (*rgb.add(1)).rgbReserved,
            ) == RGBA(0xff, 0xff, 0xff, 0)) as u8;
        }

        if RGBA(
            (*rgb.add(0)).rgbRed,
            (*rgb.add(0)).rgbGreen,
            (*rgb.add(0)).rgbBlue,
            (*rgb.add(0)).rgbReserved,
        ) == RGBA(0xff, 0xff, 0xff, 0)
        {
            return (RGBA(
                (*rgb.add(1)).rgbRed,
                (*rgb.add(1)).rgbGreen,
                (*rgb.add(1)).rgbBlue,
                (*rgb.add(1)).rgbReserved,
            ) == RGBA(0, 0, 0, 0)) as u8;
        }

        0u8
    }
}

unsafe fn bitmap_info_size(info: *const BITMAPINFO, coloruse: WORD) -> core::ffi::c_int {
    let mut colors: core::ffi::c_uint;
    let mut size: core::ffi::c_uint;
    let mut masks: core::ffi::c_uint = 0;

    if (*info).bmiHeader.biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
        let core = info as *const BITMAPCOREHEADER;
        colors = if (*core).bcBitCount <= 8 {
            1u32 << (*core).bcBitCount
        } else {
            0
        };
        (core::mem::size_of::<BITMAPCOREHEADER>()
            + (colors as usize)
                * (if coloruse as core::ffi::c_int == DIB_RGB_COLORS as core::ffi::c_int {
                    core::mem::size_of::<RGBTRIPLE>()
                } else {
                    core::mem::size_of::<WORD>()
                })) as core::ffi::c_int
    } else {
        colors = (*info).bmiHeader.biClrUsed;
        if colors > 256 {
            colors = 256;
        }
        if colors == 0 && (*info).bmiHeader.biBitCount <= 8 {
            colors = 1u32 << (*info).bmiHeader.biBitCount;
        }
        if (*info).bmiHeader.biCompression == BI_BITFIELDS as u32 {
            masks = 3;
        }
        let a = (*info).bmiHeader.biSize;
        let b = (core::mem::size_of::<BITMAPINFOHEADER>()
            + (masks as usize) * core::mem::size_of::<DWORD>()) as core::ffi::c_uint;
        size = if a > b { a } else { b };
        if (*info).bmiHeader.biSize as usize >= core::mem::size_of::<BITMAPV4HEADER>()
            && (*info).bmiHeader.biCompression == BI_BITFIELDS as u32
            && ((*info).bmiHeader.biBitCount == 16 || (*info).bmiHeader.biBitCount == 32)
        {
            size += (3 * core::mem::size_of::<DWORD>()) as core::ffi::c_uint;
        }
        (size as usize
            + (colors as usize)
                * (if coloruse as core::ffi::c_int == DIB_RGB_COLORS as core::ffi::c_int {
                    core::mem::size_of::<RGBQUAD>()
                } else {
                    core::mem::size_of::<WORD>()
                })) as core::ffi::c_int
    }
}

unsafe fn DIB_GetBitmapInfo(
    header: *const BITMAPINFOHEADER,
    width: *mut i32,
    height: *mut i32,
    bpp: *mut u16,
    compr: *mut u32,
) -> i32 {
    const CR_LOCAL: u8 = 13;
    const LF: u8 = 10;
    const EOFM: u8 = 26;
    const HighBitDetect: u8 = 0x89;
    let png_sig_pattern: [u8; 8] = [HighBitDetect, b'P', b'N', b'G', CR_LOCAL, LF, EOFM, LF];

    if (*header).biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
        let core = header as *const BITMAPCOREHEADER;
        *width = (*core).bcWidth as i32;
        *height = (*core).bcHeight as i32;
        *bpp = (*core).bcBitCount;
        *compr = 0;
        return 0;
    } else if (*header).biSize as usize == core::mem::size_of::<BITMAPINFOHEADER>()
        || (*header).biSize as usize == core::mem::size_of::<BITMAPV4HEADER>()
        || (*header).biSize as usize == core::mem::size_of::<BITMAPV5HEADER>()
    {
        *width = (*header).biWidth;
        *height = (*header).biHeight;
        *bpp = (*header).biBitCount;
        *compr = (*header).biCompression;
        return 1;
    }

    if memcmp(
        core::ptr::addr_of!((*header).biSize) as *const core::ffi::c_void,
        png_sig_pattern.as_ptr() as *const core::ffi::c_void,
        core::mem::size_of::<[u8; 8]>(),
    ) == 0
    {
        TRACE!(b"We have a PNG icon\n\0".as_ptr() as *const i8);
    } else {
        ERR!(
            "Unknown/wrong size for header of 0x%x\n",
            (*header).biSize,
        );
    }
    -1
}

unsafe fn stretch_blt_icon(hdc_dst: HDC, dst_width: i32, dst_height: i32, src: HBITMAP) {
    let hdc = CreateCompatibleDC(core::ptr::null_mut());
    let mut bm: BITMAP = core::mem::zeroed();

    GetObjectW(src as LPVOID, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);

    let hbmpPrev = SelectObject(hdc, src as HGDIOBJ);

    if hbmpPrev.is_null() {
        let info = HeapAlloc(
            GetProcessHeap(),
            0,
            (core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()),
        ) as *mut BITMAPINFO;
        if info.is_null() {
            return;
        }
        (*info).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
        (*info).bmiHeader.biWidth = bm.bmWidth;
        (*info).bmiHeader.biHeight = bm.bmHeight;
        (*info).bmiHeader.biPlanes = GetDeviceCaps(hdc_dst, PLANES as i32) as u16;
        (*info).bmiHeader.biBitCount = GetDeviceCaps(hdc_dst, BITSPIXEL as i32) as u16;
        (*info).bmiHeader.biCompression = BI_RGB as u32;
        (*info).bmiHeader.biSizeImage =
            get_dib_image_size(bm.bmWidth, bm.bmHeight, (*info).bmiHeader.biBitCount as i32) as u32;
        (*info).bmiHeader.biXPelsPerMeter = 0;
        (*info).bmiHeader.biYPelsPerMeter = 0;
        (*info).bmiHeader.biClrUsed = 0;
        (*info).bmiHeader.biClrImportant = 0;
        let bits = HeapAlloc(GetProcessHeap(), 0, (*info).bmiHeader.biSizeImage as usize) as LPVOID;
        if !bits.is_null()
            && GetDIBits(hdc, src, 0, bm.bmHeight as u32, bits, info, DIB_RGB_COLORS as u32) != 0
        {
            StretchDIBits(
                hdc_dst, 0, 0, dst_width, dst_height, 0, 0, bm.bmWidth, bm.bmHeight, bits, info,
                DIB_RGB_COLORS as u32, SRCCOPY,
            );
        }

        HeapFree(GetProcessHeap(), 0, bits);
        HeapFree(GetProcessHeap(), 0, info as LPVOID);
    } else {
        StretchBlt(hdc_dst, 0, 0, dst_width, dst_height, hdc, 0, 0, bm.bmWidth, bm.bmHeight, SRCCOPY);
        SelectObject(hdc, hbmpPrev);
    }

    DeleteDC(hdc);
}

unsafe fn bmi_has_alpha(info: *const BITMAPINFO, bits: *const core::ffi::c_void) -> BOOL {
    let mut i: i32;
    let mut has_alpha: BOOL = 0;
    let mut ptr = bits as *const core::ffi::c_uchar;

    if (*info).bmiHeader.biBitCount != 32 {
        return 0;
    }

    i = 0;
    while i < (*info).bmiHeader.biWidth * abs((*info).bmiHeader.biHeight) {
        has_alpha = if *ptr.add(3) != 0 { 1 } else { 0 };
        if has_alpha != 0 {
            break;
        }
        i += 1;
        ptr = ptr.add(4);
    }

    has_alpha
}

unsafe fn create_alpha_bitmap(
    color: HBITMAP,
    src_info: *mut BITMAPINFO,
    color_bits: *const core::ffi::c_void,
    width: i32,
    height: i32,
) -> HBITMAP {

    let mut alpha: HBITMAP = core::ptr::null_mut();
    let hbmpOld: *mut core::ffi::c_void;
    let hdc: HDC;
    let hdcScreen: HDC;
    let mut ptr: *mut u8;
    let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut size: u32;

    hdcScreen = CreateDCW(DISPLAYW.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null());
    if hdcScreen.is_null() {
        return core::ptr::null_mut();
    }
    hdc = CreateCompatibleDC(hdcScreen);
    if hdc.is_null() {
        DeleteDC(hdcScreen);
        return core::ptr::null_mut();
    }

    'done: {
        if !color.is_null() {
            let mut bm: BITMAP = core::mem::zeroed();
            let info: *mut BITMAPINFO;

            if GetObjectW(color as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, &mut bm as *mut _ as LPVOID) == 0 {
                break 'done;
            }
            if bm.bmBitsPixel != 32 {
                break 'done;
            }

            size = get_dib_image_size(bm.bmWidth, bm.bmHeight, 32) as u32;

            let field_off = {
                let dummy: *const BITMAPINFO = core::ptr::null();
                (*dummy).bmiColors.as_ptr().add(256) as usize
            };
            info = HeapAlloc(GetProcessHeap(), 0, field_off) as *mut BITMAPINFO;
            if info.is_null() {
                break 'done;
            }
            (*info).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
            (*info).bmiHeader.biWidth = bm.bmWidth;
            (*info).bmiHeader.biHeight = -bm.bmHeight;
            (*info).bmiHeader.biPlanes = 1;
            (*info).bmiHeader.biBitCount = 32;
            (*info).bmiHeader.biCompression = BI_RGB as u32;
            (*info).bmiHeader.biSizeImage = size;
            (*info).bmiHeader.biXPelsPerMeter = 0;
            (*info).bmiHeader.biYPelsPerMeter = 0;
            (*info).bmiHeader.biClrUsed = 0;
            (*info).bmiHeader.biClrImportant = 0;

            bits = HeapAlloc(GetProcessHeap(), 0, size as usize);
            if bits.is_null() {
                HeapFree(GetProcessHeap(), 0, info as LPVOID);
                break 'done;
            }
            if GetDIBits(hdc, color, 0, bm.bmHeight as u32, bits, info, DIB_RGB_COLORS as u32) == 0 {
                HeapFree(GetProcessHeap(), 0, info as LPVOID);
                break 'done;
            }
            if bmi_has_alpha(info, bits) == 0 {
                HeapFree(GetProcessHeap(), 0, info as LPVOID);
                break 'done;
            }

            ptr = bits as *mut u8;
            while ptr < (bits as *mut u8).add(size as usize) {
                let alpha_v = *ptr.add(3) as u32;
                *ptr.add(0) = ((*ptr.add(0) as u32 * alpha_v) / 255) as u8;
                *ptr.add(1) = ((*ptr.add(1) as u32 * alpha_v) / 255) as u8;
                *ptr.add(2) = ((*ptr.add(2) as u32 * alpha_v) / 255) as u8;
                ptr = ptr.add(4);
            }

            alpha = CreateDIBitmap(hdc, core::ptr::null(), (CBM_INIT | 2) as u32, bits, info, DIB_RGB_COLORS as u32);

            HeapFree(GetProcessHeap(), 0, info as LPVOID);
        } else {
            let mut bpp: u16 = 0;
            let mut compr: u32 = 0;
            let mut orig_width: i32 = 0;
            let mut orig_height: i32 = 0;

            if bmi_has_alpha(src_info, color_bits) == 0 {
                break 'done;
            }

            if DIB_GetBitmapInfo(core::ptr::addr_of!((*src_info).bmiHeader), &mut orig_width, &mut orig_height, &mut bpp, &mut compr) == 0 {
                break 'done;
            }
            if bpp != 32 {
                break 'done;
            }

            size = get_dib_image_size(orig_width, orig_height, bpp as i32) as u32;
            bits = HeapAlloc(GetProcessHeap(), 0, size as usize);
            if bits.is_null() {
                break 'done;
            }
            core::ptr::copy_nonoverlapping(color_bits as *const u8, bits as *mut u8, size as usize);

            ptr = bits as *mut u8;
            while ptr < (bits as *mut u8).add(size as usize) {
                let alpha_v = *ptr.add(3) as u32;
                *ptr.add(0) = ((*ptr.add(0) as u32 * alpha_v) / 255) as u8;
                *ptr.add(1) = ((*ptr.add(1) as u32 * alpha_v) / 255) as u8;
                *ptr.add(2) = ((*ptr.add(2) as u32 * alpha_v) / 255) as u8;
                ptr = ptr.add(4);
            }

            if (*src_info).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as u32 {
                let core_hdr = core::ptr::addr_of_mut!((*src_info).bmiHeader) as *mut BITMAPCOREHEADER;
                (*core_hdr).bcWidth = width as u16;
                (*core_hdr).bcHeight = height as u16;
            } else {
                (*src_info).bmiHeader.biWidth = width;
                (*src_info).bmiHeader.biHeight = height;
            }

            alpha = CreateDIBitmap(hdcScreen, core::ptr::null(), 2u32, core::ptr::null(), src_info, DIB_RGB_COLORS as u32);

            if (*src_info).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as u32 {
                let core_hdr = core::ptr::addr_of_mut!((*src_info).bmiHeader) as *mut BITMAPCOREHEADER;
                (*core_hdr).bcWidth = orig_width as u16;
                (*core_hdr).bcHeight = orig_height as u16;
            } else {
                (*src_info).bmiHeader.biWidth = orig_width;
                (*src_info).bmiHeader.biHeight = orig_height;
            }
            if alpha.is_null() {
                break 'done;
            }
            hbmpOld = SelectObject(hdc, alpha as *mut core::ffi::c_void);
            if hbmpOld.is_null() {
                DeleteObject(alpha as *mut core::ffi::c_void);
                alpha = core::ptr::null_mut();
                break 'done;
            }
            if StretchDIBits(hdc, 0, 0, width, height, 0, 0, orig_width, orig_height, bits, src_info, DIB_RGB_COLORS as u32, SRCCOPY as u32) == 0 {
                SelectObject(hdc, hbmpOld);
                DeleteObject(alpha as *mut core::ffi::c_void);
                alpha = core::ptr::null_mut();
            } else {
                SelectObject(hdc, hbmpOld);
            }
        }
    }

    DeleteDC(hdcScreen);
    DeleteDC(hdc);
    if !bits.is_null() {
        HeapFree(GetProcessHeap(), 0, bits);
    }

    alpha
}

unsafe fn get_best_icon_file_entry(
    dir: *const CURSORICONFILEDIR,
    dwFileSize: DWORD,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    bIcon: BOOL,
    fuLoad: DWORD,
) -> *const CURSORICONFILEDIRENTRY {
    let fakeDir: *mut CURSORICONDIR;
    let mut fakeEntry: *mut CURSORICONDIRENTRY;
    let mut i: WORD;
    let mut entry: *const CURSORICONFILEDIRENTRY;

    if (dwFileSize as usize) < core::mem::size_of::<CURSORICONFILEDIR>() {
        return core::ptr::null();
    }

    let idCount = (*dir).idCount as usize;

    let fileDirEntriesOffset = {
        let base = core::ptr::addr_of!((*dir).idEntries) as usize;
        let off = base - (dir as usize);
        off + idCount * core::mem::size_of::<CURSORICONFILEDIRENTRY>()
    };

    if (dwFileSize as usize) < fileDirEntriesOffset {
        return core::ptr::null();
    }

    let allocSize = {
        let tmp: *const CURSORICONDIR = core::ptr::null();
        let base = core::ptr::addr_of!((*tmp).idEntries) as usize;
        base + idCount * core::mem::size_of::<CURSORICONDIRENTRY>()
    };

    fakeDir = HeapAlloc(GetProcessHeap(), 0, allocSize) as *mut CURSORICONDIR;
    if fakeDir.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null();
    }

    (*fakeDir).idReserved = 0;
    (*fakeDir).idType = (*dir).idType;
    (*fakeDir).idCount = (*dir).idCount;

    i = 0;
    while (i as usize) < idCount {
        fakeEntry = (core::ptr::addr_of_mut!((*fakeDir).idEntries) as *mut CURSORICONDIRENTRY).add(i as usize);
        entry = (core::ptr::addr_of!((*dir).idEntries) as *const CURSORICONFILEDIRENTRY).add(i as usize);

        if ((*entry).dwDIBOffset > dwFileSize)
            || (((*entry).dwDIBOffset + (*entry).dwDIBSize) > dwFileSize)
        {
            ERR!("Corrupted icon file?.\n");
            HeapFree(GetProcessHeap(), 0, fakeDir as LPVOID);
            return core::ptr::null();
        }

        if bIcon != 0 {
            (*fakeEntry).ResInfo.icon.bWidth = (*entry).bWidth;
            (*fakeEntry).ResInfo.icon.bHeight = (*entry).bHeight;
            (*fakeEntry).ResInfo.icon.bColorCount = 0;
            (*fakeEntry).ResInfo.icon.bReserved = 0;
        } else {
            (*fakeEntry).ResInfo.cursor.wWidth = (*entry).bWidth as WORD;
            (*fakeEntry).ResInfo.cursor.wHeight = (*entry).bHeight as WORD;
        }

        (*fakeEntry).wPlanes = 1;

        let pHeader = (dir as *const u8).add((*entry).dwDIBOffset as usize) as *const BITMAPINFOHEADER;
        if (*pHeader).biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
            let pCore = (dir as *const u8).add((*entry).dwDIBOffset as usize) as *const BITMAPCOREHEADER;
            (*fakeEntry).wBitCount = (*pCore).bcBitCount;
        } else {
            (*fakeEntry).wBitCount = (*pHeader).biBitCount;
        }

        (*fakeEntry).dwBytesInRes = (*entry).dwDIBSize;
        (*fakeEntry).wResId = i + 1;

        i += 1;
    }

    i = LookupIconIdFromDirectoryEx(
        fakeDir as PBYTE,
        bIcon,
        cxDesired,
        cyDesired,
        fuLoad & LR_MONOCHROME,
    ) as WORD;

    HeapFree(GetProcessHeap(), 0, fakeDir as LPVOID);

    if i == 0 {
        WARN!(b"Unable to get a fit entry index.\n\0".as_ptr());
        return core::ptr::null();
    }

    (core::ptr::addr_of!((*dir).idEntries) as *const CURSORICONFILEDIRENTRY).add((i - 1) as usize)
}

// Cross-TU cdecl helper: still-C exticon.c declares & calls this (DWORD WINAPI-less)
// — must be `#[no_mangle] extern "C"` so the unmangled `_get_best_icon_file_offset`
// resolves at link.
#[no_mangle]
pub unsafe extern "C" fn get_best_icon_file_offset(
    dir: LPBYTE,
    dwFileSize: DWORD,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    bIcon: BOOL,
    fuLoad: DWORD,
    ptHotSpot: *mut POINT,
) -> DWORD {
    let entry = get_best_icon_file_entry(
        dir as *const CURSORICONFILEDIR,
        dwFileSize,
        cxDesired,
        cyDesired,
        bIcon,
        fuLoad,
    );

    if !ptHotSpot.is_null() {
        // xHotspot/yHotspot are the cursor-flavor union members (anon_1 / anon_2).
        // write_volatile so LLVM's captures(none) can't elide the out-param stores
        // (the still-C caller exticon.c reads ptHotSpot after return — §2.5).
        core::ptr::addr_of_mut!((*ptHotSpot).x)
            .write_volatile((*entry).__bindgen_anon_1.xHotspot as core::ffi::c_long);
        core::ptr::addr_of_mut!((*ptHotSpot).y)
            .write_volatile((*entry).__bindgen_anon_2.yHotspot as core::ffi::c_long);
    }

    if !entry.is_null() {
        return (*entry).dwDIBOffset;
    }

    0
}

unsafe fn CURSORICON_GetCursorDataFromBMI(
    pdata: *mut CURSORDATA,
    pbmi: *const BITMAPINFO,
) -> u8 {

    let ubmiSize: u32 = bitmap_info_size(pbmi, DIB_RGB_COLORS as WORD) as u32;
    let monochrome: u8 = is_dib_monochrome(pbmi);
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut bpp: u16 = 0;
    let mut compr: u32 = 0;
    let ibmpType: i32;
    let hdc: HDC;
    let hdcScreen: HDC;
    let mut pbmiCopy: *mut BITMAPINFO = core::ptr::null_mut();
    let mut hbmpOld: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut bResult: u8 = 0u8;
    let pvColor: *const core::ffi::c_void;
    let pvMask: *const core::ffi::c_void;

    ibmpType = DIB_GetBitmapInfo(
        core::ptr::addr_of!((*pbmi).bmiHeader),
        &mut width,
        &mut height,
        &mut bpp,
        &mut compr,
    );
    if ibmpType < 0 {
        return 0u8;
    }

    if compr != BI_RGB {
        return 0u8;
    }

    if (*pdata).cx == 0 {
        (*pdata).cx = width as u32;
    }
    if (*pdata).cy == 0 {
        (*pdata).cy = if height < 0 { (-height / 2) as u32 } else { (height / 2) as u32 };
    }

    // C: `pdata->rt == (USHORT)RT_CURSOR` — RT_CURSOR is 1 (winuser.h), NOT 2.
    // The translation wrote 2, so the real-cursor hotspot-scaling branch was
    // unreachable and every scaled cursor silently got a center hotspot.
    if (*pdata).rt == (1u32 as USHORT) {
        if (*pdata).cx != width as u32 {
            (*pdata).xHotspot = (((*pdata).xHotspot as i32 * (*pdata).cx as i32) / width) as i16;
        }
        if (*pdata).cy != (height / 2) as u32 {
            (*pdata).yHotspot = (((*pdata).yHotspot as i32 * (*pdata).cy as i32 * 2) / height) as i16;
        }
    } else {
        (*pdata).xHotspot = ((*pdata).cx / 2) as i16;
        (*pdata).yHotspot = ((*pdata).cy / 2) as i16;
    }

    hdcScreen = CreateDCW(
        DISPLAYW.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if hdcScreen.is_null() {
        return 0u8;
    }
    hdc = CreateCompatibleDC(hdcScreen);
    if hdc.is_null() {
        DeleteDC(hdcScreen);
        return 0u8;
    }

    'done: {
        let field_off = {
            let base = core::ptr::null::<BITMAPINFO>();
            (*base).bmiColors.as_ptr().add(3) as usize
        };
        let allocSize = if ubmiSize as usize > field_off { ubmiSize as usize } else { field_off };
        pbmiCopy = HeapAlloc(GetProcessHeap(), 0, allocSize) as *mut BITMAPINFO;
        if pbmiCopy.is_null() {
            break 'done;
        }
        RtlCopyMemory(
            pbmiCopy as *mut core::ffi::c_void,
            pbmi as *const core::ffi::c_void,
            ubmiSize as usize,
        );

        if (*pbmiCopy).bmiHeader.biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
            let core_hdr = core::ptr::addr_of_mut!((*pbmiCopy).bmiHeader) as *mut BITMAPCOREHEADER;
            (*core_hdr).bcHeight /= 2;
        } else {
            (*pbmiCopy).bmiHeader.biHeight /= 2;
        }
        height /= 2;

        pvColor = (pbmi as *const u8).add(ubmiSize as usize) as *const core::ffi::c_void;
        pvMask = (pvColor as *const u8).add(get_dib_image_size(width, height, bpp as i32) as usize)
            as *const core::ffi::c_void;

        if monochrome != 0 {
            (*pdata).hbmColor = core::ptr::null_mut();
            (*pdata).hbmMask = CreateBitmap(
                (*pdata).cx as i32,
                ((*pdata).cy * 2) as i32,
                1,
                1,
                core::ptr::null(),
            );
            if (*pdata).hbmMask.is_null() {
                break 'done;
            }
            hbmpOld = SelectObject(hdc, (*pdata).hbmMask as *mut core::ffi::c_void);
            if hbmpOld.is_null() {
                break 'done;
            }

            if StretchDIBits(
                hdc,
                0,
                (*pdata).cy as i32,
                (*pdata).cx as i32,
                (*pdata).cy as i32,
                0,
                0,
                width,
                height,
                pvColor,
                pbmiCopy,
                DIB_RGB_COLORS,
                SRCCOPY,
            ) == 0
            {
                break 'done;
            }
            (*pdata).bpp = 1;
        } else {
            (*pdata).hbmColor = CreateCompatibleBitmap(hdcScreen, (*pdata).cx as i32, (*pdata).cy as i32);
            if (*pdata).hbmColor.is_null() {
                break 'done;
            }
            (*pdata).hbmMask = CreateBitmap(
                (*pdata).cx as i32,
                (*pdata).cy as i32,
                1,
                1,
                core::ptr::null(),
            );
            if (*pdata).hbmMask.is_null() {
                break 'done;
            }
            hbmpOld = SelectObject(hdc, (*pdata).hbmColor as *mut core::ffi::c_void);
            if hbmpOld.is_null() {
                break 'done;
            }
            if StretchDIBits(
                hdc,
                0,
                0,
                (*pdata).cx as i32,
                (*pdata).cy as i32,
                0,
                0,
                width,
                height,
                pvColor,
                pbmiCopy,
                DIB_RGB_COLORS,
                SRCCOPY,
            ) == 0
            {
                break 'done;
            }
            (*pdata).bpp = GetDeviceCaps(hdcScreen, BITSPIXEL as core::ffi::c_int) as u32;
            (*pdata).hbmAlpha = create_alpha_bitmap(
                core::ptr::null_mut(),
                pbmiCopy,
                pvColor,
                (*pdata).cx as i32,
                (*pdata).cy as i32,
            );

            if (*pbmiCopy).bmiHeader.biSize as usize != core::mem::size_of::<BITMAPCOREHEADER>() {
                let rgb = core::ptr::addr_of_mut!((*pbmiCopy).bmiColors) as *mut RGBQUAD;

                (*pbmiCopy).bmiHeader.biClrUsed = 2;
                (*pbmiCopy).bmiHeader.biClrImportant = 2;
                (*rgb.add(0)).rgbBlue = 0x00;
                (*rgb.add(0)).rgbGreen = 0x00;
                (*rgb.add(0)).rgbRed = 0x00;
                (*rgb.add(1)).rgbBlue = 0xff;
                (*rgb.add(1)).rgbGreen = 0xff;
                (*rgb.add(1)).rgbRed = 0xff;
                (*rgb.add(0)).rgbReserved = 0;
                (*rgb.add(1)).rgbReserved = 0;
                (*pbmiCopy).bmiHeader.biBitCount = 1;
            } else {
                let core_hdr = pbmiCopy as *mut BITMAPCOREHEADER;
                let rgb = core_hdr.add(1) as *mut RGBTRIPLE;

                (*rgb.add(0)).rgbtBlue = 0x00;
                (*rgb.add(0)).rgbtGreen = 0x00;
                (*rgb.add(0)).rgbtRed = 0x00;
                (*rgb.add(1)).rgbtBlue = 0xff;
                (*rgb.add(1)).rgbtGreen = 0xff;
                (*rgb.add(1)).rgbtRed = 0xff;
                let core_hdr2 = core::ptr::addr_of_mut!((*pbmiCopy).bmiHeader) as *mut BITMAPCOREHEADER;
                (*core_hdr2).bcBitCount = 1;
            }
        }

        if SelectObject(hdc, (*pdata).hbmMask as *mut core::ffi::c_void).is_null() {
            break 'done;
        }
        bResult = (StretchDIBits(
            hdc,
            0,
            0,
            (*pdata).cx as i32,
            (*pdata).cy as i32,
            0,
            0,
            width,
            height,
            pvMask,
            pbmiCopy,
            DIB_RGB_COLORS,
            SRCCOPY,
        ) != 0) as u8;
    }

    DeleteDC(hdcScreen);
    if !hbmpOld.is_null() {
        SelectObject(hdc, hbmpOld);
    }
    DeleteDC(hdc);
    if !pbmiCopy.is_null() {
        HeapFree(GetProcessHeap(), 0, pbmiCopy as *mut core::ffi::c_void);
    }
    if bResult == 0 {
        if !(*pdata).hbmMask.is_null() {
            DeleteObject((*pdata).hbmMask as *mut core::ffi::c_void);
        }
        if !(*pdata).hbmColor.is_null() {
            DeleteObject((*pdata).hbmColor as *mut core::ffi::c_void);
        }
        if !(*pdata).hbmAlpha.is_null() {
            DeleteObject((*pdata).hbmAlpha as *mut core::ffi::c_void);
        }
    }
    bResult
}

unsafe fn CURSORICON_GetCursorDataFromIconInfo(
    pCursorData: *mut CURSORDATA,
    pIconInfo: *mut ICONINFO,
) -> i32 {

    let mut bm: BITMAP = core::mem::zeroed();

    core::ptr::write_bytes(pCursorData, 0, 1);

    if !(*pIconInfo).hbmColor.is_null() {
        let hdcScreen: HDC;
        let hdcMem: HDC;
        let hbmpPrev: *mut core::ffi::c_void;

        if GetObjectW(
            (*pIconInfo).hbmMask as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        ) == 0
        {
            return 0;
        }
        hdcScreen = CreateDCW(
            DISPLAYW.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if hdcScreen.is_null() {
            return 0;
        }
        hdcMem = CreateCompatibleDC(hdcScreen);
        if hdcMem.is_null() {
            DeleteDC(hdcScreen);
            return 0;
        }
        (*pCursorData).hbmColor = CreateCompatibleBitmap(hdcScreen, bm.bmWidth, bm.bmHeight);
        DeleteDC(hdcScreen);
        if (*pCursorData).hbmColor.is_null() {
            DeleteDC(hdcMem);
            return 0;
        }
        hbmpPrev = SelectObject(hdcMem, (*pCursorData).hbmColor as *mut core::ffi::c_void);
        if hbmpPrev.is_null() {
            DeleteDC(hdcMem);
            DeleteObject((*pCursorData).hbmColor as *mut core::ffi::c_void);
            return 0;
        }
        stretch_blt_icon(hdcMem, bm.bmWidth, bm.bmHeight, (*pIconInfo).hbmColor);
        SelectObject(hdcMem, hbmpPrev);
        DeleteDC(hdcMem);
    }

    (*pCursorData).hbmMask = CopyImage(
        (*pIconInfo).hbmMask as HGLOBAL,
        IMAGE_BITMAP,
        0,
        0,
        LR_MONOCHROME,
    ) as HBITMAP;
    if (*pCursorData).hbmMask.is_null() {
        return 0;
    }

    (*pCursorData).rt = (if (*pIconInfo).fIcon != 0 {
        1usize
    } else {
        2usize
    }) as u16;

    if !(*pCursorData).hbmColor.is_null() {
        GetObjectW(
            (*pCursorData).hbmColor as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        );
        (*pCursorData).bpp = bm.bmBitsPixel as u32;
        (*pCursorData).cx = bm.bmWidth as u32;
        (*pCursorData).cy = bm.bmHeight as u32;
        if (*pCursorData).bpp == 32 {
            (*pCursorData).hbmAlpha = create_alpha_bitmap(
                (*pCursorData).hbmColor,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
                0,
            );
        }
    } else {
        GetObjectW(
            (*pCursorData).hbmMask as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            core::ptr::addr_of_mut!(bm) as LPVOID,
        );
        (*pCursorData).bpp = 1;
        (*pCursorData).cx = bm.bmWidth as u32;
        (*pCursorData).cy = (bm.bmHeight / 2) as u32;
    }

    if (*pIconInfo).fIcon != 0 {
        (*pCursorData).xHotspot = ((*pCursorData).cx / 2) as SHORT;
        (*pCursorData).yHotspot = ((*pCursorData).cy / 2) as SHORT;
    } else {
        (*pCursorData).xHotspot = (*pIconInfo).xHotspot as SHORT;
        (*pCursorData).yHotspot = (*pIconInfo).yHotspot as SHORT;
    }

    1
}

unsafe fn dump_ani_header(header: *const ani_header) {
    let _ = header;
}

unsafe fn riff_find_chunk(chunk_id: DWORD, chunk_type: DWORD, parent_chunk: *const riff_chunk_t, chunk: *mut riff_chunk_t) {
    let mut ptr = (*parent_chunk).data;
    let mut end = (*parent_chunk).data.add(((*parent_chunk).data_size - (2 * core::mem::size_of::<DWORD>() as DWORD)) as usize);

    if chunk_type == ANI_LIST_ID || chunk_type == ANI_RIFF_ID {
        end = end.sub(core::mem::size_of::<DWORD>());
    }

    while ptr < end {
        if (chunk_type == 0 && *(ptr as *const DWORD) == chunk_id)
            || (chunk_type != 0 && *(ptr as *const DWORD) == chunk_type && *((ptr as *const DWORD).add(2)) == chunk_id)
        {
            ptr = ptr.add(core::mem::size_of::<DWORD>());
            (*chunk).data_size = (*(ptr as *const DWORD) + 1) & !1;
            ptr = ptr.add(core::mem::size_of::<DWORD>());
            if chunk_type == ANI_LIST_ID || chunk_type == ANI_RIFF_ID {
                ptr = ptr.add(core::mem::size_of::<DWORD>());
            }
            (*chunk).data = ptr;

            return;
        }

        ptr = ptr.add(core::mem::size_of::<DWORD>());
        ptr = ptr.add(((*(ptr as *const DWORD) + 1) & !1) as usize);
        ptr = ptr.add(core::mem::size_of::<DWORD>());
    }
}

unsafe fn CURSORICON_GetCursorDataFromANI(
    pCurData: *mut CURSORDATA,
    pData: *const BYTE,
    dwDataSize: DWORD,
    fuLoad: DWORD,
) -> BOOL {
    let mut i: UINT;
    let pHeader: *const ani_header;
    let mut root_chunk = riff_chunk_t {
        data_size: dwDataSize,
        data: pData,
    };
    let mut ACON_chunk = riff_chunk_t {
        data_size: 0,
        data: core::ptr::null(),
    };
    let mut anih_chunk = riff_chunk_t {
        data_size: 0,
        data: core::ptr::null(),
    };
    let mut fram_chunk = riff_chunk_t {
        data_size: 0,
        data: core::ptr::null(),
    };
    let mut rate_chunk = riff_chunk_t {
        data_size: 0,
        data: core::ptr::null(),
    };
    let mut seq_chunk = riff_chunk_t {
        data_size: 0,
        data: core::ptr::null(),
    };
    let mut icon_chunk: *const core::ffi::c_uchar;
    let mut icon_data: *const core::ffi::c_uchar;

    /* Find the root chunk */
    riff_find_chunk(ANI_ACON_ID, ANI_RIFF_ID, &root_chunk, &mut ACON_chunk);
    if ACON_chunk.data.is_null() {
        ERR!("Failed to get root chunk.\n");
        return 0;
    }

    /* Find the header chunk */
    riff_find_chunk(ANI_anih_ID, 0, &ACON_chunk, &mut anih_chunk);
    if ACON_chunk.data.is_null() {
        ERR!("Failed to get header chunk.\n");
        return 0;
    }
    pHeader = anih_chunk.data as *const ani_header;
    dump_ani_header(pHeader);

    /* Set up the master data */
    (*pCurData).CURSORF_flags |= CURSORF_ACON;
    (*pCurData).cpcur = (*pHeader).num_frames;
    (*pCurData).cicur = (*pHeader).num_steps;
    (*pCurData).iicur = (*pHeader).display_rate;

    /* Get the sequences */
    if (*pHeader).flags & ANI_FLAG_SEQUENCE != 0 {
        riff_find_chunk(ANI_seq__ID, 0, &ACON_chunk, &mut seq_chunk);
        if seq_chunk.data.is_null() {
            ERR!("No sequence data although the flag is set!\n");
            return 0;
        }
    }

    /* Get the frame rates */
    riff_find_chunk(ANI_rate_ID, 0, &ACON_chunk, &mut rate_chunk);
    if !rate_chunk.data.is_null() {
        (*pCurData).ajifRate = rate_chunk.data as *mut INT;
    }

    /* Get the frames chunk */
    riff_find_chunk(ANI_fram_ID, ANI_LIST_ID, &ACON_chunk, &mut fram_chunk);
    if fram_chunk.data.is_null() {
        ERR!("Failed to get icon list.\n");
        return 0;
    }
    icon_chunk = fram_chunk.data;
    icon_data = fram_chunk.data.add(2 * core::mem::size_of::<DWORD>());

    if (*pHeader).num_frames > 1 {
        /* Allocate frame descriptors, step indices and rates */
        (*pCurData).aspcur = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*pHeader).num_frames as usize * core::mem::size_of::<CURSORDATA>()
                + (*pHeader).num_steps as usize
                    * (core::mem::size_of::<DWORD>() + core::mem::size_of::<INT>()),
        ) as *mut CURSORDATA;
        if (*pCurData).aspcur.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
        (*pCurData).aicur = (*pCurData).aspcur.add((*pHeader).num_frames as usize) as *mut DWORD;
        (*pCurData).ajifRate =
            (*pCurData).aicur.add((*pHeader).num_steps as usize) as *mut INT;
    }

    i = 0;
    while i < (*pHeader).num_frames {
        let pFrameData: *mut CURSORDATA;
        let chunk_size: DWORD =
            *(icon_chunk.add(core::mem::size_of::<DWORD>()) as *const DWORD);
        let pbmi: *const BITMAPINFO;

        if (*pHeader).num_frames > 1 {
            pFrameData = (*pCurData).aspcur.add(i as usize);
        } else {
            pFrameData = pCurData;
        }

        (*pFrameData).rt = (*pCurData).rt;

        if (*pHeader).flags & ANI_FLAG_ICON != 0 {
            /* The chunks describe an icon file */
            let pDirEntry = get_best_icon_file_entry(
                icon_data as *const CURSORICONFILEDIR,
                chunk_size,
                (*pCurData).cx as core::ffi::c_int,
                (*pCurData).cy as core::ffi::c_int,
                1,
                fuLoad,
            );
            if pDirEntry.is_null() {
                ERR!("Unable to find the right file entry for frame %d.\n");
                HeapFree(GetProcessHeap(), 0, (*pCurData).aspcur as LPVOID);
                core::ptr::write_bytes(pCurData, 0, 1);
                return 0;
            }
            (*pFrameData).xHotspot = (*pDirEntry).__bindgen_anon_1.xHotspot as SHORT;
            (*pFrameData).yHotspot = (*pDirEntry).__bindgen_anon_2.yHotspot as SHORT;
            if (*pHeader).width == 0 || (*pHeader).height == 0 {
                (*pFrameData).cx = (*pDirEntry).bWidth as _;
                (*pFrameData).cy = (*pDirEntry).bHeight as _;
            } else {
                (*pFrameData).cx = (*pHeader).width as _;
                (*pFrameData).cy = (*pHeader).height as _;
            }
            pbmi = icon_data.add((*pDirEntry).dwDIBOffset as usize) as *const BITMAPINFO;
        } else {
            /* The chunks just describe bitmaps */
            pbmi = icon_data as *const BITMAPINFO;
            (*pFrameData).yHotspot = 0;
            (*pFrameData).xHotspot = 0;
        }

        /* Do the real work */
        CURSORICON_GetCursorDataFromBMI(pFrameData, pbmi);

        if (*pHeader).num_frames > 1 {
            (*pFrameData).CURSORF_flags |= CURSORF_ACONFRAME;
        } else {
            (*pFrameData).CURSORF_flags &= !CURSORF_ACON;
        }

        /* Next frame */
        icon_chunk = icon_chunk.add(chunk_size as usize + (2 * core::mem::size_of::<DWORD>()));
        icon_data = icon_chunk.add(2 * core::mem::size_of::<DWORD>());

        i += 1;
    }

    if (*pHeader).num_frames <= 1 {
        return 1;
    }

    if !rate_chunk.data.is_null() {
        CopyMemory(
            (*pCurData).ajifRate as LPVOID,
            rate_chunk.data as LPVOID,
            (*pHeader).num_steps as usize * core::mem::size_of::<INT>(),
        );
    } else {
        i = 0;
        while i < (*pHeader).num_steps {
            *(*pCurData).ajifRate.add(i as usize) = (*pHeader).display_rate as INT;
            i += 1;
        }
    }

    if (*pHeader).flags & ANI_FLAG_SEQUENCE != 0 {
        CopyMemory(
            (*pCurData).aicur as LPVOID,
            seq_chunk.data as LPVOID,
            (*pHeader).num_steps as usize * core::mem::size_of::<DWORD>(),
        );
    } else {
        i = 0;
        while i < (*pHeader).num_steps {
            *(*pCurData).aicur.add(i as usize) = i;
            i += 1;
        }
    }

    1
}

unsafe fn BITMAP_LoadImageW(
    mut hinst: HINSTANCE,
    lpszName: LPCWSTR,
    mut cxDesired: core::ffi::c_int,
    mut cyDesired: core::ffi::c_int,
    fuLoad: u32,
) -> HBITMAP {
    let pbmi: *const BITMAPINFO;
    let mut pbmiScaled: *mut BITMAPINFO = core::ptr::null_mut();
    let mut pbmiCopy: *mut BITMAPINFO = core::ptr::null_mut();
    let mut pvMapping: *const core::ffi::c_void = core::ptr::null();
    let mut dwOffset: u32 = 0;
    let mut hgRsrc: HGLOBAL = core::ptr::null_mut();
    let iBMISize: core::ffi::c_int;
    let mut pvBits: *mut core::ffi::c_void;
    let mut hdcScreen: HDC = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let hbmpOld: HBITMAP;
    let mut hbmpRet: HBITMAP = core::ptr::null_mut();
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut bpp: u16 = 0;
    let mut compr: u32 = 0;
    let mut ResSize: u32 = 0;

    'end: {
        if (fuLoad & LR_LOADFROMFILE) != 0 {
            let pbmfh: *const BITMAPFILEHEADER;

            pvMapping = map_fileW(lpszName, core::ptr::null_mut());
            if pvMapping.is_null() {
                return core::ptr::null_mut();
            }
            pbmfh = pvMapping as *const BITMAPFILEHEADER;
            if (*pbmfh).bfType != 0x4d42 {
                WARN!(b"Invalid/unsupported bitmap format!\n\0".as_ptr());
                pbmi = core::ptr::null();
                break 'end;
            }
            pbmi = pbmfh.add(1) as *const BITMAPINFO;

            if (*pbmfh).bfOffBits != 0 {
                dwOffset = (*pbmfh).bfOffBits - core::mem::size_of::<BITMAPFILEHEADER>() as u32;
            }
        } else {
            let hrsrc: HRSRC;

            if hinst.is_null() {
                hinst = User32Instance;
            }
            hrsrc = FindResourceW(hinst, lpszName, RT_BITMAP);
            if hrsrc.is_null() {
                return core::ptr::null_mut();
            }
            hgRsrc = LoadResource(hinst, hrsrc);
            if hgRsrc.is_null() {
                return core::ptr::null_mut();
            }
            pbmi = LockResource(hgRsrc) as *const BITMAPINFO;
            if pbmi.is_null() {
                return core::ptr::null_mut();
            }
            ResSize = SizeofResource(hinst, hrsrc);
        }

        if (*pbmi).bmiHeader.biCompression == BI_BITFIELDS
            && (*pbmi).bmiHeader.biBitCount == 32
        {
            let totalSize: usize = (*pbmi).bmiHeader.biSize as usize
                + (3 * core::mem::size_of::<u32>())
                + (*pbmi).bmiHeader.biSizeImage as usize;
            if (*pbmi).bmiHeader.biSizeImage != 0 && totalSize != ResSize as usize {
                WARN!(b"Possibly bad resource size provided\n\0".as_ptr());
            }
        }

        if DIB_GetBitmapInfo(
            core::ptr::addr_of!((*pbmi).bmiHeader),
            &mut width,
            &mut height,
            &mut bpp,
            &mut compr,
        ) == -1
        {
            break 'end;
        }
        if width > 65535 || height > 65535 {
            break 'end;
        }
        if cxDesired == 0 {
            cxDesired = width;
        }
        if cyDesired == 0 {
            cyDesired = height;
        } else if height < 0 {
            cyDesired = -cyDesired;
        }

        iBMISize = bitmap_info_size(pbmi, DIB_RGB_COLORS as WORD);

        pvBits = (pbmi as *mut u8)
            .add(if dwOffset != 0 { dwOffset as usize } else { iBMISize as usize })
            as *mut core::ffi::c_void;

        pbmiCopy = HeapAlloc(GetProcessHeap(), 0, iBMISize as usize) as *mut BITMAPINFO;
        if pbmiCopy.is_null() {
            break 'end;
        }
        CopyMemory(
            pbmiCopy as *mut core::ffi::c_void,
            pbmi as *const core::ffi::c_void,
            iBMISize as usize,
        );

        TRACE!(
            b"Size Image %d, Size Header %d, ResSize %d\n\0".as_ptr(),
            (*pbmiCopy).bmiHeader.biSizeImage,
            (*pbmiCopy).bmiHeader.biSize,
            ResSize,
        );

        if compr == BI_BITFIELDS
            && (bpp == 16 || bpp == 32)
            && (*pbmiCopy).bmiHeader.biSizeImage + (*pbmiCopy).bmiHeader.biSize == ResSize
        {
            pvBits = (pvBits as *mut u8).sub(12) as *mut core::ffi::c_void; // nolint: magic-offset (Wine GCC-resource workaround: 3 DWORD masks)
            WARN!(b"Found GCC Resource Compiled 16-bpp or 32-bpp error\n\0".as_ptr());
        }

        'create_bitmap: {
            if (fuLoad & (LR_LOADTRANSPARENT | LR_LOADMAP3DCOLORS)) != 0 {
                let bpp2: u16;
                let incr: u16;
                let mut numColors: u16;
                let pbmiColors: *mut u8;
                let ptr: *mut RGBTRIPLE;
                let mut crWindow: COLORREF;
                let cr3DShadow: COLORREF;
                let cr3DFace: COLORREF;
                let cr3DLight: COLORREF;
                let mut pixel: u8 = *(pvBits as *mut u8);
                let mut i: u32;

                if (*pbmiCopy).bmiHeader.biSize as usize
                    == core::mem::size_of::<BITMAPCOREHEADER>()
                {
                    bpp2 = (*(core::ptr::addr_of!((*pbmiCopy).bmiHeader)
                        as *const BITMAPCOREHEADER))
                        .bcBitCount;
                    numColors = 1u16 << bpp2;
                    incr = 3;
                } else {
                    bpp2 = (*pbmiCopy).bmiHeader.biBitCount;
                    incr = 4;
                    numColors = (*pbmiCopy).bmiHeader.biClrUsed as u16;
                    if numColors > 256 {
                        numColors = 256;
                    }
                    if numColors == 0 && bpp2 <= 8 {
                        numColors = 1u16 << bpp2;
                    }
                }

                if bpp2 > 8 {
                    break 'create_bitmap;
                }

                pbmiColors = (pbmiCopy as *mut u8).add((*pbmiCopy).bmiHeader.biSize as usize);

                crWindow = GetSysColor(COLOR_WINDOW as core::ffi::c_int);
                cr3DShadow = GetSysColor(COLOR_3DSHADOW as core::ffi::c_int);
                cr3DFace = GetSysColor(COLOR_3DFACE as core::ffi::c_int);
                cr3DLight = GetSysColor(COLOR_3DLIGHT as core::ffi::c_int);

                if (fuLoad & LR_LOADTRANSPARENT) != 0 {
                    match bpp2 {
                        1 => pixel >>= 7,
                        4 => pixel >>= 4,
                        8 => {}
                        _ => {
                            FIXME!("Unhandled bit depth %d.\n", bpp2 as core::ffi::c_uint);
                            break 'create_bitmap;
                        }
                    }

                    if pixel as u16 >= numColors {
                        ERR!("Wrong pixel passed in.\n");
                        break 'create_bitmap;
                    }

                    if (fuLoad & LR_LOADMAP3DCOLORS) != 0 {
                        crWindow = cr3DFace;
                    }

                    ptr = pbmiColors.add(pixel as usize * incr as usize) as *mut RGBTRIPLE;
                    (*ptr).rgbtBlue = GetBValue(crWindow);
                    (*ptr).rgbtGreen = GetGValue(crWindow);
                    (*ptr).rgbtRed = GetRValue(crWindow);
                    break 'create_bitmap;
                }

                i = 0;
                while i < numColors as u32 {
                    let p = pbmiColors.add(i as usize * incr as usize) as *mut RGBTRIPLE;
                    if (*p).rgbtBlue == (*p).rgbtRed && (*p).rgbtBlue == (*p).rgbtGreen {
                        if (*p).rgbtBlue == 128 {
                            (*p).rgbtBlue = GetBValue(cr3DShadow);
                            (*p).rgbtGreen = GetGValue(cr3DShadow);
                            (*p).rgbtRed = GetRValue(cr3DShadow);
                        }
                        if (*p).rgbtBlue == 192 {
                            (*p).rgbtBlue = GetBValue(cr3DFace);
                            (*p).rgbtGreen = GetGValue(cr3DFace);
                            (*p).rgbtRed = GetRValue(cr3DFace);
                        }
                        if (*p).rgbtBlue == 223 {
                            (*p).rgbtBlue = GetBValue(cr3DLight);
                            (*p).rgbtGreen = GetGValue(cr3DLight);
                            (*p).rgbtRed = GetRValue(cr3DLight);
                        }
                    }
                    i += 1;
                }
            }
        }

        if (fuLoad & LR_CREATEDIBSECTION) != 0 {
            pbmiScaled = HeapAlloc(GetProcessHeap(), 0, iBMISize as usize) as *mut BITMAPINFO;
            if pbmiScaled.is_null() {
                break 'end;
            }
            CopyMemory(
                pbmiScaled as *mut core::ffi::c_void,
                pbmiCopy as *const core::ffi::c_void,
                iBMISize as usize,
            );

            if (*pbmiScaled).bmiHeader.biSize as usize
                == core::mem::size_of::<BITMAPCOREHEADER>()
            {
                let pbmch =
                    core::ptr::addr_of_mut!((*pbmiScaled).bmiHeader) as *mut BITMAPCOREHEADER;
                (*pbmch).bcWidth = cxDesired as u16;
                (*pbmch).bcHeight = cyDesired as u16;
            } else {
                (*pbmiScaled).bmiHeader.biWidth = cxDesired;
                (*pbmiScaled).bmiHeader.biHeight = cyDesired;
                (*pbmiScaled).bmiHeader.biCompression = BI_RGB;
            }
        }

        if cyDesired < 0 {
            cyDesired = -cyDesired;
        }

        hdcScreen = CreateDCW(
            DISPLAYW.as_ptr(),
            core::ptr::null(),
            core::ptr::null(),
            core::ptr::null(),
        );
        if hdcScreen.is_null() {
            break 'end;
        }
        hdc = CreateCompatibleDC(hdcScreen);
        if hdc.is_null() {
            break 'end;
        }

        if (fuLoad & LR_CREATEDIBSECTION) != 0 {
            hbmpRet = CreateDIBSection(
                hdc,
                pbmiScaled,
                DIB_RGB_COLORS,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                0,
            );
        } else {
            if is_dib_monochrome(pbmiCopy) != 0 || (fuLoad & LR_MONOCHROME) != 0 {
                hbmpRet = CreateBitmap(cxDesired, cyDesired, 1, 1, core::ptr::null());
            } else {
                hbmpRet = CreateCompatibleBitmap(hdcScreen, cxDesired, cyDesired);
            }
        }

        if hbmpRet.is_null() {
            break 'end;
        }

        hbmpOld = SelectObject(hdc, hbmpRet as *mut core::ffi::c_void) as HBITMAP;
        if hbmpOld.is_null() {
            break 'end;
        }
        if StretchDIBits(
            hdc,
            0,
            0,
            cxDesired,
            cyDesired,
            0,
            0,
            width,
            height,
            pvBits,
            pbmiCopy,
            DIB_RGB_COLORS,
            SRCCOPY,
        ) == 0
        {
            ERR!("StretchDIBits failed!.\n");
            SelectObject(hdc, hbmpOld as *mut core::ffi::c_void);
            DeleteObject(hbmpRet as *mut core::ffi::c_void);
            hbmpRet = core::ptr::null_mut();
            break 'end;
        }

        SelectObject(hdc, hbmpOld as *mut core::ffi::c_void);
    }

    if !hdcScreen.is_null() {
        DeleteDC(hdcScreen);
    }
    if !hdc.is_null() {
        DeleteDC(hdc);
    }
    if !pbmiScaled.is_null() {
        HeapFree(GetProcessHeap(), 0, pbmiScaled as *mut core::ffi::c_void);
    }
    if !pbmiCopy.is_null() {
        HeapFree(GetProcessHeap(), 0, pbmiCopy as *mut core::ffi::c_void);
    }
    if !pvMapping.is_null() {
        UnmapViewOfFile(pvMapping);
    }
    if !hgRsrc.is_null() {
        FreeResource(hgRsrc);
    }

    hbmpRet
}

unsafe fn CURSORICON_LoadFromFileW(
    lpszName: LPCWSTR,
    mut cxDesired: core::ffi::c_int,
    mut cyDesired: core::ffi::c_int,
    fuLoad: UINT,
    bIcon: BOOL,
) -> HANDLE {
    let mut entry: *const CURSORICONFILEDIRENTRY;
    let mut dir: *const CURSORICONFILEDIR;
    let mut filesize: DWORD = 0;
    let mut BmpIconSize: DWORD = 0;
    let bits: PBYTE;
    let mut pbBmpIcon: PBYTE = core::ptr::null_mut();
    let mut hCurIcon: HANDLE = core::ptr::null_mut();
    let mut cursorData: CURSORDATA = core::mem::zeroed();

    bits = map_fileW(lpszName, &mut filesize) as PBYTE;
    if bits.is_null() {
        return core::ptr::null_mut();
    }

    'end: {
        /* Check for .ani. */
        if memcmp(bits as *const core::ffi::c_void, b"RIFF\0".as_ptr() as *const core::ffi::c_void, 4) == 0 {
            break 'end;
        }

        dir = bits as *const CURSORICONFILEDIR;
        entry = get_best_icon_file_entry(dir, filesize, cxDesired, cyDesired, bIcon, fuLoad);
        if entry.is_null() {
            break 'end;
        }

        /* Fix dimensions */
        if cxDesired == 0 {
            cxDesired = (*entry).bWidth as core::ffi::c_int;
        }
        if cyDesired == 0 {
            cyDesired = (*entry).bHeight as core::ffi::c_int;
        }
        /* A bit of preparation */
        if bIcon == 0 {
            cursorData.xHotspot = (*entry).__bindgen_anon_1.xHotspot as SHORT;
            cursorData.yHotspot = (*entry).__bindgen_anon_2.yHotspot as SHORT;
        }
        cursorData.rt = LOWORD((if bIcon != 0 { RT_ICON } else { RT_CURSOR }) as usize as DWORD);

        /* Try to load BMP icon */
        if CURSORICON_GetCursorDataFromBMI(
            &mut cursorData,
            bits.add((*entry).dwDIBOffset as usize) as *const BITMAPINFO,
        ) == 0
        {
            /* Convert PNG raw data to BMP icon if the icon was PNG icon */
            let pngBits: PBYTE = bits.add((*entry).dwDIBOffset as usize);
            pbBmpIcon = CURSORICON_ConvertPngToBmpIcon(pngBits, filesize, &mut BmpIconSize);
            if pbBmpIcon.is_null() {
                break 'end;
            }

            /* Find icon entry from BMP icon */
            dir = pbBmpIcon as *const CURSORICONFILEDIR;
            entry = core::ptr::addr_of!((*dir).idEntries[0]);

            /* A bit of preparation */
            core::ptr::write_bytes(&mut cursorData as *mut CURSORDATA, 0, 1);
            cursorData.rt = LOWORD((if bIcon != 0 { RT_ICON } else { RT_CURSOR }) as usize as DWORD);

            /* Can we load this BMP icon? */
            if CURSORICON_GetCursorDataFromBMI(
                &mut cursorData,
                pbBmpIcon.add((*entry).dwDIBOffset as usize) as *const BITMAPINFO,
            ) == 0
            {
                break 'end;
            }
        }

        hCurIcon = NtUserxCreateEmptyCurObject(0) as HANDLE;
        if hCurIcon.is_null() {
            break 'end;
        }

        /* Tell win32k */
        if NtUserSetCursorIconData(
            hCurIcon as HCURSOR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut cursorData,
        ) == 0
        {
            NtUserDestroyCursor(hCurIcon as HANDLE, 1);
            hCurIcon = core::ptr::null_mut();
        }
    }

    if hCurIcon.is_null() {
        if !cursorData.hbmMask.is_null() {
            DeleteObject(cursorData.hbmMask as HGDIOBJ);
        }
        if !cursorData.hbmColor.is_null() {
            DeleteObject(cursorData.hbmColor as HGDIOBJ);
        }
        if !cursorData.hbmAlpha.is_null() {
            DeleteObject(cursorData.hbmAlpha as HGDIOBJ);
        }
    }
    HeapFree(GetProcessHeap(), 0, pbBmpIcon as LPVOID);
    UnmapViewOfFile(bits as LPCVOID);
    hCurIcon
}

unsafe fn CURSORICON_LoadImageW(
    mut hinst: HINSTANCE,
    mut lpszName: LPCWSTR,
    mut cxDesired: core::ffi::c_int,
    mut cyDesired: core::ffi::c_int,
    fuLoad: UINT,
    bIcon: BOOL,
) -> HANDLE {

    type DWORD = u32;
    type UINT = core::ffi::c_uint;
    type BOOL = core::ffi::c_int;
    type WORD = u16;
    type USHORT = u16;
    type WCHAR = u16;
    type SHORT = i16;
    type HANDLE = *mut core::ffi::c_void;

    const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
    const UNICODE_NULL: WCHAR = 0;
    const RT_ICON: usize = 3;
    const RT_CURSOR: usize = 1;
    const RT_GROUP_ICON: usize = 14;
    const RT_GROUP_CURSOR: usize = 12;

    let mut hCurIcon: HANDLE = core::ptr::null_mut();
    let mut cursorData: CURSORDATA = core::mem::zeroed();

    let mut ustrRsrc: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrModule: _UNICODE_STRING = core::mem::zeroed();
    ustrModule.Length = 0;
    ustrModule.MaximumLength = 0;
    ustrModule.Buffer = core::ptr::null_mut();

    /* Fix width/height */
    if (fuLoad & LR_DEFAULTSIZE) != 0 {
        if cxDesired == 0 {
            cxDesired = GetSystemMetrics(if bIcon != 0 { SM_CXICON } else { SM_CXCURSOR } as core::ffi::c_int);
        }
        if cyDesired == 0 {
            cyDesired = GetSystemMetrics(if bIcon != 0 { SM_CYICON } else { SM_CYCURSOR } as core::ffi::c_int);
        }
    }

    if (fuLoad & LR_LOADFROMFILE) != 0 {
        return CURSORICON_LoadFromFileW(lpszName, cxDesired, cyDesired, fuLoad, bIcon);
    }

    /* Check if caller wants OEM icons */
    if hinst.is_null() {
        let idi_application: LPCWSTR = 32512usize as LPCWSTR;
        let idi_shield: LPCWSTR = 32518usize as LPCWSTR;

        hinst = User32Instance;

        if bIcon != 0 && lpszName as usize >= idi_application as usize && lpszName as usize <= idi_shield as usize {
            let mut id: usize = 100 + (lpszName as usize) - (idi_application as usize);
            if (id | 2) == 103 {
                id ^= 2;
            }
            lpszName = id as LPCWSTR;
        }
    }

    if !lpszName.is_null() {
        /* IS_INTRESOURCE: high bits zero */
        if (lpszName as usize) >> 16 == 0 {
            ustrRsrc.Buffer = lpszName as LPWSTR;
            ustrRsrc.Length = 0;
            ustrRsrc.MaximumLength = 0;
        } else {
            RtlInitUnicodeString(&mut ustrRsrc, lpszName);
        }
    }

    /* LDR_IS_RESOURCE: low two bits set */
    if (hinst as usize) & 3 == 3 {
        // C: fakeNameFmt = (sizeof(void*) > 4) ? L"\1%016IX" : L"\1%08IX"
        // Emit as static NUL-terminated UTF-16 arrays (no heap, no dangling Vec).
        static FMT64: [u16; 9] = [
            0x01, b'%' as u16, b'0' as u16, b'1' as u16, b'6' as u16,
            b'I' as u16, b'X' as u16, 0, 0,
        ];
        static FMT32: [u16; 8] = [
            0x01, b'%' as u16, b'0' as u16, b'8' as u16,
            b'I' as u16, b'X' as u16, 0, 0,
        ];
        let fakeNameFmt: LPCWSTR = if core::mem::size_of::<*mut core::ffi::c_void>() > 4 {
            FMT64.as_ptr()
        } else {
            FMT32.as_ptr()
        };
        ustrModule.MaximumLength = (18 * core::mem::size_of::<WCHAR>()) as u16;
        ustrModule.Buffer = HeapAlloc(GetProcessHeap(), 0, ustrModule.MaximumLength as usize) as LPWSTR;
        if ustrModule.Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return core::ptr::null_mut();
        }
        ustrModule.Length = (wsprintfW(ustrModule.Buffer, fakeNameFmt, hinst) as u16).wrapping_mul(core::mem::size_of::<WCHAR>() as u16);
    } else if !hinst.is_null() {
        let mut size: DWORD = MAX_PATH as DWORD;
        loop {
            ustrModule.Buffer = HeapAlloc(GetProcessHeap(), 0, (size as usize) * core::mem::size_of::<WCHAR>()) as LPWSTR;
            if ustrModule.Buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return core::ptr::null_mut();
            }
            let ret = GetModuleFileNameW(hinst as HMODULE, ustrModule.Buffer, size);
            if ret == 0 {
                HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
                return core::ptr::null_mut();
            }
            if ret == size {
                HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
                size *= 2;
                continue;
            }
            *ustrModule.Buffer.add(ret as usize) = UNICODE_NULL;
            ustrModule.Length = (ret * core::mem::size_of::<WCHAR>() as DWORD) as u16;
            ustrModule.MaximumLength = (size * core::mem::size_of::<WCHAR>() as DWORD) as u16;
            break;
        }
    }

    if (fuLoad & LR_SHARED) != 0 {
        let mut param: FINDEXISTINGCURICONPARAM = core::mem::zeroed();
        param.bIcon = bIcon as i32;
        param.cx = cxDesired as i32;
        param.cy = cyDesired as i32;
        hCurIcon = NtUserFindExistingCursorIcon(&mut ustrModule, &mut ustrRsrc, &mut param) as HANDLE;
        if !hCurIcon.is_null() {
            HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
            return hCurIcon;
        }
    }

    'done: {
        /* Find resource ID */
        let mut hrsrc = FindResourceW(
            hinst as HMODULE,
            lpszName,
            (if bIcon != 0 { RT_GROUP_ICON } else { RT_GROUP_CURSOR }) as LPCWSTR,
        );
        if hrsrc.is_null() {
            break 'done;
        }

        let mut handle = LoadResource(hinst as HMODULE, hrsrc);
        if handle.is_null() {
            break 'done;
        }

        let dir = LockResource(handle) as *mut CURSORICONDIR;
        if dir.is_null() {
            break 'done;
        }

        let wResId = LookupIconIdFromDirectoryEx(dir as PBYTE, bIcon, cxDesired, cyDesired, fuLoad);
        FreeResource(handle);

        hrsrc = FindResourceW(
            hinst as HMODULE,
            wResId as usize as LPCWSTR,
            (if bIcon != 0 { RT_ICON } else { RT_CURSOR }) as LPCWSTR,
        );
        if hrsrc.is_null() {
            break 'done;
        }

        handle = LoadResource(hinst as HMODULE, hrsrc);
        if handle.is_null() {
            break 'done;
        }

        let mut bits = LockResource(handle) as LPBYTE;
        if bits.is_null() {
            FreeResource(handle);
            break 'done;
        }

        core::ptr::write_bytes(&mut cursorData as *mut CURSORDATA, 0, 1);

        cursorData.CURSORF_flags = CURSORF_FROMRESOURCE;

        if (*dir).idType == 2 {
            let ptr = bits as *const SHORT;
            cursorData.xHotspot = *ptr.add(0);
            cursorData.yHotspot = *ptr.add(1);
            bits = bits.add(2 * core::mem::size_of::<SHORT>());
        }
        cursorData.cx = cxDesired as u32;
        cursorData.cy = cyDesired as u32;
        cursorData.rt = (if bIcon != 0 { RT_ICON } else { RT_CURSOR }) as USHORT;

        let mut bStatus: BOOL = CURSORICON_GetCursorDataFromBMI(&mut cursorData, bits as *const BITMAPINFO) as BOOL;

        FreeResource(handle);

        if bStatus == 0 {
            break 'done;
        }

        hCurIcon = NtUserxCreateEmptyCurObject(0) as HANDLE;
        if hCurIcon.is_null() {
            if !ustrModule.Buffer.is_null() {
                HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
            }
            DeleteObject(cursorData.hbmMask as HGLOBAL);
            if !cursorData.hbmColor.is_null() {
                DeleteObject(cursorData.hbmColor as HGLOBAL);
            }
            if !cursorData.hbmAlpha.is_null() {
                DeleteObject(cursorData.hbmAlpha as HGLOBAL);
            }
            return core::ptr::null_mut();
        }

        if (fuLoad & LR_SHARED) != 0 {
            cursorData.CURSORF_flags |= CURSORF_LRSHARED;
        }

        bStatus = NtUserSetCursorIconData(
            hCurIcon as HICON,
            if !hinst.is_null() { &mut ustrModule } else { core::ptr::null_mut() },
            if !lpszName.is_null() { &mut ustrRsrc } else { core::ptr::null_mut() },
            &mut cursorData,
        );

        if bStatus == 0 {
            NtUserDestroyCursor(hCurIcon as HANDLE, 1);
            if !ustrModule.Buffer.is_null() {
                HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
            }
            DeleteObject(cursorData.hbmMask as HGLOBAL);
            if !cursorData.hbmColor.is_null() {
                DeleteObject(cursorData.hbmColor as HGLOBAL);
            }
            if !cursorData.hbmAlpha.is_null() {
                DeleteObject(cursorData.hbmAlpha as HGLOBAL);
            }
            return core::ptr::null_mut();
        }
    }

    if !ustrModule.Buffer.is_null() {
        HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
    }
    hCurIcon
}

unsafe fn BITMAP_CopyImage(
    hnd: HBITMAP,
    mut desiredx: core::ffi::c_int,
    mut desiredy: core::ffi::c_int,
    flags: UINT,
) -> HBITMAP {
    let mut res: HBITMAP = core::ptr::null_mut();
    let mut ds: DIBSECTION = core::mem::zeroed();
    let objSize: core::ffi::c_int;
    let bi: *mut BITMAPINFO;

    objSize = GetObjectW(hnd as LPVOID, core::mem::size_of::<DIBSECTION>() as core::ffi::c_int, core::ptr::addr_of_mut!(ds) as LPVOID);
    if objSize == 0 {
        return core::ptr::null_mut();
    }
    if desiredx < 0 || desiredy < 0 {
        return core::ptr::null_mut();
    }

    if flags & LR_COPYFROMRESOURCE != 0 {
        FIXME!("The flag LR_COPYFROMRESOURCE is not implemented for bitmaps\n");
    }

    if flags & LR_COPYRETURNORG != 0 {
        FIXME!("The flag LR_COPYRETURNORG is not implemented for bitmaps\n");
    }

    if desiredx == 0 {
        desiredx = ds.dsBm.bmWidth;
    }
    if desiredy == 0 {
        desiredy = ds.dsBm.bmHeight;
    }

    bi = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>(),
    ) as *mut BITMAPINFO;
    if bi.is_null() {
        return core::ptr::null_mut();
    }

    (*bi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    (*bi).bmiHeader.biPlanes = ds.dsBm.bmPlanes;
    (*bi).bmiHeader.biBitCount = ds.dsBm.bmBitsPixel;
    (*bi).bmiHeader.biCompression = BI_RGB as u32;

    if flags & LR_CREATEDIBSECTION != 0 {
        let mut bits: LPVOID = core::ptr::null_mut();
        let dc: HDC = CreateCompatibleDC(core::ptr::null_mut());

        if objSize == core::mem::size_of::<DIBSECTION>() as core::ffi::c_int {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!(ds.dsBmih) as *const u8,
                bi as *mut u8,
                core::mem::size_of::<BITMAPINFOHEADER>(),
            );
        }

        (*bi).bmiHeader.biWidth = desiredx;
        (*bi).bmiHeader.biHeight = desiredy;

        GetDIBits(dc, hnd, 0, ds.dsBm.bmHeight as UINT, core::ptr::null_mut(), bi, DIB_RGB_COLORS);

        res = CreateDIBSection(dc, bi, DIB_RGB_COLORS, core::ptr::addr_of_mut!(bits), core::ptr::null_mut(), 0);
        DeleteDC(dc);
    } else {
        let mut monochrome: BOOL = (flags & LR_MONOCHROME != 0) as BOOL;

        if objSize == core::mem::size_of::<DIBSECTION>() as core::ffi::c_int {
            let dc: HDC = CreateCompatibleDC(core::ptr::null_mut());
            (*bi).bmiHeader.biWidth = ds.dsBm.bmWidth;
            (*bi).bmiHeader.biHeight = ds.dsBm.bmHeight;
            GetDIBits(dc, hnd, 0, ds.dsBm.bmHeight as UINT, core::ptr::null_mut(), bi, DIB_RGB_COLORS);
            DeleteDC(dc);

            if monochrome == 0 && ds.dsBm.bmBitsPixel == 1 {
                let colors = (*bi).bmiColors.as_ptr();
                monochrome = (((*colors.offset(0)).rgbRed == 0xff
                    && (*colors.offset(0)).rgbGreen == 0xff
                    && (*colors.offset(0)).rgbBlue == 0xff
                    && (*colors.offset(0)).rgbReserved == 0
                    && (*colors.offset(1)).rgbRed == 0
                    && (*colors.offset(1)).rgbGreen == 0
                    && (*colors.offset(1)).rgbBlue == 0
                    && (*colors.offset(1)).rgbReserved == 0)
                    || ((*colors.offset(0)).rgbRed == 0
                    && (*colors.offset(0)).rgbGreen == 0
                    && (*colors.offset(0)).rgbBlue == 0
                    && (*colors.offset(0)).rgbReserved == 0
                    && (*colors.offset(1)).rgbRed == 0xff
                    && (*colors.offset(1)).rgbGreen == 0xff
                    && (*colors.offset(1)).rgbBlue == 0xff
                    && (*colors.offset(1)).rgbReserved == 0)) as BOOL;
            }
        } else if monochrome == 0 {
            monochrome = (ds.dsBm.bmBitsPixel == 1) as BOOL;
        }

        if monochrome != 0 {
            res = CreateBitmap(desiredx, desiredy, 1, 1, core::ptr::null());
        } else {
            let screenDC: HDC = GetDC(core::ptr::null_mut());
            res = CreateCompatibleBitmap(screenDC, desiredx, desiredy);
            ReleaseDC(core::ptr::null_mut(), screenDC);
        }
    }

    if !res.is_null() {
        let copyContents: BOOL;

        if objSize == core::mem::size_of::<DIBSECTION>() as core::ffi::c_int {
            copyContents = 1;
        } else {
            let screenDC: HDC = GetDC(core::ptr::null_mut());
            let screen_depth: core::ffi::c_int = GetDeviceCaps(screenDC, BITSPIXEL as core::ffi::c_int);
            ReleaseDC(core::ptr::null_mut(), screenDC);

            copyContents = (ds.dsBm.bmBitsPixel as i32 == 1 || ds.dsBm.bmBitsPixel as i32 == screen_depth) as BOOL;
        }

        if copyContents != 0 {
            let dc: HDC;
            let bits: LPVOID;

            dc = CreateCompatibleDC(core::ptr::null_mut());

            (*bi).bmiHeader.biWidth = ds.dsBm.bmWidth;
            (*bi).bmiHeader.biHeight = ds.dsBm.bmHeight;
            (*bi).bmiHeader.biSizeImage = 0;
            (*bi).bmiHeader.biClrUsed = 0;
            (*bi).bmiHeader.biClrImportant = 0;

            GetDIBits(dc, hnd, 0, ds.dsBm.bmHeight as UINT, core::ptr::null_mut(), bi, DIB_RGB_COLORS);
            bits = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (*bi).bmiHeader.biSizeImage as usize);

            if !bits.is_null() {
                let oldBmp: HBITMAP;

                GetDIBits(dc, hnd, 0, ds.dsBm.bmHeight as UINT, bits, bi, DIB_RGB_COLORS);

                oldBmp = SelectObject(dc, res as HGDIOBJ) as HBITMAP;
                StretchDIBits(
                    dc, 0, 0, desiredx, desiredy,
                    0, 0, ds.dsBm.bmWidth, ds.dsBm.bmHeight,
                    bits, bi, DIB_RGB_COLORS, SRCCOPY,
                );
                SelectObject(dc, oldBmp as HGDIOBJ);

                HeapFree(GetProcessHeap(), 0, bits);
            }

            DeleteDC(dc);
        }

        if flags & LR_COPYDELETEORG != 0 {
            DeleteObject(hnd as HGDIOBJ);
        }
    }
    HeapFree(GetProcessHeap(), 0, bi as LPVOID);
    res
}

unsafe fn CURSORICON_CopyImage(
    hicon: HICON,
    bIcon: BOOL,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    fuFlags: UINT,
) -> HICON {
    let mut ret: HICON = core::ptr::null_mut();
    let mut ii: ICONINFO = core::mem::zeroed();
    let mut CursorData: CURSORDATA = core::mem::zeroed();

    if (fuFlags & LR_COPYFROMRESOURCE) != 0 {
        let mut ustrModule: _UNICODE_STRING = core::mem::zeroed();
        let mut ustrRsrc: _UNICODE_STRING = core::mem::zeroed();
        let hModule: HMODULE;

        ustrModule.MaximumLength = 0;
        ustrRsrc.MaximumLength = 0;

        if NtUserGetIconInfo(
            hicon as HANDLE,
            core::ptr::null_mut(),
            &mut ustrModule,
            &mut ustrRsrc,
            core::ptr::null_mut(),
            0u8 as BOOL,
        ) == 0
        {
            return core::ptr::null_mut();
        }

        ustrModule.Buffer =
            HeapAlloc(GetProcessHeap(), 0, ustrModule.MaximumLength as usize) as LPWSTR;
        if ustrModule.Buffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return core::ptr::null_mut();
        }

        if ustrRsrc.MaximumLength != 0 {
            ustrRsrc.Buffer =
                HeapAlloc(GetProcessHeap(), 0, ustrRsrc.MaximumLength as usize) as LPWSTR;
            if ustrRsrc.Buffer.is_null() {
                HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return core::ptr::null_mut();
            }
        }

        if NtUserGetIconInfo(
            hicon as HANDLE,
            core::ptr::null_mut(),
            &mut ustrModule,
            &mut ustrRsrc,
            core::ptr::null_mut(),
            0u8 as BOOL,
        ) == 0
        {
            HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
            if IS_INTRESOURCE(ustrRsrc.Buffer as LPCVOID) == 0 {
                HeapFree(GetProcessHeap(), 0, ustrRsrc.Buffer as LPVOID);
            }
            return core::ptr::null_mut();
        }

        *ustrModule
            .Buffer
            .add((ustrModule.Length as usize) / core::mem::size_of::<u16>()) = 0;
        if IS_INTRESOURCE(ustrRsrc.Buffer as LPCVOID) == 0 {
            *ustrRsrc
                .Buffer
                .add((ustrRsrc.Length as usize) / core::mem::size_of::<u16>()) = 0;
        }

        hModule = LoadLibraryExW(ustrModule.Buffer, core::ptr::null_mut(), 0x00000002);
        if hModule.is_null() {
            let _err = GetLastError();
            SetLastError(ERROR_INVALID_PARAMETER);
        } else {
            ret = CURSORICON_LoadImageW(
                hModule,
                ustrRsrc.Buffer,
                cxDesired,
                cyDesired,
                fuFlags & (LR_DEFAULTSIZE | LR_SHARED),
                bIcon,
            ) as HICON;

            FreeLibrary(hModule);
        }

        HeapFree(GetProcessHeap(), 0, ustrModule.Buffer as LPVOID);
        if IS_INTRESOURCE(ustrRsrc.Buffer as LPCVOID) == 0 {
            HeapFree(GetProcessHeap(), 0, ustrRsrc.Buffer as LPVOID);
        }

        return ret;
    }

    if GetIconInfo(hicon, &mut ii) == 0 {
        return core::ptr::null_mut();
    }

    'leave: {
        if CURSORICON_GetCursorDataFromIconInfo(&mut CursorData, &mut ii) == 0 {
            break 'leave;
        }

        if (fuFlags & LR_SHARED) != 0 {
            CursorData.CURSORF_flags |= CURSORF_LRSHARED;
        }

        ret = NtUserxCreateEmptyCurObject(0) as HICON;
        if ret.is_null() {
            break 'leave;
        }

        if NtUserSetCursorIconData(
            ret,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut CursorData,
        ) == 0
        {
            NtUserDestroyCursor(ret as HANDLE, 1);
            ret = core::ptr::null_mut();
            break 'leave;
        }
    }

    DeleteObject(ii.hbmMask as HGDIOBJ);
    if !ii.hbmColor.is_null() {
        DeleteObject(ii.hbmColor as HGDIOBJ);
    }

    if !ret.is_null() && (fuFlags & LR_COPYDELETEORG) != 0 {
        DestroyIcon(hicon);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn User32CallCopyImageFromKernel(
    Arguments: LPVOID,
    ArgumentLength: ULONG,
) -> NTSTATUS {
    let Common = Arguments as PCOPYIMAGE_CALLBACK_ARGUMENTS;

    let mut Result: HANDLE = CopyImage(
        (*Common).hImage,
        (*Common).uType,
        (*Common).cxDesired,
        (*Common).cyDesired,
        (*Common).fuFlags,
    );

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(Result) as LPVOID,
        core::mem::size_of::<HANDLE>() as ULONG,
        STATUS_SUCCESS,
    )
}

#[no_mangle]
pub unsafe extern "system" fn CopyImage(
    hImage: HANDLE,
    uType: UINT,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    fuFlags: UINT,
) -> HANDLE {
    if (fuFlags & !COPYIMAGE_VALID_FLAGS) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    match uType {
        IMAGE_BITMAP => {
            if hImage.is_null() {
                SetLastError(ERROR_INVALID_HANDLE);
            } else {
                return BITMAP_CopyImage(
                    hImage as HBITMAP,
                    cxDesired,
                    cyDesired,
                    fuFlags,
                ) as HANDLE;
            }
        }
        IMAGE_CURSOR | IMAGE_ICON => {
            if hImage.is_null() {
                SetLastError(ERROR_INVALID_CURSOR_HANDLE);
            } else {
                let mut handle = CURSORICON_CopyImage(
                    hImage as HICON,
                    (uType == IMAGE_ICON) as BOOL,
                    cxDesired,
                    cyDesired,
                    fuFlags,
                );
                if handle.is_null() && (fuFlags & LR_COPYFROMRESOURCE) != 0 {
                    let mut bm: BITMAP = core::mem::zeroed();
                    let mut iconinfo: ICONINFO = core::mem::zeroed();
                    if GetIconInfo(hImage as HICON, &mut iconinfo) == 0 {
                        ERR!("GetIconInfo Failed. hImage %p\n", hImage);
                        return core::ptr::null_mut();
                    }
                    if GetObjectW(
                        iconinfo.hbmColor as HGLOBAL,
                        core::mem::size_of::<BITMAP>() as core::ffi::c_int,
                        core::ptr::addr_of_mut!(bm) as LPVOID,
                    ) == 0
                    {
                        ERR!("GetObject Failed. iconinfo %p\n", core::ptr::addr_of!(iconinfo));
                        return core::ptr::null_mut();
                    }

                    DeleteObject(iconinfo.hbmMask as HGLOBAL);
                    DeleteObject(iconinfo.hbmColor as HGLOBAL);

                    if cxDesired == bm.bmWidth && cyDesired == bm.bmHeight {
                        handle = CURSORICON_CopyImage(
                            hImage as HICON,
                            (uType == IMAGE_ICON) as BOOL,
                            cxDesired,
                            cyDesired,
                            fuFlags & !LR_COPYFROMRESOURCE,
                        );
                    }
                }
                return handle as HANDLE;
            }
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CopyIcon(hIcon: HICON) -> HICON {
    CURSORICON_CopyImage(hIcon, 0, 0, 0, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DrawIcon(
    hDC: HDC,
    X: i32,
    Y: i32,
    hIcon: HICON,
) -> i32 {
    DrawIconEx(
        hDC,
        X,
        Y,
        hIcon,
        0,
        0,
        0,
        core::ptr::null_mut(),
        (DI_NORMAL | DI_COMPAT | DI_DEFAULTSIZE) as u32,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DrawIconEx(
    hdc: HDC,
    xLeft: i32,
    yTop: i32,
    hIcon: HICON,
    cxWidth: i32,
    cyWidth: i32,
    istepIfAniCur: u32,
    hbrFlickerFreeDraw: HBRUSH,
    diFlags: u32,
) -> i32 {
    NtUserDrawIconEx(
        hdc,
        xLeft,
        yTop,
        hIcon,
        cxWidth,
        cyWidth,
        istepIfAniCur,
        hbrFlickerFreeDraw,
        diFlags,
        0,
        core::ptr::null_mut(),
    )
}

#[no_mangle]
pub unsafe extern "system" fn GetIconInfo(hIcon: HICON, piconinfo: PICONINFO) -> i32 {
    NtUserGetIconInfo(
        hIcon as HANDLE,
        piconinfo,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    )
}

#[no_mangle]
pub unsafe extern "system" fn DestroyIcon(hIcon: HICON) -> i32 {
    NtUserDestroyCursor(hIcon as HANDLE, 0)
}

#[no_mangle]
pub unsafe extern "system" fn LoadIconA(
    hInstance: HINSTANCE,
    lpIconName: LPCSTR,
) -> HICON {
    LoadImageA(
        hInstance,
        lpIconName,
        IMAGE_ICON,
        0,
        0,
        LR_SHARED | LR_DEFAULTSIZE,
    ) as HICON
}

#[no_mangle]
pub unsafe extern "system" fn LoadIconW(
    hInstance: HINSTANCE,
    lpIconName: LPCWSTR,
) -> HICON {
    LoadImageW(
        hInstance,
        lpIconName,
        IMAGE_ICON,
        0,
        0,
        (LR_SHARED | LR_DEFAULTSIZE) as UINT,
    ) as HICON
}

#[no_mangle]
pub unsafe extern "system" fn LoadCursorA(
    hInstance: HINSTANCE,
    lpCursorName: LPCSTR,
) -> HCURSOR {
    LoadImageA(
        hInstance,
        lpCursorName,
        IMAGE_CURSOR,
        0,
        0,
        LR_SHARED | LR_DEFAULTSIZE,
    ) as HCURSOR
}

#[no_mangle]
pub unsafe extern "system" fn LoadCursorW(
    hInstance: HINSTANCE,
    lpCursorName: LPCWSTR,
) -> HCURSOR {
    LoadImageW(
        hInstance,
        lpCursorName,
        IMAGE_CURSOR,
        0,
        0,
        LR_SHARED | LR_DEFAULTSIZE,
    ) as HCURSOR
}

#[no_mangle]
pub unsafe extern "system" fn LoadCursorFromFileA(
    lpFileName: LPCSTR,
) -> HCURSOR {
    LoadImageA(
        core::ptr::null_mut(),
        lpFileName,
        IMAGE_CURSOR,
        0,
        0,
        LR_LOADFROMFILE | LR_DEFAULTSIZE,
    ) as HCURSOR
}

#[no_mangle]
pub unsafe extern "system" fn LoadCursorFromFileW(
    lpFileName: LPCWSTR,
) -> HCURSOR {
    LoadImageW(
        core::ptr::null_mut(),
        lpFileName,
        IMAGE_CURSOR,
        0,
        0,
        LR_LOADFROMFILE | LR_DEFAULTSIZE,
    ) as HCURSOR
}

#[no_mangle]
pub unsafe extern "system" fn LoadBitmapA(
    hInstance: HINSTANCE,
    lpBitmapName: LPCSTR,
) -> HBITMAP {
    LoadImageA(hInstance, lpBitmapName, IMAGE_BITMAP, 0, 0, 0) as HBITMAP
}

#[no_mangle]
pub unsafe extern "system" fn LoadBitmapW(
    hInstance: HINSTANCE,
    lpBitmapName: LPCWSTR,
) -> HBITMAP {
    LoadImageW(
        hInstance,
        lpBitmapName,
        IMAGE_BITMAP,
        0,
        0,
        0,
    ) as HBITMAP
}

#[no_mangle]
pub unsafe extern "system" fn LoadImageA(
    hinst: HINSTANCE,
    lpszName: LPCSTR,
    uType: UINT,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    fuLoad: UINT,
) -> HANDLE {
    let res: HANDLE;
    let u_name: LPWSTR;
    let len: DWORD;

    if IS_INTRESOURCE(lpszName as *const core::ffi::c_void) != 0 {
        return LoadImageW(hinst, lpszName as LPCWSTR, uType, cxDesired, cyDesired, fuLoad);
    }

    len = MultiByteToWideChar(CP_ACP, 0, lpszName, -1, core::ptr::null_mut(), 0) as DWORD;
    u_name = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<u16>(),
    ) as LPWSTR;
    MultiByteToWideChar(CP_ACP, 0, lpszName, -1, u_name, len as core::ffi::c_int);

    res = LoadImageW(hinst, u_name, uType, cxDesired, cyDesired, fuLoad);
    HeapFree(GetProcessHeap(), 0, u_name as LPVOID);
    res
}

#[no_mangle]
pub unsafe extern "system" fn LoadImageW(
    hinst: HINSTANCE,
    lpszName: LPCWSTR,
    uType: UINT,
    cxDesired: core::ffi::c_int,
    cyDesired: core::ffi::c_int,
    fuLoad: UINT,
) -> HANDLE {
    match uType {
        IMAGE_BITMAP => {
            return BITMAP_LoadImageW(hinst, lpszName, cxDesired, cyDesired, fuLoad) as HANDLE;
        }
        IMAGE_CURSOR | IMAGE_ICON => {
            return CURSORICON_LoadImageW(
                hinst,
                lpszName,
                cxDesired,
                cyDesired,
                fuLoad,
                (uType == IMAGE_ICON) as BOOL,
            );
        }
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER);
        }
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn LookupIconIdFromDirectory(
    presbits: PBYTE,
    fIcon: BOOL,
) -> core::ffi::c_int {

    LookupIconIdFromDirectoryEx(
        presbits,
        fIcon,
        if fIcon != 0 {
            GetSystemMetrics(SM_CXICON as core::ffi::c_int)
        } else {
            GetSystemMetrics(SM_CXCURSOR as core::ffi::c_int)
        },
        if fIcon != 0 {
            GetSystemMetrics(SM_CYICON as core::ffi::c_int)
        } else {
            GetSystemMetrics(SM_CYCURSOR as core::ffi::c_int)
        },
        if fIcon != 0 { 0 } else { LR_MONOCHROME },
    )
}

#[no_mangle]
pub unsafe extern "system" fn LookupIconIdFromDirectoryEx(
    presbits: PBYTE,
    fIcon: BOOL,
    mut cxDesired: core::ffi::c_int,
    mut cyDesired: core::ffi::c_int,
    Flags: UINT,
) -> core::ffi::c_int {
    let bppDesired: WORD;
    let dir = presbits as *mut CURSORICONDIR;
    let mut entry: *mut CURSORICONDIRENTRY;
    let mut i: core::ffi::c_int;
    let mut numMatch: core::ffi::c_int = 0;
    let mut iIndex: core::ffi::c_int = -1;
    let mut width: WORD;
    let mut height: WORD;
    let mut BitCount: WORD = 0;
    let mut notPaletted: BOOL = 0;
    let mut bestScore: ULONG = 0xFFFFFFFF;
    let mut score: ULONG;

    if !(!dir.is_null() && (*dir).idReserved == 0 && ((*dir).idType & 3) != 0) {
        return 0;
    }

    if Flags & LR_MONOCHROME != 0 {
        bppDesired = 1;
    } else {
        let icScreen: HDC = CreateICW(
            DISPLAYW.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if icScreen.is_null() {
            return 0;
        }
        bppDesired = GetDeviceCaps(icScreen, BITSPIXEL as core::ffi::c_int) as WORD;
        DeleteDC(icScreen);
    }

    if cxDesired == 0 {
        cxDesired = if Flags & LR_DEFAULTSIZE != 0 {
            GetSystemMetrics(if fIcon != 0 { SM_CXICON } else { SM_CXCURSOR } as core::ffi::c_int)
        } else {
            256
        };
    }
    if cyDesired == 0 {
        cyDesired = if Flags & LR_DEFAULTSIZE != 0 {
            GetSystemMetrics(if fIcon != 0 { SM_CYICON } else { SM_CYCURSOR } as core::ffi::c_int)
        } else {
            256
        };
    }

    i = 0;
    while i < (*dir).idCount as core::ffi::c_int {
        entry = (core::ptr::addr_of_mut!((*dir).idEntries) as *mut CURSORICONDIRENTRY).add(i as usize);
        width = if fIcon != 0 {
            (*entry).ResInfo.icon.bWidth as WORD
        } else {
            (*entry).ResInfo.cursor.wWidth
        };
        height = if fIcon != 0 {
            (*entry).ResInfo.icon.bHeight as WORD
        } else {
            (*entry).ResInfo.cursor.wHeight / 2
        };
        if width == 0 {
            width = 256;
        }
        if height == 0 {
            height = 256;
        }
        score = (2 * (abs(width as core::ffi::c_int - cxDesired) + abs(height as core::ffi::c_int - cyDesired))) as ULONG;
        if score > bestScore {
            i += 1;
            continue;
        }
        if width as core::ffi::c_int > cxDesired {
            score -= (width as core::ffi::c_int - cxDesired) as ULONG;
        }
        if height as core::ffi::c_int > cyDesired {
            score -= (height as core::ffi::c_int - cyDesired) as ULONG;
        }
        if score > bestScore {
            i += 1;
            continue;
        }
        if score == bestScore {
            if (*entry).wBitCount > BitCount {
                BitCount = (*entry).wBitCount;
            }
            numMatch += 1;
            i += 1;
            continue;
        }
        iIndex = i;
        numMatch = 1;
        bestScore = score;
        BitCount = (*entry).wBitCount;
        i += 1;
    }

    if numMatch == 1 {
        return (*(core::ptr::addr_of!((*dir).idEntries) as *const CURSORICONDIRENTRY).add(iIndex as usize)).wResId as core::ffi::c_int;
    }

    if bppDesired > 8 && BitCount > 8 {
        notPaletted = 1;
    }

    BitCount = 0;
    iIndex = -1;
    i = 0;
    while i < (*dir).idCount as core::ffi::c_int {
        entry = (core::ptr::addr_of_mut!((*dir).idEntries) as *mut CURSORICONDIRENTRY).add(i as usize);
        width = if fIcon != 0 {
            (*entry).ResInfo.icon.bWidth as WORD
        } else {
            (*entry).ResInfo.cursor.wWidth
        };
        height = if fIcon != 0 {
            (*entry).ResInfo.icon.bHeight as WORD
        } else {
            (*entry).ResInfo.cursor.wHeight / 2
        };
        if width == 0 {
            width = 256;
        }
        if height == 0 {
            height = 256;
        }
        score = (2 * (abs(width as core::ffi::c_int - cxDesired) + abs(height as core::ffi::c_int - cyDesired))) as ULONG;
        if width as core::ffi::c_int > cxDesired {
            score -= (width as core::ffi::c_int - cxDesired) as ULONG;
        }
        if height as core::ffi::c_int > cyDesired {
            score -= (height as core::ffi::c_int - cyDesired) as ULONG;
        }
        if score != bestScore {
            i += 1;
            continue;
        }
        if (*entry).wBitCount == bppDesired {
            return (*entry).wResId as core::ffi::c_int;
        }
        if (*entry).wBitCount > BitCount && (*entry).wBitCount < bppDesired {
            if (*entry).wBitCount <= 8 && notPaletted != 0 {
                i += 1;
                continue;
            }
            iIndex = i;
            BitCount = (*entry).wBitCount;
        }
        i += 1;
    }

    if iIndex >= 0 {
        return (*(core::ptr::addr_of!((*dir).idEntries) as *const CURSORICONDIRENTRY).add(iIndex as usize)).wResId as core::ffi::c_int;
    }

    BitCount = 0xFFFF;
    iIndex = -1;
    i = 0;
    while i < (*dir).idCount as core::ffi::c_int {
        entry = (core::ptr::addr_of_mut!((*dir).idEntries) as *mut CURSORICONDIRENTRY).add(i as usize);
        width = if fIcon != 0 {
            (*entry).ResInfo.icon.bWidth as WORD
        } else {
            (*entry).ResInfo.cursor.wWidth
        };
        height = if fIcon != 0 {
            (*entry).ResInfo.icon.bHeight as WORD
        } else {
            (*entry).ResInfo.cursor.wHeight / 2
        };
        if width == 0 {
            width = 256;
        }
        if height == 0 {
            height = 256;
        }
        score = (2 * (abs(width as core::ffi::c_int - cxDesired) + abs(height as core::ffi::c_int - cyDesired))) as ULONG;
        if width as core::ffi::c_int > cxDesired {
            score -= (width as core::ffi::c_int - cxDesired) as ULONG;
        }
        if height as core::ffi::c_int > cyDesired {
            score -= (height as core::ffi::c_int - cyDesired) as ULONG;
        }
        if score != bestScore {
            i += 1;
            continue;
        }
        if (*entry).wBitCount < BitCount {
            if (*entry).wBitCount <= 8 && notPaletted != 0 {
                i += 1;
                continue;
            }
            iIndex = i;
            BitCount = (*entry).wBitCount;
        }
        i += 1;
    }
    if iIndex >= 0 {
        return (*(core::ptr::addr_of!((*dir).idEntries) as *const CURSORICONDIRENTRY).add(iIndex as usize)).wResId as core::ffi::c_int;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn CreateIcon(
    hInstance: HINSTANCE,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
    cPlanes: BYTE,
    cBitsPixel: BYTE,
    lpbANDbits: *const BYTE,
    lpbXORbits: *const BYTE,
) -> HICON {
    let mut iinfo: ICONINFO = core::mem::zeroed();
    let hIcon: HICON;

    iinfo.fIcon = 1u8 as _;
    iinfo.xHotspot = (nWidth / 2) as _;
    iinfo.yHotspot = (nHeight / 2) as _;

    if (cPlanes as core::ffi::c_int) * (cBitsPixel as core::ffi::c_int) > 1 {
        iinfo.hbmColor = CreateBitmap(nWidth, nHeight, cPlanes as _, cBitsPixel as _, lpbXORbits as LPVOID);
        iinfo.hbmMask = CreateBitmap(nWidth, nHeight, 1, 1, lpbANDbits as LPVOID);
    } else {
        iinfo.hbmMask = CreateBitmap(nWidth, nHeight * 2, 1, 1, lpbANDbits as LPVOID);
        iinfo.hbmColor = core::ptr::null_mut();
    }

    hIcon = CreateIconIndirect(core::ptr::addr_of_mut!(iinfo));

    DeleteObject(iinfo.hbmMask as _);
    if !iinfo.hbmColor.is_null() {
        DeleteObject(iinfo.hbmColor as _);
    }

    hIcon
}

#[no_mangle]
pub unsafe extern "system" fn CreateIconFromResource(
    presbits: PBYTE,
    dwResSize: u32,
    fIcon: i32,
    dwVer: u32,
) -> HICON {
    CreateIconFromResourceEx(presbits, dwResSize, fIcon, dwVer, 0, 0, (LR_DEFAULTSIZE | LR_SHARED) as u32)
}

// Real own-export (user32.spec `@ stdcall CreateIconFromResourceEx`). The batch
// translator dropped this body; reconstructed faithfully from cursoricon.c.
#[no_mangle]
pub unsafe extern "system" fn CreateIconFromResourceEx(
    mut pbIconBits: PBYTE,
    cbIconBits: DWORD,
    fIcon: BOOL,
    dwVersion: DWORD,
    mut cxDesired: core::ffi::c_int,
    mut cyDesired: core::ffi::c_int,
    uFlags: UINT,
) -> HICON {
    let _ = dwVersion;
    let mut cursorData: CURSORDATA;
    let hIcon: HICON;
    let isAnimated: BOOL;
    let mut pbBmpIcon: PBYTE = core::ptr::null_mut();
    let mut BmpIconSize: DWORD = 0;

    if pbIconBits.is_null() || (cbIconBits as usize) < 2 * core::mem::size_of::<DWORD>() {
        ERR!("Invalid IconBits array\n");
        return core::ptr::null_mut();
    }

    if uFlags & LR_DEFAULTSIZE != 0 {
        if cxDesired == 0 {
            cxDesired = GetSystemMetrics(if fIcon != 0 { SM_CXICON } else { SM_CXCURSOR } as core::ffi::c_int);
        }
        if cyDesired == 0 {
            cyDesired = GetSystemMetrics(if fIcon != 0 { SM_CYICON } else { SM_CYCURSOR } as core::ffi::c_int);
        }
    }

    cursorData = core::mem::zeroed();
    cursorData.cx = cxDesired as ULONG;
    cursorData.cy = cyDesired as ULONG;
    cursorData.rt = LOWORD((if fIcon != 0 { RT_ICON } else { RT_CURSOR }) as usize as DWORD);

    'process: {
        /* Convert to win32k-ready data */
        if memcmp(pbIconBits as *const c_void, b"RIFF\0".as_ptr() as *const c_void, 4) == 0 {
            if CURSORICON_GetCursorDataFromANI(&mut cursorData, pbIconBits, cbIconBits, uFlags) == 0 {
                ERR!("Could not get cursor data from .ani.\n");
                return core::ptr::null_mut();
            }
            isAnimated = ((cursorData.CURSORF_flags & CURSORF_ACON) != 0) as BOOL;
            break 'process;
        }

        /* It is possible to pass Icon Directories to this API */
        let wResId = LookupIconIdFromDirectoryEx(pbIconBits, fIcon, cxDesired, cyDesired, uFlags);
        let mut ResHandle: HANDLE = core::ptr::null_mut();
        // __REACTOS__: also require the 5th byte != sizeof(BITMAPINFOHEADER)
        if wResId != 0 && (*pbIconBits.add(4) as usize != core::mem::size_of::<BITMAPINFOHEADER>()) {
            let mut hinst: HINSTANCE = core::ptr::null_mut();
            let hrsrc: HRSRC;
            let pCurIconDir = pbIconBits as *const CURSORICONDIR;

            TRACE!(b"Pointer points to a directory structure.\n\0".as_ptr());

            /* So this is a pointer to an icon directory structure. Find the module */
            if GetModuleHandleExW(
                GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                pbIconBits as LPCWSTR,
                &mut hinst as *mut HINSTANCE as *mut HMODULE,
            ) == 0
            {
                return core::ptr::null_mut();
            }

            /* Check we were given the right type of resource */
            if (fIcon != 0 && (*pCurIconDir).idType == 2) || (fIcon == 0 && (*pCurIconDir).idType == 1) {
                WARN!(b"Got a directory pointer, but called for the other type\n\0".as_ptr());
                return core::ptr::null_mut();
            }

            /* Get the relevant resource pointer */
            let hrsrc2 = FindResourceW(
                hinst as HMODULE,
                MAKEINTRESOURCEW(wResId as DWORD),
                if fIcon != 0 { RT_ICON } else { RT_CURSOR },
            );
            hrsrc = hrsrc2;
            if hrsrc.is_null() {
                return core::ptr::null_mut();
            }

            ResHandle = LoadResource(hinst as HMODULE, hrsrc);
            if ResHandle.is_null() {
                return core::ptr::null_mut();
            }

            pbIconBits = LockResource(ResHandle) as PBYTE;
            if pbIconBits.is_null() {
                FreeResource(ResHandle);
                return core::ptr::null_mut();
            }
        }

        if fIcon == 0 {
            let pt = pbIconBits as *const WORD;
            cursorData.xHotspot = *pt.add(0) as SHORT;
            cursorData.yHotspot = *pt.add(1) as SHORT;
            pbIconBits = (pt.add(2)) as PBYTE;
        }

        isAnimated = 0;

        /* Try to load BMP icon */
        if CURSORICON_GetCursorDataFromBMI(&mut cursorData, pbIconBits as *const BITMAPINFO) == 0 {
            /* Convert PNG raw data to BMP icon if the icon was PNG icon */
            pbBmpIcon = CURSORICON_ConvertPngToBmpIcon(pbIconBits, cbIconBits, &mut BmpIconSize);
            if pbBmpIcon.is_null() {
                return core::ptr::null_mut(); /* Not PNG icon or failed */
            }

            /* Find icon entry from BMP icon */
            let dir = pbBmpIcon as *const CURSORICONFILEDIR;
            let entry = core::ptr::addr_of!((*dir).idEntries[0]); /* Only one entry */

            /* A bit of preparation */
            core::ptr::write_bytes(&mut cursorData as *mut CURSORDATA, 0, 1);
            cursorData.rt = LOWORD((if fIcon != 0 { RT_ICON } else { RT_CURSOR }) as usize as DWORD);

            /* Can we load this BMP icon? */
            if CURSORICON_GetCursorDataFromBMI(
                &mut cursorData,
                pbBmpIcon.add((*entry).dwDIBOffset as usize) as *const BITMAPINFO,
            ) == 0
            {
                ERR!("Couldn't get cursor/icon data\n");
                return create_icon_from_resource_cleanup(cursorData, isAnimated, pbBmpIcon);
            }
        }
    }

    if uFlags & LR_SHARED != 0 {
        cursorData.CURSORF_flags |= CURSORF_LRSHARED;
    }

    hIcon = NtUserxCreateEmptyCurObject(isAnimated as DWORD_PTR);
    if hIcon.is_null() {
        return create_icon_from_resource_cleanup(cursorData, isAnimated, pbBmpIcon);
    }

    if NtUserSetCursorIconData(hIcon, core::ptr::null_mut(), core::ptr::null_mut(), &mut cursorData) == 0 {
        ERR!("NtUserSetCursorIconData failed.\n");
        NtUserDestroyCursor(hIcon as HANDLE, 1);
        return create_icon_from_resource_cleanup(cursorData, isAnimated, pbBmpIcon);
    }

    if isAnimated != 0 {
        HeapFree(GetProcessHeap(), 0, cursorData.aspcur as LPVOID);
    }

    HeapFree(GetProcessHeap(), 0, pbBmpIcon as LPVOID);
    hIcon
}

/// `end_error:` cleanup tail of CreateIconFromResourceEx — extracted so the early
/// returns above can run it (faithful to the C `goto end_error`).
#[inline]
unsafe fn create_icon_from_resource_cleanup(
    cursorData: CURSORDATA,
    isAnimated: BOOL,
    pbBmpIcon: PBYTE,
) -> HICON {
    HeapFree(GetProcessHeap(), 0, pbBmpIcon as LPVOID);
    if isAnimated != 0 {
        HeapFree(GetProcessHeap(), 0, cursorData.aspcur as LPVOID);
    }
    if !cursorData.hbmMask.is_null() {
        DeleteObject(cursorData.hbmMask as HGDIOBJ);
    }
    if !cursorData.hbmColor.is_null() {
        DeleteObject(cursorData.hbmColor as HGDIOBJ);
    }
    if !cursorData.hbmAlpha.is_null() {
        DeleteObject(cursorData.hbmAlpha as HGDIOBJ);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateIconIndirect(piconinfo: PICONINFO) -> HICON {

    let mut cursorData: CURSORDATA = core::mem::zeroed();
    let mut hiconRet: HICON;

    core::ptr::write_bytes(&mut cursorData as *mut CURSORDATA, 0, 1);

    'end_error: {
        if CURSORICON_GetCursorDataFromIconInfo(&mut cursorData, piconinfo) == 0 {
            return core::ptr::null_mut();
        }

        hiconRet = NtUserxCreateEmptyCurObject(0);
        if hiconRet.is_null() {
            break 'end_error;
        }

        if NtUserSetCursorIconData(
            hiconRet,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut cursorData,
        ) == 0
        {
            NtUserDestroyCursor(hiconRet as HANDLE, 0);
            break 'end_error;
        }

        return hiconRet;
    }

    DeleteObject(cursorData.hbmMask as HGLOBAL);
    if !cursorData.hbmColor.is_null() {
        DeleteObject(cursorData.hbmColor as HGLOBAL);
    }
    if !cursorData.hbmAlpha.is_null() {
        DeleteObject(cursorData.hbmAlpha as HGLOBAL);
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateCursor(
    hInst: HINSTANCE,
    xHotSpot: core::ffi::c_int,
    yHotSpot: core::ffi::c_int,
    nWidth: core::ffi::c_int,
    nHeight: core::ffi::c_int,
    pvANDPlane: *const core::ffi::c_void,
    pvXORPlane: *const core::ffi::c_void,
) -> HCURSOR {
    let mut info: ICONINFO = core::mem::zeroed();
    let hCursor: HCURSOR;

    info.fIcon = 0u8 as _;
    info.xHotspot = xHotSpot as _;
    info.yHotspot = yHotSpot as _;
    info.hbmMask = CreateBitmap(nWidth, nHeight, 1, 1, pvANDPlane);
    info.hbmColor = CreateBitmap(nWidth, nHeight, 1, 1, pvXORPlane);
    hCursor = CreateIconIndirect(&mut info);
    DeleteObject(info.hbmMask as _);
    DeleteObject(info.hbmColor as _);
    hCursor
}

#[no_mangle]
pub unsafe extern "system" fn SetSystemCursor(mut hcur: HCURSOR, id: DWORD) -> BOOL {
    if hcur.is_null() {
        hcur = LoadImageW(
            core::ptr::null_mut(),
            MAKEINTRESOURCEW(id),
            IMAGE_CURSOR,
            0,
            0,
            LR_DEFAULTSIZE,
        ) as HCURSOR;
        if hcur.is_null() {
            return 0;
        }
    }
    NtUserSetSystemCursor(hcur, id)
}

#[no_mangle]
pub unsafe extern "system" fn SetCursorPos(X: core::ffi::c_int, Y: core::ffi::c_int) -> BOOL {
    NtUserxSetCursorPos(X, Y)
}

#[no_mangle]
pub unsafe extern "system" fn GetCursorPos(lpPoint: LPPOINT) -> BOOL {
    NtUserxGetCursorPos(lpPoint)
}

#[no_mangle]
pub unsafe extern "system" fn ShowCursor(bShow: BOOL) -> core::ffi::c_int {
    NtUserxShowCursor(bShow)
}

#[no_mangle]
pub unsafe extern "system" fn GetCursor() -> HCURSOR {
    NtUserGetThreadState(THREADSTATE_GETCURSOR as _) as HCURSOR
}

#[no_mangle]
pub unsafe extern "system" fn DestroyCursor(hCursor: HCURSOR) -> BOOL {
    NtUserDestroyCursor(hCursor as HANDLE, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GetCursorFrameInfo(
    hCursor: HCURSOR,
    reserved: DWORD,
    istep: DWORD,
    rate_jiffies: PINT,
    num_steps: *mut DWORD,
) -> HCURSOR {
    NtUserGetCursorFrameInfo(hCursor, istep, rate_jiffies, num_steps)
}
