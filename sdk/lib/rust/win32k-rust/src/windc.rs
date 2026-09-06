#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion, PsGetCurrentProcessWin32Process};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, KeBugCheckEx};
use ntkrnl_sys::base::{
    STATUS_SUCCESS, InitializeListHead, InsertTailList, InsertHeadList, RemoveEntryList,
};
#[allow(unused_imports)]
use core::ffi::c_void;

// CONTAINING_RECORD(addr, type, field) — offset_of! based, like sibling crates.
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $type:ty, $field:ident) => {
        ($addr as *mut u8).offset(-(core::mem::offset_of!($type, $field) as isize)) as *mut $type
    };
}

// USERTAG_DCE pool tag. C multi-char literal 'cdsU' is big-endian; the LE
// byte order is the literal reversed -> "Usdc".
const USERTAG_DCE: u32 = u32::from_le_bytes(*b"Usdc");

// USER_VERSION / VIDEO_DRIVER_INIT_FAILURE — win32k-private constants
// (ntuser.h: USER_VERSION = MAKELONG(0,5); VIDEO_DRIVER_INIT_FAILURE bugcheck).
const USER_VERSION: u32 = 0x0005_0000;
const VIDEO_DRIVER_INIT_FAILURE: u32 = 0xB4;

// HRGN_WINDOW = (HRGN)1 (ntuser.h macro).
#[inline(always)]
fn HRGN_WINDOW() -> HANDLE {
    1usize as HANDLE
}

// IntSysCreateRectpRgnIndirect(prc) (region.h macro) -> IntSysCreateRectpRgn(...).
#[inline(always)]
unsafe fn IntSysCreateRectpRgnIndirect(prc: *const RECTL) -> PREGION {
    IntSysCreateRectpRgn((*prc).left, (*prc).top, (*prc).right, (*prc).bottom)
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// GetW32ThreadInfo — exported NTAPI (`_GetW32ThreadInfo`, cdecl on i386).
// DCU_SetDcUndeletable — FASTCALL (`@DCU_SetDcUndeletable@4`).
extern "C" {
    fn GetW32ThreadInfo() -> PTHREADINFO;
}
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DCU_SetDcUndeletable(hDC: HDC);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DCU_SetDcUndeletable(hDC: HDC);
}

// VIS_ComputeVisibleRegion — FASTCALL (`@VIS_ComputeVisibleRegion@16`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn VIS_ComputeVisibleRegion( Window: PWND, ClientArea: u8, ClipChildren: u8, ClipSiblings: u8, ) -> PREGION;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn VIS_ComputeVisibleRegion( Window: PWND, ClientArea: u8, ClipChildren: u8, ClipSiblings: u8, ) -> PREGION;
}

// ---- file-scope preamble (translated) ----
static mut LEDce: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
static mut DCECount: INT = 0;

pub const DCX_CACHECOMPAREMASK: u32 = DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN |
                                      DCX_NORESETATTRS | DCX_LOCKWINDOWUPDATE |
                                      DCX_LAYEREDWIN | DCX_CACHE | DCX_WINDOW |
                                      DCX_PARENTCLIP;

unsafe extern "C" fn DceUpdateVisRgn_impl(Dce: *mut DCE, Window: PWND, Flags: ULONG) {
    let mut RgnVisible: PREGION = core::ptr::null_mut();
    let mut DcxFlags: ULONG;
    let DesktopWindow: PWND;

    'noparent: {
        if (Flags & DCX_PARENTCLIP) != 0 {
            let Parent: PWND = (*Window).spwndParent;
            if Parent.is_null() {
                RgnVisible = core::ptr::null_mut();
                break 'noparent;
            }

            if ((*Parent).style & WS_CLIPSIBLINGS) != 0 {
                DcxFlags = DCX_CLIPSIBLINGS | (Flags & !(DCX_CLIPCHILDREN | DCX_WINDOW));
            } else {
                DcxFlags = Flags & !(DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN | DCX_WINDOW);
            }
            RgnVisible = DceGetVisRgn(Parent, DcxFlags, UserHMGetHandle(Window), Flags);
        } else if Window.is_null() {
            DesktopWindow = UserGetWindowObject(IntGetDesktopWindow());
            if !DesktopWindow.is_null() {
                RgnVisible = IntSysCreateRectpRgnIndirect(core::ptr::addr_of_mut!((*DesktopWindow).rcWindow));
            } else {
                RgnVisible = core::ptr::null_mut();
            }
        } else {
            RgnVisible = DceGetVisRgn(Window, Flags, 0 as HWND, 0);
        }
    }

    if (Flags & DCX_INTERSECTRGN) != 0 {
        let mut RgnClip: PREGION = core::ptr::null_mut();

        if !(*Dce).hrgnClip.is_null() {
            RgnClip = REGION_LockRgn((*Dce).hrgnClip);
        }

        if !RgnClip.is_null() {
            IntGdiCombineRgn(RgnVisible, RgnVisible, RgnClip, RGN_AND as INT);
            REGION_UnlockRgn(RgnClip);
        } else {
            if !RgnVisible.is_null() {
                REGION_Delete(RgnVisible);
            }
            RgnVisible = IntSysCreateRectpRgn(0, 0, 0, 0);
        }
    } else if (Flags & DCX_EXCLUDERGN) != 0 && !(*Dce).hrgnClip.is_null() {
        let RgnClip: PREGION = REGION_LockRgn((*Dce).hrgnClip);
        IntGdiCombineRgn(RgnVisible, RgnVisible, RgnClip, RGN_DIFF as INT);
        REGION_UnlockRgn(RgnClip);
    }

    (*Dce).DCXFlags &= !DCX_DCEDIRTY;
    GdiSelectVisRgn((*Dce).hDC, RgnVisible);

    if !Window.is_null() {
        IntEngWindowChanged(Window, WOC_RGN_CLIENT);
    }

    if !RgnVisible.is_null() {
        REGION_Delete(RgnVisible);
    }
}

