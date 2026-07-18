use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExFreePoolWithTag, PsGetProcessWin32Process, PsGetThreadProcess, PsGetThreadProcessId, RtlAreAllAccessesGranted, SeCaptureSubjectContext, SeLockSubjectContext, SePrivilegeCheck, SeReleaseSubjectContext, SeUnlockSubjectContext};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use ntkrnl_sys::constants::{STATUS_ACCESS_DENIED, STATUS_INVALID_DEVICE_REQUEST, STATUS_INVALID_HANDLE, STATUS_PENDING, STATUS_PRIVILEGE_NOT_HELD};

// GetProcessLuid (userfuncs.h) is plain cdecl (no NTAPI) — nm: `_GetProcessLuid`.
extern "C" {
    fn GetProcessLuid(Thread: PETHREAD, Process: PEPROCESS, Luid: PLUID) -> NTSTATUS;
}

// DestroyTimersForWindow (`@DestroyTimersForWindow@8`) is FASTCALL.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn DestroyTimersForWindow(pti: PTHREADINFO, Window: PWND) -> BOOL;
}

// UserHMGetHandle(obj) = ((obj)->head.h); head at object offset 0 (ntuser.h:230).
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    *(obj as *const HANDLE)
}

// THRDESKHEAD { HEAD h; PTHREADINFO pti; ... } — bindgen flattens h+pti into
// __bindgen_padding_0; pti sits one HEAD (= 2 pointer-widths) into the object.
#[inline(always)]
unsafe fn head_pti<T>(obj: *mut T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}

// tags.h: USERTAG_WINDOWLIST 'lwsU' (C multichar is big-endian → reversed bytes).
const USERTAG_WINDOWLIST: u32 = u32::from_le_bytes(*b"Uswl");

/* Our local copy of shutdown flags */
static mut gdwShutdownFlags: ULONG = 0;

// IntClientShutdown is cdecl (nm: `_IntClientShutdown`), referenced by other TUs.
#[no_mangle]
pub unsafe extern "C" fn IntClientShutdown(
    pWindow: PWND,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let lParams: LPARAM;
    let KillTimers: u8;
    let mut i: i32;
    let mut lResult: LRESULT = MCSR_GOODFORSHUTDOWN as LRESULT;
    let List: *mut HWND;

    KillTimers = if (wParam & MCS_ENDSESSION as WPARAM) != 0 { 1u8 } else { 0u8 };
    lParams = lParam
        & (ENDSESSION_LOGOFF | ENDSESSION_CRITICAL | ENDSESSION_CLOSEAPP) as LPARAM;

    /* First, send end sessions to children */
    List = IntWinListChildren(pWindow);

    if !List.is_null() {
        i = 0;
        'children: while !(*List.offset(i as isize)).is_null() {
            'cont: {
                let WndChild: PWND = UserGetWindowObject(*List.offset(i as isize));

                if WndChild.is_null() {
                    break 'cont;
                }

                if (wParam & MCS_QUERYENDSESSION as WPARAM) != 0 {
                    if co_IntSendMessage(
                        UserHMGetHandle(WndChild) as HWND,
                        WM_QUERYENDSESSION,
                        0,
                        lParams,
                    ) == 0
                    {
                        lResult = MCSR_DONOTSHUTDOWN as LRESULT;
                        break 'children;
                    }
                } else {
                    co_IntSendMessage(
                        UserHMGetHandle(WndChild) as HWND,
                        WM_ENDSESSION,
                        KillTimers as WPARAM,
                        lParams,
                    );
                    if KillTimers != 0 {
                        DestroyTimersForWindow(head_pti(WndChild), WndChild);
                    }
                    lResult = MCSR_SHUTDOWNFINISHED as LRESULT;
                }
            }
            i += 1;
        }
        ExFreePoolWithTag(List as *mut core::ffi::c_void, USERTAG_WINDOWLIST);
        if lResult == MCSR_DONOTSHUTDOWN as LRESULT {
            return lResult;
        }
    }

    /* Send to the caller */
    if (wParam & MCS_QUERYENDSESSION as WPARAM) != 0 {
        if co_IntSendMessage(UserHMGetHandle(pWindow) as HWND, WM_QUERYENDSESSION, 0, lParams) == 0 {
            lResult = MCSR_DONOTSHUTDOWN as LRESULT;
        }
    } else {
        co_IntSendMessage(
            UserHMGetHandle(pWindow) as HWND,
            WM_ENDSESSION,
            KillTimers as WPARAM,
            lParams,
        );
        if KillTimers != 0 {
            DestroyTimersForWindow(head_pti(pWindow), pWindow);
        }
        lResult = MCSR_SHUTDOWNFINISHED as LRESULT;
    }

    lResult
}

// Internal helpers (no external refs) — kept private. UserMode = 1 (KPROCESSOR_MODE).
unsafe fn HasPrivilege(Privilege: PPRIVILEGE_SET) -> BOOLEAN {
    let Result: BOOLEAN;
    let mut SubjectContext: SECURITY_SUBJECT_CONTEXT = core::mem::zeroed();

    /* Capture and lock the security subject context */
    SeCaptureSubjectContext(core::ptr::addr_of_mut!(SubjectContext) as _);
    SeLockSubjectContext(core::ptr::addr_of_mut!(SubjectContext) as _);

    /* Do privilege check */
    Result = SePrivilegeCheck(Privilege as _, core::ptr::addr_of_mut!(SubjectContext) as _, 1i8);

    /* Unlock and release the security subject context and return */
    SeUnlockSubjectContext(core::ptr::addr_of_mut!(SubjectContext) as _);
    SeReleaseSubjectContext(core::ptr::addr_of_mut!(SubjectContext) as _);
    Result
}

