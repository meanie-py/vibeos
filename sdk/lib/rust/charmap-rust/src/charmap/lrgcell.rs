use crate::charmap::charmap::UpdateStatusBar;
use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn SetLrgFont(infoPtr: PMAP) -> HFONT {
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hCombo: HWND = core::ptr::null_mut();
    let mut lpFontName: LPWSTR = core::ptr::null_mut();
    let mut Len: INT = 0;

    hCombo = GetDlgItem((*infoPtr).hParent,
                        (IDC_FONTCOMBO as i32));

    Len = GetWindowTextLengthW(hCombo) as INT;

    if Len != 0 as INT
    {
        lpFontName = HeapAlloc(GetProcessHeap(),
                               0 as DWORD,
                               ((Len + 1) as SIZE_T)
                                   .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T)) as LPWSTR;

        if !lpFontName.is_null()
        {
            SendMessageW(hCombo,
                         WM_GETTEXT as UINT,
                         (Len + 1) as WPARAM,
                         lpFontName as LPARAM);

            ZeroMemory!(core::ptr::addr_of_mut!(lf) as *mut core::ffi::c_void,
                       core::mem::size_of::<LOGFONTW>() as SIZE_T);

            hdc = GetDC((*infoPtr).hLrgWnd);
            lf.lfHeight = (GetDeviceCaps(hdc,
                                         (LOGPIXELSY as i32)) / 2) as LONG;
            ReleaseDC((*infoPtr).hLrgWnd,
                      hdc);

            lf.lfCharSet = DEFAULT_CHARSET as BYTE;
            wcsncpy(core::ptr::addr_of_mut!(lf.lfFaceName) as *mut WCHAR,
                    lpFontName as *const WCHAR,
                    ((core::mem::size_of::<[WCHAR; 32]>() as SIZE_T)
                        .wrapping_div(core::mem::size_of_val(&lf.lfFaceName[0]) as SIZE_T)) as _);

            hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));

            HeapFree(GetProcessHeap(),
                     0 as DWORD,
                     lpFontName as LPVOID);
        }
    }

    return hFont;
}
#[no_mangle]
pub unsafe extern "system" fn LrgCellWndProc(hwnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: PMAP = core::ptr::null_mut();
    let mut Ret: LRESULT = 0 as LRESULT;
    static mut cxClient: INT = 0;
    static mut cyClient: INT = 0;
    static mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    static mut hFont: HFONT = core::ptr::null_mut();
    let mut to_default: bool = false;

    infoPtr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as PMAP;

    if infoPtr.is_null() && uMsg != WM_CREATE {
        to_default = true;
    } else {
        match uMsg {
            WM_CREATE => {
                infoPtr = (*(lParam as LPCREATESTRUCTW)).lpCreateParams as PMAP;

                SetWindowLongPtrW(
                    hwnd,
                    GWLP_USERDATA,
                    infoPtr as LONG_PTR,
                );

                hFont = SetLrgFont(infoPtr);
            }

            WM_SIZE => {
                cxClient = LOWORD!(lParam as DWORD) as INT;
                cyClient = HIWORD!(lParam as DWORD) as INT;

                rc.left = 0 as LONG;
                rc.top = 0 as LONG;
                rc.right = cxClient as LONG;
                rc.bottom = cyClient as LONG;
            }

            WM_PAINT => {
                let mut ps: PAINTSTRUCT = core::mem::zeroed::<PAINTSTRUCT>();
                let mut hdc: HDC = core::ptr::null_mut();
                let mut hOldFont: HFONT = core::ptr::null_mut();

                hdc = BeginPaint(hwnd, &raw mut ps);

                Rectangle(
                    hdc,
                    0,
                    0,
                    cxClient,
                    cyClient,
                );

                hOldFont = SelectObject(hdc, (hFont).cast()) as HFONT;

                DrawTextW(
                    hdc,
                    &raw mut (*(*infoPtr).pActiveCell).ch as LPCWSTR,
                    1,
                    &raw mut rc,
                    (DT_CENTER | DT_VCENTER | DT_SINGLELINE) as UINT,
                );

                SelectObject(hdc, (hOldFont).cast());

                EndPaint(hwnd, &raw mut ps);

                UpdateStatusBar((*(*infoPtr).pActiveCell).ch);
            }

            WM_DESTROY => {
                DeleteObject((hFont).cast());
            }

            _ => {
                to_default = true;
            }
        }
    }

    if to_default {
        Ret = DefWindowProcW(hwnd, uMsg, wParam, lParam);
    }

    return Ret;
}
