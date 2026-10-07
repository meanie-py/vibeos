use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;

use ntkrnl_sys::ddk::{
    _EPROCESS, _ETHREAD, _LUID, _LARGE_INTEGER,
    HANDLE, KPROCESSOR_MODE, POBJECT_TYPE, PEPROCESS, PETHREAD,
    HARDERROR_MSG, PHARDERROR_MSG,
    HARDERROR_USER_PARAMETERS, PHARDERROR_USER_PARAMETERS,
    PORT_MESSAGE, PPORT_MESSAGE,
    ExGetPreviousMode,
    PsGetCurrentThread,
    SeSinglePrivilegeCheck,
    ObReferenceObjectByHandle,
    ZwAllocateVirtualMemory, ZwFreeVirtualMemory,
    LpcRequestWaitReplyPort,
};
use ntkrnl_sys::probe::probe_user_buffer;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "system" {
    static mut LpcPortObjectType: PVOID;
    static SeTcbPrivilege: _LUID;
    static SeShutdownPrivilege: _LUID;
    fn KeGetPreviousMode() -> KPROCESSOR_MODE;
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: *mut ntkrnl_sys::ddk::_LARGE_INTEGER);
}

const MAXIMUM_HARDERROR_PARAMETERS: usize = 0x5;
// ndk/lpctypes.h:101-105: 512 on _WIN64, 256 otherwise (only sizes the stack buffer).
#[cfg(target_pointer_width = "64")]
const PORT_MAXIMUM_MESSAGE_LENGTH: usize = 512;
#[cfg(not(target_pointer_width = "64"))]
const PORT_MAXIMUM_MESSAGE_LENGTH: usize = 256;
const LPC_ERROR_EVENT: u16 = 0x09;

const SEM_FAILCRITICALERRORS: ULONG = 0x0001;
const HARDERROR_OVERRIDE_ERRORMODE: ULONG = 0x10000000;
const RTL_SEM_FAILCRITICALERRORS: ULONG = 0x0010;

const OptionAbortRetryIgnore: ULONG = 0;
const OptionOk: ULONG = 1;
const OptionOkCancel: ULONG = 2;
const OptionRetryCancel: ULONG = 3;
const OptionYesNo: ULONG = 4;
const OptionYesNoCancel: ULONG = 5;
const OptionShutdownSystem: ULONG = 6;
const OptionOkNoWait: ULONG = 7;
const OptionCancelTryContinue: ULONG = 8;

const ResponseReturnToCaller: ULONG = 0;
const ResponseNotHandled: ULONG = 1;
const ResponseAbort: ULONG = 2;
const ResponseCancel: ULONG = 3;
const ResponseIgnore: ULONG = 4;
const ResponseNo: ULONG = 5;
const ResponseOk: ULONG = 6;
const ResponseRetry: ULONG = 7;
const ResponseYes: ULONG = 8;
const ResponseTryAgain: ULONG = 9;
const ResponseContinue: ULONG = 10;

const FATAL_UNHANDLED_HARD_ERROR: ULONG = 0x0000004C;

const TAG_ERR: ULONG = u32::from_le_bytes(*b"Err\0");

#[no_mangle]
pub static mut ExReadyForErrors: BOOLEAN = 0u8;
#[no_mangle]
pub static mut ExpDefaultErrorPort: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut ExpDefaultErrorPortProcess: PEPROCESS = core::ptr::null_mut();

unsafe fn ExpSystemErrorHandler(
    ErrorStatus: NTSTATUS,
    NumberOfParameters: ULONG,
    _UnicodeStringParameterMask: ULONG,
    Parameters: PULONG_PTR,
    _Shutdown: BOOLEAN,
) -> NTSTATUS {
    let mut BugCheckParameters: [ULONG_PTR; MAXIMUM_HARDERROR_PARAMETERS] = [0; MAXIMUM_HARDERROR_PARAMETERS];

    for i in 0..(NumberOfParameters as usize).min(MAXIMUM_HARDERROR_PARAMETERS) {
        BugCheckParameters[i] = *Parameters.add(i);
    }

    KeBugCheckEx(
        FATAL_UNHANDLED_HARD_ERROR,
        ErrorStatus as ULONG_PTR,
        BugCheckParameters.as_ptr() as ULONG_PTR,
        0,
        0,
    );
    STATUS_SUCCESS
}

