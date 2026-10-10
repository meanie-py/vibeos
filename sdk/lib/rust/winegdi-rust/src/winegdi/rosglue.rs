use crate::winegdi::emfdc::EMFDC_DeleteDC;
use crate::winegdi::emfdc::EMFDC_SetArcDirection;
use crate::winegdi::emfdc::EMFDC_SetMapMode;
use crate::winegdi::metadc::METADC_DeleteDC;
use crate::winegdi::metadc::METADC_DeleteObject;
use crate::winegdi::metadc::METADC_SetMapMode;
use crate::winegdi::metadc::METADC_SetRelAbs;
use crate::winegdi::emfdc::emfdc_delete_object;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// #undef SetWorldTransform: no declaration emitted.
// #define NDEBUG (empty value): no constant emitted.
// extern METADC_DeleteObject / emfdc_delete_object: prototypes only, no declarations emitted.

unsafe extern "C" fn ConvertObjectType(mut wType: WORD) -> GDILOOBJTYPE {
    /* Get the GDI object type */
    match (wType as ::core::ffi::c_int) as u32 {
        OBJ_PEN => return GDILoObjType_LO_PEN_TYPE,
        OBJ_BRUSH => return GDILoObjType_LO_BRUSH_TYPE,
        OBJ_DC => return GDILoObjType_LO_DC_TYPE,
        OBJ_METADC => return GDILoObjType_LO_METADC16_TYPE,
        OBJ_PAL => return GDILoObjType_LO_PALETTE_TYPE,
        OBJ_FONT => return GDILoObjType_LO_FONT_TYPE,
        OBJ_BITMAP => return GDILoObjType_LO_BITMAP_TYPE,
        OBJ_REGION => return GDILoObjType_LO_REGION_TYPE,
        OBJ_METAFILE => return GDILoObjType_LO_METAFILE16_TYPE,
        OBJ_MEMDC => return GDILoObjType_LO_DC_TYPE,
        OBJ_EXTPEN => return GDILoObjType_LO_EXTPEN_TYPE,
        OBJ_ENHMETADC => return GDILoObjType_LO_ALTDC_TYPE,
        OBJ_ENHMETAFILE => return GDILoObjType_LO_METAFILE_TYPE,
        OBJ_COLORSPACE => return GDILoObjType_LO_ICMLCS_TYPE,
        _ => return 0 as GDILOOBJTYPE,
    }
}
#[no_mangle]
pub unsafe extern "C" fn alloc_gdi_handle(mut pvObject: *mut ::core::ffi::c_void, mut wType: WORD, mut funcs: *const gdi_obj_funcs) -> HGDIOBJ {
    let mut eObjType: GDILOOBJTYPE = 0 as GDILOOBJTYPE;
    eObjType = ConvertObjectType(wType);
    if eObjType as u32 != GDILoObjType_LO_METAFILE_TYPE as u32
        && eObjType as u32 != GDILoObjType_LO_METAFILE16_TYPE as u32
        && eObjType as u32 != GDILoObjType_LO_METADC16_TYPE as u32
    {
        ASSERT!(FALSE != 0);
        return core::ptr::null_mut();
    }
    return GdiCreateClientObj(pvObject, eObjType);
}

