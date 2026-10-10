use crate::imm32::imm::ImmLockIMC;
use crate::imm32::ime::ImmLockImeDpi;
use crate::imm32::imm::ImmUnlockIMC;
use crate::imm32::ime::ImmUnlockImeDpi;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_491507152B61: [u16; 13] = [
    0x004Du16, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044,
    0x006C, 0x0067, 0x0000,
];
pub static RUSTCONV_WSTR_F28AD8C4C16D: [u16; 8] = [0x0044u16, 0x0049, 0x0053, 0x0050, 0x004C, 0x0041, 0x0059, 0x0000];

use core::ffi::c_void;

extern "C" {
    fn ros_dbg_log( cls: __wine_debug_class, ch: *mut __wine_debug_channel, file: *const i8, func: *const i8, line: i32, fmt: *const i8, ... );
}
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut guScanCode: [UINT; 256] = [0; 256];

static mut gptRaiseEdge: POINT = tagPOINT { x: 0, y: 0 };

static mut g_bWantSoftKBDMetrics: BOOL = (TRUE) as i32;

pub const T1_CLASSNAMEW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"S\0o\0f\0t\0K\0B\0D\0C\0l\0s\0T\x001\0\0\0",
    )
};

#[no_mangle]
pub static mut gT1K2VK: [BYTE; 60] = [
    0xc0 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x11 as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0x12 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0 as ::core::ffi::c_int as BYTE,
];

static mut g_T1LogFont: LOGFONTW = tagLOGFONTW {
    lfHeight: 0,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: 0,
    lfItalic: 0,
    lfUnderline: 0,
    lfStrikeOut: 0,
    lfCharSet: 0,
    lfOutPrecision: 0,
    lfClipPrecision: 0,
    lfQuality: 0,
    lfPitchAndFamily: 0,
    lfFaceName: [0; 32],
};

pub const C1_CLASSNAMEW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"S\0o\0f\0t\0K\0B\0D\0C\0l\0s\0C\x001\0\0\0",
    )
};

#[no_mangle]
pub static mut gC1K2VK: [BYTE; 56] = [
    0xc0 as ::core::ffi::c_int as BYTE,
    0x31 as ::core::ffi::c_int as BYTE,
    0x32 as ::core::ffi::c_int as BYTE,
    0x33 as ::core::ffi::c_int as BYTE,
    0x34 as ::core::ffi::c_int as BYTE,
    0x35 as ::core::ffi::c_int as BYTE,
    0x36 as ::core::ffi::c_int as BYTE,
    0x37 as ::core::ffi::c_int as BYTE,
    0x38 as ::core::ffi::c_int as BYTE,
    0x39 as ::core::ffi::c_int as BYTE,
    0x30 as ::core::ffi::c_int as BYTE,
    0xbd as ::core::ffi::c_int as BYTE,
    0xbb as ::core::ffi::c_int as BYTE,
    0x51 as ::core::ffi::c_int as BYTE,
    0x57 as ::core::ffi::c_int as BYTE,
    0x45 as ::core::ffi::c_int as BYTE,
    0x52 as ::core::ffi::c_int as BYTE,
    0x54 as ::core::ffi::c_int as BYTE,
    0x59 as ::core::ffi::c_int as BYTE,
    0x55 as ::core::ffi::c_int as BYTE,
    0x49 as ::core::ffi::c_int as BYTE,
    0x4f as ::core::ffi::c_int as BYTE,
    0x50 as ::core::ffi::c_int as BYTE,
    0xdb as ::core::ffi::c_int as BYTE,
    0xdd as ::core::ffi::c_int as BYTE,
    0xdc as ::core::ffi::c_int as BYTE,
    0x41 as ::core::ffi::c_int as BYTE,
    0x53 as ::core::ffi::c_int as BYTE,
    0x44 as ::core::ffi::c_int as BYTE,
    0x46 as ::core::ffi::c_int as BYTE,
    0x47 as ::core::ffi::c_int as BYTE,
    0x48 as ::core::ffi::c_int as BYTE,
    0x4a as ::core::ffi::c_int as BYTE,
    0x4b as ::core::ffi::c_int as BYTE,
    0x4c as ::core::ffi::c_int as BYTE,
    0xba as ::core::ffi::c_int as BYTE,
    0xde as ::core::ffi::c_int as BYTE,
    0x5a as ::core::ffi::c_int as BYTE,
    0x58 as ::core::ffi::c_int as BYTE,
    0x43 as ::core::ffi::c_int as BYTE,
    0x56 as ::core::ffi::c_int as BYTE,
    0x42 as ::core::ffi::c_int as BYTE,
    0x4e as ::core::ffi::c_int as BYTE,
    0x4d as ::core::ffi::c_int as BYTE,
    0xbc as ::core::ffi::c_int as BYTE,
    0xbe as ::core::ffi::c_int as BYTE,
    0xbf as ::core::ffi::c_int as BYTE,
    0x8 as ::core::ffi::c_int as BYTE,
    0x9 as ::core::ffi::c_int as BYTE,
    0x14 as ::core::ffi::c_int as BYTE,
    0xd as ::core::ffi::c_int as BYTE,
    0x10 as ::core::ffi::c_int as BYTE,
    0x2d as ::core::ffi::c_int as BYTE,
    0x2e as ::core::ffi::c_int as BYTE,
    0x20 as ::core::ffi::c_int as BYTE,
    0x1b as ::core::ffi::c_int as BYTE,
];

static mut gbC1ButtonInit: BOOL = (FALSE) as i32;

static mut gptC1ButtonPos: [POINT; 56] = [tagPOINT { x: 0, y: 0 }; 56];

// NOTE: t1keys.h and c1keys.h were not supplied. The T1KEY and C1KEY enums and the
// gT1K2VK[60] / gC1K2VK[56] tables are NOT emitted. They must be generated from those
// headers: one `DEFINE_T1K(...)` / `DEFINE_C1K(...)` line per key, using the
// code_name = code enum form and the virtual_key_code table form respectively.

#[repr(C)]
pub struct SOFTKBDDATAEX {
    pub uCount: UINT,
    pub wCode: [[WORD; 256]; 2],
}
pub type PSOFTKBDDATAEX = *mut SOFTKBDDATAEX;

// L"SoftKBDClsT1" as a NUL-terminated UTF-16 array

#[repr(C)]
pub struct T1WINDOW {
    pub cxDefWidth: INT,
    pub cxWidth47: INT,
    pub cxWidth48: INT,
    pub cxWidth49: INT,
    pub cxWidth50: INT,
    pub cxWidth51or52: INT,
    pub cxWidth53or54: INT,
    pub cxWidth55or56: INT,
    pub cxWidth57: INT,
    pub cxWidth58: INT,
    pub cyDefHeight: INT,
    pub cyHeight50: INT,
    pub KeyPos: [POINT; 60],
    pub chKeyChar: [WCHAR; 48],
    pub hbmKeyboard: HBITMAP,
    pub CharSet: DWORD,
    pub PressedKey: UINT,
    pub pt0: POINT,
    pub pt1: POINT,
    pub KeyboardSubType: LPARAM,
}
pub type PT1WINDOW = *mut T1WINDOW;

// L"SoftKBDClsC1" as a NUL-terminated UTF-16 array

#[repr(C)]
pub struct C1WINDOW {
    pub Data: [[WCHAR; 47]; 2],
    pub dwFlags: DWORD,
    pub hbmKeyboard: HBITMAP,
    pub SubType: LPARAM,
    pub iPressedKey: INT,
    pub pt1: POINT,
    pub pt2: POINT,
    pub CharSet: DWORD,
}
pub type PC1WINDOW = *mut C1WINDOW;

pub const FLAG_SHIFT_PRESSED: i32 = 1;
pub const FLAG_DRAGGING: i32 = 2;
pub const FLAG_PRESSED: i32 = 4; // nolint: const-contradiction (softkbd.c-local #define FLAG_PRESSED 4)

static mut T1_SetData_DUMMY_: () = ();

unsafe extern "C" fn Imm32PtInRect(
    ppt: *const POINT,
    x: LONG,
    y: LONG,
    cx: LONG,
    cy: LONG,
) -> BOOL {
    return ((x <= (*ppt).x) && ((*ppt).x < x + cx) && (y <= (*ppt).y) && ((*ppt).y < y + cy)) as BOOL;
}

unsafe extern "C" fn Imm32DrawBitmap(
    hdc: HDC,
    x: INT,
    y: INT,
    width: INT,
    height: INT,
    nBitmapID: INT,
) {
    let hBitmap: HBITMAP = LoadBitmapW(ghImm32Inst, MAKEINTRESOURCEW!(nBitmapID));
    let hMemDC: HDC = CreateCompatibleDC(hdc);
    let hbmOld: HGDIOBJ = SelectObject(hMemDC, hBitmap as HGDIOBJ);
    BitBlt(hdc, x, y, width, height, hMemDC, 0, 0, SRCCOPY);
    DeleteObject(SelectObject(hMemDC, hbmOld));
    DeleteDC(hMemDC);
}

unsafe extern "C" fn Imm32Clamp(x: INT, xMin: INT, xMax: INT) -> INT {
    if x < xMin {
        return xMin;
    }
    if x > xMax {
        return xMax;
    }
    return x;
}

unsafe extern "C" fn Imm32GetAllMonitorSize(prcWork: LPRECT) {
    if GetSystemMetrics((SM_CMONITORS) as i32) == 1 {
        SystemParametersInfoW(
            SPI_GETWORKAREA as UINT,
            0 as UINT,
            prcWork as PVOID,
            0 as UINT,
        );
        return;
    }

    (*prcWork).left = GetSystemMetrics((SM_XVIRTUALSCREEN) as i32) as LONG;
    (*prcWork).top = GetSystemMetrics((SM_YVIRTUALSCREEN) as i32) as LONG;
    (*prcWork).right = (*prcWork).left + GetSystemMetrics((SM_CXVIRTUALSCREEN) as i32) as LONG;
    (*prcWork).bottom = (*prcWork).top + GetSystemMetrics((SM_CYVIRTUALSCREEN) as i32) as LONG;
}

unsafe extern "C" fn Imm32GetNearestWorkArea(hwnd: HWND, prcWork: LPRECT) -> BOOL {
    let mut hMonitor: HMONITOR = core::ptr::null_mut();
    let mut mi: MONITORINFO = core::mem::zeroed();

    if GetSystemMetrics(SM_CMONITORS as core::ffi::c_int) == 1 {
        Imm32GetAllMonitorSize(prcWork);
        return (TRUE) as BOOL;
    }

    hMonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST as DWORD);
    if hMonitor.is_null() {

        return (FALSE) as BOOL;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(mi) as *mut core::ffi::c_void,
        core::mem::size_of::<MONITORINFO>(),
    );
    mi.cbSize = core::mem::size_of::<MONITORINFO>() as DWORD;
    GetMonitorInfoW(hMonitor, core::ptr::addr_of_mut!(mi));
    *prcWork = mi.rcWork;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn T1_GetTextMetric(ptm: LPTEXTMETRICW) {
    let mut wch: WCHAR = 0;
    let mut textSize: SIZE = core::mem::zeroed();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut hFontOld: HGDIOBJ = core::ptr::null_mut();
    let mut hDC: HDC = core::ptr::null_mut();
    let mut szFace: [WCHAR; (LF_FACESIZE as usize)] = [0; (LF_FACESIZE as usize)];

    ZeroMemory!(
        core::ptr::addr_of_mut!(g_T1LogFont) as *mut core::ffi::c_void,
        core::mem::size_of::<LOGFONTW>(),
    );
    g_T1LogFont.lfHeight = -12;
    g_T1LogFont.lfWeight = (FW_NORMAL as i32);
    g_T1LogFont.lfCharSet = (CHINESEBIG5_CHARSET as u8);
    if false {
        // NO_HACK branch (preprocessor-disabled in the C build)
        g_T1LogFont.lfOutPrecision = (OUT_TT_ONLY_PRECIS as u8);
        g_T1LogFont.lfClipPrecision = (CLIP_DEFAULT_PRECIS as u8);
        g_T1LogFont.lfQuality = (PROOF_QUALITY as u8);
        g_T1LogFont.lfPitchAndFamily = (FF_MODERN | FIXED_PITCH) as u8;
    }
    {
        StringCchCopyW(
            core::ptr::addr_of_mut!(g_T1LogFont.lfFaceName) as *mut WCHAR,
            (_countof!(g_T1LogFont.lfFaceName)) as _,
            RUSTCONV_WSTR_491507152B61.as_ptr(),
        );
    }
    hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(g_T1LogFont));

    hDC = GetDC(core::ptr::null_mut());
    hFontOld = SelectObject(hDC, hFont as HGDIOBJ);

    GetTextFaceW(
        hDC,
        _countof!(szFace) as core::ffi::c_int,
        szFace.as_mut_ptr(),
    );

    GetTextMetricsW(hDC, ptm);

    wch = 0x4E11 as WCHAR; /* U+4E11: 丑 */
    if GetTextExtentPoint32W(
        hDC,
        core::ptr::addr_of_mut!(wch) as *const WCHAR,
        1,
        core::ptr::addr_of_mut!(textSize),
    ) != 0
        && textSize.cx > (*ptm).tmMaxCharWidth
    {
        (*ptm).tmMaxCharWidth = textSize.cx;
    }

    DeleteObject(SelectObject(hDC, hFontOld));
    ReleaseDC(core::ptr::null_mut(), hDC);
}

