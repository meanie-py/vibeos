use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;

#[no_mangle]
pub unsafe extern "system" fn SrvRegisterConsoleVDM(
    ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    let RegisterVDMRequest = &mut (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.RegisterVDMRequest;
    let Console = (*(ApiMessage as *mut _CONSOLE_API_MESSAGE)).Data.RegisterVDMRequest.ConsoleHandle as PCONSRV_CONSOLE;

    if RegisterVDMRequest.RegisterFlags != 0 {
        let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
        let mut ViewSize: SIZE_T = 0;
        let ProcessHandle: HANDLE;

        ProcessHandle = (*(*CsrGetClientThread()).Process).ProcessHandle;
        (*Console).VDMClientProcess = ProcessHandle;

        (*Console).VDMBufferSize = RegisterVDMRequest.VDMBufferSize;

        let Size = ((*Console).VDMBufferSize.X as usize) * ((*Console).VDMBufferSize.Y as usize)
            * core::mem::size_of::<CHAR_CELL>();

        SectionSize.QuadPart = Size as i64;
        let mut Status = NtCreateSection(
            core::ptr::addr_of_mut!((*Console).VDMBufferSection),
            SECTION_ALL_ACCESS,
            core::ptr::null_mut(),
            &mut SectionSize,
            PAGE_READWRITE,
            SEC_COMMIT,
            core::ptr::null_mut(),
        );
        if Status < 0 {
            return Status;
        }

        ViewSize = 0;
        (*Console).VDMBuffer = core::ptr::null_mut();
        Status = NtMapViewOfSection(
            (*Console).VDMBufferSection,
            NtCurrentProcess(),
            core::ptr::addr_of_mut!((*Console).VDMBuffer) as *mut PVOID,
            0,
            0,
            core::ptr::null_mut(),
            &mut ViewSize,
            ViewUnmap,
            0,
            PAGE_READWRITE,
        );
        if Status < 0 {
            NtClose((*Console).VDMBufferSection);
            return Status;
        }

        ViewSize = 0;
        (*Console).ClientVDMBuffer = core::ptr::null_mut();
        Status = NtMapViewOfSection(
            (*Console).VDMBufferSection,
            ProcessHandle,
            core::ptr::addr_of_mut!((*Console).ClientVDMBuffer) as *mut PVOID,
            0,
            0,
            core::ptr::null_mut(),
            &mut ViewSize,
            ViewUnmap,
            0,
            PAGE_READWRITE,
        );
        if Status < 0 {
            NtUnmapViewOfSection(NtCurrentProcess(), (*Console).VDMBuffer);
            NtClose((*Console).VDMBufferSection);
            return Status;
        }

        RegisterVDMRequest.VDMBuffer = (*Console).ClientVDMBuffer;

        STATUS_SUCCESS
    } else {
        if !(*Console).VDMBuffer.is_null() {
            NtUnmapViewOfSection((*Console).VDMClientProcess, (*Console).ClientVDMBuffer);
            NtUnmapViewOfSection(NtCurrentProcess(), (*Console).VDMBuffer);
            NtClose((*Console).VDMBufferSection);
        }
        (*Console).VDMBuffer = core::ptr::null_mut();
        (*Console).ClientVDMBuffer = core::ptr::null_mut();

        (*Console).VDMBufferSize.X = 0;
        (*Console).VDMBufferSize.Y = 0;

        STATUS_SUCCESS
    }
}

#[no_mangle]
pub unsafe extern "system" fn SrvVDMConsoleOperation(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvRegisterConsoleOS2(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvSetConsoleOS2OemFormat(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvRegisterConsoleIME(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn SrvUnregisterConsoleIME(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: PCSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}