#[no_mangle]
pub unsafe extern "C" fn free_gdi_handle(hobj: HGDIOBJ) -> *mut ::core::ffi::c_void {
    GdiDeleteClientObj(hobj)
}
#[no_mangle]
pub unsafe extern "C" fn GDI_GetObjPtr(hobj: HGDIOBJ, wType: WORD) -> *mut ::core::ffi::c_void {
    let mut eObjType: GDILOOBJTYPE;

    /* Check if the object type matches */
    eObjType = ConvertObjectType(wType);
    if (eObjType == 0) || (GDI_HANDLE_GET_TYPE!(hobj) != eObjType)
    {
        return core::ptr::null_mut();
    }

    /* Check if we have an ALTDC */
    if eObjType == GDILoObjType_LO_ALTDC_TYPE
    {
        /* Object is stored as LDC */
        return GdiGetLDC(hobj as HDC) as *mut core::ffi::c_void;
    }

    /* Check for client objects */
    if (eObjType == GDILoObjType_LO_METAFILE_TYPE) ||
        (eObjType == GDILoObjType_LO_METAFILE16_TYPE) ||
        (eObjType == GDILoObjType_LO_METADC16_TYPE)
    {
        return GdiGetClientObjLink(hobj) as *mut core::ffi::c_void;
    }

    /* This should never happen! */
    ASSERT!(FALSE != 0);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn GDI_ReleaseObj(hobj: HGDIOBJ) {}
#[no_mangle]
pub unsafe extern "C" fn alloc_dc_ptr(magic: WORD) -> *mut WINEDC {
    let mut pWineDc: *mut WINEDC = core::ptr::null_mut();

    /* Allocate the Wine DC */
    pWineDc = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<WINEDC>() as SIZE_T,
    ) as *mut WINEDC;
    if pWineDc.is_null() {
        return core::ptr::null_mut();
    }

    ZeroMemory!(
        pWineDc as *mut core::ffi::c_void,
        core::mem::size_of::<WINEDC>(),
    );
    (*pWineDc).hBrush = GetStockObject((WHITE_BRUSH as i32)) as HBRUSH;
    (*pWineDc).hPen = GetStockObject((BLACK_PEN as i32)) as HPEN;

    if magic == (OBJ_ENHMETADC as u16) {
        /* We create a metafile DC, but we ignore the reference DC, this is
           handled by the wine code */
        (*pWineDc).hdc = NtGdiCreateMetafileDC(core::ptr::null_mut());
        if (*pWineDc).hdc.is_null() {
            HeapFree(
                GetProcessHeap(),
                0,
                pWineDc as PVOID,
            );
            return core::ptr::null_mut();
        }

        (*pWineDc).iType = (LDC_EMFLDC as i32);

        /* Set the Wine DC as LDC */
        GdiSetLDC((*pWineDc).hdc, pWineDc as PVOID);
    } else {
        // nothing else supported!
        ASSERT!(FALSE != 0);
    }

    return pWineDc as *mut WINEDC;
}
#[no_mangle]
pub unsafe extern "C" fn get_dc_ptr(hdc: HDC) -> *mut WINEDC {
    if GDI_HANDLE_GET_TYPE!(hdc) == GDILoObjType_LO_ALTDC_TYPE {
        /* The Wine DC is stored as the LDC */
        return GdiGetLDC(hdc) as *mut WINEDC;
    }

    /* Check for METADC16 */
    if GDI_HANDLE_GET_TYPE!(hdc) == GDILoObjType_LO_METADC16_TYPE {
        return GdiGetClientObjLink((hdc).cast()) as *mut WINEDC;
    }

    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn GDI_hdc_using_object(mut hobj: HGDIOBJ, mut hdc: HDC) {
    if GdiCreateClientObjLink(hobj, hdc as PVOID) == 0 {
        if DbgPrint(
            b"(%s:%d) Failed to create link for selected METADC object.\n\0" as *const u8
                as PCSTR,
            b"win32ss/gdi/gdi32/wine/rosglue.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            176 as ::core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                b"win32ss/gdi/gdi32/wine/rosglue.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                176 as ::core::ffi::c_int,
            );
        }
        return;
    }
}
#[no_mangle]
pub unsafe extern "C" fn GDI_hdc_not_using_object(mut hobj: HGDIOBJ, mut hdc: HDC) {
    let mut hdcLink: HDC = core::ptr::null_mut();
    hdcLink = GdiRemoveClientObjLink(hobj) as HDC;
    ASSERT!(hdcLink.is_null() || hdcLink == hdc);
}
#[no_mangle]
pub unsafe extern "C" fn bitmap_info_size(info: *const BITMAPINFO, coloruse: WORD) -> ::core::ffi::c_int {
    let mut colors: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;
    let mut masks: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

    if (*info).bmiHeader.biSize as usize == ::core::mem::size_of::<BITMAPCOREHEADER>() {
        let core: *const BITMAPCOREHEADER = info as *const BITMAPCOREHEADER;
        colors = (if (*core).bcBitCount as ::core::ffi::c_int <= 8 as ::core::ffi::c_int {
            (1 as ::core::ffi::c_int) << (*core).bcBitCount as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as ::core::ffi::c_uint;
        return (::core::mem::size_of::<BITMAPCOREHEADER>() as usize).wrapping_add(
            (colors as usize).wrapping_mul(
                if coloruse as ::core::ffi::c_int == DIB_RGB_COLORS as ::core::ffi::c_int {
                    ::core::mem::size_of::<RGBTRIPLE>() as usize
                } else {
                    ::core::mem::size_of::<WORD>() as usize
                },
            ),
        ) as ::core::ffi::c_int;
    } else {
        if (*info).bmiHeader.biClrUsed != 0 {
            colors = min(
                (*info).bmiHeader.biClrUsed as ::core::ffi::c_uint,
                256 as ::core::ffi::c_uint,
            ) as ::core::ffi::c_uint;
        } else {
            colors = (if (*info).bmiHeader.biBitCount as ::core::ffi::c_int > 8 as ::core::ffi::c_int {
                0 as ::core::ffi::c_int
            } else {
                (1 as ::core::ffi::c_int) << (*info).bmiHeader.biBitCount as ::core::ffi::c_int
            }) as ::core::ffi::c_uint;
        }
        if (*info).bmiHeader.biCompression == BI_BITFIELDS as DWORD {
            masks = 3 as ::core::ffi::c_uint;
        }
        size = max(
            (*info).bmiHeader.biSize as usize,
            (::core::mem::size_of::<BITMAPINFOHEADER>() as usize).wrapping_add(
                (masks as usize).wrapping_mul(::core::mem::size_of::<DWORD>() as usize),
            ),
        ) as ::core::ffi::c_uint;
        return (size as usize).wrapping_add(
            (colors as usize).wrapping_mul(
                if coloruse as ::core::ffi::c_int == DIB_RGB_COLORS as ::core::ffi::c_int {
                    ::core::mem::size_of::<RGBQUAD>() as usize
                } else {
                    ::core::mem::size_of::<WORD>() as usize
                },
            ),
        ) as ::core::ffi::c_int;
    }
}
#[no_mangle]
pub unsafe extern "C" fn get_brush_bitmap_info(
    mut hbr: HBRUSH,
    mut pbmi: PBITMAPINFO,
    mut pvBits: PVOID,
    mut puUsage: PUINT,
) -> BOOL {
    let mut hbmp: HBITMAP = core::ptr::null_mut();
    let mut hdc: HDC = core::ptr::null_mut();
    let mut Bits: PVOID = core::ptr::null_mut();

    /* Call win32k to get the bitmap handle and color usage */
    hbmp = NtGdiGetObjectBitmapHandle(hbr, puUsage as *mut UINT);
    if hbmp.is_null() {
        return (FALSE) as BOOL;
    }

    hdc = GetDC(core::ptr::null_mut());
    if hdc.is_null() {
        return (FALSE) as BOOL;
    }

    /* Initialize the BITMAPINFO */
    ZeroMemory!(
        pbmi as *mut core::ffi::c_void,
        core::mem::size_of::<BITMAPINFO>() as usize,
    );
    (*pbmi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;

    /* Retrieve information about the bitmap */
    if GetDIBits(
        hdc,
        hbmp,
        0,
        0,
        core::ptr::null_mut(),
        pbmi,
        *puUsage,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    if !pvBits.is_null() {
        /* Now allocate a buffer for the bits */
        Bits = HeapAlloc(
            GetProcessHeap(),
            0,
            (*pbmi).bmiHeader.biSizeImage as SIZE_T,
        );
        if Bits.is_null() {
            return (FALSE) as BOOL;
        }

        /* Retrieve the bitmap bits */
        if GetDIBits(
            hdc,
            hbmp,
            0,
            (*pbmi).bmiHeader.biHeight as UINT,
            Bits as LPVOID,
            pbmi,
            *puUsage,
        ) == 0
        {
            HeapFree(GetProcessHeap(), 0, Bits);
            return (FALSE) as BOOL;
        }

        CopyMemory!(
            pvBits,
            Bits,
            (*pbmi).bmiHeader.biSizeImage as usize,
        );
    }

    /* GetDIBits doesn't set biClrUsed, but wine code needs it, so we set it */
    if (*pbmi).bmiHeader.biBitCount as core::ffi::c_int <= 8 {
        (*pbmi).bmiHeader.biClrUsed =
            (1 << (*pbmi).bmiHeader.biBitCount) as DWORD;
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetVirtualResolution(
    hdc: HDC,
    cxVirtualDevicePixel: DWORD,
    cyVirtualDevicePixel: DWORD,
    cxVirtualDeviceMm: DWORD,
    cyVirtualDeviceMm: DWORD,
) -> BOOL {
    NtGdiSetVirtualResolution(hdc, (cxVirtualDevicePixel) as INT, (cyVirtualDevicePixel) as INT, (cxVirtualDeviceMm) as INT, (cyVirtualDeviceMm) as INT)
}

#[no_mangle]
pub unsafe extern "system" fn DeleteColorSpace(hcs: HCOLORSPACE) -> BOOL {
    NtGdiDeleteColorSpace((hcs).cast())
}

#[no_mangle]
pub unsafe extern "C" fn _assert(exp: *const i8, file: *const i8, line: u32) {
    DbgRaiseAssertionFailure!();
}
#[no_mangle]
pub unsafe extern "system" fn METADC_SetD(hdc: HDC, dwIn: DWORD, usMF16Id: USHORT) -> BOOL {
    match (usMF16Id as i32) as u32 {
        META_SETMAPMODE => return METADC_SetMapMode(hdc, dwIn as i32),
        META_SETRELABS => return METADC_SetRelAbs(hdc, dwIn as i32),
        _ => return (FALSE) as i32,
    }
}

#[no_mangle]
pub unsafe extern "system" fn EMFDC_SetD(mut pldc: PLDC, mut dwIn: DWORD, mut ulMFId: ULONG) -> BOOL {
    if ulMFId == EMR_SETMAPMODE {
        return EMFDC_SetMapMode((pldc as *mut LDC).cast(), dwIn as INT);
    } else if ulMFId == EMR_SETARCDIRECTION {
        return EMFDC_SetArcDirection((pldc as *mut LDC).cast(), dwIn as INT);
    } else {
        return (FALSE) as BOOL;
    }
}

#[no_mangle]
pub unsafe extern "system" fn METADC_RosGlueDeleteObject(hobj: HGDIOBJ) {
    let mut eObjectType: GDILOOBJTYPE = GDI_HANDLE_GET_TYPE!(hobj);
    let mut hdc: HDC = core::ptr::null_mut();

    /* Check for one of the types we actually handle here */
    if eObjectType as u32 != GDILoObjType_LO_BRUSH_TYPE as u32
        && eObjectType as u32 != GDILoObjType_LO_PEN_TYPE as u32
        && eObjectType as u32 != GDILoObjType_LO_EXTPEN_TYPE as u32
        && eObjectType as u32 != GDILoObjType_LO_PALETTE_TYPE as u32
        && eObjectType as u32 != GDILoObjType_LO_FONT_TYPE as u32
    {
        return;
    }

    /* Check if we have a client object link and remove it if it was found.
       The link is the HDC that the object was selected into. */
    hdc = GdiRemoveClientObjLink(hobj) as HDC;
    if hdc.is_null() {
        if DbgPrint(
            b"(%s:%d) the link was not found\n\0".as_ptr() as PCSTR,
            b"win32ss/gdi/gdi32/wine/rosglue.c\0"
                .as_ptr() as *const core::ffi::c_char,
            373 as core::ffi::c_int,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"win32ss/gdi/gdi32/wine/rosglue.c\0"
                    .as_ptr() as *const core::ffi::c_char,
                373 as core::ffi::c_int,
            );
        }
        /* The link was not found, so we are not handling this object here */
        return;
    }

    if GDI_HANDLE_GET_TYPE!(hdc as HGDIOBJ) as u32 == GDILoObjType_LO_METADC16_TYPE as u32 {
        METADC_DeleteObject(hdc, hobj);
    }

    if GDI_HANDLE_GET_TYPE!(hdc as HGDIOBJ) as u32 == GDILoObjType_LO_ALTDC_TYPE as u32 {
        let mut pWineDc: *mut LDC = GdiGetLDC(hdc) as *mut LDC;
        if !pWineDc.is_null() {
            emfdc_delete_object(hdc, hobj);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn METADC_RosGlueDeleteDC(hdc: HDC) -> BOOL {
    let mut pWineDc: *mut LDC = core::ptr::null_mut();

    if GDI_HANDLE_GET_TYPE!(hdc) == GDILoObjType_LO_METADC16_TYPE {
        return METADC_DeleteDC(hdc);
    }

    if GDI_HANDLE_GET_TYPE!(hdc) == GDILoObjType_LO_ALTDC_TYPE {
        pWineDc = GdiGetLDC(hdc) as *mut LDC;

        if !pWineDc.is_null() {
            // Handle Printer LDC
            if (*pWineDc).iType != (LDC_EMFLDC as i32) {
                //return IntDeleteDC(hdc);
            }

            EMFDC_DeleteDC((pWineDc).cast());

            /* Get rid of the LDC */
            ASSERT!(GdiGetLDC((*pWineDc).hDC) == pWineDc as PLDC);

            GdiSetLDC((*pWineDc).hDC, core::ptr::null_mut());

            /* Free the Wine DC */
            HeapFree(GetProcessHeap(), 0, pWineDc as PVOID);
        }
    }

    return NtGdiDeleteObjectApp(hdc as HANDLE);
}