unsafe extern "C" fn T1_InitButtonPos(mut pT1: PT1WINDOW) {
    let mut tm: TEXTMETRICW = core::mem::zeroed();
    let mut cxLarge: LONG = 0;
    let mut cyLarge: LONG = 0;
    let mut xKey1: LONG = 0;
    let mut yKey1: LONG = 0;
    let mut xKey2: LONG = 0;
    let mut yKey2: LONG = 0;
    let mut xKey3: LONG = 0;
    let mut yKey3: LONG = 0;
    let mut yKey4: LONG = 0;
    let mut xKey4: LONG = 0;
    let mut xKey5: LONG = 0;
    let mut yKey5: LONG = 0;
    let mut xKey6: LONG = 0;
    let mut xKey7: LONG = 0;
    let mut iKey: INT = 0;

    T1_GetTextMetric(core::ptr::addr_of_mut!(tm));

    cxLarge = (3 as LONG * tm.tmMaxCharWidth + 18 as LONG) / 2 as LONG;
    cyLarge = tm.tmHeight + 8 as LONG;

    /* key widths and heights */
    (*pT1).cxDefWidth = ((2 as LONG * tm.tmMaxCharWidth + 12 as LONG) / 2 as LONG) as INT;
    (*pT1).cxWidth47 = ((2 as LONG * tm.tmMaxCharWidth + 12 as LONG) / 2 as LONG + 1 as LONG) as INT;
    (*pT1).cxWidth49 = ((4 as LONG * tm.tmMaxCharWidth + 24 as LONG) / 2 as LONG + 3 as LONG) as INT;
    (*pT1).cxWidth51or52 = ((5 as LONG * tm.tmMaxCharWidth + 30 as LONG) / 2 as LONG + 5 as LONG) as INT;
    (*pT1).cxWidth58 = (4 as LONG * (3 as LONG * tm.tmMaxCharWidth + 18 as LONG) / 2 as LONG + 15 as LONG) as INT;
    (*pT1).cxWidth50 = (cxLarge + 2 as LONG) as INT;
    (*pT1).cxWidth48 = (*pT1).cxWidth50;
    (*pT1).cxWidth55or56 = (cxLarge + 2 as LONG) as INT;
    (*pT1).cxWidth53or54 = (*pT1).cxWidth55or56;
    (*pT1).cyHeight50 = (2 as LONG * (tm.tmHeight + 8 as LONG) + 3 as LONG) as INT;
    (*pT1).cxWidth57 = (cxLarge + 1 as LONG) as INT;
    (*pT1).cyDefHeight = cyLarge as INT;

    /* First row */
    xKey1 = gptRaiseEdge.x + 3 as LONG;
    yKey1 = gptRaiseEdge.y + 3 as LONG;
    iKey = 0 as INT;
    while iKey < T1K_Q as INT {
        (T1_KEYPOS!(pT1, iKey)).x = xKey1;
        (T1_KEYPOS!(pT1, iKey)).y = yKey1;
        xKey1 += ((*pT1).cxDefWidth as LONG + 3 as LONG) as LONG;
        iKey += 1;
    }
    (T1_KEYPOS!(pT1, T1K_BACKSPACE)).y = yKey1;
    (T1_KEYPOS!(pT1, T1K_BACKSPACE)).x = xKey1;

    /* 2nd row */
    xKey2 = 3 as LONG + gptRaiseEdge.x + (*pT1).cxWidth48 as LONG + 3 as LONG;
    yKey2 = 3 as LONG + yKey1 + cyLarge;
    (T1_KEYPOS!(pT1, T1K_TAB)).x = gptRaiseEdge.x + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_TAB)).y = yKey2;
    iKey = T1K_Q as INT;
    while iKey < T1K_A as INT {
        (T1_KEYPOS!(pT1, iKey)).x = xKey2;
        (T1_KEYPOS!(pT1, iKey)).y = yKey2;
        xKey2 += ((*pT1).cxDefWidth as LONG + 3 as LONG) as LONG;
        iKey += 1;
    }
    (T1_KEYPOS!(pT1, T1K_ENTER)).x = xKey2;
    (T1_KEYPOS!(pT1, T1K_ENTER)).y = yKey2;

    /* 3rd row */
    xKey3 = gptRaiseEdge.x + 3 as LONG + (*pT1).cxWidth49 as LONG + 3 as LONG;
    yKey3 = yKey2 + cyLarge + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_CAPS)).x = gptRaiseEdge.x + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_CAPS)).y = yKey3;
    iKey = T1K_A as INT;
    while iKey < T1K_Z as INT {
        (T1_KEYPOS!(pT1, iKey)).x = xKey3;
        (T1_KEYPOS!(pT1, iKey)).y = yKey3;
        xKey3 += ((*pT1).cxDefWidth as LONG + 3 as LONG) as LONG;
        iKey += 1;
    }

    /* 4th row */
    xKey4 = gptRaiseEdge.x + (*pT1).cxWidth51or52 as LONG + 3 as LONG + 3 as LONG;
    yKey4 = yKey3 + cyLarge + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_L_SHIFT)).x = gptRaiseEdge.x + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_L_SHIFT)).y = yKey4;
    iKey = T1K_Z as INT;
    while iKey < T1K_BACKSPACE as INT {
        (T1_KEYPOS!(pT1, iKey)).x = xKey4;
        (T1_KEYPOS!(pT1, iKey)).y = yKey4;
        xKey4 += ((*pT1).cxDefWidth as LONG + 3 as LONG) as LONG;
        iKey += 1;
    }
    (T1_KEYPOS!(pT1, T1K_R_SHIFT)).x = xKey4;
    (T1_KEYPOS!(pT1, T1K_R_SHIFT)).y = yKey4;

    /* 5th row */
    xKey5 = gptRaiseEdge.x + 3 as LONG + (*pT1).cxWidth53or54 as LONG + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_L_CTRL)).x = gptRaiseEdge.x + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_ESCAPE)).x = xKey5;
    (T1_KEYPOS!(pT1, T1K_L_ALT)).x = xKey5 + (*pT1).cxWidth57 as LONG + 3 as LONG;

    yKey5 = yKey4 + cyLarge + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_L_ALT)).y = yKey5;
    (T1_KEYPOS!(pT1, T1K_ESCAPE)).y = (T1_KEYPOS!(pT1, T1K_L_ALT)).y;
    (T1_KEYPOS!(pT1, T1K_L_CTRL)).y = (T1_KEYPOS!(pT1, T1K_ESCAPE)).y;
    (T1_KEYPOS!(pT1, T1K_R_CTRL)).y = yKey5;
    (T1_KEYPOS!(pT1, T1K_SPACE)).y = (T1_KEYPOS!(pT1, T1K_R_CTRL)).y;
    (T1_KEYPOS!(pT1, T1K_R_ALT)).y = (T1_KEYPOS!(pT1, T1K_SPACE)).y;

    xKey6 = xKey5 + (*pT1).cxWidth57 as LONG + 3 as LONG + (*pT1).cxWidth55or56 as LONG + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_SPACE)).x = xKey6;

    xKey7 = xKey6 + (*pT1).cxWidth58 as LONG + 3 as LONG;
    (T1_KEYPOS!(pT1, T1K_R_ALT)).x = xKey7;
    (T1_KEYPOS!(pT1, T1K_R_CTRL)).x = xKey7 + (*pT1).cxWidth57 as LONG + (*pT1).cxWidth55or56 as LONG + 6 as LONG;
}

unsafe extern "C" fn T1_DrawConvexRect(
    hDC: HDC,
    x: INT,
    y: INT,
    width: INT,
    height: INT,
) {
    let hBlackPen: HGDIOBJ = GetStockObject((BLACK_PEN as i32));
    let hLtGrayBrush: HGDIOBJ = GetStockObject((LTGRAY_BRUSH as i32));
    let hGrayBrush: HGDIOBJ = GetStockObject((GRAY_BRUSH as i32));
    let dx: INT = width + 4;
    let dy: INT = height + 4;
    let x0: INT = x - 2;
    let y0: INT = y + height + 2;

    /* Face */
    SelectObject(hDC, hBlackPen);
    SelectObject(hDC, hLtGrayBrush);
    Rectangle(hDC, x0, y - 2, x0 + dx, y0);

    /* Rounded corners */
    PatBlt(hDC, x0, y - 2, 1, 1, PATCOPY);
    PatBlt(hDC, x0, y0, 1, -1, PATCOPY);
    PatBlt(hDC, x0 + dx, y - 2, -1, 1, PATCOPY);
    PatBlt(hDC, x0 + dx, y0, -1, -1, PATCOPY);

    /* Light edge */
    PatBlt(hDC, x0 + 1, y + dy - 3, 1, 2 - dy, WHITENESS);
    PatBlt(hDC, x0 + 1, y - 1, dx - 2, 1, WHITENESS);

    /* Dark edge */
    SelectObject(hDC, hGrayBrush);
    PatBlt(hDC, x0 + 1, y + dy - 3, dx - 2, -1, PATCOPY);
    PatBlt(hDC, x0 + dx - 1, y + dy - 3, -1, 2 - dy, PATCOPY);
}

unsafe extern "C" fn T1_DrawLabels(
    mut hDC: HDC,
    mut pT1: *const T1WINDOW,
    mut pszBmpName: LPCWSTR,
) {
    let mut hBitmap: HBITMAP = LoadBitmapW(ghImm32Inst as HINSTANCE, pszBmpName);
    let mut hdcMem: HDC = CreateCompatibleDC(hDC);
    let mut hbmOld: HGDIOBJ = SelectObject(hdcMem, hBitmap as HGDIOBJ);
    let mut iKey: INT = 0;
    iKey = 0 as INT;
    while iKey < T1K_BACKSPACE as INT {
        let mut ppt: *const POINT = &raw const T1_KEYPOS!(pT1, iKey);
        BitBlt(
            hDC,
            (*ppt).x as INT,
            (*ppt).y as INT,
            8 as INT,
            8 as INT,
            hdcMem,
            iKey * 8 as INT,
            0 as INT,
            SRCCOPY as DWORD,
        );
        iKey += 1;
    }
    SelectObject(hdcMem, hbmOld);
    DeleteDC(hdcMem);
    DeleteObject(hBitmap as HGDIOBJ);
}

unsafe extern "C" fn T1_InitBitmap(hWnd: HWND, pT1: PT1WINDOW) {
    let mut hDC: HDC = core::ptr::null_mut();
    let mut hMemDC: HDC = core::ptr::null_mut();
    let hNullPen: HGDIOBJ = GetStockObject((NULL_PEN as i32));
    let hbrLtGray: HGDIOBJ = GetStockObject((LTGRAY_BRUSH as i32));
    let mut rc: RECT = core::mem::zeroed();
    let mut iKey: INT;

    /* Create the bitmap */
    hDC = GetDC(hWnd);
    hMemDC = CreateCompatibleDC(hDC);
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    (*pT1).hbmKeyboard = CreateCompatibleBitmap(hDC, rc.right - rc.left, rc.bottom - rc.top);
    ReleaseDC(hWnd, hDC);

    /* Draw keyboard face */
    SelectObject(hMemDC, (*pT1).hbmKeyboard as HGDIOBJ);
    SelectObject(hMemDC, hNullPen);
    SelectObject(hMemDC, hbrLtGray);
    Rectangle(hMemDC, rc.left, rc.top, rc.right + 1, rc.bottom + 1);
    DrawEdge(hMemDC, core::ptr::addr_of_mut!(rc), EDGE_RAISED, BF_RECT);

    /* 53 --> Left [Ctrl] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_L_CTRL as usize].x, (*pT1).KeyPos[T1K_L_CTRL as usize].y,
                      (*pT1).cxWidth53or54, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth53or54 / 2 + (*pT1).KeyPos[T1K_L_CTRL as usize].x - 8,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_L_CTRL as usize].y - 4,
                    16, 9, (IDB_T1_CTRL) as INT);

    /* 54 --> Right [Ctrl] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_R_CTRL as usize].x, (*pT1).KeyPos[T1K_R_CTRL as usize].y,
                      (*pT1).cxWidth53or54, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth53or54 / 2 + (*pT1).KeyPos[T1K_R_CTRL as usize].x - 8,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_R_CTRL as usize].y - 4,
                    16, 9, (IDB_T1_CTRL) as INT);

    /* 57 --> [Esc] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_ESCAPE as usize].x, (*pT1).KeyPos[T1K_ESCAPE as usize].y,
                      (*pT1).cxWidth57, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth57   / 2 + (*pT1).KeyPos[T1K_ESCAPE as usize].x - 9,
                    (*pT1).cyDefHeight / 2 + (*pT1).KeyPos[T1K_ESCAPE as usize].y - 4,
                    18, 9, (IDB_T1_ESCAPE) as INT);

    /* 55 --> Left [Alt] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_L_ALT as usize].x, (*pT1).KeyPos[T1K_L_ALT as usize].y,
                      (*pT1).cxWidth55or56, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth55or56 / 2 + (*pT1).KeyPos[T1K_L_ALT as usize].x - 8,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_L_ALT as usize].y - 4,
                    16, 9, (IDB_T1_ALT) as INT);

    /* 56 --> Right [Alt] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_R_ALT as usize].x, (*pT1).KeyPos[T1K_R_ALT as usize].y,
                      (*pT1).cxWidth55or56, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth55or56 / 2 + (*pT1).KeyPos[T1K_R_ALT as usize].x - 8,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_R_ALT as usize].y - 4,
                    16, 9, (IDB_T1_ALT) as INT);

    /* 58 --> [Space] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_SPACE as usize].x, (*pT1).KeyPos[T1K_SPACE as usize].y,
                      (*pT1).cxWidth58, (*pT1).cyDefHeight);

    /* 51 --> Left [Shift] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_L_SHIFT as usize].x, (*pT1).KeyPos[T1K_L_SHIFT as usize].y,
                      (*pT1).cxWidth51or52, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth51or52 / 2 + (*pT1).KeyPos[T1K_L_SHIFT as usize].x - 11,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_L_SHIFT as usize].y - 4,
                    23, 9, (IDB_T1_SHIFT) as INT);

    /* 52 --> Right [Shift] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_R_SHIFT as usize].x, (*pT1).KeyPos[T1K_R_SHIFT as usize].y,
                      (*pT1).cxWidth51or52, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth51or52 / 2 + (*pT1).KeyPos[T1K_R_SHIFT as usize].x - 11,
                    (*pT1).cyDefHeight   / 2 + (*pT1).KeyPos[T1K_R_SHIFT as usize].y - 4,
                    23, 9, (IDB_T1_SHIFT) as INT);

    /* 49 --> [Caps] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_CAPS as usize].x, (*pT1).KeyPos[T1K_CAPS as usize].y,
                      (*pT1).cxWidth49, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth49   / 2 + (*pT1).KeyPos[T1K_CAPS as usize].x - 11,
                    (*pT1).cyDefHeight / 2 + (*pT1).KeyPos[T1K_CAPS as usize].y - 4,
                    22, 9, (IDB_T1_CAPS) as INT);

    /* 48 --> [Tab] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_TAB as usize].x, (*pT1).KeyPos[T1K_TAB as usize].y,
                      (*pT1).cxWidth48, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth48   / 2 + (*pT1).KeyPos[T1K_TAB as usize].x - 8,
                    (*pT1).cyDefHeight / 2 + (*pT1).KeyPos[T1K_TAB as usize].y - 4,
                    16, 9, (IDB_T1_TAB) as INT);

    /* 50 --> [Enter] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_ENTER as usize].x, (*pT1).KeyPos[T1K_ENTER as usize].y,
                      (*pT1).cxWidth50, (*pT1).cyHeight50);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth50  / 2 + (*pT1).KeyPos[T1K_ENTER as usize].x - 13,
                    (*pT1).cyHeight50 / 2 + (*pT1).KeyPos[T1K_ENTER as usize].y - 4,
                    26, 9, (IDB_T1_ENTER) as INT);

    /* 47 --> [BackSpace] */
    T1_DrawConvexRect(hMemDC,
                      (*pT1).KeyPos[T1K_BACKSPACE as usize].x, (*pT1).KeyPos[T1K_BACKSPACE as usize].y,
                      (*pT1).cxWidth47, (*pT1).cyDefHeight);
    Imm32DrawBitmap(hMemDC,
                    (*pT1).cxWidth47   / 2 + (*pT1).KeyPos[T1K_BACKSPACE as usize].x - 8,
                    (*pT1).cyDefHeight / 2 + (*pT1).KeyPos[T1K_BACKSPACE as usize].y - 4,
                    16, 9, (IDB_T1_BACKSPACE) as INT);

    /* Regular keys */
    iKey = 0;
    while iKey < T1K_BACKSPACE {
        let ppt: LPPOINT = core::ptr::addr_of_mut!((*pT1).KeyPos).cast::<POINT>().offset(iKey as isize);
        T1_DrawConvexRect(hMemDC, (*ppt).x, (*ppt).y, (*pT1).cxDefWidth, (*pT1).cyDefHeight);
        iKey += 1;
    }

    T1_DrawLabels(hMemDC, pT1 as *const T1WINDOW, MAKEINTRESOURCEW!(IDB_T1_CHARS) as LPCWSTR);
    DeleteDC(hMemDC);
}

