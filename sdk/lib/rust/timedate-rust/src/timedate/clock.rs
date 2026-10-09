use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut szClockWndClass: [WCHAR; 14] = unsafe {
    ::core::mem::transmute::<[u8; 28], [WCHAR; 14]>(*b"C\0l\0o\0c\0k\0W\0n\0d\0C\0l\0a\0s\0s\0\0\0")
};

#[repr(C)]
pub struct CLOCKDATA {
    pub hGreyBrush: HBRUSH,
    pub hGreyPen: HPEN,
    pub cxClient: i32,
    pub cyClient: i32,
    pub stCurrent: SYSTEMTIME,
    pub stPrevious: SYSTEMTIME,
    pub bTimer: i32,
}

pub type PCLOCKDATA = *mut CLOCKDATA;

pub const TWOPI: f64 = 2.0 * 3.14159;

unsafe extern "C" fn RotatePoint(pt: *mut POINT, iNum: INT, iAngle: INT) {
    let mut i: INT = 0 as INT;
    let mut ptTemp: POINT = POINT { x: 0, y: 0 };
    while i < iNum {
        ptTemp.x = ((*pt.offset(i as isize)).x as f64
            * cos(TWOPI * iAngle as f64 / 360 as f64)
            + (*pt.offset(i as isize)).y as f64
                * sin(TWOPI * iAngle as f64 / 360 as f64))
            as INT as LONG;
        ptTemp.y = ((*pt.offset(i as isize)).y as f64
            * cos(TWOPI * iAngle as f64 / 360 as f64)
            - (*pt.offset(i as isize)).x as f64
                * sin(TWOPI * iAngle as f64 / 360 as f64))
            as INT as LONG;
        *pt.offset(i as isize) = ptTemp;
        i += 1;
    }
}

unsafe extern "C" fn DrawClock(hdc: HDC, pClockData: PCLOCKDATA) -> INT {
    let mut iAngle: INT;
    let mut Radius: INT;
    let mut pt: [POINT; 3] = core::mem::zeroed();
    let mut hBrushOld: HBRUSH;
    let mut hPenOld: HPEN = core::ptr::null_mut();

    /* Grey brush to fill the dots */
    hBrushOld = SelectObject(hdc, (*pClockData).hGreyBrush as HGDIOBJ) as HBRUSH;

    hPenOld = GetCurrentObject(hdc, OBJ_PEN as UINT) as HPEN;

    // TODO: Check if this conversion is correct resp. usable
    Radius = min((*pClockData).cxClient, (*pClockData).cyClient) * 2;

    iAngle = 0;
    while iAngle < 360 {
        /* Starting coords */
        pt[0 as usize].x = 0;
        pt[0 as usize].y = Radius;

        /* Rotate start coords */
        RotatePoint(pt.as_mut_ptr(), 1, iAngle);

        /* Determine whether it's a big dot or a little dot
         * i.e. 1-4 or 5, 6-9 or 10, 11-14 or 15 */
        if iAngle % 5 != 0 {
            pt[2 as usize].y = 7;
            pt[2 as usize].x = pt[2 as usize].y;
            SelectObject(hdc, (*pClockData).hGreyPen as HGDIOBJ);
        } else {
            pt[2 as usize].y = 16;
            pt[2 as usize].x = pt[2 as usize].y;
            SelectObject(hdc, GetStockObject((BLACK_PEN as UINT) as i32) as HGDIOBJ);
        }

        pt[0 as usize].x -= pt[2 as usize].x / 2;
        pt[0 as usize].y -= pt[2 as usize].y / 2;

        pt[1 as usize].x = pt[0 as usize].x + pt[2 as usize].x;
        pt[1 as usize].y = pt[0 as usize].y + pt[2 as usize].y;

        Ellipse(
            hdc,
            pt[0 as usize].x as INT,
            pt[0 as usize].y as INT,
            pt[1 as usize].x as INT,
            pt[1 as usize].y as INT,
        );

        iAngle += 6;
    }

    SelectObject(hdc, hBrushOld as HGDIOBJ);
    SelectObject(hdc, hPenOld as HGDIOBJ);
    return Radius;
}

