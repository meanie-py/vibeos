use crate::prelude::*;
use core::ffi::c_void;

crate::wine_default_debug_channel!(__wine_dbch_animate);
// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_12 {
    pub hModule: HMODULE,
    pub fnICOpen: Option<unsafe extern "C" fn(DWORD, DWORD, UINT) -> HIC>,
    pub fnICClose: Option<unsafe extern "C" fn(HIC) -> LRESULT>,
    pub fnICSendMessage: Option<unsafe extern "C" fn(HIC, UINT, DWORD_PTR, DWORD_PTR) -> LRESULT>,
    pub fnICDecompress: Option<
        unsafe extern "C" fn(
            HIC,
            DWORD,
            LPBITMAPINFOHEADER,
            LPVOID,
            LPBITMAPINFOHEADER,
            LPVOID,
        ) -> DWORD,
    >,
}

pub const ANIMATE_CLASSW: [::core::ffi::c_ushort; 13] = unsafe {
    ::core::mem::transmute::<[u8; 26], [::core::ffi::c_ushort; 13]>(
        *b"S\0y\0s\0A\0n\0i\0m\0a\0t\0e\x003\x002\0\0\0",
    )
};

static mut fnIC: C2RustUnnamed_12 = C2RustUnnamed_12 {
    hModule: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    fnICOpen: None,
    fnICClose: None,
    fnICSendMessage: None,
    fnICDecompress: None,
};


#[allow(non_camel_case_types)]
#[repr(C)]
pub struct fnIC_t {
    pub hModule: HMODULE,
    pub fnICOpen: Option<unsafe extern "system" fn(DWORD, DWORD, UINT) -> HIC>,
    pub fnICClose: Option<unsafe extern "system" fn(HIC) -> LRESULT>,
    pub fnICSendMessage: Option<unsafe extern "system" fn(HIC, UINT, DWORD_PTR, DWORD_PTR) -> LRESULT>,
    pub fnICDecompress: Option<
        unsafe extern "C" fn(
            HIC,
            DWORD,
            LPBITMAPINFOHEADER,
            LPVOID,
            LPBITMAPINFOHEADER,
            LPVOID,
        ) -> DWORD,
    >,
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct ANIMATE_INFO {
    /* reference to input stream (file or resource) */
    pub hRes: HGLOBAL,
    pub hMMio: HMMIO, /* handle to mmio stream */
    pub hwndSelf: HWND,
    pub hwndNotify: HWND,
    pub dwStyle: DWORD,
    /* information on the loaded AVI file */
    pub mah: MainAVIHeader,
    pub ash: AVIStreamHeader,
    pub inbih: LPBITMAPINFOHEADER,
    pub lpIndex: LPDWORD,
    /* data for the decompressor */
    pub hic: HIC,
    pub outbih: LPBITMAPINFOHEADER,
    pub indata: LPVOID,
    pub outdata: LPVOID,
    /* data for the background mechanism */
    pub cs: CRITICAL_SECTION,
    pub hStopEvent: HANDLE,
    pub hThread: HANDLE,
    pub threadId: DWORD,
    pub uTimer: UINT,
    /* data for playing the file */
    pub nFromFrame: i32,
    pub nToFrame: i32,
    pub nLoop: i32,
    pub currFrame: i32,
    /* transparency info*/
    pub transparentColor: COLORREF,
    pub hbrushBG: HBRUSH,
    pub hbmPrevFrame: HBITMAP,
}

pub const ANIMATE_COLOR_NONE: u32 = 0xffffffff;
unsafe extern "C" fn ANIMATE_Notify(infoPtr: *const ANIMATE_INFO, notif: UINT) {
    PostMessageW(
        (*infoPtr).hwndNotify,
        WM_COMMAND as UINT,
        MAKEWPARAM!(GetDlgCtrlID((*infoPtr).hwndSelf) as WORD, notif as WORD) as WPARAM,
        (*infoPtr).hwndSelf as LPARAM,
    );
}

unsafe extern "C" fn ANIMATE_LoadResW(mut infoPtr: *mut ANIMATE_INFO, mut hInst: HINSTANCE, mut lpName: LPCWSTR) -> BOOL {
    static aviW: [WCHAR; 4] = [0x0041u16, 0x0056, 0x0049, 0x0000];
    let mut hrsrc: HRSRC = core::ptr::null_mut();
    let mut mminfo: MMIOINFO = core::mem::zeroed();
    let mut lpAvi: LPVOID = core::ptr::null_mut();

    hrsrc = FindResourceW(hInst as HMODULE, lpName, core::ptr::addr_of!(aviW) as LPCWSTR);
    if hrsrc.is_null() {
        return FALSE;
    }

    (*infoPtr).hRes = LoadResource(hInst, hrsrc);
    if (*infoPtr).hRes.is_null() {
        return FALSE;
    }

    lpAvi = LockResource((*infoPtr).hRes) as LPVOID;
    if lpAvi.is_null() {
        return FALSE;
    }

    memset(
        core::ptr::addr_of_mut!(mminfo) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<MMIOINFO>() as size_t,
    );
    mminfo.fccIOProc = FOURCC_MEM;
    mminfo.pchBuffer = lpAvi as HPSTR;
    mminfo.cchBuffer = SizeofResource(hInst, hrsrc) as LONG;
    (*infoPtr).hMMio = mmioOpenW(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(mminfo),
        MMIO_READ,
    );
    if (*infoPtr).hMMio.is_null() {
        FreeResource((*infoPtr).hRes);
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn ANIMATE_LoadFileW(infoPtr: *mut ANIMATE_INFO, lpName: LPWSTR) -> BOOL {
    (*infoPtr).hMMio = mmioOpenW(lpName, core::ptr::null_mut(), (MMIO_ALLOCBUF | MMIO_READ | MMIO_DENYWRITE) as DWORD);

    if (*infoPtr).hMMio.is_null() {
        return FALSE;
    }
    return TRUE;
}

unsafe extern "C" fn ANIMATE_DoStop(infoPtr: *mut ANIMATE_INFO) -> BOOL {
    let mut stopped: BOOL = FALSE;

    EnterCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));

    /* should stop playing */
    if !(*infoPtr).hThread.is_null() {
        let handle: HANDLE = (*infoPtr).hThread;

        (*infoPtr).hThread = core::ptr::null_mut();
        SetEvent((*infoPtr).hStopEvent);

        if (*infoPtr).threadId != GetCurrentThreadId() {
            LeaveCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs)); /* leave it a chance to run */
            WaitForSingleObject(handle, INFINITE);
            EnterCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
        }