unsafe extern "C" fn T1_OnCreate(hWnd: HWND) -> INT {
    let mut pT1: PT1WINDOW;
    let hGlobal: HGLOBAL = GlobalAlloc(GHND, core::mem::size_of::<T1WINDOW>() as SIZE_T);
    if hGlobal.is_null() {
        return -1;
    }

    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if pT1.is_null() {
        GlobalFree(hGlobal);
        return -1;
    }

    SetWindowLongPtrW(hWnd, 0, hGlobal as LONG_PTR);
    (*pT1).pt1.y = -1;
    (*pT1).pt1.x = (*pT1).pt1.y;
    (*pT1).PressedKey = (T1K_NONE as u32);
    (*pT1).CharSet = CHINESEBIG5_CHARSET;

    T1_InitButtonPos(pT1);
    T1_InitBitmap(hWnd, pT1);
    GlobalUnlock(hGlobal);

    return 0;
}

unsafe extern "C" fn T1_DrawDragBorder(
    hWnd: HWND,
    ppt1: *const POINT,
    ppt2: *const POINT,
) {
    let mut cxBorder: INT = GetSystemMetrics((SM_CXBORDER) as i32) as INT;
    let mut cyBorder: INT = GetSystemMetrics((SM_CYBORDER) as i32) as INT;
    let mut x: INT = (*ppt1).x - (*ppt2).x;
    let mut y: INT = (*ppt1).y - (*ppt2).y;
    let mut hGrayBrush: HGDIOBJ = GetStockObject((GRAY_BRUSH as i32));
    let mut rc: RECT = core::mem::zeroed();
    let mut hDisplayDC: HDC;

    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc));
    hDisplayDC = CreateDCW(
        RUSTCONV_WSTR_F28AD8C4C16D.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    SelectObject(hDisplayDC, hGrayBrush);
    PatBlt(
        hDisplayDC,
        x,
        y,
        rc.right - rc.left - cxBorder,
        cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x,
        cyBorder + y,
        cxBorder,
        rc.bottom - rc.top - cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x + cxBorder,
        y + rc.bottom - rc.top,
        rc.right - rc.left - cxBorder,
        -cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x + rc.right - rc.left,
        y,
        -cxBorder,
        rc.bottom - rc.top - cyBorder,
        PATINVERT,
    );
    DeleteDC(hDisplayDC);
}

unsafe extern "C" fn T1_OnDestroy(hWnd: HWND) {
    let mut hGlobal: HGLOBAL;
    let mut pT1: PT1WINDOW;
    let mut hwndOwner: HWND;

    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return;
    }

    if (*pT1).pt1.x != -(1 as LONG)
        && (*pT1).pt1.y != -(1 as LONG)
    {
        T1_DrawDragBorder(hWnd, &raw mut (*pT1).pt0, &raw mut (*pT1).pt1);
    }

    DeleteObject((*pT1).hbmKeyboard as HGDIOBJ);
    GlobalUnlock(hGlobal);
    GlobalFree(hGlobal);

    hwndOwner = GetWindow(hWnd, GW_OWNER as UINT);
    if !hwndOwner.is_null() {
        SendMessageW(
            hwndOwner,
            WM_IME_NOTIFY as UINT,
            IMN_SOFTKBDDESTROYED as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn T1_InvertButton(
    hWnd: HWND,
    hDC: HDC,
    pT1: *const T1WINDOW,
    iPressed: UINT,
) {
    let mut cxWidth: INT = (*pT1).cxDefWidth;
    let mut cyHeight: INT = (*pT1).cyDefHeight;
    let mut hChoiceDC: HDC = core::ptr::null_mut();

    if iPressed >= (T1K_NONE as u32) {
        return;
    }

    if !hDC.is_null() {
        hChoiceDC = hDC;
    } else {
        hChoiceDC = GetDC(hWnd);
    }

    if iPressed >= (T1K_BACKSPACE as u32) {
        if iPressed == (T1K_BACKSPACE as u32) {
            cxWidth = (*pT1).cxWidth47;
        } else if iPressed == (T1K_TAB as u32) {
            cxWidth = (*pT1).cxWidth48;
        } else if iPressed == (T1K_ENTER as u32) {
            cxWidth = (*pT1).cxWidth50;
            cyHeight = (*pT1).cyHeight50;
        } else if iPressed == (T1K_ESCAPE as u32) {
            cxWidth = (*pT1).cxWidth57;
        } else if iPressed == (T1K_SPACE as u32) {
            cxWidth = (*pT1).cxWidth58;
        } else {
            cxWidth = 0;
            MessageBeep(0xFFFFFFFF);
        }
    }

    if cxWidth > 0 {
        PatBlt(
            hChoiceDC,
            (*pT1).KeyPos[iPressed as usize].x - 1,
            (*pT1).KeyPos[iPressed as usize].y - 1,
            cxWidth + 2,
            cyHeight + 2,
            DSTINVERT,
        );
    }

    if hDC.is_null() {
        ReleaseDC(hWnd, hChoiceDC);
    }
}
unsafe extern "C" fn T1_OnDraw(mut hDC: HDC, mut hWnd: HWND) {
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut pT1: PT1WINDOW = core::ptr::null_mut();
    let mut hMemDC: HDC = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();

    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return;
    }

    hMemDC = CreateCompatibleDC(hDC);
    SelectObject(hMemDC, (*pT1).hbmKeyboard as HGDIOBJ);
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    BitBlt(
        hDC,
        0 as INT,
        0 as INT,
        (rc.right - rc.left) as INT,
        (rc.bottom - rc.top) as INT,
        hMemDC,
        0 as INT,
        0 as INT,
        SRCCOPY as DWORD,
    );
    DeleteDC(hMemDC);

    if (*pT1).PressedKey < T1K_NONE as UINT {
        T1_InvertButton(hWnd, hDC, pT1 as *const T1WINDOW, (*pT1).PressedKey);
    }

    GlobalUnlock(hGlobal);
}

unsafe extern "C" fn T1_HitTest(pT1: *const T1WINDOW, ppt: *const POINT) -> UINT {
    let mut iKey: INT = 0;
    iKey = 0 as INT;
    while iKey < T1K_BACKSPACE as INT {
        let pptKey: *const POINT = &raw const (*pT1).KeyPos[iKey as usize];
        if Imm32PtInRect(
            ppt,
            (*pptKey).x,
            (*pptKey).y,
            (*pT1).cxDefWidth as LONG,
            (*pT1).cyDefHeight as LONG,
        ) != 0
        {
            return iKey as UINT;
        }
        iKey += 1;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_BACKSPACE as usize].x,
        (*pT1).KeyPos[T1K_BACKSPACE as usize].y,
        (*pT1).cxWidth47 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
    {
        return T1K_BACKSPACE as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_TAB as usize].x,
        (*pT1).KeyPos[T1K_TAB as usize].y,
        (*pT1).cxWidth48 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
    {
        return T1K_TAB as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_CAPS as usize].x,
        (*pT1).KeyPos[T1K_CAPS as usize].y,
        (*pT1).cxWidth49 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
    {
        return T1K_CAPS as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_ENTER as usize].x,
        (*pT1).KeyPos[T1K_ENTER as usize].y,
        (*pT1).cxWidth50 as LONG,
        (*pT1).cyHeight50 as LONG,
    ) != 0
    {
        return T1K_ENTER as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_L_SHIFT as usize].x,
        (*pT1).KeyPos[T1K_L_SHIFT as usize].y,
        (*pT1).cxWidth51or52 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
        || Imm32PtInRect(
            ppt,
            (*pT1).KeyPos[T1K_R_SHIFT as usize].x,
            (*pT1).KeyPos[T1K_R_SHIFT as usize].y,
            (*pT1).cxWidth51or52 as LONG,
            (*pT1).cyDefHeight as LONG,
        ) != 0
    {
        return T1K_L_SHIFT as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_L_CTRL as usize].x,
        (*pT1).KeyPos[T1K_L_CTRL as usize].y,
        (*pT1).cxWidth53or54 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
        || Imm32PtInRect(
            ppt,
            (*pT1).KeyPos[T1K_R_CTRL as usize].x,
            (*pT1).KeyPos[T1K_R_CTRL as usize].y,
            (*pT1).cxWidth53or54 as LONG,
            (*pT1).cyDefHeight as LONG,
        ) != 0
    {
        return T1K_L_CTRL as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_L_ALT as usize].x,
        (*pT1).KeyPos[T1K_L_ALT as usize].y,
        (*pT1).cxWidth55or56 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
        || Imm32PtInRect(
            ppt,
            (*pT1).KeyPos[T1K_R_ALT as usize].x,
            (*pT1).KeyPos[T1K_R_ALT as usize].y,
            (*pT1).cxWidth55or56 as LONG,
            (*pT1).cyDefHeight as LONG,
        ) != 0
    {
        return T1K_L_ALT as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_ESCAPE as usize].x,
        (*pT1).KeyPos[T1K_ESCAPE as usize].y,
        (*pT1).cxWidth57 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
    {
        return T1K_ESCAPE as UINT;
    }

    if Imm32PtInRect(
        ppt,
        (*pT1).KeyPos[T1K_SPACE as usize].x,
        (*pT1).KeyPos[T1K_SPACE as usize].y,
        (*pT1).cxWidth58 as LONG,
        (*pT1).cyDefHeight as LONG,
    ) != 0
    {
        return T1K_SPACE as UINT;
    }

    return T1K_NONE as UINT;
}

unsafe extern "C" fn T1_IsValidButton(mut iKey: UINT, mut pT1: *const T1WINDOW) -> BOOL {
    if iKey < T1K_BACKSPACE as ::core::ffi::c_int as UINT {
        return ((*pT1).chKeyChar[iKey as usize] != 0) as ::core::ffi::c_int;
    }
    return (iKey <= T1K_TAB as ::core::ffi::c_int as UINT
        || iKey == T1K_ENTER as ::core::ffi::c_int as UINT
        || T1K_ESCAPE as ::core::ffi::c_int as UINT <= iKey
            && iKey <= T1K_SPACE as ::core::ffi::c_int as UINT) as ::core::ffi::c_int;
}

