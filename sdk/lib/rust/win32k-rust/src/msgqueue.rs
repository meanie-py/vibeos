#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePool, ExFreePoolWithTag, ExInitializePagedLookasideList, KeInitializeEvent, KeSetEvent, KeSetKernelStackSwapEnable, KeWaitForMultipleObjects, KeWaitForSingleObject, PsGetCurrentThreadWin32Thread, PsGetThreadFreezeCount, ZwYieldExecution};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

// ExAllocate/FreeToPagedLookasideList are real NTAPI ntoskrnl exports; the
// exfuncs.h inline IS this code, and calling the export avoids the i386
// SLIST-primitive plumbing. extern "system" = stdcall on i386, C elsewhere.
extern "system" {
    fn ExAllocateFromPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST) -> PVOID;
    fn ExFreeToPagedLookasideList(Lookaside: PPAGED_LOOKASIDE_LIST, Entry: PVOID);
}

// RtlEqualMemory(a,b,len) = (RtlCompareMemory(a,b,len) == len) (ntdef.h macro).
#[inline(always)]
unsafe fn RtlEqualMemory(a: *const c_void, b: *const c_void, len: SIZE_T) -> BOOLEAN {
    (ntkrnl_sys::ddk::RtlCompareMemory(a, b, len) == len) as BOOLEAN
}
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_TIMEOUT, STATUS_UNSUCCESSFUL};
use ntkrnl_sys::interlocked::{InterlockedDecrement, InterlockedExchangePointer, InterlockedIncrement};

// NTSTATUS wait codes (ntstatus.h) not in the ntkrnl-sys catalog.
const STATUS_WAIT_0: NTSTATUS = 0x00000000;
const STATUS_WAIT_1: NTSTATUS = 0x00000001;
const STATUS_WAIT_2: NTSTATUS = 0x00000002;
const STATUS_USER_APC: NTSTATUS = 0x000000C0;
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// --- NTAPI/stdcall (`_Name@N`) eng exports. extern "system" = stdcall i386. ---
extern "system" {
    fn EngGetTickCount() -> ULONGLONG;
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// EngGetTickCount32() = (ULONG)EngGetTickCount() (eng.h FORCEINLINE; no symbol).
#[inline(always)]
unsafe fn EngGetTickCount32() -> DWORD {
    EngGetTickCount() as DWORD
}

// --- FASTCALL (`@Name@N` on i386, cdecl elsewhere) win32k exports. ---
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IdlePong();
    fn IdlePing();
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntDeliverUserAPC();
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn DoTheScreenSaver();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IdlePong();
    fn IdlePing();
    fn co_IntSendMessage(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn ValidateHwndNoErr(hWnd: HWND) -> PWND;
    fn SetLastNtError(Status: NTSTATUS);
    fn co_IntDeliverUserAPC();
    fn UserReferenceObject(obj: PVOID);
    fn UserDereferenceObject(Object: PVOID) -> BOOL;
    fn DoTheScreenSaver();
}

// UserEnterCo()/UserLeaveCo() are ntuser.h macros for UserEnterExclusive()/
// UserLeave() — both FASTCALL (`@UserEnterExclusive@0`/`@UserLeave@0` on i386).
// (Matches callback.rs.)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    #[link_name = "UserEnterExclusive"]
    fn UserEnterCo();
    #[link_name = "UserLeave"]
    fn UserLeaveCo();
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    #[link_name = "UserEnterExclusive"]
    fn UserEnterCo();
    #[link_name = "UserLeave"]
    fn UserLeaveCo();
}

// NtGdiPtInRegion — win32k GDI export (stdcall `_NtGdiPtInRegion@12` on i386).
extern "system" {
    fn NtGdiPtInRegion(hrgn: HRGN, x: INT, y: INT) -> BOOL;
}

// IntPtInWindow (winpos.h FORCEINLINE) — no exported symbol, reimplement inline.
#[inline(always)]
unsafe fn IntPtInWindow(pwnd: PWND, x: INT, y: INT) -> BOOL {
    if RECTL_bPointInRect(core::ptr::addr_of!((*pwnd).rcWindow), x, y) == 0 {
        return FALSE as BOOL;
    }
    if (*pwnd).hrgnClip.is_null() || (*pwnd).style & WS_MINIMIZE != 0 {
        return TRUE as BOOL;
    }
    NtGdiPtInRegion(
        (*pwnd).hrgnClip,
        x - (*pwnd).rcWindow.left,
        y - (*pwnd).rcWindow.top,
    )
}

// --- SystemTimerProc is a CALLBACK (stdcall on i386, C elsewhere) TIMERPROC. ---
extern "system" {
    fn SystemTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD);
}

// UserHMGetHandle(obj) = ((obj)->head.h); head.h is at object offset 0.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}
// RECTL_bPointInRect / RECTL_vSetRect (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: LONG, y: LONG) -> BOOLEAN {
    (x >= (*prcl).left && x < (*prcl).right && y >= (*prcl).top && y < (*prcl).bottom) as BOOLEAN
}
#[inline(always)]
unsafe fn RECTL_vSetRect(prcl: *mut RECTL, left: LONG, top: LONG, right: LONG, bottom: LONG) {
    (*prcl).left = left;
    (*prcl).top = top;
    (*prcl).right = right;
    (*prcl).bottom = bottom;
}
// HandleToLong(h) = (LONG)(LONG_PTR)h (basetsd.h).
#[inline(always)]
fn HandleToLong(h: *const c_void) -> LONG {
    h as isize as LONG
}
// abs (C stdlib) for INT.
#[inline(always)]
fn abs(v: INT) -> INT { v.wrapping_abs() }
// Int32x32To64(a,b) = (LONGLONG)a * (LONGLONG)b (ntbasedef.h __emul).
#[inline(always)]
fn Int32x32To64(a: LONG, b: LONG) -> LONGLONG { (a as i64) * (b as i64) }

// UserRefObjectCo / UserDerefObjectCo (object.h static __inline).
#[inline(always)]
unsafe fn UserRefObjectCo<T>(obj: *mut T, entry: PUSER_REFERENCE_ENTRY) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*entry).obj = obj as PVOID;
    UserReferenceObject(obj as PVOID);
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    (*entry).Entry.Next = (*head).Next;
    (*head).Next = core::ptr::addr_of_mut!((*entry).Entry);
}
#[inline(always)]
unsafe fn UserDerefObjectCo<T>(obj: *mut T) {
    let W32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let head = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let first = (*head).Next;
    if !first.is_null() {
        (*head).Next = (*first).Next;
    }
    UserDereferenceObject(obj as PVOID);
}

const TAG_HOOK: ULONG = u32::from_le_bytes(*b"Usho");
const PWND_BOTTOM: PWND = 1 as PWND;

// ---- minwindef.h / ntuser.h macros (no symbol). Macros (not fns) so the
//      untyped `as u32` truncation accepts mixed LONG/WORD/DWORD args. ----
// MAKELONG(a,b): (WORD)a | ((WORD)b << 16)
macro_rules! MAKELONG {
    ($a:expr, $b:expr) => {{ ((($a) as u32 & 0xffff) | ((($b) as u32 & 0xffff) << 16)) as LONG }};
}
macro_rules! MAKELPARAM {
    ($a:expr, $b:expr) => {{ MAKELONG!($a, $b) as LPARAM }};
}
macro_rules! MAKEWPARAM {
    ($a:expr, $b:expr) => {{ (MAKELONG!($a, $b) as u32) as WPARAM }};
}
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }
// GET_WHEEL_DELTA_WPARAM(w) = (short)HIWORD(w); GET_XBUTTON_WPARAM(w) = HIWORD(w)
#[inline(always)]
fn GET_WHEEL_DELTA_WPARAM(w: WPARAM) -> i32 { (HIWORD(w as DWORD) as u16) as i16 as i32 }
#[inline(always)]
fn GET_XBUTTON_WPARAM(w: WPARAM) -> DWORD { HIWORD(w as DWORD) }
// KTHREAD.SuspendCount: direct field on i386; bindgen-nested under
// __bindgen_anon_6.__bindgen_anon_1 on amd64/arm64 (matches ke_defs kt_SuspendCount).
#[inline(always)]
unsafe fn kt_suspend_count(t: *mut _KTHREAD) -> i8 {
    #[cfg(target_arch = "x86")]
    { (*t).SuspendCount }
    #[cfg(not(target_arch = "x86"))]
    { (*t).__bindgen_anon_6.__bindgen_anon_1.SuspendCount }
}

// ---- input.h key-state macros (no symbol). GET_KS_BYTE(vk)=vk*2/8;
//      GET_KS_DOWN_BIT(vk)=1<<((vk%4)*2); GET_KS_LOCK_BIT(vk)=1<<((vk%4)*2+1) ----
#[inline(always)]
unsafe fn IS_KEY_DOWN<V: Into<u32>>(ks: *const u8, vk: V) -> BOOL {
    let vk = vk.into();
    if (*ks.add((vk * 2 / 8) as usize) & (1u8 << ((vk % 4) * 2))) != 0 { TRUE as BOOL } else { FALSE as BOOL }
}
#[inline(always)]
unsafe fn IS_KEY_LOCKED<V: Into<u32>>(ks: *const u8, vk: V) -> BOOL {
    let vk = vk.into();
    if (*ks.add((vk * 2 / 8) as usize) & (1u8 << ((vk % 4) * 2 + 1))) != 0 { TRUE as BOOL } else { FALSE as BOOL }
}
#[inline(always)]
unsafe fn SET_KEY_DOWN<V: Into<u32>, D: Into<i32>>(ks: *mut u8, vk: V, down: D) {
    let vk = vk.into();
    let p = ks.add((vk * 2 / 8) as usize);
    let bit = 1u8 << ((vk % 4) * 2);
    if down.into() != 0 { *p |= bit; } else { *p &= !bit; }
}
#[inline(always)]
unsafe fn SET_KEY_LOCKED<V: Into<u32>, D: Into<i32>>(ks: *mut u8, vk: V, locked: D) {
    let vk = vk.into();
    let p = ks.add((vk * 2 / 8) as usize);
    let bit = 1u8 << ((vk % 4) * 2 + 1);
    if locked.into() != 0 { *p |= bit; } else { *p &= !bit; }
}

// ---- LIST_ENTRY FORCEINLINE list ops (ntdef.h). Operate on Flink/Blink. ----
#[inline(always)]
unsafe fn InitializeListHead(h: PLIST_ENTRY) { (*h).Flink = h; (*h).Blink = h; }
#[inline(always)]
unsafe fn IsListEmpty(h: *const _LIST_ENTRY) -> BOOLEAN { ((*h).Flink == h as *mut _LIST_ENTRY) as BOOLEAN }
#[inline(always)]
unsafe fn InsertHeadList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let f = (*h).Flink; (*e).Flink = f; (*e).Blink = h; (*f).Blink = e; (*h).Flink = e;
}
#[inline(always)]
unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink; (*e).Flink = h; (*e).Blink = b; (*b).Flink = e; (*h).Blink = e;
}
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) {
    let f = (*e).Flink; let b = (*e).Blink; (*b).Flink = f; (*f).Blink = b;
}
#[inline(always)]
unsafe fn RemoveHeadList(h: PLIST_ENTRY) -> PLIST_ENTRY {
    let e = (*h).Flink; let f = (*e).Flink; (*h).Flink = f; (*f).Blink = h; e
}

// ---- bindgen-flattened field accessors (distinct offsets) ----
// gpsi->ptCursor lives in the anonymous PERUSERSERVERINFO member that bindgen
// flattened into tagSERVERINFO's __bindgen_padding_0 tail.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}
// THRDESKHEAD.pti is at object offset 2 pointer-widths (bindgen-flattened head).
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
// THREADINFO.pEThread is the first field of THREADINFO (offset 0).
#[inline(always)]
unsafe fn pti_ethread(pti: PTHREADINFO) -> PETHREAD {
    *(pti as *const PETHREAD)
}
// PROCESSINFO.W32PF_flags follows {PEPROCESS peProcess; DWORD RefCount}.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> DWORD {
    *((ppi as *const u8).add(core::mem::size_of::<usize>() + core::mem::size_of::<u32>()) as *const DWORD)
}

// SYSTEMCUR(func) = gasyscur[ROCR_##func].handle.
#[inline(always)]
unsafe fn SYSTEMCUR(rocr_index: u32) -> PCURICON_OBJECT {
    let base = core::ptr::addr_of!(gasyscur) as *const SYSTEMCURICO;
    (*base.add(rocr_index as usize)).handle
}

// IntReferenceMessageQueue / IntDereferenceMessageQueue (msgqueue.h macros).
const USERTAG_Q: ULONG = u32::from_le_bytes(*b"Usqu");
#[inline(always)]
unsafe fn IntReferenceMessageQueue(mq: PUSER_MESSAGE_QUEUE) {
    InterlockedIncrement(core::ptr::addr_of_mut!((*mq).References));
}
#[inline(always)]
unsafe fn IntDereferenceMessageQueue(mq: PUSER_MESSAGE_QUEUE) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*mq).References)) == 0 {
        ExFreePoolWithTag(mq as PVOID, USERTAG_Q);
    }
}

// IS_MOUSE_MESSAGE / IS_KBD_MESSAGE (msgqueue.h macros).
#[inline(always)]
fn IS_MOUSE_MESSAGE(message: UINT) -> BOOLEAN {
    ((message >= WM_NCMOUSEMOVE && message <= (WM_NCMOUSEMOVE + (WM_MOUSELAST - WM_MOUSEFIRST)))
        || (message >= WM_MOUSEFIRST && message <= WM_MOUSELAST)) as BOOLEAN
}
#[inline(always)]
fn IS_KBD_MESSAGE(message: UINT) -> BOOLEAN {
    (message >= WM_KEYFIRST && message <= WM_KEYLAST) as BOOLEAN
}

