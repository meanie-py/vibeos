//! winsrv init.c → Rust. Translates:
//!   - SHUTDOWN_SETTINGS struct + ShutdownSettings global
//!   - GetRegIntFromID (registry int reader with REG_SZ/REG_DWORD fallback)
//!   - GetTimeouts (loads per-user + per-system shutdown timeouts)
//!   - DllMain
//!
//! Cohab status: init.c is currently still C-compiled (winsrv cmake
//! unchanged). Cohabitation flip will rename DllMain to `#[no_mangle]
//! pub unsafe extern "system" fn DllMain` under `winsrv_dll_only`
//! feature and comment out init.c in win32ss/user/winsrv/CMakeLists.txt.

use ntdll_sys::nt::*;

const DEFAULT_AUTO_END_TASKS: ULONG = 0; // FALSE
const DEFAULT_HUNG_APP_TIMEOUT: ULONG = 5000;
const DEFAULT_WAIT_TO_KILL_APP_TIMEOUT: ULONG = 20000;
const DEFAULT_PROCESS_TERMINATE_TIMEOUT: ULONG = 90000;

const OBJ_CASE_INSENSITIVE: ULONG = 0x00000040;
const MAXIMUM_ALLOWED: ACCESS_MASK = 0x02000000;
const KEY_READ: ACCESS_MASK = 0x00020019;
const REG_SZ: ULONG = 1;
const REG_DWORD: ULONG = 4;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as NTSTATUS;

#[repr(C)]
pub struct SHUTDOWN_SETTINGS {
    pub AutoEndTasks: u32, // BOOL = ULONG
    pub HungAppTimeout: ULONG,
    pub WaitToKillAppTimeout: ULONG,
    pub WaitToKillServiceTimeout: ULONG,
    pub ProcessTerminateTimeout: ULONG,
}

// Cohabited globals: the C init.c also defines this. When the cohab
// flip happens, comment out the C definition and let this no_mangle
// take over. For now this `pub static mut` is unreferenced (the C
// instance in init.c is the live one).
#[no_mangle]
pub static mut ShutdownSettings: SHUTDOWN_SETTINGS = SHUTDOWN_SETTINGS {
    AutoEndTasks: DEFAULT_AUTO_END_TASKS,
    HungAppTimeout: DEFAULT_HUNG_APP_TIMEOUT,
    WaitToKillAppTimeout: DEFAULT_WAIT_TO_KILL_APP_TIMEOUT,
    WaitToKillServiceTimeout: DEFAULT_WAIT_TO_KILL_APP_TIMEOUT,
    ProcessTerminateTimeout: DEFAULT_PROCESS_TERMINATE_TIMEOUT,
};

// InitializeObjectAttributes is a C macro — inline equivalent.
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PSECURITY_DESCRIPTOR,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

#[inline(always)]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }

/// Read a registry value as a ULONG, accepting REG_SZ ("12345" decimal)
/// or REG_DWORD. Returns `default_value` on any miss.
unsafe fn GetRegIntFromID(
    key_handle: HANDLE,
    value_name: *const WCHAR,
    default_value: ULONG,
) -> ULONG {
    let mut value_string: _UNICODE_STRING = core::mem::zeroed();
    let mut length: ULONG = 0;
    // KEY_VALUE_PARTIAL_INFORMATION + 32 WCHARs of payload buffer.
    // Layout: TitleIndex(4) + Type(4) + DataLength(4) + Data[1].
    let mut buffer = [0u8; core::mem::size_of::<_KEY_VALUE_PARTIAL_INFORMATION>() + 32 * 2];
    let partial_info = buffer.as_mut_ptr() as *mut _KEY_VALUE_PARTIAL_INFORMATION;

    RtlInitUnicodeString(&mut value_string, value_name);
    let mut status = NtQueryValueKey(
        key_handle,
        &mut value_string,
        KeyValuePartialInformation,
        partial_info as PVOID,
        buffer.len() as ULONG,
        &mut length,
    );

    let mut value: ULONG = 0;
    if NT_SUCCESS(status) {
        if (*partial_info).Type == REG_SZ {
            // The string lives in Data[]. Reinterpret as PWCHAR and parse decimal.
            RtlInitUnicodeString(
                &mut value_string,
                core::ptr::addr_of_mut!((*partial_info).Data) as *const WCHAR,
            );
            status = RtlUnicodeStringToInteger(&value_string, 10, &mut value);
        } else if (*partial_info).Type == REG_DWORD {
            value = *(core::ptr::addr_of_mut!((*partial_info).Data) as *const ULONG);
            status = STATUS_SUCCESS;
        } else {
            // Unexpected type — fall back to default.
            status = STATUS_UNSUCCESSFUL;
        }
    }

    if !NT_SUCCESS(status) {
        value = default_value;
    }
    value
}

