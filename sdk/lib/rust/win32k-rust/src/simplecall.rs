#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::ddk::{PsGetCurrentProcessWin32Process, PsGetCurrentThread, PsGetCurrentThreadWin32Thread, PsLookupProcessByProcessId, RtlNtStatusToDosError};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ddk::ExAllocatePoolWithTag;
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::rtl::RtlZeroMemory;

// USERTAG_SWP pool tag. C multi-char literal 'wssU' is big-endian; the LE byte
// order is the literal reversed -> "Ussw".
const USERTAG_SWP: u32 = u32::from_le_bytes(*b"Ussw");

// HWND_BOTTOM = (HWND)1 / HWND_DESKTOP = (HWND)0 (winuser.h macros).
const HWND_BOTTOM: HWND = 1usize as HWND;

// MAKELONG/MAKELONG/MAKEWPARAM (minwindef.h / winuser.h macros).
#[inline(always)]
fn MAKELONG(low: u32, high: u32) -> LONG {
    ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG
}
#[inline(always)]
fn MAKEWPARAM(low: u32, high: u32) -> WPARAM {
    MAKELONG(low, high) as u32 as WPARAM
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
// FORCEINLINE macro — no exported symbol.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(pobj: *mut T) -> HANDLE {
    *(pobj as *const HANDLE)
}

// THRDESKHEAD/THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into
// __bindgen_padding_0. pti follows HEAD{HANDLE, DWORD} and is ptr-aligned, so it
// sits at 2*usize (8 i386 / 16 amd64/arm64) — HANDLE+DWORD (=12 on 64-bit) would
// MISS the tail padding. (Matches timer.rs/event.rs head_pti.)
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// gpsi->{aiSysMet,ptCursor}: tagSERVERINFO ends with __bindgen_padding_0
// representing the anonymous PERUSERSERVERINFO member; cast that tail to access
// its fields (matches timer.rs/mouse.rs gpsi_per_user).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// PROCESSINFO->W32PF_flags lives in the flattened W32PROCESS prefix
// (peProcess ptr + RefCount DWORD), so W32PF_flags (ULONG) sits at
// size_of::<usize>()+size_of::<u32>() = 8 (i386) / 12 (amd64/arm64).
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    let off = core::mem::size_of::<usize>() + core::mem::size_of::<u32>();
    (ppi as *mut u8).add(off) as *mut DWORD
}

// IntReleaseMenuObject(MenuObj) (menu.h macro) -> UserDereferenceObject(MenuObj).
#[inline(always)]
unsafe fn IntReleaseMenuObject(MenuObj: PMENU) {
    UserDereferenceObject(MenuObj as PVOID);
}

// DesktopHeapAddressToUser (desktop.h static __inline) — no exported symbol.
#[inline(always)]
unsafe fn DesktopHeapAddressToUser(lpMem: PVOID) -> PVOID {
    let W32Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    // Skip the first mapping (global user heap mapping).
    let mut Mapping: PW32HEAP_USER_MAPPING = (*W32Process).HeapMappings.Next;
    while !Mapping.is_null() {
        if (lpMem as ULONG_PTR) >= ((*Mapping).KernelMapping as ULONG_PTR)
            && (lpMem as ULONG_PTR)
                < ((*Mapping).KernelMapping as ULONG_PTR) + (*Mapping).Limit
        {
            return (((lpMem as ULONG_PTR) - ((*Mapping).KernelMapping as ULONG_PTR))
                + ((*Mapping).UserMapping as ULONG_PTR)) as PVOID;
        }
        Mapping = (*Mapping).Next;
    }
    core::ptr::null_mut()
}

// (HOOKPROC)Param — HOOKPROC is Option<extern fn>; its layout is identical to the
// raw function pointer, so transmute the integer (matches timer.rs pfn_as_lparam).
#[inline(always)]
unsafe fn int_as_hookproc(p: DWORD_PTR) -> HOOKPROC {
    core::mem::transmute::<usize, HOOKPROC>(p as usize)
}

// co_IntShellHookNotify — cdecl (`_co_IntShellHookNotify`).
extern "C" {
    fn co_IntShellHookNotify(Message: WPARAM, wParam: WPARAM, lParam: LPARAM);
}

// FASTCALL (`@Name@N`) on i386, cdecl elsewhere. (winpos.c/foldesk.c/painting.c/menu/object.h)
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_IntSetForegroundWindow(Window: PWND) -> BOOL;
    fn co_IntSetForegroundWindowMouse(Window: PWND) -> BOOL;
    fn co_IntUpdateWindows(Window: PWND, Flags: u32, Recurse: BOOL);
    fn co_WinPosArrangeIconicWindows(Window: PWND) -> u32;
    fn co_WinPosSetWindowPos( Window: PWND, WndInsertAfter: HWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: u32, ) -> BOOL;
    fn co_UserRedrawWindow(Window: PWND, UpdateRect: *mut core::ffi::c_void, Rgn: PREGION, Flags: u32) -> u8;
    fn DWP_GetEnabledPopup(pWnd: PWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_IntSetForegroundWindow(Window: PWND) -> BOOL;
    fn co_IntSetForegroundWindowMouse(Window: PWND) -> BOOL;
    fn co_IntUpdateWindows(Window: PWND, Flags: u32, Recurse: BOOL);
    fn co_WinPosArrangeIconicWindows(Window: PWND) -> u32;
    fn co_WinPosSetWindowPos( Window: PWND, WndInsertAfter: HWND, x: core::ffi::c_int, y: core::ffi::c_int, cx: core::ffi::c_int, cy: core::ffi::c_int, flags: u32, ) -> BOOL;
    fn co_UserRedrawWindow(Window: PWND, UpdateRect: *mut core::ffi::c_void, Rgn: PREGION, Flags: u32) -> u8;
    fn DWP_GetEnabledPopup(pWnd: PWND) -> PWND;
    fn UserReferenceObject(obj: PVOID);
}

// HandleToUlong(h) = ((ULONG)(ULONG_PTR)(h)) — basetsd.h macro, no symbol.
#[inline(always)]
unsafe fn HandleToUlong(h: *mut core::ffi::c_void) -> u32 {
    h as ULONG_PTR as u32
}

// UserRefObjectCo / UserDerefObjectCo — static __inline (object.h), no exported
// symbol. ReferencesList is a real bindgen field on _THREADINFO; USER_REFERENCE_ENTRY
// has Entry (SINGLE_LIST_ENTRY) as its first field so the entry ptr == struct ptr.
#[inline(always)]
unsafe fn UserRefObjectCo(obj: PVOID, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj;
    UserReferenceObject(obj);
    // PushEntryList(&W32Thread->ReferencesList, &UserReferenceEntry->Entry)
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo(obj: PVOID) {
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    // PopEntryList(&W32Thread->ReferencesList)
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    // CONTAINING_RECORD(ReferenceEntry, USER_REFERENCE_ENTRY, Entry): Entry @ offset 0.
    let _user_reference_entry = reference_entry as *mut USER_REFERENCE_ENTRY;
    UserDereferenceObject(obj);
}

// ---- cross-file win32k symbols (decoration verified via llvm-nm) ----
//
// FASTCALL (`@Name@N`) — extern "fastcall" on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG_PTR);
    fn MsqSetMessageExtraInfo(lData: LPARAM) -> LPARAM;
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: ULONG, hWnd: HWND) -> HWND;
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *const MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR);
    fn SetLastNtError(Status: NTSTATUS);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqPostQuitMessage(pti: PTHREADINFO, ExitCode: ULONG_PTR);
    fn MsqSetMessageExtraInfo(lData: LPARAM) -> LPARAM;
    fn MsqSetStateWindow(pti: PTHREADINFO, Type: ULONG, hWnd: HWND) -> HWND;
    fn MsqPostMessage(pti: PTHREADINFO, Msg: *const MSG, HardwareMessage: BOOLEAN, MessageBits: DWORD, dwQEvent: DWORD, ExtraInfo: LONG_PTR);
    fn SetLastNtError(Status: NTSTATUS);
    fn UserDereferenceObject(obj: PVOID) -> BOOL;
}

