use win32k_sys::win32k::*;
use core::ffi::c_void;

extern "C" {

}

// RtlCopyMemory is `memcpy` (Microsoft macro). Reuse core::ptr::copy_nonoverlapping
// for byte-count copies. Keeping the function declaration here for parity with
// other GDI eng files that pull the kernel symbol.
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, n: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, n);
}

// ---- file-scope preamble (translated) ----
pub const GDITAG_CLIPOBJ: u32 = 0x6A624F43;

// XCLIPOBJ in C inherits CLIPOBJ + RWNDOBJ via __mingw anonymous struct
// extension (struct _XCLIPOBJ { CLIPOBJ; RWNDOBJ; ... }). bindgen renders
// that as `__bindgen_padding_0: [u32; 12]` covering both base structs.
// To touch the CLIPOBJ fields (rclBounds, iDComplexity, iFComplexity, iMode,
// iUniq), cast the XCLIPOBJ pointer down to CLIPOBJ — same address, same
// layout for the leading 24 bytes.
#[inline(always)]
unsafe fn clip_base(p: *mut XCLIPOBJ) -> *mut CLIPOBJ {
    p as *mut CLIPOBJ
}

unsafe extern "C" fn CompareRightDown(
    pv1: *const c_void,
    pv2: *const c_void,
) -> i32 {
    let r1 = pv1 as *const RECTL;
    let r2 = pv2 as *const RECTL;
    let Cmp: i32;

    if (*r1).top < (*r2).top
    {
        Cmp = -1;
    }
    else if (*r2).top < (*r1).top
    {
        Cmp = 1;
    }
    else
    {
        ntkrnl_sys::ASSERT!((*r1).bottom == (*r2).bottom);
        if (*r1).left < (*r2).left
        {
            Cmp = -1;
        }
        else if (*r2).left < (*r1).left
        {
            Cmp = 1;
        }
        else
        {
            ntkrnl_sys::ASSERT!((*r1).right == (*r2).right);
            Cmp = 0;
        }
    }

    Cmp
}

unsafe extern "C" fn CompareRightUp(
    pv1: *const c_void,
    pv2: *const c_void,
) -> i32 {
    let r1 = pv1 as *const RECTL;
    let r2 = pv2 as *const RECTL;
    let Cmp: i32;

    if (*r1).bottom < (*r2).bottom {
        Cmp = 1;
    } else if (*r2).bottom < (*r1).bottom {
        Cmp = -1;
    } else {
        ntkrnl_sys::ASSERT!((*r1).top == (*r2).top);
        if (*r1).left < (*r2).left {
            Cmp = -1;
        } else if (*r2).left < (*r1).left {
            Cmp = 1;
        } else {
            ntkrnl_sys::ASSERT!((*r1).right == (*r2).right);
            Cmp = 0;
        }
    }

    Cmp
}

unsafe extern "C" fn CompareLeftDown(
    pv1: *const c_void,
    pv2: *const c_void,
) -> i32 {
    let r1 = pv1 as *const RECTL;
    let r2 = pv2 as *const RECTL;
    let Cmp: i32;

    if (*r1).top < (*r2).top
    {
        Cmp = -1;
    }
    else if (*r2).top < (*r1).top
    {
        Cmp = 1;
    }
    else
    {
        if (*r1).right < (*r2).right
        {
            Cmp = 1;
        }
        else if (*r2).right < (*r1).right
        {
            Cmp = -1;
        }
        else
        {
            Cmp = 0;
        }
    }

    Cmp
}

