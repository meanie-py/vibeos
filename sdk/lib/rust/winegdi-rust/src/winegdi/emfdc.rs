use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut empty_bounds: RECTL = _RECTL {
    left: 0 as LONG,
    top: 0 as LONG,
    right: -(1 as ::core::ffi::c_int) as LONG,
    bottom: -(1 as ::core::ffi::c_int) as LONG,
};

pub const HANDLE_LIST_INC: i32 = 20;

pub const GdiGetEMFRestorDc: i32 = 5;

unsafe extern "C" fn emfdc_record(emf: *mut emf, emr: *mut EMR) -> BOOL {
    let mut len: DWORD = 0;
    let mut size: DWORD = 0;
    let mut emh: *mut ENHMETAHEADER = core::ptr::null_mut();

    // C <assert.h> assert -> rosglue's _assert -> DbgRaiseAssertionFailure; never a Rust panic.
    ASSERT!(((*emr).nSize & 3) == 0);

    (*(*emf).emh).nBytes = (*(*emf).emh).nBytes.wrapping_add((*emr).nSize);
    (*(*emf).emh).nRecords = (*(*emf).emh).nRecords.wrapping_add(1);

    size = HeapSize(
        GetProcessHeap(),
        0,
        (*emf).emh as LPCVOID,
    ) as DWORD;
    len = (*(*emf).emh).nBytes;
    if len > size {
        size = size.wrapping_add(
            size.wrapping_div(2).wrapping_add((*emr).nSize),
        );
        emh = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*emf).emh as PVOID,
            size as SIZE_T,
        ) as *mut ENHMETAHEADER;
        if emh.is_null() {
            return (FALSE) as BOOL;
        }
        (*emf).emh = emh;
    }
    memcpy(
        ((*emf).emh as *mut core::ffi::c_char)
            .offset((*(*emf).emh).nBytes as isize)
            .offset(-((*emr).nSize as isize)) as *mut core::ffi::c_void,
        emr as *const core::ffi::c_void,
        (*emr).nSize as usize,
    );
    return (TRUE) as BOOL;
}

unsafe extern "C" fn emfdc_update_bounds(mut emf: *mut emf, mut rect: *mut RECTL) {
    let mut bounds: *mut RECTL = core::ptr::addr_of_mut!((*(*emf).dc_attr).emf_bounds);
    let mut vport_rect: RECTL = *rect;
    LPtoDP(
        (*(*emf).dc_attr).hdc,
        core::ptr::addr_of_mut!(vport_rect) as LPPOINT,
        2,
    );

    /* The coordinate systems may be mirrored
       (LPtoDP handles points, not rectangles) */
    if vport_rect.left > vport_rect.right {
        let mut temp: LONG = vport_rect.right;
        vport_rect.right = vport_rect.left;
        vport_rect.left = temp;
    }
    if vport_rect.top > vport_rect.bottom {
        let mut temp_0: LONG = vport_rect.bottom;
        vport_rect.bottom = vport_rect.top;
        vport_rect.top = temp_0;
    }

    if (*bounds).left > (*bounds).right {
        /* first bounding rectangle */
        *bounds = vport_rect;
    } else {
        (*bounds).left = min((*bounds).left, vport_rect.left);
        (*bounds).top = min((*bounds).top, vport_rect.top);
        (*bounds).right = max((*bounds).right, vport_rect.right);
        (*bounds).bottom = max((*bounds).bottom, vport_rect.bottom);
    }
}

unsafe extern "C" fn get_bitmap_info(hdc: *mut HDC, bitmap: *mut HBITMAP, info: *mut BITMAPINFO) -> UINT {
    let mut blit_bitmap: HBITMAP = core::ptr::null_mut();
    let mut blit_dc: HDC = core::ptr::null_mut();
    let mut info_size: UINT;
    let mut bpp: UINT;
    let mut dib: DIBSECTION = core::mem::zeroed();

    info_size = GetObjectW(
        *bitmap as HANDLE,
        core::mem::size_of::<DIBSECTION>() as _,
        core::ptr::addr_of_mut!(dib) as LPVOID,
    ) as UINT;
    if info_size == 0 {
        return 0;
    }

    'err: {
        if info_size == core::mem::size_of::<DIBSECTION>() as UINT {
            blit_dc = *hdc;
            blit_bitmap = *bitmap;
        } else {
            let mut dib_info_buffer: [core::ffi::c_uchar; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize] =
                [0; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize];
            let mut dib_info: *mut BITMAPINFO =
                core::ptr::addr_of_mut!(dib_info_buffer) as *mut BITMAPINFO;
            let mut bmp: BITMAP = dib.dsBm;
            let mut palette: HPALETTE = core::ptr::null_mut();
            let mut bits: *mut core::ffi::c_void = core::ptr::null_mut();

            ASSERT!(info_size as usize == core::mem::size_of::<BITMAP>() as usize);

            (*dib_info).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
            (*dib_info).bmiHeader.biWidth = bmp.bmWidth;
            (*dib_info).bmiHeader.biHeight = bmp.bmHeight;
            (*dib_info).bmiHeader.biPlanes = 1;
            (*dib_info).bmiHeader.biBitCount = bmp.bmBitsPixel;
            (*dib_info).bmiHeader.biCompression = BI_RGB;
            (*dib_info).bmiHeader.biSizeImage = 0;
            (*dib_info).bmiHeader.biXPelsPerMeter = 0;
            (*dib_info).bmiHeader.biYPelsPerMeter = 0;
            (*dib_info).bmiHeader.biClrUsed = 0;
            (*dib_info).bmiHeader.biClrImportant = 0;
            match (*dib_info).bmiHeader.biBitCount as i32 {
                16 => {
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(0 as isize) = 0xf800 as DWORD;
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(1 as isize) = 0x07e0 as DWORD;
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(2 as isize) = 0x001f as DWORD;
                }
                32 => {
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(0 as isize) = 0xff0000 as DWORD;
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(1 as isize) = 0x00ff00 as DWORD;
                    *(core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut DWORD)
                        .offset(2 as isize) = 0x0000ff as DWORD;
                }
                _ => {
                    if !((*dib_info).bmiHeader.biBitCount as i32 > 8) {
                        palette = GetCurrentObject(*hdc, OBJ_PAL) as HPALETTE;
                        if palette.is_null() {
                            return FALSE;
                        }
                        if GetPaletteEntries(
                            palette,
                            0,
                            256,
                            core::ptr::addr_of_mut!((*dib_info).bmiColors) as *mut PALETTEENTRY,
                        ) == 0
                        {
                            return FALSE;
                        }
                    }
                }
            }

            blit_dc = /*NtGdi*/ CreateCompatibleDC(*hdc);
            if blit_dc.is_null() {
                return FALSE;
            }
            blit_bitmap = CreateDIBSection(
                blit_dc,
                dib_info,
                DIB_RGB_COLORS,
                core::ptr::addr_of_mut!(bits),
                core::ptr::null_mut(),
                0,
            ) as HBITMAP;
            if blit_bitmap.is_null() {
                break 'err;
            }
            if SelectObject(blit_dc, blit_bitmap as HGDIOBJ).is_null() {
                break 'err;
            }
            if BitBlt(
                blit_dc,
                0,
                0,
                bmp.bmWidth as INT,
                bmp.bmHeight as INT,
                *hdc,
                0,
                0,
                SRCCOPY,
            ) == 0
            {
                break 'err;
            }
        }

        if GetDIBits(
            blit_dc,
            blit_bitmap,
            0,
            INT_MAX,
            core::ptr::null_mut(),
            info,
            DIB_RGB_COLORS,
        ) == 0
        {
            break 'err;
        }

        bpp = (*info).bmiHeader.biBitCount as UINT;
        if bpp <= 8 {
            return (core::mem::size_of::<BITMAPINFOHEADER>()
                + ((1 << bpp) as usize) * core::mem::size_of::<RGBQUAD>())
                as UINT;
        } else if bpp == 16 || bpp == 32 {
            return (core::mem::size_of::<BITMAPINFOHEADER>()
                + (3 as usize) * core::mem::size_of::<RGBQUAD>())
                as UINT;
        }

        return core::mem::size_of::<BITMAPINFOHEADER>() as UINT;
    }

    // err:
    if !blit_dc.is_null() && blit_dc != *hdc {
        DeleteDC(blit_dc);
    }
    if !blit_bitmap.is_null() && blit_bitmap != *bitmap {
        DeleteObject(blit_bitmap as HGDIOBJ);
    }
    return 0;
}

unsafe extern "C" fn emfdc_add_handle(emf: *mut emf, obj: HGDIOBJ) -> UINT {
    let mut index: UINT;
    index = 0 as UINT;
    while index < (*emf).handles_size {
        if *(*emf).handles.offset(index as isize) == core::ptr::null_mut() {
            break;
        }
        index += 1;
    }
    if index == (*emf).handles_size {
        (*emf).handles_size += HANDLE_LIST_INC as UINT;
        (*emf).handles = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (*emf).handles as LPVOID,
            ((*emf).handles_size as SIZE_T)
                .wrapping_mul(core::mem::size_of_val(&*(*emf).handles.offset(0 as isize)) as SIZE_T),
        ) as *mut HGDIOBJ;
    }
    *(*emf).handles.offset(index as isize) = get_full_gdi_handle(obj);

    (*emf).cur_handles += 1;
    if (*emf).cur_handles > (*(*emf).emh).nHandles as UINT {
        (*(*emf).emh).nHandles += 1;
    }

    return index.wrapping_add(1 as UINT); /* index 0 is reserved for the hmf, so we increment everything by 1 */
}

unsafe extern "C" fn emfdc_find_object(mut emf: *mut emf, mut obj: HGDIOBJ) -> UINT {
    let mut index: UINT;
    index = 0 as UINT;
    while index < (*emf).handles_size {
        if *(*emf).handles.offset(index as isize) == obj {
            return index.wrapping_add(1 as UINT);
        }
        index = index.wrapping_add(1);
    }
    return 0 as UINT;
}
#[no_mangle]
pub unsafe extern "C" fn emfdc_delete_object(hdc: HDC, obj: HGDIOBJ) {
    let dc_attr: *mut WINEDC = get_dc_ptr(hdc) as *mut WINEDC;
    let emf: *mut emf = ((*dc_attr).emf).cast();
    let mut emr: EMRDELETEOBJECT = core::mem::zeroed();
    let mut index: UINT = 0;

    index = emfdc_find_object(emf, obj);
    if index == 0 {
        return;
    }

    emr.emr.iType = EMR_DELETEOBJECT;
    emr.emr.nSize = core::mem::size_of::<EMRDELETEOBJECT>() as DWORD;
    emr.ihObject = index;

    emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));

    *(*emf).handles.offset((index - 1) as isize) = core::ptr::null_mut();
    (*emf).cur_handles = (*emf).cur_handles.wrapping_sub(1);
}

unsafe extern "C" fn emfdc_create_brush(mut emf: *mut emf, mut brush: HBRUSH) -> DWORD {
    let mut index: DWORD = 0 as DWORD;
    let mut logbrush: LOGBRUSH = tagLOGBRUSH {
        lbStyle: 0,
        lbColor: 0,
        lbHatch: 0,
    };
    if GetObjectA(
        brush as HANDLE,
        ::core::mem::size_of::<LOGBRUSH>() as ::core::ffi::c_int,
        &raw mut logbrush as LPVOID,
    ) == 0
    {
        return 0 as DWORD;
    }
    match logbrush.lbStyle {
        BS_SOLID | BS_HATCHED | BS_NULL => {
            let mut emr: EMRCREATEBRUSHINDIRECT = tagEMRCREATEBRUSHINDIRECT {
                emr: tagEMR { iType: 0, nSize: 0 },
                ihBrush: 0,
                lb: tagLOGBRUSH32 {
                    lbStyle: 0,
                    lbColor: 0,
                    lbHatch: 0,
                },
            };
            emr.emr.iType = EMR_CREATEBRUSHINDIRECT as DWORD;
            emr.emr.nSize = ::core::mem::size_of::<EMRCREATEBRUSHINDIRECT>() as DWORD;
            index = emfdc_add_handle(emf, brush as HGDIOBJ) as DWORD;
            emr.ihBrush = index;
            emr.lb.lbStyle = logbrush.lbStyle;
            emr.lb.lbColor = logbrush.lbColor;
            emr.lb.lbHatch = logbrush.lbHatch as ULONG;
            if emfdc_record(emf, &raw mut emr.emr) == 0 {
                index = 0 as DWORD;
            }
        }
        BS_PATTERN | BS_DIBPATTERN => {
            let mut buffer: [::core::ffi::c_uchar; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize] =
                [0; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize];
            let mut info: *mut BITMAPINFO =
                &raw mut buffer as *mut ::core::ffi::c_uchar as *mut BITMAPINFO;
            let mut emr_0: *mut EMRCREATEDIBPATTERNBRUSHPT =
                ::core::ptr::null_mut::<EMRCREATEDIBPATTERNBRUSHPT>();
            let mut info_size: DWORD = 0;
            let mut usage: UINT = 0;
            if !(get_brush_bitmap_info(brush, info, NULL, &raw mut usage) == 0) {
                info_size = get_dib_info_size(info, usage) as DWORD;
                emr_0 = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    (::core::mem::size_of::<EMRCREATEDIBPATTERNBRUSHPT>() as SIZE_T)
                        .wrapping_add(::core::mem::size_of::<DWORD>() as SIZE_T)
                        .wrapping_add(info_size as SIZE_T)
                        .wrapping_add((*info).bmiHeader.biSizeImage as SIZE_T),
                ) as *mut EMRCREATEDIBPATTERNBRUSHPT;
                if !emr_0.is_null() {
                    (*emr_0).offBmi = (::core::mem::size_of::<EMRCREATEDIBPATTERNBRUSHPT>() as usize)
                        .wrapping_add(::core::mem::size_of::<DWORD>() as usize)
                        as DWORD;
                    *(emr_0.offset(1 as isize) as *mut DWORD) =
                        0x20000000 as DWORD;
                    if logbrush.lbStyle == BS_PATTERN as UINT
                        && (*info).bmiHeader.biBitCount as ::core::ffi::c_int
                            == 1 as ::core::ffi::c_int
                    {
                        (*emr_0).emr.iType = EMR_CREATEMONOBRUSH as DWORD;
                        usage = DIB_PAL_MONO as UINT;
                        (*emr_0).cbBmi = ::core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
                    } else {
                        (*emr_0).emr.iType = EMR_CREATEDIBPATTERNBRUSHPT as DWORD;
                        (*emr_0).cbBmi = info_size;
                    }
                    index = emfdc_add_handle(emf, brush as HGDIOBJ) as DWORD;
                    (*emr_0).ihBrush = index;
                    (*emr_0).iUsage = usage as DWORD;
                    (*emr_0).offBits = (*emr_0).offBmi.wrapping_add((*emr_0).cbBmi);
                    (*emr_0).cbBits = (*info).bmiHeader.biSizeImage;
                    (*emr_0).emr.nSize = (*emr_0).offBits.wrapping_add((*emr_0).cbBits);
                    if (*info).bmiHeader.biClrUsed
                        == ((1 as ::core::ffi::c_int)
                            << (*info).bmiHeader.biBitCount as ::core::ffi::c_int)
                            as DWORD
                    {
                        (*info).bmiHeader.biClrUsed = 0 as DWORD;
                    }
                    memcpy(
                        (emr_0 as *mut BYTE).offset((*emr_0).offBmi as isize)
                            as *mut ::core::ffi::c_void,
                        info as *const ::core::ffi::c_void,
                        (*emr_0).cbBmi as size_t,
                    );
                    get_brush_bitmap_info(
                        brush,
                        ::core::ptr::null_mut::<BITMAPINFO>(),
                        (emr_0 as *mut ::core::ffi::c_char).offset((*emr_0).offBits as isize)
                            as *mut ::core::ffi::c_void,
                        ::core::ptr::null_mut::<UINT>(),
                    );
                    if emfdc_record(emf, &raw mut (*emr_0).emr) == 0 {
                        index = 0 as DWORD;
                    }
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        emr_0 as PVOID,
                    );
                }
            }
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/gdi/gdi32/wine/emfdc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"emfdc_create_brush\0" as *const u8 as *const ::core::ffi::c_char,
                    334 as ::core::ffi::c_int,
                    b"Unknown style %x\n\0" as *const u8 as *const ::core::ffi::c_char,
                    logbrush.lbStyle,
                );
            }
        }
    }
    return index;
}

