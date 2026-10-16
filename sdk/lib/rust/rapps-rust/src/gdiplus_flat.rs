//! gdiplusflat.h's flat API, which the GDI+ C++ wrappers (gdiplus*.h, modeled by cppapp as
//! external classes) call as `DllExports::GdipX`. The C++ island has no binding for them:
//! they are declared only inside `namespace Gdiplus::DllExports`, which no module code names
//! directly. These are the C prototypes exactly as bindgen binds them for desk-rust's C island
//! (identical on i386, amd64 and arm64), limited to the functions rapps reaches.
#![allow(non_snake_case)]
use crate::prelude::*;

/// gdiplusinit.h's `GdiplusStartup` is declared outside DllExports; the island has it.
pub use crate::types::Gdiplus_GdiplusStartup as GdiplusStartup;

/// gdiplusgpstubs.h `typedef Status GpStatus;`
type GpStatus = Status;

unsafe extern "system" {
    pub fn GdipCreateBitmapFromFile(arg1: *const WCHAR, arg2: *mut *mut GpBitmap) -> GpStatus;
    pub fn GdipCreateBitmapFromFileICM(arg1: *const WCHAR, arg2: *mut *mut GpBitmap) -> GpStatus;
    pub fn GdipCreateBitmapFromGdiDib(
        arg1: *const BITMAPINFO,
        arg2: *mut ::core::ffi::c_void,
        arg3: *mut *mut GpBitmap,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromGraphics(
        arg1: INT,
        arg2: INT,
        arg3: *mut GpGraphics,
        arg4: *mut *mut GpBitmap,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromHBITMAP(
        arg1: HBITMAP,
        arg2: HPALETTE,
        arg3: *mut *mut GpBitmap,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromHICON(arg1: HICON, arg2: *mut *mut GpBitmap) -> GpStatus;
    pub fn GdipCreateBitmapFromResource(
        arg1: HINSTANCE,
        arg2: *const WCHAR,
        arg3: *mut *mut GpBitmap,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromScan0(
        arg1: INT,
        arg2: INT,
        arg3: INT,
        arg4: PixelFormat,
        arg5: *mut BYTE,
        arg6: *mut *mut GpBitmap,
    ) -> GpStatus;
    pub fn GdipCreateBitmapFromStream(arg1: *mut IStream, arg2: *mut *mut GpBitmap) -> GpStatus;
    pub fn GdipCreateBitmapFromStreamICM(arg1: *mut IStream, arg2: *mut *mut GpBitmap) -> GpStatus;
    pub fn GdipCreateFromHDC(arg1: HDC, arg2: *mut *mut GpGraphics) -> GpStatus;
    pub fn GdipCreateFromHDC2(arg1: HDC, arg2: HANDLE, arg3: *mut *mut GpGraphics) -> GpStatus;
    pub fn GdipCreateFromHWND(arg1: HWND, arg2: *mut *mut GpGraphics) -> GpStatus;
    pub fn GdipCreateFromHWNDICM(arg1: HWND, arg2: *mut *mut GpGraphics) -> GpStatus;
    pub fn GdipCreateSolidFill(arg1: ARGB, arg2: *mut *mut GpSolidFill) -> GpStatus;
    pub fn GdipDeleteBrush(arg1: *mut GpBrush) -> GpStatus;
    pub fn GdipDeleteGraphics(arg1: *mut GpGraphics) -> GpStatus;
    pub fn GdipDisposeImage(arg1: *mut GpImage) -> GpStatus;
    pub fn GdipDrawImageRect(
        arg1: *mut GpGraphics,
        arg2: *mut GpImage,
        arg3: REAL,
        arg4: REAL,
        arg5: REAL,
        arg6: REAL,
    ) -> GpStatus;
    pub fn GdipFillEllipse(
        arg1: *mut GpGraphics,
        arg2: *mut GpBrush,
        arg3: REAL,
        arg4: REAL,
        arg5: REAL,
        arg6: REAL,
    ) -> GpStatus;
    pub fn GdipGetImageGraphicsContext(arg1: *mut GpImage, arg2: *mut *mut GpGraphics) -> GpStatus;
    pub fn GdipGetImageHeight(arg1: *mut GpImage, arg2: *mut UINT) -> GpStatus;
    pub fn GdipGetImageWidth(arg1: *mut GpImage, arg2: *mut UINT) -> GpStatus;
    pub fn GdipLoadImageFromFile(arg1: *const WCHAR, arg2: *mut *mut GpImage) -> GpStatus;
    pub fn GdipLoadImageFromFileICM(arg1: *const WCHAR, arg2: *mut *mut GpImage) -> GpStatus;
    pub fn GdipLoadImageFromStream(arg1: *mut IStream, arg2: *mut *mut GpImage) -> GpStatus;
    pub fn GdipLoadImageFromStreamICM(arg1: *mut IStream, arg2: *mut *mut GpImage) -> GpStatus;
    pub fn GdipSetSmoothingMode(arg1: *mut GpGraphics, arg2: SmoothingMode) -> GpStatus;
    pub fn GdiplusShutdown(arg1: ULONG_PTR);
}
