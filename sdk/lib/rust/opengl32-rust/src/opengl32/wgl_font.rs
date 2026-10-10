use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn GetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC;
}

// ---- file-scope preamble (translated) ----
// The C declares every pglu* pointer `WINAPI` (stdcall on i386, where a cdecl call through
// it would leave the callee-popped arguments popped twice); c2rust dropped the convention.
// @c2rust file-scope data (compiler-exact tables/statics)
static mut pgluNewTess: Option<unsafe extern "system" fn() -> *mut GLUtesselator> = None;

static mut pgluDeleteTess: Option<unsafe extern "system" fn(*mut GLUtesselator) -> ()> = None;

static mut pgluTessNormal: Option<
    unsafe extern "system" fn(*mut GLUtesselator, GLdouble, GLdouble, GLdouble) -> (),
> = None;

static mut pgluTessBeginPolygon: Option<
    unsafe extern "system" fn(*mut GLUtesselator, *mut ::core::ffi::c_void) -> (),
> = None;

static mut pgluTessEndPolygon: Option<unsafe extern "system" fn(*mut GLUtesselator) -> ()> = None;

static mut pgluTessCallback: Option<
    unsafe extern "system" fn(*mut GLUtesselator, GLenum, _GLUfuncptr) -> (),
> = None;

static mut pgluTessBeginContour: Option<unsafe extern "system" fn(*mut GLUtesselator) -> ()> = None;

static mut pgluTessEndContour: Option<unsafe extern "system" fn(*mut GLUtesselator) -> ()> = None;

static mut pgluTessVertex: Option<
    unsafe extern "system" fn(*mut GLUtesselator, *mut GLdouble, *mut ::core::ffi::c_void) -> (),
> = None;

// WINE_DEFAULT_DEBUG_CHANNEL(wgl): this TU's default channel is `wgl`, not the crate-wide
// `opengl32` the prelude's `__wine_dbch___default` names. The local item shadows the glob import.
static mut __wine_dbch___default: *mut __wine_debug_channel = unsafe { &raw mut __wine_dbch_wgl };

#[repr(C)]
pub struct GLUtesselator {
    _private: [u8; 0],
}

pub type _GLUfuncptr = unsafe extern "system" fn();

pub const GLU_TESS_BEGIN: u32 = 100100;

pub const GLU_TESS_VERTEX: u32 = 100101;

pub const GLU_TESS_END: u32 = 100102;

#[repr(C)]
pub struct bezier_vector {
    pub x: GLdouble,
    pub y: GLdouble,
}

