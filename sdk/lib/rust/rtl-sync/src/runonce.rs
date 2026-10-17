use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{NtReleaseKeyedEvent, NtWaitForKeyedEvent, PRTL_RUN_ONCE, PRTL_RUN_ONCE_INIT_FN};
use crate::shared::{DWORD, interlocked_cmpxchg_ptr};

// ---- file-scope preamble (translated) ----
pub const RTL_RUN_ONCE_CHECK_ONLY: u32 = 0x00000001;
pub const RTL_RUN_ONCE_ASYNC: u32 = 0x00000002;
pub const RTL_RUN_ONCE_INIT_FAILED: u32 = 0x00000004;

#[no_mangle]
pub unsafe extern "system" fn RtlRunOnceInitialize(RunOnce: PRTL_RUN_ONCE) {
    (*RunOnce).Ptr = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn RtlRunOnceBeginInitialize(
    RunOnce: PRTL_RUN_ONCE,
    Flags: ULONG,
    Context: *mut PVOID,
) -> NTSTATUS {
    if Flags & RTL_RUN_ONCE_CHECK_ONLY != 0 {
        let val = (*RunOnce).Ptr as ULONG_PTR;

        if Flags & RTL_RUN_ONCE_ASYNC != 0 {
            return STATUS_INVALID_PARAMETER;
        }
        if val & 3 != 2 {
            return STATUS_UNSUCCESSFUL;
        }
        if !Context.is_null() {
            *Context = (val & !3) as PVOID;
        }
        return STATUS_SUCCESS;
    }

    loop {
        let mut next: ULONG_PTR;
        let val = (*RunOnce).Ptr as ULONG_PTR;

        match val & 3 {
            0 => {
                let new_val = if Flags & RTL_RUN_ONCE_ASYNC != 0 {
                    3 as PVOID
                } else {
                    1 as PVOID
                };
                if interlocked_cmpxchg_ptr(
                    core::ptr::addr_of_mut!((*RunOnce).Ptr),
                    new_val,
                    core::ptr::null_mut(),
                )
                .is_null()
                {
                    return STATUS_PENDING;
                }
            }
            1 => {
                if Flags & RTL_RUN_ONCE_ASYNC != 0 {
                    return STATUS_INVALID_PARAMETER;
                }
                next = val & !3;
                if interlocked_cmpxchg_ptr(
                    core::ptr::addr_of_mut!((*RunOnce).Ptr),
                    ((core::ptr::addr_of_mut!(next) as ULONG_PTR) | 1) as PVOID,
                    val as PVOID,
                ) == val as PVOID
                {
                    NtWaitForKeyedEvent(
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(next) as PVOID,
                        0u8,
                        core::ptr::null_mut(),
                    );
                }
            }
            2 => {
                if !Context.is_null() {
                    *Context = (val & !3) as PVOID;
                }
                return STATUS_SUCCESS;
            }
            3 => {
                if Flags & RTL_RUN_ONCE_ASYNC == 0 {
                    return STATUS_INVALID_PARAMETER;
                }
                return STATUS_PENDING;
            }
            _ => {}
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlRunOnceComplete(
    RunOnce: PRTL_RUN_ONCE,
    Flags: ULONG,
    mut Context: PVOID,
) -> NTSTATUS {
    if (Context as usize) & 3 != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    if Flags & RTL_RUN_ONCE_INIT_FAILED != 0 {
        if !Context.is_null() {
            return STATUS_INVALID_PARAMETER;
        }
        if Flags & RTL_RUN_ONCE_ASYNC != 0 {
            return STATUS_INVALID_PARAMETER;
        }
    } else {
        Context = ((Context as usize) | 2) as PVOID;
    }

    loop {
        let mut val = (*RunOnce).Ptr as usize;

        match val & 3 {
            1 => {
                if interlocked_cmpxchg_ptr(
                    core::ptr::addr_of_mut!((*RunOnce).Ptr),
                    Context,
                    val as PVOID,
                ) != val as PVOID
                {
                    continue;
                }
                val &= !3;
                while val != 0 {
                    let next = *(val as *mut usize);
                    NtReleaseKeyedEvent(0 as _, val as PVOID, 0u8, core::ptr::null_mut());
                    val = next;
                }
                return STATUS_SUCCESS;
            }
            3 => {
                if Flags & RTL_RUN_ONCE_ASYNC == 0 {
                    return STATUS_INVALID_PARAMETER;
                }
                if interlocked_cmpxchg_ptr(
                    core::ptr::addr_of_mut!((*RunOnce).Ptr),
                    Context,
                    val as PVOID,
                ) != val as PVOID
                {
                    continue;
                }
                return STATUS_SUCCESS;
            }
            _ => {
                return STATUS_UNSUCCESSFUL;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RtlRunOnceExecuteOnce(
    RunOnce: PRTL_RUN_ONCE,
    InitFn: PRTL_RUN_ONCE_INIT_FN,
    Parameter: PVOID,
    Context: *mut PVOID,
) -> NTSTATUS {
    let ret: DWORD = RtlRunOnceBeginInitialize(RunOnce, 0, Context) as DWORD;

    if ret != STATUS_PENDING as DWORD {
        return ret as NTSTATUS;
    }

    if InitFn.unwrap()(RunOnce, Parameter, Context) == 0 {
        RtlRunOnceComplete(RunOnce, RTL_RUN_ONCE_INIT_FAILED, core::ptr::null_mut());
        return STATUS_UNSUCCESSFUL;
    }

    RtlRunOnceComplete(
        RunOnce,
        0,
        if !Context.is_null() { *Context } else { core::ptr::null_mut() },
    )
}
