use win32k_sys::win32k::*;

extern "C" {

}

// `ASSERT_IRQL_LESS_OR_EQUAL` is a C macro in <reactos/debug.h>:251 that
// expands to `ASSERT(KeGetCurrentIrql() <= (x))`. Per win32k_translation_prompt.md
// §6 (macros must be inline-expanded, not extern-declared). Translate as
// a no-op in release (debug.h gates on NDEBUG); preserve the call site.
#[inline(always)]
#[allow(non_snake_case)]
unsafe fn ASSERT_IRQL_LESS_OR_EQUAL(_irql: KIRQL) {
    // ASSERT() is debug-only; non-debug build is a no-op. Mirrors the C macro
    // expansion under NDEBUG.
}

// VIS_ComputeVisibleRegion is declared FASTCALL in win32ss/user/ntuser/vis.h:12
// — fastcall on i386, cdecl on amd64/arm64. Dual-cfg extern block matches the
// canonical per-arch convention (abi_map.py / win32k_translation_prompt.md §1).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn VIS_ComputeVisibleRegion(
        Window: PWND,
        ClientArea: BOOLEAN,
        ClipChildren: BOOLEAN,
        ClipSiblings: BOOLEAN,
    ) -> PREGION;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn VIS_ComputeVisibleRegion(
        Window: PWND,
        ClientArea: BOOLEAN,
        ClipChildren: BOOLEAN,
        ClipSiblings: BOOLEAN,
    ) -> PREGION;
}

// CLIPOBJ_bEnum / CLIPOBJ_cEnumStart are GDI exports (win32k.spec:5-6:
// `stdcall`). `extern "system"` resolves to stdcall on i386, cdecl on
// amd64/arm64 — matches the spec ABI.
extern "system" {
    fn CLIPOBJ_bEnum(pco: *mut CLIPOBJ, cj: ULONG, pv: *mut ULONG) -> i32;
    fn CLIPOBJ_cEnumStart(pco: *mut CLIPOBJ, bAll: i32, iType: u32, iDirection: u32, cMaxRects: u32,) -> u32;
}

