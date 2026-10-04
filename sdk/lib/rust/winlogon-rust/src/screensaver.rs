//! screensaver.c — screen-saver management: low-level keyboard/mouse hook
//! procs that timestamp activity, the monitor thread that launches the
//! configured SCRNSAVE.EXE on idle, plus InitializeScreenSaver /
//! StartScreenSaver. Link-checked (the saver only runs after real idle on
//! a logged-in desktop, not on the boot smoke path).
//!
//! winlogon.h `#define`s USE_GETLASTINPUTINFO unconditionally (line 29 — in
//! the header, so it never shows up in build.ninja's DEFINES), so the live C
//! path polls GetLastInputInfo and installs NO low-level keyboard/mouse hooks.
//! The `#ifndef USE_GETLASTINPUTINFO` hook procs and the LastActivity
//! timestamp are compiled out of the C and are not carried here. (WLSESSION in
//! winlogon-sys still has the KeyboardHook/MouseHook/LastActivity fields; the
//! struct is private to this all-Rust binary, so they are simply unused.)
//!
//! Translated by rustconv (Opus 4.8), then hand-reconciled: rustconv's
//! hand-rolled `extern "C"` Win32 block was DROPPED (extern "C" is the
//! wrong ABI for stdcall WINAPI on i386) in favour of winlogon-sys's
//! per-arch bindgen decls; per-arch ABI wrappers added for the two
//! HOOKPROC callbacks + the thread proc; Interlocked{Exchange,
//! CompareExchange} → local AtomicI32 helpers.

use winlogon_sys::nt::*;
use crate::preamble::*;

unsafe fn LoadScreenSaverParameters(Timeout: LPDWORD) {
    let mut Enabled: BOOL = 0;

    if SystemParametersInfoW(SPI_GETSCREENSAVETIMEOUT, 0, Timeout as PVOID, 0) == 0 {
        *Timeout = INFINITE;
    } else if SystemParametersInfoW(
        SPI_GETSCREENSAVEACTIVE,
        0,
        core::ptr::addr_of_mut!(Enabled) as PVOID,
        0,
    ) == 0
    {
        *Timeout = INFINITE;
    } else if Enabled == 0 {
        *Timeout = INFINITE;
    } else {
        *Timeout *= 1000;
    }
}