#[no_mangle]
pub unsafe extern "system" fn InitDCEImpl() -> NTSTATUS {
    InitializeListHead(core::ptr::addr_of_mut!(LEDce) as *mut _);
    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceCreateDisplayDC() -> HDC {
    DceCreateDisplayDC_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceCreateDisplayDC() -> HDC {
    DceCreateDisplayDC_impl()
}

unsafe fn DceCreateDisplayDC_impl() -> HDC {
    static DISPLAY_NAME: [u16; 8] = [
        b'D' as u16, b'I' as u16, b'S' as u16, b'P' as u16,
        b'L' as u16, b'A' as u16, b'Y' as u16, 0,
    ];

    let mut DriverName: _UNICODE_STRING = core::mem::zeroed();
    DriverName.Length = (7 * core::mem::size_of::<u16>()) as u16;
    DriverName.MaximumLength = (8 * core::mem::size_of::<u16>()) as u16;
    DriverName.Buffer = DISPLAY_NAME.as_ptr() as *mut u16;

    if co_IntGraphicsCheck(1) == 0 {
        KeBugCheckEx(
            VIDEO_DRIVER_INIT_FAILURE,
            0,
            0,
            0,
            USER_VERSION as _,
        );
    }

    IntGdiCreateDC(
        &mut DriverName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    )
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn DceGetVisRgn(
    Window: PWND,
    Flags: ULONG,
    hWndChild: HWND,
    CFlags: ULONG,
) -> PREGION {
    DceGetVisRgn_impl(Window, Flags, hWndChild, CFlags)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn DceGetVisRgn(
    Window: PWND,
    Flags: ULONG,
    hWndChild: HWND,
    CFlags: ULONG,
) -> PREGION {
    DceGetVisRgn_impl(Window, Flags, hWndChild, CFlags)
}

unsafe fn DceGetVisRgn_impl(
    Window: PWND,
    Flags: ULONG,
    hWndChild: HWND,
    CFlags: ULONG,
) -> PREGION {

    let mut Rgn: PREGION = VIS_ComputeVisibleRegion(
        Window,
        (0 == (Flags & DCX_WINDOW)) as u8,
        (0 != (Flags & DCX_CLIPCHILDREN)) as u8,
        (0 != (Flags & DCX_CLIPSIBLINGS)) as u8,
    );

    if Rgn.is_null() {
        Rgn = IntSysCreateRectpRgn(0, 0, 0, 0);
    }
    Rgn
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceAllocDCE(Window: PWND, Type: DCE_TYPE) -> PDCE {
    DceAllocDCE_impl(Window, Type)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceAllocDCE(Window: PWND, Type: DCE_TYPE) -> PDCE {
    DceAllocDCE_impl(Window, Type)
}

unsafe fn DceAllocDCE_impl(Window: PWND, Type: DCE_TYPE) -> PDCE {
    let pDce: PDCE = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<DCE>() as _,
        USERTAG_DCE,
    ) as PDCE;
    if pDce.is_null() {
        return core::ptr::null_mut();
    }

    (*pDce).hDC = DceCreateDisplayDC();
    if (*pDce).hDC.is_null() {
        ExFreePoolWithTag(pDce as *mut core::ffi::c_void, USERTAG_DCE);
        return core::ptr::null_mut();
    }
    DCECount += 1;
    TRACE!(b"Alloc DCE's! %d\n\0".as_ptr() as PCHAR, DCECount);
    (*pDce).hwndCurrent = if !Window.is_null() {
        UserHMGetHandle(Window)
    } else {
        core::ptr::null_mut()
    };
    (*pDce).pwndOrg = Window;
    (*pDce).pwndClip = Window;
    (*pDce).hrgnClip = core::ptr::null_mut();
    (*pDce).hrgnClipPublic = core::ptr::null_mut();
    (*pDce).hrgnSavedVis = core::ptr::null_mut();
    (*pDce).ppiOwner = core::ptr::null_mut();

    InsertTailList(core::ptr::addr_of_mut!(LEDce) as *mut _, core::ptr::addr_of_mut!((*pDce).List) as *mut _);

    DCU_SetDcUndeletable((*pDce).hDC);

    if Type == DCE_WINDOW_DC || Type == DCE_CLASS_DC {
        (*pDce).ptiOwner = GetW32ThreadInfo();
    } else {
        TRACE!(
            b"FREE DCATTR!!!! NOT DCE_WINDOW_DC!!!!! hDC-> %p\n\0".as_ptr() as PCHAR,
            (*pDce).hDC,
        );
        GreSetDCOwner((*pDce).hDC, GDI_OBJ_HMGR_NONE);
        (*pDce).ptiOwner = core::ptr::null_mut();
    }

    if Type == DCE_CACHE_DC {
        (*pDce).DCXFlags = DCX_CACHE | DCX_DCEEMPTY;
    } else {
        (*pDce).DCXFlags = DCX_DCEBUSY;
        if !Window.is_null() {
            if Type == DCE_WINDOW_DC {
                if (*Window).style & WS_CLIPCHILDREN as u32 != 0 {
                    (*pDce).DCXFlags |= DCX_CLIPCHILDREN;
                }
                if (*Window).style & WS_CLIPSIBLINGS as u32 != 0 {
                    (*pDce).DCXFlags |= DCX_CLIPSIBLINGS;
                }
            }
        }
    }
    pDce
}

unsafe extern "system" fn DceSetDrawable(Window: PWND, hDC: HDC, Flags: ULONG, SetClipOrigin: BOOL) {
    let mut rect: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };

    if !Window.is_null() {
        if Flags & DCX_WINDOW != 0 {
            rect = (*Window).rcWindow;
        } else {
            rect = (*Window).rcClient;
        }
    }

    GreSetDCOrg(hDC, rect.left, rect.top, core::ptr::addr_of_mut!(rect));
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DceDeleteClipRgn(Dce: *mut DCE) {
    DceDeleteClipRgn_impl(Dce)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DceDeleteClipRgn(Dce: *mut DCE) {
    DceDeleteClipRgn_impl(Dce)
}

unsafe fn DceDeleteClipRgn_impl(Dce: *mut DCE) {
    (*Dce).DCXFlags &= !(DCX_EXCLUDERGN | DCX_INTERSECTRGN);

    if (*Dce).DCXFlags & DCX_KEEPCLIPRGN != 0 {
        (*Dce).DCXFlags &= !DCX_KEEPCLIPRGN;
    } else if !(*Dce).hrgnClip.is_null() {
        GreDeleteObject((*Dce).hrgnClip as HGDIOBJ);
    }

    (*Dce).hrgnClip = core::ptr::null_mut();

    (*Dce).DCXFlags |= DCX_DCEDIRTY;
    IntGdiSetHookFlags((*Dce).hDC, DCHF_INVALIDATEVISRGN as WORD);
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn DceUpdateVisRgn(Dce: *mut DCE, Window: PWND, Flags: ULONG) {
    DceUpdateVisRgn_impl(Dce, Window, Flags)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn DceUpdateVisRgn(Dce: *mut DCE, Window: PWND, Flags: ULONG) {
    DceUpdateVisRgn_impl(Dce, Window, Flags)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DceReleaseDC(dce: *mut DCE, EndPaint: BOOL) -> INT {
    DceReleaseDC_impl(dce, EndPaint)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DceReleaseDC(dce: *mut DCE, EndPaint: BOOL) -> INT {
    DceReleaseDC_impl(dce, EndPaint)
}

unsafe fn DceReleaseDC_impl(dce: *mut DCE, EndPaint: BOOL) -> INT {
    if DCX_DCEBUSY != ((*dce).DCXFlags & (DCX_INDESTROY | DCX_DCEEMPTY | DCX_DCEBUSY)) {
        return 0;
    }

    if EndPaint != 0 {
        DceUpdateVisRgn(dce, (*dce).pwndOrg, (*dce).DCXFlags);
    }

    if ((*dce).DCXFlags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN)) != 0
        && (((*dce).DCXFlags & DCX_CACHE) != 0 || EndPaint != 0)
    {
        DceDeleteClipRgn(dce);
    }

    if (*dce).DCXFlags & DCX_CACHE != 0 {
        if (*dce).DCXFlags & DCX_NORESETATTRS == 0 {
            if IntGdiCleanDC((*dce).hDC) == 0 {
                return 0;
            }

            if (*dce).DCXFlags & DCX_DCEDIRTY != 0 {
                (*dce).hwndCurrent = core::ptr::null_mut();
                (*dce).pwndOrg = core::ptr::null_mut();
                (*dce).pwndClip = core::ptr::null_mut();
                (*dce).DCXFlags &= DCX_CACHE;
                (*dce).DCXFlags |= DCX_DCEEMPTY;
            }
        }
        (*dce).DCXFlags &= !DCX_DCEBUSY;
        TRACE!(b"Exit!!!!! DCX_CACHE!!!!!!   hDC-> %p \n\0".as_ptr() as PCHAR, (*dce).hDC);
        if GreSetDCOwner((*dce).hDC, GDI_OBJ_HMGR_NONE) == 0 {
            return 0;
        }
        (*dce).ptiOwner = core::ptr::null_mut();
        (*dce).ppiOwner = core::ptr::null_mut();
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetDCEx(Wnd: PWND, ClipRegion: HANDLE, Flags: ULONG) -> HDC {
    UserGetDCEx_impl(Wnd, ClipRegion, Flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetDCEx(Wnd: PWND, ClipRegion: HANDLE, Flags: ULONG) -> HDC {
    UserGetDCEx_impl(Wnd, ClipRegion, Flags)
}

unsafe fn UserGetDCEx_impl(Wnd: PWND, mut ClipRegion: HANDLE, mut Flags: ULONG) -> HDC {
    let Parent: PWND;
    let DcxFlags: ULONG;
    let mut Dce: *mut DCE = core::ptr::null_mut();
    let mut UpdateClipOrigin: BOOL = 0;
    let mut bUpdateVisRgn: BOOL = 1;
    let mut hDC: HDC = core::ptr::null_mut();
    let ppi: PPROCESSINFO;
    let mut ListEntry: PLIST_ENTRY;

    if Wnd.is_null() {
        Flags &= !DCX_USESTYLE;
        Flags |= DCX_CACHE;
    }

    if Flags & DCX_PARENTCLIP != 0 {
        Flags |= DCX_CACHE;
    }

    if Flags & DCX_USESTYLE != 0 {
        Flags &= !(DCX_CLIPCHILDREN | DCX_CLIPSIBLINGS | DCX_PARENTCLIP);
        if Flags & DCX_WINDOW == 0 {
            if (*(*Wnd).pcls).style & CS_PARENTDC != 0 {
                Flags |= DCX_PARENTCLIP;
            }

            if Flags & DCX_CACHE == 0 && (*(*Wnd).pcls).style & CS_OWNDC == 0 {
                if (*(*Wnd).pcls).style & CS_CLASSDC == 0 {
                    Flags |= DCX_CACHE;
                } else {
                    if !(*(*Wnd).pcls).pdce.is_null() {
                        hDC = (*((*(*Wnd).pcls).pdce as PDCE)).hDC;
                    }
                    TRACE!(b"We have CLASS!!\n\0".as_ptr() as PCHAR);
                }
            }

            if (*Wnd).style & WS_CLIPSIBLINGS != 0 {
                Flags |= DCX_CLIPSIBLINGS;
            }

            if (*Wnd).style & WS_CLIPCHILDREN != 0 && (*Wnd).style & WS_MINIMIZE == 0 {
                Flags |= DCX_CLIPCHILDREN;
            }

            if (*Wnd).style & WS_MINIMIZE != 0 && !(*(*Wnd).pcls).spicn.is_null() {
                Flags |= DCX_CACHE;
            }
        } else {
            if (*Wnd).style & WS_CLIPSIBLINGS != 0 {
                Flags |= DCX_CLIPSIBLINGS;
            }
            Flags |= DCX_CACHE;
        }
    }

    if Flags & DCX_WINDOW != 0 {
        Flags &= !DCX_CLIPCHILDREN;
    }

    if Flags & DCX_NOCLIPCHILDREN != 0 {
        Flags |= DCX_CACHE;
        Flags &= !(DCX_PARENTCLIP | DCX_CLIPCHILDREN);
    }

    Parent = if !Wnd.is_null() { (*Wnd).spwndParent } else { core::ptr::null_mut() };

    if Wnd.is_null() || (*Wnd).style & WS_CHILD == 0 || Parent.is_null() {
        Flags &= !DCX_PARENTCLIP;
        Flags |= DCX_CLIPSIBLINGS;
    }

    if Flags & (DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN) != 0 {
        Flags &= !DCX_PARENTCLIP;
    }

    if Flags & DCX_PARENTCLIP != 0 {
        if (*Wnd).style & WS_VISIBLE != 0 && (*Parent).style & WS_VISIBLE != 0 {
            Flags &= !DCX_CLIPCHILDREN;
            if (*Parent).style & WS_CLIPSIBLINGS != 0 {
                Flags |= DCX_CLIPSIBLINGS;
            }
        }
    }

    if Flags & DCX_CACHE == 0 {
        // head.pti: THRDESKHEAD bindgen-flattens the embedded THROBJHEAD; pti
        // sits right after HEAD {HANDLE h; DWORD cLockObj} = 2 pointer-widths in.
        let head_pti = *((core::ptr::addr_of!((*Wnd).head) as *const u8)
            .add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO);
        if head_pti != GetW32ThreadInfo() {
            Flags |= DCX_CACHE;
        }
    }

    DcxFlags = Flags & DCX_CACHECOMPAREMASK;

    if Flags & DCX_CACHE != 0 {
        let mut DceEmpty: *mut DCE = core::ptr::null_mut();
        let mut DceUnused: *mut DCE = core::ptr::null_mut();
        KeEnterCriticalRegion();
        ListEntry = LEDce.Flink;
        while ListEntry != core::ptr::addr_of_mut!(LEDce) {
            Dce = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(DCE, List) as isize)) as *mut DCE;
            ListEntry = (*ListEntry).Flink;

            if (*Dce).DCXFlags & (DCX_CACHE | DCX_DCEBUSY) == DCX_CACHE {
                DceUnused = Dce;
                if (*Dce).DCXFlags & DCX_DCEEMPTY != 0 {
                    DceEmpty = Dce;
                } else if (*Dce).hwndCurrent == (if !Wnd.is_null() { UserHMGetHandle(Wnd) } else { core::ptr::null_mut() })
                    && ((*Dce).DCXFlags & DCX_CACHECOMPAREMASK) == DcxFlags
                {
                    UpdateClipOrigin = 1;
                    break;
                }
            }
            Dce = core::ptr::null_mut();
        }
        KeLeaveCriticalRegion();

        Dce = if DceEmpty.is_null() { DceUnused } else { DceEmpty };

        if Dce.is_null() {
            Dce = DceAllocDCE(core::ptr::null_mut(), DCE_CACHE_DC);
        }
        if Dce.is_null() {
            return core::ptr::null_mut();
        }

        (*Dce).hwndCurrent = if !Wnd.is_null() { UserHMGetHandle(Wnd) } else { core::ptr::null_mut() };
        (*Dce).pwndClip = Wnd;
        (*Dce).pwndOrg = Wnd;
    } else {
        KeEnterCriticalRegion();
        ListEntry = LEDce.Flink;
        while ListEntry != core::ptr::addr_of_mut!(LEDce) {
            Dce = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(DCE, List) as isize)) as *mut DCE;
            ListEntry = (*ListEntry).Flink;

            if (*Dce).DCXFlags & DCX_CACHE == 0 {
                if (*Dce).hwndCurrent == UserHMGetHandle(Wnd) {
                    bUpdateVisRgn = 0;
                    break;
                } else if (*Dce).hDC == hDC {
                    break;
                }
            }
            Dce = core::ptr::null_mut();
        }
        KeLeaveCriticalRegion();

        if Dce.is_null() {
            return core::ptr::null_mut();
        }

        if Flags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN) != 0
            && (*Dce).DCXFlags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN) != 0
        {
            DceDeleteClipRgn(Dce);
        }
    }

    if Dce.is_null() {
        return core::ptr::null_mut();
    }

    if GreIsHandleValid((*Dce).hDC as HGDIOBJ) == 0 {
        ERR!("FIXME: Got DCE with invalid hDC! %p\n", (*Dce).hDC);
        (*Dce).hDC = DceCreateDisplayDC();
    }

    (*Dce).DCXFlags = Flags | DCX_DCEBUSY;

    RemoveEntryList(core::ptr::addr_of_mut!((*Dce).List) as *mut _);
    InsertHeadList(core::ptr::addr_of_mut!(LEDce) as *mut _, core::ptr::addr_of_mut!((*Dce).List) as *mut _);

    if Flags & DCX_INTERSECTUPDATE != 0 && ClipRegion.is_null() {
        Flags |= DCX_INTERSECTRGN | DCX_KEEPCLIPRGN;
        (*Dce).DCXFlags |= DCX_INTERSECTRGN | DCX_KEEPCLIPRGN;
        ClipRegion = (*Wnd).hrgnUpdate as HANDLE;
        bUpdateVisRgn = 1;
    }

    if ClipRegion == HRGN_WINDOW() {
        if Flags & DCX_WINDOW == 0 {
            (*Dce).hrgnClip = NtGdiCreateRectRgn(
                (*Wnd).rcClient.left,
                (*Wnd).rcClient.top,
                (*Wnd).rcClient.right,
                (*Wnd).rcClient.bottom,
            );
        } else {
            (*Dce).hrgnClip = NtGdiCreateRectRgn(
                (*Wnd).rcWindow.left,
                (*Wnd).rcWindow.top,
                (*Wnd).rcWindow.right,
                (*Wnd).rcWindow.bottom,
            );
        }
        (*Dce).DCXFlags &= !DCX_KEEPCLIPRGN;
        bUpdateVisRgn = 1;
    } else if !ClipRegion.is_null() {
        if !(*Dce).hrgnClip.is_null() {
            ERR!("Should not be called!!\n");
            GreDeleteObject((*Dce).hrgnClip as HGDIOBJ);
            (*Dce).hrgnClip = core::ptr::null_mut();
        }
        (*Dce).hrgnClip = ClipRegion as HRGN;
        bUpdateVisRgn = 1;
    }

    if IntGdiSetHookFlags((*Dce).hDC, DCHF_VALIDATEVISRGN as WORD) != 0 {
        bUpdateVisRgn = 1;
    }

    DceSetDrawable(Wnd, (*Dce).hDC, Flags, UpdateClipOrigin);

    if bUpdateVisRgn != 0 {
        DceUpdateVisRgn(Dce, Wnd, Flags);
    }

    if (*Dce).DCXFlags & DCX_CACHE != 0 {
        TRACE!(b"ENTER!!!!!! DCX_CACHE!!!!!!   hDC-> %p\n\0".as_ptr() as PCHAR, (*Dce).hDC);
        GreSetDCOwner((*Dce).hDC, GDI_OBJ_HMGR_POWNED);
        (*Dce).ptiOwner = GetW32ThreadInfo();
    }

    if !Wnd.is_null() && (*Wnd).ExStyle & WS_EX_LAYOUTRTL != 0 && Flags & DCX_KEEPLAYOUT == 0 {
        NtGdiSetLayout((*Dce).hDC, -1, LAYOUT_RTL as ULONG);
    }

    if (*Dce).DCXFlags & DCX_PROCESSOWNED != 0 {
        ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
        // W32PF_flags: PROCESSINFO embeds W32PROCESS first; W32PF_flags follows
        // {PEPROCESS peProcess; DWORD RefCount} = pointer-width + DWORD in
        // (offset 8 i386 / 12 amd64+arm64), NOT 2*ptr (that over-counts on 64-bit).
        let w32pf = (ppi as *mut u8)
            .add(core::mem::size_of::<usize>() + core::mem::size_of::<u32>())
            as *mut u32;
        *w32pf |= W32PF_OWNDCCLEANUP;
        (*Dce).ptiOwner = core::ptr::null_mut();
        (*Dce).ppiOwner = ppi;
    }

    (*Dce).hDC
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn DceFreeDCE(pdce: PDCE, Force: BOOLEAN) {
    DceFreeDCE_impl(pdce, Force)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn DceFreeDCE(pdce: PDCE, Force: BOOLEAN) {
    DceFreeDCE_impl(pdce, Force)
}

unsafe fn DceFreeDCE_impl(pdce: PDCE, Force: BOOLEAN) {
    let mut Hit: u8 = 0u8;

    if pdce.is_null() {
        return;
    }

    (*pdce).DCXFlags |= DCX_INDESTROY;

    if Force != 0u8 && GreGetObjectOwner((*pdce).hDC as HGDIOBJ) != GDI_OBJ_HMGR_POWNED {
        if GreIsHandleValid((*pdce).hDC as HGDIOBJ) != 0 {
            GreSetDCOwner((*pdce).hDC, GDI_OBJ_HMGR_POWNED);
        } else {
            Hit = 1u8;
        }
    } else {
        if GreGetObjectOwner((*pdce).hDC as HGDIOBJ) == GDI_OBJ_HMGR_PUBLIC {
            GreSetDCOwner((*pdce).hDC, GDI_OBJ_HMGR_POWNED);
        }
    }

    if Hit == 0u8 {
        IntGdiDeleteDC((*pdce).hDC, 1);
    }

    if !(*pdce).hrgnClip.is_null() && ((*pdce).DCXFlags & DCX_KEEPCLIPRGN) == 0 {
        GreDeleteObject((*pdce).hrgnClip as HGDIOBJ);
        (*pdce).hrgnClip = core::ptr::null_mut();
    }

    RemoveEntryList(core::ptr::addr_of_mut!((*pdce).List) as *mut _);

    ExFreePoolWithTag(pdce as *mut core::ffi::c_void, USERTAG_DCE);

    DCECount -= 1;
}

// DceFreeWindowDCE — FASTCALL (`@DceFreeWindowDCE@4`), called by window.c.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceFreeWindowDCE(Window: PWND) {
    DceFreeWindowDCE_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceFreeWindowDCE(Window: PWND) {
    DceFreeWindowDCE_impl(Window)
}

unsafe fn DceFreeWindowDCE_impl(Window: PWND) {
    let mut pDCE: PDCE;
    let mut ListEntry: PLIST_ENTRY;

    if DCECount <= 0 {
        ERR!("FreeWindowDCE No Entry! %d\n", DCECount);
        return;
    }

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        pDCE = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(DCE, List) as isize)) as PDCE;
        ListEntry = (*ListEntry).Flink;
        if (*pDCE).hwndCurrent == UserHMGetHandle(Window)
            && ((*pDCE).DCXFlags & DCX_DCEEMPTY) == 0
        {
            if ((*pDCE).DCXFlags & DCX_CACHE) == 0 {
                if ((*(*Window).pcls).style & CS_CLASSDC) != 0 {
                    if ((*pDCE).DCXFlags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN)) != 0 {
                        DceDeleteClipRgn(pDCE);
                    }
                    DceUpdateVisRgn(pDCE, Window, (*pDCE).DCXFlags);
                    (*pDCE).DCXFlags = DCX_DCEEMPTY | DCX_CACHE;
                    (*pDCE).hwndCurrent = core::ptr::null_mut();
                    (*pDCE).pwndClip = core::ptr::null_mut();
                    (*pDCE).pwndOrg = (*pDCE).pwndClip;

                    TRACE!(
                        b"POWNED DCE going Cheap!! DCX_CACHE!! hDC-> %p \n\0".as_ptr() as PCHAR,
                        (*pDCE).hDC,
                    );
                    if GreSetDCOwner((*pDCE).hDC, GDI_OBJ_HMGR_NONE) == 0 {
                        ERR!("Fail Owner Switch hDC-> %p \n", (*pDCE).hDC);
                        break;
                    }
                } else if ((*(*Window).pcls).style & CS_OWNDC) != 0 {
                    DceFreeDCE(pDCE, 0u8);
                    continue;
                } else {
                    ERR!(
                        "Not POWNED or CLASSDC hwndCurrent -> %p \n",
                        (*pDCE).hwndCurrent,
                    );
                }
            } else {
                if ((*pDCE).DCXFlags & DCX_DCEBUSY) != 0 {
                    ERR!(
                        "[%p] GetDC() without ReleaseDC()!\n",
                        UserHMGetHandle(Window),
                    );
                    DceReleaseDC(pDCE, 0);
                }
                (*pDCE).DCXFlags |= DCX_DCEEMPTY;
                (*pDCE).hwndCurrent = core::ptr::null_mut();
                (*pDCE).pwndClip = core::ptr::null_mut();
                (*pDCE).pwndOrg = (*pDCE).pwndClip;
            }
        }
    }
}

// DceFreeClassDCE — FASTCALL (`@DceFreeClassDCE@4`), called by class.c.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceFreeClassDCE(pdceClass: PDCE) {
    DceFreeClassDCE_impl(pdceClass)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceFreeClassDCE(pdceClass: PDCE) {
    DceFreeClassDCE_impl(pdceClass)
}

unsafe fn DceFreeClassDCE_impl(pdceClass: PDCE) {
    let mut pDCE: PDCE;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        pDCE = CONTAINING_RECORD!(ListEntry, DCE, List) as PDCE;
        ListEntry = (*ListEntry).Flink;
        if pDCE == pdceClass {
            DceFreeDCE(pDCE, 1u8);
        }
    }
}

// DceFreeThreadDCE — FASTCALL (`@DceFreeThreadDCE@4`), called by main.c.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceFreeThreadDCE(pti: PTHREADINFO) {
    DceFreeThreadDCE_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceFreeThreadDCE(pti: PTHREADINFO) {
    DceFreeThreadDCE_impl(pti)
}

unsafe fn DceFreeThreadDCE_impl(pti: PTHREADINFO) {
    let mut pDCE: PDCE;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        pDCE = CONTAINING_RECORD!(ListEntry, DCE, List);
        ListEntry = (*ListEntry).Flink;
        if (*pDCE).ptiOwner == pti {
            if (*pDCE).DCXFlags & DCX_CACHE != 0 {
                DceFreeDCE(pDCE, 1u8);
            }
        }
    }
}

#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn DceEmptyCache() {
    DceEmptyCache_impl()
}

#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn DceEmptyCache() {
    DceEmptyCache_impl()
}

unsafe fn DceEmptyCache_impl() {
    let mut pDCE: PDCE;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        pDCE = CONTAINING_RECORD!(ListEntry, DCE, List) as PDCE;
        ListEntry = (*ListEntry).Flink;
        DceFreeDCE(pDCE, 1u8);
    }
}

