use crate::cmd::console::ConOutResPaging;
use crate::cmd::misc::IsValidPathName;
use crate::cmd::error::error_invalid_drive;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, enum, or table declarations in this preamble. The only definitions are the functions `PrintVolumeHeader` and `cmd_vol`, which are excluded from this pass.

unsafe extern "C" fn PrintVolumeHeader(pszRootPath: LPTSTR) -> INT {
    let mut szVolName: [TCHAR; 80] = [0; 80];
    let mut dwSerialNr: DWORD = 0;

    /* get the volume information of the drive */
    if GetVolumeInformation(
        pszRootPath,
        core::ptr::addr_of_mut!(szVolName) as *mut TCHAR,
        80,
        core::ptr::addr_of_mut!(dwSerialNr),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    ) == 0
    {
        ErrorMessage(GetLastError(), core::ptr::null_mut());
        return 1;
    }

    /* print drive info */
    if szVolName[0] != '\0' as TCHAR {
        ConOutResPrintf!(
            STRING_VOL_HELP1,
            *pszRootPath.add(0) as core::ffi::c_uint,
            core::ptr::addr_of_mut!(szVolName) as *mut TCHAR,
        );
    } else {
        ConOutResPrintf!(
            STRING_VOL_HELP2,
            *pszRootPath.add(0) as core::ffi::c_uint,
        );
    }

    /* print the volume serial number */
    ConOutResPrintf!(
        STRING_VOL_HELP3,
        HIWORD!(dwSerialNr) as core::ffi::c_uint,
        LOWORD!(dwSerialNr) as core::ffi::c_uint
    );
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_vol(mut param: LPTSTR) -> INT {
    let mut szRootPath: [TCHAR; 4] = ['A' as TCHAR, ':' as TCHAR, '\\' as TCHAR, 0 as TCHAR];
    let mut szPath: [TCHAR; (MAX_PATH) as _] = [0 as TCHAR; (MAX_PATH) as _];
    let mut szSlashQuestion: [TCHAR; 3] = ['/' as TCHAR, '?' as TCHAR, 0 as TCHAR];

    if _tcsncmp(param, szSlashQuestion.as_ptr(), 2) == 0 {
        ConOutResPaging((TRUE) as BOOL, STRING_VOL_HELP4);
        return 0 as INT;
    }

    nErrorLevel = 0 as INT;

    if *param.add(0) == '\0' as TCHAR {
        GetCurrentDirectory(MAX_PATH, szPath.as_mut_ptr());
        szRootPath[0] = szPath[0];
    } else {
        _tcsupr(param);
        if *param.add(1) == ':' as TCHAR {
            szRootPath[0] = *param.add(0);
        } else {
            error_invalid_drive();
            nErrorLevel = 1 as INT;
            return 1 as INT;
        }
    }

    if IsValidPathName(szRootPath.as_ptr()) == 0 {
        error_invalid_drive();
        nErrorLevel = 1 as INT;
        return 1 as INT;
    }

    /* print the header */
    if PrintVolumeHeader(szRootPath.as_mut_ptr()) == 0 {
        nErrorLevel = 1 as INT;
        return 1 as INT;
    }

    return 0 as INT;
}
