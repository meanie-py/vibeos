use win32k_sys::win32k::*;
#[allow(unused_imports)]
use core::ffi::c_void;

// ── No-op debug macros (the C ERR/WARN compile to nothing under the Wine debug-
//   channel guard). Consume args so side effects still type-check.
macro_rules! ERR  { ($($arg:expr),* $(,)?) => { { $( let _ = $arg; )* } }; }
#[allow(unused_macros)]
macro_rules! WARN { ($($arg:expr),* $(,)?) => { { $( let _ = $arg; )* } }; }

// ── ntdll Rtl* string helper + the `LoadUserApiHook` inline live in crate::internal.
use crate::internal::{LoadUserApiHook, RtlInitUnicodeString, RtlIsThreadWithinLoaderCallout};
// ── RtlCopyMemory (CRT/ntdll glue) lives in crate::winbase.
use crate::winbase::RtlCopyMemory;
// ── NtUserxMDIRedrawFrame is an inline NtUserx* wrapper provided by crate::ntwrapper
//   (no exported symbol) — use the inline, don't extern-link it.
use crate::ntwrapper::NtUserxMDIRedrawFrame;

// ── kernel32 / ntdll imports (WINAPI = stdcall on i386 → extern "system").
extern "system" {
    fn FreeLibrary(hModule: HMODULE) -> BOOL;
    fn RtlEnterCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
    fn RtlLeaveCriticalSection(CriticalSection: PRTL_CRITICAL_SECTION) -> NTSTATUS;
}

// Interlocked{Increment,Decrement} are kernel32 imports on i386 but compiler
// intrinsics (no import symbol) on amd64/arm64 — implement via Rust atomics so the
// cohab link resolves uniformly across all 3 arches.
#[inline]
unsafe fn InterlockedIncrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
        + 1
}
#[inline]
unsafe fn InterlockedDecrement(addend: *mut LONG) -> LONG {
    (*(addend as *const core::sync::atomic::AtomicI32))
        .fetch_sub(1, core::sync::atomic::Ordering::SeqCst)
        - 1
}