unsafe extern "C" fn DrawHands(hdc: HDC, pst: *mut SYSTEMTIME, fChange: BOOL, Radius: INT) {
    let mut pt: [[POINT; 5]; 3] = [
        [
            POINT { x: 0 as LONG, y: (-Radius / 6) as LONG },
            POINT { x: (Radius / 9) as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: ((Radius as f64) / 1.8) as LONG },
            POINT { x: (-Radius / 9) as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: (-Radius / 6) as LONG },
        ],
        [
            POINT { x: 0 as LONG, y: ((-Radius) as f64 / 4.5) as LONG },
            POINT { x: (Radius / 18) as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: ((Radius as f64) * 0.89) as LONG },
            POINT { x: (-Radius / 18) as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: ((-Radius) as f64 / 4.5) as LONG },
        ],
        [
            POINT { x: 0 as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: 0 as LONG },
            POINT { x: 0 as LONG, y: ((Radius as f64) * 0.89) as LONG },
        ],
    ];
    let mut i: INT = 0;
    let mut iAngle: [INT; 3] = [0; 3];
    let mut ptTemp: [[POINT; 5]; 3] =
        [[POINT { x: 0, y: 0 }; 5]; 3];

    /* Black pen for outline, white brush for fill */
    SelectObject(hdc, GetStockObject((BLACK_PEN as i32)));
    SelectObject(hdc, GetStockObject((WHITE_BRUSH as i32)));

    iAngle[0 as usize] =
        (((*pst).wHour as INT * 30) % 360
            + (*pst).wMinute as INT / 2) as INT;
    iAngle[1 as usize] = ((*pst).wMinute as INT * 6) as INT;
    iAngle[2 as usize] = ((*pst).wSecond as INT * 6) as INT;

    CopyMemory!(
        core::ptr::addr_of_mut!(ptTemp) as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(pt) as *const core::ffi::c_void,
        core::mem::size_of::<[[POINT; 5]; 3]>(),
    );

    i = (if fChange != 0 { 0 } else { 2 }) as INT;
    while i < 3 {
        RotatePoint(
            (core::ptr::addr_of_mut!(ptTemp) as *mut [POINT; 5]).offset(i as isize) as *mut POINT,
            5,
            iAngle[i as usize],
        );
        Polygon(
            hdc,
            (core::ptr::addr_of_mut!(ptTemp) as *mut [POINT; 5]).offset(i as isize) as *mut POINT,
            5,
        );
        i += 1;
    }
}