unsafe extern "C" fn T1_OnSetCursor(mut hWnd: HWND, mut lParam: LPARAM) -> BOOL {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pT1: PT1WINDOW = ::core::ptr::null_mut::<T1WINDOW>();
    let mut hCursor: HCURSOR = ::core::ptr::null_mut::<HICON__>();
    let mut iPressed: UINT = 0;
    let mut iKey: UINT = 0;
    let mut rc: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut rcWork: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return FALSE as BOOL;
    }
    if (*pT1).pt1.x != -(1 as ::core::ffi::c_int) as LONG
        && (*pT1).pt1.y != -(1 as ::core::ffi::c_int) as LONG
    {
        SetCursor(LoadCursorW(
            ::core::ptr::null_mut::<HINSTANCE__>(),
            (IDC_SIZEALL as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR
                as LPWSTR as LPCWSTR,
        ));
        GlobalUnlock(hGlobal);
        return TRUE as BOOL;
    }
    GetCursorPos(&raw mut (*pT1).pt0);
    ScreenToClient(hWnd, &raw mut (*pT1).pt0);
    iKey = T1_HitTest(pT1 as *const T1WINDOW, &raw mut (*pT1).pt0);
    if iKey >= T1K_NONE as ::core::ffi::c_int as UINT {
        hCursor = LoadCursorW(
            ::core::ptr::null_mut::<HINSTANCE__>(),
            (IDC_SIZEALL as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR
                as LPWSTR as LPCWSTR,
        );
    } else {
        hCursor = LoadCursorW(
            ::core::ptr::null_mut::<HINSTANCE__>(),
            (IDC_HAND as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR
                as LPWSTR as LPCWSTR,
        );
    }
    SetCursor(hCursor);
    if (HIWORD!(lParam as DWORD) as ::core::ffi::c_int) == WM_LBUTTONDOWN as ::core::ffi::c_int
    {
        SetCapture(hWnd);
        iPressed = (*pT1).PressedKey;
        if iPressed < T1K_NONE as ::core::ffi::c_int as UINT {
            let mut iVK: UINT = gT1K2VK[iPressed as usize] as UINT;
            keybd_event(
                iVK as BYTE,
                guScanCode[iVK as usize] as BYTE,
                KEYEVENTF_KEYUP as DWORD,
                0 as ULONG_PTR,
            );
            T1_InvertButton(
                hWnd,
                ::core::ptr::null_mut::<HDC__>(),
                pT1 as *const T1WINDOW,
                (*pT1).PressedKey,
            );
            (*pT1).PressedKey = T1K_NONE as UINT;
        }
        if iKey >= T1K_NONE as ::core::ffi::c_int as UINT {
            Imm32GetAllMonitorSize(&raw mut rcWork);
            GetCursorPos(&raw mut (*pT1).pt0);
            GetWindowRect(hWnd, &raw mut rc);
            (*pT1).pt1.x = (*pT1).pt0.x - rc.left;
            (*pT1).pt1.y = (*pT1).pt0.y - rc.top;
            T1_DrawDragBorder(hWnd, &raw mut (*pT1).pt0, &raw mut (*pT1).pt1);
        } else if T1_IsValidButton(iKey, pT1 as *const T1WINDOW) != 0 {
            let mut iVK_0: UINT = gT1K2VK[iKey as usize] as UINT;
            keybd_event(
                iVK_0 as BYTE,
                guScanCode[iVK_0 as usize] as BYTE,
                0 as DWORD,
                0 as ULONG_PTR,
            );
            (*pT1).PressedKey = iKey;
            T1_InvertButton(
                hWnd,
                (0 as usize) as HDC,
                pT1 as *const T1WINDOW,
                iKey,
            );
        } else {
            MessageBeep(0xFFFFFFFF as UINT);
        }
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn T1_OnMouseMove(hWnd: HWND) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;
    let mut hGlobal: HGLOBAL;
    let mut pT1: PT1WINDOW;

    hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return (FALSE) as BOOL;
    }

    if (*pT1).pt1.x != -(1 as LONG)
        && (*pT1).pt1.y != -(1 as LONG)
    {
        T1_DrawDragBorder(
            hWnd,
            core::ptr::addr_of_mut!((*pT1).pt0),
            core::ptr::addr_of_mut!((*pT1).pt1),
        );
        GetCursorPos(core::ptr::addr_of_mut!((*pT1).pt0));
        T1_DrawDragBorder(
            hWnd,
            core::ptr::addr_of_mut!((*pT1).pt0),
            core::ptr::addr_of_mut!((*pT1).pt1),
        );
        ret = (TRUE) as i32;
    }

    GlobalUnlock(hGlobal);
    return ret;
}

unsafe extern "C" fn T1_OnButtonUp(hWnd: HWND) -> BOOL {
    let mut ret: BOOL = FALSE as BOOL;
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut pT1: PT1WINDOW = core::ptr::null_mut();
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut iPressed: INT = 0;
    let mut hwndOwner: HWND = core::ptr::null_mut();
    let mut hwndCapture: HWND = GetCapture();
    let mut hIMC: HIMC = core::ptr::null_mut();
    let mut pIC: LPINPUTCONTEXT = core::ptr::null_mut();

    if hwndCapture == hWnd {
        ReleaseCapture();
    }

    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return FALSE as BOOL;
    }

    iPressed = (*pT1).PressedKey as INT;
    if iPressed >= T1K_NONE as INT {
        if (*pT1).pt1.x != -(1 as LONG)
            && (*pT1).pt1.y != -(1 as LONG)
        {
            T1_DrawDragBorder(hWnd, &raw mut (*pT1).pt0, &raw mut (*pT1).pt1);
            x = ((*pT1).pt0.x - (*pT1).pt1.x) as INT;
            y = ((*pT1).pt0.y - (*pT1).pt1.y) as INT;
            SetWindowPos(
                hWnd,
                core::ptr::null_mut(),
                x as ::core::ffi::c_int,
                y as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE) as UINT,
            );
            (*pT1).pt1.y = -(1 as LONG);
            (*pT1).pt1.x = (*pT1).pt1.y;
            (*pT1).PressedKey = T1K_NONE as UINT;
            ret = TRUE as BOOL;

            hwndOwner = GetWindow(hWnd, GW_OWNER as UINT);
            hIMC = GetWindowLongPtrW(hwndOwner, 0 as ::core::ffi::c_int) as HIMC;
            if !hIMC.is_null() {
                pIC = ImmLockIMC(hIMC);
                if !pIC.is_null() {
                    (*pIC).fdwInit |= INIT_SOFTKBDPOS as DWORD;
                    (*pIC).ptSoftKbdPos.x = x as LONG;
                    (*pIC).ptSoftKbdPos.y = y as LONG;
                    ImmUnlockIMC(hIMC);
                }
            }
        }
    } else {
        let mut iVK: UINT = gT1K2VK[iPressed as usize] as UINT;
        keybd_event(
            iVK as BYTE,
            guScanCode[iVK as usize] as BYTE,
            KEYEVENTF_KEYUP as DWORD,
            0 as ULONG_PTR,
        );

        T1_InvertButton(hWnd, core::ptr::null_mut(), pT1, (*pT1).PressedKey);
        (*pT1).PressedKey = T1K_NONE as UINT;
        ret = TRUE as BOOL;
    }

    GlobalUnlock(hGlobal);
    return ret;
}

unsafe extern "C" fn T1_SetData(hWnd: HWND, pData: *const SOFTKBDDATA) -> LRESULT {
    let mut hGlobal: HGLOBAL;
    let mut pT1: PT1WINDOW;
    let mut hDC: HDC;
    let mut hMemDC: HDC;
    let mut hFont: HFONT;
    let mut hFontOld: HGDIOBJ;
    let mut hbmOld: HGDIOBJ;
    let mut rc: RECT = core::mem::zeroed();
    let mut iKey: INT;

    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pT1 = GlobalLock(hGlobal) as PT1WINDOW;
    if hGlobal.is_null() || pT1.is_null() {
        return 1 as LRESULT;
    }

    hDC = GetDC(hWnd);
    hMemDC = CreateCompatibleDC(hDC);
    ReleaseDC(hWnd, hDC);

    hbmOld = SelectObject(hMemDC, (*pT1).hbmKeyboard as HGDIOBJ);
    SetBkColor(
        hMemDC,
        RGB!(192 as BYTE, 192 as BYTE, 192 as BYTE),
    );

    if (*pT1).CharSet == DEFAULT_CHARSET as DWORD {
        hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(g_T1LogFont));
    } else {
        let mut lf: LOGFONTW = g_T1LogFont;
        lf.lfCharSet = (*pT1).CharSet as BYTE;
        hFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lf));
    }
    hFontOld = SelectObject(hMemDC, hFont as HGDIOBJ);

    iKey = 0 as INT;
    while iKey < T1K_BACKSPACE as INT {
        let x0: INT = T1_KEYPOS!(pT1, iKey).x as INT;
        let y0: INT = T1_KEYPOS!(pT1, iKey).y as INT;
        let x: INT = x0 + 6 as INT;
        let y: INT = y0 + 8 as INT;
        (*pT1).chKeyChar[iKey as usize] =
            (*pData).wCode[0 as usize][gT1K2VK[iKey as usize] as usize];
        let wch: WCHAR = (*pT1).chKeyChar[iKey as usize];
        SetRect(
            core::ptr::addr_of_mut!(rc),
            x,
            y,
            x0 + (*pT1).cxDefWidth as INT,
            y0 + (*pT1).cyDefHeight as INT,
        );
        ExtTextOutW(
            hMemDC,
            x,
            y,
            ETO_OPAQUE as UINT,
            core::ptr::addr_of_mut!(rc),
            core::ptr::addr_of!(wch) as LPCWSTR,
            (wch as ::core::ffi::c_int != 0 as ::core::ffi::c_int) as ::core::ffi::c_int as UINT,
            core::ptr::null(),
        );
        iKey += 1;
    }

    DeleteObject(SelectObject(hMemDC, hFontOld));
    SelectObject(hMemDC, hbmOld);
    DeleteDC(hMemDC);
    GlobalUnlock(hGlobal);
    return 0 as LRESULT;
}

unsafe extern "C" fn T1_OnImeControl(
    mut hWnd: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut ret: LRESULT = 1 as LRESULT;
    let mut pT1: PT1WINDOW = core::ptr::null_mut::<T1WINDOW>();
    let mut hGlobal: HGLOBAL = core::ptr::null_mut::<core::ffi::c_void>();
    match wParam as u32 {
        IMC_GETSOFTKBDFONT => {
            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    958 as core::ffi::c_int,
                    b"IMC_GETSOFTKBDFONT: %p\n\0" as *const u8 as *const core::ffi::c_char,
                    lParam,
                );
            }
            hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
            pT1 = GlobalLock(hGlobal) as PT1WINDOW;
            if !hGlobal.is_null() && !pT1.is_null() {
                let plf: LPLOGFONTW = lParam as LPLOGFONTW;
                let CharSet: DWORD = (*pT1).CharSet;
                GlobalUnlock(hGlobal);

                *plf = g_T1LogFont;
                if CharSet != (DEFAULT_CHARSET as DWORD) {
                    (*plf).lfCharSet = CharSet as BYTE;
                }

                ret = 0 as LRESULT;
            }
            // break;
        }
        IMC_SETSOFTKBDFONT => {
            let plf: *const LOGFONTW = lParam as LPLOGFONTW as *const LOGFONTW;
            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_0,
                    __dbch_0,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    978 as core::ffi::c_int,
                    b"IMC_SETSOFTKBDFONT: %p\n\0" as *const u8 as *const core::ffi::c_char,
                    lParam,
                );
            }
            if g_T1LogFont.lfCharSet as core::ffi::c_int == (*plf).lfCharSet as core::ffi::c_int {
                return 0 as LRESULT;
            }

            hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
            pT1 = GlobalLock(hGlobal) as PT1WINDOW;
            if !hGlobal.is_null() && !pT1.is_null() {
                (*pT1).CharSet = (*plf).lfCharSet as DWORD;
                GlobalUnlock(hGlobal);
                return 0 as LRESULT;
            }

            // break;
        }
        IMC_GETSOFTKBDPOS => {
            let mut rc: RECT = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_1,
                    __dbch_1,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    996 as core::ffi::c_int,
                    b"IMC_GETSOFTKBDPOS\n\0" as *const u8 as *const core::ffi::c_char,
                );
            }
            GetWindowRect(hWnd, &raw mut rc);
            return MAKELRESULT!(rc.left, rc.top);
        }
        IMC_SETSOFTKBDPOS => {
            let mut pt: POINT = POINT { x: 0, y: 0 };
            let mut hwndParent: HWND = core::ptr::null_mut::<HWND__>();

            // POINTSTOPOINT(pt, lParam), expanded in place
            pt.x = ((lParam as DWORD) & 0xffff) as WORD as LONG;
            pt.y = (((lParam as DWORD) >> 16) & 0xffff) as WORD as LONG;

            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_2,
                    __dbch_2,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    1006 as core::ffi::c_int,
                    b"IMC_SETSOFTKBDPOS(%ld, %ld)\n\0" as *const u8 as *const core::ffi::c_char,
                    pt.x,
                    pt.y,
                );
            }

            SetWindowPos(
                hWnd,
                core::ptr::null_mut::<HWND__>(),
                pt.x as core::ffi::c_int,
                pt.y as core::ffi::c_int,
                0 as core::ffi::c_int,
                0 as core::ffi::c_int,
                (SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE) as UINT,
            );

            hwndParent = GetParent(hWnd);
            if !hwndParent.is_null() {
                let hIMC: HIMC =
                    GetWindowLongPtrW(hwndParent, 0) as HIMC;
                if !hIMC.is_null() {
                    let pIC: LPINPUTCONTEXT = ImmLockIMC(hIMC);
                    if !pIC.is_null() {
                        (*pIC).ptSoftKbdPos.x = pt.x;
                        (*pIC).ptSoftKbdPos.y = pt.y;
                        ImmUnlockIMC(hIMC);
                        return 0 as LRESULT;
                    }
                }
            }
            // break;
        }
        IMC_GETSOFTKBDSUBTYPE | IMC_SETSOFTKBDSUBTYPE => {
            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch_3: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_3: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_3,
                    __dbch_3,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    1031 as core::ffi::c_int,
                    b"IMC_GETSOFTKBDSUBTYPE/IMC_SETSOFTKBDSUBTYPE\n\0" as *const u8
                        as *const core::ffi::c_char,
                );
            }
            hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
            pT1 = GlobalLock(hGlobal) as PT1WINDOW;
            if hGlobal.is_null() || pT1.is_null() {
                return -(1) as LRESULT;
            }

            ret = (*pT1).KeyboardSubType as LRESULT;

            if wParam == (IMC_SETSOFTKBDSUBTYPE as WPARAM) {
                (*pT1).KeyboardSubType = lParam;
            }

            GlobalUnlock(hGlobal);
            // break;
        }
        IMC_SETSOFTKBDDATA => {
            if ((*__wine_dbch___default).flags as core::ffi::c_int
                & ((1 as core::ffi::c_int) << (__WINE_DBCL_TRACE as core::ffi::c_int)))
                != 0
            {
                let __dbch_4: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl_4: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl_4,
                    __dbch_4,
                    b"win32ss/user/imm32/softkbd.c\0"
                        as *const u8 as *const core::ffi::c_char,
                    b"T1_OnImeControl\0" as *const u8 as *const core::ffi::c_char,
                    1047 as core::ffi::c_int,
                    b"IMC_SETSOFTKBDDATA: %p\n\0" as *const u8 as *const core::ffi::c_char,
                    lParam,
                );
            }
            ret = T1_SetData(hWnd, lParam as *mut SOFTKBDDATA);
            if ret == 0 {
                InvalidateRect(hWnd, core::ptr::null::<RECT>(), (FALSE) as BOOL);
                PostMessageW(
                    hWnd,
                    WM_PAINT as UINT,
                    0 as WPARAM,
                    0 as LPARAM,
                );
            }
            // break;
        }
        _ => {}
    }
    return ret;
}