unsafe extern "C" fn wglUseFontBitmaps_common(hdc: HDC, first: DWORD, count: DWORD, mut listBase: DWORD, unicode: BOOL) -> BOOL {
    let funcs: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();
    let mut gm: GLYPHMETRICS = core::mem::zeroed();
    let mut glyph: core::ffi::c_uint;
    let mut size: core::ffi::c_uint = 0;
    let mut bitmap: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut gl_bitmap: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut org_alignment: core::ffi::c_int = 0;
    let mut ret: BOOL = (TRUE) as i32;

    (*funcs).GetIntegerv.unwrap_unchecked()(
        GL_UNPACK_ALIGNMENT as GLenum,
        core::ptr::addr_of_mut!(org_alignment),
    );
    (*funcs).PixelStorei.unwrap_unchecked()(
        GL_UNPACK_ALIGNMENT as GLenum,
        4 as GLint,
    );

    glyph = first as core::ffi::c_uint;
    while (glyph as DWORD) < first.wrapping_add(count) {
        static identity: MAT2 = MAT2 {
            eM11: FIXED {
                fract: 0 as WORD,
                value: 1 as core::ffi::c_short,
            },
            eM12: FIXED {
                fract: 0 as WORD,
                value: 0 as core::ffi::c_short,
            },
            eM21: FIXED {
                fract: 0 as WORD,
                value: 0 as core::ffi::c_short,
            },
            eM22: FIXED {
                fract: 0 as WORD,
                value: 1 as core::ffi::c_short,
            },
        };
        let mut needed_size: core::ffi::c_uint;
        let mut height: core::ffi::c_uint;
        let mut width: core::ffi::c_uint;
        let mut width_int: core::ffi::c_uint;

        if unicode != 0 {
            needed_size = GetGlyphOutlineW(
                hdc,
                glyph as UINT,
                GGO_BITMAP as UINT,
                core::ptr::addr_of_mut!(gm),
                0 as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of!(identity),
            ) as core::ffi::c_uint;
        } else {
            needed_size = GetGlyphOutlineA(
                hdc,
                glyph as UINT,
                GGO_BITMAP as UINT,
                core::ptr::addr_of_mut!(gm),
                0 as DWORD,
                core::ptr::null_mut(),
                core::ptr::addr_of!(identity),
            ) as core::ffi::c_uint;
        }

        TRACE!("Glyph: %3d / List: %d size %d\n", glyph, listBase, needed_size);
        if needed_size == GDI_ERROR as core::ffi::c_uint {
            ret = FALSE as BOOL;
            break;
        }

        if needed_size > size {
            size = needed_size;
            HeapFree(GetProcessHeap(), 0 as DWORD, bitmap);
            HeapFree(GetProcessHeap(), 0 as DWORD, gl_bitmap);
            bitmap = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                size as SIZE_T,
            ) as *mut core::ffi::c_void;
            gl_bitmap = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY as DWORD,
                size as SIZE_T,
            ) as *mut core::ffi::c_void;
        }
        if unicode != 0 {
            ret = (GetGlyphOutlineW(
                hdc,
                glyph as UINT,
                GGO_BITMAP as UINT,
                core::ptr::addr_of_mut!(gm),
                size as DWORD,
                bitmap as LPVOID,
                core::ptr::addr_of!(identity),
            ) != GDI_ERROR as DWORD) as core::ffi::c_int as BOOL;
        } else {
            ret = (GetGlyphOutlineA(
                hdc,
                glyph as UINT,
                GGO_BITMAP as UINT,
                core::ptr::addr_of_mut!(gm),
                size as DWORD,
                bitmap as LPVOID,
                core::ptr::addr_of!(identity),
            ) != GDI_ERROR as DWORD) as core::ffi::c_int as BOOL;
        }
        if ret == 0 {
            break;
        }

        if TRACE_ON!(wgl) {
            let mut bitmask: core::ffi::c_uint;
            let mut bitmap_: *mut core::ffi::c_uchar = bitmap as *mut core::ffi::c_uchar;

            TRACE!("  - bbox: %d x %d\n", gm.gmBlackBoxX, gm.gmBlackBoxY);
            TRACE!("  - origin: (%d, %d)\n", gm.gmptGlyphOrigin.x, gm.gmptGlyphOrigin.y);
            TRACE!("  - increment: %d - %d\n", gm.gmCellIncX as c_int, gm.gmCellIncY as c_int);
            if needed_size != 0 as core::ffi::c_uint {
                TRACE!("  - bitmap:\n");
                height = 0 as core::ffi::c_uint;
                while height < gm.gmBlackBoxY {
                    TRACE!("      ");
                    width = 0 as core::ffi::c_uint;
                    bitmask = 0x80 as core::ffi::c_uint;
                    while width < gm.gmBlackBoxX {
                        if bitmask == 0 as core::ffi::c_uint {
                            bitmap_ = bitmap_.offset(1 as isize);
                            bitmask = 0x80 as core::ffi::c_uint;
                        }
                        if ((*bitmap_ as core::ffi::c_uint) & bitmask) != 0 {
                            TRACE!("*");
                        } else {
                            TRACE!(" ");
                        }
                        width = width.wrapping_add(1);
                        bitmask >>= 1;
                    }
                    bitmap_ = bitmap_.offset(
                        ((4 as UINT_PTR)
                            .wrapping_sub((bitmap_ as UINT_PTR) & (0x03 as UINT_PTR)))
                            as isize,
                    );
                    TRACE!("\n");
                    height = height.wrapping_add(1);
                }
            }
        }

        /* In OpenGL, the bitmap is drawn from the bottom to the top... So we need to invert the
         * glyph for it to be drawn properly.
         */
        if needed_size != 0 as core::ffi::c_uint {
            width_int = gm
                .gmBlackBoxX
                .wrapping_add(31 as UINT)
                .wrapping_div(32 as UINT) as core::ffi::c_uint;
            height = 0 as core::ffi::c_uint;
            while height < gm.gmBlackBoxY {
                width = 0 as core::ffi::c_uint;
                while width < width_int {
                    *(gl_bitmap as *mut core::ffi::c_int).offset(
                        ((gm.gmBlackBoxY as core::ffi::c_uint)
                            .wrapping_sub(height)
                            .wrapping_sub(1 as core::ffi::c_uint))
                            .wrapping_mul(width_int)
                            .wrapping_add(width) as isize,
                    ) = *(bitmap as *mut core::ffi::c_int)
                        .offset(height.wrapping_mul(width_int).wrapping_add(width) as isize);
                    width = width.wrapping_add(1);
                }
                height = height.wrapping_add(1);
            }
        }

        let fresh_list = listBase;
        listBase = listBase.wrapping_add(1);
        (*funcs).NewList.unwrap_unchecked()(
            fresh_list as GLuint,
            GL_COMPILE as GLenum,
        );
        if needed_size != 0 as core::ffi::c_uint {
            (*funcs).Bitmap.unwrap_unchecked()(
                gm.gmBlackBoxX as GLsizei,
                gm.gmBlackBoxY as GLsizei,
                (0 - gm.gmptGlyphOrigin.x) as GLfloat,
                (gm.gmBlackBoxY as core::ffi::c_int - gm.gmptGlyphOrigin.y) as GLfloat,
                gm.gmCellIncX as GLfloat,
                gm.gmCellIncY as GLfloat,
                gl_bitmap as *const GLubyte,
            );
        } else {
            /* This is the case of 'empty' glyphs like the space character */
            (*funcs).Bitmap.unwrap_unchecked()(
                0 as GLsizei,
                0 as GLsizei,
                0 as GLfloat,
                0 as GLfloat,
                gm.gmCellIncX as GLfloat,
                gm.gmCellIncY as GLfloat,
                core::ptr::null::<GLubyte>(),
            );
        }
        (*funcs).EndList.unwrap_unchecked()();
        glyph = glyph.wrapping_add(1);
    }

    (*funcs).PixelStorei.unwrap_unchecked()(
        GL_UNPACK_ALIGNMENT as GLenum,
        org_alignment as GLint,
    );
    HeapFree(GetProcessHeap(), 0 as DWORD, bitmap);
    HeapFree(GetProcessHeap(), 0 as DWORD, gl_bitmap);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn wglUseFontBitmapsA(hdc: HDC, first: DWORD, count: DWORD, listBase: DWORD) -> BOOL {
    return wglUseFontBitmaps_common(hdc, first, count, listBase, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn wglUseFontBitmapsW(hdc: HDC, first: DWORD, count: DWORD, listBase: DWORD) -> BOOL {
    return wglUseFontBitmaps_common(hdc, first, count, listBase, (TRUE) as BOOL);
}

unsafe extern "C" fn load_libglu() -> HMODULE {
    static mut glu32W: [WCHAR; 10] = [0x0067u16, 0x006C, 0x0075, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
    static mut already_loaded: i32 = 0;
    static mut module: HMODULE = core::ptr::null_mut();

    if already_loaded != 0 {
        return module;
    }
    already_loaded = 1;

    TRACE!("Trying to load GLU library\n");
    module = LoadLibraryW(core::ptr::addr_of!(glu32W) as LPCWSTR) as HMODULE;
    if module.is_null() {
        WARN!("Failed to load glu32\n");
        return core::ptr::null_mut();
    }
    pgluNewTess = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn() -> *mut GLUtesselator>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluNewTess\0".as_ptr() as LPCSTR),
    ));
    pgluDeleteTess = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluDeleteTess\0".as_ptr() as LPCSTR),
    ));
    pgluTessBeginContour = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessBeginContour\0".as_ptr() as LPCSTR),
    ));
    pgluTessNormal = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator, GLdouble, GLdouble, GLdouble)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessNormal\0".as_ptr() as LPCSTR),
    ));
    pgluTessBeginPolygon = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator, *mut core::ffi::c_void)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessBeginPolygon\0".as_ptr() as LPCSTR),
    ));
    pgluTessCallback = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator, GLenum, _GLUfuncptr)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessCallback\0".as_ptr() as LPCSTR),
    ));
    pgluTessEndContour = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessEndContour\0".as_ptr() as LPCSTR),
    ));
    pgluTessEndPolygon = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessEndPolygon\0".as_ptr() as LPCSTR),
    ));
    pgluTessVertex = core::mem::transmute::<
        *mut core::ffi::c_void,
        Option<unsafe extern "system" fn(*mut GLUtesselator, *mut GLdouble, *mut core::ffi::c_void)>,
    >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
        GetProcAddress(module as HINSTANCE, b"gluTessVertex\0".as_ptr() as LPCSTR),
    ));
    return module;
}