unsafe extern "C" fn CompareLeftUp(
    pv1: *const c_void,
    pv2: *const c_void,
) -> i32 {
    let r1 = pv1 as *const RECTL;
    let r2 = pv2 as *const RECTL;
    let Cmp: i32;

    if (*r1).bottom < (*r2).bottom
    {
        Cmp = 1;
    }
    else if (*r2).bottom < (*r1).bottom
    {
        Cmp = -1;
    }
    else
    {
        if (*r1).right < (*r2).right
        {
            Cmp = 1;
        }
        else if (*r2).right < (*r1).right
        {
            Cmp = -1;
        }
        else
        {
            Cmp = 0;
        }
    }
    Cmp
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngInitClipObj(Clip: *mut XCLIPOBJ) {
    let base = clip_base(Clip);
    (*Clip).Rects = core::ptr::addr_of_mut!((*base).rclBounds);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngInitClipObj(Clip: *mut XCLIPOBJ) {
    let base = clip_base(Clip);
    (*Clip).Rects = core::ptr::addr_of_mut!((*base).rclBounds);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngFreeClipResources(Clip: *mut XCLIPOBJ) {
    let base = clip_base(Clip);
    if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
        EngFreeMem((*Clip).Rects as *mut c_void);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngFreeClipResources(Clip: *mut XCLIPOBJ) {
    let base = clip_base(Clip);
    if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
        EngFreeMem((*Clip).Rects as *mut c_void);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngUpdateClipRegion(
    Clip: *mut XCLIPOBJ,
    count: u32,
    pRect: *const RECTL,
    rcBounds: *const RECTL)
{
    let base = clip_base(Clip);
    if count > 1 {
        // FIELD_OFFSET(ENUMRECTS, arcl[count]) = sizeof(ENUMRECTS) + (count-1)*sizeof(RECTL)
        let alloc_size = (core::mem::size_of::<ENUMRECTS>()
            + (count as usize - 1) * core::mem::size_of::<RECTL>()) as u32;
        let NewRects = EngAllocMem(0, alloc_size, GDITAG_CLIPOBJ) as *mut RECTL;

        if !NewRects.is_null() {
            (*Clip).RectCount = count;
            (*Clip).iDirection = CD_ANY;
            core::ptr::copy_nonoverlapping(pRect, NewRects, count as usize);

            (*base).iDComplexity = DC_COMPLEX as u8;
            (*base).iFComplexity = if (*Clip).RectCount <= 4 { FC_RECT4 as u8 } else { FC_COMPLEX as u8 };
            (*base).iMode = TC_RECTANGLES as u8;
            (*base).rclBounds = *rcBounds;

            if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
                EngFreeMem((*Clip).Rects as *mut c_void);
            }
            (*Clip).Rects = NewRects;
        }
    } else {
        (*Clip).iDirection = CD_ANY;

        (*base).iDComplexity = if ((*rcBounds).top == (*rcBounds).bottom) &&
                                   ((*rcBounds).left == (*rcBounds).right) {
            DC_TRIVIAL as u8
        } else {
            DC_RECT as u8
        };

        (*base).iFComplexity = FC_RECT as u8;
        (*base).iMode = TC_RECTANGLES as u8;
        (*base).rclBounds = *rcBounds;
        (*Clip).RectCount = 1;
        if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
            EngFreeMem((*Clip).Rects as *mut c_void);
        }
        (*Clip).Rects = core::ptr::addr_of_mut!((*base).rclBounds);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngUpdateClipRegion(
    Clip: *mut XCLIPOBJ,
    count: u32,
    pRect: *const RECTL,
    rcBounds: *const RECTL)
{
    let base = clip_base(Clip);
    if count > 1 {
        let alloc_size = (core::mem::size_of::<ENUMRECTS>()
            + (count as usize - 1) * core::mem::size_of::<RECTL>()) as u32;
        let NewRects = EngAllocMem(0, alloc_size, GDITAG_CLIPOBJ) as *mut RECTL;

        if !NewRects.is_null() {
            (*Clip).RectCount = count;
            (*Clip).iDirection = CD_ANY;
            core::ptr::copy_nonoverlapping(pRect, NewRects, count as usize);

            (*base).iDComplexity = DC_COMPLEX as u8;
            (*base).iFComplexity = if (*Clip).RectCount <= 4 { FC_RECT4 as u8 } else { FC_COMPLEX as u8 };
            (*base).iMode = TC_RECTANGLES as u8;
            (*base).rclBounds = *rcBounds;

            if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
                EngFreeMem((*Clip).Rects as *mut c_void);
            }
            (*Clip).Rects = NewRects;
        }
    } else {
        (*Clip).iDirection = CD_ANY;

        (*base).iDComplexity = if ((*rcBounds).top == (*rcBounds).bottom) &&
                                   ((*rcBounds).left == (*rcBounds).right) {
            DC_TRIVIAL as u8
        } else {
            DC_RECT as u8
        };

        (*base).iFComplexity = FC_RECT as u8;
        (*base).iMode = TC_RECTANGLES as u8;
        (*base).rclBounds = *rcBounds;
        (*Clip).RectCount = 1;
        if (*Clip).Rects != core::ptr::addr_of_mut!((*base).rclBounds) {
            EngFreeMem((*Clip).Rects as *mut c_void);
        }
        (*Clip).Rects = core::ptr::addr_of_mut!((*base).rclBounds);
    }
}

/// EngCreateClip — @implemented
#[no_mangle]
pub unsafe extern "system" fn EngCreateClip() -> *mut CLIPOBJ {
    let Clip = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<XCLIPOBJ>() as u32, GDITAG_CLIPOBJ) as *mut XCLIPOBJ;
    if !Clip.is_null() {
        IntEngInitClipObj(Clip);
        TRACE!(b"Created Clip Obj %p.\n\0".as_ptr() as *const i8, Clip);
        return Clip as *mut CLIPOBJ;
    }

    ERR!("Clip object allocation failed!\n");
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn EngDeleteClip(
    pco: *mut CLIPOBJ
) {
    let pxco = pco as *mut XCLIPOBJ;
    TRACE!(b"Deleting %p.\n\0".as_ptr() as *const i8, pco);
    IntEngFreeClipResources(pxco);
    EngFreeMem(pxco as *mut c_void);
}

#[no_mangle]
pub unsafe extern "system" fn CLIPOBJ_cEnumStart(
    pco: *mut CLIPOBJ,
    bAll: i32,
    iType: u32,
    iDirection: u32,
    cMaxRects: u32,
) -> u32 {
    let Clip = pco as *mut XCLIPOBJ;
    let mut CompareFunc: SORTCOMP;

    (*Clip).bAll = bAll;
    (*Clip).iType = iType;
    (*Clip).EnumPos = 0;
    (*Clip).EnumMax = if cMaxRects > 0 { cMaxRects } else { (*Clip).RectCount };

    if CD_ANY != iDirection && (*Clip).iDirection != iDirection {
        let mut iDirection = iDirection;
        match iDirection {
            CD_RIGHTDOWN => {
                CompareFunc = Some(CompareRightDown);
            }
            CD_RIGHTUP => {
                CompareFunc = Some(CompareRightUp);
            }
            CD_LEFTDOWN => {
                CompareFunc = Some(CompareLeftDown);
            }
            CD_LEFTUP => {
                CompareFunc = Some(CompareLeftUp);
            }
            _ => {
                ERR!("Invalid iDirection %lu\n", iDirection);
                iDirection = (*Clip).iDirection;
                CompareFunc = None;
            }
        }

        if CompareFunc.is_some() {
            EngSort(
                (*Clip).Rects as PBYTE,
                core::mem::size_of::<RECTL>() as u32,
                (*Clip).RectCount,
                CompareFunc,
            );
        }

        (*Clip).iDirection = iDirection;
    }

    if (cMaxRects > 0) && ((*Clip).RectCount > cMaxRects) {
        return 0xFFFFFFFF;
    }

    (*Clip).RectCount
}
#[no_mangle]
pub unsafe extern "system" fn CLIPOBJ_bEnum(
    pco: *mut CLIPOBJ,
    cj: u32,
    pulEnumRects: *mut u32,
) -> i32 {
    // winddi.h: `BOOL APIENTRY CLIPOBJ_bEnum(...)` -- a 32-bit BOOL. Returning u8 left the
    // upper bits of the return register undefined for every C display driver that tests it.
    let Clip = pco as *mut XCLIPOBJ;
    let pERects = pulEnumRects as *mut ENUMRECTS;

    let nCopy = core::cmp::min(
        (*Clip).EnumMax - (*Clip).EnumPos,
        core::cmp::min(
            (*Clip).RectCount - (*Clip).EnumPos,
            (cj - core::mem::size_of::<u32>() as u32) / core::mem::size_of::<RECTL>() as u32,
        ),
    );

    if nCopy == 0 {
        return 0;
    }

    let src = (*Clip).Rects.add((*Clip).EnumPos as usize);
    RtlCopyMemory(
        (*pERects).arcl.as_mut_ptr() as *mut c_void,
        src as *const c_void,
        (nCopy as usize) * core::mem::size_of::<RECTL>(),
    );

    (*pERects).c = nCopy;

    (*Clip).EnumPos += nCopy;

    if (*Clip).EnumPos < (*Clip).RectCount {
        1
    } else {
        0
    }
}

// CLIPOBJ_ppoGetPath — APIENTRY (stdcall on i386, cdecl elsewhere).
// @unimplemented in C stubs.c; provide a NULL-returning stub so the link
// resolves. Real callers are display drivers (which we don't run yet).
#[no_mangle]
pub unsafe extern "system" fn CLIPOBJ_ppoGetPath(_pco: *mut CLIPOBJ) -> *mut PATHOBJ {
    core::ptr::null_mut()
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           GDI Clipping Functions
//  * FILE:              win32ss/gdi/eng/clip.c
//  * PROGRAMER:         Jason Filby
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(EngClip);
//
//
// static __inline int
// CompareRightDown(
//     const RECTL *r1,
//     const RECTL *r2)
// {
//     int Cmp;
//
//     if (r1->top < r2->top)
//     {
//         Cmp = -1;
//     }
//     else if (r2->top < r1->top)
//     {
//         Cmp = +1;
//     }
//     else
//     {
//         ASSERT(r1->bottom == r2->bottom);
//         if (r1->left < r2->left)
//         {
//             Cmp = -1;
//         }
//         else if (r2->left < r1->left)
//         {
//             Cmp = +1;
//         }
//         else
//         {
//             ASSERT(r1->right == r2->right);
//             Cmp = 0;
//         }
//     }
//
//     return Cmp;
// }
//
// static __inline int
// CompareRightUp(
//     const RECTL *r1,
//     const RECTL *r2)
// {
//     int Cmp;
//
//     if (r1->bottom < r2->bottom)
//     {
//         Cmp = +1;
//     }
//     else if (r2->bottom < r1->bottom)
//     {
//         Cmp = -1;
//     }
//     else
//     {
//         ASSERT(r1->top == r2->top);
//         if (r1->left < r2->left)
//         {
//             Cmp = -1;
//         }
//         else if (r2->left < r1->left)
//         {
//             Cmp = +1;
//         }
//         else
//         {
//             ASSERT(r1->right == r2->right);
//             Cmp = 0;
//         }
//     }
//
//     return Cmp;
// }
//
// static __inline int
// CompareLeftDown(
//     const RECTL *r1,
//     const RECTL *r2)
// {
//     int Cmp;
//
//     if (r1->top < r2->top)
//     {
//         Cmp = -1;
//     }
//     else if (r2->top < r1->top)
//     {
//         Cmp = +1;
//     }
//     else
//     {
//         ASSERT(r1->bottom == r2->bottom);
//         if (r1->right < r2->right)
//         {
//             Cmp = +1;
//         }
//         else if (r2->right < r1->right)
//         {
//             Cmp = -1;
//         }
//         else
//         {
//             ASSERT(r1->left == r2->left);
//             Cmp = 0;
//         }
//     }
//
//     return Cmp;
// }
//
// static __inline int
// CompareLeftUp(
//     const RECTL *r1,
//     const RECTL *r2)
// {
//     int Cmp;
//
//     if (r1->bottom < r2->bottom)
//     {
//         Cmp = +1;
//     }
//     else if (r2->bottom < r1->bottom)
//     {
//         Cmp = -1;
//     }
//     else
//     {
//         ASSERT(r1->top == r2->top);
//         if (r1->right < r2->right)
//         {
//             Cmp = +1;
//         }
//         else if (r2->right < r1->right)
//         {
//             Cmp = -1;
//         }
//         else
//         {
//             ASSERT(r1->left == r2->left);
//             Cmp = 0;
//         }
//     }
//     return Cmp;
// }
//
// VOID
// FASTCALL
// IntEngInitClipObj(XCLIPOBJ *Clip)
// {
//     Clip->Rects = &Clip->rclBounds;
// }
//
// VOID FASTCALL
// IntEngFreeClipResources(XCLIPOBJ *Clip)
// {
//     if (Clip->Rects != &Clip->rclBounds)
//         EngFreeMem(Clip->Rects);
// }
//
//
// VOID
// FASTCALL
// IntEngUpdateClipRegion(
//     XCLIPOBJ* Clip,
//     ULONG count,
//     const RECTL* pRect,
//     const RECTL* rcBounds)
// {
//     if(count > 1)
//     {
//         RECTL* NewRects = EngAllocMem(0, FIELD_OFFSET(ENUMRECTS, arcl[count]), GDITAG_CLIPOBJ);
//
//         if(NewRects != NULL)
//         {
//             Clip->RectCount = count;
//             Clip->iDirection = CD_ANY;
//             RtlCopyMemory(NewRects, pRect, count * sizeof(RECTL));
//
//             Clip->iDComplexity = DC_COMPLEX;
//             Clip->iFComplexity = ((Clip->RectCount <= 4) ? FC_RECT4 : FC_COMPLEX);
//             Clip->iMode = TC_RECTANGLES;
//             Clip->rclBounds = *rcBounds;
//
//             if (Clip->Rects != &Clip->rclBounds)
//                 EngFreeMem(Clip->Rects);
//             Clip->Rects = NewRects;
//         }
//     }
//     else
//     {
//         Clip->iDirection = CD_ANY;
//
//         Clip->iDComplexity = (((rcBounds->top == rcBounds->bottom) &&
//                                      (rcBounds->left == rcBounds->right))
//                                      ? DC_TRIVIAL : DC_RECT);
//
//         Clip->iFComplexity = FC_RECT;
//         Clip->iMode = TC_RECTANGLES;
//         Clip->rclBounds = *rcBounds;
//         Clip->RectCount = 1;
//         if (Clip->Rects != &Clip->rclBounds)
//             EngFreeMem(Clip->Rects);
//         Clip->Rects = &Clip->rclBounds;
//     }
// }
//
// /*
//  * @implemented
//  */
// CLIPOBJ *
// APIENTRY
// EngCreateClip(VOID)
// {
//     XCLIPOBJ *Clip = EngAllocMem(FL_ZERO_MEMORY, sizeof(XCLIPOBJ), GDITAG_CLIPOBJ);
//     if(Clip != NULL)
//     {
//         IntEngInitClipObj(Clip);
//         TRACE("Created Clip Obj %p.\n", Clip);
//         return (CLIPOBJ *)Clip;
//     }
//
//     ERR("Clip object allocation failed!\n");
//     return NULL;
// }
//
// /*
//  * @implemented
//  */
// VOID
// APIENTRY
// EngDeleteClip(
//     _In_ _Post_ptr_invalid_ CLIPOBJ *pco)
// {
//     XCLIPOBJ* pxco = (XCLIPOBJ *)pco;
//     TRACE("Deleting %p.\n", pco);
//     IntEngFreeClipResources(pxco);
//     EngFreeMem(pxco);
// }
//
// /*
//  * @implemented
//  */
// ULONG
// APIENTRY
// CLIPOBJ_cEnumStart(
//     _Inout_ CLIPOBJ *pco,
//     _In_ BOOL bAll,
//     _In_ ULONG iType,
//     _In_ ULONG iDirection,
//     _In_ ULONG cMaxRects)
// {
//     XCLIPOBJ* Clip = (XCLIPOBJ *)pco;
//     SORTCOMP CompareFunc;
//
//     Clip->bAll    = bAll;
//     Clip->iType   = iType;
//     Clip->EnumPos = 0;
//     Clip->EnumMax = (cMaxRects > 0) ? cMaxRects : Clip->RectCount;
//
//     if (CD_ANY != iDirection && Clip->iDirection != iDirection)
//     {
//         switch (iDirection)
//         {
//             case CD_RIGHTDOWN:
//                 CompareFunc = (SORTCOMP) CompareRightDown;
//                 break;
//
//             case CD_RIGHTUP:
//                 CompareFunc = (SORTCOMP) CompareRightUp;
//                 break;
//
//             case CD_LEFTDOWN:
//                 CompareFunc = (SORTCOMP) CompareLeftDown;
//                 break;
//
//             case CD_LEFTUP:
//                 CompareFunc = (SORTCOMP) CompareLeftUp;
//                 break;
//
//             default:
//                 ERR("Invalid iDirection %lu\n", iDirection);
//                 iDirection = Clip->iDirection;
//                 CompareFunc = NULL;
//                 break;
//         }
//
//         if (NULL != CompareFunc)
//         {
//             EngSort((PBYTE) Clip->Rects, sizeof(RECTL), Clip->RectCount, CompareFunc);
//         }
//
//         Clip->iDirection = iDirection;
//     }
//
//     /* Return the number of rectangles enumerated */
//     if ((cMaxRects > 0) && (Clip->RectCount > cMaxRects))
//     {
//         return 0xFFFFFFFF;
//     }
//
//     return Clip->RectCount;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// APIENTRY
// CLIPOBJ_bEnum(
//     _In_ CLIPOBJ *pco,
//     _In_ ULONG cj,
//     _Out_bytecap_(cj) ULONG *pulEnumRects)
// {
//     const RECTL* src;
//     XCLIPOBJ* Clip = (XCLIPOBJ *)pco;
//     ULONG nCopy;
//     ENUMRECTS* pERects = (ENUMRECTS*)pulEnumRects;
//
//     // Calculate how many rectangles we should copy
//     nCopy = min( Clip->EnumMax - Clip->EnumPos,
//             min( Clip->RectCount - Clip->EnumPos,
//             (cj - sizeof(ULONG)) / sizeof(RECTL)));
//
//     if(nCopy == 0)
//     {
//         return FALSE;
//     }
//
//     /* Copy rectangles */
//     src = &Clip->Rects[Clip->EnumPos];
//     RtlCopyMemory(pERects->arcl, src, nCopy * sizeof(RECTL));
//
//     pERects->c = nCopy;
//
//     Clip->EnumPos+=nCopy;
//
//     return Clip->EnumPos < Clip->RectCount;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
