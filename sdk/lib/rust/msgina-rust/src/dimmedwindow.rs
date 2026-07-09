//! Rust translation of `dll/win32/msgina/dimmedwindow.cpp` — implementation of
//! `ShellDimScreen` (the grayed/faded screenshot backdrop shell32 puts behind the
//! shutdown/logoff dialogs).
//!
//! The C++ was a single ATL COM leaf: `class CDimmedWindow :
//! CComObjectRootEx<CComMultiThreadModelNoCS>, IUnknown`, instantiated via
//! `CComObject<CDimmedWindow>::CreateInstance` and handed to the caller as a bare
//! `IUnknown*`. Nothing else in the tree names the C++ type — the ONLY external
//! entry point is the `extern "C"` ordinal-16 export `ShellDimScreen(void**, HWND*)`
//! (msgina.spec: `16 stdcall -noname ShellDimScreen(ptr ptr)`), reached by shell32
//! via `GetProcAddress(mod, (LPCSTR)16)` (dll/win32/shell32/dialogs/dialogs.cpp,
//! CallShellDimScreen). So no C++-mangled shim is needed: the ATL bases are
//! flattened into a `#[repr(C)]` lpVtbl-first object exposing a hand-rolled
//! IUnknown vtable, exactly like explorer-rust's window classes
//! (sdk/lib/rust/explorer-rust/src/explorer/trayclock.rs et al).
//!
//! ATL semantics carried over:
//!   * `CComObject::CreateInstance` — heap-allocate, construct with refcount 0,
//!     return E_OUTOFMEMORY on allocation failure.
//!   * `CComObjectRootEx<CComMultiThreadModelNoCS>` — Interlocked ref counting.
//!   * `BEGIN_COM_MAP`/`COM_INTERFACE_ENTRY_IID(IID_IUnknown, IUnknown)` — QI
//!     answers only IID_IUnknown.
//!   * `Release()==0` → destructor + heap free.
//! The C++ `CComModule gModule` global existed only to satisfy ATL's CRT hookup in
//! this TU; it has no behavior and is dropped.
//!
//! `_SEH2_TRY` around the out-pointer writes in ShellDimScreen: under
//! USE_DUMMY_PSEH the handler was dead code anyway; per the project convention the
//! Rust validates the pointers up front and returns E_INVALIDARG (the handler's
//! hr) instead of faulting.

use core::ffi::c_void;
use core::sync::atomic::{AtomicI32, Ordering};

use crate::msgina::hDllInstance;
use crate::msgina_externs::*;
use crate::prelude::*;

use crate::w;

// Please note: The INIT_TIMER is a workaround because ReactOS does not redraw the
// desktop in time, so the start menu is still visible on the dimmed screen.
const INIT_TIMER_ID: UINT_PTR = 0x112233;
const FADE_TIMER_ID: UINT_PTR = 0x12345;

// ---- Win32 constants used by this file (winuser.h / wingdi.h). Local items
// shadow any same-named prelude glob re-exports, so these are collision-safe. ----
const WS_EX_TOPMOST: DWORD = 0x00000008;
const WS_POPUP: DWORD = 0x80000000;
const SW_SHOW: c_int = 5;
const PM_REMOVE: UINT = 0x0001;
const WM_NCCREATE: UINT = 0x0081;
const WM_PAINT: UINT = 0x000F;
const WM_TIMER: UINT = 0x0113;
const SRCCOPY: DWORD = 0x00CC0020;
const BI_RGB: DWORD = 0;
const DIB_RGB_COLORS: UINT = 0;
const GWLP_USERDATA: c_int = -21;
const IDC_ARROW: PCWSTR = 32512usize as PCWSTR; // MAKEINTRESOURCE(32512)
const SM_XVIRTUALSCREEN: c_int = 76;
const SM_YVIRTUALSCREEN: c_int = 77;
const SM_CXVIRTUALSCREEN: c_int = 78;
const SM_CYVIRTUALSCREEN: c_int = 79;

const S_OK: HRESULT = 0;
const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;
const E_POINTER: HRESULT = 0x80004003u32 as HRESULT;
const E_FAIL: HRESULT = 0x80004005u32 as HRESULT;
const E_OUTOFMEMORY: HRESULT = 0x8007000Eu32 as HRESULT;
const E_INVALIDARG: HRESULT = 0x80070057u32 as HRESULT;

/// `IID_IUnknown` = {00000000-0000-0000-C000-000000000046}.
const IID_IUnknown: IID = IID {
    Data1: 0x00000000,
    Data2: 0x0000,
    Data3: 0x0000,
    Data4: [0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46],
};