unsafe extern "C" fn emfdc_select_brush(dc_attr: *mut WINEDC, brush: HBRUSH) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTOBJECT = tagEMRSELECTOBJECT {
        emr: tagEMR { iType: 0, nSize: 0 },
        ihObject: 0,
    };
    let mut index: DWORD = 0 as DWORD;
    let mut i: ::core::ffi::c_int = 0;

    i = WHITE_BRUSH as ::core::ffi::c_int;
    while i <= DC_BRUSH as ::core::ffi::c_int {
        if brush == GetStockObject(i) as HBRUSH {
            index = (i as DWORD) | (0x80000000 as DWORD);
            break;
        } else {
            i += 1;
        }
    }

    if index == 0 && {
        index = emfdc_find_object(emf, brush as HGDIOBJ) as DWORD;
        index == 0
    } {
        index = emfdc_create_brush(emf, brush);
        if index == 0 {
            return 0 as BOOL;
        }
        GDI_hdc_using_object(brush as HGDIOBJ, (*dc_attr).hdc);
    }

    emr.emr.iType = EMR_SELECTOBJECT as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRSELECTOBJECT>() as DWORD;
    emr.ihObject = index;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}

unsafe extern "C" fn emfdc_create_font(emf: *mut emf, font: HFONT) -> BOOL {
    let mut index: DWORD = 0 as DWORD;
    let mut emr: EMREXTCREATEFONTINDIRECTW = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;

    if GetObjectW(
        font as HANDLE,
        ::core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        &raw mut emr.elfw.elfLogFont as LPVOID,
    ) == 0
    {
        return FALSE as BOOL;
    }

    emr.emr.iType = EMR_EXTCREATEFONTINDIRECTW as DWORD;
    emr.emr.nSize = (::core::mem::size_of::<EMREXTCREATEFONTINDIRECTW>() as usize)
        .wrapping_add(3 as usize)
        .wrapping_div(4 as usize)
        .wrapping_mul(4 as usize) as DWORD;
    index = emfdc_add_handle(emf, font as HGDIOBJ) as DWORD;
    emr.ihFont = index;
    emr.elfw.elfFullName[0 as usize] = '\0' as i32 as WCHAR;
    emr.elfw.elfStyle[0 as usize] = '\0' as i32 as WCHAR;
    emr.elfw.elfVersion = 0 as DWORD;
    emr.elfw.elfStyleSize = 0 as DWORD;
    emr.elfw.elfMatch = 0 as DWORD;
    emr.elfw.elfReserved = 0 as DWORD;
    i = 0 as ::core::ffi::c_int;
    while i < ELF_VENDOR_SIZE as ::core::ffi::c_int {
        emr.elfw.elfVendorId[i as usize] = 0 as BYTE;
        i += 1;
    }
    emr.elfw.elfCulture = PAN_CULTURE_LATIN as DWORD;
    emr.elfw.elfPanose.bFamilyType = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bSerifStyle = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bWeight = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bProportion = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bContrast = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bStrokeVariation = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bArmStyle = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bLetterform = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bMidline = PAN_NO_FIT as BYTE;
    emr.elfw.elfPanose.bXHeight = PAN_NO_FIT as BYTE;

    return (if emfdc_record(emf, &raw mut emr.emr) != 0 {
        index
    } else {
        0 as DWORD
    }) as BOOL;
}

unsafe extern "C" fn emfdc_select_font(dc_attr: *mut WINEDC, font: HFONT) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTOBJECT = tagEMRSELECTOBJECT {
        emr: tagEMR { iType: 0, nSize: 0 },
        ihObject: 0,
    };
    let mut index: DWORD = 0;
    let mut i: ::core::ffi::c_int = 0;

    i = (OEM_FIXED_FONT as i32);
    'found: {
        while i <= (DEFAULT_GUI_FONT as i32) {
            if i != (DEFAULT_PALETTE as i32) && font == GetStockObject(i) as HFONT {
                index = (i as ::core::ffi::c_uint | 0x80000000 as ::core::ffi::c_uint) as DWORD;
                break 'found;
            }
            i += 1;
        }

        index = emfdc_find_object(emf, font as HGDIOBJ) as DWORD;
        if index == 0 {
            index = emfdc_create_font(emf, font) as DWORD;
            if index == 0 {
                return (FALSE) as BOOL;
            }
            GDI_hdc_using_object(font as HGDIOBJ, (*dc_attr).hdc);
        }
    }

    emr.emr.iType = EMR_SELECTOBJECT as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRSELECTOBJECT>() as DWORD;
    emr.ihObject = index;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}

unsafe extern "C" fn emfdc_create_pen(mut emf: *mut emf, mut hPen: HPEN) -> DWORD {
    let mut emr: EMRCREATEPEN = tagEMRCREATEPEN {
        emr: tagEMR { iType: 0, nSize: 0 },
        ihPen: 0,
        lopn: tagLOGPEN {
            lopnStyle: 0,
            lopnWidth: tagPOINT { x: 0, y: 0 },
            lopnColor: 0,
        },
    };
    let mut index: DWORD = 0;
    if GetObjectW(
        hPen as HANDLE,
        ::core::mem::size_of::<LOGPEN>() as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(emr.lopn) as LPVOID,
    ) == 0
    {
        /* must be an extended pen */
        let mut elp: *mut EXTLOGPEN = core::ptr::null_mut::<EXTLOGPEN>();
        let mut size: INT = GetObjectW(
            hPen as HANDLE,
            0 as ::core::ffi::c_int,
            core::ptr::null_mut(),
        ) as INT;

        if size == 0 {
            return 0;
        }

        elp = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut EXTLOGPEN;

        GetObjectW(
            hPen as HANDLE,
            size as ::core::ffi::c_int,
            elp as LPVOID,
        );
        /* FIXME: add support for user style pens */
        emr.lopn.lopnStyle = (*elp).elpPenStyle as UINT;
        emr.lopn.lopnWidth.x = (*elp).elpWidth as LONG;
        emr.lopn.lopnWidth.y = 0;
        emr.lopn.lopnColor = (*elp).elpColor;

        HeapFree(
            GetProcessHeap(),
            0,
            elp as PVOID,
        );
    }

    emr.emr.iType = EMR_CREATEPEN;
    emr.emr.nSize = ::core::mem::size_of::<EMRCREATEPEN>() as DWORD;
    index = emfdc_add_handle(emf, hPen as HGDIOBJ) as DWORD;
    emr.ihPen = index;
    return if emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr)) != 0 {
        index
    } else {
        0
    };
}

unsafe extern "C" fn emfdc_select_pen(dc_attr: *mut WINEDC, pen: HPEN) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTOBJECT = core::mem::zeroed();
    let mut index: DWORD = 0 as DWORD;
    let mut i: i32 = WHITE_PEN as i32;

    /* If the object is a stock pen object, do not need to create it.
     * See definitions in  wingdi.h for range of stock pens.
     * We do however have to handle setting the higher order bit to
     * designate that this is a stock object.
     */

    while i <= DC_PEN as i32 {
        if pen == GetStockObject(i) as HPEN {
            index = ((i as u32) | (0x80000000 as u32)) as DWORD;
            break;
        } else {
            i += 1;
        }
    }
    if index == 0 && {
        index = emfdc_find_object(emf, pen as HGDIOBJ) as DWORD;
        index == 0
    } {
        index = emfdc_create_pen(emf, pen) as DWORD;
        if index == 0 {
            return FALSE as BOOL;
        }
        GDI_hdc_using_object(pen as HGDIOBJ, (*dc_attr).hdc);
    }

    emr.emr.iType = EMR_SELECTOBJECT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSELECTOBJECT>() as DWORD;
    emr.ihObject = index;
    return emfdc_record(emf, &raw mut emr.emr) as BOOL;
}

// emfdc_create_palette's anonymous local `struct { EMRCREATEPALETTE hdr; PALETTEENTRY entry[255]; }`.
#[derive(Copy, Clone)]
#[repr(C)]
struct C2RustUnnamed_25 {
    hdr: EMRCREATEPALETTE,
    entry: [PALETTEENTRY; 255],
}