// Thread-proc ABI: stdcall on i386, C on amd64/arm64.
unsafe extern "system" fn ScreenSaverThreadMain(lpParameter: LPVOID) -> DWORD {
    ScreenSaverThreadMainImpl(lpParameter)
}
unsafe fn ScreenSaverThreadMainImpl(lpParameter: LPVOID) -> DWORD {
    let Session: PWLSESSION = lpParameter as PWLSESSION;
    let mut HandleArray: [HANDLE; 3] = [core::ptr::null_mut(); 3];
    let mut lastInputInfo: LASTINPUTINFO = core::mem::zeroed();
    let mut TimeToWait: DWORD;
    let mut Timeout: DWORD = 0;
    let mut ret: DWORD;

    if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
        return 0;
    }

    'cleanup: {
        (*Session).hUserActivity =
            CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        if (*Session).hUserActivity.is_null() {
            break 'cleanup;
        }

        (*Session).hEndOfScreenSaver =
            CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
        if (*Session).hEndOfScreenSaver.is_null() {
            break 'cleanup;
        }

        HandleArray[0] = (*Session).hEndOfScreenSaverThread;
        HandleArray[1] = (*Session).hScreenSaverParametersChanged;
        HandleArray[2] = (*Session).hEndOfScreenSaver;

        LoadScreenSaverParameters(core::ptr::addr_of_mut!(Timeout));

        lastInputInfo.cbSize = core::mem::size_of::<LASTINPUTINFO>() as UINT;

        loop {
            /* See the time of last activity and calculate a timeout */
            if GetLastInputInfo(&mut lastInputInfo) != 0 {
                TimeToWait =
                    Timeout.wrapping_sub(GetTickCount().wrapping_sub(lastInputInfo.dwTime));
            } else {
                WARN!("GetLastInputInfo() failed with error %lu\n", GetLastError());
                TimeToWait = 10; /* Try again in 10 ms */
            }

            if TimeToWait > Timeout {
                /* GetTickCount() got back to 0 */
                TimeToWait = Timeout;
            }

            /* Wait for the timeout, or the end of this thread */
            ret = WaitForMultipleObjects(2, HandleArray.as_ptr(), 0, TimeToWait);
            if ret == WAIT_OBJECT_0 {
                break;
            } else if ret == WAIT_OBJECT_0 + 1 {
                LoadScreenSaverParameters(core::ptr::addr_of_mut!(Timeout));
            }

            /* Check if we didn't had recent activity */
            if GetLastInputInfo(&mut lastInputInfo) == 0 {
                WARN!("GetLastInputInfo() failed with error %lu\n", GetLastError());
                continue;
            }

            if lastInputInfo.dwTime.wrapping_add(Timeout) > GetTickCount() {
                continue;
            }

            /* Run screen saver */
            PostMessageW(
                (*Session).SASWindow,
                WLX_WM_SAS,
                WLX_SAS_TYPE_SCRNSVR_TIMEOUT as WPARAM,
                0,
            );

            /* Wait for the end of this thread or of the screen saver */
            ret = WaitForMultipleObjects(3, HandleArray.as_ptr(), 0, INFINITE);
            if ret == WAIT_OBJECT_0 {
                break;
            } else if ret == WAIT_OBJECT_0 + 1 {
                LoadScreenSaverParameters(core::ptr::addr_of_mut!(Timeout));
            } else if ret == WAIT_OBJECT_0 + 2 {
                SystemParametersInfoW(SPI_SETSCREENSAVERRUNNING, 0, core::ptr::null_mut(), 0);
            }
        }
    }

    if !(*Session).hUserActivity.is_null() {
        CloseHandle((*Session).hUserActivity);
    }

    if !(*Session).hEndOfScreenSaver.is_null() {
        CloseHandle((*Session).hEndOfScreenSaver);
    }

    RevertToSelf();

    CloseHandle((*Session).hEndOfScreenSaverThread);
    CloseHandle((*Session).hScreenSaverParametersChanged);

    0
}

#[no_mangle]
pub unsafe extern "C" fn InitializeScreenSaver(Session: PWLSESSION) -> BOOL {
    let ScreenSaverThread: HANDLE;

    (*Session).hScreenSaverParametersChanged =
        CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
    if (*Session).hScreenSaverParametersChanged.is_null() {
        return 1;
    }

    (*Session).hEndOfScreenSaverThread =
        CreateEventW(core::ptr::null_mut(), 0, 0, core::ptr::null_mut());
    if (*Session).hEndOfScreenSaverThread.is_null() {
        CloseHandle((*Session).hScreenSaverParametersChanged);
        return 1;
    }

    ScreenSaverThread = CreateThread(
        core::ptr::null_mut(),
        0,
        Some(ScreenSaverThreadMain),
        Session as LPVOID,
        0,
        core::ptr::null_mut(),
    );
    if !ScreenSaverThread.is_null() {
        CloseHandle(ScreenSaverThread);
    }

    1
}

