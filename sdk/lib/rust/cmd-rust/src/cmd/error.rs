use ntdll_sys::nt::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn ErrorMessage(dwErrorCode: DWORD, szFormat: PCWSTR, mut args: ...) {
    let mut arg_ptr: va_list;
    let mut szError: PWSTR = core::ptr::null_mut();
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];
    let mut szMessage: [TCHAR; 1024] = [0; 1024];

    if dwErrorCode == ERROR_SUCCESS as DWORD {
        return;
    }

    *(core::ptr::addr_of_mut!(szMessage) as *mut TCHAR) = 0 as TCHAR;
    if !szFormat.is_null() {
        va_start(arg_ptr, szFormat);
        _vswprintf(core::ptr::addr_of_mut!(szMessage) as *mut TCHAR, szFormat, arg_ptr);
        va_end(arg_ptr);
    }

    if FormatMessage(
        (FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_ALLOCATE_BUFFER) as DWORD,
        core::ptr::null(),
        dwErrorCode,
        MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
        core::ptr::addr_of_mut!(szError) as PTSTR,
        0 as DWORD,
        core::ptr::null_mut(),
    ) != 0
    {
        ConErrPrintf(
            _T("%s%s%s"),
            szError,
            core::ptr::addr_of_mut!(szMessage) as *mut TCHAR,
            if *(core::ptr::addr_of_mut!(szMessage) as *mut TCHAR) != 0 {
                _T("\n")
            } else {
                _T("")
            },
        );
        if !szError.is_null() {
            LocalFree(szError as HLOCAL);
        }
        return;
    }

    /* Fall back just in case the error is not defined */
    LoadString(
        CMD_ModuleHandle,
        STRING_CONSOLE_ERROR as UINT,
        core::ptr::addr_of_mut!(szMsg) as *mut TCHAR,
        ARRAYSIZE(szMsg) as INT,
    );
    if !szFormat.is_null() {
        ConErrPrintf(
            _T("%s -- %s\n"),
            core::ptr::addr_of_mut!(szMsg) as *mut TCHAR,
            core::ptr::addr_of_mut!(szMessage) as *mut TCHAR,
        );
    } else {
        ConErrPrintf(
            _T("%s\n"),
            core::ptr::addr_of_mut!(szMsg) as *mut TCHAR,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn error_parameter_format(ch: WCHAR) {
    ConErrResPrintf(STRING_ERROR_PARAMETERF_ERROR, ch);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_invalid_switch(mut ch: WCHAR) {
    ConErrResPrintf(STRING_ERROR_INVALID_SWITCH, ch);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_too_many_parameters(s: PCWSTR) {
    ConErrResPrintf(STRING_ERROR_TOO_MANY_PARAMETERS, s);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_path_not_found() {
    ConErrResPuts(STRING_ERROR_PATH_NOT_FOUND);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_file_not_found() {
    ConErrResPuts(STRING_ERROR_FILE_NOT_FOUND);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_sfile_not_found(s: PCWSTR) {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];

    LoadString(CMD_ModuleHandle, STRING_ERROR_FILE_NOT_FOUND, szMsg.as_mut_ptr(), ARRAYSIZE(szMsg));
    let fmt: [u16; 9] = ['%' as u16, 's' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
    ConErrPrintf(fmt.as_ptr(), szMsg.as_mut_ptr(), s);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_req_param_missing() {
    ConErrResPuts(STRING_ERROR_REQ_PARAM_MISSING);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_invalid_drive() {
    ConErrResPuts(STRING_ERROR_INVALID_DRIVE);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_bad_command(mut s: PCWSTR) {
    ConErrResPrintf(STRING_ERROR_BADCOMMAND, s);
    nErrorLevel = 9009;
}
#[no_mangle]
pub unsafe extern "C" fn error_no_pipe() {
    ConErrResPuts(STRING_ERROR_CANNOTPIPE);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_out_of_memory() {
    ConErrResPuts(STRING_ERROR_OUT_OF_MEMORY);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_cant_exec_program() {
    ErrorMessage(ERROR_NO_ASSOCIATION as DWORD, ::core::ptr::null::<WCHAR>());
    nErrorLevel = 1 as INT;
}
#[no_mangle]
pub unsafe extern "C" fn error_invalid_parameter_format(s: PCWSTR) {
    ConErrResPrintf(STRING_ERROR_INVALID_PARAM_FORMAT, s);
    nErrorLevel = 1;
}
#[no_mangle]
pub unsafe extern "C" fn error_syntax(s: PCWSTR) {
    let mut szMsg: [WCHAR; RC_STRING_MAX_SIZE] = [0; RC_STRING_MAX_SIZE];

    LoadString(CMD_ModuleHandle as HINSTANCE, STRING_ERROR_ERROR2 as UINT, szMsg.as_mut_ptr(), szMsg.len() as INT);

    if !s.is_null() {
        let fmt1: [u16; 9] = ['%' as u16, 's' as u16, ' ' as u16, '-' as u16, ' ' as u16, '%' as u16, 's' as u16, '\n' as u16, 0];
        ConErrPrintf(fmt1.as_ptr(), szMsg.as_ptr(), s);
    } else {
        let fmt2: [u16; 5] = ['%' as u16, 's' as u16, '.' as u16, '\n' as u16, 0];
        ConErrPrintf(fmt2.as_ptr(), szMsg.as_ptr());
    }

    nErrorLevel = 1 as INT;
}
#[no_mangle]
pub unsafe extern "C" fn msg_pause() {
    ConOutResPuts(STRING_ERROR_D_PAUSEMSG);
}
