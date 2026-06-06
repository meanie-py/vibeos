//! rpcserver.c — winlogon's RPC server for the remote-registry (winreg)
//! interface. Faithful translation. Almost every BaseReg*/Open* manager
//! routine is a stub that returns ERROR_SUCCESS (ReactOS does not implement
//! remote-registry server-side); the only live logic is StartRpcServer (bind
//! the ncacn_np \pipe\winreg endpoint + register winreg_v1_0_s_ifspec +
//! listen), the MIDL allocator glue, and the two shutdown forwarders that
//! call into the (already-Rust) shutdown module.
//!
//! ABI: every manager routine is __stdcall/WINAPI → `#[no_mangle] extern
//! "system"`, so on i386 the natural `_Name@N` decoration matches what the
//! winreg-stub crate's SERVER_ROUTINES table references (it read those `@N`
//! from the compiled C object). To make `@N` independent of how rustc rounds
//! sub-word args, the stub params use 4-byte types (`*mut c_void` / `u32`) —
//! a stub ignores them anyway, and 4 bytes each reproduces the C stack size
//! exactly (e.g. BaseInitiateSystemShutdown's two BOOLEANs each occupy a
//! 4-byte slot → @20, as the C object shows). A mismatch fails the cold link
//! loudly. StartRpcServer is cdecl (`BOOL StartRpcServer(VOID)`), called by
//! winlogon.c.

use crate::wstr;
use core::ffi::c_void;
use core::ptr::null_mut;
use winlogon_sys::nt::*;

#[allow(non_camel_case_types)]
type error_status_t = u32;
#[allow(non_camel_case_types)]
type RPC_STATUS = i32;

const RPC_S_OK: RPC_STATUS = 0; // == ERROR_SUCCESS
const RPC_C_PROTSEQ_MAX_REQS_DEFAULT: u32 = 10;

// rpcrt4 RPC_ENTRY (stdcall) server APIs — not in the winlogon-sys glob.
extern "system" {
    fn RpcServerUseProtseqEpW(
        Protseq: *const u16,
        MaxCalls: u32,
        Endpoint: *const u16,
        SecurityDescriptor: *mut c_void,
    ) -> RPC_STATUS;
    fn RpcServerRegisterIf(
        IfSpec: *mut c_void,
        MgrTypeUuid: *mut c_void,
        MgrEpv: *mut c_void,
    ) -> RPC_STATUS;
    fn RpcServerListen(MinimumCallThreads: u32, MaxCalls: u32, DontWait: u32) -> RPC_STATUS;
}

// The winreg interface spec, exported by the winreg-stub crate
// (`#[no_mangle] pub static winreg_v1_0_s_ifspec: RpcIfHandle`). Read as a
// raw pointer (RpcIfHandle is a repr-transparent newtype over *const c_void).
extern "C" {
    static winreg_v1_0_s_ifspec: *const c_void;
}

#[no_mangle]
pub unsafe extern "C" fn StartRpcServer() -> BOOL {
    let mut status: RPC_STATUS;

    status = RpcServerUseProtseqEpW(
        wstr!("ncacn_np").as_ptr(),
        RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
        wstr!("\\pipe\\winreg").as_ptr(),
        null_mut(),
    );
    if status != RPC_S_OK {
        return 0; // FALSE
    }

    status = RpcServerRegisterIf(winreg_v1_0_s_ifspec as *mut c_void, null_mut(), null_mut());
    if status != RPC_S_OK {
        return 0;
    }

    status = RpcServerListen(1, 20, 1 /* TRUE */);
    if status != RPC_S_OK {
        return 0;
    }

    1 // TRUE
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len as _) as *mut c_void
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut c_void) {
    HeapFree(GetProcessHeap(), 0, ptr as _);
}

#[no_mangle]
pub unsafe extern "system" fn RPC_HKEY_rundown(_hSCObject: *mut c_void) {}

// --- manager routines -------------------------------------------------------
// All stubs: TRACE + return ERROR_SUCCESS (or void). Arg lists use 4-byte
// types to reproduce the C `@N` decoration exactly; the bodies ignore them.

