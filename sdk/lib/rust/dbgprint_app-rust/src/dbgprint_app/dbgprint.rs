use ntdll_sys::nt::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn _tmain(argc: i32, argv: *mut *mut TCHAR) -> i32 {
    let mut buf: *mut TCHAR = core::ptr::null_mut();
    let mut bufsize: usize;
    let mut i: i32;
    let mut offset: usize;

    bufsize = 0 as usize;
    i = 1 as i32;
    while i < argc {
        bufsize += (_tcslen(*argv.add(i as usize) as *const TCHAR) as usize)
            + (1 as usize);
        i += 1;
    }

    if bufsize == 0 {
        return -(1 as i32);
    }

    if (_tcsstr(
        *argv.add(1 as usize) as *const TCHAR,
        b"--winetest\0".as_ptr() as *const TCHAR,
    ) != core::ptr::null_mut())
        && (argc == (3 as i32))
    {
        let mut psBuffer: [core::ffi::c_char; 128 as usize] =
            [0; 128 as usize];
        let mut psBuffer2: [core::ffi::c_char; 128 as usize] =
            [0; 128 as usize];
        let mut nlptr2: *mut core::ffi::c_char = core::ptr::null_mut();
        let mut cmd: [core::ffi::c_char; 255 as usize] =
            [0; 255 as usize];
        let mut test: [core::ffi::c_char; 300 as usize] =
            [0; 300 as usize];
        let mut pPipe: *mut FILE = core::ptr::null_mut();
        let mut pPipe2: *mut FILE = core::ptr::null_mut();

        /* get available tests */
        strcpy(
            cmd.as_mut_ptr(),
            *argv.add(2 as usize) as *const core::ffi::c_char,
        );
        strcat(cmd.as_mut_ptr(), b" --list\0".as_ptr() as *const core::ffi::c_char);
        pPipe = _tpopen(
            cmd.as_mut_ptr(),
            b"r\0".as_ptr() as *const core::ffi::c_char,
        ) as *mut FILE;
        if !pPipe.is_null() {
            while !fgets(
                psBuffer.as_mut_ptr(),
                128 as core::ffi::c_int,
                pPipe,
            )
            .is_null()
            {
                if psBuffer[0 as usize] == b' ' as core::ffi::c_char {
                    strcpy(
                        cmd.as_mut_ptr(),
                        *argv.add(2 as usize) as *const core::ffi::c_char,
                    );
                    strcat(
                        cmd.as_mut_ptr(),
                        b" \0".as_ptr() as *const core::ffi::c_char,
                    );
                    strcat(
                        cmd.as_mut_ptr(),
                        psBuffer.as_mut_ptr().add(4 as usize),
                    );
                    /* run the current test */
                    strcpy(
                        test.as_mut_ptr(),
                        b"\n\nRunning \0".as_ptr() as *const core::ffi::c_char,
                    );
                    strcat(test.as_mut_ptr(), cmd.as_mut_ptr());
                    OutputDebugStringA(test.as_mut_ptr());
                    pPipe2 = _popen(
                        cmd.as_mut_ptr(),
                        b"r\0".as_ptr() as *const core::ffi::c_char,
                    ) as *mut FILE;
                    if !pPipe2.is_null() {
                        while !fgets(
                            psBuffer2.as_mut_ptr(),
                            128 as core::ffi::c_int,
                            pPipe2,
                        )
                        .is_null()
                        {
                            nlptr2 = strchr(
                                psBuffer2.as_mut_ptr(),
                                b'\n' as core::ffi::c_int,
                            );
                            if !nlptr2.is_null() {
                                *nlptr2 = b'\0' as core::ffi::c_char;
                            }
                            puts(psBuffer2.as_mut_ptr());
                            if !nlptr2.is_null() {
                                *nlptr2 = b'\n' as core::ffi::c_char;
                            }
                            OutputDebugStringA(psBuffer2.as_mut_ptr());
                        }
                        _pclose(pPipe2);
                    }
                }
            }
            _pclose(pPipe);
        }
    } else if (_tcsstr(
        *argv.add(1 as usize) as *const TCHAR,
        b"--process\0".as_ptr() as *const TCHAR,
    ) != core::ptr::null_mut())
        && (argc == (3 as i32))
    {
        let mut psBuffer: [core::ffi::c_char; 128 as usize] =
            [0; 128 as usize];
        let mut pPipe: *mut FILE = core::ptr::null_mut();

        pPipe = _tpopen(
            *argv.add(2 as usize) as *const TCHAR,
            b"r\0".as_ptr() as *const TCHAR,
        ) as *mut FILE;
        if !pPipe.is_null() {
            while !fgets(
                psBuffer.as_mut_ptr(),
                128 as core::ffi::c_int,
                pPipe,
            )
            .is_null()
            {
                puts(psBuffer.as_mut_ptr());
                OutputDebugStringA(psBuffer.as_mut_ptr());
            }
            _pclose(pPipe);
        }
    } else {
        buf = HeapAlloc(
            GetProcessHeap(),
            0 as u32,
            ((bufsize + (1 as usize))
                * core::mem::size_of::<TCHAR>()) as usize,
        ) as *mut TCHAR;
        if buf.is_null() {
            return -(1 as i32);
        }

        offset = 0 as usize;
        i = 1 as i32;
        while i < argc {
            let length: usize =
                _tcslen(*argv.add(i as usize) as *const TCHAR) as usize;
            _tcsncpy(
                buf.add(offset),
                *argv.add(i as usize) as *const TCHAR,
                length,
            );
            offset += length;
            if (i + (1 as i32)) < argc {
                *buf.add(offset) = _T(' ');
            } else {
                *buf.add(offset) = _T('\n');
                *buf.add(offset + (1 as usize)) = _T('\0');
            }
            offset += 1;
            i += 1;
        }
        _putts(buf);
        OutputDebugString(buf);
        HeapFree(GetProcessHeap(), 0 as u32, buf as _);
    }
    return 0 as i32;
}