unsafe extern "system" fn T1_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            return T1_OnCreate(hWnd) as LRESULT;
        }
        WM_DESTROY => {
            T1_OnDestroy(hWnd);
        }
        WM_SETCURSOR => {
            if !(T1_OnSetCursor(hWnd, lParam) != 0) {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_MOUSEMOVE => {
            if !(T1_OnMouseMove(hWnd) != 0) {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hDC: HDC = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            T1_OnDraw(hDC, hWnd);
            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
        }
        WM_SHOWWINDOW => {
            if lParam == 0 && wParam != (SW_SHOWNORMAL as WPARAM) {
                T1_OnButtonUp(hWnd);
            }
            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
        WM_MOUSEACTIVATE => {
            return MA_NOACTIVATE as LRESULT;
        }
        WM_LBUTTONUP => {
            if !(T1_OnButtonUp(hWnd) != 0) {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_IME_CONTROL => {
            return T1_OnImeControl(hWnd, wParam, lParam);
        }
        _ => {
            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
    }

    return 0 as LRESULT;
}

unsafe extern "C" fn C1_InitButtonPos() {
    let mut x: LONG = 0 as LONG;
    let mut y: LONG = 0 as LONG;
    let mut iKey: INT = 0;

    /* 1st row */
    iKey = C1K_OEM_3 as INT;
    while iKey < C1K_Q as INT {
        gptC1ButtonPos[iKey as usize].x = x;
        gptC1ButtonPos[iKey as usize].y = y;
        x += 24 as LONG;
        iKey += 1;
    }
    gptC1ButtonPos[C1K_BACKSPACE as usize].x = x;
    gptC1ButtonPos[C1K_BACKSPACE as usize].y = y;

    /* 2nd row */
    y = 28 as LONG;
    gptC1ButtonPos[C1K_TAB as usize].x = 0 as LONG;
    gptC1ButtonPos[C1K_TAB as usize].y = y;
    x = 36 as LONG;
    while iKey < C1K_A as INT {
        gptC1ButtonPos[iKey as usize].x = x;
        gptC1ButtonPos[iKey as usize].y = y;
        x += 24 as LONG;
        iKey += 1;
    }

    /* 3rd row */
    y = 56 as LONG;
    gptC1ButtonPos[C1K_CAPS as usize].x = 0 as LONG;
    gptC1ButtonPos[C1K_CAPS as usize].y = y;
    x = 42 as LONG;
    while iKey < C1K_Z as INT {
        gptC1ButtonPos[iKey as usize].x = x;
        gptC1ButtonPos[iKey as usize].y = y;
        x += 24 as LONG;
        iKey += 1;
    }
    gptC1ButtonPos[C1K_ENTER as usize].x = x;
    gptC1ButtonPos[C1K_ENTER as usize].y = y;

    /* 4th row */
    y = 84 as LONG;
    gptC1ButtonPos[C1K_SHIFT as usize].x = 0 as LONG;
    gptC1ButtonPos[C1K_SHIFT as usize].y = y;
    x = 60 as LONG;
    while iKey < C1K_BACKSPACE as INT {
        gptC1ButtonPos[iKey as usize].x = x;
        gptC1ButtonPos[iKey as usize].y = y;
        x += 24 as LONG;
        iKey += 1;
    }

    /* 5th row */
    y = 112 as LONG;
    gptC1ButtonPos[C1K_INSERT as usize].x = 0 as LONG;
    gptC1ButtonPos[C1K_INSERT as usize].y = y;
    gptC1ButtonPos[C1K_DELETE as usize].x = 58 as LONG;
    gptC1ButtonPos[C1K_DELETE as usize].y = y;
    gptC1ButtonPos[C1K_SPACE as usize].x = 96 as LONG;
    gptC1ButtonPos[C1K_SPACE as usize].y = y;
    gptC1ButtonPos[C1K_ESCAPE as usize].x = 310 as LONG;
    gptC1ButtonPos[C1K_ESCAPE as usize].y = y;
}

unsafe extern "C" fn C1_DrawConvexRect(
    mut hDC: HDC,
    mut x: INT,
    mut y: INT,
    mut width: INT,
    mut height: INT,
) {
    let mut hLtGrayBrush: HGDIOBJ = GetStockObject((LTGRAY_BRUSH as i32));
    let mut hBlackPen: HGDIOBJ = GetStockObject((BLACK_PEN as i32));
    let mut hWhiteBrush: HGDIOBJ = GetStockObject((WHITE_BRUSH as i32));
    let mut hGrayBrush: HGDIOBJ = GetStockObject((GRAY_BRUSH as i32));
    let mut y2: INT = y + height - 1;

    /* Draw face */
    SelectObject(hDC, hLtGrayBrush);
    SelectObject(hDC, hBlackPen);
    Rectangle(hDC, x, y, x + width, y + height);

    /* Draw light edge */
    SelectObject(hDC, hWhiteBrush);
    PatBlt(hDC, x, y2, 2, 1 - height, PATCOPY);
    PatBlt(hDC, x, y, width - 1, 2, PATCOPY);

    /* Draw dark edge */
    SelectObject(hDC, hGrayBrush);
    PatBlt(hDC, x + 1, y2, width - 2, -1, PATCOPY);
    PatBlt(hDC, x + width - 1, y2, -1, 2 - height, PATCOPY);
}

unsafe extern "C" fn C1_InvertButton(hDC: HDC, iKey: INT) {
    let mut width: INT = 24;
    let mut height: INT = 28;

    if iKey < 0 {
        return;
    }

    match iKey {
        C1K_BACKSPACE | C1K_TAB => {
            width = 36;
        }
        C1K_CAPS | C1K_ENTER => {
            width = 42;
        }
        C1K_SHIFT => {
            width = 60;
        }
        C1K_INSERT | C1K_DELETE | C1K_ESCAPE => {
            width = 38;
            height = 24;
        }
        C1K_SPACE => {
            width = 172;
            height = 24;
        }
        _ => {}
    }

    BitBlt(
        hDC,
        gptC1ButtonPos[iKey as usize].x as INT,
        gptC1ButtonPos[iKey as usize].y as INT,
        width,
        height,
        hDC,
        gptC1ButtonPos[iKey as usize].x as INT,
        gptC1ButtonPos[iKey as usize].y as INT,
        DSTINVERT as DWORD,
    );
}

unsafe extern "C" fn C1_DrawLabel(hDC: HDC, nBitmapID: INT) {
    let mut hBitmap: HBITMAP;
    let mut hbmOld: HGDIOBJ;
    let mut hMemDC: HDC;
    let mut iKey: INT;

    hBitmap = LoadBitmapW(ghImm32Inst, MAKEINTRESOURCEW!(nBitmapID) as LPCWSTR);
    hMemDC = CreateCompatibleDC(hDC);
    hbmOld = SelectObject(hMemDC, hBitmap as HGDIOBJ);
    iKey = C1K_OEM_3 as INT;
    while iKey < C1K_BACKSPACE as INT {
        BitBlt(
            hDC,
            (gptC1ButtonPos[iKey as usize].x + 2 as LONG) as INT,
            (gptC1ButtonPos[iKey as usize].y + 2 as LONG) as INT,
            8 as INT,
            8 as INT,
            hMemDC,
            iKey * 8 as INT,
            0 as INT,
            SRCCOPY as DWORD,
        );
        iKey += 1;
    }
    DeleteObject(SelectObject(hMemDC, hbmOld));
    DeleteDC(hMemDC);
}

unsafe extern "C" fn C1_InitBitmap(
    hDC: HDC,
    x: INT,
    y: INT,
    width: INT,
    height: INT,
) {
    let hLtGrayBrush: HGDIOBJ = GetStockObject(LTGRAY_BRUSH as INT);
    let hNullPen: HGDIOBJ = GetStockObject(NULL_PEN as INT);
    let mut iKey: INT;

    /* Draw keyboard frame */
    SelectObject(hDC, hLtGrayBrush);
    SelectObject(hDC, hNullPen);
    Rectangle(
        hDC,
        x,
        y,
        width + 1 as INT,
        height + 1 as INT,
    );

    iKey = C1K_OEM_3 as INT;
    while iKey < C1K_BACKSPACE as INT {
        C1_DrawConvexRect(
            hDC,
            gptC1ButtonPos[iKey as usize].x as INT,
            gptC1ButtonPos[iKey as usize].y as INT,
            24 as INT,
            28 as INT,
        );
        iKey += 1;
    }

    C1_DrawLabel(hDC, IDB_C1_CHARS as INT);

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_BACKSPACE as usize].x as INT,
        gptC1ButtonPos[C1K_BACKSPACE as usize].y as INT,
        36 as INT,
        28 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_BACKSPACE as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_BACKSPACE as usize].y + 2 as LONG) as INT,
        32 as INT,
        24 as INT,
        IDB_C1_BACKSPACE as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_TAB as usize].x as INT,
        gptC1ButtonPos[C1K_TAB as usize].y as INT,
        36 as INT,
        28 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_TAB as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_TAB as usize].y + 2 as LONG) as INT,
        32 as INT,
        24 as INT,
        IDB_C1_TAB as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_CAPS as usize].x as INT,
        gptC1ButtonPos[C1K_CAPS as usize].y as INT,
        42 as INT,
        28 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_CAPS as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_CAPS as usize].y + 2 as LONG) as INT,
        38 as INT,
        24 as INT,
        IDB_C1_CAPS as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_ENTER as usize].x as INT,
        gptC1ButtonPos[C1K_ENTER as usize].y as INT,
        42 as INT,
        28 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_ENTER as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_ENTER as usize].y + 2 as LONG) as INT,
        38 as INT,
        24 as INT,
        IDB_C1_ENTER as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_SHIFT as usize].x as INT,
        gptC1ButtonPos[C1K_SHIFT as usize].y as INT,
        60 as INT,
        28 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_SHIFT as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_SHIFT as usize].y + 2 as LONG) as INT,
        56 as INT,
        24 as INT,
        IDB_C1_SHIFT as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_INSERT as usize].x as INT,
        gptC1ButtonPos[C1K_INSERT as usize].y as INT,
        38 as INT,
        24 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_INSERT as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_INSERT as usize].y + 2 as LONG) as INT,
        34 as INT,
        20 as INT,
        IDB_C1_INS as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_DELETE as usize].x as INT,
        gptC1ButtonPos[C1K_DELETE as usize].y as INT,
        38 as INT,
        24 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_DELETE as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_DELETE as usize].y + 2 as LONG) as INT,
        34 as INT,
        20 as INT,
        IDB_C1_DEL as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_SPACE as usize].x as INT,
        gptC1ButtonPos[C1K_SPACE as usize].y as INT,
        172 as INT,
        24 as INT,
    );

    C1_DrawConvexRect(
        hDC,
        gptC1ButtonPos[C1K_ESCAPE as usize].x as INT,
        gptC1ButtonPos[C1K_ESCAPE as usize].y as INT,
        38 as INT,
        24 as INT,
    );
    Imm32DrawBitmap(
        hDC,
        (gptC1ButtonPos[C1K_ESCAPE as usize].x + 2 as LONG) as INT,
        (gptC1ButtonPos[C1K_ESCAPE as usize].y + 2 as LONG) as INT,
        34 as INT,
        20 as INT,
        IDB_C1_ESCAPE as INT,
    );
}

