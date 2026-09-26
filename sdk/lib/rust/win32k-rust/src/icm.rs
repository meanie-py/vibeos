use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlCompareMemory};
use core::ffi::c_void;

// Not in ntkrnl_sys::base; declared per-file, as the crate does elsewhere.
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// SafeProbeForWrite: NTAPI (extern "system" = stdcall i386 / cdecl else), Length = SIZE_T.
extern "system" {
    fn SafeProbeForWrite(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
    fn SafeProbeForRead(Address: LPVOID, Length: SIZE_T, Alignment: u32) -> NTSTATUS;
}
// SetLastNtError: FASTCALL (i386 @Name@N; cdecl elsewhere).
#[cfg(target_arch = "x86")] extern "fastcall" { fn SetLastNtError(Status: NTSTATUS); }
#[cfg(not(target_arch = "x86"))] extern "C" { fn SetLastNtError(Status: NTSTATUS); }

// ---- macro reimpls (no exported symbol) ----
// GDI_HANDLE_GET_TYPE(h) = type bits of a GDI handle.
#[inline(always)] unsafe fn GDI_HANDLE_GET_TYPE(handle: HCOLORSPACE) -> u32 { ((handle as usize) & 0x007f0000) as u32 }
// COLORSPACEOBJ_* (color.h) over GDIOBJ_* primitives.
#[inline(always)] unsafe fn COLORSPACEOBJ_AllocCSWithHandle() -> PCOLORSPACE {
    GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_COLORSPACE as ULONG, core::mem::size_of::<COLORSPACE>() as ULONG) as PCOLORSPACE
}
#[inline(always)] unsafe fn COLORSPACEOBJ_LockCS(hCS: HCOLORSPACE) -> PCOLORSPACE {
    GDIOBJ_LockObject(hCS as HGDIOBJ, GDIObjType_ICMLCS_TYPE as _) as PCOLORSPACE
}
#[inline(always)] unsafe fn COLORSPACEOBJ_UnlockCS(pCS: PCOLORSPACE) { GDIOBJ_vUnlockObject(pCS as POBJ); }