unsafe extern "C" fn fixed_to_double(fixed: POINTFX, em_size: UINT, vertex: *mut GLdouble) {
    *vertex.offset(0 as isize) = (fixed.x.value as GLdouble + fixed.x.fract as GLdouble / ((1 << 16) as GLdouble)) / em_size as GLdouble;
    *vertex.offset(1 as isize) = (fixed.y.value as GLdouble + fixed.y.fract as GLdouble / ((1 << 16) as GLdouble)) / em_size as GLdouble;
    *vertex.offset(2 as isize) = 0.0 as GLdouble;
}
unsafe extern "system" fn tess_callback_vertex(vertex: *mut core::ffi::c_void) {
    let funcs: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();
    let dbl: *mut GLdouble = vertex as *mut GLdouble;
    TRACE!("%f, %f, %f\n", *dbl.add(0), *dbl.add(1), *dbl.add(2));
    (*funcs).Vertex3dv.unwrap_unchecked()(vertex as *const GLdouble);
}
unsafe extern "system" fn tess_callback_begin(which: GLenum) {
    let funcs: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();
    TRACE!("%d\n", which);
    (*funcs).Begin.unwrap_unchecked()(which);
}

unsafe extern "system" fn tess_callback_end() {
    let funcs: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();
    TRACE!("\n");
    (*funcs).End.unwrap_unchecked()();
}

