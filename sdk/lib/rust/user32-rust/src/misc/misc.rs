use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::internal::RtlUnicodeToMultiByteN;
#[allow(unused_imports)]
use core::ffi::c_void;

// ── NtCurrentTeb + LOWORD/HIWORD + NT_SUCCESS-equivalents live in crate::internal.
use crate::internal::{HIWORD, LOWORD, NtCurrentTeb};
// ── CRT/kernel32/gdi32 glue lives in crate::winbase.
use crate::winbase::{
    GetProcessHeap, MultiByteToWideChar, WideCharToMultiByte, wcslen, NT_SUCCESS,
};
// ── user32-internal globals (gHandleTable/gHandleEntries/g_ppi/gSharedInfo) are
//   owned by misc/dllmain.rs.
use crate::misc::dllmain::{g_ppi, gHandleEntries, gHandleTable, gSharedInfo};

// ── ntdll Rtl* + kernel32 SetLastError + msvcrt strlen + the NtUserx forwarder.
//   WINAPI = stdcall on i386 → extern "system".
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlMultiByteToUnicodeN(
        UnicodeString: PWCH,
        MaxBytesInUnicodeString: ULONG,
        BytesInUnicodeString: PULONG,
        MultiByteString: *const core::ffi::c_char,
        BytesInMultiByteString: ULONG,
    ) -> NTSTATUS;
    fn NtQuerySecurityObject(
        Handle: HANDLE,
        SecurityInformation: SECURITY_INFORMATION,
        SecurityDescriptor: PSECURITY_DESCRIPTOR,
        Length: ULONG,
        LengthNeeded: PULONG,
    ) -> NTSTATUS;
    fn NtSetSecurityObject(
        Handle: HANDLE,
        SecurityInformation: SECURITY_INFORMATION,
        SecurityDescriptor: PSECURITY_DESCRIPTOR,
    ) -> NTSTATUS;
}

extern "C" {
    fn strlen(s: *const core::ffi::c_char) -> usize;
}

// ── user_x.h inline helpers (SharedPtrToUser/DesktopPtrToUser) + the
//   GetWin32ClientInfo / GetThreadDesktopInfo macros, reproduced locally (mirror
//   windows/window.rs + misc/dllmain.rs).
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}
#[no_mangle]
pub unsafe extern "C" fn SharedPtrToUser(Ptr: PVOID) -> PVOID {
    (Ptr as ULONG_PTR - gSharedInfo.ulSharedDelta) as PVOID
}
#[inline]
unsafe fn NtUserxGetDesktopMapping(ptr: PVOID) -> PVOID {
    NtUserCallOneParam(ptr as DWORD_PTR, ONEPARAM_ROUTINE_GETDESKTOPMAPPING as DWORD) as PVOID
}
// user_x.h: static __inline PVOID DesktopPtrToUser(PVOID Ptr)
#[inline]
unsafe fn DesktopPtrToUser(ptr: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(ptr)
}
#[inline]
unsafe fn GetThreadDesktopInfo() -> PDESKTOPINFO {
    let ti = GetW32ThreadInfo();
    if !ti.is_null() {
        (*GetWin32ClientInfo()).pDeskInfo
    } else {
        core::ptr::null_mut()
    }
}

// ---- file-scope preamble (translated) ----
pub static g_ObjectHeapTypeShared: [BOOL; TYPE_CTYPES as usize] =
[
    0, /* TYPE_FREE (not used) */
    0, /* TYPE_WINDOW */
    0, /* TYPE_MENU */
    1, /* TYPE_CURSOR */
    1, /* TYPE_SETWINDOWPOS */
    0, /* TYPE_HOOK */
    1, /* TYPE_CLIPDATA */
    0, /* TYPE_CALLPROC */
    1, /* TYPE_ACCELTABLE */
    0, /* TYPE_DDEACCESS */
    0, /* TYPE_DDECONV */
    0, /* TYPE_DDEXACT */
    1, /* TYPE_MONITOR */
    1, /* TYPE_KBDLAYOUT */
    1, /* TYPE_KBDFILE */
    1, /* TYPE_WINEVENTHOOK */
    1, /* TYPE_TIMER */
    0, /* TYPE_INPUTCONTEXT */
    0, /* TYPE_HIDDATA */
    0, /* TYPE_DEVICEINFO */
    0, /* TYPE_TOUCHINPUTINFO */
    0, /* TYPE_GESTUREINFOOBJ */
];