/// `IsEqualIID` (guiddef.h inline).
#[inline]
unsafe fn guid_eq(a: *const IID, b: *const IID) -> bool {
    let a = &*a;
    let b = &*b;
    a.Data1 == b.Data1 && a.Data2 == b.Data2 && a.Data3 == b.Data3 && a.Data4 == b.Data4
}

// ---- user32 / gdi32 out-calls not (yet) in msgina_externs — declared locally,
// the same per-module pattern gui.rs uses for BeginPaint/EndPaint. ----
extern "system" {
    // user32
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16; // ATOM
    fn UnregisterClassW(lpClassName: PCWSTR, hInstance: HINSTANCE) -> BOOL;
    fn DestroyWindow(hWnd: HWND) -> BOOL;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
    fn PeekMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    // gdi32
    fn CreateCompatibleBitmap(hdc: HDC, cx: c_int, cy: c_int) -> HBITMAP;
    fn GetDIBits(
        hdc: HDC,
        hbm: HBITMAP,
        start: UINT,
        cLines: UINT,
        lpvBits: PVOID,
        lpbmi: *mut BITMAPINFO,
        usage: UINT,
    ) -> c_int;
    fn SetDIBits(
        hdc: HDC,
        hbm: HBITMAP,
        start: UINT,
        cLines: UINT,
        lpBits: *const c_void,
        lpbmi: *const BITMAPINFO,
        ColorUse: UINT,
    ) -> c_int;
}

// ---- BITMAPINFO(HEADER) (wingdi.h) — not in the ntdll-sys bindgen set; local. ----
#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAPINFOHEADER {
    pub biSize: DWORD,
    pub biWidth: LONG,
    pub biHeight: LONG,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: DWORD,
    pub biSizeImage: DWORD,
    pub biXPelsPerMeter: LONG,
    pub biYPelsPerMeter: LONG,
    pub biClrUsed: DWORD,
    pub biClrImportant: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGBQUAD {
    pub rgbBlue: u8,
    pub rgbGreen: u8,
    pub rgbRed: u8,
    pub rgbReserved: u8,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [RGBQUAD; 1],
}

// ---- IUnknown vtable (the object's one COM interface) ----

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface:
        unsafe extern "system" fn(iface: *mut c_void, riid: *const IID, ppv: *mut *mut c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(iface: *mut c_void) -> ULONG,
    pub Release: unsafe extern "system" fn(iface: *mut c_void) -> ULONG,
}

// ---- the object (ATL bases flattened; lpVtbl-first COM shape) ----

#[repr(C)]
pub struct CDimmedWindow {
    /// IUnknown vtbl (the class's only interface).
    pub lpVtbl: *const IUnknownVtbl,
    /// CComObjectRootEx m_dwRef (CComMultiThreadModelNoCS → Interlocked ops).
    pub ref_: LONG,
    // --- CDimmedWindow members ---
    pub m_hwnd: HWND,
    pub m_hdc: HDC,
    pub m_hbitmap: HBITMAP,
    pub m_oldbitmap: HGDIOBJ,
    pub m_width: LONG,
    pub m_height: LONG,
    pub m_bi: BITMAPINFO,
    pub m_bytes: *mut UCHAR,
    pub m_step: c_int,
}

// `CComMultiThreadModelNoCS` ref counting = InterlockedIncrement/Decrement.
#[inline]
unsafe fn interlocked_increment(p: *mut LONG) -> LONG {
    (*(p as *const AtomicI32)).fetch_add(1, Ordering::SeqCst) + 1
}
#[inline]
unsafe fn interlocked_decrement(p: *mut LONG) -> LONG {
    (*(p as *const AtomicI32)).fetch_sub(1, Ordering::SeqCst) - 1
}

// ---- construction / destruction (CComObject new/delete of the class) ----