// ---- file-scope preamble (translated) ----
static mut pgMessageLookasideList: PPAGED_LOOKASIDE_LIST = core::ptr::null_mut();
static mut pgSendMsgLookasideList: PPAGED_LOOKASIDE_LIST = core::ptr::null_mut();
#[no_mangle]
pub static mut PostMsgCount: INT = 0;
#[no_mangle]
pub static mut SendMsgCount: INT = 0;
#[no_mangle]
pub static mut gpqCursor: PUSER_MESSAGE_QUEUE = core::ptr::null_mut();
#[no_mangle]
pub static mut gdwMouseMoveExtraInfo: ULONG_PTR = 0;
#[no_mangle]
pub static mut gdwMouseMoveTimeStamp: DWORD = 0;
// usmList — sent-message LIST_ENTRY head (msgqueue.c global). Defined here now
// that msgqueue.rs owns the file; shadows the win32k_sys bindgen extern.
#[no_mangle]
pub static mut usmList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[no_mangle]
pub unsafe extern "system" fn MsqInitializeImpl() -> NTSTATUS {
    const TAG_USRMSG: ULONG = 0x4d525355;

    pgMessageLookasideList = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PAGED_LOOKASIDE_LIST>() as SIZE_T,
        TAG_USRMSG,
    ) as *mut PAGED_LOOKASIDE_LIST;
    if pgMessageLookasideList.is_null() {
        return STATUS_NO_MEMORY;
    }
    ExInitializePagedLookasideList(
        pgMessageLookasideList as _,
        None,
        None,
        0,
        core::mem::size_of::<USER_MESSAGE>() as _,
        TAG_USRMSG,
        256,
    );

    pgSendMsgLookasideList = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<PAGED_LOOKASIDE_LIST>() as SIZE_T,
        TAG_USRMSG,
    ) as *mut PAGED_LOOKASIDE_LIST;
    if pgSendMsgLookasideList.is_null() {
        return STATUS_NO_MEMORY;
    }
    ExInitializePagedLookasideList(
        pgSendMsgLookasideList as _,
        None,
        None,
        0,
        core::mem::size_of::<USER_SENT_MESSAGE>() as SIZE_T,
        TAG_USRMSG,
        16,
    );

    InitializeListHead(core::ptr::addr_of_mut!(usmList));

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntTopLevelWindowFromPoint(x: INT, y: INT) -> PWND {
    IntTopLevelWindowFromPoint_impl(x, y)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntTopLevelWindowFromPoint(x: INT, y: INT) -> PWND {
    IntTopLevelWindowFromPoint_impl(x, y)
}

unsafe fn IntTopLevelWindowFromPoint_impl(x: INT, y: INT) -> PWND {
    let pwndDesktop: PWND = UserGetDesktopWindow();
    if pwndDesktop.is_null() {
        return core::ptr::null_mut();
    }

    let mut pWnd: PWND = (*pwndDesktop).spwndChild;
    while !pWnd.is_null() {
        if ((*pWnd).state2 & WNDS2_INDESTROY) != 0 || ((*pWnd).state & WNDS_DESTROYED) != 0 {
            pWnd = (*pWnd).spwndNext;
            continue;
        }

        if ((*pWnd).style & WS_VISIBLE) != 0
            && ((*pWnd).ExStyle & (WS_EX_LAYERED | WS_EX_TRANSPARENT))
                != (WS_EX_LAYERED | WS_EX_TRANSPARENT)
            && IntPtInWindow(pWnd, x, y) != 0
        {
            return pWnd;
        }

        pWnd = (*pWnd).spwndNext;
    }

    pwndDesktop
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserSetCursor(
    NewCursor: PCURICON_OBJECT,
    ForceChange: BOOL,
) -> PCURICON_OBJECT {
    UserSetCursor_impl(NewCursor, ForceChange)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserSetCursor(
    NewCursor: PCURICON_OBJECT,
    ForceChange: BOOL,
) -> PCURICON_OBJECT {
    UserSetCursor_impl(NewCursor, ForceChange)
}

unsafe fn UserSetCursor_impl(
    NewCursor: PCURICON_OBJECT,
    ForceChange: BOOL,
) -> PCURICON_OBJECT {
    let OldCursor: PCURICON_OBJECT;
    let hdcScreen: HDC;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let pWnd: PWND;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    OldCursor = (*MessageQueue).CursorObject;

    /* Check if cursors are different */
    if OldCursor == NewCursor {
        return OldCursor;
    }

    /* Update cursor for this message queue */
    (*MessageQueue).CursorObject = NewCursor;

    /* If cursor is not visible we have nothing to do */
    if (*MessageQueue).iCursorLevel < 0 {
        return OldCursor;
    }

    // Fixes the error message "Not the same cursor!".
    if gpqCursor.is_null() {
        gpqCursor = MessageQueue;
    }

    /* Update cursor if this message queue controls it */
    pWnd = IntTopLevelWindowFromPoint((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y);
    if !pWnd.is_null() && (*head_pti(pWnd)).MessageQueue == MessageQueue {
        /* Get the screen DC */
        hdcScreen = IntGetScreenDC();
        if hdcScreen.is_null() {
            return core::ptr::null_mut();
        }

        if !NewCursor.is_null() {
            /* Call GDI to set the new screen cursor */
            let mut CursorFrame: PCURICON_OBJECT = NewCursor;
            if (*NewCursor).CURSORF_flags & CURSORF_ACON != 0 {
                CursorFrame = *(*(NewCursor as PACON)).aspcur;
            }
            GreSetPointerShape(
                hdcScreen,
                if !(*CursorFrame).hbmAlpha.is_null() {
                    core::ptr::null_mut()
                } else {
                    (*NewCursor).hbmMask
                },
                if !(*CursorFrame).hbmAlpha.is_null() {
                    (*NewCursor).hbmAlpha
                } else {
                    (*NewCursor).hbmColor
                },
                (*CursorFrame).xHotspot as LONG,
                (*CursorFrame).yHotspot as LONG,
                (*gpsi_per_user()).ptCursor.x,
                (*gpsi_per_user()).ptCursor.y,
                if !(*CursorFrame).hbmAlpha.is_null() {
                    SPS_ALPHA as FLONG
                } else {
                    0
                },
            );
        } else {
            /* Remove the cursor */
            GreMovePointer(hdcScreen, -1, -1);
        }
        (*IntGetSysCursorInfo()).CurrentCursorObject = NewCursor;
    }

    /* Return the old cursor */
    OldCursor
}

#[no_mangle]
pub unsafe extern "C" fn UserShowCursor(bShow: BOOL) -> core::ffi::c_int {
    let hdcScreen: HDC;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let pWnd: PWND;

    hdcScreen = IntGetScreenDC();
    if hdcScreen.is_null() {
        return -1; /* No mouse */
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    /* Update counter */
    (*MessageQueue).iCursorLevel += if bShow != 0 { 1 } else { -1 };
    (*pti).iCursorLevel += if bShow != 0 { 1 } else { -1 };

    /* Check for trivial cases */
    if (bShow != 0 && (*MessageQueue).iCursorLevel != 0)
        || (bShow == 0 && (*MessageQueue).iCursorLevel != -1)
    {
        return (*MessageQueue).iCursorLevel;
    }

    /* Check if cursor is above window owned by this MessageQueue */
    pWnd = IntTopLevelWindowFromPoint((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y);
    if !pWnd.is_null() && (*head_pti(pWnd)).MessageQueue == MessageQueue {
        if bShow != 0 {
            /* Show the pointer */
            GreMovePointer(hdcScreen, (*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y);
        } else {
            /* Remove the pointer */
            GreMovePointer(hdcScreen, -1, -1);
        }

        /* Update global info */
        (*IntGetSysCursorInfo()).ShowingCursor = (*MessageQueue).iCursorLevel;
    }

    (*MessageQueue).iCursorLevel
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetKeyState(dwKey: DWORD) -> DWORD {
    UserGetKeyState_impl(dwKey)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetKeyState(dwKey: DWORD) -> DWORD {
    UserGetKeyState_impl(dwKey)
}

unsafe fn UserGetKeyState_impl(dwKey: DWORD) -> DWORD {
    let mut dwRet: DWORD = 0;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    if dwKey < 0x100 {
        if IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), dwKey) != 0 {
            dwRet |= 0xFF80;
        }
        if IS_KEY_LOCKED((*MessageQueue).afKeyState.as_mut_ptr(), dwKey) != 0 {
            dwRet |= 0x1;
        }
    } else {
        EngSetLastError(ERROR_INVALID_PARAMETER);
    }
    dwRet
}

unsafe fn UpdateKeyState(MessageQueue: PUSER_MESSAGE_QUEUE, wVk: WORD, bIsDown: BOOL) {
    TRACE!(b"UpdateKeyState wVk: %u, bIsDown: %d\n\0".as_ptr() as *const i8, wVk as u32, bIsDown);

    if bIsDown != 0 {
        /* If it's first key down event, xor lock bit */
        if IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), wVk) == 0 {
            let locked = IS_KEY_LOCKED((*MessageQueue).afKeyState.as_mut_ptr(), wVk);
            SET_KEY_LOCKED((*MessageQueue).afKeyState.as_mut_ptr(), wVk, (locked == 0) as i32);
        }

        SET_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), wVk, 1);
        (*MessageQueue).afKeyRecentDown[(wVk / 8) as usize] |= (1 << (wVk % 8)) as u8;
    } else {
        SET_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), wVk, 0);
    }
}

unsafe fn UpdateKeyStateFromMsg(MessageQueue: PUSER_MESSAGE_QUEUE, msg: *mut MSG) {
    let mut down: BOOL = 0;

    TRACE!(b"UpdateKeyStateFromMsg message:%u\n\0".as_ptr() as *const i8, (*msg).message);

    match (*msg).message {
        WM_LBUTTONDOWN => {
            down = 1;
            UpdateKeyState(MessageQueue, VK_LBUTTON as WORD, down);
        }
        WM_LBUTTONUP => {
            UpdateKeyState(MessageQueue, VK_LBUTTON as WORD, down);
        }
        WM_MBUTTONDOWN => {
            down = 1;
            UpdateKeyState(MessageQueue, VK_MBUTTON as WORD, down);
        }
        WM_MBUTTONUP => {
            UpdateKeyState(MessageQueue, VK_MBUTTON as WORD, down);
        }
        WM_RBUTTONDOWN => {
            down = 1;
            UpdateKeyState(MessageQueue, VK_RBUTTON as WORD, down);
        }
        WM_RBUTTONUP => {
            UpdateKeyState(MessageQueue, VK_RBUTTON as WORD, down);
        }
        WM_XBUTTONDOWN => {
            down = 1;
            if (*msg).wParam == XBUTTON1 as WPARAM {
                UpdateKeyState(MessageQueue, VK_XBUTTON1 as WORD, down);
            } else if (*msg).wParam == XBUTTON2 as WPARAM {
                UpdateKeyState(MessageQueue, VK_XBUTTON2 as WORD, down);
            }
        }
        WM_XBUTTONUP => {
            if (*msg).wParam == XBUTTON1 as WPARAM {
                UpdateKeyState(MessageQueue, VK_XBUTTON1 as WORD, down);
            } else if (*msg).wParam == XBUTTON2 as WPARAM {
                UpdateKeyState(MessageQueue, VK_XBUTTON2 as WORD, down);
            }
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            down = 1;
            let key = (*msg).wParam as UCHAR;
            UpdateKeyState(MessageQueue, key as WORD, down);
            match key as i32 {
                _ if key as i32 == VK_LCONTROL as i32 || key as i32 == VK_RCONTROL as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LCONTROL as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RCONTROL as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_CONTROL as WORD, down);
                }
                _ if key as i32 == VK_LMENU as i32 || key as i32 == VK_RMENU as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LMENU as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RMENU as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_MENU as WORD, down);
                }
                _ if key as i32 == VK_LSHIFT as i32 || key as i32 == VK_RSHIFT as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LSHIFT as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RSHIFT as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_SHIFT as WORD, down);
                }
                _ => {}
            }
        }
        WM_KEYUP | WM_SYSKEYUP => {
            let key = (*msg).wParam as UCHAR;
            UpdateKeyState(MessageQueue, key as WORD, down);
            match key as i32 {
                _ if key as i32 == VK_LCONTROL as i32 || key as i32 == VK_RCONTROL as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LCONTROL as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RCONTROL as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_CONTROL as WORD, down);
                }
                _ if key as i32 == VK_LMENU as i32 || key as i32 == VK_RMENU as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LMENU as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RMENU as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_MENU as WORD, down);
                }
                _ if key as i32 == VK_LSHIFT as i32 || key as i32 == VK_RSHIFT as i32 => {
                    down = (IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_LSHIFT as WORD) != 0
                        || IS_KEY_DOWN((*MessageQueue).afKeyState.as_mut_ptr(), VK_RSHIFT as WORD) != 0) as BOOL;
                    UpdateKeyState(MessageQueue, VK_SHIFT as WORD, down);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqGetDownKeyState(MessageQueue: PUSER_MESSAGE_QUEUE) -> WPARAM {
    MsqGetDownKeyState_impl(MessageQueue)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqGetDownKeyState(MessageQueue: PUSER_MESSAGE_QUEUE) -> WPARAM {
    MsqGetDownKeyState_impl(MessageQueue)
}

unsafe fn MsqGetDownKeyState_impl(MessageQueue: PUSER_MESSAGE_QUEUE) -> WPARAM {
    let mut ret: WPARAM = 0;

    let afKeyState = (*MessageQueue).afKeyState.as_ptr();

    if gspv.bMouseBtnSwap != 0 {
        if IS_KEY_DOWN(afKeyState, VK_RBUTTON) != 0 {
            ret |= MK_LBUTTON as WPARAM;
        }
        if IS_KEY_DOWN(afKeyState, VK_LBUTTON) != 0 {
            ret |= MK_RBUTTON as WPARAM;
        }
    } else {
        if IS_KEY_DOWN(afKeyState, VK_LBUTTON) != 0 {
            ret |= MK_LBUTTON as WPARAM;
        }
        if IS_KEY_DOWN(afKeyState, VK_RBUTTON) != 0 {
            ret |= MK_RBUTTON as WPARAM;
        }
    }

    if IS_KEY_DOWN(afKeyState, VK_MBUTTON) != 0 {
        ret |= MK_MBUTTON as WPARAM;
    }
    if IS_KEY_DOWN(afKeyState, VK_SHIFT) != 0 {
        ret |= MK_SHIFT as WPARAM;
    }
    if IS_KEY_DOWN(afKeyState, VK_CONTROL) != 0 {
        ret |= MK_CONTROL as WPARAM;
    }
    if IS_KEY_DOWN(afKeyState, VK_XBUTTON1) != 0 {
        ret |= MK_XBUTTON1 as WPARAM;
    }
    if IS_KEY_DOWN(afKeyState, VK_XBUTTON2) != 0 {
        ret |= MK_XBUTTON2 as WPARAM;
    }
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntMsqSetWakeMask(WakeMask: DWORD) -> HANDLE {
    IntMsqSetWakeMaskImpl(WakeMask)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntMsqSetWakeMask(WakeMask: DWORD) -> HANDLE {
    IntMsqSetWakeMaskImpl(WakeMask)
}

unsafe fn IntMsqSetWakeMaskImpl(WakeMask: DWORD) -> HANDLE {
    let Win32Thread: PTHREADINFO;
    let MessageEventHandle: HANDLE;
    let dwFlags: DWORD = HIWORD(WakeMask) as DWORD;

    Win32Thread = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if Win32Thread.is_null() || (*Win32Thread).MessageQueue.is_null() {
        return core::ptr::null_mut();
    }

    MessageEventHandle = (*Win32Thread).hEventQueueClient;

    if !(*Win32Thread).pcti.is_null() {
        if ((*(*Win32Thread).pcti).fsChangeBits as DWORD & LOWORD(WakeMask) as DWORD) != 0
            || ((dwFlags & MWMO_INPUTAVAILABLE) != 0
                && ((*(*Win32Thread).pcti).fsWakeBits as DWORD & LOWORD(WakeMask) as DWORD) != 0)
        {
            ERR!(
                "Chg 0x%x Wake 0x%x Mask 0x%x\n",
                (*(*Win32Thread).pcti).fsChangeBits as core::ffi::c_uint,
                (*(*Win32Thread).pcti).fsWakeBits as core::ffi::c_uint,
                WakeMask,
            );
            KeSetEvent((*Win32Thread).pEventQueueServer as _, IO_NO_INCREMENT as KPRIORITY, 0u8);
            return MessageEventHandle;
        }
    }

    IdlePing();

    MessageEventHandle
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntMsqClearWakeMask() -> BOOL {
    IntMsqClearWakeMaskImpl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntMsqClearWakeMask() -> BOOL {
    IntMsqClearWakeMaskImpl()
}

unsafe fn IntMsqClearWakeMaskImpl() -> BOOL {
    let Win32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if Win32Thread.is_null() || (*Win32Thread).MessageQueue.is_null() {
        return 0;
    }
    // Very hacky, but that is what they do.
    (*(*Win32Thread).pcti).fsWakeBits = 0;

    IdlePong();

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL) {
    MsqWakeQueue_impl(pti, MessageBits, KeyEvent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL) {
    MsqWakeQueue_impl(pti, MessageBits, KeyEvent)
}

unsafe fn MsqWakeQueue_impl(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL) {
    let Queue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    if (*Queue).QF_flags & QF_INDESTROY != 0 {
        ERR!("This Message Queue is in Destroy!\n");
    }
    (*(*pti).pcti).fsWakeBits |= MessageBits as WORD;
    (*(*pti).pcti).fsChangeBits |= MessageBits as WORD;

    if MessageBits & QS_KEY != 0 {
        (*pti).nCntsQBits[QSRosKey as usize] += 1;
    }
    if MessageBits & QS_MOUSE != 0 {
        if MessageBits & QS_MOUSEMOVE != 0 {
            (*pti).nCntsQBits[QSRosMouseMove as usize] += 1;
        }
        if MessageBits & QS_MOUSEBUTTON != 0 {
            (*pti).nCntsQBits[QSRosMouseButton as usize] += 1;
        }
    }
    if MessageBits & QS_POSTMESSAGE != 0 {
        (*pti).nCntsQBits[QSRosPostMessage as usize] += 1;
    }
    if MessageBits & QS_SENDMESSAGE != 0 {
        (*pti).nCntsQBits[QSRosSendMessage as usize] += 1;
    }
    if MessageBits & QS_HOTKEY != 0 {
        (*pti).nCntsQBits[QSRosHotKey as usize] += 1;
    }
    if MessageBits & QS_EVENT != 0 {
        (*pti).nCntsQBits[QSRosEvent as usize] += 1;
    }

    if KeyEvent != 0 {
        KeSetEvent((*pti).pEventQueueServer as _, IO_NO_INCREMENT as KPRIORITY, 0u8);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT) {
    ClearMsgBitsMask_impl(pti, MessageBits)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT) {
    ClearMsgBitsMask_impl(pti, MessageBits)
}

unsafe fn ClearMsgBitsMask_impl(pti: PTHREADINFO, MessageBits: UINT) {
    let mut ClrMask: UINT = 0;

    if MessageBits & QS_KEY != 0 {
        (*pti).nCntsQBits[QSRosKey as usize] -= 1;
        if (*pti).nCntsQBits[QSRosKey as usize] == 0 {
            ClrMask |= QS_KEY;
        }
    }
    if MessageBits & QS_MOUSEMOVE != 0 {
        if (*pti).nCntsQBits[QSRosMouseMove as usize] != 0 {
            (*pti).nCntsQBits[QSRosMouseMove as usize] = 0;
            ClrMask |= QS_MOUSEMOVE;
        }
    }
    if MessageBits & QS_MOUSEBUTTON != 0 {
        (*pti).nCntsQBits[QSRosMouseButton as usize] -= 1;
        if (*pti).nCntsQBits[QSRosMouseButton as usize] == 0 {
            ClrMask |= QS_MOUSEBUTTON;
        }
    }
    if MessageBits & QS_POSTMESSAGE != 0 {
        (*pti).nCntsQBits[QSRosPostMessage as usize] -= 1;
        if (*pti).nCntsQBits[QSRosPostMessage as usize] == 0 {
            ClrMask |= QS_POSTMESSAGE;
        }
    }
    if MessageBits & QS_TIMER != 0 {
        if (*pti).cTimersReady != 0 {
            (*pti).cTimersReady -= 1;
            if (*pti).cTimersReady == 0 {
                ClrMask |= QS_TIMER;
            }
        }
    }
    if MessageBits & QS_PAINT != 0 {
        if (*pti).cPaintsReady != 0 {
            (*pti).cPaintsReady -= 1;
            if (*pti).cPaintsReady == 0 {
                ClrMask |= QS_PAINT;
            }
        }
    }
    if MessageBits & QS_SENDMESSAGE != 0 {
        (*pti).nCntsQBits[QSRosSendMessage as usize] -= 1;
        if (*pti).nCntsQBits[QSRosSendMessage as usize] == 0 {
            ClrMask |= QS_SENDMESSAGE;
        }
    }
    if MessageBits & QS_HOTKEY != 0 {
        (*pti).nCntsQBits[QSRosHotKey as usize] -= 1;
        if (*pti).nCntsQBits[QSRosHotKey as usize] == 0 {
            ClrMask |= QS_HOTKEY;
        }
    }
    if MessageBits & QS_EVENT != 0 {
        (*pti).nCntsQBits[QSRosEvent as usize] -= 1;
        if (*pti).nCntsQBits[QSRosEvent as usize] == 0 {
            ClrMask |= QS_EVENT;
        }
    }

    (*(*pti).pcti).fsWakeBits &= !(ClrMask as WORD);
    (*(*pti).pcti).fsChangeBits &= !(ClrMask as WORD);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqIncPaintCountQueue(pti: PTHREADINFO) {
    MsqIncPaintCountQueue_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqIncPaintCountQueue(pti: PTHREADINFO) {
    MsqIncPaintCountQueue_impl(pti)
}

unsafe fn MsqIncPaintCountQueue_impl(pti: PTHREADINFO) {
    (*pti).cPaintsReady += 1;
    MsqWakeQueue(pti, QS_PAINT, 1);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqDecPaintCountQueue(pti: PTHREADINFO) {
    MsqDecPaintCountQueue_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqDecPaintCountQueue(pti: PTHREADINFO) {
    MsqDecPaintCountQueue_impl(pti)
}

unsafe fn MsqDecPaintCountQueue_impl(pti: PTHREADINFO) {
    ClearMsgBitsMask(pti, QS_PAINT);
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsqPostMouseMove(pti: PTHREADINFO, Msg: *mut MSG, ExtraInfo: LONG_PTR) {
    MsqPostMouseMove_impl(pti, Msg, ExtraInfo)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsqPostMouseMove(pti: PTHREADINFO, Msg: *mut MSG, ExtraInfo: LONG_PTR) {
    MsqPostMouseMove_impl(pti, Msg, ExtraInfo)
}

unsafe fn MsqPostMouseMove_impl(pti: PTHREADINFO, Msg: *mut MSG, ExtraInfo: LONG_PTR) {
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead);

    if IsListEmpty((*ListHead).Flink) == 0 {
        let Message: PUSER_MESSAGE = ((*ListHead).Blink as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as PUSER_MESSAGE;

        if (*Message).Msg.message == WM_MOUSEMOVE {
            (*Message).Msg = *Msg;

            MsqWakeQueue(pti, QS_MOUSEMOVE, 1);
            return;
        }
    }

    MsqPostMessage(pti, Msg, 1u8, QS_MOUSEMOVE, 0, ExtraInfo);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCoalesceMouseMove(pti: PTHREADINFO) {
    IntCoalesceMouseMove_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCoalesceMouseMove(pti: PTHREADINFO) {
    IntCoalesceMouseMove_impl(pti)
}

unsafe fn IntCoalesceMouseMove_impl(pti: PTHREADINFO) {
    let mut Msg: MSG = core::mem::zeroed();

    // Force time stamp to update, keeping message time in sync.
    if gdwMouseMoveTimeStamp == 0 {
        gdwMouseMoveTimeStamp = EngGetTickCount32();
    }

    // Build mouse move message.
    Msg.hwnd = core::ptr::null_mut();
    Msg.message = WM_MOUSEMOVE;
    Msg.wParam = 0;
    Msg.lParam = MAKELONG!((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y) as LPARAM;
    Msg.time = gdwMouseMoveTimeStamp;
    Msg.pt = (*gpsi_per_user()).ptCursor;

    // Post the move.
    MsqPostMouseMove(pti, core::ptr::addr_of_mut!(Msg), gdwMouseMoveExtraInfo as LONG_PTR);

    // Zero the time stamp.
    gdwMouseMoveTimeStamp = 0;

    // Clear flag since the move was posted.
    (*(*pti).MessageQueue).QF_flags &= !QF_MOUSEMOVED;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqInsertMouseMessage(
    Msg: *mut MSG,
    flags: DWORD,
    dwExtraInfo: ULONG_PTR,
    Hook: BOOL,
) {
    co_MsqInsertMouseMessage_impl(Msg, flags, dwExtraInfo, Hook)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqInsertMouseMessage(
    Msg: *mut MSG,
    flags: DWORD,
    dwExtraInfo: ULONG_PTR,
    Hook: BOOL,
) {
    co_MsqInsertMouseMessage_impl(Msg, flags, dwExtraInfo, Hook)
}

unsafe fn co_MsqInsertMouseMessage_impl(
    Msg: *mut MSG,
    flags: DWORD,
    dwExtraInfo: ULONG_PTR,
    Hook: BOOL,
) {
    let mut MouseHookData: MSLLHOOKSTRUCT = core::mem::zeroed();
    let pwnd: PWND;
    let pwndDesktop: PWND;
    let hdcScreen: HDC;
    let mut pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let CurInfo: PSYSTEM_CURSORINFO;

    (*Msg).time = EngGetTickCount32();

    MouseHookData.pt.x = LOWORD((*Msg).lParam as DWORD) as LONG;
    MouseHookData.pt.y = HIWORD((*Msg).lParam as DWORD) as LONG;
    match (*Msg).message {
        x if x == WM_MOUSEWHEEL => {
            MouseHookData.mouseData = MAKELONG!(0, GET_WHEEL_DELTA_WPARAM((*Msg).wParam)) as DWORD;
        }
        x if x == WM_XBUTTONDOWN
            || x == WM_XBUTTONUP
            || x == WM_XBUTTONDBLCLK
            || x == WM_NCXBUTTONDOWN
            || x == WM_NCXBUTTONUP
            || x == WM_NCXBUTTONDBLCLK =>
        {
            MouseHookData.mouseData = MAKELONG!(0, HIWORD((*Msg).wParam as DWORD)) as DWORD;
        }
        _ => {
            MouseHookData.mouseData = 0;
        }
    }

    MouseHookData.flags = flags;
    MouseHookData.time = (*Msg).time;
    MouseHookData.dwExtraInfo = dwExtraInfo;

    if Hook != 0 {
        if co_HOOK_CallHooks(
            WH_MOUSE_LL as INT,
            HC_ACTION as INT,
            (*Msg).message as WPARAM,
            core::ptr::addr_of_mut!(MouseHookData) as LPARAM,
        ) != 0
        {
            return;
        }
    }

    pwndDesktop = UserGetDesktopWindow();
    if pwndDesktop.is_null() {
        // Bounded drop trace (2026-07-07 doom-playable triage): every mouse
        // event is discarded while there is no desktop window.
        static mut MOU_NODESK_TRACE: u32 = 0;
        if MOU_NODESK_TRACE < 4 {
            MOU_NODESK_TRACE += 1;
            extern "C" {
                fn DbgPrint(Format: *const u8, ...) -> u32;
            }
            DbgPrint(b"[WIN32K] MOUDROP: no desktop window (msg=0x%x)\n\0".as_ptr(), (*Msg).message);
        }
        return;
    }

    (*Msg).hwnd = IntGetCaptureWindow();
    if !(*Msg).hwnd.is_null() {
        pwnd = UserGetWindowObject((*Msg).hwnd);
    } else {
        pwnd = IntTopLevelWindowFromPoint((*Msg).pt.x, (*Msg).pt.y);
        if !pwnd.is_null() {
            (*Msg).hwnd = UserHMGetHandle(pwnd);
        }
    }

    hdcScreen = IntGetScreenDC();
    CurInfo = IntGetSysCursorInfo();

    if !(*Msg).hwnd.is_null() && !pwnd.is_null() {
        pti = head_pti(pwnd);
        MessageQueue = (*pti).MessageQueue;

        if (*MessageQueue).QF_flags & QF_INDESTROY != 0 {
            // Bounded drop trace (ERR! is a no-op macro — make this visible).
            static mut MOU_DEADQ_TRACE: u32 = 0;
            if MOU_DEADQ_TRACE < 4 {
                MOU_DEADQ_TRACE += 1;
                extern "C" {
                    fn DbgPrint(Format: *const u8, ...) -> u32;
                }
                DbgPrint(b"[WIN32K] MOUDROP: dead message queue (msg=0x%x)\n\0".as_ptr(), (*Msg).message);
            }
            return;
        }

        if pti != (*MessageQueue).ptiMouse && (*MessageQueue).cThreads > 1 {
            pti = (*MessageQueue).ptiMouse;
        }

        if (*Msg).message == WM_MOUSEMOVE {
            if !hdcScreen.is_null()
                && !(*MessageQueue).CursorObject.is_null()
                && (*MessageQueue).iCursorLevel >= 0
            {
                if (*CurInfo).CurrentCursorObject != (*MessageQueue).CursorObject {
                    let curobj = (*MessageQueue).CursorObject;
                    GreSetPointerShape(
                        hdcScreen,
                        if !(*curobj).hbmAlpha.is_null() {
                            core::ptr::null_mut()
                        } else {
                            (*curobj).hbmMask
                        },
                        if !(*curobj).hbmAlpha.is_null() {
                            (*curobj).hbmAlpha
                        } else {
                            (*curobj).hbmColor
                        },
                        (*curobj).xHotspot as LONG,
                        (*curobj).yHotspot as LONG,
                        (*gpsi_per_user()).ptCursor.x,
                        (*gpsi_per_user()).ptCursor.y,
                        if !(*curobj).hbmAlpha.is_null() {
                            SPS_ALPHA as FLONG
                        } else {
                            0
                        },
                    );
                } else {
                    GreMovePointer(hdcScreen, (*Msg).pt.x, (*Msg).pt.y);
                }
            } else if (*CurInfo).ShowingCursor >= 0 {
                GreMovePointer(hdcScreen, -1, -1);
            }

            (*CurInfo).ShowingCursor = (*MessageQueue).iCursorLevel;
            (*CurInfo).CurrentCursorObject = (*MessageQueue).CursorObject;
            gpqCursor = MessageQueue;

            (*MessageQueue).QF_flags |= QF_MOUSEMOVED;
            gdwMouseMoveExtraInfo = dwExtraInfo;
            gdwMouseMoveTimeStamp = (*Msg).time;
            MsqWakeQueue(pti, QS_MOUSEMOVE, 1);
        } else {
            if IntGetCaptureWindow().is_null() {
            }

            if (*(*pti).MessageQueue).QF_flags & QF_MOUSEMOVED != 0 {
                IntCoalesceMouseMove(pti);
            }

            TRACE!(
                b"Posting mouse message to hwnd=%p!\n\0".as_ptr() as *const i8,
                UserHMGetHandle(pwnd),
            );
            MsqPostMessage(pti, Msg, 1, QS_MOUSEBUTTON, 0, dwExtraInfo as LONG_PTR);
        }
    } else if !hdcScreen.is_null() {
        // Bounded drop trace (2026-07-07 doom-playable triage, mouse parity
        // with keyboard.rs KEYDROP): a BUTTON event with no window at the
        // cursor point vanishes silently here (moves are normal desktop
        // traffic — don't trace those).
        if (*Msg).message != WM_MOUSEMOVE {
            static mut MOU_NOWND_TRACE: u32 = 0;
            if MOU_NOWND_TRACE < 8 {
                MOU_NOWND_TRACE += 1;
                extern "C" {
                    fn DbgPrint(Format: *const u8, ...) -> u32;
                }
                DbgPrint(
                    b"[WIN32K] MOUDROP: no window at point (%d,%d) msg=0x%x\n\0".as_ptr(),
                    (*Msg).pt.x,
                    (*Msg).pt.y,
                    (*Msg).message,
                );
            }
        }
        GreMovePointer(hdcScreen, (*Msg).pt.x, (*Msg).pt.y);
        (*CurInfo).ShowingCursor = 0;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsqCreateMessage(Msg: LPMSG) -> PUSER_MESSAGE {
    MsqCreateMessage_impl(Msg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsqCreateMessage(Msg: LPMSG) -> PUSER_MESSAGE {
    MsqCreateMessage_impl(Msg)
}

#[inline]
unsafe fn MsqCreateMessage_impl(Msg: LPMSG) -> PUSER_MESSAGE {
    let Message: PUSER_MESSAGE = ExAllocateFromPagedLookasideList(pgMessageLookasideList) as PUSER_MESSAGE;
    if Message.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(Message as PVOID, core::mem::size_of::<USER_MESSAGE>() as _);
    core::ptr::copy(
        Msg as *const MSG,
        core::ptr::addr_of_mut!((*Message).Msg),
        1,
    );
    PostMsgCount += 1;
    Message
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqDestroyMessage(Message: PUSER_MESSAGE) {
    if (*Message).pti.is_null() {
        return;
    }
    RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
    (*Message).pti = core::ptr::null_mut();
    ExFreeToPagedLookasideList(pgMessageLookasideList, Message as PVOID);
    PostMsgCount -= 1;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqDestroyMessage(Message: PUSER_MESSAGE) {
    if (*Message).pti.is_null() {
        return;
    }
    RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
    (*Message).pti = core::ptr::null_mut();
    ExFreeToPagedLookasideList(pgMessageLookasideList, Message as PVOID);
    PostMsgCount -= 1;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn AllocateUserMessage(KEvent: BOOL) -> PUSER_SENT_MESSAGE {
    AllocateUserMessage_impl(KEvent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn AllocateUserMessage(KEvent: BOOL) -> PUSER_SENT_MESSAGE {
    AllocateUserMessage_impl(KEvent)
}

unsafe fn AllocateUserMessage_impl(KEvent: BOOL) -> PUSER_SENT_MESSAGE {
    let Message: PUSER_SENT_MESSAGE = ExAllocateFromPagedLookasideList(pgSendMsgLookasideList) as PUSER_SENT_MESSAGE;

    if Message.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(Message as PVOID, core::mem::size_of::<USER_SENT_MESSAGE>() as _);

    if KEvent != 0 {
        (*Message).pkCompletionEvent = core::ptr::addr_of_mut!((*Message).CompletionEvent);

        KeInitializeEvent((*Message).pkCompletionEvent as _, _EVENT_TYPE_NotificationEvent, 0u8);
    }

    SendMsgCount += 1;

    Message
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn FreeUserMessage(Message: PUSER_SENT_MESSAGE) {
    FreeUserMessage_impl(Message)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn FreeUserMessage(Message: PUSER_SENT_MESSAGE) {
    FreeUserMessage_impl(Message)
}

unsafe fn FreeUserMessage_impl(Message: PUSER_SENT_MESSAGE) {
    (*Message).pkCompletionEvent = core::ptr::null_mut();

    RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));

    ExFreeToPagedLookasideList(pgSendMsgLookasideList, Message as PVOID);
    SendMsgCount -= 1;
}

#[no_mangle]
pub unsafe extern "system" fn MsqRemoveWindowMessagesFromQueue(Window: PWND) {
    let pti: PTHREADINFO;
    let mut SentMessage: PUSER_SENT_MESSAGE;
    let mut PostedMessage: PUSER_MESSAGE;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut ListHead: PLIST_ENTRY;

    pti = head_pti(Window);

    CurrentEntry = (*pti).PostedMessagesListHead.Flink;
    ListHead = core::ptr::addr_of_mut!((*pti).PostedMessagesListHead);
    while CurrentEntry != ListHead {
        PostedMessage = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as PUSER_MESSAGE;

        if (*PostedMessage).Msg.hwnd == UserHMGetHandle(Window) {
            if (*PostedMessage).Msg.message == WM_QUIT && (*pti).QuitPosted == 0 {
                (*pti).QuitPosted = 1;
                (*pti).exitCode = (*PostedMessage).Msg.wParam as _;
            }
            ClearMsgBitsMask(pti, (*PostedMessage).QS_Flags);
            MsqDestroyMessage(PostedMessage);
            CurrentEntry = (*pti).PostedMessagesListHead.Flink;
        } else {
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }

    CurrentEntry = (*pti).SentMessagesListHead.Flink;
    ListHead = core::ptr::addr_of_mut!((*pti).SentMessagesListHead);
    while CurrentEntry != ListHead {
        SentMessage = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry) as isize))
            as PUSER_SENT_MESSAGE;

        if (*SentMessage).Msg.hwnd == UserHMGetHandle(Window) {
            ERR!("Remove Window Messages %p From Sent Queue\n", SentMessage);
            CurrentEntry = (*CurrentEntry).Flink;
        } else {
            CurrentEntry = (*CurrentEntry).Flink;
        }
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqDispatchOneSentMessage(pti: PTHREADINFO) -> BOOLEAN {
    co_MsqDispatchOneSentMessage_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqDispatchOneSentMessage(pti: PTHREADINFO) -> BOOLEAN {
    co_MsqDispatchOneSentMessage_impl(pti)
}

unsafe fn co_MsqDispatchOneSentMessage_impl(pti: PTHREADINFO) -> BOOLEAN {
    let SaveMsg: PUSER_SENT_MESSAGE;
    let Message: PUSER_SENT_MESSAGE;
    let Entry: PLIST_ENTRY;
    let mut Ret: BOOL;
    let mut Result: LRESULT = 0;

    if IsListEmpty(core::ptr::addr_of_mut!((*pti).SentMessagesListHead)) != 0 {
        return 0u8;
    }

    Entry = RemoveHeadList(core::ptr::addr_of_mut!((*pti).SentMessagesListHead));
    Message = (Entry as *mut u8).sub(
        core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry),
    ) as PUSER_SENT_MESSAGE;

    (*Message).flags |= (SMF_RECEIVERBUSY | SMF_RECEIVEDMESSAGE) as u32;

    SaveMsg = (*pti).pusmCurrent;
    (*pti).pusmCurrent = Message;

    if (!(*Message).ptiSender.is_null() && pti != (*Message).ptiSender)
        || (!(*Message).ptiCallBackSender.is_null() && pti != (*Message).ptiCallBackSender)
    {
        (*(*pti).pcti).CTI_flags |= CTI_INSENDMESSAGE;
    }

    InsertTailList(
        core::ptr::addr_of_mut!(usmList),
        core::ptr::addr_of_mut!((*Message).ListEntry),
    );

    ClearMsgBitsMask(pti, (*Message).QS_Flags);

    'exit: {
        if (*Message).HookMessage == MSQ_ISHOOK as INT {
            Result = co_CallHook(
                (*Message).Msg.message as i32,
                HandleToLong((*Message).Msg.hwnd as *const core::ffi::c_void),
                (*Message).Msg.wParam,
                (*Message).Msg.lParam,
            );
        } else if (*Message).HookMessage == MSQ_INJECTMODULE as INT {
            Result = IntLoadHookModule(
                (*Message).Msg.message as i32,
                (*Message).Msg.lParam as HHOOK,
                (*Message).Msg.wParam as BOOL,
            ) as LRESULT;
        } else if (*Message).CompletionCallback.is_some()
            && (*Message).ptiCallBackSender == pti
        {
            if (*Message).QS_Flags & QS_SMRESULT != 0 {
                co_IntCallSentMessageCallback(
                    (*Message).CompletionCallback,
                    (*Message).Msg.hwnd,
                    (*Message).Msg.message,
                    (*Message).CompletionCallbackContext,
                    (*Message).lResult,
                );
                (*Message).CompletionCallback = None;
            } else {
                RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
                InsertTailList(
                    core::ptr::addr_of_mut!((*(*Message).ptiCallBackSender).SentMessagesListHead),
                    core::ptr::addr_of_mut!((*Message).ListEntry),
                );
                MsqWakeQueue((*Message).ptiCallBackSender, QS_SENDMESSAGE, 1);
                ERR!("Callback Message not processed yet. Requeuing the message\n");
                Ret = 0;
                break 'exit;
            }
        } else {
            Result = co_IntSendMessage(
                (*Message).Msg.hwnd,
                (*Message).Msg.message,
                (*Message).Msg.wParam,
                (*Message).Msg.lParam,
            );
        }

        if (*Message).CompletionCallback.is_some() {
            if !(*Message).ptiCallBackSender.is_null() {
                (*Message).lResult = Result;
                (*Message).QS_Flags |= QS_SMRESULT;

                RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
                InsertTailList(
                    core::ptr::addr_of_mut!((*(*Message).ptiCallBackSender).SentMessagesListHead),
                    core::ptr::addr_of_mut!((*Message).ListEntry),
                );
                MsqWakeQueue((*Message).ptiCallBackSender, QS_SENDMESSAGE, 1);
            }
            Ret = 1;
            break 'exit;
        }

        if (*Message).QS_Flags & QS_SMRESULT != 0 {
            Result = (*Message).lResult;
        }

        (*Message).lResult = Result;

        if (*Message).HasPackedLParam != 0 {
            if (*Message).Msg.lParam != 0 {
                ExFreePool((*Message).Msg.lParam as *mut core::ffi::c_void);
            }
        }

        (*Message).flags &= !(SMF_RECEIVERBUSY as u32);

        if !(*Message).pkCompletionEvent.is_null() {
            KeSetEvent((*Message).pkCompletionEvent as _, IO_NO_INCREMENT as KPRIORITY, 0u8);
        }

        if (*Message).flags & SMF_RECEIVERFREE as u32 != 0 {
            TRACE!(b"Receiver Freeing Message %p\n\0".as_ptr() as *const i8, Message);
            FreeUserMessage(Message);
        }

        Ret = 1;
    }

    if SaveMsg.is_null() {
        (*(*pti).pcti).CTI_flags &= !CTI_INSENDMESSAGE;
    }
    (*pti).pusmCurrent = SaveMsg;

    Ret as BOOLEAN
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqSendMessageAsync(
    ptiReceiver: PTHREADINFO,
    hwnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    CompletionCallback: SENDASYNCPROC,
    CompletionCallbackContext: ULONG_PTR,
    HasPackedLParam: BOOL,
    HookMessage: INT,
) -> BOOL {
    co_MsqSendMessageAsync_impl(
        ptiReceiver,
        hwnd,
        Msg,
        wParam,
        lParam,
        CompletionCallback,
        CompletionCallbackContext,
        HasPackedLParam,
        HookMessage,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqSendMessageAsync(
    ptiReceiver: PTHREADINFO,
    hwnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    CompletionCallback: SENDASYNCPROC,
    CompletionCallbackContext: ULONG_PTR,
    HasPackedLParam: BOOL,
    HookMessage: INT,
) -> BOOL {
    co_MsqSendMessageAsync_impl(
        ptiReceiver,
        hwnd,
        Msg,
        wParam,
        lParam,
        CompletionCallback,
        CompletionCallbackContext,
        HasPackedLParam,
        HookMessage,
    )
}

unsafe fn co_MsqSendMessageAsync_impl(
    ptiReceiver: PTHREADINFO,
    hwnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    CompletionCallback: SENDASYNCPROC,
    CompletionCallbackContext: ULONG_PTR,
    HasPackedLParam: BOOL,
    HookMessage: INT,
) -> BOOL {
    let ptiSender: PTHREADINFO;
    let Message: PUSER_SENT_MESSAGE = AllocateUserMessage(0);

    if Message.is_null() {
        ERR!("MsqSendMessageAsync(): Not enough memory to allocate a message\n");
        return 0;
    }

    ptiSender = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    (*Message).Msg.hwnd = hwnd;
    (*Message).Msg.message = Msg;
    (*Message).Msg.wParam = wParam;
    (*Message).Msg.lParam = lParam;
    (*Message).pkCompletionEvent = core::ptr::null_mut();
    (*Message).ptiReceiver = ptiReceiver;
    (*Message).ptiCallBackSender = ptiSender;
    (*Message).CompletionCallback = CompletionCallback;
    (*Message).CompletionCallbackContext = CompletionCallbackContext;
    (*Message).HookMessage = HookMessage;
    (*Message).HasPackedLParam = HasPackedLParam;
    (*Message).QS_Flags = QS_SENDMESSAGE as _;
    (*Message).flags = SMF_RECEIVERFREE as _;

    InsertTailList(
        core::ptr::addr_of_mut!((*ptiReceiver).SentMessagesListHead),
        core::ptr::addr_of_mut!((*Message).ListEntry),
    );
    MsqWakeQueue(ptiReceiver, QS_SENDMESSAGE as DWORD, 1);

    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqSendMessage(
    ptirec: PTHREADINFO,
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uTimeout: UINT,
    Block: BOOL,
    HookMessage: INT,
    uResult: *mut ULONG_PTR,
) -> NTSTATUS {
    co_MsqSendMessage_impl(ptirec, Wnd, Msg, wParam, lParam, uTimeout, Block, HookMessage, uResult)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqSendMessage(
    ptirec: PTHREADINFO,
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uTimeout: UINT,
    Block: BOOL,
    HookMessage: INT,
    uResult: *mut ULONG_PTR,
) -> NTSTATUS {
    co_MsqSendMessage_impl(ptirec, Wnd, Msg, wParam, lParam, uTimeout, Block, HookMessage, uResult)
}

unsafe fn co_MsqSendMessage_impl(
    ptirec: PTHREADINFO,
    Wnd: HWND,
    Msg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uTimeout: UINT,
    Block: BOOL,
    HookMessage: INT,
    uResult: *mut ULONG_PTR,
) -> NTSTATUS {
    let pti: PTHREADINFO;
    let SaveMsg: PUSER_SENT_MESSAGE;
    let Message: PUSER_SENT_MESSAGE;
    let mut WaitStatus: NTSTATUS;
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Entry: PLIST_ENTRY;
    let pWnd: PWND;
    let mut SwapStateEnabled: BOOLEAN = 0;
    let Result: LRESULT;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if (*pti).TIF_flags & TIF_INCLEANUP != 0 || (*ptirec).TIF_flags & TIF_INCLEANUP != 0 {
        if (*pti).TIF_flags & TIF_INCLEANUP != 0 && (*ptirec).TIF_flags & TIF_INCLEANUP == 0 {
            TRACE!(b"Send message from dying thread %u\n\0".as_ptr() as *const i8, Msg);
            co_MsqSendMessageAsync(ptirec, Wnd, Msg, wParam, lParam, None, 0, 0, HookMessage);
        }
        if !uResult.is_null() {
            *uResult = -1i64 as ULONG_PTR;
        }
        TRACE!(
            b"MsqSM: Msg %u Current pti %lu or Rec pti %lu\n\0".as_ptr() as *const i8,
            Msg,
            (*pti).TIF_flags & TIF_INCLEANUP,
            (*ptirec).TIF_flags & TIF_INCLEANUP,
        );
        return STATUS_UNSUCCESSFUL;
    }

    if IsThreadSuspended(ptirec) != 0 {
        ERR!("Sending to Suspended Thread Msg %lx\n", Msg);
        if !uResult.is_null() {
            *uResult = -1i64 as ULONG_PTR;
        }
        return STATUS_UNSUCCESSFUL;
    }

    if HookMessage == MSQ_NORMAL as INT {
        pWnd = ValidateHwndNoErr(Wnd);

        if (*pti).ppi != (*ptirec).ppi && !pWnd.is_null() {
            match Msg {
                x if x == EM_GETLINE || x == EM_SETPASSWORDCHAR || x == WM_GETTEXT => {
                    if (*gpsi).atomSysClass[ICLS_EDIT as usize] == (*(*pWnd).pcls).atomClassName
                        && (*pWnd).style & (ES_PASSWORD as u32) != 0
                    {
                        if !uResult.is_null() {
                            *uResult = -1i64 as ULONG_PTR;
                        }
                        ERR!("Running across the border without a passport!\n");
                        EngSetLastError(ERROR_ACCESS_DENIED);
                        return STATUS_UNSUCCESSFUL;
                    }
                }
                x if x == WM_NOTIFY => {
                    if !uResult.is_null() {
                        *uResult = -1i64 as ULONG_PTR;
                    }
                    ERR!("Running across the border without a passport!\n");
                    return STATUS_UNSUCCESSFUL;
                }
                _ => {}
            }
        }

        if Msg == WM_CREATE || Msg == WM_NCCREATE {
            if !uResult.is_null() {
                *uResult = -1i64 as ULONG_PTR;
            }
            ERR!("Can not tell the other State we have Create!\n");
            return STATUS_UNSUCCESSFUL;
        }
    }

    Message = AllocateUserMessage(1);
    if Message.is_null() {
        ERR!("MsqSendMessage(): Not enough memory to allocate a message\n");
        if !uResult.is_null() {
            *uResult = -1i64 as ULONG_PTR;
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Timeout.QuadPart = Int32x32To64(-10000, uTimeout as LONG);
    TRACE!(b"Timeout val %lld\n\0".as_ptr() as *const i8, Timeout.QuadPart);

    (*Message).Msg.hwnd = Wnd;
    (*Message).Msg.message = Msg;
    (*Message).Msg.wParam = wParam;
    (*Message).Msg.lParam = lParam;
    (*Message).ptiReceiver = ptirec;
    (*Message).ptiSender = pti;
    (*Message).HookMessage = HookMessage;
    (*Message).QS_Flags = QS_SENDMESSAGE;

    SaveMsg = (*pti).pusmSent;
    (*pti).pusmSent = Message;

    InsertTailList(
        core::ptr::addr_of_mut!((*ptirec).SentMessagesListHead),
        core::ptr::addr_of_mut!((*Message).ListEntry),
    );

    MsqWakeQueue(ptirec, QS_SENDMESSAGE, 1);

    if (*pti).cEnterCount == 0 {
        SwapStateEnabled = KeSetKernelStackSwapEnable(0);
    }
    (*pti).cEnterCount += 1;

    if Block != 0 {
        let mut WaitObjects: [PVOID; 2] = [core::ptr::null_mut(); 2];

        WaitObjects[0] = (*Message).pkCompletionEvent as PVOID;
        WaitObjects[1] = pti_ethread(ptirec) as PVOID;

        UserLeaveCo();

        WaitStatus = KeWaitForMultipleObjects(
            2,
            WaitObjects.as_mut_ptr(),
            WaitAny,
            _KWAIT_REASON_UserRequest,
            1i32 as i8,
            0u8,
            if uTimeout != 0 {
                core::ptr::addr_of_mut!(Timeout) as _
            } else {
                core::ptr::null_mut()
            },
            core::ptr::null_mut(),
        );

        UserEnterCo();

        if WaitStatus == STATUS_TIMEOUT {
            Entry = (*ptirec).SentMessagesListHead.Flink;
            while Entry != core::ptr::addr_of_mut!((*ptirec).SentMessagesListHead) {
                if (Entry as *mut u8).offset(
                    -(core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry) as isize),
                ) as PUSER_SENT_MESSAGE
                    == Message
                {
                    (*Message).pkCompletionEvent = core::ptr::null_mut();
                    RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
                    ClearMsgBitsMask(ptirec, (*Message).QS_Flags);
                    InsertTailList(
                        core::ptr::addr_of_mut!(usmList),
                        core::ptr::addr_of_mut!((*Message).ListEntry),
                    );
                    break;
                }
                Entry = (*Entry).Flink;
            }

            ERR!("MsqSendMessage (blocked) timed out 1 Status %lx\n", WaitStatus);
        } else if WaitStatus == STATUS_WAIT_1 {
            ERR!("Bk Receiving Thread woken up dead!\n");
            (*Message).flags |= SMF_RECEIVERDIED;
        }

        while co_MsqDispatchOneSentMessage(pti) != 0 {}
    } else {
        let mut WaitObjects: [PVOID; 3] = [core::ptr::null_mut(); 3];

        WaitObjects[0] = (*Message).pkCompletionEvent as PVOID;
        WaitObjects[1] = (*pti).pEventQueueServer as PVOID;
        WaitObjects[2] = pti_ethread(ptirec) as PVOID;

        loop {
            UserLeaveCo();

            WaitStatus = KeWaitForMultipleObjects(
                3,
                WaitObjects.as_mut_ptr(),
                WaitAny,
                _KWAIT_REASON_UserRequest,
                1i32 as i8,
                0u8,
                if uTimeout != 0 {
                    core::ptr::addr_of_mut!(Timeout) as _
                } else {
                    core::ptr::null_mut()
                },
                core::ptr::null_mut(),
            );

            UserEnterCo();

            if WaitStatus == STATUS_TIMEOUT {
                Entry = (*ptirec).SentMessagesListHead.Flink;
                while Entry != core::ptr::addr_of_mut!((*ptirec).SentMessagesListHead) {
                    if (Entry as *mut u8).offset(
                        -(core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry) as isize),
                    ) as PUSER_SENT_MESSAGE
                        == Message
                    {
                        (*Message).pkCompletionEvent = core::ptr::null_mut();
                        RemoveEntryList(core::ptr::addr_of_mut!((*Message).ListEntry));
                        ClearMsgBitsMask(ptirec, (*Message).QS_Flags);
                        InsertTailList(
                            core::ptr::addr_of_mut!(usmList),
                            core::ptr::addr_of_mut!((*Message).ListEntry),
                        );
                        break;
                    }
                    Entry = (*Entry).Flink;
                }

                WARN!(b"MsqSendMessage timed out 2 Status %lx\n\0".as_ptr() as *const i8, WaitStatus);
                break;
            } else if WaitStatus == STATUS_WAIT_2 {
                ERR!("NB Receiving Thread woken up dead!\n");
                (*Message).flags |= SMF_RECEIVERDIED;
                break;
            }

            if WaitStatus == STATUS_USER_APC {
                break;
            }

            while co_MsqDispatchOneSentMessage(pti) != 0 {}

            if WaitStatus != STATUS_WAIT_1 {
                break;
            }
        }
    }

    (*pti).cEnterCount -= 1;
    if (*pti).cEnterCount == 0 {
        KeSetKernelStackSwapEnable(SwapStateEnabled);
    }

    if WaitStatus == STATUS_USER_APC {
        TRACE!(b"User APC\n\0".as_ptr() as *const i8);

        (*Message).flags |= SMF_SENDERDIED;

        co_IntDeliverUserAPC();
        ERR!("User APC Returned\n");
    }

    KeSetEvent((*pti).pEventQueueServer as _, IO_NO_INCREMENT as KPRIORITY, 0u8);

    Result = (*Message).lResult;

    if ((*Message).flags & (SMF_RECEIVERBUSY | SMF_RECEIVEDMESSAGE)) != SMF_RECEIVEDMESSAGE {
        (*Message).flags |= SMF_RECEIVERFREE;
    }

    if (*Message).flags & SMF_RECEIVERFREE == 0 {
        TRACE!(
            b"Sender Freeing Message %p ptirec %p bit %d list empty %d\n\0".as_ptr() as *const i8,
            Message,
            ptirec,
            ((*(*ptirec).pcti).fsChangeBits & QS_SENDMESSAGE as u32 != 0) as INT,
            IsListEmpty(core::ptr::addr_of_mut!((*ptirec).SentMessagesListHead)),
        );
        FreeUserMessage(Message);
    }

    (*pti).pusmSent = SaveMsg;

    TRACE!(
        b"MSM Allocation Count %d Status %lx Result %d\n\0".as_ptr() as *const i8,
        SendMsgCount,
        WaitStatus,
        Result,
    );

    if WaitStatus != STATUS_TIMEOUT {
        if !uResult.is_null() {
            *uResult = if STATUS_WAIT_0 == WaitStatus {
                Result as ULONG_PTR
            } else {
                0
            };
        }
    }

    WaitStatus
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqPostMessage(
    pti: PTHREADINFO,
    Msg: *mut MSG,
    HardwareMessage: BOOLEAN,
    MessageBits: DWORD,
    dwQEvent: DWORD,
    ExtraInfo: LONG_PTR,
) {
    MsqPostMessage_impl(pti, Msg, HardwareMessage, MessageBits, dwQEvent, ExtraInfo)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqPostMessage(
    pti: PTHREADINFO,
    Msg: *mut MSG,
    HardwareMessage: BOOLEAN,
    MessageBits: DWORD,
    dwQEvent: DWORD,
    ExtraInfo: LONG_PTR,
) {
    MsqPostMessage_impl(pti, Msg, HardwareMessage, MessageBits, dwQEvent, ExtraInfo)
}

unsafe fn MsqPostMessage_impl(
    pti: PTHREADINFO,
    Msg: *mut MSG,
    HardwareMessage: BOOLEAN,
    mut MessageBits: DWORD,
    dwQEvent: DWORD,
    ExtraInfo: LONG_PTR,
) {
    let Message: PUSER_MESSAGE;
    let MessageQueue: PUSER_MESSAGE_QUEUE;

    MessageQueue = (*pti).MessageQueue;

    if ((*pti).TIF_flags & TIF_INCLEANUP as FLONG != 0)
        || ((*MessageQueue).QF_flags & QF_INDESTROY != 0)
    {
        ERR!("Post Msg; Thread or Q is Dead!\n");
        return;
    }

    Message = MsqCreateMessage(Msg);
    if Message.is_null() {
        return;
    }

    if (*Msg).message == WM_HOTKEY {
        MessageBits |= QS_HOTKEY;
    }

    (*Message).dwQEvent = dwQEvent;
    (*Message).ExtraInfo = ExtraInfo;
    (*Message).QS_Flags = MessageBits;
    (*Message).pti = pti;

    if HardwareMessage == 0 {
        InsertTailList(
            core::ptr::addr_of_mut!((*pti).PostedMessagesListHead),
            core::ptr::addr_of_mut!((*Message).ListEntry),
        );
    } else {
        InsertTailList(
            core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead),
            core::ptr::addr_of_mut!((*Message).ListEntry),
        );
    }

    MsqWakeQueue(pti, MessageBits, 1 as BOOL);
    TRACE!(b"Post Message %d\n\0".as_ptr() as *const i8, PostMsgCount);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG) {
    MsqPostQuitMessage_impl(pti, ExitCode)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG) {
    MsqPostQuitMessage_impl(pti, ExitCode)
}

unsafe fn MsqPostQuitMessage_impl(pti: PTHREADINFO, ExitCode: ULONG) {
    (*pti).QuitPosted = 1;
    (*pti).exitCode = ExitCode as INT;
    MsqWakeQueue(pti, (QS_POSTMESSAGE | QS_ALLPOSTMESSAGE) as DWORD, 1 as BOOL);
}

unsafe fn MsqSendParentNotify(mut pwnd: PWND, event: WORD, idChild: WORD, mut pt: POINT) {
    let pwndDesktop: PWND = UserGetDesktopWindow();

    /* pt has to be in the client coordinates of the parent window */
    pt.x += (*pwndDesktop).rcClient.left - (*pwnd).rcClient.left;
    pt.y += (*pwndDesktop).rcClient.top - (*pwnd).rcClient.top;

    loop {
        let pwndParent: PWND;

        if ((*pwnd).style & WS_CHILD) == 0 { break; }
        if ((*pwnd).ExStyle & WS_EX_NOPARENTNOTIFY) != 0 { break; }
        pwndParent = IntGetParent(pwnd);
        if pwndParent.is_null() { break; }
        if pwndParent == pwndDesktop { break; }
        pt.x += (*pwnd).rcClient.left - (*pwndParent).rcClient.left;
        pt.y += (*pwnd).rcClient.top - (*pwndParent).rcClient.top;

        pwnd = pwndParent;
        co_IntSendMessage(UserHMGetHandle(pwnd), WM_PARENTNOTIFY,
                          MAKEWPARAM!(event, idChild), MAKELPARAM!(pt.x, pt.y));
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntTrackMouseMove(
    pwndTrack: PWND,
    pDesk: PDESKTOP,
    msg: PMSG,
    hittest: USHORT,
) {
    IntTrackMouseMoveImpl(pwndTrack, pDesk, msg, hittest)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntTrackMouseMove(
    pwndTrack: PWND,
    pDesk: PDESKTOP,
    msg: PMSG,
    hittest: USHORT,
) {
    IntTrackMouseMoveImpl(pwndTrack, pDesk, msg, hittest)
}

#[inline(always)]
unsafe fn IntTrackMouseMoveImpl(
    pwndTrack: PWND,
    pDesk: PDESKTOP,
    msg: PMSG,
    hittest: USHORT,
) {

    if (*pDesk).spwndTrack != pwndTrack
        || (*msg).message != WM_MOUSEMOVE
        || (*pDesk).htEx != hittest as DWORD
    {
        if (*pDesk).spwndTrack != pwndTrack
            || (((*pDesk).htEx == HTCLIENT as DWORD) ^ (hittest as DWORD == HTCLIENT as DWORD))
        {
            if (*pDesk).dwDTFlags & DF_TME_LEAVE != 0 {
                UserPostMessage(
                    UserHMGetHandle((*pDesk).spwndTrack),
                    if (*pDesk).htEx != HTCLIENT as DWORD {
                        WM_NCMOUSELEAVE
                    } else {
                        WM_MOUSELEAVE
                    },
                    0,
                    0,
                );
            }

            if (*pDesk).dwDTFlags & DF_TME_HOVER != 0 {
                IntKillTimer((*pDesk).spwndTrack, ID_EVENT_SYSTIMER_MOUSEHOVER as UINT_PTR, 1);
            }

            (*pDesk).dwDTFlags &= !(DF_TME_LEAVE | DF_TME_HOVER);
        }

        (*pDesk).spwndTrack = pwndTrack;
        (*pDesk).htEx = hittest as DWORD;
    }

    if (*pDesk).spwndTrack == pwndTrack
        && ((*msg).message != WM_MOUSEMOVE
            || RECTL_bPointInRect(
                core::ptr::addr_of!((*pDesk).rcMouseHover),
                (*msg).pt.x,
                (*msg).pt.y,
            ) == 0)
        && (*pDesk).dwDTFlags & DF_TME_HOVER != 0
    {
        IntSetTimer(
            (*pDesk).spwndTrack,
            ID_EVENT_SYSTIMER_MOUSEHOVER as UINT_PTR,
            (*pDesk).dwMouseHoverTime,
            Some(SystemTimerProc),
            TMRF_SYSTEM as INT,
        );
        RECTL_vSetRect(
            core::ptr::addr_of_mut!((*pDesk).rcMouseHover),
            (*msg).pt.x - gspv.iMouseHoverWidth / 2,
            (*msg).pt.y - gspv.iMouseHoverHeight / 2,
            (*msg).pt.x + gspv.iMouseHoverWidth / 2,
            (*msg).pt.y + gspv.iMouseHoverHeight / 2,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn co_IntProcessMouseMessage(
    msg: *mut MSG,
    RemoveMessages: *mut BOOL,
    NotForUs: *mut BOOL,
    ExtraInfo: LONG_PTR,
    first: UINT,
    last: UINT,
) -> BOOL {
    let mut clk_msg: MSG;
    let mut pt: POINT;
    let mut message: UINT;
    let hittest: USHORT;
    let mut event: EVENTMSG = core::mem::zeroed();
    let mut hook: MOUSEHOOKSTRUCT = core::mem::zeroed();
    let mut eatMsg: BOOL = FALSE as BOOL;

    let mut pwndMsg: PWND;
    let pwndDesktop: PWND;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let pti: PTHREADINFO;
    let CurInfo: PSYSTEM_CURSORINFO;
    let pDesk: PDESKTOP;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    pwndDesktop = UserGetDesktopWindow();
    if pwndDesktop.is_null() {
        ERR!("No desktop window!\n");
        return FALSE as BOOL;
    }

    MessageQueue = (*pti).MessageQueue;
    CurInfo = IntGetSysCursorInfo();
    pwndMsg = ValidateHwndNoErr((*msg).hwnd);
    clk_msg = (*MessageQueue).msgDblClk;
    pDesk = (*pwndDesktop).head.rpdesk;

    let mut hittest_local: USHORT;

    if !(*MessageQueue).spwndCapture.is_null() {
        hittest_local = HTCLIENT as USHORT;
        pwndMsg = (*MessageQueue).spwndCapture;
    } else {
        hittest_local = 0;
        pwndMsg = co_WinPosWindowFromPoint(
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*msg).pt),
            &mut hittest_local,
            FALSE as BOOL,
        );
    }
    hittest = hittest_local;

    TRACE!(
        b"Got mouse message for %p, hittest: 0x%x\n\0".as_ptr(),
        (*msg).hwnd,
        hittest as UINT,
    );

    if pwndMsg.is_null() || (*head_pti(pwndMsg)).MessageQueue != MessageQueue {
        IntSystemSetCursor(SYSTEMCUR(ROCR_ARROW));
        core::ptr::write_volatile(RemoveMessages, TRUE as BOOL);
        return FALSE as BOOL;
    }

    if head_pti(pwndMsg) != pti && (*MessageQueue).cThreads > 1 {
        core::ptr::write_volatile(NotForUs, TRUE as BOOL);
        core::ptr::write_volatile(RemoveMessages, FALSE as BOOL);
        return FALSE as BOOL;
    }

    if MessageQueue == gpqCursor {
        IntTrackMouseMove(pwndMsg, pDesk, msg, hittest);
    } else {
        WARN!(b"Not the same cursor!\n\0".as_ptr());
    }

    (*msg).hwnd = UserHMGetHandle(pwndMsg);

    pt = core::mem::transmute::<_POINTL, tagPOINT>((*msg).pt);
    message = (*msg).message;

    if message != WM_MOUSEWHEEL {
        if hittest != HTCLIENT as USHORT {
            message = message.wrapping_add((WM_NCMOUSEMOVE as i32 - WM_MOUSEMOVE as i32) as u32);
            (*msg).wParam = hittest as WPARAM;
        } else {
            if (*MessageQueue).MenuOwner.is_null() {
                pt.x += (*pwndDesktop).rcClient.left - (*pwndMsg).rcClient.left;
                pt.y += (*pwndDesktop).rcClient.top - (*pwndMsg).rcClient.top;
            }
        }
    }
    (*msg).lParam = MAKELONG!(pt.x as WORD, pt.y as WORD) as LPARAM;

    if (*msg).message == WM_LBUTTONDOWN
        || (*msg).message == WM_RBUTTONDOWN
        || (*msg).message == WM_MBUTTONDOWN
        || (*msg).message == WM_XBUTTONDOWN
    {
        let mut update: BOOL = *RemoveMessages;

        if !(*MessageQueue).MenuOwner.is_null()
            || !(*MessageQueue).MoveSize.is_null()
            || hittest != HTCLIENT as USHORT
            || ((*(*pwndMsg).pcls).style & CS_DBLCLKS) != 0
        {
            if (*msg).message == clk_msg.message
                && (*msg).hwnd == clk_msg.hwnd
                && ((*msg).message != WM_XBUTTONDOWN
                    || GET_XBUTTON_WPARAM((*msg).wParam) == GET_XBUTTON_WPARAM(clk_msg.wParam))
                && ((*msg).time.wrapping_sub(clk_msg.time)) < gspv.iDblClickTime as ULONG
                && abs((*msg).pt.x - clk_msg.pt.x) < UserGetSystemMetrics(SM_CXDOUBLECLK) / 2
                && abs((*msg).pt.y - clk_msg.pt.y) < UserGetSystemMetrics(SM_CYDOUBLECLK) / 2
            {
                message = message.wrapping_add(WM_LBUTTONDBLCLK - WM_LBUTTONDOWN);
                if update != 0 {
                    (*MessageQueue).msgDblClk.message = 0;
                    update = FALSE as BOOL;
                }
            }
        }

        if !((first == 0 && last == 0) || (message >= first || message <= last)) {
            TRACE!(b"Message out of range!!!\n\0".as_ptr());
            return FALSE as BOOL;
        }

        if update != 0 {
            (*MessageQueue).msgDblClk = *msg;
        }
    } else {
        if !((first == 0 && last == 0) || (message >= first || message <= last)) {
            TRACE!(b"Message out of range!!!\n\0".as_ptr());
            return FALSE as BOOL;
        }

        if message == WM_MOUSEMOVE {
            (*msg).wParam = MsqGetDownKeyState(MessageQueue);
        }
    }

    if gspv.bMouseClickLock != 0 {
        let mut IsClkLck: BOOL = FALSE as BOOL;

        if (*msg).message == WM_LBUTTONUP {
            IsClkLck = (((*msg).time.wrapping_sub((*CurInfo).ClickLockTime))
                >= gspv.dwMouseClickLockTime) as BOOL;
            if IsClkLck != 0 && (*CurInfo).ClickLockActive == 0 {
                (*CurInfo).ClickLockActive = TRUE as BOOL;
            }
        } else if (*msg).message == WM_LBUTTONDOWN {
            if (*CurInfo).ClickLockActive != 0 {
                IsClkLck = TRUE as BOOL;
                (*CurInfo).ClickLockActive = FALSE as BOOL;
            }

            (*CurInfo).ClickLockTime = (*msg).time;
        }

        if IsClkLck != 0 {
            core::ptr::write_volatile(RemoveMessages, TRUE as BOOL);
            TRACE!(b"Remove and ignore the message\n\0".as_ptr());
            return FALSE as BOOL;
        }
    }

    if (*pti).TIF_flags & TIF_MSGPOSCHANGED != 0 {
        (*pti).TIF_flags &= !TIF_MSGPOSCHANGED;
        IntNotifyWinEvent(
            EVENT_OBJECT_LOCATIONCHANGE,
            core::ptr::null_mut(),
            OBJID_CLIENT as LONG,
            CHILDID_SELF as LONG,
            0,
        );
    }

    event.message = (*msg).message;
    event.time = (*msg).time;
    event.hwnd = (*msg).hwnd;
    event.paramL = (*msg).pt.x as UINT;
    event.paramH = (*msg).pt.y as UINT;
    co_HOOK_CallHooks(
        WH_JOURNALRECORD as INT,
        HC_ACTION as INT,
        0,
        core::ptr::addr_of_mut!(event) as LPARAM,
    );

    hook.pt = (*msg).pt;
    hook.hwnd = (*msg).hwnd;
    hook.wHitTestCode = hittest as UINT;
    hook.dwExtraInfo = ExtraInfo as ULONG_PTR;
    if co_HOOK_CallHooks(
        WH_MOUSE as INT,
        if *RemoveMessages != 0 { HC_ACTION as INT } else { HC_NOREMOVE as INT },
        message as WPARAM,
        core::ptr::addr_of_mut!(hook) as LPARAM,
    ) != 0
    {
        hook.pt = (*msg).pt;
        hook.hwnd = (*msg).hwnd;
        hook.wHitTestCode = hittest as UINT;
        hook.dwExtraInfo = ExtraInfo as ULONG_PTR;
        co_HOOK_CallHooks(
            WH_CBT as INT,
            HCBT_CLICKSKIPPED as INT,
            message as WPARAM,
            core::ptr::addr_of_mut!(hook) as LPARAM,
        );

        ERR!("WH_MOUSE dropped mouse message!\n");

        core::ptr::write_volatile(RemoveMessages, TRUE as BOOL);
        return FALSE as BOOL;
    }

    if hittest == HTERROR as USHORT || hittest == HTNOWHERE as USHORT {
        co_IntSendMessage(
            (*msg).hwnd,
            WM_SETCURSOR,
            (*msg).hwnd as WPARAM,
            MAKELONG!(hittest as WORD, (*msg).message as WORD) as LPARAM,
        );

        core::ptr::write_volatile(RemoveMessages, TRUE as BOOL);
        return FALSE as BOOL;
    }

    if *RemoveMessages == FALSE as BOOL || !(*MessageQueue).spwndCapture.is_null() {
        (*msg).message = message;
        return TRUE as BOOL;
    }

    if (*msg).message == WM_LBUTTONDOWN
        || (*msg).message == WM_RBUTTONDOWN
        || (*msg).message == WM_MBUTTONDOWN
        || (*msg).message == WM_XBUTTONDOWN
    {
        MsqSendParentNotify(pwndMsg, (*msg).message as WORD, 0, core::mem::transmute::<_POINTL, tagPOINT>((*msg).pt));

        if pwndMsg != (*MessageQueue).spwndActive {
            let mut pwndTop: PWND = pwndMsg;
            pwndTop = IntGetNonChildAncestor(pwndTop);

            TRACE!(
                b"Mouse pti %p pwndMsg pti %p pwndTop pti %p\n\0".as_ptr(),
                (*MessageQueue).ptiMouse,
                head_pti(pwndMsg),
                head_pti(pwndTop),
            );

            if !pwndTop.is_null() && pwndTop != pwndDesktop {
                let ret: LONG = co_IntSendMessage(
                    (*msg).hwnd,
                    WM_MOUSEACTIVATE,
                    UserHMGetHandle(pwndTop) as WPARAM,
                    MAKELONG!(hittest as WORD, (*msg).message as WORD) as LPARAM,
                ) as LONG;
                match ret as UINT {
                    MA_NOACTIVATEANDEAT => {
                        eatMsg = TRUE as BOOL;
                        /* fall through */
                    }
                    MA_NOACTIVATE => {}
                    MA_ACTIVATEANDEAT => {
                        eatMsg = TRUE as BOOL;
                        if !co_IntMouseActivateWindow(pwndTop) != 0 {
                            eatMsg = TRUE as BOOL;
                        }
                    }
                    MA_ACTIVATE | 0 => {
                        if co_IntMouseActivateWindow(pwndTop) == 0 {
                            eatMsg = TRUE as BOOL;
                        }
                    }
                    _ => {
                        ERR!("unknown WM_MOUSEACTIVATE code %d\n", ret);
                    }
                }
                if ret as UINT == MA_NOACTIVATEANDEAT || ret as UINT == MA_NOACTIVATE {
                    // already handled
                }
            }
        }
    }

    co_IntSendMessage(
        (*msg).hwnd,
        WM_SETCURSOR,
        (*msg).hwnd as WPARAM,
        MAKELONG!(hittest as WORD, (*msg).message as WORD) as LPARAM,
    );

    (*msg).message = message;
    (!(eatMsg != 0)) as BOOL
}

unsafe extern "C" fn co_IntProcessKeyboardMessage(Msg: *mut MSG, RemoveMessages: *mut BOOL) -> BOOL {
    let mut Event: EVENTMSG = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pWnd: PWND;
    let mut ImmRet: UINT;
    let mut Ret: BOOL = TRUE as BOOL;
    let mut bKeyUpDown: BOOL = FALSE as BOOL;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let uMsg: UINT = (*Msg).message;

    if uMsg == VK_PACKET as UINT {
        (*pti).wchInjected = HIWORD((*Msg).wParam as DWORD) as WCHAR;
    }

    if uMsg == WM_KEYDOWN || uMsg == WM_SYSKEYDOWN || uMsg == WM_KEYUP || uMsg == WM_SYSKEYUP {
        bKeyUpDown = TRUE as BOOL;
        match (*Msg).wParam as i32 {
            x if x == VK_LSHIFT as i32 || x == VK_RSHIFT as i32 => {
                (*Msg).wParam = VK_SHIFT as WPARAM;
            }
            x if x == VK_LCONTROL as i32 || x == VK_RCONTROL as i32 => {
                (*Msg).wParam = VK_CONTROL as WPARAM;
            }
            x if x == VK_LMENU as i32 || x == VK_RMENU as i32 => {
                (*Msg).wParam = VK_MENU as WPARAM;
            }
            _ => {}
        }
    }

    pWnd = ValidateHwndNoErr((*Msg).hwnd);
    if !pWnd.is_null() {
        UserRefObjectCo(pWnd, &mut Ref);
    }

    Event.message = uMsg;
    Event.hwnd = (*Msg).hwnd;
    Event.time = (*Msg).time;
    Event.paramL = ((*Msg).wParam & 0xFF) as DWORD | ((HIWORD((*Msg).lParam as DWORD) as DWORD) << 8);
    Event.paramH = ((*Msg).lParam & 0x7FFF) as DWORD;
    if (HIWORD((*Msg).lParam as DWORD) & 0x0100) != 0 {
        Event.paramH |= 0x8000;
    }
    co_HOOK_CallHooks(WH_JOURNALRECORD as INT, HC_ACTION as INT, 0, &mut Event as *mut _ as LPARAM);

    'exit: {
        if *RemoveMessages != 0 {
            if uMsg == WM_KEYDOWN && (*Msg).hwnd != IntGetDesktopWindow() {
                if (*Msg).wParam == VK_F1 as WPARAM {
                    UserPostMessage((*Msg).hwnd, WM_KEYF1, 0, 0);
                } else if (*Msg).wParam >= VK_BROWSER_BACK as WPARAM
                    && (*Msg).wParam <= VK_LAUNCH_APP2 as WPARAM
                {
                    co_IntSendMessage(
                        (*Msg).hwnd,
                        WM_APPCOMMAND,
                        (*Msg).hwnd as WPARAM,
                        MAKELPARAM!(0, (FAPPCOMMAND_KEY | ((*Msg).wParam as u32 - VK_BROWSER_BACK as u32 + 1))),
                    );
                }
            } else if uMsg == WM_KEYUP {
                if (*Msg).wParam == VK_APPS as WPARAM && (*(*pti).MessageQueue).MenuOwner.is_null() {
                    UserPostMessage((*Msg).hwnd, WM_CONTEXTMENU, (*Msg).hwnd as WPARAM, -1i32 as LPARAM);
                }
            }
        }

        if *RemoveMessages != 0 && uMsg == WM_SYSKEYDOWN {
            if (HIWORD((*Msg).lParam as DWORD) & KF_ALTDOWN) != 0 {
                if (*Msg).wParam == VK_ESCAPE as WPARAM || (*Msg).wParam == VK_TAB as WPARAM {
                    let wParamTmp: WPARAM = if UserGetKeyState(VK_SHIFT as DWORD) & 0x8000 != 0 {
                        SC_PREVWINDOW as WPARAM
                    } else {
                        SC_NEXTWINDOW as WPARAM
                    };
                    TRACE!(b"Send WM_SYSCOMMAND Alt-Tab/ESC Alt-Shift-Tab/ESC\n\0".as_ptr());
                    co_IntSendMessage((*Msg).hwnd, WM_SYSCOMMAND, wParamTmp, (*Msg).wParam as LPARAM);

                    Ret = FALSE as BOOL;
                    break 'exit;
                }
            }
        }

        if co_HOOK_CallHooks(
            WH_KEYBOARD as INT,
            if *RemoveMessages != 0 { HC_ACTION as i32 } else { HC_NOREMOVE as i32 },
            LOWORD((*Msg).wParam as DWORD) as WPARAM,
            (*Msg).lParam,
        ) != 0
        {
            co_HOOK_CallHooks(
                WH_CBT as INT,
                HCBT_KEYSKIPPED as i32,
                LOWORD((*Msg).wParam as DWORD) as WPARAM,
                (*Msg).lParam,
            );

            ERR!("KeyboardMessage WH_KEYBOARD Call Hook return!\n");

            *RemoveMessages = TRUE as BOOL;

            Ret = FALSE as BOOL;
        }

        if !pWnd.is_null()
            && Ret != 0
            && *RemoveMessages != 0
            && bKeyUpDown != 0
            && ((*pti).TIF_flags & TIF_DISABLEIME) == 0
        {
            ImmRet = IntImmProcessKey((*pti).MessageQueue, pWnd, uMsg, (*Msg).wParam, (*Msg).lParam);
            if ImmRet != 0 {
                if (ImmRet & (IPHK_HOTKEY | IPHK_SKIPTHISKEY)) != 0 {
                    ImmRet = 0;
                }
                if (ImmRet & IPHK_PROCESSBYIME) != 0 {
                    (*Msg).wParam = VK_PROCESSKEY as WPARAM;
                }
            }
        }
    }

    if !pWnd.is_null() {
        UserDerefObjectCo(pWnd);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn co_IntProcessHardwareMessage(
    Msg: *mut MSG,
    RemoveMessages: *mut BOOL,
    NotForUs: *mut BOOL,
    ExtraInfo: LONG_PTR,
    first: UINT,
    last: UINT,
) -> BOOL {
    if IS_MOUSE_MESSAGE((*Msg).message) != 0 {
        return co_IntProcessMouseMessage(Msg, RemoveMessages, NotForUs, ExtraInfo, first, last);
    } else if IS_KBD_MESSAGE((*Msg).message) != 0 {
        return co_IntProcessKeyboardMessage(Msg, RemoveMessages);
    }

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn filter_contains_hw_range(first: UINT, mut last: UINT) -> INT {
    if last == 0 {
        last = last.wrapping_sub(1);
    }
    if last < WM_NCMOUSEFIRST {
        return 0;
    }
    if first > WM_NCMOUSELAST && last < WM_KEYFIRST {
        return 0;
    }
    if first > WM_KEYLAST && last < WM_MOUSEFIRST {
        return 0;
    }
    if first > WM_MOUSELAST {
        return 0;
    }
    1
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn filter_contains_hw_range(first: UINT, mut last: UINT) -> INT {
    if last == 0 {
        last = last.wrapping_sub(1);
    }
    if last < WM_NCMOUSEFIRST {
        return 0;
    }
    if first > WM_NCMOUSELAST && last < WM_KEYFIRST {
        return 0;
    }
    if first > WM_KEYLAST && last < WM_MOUSEFIRST {
        return 0;
    }
    if first > WM_MOUSELAST {
        return 0;
    }
    1
}

unsafe fn is_mouse_message(message: UINT) -> BOOL {
    ((message >= WM_MOUSEFIRST && message <= WM_MOUSELAST)
        || (message >= WM_XBUTTONDOWN && message <= WM_XBUTTONDBLCLK)
        || (message >= WM_MBUTTONDOWN && message <= WM_MBUTTONDBLCLK)
        || (message >= WM_LBUTTONDOWN && message <= WM_RBUTTONDBLCLK)) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn co_MsqPeekHardwareMessage(
    pti: PTHREADINFO,
    mut Remove: BOOL,
    Window: PWND,
    MsgFilterLow: UINT,
    MsgFilterHigh: UINT,
    QSflags: UINT,
    pMsg: *mut MSG,
) -> BOOL {
    let mut AcceptMessage: BOOL;
    let mut NotForUs: BOOL;
    let mut CurrentMessage: PUSER_MESSAGE;
    let mut ListHead: PLIST_ENTRY;
    let mut msg: MSG = core::mem::zeroed();
    let mut idSave: ULONG_PTR;
    let mut QS_Flags: DWORD;
    let mut ExtraInfo: LONG_PTR;
    let mut clk_msg: MSG;
    let mut Ret: BOOL = 0;
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    if filter_contains_hw_range(MsgFilterLow, MsgFilterHigh) == 0 {
        return 0;
    }

    ListHead = (*MessageQueue).HardwareMessagesListHead.Flink;

    if IsListEmpty(ListHead) != 0 {
        return 0;
    }

    if (*MessageQueue).ptiSysLock.is_null() {
        (*MessageQueue).ptiSysLock = pti;
        (*(*pti).pcti).CTI_flags |= CTI_THREADSYSLOCK;
    }

    if (*MessageQueue).ptiSysLock != pti {
        ERR!(
            "Thread Q is locked to ptiSysLock 0x%p pti 0x%p\n",
            (*MessageQueue).ptiSysLock,
            pti,
        );
        return 0;
    }

    while ListHead != core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead) {
        CurrentMessage = (ListHead as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as PUSER_MESSAGE;
        ListHead = (*ListHead).Flink;

        if (*MessageQueue).idSysPeek == CurrentMessage as ULONG_PTR {
            TRACE!(b"Skip this message due to it is in play!\n\0".as_ptr() as *const i8);
            continue;
        }

        if (Window.is_null()
            || (Window == PWND_BOTTOM as PWND && (*CurrentMessage).Msg.hwnd.is_null())
            || (Window != PWND_BOTTOM as PWND
                && UserHMGetHandle(Window) == (*CurrentMessage).Msg.hwnd)
            || (is_mouse_message((*CurrentMessage).Msg.message) != 0))
            && ((*CurrentMessage).QS_Flags & QSflags) != 0
        {
            idSave = (*MessageQueue).idSysPeek;
            (*MessageQueue).idSysPeek = CurrentMessage as ULONG_PTR;

            msg = (*CurrentMessage).Msg;
            ExtraInfo = (*CurrentMessage).ExtraInfo;
            QS_Flags = (*CurrentMessage).QS_Flags;
            clk_msg = (*MessageQueue).msgDblClk;

            NotForUs = 0;

            UpdateKeyStateFromMsg(MessageQueue, core::ptr::addr_of_mut!(msg));
            AcceptMessage = co_IntProcessHardwareMessage(
                core::ptr::addr_of_mut!(msg),
                core::ptr::addr_of_mut!(Remove),
                core::ptr::addr_of_mut!(NotForUs),
                ExtraInfo,
                MsgFilterLow,
                MsgFilterHigh,
            );

            if NotForUs == 0 && (MsgFilterLow != 0 || MsgFilterHigh != 0) {
                if msg.message < MsgFilterLow || msg.message > MsgFilterHigh {
                    (*MessageQueue).msgDblClk = clk_msg;
                    (*MessageQueue).idSysPeek = idSave;
                    continue;
                }
            }

            if Remove != 0 {
                if !(*CurrentMessage).pti.is_null()
                    && (*MessageQueue).idSysPeek == CurrentMessage as ULONG_PTR
                {
                    MsqDestroyMessage(CurrentMessage);
                }
                ClearMsgBitsMask(pti, QS_Flags);
            }

            (*MessageQueue).idSysPeek = idSave;

            if NotForUs != 0 {
                Ret = 0;
                break;
            }

            if AcceptMessage != 0 {
                core::ptr::write_volatile(pMsg, msg);
                if RtlEqualMemory(
                    core::ptr::addr_of!((*pti).ptLast) as *const core::ffi::c_void,
                    core::ptr::addr_of!(msg.pt) as *const core::ffi::c_void,
                    core::mem::size_of::<POINT>() as SIZE_T,
                ) == 0
                {
                    (*pti).TIF_flags |= TIF_MSGPOSCHANGED;
                }
                (*pti).timeLast = msg.time as LONG;
                (*pti).ptLast = msg.pt;
                (*MessageQueue).ExtraInfo = ExtraInfo;
                Ret = 1;
                break;
            }
        }
    }

    (*MessageQueue).ptiSysLock = core::ptr::null_mut();
    (*(*pti).pcti).CTI_flags &= !CTI_THREADSYSLOCK;
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn MsqPeekMessage(
    pti: PTHREADINFO,
    Remove: BOOLEAN,
    Window: PWND,
    MsgFilterLow: UINT,
    MsgFilterHigh: UINT,
    QSflags: UINT,
    ExtraInfo: *mut LONG_PTR,
    dwQEvent: *mut DWORD,
    Message: PMSG,
) -> BOOLEAN {
    let mut CurrentMessage: PUSER_MESSAGE;
    let mut ListHead: PLIST_ENTRY;
    let mut QS_Flags: DWORD;
    let mut Ret: BOOLEAN = 0u8;

    ListHead = (*pti).PostedMessagesListHead.Flink;

    if IsListEmpty(ListHead) != 0 {
        return 0u8;
    }

    while ListHead != core::ptr::addr_of_mut!((*pti).PostedMessagesListHead) {
        CurrentMessage = (ListHead as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as PUSER_MESSAGE;
        ListHead = (*ListHead).Flink;

        if (Window.is_null()
            || (Window == PWND_BOTTOM as PWND && (*CurrentMessage).Msg.hwnd.is_null())
            || (Window != PWND_BOTTOM as PWND
                && UserHMGetHandle(Window) == (*CurrentMessage).Msg.hwnd))
            && (((MsgFilterLow == 0 && MsgFilterHigh == 0)
                && (*CurrentMessage).QS_Flags & QSflags != 0)
                || (MsgFilterLow <= (*CurrentMessage).Msg.message
                    && MsgFilterHigh >= (*CurrentMessage).Msg.message))
        {
            core::ptr::write_volatile(Message, (*CurrentMessage).Msg);
            core::ptr::write_volatile(ExtraInfo, (*CurrentMessage).ExtraInfo);
            QS_Flags = (*CurrentMessage).QS_Flags;
            if !dwQEvent.is_null() {
                core::ptr::write_volatile(dwQEvent, (*CurrentMessage).dwQEvent);
            }

            if Remove != 0 {
                if !(*CurrentMessage).pti.is_null() {
                    MsqDestroyMessage(CurrentMessage);
                }
                ClearMsgBitsMask(pti, QS_Flags);
            }
            Ret = 1u8;
            break;
        }
    }

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqWaitForNewMessages(
    pti: PTHREADINFO,
    WndFilter: PWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
) -> NTSTATUS {
    co_MsqWaitForNewMessages_impl(pti, WndFilter, MsgFilterMin, MsgFilterMax)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqWaitForNewMessages(
    pti: PTHREADINFO,
    WndFilter: PWND,
    MsgFilterMin: UINT,
    MsgFilterMax: UINT,
) -> NTSTATUS {
    co_MsqWaitForNewMessages_impl(pti, WndFilter, MsgFilterMin, MsgFilterMax)
}

unsafe fn co_MsqWaitForNewMessages_impl(
    pti: PTHREADINFO,
    _WndFilter: PWND,
    _MsgFilterMin: UINT,
    _MsgFilterMax: UINT,
) -> NTSTATUS {

    let mut ret: NTSTATUS = STATUS_SUCCESS;

    if (*(*pti).MessageQueue).QF_flags & QF_MOUSEMOVED != 0 {
        IntCoalesceMouseMove(pti);
    }

    UserLeaveCo();

    ZwYieldExecution();

    ret = KeWaitForSingleObject(
        (*pti).pEventQueueServer as PVOID,
        _KWAIT_REASON_UserRequest,
        1i32 as i8,
        0u8,
        core::ptr::null_mut(),
    );
    UserEnterCo();
    if ret == STATUS_USER_APC {
        co_IntDeliverUserAPC();
    }
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL {
    MsqIsHung_impl(pti, TimeOut)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqIsHung(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL {
    MsqIsHung_impl(pti, TimeOut)
}

unsafe fn MsqIsHung_impl(pti: PTHREADINFO, TimeOut: DWORD) -> BOOL {

    let dwTimeStamp = EngGetTickCount32();
    if dwTimeStamp.wrapping_sub((*(*pti).pcti).timeLastRead) > TimeOut
        && ((*(*pti).pcti).fsWakeMask & QS_INPUT as u16) == 0
        && PsGetThreadFreezeCount(pti_ethread(pti) as _) == 0
        && (ppi_w32pf_flags((*pti).ppi) & W32PF_APPSTARTING) == 0
    {
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IsThreadSuspended(pti: PTHREADINFO) -> BOOL {
    IsThreadSuspended_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IsThreadSuspended(pti: PTHREADINFO) -> BOOL {
    IsThreadSuspended_impl(pti)
}

#[inline]
unsafe fn IsThreadSuspended_impl(pti: PTHREADINFO) -> BOOL {
    if !pti_ethread(pti).is_null() {
        let mut Ret: BOOL = 1;
        if kt_suspend_count(&mut (*pti_ethread(pti)).Tcb) == 0 && PsGetThreadFreezeCount(pti_ethread(pti) as _) == 0 {
            Ret = 0;
        }
        return Ret;
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn HungAppSysTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    DoTheScreenSaver();
    TRACE!(b"HungAppSysTimerProc\n\0".as_ptr() as *const i8);
    // Process list of windows that are hung and waiting.
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsqInitializeMessageQueue(
    pti: PTHREADINFO,
    MessageQueue: PUSER_MESSAGE_QUEUE,
) -> u8 {
    MsqInitializeMessageQueue_impl(pti, MessageQueue)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsqInitializeMessageQueue(
    pti: PTHREADINFO,
    MessageQueue: PUSER_MESSAGE_QUEUE,
) -> u8 {
    MsqInitializeMessageQueue_impl(pti, MessageQueue)
}

unsafe fn MsqInitializeMessageQueue_impl(
    pti: PTHREADINFO,
    MessageQueue: PUSER_MESSAGE_QUEUE,
) -> u8 {
    InitializeListHead(core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead));
    (*MessageQueue).spwndFocus = core::ptr::null_mut();
    (*MessageQueue).iCursorLevel = 0;
    (*MessageQueue).CursorObject = SYSTEMCUR(ROCR_WAIT);
    if !(*MessageQueue).CursorObject.is_null() {
        TRACE!(
            b"Default cursor hcur %p\n\0".as_ptr(),
            UserHMGetHandle((*MessageQueue).CursorObject),
        );
        UserReferenceObject((*MessageQueue).CursorObject as *mut _);
    }
    RtlCopyMemory(
        (*MessageQueue).afKeyState.as_mut_ptr() as *mut _,
        (*(&raw const gafAsyncKeyState)).as_ptr() as *const _,
        core::mem::size_of_val(&*(&raw const gafAsyncKeyState)) as _,
    );
    (*MessageQueue).ptiMouse = pti;
    (*MessageQueue).ptiKeyboard = pti;
    (*MessageQueue).cThreads += 1;

    1u8
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqCleanupThreadMsgs(pti: PTHREADINFO) {
    MsqCleanupThreadMsgs_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqCleanupThreadMsgs(pti: PTHREADINFO) {
    MsqCleanupThreadMsgs_impl(pti)
}

unsafe fn MsqCleanupThreadMsgs_impl(pti: PTHREADINFO) {
    let mut CurrentEntry: PLIST_ENTRY;
    let mut CurrentMessage: PUSER_MESSAGE;
    let mut CurrentSentMessage: PUSER_SENT_MESSAGE;

    if !(*pti).pcti.is_null() {
        (*(*pti).pcti).fsWakeBits = 0;
        (*(*pti).pcti).fsChangeBits = 0;
    }

    (*pti).nCntsQBits[QSRosKey as usize] = 0;
    (*pti).nCntsQBits[QSRosMouseMove as usize] = 0;
    (*pti).nCntsQBits[QSRosMouseButton as usize] = 0;
    (*pti).nCntsQBits[QSRosPostMessage as usize] = 0;
    (*pti).nCntsQBits[QSRosSendMessage as usize] = 0;
    (*pti).nCntsQBits[QSRosHotKey as usize] = 0;
    (*pti).nCntsQBits[QSRosEvent as usize] = 0;

    while IsListEmpty(core::ptr::addr_of_mut!((*pti).PostedMessagesListHead)) == 0 {
        CurrentEntry = (*pti).PostedMessagesListHead.Flink;
        CurrentMessage = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
            as PUSER_MESSAGE;
        if (*CurrentMessage).dwQEvent != 0 {
            if (*CurrentMessage).dwQEvent == POSTEVENT_NWE as u32 {
                ExFreePoolWithTag((*CurrentMessage).ExtraInfo as PVOID, TAG_HOOK);
            }
        }
        MsqDestroyMessage(CurrentMessage);
    }

    while IsListEmpty(core::ptr::addr_of_mut!((*pti).SentMessagesListHead)) == 0 {
        CurrentEntry = (*pti).SentMessagesListHead.Flink;
        CurrentSentMessage = (CurrentEntry as *mut u8)
            .offset(-(core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry) as isize))
            as PUSER_SENT_MESSAGE;

        if !(*CurrentSentMessage).pkCompletionEvent.is_null() {
            KeSetEvent((*CurrentSentMessage).pkCompletionEvent as _, IO_NO_INCREMENT as KPRIORITY, 0u8);
        }

        if (*CurrentSentMessage).HasPackedLParam != 0 {
            if (*CurrentSentMessage).Msg.lParam != 0 {
                ExFreePool((*CurrentSentMessage).Msg.lParam as PVOID);
            }
        }

        FreeUserMessage(CurrentSentMessage);
    }

    if IsListEmpty(core::ptr::addr_of_mut!(usmList)) == 0 {
        CurrentEntry = usmList.Flink;
        while CurrentEntry != core::ptr::addr_of_mut!(usmList) {
            CurrentSentMessage = (CurrentEntry as *mut u8)
                .offset(-(core::mem::offset_of!(USER_SENT_MESSAGE, ListEntry) as isize))
                as PUSER_SENT_MESSAGE;
            CurrentEntry = (*CurrentEntry).Flink;

            if pti == (*CurrentSentMessage).ptiReceiver {
                if (*CurrentSentMessage).HasPackedLParam != 0 {
                    if (*CurrentSentMessage).Msg.lParam != 0 {
                        ExFreePool((*CurrentSentMessage).Msg.lParam as PVOID);
                    }
                }

                FreeUserMessage(CurrentSentMessage);
            } else if pti == (*CurrentSentMessage).ptiSender
                || pti == (*CurrentSentMessage).ptiCallBackSender
            {
                if ((*CurrentSentMessage).flags & (SMF_RECEIVERBUSY | SMF_RECEIVEDMESSAGE))
                    != SMF_RECEIVEDMESSAGE
                {
                    (*CurrentSentMessage).flags |= SMF_RECEIVERFREE;
                }

                if ((*CurrentSentMessage).flags & SMF_RECEIVERFREE) == 0 {
                    if (*CurrentSentMessage).HasPackedLParam != 0 {
                        if (*CurrentSentMessage).Msg.lParam != 0 {
                            ExFreePool((*CurrentSentMessage).Msg.lParam as PVOID);
                        }
                    }

                    FreeUserMessage(CurrentSentMessage);
                }
            }
        }
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn MsqCleanupMessageQueue(pti: PTHREADINFO) {
    MsqCleanupMessageQueueImpl(pti)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn MsqCleanupMessageQueue(pti: PTHREADINFO) {
    MsqCleanupMessageQueueImpl(pti)
}

unsafe fn MsqCleanupMessageQueueImpl(pti: PTHREADINFO) {
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    (*MessageQueue).cThreads -= 1;

    if (*MessageQueue).cThreads != 0 {
        if (*MessageQueue).ptiSysLock == pti {
            (*MessageQueue).ptiSysLock = core::ptr::null_mut();
        }
    }

    if (*MessageQueue).cThreads == 0 {
        while IsListEmpty(core::ptr::addr_of_mut!((*MessageQueue).HardwareMessagesListHead)) == 0 {
            let CurrentEntry: PLIST_ENTRY = (*MessageQueue).HardwareMessagesListHead.Flink;
            let CurrentMessage: PUSER_MESSAGE = (CurrentEntry as *mut u8)
                .offset(-(core::mem::offset_of!(USER_MESSAGE, ListEntry) as isize))
                as PUSER_MESSAGE;
            MsqDestroyMessage(CurrentMessage);
        }
    }

    if !(*MessageQueue).CursorObject.is_null() {
        let pCursor: PCURICON_OBJECT = (*MessageQueue).CursorObject;

        if (*IntGetSysCursorInfo()).CurrentCursorObject == pCursor {
            let hdcScreen: HDC = IntGetScreenDC();
            if !hdcScreen.is_null() {
                GreMovePointer(hdcScreen, -1, -1);
            }
            (*IntGetSysCursorInfo()).CurrentCursorObject = core::ptr::null_mut();
        }

        UserDereferenceObject(pCursor as PVOID);
    }

    if gpqForeground == MessageQueue {
        IntSetFocusMessageQueue(core::ptr::null_mut());
    }
    if gpqForegroundPrev == MessageQueue {
        gpqForegroundPrev = core::ptr::null_mut();
    }
    if gpqCursor == MessageQueue {
        gpqCursor = core::ptr::null_mut();
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqCreateMessageQueue(pti: PTHREADINFO) -> PUSER_MESSAGE_QUEUE {
    MsqCreateMessageQueue_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqCreateMessageQueue(pti: PTHREADINFO) -> PUSER_MESSAGE_QUEUE {
    MsqCreateMessageQueue_impl(pti)
}

unsafe fn MsqCreateMessageQueue_impl(pti: PTHREADINFO) -> PUSER_MESSAGE_QUEUE {
    let MessageQueue: PUSER_MESSAGE_QUEUE = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<USER_MESSAGE_QUEUE>() as SIZE_T,
        USERTAG_Q,
    ) as PUSER_MESSAGE_QUEUE;

    if MessageQueue.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(MessageQueue as *mut _, core::mem::size_of::<USER_MESSAGE_QUEUE>());
    IntReferenceMessageQueue(MessageQueue);
    if MsqInitializeMessageQueue(pti, MessageQueue) == 0 {
        IntDereferenceMessageQueue(MessageQueue);
        return core::ptr::null_mut();
    }

    MessageQueue
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqDestroyMessageQueue(pti: PTHREADINFO) {
    MsqDestroyMessageQueueImpl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqDestroyMessageQueue(pti: PTHREADINFO) {
    MsqDestroyMessageQueueImpl(pti)
}

unsafe fn MsqDestroyMessageQueueImpl(pti: PTHREADINFO) {
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    (*MessageQueue).QF_flags |= QF_INDESTROY;

    let desk: PDESKTOP = InterlockedExchangePointer(
        core::ptr::addr_of_mut!((*MessageQueue).Desktop) as *mut *mut core::ffi::c_void,
        core::ptr::null_mut(),
    ) as PDESKTOP;

    if !desk.is_null() {
        InterlockedExchangePointer(
            core::ptr::addr_of_mut!((*desk).ActiveMessageQueue) as *mut *mut core::ffi::c_void,
            core::ptr::null_mut(),
        );
        IntDereferenceMessageQueue(MessageQueue);
    }

    MsqCleanupMessageQueue(pti);

    IntDereferenceMessageQueue(MessageQueue);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqSetMessageExtraInfo(lParam: LPARAM) -> LPARAM {
    MsqSetMessageExtraInfo_impl(lParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqSetMessageExtraInfo(lParam: LPARAM) -> LPARAM {
    MsqSetMessageExtraInfo_impl(lParam)
}

unsafe fn MsqSetMessageExtraInfo_impl(lParam: LPARAM) -> LPARAM {
    let Ret: LPARAM;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;
    if MessageQueue.is_null() {
        return 0;
    }

    Ret = (*MessageQueue).ExtraInfo;
    (*MessageQueue).ExtraInfo = lParam;

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqGetMessageExtraInfo() -> LPARAM {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if MessageQueue.is_null() {
        return 0;
    }
    (*MessageQueue).ExtraInfo
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqGetMessageExtraInfo() -> LPARAM {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;
    if MessageQueue.is_null() {
        return 0;
    }
    (*MessageQueue).ExtraInfo
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_MsqReplyMessage(lResult: LRESULT) -> BOOL {
    co_MsqReplyMessage_impl(lResult)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_MsqReplyMessage(lResult: LRESULT) -> BOOL {
    co_MsqReplyMessage_impl(lResult)
}

unsafe fn co_MsqReplyMessage_impl(lResult: LRESULT) -> BOOL {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let Message: PUSER_SENT_MESSAGE = (*pti).pusmCurrent;

    if Message.is_null() {
        return 0;
    }

    if ((*Message).QS_Flags & QS_SMRESULT) != 0 {
        return 0;
    }

    if !(*Message).ptiSender.is_null() || (*Message).CompletionCallback.is_some() {
        (*Message).lResult = lResult;
        (*Message).QS_Flags |= QS_SMRESULT;
    }
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqSetStateWindow(pti: PTHREADINFO, Type: ULONG, hWnd: HWND) -> HWND {
    MsqSetStateWindowImpl(pti, Type, hWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqSetStateWindow(pti: PTHREADINFO, Type: ULONG, hWnd: HWND) -> HWND {
    MsqSetStateWindowImpl(pti, Type, hWnd)
}

unsafe fn MsqSetStateWindowImpl(pti: PTHREADINFO, Type: ULONG, hWnd: HWND) -> HWND {
    let Prev: HWND;
    let MessageQueue: PUSER_MESSAGE_QUEUE = (*pti).MessageQueue;

    match Type {
        MSQ_STATE_CAPTURE => {
            Prev = if !(*MessageQueue).spwndCapture.is_null() {
                UserHMGetHandle((*MessageQueue).spwndCapture)
            } else {
                core::ptr::null_mut()
            };
            (*MessageQueue).spwndCapture = ValidateHwndNoErr(hWnd);
            return Prev;
        }
        MSQ_STATE_ACTIVE => {
            Prev = if !(*MessageQueue).spwndActive.is_null() {
                UserHMGetHandle((*MessageQueue).spwndActive)
            } else {
                core::ptr::null_mut()
            };
            (*MessageQueue).spwndActive = ValidateHwndNoErr(hWnd);
            return Prev;
        }
        MSQ_STATE_FOCUS => {
            Prev = if !(*MessageQueue).spwndFocus.is_null() {
                UserHMGetHandle((*MessageQueue).spwndFocus)
            } else {
                core::ptr::null_mut()
            };
            (*MessageQueue).spwndFocus = ValidateHwndNoErr(hWnd);
            return Prev;
        }
        MSQ_STATE_MENUOWNER => {
            Prev = (*MessageQueue).MenuOwner;
            (*MessageQueue).MenuOwner = hWnd;
            return Prev;
        }
        MSQ_STATE_MOVESIZE => {
            Prev = (*MessageQueue).MoveSize;
            (*MessageQueue).MoveSize = hWnd;
            return Prev;
        }
        MSQ_STATE_CARET => {
            Prev = (*MessageQueue).CaretInfo.hWnd;
            (*MessageQueue).CaretInfo.hWnd = hWnd;
            return Prev;
        }
        _ => {}
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn MsqReleaseModifierKeys(MessageQueue: PUSER_MESSAGE_QUEUE) {
    MsqReleaseModifierKeys_impl(MessageQueue)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn MsqReleaseModifierKeys(MessageQueue: PUSER_MESSAGE_QUEUE) {
    MsqReleaseModifierKeys_impl(MessageQueue)
}

#[inline]
unsafe fn MsqReleaseModifierKeys_impl(MessageQueue: PUSER_MESSAGE_QUEUE) {
    let ModifierKeys: [u16; 9] = [
        VK_LCONTROL as u16, VK_RCONTROL as u16, VK_CONTROL as u16,
        VK_LMENU as u16,    VK_RMENU as u16,    VK_MENU as u16,
        VK_LSHIFT as u16,   VK_RSHIFT as u16,   VK_SHIFT as u16,
    ];

    for i in 0..ModifierKeys.len() {
        let key = ModifierKeys[i];
        let afKeyState = (*MessageQueue).afKeyState.as_mut_ptr();
        if IS_KEY_DOWN(afKeyState, key) != 0 {
            SET_KEY_DOWN(afKeyState, key, 0u8);
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyState(key: INT) -> SHORT {
    let Ret: DWORD;

    UserEnterShared();

    Ret = UserGetKeyState(key as DWORD);

    UserLeave();

    Ret as SHORT
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyboardState(lpKeyState: LPBYTE) -> DWORD {
    let mut ret: DWORD = 1;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let Status: NTSTATUS;

    UserEnterShared();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    Status = SafeProbeForWrite(lpKeyState as *mut core::ffi::c_void, (256 * core::mem::size_of::<u8>()) as ULONG, 1);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        ret = 0;
    } else {
        // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND all 768 of
        // these stores inside one _SEH2_TRY. Build the table in kernel memory and
        // publish it in one fault-tolerant copy — the `|=` was a read-modify-write
        // of user memory besides.
        let mut SafeKeyState: [u8; 256] = [0; 256];
        for i in 0..256u32 {
            if IS_KEY_DOWN((*MessageQueue).afKeyState.as_ptr(), i) != 0 {
                SafeKeyState[i as usize] |= KS_DOWN_BIT as u8;
            }
            if IS_KEY_LOCKED((*MessageQueue).afKeyState.as_ptr(), i) != 0 {
                SafeKeyState[i as usize] |= KS_LOCK_BIT as u8;
            }
        }
        if !crate::copy::publish_to_user(
            lpKeyState as *mut core::ffi::c_void,
            SafeKeyState.as_ptr() as *const core::ffi::c_void,
            SafeKeyState.len(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            ret = 0;
        }
    }

    UserLeave();

    ret
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserGetKeyboardState(lpKeyState: LPBYTE) -> DWORD {
    let mut ret: DWORD = 1;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let Status: NTSTATUS;

    UserEnterShared();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    Status = SafeProbeForWrite(lpKeyState as *mut core::ffi::c_void, (256 * core::mem::size_of::<u8>()) as _, 1);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        ret = 0;
    } else {
        // Probe is validate-only (USE_DUMMY_PSEH); the C had probe AND all 768 of
        // these stores inside one _SEH2_TRY. Build the table in kernel memory and
        // publish it in one fault-tolerant copy — the `|=` was a read-modify-write
        // of user memory besides.
        let mut SafeKeyState: [u8; 256] = [0; 256];
        for i in 0..256u32 {
            if IS_KEY_DOWN((*MessageQueue).afKeyState.as_ptr(), i) != 0 {
                SafeKeyState[i as usize] |= KS_DOWN_BIT as u8;
            }
            if IS_KEY_LOCKED((*MessageQueue).afKeyState.as_ptr(), i) != 0 {
                SafeKeyState[i as usize] |= KS_LOCK_BIT as u8;
            }
        }
        if !crate::copy::publish_to_user(
            lpKeyState as *mut core::ffi::c_void,
            SafeKeyState.as_ptr() as *const core::ffi::c_void,
            SafeKeyState.len(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            ret = 0;
        }
    }

    UserLeave();

    ret
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserSetKeyboardState(pKeyState: LPBYTE) -> BOOL {
    let mut bRet: BOOL = 1;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let Status: NTSTATUS;

    UserEnterExclusive();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    Status = SafeProbeForRead(pKeyState as *mut core::ffi::c_void, (256 * core::mem::size_of::<u8>()) as ULONG, 1);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        bRet = 0;
    } else {
        // Probe is validate-only; the C had probe AND all 512 loads in one
        // _SEH2_TRY. One fault-tolerant copy, then read kernel memory.
        let mut SafeKeyState: [u8; 256] = [0; 256];
        if !crate::copy::capture_from_user(
            SafeKeyState.as_mut_ptr() as *mut core::ffi::c_void,
            pKeyState as *const core::ffi::c_void,
            SafeKeyState.len(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            return 0;
        }
        let mut i: UINT = 0;
        while i < 256 {
            SET_KEY_DOWN(
                (*MessageQueue).afKeyState.as_mut_ptr() as *mut u8,
                i,
                (SafeKeyState[i as usize] & KS_DOWN_BIT as u8) as BOOL,
            );
            SET_KEY_LOCKED(
                (*MessageQueue).afKeyState.as_mut_ptr() as *mut u8,
                i,
                (SafeKeyState[i as usize] & KS_LOCK_BIT as u8) as BOOL,
            );
            i += 1;
        }
    }

    UserLeave();

    bRet
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserSetKeyboardState(pKeyState: LPBYTE) -> BOOL {
    let mut bRet: BOOL = 1;
    let pti: PTHREADINFO;
    let MessageQueue: PUSER_MESSAGE_QUEUE;
    let Status: NTSTATUS;

    UserEnterExclusive();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    MessageQueue = (*pti).MessageQueue;

    Status = SafeProbeForRead(pKeyState as *mut core::ffi::c_void, (256 * core::mem::size_of::<u8>()) as _, 1);
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        bRet = 0;
    } else {
        // Probe is validate-only; the C had probe AND all 512 loads in one
        // _SEH2_TRY. One fault-tolerant copy, then read kernel memory.
        let mut SafeKeyState: [u8; 256] = [0; 256];
        if !crate::copy::capture_from_user(
            SafeKeyState.as_mut_ptr() as *mut core::ffi::c_void,
            pKeyState as *const core::ffi::c_void,
            SafeKeyState.len(),
        ) {
            SetLastNtError(STATUS_ACCESS_VIOLATION);
            return 0;
        }
        let mut i: UINT = 0;
        while i < 256 {
            SET_KEY_DOWN(
                (*MessageQueue).afKeyState.as_mut_ptr() as *mut u8,
                i,
                (SafeKeyState[i as usize] & KS_DOWN_BIT as u8) as BOOL,
            );
            SET_KEY_LOCKED(
                (*MessageQueue).afKeyState.as_mut_ptr() as *mut u8,
                i,
                (SafeKeyState[i as usize] & KS_LOCK_BIT as u8) as BOOL,
            );
            i += 1;
        }
    }

    UserLeave();

    bRet
}
