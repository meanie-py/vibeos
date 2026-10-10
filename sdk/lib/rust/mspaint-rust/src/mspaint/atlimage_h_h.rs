use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn CGdiPlusToken__ReleaseGDIPlus(this: *mut core::ffi::c_void);
}

































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
        (*this_).m_bHasAlphaChannel = (FALSE) as i32;
        (*this_).m_bIsDIBSection = (FALSE) as i32;
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
    pub unsafe fn ReleaseGDIPlus() {
        CGdiPlusToken__ReleaseGDIPlus(core::ptr::addr_of_mut!(CImage__s_gdiplus) as *mut core::ffi::c_void);
    }
    pub unsafe fn Attach(&mut self, hBitmap: HBITMAP, eOrientation: DIBOrientation) {
        let this_: *mut Self = self;
        (*this_).AttachInternal(hBitmap, eOrientation, -1);
    }
    pub unsafe fn Detach(&mut self) -> HBITMAP {
        let this_: *mut Self = self;
            (*this_).m_hOldBitmap = core::ptr::null_mut();
            (*this_).m_nBPP = 0;
            (*this_).m_nPitch = 0;
            (*this_).m_nHeight = 0;
            (*this_).m_nWidth = 0;
            (*this_).m_pBits = core::ptr::null_mut();
    
            (*this_).m_bIsDIBSection = (FALSE) as i32;
            (*this_).m_bHasAlphaChannel = (FALSE) as i32;
            (*this_).m_clrTransparentColor = CLR_INVALID;
    
            let hBitmap: HBITMAP = (*this_).m_hBitmap;
            (*this_).m_hBitmap = core::ptr::null_mut();
            return hBitmap;
        }
    pub unsafe fn GetDC(&mut self) -> HDC {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_nDCRefCount >= 0);
        if InterlockedIncrement(core::ptr::addr_of_mut!((*this_).m_nDCRefCount)) == 1 {
            ATLASSERT!((*this_).m_hDC == core::ptr::null_mut());
            ATLASSERT!((*this_).m_hOldBitmap == core::ptr::null_mut());
    
            (*this_).m_hDC = CreateCompatibleDC(core::ptr::null_mut());
            ATLASSERT!((*this_).m_hDC != core::ptr::null_mut());
    
            (*this_).m_hOldBitmap = SelectObject((*this_).m_hDC, ((*this_).m_hBitmap).cast()) as HBITMAP;
            ATLASSERT!((*this_).m_hOldBitmap != core::ptr::null_mut());
        }
    
        return (*this_).m_hDC;
    }
    pub unsafe fn ReleaseDC(&mut self) {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_nDCRefCount > 0);
    
        if InterlockedDecrement(core::ptr::addr_of_mut!((*this_).m_nDCRefCount)) != 0 {
            return;
        }
    
        if !(*this_).m_hOldBitmap.is_null() {
            ATLASSERT!(!(*this_).m_hDC.is_null());
            SelectObject((*this_).m_hDC, ((*this_).m_hOldBitmap).cast());
            (*this_).m_hOldBitmap = core::ptr::null_mut();
        }
    
        if !(*this_).m_hDC.is_null() {
            DeleteDC((*this_).m_hDC);
            (*this_).m_hDC = core::ptr::null_mut();
        }
    }
    pub unsafe fn Create(&mut self, nWidth: i32, nHeight: i32, nBPP: i32, dwFlags: DWORD) -> BOOL {
        let this_: *mut Self = self;
        return (*this_).CreateEx(nWidth, nHeight, nBPP, BI_RGB, core::ptr::null_mut::<DWORD>(), dwFlags);
    }
    pub unsafe fn CreateEx(&mut self, nWidth: INT, nHeight: INT, nBPP: INT, eCompression: DWORD, pdwBitmasks: *const DWORD, dwFlags: DWORD) -> BOOL {
        let this_: *mut Self = self;
            return (*this_).CreateInternal(nWidth, nHeight, nBPP, eCompression, pdwBitmasks, dwFlags);
        }
    pub unsafe fn Destroy(&mut self) {
        let this_: *mut Self = self;
        if (*this_).m_hBitmap != core::ptr::null_mut() {
            DeleteObject(((*this_).Detach()).cast());
        }
    }
    pub unsafe fn GetHeight(&mut self) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hBitmap);
        return (*this_).m_nHeight;
    }
    pub unsafe fn GetWidth(&mut self) -> i32 {
        let this_: *mut Self = self;
        ATLASSERT!((*this_).m_hBitmap);
        return (*this_).m_nWidth;
    }
    pub unsafe fn ShouldExcludeFormat(guidFileType: *const GUID, dwExclude: DWORD) -> bool {
        // Gdiplus_ImageFormat* GUIDs (GDI+ image format identifiers)
        let Gdiplus_ImageFormatGIF: GUID = GUID { Data1: 0xb96b3cb0, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatBMP: GUID = GUID { Data1: 0xb96b3cab, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatEMF: GUID = GUID { Data1: 0xb96b3cac, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatWMF: GUID = GUID { Data1: 0xb96b3cad, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatJPEG: GUID = GUID { Data1: 0xb96b3cae, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatPNG: GUID = GUID { Data1: 0xb96b3caf, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatTIFF: GUID = GUID { Data1: 0xb96b3cb1, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
        let Gdiplus_ImageFormatIcon: GUID = GUID { Data1: 0xb96b3cb5, Data2: 0x0728, Data3: 0x11d3, Data4: [0x9d, 0x7b, 0x00, 0x00, 0xf8, 0x1e, 0xf3, 0x2e] };
    
        // ATL_CImage_ExcludeFlags_* values (ATL CImage::ExcludeFlags)
        let ATL_CImage_ExcludeFlags_excludeBMP: DWORD = 0x0000_0001; // excludeBMP
        let ATL_CImage_ExcludeFlags_excludeGIF: DWORD = 0x0000_0002; // excludeGIF
        let ATL_CImage_ExcludeFlags_excludeJPEG: DWORD = 0x0000_0004; // excludeJPEG
        let ATL_CImage_ExcludeFlags_excludePNG: DWORD = 0x0000_0008; // excludePNG
        let ATL_CImage_ExcludeFlags_excludeEMF: DWORD = 0x0000_0010; // excludeEMF
        let ATL_CImage_ExcludeFlags_excludeWMF: DWORD = 0x0000_0020; // excludeWMF
        let ATL_CImage_ExcludeFlags_excludeTIFF: DWORD = 0x0000_0040; // excludeTIFF
        let ATL_CImage_ExcludeFlags_excludeIcon: DWORD = 0x0000_0080; // excludeIcon
        let ATL_CImage_ExcludeFlags_excludeOther: DWORD = 0x8000_0000; // excludeOther
    
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
    pub unsafe fn BuildCodecFilterString(pCodecs: *const Gdiplus_ImageCodecInfo, cCodecs: UINT, strFilter: &mut CStringW, aguidFileTypes: &mut CSimpleArray<GUID>, pszAllFilesDescription: LPCTSTR, dwExclude: DWORD, chSeparator: TCHAR) -> HRESULT {
        if pCodecs.is_null() || cCodecs == 0 {
            strFilter.AppendChar(chSeparator);
            return E_FAIL;
        }
    
        if !pszAllFilesDescription.is_null() {
            *strFilter += pszAllFilesDescription;
    
            let mut bFirst: BOOL = (TRUE) as i32;
            let mut extensions = CStringW::new();
            let mut i: UINT = 0;
            while i < cCodecs {
                'body: {
                    if CImage::ShouldExcludeFormat(&((*pCodecs.add(i as usize)).FormatID), dwExclude) != false {
                        break 'body;
                    }
    
                    if bFirst != 0 {
                        bFirst = (FALSE) as i32;
                    } else {
                        extensions.AppendChar(TEXT(';'));
                    }
    
                    let mut ext = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
                    extensions += ext.GetString();
                }
                i += 1;
            }
    
            strFilter.AppendChar(chSeparator);
            *strFilter += extensions.GetString();
            strFilter.AppendChar(chSeparator);
    
            aguidFileTypes.Add(GUID_NULL);
        }
    
        let mut i: UINT = 0;
        while i < cCodecs {
            'body: {
                if CImage::ShouldExcludeFormat(&((*pCodecs.add(i as usize)).FormatID), dwExclude) != false {
                    break 'body;
                }
    
                let mut extensions = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
    
                let mut desc = CStringW::from_ptr((*pCodecs.add(i as usize)).FormatDescription);
                *strFilter += desc.GetString();
                *strFilter += TEXT(" (");
                *strFilter += extensions.GetString();
                *strFilter += TEXT(")");
                strFilter.AppendChar(chSeparator);
                *strFilter += extensions.GetString();
                strFilter.AppendChar(chSeparator);
    
                aguidFileTypes.Add((*pCodecs.add(i as usize)).FormatID);
            }
            i += 1;
        }
    
        strFilter.AppendChar(chSeparator);
        return S_OK;
    }
    pub unsafe fn GetImporterFilterString(strImporters: &mut CSimpleString, aguidFileTypes: &mut CSimpleArray<GUID>, pszAllFilesDescription: LPCTSTR, dwExclude: DWORD, chSeparator: TCHAR) -> HRESULT {
        if CImage::InitGDIPlus() == false {
            return E_FAIL;
        }
    
        let mut cDecoders: UINT = 0;
        let mut pDecoders: *mut Gdiplus_ImageCodecInfo = CImage::_getAllDecoders(&mut cDecoders);
        let mut hr: HRESULT = CImage::BuildCodecFilterString(pDecoders,
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
    
        let cEncoders: UINT = 0;
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
    pub unsafe fn FindCodecForExtension(dotext: LPCTSTR, pCodecs: *const Gdiplus_ImageCodecInfo, nCodecs: UINT) -> CLSID {
        let mut i: UINT = 0;
        while i < nCodecs {
            let mut strSpecs = CStringW::from_ptr((*pCodecs.add(i as usize)).FilenameExtension);
            let mut ichOld: i32 = 0;
            let mut ichSep: i32;
            loop {
                ichSep = strSpecs.Find(TEXT(';' as TCHAR), ichOld);
    
                let mut strSpec: CStringW;
                if ichSep < 0 {
                    strSpec = strSpecs.Mid(ichOld);
                } else {
                    strSpec = strSpecs.Mid(ichOld, ichSep - ichOld);
                }
    
                let ichDot: i32 = strSpec.ReverseFind(TEXT('.' as TCHAR));
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
        return CLSID_NULL;
    }
    pub unsafe fn FindCodecForFileType(guidFileType: *const GUID, pCodecs: *const Gdiplus_ImageCodecInfo, nCodecs: u32) -> CLSID {
        let mut iInfo: u32 = 0;
        while iInfo < nCodecs {
            if IsEqualGUID!(core::ptr::addr_of!((*pCodecs.add(iInfo as usize)).FormatID), guidFileType) != false {
                return (*pCodecs.add(iInfo as usize)).Clsid;
            }
            iInfo += 1;
        }
        return CLSID_NULL;
    }
    pub unsafe fn _getAllEncoders(cEncoders: *mut UINT) -> *mut Gdiplus_ImageCodecInfo {
        let mut total_size: UINT = 0;
        (CImage__s_gdiplus.GetImageEncodersSize)(cEncoders, &mut total_size);
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
    
        (CImage__s_gdiplus.GetImageEncoders)(*cEncoders, total_size, ret);
        return ret; // needs delete[]
    }
    pub unsafe fn _getAllDecoders(cDecoders: *mut UINT) -> *mut Gdiplus_ImageCodecInfo {
        let mut total_size: UINT = 0;
        ((*core::ptr::addr_of_mut!(CImage__s_gdiplus)).GetImageDecodersSize)(cDecoders, &mut total_size as *mut UINT);
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
    
        ((*core::ptr::addr_of_mut!(CImage__s_gdiplus)).GetImageDecoders)(*cDecoders, total_size, ret);
        return ret;
    }
    pub unsafe fn AttachInternal(&mut self, hBitmap: HBITMAP, mut eOrientation: DIBOrientation, iTransColor: LONG) {
        let this_: *mut Self = self;
        (*this_).Destroy();
    
        let mut ds: DIBSECTION = core::mem::zeroed();
        let bm: *mut BITMAP = core::ptr::addr_of_mut!(ds.dsBm);
    
        (*this_).m_bIsDIBSection = (GetObjectW((hBitmap).cast(), core::mem::size_of::<DIBSECTION>() as i32, core::ptr::addr_of_mut!(ds) as LPVOID) == core::mem::size_of::<DIBSECTION>() as i32) as BOOL;
    
        if (*this_).m_bIsDIBSection == 0 && GetObjectW((hBitmap).cast(), core::mem::size_of::<BITMAP>() as i32, bm as LPVOID) == 0 {
            return;
        }
    
        (*this_).m_hBitmap = hBitmap;
        (*this_).m_nWidth = (*bm).bmWidth;
        (*this_).m_nHeight = (*bm).bmHeight;
        (*this_).m_nBPP = ((*bm).bmBitsPixel) as _;
        (*this_).m_bHasAlphaChannel = ((*bm).bmBitsPixel == 32) as BOOL;
        (*this_).m_clrTransparentColor = CLR_INVALID;
    
        (*this_).m_nPitch = 0;
        (*this_).m_pBits = core::ptr::null_mut();
        if (*this_).m_bIsDIBSection != 0 {
            if eOrientation == ATL_CImage_DIBOrientation_DIBOR_DEFAULT {
                eOrientation = if ds.dsBmih.biHeight < 0 {
                    ATL_CImage_DIBOrientation_DIBOR_TOPDOWN
                } else {
                    ATL_CImage_DIBOrientation_DIBOR_BOTTOMUP
                };
            }
    
            let pb: LPBYTE = (*bm).bmBits as LPBYTE;
            if eOrientation == ATL_CImage_DIBOrientation_DIBOR_BOTTOMUP {
                (*this_).m_nPitch = -(*bm).bmWidthBytes;
                (*this_).m_pBits = pb.offset(((*bm).bmWidthBytes * ((*this_).m_nHeight - 1)) as isize) as LPVOID;
            } else {
                (*this_).m_nPitch = (*bm).bmWidthBytes;
                (*this_).m_pBits = pb as LPVOID;
            }
        }
    }
    pub unsafe fn CreateInternal(&mut self, nWidth: i32, nHeight: i32, nBPP: i32, eCompression: DWORD, pdwBitmasks: *const DWORD, dwFlags: DWORD) -> BOOL {
        let this_: *mut Self = self;
            ATLASSERT!(nWidth != 0);
            ATLASSERT!(nHeight != 0);
    
            // initialize BITMAPINFO extension
            let mut bi: MYBITMAPINFOEX = MYBITMAPINFOEX__make_0();
            ZeroMemory!(core::ptr::addr_of_mut!(bi) as *mut _, core::mem::size_of::<MYBITMAPINFOEX>());
            bi.__base0.bmiHeader.biSize = (core::mem::size_of::<BITMAPINFOHEADER>()) as DWORD;
            bi.__base0.bmiHeader.biWidth = nWidth;
            bi.__base0.bmiHeader.biHeight = nHeight;
            bi.__base0.bmiHeader.biPlanes = 1;
            bi.__base0.bmiHeader.biBitCount = nBPP as WORD;
            bi.__base0.bmiHeader.biCompression = eCompression;
    
            // is there alpha channel?
            let bHasAlphaCh: bool;
            bHasAlphaCh = (nBPP == 32 && (dwFlags & createAlphaChannel) != 0);
    
            // get orientation
            let eOrientation: DIBOrientation;
            eOrientation = if nHeight > 0 { ATL_CImage_DIBOrientation_DIBOR_BOTTOMUP } else { ATL_CImage_DIBOrientation_DIBOR_TOPDOWN };
    
            // does it have bit fields?
            if eCompression == BI_BITFIELDS
            {
                if nBPP == 16 || nBPP == 32
                {
                    // store the mask data
                    let pdwMask: LPDWORD = core::ptr::addr_of_mut!(bi.__base0.bmiColors) as LPDWORD;
                    *pdwMask.offset((0) as isize) = *pdwBitmasks.offset((0) as isize);
                    *pdwMask.offset((1) as isize) = *pdwBitmasks.offset((1) as isize);
                    *pdwMask.offset((2) as isize) = *pdwBitmasks.offset((2) as isize);
                }
                else
                {
                    return (FALSE) as BOOL;
                }
            }
            else
            {
                ATLASSERT!(pdwBitmasks.is_null());
                if !pdwBitmasks.is_null()
                {
                    return (FALSE) as BOOL;
                }
            }
    
            // create a DIB section
            let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
            ATLASSERT!(!hDC.is_null());
            let hbm: HBITMAP = CreateDIBSection(hDC, core::ptr::addr_of!(bi.__base0) as *const BITMAPINFO, DIB_RGB_COLORS, core::ptr::null_mut(), core::ptr::null_mut(), 0);
            ATLASSERT!(!hbm.is_null());
            DeleteDC(hDC);
    
            // attach it
            (*this_).AttachInternal(hbm, eOrientation, -(1));
            (*this_).m_bHasAlphaChannel = bHasAlphaCh as BOOL;
    
            return (hbm != core::ptr::null_mut()) as BOOL;
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
                let name: [u16; 12] = ['g' as u16, 'd' as u16, 'i' as u16, 'p' as u16, 'l' as u16, 'u' as u16, 's' as u16, '.' as u16, 'd' as u16, 'l' as u16, 'l' as u16, 0];
                (*this_).m_hInst = LoadLibrary(TEXT(name.as_ptr()));
            }
    
            if !(*this_).m_hInst.is_null()
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).Startup), b"GdiplusStartup\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).Shutdown), b"GdiplusShutdown\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromFile), b"GdipCreateBitmapFromFile\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromHBITMAP), b"GdipCreateBitmapFromHBITMAP\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateBitmapFromStream), b"GdipCreateBitmapFromStream\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).CreateHBITMAPFromBitmap), b"GdipCreateHBITMAPFromBitmap\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).DisposeImage), b"GdipDisposeImage\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageDecoders), b"GdipGetImageDecoders\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageDecodersSize), b"GdipGetImageDecodersSize\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageEncoders), b"GdipGetImageEncoders\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).GetImageEncodersSize), b"GdipGetImageEncodersSize\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).SaveImageToFile), b"GdipSaveImageToFile\0".as_ptr() as *const i8)
                && (*this_)._get_fun(core::ptr::addr_of_mut!((*this_).SaveImageToStream), b"GdipSaveImageToStream\0".as_ptr() as *const i8)
            {
                let mut input: Gdiplus_GdiplusStartupInput = Gdiplus_GdiplusStartupInput__zeroed();
                let mut output: Gdiplus_GdiplusStartupOutput = Gdiplus_GdiplusStartupOutput__zeroed();
                ((*this_).Startup)(
                    core::ptr::addr_of_mut!((*this_).m_dwToken),
                    core::ptr::addr_of_mut!(input),
                    core::ptr::addr_of_mut!(output),
                );
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
