use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut calc_OpenThemeData: type_OpenThemeData =
    unsafe { Some(dummy_OpenThemeData) };

#[no_mangle]
pub static mut calc_CloseThemeData: type_CloseThemeData =
    unsafe { Some(dummy_CloseThemeData) };

#[no_mangle]
pub static mut calc_DrawThemeBackground: type_DrawThemeBackground = unsafe {
    Some(
        dummy_DrawThemeBackground,
    )
};

#[no_mangle]
pub static mut calc_IsAppThemed: type_IsAppThemed =
    unsafe { Some(dummy_IsAppThemed) };

#[no_mangle]
pub static mut calc_IsThemeActive: type_IsThemeActive =
    unsafe { Some(dummy_IsThemeActive) };

#[no_mangle]
pub static mut calc_IsThemeBackgroundPartiallyTransparent:
    type_IsThemeBackgroundPartiallyTransparent = unsafe {
    Some(
        dummy_IsThemeBackgroundPartiallyTransparent,
    )
};

#[no_mangle]
pub static mut calc_DrawThemeParentBackground: type_DrawThemeParentBackground = unsafe {
    Some(dummy_DrawThemeParentBackground)
};

static mut hUxTheme: HMODULE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;
unsafe extern "system" fn dummy_OpenThemeData(
    hwnd: HWND,
    pszClassList: *const WCHAR,
) -> HTHEME {
    NULL
}
unsafe extern "system" fn dummy_CloseThemeData(hTheme: HTHEME) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn dummy_DrawThemeBackground(
    hTheme: HTHEME,
    hdc: HDC,
    iPartId: i32,
    iStateId: i32,
    prc: *const RECT,
    prcClip: *const RECT,
) -> HRESULT {
    E_NOTIMPL
}
unsafe extern "system" fn dummy_IsAppThemed() -> BOOL {
    (FALSE) as i32
}
unsafe extern "system" fn dummy_IsThemeActive() -> BOOL {
    (FALSE) as i32
}
unsafe extern "system" fn dummy_IsThemeBackgroundPartiallyTransparent(
    hTheme: HTHEME,
    iPartId: i32,
    iStateId: i32,
) -> BOOL {
    (FALSE) as i32
}
unsafe extern "system" fn dummy_DrawThemeParentBackground(
    hWnd: HWND,
    hdc: HDC,
    prc: *mut RECT,
) -> HRESULT {
    E_NOTIMPL
}
#[no_mangle]
pub unsafe extern "C" fn Theme_Start(mut hInstance: HINSTANCE) {
    hUxTheme = LoadLibrary(
        ['U' as u16, 'X' as u16, 'T' as u16, 'H' as u16, 'E' as u16, 'M' as u16, 'E' as u16, 0u16]
            .as_ptr(),
    ) as HMODULE;
    if hUxTheme.is_null() {
        return;
    }

    calc_OpenThemeData = ::core::mem::transmute::<FARPROC, type_OpenThemeData>(GetProcAddress(
        hUxTheme as HINSTANCE,
        b"OpenThemeData\0" as *const u8 as LPCSTR,
    ));
    if calc_OpenThemeData.is_none() {
        calc_OpenThemeData =
            Some(dummy_OpenThemeData);
    }

    calc_CloseThemeData = ::core::mem::transmute::<FARPROC, type_CloseThemeData>(GetProcAddress(
        hUxTheme as HINSTANCE,
        b"CloseThemeData\0" as *const u8 as LPCSTR,
    ));
    if calc_CloseThemeData.is_none() {
        calc_CloseThemeData = Some(dummy_CloseThemeData);
    }

    calc_DrawThemeBackground =
        ::core::mem::transmute::<FARPROC, type_DrawThemeBackground>(GetProcAddress(
            hUxTheme as HINSTANCE,
            b"DrawThemeBackground\0" as *const u8 as LPCSTR,
        ));
    if calc_DrawThemeBackground.is_none() {
        calc_DrawThemeBackground = Some(dummy_DrawThemeBackground);
    }

    calc_IsAppThemed = ::core::mem::transmute::<FARPROC, type_IsAppThemed>(GetProcAddress(
        hUxTheme as HINSTANCE,
        b"IsAppThemed\0" as *const u8 as LPCSTR,
    ));
    if calc_IsAppThemed.is_none() {
        calc_IsAppThemed =
            Some(dummy_IsAppThemed);
    }

    calc_IsThemeActive = ::core::mem::transmute::<FARPROC, type_IsThemeActive>(GetProcAddress(
        hUxTheme as HINSTANCE,
        b"IsThemeActive\0" as *const u8 as LPCSTR,
    ));
    if calc_IsThemeActive.is_none() {
        calc_IsThemeActive =
            Some(dummy_IsThemeActive);
    }

    calc_IsThemeBackgroundPartiallyTransparent = ::core::mem::transmute::<
        FARPROC,
        type_IsThemeBackgroundPartiallyTransparent,
    >(GetProcAddress(
        hUxTheme as HINSTANCE,
        b"IsThemeBackgroundPartiallyTransparent\0" as *const u8 as LPCSTR,
    ));
    if calc_IsThemeBackgroundPartiallyTransparent.is_none() {
        calc_IsThemeBackgroundPartiallyTransparent = Some(dummy_IsThemeBackgroundPartiallyTransparent);
    }

    calc_DrawThemeParentBackground =
        ::core::mem::transmute::<FARPROC, type_DrawThemeParentBackground>(GetProcAddress(
            hUxTheme as HINSTANCE,
            b"DrawThemeParentBackground\0" as *const u8 as LPCSTR,
        ));
    if calc_DrawThemeParentBackground.is_none() {
        calc_DrawThemeParentBackground = Some(dummy_DrawThemeParentBackground);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Theme_Stop() {
    if hUxTheme.is_null() {
        return;
    }
    FreeLibrary(hUxTheme);
    hUxTheme = core::ptr::null_mut();
}