/// `CDimmedWindow::CDimmedWindow()`.
unsafe fn CDimmedWindow_ctor(this: *mut CDimmedWindow) {
    (*this).lpVtbl = &vt_CDimmedWindow;
    (*this).ref_ = 0;
    // Member-init list: m_hwnd(NULL), m_hdc(NULL), m_hbitmap(NULL),
    // m_oldbitmap(NULL), m_width(0), m_height(0), m_bytes(NULL), m_step(0).
    (*this).m_hwnd = core::ptr::null_mut();
    (*this).m_hdc = core::ptr::null_mut();
    (*this).m_hbitmap = core::ptr::null_mut();
    (*this).m_oldbitmap = core::ptr::null_mut();
    (*this).m_width = 0;
    (*this).m_height = 0;
    (*this).m_bi = core::mem::zeroed();
    (*this).m_bytes = core::ptr::null_mut();
    (*this).m_step = 0;

    let mut wndclass: WNDCLASSEXW = core::mem::zeroed();
    wndclass.cbSize = core::mem::size_of::<WNDCLASSEXW>() as UINT;
    // WNDPROC in the bindgen headers is arch-exact (stdcall on i386, C on
    // amd64/arm64); our `extern "system"` fn lowers to the identical ABI on every
    // arch, the transmute only renames the ABI string.
    wndclass.lpfnWndProc = Some(core::mem::transmute(
        CDimmedWindow_WndProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
    ));
    wndclass.hInstance = hDllInstance;
    wndclass.hCursor = LoadCursorW(core::ptr::null_mut(), IDC_ARROW);
    wndclass.lpszClassName = w!("DimmedWindowClass").as_ptr();

    if RegisterClassExW(&wndclass) == 0 {
        return;
    }

    (*this).m_width = GetSystemMetrics(SM_CXVIRTUALSCREEN) as LONG;
    (*this).m_height = GetSystemMetrics(SM_CYVIRTUALSCREEN) as LONG;

    // memset(&m_bi, 0, sizeof(m_bi)) already done above; fill the header.
    // NB: the C sets biSize to sizeof(BITMAPINFO) (header + 1 RGBQUAD), not
    // sizeof(BITMAPINFOHEADER) — carried verbatim.
    (*this).m_bi.bmiHeader.biSize = core::mem::size_of::<BITMAPINFO>() as DWORD;
    (*this).m_bi.bmiHeader.biWidth = (*this).m_width;
    (*this).m_bi.bmiHeader.biHeight = (*this).m_height;
    (*this).m_bi.bmiHeader.biPlanes = 1;
    (*this).m_bi.bmiHeader.biBitCount = 32;
    (*this).m_bi.bmiHeader.biCompression = BI_RGB;
    (*this).m_bi.bmiHeader.biSizeImage = ((*this).m_width * 4 * (*this).m_height) as DWORD;
    // m_bytes = new UCHAR[m_width * 4 * m_height];
    (*this).m_bytes = HeapAlloc(
        GetProcessHeap(),
        0,
        ((*this).m_width as usize) * 4 * ((*this).m_height as usize),
    ) as *mut UCHAR;

    let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
    let y = GetSystemMetrics(SM_YVIRTUALSCREEN);

    (*this).m_hwnd = CreateWindowExW(
        WS_EX_TOPMOST,
        w!("DimmedWindowClass").as_ptr(),
        core::ptr::null(),
        WS_POPUP,
        x,
        y,
        (*this).m_width as c_int,
        (*this).m_height as c_int,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        hDllInstance,
        this as PVOID,
    );
}

/// `CDimmedWindow::~CDimmedWindow()`.
unsafe fn CDimmedWindow_dtor(this: *mut CDimmedWindow) {
    if !(*this).m_hwnd.is_null() {
        DestroyWindow((*this).m_hwnd);
    }
    UnregisterClassW(w!("DimmedWindowClass").as_ptr(), hDllInstance);
    if !(*this).m_oldbitmap.is_null() {
        SelectObject((*this).m_hdc, (*this).m_oldbitmap);
    }
    if !(*this).m_hbitmap.is_null() {
        DeleteObject((*this).m_hbitmap as HGDIOBJ);
    }
    if !(*this).m_hdc.is_null() {
        // The C++ calls DeleteObject (not DeleteDC) on the memory DC — verbatim.
        DeleteObject((*this).m_hdc as HGDIOBJ);
    }
    if !(*this).m_bytes.is_null() {
        // delete[] m_bytes;
        HeapFree(GetProcessHeap(), 0, (*this).m_bytes as PVOID);
    }
}

/// `CComObject<CDimmedWindow>::CreateInstance` — allocate + construct, ref = 0.
unsafe fn CDimmedWindow_CreateInstance(out: *mut *mut CDimmedWindow) -> HRESULT {
    let p = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<CDimmedWindow>())
        as *mut CDimmedWindow;
    if p.is_null() {
        *out = core::ptr::null_mut();
        return E_OUTOFMEMORY;
    }
    CDimmedWindow_ctor(p);
    *out = p;
    S_OK
}

unsafe fn CDimmedWindow_delete(this: *mut CDimmedWindow) {
    if !this.is_null() {
        CDimmedWindow_dtor(this);
        HeapFree(GetProcessHeap(), 0, this as PVOID);
    }
}

// ---- COM: IUnknown (BEGIN_COM_MAP → IID_IUnknown only) ----

