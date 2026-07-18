//! Rust translation of `win32ss/user/ntuser/vis.c` — visibility computations.
//! [ntuser wave 5: vis]

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadWin32Thread;

// VerifyWnd + UserReferenceObject/Dereference are FASTCALL win32k exports.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn VerifyWnd(w: PWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn VerifyWnd(w: PWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
}

// IntSysCreateRectpRgnIndirect(prc) is a region.h macro:
//   IntSysCreateRectpRgn(prc->left, prc->top, prc->right, prc->bottom)
#[inline]
unsafe fn int_sys_create_rectp_rgn_indirect(prc: *const RECTL) -> PREGION {
    IntSysCreateRectpRgn(
        (*prc).left,
        (*prc).top,
        (*prc).right,
        (*prc).bottom,
    )
}

// UserRefObjectCo/UserDerefObjectCo are object.h static __inline over the
// FASTCALL UserReferenceObject/Dereference + the per-thread ReferencesList.
#[inline]
unsafe fn push_entry_list(head: *mut SINGLE_LIST_ENTRY, entry: *mut SINGLE_LIST_ENTRY) {
    (*entry).Next = (*head).Next;
    (*head).Next = entry;
}
#[inline]
unsafe fn pop_entry_list(head: *mut SINGLE_LIST_ENTRY) -> *mut SINGLE_LIST_ENTRY {
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    first
}
#[inline]
unsafe fn user_ref_object_co(obj: PVOID, entry: *mut USER_REFERENCE_ENTRY) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj;
    UserReferenceObject(obj);
    push_entry_list(
        core::ptr::addr_of_mut!((*w32).ReferencesList),
        core::ptr::addr_of_mut!((*entry).Entry),
    );
}
#[inline]
unsafe fn user_deref_object_co(obj: PVOID) {
    let w32: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let _ = pop_entry_list(core::ptr::addr_of_mut!((*w32).ReferencesList));
    UserDereferenceObject(obj);
}

