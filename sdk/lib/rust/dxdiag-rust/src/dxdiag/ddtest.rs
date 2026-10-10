// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_EE62DBAFF03C: [u16; 7] = [0x0064u16, 0x0064, 0x0074, 0x0065, 0x0073, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const TEST_DURATION: u32 = 10000;
pub const DD_TEST_WIDTH: u32 = 640;
pub const DD_TEST_HEIGHT: u32 = 480;
pub const DD_TEST_STEP: u32 = 5;
pub const DD_SQUARE_SIZE: u32 = 100;
pub const DD_SQUARE_STEP: u32 = 2;
#[no_mangle]
pub unsafe extern "C" fn StartDDTest(
    lpDevice: *mut GUID,
    hWnd: HWND,
    hInstance: HINSTANCE,
    resTestDescription: INT,
    resResult: INT,
    TestNr: INT,
) -> BOOL {
    let mut szTestDescription: [WCHAR; 256] = [0; 256];
    let mut szCaption: [WCHAR; 256] = [0; 256];
    let mut szResult: [WCHAR; 256] = [0; 256];
    let mut szError: [WCHAR; 256] = [0; 256];
    let mut Result: BOOL = 0;

    LoadStringW(
        hInstance,
        IDS_MAIN_DIALOG as UINT,
        core::ptr::addr_of_mut!(szCaption) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    LoadStringW(
        hInstance,
        IDS_DDTEST_ERROR as UINT,
        core::ptr::addr_of_mut!(szError) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    LoadStringW(
        hInstance,
        resTestDescription as UINT,
        core::ptr::addr_of_mut!(szTestDescription) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    LoadStringW(
        hInstance,
        resResult as UINT,
        core::ptr::addr_of_mut!(szResult) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );

    if (MessageBox(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szTestDescription) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szCaption) as *mut WCHAR as LPCWSTR,
        (MB_YESNO | MB_ICONQUESTION) as UINT,
    )) as u32 == IDNO
    {
        return (FALSE) as BOOL;
    }

    ShowWindow(hWnd, (SW_SHOW) as i32);

    match TestNr {
        1 => {
            Result = DDPrimarySurfaceTest(lpDevice, hWnd);
        }
        2 => {
            Result = DDOffscreenBufferTest(lpDevice, hWnd, (FALSE) as BOOL);
        }
        3 => {
            Result = DDOffscreenBufferTest(lpDevice, hWnd, (TRUE) as BOOL);
        }
        _ => {
            Result = (FALSE) as i32;
        }
    }

    ShowWindow(hWnd, (SW_HIDE) as i32);

    if Result == 0 {
        MessageBox(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(szError) as *mut WCHAR as LPCWSTR,
            core::ptr::addr_of_mut!(szCaption) as *mut WCHAR as LPCWSTR,
            (MB_OK | MB_ICONERROR) as UINT,
        );
        return (FALSE) as BOOL;
    }

    if (MessageBox(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(szResult) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(szCaption) as *mut WCHAR as LPCWSTR,
        (MB_YESNO | MB_ICONQUESTION) as UINT,
    )) as u32 == IDYES
    {
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DDTests(mut lpDevice: *mut GUID) {
    let mut winClass: WNDCLASSEXW = core::mem::zeroed();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut hInstance: HINSTANCE = core::ptr::null_mut();
    let mut szDescription: [WCHAR; 256] = [0 as WCHAR; 256];
    let mut szCaption: [WCHAR; 256] = [0 as WCHAR; 256];

    winClass.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    winClass.style = (CS_DBLCLKS | CS_OWNDC | CS_HREDRAW | CS_VREDRAW) as UINT;
    winClass.lpfnWndProc = Some(DefWindowProcW);
    winClass.cbClsExtra = 0 as core::ffi::c_int;
    winClass.cbWndExtra = 0 as core::ffi::c_int;
    winClass.hInstance = hInstance;
    winClass.hIcon = 0 as HICON;
    winClass.hCursor = 0 as HICON;
    winClass.hbrBackground = GetStockObject((BLACK_BRUSH as i32)) as HBRUSH;
    winClass.lpszMenuName = core::ptr::null();
    winClass.lpszClassName = (RUSTCONV_WSTR_EE62DBAFF03C.as_ptr()).cast();
    winClass.hIconSm = LoadIcon(core::ptr::null_mut(), IDI_APPLICATION);

    if RegisterClassEx(core::ptr::addr_of_mut!(winClass)) == 0 {
        return;
    }

    hWnd = CreateWindowEx(
        0,
        (winClass.lpszClassName).cast(),
        core::ptr::null(),
        WS_POPUP,
        (GetSystemMetrics((SM_CXSCREEN) as i32) - (DD_TEST_WIDTH as i32)) / 2,
        (GetSystemMetrics((SM_CYSCREEN) as i32) - (DD_TEST_HEIGHT as i32)) / 2,
        (DD_TEST_WIDTH as i32),
        (DD_TEST_HEIGHT as i32),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );

    if hWnd.is_null() {
        return;
    }

    LoadStringW(
        hInstance,
        IDS_DDTEST_DESCRIPTION,
        core::ptr::addr_of_mut!(szDescription) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    );
    LoadStringW(
        hInstance,
        IDS_MAIN_DIALOG,
        core::ptr::addr_of_mut!(szCaption) as *mut WCHAR as LPWSTR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
    );
    if (MessageBox(
        core::ptr::null_mut(),
        szDescription.as_ptr(),
        szCaption.as_ptr(),
        (MB_YESNO | MB_ICONQUESTION) as UINT,
    )) as u32 == IDNO
    {
        return;
    }

    StartDDTest(lpDevice, hWnd, hInstance, (IDS_DDPRIMARY_DESCRIPTION) as INT, (IDS_DDPRIMARY_RESULT) as INT, 1);
    StartDDTest(lpDevice, hWnd, hInstance, (IDS_DDOFFSCREEN_DESCRIPTION) as INT, (IDS_DDOFFSCREEN_RESULT) as INT, 2);
    StartDDTest(lpDevice, hWnd, hInstance, (IDS_DDFULLSCREEN_DESCRIPTION) as INT, (IDS_DDFULLSCREEN_RESULT) as INT, 3);

    DestroyWindow(hWnd);
    UnregisterClass((winClass.lpszClassName).cast(), hInstance);
}
#[no_mangle]
pub unsafe extern "C" fn DDPrimarySurfaceTest(lpDevice: *mut GUID, hWnd: HWND) -> BOOL {
    let mut TimerID: UINT_PTR;
    let mut msg: MSG = core::mem::zeroed();

    let mut lpDD: LPDIRECTDRAW = core::ptr::null_mut();
    let mut lpDDSurface: LPDIRECTDRAWSURFACE = core::ptr::null_mut();
    let mut DDSurfaceDesc: DDSURFACEDESC = core::mem::zeroed();

    if DirectDrawCreate(lpDevice, core::ptr::addr_of_mut!(lpDD), core::ptr::null_mut())
        != DD_OK
    {
        return (FALSE) as BOOL;
    }

    if (*(*lpDD).lpVtbl)
        .SetCooperativeLevel
        .unwrap_unchecked()(
        lpDD as *mut IDirectDraw,
        hWnd,
        DDSCL_NORMAL as DWORD,
    ) != DD_OK
    {
        (*(*lpDD).lpVtbl)
            .Release
            .unwrap_unchecked()(lpDD as *mut IDirectDraw);
        return (FALSE) as BOOL;
    }

    /* create our primary surface */
    ZeroMemory!(
        core::ptr::addr_of_mut!(DDSurfaceDesc) as *mut core::ffi::c_void,
        core::mem::size_of::<DDSURFACEDESC>(),
    );
    DDSurfaceDesc.dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;
    DDSurfaceDesc.dwFlags = DDSD_CAPS as DWORD;
    DDSurfaceDesc.ddsCaps.dwCaps = (DDSCAPS_PRIMARYSURFACE | DDSCAPS_3DDEVICE) as DWORD;
    DDSurfaceDesc.dwBackBufferCount = 0 as DWORD;

    if (*(*lpDD).lpVtbl)
        .CreateSurface
        .unwrap_unchecked()(
        lpDD as *mut IDirectDraw,
        core::ptr::addr_of_mut!(DDSurfaceDesc),
        core::ptr::addr_of_mut!(lpDDSurface),
        core::ptr::null_mut(),
    ) != DD_OK
    {
        (*(*lpDD).lpVtbl)
            .Release
            .unwrap_unchecked()(lpDD as *mut IDirectDraw);
        return (FALSE) as BOOL;
    }

    TimerID = SetTimer(
        hWnd,
        1 as UINT_PTR,
        (TEST_DURATION as UINT),
        None,
    );

    loop {
        if PeekMessage(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0 as UINT,
            0 as UINT,
            PM_REMOVE as UINT,
        ) != 0
        {
            if msg.message == WM_TIMER as UINT && TimerID == msg.wParam {
                break;
            }
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
            if msg.message == WM_PAINT as UINT {
                DDRedrawFrame(lpDDSurface);
            }
        }
    }
    KillTimer(hWnd, TimerID);
    (*(*lpDDSurface).lpVtbl)
        .Release
        .unwrap_unchecked()(lpDDSurface as *mut IDirectDrawSurface);
    (*(*lpDD).lpVtbl)
        .Release
        .unwrap_unchecked()(lpDD as *mut IDirectDraw);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DDRedrawFrame(mut lpDDSurface: LPDIRECTDRAWSURFACE) {
    let mut hdc: HDC = core::ptr::null_mut::<HDC__>();
    if (*(*lpDDSurface).lpVtbl)
        .GetDC
        .unwrap_unchecked()(
        lpDDSurface as *mut IDirectDrawSurface, core::ptr::addr_of_mut!(hdc)
    ) == DD_OK
    {
        let mut rct: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let mut BlackBrush: HBRUSH = core::ptr::null_mut::<HBRUSH__>();
        let mut WhiteBrush: HBRUSH = core::ptr::null_mut::<HBRUSH__>();
        let mut Colour: BOOL = (FALSE) as i32;

        rct.left = ((GetSystemMetrics((SM_CXSCREEN) as i32) - (DD_TEST_WIDTH as i32)) / 2) as LONG;
        rct.right = (((GetSystemMetrics((SM_CXSCREEN) as i32) - (DD_TEST_WIDTH as i32)) / 2) + (DD_TEST_WIDTH as i32)) as LONG;
        rct.top = ((GetSystemMetrics((SM_CYSCREEN) as i32) - (DD_TEST_HEIGHT as i32)) / 2) as LONG;
        rct.bottom = (((GetSystemMetrics((SM_CYSCREEN) as i32) - (DD_TEST_HEIGHT as i32)) / 2) + (DD_TEST_HEIGHT as i32)) as LONG;

        BlackBrush = CreateSolidBrush(RGB!(0, 0, 0));
        WhiteBrush = CreateSolidBrush(RGB!(255, 255, 255));

        while (rct.bottom - rct.top) > DD_TEST_STEP as LONG {
            if Colour != 0 {
                FillRect(hdc, core::ptr::addr_of_mut!(rct) as LPCRECT, WhiteBrush);
            } else {
                FillRect(hdc, core::ptr::addr_of_mut!(rct) as LPCRECT, BlackBrush);
            };
            rct.top += DD_TEST_STEP as LONG;
            rct.bottom -= DD_TEST_STEP as LONG;
            rct.left += DD_TEST_STEP as LONG;
            rct.right -= DD_TEST_STEP as LONG;
            Colour = (Colour == 0) as ::core::ffi::c_int as BOOL;
        }
        DeleteObject(BlackBrush as HGDIOBJ);
        DeleteObject(WhiteBrush as HGDIOBJ);
        (*(*lpDDSurface).lpVtbl)
            .ReleaseDC
            .unwrap_unchecked()(
            lpDDSurface as *mut IDirectDrawSurface, hdc
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn DDOffscreenBufferTest(lpDevice: *mut GUID, hWnd: HWND, Fullscreen: BOOL) -> BOOL {
    let mut TimerID: UINT_PTR;
    let mut TimerIDUpdate: UINT_PTR;
    let mut lpDD: LPDIRECTDRAW = core::ptr::null_mut();
    let mut lpDDPrimarySurface: LPDIRECTDRAWSURFACE = core::ptr::null_mut();
    let mut lpDDBackBuffer: LPDIRECTDRAWSURFACE = core::ptr::null_mut();
    let mut DDSurfaceDesc: DDSURFACEDESC = core::mem::zeroed();
    let mut DDBBSurfaceDesc: DDSURFACEDESC = core::mem::zeroed();
    let mut DDSCaps: DDSCAPS = core::mem::zeroed();
    let mut msg: MSG = core::mem::zeroed();
    let mut rectDD: RECT = core::mem::zeroed();
    let mut wndPoint: POINT = core::mem::zeroed();
    let mut posX: INT = 0;
    let mut posY: INT = 10;
    let mut xGain: INT = (DD_SQUARE_STEP as i32);
    let mut yGain: INT = (DD_SQUARE_STEP as i32);

    if DirectDrawCreate(lpDevice, core::ptr::addr_of_mut!(lpDD), core::ptr::null_mut()) != DD_OK {
        return (FALSE) as BOOL;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(DDSurfaceDesc) as *mut core::ffi::c_void, core::mem::size_of::<DDSURFACEDESC>());
    DDSurfaceDesc.dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;

    if Fullscreen != 0 {
        if (*(*lpDD).lpVtbl).SetCooperativeLevel.unwrap_unchecked()(
            lpDD as *mut IDirectDraw,
            hWnd,
            (DDSCL_EXCLUSIVE | DDSCL_FULLSCREEN) as DWORD,
        ) != DD_OK
        {
            (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
            return (FALSE) as BOOL;
        }
        if (*(*lpDD).lpVtbl).SetDisplayMode.unwrap_unchecked()(
            lpDD as *mut IDirectDraw,
            DD_TEST_WIDTH as DWORD,
            DD_TEST_HEIGHT as DWORD,
            32 as DWORD,
        ) != DD_OK
        {
            (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
            return (FALSE) as BOOL;
        }
        DDSurfaceDesc.dwFlags = (DDSD_CAPS | DDSD_BACKBUFFERCOUNT) as DWORD;
        DDSurfaceDesc.ddsCaps.dwCaps =
            (DDSCAPS_PRIMARYSURFACE | DDSCAPS_FLIP | DDSCAPS_COMPLEX) as DWORD;
        DDSurfaceDesc.dwBackBufferCount = 1 as DWORD;
    } else {
        if (*(*lpDD).lpVtbl).SetCooperativeLevel.unwrap_unchecked()(
            lpDD as *mut IDirectDraw,
            hWnd,
            DDSCL_NORMAL as DWORD,
        ) != DD_OK
        {
            (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
            return (FALSE) as BOOL;
        }
        DDSurfaceDesc.dwFlags = DDSD_CAPS as DWORD;
        DDSurfaceDesc.ddsCaps.dwCaps = DDSCAPS_PRIMARYSURFACE as DWORD;
    }

    if (*(*lpDD).lpVtbl).CreateSurface.unwrap_unchecked()(
        lpDD as *mut IDirectDraw,
        core::ptr::addr_of_mut!(DDSurfaceDesc),
        core::ptr::addr_of_mut!(lpDDPrimarySurface),
        core::ptr::null_mut(),
    ) != DD_OK
    {
        (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
        return (FALSE) as BOOL;
    }

    if Fullscreen != 0 {
        DDSCaps.dwCaps = DDSCAPS_BACKBUFFER as DWORD;
        if (*(*lpDDPrimarySurface).lpVtbl).GetAttachedSurface.unwrap_unchecked()(
            lpDDPrimarySurface as *mut IDirectDrawSurface,
            core::ptr::addr_of_mut!(DDSCaps),
            core::ptr::addr_of_mut!(lpDDBackBuffer),
        ) != DD_OK
        {
            (*(*lpDDPrimarySurface).lpVtbl).Release.unwrap_unchecked()(
                lpDDPrimarySurface as *mut IDirectDrawSurface,
            );
            (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
            return (FALSE) as BOOL;
        }
    } else {
        /* create our offscreen back buffer */
        ZeroMemory!(core::ptr::addr_of_mut!(DDBBSurfaceDesc) as *mut core::ffi::c_void, core::mem::size_of::<DDSURFACEDESC>());
        DDBBSurfaceDesc.dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;
        DDBBSurfaceDesc.dwFlags =
            (DDSD_CAPS | DDSD_WIDTH | DDSD_HEIGHT) as DWORD;
        DDBBSurfaceDesc.dwHeight = DD_TEST_HEIGHT as DWORD;
        DDBBSurfaceDesc.dwWidth = DD_TEST_WIDTH as DWORD;
        DDBBSurfaceDesc.ddsCaps.dwCaps = DDSCAPS_OFFSCREENPLAIN as DWORD;

        if (*(*lpDD).lpVtbl).CreateSurface.unwrap_unchecked()(
            lpDD as *mut IDirectDraw,
            core::ptr::addr_of_mut!(DDBBSurfaceDesc),
            core::ptr::addr_of_mut!(lpDDBackBuffer),
            core::ptr::null_mut(),
        ) != DD_OK
        {
            (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);
            (*(*lpDDPrimarySurface).lpVtbl).Release.unwrap_unchecked()(
                lpDDPrimarySurface as *mut IDirectDrawSurface,
            );
            return (FALSE) as BOOL;
        }
        wndPoint.x = 0 as LONG;
        wndPoint.y = 0 as LONG;
        ClientToScreen(hWnd, core::ptr::addr_of_mut!(wndPoint));
        GetClientRect(hWnd, core::ptr::addr_of_mut!(rectDD));
        OffsetRect(
            core::ptr::addr_of_mut!(rectDD),
            wndPoint.x as core::ffi::c_int,
            wndPoint.y as core::ffi::c_int,
        );
    }

    /* set our timers, TimerID - for test timeout, TimerIDUpdate - for frame updating */
    TimerID = SetTimer(hWnd, 1 as UINT_PTR, TEST_DURATION as UINT, None);
    TimerIDUpdate = SetTimer(hWnd, 2 as UINT_PTR, 10 as UINT, None);
    let _ = TimerIDUpdate;

    loop {
        if PeekMessage(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0 as UINT,
            0 as UINT,
            PM_REMOVE as UINT,
        ) != 0
        {
            if msg.message == WM_TIMER as UINT {
                if msg.wParam == TimerID {
                    break;
                } else {
                    DDUpdateFrame(
                        lpDDPrimarySurface,
                        lpDDBackBuffer,
                        Fullscreen,
                        core::ptr::addr_of_mut!(posX),
                        core::ptr::addr_of_mut!(posY),
                        core::ptr::addr_of_mut!(xGain),
                        core::ptr::addr_of_mut!(yGain),
                        core::ptr::addr_of_mut!(rectDD),
                    );
                }
            }
            TranslateMessage(core::ptr::addr_of_mut!(msg));
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }

    (*(*lpDDPrimarySurface).lpVtbl).Release.unwrap_unchecked()(
        lpDDPrimarySurface as *mut IDirectDrawSurface,
    );
    /* backbuffer is released automatically when in fullscreen */
    if Fullscreen == 0 {
        (*(*lpDDBackBuffer).lpVtbl).Release.unwrap_unchecked()(
            lpDDBackBuffer as *mut IDirectDrawSurface,
        );
    }
    (*(*lpDD).lpVtbl).Release.unwrap_unchecked()(lpDD as *mut IDirectDraw);

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DDUpdateFrame(
    lpDDPrimarySurface: LPDIRECTDRAWSURFACE,
    lpDDBackBuffer: LPDIRECTDRAWSURFACE,
    Fullscreen: BOOL,
    posX: *mut INT,
    posY: *mut INT,
    gainX: *mut INT,
    gainY: *mut INT,
    rectDD: *mut RECT,
) {
    let mut hdc: HDC = core::ptr::null_mut();
    let mut DDBlitFx: DDBLTFX = core::mem::zeroed();

    /* clear back buffer and paint it black */
    ZeroMemory!(
        core::ptr::addr_of_mut!(DDBlitFx) as *mut core::ffi::c_void,
        core::mem::size_of::<DDBLTFX>(),
    );
    DDBlitFx.dwSize = core::mem::size_of::<DDBLTFX>() as DWORD;
    DDBlitFx.__bindgen_anon_5.dwFillColor = 0 as DWORD;

    (*(*lpDDBackBuffer).lpVtbl).Blt.unwrap_unchecked()(
        lpDDBackBuffer,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (DDBLT_COLORFILL | DDBLT_WAIT) as DWORD,
        core::ptr::addr_of_mut!(DDBlitFx),
    );

    if (*(*lpDDBackBuffer).lpVtbl).GetDC.unwrap_unchecked()(
        lpDDBackBuffer,
        core::ptr::addr_of_mut!(hdc),
    ) == DD_OK
    {
        let mut rct: RECT = core::mem::zeroed();
        let mut WhiteBrush: HBRUSH = core::ptr::null_mut();

        rct.left = *posX as LONG;
        rct.right = (*posX + (DD_SQUARE_SIZE as i32)) as LONG;
        rct.top = *posY as LONG;
        rct.bottom = (*posY + (DD_SQUARE_SIZE as i32)) as LONG;

        WhiteBrush = CreateSolidBrush(RGB!(
            255,
            255,
            255,
        ));
        FillRect(hdc, core::ptr::addr_of_mut!(rct) as LPCRECT, WhiteBrush);

        if (*posX) as u32 >= (DD_TEST_WIDTH - DD_SQUARE_SIZE) {
            *gainX = -(*gainX);
        }
        if (*posY) as u32 >= (DD_TEST_HEIGHT - DD_SQUARE_SIZE) {
            *gainY = -(*gainY);
        }
        if *posX < 0 {
            *gainX = -(1 as INT) * (*gainX);
        }
        if *posY < 0 {
            *gainY = -(1 as INT) * (*gainY);
        }

        *posX += *gainX;
        *posY += *gainY;

        (*(*lpDDBackBuffer).lpVtbl).ReleaseDC.unwrap_unchecked()(
            lpDDBackBuffer,
            hdc,
        );

        if Fullscreen != 0 {
            (*(*lpDDPrimarySurface).lpVtbl).Flip.unwrap_unchecked()(
                lpDDPrimarySurface,
                core::ptr::null_mut(),
                DDFLIP_WAIT as DWORD,
            );
        } else {
            (*(*lpDDPrimarySurface).lpVtbl).Blt.unwrap_unchecked()(
                lpDDPrimarySurface,
                rectDD as LPRECT,
                lpDDBackBuffer,
                core::ptr::null_mut(),
                DDBLT_WAIT as DWORD,
                core::ptr::null_mut(),
            );
        }
    }
}
