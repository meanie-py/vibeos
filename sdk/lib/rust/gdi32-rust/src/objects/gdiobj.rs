//! win32ss/gdi/gdi32/objects/gdiobj.c → Rust. GDI-object client layer:
//! GetStockObject / GetObjectType / GetFontObjectA / GetObjectA / GetObjectW /
//! GdiDeleteBrushOrPen (static) / DeleteObject.
//
// NEEDS-IMPORT: `crate::types::*` and `win32k_sys::win32k::*` BOTH export the scalar
//   Win32 aliases referenced here (BOOL/DWORD/INT/UINT/LONG/WORD/HANDLE/LPVOID/HGDIOBJ/
//   HDC/HRGN). If central integration hits E0659 ambiguous-glob on these names, drop
//   `use crate::types::*;` — win32k_sys provides every type this file needs, mirroring
//   user32-rust (which globs only win32k_sys). Kept here per the contract's import list.
// NEEDS-IMPORT: `NtCurrentTeb` is referenced via `crate::gdihelp::NtCurrentTeb()` (the
//   TEB substrate that backs GdiAllocBatchCommand). If gdihelp does not export it,
//   central integration should add it there (it is NOT in win32k-sys bindgen).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

// win32k_sys's FALSE/TRUE are bindgen u32 (#define), but BOOL is i32 — shadow with
// BOOL-typed locals so `== FALSE`/`return TRUE` type-check against the i32 BOOL returns.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── Cross-file gdi32 client functions this file calls (defined in other .c → other
//    cohab .rs / still-C objects). Decorations matched to their C prototypes:
//
//    misc/misc.c        BOOL    WINAPI  GdiValidateHandle(HGDIOBJ)        → extern "system"
//    misc/misc.c        HGDIOBJ WINAPI  GdiFixUpHandle(HGDIOBJ)           → extern "system"
//    misc/misc.c        BOOL            GdiGetHandleUserData(HGDIOBJ,DWORD,PVOID*) → extern "C" (plain cdecl)
//    objects/dc.c       DWORD   WINAPI  GetDCDWord(HDC,UINT,DWORD)        → extern "system"
//    objects/utils.c    VOID    WINAPI  EnumLogFontExW2A(LPENUMLOGFONTEXA, const ENUMLOGFONTEXW*) → extern "system"
//    wine/rosglue.c     VOID    WINAPI  METADC_RosGlueDeleteObject(HGDIOBJ) → extern "system"
//    objects/dc.c       BOOL    WINAPI  DeleteDC(HDC)                     → extern "system"
//    objects/region.c   BOOL    FASTCALL DeleteRegion(HRGN)              → extern "fastcall"
//    kernel32           VOID    WINAPI  SetLastError(DWORD)               → extern "system"
//    (GetObjectW is defined IN this file — no extern needed.)
extern "system" {
    fn GdiValidateHandle(hobj: HGDIOBJ) -> BOOL;
    fn GdiFixUpHandle(hGO: HGDIOBJ) -> HGDIOBJ;
    fn GetDCDWord(hdc: HDC, u: UINT, dwError: DWORD) -> DWORD;
    fn EnumLogFontExW2A(fontA: LPENUMLOGFONTEXA, fontW: *const ENUMLOGFONTEXW);
    fn METADC_RosGlueDeleteObject(hobj: HGDIOBJ);
    fn DeleteDC(hdc: HDC) -> BOOL;
    fn SetLastError(dwErrCode: DWORD);
}

extern "fastcall" {
    fn DeleteRegion(hrgn: HRGN) -> BOOL;
}

extern "C" {
    fn GdiGetHandleUserData(hGdiObj: HGDIOBJ, ObjectType: DWORD, UserData: *mut PVOID) -> BOOL;
    // gdi32-private global ULONG counter (objects/clientobj.c). Defined elsewhere.
    static mut gcClientObj: ULONG;
}