        CloseHandle(handle);
        CloseHandle((*infoPtr).hStopEvent);
        (*infoPtr).hStopEvent = core::ptr::null_mut();
        stopped = TRUE;
    }
    if (*infoPtr).uTimer != 0 {
        KillTimer((*infoPtr).hwndSelf, ((*infoPtr).uTimer) as UINT_PTR);
        (*infoPtr).uTimer = 0;
        stopped = TRUE;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));

    if stopped != 0 {
        ANIMATE_Notify(infoPtr, ACN_STOP);
    }

    return TRUE;
}

unsafe extern "C" fn ANIMATE_Free(infoPtr: *mut ANIMATE_INFO) {
    if !(*infoPtr).hMMio.is_null() {
        ANIMATE_DoStop(infoPtr);
        mmioClose((*infoPtr).hMMio, 0 as UINT);
        if !(*infoPtr).hRes.is_null() {
            FreeResource((*infoPtr).hRes);
            (*infoPtr).hRes = core::ptr::null_mut();
        }
        heap_free((*infoPtr).lpIndex as *mut core::ffi::c_void);
        (*infoPtr).lpIndex = core::ptr::null_mut::<DWORD>();
        if !(*infoPtr).hic.is_null() {
            fnIC.fnICClose.unwrap_unchecked()((*infoPtr).hic);
            (*infoPtr).hic = core::ptr::null_mut();
        }
        heap_free((*infoPtr).inbih as *mut core::ffi::c_void);
        (*infoPtr).inbih = core::ptr::null_mut::<tagBITMAPINFOHEADER>();
        heap_free((*infoPtr).outbih as *mut core::ffi::c_void);
        (*infoPtr).outbih = core::ptr::null_mut::<tagBITMAPINFOHEADER>();
        heap_free((*infoPtr).indata as *mut core::ffi::c_void);
        (*infoPtr).indata = core::ptr::null_mut::<core::ffi::c_void>() as LPVOID;
        heap_free((*infoPtr).outdata as *mut core::ffi::c_void);
        (*infoPtr).outdata = core::ptr::null_mut::<core::ffi::c_void>() as LPVOID;
        if !(*infoPtr).hbmPrevFrame.is_null() {
            DeleteObject((*infoPtr).hbmPrevFrame as HGDIOBJ);
            (*infoPtr).hbmPrevFrame = core::ptr::null_mut();
        }

        memset(
            core::ptr::addr_of_mut!((*infoPtr).mah) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<MainAVIHeader>() as size_t,
        );
        memset(
            core::ptr::addr_of_mut!((*infoPtr).ash) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<AVIStreamHeader>() as size_t,
        );
        (*infoPtr).currFrame = 0;
        (*infoPtr).nLoop = (*infoPtr).currFrame;
        (*infoPtr).nToFrame = (*infoPtr).nLoop;
        (*infoPtr).nFromFrame = (*infoPtr).nToFrame;
    }
    (*infoPtr).transparentColor = ANIMATE_COLOR_NONE;
}

unsafe extern "C" fn ANIMATE_TransparentBlt(infoPtr: *const ANIMATE_INFO, hdcDest: HDC, hdcSource: HDC) {
    let mut hdcMask: HDC;
    let mut hbmMask: HBITMAP;
    let mut hbmOld: HBITMAP;

    /* create a transparency mask */
    hdcMask = CreateCompatibleDC(hdcDest);
    hbmMask = CreateBitmap((*(*infoPtr).inbih).biWidth, (*(*infoPtr).inbih).biHeight, 1, 1, core::ptr::null_mut());
    hbmOld = SelectObject(hdcMask, hbmMask as HGDIOBJ) as HBITMAP;

    SetBkColor(hdcSource, (*infoPtr).transparentColor);
    BitBlt(hdcMask, 0, 0, (*(*infoPtr).inbih).biWidth, (*(*infoPtr).inbih).biHeight, hdcSource, 0, 0, SRCCOPY);

    /* mask the source bitmap */
    SetBkColor(hdcSource, RGB!(0, 0, 0));
    SetTextColor(hdcSource, RGB!(255, 255, 255));
    BitBlt(hdcSource, 0, 0, (*(*infoPtr).inbih).biWidth, (*(*infoPtr).inbih).biHeight, hdcMask, 0, 0, SRCAND);

    /* mask the destination bitmap */
    SetBkColor(hdcDest, RGB!(255, 255, 255));
    SetTextColor(hdcDest, RGB!(0, 0, 0));
    BitBlt(hdcDest, 0, 0, (*(*infoPtr).inbih).biWidth, (*(*infoPtr).inbih).biHeight, hdcMask, 0, 0, SRCAND);

    /* combine source and destination */
    BitBlt(hdcDest, 0, 0, (*(*infoPtr).inbih).biWidth, (*(*infoPtr).inbih).biHeight, hdcSource, 0, 0, SRCPAINT);

    SelectObject(hdcMask, hbmOld as HGDIOBJ);
    DeleteObject(hbmMask as HGDIOBJ);
    DeleteDC(hdcMask);
}

