// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn DibLoadImage(lpFilename: LPTSTR) -> PDIBITMAP {
    let mut lpBitmap: PDIBITMAP;
    let mut bitmap: *mut GpBitmap = core::ptr::null_mut();
    let mut lock: BitmapData = core::mem::zeroed();

    if GdipCreateBitmapFromFile(lpFilename, &mut bitmap) != Ok {
        return core::ptr::null_mut();
    }

    lpBitmap = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<DIBITMAP>()) as SIZE_T) as PDIBITMAP;
    if lpBitmap.is_null() {
        GdipDisposeImage(bitmap as *mut GpImage);
        return core::ptr::null_mut();
    }

    (*lpBitmap).info = HeapAlloc(GetProcessHeap(), 0, (core::mem::size_of::<BITMAPINFO>()) as SIZE_T) as *mut BITMAPINFO;
    if (*lpBitmap).info.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBitmap as *mut core::ffi::c_void);
        GdipDisposeImage(bitmap as *mut GpImage);
        return core::ptr::null_mut();
    }

    if GdipGetImageWidth(bitmap as *mut GpImage, &mut (*lpBitmap).width) != Ok ||
        GdipGetImageHeight(bitmap as *mut GpImage, &mut (*lpBitmap).height) != Ok {
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, lpBitmap as *mut core::ffi::c_void);
        GdipDisposeImage(bitmap as *mut GpImage);
        return core::ptr::null_mut();
    }

    (*lpBitmap).bits = (HeapAlloc(GetProcessHeap(), 0, ((*lpBitmap).width * (*lpBitmap).height * 4) as SIZE_T) as *mut u32).cast();
    if (*lpBitmap).bits.is_null() {
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, lpBitmap as *mut core::ffi::c_void);
        GdipDisposeImage(bitmap as *mut GpImage);
        return core::ptr::null_mut();
    }

    ZeroMemory!((*lpBitmap).info as *mut core::ffi::c_void, core::mem::size_of::<BITMAPINFO>());
    (*(*lpBitmap).info).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    (*(*lpBitmap).info).bmiHeader.biWidth = (*lpBitmap).width as i32;
    (*(*lpBitmap).info).bmiHeader.biHeight = -((*lpBitmap).height as i32);
    (*(*lpBitmap).info).bmiHeader.biPlanes = 1;
    (*(*lpBitmap).info).bmiHeader.biBitCount = 32;
    (*(*lpBitmap).info).bmiHeader.biCompression = BI_RGB;
    (*(*lpBitmap).info).bmiHeader.biSizeImage = (*lpBitmap).width * (*lpBitmap).height * 4;

    lock.Width = (*lpBitmap).width;
    lock.Height = (*lpBitmap).height;
    lock.Stride = ((*lpBitmap).width * 4) as i32;
    lock.PixelFormat = (PixelFormat32bppPARGB) as i32;
    lock.Scan0 = (*lpBitmap).bits as *mut core::ffi::c_void;
    lock.Reserved = 0;

    if GdipBitmapLockBits(bitmap, core::ptr::null_mut(), ImageLockModeRead | ImageLockModeUserInputBuf, (PixelFormat32bppPARGB) as PixelFormat, &mut lock) != Ok {
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).bits as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).info as *mut core::ffi::c_void);
        HeapFree(GetProcessHeap(), 0, lpBitmap as *mut core::ffi::c_void);
        GdipDisposeImage(bitmap as *mut GpImage);
        return core::ptr::null_mut();
    }

    GdipBitmapUnlockBits(bitmap, &mut lock);
    GdipDisposeImage(bitmap as *mut GpImage);

    return lpBitmap;
}
#[no_mangle]
pub unsafe extern "C" fn DibFreeImage(lpBitmap: PDIBITMAP) {
    if lpBitmap.is_null() {
        return;
    }

    if !(*lpBitmap).bits.is_null() {
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).bits as *mut _);
    }

    if !(*lpBitmap).info.is_null() {
        HeapFree(GetProcessHeap(), 0, (*lpBitmap).info as *mut _);
    }

    if !lpBitmap.is_null() {
        HeapFree(GetProcessHeap(), 0, lpBitmap as *mut _);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Display Control Panel
//  * FILE:            dll/cpl/desk/dibitmap.c
//  * PURPOSE:         DIB loading
//  *
//  * PROGRAMMERS:     Trevor McCort (lycan359@gmail.com)
//  */
//
// #include "desk.h"
//
// PDIBITMAP
// DibLoadImage(LPTSTR lpFilename)
// {
//     PDIBITMAP lpBitmap;
//     GpBitmap  *bitmap;
//     BitmapData lock;
//
//     if (GdipCreateBitmapFromFile(lpFilename, &bitmap) != Ok)
//     {
//         return NULL;
//     }
//
//     lpBitmap = HeapAlloc(GetProcessHeap(), 0, sizeof(DIBITMAP));
//     if (lpBitmap == NULL)
//     {
//         GdipDisposeImage((GpImage*)bitmap);
//         return NULL;
//     }
//
//     lpBitmap->info = HeapAlloc(GetProcessHeap(), 0, sizeof(BITMAPINFO));
//     if (lpBitmap->info == NULL)
//     {
//         HeapFree(GetProcessHeap(), 0, lpBitmap);
//         GdipDisposeImage((GpImage*)bitmap);
//         return NULL;
//     }
//
//     if (GdipGetImageWidth((GpImage*)bitmap, &lpBitmap->width) != Ok ||
//         GdipGetImageHeight((GpImage*)bitmap, &lpBitmap->height) != Ok)
//     {
//         HeapFree(GetProcessHeap(), 0, lpBitmap->info);
//         HeapFree(GetProcessHeap(), 0, lpBitmap);
//         GdipDisposeImage((GpImage*)bitmap);
//         return NULL;
//     }
//
//     lpBitmap->bits = HeapAlloc(GetProcessHeap(), 0, lpBitmap->width * lpBitmap->height * 4);
//     if (!lpBitmap->bits)
//     {
//         HeapFree(GetProcessHeap(), 0, lpBitmap->info);
//         HeapFree(GetProcessHeap(), 0, lpBitmap);
//         GdipDisposeImage((GpImage*)bitmap);
//         return NULL;
//     }
//
//     ZeroMemory(lpBitmap->info, sizeof(BITMAPINFO));
//     lpBitmap->info->bmiHeader.biSize        = sizeof(BITMAPINFOHEADER);
//     lpBitmap->info->bmiHeader.biWidth       = lpBitmap->width;
//     lpBitmap->info->bmiHeader.biHeight      = -(INT)lpBitmap->height;
//     lpBitmap->info->bmiHeader.biPlanes      = 1;
//     lpBitmap->info->bmiHeader.biBitCount    = 32;
//     lpBitmap->info->bmiHeader.biCompression = BI_RGB;
//     lpBitmap->info->bmiHeader.biSizeImage   = lpBitmap->width * lpBitmap->height * 4;
//
//     lock.Width = lpBitmap->width;
//     lock.Height = lpBitmap->height;
//     lock.Stride = lpBitmap->width * 4;
//     lock.PixelFormat = PixelFormat32bppPARGB;
//     lock.Scan0  = lpBitmap->bits;
//     lock.Reserved = 0;
//
//     if (GdipBitmapLockBits(bitmap, NULL, ImageLockModeRead | ImageLockModeUserInputBuf, PixelFormat32bppPARGB, &lock) != Ok)
//     {
//         HeapFree(GetProcessHeap(), 0, lpBitmap->bits);
//         HeapFree(GetProcessHeap(), 0, lpBitmap->info);
//         HeapFree(GetProcessHeap(), 0, lpBitmap);
//         GdipDisposeImage((GpImage*)bitmap);
//         return NULL;
//     }
//
//     GdipBitmapUnlockBits(bitmap, &lock);
//     GdipDisposeImage((GpImage*)bitmap);
//
//     return lpBitmap;
// }
//
//
// VOID
// DibFreeImage(PDIBITMAP lpBitmap)
// {
//     if (lpBitmap == NULL)
//         return;
//
//     /* Free the image data */
//     if (lpBitmap->bits != NULL)
//         HeapFree(GetProcessHeap(), 0, lpBitmap->bits);
//
//     /* Free the bitmap info */
//     if (lpBitmap->info != NULL)
//         HeapFree(GetProcessHeap(), 0, lpBitmap->info);
//
//     /* Free the bitmap structure */
//     if (lpBitmap != NULL)
//         HeapFree(GetProcessHeap(), 0, lpBitmap);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
