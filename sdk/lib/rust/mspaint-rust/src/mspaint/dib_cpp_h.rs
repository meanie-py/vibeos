use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut g_fileSize: i32 = 0;
#[no_mangle]
pub static mut g_xDpi: f32 = 96.0;
#[no_mangle]
pub static mut g_yDpi: f32 = 96.0;
#[no_mangle]
pub static mut g_fileTime: SYSTEMTIME = unsafe { core::mem::zeroed() };




#[no_mangle]
pub unsafe extern "C" fn PpcmFromDpi(dpi: f32) -> f32 {
    // 1 DPI is 0.0254 meter. 1 centimeter is 1/100 meter.
    return dpi / (0.0254f32 * 100.0f32);
}
#[no_mangle]
pub unsafe extern "C" fn CreateDIBWithProperties(width: i32, height: i32) -> HBITMAP {
    let mut bmi: BITMAPINFO = core::mem::zeroed();
    ZeroMemory!(core::ptr::addr_of_mut!(bmi) as *mut core::ffi::c_void, core::mem::size_of::<BITMAPINFO>() as usize);
    bmi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width;
    bmi.bmiHeader.biHeight = height;
    bmi.bmiHeader.biPlanes = 1 as u16;
    bmi.bmiHeader.biBitCount = 24 as u16;
    return CreateDIBSection(core::ptr::null_mut(), core::ptr::addr_of!(bmi), DIB_RGB_COLORS as u32, core::ptr::null_mut(), core::ptr::null_mut(), 0 as u32);
}
#[no_mangle]
pub unsafe extern "C" fn CreateMonoBitmap(width: i32, height: i32, bWhite: BOOL) -> HBITMAP {
    let hbm: HBITMAP = CreateBitmap(width, height, 1, 1, core::ptr::null_mut());
    if hbm.is_null() {
        return core::ptr::null_mut();
    }

    if bWhite != 0 {
        let hdc: HDC = CreateCompatibleDC(core::ptr::null_mut());
        let hbmOld: HGDIOBJ = SelectObject(hdc, hbm as HGDIOBJ);
        let mut rc: RECT = RECT { left: 0, top: 0, right: width, bottom: height };
        FillRect(hdc, core::ptr::addr_of_mut!(rc), GetStockObject((WHITE_BRUSH as i32)) as HBRUSH);
        SelectObject(hdc, hbmOld);
        DeleteDC(hdc);
    }

    return hbm;
}
#[no_mangle]
pub unsafe extern "C" fn CreateColorDIB(width: i32, height: i32, rgb: COLORREF) -> HBITMAP {
    let ret: HBITMAP = CreateDIBWithProperties(width, height);
    if ret.is_null() {
        return core::ptr::null_mut();
    }

    if rgb != 0 {
        let hdc: HDC = CreateCompatibleDC(core::ptr::null_mut());
        let hbmOld: HGDIOBJ = SelectObject(hdc, ret as HGDIOBJ);
        let mut rc: RECT = core::mem::zeroed();
        SetRect(core::ptr::addr_of_mut!(rc), 0, 0, width, height);
        let hbr: HBRUSH = CreateSolidBrush(rgb);
        FillRect(hdc, core::ptr::addr_of_mut!(rc), hbr);
        DeleteObject(hbr as HGDIOBJ);
        SelectObject(hdc, hbmOld);
        DeleteDC(hdc);
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn CopyMonoImage(hbm: HBITMAP, mut cx: INT, mut cy: INT, stretchMode: INT) -> HBITMAP {
    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as INT, core::ptr::addr_of_mut!(bm) as LPVOID) == 0 {
        return core::ptr::null_mut();
    }

    if cx == 0 || cy == 0 {
        cx = bm.bmWidth;
        cy = bm.bmHeight;
    }

    let hdc1: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hdc2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbmNew: HBITMAP = CreateBitmap(cx, cy, 1, 1, core::ptr::null_mut());
    let hbm1Old: HGDIOBJ = SelectObject(hdc1, hbm as HGDIOBJ);
    let hbm2Old: HGDIOBJ = SelectObject(hdc2, hbmNew as HGDIOBJ);
    SetStretchBltMode(hdc2, stretchMode);
    StretchBlt(hdc2, 0, 0, cx, cy, hdc1, 0, 0, bm.bmWidth, bm.bmHeight, SRCCOPY);
    SelectObject(hdc1, hbm1Old);
    SelectObject(hdc2, hbm2Old);
    DeleteDC(hdc1);
    DeleteDC(hdc2);
    return hbmNew;
}
#[no_mangle]
pub unsafe extern "C" fn CopyDIBImage(hbm: HBITMAP, cx: i32, cy: i32, stretchMode: i32) -> HBITMAP {
    if stretchMode == (STRETCH_HALFTONE as i32) {
        return CopyImage((hbm).cast(), IMAGE_BITMAP, cx, cy, LR_CREATEDIBSECTION) as HBITMAP;
    }

    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID) == 0 {
        return core::ptr::null_mut();
    }

    let mut cx = cx;
    let mut cy = cy;
    if cx == 0 || cy == 0 {
        cx = bm.bmWidth;
        cy = bm.bmHeight;
    }

    let hdc1: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hdc2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbmNew: HBITMAP = CreateDIBWithProperties(cx, cy);
    let hbm1Old: HGDIOBJ = SelectObject(hdc1, hbm as HGDIOBJ);
    let hbm2Old: HGDIOBJ = SelectObject(hdc2, hbmNew as HGDIOBJ);
    SetStretchBltMode(hdc2, stretchMode);
    StretchBlt(hdc2, 0, 0, cx, cy, hdc1, 0, 0, bm.bmWidth, bm.bmHeight, SRCCOPY);
    SelectObject(hdc1, hbm1Old);
    SelectObject(hdc2, hbm2Old);
    DeleteDC(hdc1);
    DeleteDC(hdc2);
    return hbmNew;
}
#[no_mangle]
pub unsafe extern "C" fn CachedBufferDIB(mut hbm: HBITMAP, mut minimalWidth: i32, mut minimalHeight: i32) -> HBITMAP {
    if minimalWidth <= 0 {
        minimalWidth = 1;
    }
    if minimalHeight <= 0 {
        minimalHeight = 1;
    }

    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as i32, (core::ptr::addr_of_mut!(bm)).cast()) == 0 {
        hbm = core::ptr::null_mut();
    }

    if !hbm.is_null() && minimalWidth <= bm.bmWidth && minimalHeight <= bm.bmHeight {
        return hbm;
    }

    if !hbm.is_null() {
        DeleteObject((hbm).cast());
    }

    return CreateDIBWithProperties(
        (minimalWidth * 3) / 2,
        (minimalHeight * 3) / 2,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetDIBWidth(hBitmap: HBITMAP) -> i32 {
    let mut bm: BITMAP = core::mem::zeroed();
    GetObjectW(hBitmap as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);
    return bm.bmWidth;
}
#[no_mangle]
pub unsafe extern "C" fn GetDIBHeight(hBitmap: HBITMAP) -> i32 {
    let mut bm: BITMAP = core::mem::zeroed();
    GetObjectW((hBitmap).cast(), core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID);
    return bm.bmHeight;
}
#[no_mangle]
pub unsafe extern "C" fn SaveDIBToFile(hBitmap: HBITMAP, FileName: LPCWSTR, fIsMainFile: BOOL, guidFileType: &GUID) -> BOOL {
    let mut waitCursor = CWaitCursor__make_0();

    let mut img = CImageDx__make_0();
    img.Attach(hBitmap);
    let hr: HRESULT = img.SaveDx(FileName, guidFileType, g_xDpi, g_yDpi);
    img.Detach();

    if ((hr) < 0)
    {
        ShowError((IDS_SAVEERROR) as INT, FileName);
        return (FALSE) as BOOL;
    }

    if fIsMainFile == 0
    {
        return (TRUE) as BOOL;
    }

    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind: HANDLE = FindFirstFileW(FileName, core::ptr::addr_of_mut!(find));
    if hFind == INVALID_HANDLE_VALUE
    {
        ShowError((IDS_SAVEERROR) as INT, FileName);
        return (FALSE) as BOOL;
    }
    FindClose(hFind);

    SetFileInfo(FileName, core::ptr::addr_of_mut!(find), (TRUE) as BOOL);
    g_imageSaved = (TRUE) as i32;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SetFileInfo(name: LPCWSTR, pFound: LPWIN32_FIND_DATAW, isAFile: BOOL) {
    // update file time and size
    if !pFound.is_null() {
        let mut ft: FILETIME = core::mem::zeroed();
        FileTimeToLocalFileTime(core::ptr::addr_of_mut!((*pFound).ftLastWriteTime), core::ptr::addr_of_mut!(ft));
        FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(g_fileTime));

        g_fileSize = (*pFound).nFileSizeLow as i32;
    } else {
        ZeroMemory!(core::ptr::addr_of_mut!(g_fileTime) as *mut _, core::mem::size_of::<SYSTEMTIME>());
        g_fileSize = 0;
    }

    // update g_szFileName
    if !name.is_null() && *name.add(0 as usize) != 0 {
        let mut strName = CStringW::from_ptr(name);
        GetFullPathNameW(
            strName.GetString(),
            _countof!(g_szFileName) as u32,
            core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR,
            core::ptr::null_mut(),
        );
        // The following code won't work correctly when (name == g_szFileName):
        //   ::GetFullPathNameW(name, _countof(g_szFileName), g_szFileName, NULL);
    } else {
        LoadStringW(
            g_hinstExe,
            IDS_DEFAULTFILENAME as u32,
            core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR,
            _countof!(g_szFileName) as i32,
        );
    }

    // set title
    let mut strTitle = CStringW::new();
    atl_format!(
        strTitle,
        IDS_WINDOWTITLE,
        PathFindFileNameW(core::ptr::addr_of_mut!(g_szFileName) as *mut WCHAR)
    );
    mainWindow.__base0.SetWindowTextW(strTitle.GetString());

    // update file info and recent
    g_isAFile = isAFile;
    if g_isAFile != 0 {
        registrySettings.SetMostRecentFile(core::ptr::addr_of!(g_szFileName) as *const WCHAR);
    }

    g_imageSaved = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeImage(name: LPCWSTR, pFound: LPWIN32_FIND_DATAW, isFile: BOOL) -> HBITMAP {
    let white: COLORREF = RGB!(255, 255, 255);
    let hBitmap: HBITMAP = CreateColorDIB((registrySettings.BMPWidth) as i32, (registrySettings.BMPHeight) as i32, white);
    if hBitmap == core::ptr::null_mut() {
        ShowOutOfMemory();
        imageModel.ClearHistory();
        return core::ptr::null_mut();
    }

    let hScreenDC: HDC = GetDC(core::ptr::null_mut());
    g_xDpi = GetDeviceCaps(hScreenDC, (LOGPIXELSX as i32)) as f32;
    g_yDpi = GetDeviceCaps(hScreenDC, (LOGPIXELSY as i32)) as f32;
    ReleaseDC(core::ptr::null_mut(), hScreenDC);

    return SetBitmapAndInfo(hBitmap, name, pFound, isFile);
}
#[no_mangle]
pub unsafe extern "C" fn SetBitmapAndInfo(hBitmap: HBITMAP, name: LPCWSTR, pFound: LPWIN32_FIND_DATAW, isFile: BOOL) -> HBITMAP {
    // update image
    canvasWindow.updateScrollPos(0, 0);
    imageModel.PushImageForUndo__1(hBitmap);
    imageModel.ClearHistory();

    SetFileInfo(name, pFound, isFile);
    g_imageSaved = (TRUE) as i32;
    return hBitmap;
}
#[no_mangle]
pub unsafe extern "C" fn DoLoadImageFile(hwnd: HWND, name: LPCWSTR, fIsMainFile: BOOL) -> HBITMAP {
    let mut waitCursor = CWaitCursor__make_0();

    // find the file
    let mut find: WIN32_FIND_DATAW = core::mem::zeroed();
    let hFind: HANDLE = FindFirstFileW(name, core::ptr::addr_of_mut!(find));
    if hFind == INVALID_HANDLE_VALUE // does not exist
    {
        ShowError((IDS_LOADERRORTEXT) as INT, name);
        return core::ptr::null_mut();
    }
    FindClose(hFind);

    // is file empty?
    if find.nFileSizeLow == 0 && find.nFileSizeHigh == 0
    {
        if fIsMainFile != 0 {
            return InitializeImage(name, core::ptr::addr_of_mut!(find), (TRUE) as BOOL);
        }
    }

    // load the image
    let mut img = CImageDx__make_0();
    let mut xDpi: f32 = 0;
    let mut yDpi: f32 = 0;
    let mut hr: HRESULT = img.LoadDx(name, core::ptr::addr_of_mut!(xDpi), core::ptr::addr_of_mut!(yDpi));
    if (hr < 0) && fIsMainFile != 0
    {
        imageModel.ClearHistory();
        hr = img.LoadDx(name, core::ptr::addr_of_mut!(xDpi), core::ptr::addr_of_mut!(yDpi));
    }
    if hr < 0
    {
        ATLTRACE!(b"hr: 0x%08lX\n\0".as_ptr() as *const i8, hr);
        ShowError((IDS_LOADERRORTEXT) as INT, name);
        return core::ptr::null_mut();
    }

    let hBitmap: HBITMAP = img.Detach();
    if fIsMainFile == 0 {
        return hBitmap;
    }

    if xDpi <= 0 || yDpi <= 0
    {
        let hDC: HDC = GetDC(core::ptr::null_mut());
        xDpi = GetDeviceCaps(hDC, (LOGPIXELSX as i32)) as f32;
        yDpi = GetDeviceCaps(hDC, (LOGPIXELSY as i32)) as f32;
        ReleaseDC(core::ptr::null_mut(), hDC);
    }

    g_xDpi = xDpi;
    g_yDpi = yDpi;

    SetBitmapAndInfo(hBitmap, name, core::ptr::addr_of_mut!(find), (TRUE) as BOOL);
    return hBitmap;
}
#[no_mangle]
pub unsafe extern "C" fn Rotate90DegreeBlt(hDC1: HDC, cx: INT, cy: INT, bRight: BOOL, bMono: BOOL) -> HBITMAP {
    let mut hbm2: HBITMAP;
    if bMono != 0 {
        hbm2 = CreateBitmap(cy, cx, 1, 1, core::ptr::null_mut());
    } else {
        hbm2 = CreateDIBWithProperties(cy, cx);
    }
    if hbm2.is_null() {
        return core::ptr::null_mut();
    }

    let hDC2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbm2Old: HGDIOBJ = SelectObject(hDC2, (hbm2).cast());
    if bRight != 0 {
        let mut y: INT = 0;
        while y < cy {
            let mut x: INT = 0;
            while x < cx {
                let rgb: COLORREF = GetPixel(hDC1, x, y);
                SetPixelV(hDC2, cy - (y + 1), x, rgb);
                x += 1;
            }
            y += 1;
        }
    } else {
        let mut y: INT = 0;
        while y < cy {
            let mut x: INT = 0;
            while x < cx {
                let rgb: COLORREF = GetPixel(hDC1, x, y);
                SetPixelV(hDC2, y, cx - (x + 1), rgb);
                x += 1;
            }
            y += 1;
        }
    }
    SelectObject(hDC2, hbm2Old);
    DeleteDC(hDC2);
    return hbm2;
}
#[no_mangle]
pub unsafe extern "C" fn SkewDIB(hDC1: HDC, hbm: HBITMAP, nDegree: i32, bVertical: BOOL, bMono: BOOL) -> HBITMAP {
    let waitCursor = CWaitCursor__make_0();

    if nDegree == 0 {
        return CopyDIBImage(hbm, 0, 0, (STRETCH_HALFTONE as i32));
    }

    let eTan: f64 = tan((abs(nDegree) as f64) * M_PI / 180);

    let mut bm: BITMAP = core::mem::zeroed();
    GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as i32, (core::ptr::addr_of_mut!(bm)).cast());
    let cx: i32 = bm.bmWidth;
    let cy: i32 = bm.bmHeight;
    let mut dx: i32 = 0;
    let mut dy: i32 = 0;
    if bVertical != 0 {
        dy = INT((cx as f64) * eTan) as i32;
    } else {
        dx = INT((cy as f64) * eTan) as i32;
    }

    if dx == 0 && dy == 0 {
        return CopyDIBImage(hbm, 0, 0, (STRETCH_HALFTONE as i32));
    }

    let hbmNew: HBITMAP;
    if bMono != 0 {
        hbmNew = CreateMonoBitmap(cx + dx, cy + dy, (FALSE) as BOOL);
    } else {
        hbmNew = CreateColorDIB(cx + dx, cy + dy, RGB!(255, 255, 255));
    }
    if hbmNew.is_null() {
        return core::ptr::null_mut();
    }

    let hDC2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbm2Old: HGDIOBJ = SelectObject(hDC2, hbmNew as HGDIOBJ);
    if bVertical != 0 {
        let mut x: i32 = 0;
        while x < cx {
            let delta: i32 = INT((x as f64) * eTan) as i32;
            if nDegree > 0 {
                BitBlt(hDC2, x, dy - delta, 1, cy, hDC1, x, 0, SRCCOPY);
            } else {
                BitBlt(hDC2, x, delta, 1, cy, hDC1, x, 0, SRCCOPY);
            }
            x += 1;
        }
    } else {
        let mut y: i32 = 0;
        while y < cy {
            let delta: i32 = INT((y as f64) * eTan) as i32;
            if nDegree > 0 {
                BitBlt(hDC2, dx - delta, y, cx, 1, hDC1, 0, y, SRCCOPY);
            } else {
                BitBlt(hDC2, delta, y, cx, 1, hDC1, 0, y, SRCCOPY);
            }
            y += 1;
        }
    }

    SelectObject(hDC2, hbm2Old);
    DeleteDC(hDC2);
    return hbmNew;
}
#[no_mangle]
pub unsafe extern "C" fn getSubImage(hbmWhole: HBITMAP, rcPartial: &RECT) -> HBITMAP {
    let mut rc: CRect = CRect { __base0: *rcPartial };
    let hbmPart: HBITMAP = CreateDIBWithProperties(rc.Width(), rc.Height());
    if hbmPart.is_null() {
        return core::ptr::null_mut();
    }

    let hDC1: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hDC2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbm1Old: HGDIOBJ = SelectObject(hDC1, hbmWhole as HGDIOBJ);
    let hbm2Old: HGDIOBJ = SelectObject(hDC2, hbmPart as HGDIOBJ);
    BitBlt(hDC2, 0, 0, rc.Width(), rc.Height(), hDC1, rc.__base0.left, rc.__base0.top, SRCCOPY);
    SelectObject(hDC1, hbm1Old);
    SelectObject(hDC2, hbm2Old);
    DeleteDC(hDC1);
    DeleteDC(hDC2);
    return hbmPart;
}
#[no_mangle]
pub unsafe extern "C" fn putSubImage(hbmWhole: HBITMAP, rcPartial: &RECT, hbmPart: HBITMAP) {
    let mut rc: CRect = CRect { __base0: *rcPartial };
    let hDC1: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hDC2: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hbm1Old: HGDIOBJ = SelectObject(hDC1, (hbmWhole).cast());
    let hbm2Old: HGDIOBJ = SelectObject(hDC2, (hbmPart).cast());
    BitBlt(hDC1, rc.__base0.left, rc.__base0.top, rc.Width(), rc.Height(), hDC2, 0, 0, SRCCOPY);
    SelectObject(hDC1, hbm1Old);
    SelectObject(hDC2, hbm2Old);
    DeleteDC(hDC1);
    DeleteDC(hDC2);
}
#[no_mangle]
pub unsafe extern "C" fn BitmapToClipboardDIB(hBitmap: HBITMAP) -> HGLOBAL {
    let mut waitCursor = CWaitCursor__make_0();

    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW(hBitmap as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID) == 0 {
        return core::ptr::null_mut();
    }

    let mut bmi: BITMAPINFODX = BITMAPINFODX__make_0();
    ZeroMemory!(core::ptr::addr_of_mut!(bmi) as LPVOID, core::mem::size_of::<BITMAPINFODX>());
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biWidth = bm.bmWidth;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biHeight = bm.bmHeight;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biPlanes = 1 as WORD;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biBitCount = bm.bmBitsPixel;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biCompression = BI_RGB as DWORD;
    (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biSizeImage = (bm.bmWidthBytes * bm.bmHeight) as DWORD;

    let mut cColors: INT;
    if bm.bmBitsPixel < 16 as WORD {
        cColors = (1 as INT) << (bm.bmBitsPixel as INT);
    } else {
        cColors = 0 as INT;
    }

    let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());

    if cColors != 0 {
        let hbmOld: HGDIOBJ = SelectObject(hDC, hBitmap as HGDIOBJ);
        cColors = GetDIBColorTable(
            hDC,
            0 as UINT,
            cColors as UINT,
            core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(bmi)).__base0.bmiColors) as *mut RGBQUAD,
        ) as INT;
        SelectObject(hDC, hbmOld);
    }

    let cbColors: DWORD = (cColors as DWORD) * (core::mem::size_of::<RGBQUAD>() as DWORD);
    let dwSize: DWORD = (core::mem::size_of::<BITMAPINFOHEADER>() as DWORD)
        + cbColors
        + (*core::ptr::addr_of_mut!(bmi)).__base0.bmiHeader.biSizeImage;
    let mut hGlobal: HGLOBAL = GlobalAlloc(
        (GHND as UINT) | (GMEM_SHARE as UINT),
        dwSize as SIZE_T,
    );
    if !hGlobal.is_null() {
        let mut pb: LPBYTE = GlobalLock(hGlobal) as LPBYTE;
        if !pb.is_null() {
            CopyMemory!(pb as LPVOID, core::ptr::addr_of_mut!(bmi) as LPVOID, core::mem::size_of::<BITMAPINFOHEADER>() as usize);
            pb = pb.add(core::mem::size_of::<BITMAPINFOHEADER>() as usize);

            CopyMemory!(
                pb as LPVOID,
                core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(bmi)).__base0.bmiColors) as LPVOID,
                cbColors as usize,
            );
            pb = pb.add(cbColors as usize);

            GetDIBits(
                hDC,
                (hBitmap as HGDIOBJ).cast(),
                0 as UINT,
                bm.bmHeight as UINT,
                pb as LPVOID,
                core::ptr::addr_of_mut!((*core::ptr::addr_of_mut!(bmi)).__base0) as *mut BITMAPINFO,
                DIB_RGB_COLORS as UINT,
            );

            GlobalUnlock(hGlobal);
        } else {
            GlobalFree(hGlobal);
            hGlobal = core::ptr::null_mut();
        }
    }

    DeleteDC(hDC);

    hGlobal
}

