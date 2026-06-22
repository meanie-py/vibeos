use win32k_sys::win32k::*;
// NT_SUCCESS lives in winbase; NtUserx*/gfLogonProcess come from sibling modules.
use crate::winbase::NT_SUCCESS;
use crate::ntwrapper::NtUserxRegisterLogonProcess;
use crate::misc::dllmain::gfLogonProcess;
#[allow(unused_imports)]
use core::ffi::c_void;

extern "system" {
    // ntdll export (NTAPI): CSR client transport.
    fn CsrClientCallServer(
        ApiMessage: PCSR_API_MESSAGE,
        CaptureBuffer: PCSR_CAPTURE_BUFFER,
        ApiNumber: CSR_API_NUMBER,
        DataLength: ULONG,
    ) -> NTSTATUS;
    // misc/misc.c internal user32 symbol.
    fn UserSetLastNTError(Status: NTSTATUS);
    // user32 own-exports / internal helpers, resolved at link.
    fn UpdatePerUserImmEnabling();
    fn CliImmInitializeHotKeys(action: u32, hwnd: HWND);
}

// IntLoadPreloadKeyboardLayouts is provided by windows/input.rs as cdecl
// (#[no_mangle] extern "C") → declare it cdecl so the symbol matches.
extern "C" {
    fn IntLoadPreloadKeyboardLayouts();
}

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(user32) - debug channel macro, no Rust equivalent

// ERR(...) — ReactOS debug print; no-op in the cohab build.
macro_rules! ERR { ($($t:tt)*) => {{ }}; }

// csrmsg.h: CSR_CREATE_API_NUMBER(ServerId, ApiId) := (CSR_API_NUMBER)((ServerId<<16)|ApiId).
#[inline]
fn CSR_CREATE_API_NUMBER(ServerId: u32, ApiId: u32) -> CSR_API_NUMBER {
    ((ServerId << 16) | ApiId) as CSR_API_NUMBER
}

#[no_mangle]
pub unsafe extern "system" fn RegisterServicesProcess(ServicesProcessId: DWORD) -> BOOL {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let RegisterServicesProcessRequest: PUSER_REGISTER_SERVICES_PROCESS =
        core::ptr::addr_of_mut!(ApiMessage.Data.RegisterServicesProcessRequest);

    (*RegisterServicesProcessRequest).ProcessId = ServicesProcessId as ULONG_PTR;

    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpRegisterServicesProcess),
        core::mem::size_of_val(&*RegisterServicesProcessRequest) as u32,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        UserSetLastNTError(ApiMessage.Status);
        return 0;
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn RegisterLogonProcess(dwProcessId: DWORD, bRegister: BOOL) -> BOOL {
    // gfLogonProcess is BOOLEAN (u8) in dllmain; the CSR call returns BOOL (i32).
    gfLogonProcess = (NtUserxRegisterLogonProcess(dwProcessId, bRegister) != 0) as BOOLEAN;

    if gfLogonProcess != 0 {
        let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
        let RegisterLogonProcessRequest: PUSER_REGISTER_LOGON_PROCESS =
            core::ptr::addr_of_mut!(ApiMessage.Data.RegisterLogonProcessRequest);

        (*RegisterLogonProcessRequest).ProcessId = dwProcessId as ULONG_PTR;
        (*RegisterLogonProcessRequest).Register = bRegister;

        CsrClientCallServer(
            core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
            core::ptr::null_mut(),
            CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpRegisterLogonProcess),
            core::mem::size_of::<USER_REGISTER_LOGON_PROCESS>() as u32,
        );

        if !NT_SUCCESS(ApiMessage.Status) {
            ERR!(b"Failed to register logon process with CSRSS\n\0".as_ptr() as *const i8);
            UserSetLastNTError(ApiMessage.Status);
        }
    }

    gfLogonProcess as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn Logon(IsLogon: BOOL) {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let LogonRequest: PUSER_LOGON = core::ptr::addr_of_mut!(ApiMessage.Data.LogonRequest);

    (*LogonRequest).IsLogon = IsLogon;
    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpLogon),
        core::mem::size_of::<USER_LOGON>() as u32,
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn Logon(IsLogon: BOOL) {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let LogonRequest: PUSER_LOGON = core::ptr::addr_of_mut!(ApiMessage.Data.LogonRequest);

    (*LogonRequest).IsLogon = IsLogon;
    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpLogon),
        core::mem::size_of::<USER_LOGON>() as u32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SetLogonNotifyWindow(Wnd: HWND) -> BOOL {
    NtUserSetLogonNotifyWindow(Wnd)
}

#[no_mangle]
pub unsafe extern "system" fn UpdatePerUserSystemParameters(
    dwReserved: DWORD,
    bEnable: BOOL,
) -> BOOL {

    UpdatePerUserImmEnabling();

    CliImmInitializeHotKeys(SETIMEHOTKEY_INITIALIZE, core::ptr::null_mut());

    IntLoadPreloadKeyboardLayouts();

    NtUserUpdatePerUserSystemParameters(dwReserved, bEnable)
}