unsafe extern "C" fn bezier_deviation_squared(p: *const bezier_vector) -> f64 {
    let mut deviation: bezier_vector = bezier_vector { x: 0.0, y: 0.0 };
    let mut vertex: bezier_vector = bezier_vector { x: 0.0, y: 0.0 };
    let mut base: bezier_vector = bezier_vector { x: 0.0, y: 0.0 };
    let mut base_length: f64 = 0.0;
    let mut dot: f64 = 0.0;

    vertex.x = ((*p.add(0)).x + (*p.add(1)).x * 2 as f64 + (*p.add(2)).x) / 4 as f64 - (*p.add(0)).x;
    vertex.y = ((*p.add(0)).y + (*p.add(1)).y * 2 as f64 + (*p.add(2)).y) / 4 as f64 - (*p.add(0)).y;

    base.x = (*p.add(2)).x - (*p.add(0)).x;
    base.y = (*p.add(2)).y - (*p.add(0)).y;

    base_length = sqrt(base.x * base.x + base.y * base.y);
    base.x /= base_length;
    base.y /= base_length;

    dot = base.x * vertex.x + base.y * vertex.y;
    dot = min(max(dot, 0.0), base_length);
    base.x *= dot;
    base.y *= dot;

    deviation.x = vertex.x - base.x;
    deviation.y = vertex.y - base.y;

    return deviation.x * deviation.x + deviation.y * deviation.y;
}

unsafe extern "C" fn bezier_approximate(
    p: *const bezier_vector,
    mut points: *mut bezier_vector,
    deviation: FLOAT,
) -> ::core::ffi::c_int {
    let mut first_curve: [bezier_vector; 3] = [
        bezier_vector { x: 0., y: 0. },
        bezier_vector { x: 0., y: 0. },
        bezier_vector { x: 0., y: 0. },
    ];
    let mut second_curve: [bezier_vector; 3] = [
        bezier_vector { x: 0., y: 0. },
        bezier_vector { x: 0., y: 0. },
        bezier_vector { x: 0., y: 0. },
    ];
    let mut vertex: bezier_vector = bezier_vector { x: 0., y: 0. };
    let mut total_vertices: ::core::ffi::c_int = 0;

    if bezier_deviation_squared(p) <= (deviation * deviation) as ::core::ffi::c_double {
        if !points.is_null() {
            *points = core::ptr::read(p.offset(2 as isize));
        }
        return 1 as ::core::ffi::c_int;
    }

    vertex.x = ((*p.offset(0 as isize)).x
        + (*p.offset(1 as isize)).x * 2 as GLdouble
        + (*p.offset(2 as isize)).x)
        / 4 as GLdouble;
    vertex.y = ((*p.offset(0 as isize)).y
        + (*p.offset(1 as isize)).y * 2 as GLdouble
        + (*p.offset(2 as isize)).y)
        / 4 as GLdouble;

    first_curve[0] = core::ptr::read(p.offset(0 as isize));
    first_curve[1].x = ((*p.offset(0 as isize)).x
        + (*p.offset(1 as isize)).x)
        / 2 as GLdouble;
    first_curve[1].y = ((*p.offset(0 as isize)).y
        + (*p.offset(1 as isize)).y)
        / 2 as GLdouble;
    first_curve[2] = core::ptr::read(&vertex);

    second_curve[0] = vertex;
    second_curve[1].x = ((*p.offset(2 as isize)).x
        + (*p.offset(1 as isize)).x)
        / 2 as GLdouble;
    second_curve[1].y = ((*p.offset(2 as isize)).y
        + (*p.offset(1 as isize)).y)
        / 2 as GLdouble;
    second_curve[2] = core::ptr::read(p.offset(2 as isize));

    total_vertices = bezier_approximate(
        core::ptr::addr_of_mut!(first_curve) as *mut bezier_vector,
        points,
        deviation,
    );
    if !points.is_null() {
        points = points.offset(total_vertices as isize);
    }
    total_vertices += bezier_approximate(
        core::ptr::addr_of_mut!(second_curve) as *mut bezier_vector,
        points,
        deviation,
    );
    return total_vertices;
}

