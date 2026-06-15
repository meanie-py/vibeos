use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const QUERY_ACTCTX_FLAG_VALID: u32 = QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX |
                                         QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE |
                                         QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS |
                                         QUERY_ACTCTX_FLAG_NO_ADDREF;
#[no_mangle]
pub unsafe extern "system" fn BasepFreeActivationContextActivationBlock(
    ActivationBlock: PBASEP_ACTCTX_BLOCK,
) {
    if ActivationBlock.is_null() {
        return;
    }
    if !(*ActivationBlock).ActivationContext.is_null() {
        RtlReleaseActivationContext((*ActivationBlock).ActivationContext);
        (*ActivationBlock).ActivationContext = core::ptr::null_mut();
    }
    RtlFreeHeap(RtlGetProcessHeap(), 0, ActivationBlock as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn BasepAllocateActivationContextActivationBlock(
    Flags: DWORD,
    CompletionRoutine: PVOID,
    CompletionContext: PVOID,
    ActivationBlock: *mut PBASEP_ACTCTX_BLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ContextInfo: ACTIVATION_CONTEXT_BASIC_INFORMATION;

    /* Clear the info structure */
    ContextInfo.dwFlags = 0;
    ContextInfo.hActCtx = core::ptr::null_mut();

    /* Assume failure */
    if !ActivationBlock.is_null() {
        *ActivationBlock = core::ptr::null_mut();
    }

    /* Only support valid flags */
    if (Flags & !(1 | 2)) != 0 {
        /* Fail if unknown flags are passed in */
        Status = STATUS_INVALID_PARAMETER_1;
    } else if ActivationBlock.is_null() {
        /* Fail otherwise */
        Status = STATUS_INVALID_PARAMETER_4;
    } else {
        /* Query RTL for information on the current activation context */
        Status = RtlQueryInformationActivationContext(
            RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT as DWORD,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            ActivationContextBasicInformation as ULONG,
            &raw mut ContextInfo as PVOID,
            core::mem::size_of::<ACTIVATION_CONTEXT_BASIC_INFORMATION>() as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            /* Failed -- bail out */
            // DPRINT1 macro omitted - debug print not available
            // goto Quickie handled by fallthrough
        } else {
            /* Check if the current one should be freed */
            if (ContextInfo.dwFlags & 1) != 0 {
                /* Release and clear it */
                RtlReleaseActivationContext(ContextInfo.hActCtx);
                ContextInfo.hActCtx = core::ptr::null_mut();
            }

            /* Check if there's an active context, or if the caller is forcing one */
            if (Flags & 2) == 0 || !ContextInfo.hActCtx.is_null() {
                /* Allocate the block */
                *ActivationBlock = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    core::mem::size_of::<BASEP_ACTCTX_BLOCK>() as SIZE_T,
                ) as PBASEP_ACTCTX_BLOCK;
                if (*ActivationBlock).is_null() {
                    /* Ran out of memory, fail */
                    Status = STATUS_NO_MEMORY;
                } else {
                    /* Fill it out */
                    (**ActivationBlock).ActivationContext = ContextInfo.hActCtx as PVOID;
                    (**ActivationBlock).Flags = 0;
                    if (Flags & 1) != 0 {
                        (**ActivationBlock).Flags |= 1;
                    }
                    (**ActivationBlock).CompletionRoutine = CompletionRoutine;
                    (**ActivationBlock).CompletionContext = CompletionContext;

                    /* Tell Quickie below not to free anything, since this is success */
                    ContextInfo.hActCtx = core::ptr::null_mut();

                    /* Set success status */
                    Status = STATUS_SUCCESS;
                }
            } else {
                /* Set success status */
                Status = STATUS_SUCCESS;
            }
        }
    }

    /* Failure or success path, return to caller and free on failure */
    if !ContextInfo.hActCtx.is_null() {
        RtlReleaseActivationContext(ContextInfo.hActCtx);
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn BasepProbeForDllManifest(
    DllHandle: PVOID,
    FullDllName: PCWSTR,
    ActCtx: *mut PVOID,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Info: LDR_RESOURCE_INFO = core::mem::zeroed();
    let mut Entry: *mut IMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let mut Context: ACTCTXW = core::mem::zeroed();
    let mut Result: HANDLE = core::ptr::null_mut();

    if ActCtx.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    *ActCtx = core::ptr::null_mut();

    Info.Type = RT_MANIFEST as ULONG_PTR;
    Info.Name = ISOLATIONAWARE_MANIFEST_RESOURCE_ID as ULONG_PTR;
    Info.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL) as ULONG_PTR;

    Status = LdrFindResource_U(DllHandle, &mut Info as *mut LDR_RESOURCE_INFO, 3, &mut Entry as *mut *mut IMAGE_RESOURCE_DATA_ENTRY);
    if Status == 0 {
        Context.cbSize = core::mem::size_of::<ACTCTXW>() as ULONG;
        Context.lpSource = FullDllName;
        Context.dwFlags = (ACTCTX_FLAG_RESOURCE_NAME_VALID | ACTCTX_FLAG_HMODULE_VALID) as DWORD;
        Context.hModule = DllHandle as HMODULE;
        Context.lpResourceName = ISOLATIONAWARE_MANIFEST_RESOURCE_ID as LPCWSTR;

        Status = RtlCreateActivationContext(
            0,
            &mut Context as *mut ACTCTXW as PVOID,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Result as *mut HANDLE,
        );

        if NT_SUCCESS(Status) {
            *ActCtx = Result as PVOID;
        }

        if Status == STATUS_SXS_INVALID_ACTCTXDATA_FORMAT {
            static mut Once: i32 = 0;
            let fresh0 = Once;
            Once = Once + 1;
            if fresh0 != 0 {
                // DPRINT removed
            }
            Status = STATUS_RESOURCE_DATA_NOT_FOUND;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn AddRefActCtx(hActCtx: HANDLE) {
    RtlAddRefActivationContext(hActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseActCtx(hActCtx: HANDLE) {
    RtlReleaseActivationContext(hActCtx);
}

#[no_mangle]
pub unsafe extern "system" fn ZombifyActCtx(hActCtx: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    Status = RtlZombifyActivationContext(hActCtx);
    if NT_SUCCESS(Status) {
        return 1;
    }
    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ActivateActCtx(
    hActCtx: HANDLE,
    ulCookie: PULONG_PTR,
) -> BOOL {
    let mut Status: NTSTATUS;

    /* Check if the handle was invalid */
    if hActCtx == INVALID_HANDLE_VALUE {
        /* Set error and bail out */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0; /* FALSE */
    }

    /* Call the native API */
    Status = RtlActivateActivationContext(0, hActCtx, ulCookie);
    if !NT_SUCCESS(Status) {
        /* Set error and bail out */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0; /* FALSE */
    }

    /* It worked */
    return 1; /* TRUE */
}

#[no_mangle]
pub unsafe extern "system" fn DeactivateActCtx(
    dwFlags: DWORD,
    ulCookie: ULONG_PTR,
) -> BOOL {
    let mut NativeFlags: ULONG;

    /* Check if the flags are invalid */
    if (dwFlags & !DEACTIVATE_ACTCTX_FLAG_FORCE_EARLY_DEACTIVATION) != 0
    {
        /* Set error and bail out */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return FALSE;
    }

    /* Convert flags */
    NativeFlags = 0;
    if (dwFlags & DEACTIVATE_ACTCTX_FLAG_FORCE_EARLY_DEACTIVATION) != 0
    {
        NativeFlags = RTL_DEACTIVATE_ACTIVATION_CONTEXT_FLAG_FORCE_EARLY_DEACTIVATION;
    }

    /* Call the native API -- it can never fail */
    RtlDeactivateActivationContext(NativeFlags, ulCookie);
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetCurrentActCtx(phActCtx: PHANDLE) -> BOOL {
    let Status: NTSTATUS;

    /* Check if the output handle pointer was invalid */
    if phActCtx.is_null() {
        /* Set error and bail out */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    /* Call the native API */
    Status = RtlGetActiveActivationContext(phActCtx);
    if !NT_SUCCESS(Status) {
        /* Set error and bail out */
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return 0;
    }

    /* It worked */
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn QueryActCtxW(
    dwFlags: DWORD,
    hActCtx: HANDLE,
    pvSubInstance: PVOID,
    ulInfoClass: ULONG,
    pvBuffer: PVOID,
    cbBuffer: SIZE_T,
    pcbWrittenOrRequired: *mut SIZE_T,
) -> BOOL {
    let mut NativeFlags: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    if !pcbWrittenOrRequired.is_null() {
        *pcbWrittenOrRequired = 0;
    }

    match dwFlags & 3 {
        1 => {
            dwFlags |= QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX;
        }
        2 => {
            dwFlags |= QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE;
        }
        3 => {
            dwFlags |= QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS;
        }
        _ => {}
    }

    dwFlags &= !3;

    if dwFlags & !QUERY_ACTCTX_FLAG_VALID != 0 {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
        return 0;
    }

    match ulInfoClass {
        1 | 2 | 6 | 5 => {}
        3 | 4 => {
            if pvSubInstance.is_null() {
                BaseSetLastNTError(STATUS_INVALID_PARAMETER_3);
                return 0;
            }
        }
        _ => {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER_2);
            return 0;
        }
    }

    if pvBuffer.is_null() {
        if cbBuffer != 0 {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER_4);
            return 0;
        }
        if pcbWrittenOrRequired.is_null() {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER_5);
            return 0;
        }
    }

    match dwFlags & (QUERY_ACTCTX_FLAG_VALID & !QUERY_ACTCTX_FLAG_NO_ADDREF) {
        4 => {
            NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT;
        }
        8 => {
            NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE;
        }
        16 => {
            NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS;
        }
        0 => {}
        _ => {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
            return 0;
        }
    }

    if dwFlags & QUERY_ACTCTX_FLAG_NO_ADDREF != 0 {
        NativeFlags |= RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF;
    }

    Status = RtlQueryInformationActivationContext(
        NativeFlags,
        hActCtx,
        pvSubInstance,
        ulInfoClass,
        pvBuffer,
        cbBuffer,
        pcbWrittenOrRequired,
    );

    if NT_SUCCESS(Status) {
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/actctx.c
//  * PURPOSE:         Activation contexts - NT-compatible helpers
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *
//  * NOTE:            See also kernel32/wine/actctx.c
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <k32.h>
// #define NDEBUG
// #include <debug.h>
//
// #define QUERY_ACTCTX_FLAG_VALID         (QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX | \
//                                          QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE | \
//                                          QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS | \
//                                          QUERY_ACTCTX_FLAG_NO_ADDREF)
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// VOID
// NTAPI
// BasepFreeActivationContextActivationBlock(IN PBASEP_ACTCTX_BLOCK ActivationBlock)
// {
//     /* Exit if there was nothing passed in */
//     if (!ActivationBlock) return;
//
//     /* Do we have a context? */
//     if (ActivationBlock->ActivationContext)
//     {
//         /* Release and clear it */
//         RtlReleaseActivationContext(ActivationBlock->ActivationContext);
//         ActivationBlock->ActivationContext = NULL;
//     }
//
//     /* Free the block */
//     RtlFreeHeap(RtlGetProcessHeap(), 0, ActivationBlock);
// }
//
// NTSTATUS
// NTAPI
// BasepAllocateActivationContextActivationBlock(IN DWORD Flags,
//                                               IN PVOID CompletionRoutine,
//                                               IN PVOID CompletionContext,
//                                               OUT PBASEP_ACTCTX_BLOCK *ActivationBlock)
// {
//     NTSTATUS Status;
//     ACTIVATION_CONTEXT_BASIC_INFORMATION ContextInfo;
//
//     /* Clear the info structure */
//     ContextInfo.dwFlags = 0;
//     ContextInfo.hActCtx = NULL;
//
//     /* Assume failure */
//     if (ActivationBlock) *ActivationBlock = NULL;
//
//     /* Only support valid flags */
//     if (Flags & ~(1 | 2)) // FIXME: What are they? 2 looks like BASEP_ACTCTX_FORCE_BLOCK
//     {
//         /* Fail if unknown flags are passed in */
//         Status = STATUS_INVALID_PARAMETER_1;
//         goto Quickie;
//     }
//
//     /* Caller should have passed in an activation block */
//     if (!ActivationBlock)
//     {
//         /* Fail otherwise */
//         Status = STATUS_INVALID_PARAMETER_4;
//         goto Quickie;
//     }
//
//     /* Query RTL for information on the current activation context */
//     Status = RtlQueryInformationActivationContext(RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT,
//                                                   NULL,
//                                                   NULL,
//                                                   ActivationContextBasicInformation,
//                                                   &ContextInfo,
//                                                   sizeof(ContextInfo),
//                                                   NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Failed -- bail out */
//         DPRINT1("SXS: %s - Failure getting active activation context; ntstatus %08lx\n",
//                 __FUNCTION__, Status);
//         goto Quickie;
//     }
//
//     /* Check if the current one should be freed */
//     if (ContextInfo.dwFlags & 1)
//     {
//         /* Release and clear it */
//         RtlReleaseActivationContext(ContextInfo.hActCtx);
//         ContextInfo.hActCtx = NULL;
//     }
//
//     /* Check if there's an active context, or if the caller is forcing one */
//     if (!(Flags & 2) || (ContextInfo.hActCtx))
//     {
//         /* Allocate the block */
//         *ActivationBlock = RtlAllocateHeap(RtlGetProcessHeap(),
//                                            0,
//                                            sizeof(BASEP_ACTCTX_BLOCK));
//         if (!(*ActivationBlock))
//         {
//             /* Ran out of memory, fail */
//             Status = STATUS_NO_MEMORY;
//             goto Quickie;
//         }
//
//         /* Fill it out */
//         (*ActivationBlock)->ActivationContext = ContextInfo.hActCtx;
//         (*ActivationBlock)->Flags = 0;
//         if (Flags & 1) (*ActivationBlock)->Flags |= 1; // Not sure about this flag
//         (*ActivationBlock)->CompletionRoutine = CompletionRoutine;
//         (*ActivationBlock)->CompletionContext = CompletionContext;
//
//         /* Tell Quickie below not to free anything, since this is success */
//         ContextInfo.hActCtx = NULL;
//     }
//
//     /* Set success status */
//     Status = STATUS_SUCCESS;
//
// Quickie:
//     /* Failure or success path, return to caller and free on failure */
//     if (ContextInfo.hActCtx) RtlReleaseActivationContext(ContextInfo.hActCtx);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// BasepProbeForDllManifest(IN PVOID DllHandle,
//                          IN PCWSTR FullDllName,
//                          OUT PVOID *ActCtx)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     LDR_RESOURCE_INFO Info;
//     IMAGE_RESOURCE_DATA_ENTRY *Entry;
//     ACTCTXW Context;
//     HANDLE Result;
//
//     /* Check if activation context parameter is provided */
//     if (!ActCtx)
//     {
//         ASSERT(FALSE);
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Zero it out */
//     *ActCtx = NULL;
//
//     /* Check whether the image has manifest resource associated with it */
//     Info.Type = (ULONG_PTR)RT_MANIFEST;
//     Info.Name = (ULONG_PTR)ISOLATIONAWARE_MANIFEST_RESOURCE_ID;
//     Info.Language = MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL);
//     if (!(Status = LdrFindResource_U(DllHandle, &Info, 3, &Entry)))
//     {
//         /* Create the activation context */
//         Context.cbSize = sizeof(Context);
//         Context.lpSource = FullDllName;
//         Context.dwFlags = ACTCTX_FLAG_RESOURCE_NAME_VALID | ACTCTX_FLAG_HMODULE_VALID;
//         Context.hModule = DllHandle;
//         Context.lpResourceName = (LPCWSTR)ISOLATIONAWARE_MANIFEST_RESOURCE_ID;
//
//         Status = RtlCreateActivationContext(0, (PVOID)&Context, 0, NULL, NULL, &Result);
//
//         /* Store activation context pointer if it was created successfully */
//         if (NT_SUCCESS(Status)) *ActCtx = Result;
//
//         /* CORE-10843: Windows always returns this since we pass the wrong struct */
//         if (Status == STATUS_SXS_INVALID_ACTCTXDATA_FORMAT)
//         {
//             /* Fake "Manifest not found" so the load doesn't fail  */
//             static int Once;
//             if (Once++)
//             {
//                 DPRINT1("HACK: Passed invalid ACTIVATION_CONTEXT_DATA!\n");
//             }
//             Status = STATUS_RESOURCE_DATA_NOT_FOUND;
//         }
//     }
//
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// AddRefActCtx(IN HANDLE hActCtx)
// {
//     /* Call the native API */
//     RtlAddRefActivationContext(hActCtx);
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// ReleaseActCtx(IN HANDLE hActCtx)
// {
//     /* Call the native API */
//     RtlReleaseActivationContext(hActCtx);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ZombifyActCtx(HANDLE hActCtx)
// {
//     NTSTATUS Status;
//
//     /* Call the native API */
//     Status = RtlZombifyActivationContext(hActCtx);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* Set last error if we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ActivateActCtx(IN HANDLE hActCtx,
//                OUT PULONG_PTR ulCookie)
// {
//     NTSTATUS Status;
//
//     /* Check if the handle was invalid */
//     if (hActCtx == INVALID_HANDLE_VALUE)
//     {
//         /* Set error and bail out */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Call the native API */
//     Status = RtlActivateActivationContext(0, hActCtx, ulCookie);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set error and bail out */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* It worked */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DeactivateActCtx(IN DWORD dwFlags,
//                  IN ULONG_PTR ulCookie)
// {
//     ULONG NativeFlags;
//
//     /* Check if the flags are invalid */
//     if ((dwFlags & ~DEACTIVATE_ACTCTX_FLAG_FORCE_EARLY_DEACTIVATION) != 0)
//     {
//         /* Set error and bail out */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Convert flags */
//     NativeFlags = 0;
//     if (dwFlags & DEACTIVATE_ACTCTX_FLAG_FORCE_EARLY_DEACTIVATION)
//     {
//         NativeFlags = RTL_DEACTIVATE_ACTIVATION_CONTEXT_FLAG_FORCE_EARLY_DEACTIVATION;
//     }
//
//     /* Call the native API -- it can never fail */
//     RtlDeactivateActivationContext(NativeFlags, ulCookie);
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetCurrentActCtx(OUT PHANDLE phActCtx)
// {
//     NTSTATUS Status;
//
//     /* Check if the output handle pointer was invalid */
//     if (phActCtx == NULL)
//     {
//         /* Set error and bail out */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Call the native API */
//     Status = RtlGetActiveActivationContext(phActCtx);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set error and bail out */
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* It worked */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueryActCtxW(IN DWORD dwFlags,
//              IN HANDLE hActCtx,
//              IN PVOID pvSubInstance,
//              IN ULONG ulInfoClass,
//              IN PVOID pvBuffer,
//              IN SIZE_T cbBuffer,
//              IN OUT SIZE_T *pcbWrittenOrRequired OPTIONAL)
// {
//     ULONG NativeFlags = 0;
//     NTSTATUS Status;
//
//     /* Assume failure */
//     if (pcbWrittenOrRequired) *pcbWrittenOrRequired = 0;
//
//     /* Check if native flags were passed in to the Win32 function */
//     switch (dwFlags & 3)
//     {
//         case RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT:
//             dwFlags |= QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX;
//             break;
//         case RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE:
//             dwFlags |= QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE;
//             break;
//         case (RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS - 1): // Yep, not sure why
//             dwFlags |= QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS;
//             break;
//     }
//
//     /* Now mask out the native flags */
//     dwFlags &= ~3;
//
//     /* Check if any invalid flags are left */
//     if (dwFlags & ~QUERY_ACTCTX_FLAG_VALID)
//     {
//         /* Yep, bail out */
//         DPRINT1("SXS: %s() bad flags(passed: 0x%lx, allowed: 0x%lx, bad: 0x%lx)\n",
//                 __FUNCTION__,
//                 dwFlags,
//                 QUERY_ACTCTX_FLAG_VALID,
//                 dwFlags & ~QUERY_ACTCTX_FLAG_VALID);
//         BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
//         return FALSE;
//     }
//
//     /* See if additional parameters are required */
//     switch (ulInfoClass)
//     {
//         case ActivationContextBasicInformation:
//         case ActivationContextDetailedInformation:
//         case CompatibilityInformationInActivationContext:
//         case RunlevelInformationInActivationContext:
//
//             /* Nothing to check */
//             break;
//
//         case AssemblyDetailedInformationInActivationContext:
//         case FileInformationInAssemblyOfAssemblyInActivationContext:
//
//             /* We need a subinstance for these queries*/
//             if (!pvSubInstance)
//             {
//                 /* None present, bail out */
//                 DPRINT1("SXS: %s() InfoClass 0x%lx requires SubInstance != NULL\n",
//                         __FUNCTION__,
//                         ulInfoClass);
//                 BaseSetLastNTError(STATUS_INVALID_PARAMETER_3);
//                 return FALSE;
//             }
//             break;
//
//         default:
//
//             /* Invalid class, bail out */
//             DPRINT1("SXS: %s() bad InfoClass(0x%lx)\n",
//                     __FUNCTION__,
//                     ulInfoClass);
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER_2);
//             return FALSE;
//     }
//
//     /* Check if no buffer was passed in*/
//     if (!pvBuffer)
//     {
//         /* But a non-zero length was? */
//         if (cbBuffer)
//         {
//             /* This is bogus... */
//             DPRINT1("SXS: %s() (pvBuffer == NULL) && ((cbBuffer=0x%lu) != 0)\n",
//                     __FUNCTION__,
//                      cbBuffer);
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER_4);
//             return FALSE;
//         }
//
//         /* But the caller doesn't want to know how big to make it? */
//         if (!pcbWrittenOrRequired)
//         {
//             /* That's bogus */
//             DPRINT1("SXS: %s() (pvBuffer == NULL) && (pcbWrittenOrRequired == NULL)\n",
//                     __FUNCTION__);
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER_5);
//             return FALSE;
//         }
//     }
//
//     /* These 3 flags are mutually exclusive -- only one should be present */
//     switch (dwFlags & (QUERY_ACTCTX_FLAG_VALID & ~QUERY_ACTCTX_FLAG_NO_ADDREF))
//     {
//         /* Convert into native format */
//         case QUERY_ACTCTX_FLAG_USE_ACTIVE_ACTCTX:
//             NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_USE_ACTIVE_ACTIVATION_CONTEXT;
//             break;
//         case QUERY_ACTCTX_FLAG_ACTCTX_IS_HMODULE:
//             NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_HMODULE;
//             break;
//         case QUERY_ACTCTX_FLAG_ACTCTX_IS_ADDRESS:
//             NativeFlags = RTL_QUERY_ACTIVATION_CONTEXT_FLAG_IS_ADDRESS;
//             break;
//         case 0:
//             break;
//
//         /* More than one flag is set... */
//         default:
//             /* Bail out */
//             DPRINT1("SXS: %s(dwFlags=0x%lx) more than one flag in 0x%lx was passed\n",
//                     __FUNCTION__,
//                     dwFlags,
//                     0x1C);
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER_1);
//             return FALSE;
//     }
//
//     /* Convert this last flag */
//     if (dwFlags & QUERY_ACTCTX_FLAG_NO_ADDREF)
//     {
//         NativeFlags |= RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF;
//     }
//
//     /* Now call the native API */
//     DPRINT("SXS: %s() Calling Native API with Native Flags %lx for Win32 Flags %lx\n",
//            __FUNCTION__,
//            NativeFlags,
//            dwFlags);
//     Status = RtlQueryInformationActivationContext(NativeFlags,
//                                                   hActCtx,
//                                                   pvSubInstance,
//                                                   ulInfoClass,
//                                                   pvBuffer,
//                                                   cbBuffer,
//                                                   pcbWrittenOrRequired);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* Failed, set error and return */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
