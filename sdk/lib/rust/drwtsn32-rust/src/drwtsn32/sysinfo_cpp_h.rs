use crate::prelude::*;
use core::ffi::c_void;

unsafe extern "C" fn Exception2Str(code: DWORD) -> *const i8 {
    match code {
        0xC0000005 /* EXCEPTION_ACCESS_VIOLATION */ => return b"EXCEPTION_ACCESS_VIOLATION\0".as_ptr() as *const i8,
        0x80000002 /* EXCEPTION_DATATYPE_MISALIGNMENT */ => return b"EXCEPTION_DATATYPE_MISALIGNMENT\0".as_ptr() as *const i8,
        0x80000003 /* EXCEPTION_BREAKPOINT */ => return b"EXCEPTION_BREAKPOINT\0".as_ptr() as *const i8,
        0x80000004 /* EXCEPTION_SINGLE_STEP */ => return b"EXCEPTION_SINGLE_STEP\0".as_ptr() as *const i8,
        0xC000008C /* EXCEPTION_ARRAY_BOUNDS_EXCEEDED */ => return b"EXCEPTION_ARRAY_BOUNDS_EXCEEDED\0".as_ptr() as *const i8,
        0xC000008D /* EXCEPTION_FLT_DENORMAL_OPERAND */ => return b"EXCEPTION_FLT_DENORMAL_OPERAND\0".as_ptr() as *const i8,
        0xC000008E /* EXCEPTION_FLT_DIVIDE_BY_ZERO */ => return b"EXCEPTION_FLT_DIVIDE_BY_ZERO\0".as_ptr() as *const i8,
        0xC000008F /* EXCEPTION_FLT_INEXACT_RESULT */ => return b"EXCEPTION_FLT_INEXACT_RESULT\0".as_ptr() as *const i8,
        0xC0000090 /* EXCEPTION_FLT_INVALID_OPERATION */ => return b"EXCEPTION_FLT_INVALID_OPERATION\0".as_ptr() as *const i8,
        0xC0000091 /* EXCEPTION_FLT_OVERFLOW */ => return b"EXCEPTION_FLT_OVERFLOW\0".as_ptr() as *const i8,
        0xC0000092 /* EXCEPTION_FLT_STACK_CHECK */ => return b"EXCEPTION_FLT_STACK_CHECK\0".as_ptr() as *const i8,
        0xC0000093 /* EXCEPTION_FLT_UNDERFLOW */ => return b"EXCEPTION_FLT_UNDERFLOW\0".as_ptr() as *const i8,
        0xC0000094 /* EXCEPTION_INT_DIVIDE_BY_ZERO */ => return b"EXCEPTION_INT_DIVIDE_BY_ZERO\0".as_ptr() as *const i8,
        0xC0000095 /* EXCEPTION_INT_OVERFLOW */ => return b"EXCEPTION_INT_OVERFLOW\0".as_ptr() as *const i8,
        0xC0000096 /* EXCEPTION_PRIV_INSTRUCTION */ => return b"EXCEPTION_PRIV_INSTRUCTION\0".as_ptr() as *const i8,
        0xC0000006 /* EXCEPTION_IN_PAGE_ERROR */ => return b"EXCEPTION_IN_PAGE_ERROR\0".as_ptr() as *const i8,
        0xC000001D /* EXCEPTION_ILLEGAL_INSTRUCTION */ => return b"EXCEPTION_ILLEGAL_INSTRUCTION\0".as_ptr() as *const i8,
        0xC0000025 /* EXCEPTION_NONCONTINUABLE_EXCEPTION */ => return b"EXCEPTION_NONCONTINUABLE_EXCEPTION\0".as_ptr() as *const i8,
        0xC00000FD /* EXCEPTION_STACK_OVERFLOW */ => return b"EXCEPTION_STACK_OVERFLOW\0".as_ptr() as *const i8,
        0xC0000026 /* EXCEPTION_INVALID_DISPOSITION */ => return b"EXCEPTION_INVALID_DISPOSITION\0".as_ptr() as *const i8,
        0x80000001 /* EXCEPTION_GUARD_PAGE */ => return b"EXCEPTION_GUARD_PAGE\0".as_ptr() as *const i8,
        0xC0000008 /* EXCEPTION_INVALID_HANDLE */ => return b"EXCEPTION_INVALID_HANDLE\0".as_ptr() as *const i8,
        0x80000100 /* EXCEPTION_WINE_STUB */ => return b"EXCEPTION_WINE_STUB\0".as_ptr() as *const i8,
        _ if code == STATUS_ASSERTION_FAILURE as DWORD => return b"STATUS_ASSERTION_FAILURE\0".as_ptr() as *const i8,
        _ => {}
    }

    return b"--\0".as_ptr() as *const i8;
}

