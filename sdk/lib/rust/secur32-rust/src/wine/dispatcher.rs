use crate::prelude::*;
use core::ffi::{c_void, c_int, c_uint, c_char};

// ---- substrate for fork_helper (the __REACTOS__ CreateProcessA path) -------
// CRT pipe/fd helpers (cdecl → extern "C").
extern "C" {
    fn _pipe(pfds: *mut c_int, psize: c_uint, textmode: c_int) -> c_int;
    fn _get_osfhandle(fd: c_int) -> isize;
}
// kernel32 process/handle APIs (WINAPI → extern "system").
extern "system" {
    fn CreateProcessA(
        lpApplicationName: LPCSTR,
        lpCommandLine: LPSTR,
        lpProcessAttributes: *mut c_void,
        lpThreadAttributes: *mut c_void,
        bInheritHandles: BOOL,
        dwCreationFlags: DWORD,
        lpEnvironment: LPVOID,
        lpCurrentDirectory: LPCSTR,
        lpStartupInfo: *mut STARTUPINFOA,
        lpProcessInformation: *mut PROCESS_INFORMATION,
    ) -> BOOL;
    fn SetHandleInformation(hObject: HANDLE, dwMask: DWORD, dwFlags: DWORD) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
}
const _O_BINARY: c_int = 0x8000;
const _O_NOINHERIT: c_int = 0x0080;
const STARTF_USESTDHANDLES: DWORD = 0x0000_0100;
const HANDLE_FLAG_INHERIT: DWORD = 0x0000_0001;
const CREATE_NO_WINDOW: DWORD = 0x0800_0000;

// ---- file-scope preamble (translated) ----
pub const INITIAL_BUFFER_SIZE: i32 = 200;