unsafe extern "C" fn C1_OnCreate(mut hWnd: HWND) -> INT {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pC1: PC1WINDOW = ::core::ptr::null_mut::<C1WINDOW>();
    let mut hDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut hMemDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hbmOld: HGDIOBJ = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut hbmKeyboard: HBITMAP = ::core::ptr::null_mut::<HBITMAP__>();
    hGlobal = GlobalAlloc(
        GHND as UINT,
        ::core::mem::size_of::<C1WINDOW>() as SIZE_T,
    );
    if hGlobal.is_null() {
        return -(1 as INT);
    }
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if pC1.is_null() {
        GlobalFree(hGlobal);
        return -(1 as INT);
    }
    SetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int, hGlobal as LONG_PTR);
    if gbC1ButtonInit == 0 {
        C1_InitButtonPos();
        gbC1ButtonInit = TRUE as BOOL;
    }
    (*pC1).iPressedKey = -(1 as INT);
    (*pC1).CharSet = GB2312_CHARSET as DWORD;
    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));
    hDC = GetDC(hWnd);
    hMemDC = CreateCompatibleDC(hDC);
    hbmKeyboard = CreateCompatibleBitmap(
        hDC,
        (rc.right - rc.left) as INT,
        (rc.bottom - rc.top) as INT,
    );
    ReleaseDC(hWnd, hDC);
    hbmOld = SelectObject(hMemDC, hbmKeyboard as HGDIOBJ);
    C1_InitBitmap(
        hMemDC,
        rc.left as INT,
        rc.top as INT,
        rc.right as INT,
        rc.bottom as INT,
    );
    SelectObject(hMemDC, hbmOld);
    (*pC1).hbmKeyboard = hbmKeyboard;
    DeleteDC(hMemDC);
    GlobalUnlock(hGlobal);
    return 0 as INT;
}

unsafe extern "C" fn C1_OnDraw(hDC: HDC, hWnd: HWND) {
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut pC1: PC1WINDOW = core::ptr::null_mut();
    let mut hMemDC: HDC = core::ptr::null_mut();
    let mut rc: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut hbmOld: HGDIOBJ = core::ptr::null_mut();

    hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return;
    }

    GetClientRect(hWnd, core::ptr::addr_of_mut!(rc));

    hMemDC = CreateCompatibleDC(hDC);
    hbmOld = SelectObject(hMemDC, (*pC1).hbmKeyboard as HGDIOBJ);
    BitBlt(
        hDC,
        0,
        0,
        rc.right - rc.left,
        rc.bottom - rc.top,
        hMemDC,
        0,
        0,
        SRCCOPY,
    );
    SelectObject(hMemDC, hbmOld);
    DeleteDC(hMemDC);

    GlobalUnlock(hGlobal);
}

unsafe extern "C" fn C1_SetData(hWnd: HWND, pData: *const SOFTKBDDATA) -> BOOL {
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut pC1: PC1WINDOW = core::ptr::null_mut();
    let mut hDC: HDC = core::ptr::null_mut();
    let mut hMemDC: HDC = core::ptr::null_mut();
    let mut iKey: INT = 0;
    let mut bDisabled: BOOL = 0;
    let mut hbmKeyboard: HBITMAP = core::ptr::null_mut();
    let mut hbmOld: HGDIOBJ = core::ptr::null_mut();
    let mut hFontOld: HGDIOBJ = core::ptr::null_mut();
    let mut hFont: HFONT = core::ptr::null_mut();
    let mut rc: RECT = core::mem::zeroed();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut pDataEx: *const SOFTKBDDATAEX = core::ptr::null();

    if (*pData).uCount != 2 as UINT {
        return 0 as BOOL;
    }

    pDataEx = pData as *const SOFTKBDDATAEX;

    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return FALSE as BOOL;
    }

    hDC = GetDC(hWnd);
    hMemDC = CreateCompatibleDC(hDC);

    hbmKeyboard = (*pC1).hbmKeyboard;
    hbmOld = SelectObject(hMemDC, hbmKeyboard as HGDIOBJ);

    GetObjectW(
        GetStockObject((DEFAULT_GUI_FONT as i32)) as HANDLE,
        core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        &raw mut lf as LPVOID,
    );
    lf.lfHeight = -(12 as ::core::ffi::c_int) as LONG;
    if (*pC1).CharSet != DEFAULT_CHARSET as DWORD {
        lf.lfCharSet = (*pC1).CharSet as BYTE;
    }

    hFont = CreateFontIndirectW(&raw mut lf);
    hFontOld = SelectObject(hMemDC, hFont as HGDIOBJ);
    iKey = C1K_OEM_3 as ::core::ffi::c_int as INT;
    while iKey < C1K_BACKSPACE as ::core::ffi::c_int as INT {
        (*(*pC1).Data.as_mut_ptr().add(1 as usize))[iKey as usize] =
            (*pDataEx).wCode[0 as usize]
                [gC1K2VK[iKey as usize] as BYTE as usize] as WCHAR;
        (*(*pC1).Data.as_mut_ptr().add(0 as usize))[iKey as usize] =
            (*pDataEx).wCode[1 as usize]
                [gC1K2VK[iKey as usize] as BYTE as usize] as WCHAR;
        iKey += 1;
    }

    SetBkColor(
        hMemDC,
        RGB!(
            191 as BYTE,
            191 as BYTE,
            191 as BYTE,
        ),
    );
    iKey = C1K_OEM_3 as ::core::ffi::c_int as INT;
    while iKey < C1K_BACKSPACE as ::core::ffi::c_int as INT {
        /* Upper right */
        rc.right = gptC1ButtonPos[iKey as usize].x + 24 as LONG - 2 as LONG;
        rc.top = gptC1ButtonPos[iKey as usize].y + 2 as LONG;
        rc.left = rc.right - 14 as LONG;
        rc.bottom = rc.top + 14 as LONG;
        bDisabled = ((*(*pC1).Data.as_ptr().add(0 as usize))[iKey as usize]
            as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as BOOL;
        DrawTextW(
            hMemDC,
            &raw mut (*(*pC1).Data.as_mut_ptr().add(0 as usize))[iKey as usize] as LPCWSTR,
            (bDisabled == 0) as ::core::ffi::c_int,
            &raw mut rc,
            (DT_RIGHT | DT_TOP | DT_SINGLELINE) as UINT,
        );

        /* Lower left */
        rc.left = gptC1ButtonPos[iKey as usize].x + 2 as LONG;
        rc.bottom = gptC1ButtonPos[iKey as usize].y + 28 as LONG - 2 as LONG;
        rc.right = rc.left + 14 as LONG;
        rc.top = rc.bottom - 14 as LONG;
        bDisabled = ((*(*pC1).Data.as_ptr().add(1 as usize))[iKey as usize]
            as ::core::ffi::c_int
            == 0 as ::core::ffi::c_int) as ::core::ffi::c_int as BOOL;
        DrawTextW(
            hMemDC,
            &raw mut (*(*pC1).Data.as_mut_ptr().add(1 as usize))[iKey as usize] as LPCWSTR,
            (bDisabled == 0) as ::core::ffi::c_int,
            &raw mut rc,
            (DT_LEFT | DT_BOTTOM | DT_SINGLELINE) as UINT,
        );
        iKey += 1;
    }

    if (*pC1).dwFlags & FLAG_SHIFT_PRESSED as DWORD != 0 {
        C1_InvertButton(hMemDC, C1K_SHIFT as ::core::ffi::c_int as INT);
    }

    (*pC1).dwFlags = 0 as DWORD;

    SelectObject(hMemDC, hbmOld);
    DeleteObject(SelectObject(hMemDC, hFontOld));

    DeleteDC(hMemDC);
    ReleaseDC(hWnd, hDC);

    GlobalUnlock(hGlobal);
    return TRUE as BOOL;
}

unsafe extern "C" fn C1_DrawDragBorder(hWnd: HWND, ppt1: LPPOINT, ppt2: LPPOINT) {
    let mut hGrayBrush: HGDIOBJ = GetStockObject((GRAY_BRUSH as i32));
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut rc: RECT = core::mem::zeroed();
    let mut rcWork: RECT = core::mem::zeroed();
    let mut cxBorder: INT = GetSystemMetrics((SM_CXBORDER) as i32);
    let mut cyBorder: INT = GetSystemMetrics((SM_CYBORDER) as i32);
    let mut hDisplayDC: HDC = core::ptr::null_mut();

    Imm32GetAllMonitorSize(core::ptr::addr_of_mut!(rcWork));
    hDisplayDC = CreateDCW(
        RUSTCONV_WSTR_F28AD8C4C16D.as_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    SelectObject(hDisplayDC, hGrayBrush);
    x = (*ppt1).x - (*ppt2).x;
    y = (*ppt1).y - (*ppt2).y;
    if x < rcWork.left {
        x = rcWork.left;
    }
    if y < rcWork.top {
        y = rcWork.top;
    }

    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc));

    if rc.right - rc.left + x > rcWork.right {
        x = rc.left + rcWork.right - rc.right;
    }
    if y + rc.bottom - rc.top > rcWork.bottom {
        y = rc.top + rcWork.bottom - rc.bottom;
    }

    (*ppt2).x = (*ppt1).x - x;
    (*ppt2).y = (*ppt1).y - y;

    PatBlt(
        hDisplayDC,
        x,
        y,
        rc.right - rc.left - cxBorder,
        cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x,
        y + cyBorder,
        cxBorder,
        rc.bottom - rc.top - cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x + cxBorder,
        y + rc.bottom - rc.top,
        rc.right - rc.left - cxBorder,
        -cyBorder,
        PATINVERT,
    );
    PatBlt(
        hDisplayDC,
        x + rc.right - rc.left,
        y,
        -cxBorder,
        rc.bottom - rc.top - cyBorder,
        PATINVERT,
    );

    DeleteDC(hDisplayDC);
}

unsafe extern "C" fn C1_HitTest(ppt: *const POINT) -> INT {
    let mut iKey: INT = 0;
    iKey = C1K_OEM_3 as INT;
    while iKey < C1K_BACKSPACE as INT {
        if Imm32PtInRect(
            ppt,
            gptC1ButtonPos[iKey as usize].x,
            gptC1ButtonPos[iKey as usize].y,
            (24) as LONG,
            (28) as LONG,
        ) != 0
        {
            return iKey;
        }
        iKey += 1;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_BACKSPACE) as usize].x,
        gptC1ButtonPos[(C1K_BACKSPACE) as usize].y,
        (36) as LONG,
        (28) as LONG,
    ) != 0
    {
        return (C1K_BACKSPACE) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_TAB) as usize].x,
        gptC1ButtonPos[(C1K_TAB) as usize].y,
        (36) as LONG,
        (28) as LONG,
    ) != 0
    {
        return (C1K_TAB) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_CAPS) as usize].x,
        gptC1ButtonPos[(C1K_CAPS) as usize].y,
        (42) as LONG,
        (28) as LONG,
    ) != 0
    {
        return (C1K_CAPS) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_ENTER) as usize].x,
        gptC1ButtonPos[(C1K_ENTER) as usize].y,
        (42) as LONG,
        (28) as LONG,
    ) != 0
    {
        return (C1K_ENTER) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_SHIFT) as usize].x,
        gptC1ButtonPos[(C1K_SHIFT) as usize].y,
        (60) as LONG,
        (28) as LONG,
    ) != 0
    {
        return (C1K_SHIFT) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_ESCAPE) as usize].x,
        gptC1ButtonPos[(C1K_ESCAPE) as usize].y,
        (38) as LONG,
        (24) as LONG,
    ) != 0
    {
        return (C1K_ESCAPE) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_SPACE) as usize].x,
        gptC1ButtonPos[(C1K_SPACE) as usize].y,
        (172) as LONG,
        (24) as LONG,
    ) != 0
    {
        return (C1K_SPACE) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_INSERT) as usize].x,
        gptC1ButtonPos[(C1K_INSERT) as usize].y,
        (38) as LONG,
        (24) as LONG,
    ) != 0
    {
        return (C1K_INSERT) as INT;
    }
    if Imm32PtInRect(
        ppt,
        gptC1ButtonPos[(C1K_DELETE) as usize].x,
        gptC1ButtonPos[(C1K_DELETE) as usize].y,
        (38) as LONG,
        (24) as LONG,
    ) != 0
    {
        return (C1K_DELETE) as INT;
    }

    return -((1) as INT);
}

unsafe extern "C" fn C1_OnButtonDown(hWnd: HWND, pC1: PC1WINDOW) {
    let mut iPressedKey: INT;
    let mut hMemDC: HDC;
    let mut wch: WCHAR = 0xFF as WCHAR;
    let mut hbmOld: HGDIOBJ;
    let mut hDC: HDC;

    SetCapture(hWnd);

    iPressedKey = (*pC1).iPressedKey;
    if iPressedKey == -1 as INT {
        (*pC1).dwFlags |= FLAG_DRAGGING as DWORD;
        C1_DrawDragBorder(
            hWnd,
            core::ptr::addr_of_mut!((*pC1).pt1),
            core::ptr::addr_of_mut!((*pC1).pt2),
        );
        return;
    }

    if iPressedKey < C1K_BACKSPACE as INT {
        wch = (*(*pC1).Data.as_ptr().add(((*pC1).dwFlags & 1 as DWORD == 0) as usize))
            [iPressedKey as usize];
        if wch == 0 {
            MessageBeep(0xFFFFFFFF as UINT);
            (*pC1).iPressedKey = -1 as INT;
            return;
        }
    }

    if (iPressedKey != C1K_SHIFT as INT)
        || ((*pC1).dwFlags & FLAG_SHIFT_PRESSED as DWORD == 0)
    {
        hDC = GetDC(hWnd);
        hMemDC = CreateCompatibleDC(hDC);
        hbmOld = SelectObject(hMemDC, (*pC1).hbmKeyboard as HGDIOBJ);
        C1_InvertButton(hDC, (*pC1).iPressedKey);
        C1_InvertButton(hMemDC, (*pC1).iPressedKey);
        SelectObject(hMemDC, hbmOld);
        DeleteDC(hMemDC);
        ReleaseDC(hWnd, hDC);
    }

    (*pC1).dwFlags |= FLAG_PRESSED as DWORD;
}

