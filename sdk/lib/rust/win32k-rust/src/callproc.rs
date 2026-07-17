use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadWin32Thread;
use ntkrnl_sys::interlocked::InterlockedExchangePointer;
use core::ffi::c_void;

// UserHMGetHandle(obj) = obj->head.h (ntuser.h:230 macro, no symbol). head is the
// first field and h its first field, so the handle is at object offset 0 — same
// reimplementation as accelerator.rs.
#[inline(always)]
unsafe fn UserHMGetHandle(obj: PCALLPROCDATA) -> HANDLE {
    *(obj as *const HANDLE)
}

// ---- file-scope preamble (translated) ----
// DBG_DEFAULT_CHANNEL(UserClass) - debug channel macro, no Rust declaration needed

// C returns WNDPROC, but WNDPROC is Option<extern fn> (not castable from an
// integer) and every caller immediately wants the ULONG_PTR handle value — this
// fn is file-local, so return ULONG_PTR directly and skip the round-trip casts.
unsafe fn GetCallProcHandle(CallProc: PCALLPROCDATA) -> ULONG_PTR {
    UserHMGetHandle(CallProc as _) as ULONG_PTR | 0xFFFF0000
}

// class.h declares these cdecl (no FASTCALL/NTAPI); object.c registers
// DestroyCallProc as the TYPE_CALLPROC destructor and class.c calls
// CreateCallProc, so both export their C symbol.
#[no_mangle]
pub unsafe extern "C" fn DestroyCallProc(Object: PVOID) -> BOOLEAN {
    UserDeleteObject(UserHMGetHandle(Object as PCALLPROCDATA), TYPE_CALLPROC);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn CreateCallProc(
    Desktop: PDESKTOP,
    WndProc: WNDPROC,
    Unicode: BOOL,
    pi: PPROCESSINFO,
) -> PCALLPROCDATA {
    let mut Handle: HANDLE = core::ptr::null_mut();

    let NewCallProc = UserCreateObject(
        gHandleTable,
        Desktop as *mut _,
        (*pi).ptiList as *mut _,
        &mut Handle,
        TYPE_CALLPROC,
        core::mem::size_of::<CALLPROCDATA>() as u32,
    ) as PCALLPROCDATA;

    if !NewCallProc.is_null() {
        (*NewCallProc).pfnClientPrevious = WndProc;
        (*NewCallProc).wType |= if Unicode != 0 { UserGetCPDA2U } else { UserGetCPDU2A };
        (*NewCallProc).spcpdNext = core::ptr::null_mut();

        UserDereferenceObject(NewCallProc as *mut _);
    }

    NewCallProc
}

// class.h declares this cdecl (no NTAPI) — extern "C", not "system" (which would
// mis-decorate as _UserGetCallProcInfo@8 on i386).
#[no_mangle]
pub unsafe extern "C" fn UserGetCallProcInfo(
    hCallProc: HANDLE,
    wpInfo: PWNDPROC_INFO,
) -> BOOL {
    let CallProc: PCALLPROCDATA = UserGetObject(
        gHandleTable,
        hCallProc,
        TYPE_CALLPROC,
    ) as PCALLPROCDATA;
    if CallProc.is_null() {
        return FALSE as BOOL;
    }

    (*wpInfo).WindowProc = (*CallProc).pfnClientPrevious;
    (*wpInfo).IsUnicode = ((((*CallProc).wType & UserGetCPDA2U) != 0) as u32) as BOOL;

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe extern "fastcall" fn UserSearchForCallProc(
    mut pcpd: PCALLPROCDATA,
    WndProc: WNDPROC,
    Type: GETCPD,
) -> PCALLPROCDATA {
    while !pcpd.is_null()
        && ((*pcpd).pfnClientPrevious != WndProc || (*pcpd).wType != Type)
    {
        pcpd = (*pcpd).spcpdNext;
    }
    pcpd
}

#[cfg(not(target_arch = "x86"))]
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe extern "C" fn UserSearchForCallProc(
    mut pcpd: PCALLPROCDATA,
    WndProc: WNDPROC,
    Type: GETCPD,
) -> PCALLPROCDATA {
    while !pcpd.is_null()
        && ((*pcpd).pfnClientPrevious != WndProc || (*pcpd).wType != Type)
    {
        pcpd = (*pcpd).spcpdNext;
    }
    pcpd
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserGetCPD(
    pvClsWnd: PVOID,
    Flags: GETCPD,
    ProcIn: ULONG_PTR,
) -> ULONG_PTR {
    UserGetCPD_impl(pvClsWnd, Flags, ProcIn)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserGetCPD(
    pvClsWnd: PVOID,
    Flags: GETCPD,
    ProcIn: ULONG_PTR,
) -> ULONG_PTR {
    UserGetCPD_impl(pvClsWnd, Flags, ProcIn)
}

unsafe fn UserGetCPD_impl(
    pvClsWnd: PVOID,
    Flags: GETCPD,
    ProcIn: ULONG_PTR,
) -> ULONG_PTR {
    let pCls: PCLS;
    let pWnd: PWND;
    let pDesk: PDESKTOP;
    let mut CallProc: PCALLPROCDATA = core::ptr::null_mut();
    let pti: PTHREADINFO;

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;

    if (Flags & (UserGetCPDWindow | UserGetCPDDialog)) != 0
        || (Flags & UserGetCPDWndtoCls) != 0
    {
        pWnd = pvClsWnd as PWND;
        pCls = (*pWnd).pcls;
    } else {
        pCls = pvClsWnd as PCLS;
    }

    if !(*pCls).spcpdFirst.is_null() {
        CallProc = UserSearchForCallProc(
            (*pCls).spcpdFirst,
            core::mem::transmute::<ULONG_PTR, WNDPROC>(ProcIn),
            Flags,
        );
    }

    if CallProc.is_null() {
        if (*pCls).rpdeskParent.is_null() {
            pDesk = (*pti).rpdesk;
        } else {
            pDesk = (*pCls).rpdeskParent;
        }
        CallProc = CreateCallProc(
            pDesk,
            core::mem::transmute::<ULONG_PTR, WNDPROC>(ProcIn),
            ((Flags & UserGetCPDA2U) != 0) as BOOL,
            (*pti).ppi,
        );
        if !CallProc.is_null() {
            (*CallProc).spcpdNext = (*pCls).spcpdFirst;
            InterlockedExchangePointer(
                core::ptr::addr_of_mut!((*pCls).spcpdFirst) as *mut PVOID,
                CallProc as PVOID,
            );
            (*CallProc).wType = Flags;
        }
    }

    if !CallProc.is_null() {
        GetCallProcHandle(CallProc) as ULONG_PTR
    } else {
        core::ptr::null_mut::<core::ffi::c_void>() as ULONG_PTR
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtUserGetCPD(
    hWnd: HWND,
    Flags: GETCPD,
    ProcIn: ULONG_PTR,
) -> ULONG_PTR {
    let Wnd: PWND;
    let mut Result: ULONG_PTR = 0;

    UserEnterExclusive();
    'cleanup: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'cleanup;
        }

        if (Flags & !(UserGetCPDU2A | UserGetCPDA2U)) != UserGetCPDClass {
            Result = UserGetCPD(Wnd as PVOID, Flags, ProcIn);
        }
    }

    UserLeave();
    Result
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtUserGetCPD(
    hWnd: HWND,
    Flags: GETCPD,
    ProcIn: ULONG_PTR,
) -> ULONG_PTR {
    let Wnd: PWND;
    let mut Result: ULONG_PTR = 0;

    UserEnterExclusive();
    'cleanup: {
        Wnd = UserGetWindowObject(hWnd);
        if Wnd.is_null() {
            break 'cleanup;
        }

        if (Flags & !(UserGetCPDU2A | UserGetCPDA2U)) != UserGetCPDClass {
            Result = UserGetCPD(Wnd as PVOID, Flags, ProcIn);
        }
    }

    UserLeave();
    Result
}
