use crate::clipbrd::winutils::ShowLastWin32Error;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn SendClipboardOwnerMessage(bUnicode: BOOL, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut hwndOwner: HWND;

    hwndOwner = GetClipboardOwner();
    if hwndOwner.is_null() {
        return GetLastError() as LRESULT;
    }

    if bUnicode != 0 {
        return SendMessageW(hwndOwner, uMsg, wParam, lParam);
    } else {
        return SendMessageA(hwndOwner, uMsg, wParam, lParam);
    }
}

unsafe extern "C" fn GetPredefinedClipboardFormatName(
    mut hInstance: HINSTANCE,
    mut uFormat: UINT,
    mut Unicode: BOOL,
    mut lpszFormat: PVOID,
    mut cch: UINT,
) -> ::core::ffi::c_int {
    static mut uFormatList: [FORMAT_NAME; 17] = [
        FORMAT_NAME {
            uFormat: CF_TEXT as UINT,
            uResID: STRING_CF_TEXT as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_BITMAP as UINT,
            uResID: STRING_CF_BITMAP as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_METAFILEPICT as UINT,
            uResID: STRING_CF_METAFILEPICT as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_SYLK as UINT,
            uResID: STRING_CF_SYLK as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_DIF as UINT,
            uResID: STRING_CF_DIF as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_TIFF as UINT,
            uResID: 0 as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_OEMTEXT as UINT,
            uResID: STRING_CF_OEMTEXT as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_DIB as UINT,
            uResID: STRING_CF_DIB as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_PALETTE as UINT,
            uResID: STRING_CF_PALETTE as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_PENDATA as UINT,
            uResID: 0 as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_RIFF as UINT,
            uResID: 0 as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_WAVE as UINT,
            uResID: 0 as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_UNICODETEXT as UINT,
            uResID: STRING_CF_UNICODETEXT as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_ENHMETAFILE as UINT,
            uResID: STRING_CF_ENHMETAFILE as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_HDROP as UINT,
            uResID: STRING_CF_HDROP as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_LOCALE as UINT,
            uResID: STRING_CF_LOCALE as UINT,
        },
        FORMAT_NAME {
            uFormat: CF_DIBV5 as UINT,
            uResID: STRING_CF_DIBV5 as UINT,
        },
    ];
    match uFormat {
        CF_TEXT
        | CF_BITMAP
        | CF_METAFILEPICT
        | CF_SYLK
        | CF_DIF
        | CF_OEMTEXT
        | CF_DIB
        | CF_PALETTE
        | CF_UNICODETEXT
        | CF_ENHMETAFILE
        | CF_HDROP
        | CF_LOCALE
        | CF_DIBV5 => {
            if Unicode != 0 {
                return LoadStringW(
                    hInstance,
                    uFormatList[(uFormat - 1) as usize].uResID,
                    lpszFormat as LPWSTR,
                    cch as ::core::ffi::c_int,
                );
            } else {
                return LoadStringA(
                    hInstance,
                    uFormatList[(uFormat - 1) as usize].uResID,
                    lpszFormat as LPSTR,
                    cch as ::core::ffi::c_int,
                );
            }
        }

        _ => {
            return 0;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn RetrieveClipboardFormatName(mut hInstance: HINSTANCE, mut uFormat: UINT, mut Unicode: BOOL, mut lpszFormat: PVOID, mut cch: UINT) {
    ZeroMemory!(
        lpszFormat,
        (cch as usize).wrapping_mul(
            if Unicode != 0 {
                ::core::mem::size_of::<WCHAR>()
            } else {
                ::core::mem::size_of::<CHAR>()
            },
        ),
    );

    /* Check for predefined clipboard format */
    if GetPredefinedClipboardFormatName(hInstance, uFormat, Unicode, lpszFormat, cch) != 0 {
        return;
    }

    /* Check for owner-display format */
    if uFormat == CF_OWNERDISPLAY {
        if SendClipboardOwnerMessage(
            Unicode,
            WM_ASKCBFORMATNAME,
            cch as WPARAM,
            lpszFormat as LPARAM,
        ) != 0
        {
            if Unicode != 0 {
                LoadStringW(
                    hInstance,
                    STRING_CF_UNKNOWN,
                    lpszFormat as LPWSTR,
                    cch as ::core::ffi::c_int,
                );
            } else {
                LoadStringA(
                    hInstance,
                    STRING_CF_UNKNOWN,
                    lpszFormat as LPSTR,
                    cch as ::core::ffi::c_int,
                );
            }
        }
        return;
    }

    /* Fallback to registered clipboard format */
    if Unicode != 0 {
        if GetClipboardFormatNameW(uFormat, lpszFormat as LPWSTR, cch as ::core::ffi::c_int) == 0 {
            LoadStringW(
                hInstance,
                STRING_CF_UNKNOWN,
                lpszFormat as LPWSTR,
                cch as ::core::ffi::c_int,
            );
        }
    } else {
        if GetClipboardFormatNameA(uFormat, lpszFormat as LPSTR, cch as ::core::ffi::c_int) == 0 {
            LoadStringA(
                hInstance,
                STRING_CF_UNKNOWN,
                lpszFormat as LPSTR,
                cch as ::core::ffi::c_int,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn DeleteClipboardContent() {
    if OpenClipboard(Globals.hMainWnd) == 0 {
        ShowLastWin32Error(Globals.hMainWnd);
        return;
    }

    if EmptyClipboard() == 0 {
        ShowLastWin32Error(Globals.hMainWnd);
    }

    CloseClipboard();
}
#[no_mangle]
pub unsafe extern "C" fn GetAutomaticClipboardFormat() -> UINT {
    static mut uFormatList: [UINT; 15] = [
        CF_OWNERDISPLAY,
        CF_UNICODETEXT,
        CF_TEXT,
        CF_OEMTEXT,
        CF_ENHMETAFILE,
        CF_METAFILEPICT,
        CF_DIBV5,
        CF_DIB,
        CF_BITMAP,
        CF_DSPTEXT,
        CF_DSPBITMAP,
        CF_DSPMETAFILEPICT,
        CF_DSPENHMETAFILE,
        CF_PALETTE,
        CF_HDROP
    ];

    return (GetPriorityClipboardFormat(
        core::ptr::addr_of_mut!(uFormatList) as *mut UINT,
        ARRAYSIZE!(uFormatList) as c_int,
    )) as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn IsClipboardFormatSupported(uFormat: UINT) -> BOOL {
    if uFormat == CF_OWNERDISPLAY
        || uFormat == CF_UNICODETEXT
        || uFormat == CF_TEXT
        || uFormat == CF_OEMTEXT
        || uFormat == CF_BITMAP
        || uFormat == CF_ENHMETAFILE
        || uFormat == CF_METAFILEPICT
        || uFormat == CF_DIB
        || uFormat == CF_DIBV5
        || uFormat == CF_HDROP
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn GetLineExtentW(mut lpText: LPCWSTR, mut lpNextLine: *mut LPCWSTR) -> SIZE_T {
    let mut ptr: LPCWSTR = core::ptr::null();
    /* Find the next line of text (lpText is NULL-terminated) */
    /* For newlines, focus only on '\n', not on '\r' */
    ptr = wcschr(lpText as *const u16, 0x000Au16) as LPCWSTR; // Find the end of this line.
    if !ptr.is_null() {
        /* We have the end of this line, go to the next line (ignore the endline in the count) */
        *lpNextLine = ptr.offset(1 as isize);
    } else {
        /* This line was the last one, go pointing to the terminating NULL */
        ptr = lpText.offset(wcslen(lpText) as isize);
        *lpNextLine = ptr;
    }

    return (ptr.offset_from(lpText)) as SIZE_T;
}
#[no_mangle]
pub unsafe extern "C" fn GetLineExtentA(lpText: LPCSTR, lpNextLine: *mut LPCSTR) -> SIZE_T {
    let mut ptr: LPCSTR;

    /* Find the next line of text (lpText is NULL-terminated) */
    /* For newlines, focus only on '\n', not on '\r' */
    ptr = strchr(lpText as *const i8, '\n' as i32) as LPCSTR; // Find the end of this line.
    if !ptr.is_null() {
        /* We have the end of this line, go to the next line (ignore the endline in the count) */
        *lpNextLine = ptr.offset(1 as isize);
    } else {
        /* This line was the last one, go pointing to the terminating NULL */
        ptr = lpText.offset(strlen(lpText as *const i8) as isize);
        *lpNextLine = ptr;
    }

    return ptr.offset_from(lpText) as SIZE_T;
}
#[no_mangle]
pub unsafe extern "C" fn GetClipboardDataDimensions(uFormat: UINT, pRc: PRECT) -> BOOL {
    SetRectEmpty(pRc);

    if OpenClipboard(Globals.hMainWnd) == 0 {
        return (FALSE) as BOOL;
    }

    'sw: {
        if uFormat == CF_DSPBITMAP || uFormat == CF_BITMAP {
            let mut hBitmap: HBITMAP = core::ptr::null_mut();
            let mut bmp: BITMAP = core::mem::zeroed();

            hBitmap = GetClipboardData(CF_BITMAP) as HBITMAP;
            GetObjectW(
                hBitmap as HANDLE,
                core::mem::size_of::<BITMAP>() as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(bmp) as LPVOID,
            );
            SetRect(
                pRc as LPRECT,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                bmp.bmWidth as ::core::ffi::c_int,
                bmp.bmHeight as ::core::ffi::c_int,
            );
        } else if uFormat == CF_DIB || uFormat == CF_DIBV5 {
            let mut hGlobal: HGLOBAL = core::ptr::null_mut();
            let mut lpInfoHeader: LPBITMAPINFOHEADER = core::ptr::null_mut();

            hGlobal = GetClipboardData(uFormat) as HGLOBAL;
            if hGlobal.is_null() {
                break 'sw;
            }

            lpInfoHeader = GlobalLock(hGlobal) as LPBITMAPINFOHEADER;
            if lpInfoHeader.is_null() {
                break 'sw;
            }

            if (*lpInfoHeader).biSize as usize == core::mem::size_of::<BITMAPCOREHEADER>() {
                let lpCoreHeader: LPBITMAPCOREHEADER = lpInfoHeader as LPBITMAPCOREHEADER;
                SetRect(
                    pRc as LPRECT,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    (*lpCoreHeader).bcWidth as ::core::ffi::c_int,
                    (*lpCoreHeader).bcHeight as ::core::ffi::c_int,
                );
            } else if (*lpInfoHeader).biSize as usize == core::mem::size_of::<BITMAPINFOHEADER>()
                || (*lpInfoHeader).biSize as usize == core::mem::size_of::<BITMAPV4HEADER>()
                || (*lpInfoHeader).biSize as usize == core::mem::size_of::<BITMAPV5HEADER>()
            {
                SetRect(
                    pRc as LPRECT,
                    0 as ::core::ffi::c_int,
                    0 as ::core::ffi::c_int,
                    (*lpInfoHeader).biWidth as ::core::ffi::c_int,
                    (if (*lpInfoHeader).biHeight > 0 {
                        (*lpInfoHeader).biHeight
                    } else {
                        -(*lpInfoHeader).biHeight
                    }) as ::core::ffi::c_int,
                );
            } else {
                /* Invalid format */
            }

            GlobalUnlock(hGlobal);
        } else if uFormat == CF_DSPTEXT
            || uFormat == CF_TEXT
            || uFormat == CF_OEMTEXT
            || uFormat == CF_UNICODETEXT
        {
            let mut hDC: HDC = core::ptr::null_mut();
            let mut hGlobal: HGLOBAL = core::ptr::null_mut();
            let mut lpText: PVOID = core::ptr::null_mut();
            let mut ptr: PVOID = core::ptr::null_mut();
            let mut dwSize: DWORD = 0;
            let mut txtSize: SIZE = SIZE {
                cx: 0 as LONG,
                cy: 0 as LONG,
            };
            let mut lineSize: SIZE_T = 0;

            hGlobal = GetClipboardData(uFormat) as HGLOBAL;
            if hGlobal.is_null() {
                break 'sw;
            }

            lpText = GlobalLock(hGlobal) as PVOID;
            if lpText.is_null() {
                break 'sw;
            }

            hDC = GetDC(Globals.hMainWnd);

            /* Find the size of the rectangle enclosing the text */
            loop {
                if uFormat == CF_UNICODETEXT {
                    if *(lpText as LPCWSTR) as ::core::ffi::c_int
                        == UNICODE_NULL as ::core::ffi::c_int
                    {
                        break;
                    }
                    lineSize = GetLineExtentW(
                        lpText as LPCWSTR,
                        core::ptr::addr_of_mut!(ptr) as *mut LPCWSTR,
                    );
                    dwSize = GetTabbedTextExtentW(
                        hDC,
                        lpText as LPCWSTR,
                        lineSize as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        core::ptr::null(),
                    );
                } else {
                    if *(lpText as LPCSTR) as ::core::ffi::c_int
                        == ANSI_NULL as ::core::ffi::c_int
                    {
                        break;
                    }
                    lineSize = GetLineExtentA(
                        lpText as LPCSTR,
                        core::ptr::addr_of_mut!(ptr) as *mut LPCSTR,
                    );
                    dwSize = GetTabbedTextExtentA(
                        hDC,
                        lpText as LPCSTR,
                        lineSize as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        core::ptr::null(),
                    );
                }
                txtSize.cx = max(txtSize.cx, LOWORD!(dwSize) as LONG);
                txtSize.cy += HIWORD!(dwSize) as LONG;
                lpText = ptr;
            }

            ReleaseDC(Globals.hMainWnd, hDC);

            GlobalUnlock(hGlobal);

            SetRect(
                pRc as LPRECT,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                txtSize.cx as ::core::ffi::c_int,
                txtSize.cy as ::core::ffi::c_int,
            );
        } else {
            break 'sw;
        }
    }

    CloseClipboard();

    return (TRUE) as BOOL;
}
