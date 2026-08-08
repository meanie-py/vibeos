// rustconv-assembly: whole-tu
use crate::prelude::*;
use msvcrt_sys::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _beginthread_trampoline_t {
    pub thread: HANDLE,
    /// C spells this as an ANONYMOUS union, so the C field names are `start_address` /
    /// `start_address_ex` with no qualifier. Rust has no anonymous unions, so the two
    /// arms live behind this named field; it is purely a spelling difference and does
    /// not change the layout the trampoline `memcpy`s.
    pub start: _beginthread_trampoline_start,
    pub arglist: *mut core::ffi::c_void,
}

#[repr(C)]
pub union _beginthread_trampoline_start {
    /// `void (__cdecl *)(void *)` — `_beginthread`'s entry, which returns nothing.
    pub start_address: _beginthread_start_routine_t,
    /// `unsigned int (__stdcall *)(void *)` — `_beginthreadex`'s, which returns the
    /// thread exit code. Different return type AND different convention on i386; the
    /// union is why one struct serves both.
    pub start_address_ex: _beginthreadex_start_routine_t,
}

pub type P_beginthread_trampoline_t = *mut _beginthread_trampoline_t;
#[no_mangle]
pub unsafe extern "C" fn msvcrt_get_thread_data() -> *mut thread_data_t {
    let mut ptr: *mut thread_data_t;
    let err = GetLastError();

    ptr = TlsGetValue(msvcrt_tls_index) as *mut thread_data_t;
    if ptr.is_null() {
        ptr = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<thread_data_t>() as SIZE_T,
        ) as *mut thread_data_t;
        if ptr.is_null() {
            _amsg_exit(_RT_THREAD as c_int);
        }
        if TlsSetValue(msvcrt_tls_index, ptr as LPVOID) == 0 {
            _amsg_exit(_RT_THREAD as c_int);
        }
        (*ptr).tid = GetCurrentThreadId();
        (*ptr).handle = INVALID_HANDLE_VALUE;
        (*ptr).random_seed = 1;
        if !MSVCRT_locale.is_null() {
            (*ptr).locinfo = (*MSVCRT_locale).locinfo;
            (*ptr).mbcinfo = (*MSVCRT_locale).mbcinfo;
        }
        (*ptr).cached_locale[0] = b'C' as i8;
        (*ptr).cached_locale[1] = 0;
    }
    SetLastError(err);
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn _endthread() {
    let tls: *mut thread_data_t;

    tls = TlsGetValue(msvcrt_tls_index) as *mut thread_data_t;
    if !tls.is_null() && (*tls).handle != INVALID_HANDLE_VALUE {
        CloseHandle((*tls).handle);
        (*tls).handle = INVALID_HANDLE_VALUE;
    }
    // C's `else` arm is a bare WARN(), which this build compiles away.

    _endthreadex(0);
}

