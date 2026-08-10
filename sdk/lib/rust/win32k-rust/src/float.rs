use win32k_sys::win32k::*;
use ntkrnl_sys::NT_SUCCESS;

// Per-arch FP save/restore wrappers. On i386/amd64 these are real exports
// (ntoskrnl.spec:681-686, x86-only). On arm64, `sdk/include/xdk/arm64/ke.h:69`
// has them as FORCEINLINE stubs that just return STATUS_SUCCESS — bindgen's
// `ntkrnl_sys::ddk::Ke{Save,Restore}FloatingPointState` extern on arm64 is a
// phantom (no link symbol exists). Mirror the FORCEINLINE bodies inline here.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use ntkrnl_sys::ddk::{KeSaveFloatingPointState, KeRestoreFloatingPointState};

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeSaveFloatingPointState(_FloatSave: *mut ntkrnl_sys::ddk::KFLOATING_SAVE) -> ntkrnl_sys::base::NTSTATUS {
    // arm64: FORCEINLINE stub in xdk/arm64/ke.h returns STATUS_SUCCESS.
    0
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn KeRestoreFloatingPointState(_FloatSave: *mut ntkrnl_sys::ddk::KFLOATING_SAVE) -> ntkrnl_sys::base::NTSTATUS {
    // arm64: FORCEINLINE stub in xdk/arm64/ke.h returns STATUS_SUCCESS.
    0
}

// ---- file-scope preamble (translated) ----
//
// IMPORTANT (cross-crate type identity, win32k_translation_prompt.md §5):
// `KFLOATING_SAVE` is bindgen'd independently in BOTH `win32k_sys::win32k`
// AND `ntkrnl_sys::ddk` as DISTINCT Rust types (same C layout). The kernel
// fns `KeSaveFloatingPointState` / `KeRestoreFloatingPointState` live in
// `ntkrnl_sys::ddk` and expect `ntkrnl_sys::ddk::KFLOATING_SAVE`. So the
// `FloatState` field is fully-qualified `ntkrnl_sys::ddk::KFLOATING_SAVE`,
// NOT the `win32k_sys::win32k::*` glob version.
#[repr(C)]
pub struct _WIN32K_FLOATING_SAVE {
    pub FloatState: ntkrnl_sys::ddk::KFLOATING_SAVE,
    pub IsFloatingPointSaved: BOOLEAN,
}

pub type WIN32K_FLOATING_SAVE = _WIN32K_FLOATING_SAVE;
pub type PWIN32K_FLOATING_SAVE = *mut _WIN32K_FLOATING_SAVE;

#[no_mangle]
pub unsafe extern "system" fn EngRestoreFloatingPointState(
    pBuffer: PVOID
) -> BOOL {
    let State: PWIN32K_FLOATING_SAVE = pBuffer as PWIN32K_FLOATING_SAVE;

    if (*State).IsFloatingPointSaved == 0 {
        return 0;
    }

    (*State).IsFloatingPointSaved = 0;
    let Status = KeRestoreFloatingPointState(core::ptr::addr_of_mut!((*State).FloatState));
    if !NT_SUCCESS(Status) {
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn EngSaveFloatingPointState(
    pBuffer: PVOID,
    cjBufferSize: ULONG,
) -> ULONG {
    if pBuffer.is_null() || cjBufferSize == 0 {
        let mut TempBuffer: ntkrnl_sys::ddk::KFLOATING_SAVE = core::mem::zeroed();

        // Check for floating point support.
        let Status = KeSaveFloatingPointState(core::ptr::addr_of_mut!(TempBuffer));
        if Status != ntkrnl_sys::STATUS_SUCCESS {
            return 0;
        }
        KeRestoreFloatingPointState(core::ptr::addr_of_mut!(TempBuffer));
        return core::mem::size_of::<WIN32K_FLOATING_SAVE>() as ULONG;
    }

    if (cjBufferSize as usize) < core::mem::size_of::<WIN32K_FLOATING_SAVE>() {
        return 0;
    }

    // Per MSDN, "This buffer must be zero-initialized, and must be in nonpaged memory."
    let State: PWIN32K_FLOATING_SAVE = pBuffer as PWIN32K_FLOATING_SAVE;

    // (Driver may have already saved — C just DPRINT1s and continues.)

    let Status = KeSaveFloatingPointState(core::ptr::addr_of_mut!((*State).FloatState));
    if !NT_SUCCESS(Status) {
        return 0;
    }

    (*State).IsFloatingPointSaved = 1;
    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:         See COPYING in the top level directory
//  * PROJECT:           ReactOS kernel
//  * PURPOSE:           Engine floating point functions
//  * FILE:              win32ss/gdi/eng/float.c
//  * PROGRAMER:         David Welch
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _WIN32K_FLOATING_SAVE
// {
//     KFLOATING_SAVE FloatState;
//     BOOLEAN IsFloatingPointSaved;
// } WIN32K_FLOATING_SAVE, *PWIN32K_FLOATING_SAVE;
//
// /* FUNCTIONS *****************************************************************/
//
// #ifdef _PREFAST_
// #pragma warning(disable:__WARNING_WRONG_KIND)
// #endif
//
// _Check_return_
// _Success_(return)
// _Kernel_float_restored_
// _At_(*pBuffer, _Kernel_requires_resource_held_(EngFloatState)
//                _Kernel_releases_resource_(EngFloatState))
// ENGAPI
// BOOL
// APIENTRY
// EngRestoreFloatingPointState(
//     _In_reads_(_Inexpressible_(statesize)) PVOID pBuffer)
// {
//     NTSTATUS Status;
//     PWIN32K_FLOATING_SAVE State = (PWIN32K_FLOATING_SAVE)pBuffer;
//
//     if (!State->IsFloatingPointSaved)
//     {
//         DPRINT1("The driver has attempted to restore floating point state after already restoring it.\n");
//         DPRINT1("This (probably ICafe AMD) driver has done an incorrect behavior.\n");
//         return FALSE;
//     }
//
//     State->IsFloatingPointSaved = FALSE;
//
//     Status = KeRestoreFloatingPointState(&State->FloatState);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// _Check_return_
// _Success_(((pBuffer != NULL && cjBufferSize != 0) && return == 1) ||
//           ((pBuffer == NULL || cjBufferSize == 0) && return > 0))
// _When_(pBuffer != NULL && cjBufferSize != 0 && return == 1, _Kernel_float_saved_
//     _At_(*pBuffer, _Post_valid_ _Kernel_acquires_resource_(EngFloatState)))
// _On_failure_(_Post_satisfies_(return == 0))
// ENGAPI
// ULONG
// APIENTRY
// EngSaveFloatingPointState(
//     _At_(*pBuffer, _Kernel_requires_resource_not_held_(EngFloatState))
//     _Out_writes_bytes_opt_(cjBufferSize) PVOID pBuffer,
//     _Inout_ ULONG cjBufferSize)
// {
//     PWIN32K_FLOATING_SAVE State;
//     NTSTATUS Status;
//
//     if ((pBuffer == NULL) || (cjBufferSize == 0))
//     {
//         KFLOATING_SAVE TempBuffer;
//
//         /* Check for floating point support. */
//         Status = KeSaveFloatingPointState(&TempBuffer);
//         if (Status != STATUS_SUCCESS)
//         {
//             return(0);
//         }
//         KeRestoreFloatingPointState(&TempBuffer);
//         return sizeof(WIN32K_FLOATING_SAVE);
//     }
//
//     if (cjBufferSize < sizeof(WIN32K_FLOATING_SAVE))
//     {
//         return(0);
//     }
//
//     /* Per MSDN, "This buffer must be zero-initialized, and must be in nonpaged memory." */
//     State = (PWIN32K_FLOATING_SAVE)pBuffer;
//
//     if (State->IsFloatingPointSaved)
//     {
//         DPRINT1("The driver has attempted to save floating point state after already saving it.\n");
//         DPRINT1("This (probably ICafe AMD) driver has done an incorrect behavior.\n");
//     }
//
//     Status = KeSaveFloatingPointState(&State->FloatState);
//     if (!NT_SUCCESS(Status))
//     {
//         return FALSE;
//     }
//
//     State->IsFloatingPointSaved = TRUE;
//     return TRUE;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