// fork_helper — dropped by the batch translation; recovered faithfully from
// dispatcher.c:76-250 (__REACTOS__ branch: _pipe + CreateProcessA(ntlm_auth),
// the non-REACTOS fork()/execvp path is excluded by #ifdef). Called cross-file
// by ntlm.rs → must keep the `_fork_helper` cdecl symbol (extern "C").
#[no_mangle]
pub unsafe extern "C" fn fork_helper(
    new_helper: *mut PNegoHelper,
    prog: *mut SEC_CHAR,
    argv: *mut *mut SEC_CHAR,
) -> SECURITY_STATUS {
    let mut pipe_in: [c_int; 2] = [0; 2];
    let mut pipe_out: [c_int; 2] = [0; 2];
    let mut hPipe: HANDLE;
    let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
    let mut si: STARTUPINFOA = core::mem::zeroed();
    let cmdline: *mut c_char;
    let helper: PNegoHelper;

    let _ = prog;

    if _pipe(pipe_in.as_mut_ptr(), 0, _O_BINARY | _O_NOINHERIT) < 0 {
        return SEC_E_INTERNAL_ERROR;
    }
    if _pipe(pipe_out.as_mut_ptr(), 0, _O_BINARY | _O_NOINHERIT) < 0 {
        close(pipe_in[0]);
        close(pipe_in[1]);
        return SEC_E_INTERNAL_ERROR;
    }

    helper = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<NegoHelper>() as SIZE_T) as PNegoHelper;
    if helper.is_null() {
        close(pipe_in[0]);
        close(pipe_in[1]);
        close(pipe_out[0]);
        close(pipe_out[1]);
        return SEC_E_INSUFFICIENT_MEMORY;
    }

    cmdline = flatten_cmdline(prog, argv as *const *mut c_char);
    if cmdline.is_null() {
        close(pipe_in[0]);
        close(pipe_in[1]);
        close(pipe_out[0]);
        close(pipe_out[1]);
        HeapFree(GetProcessHeap(), 0, helper as LPVOID);
        return SEC_E_INSUFFICIENT_MEMORY;
    }

    RtlZeroMemory(&mut pi as *mut _ as PVOID, core::mem::size_of::<PROCESS_INFORMATION>() as SIZE_T);
    RtlZeroMemory(&mut si as *mut _ as PVOID, core::mem::size_of::<STARTUPINFOA>() as SIZE_T);
    si.cb = core::mem::size_of::<STARTUPINFOA>() as DWORD;
    si.dwFlags |= STARTF_USESTDHANDLES;

    // reading side of the pipe is STDIN for the child
    hPipe = _get_osfhandle(pipe_out[0]) as HANDLE;
    SetHandleInformation(hPipe, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
    si.hStdInput = hPipe;

    // writing side of the pipe is STDOUT/STDERR for the child
    hPipe = _get_osfhandle(pipe_in[1]) as HANDLE;
    SetHandleInformation(hPipe, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
    si.hStdOutput = hPipe;
    si.hStdError = hPipe;

    if CreateProcessA(
        core::ptr::null(),
        cmdline,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TRUE,
        CREATE_NO_WINDOW,
        core::ptr::null_mut(),
        core::ptr::null(),
        &mut si,
        &mut pi,
    ) == 0
    {
        (*helper).helper_pid = (-1isize) as HANDLE;
    } else {
        (*helper).helper_pid = pi.hProcess;
        CloseHandle(pi.hThread);
        CloseHandle(pi.hProcess);
    }

    HeapFree(GetProcessHeap(), 0, cmdline as LPVOID);

    if (*helper).helper_pid == (-1isize) as HANDLE {
        close(pipe_in[0]);
        close(pipe_in[1]);
        close(pipe_out[0]);
        close(pipe_out[1]);
        HeapFree(GetProcessHeap(), 0, helper as LPVOID);
        return SEC_E_INTERNAL_ERROR;
    }

    *new_helper = helper;
    (*helper).major = -1;
    (*helper).minor = -1;
    (*helper).micro = -1;
    (*helper).com_buf = core::ptr::null_mut();
    (*helper).com_buf_size = 0;
    (*helper).com_buf_offset = 0;
    (*helper).session_key = core::ptr::null_mut();
    (*helper).neg_flags = 0;
    (*helper).crypt.ntlm.a4i = core::ptr::null_mut();
    (*helper).crypt.ntlm2.send_a4i = core::ptr::null_mut();
    (*helper).crypt.ntlm2.recv_a4i = core::ptr::null_mut();
    (*helper).crypt.ntlm2.send_sign_key = core::ptr::null_mut();
    (*helper).crypt.ntlm2.send_seal_key = core::ptr::null_mut();
    (*helper).crypt.ntlm2.recv_sign_key = core::ptr::null_mut();
    (*helper).crypt.ntlm2.recv_seal_key = core::ptr::null_mut();
    (*helper).pipe_in = pipe_in[0]; // keep in(read)
    close(pipe_in[1]); // close in(write)
    (*helper).pipe_out = pipe_out[1]; // keep out(write)
    close(pipe_out[0]); // close out(read)

    SEC_E_OK
}

#[no_mangle]
pub unsafe extern "C" fn flatten_cmdline(prog: *const core::ffi::c_char, argv: *const *mut core::ffi::c_char) -> *mut core::ffi::c_char {
    let mut i: i32;
    let mut argstr_size: SIZE_T = 0;
    let argstr: *mut core::ffi::c_char;
    let mut p: *mut core::ffi::c_char;

    argstr_size += (strlen(prog) + 3) as SIZE_T;
    i = 0;
    while !(*argv.offset(i as isize)).is_null() {
        argstr_size += (strlen(*argv.offset(i as isize)) + 1) as SIZE_T;
        i += 1;
    }
    argstr = HeapAlloc(GetProcessHeap(), 0, (argstr_size + 1) * core::mem::size_of::<core::ffi::c_char>() as SIZE_T) as *mut core::ffi::c_char;
    if argstr.is_null() {
        ERR!(b"ERROR: Not enough memory\n\0".as_ptr() as *const core::ffi::c_char);
        return core::ptr::null_mut();
    }

    p = argstr;
    strcpy(p, b"\"\0".as_ptr() as *const core::ffi::c_char);
    strcat(p, prog);
    strcat(p, b"\" \0".as_ptr() as *const core::ffi::c_char);
    p = p.offset(strlen(p) as isize);
    i = 0;
    while !(*argv.offset(i as isize)).is_null() {
        strcpy(p, *argv.offset(i as isize));
        p = p.offset(strlen(*argv.offset(i as isize)) as isize);
        *p = b' ' as core::ffi::c_char;
        p = p.offset(1);
        i += 1;
    }
    *p = b'\0' as core::ffi::c_char;

    argstr
}

unsafe fn read_line(helper: PNegoHelper, offset_len: *mut core::ffi::c_int) -> SECURITY_STATUS {
    let mut newline: *mut core::ffi::c_char;
    let mut read_size: core::ffi::c_int;

    if (*helper).com_buf.is_null() {
        (*helper).com_buf = HeapAlloc(GetProcessHeap(), 0, INITIAL_BUFFER_SIZE as _) as *mut core::ffi::c_char;
        if (*helper).com_buf.is_null() {
            return SEC_E_INSUFFICIENT_MEMORY;
        }

        (*helper).com_buf_size = INITIAL_BUFFER_SIZE;
        (*helper).com_buf_offset = 0;
    }

    loop {
        if (*helper).com_buf_offset + INITIAL_BUFFER_SIZE > (*helper).com_buf_size {
            let buf = HeapReAlloc(GetProcessHeap(), 0, (*helper).com_buf as _,
                ((*helper).com_buf_size + INITIAL_BUFFER_SIZE) as _) as *mut core::ffi::c_char;
            if buf.is_null() {
                return SEC_E_INSUFFICIENT_MEMORY;
            }
            (*helper).com_buf_size += INITIAL_BUFFER_SIZE;
            (*helper).com_buf = buf;
        }

        read_size = crate::externs::read((*helper).pipe_in, (*helper).com_buf.add((*helper).com_buf_offset as usize) as _,
            ((*helper).com_buf_size - (*helper).com_buf_offset) as _) as _;
        if read_size <= 0 {
            return SEC_E_INTERNAL_ERROR;
        }

        (*helper).com_buf_offset += read_size;
        newline = memchr((*helper).com_buf as _, '\n' as core::ffi::c_int, (*helper).com_buf_offset as _) as *mut core::ffi::c_char;

        if !newline.is_null() {
            break;
        }
    }

    if newline != (*helper).com_buf.add((*helper).com_buf_offset as usize) {
        *offset_len = ((*helper).com_buf.add((*helper).com_buf_offset as usize) as isize - newline.add(1) as isize) as core::ffi::c_int;
    } else {
        *offset_len = 0;
    }

    *newline = 0;

    SEC_E_OK
}

unsafe fn preserve_unused(helper: PNegoHelper, offset_len: i32) -> SECURITY_STATUS {
    if offset_len > 0 {
        core::ptr::copy(
            (*helper).com_buf.offset((*helper).com_buf_offset as isize),
            (*helper).com_buf,
            offset_len as usize,
        );
        (*helper).com_buf_offset = offset_len;
    } else {
        (*helper).com_buf_offset = 0;
    }

    SEC_E_OK
}
#[no_mangle]
pub unsafe extern "C" fn run_helper(
    helper: PNegoHelper,
    buffer: *mut core::ffi::c_char,
    max_buflen: core::ffi::c_uint,
    buflen: *mut core::ffi::c_int,
) -> SECURITY_STATUS {
    let mut offset_len: core::ffi::c_int = 0;
    let mut sec_status: SECURITY_STATUS = SEC_E_OK;

    crate::externs::write((*helper).pipe_out, buffer as *const _, lstrlenA(buffer) as usize);
    crate::externs::write((*helper).pipe_out, b"\n\0".as_ptr() as *const _, 1);

    sec_status = read_line(helper, &mut offset_len);
    if sec_status != SEC_E_OK {
        return sec_status;
    }

    *buflen = lstrlenA((*helper).com_buf);

    if (*buflen as core::ffi::c_uint) > max_buflen {
        return SEC_E_BUFFER_TOO_SMALL;
    }

    if *buflen < 2 {
        return SEC_E_ILLEGAL_MESSAGE;
    }

    if strncmp((*helper).com_buf, b"ERR\0".as_ptr() as *const _, 3) == 0 {
        return SEC_E_INVALID_TOKEN;
    }

    memcpy(
        buffer as *mut _,
        (*helper).com_buf as *const _,
        (*buflen + 1) as usize,
    );

    sec_status = preserve_unused(helper, offset_len);

    sec_status
}
#[no_mangle]
pub unsafe extern "C" fn cleanup_helper(helper: PNegoHelper) {
    if helper.is_null() {
        return;
    }

    HeapFree(GetProcessHeap(), 0, (*helper).com_buf as *mut _);
    HeapFree(GetProcessHeap(), 0, (*helper).session_key as *mut _);

    close((*helper).pipe_out);
    close((*helper).pipe_in);

    HeapFree(GetProcessHeap(), 0, helper as *mut _);
}
#[no_mangle]
pub unsafe extern "C" fn check_version(helper: PNegoHelper) {
    let mut temp: [core::ffi::c_char; 80] = [0; 80];
    let mut major: core::ffi::c_int = 0;
    let mut minor: core::ffi::c_int = 0;
    let mut micro: core::ffi::c_int = 0;

    if !helper.is_null() {
        let len = read((*helper).pipe_in, temp.as_mut_ptr() as *mut _, (core::mem::size_of_val(&temp) - 1) as _);
        if len > 8 {
            let newline = memchr(temp.as_mut_ptr() as *mut _, b'\n' as core::ffi::c_int, len as _) as *mut core::ffi::c_char;
            if !newline.is_null() {
                *newline = 0;
            } else {
                temp[len as usize] = 0;
            }

            let ret = sscanf(
                temp.as_ptr(),
                b"Version %d.%d.%d\0".as_ptr() as *const core::ffi::c_char,
                core::ptr::addr_of_mut!(major),
                core::ptr::addr_of_mut!(minor),
                core::ptr::addr_of_mut!(micro),
            );
            if ret != 3 {
                (*helper).major = -1;
                (*helper).minor = -1;
                (*helper).micro = -1;
            } else {
                (*helper).major = major;
                (*helper).minor = minor;
                (*helper).micro = micro;
            }
        }
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Copyright 2005, 2006 Kai Blin
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  *
//  * A dispatcher to run ntlm_auth for wine's sspi module.
//  */
//
// #include "precomp.h"
//
// #include <stdio.h>
// #include <process.h>
// #include <io.h>
// #include <errno.h>
// #include <fcntl.h>
//
// #ifdef __REACTOS__
// #define close _close
// #define read  _read
// #define write _write
// #endif
//
// #include <wine/debug.h>
// WINE_DEFAULT_DEBUG_CHANNEL(ntlm);
//
// #define INITIAL_BUFFER_SIZE 200
//
// char* flatten_cmdline(const char *prog, char* const argv[])
// {
//     int i;
//     SIZE_T argstr_size = 0;
//     char *argstr, *p;
//
//     /* Compute space needed for the new string, and allocate it */
//     argstr_size += strlen(prog) + 3; // 3 == 2 quotes between 'prog', and 1 space
//     for(i = 0; argv[i] != NULL; ++i)
//     {
//         argstr_size += strlen(argv[i]) + 1; // 1 for space
//     }
//     argstr = HeapAlloc(GetProcessHeap(), 0, (argstr_size + 1) * sizeof(CHAR));
//     if (argstr == NULL)
//     {
//         ERR("ERROR: Not enough memory\n");
//         return NULL;
//     }
//
//     /* Copy the contents and NULL-terminate the string */
//     p = argstr;
//     strcpy(p, "\"");    // Open quote
//     strcat(p, prog);
//     strcat(p, "\" ");   // Close quote + space
//     p += strlen(p);
//     for(i = 0; argv[i] != NULL; ++i)
//     {
//         strcpy(p, argv[i]);
//         p += strlen(argv[i]);
//         *p++ = ' ';
//     }
//     *p = '\0';
//
//     return argstr;
// }
//
// SECURITY_STATUS fork_helper(PNegoHelper *new_helper, const char *prog,
//         char* const argv[])
// {
//     int pipe_in[2];
//     int pipe_out[2];
// #ifdef __REACTOS__
//     HANDLE hPipe;
//     PROCESS_INFORMATION pi;
//     STARTUPINFOA si;
//     char* cmdline;
// #endif
//     int i;
//     PNegoHelper helper;
//
//     TRACE("%s ", debugstr_a(prog));
//     for(i = 0; argv[i] != NULL; ++i)
//     {
//         TRACE("%s ", debugstr_a(argv[i]));
//     }
//     TRACE("\n");
//
// #ifndef __REACTOS__
//
// #ifdef HAVE_PIPE2
//     if (pipe2( pipe_in, O_CLOEXEC ) < 0 )
// #endif
//     {
//         if( pipe(pipe_in) < 0 ) return SEC_E_INTERNAL_ERROR;
//         fcntl( pipe_in[0], F_SETFD, FD_CLOEXEC );
//         fcntl( pipe_in[1], F_SETFD, FD_CLOEXEC );
//     }
// #ifdef HAVE_PIPE2
//     if (pipe2( pipe_out, O_CLOEXEC ) < 0 )
// #endif
//     {
//         if( pipe(pipe_out) < 0 )
//         {
//             close(pipe_in[0]);
//             close(pipe_in[1]);
//             return SEC_E_INTERNAL_ERROR;
//         }
//         fcntl( pipe_out[0], F_SETFD, FD_CLOEXEC );
//         fcntl( pipe_out[1], F_SETFD, FD_CLOEXEC );
//     }
//
// #else
//
//     if (_pipe(pipe_in, 0, _O_BINARY /* _O_TEXT */ | _O_NOINHERIT) < 0)
//     {
//         return SEC_E_INTERNAL_ERROR;
//     }
//
//     if (_pipe(pipe_out, 0, _O_BINARY /* _O_TEXT */ | _O_NOINHERIT) < 0)
//     {
//         close(pipe_in[0]);
//         close(pipe_in[1]);
//         return SEC_E_INTERNAL_ERROR;
//     }
//
// #endif
//
//     if (!(helper = HeapAlloc(GetProcessHeap(),0, sizeof(NegoHelper))))
//     {
//         close(pipe_in[0]);
//         close(pipe_in[1]);
//         close(pipe_out[0]);
//         close(pipe_out[1]);
//         return SEC_E_INSUFFICIENT_MEMORY;
//     }
//
// #ifndef __REACTOS__
//     helper->helper_pid = fork();
// #else
//
//     cmdline = flatten_cmdline(prog, argv);
//     if (!cmdline)
//     {
//         close(pipe_in[0]);
//         close(pipe_in[1]);
//         close(pipe_out[0]);
//         close(pipe_out[1]);
//         HeapFree( GetProcessHeap(), 0, helper );
//         return SEC_E_INSUFFICIENT_MEMORY;
//     }
//
//     ZeroMemory(&pi, sizeof(pi));
//     ZeroMemory(&si, sizeof(si));
//     si.cb = sizeof(si);
//
//     si.dwFlags |= STARTF_USESTDHANDLES;
//
//     /* The reading side of the pipe is STDIN for this process */
//     hPipe = (HANDLE)_get_osfhandle(pipe_out[0]);
//     SetHandleInformation(hPipe, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
//     si.hStdInput = hPipe;
//
//     /* The writing side of the pipe is STDOUT for this process */
//     hPipe = (HANDLE)_get_osfhandle(pipe_in[1]);
//     SetHandleInformation(hPipe, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT);
//     si.hStdOutput = hPipe;
//     si.hStdError  = hPipe;
//
//     if (!CreateProcessA(NULL, cmdline, NULL, NULL, TRUE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi))
//     {
//         /* We fail just afterwards */
//         helper->helper_pid = (HANDLE)-1;
//     }
//     else
//     {
//         helper->helper_pid = pi.hProcess;
//         CloseHandle(pi.hThread);
//         CloseHandle(pi.hProcess);
//     }
//
//     HeapFree( GetProcessHeap(), 0, cmdline );
//
// #endif
//
// #ifndef __REACTOS__
//     if(helper->helper_pid == -1)
// #else
//     if(helper->helper_pid == (HANDLE)-1)
// #endif
//     {
//         close(pipe_in[0]);
//         close(pipe_in[1]);
//         close(pipe_out[0]);
//         close(pipe_out[1]);
//         HeapFree( GetProcessHeap(), 0, helper );
//         return SEC_E_INTERNAL_ERROR;
//     }
//
// #ifndef __REACTOS__
//     if(helper->helper_pid == 0)
//     {
//         /* We're in the child now */
//         dup2(pipe_out[0], 0);
//         close(pipe_out[0]);
//         close(pipe_out[1]);
//
//         dup2(pipe_in[1], 1);
//         close(pipe_in[0]);
//         close(pipe_in[1]);
//
//         execvp(prog, argv);
//
//         /* Whoops, we shouldn't get here. Big badaboom.*/
//         write(STDOUT_FILENO, "BH\n", 3);
//         _exit(1);
//     }
//     else
// #endif
//     {
//         *new_helper = helper;
//         helper->major = helper->minor = helper->micro = -1;
//         helper->com_buf = NULL;
//         helper->com_buf_size = 0;
//         helper->com_buf_offset = 0;
//         helper->session_key = NULL;
//         helper->neg_flags = 0;
//         helper->crypt.ntlm.a4i = NULL;
//         helper->crypt.ntlm2.send_a4i = NULL;
//         helper->crypt.ntlm2.recv_a4i = NULL;
//         helper->crypt.ntlm2.send_sign_key = NULL;
//         helper->crypt.ntlm2.send_seal_key = NULL;
//         helper->crypt.ntlm2.recv_sign_key = NULL;
//         helper->crypt.ntlm2.recv_seal_key = NULL;
//         helper->pipe_in = pipe_in[0];   // Keep in(read)
//         close(pipe_in[1]);              // Close in(write)
//         helper->pipe_out = pipe_out[1]; // Keep out(write)
//         close(pipe_out[0]);             // Close out(read)
//     }
//
//     return SEC_E_OK;
// }
//
// static SECURITY_STATUS read_line(PNegoHelper helper, int *offset_len)
// {
//     char *newline;
//     int read_size;
//     
//     if(helper->com_buf == NULL)
//     {
//         TRACE("Creating a new buffer for the helper\n");
//         if((helper->com_buf = HeapAlloc(GetProcessHeap(), 0, INITIAL_BUFFER_SIZE)) == NULL)
//             return SEC_E_INSUFFICIENT_MEMORY;
//         
//         /* Created a new buffer, size is INITIAL_BUFFER_SIZE, offset is 0 */
//         helper->com_buf_size = INITIAL_BUFFER_SIZE;
//         helper->com_buf_offset = 0;
//     }
//
//     do
//     {
//         TRACE("offset = %d, size = %d\n", helper->com_buf_offset, helper->com_buf_size);
//         if(helper->com_buf_offset + INITIAL_BUFFER_SIZE > helper->com_buf_size)
//         {
//             /* increment buffer size in INITIAL_BUFFER_SIZE steps */
//             char *buf = HeapReAlloc(GetProcessHeap(), 0, helper->com_buf,
//                                     helper->com_buf_size + INITIAL_BUFFER_SIZE);
//             TRACE("Resizing buffer!\n");
//             if (!buf) return SEC_E_INSUFFICIENT_MEMORY;
//             helper->com_buf_size += INITIAL_BUFFER_SIZE;
//             helper->com_buf = buf;
//         }
//         if((read_size = read(helper->pipe_in, helper->com_buf + helper->com_buf_offset,
//                     helper->com_buf_size - helper->com_buf_offset)) <= 0)
//         {
//             return SEC_E_INTERNAL_ERROR;
//         }
//         
//         TRACE("read_size = %d, read: %s\n", read_size, 
//                 debugstr_a(helper->com_buf + helper->com_buf_offset));
//         helper->com_buf_offset += read_size;
//         newline = memchr(helper->com_buf, '\n', helper->com_buf_offset);
//     }while(newline == NULL);
//
//     /* Now, if there's a newline character, and we read more than that newline,
//      * we have to store the offset so we can preserve the additional data.*/
//     if( newline != helper->com_buf + helper->com_buf_offset)
//     {
//         TRACE("offset_len is calculated from %p - %p\n", 
//                 (helper->com_buf + helper->com_buf_offset), newline+1);
//         /* the length of the offset is the number of chars after the newline */
//         *offset_len = (helper->com_buf + helper->com_buf_offset) - (newline + 1);
//     }
//     else
//     {
//         *offset_len = 0;
//     }
//     
//     *newline = '\0';
//
//     return SEC_E_OK;
// }
//
// static SECURITY_STATUS preserve_unused(PNegoHelper helper, int offset_len)
// {
//     TRACE("offset_len = %d\n", offset_len);
//
//     if(offset_len > 0)
//     {
//         memmove(helper->com_buf, helper->com_buf + helper->com_buf_offset, 
//                 offset_len);
//         helper->com_buf_offset = offset_len;
//     }
//     else
//     {
//         helper->com_buf_offset = 0;
//     }
//
//     TRACE("helper->com_buf_offset was set to: %d\n", helper->com_buf_offset);
//     return SEC_E_OK;
// }
//
// SECURITY_STATUS run_helper(PNegoHelper helper, char *buffer,
//         unsigned int max_buflen, int *buflen)
// {
//     int offset_len;
//     SECURITY_STATUS sec_status = SEC_E_OK;
//
//     TRACE("In helper: sending %s\n", debugstr_a(buffer));
//
//     /* buffer + '\n' */
//     write(helper->pipe_out, buffer, lstrlenA(buffer));
//     write(helper->pipe_out, "\n", 1);
//
//     if((sec_status = read_line(helper, &offset_len)) != SEC_E_OK)
//     {
//         return sec_status;
//     }
//     
//     TRACE("In helper: received %s\n", debugstr_a(helper->com_buf));
//     *buflen = lstrlenA(helper->com_buf);
//
//     if( *buflen > max_buflen)
//     {   
//         ERR("Buffer size too small(%d given, %d required) dropping data!\n",
//                 max_buflen, *buflen);
//         return SEC_E_BUFFER_TOO_SMALL;
//     }
//
//     if( *buflen < 2 )
//     {
//         return SEC_E_ILLEGAL_MESSAGE;
//     }
//
//     /* We only get ERR if the input size is too big. On a GENSEC error,
//      * ntlm_auth will return BH */
//     if(strncmp(helper->com_buf, "ERR", 3) == 0)
//     {
//         return SEC_E_INVALID_TOKEN;
//     }
//
//     memcpy(buffer, helper->com_buf, *buflen+1);
//
//     sec_status = preserve_unused(helper, offset_len);
//     
//     return sec_status;
// }
//
// void cleanup_helper(PNegoHelper helper)
// {
//
//     TRACE("Killing helper %p\n", helper);
//     if(helper == NULL)
//         return;
//
//     HeapFree(GetProcessHeap(), 0, helper->com_buf);
//     HeapFree(GetProcessHeap(), 0, helper->session_key);
//
//     /* closing stdin will terminate ntlm_auth */
//     close(helper->pipe_out);
//     close(helper->pipe_in);
//
// #ifndef __REACTOS__
//
// #ifdef HAVE_FORK
//     if (helper->helper_pid > 0) /* reap child */
//     {
//         pid_t wret;
//         do {
//             wret = waitpid(helper->helper_pid, NULL, 0);
//         } while (wret < 0 && errno == EINTR);
//     }
// #endif
//
// #endif
//
//     HeapFree(GetProcessHeap(), 0, helper);
// }
//
// void check_version(PNegoHelper helper)
// {
//     char temp[80];
//     char *newline;
//     int major = 0, minor = 0, micro = 0, ret;
//
//     TRACE("Checking version of helper\n");
//     if(helper != NULL)
//     {
//         int len = read(helper->pipe_in, temp, sizeof(temp)-1);
//         if (len > 8)
//         {
//             if((newline = memchr(temp, '\n', len)) != NULL)
//                 *newline = '\0';
//             else
//                 temp[len] = 0;
//
//             TRACE("Exact version is %s\n", debugstr_a(temp));
//             ret = sscanf(temp, "Version %d.%d.%d", &major, &minor, &micro);
//             if(ret != 3)
//             {
//                 ERR("Failed to get the helper version.\n");
//                 helper->major = helper->minor = helper->micro = -1;
//             }
//             else
//             {
//                 TRACE("Version recognized: %d.%d.%d\n", major, minor, micro);
//                 helper->major = major;
//                 helper->minor = minor;
//                 helper->micro = micro;
//             }
//         }
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