unsafe fn ExpRaiseHardError(
    ErrorStatus: NTSTATUS,
    NumberOfParameters: ULONG,
    UnicodeStringParameterMask: ULONG,
    Parameters: PULONG_PTR,
    ValidResponseOptions: ULONG,
    Response: *mut ULONG,
) -> NTSTATUS {
    let Process = PsGetCurrentProcess() as *mut _EPROCESS;
    let Thread = PsGetCurrentThread() as *mut _ETHREAD;
    let mut Buffer: [u8; PORT_MAXIMUM_MESSAGE_LENGTH] = [0; PORT_MAXIMUM_MESSAGE_LENGTH];
    let Message = Buffer.as_mut_ptr() as PHARDERROR_MSG;
    let mut PortHandle: HANDLE;
    let PreviousMode = KeGetPreviousMode();

    if ValidResponseOptions == OptionShutdownSystem {
        if SeSinglePrivilegeCheck(
            SeTcbPrivilege,
            PreviousMode,
        ) == 0u8
            || SeSinglePrivilegeCheck(
                SeShutdownPrivilege,
                PreviousMode,
            ) == 0u8
        {
            *Response = ResponseNotHandled;
            return STATUS_PRIVILEGE_NOT_HELD;
        }

        ExReadyForErrors = 0u8;
    }

    if (*Thread).__bindgen_anon_7.__bindgen_anon_1.HardErrorsAreDisabled() == 0 {
        if ExReadyForErrors == 0u8 && ErrorStatus < 0 {
            ExpSystemErrorHandler(
                ErrorStatus,
                NumberOfParameters,
                UnicodeStringParameterMask,
                Parameters,
                if PreviousMode != 0i8 { 1u8 } else { 0u8 },
            );
        }
    }

    if ((*Process).DefaultHardErrorProcessing & SEM_FAILCRITICALERRORS) != 0
        || (ErrorStatus as ULONG & HARDERROR_OVERRIDE_ERRORMODE) != 0
    {
        if !(*Process).ExceptionPort.is_null() {
            PortHandle = (*Process).ExceptionPort;
        } else {
            PortHandle = ExpDefaultErrorPort;
        }
    } else {
        PortHandle = core::ptr::null_mut();
    }

    if (*Thread).__bindgen_anon_7.__bindgen_anon_1.HardErrorsAreDisabled() != 0 {
        PortHandle = core::ptr::null_mut();
    }

    if (*Thread).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0 && !PortHandle.is_null() {
        let Teb = (*Thread).Tcb.Teb;
        if !Teb.is_null() {
            let hard_error_mode = (*Teb).HardErrorMode;
            if (hard_error_mode & RTL_SEM_FAILCRITICALERRORS) != 0 {
                PortHandle = core::ptr::null_mut();
            }
        }
    }

    if PortHandle.is_null() {
        *Response = ResponseReturnToCaller;
        return STATUS_SUCCESS;
    }

    if Process == ExpDefaultErrorPortProcess as *mut _EPROCESS {
        if ErrorStatus < 0 {
            ExpSystemErrorHandler(
                ErrorStatus,
                NumberOfParameters,
                UnicodeStringParameterMask,
                Parameters,
                if PreviousMode != 0i8 { 1u8 } else { 0u8 },
            );

            *Response = ResponseReturnToCaller;
            return STATUS_SUCCESS;
        }
    }

    (*Message).h.u1.s1.DataLength = (core::mem::size_of::<HARDERROR_MSG>() - core::mem::size_of::<PORT_MESSAGE>()) as i16;
    (*Message).h.u1.s1.TotalLength = core::mem::size_of::<HARDERROR_MSG>() as i16;
    (*Message).h.u2.s2.Type = LPC_ERROR_EVENT as i16;
    (*Message).h.u2.s2.DataInfoOffset = 0;
    (*Message).Status = ErrorStatus & !(HARDERROR_OVERRIDE_ERRORMODE as i32);
    (*Message).ValidResponseOptions = ValidResponseOptions;
    (*Message).UnicodeStringParameterMask = UnicodeStringParameterMask;
    (*Message).NumberOfParameters = NumberOfParameters;
    KeQuerySystemTime(core::ptr::addr_of_mut!((*Message).ErrorTime) as *mut ntkrnl_sys::ddk::_LARGE_INTEGER);

    if !Parameters.is_null() {
        core::ptr::copy_nonoverlapping(
            Parameters as *const u8,
            (*Message).Parameters.as_mut_ptr() as *mut u8,
            core::mem::size_of::<ULONG_PTR>() * NumberOfParameters as usize,
        );
    }

    let Status = LpcRequestWaitReplyPort(
        PortHandle,
        Message as PPORT_MESSAGE,
        Message as PPORT_MESSAGE,
    );
    if Status >= 0 {
        if (*Message).Response != ResponseReturnToCaller
            && (*Message).Response != ResponseNotHandled
            && (*Message).Response != ResponseAbort
            && (*Message).Response != ResponseCancel
            && (*Message).Response != ResponseIgnore
            && (*Message).Response != ResponseNo
            && (*Message).Response != ResponseOk
            && (*Message).Response != ResponseRetry
            && (*Message).Response != ResponseYes
            && (*Message).Response != ResponseTryAgain
            && (*Message).Response != ResponseContinue
        {
            (*Message).Response = ResponseReturnToCaller;
        }

        *Response = (*Message).Response;
    } else {
        *Response = ResponseReturnToCaller;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ExRaiseAccessViolation() {
    RtlRaiseStatus(STATUS_ACCESS_VIOLATION);
}

#[no_mangle]
pub unsafe extern "system" fn ExRaiseDatatypeMisalignment() {
    RtlRaiseStatus(STATUS_DATATYPE_MISALIGNMENT);
}

#[no_mangle]
pub unsafe extern "system" fn ExSystemExceptionFilter() -> LONG {
    if KeGetPreviousMode() != 0i8 {
        1i32
    } else {
        0i32
    }
}

#[no_mangle]
pub unsafe extern "system" fn ExRaiseHardError(
    ErrorStatus: NTSTATUS,
    NumberOfParameters: ULONG,
    UnicodeStringParameterMask: ULONG,
    Parameters: PULONG_PTR,
    ValidResponseOptions: ULONG,
    Response: *mut ULONG,
) -> NTSTATUS {
    let Parameters = opaque_ptr(Parameters);
    let Response = opaque_ptr(Response);
    let mut CapturedParams: [UNICODE_STRING; MAXIMUM_HARDERROR_PARAMETERS] = core::mem::zeroed();
    let mut UserData: PVOID = core::ptr::null_mut();
    let mut SafeResponse = ResponseNotHandled;

    if !Parameters.is_null() {
        if UnicodeStringParameterMask != 0 {
            // SIZE_T: usize == u32 on i386, u64 on amd64/arm64 — matches Windows SIZE_T.
            let mut AllocSize: usize = (core::mem::size_of::<HARDERROR_USER_PARAMETERS>()
                - core::mem::size_of::<[u16; 1]>()) as usize;

            for i in 0..(NumberOfParameters as usize) {
                if (UnicodeStringParameterMask & (1 << i)) != 0 {
                    core::ptr::copy_nonoverlapping(
                        *Parameters.add(i) as *const u8,
                        core::ptr::addr_of_mut!(CapturedParams[i]) as *mut u8,
                        core::mem::size_of::<UNICODE_STRING>(),
                    );

                    AllocSize += CapturedParams[i].MaximumLength as usize;
                }
            }

            let Status = ZwAllocateVirtualMemory(
                (-1isize) as HANDLE,
                &mut UserData,
                0,
                core::ptr::addr_of_mut!(AllocSize) as *mut _,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if Status < 0 {
                *Response = ResponseNotHandled;
                return Status;
            }

            let UserParams = UserData as PHARDERROR_USER_PARAMETERS;
            let mut BufferBase = (*UserParams).Buffer.as_mut_ptr();

            for i in 0..(NumberOfParameters as usize) {
                if (UnicodeStringParameterMask & (1 << i)) != 0 {
                    // ULONG_PTR field: u32 on i386, u64 on amd64/arm64. Cast via usize then `as _`.
                    (*UserParams).Parameters[i] = core::ptr::addr_of_mut!((*UserParams).Strings[i]) as usize as _;

                    core::ptr::copy_nonoverlapping(
                        CapturedParams[i].Buffer as *const u8,
                        BufferBase as *mut u8,
                        CapturedParams[i].MaximumLength as usize,
                    );

                    CapturedParams[i].Buffer = BufferBase;
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!(CapturedParams[i]) as *const u8,
                        core::ptr::addr_of_mut!((*UserParams).Strings[i]) as *mut u8,
                        core::mem::size_of::<UNICODE_STRING>(),
                    );

                    BufferBase = BufferBase.add(CapturedParams[i].MaximumLength as usize);
                } else {
                    (*UserParams).Parameters[i] = *Parameters.add(i) as usize as _;
                }
            }
        } else {
            UserData = Parameters as PVOID;
        }
    }

    let Status = ExpRaiseHardError(
        ErrorStatus,
        NumberOfParameters,
        UnicodeStringParameterMask,
        UserData as PULONG_PTR,
        ValidResponseOptions,
        &mut SafeResponse,
    );

    if !UserData.is_null() && UserData != Parameters as PVOID {
        let mut FreeSize: usize = 0;
        ZwFreeVirtualMemory((-1isize) as HANDLE, &mut UserData, core::ptr::addr_of_mut!(FreeSize) as *mut _, MEM_RELEASE);
    }

    *Response = SafeResponse;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtRaiseHardError(
    ErrorStatus: NTSTATUS,
    NumberOfParameters: ULONG,
    UnicodeStringParameterMask: ULONG,
    Parameters: PULONG_PTR,
    ValidResponseOptions: ULONG,
    Response: *mut ULONG,
) -> NTSTATUS {
    let Parameters = opaque_ptr(Parameters);
    let Response = opaque_ptr(Response);
    let mut SafeParams: PULONG_PTR = core::ptr::null_mut();
    let mut SafeResponse = ResponseNotHandled;
    let mut SafeString: UNICODE_STRING = unsafe { core::mem::zeroed() };
    let mut ParamSize = 0usize;
    let PreviousMode = ExGetPreviousMode();

    if NumberOfParameters > MAXIMUM_HARDERROR_PARAMETERS as ULONG {
        return STATUS_INVALID_PARAMETER_2;
    }

    if !Parameters.is_null() && NumberOfParameters == 0 {
        return STATUS_INVALID_PARAMETER_2;
    }

    if PreviousMode != 0i8 {
        match ValidResponseOptions {
            OptionAbortRetryIgnore
            | OptionOk
            | OptionOkCancel
            | OptionRetryCancel
            | OptionYesNo
            | OptionYesNoCancel
            | OptionShutdownSystem
            | OptionOkNoWait
            | OptionCancelTryContinue => {}
            _ => return STATUS_INVALID_PARAMETER_4,
        }

        if !Parameters.is_null() {
            ParamSize = core::mem::size_of::<ULONG_PTR>() * NumberOfParameters as usize;

            SafeParams = ExAllocatePoolWithTag(PagedPool, ParamSize, TAG_ERR) as PULONG_PTR;
            if SafeParams.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        // The C wraps everything from here to the ExpRaiseHardError call in one
        // _SEH2_TRY, whose handler frees SafeParams and returns the exception code. It
        // contains FOUR probes and all four were dropped. Restored below in the C's own
        // order; on failure we free SafeParams and return the status, as the handler did.
        //
        // The third of them is the one worth pausing on: SafeParams[i] is a pointer that
        // was itself COPIED OUT OF USER MEMORY a few lines earlier, so it is entirely
        // caller-chosen. Dereferencing it unprobed reads sizeof(UNICODE_STRING) from any
        // address in the kernel — and the fourth probe then validates the Buffer pointer
        // inside the header that was just read, which is why SafeString looks unused
        // afterwards. It is not dead code; validating it IS its purpose, and dropping the
        // probes turned a capture-and-validate sequence into a raw pointer chase.
        if let Err(e) = probe_user_buffer(
            Response as *const core::ffi::c_void,
            core::mem::size_of::<ULONG>(),
            core::mem::size_of::<ULONG>(),
            PreviousMode,
        ) {
            if !SafeParams.is_null() {
                ExFreePoolWithTag(SafeParams as PVOID, TAG_ERR);
            }
            return e;
        }

        if !Parameters.is_null() {
            if let Err(e) = probe_user_buffer(
                Parameters as *const core::ffi::c_void,
                ParamSize,
                core::mem::size_of::<ULONG_PTR>(),
                PreviousMode,
            ) {
                ExFreePoolWithTag(SafeParams as PVOID, TAG_ERR);
                return e;
            }

            core::ptr::copy_nonoverlapping(
                Parameters as *const u8,
                SafeParams as *mut u8,
                ParamSize,
            );

            if UnicodeStringParameterMask != 0 {
                for i in 0..(NumberOfParameters as usize) {
                    if (UnicodeStringParameterMask & (1 << i)) != 0 {
                        if let Err(e) = probe_user_buffer(
                            *SafeParams.add(i) as *const core::ffi::c_void,
                            core::mem::size_of::<UNICODE_STRING>(),
                            core::mem::size_of::<ULONG_PTR>(),
                            PreviousMode,
                        ) {
                            ExFreePoolWithTag(SafeParams as PVOID, TAG_ERR);
                            return e;
                        }

                        core::ptr::copy_nonoverlapping(
                            *SafeParams.add(i) as *const u8,
                            core::ptr::addr_of_mut!(SafeString) as *mut u8,
                            core::mem::size_of::<UNICODE_STRING>(),
                        );

                        if let Err(e) = probe_user_buffer(
                            SafeString.Buffer as *const core::ffi::c_void,
                            SafeString.MaximumLength as usize,
                            core::mem::size_of::<UCHAR>(),
                            PreviousMode,
                        ) {
                            ExFreePoolWithTag(SafeParams as PVOID, TAG_ERR);
                            return e;
                        }
                    }
                }
            }
        }

        let mut Status = ExpRaiseHardError(
            ErrorStatus,
            NumberOfParameters,
            UnicodeStringParameterMask,
            SafeParams,
            ValidResponseOptions,
            &mut SafeResponse,
        );

        if !SafeParams.is_null() {
            ExFreePoolWithTag(SafeParams as PVOID, TAG_ERR);
        }

        match ntkrnl_sys::probe::write_back(Response, SafeResponse) {
            Ok(()) => Status,
            Err(s) => s,
        }
    } else {
        SafeParams = Parameters;

        let Status = ExRaiseHardError(
            ErrorStatus,
            NumberOfParameters,
            UnicodeStringParameterMask,
            SafeParams,
            ValidResponseOptions,
            &mut SafeResponse,
        );

        match ntkrnl_sys::probe::write_back(Response, SafeResponse) {
            Ok(()) => Status,
            Err(s) => s,
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtSetDefaultHardErrorPort(PortHandle: HANDLE) -> NTSTATUS {
    let PortHandle = opaque_ptr(PortHandle);
    let PreviousMode = ExGetPreviousMode();
    let mut Status = STATUS_UNSUCCESSFUL;

    if SeSinglePrivilegeCheck(
        SeTcbPrivilege,
        PreviousMode,
    ) == 0u8
    {
        return STATUS_PRIVILEGE_NOT_HELD;
    }

    if ExReadyForErrors == 0u8 {
        Status = ObReferenceObjectByHandle(
            PortHandle,
            0,
            LpcPortObjectType as POBJECT_TYPE,
            PreviousMode,
            &raw mut ExpDefaultErrorPort,
            core::ptr::null_mut(),
        );
        if Status >= 0 {
            ExpDefaultErrorPortProcess = PsGetCurrentProcess() as PEPROCESS;
            ObReferenceObject(ExpDefaultErrorPortProcess as PVOID);
            ExReadyForErrors = 1u8;
            Status = STATUS_SUCCESS;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "C" fn _purecall() {
    RtlRaiseStatus(STATUS_NOT_IMPLEMENTED);
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/ex/harderr.c
//  * PURPOSE:         Error Functions and Status/Exception Dispatching/Raising
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ******************************************************************/
//
// BOOLEAN ExReadyForErrors = FALSE;
// PVOID ExpDefaultErrorPort = NULL;
// PEPROCESS ExpDefaultErrorPortProcess = NULL;
//
// /* FUNCTIONS ****************************************************************/
//
// /*++
// * @name ExpSystemErrorHandler
// *
// * For now it's a stub
// *
// * @param ErrorStatus
// *        FILLME
// *
// * @param NumberOfParameters
// *        FILLME
// *
// * @param UnicodeStringParameterMask
// *        FILLME
// *
// * @param Parameters
// *        FILLME
// *
// * @param ValidResponseOptions
// *        FILLME
// *
// * @param Response
// *        FILLME
// *
// * @return None
// *
// * @remarks None
// *
// *--*/
// NTSTATUS
// NTAPI
// ExpSystemErrorHandler(IN NTSTATUS ErrorStatus,
//                       IN ULONG NumberOfParameters,
//                       IN ULONG UnicodeStringParameterMask,
//                       IN PULONG_PTR Parameters,
//                       IN BOOLEAN Shutdown)
// {
//     ULONG_PTR BugCheckParameters[MAXIMUM_HARDERROR_PARAMETERS] = {0, 0, 0, 0};
//     ULONG i;
//
//     /* Sanity check */
//     ASSERT(NumberOfParameters <= MAXIMUM_HARDERROR_PARAMETERS);
//
//     /*
//      * KeBugCheck expects MAXIMUM_HARDERROR_PARAMETERS parameters,
//      * but we might get called with less, so use a local buffer here.
//      */
//     for (i = 0; i < NumberOfParameters; i++)
//     {
//         /* Copy them over */
//         BugCheckParameters[i] = Parameters[i];
//     }
//
//     /* FIXME: STUB */
//     KeBugCheckEx(FATAL_UNHANDLED_HARD_ERROR,
//                  ErrorStatus,
//                  (ULONG_PTR)BugCheckParameters,
//                  0,
//                  0);
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name ExpRaiseHardError
//  * @implemented
//  *
//  * See ExRaiseHardError and NtRaiseHardError, same parameters.
//  *
//  * This function performs the central work for both ExRaiseHardError
//  * and NtRaiseHardError. ExRaiseHardError is the service for kernel-mode
//  * that copies the parameters to user-mode, and NtRaiseHardError is the
//  * service for both kernel-mode and user-mode that performs parameters
//  * validation and capture if necessary.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExpRaiseHardError(IN NTSTATUS ErrorStatus,
//                   IN ULONG NumberOfParameters,
//                   IN ULONG UnicodeStringParameterMask,
//                   IN PULONG_PTR Parameters,
//                   IN ULONG ValidResponseOptions,
//                   OUT PULONG Response)
// {
//     NTSTATUS Status;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PETHREAD Thread = PsGetCurrentThread();
//     UCHAR Buffer[PORT_MAXIMUM_MESSAGE_LENGTH];
//     PHARDERROR_MSG Message = (PHARDERROR_MSG)Buffer;
//     HANDLE PortHandle;
//     KPROCESSOR_MODE PreviousMode = KeGetPreviousMode();
//
//     PAGED_CODE();
//
//     /* Check if this error will shutdown the system */
//     if (ValidResponseOptions == OptionShutdownSystem)
//     {
//         /*
//          * Check if we have the privileges.
//          *
//          * NOTE: In addition to the Shutdown privilege we also check whether
//          * the caller has the Tcb privilege. The purpose is to allow only
//          * SYSTEM processes to "shutdown" the system on hard errors (BSOD)
//          * while forbidding regular processes to do so. This behaviour differs
//          * from Windows, where any user-mode process, as soon as it has the
//          * Shutdown privilege, can trigger a hard-error BSOD.
//          */
//         if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode) ||
//             !SeSinglePrivilegeCheck(SeShutdownPrivilege, PreviousMode))
//         {
//             /* No rights */
//             *Response = ResponseNotHandled;
//             return STATUS_PRIVILEGE_NOT_HELD;
//         }
//
//         /* Don't handle any new hard errors */
//         ExReadyForErrors = FALSE;
//     }
//
//     /* Check if hard errors are not disabled */
//     if (!Thread->HardErrorsAreDisabled)
//     {
//         /* Check if we can't do errors anymore, and this is serious */
//         if (!ExReadyForErrors && NT_ERROR(ErrorStatus))
//         {
//             /* Use the system handler */
//             ExpSystemErrorHandler(ErrorStatus,
//                                   NumberOfParameters,
//                                   UnicodeStringParameterMask,
//                                   Parameters,
//                                   (PreviousMode != KernelMode) ? TRUE : FALSE);
//         }
//     }
//
//     /*
//      * Enable hard error processing if it is enabled for the process
//      * or if the exception status forces it.
//      */
//     if ((Process->DefaultHardErrorProcessing & SEM_FAILCRITICALERRORS) ||
//         (ErrorStatus & HARDERROR_OVERRIDE_ERRORMODE))
//     {
//         /* Check if we have an exception port */
//         if (Process->ExceptionPort)
//         {
//             /* Use the port */
//             PortHandle = Process->ExceptionPort;
//         }
//         else
//         {
//             /* Use our default system port */
//             PortHandle = ExpDefaultErrorPort;
//         }
//     }
//     else
//     {
//         /* Don't process the error */
//         PortHandle = NULL;
//     }
//
//     /* If hard errors are disabled, do nothing */
//     if (Thread->HardErrorsAreDisabled) PortHandle = NULL;
//
//     /*
//      * If this is not the system thread, check whether hard errors are
//      * disabled for this thread on user-mode side, and if so, do nothing.
//      */
//     if (!Thread->SystemThread && (PortHandle != NULL))
//     {
//         /* Check if we have a TEB */
//         PTEB Teb = PsGetCurrentThread()->Tcb.Teb;
//         if (Teb)
//         {
//             _SEH2_TRY
//             {
//                 if (Teb->HardErrorMode & RTL_SEM_FAILCRITICALERRORS)
//                 {
//                     PortHandle = NULL;
//                 }
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 NOTHING;
//             }
//             _SEH2_END;
//         }
//     }
//
//     /* Now check if we have a port */
//     if (PortHandle == NULL)
//     {
//         /* Just return to caller */
//         *Response = ResponseReturnToCaller;
//         return STATUS_SUCCESS;
//     }
//
//     /* Check if this is the default process */
//     if (Process == ExpDefaultErrorPortProcess)
//     {
//         /* We can't handle the error, check if this is critical */
//         if (NT_ERROR(ErrorStatus))
//         {
//             /* It is, invoke the system handler */
//             ExpSystemErrorHandler(ErrorStatus,
//                                   NumberOfParameters,
//                                   UnicodeStringParameterMask,
//                                   Parameters,
//                                   (PreviousMode != KernelMode) ? TRUE : FALSE);
//
//             /* If we survived, return to caller */
//             *Response = ResponseReturnToCaller;
//             return STATUS_SUCCESS;
//         }
//     }
//
//     /* Setup the LPC Message */
//     Message->h.u1.Length = (sizeof(HARDERROR_MSG) << 16) |
//                            (sizeof(HARDERROR_MSG) - sizeof(PORT_MESSAGE));
//     Message->h.u2.ZeroInit = 0;
//     Message->h.u2.s2.Type = LPC_ERROR_EVENT;
//     Message->Status = ErrorStatus & ~HARDERROR_OVERRIDE_ERRORMODE;
//     Message->ValidResponseOptions = ValidResponseOptions;
//     Message->UnicodeStringParameterMask = UnicodeStringParameterMask;
//     Message->NumberOfParameters = NumberOfParameters;
//     KeQuerySystemTime(&Message->ErrorTime);
//
//     /* Copy the parameters */
//     if (Parameters)
//     {
//         RtlMoveMemory(&Message->Parameters,
//                       Parameters,
//                       sizeof(ULONG_PTR) * NumberOfParameters);
//     }
//
//     /* Send the LPC Message */
//     Status = LpcRequestWaitReplyPort(PortHandle,
//                                      (PPORT_MESSAGE)Message,
//                                      (PPORT_MESSAGE)Message);
//     if (NT_SUCCESS(Status))
//     {
//         /* Check what kind of response we got */
//         if ((Message->Response != ResponseReturnToCaller) &&
//             (Message->Response != ResponseNotHandled) &&
//             (Message->Response != ResponseAbort) &&
//             (Message->Response != ResponseCancel) &&
//             (Message->Response != ResponseIgnore) &&
//             (Message->Response != ResponseNo) &&
//             (Message->Response != ResponseOk) &&
//             (Message->Response != ResponseRetry) &&
//             (Message->Response != ResponseYes) &&
//             (Message->Response != ResponseTryAgain) &&
//             (Message->Response != ResponseContinue))
//         {
//             /* Reset to a default one */
//             Message->Response = ResponseReturnToCaller;
//         }
//
//         /* Set the response */
//         *Response = Message->Response;
//     }
//     else
//     {
//         /* Set the response */
//         *Response = ResponseReturnToCaller;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name ExRaiseAccessViolation
//  * @implemented
//  *
//  * The ExRaiseAccessViolation routine can be used with structured exception
//  * handling to throw a driver-determined exception for a memory access
//  * violation that occurs when a driver processes I/O requests.
//  * See: http://msdn.microsoft.com/library/en-us/Kernel_r/hh/Kernel_r/k102_71b4c053-599c-4a6d-8a59-08aae6bdc534.xml.asp?frame=true (DEAD_LINK)
//  *      https://web.archive.org/web/20081231000748/http://www.osronline.com/ddkx/kmarch/k102_814i.htm
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// ExRaiseAccessViolation(VOID)
// {
//     /* Raise the Right Status */
//     RtlRaiseStatus(STATUS_ACCESS_VIOLATION);
// }
//
// /*++
//  * @name ExRaiseDatatypeMisalignment
//  * @implemented
//  *
//  * ExRaiseDatatypeMisalignment raises an exception with the exception
//  * code set to STATUS_DATATYPE_MISALIGNMENT
//  * See: MSDN / DDK
//  *      https://web.archive.org/web/20081231000748/http://www.osronline.com/ddkx/kmarch/k102_814i.htm
//  *
//  * @return None
//  *
//  * @remarks None
//  *
//  *--*/
// VOID
// NTAPI
// ExRaiseDatatypeMisalignment(VOID)
// {
//     /* Raise the Right Status */
//     RtlRaiseStatus(STATUS_DATATYPE_MISALIGNMENT);
// }
//
// /*++
//  * @name ExSystemExceptionFilter
//  * @implemented
//  *
//  * TODO: Add description
//  *
//  * @return FILLME
//  *
//  * @remarks None
//  *
//  *--*/
// LONG
// NTAPI
// ExSystemExceptionFilter(VOID)
// {
//     return KeGetPreviousMode() != KernelMode ?
//            EXCEPTION_EXECUTE_HANDLER : EXCEPTION_CONTINUE_SEARCH;
// }
//
// /*++
//  * @name ExRaiseHardError
//  * @implemented
//  *
//  * See NtRaiseHardError and ExpRaiseHardError.
//  *
//  * @param ErrorStatus
//  *        Error Code
//  *
//  * @param NumberOfParameters
//  *        Number of optional parameters in Parameters array
//  *
//  * @param UnicodeStringParameterMask
//  *        Optional string parameter (can be only one per error code)
//  *
//  * @param Parameters
//  *        Array of ULONG parameters for use in error message string
//  *
//  * @param ValidResponseOptions
//  *        See HARDERROR_RESPONSE_OPTION for possible values description
//  *
//  * @param Response
//  *        Pointer to HARDERROR_RESPONSE enumeration
//  *
//  * @return Status
//  *
//  *--*/
// NTSTATUS
// NTAPI
// ExRaiseHardError(IN NTSTATUS ErrorStatus,
//                  IN ULONG NumberOfParameters,
//                  IN ULONG UnicodeStringParameterMask,
//                  IN PULONG_PTR Parameters,
//                  IN ULONG ValidResponseOptions,
//                  OUT PULONG Response)
// {
//     NTSTATUS Status;
//     SIZE_T Size;
//     UNICODE_STRING CapturedParams[MAXIMUM_HARDERROR_PARAMETERS];
//     ULONG i;
//     PVOID UserData = NULL;
//     PHARDERROR_USER_PARAMETERS UserParams;
//     PWSTR BufferBase;
//     ULONG SafeResponse = ResponseNotHandled;
//
//     PAGED_CODE();
//
//     /* Check if we have parameters */
//     if (Parameters)
//     {
//         /* Check if we have strings */
//         if (UnicodeStringParameterMask)
//         {
//             /* Calculate the required size */
//             Size = FIELD_OFFSET(HARDERROR_USER_PARAMETERS, Buffer[0]);
//
//             /* Loop each parameter */
//             for (i = 0; i < NumberOfParameters; i++)
//             {
//                 /* Check if it's part of the mask */
//                 if (UnicodeStringParameterMask & (1 << i))
//                 {
//                     /* Copy it */
//                     RtlMoveMemory(&CapturedParams[i],
//                                   (PVOID)Parameters[i],
//                                   sizeof(UNICODE_STRING));
//
//                     /* Increase the size */
//                     Size += CapturedParams[i].MaximumLength;
//                 }
//             }
//
//             /* Allocate the user data region */
//             Status = ZwAllocateVirtualMemory(NtCurrentProcess(),
//                                              &UserData,
//                                              0,
//                                              &Size,
//                                              MEM_COMMIT,
//                                              PAGE_READWRITE);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Return failure */
//                 *Response = ResponseNotHandled;
//                 return Status;
//             }
//
//             /* Set the pointers to our data */
//             UserParams = UserData;
//             BufferBase = UserParams->Buffer;
//
//             /* Enter SEH block as we are writing to user-mode space */
//             _SEH2_TRY
//             {
//                 /* Loop parameters again */
//                 for (i = 0; i < NumberOfParameters; i++)
//                 {
//                     /* Check if we are in the mask */
//                     if (UnicodeStringParameterMask & (1 << i))
//                     {
//                         /* Update the base */
//                         UserParams->Parameters[i] = (ULONG_PTR)&UserParams->Strings[i];
//
//                         /* Copy the string buffer */
//                         RtlMoveMemory(BufferBase,
//                                       CapturedParams[i].Buffer,
//                                       CapturedParams[i].MaximumLength);
//
//                         /* Set buffer */
//                         CapturedParams[i].Buffer = BufferBase;
//
//                         /* Copy the string structure */
//                         UserParams->Strings[i] = CapturedParams[i];
//
//                         /* Update the pointer */
//                         BufferBase += CapturedParams[i].MaximumLength;
//                     }
//                     else
//                     {
//                         /* No need to copy any strings */
//                         UserParams->Parameters[i] = Parameters[i];
//                     }
//                 }
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 /* Return the exception code */
//                 Status = _SEH2_GetExceptionCode();
//                 DPRINT1("ExRaiseHardError - Exception when writing data to user-mode, Status 0x%08lx\n", Status);
//             }
//             _SEH2_END;
//         }
//         else
//         {
//             /* Just keep the data as is */
//             UserData = Parameters;
//         }
//     }
//
//     /* Now call the worker function */
//     Status = ExpRaiseHardError(ErrorStatus,
//                                NumberOfParameters,
//                                UnicodeStringParameterMask,
//                                UserData,
//                                ValidResponseOptions,
//                                &SafeResponse);
//
//     /* Check if we had done user-mode allocation */
//     if ((UserData) && (UserData != Parameters))
//     {
//         /* We did! Delete it */
//         Size = 0;
//         ZwFreeVirtualMemory(NtCurrentProcess(),
//                             &UserData,
//                             &Size,
//                             MEM_RELEASE);
//     }
//
//     /* Return status and the response */
//     *Response = SafeResponse;
//     return Status;
// }
//
// /*++
//  * @name NtRaiseHardError
//  * @implemented
//  *
//  * This function sends HARDERROR_MSG LPC message to a hard-error listener,
//  * typically CSRSS.EXE. See NtSetDefaultHardErrorPort for more information.
//  * See also: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Error/NtRaiseHardError.html
//  *
//  * @param ErrorStatus
//  *        Error Code
//  *
//  * @param NumberOfParameters
//  *        Number of optional parameters in Parameters array
//  *
//  * @param UnicodeStringParameterMask
//  *        Optional string parameter (can be only one per error code)
//  *
//  * @param Parameters
//  *        Array of ULONG_PTR parameters for use in error message string
//  *
//  * @param ValidResponseOptions
//  *        See HARDERROR_RESPONSE_OPTION for possible values description
//  *
//  * @param Response
//  *        Pointer to HARDERROR_RESPONSE enumeration
//  *
//  * @return Status
//  *
//  * @remarks NtRaiseHardError constitutes an easy way to display messages
//  *          in GUI without loading any Win32 API libraries.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtRaiseHardError(IN NTSTATUS ErrorStatus,
//                  IN ULONG NumberOfParameters,
//                  IN ULONG UnicodeStringParameterMask,
//                  IN PULONG_PTR Parameters,
//                  IN ULONG ValidResponseOptions,
//                  OUT PULONG Response)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PULONG_PTR SafeParams = NULL;
//     ULONG SafeResponse = ResponseNotHandled;
//     UNICODE_STRING SafeString;
//     ULONG i;
//     ULONG ParamSize = 0;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//
//     PAGED_CODE();
//
//     /* Validate parameter count */
//     if (NumberOfParameters > MAXIMUM_HARDERROR_PARAMETERS)
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER_2;
//     }
//
//     /* Make sure we have some at least */
//     if ((Parameters != NULL) && (NumberOfParameters == 0))
//     {
//         /* Fail */
//         return STATUS_INVALID_PARAMETER_2;
//     }
//
//     /* Check if we were called from user-mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* First validate the responses */
//         switch (ValidResponseOptions)
//         {
//             /* Check all valid cases */
//             case OptionAbortRetryIgnore:
//             case OptionOk:
//             case OptionOkCancel:
//             case OptionRetryCancel:
//             case OptionYesNo:
//             case OptionYesNoCancel:
//             case OptionShutdownSystem:
//             case OptionOkNoWait:
//             case OptionCancelTryContinue:
//                 break;
//
//             /* Anything else is invalid */
//             default:
//                 return STATUS_INVALID_PARAMETER_4;
//         }
//
//         /* Check if we have parameters */
//         if (Parameters)
//         {
//             /* Calculate size of the parameters */
//             ParamSize = sizeof(ULONG_PTR) * NumberOfParameters;
//
//             /* Allocate a safe buffer */
//             SafeParams = ExAllocatePoolWithTag(PagedPool, ParamSize, TAG_ERR);
//             if (!SafeParams)
//             {
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//         }
//
//         /* Enter SEH Block */
//         _SEH2_TRY
//         {
//             /* Validate the response pointer */
//             ProbeForWriteUlong(Response);
//
//             /* Check if we have parameters */
//             if (Parameters)
//             {
//                 /* Validate the parameter pointers */
//                 ProbeForRead(Parameters, ParamSize, sizeof(ULONG_PTR));
//
//                 /* Copy them */
//                 RtlCopyMemory(SafeParams, Parameters, ParamSize);
//
//                 /* Now check if there's strings in it */
//                 if (UnicodeStringParameterMask)
//                 {
//                     /* Loop every string */
//                     for (i = 0; i < NumberOfParameters; i++)
//                     {
//                         /* Check if this parameter is a string */
//                         if (UnicodeStringParameterMask & (1 << i))
//                         {
//                             /* Probe the structure */
//                             ProbeForRead((PVOID)SafeParams[i],
//                                          sizeof(UNICODE_STRING),
//                                          sizeof(ULONG_PTR));
//
//                             /* Capture it */
//                             RtlCopyMemory(&SafeString,
//                                           (PVOID)SafeParams[i],
//                                           sizeof(UNICODE_STRING));
//
//                             /* Probe the buffer */
//                             ProbeForRead(SafeString.Buffer,
//                                          SafeString.MaximumLength,
//                                          sizeof(UCHAR));
//                         }
//                     }
//                 }
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Free captured buffer */
//             if (SafeParams) ExFreePoolWithTag(SafeParams, TAG_ERR);
//
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//
//         /* Call the system function directly, because we probed */
//         Status = ExpRaiseHardError(ErrorStatus,
//                                    NumberOfParameters,
//                                    UnicodeStringParameterMask,
//                                    SafeParams,
//                                    ValidResponseOptions,
//                                    &SafeResponse);
//
//         /* Free captured buffer */
//         if (SafeParams) ExFreePoolWithTag(SafeParams, TAG_ERR);
//
//         /* Enter SEH Block to return the response */
//         _SEH2_TRY
//         {
//             /* Return the response */
//             *Response = SafeResponse;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Reuse variable */
//         SafeParams = Parameters;
//
//         /*
//          * Call the Executive Function. It will probe
//          * and copy pointers to user-mode.
//          */
//         Status = ExRaiseHardError(ErrorStatus,
//                                   NumberOfParameters,
//                                   UnicodeStringParameterMask,
//                                   SafeParams,
//                                   ValidResponseOptions,
//                                   &SafeResponse);
//
//         /* Return the response */
//         *Response = SafeResponse;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name NtSetDefaultHardErrorPort
//  * @implemented
//  *
//  * NtSetDefaultHardErrorPort is typically called only once. After the call,
//  * the kernel sets a BOOLEAN flag named ExReadyForErrors to TRUE, and all other
//  * attempts to change the default port fail with STATUS_UNSUCCESSFUL error code.
//  * See: http://undocumented.ntinternals.net/UserMode/Undocumented%20Functions/Error/NtSetDefaultHardErrorPort.html
//  *      https://web.archive.org/web/20070716133753/http://www.windowsitlibrary.com/Content/356/08/2.html
//  *
//  * @param PortHandle
//  *        Handle to named port object
//  *
//  * @return Status
//  *
//  * @remarks Privileges: SE_TCB_PRIVILEGE
//  *
//  *--*/
// NTSTATUS
// NTAPI
// NtSetDefaultHardErrorPort(IN HANDLE PortHandle)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     PAGED_CODE();
//
//     /* Check if we have the privileges */
//     if (!SeSinglePrivilegeCheck(SeTcbPrivilege, PreviousMode))
//     {
//         DPRINT1("NtSetDefaultHardErrorPort: Caller requires "
//                 "the SeTcbPrivilege privilege!\n");
//         return STATUS_PRIVILEGE_NOT_HELD;
//     }
//
//     /* Only called once during bootup, make sure we weren't called yet */
//     if (!ExReadyForErrors)
//     {
//         /* Reference the hard-error port */
//         Status = ObReferenceObjectByHandle(PortHandle,
//                                            0,
//                                            LpcPortObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&ExpDefaultErrorPort,
//                                            NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Keep also a reference to the process handling the hard errors */
//             ExpDefaultErrorPortProcess = PsGetCurrentProcess();
//             ObReferenceObject(ExpDefaultErrorPortProcess);
//             ExReadyForErrors = TRUE;
//             Status = STATUS_SUCCESS;
//         }
//     }
//
//     /* Return status to caller */
//     return Status;
// }
//
// VOID
// __cdecl
// _purecall(VOID)
// {
//     /* Not supported in Kernel Mode */
//     RtlRaiseStatus(STATUS_NOT_IMPLEMENTED);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