unsafe fn NotifyLogon(_hWndSta: HWND, _CallerLuid: PLUID, Flags: ULONG, ShutdownStatus: NTSTATUS) -> BOOL {
    let Notif: ULONG;
    let Param: ULONG;

    /* If no Winlogon notifications are needed, just return */
    if Flags & EWX_NONOTIFY != 0 {
        return FALSE as BOOL;
    }

    /* In case we cancelled the shutdown... */
    if Flags & EWX_SHUTDOWN_CANCELED != 0 {
        Notif = LN_LOGOFF_CANCELED;
        Param = ShutdownStatus as ULONG;
    } else {
        Notif = LN_LOGOFF;
        Param = Flags;
    }

    if !hwndSAS.is_null() {
        UserPostMessage(hwndSAS, WM_LOGONNOTIFY, Notif as WPARAM, Param as LPARAM);
        return TRUE as BOOL;
    }

    FALSE as BOOL
}

// UserInitiateShutdown / UserEndShutdown are cdecl, referenced by other TUs.
#[no_mangle]
pub unsafe extern "C" fn UserInitiateShutdown(
    Thread: PETHREAD,
    pFlags: PULONG,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Flags: ULONG = *pFlags;
    let mut CallerLuid: LUID = core::mem::zeroed();
    let SystemLuid: LUID = LUID {
        LowPart: 0x3e7,
        HighPart: 0,
    };
    let mut ShutdownPrivilege: PRIVILEGE_SET = PRIVILEGE_SET {
        PrivilegeCount: 1,
        Control: PRIVILEGE_SET_ALL_NECESSARY as ULONG,
        Privilege: [LUID_AND_ATTRIBUTES {
            Luid: LUID {
                LowPart: SE_SHUTDOWN_PRIVILEGE as u32,
                HighPart: 0,
            },
            Attributes: 0,
        }],
    };

    let ppi: PPROCESSINFO;

    /* Get the caller's LUID */
    Status = GetProcessLuid(Thread, core::ptr::null_mut(), core::ptr::addr_of_mut!(CallerLuid));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Check if this is the System LUID (RtlEqualLuid inlined) */
    if CallerLuid.LowPart == SystemLuid.LowPart && CallerLuid.HighPart == SystemLuid.HighPart {
        Flags |= EWX_CALLER_SYSTEM;
    } else {
        Flags &= !EWX_CALLER_SYSTEM;
    }

    *pFlags = Flags;

    /* Retrieve the Win32 process info */
    ppi = PsGetProcessWin32Process(PsGetThreadProcess(Thread as _)) as PPROCESSINFO;
    if ppi.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    /* If the caller is not Winlogon, do some security checks */
    if PsGetThreadProcessId(Thread as _) != gpidLogon {
        Flags &= !EWX_CALLER_WINLOGON;

        *pFlags = Flags;

        if (*ppi).prpwinsta.is_null() {
            return STATUS_INVALID_HANDLE;
        }

        if RtlAreAllAccessesGranted((*ppi).amwinsta, WINSTA_EXITWINDOWS) == 0 {
            return STATUS_ACCESS_DENIED;
        }

        if Flags & EWX_SHUTDOWN != 0 {
            if HasPrivilege(core::ptr::addr_of_mut!(ShutdownPrivilege)) == 0 {
                return STATUS_PRIVILEGE_NOT_HELD;
            }
        } else {
            if (*(*ppi).prpwinsta).Flags & WSS_NOIO != 0 {
                return STATUS_INVALID_DEVICE_REQUEST;
            }
        }
    }

    /* If the caller is not Winlogon, possibly notify it to perform the real shutdown */
    if PsGetThreadProcessId(Thread as _) != gpidLogon {
        NotifyLogon(hwndSAS, core::ptr::addr_of_mut!(CallerLuid), Flags, STATUS_SUCCESS);
        return STATUS_PENDING;
    }

    Flags |= EWX_CALLER_WINLOGON;
    *pFlags = Flags;

    /* Save the shutdown flags now */
    gdwShutdownFlags = Flags;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn UserEndShutdown(Thread: PETHREAD, mut ShutdownStatus: NTSTATUS) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut Flags: ULONG;
    let mut CallerLuid: LUID = core::mem::zeroed();

    Status = GetProcessLuid(Thread, core::ptr::null_mut(), &mut CallerLuid);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Copy the global flags because we're going to modify them for our purposes */
    Flags = gdwShutdownFlags;

    if NT_SUCCESS(ShutdownStatus) {
        ShutdownStatus = STATUS_SUCCESS;
    } else {
        Flags |= EWX_SHUTDOWN_CANCELED;
    }

    NotifyLogon(hwndSAS, &mut CallerLuid, Flags, ShutdownStatus);

    /* Always return success */
    STATUS_SUCCESS
}
