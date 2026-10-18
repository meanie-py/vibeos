use crate::prelude::*;
use core::ffi::c_void;

extern "C" {
    fn strcmp(s1: *const i8, s2: *const i8) -> i32;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut dc_data_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut dc_data_list: *mut wgl_dc_data = ::core::ptr::null::<wgl_dc_data>() as *mut wgl_dc_data;

#[no_mangle]
pub static mut ContextListHead: LIST_ENTRY = _LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

// WINE_DEFAULT_DEBUG_CHANNEL(wgl): this TU's default channel is `wgl`, not the crate-wide
// `opengl32` the prelude's `__wine_dbch___default` names. The local item shadows the glob import.
#[allow(non_upper_case_globals)]
static mut __wine_dbch___default: *mut __wine_debug_channel = unsafe { &raw mut __wine_dbch_wgl };

// get_dc_data_ex's local `union { HWND hwnd; HDC hdc; HANDLE u; } id;`
#[repr(C)]
#[derive(Copy, Clone)]
union C2RustUnnamed_12 {
    hwnd: HWND,
    hdc: HDC,
    u: HANDLE,
}

unsafe extern "C" fn get_dc_data_ex(
    mut hdc: HDC,
    mut format: INT,
    mut size: UINT,
    mut descr: *mut PIXELFORMATDESCRIPTOR,
) -> *mut wgl_dc_data {
    let mut hwnd: HWND = core::ptr::null_mut();
    let mut data: *mut wgl_dc_data = core::ptr::null_mut();
    let mut objType: DWORD = GetObjectType(hdc as HGDIOBJ);
    let mut flags: ULONG = 0;
    let mut id: C2RustUnnamed_12 = C2RustUnnamed_12 {
        hwnd: core::ptr::null_mut(),
    };
    if objType == OBJ_DC {
        hwnd = WindowFromDC(hdc);
        if hwnd.is_null() {
            return core::ptr::null_mut();
        }
        id.hwnd = hwnd;
        flags = WGL_DC_OBJ_DC;
    } else if objType == OBJ_MEMDC {
        id.hdc = hdc;
    } else {
        return core::ptr::null_mut();
    }
    EnterCriticalSection(core::ptr::addr_of_mut!(dc_data_cs));
    data = dc_data_list;
    while !data.is_null() {
        if (*data).owner.u == id.u {
            LeaveCriticalSection(core::ptr::addr_of_mut!(dc_data_cs));
            return data;
        }
        data = (*data).next;
    }
    data = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<wgl_dc_data>() as SIZE_T,
    ) as *mut wgl_dc_data;
    if data.is_null() {
        LeaveCriticalSection(core::ptr::addr_of_mut!(dc_data_cs));
        return core::ptr::null_mut();
    }
    (*data).owner.u = id.u;
    (*data).flags = flags;
    (*data).pixelformat = 0;
    (*data).sw_data = core::ptr::null_mut();
    (*data).icd_data = IntGetIcdData(hdc);
    if !(*data).icd_data.is_null() {
        (*data).nb_icd_formats = (*(*data).icd_data)
            .DrvDescribePixelFormat
            .unwrap_unchecked()(
            hdc,
            format,
            size,
            descr as LPPIXELFORMATDESCRIPTOR,
        ) as INT;
    } else {
        (*data).nb_icd_formats = 0;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/opengl/opengl32/wgl.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"get_dc_data_ex\0" as *const u8 as *const ::core::ffi::c_char,
            82 as ::core::ffi::c_int,
            b"ICD %S has %u formats for HDC %x.\n\0" as *const u8 as *const ::core::ffi::c_char,
            if !(*data).icd_data.is_null() {
                core::ptr::addr_of_mut!((*(*data).icd_data).DriverName) as *mut WCHAR
            } else {
                core::ptr::null_mut::<WCHAR>()
            },
            (*data).nb_icd_formats,
            hdc,
        );
    }
    (*data).nb_sw_formats = sw_DescribePixelFormat(
        hdc,
        0,
        0,
        core::ptr::null_mut::<PIXELFORMATDESCRIPTOR>(),
    );
    (*data).next = dc_data_list;
    dc_data_list = data;
    LeaveCriticalSection(core::ptr::addr_of_mut!(dc_data_cs));
    return data;
}

