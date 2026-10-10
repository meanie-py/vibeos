use crate::cmd::console::ConOutResPaging;
use crate::cmd::error::error_out_of_memory;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
/// Size of environment variable buffer
pub const ENV_BUFFER_SIZE: u32 = 1024;
#[no_mangle]
pub unsafe extern "C" fn cmd_path(mut param: LPTSTR) -> INT {
    let mut retval: INT = 0;
    'quit: {
        if _tcsncmp(param, _T("/?"), 2) == 0 {
            ConOutResPaging(TRUE, STRING_PATH_HELP1);
            return 0;
        }

        /* If param is empty, display the PATH environment variable */
        if param.is_null() || *param == 0 {
            let mut dwBuffer: DWORD;
            let mut pszBuffer: LPTSTR;

            pszBuffer = cmd_alloc(1024 /* ENV_BUFFER_SIZE */ * core::mem::size_of::<TCHAR>()) as LPTSTR;
            if pszBuffer.is_null() {
                error_out_of_memory();
                retval = 1;
                break 'quit;
            }

            dwBuffer = GetEnvironmentVariable(_T("PATH"), pszBuffer, ENV_BUFFER_SIZE);
            if dwBuffer == 0 {
                cmd_free(pszBuffer as *mut core::ffi::c_void);
                ConErrResPrintf(STRING_SET_ENV_ERROR, _T("PATH"));
                retval = 0;
                break 'quit;
            } else if dwBuffer > ENV_BUFFER_SIZE {
                let pszOldBuffer: LPTSTR = pszBuffer;
                pszBuffer = cmd_realloc(
                    pszBuffer as *mut core::ffi::c_void,
                    (dwBuffer as usize) * core::mem::size_of::<TCHAR>(),
                ) as LPTSTR;
                if pszBuffer.is_null() {
                    error_out_of_memory();
                    cmd_free(pszOldBuffer as *mut core::ffi::c_void);
                    retval = 1;
                    break 'quit;
                }
                GetEnvironmentVariable(_T("PATH"), pszBuffer, dwBuffer);
            }

            ConOutPrintf(_T("PATH=%s\n"), pszBuffer);
            cmd_free(pszBuffer as *mut core::ffi::c_void);

            retval = 0;
            break 'quit;
        }

        /* Skip leading '=' */
        if *param == _T('=') {
            param = param.offset(1);
        }

        /* Set PATH environment variable */
        if SetEnvironmentVariable(_T("PATH"), param) == 0 {
            retval = 1;
        }
    }

    // Quit:
    if BatType != CMD_TYPE {
        if retval != 0 {
            nErrorLevel = retval;
        }
    } else {
        nErrorLevel = retval;
    }

    return retval;
}