// ---- dropped-preamble helpers ----
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
// PALETTE_Share{Lock,Unlock}Palette macros (palette.h) over GDIOBJ_* primitives.
#[inline(always)] unsafe fn PALETTE_ShareLockPalette(h: HPALETTE) -> PPALETTE { GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_PALETTE as DWORD) as PPALETTE }
#[inline(always)] unsafe fn PALETTE_ShareUnlockPalette(p: PPALETTE) { GDIOBJ_vDereferenceObject(core::ptr::addr_of_mut!((*p).BaseObject)); }
#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] unsafe fn RtlCopyMemory(d: *mut c_void, s: *const c_void, n: usize) { core::ptr::copy_nonoverlapping(s as *const u8, d as *mut u8, n); }
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): COLORSPACEOBJ_AllocCSWithHandle, COLORSPACEOBJ_UnlockCS — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): COLORSPACEOBJ_AllocCSWithHandle, COLORSPACEOBJ_UnlockCS — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// drop_gdiobj_macro_externs: dropped 2 bogus extern decl(s) for GDI-object macro(s): COLORSPACEOBJ_LockCS, COLORSPACEOBJ_UnlockCS — these are macros in gdi/eng/<type>.h or gdi/ntgdi/<type>.h, not real fns. Inline-expand via GDIOBJ_* primitives per win32k_translation_prompt.md §6.
// ---- file-scope preamble (translated) ----
pub static mut hStockColorSpace: HCOLORSPACE = core::ptr::null_mut();
pub static mut IcmGammaRangeSet: INT = 128;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiCreateColorSpace(
    pLogColorSpace: PLOGCOLORSPACEEXW,
) -> HCOLORSPACE {
    IntGdiCreateColorSpaceImpl(pLogColorSpace)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiCreateColorSpace(
    pLogColorSpace: PLOGCOLORSPACEEXW,
) -> HCOLORSPACE {
    IntGdiCreateColorSpaceImpl(pLogColorSpace)
}

unsafe fn IntGdiCreateColorSpaceImpl(
    pLogColorSpace: PLOGCOLORSPACEEXW,
) -> HCOLORSPACE {
    let pCS: PCOLORSPACE = COLORSPACEOBJ_AllocCSWithHandle();
    if pCS.is_null() {
        return core::ptr::null_mut();
    }

    let hCS: HCOLORSPACE = (*pCS).BaseObject.hHmgr as HCOLORSPACE;

    (*pCS).lcsColorSpace = (*pLogColorSpace).lcsColorSpace;
    (*pCS).dwFlags = (*pLogColorSpace).dwFlags;

    COLORSPACEOBJ_UnlockCS(pCS);
    hCS
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGdiDeleteColorSpace(hColorSpace: HCOLORSPACE) -> BOOL {
    IntGdiDeleteColorSpaceImpl(hColorSpace)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGdiDeleteColorSpace(hColorSpace: HCOLORSPACE) -> BOOL {
    IntGdiDeleteColorSpaceImpl(hColorSpace)
}

unsafe fn IntGdiDeleteColorSpaceImpl(hColorSpace: HCOLORSPACE) -> BOOL {

    let mut Ret: BOOL = 0;

    if (hColorSpace != hStockColorSpace)
        && (GDI_HANDLE_GET_TYPE(hColorSpace) == GDILoObjType_LO_ICMLCS_TYPE)
    {
        Ret = GreDeleteObject(hColorSpace as _);
        if Ret == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiCreateColorSpace(
    pLogColorSpace: PLOGCOLORSPACEEXW,
) -> HANDLE {
    let mut Safelcs: LOGCOLORSPACEEXW = core::mem::zeroed();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Status = SafeProbeForRead(
        pLogColorSpace as *mut core::ffi::c_void,
        core::mem::size_of::<LOGCOLORSPACEEXW>() as _,
        1,
    );
    // The probe VALIDATES only; this copy is the access. A fault lands in Status,
    // which is checked just below -- one _SEH2_TRY's worth of behaviour.
    if NT_SUCCESS(Status)
        && !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(Safelcs) as *mut core::ffi::c_void,
            pLogColorSpace as *const core::ffi::c_void,
            core::mem::size_of::<LOGCOLORSPACEEXW>() as _,
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    IntGdiCreateColorSpace(core::ptr::addr_of_mut!(Safelcs)) as _
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDeleteColorSpace(hColorSpace: HANDLE) -> BOOL {
    IntGdiDeleteColorSpace(hColorSpace as HCOLORSPACE)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP) -> BOOL {
    IntGetDeviceGammaRamp_impl(hPDev, Ramp)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP) -> BOOL {
    IntGetDeviceGammaRamp_impl(hPDev, Ramp)
}

unsafe fn IntGetDeviceGammaRamp_impl(hPDev: HDEV, Ramp: PGAMMARAMP) -> BOOL {
    let pGDev: PPDEVOBJ = hPDev as PPDEVOBJ;

    if (*pGDev).flFlags & PDEV_DISPLAY == 0 {
        return 0;
    }

    if (*pGDev).devinfo.iDitherFormat == BMF_8BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_16BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_24BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_32BPP as _
    {
        if (*pGDev).flFlags & PDEV_GAMMARAMP_TABLE != 0 {
            RtlCopyMemory(
                Ramp as *mut _,
                (*pGDev).pvGammaRamp as *const _,
                core::mem::size_of::<GAMMARAMP>() as _,
            );
        } else {
            for i in 0..256 {
                let NewValue: i32 = i * 256;
                (*Ramp).Red[i as usize] = NewValue as u16;
                (*Ramp).Green[i as usize] = NewValue as u16;
                (*Ramp).Blue[i as usize] = NewValue as u16;
            }
        }
        return 1;
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiGetDeviceGammaRamp(hDC: HDC, Ramp: LPVOID) -> BOOL {
    let Ret: BOOL;
    let dc: PDC;
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let SafeRamp: PGAMMARAMP;

    if Ramp.is_null() {
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    SafeRamp = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<GAMMARAMP>() as _,
        GDITAG_ICM,
    ) as PGAMMARAMP;
    if SafeRamp.is_null() {
        DC_UnlockDc(dc);
        EngSetLastError(STATUS_NO_MEMORY as u32);
        return 0;
    }

    Ret = IntGetDeviceGammaRamp((*dc).ppdev as HDEV, SafeRamp);
    if Ret == 0 {
        DC_UnlockDc(dc);
        ExFreePoolWithTag(SafeRamp as LPVOID, GDITAG_ICM);
        return Ret;
    }

    Status = SafeProbeForWrite(Ramp, core::mem::size_of::<GAMMARAMP>() as _, 1);
    // Validate-only probe; the publish is the access. A fault lands in Status, checked
    // below, and the unlock/free that follow still run either way.
    if NT_SUCCESS(Status)
        && !crate::copy::publish_to_user(
            Ramp as *mut core::ffi::c_void,
            SafeRamp as *const core::ffi::c_void,
            core::mem::size_of::<GAMMARAMP>(),
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    DC_UnlockDc(dc);
    ExFreePoolWithTag(SafeRamp as LPVOID, GDITAG_ICM);

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return 0;
    }
    Ret
}

const GDITAG_ICM: u32 = u32::from_le_bytes(*b"mciG");
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetColorSpace(hdc: HDC, hColorSpace: HCOLORSPACE) -> BOOL {

    let pDC: PDC = DC_LockDc(hdc);
    if pDC.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }
    let pdcattr: PDC_ATTR = (*pDC).pdcattr;

    if (*pdcattr).hColorSpace == hColorSpace {
        DC_UnlockDc(pDC);
        return 1;
    }

    let pCS: PCOLORSPACE = COLORSPACEOBJ_LockCS(hColorSpace);
    if pCS.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    if !(*pDC).dclevel.pColorSpace.is_null() {
        GDIOBJ_vDereferenceObject((*pDC).dclevel.pColorSpace as POBJ);
    }

    (*pDC).dclevel.pColorSpace = pCS as _;
    (*pdcattr).hColorSpace = hColorSpace;

    COLORSPACEOBJ_UnlockCS(pCS);
    DC_UnlockDc(pDC);
    1
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UpdateDeviceGammaRamp(hPDev: HDEV) -> BOOL {
    UpdateDeviceGammaRamp_impl(hPDev)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UpdateDeviceGammaRamp(hPDev: HDEV) -> BOOL {
    UpdateDeviceGammaRamp_impl(hPDev)
}

unsafe fn UpdateDeviceGammaRamp_impl(hPDev: HDEV) -> BOOL {
    let mut Ret: BOOL = 0;
    let palGDI: PPALETTE;
    let palPtr: *mut PALOBJ;
    let pGDev: PPDEVOBJ = hPDev as PPDEVOBJ;

    if (*pGDev).devinfo.iDitherFormat == BMF_8BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_16BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_24BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_32BPP as _
    {
        if let Some(IcmSetDeviceGammaRamp) = (*pGDev).__bindgen_anon_1.DriverFunctions.IcmSetDeviceGammaRamp {
            return IcmSetDeviceGammaRamp(
                (*pGDev).dhpdev,
                IGRF_RGB_256WORDS as _,
                (*pGDev).pvGammaRamp,
            );
        }

        if (*pGDev).devinfo.iDitherFormat != BMF_8BPP as _
            || ((*pGDev).gdiinfo.flRaster & RC_PALETTE as u32) == 0
        {
            return 0;
        }

        if ((*pGDev).flFlags & PDEV_GAMMARAMP_TABLE as u32) == 0 {
            return 0;
        }

        palGDI = PALETTE_ShareLockPalette((*pGDev).devinfo.hpalDefault);
        if palGDI.is_null() {
            return 0;
        }
        palPtr = palGDI as *mut PALOBJ;

        if ((*pGDev).flFlags & PDEV_GAMMARAMP_TABLE as u32) != 0 {
            (*palGDI).flFlags |= PAL_GAMMACORRECTION as u32;
        } else {
            (*palGDI).flFlags &= !(PAL_GAMMACORRECTION as u32);
        }

        if ((*pGDev).flFlags & PDEV_DRIVER_PUNTED_CALL as u32) == 0 {
            (*palGDI).hPDev = hPDev;
            if let Some(SetPalette) = (*pGDev).__bindgen_anon_1.DriverFunctions.SetPalette {
                Ret = SetPalette(
                    (*pGDev).dhpdev,
                    palPtr,
                    0,
                    0,
                    (*palGDI).NumColors,
                );
            }
        }
        PALETTE_ShareUnlockPalette(palGDI);
        Ret
    } else {
        0
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntSetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP, Test: BOOL) -> BOOL {
    IntSetDeviceGammaRamp_impl(hPDev, Ramp, Test)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntSetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP, Test: BOOL) -> BOOL {
    IntSetDeviceGammaRamp_impl(hPDev, Ramp, Test)
}

unsafe fn IntSetDeviceGammaRamp_impl(hPDev: HDEV, Ramp: PGAMMARAMP, Test: BOOL) -> BOOL {
    let IcmGR: WORD;
    let mut i: WORD;
    let mut R: WORD;
    let mut G: WORD;
    let mut B: WORD;
    let mut Ret: BOOL = 0;
    let mut TstPeak: BOOL;
    let pGDev: PPDEVOBJ = hPDev as PPDEVOBJ;

    if hPDev.is_null() {
        return 0;
    }

    if ((*pGDev).flFlags & PDEV_DISPLAY) == 0 {
        return 0;
    }

    if (*pGDev).devinfo.iDitherFormat == BMF_8BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_16BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_24BPP as _
        || (*pGDev).devinfo.iDitherFormat == BMF_32BPP as _
    {
        if (*pGDev).__bindgen_anon_1.DriverFunctions.IcmSetDeviceGammaRamp.is_none() {
            // No driver support
            if ((*pGDev).devinfo.flGraphicsCaps2 & GCAPS2_CHANGEGAMMARAMP) == 0 {
                if (*pGDev).devinfo.iDitherFormat != BMF_8BPP as _
                    || ((*pGDev).gdiinfo.flRaster & RC_PALETTE) == 0
                {
                    return 0;
                }
            }
        }

        if ((*pGDev).flFlags & PDEV_GAMMARAMP_TABLE) != 0 {
            if RtlCompareMemory(
                (*pGDev).pvGammaRamp,
                Ramp as *const _,
                core::mem::size_of::<GAMMARAMP>() as _,
            ) == core::mem::size_of::<GAMMARAMP>() as _
            {
                return 1;
            }
        }

        // Verify Ramp is inside range.
        IcmGR = (-IcmGammaRangeSet) as WORD;
        TstPeak = (Test == 0) as BOOL;
        i = 0;
        while i < 256 {
            R = (*Ramp).Red[i as usize] / 256;
            G = (*Ramp).Green[i as usize] / 256;
            B = (*Ramp).Blue[i as usize] / 256;

            let mut skip = false;
            if R >= IcmGR {
                if R <= (IcmGammaRangeSet as WORD).wrapping_add(i) {
                    if G >= IcmGR
                        && G <= (IcmGammaRangeSet as WORD).wrapping_add(i)
                        && B >= IcmGR
                        && B <= (IcmGammaRangeSet as WORD).wrapping_add(i)
                    {
                        skip = true;
                    }
                }
            }

            if !skip {
                if Test != 0 {
                    return Ret;
                }

                if TstPeak != 0 {
                    if R != IcmGR.wrapping_mul(256)
                        || G != IcmGR.wrapping_mul(256)
                        || B != IcmGR.wrapping_mul(256)
                    {
                        TstPeak = 0;
                    }
                }
            }

            i += 1;
        }

        if (*pGDev).pvGammaRamp.is_null() && ((*pGDev).flFlags & PDEV_GAMMARAMP_TABLE) == 0 {
            (*pGDev).pvGammaRamp = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<GAMMARAMP>() as _,
                GDITAG_ICM,
            );
            (*pGDev).flFlags |= PDEV_GAMMARAMP_TABLE;
        }

        if !(*pGDev).pvGammaRamp.is_null() {
            RtlCopyMemory(
                (*pGDev).pvGammaRamp,
                Ramp as *const _,
                core::mem::size_of::<GAMMARAMP>() as _,
            );
        }

        Ret = UpdateDeviceGammaRamp(hPDev);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetDeviceGammaRamp(hDC: HDC, Ramp: LPVOID) -> BOOL {
    let Ret: BOOL;
    let dc: PDC;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let SafeRamp: PGAMMARAMP;

    if Ramp.is_null() {
        return 0;
    }

    dc = DC_LockDc(hDC);
    if dc.is_null() {
        EngSetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    SafeRamp = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<GAMMARAMP>() as _,
        GDITAG_ICM,
    ) as PGAMMARAMP;
    if SafeRamp.is_null() {
        DC_UnlockDc(dc);
        EngSetLastError(STATUS_NO_MEMORY as ULONG);
        return 0;
    }

    Status = SafeProbeForRead(Ramp, core::mem::size_of::<GAMMARAMP>() as _, 1);
    // The probe VALIDATES only; this copy is the access. A fault lands in Status,
    // which is checked just below -- one _SEH2_TRY's worth of behaviour.
    if NT_SUCCESS(Status)
        && !crate::copy::capture_from_user(
            SafeRamp as PVOID,
            Ramp,
            core::mem::size_of::<GAMMARAMP>() as _,
        )
    {
        Status = STATUS_ACCESS_VIOLATION;
    }

    if !NT_SUCCESS(Status) {
        DC_UnlockDc(dc);
        ExFreePoolWithTag(SafeRamp as PVOID, GDITAG_ICM);
        SetLastNtError(Status);
        return 0;
    }

    Ret = IntSetDeviceGammaRamp((*dc).ppdev as HDEV, SafeRamp, 1);
    DC_UnlockDc(dc);
    ExFreePoolWithTag(SafeRamp as PVOID, GDITAG_ICM);
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiSetIcmMode(
    hDC: HDC,
    nCommand: ULONG,
    EnableICM: ULONG,
) -> INT {
    if EnableICM == ICM_OFF {
        return ICM_OFF as INT;
    }
    if EnableICM == ICM_ON {
        return 0;
    }
    if EnableICM == ICM_QUERY {
        return ICM_OFF as INT;
    }

    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32k Subsystem
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            win32ss/gdi/ntgdi/icm.c
//  * PURPOSE:         Icm functions
//  * PROGRAMMERS:     ...
//  */
//
// #include <win32k.h>
// #include <safe_user_io.h>
//
// #define NDEBUG
// #include <debug.h>
//
// HCOLORSPACE hStockColorSpace = NULL;
//
//
// HCOLORSPACE
// FASTCALL
// IntGdiCreateColorSpace(
//     PLOGCOLORSPACEEXW pLogColorSpace)
// {
//     PCOLORSPACE pCS;
//     HCOLORSPACE hCS;
//
//     pCS = COLORSPACEOBJ_AllocCSWithHandle();
//     if (pCS == NULL) return NULL;
//
//     hCS = pCS->BaseObject.hHmgr;
//
//     pCS->lcsColorSpace = pLogColorSpace->lcsColorSpace;
//     pCS->dwFlags = pLogColorSpace->dwFlags;
//
//     COLORSPACEOBJ_UnlockCS(pCS);
//     return hCS;
// }
//
// BOOL
// FASTCALL
// IntGdiDeleteColorSpace(
//     HCOLORSPACE hColorSpace)
// {
//     BOOL Ret = FALSE;
//
//     if ((hColorSpace != hStockColorSpace) &&
//         (GDI_HANDLE_GET_TYPE(hColorSpace) == GDILoObjType_LO_ICMLCS_TYPE))
//     {
//         Ret = GreDeleteObject(hColorSpace);
//         if (!Ret) EngSetLastError(ERROR_INVALID_PARAMETER);
//     }
//
//     return Ret;
// }
//
// HANDLE
// __stdcall
// NtGdiCreateColorSpace(
//     IN PLOGCOLORSPACEEXW pLogColorSpace)
// {
//     LOGCOLORSPACEEXW Safelcs;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     Status = SafeProbeForRead(pLogColorSpace, sizeof(LOGCOLORSPACEEXW), 1);
//     if (NT_SUCCESS(Status))
//     {
//         RtlCopyMemory(&Safelcs, pLogColorSpace, sizeof(LOGCOLORSPACEEXW));
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return NULL;
//     }
//
//     return IntGdiCreateColorSpace(&Safelcs);
// }
//
// BOOL
// __stdcall
// NtGdiDeleteColorSpace(
//     IN HANDLE hColorSpace)
// {
//     return IntGdiDeleteColorSpace(hColorSpace);
// }
//
// BOOL
// FASTCALL
// IntGetDeviceGammaRamp(HDEV hPDev, PGAMMARAMP Ramp)
// {
//     PPDEVOBJ pGDev = (PPDEVOBJ) hPDev;
//     int i;
//
//     if (!(pGDev->flFlags & PDEV_DISPLAY)) return FALSE;
//
//     if ((pGDev->devinfo.iDitherFormat == BMF_8BPP)  ||
//         (pGDev->devinfo.iDitherFormat == BMF_16BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_24BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_32BPP))
//     {
//         if (pGDev->flFlags & PDEV_GAMMARAMP_TABLE)
//         {
//             RtlCopyMemory(Ramp, pGDev->pvGammaRamp, sizeof(GAMMARAMP));
//         }
//         else
//         {
//             // Generate the 256-colors array
//             for (i = 0; i < 256; i++ )
//             {
//                 int NewValue = i * 256;
//
//                 Ramp->Red[i] = Ramp->Green[i] = Ramp->Blue[i] = ((WORD)NewValue);
//             }
//         }
//         return TRUE;
//     }
//
//     return FALSE;
// }
//
// BOOL
// __stdcall
// NtGdiGetDeviceGammaRamp(
//     HDC hDC,
//     LPVOID Ramp)
// {
//     BOOL Ret;
//     PDC dc;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PGAMMARAMP SafeRamp;
//
//     if (!Ramp) return FALSE;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     SafeRamp = ExAllocatePoolWithTag(PagedPool, sizeof(GAMMARAMP), GDITAG_ICM);
//     if (!SafeRamp)
//     {
//         DC_UnlockDc(dc);
//         EngSetLastError(STATUS_NO_MEMORY);
//         return FALSE;
//     }
//
//     Ret = IntGetDeviceGammaRamp((HDEV)dc->ppdev, SafeRamp);
//     if (!Ret)
//     {
//         DC_UnlockDc(dc);
//         ExFreePoolWithTag(SafeRamp, GDITAG_ICM);
//         return Ret;
//     }
//
//     Status = SafeProbeForWrite(Ramp, sizeof(GAMMARAMP), 1);
//     if (NT_SUCCESS(Status))
//     {
//         RtlCopyMemory(Ramp, SafeRamp, sizeof(GAMMARAMP));
//     }
//
//     DC_UnlockDc(dc);
//     ExFreePoolWithTag(SafeRamp, GDITAG_ICM);
//
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return FALSE;
//     }
//     return Ret;
// }
//
// BOOL
// __stdcall
// NtGdiSetColorSpace(IN HDC hdc,
//                    IN HCOLORSPACE hColorSpace)
// {
//     PDC pDC;
//     PDC_ATTR pdcattr;
//     PCOLORSPACE pCS;
//
//     pDC = DC_LockDc(hdc);
//     if (!pDC)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//     pdcattr = pDC->pdcattr;
//
//     if (pdcattr->hColorSpace == hColorSpace)
//     {
//         DC_UnlockDc(pDC);
//         return TRUE;
//     }
//
//     pCS = COLORSPACEOBJ_LockCS(hColorSpace);
//     if (!pCS)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     if (pDC->dclevel.pColorSpace)
//     {
//         GDIOBJ_vDereferenceObject((POBJ) pDC->dclevel.pColorSpace);
//     }
//
//     pDC->dclevel.pColorSpace = pCS;
//     pdcattr->hColorSpace = hColorSpace;
//
//     COLORSPACEOBJ_UnlockCS(pCS);
//     DC_UnlockDc(pDC);
//     return TRUE;
// }
//
// BOOL
// FASTCALL
// UpdateDeviceGammaRamp(HDEV hPDev)
// {
//     BOOL Ret = FALSE;
//     PPALETTE palGDI;
//     PALOBJ *palPtr;
//     PPDEVOBJ pGDev = (PPDEVOBJ)hPDev;
//
//     if ((pGDev->devinfo.iDitherFormat == BMF_8BPP)  ||
//         (pGDev->devinfo.iDitherFormat == BMF_16BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_24BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_32BPP))
//     {
//         if (pGDev->DriverFunctions.IcmSetDeviceGammaRamp)
//             return pGDev->DriverFunctions.IcmSetDeviceGammaRamp( pGDev->dhpdev,
//                     IGRF_RGB_256WORDS,
//                     pGDev->pvGammaRamp);
//
//         if ((pGDev->devinfo.iDitherFormat != BMF_8BPP) ||
//             !(pGDev->gdiinfo.flRaster & RC_PALETTE)) return FALSE;
//
//         if (!(pGDev->flFlags & PDEV_GAMMARAMP_TABLE)) return FALSE;
//
//         palGDI = PALETTE_ShareLockPalette(pGDev->devinfo.hpalDefault);
//         if(!palGDI) return FALSE;
//         palPtr = (PALOBJ*) palGDI;
//
//         if (pGDev->flFlags & PDEV_GAMMARAMP_TABLE)
//             palGDI->flFlags |= PAL_GAMMACORRECTION;
//         else
//             palGDI->flFlags &= ~PAL_GAMMACORRECTION;
//
//         if (!(pGDev->flFlags & PDEV_DRIVER_PUNTED_CALL)) // No punting, we hook
//         {
//             // BMF_8BPP only!
//             // PALOBJ_cGetColors check mode flags and update Gamma Correction.
//             // Set the HDEV to pal and go.
//             palGDI->hPDev = hPDev;
//             Ret = pGDev->DriverFunctions.SetPalette(pGDev->dhpdev,
//                                                     palPtr,
//                                                     0,
//                                                     0,
//                                                     palGDI->NumColors);
//         }
//         PALETTE_ShareUnlockPalette(palGDI);
//         return Ret;
//     }
//     else
//         return FALSE;
// }
//
// //
// // ICM registry subkey sets internal brightness range, gamma range is 128 or
// // 256 when icm is init.
// INT IcmGammaRangeSet = 128; // <- Make it global
//
// BOOL
// FASTCALL
// IntSetDeviceGammaRamp(HDEV hPDev, PGAMMARAMP Ramp, BOOL Test)
// {
//     WORD IcmGR, i, R, G, B;
//     BOOL Ret = FALSE, TstPeak;
//     PPDEVOBJ pGDev = (PPDEVOBJ) hPDev;
//
//     if (!hPDev) return FALSE;
//
//     if (!(pGDev->flFlags & PDEV_DISPLAY )) return FALSE;
//
//     if ((pGDev->devinfo.iDitherFormat == BMF_8BPP)  ||
//         (pGDev->devinfo.iDitherFormat == BMF_16BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_24BPP) ||
//         (pGDev->devinfo.iDitherFormat == BMF_32BPP))
//     {
//         if (!pGDev->DriverFunctions.IcmSetDeviceGammaRamp)
//         {
//             // No driver support
//             if (!(pGDev->devinfo.flGraphicsCaps2 & GCAPS2_CHANGEGAMMARAMP))
//             {
//                 // Driver does not support Gamma Ramp, so test to see we
//                 // have BMF_8BPP only and palette operation support.
//                 if ((pGDev->devinfo.iDitherFormat != BMF_8BPP) ||
//                     !(pGDev->gdiinfo.flRaster & RC_PALETTE))  return FALSE;
//             }
//         }
//
//         if (pGDev->flFlags & PDEV_GAMMARAMP_TABLE)
//         {
//             if (RtlCompareMemory(pGDev->pvGammaRamp, Ramp, sizeof(GAMMARAMP)) ==
//                     sizeof(GAMMARAMP)) return TRUE;
//         }
//
//         // Verify Ramp is inside range.
//         IcmGR = -IcmGammaRangeSet;
//         TstPeak = (Test == FALSE);
//         for (i = 0; i < 256; i++)
//         {
//             R = Ramp->Red[i]   / 256;
//             G = Ramp->Green[i] / 256;
//             B = Ramp->Blue[i]  / 256;
//
//             if (R >= IcmGR)
//             {
//                 if (R <= IcmGammaRangeSet + i)
//                 {
//                     if ((G >= IcmGR) &&
//                         (G <= IcmGammaRangeSet + i) &&
//                         (B >= IcmGR) &&
//                         (B <= IcmGammaRangeSet + i) ) continue;
//                 }
//             }
//
//             if (Test) return Ret; // Don't set and return.
//
//             // No test override, check max range
//             if (TstPeak)
//             {
//                 if ((R != (IcmGR * 256)) ||
//                     (G != (IcmGR * 256)) ||
//                     (B != (IcmGR * 256)) ) TstPeak = FALSE; // W/i range.
//             }
//         }
//         // ReactOS allocates a ramp even if it is 8BPP and Palette only.
//         // This way we have a record of the change in memory.
//         if (!pGDev->pvGammaRamp && !(pGDev->flFlags & PDEV_GAMMARAMP_TABLE))
//         {
//             // If the above is true and we have nothing allocated, create it.
//             pGDev->pvGammaRamp = ExAllocatePoolWithTag(PagedPool, sizeof(GAMMARAMP), GDITAG_ICM);
//             pGDev->flFlags |= PDEV_GAMMARAMP_TABLE;
//         }
//
//         if (pGDev->pvGammaRamp)
//             RtlCopyMemory(pGDev->pvGammaRamp, Ramp, sizeof(GAMMARAMP));
//
//         Ret = UpdateDeviceGammaRamp(hPDev);
//     }
//
//     return Ret;
// }
//
// BOOL
// __stdcall
// NtGdiSetDeviceGammaRamp(
//     HDC hDC,
//     LPVOID Ramp)
// {
//     BOOL Ret;
//     PDC dc;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PGAMMARAMP SafeRamp;
//     if (!Ramp) return FALSE;
//
//     dc = DC_LockDc(hDC);
//     if (!dc)
//     {
//         EngSetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     SafeRamp = ExAllocatePoolWithTag(PagedPool, sizeof(GAMMARAMP), GDITAG_ICM);
//     if (!SafeRamp)
//     {
//         DC_UnlockDc(dc);
//         EngSetLastError(STATUS_NO_MEMORY);
//         return FALSE;
//     }
//     Status = SafeProbeForRead(Ramp, sizeof(GAMMARAMP), 1);
//     if (NT_SUCCESS(Status))
//     {
//         RtlCopyMemory(SafeRamp, Ramp, sizeof(GAMMARAMP));
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         DC_UnlockDc(dc);
//         ExFreePoolWithTag(SafeRamp, GDITAG_ICM);
//         SetLastNtError(Status);
//         return FALSE;
//     }
//
//     Ret = IntSetDeviceGammaRamp((HDEV)dc->ppdev, SafeRamp, TRUE);
//     DC_UnlockDc(dc);
//     ExFreePoolWithTag(SafeRamp, GDITAG_ICM);
//     return Ret;
// }
//
// INT
// __stdcall
// NtGdiSetIcmMode(HDC  hDC,
//                 ULONG nCommand,
//                 ULONG EnableICM) // ulMode
// {
//     /* FIXME: This should be coded someday  */
//     if (EnableICM == ICM_OFF)
//     {
//         return  ICM_OFF;
//     }
//     if (EnableICM == ICM_ON)
//     {
//         return  0;
//     }
//     if (EnableICM == ICM_QUERY)
//     {
//         return  ICM_OFF;
//     }
//
//     return  0;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
