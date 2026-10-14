use crate::ftp::ftp::null;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_ASCII: i32 = 100;

// `int checkRecv(SOCKET s);` is a prototype only (no definition in this preamble); it is
// provided by the function pipeline, so no declaration is emitted here.

pub const EPOCHFILETIME: i64 = 116444736000000000;
#[no_mangle]
pub unsafe extern "C" fn checkRecv(s: SOCKET) -> i32 {
    let mut testVal: i32;
    let mut sSet: fd_set = core::mem::zeroed();
    let mut timeout: timeval = core::mem::zeroed();
    timeout.tv_sec = 60;

    FD_ZERO!(core::ptr::addr_of_mut!(sSet));

    FD_SET!(s, core::ptr::addr_of_mut!(sSet));

    testVal = select(0, core::ptr::addr_of_mut!(sSet), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(timeout));

    if testVal == SOCKET_ERROR {
        fprintf(__iob_func().add(2), b"Socket Error\0".as_ptr() as *const i8);
    }

    return testVal;
}
#[no_mangle]
pub unsafe extern "C" fn blkfree(av0: *mut *mut ::core::ffi::c_char) {
    let mut av: *mut *mut i8 = av0;
    while !(*av).is_null() {
        let fresh19 = av;
        av = av.offset(1);
        free(*fresh19 as *mut core::ffi::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn glob(v: *const ::core::ffi::c_char) -> *mut *mut ::core::ffi::c_char {
    (NULL).cast()
}

#[no_mangle]
pub unsafe extern "C" fn sleep(time: ::core::ffi::c_int) -> ::core::ffi::c_int {
    time
}

#[no_mangle]
pub unsafe extern "C" fn herror(string: *mut ::core::ffi::c_char) -> ::core::ffi::c_int {
    0
}
#[no_mangle]
pub unsafe extern "C" fn fgetcSocket(s: ::core::ffi::c_int) -> ::core::ffi::c_int {
    static mut index: ::core::ffi::c_int = 0;
    static mut total: ::core::ffi::c_int = 0;
    static mut buffer: [::core::ffi::c_char; 4096] = [0; 4096];

    if index == total {
        index = 0;
        total = recv(
            (s) as SOCKET,
            ::core::ptr::addr_of_mut!(buffer) as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 4096]>() as ::core::ffi::c_int,
            0,
        );

        if total == SOCKET_ERROR {
            total = 0;
            return (ERROR as i32);
        }

        if total == 0 {
            return EOF;
        }
    }
    let fresh = index;
    index = index + 1;
    return buffer[fresh as usize] as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn fprintfSocket_v(
    mut s: ::core::ffi::c_int,
    mut format: *const ::core::ffi::c_char,
    ap_list: *mut ::core::ffi::c_void,
) -> *const ::core::ffi::c_char {
    let mut argptr: va_list;
    let mut buffer: [::core::ffi::c_char; 10009] = [0; 10009];

    argptr = *(ap_list as *mut va_list);
    vsprintf(buffer.as_mut_ptr(), format, argptr);

    send(
        s as SOCKET,
        buffer.as_mut_ptr(),
        strlen(buffer.as_mut_ptr()) as ::core::ffi::c_int,
        0,
    );

    return ::core::ptr::null::<::core::ffi::c_char>();
}


#[no_mangle]
pub unsafe extern "C" fn fputsSocket(format: *const core::ffi::c_char, s: core::ffi::c_int) -> *const core::ffi::c_char {
    send(s as SOCKET, format, strlen(format) as core::ffi::c_int, 0);

    return core::ptr::null::<core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn fputcSocket(s: ::core::ffi::c_int, putChar: ::core::ffi::c_char) -> ::core::ffi::c_int {
   let mut buffer: [::core::ffi::c_char; 2] = [0; 2];

   buffer[0] = putChar;
   buffer[1] = 0 as ::core::ffi::c_char;

   if SOCKET_ERROR == send(s as SOCKET, buffer.as_mut_ptr(), 1, 0) {
       let mut iret: ::core::ffi::c_int = WSAGetLastError();
       fprintf(__iob_func().add(1), b"fputcSocket: %d\n\0".as_ptr() as *const ::core::ffi::c_char, iret);
       return 0;
   } else {
       return putChar as ::core::ffi::c_int;
   }
}
#[no_mangle]
pub unsafe extern "C" fn fputSocket(mut s: ::core::ffi::c_int, mut buffer: *mut ::core::ffi::c_char, mut len: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut iret: ::core::ffi::c_int = 0;
    while len != 0 {
        iret = send(s as SOCKET, buffer, len, 0);
        if SOCKET_ERROR == iret {
            iret = WSAGetLastError();
            fprintf(__iob_func().add(1), b"fputcSocket: %d\n\0".as_ptr() as *const ::core::ffi::c_char, iret);
            return 0;
        } else {
            len -= iret;
            return len;
        }
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn fgetsSocket(mut s: ::core::ffi::c_int, mut string: *mut ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    let mut buffer: [::core::ffi::c_char; 2] = [0 as ::core::ffi::c_char; 2];
    let mut i: ::core::ffi::c_int;
    let mut count: ::core::ffi::c_int;
    i = 0;
    count = 1;
    while count != 0
        && buffer[0 as usize] as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
    {
        checkRecv((s) as SOCKET);
        count = recv(
            s as SOCKET,
            buffer.as_mut_ptr(),
            1,
            0,
        );
        if count == SOCKET_ERROR {
            printf(b"Error in fgetssocket\0".as_ptr() as *const ::core::ffi::c_char);
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if count == 1 {
            *string.offset(i as isize) = buffer[0 as usize];
            if i == MAX_ASCII - 3 {
                count = 0;
                i += 1;
                *string.offset(i as isize) = '\n' as ::core::ffi::c_char;
                i += 1;
                *string.offset(i as isize) = '\0' as ::core::ffi::c_char;
            }
        } else {
            if i == 0 {
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                *string.offset(i as isize) = '\n' as ::core::ffi::c_char;
                *string.offset((i + 1) as isize) = '\0' as ::core::ffi::c_char;
                return string;
            }
        }
        i += 1;
    }
    *string.offset(i as isize) = '\0' as ::core::ffi::c_char;
    if false {
        printf(b"%s\0".as_ptr() as *const ::core::ffi::c_char, string);
    }
    return string;
}
#[no_mangle]
pub unsafe extern "C" fn getpass(prompt: *const ::core::ffi::c_char) -> *mut ::core::ffi::c_char {
    static mut input: [::core::ffi::c_char; 256] = [0; 256];
    let mut in_0: HANDLE = core::ptr::null_mut::<::core::ffi::c_void>();
    let mut err: HANDLE = core::ptr::null_mut::<::core::ffi::c_void>();
    let mut count: DWORD = 0;
    in_0 = GetStdHandle(STD_INPUT_HANDLE);
    err = GetStdHandle(STD_ERROR_HANDLE);
    if in_0 == INVALID_HANDLE_VALUE || err == INVALID_HANDLE_VALUE {
        return core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if WriteFile(
        err,
        prompt as LPCVOID,
        strlen(prompt) as DWORD,
        &raw mut count,
        core::ptr::null_mut::<_OVERLAPPED>(),
    ) != 0
    {
        let mut istty: ::core::ffi::c_int =
            (GetFileType(in_0) == FILE_TYPE_CHAR as DWORD) as ::core::ffi::c_int;
        let mut old_flags: DWORD = 0;
        let mut rc: ::core::ffi::c_int = 0;
        if istty != 0 {
            if GetConsoleMode(in_0, &raw mut old_flags) != 0 {
                SetConsoleMode(in_0, (ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT) as DWORD);
            } else {
                istty = 0 as ::core::ffi::c_int;
            }
        }
        /* Need to read line one byte at time to avoid blocking, if not a
           tty, so always do it this way.  */
        count = 0 as DWORD;
        while 1 != 0 {
            let mut dummy: DWORD = 0;
            let mut one_char: ::core::ffi::c_char = 0;
            rc = ReadFile(
                in_0,
                &raw mut one_char as PVOID,
                1 as DWORD,
                &raw mut dummy,
                core::ptr::null_mut::<_OVERLAPPED>(),
            ) as ::core::ffi::c_int;
            if rc == 0 as ::core::ffi::c_int {
                break;
            }
            if one_char as ::core::ffi::c_int == '\r' as i32 {
                if istty != 0 {
                    continue;
                } else {
                    break;
                }
            }
            if one_char as ::core::ffi::c_int == '\n' as i32 {
                break;
            }
            /* Silently truncate password string if overly long.  */
            if (count as usize)
                < core::mem::size_of::<[::core::ffi::c_char; 256]>()
                    .wrapping_sub(1 as usize)
            {
                let fresh18 = count;
                count = count.wrapping_add(1);
                input[fresh18 as usize] = one_char;
            }
        }
        input[count as usize] = '\0' as i32 as ::core::ffi::c_char;

        WriteFile(
            err,
            b"\r\n\0".as_ptr() as *const ::core::ffi::c_char as LPCVOID,
            2 as DWORD,
            &raw mut count,
            core::ptr::null_mut::<_OVERLAPPED>(),
        );
        if istty != 0 {
            SetConsoleMode(in_0, old_flags);
        }
        if rc != 0 {
            return &raw mut input as *mut ::core::ffi::c_char;
        }
    }
    return core::ptr::null_mut::<::core::ffi::c_char>();
}
#[no_mangle]
pub unsafe extern "C" fn gettimeofday(tv: *mut timeval, tz: *mut timezone) -> ::core::ffi::c_int {
    let mut ft: FILETIME = core::mem::zeroed();
    let mut li: LARGE_INTEGER = core::mem::zeroed();
    let mut t: i64;
    static mut tzflag: i32 = 0;

    if !tv.is_null() {
        GetSystemTimeAsFileTime(core::ptr::addr_of_mut!(ft));
        li.__bindgen_anon_1.LowPart = ft.dwLowDateTime;
        li.__bindgen_anon_1.HighPart = ft.dwHighDateTime as i32;
        t = li.QuadPart as i64;
        t -= EPOCHFILETIME;
        t /= 10;
        (*tv).tv_sec = (t / 1000000) as i32;
        (*tv).tv_usec = (t % 1000000) as i32;
    }

    if !tz.is_null() {
        if tzflag == 0 {
            _tzset();
            tzflag += 1;
        }
        (*tz).tz_minuteswest = (crt_timezone() / 60) as i32;
        (*tz).tz_dsttime = crt_daylight();
    }

    return 0;
}
