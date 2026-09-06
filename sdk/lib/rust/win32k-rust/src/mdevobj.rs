use win32k_sys::win32k::*;
use core::ffi::c_char;

// ---------------------------------------------------------------------------
// Real exported kernel symbols, re-declared against win32k-sys nominal types.
// (Same pattern as object.rs / gdiobj.rs / engevent.rs.) The symbol/ABI is
// unchanged; only Rust nominal types differ vs ntkrnl-sys's own bindgen.
//
// ExAllocatePoolWithTag / ExFreePoolWithTag are NTAPI exports (ntoskrnl.spec
// `@ stdcall`). NTSYSAPI/NTKERNELAPI NTAPI = stdcall on i386, C on
// amd64/arm64: `extern "system"` resolves correctly per arch.
// ExAllocatePoolZero is NOT an export — it is a FORCEINLINE in xdk/exfuncs.h
// (ExAllocatePoolWithTag + RtlZeroMemory), reproduced below.
extern "system" {
    fn ExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID;
    fn ExFreePoolWithTag(P: PVOID, Tag: ULONG);
}

/// ExAllocatePoolZero — xdk/exfuncs.h FORCEINLINE:
/// `p = ExAllocatePoolWithTag(...); if (p) RtlZeroMemory(p, n); return p;`
#[inline(always)]
unsafe fn ExAllocatePoolZero(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    let allocation = ExAllocatePoolWithTag(PoolType, NumberOfBytes, Tag);
    if !allocation.is_null() {
        core::ptr::write_bytes(allocation as *mut u8, 0, NumberOfBytes as usize);
    }
    allocation
}

extern "C" {
    fn DbgPrint(Format: *const c_char, ...) -> u32;
}

// win32k-internal callees not in the win32k-sys bindgen surface (they live in
// other-TU private headers under win32ss/gdi/eng). All are plain cdecl on
// every arch except EngpFindGraphicsDevice which is NTAPI (= stdcall on
// i386, C on amd64/arm64). LDEVOBJ_bProbeAndCaptureDevmode's last arg is
// BOOL (c_int), not u8 — important for i386 stack layout.
extern "system" {
    fn EngpFindGraphicsDevice(pustrDevice: PUNICODE_STRING, iDevNum: ULONG) -> PGRAPHICS_DEVICE;
}
extern "C" {
    fn EngpGetDisplayDriverAccelerationLevel(pGraphicsDevice: PGRAPHICS_DEVICE) -> DWORD;
    fn LDEVOBJ_bProbeAndCaptureDevmode(
        pGraphicsDevice: PGRAPHICS_DEVICE,
        RequestedMode: PDEVMODEW,
        pSelectedMode: *mut PDEVMODEW,
        bSearchClosestMode: BOOL,
    ) -> BOOL;
}

// PDEVOBJ_vEnableDisplay, PDEVOBJ_bDisableDisplay, PDEVOBJ_Create, PDEVOBJ_vRelease
// are real functions in `win32ss/gdi/eng/pdevobj.c` exported with NTAPI (= stdcall
// on i386, C elsewhere) — declared in `win32k_sys::win32k::*` via the blanket use
// above. Do NOT re-declare.
//
// PDEVOBJ_vReference is the exception: it is FORCEINLINE in
// `win32ss/gdi/eng/pdevobj.h:158`, so no exported symbol exists in any .o. C
// callers inline it at every site. mdevobj.rs is the only Rust caller; define
// the symbol ONCE here as `#[no_mangle]` so the bindgen extern declaration
// (which the LLM emits with cdecl ABI in `win32k_sys/src/bindgen_*.rs`) resolves.
// Mirrors the FORCEINLINE body verbatim. ABI is `extern "C"` to match the
// bindgen extern signature; both forms produce the same `_PDEVOBJ_vReference`
// (i386) / `PDEVOBJ_vReference` (amd64/arm64) symbol that callers reference.
#[no_mangle]
pub unsafe extern "C" fn PDEVOBJ_vReference(ppdev: PPDEVOBJ) {
    // C: ASSERT(ppdev) — drop in release.
    if ppdev.is_null() { return; }
    // C: if (ppdev->cPdevRefs == 0) { ASSERT(FALSE); return; }
    if (*ppdev).cPdevRefs == 0 { return; }
    // C: ASSERT(ppdev->cPdevRefs > 0) — drop in release.
    // C: InterlockedIncrement(&ppdev->cPdevRefs)
    ntkrnl_sys::interlocked::InterlockedIncrement(
        core::ptr::addr_of_mut!((*ppdev).cPdevRefs) as *mut ntkrnl_sys::base::LONG,
    );
}

