//! shutdown.c — system shutdown dialog + the RPC-invoked
//! StartSystemShutdown / TerminateSystemShutdown entry points (called from
//! the still-C rpcserver.c). NOT on the boot smoke path (it fires only on
//! an actual shutdown request), so it is link-checked here; the countdown
//! dialog code (ShutdownDialogProc / OnTimer / InitiateSystemShutdownThread)
//! runs only when a timed shutdown is requested.
//!
//! Translated by rustconv (Opus 4.8), then hand-reconciled: ntdll_sys
//! import dropped (winlogon-sys is self-contained), per-arch callback ABI
//! wrappers added (DLGPROC/thread-proc are stdcall on i386, C elsewhere),
//! i386 GetWindowLongPtrW/SetWindowLongPtrW shims (the Ptr variants are
//! macros there), _InterlockedCompareExchange8 → atomic helper, DWLP_USER
//! + resource IDs + MAKEINTRESOURCEW provided locally.

use winlogon_sys::nt::*;
use crate::preamble::*;

// --- winlogon resource IDs (base/system/winlogon/resource.h) -------------
const IDD_SYSSHUTDOWN: u16 = 500;
const IDC_SYSSHUTDOWNTIMELEFT: i32 = 502;
const IDC_SYSSHUTDOWNMESSAGE: i32 = 503;
const IDS_TIMEOUTSHORTFORMAT: u32 = 1695;
const IDS_TIMEOUTLONGFORMAT: u32 = 1696;

// DWLP_USER = DWLP_DLGPROC + sizeof(DLGPROC) = sizeof(LRESULT) +
// sizeof(DLGPROC) (winuser.h). Pointer-size dependent → per-arch.
#[cfg(target_arch = "x86")]
const DWLP_USER: i32 = 8;
#[cfg(not(target_arch = "x86"))]
const DWLP_USER: i32 = 16;

// MAKEINTRESOURCEW(i) — low word is the resource id, high word zero.
#[allow(non_snake_case)]
#[inline]
unsafe fn MAKEINTRESOURCEW(i: u16) -> LPWSTR {
    i as usize as LPWSTR
}

// HWND_TOPMOST = (HWND)-1 (winuser.h macro). Int→ptr is not const-eval'able
// in Rust, so a runtime helper rather than a `const`.
#[allow(non_snake_case)]
#[inline]
fn HWND_TOPMOST() -> HWND {
    usize::MAX as HWND
}

// i386 has no separate GetWindowLongPtrW/SetWindowLongPtrW export — they
// are #defines onto the non-Ptr variants (LONG_PTR == LONG there). On
// amd64/arm64 the real exports come from the winlogon-sys glob import.
#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> LONG_PTR {
    GetWindowLongW(hwnd, index) as LONG_PTR
}
#[cfg(target_arch = "x86")]
#[allow(non_snake_case)]
unsafe fn SetWindowLongPtrW(hwnd: HWND, index: i32, val: LONG_PTR) -> LONG_PTR {
    SetWindowLongW(hwnd, index, val as LONG) as LONG_PTR
}

// _InterlockedCompareExchange8(dst, exchange, comparand) → old value at
// dst (MSVC intrinsic; not exported by mingw). Atomic byte CAS.
unsafe fn interlocked_cmpxchg8(dst: *mut i8, exchange: i8, comparand: i8) -> i8 {
    use core::sync::atomic::{AtomicI8, Ordering};
    let a = &*(dst as *const AtomicI8);
    match a.compare_exchange(comparand, exchange, Ordering::SeqCst, Ordering::SeqCst) {
        Ok(v) | Err(v) => v,
    }
}

const SHUTDOWN_TIMER_ID: UINT_PTR = 2000;
const SECONDS_PER_DAY: u32 = 86400;
const SECONDS_PER_DECADE: u32 = 315360000;

#[repr(C)]
pub struct SYS_SHUTDOWN_PARAMS {
    pub pszMessage: *mut u16,
    pub dwTimeout: u32,

    pub hShutdownDesk: HDESK,
    pub DesktopName: [u16; 512],
    pub wpPos: WINDOWPLACEMENT,

    pub bShuttingDown: u8,
    pub bRebootAfterShutdown: u8,
    pub bForceAppsClosed: u8,
    pub dwReason: u32,
}

