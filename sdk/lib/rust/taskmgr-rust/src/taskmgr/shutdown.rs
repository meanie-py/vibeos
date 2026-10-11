use crate::taskmgr::taskmgr::ShowWin32Error;
use crate::prelude::*;
use core::ffi::c_void;

const NT_INITIATE_POWERACTION_IMPLEMENTED: bool = false;

unsafe extern "C" fn EnablePrivilege(
    mut lpszPrivilegeName: LPCWSTR,
    mut bEnablePrivilege: BOOL,
) -> BOOL {
    let mut Success: BOOL = 0;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

    'quit: {
        Success = OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES as DWORD,
            &raw mut hToken,
        );
        if Success == 0 {
            return Success;
        }

        Success = LookupPrivilegeValueW(
            core::ptr::null::<WCHAR>(),
            lpszPrivilegeName,
            &raw mut tp.Privileges[0 as usize].Luid,
        );
        if Success == 0 {
            break 'quit;
        }

        tp.PrivilegeCount = 1 as DWORD;
        tp.Privileges[0 as usize].Attributes = (if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        }) as DWORD;

        Success = AdjustTokenPrivileges(
            hToken,
            FALSE as BOOL,
            &raw mut tp,
            0 as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
    }

    CloseHandle(hToken);
    return Success;
}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_StandBy() {
    let mut Status: NTSTATUS = 0;

    if EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL) == 0 {
        ShowWin32Error(GetLastError());
        return;
    }

    // NT_INITIATE_POWERACTION_IMPLEMENTED is not defined, so the #else branch is active.
    if false {
        Status = NtInitiatePowerAction(
            PowerActionSleep,
            PowerSystemSleeping1,
            0,
            (FALSE) as BOOLEAN,
        );
    } else {
        Status = NtSetSystemPowerState(
            PowerActionSleep,
            PowerSystemSleeping1,
            0,
        );
    }

    if !NT_SUCCESS!(Status) {
        ShowWin32Error(RtlNtStatusToDosError(Status) as DWORD);
    }

    EnablePrivilege(SE_SHUTDOWN_NAME, (FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_Hibernate() {
    let Status: NTSTATUS;

    if EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL) == 0 {
        ShowWin32Error(GetLastError());
        return;
    }

    if NT_INITIATE_POWERACTION_IMPLEMENTED {
        Status = NtInitiatePowerAction(
            PowerActionHibernate,
            PowerSystemHibernate,
            0,
            (FALSE) as BOOLEAN,
        );
    } else {
        Status = NtSetSystemPowerState(
            PowerActionHibernate,
            PowerSystemHibernate,
            0,
        );
    }

    if !NT_SUCCESS!(Status) {
        ShowWin32Error(RtlNtStatusToDosError(Status));
    }

    EnablePrivilege(SE_SHUTDOWN_NAME, (FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_PowerOff() {
    /* Trick: on Windows, pressing the CTRL key forces shutdown via NT API */
    let mut ForceShutdown: BOOL = ((GetKeyState(VK_CONTROL as ::core::ffi::c_int) as i32 & 0x8000 as i32) != 0) as BOOL;

    if EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL) == 0 {
        ShowWin32Error(GetLastError());
        return;
    }

    if ForceShutdown != 0 {
        let mut Status: NTSTATUS = NtShutdownSystem(ShutdownPowerOff);
        if !NT_SUCCESS!(Status) {
            ShowWin32Error(RtlNtStatusToDosError(Status));
        }
    } else {
        /* The choice of EWX_SHUTDOWN or EWX_POWEROFF may be done with NtPowerInformation */
        if ExitWindowsEx(EWX_POWEROFF, SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) == 0 {
            ShowWin32Error(GetLastError());
        }
    }

    EnablePrivilege(SE_SHUTDOWN_NAME, (FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_Reboot() {
    /* Trick: on Windows, pressing the CTRL key forces reboot via NT API */
    let ForceReboot: BOOL = (((GetKeyState(VK_CONTROL as i32) as i32) & (0x8000)) != 0) as BOOL;

    if EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL) == 0 {
        ShowWin32Error(GetLastError());
        return;
    }

    if ForceReboot != 0 {
        let Status: NTSTATUS = NtShutdownSystem(ShutdownReboot);
        if !NT_SUCCESS!(Status) {
            ShowWin32Error(RtlNtStatusToDosError(Status) as DWORD);
        }
    } else {
        if ExitWindowsEx(
            EWX_REBOOT as UINT,
            (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as DWORD,
        ) == 0
        {
            ShowWin32Error(GetLastError());
        }
    }

    EnablePrivilege(SE_SHUTDOWN_NAME, (FALSE) as BOOL);
}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_LogOffUser() {
    if ExitWindowsEx(
        EWX_LOGOFF as UINT,
        (SHTDN_REASON_MAJOR_OTHER | SHTDN_REASON_MINOR_OTHER) as DWORD,
    ) == 0
    {
        ShowWin32Error(GetLastError());
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShutDown_SwitchUser() {}
#[no_mangle]
pub unsafe extern "C" fn ShutDown_LockComputer() {
    if LockWorkStation() == 0 {
        ShowWin32Error(GetLastError());
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShutDown_Disconnect() {}

#[no_mangle]
pub unsafe extern "C" fn ShutDown_EjectComputer() {}
