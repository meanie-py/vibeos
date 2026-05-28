use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

static mut ServicesProcessIdValid: BOOLEAN = 0u8;
#[no_mangle]
pub static mut ServicesProcessId: ULONG_PTR = 0;
#[no_mangle]
pub static mut LogonProcessId: ULONG_PTR = 0;

extern "system" {
    pub static mut CsrApiPort: HANDLE;
    fn CsrQueryApiPort() -> HANDLE;
}

#[no_mangle]
pub unsafe extern "system" fn SrvRegisterLogonProcess(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyStatus: *mut ULONG,
) -> NTSTATUS {
    let RegisterLogonProcessRequest: PUSER_REGISTER_LOGON_PROCESS = 
        core::ptr::addr_of_mut!(
            (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.RegisterLogonProcessRequest
        );

    if (*RegisterLogonProcessRequest).Register != 0 {
        if LogonProcessId != 0 {
            return STATUS_LOGON_SESSION_EXISTS;
        }

        LogonProcessId = (*RegisterLogonProcessRequest).ProcessId;
    } else {
        if (*ApiMessage).Header.__bindgen_anon_1.ClientId.UniqueProcess !=
            UlongToHandle(LogonProcessId as ULONG)
        {
            return STATUS_NOT_LOGON_PROCESS;
        }

        LogonProcessId = 0;
    }

    if CsrApiPort.is_null() {
        CsrApiPort = CsrQueryApiPort();

        let Status = NtUserSetInformationThread(
            NtCurrentThread(),
            UserThreadCsrApiPort,
            core::ptr::addr_of!(CsrApiPort) as *mut core::ffi::c_void,
            core::mem::size_of::<HANDLE>() as ULONG,
        );

        if Status < 0 {
            return Status;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn SrvRegisterServicesProcess(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyStatus: *mut ULONG,
) -> NTSTATUS {
    let RegisterServicesProcessRequest: PUSER_REGISTER_SERVICES_PROCESS = 
        core::ptr::addr_of_mut!(
            (*(ApiMessage as *mut _USER_API_MESSAGE)).Data.RegisterServicesProcessRequest
        );

    if ServicesProcessIdValid != 0 {
        STATUS_INVALID_PARAMETER
    } else {
        ServicesProcessId = (*RegisterServicesProcessRequest).ProcessId;
        ServicesProcessIdValid = 1u8;
        STATUS_SUCCESS
    }
}