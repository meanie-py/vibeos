use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::rtl::RtlCopyMemory;

// SafeProbeForWrite: NTAPI (extern "system" = stdcall i386 / cdecl else), Length = SIZE_T.
extern "system" {
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// (no translatable file-scope declarations)

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetipfdDevMax(pdc: PDC) -> INT {
    IntGetipfdDevMaxImpl(pdc)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetipfdDevMax(pdc: PDC) -> INT {
    IntGetipfdDevMaxImpl(pdc)
}

unsafe fn IntGetipfdDevMaxImpl(pdc: PDC) -> INT {
    let mut Ret: INT = 0;
    let ppdev: PPDEVOBJ = (*pdc).ppdev;

    if (*ppdev).flFlags & PDEV_META_DEVICE != 0 {
        return 0;
    }

    if let Some(DescribePixelFormat) = (*ppdev).__bindgen_anon_1.DriverFunctions.DescribePixelFormat {
        Ret = DescribePixelFormat(
            (*ppdev).dhpdev,
            1,
            0,
            core::ptr::null_mut(),
        );
    }

    if Ret != 0 {
        (*pdc).ipfdDevMax = Ret;
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDescribePixelFormat(
    hdc: HDC,
    ipfd: INT,
    mut cjpfd: UINT,
    ppfd: PPIXELFORMATDESCRIPTOR,
) -> INT {
    let pdc: PDC;
    let ppdev: PPDEVOBJ;
    let mut Ret: INT = 0;
    let mut pfdSafe: PIXELFORMATDESCRIPTOR = core::mem::zeroed();

    if ppfd.is_null() && cjpfd != 0 {
        return 0;
    }

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    'exit: {
        if (*pdc).ipfdDevMax == 0 {
            if IntGetipfdDevMax(pdc) == 0 {
                break 'exit;
            }
        }

        if ppfd.is_null() {
            Ret = (*pdc).ipfdDevMax;
            break 'exit;
        }

        if ipfd < 1 || ipfd > (*pdc).ipfdDevMax {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'exit;
        }

        ppdev = (*pdc).ppdev;

        if (*ppdev).flFlags & PDEV_META_DEVICE != 0 {
            break 'exit;
        }

        if let Some(DescribePixelFormat) = (*ppdev).__bindgen_anon_1.DriverFunctions.DescribePixelFormat {
            Ret = DescribePixelFormat(
                (*ppdev).dhpdev,
                ipfd,
                core::mem::size_of::<PIXELFORMATDESCRIPTOR>() as ULONG,
                &mut pfdSafe,
            );
        }

        if Ret != 0 && cjpfd != 0 {
            cjpfd = core::cmp::min(cjpfd as usize, core::mem::size_of::<PIXELFORMATDESCRIPTOR>()) as UINT;
            // C guarded the copy with ProbeForWrite inside _SEH2_TRY; restore the
            // user-pointer validation via the non-raising SafeProbeForWrite (rustconv
            // dropped it). On failure, skip the copy rather than fault on a bad ppfd.
            // The probe VALIDATES only, so the copy still had to become the protected
            // one. A fault is skipped here for the same reason the probe failure is:
            // the caller gets Ret without the descriptor rather than a BUGCHECK.
            if NT_SUCCESS(SafeProbeForWrite(ppfd as *mut core::ffi::c_void, cjpfd as _, 1))
                && !crate::copy::publish_to_user(
                    ppfd as *mut core::ffi::c_void,
                    &mut pfdSafe as *mut _ as *const core::ffi::c_void,
                    cjpfd as usize,
                )
            {
                Ret = 0;
            }
        }
    }

    DC_UnlockDc(pdc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetPixelFormat(hdc: HDC, ipfd: INT) -> BOOL {
    let pdc: PDC;
    let ppdev: PPDEVOBJ;
    let hWnd: HWND;
    let mut pWndObj: PWNDOBJ = core::ptr::null_mut();
    let mut pso: *mut SURFOBJ = core::ptr::null_mut();
    let mut Ret: BOOL = 0;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    'Exit: {
        if (*pdc).ipfdDevMax == 0 {
            IntGetipfdDevMax(pdc);
        }

        if ipfd < 1 || ipfd > (*pdc).ipfdDevMax {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'Exit;
        }

        UserEnterExclusive();
        hWnd = UserGethWnd(hdc, core::ptr::addr_of_mut!(pWndObj));
        UserLeave();

        if hWnd.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_STYLE);
            break 'Exit;
        }

        ppdev = (*pdc).ppdev;

        if !pWndObj.is_null() {
            pso = (*pWndObj).psoOwner;
        } else {
            EngSetLastError(ERROR_INVALID_PIXEL_FORMAT);
            break 'Exit;
        }

        if (*ppdev).flFlags & PDEV_META_DEVICE != 0 {
            break 'Exit;
        }

        if (*ppdev).__bindgen_anon_1.DriverFunctions.SetPixelFormat.is_some() {
            Ret = (*ppdev).__bindgen_anon_1.DriverFunctions.SetPixelFormat.unwrap()(pso, ipfd, hWnd);
        }
    }

    DC_UnlockDc(pdc);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSwapBuffers(hdc: HDC) -> BOOL {
    let pdc: PDC;
    let ppdev: PPDEVOBJ;
    let hWnd: HWND;
    let mut pWndObj: PWNDOBJ = core::ptr::null_mut();
    let mut pso: *mut SURFOBJ = core::ptr::null_mut();
    let mut Ret: BOOL = 0;

    pdc = DC_LockDc(hdc);
    if pdc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    UserEnterExclusive();
    hWnd = UserGethWnd(hdc, core::ptr::addr_of_mut!(pWndObj));
    UserLeave();

    'Exit: {
        if hWnd.is_null() {
            EngSetLastError(ERROR_INVALID_WINDOW_STYLE);
            break 'Exit;
        }

        ppdev = (*pdc).ppdev;

        if !pWndObj.is_null() {
            pso = (*pWndObj).psoOwner;
        } else {
            EngSetLastError(ERROR_INVALID_PIXEL_FORMAT);
            break 'Exit;
        }

        if ((*ppdev).flFlags & PDEV_META_DEVICE) != 0 {
            break 'Exit;
        }

        if let Some(SwapBuffers) = (*ppdev).__bindgen_anon_1.DriverFunctions.SwapBuffers {
            Ret = SwapBuffers(pso, pWndObj);
        }
    }

    DC_UnlockDc(pdc);
    Ret
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS win32 kernel mode subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/wingl.c
//  * PURPOSE:         WinGL API
//  * PROGRAMMER:
//  */
//
// #include <win32k.h>
//
// #define NDEBUG
// #include <debug.h>
//
// static
// INT
// FASTCALL
// IntGetipfdDevMax(PDC pdc)
// {
//     INT Ret = 0;
//     PPDEVOBJ ppdev = pdc->ppdev;
//
//     if (ppdev->flFlags & PDEV_META_DEVICE)
//     {
//         return 0;
//     }
//
//     if (ppdev->DriverFunctions.DescribePixelFormat)
//     {
//         Ret = ppdev->DriverFunctions.DescribePixelFormat(
//                                                 ppdev->dhpdev,
//                                                 1,
//                                                 0,
//                                                 NULL);
//     }
//
//     if (Ret) pdc->ipfdDevMax = Ret;
//
//     return Ret;
// }
//
//
// __kernel_entry
// INT
// __stdcall
// NtGdiDescribePixelFormat(
//      HDC hdc,
//      INT ipfd,
//      UINT cjpfd,
//      PPIXELFORMATDESCRIPTOR ppfd)
// {
//     PDC pdc;
//     PPDEVOBJ ppdev;
//     INT Ret = 0;
//     PIXELFORMATDESCRIPTOR pfdSafe;
//
//     if ((ppfd == NULL) && (cjpfd != 0)) return 0;
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return 0;
//     }
//
//     if (!pdc->ipfdDevMax)
//     {
//         if (!IntGetipfdDevMax(pdc))
//         {
//             /* EngSetLastError ? */
//             goto Exit;
//         }
//     }
//
//     if (!ppfd)
//     {
//         Ret = pdc->ipfdDevMax;
//         goto Exit;
//     }
//
//     if ((ipfd < 1) || (ipfd > pdc->ipfdDevMax))
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         goto Exit;
//     }
//
//     ppdev = pdc->ppdev;
//
//     if (ppdev->flFlags & PDEV_META_DEVICE)
//     {
//         UNIMPLEMENTED;
//         goto Exit;
//     }
//
//     if (ppdev->DriverFunctions.DescribePixelFormat)
//     {
//         Ret = ppdev->DriverFunctions.DescribePixelFormat(
//                                                     ppdev->dhpdev,
//                                                     ipfd,
//                                                     sizeof(pfdSafe),
//                                                     &pfdSafe);
//     }
//
//     if (Ret && cjpfd)
//     {
//         _SEH2_TRY
//         {
//             cjpfd = min(cjpfd, sizeof(PIXELFORMATDESCRIPTOR));
//             ProbeForWrite(ppfd, cjpfd, 1);
//             RtlCopyMemory(ppfd, &pfdSafe, cjpfd);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             SetLastNtError(_SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
// Exit:
//     DC_UnlockDc(pdc);
//     return Ret;
// }
//
//
// BOOL
// __stdcall
// NtGdiSetPixelFormat(
//      HDC hdc,
//      INT ipfd)
// {
//     PDC pdc;
//     PPDEVOBJ ppdev;
//     HWND hWnd;
//     PWNDOBJ pWndObj;
//     SURFOBJ *pso = NULL;
//     BOOL Ret = FALSE;
//
//     DPRINT1("Setting pixel format from win32k!\n");
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (!pdc->ipfdDevMax)
//         IntGetipfdDevMax(pdc);
//
//     if ( ipfd < 1 ||
//         ipfd > pdc->ipfdDevMax )
//     {
//         EngSetLastError(ERROR_INVALID_PARAMETER);
//         goto Exit;
//     }
//
//     UserEnterExclusive();
//     hWnd = UserGethWnd(hdc, &pWndObj);
//     UserLeave();
//
//     if (!hWnd)
//     {
//         EngSetLastError(ERROR_INVALID_WINDOW_STYLE);
//         goto Exit;
//     }
//
//     ppdev = pdc->ppdev;
//
//     /*
//         WndObj is needed so exit on NULL pointer.
//     */
//     if (pWndObj)
//         pso = pWndObj->psoOwner;
//     else
//     {
//         EngSetLastError(ERROR_INVALID_PIXEL_FORMAT);
//         goto Exit;
//     }
//
//     if (ppdev->flFlags & PDEV_META_DEVICE)
//     {
//         UNIMPLEMENTED;
//         goto Exit;
//     }
//
//     if (ppdev->DriverFunctions.SetPixelFormat)
//     {
//         Ret = ppdev->DriverFunctions.SetPixelFormat(
//                                                 pso,
//                                                 ipfd,
//                                                 hWnd);
//     }
//
// Exit:
//     DC_UnlockDc(pdc);
//     return Ret;
// }
//
// BOOL
// __stdcall
// NtGdiSwapBuffers(
//      HDC hdc)
// {
//     PDC pdc;
//     PPDEVOBJ ppdev;
//     HWND hWnd;
//     PWNDOBJ pWndObj;
//     SURFOBJ *pso = NULL;
//     BOOL Ret = FALSE;
//
//     pdc = DC_LockDc(hdc);
//     if (!pdc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     UserEnterExclusive();
//     hWnd = UserGethWnd(hdc, &pWndObj);
//     UserLeave();
//
//     if (!hWnd)
//     {
//         EngSetLastError(ERROR_INVALID_WINDOW_STYLE);
//         goto Exit;
//     }
//
//     ppdev = pdc->ppdev;
//
//     /*
//         WndObj is needed so exit on NULL pointer.
//     */
//     if (pWndObj)
//         pso = pWndObj->psoOwner;
//     else
//     {
//         EngSetLastError(ERROR_INVALID_PIXEL_FORMAT);
//         goto Exit;
//     }
//
//     if (ppdev->flFlags & PDEV_META_DEVICE)
//     {
//         UNIMPLEMENTED;
//         goto Exit;
//     }
//
//     if (ppdev->DriverFunctions.SwapBuffers)
//     {
//         Ret = ppdev->DriverFunctions.SwapBuffers(pso, pWndObj);
//     }
//
// Exit:
//     DC_UnlockDc(pdc);
//     return Ret;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
