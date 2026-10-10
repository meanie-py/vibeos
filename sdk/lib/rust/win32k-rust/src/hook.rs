#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeStackAttachProcess, KeUnstackDetachProcess, PsGetCurrentProcessWin32Process, PsGetCurrentThreadWin32Thread, PsGetProcessWin32Process, PsIsSystemProcess, RtlInitUnicodeString, RtlNtStatusToDosError, ExAllocatePoolWithTag, ExFreePoolWithTag};
// NtCurrentTeb: FORCEINLINE per-arch TIB read (no kernel symbol) — use the intrinsic.
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ob::{ObReferenceObject, ObDereferenceObject};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

const UNICODE_NULL: u16 = 0;

// WH_* / HCBT_* bindgen-typed inconsistently (some i32, some u32). i32 aliases for
// use as match-arm patterns against an INT scrutinee (faithful — C treats all as int).
const WHI_MSGFILTER: INT = WH_MSGFILTER as INT;
const WHI_SYSMSGFILTER: INT = WH_SYSMSGFILTER as INT;
const WHI_GETMESSAGE: INT = WH_GETMESSAGE as INT;
const WHI_JOURNALPLAYBACK: INT = WH_JOURNALPLAYBACK as INT;
const WHI_JOURNALRECORD: INT = WH_JOURNALRECORD as INT;
const WHI_MOUSE_LL: INT = WH_MOUSE_LL as INT;
const WHI_KEYBOARD_LL: INT = WH_KEYBOARD_LL as INT;
const WHI_CBT: INT = WH_CBT as INT;
const WHI_DEBUG: INT = WH_DEBUG as INT;
const WHI_FOREGROUNDIDLE: INT = WH_FOREGROUNDIDLE as INT;
const WHI_KEYBOARD: INT = WH_KEYBOARD as INT;
const WHI_SHELL: INT = WH_SHELL as INT;
const WHI_MOUSE: INT = WH_MOUSE as INT;
const HCBTI_CLICKSKIPPED: INT = HCBT_CLICKSKIPPED as INT;
const HCBTI_MOVESIZE: INT = HCBT_MOVESIZE as INT;
const HCBTI_ACTIVATE: INT = HCBT_ACTIVATE as INT;
const HCBTI_CREATEWND: INT = HCBT_CREATEWND as INT;

// Pool tag TAG_HOOK = 'ohsU' (tags.h) → from_le_bytes reverses the C 4-char literal.
const TAG_HOOK: u32 = u32::from_le_bytes(*b"Usho");
// USERTAG_WINDOWLIST = 'lwsU' (tags.h).
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");

// HOOKID_TO_INDEX(id) = id - WH_MINHOOK ; HOOKID_TO_FLAG(id) = 1 << (id + 1) (hook.h).
#[inline(always)]
fn HOOKID_TO_INDEX(id: INT) -> INT { id - WH_MINHOOK as INT }
#[inline(always)]
fn HOOKID_TO_FLAG(id: INT) -> u32 { 1u32 << (id + 1) }
// ISITHOOKED(id) = current pti->fsHooks & HOOKID_TO_FLAG(id) (hook.h).
#[inline(always)]
unsafe fn ISITHOOKED(id: INT) -> u32 {
    (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).fsHooks & HOOKID_TO_FLAG(id)
}

// IS_ATOM!(x): (0 < ULONG_PTR(x) < 0x10000). Yields BOOL (i32).
macro_rules! IS_ATOM {
    ($x:expr) => {{ let v = ($x) as usize; ((v > 0) && (v < 0x10000)) as BOOL }};
}

// IntToPtr(i): wParam-sized handle pack of an INT (basetsd.h).
#[inline(always)]
fn IntToPtr(i: INT) -> WPARAM { i as isize as WPARAM }
// PtrToUint(p): truncate a pointer to UINT (basetsd.h).
#[inline(always)]
unsafe fn PtrToUint<T>(p: *mut T) -> UINT { p as usize as UINT }
// UlongToHandle(u): ULONG → HANDLE (basetsd.h).
#[inline(always)]
fn UlongToHandle(u: DWORD) -> HANDLE { u as usize as HANDLE }

// LIST_ENTRY inline helpers (rtl).
#[inline(always)]
unsafe fn IsListEmpty(h: PLIST_ENTRY) -> u8 { ((*h).Flink == h) as u8 }
#[inline(always)]
unsafe fn InsertHeadList(h: PLIST_ENTRY, e: PLIST_ENTRY) {
    let f = (*h).Flink; (*e).Flink = f; (*e).Blink = h; (*f).Blink = e; (*h).Flink = e;
}
#[inline(always)]
unsafe fn RemoveEntryList(e: PLIST_ENTRY) {
    let f = (*e).Flink; let b = (*e).Blink; (*b).Flink = f; (*f).Blink = b;
}

// THRDESKHEAD.pti is at object offset 2 pointer-widths (bindgen-flattened head).
// Matches input.rs/event.rs/timer.rs head_pti.
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    *((head as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
// W32THREAD.pEThread is the first field of THREADINFO (offset 0). Matches event.rs.
#[inline(always)]
unsafe fn pti_ethread(pti: PTHREADINFO) -> PETHREAD {
    *(pti as *const PETHREAD)
}
// W32PROCESS.peProcess is the first field of PROCESSINFO (offset 0). Matches windc.rs.
#[inline(always)]
unsafe fn ppi_eprocess(ppi: PPROCESSINFO) -> PEPROCESS {
    *(ppi as *const PEPROCESS)
}
// PROCESSINFO->W32PF_flags follows {PEPROCESS peProcess; DWORD RefCount}, so it sits
// at size_of::<usize>()+size_of::<u32>() (8 i386 / 12 amd64-arm64). Matches windc.rs/input.rs.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    (ppi as *mut u8).add(core::mem::size_of::<usize>() + core::mem::size_of::<u32>()) as *mut DWORD
}

// UserHMGetHandle(obj) = obj->head.h (object.h macro). Matches input.rs.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    (*(obj as *const _HEAD)).h
}

// RtlCopyMemory — core memcpy.
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}

// ReleaseCapturedUnicodeString (probe.h static __inline): free the captured buffer.
// Matches desktop.rs.
#[inline(always)]
unsafe fn ReleaseCapturedUnicodeString(CapturedString: PUNICODE_STRING, CurrentMode: KPROCESSOR_MODE) {
    if CurrentMode != 0 && !(*CapturedString).Buffer.is_null() {
        ExFreePoolWithTag((*CapturedString).Buffer as *mut c_void, u32::from_le_bytes(*b"RTSU"));
    }
}

// ProbeAndCaptureUnicodeString (probe.h static __inline) — probe + deep-copy.
// Delegates to the one SEH-faithful implementation: the old local copy used the
// RAISING ProbeForRead (unhandled here = BUGCHECK 0x1E) and an unprotected copy.
#[inline(always)]
unsafe fn ProbeAndCaptureUnicodeString(
    Dest: PUNICODE_STRING,
    CurrentMode: KPROCESSOR_MODE,
    UnsafeSrc: *const _UNICODE_STRING,
) -> NTSTATUS {
    crate::safe_user_io::probe_and_capture_unicode_string(Dest, CurrentMode as i8, UnsafeSrc)
}

// co_IntClientLoadLibrary + co_IntCallHookProc: STDCALL win32k exports
// (`_Name@N` on i386, cdecl elsewhere) — nm confirms @16 / @36.
extern "system" {
    fn co_IntClientLoadLibrary( pstrLibName: PUNICODE_STRING, pstrInitFunc: PUNICODE_STRING, Unload: BOOL, ApiHook: BOOL, ) -> BOOL;
    fn co_IntCallHookProc(HookId: INT, Code: INT, wParam: WPARAM, lParam: LPARAM, Proc: HOOKPROC, ihmod: INT, offPfn: ULONG_PTR, Ansi: u8, ModuleName: PUNICODE_STRING) -> LRESULT; // callback.rs: a hook's LRESULT, not 32 bits
}
extern "C" {
    // GetW32ThreadInfo + UserDeleteW32Thread + _MmCopyFromCaller: real exported cdecl symbols.
    fn GetW32ThreadInfo() -> PTHREADINFO;
    fn UserDeleteW32Thread(pti: PTHREADINFO);
    fn _MmCopyFromCaller(Target: PVOID, Source: PVOID, Bytes: UINT) -> NTSTATUS;
}

// THREADINFO embeds W32THREAD: pEThread at offset 0, RefCount (LONG) at size_of::<usize>().
#[inline(always)]
unsafe fn pti_refcount(pti: PTHREADINFO) -> *mut i32 {
    (pti as *mut u8).add(core::mem::size_of::<usize>()) as *mut i32
}
// IntReferenceThreadInfo / IntDereferenceThreadInfo (win32.h macros, no symbol).
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    InterlockedIncrement(pti_refcount(pti));
}
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    if InterlockedDecrement(pti_refcount(pti)) == 0 {
        UserDeleteW32Thread(pti);
    }
}

// MmCopyFromCaller(dst, src, len) macro → _MmCopyFromCaller (basemsg.h).
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: PVOID, src: PVOID, len: usize) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len as UINT)
}

// UserRefObjectCo / UserDerefObjectCo — static __inline (object.h), no exported symbol.
// USER_REFERENCE_ENTRY.Entry (SINGLE_LIST_ENTRY) is its first field. Matches focus.rs.
#[inline(always)]
unsafe fn UserRefObjectCo<T>(obj: *mut T, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj;
    UserReferenceObject(obj);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo<T>(obj: *mut T) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj);
}

// SafeProbeForRead/SafeProbeForWrite/ProbeForRead: NTAPI win32k/Mm exports
// (extern "system" = stdcall i386 / cdecl else). Matches desktop.rs/class.rs.
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: usize, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: usize, Alignment: u32) -> NTSTATUS;
}