/// Populate ShutdownSettings from the registry (per-user
/// `Control Panel\Desktop` + system `CurrentControlSet\Control`). C
/// `FASTCALL` is `__fastcall` on i386 (single-arg passes via ecx),
/// and a no-op (cdecl) on amd64/arm64.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn GetTimeouts(shutdown_settings: *mut SHUTDOWN_SETTINGS) {
    get_timeouts_impl(shutdown_settings)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn GetTimeouts(shutdown_settings: *mut SHUTDOWN_SETTINGS) {
    get_timeouts_impl(shutdown_settings)
}

unsafe fn get_timeouts_impl(shutdown_settings: *mut SHUTDOWN_SETTINGS) {
    // Initialize defaults first (matches C parity).
    (*shutdown_settings).AutoEndTasks = DEFAULT_AUTO_END_TASKS;
    (*shutdown_settings).HungAppTimeout = DEFAULT_HUNG_APP_TIMEOUT;
    (*shutdown_settings).WaitToKillAppTimeout = DEFAULT_WAIT_TO_KILL_APP_TIMEOUT;
    (*shutdown_settings).WaitToKillServiceTimeout = (*shutdown_settings).WaitToKillAppTimeout;
    (*shutdown_settings).ProcessTerminateTimeout = DEFAULT_PROCESS_TERMINATE_TIMEOUT;

    // Per-user desktop key.
    let mut current_user_key: HANDLE = core::ptr::null_mut();
    let status = RtlOpenCurrentUser(MAXIMUM_ALLOWED, &mut current_user_key);
    if NT_SUCCESS(status) {
        const DESKTOP_KEY: &[u16] = &[
            b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
            b' ' as u16, b'P' as u16, b'a' as u16, b'n' as u16, b'e' as u16, b'l' as u16,
            b'\\' as u16,
            b'D' as u16, b'e' as u16, b's' as u16, b'k' as u16, b't' as u16, b'o' as u16, b'p' as u16,
            0,
        ];
        let mut registry_string: _UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut registry_string, DESKTOP_KEY.as_ptr());

        let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
        InitializeObjectAttributes(
            &mut object_attributes,
            &mut registry_string,
            OBJ_CASE_INSENSITIVE,
            current_user_key,
            core::ptr::null_mut(),
        );

        let mut key_handle: HANDLE = core::ptr::null_mut();
        let s = NtOpenKey(&mut key_handle, KEY_READ, &mut object_attributes);
        if NT_SUCCESS(s) {
            const HUNG_APP: &[u16] = &[
                b'H' as u16, b'u' as u16, b'n' as u16, b'g' as u16,
                b'A' as u16, b'p' as u16, b'p' as u16,
                b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16,
                b'o' as u16, b'u' as u16, b't' as u16, 0,
            ];
            const WAIT_KILL_APP: &[u16] = &[
                b'W' as u16, b'a' as u16, b'i' as u16, b't' as u16,
                b'T' as u16, b'o' as u16,
                b'K' as u16, b'i' as u16, b'l' as u16, b'l' as u16,
                b'A' as u16, b'p' as u16, b'p' as u16,
                b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16,
                b'o' as u16, b'u' as u16, b't' as u16, 0,
            ];
            const AUTO_END_TASKS: &[u16] = &[
                b'A' as u16, b'u' as u16, b't' as u16, b'o' as u16,
                b'E' as u16, b'n' as u16, b'd' as u16,
                b'T' as u16, b'a' as u16, b's' as u16, b'k' as u16, b's' as u16, 0,
            ];
            (*shutdown_settings).HungAppTimeout = GetRegIntFromID(
                key_handle, HUNG_APP.as_ptr(), DEFAULT_HUNG_APP_TIMEOUT,
            );
            (*shutdown_settings).WaitToKillAppTimeout = GetRegIntFromID(
                key_handle, WAIT_KILL_APP.as_ptr(), DEFAULT_WAIT_TO_KILL_APP_TIMEOUT,
            );
            (*shutdown_settings).AutoEndTasks = GetRegIntFromID(
                key_handle, AUTO_END_TASKS.as_ptr(), DEFAULT_AUTO_END_TASKS,
            );
            NtClose(key_handle);
        }
        NtClose(current_user_key);
    }

    // System control key.
    const CONTROL_KEY: &[u16] = &[
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16, b't' as u16,
        b'r' as u16, b'y' as u16, b'\\' as u16,
        b'M' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16,
        b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16, b't' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16,
        b'S' as u16, b'e' as u16, b't' as u16, b'\\' as u16,
        b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, 0,
    ];
    let mut registry_string: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut registry_string, CONTROL_KEY.as_ptr());

    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    InitializeObjectAttributes(
        &mut object_attributes,
        &mut registry_string,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    let mut key_handle: HANDLE = core::ptr::null_mut();
    let s = NtOpenKey(&mut key_handle, KEY_READ, &mut object_attributes);
    if NT_SUCCESS(s) {
        const WAIT_KILL_SVC: &[u16] = &[
            b'W' as u16, b'a' as u16, b'i' as u16, b't' as u16,
            b'T' as u16, b'o' as u16,
            b'K' as u16, b'i' as u16, b'l' as u16, b'l' as u16,
            b'S' as u16, b'e' as u16, b'r' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16,
            b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16,
            b'o' as u16, b'u' as u16, b't' as u16, 0,
        ];
        (*shutdown_settings).WaitToKillServiceTimeout = GetRegIntFromID(
            key_handle, WAIT_KILL_SVC.as_ptr(), (*shutdown_settings).WaitToKillAppTimeout,
        );

        const PROC_TERM: &[u16] = &[
            b'P' as u16, b'r' as u16, b'o' as u16, b'c' as u16, b'e' as u16, b's' as u16, b's' as u16,
            b'T' as u16, b'e' as u16, b'r' as u16, b'm' as u16, b'i' as u16, b'n' as u16, b'a' as u16,
            b't' as u16, b'e' as u16,
            b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16,
            b'o' as u16, b'u' as u16, b't' as u16, 0,
        ];
        (*shutdown_settings).ProcessTerminateTimeout = GetRegIntFromID(
            key_handle, PROC_TERM.as_ptr(), DEFAULT_PROCESS_TERMINATE_TIMEOUT,
        );
        if (*shutdown_settings).ProcessTerminateTimeout < DEFAULT_HUNG_APP_TIMEOUT {
            (*shutdown_settings).ProcessTerminateTimeout = DEFAULT_HUNG_APP_TIMEOUT;
        }
        NtClose(key_handle);
    }
}

// winsrv.dll DllMain — currently a private fn (C init.c owns the real
// DllMain). Gated behind winsrv_dll_only so when the cohab flip lands,
// renaming this to `#[no_mangle] pub extern "system" fn DllMain` and
// commenting out init.c's DllMain swaps the implementation in.
#[cfg_attr(feature = "winsrv_dll_only", no_mangle)]
#[allow(dead_code)]
pub unsafe extern "system" fn DllMain(
    _h_instance_dll: HINSTANCE,
    _dw_reason: DWORD,
    _lp_reserved: LPVOID,
) -> BOOL {
    1 // TRUE
}

type DWORD = ULONG;
type LPVOID = PVOID;
type BOOL = i32;
type HINSTANCE = HANDLE;
