use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolZero};
use core::ffi::c_void;

extern "C" {
    fn TRACE(fmt: *const i8, ...);
    fn ERR(fmt: *const i8, ...);
    fn WARN(fmt: *const i8, ...);
    fn EngpFindGraphicsDevice(pustrDeviceName: PUNICODE_STRING, iDevNum: u32) -> PGRAPHICS_DEVICE;
    fn EngpGetDisplayDriverAccelerationLevel(pGraphicsDevice: PGRAPHICS_DEVICE) -> u32;
    fn LDEVOBJ_bProbeAndCaptureDevmode( pGraphicsDevice: PGRAPHICS_DEVICE, pdm: PDEVMODEW, localPdm: *mut PDEVMODEW, bDefault: u8, ) -> u8;
}

// ---- file-scope preamble (translated) ----
pub static mut gpmdev: PMDEVOBJ = core::ptr::null_mut();

pub unsafe fn MDEVOBJ_vEnable(pmdev: PMDEVOBJ) {
    let mut i: u32 = 0;
    while i < (*pmdev).cDev {
        PDEVOBJ_vEnableDisplay((*pmdev).dev[i as usize].ppdev);
        i += 1;
    }
}

unsafe fn MDEVOBJ_bDisable(pmdev: PMDEVOBJ) -> BOOL {
    let mut bSuccess: BOOL = 1;
    let mut i: ULONG;
    let mut j: ULONG;

    i = 0;
    while i < (*pmdev).cDev {
        if PDEVOBJ_bDisableDisplay((*pmdev).dev[i as usize].ppdev) == 0 {
            bSuccess = 0;
            break;
        }
        i += 1;
    }

    if bSuccess == 0 {
        j = 0;
        while j < i {
            PDEVOBJ_vEnableDisplay((*pmdev).dev[i as usize].ppdev);
            j += 1;
        }
    }

    bSuccess
}