unsafe extern "C" fn ReadKey(hKey: HKEY, ValueName: *const i8, Buffer: *mut i8, mut size: DWORD) {
    let mut dwType: DWORD = 0;
    let ret: LSTATUS = RegQueryValueExA(hKey, ValueName, core::ptr::null_mut(), core::ptr::addr_of_mut!(dwType), Buffer as LPBYTE, core::ptr::addr_of_mut!(size));
    if ret != (ERROR_SUCCESS as i32) || dwType != REG_SZ {
        *Buffer.add(0 as usize) = 0i8;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PrintSystemInfo(output: *mut FILE, data: &mut DumpData) {
    let mut LocalTime: SYSTEMTIME = core::mem::zeroed();
    GetLocalTime(core::ptr::addr_of_mut!(LocalTime));
    // The C++ concatenates buildno.h's KERNEL_VERSION_STR into the literal; the island has it
    // as a NUL-terminated constant, and a `%s` of it prints the same bytes (it holds no '%').
    xfprintf(output, b"\r\nReactOS %s DrWtsn32\r\n\r\n\0".as_ptr() as *const i8, KERNEL_VERSION_STR.as_ptr());
    xfprintf(output, b"Application exception occurred:\r\n\0".as_ptr() as *const i8);
    xfprintf(output, b"    App: %ls (pid=%d, tid=0x%x)\r\n\0".as_ptr() as *const i8, data.ProcessName.c_str(), data.ProcessID, data.ThreadID);
    xfprintf(output, b"    When: %d/%d/%d @ %02d:%02d:%02d.%d\r\n\0".as_ptr() as *const i8,
        LocalTime.wDay as i32, LocalTime.wMonth as i32, LocalTime.wYear as i32,
        LocalTime.wHour as i32, LocalTime.wMinute as i32, LocalTime.wSecond as i32, LocalTime.wMilliseconds as i32);

    xfprintf(output, b"    First chance: %u\r\n\0".as_ptr() as *const i8, data.ExceptionInfo.dwFirstChance);
    let Record: *mut EXCEPTION_RECORD = core::ptr::addr_of_mut!(data.ExceptionInfo.ExceptionRecord);
    xfprintf(output, b"    Exception number: 0x%08x (%s)\r\n\0".as_ptr() as *const i8, (*Record).ExceptionCode, Exception2Str((*Record).ExceptionCode));
    xfprintf(output, b"    Exception flags: 0x%08x\r\n\0".as_ptr() as *const i8, (*Record).ExceptionFlags);
    xfprintf(output, b"    Exception address: %p\r\n\0".as_ptr() as *const i8, (*Record).ExceptionAddress);
    if (*Record).NumberParameters != 0 {
        xfprintf(output, b"    Exception parameters: %u\r\n\0".as_ptr() as *const i8, (*Record).NumberParameters);
        let mut n: DWORD = 0;
        while n < core::cmp::min::<DWORD>(EXCEPTION_MAXIMUM_PARAMETERS, (*Record).NumberParameters) {
            xfprintf(output, b"      Parameter %u: 0x%p\r\n\0".as_ptr() as *const i8, n, (*Record).ExceptionInformation[n as usize]);
            n += 1;
        }
    }

    let mut Buffer: [i8; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut count: DWORD = core::mem::size_of_val(&Buffer) as DWORD;
    xfprintf(output, b"\r\n*----> System Information <----*\r\n\r\n\0".as_ptr() as *const i8);
    if GetComputerNameA(Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(count)) != 0 {
        xfprintf(output, b"    Computer Name: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
    }
    count = core::mem::size_of_val(&Buffer) as DWORD;
    if GetUserNameA(Buffer.as_mut_ptr(), core::ptr::addr_of_mut!(count)) != 0 {
        xfprintf(output, b"    User Name: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
    }

    let mut SMBiosBuf: PVOID;
    let mut DmiStrings: [*mut i8; (ID_STRINGS_MAX as _)] = [core::ptr::null_mut(); (ID_STRINGS_MAX as _)];
    SMBiosBuf = LoadSMBiosData(DmiStrings.as_mut_ptr());
    if !SMBiosBuf.is_null() {
        if !DmiStrings[(BIOS_VENDOR) as usize].is_null() {
            xfprintf(output, b"    BIOS Vendor: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BIOS_VENDOR) as usize]);
        }
        if !DmiStrings[(BIOS_VERSION) as usize].is_null() {
            xfprintf(output, b"    BIOS Version: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BIOS_VERSION) as usize]);
        }
        if !DmiStrings[(BIOS_DATE) as usize].is_null() {
            xfprintf(output, b"    BIOS Date: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BIOS_DATE) as usize]);
        }
        if !DmiStrings[(SYS_VENDOR) as usize].is_null() {
            xfprintf(output, b"    System Manufacturer: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(SYS_VENDOR) as usize]);
        }
        if !DmiStrings[(SYS_FAMILY) as usize].is_null() {
            xfprintf(output, b"    System Family: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(SYS_FAMILY) as usize]);
        }
        if !DmiStrings[(SYS_PRODUCT) as usize].is_null() {
            xfprintf(output, b"    System Model: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(SYS_PRODUCT) as usize]);
        }
        if !DmiStrings[(SYS_VERSION) as usize].is_null() {
            xfprintf(output, b"    System Version: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(SYS_VERSION) as usize]);
        }
        if !DmiStrings[(SYS_SKU) as usize].is_null() {
            xfprintf(output, b"    System SKU: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(SYS_SKU) as usize]);
        }
        if !DmiStrings[(BOARD_VENDOR) as usize].is_null() {
            xfprintf(output, b"    Baseboard Manufacturer: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BOARD_VENDOR) as usize]);
        }
        if !DmiStrings[(BOARD_NAME) as usize].is_null() {
            xfprintf(output, b"    Baseboard Model: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BOARD_NAME) as usize]);
        }
        if !DmiStrings[(BOARD_VERSION) as usize].is_null() {
            xfprintf(output, b"    Baseboard Version: %s\r\n\0".as_ptr() as *const i8, DmiStrings[(BOARD_VERSION) as usize]);
        }
        FreeSMBiosData(SMBiosBuf);
    }

    let mut info: SYSTEM_INFO = core::mem::zeroed();
    GetSystemInfo(core::ptr::addr_of_mut!(info));
    xfprintf(output, b"    Number of Processors: %d\r\n\0".as_ptr() as *const i8, info.dwNumberOfProcessors);

    let mut hKey: HKEY = core::mem::zeroed();
    let mut ret: LSTATUS = RegOpenKeyExA(HKEY_LOCAL_MACHINE, b"HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0\0".as_ptr() as *const i8,
        0, KEY_READ, core::ptr::addr_of_mut!(hKey));
    if ret == (ERROR_SUCCESS as i32) {
        let mut dwType: DWORD = 0;
        count = core::mem::size_of_val(&Buffer) as DWORD;
        ret = RegQueryValueExA(hKey, b"Identifier\0".as_ptr() as *const i8, core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType), Buffer.as_mut_ptr() as LPBYTE, core::ptr::addr_of_mut!(count));
        if ret == (ERROR_SUCCESS as i32) && dwType == REG_SZ {
            Buffer[count as usize] = 0;
            xfprintf(output, b"    Processor Type: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
        }
        RegCloseKey(hKey);
    }

    ret = RegOpenKeyExA(HKEY_LOCAL_MACHINE, b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0".as_ptr() as *const i8,
        0, KEY_READ, core::ptr::addr_of_mut!(hKey));
    if ret == (ERROR_SUCCESS as i32) {
        let mut Version: [i8; 50] = [0; 50];
        ReadKey(hKey, b"ProductName\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        ReadKey(hKey, b"CurrentVersion\0".as_ptr() as *const i8, Version.as_mut_ptr(), core::mem::size_of_val(&Version) as DWORD);
        xfprintf(output, b"    %s Version: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr(), Version.as_ptr());
        ReadKey(hKey, b"BuildLab\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        xfprintf(output, b"    BuildLab: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
        ReadKey(hKey, b"CSDVersion\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        if Buffer[0] != 0 {
            xfprintf(output, b"    Service Pack: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
        }
        ReadKey(hKey, b"CurrentType\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        xfprintf(output, b"    Current Type: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
        ReadKey(hKey, b"RegisteredOrganization\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        xfprintf(output, b"    Registered Organization: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());
        ReadKey(hKey, b"RegisteredOwner\0".as_ptr() as *const i8, Buffer.as_mut_ptr(), core::mem::size_of_val(&Buffer) as DWORD);
        xfprintf(output, b"    Registered Owner: %s\r\n\0".as_ptr() as *const i8, Buffer.as_ptr());

        RegCloseKey(hKey);
    }
}