unsafe extern "system" fn ClockWndProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut pClockData: PCLOCKDATA = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut hdcMem: HDC = core::ptr::null_mut();
    let mut ps: PAINTSTRUCT = core::mem::zeroed();

    pClockData = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as PCLOCKDATA;

    match uMsg {
        WM_CREATE => {
            pClockData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<CLOCKDATA>() as SIZE_T,
            ) as PCLOCKDATA;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, pClockData as LONG_PTR);

            (*pClockData).hGreyPen = CreatePen(
                (PS_SOLID as i32),
                1,
                RGB!(128, 128, 128),
            );
            (*pClockData).hGreyBrush = CreateSolidBrush(
                RGB!(128, 128, 128),
            );

            GetLocalTime(core::ptr::addr_of_mut!((*pClockData).stCurrent));
            (*pClockData).stPrevious = (*pClockData).stCurrent;

            (*pClockData).bTimer = (SetTimer(
                hwnd,
                ID_TIMER as UINT_PTR,
                (1000 - (*pClockData).stCurrent.wMilliseconds as INT) as UINT,
                None,
            ) != 0 as UINT_PTR) as BOOL;
        }

        WM_SIZE => {
            (*pClockData).cxClient = LOWORD!(lParam as DWORD) as INT;
            (*pClockData).cyClient = HIWORD!(lParam as DWORD) as INT;
        }

        WM_TIMECHANGE | WM_TIMER => {
            GetLocalTime(core::ptr::addr_of_mut!((*pClockData).stCurrent));
            InvalidateRect(
                hwnd,
                core::ptr::null(),
                FALSE as BOOL,
            );
            (*pClockData).stPrevious = (*pClockData).stCurrent;

            // Reset timeout.
            if (*pClockData).bTimer != 0 {
                SetTimer(
                    hwnd,
                    ID_TIMER as UINT_PTR,
                    (1000 - (*pClockData).stCurrent.wMilliseconds as INT) as UINT,
                    None,
                );
            }
        }

        WM_PAINT => {
            hdc = BeginPaint(hwnd, core::ptr::addr_of_mut!(ps));

            hdcMem = CreateCompatibleDC(hdc);
            if !hdcMem.is_null() {
                let mut hBmp: HBITMAP = core::ptr::null_mut();
                let mut hBmpOld: HBITMAP = core::ptr::null_mut();

                hBmp = CreateCompatibleBitmap(
                    hdc,
                    (*pClockData).cxClient,
                    (*pClockData).cyClient,
                );
                if !hBmp.is_null() {
                    let mut rcParent: RECT = core::mem::zeroed();
                    let mut hParentWnd: HWND = GetParent(hwnd);
                    let mut oldMap: INT = 0;
                    let mut Radius: INT = 0;
                    let mut oldOrg: POINT = core::mem::zeroed();

                    hBmpOld = SelectObject(hdcMem, hBmp as HGDIOBJ) as HBITMAP;

                    SetRect(
                        core::ptr::addr_of_mut!(rcParent),
                        0,
                        0,
                        (*pClockData).cxClient,
                        (*pClockData).cyClient,
                    );
                    MapWindowPoints(
                        hwnd,
                        hParentWnd,
                        core::ptr::addr_of_mut!(rcParent) as LPPOINT,
                        2,
                    );
                    OffsetViewportOrgEx(
                        hdcMem,
                        -rcParent.left,
                        -rcParent.top,
                        core::ptr::addr_of_mut!(oldOrg),
                    );
                    SendMessage(
                        hParentWnd,
                        WM_PRINT,
                        hdcMem as WPARAM,
                        (PRF_ERASEBKGND | PRF_CLIENT) as LPARAM,
                    );
                    SetViewportOrgEx(
                        hdcMem,
                        oldOrg.x,
                        oldOrg.y,
                        core::ptr::null_mut(),
                    );

                    oldMap = SetMapMode(hdcMem, (MM_ISOTROPIC as i32));
                    SetWindowExtEx(
                        hdcMem,
                        3600,
                        2700,
                        core::ptr::null_mut(),
                    );
                    SetViewportExtEx(
                        hdcMem,
                        800,
                        -(600),
                        core::ptr::null_mut(),
                    );
                    SetViewportOrgEx(
                        hdcMem,
                        (*pClockData).cxClient / 2,
                        (*pClockData).cyClient / 2,
                        core::ptr::addr_of_mut!(oldOrg),
                    );

                    Radius = DrawClock(hdcMem, pClockData);
                    DrawHands(
                        hdcMem,
                        core::ptr::addr_of_mut!((*pClockData).stPrevious),
                        (TRUE) as BOOL,
                        Radius,
                    );

                    SetMapMode(hdcMem, oldMap);
                    SetViewportOrgEx(
                        hdcMem,
                        oldOrg.x,
                        oldOrg.y,
                        core::ptr::null_mut(),
                    );

                    BitBlt(
                        hdc,
                        0,
                        0,
                        (*pClockData).cxClient,
                        (*pClockData).cyClient,
                        hdcMem,
                        0,
                        0,
                        SRCCOPY as DWORD,
                    );

                    SelectObject(hdcMem, hBmpOld as HGDIOBJ);
                    DeleteObject(hBmp as HGDIOBJ);
                }

                DeleteDC(hdcMem);
            }

            EndPaint(hwnd, core::ptr::addr_of_mut!(ps));
        }

        /* No need to erase background, handled during paint */
        WM_ERASEBKGND => {
            return 1 as LRESULT;
        }

        WM_DESTROY => {
            DeleteObject((*pClockData).hGreyPen as HGDIOBJ);
            DeleteObject((*pClockData).hGreyBrush as HGDIOBJ);

            if (*pClockData).bTimer != 0 {
                KillTimer(hwnd, ID_TIMER as UINT_PTR);
            }

            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                pClockData as PVOID,
            );
        }

        CLM_STOPCLOCK => {
            if (*pClockData).bTimer != 0 {
                KillTimer(hwnd, ID_TIMER as UINT_PTR);
                (*pClockData).bTimer = FALSE as BOOL;
            }
        }

        CLM_STARTCLOCK => {
            if (*pClockData).bTimer == 0 {
                let mut LocalTime: SYSTEMTIME = core::mem::zeroed();

                GetLocalTime(core::ptr::addr_of_mut!(LocalTime));
                (*pClockData).bTimer = (SetTimer(
                    hwnd,
                    ID_TIMER as UINT_PTR,
                    (1000 - LocalTime.wMilliseconds as INT) as UINT,
                    None,
                ) != 0 as UINT_PTR) as BOOL;
            }
        }

        _ => {
            DefWindowProcW(hwnd, uMsg, wParam, lParam);
        }
    }

    return TRUE as LRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn RegisterClockControl() -> BOOL {
    let mut wc: WNDCLASSEXW = core::mem::zeroed();
    wc.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wc.lpfnWndProc = Some(ClockWndProc);
    wc.hInstance = hApplet;
    wc.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as usize as LPCWSTR,
    );
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszClassName = core::ptr::addr_of!(szClockWndClass) as *const WCHAR as LPCWSTR;

    return (RegisterClassExW(core::ptr::addr_of_mut!(wc))
        != 0 as ATOM) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UnregisterClockControl() {
    UnregisterClassW(core::ptr::addr_of!(szClockWndClass) as *const u16, hApplet);
}
