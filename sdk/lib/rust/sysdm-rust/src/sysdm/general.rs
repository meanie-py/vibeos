// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ANIM_STEP: i32 = 2;

pub const ANIM_TIME: i32 = 50;

pub const ID_SYSUPTIME_UPDATE_TIMER: i32 = 1;

#[repr(C)]
pub struct _IMGINFO {
    pub hBitmap: HBITMAP,
    pub cxSource: INT,
    pub cySource: INT,
    pub iPlanes: INT,
    pub iBits: INT,
}
pub type IMGINFO = _IMGINFO;
pub type PIMGINFO = *mut _IMGINFO;

pub type PFGETTICKCOUNT64 = Option<unsafe extern "system" fn() -> ULONGLONG>;

static mut pImgInfo: PIMGINFO = core::ptr::null_mut();

static BlendFunc: BLENDFUNCTION = BLENDFUNCTION {
    BlendOp: (AC_SRC_OVER) as u8,
    BlendFlags: 0,
    SourceConstantAlpha: 255,
    AlphaFormat: (AC_SRC_ALPHA) as u8,
};

static mut hKernel32Vista: HMODULE = core::ptr::null_mut();

static mut pGetTickCount64: PFGETTICKCOUNT64 = None;

static mut szUptimeFormat: [WCHAR; 64] = [0; 64];
#[no_mangle]
pub unsafe extern "C" fn ShowLastWin32Error(hWndOwner: HWND) {
    let mut lpMsg: LPTSTR = core::ptr::null_mut();
    let LastError: u32;

    LastError = GetLastError();
    if LastError == ERROR_SUCCESS {
        return;
    }

    if FormatMessage(
        FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
        core::ptr::null_mut(),
        LastError,
        (LANG_USER_DEFAULT) as u32,
        (&mut lpMsg as *mut LPTSTR) as LPTSTR,
        0,
        core::ptr::null_mut(),
    ) == 0
    {
        return;
    }

    MessageBox(hWndOwner, lpMsg, core::ptr::null_mut(), MB_OK | MB_ICONERROR);
    LocalFree((lpMsg).cast());
}
unsafe extern "C" fn InitLogo(hwndDlg: HWND) {
    let mut logoBitmap: BITMAP = core::mem::zeroed();
    let mut maskBitmap: BITMAP = core::mem::zeroed();
    let mut bmpi: BITMAPINFO = core::mem::zeroed();
    let mut hDC: HDC;
    let mut hDCLogo: HDC;
    let mut hDCMask: HDC;
    let mut hMask: HBITMAP = core::ptr::null_mut();
    let mut hLogo: HBITMAP = core::ptr::null_mut();
    let mut hAlphaLogo: HBITMAP = core::ptr::null_mut();
    let mut pBits: *mut COLORREF = core::ptr::null_mut();
    let mut line: INT;
    let mut column: INT;

    hDC = GetDC(hwndDlg);
    hDCLogo = CreateCompatibleDC(core::ptr::null_mut());
    hDCMask = CreateCompatibleDC(core::ptr::null_mut());

    'Cleanup: loop {
        if hDC.is_null() || hDCLogo.is_null() || hDCMask.is_null() {
            break 'Cleanup;
        }

        ZeroMemory!(pImgInfo as *mut _ as *mut core::ffi::c_void, core::mem::size_of_val(&*pImgInfo));
        ZeroMemory!(&mut bmpi as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<BITMAPINFO>());

        hLogo = LoadImageW(hApplet, MAKEINTRESOURCEW!(IDB_ROSBMP), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR) as HBITMAP;
        hMask = LoadImageW(hApplet, MAKEINTRESOURCEW!(IDB_ROSMASK), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR) as HBITMAP;

        if hLogo.is_null() || hMask.is_null() {
            break 'Cleanup;
        }

        GetObject(hLogo as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, &mut logoBitmap as *mut _ as *mut core::ffi::c_void);
        GetObject(hMask as *mut core::ffi::c_void, core::mem::size_of::<BITMAP>() as i32, &mut maskBitmap as *mut _ as *mut core::ffi::c_void);

        if logoBitmap.bmHeight != maskBitmap.bmHeight || logoBitmap.bmWidth != maskBitmap.bmWidth {
            break 'Cleanup;
        }

        bmpi.bmiHeader.biSize = (core::mem::size_of::<BITMAPINFO>()) as u32;
        bmpi.bmiHeader.biWidth = logoBitmap.bmWidth;
        bmpi.bmiHeader.biHeight = logoBitmap.bmHeight;
        bmpi.bmiHeader.biPlanes = 1;
        bmpi.bmiHeader.biBitCount = 32;
        bmpi.bmiHeader.biCompression = BI_RGB;
        bmpi.bmiHeader.biSizeImage = 4 * logoBitmap.bmWidth as u32 * logoBitmap.bmHeight as u32;

        hAlphaLogo = CreateDIBSection(hDC, &bmpi, DIB_RGB_COLORS, &mut pBits as *mut *mut COLORREF as *mut *mut core::ffi::c_void, core::ptr::null_mut(), 0);
        if hAlphaLogo.is_null() {
            break 'Cleanup;
        }

        SelectObject(hDCLogo, hLogo as *mut core::ffi::c_void);
        SelectObject(hDCMask, hMask as *mut core::ffi::c_void);

        line = logoBitmap.bmHeight - 1;
        while line >= 0 {
            column = 0;
            while column < logoBitmap.bmWidth {
                let alpha: COLORREF = GetPixel(hDCMask, column, line) & 0xFF;
                let Color: COLORREF = GetPixel(hDCLogo, column, line);
                let r: u32;
                let g: u32;
                let b: u32;

                r = GetRValue!(Color) as u32 * alpha / 255;
                g = GetGValue!(Color) as u32 * alpha / 255;
                b = GetBValue!(Color) as u32 * alpha / 255;

                *pBits = b | (g << 8) | (r << 16) | (alpha << 24);
                pBits = pBits.offset(1);

                column += 1;
            }
            line -= 1;
        }

        (*pImgInfo).hBitmap = hAlphaLogo;
        (*pImgInfo).cxSource = logoBitmap.bmWidth;
        (*pImgInfo).cySource = logoBitmap.bmHeight;
        (*pImgInfo).iBits = (logoBitmap.bmBitsPixel) as i32;
        (*pImgInfo).iPlanes = (logoBitmap.bmPlanes) as i32;

        break 'Cleanup;
    }

    if !hMask.is_null() {
        DeleteObject(hMask as *mut core::ffi::c_void);
    }
    if !hLogo.is_null() {
        DeleteObject(hLogo as *mut core::ffi::c_void);
    }
    if !hDCMask.is_null() {
        DeleteDC(hDCMask);
    }
    if !hDCLogo.is_null() {
        DeleteDC(hDCLogo);
    }
    if !hDC.is_null() {
        ReleaseDC(hwndDlg, hDC);
    }
}
#[no_mangle]
pub unsafe extern "system" fn RosImageProc(hwnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    static mut timerid: u32 = 0;
    static mut top: u32 = 0;
    static mut offset: u32 = 0;
    static mut hCreditsBitmap: HBITMAP = core::ptr::null_mut();

    match uMsg {
        WM_LBUTTONDBLCLK => {
            if wParam & (MK_CONTROL | MK_SHIFT) as WPARAM != 0 {
                if timerid == 0 {
                    let mut hDC: HDC;
                    let mut hCreditsDC: HDC = core::ptr::null_mut();
                    let mut hLogoDC: HDC = core::ptr::null_mut();
                    let mut hFont: HFONT = core::ptr::null_mut();
                    let mut ncm: NONCLIENTMETRICS = core::mem::zeroed();
                    let mut rcCredits: RECT = core::mem::zeroed();
                    let mut szCredits: [TCHAR; 2048] = core::mem::zeroed();
                    let mut iDevsHeight: i32;

                    hDC = GetDC(core::ptr::null_mut());
                    if hDC == core::ptr::null_mut() {
                        'Cleanup: loop {
                            if hFont != core::ptr::null_mut() { DeleteObject((hFont).cast()); }
                            if hLogoDC != core::ptr::null_mut() { DeleteDC(hLogoDC); }
                            if hCreditsDC != core::ptr::null_mut() { DeleteDC(hCreditsDC); }
                            if hDC != core::ptr::null_mut() { ReleaseDC(core::ptr::null_mut(), hDC); }
                            break;
                        }
                    } else {
                        top = 0;
                        offset = 0;

                        hCreditsDC = CreateCompatibleDC(hDC);
                        hLogoDC = CreateCompatibleDC(hCreditsDC);

                        if hCreditsDC == core::ptr::null_mut() || hLogoDC == core::ptr::null_mut() {
                            'Cleanup: loop {
                                if hFont != core::ptr::null_mut() { DeleteObject((hFont).cast()); }
                                if hLogoDC != core::ptr::null_mut() { DeleteDC(hLogoDC); }
                                if hCreditsDC != core::ptr::null_mut() { DeleteDC(hCreditsDC); }
                                if hDC != core::ptr::null_mut() { ReleaseDC(core::ptr::null_mut(), hDC); }
                                break;
                            }
                        } else {
                            SetRect(&mut rcCredits, 0, 0, 0, 0);

                            ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICS>() as u32;
                            SystemParametersInfo(SPI_GETNONCLIENTMETRICS, core::mem::size_of::<NONCLIENTMETRICS>() as u32, &mut ncm as *mut _ as *mut _, 0);

                            hFont = CreateFontIndirect(&mut ncm.lfMessageFont);
                            if hFont == core::ptr::null_mut() {
                                'Cleanup: loop {
                                    if hFont != core::ptr::null_mut() { DeleteObject((hFont).cast()); }
                                    if hLogoDC != core::ptr::null_mut() { DeleteDC(hLogoDC); }
                                    if hCreditsDC != core::ptr::null_mut() { DeleteDC(hCreditsDC); }
                                    if hDC != core::ptr::null_mut() { ReleaseDC(core::ptr::null_mut(), hDC); }
                                    break;
                                }
                            } else {
                                SelectObject(hCreditsDC, (hFont).cast());

                                LoadString(hApplet, IDS_DEVS, szCredits.as_mut_ptr(), (core::mem::size_of_val(&szCredits) / core::mem::size_of::<TCHAR>()) as i32);
                                DrawText(hCreditsDC, szCredits.as_mut_ptr(), -1, &mut rcCredits, DT_CALCRECT);

                                iDevsHeight = rcCredits.bottom - rcCredits.top;

                                hCreditsBitmap = CreateBitmap((*pImgInfo).cxSource, (2 * (*pImgInfo).cySource) + iDevsHeight + 1, ((*pImgInfo).iPlanes) as UINT, ((*pImgInfo).iBits) as UINT, core::ptr::null_mut());
                                if hCreditsBitmap == core::ptr::null_mut() {
                                    'Cleanup: loop {
                                        if hFont != core::ptr::null_mut() { DeleteObject((hFont).cast()); }
                                        if hLogoDC != core::ptr::null_mut() { DeleteDC(hLogoDC); }
                                        if hCreditsDC != core::ptr::null_mut() { DeleteDC(hCreditsDC); }
                                        if hDC != core::ptr::null_mut() { ReleaseDC(core::ptr::null_mut(), hDC); }
                                        break;
                                    }
                                } else {
                                    SelectObject(hLogoDC, ((*pImgInfo).hBitmap).cast());
                                    SelectObject(hCreditsDC, (hCreditsBitmap).cast());

                                    offset += (*pImgInfo).cySource as u32;

                                    SetRect(&mut rcCredits, 0, 0, (*pImgInfo).cxSource, (2 * (*pImgInfo).cySource) + iDevsHeight + 1);
                                    FillRect(hCreditsDC, &mut rcCredits, GetSysColorBrush((COLOR_3DFACE) as i32));

                                    SetRect(&mut rcCredits, 0, offset as i32, (*pImgInfo).cxSource, offset as i32 + iDevsHeight + 1);
                                    SetBkMode(hCreditsDC, (TRANSPARENT) as i32);

                                    OffsetRect(&mut rcCredits, 1, 1);
                                    SetTextColor(hCreditsDC, GetSysColor((COLOR_BTNSHADOW) as i32));
                                    DrawText(hCreditsDC, szCredits.as_mut_ptr(), -1, &mut rcCredits, DT_CENTER);

                                    OffsetRect(&mut rcCredits, -1, -1);
                                    SetTextColor(hCreditsDC, GetSysColor((COLOR_WINDOWTEXT) as i32));
                                    DrawText(hCreditsDC, szCredits.as_mut_ptr(), -1, &mut rcCredits, DT_CENTER);

                                    offset += iDevsHeight as u32;

                                    AlphaBlend(hCreditsDC, 0, 0, (*pImgInfo).cxSource, (*pImgInfo).cySource, hLogoDC, 0, 0, (*pImgInfo).cxSource, (*pImgInfo).cySource, BlendFunc);
                                    AlphaBlend(hCreditsDC, 0, offset as i32, (*pImgInfo).cxSource, (*pImgInfo).cySource, hLogoDC, 0, 0, (*pImgInfo).cxSource, (*pImgInfo).cySource, BlendFunc);

                                    timerid = (SetTimer(hwnd, 1, (ANIM_TIME) as UINT, None)) as u32;

                                    if hFont != core::ptr::null_mut() { DeleteObject((hFont).cast()); }
                                    if hLogoDC != core::ptr::null_mut() { DeleteDC(hLogoDC); }
                                    if hCreditsDC != core::ptr::null_mut() { DeleteDC(hCreditsDC); }
                                    if hDC != core::ptr::null_mut() { ReleaseDC(core::ptr::null_mut(), hDC); }
                                }
                            }
                        }
                    }
                }
            }
        }
        WM_LBUTTONDOWN => {
            if timerid != 0 {
                let mut rcCredits: RECT = core::mem::zeroed();
                let hDC = GetDC(hwnd);
                if hDC != core::ptr::null_mut() {
                    GetClientRect(hwnd, &mut rcCredits);
                    SetRect(&mut rcCredits, 0, 0, rcCredits.right, (*pImgInfo).cySource);
                    FillRect(hDC, &mut rcCredits, GetSysColorBrush((COLOR_3DFACE) as i32));
                    ReleaseDC(hwnd, hDC);
                }
                KillTimer(hwnd, (timerid) as UINT_PTR);
                if hCreditsBitmap != core::ptr::null_mut() {
                    DeleteObject((hCreditsBitmap).cast());
                }

                InvalidateRect(hwnd, core::ptr::null_mut(), FALSE);
                top = 0;
                timerid = 0;
            }
        }
        WM_TIMER => {
            top += (ANIM_STEP) as u32;

            if top > offset {
                let mut rcCredits: RECT = core::mem::zeroed();
                let hDC = GetDC(hwnd);
                if hDC != core::ptr::null_mut() {
                    GetClientRect(hwnd, &mut rcCredits);
                    SetRect(&mut rcCredits, 0, 0, rcCredits.right, (*pImgInfo).cySource);
                    FillRect(hDC, &mut rcCredits, GetSysColorBrush((COLOR_3DFACE) as i32));
                    ReleaseDC(hwnd, hDC);
                }
                KillTimer(hwnd, (timerid) as UINT_PTR);
                if hCreditsBitmap != core::ptr::null_mut() {
                    DeleteObject((hCreditsBitmap).cast());
                }

                top = 0;
                timerid = 0;
            }

            InvalidateRect(hwnd, core::ptr::null_mut(), FALSE);
        }
        WM_PAINT => {
            let mut PS: PAINTSTRUCT = core::mem::zeroed();
            let mut hdcMem: HDC;
            let hdc: HDC;
            let mut left: i32;

            hdc = if wParam != 0 { wParam as HDC } else { BeginPaint(hwnd, &mut PS) };

            GetClientRect(hwnd, &mut PS.rcPaint);

            left = (PS.rcPaint.right - (*pImgInfo).cxSource) / 2;
            hdcMem = CreateCompatibleDC(hdc);

            if hdcMem != core::ptr::null_mut() {
                if timerid != 0 {
                    SelectObject(hdcMem, (hCreditsBitmap).cast());
                    BitBlt(hdc, left, PS.rcPaint.top, PS.rcPaint.right - PS.rcPaint.left, PS.rcPaint.top + (*pImgInfo).cySource, hdcMem, 0, top as i32, SRCCOPY);
                } else {
                    SelectObject(hdcMem, ((*pImgInfo).hBitmap).cast());
                    AlphaBlend(hdc, left, PS.rcPaint.top, (*pImgInfo).cxSource, (*pImgInfo).cySource, hdcMem, 0, 0, (*pImgInfo).cxSource, (*pImgInfo).cySource, BlendFunc);
                }

                DeleteDC(hdcMem);
            }

            if wParam == 0 {
                EndPaint(hwnd, &mut PS);
            }
        }
        _ => {}
    }
    (TRUE) as INT_PTR
}
unsafe extern "C" fn SetRegTextData(hwnd: HWND, hKey: HKEY, Value: LPTSTR, uID: u32) {
    let mut lpBuf: LPTSTR = core::ptr::null_mut();
    let mut BufSize: u32 = 0;
    let mut Type: u32 = 0;

    if RegQueryValueEx(hKey, Value, core::ptr::null_mut(), &mut Type, core::ptr::null_mut(), &mut BufSize) == (ERROR_SUCCESS) as i32 {
        lpBuf = HeapAlloc(GetProcessHeap(), 0, (BufSize) as SIZE_T) as LPTSTR;

        if lpBuf.is_null() {
            return;
        }

        if RegQueryValueEx(hKey, Value, core::ptr::null_mut(), &mut Type, lpBuf as PBYTE, &mut BufSize) == (ERROR_SUCCESS) as i32 {
            SetDlgItemText(hwnd, (uID) as i32, lpBuf);
        }

        HeapFree(GetProcessHeap(), 0, lpBuf as *mut _);
    }
}
unsafe extern "C" fn SetProcNameString(hwnd: HWND, hKey: HKEY, Value: LPTSTR, uID1: u32, uID2: u32) -> i32 {
    let mut lpBuf: LPTSTR = core::ptr::null_mut();
    let mut BufSize: u32 = 0;
    let mut Type: u32 = 0;
    let mut Ret: i32 = 0;
    let mut szBuf: [TCHAR; 31] = [0; 31];
    let mut szLastSpace: *mut TCHAR;
    let mut LastSpace: i32 = 0;

    if RegQueryValueEx(hKey, Value, core::ptr::null_mut(), &mut Type, core::ptr::null_mut(), &mut BufSize) == (ERROR_SUCCESS) as i32 {
        lpBuf = HeapAlloc(GetProcessHeap(), 0, (BufSize) as SIZE_T) as LPTSTR;

        if lpBuf.is_null() {
            return 0;
        }

        if RegQueryValueEx(hKey, Value, core::ptr::null_mut(), &mut Type, lpBuf as PBYTE, &mut BufSize) == (ERROR_SUCCESS) as i32 {
            if BufSize > ((30 + 1) * core::mem::size_of::<TCHAR>() as u32) {
                _tcsncpy(szBuf.as_mut_ptr(), lpBuf, 30);
                szBuf[30] = 0;
                szLastSpace = _tcsrchr(szBuf.as_mut_ptr(), (' ' as i32) as u16);

                if szLastSpace.is_null() {
                    LastSpace = 30;
                } else {
                    LastSpace = szLastSpace.offset_from(szBuf.as_ptr()) as i32;
                    szBuf[LastSpace as usize] = 0;
                }

                _tcsncpy(szBuf.as_mut_ptr(), lpBuf, (LastSpace) as usize);

                SetDlgItemText(hwnd, (uID1) as i32, szBuf.as_ptr());
                SetDlgItemText(hwnd, (uID2) as i32, lpBuf.offset((LastSpace + 1) as isize));

                Ret = 2;
            } else {
                SetDlgItemText(hwnd, (uID1) as i32, lpBuf);
                Ret = 1;
            }
        }

        HeapFree(GetProcessHeap(), 0, lpBuf as *mut _);
    }

    return Ret;
}
unsafe extern "C" fn MakeFloatValueString(dFloatValue: *mut f64, szOutput: LPTSTR, szAppend: LPTSTR) {
    let mut szDecimalSeparator: [TCHAR; 4] = [0; 4];

    if GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, szDecimalSeparator.as_mut_ptr(), (core::mem::size_of_val(&szDecimalSeparator) / core::mem::size_of::<TCHAR>()) as i32) > 0 {
        let uIntegral: u32 = *dFloatValue as u32;
        let uDecimals: u8 = ((*dFloatValue * 100 as f64) as u32 - uIntegral * 100) as u8;

        wsprintf(szOutput, _T(w!("%u%s%02u %s") as LPTSTR), uIntegral, szDecimalSeparator.as_ptr(), uDecimals as c_uint, szAppend);
    }
}
unsafe extern "C" fn SetProcSpeed(hwnd: HWND, hKey: HKEY, Value: LPTSTR, uID: u32) -> i32 {
    let mut szBuf: [u16; 64] = [0; 64];
    let mut szHz: [u16; 16] = [0; 16];
    let mut BufSize: u32 = core::mem::size_of::<u32>() as u32;
    let mut Type: u32 = REG_SZ;
    let mut ppi: PROCESSOR_POWER_INFORMATION = core::mem::zeroed();

    ZeroMemory!(
        &mut ppi as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PROCESSOR_POWER_INFORMATION>(),
    );

    if (CallNtPowerInformation(
        ProcessorInformation,
        core::ptr::null_mut(),
        0,
        &mut ppi as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PROCESSOR_POWER_INFORMATION>() as u32,
    ) == STATUS_SUCCESS
        && ppi.CurrentMhz != 0)
        || RegQueryValueEx(
            hKey,
            Value,
            core::ptr::null_mut(),
            &mut Type,
            &mut ppi.CurrentMhz as *mut _ as PBYTE,
            &mut BufSize,
        ) == (ERROR_SUCCESS) as i32
    {
        if ppi.CurrentMhz < 1000 {
            if LoadString(
                hApplet,
                IDS_MEGAHERTZ,
                szHz.as_mut_ptr(),
                (_countof!(szHz)) as i32,
            ) == 0
            {
                return FALSE;
            }
            StringCchPrintf!(
                szBuf.as_mut_ptr(),
                (_countof!(szBuf)) as usize,
                _T(w!("%lu %s") as *const u16),
                ppi.CurrentMhz,
                szHz.as_ptr(),
            );
        } else {
            let mut flt: f64 = ppi.CurrentMhz as f64 / 1000.0;
            if LoadString(
                hApplet,
                IDS_GIGAHERTZ,
                szHz.as_mut_ptr(),
                (_countof!(szHz)) as i32,
            ) == 0
            {
                return FALSE;
            }
            MakeFloatValueString(&mut flt, szBuf.as_mut_ptr(), szHz.as_mut_ptr());
        }

        SetDlgItemText(hwnd, (uID) as i32, szBuf.as_ptr());
        return TRUE;
    }

    FALSE
}
unsafe extern "C" fn GetSystemInformation(hwnd: HWND) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut SysKey: [TCHAR; 64] = [0; 64];
    let mut ProcKey: [TCHAR; 64] = [0; 64];
    let mut MemStat: MEMORYSTATUSEX = core::mem::zeroed();
    let mut Buf: [TCHAR; 32] = [0; 32];
    let mut SMBiosName: [u16; 96] = [0; 96];
    let mut CurMachineLine: i32 = (IDC_MACHINELINE1) as i32;

    let sys_key_str = _T(w!("HARDWARE\\DESCRIPTION\\System") as LPTSTR);
    let proc_key_str = _T(w!("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0") as LPTSTR);

    core::ptr::copy_nonoverlapping(sys_key_str, (SysKey.as_mut_ptr()).cast(), 64);
    core::ptr::copy_nonoverlapping(proc_key_str, (ProcKey.as_mut_ptr()).cast(), 64);

    if GetSystemName(SMBiosName.as_mut_ptr(), (_countof!(SMBiosName)) as usize) != 0 {
        SetDlgItemText(hwnd, CurMachineLine, SMBiosName.as_mut_ptr());
        CurMachineLine += 1;
    } else {
        if RegOpenKeyEx(HKEY_LOCAL_MACHINE, SysKey.as_ptr(), 0, KEY_READ, &mut hKey) == (ERROR_SUCCESS) as i32 {
            SetRegTextData(hwnd, hKey, _T(w!("Identifier") as LPTSTR), (CurMachineLine) as u32);
            CurMachineLine += 1;
            RegCloseKey(hKey);
        }
    }

    if RegOpenKeyEx(HKEY_LOCAL_MACHINE, ProcKey.as_ptr(), 0, KEY_READ, &mut hKey) == (ERROR_SUCCESS) as i32 {
        let mut PrevMachineLine: i32;

        SetRegTextData(hwnd, hKey, _T(w!("VendorIdentifier") as LPTSTR), (CurMachineLine) as u32);
        CurMachineLine += 1;

        PrevMachineLine = CurMachineLine;
        CurMachineLine += SetProcNameString(hwnd, hKey, _T(w!("ProcessorNameString") as LPTSTR), (CurMachineLine) as u32, (CurMachineLine + 1) as u32);

        if CurMachineLine == PrevMachineLine {
            CurMachineLine += SetProcNameString(hwnd, hKey, _T(w!("Identifier") as LPTSTR), (CurMachineLine) as u32, (CurMachineLine + 1) as u32);
        }

        if SetProcSpeed(hwnd, hKey, _T(w!("~MHz") as LPTSTR), (CurMachineLine) as u32) != 0 {
            CurMachineLine += 1;
        }
        RegCloseKey(hKey);
    }

    MemStat.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as u32;

    if GlobalMemoryStatusEx(&mut MemStat) != 0 {
        let mut szStr: [TCHAR; 32] = [0; 32];
        let mut dTotalPhys: f64;

        if MemStat.ullTotalPhys > 1024 * 1024 * 1024 {
            let mut i: u32 = 0;
            let uStrId: [u32; 3] = [IDS_GIGABYTE, IDS_TERABYTE, IDS_PETABYTE];

            MemStat.ullTotalPhys /= 1024 * 1024;

            if MemStat.ullTotalPhys > 1024 * 1024 {
                MemStat.ullTotalPhys /= 1024;
                i += 1;

                if MemStat.ullTotalPhys > 1024 * 1024 {
                    MemStat.ullTotalPhys /= 1024;
                    i += 1;

                    dTotalPhys = MemStat.ullTotalPhys as f64 / 1024 as f64;
                } else {
                    dTotalPhys = MemStat.ullTotalPhys as f64 / 1024 as f64;
                }
            } else {
                dTotalPhys = MemStat.ullTotalPhys as f64 / 1024 as f64;
            }

            LoadString(hApplet, uStrId[i as usize], szStr.as_mut_ptr(), (core::mem::size_of_val(&szStr) / core::mem::size_of::<TCHAR>()) as i32);
            MakeFloatValueString(&mut dTotalPhys, Buf.as_mut_ptr(), szStr.as_mut_ptr());
        } else {
            LoadString(hApplet, IDS_MEGABYTE, szStr.as_mut_ptr(), (core::mem::size_of_val(&szStr) / core::mem::size_of::<TCHAR>()) as i32);
            wsprintf(Buf.as_mut_ptr(), _T(w!("%u %s") as LPTSTR), (MemStat.ullTotalPhys / 1024 / 1024) as u32, szStr.as_ptr());
        }

        SetDlgItemText(hwnd, CurMachineLine, Buf.as_mut_ptr());
    }
}
unsafe extern "C" fn GetSystemVersion(hwnd: HWND) {
    let mut hRosVersion: HWND;
    let mut lenStr: usize;
    let mut lenVersion: usize;
    let pwszVersion: PCWSTR = VERSION_WITH_LEADING_SPACE!();
    let mut pwszStr: PWSTR;

    lenVersion = wcslen(pwszVersion);
    if lenVersion == 0 {
        return;
    }

    hRosVersion = GetDlgItem(hwnd, (IDC_ROSVERSION) as i32);
    if hRosVersion.is_null() {
        return;
    }
    lenStr = (GetWindowTextLengthW(hRosVersion)) as usize;
    lenStr += lenVersion + 1;
    pwszStr = HeapAlloc(
        GetProcessHeap(),
        0,
        (lenStr * core::mem::size_of::<u16>()) as SIZE_T,
    ) as PWSTR;
    if pwszStr.is_null() {
        return;
    }
    GetWindowText(hRosVersion, pwszStr, (lenStr) as i32);

    StringCchCatW(pwszStr, lenStr, pwszVersion);
    SetWindowText(hRosVersion, pwszStr);

    HeapFree(GetProcessHeap(), 0, (pwszStr).cast());
}
unsafe extern "C" fn GetTickCountQPC() -> u64 {
    let mut Counter: LARGE_INTEGER = core::mem::zeroed();
    let mut Frequency: LARGE_INTEGER = core::mem::zeroed();

    QueryPerformanceCounter(&mut Counter);
    QueryPerformanceFrequency(&mut Frequency);

    ((Counter.QuadPart * 1000) / Frequency.QuadPart) as u64
}
unsafe extern "C" fn GetSystemUptime(hwndDlg: HWND) {
    let mut hUptimeLabel: HWND;
    let mut cMilliseconds: u64;
    let mut cSeconds: u32;
    let mut szBuf: [u16; 64] = [0; 64];

    hUptimeLabel = GetDlgItem(hwndDlg, (IDC_UPTIME) as i32);
    if hUptimeLabel.is_null() {
        return;
    }

    if pGetTickCount64.is_some() {
        cMilliseconds = (pGetTickCount64.unwrap_unchecked())();
    } else {
        cMilliseconds = GetTickCountQPC();
    }

    cSeconds = (cMilliseconds / 1000) as u32;
    StringCchPrintfW!(szBuf.as_mut_ptr(), (_countof!(szBuf)) as usize, szUptimeFormat.as_ptr(),
                     cSeconds / (60*60*24),
                     (cSeconds / (60*60)) % 24,
                     (cSeconds / 60) % 60,
                     cSeconds % 60);

    SetWindowTextW(hUptimeLabel, szBuf.as_ptr());

    SetTimer(hwndDlg, (ID_SYSUPTIME_UPDATE_TIMER) as UINT_PTR, (1000 - (cMilliseconds % 1000)) as UINT, None);
}
unsafe extern "C" fn InitSystemUptime(hwndDlg: HWND) {
    let mut hKernel32: HMODULE;

    if LoadStringW(hApplet, IDS_UPTIME_FORMAT, szUptimeFormat.as_mut_ptr(), (_countof!(szUptimeFormat)) as i32) == 0 {
        return;
    }

    hKernel32 = GetModuleHandleW(w!("kernel32.dll") as *const u16);
    if !hKernel32.is_null() {
        pGetTickCount64 = core::mem::transmute(GetProcAddress(hKernel32, b"GetTickCount64\0".as_ptr() as *const i8));
        if pGetTickCount64.is_none() {
            hKernel32Vista = LoadLibraryW(w!("kernel32_vista.dll") as *const u16);
            if !hKernel32Vista.is_null() {
                pGetTickCount64 = core::mem::transmute(GetProcAddress(hKernel32Vista, b"GetTickCount64\0".as_ptr() as *const i8));
            }
        }
    }

    GetSystemUptime(hwndDlg);
}
#[no_mangle]
pub unsafe extern "system" fn GeneralPageProc(hwndDlg: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);

    match uMsg {
        WM_INITDIALOG => {
            pImgInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<IMGINFO>()) as SIZE_T) as *mut IMGINFO;
            if pImgInfo.is_null() {
                EndDialog(hwndDlg, 0);
                return (FALSE) as INT_PTR;
            }

            InitLogo(hwndDlg);
            SetWindowLongPtr(GetDlgItem(hwndDlg, (IDC_ROSIMG) as i32), GWLP_WNDPROC, RosImageProc as LONG_PTR);
            GetSystemInformation(hwndDlg);
            GetSystemVersion(hwndDlg);
            InitSystemUptime(hwndDlg);
        }

        WM_DESTROY => {
            KillTimer(hwndDlg, (ID_SYSUPTIME_UPDATE_TIMER) as UINT_PTR);

            if !hKernel32Vista.is_null() {
                FreeLibrary(hKernel32Vista);
            }

            HeapFree(GetProcessHeap(), 0, pImgInfo as *mut _);
        }

        WM_TIMER => {
            if wParam == (ID_SYSUPTIME_UPDATE_TIMER) as WPARAM {
                GetSystemUptime(hwndDlg);
            }
        }

        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_LICENCE) as u16 {
                DialogBox!(hApplet, MAKEINTRESOURCE!(IDD_LICENCE), hwndDlg, Some(LicenceDlgProc));

                return (TRUE) as INT_PTR;
            }
        }

        WM_DRAWITEM => {
            let lpDrawItem = lParam as LPDRAWITEMSTRUCT;

            if (*lpDrawItem).CtlID == IDC_ROSIMG {
                let mut hdcMem: HDC;
                let left: i32;

                left = ((*lpDrawItem).rcItem.right - (*pImgInfo).cxSource) / 2;

                hdcMem = CreateCompatibleDC((*lpDrawItem).hDC);
                if !hdcMem.is_null() {
                    SelectObject(hdcMem, ((*pImgInfo).hBitmap).cast());
                    BitBlt(
                        (*lpDrawItem).hDC,
                        left,
                        (*lpDrawItem).rcItem.top,
                        (*lpDrawItem).rcItem.right - (*lpDrawItem).rcItem.left,
                        (*lpDrawItem).rcItem.bottom - (*lpDrawItem).rcItem.top,
                        hdcMem,
                        0,
                        0,
                        SRCCOPY
                    );
                    DeleteDC(hdcMem);
                }
            }
            return (TRUE) as INT_PTR;
        }

        WM_NOTIFY => {
            let nmhdr = lParam as *mut NMHDR;

            if (*nmhdr).idFrom == (IDC_ROSHOMEPAGE_LINK) as UINT_PTR && (*nmhdr).code == NM_CLICK {
                let nml = nmhdr as *mut PNMLINK;

                ShellExecuteW(hwndDlg, w!("open"), (*(*nml)).item.szUrl.as_ptr(), core::ptr::null(), core::ptr::null(), (SW_SHOWNORMAL) as INT);
            }
        }

        _ => {}
    }

    (FALSE) as INT_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/general.c