/* Function 0 */
#[no_mangle]
pub unsafe extern "system" fn OpenClassesRoot(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 1 */
#[no_mangle]
pub unsafe extern "system" fn OpenCurrentUser(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 2 */
#[no_mangle]
pub unsafe extern "system" fn OpenLocalMachine(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 3 */
#[no_mangle]
pub unsafe extern "system" fn OpenPerformanceData(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 4 */
#[no_mangle]
pub unsafe extern "system" fn OpenUsers(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 5 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegCloseKey(_hKey: *mut c_void) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 6 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegCreateKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
    _lpClass: *mut c_void,
    _dwOptions: u32,
    _samDesired: u32,
    _lpSecurityAttributes: *mut c_void,
    _phkResult: *mut c_void,
    _lpdwDisposition: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 7 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegDeleteKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 8 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegDeleteValue(
    _hKey: *mut c_void,
    _lpValueName: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 9 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegEnumKey(
    _hKey: *mut c_void,
    _dwIndex: u32,
    _lpNameIn: *mut c_void,
    _lpNameOut: *mut c_void,
    _lpClassIn: *mut c_void,
    _lplpClassOut: *mut c_void,
    _lpftLastWriteTime: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 10 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegEnumValue(
    _hKey: *mut c_void,
    _dwIndex: u32,
    _lpValueNameIn: *mut c_void,
    _lpValueNameOut: *mut c_void,
    _lpType: *mut c_void,
    _lpData: *mut c_void,
    _lpcbData: *mut c_void,
    _lpcbLen: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 11 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegFlushKey(_hKey: *mut c_void) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 12 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegGetKeySecurity(
    _hKey: *mut c_void,
    _SecurityInformation: u32,
    _pRpcSecurityDescriptorIn: *mut c_void,
    _pRpcSecurityDescriptorOut: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 13 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegLoadKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
    _lpFile: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 14 - Not used on wire */
#[no_mangle]
pub unsafe extern "system" fn BaseRegNotifyChangeKeyValue(_IDL_handle: *mut c_void) {}

/* Function 15 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegOpenKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
    _dwOptions: u32,
    _samDesired: u32,
    _phkResult: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 16 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegQueryInfoKey(
    _hKey: *mut c_void,
    _lpClassIn: *mut c_void,
    _lpClassOut: *mut c_void,
    _lpcSubKeys: *mut c_void,
    _lpcbMaxSubKeyLen: *mut c_void,
    _lpcbMaxClassLen: *mut c_void,
    _lpcValues: *mut c_void,
    _lpcbMaxValueNameLen: *mut c_void,
    _lpcbMaxValueLen: *mut c_void,
    _lpcbSecurityDescriptor: *mut c_void,
    _lpftLastWriteTime: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 17 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegQueryValue(
    _hKey: *mut c_void,
    _lpValueName: *mut c_void,
    _lpType: *mut c_void,
    _lpData: *mut c_void,
    _lpcbData: *mut c_void,
    _lpcbLen: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 18 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegReplaceKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
    _lpNewFile: *mut c_void,
    _lpOldFile: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 19 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegRestoreKey(
    _hKey: *mut c_void,
    _lpFile: *mut c_void,
    _Flags: u32,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 20 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegSaveKey(
    _hKey: *mut c_void,
    _lpFile: *mut c_void,
    _pSecurityAttributes: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 21 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegSetKeySecurity(
    _hKey: *mut c_void,
    _SecurityInformation: u32,
    _pRpcSecurityDescriptor: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 22 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegSetValue(
    _hKey: *mut c_void,
    _lpValueName: *mut c_void,
    _dwType: u32,
    _lpData: *mut c_void,
    _cbData: u32,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 23 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegUnLoadKey(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 24 */
#[no_mangle]
pub unsafe extern "system" fn BaseInitiateSystemShutdown(
    ServerName: *mut c_void,
    lpMessage: *mut c_void,
    dwTimeout: u32,
    bForceAppsClosed: u32,
    bRebootAfterShutdown: u32,
) -> u32 {
    BaseInitiateSystemShutdownEx(
        ServerName,
        lpMessage,
        dwTimeout,
        bForceAppsClosed,
        bRebootAfterShutdown,
        0,
    )
}

/* Function 25 */
#[no_mangle]
pub unsafe extern "system" fn BaseAbortSystemShutdown(_ServerName: *mut c_void) -> u32 {
    // FIXME: Verify that the caller actually has the correct privileges
    crate::shutdown::TerminateSystemShutdown()
}

/* Function 26 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegGetVersion(
    _hKey: *mut c_void,
    _lpdwVersion: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 27 */
#[no_mangle]
pub unsafe extern "system" fn OpenCurrentConfig(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 28 - Not used on wire */
#[no_mangle]
pub unsafe extern "system" fn OpenDynData(_IDL_handle: *mut c_void) {}

/* Function 29 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegQueryMultipleValues(
    _hKey: *mut c_void,
    _val_listIn: *mut c_void,
    _val_listOut: *mut c_void,
    _num_vals: u32,
    _lpvalueBuf: *mut c_void,
    _ldwTotsize: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 30 */
#[no_mangle]
pub unsafe extern "system" fn BaseInitiateSystemShutdownEx(
    _ServerName: *mut c_void,
    lpMessage: *mut c_void,
    dwTimeout: u32,
    bForceAppsClosed: u32,
    bRebootAfterShutdown: u32,
    dwReason: u32,
) -> u32 {
    // FIXME: Verify that the caller actually has the correct privileges
    crate::shutdown::StartSystemShutdown(
        lpMessage as PUNICODE_STRING,
        dwTimeout,
        bForceAppsClosed as u8,
        bRebootAfterShutdown as u8,
        dwReason,
    )
}

/* Function 31 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegSaveKeyEx(
    _hKey: *mut c_void,
    _lpFile: *mut c_void,
    _pSecurityAttributes: *mut c_void,
    _Flags: u32,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 32 */
#[no_mangle]
pub unsafe extern "system" fn OpenPerformanceText(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 33 */
#[no_mangle]
pub unsafe extern "system" fn OpenPerformanceNlsText(
    _ServerName: *mut c_void,
    _samDesired: u32,
    _phKey: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 34 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegQueryMultipleValues2(
    _hKey: *mut c_void,
    _val_listIn: *mut c_void,
    _val_listOut: *mut c_void,
    _num_vals: u32,
    _lpvalueBuf: *mut c_void,
    _ldwTotsize: *mut c_void,
    _ldwRequiredSize: *mut c_void,
) -> error_status_t {
    ERROR_SUCCESS
}

/* Function 35 */
#[no_mangle]
pub unsafe extern "system" fn BaseRegDeleteKeyEx(
    _hKey: *mut c_void,
    _lpSubKey: *mut c_void,
    _AccessMask: u32,
    _Reserved: u32,
) -> error_status_t {
    ERROR_SUCCESS
}