unsafe extern "C" fn ANIMATE_PaintFrame(infoPtr: *mut ANIMATE_INFO, hDC: HDC) -> BOOL {
    let mut pBitmapData: *const core::ffi::c_void = core::ptr::null();
    let mut pBitmapInfo: *const BITMAPINFO = core::ptr::null();
    let mut hdcMem: HDC = core::ptr::null_mut();
    let mut hbmOld: HBITMAP = core::ptr::null_mut();
    let mut nOffsetX: core::ffi::c_int = 0;
    let mut nOffsetY: core::ffi::c_int = 0;
    let mut nWidth: core::ffi::c_int;
    let mut nHeight: core::ffi::c_int;

    if hDC.is_null() || (*infoPtr).inbih.is_null() {
        return TRUE;
    }

    if !(*infoPtr).hic.is_null() {
        pBitmapData = (*infoPtr).outdata as *const core::ffi::c_void;
        pBitmapInfo = (*infoPtr).outbih as *const BITMAPINFO;

        nWidth = (*(*infoPtr).outbih).biWidth as core::ffi::c_int;
        nHeight = (*(*infoPtr).outbih).biHeight as core::ffi::c_int;
    } else {
        pBitmapData = (*infoPtr).indata as *const core::ffi::c_void;
        pBitmapInfo = (*infoPtr).inbih as *const BITMAPINFO;

        nWidth = (*(*infoPtr).inbih).biWidth as core::ffi::c_int;
        nHeight = (*(*infoPtr).inbih).biHeight as core::ffi::c_int;
    }

    if (*infoPtr).hbmPrevFrame.is_null() {
        (*infoPtr).hbmPrevFrame = CreateCompatibleBitmap(hDC, nWidth, nHeight);
    }

    hdcMem = CreateCompatibleDC(hDC);
    hbmOld = SelectObject(hdcMem, (*infoPtr).hbmPrevFrame as HGDIOBJ) as HBITMAP;

    SetDIBits(
        hdcMem,
        (*infoPtr).hbmPrevFrame,
        0,
        nHeight as UINT,
        pBitmapData,
        pBitmapInfo,
        DIB_RGB_COLORS,
    );

    /*
     * we need to get the transparent color even without ACS_TRANSPARENT,
     * because the style can be changed later on and the color should always
     * be obtained in the first frame
     */
    if (*infoPtr).transparentColor == ANIMATE_COLOR_NONE {
        (*infoPtr).transparentColor =
            GetPixel(hdcMem, 0, 0);
    }

    if ((*infoPtr).dwStyle & ACS_TRANSPARENT) != 0 {
        let mut hdcFinal: HDC = CreateCompatibleDC(hDC);
        let mut hbmFinal: HBITMAP = CreateCompatibleBitmap(hDC, nWidth, nHeight);
        let mut hbmOld2: HBITMAP = SelectObject(hdcFinal, hbmFinal as HGDIOBJ) as HBITMAP;
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        SetRect(
            &raw mut rect,
            0,
            0,
            nWidth,
            nHeight,
        );

        if (*infoPtr).hbrushBG.is_null() {
            (*infoPtr).hbrushBG =
                GetCurrentObject(hDC, OBJ_BRUSH) as HBRUSH;
        }

        FillRect(hdcFinal, &raw mut rect as LPCRECT, (*infoPtr).hbrushBG);
        ANIMATE_TransparentBlt(infoPtr, hdcFinal, hdcMem);

        SelectObject(hdcFinal, hbmOld2 as HGDIOBJ);
        SelectObject(hdcMem, hbmFinal as HGDIOBJ);
        DeleteDC(hdcFinal);
        DeleteObject((*infoPtr).hbmPrevFrame as HGDIOBJ);
        (*infoPtr).hbmPrevFrame = hbmFinal;
    }

    if ((*infoPtr).dwStyle & ACS_CENTER) != 0 {
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        GetWindowRect((*infoPtr).hwndSelf, &raw mut rect);
        nOffsetX = ((rect.right as core::ffi::c_int - rect.left as core::ffi::c_int) - nWidth)
            / 2;
        nOffsetY = ((rect.bottom as core::ffi::c_int - rect.top as core::ffi::c_int) - nHeight)
            / 2;
    }

    BitBlt(
        hDC,
        nOffsetX,
        nOffsetY,
        nWidth,
        nHeight,
        hdcMem,
        0,
        0,
        SRCCOPY,
    );

    SelectObject(hdcMem, hbmOld as HGDIOBJ);
    DeleteDC(hdcMem);
    return TRUE;
}

unsafe extern "C" fn ANIMATE_DrawFrame(mut infoPtr: *mut ANIMATE_INFO, mut hDC: HDC) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/animate.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ANIMATE_DrawFrame\0" as *const u8 as *const ::core::ffi::c_char,
            342 as ::core::ffi::c_int,
            b"Drawing frame %d (loop %d)\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).currFrame,
            (*infoPtr).nLoop,
        );
    }
    mmioSeek(
        (*infoPtr).hMMio,
        *(*infoPtr).lpIndex.offset((*infoPtr).currFrame as isize) as LONG,
        SEEK_SET as _,
    );
    mmioRead(
        (*infoPtr).hMMio,
        (*infoPtr).indata as HPSTR,
        (*infoPtr).ash.dwSuggestedBufferSize as LONG,
    );
    if !(*infoPtr).hic.is_null()
        && fnIC.fnICDecompress.unwrap_unchecked()(
            (*infoPtr).hic,
            0 as DWORD,
            (*infoPtr).inbih,
            (*infoPtr).indata,
            (*infoPtr).outbih,
            (*infoPtr).outdata,
        ) != ICERR_OK as DWORD
    {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_WARN as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_WARN;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/comctl32/animate.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ANIMATE_DrawFrame\0" as *const u8 as *const ::core::ffi::c_char,
                350 as ::core::ffi::c_int,
                b"Decompression error\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return FALSE;
    }
    ANIMATE_PaintFrame(infoPtr, hDC);
    let fresh15 = (*infoPtr).currFrame;
    (*infoPtr).currFrame = (*infoPtr).currFrame + 1;
    if fresh15 >= (*infoPtr).nToFrame {
        (*infoPtr).currFrame = (*infoPtr).nFromFrame;
        if (*infoPtr).nLoop != -1 {
            (*infoPtr).nLoop -= 1;
            if (*infoPtr).nLoop == 0 {
                ANIMATE_DoStop(infoPtr);
            }
        }
    }
    return TRUE;
}

unsafe extern "C" fn ANIMATE_Timer(infoPtr: *mut ANIMATE_INFO) -> LRESULT {
    let mut hDC: HDC = core::ptr::null_mut::<HDC__>();

    hDC = GetDC((*infoPtr).hwndSelf);
    if hDC != 0 as HDC {
        EnterCriticalSection(&raw mut (*infoPtr).cs);
        ANIMATE_DrawFrame(infoPtr, hDC);
        LeaveCriticalSection(&raw mut (*infoPtr).cs);

        ReleaseDC((*infoPtr).hwndSelf, hDC);
    }

    return 0 as LRESULT;
}