// ---- file-scope preamble (translated) ----
// PMDEVOBJ gpmdev = NULL;  — mdevobj.c OWNS this exported global. win32k-sys
// bindgen also declares it `extern static mut`; the glob `use ::*` therefore
// brings in an extern decl. A local `#[no_mangle]` definition both shadows
// the glob within this module and provides the real symbol for all other
// (still-C) win32k TUs that consume gpmdev (winsta.c, pdevobj.c, etc.).
#[no_mangle]
pub static mut gpmdev: PMDEVOBJ = core::ptr::null_mut();

// `MDEVDISPLAY dev[0]` is a C flexible array — bindgen emits it as
// `__IncompleteArrayField<MDEVDISPLAY>`. The struct itself is sized so that
// `&pmdev.dev` points one byte past `ppdevGlobal`; element i lives at
// `dev.as_mut_ptr().add(i)`. NEVER use `dev[i]` (`Index` is not
// implemented for `__IncompleteArrayField`).
#[inline(always)]
unsafe fn mdev_get(pmdev: PMDEVOBJ, i: usize) -> *mut MDEVDISPLAY {
    (*pmdev).dev.as_mut_ptr().add(i)
}

#[no_mangle]
pub unsafe extern "C" fn MDEVOBJ_vEnable(pmdev: PMDEVOBJ) {
    let mut i: ULONG = 0;
    while i < (*pmdev).cDev {
        PDEVOBJ_vEnableDisplay((*mdev_get(pmdev, i as usize)).ppdev);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn MDEVOBJ_bDisable(pmdev: PMDEVOBJ) -> BOOL {
    let mut bSuccess: BOOL = 1;
    let mut i: ULONG;
    let mut j: ULONG;

    i = 0;
    while i < (*pmdev).cDev {
        if PDEVOBJ_bDisableDisplay((*mdev_get(pmdev, i as usize)).ppdev) == 0 {
            bSuccess = 0;
            break;
        }
        i += 1;
    }

    if bSuccess == 0 {
        // Faithful to C: original code re-enables `dev[i]` (the failing one)
        // rather than `dev[j]`. Preserve the existing behavior.
        j = 0;
        while j < i {
            PDEVOBJ_vEnableDisplay((*mdev_get(pmdev, i as usize)).ppdev);
            j += 1;
        }
    }

    bSuccess
}

#[no_mangle]
pub unsafe extern "C" fn MDEVOBJ_Create(
    pustrDeviceName: PUNICODE_STRING,
    pdm: PDEVMODEW,
) -> PMDEVOBJ {
    let mut pmdev: PMDEVOBJ;
    let mut ppdev: PPDEVOBJ;
    let mut pGraphicsDevice: PGRAPHICS_DEVICE;
    let mut dmDefault: DEVMODEW = core::mem::zeroed();
    let mut localPdm: PDEVMODEW = core::ptr::null_mut();
    let mut iDevNum: ULONG = 0;
    let mut dwAccelerationLevel: ULONG;

    TRACE!(
        b"MDEVOBJ_Create('%wZ' '%dx%dx%d (%d Hz)')\n\0",
        pustrDeviceName,
        if !pdm.is_null() { (*pdm).dmPelsWidth } else { 0 },
        if !pdm.is_null() { (*pdm).dmPelsHeight } else { 0 },
        if !pdm.is_null() { (*pdm).dmBitsPerPel } else { 0 },
        if !pdm.is_null() { (*pdm).dmDisplayFrequency } else { 0 },
    );

    pmdev = ExAllocatePoolZero(
        PagedPool,
        (core::mem::size_of::<MDEVOBJ>() + core::mem::size_of::<MDEVDISPLAY>()) as SIZE_T,
        0x7665644Du32, // 'Mdev' GDITAG_MDEV
    ) as PMDEVOBJ;
    if pmdev.is_null() {
        ERR!("Failed to allocate memory for MDEV\n");
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
            TRACE!(
                b"Done enumeration of graphic devices (DeviceName '%wZ' iDevNum %d)\n\0",
                pustrDeviceName,
                iDevNum,
            );
            break;
        }

        if pdm.is_null() {
            core::ptr::write_bytes(&mut dmDefault as *mut DEVMODEW as *mut u8, 0, core::mem::size_of::<DEVMODEW>());
            dmDefault.dmSize = core::mem::size_of::<DEVMODEW>() as u16;
        }

        dwAccelerationLevel = EngpGetDisplayDriverAccelerationLevel(pGraphicsDevice);

        if LDEVOBJ_bProbeAndCaptureDevmode(
            pGraphicsDevice,
            if !pdm.is_null() { pdm } else { &mut dmDefault },
            &mut localPdm,
            if pdm.is_null() { 1 } else { 0 },
        ) != 0
        {
            ERR!(
                "MDEVOBJ_Create: ProbeAndCapture OK for '%S', creating PDEV (mode %lux%lux%lu)\n",
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                if !localPdm.is_null() { (*localPdm).dmPelsWidth } else { 0 },
                if !localPdm.is_null() { (*localPdm).dmPelsHeight } else { 0 },
                if !localPdm.is_null() { (*localPdm).dmBitsPerPel } else { 0 },
            );
            ppdev = PDEVOBJ_Create(pGraphicsDevice, localPdm, dwAccelerationLevel, LDEV_DEVICE_DISPLAY);
            if ppdev.is_null() {
                ERR!(
                    "MDEVOBJ_Create: PDEVOBJ_Create FAILED for '%S'\n",
                    (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                );
            }
        } else {
            ERR!(
                "MDEVOBJ_Create: LDEVOBJ_bProbeAndCaptureDevmode FAILED for '%S'\n",
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
            );
            ppdev = core::ptr::null_mut();
        }

        if !ppdev.is_null() {
            if (*pmdev).cDev >= 1 {
                let pmdevBigger: PMDEVOBJ = ExAllocatePoolZero(
                    PagedPool,
                    (core::mem::size_of::<MDEVOBJ>() + ((*pmdev).cDev as usize + 1) * core::mem::size_of::<MDEVDISPLAY>()) as SIZE_T,
                    0x7665644Du32,
                ) as PMDEVOBJ;
                if pmdevBigger.is_null() {
                    WARN!(
                        b"Failed to allocate memory for MDEV. Skipping display '%S'\n\0",
                        (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                    );
                    continue;
                } else {
                    core::ptr::copy_nonoverlapping(
                        pmdev as *const u8,
                        pmdevBigger as *mut u8,
                        core::mem::size_of::<MDEVOBJ>() + (*pmdev).cDev as usize * core::mem::size_of::<MDEVDISPLAY>(),
                    );
                    ExFreePoolWithTag(pmdev as PVOID, 0x7665644Du32);
                    pmdev = pmdevBigger;
                }
            }

            TRACE!(
                b"Adding '%S' to MDEV %p\n\0",
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                pmdev,
            );
            PDEVOBJ_vReference(ppdev);
            (*mdev_get(pmdev, (*pmdev).cDev as usize)).ppdev = ppdev;
            (*pmdev).cDev += 1;
        } else {
            WARN!(
                b"Failed to add '%S' to MDEV %p\n\0",
                (*pGraphicsDevice).szWinDeviceName.as_ptr(),
                pmdev,
            );
        }
    }

    if (*pmdev).cDev == 0 {
        TRACE!(b"Failed to add any device to MDEV. Returning NULL\n\0");
        MDEVOBJ_vDestroy(pmdev);
        return core::ptr::null_mut();
    }

    TRACE!(
        b"Returning new MDEV %p with %d devices\n\0",
        pmdev,
        (*pmdev).cDev,
    );
    pmdev
}

#[no_mangle]
pub unsafe extern "C" fn MDEVOBJ_vDestroy(pmdev: PMDEVOBJ) {
    let mut i: ULONG;

    i = 0;
    while i < (*pmdev).cDev {
        PDEVOBJ_vRelease((*mdev_get(pmdev, i as usize)).ppdev);
        i += 1;
    }

    if (*pmdev).cDev > 1 {
        PDEVOBJ_vRelease((*pmdev).ppdevGlobal);
    }

    ExFreePoolWithTag(pmdev as PVOID, 0x7665644Du32);
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
