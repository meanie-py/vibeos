use crate::dxdiag::d3dtest7::D3D7Test;
use crate::dxdiag::d3dtest8::D3D8Test;
use crate::dxdiag::d3dtest9::D3D9Test;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4D93845BB551: [u16; 8] = [0x0064u16, 0x0033, 0x0064, 0x0074, 0x0065, 0x0073, 0x0074, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WIDTH: i32 = 800;

pub const HEIGHT: i32 = 600;
#[no_mangle]
pub unsafe extern "C" fn StartD3DTest(lpDevice: *mut GUID, hWnd: HWND, hInstance: HINSTANCE, pszCaption: *mut WCHAR, TestNr: INT) -> BOOL {
    let mut szTestDescriptionRaw: [WCHAR; 256] = [0; 256];
    let mut szTestDescription: [WCHAR; 256] = [0; 256];
    let mut szCaption: [WCHAR; 256] = [0; 256];
    let mut szResult: [WCHAR; 256] = [0; 256];
    let mut szError: [WCHAR; 256] = [0; 256];
    let mut Result: BOOL;

    LoadStringW(
        hInstance,
        IDS_MAIN_DIALOG as UINT,
        core::ptr::addr_of_mut!(szCaption) as *mut WCHAR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    LoadStringW(
        hInstance,
        IDS_DDTEST_ERROR as UINT,
        core::ptr::addr_of_mut!(szError) as *mut WCHAR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    LoadStringW(
        hInstance,
        IDS_D3DTEST_D3Dx as UINT,
        core::ptr::addr_of_mut!(szTestDescriptionRaw) as *mut WCHAR,
        (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as INT,
    );
    //LoadStringW(hInstance, resResult, szResult, sizeof(szResult) / sizeof(WCHAR));

    _swprintf(
        core::ptr::addr_of_mut!(szTestDescription) as *mut WCHAR,
        core::ptr::addr_of_mut!(szTestDescriptionRaw) as *mut WCHAR,
        TestNr,
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

    if TestNr == 7 {
        Result = D3D7Test(lpDevice, hWnd);
    } else if TestNr == 8 {
        Result = D3D8Test(lpDevice, hWnd);
    } else if TestNr == 9 {
        Result = D3D9Test(lpDevice, hWnd);
    } else {
        Result = (FALSE) as i32;
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
unsafe extern "system" fn WindowProc(
    hWnd: HWND,
    msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    DefWindowProc(hWnd, msg, wParam, lParam)
}
#[no_mangle]
pub unsafe extern "C" fn D3DTests(mut lpDevice: *mut GUID) {
    let mut winClass: WNDCLASSEXW = core::mem::zeroed();
    let mut hWnd: HWND = core::ptr::null_mut();
    let mut hInstance: HINSTANCE = GetModuleHandle(core::ptr::null_mut()) as HINSTANCE;
    let mut szDescription: [WCHAR; 256] = [0; 256];
    let mut szCaption: [WCHAR; 256] = [0; 256];

    winClass.cbSize = core::mem::size_of::<WNDCLASSEX>() as UINT;
    winClass.style = (CS_DBLCLKS | CS_OWNDC | CS_HREDRAW | CS_VREDRAW) as UINT;
    winClass.lpfnWndProc =
        Some(WindowProc);
    winClass.cbClsExtra = 0 as ::core::ffi::c_int;
    winClass.cbWndExtra = 0 as ::core::ffi::c_int;
    winClass.hInstance = hInstance;
    winClass.hIcon = 0 as _;
    winClass.hCursor = 0 as _;
    winClass.hbrBackground = GetStockObject((BLACK_BRUSH as i32)) as HBRUSH;
    winClass.lpszMenuName = core::ptr::null();
    winClass.lpszClassName = (RUSTCONV_WSTR_4D93845BB551.as_ptr()).cast();
    winClass.hIconSm = LoadIcon(core::ptr::null_mut(), IDI_APPLICATION as _);

    if RegisterClassEx(&raw mut winClass) == 0 {
        return;
    }

    hWnd = CreateWindowEx(
        0 as _,
        (winClass.lpszClassName).cast(),
        core::ptr::null(),
        WS_POPUP as _,
        (GetSystemMetrics((SM_CXSCREEN) as i32) - WIDTH) / 2,
        (GetSystemMetrics((SM_CYSCREEN) as i32) - HEIGHT) / 2,
        WIDTH,
        HEIGHT,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hInstance,
        core::ptr::null_mut(),
    );

    'cleanup: {
        if hWnd.is_null() {
            break 'cleanup;
        }

        LoadStringW(
            hInstance,
            IDS_D3DTEST_DESCRIPTION as UINT,
            &raw mut szDescription as LPWSTR,
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        );
        LoadStringW(
            hInstance,
            IDS_MAIN_DIALOG as UINT,
            &raw mut szCaption as LPWSTR,
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
        );
        if MessageBox(
            core::ptr::null_mut(),
            &raw mut szDescription as LPCWSTR,
            &raw mut szCaption as LPCWSTR,
            (MB_YESNO | MB_ICONQUESTION) as UINT,
        ) == IDNO as _
        {
            break 'cleanup;
        }

        StartD3DTest(lpDevice, hWnd, hInstance, &raw mut szCaption as *mut WCHAR, 7 as INT);
        StartD3DTest(lpDevice, hWnd, hInstance, &raw mut szCaption as *mut WCHAR, 8 as INT);
        StartD3DTest(lpDevice, hWnd, hInstance, &raw mut szCaption as *mut WCHAR, 9 as INT);
    }

    // cleanup:
    DestroyWindow(hWnd);
    UnregisterClass((winClass.lpszClassName).cast(), hInstance);
}