unsafe extern "system" fn ANIMATE_AnimationThread(ptr_: LPVOID) -> DWORD {
    let infoPtr: *mut ANIMATE_INFO = ptr_ as *mut ANIMATE_INFO;
    let mut event: HANDLE = core::ptr::null_mut();
    let mut timeout: DWORD;

    while 1 != 0 {
        let hDC: HDC = GetDC((*infoPtr).hwndSelf);

        EnterCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
        ANIMATE_DrawFrame(infoPtr, hDC);
        timeout = (*infoPtr).mah.dwMicroSecPerFrame;
        event = (*infoPtr).hStopEvent;
        LeaveCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));

        ReleaseDC((*infoPtr).hwndSelf, hDC);

        /* time is in microseconds, we should convert it to milliseconds */
        if (event == core::ptr::null_mut())
            || WaitForSingleObject(
                event,
                timeout
                    .wrapping_add(500 as DWORD)
                    .wrapping_div(1000 as DWORD),
            ) == WAIT_OBJECT_0 as DWORD
        {
            break;
        }
    }
    return (TRUE) as DWORD;
}

unsafe extern "C" fn ANIMATE_Play(
    mut infoPtr: *mut ANIMATE_INFO,
    mut cRepeat: UINT,
    mut wFrom: WORD,
    mut wTo: WORD,
) -> LRESULT {
    /* nothing opened */
    if (*infoPtr).hMMio.is_null() {
        return FALSE as LRESULT;
    }

    if !(*infoPtr).hThread.is_null() || (*infoPtr).uTimer != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl,
                __dbch,
                b"dll/win32/comctl32/animate.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ANIMATE_Play\0" as *const u8 as *const ::core::ffi::c_char,
                416 as ::core::ffi::c_int,
                b"Already playing\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return TRUE as LRESULT;
    }

    (*infoPtr).nFromFrame = wFrom as ::core::ffi::c_int;
    (*infoPtr).nToFrame = wTo as ::core::ffi::c_int;
    (*infoPtr).nLoop = cRepeat as ::core::ffi::c_int;

    if (*infoPtr).nToFrame == 0xFFFF {
        (*infoPtr).nToFrame =
            (*infoPtr).mah.dwTotalFrames.wrapping_sub(1 as DWORD) as ::core::ffi::c_int;
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
            b"dll/win32/comctl32/animate.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ANIMATE_Play\0" as *const u8 as *const ::core::ffi::c_char,
            428 as ::core::ffi::c_int,
            b"(repeat=%d from=%d to=%d);\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*infoPtr).nLoop,
            (*infoPtr).nFromFrame,
            (*infoPtr).nToFrame,
        );
    }

    if (*infoPtr).nFromFrame as DWORD >= (*infoPtr).mah.dwTotalFrames
        && ((*infoPtr).nFromFrame as SHORT as ::core::ffi::c_int) < 0
    {
        (*infoPtr).nFromFrame = 0;
    }

    if (*infoPtr).nFromFrame > (*infoPtr).nToFrame
        || (*infoPtr).nToFrame as DWORD >= (*infoPtr).mah.dwTotalFrames
    {
        return FALSE as LRESULT;
    }

    (*infoPtr).currFrame = (*infoPtr).nFromFrame;

    /* seek - doesn't need to start a thread or set a timer and neither
     * does it send a notification */
    if (*infoPtr).nFromFrame == (*infoPtr).nToFrame {
        let mut hDC: HDC = core::ptr::null_mut();

        if {
            hDC = GetDC((*infoPtr).hwndSelf);
            hDC
        } != core::ptr::null_mut()
        {
            ANIMATE_DrawFrame(infoPtr, hDC);

            ReleaseDC((*infoPtr).hwndSelf, hDC);
        }
        return TRUE as LRESULT;
    }

    if (*infoPtr).dwStyle & ACS_TIMER as DWORD != 0 {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_1: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_1: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_1,
                __dbch_1,
                b"dll/win32/comctl32/animate.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ANIMATE_Play\0" as *const u8 as *const ::core::ffi::c_char,
                457 as ::core::ffi::c_int,
                b"Using a timer\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        /* create a timer to display AVI */
        (*infoPtr).uTimer = SetTimer(
            (*infoPtr).hwndSelf,
            1,
            ((*infoPtr).mah.dwMicroSecPerFrame as UINT).wrapping_div(1000 as UINT),
            None,
        ) as UINT;
    } else {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_2: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_2: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_2,
                __dbch_2,
                b"dll/win32/comctl32/animate.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ANIMATE_Play\0" as *const u8 as *const ::core::ffi::c_char,
                464 as ::core::ffi::c_int,
                b"Using an animation thread\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        (*infoPtr).hStopEvent = CreateEventW(
            core::ptr::null_mut(),
            TRUE,
            FALSE,
            core::ptr::null(),
        );
        (*infoPtr).hThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(ANIMATE_AnimationThread),
            infoPtr as PVOID,
            0,
            core::ptr::addr_of_mut!((*infoPtr).threadId),
        );
        if (*infoPtr).hThread.is_null() {
            return FALSE as LRESULT;
        }
    }

    ANIMATE_Notify(infoPtr, ACN_START as UINT);

    return TRUE as LRESULT;
}

