use crate::prelude::*;
use core::ffi::c_void;


























impl Image {
    pub unsafe fn __ctor_0(&mut self, stream: *mut IStream, useEmbeddedColorManagement: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).nativeImage = core::ptr::null_mut();
        {
            if useEmbeddedColorManagement != 0 {
                (*this_).lastStatus = GdipLoadImageFromStreamICM(stream, core::ptr::addr_of_mut!((*this_).nativeImage));
            } else {
                (*this_).lastStatus = GdipLoadImageFromStream(stream, core::ptr::addr_of_mut!((*this_).nativeImage));
            }
        }
    }
    pub unsafe fn __ctor_1(&mut self, filename: *const u16, useEmbeddedColorManagement: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).nativeImage = core::ptr::null_mut();
        {
            if useEmbeddedColorManagement != 0 {
                (*this_).lastStatus = GdipLoadImageFromFileICM(filename, core::ptr::addr_of_mut!((*this_).nativeImage));
            } else {
                (*this_).lastStatus = GdipLoadImageFromFile(filename, core::ptr::addr_of_mut!((*this_).nativeImage));
            }
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        GdipDisposeImage((*this_).nativeImage);
    }
    pub unsafe fn GetHeight(&mut self) -> UINT {
        let this_: *mut Self = self;
        let mut height: UINT = 0;
        (*this_).SetStatus(GdipGetImageHeight((*this_).nativeImage, core::ptr::addr_of_mut!(height)));
        return height;
    }
    pub unsafe fn GetLastStatus(&mut self) -> Gdiplus_Status {
        let this_: *mut Self = self;
        (*this_).lastStatus
    }
    pub unsafe fn GetWidth(&mut self) -> UINT {
        let this_: *mut Self = self;
        let mut width: UINT = 0;
        (*this_).SetStatus(GdipGetImageWidth((*this_).nativeImage, core::ptr::addr_of_mut!(width)));
        return width;
    }
    pub unsafe fn __ctor_2(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn __ctor_3(&mut self, image: *mut GpImage, status: Status) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
        (*this_).__init_vtables();
        (*this_).nativeImage = image;
        (*this_).lastStatus = status;
        {
        }
    }
    pub unsafe fn SetStatus(&mut self, status: Status) -> Status {
        let this_: *mut Self = self;
        if status != Gdiplus_Status_Ok {
            (*this_).lastStatus = status;
        }
        return status;
    }
    pub unsafe fn SetNativeImage(&mut self, image: *mut Gdiplus_GpImage) {
        let this_: *mut Self = self;
        (*this_).nativeImage = image;
    }
}

impl Bitmap {
    pub unsafe fn __ctor_0(&mut self, width: INT, height: INT, target: *mut Graphics) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromGraphics(width, height, if !target.is_null() { (*target).nativeGraphics } else { core::ptr::null_mut() }, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_1(&mut self, gdiBitmapInfo: *const BITMAPINFO, gdiBitmapData: *mut core::ffi::c_void) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromGdiDib(gdiBitmapInfo, gdiBitmapData, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_2(&mut self, width: INT, height: INT, format: Gdiplus_PixelFormat) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromScan0(width, height, 0, format, core::ptr::null_mut(), core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_3(&mut self, hbm: HBITMAP, hpal: HPALETTE) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromHBITMAP(hbm, hpal, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_4(&mut self, width: INT, height: INT, stride: INT, format: Gdiplus_PixelFormat, scan0: *mut BYTE) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromScan0(width, height, stride, format, scan0, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_5(&mut self, filename: *const WCHAR, useIcm: BOOL) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();

            if useIcm != 0 {
                (*this_).__base0.lastStatus = GdipCreateBitmapFromFileICM(filename, core::ptr::addr_of_mut!(bitmap));
            } else {
                (*this_).__base0.lastStatus = GdipCreateBitmapFromFile(filename, core::ptr::addr_of_mut!(bitmap));
            }

            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_6(&mut self, hInstance: HINSTANCE, bitmapName: *const u16) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromResource(hInstance, bitmapName, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_7(&mut self, hicon: HICON) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            (*this_).__base0.lastStatus = GdipCreateBitmapFromHICON(hicon, core::ptr::addr_of_mut!(bitmap));
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn __ctor_8(&mut self, stream: *mut IStream, useIcm: i32) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            let mut bitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
            if useIcm != 0 {
                (*this_).__base0.lastStatus = GdipCreateBitmapFromStreamICM(stream, core::ptr::addr_of_mut!(bitmap));
            } else {
                (*this_).__base0.lastStatus = GdipCreateBitmapFromStream(stream, core::ptr::addr_of_mut!(bitmap));
            }
            (*this_).__base0.SetNativeImage((bitmap).cast());
        }
    }
    pub unsafe fn FromFile(filename: *const WCHAR,
        useEmbeddedColorManagement: BOOL,
    ) -> *mut Bitmap {
        Bitmap__new_5(filename, useEmbeddedColorManagement)
    }
    pub unsafe fn __ctor_9(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn __ctor_10(&mut self, nativeBitmap: *mut Gdiplus_GpBitmap) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_2();
        (*this_).__init_vtables();
        {
            (*this_).__base0.lastStatus = 0; // Ok
            (*this_).__base0.SetNativeImage((nativeBitmap).cast());
        }
    }
}