#[no_mangle]
pub unsafe extern "system" fn UserSetLastError(dwErrCode: DWORD) {
    (*NtCurrentTeb()).LastErrorValue = dwErrCode;
}

#[no_mangle]
pub unsafe extern "system" fn UserSetLastNTError(Status: NTSTATUS) {
    UserSetLastError(RtlNtStatusToDosError(Status));
}

use crate::windows::message::GetW32ThreadInfo;

#[no_mangle]
pub unsafe extern "system" fn GetUserObjectSecurity(
    hObject: HANDLE,
    pSecurityInfo: PSECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
    dwLength: DWORD,
    pdwLengthNeeded: PDWORD,
) -> BOOL {
    let Status: NTSTATUS = NtQuerySecurityObject(
        hObject,
        *pSecurityInfo,
        pSecurityDescriptor,
        dwLength,
        pdwLengthNeeded,
    );
    if !NT_SUCCESS(Status) {
        UserSetLastNTError(Status);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SetUserObjectSecurity(
    hObject: HANDLE,
    pSecurityInfo: PSECURITY_INFORMATION,
    pSecurityDescriptor: PSECURITY_DESCRIPTOR,
) -> BOOL {
    let Status = NtSetSecurityObject(hObject, *pSecurityInfo, pSecurityDescriptor);
    if !NT_SUCCESS(Status) {
        UserSetLastNTError(Status);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn IsGUIThread(bConvert: BOOL) -> BOOL {
    let ti = (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO;
    if ti.is_null() {
        if bConvert != 0 {
            NtUserGetThreadState(THREADSTATE_GETTHREADINFO as _);
            if !((*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO).is_null() {
                return TRUE as BOOL;
            } else {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            }
        }
        return FALSE as BOOL;
    } else {
        return TRUE as BOOL;
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn TestWindowProcess(Wnd: PWND) -> BOOL {
    TestWindowProcess_impl(Wnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn TestWindowProcess(Wnd: PWND) -> BOOL {
    TestWindowProcess_impl(Wnd)
}

unsafe fn TestWindowProcess_impl(Wnd: PWND) -> BOOL {
    // bindgen flattened THRDESKHEAD: head.pti via _THROBJHEAD, head.h via _HEAD.
    if (*(Wnd as *const _THROBJHEAD)).pti == (*NtCurrentTeb()).Win32ThreadInfo as PTHREADINFO {
        1
    } else {
        (NtUserQueryWindow((*(Wnd as *const _HEAD)).h as HWND, QUERY_WINDOW_UNIQUE_PROCESS_ID)
            == (*NtCurrentTeb()).ClientId.UniqueProcess as DWORD_PTR) as BOOL
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn TestState(pWnd: PWND, Flag: UINT) -> BOOL {
    TestState_impl(pWnd, Flag)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn TestState(pWnd: PWND, Flag: UINT) -> BOOL {
    TestState_impl(pWnd, Flag)
}

unsafe fn TestState_impl(pWnd: PWND, Flag: UINT) -> BOOL {
    let bit: UINT = 1 << LOWORD(Flag);
    match HIWORD(Flag) {
        0 => ((*pWnd).state & bit) as BOOL,
        1 => ((*pWnd).state2 & bit) as BOOL,
        2 => ((*pWnd).ExStyle2 & bit) as BOOL,
        _ => 0,
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn GetUser32Handle(handle: HANDLE) -> PUSER_HANDLE_ENTRY {
    GetUser32Handle_impl(handle)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn GetUser32Handle(handle: HANDLE) -> PUSER_HANDLE_ENTRY {
    GetUser32Handle_impl(handle)
}

unsafe fn GetUser32Handle_impl(handle: HANDLE) -> PUSER_HANDLE_ENTRY {
    // `scratch` is only ever used on the WoW64 path (see GetUser32HandleInto).
    // Callers that must survive a nested GetUser32Handle on the same thread pass
    // their own buffer via GetUser32HandleInto instead.
    static mut SCRATCH: USER_HANDLE_ENTRY = unsafe { core::mem::zeroed() };
    GetUser32HandleInto(handle, core::ptr::addr_of_mut!(SCRATCH))
}

/// `GetUser32Handle`, but the WoW64 path normalizes the 64-bit table entry into
/// the caller-owned `scratch` (a stack local) and returns that. On a native build
/// `scratch` is unused and the returned pointer aims straight into the shared
/// table, exactly as before.
///
/// The buffer is caller-owned because the shared USER handle table is
/// 64-bit-laid-out under WoW64 and so cannot be indexed in place — and a single
/// shared static would be clobbered by a concurrent GUI thread.
unsafe fn GetUser32HandleInto(handle: HANDLE, scratch: *mut USER_HANDLE_ENTRY) -> PUSER_HANDLE_ENTRY {
    if handle.is_null() {
        return core::ptr::null_mut();
    }

    let Index: i32 = ((((handle as UINT_PTR) & 0xffff) - FIRST_USER_HANDLE as UINT_PTR) >> 1) as i32;

    let entry: PUSER_HANDLE_ENTRY = if crate::wow64::IsWow64() {
        if !crate::wow64::read_handle_entry(Index, scratch) {
            return core::ptr::null_mut();
        }
        scratch
    } else {
        if Index < 0 || Index >= (*gHandleTable).nb_handles {
            return core::ptr::null_mut();
        }
        gHandleEntries.offset(Index as isize)
    };

    if (*entry).type_ == 0 || (*entry).ptr.is_null() {
        return core::ptr::null_mut();
    }

    let generation: u16 = ((handle as UINT_PTR) >> 16) as u16;

    if generation == (*entry).generation || generation == 0 || generation == 0xffff {
        return entry;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateHandle(handle: HANDLE, uType: UINT) -> PVOID {
    ValidateHandle_impl(handle, uType)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateHandle(handle: HANDLE, uType: UINT) -> PVOID {
    ValidateHandle_impl(handle, uType)
}

unsafe fn ValidateHandle_impl(handle: HANDLE, mut uType: UINT) -> PVOID {
    let ret: PVOID;
    let pEntry: PUSER_HANDLE_ENTRY;

    let mut scratch: USER_HANDLE_ENTRY = core::mem::zeroed();
    pEntry = GetUser32HandleInto(handle, core::ptr::addr_of_mut!(scratch));

    if !pEntry.is_null() && uType == 0 {
        uType = (*pEntry).type_ as UINT;
    }

    if pEntry.is_null()
        || (*pEntry).type_ as UINT != uType
        || (*pEntry).ptr.is_null()
        || ((*pEntry).flags as u32 & HANDLEENTRY_DESTROY as u32) != 0
        || ((*pEntry).flags as u32 & HANDLEENTRY_INDESTROY as u32) != 0
    {
        match uType {
            TYPE_WINDOW => SetLastError(ERROR_INVALID_WINDOW_HANDLE),
            TYPE_MENU => SetLastError(ERROR_INVALID_MENU_HANDLE),
            TYPE_CURSOR => SetLastError(ERROR_INVALID_CURSOR_HANDLE),
            TYPE_SETWINDOWPOS => SetLastError(ERROR_INVALID_DWP_HANDLE),
            TYPE_HOOK => SetLastError(ERROR_INVALID_HOOK_HANDLE),
            TYPE_ACCELTABLE => SetLastError(ERROR_INVALID_ACCEL_HANDLE),
            _ => SetLastError(ERROR_INVALID_HANDLE),
        }
        return core::ptr::null_mut();
    }

    if g_ObjectHeapTypeShared[uType as usize] != 0 {
        ret = SharedPtrToUser((*pEntry).ptr);
    } else if uType == TYPE_WINDOW {
        // The one that matters under WoW64: the caller dereferences this PWND.
        ret = crate::wow64::desktop_wnd_to_user((*pEntry).ptr) as PVOID;
    } else if uType == TYPE_CALLPROC {
        // CallWindowProcA/W CALL the pfnClientPrevious read out of this.
        ret = crate::wow64::desktop_cpd_to_user((*pEntry).ptr) as PVOID;
    } else {
        ret = DesktopPtrToUser((*pEntry).ptr);
    }

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
unsafe extern "fastcall" fn ValidateHandleNoErr(handle: HANDLE, uType: UINT) -> PVOID {
    ValidateHandleNoErr_impl(handle, uType)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
unsafe extern "C" fn ValidateHandleNoErr(handle: HANDLE, uType: UINT) -> PVOID {
    ValidateHandleNoErr_impl(handle, uType)
}

unsafe fn ValidateHandleNoErr_impl(handle: HANDLE, mut uType: UINT) -> PVOID {
    let ret: PVOID;
    let pEntry: PUSER_HANDLE_ENTRY;

    let mut scratch: USER_HANDLE_ENTRY = core::mem::zeroed();
    pEntry = GetUser32HandleInto(handle, core::ptr::addr_of_mut!(scratch));

    if !pEntry.is_null() && uType == 0 {
        uType = (*pEntry).type_ as UINT;
    }

    if pEntry.is_null() || (*pEntry).type_ as UINT != uType || (*pEntry).ptr.is_null() {
        return core::ptr::null_mut();
    }

    if g_ObjectHeapTypeShared[uType as usize] != 0 {
        ret = SharedPtrToUser((*pEntry).ptr);
    } else if uType == TYPE_WINDOW {
        // The one that matters under WoW64: the caller dereferences this PWND.
        ret = crate::wow64::desktop_wnd_to_user((*pEntry).ptr) as PVOID;
    } else if uType == TYPE_CALLPROC {
        // CallWindowProcA/W CALL the pfnClientPrevious read out of this.
        ret = crate::wow64::desktop_cpd_to_user((*pEntry).ptr) as PVOID;
    } else {
        ret = DesktopPtrToUser((*pEntry).ptr);
    }

    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateCallProc(hCallProc: HANDLE) -> PCALLPROCDATA {
    let pEntry: PUSER_HANDLE_ENTRY;

    let CallProc = ValidateHandle(hCallProc, TYPE_CALLPROC as UINT) as PCALLPROCDATA;

    let mut scratch: USER_HANDLE_ENTRY = core::mem::zeroed();
    pEntry = GetUser32HandleInto(hCallProc, core::ptr::addr_of_mut!(scratch));

    if CallProc != core::ptr::null_mut() && (*pEntry).__bindgen_anon_1.ppi == g_ppi {
        return CallProc;
    }

    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateCallProc(hCallProc: HANDLE) -> PCALLPROCDATA {
    let pEntry: PUSER_HANDLE_ENTRY;

    let CallProc = ValidateHandle(hCallProc, TYPE_CALLPROC as UINT) as PCALLPROCDATA;

    let mut scratch: USER_HANDLE_ENTRY = core::mem::zeroed();
    pEntry = GetUser32HandleInto(hCallProc, core::ptr::addr_of_mut!(scratch));

    if CallProc != core::ptr::null_mut() && (*pEntry).__bindgen_anon_1.ppi == g_ppi {
        return CallProc;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateHwnd(hwnd: HWND) -> PWND {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    if !hwnd.is_null() && hwnd == (*ClientInfo).CallbackWnd.hWnd {
        return (*ClientInfo).CallbackWnd.pWnd;
    }

    ValidateHandle(hwnd as HANDLE, TYPE_WINDOW) as PWND
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateHwnd(hwnd: HWND) -> PWND {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    if !hwnd.is_null() && hwnd == (*ClientInfo).CallbackWnd.hWnd {
        return (*ClientInfo).CallbackWnd.pWnd;
    }

    ValidateHandle(hwnd as HANDLE, TYPE_WINDOW) as PWND
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ValidateHwndNoErr(hwnd: HWND) -> PWND {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    if hwnd == (*ClientInfo).CallbackWnd.hWnd {
        return (*ClientInfo).CallbackWnd.pWnd;
    }

    let Wnd = ValidateHandleNoErr(hwnd as HANDLE, TYPE_WINDOW) as PWND;
    if !Wnd.is_null() {
        return Wnd;
    }

    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ValidateHwndNoErr(hwnd: HWND) -> PWND {
    let ClientInfo: PCLIENTINFO = GetWin32ClientInfo();

    if hwnd == (*ClientInfo).CallbackWnd.hWnd {
        return (*ClientInfo).CallbackWnd.pWnd;
    }

    let Wnd = ValidateHandleNoErr(hwnd as HANDLE, TYPE_WINDOW) as PWND;
    if !Wnd.is_null() {
        return Wnd;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GetThreadDesktopWnd() -> PWND {
    let mut Wnd: PWND = crate::wow64::desktop_info_spwnd(GetThreadDesktopInfo());
    if Wnd != core::ptr::null_mut() {
        Wnd = crate::wow64::desktop_wnd_to_user(Wnd as *mut core::ffi::c_void);
    }
    Wnd
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GetThreadDesktopWnd() -> PWND {
    let mut Wnd: PWND = crate::wow64::desktop_info_spwnd(GetThreadDesktopInfo());
    if Wnd != core::ptr::null_mut() {
        Wnd = crate::wow64::desktop_wnd_to_user(Wnd as *mut core::ffi::c_void);
    }
    Wnd
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ValidateHwndOrDesk(hwnd: HWND) -> PWND {
    if hwnd == core::ptr::null_mut() {
        return GetThreadDesktopWnd();
    }

    ValidateHwnd(hwnd)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ValidateHwndOrDesk(hwnd: HWND) -> PWND {
    if hwnd == core::ptr::null_mut() {
        return GetThreadDesktopWnd();
    }

    ValidateHwnd(hwnd)
}

#[no_mangle]
pub unsafe extern "system" fn WCSToMBEx(CodePage: WORD, UnicodeString: LPWSTR, mut UnicodeSize: LONG, MBString: *mut LPSTR, mut MBSize: LONG, Allocate: BOOL) -> DWORD {
    let mut Size: DWORD = 0;
    if UnicodeSize == -1 {
        UnicodeSize = (wcslen(UnicodeString) + 1) as LONG;
    }
    if MBSize == -1 {
        if Allocate == 0 {
            return 0;
        }
        MBSize = UnicodeSize * 2;
    }
    if Allocate != 0 {
        let SafeString = RtlAllocateHeap(GetProcessHeap(), 0, MBSize as SIZE_T) as LPSTR;
        if SafeString.is_null() {
            return 0;
        }
        *MBString = SafeString;
    }
    if CodePage == 0 {
        RtlUnicodeToMultiByteN(*MBString, MBSize as u32, &mut Size, UnicodeString, UnicodeSize as u32);
    } else {
        WideCharToMultiByte(CodePage as u32, 0, UnicodeString, UnicodeSize, *MBString, MBSize, core::ptr::null_mut(), core::ptr::null_mut());
    }
    UnicodeSize as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn MBToWCSEx(
    CodePage: WORD,
    MBString: LPSTR,
    mut MBSize: LONG,
    UnicodeString: *mut LPWSTR,
    mut UnicodeSize: LONG,
    Allocate: BOOL,
) -> DWORD {
    let mut Size: DWORD = 0;
    if MBSize == -1 {
        MBSize = (strlen(MBString as *const _) + 1) as LONG;
    }
    if UnicodeSize == -1 {
        if Allocate == 0 {
            return 0;
        }
        UnicodeSize = MBSize;
    }
    if Allocate != 0 {
        let SafeString = RtlAllocateHeap(GetProcessHeap(), 0, UnicodeSize as _) as LPWSTR;
        if SafeString.is_null() {
            return 0;
        }
        *UnicodeString = SafeString;
    }
    UnicodeSize *= core::mem::size_of::<WCHAR>() as LONG;
    if CodePage == 0 {
        RtlMultiByteToUnicodeN(
            *UnicodeString,
            UnicodeSize as _,
            core::ptr::addr_of_mut!(Size) as *mut _,
            MBString as *mut _,
            MBSize as _,
        );
    } else {
        Size = MultiByteToWideChar(
            CodePage as _,
            0,
            MBString,
            MBSize,
            *UnicodeString,
            UnicodeSize,
        ) as DWORD;
    }
    Size
}
