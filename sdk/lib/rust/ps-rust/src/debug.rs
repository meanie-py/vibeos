use core::ffi::c_void;
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    fn KeInitializeEvent(event: PVOID, event_type: ULONG, state: BOOLEAN);
    fn PspGetOrSetContextKernelRoutine(
        apc: PVOID,
        normal_routine: *mut PVOID,
        normal_context: *mut PVOID,
        system_argument1: *mut PVOID,
        system_argument2: *mut PVOID,
    );
    fn KeWaitForSingleObject(
        object: PVOID,
        wait_reason: ULONG,
        wait_mode: KPROCESSOR_MODE,
        alertable: BOOLEAN,
        timeout: PVOID,
    ) -> NTSTATUS;
}
extern "system" {
    fn PsGetCurrentThread() -> PETHREAD;
}

extern "C" {
    static PsThreadType: POBJECT_TYPE;
}
extern "system" {
    fn KeEnterGuardedRegion();
}
extern "system" {
    fn KeLeaveGuardedRegion();
}

const NotificationEvent: ULONG = 0;
const OriginalApcEnvironment: ULONG = 0;

#[cfg(target_arch = "x86")]
// xdk/winnt_old.h: CONTEXT_i386 | 0x20. The bare bit let the `(f & X) == X` tests below
// accept a context with no (or a foreign) arch base.
const CONTEXT_EXTENDED_REGISTERS: u32 = 0x00010020;

