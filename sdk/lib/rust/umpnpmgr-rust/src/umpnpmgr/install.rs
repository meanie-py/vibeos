// rustconv-assembly: whole-tu
use crate::prelude::*;
// cross-TU references: the C is one link unit, so a static in another .c is simply
// visible; in Rust each TU is a module and the reference has to be spelled.
use crate::umpnpmgr::umpnpmgr::GetSuppressNewUIValue;
use crate::umpnpmgr::rpcserver::{PNP_GetDeviceList, PNP_GetDeviceListSize};

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// precomp.h declares every one of these `extern`: they are module globals with EXTERNAL
// linkage, shared across the four TUs. Without #[no_mangle] each definition is mangled
// while sibling modules reference the plain name, and the crate links against undefined
// symbols — the failure only appears at LINK, never in `cargo check`.
#[no_mangle]
pub static mut hUserToken: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut hInstallEvent: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut hNoPendingInstalls: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut hDeviceInstallListMutex: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut DeviceInstallListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut hDeviceInstallListNotEmpty: HANDLE = core::ptr::null_mut();
// The C declares a LOCAL of the same name as a module global here and relies on C's
// shadowing; Rust forbids a let binding shadowing a static. Renamed the LOCAL only, so
// every use in this function still means the local, exactly as the C does.
unsafe extern "C" fn InstallDevice(DeviceInstance: PCWSTR, ShowWizard: u32) -> u32 {
    let mut DeviceInstalled: u32 = FALSE;
    let mut BytesWritten: u32 = 0;
    let mut Value: u32 = 0;
    let mut ErrCode: u32 = 0;
    let mut hInstallEventLocal: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hPipe: *mut core::ffi::c_void = INVALID_HANDLE_VALUE;
    let mut Environment: LPVOID = core::ptr::null_mut();
    let mut ProcessInfo: PROCESS_INFORMATION = core::mem::zeroed();
    let mut StartupInfo: STARTUPINFOW = core::mem::zeroed();
    let mut RandomUuid: [u8; 16] = [0; 16];
    let mut DeviceKey: HKEY = core::ptr::null_mut();
    let mut EventAttrs: SECURITY_ATTRIBUTES = core::mem::zeroed();
    let mut EventSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    let mut CommandLine: [u16; 116] = [0; 116];
    let mut InstallEventName: [u16; 73] = [0; 73];
    let mut PipeName: [u16; 74] = [0; 74];
    let mut UuidString: [u16; 39] = [0; 39];

    // DPRINT removed

    ZeroMemory!(core::ptr::addr_of_mut!(ProcessInfo) as *mut core::ffi::c_void, core::mem::size_of::<PROCESS_INFORMATION>());

    if RegOpenKeyExW(hEnumKey,
                      DeviceInstance,
                      0,
                      KEY_QUERY_VALUE,
                      core::ptr::addr_of_mut!(DeviceKey)) == (ERROR_SUCCESS) as i32
    {
        if RegQueryValueExW(DeviceKey,
                             w!("Class") as *const u16,
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::null_mut()) == (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            RegCloseKey(DeviceKey);
            return TRUE;
        }

        BytesWritten = core::mem::size_of::<u32>() as u32;
        if RegQueryValueExW(DeviceKey,
                             w!("ConfigFlags") as *const u16,
                             core::ptr::null_mut(),
                             core::ptr::null_mut(),
                             core::ptr::addr_of_mut!(Value) as PBYTE,
                             core::ptr::addr_of_mut!(BytesWritten)) == (ERROR_SUCCESS) as i32
        {
            if Value & (CONFIGFLAG_FAILEDINSTALL) as u32 != 0
            {
                // DPRINT removed
                RegCloseKey(DeviceKey);
                return TRUE;
            }
        }

        RegCloseKey(DeviceKey);
    }

    // DPRINT removed

    UuidCreate((RandomUuid.as_mut_ptr() as *mut core::ffi::c_void).cast());
    swprintf(UuidString.as_mut_ptr(), w!("{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}") as *const u16,
        // RandomUuid is a byte array standing in for the C's UUID: Data1..3 are read
        // unaligned, and Data4 is bytes 8..16 — printing bytes 0..8 again repeated the
        // first half of every install event/pipe name (found by scan_misaligned_cast).
        (RandomUuid.as_ptr() as *const u32).read_unaligned(),
        (RandomUuid.as_ptr().offset(4) as *const u16).read_unaligned() as c_uint,
        (RandomUuid.as_ptr().offset(6) as *const u16).read_unaligned() as c_uint,
        RandomUuid[8] as c_uint, RandomUuid[9] as c_uint, RandomUuid[10] as c_uint,
        RandomUuid[11] as c_uint, RandomUuid[12] as c_uint, RandomUuid[13] as c_uint,
        RandomUuid[14] as c_uint, RandomUuid[15] as c_uint);

    ErrCode = CreatePnpInstallEventSecurity(core::ptr::addr_of_mut!(EventSd));
    if ErrCode != ERROR_SUCCESS
    {
        // DPRINT removed
        return FALSE;
    }

    EventAttrs.nLength = core::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    EventAttrs.lpSecurityDescriptor = EventSd;
    EventAttrs.bInheritHandle = (FALSE) as i32;

    wcscpy(InstallEventName.as_mut_ptr(), w!("Global\\PNP_Device_Install_Event_0.") as *const u16);
    wcscat(InstallEventName.as_mut_ptr(), UuidString.as_ptr());
    hInstallEventLocal = CreateEventW(core::ptr::addr_of_mut!(EventAttrs), (TRUE) as i32, (FALSE) as i32, InstallEventName.as_ptr());
    HeapFree(GetProcessHeap(), 0, EventSd as LPVOID);
    if hInstallEventLocal.is_null()
    {
        // DPRINT removed
        DeviceInstalled = FALSE;
    } else {
        wcscpy(PipeName.as_mut_ptr(), w!("\\\\.\\pipe\\PNP_Device_Install_Pipe_0.") as *const u16);
        wcscat(PipeName.as_mut_ptr(), UuidString.as_ptr());
        hPipe = CreateNamedPipeW(PipeName.as_ptr(), (PIPE_ACCESS_OUTBOUND) as u32, (PIPE_TYPE_BYTE) as u32, 1, 512, 512, 0, core::ptr::null_mut());
        if hPipe == INVALID_HANDLE_VALUE
        {
            // DPRINT removed
            DeviceInstalled = FALSE;
        } else {
            wcscpy(CommandLine.as_mut_ptr(), w!("rundll32.exe newdev.dll,ClientSideInstall ") as *const u16);
            wcscat(CommandLine.as_mut_ptr(), PipeName.as_ptr());

            ZeroMemory!(core::ptr::addr_of_mut!(StartupInfo) as *mut core::ffi::c_void, core::mem::size_of::<STARTUPINFOW>());
            StartupInfo.cb = core::mem::size_of::<STARTUPINFOW>() as u32;

            if !hUserToken.is_null()
            {
                if CreateEnvironmentBlock(core::ptr::addr_of_mut!(Environment), hUserToken, (FALSE) as i32) == 0
                {
                    // DPRINT removed
                    DeviceInstalled = FALSE;
                } else {
                    if CreateProcessAsUserW(hUserToken, core::ptr::null_mut(), CommandLine.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as i32, (CREATE_UNICODE_ENVIRONMENT) as u32, Environment, core::ptr::null_mut(), core::ptr::addr_of_mut!(StartupInfo), core::ptr::addr_of_mut!(ProcessInfo)) == 0
                    {
                        // DPRINT removed
                        DeviceInstalled = FALSE;
                    } else {
                        if ConnectNamedPipe(hPipe, core::ptr::null_mut()) == 0
                        {
                            if GetLastError() != ERROR_PIPE_CONNECTED
                            {
                                // DPRINT removed
                                DeviceInstalled = FALSE;
                            } else {
                                Value = core::mem::size_of_val(&InstallEventName) as u32;
                                WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                                WriteFile(hPipe, InstallEventName.as_ptr() as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                                WriteFile(hPipe, core::ptr::addr_of!(ShowWizard) as *const core::ffi::c_void, core::mem::size_of_val(&ShowWizard) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                                Value = ((wcslen(DeviceInstance) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
                                WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                                WriteFile(hPipe, DeviceInstance as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                                WaitForSingleObject(ProcessInfo.hProcess, (INFINITE) as u32);

                                DeviceInstalled = if WaitForSingleObject(hInstallEventLocal, 0) == (WAIT_OBJECT_0) as u32 { 1 } else { 0 };
                            }
                        } else {
                            Value = core::mem::size_of_val(&InstallEventName) as u32;
                            WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                            WriteFile(hPipe, InstallEventName.as_ptr() as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            WriteFile(hPipe, core::ptr::addr_of!(ShowWizard) as *const core::ffi::c_void, core::mem::size_of_val(&ShowWizard) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            Value = ((wcslen(DeviceInstance) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
                            WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                            WriteFile(hPipe, DeviceInstance as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            WaitForSingleObject(ProcessInfo.hProcess, (INFINITE) as u32);

                            DeviceInstalled = if WaitForSingleObject(hInstallEventLocal, 0) == (WAIT_OBJECT_0) as u32 { 1 } else { 0 };
                        }
                    }
                }
            }
            else
            {
                if CreateProcessW(core::ptr::null_mut(), CommandLine.as_mut_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), (FALSE) as i32, 0, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(StartupInfo), core::ptr::addr_of_mut!(ProcessInfo)) == 0
                {
                    // DPRINT removed
                    DeviceInstalled = FALSE;
                } else {
                    if ConnectNamedPipe(hPipe, core::ptr::null_mut()) == 0
                    {
                        if GetLastError() != ERROR_PIPE_CONNECTED
                        {
                            // DPRINT removed
                            DeviceInstalled = FALSE;
                        } else {
                            Value = core::mem::size_of_val(&InstallEventName) as u32;
                            WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                            WriteFile(hPipe, InstallEventName.as_ptr() as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            WriteFile(hPipe, core::ptr::addr_of!(ShowWizard) as *const core::ffi::c_void, core::mem::size_of_val(&ShowWizard) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            Value = ((wcslen(DeviceInstance) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
                            WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                            WriteFile(hPipe, DeviceInstance as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                            WaitForSingleObject(ProcessInfo.hProcess, (INFINITE) as u32);

                            DeviceInstalled = if WaitForSingleObject(hInstallEventLocal, 0) == (WAIT_OBJECT_0) as u32 { 1 } else { 0 };
                        }
                    } else {
                        Value = core::mem::size_of_val(&InstallEventName) as u32;
                        WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                        WriteFile(hPipe, InstallEventName.as_ptr() as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                        WriteFile(hPipe, core::ptr::addr_of!(ShowWizard) as *const core::ffi::c_void, core::mem::size_of_val(&ShowWizard) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                        Value = ((wcslen(DeviceInstance) + 1) * (core::mem::size_of::<u16>() as u32) as usize) as u32;
                        WriteFile(hPipe, core::ptr::addr_of_mut!(Value) as *const core::ffi::c_void, core::mem::size_of_val(&Value) as u32, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());
                        WriteFile(hPipe, DeviceInstance as *const core::ffi::c_void, Value, core::ptr::addr_of_mut!(BytesWritten), core::ptr::null_mut());

                        WaitForSingleObject(ProcessInfo.hProcess, (INFINITE) as u32);

                        DeviceInstalled = if WaitForSingleObject(hInstallEventLocal, 0) == (WAIT_OBJECT_0) as u32 { 1 } else { 0 };
                    }
                }
            }
        }
    }

    if !hInstallEventLocal.is_null() {
        CloseHandle(hInstallEventLocal);
    }

    if hPipe != INVALID_HANDLE_VALUE {
        CloseHandle(hPipe);
    }

    if !Environment.is_null() {
        DestroyEnvironmentBlock(Environment);
    }

    if !ProcessInfo.hProcess.is_null() {
        CloseHandle(ProcessInfo.hProcess);
    }

    if !ProcessInfo.hThread.is_null() {
        CloseHandle(ProcessInfo.hThread);
    }

    if DeviceInstalled == 0
    {
        // DPRINT removed
    }

    DeviceInstalled
}
unsafe extern "C" fn ReadRegSzKey(
    hKey: HKEY,
    pszKey: LPCWSTR,
    pValue: *mut LPWSTR,
) -> i32 {
    let mut rc: i32;
    let mut dwType: u32 = 0;
    let mut cbData: u32 = 0;
    let mut Value: LPWSTR;

    if pValue.is_null() {
        return (ERROR_INVALID_PARAMETER) as i32;
    }

    *pValue = core::ptr::null_mut();
    rc = RegQueryValueExW(
        hKey,
        pszKey,
        core::ptr::null_mut(),
        &mut dwType,
        core::ptr::null_mut(),
        &mut cbData,
    );
    if rc != (ERROR_SUCCESS) as i32 {
        return rc;
    }
    if dwType != (REG_SZ) as u32 {
        return (ERROR_FILE_NOT_FOUND) as i32;
    }
    Value = HeapAlloc(
        GetProcessHeap(),
        0,
        (cbData + core::mem::size_of::<u16>() as u32) as SIZE_T,
    ) as LPWSTR;
    if Value.is_null() {
        return (ERROR_NOT_ENOUGH_MEMORY) as i32;
    }
    rc = RegQueryValueExW(
        hKey,
        pszKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Value as LPBYTE,
        &mut cbData,
    );
    if rc != (ERROR_SUCCESS) as i32 {
        HeapFree(GetProcessHeap(), 0, Value as LPVOID);
        return rc;
    }
    *Value.offset((cbData / core::mem::size_of::<u16>() as u32) as isize) = 0;

    *pValue = Value;
    return (ERROR_SUCCESS) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn SetupIsActive() -> BOOL {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut regType: u32 = 0;
    let mut active: u32 = 0;
    let mut size: u32 = 0;
    let mut rc: i32;
    let mut ret: BOOL = FALSE as BOOL;

    rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, w!("SYSTEM\\Setup") as LPCWSTR, 0, KEY_QUERY_VALUE, &mut hKey);
    if rc != (ERROR_SUCCESS) as i32 {
        if !hKey.is_null() {
            RegCloseKey(hKey);
        }
        // DPRINT removed
        return ret;
    }

    size = core::mem::size_of::<u32>() as u32;
    rc = RegQueryValueExW(hKey, w!("SystemSetupInProgress") as LPCWSTR, core::ptr::null_mut(), &mut regType, &mut active as *mut u32 as LPBYTE, &mut size);
    if rc != (ERROR_SUCCESS) as i32 {
        if !hKey.is_null() {
            RegCloseKey(hKey);
        }
        // DPRINT removed
        return ret;
    }
    if regType != (REG_DWORD) as u32 || size != core::mem::size_of::<u32>() as u32 {
        if !hKey.is_null() {
            RegCloseKey(hKey);
        }
        // DPRINT removed
        return ret;
    }

    ret = if active != 0 { 1 } else { 0 };

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    // DPRINT removed

    ret
}
#[no_mangle]
pub unsafe extern "C" fn CreatePnpInstallEventSecurity(
    EventSd: *mut PSECURITY_DESCRIPTOR
) -> u32 {
    let mut ErrCode: u32;
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclSize: u32;
    let mut AbsoluteSd: _SECURITY_DESCRIPTOR = core::mem::zeroed();
    let mut Size: u32 = 0;
    let mut RelativeSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
    let mut SystemSid: PSID = core::ptr::null_mut();
    let mut AdminsSid: PSID = core::ptr::null_mut();
    static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY { Value: SECURITY_NT_AUTHORITY };

    'block: {
        if AllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            1,
            SECURITY_LOCAL_SYSTEM_RID,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            core::ptr::addr_of_mut!(SystemSid)
        ) == 0 {
            return GetLastError();
        }

        if AllocateAndInitializeSid(
            core::ptr::addr_of_mut!(NtAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0,
            0,
            0,
            0,
            0,
            0,
            core::ptr::addr_of_mut!(AdminsSid)
        ) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        DaclSize = core::mem::size_of::<ACL>() as u32 +
                   core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32 + GetLengthSid(SystemSid) +
                   core::mem::size_of::<ACCESS_ALLOWED_ACE>() as u32 + GetLengthSid(AdminsSid);

        Dacl = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize as SIZE_T) as PACL;
        if Dacl.is_null() {
            ErrCode = ERROR_OUTOFMEMORY;
            break 'block;
        }
        if InitializeAcl(Dacl, DaclSize, ACL_REVISION) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if AddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            (EVENT_ALL_ACCESS) as u32,
            SystemSid
        ) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if AddAccessAllowedAce(
            Dacl,
            ACL_REVISION,
            (EVENT_ALL_ACCESS) as u32,
            AdminsSid
        ) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if InitializeSecurityDescriptor((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), SECURITY_DESCRIPTOR_REVISION) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if SetSecurityDescriptorDacl((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), (TRUE) as i32, Dacl, (FALSE) as i32) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if SetSecurityDescriptorOwner((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), SystemSid, (FALSE) as i32) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if SetSecurityDescriptorGroup((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), AdminsSid, (FALSE) as i32) == 0 {
            ErrCode = GetLastError();
            break 'block;
        }

        if MakeSelfRelativeSD((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), core::ptr::null_mut(), core::ptr::addr_of_mut!(Size)) != 0 ||
           GetLastError() != ERROR_INSUFFICIENT_BUFFER {
            ErrCode = GetLastError();
            break 'block;
        }

        RelativeSd = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, Size as SIZE_T) as PSECURITY_DESCRIPTOR;
        if RelativeSd.is_null() {
            ErrCode = ERROR_OUTOFMEMORY;
            break 'block;
        }
        if MakeSelfRelativeSD((core::ptr::addr_of_mut!(AbsoluteSd)).cast(), RelativeSd, core::ptr::addr_of_mut!(Size)) == 0 {
            ErrCode = GetLastError();
            HeapFree(GetProcessHeap(), 0, RelativeSd as LPVOID);
            break 'block;
        }

        *EventSd = RelativeSd;
        ErrCode = ERROR_SUCCESS;
    }

    if !SystemSid.is_null() {
        FreeSid(SystemSid);
    }

    if !AdminsSid.is_null() {
        FreeSid(AdminsSid);
    }

    if !Dacl.is_null() {
        HeapFree(GetProcessHeap(), 0, Dacl as LPVOID);
    }

    ErrCode
}
unsafe extern "C" fn IsConsoleBoot() -> i32 {
    let mut ControlKey: HKEY = core::ptr::null_mut();
    let mut SystemStartOptions: LPWSTR = core::ptr::null_mut();
    let mut CurrentOption: LPWSTR;
    let mut NextOption: LPWSTR;
    let mut ConsoleBoot: i32 = (FALSE) as i32;
    let mut rc: i32;

    rc = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("SYSTEM\\CurrentControlSet\\Control") as LPCWSTR,
        0,
        KEY_QUERY_VALUE,
        &mut ControlKey,
    );

    rc = ReadRegSzKey(ControlKey, w!("SystemStartOptions") as LPCWSTR, &mut SystemStartOptions);
    if rc != (ERROR_SUCCESS) as i32 {
        if !ControlKey.is_null() {
            RegCloseKey(ControlKey);
        }
        HeapFree(GetProcessHeap(), 0, SystemStartOptions as LPVOID);
        return ConsoleBoot;
    }

    CurrentOption = SystemStartOptions;
    while !CurrentOption.is_null() {
        NextOption = wcschr(CurrentOption, 0x0020u16);
        if !NextOption.is_null() {
            *NextOption = 0x0000u16;
        }
        if _wcsicmp(CurrentOption, w!("CONSOLE") as LPCWSTR) == 0 {
            // DPRINT removed
            ConsoleBoot = (TRUE) as i32;
            if !ControlKey.is_null() {
                RegCloseKey(ControlKey);
            }
            HeapFree(GetProcessHeap(), 0, SystemStartOptions as LPVOID);
            return ConsoleBoot;
        }
        CurrentOption = if !NextOption.is_null() { NextOption.offset(1) } else { core::ptr::null_mut() };
    }

    if !ControlKey.is_null() {
        RegCloseKey(ControlKey);
    }
    HeapFree(GetProcessHeap(), 0, SystemStartOptions as LPVOID);
    ConsoleBoot
}
#[no_mangle]
pub unsafe extern "C" fn IsUISuppressionAllowed() -> BOOL {
    (g_IsUISuppressed != 0 || GetSuppressNewUIValue() != 0) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn DeviceInstallThread(lpParameter: LPVOID) -> u32 {
    let mut ListEntry: PLIST_ENTRY;
    let mut Params: *mut DeviceInstallParams;

    UNREFERENCED_PARAMETER!(lpParameter);

    // DPRINT removed

    let mut deviceList: PWSTR = core::ptr::null_mut();

    'outer: loop {
        let mut devListSize: u32 = 0;
        let status = PNP_GetDeviceListSize(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut devListSize,
            0,
        );
        if status != (CR_SUCCESS) as u32 {
            break 'outer;
        }

        deviceList = HeapAlloc(
            GetProcessHeap(),
            0,
            ((devListSize as usize) * core::mem::size_of::<u16>()) as SIZE_T,
        ) as PWSTR;
        if deviceList.is_null() {
            break 'outer;
        }

        let status = PNP_GetDeviceList(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            deviceList,
            &mut devListSize,
            0,
        );
        if status == (CR_BUFFER_SMALL) as u32 {
            HeapFree(GetProcessHeap(), 0, deviceList as LPVOID);
        } else if status != (CR_SUCCESS) as u32 {
            // DPRINT removed
            HeapFree(GetProcessHeap(), 0, deviceList as LPVOID);
            break 'outer;
        } else {
            break;
        }
    }

    let mut currentDev = deviceList;
    while *currentDev.offset(0 as isize) != UNICODE_NULL {
        InstallDevice(currentDev, FALSE);
        currentDev = currentDev.offset((lstrlenW(currentDev) + 1) as isize);
    }

    HeapFree(GetProcessHeap(), 0, deviceList as LPVOID);

    // DPRINT removed

    WaitForSingleObject(hInstallEvent, (INFINITE) as u32);

    let showWizard = SetupIsActive() == 0 && IsConsoleBoot() == 0;

    loop {
        WaitForSingleObject(hDeviceInstallListMutex, (INFINITE) as u32);
        ListEntry = if IsListEmpty(&mut DeviceInstallListHead) {
            core::ptr::null_mut()
        } else {
            RemoveHeadList(&mut DeviceInstallListHead)
        };
        ReleaseMutex(hDeviceInstallListMutex);

        if ListEntry.is_null() {
            SetEvent(hNoPendingInstalls);
            WaitForSingleObject(hDeviceInstallListNotEmpty, (INFINITE) as u32);
        } else {
            ResetEvent(hNoPendingInstalls);
            Params = (ListEntry as *mut u8).offset(-(core::mem::offset_of!(DeviceInstallParams, ListEntry) as isize)) as *mut DeviceInstallParams;
            InstallDevice((*Params).DeviceIds.as_ptr(), if showWizard && IsUISuppressionAllowed() == 0 { 1u32 } else { 0u32 });
            HeapFree(GetProcessHeap(), 0, Params as LPVOID);
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2005 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             base/services/umpnpmgr/install.c
//  * PURPOSE:          Device installer
//  * PROGRAMMER:       Eric Kohl (eric.kohl@reactos.org)
//  *                   Hervé Poussineau (hpoussin@reactos.org)
//  *                   Colin Finck (colin@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ******************************************************************/
//
// HANDLE hUserToken = NULL;
// HANDLE hInstallEvent = NULL;
// HANDLE hNoPendingInstalls = NULL;
//
// /* Device-install event list */
// HANDLE hDeviceInstallListMutex;
// LIST_ENTRY DeviceInstallListHead;
// HANDLE hDeviceInstallListNotEmpty;
//
// DWORD
// CreatePnpInstallEventSecurity(
//      PSECURITY_DESCRIPTOR *EventSd);
//
// /* FUNCTIONS *****************************************************************/
//
// static BOOL
// InstallDevice(PCWSTR DeviceInstance, BOOL ShowWizard)
// {
//     BOOL DeviceInstalled = FALSE;
//     DWORD BytesWritten;
//     DWORD Value;
//     DWORD ErrCode;
//     HANDLE hInstallEvent;
//     HANDLE hPipe = INVALID_HANDLE_VALUE;
//     LPVOID Environment = NULL;
//     PROCESS_INFORMATION ProcessInfo;
//     STARTUPINFOW StartupInfo;
//     UUID RandomUuid;
//     HKEY DeviceKey;
//     SECURITY_ATTRIBUTES EventAttrs;
//     PSECURITY_DESCRIPTOR EventSd;
//
//     /* The following lengths are constant (see below), they cannot overflow */
//     WCHAR CommandLine[116];
//     WCHAR InstallEventName[73];
//     WCHAR PipeName[74];
//     WCHAR UuidString[39];
//
//     DPRINT("InstallDevice(%S, %d)\n", DeviceInstance, ShowWizard);
//
//     ZeroMemory!(&ProcessInfo, sizeof(ProcessInfo));
//
//     if (RegOpenKeyExW(hEnumKey,
//                       DeviceInstance,
//                       0,
//                       KEY_QUERY_VALUE,
//                       &DeviceKey) == ERROR_SUCCESS)
//     {
//         if (RegQueryValueExW(DeviceKey,
//                              L"Class",
//                              NULL,
//                              NULL,
//                              NULL,
//                              NULL) == ERROR_SUCCESS)
//         {
//             DPRINT("No need to install: %S\n", DeviceInstance);
//             RegCloseKey(DeviceKey);
//             return TRUE;
//         }
//
//         BytesWritten = sizeof(DWORD);
//         if (RegQueryValueExW(DeviceKey,
//                              L"ConfigFlags",
//                              NULL,
//                              NULL,
//                              (PBYTE)&Value,
//                              &BytesWritten) == ERROR_SUCCESS)
//         {
//             if (Value & CONFIGFLAG_FAILEDINSTALL)
//             {
//                 DPRINT("No need to install: %S\n", DeviceInstance);
//                 RegCloseKey(DeviceKey);
//                 return TRUE;
//             }
//         }
//
//         RegCloseKey(DeviceKey);
//     }
//
//     DPRINT1("Installing: %S\n", DeviceInstance);
//
//     /* Create a random UUID for the named pipe & event*/
//     UuidCreate(&RandomUuid);
//     swprintf(UuidString, L"{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}",
//         RandomUuid.Data1, RandomUuid.Data2, RandomUuid.Data3,
//         RandomUuid.Data4[0], RandomUuid.Data4[1], RandomUuid.Data4[2],
//         RandomUuid.Data4[3], RandomUuid.Data4[4], RandomUuid.Data4[5],
//         RandomUuid.Data4[6], RandomUuid.Data4[7]);
//
//     ErrCode = CreatePnpInstallEventSecurity(&EventSd);
//     if (ErrCode != ERROR_SUCCESS)
//     {
//         DPRINT1("CreatePnpInstallEventSecurity failed with error %u\n", GetLastError());
//         return FALSE;
//     }
//
//     /* Set up the security attributes for the event */
//     EventAttrs.nLength = sizeof(SECURITY_ATTRIBUTES);
//     EventAttrs.lpSecurityDescriptor = EventSd;
//     EventAttrs.bInheritHandle = FALSE;
//
//     /* Create the event */
//     wcscpy(InstallEventName, L"Global\\PNP_Device_Install_Event_0.");
//     wcscat(InstallEventName, UuidString);
//     hInstallEvent = CreateEventW(&EventAttrs, TRUE, FALSE, InstallEventName);
//     HeapFree(GetProcessHeap(), 0, EventSd);
//     if (!hInstallEvent)
//     {
//         DPRINT1("CreateEventW('%ls') failed with error %lu\n", InstallEventName, GetLastError());
//         goto cleanup;
//     }
//
//     /* Create the named pipe */
//     wcscpy(PipeName, L"\\\\.\\pipe\\PNP_Device_Install_Pipe_0.");
//     wcscat(PipeName, UuidString);
//     hPipe = CreateNamedPipeW(PipeName, PIPE_ACCESS_OUTBOUND, PIPE_TYPE_BYTE, 1, 512, 512, 0, NULL);
//     if (hPipe == INVALID_HANDLE_VALUE)
//     {
//         DPRINT1("CreateNamedPipeW failed with error %u\n", GetLastError());
//         goto cleanup;
//     }
//
//     /* Launch rundll32 to call ClientSideInstallW */
//     wcscpy(CommandLine, L"rundll32.exe newdev.dll,ClientSideInstall ");
//     wcscat(CommandLine, PipeName);
//
//     ZeroMemory!(&StartupInfo, sizeof(StartupInfo));
//     StartupInfo.cb = sizeof(StartupInfo);
//
//     if (hUserToken)
//     {
//         /* newdev has to run under the environment of the current user */
//         if (!CreateEnvironmentBlock(&Environment, hUserToken, FALSE))
//         {
//             DPRINT1("CreateEnvironmentBlock failed with error %d\n", GetLastError());
//             goto cleanup;
//         }
//
//         if (!CreateProcessAsUserW(hUserToken, NULL, CommandLine, NULL, NULL, FALSE, CREATE_UNICODE_ENVIRONMENT, Environment, NULL, &StartupInfo, &ProcessInfo))
//         {
//             DPRINT1("CreateProcessAsUserW failed with error %u\n", GetLastError());
//             goto cleanup;
//         }
//     }
//     else
//     {
//         /* FIXME: This is probably not correct, I guess newdev should never be run with SYSTEM privileges.
//
//            Still, we currently do that in 2nd stage setup and probably Console mode as well, so allow it here.
//            (ShowWizard is only set to FALSE for these two modes) */
//         ASSERT(!ShowWizard);
//
//         if (!CreateProcessW(NULL, CommandLine, NULL, NULL, FALSE, 0, NULL, NULL, &StartupInfo, &ProcessInfo))
//         {
//             DPRINT1("CreateProcessW failed with error %u\n", GetLastError());
//             goto cleanup;
//         }
//     }
//
//     /* Wait for the function to connect to our pipe */
//     if (!ConnectNamedPipe(hPipe, NULL))
//     {
//         if (GetLastError() != ERROR_PIPE_CONNECTED)
//         {
//             DPRINT1("ConnectNamedPipe failed with error %u\n", GetLastError());
//             goto cleanup;
//         }
//     }
//
//     /* Pass the data. The following output is partly compatible to Windows XP SP2 (researched using a modified newdev.dll to log this stuff) */
//     Value = sizeof(InstallEventName);
//     WriteFile(hPipe, &Value, sizeof(Value), &BytesWritten, NULL);
//     WriteFile(hPipe, InstallEventName, Value, &BytesWritten, NULL);
//
//     /* I couldn't figure out what the following value means under WinXP. It's usually 0 in my tests, but was also 5 once.
//        Therefore the following line is entirely ReactOS-specific. We use the value here to pass the ShowWizard variable. */
//     WriteFile(hPipe, &ShowWizard, sizeof(ShowWizard), &BytesWritten, NULL);
//
//     Value = (wcslen(DeviceInstance) + 1) * sizeof(WCHAR);
//     WriteFile(hPipe, &Value, sizeof(Value), &BytesWritten, NULL);
//     WriteFile(hPipe, DeviceInstance, Value, &BytesWritten, NULL);
//
//     /* Wait for newdev.dll to finish processing */
//     WaitForSingleObject(ProcessInfo.hProcess, INFINITE);
//
//     /* If the event got signalled, this is success */
//     DeviceInstalled = WaitForSingleObject(hInstallEvent, 0) == WAIT_OBJECT_0;
//
// cleanup:
//     if (hInstallEvent)
//         CloseHandle(hInstallEvent);
//
//     if (hPipe != INVALID_HANDLE_VALUE)
//         CloseHandle(hPipe);
//
//     if (Environment)
//         DestroyEnvironmentBlock(Environment);
//
//     if (ProcessInfo.hProcess)
//         CloseHandle(ProcessInfo.hProcess);
//
//     if (ProcessInfo.hThread)
//         CloseHandle(ProcessInfo.hThread);
//
//     if (!DeviceInstalled)
//     {
//         DPRINT1("InstallDevice failed for DeviceInstance '%ws'\n", DeviceInstance);
//     }
//
//     return DeviceInstalled;
// }
//
//
// static LONG
// ReadRegSzKey(
//      HKEY hKey,
//      LPCWSTR pszKey,
//      LPWSTR* pValue)
// {
//     LONG rc;
//     DWORD dwType;
//     DWORD cbData = 0;
//     LPWSTR Value;
//
//     if (!pValue)
//         return ERROR_INVALID_PARAMETER;
//
//     *pValue = NULL;
//     rc = RegQueryValueExW(hKey, pszKey, NULL, &dwType, NULL, &cbData);
//     if (rc != ERROR_SUCCESS)
//         return rc;
//     if (dwType != REG_SZ)
//         return ERROR_FILE_NOT_FOUND;
//     Value = HeapAlloc(GetProcessHeap(), 0, cbData + sizeof(WCHAR));
//     if (!Value)
//         return ERROR_NOT_ENOUGH_MEMORY;
//     rc = RegQueryValueExW(hKey, pszKey, NULL, NULL, (LPBYTE)Value, &cbData);
//     if (rc != ERROR_SUCCESS)
//     {
//         HeapFree(GetProcessHeap(), 0, Value);
//         return rc;
//     }
//     /* NULL-terminate the string */
//     Value[cbData / sizeof(WCHAR)] = '\0';
//
//     *pValue = Value;
//     return ERROR_SUCCESS;
// }
//
//
// BOOL
// SetupIsActive(VOID)
// {
//     HKEY hKey = NULL;
//     DWORD regType, active, size;
//     LONG rc;
//     BOOL ret = FALSE;
//
//     rc = RegOpenKeyExW(HKEY_LOCAL_MACHINE, L"SYSTEM\\Setup", 0, KEY_QUERY_VALUE, &hKey);
//     if (rc != ERROR_SUCCESS)
//         goto cleanup;
//
//     size = sizeof(DWORD);
//     rc = RegQueryValueExW(hKey, L"SystemSetupInProgress", NULL, &regType, (LPBYTE)&active, &size);
//     if (rc != ERROR_SUCCESS)
//         goto cleanup;
//     if (regType != REG_DWORD || size != sizeof(DWORD))
//         goto cleanup;
//
//     ret = (active != 0);
//
// cleanup:
//     if (hKey != NULL)
//         RegCloseKey(hKey);
//
//     DPRINT("System setup in progress? %S\n", ret ? L"YES" : L"NO");
//
//     return ret;
// }
//
//
// /**
//  * @brief
//  * Creates a security descriptor for the PnP event
//  * installation.
//  *
//  * @param[out] EventSd
//  * A pointer to an allocated security descriptor
//  * for the event.
//  *
//  * @return
//  * ERROR_SUCCESS is returned if the function has
//  * successfully created the descriptor, otherwise
//  * a Win32 error code is returned.
//  *
//  * @remarks
//  * Only admins and local system have full power
//  * over this event as privileged users can install
//  * devices on a system.
//  */
// DWORD
// CreatePnpInstallEventSecurity(
//      PSECURITY_DESCRIPTOR *EventSd)
// {
//     DWORD ErrCode;
//     PACL Dacl;
//     ULONG DaclSize;
//     SECURITY_DESCRIPTOR AbsoluteSd;
//     ULONG Size = 0;
//     PSECURITY_DESCRIPTOR RelativeSd = NULL;
//     PSID SystemSid = NULL, AdminsSid = NULL;
//     static SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//
//     if (!AllocateAndInitializeSid(&NtAuthority,
//                                   1,
//                                   SECURITY_LOCAL_SYSTEM_RID,
//                                   0, 0, 0, 0, 0, 0, 0,
//                                   &SystemSid))
//     {
//         return GetLastError();
//     }
//
//     if (!AllocateAndInitializeSid(&NtAuthority,
//                                   2,
//                                   SECURITY_BUILTIN_DOMAIN_RID,
//                                   DOMAIN_ALIAS_RID_ADMINS,
//                                   0, 0, 0, 0, 0, 0,
//                                   &AdminsSid))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     /* Compute the size needed for the DACL and allocate it */
//     DaclSize = sizeof(ACL) +
//                sizeof(ACCESS_ALLOWED_ACE) + GetLengthSid(SystemSid) +
//                sizeof(ACCESS_ALLOWED_ACE) + GetLengthSid(AdminsSid);
//
//     Dacl = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, DaclSize);
//     if (!Dacl)
//     {
//         ErrCode = ERROR_OUTOFMEMORY;
//         goto Quit;
//     }
//     if (!InitializeAcl(Dacl, DaclSize, ACL_REVISION))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              EVENT_ALL_ACCESS,
//                              SystemSid))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              EVENT_ALL_ACCESS,
//                              AdminsSid))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!InitializeSecurityDescriptor(&AbsoluteSd, SECURITY_DESCRIPTOR_REVISION))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!SetSecurityDescriptorDacl(&AbsoluteSd, TRUE, Dacl, FALSE))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!SetSecurityDescriptorOwner(&AbsoluteSd, SystemSid, FALSE))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     if (!SetSecurityDescriptorGroup(&AbsoluteSd, AdminsSid, FALSE))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     /* Retrieve the size needed for the relative SD */
//     if (MakeSelfRelativeSD(&AbsoluteSd, NULL, &Size) ||
//         (GetLastError() != ERROR_INSUFFICIENT_BUFFER))
//     {
//         ErrCode = GetLastError();
//         goto Quit;
//     }
//
//     /* Build the relative SD */
//     RelativeSd = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, Size);
//     if (RelativeSd == NULL)
//     {
//         ErrCode = ERROR_OUTOFMEMORY;
//         goto Quit;
//     }
//     if (!MakeSelfRelativeSD(&AbsoluteSd, RelativeSd, &Size))
//     {
//         ErrCode = GetLastError();
//         HeapFree(GetProcessHeap(), 0, RelativeSd);
//         goto Quit;
//     }
//
//     *EventSd = RelativeSd;
//     ErrCode = ERROR_SUCCESS;
//
// Quit:
//     if (SystemSid)
//         FreeSid(SystemSid);
//
//     if (AdminsSid)
//         FreeSid(AdminsSid);
//
//     if (Dacl)
//         HeapFree(GetProcessHeap(), 0, Dacl);
//
//     return ErrCode;
// }
//
//
// static BOOL
// IsConsoleBoot(VOID)
// {
//     HKEY ControlKey = NULL;
//     LPWSTR SystemStartOptions = NULL;
//     LPWSTR CurrentOption, NextOption; /* Pointers into SystemStartOptions */
//     BOOL ConsoleBoot = FALSE;
//     LONG rc;
//
//     rc = RegOpenKeyExW(
//         HKEY_LOCAL_MACHINE,
//         L"SYSTEM\\CurrentControlSet\\Control",
//         0,
//         KEY_QUERY_VALUE,
//         &ControlKey);
//
//     rc = ReadRegSzKey(ControlKey, L"SystemStartOptions", &SystemStartOptions);
//     if (rc != ERROR_SUCCESS)
//         goto cleanup;
//
//     /* Check for CONSOLE switch in SystemStartOptions */
//     CurrentOption = SystemStartOptions;
//     while (CurrentOption)
//     {
//         NextOption = wcschr(CurrentOption, L' ');
//         if (NextOption)
//             *NextOption = L'\0';
//         if (_wcsicmp(CurrentOption, L"CONSOLE") == 0)
//         {
//             DPRINT("Found %S. Switching to console boot\n", CurrentOption);
//             ConsoleBoot = TRUE;
//             goto cleanup;
//         }
//         CurrentOption = NextOption ? NextOption + 1 : NULL;
//     }
//
// cleanup:
//     if (ControlKey != NULL)
//         RegCloseKey(ControlKey);
//     HeapFree(GetProcessHeap(), 0, SystemStartOptions);
//     return ConsoleBoot;
// }
//
//
// FORCEINLINE
// BOOL
// IsUISuppressionAllowed(VOID)
// {
//     /* Display the newdev.dll wizard UI only if it's allowed */
//     return (g_IsUISuppressed || GetSuppressNewUIValue());
// }
//
//
// /* Loop to install all queued devices installations */
// DWORD
// WINAPI
// DeviceInstallThread(LPVOID lpParameter)
// {
//     PLIST_ENTRY ListEntry;
//     DeviceInstallParams* Params;
//
//     UNREFERENCED_PARAMETER!(lpParameter);
//
//     // Step 1: install all drivers which were configured during the boot
//
//     DPRINT("Step 1: Installing devices configured during the boot\n");
//
//     PWSTR deviceList;
//
//     while (TRUE)
//     {
//         ULONG devListSize;
//         DWORD status = PNP_GetDeviceListSize(NULL, NULL, &devListSize, 0);
//         if (status != CR_SUCCESS)
//         {
//             goto Step2;
//         }
//
//         deviceList = HeapAlloc(GetProcessHeap(), 0, devListSize * sizeof(WCHAR));
//         if (!deviceList)
//         {
//             goto Step2;
//         }
//
//         status = PNP_GetDeviceList(NULL, NULL, deviceList, &devListSize, 0);
//         if (status == CR_BUFFER_SMALL)
//         {
//             HeapFree(GetProcessHeap(), 0, deviceList);
//         }
//         else if (status != CR_SUCCESS)
//         {
//             DPRINT1("PNP_GetDeviceList failed with error %u\n", status);
//             goto Cleanup;
//         }
//         else // status == CR_SUCCESS
//         {
//             break;
//         }
//     }
//
//     for (PWSTR currentDev = deviceList;
//          currentDev[0] != UNICODE_NULL;
//          currentDev += lstrlenW(currentDev) + 1)
//     {
//         InstallDevice(currentDev, FALSE);
//     }
//
// Cleanup:
//     HeapFree(GetProcessHeap(), 0, deviceList);
//
//     // Step 2: start the wait-loop for newly added devices
// Step2:
//
//     DPRINT("Step 2: Starting the wait-loop\n");
//
//     WaitForSingleObject(hInstallEvent, INFINITE);
//
//     BOOL showWizard = !SetupIsActive() && !IsConsoleBoot();
//
//     while (TRUE)
//     {
//         /* Dequeue the next oldest device-install event */
//         WaitForSingleObject(hDeviceInstallListMutex, INFINITE);
//         ListEntry = (IsListEmpty(&DeviceInstallListHead)
//                         ? NULL : RemoveHeadList(&mut DeviceInstallListHead));
//         ReleaseMutex(hDeviceInstallListMutex);
//
//         if (ListEntry == NULL)
//         {
//             SetEvent(hNoPendingInstalls);
//             WaitForSingleObject(hDeviceInstallListNotEmpty, INFINITE);
//         }
//         else
//         {
//             ResetEvent(hNoPendingInstalls);
//             Params = CONTAINING_RECORD(ListEntry, DeviceInstallParams, ListEntry);
//             InstallDevice(Params->DeviceIds, showWizard && !IsUISuppressionAllowed());
//             HeapFree(GetProcessHeap(), 0, Params);
//         }
//     }
//
//     return 0;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
