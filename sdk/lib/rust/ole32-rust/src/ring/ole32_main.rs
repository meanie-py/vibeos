use crate::prelude::*;



// ---- file-scope preamble (translated) ----
pub const HIMETRIC_INCHES: i32 = 2540;

#[no_mangle]
pub unsafe extern "system" fn OleMetafilePictFromIconAndLabel(
    hIcon: HICON,
    lpszLabel: LPOLESTR,
    lpszSourceFile: LPOLESTR,
    iIconIndex: UINT,
) -> HGLOBAL {
    let mut mfp: METAFILEPICT = core::mem::zeroed();
    let hdc: HDC;
    let hmem: HGLOBAL;
    let mfdata: LPVOID;
    let szIconOnly: &[u8] = b"IconOnly\0";
    let mut text_size: SIZE = core::mem::zeroed();
    let width: INT;
    let icon_width: INT;
    let icon_height: INT;
    let label_offset: INT;
    let mut hdcScreen: HDC;
    let mut lf: LOGFONTW = core::mem::zeroed();
    let font: HFONT;

    TRACE!(
        b"%p %p %s %d\n\0".as_ptr() as *const i8,
        hIcon,
        lpszLabel,
        debugstr_w(lpszLabel),
        iIconIndex,
    );

    if hIcon.is_null() {
        return core::ptr::null_mut();
    }

    if SystemParametersInfoW(
        SPI_GETICONTITLELOGFONT,
        core::mem::size_of::<LOGFONTW>() as u32,
        &mut lf as *mut _ as *mut _,
        0,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    font = CreateFontIndirectW(&lf);
    if font.is_null() {
        return core::ptr::null_mut();
    }

    hdc = CreateMetaFileW(core::ptr::null_mut());
    if hdc.is_null() {
        DeleteObject(font as *mut _);
        return core::ptr::null_mut();
    }

    SelectObject(hdc, font as *mut _);

    ExtEscape(
        hdc,
        MFCOMMENT,
        szIconOnly.len() as i32,
        szIconOnly.as_ptr() as *const i8,
        0,
        core::ptr::null_mut(),
    );

    icon_width = GetSystemMetrics(SM_CXICON);
    icon_height = GetSystemMetrics(SM_CYICON);
    label_offset = icon_height;
    if !lpszLabel.is_null() {
        let screen_old_font: HFONT;
        hdcScreen = GetDC(core::ptr::null_mut());
        screen_old_font = SelectObject(hdcScreen, font as *mut _) as HFONT;
        GetTextExtentPoint32W(
            hdcScreen,
            lpszLabel,
            lstrlenW(lpszLabel),
            &mut text_size,
        );
        SelectObject(hdcScreen, screen_old_font as *mut _);
        ReleaseDC(core::ptr::null_mut(), hdcScreen);

        width = 3 * icon_width;
    } else {
        width = icon_width;
    }

    SetMapMode(hdc, MM_ANISOTROPIC);
    SetWindowOrgEx(hdc, 0, 0, core::ptr::null_mut());
    SetWindowExtEx(hdc, width, label_offset + text_size.cy, core::ptr::null_mut());

    DrawIcon(hdc, (width - icon_width) / 2, 0, hIcon);
    if !lpszLabel.is_null() {
        TextOutW(
            hdc,
            (width - text_size.cx) / 2,
            label_offset,
            lpszLabel,
            lstrlenW(lpszLabel),
        );
    }

    if !lpszSourceFile.is_null() {
        let mut szIconIndex: [i8; 10] = [0; 10];
        let path_length =
            WideCharToMultiByte(CP_ACP, 0, lpszSourceFile, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
        if path_length > 1 {
            let szPath = CoTaskMemAlloc(path_length as SIZE_T) as *mut i8;
            if !szPath.is_null() {
                WideCharToMultiByte(
                    CP_ACP,
                    0,
                    lpszSourceFile,
                    -1,
                    szPath,
                    path_length,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                ExtEscape(hdc, MFCOMMENT, path_length, szPath, 0, core::ptr::null_mut());
                CoTaskMemFree(szPath as *mut _);
            }
        }
        snprintf(
            szIconIndex.as_mut_ptr(),
            10,
            b"%u\0".as_ptr() as *const i8,
            iIconIndex,
        );
        ExtEscape(
            hdc,
            MFCOMMENT,
            (strlen(szIconIndex.as_ptr()) + 1) as i32,
            szIconIndex.as_ptr(),
            0,
            core::ptr::null_mut(),
        );
    }

    mfp.mm = MM_ANISOTROPIC;
    hdcScreen = GetDC(core::ptr::null_mut());
    mfp.xExt = MulDiv(width, HIMETRIC_INCHES, GetDeviceCaps(hdcScreen, LOGPIXELSX));
    mfp.yExt = MulDiv(
        label_offset + text_size.cy,
        HIMETRIC_INCHES,
        GetDeviceCaps(hdcScreen, LOGPIXELSY),
    );
    ReleaseDC(core::ptr::null_mut(), hdcScreen);
    mfp.hMF = CloseMetaFile(hdc);
    DeleteObject(font as *mut _);
    if mfp.hMF.is_null() {
        return core::ptr::null_mut();
    }

    hmem = GlobalAlloc(GMEM_MOVEABLE, core::mem::size_of::<METAFILEPICT>() as SIZE_T);
    if hmem.is_null() {
        DeleteMetaFile(mfp.hMF);
        return core::ptr::null_mut();
    }

    mfdata = GlobalLock(hmem);
    if mfdata.is_null() {
        GlobalFree(hmem);
        DeleteMetaFile(mfp.hMF);
        return core::ptr::null_mut();
    }

    memcpy(
        mfdata,
        &mfp as *const _ as *const _,
        core::mem::size_of::<METAFILEPICT>(),
    );
    GlobalUnlock(hmem);

    TRACE!(b"returning %p\n\0".as_ptr() as *const i8, hmem);

    hmem
}

#[no_mangle]
pub unsafe extern "system" fn CoGetActivationState(
    _guid: GUID,
    _unknown: u32,
    _unknown2: *mut u32,
) -> HRESULT {
    FIXME!(
        b"%s, %x, %p\n\0".as_ptr(),
        debugstr_guid(&guid as *const GUID),
        unknown,
        unknown2,
    );
    0x80004001u32 as i32
}

#[no_mangle]
pub unsafe extern "system" fn CoGetCallState(_unknown: i32, _unknown2: *mut u32) -> HRESULT {
    FIXME!(b"%d, %p\n\0".as_ptr() as *const i8, unknown, unknown2);
    0x80004001u32 as HRESULT
}
