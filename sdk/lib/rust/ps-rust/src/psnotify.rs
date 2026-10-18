use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};
use core::mem::transmute;

// 64 slots as Windows has, not ps.h's 8; agrees with psmgr.rs and ntkrnl-sys.
const PSP_MAX_CREATE_THREAD_NOTIFY: usize = 64; // nolint: shadowed-c-define
const PSP_MAX_CREATE_PROCESS_NOTIFY: usize = 64; // nolint: shadowed-c-define
const PSP_MAX_LOAD_IMAGE_NOTIFY: usize = 64; // nolint: shadowed-c-define

#[no_mangle]
static mut PsImageNotifyEnabled: BOOLEAN = 0u8;
#[no_mangle]
static mut PspThreadNotifyRoutineCount: ULONG = 0;
#[no_mangle]
static mut PspProcessNotifyRoutineCount: ULONG = 0;
#[no_mangle]
static mut PspLoadImageNotifyRoutineCount: ULONG = 0;
// EX_CALLBACK contains EX_FAST_REF union — must use zeroed() for initialization
#[no_mangle]
static mut PspThreadNotifyRoutine: [EX_CALLBACK; PSP_MAX_CREATE_THREAD_NOTIFY] = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut PspProcessNotifyRoutine: [EX_CALLBACK; PSP_MAX_CREATE_PROCESS_NOTIFY] = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut PspLoadImageNotifyRoutine: [EX_CALLBACK; PSP_MAX_LOAD_IMAGE_NOTIFY] = unsafe { core::mem::zeroed() };
#[no_mangle]
static mut PspLegoNotifyRoutine: PLEGO_NOTIFY_ROUTINE = None;

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn PsSetCreateProcessNotifyRoutine(
    NotifyRoutine: PCREATE_PROCESS_NOTIFY_ROUTINE,
    Remove: BOOLEAN,
) -> NTSTATUS {
    let mut i: ULONG;
    let mut CallBack: PEX_CALLBACK_ROUTINE_BLOCK;

    if Remove != 0 {
        i = 0;
        while i < PSP_MAX_CREATE_PROCESS_NOTIFY as ULONG {
            CallBack = ExReferenceCallBackBlock(core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i as usize]));
            if CallBack.is_null() {
                i += 1;
                continue;
            }

            if ExGetCallBackBlockRoutine(CallBack) == transmute(NotifyRoutine) {
                if ExCompareExchangeCallBack(
                    core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i as usize]),
                    core::ptr::null_mut(),
                    CallBack,
                ) != 0
                {
                    InterlockedDecrement(core::ptr::addr_of_mut!(PspProcessNotifyRoutineCount) as *mut LONG);

                    ExDereferenceCallBackBlock(
                        core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i as usize]),
                        CallBack,
                    );

                    ExWaitForCallBacks(CallBack);

                    ExFreeCallBack(CallBack);
                    return STATUS_SUCCESS;
                }

                ExDereferenceCallBackBlock(
                    core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i as usize]),
                    CallBack,
                );
            }
            i += 1;
        }

        return STATUS_PROCEDURE_NOT_FOUND;
    } else {
        CallBack = ExAllocateCallBack(transmute(NotifyRoutine), core::ptr::null_mut());
        if CallBack.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        i = 0;
        while i < PSP_MAX_CREATE_PROCESS_NOTIFY as ULONG {
            if ExCompareExchangeCallBack(
                core::ptr::addr_of_mut!(PspProcessNotifyRoutine[i as usize]),
                CallBack,
                core::ptr::null_mut(),
            ) != 0
            {
                InterlockedIncrement(core::ptr::addr_of_mut!(PspProcessNotifyRoutineCount) as *mut LONG);
                return STATUS_SUCCESS;
            }
            i += 1;
        }

        ExFreeCallBack(CallBack);
        return STATUS_INVALID_PARAMETER;
    }
}

