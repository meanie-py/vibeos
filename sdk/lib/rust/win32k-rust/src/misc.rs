#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::{STATUS_UNSUCCESSFUL, STATUS_INVALID_PARAMETER, STATUS_NO_TOKEN};
use ntkrnl_sys::ddk::{ExGetPreviousMode, ExAllocatePoolWithTag, ExFreePoolWithTag, ObReferenceObjectByHandle, PsGetCurrentProcessId, PsGetCurrentProcessWin32Process, PsGetCurrentThread, PsGetCurrentThreadWin32Thread, PsGetThreadId, PsGetThreadProcess, PsGetThreadWin32Thread, PsIsThreadTerminating, PsLookupThreadByThreadId, PsReferenceImpersonationToken, PsReferencePrimaryToken, RtlInitUnicodeString, RtlUnicodeStringToInteger, SeQueryAuthenticationIdToken, ZwClose, ZwOpenKey, ZwQueryValueKey};
use ntkrnl_sys::ob::ObDereferenceObject;
use ntkrnl_sys::inline::ke::NtCurrentTeb;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    // _MmCopyFromCaller / _MmCopyToCaller — the C `MmCopy{From,To}Caller(x,y,z)`
    // macros forward to these (mmcopy.h). cdecl; provided by copy.rs as
    // `__MmCopyFromCaller` / `__MmCopyToCaller`, so reference via the `_`-prefixed
    // name (matches monitor.rs/text.rs).
    fn _MmCopyFromCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
    fn _MmCopyToCaller(Target: *mut core::ffi::c_void, Source: *const core::ffi::c_void, Bytes: UINT) -> NTSTATUS;
    // _wtoi(const wchar_t*) — CRT, cdecl.
    fn _wtoi(s: *const u16) -> i32;
}
// RegReadUserSetting — win32k-rust sibling (registry.rs), NTAPI: stdcall on i386
// (`_RegReadUserSetting@20`), cdecl elsewhere → extern "system".
extern "system" {
    fn RegReadUserSetting(pwszKeyName: PCWSTR, pwszValueName: PCWSTR, ulType: ULONG, pvData: *mut core::ffi::c_void, cbDataSize: ULONG) -> NTSTATUS;
}

// MmCopyFromCaller(x,y,z) = _MmCopyFromCaller((PCHAR)x,(PCHAR)y,(UINT)z) (mmcopy.h macro).
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyFromCaller(dst, src, len as UINT)
}

// EngGetTickCount() — APIENTRY (stdcall) per eng.h; EngGetTickCount32 truncates.
extern "system" {
    fn EngGetTickCount() -> ULONGLONG;
}
#[inline(always)]
unsafe fn EngGetTickCount32() -> ULONG {
    EngGetTickCount() as ULONG
}

// MsqGetMessageExtraInfo(VOID) — FASTCALL per msgqueue.h (`@Name@0` i386, cdecl else).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn MsqGetMessageExtraInfo() -> LPARAM;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn MsqGetMessageExtraInfo() -> LPARAM;
}

// MmCopyToCaller(x,y,z) = _MmCopyToCaller((PCHAR)x,(PCHAR)y,(UINT)z) (mmcopy.h macro).
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyToCaller(dst, src, len as UINT)
}

// GetWin32ClientInfo() = (PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo) (ntuser.h
// macro — no exported symbol). Win32ClientInfo is the first byte of an inline
// array field on the TEB, so its address is the PCLIENTINFO.
#[inline(always)]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    let teb = NtCurrentTeb() as *mut ntkrnl_sys::ddk::_TEB;
    core::ptr::addr_of_mut!((*teb).Win32ClientInfo) as PCLIENTINFO
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
// FORCEINLINE macro — no exported symbol (matches focus.rs).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(pobj: *mut T) -> HWND {
    *(pobj as *const HWND)
}

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ULONG)ul (basetsd.h macro).
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE {
    ul as ULONG_PTR as HANDLE
}

// LOWORD / HIWORD (minwindef.h macros).
#[inline(always)]
fn LOWORD(l: DWORD) -> DWORD { l & 0xffff }
#[inline(always)]
fn HIWORD(l: DWORD) -> DWORD { (l >> 16) & 0xffff }

// MAKELONG / MAKEWPARAM / MAKELPARAM (minwindef.h / winuser.h macros).
#[inline(always)]
fn MAKELONG(low: u32, high: u32) -> LONG {
    ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG
}
#[inline(always)]
fn MAKEWPARAM(low: u32, high: u32) -> WPARAM {
    MAKELONG(low, high) as u32 as WPARAM
}
#[inline(always)]
fn MAKELPARAM(low: i32, high: i32) -> LPARAM {
    MAKELONG(low as u32, high as u32) as LPARAM
}

// _THRDESKHEAD/_THROBJHEAD.pti: bindgen flattens HEAD{h, cLockObj} + pti into
// __bindgen_padding_0. pti follows HEAD{HANDLE, DWORD} and is ptr-aligned, so it
// sits at 2*usize (8 i386 / 16 amd64/arm64). (Matches simplecall.rs/focus.rs.)
#[inline(always)]
unsafe fn head_pti(head: *const _THRDESKHEAD) -> PTHREADINFO {
    let off = 2 * core::mem::size_of::<usize>();
    *((head as *const u8).add(off) as *const PTHREADINFO)
}

// _THREADINFO.pEThread: the embedded W32THREAD prefix is flattened into
// __bindgen_padding_0; pEThread is the first W32THREAD member, at offset 0.
// Returns the ddk ETHREAD ptr type (PsGetThreadId's param).
#[inline(always)]
unsafe fn pti_pEThread(pti: PTHREADINFO) -> *mut ntkrnl_sys::ddk::_ETHREAD {
    *(pti as *const *mut ntkrnl_sys::ddk::_ETHREAD)
}

// PROCESSINFO->W32PF_flags lives in the flattened W32PROCESS prefix
// (peProcess ptr + RefCount DWORD), so W32PF_flags (ULONG) sits at
// size_of::<usize>()+size_of::<u32>() = 8 (i386) / 12 (amd64/arm64).
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    let off = core::mem::size_of::<usize>() + core::mem::size_of::<u32>();
    (ppi as *mut u8).add(off) as *mut DWORD
}

// gpsi->ptCursor etc.: tagSERVERINFO ends with __bindgen_padding_0 representing
// the anonymous PERUSERSERVERINFO member; cast that tail (matches focus.rs).
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// PROCESSINFO->{GDIHandleCount,UserHandleCount}: these live on the flattened
// W32PROCESS prefix embedded in PROCESSINFO (object.rs accesses them the same way).
#[inline(always)]
unsafe fn ppi_GDIHandleCount(ppi: PPROCESSINFO) -> LONG {
    (*(ppi as *mut W32PROCESS)).GDIHandleCount
}
#[inline(always)]
unsafe fn ppi_UserHandleCount(ppi: PPROCESSINFO) -> LONG {
    (*(ppi as *mut W32PROCESS)).UserHandleCount
}

// ---- file-scope preamble (translated) ----
// _scwprintf is NOT exported by ntoskrnl (only _vscwprintf is), so misc.c
// implements it as a cdecl varargs forwarder to _vscwprintf. Rust on the stable
// toolchain cannot author a C-variadic body (c_variadic is unstable), so the
// forwarder is a global_asm thunk: a va_list on each of these targets is just a
// pointer to the in-memory vararg region, so we point _vscwprintf at the
// caller's vararg slot. Symbol must match the i386 cdecl decoration `__scwprintf`.
#[cfg(target_arch = "x86")]
core::arch::global_asm!(
    ".globl __scwprintf",
    "__scwprintf:",
    "    push ebp",
    "    mov  ebp, esp",
    "    lea  eax, [ebp + 12]",   // &first vararg (ebp+0=saved ebp, +4=ret, +8=format)
    "    push eax",               // va_list
    "    push dword ptr [ebp + 8]", // format
    "    call __vscwprintf",
    "    add  esp, 8",
    "    mov  esp, ebp",
    "    pop  ebp",
    "    ret",
);