// ── user32 cross-TU exports (the Real* hook backends defined in sibling modules /
//   surviving C). All are WINAPI (stdcall on i386) and must match the USERAPIHOOK
//   field typedefs exactly (win32k-sys bindgen `extern "stdcall"`) — fn items only
//   coerce into the matching ABI fn-pointer in const-init, so declare them stdcall.
//   PreWndProc/PostWndProc et al use the in-file DefaultOWP stub instead.
extern "system" {
    fn RealAdjustWindowRectEx(lpRect: LPRECTL, dwStyle: DWORD, bMenu: BOOL, dwExStyle: DWORD) -> BOOL;
    fn RealDefWindowProcA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn RealDefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn RealDrawFrameControl(hdc: HDC, lprc: LPRECTL, uType: UINT, uState: UINT) -> BOOL;
    fn RealGetScrollInfo(hwnd: HWND, fnBar: INT, lpsi: LPSCROLLINFO) -> BOOL;
    fn RealGetSystemMetrics(nIndex: core::ffi::c_int) -> core::ffi::c_int;
    fn RealSetScrollInfo(hwnd: HWND, fnBar: core::ffi::c_int, lpsi: LPCSCROLLINFO, fRedraw: BOOL) -> INT;
    fn RealSystemParametersInfoA(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    fn RealSystemParametersInfoW(uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT) -> BOOL;
    fn GetRealWindowOwner(hWnd: HWND) -> DWORD;
    fn RealUserDrawCaption(hWnd: HWND, hDC: HDC, lpRc: LPCRECTL, uFlags: UINT) -> BOOL;
}

// ---- file-scope preamble (translated) ----
pub static mut gcLoadUserApiHook: DWORD = 0;
pub static mut gcCallUserApiHook: LONG = 0;
pub static mut gfUserApiHook: DWORD = 0;
#[no_mangle]
pub static mut ghmodUserApiHook: HINSTANCE = core::ptr::null_mut();
pub static mut gpfnInitUserApi: USERAPIHOOKPROC = None;
pub static mut gcsUserApiHook: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
pub static mut grgbDwpLiteHookMsg: [BYTE; 128] = [0; 128];
pub static mut grgbWndLiteHookMsg: [BYTE; 128] = [0; 128];
pub static mut grgbDlgLiteHookMsg: [BYTE; 128] = [0; 128];

// `guah` — the global USERAPIHOOK state. Defined here in usrapihk.c (statically
// initialised to the Real* backends); other TUs reference it as an extern (see
// crate::internal). Mirrors the C aggregate initialiser at usrapihk.c:344.
#[no_mangle]
pub static mut guah: USERAPIHOOK = USERAPIHOOK {
    size: core::mem::size_of::<USERAPIHOOK>() as DWORD,
    DefWindowProcA: Some(RealDefWindowProcA),
    DefWindowProcW: Some(RealDefWindowProcW),
    DefWndProcArray: UAHOWP { MsgBitArray: core::ptr::null_mut(), Size: 0 },
    GetScrollInfo: Some(RealGetScrollInfo),
    SetScrollInfo: Some(RealSetScrollInfo),
    EnableScrollBar: Some(NtUserEnableScrollBar),
    AdjustWindowRectEx: Some(RealAdjustWindowRectEx),
    SetWindowRgn: Some(NtUserSetWindowRgn),
    PreWndProc: Some(DefaultOWP),
    PostWndProc: Some(DefaultOWP),
    WndProcArray: UAHOWP { MsgBitArray: core::ptr::null_mut(), Size: 0 },
    PreDefDlgProc: Some(DefaultOWP),
    PostDefDlgProc: Some(DefaultOWP),
    DlgProcArray: UAHOWP { MsgBitArray: core::ptr::null_mut(), Size: 0 },
    GetSystemMetrics: Some(RealGetSystemMetrics),
    SystemParametersInfoA: Some(RealSystemParametersInfoA),
    SystemParametersInfoW: Some(RealSystemParametersInfoW),
    ForceResetUserApiHook: Some(ForceResetUserApiHook),
    DrawFrameControl: Some(RealDrawFrameControl),
    DrawCaption: Some(NtUserDrawCaption),
    MDIRedrawFrame: Some(RealMDIRedrawFrame),
    GetRealWindowOwner: Some(GetRealWindowOwner),
};

unsafe extern "system" fn DefaultOWP(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM, lResult: ULONG_PTR, pData: PDWORD) -> LRESULT {
    DefaultOWP_impl(hWnd, Msg, wParam, lParam, lResult, pData)
}

unsafe fn DefaultOWP_impl(_hWnd: HWND, _Msg: UINT, _wParam: WPARAM, _lParam: LPARAM, _lResult: ULONG_PTR, _pData: PDWORD) -> LRESULT {
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IsMsgOverride(Msg: UINT, puaowpOverride: PUAHOWP) -> BOOL {
    IsMsgOverride_impl(Msg, puaowpOverride)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IsMsgOverride(Msg: UINT, puaowpOverride: PUAHOWP) -> BOOL {
    IsMsgOverride_impl(Msg, puaowpOverride)
}

unsafe fn IsMsgOverride_impl(Msg: UINT, puaowpOverride: PUAHOWP) -> BOOL {
    let nMsg: UINT = Msg / 8;

    if !puaowpOverride.is_null() && nMsg < (*puaowpOverride).Size {
        return ((*(*puaowpOverride).MsgBitArray.add(nMsg as usize)) as u32
            & (1u32 << (Msg & WM_SETFOCUS))) as BOOL;
    }
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn CopyMsgMask(Dest: PUAHOWP, Src: PUAHOWP, hkmsg: *mut core::ffi::c_void, Size: u32) {
    CopyMsgMask_impl(Dest, Src, hkmsg, Size)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CopyMsgMask(Dest: PUAHOWP, Src: PUAHOWP, hkmsg: *mut core::ffi::c_void, Size: u32) {
    CopyMsgMask_impl(Dest, Src, hkmsg, Size)
}

unsafe fn CopyMsgMask_impl(Dest: PUAHOWP, Src: PUAHOWP, hkmsg: *mut core::ffi::c_void, Size: u32) {
    if !Src.is_null() && (*Src).Size > 0 {
        (*Dest).MsgBitArray = hkmsg as *mut BYTE;
        let mut nSize = (*Src).Size;
        if Size < nSize {
            nSize = Size;
        }
        (*Dest).Size = nSize;
        RtlCopyMemory(
            (*Dest).MsgBitArray as *mut c_void,
            (*Src).MsgBitArray as *const c_void,
            nSize as usize,
        );
        return;
    }

    (*Dest).MsgBitArray = core::ptr::null_mut();
    (*Dest).Size = 0;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IsInsideUserApiHook() -> BOOL {
    if !ghmodUserApiHook.is_null() && gfUserApiHook != 0 {
        return 1;
    }
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IsInsideUserApiHook() -> BOOL {
    if !ghmodUserApiHook.is_null() && gfUserApiHook != 0 {
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn BeginIfHookedUserApiHook() -> BOOL {
    InterlockedIncrement(core::ptr::addr_of_mut!(gcCallUserApiHook));
    if IsInsideUserApiHook() != 0 {
        return 1;
    }

    InterlockedDecrement(core::ptr::addr_of_mut!(gcCallUserApiHook));
    0
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn BeginIfHookedUserApiHook() -> BOOL {
    InterlockedIncrement(core::ptr::addr_of_mut!(gcCallUserApiHook));
    if IsInsideUserApiHook() != 0 {
        return 1;
    }

    InterlockedDecrement(core::ptr::addr_of_mut!(gcCallUserApiHook));
    0
}

unsafe extern "system" fn ForceResetUserApiHook(hInstance: HINSTANCE) -> BOOL {
    ForceResetUserApiHook_impl(hInstance)
}

unsafe fn ForceResetUserApiHook_impl(hInstance: HINSTANCE) -> BOOL {
    if ghmodUserApiHook == hInstance && RtlIsThreadWithinLoaderCallout() != 0 {
        ResetUserApiHook(core::ptr::addr_of_mut!(guah));
        gpfnInitUserApi = None;
        return 1;
    }
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn ResetUserApiHook(puah: PUSERAPIHOOK) {
    ResetUserApiHook_impl(puah)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn ResetUserApiHook(puah: PUSERAPIHOOK) {
    ResetUserApiHook_impl(puah)
}

unsafe fn ResetUserApiHook_impl(puah: PUSERAPIHOOK) {
    (*puah).size = core::mem::size_of::<USERAPIHOOK>() as u32;
    (*puah).DefWindowProcA = Some(RealDefWindowProcA);
    (*puah).DefWindowProcW = Some(RealDefWindowProcW);
    (*puah).DefWndProcArray.MsgBitArray = core::ptr::null_mut();
    (*puah).DefWndProcArray.Size = 0;
    (*puah).GetScrollInfo = Some(RealGetScrollInfo);
    (*puah).SetScrollInfo = Some(RealSetScrollInfo);
    (*puah).EnableScrollBar = Some(NtUserEnableScrollBar);
    (*puah).AdjustWindowRectEx = Some(RealAdjustWindowRectEx);
    (*puah).SetWindowRgn = Some(NtUserSetWindowRgn);
    (*puah).PreWndProc = Some(DefaultOWP);
    (*puah).PostWndProc = Some(DefaultOWP);
    (*puah).WndProcArray.MsgBitArray = core::ptr::null_mut();
    (*puah).WndProcArray.Size = 0;
    (*puah).PreDefDlgProc = Some(DefaultOWP);
    (*puah).PostDefDlgProc = Some(DefaultOWP);
    (*puah).DlgProcArray.MsgBitArray = core::ptr::null_mut();
    (*puah).DlgProcArray.Size = 0;
    (*puah).GetSystemMetrics = Some(RealGetSystemMetrics);
    (*puah).SystemParametersInfoA = Some(RealSystemParametersInfoA);
    (*puah).SystemParametersInfoW = Some(RealSystemParametersInfoW);
    (*puah).ForceResetUserApiHook = Some(ForceResetUserApiHook);
    (*puah).DrawFrameControl = Some(RealDrawFrameControl);
    (*puah).DrawCaption = Some(RealUserDrawCaption);
    (*puah).MDIRedrawFrame = Some(RealMDIRedrawFrame);
    (*puah).GetRealWindowOwner = Some(GetRealWindowOwner);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn EndUserApiHook() -> BOOL {
    EndUserApiHook_impl()
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EndUserApiHook() -> BOOL {
    EndUserApiHook_impl()
}

unsafe fn EndUserApiHook_impl() -> BOOL {

    let hModule: HMODULE;
    let pfn: USERAPIHOOKPROC;
    let mut Ret: BOOL = 0;

    if InterlockedDecrement(core::ptr::addr_of_mut!(gcCallUserApiHook)) == 0 {
        if gcLoadUserApiHook == 0 {
            RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));

            pfn = gpfnInitUserApi;
            hModule = ghmodUserApiHook;
            ghmodUserApiHook = core::ptr::null_mut();
            gpfnInitUserApi = None;

            RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));

            if let Some(pfn_fn) = pfn {
                Ret = pfn_fn(uahStop, core::ptr::null_mut());
            }

            if !hModule.is_null() {
                Ret = FreeLibrary(hModule);
            }
        }
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn ClearUserApiHook(mut hInstance: HINSTANCE) -> BOOL {
    let hModule: HMODULE;
    let mut pfn: USERAPIHOOKPROC = None;
    let mut pfn1: USERAPIHOOKPROC;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));
    hModule = ghmodUserApiHook;
    if ghmodUserApiHook == hInstance {
        pfn1 = gpfnInitUserApi;
        gcLoadUserApiHook -= 1;
        if gcLoadUserApiHook == 0 {
            gfUserApiHook = 0;
            ResetUserApiHook(core::ptr::addr_of_mut!(guah));
            if gcCallUserApiHook != 0 {
                hInstance = core::ptr::null_mut();
                pfn1 = None;
                pfn = gpfnInitUserApi;
                gcLoadUserApiHook = 1;
            } else {
                hInstance = hModule;
                ghmodUserApiHook = core::ptr::null_mut();
                gpfnInitUserApi = None;
            }
        }
    } else {
        pfn1 = None;
    }
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));

    if let Some(pfn_fn) = pfn {
        pfn_fn(uahShutdown, core::ptr::null_mut());

        RtlEnterCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));
        pfn1 = gpfnInitUserApi;

        gcLoadUserApiHook -= 1;
        if gcLoadUserApiHook == 0 {
            if gcCallUserApiHook != 0 {
                hInstance = core::ptr::null_mut();
                pfn1 = None;
            } else {
                hInstance = ghmodUserApiHook;
                ghmodUserApiHook = core::ptr::null_mut();
                gpfnInitUserApi = None;
            }
        }
        RtlLeaveCriticalSection(core::ptr::addr_of_mut!(gcsUserApiHook));
    }

    if let Some(pfn1_fn) = pfn1 {
        pfn1_fn(uahStop, core::ptr::null_mut());
    }

    (!hInstance.is_null()) as BOOL
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn InitUserApiHook(hInstance: HINSTANCE, pfn: USERAPIHOOKPROC) -> BOOL {
    let mut uah: USERAPIHOOK = core::mem::zeroed();

    ResetUserApiHook(&mut uah);

    RtlEnterCriticalSection(&raw mut gcsUserApiHook);

    let pfn_fn = pfn.unwrap();

    if pfn_fn(uahLoadInit, &mut uah as *mut USERAPIHOOK) == 0
        || uah.ForceResetUserApiHook != Some(ForceResetUserApiHook)
        || uah.size <= 0
    {
        RtlLeaveCriticalSection(&raw mut gcsUserApiHook);
        return 0;
    }

    if !ghmodUserApiHook.is_null() {
        if ghmodUserApiHook != hInstance {
            RtlLeaveCriticalSection(&raw mut gcsUserApiHook);
            pfn_fn(uahStop, core::ptr::null_mut());
            return 0;
        }
        gcLoadUserApiHook += 1;
    } else {
        ghmodUserApiHook = hInstance;
        RtlCopyMemory(
            &raw mut guah as *mut _ as *mut core::ffi::c_void,
            &uah as *const _ as *const core::ffi::c_void,
            (core::mem::size_of::<USERAPIHOOK>() - core::mem::size_of::<LONG>()) as usize,
        );
        gpfnInitUserApi = pfn;
        gcLoadUserApiHook = 1;
        gfUserApiHook = 1;

        CopyMsgMask(
            core::ptr::addr_of_mut!(guah.DefWndProcArray),
            core::ptr::addr_of_mut!(uah.DefWndProcArray),
            (*(&raw mut grgbDwpLiteHookMsg)).as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&*(&raw const grgbDwpLiteHookMsg)) as u32,
        );

        CopyMsgMask(
            core::ptr::addr_of_mut!(guah.WndProcArray),
            core::ptr::addr_of_mut!(uah.WndProcArray),
            (*(&raw mut grgbWndLiteHookMsg)).as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&*(&raw const grgbWndLiteHookMsg)) as u32,
        );

        CopyMsgMask(
            core::ptr::addr_of_mut!(guah.DlgProcArray),
            core::ptr::addr_of_mut!(uah.DlgProcArray),
            (*(&raw mut grgbDlgLiteHookMsg)).as_mut_ptr() as *mut core::ffi::c_void,
            core::mem::size_of_val(&*(&raw const grgbDlgLiteHookMsg)) as u32,
        );
    }
    RtlLeaveCriticalSection(&raw mut gcsUserApiHook);
    1
}

unsafe extern "system" fn RealMDIRedrawFrame(hWnd: HWND, flags: DWORD) -> BOOL {
    RealMDIRedrawFrame_impl(hWnd, flags)
}

unsafe fn RealMDIRedrawFrame_impl(hWnd: HWND, _flags: DWORD) -> BOOL {
    NtUserxMDIRedrawFrame(hWnd)
}

#[no_mangle]
pub unsafe extern "system" fn MDIRedrawFrame(hWnd: HWND, flags: DWORD) -> BOOL {
    let Hook: BOOL;
    let mut Ret: BOOL = 0;

    LoadUserApiHook();

    Hook = BeginIfHookedUserApiHook();

    /* Bypass SEH and go direct. */
    if Hook == 0 {
        return RealMDIRedrawFrame(hWnd, flags);
    }

    Ret = (guah.MDIRedrawFrame.unwrap())(hWnd, flags);

    EndUserApiHook();

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn RegisterUserApiHook(puah: PUSERAPIHOOKINFO) -> BOOL {
    let mut m_dllname1: _UNICODE_STRING = core::mem::zeroed();
    let mut m_funname1: _UNICODE_STRING = core::mem::zeroed();

    if (*puah).m_size as usize == core::mem::size_of::<USERAPIHOOKINFO>() {
        RtlInitUnicodeString(&mut m_dllname1, (*puah).m_dllname1);
        RtlInitUnicodeString(&mut m_funname1, (*puah).m_funname1);
        return NtUserRegisterUserApiHook(&mut m_dllname1, &mut m_funname1, 0, 0);
    }
    0
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterUserApiHook() -> BOOL {
    NtUserUnregisterUserApiHook()
}