pub type PSYS_SHUTDOWN_PARAMS = *mut SYS_SHUTDOWN_PARAMS;

// Owned solely by the (now-Rust) shutdown leaf — no other winlogon TU
// references g_ShutdownParams, so this is the single live copy.
#[no_mangle]
pub static mut g_ShutdownParams: SYS_SHUTDOWN_PARAMS = unsafe { core::mem::zeroed() };

unsafe fn DoSystemShutdown(pShutdownParams: PSYS_SHUTDOWN_PARAMS) -> i32 {
    /* If shutdown has been cancelled, bail out now */
    if (*pShutdownParams).bShuttingDown == 0 {
        return 1;
    }

    let Success = ExitWindowsEx(
        (if (*pShutdownParams).bRebootAfterShutdown != 0 {
            EWX_REBOOT
        } else {
            EWX_SHUTDOWN
        }) | (if (*pShutdownParams).bForceAppsClosed != 0 {
            EWX_FORCE
        } else {
            0
        }),
        (*pShutdownParams).dwReason,
    );
    if Success == 0 {
        /* Something went wrong, cancel shutdown */
        (*pShutdownParams).bShuttingDown = 0;
    }

    Success
}

unsafe fn OnTimer(hwndDlg: HWND, pShutdownParams: PSYS_SHUTDOWN_PARAMS) {
    let hInputDesktop: HDESK;
    let bSuccess: i32;
    let mut dwSize: u32 = 0;
    let mut iSeconds: i32;
    let iMinutes: i32;
    let iHours: i32;
    let iDays: i32;
    let mut szFormatBuffer: [u16; 32] = [0; 32];
    let mut szBuffer: [u16; 32] = [0; 32];
    let mut DesktopName: [u16; 512] = [0; 512];

    if (*pShutdownParams).bShuttingDown == 0 {
        /* Shutdown has been cancelled, close the dialog and bail out */
        EndDialog(hwndDlg, IDABORT as INT_PTR);
        return;
    }

    hInputDesktop = OpenInputDesktop(0, 0, GENERIC_ALL);
    if hInputDesktop.is_null() {
        /* No input desktop but we have a dialog: kill it */
        EndDialog(hwndDlg, 0);
        return;
    }
    bSuccess = GetUserObjectInformationW(
        hInputDesktop as HANDLE,
        UOI_NAME as i32,
        DesktopName.as_mut_ptr() as PVOID,
        core::mem::size_of::<[u16; 512]>() as u32,
        &mut dwSize,
    );
    CloseDesktop(hInputDesktop);

    if bSuccess != 0
        && wcscmp(DesktopName.as_ptr(), (*pShutdownParams).DesktopName.as_ptr()) != 0
    {
        /* Save the original dialog position to be restored later */
        (*pShutdownParams).wpPos.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;
        GetWindowPlacement(hwndDlg, &mut (*pShutdownParams).wpPos);

        /* Close the dialog */
        EndDialog(hwndDlg, IDCANCEL as INT_PTR);
        return;
    }

    /* Update the shutdown timeout */
    if (*pShutdownParams).dwTimeout < SECONDS_PER_DAY {
        iSeconds = (*pShutdownParams).dwTimeout as i32;
        iHours = iSeconds / 3600;
        iSeconds -= iHours * 3600;
        iMinutes = iSeconds / 60;
        iSeconds -= iMinutes * 60;

        LoadStringW(
            hAppInstance,
            IDS_TIMEOUTSHORTFORMAT,
            szFormatBuffer.as_mut_ptr(),
            szFormatBuffer.len() as i32,
        );
        swprintf(
            szBuffer.as_mut_ptr(),
            szFormatBuffer.as_ptr(),
            iHours,
            iMinutes,
            iSeconds,
        );
    } else {
        iDays = ((*pShutdownParams).dwTimeout / SECONDS_PER_DAY) as i32;

        LoadStringW(
            hAppInstance,
            IDS_TIMEOUTLONGFORMAT,
            szFormatBuffer.as_mut_ptr(),
            szFormatBuffer.len() as i32,
        );
        swprintf(szBuffer.as_mut_ptr(), szFormatBuffer.as_ptr(), iDays);
    }

    SetDlgItemTextW(hwndDlg, IDC_SYSSHUTDOWNTIMELEFT, szBuffer.as_ptr());

    if (*pShutdownParams).dwTimeout == 0 {
        /* Close the dialog and let the shutdown thread perform the system shutdown */
        EndDialog(hwndDlg, 0);
        return;
    }

    (*pShutdownParams).dwTimeout -= 1;
}

