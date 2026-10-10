use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_37BB6F4CC6ED: [WCHAR; 12] = [0x0067, 0x0064, 0x0069, 0x0070, 0x006C, 0x0075, 0x0073, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

extern "system" {
    fn PathFindExtensionW(pszPath: LPCWSTR) -> *mut u16;
}




















impl CImageDx {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hBitmap = core::ptr::null_mut();
        {
            (*CImageDx::_shared()).AddRef();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_hBitmap as usize != 0 {
            DeleteObject(((*this_).m_hBitmap).cast());
        }
    
        (*CImageDx::_shared()).Release();
    }
    pub unsafe fn Attach(&mut self, hbm: HBITMAP) {
        let this_: *mut Self = self;
            if !(*this_).m_hBitmap.is_null() {
                DeleteObject(((*this_).m_hBitmap).cast());
            }
            (*this_).m_hBitmap = hbm;
        }
    pub unsafe fn Detach(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        let hbmOld: HBITMAP = (*this_).m_hBitmap;
        (*this_).m_hBitmap = core::ptr::null_mut();
        return hbmOld;
    }
    pub unsafe fn GetResolution(&mut self, pImage: *mut Gdiplus_GpImage, pxDpi: *mut f32, pyDpi: *mut f32) -> BOOL {
        let this_: *mut Self = self;
        if CImageDx::get_fn(&mut (*CImageDx::_shared()).m_GetImageHorizontalResolution, b"GdipGetImageHorizontalResolution\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_GetImageVerticalResolution, b"GdipGetImageVerticalResolution\0".as_ptr() as *const i8) == false
        {
            return (FALSE) as BOOL;
        }
    
        if !pxDpi.is_null() {
            (((*CImageDx::_shared()).m_GetImageHorizontalResolution).unwrap_unchecked())(pImage, pxDpi);
        }
        if !pyDpi.is_null() {
            (((*CImageDx::_shared()).m_GetImageVerticalResolution).unwrap_unchecked())(pImage, pyDpi);
        }
    
        return (TRUE) as BOOL;
    }
    pub unsafe fn SetResolution(&mut self, pBitmap: *mut Gdiplus_GpBitmap, xDpi: f32, yDpi: f32) -> BOOL {
        let this_: *mut Self = self;
        if !CImageDx::get_fn(&mut (*CImageDx::_shared()).m_BitmapSetResolution, b"GdipBitmapSetResolution\0".as_ptr() as *const i8) {
            return (FALSE) as BOOL;
        }
    
        (((*CImageDx::_shared()).m_BitmapSetResolution).unwrap_unchecked())(pBitmap, xDpi, yDpi);
        return (TRUE) as BOOL;
    }
    pub unsafe fn LoadDx(&mut self, pszFileName: LPCWSTR, pxDpi: *mut f32, pyDpi: *mut f32) -> HRESULT {
        let this_: *mut Self = self;
        (*CImageDx::_shared()).AddRef();
    
        if CImageDx::get_fn(&mut (*CImageDx::_shared()).m_CreateBitmapFromFile, b"GdipCreateBitmapFromFile\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_CreateHBITMAPFromBitmap, b"GdipCreateHBITMAPFromBitmap\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_DisposeImage, b"GdipDisposeImage\0".as_ptr() as *const i8) == false
        {
            (*CImageDx::_shared()).Release();
            return E_FAIL;
        }
    
        // create a Gdiplus_GpBitmap object from file
        let mut pBitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
        if (((*CImageDx::_shared()).m_CreateBitmapFromFile).unwrap_unchecked())(pszFileName, core::ptr::addr_of_mut!(pBitmap)) != 0 /* Ok */
        {
            (*CImageDx::_shared()).Release();
            return E_FAIL;
        }
    
        // get an HBITMAP
        let mut hbm: HBITMAP = core::ptr::null_mut();
        // Gdiplus::Color(255, 255, 255): MakeARGB(0xFF, r, g, b), opaque white.
        let color: DWORD = 0xFFFF_FFFF; // Gdiplus::ARGB is a DWORD
        let status: Gdiplus_Status = (((*CImageDx::_shared()).m_CreateHBITMAPFromBitmap).unwrap_unchecked())(pBitmap, core::ptr::addr_of_mut!(hbm), color);
    
        // get the resolution
        if !pxDpi.is_null() || !pyDpi.is_null() {
            (*this_).GetResolution(pBitmap as *mut Gdiplus_GpImage, pxDpi, pyDpi);
        }
    
        // delete Gdiplus_GpBitmap
        (((*CImageDx::_shared()).m_DisposeImage).unwrap_unchecked())((pBitmap).cast());
    
        // attach it
        if status == 0 /* Ok */ {
            (*this_).Attach(hbm);
        }
    
        (*CImageDx::_shared()).Release();
        return if status == 0 /* Ok */ { S_OK } else { E_FAIL };
    }
    pub unsafe fn SaveDx(&mut self, pszFileName: LPCWSTR, guidFileType: &GUID, xDpi: f32, yDpi: f32) -> HRESULT {
        let this_: *mut Self = self;
        const Ok: u32 = 0; // Gdiplus Ok
    
        (*CImageDx::_shared()).AddRef();
    
        if CImageDx::get_fn(&mut (*CImageDx::_shared()).m_CreateBitmapFromHBITMAP, b"GdipCreateBitmapFromHBITMAP\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_SaveImageToFile, b"GdipSaveImageToFile\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_DisposeImage, b"GdipDisposeImage\0".as_ptr() as *const i8) == false
        {
            (*CImageDx::_shared()).Release();
            return E_FAIL;
        }
    
        // create a Gdiplus_GpBitmap from HBITMAP
        let mut pBitmap: *mut Gdiplus_GpBitmap = core::ptr::null_mut();
        (((*CImageDx::_shared()).m_CreateBitmapFromHBITMAP).unwrap_unchecked())((*this_).m_hBitmap, core::ptr::null_mut(), core::ptr::addr_of_mut!(pBitmap));
    
        // set the resolution
        (*this_).SetResolution(pBitmap, xDpi, yDpi);
    
        // Get encoders
        let mut cEncoders: UINT = 0;
        let mut pEncoders: *mut Gdiplus_ImageCodecInfo = CImageDx::GetAllEncoders(&mut cEncoders);
    
        // if the file type is null, get the file type from extension
        let mut clsid: CLSID = core::mem::zeroed();
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(GUID_NULL)) != false
        {
            let mut strExt = CStringW::from_ptr(PathFindExtensionW(pszFileName));
            clsid = CImageDx::FindCodecForExtension(strExt.GetString(), pEncoders, cEncoders);
        }
        else
        {
            clsid = CImageDx::FindCodecForFileType(guidFileType, pEncoders, cEncoders);
        }
    
        cpp_delete_array(pEncoders);
    
        // save to file
        let status: Gdiplus_Status = (((*CImageDx::_shared()).m_SaveImageToFile).unwrap_unchecked())((pBitmap).cast(), pszFileName, core::ptr::addr_of_mut!(clsid), core::ptr::null_mut());
    
        // destroy Gdiplus_GpBitmap
        (((*CImageDx::_shared()).m_DisposeImage).unwrap_unchecked())((pBitmap).cast());
    
        (*CImageDx::_shared()).Release();
    
        return if status == Ok { S_OK } else { E_FAIL };
    }
    pub unsafe fn IsExtensionSupported(pchDotExt: PWCHAR) -> BOOL {
        (*CImageDx::_shared()).AddRef();
    
        let mut cEncoders: UINT = 0;
        let pEncoders: *mut Gdiplus_ImageCodecInfo = CImageDx::GetAllEncoders(&mut cEncoders);
    
        let clsid: CLSID = CImageDx::FindCodecForExtension(pchDotExt, pEncoders, cEncoders);
        let ret: BOOL = !IsEqualGUID!(core::ptr::addr_of!(clsid), core::ptr::addr_of!(GUID_NULL)) as BOOL;
        cpp_delete_array(pEncoders);
    
        (*CImageDx::_shared()).Release();
        return ret;
    }
    pub unsafe fn _shared() -> *mut SHARED {
        static mut S_SHARED_INIT: u8 = 0;
        static mut S_SHARED: core::mem::MaybeUninit<SHARED> = core::mem::MaybeUninit::uninit();
        if S_SHARED_INIT == 0 {
            S_SHARED = core::mem::MaybeUninit::new(SHARED__make_0());
            S_SHARED_INIT = 1;
        }
        return core::ptr::addr_of_mut!(S_SHARED) as *mut SHARED;
    }
    pub unsafe fn GetAllEncoders(cEncoders: &mut u32) -> *mut Gdiplus_ImageCodecInfo {
        let mut ret: *mut Gdiplus_ImageCodecInfo = core::ptr::null_mut();
        let mut total_size: u32 = 0;
    
        if CImageDx::get_fn(&mut (*CImageDx::_shared()).m_GetImageEncodersSize, b"GdipGetImageEncodersSize\0".as_ptr() as *const i8) == false ||
            CImageDx::get_fn(&mut (*CImageDx::_shared()).m_GetImageEncoders, b"GdipGetImageEncoders\0".as_ptr() as *const i8) == false
        {
            *cEncoders = 0;
            return core::ptr::null_mut();
        }
    
        (((*CImageDx::_shared()).m_GetImageEncodersSize).unwrap_unchecked())(cEncoders as *mut u32, core::ptr::addr_of_mut!(total_size));
        if total_size == 0 {
            return core::ptr::null_mut();
        }
    
        let _ = 0;
        ret = cpp_new_array::<Gdiplus_ImageCodecInfo>(
            (total_size as usize) / core::mem::size_of::<Gdiplus_ImageCodecInfo>()
        );
        if ret.is_null() {
            *cEncoders = 0;
            return core::ptr::null_mut();
        }
    
        (((*CImageDx::_shared()).m_GetImageEncoders).unwrap_unchecked())(*cEncoders, total_size, ret);
    
        ret // needs delete[]
    }
    pub unsafe fn FindCodecForExtension(dotext: LPCWSTR, pCodecs: *const Gdiplus_ImageCodecInfo, nCodecs: UINT) -> CLSID {
        let mut i: UINT = 0;
        while i < nCodecs {
            let mut strSpecs = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
            let mut ichOld: i32 = 0;
            let mut ichSep: i32;
            loop {
                ichSep = strSpecs.Find(0x003Bu16, ichOld);
    
                let mut strSpec = CStringW::new();
                if ichSep < 0 {
                    strSpec = strSpecs.Mid(ichOld);
                } else {
                    strSpec = strSpecs.Mid__1(ichOld, ichSep - ichOld);
                }
    
                let mut ichDot: i32 = strSpec.ReverseFind(0x002Eu16);
                if ichDot >= 0 {
                    strSpec = strSpec.Mid(ichDot);
                }
    
                if dotext.is_null() || strSpec.CompareNoCase(dotext) == 0 {
                    return (*pCodecs.add(i as usize)).Clsid;
                }
    
                if ichSep < 0 {
                    break;
                }
    
                ichOld = ichSep + 1;
            }
            i += 1;
        }
        return GUID_NULL;
    }
    pub unsafe fn FindCodecForFileType(guidFileType: *const GUID, pCodecs: *const Gdiplus_ImageCodecInfo, nCodecs: UINT) -> CLSID {
        let mut iInfo: UINT = 0;
        while iInfo < nCodecs {
            if IsEqualGUID!(core::ptr::addr_of!((*pCodecs.add(iInfo as usize)).FormatID), guidFileType) != false {
                return (*pCodecs.add(iInfo as usize)).Clsid;
            }
            iInfo += 1;
        }
        return GUID_NULL;
    }
}

impl SHARED {
    pub unsafe fn Init(&mut self) -> HINSTANCE {
        let this_: *mut Self = self;
        if !(*this_).m_hGdiPlus.is_null() {
            return (*this_).m_hGdiPlus;
        }
    
        (*this_).m_hGdiPlus = LoadLibraryW(CPP_COM_WSTR_37BB6F4CC6ED.as_ptr());
        if (*this_).m_hGdiPlus.is_null() {
            return core::ptr::null_mut();
        }
    
        let Startup: CImageDx_FN_Startup = core::mem::transmute::<_, CImageDx_FN_Startup>(
            GetProcAddress((*this_).m_hGdiPlus, b"GdiplusStartup\0".as_ptr() as *const i8),
        );
        (*this_).m_Shutdown = core::mem::transmute::<_, CImageDx_FN_Shutdown>(
            GetProcAddress((*this_).m_hGdiPlus, b"GdiplusShutdown\0".as_ptr() as *const i8),
        );
        if (Startup).map_or(0, |f| f as usize) as usize == 0 || ((*this_).m_Shutdown).map_or(0, |f| f as usize) as usize == 0 {
            FreeLibrary((*this_).m_hGdiPlus);
            (*this_).m_hGdiPlus = core::ptr::null_mut();
            return core::ptr::null_mut();
        }
    
        let mut gdiplusStartupInput: Gdiplus_GdiplusStartupInput = core::mem::zeroed();
        (Startup.unwrap_unchecked())(
            core::ptr::addr_of_mut!((*this_).m_dwToken),
            core::ptr::addr_of_mut!(gdiplusStartupInput),
            core::ptr::null_mut(),
        );
    
        return (*this_).m_hGdiPlus;
    }
    pub unsafe fn Free(&mut self) {
        let this_: *mut Self = self;
        FreeLibrary((*this_).m_hGdiPlus);
        ZeroMemory!(this_ as *mut core::ffi::c_void, core::mem::size_of::<SHARED>());
    }
    pub unsafe fn AddRef(&mut self) -> LONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs += 1;
        return (*this_).m_cRefs;
    }
    pub unsafe fn Release(&mut self) -> LONG {
        let this_: *mut Self = self;
        (*this_).m_cRefs -= 1;
        if (*this_).m_cRefs == 0 {
            (*this_).Free();
            return 0;
        }
        return (*this_).m_cRefs;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hGdiPlus = core::ptr::null_mut();
        (*this_).m_cRefs = 0;
        (*this_).m_dwToken = 0;
        (*this_).m_Shutdown = core::mem::zeroed();
        (*this_).m_GetImageHorizontalResolution = core::mem::zeroed();
        (*this_).m_GetImageVerticalResolution = core::mem::zeroed();
        (*this_).m_BitmapSetResolution = core::mem::zeroed();
        (*this_).m_CreateBitmapFromHBITMAP = core::mem::zeroed();
        (*this_).m_CreateBitmapFromFile = core::mem::zeroed();
        (*this_).m_CreateHBITMAPFromBitmap = core::mem::zeroed();
        (*this_).m_SaveImageToFile = core::mem::zeroed();
        (*this_).m_DisposeImage = core::mem::zeroed();
        (*this_).m_GetImageEncodersSize = core::mem::zeroed();
        (*this_).m_GetImageEncoders = core::mem::zeroed();
    }
}

impl CImageDx {
    /// atlimagedx.h's member TEMPLATE `template <typename FN_T> static bool get_fn(FN_T& fn,
    /// const char *name)`, hand-ported (cppapp does not generate templates): resolve `fn`
    /// from gdiplus.dll on first use. `F` is the field's function-pointer type; FARPROC and
    /// it are the same size, as the C++ reinterpret_cast assumes.
    pub unsafe fn get_fn<F: Copy>(fn_: &mut Option<F>, name: *const i8) -> bool {
        if fn_.is_some() {
            return true;
        }
        let hGdiPlus: HINSTANCE = (*CImageDx::_shared()).Init();
        let p = GetProcAddress(hGdiPlus as _, name as _);
        *fn_ = core::mem::transmute_copy(&p);
        fn_.is_some()
    }
}