// amd64 Win64: rcx=format, the varargs are homed by the caller in the shadow
// space; va_start points at the slot above the home area. We home rdx/r8/r9 into
// the caller's reserved shadow space (rsp+8..) and pass va_list = &that slot.
#[cfg(target_arch = "x86_64")]
core::arch::global_asm!(
    ".globl _scwprintf",
    "_scwprintf:",
    "    mov  qword ptr [rsp + 16], rdx", // home arg2 (1st vararg)
    "    mov  qword ptr [rsp + 24], r8",  // home arg3
    "    mov  qword ptr [rsp + 32], r9",  // home arg4
    "    lea  rdx, [rsp + 16]",           // va_list -> &1st vararg
    "    sub  rsp, 40",                   // shadow space + align
    "    call _vscwprintf",
    "    add  rsp, 40",
    "    ret",
);

// arm64 AAPCS64: x0=format; varargs in x1.. then on stack. va_list points at the
// caller's vararg save area. Home x1-x7 to a local save area and pass its address.
#[cfg(target_arch = "aarch64")]
core::arch::global_asm!(
    ".globl _scwprintf",
    "_scwprintf:",
    "    stp  x29, x30, [sp, #-80]!",
    "    mov  x29, sp",
    "    stp  x1, x2, [sp, #16]",
    "    stp  x3, x4, [sp, #32]",
    "    stp  x5, x6, [sp, #48]",
    "    str  x7,     [sp, #64]",
    "    add  x1, sp, #16",               // va_list -> &1st vararg
    "    bl   _vscwprintf",
    "    ldp  x29, x30, [sp], #80",
    "    ret",
);

