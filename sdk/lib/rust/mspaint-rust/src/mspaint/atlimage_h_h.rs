use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_37BB6F4CC6ED: [WCHAR; 12] = [0x0067, 0x0064, 0x0069, 0x0070, 0x006C, 0x0075, 0x0073, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_4F2D892D6AF4: [WCHAR; 2] = [0x0029, 0x0000];
pub static CPP_COM_WSTR_ACBE5EA9B8F8: [WCHAR; 3] = [0x0020, 0x0028, 0x0000];

use core::ffi::c_void;





















impl CImage {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hBitmap = core::ptr::null_mut();
        (*this_).m_hOldBitmap = core::ptr::null_mut();
        (*this_).m_nWidth = 0;
        (*this_).m_nHeight = 0;
        (*this_).m_nPitch = 0;
        (*this_).m_nBPP = 0;
        (*this_).m_pBits = core::ptr::null_mut();
        (*this_).m_bHasAlphaChannel = (FALSE as i32);
        (*this_).m_bIsDIBSection = (FALSE as i32);
        (*this_).m_clrTransparentColor = CLR_INVALID;
        (*this_).m_hDC = core::ptr::null_mut();
        (*this_).m_nDCRefCount = 0;
        {
            CImage__s_gdiplus.IncreaseCImageCount();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).Destroy();
        CImage__s_gdiplus.DecreaseCImageCount();
    }
    pub unsafe fn Detach(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
        (*this_).m_hOldBitmap = core::ptr::null_mut();
        (*this_).m_nWidth = 0;
        (*this_).m_nHeight = 0;
        (*this_).m_nPitch = 0;
        (*this_).m_nBPP = 0;
        (*this_).m_pBits = core::ptr::null_mut();
    
        (*this_).m_bHasAlphaChannel = (FALSE as i32);
        (*this_).m_bIsDIBSection = (FALSE as i32);
        (*this_).m_clrTransparentColor = CLR_INVALID;
    
        let hBitmap: HBITMAP = (*this_).m_hBitmap;
        (*this_).m_hBitmap = core::ptr::null_mut();
        return hBitmap;
    }
    pub unsafe fn Destroy(&mut self) {
        let this_: *mut Self = self;
        if ((*this_).m_hBitmap as usize) != 0 {
            DeleteObject(((*this_).Detach()).cast());
        }
    }
    pub unsafe fn ShouldExcludeFormat(guidFileType: *const GUID, dwExclude: DWORD) -> bool {
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatGIF)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeGIF) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatBMP)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeBMP) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatEMF)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeEMF) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatWMF)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeWMF) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatJPEG)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeJPEG) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatPNG)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludePNG) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatTIFF)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeTIFF) != 0;
        }
        if IsEqualGUID!(guidFileType, core::ptr::addr_of!(Gdiplus_ImageFormatIcon)) != false {
            return (dwExclude & ATL_CImage_ExcludeFlags_excludeIcon) != 0;
        }
        return (dwExclude & ATL_CImage_ExcludeFlags_excludeOther) == ATL_CImage_ExcludeFlags_excludeOther;
    }
    pub unsafe fn BuildCodecFilterString(pCodecs: *const Gdiplus_ImageCodecInfo, cCodecs: UINT, strFilter: *mut CStringW, aguidFileTypes: *mut CSimpleArray<GUID>, pszAllFilesDescription: LPCTSTR, dwExclude: DWORD, chSeparator: TCHAR) -> HRESULT {
        if pCodecs.is_null() || cCodecs == 0 {
            (*strFilter).AppendChar(chSeparator);
            return E_FAIL;
        }
    
        if !pszAllFilesDescription.is_null() {
            (*strFilter) += pszAllFilesDescription;
    
            let mut bFirst: BOOL = (TRUE as i32);
            let mut extensions = CStringW::new();
            let mut i: UINT = 0;
            while i < cCodecs {
                'body: {
                    if CImage::ShouldExcludeFormat(&((*pCodecs.add(i as usize)).FormatID), dwExclude) != false {
                        break 'body;
                    }
    
                    if bFirst != 0 {
                        bFirst = (FALSE as i32);
                    } else {
                        extensions.AppendChar(0x003Bu16);
                    }
    
                    let mut ext = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
                    extensions += ext.GetString();
                }
                i += 1;
            }
    
            (*strFilter).AppendChar(chSeparator);
            (*strFilter) += extensions.GetString();
            (*strFilter).AppendChar(chSeparator);
    
            (*aguidFileTypes).Add(GUID_NULL);
        }
    
        let mut i: UINT = 0;
        while i < cCodecs {
            'body: {
                if CImage::ShouldExcludeFormat(&((*pCodecs.add(i as usize)).FormatID), dwExclude) != false {
                    break 'body;
                }
    
                let mut extensions = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
    
                let mut desc = CStringW::from_ptr((*pCodecs.add(i as usize)).FormatDescription);
                (*strFilter) += desc.GetString();
                (*strFilter) += CPP_COM_WSTR_ACBE5EA9B8F8.as_ptr();
                (*strFilter) += extensions.GetString();
                (*strFilter) += CPP_COM_WSTR_4F2D892D6AF4.as_ptr();
                (*strFilter).AppendChar(chSeparator);
                (*strFilter) += extensions.GetString();
                (*strFilter).AppendChar(chSeparator);
    
                (*aguidFileTypes).Add((*pCodecs.add(i as usize)).FormatID);
            }
            i += 1;
        }
    
        (*strFilter).AppendChar(chSeparator);
        return S_OK;
    }
    pub unsafe fn GetImporterFilterString(strImporters: &mut CSimpleString, aguidFileTypes: &mut CSimpleArray<GUID>, pszAllFilesDescription: LPCTSTR, dwExclude: DWORD, chSeparator: TCHAR) -> HRESULT {
        if CImage::InitGDIPlus() == false {
            return E_FAIL;
        }
    
        let mut cDecoders: UINT = 0;
        let pDecoders: *mut Gdiplus_ImageCodecInfo = CImage::_getAllDecoders(&mut cDecoders);
        let hr: HRESULT = CImage::BuildCodecFilterString(pDecoders,
                                            cDecoders,
                                            strImporters,
                                            aguidFileTypes,
                                            pszAllFilesDescription,
                                            dwExclude,
                                            chSeparator);
        cpp_delete_array(pDecoders);
        return hr;
    }
    pub unsafe fn GetExporterFilterString(strExporters: &mut CSimpleString, aguidFileTypes: &mut CSimpleArray<GUID>, pszAllFilesDescription: LPCTSTR, dwExclude: DWORD, chSeparator: TCHAR) -> HRESULT {
        if CImage::InitGDIPlus() == false {
            return E_FAIL;
        }
    
        let mut cEncoders: UINT = 0;
        let pEncoders: *mut Gdiplus_ImageCodecInfo = CImage::_getAllEncoders(&mut cEncoders);
        let hr: HRESULT = CImage::BuildCodecFilterString(pEncoders,
                                            cEncoders,
                                            strExporters,
                                            aguidFileTypes,
                                            pszAllFilesDescription,
                                            dwExclude,
                                            chSeparator);
        cpp_delete_array(pEncoders);
        return hr;
    }
    pub unsafe fn InitGDIPlus() -> bool {
        return CImage__s_gdiplus.Init();
    }
    pub unsafe fn _getAllEncoders(cEncoders: *mut UINT) -> *mut Gdiplus_ImageCodecInfo {
        let mut total_size: UINT = 0;
        ((CImage__s_gdiplus.GetImageEncodersSize).unwrap_unchecked())(cEncoders, &mut total_size);
        if total_size == 0 {
            return core::ptr::null_mut(); // failure
        }
    
        let mut ret: *mut Gdiplus_ImageCodecInfo;
        let _elem_index: usize = 0 as usize;
        ret = cpp_new_array::<Gdiplus_ImageCodecInfo>(
            total_size / core::mem::size_of::<Gdiplus_ImageCodecInfo>() as UINT,
        );
        if ret.is_null() {
            *cEncoders = 0;
            return core::ptr::null_mut(); // failure
        }
    
        ((CImage__s_gdiplus.GetImageEncoders).unwrap_unchecked())(*cEncoders, total_size, ret);
        return ret; // needs delete[]
    }
    pub unsafe fn _getAllDecoders(cDecoders: *mut UINT) -> *mut Gdiplus_ImageCodecInfo {
        let mut total_size: UINT = 0;
        (((*core::ptr::addr_of_mut!(CImage__s_gdiplus)).GetImageDecodersSize).unwrap_unchecked())(cDecoders, &mut total_size as *mut UINT);
        if total_size == 0 {
            return core::ptr::null_mut();
        }
    
        let mut ret: *mut Gdiplus_ImageCodecInfo;
        let _elem_size_probe = || core::mem::size_of_val(&*core::ptr::null::<Gdiplus_ImageCodecInfo>().add(0));
        ret = cpp_new_array::<Gdiplus_ImageCodecInfo>(
            (total_size as usize) / core::mem::size_of::<Gdiplus_ImageCodecInfo>(),
        );
        if ret.is_null() {
            *cDecoders = 0;
            return core::ptr::null_mut();
        }
    
        (((*core::ptr::addr_of_mut!(CImage__s_gdiplus)).GetImageDecoders).unwrap_unchecked())(*cDecoders, total_size, ret);
        return ret;
    }
}