unsafe extern "C" fn C1_OnSetCursor(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let mut hGlobal: HGLOBAL = core::ptr::null_mut();
    let mut pC1: PC1WINDOW = core::ptr::null_mut();
    let mut hCursor: HCURSOR = core::ptr::null_mut();
    let mut iKey: INT = 0;
    let mut pt1: POINT = POINT { x: 0, y: 0 };
    let mut pt2: POINT = POINT { x: 0, y: 0 };

    hGlobal = GetWindowLongPtrW(hWnd, 0 as i32) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return (FALSE) as BOOL;
    }

    if ((*pC1).dwFlags & (FLAG_DRAGGING as DWORD)) != 0 {
        hCursor = LoadCursorW(
            0 as HINSTANCE,
            (IDC_SIZEALL as usize) as LPCWSTR,
        );
        SetCursor(hCursor);
        GlobalUnlock(hGlobal);
        return (TRUE) as BOOL;
    }

    GetCursorPos(core::ptr::addr_of_mut!(pt1));
    pt2 = pt1;
    ScreenToClient(hWnd, core::ptr::addr_of_mut!(pt2));

    iKey = C1_HitTest(core::ptr::addr_of_mut!(pt2));
    if iKey == -(1 as INT) {
        hCursor = LoadCursorW(
            0 as HINSTANCE,
            (IDC_SIZEALL as usize) as LPCWSTR,
        );
    } else {
        hCursor = LoadCursorW(
            0 as HINSTANCE,
            (IDC_HAND as usize) as LPCWSTR,
        );
    }
    SetCursor(hCursor);

    if (HIWORD!(lParam as DWORD_PTR) as INT) == (WM_LBUTTONDOWN as INT) {
        (*pC1).pt1 = pt1;
        (*pC1).pt2 = pt2;
        (*pC1).iPressedKey = iKey;
        C1_OnButtonDown(hWnd, pC1);
    }

    GlobalUnlock(hGlobal);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn C1_OnMouseMove(
    mut hWnd: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> BOOL {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pC1: PC1WINDOW = ::core::ptr::null_mut::<C1WINDOW>();
    let mut hMemDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut dwFlags: DWORD = 0;
    let mut iPressedKey: INT = 0;
    let mut pt: POINT = POINT { x: 0, y: 0 };
    let mut hbmOld: HGDIOBJ = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut hDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut iKey: INT = 0;

    hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return (FALSE) as BOOL;
    }

    if ((*pC1).dwFlags & (FLAG_DRAGGING as DWORD)) != 0 {
        C1_DrawDragBorder(hWnd, &raw mut (*pC1).pt1, &raw mut (*pC1).pt2);
        GetCursorPos(&raw mut (*pC1).pt1);
        C1_DrawDragBorder(hWnd, &raw mut (*pC1).pt1, &raw mut (*pC1).pt2);
        GlobalUnlock(hGlobal);
        return (TRUE) as BOOL;
    }

    if (*pC1).iPressedKey != -(1 as ::core::ffi::c_int) {
        GetCursorPos(&raw mut pt);
        ScreenToClient(hWnd, &raw mut pt);
        iKey = C1_HitTest(&raw mut pt);

        hDC = GetDC(hWnd);
        hMemDC = CreateCompatibleDC(hDC);
        hbmOld = SelectObject(hMemDC, (*pC1).hbmKeyboard as HGDIOBJ);
        dwFlags = (*pC1).dwFlags;

        iPressedKey = (*pC1).iPressedKey;
        if (((dwFlags & (FLAG_PRESSED as DWORD)) != 0) as ::core::ffi::c_int)
            == ((iKey != iPressedKey) as ::core::ffi::c_int)
        {
            if iPressedKey != (C1K_SHIFT as ::core::ffi::c_int)
                || (dwFlags & (FLAG_SHIFT_PRESSED as DWORD)) == 0
            {
                C1_InvertButton(hDC, iPressedKey);
                C1_InvertButton(hMemDC, (*pC1).iPressedKey);
            }

            (*pC1).dwFlags ^= FLAG_PRESSED as DWORD;
        }

        SelectObject(hMemDC, hbmOld);
        DeleteDC(hMemDC);
        ReleaseDC(hWnd, hDC);
    }

    GlobalUnlock(hGlobal);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn C1_OnButtonUp(mut hWnd: HWND, mut wParam: WPARAM, mut lParam: LPARAM) -> BOOL {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pC1: PC1WINDOW = ::core::ptr::null_mut::<C1WINDOW>();
    let mut ret: BOOL = FALSE as BOOL;
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut iKey: INT = 0;
    let mut hDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut hMemDC: HDC = ::core::ptr::null_mut::<HDC__>();
    let mut hbmOld: HGDIOBJ = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut hIMC: HIMC = ::core::ptr::null_mut::<HIMC__>();
    let mut hwndOwner: HWND = ::core::ptr::null_mut::<HWND__>();
    let mut pIC: LPINPUTCONTEXT = ::core::ptr::null_mut::<_tagINPUTCONTEXT>();
    hGlobal = GetWindowLongPtrW(hWnd, 0 as ::core::ffi::c_int) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return FALSE as BOOL;
    }
    ReleaseCapture();
    if (*pC1).dwFlags & FLAG_DRAGGING as DWORD != 0 {
        (*pC1).dwFlags &= !FLAG_DRAGGING as DWORD;
        C1_DrawDragBorder(hWnd, &raw mut (*pC1).pt1, &raw mut (*pC1).pt2);
        x = ((*pC1).pt1.x - (*pC1).pt2.x) as INT;
        y = ((*pC1).pt1.y - (*pC1).pt2.y) as INT;
        SetWindowPos(
            hWnd,
            0 as HWND,
            x as ::core::ffi::c_int,
            y as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            0x15 as UINT,
        );
        ret = TRUE as BOOL;
        hwndOwner = GetWindow(hWnd, GW_OWNER as UINT) as HWND;
        hIMC = GetWindowLongPtrW(hwndOwner, 0 as ::core::ffi::c_int) as HIMC;
        if !hIMC.is_null() {
            pIC = ImmLockIMC(hIMC) as LPINPUTCONTEXT;
            if !pIC.is_null() {
                (*pIC).fdwInit |= INIT_SOFTKBDPOS as DWORD;
                (*pIC).ptSoftKbdPos.x = x as LONG;
                (*pIC).ptSoftKbdPos.y = y as LONG;
                ImmUnlockIMC(hIMC);
            }
        }
        GlobalUnlock(hGlobal);
        return ret;
    }
    iKey = (*pC1).iPressedKey;
    if iKey == -(1 as ::core::ffi::c_int) {
        return FALSE as BOOL;
    }
    if (*pC1).dwFlags & FLAG_PRESSED as DWORD == 0 {
        (*pC1).iPressedKey = -(1 as INT);
        GlobalUnlock(hGlobal);
        return ret;
    }
    if iKey == C1K_SHIFT as INT {
        if (*pC1).dwFlags & FLAG_SHIFT_PRESSED as DWORD == 0 {
            (*pC1).dwFlags |= FLAG_SHIFT_PRESSED as DWORD;
            (*pC1).dwFlags &= !FLAG_PRESSED as DWORD;
            (*pC1).iPressedKey = -(1 as INT);
            GlobalUnlock(hGlobal);
            return ret;
        }
    } else if iKey < C1K_BACKSPACE as INT
        && (*pC1).dwFlags & FLAG_SHIFT_PRESSED as DWORD != 0
    {
        let mut iVK: INT = gC1K2VK[(*pC1).iPressedKey as usize] as INT;
        keybd_event(
            VK_SHIFT as BYTE,
            guScanCode[C1K_SHIFT as usize] as BYTE,
            0 as DWORD,
            0 as ULONG_PTR,
        );
        keybd_event(
            iVK as BYTE,
            guScanCode[iVK as BYTE as usize] as BYTE,
            0 as DWORD,
            0 as ULONG_PTR,
        );
        keybd_event(
            iVK as BYTE,
            guScanCode[iVK as BYTE as usize] as BYTE,
            KEYEVENTF_KEYUP as DWORD,
            0 as ULONG_PTR,
        );
        keybd_event(
            VK_SHIFT as BYTE,
            guScanCode[C1K_SHIFT as usize] as BYTE,
            KEYEVENTF_KEYUP as DWORD,
            0 as ULONG_PTR,
        );
    } else {
        let mut iVK_0: INT = gC1K2VK[iKey as usize] as INT;
        keybd_event(
            iVK_0 as BYTE,
            guScanCode[iVK_0 as usize] as BYTE,
            0 as DWORD,
            0 as ULONG_PTR,
        );
        keybd_event(
            iVK_0 as BYTE,
            guScanCode[iVK_0 as usize] as BYTE,
            KEYEVENTF_KEYUP as DWORD,
            0 as ULONG_PTR,
        );
    }
    ret = TRUE as BOOL;
    hDC = GetDC(hWnd) as HDC;
    hMemDC = CreateCompatibleDC(hDC) as HDC;
    hbmOld = SelectObject(hMemDC, (*pC1).hbmKeyboard as HGDIOBJ) as HGDIOBJ;
    C1_InvertButton(hDC, (*pC1).iPressedKey);
    C1_InvertButton(hMemDC, (*pC1).iPressedKey);
    if (*pC1).iPressedKey < C1K_BACKSPACE as INT
        && (*pC1).dwFlags & FLAG_SHIFT_PRESSED as DWORD != 0
    {
        C1_InvertButton(hDC, C1K_SHIFT as INT);
        C1_InvertButton(hMemDC, C1K_SHIFT as INT);
    }
    if (*pC1).iPressedKey < C1K_BACKSPACE as INT
        || (*pC1).iPressedKey == C1K_SHIFT as INT
    {
        (*pC1).dwFlags &= !FLAG_SHIFT_PRESSED as DWORD;
    }
    SelectObject(hMemDC, hbmOld);
    DeleteDC(hMemDC);
    ReleaseDC(hWnd, hDC);
    (*pC1).dwFlags &= !FLAG_PRESSED as DWORD;
    (*pC1).iPressedKey = -(1 as INT);
    GlobalUnlock(hGlobal);
    return ret;
}

unsafe extern "C" fn C1_OnDestroy(mut hWnd: HWND) {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pC1: PC1WINDOW = ::core::ptr::null_mut::<C1WINDOW>();
    let mut hwndOwner: HWND = ::core::ptr::null_mut::<HWND__>();
    hGlobal = GetWindowLongPtrW(hWnd, 0 as i32) as HGLOBAL;
    pC1 = GlobalLock(hGlobal) as PC1WINDOW;
    if hGlobal.is_null() || pC1.is_null() {
        return;
    }
    if (*pC1).dwFlags & FLAG_DRAGGING as DWORD != 0 {
        C1_DrawDragBorder(hWnd, &raw mut (*pC1).pt1, &raw mut (*pC1).pt2);
    }
    DeleteObject((*pC1).hbmKeyboard as HGDIOBJ);
    GlobalUnlock(hGlobal);
    GlobalFree(hGlobal);
    hwndOwner = GetWindow(hWnd, GW_OWNER as UINT);
    if !hwndOwner.is_null() {
        SendMessageW(
            hwndOwner,
            WM_IME_NOTIFY as UINT,
            IMN_SOFTKBDDESTROYED as WPARAM,
            0 as LPARAM,
        );
    }
}

