// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7C8F589C7B27: [u16; 3] = [0x0025u16, 0x0031, 0x0000];
pub static RUSTCONV_WSTR_F3C6A8F2AC47: [u16; 4] = [0x0025u16, 0x006C, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1B9E0FE7B740: [u16; 2] = [0x003Fu16, 0x0000];
pub static RUSTCONV_C2R_WSTR_8B72557F3C97: [u16; 2] = [0x0074u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_9EE4EE67ED95: [u16; 2] = [0x0066u16, 0x0000];
pub static RUSTCONV_C2R_WSTR_C09125821951: [u16; 3] = [0x0069u16, 0x006D, 0x0000];
pub static RUSTCONV_C2R_WSTR_D383196BCC18: [u16; 4] = [0x0070u16, 0x0069, 0x0064, 0x0000];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub p: PROCESSENTRY32W,
    pub matched: BOOL,
    pub is_numeric: BOOL,
}

static mut force_termination: BOOL = (FALSE) as i32;

static mut kill_child_processes: BOOL = (FALSE) as i32;

static mut task_list: *mut *mut WCHAR = ::core::ptr::null::<*mut WCHAR>() as *mut *mut WCHAR;

static mut task_count: ::core::ffi::c_uint = 0;

static mut process_list: *mut C2RustUnnamed_11 =
    ::core::ptr::null::<C2RustUnnamed_11>() as *mut C2RustUnnamed_11;

static mut process_count: ::core::ffi::c_uint = 0;

#[no_mangle]
pub static mut pkill_list: *mut ::core::ffi::c_uint =
    ::core::ptr::null::<::core::ffi::c_uint>() as *mut ::core::ffi::c_uint;

#[no_mangle]
pub static mut pkill_size: DWORD = 0;

#[no_mangle]
pub static mut self_pid: DWORD = 0;

static mut opList: [PWCHAR; 5] = unsafe {
    [
        RUSTCONV_C2R_WSTR_9EE4EE67ED95.as_ptr()
            as PWCHAR,
        RUSTCONV_C2R_WSTR_C09125821951.as_ptr()
            as PWCHAR,
        RUSTCONV_C2R_WSTR_D383196BCC18.as_ptr()
            as PWCHAR,
        RUSTCONV_C2R_WSTR_1B9E0FE7B740.as_ptr()
            as PWCHAR,
        RUSTCONV_C2R_WSTR_8B72557F3C97.as_ptr()
            as PWCHAR,
    ]
};

// #define wcsicmp _wcsicmp  (alias: call sites use _wcsicmp)
// WINE_DEFAULT_DEBUG_CHANNEL(taskkill);  (debug channel declaration, no Rust equivalent)

#[repr(C)]
pub struct ProcessListEntry {
    pub p: PROCESSENTRY32W,
    pub matched: BOOL,
    pub is_numeric: BOOL,
}

#[repr(C)]
pub struct pid_close_info {
    pub pid: DWORD,
    pub found: BOOL,
}

pub const OP_PARAM_INVALID: i32 = -1;
pub const OP_PARAM_FORCE_TERMINATE: i32 = 0;
pub const OP_PARAM_IMAGE: i32 = 1;
pub const OP_PARAM_PID: i32 = 2;
pub const OP_PARAM_HELP: i32 = 3;
pub const OP_PARAM_TERMINATE_CHILD: i32 = 4;

unsafe extern "C" fn taskkill_vprintfW(msg: *const WCHAR, mut va_args: va_list) -> i32 {
    let mut wlen: i32;
    let mut count: DWORD = 0;
    let mut msg_buffer: [WCHAR; 8192] = [0; 8192];

    wlen = FormatMessageW(
        FORMAT_MESSAGE_FROM_STRING as DWORD,
        msg as LPCVOID,
        0 as DWORD,
        0 as DWORD,
        core::ptr::addr_of_mut!(msg_buffer) as LPWSTR,
        ARRAY_SIZE!(msg_buffer) as DWORD,
        core::ptr::addr_of_mut!(va_args) as *mut va_list,
    ) as i32;

    if WriteConsoleW(
        GetStdHandle(STD_OUTPUT_HANDLE),
        core::ptr::addr_of_mut!(msg_buffer) as *const core::ffi::c_void,
        wlen as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    ) == 0
    {
        let mut len: DWORD = 0;
        let mut msgA: *mut core::ffi::c_char = core::ptr::null_mut();

        /* On Windows WriteConsoleW() fails if the output is redirected. So fall
         * back to WriteFile() using OEM code page.
         */
        len = WideCharToMultiByte(
            GetOEMCP(),
            0 as DWORD,
            core::ptr::addr_of_mut!(msg_buffer) as LPCWSTR,
            wlen,
            core::ptr::null_mut::<CHAR>(),
            0 as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as DWORD;
        msgA = malloc(len as usize) as *mut core::ffi::c_char;
        if msgA.is_null() {
            return 0 as i32;
        }

        WideCharToMultiByte(
            GetOEMCP(),
            0 as DWORD,
            core::ptr::addr_of_mut!(msg_buffer) as LPCWSTR,
            wlen,
            msgA as LPSTR,
            len as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        WriteFile(
            GetStdHandle(STD_OUTPUT_HANDLE),
            msgA as LPCVOID,
            len,
            core::ptr::addr_of_mut!(count),
            FALSE as usize as LPOVERLAPPED,
        );
        free(msgA as *mut core::ffi::c_void);
    }

    return count as i32;
}

#[no_mangle]
unsafe extern "C" fn taskkill_printfW_v(msg: *const WCHAR, ap_list: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    let mut va_args: va_list = *(ap_list as *mut va_list);
    let mut len: ::core::ffi::c_int = 0;

    len = taskkill_vprintfW(msg, va_args);

    return len;
}


#[no_mangle]
unsafe extern "C" fn taskkill_message_printfW_v(mut msg: core::ffi::c_int, ap_list: *mut ::core::ffi::c_void) -> core::ffi::c_int {
    let mut va_args: va_list;
    let mut msg_buffer: [WCHAR; 8192] = [0; 8192];
    let mut len: core::ffi::c_int;

    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HINSTANCE,
        msg as UINT,
        core::ptr::addr_of_mut!(msg_buffer) as *mut WCHAR,
        ARRAY_SIZE!(msg_buffer) as core::ffi::c_int,
    );

    va_args = *(ap_list as *mut va_list);
    len = taskkill_vprintfW(core::ptr::addr_of_mut!(msg_buffer) as *mut WCHAR, va_args);

    return len;
}



unsafe extern "C" fn taskkill_message(msg: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut msg_buffer: [WCHAR; 8192] = [0; 8192];

    LoadStringW(
        GetModuleHandleW(core::ptr::null_mut()) as HINSTANCE,
        msg as UINT,
        msg_buffer.as_mut_ptr() as LPWSTR,
        ARRAY_SIZE!(msg_buffer) as ::core::ffi::c_int,
    );

    return taskkill_printfW(RUSTCONV_WSTR_7C8F589C7B27.as_ptr(), msg_buffer.as_mut_ptr() as *mut WCHAR);
}

unsafe extern "system" fn pid_enum_proc(hwnd: HWND, lParam: LPARAM) -> BOOL {
    let info: *mut pid_close_info = lParam as *mut pid_close_info;
    let mut hwnd_pid: DWORD = 0;

    GetWindowThreadProcessId(hwnd, core::ptr::addr_of_mut!(hwnd_pid));

    if hwnd_pid == (*info).pid {
        PostMessageW(hwnd, WM_CLOSE as UINT, 0 as WPARAM, 0 as LPARAM);
        (*info).found = (TRUE) as i32;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn enumerate_processes() -> BOOL {
    let mut alloc_count: ::core::ffi::c_uint = 128 as ::core::ffi::c_uint;
    let mut realloc_list: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut snapshot: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS as DWORD, 0 as DWORD);
    if snapshot == INVALID_HANDLE_VALUE {
        return (FALSE) as BOOL;
    }
    process_list = malloc(
        (alloc_count as usize).wrapping_mul(::core::mem::size_of::<C2RustUnnamed_11>() as usize),
    ) as *mut C2RustUnnamed_11;
    if process_list.is_null() {
        return (FALSE) as BOOL;
    }
    (*process_list.offset(0 as isize)).p.dwSize =
        core::mem::size_of_val(&(*process_list.offset(0 as isize)).p) as DWORD;
    if Process32FirstW(
        snapshot,
        &raw mut (*process_list.offset(0 as isize)).p,
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    loop {
        (*process_list.offset(process_count as isize)).is_numeric = FALSE as BOOL;
        let fresh15 = process_count;
        process_count = process_count.wrapping_add(1);
        (*process_list.offset(fresh15 as isize)).matched = FALSE as BOOL;
        if process_count == alloc_count {
            alloc_count = alloc_count.wrapping_mul(2 as ::core::ffi::c_uint);
            realloc_list = realloc(
                process_list as *mut ::core::ffi::c_void,
                (alloc_count as usize).wrapping_mul(::core::mem::size_of::<C2RustUnnamed_11>() as usize),
            );
            if realloc_list.is_null() {
                return (FALSE) as BOOL;
            }
            process_list = realloc_list as *mut C2RustUnnamed_11;
        }
        (*process_list.offset(process_count as isize)).p.dwSize =
            core::mem::size_of_val(&(*process_list.offset(process_count as isize)).p) as DWORD;
        if !(Process32NextW(
            snapshot,
            &raw mut (*process_list.offset(process_count as isize)).p,
        ) != 0)
        {
            break;
        }
    }
    CloseHandle(snapshot);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn mark_task_process(str: *const WCHAR, status_code: *mut i32) {
    let mut p: *const WCHAR = str;
    let mut is_numeric: BOOL;
    let mut i: u32;
    let mut pid: DWORD;
    let mut at_not_found: bool = false;
    is_numeric = (TRUE) as i32;
    while *p != 0 {
        let fresh = p;
        p = p.offset(1);
        if iswdigit(*fresh) == 0 {
            is_numeric = (FALSE) as i32;
            break;
        }
    }

    if is_numeric != 0 {
        pid = wcstol(str, core::ptr::null_mut::<*mut WCHAR>(), 10) as DWORD;
        i = 0;
        while i < process_count {
            if (*process_list.offset(i as isize)).p.th32ProcessID == pid {
                break;
            }
            i = i.wrapping_add(1);
        }
        if i == process_count || (*process_list.offset(i as isize)).matched != 0 {
            at_not_found = true;
        } else {
            (*process_list.offset(i as isize)).matched = (TRUE) as i32;
            (*process_list.offset(i as isize)).is_numeric = (TRUE) as i32;
            if pid == self_pid {
                taskkill_message((STRING_SELF_TERMINATION as i32));
                *status_code = 1;
            } else {
                let idx = pkill_size;
                pkill_size = pkill_size.wrapping_add(1);
                *pkill_list.offset(idx as isize) = i;
            }
            return;
        }
    } else {
        i = 0;
        while i < process_count {
            if wcsicmp(
                core::ptr::addr_of_mut!((*process_list.offset(i as isize)).p.szExeFile) as *mut WCHAR,
                str,
            ) == 0
                && (*process_list.offset(i as isize)).matched == 0
            {
                (*process_list.offset(i as isize)).matched = (TRUE) as i32;
                if (*process_list.offset(i as isize)).p.th32ProcessID == self_pid {
                    taskkill_message((STRING_SELF_TERMINATION as i32));
                    *status_code = 1;
                    return;
                } else {
                    let idx = pkill_size;
                    pkill_size = pkill_size.wrapping_add(1);
                    *pkill_list.offset(idx as isize) = i;
                }
            }
            i = i.wrapping_add(1);
        }
        // Cannot find any process matching the PID or name
        if pkill_size == 0 {
            at_not_found = true;
        }
    }

    if at_not_found {
        taskkill_message_printfW((STRING_SEARCH_FAILED as i32), str);
        *status_code = 128;
    }
}

unsafe extern "C" fn taskkill_message_print_process(msg: i32, index: u32) {
    let mut pid_str: [WCHAR; 16] = [0; 16];

    if (*process_list.add(index as usize)).is_numeric == 0 {
        taskkill_message_printfW(
            msg,
            core::ptr::addr_of_mut!((*process_list.add(index as usize)).p.szExeFile) as *mut WCHAR,
        );
        return;
    }
    wsprintfW(
        pid_str.as_mut_ptr(),
        RUSTCONV_WSTR_F3C6A8F2AC47.as_ptr(),
        (*process_list.add(index as usize)).p.th32ProcessID,
    );
    taskkill_message_printfW(msg, pid_str.as_mut_ptr());
}

unsafe extern "C" fn get_pid_creation_time(pid: DWORD, time: *mut FILETIME) -> BOOL {
    let mut process: HANDLE = core::ptr::null_mut();
    let mut t1: FILETIME = core::mem::zeroed();
    let mut t2: FILETIME = core::mem::zeroed();
    let mut t3: FILETIME = core::mem::zeroed();
    let mut success: BOOL = 0;

    process = OpenProcess(PROCESS_QUERY_INFORMATION, (FALSE) as BOOL, pid);
    if process.is_null() {
        return (FALSE) as BOOL;
    }

    success = GetProcessTimes(
        process,
        time,
        core::ptr::addr_of_mut!(t1),
        core::ptr::addr_of_mut!(t2),
        core::ptr::addr_of_mut!(t3),
    );
    CloseHandle(process);

    return success;
}

unsafe extern "C" fn queue_children(mut ppid: DWORD) {
    let mut parent_creation_time: FILETIME = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    let mut i: u32;
    if get_pid_creation_time(ppid, &raw mut parent_creation_time) == 0 {
        return;
    }
    i = 0;
    while i < process_count {
        let mut child_creation_time: FILETIME = FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut pid: DWORD = 0;
        if !((*process_list.offset(i as isize)).matched != 0) {
            pid = (*process_list.offset(i as isize)).p.th32ProcessID;
            if !(pid == self_pid) {
                if !(get_pid_creation_time(pid, &raw mut child_creation_time) == 0) {
                    if (*process_list.offset(i as isize)).p.th32ParentProcessID == ppid
                        && CompareFileTime(
                            &raw mut parent_creation_time,
                            &raw mut child_creation_time,
                        ) < 0
                    {
                        WINE_TRACE!(
                            b"Adding child %04lx.\n\0".as_ptr() as *const i8,
                            pid,
                        );
                        (*process_list.offset(i as isize)).matched =
                            TRUE as BOOL;
                        let fresh19 = pkill_size;
                        pkill_size = pkill_size.wrapping_add(1);
                        *pkill_list.offset(fresh19 as isize) = i;
                    }
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn mark_child_processes() {
    let mut front: ::core::ffi::c_uint = 0;
    while pkill_size.wrapping_sub(front as DWORD) > 0 {
        let mut len: ::core::ffi::c_uint = pkill_size.wrapping_sub(front as DWORD) as ::core::ffi::c_uint;
        let mut i: ::core::ffi::c_uint = 0;
        i = 0;
        while i < len {
            let fresh20 = front;
            front = front.wrapping_add(1);
            let mut pid: DWORD = (*process_list.offset(*pkill_list.offset(fresh20 as isize) as isize)).p.th32ProcessID;
            queue_children(pid);
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn send_close_messages() -> ::core::ffi::c_int {
    let mut process_name: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut info: pid_close_info = pid_close_info { pid: 0, found: 0 };
    let mut i: ::core::ffi::c_uint = 0;
    let mut status_code: ::core::ffi::c_int = 0;

    let mut index: DWORD = 0;
    while index < pkill_size {
        'body: {
            i = *pkill_list.offset(index as isize);

            info.pid = (*process_list.offset(i as isize)).p.th32ProcessID;
            process_name = &raw mut (*process_list.offset(i as isize)).p.szExeFile as *mut WCHAR;
            info.found = (FALSE) as i32;
            WINE_TRACE!(
                b"Terminating pid %04lx.\n\0".as_ptr() as *const ::core::ffi::c_char,
                info.pid,
            );
            EnumWindows(
                Some(pid_enum_proc),
                &raw mut info as LPARAM,
            );
            if info.found != 0 {
                if kill_child_processes != 0 {
                    taskkill_message_printfW(
                        (STRING_CLOSE_CHILD as i32),
                        info.pid,
                        (*process_list.offset(i as isize)).p.th32ParentProcessID,
                    );
                } else if (*process_list.offset(i as isize)).is_numeric != 0 {
                    taskkill_message_printfW((STRING_CLOSE_PID_SEARCH as i32), info.pid);
                } else {
                    taskkill_message_printfW((STRING_CLOSE_PROC_SRCH as i32), process_name, info.pid);
                }
                break 'body;
            }
            taskkill_message_print_process((STRING_SEARCH_FAILED as i32), i);
            status_code = 128;
        }
        index = index.wrapping_add(1);
    }

    return status_code;
}

unsafe extern "C" fn terminate_processes() -> ::core::ffi::c_int {
    let mut process_name: *const WCHAR = ::core::ptr::null::<WCHAR>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut status_code: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut process: HANDLE = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut pid: DWORD = 0;
    let mut index: DWORD = 0;
    index = 0 as DWORD;
    while index < pkill_size {
        i = *pkill_list.offset(index as isize);
        pid = (*process_list.offset(i as isize)).p.th32ProcessID;
        process_name = &raw mut (*process_list.offset(i as isize)).p.szExeFile as *mut WCHAR;
        process = OpenProcess(PROCESS_TERMINATE as DWORD, (FALSE) as BOOL, pid);
        if process.is_null() {
            taskkill_message_print_process((STRING_SEARCH_FAILED as i32), i);
            status_code = 128 as ::core::ffi::c_int;
        } else if TerminateProcess(process, 1 as UINT) == 0 {
            if kill_child_processes != 0 {
                taskkill_message_printfW(
                    (STRING_TERM_CHILD_FAILED as i32),
                    pid,
                    (*process_list.offset(i as isize)).p.th32ParentProcessID,
                );
            } else {
                taskkill_message_print_process((STRING_TERMINATE_FAILED as i32), i);
            }
            status_code = 1 as ::core::ffi::c_int;
            CloseHandle(process);
        } else {
            if kill_child_processes != 0 {
                taskkill_message_printfW(
                    (STRING_TERM_CHILD as i32),
                    pid,
                    (*process_list.offset(i as isize)).p.th32ParentProcessID,
                );
            } else if (*process_list.offset(i as isize)).is_numeric != 0 {
                taskkill_message_printfW((STRING_TERM_PID_SEARCH as i32), pid);
            } else {
                taskkill_message_printfW((STRING_TERM_PROC_SEARCH as i32), process_name, pid);
            }
            CloseHandle(process);
        }
        index = index.wrapping_add(1);
    }
    return status_code;
}

unsafe extern "C" fn add_to_task_list(name: *mut WCHAR) -> BOOL {
    static mut list_size: ::core::ffi::c_uint = 16;

    if task_list.is_null() {
        task_list = malloc(
            // sizeof(*task_list): a WCHAR *, not a WCHAR (half-size slots overflowed at /im #5).
            (list_size as usize).wrapping_mul(::core::mem::size_of::<*mut WCHAR>()),
        ) as *mut *mut WCHAR;
        if task_list.is_null() {
            return (FALSE) as BOOL;
        }
    } else if task_count == list_size {
        let mut realloc_list: *mut ::core::ffi::c_void =
            ::core::ptr::null_mut::<::core::ffi::c_void>();
        list_size = list_size.wrapping_mul(2);
        realloc_list = realloc(
            task_list as *mut ::core::ffi::c_void,
            // sizeof(*task_list): a WCHAR *, not a WCHAR (half-size slots overflowed at /im #5).
            (list_size as usize).wrapping_mul(::core::mem::size_of::<*mut WCHAR>()),
        );
        if realloc_list.is_null() {
            return (FALSE) as BOOL;
        }
        task_list = realloc_list as *mut *mut WCHAR;
    }
    let fresh21 = task_count;
    task_count = task_count.wrapping_add(1);
    *task_list.offset(fresh21 as isize) = name;
    return (TRUE) as BOOL;
}

unsafe extern "C" fn get_argument_type(mut argument: *mut WCHAR) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int;
    if *argument.offset(0 as isize) != 0x002Fu16
        && *argument.offset(0 as isize) != 0x002Du16
    {
        return OP_PARAM_INVALID;
    }
    argument = argument.offset(1);

    i = 0;
    while i < _countof!(opList) as ::core::ffi::c_int {
        if wcsicmp(opList[i as usize] as *const wchar_t, argument as *const wchar_t) == 0 {
            return i;
        }
        i += 1;
    }
    return OP_PARAM_INVALID;
}

unsafe extern "C" fn process_arguments(mut argc: ::core::ffi::c_int, mut argv: *mut *mut WCHAR) -> BOOL {
    let mut has_im: BOOL = (FALSE) as i32;
    let mut has_pid: BOOL = (FALSE) as i32;
    let mut has_help: BOOL = (FALSE) as i32;

    if argc > 1 {
        let mut i: ::core::ffi::c_int = 0;
        i = 1;
        while i < argc {
            let mut argument: ::core::ffi::c_int = get_argument_type(*argv.offset(i as isize));

            match argument {
                x if x == OP_PARAM_FORCE_TERMINATE => {
                    if force_termination != 0 {
                        taskkill_message_printfW(
                            (STRING_PARAM_TOO_MUCH as i32),
                            *argv.offset(i as isize),
                            1,
                        );
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }
                    force_termination = (TRUE) as i32;
                }
                x if x == OP_PARAM_IMAGE || x == OP_PARAM_PID => {
                    if (*argv.offset((i + 1) as isize)).is_null() {
                        taskkill_message_printfW((STRING_MISSING_PARAM as i32), *argv.offset(i as isize));
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }

                    if argument == OP_PARAM_IMAGE {
                        has_im = (TRUE) as i32;
                    }
                    if argument == OP_PARAM_PID {
                        has_pid = (TRUE) as i32;
                    }

                    if has_im != 0 && has_pid != 0 {
                        taskkill_message((STRING_MUTUAL_EXCLUSIVE as i32));
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }

                    if get_argument_type(*argv.offset((i + 1) as isize))
                        != OP_PARAM_INVALID
                    {
                        taskkill_message_printfW((STRING_MISSING_PARAM as i32), *argv.offset(i as isize));
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }

                    i += 1;
                    if add_to_task_list(*argv.offset(i as isize)) == 0 {
                        // add next parameters to task_list
                        return (FALSE) as BOOL;
                    }
                }
                x if x == OP_PARAM_HELP => {
                    if has_help != 0 {
                        taskkill_message_printfW(
                            (STRING_PARAM_TOO_MUCH as i32),
                            *argv.offset(i as isize),
                            1,
                        );
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }
                    has_help = (TRUE) as i32;
                }
                x if x == OP_PARAM_TERMINATE_CHILD => {
                    if kill_child_processes != 0 {
                        taskkill_message_printfW(
                            (STRING_PARAM_TOO_MUCH as i32),
                            *argv.offset(i as isize),
                            1,
                        );
                        taskkill_message((STRING_USAGE as i32));
                        return (FALSE) as BOOL;
                    }
                    kill_child_processes = (TRUE) as i32;
                }
                x if x == OP_PARAM_INVALID => {
                    taskkill_message((STRING_INVALID_OPTION as i32));
                    taskkill_message((STRING_USAGE as i32));
                    return (FALSE) as BOOL;
                }
                _ => {
                    taskkill_message((STRING_INVALID_OPTION as i32));
                    taskkill_message((STRING_USAGE as i32));
                    return (FALSE) as BOOL;
                }
            }
            i += 1;
        }
    }

    if has_help != 0 {
        if argc > 2 {
            // any parameters other than -? is specified
            taskkill_message((STRING_INVALID_SYNTAX as i32));
            taskkill_message((STRING_USAGE as i32));
            return (FALSE) as BOOL;
        } else {
            taskkill_message((STRING_USAGE as i32));
            exit(0);
        }
    } else if has_im == 0 && has_pid == 0 {
        // has_help == FALSE
        // both has_im and has_pid are missing (maybe -fi option is missing too, if implemented later)
        taskkill_message((STRING_MISSING_OPTION as i32));
        taskkill_message((STRING_USAGE as i32));
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: c_int, argv: *mut *mut WCHAR) -> c_int {
    let mut search_status: c_int = 0;
    let mut terminate_status: c_int;
    let mut i: c_uint;

    if process_arguments(argc, argv) == 0 {
        return 1;
    }

    if enumerate_processes() == 0 {
        taskkill_message((STRING_ENUM_FAILED as i32));
        return 1;
    }

    pkill_list = malloc((process_count as usize).wrapping_mul(core::mem::size_of::<c_uint>())) as *mut c_uint;
    if pkill_list.is_null() {
        return 1;
    }
    memset(
        pkill_list as *mut c_void,
        0,
        (process_count as usize).wrapping_mul(core::mem::size_of::<c_uint>()),
    );
    pkill_size = 0;

    self_pid = GetCurrentProcessId();

    i = 0;
    while i < task_count {
        mark_task_process(*task_list.offset(i as isize), core::ptr::addr_of_mut!(search_status));
        i = i.wrapping_add(1);
    }
    if kill_child_processes != 0 {
        mark_child_processes();
    }
    if force_termination != 0 {
        terminate_status = terminate_processes();
    } else {
        terminate_status = send_close_messages();
    }
    return if search_status != 0 { search_status } else { terminate_status };
}