impl MYBITMAPINFOEX {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CInitGDIPlus {
    pub unsafe fn _clear_funs(&mut self) {
        let this_: *mut Self = self;
        (*this_).Startup = None;
        (*this_).Shutdown = None;
        (*this_).CreateBitmapFromFile = None;
        (*this_).CreateBitmapFromHBITMAP = None;
        (*this_).CreateBitmapFromStream = None;
        (*this_).CreateHBITMAPFromBitmap = None;
        (*this_).DisposeImage = None;
        (*this_).GetImageDecoders = None;
        (*this_).GetImageDecodersSize = None;
        (*this_).GetImageEncoders = None;
        (*this_).GetImageEncodersSize = None;
        (*this_).SaveImageToFile = None;
        (*this_).SaveImageToStream = None;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).m_hInst = core::ptr::null_mut();
        (*this_).m_dwToken = 0;
        (*this_).m_nCImageObjects = 0;
        (*this_).m_dwLastError = ERROR_SUCCESS;
        {
            (*this_)._clear_funs();
            InitializeCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        (*this_).ReleaseGDIPlus();
        DeleteCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    }
    pub unsafe fn Init(&mut self) -> bool {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    
        if (*this_).m_dwToken == 0 {
            if (*this_).m_hInst.is_null() {
                // `::LoadLibrary(TEXT("gdiplus.dll"))`, NUL-terminated.
                let name: [u16; 12] = [
                    'g' as u16, 'd' as u16, 'i' as u16, 'p' as u16, 'l' as u16, 'u' as u16,
                    's' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0,
                ];
                (*this_).m_hInst = LoadLibraryW(name.as_ptr());
            }
    
            if !(*this_).m_hInst.is_null()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).Startup), b"GdiplusStartup\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).Shutdown), b"GdiplusShutdown\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromFile), b"GdipCreateBitmapFromFile\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromHBITMAP), b"GdipCreateBitmapFromHBITMAP\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromStream), b"GdipCreateBitmapFromStream\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateHBITMAPFromBitmap), b"GdipCreateHBITMAPFromBitmap\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).DisposeImage), b"GdipDisposeImage\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageDecoders), b"GdipGetImageDecoders\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageDecodersSize), b"GdipGetImageDecodersSize\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageEncoders), b"GdipGetImageEncoders\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageEncodersSize), b"GdipGetImageEncodersSize\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).SaveImageToFile), b"GdipSaveImageToFile\0".as_ptr() as *const i8).is_some()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).SaveImageToStream), b"GdipSaveImageToStream\0".as_ptr() as *const i8).is_some()
            {
                // `GdiplusStartupInput input;` runs gdiplusinit.h's constructor with its
                // defaults: GdiplusVersion = 1, no callback, no suppression. Zeroed, the
                // version 0 makes GdiplusStartup refuse. Output has no constructor.
                let mut input: Gdiplus_GdiplusStartupInput = core::mem::zeroed();
                input.GdiplusVersion = 1;
                let mut output: Gdiplus_GdiplusStartupOutput = core::mem::zeroed();
                // `Startup` was resolved non-NULL by the `_get_fun` chain just above.
                if let Some(startup) = (*this_).Startup {
                    startup(
                        core::ptr::addr_of_mut!((*this_).m_dwToken),
                        core::ptr::addr_of_mut!(input),
                        core::ptr::addr_of_mut!(output),
                    );
                }
            }
        }
    
        let ret: bool = (*this_).m_dwToken != 0;
        LeaveCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    
        return ret;
    }
    pub unsafe fn ReleaseGDIPlus(&mut self) {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
        if (*this_).m_dwToken != 0 {
            if let Some(shutdown) = (*this_).Shutdown {
                shutdown((*this_).m_dwToken);
            }
    
            (*this_).m_dwToken = 0;
        }
        if !(*this_).m_hInst.is_null() {
            FreeLibrary((*this_).m_hInst);
            (*this_).m_hInst = core::ptr::null_mut();
        }
        (*this_)._clear_funs();
        LeaveCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    }
    pub unsafe fn IncreaseCImageCount(&mut self) {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
        (*this_).m_nCImageObjects += 1;
        LeaveCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    }
    pub unsafe fn DecreaseCImageCount(&mut self) {
        let this_: *mut Self = self;
        EnterCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
        (*this_).m_nCImageObjects -= 1;
        if (*this_).m_nCImageObjects == 0 {
            (*this_).ReleaseGDIPlus();
        }
        LeaveCriticalSection(core::ptr::addr_of_mut!((*this_).m_sect));
    }
}

impl CInitGDIPlus {
    /// atlimage.h's member TEMPLATE `template <typename T_FUN> T_FUN _get_fun(T_FUN& fun,
    /// LPCSTR name)`, hand-ported (cppapp does not generate templates): resolve `fun` from
    /// m_hInst if it is still NULL, and return it (the caller tests it for non-NULL).
    pub unsafe fn _get_fun<F: Copy>(&mut self, fun: *mut Option<F>, name: *const i8) -> Option<F> {
        if (*fun).is_none() {
            let p = GetProcAddress(self.m_hInst as _, name as _);
            *fun = core::mem::transmute_copy(&p);
        }
        *fun
    }
}
