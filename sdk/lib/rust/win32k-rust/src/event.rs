use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion, PsGetCurrentThreadId, PsGetCurrentThreadWin32Thread, PsGetThreadWin32Thread, PsLookupThreadByThreadId};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};

// tags.h: TAG_HOOK 'ohsU' (C multichar is big-endian → reversed bytes).
const TAG_HOOK: u32 = u32::from_le_bytes(*b"Usho");
const INVALID_HANDLE_VALUE: isize = -1; // (HANDLE)-1

// FASTCALL win32k internals not present in the win32k_sys bindgen glob.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn UserReferenceObject(obj: PVOID);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn UserReferenceObject(obj: PVOID);
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *mut MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR,) -> BOOL;
}

// PtrToUint(p) = (UINT)(ULONG_PTR)(p)
#[inline(always)]
unsafe fn PtrToUint<T>(p: *mut T) -> UINT {
    p as usize as UINT
}

// UserHMGetHandle(obj) = ((obj)->head.h); head is at object offset 0, h is its
// first field, so a reinterpret read of offset 0 is exact (see ntuser.h:230).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    *(obj as *const HANDLE)
}
#[inline(always)]
unsafe fn UserHMSetHandle<T>(obj: *mut T, handle: HANDLE) {
    *(obj as *mut HANDLE) = handle;
}

// UserRefObjectCo / UserDerefObjectCo are `static __inline` in object.h —
// reimplemented here (PushEntryList / PopEntryList inlined).
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

// LIST_ENTRY inline ops (win32k_sys PLIST_ENTRY — same pattern as gdipool.rs).
#[inline(always)]
unsafe fn InitializeListHead(h: PLIST_ENTRY) {
    (*h).Flink = h;
    (*h).Blink = h;
}
#[inline(always)]
unsafe fn InsertTailList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let b = (*h).Blink;
    (*e).Flink = h;
    (*e).Blink = b;
    (*b).Flink = e;
    (*h).Blink = e;
}
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) -> u8 {
    let f = (*e).Flink;
    let b = (*e).Blink;
    (*b).Flink = f;
    (*f).Blink = b;
    (f == b) as u8
}

// eng.h: EngGetTickCount32() = (ULONG)EngGetTickCount().
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG {
    EngGetTickCount() as ULONG
}

