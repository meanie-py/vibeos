use crate::cmd::console::ConOutResPaging;
use crate::cmd::cmd::ConvertULargeInteger;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope type, constant, or table declarations in this preamble. The only top-level item is the `CommandMemory` function, which is excluded here.

#[no_mangle]
pub unsafe extern "C" fn CommandMemory(param: LPTSTR) -> INT {
    let mut msex: MEMORYSTATUSEX = core::mem::zeroed();
    let mut szMemoryLoad: [TCHAR; 20] = [0; 20];
    let mut szTotalPhys: [TCHAR; 40] = [0; 40];
    let mut szAvailPhys: [TCHAR; 40] = [0; 40];
    let mut szTotalPageFile: [TCHAR; 40] = [0; 40];
    let mut szAvailPageFile: [TCHAR; 40] = [0; 40];
    let mut szTotalVirtual: [TCHAR; 40] = [0; 40];
    let mut szAvailVirtual: [TCHAR; 40] = [0; 40];
    let mut GlobalMemoryStatusEx: Option<unsafe extern "system" fn(LPMEMORYSTATUSEX) -> BOOL> = None;

    if _tcsncmp(
        param,
        _T(&['/' as u16, '?' as u16, 0u16] as *const u16),
        2,
    ) == 0
    {
        ConOutResPaging(TRUE, STRING_MEMORY_HELP1);
        return 0;
    }

    GlobalMemoryStatusEx = core::mem::transmute::<
        FARPROC,
        Option<unsafe extern "system" fn(LPMEMORYSTATUSEX) -> BOOL>,
    >(GetProcAddress(
        GetModuleHandle(_T(&[
            'K' as u16,
            'E' as u16,
            'R' as u16,
            'N' as u16,
            'E' as u16,
            'L' as u16,
            '3' as u16,
            '2' as u16,
            0u16,
        ] as *const u16)) as HINSTANCE,
        b"GlobalMemoryStatusEx\0".as_ptr() as *const i8,
    ));

    if let Some(GlobalMemoryStatusEx) = GlobalMemoryStatusEx {
        msex.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as DWORD;
        GlobalMemoryStatusEx(&raw mut msex);
    } else {
        let mut ms: MEMORYSTATUS = core::mem::zeroed();
        ms.dwLength = core::mem::size_of::<MEMORYSTATUS>() as DWORD;
        GlobalMemoryStatus(&raw mut ms);
        msex.dwMemoryLoad = ms.dwMemoryLoad;
        msex.ullTotalPhys = ms.dwTotalPhys as DWORDLONG;
        msex.ullAvailPhys = ms.dwAvailPhys as DWORDLONG;
        msex.ullTotalPageFile = ms.dwTotalPageFile as DWORDLONG;
        msex.ullAvailPageFile = ms.dwAvailPageFile as DWORDLONG;
        msex.ullTotalVirtual = ms.dwTotalVirtual as DWORDLONG;
        msex.ullAvailVirtual = ms.dwAvailVirtual as DWORDLONG;
    }

    ConvertULargeInteger(
        msex.dwMemoryLoad as ULONGLONG,
        &raw mut szMemoryLoad as LPTSTR,
        20,
        FALSE,
    );
    ConvertULargeInteger(
        msex.ullTotalPhys as ULONGLONG,
        &raw mut szTotalPhys as LPTSTR,
        40,
        TRUE,
    );
    ConvertULargeInteger(
        msex.ullAvailPhys as ULONGLONG,
        &raw mut szAvailPhys as LPTSTR,
        40,
        TRUE,
    );
    ConvertULargeInteger(
        msex.ullTotalPageFile as ULONGLONG,
        &raw mut szTotalPageFile as LPTSTR,
        40,
        TRUE,
    );
    ConvertULargeInteger(
        msex.ullAvailPageFile as ULONGLONG,
        &raw mut szAvailPageFile as LPTSTR,
        40,
        TRUE,
    );
    ConvertULargeInteger(
        msex.ullTotalVirtual as ULONGLONG,
        &raw mut szTotalVirtual as LPTSTR,
        40,
        TRUE,
    );
    ConvertULargeInteger(
        msex.ullAvailVirtual as ULONGLONG,
        &raw mut szAvailVirtual as LPTSTR,
        40,
        TRUE,
    );

    ConOutResPrintf(
        STRING_MEMORY_HELP2,
        &raw mut szMemoryLoad as LPTSTR,
        &raw mut szTotalPhys as LPTSTR,
        &raw mut szAvailPhys as LPTSTR,
        &raw mut szTotalPageFile as LPTSTR,
        &raw mut szAvailPageFile as LPTSTR,
        &raw mut szTotalVirtual as LPTSTR,
        &raw mut szAvailVirtual as LPTSTR,
    );

    return 0;
}