unsafe extern "C" fn ANIMATE_GetAviInfo(infoPtr: *mut ANIMATE_INFO) -> BOOL {
    let __file: *const i8 = b"dll/win32/comctl32/animate.c\0".as_ptr() as *const i8;
    let __func: *const i8 = b"ANIMATE_GetAviInfo\0".as_ptr() as *const i8;
    let mut ckMainRIFF: MMCKINFO = MMCKINFO {
        ckid: 0,
        cksize: 0,
        fccType: 0,
        dwDataOffset: 0,
        dwFlags: 0,
    };
    let mut mmckHead: MMCKINFO = MMCKINFO {
        ckid: 0,
        cksize: 0,
        fccType: 0,
        dwDataOffset: 0,
        dwFlags: 0,
    };
    let mut mmckList: MMCKINFO = MMCKINFO {
        ckid: 0,
        cksize: 0,
        fccType: 0,
        dwDataOffset: 0,
        dwFlags: 0,
    };
    let mut mmckInfo: MMCKINFO = MMCKINFO {
        ckid: 0,
        cksize: 0,
        fccType: 0,
        dwDataOffset: 0,
        dwFlags: 0,
    };
    let mut numFrame: DWORD = 0;
    let mut insize: DWORD = 0;

    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(ckMainRIFF), core::ptr::null_mut::<MMCKINFO>(), 0) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 488, b"Can't find 'RIFF' chunk\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }
    if (ckMainRIFF.ckid != FOURCC_RIFF)
        || (ckMainRIFF.fccType != mmioFOURCC!('A' as i32, 'V' as i32, 'I' as i32, ' ' as i32))
    {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 494, b"Can't find 'AVI ' chunk\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmckHead.fccType = mmioFOURCC!('h' as i32, 'd' as i32, 'r' as i32, 'l' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckHead), core::ptr::addr_of_mut!(ckMainRIFF), MMIO_FINDLIST) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 500, b"Can't find 'hdrl' list\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmckInfo.ckid = mmioFOURCC!('a' as i32, 'v' as i32, 'i' as i32, 'h' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), core::ptr::addr_of_mut!(mmckHead), MMIO_FINDCHUNK) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 506, b"Can't find 'avih' chunk\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmioRead((*infoPtr).hMMio, core::ptr::addr_of_mut!((*infoPtr).mah) as LPSTR, core::mem::size_of::<MainAVIHeader>() as LONG);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 512, b"mah.dwMicroSecPerFrame=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwMicroSecPerFrame);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 513, b"mah.dwMaxBytesPerSec=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwMaxBytesPerSec);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 514, b"mah.dwPaddingGranularity=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwPaddingGranularity);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 515, b"mah.dwFlags=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwFlags);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 516, b"mah.dwTotalFrames=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwTotalFrames);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 517, b"mah.dwInitialFrames=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwInitialFrames);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 518, b"mah.dwStreams=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwStreams);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 519, b"mah.dwSuggestedBufferSize=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwSuggestedBufferSize);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 520, b"mah.dwWidth=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwWidth);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 521, b"mah.dwHeight=%d\n\0".as_ptr() as *const i8, (*infoPtr).mah.dwHeight);
    }

    mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), 0);

    mmckList.fccType = mmioFOURCC!('s' as i32, 't' as i32, 'r' as i32, 'l' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckList), core::ptr::addr_of_mut!(mmckHead), MMIO_FINDLIST) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 527, b"Can't find 'strl' list\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmckInfo.ckid = mmioFOURCC!('s' as i32, 't' as i32, 'r' as i32, 'h' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), core::ptr::addr_of_mut!(mmckList), MMIO_FINDCHUNK) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 533, b"Can't find 'strh' chunk\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmioRead((*infoPtr).hMMio, core::ptr::addr_of_mut!((*infoPtr).ash) as LPSTR, core::mem::size_of::<AVIStreamHeader>() as LONG);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 542,
            b"ash.fccType='%c%c%c%c'\n\0".as_ptr() as *const i8,
            ((((*infoPtr).ash.fccType as DWORD_PTR) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccType as DWORD_PTR) >> 8) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccType as DWORD_PTR) >> 16) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccType as DWORD_PTR) >> 24) & 0xff as DWORD_PTR) as BYTE) as i32,
        );
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 546,
            b"ash.fccHandler='%c%c%c%c'\n\0".as_ptr() as *const i8,
            ((((*infoPtr).ash.fccHandler as DWORD_PTR) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccHandler as DWORD_PTR) >> 8) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccHandler as DWORD_PTR) >> 16) & 0xff as DWORD_PTR) as BYTE) as i32,
            (((((*infoPtr).ash.fccHandler as DWORD_PTR) >> 24) & 0xff as DWORD_PTR) as BYTE) as i32,
        );
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 547, b"ash.dwFlags=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwFlags);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 548, b"ash.wPriority=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.wPriority as i32);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 549, b"ash.wLanguage=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.wLanguage as i32);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 550, b"ash.dwInitialFrames=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwInitialFrames);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 551, b"ash.dwScale=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwScale);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 552, b"ash.dwRate=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwRate);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 553, b"ash.dwStart=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwStart);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 554, b"ash.dwLength=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwLength);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 555, b"ash.dwSuggestedBufferSize=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwSuggestedBufferSize);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 556, b"ash.dwQuality=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwQuality);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 557, b"ash.dwSampleSize=%d\n\0".as_ptr() as *const i8, (*infoPtr).ash.dwSampleSize);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 559,
            b"ash.rcFrame=(%d,%d,%d,%d)\n\0".as_ptr() as *const i8,
            (*infoPtr).ash.rcFrame.top as i32,
            (*infoPtr).ash.rcFrame.left as i32,
            (*infoPtr).ash.rcFrame.bottom as i32,
            (*infoPtr).ash.rcFrame.right as i32,
        );
    }

    mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), 0);

    mmckInfo.ckid = mmioFOURCC!('s' as i32, 't' as i32, 'r' as i32, 'f' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), core::ptr::addr_of_mut!(mmckList), MMIO_FINDCHUNK) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 565, b"Can't find 'strh' chunk\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    (*infoPtr).inbih = heap_alloc_zero(mmckInfo.cksize as SIZE_T) as LPBITMAPINFOHEADER;
    if (*infoPtr).inbih.is_null() {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 571, b"Can't alloc input BIH\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    mmioRead((*infoPtr).hMMio, (*infoPtr).inbih as LPSTR, mmckInfo.cksize as LONG);

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 577, b"bih.biSize=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biSize);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 578, b"bih.biWidth=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biWidth);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 579, b"bih.biHeight=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biHeight);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 580, b"bih.biPlanes=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biPlanes as i32);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 581, b"bih.biBitCount=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biBitCount as i32);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 582, b"bih.biCompression=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biCompression);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 583, b"bih.biSizeImage=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biSizeImage);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 584, b"bih.biXPelsPerMeter=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biXPelsPerMeter);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 585, b"bih.biYPelsPerMeter=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biYPelsPerMeter);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 586, b"bih.biClrUsed=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biClrUsed);
    }
    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32)) != 0 {
        ros_dbg_log(__WINE_DBCL_TRACE, __wine_dbch___default, __file, __func, 587, b"bih.biClrImportant=%d\n\0".as_ptr() as *const i8, (*(*infoPtr).inbih).biClrImportant);
    }

    mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), 0);
    mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckList), 0);
    mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckHead), 0);

    mmckList.fccType = mmioFOURCC!('m' as i32, 'o' as i32, 'v' as i32, 'i' as i32);
    if mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckList), core::ptr::addr_of_mut!(ckMainRIFF), MMIO_FINDLIST) != 0 {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 610, b"Can't find 'movi' list\n\0".as_ptr() as *const i8);
        }
        return FALSE;
    }

    (*infoPtr).lpIndex = heap_alloc_zero(((*infoPtr).mah.dwTotalFrames as SIZE_T).wrapping_mul(core::mem::size_of::<DWORD>() as SIZE_T)) as LPDWORD;
    if (*infoPtr).lpIndex.is_null() {
        return FALSE;
    }

    insize = 0;
    numFrame = insize;
    while mmioDescend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), core::ptr::addr_of_mut!(mmckList), 0) == 0
        && numFrame < (*infoPtr).mah.dwTotalFrames
    {
        *(*infoPtr).lpIndex.offset(numFrame as isize) = mmckInfo.dwDataOffset;
        if insize < mmckInfo.cksize {
            insize = mmckInfo.cksize;
        }
        numFrame = numFrame.wrapping_add(1);
        mmioAscend((*infoPtr).hMMio, core::ptr::addr_of_mut!(mmckInfo), 0);
    }
    if numFrame != (*infoPtr).mah.dwTotalFrames {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 630, b"Found %d frames (/%d)\n\0".as_ptr() as *const i8, numFrame, (*infoPtr).mah.dwTotalFrames);
        }
        return FALSE;
    }
    if insize > (*infoPtr).ash.dwSuggestedBufferSize {
        if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_WARN as i32)) != 0 {
            ros_dbg_log(__WINE_DBCL_WARN, __wine_dbch___default, __file, __func, 634, b"insize=%d suggestedSize=%d\n\0".as_ptr() as *const i8, insize, (*infoPtr).ash.dwSuggestedBufferSize);
        }
        (*infoPtr).ash.dwSuggestedBufferSize = insize;
    }

    (*infoPtr).indata = heap_alloc_zero((*infoPtr).ash.dwSuggestedBufferSize as SIZE_T) as LPVOID;
    if (*infoPtr).indata.is_null() {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn ANIMATE_GetAviCodec(infoPtr: *mut ANIMATE_INFO) -> BOOL {
    let mut outSize: DWORD;

    /* check uncompressed AVI */
    if ((*infoPtr).ash.fccHandler == mmioFOURCC!('D' as i32, 'I' as i32, 'B' as i32, ' ' as i32))
        || ((*infoPtr).ash.fccHandler == mmioFOURCC!('R' as i32, 'L' as i32, 'E' as i32, ' ' as i32))
        || ((*infoPtr).ash.fccHandler
            == mmioFOURCC!(
                0,
                0,
                0,
                0,
            ))
    {
        (*infoPtr).hic = core::ptr::null_mut();
        return TRUE;
    }

    /* try to get a decompressor for that type */
    (*infoPtr).hic = fnIC.fnICOpen.unwrap_unchecked()(
        ICTYPE_VIDEO,
        (*infoPtr).ash.fccHandler as DWORD,
        ICMODE_DECOMPRESS as UINT,
    );
    if (*infoPtr).hic.is_null() {
        return FALSE;
    }

    outSize = fnIC.fnICSendMessage.unwrap_unchecked()(
        (*infoPtr).hic,
        ICM_DECOMPRESS_GET_FORMAT as UINT,
        (*infoPtr).inbih as DWORD_PTR,
        0 as DWORD_PTR,
    ) as DWORD;

    (*infoPtr).outbih = heap_alloc_zero(outSize as SIZE_T) as LPBITMAPINFOHEADER;
    if (*infoPtr).outbih.is_null() {
        return FALSE;
    }

    if fnIC.fnICSendMessage.unwrap_unchecked()(
        (*infoPtr).hic,
        ICM_DECOMPRESS_GET_FORMAT as UINT,
        (*infoPtr).inbih as DWORD_PTR,
        (*infoPtr).outbih as DWORD_PTR,
    ) != ICERR_OK as LRESULT
    {
        return FALSE;
    }

    (*infoPtr).outdata =
        heap_alloc_zero((*(*infoPtr).outbih).biSizeImage as SIZE_T) as LPVOID;
    if (*infoPtr).outdata.is_null() {
        return FALSE;
    }

    if fnIC.fnICSendMessage.unwrap_unchecked()(
        (*infoPtr).hic,
        ICM_DECOMPRESS_BEGIN as UINT,
        (*infoPtr).inbih as DWORD_PTR,
        (*infoPtr).outbih as DWORD_PTR,
    ) != ICERR_OK as LRESULT
    {
        return FALSE;
    }

    return TRUE;
}

unsafe extern "C" fn ANIMATE_OpenW(
    infoPtr: *mut ANIMATE_INFO,
    mut hInstance: HINSTANCE,
    mut lpszName: LPWSTR,
) -> BOOL {
    let mut hdc: HDC = core::ptr::null_mut();

    ANIMATE_Free(infoPtr);

    if lpszName.is_null() {
        /* installer of thebat! v1.62 requires FALSE here */
        return ((*infoPtr).hMMio != core::ptr::null_mut()) as BOOL;
    }

    if hInstance.is_null() {
        hInstance = GetWindowLongPtrW((*infoPtr).hwndSelf, GWLP_HINSTANCE) as HINSTANCE;
    }

    if IS_INTRESOURCE!(lpszName) == false {
        if ANIMATE_LoadResW(infoPtr, hInstance, lpszName) == 0 {
            if ANIMATE_LoadFileW(infoPtr, lpszName) == 0 {
                return FALSE;
            }
        }
    } else {
        if ANIMATE_LoadResW(infoPtr, hInstance, lpszName) == 0 {
            return FALSE;
        }
    }

    if ANIMATE_GetAviInfo(infoPtr) == 0 {
        ANIMATE_Free(infoPtr);
        return FALSE;
    }

    if ANIMATE_GetAviCodec(infoPtr) == 0 {
        ANIMATE_Free(infoPtr);
        return FALSE;
    }

    hdc = GetDC((*infoPtr).hwndSelf);
    /* native looks at the top left pixel of the first frame here too. */
    (*infoPtr).hbrushBG = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_CTLCOLORSTATIC,
        hdc as WPARAM,
        (*infoPtr).hwndSelf as LPARAM,
    ) as HBRUSH;
    ReleaseDC((*infoPtr).hwndSelf, hdc);

    if ((*infoPtr).dwStyle & (ACS_CENTER as DWORD)) == 0 {
        SetWindowPos(
            (*infoPtr).hwndSelf,
            core::ptr::null_mut(),
            0,
            0,
            ((*infoPtr).mah.dwWidth) as i32,
            ((*infoPtr).mah.dwHeight) as i32,
            (SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOZORDER) as UINT,
        );
    }

    if ((*infoPtr).dwStyle & (ACS_AUTOPLAY as DWORD)) != 0 {
        return ANIMATE_Play(
            infoPtr,
            (1 as u32).wrapping_neg() as UINT,
            0 as WORD,
            ((*infoPtr).mah.dwTotalFrames.wrapping_sub(1 as DWORD)) as WORD,
        ) as BOOL;
    }

    return TRUE;
}

