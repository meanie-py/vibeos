use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn SHForwardContextMenuMsg(
    pUnk: *mut IUnknown,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    pResult: *mut LRESULT,
    useIContextMenu2: BOOL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut pcmenu3: *mut IContextMenu3 = core::ptr::null_mut();
    let mut pcmenu2: *mut IContextMenu2 = core::ptr::null_mut();

    if pUnk.is_null() {
        return E_FAIL;
    }

    /* First try to use the IContextMenu3 interface */
    hr = IUnknown_QueryInterface(pUnk, &raw const IID_IContextMenu3, &raw mut pcmenu3 as *mut *mut core::ffi::c_void);
    if hr >= 0 {
        hr = IContextMenu3_HandleMenuMsg2(pcmenu3, uMsg, wParam, lParam, pResult);
        IContextMenu3_Release(pcmenu3);
        return hr;
    }

    /* Return failure if we can't use the IContextMenu2 interface */
    if useIContextMenu2 == 0 {
        return hr;
    }

    /* Now try to use the IContextMenu2 interface */
    hr = IUnknown_QueryInterface(pUnk, &raw const IID_IContextMenu2, &raw mut pcmenu2 as *mut *mut core::ffi::c_void);
    if hr < 0 {
        return hr;
    }

    hr = IContextMenu2_HandleMenuMsg(pcmenu2, uMsg, wParam, lParam);
    IContextMenu2_Release(pcmenu2);
    if !pResult.is_null() {
        *pResult = 0;
    }
    return if hr == S_OK { S_FALSE } else { hr };
}

#[no_mangle]
pub unsafe extern "system" fn SHAreIconsEqual(hIcon1: HICON, hIcon2: HICON) -> BOOL {
    let mut iconInfo1: ICONINFO = _ICONINFO {
        fIcon: 0,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: ::core::ptr::null_mut::<HBITMAP__>(),
        hbmColor: ::core::ptr::null_mut::<HBITMAP__>(),
    };
    let mut iconInfo2: ICONINFO = _ICONINFO {
        fIcon: 0,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: ::core::ptr::null_mut::<HBITMAP__>(),
        hbmColor: ::core::ptr::null_mut::<HBITMAP__>(),
    };
    let mut bm1: BITMAP = tagBITMAP {
        bmType: 0,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut bm2: BITMAP = tagBITMAP {
        bmType: 0,
        bmWidth: 0,
        bmHeight: 0,
        bmWidthBytes: 0,
        bmPlanes: 0,
        bmBitsPixel: 0,
        bmBits: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut bSame: BOOL = (FALSE as i32);

    if hIcon1.is_null() || hIcon2.is_null() {
        return (FALSE) as BOOL;
    }

    if GetIconInfo(hIcon1, &raw mut iconInfo1) == 0 {
        return (FALSE) as BOOL;
    }

    if GetIconInfo(hIcon2, &raw mut iconInfo2) == 0 {
        DeleteObject(iconInfo1.hbmColor as HGDIOBJ);
        DeleteObject(iconInfo1.hbmMask as HGDIOBJ);
        return (FALSE) as BOOL;
    }

    GetObjectW(
        iconInfo1.hbmColor as HANDLE,
        ::core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
        &raw mut bm1 as LPVOID,
    );
    GetObjectW(
        iconInfo2.hbmColor as HANDLE,
        ::core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
        &raw mut bm2 as LPVOID,
    );

    if bm1.bmWidth == bm2.bmWidth && bm1.bmHeight == bm2.bmHeight {
        let mut bmi: BITMAPINFO = tagBITMAPINFO {
            bmiHeader: tagBITMAPINFOHEADER {
                biSize: ::core::mem::size_of::<BITMAPINFO>() as DWORD,
                biWidth: bm1.bmWidth,
                biHeight: bm1.bmHeight,
                biPlanes: 1 as WORD,
                biBitCount: 32 as WORD,
                biCompression: 0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [tagRGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }; 1],
        };
        let mut hdc: HDC = GetDC(core::ptr::null_mut());
        let mut size: SIZE_T = (bm1.bmWidth * bm1.bmHeight * 4) as SIZE_T;
        let mut data1: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
        let mut data2: *mut BYTE = ::core::ptr::null_mut::<BYTE>();

        data1 = HeapAlloc(GetProcessHeap(), 0 as DWORD, size) as *mut BYTE;
        data2 = HeapAlloc(GetProcessHeap(), 0 as DWORD, size) as *mut BYTE;

        if !data1.is_null() && !data2.is_null() {
            if GetDIBits(
                hdc,
                iconInfo1.hbmColor,
                0 as UINT,
                bm1.bmHeight as UINT,
                data1 as LPVOID,
                &raw mut bmi,
                DIB_RGB_COLORS as UINT,
            ) != 0
                && GetDIBits(
                    hdc,
                    iconInfo2.hbmColor,
                    0 as UINT,
                    bm2.bmHeight as UINT,
                    data2 as LPVOID,
                    &raw mut bmi,
                    DIB_RGB_COLORS as UINT,
                ) != 0
            {
                bSame = (memcmp(
                    data1 as *const ::core::ffi::c_void,
                    data2 as *const ::core::ffi::c_void,
                    size as size_t,
                ) == 0) as ::core::ffi::c_int as BOOL;
            }
        }
        HeapFree(GetProcessHeap(), 0 as DWORD, data1 as PVOID);
        HeapFree(GetProcessHeap(), 0 as DWORD, data2 as PVOID);

        ReleaseDC(::core::ptr::null_mut::<HWND__>(), hdc);
    }

    DeleteObject(iconInfo1.hbmColor as HGDIOBJ);
    DeleteObject(iconInfo1.hbmMask as HGDIOBJ);

    DeleteObject(iconInfo2.hbmColor as HGDIOBJ);
    DeleteObject(iconInfo2.hbmMask as HGDIOBJ);

    return bSame;
}