#[no_mangle]
pub unsafe extern "C" fn StartScreenSaver(Session: PWLSESSION) {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hCurrentUser: HKEY = core::ptr::null_mut();
    let mut szApplicationName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szCommandLine: [WCHAR; MAX_PATH as usize + 3] = [0; MAX_PATH as usize + 3];
    let mut bufferSize: DWORD =
        (core::mem::size_of_val(&szApplicationName) - core::mem::size_of::<WCHAR>()) as DWORD;
    let mut dwType: DWORD = 0;
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = core::mem::zeroed();
    let mut HandleArray: [HANDLE; 2] = [core::ptr::null_mut(); 2];
    let mut rc: LONG;
    let Status: DWORD;
    let mut ret: BOOL = 0;

    // "Control Panel\Desktop"
    static DESKTOP_KEY: [u16; 22] = [
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16,
        b'l' as u16, b' ' as u16, b'P' as u16, b'a' as u16, b'n' as u16, b'e' as u16,
        b'l' as u16, b'\\' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'k' as u16,
        b't' as u16, b'o' as u16, b'p' as u16, 0,
    ];
    // "SCRNSAVE.EXE"
    static SCRNSAVE_VALUE: [u16; 13] = [
        b'S' as u16, b'C' as u16, b'R' as u16, b'N' as u16, b'S' as u16, b'A' as u16,
        b'V' as u16, b'E' as u16, b'.' as u16, b'E' as u16, b'X' as u16, b'E' as u16, 0,
    ];
    // "%s /s"
    static CMD_FMT: [u16; 6] = [
        b'%' as u16, b's' as u16, b' ' as u16, b'/' as u16, b's' as u16, 0,
    ];
    // "WinSta0\Default"
    static DESKTOP_PATH: [u16; 16] = [
        b'W' as u16, b'i' as u16, b'n' as u16, b'S' as u16, b't' as u16, b'a' as u16,
        b'0' as u16, b'\\' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'a' as u16,
        b'u' as u16, b'l' as u16, b't' as u16, 0,
    ];

    'cleanup: {
        if ImpersonateLoggedOnUser((*Session).UserToken) == 0 {
            break 'cleanup;
        }

        rc = RegOpenCurrentUser(KEY_READ, core::ptr::addr_of_mut!(hCurrentUser));
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        rc = RegOpenKeyExW(
            hCurrentUser,
            DESKTOP_KEY.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hKey),
        );
        if rc != ERROR_SUCCESS as LONG {
            break 'cleanup;
        }

        rc = RegQueryValueExW(
            hKey,
            SCRNSAVE_VALUE.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            szApplicationName.as_mut_ptr() as LPBYTE,
            core::ptr::addr_of_mut!(bufferSize),
        );
        if rc != ERROR_SUCCESS as LONG || dwType != REG_SZ {
            break 'cleanup;
        }

        if bufferSize == 0 {
            break 'cleanup;
        }

        /* Terminate the string */
        szApplicationName[bufferSize as usize / core::mem::size_of::<WCHAR>()] = 0;

        if wcslen(szApplicationName.as_ptr()) == 0 {
            break 'cleanup;
        }

        wsprintfW(
            szCommandLine.as_mut_ptr(),
            CMD_FMT.as_ptr(),
            szApplicationName.as_ptr(),
        );

        StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
        StartupInfo.dwFlags = STARTF_SCREENSAVER;

        /* FIXME: Run the screen saver on the secure screen saver desktop if required */
        StartupInfo.lpDesktop = DESKTOP_PATH.as_ptr() as *mut WCHAR;

        ret = CreateProcessAsUserW(
            (*Session).UserToken,
            szApplicationName.as_ptr(),
            szCommandLine.as_mut_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            IDLE_PRIORITY_CLASS,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(StartupInfo),
            core::ptr::addr_of_mut!(ProcessInformation),
        );
        if ret == 0 {
            break 'cleanup;
        }

        CloseHandle(ProcessInformation.hThread);

        SystemParametersInfoW(SPI_SETSCREENSAVERRUNNING, 1, core::ptr::null_mut(), 0);

        CallNotificationDlls(Session, StartScreenSaverHandler);

        /* Wait the end of the process or some other activity */
        ResetEvent((*Session).hUserActivity);
        HandleArray[0] = ProcessInformation.hProcess;
        HandleArray[1] = (*Session).hUserActivity;
        Status = WaitForMultipleObjects(2, HandleArray.as_ptr(), 0, INFINITE);
        if Status == WAIT_OBJECT_0 + 1 {
            /* Kill the screen saver */
            TerminateProcess(ProcessInformation.hProcess, 0);
        }

        SetEvent((*Session).hEndOfScreenSaver);

        CloseHandle(ProcessInformation.hProcess);

        CallNotificationDlls(Session, StopScreenSaverHandler);
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    if !hCurrentUser.is_null() {
        RegCloseKey(hCurrentUser);
    }

    RevertToSelf();

    if ret == 0 {
        PostMessageW(
            (*Session).SASWindow,
            WLX_WM_SAS,
            WLX_SAS_TYPE_SCRNSVR_ACTIVITY as WPARAM,
            0,
        );
    }
}