// SetLastNtError: FASTCALL win32k export (`@SetLastNtError@4` on i386, cdecl else).
// Matches object.rs.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _HOOKPACK {
    pub pHk: PHOOK,
    pub lParam: LPARAM,
    pub pHookStructs: PVOID,
}
pub type HOOKPACK = _HOOKPACK;
pub type PHOOKPACK = *mut _HOOKPACK;

pub static mut strUahModule: _UNICODE_STRING = unsafe { core::mem::zeroed() };
pub static mut strUahInitFunc: _UNICODE_STRING = unsafe { core::mem::zeroed() };
// ppiUahServer (hook.c global, hook.h `extern`) — main.c references it. Export the
// undecorated C symbol; shadows the win32k_sys bindgen `extern` decl. (Matches input.rs.)
#[no_mangle]
pub static mut ppiUahServer: PPROCESSINFO = core::ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn IntLoadHookModule(iHookID: core::ffi::c_int, hHook: HHOOK, Unload: BOOL) -> BOOL {
    let ppi: PPROCESSINFO;
    let bResult: BOOL;

    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

    TRACE!(b"IntLoadHookModule. Client PID: %p\n\0".as_ptr() as *const core::ffi::c_char, PsGetProcessId(ppi_eprocess(ppi) as _));

    if iHookID == WH_APIHOOK as core::ffi::c_int {
        if Unload == 0 && ((*ppi_w32pf_flags(ppi)) & W32PF_APIHOOKLOADED) == 0 {
            (*ppi_w32pf_flags(ppi)) |= W32PF_APIHOOKLOADED;

            bResult = co_IntClientLoadLibrary(
                core::ptr::addr_of_mut!(strUahModule),
                core::ptr::addr_of_mut!(strUahInitFunc),
                Unload,
                1,
            );
            TRACE!(b"co_IntClientLoadLibrary returned %d\n\0".as_ptr() as *const core::ffi::c_char, bResult);
            if bResult == 0 {
                (*ppi_w32pf_flags(ppi)) &= !W32PF_APIHOOKLOADED;
            }
            return bResult;
        } else if Unload != 0 && ((*ppi_w32pf_flags(ppi)) & W32PF_APIHOOKLOADED) != 0 {
            bResult = co_IntClientLoadLibrary(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                Unload,
                1,
            );
            if bResult != 0 {
                (*ppi_w32pf_flags(ppi)) &= !W32PF_APIHOOKLOADED;
            }
            return bResult;
        }

        return 1;
    }

    0
}