unsafe extern "C" fn wglUseFontOutlines_common(
    mut hdc: HDC,
    mut first: DWORD,
    mut count: DWORD,
    mut listBase: DWORD,
    mut deviation: FLOAT,
    mut extrusion: FLOAT,
    mut format: ::core::ffi::c_int,
    mut lpgmf: LPGLYPHMETRICSFLOAT,
    mut unicode: BOOL,
) -> BOOL {
    let funcs: *const GLDISPATCHTABLE = IntGetCurrentDispatchTable();
    let mut glyph: UINT = 0;
    let identity: MAT2 = MAT2 {
        eM11: FIXED { fract: 0, value: 1 },
        eM12: FIXED { fract: 0, value: 0 },
        eM21: FIXED { fract: 0, value: 0 },
        eM22: FIXED { fract: 0, value: 1 },
    };
    let mut tess: *mut GLUtesselator = ::core::ptr::null_mut::<GLUtesselator>();
    let mut lf: LOGFONTW = core::mem::zeroed();
    let mut old_font: HFONT = ::core::ptr::null_mut();
    let mut unscaled_font: HFONT = ::core::ptr::null_mut();
    let mut em_size: UINT = 1024;
    let mut rc: RECT = core::mem::zeroed();

    TRACE!("(%p, %d, %d, %d, %f, %f, %d, %p, %s)\n", hdc, first, count,
          listBase, deviation as f64, extrusion as f64, format, lpgmf,
          if unicode != 0 { b"W\0".as_ptr() } else { b"A\0".as_ptr() } as *const c_char);

    if deviation <= 0.0 {
        deviation = (1.0 / em_size as FLOAT) as FLOAT;
    }

    if format == (WGL_FONT_POLYGONS as i32) {
        if load_libglu().is_null() {
            ERR!("glu32 is required for this function but isn't available\n");
            return (FALSE) as BOOL;
        }

        tess = (pgluNewTess.unwrap_unchecked())();
        if tess.is_null() {
            return (FALSE) as BOOL;
        }
        (pgluTessCallback.unwrap_unchecked())(
            tess,
            GLU_TESS_VERTEX as GLenum,
            ::core::mem::transmute::<
                Option<unsafe extern "system" fn(*mut ::core::ffi::c_void) -> ()>,
                _GLUfuncptr,
            >(Some(tess_callback_vertex)),
        );
        (pgluTessCallback.unwrap_unchecked())(
            tess,
            GLU_TESS_BEGIN as GLenum,
            ::core::mem::transmute::<Option<unsafe extern "system" fn(GLenum) -> ()>, _GLUfuncptr>(
                Some(tess_callback_begin),
            ),
        );
        (pgluTessCallback.unwrap_unchecked())(
            tess,
            GLU_TESS_END as GLenum,
            tess_callback_end,
        );
    }

    GetObjectW(
        GetCurrentObject(hdc, OBJ_FONT as UINT) as HANDLE,
        ::core::mem::size_of::<LOGFONTW>() as ::core::ffi::c_int,
        &raw mut lf as LPVOID,
    );
    rc.left = 0 as LONG;
    rc.right = rc.left;
    rc.bottom = rc.right;
    rc.top = em_size as LONG;
    DPtoLP(hdc, &raw mut rc as LPPOINT, 2 as ::core::ffi::c_int);
    lf.lfHeight = -(abs((rc.top - rc.bottom) as ::core::ffi::c_int) as LONG);
    lf.lfOrientation = 0 as LONG;
    lf.lfEscapement = lf.lfOrientation;
    unscaled_font = CreateFontIndirectW(&raw mut lf);
    old_font = SelectObject(hdc, unscaled_font as HGDIOBJ) as HFONT;

    glyph = first as UINT;
    'error: {
    while (glyph as DWORD) < first.wrapping_add(count) {
        let mut needed: DWORD = 0;
        let mut gm: GLYPHMETRICS = core::mem::zeroed();
        let mut buf: *mut BYTE = ::core::ptr::null_mut::<BYTE>();
        let mut pph: *mut TTPOLYGONHEADER = ::core::ptr::null_mut::<TTPOLYGONHEADER>();
        let mut ppc: *mut TTPOLYCURVE = ::core::ptr::null_mut::<TTPOLYCURVE>();
        let mut vertices: *mut GLdouble = ::core::ptr::null_mut::<GLdouble>();
        let mut vertices_temp: *mut GLdouble = ::core::ptr::null_mut::<GLdouble>();
        let mut vertex_total: ::core::ffi::c_int = -(1 as ::core::ffi::c_int);

        if unicode != 0 {
            needed = GetGlyphOutlineW(
                hdc,
                glyph,
                GGO_NATIVE as UINT,
                &raw mut gm,
                0 as DWORD,
                NULL,
                &raw const identity,
            );
        } else {
            needed = GetGlyphOutlineA(
                hdc,
                glyph,
                GGO_NATIVE as UINT,
                &raw mut gm,
                0 as DWORD,
                NULL,
                &raw const identity,
            );
        }

        if needed == GDI_ERROR as DWORD {
            break 'error;
        }

        buf = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            needed as SIZE_T,
        ) as *mut BYTE;

        if buf.is_null() {
            break 'error;
        }

        if unicode != 0 {
            GetGlyphOutlineW(
                hdc,
                glyph,
                GGO_NATIVE as UINT,
                &raw mut gm,
                needed,
                buf as LPVOID,
                &raw const identity,
            );
        } else {
            GetGlyphOutlineA(
                hdc,
                glyph,
                GGO_NATIVE as UINT,
                &raw mut gm,
                needed,
                buf as LPVOID,
                &raw const identity,
            );
        }

        TRACE!("glyph %d\n", glyph);

        if !lpgmf.is_null() {
            (*lpgmf).gmfBlackBoxX = (gm.gmBlackBoxX as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;
            (*lpgmf).gmfBlackBoxY = (gm.gmBlackBoxY as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;
            (*lpgmf).gmfptGlyphOrigin.x = (gm.gmptGlyphOrigin.x as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;
            (*lpgmf).gmfptGlyphOrigin.y = (gm.gmptGlyphOrigin.y as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;
            (*lpgmf).gmfCellIncX = (gm.gmCellIncX as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;
            (*lpgmf).gmfCellIncY = (gm.gmCellIncY as ::core::ffi::c_float / em_size as ::core::ffi::c_float) as FLOAT;

            TRACE!("%fx%f at %f,%f inc %f,%f\n", (*lpgmf).gmfBlackBoxX as f64, (*lpgmf).gmfBlackBoxY as f64,
                  (*lpgmf).gmfptGlyphOrigin.x as f64, (*lpgmf).gmfptGlyphOrigin.y as f64, (*lpgmf).gmfCellIncX as f64, (*lpgmf).gmfCellIncY as f64);
            lpgmf = lpgmf.offset(1);
        }

        let fresh16 = listBase;
        listBase = listBase.wrapping_add(1);
        (*funcs).NewList.unwrap_unchecked()(fresh16 as GLuint, GL_COMPILE as GLenum);
        (*funcs).FrontFace.unwrap_unchecked()(GL_CCW as GLenum);
        if format == (WGL_FONT_POLYGONS as i32) {
            (*funcs).Normal3d.unwrap_unchecked()(
                0.0,
                0.0,
                1.0,
            );
            (pgluTessNormal.unwrap_unchecked())(
                tess,
                0 as GLdouble,
                0 as GLdouble,
                1 as GLdouble,
            );
            (pgluTessBeginPolygon.unwrap_unchecked())(tess, NULL);
        }

        'error_in_list: {
            while vertices.is_null() {
                if vertex_total != -(1 as ::core::ffi::c_int) {
                    vertices = HeapAlloc(
                        GetProcessHeap(),
                        0 as DWORD,
                        ((vertex_total * 3 as ::core::ffi::c_int) as SIZE_T)
                            .wrapping_mul(::core::mem::size_of::<GLdouble>() as SIZE_T),
                    ) as *mut GLdouble;
                    vertices_temp = vertices;
                }
                vertex_total = 0 as ::core::ffi::c_int;

                pph = buf as *mut TTPOLYGONHEADER;
                while (pph as *mut BYTE) < buf.offset(needed as isize) {
                    let mut previous: [GLdouble; 3] = [0.0; 3];
                    fixed_to_double((*pph).pfxStart, em_size, previous.as_mut_ptr());

                    if !vertices.is_null() {
                        TRACE!("\tstart %d, %d\n", (*pph).pfxStart.x.value as c_int, (*pph).pfxStart.y.value as c_int);
                    }

                    if format == (WGL_FONT_POLYGONS as i32) {
                        (pgluTessBeginContour.unwrap_unchecked())(tess);
                    } else {
                        (*funcs).Begin.unwrap_unchecked()(GL_LINE_LOOP as GLenum);
                    }

                    if !vertices.is_null() {
                        fixed_to_double((*pph).pfxStart, em_size, vertices);
                        if format == (WGL_FONT_POLYGONS as i32) {
                            (pgluTessVertex.unwrap_unchecked())(tess, vertices, vertices as *mut ::core::ffi::c_void);
                        } else {
                            (*funcs).Vertex3d.unwrap_unchecked()(
                                *vertices.offset(0 as isize),
                                *vertices.offset(1 as isize),
                                *vertices.offset(2 as isize),
                            );
                        }
                        vertices = vertices.offset(3 as isize);
                    }
                    vertex_total += 1;

                    ppc = (pph as *mut ::core::ffi::c_char)
                        .offset(::core::mem::size_of::<TTPOLYGONHEADER>() as isize)
                        as *mut TTPOLYCURVE;
                    while (ppc as *mut ::core::ffi::c_char)
                        < (pph as *mut ::core::ffi::c_char).offset((*pph).cb as isize)
                    {
                        let mut i: ::core::ffi::c_int = 0;
                        let mut j: ::core::ffi::c_int = 0;
                        let mut num: ::core::ffi::c_int = 0;

                        match ((*ppc).wType as ::core::ffi::c_int) as u32 {
                            TT_PRIM_LINE => {
                                i = 0 as ::core::ffi::c_int;
                                while i < (*ppc).cpfx as ::core::ffi::c_int {
                                    if !vertices.is_null() {
                                        TRACE!("\t\tline to %d, %d\n",
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)).x.value as c_int,
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)).y.value as c_int);
                                        fixed_to_double((*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)), em_size, vertices);
                                        if format == (WGL_FONT_POLYGONS as i32) {
                                            (pgluTessVertex.unwrap_unchecked())(tess, vertices, vertices as *mut ::core::ffi::c_void);
                                        } else {
                                            (*funcs).Vertex3d.unwrap_unchecked()(
                                                *vertices.offset(0 as isize),
                                                *vertices.offset(1 as isize),
                                                *vertices.offset(2 as isize),
                                            );
                                        }
                                        vertices = vertices.offset(3 as isize);
                                    }
                                    fixed_to_double((*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)), em_size, previous.as_mut_ptr());
                                    vertex_total += 1;
                                    i += 1;
                                }
                            }

                            TT_PRIM_QSPLINE => {
                                i = 0 as ::core::ffi::c_int;
                                while i < (*ppc).cpfx as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
                                    let mut curve: [bezier_vector; 3] =
                                        [bezier_vector { x: 0.0, y: 0.0 }, bezier_vector { x: 0.0, y: 0.0 }, bezier_vector { x: 0.0, y: 0.0 }];
                                    let mut points: *mut bezier_vector = ::core::ptr::null_mut::<bezier_vector>();
                                    let mut curve_vertex: [GLdouble; 3] = [0.0; 3];

                                    if !vertices.is_null() {
                                        TRACE!("\t\tcurve  %d,%d %d,%d\n",
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)).x.value as c_int,
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)).y.value as c_int,
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add((i + 1) as usize)).x.value as c_int,
                                              (*core::ptr::addr_of!((*ppc).apfx[0]).add((i + 1) as usize)).y.value as c_int);
                                    }

                                    curve[0 as usize].x = previous[0 as usize];
                                    curve[0 as usize].y = previous[1 as usize];
                                    fixed_to_double((*core::ptr::addr_of!((*ppc).apfx[0]).add(i as usize)), em_size, curve_vertex.as_mut_ptr());
                                    curve[1 as usize].x = curve_vertex[0 as usize];
                                    curve[1 as usize].y = curve_vertex[1 as usize];
                                    fixed_to_double(
                                        (*core::ptr::addr_of!((*ppc).apfx[0]).add((i + 1 as ::core::ffi::c_int) as usize)),
                                        em_size,
                                        curve_vertex.as_mut_ptr(),
                                    );
                                    curve[2 as usize].x = curve_vertex[0 as usize];
                                    curve[2 as usize].y = curve_vertex[1 as usize];
                                    if i < (*ppc).cpfx as ::core::ffi::c_int - 2 as ::core::ffi::c_int {
                                        curve[2 as usize].x =
                                            (curve[1 as usize].x + curve[2 as usize].x)
                                                / 2 as GLdouble;
                                        curve[2 as usize].y =
                                            (curve[1 as usize].y + curve[2 as usize].y)
                                                / 2 as GLdouble;
                                    }
                                    num = bezier_approximate(
                                        curve.as_mut_ptr(),
                                        ::core::ptr::null_mut::<bezier_vector>(),
                                        deviation,
                                    );
                                    points = HeapAlloc(
                                        GetProcessHeap(),
                                        0 as DWORD,
                                        (num as SIZE_T).wrapping_mul(::core::mem::size_of::<bezier_vector>() as SIZE_T),
                                    ) as *mut bezier_vector;
                                    num = bezier_approximate(curve.as_mut_ptr(), points, deviation);
                                    vertex_total += num;
                                    if !vertices.is_null() {
                                        j = 0 as ::core::ffi::c_int;
                                        while j < num {
                                            TRACE!("\t\t\tvertex at %f,%f\n", (*points.offset(j as isize)).x, (*points.offset(j as isize)).y);
                                            *vertices.offset(0 as isize) = (*points.offset(j as isize)).x;
                                            *vertices.offset(1 as isize) = (*points.offset(j as isize)).y;
                                            *vertices.offset(2 as isize) = 0.0;
                                            if format == (WGL_FONT_POLYGONS as i32) {
                                                (pgluTessVertex.unwrap_unchecked())(tess, vertices, vertices as *mut ::core::ffi::c_void);
                                            } else {
                                                (*funcs).Vertex3d.unwrap_unchecked()(
                                                    *vertices.offset(0 as isize),
                                                    *vertices.offset(1 as isize),
                                                    *vertices.offset(2 as isize),
                                                );
                                            }
                                            vertices = vertices.offset(3 as isize);
                                            j += 1;
                                        }
                                    }
                                    HeapFree(GetProcessHeap(), 0 as DWORD, points as PVOID);
                                    previous[0 as usize] = curve[2 as usize].x;
                                    previous[1 as usize] = curve[2 as usize].y;
                                    i += 1;
                                }
                            }
                            _ => {
                                ERR!("\t\tcurve type = %d\n", (*ppc).wType as c_int);
                                if format == (WGL_FONT_POLYGONS as i32) {
                                    (pgluTessEndContour.unwrap_unchecked())(tess);
                                } else {
                                    (*funcs).End.unwrap_unchecked()();
                                }
                                break 'error_in_list;
                            }
                        }

                        ppc = (ppc as *mut ::core::ffi::c_char)
                            .offset(
                                ::core::mem::size_of::<TTPOLYCURVE>()
                                    .wrapping_add(
                                        (((*ppc).cpfx as c_int - 1) as usize)
                                            .wrapping_mul(::core::mem::size_of::<POINTFX>()),
                                    ) as isize,
                            ) as *mut TTPOLYCURVE;
                    }
                    if format == (WGL_FONT_POLYGONS as i32) {
                        (pgluTessEndContour.unwrap_unchecked())(tess);
                    } else {
                        (*funcs).End.unwrap_unchecked()();
                    }
                    pph = (pph as *mut ::core::ffi::c_char).offset((*pph).cb as isize) as *mut TTPOLYGONHEADER;
                }
            }
        }

        if format == (WGL_FONT_POLYGONS as i32) {
            (pgluTessEndPolygon.unwrap_unchecked())(tess);
        }
        (*funcs).Translated.unwrap_unchecked()(
            gm.gmCellIncX as GLdouble / em_size as GLdouble,
            gm.gmCellIncY as GLdouble / em_size as GLdouble,
            0.0,
        );
        (*funcs).EndList.unwrap_unchecked()();

        HeapFree(GetProcessHeap(), 0 as DWORD, buf as PVOID);

        if !vertices_temp.is_null() {
            HeapFree(GetProcessHeap(), 0 as DWORD, vertices_temp as PVOID);
        }

        glyph = glyph.wrapping_add(1);
    }
    }

    DeleteObject(SelectObject(hdc, old_font as HGDIOBJ));
    if format == (WGL_FONT_POLYGONS as i32) {
        (pgluDeleteTess.unwrap_unchecked())(tess);
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglUseFontOutlinesA(
    hdc: HDC,
    first: DWORD,
    count: DWORD,
    listBase: DWORD,
    deviation: FLOAT,
    extrusion: FLOAT,
    format: i32,
    lpgmf: LPGLYPHMETRICSFLOAT,
) -> BOOL {
    return wglUseFontOutlines_common(hdc, first, count, listBase, deviation, extrusion, format, lpgmf, (FALSE) as BOOL);
}

#[no_mangle]
pub unsafe extern "system" fn wglUseFontOutlinesW(
    hdc: HDC,
    first: DWORD,
    count: DWORD,
    listBase: DWORD,
    deviation: FLOAT,
    extrusion: FLOAT,
    format: i32,
    lpgmf: LPGLYPHMETRICSFLOAT,
) -> BOOL {
    return wglUseFontOutlines_common(hdc, first, count, listBase, deviation, extrusion, format, lpgmf, (TRUE) as BOOL);
}