unsafe extern "C" fn ANIMATE_OpenA(
    mut infoPtr: *mut ANIMATE_INFO,
    mut hInstance: HINSTANCE,
    mut lpszName: LPSTR,
) -> BOOL {
    let mut lpwszName: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut result: LRESULT = 0;
    let mut len: INT = 0;
    if IS_INTRESOURCE!(lpszName) != false {
        return ANIMATE_OpenW(infoPtr, hInstance, lpszName as LPWSTR);
    }
    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszName as LPCSTR,
        -(1 as core::ffi::c_int),
        core::ptr::null_mut::<WCHAR>(),
        0 as core::ffi::c_int,
    ) as INT;
    lpwszName = heap_alloc(
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if lpwszName.is_null() {
        return FALSE as BOOL;
    }
    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        lpszName as LPCSTR,
        -(1 as core::ffi::c_int),
        lpwszName,
        len as core::ffi::c_int,
    );
    result = ANIMATE_OpenW(infoPtr, hInstance, lpwszName) as LRESULT;
    heap_free(lpwszName as *mut core::ffi::c_void);
    return result as BOOL;
}

unsafe extern "C" fn ANIMATE_Stop(infoPtr: *mut ANIMATE_INFO) -> BOOL {
    /* nothing opened */
    if (*infoPtr).hMMio.is_null() {
        return FALSE;
    }

    ANIMATE_DoStop(infoPtr);
    return TRUE;
}