unsafe extern "C" fn get_dc_data(mut hdc: HDC) -> *mut wgl_dc_data {
    return get_dc_data_ex(hdc, 0, 0, core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "C" fn release_dc_data(dc_data: *mut wgl_dc_data) {
    let _ = dc_data;
}
#[no_mangle]
pub unsafe extern "C" fn get_context(hglrc: HGLRC) -> *mut wgl_context {
    let mut context: *mut wgl_context = hglrc as *mut wgl_context;

    if hglrc.is_null() {
        return core::ptr::null_mut();
    }

    // _SEH2_TRY / _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER): a garbage HGLRC that faults on
    // the magic read is "no context". DUMMY_PSEH compiled the guard out of the C (the handler
    // was a runtime-false `if (_SEH2_Volatile0 || ...)`); rpc_try catches everything, as the
    // C's filter does.
    let magic = ntdll_sys::seh::rpc_try(|| core::ptr::read_volatile(core::ptr::addr_of!((*context).magic)));
    if magic != Ok(1196184131u32 /* 'GLRC' */) {
        context = core::ptr::null_mut();
    }

    return context;
}

#[no_mangle]
pub unsafe extern "system" fn wglDescribePixelFormat(hdc: HDC, mut format: INT, size: UINT, descr: *mut PIXELFORMATDESCRIPTOR) -> INT {
    let dc_data: *mut wgl_dc_data = get_dc_data_ex(hdc, format, size, descr);
    let mut ret: INT;

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0 as INT;
    }

    ret = (*dc_data).nb_icd_formats + (*dc_data).nb_sw_formats;

    if descr.is_null() {
        release_dc_data(dc_data);
        return ret;
    }
    if (format <= 0 as INT) || (format > ret) || ((size as usize) < core::mem::size_of::<PIXELFORMATDESCRIPTOR>()) {
        release_dc_data(dc_data);
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0 as INT;
    }

    /* Query ICD if needed */
    if format <= (*dc_data).nb_icd_formats {
        let mut icd_data: *mut ICD_Data = (*dc_data).icd_data;
        /* SetPixelFormat may have NULLified this */
        if icd_data.is_null() {
            icd_data = IntGetIcdData(hdc);
        }
        if (*icd_data).DrvDescribePixelFormat.unwrap_unchecked()(
            hdc,
            format,
            size,
            descr as LPPIXELFORMATDESCRIPTOR,
        ) == 0
        {
            ret = 0 as INT;
        }
    } else {
        /* This is a software format */
        format -= (*dc_data).nb_icd_formats;
        if sw_DescribePixelFormat(hdc, format, size, descr) == 0 {
            ret = 0 as INT;
        }
    }

    release_dc_data(dc_data);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn wglChoosePixelFormat(hdc: HDC, ppfd: *const PIXELFORMATDESCRIPTOR) -> INT {
    let mut format: PIXELFORMATDESCRIPTOR = core::mem::zeroed();
    let mut best: PIXELFORMATDESCRIPTOR = core::mem::zeroed();
    let mut i: core::ffi::c_int;
    let mut count: core::ffi::c_int;
    let mut best_format: core::ffi::c_int;
    let mut bestDBuffer: core::ffi::c_int = -(1);
    let mut bestStereo: core::ffi::c_int = -(1);

    TRACE!(
        b"%p %p: size %u version %u flags %u type %u color %u %u,%u,%u,%u accum %u depth %u stencil %u aux %u\n\0".as_ptr() as *const i8,
        hdc,
        ppfd,
        (*ppfd).nSize as core::ffi::c_int,
        (*ppfd).nVersion as core::ffi::c_int,
        (*ppfd).dwFlags,
        (*ppfd).iPixelType as core::ffi::c_int,
        (*ppfd).cColorBits as core::ffi::c_int,
        (*ppfd).cRedBits as core::ffi::c_int,
        (*ppfd).cGreenBits as core::ffi::c_int,
        (*ppfd).cBlueBits as core::ffi::c_int,
        (*ppfd).cAlphaBits as core::ffi::c_int,
        (*ppfd).cAccumBits as core::ffi::c_int,
        (*ppfd).cDepthBits as core::ffi::c_int,
        (*ppfd).cStencilBits as core::ffi::c_int,
        (*ppfd).cAuxBuffers as core::ffi::c_int,
    );

    count = wglDescribePixelFormat(
        hdc,
        0,
        0,
        core::ptr::null_mut(),
    ) as core::ffi::c_int;
    if count == 0 {
        return 0;
    }

    best_format = 0;
    best.dwFlags = PFD_GENERIC_FORMAT;
    best.cAlphaBits = (-(1) as core::ffi::c_int) as BYTE;
    best.cColorBits = (-(1) as core::ffi::c_int) as BYTE;
    best.cDepthBits = (-(1) as core::ffi::c_int) as BYTE;
    best.cStencilBits = (-(1) as core::ffi::c_int) as BYTE;
    best.cAuxBuffers = (-(1) as core::ffi::c_int) as BYTE;

    i = 1;
    while i <= count {
        'iter: {
            if wglDescribePixelFormat(
                hdc,
                i as INT,
                core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as UINT,
                core::ptr::addr_of_mut!(format),
            ) == 0
            {
                break 'iter;
            }

            if (*ppfd).iPixelType != format.iPixelType {
                TRACE!(
                    b"pixel type mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                    i,
                );
                break 'iter;
            }

            /* only use bitmap capable formats for bitmap rendering */
            if ((*ppfd).dwFlags & PFD_DRAW_TO_BITMAP) != 0 && (format.dwFlags & PFD_DRAW_TO_BITMAP) == 0
            {
                TRACE!(
                    b"PFD_DRAW_TO_BITMAP mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                    i,
                );
                break 'iter;
            }

            /* only use window capable formats for window rendering */
            if ((*ppfd).dwFlags & PFD_DRAW_TO_WINDOW) != 0 && (format.dwFlags & PFD_DRAW_TO_WINDOW) == 0
            {
                TRACE!(
                    b"PFD_DRAW_TO_WINDOW mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                    i,
                );
                break 'iter;
            }

            /* only use opengl capable formats for opengl rendering */
            if ((*ppfd).dwFlags & PFD_SUPPORT_OPENGL) != 0 && (format.dwFlags & PFD_SUPPORT_OPENGL) == 0
            {
                TRACE!(
                    b"PFD_SUPPORT_OPENGL mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                    i,
                );
                break 'iter;
            }

            /* only use GDI capable formats for GDI rendering */
            if ((*ppfd).dwFlags & PFD_SUPPORT_GDI) != 0 && (format.dwFlags & PFD_SUPPORT_GDI) == 0
            {
                TRACE!(
                    b"PFD_SUPPORT_GDI mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                    i,
                );
                break 'iter;
            }

            'checks: {
                /* Doublebuffer, see the comments above */
                if ((*ppfd).dwFlags & PFD_DOUBLEBUFFER_DONTCARE) == 0 {
                    if ((((*ppfd).dwFlags & PFD_DOUBLEBUFFER) as core::ffi::c_int) != bestDBuffer)
                        && ((format.dwFlags & PFD_DOUBLEBUFFER) == ((*ppfd).dwFlags & PFD_DOUBLEBUFFER))
                    {
                        break 'checks;
                    }

                    if bestDBuffer != -(1)
                        && (((format.dwFlags & PFD_DOUBLEBUFFER) as core::ffi::c_int) != bestDBuffer)
                    {
                        TRACE!(
                            b"double buffer mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }

                /* Stereo, see the comments above. */
                if ((*ppfd).dwFlags & PFD_STEREO_DONTCARE) == 0 {
                    if ((((*ppfd).dwFlags & PFD_STEREO) as core::ffi::c_int) != bestStereo)
                        && ((format.dwFlags & PFD_STEREO) == ((*ppfd).dwFlags & PFD_STEREO))
                    {
                        break 'checks;
                    }

                    if bestStereo != -(1)
                        && (((format.dwFlags & PFD_STEREO) as core::ffi::c_int) != bestStereo)
                    {
                        TRACE!(
                            b"stereo mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }

                if (*ppfd).cColorBits != 0 {
                    if ((*ppfd).cColorBits > best.cColorBits && format.cColorBits > best.cColorBits)
                        || (format.cColorBits >= (*ppfd).cColorBits && format.cColorBits < best.cColorBits)
                    {
                        break 'checks;
                    }

                    if best.cColorBits != format.cColorBits {
                        TRACE!(
                            b"color mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }
                if (*ppfd).cAlphaBits != 0 {
                    if ((*ppfd).cAlphaBits > best.cAlphaBits && format.cAlphaBits > best.cAlphaBits)
                        || (format.cAlphaBits >= (*ppfd).cAlphaBits && format.cAlphaBits < best.cAlphaBits)
                    {
                        break 'checks;
                    }

                    if best.cAlphaBits != format.cAlphaBits {
                        TRACE!(
                            b"alpha mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }
                if (*ppfd).cDepthBits != 0 {
                    if ((*ppfd).cDepthBits > best.cDepthBits && format.cDepthBits > best.cDepthBits)
                        || (format.cDepthBits >= (*ppfd).cDepthBits && format.cDepthBits < best.cDepthBits)
                    {
                        break 'checks;
                    }

                    if best.cDepthBits != format.cDepthBits {
                        TRACE!(
                            b"depth mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }
                if (*ppfd).cStencilBits != 0 {
                    if ((*ppfd).cStencilBits > best.cStencilBits && format.cStencilBits > best.cStencilBits)
                        || (format.cStencilBits >= (*ppfd).cStencilBits && format.cStencilBits < best.cStencilBits)
                    {
                        break 'checks;
                    }

                    if best.cStencilBits != format.cStencilBits {
                        TRACE!(
                            b"stencil mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }
                if (*ppfd).cAuxBuffers != 0 {
                    if ((*ppfd).cAuxBuffers > best.cAuxBuffers && format.cAuxBuffers > best.cAuxBuffers)
                        || (format.cAuxBuffers >= (*ppfd).cAuxBuffers && format.cAuxBuffers < best.cAuxBuffers)
                    {
                        break 'checks;
                    }

                    if best.cAuxBuffers != format.cAuxBuffers {
                        TRACE!(
                            b"aux mismatch for iPixelFormat=%d\n\0".as_ptr() as *const i8,
                            i,
                        );
                        break 'iter;
                    }
                }
                break 'iter;
            }

            /* found: */
            /* Prefer HW accelerated formats */
            if (format.dwFlags & PFD_GENERIC_FORMAT) != 0 && (best.dwFlags & PFD_GENERIC_FORMAT) == 0 {
                break 'iter;
            }
            best_format = i;
            best = format;
            bestDBuffer = (format.dwFlags & PFD_DOUBLEBUFFER) as core::ffi::c_int;
            bestStereo = (format.dwFlags & PFD_STEREO) as core::ffi::c_int;
        }
        i += 1;
    }

    TRACE!(
        b"returning %u\n\0".as_ptr() as *const i8,
        best_format,
    );
    return best_format as INT;
}

#[no_mangle]
pub unsafe extern "system" fn wglCopyContext(hglrcSrc: HGLRC, hglrcDst: HGLRC, mask: UINT) -> BOOL {
    let ctx_src: *mut wgl_context = get_context(hglrcSrc);
    let ctx_dst: *mut wgl_context = get_context(hglrcDst);

    if ctx_src.is_null() || ctx_dst.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    /* Check this is the same pixel format */
    if (*ctx_dst).icd_data != (*ctx_src).icd_data
        || (*ctx_dst).pixelformat != (*ctx_src).pixelformat
    {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return (FALSE) as BOOL;
    }

    if !(*ctx_src).icd_data.is_null() {
        return (*(*ctx_src).icd_data)
            .DrvCopyContext
            .unwrap_unchecked()(
            (*ctx_src).dhglrc,
            (*ctx_dst).dhglrc,
            mask,
        );
    }

    return sw_CopyContext((*ctx_src).dhglrc, (*ctx_dst).dhglrc, mask);
}

#[no_mangle]
pub unsafe extern "system" fn wglCreateContext(hdc: HDC) -> HGLRC {
    let mut dc_data: *mut wgl_dc_data = get_dc_data(hdc);
    let mut context: *mut wgl_context = core::ptr::null_mut();
    let mut dhglrc: DHGLRC = core::ptr::null_mut();

    TRACE!("Creating context for %p.\n", hdc);

    if dc_data.is_null() {
        WARN!("Not a DC handle!\n");
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return core::ptr::null_mut();
    }

    if (*dc_data).pixelformat == 0 {
        WARN!("Pixel format not set!\n");
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return core::ptr::null_mut();
    }

    if (*dc_data).icd_data.is_null() {
        TRACE!("Calling SW implementation.\n");
        dhglrc = sw_CreateContext(dc_data);
        TRACE!("done\n");
    } else {
        TRACE!("Calling ICD.\n");
        dhglrc = (*(*dc_data).icd_data)
            .DrvCreateContext
            .unwrap_unchecked()(hdc);
    }

    if dhglrc.is_null() {
        WARN!("Failed!\n");
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return core::ptr::null_mut();
    }

    context = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<wgl_context>() as SIZE_T,
    ) as *mut wgl_context;
    if context.is_null() {
        WARN!("Failed to allocate a context!\n");
        if (*dc_data).icd_data.is_null() {
            sw_DeleteContext(dhglrc);
        } else {
            (*(*dc_data).icd_data)
                .DrvDeleteContext
                .unwrap_unchecked()(dhglrc);
        }
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return core::ptr::null_mut();
    }

    /* Copy info from the DC data */
    (*context).dhglrc = dhglrc;
    (*context).icd_data = (*dc_data).icd_data;
    (*context).pixelformat = (*dc_data).pixelformat;
    core::ptr::write_volatile(
        &mut (*context).thread_id as *mut LONG,
        0 as LONG,
    );
    (*context).magic = 1196184131i32 as DWORD; // 'GLRC'

    /* Insert into the list */
    InsertTailList(
        core::ptr::addr_of_mut!(ContextListHead),
        core::ptr::addr_of_mut!((*context).ListEntry),
    );

    TRACE!("Success!\n");
    return context as HGLRC;
}

#[no_mangle]
pub unsafe extern "system" fn wglCreateLayerContext(hdc: HDC, iLayerPlane: i32) -> HGLRC {
    let mut dc_data: *mut wgl_dc_data = get_dc_data(hdc);
    let mut context: *mut wgl_context = core::ptr::null_mut();
    let mut dhglrc: DHGLRC = core::ptr::null_mut();

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return core::ptr::null_mut();
    }

    if (*dc_data).pixelformat == 0 {
        release_dc_data(dc_data);
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return core::ptr::null_mut();
    }

    if (*dc_data).icd_data.is_null() {
        if iLayerPlane != 0 as i32 {
            release_dc_data(dc_data);
            SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
            return core::ptr::null_mut();
        }
        dhglrc = sw_CreateContext(dc_data);
    } else {
        dhglrc = (*(*dc_data).icd_data)
            .DrvCreateLayerContext
            .unwrap_unchecked()(hdc, iLayerPlane);
    }

    if dhglrc.is_null() {
        release_dc_data(dc_data);
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return core::ptr::null_mut();
    }

    context = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<wgl_context>() as SIZE_T,
    ) as *mut wgl_context;
    if context.is_null() {
        if (*dc_data).icd_data.is_null() {
            sw_DeleteContext(dhglrc);
        } else {
            (*(*dc_data).icd_data)
                .DrvDeleteContext
                .unwrap_unchecked()(dhglrc);
        }
        release_dc_data(dc_data);
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return core::ptr::null_mut();
    }

    (*context).dhglrc = dhglrc;
    (*context).icd_data = (*dc_data).icd_data;
    (*context).pixelformat = (*dc_data).pixelformat;
    (*context).thread_id = 0 as LONG;
    (*context).magic = 1196184131i32 as DWORD; // 'GLRC'

    /* Insert into the list */
    InsertTailList(
        core::ptr::addr_of_mut!(ContextListHead),
        core::ptr::addr_of_mut!((*context).ListEntry),
    );

    release_dc_data(dc_data);
    return context as HGLRC;
}

#[no_mangle]
pub unsafe extern "system" fn wglDeleteContext(hglrc: HGLRC) -> BOOL {
    let context: *mut wgl_context = get_context(hglrc);
    let thread_id: LONG = GetCurrentThreadId() as LONG;

    if context.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    /* Own this context before touching it */
    if InterlockedCompareExchange(
        core::ptr::addr_of_mut!((*context).thread_id),
        thread_id,
        0,
    ) != 0
    {
        /* We can't delete a context current to another thread */
        if (*context).thread_id != thread_id {
            SetLastError(ERROR_BUSY as DWORD);
            return (FALSE) as BOOL;
        }

        /* This is in our thread. Release and try again */
        if wglMakeCurrent(core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
            return (FALSE) as BOOL;
        }
        return wglDeleteContext(hglrc);
    }

    if !(*context).icd_data.is_null() {
        (*(*context).icd_data)
            .DrvDeleteContext
            .unwrap_unchecked()((*context).dhglrc);
    } else {
        sw_DeleteContext((*context).dhglrc);
    }

    (*context).magic = 0;
    RemoveEntryList(core::ptr::addr_of_mut!((*context).ListEntry));
    HeapFree(
        GetProcessHeap(),
        0,
        context as PVOID,
    );

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglDescribeLayerPlane(
    mut hdc: HDC,
    mut iPixelFormat: ::core::ffi::c_int,
    mut iLayerPlane: ::core::ffi::c_int,
    mut nBytes: UINT,
    mut plpd: LPLAYERPLANEDESCRIPTOR,
) -> BOOL {
    let mut dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    if iPixelFormat <= (*dc_data).nb_icd_formats {
        return (*(*dc_data).icd_data)
            .DrvDescribeLayerPlane
            .unwrap_unchecked()(
            hdc, iPixelFormat, iLayerPlane, nBytes, plpd
        );
    }

    /* SW implementation doesn't support this */
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglGetCurrentContext() -> HGLRC {
    IntGetCurrentRC()
}

#[no_mangle]
pub unsafe extern "system" fn wglGetCurrentDC() -> HDC {
    IntGetCurrentDC()
}

#[no_mangle]
pub unsafe extern "system" fn wglGetDefaultProcAddress(lpszProc: LPCSTR) -> PROC {
    /* undocumented... */
    None
}

#[no_mangle]
pub unsafe extern "system" fn wglGetLayerPaletteEntries(
    hdc: HDC,
    iLayerPlane: ::core::ffi::c_int,
    iStart: ::core::ffi::c_int,
    cEntries: ::core::ffi::c_int,
    pcr: *mut COLORREF,
) -> ::core::ffi::c_int {
    let dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0 as ::core::ffi::c_int;
    }

    if (*dc_data).pixelformat == 0 {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return 0 as ::core::ffi::c_int;
    }

    if !(*dc_data).icd_data.is_null() {
        return (*(*dc_data).icd_data)
            .DrvGetLayerPaletteEntries
            .unwrap_unchecked()(
            hdc, iLayerPlane, iStart, cEntries, pcr
        );
    }

    /* SW implementation doesn't support this */
    return 0 as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn wglGetPixelFormat(hdc: HDC) -> INT {
    let mut ret: INT;
    let dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0 as INT;
    }

    ret = (*dc_data).pixelformat;
    release_dc_data(dc_data);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn wglGetProcAddress(name: LPCSTR) -> PROC {
    let context: *mut wgl_context = get_context(IntGetCurrentRC());
    if context.is_null() {
        return None;
    }

    /* This shall fail for opengl 1.1 functions */
    if strcmp(name as *const i8, b"glNewList\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEndList\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCallList\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCallLists\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDeleteLists\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGenLists\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glListBase\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glBegin\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glBitmap\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3b\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3bv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3ub\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3ubv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3ui\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3uiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3us\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor3usv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4b\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4bv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4ub\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4ubv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4ui\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4uiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4us\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColor4usv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEdgeFlag\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEdgeFlagv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEnd\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexd\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexdv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexi\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexs\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexsv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3b\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3bv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormal3sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos2sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos3sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRasterPos4sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectd\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectdv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRecti\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRects\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRectsv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord1sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord2sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord3sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoord4sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex2sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex3sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4i\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4iv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4s\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertex4sv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClipPlane\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColorMaterial\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCullFace\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFogf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFogfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFogi\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFogiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFrontFace\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glHint\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLighti\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightModelf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightModelfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightModeli\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLightModeliv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLineStipple\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLineWidth\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMaterialf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMaterialfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMateriali\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMaterialiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPointSize\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPolygonMode\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPolygonStipple\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glScissor\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glShadeModel\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexParameterf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexParameterfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexParameteri\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexParameteriv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexImage1D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexImage2D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexEnvf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexEnvfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexEnvi\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexEnviv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGend\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGendv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGenf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGenfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGeni\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexGeniv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFeedbackBuffer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glSelectBuffer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRenderMode\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glInitNames\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLoadName\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPassThrough\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPopName\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPushName\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDrawBuffer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClear\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClearAccum\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClearIndex\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClearColor\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClearStencil\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glClearDepth\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glStencilMask\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColorMask\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDepthMask\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexMask\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glAccum\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDisable\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEnable\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFinish\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFlush\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPopAttrib\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPushAttrib\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMap1d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMap1f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMap2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMap2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMapGrid1d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMapGrid1f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMapGrid2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMapGrid2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord1d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord1dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord1f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord1fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord2d\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord2dv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord2f\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalCoord2fv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalMesh1\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalPoint1\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalMesh2\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEvalPoint2\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glAlphaFunc\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glBlendFunc\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLogicOp\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glStencilFunc\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glStencilOp\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDepthFunc\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelZoom\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelTransferf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelTransferi\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelStoref\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelStorei\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelMapfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelMapuiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPixelMapusv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glReadBuffer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCopyPixels\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glReadPixels\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDrawPixels\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetBooleanv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetClipPlane\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetDoublev\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetError\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetFloatv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetIntegerv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetLightfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetLightiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetMapdv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetMapfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetMapiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetMaterialfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetMaterialiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetPixelMapfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetPixelMapuiv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetPixelMapusv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetPolygonStipple\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetString\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexEnvfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexEnviv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexGendv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexGenfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexGeniv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexImage\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexParameterfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexParameteriv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexLevelParameterfv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetTexLevelParameteriv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIsEnabled\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIsList\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDepthRange\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glFrustum\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLoadIdentity\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLoadMatrixf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glLoadMatrixd\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMatrixMode\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMultMatrixf\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glMultMatrixd\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glOrtho\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPopMatrix\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPushMatrix\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRotated\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glRotatef\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glScaled\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glScalef\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTranslated\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTranslatef\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glViewport\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glArrayElement\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glBindTexture\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glColorPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDisableClientState\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDrawArrays\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDrawElements\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEdgeFlagPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glEnableClientState\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexub\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIndexubv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glInterleavedArrays\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glNormalPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPolygonOffset\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexCoordPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glVertexPointer\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glAreTexturesResident\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCopyTexImage1D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCopyTexImage2D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCopyTexSubImage1D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glCopyTexSubImage2D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glDeleteTextures\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGenTextures\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glGetPointerv\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glIsTexture\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPrioritizeTextures\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexSubImage1D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glTexSubImage2D\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPopClientAttrib\0".as_ptr() as *const i8) == 0 { return None; }
    if strcmp(name as *const i8, b"glPushClientAttrib\0".as_ptr() as *const i8) == 0 { return None; }

    /* Forward */
    if !(*context).icd_data.is_null() {
        return (*(*context).icd_data)
            .DrvGetProcAddress
            .unwrap_unchecked()(name);
    }
    return sw_GetProcAddress(name);
}
#[no_mangle]
pub unsafe extern "system" fn set_api_table(table: *const GLCLTPROCTABLE) {
    IntSetCurrentDispatchTable(core::ptr::addr_of!((*table).glDispatchTable));
}

#[no_mangle]
pub unsafe extern "system" fn wglMakeCurrent(hdc: HDC, hglrc: HGLRC) -> BOOL {
    let ctx: *mut wgl_context = get_context(hglrc);
    let old_ctx: *mut wgl_context = get_context(IntGetCurrentRC());
    let mut apiTable: *const GLCLTPROCTABLE = core::ptr::null();
    let thread_id: LONG = GetCurrentThreadId() as LONG;

    if !ctx.is_null() {
        let dc_data: *mut wgl_dc_data = get_dc_data(hdc);
        if dc_data.is_null() {
            ERR!("wglMakeCurrent was passed an invalid DC handle.\n");
            SetLastError(ERROR_INVALID_HANDLE);
            return (FALSE) as BOOL;
        }

        /* Check compatibility */
        if (*ctx).icd_data != (*dc_data).icd_data || (*ctx).pixelformat != (*dc_data).pixelformat {
            /* That's bad, man */
            ERR!("HGLRC %p and HDC %p are not compatible.\n", hglrc, hdc);
            release_dc_data(dc_data);
            SetLastError(ERROR_INVALID_HANDLE);
            return (FALSE) as BOOL;
        }

        /* Set the thread ID */
        if InterlockedCompareExchange(
            core::ptr::addr_of_mut!((*ctx).thread_id),
            thread_id,
            0,
        ) != 0
        {
            /* Already current for a thread. Maybe it's us ? */
            release_dc_data(dc_data);
            if (*ctx).thread_id != thread_id {
                SetLastError(ERROR_BUSY);
            }
            return ((*ctx).thread_id == thread_id) as BOOL;
        }

        if !old_ctx.is_null() {
            /* Unset it */
            if !(*old_ctx).icd_data.is_null() {
                (*(*old_ctx).icd_data)
                    .DrvReleaseContext
                    .unwrap_unchecked()((*old_ctx).dhglrc);
            } else {
                sw_ReleaseContext((*old_ctx).dhglrc);
            }
            InterlockedExchange(
                core::ptr::addr_of_mut!((*old_ctx).thread_id),
                0,
            );
        }

        /* Call the ICD or SW implementation */
        if !(*ctx).icd_data.is_null() {
            apiTable = (*(*ctx).icd_data)
                .DrvSetContext
                .unwrap_unchecked()(
                hdc,
                (*ctx).dhglrc,
                Some(set_api_table),
            );
            if apiTable.is_null() {
                ERR!("DrvSetContext failed!\n");
                /* revert */
                InterlockedExchange(
                    core::ptr::addr_of_mut!((*ctx).thread_id),
                    0,
                );
                IntSetCurrentDispatchTable(core::ptr::null_mut());
                SetLastError(ERROR_INVALID_PARAMETER);
                return (FALSE) as BOOL;
            }
            set_api_table(apiTable);
            /* Make it current */
            IntMakeCurrent(hglrc, hdc, dc_data);
        } else {
            /* We must set current before, SW implementation relies on it */
            IntMakeCurrent(hglrc, hdc, dc_data);
            if sw_SetContext(dc_data, (*ctx).dhglrc) == 0 {
                ERR!("sw_SetContext failed!\n");
                /* revert */
                IntMakeCurrent(
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                );
                InterlockedExchange(
                    core::ptr::addr_of_mut!((*ctx).thread_id),
                    0,
                );
                SetLastError(ERROR_INVALID_PARAMETER);
                return (FALSE) as BOOL;
            }
        }
    } else if !old_ctx.is_null() {
        if !(*old_ctx).icd_data.is_null() {
            (*(*old_ctx).icd_data)
                .DrvReleaseContext
                .unwrap_unchecked()((*old_ctx).dhglrc);
        } else {
            sw_ReleaseContext((*old_ctx).dhglrc);
        }
        InterlockedExchange(
            core::ptr::addr_of_mut!((*old_ctx).thread_id),
            0,
        );
        /* Unset it */
        IntMakeCurrent(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        IntSetCurrentDispatchTable(core::ptr::null_mut());
        /* Test conformance (extreme cases) */
        return (hglrc == core::ptr::null_mut()) as BOOL;
    } else {
        /* Winetest conformance */
        let objType: DWORD = GetObjectType(hdc as HGDIOBJ);
        if objType != OBJ_DC && objType != OBJ_MEMDC {
            if !hdc.is_null() {
                ERR!("hdc (%p) is not a DC handle (ObjectType: %d)!\n", hdc, objType);
            }
            SetLastError(ERROR_INVALID_HANDLE);
            return (FALSE) as BOOL;
        }
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglRealizeLayerPalette(
    hdc: HDC,
    iLayerPlane: core::ffi::c_int,
    bRealize: BOOL,
) -> BOOL {
    let dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    if (*dc_data).pixelformat == 0 {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return (FALSE) as BOOL;
    }

    if !(*dc_data).icd_data.is_null() {
        return (*(*dc_data).icd_data)
            .DrvRealizeLayerPalette
            .unwrap_unchecked()(hdc, iLayerPlane, bRealize);
    }

    /* SW implementation doesn't support this */
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglSetLayerPaletteEntries(
    hdc: HDC,
    iLayerPlane: ::core::ffi::c_int,
    iStart: ::core::ffi::c_int,
    cEntries: ::core::ffi::c_int,
    pcr: *const COLORREF,
) -> ::core::ffi::c_int {
    let dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return 0 as ::core::ffi::c_int;
    }

    if (*dc_data).pixelformat == 0 {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return 0 as ::core::ffi::c_int;
    }

    if !(*dc_data).icd_data.is_null() {
        return (*(*dc_data).icd_data)
            .DrvSetLayerPaletteEntries
            .unwrap_unchecked()(
            hdc, iLayerPlane, iStart, cEntries, pcr
        );
    }

    /* SW implementation doesn't support this */
    return 0 as ::core::ffi::c_int;
}

#[no_mangle]
pub unsafe extern "system" fn wglSetPixelFormat(hdc: HDC, format: INT, descr: *const PIXELFORMATDESCRIPTOR) -> BOOL {
    let mut dc_data: *mut wgl_dc_data = get_dc_data(hdc);
    let mut sw_format: INT = 0;
    let mut ret: BOOL = 0;

    TRACE!("HDC %p, format %i.\n", hdc, format);

    if dc_data.is_null() {
        WARN!("Not a valid DC!.\n");
        SetLastError(ERROR_INVALID_HANDLE);
        return (FALSE) as BOOL;
    }

    if format == 0 {
        WARN!("format == 0!\n");
        SetLastError(ERROR_INVALID_PARAMETER);
        return (FALSE) as BOOL;
    }

    if (*dc_data).pixelformat != 0 {
        TRACE!("DC format already set, %i.\n", (*dc_data).pixelformat);
        return (format == (*dc_data).pixelformat) as BOOL;
    }

    if format <= (*dc_data).nb_icd_formats {
        TRACE!("Calling ICD.\n");
        ret = ((*(*dc_data).icd_data).DrvSetPixelFormat.unwrap_unchecked())(hdc, format);
        if ret != 0 {
            TRACE!("Success!\n");
            (*dc_data).pixelformat = format;
        }
        return ret;
    }

    sw_format = format - (*dc_data).nb_icd_formats;
    if sw_format <= (*dc_data).nb_sw_formats {
        TRACE!("Calling SW implementation.\n");
        ret = sw_SetPixelFormat(hdc, dc_data, sw_format);
        if ret != 0 {
            TRACE!("Success!\n");
            /* This is now officially a software-only HDC */
            (*dc_data).icd_data = ::core::ptr::null_mut::<ICD_Data>();
            (*dc_data).pixelformat = format;
        }
        return ret;
    }

    TRACE!("Invalid pixel format!\n");
    SetLastError(ERROR_INVALID_PARAMETER);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn wglShareLists(hglrcSrc: HGLRC, hglrcDst: HGLRC) -> BOOL {
    let mut ctx_src: *mut wgl_context = get_context(hglrcSrc);
    let mut ctx_dst: *mut wgl_context = get_context(hglrcDst);

    if ctx_src.is_null() || ctx_dst.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    /* Check this is the same pixel format */
    if (*ctx_dst).icd_data != (*ctx_src).icd_data
        || (*ctx_dst).pixelformat != (*ctx_src).pixelformat
    {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return (FALSE) as BOOL;
    }

    if !(*ctx_src).icd_data.is_null() {
        return (*(*ctx_src).icd_data)
            .DrvShareLists
            .unwrap_unchecked()(
            (*ctx_src).dhglrc,
            (*ctx_dst).dhglrc,
        );
    }

    return sw_ShareLists((*ctx_src).dhglrc, (*ctx_dst).dhglrc);
}

#[no_mangle]
pub unsafe extern "system" fn wglSwapBuffers(mut hdc: HDC) -> BOOL {
    let mut dc_data: *mut wgl_dc_data = get_dc_data(hdc);

    if dc_data.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    if (*dc_data).pixelformat == 0 {
        SetLastError(ERROR_INVALID_PIXEL_FORMAT as DWORD);
        return (FALSE) as BOOL;
    }

    if !(*dc_data).icd_data.is_null() {
        return (*(*dc_data).icd_data)
            .DrvSwapBuffers
            .unwrap_unchecked()(hdc);
    }

    return sw_SwapBuffers(hdc, dc_data);
}

#[no_mangle]
pub unsafe extern "system" fn wglSwapLayerBuffers(hdc: HDC, fuPlanes: UINT) -> BOOL {
    (FALSE) as i32
}

#[no_mangle]
pub unsafe extern "system" fn wglSwapMultipleBuffers(
    count: UINT,
    toSwap: *const WGLSWAP,
) -> DWORD {
    0
}
#[no_mangle]
pub unsafe extern "C" fn IntDeleteAllContexts() {
    let mut context: *mut wgl_context;
    let mut Entry: *mut LIST_ENTRY = (*core::ptr::addr_of_mut!(ContextListHead)).Flink as *mut LIST_ENTRY;

    while Entry != core::ptr::addr_of_mut!(ContextListHead) as *mut LIST_ENTRY {
        context = CONTAINING_RECORD!(Entry, wgl_context, ListEntry);
        wglDeleteContext(context as HGLRC);
        Entry = (*Entry).Flink as *mut LIST_ENTRY;
    }
}