/// `HGDIOBJ stock_objects[NB_STOCK_OBJECTS];` — the per-process stock-object cache.
#[no_mangle]
pub static mut stock_objects: [HGDIOBJ; NB_STOCK_OBJECTS as usize] =
    [core::ptr::null_mut(); NB_STOCK_OBJECTS as usize];

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetStockObject(fnObject: INT) -> HGDIOBJ {
    let mut hobj: HGDIOBJ;

    if (fnObject < 0) || (fnObject >= NB_STOCK_OBJECTS as INT) {
        return core::ptr::null_mut();
    }

    hobj = stock_objects[fnObject as usize];
    if hobj.is_null() {
        hobj = NtGdiGetStockObject(fnObject);

        if GdiValidateHandle(hobj) == FALSE {
            return core::ptr::null_mut();
        }

        stock_objects[fnObject as usize] = hobj;
    }

    hobj
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetObjectType(h: HGDIOBJ) -> DWORD {
    let mut Ret: DWORD = 0;

    if GdiValidateHandle(h) != FALSE {
        let Type = GDI_HANDLE_GET_TYPE(h) as u32;
        match Type {
            GDI_OBJECT_TYPE_PEN => {
                Ret = OBJ_PEN;
            }
            GDI_OBJECT_TYPE_BRUSH => {
                Ret = OBJ_BRUSH;
            }
            GDI_OBJECT_TYPE_BITMAP => {
                Ret = OBJ_BITMAP;
            }
            GDI_OBJECT_TYPE_FONT => {
                Ret = OBJ_FONT;
            }
            GDI_OBJECT_TYPE_PALETTE => {
                Ret = OBJ_PAL;
            }
            GDI_OBJECT_TYPE_REGION => {
                Ret = OBJ_REGION;
            }
            GDI_OBJECT_TYPE_DC => {
                if GetDCDWord(h as HDC, GdiGetIsMemDc as UINT, 0) != 0 {
                    Ret = OBJ_MEMDC;
                } else {
                    Ret = OBJ_DC;
                }
            }
            GDI_OBJECT_TYPE_COLORSPACE => {
                Ret = OBJ_COLORSPACE;
            }
            GDI_OBJECT_TYPE_METAFILE => {
                Ret = OBJ_METAFILE;
            }
            GDI_OBJECT_TYPE_ENHMETAFILE => {
                Ret = OBJ_ENHMETAFILE;
            }
            GDI_OBJECT_TYPE_METADC => {
                Ret = OBJ_METADC;
            }
            GDI_OBJECT_TYPE_EXTPEN => {
                Ret = OBJ_EXTPEN;
            }
            GDILoObjType_LO_ALTDC_TYPE => {
                // FIXME: could be something else?
                Ret = OBJ_ENHMETADC;
            }
            _ => {
                // DPRINT1("GetObjectType: Magic 0x%08x not implemented\n", Type);
            }
        }
    } else {
        /* From Wine: GetObjectType does SetLastError() on a null object */
        SetLastError(ERROR_INVALID_HANDLE);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetFontObjectA(
    hfont: HGDIOBJ,
    mut cbSize: ULONG,
    lpBuffer: LPVOID,
) -> ULONG {
    let mut elfedvW: ENUMLOGFONTEXDVW = core::mem::zeroed();
    let mut elfedvA: ENUMLOGFONTEXDVA = core::mem::zeroed();
    let cbResult: ULONG;

    /* Check if size only is requested */
    if lpBuffer.is_null() {
        return core::mem::size_of::<LOGFONTA>() as ULONG;
    }

    /* Check for size 0 */
    if cbSize == 0 {
        /* Windows does not SetLastError() */
        return 0;
    }

    /* Windows does this ... */
    if cbSize == core::mem::size_of::<LOGFONTW>() as ULONG {
        cbSize = core::mem::size_of::<LOGFONTA>() as ULONG;
    }

    /* Call win32k to get the logfont (widechar) */
    cbResult = NtGdiExtGetObjectW(
        hfont,
        core::mem::size_of::<ENUMLOGFONTEXDVW>() as INT,
        &mut elfedvW as *mut _ as LPVOID,
    ) as ULONG;
    if cbResult == 0 {
        return 0;
    }

    /* Convert the logfont from widechar to ansi */
    EnumLogFontExW2A(
        &mut elfedvA.elfEnumLogfontEx as *mut _ as LPENUMLOGFONTEXA,
        &elfedvW.elfEnumLogfontEx as *const _ as *const ENUMLOGFONTEXW,
    );
    elfedvA.elfDesignVector = elfedvW.elfDesignVector;

    /* Don't copy more than maximum */
    if cbSize > core::mem::size_of::<ENUMLOGFONTEXDVA>() as ULONG {
        cbSize = core::mem::size_of::<ENUMLOGFONTEXDVA>() as ULONG;
    }

    /* Copy the number of bytes requested */
    core::ptr::copy_nonoverlapping(
        &elfedvA as *const _ as *const u8,
        lpBuffer as *mut u8,
        cbSize as usize,
    );

    /* Return the number of bytes copied */
    cbSize
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetObjectA(hGdiObj: HGDIOBJ, cbSize: INT, lpBuffer: LPVOID) -> INT {
    let dwType = GDI_HANDLE_GET_TYPE(hGdiObj) as u32;

    /* Chjeck if this is anything else but a font */
    if dwType == GDI_OBJECT_TYPE_FONT {
        GetFontObjectA(hGdiObj, cbSize as ULONG, lpBuffer) as INT
    } else {
        /* Simply pass it to the widechar version */
        GetObjectW(hGdiObj, cbSize, lpBuffer)
    }
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn GetObjectW(
    mut hGdiObj: HGDIOBJ,
    cbSize: INT,
    lpBuffer: LPVOID,
) -> INT {
    let dwType: u32;
    let cbResult: INT;

    /* Fixup handles with upper 16 bits masked */
    hGdiObj = GdiFixUpHandle(hGdiObj);

    /* Get the object type */
    dwType = GDI_HANDLE_GET_TYPE(hGdiObj) as u32;

    /* Check what kind of object we have */
    match dwType {
        GDI_OBJECT_TYPE_PEN => {
            if lpBuffer.is_null() {
                return core::mem::size_of::<LOGPEN>() as INT;
            }
        }

        GDI_OBJECT_TYPE_BRUSH => {
            if lpBuffer.is_null() {
                return core::mem::size_of::<LOGBRUSH>() as INT;
            }
        }

        GDI_OBJECT_TYPE_BITMAP => {
            if lpBuffer.is_null() {
                return core::mem::size_of::<BITMAP>() as INT;
            }
        }

        GDI_OBJECT_TYPE_PALETTE => {
            if lpBuffer.is_null() {
                return core::mem::size_of::<WORD>() as INT;
            }
        }

        GDI_OBJECT_TYPE_FONT => {
            if lpBuffer.is_null() {
                return core::mem::size_of::<LOGFONTW>() as INT;
            }
        }

        GDI_OBJECT_TYPE_EXTPEN => {
            /* we don't know the size, ask win32k */
        }

        GDI_OBJECT_TYPE_COLORSPACE => {
            if (cbSize < 328) || lpBuffer.is_null() {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return 0;
            }
        }

        GDI_OBJECT_TYPE_DC
        | GDI_OBJECT_TYPE_REGION
        | GDI_OBJECT_TYPE_EMF
        | GDI_OBJECT_TYPE_METAFILE
        | GDI_OBJECT_TYPE_ENHMETAFILE => {
            SetLastError(ERROR_INVALID_HANDLE);
            return 0;
        }
        _ => return 0,
    }

    /* Call win32k */
    cbResult = NtGdiExtGetObjectW(hGdiObj, cbSize, lpBuffer);

    /* Handle error */
    if cbResult == 0 {
        if GdiValidateHandle(hGdiObj) == FALSE {
            if (dwType == GDI_OBJECT_TYPE_PEN)
                || (dwType == GDI_OBJECT_TYPE_EXTPEN)
                || (dwType == GDI_OBJECT_TYPE_BRUSH)
                || (dwType == GDI_OBJECT_TYPE_COLORSPACE)
            {
                SetLastError(ERROR_INVALID_PARAMETER);
            }
        } else if (dwType == GDI_OBJECT_TYPE_PEN)
            || (dwType == GDI_OBJECT_TYPE_BRUSH)
            || (dwType == GDI_OBJECT_TYPE_COLORSPACE)
            || ((dwType == GDI_OBJECT_TYPE_EXTPEN)
                && (cbSize >= core::mem::size_of::<EXTLOGPEN>() as INT))
            || ((dwType == GDI_OBJECT_TYPE_BITMAP)
                && (cbSize >= core::mem::size_of::<BITMAP>() as INT))
        {
            if cbSize != 0 {
                SetLastError(ERROR_NOACCESS);
            }
        }
    }

    cbResult
}

unsafe fn GdiDeleteBrushOrPen(hobj: HGDIOBJ) -> BOOL {
    let eObjectType: GDILoObjType;
    let mut pbrattr: PBRUSH_ATTR = core::ptr::null_mut();
    let pTeb: PTEB;
    let pgO: PGDIBSOBJECT;

    eObjectType = GDI_HANDLE_GET_TYPE(hobj) as GDILoObjType;

    if (GdiGetHandleUserData(hobj, eObjectType as DWORD, &mut pbrattr as *mut _ as *mut PVOID)
        != FALSE)
        && (!pbrattr.is_null())
    {
        pTeb = NtCurrentTeb();
        if !(*pTeb).Win32ThreadInfo.is_null() {
            pgO = GdiAllocBatchCommand(core::ptr::null_mut(), crate::gdihelp::GdiBCDelObj as u16) as PGDIBSOBJECT;
            if !pgO.is_null() {
                // FIXME: we need to mark the object as deleted!
                (*pgO).hgdiobj = hobj;
                return TRUE;
            }
        }
    }

    NtGdiDeleteObjectApp(hobj)
}

/*
 * @implemented
 */
#[no_mangle]
pub unsafe extern "system" fn DeleteObject(hObject: HGDIOBJ) -> BOOL {
    /* Check if the handle is valid (FIXME: we need some special
       sauce for the stock object flag) */
    if GdiValidateHandle(hObject) == FALSE {
        return FALSE;
    }

    /* Check if this is a stock object */
    if GDI_HANDLE_IS_STOCKOBJ(hObject) {
        /* Ignore the attempt to delete a stock object */
        // DPRINT1("Trying to delete system object 0x%p\n", hObject);
        return TRUE;
    }

    /* If we have any METAFILE objects, we need to check them */
    if gcClientObj > 0 {
        // DPRINT("Going Glue\n");
        METADC_RosGlueDeleteObject(hObject);
    }

    /* Switch by object type */
    match GDI_HANDLE_GET_TYPE(hObject) as u32 {
        GDILoObjType_LO_METAFILE16_TYPE | GDILoObjType_LO_METAFILE_TYPE => FALSE,

        GDILoObjType_LO_DC_TYPE | GDILoObjType_LO_ALTDC_TYPE => DeleteDC(hObject as HDC),

        GDILoObjType_LO_ICMLCS_TYPE => NtGdiDeleteColorSpace(hObject),

        GDILoObjType_LO_REGION_TYPE => DeleteRegion(hObject as HRGN),

        GDILoObjType_LO_BRUSH_TYPE
        | GDILoObjType_LO_PEN_TYPE
        | GDILoObjType_LO_EXTPEN_TYPE => GdiDeleteBrushOrPen(hObject),

        // GDILoObjType_LO_FONT_TYPE | GDILoObjType_LO_BITMAP_TYPE | default
        _ => NtGdiDeleteObjectApp(hObject),
    }
}