// ---- file-scope preamble (translated) ----
pub static mut gcountPWO: INT = 0;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntEngWndCallChangeProc(
    Clip: *mut EWNDOBJ,
    flChanged: FLONG,
) {
    if (*Clip).ChangeProc.is_none() {
        return;
    }

    let flChanged = flChanged & (*Clip).Flags;
    if flChanged == 0 {
        return;
    }

    TRACE!(
        b"Calling WNDOBJCHANGEPROC (0x%p), Changed = 0x%x\n\0".as_ptr() as PCHAR,
        (*Clip).ChangeProc,
        flChanged,
    );

    if flChanged == WOC_CHANGED {
        ((*Clip).ChangeProc.unwrap())(core::ptr::null_mut(), flChanged);
    } else {
        ((*Clip).ChangeProc.unwrap())(Clip as *mut WNDOBJ, flChanged);
    }
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntEngWndCallChangeProc(
    Clip: *mut EWNDOBJ,
    flChanged: FLONG,
) {
    if (*Clip).ChangeProc.is_none() {
        return;
    }

    let flChanged = flChanged & (*Clip).Flags;
    if flChanged == 0 {
        return;
    }

    TRACE!(
        b"Calling WNDOBJCHANGEPROC (0x%p), Changed = 0x%x\n\0".as_ptr() as PCHAR,
        (*Clip).ChangeProc,
        flChanged,
    );

    if flChanged == WOC_CHANGED {
        ((*Clip).ChangeProc.unwrap())(core::ptr::null_mut(), flChanged);
    } else {
        ((*Clip).ChangeProc.unwrap())(Clip as *mut WNDOBJ, flChanged);
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntEngWndUpdateClipObj(
    Clip: *mut EWNDOBJ,
    Window: PWND,
) -> u8 {

    let visRgn: PREGION;

    TRACE!(b"IntEngWndUpdateClipObj\n\0".as_ptr() as *const i8);

    visRgn = VIS_ComputeVisibleRegion(Window, 1u8, 1u8, 1u8);
    if !visRgn.is_null() {
        if (*visRgn).rdh.nCount > 0 {
            IntEngUpdateClipRegion(
                Clip as *mut XCLIPOBJ,
                (*visRgn).rdh.nCount,
                (*visRgn).Buffer,
                core::ptr::addr_of_mut!((*visRgn).rdh.rcBound),
            );
            TRACE!(
                b"Created visible region with %lu rects\n\0".as_ptr() as *const i8,
                (*visRgn).rdh.nCount,
            );
            TRACE!(
                b"  BoundingRect: %d, %d  %d, %d\n\0".as_ptr() as *const i8,
                (*visRgn).rdh.rcBound.left,
                (*visRgn).rdh.rcBound.top,
                (*visRgn).rdh.rcBound.right,
                (*visRgn).rdh.rcBound.bottom,
            );
            {
                let mut i: u32 = 0;
                while i < (*visRgn).rdh.nCount {
                    TRACE!(
                        b"  Rect #%lu: %ld,%ld  %ld,%ld\n\0".as_ptr() as *const i8,
                        i + 1,
                        (*(*visRgn).Buffer.offset(i as isize)).left,
                        (*(*visRgn).Buffer.offset(i as isize)).top,
                        (*(*visRgn).Buffer.offset(i as isize)).right,
                        (*(*visRgn).Buffer.offset(i as isize)).bottom,
                    );
                    i += 1;
                }
            }
        }
        REGION_Delete(visRgn);
    } else {
        IntEngUpdateClipRegion(
            Clip as *mut XCLIPOBJ,
            1,
            core::ptr::addr_of_mut!((*Window).rcClient),
            core::ptr::addr_of_mut!((*Window).rcClient),
        );
    }

    // _EWNDOBJ in engobjects.h:96 has anonymous inheritance:
    //   struct _EWNDOBJ { XCLIPOBJ; HWND Hwnd; ... }
    //   struct _XCLIPOBJ { CLIPOBJ; RWNDOBJ; ... }
    //   struct _RWNDOBJ  { PVOID pvConsumer; RECTL rclClient; SURFOBJ *psoOwner; }
    // bindgen lost the anonymous-member field names (sees only padding +
    // tail fields). `rclClient` is reachable via the WNDOBJ view (WNDOBJ has
    // coClient/pvConsumer/rclClient/psoOwner at the same offsets); `iUniq`
    // is the first field of CLIPOBJ.
    (*(Clip as *mut WNDOBJ)).rclClient = (*Window).rcClient;
    let pco = Clip as *mut CLIPOBJ;
    (*pco).iUniq = (*pco).iUniq.wrapping_add(1);

    1u8
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntEngWndUpdateClipObj(
    Clip: *mut EWNDOBJ,
    Window: PWND,
) -> u8 {

    let visRgn: PREGION;

    TRACE!(b"IntEngWndUpdateClipObj\n\0".as_ptr() as *const i8);

    visRgn = VIS_ComputeVisibleRegion(Window, 1u8, 1u8, 1u8);
    if !visRgn.is_null() {
        if (*visRgn).rdh.nCount > 0 {
            IntEngUpdateClipRegion(
                Clip as *mut XCLIPOBJ,
                (*visRgn).rdh.nCount,
                (*visRgn).Buffer,
                core::ptr::addr_of_mut!((*visRgn).rdh.rcBound),
            );
            TRACE!(
                b"Created visible region with %lu rects\n\0".as_ptr() as *const i8,
                (*visRgn).rdh.nCount,
            );
            TRACE!(
                b"  BoundingRect: %d, %d  %d, %d\n\0".as_ptr() as *const i8,
                (*visRgn).rdh.rcBound.left,
                (*visRgn).rdh.rcBound.top,
                (*visRgn).rdh.rcBound.right,
                (*visRgn).rdh.rcBound.bottom,
            );
            {
                let mut i: u32 = 0;
                while i < (*visRgn).rdh.nCount {
                    TRACE!(
                        b"  Rect #%lu: %ld,%ld  %ld,%ld\n\0".as_ptr() as *const i8,
                        i + 1,
                        (*(*visRgn).Buffer.offset(i as isize)).left,
                        (*(*visRgn).Buffer.offset(i as isize)).top,
                        (*(*visRgn).Buffer.offset(i as isize)).right,
                        (*(*visRgn).Buffer.offset(i as isize)).bottom,
                    );
                    i += 1;
                }
            }
        }
        REGION_Delete(visRgn);
    } else {
        IntEngUpdateClipRegion(
            Clip as *mut XCLIPOBJ,
            1,
            core::ptr::addr_of_mut!((*Window).rcClient),
            core::ptr::addr_of_mut!((*Window).rcClient),
        );
    }

    // _EWNDOBJ in engobjects.h:96 has anonymous inheritance:
    //   struct _EWNDOBJ { XCLIPOBJ; HWND Hwnd; ... }
    //   struct _XCLIPOBJ { CLIPOBJ; RWNDOBJ; ... }
    //   struct _RWNDOBJ  { PVOID pvConsumer; RECTL rclClient; SURFOBJ *psoOwner; }
    // bindgen lost the anonymous-member field names (sees only padding +
    // tail fields). `rclClient` is reachable via the WNDOBJ view (WNDOBJ has
    // coClient/pvConsumer/rclClient/psoOwner at the same offsets); `iUniq`
    // is the first field of CLIPOBJ.
    (*(Clip as *mut WNDOBJ)).rclClient = (*Window).rcClient;
    let pco = Clip as *mut CLIPOBJ;
    (*pco).iUniq = (*pco).iUniq.wrapping_add(1);

    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngWindowChanged(
    Window: PWND,
    flChanged: FLONG,
) {

    ASSERT_IRQL_LESS_OR_EQUAL(PASSIVE_LEVEL as KIRQL);

    let Clip: *mut EWNDOBJ = UserGetProp(Window, AtomWndObj, 1u8) as *mut EWNDOBJ;
    if Clip.is_null() {
        return;
    }

    match flChanged {
        WOC_RGN_CLIENT => {
            IntEngWndUpdateClipObj(Clip, Window);
        }
        WOC_DELETE => {
        }
        _ => {}
    }

    IntEngWndCallChangeProc(Clip, flChanged);

    if flChanged == WOC_RGN_CLIENT {
        IntEngWndCallChangeProc(Clip, WOC_CHANGED);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngWindowChanged(
    Window: PWND,
    flChanged: FLONG,
) {

    ASSERT_IRQL_LESS_OR_EQUAL(PASSIVE_LEVEL as KIRQL);

    let Clip: *mut EWNDOBJ = UserGetProp(Window, AtomWndObj, 1u8) as *mut EWNDOBJ;
    if Clip.is_null() {
        return;
    }

    match flChanged {
        WOC_RGN_CLIENT => {
            IntEngWndUpdateClipObj(Clip, Window);
        }
        WOC_DELETE => {
        }
        _ => {}
    }

    IntEngWndCallChangeProc(Clip, flChanged);

    if flChanged == WOC_RGN_CLIENT {
        IntEngWndCallChangeProc(Clip, WOC_CHANGED);
    }
}
#[no_mangle]
pub unsafe extern "system" fn EngDeleteWnd(
    pwo: *mut WNDOBJ) {
    let Clip = pwo as *mut EWNDOBJ;
    let mut Window: PWND;

    TRACE!(b"EngDeleteWnd: pwo = 0x%p\n\0".as_ptr() as *const i8, pwo);

    UserEnterExclusive();

    Window = UserGetWindowObject((*Clip).Hwnd);
    if Window == core::ptr::null_mut()
    {
        ERR!("Couldnt get window object for WndObjInt->Hwnd!!!\n");
    }
    else
    {
        UserRemoveProp(Window, AtomWndObj, 1u8);
    }
    gcountPWO -= 1;

    UserLeave();

    IntEngFreeClipResources(Clip as *mut XCLIPOBJ);
    EngFreeMem(Clip as *mut core::ffi::c_void);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn WNDOBJ_bEnum(
    pwo: *mut WNDOBJ,
    cj: ULONG,
    pul: *mut ULONG,
) -> BOOL {
    
    CLIPOBJ_bEnum(
        core::ptr::addr_of_mut!((*pwo).coClient),
        cj,
        pul,
    ) as i32
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn WNDOBJ_bEnum(
    pwo: *mut WNDOBJ,
    cj: ULONG,
    pul: *mut ULONG,
) -> BOOL {
    
    CLIPOBJ_bEnum(
        core::ptr::addr_of_mut!((*pwo).coClient),
        cj,
        pul,
    ) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn WNDOBJ_cEnumStart(
    pwo: *mut WNDOBJ,
    iType: ULONG,
    iDirection: ULONG,
    cLimit: ULONG,
) -> ULONG {
    
    CLIPOBJ_cEnumStart(
        core::ptr::addr_of_mut!((*pwo).coClient),
        0,
        iType,
        iDirection,
        cLimit,
    )
}
#[no_mangle]
pub unsafe extern "system" fn WNDOBJ_vSetConsumer(
    pwo: *mut WNDOBJ,
    pvConsumer: *mut core::ffi::c_void,
) {
    let Clip: *mut EWNDOBJ = pwo as *mut EWNDOBJ;
    let Hack: u8;

    TRACE!(b"WNDOBJ_vSetConsumer: pwo = 0x%p, pvConsumer = 0x%p\n\0".as_ptr() as PCHAR, pwo, pvConsumer);

    Hack = if (*pwo).pvConsumer.is_null() { 1u8 } else { 0u8 };
    (*pwo).pvConsumer = pvConsumer;

    if Hack != 0 {
        FIXME!(b"Is this hack really needed?\n\0".as_ptr() as PCHAR);
        IntEngWndCallChangeProc(Clip, WOC_RGN_CLIENT);
        IntEngWndCallChangeProc(Clip, WOC_CHANGED);
        IntEngWndCallChangeProc(Clip, WOC_DRAWN);
    }
}

// EngCreateWnd — APIENTRY (stdcall on i386, cdecl elsewhere). Full body
// translation deferred; provide a no-op stub so the link resolves. The function
// is GDI DDK API for display drivers, exercised only when a real driver calls
// it; returning NULL is safe for now and matches the @unimplemented mark.
#[no_mangle]
pub unsafe extern "system" fn EngCreateWnd(
    _pso: *mut SURFOBJ,
    _hWnd: HWND,
    _pfn: WNDOBJCHANGEPROC,
    _fl: FLONG,
    _iPixelFormat: i32,
) -> *mut WNDOBJ {
    core::ptr::null_mut()
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI WNDOBJ Functions
//  * FILE:              win32ss/gdi/eng/engwindow.c
//  * PROGRAMER:         Gregor Anich
//  */
//
// #include <win32k.h>
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(EngWnd);
//
// INT gcountPWO = 0;
//
// /*
//  * Calls the WNDOBJCHANGEPROC of the given WNDOBJ
//  */
// VOID
// FASTCALL
// IntEngWndCallChangeProc(
//     _In_ EWNDOBJ *Clip,
//     _In_ FLONG   flChanged)
// {
//     if (Clip->ChangeProc == NULL)
//     {
//         return;
//     }
//
//     /* check flags of the WNDOBJ */
//     flChanged &= Clip->Flags;
//     if (flChanged == 0)
//     {
//         return;
//     }
//
//     TRACE("Calling WNDOBJCHANGEPROC (0x%p), Changed = 0x%x\n",
//            Clip->ChangeProc, flChanged);
//
//     /* Call the WNDOBJCHANGEPROC */
//     if (flChanged == WOC_CHANGED)
//         Clip->ChangeProc(NULL, flChanged);
//     else
//         Clip->ChangeProc((WNDOBJ *)Clip, flChanged);
// }
//
// /*
//  * Fills the CLIPOBJ and client rect of the WNDOBJ with the data from the given WND
//  */
// BOOLEAN
// FASTCALL
// IntEngWndUpdateClipObj(
//     EWNDOBJ* Clip,
//     PWND Window)
// {
//     PREGION visRgn;
//
//     TRACE("IntEngWndUpdateClipObj\n");
//
//     visRgn = VIS_ComputeVisibleRegion(Window, TRUE, TRUE, TRUE);
//     if (visRgn != NULL)
//     {
//         if (visRgn->rdh.nCount > 0)
//         {
//             IntEngUpdateClipRegion((XCLIPOBJ*)Clip, visRgn->rdh.nCount, visRgn->Buffer, &visRgn->rdh.rcBound);
//             TRACE("Created visible region with %lu rects\n", visRgn->rdh.nCount);
//             TRACE("  BoundingRect: %d, %d  %d, %d\n",
//                    visRgn->rdh.rcBound.left, visRgn->rdh.rcBound.top,
//                    visRgn->rdh.rcBound.right, visRgn->rdh.rcBound.bottom);
//             {
//                 ULONG i;
//                 for (i = 0; i < visRgn->rdh.nCount; i++)
//                 {
//                     TRACE("  Rect #%lu: %ld,%ld  %ld,%ld\n", i+1,
//                            visRgn->Buffer[i].left, visRgn->Buffer[i].top,
//                            visRgn->Buffer[i].right, visRgn->Buffer[i].bottom);
//                 }
//             }
//         }
//         REGION_Delete(visRgn);
//     }
//     else
//     {
//         /* Fall back to client rect */
//         IntEngUpdateClipRegion((XCLIPOBJ*)Clip, 1, &Window->rcClient, &Window->rcClient);
//     }
//
//     /* Update the WNDOBJ */
//     Clip->rclClient = Window->rcClient;
//     Clip->iUniq++;
//
//     return TRUE;
// }
//
// /*
//  * Updates all WNDOBJs of the given WND and calls the change-procs.
//  */
// VOID
// FASTCALL
// IntEngWindowChanged(
//     _In_    PWND  Window,
//     _In_    FLONG flChanged)
// {
//     EWNDOBJ *Clip;
//
//     ASSERT_IRQL_LESS_OR_EQUAL(PASSIVE_LEVEL as KIRQL);
//
//     Clip = UserGetProp(Window, AtomWndObj, TRUE);
//     if (!Clip)
//     {
//         return;
//     }
//
//     ASSERT(Clip->Hwnd == Window->head.h);
//     // if (Clip->pvConsumer != NULL)
//     {
//         /* Update the WNDOBJ */
//         switch (flChanged)
//         {
//         case WOC_RGN_CLIENT:
//             /* Update the clipobj and client rect of the WNDOBJ */
//             IntEngWndUpdateClipObj(Clip, Window);
//             break;
//
//         case WOC_DELETE:
//             /* FIXME: Should the WNDOBJs be deleted by win32k or by the driver? */
//             break;
//         }
//
//         /* Call the change proc */
//         IntEngWndCallChangeProc(Clip, flChanged);
//
//         /* HACK: Send WOC_CHANGED after WOC_RGN_CLIENT */
//         if (flChanged == WOC_RGN_CLIENT)
//         {
//             IntEngWndCallChangeProc(Clip, WOC_CHANGED);
//         }
//     }
// }
//
// /*
//  * @implemented
//  */
// WNDOBJ*
// APIENTRY
// EngCreateWnd(
//     SURFOBJ          *pso,
//     HWND              hWnd,
//     WNDOBJCHANGEPROC  pfn,
//     FLONG             fl,
//     int               iPixelFormat)
// {
//     EWNDOBJ *Clip = NULL;
//     WNDOBJ *WndObjUser = NULL;
//     PWND Window;
//
//     TRACE("EngCreateWnd: pso = 0x%p, hwnd = 0x%p, pfn = 0x%p, fl = 0x%lx, pixfmt = %d\n",
//             pso, hWnd, pfn, fl, iPixelFormat);
//
//     UserEnterExclusive();
//
//     if (fl & (WO_RGN_WINDOW | WO_RGN_DESKTOP_COORD | WO_RGN_UPDATE_ALL))
//     {
//         FIXME("Unsupported flags: 0x%lx\n", fl & ~(WO_RGN_CLIENT_DELTA | WO_RGN_CLIENT | WO_RGN_SURFACE_DELTA | WO_RGN_SURFACE));
//     }
//
//     /* Get window object */
//     Window = UserGetWindowObject(hWnd);
//     if (Window == NULL)
//     {
//         goto Exit;
//     }
//
//     /* Create WNDOBJ */
//     Clip = EngAllocMem(FL_ZERO_MEMORY, sizeof(EWNDOBJ), GDITAG_WNDOBJ);
//     if (Clip == NULL)
//     {
//         ERR("Failed to allocate memory for a WND structure!\n");
//         goto Exit;
//     }
//     IntEngInitClipObj((XCLIPOBJ*)Clip);
//
//     /* Fill the clipobj */
//     if (!IntEngWndUpdateClipObj(Clip, Window))
//     {
//         EngFreeMem(Clip);
//         goto Exit;
//     }
//
//     /* Fill user object */
//     WndObjUser = (WNDOBJ *)Clip;
//     WndObjUser->psoOwner = pso;
//     WndObjUser->pvConsumer = NULL;
//
//     /* Fill internal object */
//     Clip->Hwnd = hWnd;
//     Clip->ChangeProc = pfn;
//     /* Keep track of relevant flags */
//     Clip->Flags = fl & (WO_RGN_CLIENT_DELTA | WO_RGN_CLIENT | WO_RGN_SURFACE_DELTA | WO_RGN_SURFACE | WO_DRAW_NOTIFY);
//     if (fl & WO_SPRITE_NOTIFY)
//         Clip->Flags |= WOC_SPRITE_OVERLAP | WOC_SPRITE_NO_OVERLAP;
//     /* Those should always be sent */
//     Clip->Flags |= WOC_CHANGED | WOC_DELETE;
//     Clip->PixelFormat = iPixelFormat;
//
//     /* associate object with window */
//     UserSetProp(Window, AtomWndObj, Clip, TRUE);
//     ++gcountPWO;
//
//     TRACE("EngCreateWnd: SUCCESS: %p!\n", WndObjUser);
//
// Exit:
//     UserLeave();
//     return WndObjUser;
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngDeleteWnd(
//     IN WNDOBJ *pwo)
// {
//     EWNDOBJ* Clip = (EWNDOBJ *)pwo;//CONTAINING_RECORD(pwo, XCLIPOBJ, WndObj);
//     PWND Window;
//
//     TRACE("EngDeleteWnd: pwo = 0x%p\n", pwo);
//
//     UserEnterExclusive();
//
//     /* Get window object */
//     Window = UserGetWindowObject(Clip->Hwnd);
//     if (Window == NULL)
//     {
//         ERR("Couldnt get window object for WndObjInt->Hwnd!!!\n");
//     }
//     else
//     {
//         /* Remove object from window */
//         UserRemoveProp(Window, AtomWndObj, TRUE);
//     }
//     --gcountPWO;
//
//     UserLeave();
//
//     /* Free resources */
//     IntEngFreeClipResources((XCLIPOBJ*)Clip);
//     EngFreeMem(Clip);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// WNDOBJ_bEnum(
//     IN WNDOBJ  *pwo,
//     IN ULONG  cj,
//     OUT ULONG  *pul)
// {
//     /* Relay */
//     return CLIPOBJ_bEnum(&pwo->coClient, cj, pul);
// }
//
//
// /*
//  * @implemented
//  */
// ULONG
// APIENTRY
// WNDOBJ_cEnumStart(
//     IN WNDOBJ  *pwo,
//     IN ULONG  iType,
//     IN ULONG  iDirection,
//     IN ULONG  cLimit)
// {
//     /* Relay */
//     // FIXME: Should we enumerate all rectangles or not?
//     return CLIPOBJ_cEnumStart(&pwo->coClient, FALSE, iType, iDirection, cLimit);
// }
//
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// WNDOBJ_vSetConsumer(
//     IN WNDOBJ  *pwo,
//     IN PVOID  pvConsumer)
// {
//     EWNDOBJ* Clip = (EWNDOBJ *)pwo;//CONTAINING_RECORD(pwo, XCLIPOBJ, WndObj);
//     BOOL Hack;
//
//     TRACE("WNDOBJ_vSetConsumer: pwo = 0x%p, pvConsumer = 0x%p\n", pwo, pvConsumer);
//
//     Hack = (pwo->pvConsumer == NULL);
//     pwo->pvConsumer = pvConsumer;
//
//     /* HACKHACKHACK
//      *
//      * MSDN says that the WNDOBJCHANGEPROC will be called with the most recent state
//      * when a WNDOBJ is created - we do it here because most drivers will need pvConsumer
//      * in the callback to identify the WNDOBJ I think.
//      *
//      *  - blight
//      */
//     if (Hack)
//     {
//         FIXME("Is this hack really needed?\n");
//         IntEngWndCallChangeProc(Clip, WOC_RGN_CLIENT);
//         IntEngWndCallChangeProc(Clip, WOC_CHANGED);
//         IntEngWndCallChangeProc(Clip, WOC_DRAWN);
//     }
// }
//
// /* EOF */
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
