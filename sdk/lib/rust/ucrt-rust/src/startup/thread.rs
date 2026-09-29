//! `startup/thread.cpp` -- `_beginthread`, `_beginthreadex`, `_endthread`,
//! `_endthreadex`, and the thread entry trampoline behind both.
//!
//! Details that are silent if wrong:
//! * Handle ownership: `_beginthread` stores the thread handle in the thread's
//!   parameter block, and the thread CLOSES it when it ends (so the value
//!   `_beginthread` returns may already be dead); `_beginthreadex` never
//!   stores it, and the caller must `CloseHandle` it.
//! * Failure values: `_beginthread` returns `-1` (`INVALID_HANDLE_VALUE`),
//!   `_beginthreadex` returns `0`. A NULL procedure is EINVAL + the
//!   invalid-parameter handler; a `CreateThread`/`ResumeThread` failure maps
//!   `GetLastError()` into errno BEFORE the parameter block is released.
//! * `_beginthread` creates the thread SUSPENDED, records the handle, then
//!   resumes it; `_beginthreadex` passes the caller's creation flags straight
//!   through and writes the thread id only on success and only if asked.
//! * The parameter block (`__acrt_thread_parameter`) takes a module
//!   reference on the procedure's module (`GetModuleHandleExW(FROM_ADDRESS)`);
//!   a thread that ends through the CRT releases it with
//!   `FreeLibraryAndExitThread`. The block itself is freed with the thread's
//!   ptd (`__acrt_freeptd` -> `destroy_ptd` frees `_beginthread_context`), not
//!   here; on a failed create it is released here (handle, module, memory).
//! * The trampoline stores the block in the new thread's ptd
//!   (`__acrt_getptd()`, which allocates it), RoInitializes into the MTA when
//!   the begin-thread policy says so, calls the procedure (`__cdecl void(void*)`
//!   for `_beginthread`, `__stdcall unsigned(void*)` for `_beginthreadex`) and
//!   ends via `_endthreadex(result)` / `_endthreadex(0)`. Its
//!   `__try/__except(_seh_filter_exe)` is dead under this build's dummy PSEH.
//! * `_endthread(ex)` on a thread with no ptd, or one not started by
//!   `_beginthread(ex)`, is a plain `ExitThread(code)`.

use core::ffi::{c_int, c_uint, c_void};
use core::ptr::null_mut;

use crate::ptd::AcrtPtd;

/// `__acrt_thread_parameter` (`corecrt_internal.h:834`): four pointers and
/// a `bool`. Plain C layout; pinned below.
#[repr(C)]
pub struct AcrtThreadParameter {
    pub procedure: *mut c_void,
    pub context: *mut c_void,
    /// `_beginthread` only; the ending thread closes it.
    pub thread_handle: *mut c_void,
    /// Reference on the procedure's module; the ending thread releases it.
    pub module_handle: *mut c_void,
    /// RoInitialize(MTA) succeeded on the new thread.
    pub initialized_apartment: bool,
}

#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(core::mem::offset_of!(AcrtThreadParameter, thread_handle) == 16);
    assert!(core::mem::offset_of!(AcrtThreadParameter, module_handle) == 24);
    assert!(core::mem::offset_of!(AcrtThreadParameter, initialized_apartment) == 32);
    assert!(core::mem::size_of::<AcrtThreadParameter>() == 40);
};
#[cfg(target_pointer_width = "32")]
const _: () = {
    assert!(core::mem::offset_of!(AcrtThreadParameter, thread_handle) == 8);
    assert!(core::mem::offset_of!(AcrtThreadParameter, module_handle) == 12);
    assert!(core::mem::offset_of!(AcrtThreadParameter, initialized_apartment) == 16);
    assert!(core::mem::size_of::<AcrtThreadParameter>() == 20);
};

/// `_beginthread_proc_type`: `void (__cdecl*)(void*)`.
pub type BeginthreadProc = Option<unsafe extern "C" fn(*mut c_void)>;
/// `_beginthreadex_proc_type`: `unsigned (__stdcall*)(void*)`.
pub type BeginthreadexProc = Option<unsafe extern "system" fn(*mut c_void) -> c_uint>;

/// `LPTHREAD_START_ROUTINE`.
type ThreadStartRoutine = unsafe extern "system" fn(*mut c_void) -> u32;