#[inline]
unsafe fn UlongToPtr(value: ULONG) -> PVOID {
    value as usize as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn PsGetContextThread(
    thread: PETHREAD,
    thread_context: PCONTEXT,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let thread = opaque_ptr(thread);
    let mut get_set_context: GET_SET_CTX_CONTEXT = core::mem::zeroed();
    let mut size: usize;
    let flags: ULONG;

    size = core::mem::size_of::<CONTEXT>();

    // Read and validate ContextFlags from user-mode pointer
    flags = match probe_read_user::<ULONG>(core::ptr::addr_of!((*thread_context).ContextFlags) as *const ULONG, previous_mode) {
        Ok(v) => v,
        Err(s) => return s,
    };

    #[cfg(target_arch = "x86")]
    {
        if (flags & CONTEXT_EXTENDED_REGISTERS) != CONTEXT_EXTENDED_REGISTERS {
            size = core::mem::offset_of!(CONTEXT, ExtendedRegisters);
        }
    }

    // Validate user-mode pointer for writing before we use it
    if previous_mode != 0i8 {
        if let Err(s) = probe_user_buffer(
            thread_context as PVOID,
            size,
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(get_set_context.Event) as PVOID,
        NotificationEvent,
        0u8,
    );

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!(get_set_context.Context) as *mut u8,
        0,
        size,
    );
    get_set_context.Context.ContextFlags = flags;
    get_set_context.Mode = previous_mode;

    let status: NTSTATUS;

    if thread == PsGetCurrentThread() as PETHREAD {
        get_set_context.Apc.SystemArgument1 = core::ptr::null_mut();
        get_set_context.Apc.SystemArgument2 = thread as PVOID;

        KeEnterGuardedRegion();

        PspGetOrSetContextKernelRoutine(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PVOID,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(get_set_context.Apc.SystemArgument1),
            core::ptr::addr_of_mut!(get_set_context.Apc.SystemArgument2),
        );

        KeLeaveGuardedRegion();

        status = STATUS_SUCCESS;
    } else {
        KeInitializeApc(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PKAPC,
            core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb) as PKTHREAD,
            _KAPC_ENVIRONMENT_OriginalApcEnvironment,
            core::mem::transmute(PspGetOrSetContextKernelRoutine as usize),
            None,
            None,
            0i8,
            core::ptr::null_mut(),
        );

        if KeInsertQueueApc(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PKAPC,
            core::ptr::null_mut(),
            thread as PVOID,
            2,
        ) == 0u8
        {
            status = STATUS_UNSUCCESSFUL;
        } else {
            status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!(get_set_context.Event) as PVOID,
                0,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }

    // Copy context to user-mode buffer with validation
    if let Err(s) = safe_copy_to_user(
        thread_context as *mut c_void,
        core::ptr::addr_of_mut!(get_set_context.Context) as *const c_void,
        size,
        core::mem::size_of::<ULONG>(),
        previous_mode,
    ) {
        return s;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn PsSetContextThread(
    thread: PETHREAD,
    thread_context: PCONTEXT,
    previous_mode: KPROCESSOR_MODE,
) -> NTSTATUS {
    let thread = opaque_ptr(thread);
    let mut get_set_context: GET_SET_CTX_CONTEXT = core::mem::zeroed();
    let mut size: usize;
    let flags: ULONG;

    size = core::mem::size_of::<CONTEXT>();

    // Read and validate ContextFlags from user-mode pointer
    flags = match probe_read_user::<ULONG>(core::ptr::addr_of!((*thread_context).ContextFlags) as *const ULONG, previous_mode) {
        Ok(v) => v,
        Err(s) => return s,
    };

    #[cfg(target_arch = "x86")]
    {
        if (flags & CONTEXT_EXTENDED_REGISTERS) != CONTEXT_EXTENDED_REGISTERS {
            size = core::mem::offset_of!(CONTEXT, ExtendedRegisters);
        }
    }

    // Validate user-mode pointer for reading before we copy from it
    if previous_mode != 0i8 {
        if let Err(s) = probe_user_buffer(
            thread_context as PVOID,
            size,
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return s;
        }
    }

    // Copy context from user-mode buffer with validation
    if let Err(s) = safe_copy_from_user(
        core::ptr::addr_of_mut!(get_set_context.Context) as PVOID,
        thread_context as *const c_void,
        size,
        core::mem::size_of::<ULONG>(),
        previous_mode,
    ) {
        return s;
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!(get_set_context.Event) as PVOID,
        NotificationEvent,
        0u8,
    );

    get_set_context.Context.ContextFlags = flags;
    get_set_context.Mode = previous_mode;

    let status: NTSTATUS;

    if thread == PsGetCurrentThread() as PETHREAD {
        get_set_context.Apc.SystemArgument1 = UlongToPtr(1);
        get_set_context.Apc.SystemArgument2 = thread as PVOID;

        KeEnterGuardedRegion();

        PspGetOrSetContextKernelRoutine(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PVOID,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(get_set_context.Apc.SystemArgument1),
            core::ptr::addr_of_mut!(get_set_context.Apc.SystemArgument2),
        );

        KeLeaveGuardedRegion();

        status = STATUS_SUCCESS;
    } else {
        KeInitializeApc(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PKAPC,
            core::ptr::addr_of_mut!((*(thread as *mut _ETHREAD)).Tcb) as PKTHREAD,
            _KAPC_ENVIRONMENT_OriginalApcEnvironment,
            core::mem::transmute(PspGetOrSetContextKernelRoutine as usize),
            None,
            None,
            0i8,
            core::ptr::null_mut(),
        );

        if KeInsertQueueApc(
            core::ptr::addr_of_mut!(get_set_context.Apc) as PKAPC,
            UlongToPtr(1),
            thread as PVOID,
            2,
        ) == 0u8
        {
            status = STATUS_UNSUCCESSFUL;
        } else {
            status = KeWaitForSingleObject(
                core::ptr::addr_of_mut!(get_set_context.Event) as PVOID,
                0,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtGetContextThread(
    thread_handle: HANDLE,
    thread_context: PCONTEXT,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let previous_mode: KPROCESSOR_MODE = ExGetPreviousMode();

    let mut status = ObReferenceObjectByHandle(
        thread_handle,
        THREAD_GET_CONTEXT,
        PsThreadType,
        previous_mode,
        core::ptr::addr_of_mut!(thread) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 {
        status = STATUS_INVALID_HANDLE;
    } else {
        status = PsGetContextThread(thread, thread_context, previous_mode);
    }

    ObfDereferenceObject(thread as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetContextThread(
    thread_handle: HANDLE,
    thread_context: PCONTEXT,
) -> NTSTATUS {
    let thread_handle = opaque_ptr(thread_handle);
    let mut thread: PETHREAD = core::ptr::null_mut();
    let previous_mode: KPROCESSOR_MODE = ExGetPreviousMode();

    let mut status = ObReferenceObjectByHandle(
        thread_handle,
        THREAD_SET_CONTEXT,
        PsThreadType,
        previous_mode,
        core::ptr::addr_of_mut!(thread) as *mut PVOID,
        core::ptr::null_mut(),
    );

    if status < 0 {
        return status;
    }

    if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() != 0 {
        status = STATUS_INVALID_HANDLE;
    } else {
        status = PsSetContextThread(thread, thread_context, previous_mode);
    }

    ObfDereferenceObject(thread as PVOID);
    status
}