#[no_mangle]
pub unsafe extern "system" fn PsSetLegoNotifyRoutine(LegoNotifyRoutine: PVOID) -> ULONG {
    let LegoNotifyRoutine = opaque_ptr(LegoNotifyRoutine);
    PspLegoNotifyRoutine = transmute(LegoNotifyRoutine);

    #[cfg(target_arch = "x86")]
    { core::mem::offset_of!(_KTHREAD, __bindgen_anon_9.__bindgen_anon_5.LegoData) as ULONG }
    #[cfg(not(target_arch = "x86"))]
    { core::mem::offset_of!(_KTHREAD, __bindgen_anon_7.__bindgen_anon_5.LegoData) as ULONG }
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn PsRemoveLoadImageNotifyRoutine(
    NotifyRoutine: PLOAD_IMAGE_NOTIFY_ROUTINE,
) -> NTSTATUS {
    let mut i: ULONG;
    let mut CallBack: PEX_CALLBACK_ROUTINE_BLOCK;

    i = 0;
    while i < PSP_MAX_LOAD_IMAGE_NOTIFY as ULONG {
        CallBack = ExReferenceCallBackBlock(core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i as usize]));
        if !CallBack.is_null() {
            if ExGetCallBackBlockRoutine(CallBack) == transmute(NotifyRoutine) {
                if ExCompareExchangeCallBack(
                    core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i as usize]),
                    core::ptr::null_mut(),
                    CallBack,
                ) != 0
                {
                    InterlockedDecrement(core::ptr::addr_of_mut!(PspLoadImageNotifyRoutineCount) as *mut LONG);
                    ExDereferenceCallBackBlock(
                        core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i as usize]),
                        CallBack,
                    );

                    ExWaitForCallBacks(CallBack);

                    ExFreeCallBack(CallBack);
                    return STATUS_SUCCESS;
                }
            }

            ExDereferenceCallBackBlock(
                core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i as usize]),
                CallBack,
            );
        }
        i += 1;
    }

    STATUS_PROCEDURE_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn PsSetLoadImageNotifyRoutine(
    NotifyRoutine: PLOAD_IMAGE_NOTIFY_ROUTINE,
) -> NTSTATUS {
    let mut i: ULONG;
    let CallBack: PEX_CALLBACK_ROUTINE_BLOCK;

    CallBack = ExAllocateCallBack(transmute(NotifyRoutine), core::ptr::null_mut());
    if CallBack.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < PSP_MAX_LOAD_IMAGE_NOTIFY as ULONG {
        if ExCompareExchangeCallBack(
            core::ptr::addr_of_mut!(PspLoadImageNotifyRoutine[i as usize]),
            CallBack,
            core::ptr::null_mut(),
        ) != 0
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(PspLoadImageNotifyRoutineCount) as *mut LONG);
            PsImageNotifyEnabled = 1u8;
            return STATUS_SUCCESS;
        }
        i += 1;
    }

    ExFreeCallBack(CallBack);
    STATUS_INSUFFICIENT_RESOURCES
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "system" fn PsRemoveCreateThreadNotifyRoutine(
    NotifyRoutine: PCREATE_THREAD_NOTIFY_ROUTINE,
) -> NTSTATUS {
    let mut i: ULONG;
    let mut CallBack: PEX_CALLBACK_ROUTINE_BLOCK;

    i = 0;
    while i < PSP_MAX_CREATE_THREAD_NOTIFY as ULONG {
        CallBack = ExReferenceCallBackBlock(core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i as usize]));
        if !CallBack.is_null() {
            if ExGetCallBackBlockRoutine(CallBack) == transmute(NotifyRoutine) {
                if ExCompareExchangeCallBack(
                    core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i as usize]),
                    core::ptr::null_mut(),
                    CallBack,
                ) != 0
                {
                    InterlockedDecrement(core::ptr::addr_of_mut!(PspThreadNotifyRoutineCount) as *mut LONG);
                    ExDereferenceCallBackBlock(
                        core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i as usize]),
                        CallBack,
                    );

                    ExWaitForCallBacks(CallBack);

                    ExFreeCallBack(CallBack);
                    return STATUS_SUCCESS;
                }
            }

            ExDereferenceCallBackBlock(
                core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i as usize]),
                CallBack,
            );
        }
        i += 1;
    }

    STATUS_PROCEDURE_NOT_FOUND
}

#[no_mangle]
pub unsafe extern "system" fn PsSetCreateThreadNotifyRoutine(
    NotifyRoutine: PCREATE_THREAD_NOTIFY_ROUTINE,
) -> NTSTATUS {
    let mut i: ULONG;
    let CallBack: PEX_CALLBACK_ROUTINE_BLOCK;

    CallBack = ExAllocateCallBack(transmute(NotifyRoutine), core::ptr::null_mut());
    if CallBack.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    i = 0;
    while i < PSP_MAX_CREATE_THREAD_NOTIFY as ULONG {
        if ExCompareExchangeCallBack(
            core::ptr::addr_of_mut!(PspThreadNotifyRoutine[i as usize]),
            CallBack,
            core::ptr::null_mut(),
        ) != 0
        {
            InterlockedIncrement(core::ptr::addr_of_mut!(PspThreadNotifyRoutineCount) as *mut LONG);
            return STATUS_SUCCESS;
        }
        i += 1;
    }

    ExFreeCallBack(CallBack);
    STATUS_INSUFFICIENT_RESOURCES
}