unsafe fn IntHookModuleUnloaded(pdesk: PDESKTOP, iHookID: core::ffi::c_int, hHook: HHOOK) -> BOOL {
    let mut ptiCurrent: PTHREADINFO;
    let mut ListEntry: PLIST_ENTRY;
    let ppiCsr: PPROCESSINFO;

    TRACE!(b"IntHookModuleUnloaded: iHookID=%d\n\0".as_ptr() as *const core::ffi::c_char, iHookID);

    ppiCsr = PsGetProcessWin32Process(gpepCSRSS as _) as PPROCESSINFO;

    ListEntry = (*pdesk).PtiList.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*pdesk).PtiList) {
        ptiCurrent = (ListEntry as *mut u8)
            .offset(-(core::mem::offset_of!(THREADINFO, PtiLink) as isize))
            as PTHREADINFO;

        if PsIsSystemProcess(ppi_eprocess((*ptiCurrent).ppi) as _) == 0
            && (*ptiCurrent).ppi != ppiCsr
        {
            if (*ppi_w32pf_flags((*ptiCurrent).ppi)) & W32PF_APIHOOKLOADED != 0 {
                TRACE!(
                    b"IntHookModuleUnloaded: sending message to PID %p, ppi=%p\n\0".as_ptr()
                        as *const core::ffi::c_char,
                    PsGetProcessId(ppi_eprocess((*ptiCurrent).ppi) as _),
                    (*ptiCurrent).ppi,
                );
                co_MsqSendMessageAsync(
                    ptiCurrent,
                    core::ptr::null_mut(),
                    iHookID as UINT,
                    1usize as WPARAM,
                    hHook as LPARAM,
                    None,
                    0,
                    0,
                    MSQ_INJECTMODULE as INT,
                );
            }
        }
        ListEntry = (*ListEntry).Flink;
    }

    1
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn UserLoadApiHook() -> BOOL {
    IntLoadHookModule(WH_APIHOOK as core::ffi::c_int, 0 as HHOOK, 0)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn UserLoadApiHook() -> BOOL {
    IntLoadHookModule(WH_APIHOOK as core::ffi::c_int, 0 as HHOOK, 0)
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn UserRegisterUserApiHook(
    pstrDllName: PUNICODE_STRING,
    pstrFuncName: PUNICODE_STRING,
) -> BOOL {
    UserRegisterUserApiHook_impl(pstrDllName, pstrFuncName)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn UserRegisterUserApiHook(
    pstrDllName: PUNICODE_STRING,
    pstrFuncName: PUNICODE_STRING,
) -> BOOL {
    UserRegisterUserApiHook_impl(pstrDllName, pstrFuncName)
}

unsafe fn UserRegisterUserApiHook_impl(
    pstrDllName: PUNICODE_STRING,
    pstrFuncName: PUNICODE_STRING,
) -> BOOL {
    let pti: PTHREADINFO;
    let mut ptiCurrent: PTHREADINFO;
    let List: *mut HWND;
    let DesktopWindow: PWND;
    let mut pwndCurrent: PWND;
    let mut i: ULONG;
    let ppiCsr: PPROCESSINFO;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ppiCsr = PsGetProcessWin32Process(gpepCSRSS as _) as PPROCESSINFO;

    if (*gpsi).dwSRVIFlags & SRVINFO_APIHOOK != 0 {
        return 0;
    }

    (*gpsi).dwSRVIFlags |= SRVINFO_APIHOOK;

    strUahModule = *pstrDllName;
    strUahInitFunc = *pstrFuncName;
    ppiUahServer = (*pti).ppi;

    DesktopWindow = UserGetWindowObject(IntGetDesktopWindow());
    List = IntWinListChildren(DesktopWindow);

    if !List.is_null() {
        i = 0;
        while *List.offset(i as isize) != core::ptr::null_mut() {
            pwndCurrent = UserGetWindowObject(*List.offset(i as isize));
            if pwndCurrent.is_null() {
                i += 1;
                continue;
            }
            ptiCurrent = head_pti(core::ptr::addr_of!((*pwndCurrent).head));

            if PsIsSystemProcess(ppi_eprocess((*ptiCurrent).ppi) as _) != 0
                || (*ptiCurrent).ppi == ppiCsr
            {
                i += 1;
                continue;
            }

            co_MsqSendMessageAsync(
                ptiCurrent,
                core::ptr::null_mut(),
                WH_APIHOOK as UINT,
                0,
                0,
                None,
                0,
                0,
                MSQ_INJECTMODULE as INT,
            );
            i += 1;
        }
        ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserUnregisterUserApiHook() -> BOOL {
    let pti: PTHREADINFO;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if ((*gpsi).dwSRVIFlags & SRVINFO_APIHOOK) == 0 {
        return FALSE as BOOL;
    }

    if ppiUahServer != PsGetCurrentProcessWin32Process() as PPROCESSINFO {
        return FALSE as BOOL;
    }

    TRACE!(
        b"UserUnregisterUserApiHook. Server PID: %p\n\0".as_ptr() as *const core::ffi::c_char,
        PsGetProcessId(ppi_eprocess((*pti).ppi) as _),
    );

    (*gpsi).dwSRVIFlags &= !SRVINFO_APIHOOK;
    ppiUahServer = core::ptr::null_mut();
    ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(strUahModule), 1i32 as KPROCESSOR_MODE);
    ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(strUahInitFunc), 1i32 as KPROCESSOR_MODE);

    IntHookModuleUnloaded((*pti).rpdesk, WH_APIHOOK as core::ffi::c_int, core::ptr::null_mut())
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserUnregisterUserApiHook() -> BOOL {
    let pti: PTHREADINFO;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if ((*gpsi).dwSRVIFlags & SRVINFO_APIHOOK) == 0 {
        return FALSE as BOOL;
    }

    if ppiUahServer != PsGetCurrentProcessWin32Process() as PPROCESSINFO {
        return FALSE as BOOL;
    }

    TRACE!(
        b"UserUnregisterUserApiHook. Server PID: %p\n\0".as_ptr() as *const core::ffi::c_char,
        PsGetProcessId(ppi_eprocess((*pti).ppi) as _),
    );

    (*gpsi).dwSRVIFlags &= !SRVINFO_APIHOOK;
    ppiUahServer = core::ptr::null_mut();
    ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(strUahModule), 1i32 as KPROCESSOR_MODE);
    ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(strUahInitFunc), 1i32 as KPROCESSOR_MODE);

    IntHookModuleUnloaded((*pti).rpdesk, WH_APIHOOK as core::ffi::c_int, core::ptr::null_mut())
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntCallLowLevelHook(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    co_IntCallLowLevelHook_impl(Hook, Code, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntCallLowLevelHook(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    co_IntCallLowLevelHook_impl(Hook, Code, wParam, lParam)
}

unsafe fn co_IntCallLowLevelHook_impl(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let Status: NTSTATUS;
    let pti: PTHREADINFO;
    let pHP: PHOOKPACK;
    let mut Size: INT = 0;
    let mut uTimeout: UINT = 300;
    let mut Block: BOOL = FALSE as BOOL;
    let mut uResult: ULONG_PTR = 0;

    if !(*Hook).ptiHooked.is_null() {
        pti = (*Hook).ptiHooked;
    } else {
        pti = head_pti(core::ptr::addr_of!((*Hook).head));
    }

    pHP = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<HOOKPACK>() as _,
        TAG_HOOK,
    ) as PHOOKPACK;
    if pHP.is_null() {
        return 0;
    }

    (*pHP).pHk = Hook;
    (*pHP).lParam = lParam;
    (*pHP).pHookStructs = core::ptr::null_mut();

    match (*Hook).HookId as u32 {
        x if x == WH_JOURNALPLAYBACK || x == WH_JOURNALRECORD => {
            uTimeout = 0;
            Size = core::mem::size_of::<EVENTMSG>() as INT;
        }
        x if x == WH_KEYBOARD_LL => {
            Size = core::mem::size_of::<KBDLLHOOKSTRUCT>() as INT;
        }
        x if x == WH_MOUSE_LL => {
            Size = core::mem::size_of::<MSLLHOOKSTRUCT>() as INT;
        }
        x if x == WH_MOUSE => {
            uTimeout = 200;
            Block = TRUE as BOOL;
            Size = core::mem::size_of::<MOUSEHOOKSTRUCT>() as INT;
        }
        x if x == WH_KEYBOARD => {
            uTimeout = 200;
            Block = TRUE as BOOL;
        }
        _ => {}
    }

    if Size != 0 {
        (*pHP).pHookStructs =
            ExAllocatePoolWithTag(NonPagedPool, Size as _, TAG_HOOK);
        if !(*pHP).pHookStructs.is_null() {
            RtlCopyMemory(
                (*pHP).pHookStructs,
                lParam as PVOID,
                Size as usize,
            );
        }
    }

    Status = co_MsqSendMessage(
        pti,
        IntToPtr(Code) as HWND,
        (*Hook).HookId as UINT,
        wParam,
        pHP as LPARAM,
        uTimeout,
        Block,
        MSQ_ISHOOK as INT,
        &mut uResult,
    );
    if !NT_SUCCESS(Status) {
        ERR!(
            "Error Hook Call SendMsg. %d Status: 0x%x\n",
            (*Hook).HookId,
            Status,
        );
        if !(*pHP).pHookStructs.is_null() {
            ExFreePoolWithTag((*pHP).pHookStructs, TAG_HOOK);
        }
        ExFreePoolWithTag(pHP as PVOID, TAG_HOOK);
    }
    if NT_SUCCESS(Status) {
        uResult as LRESULT
    } else {
        0
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn co_CallHook(
    HookId: INT,
    Code: INT,
    wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut Result: LRESULT = 0;
    let phk: PHOOK;
    let pHP: PHOOKPACK = lParam as PHOOKPACK;

    phk = (*pHP).pHk;
    lParam = (*pHP).lParam;

    match HookId as u32 {
        WH_JOURNALPLAYBACK | WH_JOURNALRECORD | WH_KEYBOARD_LL | WH_MOUSE_LL | WH_MOUSE => {
            lParam = (*pHP).pHookStructs as LPARAM;
        }
        WH_KEYBOARD => {}
        _ => {}
    }

    if UserObjectInDestroy(UserHMGetHandle(phk)) == 0 {
        Result = (co_IntCallHookProc(
            HookId,
            Code,
            wParam,
            (lParam) as _,
            (*phk).Proc,
            (*phk).ihmod as INT,
            (*phk).offPfn,
            ((*phk).Ansi as INT) as _,
            core::ptr::addr_of_mut!((*phk).ModuleName),
        )) as _;
    }

    if !(*pHP).pHookStructs.is_null() {
        ExFreePoolWithTag((*pHP).pHookStructs, TAG_HOOK);
    }
    ExFreePoolWithTag(pHP as PVOID, TAG_HOOK);
    Result
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn co_CallHook(
    HookId: INT,
    Code: INT,
    wParam: WPARAM,
    mut lParam: LPARAM,
) -> LRESULT {
    let mut Result: LRESULT = 0;
    let phk: PHOOK;
    let pHP: PHOOKPACK = lParam as PHOOKPACK;

    phk = (*pHP).pHk;
    lParam = (*pHP).lParam;

    match HookId as u32 {
        WH_JOURNALPLAYBACK | WH_JOURNALRECORD | WH_KEYBOARD_LL | WH_MOUSE_LL | WH_MOUSE => {
            lParam = (*pHP).pHookStructs as LPARAM;
        }
        WH_KEYBOARD => {}
        _ => {}
    }

    if UserObjectInDestroy(UserHMGetHandle(phk)) == 0 {
        Result = (co_IntCallHookProc(
            HookId,
            Code,
            (wParam) as _,
            (lParam) as _,
            (*phk).Proc,
            (*phk).ihmod as INT,
            (*phk).offPfn,
            ((*phk).Ansi as INT) as _,
            core::ptr::addr_of_mut!((*phk).ModuleName),
        )) as _;
    }

    if !(*pHP).pHookStructs.is_null() {
        ExFreePoolWithTag((*pHP).pHookStructs, TAG_HOOK);
    }
    ExFreePoolWithTag(pHP as PVOID, TAG_HOOK);
    Result
}

unsafe extern "system" fn co_HOOK_CallHookNext(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    TRACE!(b"Calling Next HOOK %d\n\0".as_ptr() as *const i8, (*Hook).HookId);

    (co_IntCallHookProc(
        (*Hook).HookId,
        Code,
        (wParam) as _,
        (lParam) as _,
        (*Hook).Proc,
        (*Hook).ihmod as INT,
        (*Hook).offPfn,
        ((*Hook).Ansi as INT) as _,
        core::ptr::addr_of_mut!((*Hook).ModuleName),
    )) as _
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntCallDebugHook(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    co_IntCallDebugHook_impl(Hook, Code, wParam, lParam, Ansi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntCallDebugHook(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    co_IntCallDebugHook_impl(Hook, Code, wParam, lParam, Ansi)
}

unsafe fn co_IntCallDebugHook_impl(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
    _Ansi: BOOL,
) -> LRESULT {
    let mut Status: NTSTATUS;
    let mut lResult: LRESULT = 0;
    let Size: ULONG;
    let mut Debug: DEBUGHOOKINFO = core::mem::zeroed();
    let mut HooklParam: PVOID = core::ptr::null_mut();
    let mut BadChk: BOOL = 0;

    if lParam != 0 {
        Status = SafeProbeForRead(
            lParam as PVOID,
            core::mem::size_of::<DEBUGHOOKINFO>() as ULONG as usize,
            1,
        );
        // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
        // takes the existing bail-out below instead of faulting unhandled.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(core::ptr::addr_of_mut!(Debug) as PVOID, lParam as PVOID, core::mem::size_of::<DEBUGHOOKINFO>())
        {
            BadChk = 1;
        }

        if BadChk != 0 {
            ERR!("HOOK WH_DEBUG read from lParam ERROR!\n");
            return lResult;
        }
    } else {
        return lResult;
    }

    match wParam as INT {
        WHI_CBT => {
            match Debug.code {
                HCBTI_CLICKSKIPPED => {
                    Size = core::mem::size_of::<MOUSEHOOKSTRUCTEX>() as ULONG;
                }
                HCBTI_MOVESIZE => {
                    Size = core::mem::size_of::<RECT>() as ULONG;
                }
                HCBTI_ACTIVATE => {
                    Size = core::mem::size_of::<CBTACTIVATESTRUCT>() as ULONG;
                }
                HCBTI_CREATEWND => {
                    Size = core::mem::size_of::<CBT_CREATEWND>() as ULONG;
                }
                _ => {
                    Size = core::mem::size_of::<LPARAM>() as ULONG;
                }
            }
        }
        WHI_MOUSE_LL => {
            Size = core::mem::size_of::<MSLLHOOKSTRUCT>() as ULONG;
        }
        WHI_KEYBOARD_LL => {
            Size = core::mem::size_of::<KBDLLHOOKSTRUCT>() as ULONG;
        }
        WHI_MSGFILTER | WHI_SYSMSGFILTER | WHI_GETMESSAGE => {
            Size = core::mem::size_of::<MSG>() as ULONG;
        }
        WHI_JOURNALPLAYBACK | WHI_JOURNALRECORD => {
            Size = core::mem::size_of::<EVENTMSG>() as ULONG;
        }
        _ => {
            Size = core::mem::size_of::<LPARAM>() as ULONG;
        }
    }

    if Size as usize > core::mem::size_of::<LPARAM>() {
        HooklParam = ExAllocatePoolWithTag(NonPagedPool, Size as _, TAG_HOOK);
    }

    if !HooklParam.is_null() {
        Status = SafeProbeForRead(Debug.lParam as PVOID, Size as usize, 1);
        // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
        // takes the existing bail-out below instead of faulting unhandled.
        if !NT_SUCCESS(Status)
            || !crate::copy::capture_from_user(HooklParam, Debug.lParam as PVOID, Size as usize)
        {
            BadChk = 1;
        }

        if BadChk != 0 {
            ERR!("HOOK WH_DEBUG read from Debug.lParam ERROR!\n");
            ExFreePoolWithTag(HooklParam, TAG_HOOK);
            return lResult;
        }
    }

    if !HooklParam.is_null() {
        Debug.lParam = HooklParam as LPARAM;
    }
    lResult = co_HOOK_CallHookNext(Hook, Code, wParam, core::ptr::addr_of_mut!(Debug) as LPARAM);
    if !HooklParam.is_null() {
        ExFreePoolWithTag(HooklParam, TAG_HOOK);
    }

    lResult
}

unsafe extern "system" fn co_UserCallNextHookEx(
    Hook: PHOOK,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    let mut lResult: LRESULT = 0;
    let mut BadChk: BOOL = 0;
    let mut Status: NTSTATUS;

    if (*Hook).HookId == WH_MOUSE as i32
        || ((*Hook).HookId == WH_CBT as i32 && Code == HCBT_CLICKSKIPPED as i32)
    {
        let mut Mouse: MOUSEHOOKSTRUCTEX = core::mem::zeroed();
        if lParam != 0 {
            Status = SafeProbeForRead(
                lParam as PVOID,
                core::mem::size_of::<MOUSEHOOKSTRUCTEX>(),
                1,
            );
            // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
            // takes the existing bail-out below instead of faulting unhandled.
            if !NT_SUCCESS(Status)
                || !crate::copy::capture_from_user(&mut Mouse as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<MOUSEHOOKSTRUCTEX>())
            {
                BadChk = 1;
            }

            if BadChk != 0 {
                ERR!("HOOK WH_MOUSE read from lParam ERROR!\n");
            }
        }

        if BadChk == 0 {
            lResult = co_HOOK_CallHookNext(Hook, Code, wParam, &mut Mouse as *mut _ as LPARAM);
        }

        return lResult;
    }

    match (*Hook).HookId as INT {
        WHI_MOUSE_LL => {
            let mut Mouse: MSLLHOOKSTRUCT = core::mem::zeroed();

            if lParam != 0 {
                Status = SafeProbeForRead(
                    lParam as PVOID,
                    core::mem::size_of::<MSLLHOOKSTRUCT>(),
                    1,
                );
                // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                // takes the existing bail-out below instead of faulting unhandled.
                if !NT_SUCCESS(Status)
                    || !crate::copy::capture_from_user(&mut Mouse as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<MSLLHOOKSTRUCT>())
                {
                    BadChk = 1;
                }

                if BadChk != 0 {
                    ERR!("HOOK WH_MOUSE_LL read from lParam ERROR!\n");
                }
            }

            if BadChk == 0 {
                lResult =
                    co_HOOK_CallHookNext(Hook, Code, wParam, &mut Mouse as *mut _ as LPARAM);
            }
        }

        WHI_KEYBOARD_LL => {
            let mut Keyboard: KBDLLHOOKSTRUCT = core::mem::zeroed();

            if lParam != 0 {
                Status = SafeProbeForRead(
                    lParam as PVOID,
                    core::mem::size_of::<KBDLLHOOKSTRUCT>(),
                    1,
                );
                // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                // takes the existing bail-out below instead of faulting unhandled.
                if !NT_SUCCESS(Status)
                    || !crate::copy::capture_from_user(&mut Keyboard as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<KBDLLHOOKSTRUCT>())
                {
                    BadChk = 1;
                }

                if BadChk != 0 {
                    ERR!("HOOK WH_KEYBORD_LL read from lParam ERROR!\n");
                }
            }

            if BadChk == 0 {
                lResult =
                    co_HOOK_CallHookNext(Hook, Code, wParam, &mut Keyboard as *mut _ as LPARAM);
            }
        }

        WHI_MSGFILTER | WHI_SYSMSGFILTER | WHI_GETMESSAGE => {
            let mut Msg: MSG = core::mem::zeroed();

            if lParam != 0 {
                Status =
                    SafeProbeForRead(lParam as PVOID, core::mem::size_of::<MSG>(), 1);
                // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                // takes the existing bail-out below instead of faulting unhandled.
                if !NT_SUCCESS(Status)
                    || !crate::copy::capture_from_user(&mut Msg as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<MSG>())
                {
                    BadChk = 1;
                }

                if BadChk != 0 {
                    ERR!("HOOK WH_XMESSAGEX read from lParam ERROR!\n");
                }
            }

            if BadChk == 0 {
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, &mut Msg as *mut _ as LPARAM);

                if lParam != 0 && (*Hook).HookId == WH_GETMESSAGE as i32 {
                    Status = SafeProbeForWrite(
                        lParam as PVOID,
                        core::mem::size_of::<MSG>(),
                        1,
                    );
                    // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                    // takes the existing bail-out below instead of faulting unhandled.
                    if !NT_SUCCESS(Status)
                        || !crate::copy::capture_from_user(lParam as PVOID, &mut Msg as *mut _ as PVOID, core::mem::size_of::<MSG>())
                    {
                        BadChk = 1;
                    }

                    if BadChk != 0 {
                        ERR!("HOOK WH_GETMESSAGE write to lParam ERROR!\n");
                    }
                }
            }
        }

        WHI_CBT => {
            TRACE!(b"HOOK WH_CBT!\n\0".as_ptr() as *const i8);
            match Code as INT {
                HCBTI_CREATEWND => {
                    let pcbtcww: LPCBT_CREATEWNDW = lParam as LPCBT_CREATEWNDW;

                    TRACE!(b"HOOK HCBT_CREATEWND\n\0".as_ptr() as *const i8);
                    if Ansi != 0 {
                        Status = SafeProbeForRead(
                            pcbtcww as PVOID,
                            core::mem::size_of::<CBT_CREATEWNDA>(),
                            1,
                        );
                        // Each of these pointers has to be CAPTURED before it can be
                        // probed: reading it is itself an access to the probed-only
                        // user struct, which is the fault this whole class is about.
                        let mut lpcsA: PVOID = core::ptr::null_mut();
                        if NT_SUCCESS(Status)
                            && !crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(lpcsA) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*pcbtcww).lpcs) as *const core::ffi::c_void,
                                core::mem::size_of::<PVOID>(),
                            )
                        {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                        if NT_SUCCESS(Status) {
                            Status = SafeProbeForWrite(
                                lpcsA,
                                core::mem::size_of::<CREATESTRUCTA>(),
                                1,
                            );
                        }
                        let mut lpszNameA: PVOID = core::ptr::null_mut();
                        let mut lpszClassA: PVOID = core::ptr::null_mut();
                        if NT_SUCCESS(Status)
                            && (!crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(lpszNameA) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*(lpcsA as *const CREATESTRUCTA)).lpszName)
                                    as *const core::ffi::c_void,
                                core::mem::size_of::<PVOID>(),
                            )
                                || !crate::copy::capture_from_user(
                                    core::ptr::addr_of_mut!(lpszClassA) as *mut core::ffi::c_void,
                                    core::ptr::addr_of!((*(lpcsA as *const CREATESTRUCTA)).lpszClass)
                                        as *const core::ffi::c_void,
                                    core::mem::size_of::<PVOID>(),
                                ))
                        {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                        if NT_SUCCESS(Status) {
                            Status = SafeProbeForRead(
                                lpszNameA,
                                core::mem::size_of::<CHAR>(),
                                1,
                            );
                        }

                        if NT_SUCCESS(Status) && IS_ATOM!(lpszClassA as *const CHAR) == 0 {
                            Status = SafeProbeForRead(
                                lpszClassA,
                                core::mem::size_of::<CHAR>(),
                                1,
                            );
                        }
                    } else {
                        Status = SafeProbeForRead(
                            pcbtcww as PVOID,
                            core::mem::size_of::<CBT_CREATEWNDW>(),
                            1,
                        );
                        // Each of these pointers has to be CAPTURED before it can be
                        // probed: reading it is itself an access to the probed-only
                        // user struct, which is the fault this whole class is about.
                        let mut lpcsW: PVOID = core::ptr::null_mut();
                        if NT_SUCCESS(Status)
                            && !crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(lpcsW) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*pcbtcww).lpcs) as *const core::ffi::c_void,
                                core::mem::size_of::<PVOID>(),
                            )
                        {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                        if NT_SUCCESS(Status) {
                            Status = SafeProbeForWrite(
                                lpcsW,
                                core::mem::size_of::<CREATESTRUCTW>(),
                                1,
                            );
                        }
                        let mut lpszNameW: PVOID = core::ptr::null_mut();
                        let mut lpszClassW: PVOID = core::ptr::null_mut();
                        if NT_SUCCESS(Status)
                            && (!crate::copy::capture_from_user(
                                core::ptr::addr_of_mut!(lpszNameW) as *mut core::ffi::c_void,
                                core::ptr::addr_of!((*(lpcsW as *const CREATESTRUCTW)).lpszName)
                                    as *const core::ffi::c_void,
                                core::mem::size_of::<PVOID>(),
                            )
                                || !crate::copy::capture_from_user(
                                    core::ptr::addr_of_mut!(lpszClassW) as *mut core::ffi::c_void,
                                    core::ptr::addr_of!((*(lpcsW as *const CREATESTRUCTW)).lpszClass)
                                        as *const core::ffi::c_void,
                                    core::mem::size_of::<PVOID>(),
                                ))
                        {
                            Status = STATUS_ACCESS_VIOLATION;
                        }
                        if NT_SUCCESS(Status) {
                            Status = SafeProbeForRead(
                                lpszNameW,
                                core::mem::size_of::<WCHAR>(),
                                1,
                            );
                        }

                        if NT_SUCCESS(Status) && IS_ATOM!(lpszClassW as *const WCHAR) == 0 {
                            Status = SafeProbeForRead(
                                lpszClassW,
                                core::mem::size_of::<WCHAR>(),
                                1,
                            );
                        }
                    }

                    if !NT_SUCCESS(Status) {
                        BadChk = 1;
                    }

                    if BadChk != 0 {
                        ERR!("HOOK HCBT_CREATEWND write ERROR!\n");
                    }

                    if BadChk == 0 && !(*Hook).Proc.is_none() {
                        lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
                    }
                }

                HCBTI_MOVESIZE => {
                    let mut rt: RECTL = core::mem::zeroed();

                    TRACE!(b"HOOK HCBT_MOVESIZE\n\0".as_ptr() as *const i8);

                    if lParam != 0 {
                        Status = SafeProbeForRead(
                            lParam as PVOID,
                            core::mem::size_of::<RECT>(),
                            1,
                        );
                        // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                        // takes the existing bail-out below instead of faulting unhandled.
                        if !NT_SUCCESS(Status)
                            || !crate::copy::capture_from_user(&mut rt as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<RECT>())
                        {
                            BadChk = 1;
                        }

                        if BadChk != 0 {
                            ERR!("HOOK HCBT_MOVESIZE read from lParam ERROR!\n");
                        }
                    }

                    if BadChk == 0 {
                        lResult =
                            co_HOOK_CallHookNext(Hook, Code, wParam, &mut rt as *mut _ as LPARAM);
                    }
                }

                HCBTI_ACTIVATE => {
                    let mut CbAs: CBTACTIVATESTRUCT = core::mem::zeroed();

                    TRACE!(b"HOOK HCBT_ACTIVATE\n\0".as_ptr() as *const i8);
                    if lParam != 0 {
                        Status = SafeProbeForRead(
                            lParam as PVOID,
                            core::mem::size_of::<CBTACTIVATESTRUCT>(),
                            1,
                        );
                        // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                        // takes the existing bail-out below instead of faulting unhandled.
                        if !NT_SUCCESS(Status)
                            || !crate::copy::capture_from_user(&mut CbAs as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<CBTACTIVATESTRUCT>())
                        {
                            BadChk = 1;
                        }

                        if BadChk != 0 {
                            ERR!("HOOK HCBT_ACTIVATE read from lParam ERROR!\n");
                        }
                    }

                    if BadChk == 0 {
                        lResult = co_HOOK_CallHookNext(
                            Hook,
                            Code,
                            wParam,
                            &mut CbAs as *mut _ as LPARAM,
                        );
                    }
                }

                _ => {
                    TRACE!(b"HOOK HCBT_ %d\n\0".as_ptr() as *const i8, Code);
                    lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
                }
            }
        }

        WHI_JOURNALPLAYBACK | WHI_JOURNALRECORD => {
            let mut EventMsg: EVENTMSG = core::mem::zeroed();

            if lParam != 0 {
                Status = SafeProbeForRead(
                    lParam as PVOID,
                    core::mem::size_of::<EVENTMSG>(),
                    1,
                );
                // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                // takes the existing bail-out below instead of faulting unhandled.
                if !NT_SUCCESS(Status)
                    || !crate::copy::capture_from_user(&mut EventMsg as *mut _ as PVOID, lParam as PVOID, core::mem::size_of::<EVENTMSG>())
                {
                    BadChk = 1;
                }

                if BadChk != 0 {
                    ERR!("HOOK WH_JOURNAL read from lParam ERROR!\n");
                }
            }

            if BadChk == 0 {
                let arg = if lParam != 0 {
                    &mut EventMsg as *mut _ as LPARAM
                } else {
                    core::ptr::null_mut::<EVENTMSG>() as LPARAM
                };
                lResult = co_HOOK_CallHookNext(Hook, Code, wParam, arg);

                if lParam != 0 {
                    Status = SafeProbeForWrite(
                        lParam as PVOID,
                        core::mem::size_of::<EVENTMSG>(),
                        1,
                    );
                    // Validate-only probe; the copy is the access. A copy fault sets BadChk, so it
                    // takes the existing bail-out below instead of faulting unhandled.
                    if !NT_SUCCESS(Status)
                        || !crate::copy::capture_from_user(lParam as PVOID, &mut EventMsg as *mut _ as PVOID, core::mem::size_of::<EVENTMSG>())
                    {
                        BadChk = 1;
                    }

                    if BadChk != 0 {
                        ERR!("HOOK WH_JOURNAL write to lParam ERROR!\n");
                    }
                }
            }
        }

        WHI_DEBUG => {
            lResult = co_IntCallDebugHook(Hook, Code, wParam, lParam, Ansi);
        }

        WHI_FOREGROUNDIDLE | WHI_KEYBOARD | WHI_SHELL => {
            lResult = co_HOOK_CallHookNext(Hook, Code, wParam, lParam);
        }

        _ => {
            ERR!("Unsupported HOOK Id -> %d\n", (*Hook).HookId);
        }
    }

    lResult
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetHookObject(hHook: HHOOK) -> PHOOK {
    IntGetHookObjectImpl(hHook)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetHookObject(hHook: HHOOK) -> PHOOK {
    IntGetHookObjectImpl(hHook)
}

unsafe fn IntGetHookObjectImpl(hHook: HHOOK) -> PHOOK {
    let Hook: PHOOK;

    if hHook.is_null() {
        EngSetLastError(ERROR_INVALID_HOOK_HANDLE);
        return core::ptr::null_mut();
    }

    Hook = UserGetObject(gHandleTable, hHook as _, TYPE_HOOK) as PHOOK;
    if Hook.is_null() {
        EngSetLastError(ERROR_INVALID_HOOK_HANDLE);
        return core::ptr::null_mut();
    }

    UserReferenceObject(Hook as _);

    Hook
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetGlobalHookHandles(pdo: PDESKTOP, HookId: i32) -> *mut HHOOK {
    IntGetGlobalHookHandles_impl(pdo, HookId)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetGlobalHookHandles(pdo: PDESKTOP, HookId: i32) -> *mut HHOOK {
    IntGetGlobalHookHandles_impl(pdo, HookId)
}

unsafe fn IntGetGlobalHookHandles_impl(pdo: PDESKTOP, HookId: i32) -> *mut HHOOK {
    let pLastHead: PLIST_ENTRY;
    let mut pElem: PLIST_ENTRY;
    let mut i: u32 = 0;
    let mut cHooks: u32 = 0;
    let pList: *mut HHOOK;
    let mut pHook: PHOOK;

    pLastHead = core::ptr::addr_of_mut!(
        (*(*pdo).pDeskInfo).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
    );

    pElem = (*pLastHead).Flink;
    while pElem != pLastHead {
        cHooks += 1;
        pElem = (*pElem).Flink;
    }

    pList = ExAllocatePoolWithTag(
        PagedPool,
        (((cHooks + 1) as usize) * core::mem::size_of::<HHOOK>()) as _,
        TAG_HOOK,
    ) as *mut HHOOK;

    if pList.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    pElem = (*pLastHead).Flink;
    while pElem != pLastHead {
        pHook = (pElem as *mut u8).offset(-(core::mem::offset_of!(HOOK, Chain) as isize)) as PHOOK;
        *pList.offset(i as isize) = UserHMGetHandle(pHook) as HHOOK;
        i += 1;
        pElem = (*pElem).Flink;
    }
    *pList.offset(i as isize) = core::ptr::null_mut();

    pList
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetNextHook(Hook: PHOOK) -> PHOOK {
    IntGetNextHook_impl(Hook)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetNextHook(Hook: PHOOK) -> PHOOK {
    IntGetNextHook_impl(Hook)
}

unsafe fn IntGetNextHook_impl(Hook: PHOOK) -> PHOOK {
    let HookId: i32 = (*Hook).HookId;
    let pLastHead: PLIST_ENTRY;
    let pElem: PLIST_ENTRY;
    let pti: PTHREADINFO;

    if !(*Hook).ptiHooked.is_null() {
        pti = (*Hook).ptiHooked;
        pLastHead = core::ptr::addr_of_mut!((*pti).aphkStart[HOOKID_TO_INDEX(HookId) as usize]);
    } else {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        pLastHead = core::ptr::addr_of_mut!(
            (*(*(*pti).rpdesk).pDeskInfo).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
        );
    }

    pElem = (*Hook).Chain.Flink;
    if pElem != pLastHead {
        return (pElem as *mut u8)
            .sub(core::mem::offset_of!(HOOK, Chain)) as PHOOK;
    }
    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntFreeHook(Hook: PHOOK) {
    IntFreeHookImpl(Hook)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntFreeHook(Hook: PHOOK) {
    IntFreeHookImpl(Hook)
}

unsafe fn IntFreeHookImpl(Hook: PHOOK) {
    RemoveEntryList(core::ptr::addr_of_mut!((*Hook).Chain));
    if !(*Hook).ModuleName.Buffer.is_null() {
        ExFreePoolWithTag((*Hook).ModuleName.Buffer as *mut _, TAG_HOOK);
        (*Hook).ModuleName.Buffer = core::ptr::null_mut();
    }
    UserDeleteObject(UserHMGetHandle(Hook), TYPE_HOOK);
}

#[no_mangle]
pub unsafe extern "C" fn IntRemoveHook(Object: PVOID) -> BOOLEAN {
    let HookId: INT;
    let ptiHook: PTHREADINFO;
    let pti: PTHREADINFO;
    let pdo: PDESKTOP;
    let Hook: PHOOK = Object as PHOOK;

    HookId = (*Hook).HookId as INT;
    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if !(*Hook).ptiHooked.is_null() {
        ptiHook = (*Hook).ptiHooked;

        IntFreeHook(Hook);

        if IsListEmpty(core::ptr::addr_of_mut!(
            (*ptiHook).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
        )) != 0
        {
            let bOtherProcess: BOOL;
            let mut ApcState: _KAPC_STATE = core::mem::zeroed();

            (*ptiHook).fsHooks &= !HOOKID_TO_FLAG(HookId);
            bOtherProcess = ((*ptiHook).ppi != (*pti).ppi) as BOOL;

            if bOtherProcess != 0 {
                KeStackAttachProcess(
                    core::ptr::addr_of_mut!((*ppi_eprocess((*ptiHook).ppi)).Pcb) as _,
                    core::ptr::addr_of_mut!(ApcState) as _,
                );
            }

            (*(*ptiHook).pClientInfo).fsHooks = (*ptiHook).fsHooks;

            if bOtherProcess != 0 {
                KeUnstackDetachProcess(core::ptr::addr_of_mut!(ApcState) as _);
            }
        }
    } else {
        IntFreeHook(Hook);

        pdo = IntGetActiveDesktop();

        if !pdo.is_null()
            && !(*pdo).pDeskInfo.is_null()
            && IsListEmpty(core::ptr::addr_of_mut!(
                (*(*pdo).pDeskInfo).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
            )) != 0
        {
            (*(*pdo).pDeskInfo).fsHooks &= !HOOKID_TO_FLAG(HookId);
        }
    }

    1u8
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn co_HOOK_CallHooks(
    HookId: INT,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Hook: PHOOK;
    let SaveHook: PHOOK;
    let pti: PTHREADINFO;
    let mut ClientInfo: PCLIENTINFO;
    let pLastHead: PLIST_ENTRY;
    let pdo: PDESKTOP;
    let mut Local: BOOL = 0;
    let mut IsGlobal: BOOL = 0;
    let mut Result: LRESULT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    'Exit: loop {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        if pti.is_null() || (*pti).rpdesk.is_null() || (*(*pti).rpdesk).pDeskInfo.is_null() {
            pdo = IntGetActiveDesktop();
            if pti.is_null()
                || pdo.is_null()
                || (!(HookId == WH_KEYBOARD_LL as i32) && !(HookId == WH_MOUSE_LL as i32))
            {
                TRACE!(b"No PDO %d\n\0".as_ptr() as PCHAR, HookId);
                break 'Exit;
            }
        } else {
            pdo = (*pti).rpdesk;
        }

        if (*pti).TIF_flags & ((TIF_INCLEANUP | TIF_DISABLEHOOKS) as u32) != 0 {
            TRACE!(b"Hook Thread dead %d\n\0".as_ptr() as PCHAR, HookId);
            break 'Exit;
        }

        if ISITHOOKED(HookId) != 0 {
            TRACE!(b"Local Hooker %d\n\0".as_ptr() as PCHAR, HookId);
            Local = 1;
        }

        if (*(*pdo).pDeskInfo).fsHooks & HOOKID_TO_FLAG(HookId) != 0 {
            TRACE!(b"Global Hooker %d\n\0".as_ptr() as PCHAR, HookId);
            IsGlobal = 1;
        }

        if Local == 0 && IsGlobal == 0 {
            break 'Exit;
        }

        Hook = core::ptr::null_mut();

        if Local != 0 {
            pLastHead = core::ptr::addr_of_mut!(
                (*pti).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
            ) as PLIST_ENTRY;
            if IsListEmpty(pLastHead) != 0 {
                ERR!("No Local Hook Found!\n");
                break 'Exit;
            }

            Hook = ((*pLastHead).Flink as *mut u8)
                .offset(-(core::mem::offset_of!(HOOK, Chain) as isize))
                as PHOOK;
            ObReferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
            IntReferenceThreadInfo(pti);
            UserRefObjectCo(Hook as *mut core::ffi::c_void, &mut Ref);

            ClientInfo = (*pti).pClientInfo;
            SaveHook = (*pti).sphkCurrent;

            (*pti).sphkCurrent = Hook;
            (*Hook).phkNext = IntGetNextHook(Hook);
            if !ClientInfo.is_null() {
                (*ClientInfo).phkCurrent = Hook;
            }
            Result = (co_IntCallHookProc(
                HookId,
                Code,
                wParam,
                (lParam) as _,
                (*Hook).Proc,
                (*Hook).ihmod as INT,
                (*Hook).offPfn,
                ((*Hook).Ansi as INT) as _,
                core::ptr::addr_of_mut!((*Hook).ModuleName),
            )) as _;
            if !ClientInfo.is_null() {
                (*ClientInfo).phkCurrent = SaveHook;
            }
            (*pti).sphkCurrent = SaveHook;
            (*Hook).phkNext = core::ptr::null_mut();
            UserDerefObjectCo(Hook as *mut core::ffi::c_void);
            IntDereferenceThreadInfo(pti);
            ObDereferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
        }

        if IsGlobal != 0 {
            let ptiHook: PTHREADINFO;
            let pHookHandles: *mut HHOOK;
            let mut i: u32;

            pHookHandles = IntGetGlobalHookHandles(pdo, HookId);
            if pHookHandles.is_null() {
                break 'Exit;
            }

            i = 0;
            while !(*pHookHandles.offset(i as isize)).is_null() {
                Hook = UserGetObject(
                    gHandleTable,
                    *pHookHandles.offset(i as isize) as _,
                    TYPE_HOOK,
                ) as PHOOK;
                if Hook.is_null() {
                    ERR!("Invalid hook!\n");
                    i += 1;
                    continue;
                }

                let ptiHook = head_pti(core::ptr::addr_of!((*Hook).head));

                if ((*pti).TIF_flags & (TIF_DISABLEHOOKS as u32)) != 0
                    || ((*ptiHook).TIF_flags & (TIF_INCLEANUP as u32)) != 0
                {
                    TRACE!(
                        b"Next Hook %p, %p\n\0".as_ptr() as PCHAR,
                        (*ptiHook).rpdesk,
                        pdo,
                    );
                    i += 1;
                    continue;
                }
                UserRefObjectCo(Hook as *mut core::ffi::c_void, &mut Ref);

                if ptiHook != pti {
                    if HookId == WH_JOURNALPLAYBACK as i32
                        || HookId == WH_JOURNALRECORD as i32
                        || HookId == WH_KEYBOARD as i32
                        || HookId == WH_MOUSE as i32
                        || HookId == WH_KEYBOARD_LL as i32
                        || HookId == WH_MOUSE_LL as i32
                    {
                        TRACE!(
                            b"\nGlobal Hook posting to another Thread! %d\n\0".as_ptr()
                                as PCHAR,
                            HookId,
                        );
                        Result = co_IntCallLowLevelHook(Hook, Code, wParam, lParam);
                    } else if (*ptiHook).ppi == (*pti).ppi {
                        TRACE!(
                            b"\nGlobal Hook calling to another Thread! %d\n\0".as_ptr()
                                as PCHAR,
                            HookId,
                        );
                        ObReferenceObject(pti_ethread(ptiHook) as *mut core::ffi::c_void);
                        IntReferenceThreadInfo(ptiHook);
                        Result = (co_IntCallHookProc(
                            HookId,
                            Code,
                            wParam,
                            (lParam) as _,
                            (*Hook).Proc,
                            (*Hook).ihmod as INT,
                            (*Hook).offPfn,
                            ((*Hook).Ansi as INT) as _,
                            core::ptr::addr_of_mut!((*Hook).ModuleName),
                        )) as _;
                        IntDereferenceThreadInfo(ptiHook);
                        ObDereferenceObject(pti_ethread(ptiHook) as *mut core::ffi::c_void);
                    }
                } else {
                    TRACE!(
                        b"Global going Local Hook calling to Thread! %d\n\0".as_ptr()
                            as PCHAR,
                        HookId,
                    );
                    ObReferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
                    IntReferenceThreadInfo(pti);
                    Result = (co_IntCallHookProc(
                        HookId,
                        Code,
                        wParam,
                        (lParam) as _,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        ((*Hook).Ansi as INT) as _,
                        core::ptr::addr_of_mut!((*Hook).ModuleName),
                    )) as _;
                    IntDereferenceThreadInfo(pti);
                    ObDereferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
                }
                UserDerefObjectCo(Hook as *mut core::ffi::c_void);
                i += 1;
            }
            ExFreePoolWithTag(pHookHandles as *mut core::ffi::c_void, TAG_HOOK);
            TRACE!(
                b"Ret: Global HookId %d Result 0x%x\n\0".as_ptr() as PCHAR,
                HookId,
                Result,
            );
        }
        break 'Exit;
    }

    Result
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn co_HOOK_CallHooks(
    HookId: INT,
    Code: INT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let mut Hook: PHOOK;
    let SaveHook: PHOOK;
    let pti: PTHREADINFO;
    let mut ClientInfo: PCLIENTINFO;
    let pLastHead: PLIST_ENTRY;
    let pdo: PDESKTOP;
    let mut Local: BOOL = 0;
    let mut IsGlobal: BOOL = 0;
    let mut Result: LRESULT = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    'Exit: loop {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        if pti.is_null() || (*pti).rpdesk.is_null() || (*(*pti).rpdesk).pDeskInfo.is_null() {
            pdo = IntGetActiveDesktop();
            if pti.is_null()
                || pdo.is_null()
                || (!(HookId == WH_KEYBOARD_LL as i32) && !(HookId == WH_MOUSE_LL as i32))
            {
                TRACE!(b"No PDO %d\n\0".as_ptr() as PCHAR, HookId);
                break 'Exit;
            }
        } else {
            pdo = (*pti).rpdesk;
        }

        if (*pti).TIF_flags & ((TIF_INCLEANUP | TIF_DISABLEHOOKS) as u32) != 0 {
            TRACE!(b"Hook Thread dead %d\n\0".as_ptr() as PCHAR, HookId);
            break 'Exit;
        }

        if ISITHOOKED(HookId) != 0 {
            TRACE!(b"Local Hooker %d\n\0".as_ptr() as PCHAR, HookId);
            Local = 1;
        }

        if (*(*pdo).pDeskInfo).fsHooks & HOOKID_TO_FLAG(HookId) != 0 {
            TRACE!(b"Global Hooker %d\n\0".as_ptr() as PCHAR, HookId);
            IsGlobal = 1;
        }

        if Local == 0 && IsGlobal == 0 {
            break 'Exit;
        }

        Hook = core::ptr::null_mut();

        if Local != 0 {
            pLastHead = core::ptr::addr_of_mut!(
                (*pti).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
            ) as PLIST_ENTRY;
            if IsListEmpty(pLastHead) != 0 {
                ERR!("No Local Hook Found!\n");
                break 'Exit;
            }

            Hook = ((*pLastHead).Flink as *mut u8)
                .offset(-(core::mem::offset_of!(HOOK, Chain) as isize))
                as PHOOK;
            ObReferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
            IntReferenceThreadInfo(pti);
            UserRefObjectCo(Hook as *mut core::ffi::c_void, &mut Ref);

            ClientInfo = (*pti).pClientInfo;
            SaveHook = (*pti).sphkCurrent;

            (*pti).sphkCurrent = Hook;
            (*Hook).phkNext = IntGetNextHook(Hook);
            if !ClientInfo.is_null() {
                (*ClientInfo).phkCurrent = Hook;
            }
            Result = (co_IntCallHookProc(
                HookId,
                Code,
                (wParam) as _,
                (lParam) as _,
                (*Hook).Proc,
                (*Hook).ihmod as INT,
                (*Hook).offPfn,
                ((*Hook).Ansi as INT) as _,
                core::ptr::addr_of_mut!((*Hook).ModuleName),
            )) as _;
            if !ClientInfo.is_null() {
                (*ClientInfo).phkCurrent = SaveHook;
            }
            (*pti).sphkCurrent = SaveHook;
            (*Hook).phkNext = core::ptr::null_mut();
            UserDerefObjectCo(Hook as *mut core::ffi::c_void);
            IntDereferenceThreadInfo(pti);
            ObDereferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
        }

        if IsGlobal != 0 {
            let ptiHook: PTHREADINFO;
            let pHookHandles: *mut HHOOK;
            let mut i: u32;

            pHookHandles = IntGetGlobalHookHandles(pdo, HookId);
            if pHookHandles.is_null() {
                break 'Exit;
            }

            i = 0;
            while !(*pHookHandles.offset(i as isize)).is_null() {
                Hook = UserGetObject(
                    gHandleTable,
                    *pHookHandles.offset(i as isize) as _,
                    TYPE_HOOK,
                ) as PHOOK;
                if Hook.is_null() {
                    ERR!("Invalid hook!\n");
                    i += 1;
                    continue;
                }

                let ptiHook = head_pti(core::ptr::addr_of!((*Hook).head));

                if ((*pti).TIF_flags & (TIF_DISABLEHOOKS as u32)) != 0
                    || ((*ptiHook).TIF_flags & (TIF_INCLEANUP as u32)) != 0
                {
                    TRACE!(
                        b"Next Hook %p, %p\n\0".as_ptr() as PCHAR,
                        (*ptiHook).rpdesk,
                        pdo,
                    );
                    i += 1;
                    continue;
                }
                UserRefObjectCo(Hook as *mut core::ffi::c_void, &mut Ref);

                if ptiHook != pti {
                    if HookId == WH_JOURNALPLAYBACK as i32
                        || HookId == WH_JOURNALRECORD as i32
                        || HookId == WH_KEYBOARD as i32
                        || HookId == WH_MOUSE as i32
                        || HookId == WH_KEYBOARD_LL as i32
                        || HookId == WH_MOUSE_LL as i32
                    {
                        TRACE!(
                            b"\nGlobal Hook posting to another Thread! %d\n\0".as_ptr()
                                as PCHAR,
                            HookId,
                        );
                        Result = co_IntCallLowLevelHook(Hook, Code, wParam, lParam);
                    } else if (*ptiHook).ppi == (*pti).ppi {
                        TRACE!(
                            b"\nGlobal Hook calling to another Thread! %d\n\0".as_ptr()
                                as PCHAR,
                            HookId,
                        );
                        ObReferenceObject(pti_ethread(ptiHook) as *mut core::ffi::c_void);
                        IntReferenceThreadInfo(ptiHook);
                        Result = (co_IntCallHookProc(
                            HookId,
                            Code,
                            (wParam) as _,
                            (lParam) as _,
                            (*Hook).Proc,
                            (*Hook).ihmod as INT,
                            (*Hook).offPfn,
                            ((*Hook).Ansi as INT) as _,
                            core::ptr::addr_of_mut!((*Hook).ModuleName),
                        )) as _;
                        IntDereferenceThreadInfo(ptiHook);
                        ObDereferenceObject(pti_ethread(ptiHook) as *mut core::ffi::c_void);
                    }
                } else {
                    TRACE!(
                        b"Global going Local Hook calling to Thread! %d\n\0".as_ptr()
                            as PCHAR,
                        HookId,
                    );
                    ObReferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
                    IntReferenceThreadInfo(pti);
                    Result = (co_IntCallHookProc(
                        HookId,
                        Code,
                        (wParam) as _,
                        (lParam) as _,
                        (*Hook).Proc,
                        (*Hook).ihmod as INT,
                        (*Hook).offPfn,
                        ((*Hook).Ansi as INT) as _,
                        core::ptr::addr_of_mut!((*Hook).ModuleName),
                    )) as _;
                    IntDereferenceThreadInfo(pti);
                    ObDereferenceObject(pti_ethread(pti) as *mut core::ffi::c_void);
                }
                UserDerefObjectCo(Hook as *mut core::ffi::c_void);
                i += 1;
            }
            ExFreePoolWithTag(pHookHandles as *mut core::ffi::c_void, TAG_HOOK);
            TRACE!(
                b"Ret: Global HookId %d Result 0x%x\n\0".as_ptr() as PCHAR,
                HookId,
                Result,
            );
        }
        break 'Exit;
    }

    Result
}

#[no_mangle]
#[cfg(target_pointer_width = "32")]
pub unsafe extern "fastcall" fn IntUnhookWindowsHook(HookId: INT, pfnFilterProc: HOOKPROC) -> BOOL {
    IntUnhookWindowsHook_impl(HookId, pfnFilterProc)
}

#[no_mangle]
#[cfg(target_pointer_width = "64")]
pub unsafe extern "C" fn IntUnhookWindowsHook(HookId: INT, pfnFilterProc: HOOKPROC) -> BOOL {
    IntUnhookWindowsHook_impl(HookId, pfnFilterProc)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntUnhookWindowsHook_impl(HookId: INT, pfnFilterProc: HOOKPROC) -> BOOL {
    let mut Hook: PHOOK;
    let pLastHead: PLIST_ENTRY;
    let mut pElement: PLIST_ENTRY;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if HookId < WH_MINHOOK as INT || (WH_MAXHOOK as INT) < HookId {
        EngSetLastError(ERROR_INVALID_HOOK_FILTER);
        return 0;
    }

    if (*pti).fsHooks != 0 {
        pLastHead = core::ptr::addr_of_mut!((*pti).aphkStart[HOOKID_TO_INDEX(HookId) as usize]);

        pElement = (*pLastHead).Flink;
        while pElement != pLastHead {
            Hook = (pElement as *mut u8).offset(-(core::mem::offset_of!(HOOK, Chain) as isize)) as PHOOK;

            pElement = (*Hook).Chain.Flink;

            if (*Hook).Proc == pfnFilterProc {
                if head_pti(core::ptr::addr_of!((*Hook).head)) == pti {
                    IntRemoveHook(Hook as PVOID);
                    return 1;
                } else {
                    EngSetLastError(ERROR_ACCESS_DENIED);
                    return 0;
                }
            }
        }
    }
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallNextHookEx(
    Code: core::ffi::c_int,
    wParam: WPARAM,
    lParam: LPARAM,
    Ansi: BOOL,
) -> LRESULT {
    let pti: PTHREADINFO;
    let HookObj: PHOOK;
    let NextObj: PHOOK;
    let mut ClientInfo: PCLIENTINFO;
    let mut lResult: LRESULT = 0;

    UserEnterExclusive();

    'exit: {
        pti = GetW32ThreadInfo();

        HookObj = (*pti).sphkCurrent;

        if HookObj.is_null() {
            break 'exit;
        }

        NextObj = (*HookObj).phkNext;

        (*pti).sphkCurrent = NextObj;
        ClientInfo = (*pti).pClientInfo;

        (*ClientInfo).phkCurrent = NextObj;

        if !ClientInfo.is_null() && !NextObj.is_null() {
            (*NextObj).phkNext = IntGetNextHook(NextObj);
            lResult = co_UserCallNextHookEx(NextObj, Code, wParam, lParam, (*NextObj).Ansi as INT);
        }
    }

    UserLeave();
    lResult
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowsHookAW(
    idHook: ::core::ffi::c_int,
    lpfn: HOOKPROC,
    Ansi: BOOL,
) -> HHOOK {
    let ThreadId: DWORD;
    let mut USModuleName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(USModuleName) as _, core::ptr::null_mut());
    ThreadId = PtrToUint((*(NtCurrentTeb() as *mut _TEB)).ClientId.UniqueThread) as DWORD;

    NtUserSetWindowsHookEx(
        core::ptr::null_mut(),
        &mut USModuleName,
        ThreadId,
        idHook,
        lpfn,
        Ansi,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetWindowsHookEx(
    Mod: HINSTANCE,
    UnsafeModuleName: PUNICODE_STRING,
    ThreadId: DWORD,
    HookId: ::core::ffi::c_int,
    HookProc: HOOKPROC,
    Ansi: BOOL,
) -> HHOOK {
    let mut WinStaObj: PWINSTATION_OBJECT = core::ptr::null_mut();
    let mut Hook: PHOOK = core::ptr::null_mut();
    let mut ModuleName: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Handle: HHOOK = core::ptr::null_mut();
    let pti: PTHREADINFO;
    let mut ptiHook: PTHREADINFO = core::ptr::null_mut();
    let mut Ret: HHOOK = core::ptr::null_mut();

    UserEnterExclusive();

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    'cleanup: loop {
        if HookId < WH_MINHOOK as ::core::ffi::c_int || (WH_MAXHOOK as ::core::ffi::c_int) < HookId {
            EngSetLastError(ERROR_INVALID_HOOK_FILTER);
            break 'cleanup;
        }

        if HookProc.is_none() {
            EngSetLastError(ERROR_INVALID_FILTER_PROC);
            break 'cleanup;
        }

        if ThreadId != 0 {
            if HookId == WH_JOURNALRECORD as ::core::ffi::c_int
                || HookId == WH_JOURNALPLAYBACK as ::core::ffi::c_int
                || HookId == WH_KEYBOARD_LL as ::core::ffi::c_int
                || HookId == WH_MOUSE_LL as ::core::ffi::c_int
                || HookId == WH_SYSMSGFILTER as ::core::ffi::c_int
            {
                EngSetLastError(ERROR_GLOBAL_ONLY_HOOK);
                break 'cleanup;
            }

            ptiHook = IntTID2PTI(UlongToHandle(ThreadId));
            if ptiHook.is_null() {
                EngSetLastError(ERROR_INVALID_PARAMETER);
                break 'cleanup;
            }

            if (*ptiHook).rpdesk != (*pti).rpdesk {
                EngSetLastError(ERROR_ACCESS_DENIED);
                break 'cleanup;
            }

            if (*ptiHook).ppi != (*pti).ppi {
                if Mod.is_null()
                    && (HookId == WH_GETMESSAGE as ::core::ffi::c_int
                        || HookId == WH_CALLWNDPROC as ::core::ffi::c_int
                        || HookId == WH_CBT as ::core::ffi::c_int
                        || HookId == WH_HARDWARE as ::core::ffi::c_int
                        || HookId == WH_DEBUG as ::core::ffi::c_int
                        || HookId == WH_SHELL as ::core::ffi::c_int
                        || HookId == WH_FOREGROUNDIDLE as ::core::ffi::c_int
                        || HookId == WH_CALLWNDPROCRET as ::core::ffi::c_int)
                {
                    EngSetLastError(ERROR_HOOK_NEEDS_HMOD);
                    break 'cleanup;
                }

                if ((*ptiHook).TIF_flags & (TIF_CSRSSTHREAD | TIF_SYSTEMTHREAD)) != 0
                    && (HookId == WH_GETMESSAGE as ::core::ffi::c_int
                        || HookId == WH_CALLWNDPROC as ::core::ffi::c_int
                        || HookId == WH_CBT as ::core::ffi::c_int
                        || HookId == WH_HARDWARE as ::core::ffi::c_int
                        || HookId == WH_DEBUG as ::core::ffi::c_int
                        || HookId == WH_SHELL as ::core::ffi::c_int
                        || HookId == WH_FOREGROUNDIDLE as ::core::ffi::c_int
                        || HookId == WH_CALLWNDPROCRET as ::core::ffi::c_int)
                {
                    EngSetLastError(ERROR_HOOK_TYPE_NOT_ALLOWED);
                    break 'cleanup;
                }
            }
        } else {
            ptiHook = pti;
            if Mod.is_null()
                && (HookId == WH_GETMESSAGE as ::core::ffi::c_int
                    || HookId == WH_CALLWNDPROC as ::core::ffi::c_int
                    || HookId == WH_CBT as ::core::ffi::c_int
                    || HookId == WH_SYSMSGFILTER as ::core::ffi::c_int
                    || HookId == WH_HARDWARE as ::core::ffi::c_int
                    || HookId == WH_DEBUG as ::core::ffi::c_int
                    || HookId == WH_SHELL as ::core::ffi::c_int
                    || HookId == WH_FOREGROUNDIDLE as ::core::ffi::c_int
                    || HookId == WH_CALLWNDPROCRET as ::core::ffi::c_int)
            {
                EngSetLastError(ERROR_HOOK_NEEDS_HMOD);
                break 'cleanup;
            }
        }

        Status = IntValidateWindowStationHandle(
            (*(PsGetCurrentProcess() as *mut _EPROCESS)).Win32WindowStation as _,
            1i32 as i8,
            0,
            &mut WinStaObj,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }
        ObDereferenceObject(WinStaObj as PVOID);

        Hook = UserCreateObject(
            gHandleTable,
            core::ptr::null_mut(),
            ptiHook as _,
            &mut Handle as *mut HHOOK as PHANDLE,
            TYPE_HOOK as _,
            core::mem::size_of::<HOOK>() as u32,
        ) as PHOOK;

        if Hook.is_null() {
            break 'cleanup;
        }

        (*Hook).ihmod = Mod as INT_PTR;
        (*Hook).HookId = HookId;
        (*Hook).rpdesk = (*ptiHook).rpdesk;
        (*Hook).phkNext = core::ptr::null_mut();
        (*Hook).Proc = HookProc;
        (*Hook).Ansi = Ansi as u8;

        if ThreadId != 0 {
            InsertHeadList(
                core::ptr::addr_of_mut!((*ptiHook).aphkStart[HOOKID_TO_INDEX(HookId) as usize]),
                core::ptr::addr_of_mut!((*Hook).Chain),
            );
            (*ptiHook).sphkCurrent = core::ptr::null_mut();
            (*Hook).ptiHooked = ptiHook;
            (*ptiHook).fsHooks |= HOOKID_TO_FLAG(HookId);

            if !(*ptiHook).pClientInfo.is_null() {
                if (*ptiHook).ppi == (*pti).ppi {
                    (*(*ptiHook).pClientInfo).fsHooks = (*ptiHook).fsHooks;
                    (*(*ptiHook).pClientInfo).phkCurrent = core::ptr::null_mut();
                } else {
                    let mut ApcState: _KAPC_STATE = core::mem::zeroed();

                    KeStackAttachProcess(
                        core::ptr::addr_of_mut!((*ppi_eprocess((*ptiHook).ppi)).Pcb) as _,
                        &mut ApcState as *mut _ as _,
                    );
                    (*(*ptiHook).pClientInfo).fsHooks = (*ptiHook).fsHooks;
                    (*(*ptiHook).pClientInfo).phkCurrent = core::ptr::null_mut();
                    KeUnstackDetachProcess(&mut ApcState as *mut _ as _);
                }
            }
        } else {
            InsertHeadList(
                core::ptr::addr_of_mut!(
                    (*(*(*ptiHook).rpdesk).pDeskInfo).aphkStart[HOOKID_TO_INDEX(HookId) as usize]
                ),
                core::ptr::addr_of_mut!((*Hook).Chain),
            );
            (*Hook).ptiHooked = core::ptr::null_mut();
            (*(*(*ptiHook).rpdesk).pDeskInfo).fsHooks |= HOOKID_TO_FLAG(HookId);
            (*ptiHook).sphkCurrent = core::ptr::null_mut();
            (*(*ptiHook).pClientInfo).phkCurrent = core::ptr::null_mut();
        }

        RtlInitUnicodeString(
            core::ptr::addr_of_mut!((*Hook).ModuleName) as _,
            core::ptr::null_mut(),
        );

        if !Mod.is_null() {
            Status = MmCopyFromCaller(
                &mut ModuleName as *mut _UNICODE_STRING as PVOID,
                UnsafeModuleName as PVOID,
                core::mem::size_of::<_UNICODE_STRING>(),
            );
            if !NT_SUCCESS(Status) {
                IntRemoveHook(Hook as PVOID);
                SetLastNtError(Status);
                break 'cleanup;
            }

            (*Hook).ModuleName.Buffer = ExAllocatePoolWithTag(
                PagedPool,
                ModuleName.MaximumLength as _,
                TAG_HOOK,
            ) as *mut u16;
            if (*Hook).ModuleName.Buffer.is_null() {
                IntRemoveHook(Hook as PVOID);
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }

            (*Hook).ModuleName.MaximumLength = ModuleName.MaximumLength;
            Status = MmCopyFromCaller(
                (*Hook).ModuleName.Buffer as PVOID,
                ModuleName.Buffer as PVOID,
                ModuleName.MaximumLength as usize,
            );
            if !NT_SUCCESS(Status) {
                ExFreePoolWithTag((*Hook).ModuleName.Buffer as PVOID, TAG_HOOK);
                (*Hook).ModuleName.Buffer = core::ptr::null_mut();
                IntRemoveHook(Hook as PVOID);
                SetLastNtError(Status);
                break 'cleanup;
            }

            (*Hook).ModuleName.Length = ModuleName.Length;
            (*Hook).offPfn = (HookProc.map_or(core::ptr::null::<u8>(), |p| p as *const u8) as usize)
                .wrapping_sub(Mod as usize) as ULONG_PTR;
        } else {
            (*Hook).offPfn = 0;
        }

        Ret = Handle;
        break 'cleanup;
    }

    if !Hook.is_null() {
        UserDereferenceObject(Hook as PVOID);
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnhookWindowsHookEx(Hook: HHOOK) -> BOOL {
    let HookObj: PHOOK;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'exit: {
        HookObj = IntGetHookObject(Hook);
        if HookObj.is_null() {
            break 'exit;
        }

        IntRemoveHook(HookObj as PVOID);

        UserDereferenceObject(HookObj as PVOID);

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserRegisterUserApiHook(
    m_dllname1: PUNICODE_STRING,
    m_funname1: PUNICODE_STRING,
    dwUnknown3: DWORD,
    dwUnknown4: DWORD,
) -> BOOL {
    let ret: BOOL;
    let mut strDllNameSafe: _UNICODE_STRING = core::mem::zeroed();
    let mut strFuncNameSafe: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = ProbeAndCaptureUnicodeString(
        core::ptr::addr_of_mut!(strDllNameSafe),
        1i32 as KPROCESSOR_MODE,
        m_dllname1,
    );
    if !NT_SUCCESS(Status) {
        EngSetLastError(RtlNtStatusToDosError(Status));
        return FALSE as BOOL;
    }

    Status = ProbeAndCaptureUnicodeString(
        core::ptr::addr_of_mut!(strFuncNameSafe),
        1i32 as KPROCESSOR_MODE,
        m_funname1,
    );
    if !NT_SUCCESS(Status) {
        ReleaseCapturedUnicodeString(
            core::ptr::addr_of_mut!(strDllNameSafe),
            1i32 as KPROCESSOR_MODE,
        );
        EngSetLastError(RtlNtStatusToDosError(Status));
        return FALSE as BOOL;
    }

    UserEnterExclusive();

    ret = UserRegisterUserApiHook(
        core::ptr::addr_of_mut!(strDllNameSafe),
        core::ptr::addr_of_mut!(strFuncNameSafe),
    );

    UserLeave();

    if ret == FALSE as BOOL {
        ReleaseCapturedUnicodeString(
            core::ptr::addr_of_mut!(strDllNameSafe),
            1i32 as KPROCESSOR_MODE,
        );
        ReleaseCapturedUnicodeString(
            core::ptr::addr_of_mut!(strFuncNameSafe),
            1i32 as KPROCESSOR_MODE,
        );
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUnregisterUserApiHook() -> BOOL {
    let ret: BOOL;

    UserEnterExclusive();
    ret = UserUnregisterUserApiHook();
    UserLeave();

    ret
}
