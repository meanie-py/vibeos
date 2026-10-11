#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{
    ExInitializeResourceLite, KeSetTimer, PsGetCurrentThreadWin32Thread,
    ExAllocatePoolWithTag, ExAcquireResourceExclusiveLite, ExReleaseResourceLite,
    KeEnterCriticalRegion, KeLeaveCriticalRegion,
};
use ntkrnl_sys::base::{
    NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES,
    InsertTailList, RemoveEntryList, InitializeListHead,
};
use ntkrnl_sys::constants::STATUS_UNSUCCESSFUL;
use ntkrnl_sys::rtl::{RtlClearBit, RtlFindClearBitsAndSet, RtlInitializeBitMap, RtlClearAllBits};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::mm::ALIGN_UP_BY;
use ntkrnl_sys::CONTAINING_RECORD;
use core::ffi::c_void;

// NT_SUCCESS is imported for parity with sibling translations.
const _: () = { let _ = NT_SUCCESS; };

// USERTAG / GDI pool tags. C multi-char literals are big-endian; the LE byte
// order is the literal reversed.
//   TAG_INTERNAL_SYNC = 'cnys' (0x636E7973) -> LE bytes "sync"
//   TAG_TIMERBMP      = 'BMIT' (0x424D4954) -> LE bytes "TIMB"
const TAG_INTERNAL_SYNC: u32 = u32::from_le_bytes(*b"sync");
const TAG_TIMERBMP: u32 = u32::from_le_bytes(*b"TIMB");

// --- FORCEINLINE / macro helpers (no exported kernel symbol) ---

// UserHMGetHandle(obj) = obj->head.h  (HANDLE at offset 0 of HEAD).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HWND {
    *(obj as *const HWND)
}

// UserHMSetHandle(obj, handle) = obj->head.h = handle.
#[inline(always)]
unsafe fn UserHMSetHandle<T>(obj: *mut T, handle: HANDLE) {
    *(obj as *mut HANDLE) = handle;
}

// TimerEnterExclusive()/TimerLeave() — C macros wrapping the TimerLock ERESOURCE.
#[inline(always)]
unsafe fn TimerEnterExclusive() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(TimerLock) as _, 1u8);
}
#[inline(always)]
unsafe fn TimerLeave() {
    ExReleaseResourceLite(core::ptr::addr_of_mut!(TimerLock) as _);
    KeLeaveCriticalRegion();
}

// IntLock/UnlockWindowlessTimerBitmap() — C macros over the Mutex FAST_MUTEX.
#[inline(always)]
unsafe fn IntLockWindowlessTimerBitmap() {
    ExEnterCriticalRegionAndAcquireFastMutexUnsafe(Mutex);
}
#[inline(always)]
unsafe fn IntUnlockWindowlessTimerBitmap() {
    ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(Mutex);
}

// MAKELPARAM(low, high) macro.
#[inline(always)]
fn MAKELPARAM(low: u32, high: u32) -> LPARAM {
    (((low & 0xffff) | ((high & 0xffff) << 16)) as u32) as i32 as LPARAM
}

// RECTL_bPointInRect — FORCEINLINE in win32ss/gdi/ntgdi/rect.h.
#[inline(always)]
unsafe fn RECTL_bPointInRect(prcl: *const RECTL, x: LONG, y: LONG) -> BOOL {
    (x >= (*prcl).left && x < (*prcl).right && y >= (*prcl).top && y < (*prcl).bottom) as BOOL
}

// gpsi->ptCursor accessor. tagSERVERINFO ends with __bindgen_padding_0
// representing the anonymous PERUSERSERVERINFO member; cast that tail to access
// ptCursor (matches mouse.rs/winsta.rs).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into __bindgen_padding_0.
// pti is a pointer (ptr-aligned) following HEAD{HANDLE, DWORD}, so on amd64 there is
// 4 bytes of tail padding: pti sits at offset 16 (amd64) / 8 (i386). HANDLE+DWORD
// (=12 on amd64) MISSES the padding — use 2*usize, which is 16/8 correctly per arch
// (matches event.rs head_pti). [amd64 BUGCHECK 0x1E AV fix, timer wave 17]
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// W32THREAD.pEThread is the first field of THREADINFO (offset 0); bindgen
// flattened the W32THREAD prefix into __bindgen_padding_0 (matches event.rs).
#[inline(always)]
unsafe fn pti_ethread(pti: PTHREADINFO) -> PETHREAD {
    *(pti as *const PETHREAD)
}

// (LPARAM)pTmr->pfn — TIMERPROC is Option<extern fn>; its layout is identical to
// the raw function pointer (None == null), so transmute to an integer LPARAM.
#[inline(always)]
unsafe fn pfn_as_lparam(pfn: TIMERPROC) -> LPARAM {
    core::mem::transmute::<TIMERPROC, usize>(pfn) as LPARAM
}

// --- Cross-file functions (real symbols; ABI verified against win32ss objs) ---

// FASTCALL on i386 (`@Name@N`), cdecl elsewhere. (msgqueue.h / engmisc.c)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR) -> BOOL;
    fn MsqGetDownKeyState(MessageQueue: PUSER_MESSAGE_QUEUE) -> WPARAM;
    fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ClearMsgBitsMask(pti: PTHREADINFO, MessageBits: UINT);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR) -> BOOL;
    fn MsqGetDownKeyState(MessageQueue: PUSER_MESSAGE_QUEUE) -> WPARAM;
    fn MsqWakeQueue(pti: PTHREADINFO, MessageBits: DWORD, KeyEvent: BOOL);
}

// EngGetTickCount32() = (ULONG)EngGetTickCount(); EngGetTickCount is APIENTRY
// (stdcall) per eng.h.
extern "system" {
    fn EngGetTickCount() -> ULONGLONG;
}
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG {
    EngGetTickCount() as ULONG
}

// ExEnterCriticalRegionAndAcquireFastMutexUnsafe / release — FASTCALL per
// ntoskrnl.spec (`@Name@4` in nm); fastcall on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    fn ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex: PFAST_MUTEX);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ExEnterCriticalRegionAndAcquireFastMutexUnsafe(FastMutex: PFAST_MUTEX);
    fn ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(FastMutex: PFAST_MUTEX);
}

// ExInitializeFastMutex — FORCEINLINE (xdk/exfuncs.h), no exported symbol.
// Reimplement inline: Count=FM_LOCK_BIT, Owner/Contention=0, init the event.
// KeInitializeEvent is stdcall NTAPI (`_KeInitializeEvent@12`).
extern "system" {
    fn KeInitializeEvent(Event: PRKEVENT, Type: EVENT_TYPE, State: BOOLEAN);
}
const FM_LOCK_BIT: LONG = 0x1;
#[inline(always)]
unsafe fn ExInitializeFastMutex(FastMutex: PFAST_MUTEX) {
    (*FastMutex).Count = FM_LOCK_BIT;
    (*FastMutex).Owner = core::ptr::null_mut();
    (*FastMutex).Contention = 0;
    KeInitializeEvent(
        core::ptr::addr_of_mut!((*FastMutex).Event) as PRKEVENT,
        _EVENT_TYPE_SynchronizationEvent as EVENT_TYPE,
        0,
    );
}