//  * PURPOSE:     General System Information
//  * COPYRIGHT:   Copyright Thomas Weidenmueller <w3seek@reactos.org>
//  *              Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *              Copyright 2006-2007 Colin Finck <mail@colinfinck.de>
//  *
//  */
//
// #include "precomp.h"
//
// #include <winnls.h>
// #include <powrprof.h>
// #include <buildno.h>
//
// #define ANIM_STEP 2
// #define ANIM_TIME 50
// #define ID_SYSUPTIME_UPDATE_TIMER 1
//
// typedef struct _IMGINFO
// {
//     HBITMAP hBitmap;
//     INT cxSource;
//     INT cySource;
//     INT iPlanes;
//     INT iBits;
// } IMGINFO, *PIMGINFO;
//
// typedef ULONGLONG (WINAPI *PFGETTICKCOUNT64)(VOID);
//
// static PIMGINFO pImgInfo;
// static const BLENDFUNCTION BlendFunc = {AC_SRC_OVER, 0, 255, AC_SRC_ALPHA};
// static HMODULE hKernel32Vista = NULL;
// static PFGETTICKCOUNT64 pGetTickCount64 = NULL;
// static WCHAR szUptimeFormat[64];
//
// VOID ShowLastWin32Error(HWND hWndOwner)
// {
//     LPTSTR lpMsg;
//     DWORD LastError;
//
//     LastError = GetLastError();
//     if (LastError == ERROR_SUCCESS)
//         return;
//
//     if (!FormatMessage(FORMAT_MESSAGE_ALLOCATE_BUFFER |
//                        FORMAT_MESSAGE_FROM_SYSTEM |
//                        FORMAT_MESSAGE_IGNORE_INSERTS,
//                        NULL,
//                        LastError,
//                        LANG_USER_DEFAULT,
//                        (LPTSTR)&lpMsg,
//                        0, NULL))
//     {
//         return;
//     }
//
//     MessageBox(hWndOwner, lpMsg, NULL, MB_OK | MB_ICONERROR);
//     LocalFree(lpMsg);
// }
//
//
// static VOID InitLogo(HWND hwndDlg)
// {
//     BITMAP logoBitmap;
//     BITMAP maskBitmap;
//     BITMAPINFO bmpi;
//     HDC hDC, hDCLogo, hDCMask;
//     HBITMAP hMask = NULL, hLogo = NULL;
//     HBITMAP hAlphaLogo = NULL;
//     COLORREF *pBits;
//     INT line, column;
//
//     hDC = GetDC(hwndDlg);
//     hDCLogo = CreateCompatibleDC(NULL);
//     hDCMask = CreateCompatibleDC(NULL);
//
//     if (hDC == NULL || hDCLogo == NULL || hDCMask == NULL)
//         goto Cleanup;
//
//     ZeroMemory(pImgInfo, sizeof(*pImgInfo));
//     ZeroMemory(&bmpi, sizeof(bmpi));
//
//     hLogo = (HBITMAP)LoadImageW(hApplet, MAKEINTRESOURCEW(IDB_ROSBMP), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//     hMask = (HBITMAP)LoadImageW(hApplet, MAKEINTRESOURCEW(IDB_ROSMASK), IMAGE_BITMAP, 0, 0, LR_DEFAULTCOLOR);
//
//     if (hLogo == NULL || hMask == NULL)
//         goto Cleanup;
//
//     GetObject(hLogo, sizeof(logoBitmap), &logoBitmap);
//     GetObject(hMask, sizeof(maskBitmap), &maskBitmap);
//
//     if (logoBitmap.bmHeight != maskBitmap.bmHeight || logoBitmap.bmWidth != maskBitmap.bmWidth)
//         goto Cleanup;
//
//     bmpi.bmiHeader.biSize = sizeof(BITMAPINFO);
//     bmpi.bmiHeader.biWidth = logoBitmap.bmWidth;
//     bmpi.bmiHeader.biHeight = logoBitmap.bmHeight;
//     bmpi.bmiHeader.biPlanes = 1;
//     bmpi.bmiHeader.biBitCount = 32;
//     bmpi.bmiHeader.biCompression = BI_RGB;
//     bmpi.bmiHeader.biSizeImage = 4 * logoBitmap.bmWidth * logoBitmap.bmHeight;
//
//     /* Create a premultiplied bitmap */
//     hAlphaLogo = CreateDIBSection(hDC, &bmpi, DIB_RGB_COLORS, (PVOID*)&pBits, 0, 0);
//     if (!hAlphaLogo)
//         goto Cleanup;
//
//     SelectObject(hDCLogo, hLogo);
//     SelectObject(hDCMask, hMask);
//
//     for (line = logoBitmap.bmHeight - 1; line >= 0; line--)
//     {
//         for (column = 0; column < logoBitmap.bmWidth; column++)
//         {
//             COLORREF alpha = GetPixel(hDCMask, column, line) & 0xFF;
//             COLORREF Color = GetPixel(hDCLogo, column, line);
//             DWORD r, g, b;
//
//             r = GetRValue(Color) * alpha / 255;
//             g = GetGValue(Color) * alpha / 255;
//             b = GetBValue(Color) * alpha / 255;
//
//             *pBits++ = b | (g << 8) | (r << 16) | (alpha << 24);
//         }
//     }
//
//     pImgInfo->hBitmap = hAlphaLogo;
//     pImgInfo->cxSource = logoBitmap.bmWidth;
//     pImgInfo->cySource = logoBitmap.bmHeight;
//     pImgInfo->iBits = logoBitmap.bmBitsPixel;
//     pImgInfo->iPlanes = logoBitmap.bmPlanes;
//
// Cleanup:
//     if (hMask != NULL) DeleteObject(hMask);
//     if (hLogo != NULL) DeleteObject(hLogo);
//     if (hDCMask != NULL) DeleteDC(hDCMask);
//     if (hDCLogo != NULL) DeleteDC(hDCLogo);
//     if (hDC != NULL) ReleaseDC(hwndDlg, hDC);
// }
//
// LRESULT __stdcall RosImageProc(HWND hwnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     static UINT timerid = 0, top = 0, offset;
//     static HBITMAP hCreditsBitmap;
//
//     switch (uMsg)
//     {
//         case WM_LBUTTONDBLCLK:
//             if (wParam & (MK_CONTROL | MK_SHIFT))
//             {
//                 if (timerid == 0)
//                 {
//                     HDC hDC;
//                     HDC hCreditsDC = NULL, hLogoDC = NULL;
//                     HFONT hFont = NULL;
//                     NONCLIENTMETRICS ncm;
//                     RECT rcCredits;
//                     TCHAR szCredits[2048];
//                     INT iDevsHeight;
//
//                     hDC = GetDC(NULL);
//                     if (hDC == NULL)
//                         goto Cleanup;
//
//                     top = 0;
//                     offset = 0;
//
//                     hCreditsDC = CreateCompatibleDC(hDC);
//                     hLogoDC = CreateCompatibleDC(hCreditsDC);
//
//                     if (hCreditsDC == NULL || hLogoDC == NULL)
//                         goto Cleanup;
//
//                     SetRect(&rcCredits, 0, 0, 0, 0);
//
//                     ncm.cbSize = sizeof(NONCLIENTMETRICS);
//                     SystemParametersInfo(SPI_GETNONCLIENTMETRICS, sizeof(NONCLIENTMETRICS), &ncm, 0);
//
//                     hFont = CreateFontIndirect(&ncm.lfMessageFont);
//                     if (!hFont)
//                         goto Cleanup;
//                     SelectObject(hCreditsDC, hFont);
//
//                     LoadString(hApplet, IDS_DEVS, szCredits, sizeof(szCredits) / sizeof(TCHAR));
//                     DrawText(hCreditsDC, szCredits, -1, &rcCredits, DT_CALCRECT);
//
//                     iDevsHeight = rcCredits.bottom - rcCredits.top;
//
//                     hCreditsBitmap = CreateBitmap(pImgInfo->cxSource, (2 * pImgInfo->cySource) + iDevsHeight + 1, pImgInfo->iPlanes, pImgInfo->iBits, NULL);
//                     if (!hCreditsBitmap)
//                         goto Cleanup;
//
//                     SelectObject(hLogoDC, pImgInfo->hBitmap);
//                     SelectObject(hCreditsDC, hCreditsBitmap);
//
//                     offset += pImgInfo->cySource;
//
//                     SetRect(&rcCredits, 0, 0, pImgInfo->cxSource, (2 * pImgInfo->cySource) + iDevsHeight + 1);
//                     FillRect(hCreditsDC, &rcCredits, GetSysColorBrush(COLOR_3DFACE));
//
//                     SetRect(&rcCredits, 0, offset, pImgInfo->cxSource, offset + iDevsHeight + 1);
//                     SetBkMode(hCreditsDC, TRANSPARENT);
//
//                     OffsetRect(&rcCredits, 1, 1);
//                     SetTextColor(hCreditsDC, GetSysColor(COLOR_BTNSHADOW));
//                     DrawText(hCreditsDC, szCredits, -1, &rcCredits, DT_CENTER);
//
//                     OffsetRect(&rcCredits, -1, -1);
//                     SetTextColor(hCreditsDC, GetSysColor(COLOR_WINDOWTEXT));
//                     DrawText(hCreditsDC, szCredits, -1, &rcCredits, DT_CENTER);
//
//                     offset += iDevsHeight;
//
//                     AlphaBlend(hCreditsDC, 0, 0, pImgInfo->cxSource, pImgInfo->cySource, hLogoDC, 0,  0, pImgInfo->cxSource, pImgInfo->cySource, BlendFunc);
//                     AlphaBlend(hCreditsDC, 0, offset, pImgInfo->cxSource, pImgInfo->cySource, hLogoDC, 0,  0, pImgInfo->cxSource, pImgInfo->cySource, BlendFunc);
//
//                     timerid = SetTimer(hwnd, 1, ANIM_TIME, NULL);
//
// Cleanup:
//                     if (hFont != NULL) DeleteObject(hFont);
//                     if (hLogoDC != NULL) DeleteDC(hLogoDC);
//                     if (hCreditsDC != NULL) DeleteDC(hCreditsDC);
//                     if (hDC != NULL) ReleaseDC(NULL, hDC);
//                 }
//             }
//             break;
//         case WM_LBUTTONDOWN:
//             if (timerid)
//             {
//                 RECT rcCredits;
//                 HDC hDC = GetDC(hwnd);
//                 if (hDC != NULL)
//                 {
//                     GetClientRect(hwnd, &rcCredits);
//                     SetRect(&rcCredits, 0, 0, rcCredits.right, pImgInfo->cySource);
//                     FillRect(hDC, &rcCredits, GetSysColorBrush(COLOR_3DFACE));
//                     ReleaseDC(hwnd, hDC);
//                 }
//                 KillTimer(hwnd, timerid);
//                 if (hCreditsBitmap != NULL)
//                     DeleteObject(hCreditsBitmap);
//
//                 InvalidateRect(hwnd, NULL, FALSE);
//                 top = 0;
//                 timerid = 0;
//             }
//             break;
//         case WM_TIMER:
//             top += ANIM_STEP;
//
//             if (top > offset)
//             {
//                 RECT rcCredits;
//                 HDC hDC = GetDC(hwnd);
//                 if (hDC != NULL)
//                 {
//                     GetClientRect(hwnd, &rcCredits);
//                     SetRect(&rcCredits, 0, 0, rcCredits.right, pImgInfo->cySource);
//                     FillRect(hDC, &rcCredits, GetSysColorBrush(COLOR_3DFACE));
//                     ReleaseDC(hwnd, hDC);
//                 }
//                 KillTimer(hwnd, timerid);
//                 if (hCreditsBitmap != NULL)
//                     DeleteObject(hCreditsBitmap);
//
//                 top = 0;
//                 timerid = 0;
//             }
//
//             InvalidateRect(hwnd, NULL, FALSE);
//             break;
//         case WM_PAINT:
//         {
//             PAINTSTRUCT PS;
//             HDC hdcMem, hdc;
//             LONG left;
//
//             hdc = wParam != 0 ? (HDC)wParam : BeginPaint(hwnd, &PS);
//
//             GetClientRect(hwnd, &PS.rcPaint);
//
//             /* Position image in center of dialog */
//             left = (PS.rcPaint.right - pImgInfo->cxSource) / 2;
//             hdcMem = CreateCompatibleDC(hdc);
//
//             if (hdcMem != NULL)
//             {
//                 if(timerid != 0)
//                 {
//                     SelectObject(hdcMem, hCreditsBitmap);
//                     BitBlt(hdc, left, PS.rcPaint.top, PS.rcPaint.right - PS.rcPaint.left, PS.rcPaint.top + pImgInfo->cySource, hdcMem, 0, top, SRCCOPY);
//                 }
//                 else
//                 {
//                     SelectObject(hdcMem, pImgInfo->hBitmap);
//                     AlphaBlend(hdc, left, PS.rcPaint.top, pImgInfo->cxSource, pImgInfo->cySource, hdcMem, 0, 0, pImgInfo->cxSource, pImgInfo->cySource, BlendFunc);
//                 }
//
//                 DeleteDC(hdcMem);
//             }
//
//             if (wParam == 0)
//                 EndPaint(hwnd,&PS);
//             break;
//         }
//     }
//     return TRUE;
// }
//
// static VOID SetRegTextData(HWND hwnd, HKEY hKey, LPTSTR Value, UINT uID)
// {
//     LPTSTR lpBuf = NULL;
//     DWORD BufSize = 0;
//     DWORD Type;
//
//     if (RegQueryValueEx(hKey, Value, NULL, &Type, NULL, &BufSize) == ERROR_SUCCESS)
//     {
//         lpBuf = HeapAlloc(GetProcessHeap(), 0, BufSize);
//
//         if (!lpBuf)
//             return;
//
//         if (RegQueryValueEx(hKey, Value, NULL, &Type, (PBYTE)lpBuf, &BufSize) == ERROR_SUCCESS)
//             SetDlgItemText(hwnd, uID, lpBuf);
//
//         HeapFree(GetProcessHeap(), 0, lpBuf);
//     }
// }
//
// static INT SetProcNameString(HWND hwnd, HKEY hKey, LPTSTR Value, UINT uID1, UINT uID2)
// {
//     LPTSTR lpBuf = NULL;
//     DWORD BufSize = 0;
//     DWORD Type;
//     INT Ret = 0;
//     TCHAR szBuf[31];
//     TCHAR* szLastSpace;
//     INT LastSpace = 0;
//
//     if (RegQueryValueEx(hKey, Value, NULL, &Type, NULL, &BufSize) == ERROR_SUCCESS)
//     {
//         lpBuf = HeapAlloc(GetProcessHeap(), 0, BufSize);
//
//         if (!lpBuf)
//             return 0;
//
//         if (RegQueryValueEx(hKey, Value, NULL, &Type, (PBYTE)lpBuf, &BufSize) == ERROR_SUCCESS)
//         {
//             if (BufSize > ((30 + 1) * sizeof(TCHAR)))
//             {
//                 /* Wrap the Processor Name String like XP does:                           *
//                 *   - Take the first 30 characters and look for the last space.          *
//                 *     Then wrap the string after this space.                             *
//                 *   - If no space is found, wrap the string after character 30.          *
//                 *                                                                        *
//                 * For example the Processor Name String of a Pentium 4 is right-aligned. *
//                 * With this wrapping the first line looks centered.                      */
//
//                 _tcsncpy(szBuf, lpBuf, 30);
//                 szBuf[30] = 0;
//                 szLastSpace = _tcsrchr(szBuf, ' ');
//
//                 if (szLastSpace == 0)
//                 {
//                     LastSpace = 30;
//                 }
//                 else
//                 {
//                     LastSpace = (szLastSpace - szBuf);
//                     szBuf[LastSpace] = 0;
//                 }
//
//                 _tcsncpy(szBuf, lpBuf, LastSpace);
//
//                 SetDlgItemText(hwnd, uID1, szBuf);
//                 SetDlgItemText(hwnd, uID2, lpBuf+LastSpace+1);
//
//                 /* Return the number of used lines */
//                 Ret = 2;
//             }
//             else
//             {
//                 SetDlgItemText(hwnd, uID1, lpBuf);
//                 Ret = 1;
//             }
//         }
//
//         HeapFree(GetProcessHeap(), 0, lpBuf);
//     }
//
//     return Ret;
// }
//
// static VOID MakeFloatValueString(DOUBLE* dFloatValue, LPTSTR szOutput, LPTSTR szAppend)
// {
//     TCHAR szDecimalSeparator[4];
//
//     /* Get the decimal separator for the current locale */
//     if (GetLocaleInfo(LOCALE_USER_DEFAULT, LOCALE_SDECIMAL, szDecimalSeparator, sizeof(szDecimalSeparator) / sizeof(TCHAR)) > 0)
//     {
//         UCHAR uDecimals;
//         UINT uIntegral;
//
//         /* Show the value with two decimals */
//         uIntegral = (UINT)*dFloatValue;
//         uDecimals = (UCHAR)((UINT)(*dFloatValue * 100) - uIntegral * 100);
//
//         wsprintf(szOutput, _T("%u%s%02u %s"), uIntegral, szDecimalSeparator, uDecimals, szAppend);
//     }
// }
//
// static BOOL SetProcSpeed(HWND hwnd, HKEY hKey, LPTSTR Value, UINT uID)
// {
//     TCHAR szBuf[64], szHz[16];
//     DWORD BufSize = sizeof(DWORD);
//     DWORD Type = REG_SZ;
//     PROCESSOR_POWER_INFORMATION ppi;
//
//     ZeroMemory(&ppi, sizeof(ppi));
//
//     if ((CallNtPowerInformation(ProcessorInformation,
//                                 NULL,
//                                 0,
//                                 (PVOID)&ppi,
//                                 sizeof(ppi)) == STATUS_SUCCESS &&
//          ppi.CurrentMhz != 0) || RegQueryValueEx(hKey, Value, NULL, &Type, (PBYTE)&ppi.CurrentMhz, &BufSize) == ERROR_SUCCESS)
//     {
//         if (ppi.CurrentMhz < 1000)
//         {
//             if (!LoadString(hApplet, IDS_MEGAHERTZ, szHz, _countof(szHz)))
//             {
//                 return FALSE;
//             }
//             StringCchPrintf(szBuf, _countof(szBuf), _T("%lu %s"), ppi.CurrentMhz, szHz);
//         }
//         else
//         {
//             double flt = ppi.CurrentMhz / 1000.0;
//             if (!LoadString(hApplet, IDS_GIGAHERTZ, szHz, _countof(szHz)))
//             {
//                 return FALSE;
//             }
//             MakeFloatValueString(&flt, szBuf, szHz);
//         }
//
//         SetDlgItemText(hwnd, uID, szBuf);
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// static VOID GetSystemInformation(HWND hwnd)
// {
//     HKEY hKey;
//     TCHAR SysKey[] = _T("HARDWARE\\DESCRIPTION\\System");
//     TCHAR ProcKey[] = _T("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
//     MEMORYSTATUSEX MemStat;
//     TCHAR Buf[32];
//     WCHAR SMBiosName[96];
//     INT CurMachineLine = IDC_MACHINELINE1;
//
//     /*
//      * Get hardware device name or motherboard name
//      * using information from raw SMBIOS data
//      */
//     if (GetSystemName(SMBiosName, _countof(SMBiosName)))
//     {
//         SetDlgItemText(hwnd, CurMachineLine, SMBiosName);
//         CurMachineLine++;
//     }
//     else
//     {
//         /* If SMBIOS is not available, use System Identifier */
//         if (RegOpenKeyEx(HKEY_LOCAL_MACHINE, SysKey, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//         {
//             SetRegTextData(hwnd, hKey, _T("Identifier"), CurMachineLine);
//             CurMachineLine++;
//             RegCloseKey(hKey);
//         }
//     }
//     /*
//      * Get Processor information
//      * although undocumented, this information is being pulled
//      * directly out of the registry instead of via setupapi as it
//      * contains all the info we need, and should remain static
//      */
//     if (RegOpenKeyEx(HKEY_LOCAL_MACHINE, ProcKey, 0, KEY_READ, &hKey) == ERROR_SUCCESS)
//     {
//         INT PrevMachineLine;
//
//         SetRegTextData(hwnd, hKey, _T("VendorIdentifier"), CurMachineLine);
//         CurMachineLine++;
//
//         PrevMachineLine = CurMachineLine;
//         CurMachineLine += SetProcNameString(hwnd,
//                                             hKey,
//                                             _T("ProcessorNameString"),
//                                             CurMachineLine,
//                                             CurMachineLine + 1);
//
//         if (CurMachineLine == PrevMachineLine)
//         {
//             /* TODO: Try obtaining CPU name from WMI (i.e. CIM_Processor) */
//
//             /* Brand String is not available, use Identifier instead */
//             CurMachineLine += SetProcNameString(hwnd,
//                                                 hKey,
//                                                 _T("Identifier"),
//                                                 CurMachineLine,
//                                                 CurMachineLine + 1);
//         }
//
//         if (SetProcSpeed(hwnd, hKey, _T("~MHz"), CurMachineLine))
//             CurMachineLine++;
//         RegCloseKey(hKey);
//     }
//
//     /* Get total physical RAM */
//     MemStat.dwLength = sizeof(MemStat);
//
//     if (GlobalMemoryStatusEx(&MemStat))
//     {
//         TCHAR szStr[32];
//         double dTotalPhys;
//
//         if (MemStat.ullTotalPhys > 1024 * 1024 * 1024)
//         {
//             UINT i = 0;
//             static const UINT uStrId[] = { IDS_GIGABYTE, IDS_TERABYTE, IDS_PETABYTE};
//
//             // We're dealing with GBs or more
//             MemStat.ullTotalPhys /= 1024 * 1024;
//
//             if (MemStat.ullTotalPhys > 1024 * 1024)
//             {
//                 // We're dealing with TBs or more
//                 MemStat.ullTotalPhys /= 1024;
//                 i++;
//
//                 if (MemStat.ullTotalPhys > 1024 * 1024)
//                 {
//                     // We're dealing with PBs or more
//                     MemStat.ullTotalPhys /= 1024;
//                     i++;
//
//                     dTotalPhys = (double)MemStat.ullTotalPhys / 1024;
//                 }
//                 else
//                 {
//                     dTotalPhys = (double)MemStat.ullTotalPhys / 1024;
//                 }
//             }
//             else
//             {
//                 dTotalPhys = (double)MemStat.ullTotalPhys / 1024;
//             }
//
//             LoadString(hApplet, uStrId[i], szStr, sizeof(szStr) / sizeof(TCHAR));
//             MakeFloatValueString(&dTotalPhys, Buf, szStr);
//         }
//         else
//         {
//             // We're dealing with MBs, don't show any decimals
//             LoadString(hApplet, IDS_MEGABYTE, szStr, sizeof(szStr) / sizeof(TCHAR));
//             wsprintf(Buf, _T("%u %s"), (UINT)MemStat.ullTotalPhys / 1024 / 1024, szStr);
//         }
//
//         SetDlgItemText(hwnd, CurMachineLine, Buf);
//     }
// }
//
// static VOID GetSystemVersion(HWND hwnd)
// {
//     HWND hRosVersion;
//     SIZE_T lenStr, lenVersion;
//     PCWSTR pwszVersion = L" " TEXT(KERNEL_VERSION_RC);
//     PWSTR pwszStr;
//
//     lenVersion = wcslen(pwszVersion);
//     if (lenVersion == 0)
//     {
//         return;
//     }
//
//     hRosVersion = GetDlgItem(hwnd, IDC_ROSVERSION);
//     if (!hRosVersion)
//     {
//         return;
//     }
//     lenStr = GetWindowTextLengthW(hRosVersion);
//     lenStr += lenVersion + 1;
//     pwszStr = HeapAlloc(GetProcessHeap(), 0, lenStr * sizeof(WCHAR));
//     if (!pwszStr)
//     {
//         return;
//     }
//     GetWindowText(hRosVersion, pwszStr, lenStr);
//
//     StringCchCatW(pwszStr, lenStr, pwszVersion);
//     SetWindowText(hRosVersion, pwszStr);
//
//     HeapFree(GetProcessHeap(), 0, pwszStr);
// }
//
// /**
//  * @brief
//  * An equivalent of GetTickCount64, implemented using QueryPerformanceCounter.
//  *
//  * @return
//  * The number of milliseconds that have elapsed since the system was started.
//  */
// static ULONGLONG GetTickCountQPC(VOID)
// {
//     LARGE_INTEGER Counter, Frequency;
//
//     QueryPerformanceCounter(&Counter);
//     QueryPerformanceFrequency(&Frequency);
//
//     return (Counter.QuadPart * 1000) / Frequency.QuadPart;
// }
//
// static VOID GetSystemUptime(HWND hwndDlg)
// {
//     HWND hUptimeLabel;
//     ULONGLONG cMilliseconds;
//     ULONG cSeconds;
//     WCHAR szBuf[64];
//
//     hUptimeLabel = GetDlgItem(hwndDlg, IDC_UPTIME);
//     if (!hUptimeLabel)
//     {
//         return;
//     }
//
//     if (pGetTickCount64)
//     {
//         cMilliseconds = pGetTickCount64();
//     }
//     else
//     {
//         cMilliseconds = GetTickCountQPC();
//     }
//
//     cSeconds = cMilliseconds / 1000;
//     StringCchPrintfW(szBuf, _countof(szBuf), szUptimeFormat,
//                      cSeconds / (60*60*24),     // Days
//                      (cSeconds / (60*60)) % 24, // Hours
//                      (cSeconds / 60) % 60,      // Minutes
//                      cSeconds % 60);            // Seconds
//
//     SetWindowTextW(hUptimeLabel, szBuf);
//
//     /* Set update timer (reset timeout if the timer exists) */
//     SetTimer(hwndDlg, ID_SYSUPTIME_UPDATE_TIMER, 1000 - (cMilliseconds % 1000), NULL);
// }
//
// static VOID InitSystemUptime(HWND hwndDlg)
// {
//     HMODULE hKernel32;
//
//     /* Load time format string */
//     if (LoadStringW(hApplet, IDS_UPTIME_FORMAT, szUptimeFormat, _countof(szUptimeFormat)) == 0)
//     {
//         return;
//     }
//
//     /* Load required DLLs */
//     hKernel32 = GetModuleHandleW(L"kernel32.dll");
//     if (hKernel32)
//     {
//         pGetTickCount64 = (PFGETTICKCOUNT64)GetProcAddress(hKernel32, "GetTickCount64");
//         if (!pGetTickCount64)
//         {
//             hKernel32Vista = LoadLibraryW(L"kernel32_vista.dll");
//             if (hKernel32Vista)
//             {
//                 pGetTickCount64 = (PFGETTICKCOUNT64)GetProcAddress(hKernel32Vista, "GetTickCount64");
//             }
//         }
//     }
//
//     /* Show system uptime and set update timer */
//     GetSystemUptime(hwndDlg);
// }
//
// /* Property page dialog callback */
// INT_PTR __stdcall GeneralPageProc(HWND hwndDlg, UINT uMsg, WPARAM wParam, LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(lParam);
//     UNREFERENCED_PARAMETER(wParam);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pImgInfo = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(IMGINFO));
//             if (pImgInfo == NULL)
//             {
//                 EndDialog(hwndDlg, 0);
//                 return FALSE;
//             }
//
//             InitLogo(hwndDlg);
//             SetWindowLongPtr(GetDlgItem(hwndDlg, IDC_ROSIMG), GWLP_WNDPROC, (LONG_PTR)RosImageProc);
//             GetSystemInformation(hwndDlg);
//             GetSystemVersion(hwndDlg);
//             InitSystemUptime(hwndDlg);
//             break;
//         }
//
//         case WM_DESTROY:
//         {
//             KillTimer(hwndDlg, ID_SYSUPTIME_UPDATE_TIMER);
//
//             if (hKernel32Vista)
//             {
//                 FreeLibrary(hKernel32Vista);
//             }
//
//             HeapFree(GetProcessHeap(), 0, pImgInfo);
//             break;
//         }
//
//         case WM_TIMER:
//         {
//             if (wParam == ID_SYSUPTIME_UPDATE_TIMER)
//             {
//                 /* Update system uptime */
//                 GetSystemUptime(hwndDlg);
//             }
//
//             break;
//         }
//
//         case WM_COMMAND:
//         {
//             if (LOWORD(wParam) == IDC_LICENCE)
//             {
//                 DialogBox(hApplet, MAKEINTRESOURCE(IDD_LICENCE), hwndDlg, LicenceDlgProc);
//
//                 return TRUE;
//             }
//             break;
//         }
//
//         case WM_DRAWITEM:
//         {
//             LPDRAWITEMSTRUCT lpDrawItem = (LPDRAWITEMSTRUCT)lParam;
//
//             if (lpDrawItem->CtlID == IDC_ROSIMG)
//             {
//                 HDC hdcMem;
//                 LONG left;
//
//                 /* Position image in centre of dialog */
//                 left = (lpDrawItem->rcItem.right - pImgInfo->cxSource) / 2;
//
//                 hdcMem = CreateCompatibleDC(lpDrawItem->hDC);
//                 if (hdcMem != NULL)
//                 {
//                     SelectObject(hdcMem, pImgInfo->hBitmap);
//                     BitBlt(lpDrawItem->hDC,
//                            left,
//                            lpDrawItem->rcItem.top,
//                            lpDrawItem->rcItem.right - lpDrawItem->rcItem.left,
//                            lpDrawItem->rcItem.bottom - lpDrawItem->rcItem.top,
//                            hdcMem,
//                            0,
//                            0,
//                            SRCCOPY);
//                     DeleteDC(hdcMem);
//                 }
//             }
//             return TRUE;
//         }
//
//         case WM_NOTIFY:
//         {
//             NMHDR *nmhdr = (NMHDR *)lParam;
//
//             if (nmhdr->idFrom == IDC_ROSHOMEPAGE_LINK && nmhdr->code == NM_CLICK)
//             {
//                 PNMLINK nml = (PNMLINK)nmhdr;
//
//                 ShellExecuteW(hwndDlg, L"open", nml->item.szUrl, NULL, NULL, SW_SHOWNORMAL);
//             }
//             break;
//         }
//
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