unsafe extern "C" fn C1_OnImeControl(
    mut hWnd: HWND,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut hGlobal: HGLOBAL = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pC1: PC1WINDOW = ::core::ptr::null_mut::<C1WINDOW>();
    let mut lf: LOGFONTW = tagLOGFONTW {
        lfHeight: 0,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: 0,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: 0,
        lfOutPrecision: 0,
        lfClipPrecision: 0,
        lfQuality: 0,
        lfPitchAndFamily: 0,
        lfFaceName: [0; 32],
    };
    let mut rc: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut ret: LRESULT = 0 as LRESULT;
    let mut hDC: HDC = ::core::ptr::null_mut::<HDC__>();
    match wParam as u32 {
        IMC_GETSOFTKBDFONT => {
            hDC = GetDC(hWnd);
            GetObjectW(
                GetStockObject((DEFAULT_GUI_FONT as i32)) as HANDLE,
                core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(lf) as LPVOID,
            );
            ReleaseDC(hWnd, hDC);
            *(lParam as LPLOGFONTW) = lf;
        }
        IMC_SETSOFTKBDFONT => {
            let mut plf: LPLOGFONTW = lParam as LPLOGFONTW;
            let mut lf_0: LOGFONTW = tagLOGFONTW {
                lfHeight: 0,
                lfWidth: 0,
                lfEscapement: 0,
                lfOrientation: 0,
                lfWeight: 0,
                lfItalic: 0,
                lfUnderline: 0,
                lfStrikeOut: 0,
                lfCharSet: 0,
                lfOutPrecision: 0,
                lfClipPrecision: 0,
                lfQuality: 0,
                lfPitchAndFamily: 0,
                lfFaceName: [0; 32],
            };
            GetObjectW(
                GetStockObject((DEFAULT_GUI_FONT as i32)) as HANDLE,
                core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
                core::ptr::addr_of_mut!(lf_0) as LPVOID,
            );
            if lf_0.lfCharSet == (*plf).lfCharSet {
                return 0 as LRESULT;
            }

            hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
            pC1 = GlobalLock(hGlobal) as PC1WINDOW;
            if hGlobal.is_null() || pC1.is_null() {
                return 1 as LRESULT;
            }

            (*pC1).CharSet = (*plf).lfCharSet as DWORD;
            GlobalUnlock(hGlobal);
        }
        IMC_GETSOFTKBDPOS => {
            let mut rc_0: RECT = tagRECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            GetWindowRect(hWnd, core::ptr::addr_of_mut!(rc_0));
            return MAKELRESULT!(rc_0.left, rc_0.top);
        }
        IMC_SETSOFTKBDPOS => {
            let mut pt: POINT = tagPOINT { x: 0, y: 0 };
            pt.x = ((*(core::ptr::addr_of_mut!(lParam) as *mut DWORD)) as DWORD_PTR
                & 0xffff as DWORD_PTR) as WORD as LONG;
            pt.y = (((*(core::ptr::addr_of_mut!(lParam) as *mut DWORD)) as DWORD_PTR >> 16)
                & 0xffff as DWORD_PTR) as WORD as LONG;
            SetWindowPos(
                hWnd,
                ::core::ptr::null_mut::<HWND__>(),
                pt.x as ::core::ffi::c_int,
                pt.y as ::core::ffi::c_int,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOSIZE) as UINT,
            );
        }
        IMC_GETSOFTKBDSUBTYPE | IMC_SETSOFTKBDSUBTYPE => {
            hGlobal = GetWindowLongPtrW(hWnd, 0) as HGLOBAL;
            pC1 = GlobalLock(hGlobal) as PC1WINDOW;
            if hGlobal.is_null() || pC1.is_null() {
                return -((1) as LRESULT);
            }
            ret = (*pC1).SubType as LRESULT;
            if wParam == IMC_SETSOFTKBDSUBTYPE as WPARAM {
                (*pC1).SubType = lParam;
            }
            GlobalUnlock(hGlobal);
        }
        IMC_SETSOFTKBDDATA => {
            if C1_SetData(hWnd, lParam as *mut SOFTKBDDATA) != 0 {
                return -((1) as LRESULT);
            }

            InvalidateRect(hWnd, core::ptr::null(), 0);
        }
        _ => {}
    }
    return ret;
}

unsafe extern "system" fn C1_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMsg {
        WM_CREATE => {
            return C1_OnCreate(hWnd) as LRESULT;
        }
        WM_DESTROY => {
            C1_OnDestroy(hWnd);
        }
        WM_SETCURSOR => {
            if C1_OnSetCursor(hWnd, lParam) == 0 {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_MOUSEMOVE => {
            if C1_OnMouseMove(hWnd, wParam, lParam) == 0 {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_LBUTTONUP => {
            if C1_OnButtonUp(hWnd, wParam, lParam) == 0 {
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = core::mem::zeroed();
            let mut hDC: HDC = BeginPaint(hWnd, core::ptr::addr_of_mut!(ps));
            C1_OnDraw(hDC, hWnd);
            EndPaint(hWnd, core::ptr::addr_of_mut!(ps));
        }
        WM_IME_CONTROL => {
            return C1_OnImeControl(hWnd, wParam, lParam);
        }
        WM_MOUSEACTIVATE => {
            return MA_NOACTIVATE as LRESULT;
        }
        _ => {
            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
    }
    return 0 as LRESULT;
}

unsafe extern "C" fn Imm32RegisterSoftKeyboard(mut uType: UINT) -> BOOL {
    let mut wcx: WNDCLASSEXW = core::mem::zeroed();
    let mut pszClass: LPCWSTR = if uType == SOFTKEYBOARD_TYPE_T1 as UINT {
        (T1_CLASSNAMEW).as_ptr() as LPCWSTR
    } else {
        (C1_CLASSNAMEW).as_ptr() as LPCWSTR
    };
    if GetClassInfoExW(ghImm32Inst as HINSTANCE, pszClass, &raw mut wcx) != 0 {
        return TRUE as BOOL;
    }

    ZeroMemory!(
        &raw mut wcx as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<WNDCLASSEXW>() as size_t,
    );
    wcx.cbSize = ::core::mem::size_of::<WNDCLASSEXW>() as UINT;
    wcx.style = CS_IME as UINT;
    wcx.cbWndExtra = ::core::mem::size_of::<PT1WINDOW>() as ::core::ffi::c_int;
    wcx.hIcon = LoadIconW(
        ::core::ptr::null_mut::<HINSTANCE__>(),
        (IDI_APPLICATION as usize) as LPCWSTR,
    );
    wcx.hInstance = ghImm32Inst as HINSTANCE;
    wcx.hCursor = LoadCursorW(
        ::core::ptr::null_mut::<HINSTANCE__>(),
        (IDC_SIZEALL as usize) as LPCWSTR,
    );
    wcx.lpszClassName = pszClass;

    if uType == SOFTKEYBOARD_TYPE_T1 as UINT {
        wcx.lpfnWndProc = Some(T1_WindowProc);
        wcx.hbrBackground = GetStockObject(NULL_BRUSH as ::core::ffi::c_int) as HBRUSH;
    } else {
        wcx.lpfnWndProc = Some(C1_WindowProc);
        wcx.hbrBackground = GetStockObject(LTGRAY_BRUSH as ::core::ffi::c_int) as HBRUSH;
    }

    return (RegisterClassExW(&raw mut wcx) != 0) as BOOL;
}

unsafe extern "C" fn Imm32GetSoftKeyboardDimension(
    mut uType: UINT,
    mut pcx: LPINT,
    mut pcy: LPINT,
) {
    if uType == SOFTKEYBOARD_TYPE_T1 {
        let mut tm: TEXTMETRICW = core::mem::zeroed::<TEXTMETRICW>();
        T1_GetTextMetric(core::ptr::addr_of_mut!(tm));
        *pcx = 15 * tm.tmMaxCharWidth + 2 * gptRaiseEdge.x + 139;
        *pcy = 5 * tm.tmHeight + 2 * gptRaiseEdge.y + 58;
    } else {
        let mut cxEdge: INT = GetSystemMetrics((SM_CXEDGE) as i32);
        let mut cyEdge: INT = GetSystemMetrics((SM_CXEDGE) as i32);
        *pcx = 2 * (GetSystemMetrics((SM_CXBORDER) as i32) + cxEdge) + 348;
        *pcy = 2 * (GetSystemMetrics((SM_CYBORDER) as i32) + cyEdge) + 136;
    }
}

#[no_mangle]
pub unsafe extern "system" fn ImmCreateSoftKeyboard(
    uType: UINT,
    hwndParent: HWND,
    x: INT,
    y: INT,
) -> HWND {
    let mut hKL: HKL = core::ptr::null_mut();
    let mut pImeDpi: PIMEDPI = core::ptr::null_mut();
    let mut iVK: UINT = 0;
    let mut xSoftKBD: INT = 0;
    let mut ySoftKBD: INT = 0;
    let mut cxSoftKBD: INT = 0;
    let mut cySoftKBD: INT = 0;
    let mut cxEdge: INT = 0;
    let mut cyEdge: INT = 0;
    let mut hwndSoftKBD: HWND = core::ptr::null_mut();
    let mut Style: DWORD = 0;
    let mut ExStyle: DWORD = 0;
    let mut UICaps: DWORD = 0;
    let mut pszClass: LPCWSTR = core::ptr::null();
    let mut rcWorkArea: RECT = core::mem::zeroed();

    if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_TRACE as u32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/softkbd.c\0"
                .as_ptr() as *const i8,
            b"ImmCreateSoftKeyboard\0".as_ptr() as *const i8,
            2124i32,
            b"(%u, %p, %d, %d)\n\0".as_ptr() as *const i8,
            uType,
            hwndParent,
            x,
            y,
        );
    }

    if (uType != SOFTKEYBOARD_TYPE_T1) && (uType != SOFTKEYBOARD_TYPE_C1) {
        if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/softkbd.c\0"
                    .as_ptr() as *const i8,
                b"ImmCreateSoftKeyboard\0".as_ptr() as *const i8,
                2128i32,
                b"uType: %u\n\0".as_ptr() as *const i8,
                uType,
            );
        }
        return core::ptr::null_mut();
    }

    /* Check IME */
    hKL = GetKeyboardLayout(0);
    pImeDpi = ImmLockImeDpi(hKL);
    if IS_NULL_UNEXPECTEDLY!(pImeDpi) != false {
        return core::ptr::null_mut(); /* No IME */
    }

    UICaps = (*pImeDpi).ImeInfo.fdwUICaps;
    ImmUnlockImeDpi(pImeDpi);

    /* Check IME capability */
    if (UICaps & UI_CAP_SOFTKBD) == 0 {
        if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/user/imm32/softkbd.c\0"
                    .as_ptr() as *const i8,
                b"ImmCreateSoftKeyboard\0".as_ptr() as *const i8,
                2144i32,
                b"UICaps: 0x%X\n\0".as_ptr() as *const i8,
                UICaps,
            );
        }
        return core::ptr::null_mut(); /* No capability for soft keyboard */
    }

    /* Want metrics? */
    if g_bWantSoftKBDMetrics != 0 {
        iVK = 0;
        while iVK < 0xFF {
            guScanCode[iVK as usize] = MapVirtualKeyW(iVK, 0);
            iVK = iVK.wrapping_add(1);
        }

        cxEdge = GetSystemMetrics((SM_CXEDGE) as i32);
        cyEdge = GetSystemMetrics((SM_CYEDGE) as i32);
        gptRaiseEdge.x = (GetSystemMetrics((SM_CXBORDER) as i32) + cxEdge) as LONG;
        gptRaiseEdge.y = (GetSystemMetrics((SM_CYBORDER) as i32) + cyEdge) as LONG;

        g_bWantSoftKBDMetrics = (FALSE) as i32;
    }

    if Imm32GetNearestWorkArea(hwndParent, &mut rcWorkArea) == 0 {
        return core::ptr::null_mut();
    }

    /* Register the window class */
    if Imm32RegisterSoftKeyboard(uType) == 0 {
        if ((*__wine_dbch___default).flags as u32) & ((1u32) << (__WINE_DBCL_ERR as u32)) != 0 {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_ERR;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"win32ss/user/imm32/softkbd.c\0"
                    .as_ptr() as *const i8,
                b"ImmCreateSoftKeyboard\0".as_ptr() as *const i8,
                2170i32,
                b"\n\0".as_ptr() as *const i8,
            );
        }
        return core::ptr::null_mut();
    }

    /* Calculate keyboard size */
    Imm32GetSoftKeyboardDimension(uType, &mut cxSoftKBD, &mut cySoftKBD);

    /* Adjust keyboard position */
    xSoftKBD = Imm32Clamp(
        x,
        rcWorkArea.left as INT,
        (rcWorkArea.right - cxSoftKBD as LONG) as INT,
    );
    ySoftKBD = Imm32Clamp(
        y,
        rcWorkArea.top as INT,
        (rcWorkArea.bottom - cySoftKBD as LONG) as INT,
    );

    /* Create soft keyboard window */
    if uType == SOFTKEYBOARD_TYPE_T1 {
        Style = (WS_POPUP | WS_DISABLED);
        ExStyle = 0;
        pszClass = (T1_CLASSNAMEW).as_ptr();
    } else {
        Style = (WS_POPUP | WS_DISABLED | WS_BORDER);
        ExStyle = (WS_EX_WINDOWEDGE | WS_EX_DLGMODALFRAME);
        pszClass = (C1_CLASSNAMEW).as_ptr();
    }
    hwndSoftKBD = CreateWindowExW(
        ExStyle,
        pszClass,
        core::ptr::null(),
        Style,
        xSoftKBD,
        ySoftKBD,
        cxSoftKBD,
        cySoftKBD,
        hwndParent,
        core::ptr::null_mut(),
        ghImm32Inst,
        core::ptr::null_mut(),
    );
    /* Initial is hidden */
    ShowWindow(hwndSoftKBD, (SW_HIDE) as i32);
    UpdateWindow(hwndSoftKBD);

    return hwndSoftKBD;
}

#[no_mangle]
pub unsafe extern "system" fn ImmShowSoftKeyboard(hwndSoftKBD: HWND, nCmdShow: INT) -> BOOL {
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/user/imm32/softkbd.c\0"
                as *const u8 as *const i8,
            b"ImmShowSoftKeyboard\0" as *const u8 as *const i8,
            2214i32,
            b"(%p, %d)\n\0" as *const u8 as *const i8,
            hwndSoftKBD,
            nCmdShow,
        );
    }
    if (nCmdShow) as u32 != SW_HIDE && (nCmdShow) as u32 != SW_SHOWNOACTIVATE {
        if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_WARN as i32 != 0 {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/user/imm32/softkbd.c\0"
                    as *const u8 as *const i8,
                b"ImmShowSoftKeyboard\0" as *const u8 as *const i8,
                2217i32,
                b"nCmdShow %d is unexpected\n\0" as *const u8 as *const i8,
                nCmdShow,
            );
        }
    }
    return (!hwndSoftKBD.is_null() && ShowWindow(hwndSoftKBD, nCmdShow) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImmDestroySoftKeyboard(hwndSoftKBD: HWND) -> BOOL {
    // TRACE("(%p)\n", hwndSoftKBD);
    DestroyWindow(hwndSoftKBD)
}
