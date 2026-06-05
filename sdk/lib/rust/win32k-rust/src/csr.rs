use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ObReferenceObjectByHandle, LpcRequestPort, LpcRequestWaitReplyPort};
use ntkrnl_sys::ob::{ObReferenceObject, ObDereferenceObject};
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::constants::STATUS_INVALID_PORT_HANDLE;
use core::ffi::c_void;

// RIT / desktop / ghost thread mains are all NTAPI (VOID NTAPI X(VOID)).
extern "system" {
    fn RawInputThreadMain();
    fn DesktopThreadMain();
    fn UserGhostThreadEntry();
}

// LpcPortObjectType is a DATA export from ntoskrnl (ntoskrnl.spec `@ extern`).
// Per-arch cross-DLL import (same shape as ExEventObjectType, [[feedback_
// exeventobjecttype_amd64_ptr_to_ptr]]):
//  - i386/arm64: raw-dylib +verbatim makes the __imp_ thunk; the symbol IS the
//    POBJECT_TYPE value → use directly.
//  - amd64: bind to the __imp_ thunk explicitly (ptr-to-ptr) and deref once. A
//    plain extern here silently resolves to garbage → ObReferenceObjectByHandle
//    returns STATUS_OBJECT_TYPE_MISMATCH (0xc0000024) and InitCsrApiPort fails.
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut LpcPortObjectType: *mut _OBJECT_TYPE;
}
#[cfg(target_arch = "x86_64")]
extern "C" {
    #[link_name = "__imp_LpcPortObjectType"]
    static mut LpcPortObjectType_imp: *mut *mut _OBJECT_TYPE;
}
#[cfg(any(target_arch = "x86", target_arch = "aarch64"))]
#[inline(always)]
unsafe fn lpc_port_object_type() -> PVOID {
    LpcPortObjectType as PVOID
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn lpc_port_object_type() -> PVOID {
    *LpcPortObjectType_imp as PVOID
}

// CSR_CREATE_API_NUMBER(ServerId, ApiId) = (ServerId << 16) | ApiId
#[inline(always)]
fn CSR_CREATE_API_NUMBER(server_id: u32, api_id: u32) -> CSR_API_NUMBER {
    ((server_id << 16) | api_id) as CSR_API_NUMBER
}

// csr.h externs these data globals (gpepCSRSS read by console.rs/ntuser.rs etc.).
#[no_mangle] pub static mut gpepCSRSS: PEPROCESS = core::ptr::null_mut();
#[no_mangle] pub static mut CsrApiPort: PVOID = core::ptr::null_mut();
#[no_mangle] pub static mut gdwPendingSystemThreads: DWORD = 0;

// All csr.c functions are cdecl (no NTAPI) EXCEPT CsrClientCallServer (NTAPI).
#[no_mangle]
pub unsafe extern "C" fn InitCsrProcess() {
    gpepCSRSS = PsGetCurrentProcess() as PEPROCESS;
    ObReferenceObject(gpepCSRSS as PVOID);
}

#[no_mangle]
pub unsafe extern "C" fn ResetCsrProcess() {
    if !gpepCSRSS.is_null() {
        ObDereferenceObject(gpepCSRSS as *mut _);
    }
    gpepCSRSS = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn InitCsrApiPort(CsrPortHandle: HANDLE) -> NTSTATUS {
    let Status = ObReferenceObjectByHandle(
        CsrPortHandle,
        0,
        lpc_port_object_type() as _,
        1i8, // UserMode
        core::ptr::addr_of_mut!(CsrApiPort),
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        CsrApiPort = core::ptr::null_mut();
    }
    Status
}

#[no_mangle]
pub unsafe extern "C" fn ResetCsrApiPort() {
    if !CsrApiPort.is_null() {
        ObDereferenceObject(CsrApiPort);
    }
    CsrApiPort = core::ptr::null_mut();
}

// Copied from ntdll csr/connect.c::CsrClientCallServer, adapted for kernel-mode.
// The CaptureBuffer pointer-fixup blocks are #if 0 in the C source, so dropped.
// NOTE: this is really a co_* function (UserLeaveCo/UserEnterCo around the LPC).
#[no_mangle]
pub unsafe extern "system" fn CsrClientCallServer(
    ApiMessage: PCSR_API_MESSAGE,
    _CaptureBuffer: PCSR_CAPTURE_BUFFER,
    ApiNumber: CSR_API_NUMBER,
    DataLength: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;

    /* Do we have a connection to CSR yet? */
    if CsrApiPort.is_null() {
        return STATUS_INVALID_PORT_HANDLE;
    }

    /* Fill out the Port Message Header (u1.s1 = CSHORT pair, u2.ZeroInit = ULONG) */
    (*ApiMessage).Header.u2.ZeroInit = 0;
    let total = core::mem::offset_of!(_CSR_API_MESSAGE, __bindgen_anon_1.__bindgen_anon_1.Data) as u32
        + DataLength;
    (*ApiMessage).Header.u1.s1.TotalLength = total as CSHORT;
    (*ApiMessage).Header.u1.s1.DataLength =
        (total - core::mem::size_of::<PORT_MESSAGE>() as u32) as CSHORT;

    /* Fill out the CSR Header (these live in the CSR_API_MESSAGE union tail) */
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.ApiNumber = ApiNumber;
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.CsrCaptureData = core::ptr::null_mut();

    UserLeave(); // UserLeaveCo

    /* Send the LPC Message (cross-crate PPORT_MESSAGE → `as _`) */
    let pmsg = core::ptr::addr_of_mut!((*ApiMessage).Header) as _;
    if PsGetCurrentProcess() as *mut c_void == gpepCSRSS as *mut c_void {
        Status = LpcRequestPort(CsrApiPort, pmsg);
    } else {
        Status = LpcRequestWaitReplyPort(CsrApiPort, pmsg, pmsg);
    }

    UserEnterExclusive(); // UserEnterCo

    if !NT_SUCCESS(Status) {
        /* We failed. Overwrite the return value with the failure. */
        (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status = Status;
    }

    /* Return the CSR Result */
    (*ApiMessage).__bindgen_anon_1.__bindgen_anon_1.Status
}

#[no_mangle]
pub unsafe extern "C" fn UserSystemThreadProc(_bRemoteProcess: BOOL) -> DWORD {
    let Type: DWORD;

    if gdwPendingSystemThreads == 0 {
        return 0;
    }

    /* Decide which thread this will be */
    if gdwPendingSystemThreads & ST_RIT != 0 {
        Type = ST_RIT;
    } else if gdwPendingSystemThreads & ST_DESKTOP_THREAD != 0 {
        Type = ST_DESKTOP_THREAD;
    } else {
        Type = ST_GHOST_THREAD;
    }

    /* Unmark it as pending */
    gdwPendingSystemThreads &= !Type;

    UserLeave();

    // ST_* are bindgen consts → `==` comparisons, not match arms (binding trap).
    if Type == ST_RIT {
        RawInputThreadMain();
    } else if Type == ST_DESKTOP_THREAD {
        DesktopThreadMain();
    } else if Type == ST_GHOST_THREAD {
        UserGhostThreadEntry();
    }

    UserEnterShared();

    0
}

#[no_mangle]
pub unsafe extern "C" fn UserCreateSystemThread(Type: DWORD) -> BOOL {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let pCreateThreadRequest: PUSER_CREATE_SYSTEM_THREAD =
        core::ptr::addr_of_mut!(ApiMessage.Data.CreateSystemThreadRequest);

    if (gdwPendingSystemThreads & Type) != 0 {
        return TRUE as BOOL;
    }

    /* Mark what the new thread needs to do */
    gdwPendingSystemThreads |= Type;

    /* Ask winsrv to create a new system thread (re-enters win32k via
     * UserSystemThreadProc). */
    (*pCreateThreadRequest).bRemote = FALSE as _;
    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpCreateSystemThreads),
        core::mem::size_of::<USER_CREATE_SYSTEM_THREAD>() as ULONG,
    );

    // USER_API_MESSAGE.Status is a direct field (NOT the CSR_API_MESSAGE union tail).
    if !NT_SUCCESS(ApiMessage.Status) {
        return FALSE as BOOL;
    }

    TRUE as BOOL
}