const INVALID_HANDLE_VALUE: *mut c_void = usize::MAX as *mut c_void;
const EINVAL: c_int = 22;
/// `CREATE_SUSPENDED`.
const CREATE_SUSPENDED: u32 = 0x4;
/// `GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS`.
const GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS: u32 = 0x4;
/// `begin_thread_init_policy_ro_initialize` (unknown, none, ro_initialize).
const BEGIN_THREAD_INIT_POLICY_RO_INITIALIZE: c_int = 2;
/// `RO_INIT_MULTITHREADED`.
const RO_INIT_MULTITHREADED: c_int = 1;
const S_OK: i32 = 0;

extern "C" {
    fn _errno() -> *mut c_int;
    fn _invalid_parameter_noinfo();
    fn calloc(count: usize, size: usize) -> *mut c_void;
    fn free(p: *mut c_void);
    fn __acrt_errno_map_os_error(err: u32);
    fn __acrt_getptd() -> *mut AcrtPtd;
    fn __acrt_getptd_noexit() -> *mut AcrtPtd;
    fn __acrt_get_begin_thread_init_policy() -> c_int;
}

extern "system" {
    fn CreateThread(
        thread_attributes: *mut c_void,
        stack_size: usize,
        start_address: ThreadStartRoutine,
        parameter: *mut c_void,
        creation_flags: u32,
        thread_id: *mut u32,
    ) -> *mut c_void;
    fn ResumeThread(thread: *mut c_void) -> u32;
    fn ExitThread(exit_code: u32) -> !;
    fn FreeLibraryAndExitThread(module: *mut c_void, exit_code: u32) -> !;
    fn CloseHandle(handle: *mut c_void) -> i32;
    fn FreeLibrary(module: *mut c_void) -> c_int;
    fn GetLastError() -> u32;
    fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut c_void) -> c_int;
    /// `HRESULT WINAPI __acrt_RoInitialize(RO_INIT_TYPE)` (`corecrt_internal.h:1342`).
    fn __acrt_RoInitialize(init_type: c_int) -> i32;
    /// `void WINAPI __acrt_RoUninitialize(void)`.
    fn __acrt_RoUninitialize();
}

/// `thread_parameter_free_policy`: what `unique_thread_parameter` does to a
/// block it still owns.
unsafe fn release_thread_parameter(parameter: *mut AcrtThreadParameter) {
    if parameter.is_null() {
        return;
    }

    if !(*parameter).thread_handle.is_null() {
        CloseHandle((*parameter).thread_handle);
    }

    if !(*parameter).module_handle.is_null() {
        FreeLibrary((*parameter).module_handle);
    }

    free(parameter as *mut c_void);
}

/// `thread_start<_beginthread_proc_type, false>`.
unsafe extern "system" fn thread_start(parameter: *mut c_void) -> u32 {
    let context = thread_start_prologue(parameter);
    let procedure = core::mem::transmute::<*mut c_void, unsafe extern "C" fn(*mut c_void)>((*context).procedure);
    procedure((*context).context);
    common_end_thread(0)
}

/// `thread_start<_beginthreadex_proc_type, true>`.
unsafe extern "system" fn thread_start_ex(parameter: *mut c_void) -> u32 {
    let context = thread_start_prologue(parameter);
    let procedure =
        core::mem::transmute::<*mut c_void, unsafe extern "system" fn(*mut c_void) -> c_uint>((*context).procedure);
    common_end_thread(procedure((*context).context))
}

/// The shared head of both `thread_start` instantiations.
#[inline(always)]
unsafe fn thread_start_prologue(parameter: *mut c_void) -> *mut AcrtThreadParameter {
    if parameter.is_null() {
        ExitThread(GetLastError());
    }

    let context = parameter as *mut AcrtThreadParameter;

    (*__acrt_getptd()).beginthread_context = context as *mut c_void;

    if __acrt_get_begin_thread_init_policy() == BEGIN_THREAD_INIT_POLICY_RO_INITIALIZE {
        (*context).initialized_apartment = __acrt_RoInitialize(RO_INIT_MULTITHREADED) == S_OK;
    }

    context
}