#[no_mangle]
pub unsafe extern "C" fn _endthreadex(retval: u32) {
    // TRACE("(%d)\n", retval);
    ExitThread(retval);
}
// `DWORD CALLBACK` — CALLBACK is WINAPI, which is a real convention only on i386.
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn _beginthread_trampoline(arg: LPVOID) -> DWORD {
    _beginthread_trampoline_body(arg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn _beginthread_trampoline(arg: LPVOID) -> DWORD {
    _beginthread_trampoline_body(arg)
}

/// The body is shared by the two convention arms above so there is exactly one copy to
/// read. It arrived from the translator with its call set PERMUTED —
/// `msvcrt_get_thread_data` standing in for the `memcpy`, `memcpy` for the `free`, and
/// `free(local_trampoline.arglist)` in place of the `start_address(arglist)` call that
/// is the entire point of the function. Every call in the C appeared somewhere, which is
/// all `--seal-calls` checks; the thread would have leaked its argument block and then
/// never run the user's entry point.
#[inline]
unsafe fn _beginthread_trampoline_body(arg: LPVOID) -> DWORD {
    let mut local_trampoline: _beginthread_trampoline_t = core::mem::zeroed();
    let data = msvcrt_get_thread_data();

    memcpy(
        &mut local_trampoline as *mut _ as *mut core::ffi::c_void,
        arg,
        core::mem::size_of::<_beginthread_trampoline_t>(),
    );
    free(arg);
    (*data).handle = local_trampoline.thread;

    (local_trampoline.start.start_address.unwrap_unchecked())(local_trampoline.arglist);
    _endthread();
    0
}

#[no_mangle]
pub unsafe extern "C" fn _beginthread(
    start_address: _beginthread_start_routine_t,
    stack_size: u32,
    arglist: *mut core::ffi::c_void,
) -> usize {
    let trampoline: *mut _beginthread_trampoline_t;
    let thread: HANDLE;

    if MSVCRT_CHECK_PMT!(start_address.is_some()) == 0 {
        return -1isize as usize;
    }

    trampoline =
        malloc(core::mem::size_of::<_beginthread_trampoline_t>()) as *mut _beginthread_trampoline_t;
    if trampoline.is_null() {
        *_errno() = EAGAIN as c_int;
        return -1isize as usize;
    }

    thread = CreateThread(
        core::ptr::null_mut(),
        stack_size,
        Some(_beginthread_trampoline),
        trampoline as PVOID,
        CREATE_SUSPENDED,
        core::ptr::null_mut(),
    );
    if thread.is_null() {
        free(trampoline as *mut core::ffi::c_void);
        msvcrt_set_errno(GetLastError() as c_int);
        return -1isize as usize;
    }

    (*trampoline).thread = thread;
    (*trampoline).start.start_address = start_address;
    (*trampoline).arglist = arglist;

    // C compares a DWORD against `-1`, which converts to 0xFFFFFFFF.
    if ResumeThread(thread) == -1i32 as DWORD {
        free(trampoline as *mut core::ffi::c_void);
        *_errno() = EAGAIN as c_int;
        return -1isize as usize;
    }

    thread as usize
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn _beginthreadex_trampoline(arg: LPVOID) -> DWORD {
    _beginthreadex_trampoline_body(arg)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn _beginthreadex_trampoline(arg: LPVOID) -> DWORD {
    _beginthreadex_trampoline_body(arg)
}

/// Body shared by both convention arms — see `_beginthread_trampoline_body`. Splitting a
/// dual into two full copies means only one is ever compiled, so a later hand-edit to the
/// wrong twin is invisible to `cargo check` on every target that does not select it.
#[inline]
unsafe fn _beginthreadex_trampoline_body(arg: LPVOID) -> DWORD {
    let retval: core::ffi::c_uint;
    let mut local_trampoline: _beginthread_trampoline_t = core::mem::zeroed();
    let data: *mut thread_data_t = msvcrt_get_thread_data();

    memcpy(
        &mut local_trampoline as *mut _ as *mut core::ffi::c_void,
        arg,
        core::mem::size_of::<_beginthread_trampoline_t>(),
    );
    free(arg);
    (*data).handle = local_trampoline.thread;

    retval = (local_trampoline.start.start_address_ex.unwrap_unchecked())(local_trampoline.arglist);
    _endthreadex(retval);
    0
}

#[no_mangle]
pub unsafe extern "C" fn _beginthreadex(
    security: *mut core::ffi::c_void,
    stack_size: core::ffi::c_uint,
    start_address: _beginthreadex_start_routine_t,
    arglist: *mut core::ffi::c_void,
    initflag: core::ffi::c_uint,
    thrdaddr: *mut core::ffi::c_uint,
) -> usize {
    let trampoline: *mut _beginthread_trampoline_t;
    let thread: HANDLE;

    if MSVCRT_CHECK_PMT!(start_address.is_some()) == 0 {
        return 0;
    }

    trampoline =
        malloc(core::mem::size_of::<_beginthread_trampoline_t>()) as *mut _beginthread_trampoline_t;
    if trampoline.is_null() {
        return 0;
    }

    (*trampoline).thread = INVALID_HANDLE_VALUE;
    (*trampoline).start.start_address_ex = start_address;
    (*trampoline).arglist = arglist;

    thread = CreateThread(
        security as LPSECURITY_ATTRIBUTES,
        stack_size,
        Some(_beginthreadex_trampoline),
        trampoline as PVOID,
        initflag,
        thrdaddr as PDWORD,
    );
    if thread.is_null() {
        free(trampoline as *mut core::ffi::c_void);
        msvcrt_set_errno(GetLastError() as c_int);
        return 0;
    }

    thread as usize
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * msvcrt.dll thread functions
//  *
//  * Copyright 2000 Jon Griffiths
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
//  */
// #include <process.h>
// #include "msvcrt.h"
// #include "wine/debug.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(msvcrt);
//
// /********************************************************************/
//
// typedef struct {
//   HANDLE thread;
//   union {
//       _beginthread_start_routine_t start_address;
//       _beginthreadex_start_routine_t start_address_ex;
//   };
//   void *arglist;
// } _beginthread_trampoline_t;
//
// /*********************************************************************
//  *		msvcrt_get_thread_data
//  *
//  * Return the thread local storage structure.
//  */
// thread_data_t * msvcrt_get_thread_data(void)
// {
//     thread_data_t *ptr;
//     DWORD err = GetLastError();  /* need to preserve last error */
//
//     if (!(ptr = TlsGetValue( msvcrt_tls_index )))
//     {
//         if (!(ptr = HeapAlloc( GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(*ptr) )))
//             _amsg_exit( _RT_THREAD );
//         if (!TlsSetValue( msvcrt_tls_index, ptr )) _amsg_exit( _RT_THREAD );
//         ptr->tid = GetCurrentThreadId();
//         ptr->handle = INVALID_HANDLE_VALUE;
//         ptr->random_seed = 1;
//         if (MSVCRT_locale)
//         {
//             ptr->locinfo = MSVCRT_locale->locinfo;
//             ptr->mbcinfo = MSVCRT_locale->mbcinfo;
//         }
//         ptr->cached_locale[0] = 'C';
//         ptr->cached_locale[1] = 0;
//     }
//     SetLastError( err );
//     return ptr;
// }
//
// /*********************************************************************
//  *		_endthread (MSVCRT.@)
//  */
// void CDECL _endthread(void)
// {
//   thread_data_t *tls;
//
//   TRACE("(void)\n");
//
//   tls = TlsGetValue(msvcrt_tls_index);
//   if (tls && tls->handle != INVALID_HANDLE_VALUE)
//   {
//       CloseHandle(tls->handle);
//       tls->handle = INVALID_HANDLE_VALUE;
//   } else
//       WARN("tls=%p tls->handle=%p\n", tls, tls ? tls->handle : INVALID_HANDLE_VALUE);
//
//   _endthreadex(0);
// }
//
// /*********************************************************************
//  *		_endthreadex (MSVCRT.@)
//  */
// void CDECL _endthreadex(
//   unsigned int retval) /* [in] Thread exit code */
// {
//   TRACE("(%d)\n", retval);
//
//
//   ExitThread(retval);
// }
//
// /*********************************************************************
//  *		_beginthread_trampoline
//  */
// static DWORD __stdcall _beginthread_trampoline(LPVOID arg)
// {
//     _beginthread_trampoline_t local_trampoline;
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     memcpy(&local_trampoline,arg,sizeof(local_trampoline));
//     free(arg);
//     data->handle = local_trampoline.thread;
//
//     local_trampoline.start_address(local_trampoline.arglist);
//     _endthread();
// }
//
// /*********************************************************************
//  *		_beginthread (MSVCRT.@)
//  */
// uintptr_t CDECL _beginthread(
//   _beginthread_start_routine_t start_address, /* [in] Start address of routine that begins execution of new thread */
//   unsigned int stack_size, /* [in] Stack size for new thread or 0 */
//   void *arglist)           /* [in] Argument list to be passed to new thread or NULL */
// {
//   _beginthread_trampoline_t* trampoline;
//   HANDLE thread;
//
//   TRACE("(%p, %d, %p)\n", start_address, stack_size, arglist);
//
//   if (!MSVCRT_CHECK_PMT(start_address)) return -1;
//
//   trampoline = malloc(sizeof(*trampoline));
//   if(!trampoline) {
//       *_errno() = EAGAIN as c_int;
//       return -1;
//   }
//
//   thread = CreateThread(NULL, stack_size, _beginthread_trampoline,
//           trampoline, CREATE_SUSPENDED, NULL);
//   if(!thread) {
//       free(trampoline);
//       msvcrt_set_errno(GetLastError() as c_int);
//       return -1;
//   }
//
//   trampoline->thread = thread;
//   trampoline->start_address = start_address;
//   trampoline->arglist = arglist;
//
//
//   if(ResumeThread(thread) == -1) {
//       free(trampoline);
//       *_errno() = EAGAIN as c_int;
//       return -1;
//   }
//
//   return (uintptr_t)thread;
// }
//
// /*********************************************************************
//  *		_beginthreadex_trampoline
//  */
// static DWORD __stdcall _beginthreadex_trampoline(LPVOID arg)
// {
//     unsigned int retval;
//     _beginthread_trampoline_t local_trampoline;
//     thread_data_t *data = msvcrt_get_thread_data();
//
//     memcpy(&local_trampoline, arg, sizeof(local_trampoline));
//     free(arg);
//     data->handle = local_trampoline.thread;
//
//     retval = local_trampoline.start_address_ex(local_trampoline.arglist);
//     _endthreadex(retval);
// }
// /*********************************************************************
//  *		_beginthreadex (MSVCRT.@)
//  */
// uintptr_t CDECL _beginthreadex(
//   void *security,          /* [in] Security descriptor for new thread; must be NULL for Windows 9x applications */
//   unsigned int stack_size, /* [in] Stack size for new thread or 0 */
//   _beginthreadex_start_routine_t start_address, /* [in] Start address of routine that begins execution of new thread */
//   void *arglist,           /* [in] Argument list to be passed to new thread or NULL */
//   unsigned int initflag,   /* [in] Initial state of new thread (0 for running or CREATE_SUSPEND for suspended) */
//   unsigned int *thrdaddr)  /* [out] Points to a 32-bit variable that receives the thread identifier */
// {
//   _beginthread_trampoline_t* trampoline;
//   HANDLE thread;
//
//   TRACE("(%p, %d, %p, %p, %d, %p)\n", security, stack_size, start_address, arglist, initflag, thrdaddr);
//
//   /* FIXME: may use different errno / return values */
//   if (!MSVCRT_CHECK_PMT(start_address)) return 0;
//
//   if (!(trampoline = malloc(sizeof(*trampoline))))
//       return 0;
//
//   trampoline->thread = INVALID_HANDLE_VALUE;
//   trampoline->start_address_ex = start_address;
//   trampoline->arglist = arglist;
//
//
//   thread = CreateThread(security, stack_size, _beginthreadex_trampoline,
//           trampoline, initflag, (DWORD *)thrdaddr);
//   if(!thread) {
//       free(trampoline);
//       msvcrt_set_errno(GetLastError() as c_int);
//       return 0;
//   }
//
//   return (uintptr_t)thread;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