unsafe extern "C" fn emfdc_create_palette(mut emf: *mut emf, mut hPal: HPALETTE) -> DWORD {
    let mut i: WORD = 0 as WORD;
    let mut pal: C2RustUnnamed_25 = core::mem::zeroed();

    memset(
        &raw mut pal as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<C2RustUnnamed_25>() as usize,
    );

    if GetObjectW(
        hPal as HANDLE,
        (core::mem::size_of::<LOGPALETTE>() as usize)
            .wrapping_add(core::mem::size_of::<[PALETTEENTRY; 255]>() as usize)
            as core::ffi::c_int,
        &raw mut pal.hdr.lgpl as LPVOID,
    ) == 0
    {
        return 0 as DWORD;
    }

    i = 0 as WORD;
    while (i as core::ffi::c_int) < pal.hdr.lgpl.palNumEntries as core::ffi::c_int {
        (*(&raw mut pal.hdr.lgpl.palPalEntry as *mut PALETTEENTRY).offset(i as isize)).peFlags =
            0 as BYTE;
        i = i.wrapping_add(1);
    }

    pal.hdr.emr.iType = EMR_CREATEPALETTE as DWORD;
    pal.hdr.emr.nSize = (core::mem::size_of::<EMRCREATEPALETTE>() as usize).wrapping_add(
        (pal.hdr.lgpl.palNumEntries as usize)
            .wrapping_mul(core::mem::size_of::<PALETTEENTRY>() as usize),
    ) as DWORD;
    pal.hdr.ihPal = emfdc_add_handle(emf, hPal as HGDIOBJ) as DWORD;

    if emfdc_record(emf, &raw mut pal.hdr.emr) == 0 {
        pal.hdr.ihPal = 0 as DWORD;
    }
    return pal.hdr.ihPal;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SelectPalette(dc_attr: *mut WINEDC, palette: HPALETTE) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTPALETTE = core::mem::zeroed();
    let mut index: DWORD = 0 as DWORD;

    if palette == GetStockObject((DEFAULT_PALETTE as i32)) as HPALETTE {
        index = (DEFAULT_PALETTE as DWORD) | (0x80000000 as DWORD);
    } else {
        index = emfdc_find_object(emf, palette as HGDIOBJ) as DWORD;
        if index == 0 {
            index = emfdc_create_palette(emf, palette);
            if index == 0 {
                return 0 as BOOL;
            }
            GDI_hdc_using_object(palette as HGDIOBJ, (*dc_attr).hdc);
        }
    }

    emr.emr.iType = EMR_SELECTPALETTE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSELECTPALETTE>() as DWORD;
    emr.ihPal = index;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_SelectObject(dc_attr: *mut WINEDC, obj: HGDIOBJ) -> BOOL {
    match GDI_HANDLE_GET_TYPE!(obj) {
        1048576 => {
            // GDILoObjType_LO_BRUSH_TYPE
            return emfdc_select_brush(dc_attr, obj as HBRUSH);
        }
        655360 => {
            // GDILoObjType_LO_FONT_TYPE
            return emfdc_select_font(dc_attr, obj as HFONT);
        }
        3145728 | 5242880 => {
            // GDILoObjType_LO_PEN_TYPE, GDILoObjType_LO_EXTPEN_TYPE
            return emfdc_select_pen(dc_attr, obj as HPEN);
        }
        _ => {
            return (TRUE) as BOOL;
        }
    }
}

unsafe extern "C" fn can_use_short_points(pts: *const POINT, count: UINT) -> BOOL {
    let mut i: UINT;
    i = 0;
    while i < count {
        if (((*pts.offset(i as isize)).x + 0x8000) & !(0xffff as LONG)) != 0
            || (((*pts.offset(i as isize)).y + 0x8000) & !(0xffff as LONG)) != 0
        {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn store_points(
    dest: *mut POINTL,
    pts: *const POINT,
    count: UINT,
    short_points: BOOL,
) -> *mut core::ffi::c_void {
    if short_points != 0 {
        let mut i: UINT;
        let dest_short: *mut POINTS = dest as *mut POINTS;
        i = 0 as UINT;
        while i < count {
            (*dest_short.offset(i as isize)).x = (*pts.offset(i as isize)).x as SHORT;
            (*dest_short.offset(i as isize)).y = (*pts.offset(i as isize)).y as SHORT;
            i = i.wrapping_add(1);
        }
        return dest_short.offset(count as isize) as *mut core::ffi::c_void;
    } else {
        memcpy(
            dest as *mut core::ffi::c_void,
            pts as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<POINTL>()),
        );
        return dest.offset(count as isize) as *mut core::ffi::c_void;
    }
}

unsafe extern "C" fn get_points_bounds(
    bounds: *mut RECTL,
    pts: *const POINT,
    count: UINT,
    dc_attr: *mut WINEDC,
) {
    let mut i: UINT;
    if !dc_attr.is_null() {
        let mut cur_pos: POINT = POINT { x: 0, y: 0 };
        GetCurrentPositionEx((*dc_attr).hdc, core::ptr::addr_of_mut!(cur_pos));
        (*bounds).left = cur_pos.x;
        (*bounds).right = (*bounds).left;
        (*bounds).top = cur_pos.y;
        (*bounds).bottom = (*bounds).top;
    } else if count != 0 {
        (*bounds).left = (*pts.offset(0 as isize)).x;
        (*bounds).right = (*bounds).left;
        (*bounds).top = (*pts.offset(0 as isize)).y;
        (*bounds).bottom = (*bounds).top;
    } else {
        *bounds = empty_bounds;
    }
    i = 0 as UINT;
    while i < count {
        (*bounds).left = min((*bounds).left, (*pts.offset(i as isize)).x);
        (*bounds).right = max((*bounds).right, (*pts.offset(i as isize)).x);
        (*bounds).top = min((*bounds).top, (*pts.offset(i as isize)).y);
        (*bounds).bottom = max((*bounds).bottom, (*pts.offset(i as isize)).y);
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn emfdrv_stroke_and_fill_path(emf: *mut emf, type_: INT) -> BOOL {
    let mut emr: EMRSTROKEANDFILLPATH = core::mem::zeroed();
    let mut Points: LPPOINT = core::ptr::null_mut();
    let mut Types: LPBYTE = core::ptr::null_mut();
    let mut nSize: INT = 0;

    emr.emr.iType = type_ as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSTROKEANDFILLPATH>() as DWORD;

    nSize = GetPath(
        (*(*emf).dc_attr).hdc,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    ) as INT;
    if nSize != -(1) {
        Points = HeapAlloc(
            GetProcessHeap(),
            0,
            (nSize as SIZE_T).wrapping_mul(core::mem::size_of::<POINT>() as SIZE_T),
        ) as LPPOINT;
        Types = HeapAlloc(
            GetProcessHeap(),
            0,
            (nSize as SIZE_T).wrapping_mul(core::mem::size_of::<BYTE>() as SIZE_T),
        ) as LPBYTE;

        GetPath((*(*emf).dc_attr).hdc, Points, Types, nSize);
        get_points_bounds(
            &raw mut emr.rclBounds,
            Points as *const POINT,
            nSize as UINT,
            core::ptr::null_mut(),
        );

        HeapFree(GetProcessHeap(), 0, Points as PVOID);
        HeapFree(GetProcessHeap(), 0, Types as PVOID);
    } else {
        emr.rclBounds = empty_bounds;
    }

    if emfdc_record(emf, &raw mut emr.emr) == 0 {
        return (FALSE) as BOOL;
    }
    if nSize == -(1) {
        return (FALSE) as BOOL;
    }
    emfdc_update_bounds(emf, &raw mut emr.rclBounds);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_MoveTo(dc_attr: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRMOVETOEX = core::mem::zeroed();
    emr.emr.iType = EMR_MOVETOEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRMOVETOEX>() as DWORD;
    emr.ptl.x = x as LONG;
    emr.ptl.y = y as LONG;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_LineTo(dc_attr: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let mut emr: EMRLINETO = tagEMRLINETO {
        emr: tagEMR { iType: 0, nSize: 0 },
        ptl: _POINTL { x: 0, y: 0 },
    };
    let mut Ret: BOOL = 0;
    emr.emr.iType = EMR_LINETO as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRLINETO>() as DWORD;
    emr.ptl.x = x as LONG;
    emr.ptl.y = y as LONG;
    Ret = emfdc_record((*dc_attr).emf as *mut emf, &raw mut emr.emr);
    EMFDRV_LineTo(dc_attr, x, y);
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ArcChordPie(
    mut dc_attr: *mut WINEDC,
    mut left: INT,
    mut top: INT,
    mut right: INT,
    mut bottom: INT,
    mut xstart: INT,
    mut ystart: INT,
    mut xend: INT,
    mut yend: INT,
    mut type_0: DWORD,
) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRARC = tagEMRARC {
        emr: tagEMR { iType: 0, nSize: 0 },
        rclBox: _RECTL {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        ptlStart: _POINTL { x: 0, y: 0 },
        ptlEnd: _POINTL { x: 0, y: 0 },
    };
    let mut temp: INT = 0;
    let mut Ret: BOOL = 0;
    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }
    if left > right {
        temp = left;
        left = right;
        right = temp;
    }
    if top > bottom {
        temp = top;
        top = bottom;
        bottom = temp;
    }
    if GetGraphicsMode((*dc_attr).hdc) == (GM_COMPATIBLE as i32) {
        right -= 1;
        bottom -= 1;
    }
    emr.emr.iType = type_0;
    emr.emr.nSize = ::core::mem::size_of::<EMRARC>() as DWORD;
    emr.rclBox.left = left as LONG;
    emr.rclBox.top = top as LONG;
    emr.rclBox.right = right as LONG;
    emr.rclBox.bottom = bottom as LONG;
    emr.ptlStart.x = xstart as LONG;
    emr.ptlStart.y = ystart as LONG;
    emr.ptlEnd.x = xend as LONG;
    emr.ptlEnd.y = yend as LONG;
    Ret = emfdc_record(emf, &raw mut emr.emr);
    EMFDRV_ArcChordPie(
        dc_attr, left, top, right, bottom, xstart, ystart, xend, yend, type_0,
    );
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_AngleArc(dc_attr: *mut WINEDC, x: INT, y: INT, radius: DWORD, start: FLOAT, sweep: FLOAT) -> BOOL {
    let mut emr: EMRANGLEARC = core::mem::zeroed();

    emr.emr.iType = EMR_ANGLEARC as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRANGLEARC>() as DWORD;
    emr.ptlCenter.x = x as LONG;
    emr.ptlCenter.y = y as LONG;
    emr.nRadius = radius;
    emr.eStartAngle = start;
    emr.eSweepAngle = sweep;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_Ellipse(
    dc_attr: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRELLIPSE = tagEMRELLIPSE {
        emr: tagEMR { iType: 0, nSize: 0 },
        rclBox: _RECTL {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    };
    let mut Ret: BOOL = 0;
    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }
    emr.emr.iType = EMR_ELLIPSE as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRELLIPSE>() as DWORD;
    emr.rclBox.left = min(left, right) as LONG;
    emr.rclBox.top = min(top, bottom) as LONG;
    emr.rclBox.right = max(left, right) as LONG;
    emr.rclBox.bottom = max(top, bottom) as LONG;
    if GetGraphicsMode((*dc_attr).hdc) == (GM_COMPATIBLE as i32) {
        emr.rclBox.right -= 1;
        emr.rclBox.bottom -= 1;
    }
    Ret = emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
    EMFDRV_Ellipse(dc_attr, left, top, right, bottom);
    return Ret;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_Rectangle(
    dc_attr: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRRECTANGLE = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }

    emr.emr.iType = EMR_RECTANGLE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRRECTANGLE>() as DWORD;
    emr.rclBox.left = min(left, right) as LONG;
    emr.rclBox.top = min(top, bottom) as LONG;
    emr.rclBox.right = max(left, right) as LONG;
    emr.rclBox.bottom = max(top, bottom) as LONG;
    if GetGraphicsMode((*dc_attr).hdc) == (GM_COMPATIBLE as i32) {
        emr.rclBox.right -= 1;
        emr.rclBox.bottom -= 1;
    }
    Ret = emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
    EMFDRV_Rectangle(dc_attr, left, top, right, bottom);
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_RoundRect(dc_attr: *mut WINEDC, left: INT, top: INT, right: INT, bottom: INT, ell_width: INT, ell_height: INT) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRROUNDRECT = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    if left == right || top == bottom {
        return (FALSE) as BOOL;
    }

    emr.emr.iType = EMR_ROUNDRECT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRROUNDRECT>() as DWORD;
    emr.rclBox.left = min(left, right) as LONG;
    emr.rclBox.top = min(top, bottom) as LONG;
    emr.rclBox.right = max(left, right) as LONG;
    emr.rclBox.bottom = max(top, bottom) as LONG;
    emr.szlCorner.cx = ell_width as LONG;
    emr.szlCorner.cy = ell_height as LONG;
    if GetGraphicsMode((*dc_attr).hdc) == (GM_COMPATIBLE as i32) {
        emr.rclBox.right -= 1;
        emr.rclBox.bottom -= 1;
    }
    Ret = emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
    EMFDRV_RoundRect(dc_attr, left, top, right, bottom, ell_width, ell_height);
    return Ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetPixel(
    dc_attr: *mut WINEDC,
    x: INT,
    y: INT,
    color: COLORREF,
) -> BOOL {
    let mut emr: EMRSETPIXELV = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    emr.emr.iType = EMR_SETPIXELV as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETPIXELV>() as DWORD;
    emr.ptlPixel.x = x;
    emr.ptlPixel.y = y;
    emr.crColor = color;
    Ret = emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
    EMFDRV_SetPixel(dc_attr, x, y, color);
    return Ret;
}

unsafe extern "C" fn emfdc_polylinegon(dc_attr: *mut WINEDC, points: *const POINT, count: INT, type_: DWORD) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: *mut EMRPOLYLINE = core::ptr::null_mut::<EMRPOLYLINE>();
    let mut size: DWORD;
    let mut ret: BOOL;
    let use_small_emr: BOOL = can_use_short_points(points, count as UINT);

    size = if use_small_emr != 0 {
        (core::mem::offset_of!(EMRPOLYLINE16, apts) + (count as usize) * core::mem::size_of::<POINTS>()) as DWORD
    } else {
        (core::mem::offset_of!(EMRPOLYLINE, aptl) + (count as usize) * core::mem::size_of::<POINTL>()) as DWORD
    };

    emr = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut EMRPOLYLINE;
    (*emr).emr.iType = if use_small_emr != 0 {
        type_.wrapping_add(EMR_POLYLINE16 as DWORD).wrapping_sub(EMR_POLYLINE as DWORD)
    } else {
        type_
    };
    (*emr).emr.nSize = size;
    (*emr).cptl = count as DWORD;

    store_points(
        core::ptr::addr_of_mut!((*emr).aptl) as *mut POINTL,
        points,
        count as UINT,
        use_small_emr,
    );

    if (*emf).path == 0 {
        get_points_bounds(
            core::ptr::addr_of_mut!((*emr).rclBounds),
            points,
            count as UINT,
            if type_ == EMR_POLYBEZIERTO as DWORD || type_ == EMR_POLYLINETO as DWORD {
                dc_attr
            } else {
                (0) as *mut WINEDC
            },
        );
    } else {
        (*emr).rclBounds = empty_bounds;
    }

    ret = emfdc_record(emf, core::ptr::addr_of_mut!((*emr).emr));
    if ret != 0 && (*emf).path == 0 {
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, emr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_Polyline(
    mut dc_attr: *mut WINEDC,
    mut points: *const POINT,
    mut count: INT,
) -> BOOL {
    return emfdc_polylinegon(dc_attr, points, count, EMR_POLYLINE);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolylineTo(dc_attr: *mut WINEDC, points: *const POINT, count: INT) -> BOOL {
    return emfdc_polylinegon(dc_attr, points, count, EMR_POLYLINETO);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_Polygon(dc_attr: *mut WINEDC, pt: *const POINT, count: INT) -> BOOL {
    if count < 2 {
        return (FALSE) as BOOL;
    }
    return emfdc_polylinegon(dc_attr, pt, count, EMR_POLYGON as DWORD);
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolyBezier(
    dc_attr: *mut WINEDC,
    pts: *const POINT,
    count: DWORD,
) -> BOOL {
    return emfdc_polylinegon(dc_attr, pts, count as INT, EMR_POLYBEZIER as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolyBezierTo(dc_attr: *mut WINEDC, pts: *const POINT, count: DWORD) -> BOOL {
    return emfdc_polylinegon(dc_attr, pts, count as INT, EMR_POLYBEZIERTO as DWORD);
}

unsafe extern "C" fn emfdc_poly_polylinegon(
    mut emf: *mut emf,
    mut pt: *const POINT,
    mut counts: *const INT,
    mut polys: UINT,
    mut type_0: DWORD,
) -> BOOL {
    let mut emr: *mut EMRPOLYPOLYLINE = core::ptr::null_mut::<EMRPOLYPOLYLINE>();
    let mut cptl: DWORD = 0;
    let mut poly: DWORD = 0;
    let mut size: DWORD = 0;
    let mut ret: BOOL = 0;
    let mut use_small_emr: BOOL = 0;
    let mut bounds_valid: BOOL = (TRUE) as i32;
    while poly < polys as DWORD {
        cptl = cptl.wrapping_add(*counts.offset(poly as isize) as DWORD);
        if *counts.offset(poly as isize) < 2 {
            bounds_valid = (FALSE) as i32;
        }
        poly = poly.wrapping_add(1);
    }
    if cptl == 0 {
        bounds_valid = (FALSE) as i32;
    }
    use_small_emr = can_use_short_points(pt, cptl as UINT);

    size = (core::mem::offset_of!(EMRPOLYPOLYLINE, aPolyCounts)
        + (polys as usize).wrapping_mul(core::mem::size_of::<DWORD>())) as DWORD;
    if use_small_emr != 0 {
        size = size.wrapping_add(
            (cptl as usize).wrapping_mul(core::mem::size_of::<POINTS>()) as DWORD,
        );
    } else {
        size = size.wrapping_add(
            (cptl as usize).wrapping_mul(core::mem::size_of::<POINTL>()) as DWORD,
        );
    }

    emr = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut EMRPOLYPOLYLINE;

    (*emr).emr.iType = type_0;
    if use_small_emr != 0 {
        (*emr).emr.iType = (*emr).emr.iType.wrapping_add(
            (EMR_POLYPOLYLINE16 - EMR_POLYPOLYLINE) as DWORD,
        );
    }

    (*emr).emr.nSize = size;
    if bounds_valid != 0 && (*emf).path == 0 {
        get_points_bounds(
            &raw mut (*emr).rclBounds,
            pt,
            cptl as UINT,
            core::ptr::null_mut(),
        );
    } else {
        (*emr).rclBounds = empty_bounds;
    }
    (*emr).nPolys = polys as DWORD;
    (*emr).cptl = cptl;

    if polys != 0 {
        memcpy(
            &raw mut (*emr).aPolyCounts as *mut DWORD as *mut core::ffi::c_void,
            counts as *const core::ffi::c_void,
            (polys as usize).wrapping_mul(core::mem::size_of::<DWORD>()),
        );
        store_points(
            (&raw mut (*emr).aPolyCounts as *mut DWORD).offset(polys as isize) as *mut POINTL,
            pt,
            cptl as UINT,
            use_small_emr,
        );
    }

    ret = emfdc_record(emf, &raw mut (*emr).emr);
    if ret != 0 && bounds_valid == 0 {
        ret = (FALSE) as i32;
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    }
    if ret != 0 && (*emf).path == 0 {
        emfdc_update_bounds(emf, &raw mut (*emr).rclBounds);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolyPolyline(
    dc_attr: *mut WINEDC,
    pt: *const POINT,
    counts: *const DWORD,
    polys: DWORD,
) -> BOOL {
    return emfdc_poly_polylinegon(
        (*dc_attr).emf as *mut emf,
        pt,
        counts as *const INT,
        polys as UINT,
        EMR_POLYPOLYLINE as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolyPolygon(
    mut dc_attr: *mut WINEDC,
    mut pt: *const POINT,
    mut counts: *const INT,
    mut polys: UINT,
) -> BOOL {
    return emfdc_poly_polylinegon(
        (*dc_attr).emf as *mut emf,
        pt,
        counts,
        polys,
        EMR_POLYPOLYGON as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PolyDraw(
    dc_attr: *mut WINEDC,
    pts: *const POINT,
    types: *const BYTE,
    count: DWORD,
) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: *mut EMRPOLYDRAW = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut types_dest: *mut BYTE = core::ptr::null_mut();
    let use_small_emr: BOOL = can_use_short_points(pts, count as UINT);
    let mut size: DWORD = 0;

    size = if use_small_emr != 0 {
        (core::mem::offset_of!(EMRPOLYDRAW16, apts) + (count as usize) * core::mem::size_of::<POINTS>()) as DWORD
    } else {
        (core::mem::offset_of!(EMRPOLYDRAW, aptl) + (count as usize) * core::mem::size_of::<POINTL>()) as DWORD
    };
    size = size.wrapping_add((count.wrapping_add(3)) & !(3));

    emr = HeapAlloc(GetProcessHeap(), 0, (size) as SIZE_T) as *mut EMRPOLYDRAW;
    if emr.is_null() {
        return (FALSE) as BOOL;
    }

    (*emr).emr.iType = (if use_small_emr != 0 {
        EMR_POLYDRAW16
    } else {
        EMR_POLYDRAW
    }) as DWORD;
    (*emr).emr.nSize = size;
    (*emr).cptl = count;

    types_dest = store_points(
        core::ptr::addr_of_mut!((*emr).aptl) as *mut POINTL,
        pts,
        count as UINT,
        use_small_emr,
    ) as *mut BYTE;
    memcpy(
        types_dest as *mut core::ffi::c_void,
        types as *const core::ffi::c_void,
        count as usize,
    );
    if (count & 3) != 0 {
        memset(
            types_dest.add(count as usize) as *mut core::ffi::c_void,
            0,
            (4 - (count & 3)) as usize,
        );
    }

    if (*emf).path == 0 {
        get_points_bounds(
            core::ptr::addr_of_mut!((*emr).rclBounds),
            pts,
            count as UINT,
            core::ptr::null_mut(),
        );
    } else {
        (*emr).rclBounds = empty_bounds;
    }

    ret = emfdc_record(emf, core::ptr::addr_of_mut!((*emr).emr));
    if ret != 0 && (*emf).path == 0 {
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(GetProcessHeap(), 0, emr as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_ExtFloodFill(
    dc_attr: *mut WINEDC,
    x: INT,
    y: INT,
    color: COLORREF,
    fill_type: UINT,
) -> BOOL {
    let mut emr: EMREXTFLOODFILL = core::mem::zeroed();

    emr.emr.iType = EMR_EXTFLOODFILL as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMREXTFLOODFILL>() as DWORD;
    emr.ptlStart.x = x as LONG;
    emr.ptlStart.y = y as LONG;
    emr.crColor = color;
    emr.iMode = fill_type as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_FillRgn(
    mut dc_attr: *mut WINEDC,
    mut hrgn: HRGN,
    mut hbrush: HBRUSH,
) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: *mut EMRFILLRGN = core::ptr::null_mut::<EMRFILLRGN>();
    let mut size: DWORD = 0;
    let mut rgnsize: DWORD = 0;
    let mut index: DWORD = 0;
    let mut ret: BOOL = 0;
    index = emfdc_create_brush(emf, hbrush);
    if index == 0 {
        return (FALSE) as BOOL;
    }
    rgnsize = GetRegionData(hrgn, 0 as DWORD, core::ptr::null_mut::<_RGNDATA>());
    size = rgnsize.wrapping_add(core::mem::offset_of!(EMRFILLRGN, RgnData) as DWORD);
    emr = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut EMRFILLRGN;

    GetRegionData(hrgn, rgnsize, &raw mut (*emr).RgnData as LPRGNDATA);

    (*emr).emr.iType = EMR_FILLRGN as DWORD;
    (*emr).emr.nSize = size;
    (*emr).rclBounds.left = (*(&raw mut (*emr).RgnData as *mut RGNDATA))
        .rdh
        .rcBound
        .left;
    (*emr).rclBounds.top = (*(&raw mut (*emr).RgnData as *mut RGNDATA)).rdh.rcBound.top;
    (*emr).rclBounds.right = (*(&raw mut (*emr).RgnData as *mut RGNDATA))
        .rdh
        .rcBound
        .right
        - (1 as LONG);
    (*emr).rclBounds.bottom = (*(&raw mut (*emr).RgnData as *mut RGNDATA))
        .rdh
        .rcBound
        .bottom
        - (1 as LONG);
    (*emr).cbRgnData = rgnsize;
    (*emr).ihBrush = index;

    ret = emfdc_record(emf, &raw mut (*emr).emr);
    if ret != 0 {
        emfdc_update_bounds(emf, &raw mut (*emr).rclBounds);
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, emr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_FrameRgn(
    dc_attr: *mut WINEDC,
    hrgn: HRGN,
    hbrush: HBRUSH,
    width: INT,
    height: INT,
) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: *mut EMRFRAMERGN = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut rgnsize: DWORD = 0;
    let mut index: DWORD = 0;
    let mut ret: BOOL = 0;

    index = emfdc_create_brush(emf, hbrush);
    if index == 0 {
        return (FALSE) as BOOL;
    }

    rgnsize = GetRegionData(hrgn, 0, core::ptr::null_mut());
    size = rgnsize.wrapping_add(core::mem::offset_of!(EMRFRAMERGN, RgnData) as DWORD);
    emr = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut EMRFRAMERGN;

    GetRegionData(
        hrgn,
        rgnsize,
        core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA,
    );

    (*emr).emr.iType = EMR_FRAMERGN as DWORD;
    (*emr).emr.nSize = size;
    (*emr).rclBounds.left =
        (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.left;
    (*emr).rclBounds.top =
        (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.top;
    (*emr).rclBounds.right =
        (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.right
            - 1;
    (*emr).rclBounds.bottom =
        (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.bottom
            - 1;
    (*emr).cbRgnData = rgnsize;
    (*emr).ihBrush = index;
    (*emr).szlStroke.cx = width as LONG;
    (*emr).szlStroke.cy = height as LONG;

    ret = emfdc_record(emf, core::ptr::addr_of_mut!((*emr).emr));
    if ret != 0 {
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    return ret;
}

unsafe extern "C" fn emfdc_paint_invert_region(mut emf: *mut emf, mut hrgn: HRGN, mut iType: DWORD) -> BOOL {
    let mut emr: *mut EMRINVERTRGN = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut rgnsize: DWORD = 0;
    let mut ret: BOOL = 0;
    rgnsize = GetRegionData(hrgn, 0, core::ptr::null_mut());
    size = (rgnsize as u64).wrapping_add(core::mem::offset_of!(EMRINVERTRGN, RgnData) as u64) as DWORD;
    emr = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut EMRINVERTRGN;
    GetRegionData(hrgn, rgnsize, core::ptr::addr_of_mut!((*emr).RgnData) as LPRGNDATA);
    (*emr).emr.iType = iType;
    (*emr).emr.nSize = size;
    (*emr).rclBounds.left = (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.left;
    (*emr).rclBounds.top = (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.top;
    (*emr).rclBounds.right = (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.right - 1 as LONG;
    (*emr).rclBounds.bottom = (*(core::ptr::addr_of_mut!((*emr).RgnData) as *mut RGNDATA)).rdh.rcBound.bottom - 1 as LONG;
    (*emr).cbRgnData = rgnsize;
    ret = emfdc_record(emf, core::ptr::addr_of_mut!((*emr).emr));
    if ret != 0 {
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(GetProcessHeap(), 0, emr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PaintRgn(mut dc_attr: *mut WINEDC, mut hrgn: HRGN) -> BOOL {
    return emfdc_paint_invert_region((*dc_attr).emf as *mut emf, hrgn, EMR_PAINTRGN as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_InvertRgn(dc_attr: *mut WINEDC, hrgn: HRGN) -> BOOL {
    return emfdc_paint_invert_region((*dc_attr).emf as *mut emf, hrgn, EMR_INVERTRGN as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ExtTextOut(dc_attr: *mut WINEDC, x: INT, y: INT, flags: UINT, rect: *const RECT, str: *const WCHAR, count: UINT, dx: *const INT) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut ex_scale: FLOAT = 0.0;
    let mut ey_scale: FLOAT = 0.0;
    let mut emr: *mut EMREXTTEXTOUTW;
    let mut text_height: INT = 0 as INT;
    let mut text_width: INT = 0 as INT;
    let mut tm: TEXTMETRICW = core::mem::zeroed::<TEXTMETRICW>();
    let mut size: DWORD;
    let mut ret: BOOL;

    if count > (INT_MAX) as UINT {
        return (FALSE) as BOOL;
    }

    size = (core::mem::size_of::<EMREXTTEXTOUTW>() as DWORD)
        .wrapping_add(
            ((count.wrapping_add(1 as UINT)) & !(1 as UINT))
                .wrapping_mul(core::mem::size_of::<WCHAR>() as UINT),
        )
        .wrapping_add(count.wrapping_mul(core::mem::size_of::<INT>() as UINT));

    emr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, size as SIZE_T) as *mut EMREXTTEXTOUTW;

    if GetGraphicsMode((*dc_attr).hdc) == (GM_COMPATIBLE as i32) {
        let horzSize: INT = GetDeviceCaps((*dc_attr).hdc, HORZSIZE as INT) as INT;
        let horzRes: INT = GetDeviceCaps((*dc_attr).hdc, HORZRES as INT) as INT;
        let vertSize: INT = GetDeviceCaps((*dc_attr).hdc, VERTSIZE as INT) as INT;
        let vertRes: INT = GetDeviceCaps((*dc_attr).hdc, VERTRES as INT) as INT;
        let mut wndext: SIZE = core::mem::zeroed::<SIZE>();
        let mut vportext: SIZE = core::mem::zeroed::<SIZE>();

        GetViewportExtEx((*dc_attr).hdc, core::ptr::addr_of_mut!(vportext));
        GetWindowExtEx((*dc_attr).hdc, core::ptr::addr_of_mut!(wndext));
        ex_scale = 100.0 * ((horzSize as FLOAT) / (horzRes as FLOAT))
            / ((wndext.cx as FLOAT) / (vportext.cx as FLOAT));
        ey_scale = 100.0 * ((vertSize as FLOAT) / (vertRes as FLOAT))
            / ((wndext.cy as FLOAT) / (vportext.cy as FLOAT));
    } else {
        ex_scale = 0.0;
        ey_scale = 0.0;
    }

    (*emr).emr.iType = EMR_EXTTEXTOUTW as DWORD;
    (*emr).emr.nSize = size;
    (*emr).iGraphicsMode = GetGraphicsMode((*dc_attr).hdc) as DWORD;
    (*emr).exScale = ex_scale;
    (*emr).eyScale = ey_scale;
    (*emr).emrtext.ptlReference.x = x as LONG;
    (*emr).emrtext.ptlReference.y = y as LONG;
    (*emr).emrtext.nChars = count as DWORD;
    (*emr).emrtext.offString = core::mem::size_of::<EMREXTTEXTOUTW>() as DWORD;
    memcpy(
        (emr as *mut core::ffi::c_char).offset((*emr).emrtext.offString as isize) as *mut core::ffi::c_void,
        str as *const core::ffi::c_void,
        (count as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize),
    );
    (*emr).emrtext.fOptions = flags as DWORD;
    if rect.is_null() {
        (*emr).emrtext.rcl.top = 0 as LONG;
        (*emr).emrtext.rcl.left = (*emr).emrtext.rcl.top;
        (*emr).emrtext.rcl.bottom = -(1 as LONG);
        (*emr).emrtext.rcl.right = (*emr).emrtext.rcl.bottom;
    } else {
        (*emr).emrtext.rcl.left = (*rect).left;
        (*emr).emrtext.rcl.top = (*rect).top;
        (*emr).emrtext.rcl.right = (*rect).right;
        (*emr).emrtext.rcl.bottom = (*rect).bottom;
    }
    (*emr).emrtext.offDx = (*emr).emrtext.offString.wrapping_add(
        ((count.wrapping_add(1 as UINT)) & !(1 as UINT))
            .wrapping_mul(core::mem::size_of::<WCHAR>() as UINT) as DWORD,
    );
    if !dx.is_null() {
        let mut i: UINT;
        let mut str_size: SIZE = core::mem::zeroed::<SIZE>();
        memcpy(
            (emr as *mut core::ffi::c_char).offset((*emr).emrtext.offDx as isize) as *mut core::ffi::c_void,
            dx as *const core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<INT>() as usize),
        );
        i = 0 as UINT;
        while i < count {
            text_width += *dx.offset(i as isize);
            i = i.wrapping_add(1);
        }
        if GetTextExtentPoint32W((*dc_attr).hdc, str, count as INT, core::ptr::addr_of_mut!(str_size)) != 0 {
            text_height = str_size.cy as INT;
        }
    } else {
        let mut i: UINT;
        let emf_dx: *mut INT = (emr as *mut core::ffi::c_char).offset((*emr).emrtext.offDx as isize) as *mut INT;
        let mut charSize: SIZE = core::mem::zeroed::<SIZE>();
        i = 0 as UINT;
        while i < count {
            if GetTextExtentPoint32W(
                (*dc_attr).hdc,
                str.offset(i as isize),
                1 as INT,
                core::ptr::addr_of_mut!(charSize),
            ) != 0
            {
                *emf_dx.offset(i as isize) = charSize.cx as INT;
                text_width += charSize.cx;
                text_height = max(text_height, charSize.cy);
            }
            i = i.wrapping_add(1);
        }
    }

    'no_bounds: {
        if (*emf).path != 0 {
            (*emr).rclBounds.top = 0 as LONG;
            (*emr).rclBounds.left = (*emr).rclBounds.top;
            (*emr).rclBounds.bottom = -(1 as LONG);
            (*emr).rclBounds.right = (*emr).rclBounds.bottom;
            break 'no_bounds;
        }

        /* FIXME: handle font escapement */
        match GetTextAlign((*dc_attr).hdc) & (TA_LEFT | TA_RIGHT | TA_CENTER) as UINT {
            TA_CENTER => {
                (*emr).rclBounds.left = (x - (text_width / 2) - 1) as LONG;
                (*emr).rclBounds.right = (x + (text_width / 2) + 1) as LONG;
            }
            TA_RIGHT => {
                (*emr).rclBounds.left = (x - text_width - 1) as LONG;
                (*emr).rclBounds.right = x as LONG;
            }
            _ => {
                /* TA_LEFT */
                (*emr).rclBounds.left = x as LONG;
                (*emr).rclBounds.right = (x + text_width + 1) as LONG;
            }
        }

        match GetTextAlign((*dc_attr).hdc) & (TA_TOP | TA_BOTTOM | TA_BASELINE) as UINT {
            TA_BASELINE => {
                if GetTextMetricsW((*dc_attr).hdc, core::ptr::addr_of_mut!(tm)) == 0 {
                    tm.tmDescent = 0 as LONG;
                }
                /* Play safe here... it's better to have a bounding box */
                /* that is too big than too small. */
                (*emr).rclBounds.top = (y - text_height - 1) as LONG;
                (*emr).rclBounds.bottom = (y + tm.tmDescent + 1) as LONG;
            }
            TA_BOTTOM => {
                (*emr).rclBounds.top = (y - text_height - 1) as LONG;
                (*emr).rclBounds.bottom = y as LONG;
            }
            _ => {
                /* TA_TOP */
                (*emr).rclBounds.top = y as LONG;
                (*emr).rclBounds.bottom = (y + text_height + 1) as LONG;
            }
        }
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }

    ret = emfdc_record(emf, core::ptr::addr_of_mut!((*emr).emr));
    HeapFree(GetProcessHeap(), 0 as DWORD, emr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_GradientFill(
    dc_attr: *mut WINEDC,
    vert_array: *mut TRIVERTEX,
    nvert: ULONG,
    grad_array: *mut ::core::ffi::c_void,
    ngrad: ULONG,
    mode: ULONG,
) -> BOOL {
    let mut emr: *mut EMRGRADIENTFILL = ::core::ptr::null_mut::<EMRGRADIENTFILL>();
    let mut i: ULONG = 0;
    let mut pt: ULONG = 0;
    let mut size: ULONG = 0;
    let mut num_pts: ULONG = ngrad.wrapping_mul(
        if mode == GRADIENT_FILL_TRIANGLE {
            3
        } else {
            2
        },
    );
    let pts: *const ULONG = grad_array as *const ULONG;
    let mut ret: BOOL = 0;

    size = ((FIELD_OFFSET!(EMRGRADIENTFILL, Ver[nvert as usize]) as usize) as ULONG)
        .wrapping_add(num_pts.wrapping_mul(
            core::mem::size_of_val(&*pts.add(0 as usize)) as ULONG,
        ));

    emr = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut EMRGRADIENTFILL;
    if emr.is_null() {
        return (FALSE) as BOOL;
    }

    i = 0;
    while i < num_pts {
        pt = *pts.offset(i as isize);

        if i == 0 {
            (*emr).rclBounds.right = (*vert_array.offset(pt as isize)).x;
            (*emr).rclBounds.left = (*emr).rclBounds.right;
            (*emr).rclBounds.bottom = (*vert_array.offset(pt as isize)).y;
            (*emr).rclBounds.top = (*emr).rclBounds.bottom;
        } else {
            if (*vert_array.offset(pt as isize)).x < (*emr).rclBounds.left {
                (*emr).rclBounds.left = (*vert_array.offset(pt as isize)).x;
            } else if (*vert_array.offset(pt as isize)).x > (*emr).rclBounds.right {
                (*emr).rclBounds.right = (*vert_array.offset(pt as isize)).x;
            }
            if (*vert_array.offset(pt as isize)).y < (*emr).rclBounds.top {
                (*emr).rclBounds.top = (*vert_array.offset(pt as isize)).y;
            } else if (*vert_array.offset(pt as isize)).y > (*emr).rclBounds.bottom {
                (*emr).rclBounds.bottom = (*vert_array.offset(pt as isize)).y;
            }
        }
        i = i.wrapping_add(1);
    }
    (*emr).rclBounds.right -= 1;
    (*emr).rclBounds.bottom -= 1;

    (*emr).emr.iType = EMR_GRADIENTFILL as DWORD;
    (*emr).emr.nSize = size as DWORD;
    (*emr).nVer = nvert as DWORD;
    (*emr).nTri = ngrad as DWORD;
    (*emr).ulMode = mode;
    memcpy(
        &raw mut (*emr).Ver as *mut TRIVERTEX as *mut ::core::ffi::c_void,
        vert_array as *const ::core::ffi::c_void,
        (nvert as usize).wrapping_mul(core::mem::size_of_val(
            &*vert_array.add(0 as usize),
        )),
    );
    memcpy(
        (&raw mut (*emr).Ver as *mut TRIVERTEX).offset(nvert as isize) as *mut ::core::ffi::c_void,
        pts as *const ::core::ffi::c_void,
        (num_pts as usize).wrapping_mul(core::mem::size_of_val(
            &*pts.add(0 as usize),
        )),
    );

    emfdc_update_bounds((*dc_attr).emf as *mut emf, &raw mut (*emr).rclBounds);
    ret = emfdc_record((*dc_attr).emf as *mut emf, &raw mut (*emr).emr);
    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_FillPath(dc_attr: *mut WINEDC) -> BOOL {
    return emfdrv_stroke_and_fill_path((*dc_attr).emf as *mut emf, (EMR_FILLPATH) as INT);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_StrokeAndFillPath(dc_attr: *mut WINEDC) -> BOOL {
    return emfdrv_stroke_and_fill_path((*dc_attr).emf as *mut emf, (EMR_STROKEANDFILLPATH) as INT);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_StrokePath(dc_attr: *mut WINEDC) -> BOOL {
    return emfdrv_stroke_and_fill_path((*dc_attr).emf as *mut emf, (EMR_STROKEPATH) as INT);
}

unsafe extern "C" fn emfdrv_stretchblt(
    emf: *mut emf,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    rop: DWORD,
    type_: DWORD,
) -> BOOL {
    let mut src_info: BITMAPINFO = core::mem::zeroed();
    src_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut bmi_size: UINT = 0;
    let mut emr_size: UINT = 0;
    let mut size: UINT = 0;
    let mut bitmap: HBITMAP = core::ptr::null_mut();
    let mut blit_bitmap: HBITMAP = core::ptr::null_mut();
    let mut emr: *mut EMRBITBLT = core::ptr::null_mut();
    let mut bmi: *mut BITMAPINFO = core::ptr::null_mut();
    let mut blit_dc: HDC = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    if !hdc_src.is_null() && GDI_HANDLE_GET_TYPE!(hdc_src) == GDILoObjType_LO_ALTDC_TYPE {
        let pldc: *mut WINEDC = get_dc_ptr(hdc_src) as *mut WINEDC;

        if (*pldc).iType == (LDC_EMFLDC as i32) {
            return (FALSE) as BOOL;
        }
    }

    bitmap = GetCurrentObject(hdc_src, OBJ_BITMAP) as HBITMAP;
    if bitmap.is_null() {
        return (FALSE) as BOOL;
    }

    blit_dc = hdc_src;
    blit_bitmap = bitmap;
    bmi_size = get_bitmap_info(
        core::ptr::addr_of_mut!(blit_dc),
        core::ptr::addr_of_mut!(blit_bitmap),
        core::ptr::addr_of_mut!(src_info),
    );
    if bmi_size == 0 {
        return (FALSE) as BOOL;
    }

    /* EMRSTRETCHBLT and EMRALPHABLEND have the same structure */
    emr_size = if type_ == EMR_BITBLT {
        core::mem::size_of::<EMRBITBLT>() as UINT
    } else {
        core::mem::size_of::<EMRSTRETCHBLT>() as UINT
    };
    size = emr_size
        .wrapping_add(bmi_size)
        .wrapping_add(src_info.bmiHeader.biSizeImage);

    'err: {
        emr = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut EMRBITBLT;
        if emr.is_null() {
            break 'err;
        }

        (*emr).emr.iType = type_;
        (*emr).emr.nSize = size as DWORD;
        (*emr).rclBounds.left = x_dst as LONG;
        (*emr).rclBounds.top = y_dst as LONG;
        (*emr).rclBounds.right = (x_dst + width_dst - 1) as LONG;
        (*emr).rclBounds.bottom = (y_dst + height_dst - 1) as LONG;
        (*emr).xDest = x_dst as LONG;
        (*emr).yDest = y_dst as LONG;
        (*emr).cxDest = width_dst as LONG;
        (*emr).cyDest = height_dst as LONG;
        (*emr).xSrc = x_src as LONG;
        (*emr).ySrc = y_src as LONG;
        if type_ != EMR_BITBLT {
            let emr_stretchblt: *mut EMRSTRETCHBLT = emr as *mut EMRSTRETCHBLT;
            (*emr_stretchblt).cxSrc = width_src as LONG;
            (*emr_stretchblt).cySrc = height_src as LONG;
        }
        (*emr).dwRop = rop;
        NtGdiGetTransform(
            hdc_src,
            GdiWorldSpaceToDeviceSpace as DWORD,
            core::ptr::addr_of_mut!((*emr).xformSrc),
        );
        (*emr).crBkColorSrc = GetBkColor(hdc_src);
        (*emr).iUsageSrc = DIB_RGB_COLORS;
        (*emr).offBmiSrc = emr_size as DWORD;
        (*emr).cbBmiSrc = bmi_size as DWORD;
        (*emr).offBitsSrc = emr_size.wrapping_add(bmi_size) as DWORD;
        (*emr).cbBitsSrc = src_info.bmiHeader.biSizeImage;

        bmi = (emr as *mut BYTE).offset((*emr).offBmiSrc as isize) as *mut BITMAPINFO;
        (*bmi).bmiHeader = src_info.bmiHeader;
        ret = GetDIBits(
            blit_dc,
            blit_bitmap,
            0,
            src_info.bmiHeader.biHeight as UINT,
            (emr as *mut BYTE).offset((*emr).offBitsSrc as isize) as LPVOID,
            bmi as LPBITMAPINFO,
            DIB_RGB_COLORS,
        ) as BOOL;

        if ret != 0 {
            ret = emfdc_record(emf, emr as *mut EMR);
            if ret != 0 {
                emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
            }
        }
    }

    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    if blit_bitmap != bitmap {
        DeleteObject(blit_bitmap as HGDIOBJ);
    }
    if blit_dc != hdc_src {
        DeleteDC(blit_dc);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_AlphaBlend(
    dc_attr: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    mut blend_function: BLENDFUNCTION,
) -> BOOL {
    return emfdrv_stretchblt(
        (*dc_attr).emf as *mut emf,
        x_dst,
        y_dst,
        width_dst,
        height_dst,
        hdc_src,
        x_src,
        y_src,
        width_src,
        height_src,
        *(&raw mut blend_function as *mut DWORD),
        EMR_ALPHABLEND as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PatBlt(
    mut dc_attr: *mut WINEDC,
    mut left: INT,
    mut top: INT,
    mut width: INT,
    mut height: INT,
    mut rop: DWORD,
) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRBITBLT = core::mem::zeroed();
    let mut ret: BOOL = 0;

    emr.emr.iType = EMR_BITBLT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRBITBLT>() as DWORD;
    emr.rclBounds.left = left as LONG;
    emr.rclBounds.top = top as LONG;
    emr.rclBounds.right = (left + width - 1) as LONG;
    emr.rclBounds.bottom = (top + height - 1) as LONG;
    emr.xDest = left as LONG;
    emr.yDest = top as LONG;
    emr.cxDest = width as LONG;
    emr.cyDest = height as LONG;
    emr.dwRop = rop;
    emr.xSrc = 0 as LONG;
    emr.ySrc = 0 as LONG;
    emr.xformSrc.eM11 = 1.0;
    emr.xformSrc.eM12 = 0.0;
    emr.xformSrc.eM21 = 0.0;
    emr.xformSrc.eM22 = 1.0;
    emr.xformSrc.eDx = 0.0;
    emr.xformSrc.eDy = 0.0;
    emr.crBkColorSrc = 0 as COLORREF;
    emr.iUsageSrc = 0 as DWORD;
    emr.offBmiSrc = 0 as DWORD;
    emr.cbBmiSrc = 0 as DWORD;
    emr.offBitsSrc = 0 as DWORD;
    emr.cbBitsSrc = 0 as DWORD;

    ret = emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
    if ret != 0 {
        emfdc_update_bounds(emf, core::ptr::addr_of_mut!(emr.rclBounds));
    }
    return ret;
}

unsafe extern "C" fn rop_uses_src(rop: DWORD) -> BOOL {
    return (((rop >> 2) & 0x330000) != (rop & 0x330000)) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_BitBlt(
    dc_attr: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width: INT,
    height: INT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    rop: DWORD,
) -> BOOL {
    if rop_uses_src(rop) == 0 {
        return EMFDC_PatBlt(dc_attr, x_dst, y_dst, width, height, rop);
    }
    return emfdrv_stretchblt(
        (*dc_attr).emf as *mut emf,
        x_dst,
        y_dst,
        width,
        height,
        hdc_src,
        x_src,
        y_src,
        width,
        height,
        rop,
        EMR_BITBLT,
    );
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_StretchBlt(
    dc_attr: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    rop: DWORD,
) -> BOOL {
    if rop_uses_src(rop) == 0 {
        return EMFDC_PatBlt(dc_attr, x_dst, y_dst, width_dst, height_dst, rop);
    }
    return emfdrv_stretchblt(
        (*dc_attr).emf as *mut emf,
        x_dst,
        y_dst,
        width_dst,
        height_dst,
        hdc_src,
        x_src,
        y_src,
        width_src,
        height_src,
        rop,
        EMR_STRETCHBLT as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_TransparentBlt(
    dc_attr: *mut WINEDC,
    x_dst: ::core::ffi::c_int,
    y_dst: ::core::ffi::c_int,
    width_dst: ::core::ffi::c_int,
    height_dst: ::core::ffi::c_int,
    hdc_src: HDC,
    x_src: ::core::ffi::c_int,
    y_src: ::core::ffi::c_int,
    width_src: ::core::ffi::c_int,
    height_src: ::core::ffi::c_int,
    color: UINT,
) -> BOOL {
    return emfdrv_stretchblt(
        (*dc_attr).emf as *mut emf,
        x_dst as INT,
        y_dst as INT,
        width_dst as INT,
        height_dst as INT,
        hdc_src,
        x_src as INT,
        y_src as INT,
        width_src as INT,
        height_src as INT,
        color as DWORD,
        EMR_TRANSPARENTBLT as DWORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_MaskBlt(
    dc_attr: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    mask: HBITMAP,
    x_mask: INT,
    y_mask: INT,
    rop: DWORD,
) -> BOOL {
    let mut mask_info_buffer: [u8; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize] =
        [0; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize];
    let mut mask_bits_info: *mut BITMAPINFO = mask_info_buffer.as_mut_ptr() as *mut BITMAPINFO;
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut mask_info: BITMAPINFO = core::mem::zeroed();
    mask_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut src_info: BITMAPINFO = core::mem::zeroed();
    src_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut bitmap: HBITMAP = core::ptr::null_mut();
    let mut blit_bitmap: HBITMAP = core::ptr::null_mut();
    let mut mask_bitmap: HBITMAP = core::ptr::null_mut();
    let mut bmi_size: UINT = 0;
    let mut size: UINT = 0;
    let mut mask_info_size: UINT = 0;
    let mut emr: *mut EMRMASKBLT = core::ptr::null_mut();
    let mut bmi: *mut BITMAPINFO = core::ptr::null_mut();
    let mut blit_dc: HDC = core::ptr::null_mut();
    let mut mask_dc: HDC = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE) as i32;

    if rop_uses_src(rop) == 0 {
        return EMFDC_PatBlt(dc_attr, x_dst, y_dst, width_dst, height_dst, rop);
    }

    if !hdc_src.is_null()
        && GDI_HANDLE_GET_TYPE!(hdc_src) == GDILoObjType_LO_ALTDC_TYPE
    {
        let mut pldc: *mut WINEDC = get_dc_ptr(hdc_src) as *mut WINEDC;

        if (*pldc).iType == (LDC_EMFLDC as i32) {
            return (FALSE) as BOOL;
        }
    }

    bitmap = GetCurrentObject(hdc_src, OBJ_BITMAP as UINT) as HBITMAP;
    if bitmap.is_null() {
        return (FALSE) as BOOL;
    }
    blit_dc = hdc_src;
    blit_bitmap = bitmap;
    bmi_size = get_bitmap_info(&raw mut blit_dc, &raw mut blit_bitmap, &raw mut src_info);
    if bmi_size == 0 {
        return (FALSE) as BOOL;
    }

    'err: {
        if !mask.is_null() {
            mask_dc = hdc_src;
            mask_bitmap = mask;
            mask_info_size = get_bitmap_info(&raw mut mask_dc, &raw mut mask_bitmap, &raw mut mask_info);
            if mask_info_size == 0 {
                break 'err;
            }
            if mask_info.bmiHeader.biBitCount as core::ffi::c_int == 1 {
                mask_info_size = core::mem::size_of::<BITMAPINFOHEADER>() as UINT; /* don't include colors */
            }
        } else {
            mask_info.bmiHeader.biSizeImage = 0;
        }

        size = (core::mem::size_of::<EMRMASKBLT>() as UINT)
            .wrapping_add(bmi_size)
            .wrapping_add(src_info.bmiHeader.biSizeImage)
            .wrapping_add(mask_info_size)
            .wrapping_add(mask_info.bmiHeader.biSizeImage);

        emr = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut EMRMASKBLT;
        if emr.is_null() {
            break 'err;
        }

        (*emr).emr.iType = EMR_MASKBLT as DWORD;
        (*emr).emr.nSize = size as DWORD;
        (*emr).rclBounds.left = x_dst as LONG;
        (*emr).rclBounds.top = y_dst as LONG;
        (*emr).rclBounds.right = (x_dst + width_dst - 1) as LONG;
        (*emr).rclBounds.bottom = (y_dst + height_dst - 1) as LONG;
        (*emr).xDest = x_dst as LONG;
        (*emr).yDest = y_dst as LONG;
        (*emr).cxDest = width_dst as LONG;
        (*emr).cyDest = height_dst as LONG;
        (*emr).dwRop = rop;
        (*emr).xSrc = x_src as LONG;
        (*emr).ySrc = y_src as LONG;
        NtGdiGetTransform(hdc_src, GdiWorldSpaceToDeviceSpace as DWORD, &raw mut (*emr).xformSrc);
        (*emr).crBkColorSrc = GetBkColor(hdc_src);
        (*emr).iUsageSrc = DIB_RGB_COLORS as DWORD;
        (*emr).offBmiSrc = core::mem::size_of::<EMRMASKBLT>() as DWORD;
        (*emr).cbBmiSrc = bmi_size as DWORD;
        (*emr).offBitsSrc = (*emr).offBmiSrc.wrapping_add(bmi_size as DWORD);
        (*emr).cbBitsSrc = src_info.bmiHeader.biSizeImage;
        (*emr).xMask = x_mask as LONG;
        (*emr).yMask = y_mask as LONG;
        (*emr).iUsageMask = DIB_PAL_MONO as DWORD;
        (*emr).offBmiMask = if mask_info_size != 0 {
            (*emr).offBitsSrc.wrapping_add((*emr).cbBitsSrc)
        } else {
            0 as DWORD
        };
        (*emr).cbBmiMask = mask_info_size as DWORD;
        (*emr).offBitsMask = (*emr).offBmiMask.wrapping_add((*emr).cbBmiMask);
        (*emr).cbBitsMask = mask_info.bmiHeader.biSizeImage;

        bmi = (emr as *mut core::ffi::c_char).offset((*emr).offBmiSrc as isize) as *mut BITMAPINFO;
        (*bmi).bmiHeader = src_info.bmiHeader;
        ret = GetDIBits(
            blit_dc,
            blit_bitmap,
            0 as UINT,
            src_info.bmiHeader.biHeight as UINT,
            (emr as *mut core::ffi::c_char).offset((*emr).offBitsSrc as isize) as LPVOID,
            bmi as LPBITMAPINFO,
            DIB_RGB_COLORS as UINT,
        );
        if ret == 0 {
            break 'err;
        }

        if mask_info_size != 0 {
            (*mask_bits_info).bmiHeader = mask_info.bmiHeader;
            ret = GetDIBits(
                blit_dc,
                mask_bitmap,
                0 as UINT,
                mask_info.bmiHeader.biHeight as UINT,
                (emr as *mut core::ffi::c_char).offset((*emr).offBitsMask as isize) as LPVOID,
                mask_bits_info as LPBITMAPINFO,
                DIB_RGB_COLORS as UINT,
            );
            if ret != 0 {
                memcpy(
                    (emr as *mut core::ffi::c_char).offset((*emr).offBmiMask as isize) as *mut core::ffi::c_void,
                    mask_bits_info as *const core::ffi::c_void,
                    mask_info_size as size_t,
                );
            }
        }

        if ret != 0 {
            ret = emfdc_record(emf, emr as *mut EMR);
            if ret != 0 {
                emfdc_update_bounds(emf, &raw mut (*emr).rclBounds);
            }
        }
    }

    HeapFree(GetProcessHeap(), 0 as DWORD, emr as PVOID);
    if mask_bitmap != mask {
        DeleteObject(mask_bitmap as HGDIOBJ);
    }
    if mask_dc != hdc_src {
        DeleteObject(mask_dc as HGDIOBJ);
    }
    if blit_bitmap != bitmap {
        DeleteObject(blit_bitmap as HGDIOBJ);
    }
    if blit_dc != hdc_src {
        DeleteDC(blit_dc);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_PlgBlt(
    dc_attr: *mut WINEDC,
    points: *const POINT,
    hdc_src: HDC,
    x_src: INT,
    y_src: INT,
    width: INT,
    height: INT,
    mask: HBITMAP,
    x_mask: INT,
    y_mask: INT,
) -> BOOL {
    let mut mask_info_buffer = [0 as ::core::ffi::c_uchar; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize];
    let mut mask_bits_info: *mut BITMAPINFO = mask_info_buffer.as_mut_ptr() as *mut BITMAPINFO;
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut mask_info: BITMAPINFO = core::mem::zeroed();
    mask_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut src_info: BITMAPINFO = core::mem::zeroed();
    src_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut bitmap: HBITMAP = core::ptr::null_mut();
    let mut blit_bitmap: HBITMAP = core::ptr::null_mut();
    let mut mask_bitmap: HBITMAP = core::ptr::null_mut();
    let mut bmi_size: UINT = 0;
    let mut size: UINT = 0;
    let mut mask_info_size: UINT = 0;
    let mut emr: *mut EMRPLGBLT = core::ptr::null_mut();
    let mut bmi: *mut BITMAPINFO = core::ptr::null_mut();
    let mut blit_dc: HDC = core::ptr::null_mut();
    let mut mask_dc: HDC = core::ptr::null_mut();
    let mut x_min: ::core::ffi::c_int = 0;
    let mut y_min: ::core::ffi::c_int = 0;
    let mut x_max: ::core::ffi::c_int = 0;
    let mut y_max: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    let mut ret: BOOL = (FALSE) as i32;

    if !hdc_src.is_null() && GDI_HANDLE_GET_TYPE!(hdc_src) == GDILoObjType_LO_ALTDC_TYPE {
        let mut pldc: *mut WINEDC = get_dc_ptr(hdc_src) as *mut WINEDC;

        if (*pldc).iType == (LDC_EMFLDC as i32) {
            return (FALSE) as BOOL;
        }
    }

    bitmap = GetCurrentObject(hdc_src, OBJ_BITMAP) as HBITMAP;
    if bitmap.is_null() {
        return (FALSE) as BOOL;
    }

    blit_dc = hdc_src;
    blit_bitmap = bitmap;
    bmi_size = get_bitmap_info(core::ptr::addr_of_mut!(blit_dc), core::ptr::addr_of_mut!(blit_bitmap), core::ptr::addr_of_mut!(src_info));
    if bmi_size == 0 {
        return (FALSE) as BOOL;
    }

    'err: {
        if !mask.is_null() {
            mask_dc = hdc_src;
            mask_bitmap = mask;
            mask_info_size = get_bitmap_info(core::ptr::addr_of_mut!(mask_dc), core::ptr::addr_of_mut!(mask_bitmap), core::ptr::addr_of_mut!(mask_info));
            if mask_info_size == 0 {
                break 'err;
            }
            if mask_info.bmiHeader.biBitCount == 1 {
                mask_info_size = core::mem::size_of::<BITMAPINFOHEADER>() as UINT; /* don't include colors */
            }
        } else {
            mask_info.bmiHeader.biSizeImage = 0;
        }

        size = (core::mem::size_of::<EMRPLGBLT>() as UINT)
            .wrapping_add(bmi_size)
            .wrapping_add(src_info.bmiHeader.biSizeImage)
            .wrapping_add(mask_info_size)
            .wrapping_add(mask_info.bmiHeader.biSizeImage);

        emr = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut EMRPLGBLT;
        if emr.is_null() {
            break 'err;
        }

        (*emr).emr.iType = EMR_PLGBLT;
        (*emr).emr.nSize = size as DWORD;

        /* FIXME: not exactly what native does */
        x_max = ((*points.offset(1 as isize)).x
            + (*points.offset(2 as isize)).x
            - (*points.offset(0 as isize)).x) as ::core::ffi::c_int;
        x_min = x_max;
        y_max = ((*points.offset(1 as isize)).y
            + (*points.offset(2 as isize)).y
            - (*points.offset(0 as isize)).y) as ::core::ffi::c_int;
        y_min = y_max;
        i = 0 as ::core::ffi::c_int;
        while i < (ARRAYSIZE!((*emr).aptlDest) as ::core::ffi::c_int) {
            x_min = min(x_min, (*points.offset(i as isize)).x as ::core::ffi::c_int);
            y_min = min(y_min, (*points.offset(i as isize)).y as ::core::ffi::c_int);
            x_max = max(x_max, (*points.offset(i as isize)).x as ::core::ffi::c_int);
            y_max = max(y_min, (*points.offset(i as isize)).y as ::core::ffi::c_int);
            i += 1;
        }
        (*emr).rclBounds.left = x_min as LONG;
        (*emr).rclBounds.top = y_min as LONG;
        (*emr).rclBounds.right = x_max as LONG;
        (*emr).rclBounds.bottom = y_max as LONG;
        memcpy(
            core::ptr::addr_of_mut!((*emr).aptlDest) as *mut ::core::ffi::c_void,
            points as *const ::core::ffi::c_void,
            core::mem::size_of::<[POINTL; 3]>() as size_t,
        );
        (*emr).xSrc = x_src as LONG;
        (*emr).ySrc = y_src as LONG;
        (*emr).cxSrc = width as LONG;
        (*emr).cySrc = height as LONG;
        NtGdiGetTransform(hdc_src, GdiWorldSpaceToDeviceSpace as DWORD, core::ptr::addr_of_mut!((*emr).xformSrc));
        (*emr).crBkColorSrc = GetBkColor(hdc_src);
        (*emr).iUsageSrc = DIB_RGB_COLORS;
        (*emr).offBmiSrc = core::mem::size_of::<EMRPLGBLT>() as DWORD;
        (*emr).cbBmiSrc = bmi_size as DWORD;
        (*emr).offBitsSrc = (*emr).offBmiSrc.wrapping_add(bmi_size as DWORD);
        (*emr).cbBitsSrc = src_info.bmiHeader.biSizeImage;
        (*emr).xMask = x_mask as LONG;
        (*emr).yMask = y_mask as LONG;
        (*emr).iUsageMask = DIB_PAL_MONO;
        (*emr).offBmiMask = if mask_info_size != 0 {
            (*emr).offBitsSrc.wrapping_add((*emr).cbBitsSrc)
        } else {
            0
        };
        (*emr).cbBmiMask = mask_info_size as DWORD;
        (*emr).offBitsMask = (*emr).offBmiMask.wrapping_add((*emr).cbBmiMask);
        (*emr).cbBitsMask = mask_info.bmiHeader.biSizeImage;

        bmi = (emr as *mut ::core::ffi::c_char).offset((*emr).offBmiSrc as isize) as *mut BITMAPINFO;
        (*bmi).bmiHeader = src_info.bmiHeader;
        ret = GetDIBits(
            blit_dc,
            blit_bitmap,
            0,
            src_info.bmiHeader.biHeight as UINT,
            (emr as *mut ::core::ffi::c_char).offset((*emr).offBitsSrc as isize) as LPVOID,
            bmi as LPBITMAPINFO,
            DIB_RGB_COLORS,
        ) as BOOL;
        if ret == 0 {
            break 'err;
        }

        if mask_info_size != 0 {
            (*mask_bits_info).bmiHeader = mask_info.bmiHeader;
            ret = GetDIBits(
                blit_dc,
                mask_bitmap,
                0,
                mask_info.bmiHeader.biHeight as UINT,
                (emr as *mut ::core::ffi::c_char).offset((*emr).offBitsMask as isize) as LPVOID,
                mask_bits_info as LPBITMAPINFO,
                DIB_RGB_COLORS,
            ) as BOOL;
            if ret != 0 {
                memcpy(
                    (emr as *mut ::core::ffi::c_char).offset((*emr).offBmiMask as isize) as *mut ::core::ffi::c_void,
                    mask_bits_info as *const ::core::ffi::c_void,
                    mask_info_size as size_t,
                );
            }
        }

        if ret != 0 {
            ret = emfdc_record(emf, emr as *mut EMR);
            if ret != 0 {
                emfdc_update_bounds(emf, core::ptr::addr_of_mut!((*emr).rclBounds));
            }
        }
    }

    HeapFree(GetProcessHeap(), 0, emr as PVOID);
    if mask_bitmap != mask {
        DeleteObject(mask_bitmap as HGDIOBJ);
    }
    if mask_dc != hdc_src {
        DeleteObject((mask_dc).cast());
    }
    if blit_bitmap != bitmap {
        DeleteObject(blit_bitmap as HGDIOBJ);
    }
    if blit_dc != hdc_src {
        DeleteDC(blit_dc);
    }
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_StretchDIBits(
    dc_attr: *mut WINEDC,
    x_dst: INT,
    y_dst: INT,
    width_dst: INT,
    height_dst: INT,
    x_src: INT,
    y_src: INT,
    width_src: INT,
    height_src: INT,
    bits: *const core::ffi::c_void,
    info: *const BITMAPINFO,
    usage: UINT,
    rop: DWORD,
) -> BOOL {
    let mut emr: *mut EMRSTRETCHDIBITS = core::ptr::null_mut();
    let mut ret: BOOL = 0;
    let mut bmi_size: UINT = 0;
    let mut emr_size: UINT = 0;

    /* calculate the size of the colour table */
    bmi_size = get_dib_info_size(info, usage) as UINT;

    emr_size = (core::mem::size_of::<EMRSTRETCHDIBITS>() as UINT)
        .wrapping_add(bmi_size)
        .wrapping_add((*info).bmiHeader.biSizeImage as UINT);
    emr = HeapAlloc(
        GetProcessHeap(),
        0,
        emr_size as SIZE_T,
    ) as *mut EMRSTRETCHDIBITS;
    if emr.is_null() {
        return 0;
    }

    /* write a bitmap info header (with colours) to the record */
    memcpy(
        emr.offset(1 as isize) as *mut core::ffi::c_void,
        info as *const core::ffi::c_void,
        bmi_size as usize,
    );

    /* write bitmap bits to the record */
    memcpy(
        (emr.offset(1 as isize) as *mut BYTE).offset(bmi_size as isize)
            as *mut core::ffi::c_void,
        bits,
        (*info).bmiHeader.biSizeImage as usize,
    );

    /* fill in the EMR header at the front of our piece of memory */
    (*emr).emr.iType = EMR_STRETCHDIBITS as DWORD;
    (*emr).emr.nSize = emr_size as DWORD;

    (*emr).xDest = x_dst as LONG;
    (*emr).yDest = y_dst as LONG;
    (*emr).cxDest = width_dst as LONG;
    (*emr).cyDest = height_dst as LONG;
    (*emr).dwRop = rop;
    (*emr).xSrc = x_src as LONG;
    (*emr).ySrc = y_src as LONG;

    (*emr).iUsageSrc = usage as DWORD;
    (*emr).offBmiSrc = core::mem::size_of::<EMRSTRETCHDIBITS>() as DWORD;
    (*emr).cbBmiSrc = bmi_size as DWORD;
    (*emr).offBitsSrc = (*emr).offBmiSrc.wrapping_add(bmi_size as DWORD);
    (*emr).cbBitsSrc = (*info).bmiHeader.biSizeImage;

    (*emr).cxSrc = width_src as LONG;
    (*emr).cySrc = height_src as LONG;

    (*emr).rclBounds.left = x_dst as LONG;
    (*emr).rclBounds.top = y_dst as LONG;
    (*emr).rclBounds.right = (x_dst + width_dst) as LONG;
    (*emr).rclBounds.bottom = (y_dst + height_dst) as LONG;

    /* save the record we just created */
    ret = emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!((*emr).emr));
    if ret != 0 {
        emfdc_update_bounds((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetDIBitsToDevice(
    mut dc_attr: *mut WINEDC,
    mut x_dst: INT,
    mut y_dst: INT,
    mut width: DWORD,
    mut height: DWORD,
    mut x_src: INT,
    mut y_src: INT,
    mut startscan: UINT,
    mut lines: UINT,
    mut bits: *const ::core::ffi::c_void,
    mut info: *const BITMAPINFO,
    mut usage: UINT,
) -> BOOL {
    let mut emr: *mut EMRSETDIBITSTODEVICE = ::core::ptr::null_mut::<EMRSETDIBITSTODEVICE>();
    let mut bmiSize: DWORD = get_dib_info_size(info, usage) as DWORD;
    let mut size: DWORD = (::core::mem::size_of::<EMRSETDIBITSTODEVICE>() as usize)
        .wrapping_add(bmiSize as usize)
        .wrapping_add((*info).bmiHeader.biSizeImage as usize) as DWORD;
    let mut ret: BOOL = 0;
    emr = HeapAlloc(GetProcessHeap(), 0 as DWORD, size as SIZE_T) as *mut EMRSETDIBITSTODEVICE;
    if emr.is_null() {
        return (FALSE) as BOOL;
    }
    (*emr).emr.iType = EMR_SETDIBITSTODEVICE as DWORD;
    (*emr).emr.nSize = size;
    (*emr).rclBounds.left = x_dst as LONG;
    (*emr).rclBounds.top = y_dst as LONG;
    (*emr).rclBounds.right = (x_dst as DWORD)
        .wrapping_add(width)
        .wrapping_sub(1 as DWORD) as LONG;
    (*emr).rclBounds.bottom = (y_dst as DWORD)
        .wrapping_add(height)
        .wrapping_sub(1 as DWORD) as LONG;
    (*emr).xDest = x_dst as LONG;
    (*emr).yDest = y_dst as LONG;
    (*emr).xSrc = x_src as LONG;
    (*emr).ySrc = y_src as LONG;
    (*emr).cxSrc = width as LONG;
    (*emr).cySrc = height as LONG;
    (*emr).offBmiSrc = ::core::mem::size_of::<EMRSETDIBITSTODEVICE>() as DWORD;
    (*emr).cbBmiSrc = bmiSize;
    (*emr).offBitsSrc = (::core::mem::size_of::<EMRSETDIBITSTODEVICE>() as usize)
        .wrapping_add(bmiSize as usize) as DWORD;
    (*emr).cbBitsSrc = (*info).bmiHeader.biSizeImage;
    (*emr).iUsageSrc = usage as DWORD;
    (*emr).iStartScan = startscan as DWORD;
    (*emr).cScans = lines as DWORD;
    memcpy(
        (emr as *mut BYTE).offset((*emr).offBmiSrc as isize) as *mut ::core::ffi::c_void,
        info as *const ::core::ffi::c_void,
        bmiSize as usize,
    );
    memcpy(
        (emr as *mut BYTE).offset((*emr).offBitsSrc as isize) as *mut ::core::ffi::c_void,
        bits,
        (*info).bmiHeader.biSizeImage as usize,
    );
    ret = emfdc_record((*dc_attr).emf as *mut emf, emr as *mut EMR);
    if ret != 0 {
        emfdc_update_bounds((*dc_attr).emf as *mut emf, ::core::ptr::addr_of_mut!((*emr).rclBounds));
    }
    HeapFree(GetProcessHeap(), 0 as DWORD, emr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetDCBrushColor(dc_attr: *mut WINEDC, color: COLORREF) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTOBJECT = core::mem::zeroed();
    let mut index: DWORD = 0;

    if GetCurrentObject((*dc_attr).hdc, OBJ_BRUSH) != GetStockObject((DC_BRUSH as i32)) {
        return (TRUE) as BOOL;
    }

    if !(*emf).dc_brush.is_null() {
        DeleteObject((*emf).dc_brush as HGDIOBJ);
    }
    (*emf).dc_brush = CreateSolidBrush(color);
    if (*emf).dc_brush.is_null() {
        return (FALSE) as BOOL;
    }
    index = emfdc_create_brush(emf, (*emf).dc_brush);
    if index == 0 {
        return (FALSE) as BOOL;
    }
    GDI_hdc_using_object((*emf).dc_brush as HGDIOBJ, (*dc_attr).hdc);
    emr.emr.iType = EMR_SELECTOBJECT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSELECTOBJECT>() as DWORD;
    emr.ihObject = index;
    return emfdc_record(emf, &raw mut emr.emr);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetDCPenColor(dc_attr: *mut WINEDC, color: COLORREF) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRSELECTOBJECT = tagEMRSELECTOBJECT {
        emr: tagEMR { iType: 0, nSize: 0 },
        ihObject: 0,
    };
    let mut index: DWORD = 0;
    let mut logpen: LOGPEN = tagLOGPEN {
        lopnStyle: PS_SOLID as UINT,
        lopnWidth: tagPOINT {
            x: 0 as LONG,
            y: 0 as LONG,
        },
        lopnColor: color,
    };
    if GetCurrentObject((*dc_attr).hdc, OBJ_PEN as UINT)
        != GetStockObject((DC_PEN as i32))
    {
        return (TRUE) as BOOL;
    }
    if !(*emf).dc_pen.is_null() {
        DeleteObject((*emf).dc_pen as HGDIOBJ);
    }
    (*emf).dc_pen = CreatePenIndirect(core::ptr::addr_of_mut!(logpen));
    if (*emf).dc_pen.is_null() {
        return (FALSE) as BOOL;
    }
    index = emfdc_create_pen(emf, (*emf).dc_pen);
    if index == 0 {
        return (FALSE) as BOOL;
    }
    GDI_hdc_using_object((*emf).dc_pen as HGDIOBJ, (*dc_attr).hdc);
    emr.emr.iType = EMR_SELECTOBJECT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSELECTOBJECT>() as DWORD;
    emr.ihObject = index;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SaveDC(dc_attr: *mut WINEDC) -> BOOL {
    let mut emr: EMRSAVEDC = core::mem::zeroed();
    emr.emr.iType = EMR_SAVEDC as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSAVEDC>() as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_RestoreDC(dc_attr: *mut WINEDC, level: INT) -> BOOL {
    let mut emr: EMRRESTOREDC = core::mem::zeroed();

    (*dc_attr).save_level = GetDCDWord((*dc_attr).hdc, GdiGetEMFRestorDc as UINT, 0 as DWORD)
        .wrapping_sub(1 as DWORD) as INT;

    if abs(level as ::core::ffi::c_int) > (*dc_attr).save_level || level == 0 {
        return (FALSE) as BOOL;
    }

    emr.emr.iType = EMR_RESTOREDC as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRRESTOREDC>() as DWORD;
    if level < 0 {
        emr.iRelative = level as LONG;
    } else {
        emr.iRelative = (level
            - (*dc_attr).save_level
            - 1) as LONG;
    }
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetTextAlign(dc_attr: *mut WINEDC, align: UINT) -> BOOL {
    let mut emr: EMRSETTEXTALIGN = core::mem::zeroed();
    emr.emr.iType = EMR_SETTEXTALIGN as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETTEXTALIGN>() as DWORD;
    emr.iMode = align as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetTextJustification(dc_attr: *mut WINEDC, extra: INT, breaks: INT) -> BOOL {
    let mut emr: EMRSETTEXTJUSTIFICATION = core::mem::zeroed();
    emr.emr.iType = EMR_SETTEXTJUSTIFICATION as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETTEXTJUSTIFICATION>() as DWORD;
    emr.nBreakExtra = extra;
    emr.nBreakCount = breaks;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetBkMode(dc_attr: *mut WINEDC, mode: INT) -> BOOL {
    let mut emr: EMRSETBKMODE = core::mem::zeroed();
    emr.emr.iType = EMR_SETBKMODE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETBKMODE>() as DWORD;
    emr.iMode = mode as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetBkColor(dc_attr: *mut WINEDC, color: COLORREF) -> BOOL {
    let mut emr: EMRSETBKCOLOR = core::mem::zeroed();

    emr.emr.iType = EMR_SETBKCOLOR;
    emr.emr.nSize = core::mem::size_of::<EMRSETBKCOLOR>() as DWORD;
    emr.crColor = color;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetTextColor(dc_attr: *mut WINEDC, color: COLORREF) -> BOOL {
    let mut emr: EMRSETTEXTCOLOR = core::mem::zeroed();
    emr.emr.iType = EMR_SETTEXTCOLOR;
    emr.emr.nSize = core::mem::size_of::<EMRSETTEXTCOLOR>() as DWORD;
    emr.crColor = color;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetROP2(dc_attr: *mut WINEDC, rop: INT) -> BOOL {
    let mut emr: EMRSETROP2 = core::mem::zeroed();

    emr.emr.iType = EMR_SETROP2 as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETROP2>() as DWORD;
    emr.iMode = rop as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetPolyFillMode(dc_attr: *mut WINEDC, mode: INT) -> BOOL {
    let mut emr: EMRSETPOLYFILLMODE = core::mem::zeroed();

    emr.emr.iType = EMR_SETPOLYFILLMODE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETPOLYFILLMODE>() as DWORD;
    emr.iMode = (mode) as u32;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetStretchBltMode(dc_attr: *mut WINEDC, mode: INT) -> BOOL {
    let mut emr: EMRSETSTRETCHBLTMODE = core::mem::zeroed();
    emr.emr.iType = EMR_SETSTRETCHBLTMODE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETSTRETCHBLTMODE>() as DWORD;
    emr.iMode = mode as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetArcDirection(dc_attr: *mut WINEDC, dir: INT) -> BOOL {
    let mut emr: EMRSETARCDIRECTION = tagEMRSETARCDIRECTION {
        emr: tagEMR { iType: 0, nSize: 0 },
        iArcDirection: 0,
    };
    emr.emr.iType = EMR_SETARCDIRECTION as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETARCDIRECTION>() as DWORD;
    emr.iArcDirection = dir as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ExcludeClipRect(
    dc_attr: *mut WINEDC,
    left: INT,
    top: INT,
    right: INT,
    bottom: INT,
) -> BOOL {
    let mut emr: EMREXCLUDECLIPRECT = core::mem::zeroed();
    emr.emr.iType = EMR_EXCLUDECLIPRECT as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMREXCLUDECLIPRECT>() as DWORD;
    emr.rclClip.left = left as LONG;
    emr.rclClip.top = top as LONG;
    emr.rclClip.right = right as LONG;
    emr.rclClip.bottom = bottom as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_IntersectClipRect(
    mut dc_attr: *mut WINEDC,
    mut left: INT,
    mut top: INT,
    mut right: INT,
    mut bottom: INT,
) -> BOOL {
    let mut emr: EMRINTERSECTCLIPRECT = tagEMREXCLUDECLIPRECT {
        emr: tagEMR { iType: 0, nSize: 0 },
        rclClip: _RECTL {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
    };
    emr.emr.iType = EMR_INTERSECTCLIPRECT as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRINTERSECTCLIPRECT>() as DWORD;
    emr.rclClip.left = left as LONG;
    emr.rclClip.top = top as LONG;
    emr.rclClip.right = right as LONG;
    emr.rclClip.bottom = bottom as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_OffsetClipRgn(dc_attr: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let mut emr: EMROFFSETCLIPRGN = core::mem::zeroed();

    emr.emr.iType = EMR_OFFSETCLIPRGN as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMROFFSETCLIPRGN>() as DWORD;
    emr.ptlOffset.x = x as LONG;
    emr.ptlOffset.y = y as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ExtSelectClipRgn(
    dc_attr: *mut WINEDC,
    hrgn: HRGN,
    mode: INT,
) -> BOOL {
    let mut emr: *mut EMREXTSELECTCLIPRGN = core::ptr::null_mut();
    let mut size: DWORD = 0;
    let mut rgnsize: DWORD = 0;
    let mut ret: BOOL;
    if hrgn.is_null() {
        if mode != (RGN_COPY as i32) {
            return (ERROR) as BOOL;
        }
        rgnsize = 0;
    } else {
        rgnsize = GetRegionData(hrgn, 0, core::ptr::null_mut());
    }
    size = rgnsize + core::mem::offset_of!(EMREXTSELECTCLIPRGN, RgnData) as DWORD;
    emr = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as *mut EMREXTSELECTCLIPRGN;
    if rgnsize != 0 {
        GetRegionData(
            hrgn,
            rgnsize,
            core::ptr::addr_of_mut!((*emr).RgnData) as LPRGNDATA,
        );
    }
    (*emr).emr.iType = EMR_EXTSELECTCLIPRGN;
    (*emr).emr.nSize = size;
    (*emr).cbRgnData = rgnsize;
    (*emr).iMode = mode as DWORD;
    ret = emfdc_record(
        (*dc_attr).emf as *mut emf,
        core::ptr::addr_of_mut!((*emr).emr),
    );
    HeapFree(
        GetProcessHeap(),
        0,
        emr as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetMapMode(dc_attr: *mut WINEDC, mode: INT) -> BOOL {
    let mut emr: EMRSETMAPMODE = core::mem::zeroed();
    emr.emr.iType = EMR_SETMAPMODE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETMAPMODE>() as DWORD;
    emr.iMode = mode as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetViewportExtEx(
    mut dc_attr: *mut WINEDC,
    mut cx: INT,
    mut cy: INT,
) -> BOOL {
    let mut emr: EMRSETVIEWPORTEXTEX = tagEMRSETVIEWPORTEXTEX {
        emr: tagEMR { iType: 0, nSize: 0 },
        szlExtent: tagSIZE { cx: 0, cy: 0 },
    };
    emr.emr.iType = EMR_SETVIEWPORTEXTEX as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRSETVIEWPORTEXTEX>() as DWORD;
    emr.szlExtent.cx = cx as LONG;
    emr.szlExtent.cy = cy as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, &raw mut emr.emr);
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetWindowExtEx(dc_attr: *mut WINEDC, cx: INT, cy: INT) -> BOOL {
    let mut emr: EMRSETWINDOWEXTEX = core::mem::zeroed();

    emr.emr.iType = EMR_SETWINDOWEXTEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETWINDOWEXTEX>() as DWORD;
    emr.szlExtent.cx = cx as LONG;
    emr.szlExtent.cy = cy as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetViewportOrgEx(dc_attr: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let mut emr: EMRSETVIEWPORTORGEX = core::mem::zeroed();
    emr.emr.iType = EMR_SETVIEWPORTORGEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETVIEWPORTORGEX>() as DWORD;
    emr.ptlOrigin.x = x as LONG;
    emr.ptlOrigin.y = y as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetWindowOrgEx(
    dc_attr: *mut WINEDC,
    x: INT,
    y: INT,
) -> BOOL {
    let mut emr: EMRSETWINDOWORGEX = core::mem::zeroed();
    emr.emr.iType = EMR_SETWINDOWORGEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETWINDOWORGEX>() as DWORD;
    emr.ptlOrigin.x = x as LONG;
    emr.ptlOrigin.y = y as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ScaleViewportExtEx(
    dc_attr: *mut WINEDC,
    x_num: INT,
    x_denom: INT,
    y_num: INT,
    y_denom: INT,
) -> BOOL {
    let mut emr: EMRSCALEVIEWPORTEXTEX = core::mem::zeroed();

    emr.emr.iType = EMR_SCALEVIEWPORTEXTEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSCALEVIEWPORTEXTEX>() as DWORD;
    emr.xNum = x_num as LONG;
    emr.xDenom = x_denom as LONG;
    emr.yNum = y_num as LONG;
    emr.yDenom = y_denom as LONG;
    return emfdc_record(
        (*dc_attr).emf as *mut emf,
        core::ptr::addr_of_mut!(emr.emr) as *mut EMR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ScaleWindowExtEx(
    mut dc_attr: *mut WINEDC,
    mut x_num: INT,
    mut x_denom: INT,
    mut y_num: INT,
    mut y_denom: INT,
) -> BOOL {
    let mut emr: EMRSCALEWINDOWEXTEX = tagEMRSCALEVIEWPORTEXTEX {
        emr: tagEMR { iType: 0, nSize: 0 },
        xNum: 0,
        xDenom: 0,
        yNum: 0,
        yDenom: 0,
    };
    emr.emr.iType = EMR_SCALEWINDOWEXTEX as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRSCALEWINDOWEXTEX>() as DWORD;
    emr.xNum = x_num as LONG;
    emr.xDenom = x_denom as LONG;
    emr.yNum = y_num as LONG;
    emr.yDenom = y_denom as LONG;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetLayout(dc_attr: *mut WINEDC, layout: DWORD) -> BOOL {
    let mut emr: EMRSETLAYOUT = core::mem::zeroed();
    emr.emr.iType = EMR_SETLAYOUT;
    emr.emr.nSize = core::mem::size_of::<EMRSETLAYOUT>() as DWORD;
    emr.iMode = layout;
    return emfdc_record(((*dc_attr).emf).cast(), core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetWorldTransform(dc_attr: *mut WINEDC, xform: *const XFORM) -> BOOL {
    let mut emr: EMRSETWORLDTRANSFORM = core::mem::zeroed();
    emr.emr.iType = EMR_SETWORLDTRANSFORM as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETWORLDTRANSFORM>() as DWORD;
    emr.xform = *xform;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_ModifyWorldTransform(
    dc_attr: *mut WINEDC,
    xform: *const XFORM,
    mode: DWORD,
) -> BOOL {
    let mut emr: EMRMODIFYWORLDTRANSFORM = core::mem::zeroed();

    emr.emr.iType = EMR_MODIFYWORLDTRANSFORM as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRMODIFYWORLDTRANSFORM>() as DWORD;
    if mode == MWT_IDENTITY as DWORD {
        emr.xform.eM11 = 1 as f32;
        emr.xform.eM12 = 0 as f32;
        emr.xform.eM21 = 0 as f32;
        emr.xform.eM22 = 1 as f32;
        emr.xform.eDx = 0 as f32;
        emr.xform.eDy = 0 as f32;
    } else {
        emr.xform = *xform;
    }
    emr.iMode = mode;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetMapperFlags(mut dc_attr: *mut WINEDC, mut flags: DWORD) -> BOOL {
    let mut emr: EMRSETMAPPERFLAGS = EMRSETMAPPERFLAGS {
        emr: tagEMR { iType: 0, nSize: 0 },
        dwFlags: 0,
    };
    emr.emr.iType = EMR_SETMAPPERFLAGS as DWORD;
    emr.emr.nSize = ::core::mem::size_of::<EMRSETMAPPERFLAGS>() as DWORD;
    emr.dwFlags = flags;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_AbortPath(dc_attr: *mut WINEDC) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRABORTPATH = core::mem::zeroed();

    emr.emr.iType = EMR_ABORTPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRABORTPATH>() as DWORD;

    (*emf).path = FALSE as BOOL;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_BeginPath(dc_attr: *mut WINEDC) -> BOOL {
    let emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRBEGINPATH = core::mem::zeroed();

    emr.emr.iType = EMR_BEGINPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRBEGINPATH>() as DWORD;
    if emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr)) == 0 {
        return (FALSE) as BOOL;
    }

    (*emf).path = (TRUE) as i32;
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_CloseFigure(dc_attr: *mut WINEDC) -> BOOL {
    let mut emr: EMRCLOSEFIGURE = core::mem::zeroed();

    emr.emr.iType = EMR_CLOSEFIGURE as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRCLOSEFIGURE>() as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_EndPath(mut dc_attr: *mut WINEDC) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut emr: EMRENDPATH = core::mem::zeroed();
    (*emf).path = FALSE as BOOL;
    emr.emr.iType = EMR_ENDPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRENDPATH>() as DWORD;
    return emfdc_record(emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_FlattenPath(dc_attr: *mut WINEDC) -> BOOL {
    let mut emr: EMRFLATTENPATH = core::mem::zeroed();
    emr.emr.iType = EMR_FLATTENPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRFLATTENPATH>() as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SelectClipPath(dc_attr: *mut WINEDC, mode: INT) -> BOOL {
    let mut emr: EMRSELECTCLIPPATH = core::mem::zeroed();

    emr.emr.iType = EMR_SELECTCLIPPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSELECTCLIPPATH>() as DWORD;
    emr.iMode = mode as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_WidenPath(dc_attr: *mut WINEDC) -> BOOL {
    let mut emr: EMRWIDENPATH = core::mem::zeroed();
    emr.emr.iType = EMR_WIDENPATH as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRWIDENPATH>() as DWORD;
    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_DeleteDC(mut dc_attr: *mut WINEDC) -> BOOL {
    let mut emf: *mut emf = (*dc_attr).emf as *mut emf;
    let mut index: UINT = 0;
    HeapFree(GetProcessHeap(), 0, (*emf).emh as PVOID);
    index = 0;
    while index < (*emf).handles_size {
        if !(*(*emf).handles.offset(index as isize)).is_null() {
            GDI_hdc_not_using_object(
                *(*emf).handles.offset(index as isize),
                (*(*emf).dc_attr).hdc,
            );
        }
        index = index.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0, (*emf).handles as PVOID);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn EMFDC_WriteEscape(dc_attr: *mut WINEDC, nEscape: INT, cbInput: INT, lpszInData: LPSTR, emrType: DWORD) -> BOOL {
    let mut pemr: PEMRESCAPE;
    let mut total: UINT;
    let mut rounded_size: UINT;
    let mut ret: BOOL;

    rounded_size = ((cbInput as UINT) + 3) & !3;
    total = core::mem::offset_of!(EMRESCAPE, Data) as UINT + rounded_size;

    pemr = RtlAllocateHeap(GetProcessHeap(), 0, total as SIZE_T) as PEMRESCAPE;
    if pemr.is_null() {
        return 0;
    }

    RtlZeroMemory!(pemr as *mut core::ffi::c_void, total as usize);

    (*pemr).emr.iType = emrType;
    (*pemr).emr.nSize = total;
    (*pemr).iEsc = nEscape;
    (*pemr).cjIn = cbInput;

    RtlCopyMemory!(
        core::ptr::addr_of_mut!((*pemr).Data[0]) as *mut core::ffi::c_void,
        lpszInData as *const core::ffi::c_void,
        cbInput as usize,
    );

    ret = emfdc_record(((*dc_attr).emf).cast(), core::ptr::addr_of_mut!((*pemr).emr));

    RtlFreeHeap(GetProcessHeap(), 0, pemr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_WriteNamedEscape(
    dc_attr: *mut WINEDC,
    pDriver: PWCHAR,
    nEscape: INT,
    cbInput: INT,
    lpszInData: LPCSTR,
) -> BOOL {
    let mut pemr: PEMRNAMEDESCAPE = core::ptr::null_mut();
    let mut sizestr: UINT;
    let mut total: UINT;
    let rounded_size: UINT;
    let mut ret: INT;

    rounded_size = ((cbInput as UINT).wrapping_add(3)) & !(3 as UINT);
    total = (core::mem::offset_of!(EMRNAMEDESCAPE, Data) as UINT).wrapping_add(rounded_size);

    sizestr = ((wcslen(pDriver as *const WCHAR) as UINT)
        .wrapping_add(1 as UINT))
        .wrapping_mul(core::mem::size_of::<WCHAR>() as UINT);
    total = total.wrapping_add(sizestr);

    sizestr = sizestr;
    pemr = RtlAllocateHeap(
        GetProcessHeap(),
        0 as ULONG,
        total as SIZE_T,
    ) as PEMRNAMEDESCAPE;
    if pemr.is_null() {
        return 0 as BOOL;
    }

    core::ptr::write_bytes(pemr as *mut u8, 0, total as usize);

    (*pemr).emr.iType = EMR_NAMEDESCAPE as DWORD;
    (*pemr).emr.nSize = total as DWORD;
    (*pemr).iEsc = nEscape;
    (*pemr).cjIn = cbInput;

    // RtlCopyMemory( &pemr->Data[0], lpszInData, cbInput ): Rust's order is (src, dst, count).
    core::ptr::copy_nonoverlapping(
        lpszInData as *const u8,
        (&raw mut (*pemr).Data as *mut BYTE).offset(0 as isize) as *mut u8,
        cbInput as usize,
    );
    // WARNING :
    //   Need to remember with wine, theses headers are relocatable.
    // RtlCopyMemory( &pemr->Data[rounded_size], pDriver, sizestr ): (src, dst, count).
    core::ptr::copy_nonoverlapping(
        pDriver as *const u8,
        (&raw mut (*pemr).Data as *mut BYTE).offset(rounded_size as isize) as *mut u8,
        sizestr as usize,
    );

    ret = emfdc_record(((*dc_attr).emf).cast(), &raw mut (*pemr).emr) as INT;

    RtlFreeHeap(
        GetProcessHeap(),
        0 as ULONG,
        pemr as PVOID,
    );
    return ret as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetMetaRgn(dc_attr: *mut WINEDC) -> BOOL {
    let mut emr: EMRSETMETARGN = core::mem::zeroed();
    emr.emr.iType = EMR_SETMETARGN as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETMETARGN>() as DWORD;

    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}
#[no_mangle]
pub unsafe extern "C" fn EMFDC_SetBrushOrg(dc_attr: *mut WINEDC, x: INT, y: INT) -> BOOL {
    let mut emr: EMRSETBRUSHORGEX = core::mem::zeroed();

    emr.emr.iType = EMR_SETBRUSHORGEX as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMRSETBRUSHORGEX>() as DWORD;
    emr.ptlOrigin.x = x as LONG;
    emr.ptlOrigin.y = y as LONG;

    return emfdc_record((*dc_attr).emf as *mut emf, core::ptr::addr_of_mut!(emr.emr));
}

#[no_mangle]
pub unsafe extern "system" fn EMFDC_GdiComment(hdc: HDC, bytes: UINT, buffer: *const BYTE) -> BOOL {
    let mut dc_attr: *mut WINEDC = core::ptr::null_mut();
    let mut emr: *mut EMRGDICOMMENT = core::ptr::null_mut();
    let mut total: UINT = 0;
    let mut rounded_size: UINT = 0;
    let mut ret: BOOL = 0;

    dc_attr = get_dc_ptr(hdc) as *mut WINEDC;
    if dc_attr.is_null() || (*dc_attr).emf.is_null() {
        return (FALSE) as BOOL;
    }

    rounded_size = bytes.wrapping_add(3) & !(3);
    total = (core::mem::offset_of!(EMRGDICOMMENT, Data) as UINT).wrapping_add(rounded_size);

    emr = HeapAlloc(GetProcessHeap(), 0, (total) as SIZE_T) as *mut EMRGDICOMMENT;
    (*emr).emr.iType = EMR_GDICOMMENT;
    (*emr).emr.nSize = total;
    (*emr).cbData = bytes;
    memset(
        (&raw mut (*emr).Data as *mut BYTE).offset(bytes as isize) as *mut core::ffi::c_void,
        0,
        rounded_size.wrapping_sub(bytes) as usize,
    );
    memcpy(
        (&raw mut (*emr).Data as *mut BYTE).offset(0 as isize) as *mut core::ffi::c_void,
        buffer as *const core::ffi::c_void,
        bytes as usize,
    );

    ret = emfdc_record((*dc_attr).emf as *mut emf, &raw mut (*emr).emr);

    HeapFree(GetProcessHeap(), 0, emr as PVOID);

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateEnhMetaFileA(
    hdc: HDC,
    filename: *const ::core::ffi::c_char,
    rect: *const RECT,
    description: *const ::core::ffi::c_char,
) -> HDC {
    let mut filenameW: *mut WCHAR = core::ptr::null_mut();
    let mut descriptionW: *mut WCHAR = core::ptr::null_mut();
    let mut len1: DWORD = 0;
    let mut len2: DWORD = 0;
    let mut total: DWORD = 0;
    let mut ret: HDC = core::ptr::null_mut();

    if !filename.is_null() {
        total = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            filename as LPCSTR,
            -(1 as ::core::ffi::c_int),
            core::ptr::null_mut(),
            0 as ::core::ffi::c_int,
        ) as DWORD;
        filenameW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (total as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            filename as LPCSTR,
            -(1 as ::core::ffi::c_int),
            filenameW as LPWSTR,
            total as ::core::ffi::c_int,
        );
    }

    if !description.is_null() {
        len1 = strlen(description) as DWORD;
        len2 = strlen(
            description
                .offset(len1 as isize)
                .offset(1 as isize),
        ) as DWORD;
        total = MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            description as LPCSTR,
            len1.wrapping_add(len2).wrapping_add(3 as DWORD)
                as ::core::ffi::c_int,
            core::ptr::null_mut(),
            0 as ::core::ffi::c_int,
        ) as DWORD;
        descriptionW = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (total as SIZE_T).wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as *mut WCHAR;
        MultiByteToWideChar(
            CP_ACP as UINT,
            0 as DWORD,
            description as LPCSTR,
            len1.wrapping_add(len2).wrapping_add(3 as DWORD)
                as ::core::ffi::c_int,
            descriptionW as LPWSTR,
            total as ::core::ffi::c_int,
        );
    }

    ret = CreateEnhMetaFileW(
        hdc,
        filenameW as *const WCHAR,
        rect,
        descriptionW as *const WCHAR,
    );

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        filenameW as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        descriptionW as PVOID,
    );
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn CreateEnhMetaFileW(
    hdc: HDC,
    filename: *const WCHAR,
    rect: *const RECT,
    description: *const WCHAR,
) -> HDC {
    let mut ret: HDC = core::ptr::null_mut();
    let mut emf: *mut emf = core::ptr::null_mut();
    let mut dc_attr: *mut WINEDC = core::ptr::null_mut();
    let mut file: HANDLE = core::ptr::null_mut();
    let mut size: DWORD = 0 as DWORD;
    let mut length: DWORD = 0 as DWORD;

    //if (!(ret = NtGdiCreateMetafileDC( hdc ))) return 0;
    dc_attr = alloc_dc_ptr(OBJ_ENHMETADC as WORD) as *mut WINEDC;
    if dc_attr.is_null() {
        if !(*dc_attr).hdc.is_null() {
            DeleteDC((*dc_attr).hdc);
        }
        return (0 as usize) as HDC;
    }

    ret = (*dc_attr).hdc;

    emf = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<emf>() as SIZE_T,
    ) as *mut emf;
    if emf.is_null() {
        DeleteDC(ret);
        return (0 as usize) as HDC;
    }

    (*emf).dc_attr = dc_attr;
    (*dc_attr).emf = emf as PEMF;

    if !description.is_null() {
        length = lstrlenW(description as LPCWSTR) as DWORD;
        length = length.wrapping_add(lstrlenW(
            description
                .offset(length as isize)
                .offset(1 as isize),
        ) as DWORD);
        length = length.wrapping_add(3 as DWORD);
        length = length.wrapping_mul(2 as DWORD);
    }
    size = (core::mem::size_of::<ENHMETAHEADER>() as DWORD).wrapping_add(
        length
            .wrapping_add(3 as DWORD)
            .wrapping_div(4 as DWORD)
            .wrapping_mul(4 as DWORD),
    ) as DWORD;

    (*emf).emh = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        size as SIZE_T,
    ) as *mut ENHMETAHEADER;
    if (*emf).emh.is_null() {
        DeleteDC(ret);
        return (0 as usize) as HDC;
    }
    (*emf).dc_attr = dc_attr;

    (*emf).handles = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (HANDLE_LIST_INC as SIZE_T).wrapping_mul(
            (core::mem::size_of::<HGDIOBJ>() + (0 as usize) * 0) as SIZE_T,
        ),
    ) as *mut HGDIOBJ;
    (*emf).handles_size = HANDLE_LIST_INC as UINT;
    (*emf).cur_handles = 1 as UINT;
    (*emf).file = (0 as usize) as HANDLE;
    (*emf).dc_brush = (0 as usize) as HBRUSH;
    (*emf).dc_pen = (0 as usize) as HPEN;
    (*emf).path = FALSE as BOOL;

    (*(*emf).emh).iType = EMR_HEADER as DWORD;
    (*(*emf).emh).nSize = size;

    (*dc_attr).emf_bounds.top = 0 as LONG;
    (*dc_attr).emf_bounds.left = (*dc_attr).emf_bounds.top;
    (*dc_attr).emf_bounds.bottom = -(1 as LONG);
    (*dc_attr).emf_bounds.right = (*dc_attr).emf_bounds.bottom;

    if !rect.is_null() {
        (*(*emf).emh).rclFrame.left = (*rect).left;
        (*(*emf).emh).rclFrame.top = (*rect).top;
        (*(*emf).emh).rclFrame.right = (*rect).right;
        (*(*emf).emh).rclFrame.bottom = (*rect).bottom;
    } else {
        (*(*emf).emh).rclFrame.top = 0 as LONG;
        (*(*emf).emh).rclFrame.left = (*(*emf).emh).rclFrame.top;
        (*(*emf).emh).rclFrame.bottom = -(1 as LONG);
        (*(*emf).emh).rclFrame.right = (*(*emf).emh).rclFrame.bottom;
    }

    (*(*emf).emh).dSignature = ENHMETA_SIGNATURE as DWORD;
    (*(*emf).emh).nVersion = 0x10000 as DWORD;
    (*(*emf).emh).nBytes = (*(*emf).emh).nSize;
    (*(*emf).emh).nRecords = 1 as DWORD;
    (*(*emf).emh).nHandles = 1 as WORD;

    (*(*emf).emh).sReserved = 0 as WORD; /* According to docs, this is reserved and must be 0 */
    (*(*emf).emh).nDescription = length / (2 as DWORD);

    (*(*emf).emh).offDescription = (if length != 0 {
        core::mem::size_of::<ENHMETAHEADER>() as DWORD
    } else {
        0 as DWORD
    }) as DWORD;

    (*(*emf).emh).nPalEntries = 0 as DWORD; /* I guess this should start at 0 */

    /* Size in pixels */
    (*(*emf).emh).szlDevice.cx = GetDeviceCaps(ret, HORZRES as c_int) as LONG;
    (*(*emf).emh).szlDevice.cy = GetDeviceCaps(ret, VERTRES as c_int) as LONG;

    /* Size in millimeters */
    (*(*emf).emh).szlMillimeters.cx = GetDeviceCaps(ret, HORZSIZE as c_int) as LONG;
    (*(*emf).emh).szlMillimeters.cy = GetDeviceCaps(ret, VERTSIZE as c_int) as LONG;

    /* Size in micrometers */
    (*(*emf).emh).szlMicrometers.cx = (*(*emf).emh).szlMillimeters.cx * (1000 as LONG);
    (*(*emf).emh).szlMicrometers.cy = (*(*emf).emh).szlMillimeters.cy * (1000 as LONG);

    memcpy(
        ((*emf).emh as *mut c_char)
            .offset((core::mem::size_of::<ENHMETAHEADER>() + (core::mem::size_of::<ENHMETAHEADER>() as usize) * 0) as isize)
            as *mut c_void,
        description as *const c_void,
        length as usize,
    );

    if !filename.is_null() {
        /* disk based metafile */
        file = CreateFileW(
            filename as LPCWSTR,
            (GENERIC_WRITE as DWORD) | (GENERIC_READ as DWORD),
            0 as DWORD,
            core::ptr::null_mut(),
            CREATE_ALWAYS as DWORD,
            0 as DWORD,
            0 as usize as HANDLE,
        );
        if file == ((INVALID_HANDLE_VALUE as isize) as HANDLE) {
            DeleteDC(ret);
            return (0 as usize) as HDC;
        }
        (*emf).file = file;
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn CloseEnhMetaFile(mut hdc: HDC) -> HENHMETAFILE {
    let mut hmf: HENHMETAFILE = core::ptr::null_mut::<HENHMETAFILE__>();
    let mut emf: *mut emf = core::ptr::null_mut::<emf>();
    let mut dc_attr: *mut WINEDC = core::ptr::null_mut::<WINEDC>();
    let mut emr: EMREOF = tagEMREOF {
        emr: tagEMR { iType: 0, nSize: 0 },
        nPalEntries: 0,
        offPalEntries: 0,
        nSizeLast: 0,
    };
    let mut mapping: HANDLE = core::ptr::null_mut();
    dc_attr = get_dc_ptr(hdc) as *mut WINEDC;
    if dc_attr.is_null() || (*dc_attr).emf.is_null() {
        return core::ptr::null_mut();
    }
    emf = (*dc_attr).emf as *mut emf;
    if (*dc_attr).save_level != 0 {
        RestoreDC(hdc, 1 as ::core::ffi::c_int);
    }
    if !(*emf).dc_brush.is_null() {
        DeleteObject((*emf).dc_brush as HGDIOBJ);
    }
    if !(*emf).dc_pen.is_null() {
        DeleteObject((*emf).dc_pen as HGDIOBJ);
    }
    emr.emr.iType = EMR_EOF as DWORD;
    emr.emr.nSize = core::mem::size_of::<EMREOF>() as DWORD;
    emr.nPalEntries = 0 as DWORD;
    emr.offPalEntries = core::mem::offset_of!(EMREOF, nSizeLast) as DWORD;
    emr.nSizeLast = emr.emr.nSize;
    emfdc_record(emf, &raw mut emr.emr);
    (*(*emf).emh).rclBounds = (*dc_attr).emf_bounds;

    /* Update rclFrame if not initialized in CreateEnhMetaFile */
    if (*(*emf).emh).rclFrame.left > (*(*emf).emh).rclFrame.right {
        (*(*emf).emh).rclFrame.left =
            (*(*emf).emh).rclBounds.left * (*(*emf).emh).szlMillimeters.cx * 100 as LONG
                / (*(*emf).emh).szlDevice.cx;
        (*(*emf).emh).rclFrame.top =
            (*(*emf).emh).rclBounds.top * (*(*emf).emh).szlMillimeters.cy * 100 as LONG
                / (*(*emf).emh).szlDevice.cy;
        (*(*emf).emh).rclFrame.right =
            (*(*emf).emh).rclBounds.right * (*(*emf).emh).szlMillimeters.cx * 100 as LONG
                / (*(*emf).emh).szlDevice.cx;
        (*(*emf).emh).rclFrame.bottom =
            (*(*emf).emh).rclBounds.bottom * (*(*emf).emh).szlMillimeters.cy * 100 as LONG
                / (*(*emf).emh).szlDevice.cy;
    }

    if !(*emf).file.is_null() {
        /* disk based metafile */
        let mut bytes_written: DWORD = 0;
        if WriteFile(
            (*emf).file,
            (*emf).emh as LPCVOID,
            (*(*emf).emh).nBytes,
            &raw mut bytes_written,
            core::ptr::null_mut::<_OVERLAPPED>(),
        ) == 0
        {
            CloseHandle((*emf).file);
            return core::ptr::null_mut();
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            (*emf).emh as PVOID,
        );
        mapping = CreateFileMappingA(
            (*emf).file,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            PAGE_READONLY as DWORD,
            0 as DWORD,
            0 as DWORD,
            core::ptr::null::<CHAR>(),
        );
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/emfdc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CloseEnhMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                2559 as ::core::ffi::c_int,
                b"mapping = %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                mapping,
            );
        }
        (*emf).emh = MapViewOfFile(
            mapping,
            FILE_MAP_READ as DWORD,
            0 as DWORD,
            0 as DWORD,
            0 as SIZE_T,
        ) as *mut ENHMETAHEADER;
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/gdi/gdi32/wine/emfdc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CloseEnhMetaFile\0" as *const u8 as *const ::core::ffi::c_char,
                2561 as ::core::ffi::c_int,
                b"view = %p\n\0" as *const u8 as *const ::core::ffi::c_char,
                (*emf).emh,
            );
        }
        CloseHandle(mapping);
        CloseHandle((*emf).file);
    }

    hmf = EMF_Create_HENHMETAFILE(
        (*emf).emh,
        (*(*emf).emh).nBytes,
        ((*emf).file != core::ptr::null_mut()) as ::core::ffi::c_int,
    );
    (*emf).emh = core::ptr::null_mut::<ENHMETAHEADER>(); /* So it won't be deleted */
    DeleteDC(hdc);
    return hmf;
}