unsafe extern "system" fn CDimmedWindow_QueryInterface(
    iface: *mut c_void,
    riid: *const IID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    if ppv.is_null() {
        return E_POINTER;
    }
    if guid_eq(riid, &IID_IUnknown) {
        *ppv = iface;
        CDimmedWindow_AddRef(iface);
        return S_OK;
    }
    *ppv = core::ptr::null_mut();
    E_NOINTERFACE
}

unsafe extern "system" fn CDimmedWindow_AddRef(iface: *mut c_void) -> ULONG {
    let this = iface as *mut CDimmedWindow;
    interlocked_increment(core::ptr::addr_of_mut!((*this).ref_)) as ULONG
}

unsafe extern "system" fn CDimmedWindow_Release(iface: *mut c_void) -> ULONG {
    let this = iface as *mut CDimmedWindow;
    let n = interlocked_decrement(core::ptr::addr_of_mut!((*this).ref_));
    if n == 0 {
        CDimmedWindow_delete(this);
        return 0;
    }
    n as ULONG
}

static vt_CDimmedWindow: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: CDimmedWindow_QueryInterface,
    AddRef: CDimmedWindow_AddRef,
    Release: CDimmedWindow_Release,
};

// ---- methods ----

/// `CDimmedWindow::WaitForInit` — pump until the window shows.
/// (This is needed so that we do not capture the start menu while it's closing.)
unsafe fn CDimmedWindow_WaitForInit(this: *mut CDimmedWindow) {
    let mut msg: MSG = core::mem::zeroed();

    while IsWindow((*this).m_hwnd) != 0 && IsWindowVisible((*this).m_hwnd) == 0 {
        while PeekMessageW(&mut msg, (*this).m_hwnd, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);

            if IsWindowVisible((*this).m_hwnd) != 0 {
                break;
            }
        }
    }
}

/// `CDimmedWindow::Init` — capture the desktop, show, start the fade timer.
unsafe fn CDimmedWindow_Init(this: *mut CDimmedWindow) {
    CDimmedWindow_Capture(this);

    ShowWindow((*this).m_hwnd, SW_SHOW);
    SetForegroundWindow((*this).m_hwnd);
    EnableWindow((*this).m_hwnd, 0);

    SetTimer((*this).m_hwnd, FADE_TIMER_ID, 200, core::ptr::null_mut());
}

/// `CDimmedWindow::Capture` — snapshot the desktop into the memory DC.
unsafe fn CDimmedWindow_Capture(this: *mut CDimmedWindow) {
    let desktopWnd = GetDesktopWindow();
    let desktopDC = GetDC(desktopWnd);

    (*this).m_hdc = CreateCompatibleDC(desktopDC);

    (*this).m_hbitmap =
        CreateCompatibleBitmap(desktopDC, (*this).m_width as c_int, (*this).m_height as c_int);
    (*this).m_oldbitmap = SelectObject((*this).m_hdc, (*this).m_hbitmap as HGDIOBJ);
    BitBlt(
        (*this).m_hdc,
        0,
        0,
        (*this).m_width as c_int,
        (*this).m_height as c_int,
        desktopDC,
        0,
        0,
        SRCCOPY,
    );

    ReleaseDC(desktopWnd, desktopDC);
}

/// `CDimmedWindow::Step` — one fade increment; false when done.
unsafe fn CDimmedWindow_Step(this: *mut CDimmedWindow) -> bool {
    // Stop after 10 steps
    let step = (*this).m_step;
    (*this).m_step += 1;
    if step > 10 || (*this).m_bytes.is_null() {
        return false;
    }

    let lines = GetDIBits(
        (*this).m_hdc,
        (*this).m_hbitmap,
        0,
        (*this).m_height as UINT,
        (*this).m_bytes as PVOID,
        core::ptr::addr_of_mut!((*this).m_bi),
        DIB_RGB_COLORS,
    );
    if lines != 0 {
        let bytes = (*this).m_bytes;
        let width = (*this).m_width as isize;
        let height = (*this).m_height as isize;
        for xh in 0..height {
            let h = width * 4 * xh;
            for w in 0..width {
                let b = *bytes.offset(h + w * 4) as c_int;
                let g = *bytes.offset(h + w * 4 + 1) as c_int;
                let r = *bytes.offset(h + w * 4 + 2) as c_int;

                // Standard formula to convert a color.
                let mut gray = (r * 30 + g * 59 + b * 11) / 100;
                if gray < 0 {
                    gray = 0;
                }

                // Do not fade too fast.
                let r = (r * 2 + gray) / 3;
                let g = (g * 2 + gray) / 3;
                let b = (b * 2 + gray) / 3;

                *bytes.offset(h + w * 4) = b as UCHAR;
                *bytes.offset(h + w * 4 + 1) = g as UCHAR;
                *bytes.offset(h + w * 4 + 2) = r as UCHAR;
            }
        }
        SetDIBits(
            (*this).m_hdc,
            (*this).m_hbitmap,
            0,
            lines as UINT,
            (*this).m_bytes as *const c_void,
            core::ptr::addr_of!((*this).m_bi),
            DIB_RGB_COLORS,
        );
    }
    true
}

