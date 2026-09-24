use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

// EXLATEOBJ_* / SafeProbeFor* / PsGetCurrentThreadWin32Thread are NTAPI (stdcall on i386).
extern "system" {
    fn EXLATEOBJ_vCleanup(pexlo: *mut EXLATEOBJ);
    fn EXLATEOBJ_vInitialize(pexlo: *mut EXLATEOBJ, ppalSrc: PPALETTE, ppalDst: PPALETTE, crBackgroundClr: COLORREF, crForegroundClr: COLORREF, crDIBBackgroundClr: COLORREF);
    fn SafeProbeForRead(Address: *const core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn PsGetCurrentThreadWin32Thread() -> PVOID;
}
// TranslateCOLORREF is plain cdecl; memcpy is cdecl.
extern "C" {
    fn TranslateCOLORREF(pdc: PDC, crColor: COLORREF) -> COLORREF;
    fn memcpy(dest: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize) -> *mut core::ffi::c_void;
}
// SetLastNtError / UserReference*Object / co_UserRedrawWindow are FASTCALL (@Name@N on i386).
#[cfg(target_arch = "x86")] extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn co_UserRedrawWindow(Wnd: PWND, UpdateRect: *const RECTL, UpdateRgn: PREGION, Flags: ULONG) -> BOOL;
}
#[cfg(not(target_arch = "x86"))] extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn co_UserRedrawWindow(Wnd: PWND, UpdateRect: *const RECTL, UpdateRgn: PREGION, Flags: ULONG) -> BOOL;
}