// DBG_DEFAULT_CHANNEL(UserWinpos) — debug channel macro, no translation needed.

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn VIS_ComputeVisibleRegion(
    Wnd: PWND,
    ClientArea: BOOLEAN,
    ClipChildren: BOOLEAN,
    ClipSiblings: BOOLEAN,
) -> PREGION {
    VIS_ComputeVisibleRegion_impl(Wnd, ClientArea, ClipChildren, ClipSiblings)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn VIS_ComputeVisibleRegion(
    Wnd: PWND,
    ClientArea: BOOLEAN,
    ClipChildren: BOOLEAN,
    ClipSiblings: BOOLEAN,
) -> PREGION {
    VIS_ComputeVisibleRegion_impl(Wnd, ClientArea, ClipChildren, ClipSiblings)
}

unsafe fn VIS_ComputeVisibleRegion_impl(
    Wnd: PWND,
    ClientArea: BOOLEAN,
    ClipChildren: BOOLEAN,
    ClipSiblings: BOOLEAN,
) -> PREGION {
    let VisRgn: PREGION;
    let mut ClipRgn: PREGION;
    let mut PreviousWindow: PWND;
    let mut CurrentWindow: PWND;
    let mut CurrentSibling: PWND;

    if Wnd.is_null() || ((*Wnd).style & WS_VISIBLE as u32) == 0 {
        return core::ptr::null_mut();
    }

    if ClientArea != 0 {
        VisRgn = int_sys_create_rectp_rgn_indirect(core::ptr::addr_of!((*Wnd).rcClient));
    } else {
        VisRgn = int_sys_create_rectp_rgn_indirect(core::ptr::addr_of!((*Wnd).rcWindow));
    }

    PreviousWindow = Wnd;
    CurrentWindow = (*Wnd).spwndParent;
    while !CurrentWindow.is_null() {
        if VerifyWnd(CurrentWindow).is_null() {
            // ERR("ATM the Current Window or Parent is dead!") — retail no-op.
            if !VisRgn.is_null() {
                REGION_Delete(VisRgn);
            }
            return core::ptr::null_mut();
        }

        if ((*CurrentWindow).style & WS_VISIBLE as u32) == 0 {
            if !VisRgn.is_null() {
                REGION_Delete(VisRgn);
            }
            return core::ptr::null_mut();
        }

        ClipRgn = int_sys_create_rectp_rgn_indirect(core::ptr::addr_of!((*CurrentWindow).rcClient));
        IntGdiCombineRgn(VisRgn, VisRgn, ClipRgn, RGN_AND as i32);
        REGION_Delete(ClipRgn);

        if ((*PreviousWindow).style & WS_CLIPSIBLINGS as u32) != 0
            || (PreviousWindow == Wnd && ClipSiblings != 0)
        {
            CurrentSibling = (*CurrentWindow).spwndChild;
            while !CurrentSibling.is_null() && CurrentSibling != PreviousWindow {
                if ((*CurrentSibling).style & WS_VISIBLE as u32) != 0
                    && ((*CurrentSibling).ExStyle & WS_EX_TRANSPARENT as u32) == 0
                {
                    ClipRgn = int_sys_create_rectp_rgn_indirect(core::ptr::addr_of!(
                        (*CurrentSibling).rcWindow
                    ));
                    if !(*CurrentSibling).hrgnClip.is_null()
                        && ((*CurrentSibling).style & WS_MINIMIZE as u32) == 0
                    {
                        let SiblingClipRgn = REGION_LockRgn((*CurrentSibling).hrgnClip);
                        if !SiblingClipRgn.is_null() {
                            REGION_bOffsetRgn(
                                ClipRgn,
                                -(*CurrentSibling).rcWindow.left,
                                -(*CurrentSibling).rcWindow.top,
                            );
                            IntGdiCombineRgn(ClipRgn, ClipRgn, SiblingClipRgn, RGN_AND as i32);
                            REGION_bOffsetRgn(
                                ClipRgn,
                                (*CurrentSibling).rcWindow.left,
                                (*CurrentSibling).rcWindow.top,
                            );
                            REGION_UnlockRgn(SiblingClipRgn);
                        }
                    }
                    IntGdiCombineRgn(VisRgn, VisRgn, ClipRgn, RGN_DIFF as i32);
                    REGION_Delete(ClipRgn);
                }
                CurrentSibling = (*CurrentSibling).spwndNext;
            }
        }

        PreviousWindow = CurrentWindow;
        CurrentWindow = (*CurrentWindow).spwndParent;
    }

    if ClipChildren != 0 {
        CurrentWindow = (*Wnd).spwndChild;
        while !CurrentWindow.is_null() {
            if ((*CurrentWindow).style & WS_VISIBLE as u32) != 0
                && ((*CurrentWindow).ExStyle & WS_EX_TRANSPARENT as u32) == 0
            {
                ClipRgn = int_sys_create_rectp_rgn_indirect(core::ptr::addr_of!(
                    (*CurrentWindow).rcWindow
                ));
                if !(*CurrentWindow).hrgnClip.is_null()
                    && ((*CurrentWindow).style & WS_MINIMIZE as u32) == 0
                {
                    let CurrentRgnClip = REGION_LockRgn((*CurrentWindow).hrgnClip);
                    if !CurrentRgnClip.is_null() {
                        REGION_bOffsetRgn(
                            ClipRgn,
                            -(*CurrentWindow).rcWindow.left,
                            -(*CurrentWindow).rcWindow.top,
                        );
                        IntGdiCombineRgn(ClipRgn, ClipRgn, CurrentRgnClip, RGN_AND as i32);
                        REGION_bOffsetRgn(
                            ClipRgn,
                            (*CurrentWindow).rcWindow.left,
                            (*CurrentWindow).rcWindow.top,
                        );
                        REGION_UnlockRgn(CurrentRgnClip);
                    }
                }
                IntGdiCombineRgn(VisRgn, VisRgn, ClipRgn, RGN_DIFF as i32);
                REGION_Delete(ClipRgn);
            }
            CurrentWindow = (*CurrentWindow).spwndNext;
        }
    }

    if !(*Wnd).hrgnClip.is_null() && ((*Wnd).style & WS_MINIMIZE as u32) == 0 {
        let WndRgnClip = REGION_LockRgn((*Wnd).hrgnClip);
        if !WndRgnClip.is_null() {
            REGION_bOffsetRgn(VisRgn, -(*Wnd).rcWindow.left, -(*Wnd).rcWindow.top);
            IntGdiCombineRgn(VisRgn, VisRgn, WndRgnClip, RGN_AND as i32);
            REGION_bOffsetRgn(VisRgn, (*Wnd).rcWindow.left, (*Wnd).rcWindow.top);
            REGION_UnlockRgn(WndRgnClip);
        }
    }

    VisRgn
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_VIS_WindowLayoutChanged(Wnd: PWND, NewlyExposed: PREGION) {
    co_VIS_WindowLayoutChanged_impl(Wnd, NewlyExposed)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_VIS_WindowLayoutChanged(Wnd: PWND, NewlyExposed: PREGION) {
    co_VIS_WindowLayoutChanged_impl(Wnd, NewlyExposed)
}

unsafe fn co_VIS_WindowLayoutChanged_impl(Wnd: PWND, NewlyExposed: PREGION) {
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    let Parent: PWND = (*Wnd).spwndParent;
    if !Parent.is_null() {
        let TempRgn: PREGION = IntSysCreateRectpRgn(0, 0, 0, 0);

        if TempRgn.is_null() {
            return;
        }

        IntGdiCombineRgn(TempRgn, NewlyExposed, core::ptr::null_mut(), RGN_COPY as i32);
        REGION_bOffsetRgn(
            TempRgn,
            (*Wnd).rcWindow.left - (*Parent).rcClient.left,
            (*Wnd).rcWindow.top - (*Parent).rcClient.top,
        );

        user_ref_object_co(Parent as PVOID, &mut Ref);
        co_UserRedrawWindow(
            Parent,
            core::ptr::null(),
            TempRgn,
            RDW_FRAME | RDW_ERASE | RDW_INVALIDATE | RDW_ALLCHILDREN,
        );
        user_deref_object_co(Parent as PVOID);

        REGION_Delete(TempRgn);
    }
}