unsafe fn MDEVOBJ_Create(
    pustrDeviceName: PUNICODE_STRING,
    pdm: PDEVMODEW,
) -> PMDEVOBJ {
    let mut pmdev: PMDEVOBJ = core::ptr::null_mut();
    let mut ppdev: PPDEVOBJ;
    let mut pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut dmDefault: DEVMODEW = core::mem::zeroed();
    let mut localPdm: PDEVMODEW;
    let mut iDevNum: u32 = 0;
    let mut dwAccelerationLevel: u32 = 0;

    TRACE(
        b"MDEVOBJ_Create('%wZ' '%dx%dx%d (%d Hz)')\n\0".as_ptr() as *const i8,
        pustrDeviceName,
        if !pdm.is_null() { (*pdm).dmPelsWidth } else { 0 },
        if !pdm.is_null() { (*pdm).dmPelsHeight } else { 0 },
        if !pdm.is_null() { (*pdm).dmBitsPerPel } else { 0 },
        if !pdm.is_null() { (*pdm).dmDisplayFrequency } else { 0 },
    );

    pmdev = ExAllocatePoolZero(
        PagedPool,
        core::mem::size_of::<MDEVOBJ>() + core::mem::size_of::<MDEVDISPLAY>(),
        0x7665644Du32,
    ) as PMDEVOBJ;
    if pmdev.is_null() {
        ERR(b"Failed to allocate memory for MDEV\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    (*pmdev).cDev = 0;

    loop {
        if !pustrDeviceName.is_null() {
            pGraphicsDevice = if iDevNum == 0 {
                EngpFindGraphicsDevice(pustrDeviceName, 0)
            } else {
                core::ptr::null_mut()
            };
        } else {
            pGraphicsDevice = EngpFindGraphicsDevice(core::ptr::null_mut(), iDevNum);
        }
        iDevNum += 1;

        if pGraphicsDevice.is_null() {
            TRACE(
                b"Done enumeration of graphic devices (DeviceName '%wZ' iDevNum %d)\n\0".as_ptr() as *const i8,
                pustrDeviceName,
                iDevNum,
            );
            break;
        }

        if pdm.is_null() {
            core::ptr::write_bytes(&mut dmDefault as *mut DEVMODEW, 0, 1);
            dmDefault.dmSize = core::mem::size_of::<DEVMODEW>() as u16;
        }

        dwAccelerationLevel = EngpGetDisplayDriverAccelerationLevel(pGraphicsDevice);

        if LDEVOBJ_bProbeAndCaptureDevmode(
            pGraphicsDevice,
            if !pdm.is_null() { pdm } else { &mut dmDefault as *mut DEVMODEW },
            &mut localPdm as *mut PDEVMODEW,
            if pdm.is_null() { 1u8 } else { 0u8 },
        ) != 0
        {
            ERR(
                b"MDEVOBJ_Create: ProbeAndCapture OK for '%S', creating PDEV (mode %lux%lux%lu)\n\0".as_ptr() as *const i8,
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                if !localPdm.is_null() { (*localPdm).dmPelsWidth } else { 0 },
                if !localPdm.is_null() { (*localPdm).dmPelsHeight } else { 0 },
                if !localPdm.is_null() { (*localPdm).dmBitsPerPel } else { 0 },
            );
            ppdev = PDEVOBJ_Create(pGraphicsDevice, localPdm, dwAccelerationLevel, LDEV_DEVICE_DISPLAY);
            if ppdev.is_null() {
                ERR(
                    b"MDEVOBJ_Create: PDEVOBJ_Create FAILED for '%S'\n\0".as_ptr() as *const i8,
                    (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                );
            }
        } else {
            ERR(
                b"MDEVOBJ_Create: LDEVOBJ_bProbeAndCaptureDevmode FAILED for '%S'\n\0".as_ptr() as *const i8,
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
            );
            ppdev = core::ptr::null_mut();
        }

        if !ppdev.is_null() {
            if (*pmdev).cDev >= 1 {
                let pmdevBigger: PMDEVOBJ = ExAllocatePoolZero(
                    PagedPool,
                    core::mem::size_of::<MDEVOBJ>() + (((*pmdev).cDev + 1) as usize) * core::mem::size_of::<MDEVDISPLAY>(),
                    0x7665644Du32,
                ) as PMDEVOBJ;
                if pmdevBigger.is_null() {
                    WARN(
                        b"Failed to allocate memory for MDEV. Skipping display '%S'\n\0".as_ptr() as *const i8,
                        (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                    );
                    continue;
                } else {
                    core::ptr::copy_nonoverlapping(
                        pmdev as *const u8,
                        pmdevBigger as *mut u8,
                        core::mem::size_of::<MDEVOBJ>() + ((*pmdev).cDev as usize) * core::mem::size_of::<MDEVDISPLAY>(),
                    );
                    ExFreePoolWithTag(pmdev as *mut core::ffi::c_void, 0x7665644Du32);
                    pmdev = pmdevBigger;
                }
            }

            TRACE(
                b"Adding '%S' to MDEV %p\n\0".as_ptr() as *const i8,
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                pmdev,
            );
            PDEVOBJ_vReference(ppdev);
            (*pmdev).dev[(*pmdev).cDev as usize].ppdev = ppdev;
            (*pmdev).cDev += 1;
        } else {
            WARN(
                b"Failed to add '%S' to MDEV %p\n\0".as_ptr() as *const i8,
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                pmdev,
            );
        }
    }

    if (*pmdev).cDev == 0 {
        TRACE(b"Failed to add any device to MDEV. Returning NULL\n\0".as_ptr() as *const i8);
        MDEVOBJ_vDestroy(pmdev);
        return core::ptr::null_mut();
    }

    TRACE(
        b"Returning new MDEV %p with %d devices\n\0".as_ptr() as *const i8,
        pmdev,
        (*pmdev).cDev,
    );
    pmdev
}

unsafe fn MDEVOBJ_vDestroy(pmdev: PMDEVOBJ) {
    let mut i: u32;

    i = 0;
    while i < (*pmdev).cDev {
        PDEVOBJ_vRelease((*pmdev).dev[i as usize].ppdev);
        i += 1;
    }

    if (*pmdev).cDev > 1 {
        PDEVOBJ_vRelease((*pmdev).ppdevGlobal);
    }

    ExFreePoolWithTag(pmdev as *mut core::ffi::c_void, 0x7665644Du32);
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * PURPOSE:          Support for meta devices
//  * FILE:             win32ss/gdi/eng/mdevobj.c
//  * PROGRAMER:        Hervé Poussineau
//  */
//
// #include <win32k.h>
// #define NDEBUG
// #include <debug.h>
// DBG_DEFAULT_CHANNEL(EngMDev);
//
// PMDEVOBJ gpmdev = NULL; /* FIXME: should be stored in gpDispInfo->pmdev */
//
// VOID
// MDEVOBJ_vEnable(
//     _Inout_ PMDEVOBJ pmdev)
// {
//     ULONG i;
//
//     for (i = 0; i < pmdev->cDev; i++)
//     {
//         PDEVOBJ_vEnableDisplay(pmdev->dev[i].ppdev);
//     }
// }
//
// BOOL
// MDEVOBJ_bDisable(
//     _Inout_ PMDEVOBJ pmdev)
// {
//     BOOL bSuccess = TRUE;
//     ULONG i, j;
//
//     for (i = 0; i < pmdev->cDev; i++)
//     {
//         if (!PDEVOBJ_bDisableDisplay(pmdev->dev[i].ppdev))
//         {
//             bSuccess = FALSE;
//             break;
//         }
//     }
//
//     if (!bSuccess)
//     {
//         /* Failed to disable all PDEVs. Reenable those we have disabled */
//         for (j = 0; j < i; j++)
//         {
//             PDEVOBJ_vEnableDisplay(pmdev->dev[i].ppdev);
//         }
//     }
//
//     return bSuccess;
// }
//
// PMDEVOBJ
// MDEVOBJ_Create(
//     _In_opt_ PUNICODE_STRING pustrDeviceName,
//     _In_opt_ PDEVMODEW pdm)
// {
//     PMDEVOBJ pmdev = NULL;
//     PPDEVOBJ ppdev;
//     PGRAPHICS_DEVICE pGraphicsDevice;
//     DEVMODEW dmDefault;
//     PDEVMODEW localPdm;
//     ULONG iDevNum = 0;
//     ULONG dwAccelerationLevel = 0;
//
//     TRACE("MDEVOBJ_Create('%wZ' '%dx%dx%d (%d Hz)')\n",
//         pustrDeviceName,
//         pdm ? pdm->dmPelsWidth : 0,
//         pdm ? pdm->dmPelsHeight : 0,
//         pdm ? pdm->dmBitsPerPel : 0,
//         pdm ? pdm->dmDisplayFrequency : 0);
//
//     pmdev = ExAllocatePoolZero(PagedPool, sizeof(MDEVOBJ) + sizeof(MDEVDISPLAY), GDITAG_MDEV);
//     if (!pmdev)
//     {
//         ERR("Failed to allocate memory for MDEV\n");
//         return NULL;
//     }
//
//     pmdev->cDev = 0;
//
//     while (TRUE)
//     {
//         /* Get the right graphics devices: either the specified one, or all of them (one after one) */
//         if (pustrDeviceName)
//             pGraphicsDevice = (iDevNum == 0) ? EngpFindGraphicsDevice(pustrDeviceName, 0) : NULL;
//         else
//             pGraphicsDevice = EngpFindGraphicsDevice(NULL, iDevNum);
//         iDevNum++;
//
//         if (!pGraphicsDevice)
//         {
//             TRACE("Done enumeration of graphic devices (DeviceName '%wZ' iDevNum %d)\n", pustrDeviceName, iDevNum);
//             break;
//         }
//
//         if (!pdm)
//         {
//             /* No settings requested. Provide nothing and LDEVOBJ_bProbeAndCaptureDevmode
//              * will read default settings from registry */
//             RtlZeroMemory(&dmDefault, sizeof(dmDefault));
//             dmDefault.dmSize = sizeof(dmDefault);
//         }
//
//         dwAccelerationLevel = EngpGetDisplayDriverAccelerationLevel(pGraphicsDevice);
//
//         /* Get or create a PDEV for these settings */
//         if (LDEVOBJ_bProbeAndCaptureDevmode(pGraphicsDevice, pdm ? pdm : &dmDefault, &localPdm, !pdm))
//         {
//             ERR("MDEVOBJ_Create: ProbeAndCapture OK for '%S', creating PDEV (mode %lux%lux%lu)\n",
//                 pGraphicsDevice->szWinDeviceName,
//                 localPdm ? localPdm->dmPelsWidth : 0,
//                 localPdm ? localPdm->dmPelsHeight : 0,
//                 localPdm ? localPdm->dmBitsPerPel : 0);
//             ppdev = PDEVOBJ_Create(pGraphicsDevice, localPdm, dwAccelerationLevel, LDEV_DEVICE_DISPLAY);
//             if (!ppdev)
//                 ERR("MDEVOBJ_Create: PDEVOBJ_Create FAILED for '%S'\n", pGraphicsDevice->szWinDeviceName);
//         }
//         else
//         {
//             ERR("MDEVOBJ_Create: LDEVOBJ_bProbeAndCaptureDevmode FAILED for '%S'\n", pGraphicsDevice->szWinDeviceName);
//             ppdev = NULL;
//         }
//
//         if (ppdev)
//         {
//             /* Great. We have a found a matching PDEV. Store it in MDEV */
//             if (pmdev->cDev >= 1)
//             {
//                 /* We have to reallocate MDEV to add space for the new display */
//                 PMDEVOBJ pmdevBigger = ExAllocatePoolZero(PagedPool, sizeof(MDEVOBJ) + (pmdev->cDev + 1) * sizeof(MDEVDISPLAY), GDITAG_MDEV);
//                 if (!pmdevBigger)
//                 {
//                     WARN("Failed to allocate memory for MDEV. Skipping display '%S'\n", pGraphicsDevice->szWinDeviceName);
//                     continue;
//                 }
//                 else
//                 {
//                     /* Copy existing data */
//                     RtlCopyMemory(pmdevBigger, pmdev, sizeof(MDEVOBJ) + pmdev->cDev * sizeof(MDEVDISPLAY));
//                     ExFreePoolWithTag(pmdev, GDITAG_MDEV);
//                     pmdev = pmdevBigger;
//                 }
//             }
//
//             TRACE("Adding '%S' to MDEV %p\n", pGraphicsDevice->szWinDeviceName, pmdev);
//             PDEVOBJ_vReference(ppdev);
//             pmdev->dev[pmdev->cDev].ppdev = ppdev;
//             pmdev->cDev++;
//         }
//         else
//         {
//             WARN("Failed to add '%S' to MDEV %p\n", pGraphicsDevice->szWinDeviceName, pmdev);
//         }
//     }
//
//     if (pmdev->cDev == 0)
//     {
//         TRACE("Failed to add any device to MDEV. Returning NULL\n");
//         MDEVOBJ_vDestroy(pmdev);
//         return NULL;
//     }
//
//     TRACE("Returning new MDEV %p with %d devices\n", pmdev, pmdev->cDev);
//     return pmdev;
// }
//
// VOID
// MDEVOBJ_vDestroy(
//     _Inout_ PMDEVOBJ pmdev)
// {
//     ULONG i;
//
//     for (i = 0; i < pmdev->cDev; i++)
//     {
//         PDEVOBJ_vRelease(pmdev->dev[i].ppdev);
//     }
//
//     if (pmdev->cDev > 1)
//         PDEVOBJ_vRelease(pmdev->ppdevGlobal);
//
//     ExFreePoolWithTag(pmdev, GDITAG_MDEV);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