#[no_mangle]
pub unsafe extern "C" fn BitmapFromClipboardDIB(hGlobal: HGLOBAL) -> HBITMAP {
    let mut waitCursor = CWaitCursor__make_0();

    let mut pb: LPBYTE = GlobalLock(hGlobal) as LPBYTE;
    if pb.is_null() {
        return core::ptr::null_mut();
    }

    let pbmi: LPBITMAPINFO = pb as LPBITMAPINFO;
    pb = pb.add((*pbmi).bmiHeader.biSize as usize);

    let mut cColors: i32 = 0;
    let mut cbColors: i32 = 0;
    if (*pbmi).bmiHeader.biSize == core::mem::size_of::<BITMAPCOREHEADER>() as u32 {
        let pbmci: LPBITMAPCOREINFO = pbmi as LPBITMAPCOREINFO;
        let BitCount: u16 = (*pbmci).bmciHeader.bcBitCount;
        if BitCount < 16 {
            cColors = 1 << BitCount;
            cbColors = cColors * core::mem::size_of::<RGBTRIPLE>() as i32;
            pb = pb.add(cbColors as usize);
        }
    } else if (*pbmi).bmiHeader.biSize >= core::mem::size_of::<BITMAPINFOHEADER>() as u32 {
        let BitCount: u16 = (*pbmi).bmiHeader.biBitCount;
        if BitCount < 16 {
            cColors = 1 << BitCount;
            cbColors = cColors * core::mem::size_of::<RGBQUAD>() as i32;
            pb = pb.add(cbColors as usize);
        }
    }

    let hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let hBitmap: HBITMAP = CreateDIBSection(
        hDC,
        pbmi,
        DIB_RGB_COLORS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    if !hBitmap.is_null() {
        SetDIBits(
            hDC,
            hBitmap,
            0,
            (labs((*pbmi).bmiHeader.biHeight)) as UINT,
            pb as LPVOID,
            pbmi,
            DIB_RGB_COLORS,
        );
    }
    DeleteDC(hDC);

    GlobalUnlock(hGlobal);

    return hBitmap;
}
#[no_mangle]
pub unsafe extern "C" fn BitmapFromHEMF(hEMF: HENHMETAFILE) -> HBITMAP {
    let mut waitCursor: CWaitCursor = CWaitCursor__make_0();

    let mut header: ENHMETAHEADER = core::mem::zeroed();
    if GetEnhMetaFileHeader(hEMF, core::mem::size_of::<ENHMETAHEADER>() as u32, core::ptr::addr_of_mut!(header)) == 0 {
        return core::ptr::null_mut();
    }

    let mut rc: CRect = CRect {
        __base0: *(core::ptr::addr_of!(header.rclBounds) as LPRECT),
    };
    let mut cx: i32 = rc.Width();
    let mut cy: i32 = rc.Height();
    let mut hbm: HBITMAP = CreateColorDIB(cx, cy, RGB!(255, 255, 255));
    if hbm.is_null() {
        return core::ptr::null_mut();
    }

    let mut hDC: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let mut hbmOld: HGDIOBJ = SelectObject(hDC, (hbm).cast());
    PlayEnhMetaFile(hDC, hEMF, core::ptr::addr_of_mut!(rc.__base0) as *const RECT);
    SelectObject(hDC, hbmOld);
    DeleteDC(hDC);

    return hbm;
}
#[no_mangle]
pub unsafe extern "C" fn IsBitmapBlackAndWhite(hbm: HBITMAP) -> BOOL {
    let mut waitCursor = CWaitCursor__make_0();

    let mut bm: BITMAP = core::mem::zeroed();
    if GetObjectW(hbm as HGDIOBJ, core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID) == 0 {
        return (FALSE) as BOOL;
    }

    if bm.bmBitsPixel == 1 {
        return (TRUE) as BOOL;
    }

    let mut bmi: BITMAPINFOEX = BITMAPINFOEX__make_0();
    core::ptr::write_bytes(core::ptr::addr_of_mut!(bmi) as *mut u8, 0, core::mem::size_of::<BITMAPINFOEX>());
    bmi.__base0.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    bmi.__base0.bmiHeader.biWidth = bm.bmWidth;
    bmi.__base0.bmiHeader.biHeight = bm.bmHeight;
    bmi.__base0.bmiHeader.biPlanes = 1;
    bmi.__base0.bmiHeader.biBitCount = 24;

    let widthbytes: DWORD = WIDTHBYTES!(24 * bm.bmWidth) as DWORD;
    let cbBits: DWORD = widthbytes * (bm.bmHeight as DWORD);
    let pbBits: LPBYTE = cpp_new_array::<BYTE>(cbBits) as LPBYTE;

    let hdc: HDC = CreateCompatibleDC(core::ptr::null_mut());
    GetDIBits(hdc, hbm, 0, bm.bmHeight as UINT, pbBits as LPVOID, core::ptr::addr_of_mut!(bmi.__base0) as *mut BITMAPINFO, DIB_RGB_COLORS);
    DeleteDC(hdc);

    let mut bBlackAndWhite: BOOL = (TRUE) as i32;
    'finish: {
        let mut y: LONG = 0;
        while y < bm.bmHeight {
            let mut pbLine: LPBYTE = pbBits.add((widthbytes as usize) * (y as usize));
            let mut x: LONG = 0;
            while x < bm.bmWidth {
                let Blue: BYTE = *pbLine;
                pbLine = pbLine.add(1);
                let Green: BYTE = *pbLine;
                pbLine = pbLine.add(1);
                let Red: BYTE = *pbLine;
                pbLine = pbLine.add(1);
                let rgbColor: COLORREF = RGB!(Red, Green, Blue);
                if rgbColor != RGB!(0, 0, 0) && rgbColor != RGB!(255, 255, 255) {
                    bBlackAndWhite = (FALSE) as i32;
                    break 'finish;
                }
                x += 1;
            }
            y += 1;
        }
    }

    // Finish:
    cpp_delete_array(pbBits);

    return bBlackAndWhite;
}
#[no_mangle]
pub unsafe extern "C" fn ConvertToBlackAndWhite(hbm: HBITMAP) -> HBITMAP {
    let mut waitCursor = CWaitCursor__make_0();

    let mut bm: BITMAP = core::mem::zeroed();
    if !(GetObjectW((hbm).cast(), core::mem::size_of::<BITMAP>() as i32, core::ptr::addr_of_mut!(bm) as LPVOID) != 0) {
        return core::ptr::null_mut();
    }

    let mut bmi: BITMAPINFOEX = BITMAPINFOEX__make_0();
    ZeroMemory!(core::ptr::addr_of_mut!(bmi) as LPVOID, core::mem::size_of::<BITMAPINFOEX>());
    bmi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = bm.bmWidth;
    bmi.bmiHeader.biHeight = bm.bmHeight;
    bmi.bmiHeader.biPlanes = 1 as u16;
    bmi.bmiHeader.biBitCount = 1 as u16;
    (*core::ptr::addr_of_mut!(bmi.bmiColors[0]).add(1 as usize)).rgbBlue = 255 as u8;
    (*core::ptr::addr_of_mut!(bmi.bmiColors[0]).add(1 as usize)).rgbGreen = 255 as u8;
    (*core::ptr::addr_of_mut!(bmi.bmiColors[0]).add(1 as usize)).rgbRed = 255 as u8;
    let hdc: HDC = CreateCompatibleDC(core::ptr::null_mut());
    let mut pvMonoBits: LPVOID = core::ptr::null_mut();
    let hMonoBitmap: HBITMAP = CreateDIBSection(
        hdc,
        core::ptr::addr_of_mut!(bmi.__base0) as *const BITMAPINFO,
        DIB_RGB_COLORS as u32,
        core::ptr::addr_of_mut!(pvMonoBits),
        core::ptr::null_mut(),
        0 as u32,
    );
    if hMonoBitmap.is_null() {
        DeleteDC(hdc);
        return core::ptr::null_mut();
    }

    let hNewBitmap: HBITMAP = CreateDIBWithProperties(bm.bmWidth, bm.bmHeight);
    if !hNewBitmap.is_null() {
        GetDIBits(
            hdc,
            hbm,
            0 as u32,
            bm.bmHeight as u32,
            pvMonoBits,
            core::ptr::addr_of_mut!(bmi.__base0) as *mut BITMAPINFO,
            DIB_RGB_COLORS as u32,
        );
        SetDIBits(
            hdc,
            hNewBitmap,
            0 as u32,
            bm.bmHeight as u32,
            pvMonoBits,
            core::ptr::addr_of_mut!(bmi.__base0) as *const BITMAPINFO,
            DIB_RGB_COLORS as u32,
        );
    }
    DeleteObject(hMonoBitmap as HGDIOBJ);
    DeleteDC(hdc);

    hNewBitmap
}

impl BITMAPINFOEX {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl BITMAPINFODX {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}
