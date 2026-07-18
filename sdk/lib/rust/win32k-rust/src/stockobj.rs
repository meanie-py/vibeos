use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlGetDefaultCodePage};
use crate::brushbody::pbrushbody;
use crate::floatobj::FLOATOBJ_SetLong;

extern "C" {
    fn abs(arg1: i32) -> i32;
    fn wcscpy(dest: *mut u16, src: *const u16) -> *mut u16;
}

// PEN_AllocPenWithHandle is NTAPI (pen.h) — stdcall on i386.
extern "system" {
    fn PEN_AllocPenWithHandle() -> PBRUSH;
}

// TextIntCreateFontIndirect is FASTCALL (text.h).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn TextIntCreateFontIndirect(lplf: *mut LOGFONTW, hFont: *mut HFONT);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn TextIntCreateFontIndirect(lplf: *mut LOGFONTW, hFont: *mut HFONT);
}

// RGB / PEN_UnlockPen are macros (wingdi.h / pen.h) — inline them.
#[inline(always)]
fn RGB(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}
#[inline(always)]
unsafe fn PEN_UnlockPen(p: PBRUSH) {
    GDIOBJ_vUnlockObject(p as POBJ);
}

// SERVERINFO embeds an anonymous PERUSERSERVERINFO (ntuser.h:1072). bindgen
// flattened it into `tagSERVERINFO.__bindgen_padding_0`, so argbSystem/
// ahbrSystem are unreachable on `*gpsi`. The padding field begins exactly at
// the PERUSERSERVERINFO offset, so its address re-typed gives the real view.
#[inline(always)]
unsafe fn peruser(psi: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*psi).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// ---- file-scope preamble (translated) ----
pub static SysColors: [COLORREF; 31] = [
    0x00D0C8D4, /* COLOR_SCROLLBAR  */
    0x00A56E3A, /* COLOR_BACKGROUND  */
    0x006A240A, /* COLOR_ACTIVECAPTION  */
    0x00808080, /* COLOR_INACTIVECAPTION  */
    0x00D0C8D4, /* COLOR_MENU  */
    0x00FFFFFF, /* COLOR_WINDOW  */
    0x00000000, /* COLOR_WINDOWFRAME  */
    0x00000000, /* COLOR_MENUTEXT  */
    0x00000000, /* COLOR_WINDOWTEXT  */
    0x00FFFFFF, /* COLOR_CAPTIONTEXT  */
    0x00D0C8D4, /* COLOR_ACTIVEBORDER  */
    0x00D0C8D4, /* COLOR_INACTIVEBORDER  */
    0x00808080, /* COLOR_APPWORKSPACE  */
    0x006A240A, /* COLOR_HIGHLIGHT  */
    0x00FFFFFF, /* COLOR_HIGHLIGHTTEXT  */
    0x00D0C8D4, /* COLOR_BTNFACE  */
    0x00808080, /* COLOR_BTNSHADOW  */
    0x00808080, /* COLOR_GRAYTEXT  */
    0x00000000, /* COLOR_BTNTEXT  */
    0x00D0C8D4, /* COLOR_INACTIVECAPTIONTEXT  */
    0x00FFFFFF, /* COLOR_BTNHIGHLIGHT  */
    0x00404040, /* COLOR_3DDKSHADOW  */
    0x00D0C8D4, /* COLOR_3DLIGHT  */
    0x00000000, /* COLOR_INFOTEXT  */
    0x00E1FFFF, /* COLOR_INFOBK  */
    0x00B5B5B5, /* COLOR_UNKNOWN  */
    0x00800000, /* COLOR_HOTLIGHT  */
    0x00F0CAA6, /* COLOR_GRADIENTACTIVECAPTION  */
    0x00C0C0C0, /* COLOR_GRADIENTINACTIVECAPTION  */
    0x00C56A31, /* COLOR_MENUHILIGHT  */
    0x00D8E9EC  /* COLOR_MENUBAR  */
];

#[no_mangle]
pub static mut hSystemBM: HDC = core::ptr::null_mut();

pub static WhitePen: LOGPEN = LOGPEN {
    lopnStyle: PS_SOLID,
    lopnWidth: POINTL { x: 0, y: 0 },
    lopnColor: 0x00FFFFFF,
};

pub static BlackPen: LOGPEN = LOGPEN {
    lopnStyle: PS_SOLID,
    lopnWidth: POINTL { x: 0, y: 0 },
    lopnColor: 0x00000000,
};

pub static NullPen: LOGPEN = LOGPEN {
    lopnStyle: PS_NULL,
    lopnWidth: POINTL { x: 0, y: 0 },
    lopnColor: 0,
};

