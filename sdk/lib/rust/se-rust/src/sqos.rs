use ntkrnl_sys::ddk::*;
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES, NT_SUCCESS, TRUE, FALSE};
use ntkrnl_sys::constants::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::probe::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

#[no_mangle]
pub unsafe extern "system" fn SepCaptureSecurityQualityOfService(
    ObjectAttributes: POBJECT_ATTRIBUTES,
    AccessMode: KPROCESSOR_MODE,
    PoolType: POOL_TYPE,
    CaptureIfKernel: BOOLEAN,
    CapturedSecurityQualityOfService: *mut PSECURITY_QUALITY_OF_SERVICE,
    Present: PBOOLEAN,
) -> NTSTATUS {
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let CapturedSecurityQualityOfService = opaque_ptr(CapturedSecurityQualityOfService);
    let Present = opaque_ptr(Present);
    let mut captured_qos: PSECURITY_QUALITY_OF_SERVICE;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if !ObjectAttributes.is_null() {
        if AccessMode != 0i8 {
            let mut safe_qos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();

            // ProbeForRead ObjectAttributes
            if let Err(s) = probe_user_buffer(
                ObjectAttributes as *const core::ffi::c_void,
                core::mem::size_of::<_OBJECT_ATTRIBUTES>(),
                core::mem::size_of::<ULONG>(),
                AccessMode,
            ) {
                return s;
            }
            // Capture once: the C reads Length, then SecurityQualityOfService three
            // times (probe, Length check, copy), each from live user memory. Another
            // thread could swap the pointer between the probe and the copy.
            let oa: _OBJECT_ATTRIBUTES = match read_back(ObjectAttributes as *const _OBJECT_ATTRIBUTES) {
                Ok(v) => v,
                Err(s) => return s,
            };
            let user_qos = oa.SecurityQualityOfService as *const core::ffi::c_void;
            if oa.Length == core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG {
                if !user_qos.is_null() {
                    // ProbeForRead + RtlCopyMemory inside _SEH2_TRY. The Length check
                    // reads the captured copy, which is the value the copy carries.
                    if let Err(s) = safe_copy_from_user(
                        core::ptr::addr_of_mut!(safe_qos) as *mut core::ffi::c_void,
                        user_qos,
                        core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>(),
                        core::mem::align_of::<ULONG>(),
                        AccessMode,
                    ) {
                        return s;
                    }
                    if safe_qos.Length == core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG {
                        // Present and CapturedSecurityQualityOfService are kernel-mode output pointers — direct write
                        *Present = 1u8;
                    } else {
                        status = STATUS_INVALID_PARAMETER;
                    }
                } else {
                    // Kernel-mode output pointers — direct write, no probe
                    *CapturedSecurityQualityOfService = core::ptr::null_mut();
                    *Present = 0u8;
                }
            } else {
                status = STATUS_INVALID_PARAMETER;
            }

            if status >= 0 {
                if *Present != 0 {
                    captured_qos = ExAllocatePoolWithTag(
                        PoolType,
                        core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as SIZE_T,
                        TAG_QOS,
                    ) as PSECURITY_QUALITY_OF_SERVICE;
                    if !captured_qos.is_null() {
                        core::ptr::copy_nonoverlapping(
                            core::ptr::addr_of!(safe_qos) as *const u8,
                            captured_qos as *mut u8,
                            core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>(),
                        );
                        *CapturedSecurityQualityOfService = captured_qos;
                    } else {
                        status = STATUS_INSUFFICIENT_RESOURCES;
                    }
                } else {
                    *CapturedSecurityQualityOfService = core::ptr::null_mut();
                }
            }
        } else {
            if (*ObjectAttributes).Length == core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG {
                if CaptureIfKernel != 0 {
                    if !(*ObjectAttributes).SecurityQualityOfService.is_null() {
                        if (*((*ObjectAttributes).SecurityQualityOfService as PSECURITY_QUALITY_OF_SERVICE)).Length
                            == core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as ULONG
                        {
                            captured_qos = ExAllocatePoolWithTag(
                                PoolType,
                                core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as SIZE_T,
                                TAG_QOS,
                            ) as PSECURITY_QUALITY_OF_SERVICE;
                            if !captured_qos.is_null() {
                                core::ptr::copy_nonoverlapping(
                                    (*ObjectAttributes).SecurityQualityOfService as *const u8,
                                    captured_qos as *mut u8,
                                    core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>(),
                                );
                                *CapturedSecurityQualityOfService = captured_qos;
                                *Present = 1u8;
                            } else {
                                status = STATUS_INSUFFICIENT_RESOURCES;
                            }
                        } else {
                            status = STATUS_INVALID_PARAMETER;
                        }
                    } else {
                        *CapturedSecurityQualityOfService = core::ptr::null_mut();
                        *Present = 0u8;
                    }
                } else {
                    *CapturedSecurityQualityOfService = (*ObjectAttributes).SecurityQualityOfService as PSECURITY_QUALITY_OF_SERVICE;
                    *Present = if !(*ObjectAttributes).SecurityQualityOfService.is_null() { 1u8 } else { 0u8 };
                }
            } else {
                status = STATUS_INVALID_PARAMETER;
            }
        }
    } else {
        *CapturedSecurityQualityOfService = core::ptr::null_mut();
        *Present = 0u8;
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn SepReleaseSecurityQualityOfService(
    CapturedSecurityQualityOfService: PSECURITY_QUALITY_OF_SERVICE,
    AccessMode: KPROCESSOR_MODE,
    CaptureIfKernel: BOOLEAN,
) {
    if !CapturedSecurityQualityOfService.is_null()
        && (AccessMode != 0i8 || CaptureIfKernel != 0)
    {
        ExFreePoolWithTag(CapturedSecurityQualityOfService as PVOID, TAG_QOS);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Security Quality of Service (SQoS) implementation support
//  * COPYRIGHT:   Copyright David Welch <welch@cwcom.net>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS *******************************************************************/
//
// /**
//  * @brief
//  * Captures the security quality of service data given the object
//  * attributes from an object.
//  *
//  * @param[in] ObjectAttributes
//  * Attributes of an object where SQOS is to be retrieved. If the caller
//  * doesn't fill object attributes to the function, it automatically assumes
//  * SQOS is not present, or if, there's no SQOS present in the object attributes list
//  * of the object itself.
//  *
//  * @param[in] AccessMode
//  * Processor access mode.
//  *
//  * @param[in] PoolType
//  * The pool type for the captured SQOS to be used for allocation.
//  *
//  * @param[in] CaptureIfKernel
//  * Capture access condition. To be set to TRUE if the capture is done within the kernel,
//  * FALSE if the capture is done in a kernel mode driver or user mode otherwise.
//  *
//  * @param[out] CapturedSecurityQualityOfService
//  * The captured SQOS data from the object.
//  *
//  * @param[out] Present
//  * Returns TRUE if SQOS is present in an object, FALSE otherwise. FALSE is also immediately
//  * returned if no object attributes is given to the call.
//  *
//  * @return
//  * STATUS_SUCCESS if SQOS from the object has been fully and successfully captured. STATUS_INVALID_PARAMETER
//  * if the caller submits an invalid object attributes list. STATUS_INSUFFICIENT_RESOURCES if the function has
//  * failed to allocate some resources in the pool for the captured SQOS. A failure NTSTATUS code is returned
//  * otherwise.
//  */
// NTSTATUS
// NTAPI
// SepCaptureSecurityQualityOfService(
//     _In_opt_ POBJECT_ATTRIBUTES ObjectAttributes,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ POOL_TYPE PoolType,
//     _In_ BOOLEAN CaptureIfKernel,
//     _Out_ PSECURITY_QUALITY_OF_SERVICE *CapturedSecurityQualityOfService,
//     _Out_ PBOOLEAN Present)
// {
//     PSECURITY_QUALITY_OF_SERVICE CapturedQos;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     PAGED_CODE();
//
//     ASSERT(CapturedSecurityQualityOfService);
//     ASSERT(Present);
//
//     if (ObjectAttributes != NULL)
//     {
//         if (AccessMode != KernelMode)
//         {
//             SECURITY_QUALITY_OF_SERVICE SafeQos;
//
//             _SEH2_TRY
//             {
//                 ProbeForRead(ObjectAttributes,
//                              sizeof(OBJECT_ATTRIBUTES),
//                              sizeof(ULONG));
//                 if (ObjectAttributes->Length == sizeof(OBJECT_ATTRIBUTES))
//                 {
//                     if (ObjectAttributes->SecurityQualityOfService != NULL)
//                     {
//                         ProbeForRead(ObjectAttributes->SecurityQualityOfService,
//                                      sizeof(SECURITY_QUALITY_OF_SERVICE),
//                                      sizeof(ULONG));
//
//                         if (((PSECURITY_QUALITY_OF_SERVICE)ObjectAttributes->SecurityQualityOfService)->Length ==
//                             sizeof(SECURITY_QUALITY_OF_SERVICE))
//                         {
//                             /*
//                              * Don't allocate memory here because ExAllocate should bugcheck
//                              * the system if it's buggy, SEH would catch that! So make a local
//                              * copy of the qos structure.
//                              */
//                             RtlCopyMemory(&SafeQos,
//                                           ObjectAttributes->SecurityQualityOfService,
//                                           sizeof(SECURITY_QUALITY_OF_SERVICE));
//                             *Present = TRUE;
//                         }
//                         else
//                         {
//                             Status = STATUS_INVALID_PARAMETER;
//                         }
//                     }
//                     else
//                     {
//                         *CapturedSecurityQualityOfService = NULL;
//                         *Present = FALSE;
//                     }
//                 }
//                 else
//                 {
//                     Status = STATUS_INVALID_PARAMETER;
//                 }
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//
//             if (NT_SUCCESS(Status))
//             {
//                 if (*Present)
//                 {
//                     CapturedQos = ExAllocatePoolWithTag(PoolType,
//                                                         sizeof(SECURITY_QUALITY_OF_SERVICE),
//                                                         TAG_QOS);
//                     if (CapturedQos != NULL)
//                     {
//                         RtlCopyMemory(CapturedQos,
//                                       &SafeQos,
//                                       sizeof(SECURITY_QUALITY_OF_SERVICE));
//                         *CapturedSecurityQualityOfService = CapturedQos;
//                     }
//                     else
//                     {
//                         Status = STATUS_INSUFFICIENT_RESOURCES;
//                     }
//                 }
//                 else
//                 {
//                     *CapturedSecurityQualityOfService = NULL;
//                 }
//             }
//         }
//         else
//         {
//             if (ObjectAttributes->Length == sizeof(OBJECT_ATTRIBUTES))
//             {
//                 if (CaptureIfKernel)
//                 {
//                     if (ObjectAttributes->SecurityQualityOfService != NULL)
//                     {
//                         if (((PSECURITY_QUALITY_OF_SERVICE)ObjectAttributes->SecurityQualityOfService)->Length ==
//                             sizeof(SECURITY_QUALITY_OF_SERVICE))
//                         {
//                             CapturedQos = ExAllocatePoolWithTag(PoolType,
//                                                                 sizeof(SECURITY_QUALITY_OF_SERVICE),
//                                                                 TAG_QOS);
//                             if (CapturedQos != NULL)
//                             {
//                                 RtlCopyMemory(CapturedQos,
//                                               ObjectAttributes->SecurityQualityOfService,
//                                               sizeof(SECURITY_QUALITY_OF_SERVICE));
//                                 *CapturedSecurityQualityOfService = CapturedQos;
//                                 *Present = TRUE;
//                             }
//                             else
//                             {
//                                 Status = STATUS_INSUFFICIENT_RESOURCES;
//                             }
//                         }
//                         else
//                         {
//                             Status = STATUS_INVALID_PARAMETER;
//                         }
//                     }
//                     else
//                     {
//                         *CapturedSecurityQualityOfService = NULL;
//                         *Present = FALSE;
//                     }
//                 }
//                 else
//                 {
//                     *CapturedSecurityQualityOfService = (PSECURITY_QUALITY_OF_SERVICE)ObjectAttributes->SecurityQualityOfService;
//                     *Present = (ObjectAttributes->SecurityQualityOfService != NULL);
//                 }
//             }
//             else
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
//     else
//     {
//         *CapturedSecurityQualityOfService = NULL;
//         *Present = FALSE;
//     }
//
//     return Status;
// }
//
// /**
//  * @brief
//  * Releases (frees) the captured SQOS data from an object in the memory pool.
//  *
//  * @param[in] CapturedSecurityQualityOfService
//  * The captured SQOS data to be released.
//  *
//  * @param[in] AccessMode
//  * Processor access mode.
//  *
//  * @param[in] CaptureIfKernel
//  * Capture access condition. To be set to TRUE if the capture is done within the kernel,
//  * FALSE if the capture is done in a kernel mode driver or user mode otherwise.
//  *
//  * @return
//  * Nothing.
//  */
// VOID
// NTAPI
// SepReleaseSecurityQualityOfService(
//     _In_opt_ PSECURITY_QUALITY_OF_SERVICE CapturedSecurityQualityOfService,
//     _In_ KPROCESSOR_MODE AccessMode,
//     _In_ BOOLEAN CaptureIfKernel)
// {
//     PAGED_CODE();
//
//     if (CapturedSecurityQualityOfService != NULL &&
//         (AccessMode != KernelMode || CaptureIfKernel))
//     {
//         ExFreePoolWithTag(CapturedSecurityQualityOfService, TAG_QOS);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