// HungAppSysTimerProc — VOID CALLBACK (msgqueue.h). Must match the TIMERPROC
// fn-pointer ABI exactly: bindgen emits TIMERPROC as `extern "stdcall"` on i386
// (CALLBACK=stdcall, `_HungAppSysTimerProc@16`) and `extern "C"` on amd64/arm64.
extern "system" {
    fn HungAppSysTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD);
}

// ---- file-scope preamble (translated) ----
static mut TimersListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };
static mut TimeLast: LONG = 0;

pub const MAX_WINDOW_LESS_TIMER_ID: ULONG = 32768 - 1;
pub const NUM_WINDOW_LESS_TIMERS: ULONG = 32768 - 256;
pub const HINTINDEX_BEGIN_VALUE: ULONG = 0;

static mut Mutex: PFAST_MUTEX = core::ptr::null_mut();
static mut WindowLessTimersBitMap: RTL_BITMAP = unsafe { core::mem::zeroed() };
static mut WindowLessTimersBitMapBuffer: PVOID = core::ptr::null_mut();
static mut HintIndex: ULONG = HINTINDEX_BEGIN_VALUE;
pub static mut TimerLock: ERESOURCE = unsafe { core::mem::zeroed() };

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CreateTimer() -> PTIMER {
    CreateTimer_impl()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CreateTimer() -> PTIMER {
    CreateTimer_impl()
}

unsafe fn CreateTimer_impl() -> PTIMER {

    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut Ret: PTIMER = core::ptr::null_mut();

    Ret = UserCreateObject(
        gHandleTable,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut Handle,
        TYPE_TIMER,
        core::mem::size_of::<TIMER>() as _,
    ) as PTIMER;

    if !Ret.is_null() {
        UserHMSetHandle(Ret as *mut core::ffi::c_void, Handle);
        // NOT linked here, unlike the C: IntSetTimer links it once it is initialized.
    }

    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn RemoveTimer(pTmr: PTIMER) -> BOOL {
    RemoveTimer_impl(pTmr)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn RemoveTimer(pTmr: PTIMER) -> BOOL {
    RemoveTimer_impl(pTmr)
}

unsafe fn RemoveTimer_impl(pTmr: PTIMER) -> BOOL {
    let mut Ret: BOOL = 0;
    if !pTmr.is_null() {
        RemoveEntryList(core::ptr::addr_of_mut!((*pTmr).ptmrList) as _);
        if (*pTmr).pWnd.is_null() && ((*pTmr).flags & TMRF_SYSTEM as u32) == 0 {
            let ulBitmapIndex: ULONG = (MAX_WINDOW_LESS_TIMER_ID - (*pTmr).nID as u32) as ULONG;
            IntLockWindowlessTimerBitmap();
            RtlClearBit(core::ptr::addr_of_mut!(WindowLessTimersBitMap) as _, ulBitmapIndex);
            IntUnlockWindowlessTimerBitmap();
        }
        UserDereferenceObject(pTmr as PVOID);
        Ret = UserDeleteObject(UserHMGetHandle(pTmr) as PVOID, TYPE_TIMER as u32);
    }
    if Ret == 0 {
        ERR!("Warning: Unable to delete timer\n");
    }

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn FindTimer(Window: PWND, nID: UINT_PTR, flags: UINT) -> PTIMER {
    FindTimer_impl(Window, nID, flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn FindTimer(Window: PWND, nID: UINT_PTR, flags: UINT) -> PTIMER {
    FindTimer_impl(Window, nID, flags)
}

unsafe fn FindTimer_impl(Window: PWND, nID: UINT_PTR, flags: UINT) -> PTIMER {
    let mut pLE: PLIST_ENTRY;
    let mut pTmr: PTIMER;
    let mut RetTmr: PTIMER = core::ptr::null_mut();

    TimerEnterExclusive();
    pLE = TimersListHead.Flink;
    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        pTmr = (pLE as *mut u8)
            .offset(-(core::mem::offset_of!(TIMER, ptmrList) as isize)) as PTIMER;

        if (*pTmr).nID == nID
            && (*pTmr).pWnd == Window
            && ((*pTmr).flags & (TMRF_SYSTEM | TMRF_RIT))
                == (flags & (TMRF_SYSTEM | TMRF_RIT))
        {
            RetTmr = pTmr;
            break;
        }

        pLE = (*pLE).Flink;
    }
    TimerLeave();

    RetTmr
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn FindSystemTimer(pMsg: PMSG) -> PTIMER {
    FindSystemTimer_impl(pMsg)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn FindSystemTimer(pMsg: PMSG) -> PTIMER {
    FindSystemTimer_impl(pMsg)
}

unsafe fn FindSystemTimer_impl(pMsg: PMSG) -> PTIMER {
    let mut pLE: PLIST_ENTRY;
    let mut pTmr: PTIMER = core::ptr::null_mut();

    TimerEnterExclusive();
    pLE = TimersListHead.Flink;
    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        pTmr = (pLE as *mut u8).offset(-(core::mem::offset_of!(TIMER, ptmrList) as isize)) as PTIMER;

        if (*pMsg).lParam == pfn_as_lparam((*pTmr).pfn) && ((*pTmr).flags & TMRF_SYSTEM as u32) != 0 {
            break;
        }

        pLE = (*pLE).Flink;
    }
    TimerLeave();

    pTmr
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn ValidateTimerCallback(pti: PTHREADINFO, lParam: LPARAM) -> BOOL {
    ValidateTimerCallback_impl(pti, lParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn ValidateTimerCallback(pti: PTHREADINFO, lParam: LPARAM) -> BOOL {
    ValidateTimerCallback_impl(pti, lParam)
}

unsafe fn ValidateTimerCallback_impl(pti: PTHREADINFO, lParam: LPARAM) -> BOOL {
    let mut Ret: BOOL = 0;

    TimerEnterExclusive();
    let mut pLE: PLIST_ENTRY = TimersListHead.Flink;
    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        let pTmr: PTIMER = (pLE as *mut u8)
            .sub(core::mem::offset_of!(TIMER, ptmrList)) as PTIMER;
        if lParam == pfn_as_lparam((*pTmr).pfn)
            && ((*pTmr).flags & (TMRF_SYSTEM | TMRF_RIT)) == 0
            && (*(*pTmr).pti).ppi == (*pti).ppi
        {
            Ret = 1;
            break;
        }
        pLE = (*pLE).Flink;
    }
    TimerLeave();

    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetTimer(
    Window: PWND,
    IDEvent: UINT_PTR,
    Elapse: UINT,
    TimerFunc: TIMERPROC,
    Type: INT,
) -> UINT_PTR {
    IntSetTimer_impl(Window, IDEvent, Elapse, TimerFunc, Type)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetTimer(
    Window: PWND,
    IDEvent: UINT_PTR,
    Elapse: UINT,
    TimerFunc: TIMERPROC,
    Type: INT,
) -> UINT_PTR {
    IntSetTimer_impl(Window, IDEvent, Elapse, TimerFunc, Type)
}

unsafe fn IntSetTimer_impl(
    Window: PWND,
    mut IDEvent: UINT_PTR,
    mut Elapse: UINT,
    TimerFunc: TIMERPROC,
    Type: INT,
) -> UINT_PTR {

    let mut pTmr: PTIMER;
    let mut Ret: UINT_PTR = IDEvent;
    let ulBitmapIndex: ULONG;
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    DueTime.QuadPart = -97656i64;

    if Elapse > USER_TIMER_MAXIMUM {
        Elapse = USER_TIMER_MAXIMUM;
    }

    if Elapse < USER_TIMER_MINIMUM {
        Elapse = USER_TIMER_MINIMUM;
    }

    if !Window.is_null() && IDEvent == 0 {
        Ret = 1;
    }

    pTmr = FindTimer(Window, IDEvent, Type as UINT);

    if pTmr.is_null() && Window.is_null() && (Type & TMRF_SYSTEM as INT) == 0 {
        IntLockWindowlessTimerBitmap();

        ulBitmapIndex = RtlFindClearBitsAndSet(
            core::ptr::addr_of_mut!(WindowLessTimersBitMap) as _,
            1,
            HintIndex,
        );
        HintIndex = (ulBitmapIndex + 1) % (32768 - 256);
        if ulBitmapIndex == ULONG_MAX {
            IntUnlockWindowlessTimerBitmap();
            EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
            return 0;
        }

        IDEvent = (32768 - 1) - ulBitmapIndex as UINT_PTR;
        Ret = IDEvent;

        IntUnlockWindowlessTimerBitmap();
    }

    if pTmr.is_null() {
        pTmr = CreateTimer();
        if pTmr.is_null() {
            return 0;
        }

        if !Window.is_null() && (Type & TMRF_TIFROMWND as INT) != 0 {
            (*pTmr).pti = (*pti_ethread(head_pti(&(*Window).head))).Tcb.Win32Thread as _;
        } else {
            if (Type & TMRF_RIT as INT) != 0 {
                (*pTmr).pti = ptiRawInput;
            } else {
                (*pTmr).pti = PsGetCurrentThreadWin32Thread() as _;
            }
        }

        (*pTmr).pWnd = Window;
        (*pTmr).cmsCountdown = Elapse as INT;
        (*pTmr).cmsRate = Elapse as INT;
        (*pTmr).pfn = TimerFunc;
        (*pTmr).nID = IDEvent;
        (*pTmr).flags = (Type | TMRF_INIT as INT) as _;

        // Divergence from the C, which links the timer in CreateTimer, before any field is
        // set, and without TimerLock. ProcessTimers walks the list on the raw input thread
        // under TimerLock, so on two CPUs it can meet a zeroed timer: one pass drives the
        // countdown negative, the next dereferences its NULL pti (BUGCHECK 0x1E at
        // ProcessTimers, amd64-bios -smp 2 gates 2026-10-09/10; the live list showed the
        // timer with pti/rate/flags/pfn all zero and countdown -16). Link it initialized,
        // under the lock every other list user takes.
        TimerEnterExclusive();
        InsertTailList(
            core::ptr::addr_of_mut!(TimersListHead) as _,
            core::ptr::addr_of_mut!((*pTmr).ptmrList) as _,
        );
        TimerLeave();
    } else {
        (*pTmr).cmsCountdown = Elapse as INT;
        (*pTmr).cmsRate = Elapse as INT;
    }

    if TimersListHead.Flink == TimersListHead.Blink {
        KeSetTimer(MasterTimer as _, core::mem::transmute(DueTime), core::ptr::null_mut());
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SystemTimerProc(hwnd: HWND, uMsg: UINT, idEvent: UINT_PTR, dwTime: DWORD) {
    let pDesk: PDESKTOP;
    let mut pWnd: PWND = core::ptr::null_mut();

    if !hwnd.is_null() {
        pWnd = UserGetWindowObject(hwnd);
        if pWnd.is_null() {
            ERR!("System Timer Proc has invalid window handle! %p Id: %u\n", hwnd, idEvent);
            return;
        }
    } else {
        TRACE!(b"Windowless Timer Running!\n\0".as_ptr());
        return;
    }

    // idEvent is UINT_PTR (u32 on i386, u64 on amd64/arm64); the ID_EVENT_*
    // constants are u32 — widen them to UINT_PTR to match the switch selector.
    match idEvent {
        x if x == ID_EVENT_SYSTIMER_MOUSEHOVER as UINT_PTR => {
            let mut Point: POINTL;
            let Msg: UINT;
            let wParam: WPARAM;

            pDesk = (*pWnd).head.rpdesk;
            if ((*pDesk).dwDTFlags & DF_TME_HOVER) != 0 && pWnd == (*pDesk).spwndTrack {
                Point = (*gpsi_per_user()).ptCursor;
                if RECTL_bPointInRect(core::ptr::addr_of_mut!((*pDesk).rcMouseHover), Point.x, Point.y) != 0 {
                    if (*pDesk).htEx == HTCLIENT as u32 {
                        wParam = MsqGetDownKeyState((*head_pti(&(*pWnd).head)).MessageQueue as PUSER_MESSAGE_QUEUE);
                        Msg = WM_MOUSEHOVER as UINT;

                        if ((*pWnd).ExStyle & WS_EX_LAYOUTRTL) != 0 {
                            Point.x = (*pWnd).rcClient.right - Point.x - 1;
                        } else {
                            Point.x -= (*pWnd).rcClient.left;
                        }
                        Point.y -= (*pWnd).rcClient.top;
                    } else {
                        wParam = (*pDesk).htEx as WPARAM;
                        Msg = WM_NCMOUSEHOVER as UINT;
                    }
                    TRACE!(b"Generating WM_NCMOUSEHOVER\n\0".as_ptr());
                    UserPostMessage(hwnd, Msg, wParam, MAKELPARAM(Point.x as u32, Point.y as u32));
                    (*pDesk).dwDTFlags &= !DF_TME_HOVER;
                    // break; Kill this timer.
                    IntKillTimer(pWnd, idEvent, 1);
                    return;
                }
            }
            return;
        }
        x if x == ID_EVENT_SYSTIMER_FLASHWIN as UINT_PTR => {
            let mut fwi: FLASHWINFO = FLASHWINFO {
                cbSize: core::mem::size_of::<FLASHWINFO>() as UINT,
                hwnd: UserHMGetHandle(pWnd),
                dwFlags: FLASHW_SYSTIMER,
                uCount: 0,
                dwTimeout: 0,
            };

            IntFlashWindowEx(pWnd, core::ptr::addr_of_mut!(fwi));
            return;
        }
        _ => {
            ERR!("System Timer Proc invalid id %u!\n", idEvent);
        }
    }
    IntKillTimer(pWnd, idEvent, 1);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn StartTheTimers() {
    IntSetTimer(
        core::ptr::null_mut(),
        0,
        1000,
        Some(HungAppSysTimerProc),
        TMRF_RIT as INT,
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn StartTheTimers() {
    IntSetTimer(
        core::ptr::null_mut(),
        0,
        1000,
        Some(HungAppSysTimerProc),
        TMRF_RIT as INT,
    );
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn SystemTimerSet(
    Window: PWND,
    nIDEvent: UINT_PTR,
    uElapse: UINT,
    lpTimerFunc: TIMERPROC,
) -> UINT_PTR {
    SystemTimerSet_impl(Window, nIDEvent, uElapse, lpTimerFunc)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn SystemTimerSet(
    Window: PWND,
    nIDEvent: UINT_PTR,
    uElapse: UINT,
    lpTimerFunc: TIMERPROC,
) -> UINT_PTR {
    SystemTimerSet_impl(Window, nIDEvent, uElapse, lpTimerFunc)
}

unsafe fn SystemTimerSet_impl(
    Window: PWND,
    nIDEvent: UINT_PTR,
    uElapse: UINT,
    lpTimerFunc: TIMERPROC,
) -> UINT_PTR {
    if !Window.is_null()
        && (*pti_ethread(head_pti(&(*Window).head))).ThreadsProcess as *mut c_void != PsGetCurrentProcess() as *mut c_void
    {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }
    IntSetTimer(Window, nIDEvent, uElapse, lpTimerFunc, TMRF_SYSTEM as INT)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn PostTimerMessages(Window: PWND) -> BOOL {
    PostTimerMessages_impl(Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn PostTimerMessages(Window: PWND) -> BOOL {
    PostTimerMessages_impl(Window)
}

unsafe fn PostTimerMessages_impl(Window: PWND) -> BOOL {

    let mut pLE: PLIST_ENTRY;
    let mut Msg: MSG = core::mem::zeroed();
    let pti: PTHREADINFO;
    let mut Hit: BOOL = 0;
    let pTmr: PTIMER;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    TimerEnterExclusive();
    pLE = TimersListHead.Flink;
    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        let pTmr_cur = (pLE as *mut u8)
            .offset(-(core::mem::offset_of!(TIMER, ptmrList) as isize)) as PTIMER;

        if ((*pTmr_cur).flags & TMRF_READY) != 0
            && (*pTmr_cur).pti == pti
            && ((*pTmr_cur).pWnd == Window || Window.is_null())
        {
            Msg.hwnd = if !(*pTmr_cur).pWnd.is_null() {
                UserHMGetHandle((*pTmr_cur).pWnd)
            } else {
                core::ptr::null_mut()
            };
            Msg.message = if ((*pTmr_cur).flags & TMRF_SYSTEM) != 0 {
                WM_SYSTIMER
            } else {
                WM_TIMER
            };
            Msg.wParam = (*pTmr_cur).nID as WPARAM;
            Msg.lParam = pfn_as_lparam((*pTmr_cur).pfn);
            Msg.time = EngGetTickCount32();
            Msg.pt = core::mem::transmute_copy(&(*gpsi_per_user()).ptCursor);

            MsqPostMessage(
                pti,
                core::ptr::addr_of_mut!(Msg),
                0,
                (QS_POSTMESSAGE | QS_ALLPOSTMESSAGE) as DWORD,
                0,
                0,
            );
            (*pTmr_cur).flags &= !TMRF_READY;
            ClearMsgBitsMask(pti, QS_TIMER as UINT);
            Hit = 1;

            if pLE != core::ptr::addr_of_mut!(TimersListHead) {
                RemoveEntryList(core::ptr::addr_of_mut!((*pTmr_cur).ptmrList) as _);
                InsertTailList(core::ptr::addr_of_mut!(TimersListHead) as _, core::ptr::addr_of_mut!((*pTmr_cur).ptmrList) as _);
            }
            break;
        }

        pLE = (*pLE).Flink;
    }

    TimerLeave();

    Hit
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ProcessTimers() {
    ProcessTimers_impl();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ProcessTimers() {
    ProcessTimers_impl();
}

unsafe fn ProcessTimers_impl() {
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let Time: LONG;
    let mut pLE: PLIST_ENTRY;
    let mut pTmr: PTIMER;
    let mut TimerCount: LONG = 0;

    TimerEnterExclusive();
    pLE = TimersListHead.Flink;
    Time = EngGetTickCount32() as LONG;

    DueTime.QuadPart = -97656i64; // 1024hz .9765625 ms set to 10.0 ms

    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        pTmr = (pLE as *mut u8).offset(-(core::mem::offset_of!(TIMER, ptmrList) as isize)) as PTIMER;
        TimerCount += 1;
        if (*pTmr).flags & TMRF_WAITING as u32 != 0 {
            pLE = (*pTmr).ptmrList.Flink;
            pTmr = (pLE as *mut u8).offset(-(core::mem::offset_of!(TIMER, ptmrList) as isize)) as PTIMER;
            continue;
        }

        if (*pTmr).flags & TMRF_INIT as u32 != 0 {
            (*pTmr).flags &= !(TMRF_INIT as u32); // Skip this run.
        } else {
            if (*pTmr).cmsCountdown < 0 {
                if ((*pTmr).flags & TMRF_READY as u32 == 0)
                    && ((*(*pTmr).pti).TIF_flags & TIF_INCLEANUP as u32 == 0)
                {
                    if (*pTmr).flags & TMRF_ONESHOT as u32 != 0 {
                        (*pTmr).flags |= TMRF_WAITING as u32;
                    }

                    if (*pTmr).flags & TMRF_RIT as u32 != 0 {
                        // Hard coded call here, inside raw input thread.
                        ((*pTmr).pfn.unwrap())(
                            core::ptr::null_mut(),
                            WM_SYSTIMER as UINT,
                            (*pTmr).nID,
                            pTmr as DWORD,
                        );
                    } else {
                        (*pTmr).flags |= TMRF_READY as u32; // Set timer ready to be ran.
                        // Set thread message queue for this timer.
                        if !(*pTmr).pti.is_null() {
                            // Wakeup thread
                            (*(*pTmr).pti).cTimersReady += 1;
                            MsqWakeQueue((*pTmr).pti, QS_TIMER as u32, 1);
                        }
                    }
                }
                (*pTmr).cmsCountdown = (*pTmr).cmsRate;
            } else {
                (*pTmr).cmsCountdown -= Time - TimeLast;
            }
        }

        pLE = (*pLE).Flink;
    }

    // Restart the timer thread!
    KeSetTimer(MasterTimer as _, core::mem::transmute(DueTime), core::ptr::null_mut());

    TimeLast = Time;

    TimerLeave();
    TRACE!(b"TimerCount = %d\n\0".as_ptr(), TimerCount);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL {
    DestroyTimersForWindow_impl(pti, Window)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL {
    DestroyTimersForWindow_impl(pti, Window)
}

unsafe fn DestroyTimersForWindow_impl(pti: PTHREADINFO, Window: PWND) -> BOOL {
    let mut pLE: PLIST_ENTRY;
    let mut pTmr: PTIMER;
    let mut TimersRemoved: BOOL = 0;

    if Window.is_null() {
        return 0;
    }

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(TimerLock) as _, 1u8);

    pLE = TimersListHead.Flink;
    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        pTmr = CONTAINING_RECORD!(pLE, TIMER, ptmrList);
        pLE = (*pLE).Flink;
        if !pTmr.is_null() && (*pTmr).pti == pti && (*pTmr).pWnd == Window {
            TimersRemoved = RemoveTimer(pTmr);
        }
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(TimerLock) as _);
    KeLeaveCriticalRegion();

    TimersRemoved
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn DestroyTimersForThread(pti: PTHREADINFO) -> BOOL {
    DestroyTimersForThread_impl(pti)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn DestroyTimersForThread(pti: PTHREADINFO) -> BOOL {
    DestroyTimersForThread_impl(pti)
}

unsafe fn DestroyTimersForThread_impl(pti: PTHREADINFO) -> BOOL {
    let mut pLE: PLIST_ENTRY = TimersListHead.Flink;
    let mut pTmr: PTIMER;
    let mut TimersRemoved: BOOL = 0;

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(&raw mut TimerLock as *mut _ as _, 1);

    while pLE != core::ptr::addr_of_mut!(TimersListHead) {
        pTmr = CONTAINING_RECORD!(pLE, TIMER, ptmrList);
        pLE = (*pLE).Flink;
        if !pTmr.is_null() && (*pTmr).pti == pti {
            TimersRemoved = RemoveTimer(pTmr);
        }
    }

    ExReleaseResourceLite(&raw mut TimerLock as *mut _ as _);
    KeLeaveCriticalRegion();

    TimersRemoved
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntKillTimer(Window: PWND, IDEvent: UINT_PTR, SystemTimer: BOOL) -> BOOL {
    IntKillTimer_impl(Window, IDEvent, SystemTimer)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntKillTimer(Window: PWND, IDEvent: UINT_PTR, SystemTimer: BOOL) -> BOOL {
    IntKillTimer_impl(Window, IDEvent, SystemTimer)
}

unsafe fn IntKillTimer_impl(Window: PWND, IDEvent: UINT_PTR, SystemTimer: BOOL) -> BOOL {
    let mut pTmr: PTIMER = core::ptr::null_mut();

    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(core::ptr::addr_of_mut!(TimerLock) as _, 1u8);

    pTmr = FindTimer(Window, IDEvent, if SystemTimer != 0 { TMRF_SYSTEM as UINT } else { 0 });

    if !pTmr.is_null() {
        RemoveTimer(pTmr);
    }

    ExReleaseResourceLite(core::ptr::addr_of_mut!(TimerLock) as _);
    KeLeaveCriticalRegion();

    if !pTmr.is_null() { 1 } else { 0 }
}

#[no_mangle]
pub unsafe extern "system" fn InitTimerImpl() -> NTSTATUS {
    let BitmapBytes: ULONG;

    Mutex = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<FAST_MUTEX>() as SIZE_T,
        TAG_INTERNAL_SYNC,
    ) as PFAST_MUTEX;
    if Mutex.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    ExInitializeFastMutex(Mutex as _);

    BitmapBytes = (ALIGN_UP_BY(NUM_WINDOW_LESS_TIMERS as usize, core::mem::size_of::<ULONG>() * 8) / 8) as ULONG;
    WindowLessTimersBitMapBuffer = ExAllocatePoolWithTag(NonPagedPool, BitmapBytes as SIZE_T, TAG_TIMERBMP);
    if WindowLessTimersBitMapBuffer.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    RtlInitializeBitMap(
        core::ptr::addr_of_mut!(WindowLessTimersBitMap) as _,
        WindowLessTimersBitMapBuffer as PULONG,
        NUM_WINDOW_LESS_TIMERS,
    );

    RtlClearAllBits(core::ptr::addr_of_mut!(WindowLessTimersBitMap) as _);

    ExInitializeResourceLite(core::ptr::addr_of_mut!(TimerLock) as _);
    InitializeListHead(core::ptr::addr_of_mut!(TimersListHead) as _);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetTimer(
    hWnd: HWND,
    nIDEvent: UINT_PTR,
    uElapse: UINT,
    lpTimerFunc: TIMERPROC,
) -> UINT_PTR {
    let mut Window: PWND = core::ptr::null_mut();
    let ret: UINT_PTR;

    UserEnterExclusive();
    if !hWnd.is_null() {
        Window = UserGetWindowObject(hWnd);
    }

    ret = IntSetTimer(Window, nIDEvent, uElapse, lpTimerFunc, TMRF_TIFROMWND as INT);

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserKillTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL {
    let mut Window: PWND = core::ptr::null_mut();
    let ret: BOOL;

    UserEnterExclusive();
    if !hWnd.is_null() {
        Window = UserGetWindowObject(hWnd);
    }

    ret = IntKillTimer(Window, uIDEvent, 0);

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetSystemTimer(
    hWnd: HWND,
    nIDEvent: UINT_PTR,
    uElapse: UINT,
    lpTimerFunc: TIMERPROC,
) -> UINT_PTR {
    let ret: UINT_PTR;

    UserEnterExclusive();

    ret = IntSetTimer(
        UserGetWindowObject(hWnd),
        nIDEvent,
        uElapse,
        None,
        TMRF_SYSTEM as INT,
    );

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserValidateTimerCallback(lParam: LPARAM) -> BOOL {
    let mut Ret: BOOL = 0;

    UserEnterShared();

    Ret = ValidateTimerCallback(PsGetCurrentThreadWin32Thread() as PTHREADINFO, lParam);

    UserLeave();
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Window timers messages
//  * FILE:             win32ss/user/ntuser/timer.c
//  * PROGRAMER:        Gunnar
//  *                   Thomas Weidenmueller (w3seek@users.sourceforge.net)
//  *                   Michael Martin (michael.martin@reactos.org)
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserTimer);
//
// /* GLOBALS *******************************************************************/
//
// static LIST_ENTRY TimersListHead;
// static LONG TimeLast = 0;
//
// /* Windows 2000 has room for 32768 window-less timers */
// /* These values give timer IDs [256,32767], same as on Windows */
// #define MAX_WINDOW_LESS_TIMER_ID  (32768 - 1)
// #define NUM_WINDOW_LESS_TIMERS    (32768 - 256)
//
// #define HINTINDEX_BEGIN_VALUE   0
//
// static PFAST_MUTEX    Mutex;
// static RTL_BITMAP     WindowLessTimersBitMap;
// static PVOID          WindowLessTimersBitMapBuffer;
// static ULONG          HintIndex = HINTINDEX_BEGIN_VALUE;
//
// ERESOURCE TimerLock;
//
// #define IntLockWindowlessTimerBitmap() \
//   ExEnterCriticalRegionAndAcquireFastMutexUnsafe(Mutex)
//
// #define IntUnlockWindowlessTimerBitmap() \
//   ExReleaseFastMutexUnsafeAndLeaveCriticalRegion(Mutex)
//
// #define TimerEnterExclusive() \
// { \
//   KeEnterCriticalRegion(); \
//   ExAcquireResourceExclusiveLite(&TimerLock, TRUE); \
// }
//
// #define TimerLeave() \
// { \
//   ExReleaseResourceLite(&TimerLock); \
//   KeLeaveCriticalRegion(); \
// }
//
//
// /* FUNCTIONS *****************************************************************/
// static
// PTIMER
// FASTCALL
// CreateTimer(VOID)
// {
//   HANDLE Handle;
//   PTIMER Ret = NULL;
//
//   Ret = UserCreateObject(gHandleTable, NULL, NULL, &Handle, TYPE_TIMER, sizeof(TIMER));
//   if (Ret)
//   {
//      UserHMSetHandle(Ret, Handle);
//      InsertTailList(&TimersListHead, &Ret->ptmrList);
//   }
//
//   return Ret;
// }
//
// static
// BOOL
// FASTCALL
// RemoveTimer(PTIMER pTmr)
// {
//   BOOL Ret = FALSE;
//   if (pTmr)
//   {
//      /* Set the flag, it will be removed when ready */
//      RemoveEntryList(&pTmr->ptmrList);
//      if ((pTmr->pWnd == NULL) && (!(pTmr->flags & TMRF_SYSTEM))) // System timers are reusable.
//      {
//         ULONG ulBitmapIndex;
//
//         ASSERT(pTmr->nID <= MAX_WINDOW_LESS_TIMER_ID);
//         ulBitmapIndex = (ULONG)(MAX_WINDOW_LESS_TIMER_ID - pTmr->nID);
//         IntLockWindowlessTimerBitmap();
//         RtlClearBit(&WindowLessTimersBitMap, ulBitmapIndex);
//         IntUnlockWindowlessTimerBitmap();
//      }
//      UserDereferenceObject(pTmr);
//      Ret = UserDeleteObject( UserHMGetHandle(pTmr), TYPE_TIMER);
//   }
//   if (!Ret) ERR("Warning: Unable to delete timer\n");
//
//   return Ret;
// }
//
// PTIMER
// FASTCALL
// FindTimer(PWND Window,
//           UINT_PTR nID,
//           UINT flags)
// {
//   PLIST_ENTRY pLE;
//   PTIMER pTmr, RetTmr = NULL;
//
//   TimerEnterExclusive();
//   pLE = TimersListHead.Flink;
//   while (pLE != &TimersListHead)
//   {
//     pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//
//     if ( pTmr->nID == nID &&
//          pTmr->pWnd == Window &&
//         (pTmr->flags & (TMRF_SYSTEM|TMRF_RIT)) == (flags & (TMRF_SYSTEM|TMRF_RIT)))
//     {
//        RetTmr = pTmr;
//        break;
//     }
//
//     pLE = pLE->Flink;
//   }
//   TimerLeave();
//
//   return RetTmr;
// }
//
// PTIMER
// FASTCALL
// FindSystemTimer(PMSG pMsg)
// {
//   PLIST_ENTRY pLE;
//   PTIMER pTmr = NULL;
//
//   TimerEnterExclusive();
//   pLE = TimersListHead.Flink;
//   while (pLE != &TimersListHead)
//   {
//     pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//
//     if ( pMsg->lParam == (LPARAM)pTmr->pfn &&
//          (pTmr->flags & TMRF_SYSTEM) )
//        break;
//
//     pLE = pLE->Flink;
//   }
//   TimerLeave();
//
//   return pTmr;
// }
//
// BOOL
// FASTCALL
// ValidateTimerCallback(PTHREADINFO pti,
//                       LPARAM lParam)
// {
//   PLIST_ENTRY pLE;
//   BOOL Ret = FALSE;
//   PTIMER pTmr;
//
//   TimerEnterExclusive();
//   pLE = TimersListHead.Flink;
//   while (pLE != &TimersListHead)
//   {
//     pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//     if ( (lParam == (LPARAM)pTmr->pfn) &&
//         !(pTmr->flags & (TMRF_SYSTEM|TMRF_RIT)) &&
//          (pTmr->pti->ppi == pti->ppi) )
//     {
//        Ret = TRUE;
//        break;
//     }
//     pLE = pLE->Flink;
//   }
//   TimerLeave();
//
//   return Ret;
// }
//
// UINT_PTR FASTCALL
// IntSetTimer( PWND Window,
//                   UINT_PTR IDEvent,
//                   UINT Elapse,
//                   TIMERPROC TimerFunc,
//                   INT Type)
// {
//   PTIMER pTmr;
//   UINT_PTR Ret = IDEvent;
//   ULONG ulBitmapIndex;
//   LARGE_INTEGER DueTime;
//   DueTime.QuadPart = (LONGLONG)(-97656); // 1024hz .9765625 ms set to 10.0 ms
//
//   /* Windows XP SP2 and Windows Server 2003 behaviour */
//   if (Elapse > USER_TIMER_MAXIMUM)
//   {
//      TRACE("Adjusting uElapse\n");
//      Elapse = USER_TIMER_MAXIMUM;
//   }
//
//   /* Windows 2k/XP and Windows Server 2003 SP1 behaviour */
//   if (Elapse < USER_TIMER_MINIMUM)
//   {
//      TRACE("Adjusting uElapse\n");
//      Elapse = USER_TIMER_MINIMUM; // 1024hz .9765625 ms, set to 10.0 ms (+/-)1 ms
//   }
//
//   /* Passing an IDEvent of 0 and the SetTimer returns 1.
//      It will create the timer with an ID of 0 */
//   if ((Window) && (IDEvent == 0))
//      Ret = 1;
//
//   pTmr = FindTimer(Window, IDEvent, Type);
//
//   if ((!pTmr) && (Window == NULL) && (!(Type & TMRF_SYSTEM)))
//   {
//       IntLockWindowlessTimerBitmap();
//
//       ulBitmapIndex = RtlFindClearBitsAndSet(&WindowLessTimersBitMap, 1, HintIndex);
//       HintIndex = (ulBitmapIndex + 1) % NUM_WINDOW_LESS_TIMERS;
//       if (ulBitmapIndex == ULONG_MAX)
//       {
//          IntUnlockWindowlessTimerBitmap();
//          ERR("Unable to find a free window-less timer id\n");
//          EngSetLastError(ERROR_NO_SYSTEM_RESOURCES);
//          return 0;
//       }
//
//       ASSERT(ulBitmapIndex < NUM_WINDOW_LESS_TIMERS);
//       IDEvent = MAX_WINDOW_LESS_TIMER_ID - ulBitmapIndex;
//       Ret = IDEvent;
//
//       IntUnlockWindowlessTimerBitmap();
//   }
//
//   if (!pTmr)
//   {
//      pTmr = CreateTimer();
//      if (!pTmr) return 0;
//
//      if (Window && (Type & TMRF_TIFROMWND))
//         pTmr->pti = Window->head.pti->pEThread->Tcb.Win32Thread;
//      else
//      {
//         if (Type & TMRF_RIT)
//            pTmr->pti = ptiRawInput;
//         else
//            pTmr->pti = PsGetCurrentThreadWin32Thread();
//      }
//
//      pTmr->pWnd    = Window;
//      pTmr->cmsCountdown = Elapse;
//      pTmr->cmsRate = Elapse;
//      pTmr->pfn     = TimerFunc;
//      pTmr->nID     = IDEvent;
//      pTmr->flags   = Type|TMRF_INIT;
//   }
//   else
//   {
//      pTmr->cmsCountdown = Elapse;
//      pTmr->cmsRate = Elapse;
//   }
//
//   ASSERT(MasterTimer != NULL);
//   // Start the timer thread!
//   if (TimersListHead.Flink == TimersListHead.Blink) // There is only one timer
//      KeSetTimer(MasterTimer, DueTime, NULL);
//
//   return Ret;
// }
//
// //
// // Process win32k system timers.
// //
// VOID
// __stdcall
// SystemTimerProc(HWND hwnd,
//                 UINT uMsg,
//                 UINT_PTR idEvent,
//                 DWORD dwTime)
// {
//   PDESKTOP pDesk;
//   PWND pWnd = NULL;
//
//   if (hwnd)
//   {
//      pWnd = UserGetWindowObject(hwnd);
//      if (!pWnd)
//      {
//         ERR("System Timer Proc has invalid window handle! %p Id: %u\n", hwnd, idEvent);
//         return;
//      }
//   }
//   else
//   {
//      TRACE( "Windowless Timer Running!\n" );
//      return;
//   }
//
//   switch (idEvent)
//   {
// /*
//    Used in NtUserTrackMouseEvent.
//  */
//      case ID_EVENT_SYSTIMER_MOUSEHOVER:
//        {
//           POINT Point;
//           UINT Msg;
//           WPARAM wParam;
//
//           pDesk = pWnd->head.rpdesk;
//           if ( pDesk->dwDTFlags & DF_TME_HOVER &&
//                pWnd == pDesk->spwndTrack )
//           {
//              Point = gpsi->ptCursor;
//              if ( RECTL_bPointInRect(&pDesk->rcMouseHover, Point.x, Point.y) )
//              {
//                 if (pDesk->htEx == HTCLIENT) // In a client area.
//                 {
//                    wParam = MsqGetDownKeyState(pWnd->head.pti->MessageQueue);
//                    Msg = WM_MOUSEHOVER as UINT;
//
//                    if (pWnd->ExStyle & WS_EX_LAYOUTRTL)
//                    {
//                       Point.x = pWnd->rcClient.right - Point.x - 1;
//                    }
//                    else
//                       Point.x -= pWnd->rcClient.left;
//                    Point.y -= pWnd->rcClient.top;
//                 }
//                 else
//                 {
//                    wParam = pDesk->htEx; // Need to support all HTXYZ hits.
//                    Msg = WM_NCMOUSEHOVER as UINT;
//                 }
//                 TRACE("Generating WM_NCMOUSEHOVER\n");
//                 UserPostMessage(hwnd, Msg, wParam, MAKELPARAM(Point.x, Point.y));
//                 pDesk->dwDTFlags &= ~DF_TME_HOVER;
//                 break; // Kill this timer.
//              }
//           }
//        }
//        return; // Not this window so just return.
//
//      case ID_EVENT_SYSTIMER_FLASHWIN:
//        {
//           FLASHWINFO fwi =
//             {sizeof(FLASHWINFO),
//              UserHMGetHandle(pWnd),
//              FLASHW_SYSTIMER,0,0};
//
//           IntFlashWindowEx(pWnd, &fwi);
//        }
//        return;
//
//      default:
//        ERR("System Timer Proc invalid id %u!\n", idEvent);
//        break;
//   }
//   IntKillTimer(pWnd, idEvent, TRUE);
// }
//
// VOID
// FASTCALL
// StartTheTimers(VOID)
// {
//   // Need to start gdi syncro timers then start timer with Hang App proc
//   // that calles Idle process so the screen savers will know to run......
//   IntSetTimer(NULL, 0, 1000, HungAppSysTimerProc, TMRF_RIT);
// // Test Timers
// //  IntSetTimer(NULL, 0, 1000, SystemTimerProc, TMRF_RIT);
// }
//
// UINT_PTR
// FASTCALL
// SystemTimerSet( PWND Window,
//                 UINT_PTR nIDEvent,
//                 UINT uElapse,
//                 TIMERPROC lpTimerFunc)
// {
//   if (Window && Window->head.pti->pEThread->ThreadsProcess != PsGetCurrentProcess())
//   {
//      EngSetLastError(ERROR_ACCESS_DENIED);
//      TRACE("SysemTimerSet: Access Denied!\n");
//      return 0;
//   }
//   return IntSetTimer( Window, nIDEvent, uElapse, lpTimerFunc, TMRF_SYSTEM);
// }
//
// BOOL
// FASTCALL
// PostTimerMessages(PWND Window)
// {
//   PLIST_ENTRY pLE;
//   MSG Msg;
//   PTHREADINFO pti;
//   BOOL Hit = FALSE;
//   PTIMER pTmr;
//
//   pti = PsGetCurrentThreadWin32Thread();
//
//   TimerEnterExclusive();
//   pLE = TimersListHead.Flink;
//   while(pLE != &TimersListHead)
//   {
//      pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//      if ( (pTmr->flags & TMRF_READY) &&
//           (pTmr->pti == pti) &&
//           ((pTmr->pWnd == Window) || (Window == NULL)) )
//         {
//            Msg.hwnd    = (pTmr->pWnd ? UserHMGetHandle(pTmr->pWnd) : NULL);
//            Msg.message = (pTmr->flags & TMRF_SYSTEM) ? WM_SYSTIMER : WM_TIMER;
//            Msg.wParam  = (WPARAM) pTmr->nID;
//            Msg.lParam  = (LPARAM) pTmr->pfn;
//            Msg.time    = EngGetTickCount32();
//            // Fix all wine win:test_GetMessagePos WM_TIMER tests. See CORE-10867.
//            Msg.pt      = gpsi->ptCursor;
//
//            MsqPostMessage(pti, &Msg, FALSE, (QS_POSTMESSAGE|QS_ALLPOSTMESSAGE), 0, 0);
//            pTmr->flags &= ~TMRF_READY;
//            ClearMsgBitsMask(pti, QS_TIMER);
//            Hit = TRUE;
//            // Now move this entry to the end of the list so it will not be
//            // called again in the next msg loop.
//            if (pLE != &TimersListHead)
//            {
//               RemoveEntryList(&pTmr->ptmrList);
//               InsertTailList(&TimersListHead, &pTmr->ptmrList);
//            }
//            break;
//         }
//
//      pLE = pLE->Flink;
//   }
//
//   TimerLeave();
//
//   return Hit;
// }
//
// VOID
// FASTCALL
// ProcessTimers(VOID)
// {
//   LARGE_INTEGER DueTime;
//   LONG Time;
//   PLIST_ENTRY pLE;
//   PTIMER pTmr;
//   LONG TimerCount = 0;
//
//   TimerEnterExclusive();
//   pLE = TimersListHead.Flink;
//   Time = EngGetTickCount32() as LONG;
//
//   DueTime.QuadPart = (LONGLONG)(-97656); // 1024hz .9765625 ms set to 10.0 ms
//
//   while(pLE != &TimersListHead)
//   {
//     pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//     TimerCount++;
//     if (pTmr->flags & TMRF_WAITING)
//     {
//        pLE = pTmr->ptmrList.Flink;
//        pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//        continue;
//     }
//
//     if (pTmr->flags & TMRF_INIT)
//     {
//        pTmr->flags &= ~TMRF_INIT; // Skip this run.
//     }
//     else
//     {
//        if (pTmr->cmsCountdown < 0)
//        {
//           ASSERT(pTmr->pti);
//           if ((!(pTmr->flags & TMRF_READY)) && (!(pTmr->pti->TIF_flags & TIF_INCLEANUP)))
//           {
//              if (pTmr->flags & TMRF_ONESHOT)
//                 pTmr->flags |= TMRF_WAITING;
//
//              if (pTmr->flags & TMRF_RIT)
//              {
//                 // Hard coded call here, inside raw input thread.
//                 pTmr->pfn(NULL, WM_SYSTIMER, pTmr->nID, (LPARAM)pTmr);
//              }
//              else
//              {
//                 pTmr->flags |= TMRF_READY; // Set timer ready to be ran.
//                 // Set thread message queue for this timer.
//                 if (pTmr->pti)
//                 {  // Wakeup thread
//                    pTmr->pti->cTimersReady++;
//                    ASSERT(pTmr->pti->pEventQueueServer != NULL);
//                    MsqWakeQueue(pTmr->pti, QS_TIMER, TRUE);
//                 }
//              }
//           }
//           pTmr->cmsCountdown = pTmr->cmsRate;
//        }
//        else
//           pTmr->cmsCountdown -= Time - TimeLast;
//     }
//
//     pLE = pLE->Flink;
//   }
//
//   // Restart the timer thread!
//   ASSERT(MasterTimer != NULL);
//   KeSetTimer(MasterTimer, DueTime, NULL);
//
//   TimeLast = Time;
//
//   TimerLeave();
//   TRACE("TimerCount = %d\n", TimerCount);
// }
//
// BOOL FASTCALL
// DestroyTimersForWindow(PTHREADINFO pti, PWND Window)
// {
//    PLIST_ENTRY pLE;
//    PTIMER pTmr;
//    BOOL TimersRemoved = FALSE;
//
//    if (Window == NULL)
//       return FALSE;
//
//    TimerEnterExclusive();
//    pLE = TimersListHead.Flink;
//    while(pLE != &TimersListHead)
//    {
//       pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//       pLE = pLE->Flink; /* get next timer list entry before current timer is removed */
//       if ((pTmr) && (pTmr->pti == pti) && (pTmr->pWnd == Window))
//       {
//          TimersRemoved = RemoveTimer(pTmr);
//       }
//    }
//
//    TimerLeave();
//
//    return TimersRemoved;
// }
//
// BOOL FASTCALL
// DestroyTimersForThread(PTHREADINFO pti)
// {
//    PLIST_ENTRY pLE = TimersListHead.Flink;
//    PTIMER pTmr;
//    BOOL TimersRemoved = FALSE;
//
//    TimerEnterExclusive();
//
//    while(pLE != &TimersListHead)
//    {
//       pTmr = CONTAINING_RECORD(pLE, TIMER, ptmrList);
//       pLE = pLE->Flink; /* get next timer list entry before current timer is removed */
//       if ((pTmr) && (pTmr->pti == pti))
//       {
//          TimersRemoved = RemoveTimer(pTmr);
//       }
//    }
//
//    TimerLeave();
//
//    return TimersRemoved;
// }
//
// BOOL FASTCALL
// IntKillTimer(PWND Window, UINT_PTR IDEvent, BOOL SystemTimer)
// {
//    PTIMER pTmr = NULL;
//    TRACE("IntKillTimer Window %p id %uI systemtimer %s\n",
//          Window, IDEvent, SystemTimer ? "TRUE" : "FALSE");
//
//    TimerEnterExclusive();
//    pTmr = FindTimer(Window, IDEvent, SystemTimer ? TMRF_SYSTEM : 0);
//
//    if (pTmr)
//    {
//       RemoveTimer(pTmr);
//    }
//    TimerLeave();
//
//    return pTmr ? TRUE :  FALSE;
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// InitTimerImpl(VOID)
// {
//    ULONG BitmapBytes;
//
//    /* Allocate FAST_MUTEX from non paged pool */
//    Mutex = ExAllocatePoolWithTag(NonPagedPool, sizeof(FAST_MUTEX), TAG_INTERNAL_SYNC);
//    if (!Mutex)
//    {
//        return STATUS_INSUFFICIENT_RESOURCES;
//    }
//
//    ExInitializeFastMutex(Mutex as _);
//
//    BitmapBytes = ALIGN_UP_BY(NUM_WINDOW_LESS_TIMERS, sizeof(ULONG) * 8) / 8;
//    WindowLessTimersBitMapBuffer = ExAllocatePoolWithTag(NonPagedPool, BitmapBytes, TAG_TIMERBMP);
//    if (WindowLessTimersBitMapBuffer == NULL)
//    {
//       return STATUS_UNSUCCESSFUL;
//    }
//
//    RtlInitializeBitMap(&WindowLessTimersBitMap,
//                        WindowLessTimersBitMapBuffer,
//                        NUM_WINDOW_LESS_TIMERS);
//
//    /* Yes we need this, since ExAllocatePoolWithTag isn't supposed to zero out allocated memory */
//    RtlClearAllBits(&WindowLessTimersBitMap);
//
//    ExInitializeResourceLite(&TimerLock);
//    InitializeListHead(&TimersListHead);
//
//    return STATUS_SUCCESS;
// }
//
// UINT_PTR
// __stdcall
// NtUserSetTimer
// (
//    HWND hWnd,
//    UINT_PTR nIDEvent,
//    UINT uElapse,
//    TIMERPROC lpTimerFunc
// )
// {
//    PWND Window = NULL;
//    UINT_PTR ret;
//
//    TRACE("Enter NtUserSetTimer\n");
//    UserEnterExclusive();
//    if (hWnd) Window = UserGetWindowObject(hWnd);
//
//    ret = IntSetTimer(Window, nIDEvent, uElapse, lpTimerFunc, TMRF_TIFROMWND);
//
//    UserLeave();
//    TRACE("Leave NtUserSetTimer, ret=%u\n", ret);
//
//    return ret;
// }
//
//
// BOOL
// __stdcall
// NtUserKillTimer
// (
//    HWND hWnd,
//    UINT_PTR uIDEvent
// )
// {
//    PWND Window = NULL;
//    BOOL ret;
//
//    TRACE("Enter NtUserKillTimer\n");
//    UserEnterExclusive();
//    if (hWnd) Window = UserGetWindowObject(hWnd);
//
//    ret = IntKillTimer(Window, uIDEvent, FALSE);
//
//    UserLeave();
//
//    TRACE("Leave NtUserKillTimer, ret=%i\n", ret);
//    return ret;
// }
//
//
// UINT_PTR
// __stdcall
// NtUserSetSystemTimer(
//    HWND hWnd,
//    UINT_PTR nIDEvent,
//    UINT uElapse,
//    TIMERPROC lpTimerFunc
// )
// {
//     UINT_PTR ret;
//
//     UserEnterExclusive();
//     TRACE("Enter NtUserSetSystemTimer\n");
//
//     ret = IntSetTimer(UserGetWindowObject(hWnd), nIDEvent, uElapse, NULL, TMRF_SYSTEM);
//
//     UserLeave();
//
//     TRACE("Leave NtUserSetSystemTimer, ret=%u\n", ret);
//     return ret;
// }
//
// BOOL
// __stdcall
// NtUserValidateTimerCallback(
//     LPARAM lParam)
// {
//   BOOL Ret = FALSE;
//
//   UserEnterShared();
//
//   Ret = ValidateTimerCallback(PsGetCurrentThreadWin32Thread(), lParam);
//
//   UserLeave();
//   return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