pub static mut OEMFixedFont: LOGFONTW = LOGFONTW {
    lfHeight: 12,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_NORMAL) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (OEM_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_DEFAULT_PRECIS) as _,
    lfQuality: (DEFAULT_QUALITY) as _,
    lfPitchAndFamily: (FF_DONTCARE | FIXED_PITCH) as _,
    lfFaceName: [84u16, 101, 114, 109, 105, 110, 97, 108, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut AnsiFixedFont: LOGFONTW = LOGFONTW {
    lfHeight: 12,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_NORMAL) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (ANSI_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_STROKE_PRECIS) as _,
    lfQuality: (PROOF_QUALITY) as _,
    lfPitchAndFamily: (FF_DONTCARE | FIXED_PITCH) as _,
    lfFaceName: [67u16, 111, 117, 114, 105, 101, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut AnsiVarFont: LOGFONTW = LOGFONTW {
    lfHeight: 12,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_NORMAL) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (ANSI_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_STROKE_PRECIS) as _,
    lfQuality: (PROOF_QUALITY) as _,
    lfPitchAndFamily: (FF_DONTCARE | VARIABLE_PITCH) as _,
    lfFaceName: [77u16, 83, 32, 83, 97, 110, 115, 32, 83, 101, 114, 105, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

#[no_mangle]
pub static mut SystemFont: LOGFONTW = LOGFONTW {
    lfHeight: 16,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_BOLD) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (DEFAULT_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_DEFAULT_PRECIS) as _,
    lfQuality: (DEFAULT_QUALITY) as _,
    lfPitchAndFamily: (FF_DONTCARE | VARIABLE_PITCH) as _,
    lfFaceName: [83u16, 121, 115, 116, 101, 109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut DeviceDefaultFont: LOGFONTW = LOGFONTW {
    lfHeight: 16,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_BOLD) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (ANSI_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_DEFAULT_PRECIS) as _,
    lfQuality: (DEFAULT_QUALITY) as _,
    lfPitchAndFamily: (FF_SWISS | VARIABLE_PITCH) as _,
    lfFaceName: [83u16, 121, 115, 116, 101, 109, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut SystemFixedFont: LOGFONTW = LOGFONTW {
    lfHeight: 15,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_NORMAL) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (DEFAULT_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_DEFAULT_PRECIS) as _,
    lfQuality: (DEFAULT_QUALITY) as _,
    lfPitchAndFamily: (FF_DONTCARE | FIXED_PITCH) as _,
    lfFaceName: [70u16, 105, 120, 101, 100, 115, 121, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

pub static mut DefaultGuiFont: LOGFONTW = LOGFONTW {
    lfHeight: -11i32 as i32,
    lfWidth: 0,
    lfEscapement: 0,
    lfOrientation: 0,
    lfWeight: (FW_NORMAL) as _,
    lfItalic: 0u8,
    lfUnderline: 0u8,
    lfStrikeOut: 0u8,
    lfCharSet: (DEFAULT_CHARSET) as _,
    lfOutPrecision: (OUT_DEFAULT_PRECIS) as _,
    lfClipPrecision: (CLIP_DEFAULT_PRECIS) as _,
    lfQuality: (PROOF_QUALITY) as _,
    lfPitchAndFamily: (DEFAULT_PITCH | FF_DONTCARE) as _,
    lfFaceName: [77u16, 83, 32, 83, 104, 101, 108, 108, 32, 68, 108, 103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

#[no_mangle]
pub static mut StockObjects: [HGDIOBJ; NB_STOCK_OBJECTS as usize] = [core::ptr::null_mut(); NB_STOCK_OBJECTS as usize];

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCreateStockPen(
    dwPenStyle: u32,
    dwWidth: u32,
    ulBrushStyle: u32,
    ulColor: u32,
) -> HPEN {
    let mut dwPenStyle = dwPenStyle;
    let mut dwWidth = dwWidth;
    
    let pbrushPen = PEN_AllocPenWithHandle();
    if pbrushPen.is_null() {
        return core::ptr::null_mut();
    }

    if (dwPenStyle & PS_STYLE_MASK) == PS_NULL {
        dwWidth = 1;
    }

    (*pbrushbody(pbrushPen)).iHatch = 0;
    (*pbrushbody(pbrushPen)).lWidth = abs(dwWidth as i32);
    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pbrushbody(pbrushPen)).eWidth), (*pbrushbody(pbrushPen)).lWidth);
    (*pbrushbody(pbrushPen)).ulPenStyle = dwPenStyle;
    (*pbrushbody(pbrushPen)).BrushAttr.lbColor = ulColor;
    (*pbrushbody(pbrushPen)).iBrushStyle = ulBrushStyle as INT;
    (*pbrushbody(pbrushPen)).hbmClient = core::ptr::null_mut();
    (*pbrushbody(pbrushPen)).dwStyleCount = 0;
    (*pbrushbody(pbrushPen)).pStyle = core::ptr::null_mut();
    (*pbrushbody(pbrushPen)).flAttrs = BR_IS_OLDSTYLEPEN;

    match dwPenStyle & PS_STYLE_MASK {
        PS_NULL => {
            (*pbrushbody(pbrushPen)).flAttrs |= BR_IS_NULL;
        }
        PS_SOLID => {
            (*pbrushbody(pbrushPen)).flAttrs |= BR_IS_SOLID;
        }
        _ => {}
    }
    
    let hPen = (*pbrushPen).BaseObject.hHmgr as HPEN;
    PEN_UnlockPen(pbrushPen);
    hPen
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCreateStockPen(
    dwPenStyle: u32,
    dwWidth: u32,
    ulBrushStyle: u32,
    ulColor: u32,
) -> HPEN {
    let mut dwPenStyle = dwPenStyle;
    let mut dwWidth = dwWidth;
    
    let pbrushPen = PEN_AllocPenWithHandle();
    if pbrushPen.is_null() {
        return core::ptr::null_mut();
    }

    if (dwPenStyle & PS_STYLE_MASK) == PS_NULL {
        dwWidth = 1;
    }

    (*pbrushbody(pbrushPen)).iHatch = 0;
    (*pbrushbody(pbrushPen)).lWidth = abs(dwWidth as i32);
    FLOATOBJ_SetLong(core::ptr::addr_of_mut!((*pbrushbody(pbrushPen)).eWidth), (*pbrushbody(pbrushPen)).lWidth);
    (*pbrushbody(pbrushPen)).ulPenStyle = dwPenStyle;
    (*pbrushbody(pbrushPen)).BrushAttr.lbColor = ulColor;
    (*pbrushbody(pbrushPen)).iBrushStyle = ulBrushStyle as INT;
    (*pbrushbody(pbrushPen)).hbmClient = core::ptr::null_mut();
    (*pbrushbody(pbrushPen)).dwStyleCount = 0;
    (*pbrushbody(pbrushPen)).pStyle = core::ptr::null_mut();
    (*pbrushbody(pbrushPen)).flAttrs = BR_IS_OLDSTYLEPEN;

    match dwPenStyle & PS_STYLE_MASK {
        PS_NULL => {
            (*pbrushbody(pbrushPen)).flAttrs |= BR_IS_NULL;
        }
        PS_SOLID => {
            (*pbrushbody(pbrushPen)).flAttrs |= BR_IS_SOLID;
        }
        _ => {}
    }
    
    let hPen = (*pbrushPen).BaseObject.hHmgr as HPEN;
    PEN_UnlockPen(pbrushPen);
    hPen
}

unsafe fn CreateStockFonts() {
    let mut ActiveCodePage: u16 = 0;
    let mut OemCodePage: u16 = 0;
    let bActiveCharSet: u8;
    let mut bOemCharSet: u8;
    let bIsCJK: u8;
    let SimSun: [u16; 3] = [0x5B8B, 0x4F53, 0];
    let MingLiU: [u16; 4] = [0x7D30, 0x660E, 0x9AD4, 0];
    let Batang: [u16; 3] = [0xBC14, 0xD0D5, 0];

    RtlGetDefaultCodePage(core::ptr::addr_of_mut!(ActiveCodePage), core::ptr::addr_of_mut!(OemCodePage));
    bActiveCharSet = IntCharSetFromCodePage(ActiveCodePage as UINT);
    bOemCharSet = IntCharSetFromCodePage(OemCodePage as UINT);

    if bOemCharSet == DEFAULT_CHARSET as u8 {
        bOemCharSet = OEM_CHARSET as u8;
    }

    match ActiveCodePage {
        936 => {
            bIsCJK = 1u8;
            wcscpy((*(&raw mut DefaultGuiFont)).lfFaceName.as_mut_ptr(), SimSun.as_ptr());
        }

        950 => {
            bIsCJK = 1u8;
            wcscpy((*(&raw mut DefaultGuiFont)).lfFaceName.as_mut_ptr(), MingLiU.as_ptr());
        }

        932 => {
            bIsCJK = 1u8;
            wcscpy((*(&raw mut DefaultGuiFont)).lfFaceName.as_mut_ptr(), [0x004D, 0x0053, 0x0020, 0x0055, 0x0049, 0x0020, 0x0047, 0x006F, 0x0074, 0x0068, 0x0069, 0x0063, 0].as_ptr());
        }

        949 | 1361 => {
            bIsCJK = 1u8;
            wcscpy((*(&raw mut DefaultGuiFont)).lfFaceName.as_mut_ptr(), Batang.as_ptr());
        }

        _ => {
            bIsCJK = 0u8;
            wcscpy((*(&raw mut DefaultGuiFont)).lfFaceName.as_mut_ptr(), [0x004D, 0x0053, 0x0020, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0044, 0x006C, 0x0067, 0].as_ptr());
        }
    }

    if bIsCJK != 0 {
        OEMFixedFont.lfHeight = 18;
        OEMFixedFont.lfPitchAndFamily = (FF_DONTCARE | FIXED_PITCH) as _;
        SystemFont.lfHeight = 18;
        SystemFont.lfPitchAndFamily = (FF_DONTCARE | VARIABLE_PITCH) as _;
        DeviceDefaultFont.lfHeight = 18;
        DeviceDefaultFont.lfPitchAndFamily = (FF_DONTCARE | VARIABLE_PITCH) as _;
        SystemFixedFont.lfHeight = 18;
        SystemFixedFont.lfPitchAndFamily = (FF_DONTCARE | FIXED_PITCH) as _;
        DefaultGuiFont.lfHeight = -12;
    } else {
        OEMFixedFont.lfHeight = 12;
        OEMFixedFont.lfPitchAndFamily = (FF_MODERN | FIXED_PITCH) as _;
        SystemFont.lfHeight = 16;
        SystemFont.lfPitchAndFamily = (FF_SWISS | VARIABLE_PITCH) as _;
        DeviceDefaultFont.lfHeight = 16;
        DeviceDefaultFont.lfPitchAndFamily = (FF_SWISS | VARIABLE_PITCH) as _;
        if bActiveCharSet == RUSSIAN_CHARSET as u8 {
            SystemFixedFont.lfHeight = 16;
            SystemFixedFont.lfPitchAndFamily = (FF_SWISS | FIXED_PITCH) as _;
        } else {
            SystemFixedFont.lfHeight = 15;
            SystemFixedFont.lfPitchAndFamily = (FF_MODERN | FIXED_PITCH) as _;
        }
        DefaultGuiFont.lfHeight = -11;
    }

    OEMFixedFont.lfCharSet = bOemCharSet;
    SystemFont.lfCharSet = bActiveCharSet;
    DeviceDefaultFont.lfCharSet = bActiveCharSet;
    SystemFixedFont.lfCharSet = bActiveCharSet;
    DefaultGuiFont.lfCharSet = bActiveCharSet;

    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(OEMFixedFont), core::ptr::addr_of_mut!(StockObjects[OEM_FIXED_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(AnsiFixedFont), core::ptr::addr_of_mut!(StockObjects[ANSI_FIXED_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(AnsiVarFont), core::ptr::addr_of_mut!(StockObjects[ANSI_VAR_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(SystemFont), core::ptr::addr_of_mut!(StockObjects[SYSTEM_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(DeviceDefaultFont), core::ptr::addr_of_mut!(StockObjects[DEVICE_DEFAULT_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(SystemFixedFont), core::ptr::addr_of_mut!(StockObjects[SYSTEM_FIXED_FONT as usize]) as *mut HFONT);
    TextIntCreateFontIndirect(core::ptr::addr_of_mut!(DefaultGuiFont), core::ptr::addr_of_mut!(StockObjects[DEFAULT_GUI_FONT as usize]) as *mut HFONT);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CreateStockObjects() {
    let mut Object: u32;

    StockObjects[WHITE_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(255, 255, 255))) as HGDIOBJ;
    StockObjects[DC_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(255, 255, 255))) as HGDIOBJ;
    StockObjects[LTGRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(192, 192, 192))) as HGDIOBJ;
    StockObjects[GRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(128, 128, 128))) as HGDIOBJ;
    StockObjects[DKGRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(64, 64, 64))) as HGDIOBJ;
    StockObjects[BLACK_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(0, 0, 0))) as HGDIOBJ;
    StockObjects[NULL_BRUSH as usize] = (IntGdiCreateNullBrush()) as HGDIOBJ;

    StockObjects[WHITE_PEN as usize] = (IntCreateStockPen(WhitePen.lopnStyle, WhitePen.lopnWidth.x as u32, BS_SOLID, WhitePen.lopnColor)) as HGDIOBJ;
    StockObjects[BLACK_PEN as usize] = (IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor)) as HGDIOBJ;
    StockObjects[DC_PEN as usize] = (IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor)) as HGDIOBJ;
    StockObjects[NULL_PEN as usize] = (IntCreateStockPen(NullPen.lopnStyle, NullPen.lopnWidth.x as u32, BS_SOLID, NullPen.lopnColor)) as HGDIOBJ;

    StockObjects[20] = (core::ptr::null_mut()) as HGDIOBJ;
    StockObjects[DEFAULT_BITMAP as usize] = (GreCreateBitmap(1, 1, 1, 1, core::ptr::null_mut())) as HGDIOBJ;

    CreateStockFonts();

    StockObjects[DEFAULT_PALETTE as usize] = ((*gppalDefault).BaseObject.hHmgr as HGDIOBJ) as HGDIOBJ;

    Object = 0;
    while Object < NB_STOCK_OBJECTS {
        if !StockObjects[Object as usize].is_null() {
            GDIOBJ_ConvertToStockObj(core::ptr::addr_of_mut!(StockObjects[Object as usize]));
        }
        Object += 1;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CreateStockObjects() {
    let mut Object: u32;

    StockObjects[WHITE_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(255, 255, 255))) as HGDIOBJ;
    StockObjects[DC_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(255, 255, 255))) as HGDIOBJ;
    StockObjects[LTGRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(192, 192, 192))) as HGDIOBJ;
    StockObjects[GRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(128, 128, 128))) as HGDIOBJ;
    StockObjects[DKGRAY_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(64, 64, 64))) as HGDIOBJ;
    StockObjects[BLACK_BRUSH as usize] = (IntGdiCreateSolidBrush(RGB(0, 0, 0))) as HGDIOBJ;
    StockObjects[NULL_BRUSH as usize] = (IntGdiCreateNullBrush()) as HGDIOBJ;

    StockObjects[WHITE_PEN as usize] = (IntCreateStockPen(WhitePen.lopnStyle, WhitePen.lopnWidth.x as u32, BS_SOLID, WhitePen.lopnColor)) as HGDIOBJ;
    StockObjects[BLACK_PEN as usize] = (IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor)) as HGDIOBJ;
    StockObjects[DC_PEN as usize] = (IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor)) as HGDIOBJ;
    StockObjects[NULL_PEN as usize] = (IntCreateStockPen(NullPen.lopnStyle, NullPen.lopnWidth.x as u32, BS_SOLID, NullPen.lopnColor)) as HGDIOBJ;

    StockObjects[20] = (core::ptr::null_mut()) as HGDIOBJ;
    StockObjects[DEFAULT_BITMAP as usize] = (GreCreateBitmap(1, 1, 1, 1, core::ptr::null_mut())) as HGDIOBJ;

    CreateStockFonts();

    StockObjects[DEFAULT_PALETTE as usize] = ((*gppalDefault).BaseObject.hHmgr as HGDIOBJ) as HGDIOBJ;

    Object = 0;
    while Object < NB_STOCK_OBJECTS {
        if !StockObjects[Object as usize].is_null() {
            GDIOBJ_ConvertToStockObj(core::ptr::addr_of_mut!(StockObjects[Object as usize]));
        }
        Object += 1;
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiGetStockObject(Object: INT) -> HGDIOBJ {
    if (Object < 0) || (NB_STOCK_OBJECTS as INT <= Object) {
        core::ptr::null_mut()
    } else {
        StockObjects[Object as usize]
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiGetStockObject(Object: INT) -> HGDIOBJ {
    if (Object < 0) || (NB_STOCK_OBJECTS as INT <= Object) {
        core::ptr::null_mut()
    } else {
        StockObjects[Object as usize]
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetSysColors(nColors: u32, Elements: *const i32, Colors: *const COLORREF) {
    let mut i: u32 = 0;
    let mut elem_ptr = Elements;
    let mut color_ptr = Colors;
    
    while i < nColors {
        if (*elem_ptr as u32) < NUM_SYSCOLORS {
            (*peruser(gpsi)).argbSystem[*elem_ptr as usize] = *color_ptr;
            IntGdiSetSolidBrushColor((*peruser(gpsi)).ahbrSystem[*elem_ptr as usize], *color_ptr);
        }
        elem_ptr = elem_ptr.offset(1);
        color_ptr = color_ptr.offset(1);
        i += 1;
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetSysColors(nColors: u32, Elements: *const i32, Colors: *const COLORREF) {
    let mut i: u32 = 0;
    let mut elem_ptr = Elements;
    let mut color_ptr = Colors;
    
    while i < nColors {
        if (*elem_ptr as u32) < NUM_SYSCOLORS {
            (*peruser(gpsi)).argbSystem[*elem_ptr as usize] = *color_ptr;
            IntGdiSetSolidBrushColor((*peruser(gpsi)).ahbrSystem[*elem_ptr as usize], *color_ptr);
        }
        elem_ptr = elem_ptr.offset(1);
        color_ptr = color_ptr.offset(1);
        i += 1;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetSysColorBrush(Object: INT) -> HGDIOBJ {
    if (Object < 0) || (NUM_SYSCOLORS as INT <= Object) {
        core::ptr::null_mut()
    } else {
        (*peruser(gpsi)).ahbrSystem[Object as usize] as HGDIOBJ
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetSysColorBrush(Object: INT) -> HGDIOBJ {
    if (Object < 0) || (NUM_SYSCOLORS as INT <= Object) {
        core::ptr::null_mut()
    } else {
        (*peruser(gpsi)).ahbrSystem[Object as usize] as HGDIOBJ
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetSysColor(nIndex: INT) -> DWORD {
    if NUM_SYSCOLORS <= nIndex as UINT {
        0
    } else {
        (*peruser(gpsi)).argbSystem[nIndex as usize]
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetSysColor(nIndex: INT) -> DWORD {
    if NUM_SYSCOLORS <= nIndex as UINT {
        0
    } else {
        (*peruser(gpsi)).argbSystem[nIndex as usize]
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn CreateSysColorObjects() {
    for i in 0..NUM_SYSCOLORS as usize {
        (*peruser(gpsi)).argbSystem[i] = SysColors[i];
    }

    for i in 0..NUM_SYSCOLORS as usize {
        if (*peruser(gpsi)).ahbrSystem[i].is_null() {
            (*peruser(gpsi)).ahbrSystem[i] = IntGdiCreateSolidBrush(SysColors[i]);
            if !(*peruser(gpsi)).ahbrSystem[i].is_null() {
                GDIOBJ_ConvertToStockObj(core::ptr::addr_of_mut!((*peruser(gpsi)).ahbrSystem[i]) as *mut HGDIOBJ);
            }
        }
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn CreateSysColorObjects() {
    for i in 0..NUM_SYSCOLORS as usize {
        (*peruser(gpsi)).argbSystem[i] = SysColors[i];
    }

    for i in 0..NUM_SYSCOLORS as usize {
        if (*peruser(gpsi)).ahbrSystem[i].is_null() {
            (*peruser(gpsi)).ahbrSystem[i] = IntGdiCreateSolidBrush(SysColors[i]);
            if !(*peruser(gpsi)).ahbrSystem[i].is_null() {
                GDIOBJ_ConvertToStockObj(core::ptr::addr_of_mut!((*peruser(gpsi)).ahbrSystem[i]) as *mut HGDIOBJ);
            }
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/stockobj.c
//  * PURPOSE:         Stock objects functions
//  * PROGRAMMERS:     Colin Finck <colin@reactos.org>
//  *                  Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// static const COLORREF SysColors[] =
// {
//     RGB(212, 208, 200), /* COLOR_SCROLLBAR  */
//     RGB(58, 110, 165),  /* COLOR_BACKGROUND  */
//     RGB(10, 36, 106),   /* COLOR_ACTIVECAPTION  */
//     RGB(128, 128, 128), /* COLOR_INACTIVECAPTION  */
//     RGB(212, 208, 200), /* COLOR_MENU  */
//     RGB(255, 255, 255), /* COLOR_WINDOW  */
//     RGB(0, 0, 0),       /* COLOR_WINDOWFRAME  */
//     RGB(0, 0, 0),       /* COLOR_MENUTEXT  */
//     RGB(0, 0, 0),       /* COLOR_WINDOWTEXT  */
//     RGB(255, 255, 255), /* COLOR_CAPTIONTEXT  */
//     RGB(212, 208, 200), /* COLOR_ACTIVEBORDER  */
//     RGB(212, 208, 200), /* COLOR_INACTIVEBORDER  */
//     RGB(128, 128, 128), /* COLOR_APPWORKSPACE  */
//     RGB(10, 36, 106),   /* COLOR_HIGHLIGHT  */
//     RGB(255, 255, 255), /* COLOR_HIGHLIGHTTEXT  */
//     RGB(212, 208, 200), /* COLOR_BTNFACE  */
//     RGB(128, 128, 128), /* COLOR_BTNSHADOW  */
//     RGB(128, 128, 128), /* COLOR_GRAYTEXT  */
//     RGB(0, 0, 0),       /* COLOR_BTNTEXT  */
//     RGB(212, 208, 200), /* COLOR_INACTIVECAPTIONTEXT  */
//     RGB(255, 255, 255), /* COLOR_BTNHIGHLIGHT  */
//     RGB(64, 64, 64),    /* COLOR_3DDKSHADOW  */
//     RGB(212, 208, 200), /* COLOR_3DLIGHT  */
//     RGB(0, 0, 0),       /* COLOR_INFOTEXT  */
//     RGB(255, 255, 225), /* COLOR_INFOBK  */
//     RGB(181, 181, 181), /* COLOR_UNKNOWN  */
//     RGB(0, 0, 128),     /* COLOR_HOTLIGHT  */
//     RGB(166, 202, 240), /* COLOR_GRADIENTACTIVECAPTION  */
//     RGB(192, 192, 192), /* COLOR_GRADIENTINACTIVECAPTION  */
//     RGB(49, 106, 197),  /* COLOR_MENUHILIGHT  */
//     RGB(236, 233, 216)  /* COLOR_MENUBAR  */
// };
//
// // System Bitmap DC
// HDC hSystemBM;
//
// /*  GDI stock objects */
//
// static LOGPEN WhitePen =
//     { PS_SOLID, { 0, 0 }, RGB(255,255,255) };
//
// static LOGPEN BlackPen =
//     { PS_SOLID, { 0, 0 }, RGB(0,0,0) };
//
// static LOGPEN NullPen =
//     { PS_NULL, { 0, 0 }, 0 };
//
// static LOGFONTW OEMFixedFont =
//     { 12, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, OEM_CHARSET,
//       OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, FF_DONTCARE | FIXED_PITCH, L"Terminal"
//     };
//
// static LOGFONTW AnsiFixedFont =
//     { 12, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, ANSI_CHARSET,
//       OUT_DEFAULT_PRECIS, /*CLIP_DEFAULT_PRECIS*/ CLIP_STROKE_PRECIS, /*DEFAULT_QUALITY*/ PROOF_QUALITY, FF_DONTCARE | FIXED_PITCH, L"Courier"
//     };
//
// static LOGFONTW AnsiVarFont =
//     { 12, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, ANSI_CHARSET,
//       OUT_DEFAULT_PRECIS, /*CLIP_DEFAULT_PRECIS*/ CLIP_STROKE_PRECIS, /*DEFAULT_QUALITY*/ PROOF_QUALITY, FF_DONTCARE | VARIABLE_PITCH, L"MS Sans Serif"
//     };
//
// static LOGFONTW SystemFont =
//     { 16, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
//       OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, FF_DONTCARE | VARIABLE_PITCH, L"System"
//     };
//
// static LOGFONTW DeviceDefaultFont =
//     { 16, 0, 0, 0, FW_BOLD, FALSE, FALSE, FALSE, ANSI_CHARSET,
//       OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, FF_SWISS | VARIABLE_PITCH, L"System"
//     };
//
// static LOGFONTW SystemFixedFont =
//     { 15, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
//       OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, DEFAULT_QUALITY, FF_DONTCARE | FIXED_PITCH, L"Fixedsys"
//     };
//
// static LOGFONTW DefaultGuiFont =
//     { -11, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
//       OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, /*DEFAULT_QUALITY*/ PROOF_QUALITY, DEFAULT_PITCH | FF_DONTCARE, L"MS Shell Dlg"
//     };
//
// HGDIOBJ StockObjects[NB_STOCK_OBJECTS];
//
// static
// HPEN
// FASTCALL
// IntCreateStockPen(DWORD dwPenStyle,
//                   DWORD dwWidth,
//                   ULONG ulBrushStyle,
//                   ULONG ulColor)
// {
//     HPEN hPen;
//     PBRUSH pbrushPen;
//
//     pbrushPen = PEN_AllocPenWithHandle();
//     if (pbrushPen == NULL) return NULL;
//
//     if ((dwPenStyle & PS_STYLE_MASK) == PS_NULL) dwWidth = 1;
//
//     pbrushPen->iHatch = 0;
//     pbrushPen->lWidth = abs(dwWidth);
//     FLOATOBJ_SetLong(&pbrushPen->eWidth, pbrushPen->lWidth);
//     pbrushPen->ulPenStyle = dwPenStyle;
//     pbrushPen->BrushAttr.lbColor = ulColor;
//     pbrushPen->iBrushStyle = ulBrushStyle;
//     pbrushPen->hbmClient = (HANDLE)NULL;
//     pbrushPen->dwStyleCount = 0;
//     pbrushPen->pStyle = 0;
//     pbrushPen->flAttrs = BR_IS_OLDSTYLEPEN;
//
//     switch (dwPenStyle & PS_STYLE_MASK)
//     {
//         case PS_NULL:
//             pbrushPen->flAttrs |= BR_IS_NULL;
//             break;
//
//         case PS_SOLID:
//             pbrushPen->flAttrs |= BR_IS_SOLID;
//             break;
//     }
//     hPen = pbrushPen->BaseObject.hHmgr;
//     PEN_UnlockPen(pbrushPen);
//     return hPen;
// }
//
// static VOID FASTCALL
// CreateStockFonts(void)
// {
//     USHORT ActiveCodePage, OemCodePage;
//     BYTE bActiveCharSet, bOemCharSet;
//     BOOL bIsCJK;
//     static const WCHAR SimSun[] = { 0x5B8B, 0x4F53, 0 };
//     static const WCHAR MingLiU[] = { 0x7D30, 0x660E, 0x9AD4, 0 };
//     static const WCHAR Batang[] = { 0xBC14, 0xD0D5, 0 };
//
//     RtlGetDefaultCodePage(&ActiveCodePage, &OemCodePage);
//     bActiveCharSet = IntCharSetFromCodePage(ActiveCodePage as UINT);
//     bOemCharSet = IntCharSetFromCodePage(OemCodePage as UINT);
//
//     if (bOemCharSet == DEFAULT_CHARSET as u8)
//         bOemCharSet = OEM_CHARSET as u8;
//
//     switch (ActiveCodePage)
//     {
//         case 936:
//             /* Simplified Chinese */
//             bIsCJK = TRUE;
//             wcscpy(DefaultGuiFont.lfFaceName, SimSun);
//             break;
//
//         case 950:
//             /* Traditional Chinese */
//             bIsCJK = TRUE;
//             wcscpy(DefaultGuiFont.lfFaceName, MingLiU);
//             break;
//
//         case 932:
//             /* Japanese */
//             bIsCJK = TRUE;
//             wcscpy(DefaultGuiFont.lfFaceName, L"MS UI Gothic");
//             break;
//
//         case 949:
//         case 1361:
//             /* Korean */
//             bIsCJK = TRUE;
//             wcscpy(DefaultGuiFont.lfFaceName, Batang);
//             break;
//
//         default:
//             /* Otherwise */
//             bIsCJK = FALSE;
//             wcscpy(DefaultGuiFont.lfFaceName, L"MS Shell Dlg");
//             break;
//     }
//
//     if (bIsCJK)
//     {
//         OEMFixedFont.lfHeight = 18;
//         OEMFixedFont.lfPitchAndFamily = (FF_DONTCARE | FIXED_PITCH) as _;
//         SystemFont.lfHeight = 18;
//         SystemFont.lfPitchAndFamily = (FF_DONTCARE | VARIABLE_PITCH) as _;
//         DeviceDefaultFont.lfHeight = 18;
//         DeviceDefaultFont.lfPitchAndFamily = (FF_DONTCARE | VARIABLE_PITCH) as _;
//         SystemFixedFont.lfHeight = 18;
//         SystemFixedFont.lfPitchAndFamily = (FF_DONTCARE | FIXED_PITCH) as _;
//         DefaultGuiFont.lfHeight = -12;
//     }
//     else
//     {
//         OEMFixedFont.lfHeight = 12;
//         OEMFixedFont.lfPitchAndFamily = (FF_MODERN | FIXED_PITCH) as _;
//         SystemFont.lfHeight = 16;
//         SystemFont.lfPitchAndFamily = (FF_SWISS | VARIABLE_PITCH) as _;
//         DeviceDefaultFont.lfHeight = 16;
//         DeviceDefaultFont.lfPitchAndFamily = (FF_SWISS | VARIABLE_PITCH) as _;
//         if (bActiveCharSet == RUSSIAN_CHARSET as u8)
//         {
//             SystemFixedFont.lfHeight = 16;
//             SystemFixedFont.lfPitchAndFamily = (FF_SWISS | FIXED_PITCH) as _;
//         }
//         else
//         {
//             SystemFixedFont.lfHeight = 15;
//             SystemFixedFont.lfPitchAndFamily = (FF_MODERN | FIXED_PITCH) as _;
//         }
//         DefaultGuiFont.lfHeight = -11;
//     }
//
//     OEMFixedFont.lfCharSet = bOemCharSet;
//     SystemFont.lfCharSet = bActiveCharSet;
//     DeviceDefaultFont.lfCharSet = bActiveCharSet;
//     SystemFixedFont.lfCharSet = bActiveCharSet;
//     DefaultGuiFont.lfCharSet = bActiveCharSet;
//
//     TextIntCreateFontIndirect(&OEMFixedFont, (HFONT*)&StockObjects[OEM_FIXED_FONT]);
//     TextIntCreateFontIndirect(&AnsiFixedFont, (HFONT*)&StockObjects[ANSI_FIXED_FONT]);
//     TextIntCreateFontIndirect(&AnsiVarFont, (HFONT*)&StockObjects[ANSI_VAR_FONT]);
//     TextIntCreateFontIndirect(&SystemFont, (HFONT*)&StockObjects[SYSTEM_FONT]);
//     TextIntCreateFontIndirect(&DeviceDefaultFont, (HFONT*)&StockObjects[DEVICE_DEFAULT_FONT]);
//     TextIntCreateFontIndirect(&SystemFixedFont, (HFONT*)&StockObjects[SYSTEM_FIXED_FONT]);
//     TextIntCreateFontIndirect(&DefaultGuiFont, (HFONT*)&StockObjects[DEFAULT_GUI_FONT]);
// }
//
// /*!
//  * Creates a bunch of stock objects: brushes, pens, fonts.
// */
// VOID FASTCALL
// CreateStockObjects(void)
// {
//     UINT Object;
//
//     DPRINT("Beginning creation of stock objects\n");
//
//     /* Create GDI Stock Objects from the logical structures we've defined */
//
//     StockObjects[WHITE_BRUSH] =  IntGdiCreateSolidBrush(RGB(255,255,255));
//     StockObjects[DC_BRUSH]    =  IntGdiCreateSolidBrush(RGB(255,255,255));
//     StockObjects[LTGRAY_BRUSH] = IntGdiCreateSolidBrush(RGB(192,192,192));
//     StockObjects[GRAY_BRUSH] =   IntGdiCreateSolidBrush(RGB(128,128,128));
//     StockObjects[DKGRAY_BRUSH] = IntGdiCreateSolidBrush(RGB(64,64,64));
//     StockObjects[BLACK_BRUSH] =  IntGdiCreateSolidBrush(RGB(0,0,0));
//     StockObjects[NULL_BRUSH] =   IntGdiCreateNullBrush();
//
//     StockObjects[WHITE_PEN] = IntCreateStockPen(WhitePen.lopnStyle, WhitePen.lopnWidth.x as u32, BS_SOLID, WhitePen.lopnColor);
//     StockObjects[BLACK_PEN] = IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor);
//     StockObjects[DC_PEN]    = IntCreateStockPen(BlackPen.lopnStyle, BlackPen.lopnWidth.x as u32, BS_SOLID, BlackPen.lopnColor);
//     StockObjects[NULL_PEN]  = IntCreateStockPen(NullPen.lopnStyle, NullPen.lopnWidth.x as u32, BS_SOLID, NullPen.lopnColor);
//
//     StockObjects[20] = NULL; /* TODO: Unknown internal stock object */
//     StockObjects[DEFAULT_BITMAP] = GreCreateBitmap(1, 1, 1, 1, NULL);
//
//     CreateStockFonts();
//
//     StockObjects[DEFAULT_PALETTE] = (HGDIOBJ)gppalDefault->BaseObject.hHmgr;
//
//     for (Object = 0; Object < NB_STOCK_OBJECTS; Object++)
//     {
//         if (NULL != StockObjects[Object])
//         {
//             GDIOBJ_ConvertToStockObj(&StockObjects[Object]);
//         }
//     }
//
//     DPRINT("Completed creation of stock objects\n");
// }
//
// /*!
//  * Return stock object.
//  * \param	Object - stock object id.
//  * \return	Handle to the object.
// */
// HGDIOBJ APIENTRY
// NtGdiGetStockObject(INT Object)
// {
//     DPRINT("NtGdiGetStockObject index %d\n", Object);
//
//     return ((Object < 0) || (NB_STOCK_OBJECTS <= Object)) ? NULL : StockObjects[Object];
// }
//
// VOID FASTCALL
// IntSetSysColors(UINT nColors, CONST INT *Elements, CONST COLORREF *Colors)
// {
//     UINT i;
//
//     for (i = 0; i < nColors; i++)
//     {
//         if ((UINT)(*Elements) < NUM_SYSCOLORS)
//         {
//             gpsi->argbSystem[*Elements] = *Colors;
//             IntGdiSetSolidBrushColor(gpsi->ahbrSystem[*Elements], *Colors);
//         }
//         Elements++;
//         Colors++;
//     }
// }
//
// HGDIOBJ FASTCALL
// IntGetSysColorBrush(INT Object)
// {
//     return ((Object < 0) || (NUM_SYSCOLORS <= Object)) ? NULL : gpsi->ahbrSystem[Object];
// }
//
// DWORD FASTCALL
// IntGetSysColor(INT nIndex)
// {
//     return (NUM_SYSCOLORS <= (UINT)nIndex) ? 0 : gpsi->argbSystem[nIndex];
// }
//
// VOID FASTCALL
// CreateSysColorObjects(VOID)
// {
//     UINT i;
//
//     for (i = 0; i < NUM_SYSCOLORS; i++)
//     {
//         gpsi->argbSystem[i] = SysColors[i];
//     }
//
//     /* Create the syscolor brushes */
//     for (i = 0; i < NUM_SYSCOLORS; i++)
//     {
//         if (gpsi->ahbrSystem[i] == NULL)
//         {
//             gpsi->ahbrSystem[i] = IntGdiCreateSolidBrush(SysColors[i]);
//             if (gpsi->ahbrSystem[i] != NULL)
//             {
//                 GDIOBJ_ConvertToStockObj((HGDIOBJ*)&gpsi->ahbrSystem[i]);
//             }
//         }
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
