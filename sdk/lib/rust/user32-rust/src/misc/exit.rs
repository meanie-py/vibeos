use win32k_sys::win32k::*;
use core::ffi::c_void;

// ---- substrate glue (added for compile; mirrors misc/logon.rs + misc/desktop.rs) ----

// ntstatus.h / winbase.h constants not present in the win32k bindgen surface.
const STATUS_CANT_WAIT: NTSTATUS = 0xC00000D8u32 as i32;
const INFINITE: DWORD = 0xFFFFFFFF;

// ntdll.h: `NTSTATUS NT_SUCCESS(NTSTATUS) := (Status >= 0)`.
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// csrmsg.h: `CSR_CREATE_API_NUMBER(ServerId, ApiId) := (CSR_API_NUMBER)((ServerId<<16)|ApiId)`.
#[inline]
fn CSR_CREATE_API_NUMBER(ServerId: u32, ApiId: u32) -> CSR_API_NUMBER {
    ((ServerId << 16) | ApiId) as CSR_API_NUMBER
}

extern "system" {
    // ntdll export (NTAPI): CSR client transport.
    fn CsrClientCallServer(
        ApiMessage: PCSR_API_MESSAGE,
        CaptureBuffer: PCSR_CAPTURE_BUFFER,
        ApiNumber: CSR_API_NUMBER,
        DataLength: ULONG,
    ) -> NTSTATUS;

    // user32's OWN internal helpers (misc/misc.c), resolved at link to the crate's
    // own `#[no_mangle]`/local symbols.
    fn UserSetLastNTError(Status: NTSTATUS);
    fn UserSetLastError(dwErrCode: DWORD);

    // kernel32 imports.
    fn GetLastError() -> DWORD;
    fn ExitThread(dwExitCode: DWORD) -> !;
    fn CreateThread(
        lpThreadAttributes: LPVOID,
        dwStackSize: SIZE_T,
        lpStartAddress: unsafe extern "system" fn(LPVOID) -> DWORD,
        lpParameter: LPVOID,
        dwCreationFlags: DWORD,
        lpThreadId: LPDWORD,
    ) -> HANDLE;
    fn GetExitCodeThread(hThread: HANDLE, lpExitCode: LPDWORD) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;

    // user32 public exports referenced here (sibling modules of this DLL).
    fn MsgWaitForMultipleObjectsEx(
        nCount: DWORD,
        pHandles: *const HANDLE,
        dwMilliseconds: DWORD,
        dwWakeMask: DWORD,
        dwFlags: DWORD,
    ) -> DWORD;
    fn PeekMessageW(
        lpMsg: LPMSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
        wRemoveMsg: UINT,
    ) -> BOOL;
    fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct EXIT_REACTOS_DATA {
    pub uFlags: UINT,
    pub dwReserved: DWORD,
}
pub type PEXIT_REACTOS_DATA = *mut EXIT_REACTOS_DATA;

unsafe extern "system" fn ExitWindowsThread(Param: LPVOID) -> DWORD {
    let dwExitCode: DWORD;
    let ExitData = Param as PEXIT_REACTOS_DATA;

    if ExitWindowsWorker((*ExitData).uFlags, (*ExitData).dwReserved, 1) != 0 {
        dwExitCode = ERROR_SUCCESS;
    } else {
        dwExitCode = GetLastError();
    }

    ExitThread(dwExitCode);
}

unsafe fn ExitWindowsWorker(uFlags: UINT,
                            dwReserved: DWORD,
                            bCalledFromThread: BOOL) -> BOOL
{
    let mut ExitData: EXIT_REACTOS_DATA = core::mem::zeroed();
    let hExitThread: HANDLE;
    let mut ExitCode: DWORD;
    let mut msg: MSG = core::mem::zeroed();

    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let ExitReactOSRequest: PUSER_EXIT_REACTOS = core::ptr::addr_of_mut!(ApiMessage.Data.ExitReactOSRequest);

    (*ExitReactOSRequest).Flags = uFlags;

    CsrClientCallServer(core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
                        core::ptr::null_mut(),
                        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpExitWindowsEx),
                        core::mem::size_of::<USER_EXIT_REACTOS>() as ULONG);

    if NT_SUCCESS(ApiMessage.Status) || ApiMessage.Status == STATUS_CANT_WAIT
    {
        if (*ExitReactOSRequest).LastError != ERROR_SUCCESS
        {
            UserSetLastError((*ExitReactOSRequest).LastError);
        }
    }
    else
    {
        UserSetLastNTError(ApiMessage.Status);
        (*ExitReactOSRequest).Success = 0;
    }

    if NT_SUCCESS(ApiMessage.Status)
    {
        return (*ExitReactOSRequest).Success;
    }

    if ApiMessage.Status != STATUS_CANT_WAIT || bCalledFromThread != 0
    {
        UserSetLastNTError(ApiMessage.Status);
        return 0;
    }

    ExitData.uFlags     = uFlags;
    ExitData.dwReserved = dwReserved;
    hExitThread = CreateThread(core::ptr::null_mut(),
                               0,
                               ExitWindowsThread,
                               core::ptr::addr_of_mut!(ExitData) as LPVOID,
                               0,
                               core::ptr::null_mut());
    if hExitThread.is_null()
    {
        return 0;
    }

    while MsgWaitForMultipleObjectsEx(1,
                                      core::ptr::addr_of!(hExitThread) as *const HANDLE,
                                      INFINITE,
                                      QS_ALLINPUT,
                                      0) != 0
    {
        while PeekMessageW(core::ptr::addr_of_mut!(msg),
                           core::ptr::null_mut(),
                           0,
                           0,
                           PM_REMOVE) != 0
        {
            DispatchMessageW(core::ptr::addr_of_mut!(msg));
        }
    }

    ExitCode = 0;
    if GetExitCodeThread(hExitThread, core::ptr::addr_of_mut!(ExitCode)) == 0
    {
        ExitCode = GetLastError();
    }

    CloseHandle(hExitThread);

    if ExitCode != ERROR_SUCCESS
    {
        UserSetLastError(ExitCode);
    }

    (ExitCode == ERROR_SUCCESS) as BOOL
}

#[no_mangle]
pub unsafe extern "system" fn ExitWindowsEx(uFlags: UINT,
                                            dwReserved: DWORD) -> BOOL {
    ExitWindowsWorker(uFlags, dwReserved, 0)
}

#[no_mangle]
pub unsafe extern "system" fn EndTask(hWnd: HWND, fShutDown: BOOL, fForce: BOOL) -> BOOL {
    let mut ApiMessage: USER_API_MESSAGE = core::mem::zeroed();
    let EndTaskRequest: PUSER_END_TASK =
        core::ptr::addr_of_mut!(ApiMessage.Data.EndTaskRequest);

    let _ = fShutDown;

    (*EndTaskRequest).WndHandle = hWnd;
    (*EndTaskRequest).Force = fForce;

    CsrClientCallServer(
        core::ptr::addr_of_mut!(ApiMessage) as PCSR_API_MESSAGE,
        core::ptr::null_mut(),
        CSR_CREATE_API_NUMBER(USERSRV_SERVERDLL_INDEX, UserpEndTask),
        core::mem::size_of::<_USER_END_TASK>() as u32,
    );

    if !NT_SUCCESS(ApiMessage.Status) {
        UserSetLastNTError(ApiMessage.Status);
        return 0;
    }

    if (*EndTaskRequest).LastError != ERROR_SUCCESS {
        UserSetLastError((*EndTaskRequest).LastError);
    }

    (*EndTaskRequest).Success
}