// UserRef/DerefObjectCo are static __inline (object.h): ref the obj + push/pop the
// per-thread ReferencesList. USER_REFERENCE_ENTRY.Entry is the first field, so the
// PopEntryList node IS the entry (CONTAINING_RECORD offset 0).
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PWND, entry: *mut USER_REFERENCE_ENTRY) {
    let w32 = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    (*entry).Entry.Next = (*w32).ReferencesList.Next; // PushEntryList
    (*w32).ReferencesList.Next = core::ptr::addr_of_mut!((*entry).Entry);
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PWND) {
    let w32 = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let re = (*w32).ReferencesList.Next; // PopEntryList
    if !re.is_null() {
        (*w32).ReferencesList.Next = (*re).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

// GDIOBJ macro inlines (palette.h): share-lock/unlock + unlock.
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HPALETTE) -> PPALETTE { GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
#[inline(always)] unsafe fn PALETTE_UnlockPalette(p: PPALETTE) { GDIOBJ_vUnlockObject(core::ptr::addr_of_mut!((*p).BaseObject)); }

// Color macros (wingdi.h).
#[inline(always)] fn RGB(r: u32, g: u32, b: u32) -> COLORREF { ((r & 0xff) | ((g & 0xff) << 8) | ((b & 0xff) << 16)) as COLORREF }
#[inline(always)] fn GetRValue(rgb: u32) -> u32 { rgb & 0xff }
#[inline(always)] fn GetGValue(rgb: u32) -> u32 { (rgb >> 8) & 0xff }
#[inline(always)] fn GetBValue(rgb: u32) -> u32 { (rgb >> 16) & 0xff }

// GDI handle macros (ntgdihdl.h): TYPE_MASK 0x007f0000, STOCK_MASK 0x00800000.
#[inline(always)] fn GDI_HANDLE_IS_TYPE(h: HGDIOBJ, t: u32) -> BOOL { (((h as usize as u32) & 0x007f0000) == t) as BOOL }
#[inline(always)] fn GDI_HANDLE_IS_STOCKOBJ(h: HGDIOBJ) -> BOOL { (((h as usize as u32) & 0x00800000) != 0) as BOOL }

// CalculateShift (palette.h FORCEINLINE, via BitScanReverse) + _rotl intrinsic.
#[inline(always)]
fn CalculateShift(ulMask1: u32, ulMask2: u32) -> u32 {
    let s1 = if ulMask1 != 0 { 31 - ulMask1.leading_zeros() } else { 0 };
    let s2 = if ulMask2 != 0 { 31 - ulMask2.leading_zeros() } else { 0 };
    let mut s = s2.wrapping_sub(s1);
    if (s as i32) < 0 { s = s.wrapping_add(32); }
    s
}
#[inline(always)] fn _rotl(value: u32, shift: u32) -> u32 { value.rotate_left(shift) }

#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
const TAG_PALETTE: u32 = u32::from_le_bytes(*b"GPal"); // GDITAG_PALETTE 'laPG'
// ---- file-scope preamble (translated) ----
pub const PAL_SETPOWNER: u32 = 0x8000;

pub const MAX_PALCOLORS: u32 = 65536;

pub static mut SystemPaletteUse: UINT = SYSPAL_NOSTATIC;

pub static g_sysPalTemplate: [PALETTEENTRY; NB_RESERVED_COLORS as usize] = [
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x00, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x80, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x80, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0x80, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x00, peBlue: 0x80, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x80, peBlue: 0x80, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xc0, peGreen: 0xc0, peBlue: 0xc0, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xc0, peGreen: 0xdc, peBlue: 0xc0, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xa6, peGreen: 0xca, peBlue: 0xf0, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xff, peGreen: 0xfb, peBlue: 0xf0, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xa0, peGreen: 0xa0, peBlue: 0xa4, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x80, peGreen: 0x80, peBlue: 0x80, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xff, peGreen: 0x00, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0xff, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xff, peGreen: 0xff, peBlue: 0x00, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0x00, peBlue: 0xff, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xff, peGreen: 0x00, peBlue: 0xff, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0x00, peGreen: 0xff, peBlue: 0xff, peFlags: PC_SYS_USED as u8 },
    PALETTEENTRY { peRed: 0xff, peGreen: 0xff, peBlue: 0xff, peFlags: PC_SYS_USED as u8 },
];

// palette.c-owned globals (BSS). Flipping palette.c removes the C definitions,
// so palette.rs defines the real #[no_mangle] symbols the other GDI units
// (engbrush/surface/stockobj/fillshap/mouse/xlateobj/dcutil/dibobj/freetype/…)
// link against. bindgen models them as extern decls / [;0] — shadowed here.
#[no_mangle] pub static mut gpalRGB: PALETTE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut gpalBGR: PALETTE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut gpalRGB555: PALETTE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut gpalRGB565: PALETTE = unsafe { core::mem::zeroed() };
#[no_mangle] pub static mut gppalMono: PPALETTE = core::ptr::null_mut();
#[no_mangle] pub static mut gppalDefault: PPALETTE = core::ptr::null_mut();
#[no_mangle] pub static mut appalSurfaceDefault: [PPALETTE; 11] = [core::ptr::null_mut(); 11];

unsafe fn GetNumberOfBits(mut dwMask: u32) -> u16 {
    let mut wBits: u16 = 0;
    while dwMask != 0 {
        dwMask = dwMask & (dwMask - 1);
        wBits += 1;
    }
    wBits
}

#[no_mangle]
pub unsafe extern "system" fn InitPaletteImpl() -> NTSTATUS {
    gppalDefault = PALETTE_AllocPalWithHandle(
        PAL_INDEXED,
        20,
        g_sysPalTemplate.as_ptr(),
        0,
        0,
        0,
    );
    GDIOBJ_vReferenceObjectByPointer(core::ptr::addr_of_mut!((*gppalDefault).BaseObject));
    PALETTE_UnlockPalette(gppalDefault);

    gpalRGB.flFlags = PAL_RGB;
    gpalRGB.RedMask = RGB(0xFF, 0x00, 0x00);
    gpalRGB.GreenMask = RGB(0x00, 0xFF, 0x00);
    gpalRGB.BlueMask = RGB(0x00, 0x00, 0xFF);
    gpalRGB.BaseObject.__bindgen_anon_1.ulShareCount = 1;
    gpalRGB.BaseObject.BaseFlags = 0;

    gpalBGR.flFlags = PAL_BGR;
    gpalBGR.RedMask = RGB(0x00, 0x00, 0xFF);
    gpalBGR.GreenMask = RGB(0x00, 0xFF, 0x00);
    gpalBGR.BlueMask = RGB(0xFF, 0x00, 0x00);
    gpalBGR.BaseObject.__bindgen_anon_1.ulShareCount = 1;
    gpalBGR.BaseObject.BaseFlags = 0;

    gpalRGB555.flFlags = PAL_RGB16_555 | PAL_BITFIELDS;
    gpalRGB555.RedMask = 0x7C00;
    gpalRGB555.GreenMask = 0x3E0;
    gpalRGB555.BlueMask = 0x1F;
    gpalRGB555.BaseObject.__bindgen_anon_1.ulShareCount = 1;
    gpalRGB555.BaseObject.BaseFlags = 0;

    gpalRGB565.flFlags = PAL_RGB16_565 | PAL_BITFIELDS;
    gpalRGB565.RedMask = 0xF800;
    gpalRGB565.GreenMask = 0x7E0;
    gpalRGB565.BlueMask = 0x1F;
    gpalRGB565.BaseObject.__bindgen_anon_1.ulShareCount = 1;
    gpalRGB565.BaseObject.BaseFlags = 0;

    gppalMono = PALETTE_AllocPalette(
        PAL_MONOCHROME | PAL_INDEXED,
        2,
        core::ptr::null_mut(),
        0,
        0,
        0,
    );
    PALETTE_vSetRGBColorForIndex(gppalMono, 0, 0x000000);
    PALETTE_vSetRGBColorForIndex(gppalMono, 1, 0xffffff);

    appalSurfaceDefault[BMF_1BPP as usize] = gppalMono;
    appalSurfaceDefault[BMF_4BPP as usize] = gppalDefault;
    appalSurfaceDefault[BMF_8BPP as usize] = gppalDefault;
    appalSurfaceDefault[BMF_16BPP as usize] = core::ptr::addr_of_mut!(gpalRGB565);
    appalSurfaceDefault[BMF_24BPP as usize] = core::ptr::addr_of_mut!(gpalBGR);
    appalSurfaceDefault[BMF_32BPP as usize] = core::ptr::addr_of_mut!(gpalBGR);
    appalSurfaceDefault[BMF_4RLE as usize] = gppalDefault;
    appalSurfaceDefault[BMF_8RLE as usize] = gppalDefault;
    appalSurfaceDefault[BMF_JPEG as usize] = core::ptr::addr_of_mut!(gpalRGB);
    appalSurfaceDefault[BMF_PNG as usize] = core::ptr::addr_of_mut!(gpalRGB);

    STATUS_SUCCESS
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn PALETTE_ValidateFlags(lpPalE: *mut PALETTEENTRY, size: INT) {
    let mut i: INT = 0;
    while i < size {
        (*lpPalE.offset(i as isize)).peFlags = (PC_SYS_USED as u8) | ((*lpPalE.offset(i as isize)).peFlags & 0x07);
        i += 1;
    }
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn PALETTE_ValidateFlags(lpPalE: *mut PALETTEENTRY, size: INT) {
    let mut i: INT = 0;
    while i < size {
        (*lpPalE.offset(i as isize)).peFlags = (PC_SYS_USED as u8) | ((*lpPalE.offset(i as isize)).peFlags & 0x07);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_AllocPalette(
    iMode: ULONG,
    cColors: ULONG,
    pEntries: *const PALETTEENTRY,
    flRed: FLONG,
    flGreen: FLONG,
    flBlue: FLONG,
) -> PPALETTE {
    let mut iMode = iMode;
    let mut cColors = cColors;
    let mut fl: ULONG = 0;
    let mut cjSize: ULONG = core::mem::size_of::<PALETTE>() as ULONG;

    if (iMode & PAL_INDEXED) != 0 {
        if cColors == 0 || cColors > 1024 {
            return core::ptr::null_mut();
        }

        if cColors == 2 {
            iMode |= PAL_MONOCHROME;
        }

        cjSize += cColors * core::mem::size_of::<PALETTEENTRY>() as ULONG;
    } else {
        cColors = 0;
        fl |= BASEFLAG_LOOKASIDE;
    }

    let ppal = GDIOBJ_AllocateObject(GDIObjType_PAL_TYPE as u8, cjSize, fl) as PPALETTE;
    if ppal.is_null() {
        return core::ptr::null_mut();
    }

    (*ppal).flFlags = iMode;
    (*ppal).NumColors = cColors;
    (*ppal).IndexedColors = core::ptr::addr_of_mut!((*ppal).apalColors) as *mut PALETTEENTRY;

    if (iMode & PAL_INDEXED) != 0 {
        if !pEntries.is_null() {
            RtlCopyMemory(
                (*ppal).IndexedColors as *mut core::ffi::c_void,
                pEntries as *const core::ffi::c_void,
                (cColors as usize) * core::mem::size_of::<PALETTEENTRY>(),
            );
        }
    } else if (iMode & PAL_BITFIELDS) != 0 {
        (*ppal).RedMask = flRed;
        (*ppal).GreenMask = flGreen;
        (*ppal).BlueMask = flBlue;

        if flRed == 0x7c00 && flGreen == 0x3E0 && flBlue == 0x1F {
            (*ppal).flFlags |= PAL_RGB16_555;
        } else if flRed == 0xF800 && flGreen == 0x7E0 && flBlue == 0x1F {
            (*ppal).flFlags |= PAL_RGB16_565;
        } else if flRed == 0xFF0000 && flGreen == 0xFF00 && flBlue == 0xFF {
            (*ppal).flFlags |= PAL_BGR;
        } else if flRed == 0xFF && flGreen == 0xFF00 && flBlue == 0xFF0000 {
            (*ppal).flFlags |= PAL_RGB;
        }
    }

    ppal
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_AllocPalWithHandle(
    iMode: ULONG,
    cColors: ULONG,
    pEntries: *const PALETTEENTRY,
    flRed: FLONG,
    flGreen: FLONG,
    flBlue: FLONG,
) -> PPALETTE {
    let ppal = PALETTE_AllocPalette(iMode, cColors, pEntries, flRed, flGreen, flBlue);
    if ppal.is_null() {
        return core::ptr::null_mut();
    }

    if GDIOBJ_hInsertObject(core::ptr::addr_of_mut!((*ppal).BaseObject), GDI_OBJ_HMGR_POWNED).is_null() {
        GDIOBJ_vFreeObject(core::ptr::addr_of_mut!((*ppal).BaseObject));
        return core::ptr::null_mut();
    }

    ppal
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_vCleanup(ObjectBody: PVOID) {
    let pPal: PPALETTE = ObjectBody as PPALETTE;
    if !(*pPal).IndexedColors.is_null() && (*pPal).IndexedColors != (*pPal).apalColors.as_mut_ptr() {
        ExFreePoolWithTag((*pPal).IndexedColors as PVOID, TAG_PALETTE);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PALETTE_GetObject(ppal: PPALETTE, cbCount: INT, lpBuffer: LPLOGBRUSH) -> INT {
    PALETTE_GetObject_impl(ppal, cbCount, lpBuffer)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PALETTE_GetObject(ppal: PPALETTE, cbCount: INT, lpBuffer: LPLOGBRUSH) -> INT {
    PALETTE_GetObject_impl(ppal, cbCount, lpBuffer)
}

#[inline(always)]
unsafe fn PALETTE_GetObject_impl(ppal: PPALETTE, cbCount: INT, lpBuffer: LPLOGBRUSH) -> INT {
    if lpBuffer.is_null() {
        return core::mem::size_of::<WORD>() as INT;
    }

    if (cbCount as UINT) < core::mem::size_of::<WORD>() as UINT {
        return 0;
    }

    *(lpBuffer as *mut WORD) = (*ppal).NumColors as WORD;
    core::mem::size_of::<WORD>() as INT
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_ulGetNearestPaletteIndex(ppal: *mut PALETTE, iColor: ULONG) -> ULONG {
    let mut ulDiff: ULONG;
    let mut ulColorDiff: ULONG;
    let mut ulMinimalDiff: ULONG = 0xFFFFFF;
    let mut ulBestIndex: ULONG = 0;
    let peColor: PALETTEENTRY = *(&iColor as *const ULONG as *const PALETTEENTRY);

    let mut i: ULONG = 0;
    while i < (*ppal).NumColors {
        let entry = (*ppal).IndexedColors.offset(i as isize);
        ulDiff = peColor.peRed as ULONG - (*entry).peRed as ULONG;
        ulColorDiff = ulDiff * ulDiff;
        ulDiff = peColor.peGreen as ULONG - (*entry).peGreen as ULONG;
        ulColorDiff += ulDiff * ulDiff;
        ulDiff = peColor.peBlue as ULONG - (*entry).peBlue as ULONG;
        ulColorDiff += ulDiff * ulDiff;

        if ulColorDiff < ulMinimalDiff {
            ulBestIndex = i;
            ulMinimalDiff = ulColorDiff;

            if ulMinimalDiff == 0 {
                break;
            }
        }

        i += 1;
    }

    ulBestIndex
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_ulGetNearestBitFieldsIndex(ppal: *mut PALETTE, ulColor: ULONG) -> ULONG {
    let ulNewColor: ULONG;

    (*ppal).ulRedShift = CalculateShift(RGB(0xff, 0, 0), (*ppal).RedMask);
    (*ppal).ulGreenShift = CalculateShift(RGB(0, 0xff, 0), (*ppal).GreenMask);
    (*ppal).ulBlueShift = CalculateShift(RGB(0, 0, 0xff), (*ppal).BlueMask);

    let mut color = _rotl(ulColor, (*ppal).ulRedShift) & (*ppal).RedMask;
    color |= _rotl(ulColor, (*ppal).ulGreenShift) & (*ppal).GreenMask;
    color |= _rotl(ulColor, (*ppal).ulBlueShift) & (*ppal).BlueMask;
    ulNewColor = color;

    ulNewColor
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_ulGetNearestIndex(ppal: *mut PALETTE, ulColor: ULONG) -> ULONG {
    if ((*ppal).flFlags & PAL_RGB) != 0 {
        return ulColor;
    }
    if ((*ppal).flFlags & PAL_BGR) != 0 {
        return RGB(GetBValue(ulColor), GetGValue(ulColor), GetRValue(ulColor));
    }
    if ((*ppal).flFlags & PAL_INDEXED) != 0 {
        return PALETTE_ulGetNearestPaletteIndex(ppal, ulColor);
    } else {
        return PALETTE_ulGetNearestBitFieldsIndex(ppal, ulColor);
    }
}
#[no_mangle]
pub unsafe extern "system" fn PALETTE_vGetBitMasks(ppal: PPALETTE, pulColors: PULONG) {
    if ((*ppal).flFlags & PAL_INDEXED as u32) != 0 || ((*ppal).flFlags & PAL_RGB as u32) != 0 {
        *pulColors.offset(0) = RGB(0xFF, 0x00, 0x00);
        *pulColors.offset(1) = RGB(0x00, 0xFF, 0x00);
        *pulColors.offset(2) = RGB(0x00, 0x00, 0xFF);
    } else if ((*ppal).flFlags & PAL_BGR as u32) != 0 {
        *pulColors.offset(0) = RGB(0x00, 0x00, 0xFF);
        *pulColors.offset(1) = RGB(0x00, 0xFF, 0x00);
        *pulColors.offset(2) = RGB(0xFF, 0x00, 0x00);
    } else if ((*ppal).flFlags & PAL_BITFIELDS as u32) != 0 {
        *pulColors.offset(0) = (*ppal).RedMask;
        *pulColors.offset(1) = (*ppal).GreenMask;
        *pulColors.offset(2) = (*ppal).BlueMask;
    }
}

fastcall_fn! {
#[no_mangle]
pub unsafe fn ColorCorrection(PalGDI: PPALETTE, PaletteEntry: PPALETTEENTRY, Colors: u32) {
    let ppdev: PPDEVOBJ = (*PalGDI).hPDev as PPDEVOBJ;

    if ppdev.is_null() { return; }

    if (*ppdev).flFlags & PDEV_GAMMARAMP_TABLE != 0 {
        let GammaRamp: PGAMMARAMP = (*ppdev).pvGammaRamp as PGAMMARAMP;
        let mut i: u32 = 0;
        while i < Colors {
            let entry = PaletteEntry.offset(i as isize);
            (*entry).peRed = (*entry).peRed.wrapping_add((*GammaRamp).Red[i as usize] as u8);
            (*entry).peGreen = (*entry).peGreen.wrapping_add((*GammaRamp).Green[i as usize] as u8);
            (*entry).peBlue = (*entry).peBlue.wrapping_add((*GammaRamp).Blue[i as usize] as u8);
            i += 1;
        }
    }
}
}
#[no_mangle]
pub unsafe extern "system" fn EngCreatePalette(
    iMode: ULONG,
    cColors: ULONG,
    pulColors: *mut ULONG,
    flRed: FLONG,
    flGreen: FLONG,
    flBlue: FLONG,
) -> HPALETTE {
    let ppal: PPALETTE = PALETTE_AllocPalette(
        iMode,
        cColors,
        pulColors as PPALETTEENTRY,
        flRed,
        flGreen,
        flBlue,
    );
    if ppal.is_null() {
        return core::ptr::null_mut();
    }

    let hpal: HPALETTE = GDIOBJ_hInsertObject(
        core::ptr::addr_of_mut!((*ppal).BaseObject),
        GDI_OBJ_HMGR_PUBLIC,
    ) as HPALETTE;
    if hpal.is_null() {
        GDIOBJ_vFreeObject(core::ptr::addr_of_mut!((*ppal).BaseObject));
        return core::ptr::null_mut();
    }

    PALETTE_UnlockPalette(ppal);
    hpal
}
#[no_mangle]
pub unsafe extern "system" fn EngDeletePalette(hpal: HPALETTE) -> u32 {

    let ppal = PALETTE_ShareLockPalette(hpal);
    if ppal.is_null() {
        return 0;
    }

    GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*ppal).BaseObject));

    1
}
#[no_mangle]
pub unsafe extern "system" fn PALOBJ_cGetColors(PalObj: *mut PALOBJ, Start: u32, mut Colors: u32, PaletteEntry: *mut u32) -> u32 {
    let PalGDI: *mut PALETTE = PalObj as *mut PALETTE;

    if Start >= (*PalGDI).NumColors {
        return 0;
    }

    let available = (*PalGDI).NumColors - Start;
    if available < Colors {
        Colors = available;
    }

    RtlCopyMemory(
        PaletteEntry as *mut core::ffi::c_void,
        (*PalGDI).IndexedColors.offset(Start as isize) as *const core::ffi::c_void,
        (core::mem::size_of::<u32>() as u32 * Colors) as usize,
    );

    if (*PalGDI).flFlags & PAL_GAMMACORRECTION != 0 {
        ColorCorrection(PalGDI, PaletteEntry as PPALETTEENTRY, Colors);
    }

    Colors
}
#[no_mangle]
pub unsafe extern "system" fn GreCreatePaletteInternal(
    pLogPal: LPLOGPALETTE,
    cEntries: UINT,
) -> HPALETTE {
    let mut hpal: HPALETTE = core::ptr::null_mut();
    let ppal: PPALETTE;

    (*pLogPal).palNumEntries = cEntries as u16;
    ppal = PALETTE_AllocPalWithHandle(
        PAL_INDEXED,
        cEntries,
        core::ptr::addr_of_mut!((*pLogPal).palPalEntry) as *const PALETTEENTRY,
        0,
        0,
        0,
    );

    if !ppal.is_null() {
        PALETTE_ValidateFlags((*ppal).IndexedColors, (*ppal).NumColors as INT);

        hpal = (*ppal).BaseObject.hHmgr as HPALETTE;
        PALETTE_UnlockPalette(ppal);
    }

    hpal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreatePaletteInternal(
    plogpalUser: LPLOGPALETTE,
    cEntries: UINT,
) -> HPALETTE {
    let mut hpal: HPALETTE;
    let ppal: PPALETTE;
    let mut i: ULONG;
    let cjSize: ULONG;

    ppal = PALETTE_AllocPalWithHandle(PAL_INDEXED as ULONG, cEntries as ULONG, core::ptr::null_mut(), 0, 0, 0);
    if ppal.is_null() {
        return core::ptr::null_mut();
    }

    cjSize = (core::mem::size_of::<LOGPALETTE>() as ULONG)
        - (core::mem::size_of::<PALETTEENTRY>() as ULONG)
        + (cEntries * core::mem::size_of::<PALETTEENTRY>() as ULONG);

    {
        let __SehStatus: NTSTATUS = SafeProbeForRead(plogpalUser as *mut core::ffi::c_void, cjSize as SIZE_T, 1);
        if !NT_SUCCESS(__SehStatus) {
            GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*ppal).BaseObject));
            return core::ptr::null_mut();
        }
    }

    // Probe is validate-only (USE_DUMMY_PSEH); the C wrapped probe AND this whole
    // loop in one _SEH2_TRY. One bulk capture replaces cEntries raw loads.
    if !crate::copy::capture_from_user(
        (*ppal).IndexedColors as *mut core::ffi::c_void,
        core::ptr::addr_of!((*plogpalUser).palPalEntry) as *const core::ffi::c_void,
        cEntries as usize * core::mem::size_of::<PALETTEENTRY>(),
    ) {
        GDIOBJ_vDeleteObject(core::ptr::addr_of_mut!((*ppal).BaseObject));
        return core::ptr::null_mut();
    }

    PALETTE_ValidateFlags((*ppal).IndexedColors, cEntries as INT);
    hpal = (*ppal).BaseObject.hHmgr as HPALETTE;
    PALETTE_UnlockPalette(ppal);

    hpal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateHalftonePalette(hDC: HDC) -> HPALETTE {
    let mut PalEntries: [PALETTEENTRY; 256] = [core::mem::zeroed(); 256];
    let mut hpal: HPALETTE = core::ptr::null_mut();

    let pdc: PDC = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return core::ptr::null_mut();
    }

    RtlZeroMemory(
        PalEntries.as_mut_ptr() as *mut _,
        core::mem::size_of::<[PALETTEENTRY; 256]>(),
    );

    let mut i: i32;

    i = 0;
    while i < 10 {
        PalEntries[i as usize].peRed = g_sysPalTemplate[i as usize].peRed;
        PalEntries[i as usize].peGreen = g_sysPalTemplate[i as usize].peGreen;
        PalEntries[i as usize].peBlue = g_sysPalTemplate[i as usize].peBlue;

        PalEntries[(246 + i) as usize].peRed = g_sysPalTemplate[(10 + i) as usize].peRed;
        PalEntries[(246 + i) as usize].peGreen = g_sysPalTemplate[(10 + i) as usize].peGreen;
        PalEntries[(246 + i) as usize].peBlue = g_sysPalTemplate[(10 + i) as usize].peBlue;
        i += 1;
    }

    let ppal: PPALETTE = PALETTE_ShareLockPalette((*pdc).dclevel.hpal);
    if !ppal.is_null() && ((*ppal).flFlags & PAL_INDEXED) != 0 {
        // FIXME: optimize the palette for the current palette
        // UNIMPLEMENTED
    } else {
        let mut r: i32 = 0;
        while r < 6 {
            let mut g: i32 = 0;
            while g < 6 {
                let mut b: i32 = 0;
                while b < 6 {
                    i = r + g * 6 + b * 36 + 10;
                    PalEntries[i as usize].peRed = (r * 51) as u8;
                    PalEntries[i as usize].peGreen = (g * 51) as u8;
                    PalEntries[i as usize].peBlue = (b * 51) as u8;
                    b += 1;
                }
                g += 1;
            }
            r += 1;
        }

        i = 216;
        while i < 246 {
            let v: i32 = (i - 216) << 3;
            PalEntries[i as usize].peRed = v as u8;
            PalEntries[i as usize].peGreen = v as u8;
            PalEntries[i as usize].peBlue = v as u8;
            i += 1;
        }
    }

    if !ppal.is_null() {
        PALETTE_ShareUnlockPalette(ppal);
    }

    DC_UnlockDc(pdc);

    let ppal2: PPALETTE = PALETTE_AllocPalWithHandle(
        PAL_INDEXED,
        256,
        PalEntries.as_ptr(),
        0,
        0,
        0,
    );
    if !ppal2.is_null() {
        hpal = (*ppal2).BaseObject.hHmgr as HPALETTE;
        PALETTE_UnlockPalette(ppal2);
    }

    hpal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiResizePalette(hpal: HPALETTE, Entries: UINT) -> BOOL {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetColorAdjustment(
    hdc: HDC,
    pca: LPCOLORADJUSTMENT,
) -> u8 {
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetColorAdjustment(
    hdc: HDC,
    pca: LPCOLORADJUSTMENT,
) -> u8 {
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetNearestColor(hDC: HDC, mut Color: COLORREF) -> COLORREF {

    let mut nearest: COLORREF = CLR_INVALID;
    let dc: PDC;
    let mut exlo: EXLATEOBJ = core::mem::zeroed();
    let ppal: PPALETTE;

    dc = DC_LockDc(hDC);

    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return CLR_INVALID;
    }

    if (*dc).dclevel.pSurface.is_null() {
        ppal = gppalMono;
    } else {
        ppal = (*(*dc).dclevel.pSurface).ppal;
    }

    Color = TranslateCOLORREF(dc, Color);

    EXLATEOBJ_vInitialize(
        core::ptr::addr_of_mut!(exlo),
        ppal,
        core::ptr::addr_of_mut!(gpalRGB),
        0,
        (0xffu32) | (0xffu32 << 8) | (0xffu32 << 16),
        0,
    );

    nearest = XLATEOBJ_iXlate(core::ptr::addr_of_mut!(exlo.xlo), Color);

    EXLATEOBJ_vCleanup(core::ptr::addr_of_mut!(exlo));

    DC_UnlockDc(dc);

    nearest
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetNearestPaletteIndex(hpal: HPALETTE, crColor: COLORREF) -> UINT {
    let ppal: PPALETTE = PALETTE_ShareLockPalette(hpal);
    let mut index: UINT = 0;

    if !ppal.is_null() {
        if (*ppal).flFlags & PAL_INDEXED != 0 {
            index = PALETTE_ulGetNearestPaletteIndex(ppal, crColor);
        }
        PALETTE_ShareUnlockPalette(ppal);
    }

    index
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGdiRealizePalette(hDC: HDC) -> UINT {
    IntGdiRealizePaletteImpl(hDC)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGdiRealizePalette(hDC: HDC) -> UINT {
    IntGdiRealizePaletteImpl(hDC)
}

unsafe fn IntGdiRealizePaletteImpl(hDC: HDC) -> UINT {
    let realize: UINT = 0;
    let pdc: PDC;
    let ppalSurf: *mut PALETTE;
    let ppalDC: *mut PALETTE;

    pdc = DC_LockDc(hDC);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    'cleanup: {
        if (*pdc).dclevel.pSurface.is_null() {
            break 'cleanup;
        }

        if (*pdc).dctype as u32 == DCTYPE_DIRECT as u32 {
            break 'cleanup;
        }

        ppalSurf = (*(*pdc).dclevel.pSurface).ppal;
        ppalDC = (*pdc).dclevel.ppal;

        if ((*ppalSurf).flFlags & PAL_INDEXED as u32) == 0 {
            break 'cleanup;
        }

        let _ = ppalDC;
    }

    DC_UnlockDc(pdc);
    realize
}

#[no_mangle]
pub unsafe extern "system" fn IntAnimatePalette(
    hPal: HPALETTE,
    mut StartIndex: UINT,
    mut NumEntries: UINT,
    PaletteColors: PPALETTEENTRY,
) -> UINT {
    let mut ret: UINT = 0;

    if hPal as HGDIOBJ != NtGdiGetStockObject(DEFAULT_PALETTE as INT) {
        let palPtr: PPALETTE;
        let pal_entries: UINT;
        let mut pptr: *const PALETTEENTRY = PaletteColors;

        palPtr = PALETTE_ShareLockPalette(hPal);
        if palPtr.is_null() {
            return 0;
        }

        pal_entries = (*palPtr).NumColors;
        if StartIndex >= pal_entries {
            PALETTE_ShareUnlockPalette(palPtr);
            return 0;
        }
        if StartIndex + NumEntries > pal_entries {
            NumEntries = pal_entries - StartIndex;
        }

        NumEntries += StartIndex;
        while StartIndex < NumEntries {
            if (*(*palPtr).IndexedColors.offset(StartIndex as isize)).peFlags & (PC_RESERVED as u8) != 0 {
                memcpy(
                    (*palPtr).IndexedColors.offset(StartIndex as isize) as *mut _,
                    pptr as *const _,
                    core::mem::size_of::<PALETTEENTRY>() as _,
                );
                ret += 1;
                PALETTE_ValidateFlags((*palPtr).IndexedColors.offset(StartIndex as isize), 1);
            }
            StartIndex += 1;
            pptr = pptr.offset(1);
        }

        PALETTE_ShareUnlockPalette(palPtr);
    }
    ret
}
#[no_mangle]
pub unsafe extern "system" fn IntGetPaletteEntries(
    hpal: HPALETTE,
    StartIndex: u32,
    mut Entries: u32,
    pe: LPPALETTEENTRY,
) -> u32 {

    let palGDI: PPALETTE = PALETTE_ShareLockPalette(hpal);
    if palGDI.is_null() {
        return 0;
    }

    let numEntries: u32 = (*palGDI).NumColors;
    if !pe.is_null() {
        if numEntries < StartIndex + Entries {
            Entries = numEntries - StartIndex;
        }
        if numEntries <= StartIndex {
            PALETTE_ShareUnlockPalette(palGDI);
            return 0;
        }
        memcpy(
            pe as *mut core::ffi::c_void,
            (*palGDI).IndexedColors.offset(StartIndex as isize) as *const core::ffi::c_void,
            (Entries as usize) * core::mem::size_of::<PALETTEENTRY>(),
        );
    } else {
        Entries = numEntries;
    }

    PALETTE_ShareUnlockPalette(palGDI);
    Entries
}
#[no_mangle]
pub unsafe extern "system" fn IntGetSystemPaletteEntries(
    hDC: HDC,
    StartIndex: UINT,
    mut Entries: UINT,
    pe: LPPALETTEENTRY,
) -> UINT {

    let mut palGDI: PPALETTE = core::ptr::null_mut();
    let mut dc: PDC = core::ptr::null_mut();
    let mut EntriesSize: UINT = 0;
    let mut Ret: UINT = 0;

    if Entries == 0 {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if !pe.is_null() {
        EntriesSize = Entries * core::mem::size_of::<PALETTEENTRY>() as UINT;
        if Entries != EntriesSize / core::mem::size_of::<PALETTEENTRY>() as UINT {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    palGDI = PALETTE_ShareLockPalette((*dc).dclevel.hpal);
    if !palGDI.is_null() {
        if !pe.is_null() {
            if StartIndex >= (*palGDI).NumColors {
                Entries = 0;
            } else if Entries > (*palGDI).NumColors - StartIndex {
                Entries = (*palGDI).NumColors - StartIndex;
            }

            memcpy(
                pe as *mut core::ffi::c_void,
                (*palGDI).IndexedColors.add(StartIndex as usize) as *const core::ffi::c_void,
                (Entries as usize) * core::mem::size_of::<PALETTEENTRY>(),
            );

            Ret = Entries;
        } else {
            Ret = (*(*dc).ppdev).gdiinfo.ulNumPalReg;
        }
    }

    if !palGDI.is_null() {
        PALETTE_ShareUnlockPalette(palGDI);
    }

    if !dc.is_null() {
        DC_UnlockDc(dc);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn IntSetPaletteEntries(
    hpal: HPALETTE,
    Start: UINT,
    mut Entries: UINT,
    pe: LPPALETTEENTRY,
) -> UINT {
    let palGDI: PPALETTE;
    let numEntries: ULONG;

    if GDI_HANDLE_IS_STOCKOBJ(hpal as HGDIOBJ) != 0 {
        return 0;
    }

    palGDI = PALETTE_ShareLockPalette(hpal);
    if palGDI.is_null() {
        return 0;
    }

    numEntries = (*palGDI).NumColors;
    if Start >= numEntries {
        PALETTE_ShareUnlockPalette(palGDI);
        return 0;
    }
    if numEntries < Start + Entries {
        Entries = numEntries - Start;
    }
    memcpy(
        (*palGDI).IndexedColors.add(Start as usize) as *mut _,
        pe as *const _,
        (Entries as usize) * core::mem::size_of::<PALETTEENTRY>(),
    );
    PALETTE_ShareUnlockPalette(palGDI);

    Entries
}

#[no_mangle]
pub unsafe extern "system" fn GreGetSetColorTable(
    hdc: HDC,
    iStartIndex: ULONG,
    cEntries: ULONG,
    mut prgbColors: *mut RGBQUAD,
    bSet: BOOL,
) -> ULONG {
    let pdc: PDC;
    let psurf: PSURFACE;
    let mut ppal: PPALETTE = core::ptr::null_mut();
    let mut iResult: ULONG = 0;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        return 0;
    }

    psurf = (*pdc).dclevel.pSurface;

    if psurf.is_null() {
        if bSet == 0 {
            ppal = gppalMono;
        }
    } else if (*psurf).SurfObj.iType as u32 == STYPE_BITMAP {
        ppal = (*psurf).ppal;
    }

    if !ppal.is_null()
        && ((*ppal).flFlags as u32 & PAL_INDEXED as u32) != 0
        && iStartIndex < (*ppal).NumColors
    {
        let sum = iStartIndex + cEntries;
        let iEndIndex = if sum < (*ppal).NumColors { sum } else { (*ppal).NumColors };

        if bSet != 0 {
            let mut i = iStartIndex;
            while i < iEndIndex {
                (*ppal).IndexedColors.add(i as usize).as_mut().unwrap().peRed = (*prgbColors).rgbRed;
                (*ppal).IndexedColors.add(i as usize).as_mut().unwrap().peGreen = (*prgbColors).rgbGreen;
                (*ppal).IndexedColors.add(i as usize).as_mut().unwrap().peBlue = (*prgbColors).rgbBlue;
                prgbColors = prgbColors.add(1);
                i += 1;
            }

            (*(*pdc).pdcattr).ulDirty_ |= (DIRTY_FILL | DIRTY_LINE | DIRTY_BACKGROUND | DIRTY_TEXT) as ULONG;
        } else {
            let mut i = iStartIndex;
            while i < iEndIndex {
                let entry = (*ppal).IndexedColors.add(i as usize);
                (*prgbColors).rgbRed = (*entry).peRed;
                (*prgbColors).rgbGreen = (*entry).peGreen;
                (*prgbColors).rgbBlue = (*entry).peBlue;
                (*prgbColors).rgbReserved = 0;
                prgbColors = prgbColors.add(1);
                i += 1;
            }
        }

        iResult = iEndIndex - iStartIndex;
    }

    DC_UnlockDc(pdc);

    iResult
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDoPalette(
    hObj: HGDIOBJ,
    iStart: WORD,
    cEntries: WORD,
    pUnsafeEntries: LPVOID,
    iFunc: DWORD,
    bInbound: BOOL,
) -> LONG {

    let mut ret: LONG;
    let mut pEntries: LPVOID = core::ptr::null_mut();
    let mut cjSize: SIZE_T;
    let Status: NTSTATUS;

    if !pUnsafeEntries.is_null() {
        if cEntries == 0 {
            return 0;
        }

        cjSize = (cEntries as SIZE_T) * (core::mem::size_of::<PALETTEENTRY>() as SIZE_T);
        pEntries = ExAllocatePoolWithTag(PagedPool, cjSize, TAG_PALETTE);
        if pEntries.is_null() {
            return 0;
        }

        if bInbound != 0 {
            let __SehStatus: NTSTATUS = SafeProbeForRead(pUnsafeEntries, cjSize as SIZE_T, 1);
            if !NT_SUCCESS(__SehStatus) {
                ExFreePoolWithTag(pEntries, TAG_PALETTE);
                return 0;
            }
            memcpy(pEntries, pUnsafeEntries, cjSize as usize);
        } else {
            RtlZeroMemory(pEntries, cjSize as usize);
        }
    }

    ret = 0;
    match iFunc {
        GdiPalAnimate => {
            if !pEntries.is_null() {
                ret = IntAnimatePalette(hObj as HPALETTE, iStart as UINT, cEntries as UINT, pEntries as PPALETTEENTRY) as LONG;
            }
        }
        GdiPalSetEntries => {
            if !pEntries.is_null() {
                ret = IntSetPaletteEntries(hObj as HPALETTE, iStart as UINT, cEntries as UINT, pEntries as LPPALETTEENTRY) as LONG;
            }
        }
        GdiPalGetEntries => {
            ret = IntGetPaletteEntries(hObj as HPALETTE, iStart as u32, cEntries as u32, pEntries as LPPALETTEENTRY) as LONG;
        }
        GdiPalGetSystemEntries => {
            ret = IntGetSystemPaletteEntries(hObj as HDC, iStart as UINT, cEntries as UINT, pEntries as LPPALETTEENTRY) as LONG;
        }
        GdiPalSetColorTable => {
            if !pEntries.is_null() {
                ret = GreGetSetColorTable(hObj as HDC, iStart as ULONG, cEntries as ULONG, pEntries as *mut RGBQUAD, 1) as LONG;
            }
        }
        GdiPalGetColorTable => {
            if !pEntries.is_null() {
                ret = GreGetSetColorTable(hObj as HDC, iStart as ULONG, cEntries as ULONG, pEntries as *mut RGBQUAD, 0) as LONG;
            }
        }
        _ => {}
    }

    if !pEntries.is_null() {
        if bInbound == 0 && ret > 0 {
            let m = if (cEntries as LONG) < ret { cEntries as LONG } else { ret };
            cjSize = (m as SIZE_T) * (core::mem::size_of::<PALETTEENTRY>() as SIZE_T);
            Status = SafeProbeForWrite(pUnsafeEntries, cjSize as SIZE_T, 1);
            if !NT_SUCCESS(Status) {
                ret = 0;
            } else {
                memcpy(pUnsafeEntries, pEntries, cjSize as usize);
            }
        }
        ExFreePoolWithTag(pEntries, TAG_PALETTE);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetSystemPaletteUse(hDC: HDC, Usage: UINT) -> UINT {
    let mut old: UINT = SystemPaletteUse;

    if (NtGdiGetDeviceCaps(hDC, RASTERCAPS as INT) & RC_PALETTE as INT) == 0 {
        return SYSPAL_ERROR;
    }

    match Usage {
        SYSPAL_NOSTATIC | SYSPAL_NOSTATIC256 | SYSPAL_STATIC => {
            SystemPaletteUse = Usage;
        }
        _ => {
            old = SYSPAL_ERROR;
        }
    }

    old
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetSystemPaletteUse(hDC: HDC) -> UINT {
    SystemPaletteUse
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiUpdateColors(hDC: HDC) -> BOOL {
    let Wnd: PWND;
    let calledFromUser: BOOL;
    let ret: BOOL;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    calledFromUser = UserIsEntered();

    if calledFromUser == 0 {
        UserEnterExclusive();
    }

    Wnd = UserGetWindowObject(IntWindowFromDC(hDC));
    if Wnd.is_null() {
        EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);

        if calledFromUser == 0 {
            UserLeave();
        }

        return 0;
    }

    UserRefObjectCo(Wnd, &mut Ref);
    ret = co_UserRedrawWindow(Wnd, core::ptr::null_mut(), core::ptr::null_mut(), RDW_INVALIDATE);
    UserDerefObjectCo(Wnd);

    if calledFromUser == 0 {
        UserLeave();
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiUnrealizeObject(hgdiobj: HGDIOBJ) -> BOOL {
    let mut Ret: BOOL = 0;
    let palGDI: PPALETTE;

    if hgdiobj.is_null()
        || GDI_HANDLE_IS_STOCKOBJ(hgdiobj) != 0
        || GDI_HANDLE_IS_TYPE(hgdiobj, GDI_OBJECT_TYPE_PALETTE as _) == 0
    {
        return Ret;
    }

    palGDI = PALETTE_ShareLockPalette(hgdiobj as HPALETTE);
    if palGDI.is_null() {
        return 0;
    }

    Ret = 1;
    PALETTE_ShareUnlockPalette(palGDI);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngCreatePalette(
    iMode: ULONG,
    cColors: ULONG,
    pulColors: *mut ULONG,
    flRed: FLONG,
    flGreen: FLONG,
    flBlue: FLONG,
) -> HPALETTE {
    let mut hPal: HPALETTE = core::ptr::null_mut();
    let mut ulColors: [ULONG; WINDDI_MAXSETPALETTECOLORS as usize] =
        [0; WINDDI_MAXSETPALETTECOLORS as usize];
    let pulcSafe: *mut ULONG;

    const MAX_PALCOLORS: ULONG = 65536;
    const GDITAG_UMPD: ULONG = 0x44504d55;

    if cColors > MAX_PALCOLORS {
        return core::ptr::null_mut();
    }

    if cColors <= WINDDI_MAXSETPALETTECOLORS {
        pulcSafe = ulColors.as_mut_ptr();
    } else {
        pulcSafe = ExAllocatePoolWithTag(
            PagedPool,
            ((cColors as usize) * core::mem::size_of::<ULONG>()) as SIZE_T,
            GDITAG_UMPD,
        ) as *mut ULONG;
    }

    {
        let __SehStatus: NTSTATUS = SafeProbeForRead(
            pulColors as *mut core::ffi::c_void,
            ((cColors as usize) * core::mem::size_of::<ULONG>()) as SIZE_T,
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            SetLastNtError(__SehStatus);
            if cColors > WINDDI_MAXSETPALETTECOLORS {
                ExFreePoolWithTag(pulcSafe as *mut core::ffi::c_void, GDITAG_UMPD);
            }
            return hPal;
        }
    }

    RtlCopyMemory(
        pulcSafe as *mut core::ffi::c_void,
        pulColors as *const core::ffi::c_void,
        (cColors as usize) * core::mem::size_of::<ULONG>(),
    );

    hPal = EngCreatePalette(iMode, cColors, pulcSafe, flRed, flGreen, flBlue);

    if cColors > WINDDI_MAXSETPALETTECOLORS {
        ExFreePoolWithTag(pulcSafe as *mut core::ffi::c_void, GDITAG_UMPD);
    }

    hPal
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEngDeletePalette(hPal: HPALETTE) -> u32 {
    EngDeletePalette(hPal)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS Win32k subsystem
//  * PURPOSE:           GDI Palette Functions
//  * FILE:              win32ss/gdi/ntgdi/palette.c
//  * PROGRAMERS:        Jason Filby
//  *                    Timo Kreuzer
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #define PAL_SETPOWNER 0x8000
// #define MAX_PALCOLORS 65536
//
// static UINT SystemPaletteUse = SYSPAL_NOSTATIC;  /* The program need save the pallete and restore it */
//
// PALETTE gpalRGB, gpalBGR, gpalRGB555, gpalRGB565, *gppalMono, *gppalDefault;
// PPALETTE appalSurfaceDefault[11];
//
// const PALETTEENTRY g_sysPalTemplate[NB_RESERVED_COLORS] =
// {
//   // First 10 entries in the system palette
//   // Red  Green Blue  Flags
//   { 0x00, 0x00, 0x00, PC_SYS_USED },
//   { 0x80, 0x00, 0x00, PC_SYS_USED },
//   { 0x00, 0x80, 0x00, PC_SYS_USED },
//   { 0x80, 0x80, 0x00, PC_SYS_USED },
//   { 0x00, 0x00, 0x80, PC_SYS_USED },
//   { 0x80, 0x00, 0x80, PC_SYS_USED },
//   { 0x00, 0x80, 0x80, PC_SYS_USED },
//   { 0xc0, 0xc0, 0xc0, PC_SYS_USED },
//   { 0xc0, 0xdc, 0xc0, PC_SYS_USED },
//   { 0xa6, 0xca, 0xf0, PC_SYS_USED },
//
//   // ... c_min/2 dynamic colorcells
//   // ... gap (for sparse palettes)
//   // ... c_min/2 dynamic colorcells
//
//   { 0xff, 0xfb, 0xf0, PC_SYS_USED },
//   { 0xa0, 0xa0, 0xa4, PC_SYS_USED },
//   { 0x80, 0x80, 0x80, PC_SYS_USED },
//   { 0xff, 0x00, 0x00, PC_SYS_USED },
//   { 0x00, 0xff, 0x00, PC_SYS_USED },
//   { 0xff, 0xff, 0x00, PC_SYS_USED },
//   { 0x00, 0x00, 0xff, PC_SYS_USED },
//   { 0xff, 0x00, 0xff, PC_SYS_USED },
//   { 0x00, 0xff, 0xff, PC_SYS_USED },
//   { 0xff, 0xff, 0xff, PC_SYS_USED }     // Last 10
// };
//
// unsigned short GetNumberOfBits(unsigned int dwMask)
// {
//    unsigned short wBits;
//    for (wBits = 0; dwMask; dwMask = dwMask & (dwMask - 1))
//       wBits++;
//    return wBits;
// }
//
// // Create the system palette
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitPaletteImpl(VOID)
// {
//     // Create default palette (20 system colors)
//     gppalDefault = PALETTE_AllocPalWithHandle(PAL_INDEXED,
//                                               20,
//                                               g_sysPalTemplate,
//                                               0, 0, 0);
//     GDIOBJ_vReferenceObjectByPointer(&gppalDefault->BaseObject);
//     PALETTE_UnlockPalette(gppalDefault);
//
//     /*  palette_size = visual->map_entries; */
//
//     gpalRGB.flFlags = PAL_RGB;
//     gpalRGB.RedMask = RGB(0xFF, 0x00, 0x00);
//     gpalRGB.GreenMask = RGB(0x00, 0xFF, 0x00);
//     gpalRGB.BlueMask = RGB(0x00, 0x00, 0xFF);
//     gpalRGB.BaseObject.__bindgen_anon_1.ulShareCount = 1;
//     gpalRGB.BaseObject.BaseFlags = 0 ;
//
//     gpalBGR.flFlags = PAL_BGR;
//     gpalBGR.RedMask = RGB(0x00, 0x00, 0xFF);
//     gpalBGR.GreenMask = RGB(0x00, 0xFF, 0x00);
//     gpalBGR.BlueMask = RGB(0xFF, 0x00, 0x00);
//     gpalBGR.BaseObject.__bindgen_anon_1.ulShareCount = 1;
//     gpalBGR.BaseObject.BaseFlags = 0 ;
//
//     gpalRGB555.flFlags = PAL_RGB16_555 | PAL_BITFIELDS;
//     gpalRGB555.RedMask = 0x7C00;
//     gpalRGB555.GreenMask = 0x3E0;
//     gpalRGB555.BlueMask = 0x1F;
//     gpalRGB555.BaseObject.__bindgen_anon_1.ulShareCount = 1;
//     gpalRGB555.BaseObject.BaseFlags = 0 ;
//
//     gpalRGB565.flFlags = PAL_RGB16_565 | PAL_BITFIELDS;
//     gpalRGB565.RedMask = 0xF800;
//     gpalRGB565.GreenMask = 0x7E0;
//     gpalRGB565.BlueMask = 0x1F;
//     gpalRGB565.BaseObject.__bindgen_anon_1.ulShareCount = 1;
//     gpalRGB565.BaseObject.BaseFlags = 0 ;
//
//     gppalMono = PALETTE_AllocPalette(PAL_MONOCHROME|PAL_INDEXED, 2, NULL, 0, 0, 0);
//     PALETTE_vSetRGBColorForIndex(gppalMono, 0, 0x000000);
//     PALETTE_vSetRGBColorForIndex(gppalMono, 1, 0xffffff);
//
//     /* Initialize default surface palettes */
//     appalSurfaceDefault[BMF_1BPP] = gppalMono;
//     appalSurfaceDefault[BMF_4BPP] = gppalDefault;
//     appalSurfaceDefault[BMF_8BPP] = gppalDefault;
//     appalSurfaceDefault[BMF_16BPP] = &gpalRGB565;
//     appalSurfaceDefault[BMF_24BPP] = &gpalBGR;
//     appalSurfaceDefault[BMF_32BPP] = &gpalBGR;
//     appalSurfaceDefault[BMF_4RLE] = gppalDefault;
//     appalSurfaceDefault[BMF_8RLE] = gppalDefault;
//     appalSurfaceDefault[BMF_JPEG] = &gpalRGB;
//     appalSurfaceDefault[BMF_PNG] = &gpalRGB;
//
//     return STATUS_SUCCESS;
// }
//
// VOID FASTCALL PALETTE_ValidateFlags(PALETTEENTRY* lpPalE, INT size)
// {
//     int i = 0;
//     for (; i<size ; i++)
//         lpPalE[i].peFlags = PC_SYS_USED | (lpPalE[i].peFlags & 0x07);
// }
//
//
// PPALETTE
// NTAPI
// PALETTE_AllocPalette(
//     _In_ ULONG iMode,
//     _In_ ULONG cColors,
//     _In_opt_ const PALETTEENTRY* pEntries,
//     _In_ FLONG flRed,
//     _In_ FLONG flGreen,
//     _In_ FLONG flBlue)
// {
//     PPALETTE ppal;
//     ULONG fl = 0, cjSize = sizeof(PALETTE);
//
//     /* Check if the palette has entries */
//     if (iMode & PAL_INDEXED)
//     {
//         /* Check color count */
//         if ((cColors == 0) || (cColors > 1024)) return NULL;
//
//         /* Mark 2 color indexed palettes as monochrome */
//         if (cColors == 2) iMode |= PAL_MONOCHROME;
//
//         /* Allocate enough space for the palete entries */
//         cjSize += cColors * sizeof(PALETTEENTRY);
//     }
//     else
//     {
//         /* There are no palette entries */
//         cColors = 0;
//
//         /* We can use the lookaside list */
//         fl |= BASEFLAG_LOOKASIDE;
//     }
//
//     /* Allocate the object (without a handle!) */
//     ppal = (PPALETTE)GDIOBJ_AllocateObject(GDIObjType_PAL_TYPE, cjSize, fl);
//     if (!ppal)
//     {
//         return NULL;
//     }
//
//     /* Set mode, color count and entry pointer */
//     ppal->flFlags = iMode;
//     ppal->NumColors = cColors;
//     ppal->IndexedColors = ppal->apalColors;
//
//     /* Check what kind of palette this is */
//     if (iMode & PAL_INDEXED)
//     {
//         /* Check if we got a color array */
//         if (pEntries)
//         {
//             /* Copy the entries */
//             RtlCopyMemory(ppal->IndexedColors, pEntries, cColors * sizeof(pEntries[0]));
//         }
//     }
//     else if (iMode & PAL_BITFIELDS)
//     {
//         /* Copy the color masks */
//         ppal->RedMask = flRed;
//         ppal->GreenMask = flGreen;
//         ppal->BlueMask = flBlue;
//
//         /* Check what masks we have and set optimization flags */
//         if ((flRed == 0x7c00) && (flGreen == 0x3E0) && (flBlue == 0x1F))
//             ppal->flFlags |= PAL_RGB16_555;
//         else if ((flRed == 0xF800) && (flGreen == 0x7E0) && (flBlue == 0x1F))
//             ppal->flFlags |= PAL_RGB16_565;
//         else if ((flRed == 0xFF0000) && (flGreen == 0xFF00) && (flBlue == 0xFF))
//             ppal->flFlags |= PAL_BGR;
//         else if ((flRed == 0xFF) && (flGreen == 0xFF00) && (flBlue == 0xFF0000))
//             ppal->flFlags |= PAL_RGB;
//     }
//
//     return ppal;
// }
//
// PPALETTE
// NTAPI
// PALETTE_AllocPalWithHandle(
//     _In_ ULONG iMode,
//     _In_ ULONG cColors,
//     _In_opt_ const PALETTEENTRY* pEntries,
//     _In_ FLONG flRed,
//     _In_ FLONG flGreen,
//     _In_ FLONG flBlue)
// {
//     PPALETTE ppal;
//
//     /* Allocate the palette without a handle */
//     ppal = PALETTE_AllocPalette(iMode, cColors, pEntries, flRed, flGreen, flBlue);
//     if (!ppal) return NULL;
//
//     /* Insert the palette into the handle table */
//     if (!GDIOBJ_hInsertObject(&ppal->BaseObject, GDI_OBJ_HMGR_POWNED))
//     {
//         DPRINT1("Could not insert palette into handle table.\n");
//         GDIOBJ_vFreeObject(&ppal->BaseObject);
//         return NULL;
//     }
//
//     return ppal;
// }
//
// VOID
// NTAPI
// PALETTE_vCleanup(PVOID ObjectBody)
// {
//     PPALETTE pPal = (PPALETTE)ObjectBody;
//     if (pPal->IndexedColors && pPal->IndexedColors != pPal->apalColors)
//     {
//         ExFreePoolWithTag(pPal->IndexedColors, TAG_PALETTE);
//     }
// }
//
// INT
// FASTCALL
// PALETTE_GetObject(PPALETTE ppal, INT cbCount, LPLOGBRUSH lpBuffer)
// {
//     if (!lpBuffer)
//     {
//         return sizeof(WORD);
//     }
//
//     if ((UINT)cbCount < sizeof(WORD)) return 0;
//     *((WORD*)lpBuffer) = (WORD)ppal->NumColors;
//     return sizeof(WORD);
// }
//
// ULONG
// NTAPI
// PALETTE_ulGetNearestPaletteIndex(PALETTE* ppal, ULONG iColor)
// {
//     ULONG ulDiff, ulColorDiff, ulMinimalDiff = 0xFFFFFF;
//     ULONG i, ulBestIndex = 0;
//     PALETTEENTRY peColor = *(PPALETTEENTRY)&iColor;
//     ASSERT(ppal->flFlags & PAL_INDEXED);
//
//     /* Loop all palette entries */
//     for (i = 0; i < ppal->NumColors; i++)
//     {
//         /* Calculate distance in the color cube */
//         ulDiff = peColor.peRed - ppal->IndexedColors[i].peRed;
//         ulColorDiff = ulDiff * ulDiff;
//         ulDiff = peColor.peGreen - ppal->IndexedColors[i].peGreen;
//         ulColorDiff += ulDiff * ulDiff;
//         ulDiff = peColor.peBlue - ppal->IndexedColors[i].peBlue;
//         ulColorDiff += ulDiff * ulDiff;
//
//         /* Check for a better match */
//         if (ulColorDiff < ulMinimalDiff)
//         {
//             ulBestIndex = i;
//             ulMinimalDiff = ulColorDiff;
//
//             /* Break on exact match */
//             if (ulMinimalDiff == 0) break;
//         }
//     }
//
//     return ulBestIndex;
// }
//
// ULONG
// NTAPI
// PALETTE_ulGetNearestBitFieldsIndex(PALETTE* ppal, ULONG ulColor)
// {
//     ULONG ulNewColor;
//     ASSERT(ppal->flFlags & PAL_BITFIELDS);
//
//     // FIXME: HACK, should be stored already
//     ppal->ulRedShift = CalculateShift(RGB(0xff,0,0), ppal->RedMask);
//     ppal->ulGreenShift = CalculateShift(RGB(0,0xff,0), ppal->GreenMask);
//     ppal->ulBlueShift = CalculateShift(RGB(0,0,0xff), ppal->BlueMask);
//
//     ulNewColor = _rotl(ulColor, ppal->ulRedShift) & ppal->RedMask;
//     ulNewColor |= _rotl(ulColor, ppal->ulGreenShift) & ppal->GreenMask;
//     ulNewColor |= _rotl(ulColor, ppal->ulBlueShift) & ppal->BlueMask;
//
//    return ulNewColor;
// }
//
// ULONG
// NTAPI
// PALETTE_ulGetNearestIndex(PALETTE* ppal, ULONG ulColor)
// {
//     if (ppal->flFlags & PAL_RGB)
//         return ulColor;
//     if (ppal->flFlags & PAL_BGR)
//         return RGB(GetBValue(ulColor), GetGValue(ulColor), GetRValue(ulColor));
//     if (ppal->flFlags & PAL_INDEXED) // Use fl & PALINDEXED
//         return PALETTE_ulGetNearestPaletteIndex(ppal, ulColor);
//     else
//         return PALETTE_ulGetNearestBitFieldsIndex(ppal, ulColor);
// }
//
// VOID
// NTAPI
// PALETTE_vGetBitMasks(PPALETTE ppal, PULONG pulColors)
// {
//     ASSERT(pulColors);
//
//     if (ppal->flFlags & PAL_INDEXED || ppal->flFlags & PAL_RGB)
//     {
//         pulColors[0] = RGB(0xFF, 0x00, 0x00);
//         pulColors[1] = RGB(0x00, 0xFF, 0x00);
//         pulColors[2] = RGB(0x00, 0x00, 0xFF);
//     }
//     else if (ppal->flFlags & PAL_BGR)
//     {
//         pulColors[0] = RGB(0x00, 0x00, 0xFF);
//         pulColors[1] = RGB(0x00, 0xFF, 0x00);
//         pulColors[2] = RGB(0xFF, 0x00, 0x00);
//     }
//     else if (ppal->flFlags & PAL_BITFIELDS)
//     {
//         pulColors[0] = ppal->RedMask;
//         pulColors[1] = ppal->GreenMask;
//         pulColors[2] = ppal->BlueMask;
//     }
// }
//
// VOID
// FASTCALL
// ColorCorrection(PPALETTE PalGDI, PPALETTEENTRY PaletteEntry, ULONG Colors)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)PalGDI->hPDev;
//
//     if (!ppdev) return;
//
//     if (ppdev->flFlags & PDEV_GAMMARAMP_TABLE)
//     {
//         ULONG i;
//         PGAMMARAMP GammaRamp = (PGAMMARAMP)ppdev->pvGammaRamp;
//         for ( i = 0; i < Colors; i++)
//         {
//             PaletteEntry[i].peRed   += GammaRamp->Red[i];
//             PaletteEntry[i].peGreen += GammaRamp->Green[i];
//             PaletteEntry[i].peBlue  += GammaRamp->Blue[i];
//         }
//     }
//     return;
// }
//
// /** Display Driver Interface **************************************************/
//
// /*
//  * @implemented
//  */
// HPALETTE
// APIENTRY
// EngCreatePalette(
//     ULONG iMode,
//     ULONG cColors,
//     ULONG *pulColors,
//     ULONG flRed,
//     ULONG flGreen,
//     ULONG flBlue)
// {
//     PPALETTE ppal;
//     HPALETTE hpal;
//
//     ppal = PALETTE_AllocPalette(iMode, cColors, (PPALETTEENTRY)pulColors, flRed, flGreen, flBlue);
//     if (!ppal) return NULL;
//
//     hpal = GDIOBJ_hInsertObject(&ppal->BaseObject, GDI_OBJ_HMGR_PUBLIC);
//     if (!hpal)
//     {
//         DPRINT1("Could not insert palette into handle table.\n");
//         GDIOBJ_vFreeObject(&ppal->BaseObject);
//         return NULL;
//     }
//
//     PALETTE_UnlockPalette(ppal);
//     return hpal;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// EngDeletePalette(IN HPALETTE hpal)
// {
//     PPALETTE ppal;
//
//     ppal = PALETTE_ShareLockPalette(hpal);
//     if (!ppal) return FALSE;
//
//     GDIOBJ_vDeleteObject(&ppal->BaseObject);
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// ULONG
// APIENTRY
// PALOBJ_cGetColors(PALOBJ *PalObj, ULONG Start, ULONG Colors, ULONG *PaletteEntry)
// {
//     PALETTE *PalGDI;
//
//     PalGDI = (PALETTE*)PalObj;
//
//     if (Start >= PalGDI->NumColors)
//         return 0;
//
//     Colors = min(Colors, PalGDI->NumColors - Start);
//
//     /* NOTE: PaletteEntry ULONGs are in the same order as PALETTEENTRY. */
//     RtlCopyMemory(PaletteEntry, PalGDI->IndexedColors + Start, sizeof(ULONG) * Colors);
//
//     if (PalGDI->flFlags & PAL_GAMMACORRECTION)
//         ColorCorrection(PalGDI, (PPALETTEENTRY)PaletteEntry, Colors);
//
//     return Colors;
// }
//
//
// /** Systemcall Interface ******************************************************/
//
// HPALETTE
// NTAPI
// GreCreatePaletteInternal(
//     IN LPLOGPALETTE pLogPal,
//     IN UINT cEntries)
// {
//     HPALETTE hpal = NULL;
//     PPALETTE ppal;
//
//     pLogPal->palNumEntries = cEntries;
//     ppal = PALETTE_AllocPalWithHandle(PAL_INDEXED,
//                                       cEntries,
//                                       pLogPal->palPalEntry,
//                                       0, 0, 0);
//
//     if (ppal != NULL)
//     {
//         PALETTE_ValidateFlags(ppal->IndexedColors, ppal->NumColors);
//
//         hpal = ppal->BaseObject.hHmgr;
//         PALETTE_UnlockPalette(ppal);
//     }
//
//     return hpal;
// }
//
// /*
//  * @implemented
//  */
// HPALETTE
// APIENTRY
// NtGdiCreatePaletteInternal(
//     IN LPLOGPALETTE plogpalUser,
//     IN UINT cEntries)
// {
//     HPALETTE hpal = NULL;
//     PPALETTE ppal;
//     ULONG i, cjSize;
//
//     ppal = PALETTE_AllocPalWithHandle(PAL_INDEXED, cEntries, NULL, 0, 0, 0);
//     if (ppal == NULL)
//     {
//         return NULL;
//     }
//
//     cjSize = FIELD_OFFSET(LOGPALETTE, palPalEntry[cEntries]);
//
//     {
//         NTSTATUS __SehStatus = SafeProbeForRead(plogpalUser, cjSize as SIZE_T, 1);
//         if (!NT_SUCCESS(__SehStatus))
//         {
//             GDIOBJ_vDeleteObject(&ppal->BaseObject);
//             return NULL;
//         }
//     }
//     for (i = 0; i < cEntries; i++)
//     {
//         ppal->IndexedColors[i] = plogpalUser->palPalEntry[i];
//     }
//
//     PALETTE_ValidateFlags(ppal->IndexedColors, cEntries);
//     hpal = ppal->BaseObject.hHmgr;
//     PALETTE_UnlockPalette(ppal);
//
//     return hpal;
// }
//
// HPALETTE
// APIENTRY
// NtGdiCreateHalftonePalette(HDC  hDC)
// {
//     int i, r, g, b;
//     PALETTEENTRY PalEntries[256];
//     PPALETTE ppal;
//     PDC pdc;
//     HPALETTE hpal = NULL;
//
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return NULL;
//     }
//
//     RtlZeroMemory(PalEntries, sizeof(PalEntries));
//
//     /* First and last ten entries are default ones */
//     for (i = 0; i < 10; i++)
//     {
//         PalEntries[i].peRed = g_sysPalTemplate[i].peRed;
//         PalEntries[i].peGreen = g_sysPalTemplate[i].peGreen;
//         PalEntries[i].peBlue = g_sysPalTemplate[i].peBlue;
//
//         PalEntries[246 + i].peRed = g_sysPalTemplate[10 + i].peRed;
//         PalEntries[246 + i].peGreen = g_sysPalTemplate[10 + i].peGreen;
//         PalEntries[246 + i].peBlue = g_sysPalTemplate[10 + i].peBlue;
//     }
//
//     ppal = PALETTE_ShareLockPalette(pdc->dclevel.hpal);
//     if (ppal && (ppal->flFlags & PAL_INDEXED))
//     {
//         /* FIXME: optimize the palette for the current palette */
//         UNIMPLEMENTED;
//     }
//     else
//     {
//         for (r = 0; r < 6; r++)
//         {
//             for (g = 0; g < 6; g++)
//             {
//                 for (b = 0; b < 6; b++)
//                 {
//                     i = r + g*6 + b*36 + 10;
//                     PalEntries[i].peRed = r * 51;
//                     PalEntries[i].peGreen = g * 51;
//                     PalEntries[i].peBlue = b * 51;
//                 }
//             }
//         }
//
//         for (i = 216; i < 246; i++)
//         {
//             int v = (i - 216) << 3;
//             PalEntries[i].peRed = v;
//             PalEntries[i].peGreen = v;
//             PalEntries[i].peBlue = v;
//         }
//     }
//
//     if (ppal)
//         PALETTE_ShareUnlockPalette(ppal);
//
//     DC_UnlockDc(pdc);
//
//     ppal = PALETTE_AllocPalWithHandle(PAL_INDEXED, 256, PalEntries, 0, 0, 0);
//     if (ppal)
//     {
//         hpal = ppal->BaseObject.hHmgr;
//         PALETTE_UnlockPalette(ppal);
//     }
//
//     return hpal;
// }
//
// BOOL
// APIENTRY
// NtGdiResizePalette(
//     HPALETTE hpal,
//     UINT Entries)
// {
// /*  PALOBJ *palPtr = (PALOBJ*)AccessUserObject(hPal);
//   UINT cPrevEnt, prevVer;
//   INT prevsize, size = sizeof(LOGPALETTE) + (cEntries - 1) * sizeof(PALETTEENTRY);
//   XLATEOBJ *XlateObj = NULL;
//
//   if(!palPtr) return FALSE;
//   cPrevEnt = palPtr->logpalette->palNumEntries;
//   prevVer = palPtr->logpalette->palVersion;
//   prevsize = sizeof(LOGPALETTE) + (cPrevEnt - 1) * sizeof(PALETTEENTRY) + sizeof(int*) + sizeof(GDIOBJHDR);
//   size += sizeof(int*) + sizeof(GDIOBJHDR);
//   XlateObj = palPtr->logicalToSystem;
//
//   if (!(palPtr = GDI_ReallocObject(size, hPal, palPtr))) return FALSE;
//
//   if(XlateObj)
//   {
//     XLATEOBJ *NewXlateObj = (int*) HeapReAlloc(GetProcessHeap(), 0, XlateObj, cEntries * sizeof(int));
//     if(NewXlateObj == NULL)
//     {
//       ERR("Can not resize logicalToSystem -- out of memory!\n");
//       GDI_ReleaseObj( hPal );
//       return FALSE;
//     }
//     palPtr->logicalToSystem = NewXlateObj;
//   }
//
//   if(cEntries > cPrevEnt)
//   {
//     if(XlateObj) memset(palPtr->logicalToSystem + cPrevEnt, 0, (cEntries - cPrevEnt)*sizeof(int));
//     memset( (BYTE*)palPtr + prevsize, 0, size - prevsize );
//     PALETTE_ValidateFlags((PALETTEENTRY*)((BYTE*)palPtr + prevsize), cEntries - cPrevEnt );
//   }
//   palPtr->logpalette->palNumEntries = cEntries;
//   palPtr->logpalette->palVersion = prevVer;
// //    GDI_ReleaseObj( hPal );
//   return TRUE; */
//
//   UNIMPLEMENTED;
//   return FALSE;
// }
//
// BOOL
// APIENTRY
// NtGdiGetColorAdjustment(
//     HDC hdc,
//     LPCOLORADJUSTMENT pca)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// BOOL
// APIENTRY
// NtGdiSetColorAdjustment(
//     HDC hdc,
//     LPCOLORADJUSTMENT pca)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// COLORREF
// APIENTRY
// NtGdiGetNearestColor(
//     _In_ HDC hDC,
//     _In_ COLORREF Color)
// {
//     COLORREF nearest = CLR_INVALID;
//     PDC dc;
//     EXLATEOBJ exlo;
//     PPALETTE ppal;
//
//     dc = DC_LockDc(hDC);
//
//     if(dc == NULL)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return CLR_INVALID;
//     }
//
//     /// FIXME: shouldn't dereference pSurface while the PDEV is not locked
//     if(dc->dclevel.pSurface == NULL)
//         ppal = gppalMono;
//     else
//         ppal = dc->dclevel.pSurface->ppal;
//
//     /* Translate the color to the DC format */
//     Color = TranslateCOLORREF(dc, Color);
//
//     /* XLATE it back to RGB color space */
//     EXLATEOBJ_vInitialize(&exlo,
//         ppal,
//         &gpalRGB,
//         0,
//         RGB(0xff, 0xff, 0xff),
//         RGB(0, 0, 0));
//
//     nearest = XLATEOBJ_iXlate(&exlo.xlo, Color);
//
//     EXLATEOBJ_vCleanup(&exlo);
//
//     /* We're done */
//     DC_UnlockDc(dc);
//
//    return nearest;
// }
//
// UINT
// APIENTRY
// NtGdiGetNearestPaletteIndex(
//     HPALETTE hpal,
//     COLORREF crColor)
// {
//     PPALETTE ppal = PALETTE_ShareLockPalette(hpal);
//     UINT index  = 0;
//
//     if (ppal)
//     {
//         if (ppal->flFlags & PAL_INDEXED)
//         {
//             /* Return closest match for the given RGB color */
//             index = PALETTE_ulGetNearestPaletteIndex(ppal, crColor);
//         }
//         // else SetLastError ?
//         PALETTE_ShareUnlockPalette(ppal);
//     }
//
//     return index;
// }
//
// UINT
// FASTCALL
// IntGdiRealizePalette(HDC hDC)
// {
//     UINT realize = 0;
//     PDC pdc;
//     PALETTE *ppalSurf, *ppalDC;
//
//     pdc = DC_LockDc(hDC);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     if (!pdc->dclevel.pSurface)
//     {
//         goto cleanup;
//     }
//
//     if (pdc->dctype == DCTYPE_DIRECT)
//     {
//         static BOOL g_WarnedOnce = FALSE;
//         if (!g_WarnedOnce)
//         {
//             g_WarnedOnce = TRUE;
//             UNIMPLEMENTED;
//         }
//         goto cleanup;
//     }
//
//     /// FIXME: shouldn't dereference pSurface while the PDEV is not locked
//     ppalSurf = pdc->dclevel.pSurface->ppal;
//     ppalDC = pdc->dclevel.ppal;
//
//     if (!(ppalSurf->flFlags & PAL_INDEXED))
//     {
//         // FIXME: Set error?
//         goto cleanup;
//     }
//
//     ASSERT(ppalDC->flFlags & PAL_INDEXED);
//
//     DPRINT1("RealizePalette unimplemented for %s\n", 
//             (pdc->dctype == DCTYPE_MEMORY ? "memory managed DCs" : "device DCs"));
//
// cleanup:
//     DC_UnlockDc(pdc);
//     return realize;
// }
//
// UINT APIENTRY
// IntAnimatePalette(HPALETTE hPal,
//                   UINT StartIndex,
//                   UINT NumEntries,
//                   CONST PPALETTEENTRY PaletteColors)
// {
//     UINT ret = 0;
//
//     if( hPal != NtGdiGetStockObject(DEFAULT_PALETTE) )
//     {
//         PPALETTE palPtr;
//         UINT pal_entries;
//         const PALETTEENTRY *pptr = PaletteColors;
//
//         palPtr = PALETTE_ShareLockPalette(hPal);
//         if (!palPtr) return FALSE;
//
//         pal_entries = palPtr->NumColors;
//         if (StartIndex >= pal_entries)
//         {
//             PALETTE_ShareUnlockPalette(palPtr);
//             return FALSE;
//         }
//         if (StartIndex+NumEntries > pal_entries) NumEntries = pal_entries - StartIndex;
//
//         for (NumEntries += StartIndex; StartIndex < NumEntries; StartIndex++, pptr++)
//         {
//             /* According to MSDN, only animate PC_RESERVED colours */
//             if (palPtr->IndexedColors[StartIndex].peFlags & PC_RESERVED)
//             {
//                 memcpy( &palPtr->IndexedColors[StartIndex], pptr,
//                         sizeof(PALETTEENTRY) );
//                 ret++;
//                 PALETTE_ValidateFlags(&palPtr->IndexedColors[StartIndex], 1);
//             }
//         }
//
//         PALETTE_ShareUnlockPalette(palPtr);
//
// #if 0
// /* FIXME: This is completely broken! We cannot call UserGetDesktopWindow
//    without first acquiring the USER lock. But the whole process here is
//    screwed anyway. Instead of messing with the desktop DC, we need to
//    check, whether the palette is associated with a PDEV and whether that
//    PDEV supports palette operations. Then we need to call pfnDrvSetPalette.
//    But since IntGdiRealizePalette() is not even implemented for direct DCs,
//    we can as well just do nothing, that will at least not ASSERT!
//    I leave the whole thing here, to scare people away, who want to "fix" it. */
//
//         /* Immediately apply the new palette if current window uses it */
//         Wnd = UserGetDesktopWindow();
//         hDC =  UserGetWindowDC(Wnd);
//         dc = DC_LockDc(hDC);
//         if (NULL != dc)
//         {
//             if (dc->dclevel.hpal == hPal)
//             {
//                 DC_UnlockDc(dc);
//                 IntGdiRealizePalette(hDC);
//             }
//             else
//                 DC_UnlockDc(dc);
//         }
//         UserReleaseDC(Wnd,hDC, FALSE);
// #endif // 0
//     }
//     return ret;
// }
//
// UINT APIENTRY
// IntGetPaletteEntries(
//     HPALETTE hpal,
//     UINT StartIndex,
//     UINT  Entries,
//     LPPALETTEENTRY  pe)
// {
//     PPALETTE palGDI;
//     UINT numEntries;
//
//     palGDI = (PPALETTE) PALETTE_ShareLockPalette(hpal);
//     if (NULL == palGDI)
//     {
//         return 0;
//     }
//
//     numEntries = palGDI->NumColors;
//     if (NULL != pe)
//     {
//         if (numEntries < StartIndex + Entries)
//         {
//             Entries = numEntries - StartIndex;
//         }
//         if (numEntries <= StartIndex)
//         {
//             PALETTE_ShareUnlockPalette(palGDI);
//             return 0;
//         }
//         memcpy(pe, palGDI->IndexedColors + StartIndex, Entries * sizeof(PALETTEENTRY));
//     }
//     else
//     {
//         Entries = numEntries;
//     }
//
//     PALETTE_ShareUnlockPalette(palGDI);
//     return Entries;
// }
//
// UINT APIENTRY
// IntGetSystemPaletteEntries(HDC  hDC,
//                            UINT  StartIndex,
//                            UINT  Entries,
//                            LPPALETTEENTRY  pe)
// {
//     PPALETTE palGDI = NULL;
//     PDC dc = NULL;
//     UINT EntriesSize = 0;
//     UINT Ret = 0;
//
//     if (Entries == 0)
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     if (pe != NULL)
//     {
//         EntriesSize = Entries * sizeof(pe[0]);
//         if (Entries != EntriesSize / sizeof(pe[0]))
//         {
//             /* Integer overflow! */
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             return 0;
//         }
//     }
//
//     if (!(dc = DC_LockDc(hDC)))
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     palGDI = PALETTE_ShareLockPalette(dc->dclevel.hpal);
//     if (palGDI != NULL)
//     {
//         if (pe != NULL)
//         {
//             if (StartIndex >= palGDI->NumColors)
//                 Entries = 0;
//             else if (Entries > palGDI->NumColors - StartIndex)
//                 Entries = palGDI->NumColors - StartIndex;
//
//             memcpy(pe,
//                    palGDI->IndexedColors + StartIndex,
//                    Entries * sizeof(pe[0]));
//
//             Ret = Entries;
//         }
//         else
//         {
//             Ret = dc->ppdev->gdiinfo.ulNumPalReg;
//         }
//     }
//
//     if (palGDI != NULL)
//         PALETTE_ShareUnlockPalette(palGDI);
//
//     if (dc != NULL)
//         DC_UnlockDc(dc);
//
//     return Ret;
// }
//
// UINT
// APIENTRY
// IntSetPaletteEntries(
//     HPALETTE  hpal,
//     UINT  Start,
//     UINT  Entries,
//     CONST LPPALETTEENTRY pe)
// {
//     PPALETTE palGDI;
//     ULONG numEntries;
//
//     if (GDI_HANDLE_IS_STOCKOBJ(hpal))
//     {
//     	return 0;
//     }
//
//     palGDI = PALETTE_ShareLockPalette(hpal);
//     if (!palGDI) return 0;
//
//     numEntries = palGDI->NumColors;
//     if (Start >= numEntries)
//     {
//         PALETTE_ShareUnlockPalette(palGDI);
//         return 0;
//     }
//     if (numEntries < Start + Entries)
//     {
//         Entries = numEntries - Start;
//     }
//     memcpy(palGDI->IndexedColors + Start, pe, Entries * sizeof(PALETTEENTRY));
//     PALETTE_ShareUnlockPalette(palGDI);
//
//     return Entries;
// }
//
// ULONG
// APIENTRY
// GreGetSetColorTable(
//     HDC hdc,
//     ULONG iStartIndex,
//     ULONG cEntries,
//     RGBQUAD *prgbColors,
//     BOOL bSet)
// {
//     PDC pdc;
//     PSURFACE psurf;
//     PPALETTE ppal = NULL;
//     ULONG i, iEndIndex, iResult = 0;
//
//     /* Lock the DC */
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         return 0;
//     }
//
//     /* Get the surface from the DC */
//     psurf = pdc->dclevel.pSurface;
//
//     /* Check if we have the default surface */
//     if (psurf == NULL)
//     {
//         /* Use a mono palette */
//         if (!bSet)
//             ppal = gppalMono;
//     }
//     else if (psurf->SurfObj.iType == STYPE_BITMAP)
//     {
//         /* Get the palette of the surface */
//         ppal = psurf->ppal;
//     }
//
//     /* Check if this is an indexed palette and the range is ok */
//     if (ppal && (ppal->flFlags & PAL_INDEXED) &&
//         (iStartIndex < ppal->NumColors))
//     {
//         /* Calculate the end of the operation */
//         iEndIndex = min(iStartIndex + cEntries, ppal->NumColors);
//
//         /* Check what operation to perform */
//         if (bSet)
//         {
//             /* Loop all colors and set the palette entries */
//             for (i = iStartIndex; i < iEndIndex; i++, prgbColors++)
//             {
//                 ppal->IndexedColors[i].peRed = prgbColors->rgbRed;
//                 ppal->IndexedColors[i].peGreen = prgbColors->rgbGreen;
//                 ppal->IndexedColors[i].peBlue = prgbColors->rgbBlue;
//             }
//
//             /* Mark the dc brushes invalid */
//             pdc->pdcattr->ulDirty_ |= DIRTY_FILL|DIRTY_LINE|
//                                       DIRTY_BACKGROUND|DIRTY_TEXT;
//         }
//         else
//         {
//             /* Loop all colors and get the palette entries */
//             for (i = iStartIndex; i < iEndIndex; i++, prgbColors++)
//             {
//                 prgbColors->rgbRed = ppal->IndexedColors[i].peRed;
//                 prgbColors->rgbGreen = ppal->IndexedColors[i].peGreen;
//                 prgbColors->rgbBlue = ppal->IndexedColors[i].peBlue;
//                 prgbColors->rgbReserved = 0;
//             }
//         }
//
//         /* Calculate how many entries were modified */
//         iResult = iEndIndex - iStartIndex;
//     }
//
//     /* Unlock the DC */
//     DC_UnlockDc(pdc);
//
//     return iResult;
// }
//
// __kernel_entry
// LONG
// APIENTRY
// NtGdiDoPalette(
//     _In_ HGDIOBJ hObj,
//     _In_ WORD iStart,
//     _In_ WORD cEntries,
//     _When_(bInbound!=0, _In_reads_bytes_(cEntries*sizeof(PALETTEENTRY)))
//     _When_(bInbound==0, _Out_writes_bytes_(cEntries*sizeof(PALETTEENTRY))) LPVOID pUnsafeEntries,
//     _In_ DWORD iFunc,
//     _In_ BOOL bInbound)
// {
// 	LONG ret;
// 	LPVOID pEntries = NULL;
// 	SIZE_T cjSize;
// 	NTSTATUS Status;
//
// 	if (pUnsafeEntries)
// 	{
// 		if (cEntries == 0)
// 			return 0;
//
// 		cjSize = cEntries * sizeof(PALETTEENTRY);
// 		pEntries = ExAllocatePoolWithTag(PagedPool, cjSize, TAG_PALETTE);
// 		if (!pEntries)
// 			return 0;
//
// 		if (bInbound)
// 		{
// 			{
// 			    NTSTATUS __SehStatus = SafeProbeForRead(pUnsafeEntries, cjSize as SIZE_T, 1);
// 			    if (!NT_SUCCESS(__SehStatus))
// 			    {
// 			        ExFreePoolWithTag(pEntries, TAG_PALETTE);
// 			        return 0;
// 			    }
// 			}
// 			memcpy(pEntries, pUnsafeEntries, cjSize);
// 		}
// 		else
// 		{
// 		    /* Zero it out, so we don't accidentally leak kernel data */
// 		    RtlZeroMemory(pEntries, cjSize as usize);
// 		}
// 	}
//
// 	ret = 0;
// 	switch(iFunc)
// 	{
// 		case GdiPalAnimate:
// 			if (pEntries)
// 				ret = IntAnimatePalette((HPALETTE)hObj, iStart, cEntries, (CONST PPALETTEENTRY)pEntries);
// 			break;
//
// 		case GdiPalSetEntries:
// 			if (pEntries)
// 				ret = IntSetPaletteEntries((HPALETTE)hObj, iStart, cEntries, (CONST LPPALETTEENTRY)pEntries);
// 			break;
//
// 		case GdiPalGetEntries:
// 			ret = IntGetPaletteEntries((HPALETTE)hObj, iStart, cEntries, (LPPALETTEENTRY)pEntries);
// 			break;
//
// 		case GdiPalGetSystemEntries:
// 			ret = IntGetSystemPaletteEntries((HDC)hObj, iStart, cEntries, (LPPALETTEENTRY)pEntries);
// 			break;
//
// 		case GdiPalSetColorTable:
// 			if (pEntries)
// 				ret = GreGetSetColorTable((HDC)hObj, iStart, cEntries, (RGBQUAD*)pEntries, TRUE);
// 			break;
//
// 		case GdiPalGetColorTable:
// 			if (pEntries)
// 				ret = GreGetSetColorTable((HDC)hObj, iStart, cEntries, (RGBQUAD*)pEntries, FALSE);
// 			break;
// 	}
//
// 	if (pEntries)
// 	{
// 		if (!bInbound && (ret > 0))
// 		{
// 			cjSize = min(cEntries, ret) * sizeof(PALETTEENTRY);
// 			Status = SafeProbeForWrite(pUnsafeEntries, cjSize as SIZE_T, 1);
// 			if (!NT_SUCCESS(Status))
// 			{
// 				ret = 0;
// 			}
// 			else
// 			{
// 				memcpy(pUnsafeEntries, pEntries, cjSize);
// 			}
// 		}
// 		ExFreePoolWithTag(pEntries, TAG_PALETTE);
// 	}
//
// 	return ret;
// }
//
//
// UINT APIENTRY
// NtGdiSetSystemPaletteUse(HDC hDC, UINT Usage)
// {
//     UINT old = SystemPaletteUse;
//
//     /* Device doesn't support colour palettes */
//     if (!(NtGdiGetDeviceCaps(hDC, RASTERCAPS) & RC_PALETTE)) {
//         return SYSPAL_ERROR;
//     }
//
//     switch (Usage)
// 	{
// 		case SYSPAL_NOSTATIC:
//         case SYSPAL_NOSTATIC256:
//         case SYSPAL_STATIC:
// 				SystemPaletteUse = Usage;
// 				break;
//
//         default:
// 				old=SYSPAL_ERROR;
// 				break;
// 	}
//
//  return old;
// }
//
// UINT
// APIENTRY
// NtGdiGetSystemPaletteUse(HDC hDC)
// {
//     return SystemPaletteUse;
// }
//
// BOOL
// APIENTRY
// NtGdiUpdateColors(HDC hDC)
// {
//    PWND Wnd;
//    BOOL calledFromUser, ret;
//    USER_REFERENCE_ENTRY Ref;
//
//    calledFromUser = UserIsEntered();
//
//    if (!calledFromUser){
//       UserEnterExclusive();
//    }
//
//    Wnd = UserGetWindowObject(IntWindowFromDC(hDC));
//    if (Wnd == NULL)
//    {
//       EngSetLastError(ERROR_INVALID_WINDOW_HANDLE);
//
//       if (!calledFromUser){
//          UserLeave();
//       }
//
//       return FALSE;
//    }
//
//    UserRefObjectCo(Wnd, &Ref);
//    ret = co_UserRedrawWindow(Wnd, NULL, 0, RDW_INVALIDATE);
//    UserDerefObjectCo(Wnd);
//
//    if (!calledFromUser){
//       UserLeave();
//    }
//
//    return ret;
// }
//
// BOOL
// APIENTRY
// NtGdiUnrealizeObject(HGDIOBJ hgdiobj)
// {
//    BOOL Ret = FALSE;
//    PPALETTE palGDI;
//
//    if ( !hgdiobj ||
//         GDI_HANDLE_IS_STOCKOBJ(hgdiobj) ||
//         !GDI_HANDLE_IS_TYPE(hgdiobj, GDI_OBJECT_TYPE_PALETTE) )
//       return Ret;
//
//    palGDI = PALETTE_ShareLockPalette(hgdiobj as HPALETTE);
//    if (!palGDI) return FALSE;
//
//    // FIXME!!
//    // Need to do something!!!
//    // Zero out Current and Old Translated pointers?
//    //
//    Ret = TRUE;
//    PALETTE_ShareUnlockPalette(palGDI);
//    return Ret;
// }
//
// __kernel_entry
// HPALETTE
// APIENTRY
// NtGdiEngCreatePalette(
//     _In_ ULONG iMode,
//     _In_ ULONG cColors,
//     _In_ ULONG *pulColors,
//     _In_ FLONG flRed,
//     _In_ FLONG flGreen,
//     _In_ FLONG flBlue)
// {
//     HPALETTE hPal = NULL;
//     ULONG *pulcSafe, ulColors[WINDDI_MAXSETPALETTECOLORS];
//
//     if ( cColors > MAX_PALCOLORS ) return NULL;
//
//     if ( cColors <= WINDDI_MAXSETPALETTECOLORS )
//     {
//         pulcSafe = ulColors;
//     }
//     else
//     {
//         pulcSafe = ExAllocatePoolWithTag(PagedPool, cColors * sizeof(ULONG), GDITAG_UMPD );
//     }
//
//         {
//             NTSTATUS __SehStatus = SafeProbeForRead(pulColors, cColors * sizeof(ULONG), 1);
//             if (!NT_SUCCESS(__SehStatus))
//             {
//                 SetLastNtError(__SehStatus);
//                 if ( cColors > WINDDI_MAXSETPALETTECOLORS ) ExFreePoolWithTag( pulcSafe, GDITAG_UMPD );
//                 return hPal;
//             }
//         }
//         RtlCopyMemory( pulcSafe, pulColors, cColors * sizeof(ULONG) );
//
//     hPal = EngCreatePalette( iMode/*|PAL_SETPOWNER*/, cColors, pulcSafe, flRed, flGreen, flBlue );
//
//     if ( cColors > WINDDI_MAXSETPALETTECOLORS ) ExFreePoolWithTag( pulcSafe, GDITAG_UMPD );
//
//     return hPal;
// }
//
// __kernel_entry
// BOOL
// APIENTRY
// NtGdiEngDeletePalette(
//     _In_ HPALETTE hPal)
// {
//     return EngDeletePalette(hPal);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
