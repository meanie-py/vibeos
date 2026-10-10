use crate::cmd::console::ConOutResPaging;
use crate::cmd::cmd::ConvertULargeInteger;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope typedefs, object-like `#define` constants, enums, or data tables appear in this preamble. The only declarations are the functions `PrintDiskInfo` and `CommandFree`, which are excluded from this pass.

unsafe extern "C" fn PrintDiskInfo(szDisk: LPTSTR) {
    let mut szMsg: [TCHAR; RC_STRING_MAX_SIZE] = core::mem::zeroed();
    let mut szRootPath: [TCHAR; 4] = _T("A:\\");
    let mut szDrive: [TCHAR; 2] = _T("A");
    let mut szVolume: [TCHAR; 64] = core::mem::zeroed();
    let mut szSerial: [TCHAR; 10] = core::mem::zeroed();
    let mut szTotal: [TCHAR; 40] = core::mem::zeroed();
    let mut szUsed: [TCHAR; 40] = core::mem::zeroed();
    let mut szFree: [TCHAR; 40] = core::mem::zeroed();
    let mut dwSerial: DWORD = 0;
    let mut uliSize: ULONGLONG = 0;
    let mut dwSecPerCl: DWORD = 0;
    let mut dwBytPerSec: DWORD = 0;
    let mut dwFreeCl: DWORD = 0;
    let mut dwTotCl: DWORD = 0;

    if _tcslen(szDisk) < 2
        || *szDisk.add(1) != _T(':')
    {
        ConErrResPrintf(STRING_FREE_ERROR1);
        return;
    }

    szRootPath[0] = *szDisk.add(0);
    szDrive[0] = _totupper(szRootPath[0]);

    if !GetVolumeInformation(
        addr_of_mut!(szRootPath) as LPTSTR,
        addr_of_mut!(szVolume) as LPTSTR,
        64,
        core::ptr::addr_of_mut!(dwSerial),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    ) != 0
    {
        LoadString(
            CMD_ModuleHandle,
            STRING_FREE_ERROR1,
            addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg),
        );
        ConErrPrintf(_T("%s %s:\n"), addr_of_mut!(szMsg) as LPTSTR, addr_of_mut!(szDrive) as LPTSTR);
        return;
    }

    if szVolume[0] == _T('\0') {
        LoadString(
            CMD_ModuleHandle,
            STRING_FREE_ERROR2,
            addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg),
        );
        _tcscpy(addr_of_mut!(szVolume) as LPTSTR, addr_of_mut!(szMsg) as LPTSTR);
    }

    _stprintf(
        addr_of_mut!(szSerial) as LPTSTR,
        _T("%04X-%04X"),
        HIWORD(dwSerial),
        LOWORD(dwSerial),
    );

    if GetDiskFreeSpace(
        addr_of_mut!(szRootPath) as LPTSTR,
        core::ptr::addr_of_mut!(dwSecPerCl),
        core::ptr::addr_of_mut!(dwBytPerSec),
        core::ptr::addr_of_mut!(dwFreeCl),
        core::ptr::addr_of_mut!(dwTotCl),
    ) == 0
    {
        LoadString(
            CMD_ModuleHandle,
            STRING_FREE_ERROR1,
            addr_of_mut!(szMsg) as LPTSTR,
            ARRAYSIZE(szMsg),
        );
        ConErrPrintf(_T("%s %s:\n"), addr_of_mut!(szMsg) as LPTSTR, addr_of_mut!(szDrive) as LPTSTR);
        return;
    }

    uliSize = (dwSecPerCl.wrapping_mul(dwBytPerSec) as ULONGLONG).wrapping_mul(dwTotCl as ULONGLONG);
    ConvertULargeInteger(
        uliSize,
        addr_of_mut!(szTotal) as LPTSTR,
        40,
        TRUE,
    );

    uliSize = (dwSecPerCl.wrapping_mul(dwBytPerSec) as ULONGLONG)
        .wrapping_mul(dwTotCl.wrapping_sub(dwFreeCl) as ULONGLONG);
    ConvertULargeInteger(
        uliSize,
        addr_of_mut!(szUsed) as LPTSTR,
        40,
        TRUE,
    );

    uliSize = (dwSecPerCl.wrapping_mul(dwBytPerSec) as ULONGLONG).wrapping_mul(dwFreeCl as ULONGLONG);
    ConvertULargeInteger(
        uliSize,
        addr_of_mut!(szFree) as LPTSTR,
        40,
        TRUE,
    );

    ConOutResPrintf(
        STRING_FREE_HELP1,
        addr_of_mut!(szDrive) as LPTSTR,
        addr_of_mut!(szVolume) as LPTSTR,
        addr_of_mut!(szSerial) as LPTSTR,
        addr_of_mut!(szTotal) as LPTSTR,
        addr_of_mut!(szUsed) as LPTSTR,
        addr_of_mut!(szFree) as LPTSTR,
    );
}
#[no_mangle]
pub unsafe extern "C" fn CommandFree(param: LPTSTR) -> INT {
    let mut szParam: LPTSTR = core::ptr::null_mut();
    let mut szDefPath: [TCHAR; MAX_PATH] = [0; MAX_PATH];
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut arg: *mut LPTSTR = core::ptr::null_mut();

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPaging(TRUE, STRING_FREE_HELP2);
        return 0;
    }

    if param.is_null() || *param == _T('\0') {
        GetCurrentDirectory(MAX_PATH, szDefPath.as_mut_ptr());
        szDefPath[2] = _T('\0');
        szParam = szDefPath.as_mut_ptr();
    } else {
        szParam = param;
    }

    arg = split(
        szParam,
        core::ptr::addr_of_mut!(argc),
        FALSE,
        FALSE,
    );

    i = 0;
    while i < argc {
        PrintDiskInfo(*arg.offset(i as isize));
        i += 1;
    }

    freep(arg);

    return 0;
}
