use crate::prelude::*;
use core::ffi::c_void;

// WINE_DEFAULT_DEBUG_CHANNEL(metafile): shadows the prelude's (enhmetafile) default channel.
static mut __wine_dbch___default: *mut __wine_debug_channel =
    &raw mut __wine_dbch_metafile;

// ---- file-scope preamble (translated) ----
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct metadc {
    pub hdc: HDC,
    pub mh: *mut METAHEADER,
    pub handles_size: UINT,
    pub cur_handles: UINT,
    pub handles: *mut HGDIOBJ,
    pub hFile: HANDLE,
    pub pen: HPEN,
    pub brush: HBRUSH,
    pub font: HFONT,
}

pub const HANDLE_LIST_INC: i32 = 20;

// WINE_DEFAULT_DEBUG_CHANNEL(metafile): per-file debug channel declaration.
// Emitted by the Wine debug macro expansion; channel name is "metafile".
#[no_mangle]
pub unsafe extern "C" fn get_metadc_ptr(hdc: HDC) -> *mut metadc {
    let mut metafile: *mut metadc = GdiGetClientObjLink(hdc as HGDIOBJ) as *mut metadc;
    if metafile.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    }
    return metafile;
}

unsafe extern "C" fn metadc_write_record(metadc: *mut metadc, mr: *mut METARECORD, rlen: UINT) -> BOOL {
    let mut len: DWORD;
    let mut size: DWORD;
    let mut mh: *mut METAHEADER;
    len = (*(*metadc).mh).mtSize as DWORD * core::mem::size_of::<WORD>() as DWORD + rlen as DWORD;
    size = HeapSize(GetProcessHeap(), 0, (*metadc).mh as LPVOID) as DWORD;
    if len > size {
        size += size / core::mem::size_of::<WORD>() as DWORD + rlen as DWORD;
        mh = HeapReAlloc(GetProcessHeap(), 0, (*metadc).mh as LPVOID, size as SIZE_T) as *mut METAHEADER;
        if mh.is_null() {
            return (FALSE) as BOOL;
        }
        (*metadc).mh = mh;
    }
    memcpy(
        ((*metadc).mh as *mut WORD).offset((*(*metadc).mh).mtSize as isize) as *mut core::ffi::c_void,
        mr as *const core::ffi::c_void,
        rlen as usize,
    );
    (*(*metadc).mh).mtSize += (rlen as size_t / core::mem::size_of::<WORD>()) as DWORD;
    (*(*metadc).mh).mtMaxRecord = max((*(*metadc).mh).mtMaxRecord, (rlen as size_t / core::mem::size_of::<WORD>()) as DWORD);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn metadc_record(hdc: HDC, mr: *mut METARECORD, rlen: DWORD) -> BOOL {
    let metadc: *mut metadc;
    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    return metadc_write_record(metadc, mr, rlen);
}

unsafe extern "C" fn metadc_param0(hdc: HDC, func: INT16) -> BOOL {
    let mut mr: METARECORD = core::mem::zeroed();
    mr.rdSize = (FIELD_OFFSET!(METARECORD, rdParm[0]) as usize
        / core::mem::size_of::<WORD>()) as DWORD;
    mr.rdFunction = func as WORD;
    return metadc_record(
        hdc,
        core::ptr::addr_of_mut!(mr),
        (mr.rdSize as usize * core::mem::size_of::<WORD>()) as DWORD,
    );
}

unsafe extern "C" fn metadc_param1(hdc: HDC, func: i16, param: i16) -> BOOL {
    let mut mr: METARECORD = core::mem::zeroed();

    mr.rdSize = (core::mem::size_of::<METARECORD>() / core::mem::size_of::<WORD>()) as DWORD;
    mr.rdFunction = func as WORD;
    mr.rdParm[0] = param as WORD;
    return metadc_record(
        hdc,
        core::ptr::addr_of_mut!(mr),
        (mr.rdSize as usize).wrapping_mul(core::mem::size_of::<WORD>()) as DWORD,
    );
}

unsafe extern "C" fn metadc_param2(hdc: HDC, func: INT16, param1: INT16, param2: INT16) -> BOOL {
    let mut buffer: [u8; (FIELD_OFFSET!(METARECORD, rdParm) + (2 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize] = [0; (FIELD_OFFSET!(METARECORD, rdParm) + (2 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize];
    let mut mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (core::mem::size_of_val(&buffer) / core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = func as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).add(0) = param2 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).add(1) = param1 as WORD;
    return metadc_record(hdc, mr, core::mem::size_of_val(&buffer) as DWORD);
}

unsafe extern "C" fn metadc_param4(
    hdc: HDC,
    func: INT16,
    param1: INT16,
    param2: INT16,
    param3: INT16,
    param4: INT16,
) -> BOOL {
    let mut buffer: [u8; (FIELD_OFFSET!(METARECORD, rdParm) + (4 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize] =
        [0; (FIELD_OFFSET!(METARECORD, rdParm) + (4 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize];
    let mut mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (core::mem::size_of_val(&buffer) / core::mem::size_of::<WORD>()) as _;
    (*mr).rdFunction = func as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(0 as isize) = param4 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(1 as isize) = param3 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(2 as isize) = param2 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(3 as isize) = param1 as WORD;
    return metadc_record(
        hdc,
        mr,
        core::mem::size_of_val(&buffer) as DWORD,
    );
}

unsafe extern "C" fn metadc_param5(
    hdc: HDC,
    func: ::core::ffi::c_short,
    param1: ::core::ffi::c_short,
    param2: ::core::ffi::c_short,
    param3: ::core::ffi::c_short,
    param4: ::core::ffi::c_short,
    param5: ::core::ffi::c_short,
) -> BOOL {
    const BUFFER_LEN: usize = (FIELD_OFFSET!(METARECORD, rdParm)
        + (5 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize;
    let mut buffer: [u8; BUFFER_LEN] = [0u8; BUFFER_LEN];
    let mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (core::mem::size_of::<[u8; BUFFER_LEN]>()
        / core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = func as WORD;
    (*(*mr).rdParm.as_mut_ptr().add(0 as usize)) = param5 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(1 as usize)) = param4 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(2 as usize)) = param3 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(3 as usize)) = param2 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(4 as usize)) = param1 as WORD;
    return metadc_record(hdc, mr, core::mem::size_of::<[u8; BUFFER_LEN]>() as DWORD);
}

unsafe extern "C" fn metadc_param6(hdc: HDC, func: ::core::ffi::c_short, param1: ::core::ffi::c_short, param2: ::core::ffi::c_short, param3: ::core::ffi::c_short, param4: ::core::ffi::c_short, param5: ::core::ffi::c_short, param6: ::core::ffi::c_short) -> BOOL {
    let mut buffer: [::core::ffi::c_uchar; (core::mem::offset_of!(METARECORD, rdParm) + 6 * RTL_FIELD_SIZE!(METARECORD, rdParm))] = [0; (core::mem::offset_of!(METARECORD, rdParm) + 6 * RTL_FIELD_SIZE!(METARECORD, rdParm))];
    let mut mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (buffer.len() / core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = func as WORD;
    (*(*mr).rdParm.as_mut_ptr().add(0 as usize)) = param6 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(1 as usize)) = param5 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(2 as usize)) = param4 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(3 as usize)) = param3 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(4 as usize)) = param2 as WORD;
    (*core::ptr::addr_of_mut!((*mr).rdParm[0]).add(5 as usize)) = param1 as WORD;
    return metadc_record(hdc, mr, buffer.len() as DWORD);
}

unsafe extern "C" fn metadc_param8(hdc: HDC, func: INT16, param1: INT16, param2: INT16,
                                   param3: INT16, param4: INT16, param5: INT16,
                                   param6: INT16, param7: INT16, param8: INT16) -> BOOL {
    let mut buffer = [0u8; (FIELD_OFFSET!(METARECORD, rdParm) + (8 * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize];
    let mut mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (buffer.len() / core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = func as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(0 as isize) = param8 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(1 as isize) = param7 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(2 as isize) = param6 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(3 as isize) = param5 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(4 as isize) = param4 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(5 as isize) = param3 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(6 as isize) = param2 as WORD;
    *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(7 as isize) = param1 as WORD;
    return metadc_record(hdc, mr, buffer.len() as DWORD);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SaveDC(hdc: HDC) -> BOOL {
    return metadc_param0(hdc, META_SAVEDC as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_RestoreDC(hdc: HDC, level: INT) -> BOOL {
    return metadc_param1(hdc, META_RESTOREDC as i16, level as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetTextAlign(hdc: HDC, align: UINT) -> BOOL {
    return metadc_param2(hdc, META_SETTEXTALIGN as INT16, HIWORD!(align) as INT16, LOWORD!(align) as INT16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetBkMode(mut hdc: HDC, mut mode: INT) -> BOOL {
    return metadc_param1(
        hdc,
        META_SETBKMODE as ::core::ffi::c_short,
        mode as WORD as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetBkColor(hdc: HDC, color: COLORREF) -> BOOL {
    return metadc_param2(hdc, META_SETBKCOLOR as i16, HIWORD!(color) as i16, LOWORD!(color) as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetTextColor(hdc: HDC, color: COLORREF) -> BOOL {
    return metadc_param2(hdc, (META_SETTEXTCOLOR) as INT16, HIWORD!(color) as INT16, LOWORD!(color) as INT16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetROP2(hdc: HDC, rop: INT) -> BOOL {
    return metadc_param1(hdc, META_SETROP2 as ::core::ffi::c_short, rop as WORD as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetRelAbs(hdc: HDC, mode: INT) -> BOOL {
    return metadc_param1(hdc, META_SETRELABS as i16, mode as WORD as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetPolyFillMode(hdc: HDC, mode: INT) -> BOOL {
    return metadc_param1(
        hdc,
        META_SETPOLYFILLMODE as ::core::ffi::c_short,
        mode as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetStretchBltMode(hdc: HDC, mode: INT) -> BOOL {
    return metadc_param1(hdc, META_SETSTRETCHBLTMODE as i16, mode as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_IntersectClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL {
    return metadc_param4(hdc, META_INTERSECTCLIPRECT as ::core::ffi::c_short, left as ::core::ffi::c_short, top as ::core::ffi::c_short, right as ::core::ffi::c_short, bottom as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ExcludeClipRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL {
    return metadc_param4(
        hdc,
        META_EXCLUDECLIPRECT as ::core::ffi::c_short,
        left as ::core::ffi::c_short,
        top as ::core::ffi::c_short,
        right as ::core::ffi::c_short,
        bottom as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_OffsetClipRgn(mut hdc: HDC, mut x: INT, mut y: INT) -> BOOL {
    return metadc_param2(
        hdc,
        META_OFFSETCLIPRGN as ::core::ffi::c_short,
        x as ::core::ffi::c_short,
        y as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetLayout(hdc: HDC, layout: DWORD) -> BOOL {
    return metadc_param2(
        hdc,
        META_SETLAYOUT as i16,
        HIWORD!(layout) as i16,
        LOWORD!(layout) as i16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetMapMode(hdc: HDC, mode: INT) -> BOOL {
    return metadc_param1(hdc, META_SETMAPMODE as i16, mode as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetViewportExtEx(hdc: HDC, x: i32, y: i32) -> i32 {
    return metadc_param2(hdc, META_SETVIEWPORTEXT as ::core::ffi::c_short, x as ::core::ffi::c_short, y as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetViewportOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL {
    return metadc_param2(
        hdc,
        META_SETVIEWPORTORG as ::core::ffi::c_short,
        x as ::core::ffi::c_short,
        y as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetWindowExtEx(hdc: HDC, x: INT, y: INT) -> BOOL {
    return metadc_param2(hdc, META_SETWINDOWEXT as i16, x as i16, y as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetWindowOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL {
    return metadc_param2(hdc, META_SETWINDOWORG as ::core::ffi::c_short, x as ::core::ffi::c_short, y as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_OffsetViewportOrgEx(hdc: HDC, x: i32, y: i32) -> i32 {
    return metadc_param2(hdc, META_OFFSETVIEWPORTORG as i16, x as i16, y as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_OffsetWindowOrgEx(hdc: HDC, x: INT, y: INT) -> BOOL {
    return metadc_param2(hdc, META_OFFSETWINDOWORG as i16, x as i16, y as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ScaleViewportExtEx(hdc: HDC, x_num: INT, x_denom: INT, y_num: INT, y_denom: INT) -> BOOL {
    return metadc_param4(hdc, META_SCALEVIEWPORTEXT as ::core::ffi::c_short, x_num as ::core::ffi::c_short, x_denom as ::core::ffi::c_short, y_num as ::core::ffi::c_short, y_denom as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ScaleWindowExtEx(hdc: HDC, x_num: i32, x_denom: i32, y_num: i32, y_denom: i32) -> i32 {
    return metadc_param4(hdc, META_SCALEWINDOWEXT as i16, x_num as i16, x_denom as i16, y_num as i16, y_denom as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetTextJustification(hdc: HDC, extra: i32, breaks: i32) -> i32 {
    return metadc_param2(
        hdc,
        META_SETTEXTJUSTIFICATION as i16,
        extra as i16,
        breaks as i16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetTextCharacterExtra(hdc: HDC, extra: INT) -> BOOL {
    return metadc_param1(
        hdc,
        META_SETTEXTCHAREXTRA as ::core::ffi::c_short,
        extra as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetMapperFlags(hdc: HDC, flags: DWORD) -> BOOL {
    return metadc_param2(hdc, (META_SETMAPPERFLAGS) as INT16, HIWORD!(flags) as INT16, LOWORD!(flags) as INT16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_MoveTo(hdc: HDC, x: INT, y: INT) -> BOOL {
    return metadc_param2(hdc, META_MOVETO as ::core::ffi::c_short, x as ::core::ffi::c_short, y as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_LineTo(mut hdc: HDC, mut x: INT, mut y: INT) -> BOOL {
    return metadc_param2(
        hdc,
        META_LINETO as ::core::ffi::c_short,
        x as ::core::ffi::c_short,
        y as ::core::ffi::c_short,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Arc(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT, xstart: INT, ystart: INT, xend: INT, yend: INT) -> BOOL {
     return metadc_param8( hdc, META_ARC as i16, left as i16, top as i16, right as i16, bottom as i16,
                           xstart as i16, ystart as i16, xend as i16, yend as i16 );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Pie(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT, xstart: INT, ystart: INT, xend: INT, yend: INT) -> BOOL {
    return metadc_param8(hdc, META_PIE as i16, left as i16, top as i16, right as i16, bottom as i16, xstart as i16, ystart as i16, xend as i16, yend as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Chord(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT,
                   xstart: INT, ystart: INT, xend: INT, yend: INT) -> BOOL {
    return metadc_param8(hdc, META_CHORD as i16, left as i16, top as i16, right as i16, bottom as i16,
                          xstart as i16, ystart as i16, xend as i16, yend as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Ellipse(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL {
    return metadc_param4(
        hdc,
        META_ELLIPSE as i16,
        left as i16,
        top as i16,
        right as i16,
        bottom as i16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Rectangle(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT) -> BOOL {
    return metadc_param4(hdc, META_RECTANGLE as i16, left as i16, top as i16, right as i16, bottom as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_RoundRect(hdc: HDC, left: INT, top: INT, right: INT, bottom: INT, ell_width: INT, ell_height: INT) -> BOOL {
    return metadc_param6(hdc, META_ROUNDRECT as ::core::ffi::c_short, left as ::core::ffi::c_short, top as ::core::ffi::c_short, right as ::core::ffi::c_short, bottom as ::core::ffi::c_short, ell_width as ::core::ffi::c_short, ell_height as ::core::ffi::c_short);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetPixel(hdc: HDC, x: INT, y: INT, color: COLORREF) -> BOOL {
    return metadc_param4(
        hdc,
        (META_SETPIXEL) as INT16,
        x as i16,
        y as i16,
        HIWORD!(color) as i16,
        LOWORD!(color) as i16,
    );
}

unsafe extern "C" fn metadc_poly(
    mut hdc: HDC,
    mut func: ::core::ffi::c_short,
    mut pt: *mut POINTS,
    mut count: ::core::ffi::c_short,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut len: DWORD = 0;
    let mut mr: *mut METARECORD = ::core::ptr::null_mut::<METARECORD>();
    len = (::core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add((count as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as usize)
        as DWORD;
    mr = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY as DWORD, len as SIZE_T) as *mut METARECORD;
    if mr.is_null() {
        return (FALSE) as BOOL;
    }
    (*mr).rdSize = len.wrapping_div(2 as DWORD);
    (*mr).rdFunction = func as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD) = count as WORD;
    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD).offset(1 as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        pt as *const ::core::ffi::c_void,
        (count as ::core::ffi::c_int * 4 as ::core::ffi::c_int) as usize,
    );
    ret = metadc_record(hdc, mr, (*mr).rdSize.wrapping_mul(2 as DWORD));
    HeapFree(GetProcessHeap(), 0 as DWORD, mr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Polyline(hdc: HDC, pt: *const POINT, count: INT) -> BOOL {
    let mut i: INT = 0;
    let mut pts: *mut POINTS = core::ptr::null_mut::<POINTS>();
    let mut ret: BOOL = 0;

    pts = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((core::mem::size_of::<POINTS>() as usize).wrapping_mul(count as usize)) as SIZE_T,
    ) as *mut POINTS;
    if pts.is_null() {
        return (FALSE) as BOOL;
    }
    i = count;
    loop {
        let fresh = i;
        i = i - 1;
        if !(fresh != 0) {
            break;
        }
        (*pts.offset(i as isize)).x = (*pt.offset(i as isize)).x as INT16;
        (*pts.offset(i as isize)).y = (*pt.offset(i as isize)).y as INT16;
    }
    ret = metadc_poly(
        hdc,
        META_POLYLINE as INT16,
        pts,
        count as INT16,
    );

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pts as LPVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_Polygon(
    mut hdc: HDC,
    mut pt: *const POINT,
    mut count: INT,
) -> BOOL {
    let mut i: ::core::ffi::c_int = 0;
    let mut pts: *mut POINTS = ::core::ptr::null_mut::<POINTS>();
    let mut ret: BOOL = 0;
    pts = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((::core::mem::size_of::<POINTS>()).wrapping_mul(count as usize)) as SIZE_T,
    ) as *mut POINTS;
    if pts.is_null() {
        return (FALSE) as BOOL;
    }
    i = count as ::core::ffi::c_int;
    loop {
        let fresh16 = i;
        i = i - 1;
        if !(fresh16 != 0) {
            break;
        }
        (*pts.offset(i as isize)).x = (*pt.offset(i as isize)).x as SHORT;
        (*pts.offset(i as isize)).y = (*pt.offset(i as isize)).y as SHORT;
    }
    ret = metadc_poly(
        hdc,
        META_POLYGON as ::core::ffi::c_short,
        pts,
        count as ::core::ffi::c_short,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        pts as *mut ::core::ffi::c_void,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_PolyPolygon(
    hdc: HDC,
    pt: *const POINT,
    counts: *const INT,
    polygons: UINT,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut len: DWORD = 0;
    let mut mr: *mut METARECORD = core::ptr::null_mut::<METARECORD>();
    let mut i: UINT = 0;
    let mut j: UINT = 0;
    let mut pts: *mut POINTS = core::ptr::null_mut::<POINTS>();
    let mut totalpoint16: INT16 = 0;
    let mut pointcounts: *mut INT16 = core::ptr::null_mut::<INT16>();

    i = 0;
    while i < polygons {
        totalpoint16 = (totalpoint16 as core::ffi::c_int
            + *counts.offset(i as isize) as core::ffi::c_int) as INT16;
        i = i.wrapping_add(1);
    }

    /* allocate space for all points */
    pts = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<POINTS>() as SIZE_T).wrapping_mul(totalpoint16 as SIZE_T),
    ) as *mut POINTS;
    pointcounts = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<INT16>() as SIZE_T).wrapping_mul(totalpoint16 as SIZE_T),
    ) as *mut INT16;

    /* copy point counts */
    i = 0;
    while i < polygons {
        *pointcounts.offset(i as isize) = *counts.offset(i as isize) as INT16;
        i = i.wrapping_add(1);
    }

    /* convert all points */
    j = totalpoint16 as UINT;
    loop {
        let fresh17 = j;
        j = j.wrapping_sub(1);
        if !(fresh17 != 0) {
            break;
        }
        (*pts.offset(j as isize)).x = (*pt.offset(j as isize)).x as SHORT;
        (*pts.offset(j as isize)).y = (*pt.offset(j as isize)).y as SHORT;
    }

    len = (core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add(core::mem::size_of::<WORD>() as usize)
        .wrapping_add((polygons as usize).wrapping_mul(core::mem::size_of::<INT16>() as usize))
        .wrapping_add(
            (totalpoint16 as usize).wrapping_mul(core::mem::size_of::<POINTS>() as usize),
        ) as DWORD;

    mr = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        HeapFree(GetProcessHeap(), 0, pts as PVOID);
        HeapFree(
            GetProcessHeap(),
            0,
            pointcounts as PVOID,
        );
        return (FALSE) as BOOL;
    }

    (*mr).rdSize = (len as usize)
        .wrapping_div(core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = META_POLYPOLYGON as WORD;
    *((&raw mut (*mr).rdParm) as *mut WORD) = polygons as WORD;
    memcpy(
        ((&raw mut (*mr).rdParm) as *mut WORD).offset(1 as isize)
            as *mut core::ffi::c_void,
        pointcounts as *const core::ffi::c_void,
        (polygons as usize).wrapping_mul(core::mem::size_of::<INT16>()),
    );
    memcpy(
        ((&raw mut (*mr).rdParm) as *mut WORD)
            .offset(1 as isize)
            .offset(polygons as isize) as *mut core::ffi::c_void,
        pts as *const core::ffi::c_void,
        (totalpoint16 as usize).wrapping_mul(core::mem::size_of::<POINTS>() as usize),
    );
    ret = metadc_record(
        hdc,
        mr,
        ((*mr).rdSize as usize).wrapping_mul(core::mem::size_of::<WORD>())
            as DWORD,
    );

    HeapFree(GetProcessHeap(), 0, pts as PVOID);
    HeapFree(
        GetProcessHeap(),
        0,
        pointcounts as PVOID,
    );
    HeapFree(GetProcessHeap(), 0, mr as PVOID);
    return ret;
}

#[no_mangle]
pub unsafe extern "C" fn METADC_ExtFloodFill(hdc: HDC, x: INT, y: INT, color: COLORREF, fill_type: UINT) -> BOOL {
    return metadc_param5(
        hdc,
        (META_EXTFLOODFILL as i16),
        x as i16,
        y as i16,
        HIWORD!(color) as i16,
        LOWORD!(color) as i16,
        fill_type as i16,
    );
}

unsafe extern "C" fn metadc_add_handle(metadc: *mut metadc, obj: HGDIOBJ) -> UINT {
    let mut index: UINT16 = 0 as UINT16;
    while (index as UINT) < (*metadc).handles_size {
        if *(*metadc).handles.offset(index as isize) == 0 as HGDIOBJ {
            break;
        }
        index = index.wrapping_add(1);
    }
    if index as UINT == (*metadc).handles_size {
        (*metadc).handles_size = (*metadc).handles_size.wrapping_add(HANDLE_LIST_INC as UINT);
        let elem_idx: usize = 0 as usize;
        let _ = elem_idx;
        (*metadc).handles = HeapReAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (*metadc).handles as LPVOID,
            ((*metadc).handles_size as SIZE_T)
                .wrapping_mul(core::mem::size_of::<HGDIOBJ>() as SIZE_T),
        ) as *mut HGDIOBJ;
    }
    *(*metadc).handles.offset(index as isize) = get_full_gdi_handle(obj);

    (*metadc).cur_handles = (*metadc).cur_handles.wrapping_add(1);
    if (*metadc).cur_handles > (*(*metadc).mh).mtNoObjects as UINT {
        (*(*metadc).mh).mtNoObjects = (*(*metadc).mh).mtNoObjects.wrapping_add(1);
    }

    return index as UINT;
}

unsafe extern "C" fn metadc_remove_handle(metadc: *mut metadc, index: UINT) -> BOOL {
    let mut ret: BOOL = (FALSE) as i32;

    if index < (*metadc).handles_size && !(*(*metadc).handles.offset(index as isize)).is_null() {
        *(*metadc).handles.offset(index as isize) = core::ptr::null_mut();
        (*metadc).cur_handles = (*metadc).cur_handles.wrapping_sub(1);
        ret = (TRUE) as i32;
    }
    return ret;
}

unsafe extern "C" fn metadc_create_brush(metadc: *mut metadc, brush: HBRUSH) -> INT16 {
    let mut size: DWORD = 0;
    let mut mr: *mut METARECORD = core::ptr::null_mut();
    let mut logbrush: LOGBRUSH = core::mem::zeroed();
    let mut r: BOOL = 0;

    if GetObjectA((brush).cast(), core::mem::size_of::<LOGBRUSH>() as core::ffi::c_int, core::ptr::addr_of_mut!(logbrush) as LPVOID) == 0 {
        return -(1 as INT16);
    }

    'done: {
        match logbrush.lbStyle {
            BS_SOLID | BS_NULL | BS_HATCHED => {
                let mut lb16: LOGBRUSH16 = core::mem::zeroed();

                lb16.lbStyle = logbrush.lbStyle as UINT16;
                lb16.lbColor = logbrush.lbColor;
                lb16.lbHatch = logbrush.lbHatch as INT16;
                size = (core::mem::size_of::<METARECORD>() as usize)
                    .wrapping_add(core::mem::size_of::<LOGBRUSH16>() as usize)
                    .wrapping_sub(core::mem::size_of::<WORD>() as usize) as DWORD;
                mr = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut METARECORD;
                (*mr).rdSize = (size as usize).wrapping_div(core::mem::size_of::<WORD>() as usize) as DWORD;
                (*mr).rdFunction = META_CREATEBRUSHINDIRECT as WORD;
                memcpy(
                    core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(lb16) as *const core::ffi::c_void,
                    core::mem::size_of::<LOGBRUSH16>() as size_t,
                );
            }
            BS_PATTERN | BS_DIBPATTERN => {
                let mut buffer: [core::ffi::c_uchar; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize] =
                    [0; (FIELD_OFFSET!(BITMAPINFO, bmiColors) + (256 * RTL_FIELD_SIZE!(BITMAPINFO, bmiColors)) as i32) as usize];
                let mut dst_info: *mut BITMAPINFO = core::ptr::null_mut();
                let mut src_info: *mut BITMAPINFO = buffer.as_mut_ptr() as *mut BITMAPINFO;
                let mut info_size: DWORD = 0;
                let mut usage: UINT = 0;

                if get_brush_bitmap_info(brush, src_info, core::ptr::null_mut(), core::ptr::addr_of_mut!(usage)) == 0 {
                    break 'done;
                }

                info_size = get_dib_info_size(src_info, usage) as DWORD;
                size = FIELD_OFFSET!(METARECORD, rdParm[2]) as DWORD
                    + info_size
                    + (*src_info).bmiHeader.biSizeImage;

                mr = HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut METARECORD;
                if mr.is_null() {
                    break 'done;
                }
                (*mr).rdFunction = META_DIBCREATEPATTERNBRUSH as WORD;
                (*mr).rdSize = (size as usize).wrapping_div(core::mem::size_of::<WORD>() as usize) as DWORD;
                *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(0 as isize) = logbrush.lbStyle as WORD;
                *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(1 as isize) = usage as WORD;
                dst_info = (core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(2 as isize) as *mut BITMAPINFO;
                get_brush_bitmap_info(
                    brush,
                    dst_info,
                    (dst_info as *mut core::ffi::c_char).offset(info_size as isize) as *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                );
                if (*dst_info).bmiHeader.biClrUsed
                    == ((1 as DWORD) << (*dst_info).bmiHeader.biBitCount as u32)
                {
                    (*dst_info).bmiHeader.biClrUsed = 0 as DWORD;
                }
            }
            _ => {
                if (*__wine_dbch___default).flags as core::ffi::c_int
                    & (1 as core::ffi::c_int) << __WINE_DBCL_FIXME as core::ffi::c_int
                    != 0
                {
                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
                    ros_dbg_log(
                        __dbcl,
                        __dbch,
                        b"win32ss/gdi/gdi32/wine/metadc.c\0"
                            as *const u8 as *const core::ffi::c_char,
                        b"metadc_create_brush\0" as *const u8 as *const core::ffi::c_char,
                        587 as core::ffi::c_int,
                        b"Unknown brush style %x\n\0" as *const u8 as *const core::ffi::c_char,
                        logbrush.lbStyle,
                    );
                }
                return 0 as INT16;
            }
        }

        r = metadc_write_record(
            metadc,
            mr,
            ((*mr).rdSize as usize).wrapping_mul(core::mem::size_of::<WORD>() as usize) as u32,
        );
        HeapFree(GetProcessHeap(), 0, mr as PVOID);
        if r == 0 {
            return -(1 as INT16);
        }
    }

    return metadc_add_handle(metadc, brush as HGDIOBJ) as INT16;
}

unsafe extern "C" fn metadc_create_region(metadc: *mut metadc, hrgn: HRGN) -> INT16 {
    let mut len: DWORD = 0;
    let mut mr: *mut METARECORD = core::ptr::null_mut();
    let mut rgndata: *mut RGNDATA = core::ptr::null_mut();
    let mut cur_rect: *mut RECT = core::ptr::null_mut();
    let mut end_rect: *mut RECT = core::ptr::null_mut();
    let mut bands: WORD = 0 as WORD;
    let mut max_bounds: WORD = 0 as WORD;
    let mut param: *mut WORD = core::ptr::null_mut();
    let mut start_band: *mut WORD = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    len = GetRegionData(hrgn, 0 as DWORD, core::ptr::null_mut());
    if len == 0 {
        return -(1 as INT16);
    }
    rgndata = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        len as SIZE_T,
    ) as *mut RGNDATA;
    if rgndata.is_null() {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"win32ss/gdi/gdi32/wine/metadc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"metadc_create_region\0" as *const u8 as *const ::core::ffi::c_char,
                611 as ::core::ffi::c_int,
                b"Can't alloc rgndata buffer\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return -(1 as INT16);
    }
    GetRegionData(hrgn, len, rgndata as LPRGNDATA);

    /* Overestimate of length:
     * Assume every rect is a separate band -> 6 WORDs per rect,
     * see MF_Play_MetaCreateRegion for format details.
     */
    len = (core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add(20 as usize)
        .wrapping_add(((*rgndata).rdh.nCount.wrapping_mul(12 as DWORD)) as usize)
        as DWORD;
    mr = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int))
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"win32ss/gdi/gdi32/wine/metadc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"metadc_create_region\0" as *const u8 as *const ::core::ffi::c_char,
                623 as ::core::ffi::c_int,
                b"Can't alloc METARECORD buffer\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            rgndata as PVOID,
        );
        return -(1 as INT16);
    }

    param = (&raw mut (*mr).rdParm as *mut WORD).offset(11 as isize);
    start_band = core::ptr::null_mut();

    end_rect = (&raw mut (*rgndata).Buffer as *mut RECT).offset((*rgndata).rdh.nCount as isize);
    cur_rect = &raw mut (*rgndata).Buffer as *mut RECT;
    while cur_rect < end_rect {
        if !start_band.is_null()
            && (*cur_rect).top as LONG == *start_band.offset(1 as isize) as LONG
        {
            *param = (*cur_rect).left as WORD;
            param = param.offset(1);
            *param = (*cur_rect).right as WORD;
            param = param.offset(1);
        } else {
            if !start_band.is_null() {
                *start_band = (param.offset_from(start_band) as isize
                    - 3 as isize) as WORD;
                *param = *start_band;
                param = param.offset(1);
                if *start_band > max_bounds {
                    max_bounds = *start_band;
                }
                bands = bands.wrapping_add(1);
            }
            start_band = param;
            param = param.offset(1);
            *param = (*cur_rect).top as WORD;
            param = param.offset(1);
            *param = (*cur_rect).bottom as WORD;
            param = param.offset(1);
            *param = (*cur_rect).left as WORD;
            param = param.offset(1);
            *param = (*cur_rect).right as WORD;
            param = param.offset(1);
        }
        cur_rect = cur_rect.offset(1);
    }

    if !start_band.is_null() {
        *start_band = (param.offset_from(start_band) as isize
            - 3 as isize) as WORD;
        *param = *start_band;
        param = param.offset(1);
        if *start_band > max_bounds {
            max_bounds = *start_band;
        }
        bands = bands.wrapping_add(1);
    }

    *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize) =
        0 as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(1 as isize) =
        6 as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize) =
        0x2f6 as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(3 as isize) =
        0 as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(4 as isize) =
        ((param.offset_from(&raw mut (*mr).rdFunction) as isize as usize)
            .wrapping_mul(core::mem::size_of::<WORD>() as usize)) as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(5 as isize) = bands;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(6 as isize) = max_bounds;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(7 as isize) =
        (*rgndata).rdh.rcBound.left as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(8 as isize) =
        (*rgndata).rdh.rcBound.top as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(9 as isize) =
        (*rgndata).rdh.rcBound.right as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(10 as isize) =
        (*rgndata).rdh.rcBound.bottom as WORD;
    (*mr).rdFunction = META_CREATEREGION as WORD;
    (*mr).rdSize = param.offset_from(mr as *mut WORD) as isize as DWORD;
    ret = metadc_write_record(
        metadc,
        mr,
        (*mr).rdSize.wrapping_mul(2 as DWORD) as UINT,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        rgndata as PVOID,
    );
    if ret == 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int))
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"win32ss/gdi/gdi32/wine/metadc.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"metadc_create_region\0" as *const u8 as *const ::core::ffi::c_char,
                684 as ::core::ffi::c_int,
                b"MFDRV_WriteRecord failed\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return -(1 as INT16);
    }
    return metadc_add_handle(metadc, (hrgn).cast()) as INT16;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_PaintRgn(mut hdc: HDC, mut hrgn: HRGN) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut index: INT16 = 0;
    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    index = metadc_create_region(metadc, hrgn);
    if index as ::core::ffi::c_int == -(1) {
        return (FALSE) as BOOL;
    }
    return metadc_param1(hdc, (META_PAINTREGION as i16), index);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_InvertRgn(mut hdc: HDC, mut hrgn: HRGN) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut index: INT16 = 0;
    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    index = metadc_create_region(metadc, hrgn);
    if index == -1 {
        return (FALSE) as BOOL;
    }
    return metadc_param1(hdc, (META_INVERTREGION as i16), index);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_FillRgn(hdc: HDC, hrgn: HRGN, hbrush: HBRUSH) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut rgn: INT16 = 0;
    let mut brush: INT16 = 0;

    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }

    rgn = metadc_create_region(metadc, hrgn);
    if rgn == -1 {
        return (FALSE) as BOOL;
    }
    brush = metadc_create_brush(metadc, hbrush);
    if brush == 0 {
        return (FALSE) as BOOL;
    }
    return metadc_param2(hdc, (META_FILLREGION) as INT16, rgn, brush);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_FrameRgn(hdc: HDC, hrgn: HRGN, hbrush: HBRUSH, x: INT, y: INT) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut::<metadc>();
    let mut rgn: INT16 = 0;
    let mut brush: INT16 = 0;

    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    rgn = metadc_create_region(metadc, hrgn);
    if rgn == -1 {
        return (FALSE) as BOOL;
    }
    brush = metadc_create_brush(metadc, hbrush);
    if brush == 0 {
        return (FALSE) as BOOL;
    }
    return metadc_param4(
        hdc,
        META_FRAMEREGION as i16,
        rgn as i16,
        brush as i16,
        x as i16,
        y as i16,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_PatBlt(hdc: HDC, left: INT, top: INT, width: INT, height: INT, rop: DWORD) -> BOOL {
    return metadc_param6(
        hdc,
        META_PATBLT as INT16,
        left as INT16,
        top as INT16,
        width as INT16,
        height as INT16,
        HIWORD!(rop) as INT16,
        LOWORD!(rop) as INT16,
    );
}

unsafe extern "C" fn metadc_stretchblt(
    hdc: HDC,
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
    type_: WORD,
) -> BOOL {
    let mut src_info: BITMAPINFO = core::mem::zeroed();
    src_info.bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as DWORD;
    let mut bmi_size: UINT = 0;
    let mut size: UINT = 0;
    let mut bpp: UINT = 0;
    let mut i: INT = 0;
    let mut bitmap_offset: INT = 0;
    let mut bmi: *mut BITMAPINFO = core::ptr::null_mut();
    let mut mr: *mut METARECORD = core::ptr::null_mut();
    let mut bitmap: HBITMAP = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    bitmap = GetCurrentObject(hdc_src, OBJ_BITMAP as UINT) as HBITMAP;
    if bitmap.is_null() {
        return (FALSE) as BOOL;
    }
    if GetDIBits(
        hdc_src,
        bitmap,
        0 as UINT,
        INT_MAX as UINT,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(src_info),
        DIB_RGB_COLORS as UINT,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    bpp = src_info.bmiHeader.biBitCount as UINT;
    if bpp <= 8 as UINT {
        bmi_size = (core::mem::size_of::<BITMAPINFOHEADER>() as UINT).wrapping_add(
            ((1 as UINT) << bpp)
                .wrapping_mul(core::mem::size_of::<RGBQUAD>() as UINT),
        );
    } else if bpp == 16 as UINT || bpp == 32 as UINT {
        bmi_size = (core::mem::size_of::<BITMAPINFOHEADER>() as UINT).wrapping_add(
            (3 as UINT)
                .wrapping_mul(core::mem::size_of::<RGBQUAD>() as UINT),
        );
    } else {
        bmi_size = core::mem::size_of::<BITMAPINFOHEADER>() as UINT;
    }

    bitmap_offset = if type_ as INT == (META_DIBBITBLT as i32) {
        8
    } else {
        10
    };
    size = (FIELD_OFFSET!(METARECORD, rdParm[bitmap_offset as usize]) as UINT)
        .wrapping_add(bmi_size)
        .wrapping_add(src_info.bmiHeader.biSizeImage);
    mr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        size as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        return (FALSE) as BOOL;
    }
    (*mr).rdFunction = type_;
    bmi = (core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(bitmap_offset as isize)
        as *mut BITMAPINFO;
    (*bmi).bmiHeader = src_info.bmiHeader;

    ret = GetDIBits(
        hdc_src,
        bitmap,
        0 as UINT,
        src_info.bmiHeader.biHeight as UINT,
        (bmi as *mut BYTE).offset(bmi_size as isize) as LPVOID,
        bmi,
        DIB_RGB_COLORS as UINT,
    ) as BOOL;
    if ret != 0 {
        (*mr).rdSize = (size / core::mem::size_of::<WORD>() as UINT) as DWORD;

        let fresh29 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh29 as isize) =
            (LOWORD!(rop) as DWORD_PTR & 0xffff as DWORD_PTR) as WORD;
        let fresh30 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh30 as isize) =
            (HIWORD!(rop) as DWORD_PTR & 0xffff as DWORD_PTR) as WORD;
        if bitmap_offset > 8 {
            let fresh31 = i;
            i = i + 1;
            *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh31 as isize) =
                height_src as WORD;
            let fresh32 = i;
            i = i + 1;
            *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh32 as isize) =
                width_src as WORD;
        }
        let fresh33 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh33 as isize) =
            y_src as WORD;
        let fresh34 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh34 as isize) =
            x_src as WORD;
        let fresh35 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh35 as isize) =
            height_dst as WORD;
        let fresh36 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh36 as isize) =
            width_dst as WORD;
        let fresh37 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh37 as isize) =
            y_dst as WORD;
        let fresh38 = i;
        i = i + 1;
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD).offset(fresh38 as isize) =
            x_dst as WORD;
        ret = metadc_record(hdc, mr, size as DWORD);
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_BitBlt(hdc: HDC, x_dst: INT, y_dst: INT, width: INT, height: INT, hdc_src: HDC, x_src: INT, y_src: INT, rop: DWORD) -> BOOL {
    return metadc_stretchblt(hdc, x_dst, y_dst, width, height, hdc_src, x_src, y_src, width, height, rop, META_DIBBITBLT as WORD);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_StretchBlt(
    hdc_dst: HDC,
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
    return metadc_stretchblt(
        hdc_dst,
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
        META_DIBSTRETCHBLT as WORD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn METADC_StretchDIBits(
    hdc: HDC,
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
) -> INT {
    let infosize: DWORD = get_dib_info_size(info, usage) as DWORD;
    let len: DWORD = (core::mem::size_of::<METARECORD>()
        + 10 as usize * core::mem::size_of::<WORD>()
        + infosize as usize
        + (*info).bmiHeader.biSizeImage as usize) as DWORD;
    let mut mr: *mut METARECORD = core::ptr::null_mut();

    mr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        return 0 as INT;
    }

    (*mr).rdSize = (len as usize / (core::mem::size_of::<WORD>() as usize)) as DWORD;
    (*mr).rdFunction = META_STRETCHDIB as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize) = LOWORD!(rop);
    *(&raw mut (*mr).rdParm as *mut WORD).offset(1 as isize) = HIWORD!(rop);
    *(&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize) = usage as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(3 as isize) = height_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(4 as isize) = width_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(5 as isize) = y_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(6 as isize) = x_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(7 as isize) = height_dst as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(8 as isize) = width_dst as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(9 as isize) = y_dst as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(10 as isize) = x_dst as WORD;

    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD).offset(11 as isize)
            as *mut core::ffi::c_void,
        info as *const core::ffi::c_void,
        infosize as usize,
    );
    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD)
            .offset(11 as isize)
            .offset((infosize / (2 as DWORD)) as isize)
            as *mut core::ffi::c_void,
        bits,
        (*info).bmiHeader.biSizeImage as usize,
    );
    metadc_record(
        hdc,
        mr,
        (*mr).rdSize * (core::mem::size_of::<WORD>() as DWORD),
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as PVOID,
    );
    return height_src;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SetDIBitsToDevice(
    hdc: HDC,
    x_dst: INT,
    y_dst: INT,
    width: DWORD,
    height: DWORD,
    x_src: INT,
    y_src: INT,
    startscan: UINT,
    lines: UINT,
    bits: *const core::ffi::c_void,
    info: *const BITMAPINFO,
    coloruse: UINT,
) -> INT {
    let infosize: DWORD = get_dib_info_size(info, coloruse) as DWORD;
    let len: DWORD = (core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add((8 as usize).wrapping_mul(core::mem::size_of::<WORD>() as usize))
        .wrapping_add(infosize as usize)
        .wrapping_add((*info).bmiHeader.biSizeImage as usize) as DWORD;
    let mut mr: *mut METARECORD = core::ptr::null_mut::<METARECORD>();

    mr = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        return 0 as INT;
    }

    (*mr).rdSize = (len as usize).wrapping_div(core::mem::size_of::<WORD>() as usize) as DWORD;
    (*mr).rdFunction = META_SETDIBTODEV as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize) = coloruse as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(1 as isize) = lines as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize) = startscan as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(3 as isize) = y_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(4 as isize) = x_src as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(5 as isize) = height as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(6 as isize) = width as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(7 as isize) = y_dst as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(8 as isize) = x_dst as WORD;

    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD).offset(9 as isize)
            as *mut core::ffi::c_void,
        info as *const core::ffi::c_void,
        infosize as size_t,
    );
    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD)
            .offset(9 as isize)
            .offset((infosize as usize).wrapping_div(core::mem::size_of::<WORD>() as usize) as isize)
            as *mut core::ffi::c_void,
        bits,
        (*info).bmiHeader.biSizeImage as size_t,
    );
    metadc_record(
        hdc,
        mr,
        ((*mr).rdSize as usize).wrapping_mul(core::mem::size_of::<WORD>() as usize) as DWORD,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as PVOID,
    );
    return lines as INT;
}

unsafe extern "C" fn metadc_text(
    hdc: HDC,
    x: ::core::ffi::c_short,
    y: ::core::ffi::c_short,
    flags: UINT16,
    rect: *const RECT16,
    str: *const ::core::ffi::c_char,
    count: ::core::ffi::c_short,
    dx: *const INT16,
) -> BOOL {
    let mut ret: BOOL = 0;
    let mut len: DWORD = 0;
    let mut mr: *mut METARECORD = ::core::ptr::null_mut::<METARECORD>();
    let mut isrect: BOOL = (flags as BOOL) & ((ETO_CLIPPED | ETO_OPAQUE)) as i32;
    len = (::core::mem::size_of::<METARECORD>() as DWORD)
        .wrapping_add(
            ((((count as ::core::ffi::c_int) + (1 as ::core::ffi::c_int))
                >> (1 as ::core::ffi::c_int))
                * (2 as ::core::ffi::c_int)) as DWORD,
        )
        .wrapping_add(
            (2 as DWORD)
                .wrapping_mul(::core::mem::size_of::<::core::ffi::c_short>() as DWORD),
        )
        .wrapping_add(core::mem::size_of::<UINT16>() as DWORD);
    if isrect != 0 {
        len = len.wrapping_add(core::mem::size_of::<RECT16>() as DWORD);
    }
    if !dx.is_null() {
        len = len.wrapping_add((count as DWORD).wrapping_mul(core::mem::size_of::<INT16>() as DWORD));
    }
    mr = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        return FALSE as BOOL;
    }
    (*mr).rdSize = len / (core::mem::size_of::<WORD>() as DWORD);
    (*mr).rdFunction = META_EXTTEXTOUT as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize) = y as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(1 as isize) = x as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize) = count as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(3 as isize) = flags as WORD;
    if isrect != 0 {
        memcpy(
            (&raw mut (*mr).rdParm as *mut WORD).offset(4 as isize)
                as *mut ::core::ffi::c_void,
            rect as *const ::core::ffi::c_void,
            (core::mem::size_of::<RECT16>() as usize),
        );
    }
    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD).offset(
            (if isrect != 0 {
                8
            } else {
                4
            }) as isize,
        ) as *mut ::core::ffi::c_void,
        str as *const ::core::ffi::c_void,
        count as usize,
    );
    if !dx.is_null() {
        memcpy(
            (&raw mut (*mr).rdParm as *mut WORD)
                .offset(
                    (if isrect != 0 {
                        8
                    } else {
                        4
                    }) as isize,
                )
                .offset(
                    (((count as ::core::ffi::c_int) + (1 as ::core::ffi::c_int))
                        >> (1 as ::core::ffi::c_int))
                        as isize,
                ) as *mut ::core::ffi::c_void,
            dx as *const ::core::ffi::c_void,
            (count as usize).wrapping_mul(core::mem::size_of::<INT16>() as usize),
        );
    }
    ret = metadc_record(
        hdc,
        mr,
        (*mr).rdSize.wrapping_mul(core::mem::size_of::<WORD>() as DWORD),
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as LPVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ExtTextOut(
    mut hdc: HDC,
    mut x: INT,
    mut y: INT,
    mut flags: UINT,
    mut lprect: *const RECT,
    mut str: *const WCHAR,
    mut count: UINT,
    mut dx: *const INT,
) -> BOOL {
    let mut rect16: RECT16 = RECT16 {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut lpdx16: LPINT16 = ::core::ptr::null_mut::<INT16>();
    let mut ret: BOOL = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut ascii: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: DWORD = 0;
    let mut csi: CHARSETINFO = tagCHARSETINFO {
        ciCharset: 0,
        ciACP: 0,
        fs: tagFONTSIGNATURE {
            fsUsb: [0; 4],
            fsCsb: [0; 2],
        },
    };
    let mut charset: ::core::ffi::c_int = GetTextCharset(hdc);
    let mut cp: UINT = CP_ACP as UINT;
    if TranslateCharsetInfo(
        ULongToPtr!(charset as ::core::ffi::c_ulong) as PDWORD,
        &raw mut csi,
        TCI_SRCCHARSET as DWORD,
    ) != 0
    {
        cp = csi.ciACP;
    } else {
        match charset {
            __c if __c == OEM_CHARSET as ::core::ffi::c_int => {
                cp = GetOEMCP();
            }
            __c if __c == DEFAULT_CHARSET as ::core::ffi::c_int => {
                cp = GetACP();
            }
            __c if __c == VISCII_CHARSET as ::core::ffi::c_int
                || __c == TCVN_CHARSET as ::core::ffi::c_int
                || __c == KOI8_CHARSET as ::core::ffi::c_int
                || __c == ISO3_CHARSET as ::core::ffi::c_int
                || __c == ISO4_CHARSET as ::core::ffi::c_int
                || __c == ISO10_CHARSET as ::core::ffi::c_int
                || __c == CELTIC_CHARSET as ::core::ffi::c_int => {
                cp = CP_ACP as UINT;
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
                        b"win32ss/gdi/gdi32/wine/metadc.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"METADC_ExtTextOut\0" as *const u8
                            as *const ::core::ffi::c_char,
                        948 as ::core::ffi::c_int,
                        b"Can't find codepage for charset %d\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        charset,
                    );
                }
            }
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/gdi/gdi32/wine/metadc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"METADC_ExtTextOut\0" as *const u8 as *const ::core::ffi::c_char,
            954 as ::core::ffi::c_int,
            b"cp = %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            cp,
        );
    }
    len = WideCharToMultiByte(
        cp,
        0 as DWORD,
        str as LPCWSTR,
        count as ::core::ffi::c_int,
        ::core::ptr::null_mut::<CHAR>(),
        0 as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as DWORD;
    ascii = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        len as SIZE_T,
    ) as *mut ::core::ffi::c_char;
    WideCharToMultiByte(
        cp,
        0 as DWORD,
        str as LPCWSTR,
        count as ::core::ffi::c_int,
        ascii as LPSTR,
        len as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    );
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_1,
            __dbch_1,
            b"win32ss/gdi/gdi32/wine/metadc.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"METADC_ExtTextOut\0" as *const u8 as *const ::core::ffi::c_char,
            958 as ::core::ffi::c_int,
            b"mapped %s -> %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_wn(str, count as ::core::ffi::c_int),
            debugstr_an(ascii, len as ::core::ffi::c_int),
        );
    }
    if !lprect.is_null() {
        rect16.left = (*lprect).left as INT16;
        rect16.top = (*lprect).top as INT16;
        rect16.right = (*lprect).right as INT16;
        rect16.bottom = (*lprect).bottom as INT16;
    }
    if !dx.is_null() {
        lpdx16 = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (::core::mem::size_of::<INT16>() as SIZE_T).wrapping_mul(len as SIZE_T),
        ) as LPINT16;
        j = 0 as ::core::ffi::c_uint;
        i = j;
        while (i as DWORD) < len {
            if IsDBCSLeadByteEx(cp, *ascii.offset(i as isize) as BYTE) != 0 {
                let fresh39 = j;
                j = j.wrapping_add(1);
                let fresh40 = i;
                i = i.wrapping_add(1);
                *lpdx16.offset(fresh40 as isize) = *dx.offset(fresh39 as isize) as INT16;
                let fresh41 = i;
                i = i.wrapping_add(1);
                *lpdx16.offset(fresh41 as isize) = 0 as INT16;
            } else {
                let fresh42 = j;
                j = j.wrapping_add(1);
                let fresh43 = i;
                i = i.wrapping_add(1);
                *lpdx16.offset(fresh43 as isize) = *dx.offset(fresh42 as isize) as INT16;
            }
        }
    }
    ret = metadc_text(
        hdc,
        x as ::core::ffi::c_short,
        y as ::core::ffi::c_short,
        flags as UINT16,
        if !lprect.is_null() {
            &raw mut rect16
        } else {
            ::core::ptr::null_mut::<RECT16>()
        },
        ascii,
        len as ::core::ffi::c_short,
        lpdx16 as *const INT16,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        ascii as PVOID,
    );
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        lpdx16 as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ExtSelectClipRgn(
    mut hdc: HDC,
    mut hrgn: HRGN,
    mut mode: INT,
) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut::<metadc>();
    let mut rgn: INT16 = 0;
    let mut ret: INT = 0;
    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    if mode != (RGN_COPY as i32) {
        return (ERROR) as BOOL;
    }
    if hrgn.is_null() {
        return (NULLREGION) as BOOL;
    }
    rgn = metadc_create_region(metadc, hrgn);
    if rgn == -1 {
        return (ERROR) as BOOL;
    }
    ret = (if metadc_param1(
        hdc,
        META_SELECTOBJECT as ::core::ffi::c_short,
        rgn as ::core::ffi::c_short,
    ) != 0
    {
        NULLREGION
    } else {
        ERROR
    }) as INT;
    metadc_param1(
        hdc,
        META_DELETEOBJECT as ::core::ffi::c_short,
        rgn as ::core::ffi::c_short,
    );
    metadc_remove_handle(metadc, rgn as UINT);
    return ret as BOOL;
}

unsafe extern "C" fn metadc_find_object(metadc: *mut metadc, obj: HGDIOBJ) -> INT16 {
    let mut index: INT16;
    index = 0 as INT16;
    while (index as UINT) < (*metadc).handles_size {
        if *(*metadc).handles.offset(index as isize) == obj {
            return index;
        }
        index += 1;
    }
    return -(1 as INT16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_DeleteObject(hdc: HDC, obj: HGDIOBJ) {
    let metadc: *mut metadc = get_metadc_ptr(hdc);
    let mut mr: METARECORD = core::mem::zeroed();
    let mut index: INT16;

    index = metadc_find_object(metadc, obj);
    if index < 0 {
        return;
    }
    if obj == (*metadc).pen as HGDIOBJ
        || obj == (*metadc).brush as HGDIOBJ
        || obj == (*metadc).font as HGDIOBJ
    {
        return;
    }

    mr.rdSize = (core::mem::size_of::<METARECORD>() / core::mem::size_of::<WORD>()) as DWORD;
    mr.rdFunction = META_DELETEOBJECT as WORD;
    mr.rdParm[0] = index as WORD;

    metadc_write_record(
        metadc,
        core::ptr::addr_of_mut!(mr),
        (mr.rdSize as usize * core::mem::size_of::<WORD>()) as UINT,
    );

    *(*metadc).handles.offset(index as isize) = core::ptr::null_mut();
    (*metadc).cur_handles = (*metadc).cur_handles.wrapping_sub(1);
}

unsafe extern "C" fn metadc_select_object(hdc: HDC, index: INT16) -> BOOL {
    let mut mr: METARECORD = core::mem::zeroed();

    mr.rdSize = (core::mem::size_of::<METARECORD>() / 2) as DWORD;
    mr.rdFunction = META_SELECTOBJECT as WORD;
    mr.rdParm[0] = index as WORD;
    return metadc_record(
        hdc,
        core::ptr::addr_of_mut!(mr),
        ((mr.rdSize as usize) * core::mem::size_of::<WORD>()) as DWORD,
    );
}

unsafe extern "C" fn METADC_SelectBrush(mut hdc: HDC, mut hbrush: HBRUSH) -> HBRUSH {
    let mut metadc: *mut metadc = get_metadc_ptr(hdc);
    let mut index: INT16;
    let mut ret: HBRUSH;

    index = metadc_find_object(metadc, hbrush as HGDIOBJ);
    if index < 0 {
        index = metadc_create_brush(metadc, hbrush);
        if index < 0 {
            return core::ptr::null_mut();
        }
        GDI_hdc_using_object(hbrush as HGDIOBJ, hdc);
    }
    if metadc_select_object(hdc, index) == 0 {
        return core::ptr::null_mut();
    }
    ret = (*metadc).brush;
    (*metadc).brush = hbrush;
    return ret;
}

unsafe extern "C" fn metadc_create_font(
    mut metadc: *mut metadc,
    mut font: HFONT,
    mut logfont: *mut LOGFONTW,
) -> UINT16 {
    let mut buffer: [::core::ffi::c_uchar; core::mem::offset_of!(METARECORD, rdParm) + core::mem::size_of::<LOGFONT16>()] = [0; core::mem::offset_of!(METARECORD, rdParm) + core::mem::size_of::<LOGFONT16>()];
    let mut mr: *mut METARECORD = &raw mut buffer as *mut METARECORD;
    let mut font16: *mut LOGFONT16 = ::core::ptr::null_mut::<LOGFONT16>();
    let mut written: INT = 0;
    (*mr).rdSize = (::core::mem::size_of::<[::core::ffi::c_uchar; core::mem::offset_of!(METARECORD, rdParm) + core::mem::size_of::<LOGFONT16>()]>() as usize)
        .wrapping_div(::core::mem::size_of::<WORD>() as usize) as DWORD;
    (*mr).rdFunction = META_CREATEFONTINDIRECT as WORD;
    font16 = &raw mut (*mr).rdParm as *mut LOGFONT16;
    (*font16).lfHeight = (*logfont).lfHeight as INT16;
    (*font16).lfWidth = (*logfont).lfWidth as INT16;
    (*font16).lfEscapement = (*logfont).lfEscapement as INT16;
    (*font16).lfOrientation = (*logfont).lfOrientation as INT16;
    (*font16).lfWeight = (*logfont).lfWeight as INT16;
    (*font16).lfItalic = (*logfont).lfItalic;
    (*font16).lfUnderline = (*logfont).lfUnderline;
    (*font16).lfStrikeOut = (*logfont).lfStrikeOut;
    (*font16).lfCharSet = (*logfont).lfCharSet;
    (*font16).lfOutPrecision = (*logfont).lfOutPrecision;
    (*font16).lfClipPrecision = (*logfont).lfClipPrecision;
    (*font16).lfQuality = (*logfont).lfQuality;
    (*font16).lfPitchAndFamily = (*logfont).lfPitchAndFamily;
    written = WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        &raw mut (*logfont).lfFaceName as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        &raw mut (*font16).lfFaceName as LPSTR,
        (LF_FACESIZE - 1) as ::core::ffi::c_int,
        ::core::ptr::null::<CHAR>(),
        ::core::ptr::null_mut::<BOOL>(),
    ) as INT;
    /* Zero pad the facename buffer, so that we don't write uninitialized data to disk */
    memset(
        (&raw mut (*font16).lfFaceName as *mut CHAR).offset(written as isize)
            as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (LF_FACESIZE - (written) as u32) as size_t,
    );

    if metadc_write_record(
        metadc,
        mr,
        (*mr).rdSize.wrapping_mul(2 as DWORD) as ::core::ffi::c_uint,
    ) == 0
    {
        return 0 as UINT16;
    }
    return metadc_add_handle(metadc, font as HGDIOBJ) as UINT16;
}

unsafe extern "C" fn METADC_SelectFont(hdc: HDC, hfont: HFONT) -> HFONT {
    let metadc: *mut metadc = get_metadc_ptr(hdc);
    let mut font: LOGFONTW = core::mem::zeroed();
    let mut index: INT16;
    let mut ret: HFONT;

    index = metadc_find_object(metadc, hfont as HGDIOBJ);
    if index < 0 {
        if GetObjectW(hfont as HGDIOBJ, core::mem::size_of::<LOGFONTW>() as i32, &raw mut font as LPVOID) == 0 {
            return core::ptr::null_mut();
        }
        index = metadc_create_font(metadc, hfont, &raw mut font) as INT16;
        if index < 0 {
            return core::ptr::null_mut();
        }
        GDI_hdc_using_object(hfont as HGDIOBJ, hdc);
    }
    if metadc_select_object(hdc, index) == 0 {
        return core::ptr::null_mut();
    }
    ret = (*metadc).font;
    (*metadc).font = hfont;
    return ret;
}

unsafe extern "C" fn metadc_create_pen(metadc: *mut metadc, pen: HPEN, logpen: *mut LOGPEN16) -> UINT16 {
    let mut buffer = [0u8; (FIELD_OFFSET!(METARECORD, rdParm)
        + ((core::mem::size_of::<LOGPEN16>() / core::mem::size_of::<WORD>()) * RTL_FIELD_SIZE!(METARECORD, rdParm)) as i32) as usize];
    let mut mr: *mut METARECORD = core::ptr::addr_of_mut!(buffer) as *mut METARECORD;

    (*mr).rdSize = (core::mem::size_of_val(&buffer) / core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = (META_CREATEPENINDIRECT as u16);
    memcpy(
        core::ptr::addr_of_mut!((*mr).rdParm) as *mut ::core::ffi::c_void,
        logpen as *const ::core::ffi::c_void,
        core::mem::size_of::<LOGPEN16>(),
    );
    if metadc_write_record(
        metadc,
        mr,
        ((*mr).rdSize as usize * core::mem::size_of::<WORD>()) as UINT,
    ) == 0
    {
        return 0;
    }
    return metadc_add_handle(metadc, (pen).cast()) as UINT16;
}

unsafe extern "C" fn METADC_SelectPen(hdc: HDC, hpen: HPEN) -> HPEN {
    let metadc: *mut metadc = get_metadc_ptr(hdc);
    let mut logpen: LOGPEN16 = core::mem::zeroed();
    let mut index: INT16;
    let mut ret: HPEN;

    index = metadc_find_object(metadc, hpen as HGDIOBJ);
    if (index as i32) < 0 {
        /* must be an extended pen */
        let size: INT = GetObjectW(hpen as HANDLE, 0, core::ptr::null_mut()) as INT;

        if size == 0 {
            return core::ptr::null_mut();
        }

        if size as usize == core::mem::size_of::<LOGPEN>() {
            let mut pen: LOGPEN = core::mem::zeroed();

            GetObjectW(
                hpen as HANDLE,
                core::mem::size_of::<LOGPEN>() as INT,
                core::ptr::addr_of_mut!(pen) as LPVOID,
            );
            logpen.lopnStyle = pen.lopnStyle as UINT16;
            logpen.lopnWidth.x = pen.lopnWidth.x as INT16;
            logpen.lopnWidth.y = pen.lopnWidth.y as INT16;
            logpen.lopnColor = pen.lopnColor;
        } else {
            /* must be an extended pen */
            let elp: *mut EXTLOGPEN =
                HeapAlloc(GetProcessHeap(), 0, size as SIZE_T) as *mut EXTLOGPEN;

            GetObjectW(hpen as HANDLE, size as INT, elp as LPVOID);
            /* FIXME: add support for user style pens */
            logpen.lopnStyle = (*elp).elpPenStyle as UINT16;
            logpen.lopnWidth.x = (*elp).elpWidth as INT16;
            logpen.lopnWidth.y = 0;
            logpen.lopnColor = (*elp).elpColor;

            HeapFree(GetProcessHeap(), 0, elp as PVOID);
        }

        index = metadc_create_pen(metadc, hpen, core::ptr::addr_of_mut!(logpen)) as INT16;
        if (index as i32) < 0 {
            return core::ptr::null_mut();
        }
        GDI_hdc_using_object(hpen as HGDIOBJ, hdc);
    }

    if metadc_select_object(hdc, index) == 0 {
        return core::ptr::null_mut();
    }
    ret = (*metadc).pen;
    (*metadc).pen = hpen;
    return ret;
}

unsafe extern "C" fn metadc_create_palette(
    mut metadc: *mut metadc,
    mut palette: HPALETTE,
    mut log_palette: *mut LOGPALETTE,
    mut sizeofPalette: ::core::ffi::c_int,
) -> BOOL {
    let mut index: ::core::ffi::c_int = 0;
    let mut ret: BOOL = 0;
    let mut mr: *mut METARECORD = ::core::ptr::null_mut::<METARECORD>();

    mr = HeapAlloc(
        GetProcessHeap(),
        0,
        (::core::mem::size_of::<METARECORD>() as SIZE_T)
            .wrapping_add(sizeofPalette as SIZE_T)
            .wrapping_sub(::core::mem::size_of::<WORD>() as SIZE_T),
    ) as *mut METARECORD;
    if mr.is_null() {
        return (FALSE) as BOOL;
    }
    (*mr).rdSize = ((::core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add(sizeofPalette as usize)
        .wrapping_sub(::core::mem::size_of::<WORD>() as usize)
        .wrapping_div(::core::mem::size_of::<WORD>() as usize)) as DWORD;
    (*mr).rdFunction = META_CREATEPALETTE as WORD;
    memcpy(
        core::ptr::addr_of_mut!((*mr).rdParm) as *mut ::core::ffi::c_void,
        log_palette as *const ::core::ffi::c_void,
        sizeofPalette as usize,
    );
    if metadc_write_record(
        metadc,
        mr,
        ((*mr).rdSize as usize).wrapping_mul(::core::mem::size_of::<WORD>() as usize)
            as ::core::ffi::c_uint,
    ) == 0
    {
        HeapFree(GetProcessHeap(), 0, mr as PVOID);
        return (FALSE) as BOOL;
    }

    (*mr).rdSize = ((::core::mem::size_of::<METARECORD>() as usize)
        .wrapping_div(::core::mem::size_of::<WORD>() as usize)) as DWORD;
    (*mr).rdFunction = META_SELECTPALETTE as WORD;

    index = metadc_add_handle(metadc, palette as HGDIOBJ) as ::core::ffi::c_int;
    if index == -(1 as ::core::ffi::c_int) {
        ret = (FALSE) as i32;
    } else {
        *(core::ptr::addr_of_mut!((*mr).rdParm) as *mut WORD)
            .offset(0 as isize) = index as WORD;
        ret = metadc_write_record(
            metadc,
            mr,
            ((*mr).rdSize as usize)
                .wrapping_mul(::core::mem::size_of::<WORD>() as usize)
                as ::core::ffi::c_uint,
        );
    }
    HeapFree(GetProcessHeap(), 0, mr as PVOID);
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SelectPalette(hdc: HDC, palette: HPALETTE) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut log_palette: PLOGPALETTE = core::ptr::null_mut();
    let mut count: WORD = 0;
    let mut ret: BOOL = 0;
    let mut size: i32 = 0;

    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    GetObjectA(
        palette as HANDLE,
        core::mem::size_of::<WORD>() as i32,
        &raw mut count as LPVOID,
    );
    if count == 0 {
        return 0;
    }

    size = (core::mem::size_of::<LOGPALETTE>() as usize).wrapping_add(
        ((count as i32 - 1) as usize)
            .wrapping_mul(core::mem::size_of::<PALETTEENTRY>()),
    ) as i32;
    log_palette = HeapAlloc(
        GetProcessHeap(),
        0,
        size as SIZE_T,
    ) as PLOGPALETTE;
    if log_palette.is_null() {
        return 0;
    }

    (*log_palette).palVersion = 0x300;
    (*log_palette).palNumEntries = count;

    GetPaletteEntries(
        palette,
        0,
        count as UINT,
        &raw mut (*log_palette).palPalEntry as LPPALETTEENTRY,
    );

    ret = metadc_create_palette(metadc, palette, log_palette as *mut LOGPALETTE, size);

    HeapFree(
        GetProcessHeap(),
        0,
        log_palette as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_RealizePalette(hdc: HDC) -> BOOL {
    return metadc_param0(hdc, META_REALIZEPALETTE as i16);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ {
    match GDI_HANDLE_GET_TYPE!(obj) {
        1048576 /* GDILoObjType_LO_BRUSH_TYPE */ => {
            return METADC_SelectBrush(hdc, obj as HBRUSH) as HGDIOBJ;
        }
        655360 /* GDILoObjType_LO_FONT_TYPE */ => {
            return METADC_SelectFont(hdc, obj as HFONT) as HGDIOBJ;
        }
        3145728 /* GDILoObjType_LO_PEN_TYPE */ | 5242880 /* GDILoObjType_LO_EXTPEN_TYPE */ => {
            return METADC_SelectPen(hdc, obj as HPEN) as HGDIOBJ;
        }
        _ => {
            SetLastError(ERROR_INVALID_FUNCTION as DWORD);
            return core::ptr::null_mut();
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn METADC_ExtEscape(
    mut hdc: HDC,
    mut escape: INT,
    mut input_size: INT,
    mut input: LPCSTR,
    mut output_size: INT,
    mut output: LPVOID,
) -> BOOL {
    let mut mr: *mut METARECORD = core::ptr::null_mut::<METARECORD>();
    let mut len: DWORD = 0;
    let mut ret: BOOL = 0;
    if output_size != 0 {
        return (FALSE) as BOOL;
    }
    len = (core::mem::size_of::<METARECORD>() as usize)
        .wrapping_add(core::mem::size_of::<WORD>() as usize)
        .wrapping_add(
            ((input_size + 1) & !1) as usize,
        ) as DWORD;
    mr = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        len as SIZE_T,
    ) as *mut METARECORD;
    if mr.is_null() {
        return (FALSE) as BOOL;
    }
    (*mr).rdSize = (len as usize).wrapping_div(core::mem::size_of::<WORD>()) as DWORD;
    (*mr).rdFunction = META_ESCAPE as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(0 as isize) = escape as WORD;
    *(&raw mut (*mr).rdParm as *mut WORD).offset(1 as isize) = input_size as WORD;
    memcpy(
        (&raw mut (*mr).rdParm as *mut WORD).offset(2 as isize) as *mut WORD
            as *mut core::ffi::c_void,
        input as *const core::ffi::c_void,
        input_size as usize,
    );
    ret = metadc_record(hdc, mr, len);
    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        mr as PVOID,
    );
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn METADC_GetDeviceCaps(hdc: HDC, cap: INT) -> INT {
    if get_metadc_ptr(hdc).is_null() {
        return 0;
    }

    match (cap) as u32 {
        TECHNOLOGY => {
            return (DT_METAFILE) as INT;
        }
        TEXTCAPS => {
            return 0;
        }
        _ => {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
                != 0
            {
                let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
                ros_dbg_log(
                    __dbcl,
                    __dbch,
                    b"win32ss/gdi/gdi32/wine/metadc.c\0"
                        as *const u8 as *const ::core::ffi::c_char,
                    b"METADC_GetDeviceCaps\0" as *const u8 as *const ::core::ffi::c_char,
                    1300 as ::core::ffi::c_int,
                    b" unsupported capability %d, will return 0\n\0" as *const u8
                        as *const ::core::ffi::c_char,
                    cap,
                );
            }
        }
    }

    return 0;
}

unsafe extern "C" fn metadc_free(metadc: *mut metadc) {
    let mut index: DWORD;

    CloseHandle((*metadc).hFile);
    HeapFree(GetProcessHeap(), 0, (*metadc).mh as LPVOID);
    index = 0;
    while index < (*metadc).handles_size as DWORD {
        if !(*(*metadc).handles.offset(index as isize)).is_null() {
            GDI_hdc_not_using_object(*(*metadc).handles.offset(index as isize), (*metadc).hdc);
        }
        index = index.wrapping_add(1);
    }
    HeapFree(GetProcessHeap(), 0, (*metadc).handles as LPVOID);
    HeapFree(GetProcessHeap(), 0, metadc as LPVOID);
}
#[no_mangle]
pub unsafe extern "C" fn METADC_DeleteDC(mut hdc: HDC) -> BOOL {
    let mut metadc: *mut metadc = core::ptr::null_mut::<metadc>();
    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return (FALSE) as BOOL;
    }
    if GdiDeleteClientObj(hdc as HGDIOBJ).is_null() {
        return (FALSE) as BOOL;
    }
    metadc_free(metadc);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CreateMetaFileW(filename: *const WCHAR) -> HDC {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut hdc: HANDLE = core::ptr::null_mut();
    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"win32ss/gdi/gdi32/wine/metadc.c\0"
                as *const u8 as *const i8,
            b"CreateMetaFileW\0" as *const u8 as *const i8,
            1339 as i32,
            b"%s\n\0" as *const u8 as *const i8,
            debugstr_w(filename),
        );
    }
    metadc = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<metadc>() as SIZE_T,
    ) as *mut metadc;
    if metadc.is_null() {
        return core::ptr::null_mut();
    }
    (*metadc).mh = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<METAHEADER>() as SIZE_T,
    ) as *mut METAHEADER;
    if (*metadc).mh.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            metadc as PVOID,
        );
        return core::ptr::null_mut();
    }

    hdc = GdiCreateClientObj(metadc as PVOID, GDILoObjType_LO_METADC16_TYPE) as HANDLE;
    if hdc.is_null() {
        HeapFree(
            GetProcessHeap(),
            0 as DWORD,
            metadc as PVOID,
        );
        return core::ptr::null_mut();
    }

    (*metadc).hdc = hdc as HDC;

    (*metadc).handles = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY as DWORD,
        (HANDLE_LIST_INC as SIZE_T).wrapping_mul(core::mem::size_of::<HGDIOBJ>() as SIZE_T),
    ) as *mut HGDIOBJ;
    let _handles_idx: usize = 0 as usize;
    (*metadc).handles_size = HANDLE_LIST_INC as UINT;
    (*metadc).cur_handles = 0 as UINT;

    (*metadc).hFile = 0 as HANDLE;

    (*(*metadc).mh).mtHeaderSize = (core::mem::size_of::<METAHEADER>() as usize)
        .wrapping_div(core::mem::size_of::<WORD>() as usize)
        as WORD;
    (*(*metadc).mh).mtVersion = 0x0300 as WORD;
    (*(*metadc).mh).mtSize = (*(*metadc).mh).mtHeaderSize as DWORD;
    (*(*metadc).mh).mtNoObjects = 0 as WORD;
    (*(*metadc).mh).mtMaxRecord = 0 as DWORD;
    (*(*metadc).mh).mtNoParameters = 0 as WORD;
    (*(*metadc).mh).mtType = METAFILE_MEMORY as WORD;

    (*metadc).pen = GetStockObject(BLACK_PEN as i32) as HPEN;
    (*metadc).brush = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;
    (*metadc).font = GetStockObject(DEVICE_DEFAULT_FONT as i32) as HFONT;

    SetVirtualResolution(
        hdc as HDC,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as DWORD,
    );

    if !filename.is_null() {
        let mut file: HANDLE = CreateFileW(
            filename as LPCWSTR,
            GENERIC_WRITE as DWORD,
            0 as DWORD,
            core::ptr::null_mut(),
            CREATE_ALWAYS as DWORD,
            0 as DWORD,
            0 as HANDLE,
        );
        if file == INVALID_HANDLE_VALUE as HANDLE {
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                metadc as PVOID,
            );
            GdiDeleteClientObj(hdc as HGDIOBJ);
            return 0 as HDC;
        }
        (*metadc).hFile = file;
    }

    if (*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"win32ss/gdi/gdi32/wine/metadc.c\0"
                as *const u8 as *const i8,
            b"CreateMetaFileW\0" as *const u8 as *const i8,
            1393 as i32,
            b"returning %p\n\0" as *const u8 as *const i8,
            hdc,
        );
    }
    return hdc as HDC;
}

#[no_mangle]
pub unsafe extern "system" fn CreateMetaFileA(filename: *const i8) -> HDC {
    let mut filenameW: LPWSTR = core::ptr::null_mut();
    let mut len: DWORD = 0;
    let mut hdc: HDC = core::ptr::null_mut();
    if filename.is_null() {
        return CreateMetaFileW(core::ptr::null());
    }
    len = MultiByteToWideChar(
        CP_ACP,
        0,
        filename as LPCSTR,
        -1,
        core::ptr::null_mut(),
        0,
    ) as DWORD;
    filenameW = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        filename as LPCSTR,
        -1,
        filenameW,
        len as i32,
    );

    hdc = CreateMetaFileW(filenameW as *const WCHAR);

    HeapFree(
        GetProcessHeap(),
        0,
        filenameW as PVOID,
    );
    return hdc;
}

#[no_mangle]
pub unsafe extern "system" fn CloseMetaFile(mut hdc: HDC) -> HMETAFILE {
    let mut metadc: *mut metadc = core::ptr::null_mut();
    let mut bytes_written: DWORD = 0;
    let mut hmf: HMETAFILE = core::ptr::null_mut();

    metadc = get_metadc_ptr(hdc);
    if metadc.is_null() {
        return core::ptr::null_mut(); // C: `return FALSE;` from an HMETAFILE fn
    }

    /* Construct the end of metafile record - this is documented
     * in SDK Knowledgebase Q99334.
     */
    if metadc_param0(hdc, (META_EOF) as INT16) == 0 {
        return core::ptr::null_mut(); // C: `return FALSE;` from an HMETAFILE fn
    }
    if GdiDeleteClientObj(hdc as HGDIOBJ) == core::ptr::null_mut() {
        return core::ptr::null_mut(); // C: `return FALSE;` from an HMETAFILE fn
    }

    if !(*metadc).hFile.is_null()
        && WriteFile(
            (*metadc).hFile,
            (*metadc).mh as LPCVOID,
            ((*(*metadc).mh).mtSize as DWORD)
                .wrapping_mul(core::mem::size_of::<WORD>() as DWORD),
            core::ptr::addr_of_mut!(bytes_written),
            core::ptr::null_mut(),
        ) == 0
    {
        metadc_free(metadc);
        return core::ptr::null_mut(); // C: `return FALSE;` from an HMETAFILE fn
    }

    /* Now allocate a global handle for the metafile */
    hmf = MF_Create_HMETAFILE((*metadc).mh);
    if !hmf.is_null() {
        (*metadc).mh = core::ptr::null_mut(); /* So it won't be deleted */
    }
    metadc_free(metadc);
    return hmf;
}