unsafe extern "C" fn ANIMATE_Create(hWnd: HWND, lpcs: *const CREATESTRUCTW) -> BOOL {
    static msvfw32W: [WCHAR; 12] = [
    0x006Du16, 0x0073, 0x0076, 0x0066, 0x0077, 0x0033, 0x0032, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];
    let mut infoPtr: *mut ANIMATE_INFO;

    if fnIC.hModule.is_null() {
        fnIC.hModule = LoadLibraryW(core::ptr::addr_of!(msvfw32W) as LPCWSTR) as HMODULE;
        if fnIC.hModule.is_null() {
            return FALSE;
        }

        fnIC.fnICOpen = core::mem::transmute::<
            *mut core::ffi::c_void,
            Option<unsafe extern "C" fn(DWORD, DWORD, UINT) -> HIC>,
        >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress(
                fnIC.hModule as HINSTANCE,
                b"ICOpen\0" as *const u8 as LPCSTR,
            ),
        ));
        fnIC.fnICClose = core::mem::transmute::<
            *mut core::ffi::c_void,
            Option<unsafe extern "C" fn(HIC) -> LRESULT>,
        >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress(
                fnIC.hModule as HINSTANCE,
                b"ICClose\0" as *const u8 as LPCSTR,
            ),
        ));
        fnIC.fnICSendMessage = core::mem::transmute::<
            *mut core::ffi::c_void,
            Option<unsafe extern "C" fn(HIC, UINT, DWORD_PTR, DWORD_PTR) -> LRESULT>,
        >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress(
                fnIC.hModule as HINSTANCE,
                b"ICSendMessage\0" as *const u8 as LPCSTR,
            ),
        ));
        fnIC.fnICDecompress = core::mem::transmute::<
            *mut core::ffi::c_void,
            Option<
                unsafe extern "C" fn(
                    HIC,
                    DWORD,
                    LPBITMAPINFOHEADER,
                    LPVOID,
                    LPBITMAPINFOHEADER,
                    LPVOID,
                ) -> DWORD,
            >,
        >(core::mem::transmute::<FARPROC, *mut core::ffi::c_void>(
            GetProcAddress(
                fnIC.hModule as HINSTANCE,
                b"ICDecompress\0" as *const u8 as LPCSTR,
            ),
        ));
    }

    /* allocate memory for info structure */
    infoPtr = heap_alloc_zero(core::mem::size_of::<ANIMATE_INFO>() as SIZE_T) as *mut ANIMATE_INFO;
    if infoPtr.is_null() {
        return FALSE;
    }

    /* store crossref hWnd <-> info structure */
    SetWindowLongPtrW(hWnd, 0, infoPtr as DWORD_PTR as LONG_PTR);
    (*infoPtr).hwndSelf = hWnd;
    (*infoPtr).hwndNotify = (*lpcs).hwndParent;
    (*infoPtr).transparentColor = ANIMATE_COLOR_NONE;
    (*infoPtr).hbmPrevFrame = core::ptr::null_mut();
    (*infoPtr).dwStyle = (*lpcs).style as DWORD;

    InitializeCriticalSection(&raw mut (*infoPtr).cs);
    (*(*infoPtr).cs.DebugInfo).__bindgen_anon_1.Spare[0 as usize] =
        concat!(file!(), ": ANIMATE_INFO*->cs\0").as_ptr() as DWORD_PTR;

    return TRUE;
}

unsafe extern "C" fn ANIMATE_Destroy(infoPtr: *mut ANIMATE_INFO) -> LRESULT {
    /* free avi data */
    ANIMATE_Free(infoPtr);

    /* free animate info data */
    SetWindowLongPtrW((*infoPtr).hwndSelf, 0, 0);

    (*(*infoPtr).cs.DebugInfo).__bindgen_anon_1.Spare[0] = 0;
    DeleteCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
    heap_free(infoPtr as *mut core::ffi::c_void);

    return 0;
}