// GetW32ThreadInfo — exported NTAPI (`_GetW32ThreadInfo`, cdecl on i386).
extern "C" {
    fn GetW32ThreadInfo() -> PTHREADINFO;
}

// SafeProbeForWrite — NTAPI (`_SafeProbeForWrite@12`, stdcall on i386).
extern "system" {
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// Exported data (`_gpidLogon`) — referenced by C.
#[no_mangle]
pub static mut gpidLogon: HANDLE = 0 as HANDLE;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn co_IntRegisterLogonProcess(
    mut ProcessId: HANDLE,
    Register: BOOL,
) -> BOOL {
    co_IntRegisterLogonProcess_impl(ProcessId, Register)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn co_IntRegisterLogonProcess(
    mut ProcessId: HANDLE,
    Register: BOOL,
) -> BOOL {
    co_IntRegisterLogonProcess_impl(ProcessId, Register)
}

unsafe fn co_IntRegisterLogonProcess_impl(mut ProcessId: HANDLE, Register: BOOL) -> BOOL {
    let mut Status: NTSTATUS;
    let mut Process: PEPROCESS = core::ptr::null_mut();

    Status = PsLookupProcessByProcessId(ProcessId, &mut Process as *mut _ as *mut _);
    if !NT_SUCCESS(Status) {
        EngSetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    ProcessId = (*Process).UniqueProcessId;

    ObDereferenceObject(Process as _);

    if Register != 0 {
        if gpidLogon != core::ptr::null_mut() {
            return 0;
        }

        gpidLogon = ProcessId;
    } else {
        if gpidLogon != ProcessId {
            return 0;
        }

        gpidLogon = core::ptr::null_mut();
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallNoParam(Routine: DWORD) -> DWORD_PTR {
    let mut Result: DWORD_PTR = 0;

    UserEnterExclusive();

    match Routine {
        NOPARAM_ROUTINE_CREATEMENU => {
            Result = UserCreateMenu((*GetW32ThreadInfo()).rpdesk, 0) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_CREATEMENUPOPUP => {
            Result = UserCreateMenu((*GetW32ThreadInfo()).rpdesk, 1) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_DESTROY_CARET => {
            Result = co_IntDestroyCaret((*PsGetCurrentThread()).Tcb.Win32Thread as PTHREADINFO) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_INIT_MESSAGE_PUMP => {
            Result = IntInitMessagePumpHook() as DWORD_PTR;
        }

        NOPARAM_ROUTINE_UNINIT_MESSAGE_PUMP => {
            Result = IntUninitMessagePumpHook() as DWORD_PTR;
        }

        NOPARAM_ROUTINE_MSQCLEARWAKEMASK => {
            Result = IntMsqClearWakeMask() as DWORD_PTR;
        }

        NOPARAM_ROUTINE_GETMSESSAGEPOS => {
            let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
            Result = MAKELONG((*pti).ptLast.x as u32, (*pti).ptLast.y as u32) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_RELEASECAPTURE => {
            Result = IntReleaseCapture() as DWORD_PTR;
        }

        NOPARAM_ROUTINE_LOADUSERAPIHOOK => {
            Result = UserLoadApiHook() as DWORD_PTR;
        }

        NOPARAM_ROUTINE_ZAPACTIVEANDFOUS => {
            let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
            (*(*pti).MessageQueue).spwndFocus = core::ptr::null_mut();
            (*(*pti).MessageQueue).spwndActive = core::ptr::null_mut();
            Result = 0;
        }

        NOPARAM_ROUTINE_GETIMESHOWSTATUS => {
            Result = (gfIMEShowStatus != 0) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_ISCONSOLEMODE => {
            Result = (ScreenDeviceContext == core::ptr::null_mut()) as DWORD_PTR;
        }

        NOPARAM_ROUTINE_UPDATEPERUSERIMMENABLING => {
            if UserIsIMMEnabled() != 0 {
                (*gpsi).dwSRVIFlags |= SRVINFO_IMM32;
            } else {
                (*gpsi).dwSRVIFlags &= !SRVINFO_IMM32;
            }

            Result = 1;
        }

        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    UserLeave();

    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallOneParam(
    Param: DWORD_PTR,
    Routine: DWORD,
) -> DWORD_PTR {
    let mut Result: DWORD_PTR;
    let Status: NTSTATUS;

    UserEnterExclusive();

    match Routine {
        ONEPARAM_ROUTINE_POSTQUITMESSAGE => {
            let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
            MsqPostQuitMessage(pti, Param);
            Result = 1 as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_BEGINDEFERWNDPOS => {
            'block: {
                let psmwp: PSMWP;
                let mut hDwp: HDWP = core::ptr::null_mut();
                let mut count: INT = Param as INT;

                if count < 0 {
                    EngSetLastError(ERROR_INVALID_PARAMETER);
                    Result = 0;
                    break 'block;
                }

                if count == 0 {
                    count = 8;
                }

                psmwp = UserCreateObject(
                    gHandleTable,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(hDwp) as PHANDLE,
                    TYPE_SETWINDOWPOS,
                    core::mem::size_of::<SMWP>() as u32,
                ) as PSMWP;
                if psmwp.is_null() {
                    Result = 0;
                    break 'block;
                }

                (*psmwp).acvr = ExAllocatePoolWithTag(
                    PagedPool,
                    ((count as usize) * core::mem::size_of::<CVR>()) as SIZE_T,
                    USERTAG_SWP,
                ) as *mut _;
                if (*psmwp).acvr.is_null() {
                    UserDeleteObject(hDwp as PVOID, TYPE_SETWINDOWPOS);
                    Result = 0;
                    break 'block;
                }

                RtlZeroMemory(
                    (*psmwp).acvr as *mut _,
                    (count as usize) * core::mem::size_of::<CVR>(),
                );
                (*psmwp).set_bHandle(1);
                (*psmwp).ccvr = 0;
                (*psmwp).ccvrAlloc = count;
                Result = hDwp as DWORD_PTR;
            }
        }

        ONEPARAM_ROUTINE_SHOWCURSOR => {
            Result = UserShowCursor(Param as BOOL) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_GETDESKTOPMAPPING => {
            let ti: PTHREADINFO = GetW32ThreadInfo();
            if !ti.is_null() {
                Result = DesktopHeapAddressToUser(Param as PVOID) as DWORD_PTR;
            } else {
                Result = 0;
            }
        }

        ONEPARAM_ROUTINE_WINDOWFROMDC => {
            Result = IntWindowFromDC(Param as HDC) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_SWAPMOUSEBUTTON => {
            Result = gspv.bMouseBtnSwap as DWORD_PTR;
            gspv.bMouseBtnSwap = if Param != 0 { 1 } else { 0 };
            (*gpsi_per_user()).aiSysMet[SM_SWAPBUTTON as usize] = gspv.bMouseBtnSwap as _;
        }

        ONEPARAM_ROUTINE_SETCARETBLINKTIME => {
            Result = IntSetCaretBlinkTime(Param as UINT) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_SETMESSAGEEXTRAINFO => {
            Result = MsqSetMessageExtraInfo(Param as LPARAM) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_CREATEEMPTYCUROBJECT => {
            Result = IntCreateCurIconHandle(Param as u8) as DWORD_PTR;
            if Result == 0 {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                Result = 0;
            }
        }

        ONEPARAM_ROUTINE_ENABLEPROCWNDGHSTING => {
            'block: {
                let Process: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

                if !Process.is_null() {
                    let Enable: BOOL = (Param != 0) as BOOL;

                    if Enable != 0 {
                        *ppi_w32pf_flags(Process) &= !W32PF_NOWINDOWGHOSTING;
                    } else {
                        *ppi_w32pf_flags(Process) |= W32PF_NOWINDOWGHOSTING;
                    }

                    Result = 1 as DWORD_PTR;
                    break 'block;
                }

                Result = 0;
            }
        }

        ONEPARAM_ROUTINE_GETINPUTEVENT => {
            Result = IntMsqSetWakeMask(Param as DWORD) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_GETKEYBOARDTYPE => {
            Result = UserGetKeyboardType(Param as DWORD) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_GETKEYBOARDLAYOUT => {
            Result = UserGetKeyboardLayout(Param as DWORD) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_RELEASEDC => {
            Result = UserReleaseDC(core::ptr::null_mut(), Param as HDC, 0) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_REALIZEPALETTE => {
            Result = UserRealizePalette(Param as HDC) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_GETQUEUESTATUS => {
            Result = IntGetQueueStatus(Param as DWORD) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_ENUMCLIPBOARDFORMATS => {
            Result = UserEnumClipboardFormats(Param as UINT) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_GETCURSORPOS => {
            'block: {
                let pptl: PPOINTL;
                let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
                Result = 1 as DWORD_PTR;
                if (*pti).rpdesk != IntGetActiveDesktop() {
                    Result = 0;
                    break 'block;
                }
                Status = SafeProbeForWrite(
                    Param as *mut core::ffi::c_void,
                    core::mem::size_of::<POINT>() as SIZE_T,
                    1,
                );
                if !NT_SUCCESS(Status) {
                    Result = 0;
                } else {
                    pptl = Param as PPOINTL;
                    if !crate::copy::write_user(pptl, (*gpsi_per_user()).ptCursor) {
                        SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
                        Result = 0;
                    }
                }
            }
        }

        ONEPARAM_ROUTINE_SETPROCDEFLAYOUT => {
            'block: {
                let ppi: PPROCESSINFO;
                if (Param & LAYOUT_ORIENTATIONMASK as DWORD_PTR) != 0
                    || Param == LAYOUT_LTR as DWORD_PTR
                {
                    ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
                    (*ppi).dwLayout = Param as _;
                    Result = 1 as DWORD_PTR;
                    break 'block;
                }
                EngSetLastError(ERROR_INVALID_PARAMETER);
                Result = 0;
            }
        }

        ONEPARAM_ROUTINE_GETPROCDEFLAYOUT => {
            'block: {
                let ppi: PPROCESSINFO;
                let pdwLayout: PDWORD;
                Result = 1 as DWORD_PTR;

                if PsGetCurrentProcess() as PVOID == gpepCSRSS as PVOID {
                    EngSetLastError(ERROR_INVALID_ACCESS);
                    Result = 0;
                    break 'block;
                }

                ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
                Status = SafeProbeForWrite(
                    Param as *mut core::ffi::c_void,
                    core::mem::size_of::<DWORD>() as SIZE_T,
                    core::mem::align_of::<DWORD>() as ULONG,
                );
                if !NT_SUCCESS(Status) {
                    SetLastNtError(Status);
                    Result = 0;
                } else {
                    pdwLayout = Param as PDWORD;
                    // Validate-only probe (USE_DUMMY_PSEH) + a raw store. The C
                    // had both inside one _SEH2_TRY. user32:win passes an
                    // unmapped Param and this store BUGCHECKs 0x1E -- it is the
                    // fault that was hiding behind NtUserFlashWindowEx's.
                    if !crate::copy::write_user(pdwLayout, (*ppi).dwLayout as DWORD) {
                        SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as NTSTATUS);
                        Result = 0;
                    }
                }
            }
        }

        ONEPARAM_ROUTINE_REPLYMESSAGE => {
            Result = co_MsqReplyMessage(Param as LRESULT) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_MESSAGEBEEP => {
            Result = UserPostMessage(
                hwndSAS,
                WM_LOGONNOTIFY,
                LN_MESSAGE_BEEP as _,
                Param as LPARAM,
            ) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_CREATESYSTEMTHREADS => {
            Result = UserSystemThreadProc(Param as BOOL) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_LOCKFOREGNDWINDOW => {
            Result = IntLockSetForegroundWindow(Param as UINT) as DWORD_PTR;
        }

        ONEPARAM_ROUTINE_ALLOWSETFOREGND => {
            Result = IntAllowSetForegroundWindow(Param as DWORD) as DWORD_PTR;
        }

        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Result = 0;
        }
    }

    UserLeave();

    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallTwoParam(
    Param1: DWORD_PTR,
    Param2: DWORD_PTR,
    Routine: DWORD,
) -> DWORD_PTR {
    let Window: PWND;
    let mut Ret: DWORD_PTR;

    UserEnterExclusive();

    match Routine {
        TWOPARAM_ROUTINE_REDRAWTITLE => {
            Window = UserGetWindowObject(Param1 as HWND);
            Ret = UserPaintCaption(Window, Param2 as INT) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_SETMENUBARHEIGHT => {
            let MenuObject: PMENU = IntGetMenuObject(Param1 as HMENU);
            if MenuObject.is_null() {
                Ret = 0;
            } else {
                if Param2 > 0 {
                    Ret = ((*MenuObject).cyMenu == Param2 as i32 as ULONG) as DWORD_PTR;
                    (*MenuObject).cyMenu = Param2 as i32 as ULONG;
                } else {
                    Ret = (*MenuObject).cyMenu as DWORD_PTR;
                }
                IntReleaseMenuObject(MenuObject);
            }
        }

        TWOPARAM_ROUTINE_SETGUITHRDHANDLE => {
            let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
            Ret = MsqSetStateWindow(pti, Param1 as ULONG, Param2 as HWND) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_ENABLEWINDOW => {
            Ret = IntEnableWindow(Param1 as HWND, Param2 as BOOL) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_SHOWOWNEDPOPUPS => {
            Window = UserGetWindowObject(Param1 as HWND);
            if Window.is_null() {
                Ret = 0;
            } else {
                Ret = IntShowOwnedPopups(Window, Param2 as BOOL) as DWORD_PTR;
            }
        }

        TWOPARAM_ROUTINE_ROS_UPDATEUISTATE => {
            Window = UserGetWindowObject(Param1 as HWND);
            if Window.is_null() {
                Ret = 0;
            } else {
                let wParam: WPARAM = MAKEWPARAM(
                    ((Param2 >> 3) & 0x3) as u32,
                    (Param2 & (UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE) as DWORD_PTR) as u32,
                );
                Ret = UserUpdateUiState(Window, wParam) as DWORD_PTR;
            }
        }

        TWOPARAM_ROUTINE_SWITCHTOTHISWINDOW => {
            let hwnd: HWND = Param1 as HWND;
            let fAltTab: BOOL = Param2 as BOOL;
            Ret = 0;
            Window = UserGetWindowObject(hwnd);
            if !Window.is_null() {
                if !gpqForeground.is_null() && fAltTab == 0 {
                    let pwndActive: PWND = (*gpqForeground).spwndActive;
                    if !pwndActive.is_null()
                        && ((*pwndActive).ExStyle & WS_EX_TOPMOST) == 0
                    {
                        co_WinPosSetWindowPos(
                            pwndActive,
                            HWND_BOTTOM,
                            0,
                            0,
                            0,
                            0,
                            (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS) as UINT,
                        );
                    }

                    UserSetActiveWindow(Window);
                } else {
                    co_IntSetForegroundWindowMouse(Window);

                    if fAltTab != 0 && ((*Window).style & WS_MINIMIZE as DWORD) != 0 {
                        let msg: MSG = MSG {
                            hwnd: UserHMGetHandle(Window) as HWND,
                            message: WM_SYSCOMMAND,
                            wParam: SC_RESTORE as WPARAM,
                            lParam: 0,
                            ..core::mem::zeroed()
                        };
                        MsqPostMessage(
                            head_pti(&(*Window).head),
                            &msg,
                            0u8,
                            QS_POSTMESSAGE,
                            0,
                            0,
                        );
                    }
                }
            }
        }

        TWOPARAM_ROUTINE_SETCARETPOS => {
            Ret = co_IntSetCaretPos(Param1 as i32, Param2 as i32) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_REGISTERLOGONPROCESS => {
            Ret = co_IntRegisterLogonProcess(Param1 as HANDLE, Param2 as BOOL) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_SETCURSORPOS => {
            Ret = UserSetCursorPos(Param1 as i32, Param2 as i32, 0, 0, 0) as DWORD_PTR;
        }

        TWOPARAM_ROUTINE_UNHOOKWINDOWSHOOK => {
            Ret = IntUnhookWindowsHook(Param1 as i32, int_as_hookproc(Param2)) as DWORD_PTR;
        }

        _ => {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            Ret = 0;
        }
    }

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallHwndLock(hWnd: HWND, Routine: DWORD) -> BOOL {
    let mut Ret: BOOL = 0;
    let Window: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            Ret = 0;
            break 'exit;
        }

        UserRefObjectCo(Window as PVOID, &mut Ref);

        match Routine {
            HWNDLOCK_ROUTINE_ARRANGEICONICWINDOWS => {
                co_WinPosArrangeIconicWindows(Window);
            }

            HWNDLOCK_ROUTINE_DRAWMENUBAR => {
                Ret = 1;
                if ((*Window).style & (WS_CHILD | WS_POPUP)) != WS_CHILD {
                    co_WinPosSetWindowPos(
                        Window,
                        HWND_DESKTOP,
                        0, 0, 0, 0,
                        SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                    );
                }
            }

            HWNDLOCK_ROUTINE_REDRAWFRAME => {
                co_WinPosSetWindowPos(
                    Window,
                    HWND_DESKTOP,
                    0, 0, 0, 0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
                Ret = 1;
            }

            HWNDLOCK_ROUTINE_REDRAWFRAMEANDHOOK => {
                co_WinPosSetWindowPos(
                    Window,
                    HWND_DESKTOP,
                    0, 0, 0, 0,
                    SWP_NOSIZE | SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
                if (*Window).spwndOwner.is_null() && IntGetParent(Window).is_null() {
                    co_IntShellHookNotify(HSHELL_REDRAW as _, hWnd as WPARAM, 0);
                }
                Ret = 1;
            }

            HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOW => {
                Ret = co_IntSetForegroundWindow(Window);
            }

            HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOWMOUSE => {
                Ret = co_IntSetForegroundWindowMouse(Window);
            }

            HWNDLOCK_ROUTINE_UPDATEWINDOW => {
                co_IntUpdateWindows(Window, RDW_ALLCHILDREN as _, 0);
                Ret = 1;
            }

            HWNDLOCK_ROUTINE_CHECKIMESHOWSTATUSINTHRD => {
                IntCheckImeShowStatusInThread(Window);
            }

            _ => {}
        }

        UserDerefObjectCo(Window as PVOID);
    }

    UserLeave();

    Ret
}

const HWND_DESKTOP: HWND = core::ptr::null_mut();
#[no_mangle]
pub unsafe extern "system" fn NtUserCallHwndOpt(
    hWnd: HWND,
    Routine: DWORD,
) -> HWND {
    match Routine {
        HWNDOPT_ROUTINE_SETPROGMANWINDOW => {
            (*(*GetW32ThreadInfo()).pDeskInfo).hProgmanWindow = hWnd;
        }
        HWNDOPT_ROUTINE_SETTASKMANWINDOW => {
            (*(*GetW32ThreadInfo()).pDeskInfo).hTaskManWindow = hWnd;
        }
        _ => {}
    }

    hWnd
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallHwnd(
    hWnd: HWND,
    Routine: DWORD,
) -> DWORD_PTR {

    match Routine {
        HWND_ROUTINE_GETWNDCONTEXTHLPID => {
            UserEnterShared();

            let Window: PWND = UserGetWindowObject(hWnd);
            if Window.is_null() {
                UserLeave();
                return 0;
            }

            let HelpId = HandleToUlong(UserGetProp(
                Window,
                (*gpsi).atomContextHelpIdProp,
                1u8,
            )) as DWORD_PTR;

            UserLeave();
            HelpId
        }

        HWND_ROUTINE_REGISTERSHELLHOOKWINDOW => {
            if IntIsWindow(hWnd) != 0 {
                return IntRegisterShellHookWindow(hWnd) as DWORD_PTR;
            }
            0
        }

        HWND_ROUTINE_DEREGISTERSHELLHOOKWINDOW => {
            if IntIsWindow(hWnd) != 0 {
                return IntDeRegisterShellHookWindow(hWnd) as DWORD_PTR;
            }
            0
        }

        HWND_ROUTINE_SETMSGBOX => {
            UserEnterExclusive();
            let Window: PWND = UserGetWindowObject(hWnd);
            if !Window.is_null() {
                (*Window).state |= WNDS_MSGBOX;
            }
            UserLeave();
            0
        }

        HWND_ROUTINE_DWP_GETENABLEDPOPUP => {
            UserEnterShared();
            let mut pWnd: PWND = UserGetWindowObject(hWnd);
            if !pWnd.is_null() {
                pWnd = DWP_GetEnabledPopup(pWnd);
            }
            UserLeave();
            pWnd as DWORD_PTR
        }

        _ => 0,
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallHwndParam(
    hWnd: HWND,
    Param: DWORD_PTR,
    Routine: DWORD,
) -> DWORD {

    match Routine {
        HWNDPARAM_ROUTINE_KILLSYSTEMTIMER => {
            UserEnterExclusive();
            let ret = IntKillTimer(UserGetWindowObject(hWnd), Param as UINT_PTR, 1);
            UserLeave();
            return ret as DWORD;
        }

        HWNDPARAM_ROUTINE_SETWNDCONTEXTHLPID => {
            UserEnterExclusive();
            let Window = UserGetWindowObject(hWnd);
            if Window.is_null() {
                UserLeave();
                return 0;
            }

            if Param != 0 {
                UserSetProp(
                    Window,
                    (*gpsi).atomContextHelpIdProp,
                    Param as HANDLE,
                    1u8,
                );
            } else {
                UserRemoveProp(Window, (*gpsi).atomContextHelpIdProp, 1u8);
            }

            UserLeave();
            return 1;
        }

        HWNDPARAM_ROUTINE_SETDIALOGPOINTER => {
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

            UserEnterExclusive();

            let pWnd = UserGetWindowObject(hWnd);
            if pWnd.is_null() {
                UserLeave();
                return 0;
            }
            UserRefObjectCo(pWnd as PVOID, &mut Ref);

            if (*head_pti(&(*pWnd).head)).ppi as PVOID == PsGetCurrentProcessWin32Process()
                && (*pWnd).cbwndExtra >= DLGWINDOWEXTRA as ULONG
                && ((*pWnd).state & WNDS_SERVERSIDEWINDOWPROC) == 0
            {
                (*pWnd).DialogPointer = Param as PVOID;
                if Param != 0 {
                    if (*pWnd).fnid == 0 {
                        (*pWnd).fnid = FNID_DIALOG as _;
                    }
                    (*pWnd).state |= WNDS_DIALOGWINDOW;
                } else {
                    (*pWnd).fnid |= FNID_DESTROY as DWORD;
                    (*pWnd).state &= !WNDS_DIALOGWINDOW;
                }
            }

            UserDerefObjectCo(pWnd as PVOID);
            UserLeave();
            return 0;
        }

        HWNDPARAM_ROUTINE_ROS_NOTIFYWINEVENT => {
            UserEnterExclusive();
            let pne = Param as PNOTIFYEVENT;
            let pWnd = if !hWnd.is_null() {
                UserGetWindowObject(hWnd)
            } else {
                core::ptr::null_mut()
            };
            IntNotifyWinEvent(
                (*pne).event,
                pWnd,
                (*pne).idObject,
                (*pne).idChild,
                (*pne).flags,
            );
            UserLeave();
            return 0;
        }

        HWNDPARAM_ROUTINE_CLEARWINDOWSTATE => {
            UserEnterExclusive();
            let pWnd = UserGetWindowObject(hWnd);
            if !pWnd.is_null() {
                IntClearWindowState(pWnd, Param as UINT);
            }
            UserLeave();
            return 0;
        }

        HWNDPARAM_ROUTINE_SETWINDOWSTATE => {
            UserEnterExclusive();
            let pWnd = UserGetWindowObject(hWnd);
            if !pWnd.is_null() {
                IntSetWindowState(pWnd, Param as UINT);
            }
            UserLeave();
            return 0;
        }

        _ => {}
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCallHwndParamLock(
    hWnd: HWND,
    Param: DWORD_PTR,
    Routine: DWORD,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let Window: PWND;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    UserEnterExclusive();

    'exit: {
        Window = UserGetWindowObject(hWnd);
        if Window.is_null() {
            Ret = 0;
            break 'exit;
        }

        UserRefObjectCo(Window as PVOID, core::ptr::addr_of_mut!(Ref));

        match Routine {
            TWOPARAM_ROUTINE_IMESHOWSTATUSCHANGE => {
                Ret = IntBroadcastImeShowStatusChange(Window, (Param != 0) as BOOL) as DWORD;
            }
            TWOPARAM_ROUTINE_VALIDATERGN => {
                let Rgn: PREGION = REGION_LockRgn(Param as HRGN);
                Ret = co_UserRedrawWindow(Window, core::ptr::null_mut(), Rgn, RDW_VALIDATE) as DWORD;
                if !Rgn.is_null() {
                    REGION_UnlockRgn(Rgn);
                }
            }
            _ => {}
        }

        UserDerefObjectCo(Window as PVOID);
    }

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
//  * PURPOSE:          NtUserCallXxx call stubs
//  * FILE:             win32ss/user/ntuser/simplecall.c
//  * PROGRAMERS:       Ge van Geldorp (ge@gse.nl)
//  *                   Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// DBG_DEFAULT_CHANNEL(UserMisc);
//
// /* Registered logon process ID */
// HANDLE gpidLogon = 0;
//
// BOOL FASTCALL
// co_IntRegisterLogonProcess(HANDLE ProcessId, BOOL Register)
// {
//     NTSTATUS Status;
//     PEPROCESS Process;
//
//     Status = PsLookupProcessByProcessId(ProcessId, &Process);
//     if (!NT_SUCCESS(Status))
//     {
//         EngSetLastError(RtlNtStatusToDosError(Status));
//         return FALSE;
//     }
//
//     ProcessId = Process->UniqueProcessId;
//
//     ObDereferenceObject(Process);
//
//     if (Register)
//     {
//         /* Register the logon process */
//         if (gpidLogon != 0)
//             return FALSE;
//
//         gpidLogon = ProcessId;
//     }
//     else
//     {
//         /* Deregister the logon process */
//         if (gpidLogon != ProcessId)
//             return FALSE;
//
//         gpidLogon = 0;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// DWORD_PTR
// __stdcall
// NtUserCallNoParam(DWORD Routine)
// {
//     DWORD_PTR Result = 0;
//
//     TRACE("Enter NtUserCallNoParam\n");
//     UserEnterExclusive();
//
//     switch (Routine)
//     {
//         case NOPARAM_ROUTINE_CREATEMENU:
//             Result = (DWORD_PTR)UserCreateMenu(GetW32ThreadInfo()->rpdesk, FALSE);
//             break;
//
//         case NOPARAM_ROUTINE_CREATEMENUPOPUP:
//             Result = (DWORD_PTR)UserCreateMenu(GetW32ThreadInfo()->rpdesk, TRUE);
//             break;
//
//         case NOPARAM_ROUTINE_DESTROY_CARET:
//             Result = (DWORD_PTR)co_IntDestroyCaret(PsGetCurrentThread()->Tcb.Win32Thread);
//             break;
//
//         case NOPARAM_ROUTINE_INIT_MESSAGE_PUMP:
//             Result = (DWORD_PTR)IntInitMessagePumpHook();
//             break;
//
//         case NOPARAM_ROUTINE_UNINIT_MESSAGE_PUMP:
//             Result = (DWORD_PTR)IntUninitMessagePumpHook();
//             break;
//
//         case NOPARAM_ROUTINE_MSQCLEARWAKEMASK:
//             Result = (DWORD_PTR)IntMsqClearWakeMask();
//             break;
//
//         case NOPARAM_ROUTINE_GETMSESSAGEPOS:
//         {
//             PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//             Result = (DWORD_PTR)MAKELONG(pti->ptLast.x, pti->ptLast.y);
//             break;
//         }
//
//         case NOPARAM_ROUTINE_RELEASECAPTURE:
//             Result = (DWORD_PTR)IntReleaseCapture();
//             break;
//
//         case NOPARAM_ROUTINE_LOADUSERAPIHOOK:
//             Result = UserLoadApiHook();
//             break;
//
//         case NOPARAM_ROUTINE_ZAPACTIVEANDFOUS:
//         {
//             PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//             TRACE("Zapping the Active and Focus window out of the Queue!\n");
//             pti->MessageQueue->spwndFocus = NULL;
//             pti->MessageQueue->spwndActive = NULL;
//             Result = 0;
//             break;
//         }
//
//         case NOPARAM_ROUTINE_GETIMESHOWSTATUS:
//             Result = !!gfIMEShowStatus;
//             break;
//
//         /* this is a ReactOS only case and is needed for gui-on-demand */
//         case NOPARAM_ROUTINE_ISCONSOLEMODE:
//             Result = (ScreenDeviceContext == NULL);
//             break;
//
//         case NOPARAM_ROUTINE_UPDATEPERUSERIMMENABLING:
//             if (UserIsIMMEnabled())
//                 gpsi->dwSRVIFlags |= SRVINFO_IMM32;
//             else
//                 gpsi->dwSRVIFlags &= ~SRVINFO_IMM32;
//
//             Result = TRUE; // Always return TRUE.
//             break;
//
//         default:
//             ERR("Calling invalid routine number 0x%x in NtUserCallNoParam\n", Routine);
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             break;
//     }
//
//     TRACE("Leave NtUserCallNoParam, ret=%p\n",(PVOID)Result);
//     UserLeave();
//
//     return Result;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD_PTR
// __stdcall
// NtUserCallOneParam(
//     DWORD_PTR Param,
//     DWORD Routine)
// {
//     DWORD_PTR Result;
//     NTSTATUS Status;
//
//     TRACE("Enter NtUserCallOneParam\n");
//
//     UserEnterExclusive();
//
//     switch (Routine)
//     {
//         case ONEPARAM_ROUTINE_POSTQUITMESSAGE:
//         {
//             PTHREADINFO pti;
//             pti = PsGetCurrentThreadWin32Thread();
//             MsqPostQuitMessage(pti, Param);
//             Result = TRUE;
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_BEGINDEFERWNDPOS:
//         {
//             PSMWP psmwp;
//             HDWP hDwp = NULL;
//             INT count = (INT)Param;
//
//             if (count < 0)
//             {
//                 EngSetLastError(ERROR_INVALID_PARAMETER);
//                 Result = 0;
//                 break;
//             }
//
//             /* Windows allows zero count, in which case it allocates context for 8 moves */
//             if (count == 0) count = 8;
//
//             psmwp = (PSMWP)UserCreateObject(gHandleTable,
//                                             NULL,
//                                             NULL,
//                                             (PHANDLE)&hDwp,
//                                             TYPE_SETWINDOWPOS,
//                                             sizeof(SMWP));
//             if (!psmwp)
//             {
//                 Result = 0;
//                 break;
//             }
//
//             psmwp->acvr = ExAllocatePoolWithTag(PagedPool, count * sizeof(CVR), USERTAG_SWP);
//             if (!psmwp->acvr)
//             {
//                 UserDeleteObject(hDwp, TYPE_SETWINDOWPOS);
//                 Result = 0;
//                 break;
//             }
//
//             RtlZeroMemory(psmwp->acvr, count * sizeof(CVR));
//             psmwp->bHandle = TRUE;
//             psmwp->ccvr = 0;          // actualCount
//             psmwp->ccvrAlloc = count; // suggestedCount
//             Result = (DWORD_PTR)hDwp;
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_SHOWCURSOR:
//             Result = (DWORD_PTR)UserShowCursor((BOOL)Param);
//             break;
//
//         case ONEPARAM_ROUTINE_GETDESKTOPMAPPING:
//         {
//             PTHREADINFO ti;
//             ti = GetW32ThreadInfo();
//             if (ti != NULL)
//             {
//                 /* Try convert the pointer to a user mode pointer if the desktop is
//                    mapped into the process */
//                 Result = (DWORD_PTR)DesktopHeapAddressToUser((PVOID)Param);
//             }
//             else
//             {
//                 Result = 0;
//             }
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_WINDOWFROMDC:
//             Result = (DWORD_PTR)IntWindowFromDC((HDC)Param);
//             break;
//
//         case ONEPARAM_ROUTINE_SWAPMOUSEBUTTON:
//         {
//             Result = gspv.bMouseBtnSwap;
//             gspv.bMouseBtnSwap = Param ? TRUE : FALSE;
//             gpsi->aiSysMet[SM_SWAPBUTTON] = gspv.bMouseBtnSwap;
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_SETCARETBLINKTIME:
//             Result = (DWORD_PTR)IntSetCaretBlinkTime((UINT)Param);
//             break;
//
//         case ONEPARAM_ROUTINE_SETMESSAGEEXTRAINFO:
//             Result = (DWORD_PTR)MsqSetMessageExtraInfo((LPARAM)Param);
//             break;
//
//         case ONEPARAM_ROUTINE_CREATEEMPTYCUROBJECT:
//         {
//             if (!(Result = (DWORD_PTR)IntCreateCurIconHandle((DWORD)Param)))
//             {
//                 EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//                 Result = 0;
//             }
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_ENABLEPROCWNDGHSTING:
//         {
//             BOOL Enable;
//             PPROCESSINFO Process = PsGetCurrentProcessWin32Process();
//
//             if (Process != NULL)
//             {
//                 Enable = (BOOL)(Param != 0);
//
//                 if (Enable)
//                 {
//                     Process->W32PF_flags &= ~W32PF_NOWINDOWGHOSTING;
//                 }
//                 else
//                 {
//                     Process->W32PF_flags |= W32PF_NOWINDOWGHOSTING;
//                 }
//
//                 Result = TRUE;
//                 break;
//             }
//
//             Result = FALSE;
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_GETINPUTEVENT:
//             Result = (DWORD_PTR)IntMsqSetWakeMask(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_GETKEYBOARDTYPE:
//             Result = UserGetKeyboardType(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_GETKEYBOARDLAYOUT:
//             Result = (DWORD_PTR)UserGetKeyboardLayout(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_RELEASEDC:
//             Result = UserReleaseDC(NULL, (HDC) Param, FALSE);
//             break;
//
//         case ONEPARAM_ROUTINE_REALIZEPALETTE:
//             Result = UserRealizePalette((HDC) Param);
//             break;
//
//         case ONEPARAM_ROUTINE_GETQUEUESTATUS:
//         {
//             Result = IntGetQueueStatus((DWORD)Param);
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_ENUMCLIPBOARDFORMATS:
//             /* FIXME: Should use UserEnterShared */
//             Result = UserEnumClipboardFormats(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_GETCURSORPOS:
//         {
//             PPOINTL pptl;
//             PTHREADINFO pti = PsGetCurrentThreadWin32Thread();
//             Result = TRUE;
//             if (pti->rpdesk != IntGetActiveDesktop())
//             {
//                 Result = FALSE;
//                 break;
//             }
//             Status = SafeProbeForWrite((POINT*)Param, sizeof(POINT), 1);
//             if (!NT_SUCCESS(Status))
//             {
//                 Result = FALSE;
//             }
//             else
//             {
//                 pptl = (PPOINTL)Param;
//                 *pptl = gpsi->ptCursor;
//             }
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_SETPROCDEFLAYOUT:
//         {
//             PPROCESSINFO ppi;
//             if (Param & LAYOUT_ORIENTATIONMASK || Param == LAYOUT_LTR)
//             {
//                 ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
//                 ppi->dwLayout = Param;
//                 Result = TRUE;
//                 break;
//             }
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             Result = FALSE;
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_GETPROCDEFLAYOUT:
//         {
//             PPROCESSINFO ppi;
//             PDWORD pdwLayout;
//             Result = TRUE;
//
//             if (PsGetCurrentProcess() == gpepCSRSS)
//             {
//                 EngSetLastError(ERROR_INVALID_ACCESS);
//                 Result = FALSE;
//                 break;
//             }
//
//             ppi = PsGetCurrentProcessWin32Process();
//             Status = SafeProbeForWrite((PDWORD)Param, sizeof(DWORD), __alignof(DWORD));
//             if (!NT_SUCCESS(Status))
//             {
//                 SetLastNtError(Status);
//                 Result = FALSE;
//             }
//             else
//             {
//                 pdwLayout = (PDWORD)Param;
//                 *pdwLayout = ppi->dwLayout;
//             }
//             break;
//         }
//
//         case ONEPARAM_ROUTINE_REPLYMESSAGE:
//             Result = co_MsqReplyMessage((LRESULT)Param);
//             break;
//
//         case ONEPARAM_ROUTINE_MESSAGEBEEP:
//             /* TODO: Implement sound sentry */
//             Result = UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_MESSAGE_BEEP, Param);
//            break;
//
//         case ONEPARAM_ROUTINE_CREATESYSTEMTHREADS:
//             Result = UserSystemThreadProc(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_LOCKFOREGNDWINDOW:
//             Result = (DWORD_PTR)IntLockSetForegroundWindow(Param);
//             break;
//
//         case ONEPARAM_ROUTINE_ALLOWSETFOREGND:
//             Result = (DWORD_PTR)IntAllowSetForegroundWindow(Param);
//             break;
//
//         default:
//             ERR("Calling invalid routine number 0x%x in NtUserCallOneParam(), Param=0x%x\n",
//                 Routine, Param);
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             Result = 0;
//             break;
//     }
//
//    TRACE("Leave NtUserCallOneParam, ret=%p\n", (PVOID)Result);
//    UserLeave();
//
//    return Result;
// }
//
//
//
// /*
//  * @implemented
//  */
// DWORD_PTR
// __stdcall
// NtUserCallTwoParam(
//     DWORD_PTR Param1,
//     DWORD_PTR Param2,
//     DWORD Routine)
// {
//     PWND Window;
//     DWORD_PTR Ret;
//     TRACE("Enter NtUserCallTwoParam\n");
//     UserEnterExclusive();
//
//     switch (Routine)
//     {
//         case TWOPARAM_ROUTINE_REDRAWTITLE:
//         {
//             Window = UserGetWindowObject((HWND)Param1);
//             Ret = (DWORD_PTR)UserPaintCaption(Window, (INT)Param2);
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_SETMENUBARHEIGHT:
//         {
//             PMENU MenuObject = IntGetMenuObject((HMENU)Param1);
//             if (!MenuObject)
//             {
//                 Ret = 0;
//                 break;
//             }
//
//             if (Param2 > 0)
//             {
//                 Ret = (MenuObject->cyMenu == (int)Param2);
//                 MenuObject->cyMenu = (int)Param2;
//             }
//             else
//                 Ret = (DWORD_PTR)MenuObject->cyMenu;
//             IntReleaseMenuObject(MenuObject);
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_SETGUITHRDHANDLE:
//         {
//             PTHREADINFO pti = (PTHREADINFO)PsGetCurrentThreadWin32Thread();
//             ASSERT(pti->MessageQueue);
//             Ret = (DWORD_PTR)MsqSetStateWindow(pti, (ULONG)Param1, (HWND)Param2);
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_ENABLEWINDOW:
//             Ret = IntEnableWindow((HWND)Param1, (BOOL)Param2);
//             break;
//
//         case TWOPARAM_ROUTINE_SHOWOWNEDPOPUPS:
//         {
//             Window = UserGetWindowObject((HWND)Param1);
//             if (!Window)
//             {
//                 Ret = 0;
//                 break;
//             }
//
//             Ret = (DWORD_PTR)IntShowOwnedPopups(Window, (BOOL)Param2);
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_ROS_UPDATEUISTATE:
//         {
//             WPARAM wParam;
//             Window = UserGetWindowObject((HWND)Param1);
//             if (!Window)
//             {
//                 Ret = 0;
//                 break;
//             }
//
//             /* Unpack wParam */
//             wParam = MAKEWPARAM((Param2 >> 3) & 0x3,
//                                 Param2 & (UISF_HIDEFOCUS | UISF_HIDEACCEL | UISF_ACTIVE));
//
//             Ret = UserUpdateUiState(Window, wParam);
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_SWITCHTOTHISWINDOW:
//         {
//             HWND hwnd = (HWND)Param1;
//             BOOL fAltTab = (BOOL)Param2;
//             Ret = 0;
//             Window = UserGetWindowObject(hwnd);
//             if (!Window)
//                 break;
//
//             if (gpqForeground && !fAltTab)
//             {
//                 PWND pwndActive = gpqForeground->spwndActive;
//                 if (pwndActive && !(pwndActive->ExStyle & WS_EX_TOPMOST))
//                 {
//                     co_WinPosSetWindowPos(pwndActive, HWND_BOTTOM, 0, 0, 0, 0,
//                                           SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE |
//                                           SWP_ASYNCWINDOWPOS);
//                 }
//
//                 UserSetActiveWindow(Window);
//                 break;
//             }
//
//             co_IntSetForegroundWindowMouse(Window);
//
//             if (fAltTab && (Window->style & WS_MINIMIZE))
//             {
//                 MSG msg = { UserHMGetHandle(Window), WM_SYSCOMMAND, SC_RESTORE, 0 };
//                 MsqPostMessage(Window->head.pti, &msg, FALSE, QS_POSTMESSAGE, 0, 0);
//             }
//             break;
//         }
//
//         case TWOPARAM_ROUTINE_SETCARETPOS:
//             Ret = (DWORD_PTR)co_IntSetCaretPos((int)Param1, (int)Param2);
//             break;
//
//         case TWOPARAM_ROUTINE_REGISTERLOGONPROCESS:
//             Ret = (DWORD_PTR)co_IntRegisterLogonProcess((HANDLE)Param1, (BOOL)Param2);
//             break;
//
//         case TWOPARAM_ROUTINE_SETCURSORPOS:
//             Ret = (DWORD_PTR)UserSetCursorPos((int)Param1, (int)Param2, 0, 0, FALSE);
//             break;
//
//         case TWOPARAM_ROUTINE_UNHOOKWINDOWSHOOK:
//             Ret = IntUnhookWindowsHook((int)Param1, (HOOKPROC)Param2);
//             break;
//
//         default:
//             ERR("Calling invalid routine number 0x%x in NtUserCallTwoParam(), Param1=0x%x Parm2=0x%x\n",
//                 Routine, Param1, Param2);
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             Ret = 0;
//     }
//
//
//     TRACE("Leave NtUserCallTwoParam, ret=%p\n", (PVOID)Ret);
//     UserLeave();
//
//     return Ret;
// }
//
//
// /*
//  * @unimplemented
//  */
// BOOL
// __stdcall
// NtUserCallHwndLock(
//     HWND hWnd,
//     DWORD Routine)
// {
//     BOOL Ret = FALSE;
//     PWND Window;
//     USER_REFERENCE_ENTRY Ref;
//
//     TRACE("Enter NtUserCallHwndLock\n");
//     UserEnterExclusive();
//
//     if (!(Window = UserGetWindowObject(hWnd)))
//     {
//         Ret = FALSE;
//         goto Exit;
//     }
//
//     UserRefObjectCo(Window, &Ref);
//
//     /* FIXME: Routine can be 0x53 - 0x5E */
//     switch (Routine)
//     {
//         case HWNDLOCK_ROUTINE_ARRANGEICONICWINDOWS:
//             co_WinPosArrangeIconicWindows(Window);
//             break;
//
//         case HWNDLOCK_ROUTINE_DRAWMENUBAR:
//         {
//             TRACE("HWNDLOCK_ROUTINE_DRAWMENUBAR\n");
//             Ret = TRUE;
//             if ((Window->style & (WS_CHILD | WS_POPUP)) != WS_CHILD)
//                 co_WinPosSetWindowPos(Window,
//                                       HWND_DESKTOP,
//                                       0, 0, 0, 0,
//                                       SWP_NOSIZE |
//                                       SWP_NOMOVE |
//                                       SWP_NOZORDER |
//                                       SWP_NOACTIVATE |
//                                       SWP_FRAMECHANGED);
//             break;
//         }
//
//         case HWNDLOCK_ROUTINE_REDRAWFRAME:
//             co_WinPosSetWindowPos(Window,
//                                   HWND_DESKTOP,
//                                   0, 0, 0, 0,
//                                   SWP_NOSIZE |
//                                   SWP_NOMOVE |
//                                   SWP_NOZORDER |
//                                   SWP_NOACTIVATE |
//                                   SWP_FRAMECHANGED);
//             Ret = TRUE;
//             break;
//
//         case HWNDLOCK_ROUTINE_REDRAWFRAMEANDHOOK:
//             co_WinPosSetWindowPos(Window,
//                                   HWND_DESKTOP,
//                                   0, 0, 0, 0,
//                                   SWP_NOSIZE |
//                                   SWP_NOMOVE |
//                                   SWP_NOZORDER |
//                                   SWP_NOACTIVATE |
//                                   SWP_FRAMECHANGED);
//             if (!Window->spwndOwner && !IntGetParent(Window))
//             {
//                 co_IntShellHookNotify(HSHELL_REDRAW, (WPARAM)hWnd, FALSE); // FIXME Flashing?
//             }
//             Ret = TRUE;
//             break;
//
//         case HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOW:
//             TRACE("co_IntSetForegroundWindow 1 0x%p\n", hWnd);
//             Ret = co_IntSetForegroundWindow(Window);
//             TRACE("co_IntSetForegroundWindow 2 0x%p\n", hWnd);
//             break;
//
//         case HWNDLOCK_ROUTINE_SETFOREGROUNDWINDOWMOUSE:
//             TRACE("co_IntSetForegroundWindow M 1 0x%p\n", hWnd);
//             Ret = co_IntSetForegroundWindowMouse(Window);
//             TRACE("co_IntSetForegroundWindow M 2 0x%p\n", hWnd);
//             break;
//
//         case HWNDLOCK_ROUTINE_UPDATEWINDOW:
//             co_IntUpdateWindows(Window, RDW_ALLCHILDREN, FALSE);
//             Ret = TRUE;
//             break;
//
//         case HWNDLOCK_ROUTINE_CHECKIMESHOWSTATUSINTHRD:
//             IntCheckImeShowStatusInThread(Window);
//             break;
//     }
//
//     UserDerefObjectCo(Window as PVOID);
//
// Exit:
//     TRACE("Leave NtUserCallHwndLock, ret=%u\n", Ret);
//     UserLeave();
//
//     return Ret;
// }
//
// /*
//  * @unimplemented
//  */
// HWND
// __stdcall
// NtUserCallHwndOpt(
//     HWND hWnd,
//     DWORD Routine)
// {
//     switch (Routine)
//     {
//         case HWNDOPT_ROUTINE_SETPROGMANWINDOW:
//             GetW32ThreadInfo()->pDeskInfo->hProgmanWindow = hWnd;
//             break;
//
//         case HWNDOPT_ROUTINE_SETTASKMANWINDOW:
//             GetW32ThreadInfo()->pDeskInfo->hTaskManWindow = hWnd;
//             break;
//     }
//
//     return hWnd;
// }
//
// DWORD_PTR
// __stdcall
// NtUserCallHwnd(
//     HWND hWnd,
//     DWORD Routine)
// {
//     switch (Routine)
//     {
//         case HWND_ROUTINE_GETWNDCONTEXTHLPID:
//         {
//             PWND Window;
//             DWORD HelpId;
//
//             UserEnterShared();
//
//             if (!(Window = UserGetWindowObject(hWnd)))
//             {
//                 UserLeave();
//                 return 0;
//             }
//
//             HelpId = HandleToUlong(UserGetProp(Window, gpsi->atomContextHelpIdProp, TRUE));
//
//             UserLeave();
//             return HelpId;
//         }
//
//         case HWND_ROUTINE_REGISTERSHELLHOOKWINDOW:
//             if (IntIsWindow(hWnd))
//                 return IntRegisterShellHookWindow(hWnd);
//             return FALSE;
//             break;
//
//         case HWND_ROUTINE_DEREGISTERSHELLHOOKWINDOW:
//             if (IntIsWindow(hWnd))
//                 return IntDeRegisterShellHookWindow(hWnd);
//             return FALSE;
//
//         case HWND_ROUTINE_SETMSGBOX:
//         {
//             PWND Window;
//             UserEnterExclusive();
//             if ((Window = UserGetWindowObject(hWnd)))
//             {
//                 Window->state |= WNDS_MSGBOX;
//             }
//             UserLeave();
//             return FALSE;
//         }
//
//         case HWND_ROUTINE_DWP_GETENABLEDPOPUP:
//         {
//             PWND pWnd;
//             UserEnterShared();
//             pWnd = UserGetWindowObject(hWnd);
//             if (pWnd)
//                 pWnd = DWP_GetEnabledPopup(pWnd);
//             UserLeave();
//             return (DWORD_PTR)pWnd;
//         }
//     }
//
//     STUB;
//
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserCallHwndParam(
//     HWND hWnd,
//     DWORD_PTR Param,
//     DWORD Routine)
// {
//
//     switch (Routine)
//     {
//         case HWNDPARAM_ROUTINE_KILLSYSTEMTIMER:
//         {
//             DWORD ret;
//
//             UserEnterExclusive();
//             ret = IntKillTimer(UserGetWindowObject(hWnd), (UINT_PTR)Param, TRUE);
//             UserLeave();
//             return ret;
//         }
//
//         case HWNDPARAM_ROUTINE_SETWNDCONTEXTHLPID:
//         {
//             PWND Window;
//
//             UserEnterExclusive();
//             if (!(Window = UserGetWindowObject(hWnd)))
//             {
//                 UserLeave();
//                 return FALSE;
//             }
//
//             if (Param)
//                 UserSetProp(Window, gpsi->atomContextHelpIdProp, (HANDLE)Param, TRUE);
//             else
//                 UserRemoveProp(Window, gpsi->atomContextHelpIdProp, TRUE);
//
//             UserLeave();
//             return TRUE;
//         }
//
//         case HWNDPARAM_ROUTINE_SETDIALOGPOINTER:
//         {
//             PWND pWnd;
//             USER_REFERENCE_ENTRY Ref;
//
//             UserEnterExclusive();
//
//             if (!(pWnd = UserGetWindowObject(hWnd)))
//             {
//                 UserLeave();
//                 return 0;
//             }
//             UserRefObjectCo(pWnd, &Ref);
//
//             if (pWnd->head.pti->ppi == PsGetCurrentProcessWin32Process() &&
//                 pWnd->cbwndExtra >= DLGWINDOWEXTRA &&
//                 !(pWnd->state & WNDS_SERVERSIDEWINDOWPROC))
//             {
//                 pWnd->DialogPointer = (PVOID)Param;
//                 if (Param)
//                 {
//                     if (!pWnd->fnid) pWnd->fnid = FNID_DIALOG;
//                     pWnd->state |= WNDS_DIALOGWINDOW;
//                 }
//                 else
//                 {
//                     pWnd->fnid |= FNID_DESTROY;
//                     pWnd->state &= ~WNDS_DIALOGWINDOW;
//                 }
//             }
//
//             UserDerefObjectCo(pWnd as PVOID);
//             UserLeave();
//             return 0;
//         }
//
//         case HWNDPARAM_ROUTINE_ROS_NOTIFYWINEVENT:
//         {
//             PWND pWnd;
//             PNOTIFYEVENT pne;
//             UserEnterExclusive();
//             pne = (PNOTIFYEVENT)Param;
//             if (hWnd)
//                 pWnd = UserGetWindowObject(hWnd);
//             else
//                 pWnd = NULL;
//             IntNotifyWinEvent(pne->event, pWnd, pne->idObject, pne->idChild, pne->flags);
//             UserLeave();
//             return 0;
//         }
//
//         case HWNDPARAM_ROUTINE_CLEARWINDOWSTATE:
//         {
//             PWND pWnd;
//             UserEnterExclusive();
//             pWnd = UserGetWindowObject(hWnd);
//             if (pWnd) IntClearWindowState(pWnd, (UINT)Param);
//             UserLeave();
//             return 0;
//         }
//
//         case HWNDPARAM_ROUTINE_SETWINDOWSTATE:
//         {
//             PWND pWnd;
//             UserEnterExclusive();
//             pWnd = UserGetWindowObject(hWnd);
//             if (pWnd) IntSetWindowState(pWnd, (UINT)Param);
//             UserLeave();
//             return 0;
//         }
//     }
//
//     STUB;
//
//     return 0;
// }
//
// DWORD
// __stdcall
// NtUserCallHwndParamLock(
//     HWND hWnd,
//     DWORD_PTR Param,
//     DWORD Routine)
// {
//     DWORD Ret = FALSE;
//     PWND Window;
//     USER_REFERENCE_ENTRY Ref;
//
//     TRACE("Enter NtUserCallHwndParamLock\n");
//     UserEnterExclusive();
//
//     if (!(Window = UserGetWindowObject(hWnd)))
//     {
//         Ret = FALSE;
//         goto Exit;
//     }
//
//     UserRefObjectCo(Window, &Ref);
//
//     switch (Routine)
//     {
//         case TWOPARAM_ROUTINE_IMESHOWSTATUSCHANGE:
//             Ret = IntBroadcastImeShowStatusChange(Window, !!Param);
//             break;
//
//         case TWOPARAM_ROUTINE_VALIDATERGN:
//         {
//             PREGION Rgn = REGION_LockRgn((HRGN)Param);
//             Ret = (DWORD)co_UserRedrawWindow(Window, NULL, Rgn, RDW_VALIDATE);
//             if (Rgn) REGION_UnlockRgn(Rgn);
//             break;
//         }
//     }
//
//     UserDerefObjectCo(Window as PVOID);
//
// Exit:
//
//     TRACE("Leave NtUserCallHwndParamLock, ret=%lu\n", Ret);
//     UserLeave();
//
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
