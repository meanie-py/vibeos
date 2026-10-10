use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MAX_STR: usize = 1024;
#[no_mangle]
pub unsafe extern "C" fn find_str(mut sz: *mut ::core::ffi::c_char, mut p: *mut FILE, mut invert_search: ::core::ffi::c_int, mut count_lines: ::core::ffi::c_int, mut number_output: ::core::ffi::c_int, mut ignore_case: ::core::ffi::c_int, mut at_start: ::core::ffi::c_int, mut literal_search: ::core::ffi::c_int, mut at_end: ::core::ffi::c_int, mut reg_express: ::core::ffi::c_int, mut exact_match: ::core::ffi::c_int, mut sub_dirs: ::core::ffi::c_int, mut only_fname: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut length: ::core::ffi::c_int = 0;
    let mut line_number: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut total_lines: ::core::ffi::c_long = 0 as ::core::ffi::c_long;
    let mut c: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut temp_str: [::core::ffi::c_char; MAX_STR] = [0; MAX_STR];
    let mut this_line: [::core::ffi::c_char; MAX_STR] = [0; MAX_STR];
    if ignore_case != 0 {
        length = strlen(sz) as ::core::ffi::c_int;
        i = 0;
        while i < length {
            *sz.offset(i as isize) =
                toupper(*sz.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_char;
            i += 1;
        }
    }
    while !fgets(
        &raw mut temp_str as *mut ::core::ffi::c_char,
        (MAX_STR as i32),
        p,
    )
    .is_null()
    {
        length = strlen(&raw mut temp_str as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        if length > 0 && temp_str[(length - 1) as usize] as ::core::ffi::c_int == '\n' as i32 {
            temp_str[(length - 1) as usize] = '\0' as i32 as ::core::ffi::c_char;
        }
        line_number += 1;
        strcpy(
            &raw mut this_line as *mut ::core::ffi::c_char,
            &raw mut temp_str as *mut ::core::ffi::c_char,
        );
        if ignore_case != 0 {
            i = 0;
            while i < length {
                temp_str[i as usize] =
                    toupper(temp_str[i as usize] as ::core::ffi::c_int) as ::core::ffi::c_char;
                i += 1;
            }
        }
        c = strstr(&raw mut temp_str as *mut ::core::ffi::c_char, sz);
        if if invert_search != 0 {
            (c == ::core::ptr::null_mut::<::core::ffi::c_char>()) as ::core::ffi::c_int
        } else {
            (c != ::core::ptr::null_mut::<::core::ffi::c_char>()) as ::core::ffi::c_int
        } != 0
        {
            if count_lines == 0 {
                if number_output != 0 {
                    printf(
                        b"%ld:\0".as_ptr() as *const ::core::ffi::c_char,
                        line_number,
                    );
                }
                puts(&raw mut this_line as *mut ::core::ffi::c_char);
            }
            total_lines += 1;
        }
    }
    if count_lines != 0 {
        printf(
            b"%ld\n\0".as_ptr() as *const ::core::ffi::c_char,
            total_lines,
        );
    }
    return if total_lines > 0 {
        1
    } else {
        0
    };
}
#[no_mangle]
pub unsafe extern "C" fn usage() {
    let mut lpUsage: [TCHAR; 4096] = [0; 4096];

    LoadString(
        GetModuleHandle(core::ptr::null_mut()),
        IDS_USAGE,
        core::ptr::addr_of_mut!(lpUsage) as *mut TCHAR as LPTSTR,
        4096,
    );
    CharToOem(
        core::ptr::addr_of_mut!(lpUsage) as *mut TCHAR as LPTSTR,
        core::ptr::addr_of_mut!(lpUsage) as *mut TCHAR as LPTSTR,
    );
    printf(core::ptr::addr_of_mut!(lpUsage) as *mut TCHAR);
}

#[no_mangle]
pub unsafe extern "C" fn main(mut argc: i32, mut argv: *mut *mut i8) -> i32 {
    let mut opt: *mut i8;
    let mut needle: *mut i8 = core::ptr::null_mut();
    let mut ret: i32 = 0;
    let mut lpMessage: [TCHAR; 4096 as usize] = core::mem::zeroed();

    let mut invert_search: i32 = 0; /* flag to invert the search */
    let mut count_lines: i32 = 0; /* flag to whether/not count lines */
    let mut number_output: i32 = 0; /* flag to print line numbers */
    let mut ignore_case: i32 = 0; /* flag to be case insensitive */
    let mut at_start: i32 = 0; /* flag to Match if at the beginning of a line. */
    let mut at_end: i32 = 0; /* flag to Match if at the beginning of a line. */
    let mut reg_express: i32 = 0; /* flag to use/not use regular expressions */
    let mut exact_match: i32 = 0; /* flag to be exact match */
    let mut sub_dirs: i32 = 0; /* this and all subdirectories */
    let mut only_fname: i32 = 0; /* print only the name of the file*/
    let mut literal_search: i32 = 0;

    let mut pfile: *mut FILE;
    let mut hfind: intptr_t; // C `int` truncates _findfirst's pointer-sized handle on 64-bit
    let mut finddata: _finddata_t = core::mem::zeroed(); /* _findfirst, filenext block */

    /* Scan the command line */
    loop {
        argc -= 1;
        if !((argc != 0) && needle.is_null()) {
            break;
        }
        argv = argv.add(1);
        opt = *argv;
        if *opt == b'/' as i8 {
            match *opt.offset(1 as isize) as u8 {
                b'b' | b'B' => {
                    /* Matches pattern if at the beginning of a line */
                    at_start = 1;
                }
                b'e' | b'E' => {
                    /* matches pattern if at end of line */
                    at_end = 1;
                }
                b'i' | b'I' => {
                    /* Ignore */
                    ignore_case = 1;
                }
                b'm' | b'M' => {
                    /* only filename */
                    only_fname = 1;
                }
                b'n' | b'N' => {
                    /* Number */
                    number_output = 1;
                }
                b'r' | b'R' => {
                    /* search strings as regular expressions */
                    reg_express = 1;
                }
                b's' | b'S' => {
                    /* search files in child directory too*/
                    sub_dirs = 1;
                }
                b'v' | b'V' => {
                    /* Not with */
                    invert_search = 1;
                }
                b'x' | b'X' => {
                    /* exact match */
                    exact_match = 1;
                }
                _ => {
                    usage();
                    exit(2); /* syntax error .. return error 2 */
                }
            }
        } else {
            /* Get the string */
            if needle.is_null() {
                /* Assign the string to find */
                needle = *argv;
            }
        }
    }

    /* Check for search string */
    if needle.is_null() {
        /* No string? */
        usage();
        exit(1);
    }

    /* Scan the files for the string */
    if argc == 0 as i32 {
        ret = find_str(
            needle,
            __iob_func().add(0),
            invert_search,
            count_lines,
            number_output,
            ignore_case,
            at_start,
            literal_search,
            at_end,
            reg_express,
            exact_match,
            sub_dirs,
            only_fname,
        );
    }

    argc -= 1;
    while argc >= 0 as i32 {
        argv = argv.add(1);
        hfind = (_findfirst(*argv, core::ptr::addr_of_mut!(finddata))) as _;
        if hfind < 0 {
            /* We were not able to find a file. Display a message and
               set the exit status. */
            LoadString(
                GetModuleHandle(core::ptr::null_mut()),
                IDS_NO_SUCH_FILE,
                lpMessage.as_mut_ptr() as LPTSTR,
                4096,
            );
            CharToOem(lpMessage.as_mut_ptr(), lpMessage.as_mut_ptr());
            fprintf(__iob_func().add(2), lpMessage.as_mut_ptr(), *argv); //
        } else {
            /* repeat find next file to match the filemask */
            loop {
                /* We have found a file, so try to open it */
                pfile = fopen(finddata.name.as_ptr(), b"r\0".as_ptr() as *const i8);
                if !pfile.is_null() {
                    printf(
                        b"---------------- %s\n\0".as_ptr() as *const i8,
                        finddata.name.as_ptr(),
                    );
                    ret = find_str(
                        needle,
                        pfile,
                        invert_search,
                        count_lines,
                        number_output,
                        ignore_case,
                        at_start,
                        literal_search,
                        at_end,
                        reg_express,
                        exact_match,
                        sub_dirs,
                        only_fname,
                    );
                    fclose(pfile);
                } else {
                    LoadString(
                        GetModuleHandle(core::ptr::null_mut()),
                        IDS_CANNOT_OPEN,
                        lpMessage.as_mut_ptr() as LPTSTR,
                        4096,
                    );
                    CharToOem(lpMessage.as_mut_ptr(), lpMessage.as_mut_ptr());
                    fprintf(__iob_func().add(2), lpMessage.as_mut_ptr(), finddata.name.as_ptr());
                }
                if !(_findnext((hfind) as intptr_t, core::ptr::addr_of_mut!(finddata))
                    > 0 as i32)
                {
                    break;
                }
            }
        }
        _findclose((hfind) as _);
        argc -= 1;
    } /* for each argv */

    /* RETURN: If the string was found at least once, returns 0.
     * If the string was not found at all, returns 1.
     * (Note that find_str.c returns the exact opposite values.)
     */
    exit(if ret != 0 {
        0
    } else {
        1
    });
    #[allow(unreachable_code)]
    0
}
