use win32k_sys::win32k::*;
// ntdll Rtl*/Zw* + the NtCurrentTeb() TEB inline come from the user32 substrate
// (crate::internal), NOT from ntkrnl_sys (which is a kernel-only crate and not a
// dependency of this usermode DLL).
use crate::internal::{
    NtCurrentTeb, RtlFreeAnsiString, RtlInitUnicodeString, RtlUnicodeStringToAnsiString,
    ZwCallbackReturn,
};
// RtlCopyMemory / NT_SUCCESS function-style helpers.
use crate::winbase::{NT_SUCCESS, RtlCopyMemory};
// NtUserx* — inline forwarders to the NtUserCall* multiplexers (ntwrapper.h port).
use crate::ntwrapper::{
    NtUserxDeregisterShellHookWindow, NtUserxNotifyWinEvent, NtUserxRegisterShellHookWindow,
    NtUserxUnhookWindowsHook,
};

// WH_MSGFILTER (-1) is the lone negative WH_* → bindgen types it i32 while the rest
// (WH_CBT=5 …) are u32. The HookId match scrutinee is cast `as u32`, so alias the
// negative one to u32 (0xFFFFFFFF) for use as a match pattern.
const WH_MSGFILTER_U: u32 = WH_MSGFILTER as u32;

// ---- NTSTATUS constants (ntstatus.h) — not in win32k_sys's surface ----
const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_SUPPORTED: NTSTATUS = 0xC00000BBu32 as NTSTATUS;

// ---- kernel32 / user32-internal externs not provided by the substrate ----
//
// The C uses the generic LoadLibrary/GetModuleHandle macros; user32 is built
// UNICODE, so they resolve to the …W forms. InitUserApiHook / ClearUserApiHook /
// ghmodUserApiHook are user32-internal symbols living in the surviving C objects
// (usrapihk.c, cohab).
extern "system" {
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: *mut u16, nSize: u32) -> u32;
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: *mut core::ffi::c_void, dwFlags: u32) -> HMODULE;
    fn FreeLibrary(hLibModule: HMODULE) -> i32;
    fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut core::ffi::c_void;
    fn GetCurrentProcessId() -> DWORD;
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapAlloc(hHeap: *mut core::ffi::c_void, dwFlags: u32, dwBytes: usize) -> *mut core::ffi::c_void;
    fn HeapFree(hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void) -> i32;
    fn SetLastError(dwErrCode: DWORD);
    // user32-internal (usrapihk.c / dllmain.c) — WINAPI = stdcall on i386.
    fn InitUserApiHook(hInstance: HINSTANCE, pfn: USERAPIHOOKPROC) -> BOOL;
    fn ClearUserApiHook(hInstance: HINSTANCE) -> BOOL;
}

extern "C" {
    static mut ghmodUserApiHook: HINSTANCE; // user32p.h
}

// `LoadLibrary(...)` generic macro → …W on a UNICODE build.
#[inline]
unsafe fn LoadLibrary(name: LPCWSTR) -> HMODULE {
    LoadLibraryW(name)
}
// `GetModuleHandle(...)` generic macro → …W on a UNICODE build.
#[inline]
unsafe fn GetModuleHandle(name: LPCWSTR) -> HMODULE {
    GetModuleHandleW(name)
}

// `PtrToUint(p)` — basetsd.h, truncate pointer to 32-bit unsigned.
#[inline]
fn PtrToUint(p: PVOID) -> core::ffi::c_uint {
    p as usize as core::ffi::c_uint
}

// ---- user_x.h inlines (module-local, mirrors message.rs) ----
//
// These are FORCEINLINE helpers, not exported symbols; reproduced here because
// the substrate does not expose them (message.rs likewise defines its own).

/// user_x.h `GetWin32ClientInfo()` — `&NtCurrentTeb()->Win32ClientInfo`.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as *mut _ as PCLIENTINFO
}

/// ntuser.h `HOOKID_TO_FLAG(HookId)` — `1 << ((HookId) + 1)`.
#[inline]
fn HOOKID_TO_FLAG(HookId: UINT) -> ULONG {
    1u32 << (HookId + 1)
}