// DceResetActiveDCEs — FASTCALL (`@DceResetActiveDCEs@4`), called by window.c/winpos.c.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn DceResetActiveDCEs(Window: PWND) {
    DceResetActiveDCEs_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn DceResetActiveDCEs(Window: PWND) {
    DceResetActiveDCEs_impl(Window)
}

unsafe fn DceResetActiveDCEs_impl(Window: PWND) {
    let mut pDCE: *mut DCE;
    let mut dc: PDC;
    let mut CurrentWindow: PWND;
    let mut DeltaX: INT;
    let mut DeltaY: INT;
    let mut ListEntry: PLIST_ENTRY;

    if Window.is_null() {
        return;
    }

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        pDCE = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(DCE, List) as isize)) as *mut DCE;
        ListEntry = (*ListEntry).Flink;
        if ((*pDCE).DCXFlags & (DCX_DCEEMPTY | DCX_INDESTROY)) == 0 {
            if UserHMGetHandle(Window as *mut core::ffi::c_void) == (*pDCE).hwndCurrent {
                CurrentWindow = Window;
            } else {
                if (*pDCE).hwndCurrent.is_null() {
                    CurrentWindow = core::ptr::null_mut();
                } else {
                    CurrentWindow = UserGetWindowObject((*pDCE).hwndCurrent);
                }
                if CurrentWindow.is_null() {
                    continue;
                }
            }

            if GreIsHandleValid((*pDCE).hDC as HGDIOBJ) == 0 {
                continue;
            }
            dc = DC_LockDc((*pDCE).hDC);
            if dc.is_null() {
                continue;
            }

            if Window == CurrentWindow || IntIsChildWindow(Window, CurrentWindow) != 0 {
                if ((*pDCE).DCXFlags & DCX_WINDOW) != 0 {
                    DeltaX = (*CurrentWindow).rcWindow.left - (*dc).ptlDCOrig.x;
                    DeltaY = (*CurrentWindow).rcWindow.top - (*dc).ptlDCOrig.y;
                    (*dc).ptlDCOrig.x = (*CurrentWindow).rcWindow.left;
                    (*dc).ptlDCOrig.y = (*CurrentWindow).rcWindow.top;
                } else {
                    DeltaX = (*CurrentWindow).rcClient.left - (*dc).ptlDCOrig.x;
                    DeltaY = (*CurrentWindow).rcClient.top - (*dc).ptlDCOrig.y;
                    (*dc).ptlDCOrig.x = (*CurrentWindow).rcClient.left;
                    (*dc).ptlDCOrig.y = (*CurrentWindow).rcClient.top;
                }

                if !(*dc).dclevel.prgnClip.is_null() {
                    REGION_bOffsetRgn((*dc).dclevel.prgnClip, DeltaX, DeltaY);
                    (*dc).fs |= DC_DIRTY_RAO as INT;
                }
                if !(*pDCE).hrgnClip.is_null() {
                    NtGdiOffsetRgn((*pDCE).hrgnClip, DeltaX, DeltaY);
                }
            }
            DC_UnlockDc(dc);

            DceUpdateVisRgn(pDCE, CurrentWindow, (*pDCE).DCXFlags);
            IntGdiSetHookFlags((*pDCE).hDC, DCHF_VALIDATEVISRGN as WORD);
        }
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntWindowFromDC(hDc: HDC) -> HWND {
    IntWindowFromDC_impl(hDc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntWindowFromDC(hDc: HDC) -> HWND {
    IntWindowFromDC_impl(hDc)
}

unsafe fn IntWindowFromDC_impl(hDc: HDC) -> HWND {
    let mut Ret: HWND = core::ptr::null_mut();

    let mut ListEntry: PLIST_ENTRY = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        let Dce: *mut DCE = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(DCE, List) as isize))
            as *mut DCE;
        ListEntry = (*ListEntry).Flink;
        if (*Dce).hDC == hDc {
            if (*Dce).DCXFlags & DCX_INDESTROY != 0 {
                Ret = core::ptr::null_mut();
            } else {
                Ret = (*Dce).hwndCurrent;
            }
            break;
        }
    }
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserReleaseDC(Window: PWND, hDc: HDC, EndPaint: BOOL) -> INT {
    UserReleaseDC_impl(Window, hDc, EndPaint)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserReleaseDC(Window: PWND, hDc: HDC, EndPaint: BOOL) -> INT {
    UserReleaseDC_impl(Window, hDc, EndPaint)
}

unsafe fn UserReleaseDC_impl(Window: PWND, hDc: HDC, EndPaint: BOOL) -> INT {
    let mut dce: PDCE = core::ptr::null_mut();
    let mut ListEntry: PLIST_ENTRY;
    let mut nRet: INT = 0;
    let mut Hit: BOOL = 0;

    ListEntry = LEDce.Flink;
    while ListEntry != core::ptr::addr_of_mut!(LEDce) {
        dce = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(DCE, List) as isize)) as PDCE;
        ListEntry = (*ListEntry).Flink;
        if (*dce).hDC == hDc {
            Hit = 1;
            break;
        }
    }

    if Hit != 0 && ((*dce).DCXFlags & DCX_DCEBUSY) != 0 {
        nRet = DceReleaseDC(dce, EndPaint);
    }

    nRet
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetWindowDC(Wnd: PWND) -> HDC {
    UserGetDCEx(Wnd, core::ptr::null_mut(), DCX_USESTYLE | DCX_WINDOW)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetWindowDC(Wnd: PWND) -> HDC {
    UserGetDCEx(Wnd, core::ptr::null_mut(), DCX_USESTYLE | DCX_WINDOW)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGethWnd(hdc: HDC, pwndo: *mut PWNDOBJ) -> HWND {
    UserGethWnd_impl(hdc, pwndo)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGethWnd(hdc: HDC, pwndo: *mut PWNDOBJ) -> HWND {
    UserGethWnd_impl(hdc, pwndo)
}

unsafe fn UserGethWnd_impl(hdc: HDC, pwndo: *mut PWNDOBJ) -> HWND {
    let mut Clip: *mut EWNDOBJ;
    let Wnd: PWND;
    let hWnd: HWND;

    hWnd = IntWindowFromDC(hdc);

    if !hWnd.is_null() {
        Wnd = UserGetWindowObject(hWnd);
        if !Wnd.is_null() {
            Clip = UserGetProp(Wnd, AtomWndObj, 1u8) as *mut EWNDOBJ;

            if !Clip.is_null() && (*Clip).Hwnd == hWnd {
                if !pwndo.is_null() {
                    *pwndo = Clip as PWNDOBJ;
                }
            }
        }
    }
    hWnd
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetDCEx(hWnd: HWND, ClipRegion: HANDLE, Flags: ULONG) -> HDC {
    let mut Wnd: PWND = core::ptr::null_mut();
    let mut Ret: HDC = core::ptr::null_mut();

    UserEnterExclusive();

    'block: {
        if !hWnd.is_null() {
            Wnd = UserGetWindowObject(hWnd);
            if Wnd.is_null() {
                break 'block;
            }
        }
        Ret = UserGetDCEx(Wnd, ClipRegion, Flags);
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetWindowDC(hWnd: HWND) -> HDC {
    NtUserGetDCEx(hWnd, core::ptr::null_mut(), (DCX_USESTYLE | DCX_WINDOW) as ULONG)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetDC(hWnd: HWND) -> HDC {
    NtUserGetDCEx(
        hWnd,
        core::ptr::null_mut(),
        if hWnd.is_null() {
            DCX_CACHE | DCX_WINDOW
        } else {
            DCX_USESTYLE
        },
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSelectPalette(
    hDC: HDC,
    hpal: HPALETTE,
    ForceBackground: BOOL,
) -> HPALETTE {
    let oldPal: HPALETTE;
    UserEnterExclusive();
    oldPal = GdiSelectPalette(hDC, hpal, ForceBackground);
    UserLeave();
    oldPal
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32k subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/user/ntuser/windc.c
//  * PURPOSE:         Window DC management
//  * COPYRIGHT:       Copyright 2007 ReactOS Team
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserDce);
//
// /* GLOBALS *******************************************************************/
//
// /* NOTE: I think we should store this per window station (including GDI objects) */
// /* Answer: No, use the DCE pMonitor to compare with! */
//
// static LIST_ENTRY LEDce;
// static INT DCECount = 0; // Count of DCE in system.
//
// #define DCX_CACHECOMPAREMASK (DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN | \
//                               DCX_NORESETATTRS | DCX_LOCKWINDOWUPDATE | \
//                               DCX_LAYEREDWIN | DCX_CACHE | DCX_WINDOW | \
//                               DCX_PARENTCLIP)
//
// /* FUNCTIONS *****************************************************************/
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitDCEImpl(VOID)
// {
//     InitializeListHead(&LEDce);
//     return STATUS_SUCCESS;
// }
//
// //
// // This should be moved to dc.c or dcutil.c.
// //
// HDC FASTCALL
// DceCreateDisplayDC(VOID)
// {
//   UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"DISPLAY");
//
//   if (!co_IntGraphicsCheck(TRUE))
//     KeBugCheckEx(VIDEO_DRIVER_INIT_FAILURE, 0, 0, 0, USER_VERSION);
//
//   return IntGdiCreateDC(&DriverName, NULL, NULL, NULL, FALSE);
// }
//
// /* Returns the DCE pointer from the HDC handle */
// DCE*
// FASTCALL
// DceGetDceFromDC(HDC hdc)
// {
//     PLIST_ENTRY ListEntry;
//     DCE* dce;
//
//     ListEntry = LEDce.Flink;
//     while (ListEntry != &LEDce)
//     {
//         dce = CONTAINING_RECORD(ListEntry, DCE, List);
//         ListEntry = ListEntry->Flink;
//         if (dce->hDC == hdc)
//             return dce;
//     }
//
//     return NULL;
// }
//
// static
// PREGION FASTCALL
// DceGetVisRgn(PWND Window, ULONG Flags, HWND hWndChild, ULONG CFlags)
// {
//     PREGION Rgn;
//     Rgn = VIS_ComputeVisibleRegion( Window,
//                                     0 == (Flags & DCX_WINDOW),
//                                     0 != (Flags & DCX_CLIPCHILDREN),
//                                     0 != (Flags & DCX_CLIPSIBLINGS));
//     /* Caller expects a non-null region */
//     if (!Rgn)
//         Rgn = IntSysCreateRectpRgn(0, 0, 0, 0);
//     return Rgn;
// }
//
// PDCE FASTCALL
// DceAllocDCE(PWND Window OPTIONAL, DCE_TYPE Type)
// {
//   PDCE pDce;
//
//   pDce = ExAllocatePoolWithTag(PagedPool, sizeof(DCE), USERTAG_DCE);
//   if(!pDce)
//         return NULL;
//
//   pDce->hDC = DceCreateDisplayDC();
//   if (!pDce->hDC)
//   {
//       ExFreePoolWithTag(pDce, USERTAG_DCE);
//       return NULL;
//   }
//   DCECount++;
//   TRACE("Alloc DCE's! %d\n",DCECount);
//   pDce->hwndCurrent = (Window ? UserHMGetHandle(Window) : NULL);
//   pDce->pwndOrg  = Window;
//   pDce->pwndClip = Window;
//   pDce->hrgnClip = NULL;
//   pDce->hrgnClipPublic = NULL;
//   pDce->hrgnSavedVis = NULL;
//   pDce->ppiOwner = NULL;
//
//   InsertTailList(&LEDce, &pDce->List);
//
//   DCU_SetDcUndeletable(pDce->hDC);
//
//   if (Type == DCE_WINDOW_DC || Type == DCE_CLASS_DC) // Window DCE have ownership.
//   {
//      pDce->ptiOwner = GetW32ThreadInfo();
//   }
//   else
//   {
//      TRACE("FREE DCATTR!!!! NOT DCE_WINDOW_DC!!!!! hDC-> %p\n", pDce->hDC);
//      GreSetDCOwner(pDce->hDC, GDI_OBJ_HMGR_NONE);
//      pDce->ptiOwner = NULL;
//   }
//
//   if (Type == DCE_CACHE_DC)
//   {
//      pDce->DCXFlags = DCX_CACHE | DCX_DCEEMPTY;
//   }
//   else
//   {
//      pDce->DCXFlags = DCX_DCEBUSY;
//      if (Window)
//      {
//         if (Type == DCE_WINDOW_DC)
//         {
//           if (Window->style & WS_CLIPCHILDREN) pDce->DCXFlags |= DCX_CLIPCHILDREN;
//           if (Window->style & WS_CLIPSIBLINGS) pDce->DCXFlags |= DCX_CLIPSIBLINGS;
//         }
//      }
//   }
//   return(pDce);
// }
//
// static VOID __stdcall
// DceSetDrawable( PWND Window OPTIONAL,
//                 HDC hDC,
//                 ULONG Flags,
//                 BOOL SetClipOrigin)
// {
//   RECTL rect = {0,0,0,0};
//
//   if (Window)
//   {
//       if (Flags & DCX_WINDOW)
//       {
//          rect = Window->rcWindow;
//       }
//       else
//       {
//          rect = Window->rcClient;
//       }
//   }
//
//   /* Set DC Origin and Window Rectangle */
//   GreSetDCOrg( hDC, rect.left, rect.top, &rect);
// }
//
//
// static VOID FASTCALL
// DceDeleteClipRgn(DCE* Dce)
// {
//    Dce->DCXFlags &= ~(DCX_EXCLUDERGN | DCX_INTERSECTRGN);
//
//    if (Dce->DCXFlags & DCX_KEEPCLIPRGN )
//    {
//       Dce->DCXFlags &= ~DCX_KEEPCLIPRGN;
//    }
//    else if (Dce->hrgnClip != NULL)
//    {
//       GreDeleteObject(Dce->hrgnClip);
//    }
//
//    Dce->hrgnClip = NULL;
//
//    /* Make it dirty so that the vis rgn gets recomputed next time */
//    Dce->DCXFlags |= DCX_DCEDIRTY;
//    IntGdiSetHookFlags(Dce->hDC, DCHF_INVALIDATEVISRGN);
// }
//
// VOID
// FASTCALL
// DceUpdateVisRgn(DCE *Dce, PWND Window, ULONG Flags)
// {
//    PREGION RgnVisible = NULL;
//    ULONG DcxFlags;
//    PWND DesktopWindow;
//
//    if (Flags & DCX_PARENTCLIP)
//    {
//       PWND Parent;
//
//       Parent = Window->spwndParent;
//       if (!Parent)
//       {
//          RgnVisible = NULL;
//          goto noparent;
//       }
//
//       if (Parent->style & WS_CLIPSIBLINGS)
//       {
//          DcxFlags = DCX_CLIPSIBLINGS |
//                     (Flags & ~(DCX_CLIPCHILDREN | DCX_WINDOW));
//       }
//       else
//       {
//          DcxFlags = Flags & ~(DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN | DCX_WINDOW);
//       }
//       RgnVisible = DceGetVisRgn(Parent, DcxFlags, UserHMGetHandle(Window), Flags);
//    }
//    else if (Window == NULL)
//    {
//       DesktopWindow = UserGetWindowObject(IntGetDesktopWindow());
//       if (NULL != DesktopWindow)
//       {
//          RgnVisible = IntSysCreateRectpRgnIndirect(&DesktopWindow->rcWindow);
//       }
//       else
//       {
//          RgnVisible = NULL;
//       }
//    }
//    else
//    {
//       RgnVisible = DceGetVisRgn(Window, Flags, 0, 0);
//    }
//
// noparent:
//    if (Flags & DCX_INTERSECTRGN)
//    {
//       PREGION RgnClip = NULL;
//
//       if (Dce->hrgnClip != NULL)
//           RgnClip = REGION_LockRgn(Dce->hrgnClip);
//
//       if (RgnClip)
//       {
//          IntGdiCombineRgn(RgnVisible, RgnVisible, RgnClip, RGN_AND);
//          REGION_UnlockRgn(RgnClip);
//       }
//       else
//       {
//          if (RgnVisible != NULL)
//          {
//             REGION_Delete(RgnVisible);
//          }
//          RgnVisible = IntSysCreateRectpRgn(0, 0, 0, 0);
//       }
//    }
//    else if ((Flags & DCX_EXCLUDERGN) && Dce->hrgnClip != NULL)
//    {
//        PREGION RgnClip = REGION_LockRgn(Dce->hrgnClip);
//        IntGdiCombineRgn(RgnVisible, RgnVisible, RgnClip, RGN_DIFF);
//        REGION_UnlockRgn(RgnClip);
//    }
//
//    Dce->DCXFlags &= ~DCX_DCEDIRTY;
//    GdiSelectVisRgn(Dce->hDC, RgnVisible);
//
//    /* Tell GDI driver */
//    if (Window)
//        IntEngWindowChanged(Window, WOC_RGN_CLIENT);
//
//    if (RgnVisible != NULL)
//    {
//       REGION_Delete(RgnVisible);
//    }
// }
//
// static INT FASTCALL
// DceReleaseDC(DCE* dce, BOOL EndPaint)
// {
//    if (DCX_DCEBUSY != (dce->DCXFlags & (DCX_INDESTROY | DCX_DCEEMPTY | DCX_DCEBUSY)))
//    {
//       return 0;
//    }
//
//    /* Restore previous visible region */
//    if (EndPaint)
//    {
//       DceUpdateVisRgn(dce, dce->pwndOrg, dce->DCXFlags);
//    }
//
//    if ((dce->DCXFlags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN)) &&
//          ((dce->DCXFlags & DCX_CACHE) || EndPaint))
//    {
//       DceDeleteClipRgn(dce);
//    }
//
//    if (dce->DCXFlags & DCX_CACHE)
//    {
//       if (!(dce->DCXFlags & DCX_NORESETATTRS))
//       {
//          // Clean the DC
//          if (!IntGdiCleanDC(dce->hDC)) return 0;
//
//          if (dce->DCXFlags & DCX_DCEDIRTY)
//          {
//            /* Don't keep around invalidated entries
//             * because SetDCState() disables hVisRgn updates
//             * by removing dirty bit. */
//            dce->hwndCurrent = 0;
//            dce->pwndOrg  = NULL;
//            dce->pwndClip = NULL;
//            dce->DCXFlags &= DCX_CACHE;
//            dce->DCXFlags |= DCX_DCEEMPTY;
//          }
//       }
//       dce->DCXFlags &= ~DCX_DCEBUSY;
//       TRACE("Exit!!!!! DCX_CACHE!!!!!!   hDC-> %p \n", dce->hDC);
//       if (!GreSetDCOwner(dce->hDC, GDI_OBJ_HMGR_NONE))
//          return 0;
//       dce->ptiOwner = NULL; // Reset ownership.
//       dce->ppiOwner = NULL;
//
//    }
//    return 1; // Released!
// }
//
//
// HDC FASTCALL
// UserGetDCEx(PWND Wnd OPTIONAL, HANDLE ClipRegion, ULONG Flags)
// {
//    PWND Parent;
//    ULONG DcxFlags;
//    DCE* Dce = NULL;
//    BOOL UpdateClipOrigin = FALSE;
//    BOOL bUpdateVisRgn = TRUE;
//    HDC hDC = NULL;
//    PPROCESSINFO ppi;
//    PLIST_ENTRY ListEntry;
//
//    if (NULL == Wnd)
//    {
//       Flags &= ~DCX_USESTYLE;
//       Flags |= DCX_CACHE;
//    }
//
//    if (Flags & DCX_PARENTCLIP) Flags |= DCX_CACHE;
//
//    // When GetDC is called with hWnd nz, DCX_CACHE & _WINDOW are clear w _USESTYLE set.
//    if (Flags & DCX_USESTYLE)
//    {
//       Flags &= ~(DCX_CLIPCHILDREN | DCX_CLIPSIBLINGS | DCX_PARENTCLIP);
//       if (!(Flags & DCX_WINDOW)) // Not window rectangle
//       {
//          if (Wnd->pcls->style & CS_PARENTDC)
//          {
//             Flags |= DCX_PARENTCLIP;
//          }
//
//          if (!(Flags & DCX_CACHE) && // Not on the cheap wine list.
//              !(Wnd->pcls->style & CS_OWNDC) )
//          {
//             if (!(Wnd->pcls->style & CS_CLASSDC))
//             // The window is not POWNED or has any CLASS, so we are looking for cheap wine.
//                Flags |= DCX_CACHE;
//             else
//             {
//                if (Wnd->pcls->pdce) hDC = ((PDCE)Wnd->pcls->pdce)->hDC;
//                TRACE("We have CLASS!!\n");
//             }
//          }
//
//          if (Wnd->style & WS_CLIPSIBLINGS)
//          {
//             Flags |= DCX_CLIPSIBLINGS;
//          }
//
//          if (Wnd->style & WS_CLIPCHILDREN &&
//              !(Wnd->style & WS_MINIMIZE))
//          {
//             Flags |= DCX_CLIPCHILDREN;
//          }
//          /* If minized with icon in the set, we are forced to be cheap! */
//          if (Wnd->style & WS_MINIMIZE && Wnd->pcls->spicn)
//          {
//             Flags |= DCX_CACHE;
//          }
//       }
//       else
//       {
//          if (Wnd->style & WS_CLIPSIBLINGS) Flags |= DCX_CLIPSIBLINGS;
//          Flags |= DCX_CACHE;
//       }
//    }
//
//    if (Flags & DCX_WINDOW) Flags &= ~DCX_CLIPCHILDREN;
//
//    if (Flags & DCX_NOCLIPCHILDREN)
//    {
//       Flags |= DCX_CACHE;
//       Flags &= ~(DCX_PARENTCLIP | DCX_CLIPCHILDREN);
//    }
//
//    Parent = (Wnd ? Wnd->spwndParent : NULL);
//
//    if (NULL == Wnd || !(Wnd->style & WS_CHILD) || NULL == Parent)
//    {
//       Flags &= ~DCX_PARENTCLIP;
//       Flags |= DCX_CLIPSIBLINGS;
//    }
//
//    /* It seems parent clip is ignored when clipping siblings or children */
//    if (Flags & (DCX_CLIPSIBLINGS | DCX_CLIPCHILDREN)) Flags &= ~DCX_PARENTCLIP;
//
//    if (Flags & DCX_PARENTCLIP)
//    {
//       if ((Wnd->style & WS_VISIBLE) &&
//           (Parent->style & WS_VISIBLE))
//       {
//          Flags &= ~DCX_CLIPCHILDREN;
//          if (Parent->style & WS_CLIPSIBLINGS)
//          {
//             Flags |= DCX_CLIPSIBLINGS;
//          }
//       }
//    }
//
//    // Window nz, check to see if we still own this or it is just cheap wine tonight.
//    if (!(Flags & DCX_CACHE))
//    {
//       if ( Wnd->head.pti != GetW32ThreadInfo())
//          Flags |= DCX_CACHE; // Ah~ Not Powned! Forced to be cheap~
//    }
//
//    DcxFlags = Flags & DCX_CACHECOMPAREMASK;
//
//    if (Flags & DCX_CACHE)
//    { // Scan the cheap wine list for our match.
//       DCE* DceEmpty = NULL;
//       DCE* DceUnused = NULL;
//       KeEnterCriticalRegion();
//       ListEntry = LEDce.Flink;
//       while (ListEntry != &LEDce)
//       {
//          Dce = CONTAINING_RECORD(ListEntry, DCE, List);
//          ListEntry = ListEntry->Flink;
// //
// // The way I understand this, you can have more than one DC per window.
// // Only one Owned if one was requested and saved and one Cached.
// //
//          if ((Dce->DCXFlags & (DCX_CACHE | DCX_DCEBUSY)) == DCX_CACHE)
//          {
//             DceUnused = Dce;
//             if (Dce->DCXFlags & DCX_DCEEMPTY)
//             {
//                DceEmpty = Dce;
//             }
//             else if (Dce->hwndCurrent == (Wnd ? UserHMGetHandle(Wnd) : NULL) &&
//                      ((Dce->DCXFlags & DCX_CACHECOMPAREMASK) == DcxFlags))
//             {
//                UpdateClipOrigin = TRUE;
//                break;
//             }
//          }
//          Dce = NULL; // Loop issue?
//       }
//       KeLeaveCriticalRegion();
//
//       Dce = (DceEmpty == NULL) ? DceUnused : DceEmpty;
//
//       if (Dce == NULL)
//       {
//          Dce = DceAllocDCE(NULL, DCE_CACHE_DC);
//       }
//       if (Dce == NULL) return NULL;
//
//       Dce->hwndCurrent = (Wnd ? UserHMGetHandle(Wnd) : NULL);
//       Dce->pwndOrg = Dce->pwndClip = Wnd;
//    }
//    else // If we are here, we are POWNED or having CLASS.
//    {
//       KeEnterCriticalRegion();
//       ListEntry = LEDce.Flink;
//       while (ListEntry != &LEDce)
//       {
//           Dce = CONTAINING_RECORD(ListEntry, DCE, List);
//           ListEntry = ListEntry->Flink;
//
//           // Skip Cache DCE entries.
//           if (!(Dce->DCXFlags & DCX_CACHE))
//           {
//              // Check for Window handle than HDC match for CLASS.
//              if (Dce->hwndCurrent == UserHMGetHandle(Wnd))
//              {
//                 bUpdateVisRgn = FALSE;
//                 break;
//              }
//              else if (Dce->hDC == hDC) break;
//           }
//           Dce = NULL; // Loop issue?
//       }
//       KeLeaveCriticalRegion();
//
//       if (Dce == NULL)
//       {
//          return(NULL);
//       }
//
//       if ( (Flags & (DCX_INTERSECTRGN|DCX_EXCLUDERGN)) &&
//            (Dce->DCXFlags & (DCX_INTERSECTRGN|DCX_EXCLUDERGN)) )
//       {
//          DceDeleteClipRgn(Dce);
//       }
//    }
// // First time use hax, need to use DceAllocDCE during window display init.
//    if (NULL == Dce)
//    {
//       return(NULL);
//    }
//
//    if (!GreIsHandleValid(Dce->hDC))
//    {
//       ERR("FIXME: Got DCE with invalid hDC! %p\n", Dce->hDC);
//       Dce->hDC = DceCreateDisplayDC();
//       /* FIXME: Handle error */
//    }
//
//    Dce->DCXFlags = Flags | DCX_DCEBUSY;
//
//    /*
//     * Bump it up! This prevents the random errors in wine dce tests and with
//     * proper bits set in DCX_CACHECOMPAREMASK.
//     * Reference:
//     *   https://reactos.org/archives/public/ros-dev/2008-July/010498.html
//     *   https://reactos.org/archives/public/ros-dev/2008-July/010499.html
//     */
//    RemoveEntryList(&Dce->List);
//    InsertHeadList(&LEDce, &Dce->List);
//
//    /* Introduced in rev 6691 and modified later. */
//    if ( (Flags & DCX_INTERSECTUPDATE) && !ClipRegion )
//    {
//       Flags |= DCX_INTERSECTRGN | DCX_KEEPCLIPRGN;
//       Dce->DCXFlags |= DCX_INTERSECTRGN | DCX_KEEPCLIPRGN;
//       ClipRegion = Wnd->hrgnUpdate;
//       bUpdateVisRgn = TRUE;
//    }
//
//    if (ClipRegion == HRGN_WINDOW)
//    {
//       if (!(Flags & DCX_WINDOW))
//       {
//          Dce->hrgnClip = NtGdiCreateRectRgn(
//              Wnd->rcClient.left,
//              Wnd->rcClient.top,
//              Wnd->rcClient.right,
//              Wnd->rcClient.bottom);
//       }
//       else
//       {
//           Dce->hrgnClip = NtGdiCreateRectRgn(
//               Wnd->rcWindow.left,
//               Wnd->rcWindow.top,
//               Wnd->rcWindow.right,
//               Wnd->rcWindow.bottom);
//       }
//       Dce->DCXFlags &= ~DCX_KEEPCLIPRGN;
//       bUpdateVisRgn = TRUE;
//    }
//    else if (ClipRegion != NULL)
//    {
//       if (Dce->hrgnClip != NULL)
//       {
//          ERR("Should not be called!!\n");
//          GreDeleteObject(Dce->hrgnClip);
//          Dce->hrgnClip = NULL;
//       }
//       Dce->hrgnClip = ClipRegion;
//       bUpdateVisRgn = TRUE;
//    }
//
//    if (IntGdiSetHookFlags(Dce->hDC, DCHF_VALIDATEVISRGN)) bUpdateVisRgn = TRUE;
//
//    DceSetDrawable(Wnd, Dce->hDC, Flags, UpdateClipOrigin);
//
//    if (bUpdateVisRgn) DceUpdateVisRgn(Dce, Wnd, Flags);
//
//    if (Dce->DCXFlags & DCX_CACHE)
//    {
//       TRACE("ENTER!!!!!! DCX_CACHE!!!!!!   hDC-> %p\n", Dce->hDC);
//       // Need to set ownership so Sync dcattr will work.
//       GreSetDCOwner(Dce->hDC, GDI_OBJ_HMGR_POWNED);
//       Dce->ptiOwner = GetW32ThreadInfo(); // Set the temp owning
//    }
//
//    if ( Wnd &&
//         Wnd->ExStyle & WS_EX_LAYOUTRTL &&
//        !(Flags & DCX_KEEPLAYOUT) )
//    {
//       NtGdiSetLayout(Dce->hDC, -1, LAYOUT_RTL);
//    }
//
//    if (Dce->DCXFlags & DCX_PROCESSOWNED)
//    {
//       ppi = PsGetCurrentProcessWin32Process();
//       ppi->W32PF_flags |= W32PF_OWNDCCLEANUP;
//       Dce->ptiOwner = NULL;
//       Dce->ppiOwner = ppi;
//    }
//
//    return(Dce->hDC);
// }
//
// /***********************************************************************
//  *           DceFreeDCE
//  */
// void FASTCALL
// DceFreeDCE(PDCE pdce, BOOLEAN Force)
// {
//   BOOL Hit = FALSE;
//
//   ASSERT(pdce != NULL);
//   if (NULL == pdce) return;
//
//   pdce->DCXFlags |= DCX_INDESTROY;
//
//   if (Force &&
//       GreGetObjectOwner(pdce->hDC) != GDI_OBJ_HMGR_POWNED)
//   {
//      TRACE("Change ownership for DCE! -> %p\n" , pdce);
//      // NOTE: Windows sets W32PF_OWNDCCLEANUP and moves on.
//      if (GreIsHandleValid(pdce->hDC))
//      {
//          GreSetDCOwner(pdce->hDC, GDI_OBJ_HMGR_POWNED);
//      }
//      else
//      {
//          ERR("Attempted to change ownership of an DCEhDC %p currently being destroyed!!!\n",
//              pdce->hDC);
//          Hit = TRUE;
//      }
//   }
//   else
//   {
//      if (GreGetObjectOwner(pdce->hDC) == GDI_OBJ_HMGR_PUBLIC)
//         GreSetDCOwner(pdce->hDC, GDI_OBJ_HMGR_POWNED);
//   }
//
//   if (!Hit) IntGdiDeleteDC(pdce->hDC, TRUE);
//
//   if (pdce->hrgnClip && !(pdce->DCXFlags & DCX_KEEPCLIPRGN))
//   {
//       GreDeleteObject(pdce->hrgnClip);
//       pdce->hrgnClip = NULL;
//   }
//
//   RemoveEntryList(&pdce->List);
//
//   ExFreePoolWithTag(pdce, USERTAG_DCE);
//
//   DCECount--;
//   TRACE("Freed DCE's! %d \n", DCECount);
// }
//
// /***********************************************************************
//  *           DceFreeWindowDCE
//  *
//  * Remove owned DCE and reset unreleased cache DCEs.
//  */
// void FASTCALL
// DceFreeWindowDCE(PWND Window)
// {
//   PDCE pDCE;
//   PLIST_ENTRY ListEntry;
//
//   if (DCECount <= 0)
//   {
//      ERR("FreeWindowDCE No Entry! %d\n",DCECount);
//      return;
//   }
//
//   ListEntry = LEDce.Flink;
//   while (ListEntry != &LEDce)
//   {
//      pDCE = CONTAINING_RECORD(ListEntry, DCE, List);
//      ListEntry = ListEntry->Flink;
//      if ( pDCE->hwndCurrent == UserHMGetHandle(Window) &&
//           !(pDCE->DCXFlags & DCX_DCEEMPTY) )
//      {
//         if (!(pDCE->DCXFlags & DCX_CACHE)) /* Owned or Class DCE */
//         {
//            if (Window->pcls->style & CS_CLASSDC) /* Test Class first */
//            {
//               if (pDCE->DCXFlags & (DCX_INTERSECTRGN | DCX_EXCLUDERGN)) /* Class DCE */
//                  DceDeleteClipRgn(pDCE);
//               // Update and reset Vis Rgn and clear the dirty bit.
//               // Should release VisRgn than reset it to default.
//               DceUpdateVisRgn(pDCE, Window, pDCE->DCXFlags);
//               pDCE->DCXFlags = DCX_DCEEMPTY|DCX_CACHE;
//               pDCE->hwndCurrent = 0;
//               pDCE->pwndOrg = pDCE->pwndClip = NULL;
//
//               TRACE("POWNED DCE going Cheap!! DCX_CACHE!! hDC-> %p \n",
//                     pDCE->hDC);
//               if (!GreSetDCOwner( pDCE->hDC, GDI_OBJ_HMGR_NONE))
//               {
//                   ERR("Fail Owner Switch hDC-> %p \n", pDCE->hDC);
//                   break;
//               }
//               /* Do not change owner so thread can clean up! */
//            }
//            else if (Window->pcls->style & CS_OWNDC) /* Owned DCE */
//            {
//               DceFreeDCE(pDCE, FALSE);
//               continue;
//            }
//            else
//            {
//               ERR("Not POWNED or CLASSDC hwndCurrent -> %p \n",
//                   pDCE->hwndCurrent);
//               // ASSERT(FALSE); /* bug 5320 */
//            }
//         }
//         else
//         {
//            if (pDCE->DCXFlags & DCX_DCEBUSY) /* Shared cache DCE */
//            {
//               /* FIXME: AFAICS we are doing the right thing here so
//                * this should be a TRACE. But this is best left as an ERR
//                * because the 'application error' is likely to come from
//                * another part of Wine (i.e. it's our fault after all).
//                * We should change this to TRACE when ReactOS is more stable
//                * (for 1.0?).
//                */
//               ERR("[%p] GetDC() without ReleaseDC()!\n", UserHMGetHandle(Window));
//               DceReleaseDC(pDCE, FALSE);
//            }
//            pDCE->DCXFlags |= DCX_DCEEMPTY;
//            pDCE->hwndCurrent = 0;
//            pDCE->pwndOrg = pDCE->pwndClip = NULL;
//         }
//      }
//   }
// }
//
// void FASTCALL
// DceFreeClassDCE(PDCE pdceClass)
// {
//    PDCE pDCE;
//    PLIST_ENTRY ListEntry;
//
//    ListEntry = LEDce.Flink;
//    while (ListEntry != &LEDce)
//    {
//        pDCE = CONTAINING_RECORD(ListEntry, DCE, List);
//        ListEntry = ListEntry->Flink;
//        if (pDCE == pdceClass)
//        {
//           DceFreeDCE(pDCE, TRUE); // Might have gone cheap!
//        }
//    }
// }
//
// void FASTCALL
// DceFreeThreadDCE(PTHREADINFO pti)
// {
//    PDCE pDCE;
//    PLIST_ENTRY ListEntry;
//
//    ListEntry = LEDce.Flink;
//    while (ListEntry != &LEDce)
//    {
//        pDCE = CONTAINING_RECORD(ListEntry, DCE, List);
//        ListEntry = ListEntry->Flink;
//        if (pDCE->ptiOwner == pti)
//        {
//           if (pDCE->DCXFlags & DCX_CACHE)
//           {
//              DceFreeDCE(pDCE, TRUE);
//           }
//        }
//    }
// }
//
// VOID FASTCALL
// DceEmptyCache(VOID)
// {
//    PDCE pDCE;
//    PLIST_ENTRY ListEntry;
//
//    ListEntry = LEDce.Flink;
//    while (ListEntry != &LEDce)
//    {
//       pDCE = CONTAINING_RECORD(ListEntry, DCE, List);
//       ListEntry = ListEntry->Flink;
//       DceFreeDCE(pDCE, TRUE);
//    }
// }
//
// VOID FASTCALL
// DceResetActiveDCEs(PWND Window)
// {
//    DCE *pDCE;
//    PDC dc;
//    PWND CurrentWindow;
//    INT DeltaX;
//    INT DeltaY;
//    PLIST_ENTRY ListEntry;
//
//    if (NULL == Window)
//    {
//       return;
//    }
//
//    ListEntry = LEDce.Flink;
//    while (ListEntry != &LEDce)
//    {
//       pDCE = CONTAINING_RECORD(ListEntry, DCE, List);
//       ListEntry = ListEntry->Flink;
//       if (0 == (pDCE->DCXFlags & (DCX_DCEEMPTY|DCX_INDESTROY)))
//       {
//          if (UserHMGetHandle(Window) == pDCE->hwndCurrent)
//          {
//             CurrentWindow = Window;
//          }
//          else
//          {
//             if (!pDCE->hwndCurrent)
//                CurrentWindow = NULL;
//             else
//                CurrentWindow = UserGetWindowObject(pDCE->hwndCurrent);
//             if (NULL == CurrentWindow)
//             {
//                continue;
//             }
//          }
//
//          if (!GreIsHandleValid(pDCE->hDC) ||
//              (dc = DC_LockDc(pDCE->hDC)) == NULL)
//          {
//             continue;
//          }
//          if (Window == CurrentWindow || IntIsChildWindow(Window, CurrentWindow))
//          {
//             if (pDCE->DCXFlags & DCX_WINDOW)
//             {
//                DeltaX = CurrentWindow->rcWindow.left - dc->ptlDCOrig.x;
//                DeltaY = CurrentWindow->rcWindow.top - dc->ptlDCOrig.y;
//                dc->ptlDCOrig.x = CurrentWindow->rcWindow.left;
//                dc->ptlDCOrig.y = CurrentWindow->rcWindow.top;
//             }
//             else
//             {
//                DeltaX = CurrentWindow->rcClient.left - dc->ptlDCOrig.x;
//                DeltaY = CurrentWindow->rcClient.top - dc->ptlDCOrig.y;
//                dc->ptlDCOrig.x = CurrentWindow->rcClient.left;
//                dc->ptlDCOrig.y = CurrentWindow->rcClient.top;
//             }
//
//             if (NULL != dc->dclevel.prgnClip)
//             {
//                REGION_bOffsetRgn(dc->dclevel.prgnClip, DeltaX, DeltaY);
//                dc->fs |= DC_DIRTY_RAO;
//             }
//             if (NULL != pDCE->hrgnClip)
//             {
//                NtGdiOffsetRgn(pDCE->hrgnClip, DeltaX, DeltaY);
//             }
//          }
//          DC_UnlockDc(dc);
//
//          DceUpdateVisRgn(pDCE, CurrentWindow, pDCE->DCXFlags);
//          IntGdiSetHookFlags(pDCE->hDC, DCHF_VALIDATEVISRGN);
//       }
//    }
// }
//
// HWND FASTCALL
// IntWindowFromDC(HDC hDc)
// {
//   DCE *Dce;
//   PLIST_ENTRY ListEntry;
//   HWND Ret = NULL;
//
//   ListEntry = LEDce.Flink;
//   while (ListEntry != &LEDce)
//   {
//       Dce = CONTAINING_RECORD(ListEntry, DCE, List);
//       ListEntry = ListEntry->Flink;
//       if (Dce->hDC == hDc)
//       {
//          if (Dce->DCXFlags & DCX_INDESTROY)
//             Ret = NULL;
//          else
//             Ret = Dce->hwndCurrent;
//          break;
//       }
//   }
//   return Ret;
// }
//
// INT FASTCALL
// UserReleaseDC(PWND Window, HDC hDc, BOOL EndPaint)
// {
//   PDCE dce;
//   PLIST_ENTRY ListEntry;
//   INT nRet = 0;
//   BOOL Hit = FALSE;
//
//   TRACE("%p %p\n", Window, hDc);
//   ListEntry = LEDce.Flink;
//   while (ListEntry != &LEDce)
//   {
//      dce = CONTAINING_RECORD(ListEntry, DCE, List);
//      ListEntry = ListEntry->Flink;
//      if (dce->hDC == hDc)
//      {
//         Hit = TRUE;
//         break;
//      }
//   }
//
//   if ( Hit && (dce->DCXFlags & DCX_DCEBUSY))
//   {
//      nRet = DceReleaseDC(dce, EndPaint);
//   }
//
//   return nRet;
// }
//
// HDC FASTCALL
// UserGetWindowDC(PWND Wnd)
// {
//   return UserGetDCEx(Wnd, 0, DCX_USESTYLE | DCX_WINDOW);
// }
//
// HWND FASTCALL
// UserGethWnd( HDC hdc, PWNDOBJ *pwndo)
// {
//   EWNDOBJ* Clip;
//   PWND Wnd;
//   HWND hWnd;
//
//   hWnd = IntWindowFromDC(hdc);
//
//   if (hWnd && (Wnd = UserGetWindowObject(hWnd)))
//   {
//      Clip = (EWNDOBJ*)UserGetProp(Wnd, AtomWndObj, TRUE);
//
//      if ( Clip && Clip->Hwnd == hWnd )
//      {
//         if (pwndo) *pwndo = (PWNDOBJ)Clip;
//      }
//   }
//   return hWnd;
// }
//
// HDC __stdcall
// NtUserGetDCEx(HWND hWnd OPTIONAL, HANDLE ClipRegion, ULONG Flags)
// {
//   PWND Wnd=NULL;
//   HDC Ret = NULL;
//
//   TRACE("Enter NtUserGetDCEx: hWnd %p, ClipRegion %p, Flags %x.\n",
//       hWnd, ClipRegion, Flags);
//   UserEnterExclusive();
//
//   if (hWnd && !(Wnd = UserGetWindowObject(hWnd)))
//   {
//       goto Exit; // Return NULL
//   }
//   Ret = UserGetDCEx(Wnd, ClipRegion, Flags);
//
// Exit:
//   TRACE("Leave NtUserGetDCEx, ret=%p\n", Ret);
//   UserLeave();
//   return Ret;
// }
//
// /*
//  * NtUserGetWindowDC
//  *
//  * The NtUserGetWindowDC function retrieves the device context (DC) for the
//  * entire window, including title bar, menus, and scroll bars. A window device
//  * context permits painting anywhere in a window, because the origin of the
//  * device context is the upper-left corner of the window instead of the client
//  * area.
//  *
//  * Status
//  *    @implemented
//  */
// HDC __stdcall
// NtUserGetWindowDC(HWND hWnd)
// {
//   return NtUserGetDCEx(hWnd, 0, DCX_USESTYLE | DCX_WINDOW);
// }
//
// HDC __stdcall
// NtUserGetDC(HWND hWnd)
// {
//  TRACE("NtUGetDC -> %p:%x\n", hWnd, !hWnd ? DCX_CACHE | DCX_WINDOW : DCX_USESTYLE);
//
//   return NtUserGetDCEx(hWnd, NULL, NULL == hWnd ? DCX_CACHE | DCX_WINDOW : DCX_USESTYLE);
// }
//
// /*!
//  * Select logical palette into device context.
//  * \param	hDC 				handle to the device context
//  * \param	hpal				handle to the palette
//  * \param	ForceBackground 	If this value is FALSE the logical palette will be copied to the device palette only when the application
//  * 								is in the foreground. If this value is TRUE then map the colors in the logical palette to the device
//  * 								palette colors in the best way.
//  * \return	old palette
//  *
//  * \todo	implement ForceBackground == TRUE
// */
// HPALETTE
// __stdcall
// NtUserSelectPalette(HDC  hDC,
//               HPALETTE  hpal,
//        BOOL  ForceBackground)
// {
//     HPALETTE oldPal;
//     UserEnterExclusive();
//     // Implement window checks
//     oldPal = GdiSelectPalette( hDC, hpal, ForceBackground);
//     UserLeave();
//     return oldPal;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