// THRDESKHEAD pti is at object offset 2 pointer-widths (bindgen-flattened head).
#[inline(always)]
unsafe fn head_pti<T>(obj: *mut T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// W32THREAD.pEThread is the first field of THREADINFO (offset 0).
#[inline(always)]
unsafe fn pti_ethread(pti: PTHREADINFO) -> PETHREAD {
    *(pti as *const PETHREAD)
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _EVENTPACK {
    pub pEH: PEVENTHOOK,
    pub idObject: LONG,
    pub idChild: LONG,
    pub idThread: LONG,
}
pub type EVENTPACK = _EVENTPACK;
pub type PEVENTPACK = *mut _EVENTPACK;

pub static mut GlobalEvents: PEVENTTABLE = core::ptr::null_mut();

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn GetMaskFromEvent(Event: DWORD) -> DWORD {
    GetMaskFromEvent_impl(Event)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn GetMaskFromEvent(Event: DWORD) -> DWORD {
    GetMaskFromEvent_impl(Event)
}

unsafe fn GetMaskFromEvent_impl(Event: DWORD) -> DWORD {
    let mut Ret: DWORD = 0;

    if Event > EVENT_OBJECT_STATECHANGE {
        if Event == EVENT_OBJECT_LOCATIONCHANGE {
            return SRV_EVENT_LOCATIONCHANGE;
        }
        if Event == EVENT_OBJECT_NAMECHANGE {
            return SRV_EVENT_NAMECHANGE;
        }
        if Event == EVENT_OBJECT_VALUECHANGE {
            return SRV_EVENT_VALUECHANGE;
        }
        return SRV_EVENT_CREATE;
    }

    if Event == EVENT_OBJECT_STATECHANGE {
        return SRV_EVENT_STATECHANGE;
    }

    Ret = SRV_EVENT_RUNNING;

    if Event < EVENT_SYSTEM_MENUSTART {
        return SRV_EVENT_CREATE;
    }

    if Event <= EVENT_SYSTEM_MENUPOPUPEND {
        Ret = SRV_EVENT_MENU;
    } else {
        if Event <= EVENT_CONSOLE_CARET - 1 {
            return SRV_EVENT_CREATE;
        }
        if Event <= EVENT_CONSOLE_END_APPLICATION {
            return SRV_EVENT_END_APPLICATION;
        }
        if Event != EVENT_OBJECT_FOCUS {
            return SRV_EVENT_CREATE;
        }
    }
    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSetSrvEventMask(EventMin: UINT, EventMax: UINT) {
    IntSetSrvEventMask_impl(EventMin, EventMax)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetSrvEventMask(EventMin: UINT, EventMax: UINT) {
    IntSetSrvEventMask_impl(EventMin, EventMax)
}

unsafe fn IntSetSrvEventMask_impl(EventMin: UINT, EventMax: UINT) {
    let mut event: UINT = EventMin;
    while event <= EventMax {
        if (event >= EVENT_SYSTEM_SOUND && event <= EVENT_SYSTEM_MINIMIZEEND)
            || (event >= EVENT_CONSOLE_CARET && event <= EVENT_CONSOLE_END_APPLICATION)
            || (event >= EVENT_OBJECT_CREATE && event <= EVENT_OBJECT_ACCELERATORCHANGE)
        {
            (*gpsi).dwInstalledEventHooks |= GetMaskFromEvent(event);
        }
        if event > EVENT_SYSTEM_MINIMIZEEND && event < EVENT_CONSOLE_CARET {
            event = EVENT_CONSOLE_CARET - 1;
            (*gpsi).dwInstalledEventHooks |= GetMaskFromEvent(event);
        }
        if event > EVENT_CONSOLE_END_APPLICATION && event < EVENT_OBJECT_CREATE {
            event = EVENT_OBJECT_CREATE - 1;
            (*gpsi).dwInstalledEventHooks |= GetMaskFromEvent(event);
        }
        if event > EVENT_OBJECT_ACCELERATORCHANGE && event < EVENT_MAX {
            event = EVENT_MAX - 1;
            (*gpsi).dwInstalledEventHooks |= GetMaskFromEvent(event);
            break;
        }
        event += 1;
    }
    if (*gpsi).dwInstalledEventHooks == 0 {
        (*gpsi).dwInstalledEventHooks |= SRV_EVENT_RUNNING;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCallLowLevelEvent(
    pEH: PEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
    idThread: LONG,
) -> LRESULT {
    IntCallLowLevelEvent_impl(pEH, event, hwnd, idObject, idChild, idThread)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCallLowLevelEvent(
    pEH: PEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
    idThread: LONG,
) -> LRESULT {
    IntCallLowLevelEvent_impl(pEH, event, hwnd, idObject, idChild, idThread)
}

unsafe fn IntCallLowLevelEvent_impl(
    pEH: PEVENTHOOK,
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
    idThread: LONG,
) -> LRESULT {
    let pEP = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<EVENTPACK>() as SIZE_T,
        TAG_HOOK,
    ) as PEVENTPACK;
    if pEP.is_null() {
        return 0;
    }

    (*pEP).pEH = pEH;
    (*pEP).idObject = idObject;
    (*pEP).idChild = idChild;
    (*pEP).idThread = idThread;

    let mut Msg: MSG = core::mem::zeroed();
    Msg.message = event;
    Msg.hwnd = hwnd;
    Msg.wParam = 0;
    Msg.lParam = POSTEVENT_NWE as _;
    Msg.time = 0;

    MsqPostMessage(
        (*pEH).head.pti,
        core::ptr::addr_of_mut!(Msg),
        0u8,
        QS_EVENT,
        POSTEVENT_NWE,
        pEP as LONG_PTR,
    );
    0
}

// IntRemoveEvent is plain cdecl (nm: `_IntRemoveEvent`), referenced by another TU.
#[no_mangle]
pub unsafe extern "C" fn IntRemoveEvent(Object: PVOID) -> BOOLEAN {
    let pEH: PEVENTHOOK = Object as PEVENTHOOK;
    if !pEH.is_null() {
        KeEnterCriticalRegion();
        RemoveEntryList(core::ptr::addr_of_mut!((*pEH).Chain));
        (*GlobalEvents).Counts -= 1;
        if (*GlobalEvents).Counts == 0 {
            (*gpsi).dwInstalledEventHooks = 0;
        }
        UserDeleteObject(UserHMGetHandle(pEH), TYPE_WINEVENTHOOK);
        KeLeaveCriticalRegion();
        return 1u8;
    }
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn co_EVENT_CallEvents(
    event: DWORD,
    hwnd: HWND,
    idObject: UINT_PTR,
    idChild: LONG_PTR,
) -> LRESULT {
    let pEH: PEVENTHOOK;
    let Result: LRESULT;
    let pEP: PEVENTPACK = idChild as PEVENTPACK;

    pEH = (*pEP).pEH;
    Result = co_IntCallEventProc(
        UserHMGetHandle(pEH) as HWINEVENTHOOK,
        event,
        hwnd,
        (*pEP).idObject,
        (*pEP).idChild,
        (*pEP).idThread as DWORD,
        EngGetTickCount32(),
        (*pEH).Proc,
        (*pEH).ihmod as INT,
        (*pEH).offPfn,
    );

    ExFreePoolWithTag(pEP as *mut core::ffi::c_void, TAG_HOOK);
    Result
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntNotifyWinEvent(
    Event: DWORD,
    pWnd: PWND,
    idObject: LONG,
    idChild: LONG,
    flags: DWORD,
) {
    IntNotifyWinEvent_impl(Event, pWnd, idObject, idChild, flags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntNotifyWinEvent(
    Event: DWORD,
    pWnd: PWND,
    idObject: LONG,
    idChild: LONG,
    flags: DWORD,
) {
    IntNotifyWinEvent_impl(Event, pWnd, idObject, idChild, flags)
}

unsafe fn IntNotifyWinEvent_impl(
    Event: DWORD,
    pWnd: PWND,
    idObject: LONG,
    idChild: LONG,
    flags: DWORD,
) {
    let mut pEH: PEVENTHOOK;
    let mut ListEntry: PLIST_ENTRY;
    let pti: PTHREADINFO;
    let ptiCurrent: PTHREADINFO;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if GlobalEvents.is_null() || (*GlobalEvents).Counts == 0 {
        return;
    }

    if !pWnd.is_null() && ((*pWnd).state & WNDS_DESTROYED) != 0 {
        return;
    }

    ptiCurrent = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !pWnd.is_null() && (flags & WEF_SETBYWNDPTI) != 0 {
        pti = head_pti(pWnd);
    } else {
        pti = ptiCurrent;
    }

    let EventsHead = core::ptr::addr_of_mut!((*GlobalEvents).Events);
    ListEntry = (*EventsHead).Flink;

    while ListEntry != EventsHead {
        let pEH_local: PEVENTHOOK = {
            let offset = core::mem::offset_of!(EVENTHOOK, Chain);
            (ListEntry as *mut u8).sub(offset) as PEVENTHOOK
        };
        let _ = pEH_local;
        pEH = pEH_local;
        ListEntry = (*ListEntry).Flink;

        if Event >= (*pEH).eventMin && Event <= (*pEH).eventMax {
            if !(((*pEH).idProcess != 0
                && (*pEH).idProcess != PtrToUint((*pti_ethread(pti)).Cid.UniqueProcess))
                || ((*pEH).Flags & WINEVENT_SKIPOWNPROCESS != 0
                    && (*head_pti(pEH)).ppi == (*pti).ppi)
                || ((*pEH).idThread != 0
                    && (*pEH).idThread != PtrToUint((*pti_ethread(pti)).Cid.UniqueThread))
                || ((*pEH).Flags & WINEVENT_SKIPOWNTHREAD != 0
                    && head_pti(pEH) == pti)
                || (*head_pti(pEH)).rpdesk != (*ptiCurrent).rpdesk)
            {
                UserRefObjectCo(pEH as *mut _, core::ptr::addr_of_mut!(Ref));
                if (*pEH).Flags & WINEVENT_INCONTEXT != 0 {
                    co_IntCallEventProc(
                        UserHMGetHandle(pEH as *mut _) as HWINEVENTHOOK,
                        Event,
                        if !pWnd.is_null() {
                            UserHMGetHandle(pWnd as *mut _) as HWND
                        } else {
                            core::ptr::null_mut()
                        },
                        idObject,
                        idChild,
                        PtrToUint(PsGetCurrentThreadId()),
                        EngGetTickCount32(),
                        (*pEH).Proc,
                        (*pEH).ihmod as INT,
                        (*pEH).offPfn,
                    );
                } else {
                    IntCallLowLevelEvent(
                        pEH,
                        Event,
                        if !pWnd.is_null() {
                            UserHMGetHandle(pWnd as *mut _) as HWND
                        } else {
                            core::ptr::null_mut()
                        },
                        idObject,
                        idChild,
                        PtrToUint(PsGetCurrentThreadId()) as LONG,
                    );
                }
                UserDerefObjectCo(pEH as *mut _);
            }
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserNotifyWinEvent(
    Event: DWORD,
    hWnd: HWND,
    idObject: LONG,
    idChild: LONG,
) {
    let mut Window: PWND = core::ptr::null_mut();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    UserEnterExclusive();

    if !hWnd.is_null() && hWnd != INVALID_HANDLE_VALUE as HWND {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            UserLeave();
            return;
        }
    }

    if (*gpsi).dwInstalledEventHooks & GetMaskFromEvent(Event) != 0 {
        if !Window.is_null() {
            UserRefObjectCo(Window, &mut Ref);
        }
        IntNotifyWinEvent(Event, Window, idObject, idChild, WEF_SETBYWNDPTI as DWORD);
        if !Window.is_null() {
            UserDerefObjectCo(Window);
        }
    }
    UserLeave();
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWinEventHook(
    eventMin: UINT,
    eventMax: UINT,
    mut hmodWinEventProc: HMODULE,
    puString: PUNICODE_STRING,
    lpfnWinEventProc: WINEVENTPROC,
    idProcess: DWORD,
    idThread: DWORD,
    dwflags: UINT,
) -> HWINEVENTHOOK {
    let pEH: PEVENTHOOK;
    let mut Ret: HWINEVENTHOOK = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let pti: PTHREADINFO;

    UserEnterExclusive();

    'block: {
        if GlobalEvents.is_null() {
            GlobalEvents = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<EVENTTABLE>() as SIZE_T,
                TAG_HOOK,
            ) as PEVENTTABLE;
            if GlobalEvents.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'block;
            }
            (*GlobalEvents).Counts = 0;
            InitializeListHead(core::ptr::addr_of_mut!((*GlobalEvents).Events));
        }

        if eventMin > eventMax {
            EngSetLastError(ERROR_INVALID_HOOK_FILTER);
            break 'block;
        }

        if lpfnWinEventProc.is_none() {
            EngSetLastError(ERROR_INVALID_FILTER_PROC);
            break 'block;
        }

        if (dwflags & WINEVENT_INCONTEXT) != 0 {
            if hmodWinEventProc.is_null() {
                EngSetLastError(ERROR_HOOK_NEEDS_HMOD);
                break 'block;
            }
            if puString.is_null() {
                EngSetLastError(ERROR_DLL_NOT_FOUND);
                break 'block;
            }
        } else {
            hmodWinEventProc = core::ptr::null_mut();
        }

        if idThread != 0 {
            let mut Thread: PETHREAD = core::ptr::null_mut();
            Status = PsLookupThreadByThreadId(
                idThread as HANDLE,
                core::ptr::addr_of_mut!(Thread) as _,
            );
            if !NT_SUCCESS(Status) {
                EngSetLastError(ERROR_INVALID_THREAD_ID);
                break 'block;
            }
            pti = PsGetThreadWin32Thread(Thread as _) as PTHREADINFO;
            ObDereferenceObject(Thread as *mut _);
        } else {
            pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        }

        pEH = UserCreateObject(
            gHandleTable,
            core::ptr::null_mut(),
            pti as *mut _,
            core::ptr::addr_of_mut!(Handle),
            TYPE_WINEVENTHOOK as u32,
            core::mem::size_of::<EVENTHOOK>() as u32,
        ) as PEVENTHOOK;

        if !pEH.is_null() {
            InsertTailList(
                core::ptr::addr_of_mut!((*GlobalEvents).Events),
                core::ptr::addr_of_mut!((*pEH).Chain),
            );
            (*GlobalEvents).Counts += 1;

            UserHMSetHandle(pEH as *mut _, Handle);
            (*pEH).eventMin = eventMin;
            (*pEH).eventMax = eventMax;
            (*pEH).idProcess = idProcess;
            (*pEH).idThread = idThread;
            (*pEH).Flags = dwflags;

            if !hmodWinEventProc.is_null() {
                (*pEH).offPfn = (lpfnWinEventProc
                    .map(|f| f as usize)
                    .unwrap_or(0)
                    .wrapping_sub(hmodWinEventProc as usize))
                    as ULONG_PTR;
                (*pEH).ihmod = hmodWinEventProc as INT_PTR;
                (*pEH).Proc = lpfnWinEventProc;
            } else {
                (*pEH).Proc = lpfnWinEventProc;
                (*pEH).offPfn = 0;
                (*pEH).ihmod = hmodWinEventProc as INT_PTR;
            }

            UserDereferenceObject(pEH as *mut _);

            Ret = Handle as HWINEVENTHOOK;
            IntSetSrvEventMask(eventMin, eventMax);
        }
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnhookWinEvent(hWinEventHook: HWINEVENTHOOK) -> BOOL {
    let pEH: PEVENTHOOK;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    pEH = UserGetObject(gHandleTable, hWinEventHook as PVOID, TYPE_WINEVENTHOOK) as PEVENTHOOK;
    if !pEH.is_null() {
        Ret = IntRemoveEvent(pEH as PVOID) as BOOL;
    }

    UserLeave();
    Ret
}