/// `create_thread_parameter`.
unsafe fn create_thread_parameter(procedure: *mut c_void, context: *mut c_void) -> *mut AcrtThreadParameter {
    let parameter = calloc(1, core::mem::size_of::<AcrtThreadParameter>()) as *mut AcrtThreadParameter;
    if parameter.is_null() {
        return null_mut();
    }

    (*parameter).procedure = procedure;
    (*parameter).context = context;

    // Pin the procedure's module while the thread runs; released at thread
    // end. Failure leaves `_module_handle` null (calloc'd).
    GetModuleHandleExW(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
        procedure as *const u16,
        core::ptr::addr_of_mut!((*parameter).module_handle),
    );

    parameter
}

/// `_beginthread`: -1 on failure; the new thread owns (and closes) the handle.
#[no_mangle]
pub unsafe extern "C" fn _beginthread(procedure: BeginthreadProc, stack_size: c_uint, context: *mut c_void) -> usize {
    let Some(procedure) = procedure else {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return INVALID_HANDLE_VALUE as usize;
    };

    let parameter = create_thread_parameter(procedure as *mut c_void, context);
    if parameter.is_null() {
        return INVALID_HANDLE_VALUE as usize;
    }

    // Suspended, so the handle can be recorded before the thread can end.
    let mut thread_id: u32 = 0;
    let thread_handle = CreateThread(
        null_mut(),
        stack_size as usize,
        thread_start,
        parameter as *mut c_void,
        CREATE_SUSPENDED,
        &mut thread_id,
    );

    if thread_handle.is_null() {
        __acrt_errno_map_os_error(GetLastError());
        release_thread_parameter(parameter);
        return INVALID_HANDLE_VALUE as usize;
    }

    (*parameter).thread_handle = thread_handle;

    if ResumeThread(thread_handle) == u32::MAX {
        __acrt_errno_map_os_error(GetLastError());
        // Still owned here: closes the handle, drops the module reference.
        release_thread_parameter(parameter);
        return INVALID_HANDLE_VALUE as usize;
    }

    // The thread owns its parameter now.
    thread_handle as usize
}

/// `_beginthreadex`: 0 on failure; the caller owns the handle.
#[no_mangle]
pub unsafe extern "C" fn _beginthreadex(
    security_descriptor: *mut c_void,
    stack_size: c_uint,
    procedure: BeginthreadexProc,
    context: *mut c_void,
    creation_flags: c_uint,
    thread_id_result: *mut c_uint,
) -> usize {
    let Some(procedure) = procedure else {
        *_errno() = EINVAL;
        _invalid_parameter_noinfo();
        return 0;
    };

    let parameter = create_thread_parameter(procedure as *mut c_void, context);
    if parameter.is_null() {
        return 0;
    }

    let mut thread_id: u32 = 0;
    let thread_handle = CreateThread(
        security_descriptor,
        stack_size as usize,
        thread_start_ex,
        parameter as *mut c_void,
        creation_flags,
        &mut thread_id,
    );

    if thread_handle.is_null() {
        __acrt_errno_map_os_error(GetLastError());
        release_thread_parameter(parameter);
        return 0;
    }

    if !thread_id_result.is_null() {
        *thread_id_result = thread_id;
    }

    // The thread owns its parameter now.
    thread_handle as usize
}

/// `common_end_thread`.
unsafe fn common_end_thread(return_code: c_uint) -> ! {
    let ptd = __acrt_getptd_noexit();
    if ptd.is_null() {
        ExitThread(return_code);
    }

    let parameter = (*ptd).beginthread_context as *mut AcrtThreadParameter;
    if parameter.is_null() {
        ExitThread(return_code);
    }

    if (*parameter).initialized_apartment {
        __acrt_RoUninitialize();
    }

    let thread_handle = (*parameter).thread_handle;
    if thread_handle != INVALID_HANDLE_VALUE && !thread_handle.is_null() {
        CloseHandle(thread_handle);
    }

    let module_handle = (*parameter).module_handle;
    if module_handle != INVALID_HANDLE_VALUE && !module_handle.is_null() {
        FreeLibraryAndExitThread(module_handle, return_code)
    } else {
        ExitThread(return_code)
    }
}

/// `_endthread`: end a `_beginthread` thread with exit code 0.
#[no_mangle]
pub unsafe extern "C" fn _endthread() {
    common_end_thread(0)
}

/// `_endthreadex`.
#[no_mangle]
pub unsafe extern "C" fn _endthreadex(return_code: c_uint) {
    common_end_thread(return_code)
}