// DLGPROC ABI: stdcall on i386, C on amd64/arm64. winlogon-sys emits the
// callback type accordingly; `extern "system"` is a *distinct* fn-ptr type
// from both even at the same ABI, so match per-arch via thin wrappers.
unsafe extern "system" fn ShutdownDialogProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    ShutdownDialogProcImpl(hwndDlg, uMsg, wParam, lParam)
}

unsafe fn ShutdownDialogProcImpl(
    hwndDlg: HWND,
    uMsg: UINT,
    _wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pShutdownParams: PSYS_SHUTDOWN_PARAMS;

    pShutdownParams = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSYS_SHUTDOWN_PARAMS;

    match uMsg {
        WM_INITDIALOG => {
            pShutdownParams = lParam as PSYS_SHUTDOWN_PARAMS;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pShutdownParams as LONG_PTR);

            /* Display the shutdown message */
            if !(*pShutdownParams).pszMessage.is_null() {
                SetDlgItemTextW(hwndDlg, IDC_SYSSHUTDOWNMESSAGE, (*pShutdownParams).pszMessage);
            }

            /* Remove the Close menu item */
            DeleteMenu(GetSystemMenu(hwndDlg, 0), SC_CLOSE, MF_BYCOMMAND);

            /* Position the window (initial position, or restore from old) */
            if (*pShutdownParams).wpPos.length == core::mem::size_of::<WINDOWPLACEMENT>() as u32 {
                SetWindowPlacement(hwndDlg, &mut (*pShutdownParams).wpPos);
            }

            SetWindowPos(
                hwndDlg,
                HWND_TOPMOST(),
                0,
                0,
                0,
                0,
                SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE,
            );

            /* Initialize the timer */
            PostMessageW(hwndDlg, WM_TIMER, 0, 0);
            SetTimer(hwndDlg, SHUTDOWN_TIMER_ID, 1000, None);
        }

        /* NOTE: Do not handle WM_CLOSE */
        WM_DESTROY => {
            KillTimer(hwndDlg, SHUTDOWN_TIMER_ID);
        }

        WM_TIMER => {
            OnTimer(hwndDlg, pShutdownParams);
        }

        _ => return 0,
    }

    1
}

unsafe extern "system" fn InitiateSystemShutdownThread(lpParameter: LPVOID) -> u32 {
    InitiateSystemShutdownThreadImpl(lpParameter)
}

