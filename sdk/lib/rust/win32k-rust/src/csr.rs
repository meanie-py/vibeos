use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ObReferenceObjectByHandle, Type};
use core::ffi::c_void;

extern "C" {
    fn RawInputThreadMain();
    fn DesktopThreadMain();
}

// ---- file-scope preamble (translated) ----
pub static mut gpepCSRSS: PEPROCESS = core::ptr::null_mut();
pub static mut CsrApiPort: PVOID = core::ptr::null_mut();
pub static mut gdwPendingSystemThreads: DWORD = 0;

unsafe fn InitCsrProcess() {
    gpepCSRSS = PsGetCurrentProcess();
    ObReferenceObject(gpepCSRSS as PVOID);
}

unsafe fn ResetCsrProcess() {
    if !gpepCSRSS.is_null() {
        ObDereferenceObject(gpepCSRSS as *mut _);
    }

    gpepCSRSS = core::ptr::null_mut();
}

unsafe fn InitCsrApiPort(CsrPortHandle: HANDLE) -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = ObReferenceObjectByHandle(
        CsrPortHandle,
        0,
        LpcPortObjectType,
        1i32 as i8,
        core::ptr::addr_of_mut!(CsrApiPort),
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        CsrApiPort = core::ptr::null_mut();
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ResetCsrApiPort() {
    if !CsrApiPort.is_null() {
        ObDereferenceObject(CsrApiPort);
    }

    CsrApiPort = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn UserSystemThreadProc(_bRemoteProcess: BOOL) -> DWORD {
    let Type: DWORD;

    if gdwPendingSystemThreads == 0 {
        return 0;
    }

    if gdwPendingSystemThreads & ST_RIT != 0 {
        Type = ST_RIT;
    } else if gdwPendingSystemThreads & ST_DESKTOP_THREAD != 0 {
        Type = ST_DESKTOP_THREAD;
    } else {
        Type = ST_GHOST_THREAD;
    }

    gdwPendingSystemThreads &= !Type;

    UserLeave();

    match Type {
        x if x == ST_RIT => {
            RawInputThreadMain();
        }
        x if x == ST_DESKTOP_THREAD => {
            DesktopThreadMain();
        }
        x if x == ST_GHOST_THREAD => {
            UserGhostThreadEntry();
        }
        _ => {}
    }

    UserEnterShared();

    0
}

#[no_mangle]
pub unsafe extern "system" fn UserCreateSystemThread(Type: DWORD) -> BOOL {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let pCreateThreadRequest: PUSER_CREATE_SYSTEM_THREAD =
        core::ptr::addr_of_mut!(ApiMessage.Data.CreateSystemThreadRequest);

    if (gdwPendingSystemThreads & Type) != 0 {
        return TRUE as BOOL;
    }

    /* We can't pass a parameter to the new thread so mark what the new thread needs to do */
    gdwPendingSystemThreads |= Type;

    /* Ask winsrv to create a new system thread. */
    (*pCreateThreadRequest).bRemote = FALSE as _;
    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpCreateSystemThreads),
        core::mem::size_of::<USER_CREATE_SYSTEM_THREAD>() as ULONG,
    );

    if !NT_SUCCESS(ApiMessage.__bindgen_anon_1.Status) {
        return FALSE as BOOL;
    }

    TRUE as BOOL
}