/// `CDimmedWindow::Blt`.
unsafe fn CDimmedWindow_Blt(this: *mut CDimmedWindow, hdc: HDC) {
    BitBlt(
        hdc,
        0,
        0,
        (*this).m_width as c_int,
        (*this).m_height as c_int,
        (*this).m_hdc,
        0,
        0,
        SRCCOPY,
    );
}

// ---- window procedure ----

unsafe extern "system" fn CDimmedWindow_WndProc(
    hWnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    match uMsg {
        WM_NCCREATE => {
            let lpcs = lParam as *const CREATESTRUCTW;
            let info = (*lpcs).lpCreateParams as *mut CDimmedWindow;
            SetWindowLongPtrW(hWnd, GWLP_USERDATA, info as isize);
            SetTimer(hWnd, INIT_TIMER_ID, 50, core::ptr::null_mut());
            // break → falls through to DefWindowProc
        }

        WM_PAINT => {
            let info = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut CDimmedWindow;
            if !info.is_null() {
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                BeginPaint(hWnd, &mut ps);
                CDimmedWindow_Blt(info, ps.hdc);
                EndPaint(hWnd, &ps);
            }
            return 0;
        }

        WM_TIMER => {
            if wParam as UINT_PTR == INIT_TIMER_ID {
                let info = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut CDimmedWindow;
                KillTimer(hWnd, INIT_TIMER_ID);
                // The C++ calls info->Init() unchecked; GWLP_USERDATA is always
                // set by WM_NCCREATE before this timer can fire.
                CDimmedWindow_Init(info);
            } else if wParam as UINT_PTR == FADE_TIMER_ID {
                let info = GetWindowLongPtrW(hWnd, GWLP_USERDATA) as *mut CDimmedWindow;
                if !info.is_null() && CDimmedWindow_Step(info) {
                    InvalidateRect(hWnd, core::ptr::null(), 1);
                } else {
                    KillTimer(hWnd, FADE_TIMER_ID);
                }
            }
            return 0;
        }

        _ => {}
    }

    DefWindowProcW(hWnd, uMsg, wParam, lParam)
}

// ---- the export ----

/// `ShellDimScreen` — msgina.spec ordinal 16 (`16 stdcall -noname
/// ShellDimScreen(ptr ptr)`); shell32 resolves it by ordinal via GetProcAddress
/// (dialogs.cpp CallShellDimScreen). Creates the dimmed backdrop window and
/// returns it as an IUnknown* (caller Releases it to tear the window down) plus
/// its HWND (used as the dialog parent).
#[no_mangle]
pub unsafe extern "system" fn ShellDimScreen(
    pUnknown: *mut *mut c_void,
    hWindow: *mut HWND,
) -> HRESULT {
    let mut pWindow: *mut CDimmedWindow = core::ptr::null_mut();
    let hr = CDimmedWindow_CreateInstance(&mut pWindow);
    if pWindow.is_null() {
        return hr;
    }

    CDimmedWindow_WaitForInit(pWindow);

    if IsWindow((*pWindow).m_hwnd) == 0 {
        let mut refcount = CDimmedWindow_AddRef(pWindow as *mut c_void);
        while refcount != 0 {
            refcount = CDimmedWindow_Release(pWindow as *mut c_void);
        }

        return E_FAIL;
    }

    // _SEH2_TRY around the out-pointer writes: under USE_DUMMY_PSEH the C
    // handler was dead code; validate the pointers instead (the handler's hr
    // was E_INVALIDARG, with the same AddRef/Release-to-zero teardown).
    if pUnknown.is_null() || hWindow.is_null() {
        let mut refcount = CDimmedWindow_AddRef(pWindow as *mut c_void);
        while refcount != 0 {
            refcount = CDimmedWindow_Release(pWindow as *mut c_void);
        }
        return E_INVALIDARG;
    }

    let _hr = CDimmedWindow_QueryInterface(pWindow as *mut c_void, &IID_IUnknown, pUnknown);
    *hWindow = (*pWindow).m_hwnd;
    S_OK
}
