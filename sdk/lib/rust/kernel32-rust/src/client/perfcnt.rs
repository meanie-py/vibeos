use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----

#[no_mangle]
pub unsafe extern "system" fn QueryPerformanceCounter(
    lpPerformanceCount: PLARGE_INTEGER,
) -> BOOL {
    let mut Frequency: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS = 0;

    Status = NtQueryPerformanceCounter(lpPerformanceCount, &mut Frequency);
    if Frequency.QuadPart == 0 {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn QueryPerformanceFrequency(lpFrequency: PLARGE_INTEGER) -> BOOL {
    let mut Count: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;

    Status = NtQueryPerformanceCounter(&mut Count, lpFrequency);
    if (*lpFrequency).QuadPart == 0 {
        Status = STATUS_NOT_IMPLEMENTED;
    }

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:            See COPYING in the top level directory
//  * PROJECT:              ReactOS Win32 Base API
//  * FILE:                 dll/win32/kernel32/client/perfcnt.c
//  * PURPOSE:              Performance Counter
//  * PROGRAMMER:           Eric Kohl
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueryPerformanceCounter(OUT PLARGE_INTEGER lpPerformanceCount)
// {
//     LARGE_INTEGER Frequency;
//     NTSTATUS Status;
//
//     Status = NtQueryPerformanceCounter(lpPerformanceCount, &Frequency);
//     if (Frequency.QuadPart == 0) Status = STATUS_NOT_IMPLEMENTED;
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// QueryPerformanceFrequency(OUT PLARGE_INTEGER lpFrequency)
// {
//     LARGE_INTEGER Count;
//     NTSTATUS Status;
//
//     Status = NtQueryPerformanceCounter(&Count, lpFrequency);
//     if (lpFrequency->QuadPart == 0) Status = STATUS_NOT_IMPLEMENTED;
//
//     if (!NT_SUCCESS(Status))
//     {
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
