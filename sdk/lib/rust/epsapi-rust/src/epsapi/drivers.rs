use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "system" fn PsaEnumerateSystemModules(
    Callback: PSYSMOD_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    let mut psmModules: PRTL_PROCESS_MODULES = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    loop {
        /* capture the system modules */
        Status = PsaCaptureSystemModules(core::ptr::addr_of_mut!(psmModules));

        if !NT_SUCCESS!(Status) {
            break;
        }

        /* walk the system modules */
        Status = PsaWalkSystemModules(psmModules, Callback, CallbackContext);
        if !((0) != 0) {
            break;
        }
    }
    /* free the capture */
    PsaFreeCapture(psmModules as PVOID);

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PsaCaptureSystemModules(
    SystemModules: *mut PRTL_PROCESS_MODULES,
) -> NTSTATUS {
    let mut nSize: SIZE_T = 0 as SIZE_T;
    let mut psmModules: PRTL_PROCESS_MODULES = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    loop {
        /* initial probe. We just get the count of system modules */
        Status = NtQuerySystemInformation(
            SystemModuleInformation,
            core::ptr::addr_of_mut!(nSize) as PVOID,
            core::mem::size_of::<SIZE_T>() as ULONG,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS!(Status) && (Status != STATUS_INFO_LENGTH_MISMATCH) {
            DPRINT!(FAILED_WITH_STATUS, "NtQuerySystemInformation", Status);
            break;
        }

        /* RATIONALE: the loading of a system module is a rare occurrence. To
           minimize memory operations that could be expensive, or fragment the
           pool/heap, we try to determine the buffer size in advance, knowing that
           the number of elements is unlikely to change */
        nSize = (core::mem::size_of::<RTL_PROCESS_MODULES>() as SIZE_T).wrapping_add(
            nSize.wrapping_mul(core::mem::size_of::<RTL_PROCESS_MODULES>() as SIZE_T),
        );

        psmModules = core::ptr::null_mut();

        loop {
            let mut pTmp: PVOID;

            /* free the buffer, and reallocate it to the new size. RATIONALE: since we
               ignore the buffer's content at this point, there's no point in a realloc,
               that could end up copying a large chunk of data we'd discard anyway */
            PsaiFree(psmModules as PVOID);
            psmModules = core::ptr::null_mut();
            pTmp = PsaiMalloc(nSize) as PVOID;

            if pTmp.is_null() {
                Status = STATUS_NO_MEMORY;
                DPRINT!(FAILED_WITH_STATUS, "PsaiMalloc", Status);
                break;
            }

            psmModules = pTmp as PRTL_PROCESS_MODULES;

            /* query the information */
            Status = NtQuerySystemInformation(
                SystemModuleInformation,
                psmModules as PVOID,
                nSize as ULONG,
                core::ptr::null_mut(),
            );

            /* double the buffer for the next loop */
            nSize = nSize.wrapping_mul(2 as SIZE_T);

            if Status != STATUS_INFO_LENGTH_MISMATCH {
                break;
            }
        }

        if !NT_SUCCESS!(Status) {
            DPRINT!(FAILED_WITH_STATUS, "NtQuerySystemInformation", Status);
            break;
        }

        *SystemModules = psmModules;

        Status = STATUS_SUCCESS;

        if (0) == 0 {
            break;
        }
    }

    /* in case of failure, free the buffer */
    if !NT_SUCCESS!(Status) && !psmModules.is_null() {
        PsaiFree(psmModules as PVOID);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn PsaWalkSystemModules(
    SystemModules: PRTL_PROCESS_MODULES,
    Callback: PSYSMOD_ENUM_ROUTINE,
    CallbackContext: PVOID,
) -> NTSTATUS {
    let mut i: ULONG;
    let mut Status: NTSTATUS;

    /* repeat until all modules have been returned */
    i = 0;
    while i < (*SystemModules).NumberOfModules {
        /* return current module to the callback */
        Status = (Callback.unwrap_unchecked())(
            (core::ptr::addr_of_mut!((*SystemModules).Modules) as *mut RTL_PROCESS_MODULE_INFORMATION)
                .offset(i as isize) as PRTL_PROCESS_MODULE_INFORMATION,
            CallbackContext,
        );

        if !NT_SUCCESS!(Status) {
            return Status;
        }
        i = i.wrapping_add(1);
    }

    return STATUS_SUCCESS;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkFirstSystemModule(
    SystemModules: PRTL_PROCESS_MODULES,
) -> PRTL_PROCESS_MODULE_INFORMATION {
    return core::ptr::addr_of_mut!((*SystemModules).Modules[0 as usize]);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkFirstSystemModule(
    SystemModules: PRTL_PROCESS_MODULES,
) -> PRTL_PROCESS_MODULE_INFORMATION {
    return core::ptr::addr_of_mut!((*SystemModules).Modules[0 as usize]);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn PsaWalkNextSystemModule(
    mut CurrentSystemModule: PRTL_PROCESS_MODULES,
) -> PRTL_PROCESS_MODULE_INFORMATION {
    return ((CurrentSystemModule as ULONG_PTR)
        + ((FIELD_OFFSET!(RTL_PROCESS_MODULES, Modules[1]) as ULONG_PTR)
            - (FIELD_OFFSET!(RTL_PROCESS_MODULES, Modules[0]) as ULONG_PTR)))
        as PRTL_PROCESS_MODULE_INFORMATION;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn PsaWalkNextSystemModule(
    mut CurrentSystemModule: PRTL_PROCESS_MODULES,
) -> PRTL_PROCESS_MODULE_INFORMATION {
    return ((CurrentSystemModule as ULONG_PTR)
        + ((FIELD_OFFSET!(RTL_PROCESS_MODULES, Modules[1]) as ULONG_PTR)
            - (FIELD_OFFSET!(RTL_PROCESS_MODULES, Modules[0]) as ULONG_PTR)))
        as PRTL_PROCESS_MODULE_INFORMATION;
}