unsafe fn InitiateSystemShutdownThreadImpl(lpParameter: LPVOID) -> u32 {
    let pShutdownParams: PSYS_SHUTDOWN_PARAMS;
    let mut hInputDesktop: HDESK;
    let mut dwSize: u32 = 0;

    pShutdownParams = lpParameter as PSYS_SHUTDOWN_PARAMS;

    /* Default to initial dialog position */
    (*pShutdownParams).wpPos.length = 0;

    /* Continuously display the shutdown dialog on the current input desktop */
    loop {
        /* Retrieve the current input desktop */
        hInputDesktop = OpenInputDesktop(0, 0, GENERIC_ALL);
        if hInputDesktop.is_null() {
            /* No input desktop on the current WinSta0, just shut down */
            break;
        }

        /* Remember it for checking desktop changes later */
        (*pShutdownParams).hShutdownDesk = hInputDesktop;
        GetUserObjectInformationW(
            (*pShutdownParams).hShutdownDesk as HANDLE,
            UOI_NAME as i32,
            (*pShutdownParams).DesktopName.as_mut_ptr() as PVOID,
            core::mem::size_of::<[u16; 512]>() as u32,
            &mut dwSize,
        );

        /* Assign the desktop to the current thread */
        SetThreadDesktop(hInputDesktop);

        /* Display the shutdown dialog on the current input desktop */
        let res = DialogBoxParamW(
            hAppInstance,
            MAKEINTRESOURCEW(IDD_SYSSHUTDOWN),
            core::ptr::null_mut(),
            Some(ShutdownDialogProc),
            pShutdownParams as LPARAM,
        );

        /* Close the desktop */
        CloseDesktop(hInputDesktop);

        if (res == -1)
            || (res != IDCANCEL as INT_PTR)
            || !((*pShutdownParams).bShuttingDown != 0 && (*pShutdownParams).dwTimeout > 0)
        {
            break;
        }
    }

    /* Reset dialog information */
    (*pShutdownParams).hShutdownDesk = core::ptr::null_mut();
    core::ptr::write_bytes(
        (*pShutdownParams).DesktopName.as_mut_ptr() as *mut u8,
        0,
        core::mem::size_of::<[u16; 512]>(),
    );
    core::ptr::write_bytes(
        &mut (*pShutdownParams).wpPos as *mut WINDOWPLACEMENT as *mut u8,
        0,
        core::mem::size_of::<WINDOWPLACEMENT>(),
    );

    if !(*pShutdownParams).pszMessage.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pShutdownParams).pszMessage as LPVOID);
        (*pShutdownParams).pszMessage = core::ptr::null_mut();
    }

    if (*pShutdownParams).bShuttingDown != 0 {
        /* Perform the system shutdown */
        if DoSystemShutdown(pShutdownParams) != 0 {
            return ERROR_SUCCESS;
        } else {
            return GetLastError();
        }
    }

    (*pShutdownParams).bShuttingDown = 0;
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn TerminateSystemShutdown() -> u32 {
    if interlocked_cmpxchg8(
        core::ptr::addr_of_mut!(g_ShutdownParams.bShuttingDown) as *mut i8,
        0,
        1,
    ) == 0
    {
        return ERROR_NO_SHUTDOWN_IN_PROGRESS;
    }

    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn StartSystemShutdown(
    pMessage: PUNICODE_STRING,
    dwTimeout: u32,
    bForceAppsClosed: u8,
    bRebootAfterShutdown: u8,
    dwReason: u32,
) -> u32 {
    let hThread: HANDLE;

    /* Fail if the timeout is 10 years or more */
    if dwTimeout >= SECONDS_PER_DECADE {
        return ERROR_INVALID_PARAMETER;
    }

    if interlocked_cmpxchg8(
        core::ptr::addr_of_mut!(g_ShutdownParams.bShuttingDown) as *mut i8,
        1,
        0,
    ) == 1
    {
        return ERROR_SHUTDOWN_IN_PROGRESS;
    }

    if dwTimeout != 0
        && !pMessage.is_null()
        && (*pMessage).Length != 0
        && !(*pMessage).Buffer.is_null()
    {
        g_ShutdownParams.pszMessage = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((*pMessage).Length as usize + core::mem::size_of::<u16>()) as _,
        ) as *mut u16;
        if g_ShutdownParams.pszMessage.is_null() {
            g_ShutdownParams.bShuttingDown = 0;
            return GetLastError();
        }

        wcsncpy(
            g_ShutdownParams.pszMessage,
            (*pMessage).Buffer,
            (*pMessage).Length as usize / core::mem::size_of::<u16>(),
        );
    } else {
        g_ShutdownParams.pszMessage = core::ptr::null_mut();
    }

    g_ShutdownParams.dwTimeout = dwTimeout;
    g_ShutdownParams.bForceAppsClosed = bForceAppsClosed;
    g_ShutdownParams.bRebootAfterShutdown = bRebootAfterShutdown;
    g_ShutdownParams.dwReason = dwReason;

    /* If dwTimeout is zero perform an immediate system shutdown,
     * otherwise display the countdown shutdown dialog. */
    if g_ShutdownParams.dwTimeout == 0 {
        if DoSystemShutdown(core::ptr::addr_of_mut!(g_ShutdownParams)) != 0 {
            return ERROR_SUCCESS;
        }
    } else {
        hThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(InitiateSystemShutdownThread),
            core::ptr::addr_of_mut!(g_ShutdownParams) as LPVOID,
            0,
            core::ptr::null_mut(),
        );
        if !hThread.is_null() {
            CloseHandle(hThread);
            return ERROR_SUCCESS;
        }
    }

    if !g_ShutdownParams.pszMessage.is_null() {
        HeapFree(GetProcessHeap(), 0, g_ShutdownParams.pszMessage as LPVOID);
        g_ShutdownParams.pszMessage = core::ptr::null_mut();
    }

    g_ShutdownParams.bShuttingDown = 0;
    GetLastError()
}
