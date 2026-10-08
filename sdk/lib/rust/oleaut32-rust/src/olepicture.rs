use crate::prelude::*;

extern "system" {
    fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
    fn GetEnhMetaFileHeader( hemf: HENHMETAFILE, nSize: u32, lpEnhMetaHeader: *mut ENHMETAHEADER, ) -> u32;
    fn SetBkColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetMapMode(hdc: HDC, iMode: i32) -> i32;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn SetViewportExtEx(hdc: HDC, x: i32, y: i32, lpsz: *mut SIZE) -> i32;
    fn SetViewportOrgEx(hdc: HDC, x: i32, y: i32, lppt: *mut POINT) -> i32;
    fn SetWindowExtEx(hdc: HDC, x: i32, y: i32, lpsz: *mut SIZE) -> i32;
    fn SetWindowOrgEx(hdc: HDC, x: i32, y: i32, lppt: *mut POINT) -> i32;
    fn StretchBlt( hdcDest: HDC, xDest: i32, yDest: i32, wDest: i32, hDest: i32, hdcSrc: HDC, xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32, rop: u32, ) -> i32;
    fn SetEnhMetaFileBits(nSize: UINT, pb: *const BYTE) -> HENHMETAFILE;
    fn SetMetaFileBitsEx(cbBuffer: u32, lpData: *const BYTE) -> HMETAFILE;
    fn GetDIBits( hdc: HDC, hbm: HBITMAP, start: u32, cLines: u32, lpvBits: LPVOID, lpbmi: *mut BITMAPINFO, usage: u32, ) -> i32;
    fn GetIconInfo(hIcon: HICON, piconinfo: *mut ICONINFO) -> i32;
    fn GetEnhMetaFileBits( hemf: HENHMETAFILE, nSize: ::core::ffi::c_uint, lpData: LPBYTE, ) -> ::core::ffi::c_uint;
    fn CoInternetParseUrl( pwzUrl: LPCWSTR, ParseAction: u32, dwFlags: DWORD, pszResult: LPWSTR, cchResult: DWORD, pcchResult: *mut DWORD, dwReserved: DWORD, ) -> HRESULT;
    fn CreateBindCtx(reserved: DWORD, ppbc: *mut *mut IBindCtx) -> HRESULT;
    fn CreateURLMoniker( pMkCtx: *mut IMoniker, szURL: LPCWSTR, ppmk: *mut *mut IMoniker, ) -> HRESULT;
}
extern "C" {
    fn printf(format: *const i8, ...) -> i32;
    fn wcsncmp(s1: *const wchar_t, s2: *const wchar_t, n: usize) -> ::core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut GUID_WICPixelFormatDontCare: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat1bppIndexed: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat2bppIndexed: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat4bppIndexed: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppIndexed: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormatBlackWhite: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat2bppGray: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat4bppGray: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppGray: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppGray: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppAlpha: GUID = _GUID {
    Data1: 0xe6cd0116 as ::core::ffi::c_uint,
    Data2: 0xeeba as ::core::ffi::c_ushort,
    Data3: 0x4161 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppBGR555: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppBGR565: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppBGRA5551: GUID = _GUID {
    Data1: 0x5ec7c2b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf1e6 as ::core::ffi::c_ushort,
    Data3: 0x4961 as ::core::ffi::c_ushort,
    Data4: [
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat24bppBGR: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat24bppRGB: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppBGR: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppBGRA: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppPBGRA: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppRGB: GUID = _GUID {
    Data1: 0xd98c6b95 as ::core::ffi::c_uint,
    Data2: 0x3efe as ::core::ffi::c_ushort,
    Data3: 0x47d6 as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppRGBA: GUID = _GUID {
    Data1: 0xf5c7ad2d as ::core::ffi::c_uint,
    Data2: 0x6a8d as ::core::ffi::c_ushort,
    Data3: 0x43dd as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppPRGBA: GUID = _GUID {
    Data1: 0x3cc4a650 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xa527 as ::core::ffi::c_ushort,
    Data3: 0x4d37 as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppGrayFloat: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bppRGB: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bppBGR: GUID = _GUID {
    Data1: 0xe605a384 as ::core::ffi::c_uint,
    Data2: 0xb468 as ::core::ffi::c_ushort,
    Data3: 0x46ce as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGB: GUID = _GUID {
    Data1: 0xa1182111 as ::core::ffi::c_uint,
    Data2: 0x186d as ::core::ffi::c_ushort,
    Data3: 0x4d42 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGBA: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppBGRA: GUID = _GUID {
    Data1: 0x1562ff7c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd352 as ::core::ffi::c_ushort,
    Data3: 0x46f9 as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x46 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppPRGBA: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppPBGRA: GUID = _GUID {
    Data1: 0x8c518e8e as ::core::ffi::c_uint,
    Data2: 0xa4ec as ::core::ffi::c_ushort,
    Data3: 0x468b as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppGrayFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppBGR101010: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bppRGBFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bppBGRFixedPoint: GUID = _GUID {
    Data1: 0x49ca140e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xcab6 as ::core::ffi::c_ushort,
    Data3: 0x493b as ::core::ffi::c_ushort,
    Data4: [
        0x9d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat96bppRGBFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat96bppRGBFloat: GUID = _GUID {
    Data1: 0xe3fed78f as ::core::ffi::c_uint,
    Data2: 0xe8db as ::core::ffi::c_ushort,
    Data3: 0x4acf as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bppRGBAFloat: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bppPRGBAFloat: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bppRGBFloat: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppCMYK: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGBAFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppBGRAFixedPoint: GUID = _GUID {
    Data1: 0x356de33c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x54d2 as ::core::ffi::c_ushort,
    Data3: 0x4a23 as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGBFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bppRGBAFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bppRGBFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGBAHalf: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppPRGBAHalf: GUID = _GUID {
    Data1: 0x58ad26c2 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc623 as ::core::ffi::c_ushort,
    Data3: 0x4d9d as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppRGBHalf: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bppRGBHalf: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppRGBE: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppGrayHalf: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppGrayFixedPoint: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppRGBA1010102: GUID = _GUID {
    Data1: 0x25238d72 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfcf9 as ::core::ffi::c_ushort,
    Data3: 0x4522 as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppRGBA1010102XR: GUID = _GUID {
    Data1: 0xde6b9a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xc101 as ::core::ffi::c_ushort,
    Data3: 0x434b as ::core::ffi::c_ushort,
    Data4: [
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppR10G10B10A2: GUID = _GUID {
    Data1: 0x604e1bb5 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8a3c as ::core::ffi::c_ushort,
    Data3: 0x4b65 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bppR10G10B10A2HDR10: GUID = _GUID {
    Data1: 0x9c215c5d as ::core::ffi::c_uint,
    Data2: 0x1acc as ::core::ffi::c_ushort,
    Data3: 0x4f0e as ::core::ffi::c_ushort,
    Data4: [
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bppCMYK: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat24bpp3Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bpp4Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat40bpp5Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bpp6Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat56bpp7Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bpp8Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bpp3Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bpp4Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat80bpp5Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat96bpp6Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat112bpp7Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bpp8Channels: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat40bppCMYKAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat80bppCMYKAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat32bpp3ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat40bpp4ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat48bpp5ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat56bpp6ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bpp7ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat72bpp8ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat64bpp3ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat80bpp4ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat96bpp5ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat112bpp6ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat128bpp7ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat144bpp8ChannelsAlpha: GUID = _GUID {
    Data1: 0x6fddc324 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4e03 as ::core::ffi::c_ushort,
    Data3: 0x4bfe as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppY: GUID = _GUID {
    Data1: 0x91b4db54 as ::core::ffi::c_uint,
    Data2: 0x2df9 as ::core::ffi::c_ushort,
    Data3: 0x42f0 as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x49 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppCb: GUID = _GUID {
    Data1: 0x1339f224 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6bfe as ::core::ffi::c_ushort,
    Data3: 0x4c3e as ::core::ffi::c_ushort,
    Data4: [
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat8bppCr: GUID = _GUID {
    Data1: 0xb8145053 as ::core::ffi::c_uint,
    Data2: 0x2116 as ::core::ffi::c_ushort,
    Data3: 0x49f0 as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppCbCr: GUID = _GUID {
    Data1: 0xff95ba6e as ::core::ffi::c_uint,
    Data2: 0x11e0 as ::core::ffi::c_ushort,
    Data3: 0x4263 as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppYQuantizedDctCoefficients: GUID = _GUID {
    Data1: 0xa355f433 as ::core::ffi::c_uint,
    Data2: 0x48e8 as ::core::ffi::c_ushort,
    Data3: 0x4a42 as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppCbQuantizedDctCoefficients: GUID = _GUID {
    Data1: 0xd2c4ff61 as ::core::ffi::c_uint,
    Data2: 0x56a5 as ::core::ffi::c_ushort,
    Data3: 0x49c2 as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_WICPixelFormat16bppCrQuantizedDctCoefficients: GUID = _GUID {
    Data1: 0x2fe354f0 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1680 as ::core::ffi::c_ushort,
    Data3: 0x42d8 as ::core::ffi::c_ushort,
    Data4: [
        0x92 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICColorContext: GUID = _GUID {
    Data1: 0x3c613a02 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x34b2 as ::core::ffi::c_ushort,
    Data3: 0x44ea as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapSource: GUID = _GUID {
    Data1: 0x120 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapLock: GUID = _GUID {
    Data1: 0x123 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapFlipRotator: GUID = _GUID {
    Data1: 0x5009834f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x2d6a as ::core::ffi::c_ushort,
    Data3: 0x41ce as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmap: GUID = _GUID {
    Data1: 0x121 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICPalette: GUID = _GUID {
    Data1: 0x40 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICComponentInfo: GUID = _GUID {
    Data1: 0x23bc3f0a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x698b as ::core::ffi::c_ushort,
    Data3: 0x4357 as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICMetadataQueryReader: GUID = _GUID {
    Data1: 0x30989668 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe1c9 as ::core::ffi::c_ushort,
    Data3: 0x4597 as ::core::ffi::c_ushort,
    Data4: [
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICMetadataQueryWriter: GUID = _GUID {
    Data1: 0xa721791a as ::core::ffi::c_uint,
    Data2: 0xdef as ::core::ffi::c_ushort,
    Data3: 0x4d06 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapFrameDecode: GUID = _GUID {
    Data1: 0x3b16811b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6a43 as ::core::ffi::c_ushort,
    Data3: 0x4ec9 as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICPixelFormatInfo: GUID = _GUID {
    Data1: 0xe8eda601 as ::core::ffi::c_uint,
    Data2: 0x3d48 as ::core::ffi::c_ushort,
    Data3: 0x431a as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x69 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICPixelFormatInfo2: GUID = _GUID {
    Data1: 0xa9db33a2 as ::core::ffi::c_uint,
    Data2: 0xaf5f as ::core::ffi::c_ushort,
    Data3: 0x43c7 as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapCodecInfo: GUID = _GUID {
    Data1: 0xe87a44c4 as ::core::ffi::c_uint,
    Data2: 0xb76e as ::core::ffi::c_ushort,
    Data3: 0x4c47 as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x14 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapDecoderInfo: GUID = _GUID {
    Data1: 0xd8cd007f as ::core::ffi::c_uint,
    Data2: 0xd08f as ::core::ffi::c_ushort,
    Data3: 0x4191 as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x23 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapDecoder: GUID = _GUID {
    Data1: 0x9edde9e7 as ::core::ffi::c_uint,
    Data2: 0x8dee as ::core::ffi::c_ushort,
    Data3: 0x47ea as ::core::ffi::c_ushort,
    Data4: [
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapFrameEncode: GUID = _GUID {
    Data1: 0x105 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapEncoderInfo: GUID = _GUID {
    Data1: 0x94c9b4ee as ::core::ffi::c_uint,
    Data2: 0xa09f as ::core::ffi::c_ushort,
    Data3: 0x4f92 as ::core::ffi::c_ushort,
    Data4: [
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapEncoder: GUID = _GUID {
    Data1: 0x103 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICFormatConverter: GUID = _GUID {
    Data1: 0x301 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICFormatConverterInfo: GUID = _GUID {
    Data1: 0x9f34fb65 as ::core::ffi::c_uint,
    Data2: 0x13f4 as ::core::ffi::c_ushort,
    Data3: 0x4f15 as ::core::ffi::c_ushort,
    Data4: [
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x26 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICStream: GUID = _GUID {
    Data1: 0x135ff860 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x22b7 as ::core::ffi::c_ushort,
    Data3: 0x4ddf as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapScaler: GUID = _GUID {
    Data1: 0x302 as ::core::ffi::c_uint,
    Data2: 0xa8f2 as ::core::ffi::c_ushort,
    Data3: 0x4877 as ::core::ffi::c_ushort,
    Data4: [
        0xba as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICBitmapClipper: GUID = _GUID {
    Data1: 0xe4fbcf03 as ::core::ffi::c_uint,
    Data2: 0x223d as ::core::ffi::c_ushort,
    Data3: 0x4e81 as ::core::ffi::c_ushort,
    Data4: [
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICColorTransform: GUID = _GUID {
    Data1: 0xb66f034f as ::core::ffi::c_uint,
    Data2: 0xd0e2 as ::core::ffi::c_ushort,
    Data3: 0x40ab as ::core::ffi::c_ushort,
    Data4: [
        0xb4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICFastMetadataEncoder: GUID = _GUID {
    Data1: 0xb84e2c09 as ::core::ffi::c_uint,
    Data2: 0x78c9 as ::core::ffi::c_ushort,
    Data3: 0x4ac4 as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x66 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICImagingFactory: GUID = _GUID {
    Data1: 0xcacaf262 as ::core::ffi::c_uint,
    Data2: 0x9370 as ::core::ffi::c_ushort,
    Data3: 0x4615 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICImagingFactory1: GUID = _GUID {
    Data1: 0xcacaf262 as ::core::ffi::c_uint,
    Data2: 0x9370 as ::core::ffi::c_ushort,
    Data3: 0x4615 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x55 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICImagingFactory2: GUID = _GUID {
    Data1: 0x317d06e8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5f24 as ::core::ffi::c_ushort,
    Data3: 0x433d as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICImagingFactory: GUID = _GUID {
    Data1: 0xec5ec8a9 as ::core::ffi::c_uint,
    Data2: 0xc395 as ::core::ffi::c_ushort,
    Data3: 0x4314 as ::core::ffi::c_ushort,
    Data4: [
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICImagingFactory2: GUID = _GUID {
    Data1: 0x7b816b45 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x1996 as ::core::ffi::c_ushort,
    Data3: 0x4476 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xde as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICEnumMetadataItem: GUID = _GUID {
    Data1: 0xdc2bb46d as ::core::ffi::c_uint,
    Data2: 0x3f07 as ::core::ffi::c_ushort,
    Data3: 0x481e as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICDdsDecoder: GUID = _GUID {
    Data1: 0x409cd537 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8532 as ::core::ffi::c_ushort,
    Data3: 0x40cb as ::core::ffi::c_ushort,
    Data4: [
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICDdsEncoder: GUID = _GUID {
    Data1: 0x5cacdb4c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x407e as ::core::ffi::c_ushort,
    Data3: 0x41b3 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x32 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICDdsFrameDecode: GUID = _GUID {
    Data1: 0x3d4c0c61 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x18a4 as ::core::ffi::c_ushort,
    Data3: 0x41e4 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x80 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IWICWineDecoder: GUID = _GUID {
    Data1: 0xb9bd430d as ::core::ffi::c_uint,
    Data2: 0x28a8 as ::core::ffi::c_ushort,
    Data3: 0x41d3 as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICBmpDecoder: GUID = _GUID {
    Data1: 0x6b462062 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7cbf as ::core::ffi::c_ushort,
    Data3: 0x400d as ::core::ffi::c_ushort,
    Data4: [
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x27 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICPngDecoder: GUID = _GUID {
    Data1: 0x389ea17b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5078 as ::core::ffi::c_ushort,
    Data3: 0x4cde as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICPngDecoder1: GUID = _GUID {
    Data1: 0x389ea17b as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5078 as ::core::ffi::c_ushort,
    Data3: 0x4cde as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICPngDecoder2: GUID = _GUID {
    Data1: 0xe018945b as ::core::ffi::c_uint,
    Data2: 0xaa86 as ::core::ffi::c_ushort,
    Data3: 0x4008 as ::core::ffi::c_ushort,
    Data4: [
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICIcoDecoder: GUID = _GUID {
    Data1: 0xc61bfcdf as ::core::ffi::c_uint,
    Data2: 0x2e0f as ::core::ffi::c_ushort,
    Data3: 0x4aad as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICJpegDecoder: GUID = _GUID {
    Data1: 0x9456a480 as ::core::ffi::c_uint,
    Data2: 0xe88b as ::core::ffi::c_ushort,
    Data3: 0x43ea as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICGifDecoder: GUID = _GUID {
    Data1: 0x381dda3c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x9ce9 as ::core::ffi::c_ushort,
    Data3: 0x4834 as ::core::ffi::c_ushort,
    Data4: [
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICTiffDecoder: GUID = _GUID {
    Data1: 0xb54e85d9 as ::core::ffi::c_uint,
    Data2: 0xfe23 as ::core::ffi::c_ushort,
    Data3: 0x499f as ::core::ffi::c_ushort,
    Data4: [
        0x8b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x13 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICWmpDecoder: GUID = _GUID {
    Data1: 0xa26cec36 as ::core::ffi::c_uint,
    Data2: 0x234c as ::core::ffi::c_ushort,
    Data3: 0x4950 as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICDdsDecoder: GUID = _GUID {
    Data1: 0x9053699f as ::core::ffi::c_uint,
    Data2: 0xa341 as ::core::ffi::c_ushort,
    Data3: 0x429d as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICBmpEncoder: GUID = _GUID {
    Data1: 0x69be8bb4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xd66d as ::core::ffi::c_ushort,
    Data3: 0x47c8 as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x43 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICPngEncoder: GUID = _GUID {
    Data1: 0x27949969 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x876a as ::core::ffi::c_ushort,
    Data3: 0x41d7 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x35 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICJpegEncoder: GUID = _GUID {
    Data1: 0x1a34f5c1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x4a5a as ::core::ffi::c_ushort,
    Data3: 0x46dc as ::core::ffi::c_ushort,
    Data4: [
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x44 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x67 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICGifEncoder: GUID = _GUID {
    Data1: 0x114f5598 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb22 as ::core::ffi::c_ushort,
    Data3: 0x40a0 as ::core::ffi::c_ushort,
    Data4: [
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x95 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xad as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICTiffEncoder: GUID = _GUID {
    Data1: 0x131be10 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x2001 as ::core::ffi::c_ushort,
    Data3: 0x4c5f as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICWmpEncoder: GUID = _GUID {
    Data1: 0xac4ce3cb as ::core::ffi::c_uint,
    Data2: 0xe1c1 as ::core::ffi::c_ushort,
    Data3: 0x44cd as ::core::ffi::c_ushort,
    Data4: [
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x16 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x65 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICDdsEncoder: GUID = _GUID {
    Data1: 0xa61dde94 as ::core::ffi::c_uint,
    Data2: 0x66ce as ::core::ffi::c_ushort,
    Data3: 0x4ac1 as ::core::ffi::c_ushort,
    Data4: [
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICAdngDecoder: GUID = _GUID {
    Data1: 0x981d9411 as ::core::ffi::c_uint,
    Data2: 0x909e as ::core::ffi::c_ushort,
    Data3: 0x42a7 as ::core::ffi::c_ushort,
    Data4: [
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x47 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xff as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICJpegQualcommPhoneEncoder: GUID = _GUID {
    Data1: 0x68ed5c62 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xf534 as ::core::ffi::c_ushort,
    Data3: 0x4979 as ::core::ffi::c_ushort,
    Data4: [
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x68 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4c as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICHeifDecoder: GUID = _GUID {
    Data1: 0xe9a4a80a as ::core::ffi::c_uint,
    Data2: 0x44fe as ::core::ffi::c_ushort,
    Data3: 0x4de4 as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICHeifEncoder: GUID = _GUID {
    Data1: 0xdbecec1 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x9eb3 as ::core::ffi::c_ushort,
    Data3: 0x4860 as ::core::ffi::c_ushort,
    Data4: [
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICWebpDecoder: GUID = _GUID {
    Data1: 0x7693e886 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x51c9 as ::core::ffi::c_ushort,
    Data3: 0x4070 as ::core::ffi::c_ushort,
    Data4: [
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x73 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfa as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICRAWDecoder: GUID = _GUID {
    Data1: 0x41945702 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x8302 as ::core::ffi::c_ushort,
    Data3: 0x44a6 as ::core::ffi::c_ushort,
    Data4: [
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x45 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICDefaultFormatConverter: GUID = _GUID {
    Data1: 0x1a3f11dc as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xb514 as ::core::ffi::c_ushort,
    Data3: 0x4b17 as ::core::ffi::c_ushort,
    Data4: [
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x54 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x51 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x52 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICFormatConverterHighColor: GUID = _GUID {
    Data1: 0xac75d454 as ::core::ffi::c_uint,
    Data2: 0x9f37 as ::core::ffi::c_ushort,
    Data3: 0x48f8 as ::core::ffi::c_ushort,
    Data4: [
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x19 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x85 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x60 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICFormatConverterNChannel: GUID = _GUID {
    Data1: 0xc17cabb2 as ::core::ffi::c_uint,
    Data2: 0xd4a3 as ::core::ffi::c_ushort,
    Data3: 0x47d7 as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICFormatConverterWMPhoto: GUID = _GUID {
    Data1: 0x9cb5172b as ::core::ffi::c_uint,
    Data2: 0xd600 as ::core::ffi::c_ushort,
    Data3: 0x46ba as ::core::ffi::c_ushort,
    Data4: [
        0xab as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICPlanarFormatConverter: GUID = _GUID {
    Data1: 0x184132b8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x32f8 as ::core::ffi::c_ushort,
    Data3: 0x4784 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x34 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatBmp: GUID = _GUID {
    Data1: 0xaf1d87e as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xfcfe as ::core::ffi::c_ushort,
    Data3: 0x4188 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xeb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x71 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatPng: GUID = _GUID {
    Data1: 0x1b7cfaf4 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x713f as ::core::ffi::c_ushort,
    Data3: 0x473c as ::core::ffi::c_ushort,
    Data4: [
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x61 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatIco: GUID = _GUID {
    Data1: 0xa3a860c4 as ::core::ffi::c_uint,
    Data2: 0x338f as ::core::ffi::c_ushort,
    Data3: 0x4c17 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatJpeg: GUID = _GUID {
    Data1: 0x19e4a5aa as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x5662 as ::core::ffi::c_ushort,
    Data3: 0x4fc5 as ::core::ffi::c_ushort,
    Data4: [
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x17 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x58 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatTiff: GUID = _GUID {
    Data1: 0x163bcc30 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe2e9 as ::core::ffi::c_ushort,
    Data3: 0x4f0b as ::core::ffi::c_ushort,
    Data4: [
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x88 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatGif: GUID = _GUID {
    Data1: 0x1f8a5601 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7d4d as ::core::ffi::c_ushort,
    Data3: 0x4cbd as ::core::ffi::c_ushort,
    Data4: [
        0x9c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x82 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatWmp: GUID = _GUID {
    Data1: 0x57a37caa as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x367a as ::core::ffi::c_ushort,
    Data3: 0x4540 as ::core::ffi::c_ushort,
    Data4: [
        0x91 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatDds: GUID = _GUID {
    Data1: 0x9967cb95 as ::core::ffi::c_uint,
    Data2: 0x2e85 as ::core::ffi::c_ushort,
    Data3: 0x4ac8 as ::core::ffi::c_ushort,
    Data4: [
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x83 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xcc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x25 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatAdng: GUID = _GUID {
    Data1: 0xf3ff6d0d as ::core::ffi::c_uint,
    Data2: 0x38c0 as ::core::ffi::c_ushort,
    Data3: 0x41c4 as ::core::ffi::c_ushort,
    Data4: [
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xfe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x84 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatHeif: GUID = _GUID {
    Data1: 0xe1e62521 as ::core::ffi::c_uint,
    Data2: 0x6787 as ::core::ffi::c_ushort,
    Data3: 0x405b as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatWebp: GUID = _GUID {
    Data1: 0xe094b0e2 as ::core::ffi::c_uint,
    Data2: 0x67f2 as ::core::ffi::c_ushort,
    Data3: 0x45b3 as ::core::ffi::c_ushort,
    Data4: [
        0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xea as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x37 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_ContainerFormatRaw: GUID = _GUID {
    Data1: 0xfe99ce60 as ::core::ffi::c_uint,
    Data2: 0xf19c as ::core::ffi::c_ushort,
    Data3: 0x433c as ::core::ffi::c_ushort,
    Data4: [
        0xa3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xac as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xef as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x21 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VendorMicrosoft: GUID = _GUID {
    Data1: 0xf0e749ca as ::core::ffi::c_uint,
    Data2: 0xedef as ::core::ffi::c_ushort,
    Data3: 0x4589 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xee as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x6a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut GUID_VendorMicrosoftBuiltIn: GUID = _GUID {
    Data1: 0x257a30fd as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x6b6 as ::core::ffi::c_ushort,
    Data3: 0x462b as ::core::ffi::c_ushort,
    Data4: [
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x63 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x86 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x33 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CLSID_WICImagingCategories: GUID = _GUID {
    Data1: 0xfae3d380 as ::core::ffi::c_uint,
    Data2: 0xfea4 as ::core::ffi::c_ushort,
    Data3: 0x4623 as ::core::ffi::c_ushort,
    Data4: [
        0x8c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x75 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x10 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x81 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICBitmapDecoders: GUID = _GUID {
    Data1: 0x7ed96837 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x96f0 as ::core::ffi::c_ushort,
    Data3: 0x4812 as ::core::ffi::c_ushort,
    Data4: [
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3c as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x11 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICBitmapEncoders: GUID = _GUID {
    Data1: 0xac757296 as ::core::ffi::c_uint,
    Data2: 0x3522 as ::core::ffi::c_ushort,
    Data3: 0x4e11 as ::core::ffi::c_ushort,
    Data4: [
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x62 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICFormatConverters: GUID = _GUID {
    Data1: 0x7835eae8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xbf14 as ::core::ffi::c_ushort,
    Data3: 0x49d1 as ::core::ffi::c_ushort,
    Data4: [
        0x93 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xce as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x40 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x48 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICMetadataReader: GUID = _GUID {
    Data1: 0x5af94d8 as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x7174 as ::core::ffi::c_ushort,
    Data3: 0x4cd2 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x41 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x24 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICMetadataWriter: GUID = _GUID {
    Data1: 0xabe3b9a4 as ::core::ffi::c_uint,
    Data2: 0x257d as ::core::ffi::c_ushort,
    Data3: 0x4b97 as ::core::ffi::c_ushort,
    Data4: [
        0xbd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x29 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2e as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut CATID_WICPixelFormats: GUID = _GUID {
    Data1: 0x2b46e70f as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xcda7 as ::core::ffi::c_ushort,
    Data3: 0x473e as ::core::ffi::c_ushort,
    Data4: [
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdc as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x30 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x39 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut __wine_dbch_olepicture: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"olepicture\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_olepicture as *mut __wine_debug_channel };

static mut STDPIC_CF: IClassFactoryImpl = unsafe {
    IClassFactoryImpl {
        IClassFactory_iface: IClassFactory {
            lpVtbl: &raw const SPCF_Vtbl as *mut IClassFactoryVtbl,
        },
        ref_: 1 as LONG,
    }
};

pub const BITMAP_FORMAT_BMP: u16 = 0x4d42;
pub const BITMAP_FORMAT_JPEG: u16 = 0xd8ff;
pub const BITMAP_FORMAT_GIF: u16 = 0x4947;
pub const BITMAP_FORMAT_PNG: u16 = 0x5089;
pub const BITMAP_FORMAT_APM: u16 = 0xcdd7;

// C: these three sit inside #include "pshpack1.h" (byte-packed, they mirror
// FILE formats). Translated as plain repr(C), APM_HEADER was 24 bytes instead
// of 22, so OleLoadPicture read a placeable metafile from the wrong offset and
// failed (Wine's olepicture test_apm), and CURSORICONFILEDIR's entries landed
// at offset 8 instead of 6, two bytes off for every icon and cursor.
const _: () = assert!(core::mem::size_of::<_APM_HEADER>() == 22);
const _: () = assert!(core::mem::size_of::<CURSORICONFILEDIRENTRY>() == 16);
const _: () = assert!(core::mem::offset_of!(CURSORICONFILEDIR, idEntries) == 6);
#[repr(C, packed)]
pub struct _APM_HEADER {
    pub key: DWORD,
    pub handle: WORD,
    pub left: SHORT,
    pub top: SHORT,
    pub right: SHORT,
    pub bottom: SHORT,
    pub inch: WORD,
    pub reserved: DWORD,
    pub checksum: WORD,
}
pub type APM_HEADER = _APM_HEADER;

#[repr(C, packed)]
pub struct CURSORICONFILEDIRENTRY {
    pub bWidth: BYTE,
    pub bHeight: BYTE,
    pub bColorCount: BYTE,
    pub bReserved: BYTE,
    pub xHotspot: WORD,
    pub yHotspot: WORD,
    pub dwDIBSize: DWORD,
    pub dwDIBOffset: DWORD,
}

#[repr(C, packed)]
pub struct CURSORICONFILEDIR {
    pub idReserved: WORD,
    pub idType: WORD,
    pub idCount: WORD,
    pub idEntries: [CURSORICONFILEDIRENTRY; 1],
}

#[repr(C)]
pub struct OLEPictureImpl {
    pub IPicture_iface: IPicture,
    pub IDispatch_iface: IDispatch,
    pub IPersistStream_iface: IPersistStream,
    pub IConnectionPointContainer_iface: IConnectionPointContainer,
    pub ref_: LONG,
    pub fOwn: BOOL,
    pub desc: PICTDESC,
    pub origWidth: DWORD,
    pub origHeight: DWORD,
    pub himetricWidth: OLE_XSIZE_HIMETRIC,
    pub himetricHeight: OLE_YSIZE_HIMETRIC,
    pub pCP: *mut IConnectionPoint,
    pub keepOrigFormat: BOOL,
    pub hDCCur: HDC,
    pub stock_bitmap: HBITMAP,
    pub hbmMask: HBITMAP,
    pub hbmXor: HBITMAP,
    pub rgbTrans: COLORREF,
    pub data: *mut core::ffi::c_void,
    pub datalen: core::ffi::c_int,
    pub bIsDirty: BOOL,
    pub loadtime_magic: core::ffi::c_uint,
    pub loadtime_format: core::ffi::c_uint,
}

#[repr(C)]
pub struct IClassFactoryImpl {
    pub IClassFactory_iface: IClassFactory,
    pub ref_: LONG,
}

fn HRESULT_FROM_WIN32(x: u32) -> HRESULT {
    if x as i32 > 0 {
        ((((x & 0xFFFF) | ((7 as u32) << 16)) | 0x80000000) as u32) as HRESULT
    } else {
        x as i32
    }
}

// urlmon.idl PARSEACTION: PARSE_CANONICALIZE = 1 ... PARSE_PATH_FROM_URL is the ninth.
// It was 2 (PARSE_FRIENDLY), which urlmon does not implement: every `file:` path
// handed to OleLoadPicturePath came back E_NOTIMPL.
const PARSE_PATH_FROM_URL: u32 = 9;
const E_INVALIDARG: HRESULT = 0x80070057u32 as i32;
const INET_E_RESOURCE_NOT_FOUND: HRESULT = 0x800C0005u32 as i32;
unsafe extern "C" fn impl_from_IPicture(iface: *mut IPicture) -> *mut OLEPictureImpl {
    let offset = core::mem::offset_of!(OLEPictureImpl, IPicture_iface);
    (iface as *mut u8).sub(offset) as *mut OLEPictureImpl
}
unsafe extern "C" fn impl_from_IDispatch(iface: *mut IDispatch) -> *mut OLEPictureImpl {
    (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(OLEPictureImpl, IDispatch_iface) as ULONG_PTR
    ) as *mut OLEPictureImpl
}
unsafe extern "C" fn impl_from_IPersistStream(iface: *mut IPersistStream) -> *mut OLEPictureImpl {
    (iface as usize).wrapping_sub(
        core::mem::offset_of!(OLEPictureImpl, IPersistStream_iface)
    ) as *mut OLEPictureImpl
}
unsafe extern "C" fn impl_from_IConnectionPointContainer(
    iface: *mut IConnectionPointContainer,
) -> *mut OLEPictureImpl {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(OLEPictureImpl, IConnectionPointContainer_iface) as ULONG_PTR,
    ) as *mut OLEPictureImpl;
}
unsafe extern "C" fn xpixels_to_himetric(pixels: INT, hdc: HDC) -> OLE_XSIZE_HIMETRIC {
    const LOGPIXELSX: i32 = 88;
    
    MulDiv(pixels, 2540, GetDeviceCaps(hdc, LOGPIXELSX)) as OLE_XSIZE_HIMETRIC
}
unsafe extern "C" fn ypixels_to_himetric(pixels: i32, hdc: HDC) -> i32 {
    const LOGPIXELSY: i32 = 90;
    
    MulDiv(pixels, 2540, GetDeviceCaps(hdc, LOGPIXELSY))
}
unsafe extern "C" fn OLEPictureImpl_SetBitmap(This: *mut OLEPictureImpl) {
    let mut bm: BITMAP = core::mem::zeroed();
    let hdcRef: HDC;

    TRACE!(b"bitmap handle %p\n\0".as_ptr() as *const i8, (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap);
    
    if GetObjectW(
        (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as HANDLE,
        core::mem::size_of::<BITMAP>() as i32,
        &mut bm as *mut BITMAP as LPVOID,
    ) as usize != core::mem::size_of::<BITMAP>()
    {
        ERR!(b"GetObject fails\n\0".as_ptr() as *const i8);
        return;
    }
    
    (*This).origWidth = bm.bmWidth as DWORD;
    (*This).origHeight = bm.bmHeight as DWORD;

    TRACE!(
        b"width %d, height %d, bpp %d\n\0".as_ptr() as *const i8,
        bm.bmWidth,
        bm.bmHeight,
        bm.bmBitsPixel as i32,
    );

    hdcRef = CreateCompatibleDC(core::ptr::null_mut());

    (*This).himetricWidth = xpixels_to_himetric(bm.bmWidth as INT, hdcRef);
    (*This).himetricHeight = ypixels_to_himetric(bm.bmHeight as INT, hdcRef);
    (*This).stock_bitmap = GetCurrentObject(hdcRef, OBJ_BITMAP as UINT) as HBITMAP;

    (*This).loadtime_format = BITMAP_FORMAT_BMP as u32;

    DeleteDC(hdcRef);
}
unsafe extern "C" fn OLEPictureImpl_SetIcon(This: *mut OLEPictureImpl) {
    let mut infoIcon: ICONINFO = core::mem::zeroed();

    TRACE!(b"icon handle %p\n\0".as_ptr() as *const i8, (*This).desc.__bindgen_anon_1.icon.as_ref().hicon);
    
    if GetIconInfo((*This).desc.__bindgen_anon_1.icon.as_ref().hicon, &mut infoIcon) != 0 {
        let hdcRef: HDC;
        let mut bm: core::mem::MaybeUninit<BITMAP> = core::mem::MaybeUninit::uninit();

        TRACE!(b"bitmap handle for icon is %p\n\0".as_ptr() as *const i8, infoIcon.hbmColor);
        
        if GetObjectW(
            (if !infoIcon.hbmColor.is_null() {
                infoIcon.hbmColor
            } else {
                infoIcon.hbmMask
            }) as *mut core::ffi::c_void,
            core::mem::size_of::<BITMAP>() as i32,
            bm.as_mut_ptr() as LPVOID,
        ) as usize != core::mem::size_of::<BITMAP>()
        {
            ERR!(b"GetObject fails on icon bitmap\n\0".as_ptr() as *const i8);
            return;
        }

        let bm = bm.assume_init();
        
        (*This).origWidth = bm.bmWidth as u32;
        (*This).origHeight = (if !infoIcon.hbmColor.is_null() {
            bm.bmHeight
        } else {
            bm.bmHeight / 2
        }) as u32;
        
        hdcRef = GetDC(core::ptr::null_mut());

        (*This).himetricWidth = xpixels_to_himetric((*This).origWidth as i32, hdcRef);
        (*This).himetricHeight = ypixels_to_himetric((*This).origHeight as i32, hdcRef);

        ReleaseDC(core::ptr::null_mut(), hdcRef);

        DeleteObject(infoIcon.hbmMask as *mut core::ffi::c_void);
        if !infoIcon.hbmColor.is_null() {
            DeleteObject(infoIcon.hbmColor as *mut core::ffi::c_void);
        }
    } else {
        ERR!(b"GetIconInfo() fails on icon %p\n\0".as_ptr() as *const i8, (*This).desc.__bindgen_anon_1.icon.as_ref().hicon);
    }
}
unsafe extern "C" fn OLEPictureImpl_SetEMF(This: *mut OLEPictureImpl) {
    let mut emh: ENHMETAHEADER = core::mem::zeroed();
    
    
    GetEnhMetaFileHeader(
        (*This).desc.__bindgen_anon_1.emf.as_ref().hemf,
        core::mem::size_of::<ENHMETAHEADER>() as u32,
        &mut emh,
    );
    
    (*This).origWidth = 0;
    (*This).origHeight = 0;
    (*This).himetricWidth = (emh.rclFrame.right - emh.rclFrame.left) as OLE_XSIZE_HIMETRIC;
    (*This).himetricHeight = (emh.rclFrame.bottom - emh.rclFrame.top) as OLE_YSIZE_HIMETRIC;
}
unsafe extern "C" fn OLEPictureImpl_Construct(
    pictDesc: LPPICTDESC,
    fOwn: BOOL,
    pict: *mut *mut OLEPictureImpl,
) -> HRESULT {

    let newObject: *mut OLEPictureImpl;
    let hr: HRESULT;

    if !pictDesc.is_null() {
        TRACE!(b"(%p) type = %d\n\0".as_ptr(), pictDesc, (*pictDesc).picType);
    }

    newObject = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<OLEPictureImpl>() as SIZE_T,
    ) as *mut OLEPictureImpl;
    if newObject.is_null() {
        return E_OUTOFMEMORY;
    }

    (*newObject).IPicture_iface.lpVtbl = &raw const OLEPictureImpl_VTable as *mut IPictureVtbl;
    (*newObject).IDispatch_iface.lpVtbl = &raw const OLEPictureImpl_IDispatch_VTable as *mut IDispatchVtbl;
    (*newObject).IPersistStream_iface.lpVtbl = &raw const OLEPictureImpl_IPersistStream_VTable as *mut IPersistStreamVtbl;
    (*newObject).IConnectionPointContainer_iface.lpVtbl = &raw const OLEPictureImpl_IConnectionPointContainer_VTable as *mut IConnectionPointContainerVtbl;

    (*newObject).pCP = core::ptr::null_mut();
    hr = CreateConnectionPoint(
        &mut (*newObject).IPicture_iface as *mut IPicture as *mut IUnknown,
        &IID_IPropertyNotifySink,
        &mut (*newObject).pCP,
    );
    if hr != S_OK {
        HeapFree(GetProcessHeap(), 0, newObject as LPVOID);
        return hr;
    }

    (*newObject).ref_ = 1;
    (*newObject).hDCCur = core::ptr::null_mut();

    (*newObject).fOwn = fOwn;

    (*newObject).keepOrigFormat = TRUE;

    (*newObject).hbmMask = core::ptr::null_mut();
    (*newObject).hbmXor = core::ptr::null_mut();
    (*newObject).loadtime_magic = 0xdeadbeef;
    (*newObject).loadtime_format = 0;
    (*newObject).bIsDirty = FALSE;

    if !pictDesc.is_null() {
        (*newObject).desc = core::ptr::read(pictDesc);

        match (*pictDesc).picType {
            PICTYPE_BITMAP => {
                OLEPictureImpl_SetBitmap(newObject);
            }
            PICTYPE_METAFILE => {
                TRACE!(b"metafile handle %p\n\0".as_ptr(), (*pictDesc).__bindgen_anon_1.wmf.as_ref().hmeta);
                (*newObject).himetricWidth = (*pictDesc).__bindgen_anon_1.wmf.as_ref().xExt;
                (*newObject).himetricHeight = (*pictDesc).__bindgen_anon_1.wmf.as_ref().yExt;
            }
            PICTYPE_NONE => {
                (*newObject).himetricWidth = 0;
                (*newObject).himetricHeight = 0;
            }
            PICTYPE_ICON => {
                OLEPictureImpl_SetIcon(newObject);
            }
            PICTYPE_ENHMETAFILE => {
                OLEPictureImpl_SetEMF(newObject);
            }
            _ => {
                WARN!(b"Unsupported type %d\n\0".as_ptr(), (*pictDesc).picType);
                IPicture_Release(&mut (*newObject).IPicture_iface);
                return E_UNEXPECTED;
            }
        }
    } else {
        (*newObject).desc.picType = PICTYPE_UNINITIALIZED;
    }

    TRACE!(b"returning %p\n\0".as_ptr(), newObject);
    *pict = newObject;
    S_OK
}
unsafe extern "C" fn OLEPictureImpl_Destroy(Obj: *mut OLEPictureImpl) {
    TRACE!(b"(%p)\n\0".as_ptr() as *const i8, Obj);

    if !(*Obj).pCP.is_null() {
        IConnectionPoint_Release((*Obj).pCP);
    }

    if (*Obj).fOwn != 0 {
        match (*Obj).desc.picType {
            1 => {
                DeleteObject((*Obj).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as *mut core::ffi::c_void);
                if !(*Obj).hbmMask.is_null() {
                    DeleteObject((*Obj).hbmMask as *mut core::ffi::c_void);
                }
                if !(*Obj).hbmXor.is_null() {
                    DeleteObject((*Obj).hbmXor as *mut core::ffi::c_void);
                }
            }
            2 => {
                DeleteMetaFile((*Obj).desc.__bindgen_anon_1.wmf.as_ref().hmeta);
            }
            3 => {
                DestroyIcon((*Obj).desc.__bindgen_anon_1.icon.as_ref().hicon);
            }
            4 => {
                DeleteEnhMetaFile((*Obj).desc.__bindgen_anon_1.emf.as_ref().hemf);
            }
            0 | 0xFFFFFFFF => {}
            _ => {
                FIXME!(b"Unsupported type %d - unable to delete\n\0".as_ptr() as *const i8, (*Obj).desc.picType);
            }
        }
    }
    HeapFree(GetProcessHeap(), 0, (*Obj).data as LPVOID);
    HeapFree(GetProcessHeap(), 0, Obj as LPVOID);
}
stdmethod! {
fn OLEPictureImpl_AddRef(iface: *mut IPicture) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    let refCount: ULONG = InterlockedIncrement(&mut (*This).ref_) as ULONG;

    TRACE!(b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8, This, refCount.wrapping_sub(1));

    return refCount;
}
}
stdmethod! {
fn OLEPictureImpl_Release(iface: *mut IPicture) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    let refCount: ULONG = InterlockedDecrement(&raw mut (*This).ref_) as ULONG;

    TRACE!(b"(%p)->(ref before=%d)\n\0".as_ptr() as *const i8, This, refCount.wrapping_add(1));

    if refCount == 0 {
        OLEPictureImpl_Destroy(This);
    }

    refCount
}
}
stdmethod! {
fn OLEPictureImpl_QueryInterface(
    iface: *mut IPicture,
    riid: *const IID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);

    TRACE!(b"(%p)->(%s, %p)\n\0".as_ptr() as *const i8, This, debugstr_guid(riid), ppvObject);

    if ppvObject.is_null() {
        return E_INVALIDARG;
    }

    *ppvObject = core::ptr::null_mut();

    if IsEqualIID(&IID_IUnknown as *const IID, riid) || IsEqualIID(&IID_IPicture as *const IID, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IPicture_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(&IID_IDispatch as *const IID, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IDispatch_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(&IID_IPictureDisp as *const IID, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IDispatch_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(&IID_IPersist as *const IID, riid) || IsEqualIID(&IID_IPersistStream as *const IID, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IPersistStream_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID(&IID_IConnectionPointContainer as *const IID, riid) {
        *ppvObject = core::ptr::addr_of_mut!((*This).IConnectionPointContainer_iface) as *mut core::ffi::c_void;
    }

    if (*ppvObject).is_null() {
        FIXME!(b"() : asking for unsupported interface %s\n\0".as_ptr() as *const i8, debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    IPicture_AddRef(iface);

    return S_OK;
}
}
unsafe extern "C" fn OLEPicture_SendNotify(this: *mut OLEPictureImpl, dispID: DISPID) {
    let mut pEnum: *mut IEnumConnections = core::ptr::null_mut();
    let mut CD: CONNECTDATA = CONNECTDATA {
        pUnk: core::ptr::null_mut(),
        dwCookie: 0,
    };

    if IConnectionPoint_EnumConnections((*this).pCP, &mut pEnum) != S_OK {
        return;
    }

    while IEnumConnections_Next(pEnum, 1, &mut CD, core::ptr::null_mut()) == S_OK {
        let mut sink: *mut IPropertyNotifySink = core::ptr::null_mut();
        
        IUnknown_QueryInterface(CD.pUnk, &IID_IPropertyNotifySink, core::ptr::addr_of_mut!(sink) as *mut *mut core::ffi::c_void);
        IPropertyNotifySink_OnChanged(sink, dispID);
        IPropertyNotifySink_Release(sink);
        IUnknown_Release(CD.pUnk);
    }
    
    IEnumConnections_Release(pEnum);
}
stdmethod! {
fn OLEPictureImpl_get_Handle(
    iface: *mut IPicture,
    phandle: *mut OLE_HANDLE,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, phandle);

    if phandle.is_null() {
        return E_POINTER;
    }

    match (*This).desc.picType {
        0 | 4294967295 => {
            *phandle = 0 as OLE_HANDLE;
        }
        1 => {
            *phandle = HandleToUlong((*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as HANDLE) as OLE_HANDLE;
        }
        2 => {
            *phandle = HandleToUlong((*This).desc.__bindgen_anon_1.wmf.as_ref().hmeta as HANDLE) as OLE_HANDLE;
        }
        3 => {
            *phandle = HandleToUlong((*This).desc.__bindgen_anon_1.icon.as_ref().hicon as HANDLE) as OLE_HANDLE;
        }
        4 => {
            *phandle = HandleToUlong((*This).desc.__bindgen_anon_1.emf.as_ref().hemf as HANDLE) as OLE_HANDLE;
        }
        _ => {
            FIXME!(b"Unimplemented type %d\n\0".as_ptr() as *const i8, (*This).desc.picType);
            return E_NOTIMPL;
        }
    }
    TRACE!(b"returning handle %08x\n\0".as_ptr() as *const i8, *phandle);
    return S_OK;
}
}
stdmethod! {
fn OLEPictureImpl_get_hPal(
    iface: *mut IPicture,
    phandle: *mut OLE_HANDLE,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);

    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, phandle);

    if phandle.is_null() {
        return E_POINTER;
    }

    if (*This).desc.picType == PICTYPE_BITMAP {
        *phandle = HandleToUlong((*This).desc.__bindgen_anon_1.bmp.as_ref().hpal as HANDLE) as OLE_HANDLE;
        return S_OK;
    }

    return E_FAIL;
}
}
stdmethod! {
fn OLEPictureImpl_get_Type(
    iface: *mut IPicture,
    ptype: *mut i16,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%p): type is %d\n\0".as_ptr() as *const i8, This, ptype, (*This).desc.picType);

    if ptype.is_null() {
        return E_POINTER;
    }

    *ptype = (*This).desc.picType as i16;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_get_Width(
    iface: *mut IPicture,
    pwidth: *mut OLE_XSIZE_HIMETRIC,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(
        b"(%p)->(%p): width is %d\n\0".as_ptr() as *const i8,
        This,
        pwidth,
        (*This).himetricWidth,
    );
    *pwidth = (*This).himetricWidth;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_get_Height(
    iface: *mut IPicture,
    pheight: *mut OLE_YSIZE_HIMETRIC,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    if (*__wine_dbch___default).flags as i32
        & (1i32) << __WINE_DBCL_TRACE as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0"
                as *const u8 as *const i8,
            b"OLEPictureImpl_get_Height\0" as *const u8 as *const i8,
            611i32,
            b"(%p)->(%p): height is %d\n\0" as *const u8 as *const i8,
            This,
            pheight,
            (*This).himetricHeight,
        );
    }
    *pheight = (*This).himetricHeight;
    S_OK
}
}
unsafe extern "C" fn render_masked_bitmap(
    This: *mut OLEPictureImpl,
    hdc: HDC,
    x: LONG,
    y: LONG,
    cx: LONG,
    cy: LONG,
    xSrc: OLE_XPOS_HIMETRIC,
    ySrc: OLE_YPOS_HIMETRIC,
    cxSrc: OLE_XSIZE_HIMETRIC,
    cySrc: OLE_YSIZE_HIMETRIC,
    hbmMask: HBITMAP,
    hbmXor: HBITMAP,
) {

    const MM_ANISOTROPIC: i32 = 8;
    const SRCAND: u32 = 0x008800C6;
    const SRCPAINT: u32 = 0x00EE0086;
    const SRCCOPY: u32 = 0x00CC0020;

    let hdcBmp: HDC;

    hdcBmp = CreateCompatibleDC(core::ptr::null_mut());
    SetMapMode(hdcBmp, MM_ANISOTROPIC);
    SetWindowOrgEx(hdcBmp, 0, 0, core::ptr::null_mut());
    SetWindowExtEx(
        hdcBmp,
        (*This).himetricWidth as i32,
        (*This).himetricHeight as i32,
        core::ptr::null_mut(),
    );
    SetViewportOrgEx(hdcBmp, 0, (*This).origHeight as i32, core::ptr::null_mut());
    SetViewportExtEx(
        hdcBmp,
        (*This).origWidth as i32,
        -((*This).origHeight as i32),
        core::ptr::null_mut(),
    );

    if !hbmMask.is_null() {
        SetBkColor(hdc, RGB(255, 255, 255));
        SetTextColor(hdc, RGB(0, 0, 0));

        SelectObject(hdcBmp, hbmMask as HGDIOBJ);
        StretchBlt(
            hdc,
            x as i32,
            y as i32,
            cx as i32,
            cy as i32,
            hdcBmp,
            xSrc as i32,
            ySrc as i32,
            cxSrc as i32,
            cySrc as i32,
            SRCAND,
        );

        if !hbmXor.is_null() {
            SelectObject(hdcBmp, hbmXor as HGDIOBJ);
            StretchBlt(
                hdc,
                x as i32,
                y as i32,
                cx as i32,
                cy as i32,
                hdcBmp,
                xSrc as i32,
                ySrc as i32,
                cxSrc as i32,
                cySrc as i32,
                SRCPAINT,
            );
        } else {
            StretchBlt(
                hdc,
                x as i32,
                y as i32,
                cx as i32,
                cy as i32,
                hdcBmp,
                xSrc as i32,
                ySrc as i32 - (*This).himetricHeight as i32,
                cxSrc as i32,
                cySrc as i32,
                SRCPAINT,
            );
        }
    } else {
        SelectObject(hdcBmp, hbmXor as HGDIOBJ);
        StretchBlt(
            hdc,
            x as i32,
            y as i32,
            cx as i32,
            cy as i32,
            hdcBmp,
            xSrc as i32,
            ySrc as i32,
            cxSrc as i32,
            cySrc as i32,
            SRCCOPY,
        );
    }

    DeleteDC(hdcBmp);
}
stdmethod! {
fn OLEPictureImpl_Render(
    iface: *mut IPicture,
    hdc: HDC,
    x: LONG,
    y: LONG,
    cx: LONG,
    cy: LONG,
    xSrc: OLE_XPOS_HIMETRIC,
    ySrc: OLE_YPOS_HIMETRIC,
    cxSrc: OLE_XSIZE_HIMETRIC,
    cySrc: OLE_YSIZE_HIMETRIC,
    prcWBounds: LPCRECT,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(
        b"(%p)->(%p, (%d,%d), (%d,%d) <- (%d,%d), (%d,%d), %p)\n\0".as_ptr() as *const i8,
        This,
        hdc,
        x,
        y,
        cx,
        cy,
        xSrc,
        ySrc,
        cxSrc,
        cySrc,
        prcWBounds,
    );
    if !prcWBounds.is_null() {
        TRACE!(
            b"prcWBounds %s\n\0".as_ptr() as *const i8,
            wine_dbgstr_rect(prcWBounds as *const RECT),
        );
    }

    if cx == 0 || cy == 0 || cxSrc == 0 || cySrc == 0 {
        return CTL_E_INVALIDPROPERTYVALUE;
    }

    match (*This).desc.picType {
        PICTYPE_UNINITIALIZED | PICTYPE_NONE => {
            return S_OK;
        }
        PICTYPE_BITMAP => {
            let hbmMask: HBITMAP;
            let hbmXor: HBITMAP;

            if !(*This).hbmMask.is_null() {
                hbmMask = (*This).hbmMask;
                hbmXor = (*This).hbmXor;
            } else {
                hbmMask = core::ptr::null_mut();
                hbmXor = (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap;
            }

            render_masked_bitmap(
                This, hdc, x, y, cx, cy, xSrc, ySrc, cxSrc, cySrc, hbmMask, hbmXor,
            );
        }
        PICTYPE_ICON => {
            let mut info: ICONINFO = core::mem::zeroed();

            if GetIconInfo((*This).desc.__bindgen_anon_1.icon.as_ref().hicon, &mut info) == 0 {
                return E_FAIL;
            }

            render_masked_bitmap(
                This,
                hdc,
                x,
                y,
                cx,
                cy,
                xSrc,
                ySrc,
                cxSrc,
                cySrc,
                info.hbmMask,
                info.hbmColor,
            );

            DeleteObject(info.hbmMask as HGDIOBJ);
            if !info.hbmColor.is_null() {
                DeleteObject(info.hbmColor as HGDIOBJ);
            }
        }
        PICTYPE_METAFILE => {
            let mut prevOrg: POINT = core::mem::zeroed();
            let mut prevWndOrg: POINT = core::mem::zeroed();
            let mut prevExt: SIZE = core::mem::zeroed();
            let mut prevWndExt: SIZE = core::mem::zeroed();
            let oldmode: i32;

            oldmode = SetMapMode(hdc, MM_ANISOTROPIC);
            SetWindowOrgEx(
                hdc,
                xSrc as i32,
                (*This).himetricHeight as i32 - ySrc as i32,
                &mut prevWndOrg,
            );
            SetWindowExtEx(hdc, cxSrc as i32, -(cySrc as i32), &mut prevWndExt);
            SetViewportOrgEx(hdc, x as i32, y as i32, &mut prevOrg);
            SetViewportExtEx(hdc, cx as i32, cy as i32, &mut prevExt);

            if PlayMetaFile(hdc, (*This).desc.__bindgen_anon_1.wmf.as_ref().hmeta) == 0 {
                ERR!(b"PlayMetaFile failed!\n\0".as_ptr() as *const i8);
            }

            SetWindowExtEx(
                hdc,
                prevWndExt.cx as i32,
                prevWndExt.cy as i32,
                core::ptr::null_mut(),
            );
            SetWindowOrgEx(
                hdc,
                prevWndOrg.x as i32,
                prevWndOrg.y as i32,
                core::ptr::null_mut(),
            );
            SetViewportExtEx(
                hdc,
                prevExt.cx as i32,
                prevExt.cy as i32,
                core::ptr::null_mut(),
            );
            SetViewportOrgEx(
                hdc,
                prevOrg.x as i32,
                prevOrg.y as i32,
                core::ptr::null_mut(),
            );
            SetMapMode(hdc, oldmode);
        }
        PICTYPE_ENHMETAFILE => {
            let mut rc: RECT = RECT {
                left: x,
                top: y,
                right: x + cx,
                bottom: y + cy,
            };
            PlayEnhMetaFile(hdc, (*This).desc.__bindgen_anon_1.emf.as_ref().hemf, &mut rc as LPCRECT);
        }
        _ => {
            FIXME!(
                b"type %d not implemented\n\0".as_ptr() as *const i8,
                (*This).desc.picType,
            );
            return E_NOTIMPL;
        }
    }
    return S_OK;
}
}
stdmethod! {
fn OLEPictureImpl_set_hPal(
    iface: *mut IPicture,
    hpal: OLE_HANDLE,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);

    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_TRACE as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0"
                as *const u8 as *const i8,
            b"OLEPictureImpl_set_hPal\0" as *const u8 as *const i8,
            774i32,
            b"(%p)->(%08x)\n\0" as *const u8 as *const i8,
            This,
            hpal,
        );
    }

    if (*This).desc.picType == PICTYPE_BITMAP as u32 {
        (*This).desc.__bindgen_anon_1.bmp.as_mut().hpal = ULongToHandle(hpal as ULONG) as HPALETTE;
        OLEPicture_SendNotify(This, DISPID_PICT_HPAL);
        return S_OK;
    }

    return E_FAIL;
}
}
stdmethod! {
fn OLEPictureImpl_get_CurDC(
    iface: *mut IPicture,
    phdc: *mut HDC,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p), returning %p\n\0".as_ptr() as *const i8, This, (*This).hDCCur);
    if !phdc.is_null() {
        *phdc = (*This).hDCCur;
    }
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_SelectPicture(
    iface: *mut IPicture,
    hdcIn: HDC,
    phdcOut: *mut HDC,
    phbmpOut: *mut OLE_HANDLE,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%p, %p, %p)\n\0".as_ptr() as *const i8, This, hdcIn, phdcOut, phbmpOut);
    
    if (*This).desc.picType == PICTYPE_BITMAP {
        if !phdcOut.is_null() {
            *phdcOut = (*This).hDCCur;
        }
        if !(*This).hDCCur.is_null() {
            SelectObject((*This).hDCCur, (*This).stock_bitmap as HGDIOBJ);
        }
        if !hdcIn.is_null() {
            SelectObject(hdcIn, (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as HGDIOBJ);
        }
        (*This).hDCCur = hdcIn;
        if !phbmpOut.is_null() {
            *phbmpOut = HandleToUlong((*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as HANDLE) as OLE_HANDLE;
        }
        S_OK
    } else {
        FIXME!(b"Don't know how to select picture type %d\n\0".as_ptr() as *const i8, (*This).desc.picType);
        E_FAIL
    }
}
}
stdmethod! {
fn OLEPictureImpl_get_KeepOriginalFormat(
    iface: *mut IPicture,
    pfKeep: *mut BOOL,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%p)\n\0".as_ptr() as *const i8, This, pfKeep);
    if pfKeep.is_null() {
        return E_POINTER;
    }
    *pfKeep = (*This).keepOrigFormat;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_put_KeepOriginalFormat(
    iface: *mut IPicture,
    keep: BOOL,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%d)\n\0".as_ptr() as *const i8, This, keep);
    (*This).keepOrigFormat = keep;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_PictureChanged(iface: *mut IPicture) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    OLEPicture_SendNotify(This, DISPID_PICT_HANDLE);
    (*This).bIsDirty = 1;
    return S_OK;
}
}
stdmethod! {
fn OLEPictureImpl_get_Attributes(
    iface: *mut IPicture,
    pdwAttr: *mut DWORD,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    TRACE!(b"(%p)->(%p).\n\0".as_ptr() as *const i8, This, pdwAttr);

    if pdwAttr.is_null() {
        return E_POINTER;
    }

    *pdwAttr = 0;
    match (*This).desc.picType {
        PICTYPE_UNINITIALIZED | PICTYPE_NONE => {}
        PICTYPE_BITMAP => {
            if !(*This).hbmMask.is_null() {
                *pdwAttr = PICTURE_TRANSPARENT;
            }
        }
        PICTYPE_ICON => {
            *pdwAttr = PICTURE_TRANSPARENT;
        }
        PICTYPE_ENHMETAFILE | PICTYPE_METAFILE => {
            *pdwAttr = PICTURE_TRANSPARENT | PICTURE_SCALABLE;
        }
        _ => {
            FIXME!(b"Unknown pictype %d\n\0".as_ptr() as *const i8, (*This).desc.picType);
        }
    }
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_IConnectionPointContainer_QueryInterface(
    iface: *mut IConnectionPointContainer,
    riid: *const IID,
    ppvoid: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IConnectionPointContainer(iface);
    return IPicture_QueryInterface(core::ptr::addr_of_mut!((*This).IPicture_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEPictureImpl_IConnectionPointContainer_AddRef(
    iface: *mut IConnectionPointContainer,
) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IConnectionPointContainer(iface);
    return IPicture_AddRef(core::ptr::addr_of_mut!((*This).IPicture_iface));
}
}
stdmethod! {
fn OLEPictureImpl_IConnectionPointContainer_Release(
    iface: *mut IConnectionPointContainer,
) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IConnectionPointContainer(iface);
    return IPicture_Release(core::ptr::addr_of_mut!((*This).IPicture_iface));
}
}
stdmethod! {
fn OLEPictureImpl_EnumConnectionPoints(
    iface: *mut IConnectionPointContainer,
    ppEnum: *mut *mut IEnumConnectionPoints,
) -> HRESULT {
    let _This: *mut OLEPictureImpl = impl_from_IConnectionPointContainer(iface);
    FIXME!(
        b"(%p,%p), stub!\n\0".as_ptr() as *const i8,
        This,
        ppEnum,
    );
    E_NOTIMPL
}
}
stdmethod! {
fn OLEPictureImpl_FindConnectionPoint(
    iface: *mut IConnectionPointContainer,
    riid: *const IID,
    ppCP: *mut *mut IConnectionPoint,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IConnectionPointContainer(iface);
    TRACE!(
        b"(%p,%s,%p)\n\0".as_ptr() as *const i8,
        This,
        debugstr_guid(riid as *const _GUID),
        ppCP,
    );
    if ppCP.is_null() {
        return E_POINTER;
    }
    *ppCP = core::ptr::null_mut::<IConnectionPoint>();
    if IsEqualGUID(
        riid as *const GUID,
        &raw const IID_IPropertyNotifySink as *const GUID,
    )
    {
        return IConnectionPoint_QueryInterface(
            (*This).pCP,
            &raw const IID_IConnectionPoint,
            ppCP as *mut *mut ::core::ffi::c_void,
        );
    }
    FIXME!(
        b"no connection point for %s\n\0".as_ptr() as *const i8,
        debugstr_guid(riid as *const _GUID),
    );
    return CONNECT_E_NOCONNECTION;
}
}
stdmethod! {
fn OLEPictureImpl_IPersistStream_QueryInterface(
    iface: *mut IPersistStream,
    riid: *const IID,
    ppvoid: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);
    return IPicture_QueryInterface(core::ptr::addr_of_mut!((*This).IPicture_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEPictureImpl_IPersistStream_AddRef(iface: *mut IPersistStream) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);
    return IPicture_AddRef(core::ptr::addr_of_mut!((*This).IPicture_iface));
}
}
stdmethod! {
fn OLEPictureImpl_IPersistStream_Release(
    iface: *mut IPersistStream,
) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);
    return IPicture_Release(core::ptr::addr_of_mut!((*This).IPicture_iface));
}
}
stdmethod! {
fn OLEPictureImpl_GetClassID(
    iface: *mut IPersistStream,
    pClassID: *mut CLSID
) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0".as_ptr() as *const i8,
            b"OLEPictureImpl_GetClassID\0".as_ptr() as *const i8,
            995i32,
            b"(%p)\n\0".as_ptr() as *const i8,
            pClassID,
        );
    }
    *pClassID = CLSID_StdPicture;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_IsDirty(
    iface: *mut IPersistStream
) -> HRESULT {
    let _This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);
    FIXME!(b"(%p),stub!\n\0".as_ptr() as *const i8, This);
    E_NOTIMPL
}
}
unsafe extern "C" fn OLEPictureImpl_LoadWICSource(
    This: *mut OLEPictureImpl,
    src: *mut IWICBitmapSource,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut bih: BITMAPINFOHEADER = core::mem::zeroed();
    let mut width: UINT = 0;
    let mut height: UINT = 0;
    let stride: UINT;
    let buffersize: UINT;
    let mut bits: *mut BYTE = core::ptr::null_mut();
    let mut mask: *mut BYTE = core::ptr::null_mut();
    let mut rc: WICRect = core::mem::zeroed();
    let mut real_source: *mut IWICBitmapSource = core::ptr::null_mut();
    let mut x: UINT;
    let mut y: UINT;
    let white: COLORREF = RGB(255, 255, 255);
    let black: COLORREF = RGB(0, 0, 0);
    let mut has_alpha: BOOL = FALSE;

    hr = WICConvertBitmapSource(
        &raw const GUID_WICPixelFormat32bppBGRA,
        src,
        &mut real_source,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IWICBitmapSource_GetSize(real_source, &mut width, &mut height);
    if FAILED(hr) {
        HeapFree(GetProcessHeap(), 0, mask as LPVOID);
        IWICBitmapSource_Release(real_source);
        return hr;
    }

    bih.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bih.biWidth = width as i32;
    bih.biHeight = -(height as i32);
    bih.biPlanes = 1;
    bih.biBitCount = 32;
    bih.biCompression = 0;
    bih.biSizeImage = 0;
    bih.biXPelsPerMeter = 4085;
    bih.biYPelsPerMeter = 4085;
    bih.biClrUsed = 0;
    bih.biClrImportant = 0;

    stride = 4 * width;
    buffersize = stride * height;

    mask = HeapAlloc(GetProcessHeap(), 0, buffersize as SIZE_T) as *mut BYTE;
    if mask.is_null() {
        hr = 0x8007000Eu32 as HRESULT;
        HeapFree(GetProcessHeap(), 0, mask as LPVOID);
        IWICBitmapSource_Release(real_source);
        return hr;
    }

    (*This).desc.__bindgen_anon_1.bmp.as_mut().hbitmap = CreateDIBSection(
        core::ptr::null_mut(),
        &mut bih as *mut _ as *mut _,
        0,
        &mut bits as *mut _ as *mut _,
        core::ptr::null_mut(),
        0,
    );
    if (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap.is_null() {
        hr = 0x80004005u32 as HRESULT;
        HeapFree(GetProcessHeap(), 0, mask as LPVOID);
        IWICBitmapSource_Release(real_source);
        return hr;
    }

    rc.X = 0;
    rc.Y = 0;
    rc.Width = width as i32;
    rc.Height = height as i32;
    hr = IWICBitmapSource_CopyPixels(real_source, &mut rc, stride, buffersize, bits);
    if FAILED(hr) {
        DeleteObject((*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as *mut _);
        HeapFree(GetProcessHeap(), 0, mask as LPVOID);
        IWICBitmapSource_Release(real_source);
        return hr;
    }

    (*This).desc.picType = 1;
    OLEPictureImpl_SetBitmap(This);

    y = 0;
    while y < height {
        x = 0;
        while x < width {
            let pixel: *mut u32 = bits.offset((stride * y + 4 * x) as isize) as *mut u32;
            if (*pixel & 0x80000000) == 0 {
                has_alpha = TRUE;
                *(mask.offset((stride * y + 4 * x) as isize) as *mut u32) = black;
            } else {
                *(mask.offset((stride * y + 4 * x) as isize) as *mut u32) = white;
            }
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }

    if has_alpha != 0 {
        let hdcref: HDC;
        let hdcBmp: HDC;
        let hdcXor: HDC;
        let hdcMask: HDC;
        let hbmoldBmp: HBITMAP;
        let hbmoldXor: HBITMAP;
        let hbmoldMask: HBITMAP;

        hdcref = GetDC(core::ptr::null_mut());

        (*This).hbmXor = CreateDIBitmap(
            hdcref,
            &bih as *const BITMAPINFOHEADER as *const core::ffi::c_void,
            4,
            mask as *const _,
            &bih as *const _ as *const _,
            0,
        );

        (*This).hbmMask = CreateBitmap(width as i32, -(height as i32), 1, 1, core::ptr::null());
        hdcBmp = CreateCompatibleDC(core::ptr::null_mut());
        hdcXor = CreateCompatibleDC(core::ptr::null_mut());
        hdcMask = CreateCompatibleDC(core::ptr::null_mut());

        hbmoldBmp = SelectObject(hdcBmp, (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap as *mut _) as HBITMAP;
        hbmoldXor = SelectObject(hdcXor, (*This).hbmXor as *mut _) as HBITMAP;
        hbmoldMask = SelectObject(hdcMask, (*This).hbmMask as *mut _) as HBITMAP;

        SetBkColor(hdcXor, black);
        BitBlt(hdcMask, 0, 0, width as i32, height as i32, hdcXor, 0, 0, 0x00CC0020);
        BitBlt(hdcXor, 0, 0, width as i32, height as i32, hdcBmp, 0, 0, 0x008800C6);

        SelectObject(hdcBmp, hbmoldBmp as *mut _);
        SelectObject(hdcXor, hbmoldXor as *mut _);
        SelectObject(hdcMask, hbmoldMask as *mut _);

        DeleteDC(hdcBmp);
        DeleteDC(hdcXor);
        DeleteDC(hdcMask);
        ReleaseDC(core::ptr::null_mut(), hdcref);
    }

    HeapFree(GetProcessHeap(), 0, mask as LPVOID);
    IWICBitmapSource_Release(real_source);
    hr
}
unsafe extern "C" fn OLEPictureImpl_LoadWICDecoder(
    This: *mut OLEPictureImpl,
    decoder_clsid: *const CLSID,
    xbuf: *mut BYTE,
    xread: ULONG,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut factory: *mut IWICImagingFactory = core::ptr::null_mut();
    let mut decoder: *mut IWICBitmapDecoder = core::ptr::null_mut();
    let mut framedecode: *mut IWICBitmapFrameDecode = core::ptr::null_mut();
    let mut stream: *mut IWICStream = core::ptr::null_mut();

    let initresult = CoInitialize(core::ptr::null_mut());

    hr = CoCreateInstance(
        &raw const CLSID_WICImagingFactory,
        core::ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &raw const IID_IWICImagingFactory,
        &mut factory as *mut _ as *mut LPVOID,
    );

    if SUCCEEDED(hr) {
        hr = IWICImagingFactory_CreateStream(factory, &mut stream);
        IWICImagingFactory_Release(factory);
    }

    if SUCCEEDED(hr) {
        hr = IWICStream_InitializeFromMemory(stream, xbuf, xread);

        if SUCCEEDED(hr) {
            hr = CoCreateInstance(
                decoder_clsid,
                core::ptr::null_mut(),
                CLSCTX_INPROC_SERVER,
                &raw const IID_IWICBitmapDecoder,
                &mut decoder as *mut _ as *mut LPVOID,
            );

            if SUCCEEDED(hr) {
                hr = IWICBitmapDecoder_Initialize(decoder, stream as *mut IStream, WICDecodeMetadataCacheOnLoad);

                if SUCCEEDED(hr) {
                    hr = IWICBitmapDecoder_GetFrame(decoder, 0, &mut framedecode);
                }

                IWICBitmapDecoder_Release(decoder);
            }
        }

        IWICStream_Release(stream);
    }

    if SUCCEEDED(hr) {
        hr = OLEPictureImpl_LoadWICSource(This, framedecode as *mut IWICBitmapSource);
        IWICBitmapFrameDecode_Release(framedecode);
    }

    if SUCCEEDED(initresult) {
        CoUninitialize();
    }

    hr
}
unsafe extern "C" fn OLEPictureImpl_LoadIcon(
    This: *mut OLEPictureImpl,
    xbuf: *mut BYTE,
    xread: ULONG,
) -> HRESULT {
    let hicon: HICON;
    let cifd: *mut CURSORICONFILEDIR = xbuf as *mut CURSORICONFILEDIR;
    let hdcRef: HDC;
    let mut i: i32;

    TRACE!(
        b"(this %p, xbuf %p, xread %u)\n\0".as_ptr() as *const i8,
        This,
        xbuf,
        xread,
    );

    if (*cifd).idCount == 0 {
        ERR!(b"Invalid icon count of zero.\n\0".as_ptr() as *const i8);
        return E_FAIL;
    }

    i = 0;

    if (*cifd).idCount != 1 {
        i = 0;
        while i < (*cifd).idCount as i32 {
            // olepicture.c: `CURSORICONFILEDIRENTRY idEntries[1];` with `WORD idCount`
            // -- a flexible array, and this very loop only runs when idCount != 1, so
            // the multi-entry path IS the panicking path. Any .ico carrying more than
            // one size (most of them) hit it.
            if (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bWidth as i32 == 32 {
                break;
            }
            i += 1;
        }
        if i == (*cifd).idCount as i32 {
            i = 0;
        }
    }

    if xread < (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBOffset + (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize {
        ERR!(
            b"Icon data address %u is over %u bytes available.\n\0".as_ptr() as *const i8,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBOffset + (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize,
            xread,
        );
        return E_FAIL;
    }

    if (*cifd).idType == 2 {
        let buf: LPBYTE = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize + 4) as SIZE_T,
        ) as LPBYTE;
        memcpy(
            buf as *mut core::ffi::c_void,
            core::ptr::addr_of!((*(&raw mut (*cifd).idEntries[0]).add(i as usize)).xHotspot) as *const core::ffi::c_void,
            4,
        );
        memcpy(
            buf.offset(4) as *mut core::ffi::c_void,
            xbuf.offset((*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBOffset as isize) as *const core::ffi::c_void,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize as usize,
        );
        hicon = CreateIconFromResourceEx(
            buf,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize + 4,
            FALSE,
            0x00030000,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bWidth as i32,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bHeight as i32,
            0,
        );
        HeapFree(GetProcessHeap(), 0, buf as *mut core::ffi::c_void);
    } else {
        hicon = CreateIconFromResourceEx(
            xbuf.offset((*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBOffset as isize),
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).dwDIBSize,
            TRUE,
            0x00030000,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bWidth as i32,
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bHeight as i32,
            0,
        );
    }

    if hicon.is_null() {
        ERR!(b"CreateIcon failed.\n\0".as_ptr() as *const i8);
        return E_FAIL;
    } else {
        (*This).desc.picType = PICTYPE_ICON;
        (*This).desc.__bindgen_anon_1.icon.as_mut().hicon = hicon;
        (*This).origWidth = (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bWidth as DWORD;
        (*This).origHeight = (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bHeight as DWORD;
        hdcRef = CreateCompatibleDC(core::ptr::null_mut());
        (*This).himetricWidth = xpixels_to_himetric(
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bWidth as i32,
            hdcRef,
        );
        (*This).himetricHeight = ypixels_to_himetric(
            (*(&raw mut (*cifd).idEntries[0]).add(i as usize)).bHeight as i32,
            hdcRef,
        );
        DeleteDC(hdcRef);
        return S_OK;
    }
}
unsafe extern "C" fn OLEPictureImpl_LoadEnhMetafile(
    This: *mut OLEPictureImpl,
    data: *const BYTE,
    size: ULONG,
) -> HRESULT {

    let hemf: HENHMETAFILE;
    let mut hdr: ENHMETAHEADER = core::mem::zeroed();

    hemf = SetEnhMetaFileBits(size as UINT, data);
    if hemf.is_null() {
        return E_FAIL;
    }

    GetEnhMetaFileHeader(
        hemf,
        core::mem::size_of::<ENHMETAHEADER>() as UINT,
        &mut hdr,
    );

    (*This).desc.picType = PICTYPE_ENHMETAFILE as UINT;
    (*This).desc.__bindgen_anon_1.emf.as_mut().hemf = hemf;

    (*This).origWidth = 0 as DWORD;
    (*This).origHeight = 0 as DWORD;
    (*This).himetricWidth = (hdr.rclFrame.right - hdr.rclFrame.left) as OLE_XSIZE_HIMETRIC;
    (*This).himetricHeight = (hdr.rclFrame.bottom - hdr.rclFrame.top) as OLE_YSIZE_HIMETRIC;

    S_OK
}
unsafe extern "C" fn OLEPictureImpl_LoadAPM(
    This: *mut OLEPictureImpl,
    data: *const BYTE,
    size: ULONG,
) -> HRESULT {

    let header: *const APM_HEADER = data as *const APM_HEADER;
    let hmf: HMETAFILE;

    if (size as usize) < core::mem::size_of::<APM_HEADER>() {
        return E_FAIL;
    }
    if (*header).key != 0x9ac6cdd7 {
        return E_FAIL;
    }

    hmf = SetMetaFileBitsEx(
        (size as usize - core::mem::size_of::<APM_HEADER>()) as u32,
        data.offset(core::mem::size_of::<APM_HEADER>() as isize),
    );
    if hmf.is_null() {
        return E_FAIL;
    }

    (*This).desc.picType = PICTYPE_METAFILE;
    (*This).desc.__bindgen_anon_1.wmf.as_mut().hmeta = hmf;
    (*This).desc.__bindgen_anon_1.wmf.as_mut().xExt = 0;
    (*This).desc.__bindgen_anon_1.wmf.as_mut().yExt = 0;

    (*This).origWidth = 0;
    (*This).origHeight = 0;
    (*This).himetricWidth = MulDiv(
        (*header).right as i32 - (*header).left as i32,
        2540,
        (*header).inch as i32,
    ) as OLE_XSIZE_HIMETRIC;
    (*This).himetricHeight = MulDiv(
        (*header).bottom as i32 - (*header).top as i32,
        2540,
        (*header).inch as i32,
    ) as OLE_YSIZE_HIMETRIC;
    S_OK
}
stdmethod! {
fn OLEPictureImpl_Load(
    iface: *mut IPersistStream,
    pStm: *mut IStream,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut headerisdata: BOOL;
    let mut statfailed: BOOL = FALSE;
    let mut xread: ULONG = 0;
    let mut toread: ULONG;
    let mut headerread: ULONG;
    let mut xbuf: *mut BYTE;
    let mut header: [DWORD; 2] = [0; 2];
    let magic: WORD;
    let mut statstg: STATSTG = core::mem::zeroed();
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);

    TRACE!(b"(%p,%p)\n\0".as_ptr() as *const i8, This, pStm);

    hr = IStream_Stat(pStm, &mut statstg, 1);
    if hr != 0 {
        TRACE!(b"stat failed with hres %x, proceeding to read all data.\n\0".as_ptr() as *const i8, hr);
        statfailed = TRUE;
        statstg.cbSize.QuadPart = 8;
    }

    toread = 0;
    headerread = 0;
    headerisdata = FALSE;

    loop {
        hr = IStream_Read(pStm, header.as_mut_ptr() as *mut core::ffi::c_void, 8, &mut xread);
        if hr != 0 || xread != 8 {
            ERR!(b"Failure while reading picture header (hr is %x, nread is %d).\n\0".as_ptr() as *const i8, hr, xread);
            return if hr != 0 { hr } else { 0x80004005u32 as HRESULT };
        }
        headerread = headerread.wrapping_add(xread);
        xread = 0;

        if memcmp(
            &header[0] as *const DWORD as *const core::ffi::c_void,
            b"lt\0\0\0".as_ptr() as *const core::ffi::c_void,
            4,
        ) == 0
            && (statfailed != 0 || header[1].wrapping_add(headerread) as u64 <= statstg.cbSize.QuadPart)
        {
            if toread != 0 && toread != header[1] {
                FIXME!(b"varying lengths of image data (prev=%u curr=%u), only last one will be used\n\0".as_ptr() as *const i8, toread, header[1]);
            }
            toread = header[1];
            if statfailed != 0 {
                statstg.cbSize.QuadPart = header[1].wrapping_add(8) as u64;
                statfailed = FALSE;
            }
            if toread == 0 {
                break;
            }
        } else if memcmp(&header[0] as *const DWORD as *const core::ffi::c_void, b"GIF8\0".as_ptr() as *const core::ffi::c_void, 4) == 0
            || memcmp(&header[0] as *const DWORD as *const core::ffi::c_void, b"BM\0".as_ptr() as *const core::ffi::c_void, 2) == 0
            || memcmp(&header[0] as *const DWORD as *const core::ffi::c_void, b"\xff\xd8\0".as_ptr() as *const core::ffi::c_void, 2) == 0
            || header[0] == 1
            || header[0] == 0x10000
            || header[0] == 0x20000
            || header[1] as u64 > statstg.cbSize.QuadPart
            || header[1] == 0
        {
            headerisdata = TRUE;
            if toread == 0 {
                toread = (statstg.cbSize.QuadPart as u64).wrapping_sub(8) as ULONG;
            } else {
                toread = toread.wrapping_sub(8);
            }
            xread = 8;
        } else {
            FIXME!(b"Unknown stream header magic: %08x\n\0".as_ptr() as *const i8, header[0]);
            toread = header[1];
        }

        if headerisdata != 0 {
            break;
        }
    }

    if statfailed != 0 {
        let mut sizeinc: u32 = 4096;
        let mut origsize: u32 = sizeinc;
        let mut nread: ULONG = 42;

        TRACE!(b"Reading all data from stream.\n\0".as_ptr() as *const i8);
        xbuf = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, origsize as SIZE_T) as *mut BYTE;
        if headerisdata != 0 {
            memcpy(xbuf as *mut core::ffi::c_void, header.as_ptr() as *const core::ffi::c_void, 8);
        }
        loop {
            while xread < origsize {
                hr = IStream_Read(pStm, xbuf.add(xread as usize) as *mut core::ffi::c_void, origsize.wrapping_sub(xread), &mut nread);
                xread = xread.wrapping_add(nread);
                if hr != 0 || nread == 0 {
                    break;
                }
            }
            if nread == 0 || hr != 0 {
                break;
            }
            if xread == origsize {
                origsize = origsize.wrapping_add(sizeinc);
                sizeinc = 2u32.wrapping_mul(sizeinc);
                xbuf = HeapReAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, xbuf as *mut core::ffi::c_void, origsize as SIZE_T) as *mut BYTE;
            }
        }
        if hr != 0 {
            TRACE!(b"hr in no-stat loader case is %08x\n\0".as_ptr() as *const i8, hr);
        }
        TRACE!(b"loaded %d bytes.\n\0".as_ptr() as *const i8, xread);
        (*This).datalen = xread as i32;
        (*This).data = xbuf as *mut core::ffi::c_void;
    } else {
        (*This).datalen = toread.wrapping_add(if headerisdata != 0 { 8 } else { 0 }) as i32;
        (*This).data = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (*This).datalen as SIZE_T);
        xbuf = (*This).data as *mut BYTE;
        if xbuf.is_null() {
            return 0x8007000Eu32 as HRESULT;
        }

        if headerisdata != 0 {
            memcpy(xbuf as *mut core::ffi::c_void, header.as_ptr() as *const core::ffi::c_void, 8);
        }

        while xread < (*This).datalen as u32 {
            let mut nread_0: ULONG = 0;
            hr = IStream_Read(pStm, xbuf.add(xread as usize) as *mut core::ffi::c_void, ((*This).datalen as u32).wrapping_sub(xread), &mut nread_0);
            xread = xread.wrapping_add(nread_0);
            if hr != 0 || nread_0 == 0 {
                break;
            }
        }
        if xread != (*This).datalen as u32 {
            ERR!(b"Could only read %d of %d bytes out of stream?\n\0".as_ptr() as *const i8, xread, (*This).datalen);
        }
    }

    if (*This).datalen == 0 {
        (*This).desc.picType = 0;
        return 0;
    }

    magic = (*xbuf.add(0) as i32 + ((*xbuf.add(1) as i32) << 8)) as WORD;
    (*This).loadtime_format = magic as u32;

    match magic as i32 {
        0x4947 => {
            hr = OLEPictureImpl_LoadWICDecoder(This, &raw const CLSID_WICGifDecoder, xbuf, xread);
        }
        0xd8ff => {
            hr = OLEPictureImpl_LoadWICDecoder(This, &raw const CLSID_WICJpegDecoder, xbuf, xread);
        }
        0x4d42 => {
            hr = OLEPictureImpl_LoadWICDecoder(This, &raw const CLSID_WICBmpDecoder, xbuf, xread);
        }
        0x5089 => {
            hr = OLEPictureImpl_LoadWICDecoder(This, &raw const CLSID_WICPngDecoder, xbuf, xread);
        }
        0xcdd7 => {
            hr = OLEPictureImpl_LoadAPM(This, xbuf, xread);
        }
        0x0000 => {
            hr = OLEPictureImpl_LoadIcon(This, xbuf, xread);
        }
        _ => {
            hr = OLEPictureImpl_LoadEnhMetafile(This, xbuf, xread);
            if hr != 0 {
                FIXME!(b"Unknown magic %04x, %d read bytes:\n\0".as_ptr() as *const i8, magic as i32, xread);
                hr = 0x80004005u32 as HRESULT;
                let mut i: u32 = 0;
                while i < xread.wrapping_add(8) {
                    if i < 8 {
                        MESSAGE!(b"%02x \0".as_ptr() as *const i8, *(header.as_ptr() as *const u8).add(i as usize) as i32);
                    } else {
                        MESSAGE!(b"%02x \0".as_ptr() as *const i8, *xbuf.add(i.wrapping_sub(8) as usize) as i32);
                    }
                    if i.wrapping_rem(10) == 9 {
                        MESSAGE!(b"\n\0".as_ptr() as *const i8);
                    }
                    i = i.wrapping_add(1);
                }
                MESSAGE!(b"\n\0".as_ptr() as *const i8);
            }
        }
    }

    (*This).bIsDirty = FALSE;

    if hr == 0 {
        OLEPicture_SendNotify(This, 0);
    }
    hr
}
}
/// Wine 10.0 additions (keeping this crate's BOOL return): a NULL ppBuffer
/// only computes *pLength (GetSizeMax's query); a 16/32 bpp bitmap is
/// serialised as 24 bpp (Windows' BMP writer does the same); a bitmap GDI
/// cannot describe fails instead of serialising garbage; the screen DC is
/// released (it leaked).
unsafe extern "C" fn serializeBMP(
    hBitmap: HBITMAP,
    ppBuffer: *mut *mut core::ffi::c_void,
    pLength: *mut u32,
) -> i32 {
    let mut hBitmap = hBitmap;
    let mut hdib: HBITMAP = core::ptr::null_mut();

    let mut success: i32 = 0;
    let hDC: HDC;
    let pInfoBitmap: *mut BITMAPINFO;
    let mut iNumPaletteEntries: i32;
    let pPixelData: *mut u8;
    let pFileHeader: *mut BITMAPFILEHEADER;
    let pInfoHeader: *mut BITMAPINFO;

    pInfoBitmap = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<BITMAPINFOHEADER>().wrapping_add(
            256usize.wrapping_mul(core::mem::size_of::<RGBQUAD>())
        ) as SIZE_T ,
    ) as *mut BITMAPINFO;

    hDC = GetDC(core::ptr::null_mut());

    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW(hBitmap as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, &mut bm as *mut BITMAP as LPVOID) == 0 {
        HeapFree(GetProcessHeap(), 0, pInfoBitmap as LPVOID);
        ReleaseDC(core::ptr::null_mut(), hDC);
        return 0;
    }
    if bm.bmBitsPixel == 16 || bm.bmBitsPixel == 32 {
        (*pInfoBitmap).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
        (*pInfoBitmap).bmiHeader.biWidth = bm.bmWidth;
        (*pInfoBitmap).bmiHeader.biHeight = bm.bmHeight;
        (*pInfoBitmap).bmiHeader.biPlanes = 1;
        (*pInfoBitmap).bmiHeader.biBitCount = 24;
        (*pInfoBitmap).bmiHeader.biCompression = 0; /* BI_RGB */
        let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();
        hdib = CreateDIBSection(hDC, pInfoBitmap as *const core::ffi::c_void, 0 /* DIB_RGB_COLORS */,
            &mut bits, core::ptr::null_mut(), 0) as HBITMAP;
        let mut ok = !hdib.is_null();
        if ok && !ppBuffer.is_null() {
            let src = CreateCompatibleDC(hDC);
            let dst = CreateCompatibleDC(hDC);
            SelectObject(src, hBitmap as HGDIOBJ);
            SelectObject(dst, hdib as HGDIOBJ);
            ok = BitBlt(dst, 0, 0, bm.bmWidth, bm.bmHeight, src, 0, 0, 0x00CC0020 /* SRCCOPY */) != 0;
            DeleteDC(src);
            DeleteDC(dst);
        }
        if !ok {
            if !hdib.is_null() {
                DeleteObject(hdib as HGDIOBJ);
            }
            HeapFree(GetProcessHeap(), 0, pInfoBitmap as LPVOID);
            ReleaseDC(core::ptr::null_mut(), hDC);
            return 0;
        }
        hBitmap = hdib;
    }

    (*pInfoBitmap).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    if GetDIBits(
        hDC,
        hBitmap,
        0,
        0,
        core::ptr::null_mut(),
        pInfoBitmap,
        0,
    ) == 0
    {
        if !hdib.is_null() {
            DeleteObject(hdib as HGDIOBJ);
        }
        HeapFree(GetProcessHeap(), 0, pInfoBitmap as LPVOID);
        ReleaseDC(core::ptr::null_mut(), hDC);
        return 0;
    }

    if ppBuffer.is_null() {
        let entries: usize = if (*pInfoBitmap).bmiHeader.biClrUsed != 0 {
            ((*pInfoBitmap).bmiHeader.biClrUsed as usize).min(256)
        } else if (*pInfoBitmap).bmiHeader.biBitCount <= 8 {
            1usize << (*pInfoBitmap).bmiHeader.biBitCount
        } else {
            0
        };
        *pLength = (core::mem::size_of::<BITMAPFILEHEADER>()
            + core::mem::size_of::<BITMAPINFOHEADER>()
            + entries * core::mem::size_of::<RGBQUAD>()
            + (*pInfoBitmap).bmiHeader.biSizeImage as usize) as u32;
        if !hdib.is_null() {
            DeleteObject(hdib as HGDIOBJ);
        }
        HeapFree(GetProcessHeap(), 0, pInfoBitmap as LPVOID);
        ReleaseDC(core::ptr::null_mut(), hDC);
        return 1;
    }

    pPixelData = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (*pInfoBitmap).bmiHeader.biSizeImage as SIZE_T,
    ) as *mut u8;
    GetDIBits(
        hDC,
        hBitmap,
        0,
        (*pInfoBitmap).bmiHeader.biHeight as u32,
        pPixelData as LPVOID,
        pInfoBitmap,
        0,
    );

    if (*pInfoBitmap).bmiHeader.biClrUsed != 0 {
        iNumPaletteEntries = (*pInfoBitmap).bmiHeader.biClrUsed as i32;
        if iNumPaletteEntries > 256 {
            iNumPaletteEntries = 256;
        }
    } else if (*pInfoBitmap).bmiHeader.biBitCount as i32 <= 8 {
        iNumPaletteEntries = 1 << (*pInfoBitmap).bmiHeader.biBitCount as i32;
    } else {
        iNumPaletteEntries = 0;
    }

    *pLength = (core::mem::size_of::<BITMAPFILEHEADER>() as usize)
        .wrapping_add(core::mem::size_of::<BITMAPINFOHEADER>() as usize)
        .wrapping_add(
            (iNumPaletteEntries as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>() as usize)
        )
        .wrapping_add((*pInfoBitmap).bmiHeader.biSizeImage as usize) as u32;

    // (ppBuffer is non-NULL here: the NULL sizing call returned above.)
    *ppBuffer = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        *pLength as SIZE_T,
    );

    pFileHeader = *ppBuffer as *mut BITMAPFILEHEADER;
    (*pFileHeader).bfType = BITMAP_FORMAT_BMP as u16;
    (*pFileHeader).bfSize = *pLength;
    (*pFileHeader).bfOffBits = (core::mem::size_of::<BITMAPFILEHEADER>() as usize)
        .wrapping_add(core::mem::size_of::<BITMAPINFOHEADER>() as usize)
        .wrapping_add(
            (iNumPaletteEntries as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>() as usize)
        ) as u32;

    pInfoHeader = (*ppBuffer as *mut u8)
        .offset(core::mem::size_of::<BITMAPFILEHEADER>() as isize)
        as *mut BITMAPINFO;
    memcpy(
        pInfoHeader as *mut core::ffi::c_void,
        pInfoBitmap as *const core::ffi::c_void,
        (core::mem::size_of::<BITMAPINFOHEADER>() as usize).wrapping_add(
            (iNumPaletteEntries as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>() as usize)
        ),
    );
    memcpy(
        (*ppBuffer as *mut u8)
            .offset(core::mem::size_of::<BITMAPFILEHEADER>() as isize)
            .offset(core::mem::size_of::<BITMAPINFOHEADER>() as isize)
            .offset(
                (iNumPaletteEntries as usize)
                    .wrapping_mul(core::mem::size_of::<RGBQUAD>() as usize) as isize
            ) as *mut core::ffi::c_void,
        pPixelData as *const core::ffi::c_void,
        (*pInfoBitmap).bmiHeader.biSizeImage as usize,
    );
    success = 1;

    HeapFree(GetProcessHeap(), 0, pPixelData as LPVOID);
    HeapFree(GetProcessHeap(), 0, pInfoBitmap as LPVOID);
    if !hdib.is_null() {
        DeleteObject(hdib as HGDIOBJ);
    }
    ReleaseDC(core::ptr::null_mut(), hDC);
    return success;
}
unsafe extern "C" fn serializeIcon(
    hIcon: HICON,
    ppBuffer: *mut *mut core::ffi::c_void,
    pLength: *mut u32,
) -> i32 {
    let mut infoIcon: ICONINFO = core::mem::zeroed();
    let mut success: i32 = 0;

    *ppBuffer = core::ptr::null_mut();
    *pLength = 0;
    
    if GetIconInfo(hIcon, &mut infoIcon) != 0 {
        let hDC: HDC;
        let pInfoBitmap: *mut BITMAPINFO;
        let mut pIconData: *mut u8 = core::ptr::null_mut();
        let mut iDataSize: u32 = 0;

        pInfoBitmap = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>()) as SIZE_T,
        ) as *mut BITMAPINFO;

        hDC = GetDC(core::ptr::null_mut());
        (*pInfoBitmap).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
        GetDIBits(hDC, infoIcon.hbmColor, 0, 0, core::ptr::null_mut(), pInfoBitmap, 0);

        let pIconDir: *mut CURSORICONFILEDIR;
        let mut pIconEntry: *mut CURSORICONFILEDIRENTRY;
        let mut pIconBitmapHeader: *mut BITMAPINFOHEADER;
        let iOffsetPalette: u32;
        let iOffsetColorData: u32;
        let iOffsetMaskData: u32;
        let iLengthScanLineMask: u32;
        let mut iNumEntriesPalette: u32;

        iLengthScanLineMask = ((((*pInfoBitmap).bmiHeader.biWidth + 31) >> 5) << 2) as u32;

        iDataSize += (3 * core::mem::size_of::<u16>() + core::mem::size_of::<CURSORICONFILEDIRENTRY>() + core::mem::size_of::<BITMAPINFOHEADER>()) as u32;
        pIconData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, iDataSize as SIZE_T) as *mut u8;

        pIconDir = pIconData as *mut CURSORICONFILEDIR;
        (*pIconDir).idType = 1;
        (*pIconDir).idCount = 1;
        (*pIconDir).idReserved = 0;

        pIconEntry = pIconData.offset((3 * core::mem::size_of::<u16>()) as isize) as *mut CURSORICONFILEDIRENTRY;
        (*pIconEntry).bWidth = (*pInfoBitmap).bmiHeader.biWidth as u8;
        (*pIconEntry).bHeight = (*pInfoBitmap).bmiHeader.biHeight as u8;
        (*pIconEntry).bColorCount = if (*pInfoBitmap).bmiHeader.biBitCount < 8 {
            1 << (*pInfoBitmap).bmiHeader.biBitCount
        } else {
            0
        };
        (*pIconEntry).xHotspot = (*pInfoBitmap).bmiHeader.biPlanes;
        (*pIconEntry).yHotspot = (*pInfoBitmap).bmiHeader.biBitCount;
        (*pIconEntry).dwDIBSize = 0;
        (*pIconEntry).dwDIBOffset = (3 * core::mem::size_of::<u16>() + core::mem::size_of::<CURSORICONFILEDIRENTRY>()) as u32;

        pIconBitmapHeader = pIconData
            .offset((3 * core::mem::size_of::<u16>()) as isize)
            .offset(core::mem::size_of::<CURSORICONFILEDIRENTRY>() as isize) as *mut BITMAPINFOHEADER;
        *pIconBitmapHeader = core::ptr::read(core::ptr::addr_of!((*pInfoBitmap).bmiHeader));

        if ((*pInfoBitmap).bmiHeader.biBitCount == 16 && (*pInfoBitmap).bmiHeader.biCompression == 0)
            || (*pInfoBitmap).bmiHeader.biBitCount == 24
            || ((*pInfoBitmap).bmiHeader.biBitCount == 32 && (*pInfoBitmap).bmiHeader.biCompression == 0)
        {
            iNumEntriesPalette = (*pInfoBitmap).bmiHeader.biClrUsed;
            if iNumEntriesPalette > 256 {
                iNumEntriesPalette = 256;
            }
        } else if ((*pInfoBitmap).bmiHeader.biBitCount == 16 || (*pInfoBitmap).bmiHeader.biBitCount == 32)
            && (*pInfoBitmap).bmiHeader.biCompression == 3
        {
            iNumEntriesPalette = 3;
        } else if (*pInfoBitmap).bmiHeader.biBitCount <= 8 {
            iNumEntriesPalette = 1 << (*pInfoBitmap).bmiHeader.biBitCount;
        } else {
            iNumEntriesPalette = 0;
        }

        iOffsetPalette = iDataSize;
        iDataSize += iNumEntriesPalette * core::mem::size_of::<u32>() as u32;
        iOffsetColorData = iDataSize;
        iDataSize += (*pIconBitmapHeader).biSizeImage;
        iOffsetMaskData = iDataSize;
        iDataSize += (*pIconBitmapHeader).biHeight as u32 * iLengthScanLineMask;
        (*pIconBitmapHeader).biSizeImage += (*pIconBitmapHeader).biHeight as u32 * iLengthScanLineMask;
        (*pIconBitmapHeader).biHeight *= 2;
        pIconData = HeapReAlloc(GetProcessHeap(), 0, pIconData as *mut core::ffi::c_void, iDataSize as SIZE_T) as *mut u8;
        pIconEntry = pIconData.offset((3 * core::mem::size_of::<u16>()) as isize) as *mut CURSORICONFILEDIRENTRY;
        pIconBitmapHeader = pIconData
            .offset((3 * core::mem::size_of::<u16>()) as isize)
            .offset(core::mem::size_of::<CURSORICONFILEDIRENTRY>() as isize) as *mut BITMAPINFOHEADER;
        (*pIconEntry).dwDIBSize = iDataSize - (3 * core::mem::size_of::<u16>() + core::mem::size_of::<CURSORICONFILEDIRENTRY>()) as u32;

        GetDIBits(
            hDC,
            infoIcon.hbmColor,
            0,
            (*pInfoBitmap).bmiHeader.biHeight as u32,
            pIconData.offset(iOffsetColorData as isize) as *mut core::ffi::c_void,
            pInfoBitmap,
            0,
        );
        if iNumEntriesPalette > 0 {
            memcpy(
                pIconData.offset(iOffsetPalette as isize) as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!((*pInfoBitmap).bmiColors) as *const core::ffi::c_void,
                (iNumEntriesPalette as usize) * core::mem::size_of::<RGBQUAD>(),
            );
        }

        memset(
            pIconData.offset(iOffsetMaskData as isize) as *mut core::ffi::c_void,
            0,
            (iDataSize - iOffsetMaskData) as usize,
        );
        memset(
            pInfoBitmap as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<BITMAPINFOHEADER>() + 256 * core::mem::size_of::<RGBQUAD>(),
        );
        (*pInfoBitmap).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;

        GetDIBits(hDC, infoIcon.hbmMask, 0, 0, core::ptr::null_mut(), pInfoBitmap, 0);
        GetDIBits(
            hDC,
            infoIcon.hbmMask,
            0,
            (*pIconEntry).bHeight as u32,
            pIconData.offset(iOffsetMaskData as isize) as *mut core::ffi::c_void,
            pInfoBitmap,
            0,
        );

        *ppBuffer = pIconData as *mut core::ffi::c_void;
        *pLength = iDataSize;
        success = 1;

        if !hDC.is_null() {
            ReleaseDC(core::ptr::null_mut(), hDC);
        }
        DeleteObject(infoIcon.hbmMask as *mut core::ffi::c_void);
        if !infoIcon.hbmColor.is_null() {
            DeleteObject(infoIcon.hbmColor as *mut core::ffi::c_void);
        }
        HeapFree(GetProcessHeap(), 0, pInfoBitmap as *mut core::ffi::c_void);
    } else {
        printf(
            b"ERROR: Unable to get icon information (error %u)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
    success
}

#[repr(C)]
struct BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER,
    bmiColors: [RGBQUAD; 1],
}

#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

#[repr(C)]
struct RGBQUAD {
    rgbBlue: u8,
    rgbGreen: u8,
    rgbRed: u8,
    rgbReserved: u8,
}
unsafe extern "C" fn serializeEMF(
    hemf: HENHMETAFILE,
    buf: *mut *mut ::core::ffi::c_void,
    size: *mut ::core::ffi::c_uint,
) -> ::core::ffi::c_int {

    *size = GetEnhMetaFileBits(hemf, 0, core::ptr::null_mut());
    if *size == 0 {
        return 0;
    }

    *buf = HeapAlloc(GetProcessHeap(), 0, *size as SIZE_T);
    if (*buf).is_null() {
        return 0;
    }

    return (GetEnhMetaFileBits(hemf, *size, *buf as LPBYTE) != 0) as ::core::ffi::c_int;
}
stdmethod! {
fn OLEPictureImpl_Save(
    iface: *mut IPersistStream,
    pStm: *mut IStream,
    fClearDirty: i32,
) -> HRESULT {
    let mut hResult: HRESULT = E_NOTIMPL;
    let mut pIconData: *mut ::core::ffi::c_void = core::ptr::null_mut();
    let mut iDataSize: u32 = 0;
    let mut header: [u32; 2] = [0; 2];
    let mut dummy: u32 = 0;
    let mut serializeResult: i32 = 0;
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);

    TRACE!(b"%p %p %d\n\0".as_ptr(), This, pStm, fClearDirty);

    let current_block_72: u64;
    match (*This).desc.picType {
        0 => {
            header[0] = 0x0000746c;
            header[1] = 0;
            hResult = IStream_Write(
                pStm,
                core::ptr::addr_of_mut!(header) as *const ::core::ffi::c_void,
                (2 * core::mem::size_of::<u32>()) as u32,
                core::ptr::addr_of_mut!(dummy),
            );
        }
        3 => {
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                if serializeIcon(
                    (*This).desc.__bindgen_anon_1.icon.as_ref().hicon,
                    core::ptr::addr_of_mut!(pIconData),
                    core::ptr::addr_of_mut!(iDataSize),
                ) == 0
                {
                    ERR!(
                        b"(%p,%p,%d), serializeIcon() failed\n\0".as_ptr(),
                        This,
                        pStm,
                        fClearDirty,
                    );
                    hResult = E_FAIL;
                    current_block_72 = 54079586644752974;
                } else {
                    HeapFree(GetProcessHeap(), 0, (*This).data);
                    (*This).data = pIconData;
                    (*This).datalen = iDataSize as i32;
                    current_block_72 = 5783071609795492627;
                }
            } else {
                current_block_72 = 5783071609795492627;
            }
            match current_block_72 {
                54079586644752974 => {}
                _ => {
                    header[0] = if (*This).loadtime_magic != 0xdeadbeef {
                        (*This).loadtime_magic
                    } else {
                        0x0000746c
                    };
                    header[1] = (*This).datalen as u32;
                    IStream_Write(
                        pStm,
                        core::ptr::addr_of_mut!(header) as *const ::core::ffi::c_void,
                        (2 * core::mem::size_of::<u32>()) as u32,
                        core::ptr::addr_of_mut!(dummy),
                    );
                    IStream_Write(pStm, (*This).data, (*This).datalen as u32, core::ptr::addr_of_mut!(dummy));
                    hResult = S_OK;
                }
            }
        }
        1 => {
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                match if (*This).keepOrigFormat != 0 {
                    (*This).loadtime_format
                } else {
                    0x4d42
                } {
                    0x4d42 => {
                        serializeResult = serializeBMP(
                            (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap,
                            core::ptr::addr_of_mut!(pIconData),
                            core::ptr::addr_of_mut!(iDataSize),
                        );
                    }
                    0xd8ff => {
                        FIXME!(
                            b"(%p,%p,%d), PICTYPE_BITMAP (format JPEG) not implemented!\n\0".as_ptr(),
                            This,
                            pStm,
                            fClearDirty,
                        );
                    }
                    0x4947 => {
                        FIXME!(
                            b"(%p,%p,%d), PICTYPE_BITMAP (format GIF) not implemented!\n\0".as_ptr(),
                            This,
                            pStm,
                            fClearDirty,
                        );
                    }
                    0x5089 => {
                        FIXME!(
                            b"(%p,%p,%d), PICTYPE_BITMAP (format PNG) not implemented!\n\0".as_ptr(),
                            This,
                            pStm,
                            fClearDirty,
                        );
                    }
                    _ => {
                        FIXME!(
                            b"(%p,%p,%d), PICTYPE_BITMAP (format UNKNOWN, using BMP?) not implemented!\n\0".as_ptr(),
                            This,
                            pStm,
                            fClearDirty,
                        );
                    }
                }

                if serializeResult == 0 {
                    hResult = E_FAIL;
                    current_block_72 = 54079586644752974;
                } else {
                    HeapFree(GetProcessHeap(), 0, (*This).data);
                    (*This).data = pIconData;
                    (*This).datalen = iDataSize as i32;
                    current_block_72 = 5235537862154438448;
                }
            } else {
                current_block_72 = 5235537862154438448;
            }
            match current_block_72 {
                54079586644752974 => {}
                _ => {
                    header[0] = if (*This).loadtime_magic != 0xdeadbeef {
                        (*This).loadtime_magic
                    } else {
                        0x0000746c
                    };
                    header[1] = (*This).datalen as u32;
                    IStream_Write(
                        pStm,
                        core::ptr::addr_of_mut!(header) as *const ::core::ffi::c_void,
                        (2 * core::mem::size_of::<u32>()) as u32,
                        core::ptr::addr_of_mut!(dummy),
                    );
                    IStream_Write(pStm, (*This).data, (*This).datalen as u32, core::ptr::addr_of_mut!(dummy));
                    hResult = S_OK;
                }
            }
        }
        4 => {
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                serializeResult = serializeEMF(
                    (*This).desc.__bindgen_anon_1.emf.as_ref().hemf,
                    core::ptr::addr_of_mut!(pIconData),
                    core::ptr::addr_of_mut!(iDataSize),
                );
                if serializeResult == 0 {
                    hResult = E_FAIL;
                    current_block_72 = 54079586644752974;
                } else {
                    HeapFree(GetProcessHeap(), 0, (*This).data);
                    (*This).data = pIconData;
                    (*This).datalen = iDataSize as i32;
                    current_block_72 = 12758904613967585247;
                }
            } else {
                current_block_72 = 12758904613967585247;
            }
            match current_block_72 {
                54079586644752974 => {}
                _ => {
                    header[0] = 0x0000746c;
                    header[1] = (*This).datalen as u32;
                    IStream_Write(
                        pStm,
                        core::ptr::addr_of_mut!(header) as *const ::core::ffi::c_void,
                        (2 * core::mem::size_of::<u32>()) as u32,
                        core::ptr::addr_of_mut!(dummy),
                    );
                    IStream_Write(pStm, (*This).data, (*This).datalen as u32, core::ptr::addr_of_mut!(dummy));
                    hResult = S_OK;
                }
            }
        }
        2 => {
            FIXME!(
                b"(%p,%p,%d), PICTYPE_METAFILE not implemented!\n\0".as_ptr(),
                This,
                pStm,
                fClearDirty,
            );
        }
        _ => {
            FIXME!(
                b"(%p,%p,%d), [unknown type] not implemented!\n\0".as_ptr(),
                This,
                pStm,
                fClearDirty,
            );
        }
    }
    if hResult == S_OK && fClearDirty != 0 {
        (*This).bIsDirty = 0;
    }
    hResult
}
}
stdmethod! {
fn OLEPictureImpl_GetSizeMax(
    iface: *mut IPersistStream,
    pcbSize: *mut ULARGE_INTEGER,
) -> HRESULT {
    // Wine 10.0 implements this (was an E_NOTIMPL stub): the stored data's
    // length, or for a dirty/unloaded bitmap the size serializeBMP would
    // write, plus the 8-byte stream header Save emits.
    let This: *mut OLEPictureImpl = impl_from_IPersistStream(iface);
    let mut hr: HRESULT = E_NOTIMPL;
    let mut datasize: u32 = (*This).datalen as u32;

    if pcbSize.is_null() {
        return E_INVALIDARG;
    }

    match (*This).desc.picType as i32 {
        0 /* PICTYPE_NONE */ => hr = S_OK,
        1 /* PICTYPE_BITMAP */ => {
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                let fmt = if (*This).keepOrigFormat != 0 { (*This).loadtime_format as u32 } else { 0x4d42 };
                if fmt == 0x4d42 /* BITMAP_FORMAT_BMP */ {
                    hr = if serializeBMP(
                        (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap,
                        core::ptr::null_mut(),
                        &mut datasize,
                    ) != 0 { S_OK } else { E_INVALIDARG };
                }
                // JPEG/GIF/PNG re-encoding is not implemented (Wine: FIXME)
            } else {
                hr = S_OK;
            }
        }
        _ => {} /* icon / metafile / enhmetafile: not implemented (Wine: FIXME) */
    }

    (*pcbSize).u.HighPart = 0;
    (*pcbSize).u.LowPart = datasize.wrapping_add(8);
    hr
}
}
stdmethod! {
fn OLEPictureImpl_SaveAsFile(
    iface: *mut IPicture,
    stream: *mut IStream,
    mem_copy: BOOL,
    size: *mut LONG,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IPicture(iface);
    let mut data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut data_size: u32 = 0;
    let mut written: ULONG = 0;
    let hr: HRESULT;

    FIXME!(
        b"(%p)->(%p,%d,%p): semi-stub\n\0".as_ptr() as *const i8,
        This,
        stream,
        mem_copy,
        size,
    );

    match (*This).desc.picType {
        0 => {
            return 0;
        }
        3 => {
            if mem_copy == 0 {
                return 0x80004005u32 as i32;
            }
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                if serializeIcon(
                    (*This).desc.__bindgen_anon_1.icon.as_ref().hicon,
                    core::ptr::addr_of_mut!(data),
                    core::ptr::addr_of_mut!(data_size),
                ) == 0
                {
                    return 0x80004005u32 as i32;
                }
                HeapFree(GetProcessHeap(), 0, (*This).data);
                (*This).data = data;
                (*This).datalen = data_size as i32;
            }
            hr = IStream_Write(
                stream,
                (*This).data,
                (*This).datalen as u32,
                core::ptr::addr_of_mut!(written),
            );
            if hr == 0 && !size.is_null() {
                *size = written as i32;
            }
            return hr;
        }
        1 => {
            if mem_copy == 0 {
                return 0x80004005u32 as i32;
            }
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                match if (*This).keepOrigFormat != 0 {
                    (*This).loadtime_format
                } else {
                    0x4d42
                } {
                    19778 => {
                        if serializeBMP(
                            (*This).desc.__bindgen_anon_1.bmp.as_ref().hbitmap,
                            core::ptr::addr_of_mut!(data),
                            core::ptr::addr_of_mut!(data_size),
                        ) == 0
                        {
                            return 0x80004005u32 as i32;
                        }
                    }
                    55551 => {
                        FIXME!(b"BITMAP_FORMAT_JPEG is not implemented\n\0".as_ptr() as *const i8);
                        return 0x80004001u32 as i32;
                    }
                    18759 => {
                        FIXME!(b"BITMAP_FORMAT_GIF is not implemented\n\0".as_ptr() as *const i8);
                        return 0x80004001u32 as i32;
                    }
                    20617 => {
                        FIXME!(b"BITMAP_FORMAT_PNG is not implemented\n\0".as_ptr() as *const i8);
                        return 0x80004001u32 as i32;
                    }
                    _ => {
                        FIXME!(
                            b"PICTYPE_BITMAP/%#x is not implemented\n\0".as_ptr() as *const i8,
                            (*This).loadtime_format,
                        );
                        return 0x80004001u32 as i32;
                    }
                }
                HeapFree(GetProcessHeap(), 0, (*This).data);
                (*This).data = data;
                (*This).datalen = data_size as i32;
            }
            hr = IStream_Write(
                stream,
                (*This).data,
                (*This).datalen as u32,
                core::ptr::addr_of_mut!(written),
            );
            if hr == 0 && !size.is_null() {
                *size = written as i32;
            }
            return hr;
        }
        2 => {
            FIXME!(b"PICTYPE_METAFILE is not implemented\n\0".as_ptr() as *const i8);
            return 0x80004001u32 as i32;
        }
        4 => {
            if mem_copy == 0 {
                return 0x80004005u32 as i32;
            }
            if (*This).bIsDirty != 0 || (*This).data.is_null() {
                if serializeEMF(
                    (*This).desc.__bindgen_anon_1.emf.as_ref().hemf,
                    core::ptr::addr_of_mut!(data),
                    core::ptr::addr_of_mut!(data_size),
                ) == 0
                {
                    return 0x80004005u32 as i32;
                }
                HeapFree(GetProcessHeap(), 0, (*This).data);
                (*This).data = data;
                (*This).datalen = data_size as i32;
            }
            hr = IStream_Write(
                stream,
                (*This).data,
                (*This).datalen as u32,
                core::ptr::addr_of_mut!(written),
            );
            if hr == 0 && !size.is_null() {
                *size = written as i32;
            }
            return hr;
        }
        _ => {
            FIXME!(
                b"%#x is not implemented\n\0".as_ptr() as *const i8,
                (*This).desc.picType,
            );
        }
    }
    return 0x80004001u32 as i32;
}
}
stdmethod! {
fn OLEPictureImpl_IDispatch_QueryInterface(
    iface: *mut IDispatch,
    riid: *const IID,
    ppvoid: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IDispatch(iface);
    return IPicture_QueryInterface(core::ptr::addr_of_mut!((*This).IPicture_iface), riid, ppvoid);
}
}
stdmethod! {
fn OLEPictureImpl_IDispatch_AddRef(iface: *mut IDispatch) -> ULONG {
    let This: *mut OLEPictureImpl = impl_from_IDispatch(iface);
    return IPicture_AddRef(core::ptr::addr_of_mut!((*This).IPicture_iface));
}
}
stdmethod! {
fn OLEPictureImpl_IDispatch_Release(
    iface: *mut IDispatch
) -> ULONG {
    let This = impl_from_IDispatch(iface);
    IPicture_Release(core::ptr::addr_of_mut!((*This).IPicture_iface))
}
}
stdmethod! {
fn OLEPictureImpl_GetTypeInfoCount(
    iface: *mut IDispatch,
    pctinfo: *mut ::core::ffi::c_uint,
) -> HRESULT {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"OLEPictureImpl_GetTypeInfoCount\0" as *const u8 as *const ::core::ffi::c_char,
            2020 as ::core::ffi::c_int,
            b"(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            pctinfo,
        );
    }
    *pctinfo = 1 as ::core::ffi::c_uint;
    S_OK
}
}
stdmethod! {
fn OLEPictureImpl_GetTypeInfo(
    iface: *mut IDispatch,
    iTInfo: UINT,
    lcid: LCID,
    ppTInfo: *mut *mut ITypeInfo,
) -> HRESULT {
    static stdole2tlb: [WCHAR; 12] = [
        's' as u16,
        't' as u16,
        'd' as u16,
        'o' as u16,
        'l' as u16,
        'e' as u16,
        '2' as u16,
        '.' as u16,
        't' as u16,
        'l' as u16,
        'b' as u16,
        0,
    ];
    let mut tl: *mut ITypeLib = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(b"(iTInfo=%d, lcid=%04x, %p)\n\0".as_ptr() as *const i8, iTInfo, lcid as i32, ppTInfo);

    if iTInfo != 0 {
        return E_FAIL;
    }

    hres = LoadTypeLib(stdole2tlb.as_ptr(), &mut tl as *mut *mut ITypeLib as *mut *mut crate::olesys::ITypeLib);
    if FAILED(hres) {
        ERR!(b"Could not load stdole2.tlb\n\0".as_ptr() as *const i8);
        return hres;
    }

    hres = ITypeLib_GetTypeInfoOfGuid(tl, &IID_IPictureDisp, ppTInfo);
    if FAILED(hres) {
        ERR!(b"Did not get IPictureDisp typeinfo from typelib, hres %x\n\0".as_ptr() as *const i8, hres);
    }

    return hres;
}
}
stdmethod! {
fn OLEPictureImpl_GetIDsOfNames(
    iface: *mut IDispatch,
    riid: *const IID,
    rgszNames: *mut LPOLESTR,
    cNames: UINT,
    lcid: LCID,
    rgDispId: *mut DISPID,
) -> HRESULT {
    let mut pTInfo: *mut ITypeInfo = core::ptr::null_mut();
    let mut hres: HRESULT;

    TRACE!(
        b"(%p,%s,%p,cNames=%d,lcid=%04x,%p)\n\0".as_ptr() as *const i8,
        iface,
        debugstr_guid(riid),
        rgszNames,
        cNames,
        lcid as i32,
        rgDispId,
    );

    if cNames == 0 {
        return E_INVALIDARG;
    } else {
        hres = OLEPictureImpl_GetTypeInfo(iface, 0, lcid, &mut pTInfo);

        if FAILED(hres) {
            ERR!(b"GetTypeInfo failed.\n\0".as_ptr() as *const i8);
            return hres;
        }

        hres = DispGetIDsOfNames(pTInfo as *mut crate::olesys::ITypeInfo, rgszNames as *mut *mut OLECHAR, cNames, rgDispId);
        ITypeInfo_Release(pTInfo);

        return hres;
    }
}
}
stdmethod! {
fn OLEPictureImpl_Invoke(
    iface: *mut IDispatch,
    dispIdMember: DISPID,
    riid: *const IID,
    lcid: LCID,
    wFlags: WORD,
    pDispParams: *mut DISPPARAMS,
    pVarResult: *mut VARIANT,
    pExepInfo: *mut EXCEPINFO,
    puArgErr: *mut UINT,
) -> HRESULT {
    let This: *mut OLEPictureImpl = impl_from_IDispatch(iface);
    let mut hr: HRESULT;

    if !IsEqualIID(riid, &IID_NULL) {
        ERR!(b"riid was %s instead of IID_NULL\n\0".as_ptr() as *const i8, debugstr_guid(riid));
        return DISP_E_UNKNOWNNAME;
    }

    if pDispParams.is_null() {
        ERR!(b"null pDispParams not allowed\n\0".as_ptr() as *const i8);
        return DISP_E_PARAMNOTOPTIONAL;
    }

    if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
        if (*pDispParams).cArgs != 0 {
            ERR!(
                b"param count for DISPATCH_PROPERTYGET was %d instead of 0\n\0".as_ptr() as *const i8,
                (*pDispParams).cArgs,
            );
            return DISP_E_BADPARAMCOUNT;
        }
        if pVarResult.is_null() {
            ERR!(b"null pVarResult not allowed when DISPATCH_PROPERTYGET specified\n\0".as_ptr() as *const i8);
            return DISP_E_PARAMNOTOPTIONAL;
        }
    } else if wFlags as i32 & DISPATCH_PROPERTYPUT != 0 {
        if (*pDispParams).cArgs != 1 {
            ERR!(
                b"param count for DISPATCH_PROPERTYPUT was %d instead of 1\n\0".as_ptr() as *const i8,
                (*pDispParams).cArgs,
            );
            return DISP_E_BADPARAMCOUNT;
        }
    }

    match dispIdMember {
        DISPID_PICT_HANDLE => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                TRACE!(b"DISPID_PICT_HANDLE\n\0".as_ptr() as *const i8);
                *V_VT_ref(pVarResult) = VT_I4 as i32 as VARTYPE;
                return IPicture_get_Handle(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_UINT_ref(pVarResult),
                );
            }
        }
        DISPID_PICT_HPAL => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                TRACE!(b"DISPID_PICT_HPAL\n\0".as_ptr() as *const i8);
                *V_VT_ref(pVarResult) = VT_I4 as i32 as VARTYPE;
                return IPicture_get_hPal(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_UINT_ref(pVarResult),
                );
            } else if wFlags as i32 & DISPATCH_PROPERTYPUT != 0 {
                let mut vararg: VARIANTARG = core::mem::zeroed();

                TRACE!(b"DISPID_PICT_HPAL\n\0".as_ptr() as *const i8);

                VariantInit(core::ptr::addr_of_mut!(vararg));
                hr = VariantChangeTypeEx(
                    core::ptr::addr_of_mut!(vararg),
                    (*pDispParams).rgvarg.offset(0),
                    lcid,
                    0,
                    VT_I4 as i32 as VARTYPE,
                );
                if FAILED(hr) {
                    return hr;
                }

                hr = IPicture_set_hPal(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_I4(core::ptr::addr_of_mut!(vararg)) as OLE_HANDLE,
                );

                VariantClear(core::ptr::addr_of_mut!(vararg));
                return hr;
            }
        }
        DISPID_PICT_TYPE => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                TRACE!(b"DISPID_PICT_TYPE\n\0".as_ptr() as *const i8);
                *V_VT_ref(pVarResult) = VT_I2 as i32 as VARTYPE;
                return OLEPictureImpl_get_Type(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_I2_ref(pVarResult),
                );
            }
        }
        DISPID_PICT_WIDTH => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                TRACE!(b"DISPID_PICT_WIDTH\n\0".as_ptr() as *const i8);
                *V_VT_ref(pVarResult) = VT_I4 as i32 as VARTYPE;
                return IPicture_get_Width(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_I4_ref(pVarResult),
                );
            }
        }
        DISPID_PICT_HEIGHT => {
            if wFlags as u32 & DISPATCH_PROPERTYGET != 0 {
                TRACE!(b"DISPID_PICT_HEIGHT\n\0".as_ptr() as *const i8);
                *V_VT_ref(pVarResult) = VT_I4 as i32 as VARTYPE;
                return IPicture_get_Height(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    V_I4_ref(pVarResult),
                );
            }
        }
        DISPID_PICT_RENDER => {
            if wFlags as i32 & DISPATCH_METHOD != 0 {
                let args: *mut VARIANTARG = (*pDispParams).rgvarg;

                TRACE!(b"DISPID_PICT_RENDER\n\0".as_ptr() as *const i8);

                if (*pDispParams).cArgs != 10 {
                    return DISP_E_BADPARAMCOUNT;
                }

                let mut i: i32 = 0;
                while (i as u32) < (*pDispParams).cArgs {
                    if V_VT(args.offset(i as isize)) != VT_I4 as i32 as VARTYPE {
                        ERR!(
                            b"DISPID_PICT_RENDER: wrong argument type %d:%d\n\0".as_ptr() as *const i8,
                            i,
                            V_VT(args.offset(i as isize)) as i32,
                        );
                        return DISP_E_TYPEMISMATCH;
                    }
                    i += 1;
                }

                return IPicture_Render(
                    core::ptr::addr_of_mut!((*This).IPicture_iface),
                    LongToHandle(V_I4(args.offset(9)) as i32) as HDC,
                    V_I4(args.offset(8)),
                    V_I4(args.offset(7)),
                    V_I4(args.offset(6)),
                    V_I4(args.offset(5)),
                    V_I4(args.offset(4)),
                    V_I4(args.offset(3)),
                    V_I4(args.offset(2)),
                    V_I4(args.offset(1)),
                    core::ptr::null::<RECT>(),
                );
            }
        }
        _ => {}
    }

    ERR!(
        b"invalid dispid 0x%x or wFlags 0x%x\n\0".as_ptr() as *const i8,
        dispIdMember,
        wFlags as i32,
    );
    DISP_E_MEMBERNOTFOUND
}
}

#[no_mangle]
pub unsafe extern "system" fn OleCreatePictureIndirect(
    lpPictDesc: LPPICTDESC,
    riid: REFIID,
    Own: BOOL,
    ppvObj: *mut *mut core::ffi::c_void,
) -> HRESULT {

    let mut newPict: *mut OLEPictureImpl = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        b"(%p,%s,%d,%p)\n\0".as_ptr(),
        lpPictDesc,
        debugstr_guid(riid),
        Own,
        ppvObj,
    );

    *ppvObj = core::ptr::null_mut();

    hr = OLEPictureImpl_Construct(lpPictDesc, Own, &mut newPict);
    if hr != S_OK {
        return hr;
    }

    hr = IPicture_QueryInterface(
        core::ptr::addr_of_mut!((*newPict).IPicture_iface),
        riid,
        ppvObj,
    );

    IPicture_Release(core::ptr::addr_of_mut!((*newPict).IPicture_iface));

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn OleLoadPicture(
    lpstream: LPSTREAM,
    _lSize: LONG,
    fRunmode: BOOL,
    riid: REFIID,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut ps: LPPERSISTSTREAM = core::ptr::null_mut();
    let mut newpic: *mut IPicture = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        b"(%p,%d,%d,%s,%p), partially implemented.\n\0".as_ptr() as *const i8,
        lpstream,
        lSize,
        fRunmode,
        debugstr_guid(riid),
        ppvObj,
    );

    hr = OleCreatePictureIndirect(
        core::ptr::null_mut(),
        riid,
        if fRunmode == 0 { 1 } else { 0 },
        &raw mut newpic as *mut *mut core::ffi::c_void,
    );
    if hr != S_OK {
        return hr;
    }

    hr = IPicture_QueryInterface(
        newpic,
        &raw const IID_IPersistStream,
        &raw mut ps as *mut *mut core::ffi::c_void,
    );
    if hr != S_OK {
        ERR!(b"Could not get IPersistStream iface from Ole Picture?\n\0".as_ptr() as *const i8);
        IPicture_Release(newpic);
        *ppvObj = core::ptr::null_mut();
        return hr;
    }

    hr = IPersistStream_Load(ps, lpstream);
    IPersistStream_Release(ps);
    if FAILED(hr) {
        ERR!(b"IPersistStream_Load failed\n\0".as_ptr() as *const i8);
        IPicture_Release(newpic);
        *ppvObj = core::ptr::null_mut();
        return hr;
    }

    hr = IPicture_QueryInterface(newpic, riid, ppvObj as *mut *mut core::ffi::c_void);
    if hr != S_OK {
        ERR!(
            b"Failed to get interface %s from IPicture.\n\0".as_ptr() as *const i8,
            debugstr_guid(riid),
        );
    }
    IPicture_Release(newpic);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleLoadPictureEx(
    lpstream: LPSTREAM,
    _lSize: LONG,
    fRunmode: BOOL,
    riid: REFIID,
    _xsiz: DWORD,
    _ysiz: DWORD,
    _flags: DWORD,
    ppvObj: *mut LPVOID,
) -> HRESULT {
    let mut ps: LPPERSISTSTREAM = core::ptr::null_mut();
    let mut newpic: *mut IPicture = core::ptr::null_mut();
    let mut hr: HRESULT;

    FIXME!(
        b"(%p,%d,%d,%s,x=%d,y=%d,f=%x,%p), partially implemented.\n\0".as_ptr() as *const i8,
        lpstream,
        lSize,
        fRunmode,
        debugstr_guid(riid),
        xsiz,
        ysiz,
        flags,
        ppvObj,
    );

    hr = OleCreatePictureIndirect(
        core::ptr::null_mut(),
        riid,
        if fRunmode == 0 { 1 } else { 0 },
        core::ptr::addr_of_mut!(newpic) as *mut *mut core::ffi::c_void,
    );
    if hr != S_OK {
        return hr;
    }

    hr = IPicture_QueryInterface(
        newpic,
        core::ptr::addr_of!(IID_IPersistStream),
        core::ptr::addr_of_mut!(ps) as *mut *mut core::ffi::c_void,
    );
    if hr != S_OK {
        ERR!(b"Could not get IPersistStream iface from Ole Picture?\n\0".as_ptr() as *const i8);
        IPicture_Release(newpic);
        *ppvObj = core::ptr::null_mut();
        return hr;
    }

    hr = IPersistStream_Load(ps, lpstream);
    IPersistStream_Release(ps);
    if FAILED(hr) {
        ERR!(b"IPersistStream_Load failed\n\0".as_ptr() as *const i8);
        IPicture_Release(newpic);
        *ppvObj = core::ptr::null_mut();
        return hr;
    }

    hr = IPicture_QueryInterface(newpic, riid, ppvObj as *mut *mut core::ffi::c_void);
    if hr != S_OK {
        ERR!(
            b"Failed to get interface %s from IPicture.\n\0".as_ptr() as *const i8,
            debugstr_guid(riid),
        );
    }
    IPicture_Release(newpic);
    return hr;
}
unsafe extern "C" fn create_stream(
    filename: *const u16,
    stream: *mut *mut IStream,
) -> HRESULT {
    let mut hFile: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut dwFileSize: u32 = 0;
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut dwBytesRead: u32 = 0;
    let mut hr: i32 = 0;
    
    hFile = CreateFileW(
        filename,
        GENERIC_READ,
        0,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        0,
        core::ptr::null_mut(),
    );
    
    if hFile == INVALID_HANDLE_VALUE {
        return if GetLastError() as i32 > 0 {
            ((GetLastError() & 0xffff) | ((7 << 16) as u32) | 0x80000000) as i32
        } else {
            GetLastError() as i32
        };
    }
    
    dwFileSize = GetFileSize(hFile, core::ptr::null_mut());
    
    if dwFileSize != INVALID_FILE_SIZE {
        // C: GMEM_FIXED (0). This read HEAP_ZERO_MEMORY (0x8), which is not a GMEM_
        // flag at all: GlobalAlloc rejects it (ERROR_INVALID_PARAMETER), so every
        // file-backed OleLoadPicturePath / OleLoadPictureFile failed.
        hGlobal = GlobalAlloc(0 /* GMEM_FIXED */, dwFileSize as SIZE_T);
        if hGlobal.is_null() {
            hr = -2147024882i32;
        } else if ReadFile(
            hFile,
            hGlobal as *mut core::ffi::c_void,
            dwFileSize,
            core::ptr::addr_of_mut!(dwBytesRead),
            core::ptr::null_mut(),
        ) == 0
        {
            GlobalFree(hGlobal);
            hr = if GetLastError() as i32 > 0 {
                ((GetLastError() & 0xffff) | ((7 << 16) as u32) | 0x80000000) as i32
            } else {
                GetLastError() as i32
            };
        }
    }
    
    CloseHandle(hFile);
    
    if hr < 0 {
        return hr;
    }
    
    hr = CreateStreamOnHGlobal(hGlobal, 1, stream);
    if hr < 0 {
        GlobalFree(hGlobal);
    }
    
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleLoadPictureFile(
    filename: VARIANT,
    picture: *mut *mut IDispatch,
) -> HRESULT {
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!(
        b"(%s,%p)\n\0".as_ptr() as *const i8,
        wine_dbgstr_variant(&filename),
        picture,
    );

    if V_VT(&filename) != VT_BSTR as u16 {
        return CTL_E_FILENOTFOUND;
    }

    hr = create_stream(V_BSTR(&filename), &mut stream);
    if hr != S_OK {
        if hr == HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND) {
            return CTL_E_FILENOTFOUND;
        }
        return CTL_E_PATHFILEACCESSERROR;
    }

    hr = OleLoadPicture(
        stream as LPSTREAM,
        0,
        FALSE,
        &IID_IDispatch,
        picture as *mut LPVOID,
    );
    IStream_Release(stream);
    hr
}

#[no_mangle]
pub unsafe extern "system" fn OleSavePictureFile(
    picture: *mut IDispatch,
    filename: BSTR,
) -> HRESULT {
    
    if (*__wine_dbch___default).flags as i32
        & (1i32 << __WINE_DBCL_FIXME as i32)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0"
                .as_ptr() as *const i8,
            b"OleSavePictureFile\0".as_ptr() as *const i8,
            2490i32,
            b"(%p %s): stub\n\0".as_ptr() as *const i8,
            picture,
            debugstr_w(filename as *const u16),
        );
    }
    CTL_E_FILENOTFOUND
}

#[no_mangle]
pub unsafe extern "system" fn OleLoadPicturePath(
    szURLorPath: LPOLESTR,
    _punkCaller: LPUNKNOWN,
    _dwReserved: DWORD,
    _clrReserved: OLE_COLOR,
    riid: REFIID,
    ppvRet: *mut LPVOID,
) -> HRESULT {
    static mut file: [WCHAR; 6] = [
        'f' as i32 as WCHAR,
        'i' as i32 as WCHAR,
        'l' as i32 as WCHAR,
        'e' as i32 as WCHAR,
        ':' as i32 as WCHAR,
        0 as ::core::ffi::c_int as WCHAR,
    ];
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut hRes: HRESULT;
    let file_candidate: *mut WCHAR;
    let mut path_buf: [WCHAR; 260] = [0; 260];

    if szURLorPath.is_null() || ppvRet.is_null() {
        return E_INVALIDARG;
    }

    *ppvRet = core::ptr::null_mut();

    if wcsncmp(
        szURLorPath as *const wchar_t,
        core::ptr::addr_of!(file) as *const wchar_t,
        5,
    ) == 0
    {
        let mut size: DWORD = 0;
        hRes = CoInternetParseUrl(
            szURLorPath as LPCWSTR,
            PARSE_PATH_FROM_URL,
            0,
            path_buf.as_mut_ptr(),
            MAX_PATH as DWORD,
            core::ptr::addr_of_mut!(size),
            0,
        );
        if hRes < 0 {
            return hRes;
        }
        file_candidate = path_buf.as_mut_ptr();
    } else {
        file_candidate = szURLorPath as *mut WCHAR;
    }

    if *file_candidate.offset(1) as ::core::ffi::c_int == ':' as i32 {
        hRes = create_stream(file_candidate, core::ptr::addr_of_mut!(stream));
        if hRes < 0 {
            return INET_E_RESOURCE_NOT_FOUND;
        }
    } else {
        let mut pmnk: *mut IMoniker = core::ptr::null_mut();
        let mut pbc: *mut IBindCtx = core::ptr::null_mut();
        hRes = CreateBindCtx(0, core::ptr::addr_of_mut!(pbc));
        if hRes >= 0 {
            hRes = CreateURLMoniker(
                core::ptr::null_mut(),
                szURLorPath as LPCWSTR,
                core::ptr::addr_of_mut!(pmnk),
            );
            if hRes >= 0 {
                hRes = IMoniker_BindToStorage(
                    pmnk,
                    pbc,
                    core::ptr::null_mut(),
                    core::ptr::addr_of!(IID_IStream),
                    core::ptr::addr_of_mut!(stream) as *mut LPVOID,
                );
                IMoniker_Release(pmnk);
            }
            IBindCtx_Release(pbc);
        }
        if hRes < 0 {
            return hRes;
        }
    }

    hRes = OleLoadPicture(stream as LPSTREAM, 0, FALSE as BOOL, riid, ppvRet);

    IStream_Release(stream);

    hRes
}
unsafe extern "C" fn impl_from_IClassFactory(
    iface: *mut IClassFactory,
) -> *mut IClassFactoryImpl {
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(IClassFactoryImpl, IClassFactory_iface) as ULONG_PTR,
    ) as *mut IClassFactoryImpl;
}
stdmethod! {
fn SPCF_QueryInterface(
    iface: LPCLASSFACTORY,
    riid: *const IID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/oleaut32/olepicture.c\0"
                as *const u8 as *const i8,
            b"SPCF_QueryInterface\0" as *const u8 as *const i8,
            2579 as i32,
            b"(%p)->(%s,%p),stub!\n\0" as *const u8 as *const i8,
            This,
            debugstr_guid(riid as *const _GUID),
            ppobj,
        );
    }
    return E_NOINTERFACE;
}
}
stdmethod! {
fn SPCF_AddRef(iface: LPCLASSFACTORY) -> ULONG {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}
}
stdmethod! {
fn SPCF_Release(iface: LPCLASSFACTORY) -> ULONG {
    let This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    return InterlockedDecrement(&mut (*This).ref_) as ULONG;
}
}
stdmethod! {
fn SPCF_CreateInstance(
    iface: LPCLASSFACTORY,
    pOuter: LPUNKNOWN,
    riid: REFIID,
    ppobj: *mut LPVOID,
) -> HRESULT {
    return OleCreatePictureIndirect(
        core::ptr::null_mut(),
        riid,
        1,
        ppobj as *mut *mut core::ffi::c_void,
    );
}
}
stdmethod! {
fn SPCF_LockServer(iface: LPCLASSFACTORY, dolock: BOOL) -> HRESULT {
    let _This: *mut IClassFactoryImpl = impl_from_IClassFactory(iface as *mut IClassFactory);
    FIXME!(b"(%p)->(%d),stub!\n\0".as_ptr() as *const i8, This, dolock);
    S_OK
}
}
#[no_mangle]
pub unsafe extern "C" fn _get_STDPIC_CF(ppv: *mut LPVOID) {
    *ppv = core::ptr::addr_of_mut!(STDPIC_CF) as LPVOID;
}

// COM vtable tables — generated from olepicture.c by vtbl_gen.py
/// olepicture.c:2257 `static const IPictureVtbl OLEPictureImpl_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEPictureImpl_VTable: IPictureVtbl = IPictureVtbl {
    QueryInterface: Some(OLEPictureImpl_QueryInterface),
    AddRef: Some(OLEPictureImpl_AddRef),
    Release: Some(OLEPictureImpl_Release),
    get_Handle: Some(OLEPictureImpl_get_Handle),
    get_hPal: Some(OLEPictureImpl_get_hPal),
    get_Type: Some(OLEPictureImpl_get_Type),
    get_Width: Some(OLEPictureImpl_get_Width),
    get_Height: Some(OLEPictureImpl_get_Height),
    Render: Some(OLEPictureImpl_Render),
    set_hPal: Some(OLEPictureImpl_set_hPal),
    get_CurDC: Some(OLEPictureImpl_get_CurDC),
    SelectPicture: Some(OLEPictureImpl_SelectPicture),
    get_KeepOriginalFormat: Some(OLEPictureImpl_get_KeepOriginalFormat),
    put_KeepOriginalFormat: Some(OLEPictureImpl_put_KeepOriginalFormat),
    PictureChanged: Some(OLEPictureImpl_PictureChanged),
    SaveAsFile: Some(OLEPictureImpl_SaveAsFile),
    get_Attributes: Some(OLEPictureImpl_get_Attributes),
};

/// olepicture.c:2278 `static const IDispatchVtbl OLEPictureImpl_IDispatch_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEPictureImpl_IDispatch_VTable: IDispatchVtbl = IDispatchVtbl {
    QueryInterface: Some(OLEPictureImpl_IDispatch_QueryInterface),
    AddRef: Some(OLEPictureImpl_IDispatch_AddRef),
    Release: Some(OLEPictureImpl_IDispatch_Release),
    GetTypeInfoCount: Some(OLEPictureImpl_GetTypeInfoCount),
    GetTypeInfo: Some(OLEPictureImpl_GetTypeInfo),
    GetIDsOfNames: Some(OLEPictureImpl_GetIDsOfNames),
    Invoke: Some(OLEPictureImpl_Invoke),
};

/// olepicture.c:2289 `static const IPersistStreamVtbl OLEPictureImpl_IPersistStream_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEPictureImpl_IPersistStream_VTable: IPersistStreamVtbl = IPersistStreamVtbl {
    QueryInterface: Some(OLEPictureImpl_IPersistStream_QueryInterface),
    AddRef: Some(OLEPictureImpl_IPersistStream_AddRef),
    Release: Some(OLEPictureImpl_IPersistStream_Release),
    GetClassID: Some(OLEPictureImpl_GetClassID),
    IsDirty: Some(OLEPictureImpl_IsDirty),
    Load: Some(OLEPictureImpl_Load),
    Save: Some(OLEPictureImpl_Save),
    GetSizeMax: Some(OLEPictureImpl_GetSizeMax),
};

/// olepicture.c:2301 `static const IConnectionPointContainerVtbl OLEPictureImpl_IConnectionPointContainer_VTable`  (slot order: com-sys/bindgen_i386.rs)
pub static OLEPictureImpl_IConnectionPointContainer_VTable: IConnectionPointContainerVtbl = IConnectionPointContainerVtbl {
    QueryInterface: Some(OLEPictureImpl_IConnectionPointContainer_QueryInterface),
    AddRef: Some(OLEPictureImpl_IConnectionPointContainer_AddRef),
    Release: Some(OLEPictureImpl_IConnectionPointContainer_Release),
    EnumConnectionPoints: Some(OLEPictureImpl_EnumConnectionPoints),
    FindConnectionPoint: Some(OLEPictureImpl_FindConnectionPoint),
};

/// olepicture.c:2609 `static const IClassFactoryVtbl SPCF_Vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static SPCF_Vtbl: IClassFactoryVtbl = IClassFactoryVtbl {
    QueryInterface: Some(SPCF_QueryInterface),
    AddRef: Some(SPCF_AddRef),
    Release: Some(SPCF_Release),
    CreateInstance: Some(SPCF_CreateInstance),
    LockServer: Some(SPCF_LockServer),
};


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * OLE Picture object
//  *
//  * Implementation of OLE IPicture and related interfaces
//  *
//  * Copyright 2000 Huw D M Davies for CodeWeavers.
//  * Copyright 2001 Marcus Meissner
//  * Copyright 2008 Kirill K. Smirnov
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * BUGS
//  *
//  * Support PICTYPE_BITMAP and PICTYPE_ICON, although only bitmaps very well..
//  * Lots of methods are just stubs.
//  *
//  *
//  * NOTES (or things that msdn doesn't tell you)
//  *
//  * The width and height properties are returned in HIMETRIC units (0.01mm)
//  * IPicture::Render also uses these to select a region of the src picture.
//  * A bitmap's size is converted into these units by using the screen resolution
//  * thus an 8x8 bitmap on a 96dpi screen has a size of 212x212 (8/96 * 2540).
//  *
//  */
//
// #include <stdarg.h>
// #include <stdio.h>
// #include <string.h>
//
// #define COBJMACROS
// #define NONAMELESSUNION
//
// #include "winerror.h"
// #include "windef.h"
// #include "winbase.h"
// #include "wingdi.h"
// #include "winuser.h"
// #include "ole2.h"
// #include "olectl.h"
// #include "oleauto.h"
// #include "connpt.h"
// #include "urlmon.h"
// #include "initguid.h"
// #include "wincodec.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(olepicture);
//
// #define BITMAP_FORMAT_BMP   0x4d42 /* "BM" */
// #define BITMAP_FORMAT_JPEG  0xd8ff
// #define BITMAP_FORMAT_GIF   0x4947
// #define BITMAP_FORMAT_PNG   0x5089
// #define BITMAP_FORMAT_APM   0xcdd7
//
// #include "pshpack1.h"
//
// /* Header for Aldus Placable Metafiles - a standard metafile follows */
// typedef struct _APM_HEADER
// {
//     DWORD key;
//     WORD handle;
//     SHORT left;
//     SHORT top;
//     SHORT right;
//     SHORT bottom;
//     WORD inch;
//     DWORD reserved;
//     WORD checksum;
// } APM_HEADER;
//
// typedef struct {
//     BYTE bWidth;
//     BYTE bHeight;
//     BYTE bColorCount;
//     BYTE bReserved;
//     WORD xHotspot;
//     WORD yHotspot;
//     DWORD dwDIBSize;
//     DWORD dwDIBOffset;
// } CURSORICONFILEDIRENTRY;
//
// typedef struct
// {
//     WORD                idReserved;
//     WORD                idType;
//     WORD                idCount;
//     CURSORICONFILEDIRENTRY  idEntries[1];
// } CURSORICONFILEDIR;
//
// #include "poppack.h"
//
// /*************************************************************************
//  *  Declaration of implementation class
//  */
//
// typedef struct OLEPictureImpl {
//
//   /*
//    * IPicture handles IUnknown
//    */
//
//     IPicture                  IPicture_iface;
//     IDispatch                 IDispatch_iface;
//     IPersistStream            IPersistStream_iface;
//     IConnectionPointContainer IConnectionPointContainer_iface;
//
//   /* Object reference count */
//     LONG ref;
//
//   /* We own the object and must destroy it ourselves */
//     BOOL fOwn;
//
//   /* Picture description */
//     PICTDESC desc;
//
//   /* These are the pixel size of a bitmap */
//     DWORD origWidth;
//     DWORD origHeight;
//
//   /* And these are the size of the picture converted into HIMETRIC units */
//     OLE_XSIZE_HIMETRIC himetricWidth;
//     OLE_YSIZE_HIMETRIC himetricHeight;
//
//     IConnectionPoint *pCP;
//
//     BOOL keepOrigFormat;
//     HDC	hDCCur;
//     HBITMAP stock_bitmap;
//
//   /* Bitmap transparency mask */
//     HBITMAP hbmMask;
//     HBITMAP hbmXor;
//     COLORREF rgbTrans;
//
//   /* data */
//     void* data;
//     int datalen;
//     BOOL bIsDirty;                  /* Set to TRUE if picture has changed */
//     unsigned int loadtime_magic;    /* If a length header was found, saves value */
//     unsigned int loadtime_format;   /* for PICTYPE_BITMAP only, keeps track of image format (GIF/BMP/JPEG) */
// } OLEPictureImpl;
//
// static inline OLEPictureImpl *impl_from_IPicture(IPicture *iface)
// {
//     return CONTAINING_RECORD(iface, OLEPictureImpl, IPicture_iface);
// }
//
// static inline OLEPictureImpl *impl_from_IDispatch( IDispatch *iface )
// {
//     return CONTAINING_RECORD(iface, OLEPictureImpl, IDispatch_iface);
// }
//
// static inline OLEPictureImpl *impl_from_IPersistStream( IPersistStream *iface )
// {
//     return CONTAINING_RECORD(iface, OLEPictureImpl, IPersistStream_iface);
// }
//
// static inline OLEPictureImpl *impl_from_IConnectionPointContainer( IConnectionPointContainer *iface )
// {
//     return CONTAINING_RECORD(iface, OLEPictureImpl, IConnectionPointContainer_iface);
// }
//
// /*
//  * Predeclare VTables.  They get initialized at the end.
//  */
// static const IPictureVtbl OLEPictureImpl_VTable;
// static const IDispatchVtbl OLEPictureImpl_IDispatch_VTable;
// static const IPersistStreamVtbl OLEPictureImpl_IPersistStream_VTable;
// static const IConnectionPointContainerVtbl OLEPictureImpl_IConnectionPointContainer_VTable;
//
// /* pixels to HIMETRIC units conversion */
// static inline OLE_XSIZE_HIMETRIC xpixels_to_himetric(INT pixels, HDC hdc)
// {
//     return MulDiv(pixels, 2540, GetDeviceCaps(hdc, LOGPIXELSX));
// }
//
// static inline OLE_YSIZE_HIMETRIC ypixels_to_himetric(INT pixels, HDC hdc)
// {
//     return MulDiv(pixels, 2540, GetDeviceCaps(hdc, LOGPIXELSY));
// }
//
// /***********************************************************************
//  * Implementation of the OLEPictureImpl class.
//  */
//
// static void OLEPictureImpl_SetBitmap(OLEPictureImpl *This)
// {
//   BITMAP bm;
//   HDC hdcRef;
//
//   TRACE("bitmap handle %p\n", This->desc.u.bmp.hbitmap);
//   if(GetObjectW(This->desc.u.bmp.hbitmap, sizeof(bm), &bm) != sizeof(bm)) {
//     ERR("GetObject fails\n");
//     return;
//   }
//   This->origWidth = bm.bmWidth;
//   This->origHeight = bm.bmHeight;
//
//   TRACE("width %d, height %d, bpp %d\n", bm.bmWidth, bm.bmHeight, bm.bmBitsPixel);
//
//   /* The width and height are stored in HIMETRIC units (0.01 mm),
//      so we take our pixel width divide by pixels per inch and
//      multiply by 25.4 * 100 */
//   /* Should we use GetBitmapDimension if available? */
//   hdcRef = CreateCompatibleDC(0);
//
//   This->himetricWidth  = xpixels_to_himetric(bm.bmWidth, hdcRef);
//   This->himetricHeight = ypixels_to_himetric(bm.bmHeight, hdcRef);
//   This->stock_bitmap = GetCurrentObject( hdcRef, OBJ_BITMAP );
//
//   This->loadtime_format = BITMAP_FORMAT_BMP;
//
//   DeleteDC(hdcRef);
// }
//
// static void OLEPictureImpl_SetIcon(OLEPictureImpl * This)
// {
//     ICONINFO infoIcon;
//
//     TRACE("icon handle %p\n", This->desc.u.icon.hicon);
//     if (GetIconInfo(This->desc.u.icon.hicon, &infoIcon)) {
//         HDC hdcRef;
//         BITMAP bm;
//
//         TRACE("bitmap handle for icon is %p\n", infoIcon.hbmColor);
//         if(GetObjectW(infoIcon.hbmColor ? infoIcon.hbmColor : infoIcon.hbmMask, sizeof(bm), &bm) != sizeof(bm)) {
//             ERR("GetObject fails on icon bitmap\n");
//             return;
//         }
//
//         This->origWidth = bm.bmWidth;
//         This->origHeight = infoIcon.hbmColor ? bm.bmHeight : bm.bmHeight / 2;
//         /* see comment on HIMETRIC on OLEPictureImpl_SetBitmap() */
//         hdcRef = GetDC(0);
//
//         This->himetricWidth  = xpixels_to_himetric(This->origWidth, hdcRef);
//         This->himetricHeight = ypixels_to_himetric(This->origHeight, hdcRef);
//
//         ReleaseDC(0, hdcRef);
//
//         DeleteObject(infoIcon.hbmMask);
//         if (infoIcon.hbmColor) DeleteObject(infoIcon.hbmColor);
//     } else {
//         ERR("GetIconInfo() fails on icon %p\n", This->desc.u.icon.hicon);
//     }
// }
//
// static void OLEPictureImpl_SetEMF(OLEPictureImpl *This)
// {
//     ENHMETAHEADER emh;
//
//     GetEnhMetaFileHeader(This->desc.u.emf.hemf, sizeof(emh), &emh);
//
//     This->origWidth = 0;
//     This->origHeight = 0;
//     This->himetricWidth = emh.rclFrame.right - emh.rclFrame.left;
//     This->himetricHeight = emh.rclFrame.bottom - emh.rclFrame.top;
// }
//
// /************************************************************************
//  * OLEPictureImpl_Construct
//  *
//  * This method will construct a new instance of the OLEPictureImpl
//  * class.
//  *
//  * The caller of this method must release the object when it's
//  * done with it.
//  */
// static HRESULT OLEPictureImpl_Construct(LPPICTDESC pictDesc, BOOL fOwn, OLEPictureImpl **pict)
// {
//   OLEPictureImpl *newObject;
//   HRESULT hr;
//
//   if (pictDesc)
//       TRACE("(%p) type = %d\n", pictDesc, pictDesc->picType);
//
//   /*
//    * Allocate space for the object.
//    */
//   newObject = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(OLEPictureImpl));
//   if (!newObject)
//     return E_OUTOFMEMORY;
//
//   /*
//    * Initialize the virtual function table.
//    */
//   newObject->IPicture_iface.lpVtbl = &OLEPictureImpl_VTable;
//   newObject->IDispatch_iface.lpVtbl = &OLEPictureImpl_IDispatch_VTable;
//   newObject->IPersistStream_iface.lpVtbl = &OLEPictureImpl_IPersistStream_VTable;
//   newObject->IConnectionPointContainer_iface.lpVtbl = &OLEPictureImpl_IConnectionPointContainer_VTable;
//
//   newObject->pCP = NULL;
//   hr = CreateConnectionPoint((IUnknown*)&newObject->IPicture_iface, &IID_IPropertyNotifySink,
//                         &newObject->pCP);
//   if (hr != S_OK)
//   {
//     HeapFree(GetProcessHeap(), 0, newObject);
//     return hr;
//   }
//
//   /*
//    * Start with one reference count. The caller of this function
//    * must release the interface pointer when it is done.
//    */
//   newObject->ref	= 1;
//   newObject->hDCCur	= 0;
//
//   newObject->fOwn	= fOwn;
//
//   /* dunno about original value */
//   newObject->keepOrigFormat = TRUE;
//
//   newObject->hbmMask = NULL;
//   newObject->hbmXor = NULL;
//   newObject->loadtime_magic = 0xdeadbeef;
//   newObject->loadtime_format = 0;
//   newObject->bIsDirty = FALSE;
//
//   if (pictDesc) {
//       newObject->desc = *pictDesc;
//
//       switch(pictDesc->picType) {
//       case PICTYPE_BITMAP:
// 	OLEPictureImpl_SetBitmap(newObject);
// 	break;
//
//       case PICTYPE_METAFILE:
// 	TRACE("metafile handle %p\n", pictDesc->u.wmf.hmeta);
// 	newObject->himetricWidth = pictDesc->u.wmf.xExt;
// 	newObject->himetricHeight = pictDesc->u.wmf.yExt;
// 	break;
//
//       case PICTYPE_NONE:
// 	/* not sure what to do here */
// 	newObject->himetricWidth = newObject->himetricHeight = 0;
// 	break;
//
//       case PICTYPE_ICON:
//         OLEPictureImpl_SetIcon(newObject);
//         break;
//
//       case PICTYPE_ENHMETAFILE:
//         OLEPictureImpl_SetEMF(newObject);
//         break;
//
//       default:
//         WARN("Unsupported type %d\n", pictDesc->picType);
//         IPicture_Release(&newObject->IPicture_iface);
//         return E_UNEXPECTED;
//       }
//   } else {
//       newObject->desc.picType = PICTYPE_UNINITIALIZED;
//   }
//
//   TRACE("returning %p\n", newObject);
//   *pict = newObject;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_Destroy
//  *
//  * This method is called by the Release method when the reference
//  * count goes down to 0. It will free all resources used by
//  * this object.  */
// static void OLEPictureImpl_Destroy(OLEPictureImpl* Obj)
// {
//   TRACE("(%p)\n", Obj);
//
//   if (Obj->pCP)
//     IConnectionPoint_Release(Obj->pCP);
//
//   if(Obj->fOwn) { /* We need to destroy the picture */
//     switch(Obj->desc.picType) {
//     case PICTYPE_BITMAP:
//       DeleteObject(Obj->desc.u.bmp.hbitmap);
//       if (Obj->hbmMask != NULL) DeleteObject(Obj->hbmMask);
//       if (Obj->hbmXor != NULL) DeleteObject(Obj->hbmXor);
//       break;
//     case PICTYPE_METAFILE:
//       DeleteMetaFile(Obj->desc.u.wmf.hmeta);
//       break;
//     case PICTYPE_ICON:
//       DestroyIcon(Obj->desc.u.icon.hicon);
//       break;
//     case PICTYPE_ENHMETAFILE:
//       DeleteEnhMetaFile(Obj->desc.u.emf.hemf);
//       break;
//     case PICTYPE_NONE:
//     case PICTYPE_UNINITIALIZED:
//       /* Nothing to do */
//       break;
//     default:
//       FIXME("Unsupported type %d - unable to delete\n", Obj->desc.picType);
//       break;
//     }
//   }
//   HeapFree(GetProcessHeap(), 0, Obj->data);
//   HeapFree(GetProcessHeap(), 0, Obj);
// }
//
//
// /************************************************************************
//  * OLEPictureImpl_AddRef (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_AddRef(
//   IPicture* iface)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   ULONG refCount = InterlockedIncrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount - 1);
//
//   return refCount;
// }
//
// /************************************************************************
//  * OLEPictureImpl_Release (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_Release(
//       IPicture* iface)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   ULONG refCount = InterlockedDecrement(&This->ref);
//
//   TRACE("(%p)->(ref before=%d)\n", This, refCount + 1);
//
//   /*
//    * If the reference count goes down to 0, perform suicide.
//    */
//   if (!refCount) OLEPictureImpl_Destroy(This);
//
//   return refCount;
// }
//
// /************************************************************************
//  * OLEPictureImpl_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_QueryInterface(
//   IPicture*  iface,
//   REFIID  riid,
//   void**  ppvObject)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//
//   TRACE("(%p)->(%s, %p)\n", This, debugstr_guid(riid), ppvObject);
//
//   if (!ppvObject)
//     return E_INVALIDARG;
//
//   *ppvObject = 0;
//
//   if (IsEqualIID(&IID_IUnknown, riid) || IsEqualIID(&IID_IPicture, riid))
//     *ppvObject = &This->IPicture_iface;
//   else if (IsEqualIID(&IID_IDispatch, riid))
//     *ppvObject = &This->IDispatch_iface;
//   else if (IsEqualIID(&IID_IPictureDisp, riid))
//     *ppvObject = &This->IDispatch_iface;
//   else if (IsEqualIID(&IID_IPersist, riid) || IsEqualIID(&IID_IPersistStream, riid))
//     *ppvObject = &This->IPersistStream_iface;
//   else if (IsEqualIID(&IID_IConnectionPointContainer, riid))
//     *ppvObject = &This->IConnectionPointContainer_iface;
//
//   if (!*ppvObject)
//   {
//     FIXME("() : asking for unsupported interface %s\n",debugstr_guid(riid));
//     return E_NOINTERFACE;
//   }
//
//   IPicture_AddRef(iface);
//
//   return S_OK;
// }
//
// /***********************************************************************
//  *    OLEPicture_SendNotify (internal)
//  *
//  * Sends notification messages of changed properties to any interested
//  * connections.
//  */
// static void OLEPicture_SendNotify(OLEPictureImpl* this, DISPID dispID)
// {
//   IEnumConnections *pEnum;
//   CONNECTDATA CD;
//
//   if (IConnectionPoint_EnumConnections(this->pCP, &pEnum) != S_OK)
//       return;
//   while(IEnumConnections_Next(pEnum, 1, &CD, NULL) == S_OK) {
//     IPropertyNotifySink *sink;
//
//     IUnknown_QueryInterface(CD.pUnk, &IID_IPropertyNotifySink, (LPVOID)&sink);
//     IPropertyNotifySink_OnChanged(sink, dispID);
//     IPropertyNotifySink_Release(sink);
//     IUnknown_Release(CD.pUnk);
//   }
//   IEnumConnections_Release(pEnum);
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_Handle
//  */
// static HRESULT WINAPI OLEPictureImpl_get_Handle(IPicture *iface,
// 						OLE_HANDLE *phandle)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p)\n", This, phandle);
//
//   if(!phandle)
//     return E_POINTER;
//
//   switch(This->desc.picType) {
//   case PICTYPE_NONE:
//   case PICTYPE_UNINITIALIZED:
//     *phandle = 0;
//     break;
//   case PICTYPE_BITMAP:
//     *phandle = HandleToUlong(This->desc.u.bmp.hbitmap);
//     break;
//   case PICTYPE_METAFILE:
//     *phandle = HandleToUlong(This->desc.u.wmf.hmeta);
//     break;
//   case PICTYPE_ICON:
//     *phandle = HandleToUlong(This->desc.u.icon.hicon);
//     break;
//   case PICTYPE_ENHMETAFILE:
//     *phandle = HandleToUlong(This->desc.u.emf.hemf);
//     break;
//   default:
//     FIXME("Unimplemented type %d\n", This->desc.picType);
//     return E_NOTIMPL;
//   }
//   TRACE("returning handle %08x\n", *phandle);
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_hPal
//  */
// static HRESULT WINAPI OLEPictureImpl_get_hPal(IPicture *iface,
// 					      OLE_HANDLE *phandle)
// {
//     OLEPictureImpl *This = impl_from_IPicture(iface);
//
//     TRACE("(%p)->(%p)\n", This, phandle);
//
//     if (!phandle) return E_POINTER;
//
//     if (This->desc.picType == PICTYPE_BITMAP)
//     {
//         *phandle = HandleToUlong(This->desc.u.bmp.hpal);
//         return S_OK;
//     }
//
//     return E_FAIL;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_Type
//  */
// static HRESULT WINAPI OLEPictureImpl_get_Type(IPicture *iface,
// 					      short *ptype)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p): type is %d\n", This, ptype, This->desc.picType);
//
//   if(!ptype)
//     return E_POINTER;
//
//   *ptype = This->desc.picType;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_Width
//  */
// static HRESULT WINAPI OLEPictureImpl_get_Width(IPicture *iface,
// 					       OLE_XSIZE_HIMETRIC *pwidth)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p): width is %d\n", This, pwidth, This->himetricWidth);
//   *pwidth = This->himetricWidth;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_Height
//  */
// static HRESULT WINAPI OLEPictureImpl_get_Height(IPicture *iface,
// 						OLE_YSIZE_HIMETRIC *pheight)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p): height is %d\n", This, pheight, This->himetricHeight);
//   *pheight = This->himetricHeight;
//   return S_OK;
// }
//
// static void render_masked_bitmap(OLEPictureImpl *This, HDC hdc,
//     LONG x, LONG y, LONG cx, LONG cy, OLE_XPOS_HIMETRIC xSrc, OLE_YPOS_HIMETRIC ySrc,
//     OLE_XSIZE_HIMETRIC cxSrc, OLE_YSIZE_HIMETRIC cySrc, HBITMAP hbmMask, HBITMAP hbmXor)
// {
//     HDC hdcBmp;
//
//     /* Set a mapping mode that maps bitmap pixels into HIMETRIC units.
//      * NB y-axis gets flipped
//      */
//
//     hdcBmp = CreateCompatibleDC(0);
//     SetMapMode(hdcBmp, MM_ANISOTROPIC);
//     SetWindowOrgEx(hdcBmp, 0, 0, NULL);
//     SetWindowExtEx(hdcBmp, This->himetricWidth, This->himetricHeight, NULL);
//     SetViewportOrgEx(hdcBmp, 0, This->origHeight, NULL);
//     SetViewportExtEx(hdcBmp, This->origWidth, -This->origHeight, NULL);
//
//     if (hbmMask)
//     {
//         SetBkColor(hdc, RGB(255, 255, 255));
//         SetTextColor(hdc, RGB(0, 0, 0));
//
//         SelectObject(hdcBmp, hbmMask);
//         StretchBlt(hdc, x, y, cx, cy, hdcBmp, xSrc, ySrc, cxSrc, cySrc, SRCAND);
//
//         if (hbmXor)
//         {
//             SelectObject(hdcBmp, hbmXor);
//             StretchBlt(hdc, x, y, cx, cy, hdcBmp, xSrc, ySrc, cxSrc, cySrc, SRCPAINT);
//         }
//         else StretchBlt(hdc, x, y, cx, cy, hdcBmp, xSrc, ySrc - This->himetricHeight,
//                         cxSrc, cySrc, SRCPAINT);
//     }
//     else
//     {
//         SelectObject(hdcBmp, hbmXor);
//         StretchBlt(hdc, x, y, cx, cy, hdcBmp, xSrc, ySrc, cxSrc, cySrc, SRCCOPY);
//     }
//
//     DeleteDC(hdcBmp);
// }
//
// /************************************************************************
//  * OLEPictureImpl_Render
//  */
// static HRESULT WINAPI OLEPictureImpl_Render(IPicture *iface, HDC hdc,
// 					    LONG x, LONG y, LONG cx, LONG cy,
// 					    OLE_XPOS_HIMETRIC xSrc,
// 					    OLE_YPOS_HIMETRIC ySrc,
// 					    OLE_XSIZE_HIMETRIC cxSrc,
// 					    OLE_YSIZE_HIMETRIC cySrc,
// 					    LPCRECT prcWBounds)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p, (%d,%d), (%d,%d) <- (%d,%d), (%d,%d), %p)\n",
// 	This, hdc, x, y, cx, cy, xSrc, ySrc, cxSrc, cySrc, prcWBounds);
//   if(prcWBounds)
//   TRACE("prcWBounds %s\n", wine_dbgstr_rect(prcWBounds));
//
//   if(cx == 0 || cy == 0 || cxSrc == 0 || cySrc == 0){
//     return CTL_E_INVALIDPROPERTYVALUE;
//   }
//
//   /*
//    * While the documentation suggests this to be here (or after rendering?)
//    * it does cause an endless recursion in my sample app. -MM 20010804
//   OLEPicture_SendNotify(This,DISPID_PICT_RENDER);
//    */
//
//   switch(This->desc.picType) {
//   case PICTYPE_UNINITIALIZED:
//   case PICTYPE_NONE:
//     /* nothing to do */
//     return S_OK;
//   case PICTYPE_BITMAP:
//   {
//     HBITMAP hbmMask, hbmXor;
//
//     if (This->hbmMask)
//     {
//         hbmMask = This->hbmMask;
//         hbmXor = This->hbmXor;
//     }
//     else
//     {
//         hbmMask = 0;
//         hbmXor = This->desc.u.bmp.hbitmap;
//     }
//
//     render_masked_bitmap(This, hdc, x, y, cx, cy, xSrc, ySrc, cxSrc, cySrc, hbmMask, hbmXor);
//     break;
//   }
//
//   case PICTYPE_ICON:
//   {
//     ICONINFO info;
//
//     if (!GetIconInfo(This->desc.u.icon.hicon, &info))
//         return E_FAIL;
//
//     render_masked_bitmap(This, hdc, x, y, cx, cy, xSrc, ySrc, cxSrc, cySrc, info.hbmMask, info.hbmColor);
//
//     DeleteObject(info.hbmMask);
//     if (info.hbmColor) DeleteObject(info.hbmColor);
//     break;
//   }
//
//   case PICTYPE_METAFILE:
//   {
//     POINT prevOrg, prevWndOrg;
//     SIZE prevExt, prevWndExt;
//     int oldmode;
//
//     /* Render the WMF to the appropriate location by setting the
//        appropriate ratio between "device units" and "logical units" */
//     oldmode = SetMapMode(hdc, MM_ANISOTROPIC);
//     /* For the "source rectangle" the y-axis must be inverted */
//     SetWindowOrgEx(hdc, xSrc, This->himetricHeight-ySrc, &prevWndOrg);
//     SetWindowExtEx(hdc, cxSrc, -cySrc, &prevWndExt);
//     /* For the "destination rectangle" no inversion is necessary */
//     SetViewportOrgEx(hdc, x, y, &prevOrg);
//     SetViewportExtEx(hdc, cx, cy, &prevExt);
//
//     if (!PlayMetaFile(hdc, This->desc.u.wmf.hmeta))
//         ERR("PlayMetaFile failed!\n");
//
//     /* We're done, restore the DC to the previous settings for converting
//        logical units to device units */
//     SetWindowExtEx(hdc, prevWndExt.cx, prevWndExt.cy, NULL);
//     SetWindowOrgEx(hdc, prevWndOrg.x, prevWndOrg.y, NULL);
//     SetViewportExtEx(hdc, prevExt.cx, prevExt.cy, NULL);
//     SetViewportOrgEx(hdc, prevOrg.x, prevOrg.y, NULL);
//     SetMapMode(hdc, oldmode);
//     break;
//   }
//
//   case PICTYPE_ENHMETAFILE:
//   {
//     RECT rc = { x, y, x + cx, y + cy };
//     PlayEnhMetaFile(hdc, This->desc.u.emf.hemf, &rc);
//     break;
//   }
//
//   default:
//     FIXME("type %d not implemented\n", This->desc.picType);
//     return E_NOTIMPL;
//   }
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_set_hPal
//  */
// static HRESULT WINAPI OLEPictureImpl_set_hPal(IPicture *iface,
// 					      OLE_HANDLE hpal)
// {
//     OLEPictureImpl *This = impl_from_IPicture(iface);
//
//     TRACE("(%p)->(%08x)\n", This, hpal);
//
//     if (This->desc.picType == PICTYPE_BITMAP)
//     {
//         This->desc.u.bmp.hpal = ULongToHandle(hpal);
//         OLEPicture_SendNotify(This,DISPID_PICT_HPAL);
//         return S_OK;
//     }
//
//     return E_FAIL;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_CurDC
//  */
// static HRESULT WINAPI OLEPictureImpl_get_CurDC(IPicture *iface,
// 					       HDC *phdc)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p), returning %p\n", This, This->hDCCur);
//   if (phdc) *phdc = This->hDCCur;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_SelectPicture
//  */
// static HRESULT WINAPI OLEPictureImpl_SelectPicture(IPicture *iface,
// 						   HDC hdcIn,
// 						   HDC *phdcOut,
// 						   OLE_HANDLE *phbmpOut)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p, %p, %p)\n", This, hdcIn, phdcOut, phbmpOut);
//   if (This->desc.picType == PICTYPE_BITMAP) {
//       if (phdcOut)
// 	  *phdcOut = This->hDCCur;
//       if (This->hDCCur) SelectObject(This->hDCCur,This->stock_bitmap);
//       if (hdcIn) SelectObject(hdcIn,This->desc.u.bmp.hbitmap);
//       This->hDCCur = hdcIn;
//       if (phbmpOut)
// 	  *phbmpOut = HandleToUlong(This->desc.u.bmp.hbitmap);
//       return S_OK;
//   } else {
//       FIXME("Don't know how to select picture type %d\n",This->desc.picType);
//       return E_FAIL;
//   }
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_KeepOriginalFormat
//  */
// static HRESULT WINAPI OLEPictureImpl_get_KeepOriginalFormat(IPicture *iface,
// 							    BOOL *pfKeep)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p)\n", This, pfKeep);
//   if (!pfKeep)
//       return E_POINTER;
//   *pfKeep = This->keepOrigFormat;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_put_KeepOriginalFormat
//  */
// static HRESULT WINAPI OLEPictureImpl_put_KeepOriginalFormat(IPicture *iface,
// 							    BOOL keep)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%d)\n", This, keep);
//   This->keepOrigFormat = keep;
//   /* FIXME: what DISPID notification here? */
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_PictureChanged
//  */
// static HRESULT WINAPI OLEPictureImpl_PictureChanged(IPicture *iface)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->()\n", This);
//   OLEPicture_SendNotify(This,DISPID_PICT_HANDLE);
//   This->bIsDirty = TRUE;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_get_Attributes
//  */
// static HRESULT WINAPI OLEPictureImpl_get_Attributes(IPicture *iface,
// 						    DWORD *pdwAttr)
// {
//   OLEPictureImpl *This = impl_from_IPicture(iface);
//   TRACE("(%p)->(%p).\n", This, pdwAttr);
//
//   if(!pdwAttr)
//     return E_POINTER;
//
//   *pdwAttr = 0;
//   switch (This->desc.picType) {
//   case PICTYPE_UNINITIALIZED:
//   case PICTYPE_NONE: break;
//   case PICTYPE_BITMAP: 	if (This->hbmMask) *pdwAttr = PICTURE_TRANSPARENT; break;	/* not 'truly' scalable, see MSDN. */
//   case PICTYPE_ICON: *pdwAttr     = PICTURE_TRANSPARENT;break;
//   case PICTYPE_ENHMETAFILE: /* fall through */
//   case PICTYPE_METAFILE: *pdwAttr = PICTURE_TRANSPARENT|PICTURE_SCALABLE;break;
//   default:FIXME("Unknown pictype %d\n",This->desc.picType);break;
//   }
//   return S_OK;
// }
//
//
// /************************************************************************
//  *    IConnectionPointContainer
//  */
// static HRESULT WINAPI OLEPictureImpl_IConnectionPointContainer_QueryInterface(
//   IConnectionPointContainer* iface,
//   REFIID riid,
//   VOID** ppvoid)
// {
//   OLEPictureImpl *This = impl_from_IConnectionPointContainer(iface);
//
//   return IPicture_QueryInterface(&This->IPicture_iface,riid,ppvoid);
// }
//
// static ULONG WINAPI OLEPictureImpl_IConnectionPointContainer_AddRef(
//   IConnectionPointContainer* iface)
// {
//   OLEPictureImpl *This = impl_from_IConnectionPointContainer(iface);
//
//   return IPicture_AddRef(&This->IPicture_iface);
// }
//
// static ULONG WINAPI OLEPictureImpl_IConnectionPointContainer_Release(
//   IConnectionPointContainer* iface)
// {
//   OLEPictureImpl *This = impl_from_IConnectionPointContainer(iface);
//
//   return IPicture_Release(&This->IPicture_iface);
// }
//
// static HRESULT WINAPI OLEPictureImpl_EnumConnectionPoints(
//   IConnectionPointContainer* iface,
//   IEnumConnectionPoints** ppEnum)
// {
//   OLEPictureImpl *This = impl_from_IConnectionPointContainer(iface);
//
//   FIXME("(%p,%p), stub!\n",This,ppEnum);
//   return E_NOTIMPL;
// }
//
// static HRESULT WINAPI OLEPictureImpl_FindConnectionPoint(
//   IConnectionPointContainer* iface,
//   REFIID riid,
//   IConnectionPoint **ppCP)
// {
//   OLEPictureImpl *This = impl_from_IConnectionPointContainer(iface);
//   TRACE("(%p,%s,%p)\n",This,debugstr_guid(riid),ppCP);
//   if (!ppCP)
//       return E_POINTER;
//   *ppCP = NULL;
//   if (IsEqualGUID(riid,&IID_IPropertyNotifySink))
//       return IConnectionPoint_QueryInterface(This->pCP, &IID_IConnectionPoint, (void**)ppCP);
//   FIXME("no connection point for %s\n",debugstr_guid(riid));
//   return CONNECT_E_NOCONNECTION;
// }
//
//
// /************************************************************************
//  *    IPersistStream
//  */
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_IPersistStream_QueryInterface(
//   IPersistStream* iface,
//   REFIID     riid,
//   VOID**     ppvoid)
// {
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//
//   return IPicture_QueryInterface(&This->IPicture_iface, riid, ppvoid);
// }
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_AddRef (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_IPersistStream_AddRef(
//   IPersistStream* iface)
// {
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//
//   return IPicture_AddRef(&This->IPicture_iface);
// }
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_Release (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_IPersistStream_Release(
//   IPersistStream* iface)
// {
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//
//   return IPicture_Release(&This->IPicture_iface);
// }
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_GetClassID
//  */
// static HRESULT WINAPI OLEPictureImpl_GetClassID(
//   IPersistStream* iface,CLSID* pClassID)
// {
//   TRACE("(%p)\n", pClassID);
//   *pClassID = CLSID_StdPicture;
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_IsDirty
//  */
// static HRESULT WINAPI OLEPictureImpl_IsDirty(
//   IPersistStream* iface)
// {
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//   FIXME("(%p),stub!\n",This);
//   return E_NOTIMPL;
// }
//
// static HRESULT OLEPictureImpl_LoadWICSource(OLEPictureImpl *This, IWICBitmapSource *src)
// {
//     HRESULT hr;
//     BITMAPINFOHEADER bih;
//     UINT width, height;
//     UINT stride, buffersize;
//     BYTE *bits, *mask = NULL;
//     WICRect rc;
//     IWICBitmapSource *real_source;
//     UINT x, y;
//     COLORREF white = RGB(255, 255, 255), black = RGB(0, 0, 0);
//     BOOL has_alpha=FALSE;
//
//     hr = WICConvertBitmapSource(&GUID_WICPixelFormat32bppBGRA, src, &real_source);
//     if (FAILED(hr)) return hr;
//
//     hr = IWICBitmapSource_GetSize(real_source, &width, &height);
//     if (FAILED(hr)) goto end;
//
//     bih.biSize = sizeof(bih);
//     bih.biWidth = width;
//     bih.biHeight = -height;
//     bih.biPlanes = 1;
//     bih.biBitCount = 32;
//     bih.biCompression = BI_RGB;
//     bih.biSizeImage = 0;
//     bih.biXPelsPerMeter = 4085; /* olepicture ignores the stored resolution */
//     bih.biYPelsPerMeter = 4085;
//     bih.biClrUsed = 0;
//     bih.biClrImportant = 0;
//
//     stride = 4 * width;
//     buffersize = stride * height;
//
//     mask = HeapAlloc(GetProcessHeap(), 0, buffersize);
//     if (!mask)
//     {
//         hr = E_OUTOFMEMORY;
//         goto end;
//     }
//
//     This->desc.u.bmp.hbitmap = CreateDIBSection(0, (BITMAPINFO*)&bih, DIB_RGB_COLORS, (void **)&bits, NULL, 0);
//     if (This->desc.u.bmp.hbitmap == 0)
//     {
//         hr = E_FAIL;
//         goto end;
//     }
//
//     rc.X = 0;
//     rc.Y = 0;
//     rc.Width = width;
//     rc.Height = height;
//     hr = IWICBitmapSource_CopyPixels(real_source, &rc, stride, buffersize, bits);
//     if (FAILED(hr))
//     {
//         DeleteObject(This->desc.u.bmp.hbitmap);
//         goto end;
//     }
//
//     This->desc.picType = PICTYPE_BITMAP;
//     OLEPictureImpl_SetBitmap(This);
//
//     /* set transparent pixels to black, all others to white */
//     for(y = 0; y < height; y++){
//         for(x = 0; x < width; x++){
//             DWORD *pixel = (DWORD*)(bits + stride*y + 4*x);
//             if((*pixel & 0x80000000) == 0)
//             {
//                 has_alpha = TRUE;
//                 *(DWORD *)(mask + stride * y + 4 * x) = black;
//             }
//             else
//                 *(DWORD *)(mask + stride * y + 4 * x) = white;
//         }
//     }
//
//     if (has_alpha)
//     {
//         HDC hdcref, hdcBmp, hdcXor, hdcMask;
//         HBITMAP hbmoldBmp, hbmoldXor, hbmoldMask;
//
//         hdcref = GetDC(0);
//
//         This->hbmXor = CreateDIBitmap(
//             hdcref,
//             &bih,
//             CBM_INIT,
//             mask,
//             (BITMAPINFO*)&bih,
//             DIB_RGB_COLORS
//         );
//
//         This->hbmMask = CreateBitmap(width,-height,1,1,NULL);
//         hdcBmp = CreateCompatibleDC(NULL);
//         hdcXor = CreateCompatibleDC(NULL);
//         hdcMask = CreateCompatibleDC(NULL);
//
//         hbmoldBmp = SelectObject(hdcBmp,This->desc.u.bmp.hbitmap);
//         hbmoldXor = SelectObject(hdcXor,This->hbmXor);
//         hbmoldMask = SelectObject(hdcMask,This->hbmMask);
//
//         SetBkColor(hdcXor,black);
//         BitBlt(hdcMask,0,0,width,height,hdcXor,0,0,SRCCOPY);
//         BitBlt(hdcXor,0,0,width,height,hdcBmp,0,0,SRCAND);
//
//         SelectObject(hdcBmp,hbmoldBmp);
//         SelectObject(hdcXor,hbmoldXor);
//         SelectObject(hdcMask,hbmoldMask);
//
//         DeleteDC(hdcBmp);
//         DeleteDC(hdcXor);
//         DeleteDC(hdcMask);
//         ReleaseDC(0, hdcref);
//     }
//
// end:
//     HeapFree(GetProcessHeap(), 0, mask);
//     IWICBitmapSource_Release(real_source);
//     return hr;
// }
//
// static HRESULT OLEPictureImpl_LoadWICDecoder(OLEPictureImpl *This, REFCLSID decoder_clsid, BYTE *xbuf, ULONG xread)
// {
//     HRESULT hr;
//     IWICImagingFactory *factory;
//     IWICBitmapDecoder *decoder;
//     IWICBitmapFrameDecode *framedecode;
//     HRESULT initresult;
//     IWICStream *stream;
//
//     initresult = CoInitialize(NULL);
//
//     hr = CoCreateInstance(&CLSID_WICImagingFactory, NULL, CLSCTX_INPROC_SERVER,
//         &IID_IWICImagingFactory, (void**)&factory);
//     if (SUCCEEDED(hr)) /* created factory */
//     {
//         hr = IWICImagingFactory_CreateStream(factory, &stream);
//         IWICImagingFactory_Release(factory);
//     }
//
//     if (SUCCEEDED(hr)) /* created stream */
//     {
//         hr = IWICStream_InitializeFromMemory(stream, xbuf, xread);
//
//         if (SUCCEEDED(hr)) /* initialized stream */
//         {
//             hr = CoCreateInstance(decoder_clsid, NULL, CLSCTX_INPROC_SERVER,
//                 &IID_IWICBitmapDecoder, (void**)&decoder);
//             if (SUCCEEDED(hr)) /* created decoder */
//             {
//                 hr = IWICBitmapDecoder_Initialize(decoder, (IStream*)stream, WICDecodeMetadataCacheOnLoad);
//
//                 if (SUCCEEDED(hr)) /* initialized decoder */
//                     hr = IWICBitmapDecoder_GetFrame(decoder, 0, &framedecode);
//
//                 IWICBitmapDecoder_Release(decoder);
//             }
//         }
//
//         IWICStream_Release(stream);
//     }
//
//     if (SUCCEEDED(hr)) /* got framedecode */
//     {
//         hr = OLEPictureImpl_LoadWICSource(This, (IWICBitmapSource*)framedecode);
//         IWICBitmapFrameDecode_Release(framedecode);
//     }
//
//     if (SUCCEEDED(initresult)) CoUninitialize();
//     return hr;
// }
//
// /*****************************************************
// *   start of Icon-specific code
// */
//
// static HRESULT OLEPictureImpl_LoadIcon(OLEPictureImpl *This, BYTE *xbuf, ULONG xread)
// {
//     HICON hicon;
//     CURSORICONFILEDIR	*cifd = (CURSORICONFILEDIR*)xbuf;
//     HDC hdcRef;
//     int	i;
//
//     TRACE("(this %p, xbuf %p, xread %u)\n", This, xbuf, xread);
//
//     /*
//     FIXME("icon.idReserved=%d\n",cifd->idReserved);
//     FIXME("icon.idType=%d\n",cifd->idType);
//     FIXME("icon.idCount=%d\n",cifd->idCount);
//
//     for (i=0;i<cifd->idCount;i++) {
// 	FIXME("[%d] width %d\n",i,cifd->idEntries[i].bWidth);
// 	FIXME("[%d] height %d\n",i,cifd->idEntries[i].bHeight);
// 	FIXME("[%d] bColorCount %d\n",i,cifd->idEntries[i].bColorCount);
// 	FIXME("[%d] bReserved %d\n",i,cifd->idEntries[i].bReserved);
// 	FIXME("[%d] xHotspot %d\n",i,cifd->idEntries[i].xHotspot);
// 	FIXME("[%d] yHotspot %d\n",i,cifd->idEntries[i].yHotspot);
// 	FIXME("[%d] dwDIBSize %d\n",i,cifd->idEntries[i].dwDIBSize);
// 	FIXME("[%d] dwDIBOffset %d\n",i,cifd->idEntries[i].dwDIBOffset);
//     }
//     */
//
//     /* Need at least one icon to do something. */
//     if (!cifd->idCount)
//     {
//         ERR("Invalid icon count of zero.\n");
//         return E_FAIL;
//     }
//     i=0;
//     /* If we have more than one icon, try to find the best.
//      * this currently means '32 pixel wide'.
//      */
//     if (cifd->idCount!=1) {
// 	for (i=0;i<cifd->idCount;i++) {
// 	    if (cifd->idEntries[i].bWidth == 32)
// 		break;
// 	}
// 	if (i==cifd->idCount) i=0;
//     }
//     if (xread < cifd->idEntries[i].dwDIBOffset + cifd->idEntries[i].dwDIBSize)
//     {
//         ERR("Icon data address %u is over %u bytes available.\n",
//             cifd->idEntries[i].dwDIBOffset + cifd->idEntries[i].dwDIBSize, xread);
//         return E_FAIL;
//     }
//     if (cifd->idType == 2)
//     {
//         LPBYTE buf = HeapAlloc(GetProcessHeap(), 0, cifd->idEntries[i].dwDIBSize + 4);
//         memcpy(buf, &cifd->idEntries[i].xHotspot, 4);
//         memcpy(buf + 4, xbuf+cifd->idEntries[i].dwDIBOffset, cifd->idEntries[i].dwDIBSize);
//         hicon = CreateIconFromResourceEx(
// 		    buf,
// 		    cifd->idEntries[i].dwDIBSize + 4,
// 		    FALSE, /* is cursor */
// 		    0x00030000,
// 		    cifd->idEntries[i].bWidth,
// 		    cifd->idEntries[i].bHeight,
// 		    0
// 	);
// 	HeapFree(GetProcessHeap(), 0, buf);
//     }
//     else
//     {
//         hicon = CreateIconFromResourceEx(
// 		    xbuf+cifd->idEntries[i].dwDIBOffset,
// 		    cifd->idEntries[i].dwDIBSize,
// 		    TRUE, /* is icon */
// 		    0x00030000,
// 		    cifd->idEntries[i].bWidth,
// 		    cifd->idEntries[i].bHeight,
// 		    0
// 	);
//     }
//     if (!hicon) {
// 	ERR("CreateIcon failed.\n");
// 	return E_FAIL;
//     } else {
// 	This->desc.picType = PICTYPE_ICON;
// 	This->desc.u.icon.hicon = hicon;
// 	This->origWidth = cifd->idEntries[i].bWidth;
// 	This->origHeight = cifd->idEntries[i].bHeight;
// 	hdcRef = CreateCompatibleDC(0);
// 	This->himetricWidth = xpixels_to_himetric(cifd->idEntries[i].bWidth, hdcRef);
// 	This->himetricHeight= ypixels_to_himetric(cifd->idEntries[i].bHeight, hdcRef);
// 	DeleteDC(hdcRef);
// 	return S_OK;
//     }
// }
//
// static HRESULT OLEPictureImpl_LoadEnhMetafile(OLEPictureImpl *This,
//                                               const BYTE *data, ULONG size)
// {
//     HENHMETAFILE hemf;
//     ENHMETAHEADER hdr;
//
//     hemf = SetEnhMetaFileBits(size, data);
//     if (!hemf) return E_FAIL;
//
//     GetEnhMetaFileHeader(hemf, sizeof(hdr), &hdr);
//
//     This->desc.picType = PICTYPE_ENHMETAFILE;
//     This->desc.u.emf.hemf = hemf;
//
//     This->origWidth = 0;
//     This->origHeight = 0;
//     This->himetricWidth = hdr.rclFrame.right - hdr.rclFrame.left;
//     This->himetricHeight = hdr.rclFrame.bottom - hdr.rclFrame.top;
//
//     return S_OK;
// }
//
// static HRESULT OLEPictureImpl_LoadAPM(OLEPictureImpl *This,
//                                       const BYTE *data, ULONG size)
// {
//     const APM_HEADER *header = (const APM_HEADER *)data;
//     HMETAFILE hmf;
//
//     if (size < sizeof(APM_HEADER))
//         return E_FAIL;
//     if (header->key != 0x9ac6cdd7)
//         return E_FAIL;
//
//     /* SetMetaFileBitsEx performs data check on its own */
//     hmf = SetMetaFileBitsEx(size - sizeof(*header), data + sizeof(*header));
//     if (!hmf) return E_FAIL;
//
//     This->desc.picType = PICTYPE_METAFILE;
//     This->desc.u.wmf.hmeta = hmf;
//     This->desc.u.wmf.xExt = 0;
//     This->desc.u.wmf.yExt = 0;
//
//     This->origWidth = 0;
//     This->origHeight = 0;
//     This->himetricWidth = MulDiv((INT)header->right - header->left, 2540, header->inch);
//     This->himetricHeight = MulDiv((INT)header->bottom - header->top, 2540, header->inch);
//     return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_IPersistStream_Load (IUnknown)
//  *
//  * Loads the binary data from the IStream. Starts at current position.
//  * There appears to be an 2 DWORD header:
//  * 	DWORD magic;
//  * 	DWORD len;
//  *
//  * Currently implemented: BITMAP, ICON, CURSOR, JPEG, GIF, WMF, EMF
//  */
// static HRESULT WINAPI OLEPictureImpl_Load(IPersistStream* iface, IStream *pStm) {
//   HRESULT	hr;
//   BOOL		headerisdata;
//   BOOL		statfailed = FALSE;
//   ULONG		xread, toread;
//   ULONG 	headerread;
//   BYTE 		*xbuf;
//   DWORD		header[2];
//   WORD		magic;
//   STATSTG       statstg;
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//   
//   TRACE("(%p,%p)\n",This,pStm);
//
//   /****************************************************************************************
//    * Part 1: Load the data
//    */
//   /* Sometimes we have a header, sometimes we don't. Apply some guesses to find
//    * out whether we do.
//    *
//    * UPDATE: the IStream can be mapped to a plain file instead of a stream in a
//    * compound file. This may explain most, if not all, of the cases of "no
//    * header", and the header validation should take this into account.
//    * At least in Visual Basic 6, resource streams, valid headers are
//    *    header[0] == "lt\0\0",
//    *    header[1] == length_of_stream.
//    *
//    * Also handle streams where we do not have a working "Stat" method by
//    * reading all data until the end of the stream.
//    */
//   hr = IStream_Stat(pStm,&statstg,STATFLAG_NONAME);
//   if (hr != S_OK) {
//       TRACE("stat failed with hres %x, proceeding to read all data.\n",hr);
//       statfailed = TRUE;
//       /* we will read at least 8 byte ... just right below */
//       statstg.cbSize.QuadPart = 8;
//   }
//
//   toread = 0;
//   headerread = 0;
//   headerisdata = FALSE;
//   do {
//       hr = IStream_Read(pStm, header, 8, &xread);
//       if (hr != S_OK || xread!=8) {
//           ERR("Failure while reading picture header (hr is %x, nread is %d).\n",hr,xread);
//           return (hr?hr:E_FAIL);
//       }
//       headerread += xread;
//       xread = 0;
//
//       if (!memcmp(&(header[0]),"lt\0\0", 4) && (statfailed || (header[1] + headerread <= statstg.cbSize.QuadPart))) {
//           if (toread != 0 && toread != header[1]) 
//               FIXME("varying lengths of image data (prev=%u curr=%u), only last one will be used\n",
//                   toread, header[1]);
//           toread = header[1];
//           if (statfailed)
//           {
//               statstg.cbSize.QuadPart = header[1] + 8;
//               statfailed = FALSE;
//           }
//           if (toread == 0) break;
//       } else {
//           if (!memcmp(&(header[0]), "GIF8",     4) ||   /* GIF header */
//               !memcmp(&(header[0]), "BM",       2) ||   /* BMP header */
//               !memcmp(&(header[0]), "\xff\xd8", 2) ||   /* JPEG header */
//               (header[0] == EMR_HEADER)            ||   /* EMF header */
//               (header[0] == 0x10000)               ||   /* icon: idReserved 0, idType 1 */
//               (header[0] == 0x20000)               ||   /* cursor: idReserved 0, idType 2 */
//               (header[1] > statstg.cbSize.QuadPart)||   /* invalid size */
//               (header[1]==0)
//           ) {/* Found start of bitmap data */
//               headerisdata = TRUE;
//               if (toread == 0) 
//               	  toread = statstg.cbSize.QuadPart-8;
//               else toread -= 8;
//               xread = 8;
//           } else {
//               FIXME("Unknown stream header magic: %08x\n", header[0]);
//               toread = header[1];
//           }
//       }
//   } while (!headerisdata);
//
//   if (statfailed) { /* we don't know the size ... read all we get */
//       unsigned int sizeinc = 4096;
//       unsigned int origsize = sizeinc;
//       ULONG nread = 42;
//
//       TRACE("Reading all data from stream.\n");
//       xbuf = HeapAlloc (GetProcessHeap(), HEAP_ZERO_MEMORY, origsize);
//       if (headerisdata)
//           memcpy (xbuf, header, 8);
//       while (1) {
//           while (xread < origsize) {
//               hr = IStream_Read(pStm,xbuf+xread,origsize-xread,&nread);
//               xread += nread;
//               if (hr != S_OK || !nread)
//                   break;
//           }
//           if (!nread || hr != S_OK) /* done, or error */
//               break;
//           if (xread == origsize) {
//               origsize += sizeinc;
//               sizeinc = 2*sizeinc; /* exponential increase */
//               xbuf = HeapReAlloc (GetProcessHeap(), HEAP_ZERO_MEMORY, xbuf, origsize);
//           }
//       }
//       if (hr != S_OK)
//           TRACE("hr in no-stat loader case is %08x\n", hr);
//       TRACE("loaded %d bytes.\n", xread);
//       This->datalen = xread;
//       This->data    = xbuf;
//   } else {
//       This->datalen = toread+(headerisdata?8:0);
//       xbuf = This->data = HeapAlloc (GetProcessHeap(), HEAP_ZERO_MEMORY, This->datalen);
//       if (!xbuf)
//           return E_OUTOFMEMORY;
//
//       if (headerisdata)
//           memcpy (xbuf, header, 8);
//
//       while (xread < This->datalen) {
//           ULONG nread;
//           hr = IStream_Read(pStm,xbuf+xread,This->datalen-xread,&nread);
//           xread += nread;
//           if (hr != S_OK || !nread)
//               break;
//       }
//       if (xread != This->datalen)
//           ERR("Could only read %d of %d bytes out of stream?\n",xread,This->datalen);
//   }
//   if (This->datalen == 0) { /* Marks the "NONE" picture */
//       This->desc.picType = PICTYPE_NONE;
//       return S_OK;
//   }
//
//
//   /****************************************************************************************
//    * Part 2: Process the loaded data
//    */
//
//   magic = xbuf[0] + (xbuf[1]<<8);
//   This->loadtime_format = magic;
//
//   switch (magic) {
//   case BITMAP_FORMAT_GIF: /* GIF */
//     hr = OLEPictureImpl_LoadWICDecoder(This, &CLSID_WICGifDecoder, xbuf, xread);
//     break;
//   case BITMAP_FORMAT_JPEG: /* JPEG */
//     hr = OLEPictureImpl_LoadWICDecoder(This, &CLSID_WICJpegDecoder, xbuf, xread);
//     break;
//   case BITMAP_FORMAT_BMP: /* Bitmap */
//     hr = OLEPictureImpl_LoadWICDecoder(This, &CLSID_WICBmpDecoder, xbuf, xread);
//     break;
//   case BITMAP_FORMAT_PNG: /* PNG */
//     hr = OLEPictureImpl_LoadWICDecoder(This, &CLSID_WICPngDecoder, xbuf, xread);
//     break;
//   case BITMAP_FORMAT_APM: /* APM */
//     hr = OLEPictureImpl_LoadAPM(This, xbuf, xread);
//     break;
//   case 0x0000: { /* ICON or CURSOR, first word is dwReserved */
//     hr = OLEPictureImpl_LoadIcon(This, xbuf, xread);
//     break;
//   }
//   default:
//   {
//     unsigned int i;
//
//     /* let's see if it's a EMF */
//     hr = OLEPictureImpl_LoadEnhMetafile(This, xbuf, xread);
//     if (hr == S_OK) break;
//
//     FIXME("Unknown magic %04x, %d read bytes:\n",magic,xread);
//     hr=E_FAIL;
//     for (i=0;i<xread+8;i++) {
// 	if (i<8) MESSAGE("%02x ",((unsigned char*)header)[i]);
// 	else MESSAGE("%02x ",xbuf[i-8]);
//         if (i % 10 == 9) MESSAGE("\n");
//     }
//     MESSAGE("\n");
//     break;
//   }
//   }
//   This->bIsDirty = FALSE;
//
//   /* FIXME: this notify is not really documented */
//   if (hr==S_OK)
//       OLEPicture_SendNotify(This,DISPID_PICT_TYPE);
//   return hr;
// }
//
// static BOOL serializeBMP(HBITMAP hBitmap, void ** ppBuffer, unsigned int * pLength)
// {
//     BOOL success = FALSE;
//     HDC hDC;
//     BITMAPINFO * pInfoBitmap;
//     int iNumPaletteEntries;
//     unsigned char * pPixelData;
//     BITMAPFILEHEADER * pFileHeader;
//     BITMAPINFO * pInfoHeader;
//
//     pInfoBitmap = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//         sizeof(BITMAPINFOHEADER) + 256 * sizeof(RGBQUAD));
//
//     /* Find out bitmap size and padded length */
//     hDC = GetDC(0);
//     pInfoBitmap->bmiHeader.biSize = sizeof(pInfoBitmap->bmiHeader);
//     GetDIBits(hDC, hBitmap, 0, 0, NULL, pInfoBitmap, DIB_RGB_COLORS);
//
//     /* Fetch bitmap palette & pixel data */
//
//     pPixelData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, pInfoBitmap->bmiHeader.biSizeImage);
//     GetDIBits(hDC, hBitmap, 0, pInfoBitmap->bmiHeader.biHeight, pPixelData, pInfoBitmap, DIB_RGB_COLORS);
//
//     /* Calculate the total length required for the BMP data */
//     if (pInfoBitmap->bmiHeader.biClrUsed != 0) {
// 	iNumPaletteEntries = pInfoBitmap->bmiHeader.biClrUsed;
// 	if (iNumPaletteEntries > 256) iNumPaletteEntries = 256;
//     } else {
// 	if (pInfoBitmap->bmiHeader.biBitCount <= 8)
// 	    iNumPaletteEntries = 1 << pInfoBitmap->bmiHeader.biBitCount;
// 	else
//     	    iNumPaletteEntries = 0;
//     }
//     *pLength =
//         sizeof(BITMAPFILEHEADER) +
//         sizeof(BITMAPINFOHEADER) +
//         iNumPaletteEntries * sizeof(RGBQUAD) +
//         pInfoBitmap->bmiHeader.biSizeImage;
//     *ppBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, *pLength);
//
//     /* Fill the BITMAPFILEHEADER */
//     pFileHeader = *ppBuffer;
//     pFileHeader->bfType = BITMAP_FORMAT_BMP;
//     pFileHeader->bfSize = *pLength;
//     pFileHeader->bfOffBits =
//         sizeof(BITMAPFILEHEADER) +
//         sizeof(BITMAPINFOHEADER) +
//         iNumPaletteEntries * sizeof(RGBQUAD);
//
//     /* Fill the BITMAPINFOHEADER and the palette data */
//     pInfoHeader = (BITMAPINFO *)((unsigned char *)(*ppBuffer) + sizeof(BITMAPFILEHEADER));
//     memcpy(pInfoHeader, pInfoBitmap, sizeof(BITMAPINFOHEADER) + iNumPaletteEntries * sizeof(RGBQUAD));
//     memcpy(
//         (unsigned char *)(*ppBuffer) +
//             sizeof(BITMAPFILEHEADER) +
//             sizeof(BITMAPINFOHEADER) +
//             iNumPaletteEntries * sizeof(RGBQUAD),
//         pPixelData, pInfoBitmap->bmiHeader.biSizeImage);
//     success = TRUE;
//
//     HeapFree(GetProcessHeap(), 0, pPixelData);
//     HeapFree(GetProcessHeap(), 0, pInfoBitmap);
//     return success;
// }
//
// static BOOL serializeIcon(HICON hIcon, void ** ppBuffer, unsigned int * pLength)
// {
// 	ICONINFO infoIcon;
//         BOOL success = FALSE;
//
// 	*ppBuffer = NULL; *pLength = 0;
// 	if (GetIconInfo(hIcon, &infoIcon)) {
// 		HDC hDC;
// 		BITMAPINFO * pInfoBitmap;
// 		unsigned char * pIconData = NULL;
// 		unsigned int iDataSize = 0;
//
//         pInfoBitmap = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(BITMAPINFOHEADER) + 256 * sizeof(RGBQUAD));
//
// 		/* Find out icon size */
// 		hDC = GetDC(0);
// 		pInfoBitmap->bmiHeader.biSize = sizeof(pInfoBitmap->bmiHeader);
// 		GetDIBits(hDC, infoIcon.hbmColor, 0, 0, NULL, pInfoBitmap, DIB_RGB_COLORS);
// 		if (1) {
// 			/* Auxiliary pointers */
// 			CURSORICONFILEDIR * pIconDir;
// 			CURSORICONFILEDIRENTRY * pIconEntry;
// 			BITMAPINFOHEADER * pIconBitmapHeader;
// 			unsigned int iOffsetPalette;
// 			unsigned int iOffsetColorData;
// 			unsigned int iOffsetMaskData;
//
// 			unsigned int iLengthScanLineMask;
// 			unsigned int iNumEntriesPalette;
//
// 			iLengthScanLineMask = ((pInfoBitmap->bmiHeader.biWidth + 31) >> 5) << 2;
// /*
// 			FIXME("DEBUG: bitmap size is %d x %d\n",
// 				pInfoBitmap->bmiHeader.biWidth,
// 				pInfoBitmap->bmiHeader.biHeight);
// 			FIXME("DEBUG: bitmap bpp is %d\n",
// 				pInfoBitmap->bmiHeader.biBitCount);
// 			FIXME("DEBUG: bitmap nplanes is %d\n",
// 				pInfoBitmap->bmiHeader.biPlanes);
// 			FIXME("DEBUG: bitmap biSizeImage is %u\n",
// 				pInfoBitmap->bmiHeader.biSizeImage);
// */
// 			/* Let's start with one CURSORICONFILEDIR and one CURSORICONFILEDIRENTRY */
// 			iDataSize += 3 * sizeof(WORD) + sizeof(CURSORICONFILEDIRENTRY) + sizeof(BITMAPINFOHEADER);
// 			pIconData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, iDataSize);
//
// 			/* Fill out the CURSORICONFILEDIR */
// 			pIconDir = (CURSORICONFILEDIR *)pIconData;
// 			pIconDir->idType = 1;
// 			pIconDir->idCount = 1;
// 			pIconDir->idReserved = 0;
//
// 			/* Fill out the CURSORICONFILEDIRENTRY */
// 			pIconEntry = (CURSORICONFILEDIRENTRY *)(pIconData + 3 * sizeof(WORD));
// 			pIconEntry->bWidth = (unsigned char)pInfoBitmap->bmiHeader.biWidth;
// 			pIconEntry->bHeight = (unsigned char)pInfoBitmap->bmiHeader.biHeight;
// 			pIconEntry->bColorCount =
// 				(pInfoBitmap->bmiHeader.biBitCount < 8)
// 				? 1 << pInfoBitmap->bmiHeader.biBitCount
// 				: 0;
// 			pIconEntry->xHotspot = pInfoBitmap->bmiHeader.biPlanes;
// 			pIconEntry->yHotspot = pInfoBitmap->bmiHeader.biBitCount;
// 			pIconEntry->dwDIBSize = 0;
// 			pIconEntry->dwDIBOffset = 3 * sizeof(WORD) + sizeof(CURSORICONFILEDIRENTRY);
//
// 			/* Fill out the BITMAPINFOHEADER */
// 			pIconBitmapHeader = (BITMAPINFOHEADER *)(pIconData + 3 * sizeof(WORD) + sizeof(CURSORICONFILEDIRENTRY));
// 			*pIconBitmapHeader = pInfoBitmap->bmiHeader;
//
// 			/*	Find out whether a palette exists for the bitmap */
// 			if (	(pInfoBitmap->bmiHeader.biBitCount == 16 && pInfoBitmap->bmiHeader.biCompression == BI_RGB)
// 				||	(pInfoBitmap->bmiHeader.biBitCount == 24)
// 				||	(pInfoBitmap->bmiHeader.biBitCount == 32 && pInfoBitmap->bmiHeader.biCompression == BI_RGB)) {
// 				iNumEntriesPalette = pInfoBitmap->bmiHeader.biClrUsed;
// 				if (iNumEntriesPalette > 256) iNumEntriesPalette = 256; 
// 			} else if ((pInfoBitmap->bmiHeader.biBitCount == 16 || pInfoBitmap->bmiHeader.biBitCount == 32)
// 				&& pInfoBitmap->bmiHeader.biCompression == BI_BITFIELDS) {
// 				iNumEntriesPalette = 3;
// 			} else if (pInfoBitmap->bmiHeader.biBitCount <= 8) {
// 				iNumEntriesPalette = 1 << pInfoBitmap->bmiHeader.biBitCount;
// 			} else {
// 				iNumEntriesPalette = 0;
// 			}
//
// 			/*  Add bitmap size and header size to icon data size. */
// 			iOffsetPalette = iDataSize;
// 			iDataSize += iNumEntriesPalette * sizeof(DWORD);
// 			iOffsetColorData = iDataSize;
// 			iDataSize += pIconBitmapHeader->biSizeImage;
// 			iOffsetMaskData = iDataSize;
// 			iDataSize += pIconBitmapHeader->biHeight * iLengthScanLineMask;
// 			pIconBitmapHeader->biSizeImage += pIconBitmapHeader->biHeight * iLengthScanLineMask;
// 			pIconBitmapHeader->biHeight *= 2;
// 			pIconData = HeapReAlloc(GetProcessHeap(), 0, pIconData, iDataSize);
// 			pIconEntry = (CURSORICONFILEDIRENTRY *)(pIconData + 3 * sizeof(WORD));
// 			pIconBitmapHeader = (BITMAPINFOHEADER *)(pIconData + 3 * sizeof(WORD) + sizeof(CURSORICONFILEDIRENTRY));
// 			pIconEntry->dwDIBSize = iDataSize - (3 * sizeof(WORD) + sizeof(CURSORICONFILEDIRENTRY));
//
// 			/* Get the actual bitmap data from the icon bitmap */
// 			GetDIBits(hDC, infoIcon.hbmColor, 0, pInfoBitmap->bmiHeader.biHeight,
// 				pIconData + iOffsetColorData, pInfoBitmap, DIB_RGB_COLORS);
// 			if (iNumEntriesPalette > 0) {
// 				memcpy(pIconData + iOffsetPalette, pInfoBitmap->bmiColors,
// 					iNumEntriesPalette * sizeof(RGBQUAD));
// 			}
//
// 			/* Reset all values so that GetDIBits call succeeds */
// 			memset(pIconData + iOffsetMaskData, 0, iDataSize - iOffsetMaskData);
// 			memset(pInfoBitmap, 0, sizeof(BITMAPINFOHEADER) + 256 * sizeof(RGBQUAD));
// 			pInfoBitmap->bmiHeader.biSize = sizeof(pInfoBitmap->bmiHeader);
// /*
//             if (!(GetDIBits(hDC, infoIcon.hbmMask, 0, 0, NULL, pInfoBitmap, DIB_RGB_COLORS)
// 				&& GetDIBits(hDC, infoIcon.hbmMask, 0, pIconEntry->bHeight,
// 					pIconData + iOffsetMaskData, pInfoBitmap, DIB_RGB_COLORS))) {
//
//                 printf("ERROR: unable to get bitmap mask (error %u)\n",
// 					GetLastError());
//
// 			}
// */
//             GetDIBits(hDC, infoIcon.hbmMask, 0, 0, NULL, pInfoBitmap, DIB_RGB_COLORS);
//             GetDIBits(hDC, infoIcon.hbmMask, 0, pIconEntry->bHeight, pIconData + iOffsetMaskData, pInfoBitmap, DIB_RGB_COLORS);
//
// 			/* Write out everything produced so far to the stream */
// 			*ppBuffer = pIconData; *pLength = iDataSize;
//                         success = TRUE;
// 		} else {
// /*
// 			printf("ERROR: unable to get bitmap information via GetDIBits() (error %u)\n",
// 				GetLastError());
// */
// 		}
// 		/*
// 			Remarks (from MSDN entry on GetIconInfo):
//
// 			GetIconInfo creates bitmaps for the hbmMask and hbmColor
// 			members of ICONINFO. The calling application must manage
// 			these bitmaps and delete them when they are no longer
// 			necessary.
// 		 */
// 		if (hDC) ReleaseDC(0, hDC);
// 		DeleteObject(infoIcon.hbmMask);
// 		if (infoIcon.hbmColor) DeleteObject(infoIcon.hbmColor);
// 		HeapFree(GetProcessHeap(), 0, pInfoBitmap);
// 	} else {
// 		printf("ERROR: Unable to get icon information (error %u)\n",
// 			GetLastError());
// 	}
//         return success;
// }
//
// static BOOL serializeEMF(HENHMETAFILE hemf, void **buf, unsigned *size)
// {
//     *size = GetEnhMetaFileBits(hemf, 0, NULL);
//     if (!*size) return FALSE;
//
//     *buf = HeapAlloc(GetProcessHeap(), 0, *size);
//     if (!*buf) return FALSE;
//
//     return GetEnhMetaFileBits(hemf, *size, *buf) != 0;
// }
//
// static HRESULT WINAPI OLEPictureImpl_Save(
//   IPersistStream* iface,IStream*pStm,BOOL fClearDirty)
// {
//     HRESULT hResult = E_NOTIMPL;
//     void * pIconData;
//     unsigned int iDataSize;
//     DWORD header[2];
//     ULONG dummy;
//     BOOL serializeResult = FALSE;
//     OLEPictureImpl *This = impl_from_IPersistStream(iface);
//
//     TRACE("%p %p %d\n", This, pStm, fClearDirty);
//
//     switch (This->desc.picType) {
//     case PICTYPE_NONE:
//         header[0] = 0x0000746c;
//         header[1] = 0;
//         hResult = IStream_Write(pStm, header, 2 * sizeof(DWORD), &dummy);
//         break;
//
//     case PICTYPE_ICON:
//         if (This->bIsDirty || !This->data) {
//             if (!serializeIcon(This->desc.u.icon.hicon, &pIconData, &iDataSize)) {
//                 ERR("(%p,%p,%d), serializeIcon() failed\n", This, pStm, fClearDirty);
//                 hResult = E_FAIL;
//                 break;
//             }
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = pIconData;
//             This->datalen = iDataSize;
//         }
//
//         header[0] = (This->loadtime_magic != 0xdeadbeef) ? This->loadtime_magic : 0x0000746c;
//         header[1] = This->datalen;
//         IStream_Write(pStm, header, 2 * sizeof(DWORD), &dummy);
//         IStream_Write(pStm, This->data, This->datalen, &dummy);
//         hResult = S_OK;
//         break;
//     case PICTYPE_BITMAP:
//         if (This->bIsDirty || !This->data) {
//             switch (This->keepOrigFormat ? This->loadtime_format : BITMAP_FORMAT_BMP) {
//             case BITMAP_FORMAT_BMP:
//                 serializeResult = serializeBMP(This->desc.u.bmp.hbitmap, &pIconData, &iDataSize);
//                 break;
//             case BITMAP_FORMAT_JPEG:
//                 FIXME("(%p,%p,%d), PICTYPE_BITMAP (format JPEG) not implemented!\n",This,pStm,fClearDirty);
//                 break;
//             case BITMAP_FORMAT_GIF:
//                 FIXME("(%p,%p,%d), PICTYPE_BITMAP (format GIF) not implemented!\n",This,pStm,fClearDirty);
//                 break;
//             case BITMAP_FORMAT_PNG:
//                 FIXME("(%p,%p,%d), PICTYPE_BITMAP (format PNG) not implemented!\n",This,pStm,fClearDirty);
//                 break;
//             default:
//                 FIXME("(%p,%p,%d), PICTYPE_BITMAP (format UNKNOWN, using BMP?) not implemented!\n",This,pStm,fClearDirty);
//                 break;
//             }
//
//             if (!serializeResult)
//             {
//                 hResult = E_FAIL;
//                 break;
//             }
//
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = pIconData;
//             This->datalen = iDataSize;
//         }
//
//         header[0] = (This->loadtime_magic != 0xdeadbeef) ? This->loadtime_magic : 0x0000746c;
//         header[1] = This->datalen;
//         IStream_Write(pStm, header, 2 * sizeof(DWORD), &dummy);
//         IStream_Write(pStm, This->data, This->datalen, &dummy);
//         hResult = S_OK;
//         break;
//
//     case PICTYPE_ENHMETAFILE:
//         if (This->bIsDirty || !This->data)
//         {
//             serializeResult = serializeEMF(This->desc.u.emf.hemf, &pIconData, &iDataSize);
//             if (!serializeResult)
//             {
//                 hResult = E_FAIL;
//                 break;
//             }
//
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = pIconData;
//             This->datalen = iDataSize;
//         }
//         header[0] = 0x0000746c;
//         header[1] = This->datalen;
//         IStream_Write(pStm, header, 2 * sizeof(DWORD), &dummy);
//         IStream_Write(pStm, This->data, This->datalen, &dummy);
//         hResult = S_OK;
//         break;
//
//     case PICTYPE_METAFILE:
//         FIXME("(%p,%p,%d), PICTYPE_METAFILE not implemented!\n",This,pStm,fClearDirty);
//         break;
//     default:
//         FIXME("(%p,%p,%d), [unknown type] not implemented!\n",This,pStm,fClearDirty);
//         break;
//     }
//     if (hResult == S_OK && fClearDirty) This->bIsDirty = FALSE;
//     return hResult;
// }
//
// static HRESULT WINAPI OLEPictureImpl_GetSizeMax(
//   IPersistStream* iface,ULARGE_INTEGER*pcbSize)
// {
//   OLEPictureImpl *This = impl_from_IPersistStream(iface);
//   FIXME("(%p,%p),stub!\n",This,pcbSize);
//   return E_NOTIMPL;
// }
//
// /************************************************************************
//  * OLEPictureImpl_SaveAsFile
//  */
// static HRESULT WINAPI OLEPictureImpl_SaveAsFile(IPicture *iface,
//     IStream *stream, BOOL mem_copy, LONG *size)
// {
//     OLEPictureImpl *This = impl_from_IPicture(iface);
//     void *data;
//     unsigned data_size;
//     ULONG written;
//     HRESULT hr;
//
//     FIXME("(%p)->(%p,%d,%p): semi-stub\n", This, stream, mem_copy, size);
//
//     switch (This->desc.picType)
//     {
//     case PICTYPE_NONE:
//         return S_OK;
//
//     case PICTYPE_ICON:
//         if (!mem_copy) return E_FAIL;
//
//         if (This->bIsDirty || !This->data)
//         {
//             if (!serializeIcon(This->desc.u.icon.hicon, &data, &data_size))
//                 return E_FAIL;
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = data;
//             This->datalen = data_size;
//         }
//         hr = IStream_Write(stream, This->data, This->datalen, &written);
//         if (hr == S_OK && size) *size = written;
//         return hr;
//
//     case PICTYPE_BITMAP:
//         if (!mem_copy) return E_FAIL;
//
//         if (This->bIsDirty || !This->data)
//         {
//             switch (This->keepOrigFormat ? This->loadtime_format : BITMAP_FORMAT_BMP)
//             {
//             case BITMAP_FORMAT_BMP:
//                 if (!serializeBMP(This->desc.u.bmp.hbitmap, &data, &data_size))
//                     return E_FAIL;
//                 break;
//             case BITMAP_FORMAT_JPEG:
//                 FIXME("BITMAP_FORMAT_JPEG is not implemented\n");
//                 return E_NOTIMPL;
//             case BITMAP_FORMAT_GIF:
//                 FIXME("BITMAP_FORMAT_GIF is not implemented\n");
//                 return E_NOTIMPL;
//             case BITMAP_FORMAT_PNG:
//                 FIXME("BITMAP_FORMAT_PNG is not implemented\n");
//                 return E_NOTIMPL;
//             default:
//                 FIXME("PICTYPE_BITMAP/%#x is not implemented\n", This->loadtime_format);
//                 return E_NOTIMPL;
//             }
//
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = data;
//             This->datalen = data_size;
//         }
//         hr = IStream_Write(stream, This->data, This->datalen, &written);
//         if (hr == S_OK && size) *size = written;
//         return hr;
//
//     case PICTYPE_METAFILE:
//         FIXME("PICTYPE_METAFILE is not implemented\n");
//         return E_NOTIMPL;
//
//     case PICTYPE_ENHMETAFILE:
//         if (!mem_copy) return E_FAIL;
//
//         if (This->bIsDirty || !This->data)
//         {
//             if (!serializeEMF(This->desc.u.emf.hemf, &data, &data_size))
//                 return E_FAIL;
//             HeapFree(GetProcessHeap(), 0, This->data);
//             This->data = data;
//             This->datalen = data_size;
//         }
//         hr = IStream_Write(stream, This->data, This->datalen, &written);
//         if (hr == S_OK && size) *size = written;
//         return hr;
//
//     default:
//         FIXME("%#x is not implemented\n", This->desc.picType);
//         break;
//     }
//     return E_NOTIMPL;
// }
//
// /************************************************************************
//  *    IDispatch
//  */
//
// /************************************************************************
//  * OLEPictureImpl_IDispatch_QueryInterface (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_IDispatch_QueryInterface(
//   IDispatch* iface,
//   REFIID     riid,
//   VOID**     ppvoid)
// {
//   OLEPictureImpl *This = impl_from_IDispatch(iface);
//
//   return IPicture_QueryInterface(&This->IPicture_iface, riid, ppvoid);
// }
//
// /************************************************************************
//  * OLEPictureImpl_IDispatch_AddRef (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_IDispatch_AddRef(
//   IDispatch* iface)
// {
//   OLEPictureImpl *This = impl_from_IDispatch(iface);
//
//   return IPicture_AddRef(&This->IPicture_iface);
// }
//
// /************************************************************************
//  * OLEPictureImpl_IDispatch_Release (IUnknown)
//  *
//  * See Windows documentation for more details on IUnknown methods.
//  */
// static ULONG WINAPI OLEPictureImpl_IDispatch_Release(
//   IDispatch* iface)
// {
//   OLEPictureImpl *This = impl_from_IDispatch(iface);
//
//   return IPicture_Release(&This->IPicture_iface);
// }
//
// /************************************************************************
//  * OLEPictureImpl_GetTypeInfoCount (IDispatch)
//  *
//  * See Windows documentation for more details on IDispatch methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_GetTypeInfoCount(
//   IDispatch*    iface,
//   unsigned int* pctinfo)
// {
//   TRACE("(%p)\n", pctinfo);
//
//   *pctinfo = 1;
//
//   return S_OK;
// }
//
// /************************************************************************
//  * OLEPictureImpl_GetTypeInfo (IDispatch)
//  *
//  * See Windows documentation for more details on IDispatch methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_GetTypeInfo(
//   IDispatch*  iface,
//   UINT      iTInfo,
//   LCID        lcid,
//   ITypeInfo** ppTInfo)
// {
//   static const WCHAR stdole2tlb[] = {'s','t','d','o','l','e','2','.','t','l','b',0};
//   ITypeLib *tl;
//   HRESULT hres;
//
//   TRACE("(iTInfo=%d, lcid=%04x, %p)\n", iTInfo, (int)lcid, ppTInfo);
//
//   if (iTInfo != 0)
//     return E_FAIL;
//
//   hres = LoadTypeLib(stdole2tlb, &tl);
//   if (FAILED(hres))
//   {
//     ERR("Could not load stdole2.tlb\n");
//     return hres;
//   }
//
//   hres = ITypeLib_GetTypeInfoOfGuid(tl, &IID_IPictureDisp, ppTInfo);
//   if (FAILED(hres))
//     ERR("Did not get IPictureDisp typeinfo from typelib, hres %x\n", hres);
//
//   return hres;
// }
//
// /************************************************************************
//  * OLEPictureImpl_GetIDsOfNames (IDispatch)
//  *
//  * See Windows documentation for more details on IDispatch methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_GetIDsOfNames(
//   IDispatch*  iface,
//   REFIID      riid,
//   LPOLESTR* rgszNames,
//   UINT      cNames,
//   LCID        lcid,
//   DISPID*     rgDispId)
// {
//   ITypeInfo * pTInfo;
//   HRESULT hres;
//
//   TRACE("(%p,%s,%p,cNames=%d,lcid=%04x,%p)\n", iface, debugstr_guid(riid),
//         rgszNames, cNames, (int)lcid, rgDispId);
//
//   if (cNames == 0)
//   {
//     return E_INVALIDARG;
//   }
//   else
//   {
//     /* retrieve type information */
//     hres = OLEPictureImpl_GetTypeInfo(iface, 0, lcid, &pTInfo);
//
//     if (FAILED(hres))
//     {
//       ERR("GetTypeInfo failed.\n");
//       return hres;
//     }
//
//     /* convert names to DISPIDs */
//     hres = DispGetIDsOfNames (pTInfo, rgszNames, cNames, rgDispId);
//     ITypeInfo_Release(pTInfo);
//
//     return hres;
//   }
// }
//
// /************************************************************************
//  * OLEPictureImpl_Invoke (IDispatch)
//  *
//  * See Windows documentation for more details on IDispatch methods.
//  */
// static HRESULT WINAPI OLEPictureImpl_Invoke(
//   IDispatch*  iface,
//   DISPID      dispIdMember,
//   REFIID      riid,
//   LCID        lcid,
//   WORD        wFlags,
//   DISPPARAMS* pDispParams,
//   VARIANT*    pVarResult,
//   EXCEPINFO*  pExepInfo,
//   UINT*     puArgErr)
// {
//   OLEPictureImpl *This = impl_from_IDispatch(iface);
//   HRESULT hr;
//
//   /* validate parameters */
//
//   if (!IsEqualIID(riid, &IID_NULL))
//   {
//     ERR("riid was %s instead of IID_NULL\n", debugstr_guid(riid));
//     return DISP_E_UNKNOWNNAME;
//   }
//
//   if (!pDispParams)
//   {
//     ERR("null pDispParams not allowed\n");
//     return DISP_E_PARAMNOTOPTIONAL;
//   }
//
//   if (wFlags & DISPATCH_PROPERTYGET)
//   {
//     if (pDispParams->cArgs != 0)
//     {
//       ERR("param count for DISPATCH_PROPERTYGET was %d instead of 0\n", pDispParams->cArgs);
//       return DISP_E_BADPARAMCOUNT;
//     }
//     if (!pVarResult)
//     {
//       ERR("null pVarResult not allowed when DISPATCH_PROPERTYGET specified\n");
//       return DISP_E_PARAMNOTOPTIONAL;
//     }
//   }
//   else if (wFlags & DISPATCH_PROPERTYPUT)
//   {
//     if (pDispParams->cArgs != 1)
//     {
//       ERR("param count for DISPATCH_PROPERTYPUT was %d instead of 1\n", pDispParams->cArgs);
//       return DISP_E_BADPARAMCOUNT;
//     }
//   }
//
//   switch (dispIdMember)
//   {
//   case DISPID_PICT_HANDLE:
//     if (wFlags & DISPATCH_PROPERTYGET)
//     {
//       TRACE("DISPID_PICT_HANDLE\n");
//       V_VT(pVarResult) = VT_I4;
//       return IPicture_get_Handle(&This->IPicture_iface, &V_UINT(pVarResult));
//     }
//     break;
//   case DISPID_PICT_HPAL:
//     if (wFlags & DISPATCH_PROPERTYGET)
//     {
//       TRACE("DISPID_PICT_HPAL\n");
//       V_VT(pVarResult) = VT_I4;
//       return IPicture_get_hPal(&This->IPicture_iface, &V_UINT(pVarResult));
//     }
//     else if (wFlags & DISPATCH_PROPERTYPUT)
//     {
//       VARIANTARG vararg;
//
//       TRACE("DISPID_PICT_HPAL\n");
//
//       VariantInit(&vararg);
//       hr = VariantChangeTypeEx(&vararg, &pDispParams->rgvarg[0], lcid, 0, VT_I4);
//       if (FAILED(hr))
//         return hr;
//
//       hr = IPicture_set_hPal(&This->IPicture_iface, V_I4(&vararg));
//
//       VariantClear(&vararg);
//       return hr;
//     }
//     break;
//   case DISPID_PICT_TYPE:
//     if (wFlags & DISPATCH_PROPERTYGET)
//     {
//       TRACE("DISPID_PICT_TYPE\n");
//       V_VT(pVarResult) = VT_I2;
//       return OLEPictureImpl_get_Type(&This->IPicture_iface, &V_I2(pVarResult));
//     }
//     break;
//   case DISPID_PICT_WIDTH:
//     if (wFlags & DISPATCH_PROPERTYGET)
//     {
//       TRACE("DISPID_PICT_WIDTH\n");
//       V_VT(pVarResult) = VT_I4;
//       return IPicture_get_Width(&This->IPicture_iface, &V_I4(pVarResult));
//     }
//     break;
//   case DISPID_PICT_HEIGHT:
//     if (wFlags & DISPATCH_PROPERTYGET)
//     {
//       TRACE("DISPID_PICT_HEIGHT\n");
//       V_VT(pVarResult) = VT_I4;
//       return IPicture_get_Height(&This->IPicture_iface, &V_I4(pVarResult));
//     }
//     break;
//   case DISPID_PICT_RENDER:
//     if (wFlags & DISPATCH_METHOD)
//     {
//       VARIANTARG *args = pDispParams->rgvarg;
//       int i;
//
//       TRACE("DISPID_PICT_RENDER\n");
//
//       if (pDispParams->cArgs != 10)
//         return DISP_E_BADPARAMCOUNT;
//
//       /* All parameters are supposed to be VT_I4 (on 64 bits too). */
//       for (i = 0; i < pDispParams->cArgs; i++)
//         if (V_VT(&args[i]) != VT_I4)
//         {
//           ERR("DISPID_PICT_RENDER: wrong argument type %d:%d\n", i, V_VT(&args[i]));
//           return DISP_E_TYPEMISMATCH;
//         }
//
//       /* FIXME: rectangle pointer argument handling seems broken on 64 bits,
//                 currently Render() doesn't use it at all so for now NULL is passed. */
//       return IPicture_Render(&This->IPicture_iface,
//                 LongToHandle(V_I4(&args[9])),
//                              V_I4(&args[8]),
//                              V_I4(&args[7]),
//                              V_I4(&args[6]),
//                              V_I4(&args[5]),
//                              V_I4(&args[4]),
//                              V_I4(&args[3]),
//                              V_I4(&args[2]),
//                              V_I4(&args[1]),
//                                       NULL);
//     }
//     break;
//   }
//
//   ERR("invalid dispid 0x%x or wFlags 0x%x\n", dispIdMember, wFlags);
//   return DISP_E_MEMBERNOTFOUND;
// }
//
//
// static const IPictureVtbl OLEPictureImpl_VTable =
// {
//   OLEPictureImpl_QueryInterface,
//   OLEPictureImpl_AddRef,
//   OLEPictureImpl_Release,
//   OLEPictureImpl_get_Handle,
//   OLEPictureImpl_get_hPal,
//   OLEPictureImpl_get_Type,
//   OLEPictureImpl_get_Width,
//   OLEPictureImpl_get_Height,
//   OLEPictureImpl_Render,
//   OLEPictureImpl_set_hPal,
//   OLEPictureImpl_get_CurDC,
//   OLEPictureImpl_SelectPicture,
//   OLEPictureImpl_get_KeepOriginalFormat,
//   OLEPictureImpl_put_KeepOriginalFormat,
//   OLEPictureImpl_PictureChanged,
//   OLEPictureImpl_SaveAsFile,
//   OLEPictureImpl_get_Attributes
// };
//
// static const IDispatchVtbl OLEPictureImpl_IDispatch_VTable =
// {
//   OLEPictureImpl_IDispatch_QueryInterface,
//   OLEPictureImpl_IDispatch_AddRef,
//   OLEPictureImpl_IDispatch_Release,
//   OLEPictureImpl_GetTypeInfoCount,
//   OLEPictureImpl_GetTypeInfo,
//   OLEPictureImpl_GetIDsOfNames,
//   OLEPictureImpl_Invoke
// };
//
// static const IPersistStreamVtbl OLEPictureImpl_IPersistStream_VTable =
// {
//   OLEPictureImpl_IPersistStream_QueryInterface,
//   OLEPictureImpl_IPersistStream_AddRef,
//   OLEPictureImpl_IPersistStream_Release,
//   OLEPictureImpl_GetClassID,
//   OLEPictureImpl_IsDirty,
//   OLEPictureImpl_Load,
//   OLEPictureImpl_Save,
//   OLEPictureImpl_GetSizeMax
// };
//
// static const IConnectionPointContainerVtbl OLEPictureImpl_IConnectionPointContainer_VTable =
// {
//   OLEPictureImpl_IConnectionPointContainer_QueryInterface,
//   OLEPictureImpl_IConnectionPointContainer_AddRef,
//   OLEPictureImpl_IConnectionPointContainer_Release,
//   OLEPictureImpl_EnumConnectionPoints,
//   OLEPictureImpl_FindConnectionPoint
// };
//
// /***********************************************************************
//  * OleCreatePictureIndirect (OLEAUT32.419)
//  */
// HRESULT WINAPI OleCreatePictureIndirect(LPPICTDESC lpPictDesc, REFIID riid,
// 		            BOOL Own, void **ppvObj )
// {
//   OLEPictureImpl* newPict;
//   HRESULT hr;
//
//   TRACE("(%p,%s,%d,%p)\n", lpPictDesc, debugstr_guid(riid), Own, ppvObj);
//
//   *ppvObj = NULL;
//
//   hr = OLEPictureImpl_Construct(lpPictDesc, Own, &newPict);
//   if (hr != S_OK) return hr;
//
//   /*
//    * Make sure it supports the interface required by the caller.
//    */
//   hr = IPicture_QueryInterface(&newPict->IPicture_iface, riid, ppvObj);
//
//   /*
//    * Release the reference obtained in the constructor. If
//    * the QueryInterface was unsuccessful, it will free the class.
//    */
//   IPicture_Release(&newPict->IPicture_iface);
//
//   return hr;
// }
//
//
// /***********************************************************************
//  * OleLoadPicture (OLEAUT32.418)
//  */
// HRESULT WINAPI OleLoadPicture( LPSTREAM lpstream, LONG lSize, BOOL fRunmode,
// 		            REFIID riid, LPVOID *ppvObj )
// {
//   LPPERSISTSTREAM ps;
//   IPicture	*newpic;
//   HRESULT hr;
//
//   TRACE("(%p,%d,%d,%s,%p), partially implemented.\n",
// 	lpstream, lSize, fRunmode, debugstr_guid(riid), ppvObj);
//
//   hr = OleCreatePictureIndirect(NULL,riid,!fRunmode,(LPVOID*)&newpic);
//   if (hr != S_OK)
//     return hr;
//   hr = IPicture_QueryInterface(newpic,&IID_IPersistStream, (LPVOID*)&ps);
//   if (hr != S_OK) {
//       ERR("Could not get IPersistStream iface from Ole Picture?\n");
//       IPicture_Release(newpic);
//       *ppvObj = NULL;
//       return hr;
//   }
//   hr = IPersistStream_Load(ps,lpstream);
//   IPersistStream_Release(ps);
//   if (FAILED(hr))
//   {
//       ERR("IPersistStream_Load failed\n");
//       IPicture_Release(newpic);
//       *ppvObj = NULL;
//       return hr;
//   }
//   hr = IPicture_QueryInterface(newpic,riid,ppvObj);
//   if (hr != S_OK)
//       ERR("Failed to get interface %s from IPicture.\n",debugstr_guid(riid));
//   IPicture_Release(newpic);
//   return hr;
// }
//
// /***********************************************************************
//  * OleLoadPictureEx (OLEAUT32.401)
//  */
// HRESULT WINAPI OleLoadPictureEx( LPSTREAM lpstream, LONG lSize, BOOL fRunmode,
// 		            REFIID riid, DWORD xsiz, DWORD ysiz, DWORD flags, LPVOID *ppvObj )
// {
//   LPPERSISTSTREAM ps;
//   IPicture	*newpic;
//   HRESULT hr;
//
//   FIXME("(%p,%d,%d,%s,x=%d,y=%d,f=%x,%p), partially implemented.\n",
// 	lpstream, lSize, fRunmode, debugstr_guid(riid), xsiz, ysiz, flags, ppvObj);
//
//   hr = OleCreatePictureIndirect(NULL,riid,!fRunmode,(LPVOID*)&newpic);
//   if (hr != S_OK)
//     return hr;
//   hr = IPicture_QueryInterface(newpic,&IID_IPersistStream, (LPVOID*)&ps);
//   if (hr != S_OK) {
//       ERR("Could not get IPersistStream iface from Ole Picture?\n");
//       IPicture_Release(newpic);
//       *ppvObj = NULL;
//       return hr;
//   }
//   hr = IPersistStream_Load(ps,lpstream);
//   IPersistStream_Release(ps);
//   if (FAILED(hr))
//   {
//       ERR("IPersistStream_Load failed\n");
//       IPicture_Release(newpic);
//       *ppvObj = NULL;
//       return hr;
//   }
//   hr = IPicture_QueryInterface(newpic,riid,ppvObj);
//   if (hr != S_OK)
//       ERR("Failed to get interface %s from IPicture.\n",debugstr_guid(riid));
//   IPicture_Release(newpic);
//   return hr;
// }
//
// static HRESULT create_stream(const WCHAR *filename, IStream **stream)
// {
//     HANDLE hFile;
//     DWORD dwFileSize;
//     HGLOBAL hGlobal = NULL;
//     DWORD dwBytesRead;
//     HRESULT hr = S_OK;
//
//     hFile = CreateFileW(filename, GENERIC_READ, 0, NULL, OPEN_EXISTING, 0, NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//         return HRESULT_FROM_WIN32(GetLastError());
//
//     dwFileSize = GetFileSize(hFile, NULL);
//     if (dwFileSize != INVALID_FILE_SIZE)
//     {
//         hGlobal = GlobalAlloc(GMEM_FIXED, dwFileSize);
//         if (!hGlobal)
//             hr = E_OUTOFMEMORY;
//         else
//         {
//             if (!ReadFile(hFile, hGlobal, dwFileSize, &dwBytesRead, NULL))
//             {
//                 GlobalFree(hGlobal);
//                 hr = HRESULT_FROM_WIN32(GetLastError());
//             }
//         }
//     }
//
//     CloseHandle(hFile);
//
//     if (FAILED(hr)) return hr;
//
//     hr = CreateStreamOnHGlobal(hGlobal, TRUE, stream);
//     if (FAILED(hr))
//         GlobalFree(hGlobal);
//
//     return hr;
// }
//
// /***********************************************************************
//  * OleLoadPictureFile (OLEAUT32.422)
//  */
// HRESULT WINAPI OleLoadPictureFile(VARIANT filename, IDispatch **picture)
// {
//     IStream *stream;
//     HRESULT hr;
//
//     TRACE("(%s,%p)\n", wine_dbgstr_variant(&filename), picture);
//
//     if (V_VT(&filename) != VT_BSTR)
//         return CTL_E_FILENOTFOUND;
//
//     hr = create_stream(V_BSTR(&filename), &stream);
//     if (hr != S_OK)
//     {
//         if (hr == HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND))
//             return CTL_E_FILENOTFOUND;
//
//         return CTL_E_PATHFILEACCESSERROR;
//     }
//
//     hr = OleLoadPicture(stream, 0, FALSE, &IID_IDispatch, (void **)picture);
//     IStream_Release(stream);
//     return hr;
// }
//
// /***********************************************************************
//  * OleSavePictureFile (OLEAUT32.423)
//  */
// HRESULT WINAPI OleSavePictureFile(IDispatch *picture, BSTR filename)
// {
//   FIXME("(%p %s): stub\n", picture, debugstr_w(filename));
//   return CTL_E_FILENOTFOUND;
// }
//
// /***********************************************************************
//  * OleLoadPicturePath (OLEAUT32.424)
//  */
// HRESULT WINAPI OleLoadPicturePath( LPOLESTR szURLorPath, LPUNKNOWN punkCaller,
// 		DWORD dwReserved, OLE_COLOR clrReserved, REFIID riid,
// 		LPVOID *ppvRet )
// {
//   static const WCHAR file[] = { 'f','i','l','e',':',0 };
//   IStream *stream;
//   HRESULT hRes;
//   WCHAR *file_candidate;
//   WCHAR path_buf[MAX_PATH];
//
//   TRACE("(%s,%p,%d,%08x,%s,%p): stub\n",
//         debugstr_w(szURLorPath), punkCaller, dwReserved, clrReserved,
//         debugstr_guid(riid), ppvRet);
//
//   if (!szURLorPath || !ppvRet)
//       return E_INVALIDARG;
//
//   *ppvRet = NULL;
//
//   /* Convert file URLs to DOS paths. */
//   if (wcsncmp(szURLorPath, file, 5) == 0) {
//       DWORD size;
//       hRes = CoInternetParseUrl(szURLorPath, PARSE_PATH_FROM_URL, 0, path_buf,
//                                 ARRAY_SIZE(path_buf), &size, 0);
//       if (FAILED(hRes))
//           return hRes;
//
//       file_candidate = path_buf;
//   }
//   else
//       file_candidate = szURLorPath;
//
//   /* Handle candidate DOS paths separately. */
//   if (file_candidate[1] == ':') {
//       hRes = create_stream(file_candidate, &stream);
//       if (FAILED(hRes))
// 	  return INET_E_RESOURCE_NOT_FOUND;
//   } else {
//       IMoniker *pmnk;
//       IBindCtx *pbc;
//
//       hRes = CreateBindCtx(0, &pbc);
//       if (SUCCEEDED(hRes)) 
//       {
// 	  hRes = CreateURLMoniker(NULL, szURLorPath, &pmnk);
// 	  if (SUCCEEDED(hRes))
// 	  {	         
// 	      hRes = IMoniker_BindToStorage(pmnk, pbc, NULL, &IID_IStream, (LPVOID*)&stream);
// 	      IMoniker_Release(pmnk);
// 	  }
// 	  IBindCtx_Release(pbc);
//       }
//       if (FAILED(hRes))
// 	  return hRes;
//   }
//
//   hRes = OleLoadPicture(stream, 0, FALSE, riid, ppvRet);
//
//   IStream_Release(stream);
//
//   return hRes;
// }
//
// /*******************************************************************************
//  * StdPic ClassFactory
//  */
// typedef struct
// {
//     /* IUnknown fields */
//     IClassFactory IClassFactory_iface;
//     LONG          ref;
// } IClassFactoryImpl;
//
// static inline IClassFactoryImpl *impl_from_IClassFactory(IClassFactory *iface)
// {
//        return CONTAINING_RECORD(iface, IClassFactoryImpl, IClassFactory_iface);
// }
//
// static HRESULT WINAPI
// SPCF_QueryInterface(LPCLASSFACTORY iface,REFIID riid,LPVOID *ppobj) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
//
// 	FIXME("(%p)->(%s,%p),stub!\n",This,debugstr_guid(riid),ppobj);
// 	return E_NOINTERFACE;
// }
//
// static ULONG WINAPI
// SPCF_AddRef(LPCLASSFACTORY iface) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	return InterlockedIncrement(&This->ref);
// }
//
// static ULONG WINAPI SPCF_Release(LPCLASSFACTORY iface) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	/* static class, won't be  freed */
// 	return InterlockedDecrement(&This->ref);
// }
//
// static HRESULT WINAPI SPCF_CreateInstance(
// 	LPCLASSFACTORY iface,LPUNKNOWN pOuter,REFIID riid,LPVOID *ppobj
// ) {
//     /* Creates an uninitialized picture */
//     return OleCreatePictureIndirect(NULL,riid,TRUE,ppobj);
//
// }
//
// static HRESULT WINAPI SPCF_LockServer(LPCLASSFACTORY iface,BOOL dolock) {
// 	IClassFactoryImpl *This = impl_from_IClassFactory(iface);
// 	FIXME("(%p)->(%d),stub!\n",This,dolock);
// 	return S_OK;
// }
//
// static const IClassFactoryVtbl SPCF_Vtbl = {
// 	SPCF_QueryInterface,
// 	SPCF_AddRef,
// 	SPCF_Release,
// 	SPCF_CreateInstance,
// 	SPCF_LockServer
// };
// static IClassFactoryImpl STDPIC_CF = {{&SPCF_Vtbl}, 1 };
//
// void _get_STDPIC_CF(LPVOID *ppv) { *ppv = &STDPIC_CF; }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