unsafe extern "C" fn ANIMATE_EraseBackground(infoPtr: *const ANIMATE_INFO, hdc: HDC) -> BOOL {
    let mut rect: RECT = core::mem::zeroed();
    let mut hBrush: HBRUSH = core::ptr::null_mut();

    hBrush = SendMessageW(
        (*infoPtr).hwndNotify,
        WM_CTLCOLORSTATIC,
        hdc as WPARAM,
        (*infoPtr).hwndSelf as LPARAM,
    ) as HBRUSH;
    GetClientRect((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(rect));
    FillRect(
        hdc,
        core::ptr::addr_of_mut!(rect),
        if !hBrush.is_null() {
            hBrush
        } else {
            GetCurrentObject(hdc, OBJ_BRUSH) as HBRUSH
        },
    );

    return TRUE;
}

unsafe extern "C" fn ANIMATE_StyleChanged(infoPtr: *mut ANIMATE_INFO, wStyleType: WPARAM, lpss: *const STYLESTRUCT) -> LRESULT {
    if wStyleType != ((GWL_STYLE) as u32) as _ { return 0; }

    (*infoPtr).dwStyle = (*lpss).styleNew;

    InvalidateRect((*infoPtr).hwndSelf, core::ptr::null_mut(), TRUE);
    return 0;
}

unsafe extern "system" fn ANIMATE_WindowProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    let mut infoPtr: *mut ANIMATE_INFO = GetWindowLongPtrW(hWnd, 0) as *mut ANIMATE_INFO;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/comctl32/animate.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ANIMATE_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
            880 as ::core::ffi::c_int,
            b"hwnd=%p msg=%x wparam=%lx lparam=%lx\n\0" as *const u8
                as *const ::core::ffi::c_char,
            hWnd,
            uMsg,
            wParam,
            lParam,
        );
    }
    if infoPtr.is_null() && uMsg != WM_NCCREATE {
        return DefWindowProcW(hWnd, uMsg, wParam, lParam);
    }
    match uMsg {
        m if m == ACM_OPENA => {
            return ANIMATE_OpenA(infoPtr, wParam as HINSTANCE, lParam as LPSTR) as LRESULT;
        }
        m if m == ACM_OPENW => {
            return ANIMATE_OpenW(infoPtr, wParam as HINSTANCE, lParam as LPWSTR) as LRESULT;
        }
        m if m == ACM_PLAY => {
            return ANIMATE_Play(
                infoPtr,
                wParam as INT as UINT,
                LOWORD!(lParam as DWORD_PTR as DWORD),
                HIWORD!(lParam as DWORD_PTR as DWORD),
            );
        }
        m if m == ACM_STOP => {
            return ANIMATE_Stop(infoPtr) as LRESULT;
        }
        m if m == WM_CLOSE => {
            ANIMATE_Free(infoPtr);
            return 0;
        }
        m if m == WM_NCCREATE => {
            return ANIMATE_Create(hWnd, lParam as *const CREATESTRUCTW) as LRESULT;
        }
        m if m == WM_NCHITTEST => {
            return HTTRANSPARENT as LRESULT;
        }
        m if m == WM_DESTROY => {
            return ANIMATE_Destroy(infoPtr);
        }
        m if m == WM_ERASEBKGND => {
            return ANIMATE_EraseBackground(infoPtr, wParam as HDC) as LRESULT;
        }
        m if m == WM_STYLECHANGED => {
            return ANIMATE_StyleChanged(infoPtr, wParam, lParam as LPSTYLESTRUCT as *const STYLESTRUCT);
        }
        m if m == WM_TIMER => {
            return ANIMATE_Timer(infoPtr);
        }
        m if m == WM_PRINTCLIENT || m == WM_PAINT => {
            // the animation has not decompressed (and displayed) the first frame yet, don't paint
            if (*infoPtr).hbmPrevFrame.is_null() {
                // default paint handling
                return DefWindowProcW(hWnd, uMsg, wParam, lParam);
            }

            if wParam != 0 {
                EnterCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
                ANIMATE_PaintFrame(infoPtr, wParam as HDC);
                LeaveCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
            } else {
                let mut ps: PAINTSTRUCT = core::mem::zeroed();
                let hDC: HDC = BeginPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));

                EnterCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));
                ANIMATE_PaintFrame(infoPtr, hDC);
                LeaveCriticalSection(core::ptr::addr_of_mut!((*infoPtr).cs));

                EndPaint((*infoPtr).hwndSelf, core::ptr::addr_of_mut!(ps));
            }
        }
        m if m == WM_SIZE => {
            if ((*infoPtr).dwStyle & ACS_CENTER) != 0 {
                InvalidateRect((*infoPtr).hwndSelf, core::ptr::null::<RECT>(), TRUE);
            }
            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
        _ => {
            if uMsg >= WM_USER
                && uMsg < WM_APP
                && COMCTL32_IsReflectedMessage(uMsg) == 0
            {
                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                    & (1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int
                    != 0
                {
                    let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
                    let __dbcl_0: __wine_debug_class = __WINE_DBCL_ERR;
                    ros_dbg_log(
                        __dbcl_0,
                        __dbch_0,
                        b"dll/win32/comctl32/animate.c\0"
                            as *const u8 as *const ::core::ffi::c_char,
                        b"ANIMATE_WindowProc\0" as *const u8 as *const ::core::ffi::c_char,
                        958 as ::core::ffi::c_int,
                        b"unknown msg %04x wp=%08lx lp=%08lx\n\0" as *const u8
                            as *const ::core::ffi::c_char,
                        uMsg,
                        wParam,
                        lParam,
                    );
                }
            }

            return DefWindowProcW(hWnd, uMsg, wParam, lParam);
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn ANIMATE_Register() {
    let mut wndClass: WNDCLASSW = core::mem::zeroed();

    ZeroMemory!(
        &raw mut wndClass as *mut core::ffi::c_void,
        core::mem::size_of::<WNDCLASSW>(),
    );
    wndClass.style = (CS_GLOBALCLASS | CS_DBLCLKS) as UINT;
    wndClass.lpfnWndProc =
        Some(ANIMATE_WindowProc);
    wndClass.cbClsExtra = 0 as core::ffi::c_int;
    wndClass.cbWndExtra =
        core::mem::size_of::<*mut ANIMATE_INFO>() as core::ffi::c_int;
    wndClass.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPCWSTR,
    );
    wndClass.hbrBackground =
        (COLOR_BTNFACE + 1) as HBRUSH;
    wndClass.lpszClassName = (ANIMATE_CLASSW).as_ptr();

    RegisterClassW(&raw mut wndClass);
}
#[no_mangle]
pub unsafe extern "C" fn ANIMATE_Unregister() {
    UnregisterClassW((ANIMATE_CLASSW).as_ptr(), core::ptr::null_mut());
}