extern "C" {
    fn _vscwprintf(format: *const wchar_t, argptr: *mut i8) -> core::ffi::c_int;
}
// Keep _vscwprintf referenced so the import is retained for the asm thunk.
#[used]
static _VSCWPRINTF_REF: unsafe extern "C" fn(*const wchar_t, *mut i8) -> core::ffi::c_int = _vscwprintf;

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntTID2PTI(id: HANDLE) -> PTHREADINFO {
    IntTID2PTI_impl(id)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntTID2PTI(id: HANDLE) -> PTHREADINFO {
    IntTID2PTI_impl(id)
}

unsafe fn IntTID2PTI_impl(id: HANDLE) -> PTHREADINFO {
    let mut Thread: *mut ntkrnl_sys::ddk::_ETHREAD = core::ptr::null_mut();
    let Status = PsLookupThreadByThreadId(id, &mut Thread);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }
    if PsIsThreadTerminating(Thread) != 0 {
        ObDereferenceObject(Thread as *mut _);
        return core::ptr::null_mut();
    }
    let mut pti = PsGetThreadWin32Thread(Thread) as PTHREADINFO;
    if pti.is_null() {
        ObDereferenceObject(Thread as *mut _);
        return core::ptr::null_mut();
    }
    if (*pti).TIF_flags & TIF_INCLEANUP != 0 {
        pti = core::ptr::null_mut();
    }
    if !pti.is_null() && (*pti).TIF_flags & TIF_GUITHREADINITIALIZED == 0 {
        pti = core::ptr::null_mut();
    }
    if PsGetThreadId(Thread) != id {
        pti = core::ptr::null_mut();
    }
    ObDereferenceObject(Thread as *mut _);
    pti
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetLanguageToggle(
    pszType: PCWSTR,
    dwDefaultValue: DWORD,
) -> DWORD {
    let mut dwValue: DWORD = dwDefaultValue;
    let mut szBuff: [WCHAR; 4] = [0; 4];

    let Status: NTSTATUS = RegReadUserSetting(
        w!("Keyboard Layout\\Toggle").as_ptr() as PCWSTR,
        pszType,
        REG_SZ,
        szBuff.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&szBuff) as DWORD,
    );

    if NT_SUCCESS(Status) {
        szBuff[szBuff.len() - 1] = 0;
        dwValue = _wtoi(szBuff.as_ptr()) as DWORD;
    }

    dwValue
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetLanguageToggle(
    pszType: PCWSTR,
    dwDefaultValue: DWORD,
) -> DWORD {
    let mut dwValue: DWORD = dwDefaultValue;
    let mut szBuff: [WCHAR; 4] = [0; 4];

    let Status: NTSTATUS = RegReadUserSetting(
        w!("Keyboard Layout\\Toggle").as_ptr() as PCWSTR,
        pszType,
        REG_SZ,
        szBuff.as_mut_ptr() as *mut core::ffi::c_void,
        core::mem::size_of_val(&szBuff) as DWORD,
    );

    if NT_SUCCESS(Status) {
        szBuff[szBuff.len() - 1] = 0;
        dwValue = _wtoi(szBuff.as_ptr()) as DWORD;
    }

    dwValue
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetLanguageID() -> USHORT {
    user_get_language_id_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetLanguageID() -> USHORT {
    user_get_language_id_impl()
}

unsafe fn user_get_language_id_impl() -> USHORT {
    const TAG_STRING: ULONG = u32::from_be_bytes(*b" RTS"); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObAttr: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Ret: ULONG = ((SUBLANG_DEFAULT as ULONG) << 10) | (LANG_ENGLISH as ULONG);
    let mut Size: ULONG = (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()
        + (MAX_PATH as usize) * core::mem::size_of::<u16>()) as ULONG;
    let mut Language: ntkrnl_sys::ddk::_UNICODE_STRING = core::mem::zeroed();

    let lang_path: [u16; 71] = {
        let s = "\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Nls\\Language\0";
        let mut arr = [0u16; 71];
        let mut i = 0;
        for c in s.encode_utf16() {
            arr[i] = c;
            i += 1;
        }
        arr
    };
    RtlInitUnicodeString(&mut Language, lang_path.as_ptr());

    ObAttr.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as ULONG;
    ObAttr.RootDirectory = core::ptr::null_mut();
    ObAttr.ObjectName = &mut Language;
    ObAttr.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
    ObAttr.SecurityDescriptor = core::ptr::null_mut();
    ObAttr.SecurityQualityOfService = core::ptr::null_mut();

    if NT_SUCCESS(ZwOpenKey(&mut KeyHandle, KEY_READ, &mut ObAttr)) {
        let pKeyInfo = ExAllocatePoolWithTag(PagedPool, Size as _, TAG_STRING)
            as PKEY_VALUE_PARTIAL_INFORMATION;
        if !pKeyInfo.is_null() {
            let default_str: [u16; 8] = {
                let s = "Default\0";
                let mut arr = [0u16; 8];
                let mut i = 0;
                for c in s.encode_utf16() {
                    arr[i] = c;
                    i += 1;
                }
                arr
            };
            RtlInitUnicodeString(&mut Language, default_str.as_ptr());

            if NT_SUCCESS(ZwQueryValueKey(
                KeyHandle,
                &mut Language,
                KeyValuePartialInformation,
                pKeyInfo as *mut _,
                Size,
                &mut Size,
            )) {
                RtlInitUnicodeString(
                    &mut Language,
                    core::ptr::addr_of!((*pKeyInfo).Data) as PWSTR,
                );
                if !NT_SUCCESS(RtlUnicodeStringToInteger(&mut Language, 16, &mut Ret)) {
                    Ret = ((SUBLANG_DEFAULT as ULONG) << 10) | (LANG_ENGLISH as ULONG);
                }
            }
            ExFreePoolWithTag(pKeyInfo as *mut _, TAG_STRING);
        }
        ZwClose(KeyHandle);
    }

    Ret as USHORT
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GetControlColor(
    mut pwndParent: PWND,
    pwnd: PWND,
    hdc: HDC,
    CtlMsg: UINT,
) -> HBRUSH {
    GetControlColor_impl(pwndParent, pwnd, hdc, CtlMsg)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GetControlColor(
    mut pwndParent: PWND,
    pwnd: PWND,
    hdc: HDC,
    CtlMsg: UINT,
) -> HBRUSH {
    GetControlColor_impl(pwndParent, pwnd, hdc, CtlMsg)
}

unsafe fn GetControlColor_impl(
    mut pwndParent: PWND,
    pwnd: PWND,
    hdc: HDC,
    CtlMsg: UINT,
) -> HBRUSH {
    let mut hBrush: HBRUSH;

    if pwndParent.is_null() {
        pwndParent = pwnd;
    }

    if (*head_pti(core::ptr::addr_of!((*pwndParent).head))).ppi != PsGetCurrentProcessWin32Process() as PPROCESSINFO {
        return IntDefWindowProc(
            pwndParent,
            CtlMsg,
            hdc as WPARAM,
            UserHMGetHandle(pwnd) as LPARAM,
            0,
        ) as HBRUSH;
    }

    hBrush = co_IntSendMessage(
        UserHMGetHandle(pwndParent),
        CtlMsg,
        hdc as WPARAM,
        UserHMGetHandle(pwnd) as LPARAM,
    ) as HBRUSH;

    if hBrush.is_null() || GreIsHandleValid(hBrush as HGDIOBJ) == 0 {
        hBrush = IntDefWindowProc(
            pwndParent,
            CtlMsg,
            hdc as WPARAM,
            UserHMGetHandle(pwnd) as LPARAM,
            0,
        ) as HBRUSH;
    }
    hBrush
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GetControlBrush(
    pwnd: PWND,
    hdc: HDC,
    ctlType: UINT,
) -> HBRUSH {
    let pwndParent = IntGetParent(pwnd);
    GetControlColor(pwndParent, pwnd, hdc, ctlType)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GetControlBrush(
    pwnd: PWND,
    hdc: HDC,
    ctlType: UINT,
) -> HBRUSH {
    let pwndParent = IntGetParent(pwnd);
    GetControlColor(pwndParent, pwnd, hdc, ctlType)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetControlBrush(
    hwnd: HWND,
    hdc: HDC,
    ctlType: UINT,
) -> HBRUSH {
    let mut hBrush: HBRUSH = core::ptr::null_mut();

    UserEnterExclusive();
    let pwnd = UserGetWindowObject(hwnd);
    if !pwnd.is_null()
        && (ctlType.wrapping_sub(WM_CTLCOLORMSGBOX)) < CTLCOLOR_MAX
        && !hdc.is_null()
    {
        hBrush = GetControlBrush(pwnd, hdc, ctlType);
    }
    UserLeave();
    hBrush
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetControlColor(
    hwndParent: HWND,
    hwnd: HWND,
    hdc: HDC,
    CtlMsg: UINT,
) -> HBRUSH {
    let mut pwnd: PWND;
    let mut pwndParent: PWND = core::ptr::null_mut();
    let mut hBrush: HBRUSH = core::ptr::null_mut();

    UserEnterExclusive();
    pwnd = UserGetWindowObject(hwnd);
    if !pwnd.is_null()
        && (CtlMsg.wrapping_sub(WM_CTLCOLORMSGBOX) < CTLCOLOR_MAX)
        && !hdc.is_null()
    {
        if !hwndParent.is_null() {
            pwndParent = UserGetWindowObject(hwndParent);
        }
        hBrush = GetControlColor(pwndParent, pwnd, hdc, CtlMsg);
    }
    UserLeave();
    hBrush
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetThreadState(Routine: DWORD) -> DWORD_PTR {
    let mut ret: DWORD_PTR = 0;
    let pti: PTHREADINFO;

    if Routine != THREADSTATE_GETTHREADINFO {
        UserEnterShared();
    } else {
        UserEnterExclusive();
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    match Routine {
        THREADSTATE_GETTHREADINFO => {
            ret = 1;
        }
        THREADSTATE_FOCUSWINDOW => {
            ret = IntGetThreadFocusWindow() as DWORD_PTR;
        }
        THREADSTATE_CAPTUREWINDOW => {
            ret = IntGetCapture() as DWORD_PTR;
        }
        THREADSTATE_PROGMANWINDOW => {
            ret = (*(*GetW32ThreadInfo()).pDeskInfo).hProgmanWindow as DWORD_PTR;
        }
        THREADSTATE_TASKMANWINDOW => {
            ret = (*(*GetW32ThreadInfo()).pDeskInfo).hTaskManWindow as DWORD_PTR;
        }
        THREADSTATE_ACTIVEWINDOW => {
            ret = UserGetActiveWindow() as DWORD_PTR;
        }
        THREADSTATE_INSENDMESSAGE => {
            let Message: PUSER_SENT_MESSAGE =
                (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).pusmCurrent;

            ret = ISMEX_NOSEND as DWORD_PTR;
            if !Message.is_null() {
                if !(*Message).ptiSender.is_null() {
                    ret = ISMEX_SEND as DWORD_PTR;
                } else if (*Message).CompletionCallback.is_some() {
                    ret = ISMEX_CALLBACK as DWORD_PTR;
                } else {
                    ret = ISMEX_NOTIFY as DWORD_PTR;
                }
                if (*Message).QS_Flags & QS_SMRESULT != 0 {
                    ret |= ISMEX_REPLIED as DWORD_PTR;
                }
            }
        }
        THREADSTATE_GETMESSAGETIME => {
            ret = (*(PsGetCurrentThreadWin32Thread() as PTHREADINFO)).timeLast as DWORD_PTR;
        }
        THREADSTATE_UPTIMELASTREAD => {
            (*(*pti).pcti).timeLastRead = EngGetTickCount32();
        }
        THREADSTATE_GETINPUTSTATE => {
            ret = (LOWORD(IntGetQueueStatus(
                QS_POSTMESSAGE | QS_TIMER | QS_PAINT | QS_SENDMESSAGE | QS_INPUT | QS_RAWINPUT,
            )) as DWORD_PTR)
                & (QS_KEY | QS_MOUSEBUTTON) as DWORD_PTR;
        }
        THREADSTATE_FOREGROUNDTHREAD => {
            ret = (gpqForeground == (*GetW32ThreadInfo()).MessageQueue) as DWORD_PTR;
        }
        THREADSTATE_GETCURSOR => {
            ret = if !(*(*GetW32ThreadInfo()).MessageQueue).CursorObject.is_null() {
                UserHMGetHandle((*(*GetW32ThreadInfo()).MessageQueue).CursorObject) as DWORD_PTR
            } else {
                0
            };
        }
        THREADSTATE_GETMESSAGEEXTRAINFO => {
            ret = MsqGetMessageExtraInfo() as DWORD_PTR;
        }
        THREADSTATE_DEFAULTIMEWINDOW => {
            if !(*pti).spwndDefaultIme.is_null() {
                ret = UserHMGetHandle((*pti).spwndDefaultIme) as DWORD_PTR;
            }
        }
        THREADSTATE_DEFAULTINPUTCONTEXT => {
            if !(*pti).spDefaultImc.is_null() {
                ret = UserHMGetHandle((*pti).spDefaultImc) as DWORD_PTR;
            }
        }
        THREADSTATE_CHANGEBITS => {
            ret = (*(*pti).pcti).fsChangeBits as DWORD_PTR;
        }
        THREADSTATE_IMECOMPATFLAGS => {
            ret = (*(*pti).ppi).dwImeCompatFlags as DWORD_PTR;
        }
        THREADSTATE_OLDKEYBOARDLAYOUT => {
            ret = (*pti).hklPrev as DWORD_PTR;
        }
        THREADSTATE_ISWINLOGON => {
            ret = (gpidLogon == PsGetCurrentProcessId()) as DWORD_PTR;
        }
        THREADSTATE_UNKNOWN_0x10 => {
            FIXME!(b"stub\n\0".as_ptr() as *const i8);
        }
        THREADSTATE_CHECKCONIME => {
            ret = (IntTID2PTI(UlongToHandle((*(*pti).rpdesk).dwConsoleThreadId)) == pti)
                as DWORD_PTR;
        }
        _ => {}
    }

    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetThreadState(Set: DWORD, Flags: DWORD) -> DWORD {
    let pti: PTHREADINFO;
    let mut Ret: DWORD = 0;

    if Set & !(QF_FF10STATUS | QF_DIALOGACTIVE | QF_TABSWITCHING | QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) != 0 {
        return 0;
    }
    if Flags & !(QF_FF10STATUS | QF_DIALOGACTIVE | QF_TABSWITCHING | QF_FMENUSTATUS | QF_FMENUSTATUSBREAK) != 0 {
        return 0;
    }

    UserEnterExclusive();
    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if !(*pti).MessageQueue.is_null() {
        Ret = (*(*pti).MessageQueue).QF_flags;
        if Set != 0 {
            (*(*pti).MessageQueue).QF_flags |= Set & Flags;
        } else {
            if Flags != 0 {
                (*(*pti).MessageQueue).QF_flags &= !Flags;
            }
        }
    }
    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetDoubleClickTime() -> UINT {
    let Result: UINT;

    UserEnterShared();

    Result = gspv.iDblClickTime as UINT;

    UserLeave();
    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetGUIThreadInfo(
    idThread: DWORD,
    lpgui: LPGUITHREADINFO,
) -> BOOL {
    let mut Status: NTSTATUS;
    let CaretInfo: PTHRDCARETINFO;
    let mut SafeGui: GUITHREADINFO = core::mem::zeroed();
    let Desktop: PDESKTOP;
    let MsgQueue: PUSER_MESSAGE_QUEUE;
    let W32Thread: PTHREADINFO;
    let pti: PTHREADINFO;
    let mut Ret: BOOL = 0;

    UserEnterShared();

    'block: {
        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(SafeGui) as *mut core::ffi::c_void,
            lpgui as *mut core::ffi::c_void,
            core::mem::size_of::<DWORD>(),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'block;
        }

        if SafeGui.cbSize != core::mem::size_of::<GUITHREADINFO>() as UINT {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'block;
        }

        if idThread != 0 {
            pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

            W32Thread = IntTID2PTI(UlongToHandle(idThread));

            if W32Thread.is_null() {
                EngSetLastError(ERROR_ACCESS_DENIED);
                break 'block;
            }

            Desktop = (*W32Thread).rpdesk;

            if Desktop.is_null() || Desktop != (*pti).rpdesk {
                EngSetLastError(ERROR_ACCESS_DENIED);
                break 'block;
            }

            if !(*W32Thread).MessageQueue.is_null() {
                MsgQueue = (*W32Thread).MessageQueue;
            } else {
                MsgQueue = (*Desktop).ActiveMessageQueue;
            }
        } else {
            MsgQueue = IntGetFocusMessageQueue();
            if MsgQueue.is_null() {
                EngSetLastError(ERROR_ACCESS_DENIED);
                break 'block;
            }
        }

        CaretInfo = core::ptr::addr_of_mut!((*MsgQueue).CaretInfo);

        SafeGui.flags = if (*CaretInfo).Visible != 0 {
            GUI_CARETBLINKING
        } else {
            0
        };

        SafeGui.hwndMenuOwner = (*MsgQueue).MenuOwner;

        if !(*MsgQueue).MenuOwner.is_null() {
            SafeGui.flags |= GUI_INMENUMODE | (*MsgQueue).MenuState as u32;
        }

        if !(*MsgQueue).MoveSize.is_null() {
            SafeGui.flags |= GUI_INMOVESIZE;
        }

        SafeGui.hwndActive = if !(*MsgQueue).spwndActive.is_null() {
            UserHMGetHandle((*MsgQueue).spwndActive as *mut core::ffi::c_void)
        } else {
            core::ptr::null_mut()
        };
        SafeGui.hwndFocus = if !(*MsgQueue).spwndFocus.is_null() {
            UserHMGetHandle((*MsgQueue).spwndFocus as *mut core::ffi::c_void)
        } else {
            core::ptr::null_mut()
        };
        SafeGui.hwndCapture = if !(*MsgQueue).spwndCapture.is_null() {
            UserHMGetHandle((*MsgQueue).spwndCapture as *mut core::ffi::c_void)
        } else {
            core::ptr::null_mut()
        };
        SafeGui.hwndMoveSize = (*MsgQueue).MoveSize;
        SafeGui.hwndCaret = (*CaretInfo).hWnd;

        SafeGui.rcCaret.left = (*CaretInfo).Pos.x;
        SafeGui.rcCaret.top = (*CaretInfo).Pos.y;
        SafeGui.rcCaret.right = SafeGui.rcCaret.left + (*CaretInfo).Size.cx;
        SafeGui.rcCaret.bottom = SafeGui.rcCaret.top + (*CaretInfo).Size.cy;

        Status = MmCopyToCaller(
            lpgui as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(SafeGui) as *mut core::ffi::c_void,
            core::mem::size_of::<GUITHREADINFO>(),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'block;
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetGuiResources(
    hProcess: HANDLE,
    uiFlags: DWORD,
) -> DWORD {
    let mut Process: PEPROCESS = core::ptr::null_mut();
    let W32Process: PPROCESSINFO;
    let Status: NTSTATUS;
    let mut Ret: DWORD = 0;

    UserEnterShared();

    Status = ObReferenceObjectByHandle(
        hProcess,
        PROCESS_QUERY_INFORMATION,
        *PsProcessType as _,
        ExGetPreviousMode(),
        &mut Process as *mut PEPROCESS as *mut PVOID,
        core::ptr::null_mut(),
    );

    'exit: {
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        }

        W32Process = (*Process).Win32Process as PPROCESSINFO;
        if W32Process.is_null() {
            ObDereferenceObject(Process as PVOID);
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        match uiFlags {
            GR_GDIOBJECTS => {
                Ret = ppi_GDIHandleCount(W32Process) as DWORD;
            }
            GR_USEROBJECTS => {
                Ret = ppi_UserHandleCount(W32Process) as DWORD;
            }
            _ => {
                EngSetLastError(ERROR_INVALID_PARAMETER);
            }
        }

        ObDereferenceObject(Process as PVOID);
    }

    UserLeave();
    Ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetWindowState(pWnd: PWND, Flag: UINT) {
    IntSetWindowState_impl(pWnd, Flag)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetWindowState(pWnd: PWND, Flag: UINT) {
    IntSetWindowState_impl(pWnd, Flag)
}

unsafe fn IntSetWindowState_impl(pWnd: PWND, Flag: UINT) {
    let bit: UINT;
    if (*gptiCurrent).ppi != (*head_pti(core::ptr::addr_of!((*pWnd).head))).ppi {
        return;
    }
    bit = 1u32 << LOWORD(Flag);
    TRACE!(b"SWS %x\n\0".as_ptr() as *const i8, bit);
    match HIWORD(Flag) {
        0 => {
            (*pWnd).state |= bit;
        }
        1 => {
            (*pWnd).state2 |= bit;
        }
        2 => {
            (*pWnd).ExStyle2 |= bit;
        }
        _ => {}
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntClearWindowState(pWnd: PWND, Flag: UINT) {
    IntClearWindowState_impl(pWnd, Flag)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntClearWindowState(pWnd: PWND, Flag: UINT) {
    IntClearWindowState_impl(pWnd, Flag)
}

unsafe fn IntClearWindowState_impl(pWnd: PWND, Flag: UINT) {
    let bit: UINT;
    if (*gptiCurrent).ppi != (*head_pti(core::ptr::addr_of!((*pWnd).head))).ppi {
        return;
    }
    bit = 1u32 << ((Flag & 0xFFFF) as UINT);
    match (Flag >> 16) & 0xFFFF {
        0 => {
            (*pWnd).state &= !bit;
        }
        1 => {
            (*pWnd).state2 &= !bit;
        }
        2 => {
            (*pWnd).ExStyle2 &= !bit;
        }
        _ => {}
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn IntSafeCopyUnicodeString(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {
    IntSafeCopyUnicodeString_impl(Dest, Source)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn IntSafeCopyUnicodeString(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {
    IntSafeCopyUnicodeString_impl(Dest, Source)
}

unsafe fn IntSafeCopyUnicodeString_impl(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {

    const TAG_STRING: u32 = u32::from_be_bytes(*b" RTS"); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

    let mut Status: NTSTATUS;
    let Src: PWSTR;

    Status = MmCopyFromCaller(
        Dest as *mut core::ffi::c_void,
        Source as *const core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*Dest).Length > 0x4000 {
        return STATUS_UNSUCCESSFUL;
    }

    Src = (*Dest).Buffer;
    (*Dest).Buffer = core::ptr::null_mut();
    (*Dest).MaximumLength = (*Dest).Length;

    if (*Dest).Length > 0 && !Src.is_null() {
        (*Dest).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (*Dest).MaximumLength as _,
            TAG_STRING,
        ) as PWSTR;
        if (*Dest).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        Status = MmCopyFromCaller(
            (*Dest).Buffer as *mut core::ffi::c_void,
            Src as *const core::ffi::c_void,
            (*Dest).Length as usize,
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag((*Dest).Buffer as *mut core::ffi::c_void, TAG_STRING);
            (*Dest).Buffer = core::ptr::null_mut();
            return Status;
        }

        return STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSafeCopyUnicodeStringTerminateNULL(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {
    IntSafeCopyUnicodeStringTerminateNULL_impl(Dest, Source)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSafeCopyUnicodeStringTerminateNULL(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {
    IntSafeCopyUnicodeStringTerminateNULL_impl(Dest, Source)
}

unsafe fn IntSafeCopyUnicodeStringTerminateNULL_impl(
    Dest: PUNICODE_STRING,
    Source: PUNICODE_STRING,
) -> NTSTATUS {

    const TAG_STRING: u32 = u32::from_be_bytes(*b" RTS"); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

    let mut Status: NTSTATUS;
    let mut Src: PWSTR;

    Status = MmCopyFromCaller(
        Dest as *mut core::ffi::c_void,
        Source as *const core::ffi::c_void,
        core::mem::size_of::<_UNICODE_STRING>(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*Dest).Length > 0x4000 {
        return STATUS_UNSUCCESSFUL;
    }

    Src = (*Dest).Buffer;
    (*Dest).Buffer = core::ptr::null_mut();
    (*Dest).MaximumLength = 0;

    if (*Dest).Length > 0 && !Src.is_null() {
        (*Dest).MaximumLength = (*Dest).Length + core::mem::size_of::<u16>() as u16;
        (*Dest).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            (*Dest).MaximumLength as _,
            TAG_STRING,
        ) as PWSTR;
        if (*Dest).Buffer.is_null() {
            return STATUS_NO_MEMORY;
        }

        Status = MmCopyFromCaller(
            (*Dest).Buffer as *mut core::ffi::c_void,
            Src as *const core::ffi::c_void,
            (*Dest).Length as usize,
        );
        if !NT_SUCCESS(Status) {
            ExFreePoolWithTag((*Dest).Buffer as *mut core::ffi::c_void, TAG_STRING);
            (*Dest).Buffer = core::ptr::null_mut();
            return Status;
        }

        Src = ((*Dest).Buffer as PBYTE).add((*Dest).Length as usize) as PWSTR;
        *Src = 0;

        return STATUS_SUCCESS;
    }

    STATUS_SUCCESS
}

unsafe extern "system" fn UserDbgAssertThreadInfo(_showCaller: BOOL) {
    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let Teb: PTEB = NtCurrentTeb() as PTEB;
    let pci: PCLIENTINFO = GetWin32ClientInfo();

    let _ = Teb;
    let _ = pti;
    let _ = ppi;
    let _ = pci;
}
#[no_mangle]
pub unsafe extern "system" fn UserDbgPreServiceHook(ulSyscallId: ULONG, pulArguments: PULONG_PTR) {
    let _ = ulSyscallId;
    let _ = pulArguments;
    UserDbgAssertThreadInfo(0);
}
#[no_mangle]
pub unsafe extern "system" fn UserDbgPostServiceHook(ulSyscallId: ULONG, ulResult: ULONG_PTR) -> ULONG_PTR {
    let _ = ulSyscallId;

    UserDbgAssertThreadInfo(TRUE as BOOL);

    ulResult
}

#[no_mangle]
pub unsafe extern "C" fn GetW32ProcessInfo() -> PPROCESSINFO {
    PsGetCurrentProcessWin32Process() as PPROCESSINFO
}

#[no_mangle]
pub unsafe extern "C" fn GetW32ThreadInfo() -> PTHREADINFO {
    UserDbgAssertThreadInfo(1);
    PsGetCurrentThreadWin32Thread() as PTHREADINFO
}

#[no_mangle]
pub unsafe extern "C" fn GetProcessLuid(
    mut Thread: ntkrnl_sys::ddk::PETHREAD,
    mut Process: ntkrnl_sys::ddk::PEPROCESS,
    Luid: ntkrnl_sys::ddk::PLUID,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Token: ntkrnl_sys::ddk::PACCESS_TOKEN = core::ptr::null_mut();
    let mut ImpersonationLevel: ntkrnl_sys::ddk::SECURITY_IMPERSONATION_LEVEL = 0;
    let mut CopyOnOpen: BOOLEAN = 0;
    let mut EffectiveOnly: BOOLEAN = 0;

    if !Thread.is_null() && !Process.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if Thread.is_null() && Process.is_null() {
        Thread = PsGetCurrentThread();
    }

    if !Thread.is_null() {
        Token = PsReferenceImpersonationToken(
            Thread,
            &mut CopyOnOpen,
            &mut EffectiveOnly,
            &mut ImpersonationLevel,
        );

        if Token.is_null() {
            Process = PsGetThreadProcess(Thread);
        }
    }

    if Token.is_null() && !Process.is_null() {
        Token = PsReferencePrimaryToken(Process) as ntkrnl_sys::ddk::PACCESS_TOKEN;

        if Token.is_null() {
            return STATUS_NO_TOKEN;
        }
    }

    Status = SeQueryAuthenticationIdToken(Token, Luid);

    ObDereferenceObject(Token);
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS Win32k subsystem
//  * PURPOSE:          Miscellaneous User functions
//  * FILE:             win32ss/user/ntuser/misc.c
//  * PROGRAMER:        Ge van Geldorp (ge@gse.nl)
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserMisc);
//
//
// /*
//  * NOTE: _scwprintf() is NOT exported by ntoskrnl.exe,
//  * only _vscwprintf() is, so we need to implement it here.
//  * Code comes from sdk/lib/crt/printf/_scwprintf.c .
//  * See also win32ss/user/winsrv/usersrv/harderror.c .
//  */
// int
// __cdecl
// _scwprintf(
//     const wchar_t *format,
//     ...)
// {
//     int len;
//     va_list args;
//
//     va_start(args, format);
//     len = _vscwprintf(format, args);
//     va_end(args);
//
//     return len;
// }
//
//
// /*
//  * Test the Thread to verify and validate it. Hard to the core tests are required.
//  * Win: PtiFromThreadId
//  */
// PTHREADINFO
// FASTCALL
// IntTID2PTI(HANDLE id)
// {
//    NTSTATUS Status;
//    PETHREAD Thread;
//    PTHREADINFO pti;
//    Status = PsLookupThreadByThreadId(id, &Thread);
//    if (!NT_SUCCESS(Status))
//    {
//       return NULL;
//    }
//    if (PsIsThreadTerminating(Thread))
//    {
//       ObDereferenceObject(Thread);
//       return NULL;
//    }
//    pti = PsGetThreadWin32Thread(Thread);
//    if (!pti)
//    {
//       ObDereferenceObject(Thread);
//       return NULL;
//    }
//    // Validate and verify!
//    _SEH2_TRY
//    {
//       if (pti->TIF_flags & TIF_INCLEANUP) pti = NULL;
//       if (pti && !(pti->TIF_flags & TIF_GUITHREADINITIALIZED)) pti = NULL;
//       if (PsGetThreadId(Thread) != id) pti = NULL;
//    }
//    _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//    {
//       pti = NULL;
//    }
//    _SEH2_END
//    ObDereferenceObject(Thread);
//    return pti;
// }
//
// /**
//  * @see https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-2000-server/cc976564%28v=technet.10%29
//  */
// DWORD
// FASTCALL
// UserGetLanguageToggle(
//      PCWSTR pszType,
//      DWORD dwDefaultValue)
// {
//     NTSTATUS Status;
//     DWORD dwValue = dwDefaultValue;
//     WCHAR szBuff[4];
//
//     Status = RegReadUserSetting(L"Keyboard Layout\\Toggle", pszType, REG_SZ, szBuff, sizeof(szBuff));
//     if (NT_SUCCESS(Status))
//     {
//         szBuff[RTL_NUMBER_OF(szBuff) - 1] = UNICODE_NULL;
//         dwValue = _wtoi(szBuff);
//     }
//
//     TRACE("%ls: %lu\n", pszType, dwValue);
//     return dwValue;
// }
//
// USHORT
// FASTCALL
// UserGetLanguageID(VOID)
// {
//   HANDLE KeyHandle;
//   OBJECT_ATTRIBUTES ObAttr;
// //  https://learn.microsoft.com/en-us/troubleshoot/windows-server/setup-upgrade-and-drivers/use-language-id-identify-language-pack
//   ULONG Ret = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
//   PKEY_VALUE_PARTIAL_INFORMATION pKeyInfo;
//   ULONG Size = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + MAX_PATH*sizeof(WCHAR);
//   UNICODE_STRING Language;
//
//   RtlInitUnicodeString( &Language,
//     L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Nls\\Language");
//
//   InitializeObjectAttributes( &ObAttr,
//                               &Language,
//                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                               NULL,
//                               NULL);
//
//   if ( NT_SUCCESS(ZwOpenKey(&KeyHandle, KEY_READ, &ObAttr)))
//   {
//      pKeyInfo = ExAllocatePoolWithTag(PagedPool, Size, TAG_STRING);
//      if ( pKeyInfo )
//      {
//         RtlInitUnicodeString(&Language, L"Default");
//
//         if ( NT_SUCCESS(ZwQueryValueKey( KeyHandle,
//                                          &Language,
//                         KeyValuePartialInformation,
//                                           pKeyInfo,
//                                               Size,
//                                              &Size)) )
//       {
//         RtlInitUnicodeString(&Language, (PWSTR)pKeyInfo->Data);
//         if (!NT_SUCCESS(RtlUnicodeStringToInteger(&Language, 16, &Ret)))
//         {
//             Ret = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
//         }
//       }
//       ExFreePoolWithTag(pKeyInfo, TAG_STRING);
//     }
//     ZwClose(KeyHandle);
//   }
//   TRACE("Language ID = %x\n",Ret);
//   return (USHORT) Ret;
// }
//
// HBRUSH
// FASTCALL
// GetControlColor(
//    PWND pwndParent,
//    PWND pwnd,
//    HDC hdc,
//    UINT CtlMsg)
// {
//     HBRUSH hBrush;
//
//     if (!pwndParent) pwndParent = pwnd;
//
//     if ( pwndParent->head.pti->ppi != PsGetCurrentProcessWin32Process())
//     {
//        return (HBRUSH)IntDefWindowProc( pwndParent, CtlMsg, (WPARAM)hdc, (LPARAM)UserHMGetHandle(pwnd), FALSE);
//     }
//
//     hBrush = (HBRUSH)co_IntSendMessage( UserHMGetHandle(pwndParent), CtlMsg, (WPARAM)hdc, (LPARAM)UserHMGetHandle(pwnd));
//
//     if (!hBrush || !GreIsHandleValid(hBrush))
//     {
//        hBrush = (HBRUSH)IntDefWindowProc( pwndParent, CtlMsg, (WPARAM)hdc, (LPARAM)UserHMGetHandle(pwnd), FALSE);
//     }
//     return hBrush;
// }
//
// HBRUSH
// FASTCALL
// GetControlBrush(
//    PWND pwnd,
//    HDC  hdc,
//    UINT ctlType)
// {
//     PWND pwndParent = IntGetParent(pwnd);
//     return GetControlColor( pwndParent, pwnd, hdc, ctlType);
// }
//
// HBRUSH
// __stdcall
// NtUserGetControlBrush(
//    HWND hwnd,
//    HDC  hdc,
//    UINT ctlType)
// {
//    PWND pwnd;
//    HBRUSH hBrush = NULL;
//
//    UserEnterExclusive();
//    if ( (pwnd = UserGetWindowObject(hwnd)) &&
//        ((ctlType - WM_CTLCOLORMSGBOX) < CTLCOLOR_MAX) &&
//         hdc )
//    {
//       hBrush = GetControlBrush(pwnd, hdc, ctlType);
//    }
//    UserLeave();
//    return hBrush;
// }
//
// /*
//  * Called from PaintRect, works almost like wine PaintRect16 but returns hBrush.
//  */
// HBRUSH
// __stdcall
// NtUserGetControlColor(
//    HWND hwndParent,
//    HWND hwnd,
//    HDC hdc,
//    UINT CtlMsg) // Wine PaintRect: WM_CTLCOLORMSGBOX + hbrush
// {
//    PWND pwnd, pwndParent = NULL;
//    HBRUSH hBrush = NULL;
//
//    UserEnterExclusive();
//    if ( (pwnd = UserGetWindowObject(hwnd)) &&
//        ((CtlMsg - WM_CTLCOLORMSGBOX) < CTLCOLOR_MAX) &&
//         hdc )
//    {
//       if (hwndParent) pwndParent = UserGetWindowObject(hwndParent);
//       hBrush = GetControlColor( pwndParent, pwnd, hdc, CtlMsg);
//    }
//    UserLeave();
//    return hBrush;
// }
//
// /*
//  * @unimplemented
//  */
// DWORD_PTR __stdcall
// NtUserGetThreadState(
//    DWORD Routine)
// {
//    DWORD_PTR ret = 0;
//    PTHREADINFO pti;
//
//    TRACE("Enter NtUserGetThreadState\n");
//    if (Routine != THREADSTATE_GETTHREADINFO)
//    {
//        UserEnterShared();
//    }
//    else
//    {
//        UserEnterExclusive();
//    }
//
//    pti = PsGetCurrentThreadWin32Thread();
//
//    switch (Routine)
//    {
//       case THREADSTATE_GETTHREADINFO:
//          ret = TRUE;
//          break;
//       case THREADSTATE_FOCUSWINDOW:
//          ret = (DWORD_PTR)IntGetThreadFocusWindow();
//          break;
//       case THREADSTATE_CAPTUREWINDOW:
//          /* FIXME: Should use UserEnterShared */
//          ret = (DWORD_PTR)IntGetCapture();
//          break;
//       case THREADSTATE_PROGMANWINDOW: /* FIXME: Delete this HACK */
//          ret = (DWORD_PTR)GetW32ThreadInfo()->pDeskInfo->hProgmanWindow;
//          break;
//       case THREADSTATE_TASKMANWINDOW: /* FIXME: Delete this HACK */
//          ret = (DWORD_PTR)GetW32ThreadInfo()->pDeskInfo->hTaskManWindow;
//          break;
//       case THREADSTATE_ACTIVEWINDOW:
//          ret = (DWORD_PTR)UserGetActiveWindow();
//          break;
//       case THREADSTATE_INSENDMESSAGE:
//          {
//            PUSER_SENT_MESSAGE Message =
//                 ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->pusmCurrent;
//            TRACE("THREADSTATE_INSENDMESSAGE\n");
//
//            ret = ISMEX_NOSEND;
//            if (Message)
//            {
//              if (Message->ptiSender)
//                 ret = ISMEX_SEND;
//              else
//              {
//                 if (Message->CompletionCallback)
//                    ret = ISMEX_CALLBACK;
//                 else
//                    ret = ISMEX_NOTIFY;
//              }
//              /* If ReplyMessage */
//              if (Message->QS_Flags & QS_SMRESULT) ret |= ISMEX_REPLIED;
//            }
//
//            break;
//          }
//       case THREADSTATE_GETMESSAGETIME:
//          ret = ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->timeLast;
//          break;
//
//       case THREADSTATE_UPTIMELASTREAD:
//          pti->pcti->timeLastRead = EngGetTickCount32();
//          break;
//
//       case THREADSTATE_GETINPUTSTATE:
//          ret = LOWORD(IntGetQueueStatus(QS_POSTMESSAGE|QS_TIMER|QS_PAINT|QS_SENDMESSAGE|QS_INPUT)) & (QS_KEY | QS_MOUSEBUTTON);
//          break;
//
//       case THREADSTATE_FOREGROUNDTHREAD:
//          ret = (gpqForeground == GetW32ThreadInfo()->MessageQueue);
//          break;
//       case THREADSTATE_GETCURSOR:
//          ret = (DWORD_PTR) (GetW32ThreadInfo()->MessageQueue->CursorObject ?
//                             UserHMGetHandle(GetW32ThreadInfo()->MessageQueue->CursorObject) : 0);
//          break;
//       case THREADSTATE_GETMESSAGEEXTRAINFO:
//          ret = (DWORD_PTR)MsqGetMessageExtraInfo();
//          break;
//       case THREADSTATE_DEFAULTIMEWINDOW:
//          if (pti->spwndDefaultIme)
//             ret = (ULONG_PTR)UserHMGetHandle(pti->spwndDefaultIme);
//          break;
//       case THREADSTATE_DEFAULTINPUTCONTEXT:
//          if (pti->spDefaultImc)
//              ret = (ULONG_PTR)UserHMGetHandle(pti->spDefaultImc);
//          break;
//       case THREADSTATE_CHANGEBITS:
//          ret = pti->pcti->fsChangeBits;
//          break;
//       case THREADSTATE_IMECOMPATFLAGS:
//          ret = pti->ppi->dwImeCompatFlags;
//          break;
//       case THREADSTATE_OLDKEYBOARDLAYOUT:
//          ret = (ULONG_PTR)pti->hklPrev;
//          break;
//       case THREADSTATE_ISWINLOGON:
//          ret = (gpidLogon == PsGetCurrentProcessId());
//          break;
//       case THREADSTATE_UNKNOWN_0x10:
//          FIXME("stub\n");
//          break;
//       case THREADSTATE_CHECKCONIME:
//          ret = (IntTID2PTI(UlongToHandle(pti->rpdesk->dwConsoleThreadId)) == pti);
//          break;
//    }
//
//    TRACE("Leave NtUserGetThreadState, ret=%lu\n", ret);
//    UserLeave();
//
//    return ret;
// }
//
// DWORD
// __stdcall
// NtUserSetThreadState(
//    DWORD Set,
//    DWORD Flags)
// {
//    PTHREADINFO pti;
//    DWORD Ret = 0;
//    // Test the only flags user can change.
//    if (Set & ~(QF_FF10STATUS|QF_DIALOGACTIVE|QF_TABSWITCHING|QF_FMENUSTATUS|QF_FMENUSTATUSBREAK)) return 0;
//    if (Flags & ~(QF_FF10STATUS|QF_DIALOGACTIVE|QF_TABSWITCHING|QF_FMENUSTATUS|QF_FMENUSTATUSBREAK)) return 0;
//    UserEnterExclusive();
//    pti = PsGetCurrentThreadWin32Thread();
//    if (pti->MessageQueue)
//    {
//       Ret = pti->MessageQueue->QF_flags;    // Get the queue flags.
//       if (Set)
//          pti->MessageQueue->QF_flags |= (Set&Flags); // Set the queue flags.
//       else
//       {
//          if (Flags) pti->MessageQueue->QF_flags &= ~Flags; // Clr the queue flags.
//       }
//    }
//    UserLeave();
//    return Ret;
// }
//
// UINT
// __stdcall
// NtUserGetDoubleClickTime(VOID)
// {
//    UINT Result;
//
//    TRACE("Enter NtUserGetDoubleClickTime\n");
//    UserEnterShared();
//
//    // FIXME: Check if this works on non-interactive winsta
//    Result = gspv.iDblClickTime;
//
//    TRACE("Leave NtUserGetDoubleClickTime, ret=%u\n", Result);
//    UserLeave();
//    return Result;
// }
//
// BOOL
// __stdcall
// NtUserGetGUIThreadInfo(
//    DWORD idThread, /* If NULL use foreground thread */
//    LPGUITHREADINFO lpgui)
// {
//    NTSTATUS Status;
//    PTHRDCARETINFO CaretInfo;
//    GUITHREADINFO SafeGui;
//    PDESKTOP Desktop;
//    PUSER_MESSAGE_QUEUE MsgQueue;
//    PTHREADINFO W32Thread, pti;
//    BOOL Ret = FALSE;
//
//    TRACE("Enter NtUserGetGUIThreadInfo\n");
//    UserEnterShared();
//
//    Status = MmCopyFromCaller(&SafeGui, lpgui, sizeof(DWORD));
//    if(!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Exit; // Return FALSE
//    }
//
//    if(SafeGui.cbSize != sizeof(GUITHREADINFO))
//    {
//       EngSetLastError(ERROR_INVALID_PARAMETER);
//       goto Exit; // Return FALSE
//    }
//
//    if (idThread)
//    {
//       pti = PsGetCurrentThreadWin32Thread();
//
//       // Validate Tread ID
//       W32Thread = IntTID2PTI(UlongToHandle(idThread));
//
//       if ( !W32Thread )
//       {
//           EngSetLastError(ERROR_ACCESS_DENIED);
//           goto Exit; // Return FALSE
//       }
//
//       Desktop = W32Thread->rpdesk;
//
//       // Check Desktop and it must be the same as current.
//       if ( !Desktop || Desktop != pti->rpdesk )
//       {
//           EngSetLastError(ERROR_ACCESS_DENIED);
//           goto Exit; // Return FALSE
//       }
//
//       if ( W32Thread->MessageQueue )
//         MsgQueue = W32Thread->MessageQueue;
//       else
//       {
//         MsgQueue = Desktop->ActiveMessageQueue;
//       }
//    }
//    else
//    {  /* Get the foreground thread */
//       /* FIXME: Handle NULL queue properly? */
//       MsgQueue = IntGetFocusMessageQueue();
//       if(!MsgQueue)
//       {
//         EngSetLastError(ERROR_ACCESS_DENIED);
//         goto Exit; // Return FALSE
//       }
//    }
//
//    CaretInfo = &MsgQueue->CaretInfo;
//
//    SafeGui.flags = (CaretInfo->Visible ? GUI_CARETBLINKING : 0);
// /*
//    if (W32Thread->pMenuState->pGlobalPopupMenu)
//    {
//        SafeGui.flags |= GUI_INMENUMODE;
//
//        if (W32Thread->pMenuState->pGlobalPopupMenu->spwndNotify)
//           SafeGui.hwndMenuOwner = UserHMGetHandle(W32Thread->pMenuState->pGlobalPopupMenu->spwndNotify);
//
//        if (W32Thread->pMenuState->pGlobalPopupMenu->fHasMenuBar)
//        {
//           if (W32Thread->pMenuState->pGlobalPopupMenu->fIsSysMenu)
//           {
//              SafeGui.flags |= GUI_SYSTEMMENUMODE;
//           }
//        }
//        else
//        {
//           SafeGui.flags |= GUI_POPUPMENUMODE;
//        }
//    }
//  */
//    SafeGui.hwndMenuOwner = MsgQueue->MenuOwner;
//
//    if (MsgQueue->MenuOwner)
//       SafeGui.flags |= GUI_INMENUMODE | MsgQueue->MenuState;
//
//    if (MsgQueue->MoveSize)
//       SafeGui.flags |= GUI_INMOVESIZE;
//
//    /* FIXME: Add flag GUI_16BITTASK */
//
//    SafeGui.hwndActive = MsgQueue->spwndActive ? UserHMGetHandle(MsgQueue->spwndActive) : 0;
//    SafeGui.hwndFocus = MsgQueue->spwndFocus ? UserHMGetHandle(MsgQueue->spwndFocus) : 0;
//    SafeGui.hwndCapture = MsgQueue->spwndCapture ? UserHMGetHandle(MsgQueue->spwndCapture) : 0;
//    SafeGui.hwndMoveSize = MsgQueue->MoveSize;
//    SafeGui.hwndCaret = CaretInfo->hWnd;
//
//    SafeGui.rcCaret.left = CaretInfo->Pos.x;
//    SafeGui.rcCaret.top = CaretInfo->Pos.y;
//    SafeGui.rcCaret.right = SafeGui.rcCaret.left + CaretInfo->Size.cx;
//    SafeGui.rcCaret.bottom = SafeGui.rcCaret.top + CaretInfo->Size.cy;
//
//    Status = MmCopyToCaller(lpgui, &SafeGui, sizeof(GUITHREADINFO));
//    if(!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Exit; // Return FALSE
//    }
//
//    Ret = TRUE;
//
// Exit:
//    TRACE("Leave NtUserGetGUIThreadInfo, ret=%i\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
//
// DWORD
// __stdcall
// NtUserGetGuiResources(
//    HANDLE hProcess,
//    DWORD uiFlags)
// {
//    PEPROCESS Process;
//    PPROCESSINFO W32Process;
//    NTSTATUS Status;
//    DWORD Ret = 0;
//
//    TRACE("Enter NtUserGetGuiResources\n");
//    UserEnterShared();
//
//    Status = ObReferenceObjectByHandle(hProcess,
//                                       PROCESS_QUERY_INFORMATION,
//                                       *PsProcessType,
//                                       ExGetPreviousMode(),
//                                       (PVOID*)&Process,
//                                       NULL);
//
//    if(!NT_SUCCESS(Status))
//    {
//       SetLastNtError(Status);
//       goto Exit; // Return 0
//    }
//
//    W32Process = (PPROCESSINFO)Process->Win32Process;
//    if(!W32Process)
//    {
//       ObDereferenceObject(Process);
//       EngSetLastError(ERROR_INVALID_PARAMETER);
//       goto Exit; // Return 0
//    }
//
//    switch(uiFlags)
//    {
//       case GR_GDIOBJECTS:
//          {
//             Ret = (DWORD)W32Process->GDIHandleCount;
//             break;
//          }
//       case GR_USEROBJECTS:
//          {
//             Ret = (DWORD)W32Process->UserHandleCount;
//             break;
//          }
//       default:
//          {
//             EngSetLastError(ERROR_INVALID_PARAMETER);
//             break;
//          }
//    }
//
//    ObDereferenceObject(Process);
//
// Exit:
//    TRACE("Leave NtUserGetGuiResources, ret=%lu\n", Ret);
//    UserLeave();
//    return Ret;
// }
//
// VOID FASTCALL
// IntSetWindowState(PWND pWnd, UINT Flag)
// {
//    UINT bit;
//    if (gptiCurrent->ppi != pWnd->head.pti->ppi) return;
//    bit = 1 << LOWORD(Flag);
//    TRACE("SWS %x\n",bit);
//    switch(HIWORD(Flag))
//    {
//       case 0:
//           pWnd->state |= bit;
//           break;
//       case 1:
//           pWnd->state2 |= bit;
//           break;
//       case 2:
//           pWnd->ExStyle2 |= bit;
//           break;
//    }
// }
//
// VOID FASTCALL
// IntClearWindowState(PWND pWnd, UINT Flag)
// {
//    UINT bit;
//    if (gptiCurrent->ppi != pWnd->head.pti->ppi) return;
//    bit = 1 << LOWORD(Flag);
//    TRACE("CWS %x\n",bit);
//    switch(HIWORD(Flag))
//    {
//       case 0:
//           pWnd->state &= ~bit;
//           break;
//       case 1:
//           pWnd->state2 &= ~bit;
//           break;
//       case 2:
//           pWnd->ExStyle2 &= ~bit;
//           break;
//    }
// }
//
// NTSTATUS FASTCALL
// IntSafeCopyUnicodeString(PUNICODE_STRING Dest,
//                          PUNICODE_STRING Source)
// {
//    NTSTATUS Status;
//    PWSTR Src;
//
//    Status = MmCopyFromCaller(Dest, Source, sizeof(UNICODE_STRING));
//    if(!NT_SUCCESS(Status))
//    {
//       return Status;
//    }
//
//    if(Dest->Length > 0x4000)
//    {
//       return STATUS_UNSUCCESSFUL;
//    }
//
//    Src = Dest->Buffer;
//    Dest->Buffer = NULL;
//    Dest->MaximumLength = Dest->Length;
//
//    if(Dest->Length > 0 && Src)
//    {
//       Dest->Buffer = ExAllocatePoolWithTag(PagedPool, Dest->MaximumLength, TAG_STRING);
//       if(!Dest->Buffer)
//       {
//          return STATUS_NO_MEMORY;
//       }
//
//       Status = MmCopyFromCaller(Dest->Buffer, Src, Dest->Length);
//       if(!NT_SUCCESS(Status))
//       {
//          ExFreePoolWithTag(Dest->Buffer, TAG_STRING);
//          Dest->Buffer = NULL;
//          return Status;
//       }
//
//
//       return STATUS_SUCCESS;
//    }
//
//    /* String is empty */
//    return STATUS_SUCCESS;
// }
//
// NTSTATUS FASTCALL
// IntSafeCopyUnicodeStringTerminateNULL(PUNICODE_STRING Dest,
//                                       PUNICODE_STRING Source)
// {
//    NTSTATUS Status;
//    PWSTR Src;
//
//    Status = MmCopyFromCaller(Dest, Source, sizeof(UNICODE_STRING));
//    if(!NT_SUCCESS(Status))
//    {
//       return Status;
//    }
//
//    if(Dest->Length > 0x4000)
//    {
//       return STATUS_UNSUCCESSFUL;
//    }
//
//    Src = Dest->Buffer;
//    Dest->Buffer = NULL;
//    Dest->MaximumLength = 0;
//
//    if(Dest->Length > 0 && Src)
//    {
//       Dest->MaximumLength = Dest->Length + sizeof(WCHAR);
//       Dest->Buffer = ExAllocatePoolWithTag(PagedPool, Dest->MaximumLength, TAG_STRING);
//       if(!Dest->Buffer)
//       {
//          return STATUS_NO_MEMORY;
//       }
//
//       Status = MmCopyFromCaller(Dest->Buffer, Src, Dest->Length);
//       if(!NT_SUCCESS(Status))
//       {
//          ExFreePoolWithTag(Dest->Buffer, TAG_STRING);
//          Dest->Buffer = NULL;
//          return Status;
//       }
//
//       /* Make sure the string is null-terminated */
//       Src = (PWSTR)((PBYTE)Dest->Buffer + Dest->Length);
//       *Src = L'\0';
//
//       return STATUS_SUCCESS;
//    }
//
//    /* String is empty */
//    return STATUS_SUCCESS;
// }
//
// void UserDbgAssertThreadInfo(BOOL showCaller)
// {
//     PTEB Teb;
//     PPROCESSINFO ppi;
//     PCLIENTINFO pci;
//     PTHREADINFO pti;
//
//     ppi = PsGetCurrentProcessWin32Process();
//     pti = PsGetCurrentThreadWin32Thread();
//     Teb = NtCurrentTeb();
//     pci = GetWin32ClientInfo();
//
//     ASSERT(Teb);
//     ASSERT(pti);
//     ASSERT(pti->ppi == ppi);
//     ASSERT(pti->pClientInfo == pci);
//     ASSERT(Teb->Win32ThreadInfo == pti);
//     ASSERT(pci->ppi == ppi);
//     ASSERT(pci->fsHooks == pti->fsHooks);
//     ASSERT(pci->ulClientDelta == DesktopHeapGetUserDelta());
//     if (pti->pcti && pci->pDeskInfo)
//         ASSERT(pci->pClientThreadInfo == (PVOID)((ULONG_PTR)pti->pcti - pci->ulClientDelta));
//     if (pti->KeyboardLayout)
//         ASSERT(pci->hKL == pti->KeyboardLayout->hkl);
//     if(pti->rpdesk != NULL)
//         ASSERT(pti->pDeskInfo == pti->rpdesk->pDeskInfo);
//
//     /*too bad we still get this assertion*/
//
//     // Why? Not all flags are passed to the user and doing so could crash the system........
//
//     /* ASSERT(pci->dwTIFlags == pti->TIF_flags); */
// /*    if(pci->dwTIFlags != pti->TIF_flags)
//     {
//         ERR("pci->dwTIFlags(0x%x) doesn't match pti->TIF_flags(0x%x)\n", pci->dwTIFlags, pti->TIF_flags);
//         if(showCaller)
//         {
//             DbgPrint("Caller:\n");
//             KeRosDumpStackFrames(NULL, 10);
//         }
//         pci->dwTIFlags = pti->TIF_flags;
//     }
// */
// }
//
// void
// NTAPI
// UserDbgPreServiceHook(ULONG ulSyscallId, PULONG_PTR pulArguments)
// {
//     UserDbgAssertThreadInfo(FALSE);
// }
//
// ULONG_PTR
// NTAPI
// UserDbgPostServiceHook(ULONG ulSyscallId, ULONG_PTR ulResult)
// {
//     /* Make sure that the first syscall is NtUserInitialize */
//     /* too bad this fails */
//     // ASSERT(gpepCSRSS);
//
//     UserDbgAssertThreadInfo(TRUE);
//
//     return ulResult;
// }
//
//
// PPROCESSINFO
// GetW32ProcessInfo(VOID)
// {
//     return (PPROCESSINFO)PsGetCurrentProcessWin32Process();
// }
//
// PTHREADINFO
// GetW32ThreadInfo(VOID)
// {
//     UserDbgAssertThreadInfo(TRUE);
//     return (PTHREADINFO)PsGetCurrentThreadWin32Thread();
// }
//
//
// NTSTATUS
// GetProcessLuid(
//     IN PETHREAD Thread OPTIONAL,
//     IN PEPROCESS Process OPTIONAL,
//     OUT PLUID Luid)
// {
//     NTSTATUS Status;
//     PACCESS_TOKEN Token = NULL;
//     SECURITY_IMPERSONATION_LEVEL ImpersonationLevel;
//     BOOLEAN CopyOnOpen, EffectiveOnly;
//
//     if (Thread && Process)
//         return STATUS_INVALID_PARAMETER;
//
//     /* If nothing has been specified, use the current thread */
//     if (!Thread && !Process)
//         Thread = PsGetCurrentThread();
//
//     if (Thread)
//     {
//         /* Use a thread token */
//         ASSERT(!Process);
//         Token = PsReferenceImpersonationToken(Thread,
//                                               &CopyOnOpen,
//                                               &EffectiveOnly,
//                                               &ImpersonationLevel);
//
//         /* If we don't have a thread token, use a process token */
//         if (!Token)
//             Process = PsGetThreadProcess(Thread);
//     }
//     if (!Token && Process)
//     {
//         /* Use a process token */
//         Token = PsReferencePrimaryToken(Process);
//
//         /* If we don't have a token, fail */
//         if (!Token)
//             return STATUS_NO_TOKEN;
//     }
//     ASSERT(Token);
//
//     /* Query the LUID */
//     Status = SeQueryAuthenticationIdToken(Token, Luid);
//
//     /* Get rid of the token and return */
//     ObDereferenceObject(Token);
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}