/// user_x.h `ISITHOOKED(HookId)` — checks thread + desktop hook bitmasks.
#[inline]
unsafe fn ISITHOOKED(HookId: UINT) -> BOOL {
    let pci = GetWin32ClientInfo();
    let flag = HOOKID_TO_FLAG(HookId);
    if ((*pci).fsHooks & flag) != 0 {
        return 1;
    }
    (!(*pci).pDeskInfo.is_null()
        && (crate::wow64::desktop_info_fshooks((*pci).pDeskInfo) & flag) != 0) as BOOL
}

/// user_x.h `DesktopPtrToUser(p)` — server→client desktop heap remap.
#[inline]
unsafe fn DesktopPtrToUser(p: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(p)
}

/// `TRACE!(...)` — WINE debug channel; a no-op in the cohab build.

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _NOTIFYEVENT {
    pub event: DWORD,
    pub idObject: LONG,
    pub idChild: LONG,
    pub flags: DWORD,
}
pub type NOTIFYEVENT = _NOTIFYEVENT;
pub type PNOTIFYEVENT = *mut _NOTIFYEVENT;

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
unsafe extern "fastcall" fn IntSetWindowsHook(
    idHook: core::ffi::c_int,
    lpfn: HOOKPROC,
    hMod: HINSTANCE,
    dwThreadId: u32,
    bAnsi: i32,
) -> HHOOK {
    IntSetWindowsHook_impl(idHook, lpfn, hMod, dwThreadId, bAnsi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSetWindowsHook(
    idHook: core::ffi::c_int,
    lpfn: HOOKPROC,
    hMod: HINSTANCE,
    dwThreadId: u32,
    bAnsi: i32,
) -> HHOOK {
    IntSetWindowsHook_impl(idHook, lpfn, hMod, dwThreadId, bAnsi)
}

unsafe fn IntSetWindowsHook_impl(
    idHook: core::ffi::c_int,
    lpfn: HOOKPROC,
    hMod: HINSTANCE,
    dwThreadId: u32,
    bAnsi: i32,
) -> HHOOK {
    let mut ModuleName: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut USModuleName: _UNICODE_STRING = core::mem::zeroed();

    if !hMod.is_null() {
        if 0 == GetModuleFileNameW(hMod, ModuleName.as_mut_ptr(), MAX_PATH) {
            return core::ptr::null_mut();
        }
        RtlInitUnicodeString(&mut USModuleName, ModuleName.as_ptr());
    } else {
        RtlInitUnicodeString(&mut USModuleName, core::ptr::null());
    }

    NtUserSetWindowsHookEx(hMod, &mut USModuleName, dwThreadId, idHook, lpfn, bAnsi)
}

// Cross-TU export: dialog.rs/scrollbar.rs (and other user32 TUs) call @IntNotifyWinEvent@20.
#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn IntNotifyWinEvent(
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
    flags: DWORD,
) {
    let mut ne: NOTIFYEVENT = core::mem::zeroed();
    ne.event = event;
    ne.idObject = idObject;
    ne.idChild = idChild;
    ne.flags = flags;
    if !gpsi.is_null() && (crate::wow64::psi::installed_event_hooks() & GetMaskFromEvent(event)) != 0 {
        NtUserxNotifyWinEvent(hwnd, core::ptr::addr_of_mut!(ne) as PVOID);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn IntNotifyWinEvent(
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
    flags: DWORD,
) {
    let mut ne: NOTIFYEVENT = core::mem::zeroed();
    ne.event = event;
    ne.idObject = idObject;
    ne.idChild = idChild;
    ne.flags = flags;
    if !gpsi.is_null() && (crate::wow64::psi::installed_event_hooks() & GetMaskFromEvent(event)) != 0 {
        NtUserxNotifyWinEvent(hwnd, core::ptr::addr_of_mut!(ne) as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn CallMsgFilterA(lpMsg: LPMSG, nCode: core::ffi::c_int) -> BOOL {
    let mut Msg: MSG = core::mem::zeroed();
    if !(*NtCurrentTeb()).Win32ThreadInfo.is_null()
        && (ISITHOOKED(WH_MSGFILTER as _) != 0 || ISITHOOKED(WH_SYSMSGFILTER as _) != 0)
    {
        if (*lpMsg).message & !WM_MAXIMUM != 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        RtlCopyMemory(
            core::ptr::addr_of_mut!(Msg) as _,
            lpMsg as _,
            core::mem::size_of::<MSG>() as _,
        );
        return NtUserCallMsgFilter(core::ptr::addr_of_mut!(Msg), nCode);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CallMsgFilterW(lpMsg: LPMSG, nCode: i32) -> i32 {
    let mut Msg: MSG = core::mem::zeroed();
    if !(*NtCurrentTeb()).Win32ThreadInfo.is_null()
        && (ISITHOOKED(WH_MSGFILTER as _) != 0 || ISITHOOKED(WH_SYSMSGFILTER as _) != 0)
    {
        if ((*lpMsg).message & !(WM_MAXIMUM as u32)) != 0 {
            SetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }
        RtlCopyMemory(
            core::ptr::addr_of_mut!(Msg) as _,
            lpMsg as _,
            core::mem::size_of::<MSG>() as _,
        );
        return NtUserCallMsgFilter(core::ptr::addr_of_mut!(Msg), nCode);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn CallNextHookEx(
    Hook: HHOOK,
    Code: core::ffi::c_int,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let ClientInfo: PCLIENTINFO;
    let Flags: ULONG_PTR;
    let Save: ULONG_PTR;
    let pHook: PHOOK;
    let phkNext: PHOOK;
    let mut lResult: LRESULT = 0;

    ClientInfo = GetWin32ClientInfo();

    if (*ClientInfo).phkCurrent.is_null() {
        return 0;
    }

    crate::wow64::wow64_unsupported("HOOK");
    pHook = DesktopPtrToUser((*ClientInfo).phkCurrent as _) as PHOOK;

    if (*pHook).phkNext.is_null() {
        return 0;
    }

    phkNext = DesktopPtrToUser((*pHook).phkNext as _) as PHOOK;

    if (*phkNext).HookId == WH_CALLWNDPROC as _
        || (*phkNext).HookId == WH_CALLWNDPROCRET as _
    {
        Save = (*ClientInfo).dwHookData;
        Flags = (*ClientInfo).CI_flags & CI_CURTHPRHOOK as ULONG_PTR;

        if wParam != 0 {
            (*ClientInfo).CI_flags |= CI_CURTHPRHOOK as ULONG_PTR;
        } else {
            (*ClientInfo).CI_flags &= !(CI_CURTHPRHOOK as ULONG_PTR);
        }

        if (*phkNext).HookId == WH_CALLWNDPROC as _ {
            let pCWP: PCWPSTRUCT = lParam as PCWPSTRUCT;

            NtUserMessageCall(
                (*pCWP).hwnd,
                (*pCWP).message,
                (*pCWP).wParam,
                (*pCWP).lParam,
                core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
                FNID_CALLWNDPROC,
                (*phkNext).Ansi as BOOL,
            );
        } else {
            let pCWPR: PCWPRETSTRUCT = lParam as PCWPRETSTRUCT;

            (*ClientInfo).dwHookData = (*pCWPR).lResult as ULONG_PTR;

            NtUserMessageCall(
                (*pCWPR).hwnd,
                (*pCWPR).message,
                (*pCWPR).wParam,
                (*pCWPR).lParam,
                core::ptr::addr_of_mut!(lResult) as ULONG_PTR,
                FNID_CALLWNDPROCRET,
                (*phkNext).Ansi as BOOL,
            );
        }
        (*ClientInfo).CI_flags ^= ((*ClientInfo).CI_flags ^ Flags) & CI_CURTHPRHOOK as ULONG_PTR;
        (*ClientInfo).dwHookData = Save;
    } else {
        lResult = NtUserCallNextHookEx(Code, wParam, lParam, (*pHook).Ansi as BOOL);
    }

    lResult
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookW(idHook: core::ffi::c_int, lpfn: HOOKPROC) -> HHOOK {
    let ThreadId: u32 = PtrToUint((*NtCurrentTeb()).ClientId.UniqueThread) as u32;
    IntSetWindowsHook(idHook, lpfn, core::ptr::null_mut(), ThreadId, 0)
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookA(idHook: core::ffi::c_int, lpfn: HOOKPROC) -> HHOOK {
    let ThreadId: u32 = PtrToUint((*NtCurrentTeb()).ClientId.UniqueThread) as u32;
    IntSetWindowsHook(idHook, lpfn, core::ptr::null_mut(), ThreadId, 1)
}

#[no_mangle]
pub unsafe extern "system" fn DeregisterShellHookWindow(hWnd: HWND) -> i32 {
    NtUserxDeregisterShellHookWindow(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn RegisterShellHookWindow(hWnd: HWND) -> i32 {
    NtUserxRegisterShellHookWindow(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn UnhookWindowsHook(nCode: core::ffi::c_int, pfnFilterProc: HOOKPROC) -> i32 {
    NtUserxUnhookWindowsHook(nCode, pfnFilterProc)
}

#[no_mangle]
pub unsafe extern "system" fn NotifyWinEvent(
    event: DWORD,
    hwnd: HWND,
    idObject: LONG,
    idChild: LONG,
) {
    if !gpsi.is_null() && (crate::wow64::psi::installed_event_hooks() & GetMaskFromEvent(event)) != 0 {
        NtUserNotifyWinEvent(event, hwnd, idObject, idChild);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetWinEventHook(
    eventMin: UINT,
    eventMax: UINT,
    hmodWinEventProc: HMODULE,
    pfnWinEventProc: WINEVENTPROC,
    idProcess: DWORD,
    idThread: DWORD,
    dwFlags: UINT,
) -> HWINEVENTHOOK {
    let mut ModuleName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut USModuleName: _UNICODE_STRING = core::mem::zeroed();
    let pusmodName: PUNICODE_STRING;

    RtlInitUnicodeString(&mut USModuleName, core::ptr::null());

    if !hmodWinEventProc.is_null() && (dwFlags & WINEVENT_INCONTEXT) != 0 {
        if 0 == GetModuleFileNameW(hmodWinEventProc as HINSTANCE, ModuleName.as_mut_ptr(), MAX_PATH) {
            return core::ptr::null_mut();
        }
        RtlInitUnicodeString(&mut USModuleName, ModuleName.as_ptr());
        pusmodName = &mut USModuleName;
    } else {
        pusmodName = core::ptr::null_mut();
    }

    NtUserSetWinEventHook(
        eventMin,
        eventMax,
        hmodWinEventProc,
        pusmodName,
        pfnWinEventProc,
        idProcess,
        idThread,
        dwFlags,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IsWinEventHookInstalled(event: DWORD) -> BOOL {
    if !((*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO).is_null() {
        return if !gpsi.is_null() {
            ((crate::wow64::psi::installed_event_hooks() & GetMaskFromEvent(event)) != 0) as BOOL
        } else {
            0
        };
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookExA(
    idHook: core::ffi::c_int,
    lpfn: HOOKPROC,
    hMod: HINSTANCE,
    dwThreadId: u32,
) -> HHOOK {
    IntSetWindowsHook(idHook, lpfn, hMod, dwThreadId, 1)
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowsHookExW(
    idHook: core::ffi::c_int,
    lpfn: HOOKPROC,
    hMod: HINSTANCE,
    dwThreadId: u32,
) -> HHOOK {
    IntSetWindowsHook(idHook, lpfn, hMod, dwThreadId, 0)
}

unsafe fn ClientLoadLibrary(
    pstrLibName: PUNICODE_STRING,
    pstrInitFunc: PUNICODE_STRING,
    bUnload: BOOL,
    bApiHook: BOOL,
) -> BOOL {
    let hLibrary: HINSTANCE;
    let pInitFunction: PVOID;
    let Status: NTSTATUS;
    let mut InitFuncName: ANSI_STRING = core::mem::zeroed();
    let mut bResult: BOOL = 0;

    TRACE!(
        b"ClientLoadLibrary: pid: %d, strLibraryName: %S, strInitFuncName: %S, bUnload: %d, bApiHook:%d\n\0".as_ptr() as *const i8,
        GetCurrentProcessId(),
        (*pstrLibName).Buffer,
        (*pstrInitFunc).Buffer,
        bUnload,
        bApiHook,
    );

    if bUnload == 0 {
        hLibrary = LoadLibrary((*pstrLibName).Buffer);
        if hLibrary == 0 as HINSTANCE {
            return 0;
        }

        if bApiHook == 0 {
            return 1;
        }

        Status = RtlUnicodeStringToAnsiString(&mut InitFuncName, pstrInitFunc, 1);
        if !NT_SUCCESS(Status) {
            FreeLibrary(hLibrary);
            return 0;
        }

        pInitFunction = GetProcAddress(hLibrary, InitFuncName.Buffer);
        if !pInitFunction.is_null() {
            // C: (USERAPIHOOKPROC)pInitFunction — USERAPIHOOKPROC is Option<fn>,
            // so transmute the raw PVOID into the optional fn pointer.
            bResult = InitUserApiHook(
                hLibrary,
                core::mem::transmute::<PVOID, USERAPIHOOKPROC>(pInitFunction),
            );
        }

        RtlFreeAnsiString(&mut InitFuncName);

        if bResult == 0 {
            FreeLibrary(hLibrary);
        }
    } else {
        if bApiHook != 0 {
            hLibrary = ghmodUserApiHook;
            bResult = ClearUserApiHook(ghmodUserApiHook);

            if bResult == 0 {
                return 1;
            }
        } else {
            hLibrary = GetModuleHandle((*pstrLibName).Buffer);
            if hLibrary == core::ptr::null_mut() {
                return 0;
            }
        }

        bResult = FreeLibrary(hLibrary);
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn User32CallClientLoadLibraryFromKernel(
    Arguments: *mut core::ffi::c_void,
    ArgumentLength: u32,
) -> i32 {
    let Argument = Arguments as PCLIENT_LOAD_LIBRARY_ARGUMENTS;

    if !(*Argument).strLibraryName.Buffer.is_null() {
        (*Argument).strLibraryName.Buffer =
            ((*Argument).strLibraryName.Buffer as usize + Argument as usize) as *mut u16;
    }
    if !(*Argument).strInitFuncName.Buffer.is_null() {
        (*Argument).strInitFuncName.Buffer =
            ((*Argument).strInitFuncName.Buffer as usize + Argument as usize) as *mut u16;
    }

    let mut bResult: i32 = ClientLoadLibrary(
        core::ptr::addr_of_mut!((*Argument).strLibraryName),
        core::ptr::addr_of_mut!((*Argument).strInitFuncName),
        (*Argument).Unload,
        (*Argument).ApiHook,
    );

    ZwCallbackReturn(
        core::ptr::addr_of_mut!(bResult) as *mut core::ffi::c_void,
        core::mem::size_of::<HINSTANCE>() as u32,
        STATUS_SUCCESS,
    )
}
#[no_mangle]
pub unsafe extern "system" fn User32CallHookProcFromKernel(
    Arguments: *mut core::ffi::c_void,
    ArgumentLength: u32,
) -> i32 {
    let Common: PHOOKPROC_CALLBACK_ARGUMENTS;
    let mut pCsw: *mut CREATESTRUCTW = core::ptr::null_mut();
    let mut pCbtCreatewndw: *mut CBT_CREATEWNDW = core::ptr::null_mut();
    let mut CbtCreatewndExtra: PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS = core::ptr::null_mut();
    let mut KeyboardLlData: KBDLLHOOKSTRUCT = core::mem::zeroed();
    let pKeyboardLlData: *mut KBDLLHOOKSTRUCT;
    let mut MouseLlData: MSLLHOOKSTRUCT = core::mem::zeroed();
    let pMouseLlData: *mut MSLLHOOKSTRUCT;
    let pcMsg: *mut MSG;
    let pMsg: *mut MSG;
    let mut pMHook: PMOUSEHOOKSTRUCT;
    let pCWP: *mut CWPSTRUCT;
    let pCWPR: *mut CWPRETSTRUCT;
    let prl: PRECTL;
    let pcbtas: LPCBTACTIVATESTRUCT;
    let mut Proc: HOOKPROC;
    let mut wParam: WPARAM = 0;
    let mut lParam: LPARAM = 0;
    let mut Result: LRESULT = 0;
    let mut Hit: u8 = 0u8;
    let mut Loaded: u8 = 0u8;
    let mut mod_: HMODULE = core::ptr::null_mut();
    let mut Status: i32 = STATUS_SUCCESS;

    const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x00000008;
    const STATUS_UNSUCCESSFUL: i32 = 0xC0000001u32 as i32;

    Common = Arguments as PHOOKPROC_CALLBACK_ARGUMENTS;

    Proc = (*Common).Proc;

    if (*Common).offPfn != 0 && (*Common).Mod != 0 {
        mod_ = GetModuleHandleW(core::ptr::addr_of!((*Common).ModuleName) as LPCWSTR);
        if mod_.is_null() {
            mod_ = LoadLibraryExW(
                core::ptr::addr_of!((*Common).ModuleName) as LPCWSTR,
                core::ptr::null_mut(),
                LOAD_WITH_ALTERED_SEARCH_PATH,
            );
            if !mod_.is_null() {
                Loaded = 1u8;
            }
        }
        if !mod_.is_null() {
            Proc = core::mem::transmute((mod_ as *mut u8).add((*Common).offPfn as usize));
        }
    }

    match (*Common).HookId as u32 {
        WH_CBT => {
            match (*Common).Code as u32 {
                HCBT_CREATEWND => {
                    CbtCreatewndExtra = (Common as *mut u8).offset((*Common).lParam as isize)
                        as PHOOKPROC_CBT_CREATEWND_EXTRA_ARGUMENTS;

                    pCbtCreatewndw = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<CBT_CREATEWNDW>())
                        as *mut CBT_CREATEWNDW;
                    core::ptr::copy_nonoverlapping(
                        CbtCreatewndExtra as *const u8,
                        pCbtCreatewndw as *mut u8,
                        core::mem::size_of::<CBT_CREATEWNDW>(),
                    );

                    pCsw = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<CREATESTRUCTW>())
                        as *mut CREATESTRUCTW;
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*CbtCreatewndExtra).Cs) as *const u8,
                        pCsw as *mut u8,
                        core::mem::size_of::<CREATESTRUCTW>(),
                    );

                    (*pCbtCreatewndw).lpcs = pCsw;
                    (*pCbtCreatewndw).hwndInsertAfter = (*CbtCreatewndExtra).WndInsertAfter;
                    wParam = (*Common).wParam;
                    lParam = pCbtCreatewndw as LPARAM;
                }
                HCBT_CLICKSKIPPED => {
                    pMHook = (Common as *mut u8).offset((*Common).lParam as isize) as PMOUSEHOOKSTRUCT;
                    lParam = pMHook as LPARAM;
                    wParam = (*Common).wParam;
                }
                HCBT_MOVESIZE => {
                    prl = (Common as *mut u8).offset((*Common).lParam as isize) as PRECTL;
                    lParam = prl as LPARAM;
                    wParam = (*Common).wParam;
                }
                HCBT_ACTIVATE => {
                    pcbtas = (Common as *mut u8).offset((*Common).lParam as isize) as LPCBTACTIVATESTRUCT;
                    lParam = pcbtas as LPARAM;
                    wParam = (*Common).wParam;
                }
                HCBT_KEYSKIPPED | HCBT_MINMAX | HCBT_SETFOCUS | HCBT_SYSCOMMAND
                | HCBT_DESTROYWND | HCBT_QS => {
                    wParam = (*Common).wParam;
                    lParam = (*Common).lParam;
                }
                _ => {
                    if Loaded != 0 {
                        FreeLibrary(mod_);
                    }
                    return ZwCallbackReturn(core::ptr::null_mut(), 0, STATUS_NOT_SUPPORTED);
                }
            }

            if let Some(p) = Proc {
                Result = p((*Common).Code, wParam, lParam);
            }

            if (*Common).Code == HCBT_CREATEWND as INT {
                (*CbtCreatewndExtra).WndInsertAfter = (*pCbtCreatewndw).hwndInsertAfter;
                (*CbtCreatewndExtra).Cs.x = (*(*pCbtCreatewndw).lpcs).x;
                (*CbtCreatewndExtra).Cs.y = (*(*pCbtCreatewndw).lpcs).y;
                (*CbtCreatewndExtra).Cs.cx = (*(*pCbtCreatewndw).lpcs).cx;
                (*CbtCreatewndExtra).Cs.cy = (*(*pCbtCreatewndw).lpcs).cy;
                HeapFree(GetProcessHeap(), 0, pCsw as *mut core::ffi::c_void);
                HeapFree(GetProcessHeap(), 0, pCbtCreatewndw as *mut core::ffi::c_void);
            }
        }
        WH_KEYBOARD_LL => {
            pKeyboardLlData = (Common as *mut u8).offset((*Common).lParam as isize) as PKBDLLHOOKSTRUCT;
            core::ptr::copy_nonoverlapping(
                pKeyboardLlData as *const u8,
                core::ptr::addr_of_mut!(KeyboardLlData) as *mut u8,
                core::mem::size_of::<KBDLLHOOKSTRUCT>(),
            );
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, core::ptr::addr_of_mut!(KeyboardLlData) as LPARAM);
        }
        WH_MOUSE_LL => {
            pMouseLlData = (Common as *mut u8).offset((*Common).lParam as isize) as PMSLLHOOKSTRUCT;
            core::ptr::copy_nonoverlapping(
                pMouseLlData as *const u8,
                core::ptr::addr_of_mut!(MouseLlData) as *mut u8,
                core::mem::size_of::<MSLLHOOKSTRUCT>(),
            );
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, core::ptr::addr_of_mut!(MouseLlData) as LPARAM);
        }
        WH_MOUSE => {
            pMHook = (Common as *mut u8).offset((*Common).lParam as isize) as PMOUSEHOOKSTRUCT;
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, pMHook as LPARAM);
        }
        WH_CALLWNDPROC => {
            let pcwps = Common as PCWP_Struct;
            let pCWPT = core::ptr::addr_of_mut!((*pcwps).cwps);
            let pCWP_local = HeapAlloc(
                GetProcessHeap(),
                0,
                (*Common).lParamSize as usize + core::mem::size_of::<CWPSTRUCT>(),
            ) as *mut CWPSTRUCT;
            core::ptr::copy_nonoverlapping(
                pCWPT as *const u8,
                pCWP_local as *mut u8,
                core::mem::size_of::<CWPSTRUCT>(),
            );
            if (*Common).lParamSize != 0 {
                (*pCWP_local).lParam = (pCWP_local as *mut u8).add(core::mem::size_of::<CWPSTRUCT>()) as LPARAM;
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*pcwps).Extra) as *const u8,
                    (pCWP_local as *mut u8).add(core::mem::size_of::<CWPSTRUCT>()),
                    (*Common).lParamSize as usize,
                );
            }
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, pCWP_local as LPARAM);
            HeapFree(GetProcessHeap(), 0, pCWP_local as *mut core::ffi::c_void);
            let _ = pCWP;
        }
        WH_CALLWNDPROCRET => {
            let pcwprs = Common as PCWPR_Struct;
            let pCWPRT = core::ptr::addr_of_mut!((*pcwprs).cwprs);
            let pCWPR_local = HeapAlloc(
                GetProcessHeap(),
                0,
                (*Common).lParamSize as usize + core::mem::size_of::<CWPRETSTRUCT>(),
            ) as *mut CWPRETSTRUCT;
            core::ptr::copy_nonoverlapping(
                pCWPRT as *const u8,
                pCWPR_local as *mut u8,
                core::mem::size_of::<CWPRETSTRUCT>(),
            );
            if (*Common).lParamSize != 0 {
                (*pCWPR_local).lParam = (pCWPR_local as *mut u8).add(core::mem::size_of::<CWPRETSTRUCT>()) as LPARAM;
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*pcwprs).Extra) as *const u8,
                    (pCWPR_local as *mut u8).add(core::mem::size_of::<CWPRETSTRUCT>()),
                    (*Common).lParamSize as usize,
                );
            }
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, pCWPR_local as LPARAM);
            HeapFree(GetProcessHeap(), 0, pCWPR_local as *mut core::ffi::c_void);
            let _ = pCWPR;
        }
        WH_MSGFILTER_U | WH_SYSMSGFILTER | WH_GETMESSAGE => {
            pMsg = (Common as *mut u8).offset((*Common).lParam as isize) as PMSG;
            pcMsg = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<MSG>()) as *mut MSG;
            core::ptr::copy_nonoverlapping(
                pMsg as *const u8,
                pcMsg as *mut u8,
                core::mem::size_of::<MSG>(),
            );
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, pcMsg as LPARAM);

            if Hit == 0 && (*Common).HookId as u32 == WH_GETMESSAGE {
                core::ptr::copy_nonoverlapping(
                    pcMsg as *const u8,
                    pMsg as *mut u8,
                    core::mem::size_of::<MSG>(),
                );
            }
            HeapFree(GetProcessHeap(), 0, pcMsg as *mut core::ffi::c_void);
        }
        WH_KEYBOARD | WH_SHELL => {
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, (*Common).lParam);
        }
        WH_FOREGROUNDIDLE => {
            Result = (Proc.unwrap())((*Common).Code, (*Common).wParam, (*Common).lParam);
        }
        _ => {
            if Loaded != 0 {
                FreeLibrary(mod_);
            }
            return ZwCallbackReturn(core::ptr::null_mut(), 0, STATUS_NOT_SUPPORTED);
        }
    }

    if Hit != 0 {
        Status = STATUS_UNSUCCESSFUL;
    }
    if Loaded != 0 {
        FreeLibrary(mod_);
    }
    (*Common).Result = Result;
    ZwCallbackReturn(Arguments, ArgumentLength, Status)
}

#[no_mangle]
pub unsafe extern "system" fn User32CallEventProcFromKernel(
    Arguments: *mut core::ffi::c_void,
    _ArgumentLength: u32,
) -> i32 {
    User32CallEventProcFromKernel_impl(Arguments)
}

unsafe fn User32CallEventProcFromKernel_impl(Arguments: *mut core::ffi::c_void) -> i32 {
    const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x00000008;

    let Common: PEVENTPROC_CALLBACK_ARGUMENTS = Arguments as PEVENTPROC_CALLBACK_ARGUMENTS;
    let mut module: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut mod_: HMODULE = core::ptr::null_mut();
    let mut Loaded: u8 = 0u8;

    let mut Proc: WINEVENTPROC = (*Common).Proc;

    if (*Common).offPfn != 0 && (*Common).Mod != 0 {
        // Validate the module again.
        let len = GetModuleFileNameW((*Common).Mod as HINSTANCE, module.as_mut_ptr(), MAX_PATH);
        if len == 0 || len >= MAX_PATH {
            (*Common).Mod = 0;
        }

        if (*Common).Mod != 0 {
            mod_ = GetModuleHandleW(module.as_ptr());
            if mod_.is_null() {
                mod_ = LoadLibraryExW(
                    module.as_ptr(),
                    core::ptr::null_mut(),
                    LOAD_WITH_ALTERED_SEARCH_PATH,
                );
                if !mod_.is_null() {
                    Loaded = 1u8; // Free it only when loaded.
                }
            }
        }

        if !mod_.is_null() {
            Proc = core::mem::transmute((mod_ as *mut u8).add((*Common).offPfn as usize));
        }
    }

    if let Some(p) = Proc {
        p(
            (*Common).hook,
            (*Common).event,
            (*Common).hwnd,
            (*Common).idObject,
            (*Common).idChild,
            (*Common).dwEventThread,
            (*Common).dwmsEventTime,
        );
    }

    if Loaded != 0 {
        FreeLibrary(mod_);
    }

    ZwCallbackReturn(core::ptr::null_mut(), 0, STATUS_SUCCESS)
}
