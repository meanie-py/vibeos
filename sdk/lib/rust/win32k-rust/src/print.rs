use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::base::{STATUS_SUCCESS, NT_SUCCESS};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

// ExAllocatePoolZero is a FORCEINLINE (xdk/exfuncs.h) — alloc then zero, no symbol.
#[inline(always)]
unsafe fn ExAllocatePoolZero_rs(pool: POOL_TYPE, n: usize, tag: u32) -> PVOID {
    let p = ExAllocatePoolWithTag(pool, n as _, tag);
    if !p.is_null() {
        RtlZeroMemory(p, n as _);
    }
    p
}

// EngpGetPDEV + SafeProbeFor* are NTAPI — extern "system" (stdcall i386 / cdecl else).
extern "system" {
    fn EngpGetPDEV(pustrDevice: *mut core::ffi::c_void) -> PPDEVOBJ;
    fn SafeProbeForRead(Address: *mut core::ffi::c_void, Length: usize, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut core::ffi::c_void, Length: usize, Alignment: u32) -> NTSTATUS;
}
// SetLastNtError is FASTCALL (i386 @SetLastNtError@4); cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// SURFACE_ShareLockByPointer = FORCEINLINE GDIOBJ_vReferenceObjectByPointer(&psurf->BaseObject).
#[inline(always)]
unsafe fn SURFACE_ShareLockByPointer(psurf: PSURFACE) {
    GDIOBJ_vReferenceObjectByPointer(core::ptr::addr_of_mut!((*psurf).BaseObject) as POBJ);
}
// SURFACE_ShareUnlockSurface(p) = GDIOBJ_vDereferenceObject((POBJ)p) (macro).
#[inline(always)]
unsafe fn SURFACE_ShareUnlockSurface(psurf: PSURFACE) {
    GDIOBJ_vDereferenceObject(psurf as POBJ);
}
// drop_gdiobj_macro_externs: dropped 1 bogus extern decl(s) for GDI-object macro(s): SURFACE_ShareUnlockSurface — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();
#[no_mangle]
pub unsafe extern "system" fn NtGdiAbortDoc(hDC: HDC) -> INT {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEndDoc(hDC: HDC) -> INT {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiEndPage(hDC: HDC) -> INT {
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiEscape(
    dc: PDC,
    Escape: INT,
    InSize: INT,
    InData: LPCSTR,
    OutData: LPVOID,
) -> INT {
    if Escape == QUERYESCSUPPORT as INT {
        return 0;
    }

    SP_ERROR
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiEscape(
    dc: PDC,
    Escape: INT,
    InSize: INT,
    InData: LPCSTR,
    OutData: LPVOID,
) -> INT {
    if Escape == QUERYESCSUPPORT as INT {
        return 0;
    }

    SP_ERROR
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiEscape(
    hDC: HDC,
    Escape: INT,
    InSize: INT,
    InData: LPCSTR,
    OutData: LPVOID,
) -> INT {
    let dc: PDC = DC_LockDc(hDC);
    if dc == core::ptr::null_mut() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return SP_ERROR;
    }

    let ret = IntGdiEscape(dc, Escape, InSize, InData, OutData);

    DC_UnlockDc(dc);
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiExtEscape(
    hDC: HDC,
    pDriver: *mut u16,
    _nDriver: INT,
    Escape: INT,
    InSize: INT,
    UnsafeInData: LPSTR,
    OutSize: INT,
    UnsafeOutData: LPSTR,
) -> INT {
    let mut SafeInData: LPVOID = core::ptr::null_mut();
    let mut SafeOutData: LPVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Result: INT;
    let ppdev: PPDEVOBJ;
    let psurf: PSURFACE;

    const GDITAG_TEMP: u32 = u32::from_le_bytes([b'p', b'm', b't', b'G']);

    /* Validate input parameters */
    if (InSize < 0) || (OutSize < 0) || (UnsafeInData.is_null() && InSize != 0) {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return -1;
    }

    if hDC.is_null() {
        if !pDriver.is_null() {
            /* FIXME : Get the pdev from its name */
            return -1;
        }

        ppdev = EngpGetPDEV(core::ptr::null_mut());
        if ppdev.is_null() {
            EngSetLastError(ERROR_BAD_DEVICE);
            return -1;
        }

        /* We're using the primary surface of the pdev. Lock it */
        EngAcquireSemaphore((*ppdev).hsemDevLock);

        psurf = (*ppdev).pSurface;
        if psurf.is_null() {
            EngReleaseSemaphore((*ppdev).hsemDevLock);
            PDEVOBJ_vRelease(ppdev);
            return 0;
        }
        SURFACE_ShareLockByPointer(psurf);
    } else {
        let pDC: PDC = DC_LockDc(hDC);
        if pDC.is_null() {
            EngSetLastError(ERROR_INVALID_HANDLE);
            return -1;
        }

        /* Get the PDEV from the DC */
        ppdev = (*pDC).ppdev;
        PDEVOBJ_vReference(ppdev);

        EngAcquireSemaphore((*ppdev).hsemDevLock);

        /* Check if we have a surface */
        psurf = (*pDC).dclevel.pSurface;
        if psurf.is_null() {
            EngReleaseSemaphore((*ppdev).hsemDevLock);
            PDEVOBJ_vRelease(ppdev);
            DC_UnlockDc(pDC);
            return 0;
        }
        SURFACE_ShareLockByPointer(psurf);

        /* We're done with the DC */
        DC_UnlockDc(pDC);
    }

    'exit: {
        /* See if we actually have a driver function to call */
        if (*ppdev).__bindgen_anon_1.DriverFunctions.Escape.is_none() {
            Result = 0;
            break 'exit;
        }

        if InSize != 0 {
            SafeInData = ExAllocatePoolWithTag(PagedPool, InSize as _, GDITAG_TEMP);
            if SafeInData.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                Result = 0;
                break 'exit;
            }
            Status = SafeProbeForRead(UnsafeInData as *mut core::ffi::c_void, InSize as usize, 1);
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                Result = 0;
                break 'exit;
            } else {
                if !crate::copy::capture_from_user(
                    SafeInData as *mut core::ffi::c_void,
                    UnsafeInData as *const core::ffi::c_void,
                    InSize as usize,
                ) {
                    SetLastNtError(ntkrnl_sys::probe::STATUS_ACCESS_VIOLATION as _);
                    Result = 0;
                    break 'exit;
                }
            }
        }

        if OutSize != 0 {
            if !UnsafeOutData.is_null() {
                Status = SafeProbeForWrite(UnsafeOutData as *mut core::ffi::c_void, OutSize as usize, 1);
                if !NT_SUCCESS(Status) {
                    SetLastNtError(Status);
                    Result = 0;
                    break 'exit;
                }
            }

            SafeOutData = ExAllocatePoolZero_rs(PagedPool, OutSize as _, GDITAG_TEMP);
            if SafeOutData.is_null() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                Result = 0;
                break 'exit;
            }
        }

        /* Finally call the driver */
        Result = ((*ppdev).__bindgen_anon_1.DriverFunctions.Escape.unwrap())(
            core::ptr::addr_of_mut!((*psurf).SurfObj),
            Escape as _,
            InSize as _,
            SafeInData,
            OutSize as _,
            SafeOutData,
        ) as INT;

        if OutSize != 0 && !UnsafeOutData.is_null() && !SafeOutData.is_null() {
            RtlCopyMemory(UnsafeOutData as *mut core::ffi::c_void, SafeOutData, OutSize as usize);
        }
    }

    SURFACE_ShareUnlockSurface(psurf);
    EngReleaseSemaphore((*ppdev).hsemDevLock);
    PDEVOBJ_vRelease(ppdev);

    if !SafeInData.is_null() {
        ExFreePoolWithTag(SafeInData, GDITAG_TEMP);
    }

    if !SafeOutData.is_null() {
        ExFreePoolWithTag(SafeOutData, GDITAG_TEMP);
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiStartDoc(
    hdc: HDC,
    pdi: *mut DOCINFOW,
    pbBanding: *mut BOOL,
    iJob: INT,
) -> INT {
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiStartPage(hDC: HDC) -> INT {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/print.c
//  * PURPOSE:         Print functions
//  * PROGRAMMER:
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// INT
// __stdcall
// NtGdiAbortDoc(HDC  hDC)
// {
//   UNIMPLEMENTED;
//   return 0;
// }
//
// INT
// __stdcall
// NtGdiEndDoc(HDC  hDC)
// {
//   UNIMPLEMENTED;
//   return 0;
// }
//
// INT
// __stdcall
// NtGdiEndPage(HDC  hDC)
// {
//   UNIMPLEMENTED;
//   return 0;
// }
//
// INT
// FASTCALL
// IntGdiEscape(PDC    dc,
//              INT    Escape,
//              INT    InSize,
//              LPCSTR InData,
//              LPVOID OutData)
// {
//   if (Escape == QUERYESCSUPPORT)
//     return FALSE;
//
//   UNIMPLEMENTED;
//   return SP_ERROR;
// }
//
// INT
// __stdcall
// NtGdiEscape(HDC  hDC,
//             INT  Escape,
//             INT  InSize,
//             LPCSTR  InData,
//             LPVOID  OutData)
// {
//   PDC dc;
//   INT ret;
//
//   dc = DC_LockDc(hDC);
//   if (dc == NULL)
//   {
//     EngSetLastError(ERROR_INVALID_HANDLE);
//     return SP_ERROR;
//   }
//
//   /* TODO: FIXME: Don't pass umode buffer to an Int function */
//   ret = IntGdiEscape(dc, Escape, InSize, InData, OutData);
//
//   DC_UnlockDc( dc );
//   return ret;
// }
//
// INT
// __stdcall
// NtGdiExtEscape(
//    HDC    hDC,
//    IN OPTIONAL PWCHAR pDriver,
//    IN INT nDriver,
//    INT    Escape,
//    INT    InSize,
//    OPTIONAL LPSTR UnsafeInData,
//    INT    OutSize,
//    OPTIONAL LPSTR  UnsafeOutData)
// {
//    LPVOID   SafeInData = NULL;
//    LPVOID   SafeOutData = NULL;
//    NTSTATUS Status = STATUS_SUCCESS;
//    INT      Result;
//    PPDEVOBJ ppdev;
//    PSURFACE psurf;
//
//    /* Validate input parameters */
//    if ((InSize < 0) || (OutSize < 0) || (UnsafeInData == NULL && InSize != 0))
//    {
//       EngSetLastError(ERROR_INVALID_PARAMETER);
//       return -1;
//    }
//
//    if (hDC == NULL)
//    {
//       if (pDriver)
//       {
//          /* FIXME : Get the pdev from its name */
//          UNIMPLEMENTED;
//          return -1;
//       }
//
//       ppdev = EngpGetPDEV(NULL);
//       if (!ppdev)
//       {
//          EngSetLastError(ERROR_BAD_DEVICE);
//          return -1;
//       }
//
//       /* We're using the primary surface of the pdev. Lock it */
//       EngAcquireSemaphore(ppdev->hsemDevLock);
//
//       psurf = ppdev->pSurface;
//       if (!psurf)
//       {
//          EngReleaseSemaphore(ppdev->hsemDevLock);
//          PDEVOBJ_vRelease(ppdev);
//          return 0;
//       }
//       SURFACE_ShareLockByPointer(psurf);
//    }
//    else
//    {
//       PDC pDC = DC_LockDc(hDC);
//       if (pDC == NULL)
//       {
//          EngSetLastError(ERROR_INVALID_HANDLE);
//          return -1;
//       }
//
//       /* Get the PDEV from the DC */
//       ppdev = pDC->ppdev;
//       PDEVOBJ_vReference(ppdev);
//
//       EngAcquireSemaphore(ppdev->hsemDevLock);
//
//       /* Check if we have a surface */
//       psurf = pDC->dclevel.pSurface;
//       if (!psurf)
//       {
//          EngReleaseSemaphore(ppdev->hsemDevLock);
//          PDEVOBJ_vRelease(ppdev);
//          DC_UnlockDc(pDC);
//          return 0;
//       }
//       SURFACE_ShareLockByPointer(psurf);
//
//       /* We're done with the DC */
//       DC_UnlockDc(pDC);
//    }
//
//    /* See if we actually have a driver function to call */
//    if (ppdev->DriverFunctions.Escape == NULL)
//    {
//       Result = 0;
//       goto Exit;
//    }
//
//    if (InSize)
//    {
//       SafeInData = ExAllocatePoolWithTag(PagedPool, InSize, GDITAG_TEMP);
//       if (SafeInData == NULL)
//       {
//          EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//          goto Exit;
//       }
//       Status = SafeProbeForRead(UnsafeInData, InSize, 1);
//       if (!NT_SUCCESS(Status))
//       {
//           SetLastNtError(Status);
//           goto Exit;
//       }
//       else
//       {
//           RtlCopyMemory(SafeInData, UnsafeInData, InSize);
//       }
//    }
//
//    if (OutSize)
//    {
//       if (UnsafeOutData != NULL)
//       {
//          Status = SafeProbeForWrite(UnsafeOutData, OutSize, 1);
//          if (!NT_SUCCESS(Status))
//          {
//              SetLastNtError(Status);
//              goto Exit;
//          }
//       }
//
//       SafeOutData = ExAllocatePoolZero(PagedPool, OutSize, GDITAG_TEMP);
//       if (SafeOutData == NULL)
//       {
//          EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//          goto Exit;
//       }
//    }
//
//    /* Finally call the driver */
//    Result = ppdev->DriverFunctions.Escape(
//          &psurf->SurfObj,
//          Escape,
//          InSize,
//          SafeInData,
//          OutSize,
//          SafeOutData);
//
//    if (OutSize != 0 && UnsafeOutData != NULL && SafeOutData != NULL)
//    {
//       _SEH2_TRY
//       {
//          RtlCopyMemory(UnsafeOutData, SafeOutData, OutSize);
//       }
//       _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//       {
//          Status = _SEH2_GetExceptionCode();
//          SetLastNtError(Status);
//          Result = -1;
//       }
//       _SEH2_END;
//    }
//
// Exit:
//    SURFACE_ShareUnlockSurface(psurf);
//    EngReleaseSemaphore(ppdev->hsemDevLock);
//    PDEVOBJ_vRelease(ppdev);
//
//    if (SafeInData != NULL)
//    {
//       ExFreePoolWithTag(SafeInData, GDITAG_TEMP);
//    }
//
//    if (SafeOutData != NULL)
//    {
//       ExFreePoolWithTag(SafeOutData, GDITAG_TEMP);
//    }
//
//    return Result;
// }
//
// INT
// __stdcall
// NtGdiStartDoc(
//     IN HDC hdc,
//     IN DOCINFOW *pdi,
//     OUT BOOL *pbBanding,
//     IN INT iJob)
// {
//   UNIMPLEMENTED;
//   return 0;
// }
//
// INT
// __stdcall
// NtGdiStartPage(HDC  hDC)
// {
//   UNIMPLEMENTED;
//   return 0;
// }
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
