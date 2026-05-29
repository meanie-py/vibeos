use win32k_sys::win32k::*;
use core::ffi::c_void;
use crate::dib_helpers::*;

extern "system" {
    fn EXLATEOBJ_vCleanup(exlate: *mut EXLATEOBJ);
    fn EXLATEOBJ_vInitialize(exlate: *mut EXLATEOBJ, ppalSrc: *mut core::ffi::c_void, ppalDst: *mut core::ffi::c_void, arg3: u32, arg4: u32, arg5: u32);
    fn pfnDibPutPixel(surf: *mut SURFOBJ, x: i32, y: i32, color: u32);
    static gpalRGB: core::ffi::c_void;
    static DibFunctionsForBitmapFormat: [DIB_FUNCTIONS; 256];
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub union NICEPIXEL32 {
    pub ul: ULONG,
    pub col: NICEPIXEL32_col,
}

#[repr(C)]
pub struct NICEPIXEL32_col {
    pub red: UCHAR,
    pub green: UCHAR,
    pub blue: UCHAR,
    pub alpha: UCHAR,
}

#[inline(always)]

#[repr(C)]
union NICEPIXEL32 {
    ul: u32,
    col: NICEPIXEL32_col,
}

#[repr(C)]
struct NICEPIXEL32_col {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

#[repr(C)]
struct DIB_FUNCTIONS {
    DIB_PutPixel: PFN_DIB_PutPixel,
}

unsafe fn Clamp8(val: u32) -> u8 {
    if val > 255 { 255 } else { val as u8 }
}

#[no_mangle]
pub unsafe extern "C" fn DIB_XXBPP_AlphaBlend(
    Dest: *mut SURFOBJ,
    Source: *mut SURFOBJ,
    DestRect: *mut RECTL,
    SourceRect: *mut RECTL,
    ClipRegion: *mut CLIPOBJ,
    ColorTranslation: *mut XLATEOBJ,
    BlendObj: *mut BLENDOBJ,
) -> u8 {
    let mut DstX: i32;
    let mut DstY: i32;
    let mut SrcX: i32;
    let mut SrcY: i32;
    let BlendFunc: BLENDFUNCTION;
    let mut DstPixel32: NICEPIXEL32;
    let mut SrcPixel32: NICEPIXEL32;
    let mut Alpha: u8;
    let SrcBpp: u8 = BitsPerFormat((*Source).iBitmapFormat);
    let pexlo: *mut EXLATEOBJ;
    let mut exloSrcRGB: EXLATEOBJ = core::mem::zeroed();
    let mut exloDstRGB: EXLATEOBJ = core::mem::zeroed();
    let mut exloRGBSrc: EXLATEOBJ = core::mem::zeroed();
    let pfnDibPutPixel: PFN_DIB_PutPixel = (*DibFunctionsForBitmapFormat.as_ptr().add((*Dest).iBitmapFormat as usize)).DIB_PutPixel;

    BlendFunc = (*BlendObj).BlendFunction;
    if BlendFunc.BlendOp != AC_SRC_OVER {
        return 0u8;
    }
    if BlendFunc.BlendFlags != 0 {
        return 0u8;
    }
    if (BlendFunc.AlphaFormat & !AC_SRC_ALPHA) != 0 {
        return 0u8;
    }
    if (BlendFunc.AlphaFormat & AC_SRC_ALPHA) != 0 && SrcBpp != 32 {
        return 0u8;
    }

    if ColorTranslation.is_null() {
        return 0u8;
    }

    pexlo = (ColorTranslation as *mut u8).sub(core::mem::offset_of!(EXLATEOBJ, xlo)) as *mut EXLATEOBJ;
    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exloSrcRGB),
        (*pexlo).ppalSrc,
        core::ptr::addr_of!(gpalRGB) as *mut core::ffi::c_void,
        0,
        0,
        0,
    );
    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exloDstRGB),
        (*pexlo).ppalDst,
        core::ptr::addr_of!(gpalRGB) as *mut core::ffi::c_void,
        0,
        0,
        0,
    );
    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exloRGBSrc),
        core::ptr::addr_of!(gpalRGB) as *mut core::ffi::c_void,
        (*pexlo).ppalSrc,
        0,
        0,
        0,
    );

    SrcY = (*SourceRect).top;
    DstY = (*DestRect).top;
    while DstY < (*DestRect).bottom {
        SrcX = (*SourceRect).left;
        DstX = (*DestRect).left;
        while DstX < (*DestRect).right {
            SrcPixel32.ul = DIB_GetSource(Source, SrcX, SrcY, core::ptr::addr_of_mut!(exloSrcRGB.xlo));
            SrcPixel32.col.red = (SrcPixel32.col.red as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
            SrcPixel32.col.green = (SrcPixel32.col.green as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;
            SrcPixel32.col.blue = (SrcPixel32.col.blue as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8;

            Alpha = if (BlendFunc.AlphaFormat & AC_SRC_ALPHA) != 0 {
                (SrcPixel32.col.alpha as u32 * BlendFunc.SourceConstantAlpha as u32 / 255) as u8
            } else {
                BlendFunc.SourceConstantAlpha
            };

            DstPixel32.ul = DIB_GetSource(Dest, DstX, DstY, core::ptr::addr_of_mut!(exloDstRGB.xlo));
            DstPixel32.col.red = Clamp8(DstPixel32.col.red as u32 * (255 - Alpha as u32) / 255 + SrcPixel32.col.red as u32);
            DstPixel32.col.green = Clamp8(DstPixel32.col.green as u32 * (255 - Alpha as u32) / 255 + SrcPixel32.col.green as u32);
            DstPixel32.col.blue = Clamp8(DstPixel32.col.blue as u32 * (255 - Alpha as u32) / 255 + SrcPixel32.col.blue as u32);
            DstPixel32.ul = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exloRGBSrc.xlo), DstPixel32.ul);
            pfnDibPutPixel.unwrap()(Dest, DstX, DstY, XLATEOBJ_iXlate(ColorTranslation, DstPixel32.ul));

            DstX += 1;
            SrcX = (*SourceRect).left + ((DstX - (*DestRect).left) * ((*SourceRect).right - (*SourceRect).left)) / ((*DestRect).right - (*DestRect).left);
        }
        DstY += 1;
        SrcY = (*SourceRect).top + ((DstY - (*DestRect).top) * ((*SourceRect).bottom - (*SourceRect).top)) / ((*DestRect).bottom - (*DestRect).top);
    }

    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exloDstRGB));
    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exloRGBSrc));
    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exloSrcRGB));

    1u8
}
