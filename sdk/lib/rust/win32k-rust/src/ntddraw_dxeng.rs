use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, PsGetCurrentThread};
use ntkrnl_sys::interlocked::InterlockedIncrement;

// ---- file-scope preamble (translated) ----
pub static mut ghsemShareDevLock: HSEMAPHORE = core::ptr::null_mut();
pub static mut gcEngFuncs: ULONG = DXENG_INDEX_DxEngLoadImage + 1;
pub static mut gaEngFuncs: [DRVFN; 43] = [
    DRVFN { iFunc: 0, pfn: None },
    DRVFN { iFunc: DXENG_INDEX_DxEngNUIsTermSrv, pfn: Some(unsafe { core::mem::transmute(DxEngNUIsTermSrv as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngScreenAccessCheck, pfn: Some(unsafe { core::mem::transmute(DxEngScreenAccessCheck as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngRedrawDesktop, pfn: Some(unsafe { core::mem::transmute(DxEngRedrawDesktop as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngDispUniq, pfn: Some(unsafe { core::mem::transmute(DxEngDispUniq as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngIncDispUniq, pfn: Some(unsafe { core::mem::transmute(DxEngIncDispUniq as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngVisRgnUniq, pfn: Some(unsafe { core::mem::transmute(DxEngVisRgnUniq as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngLockShareSem, pfn: Some(unsafe { core::mem::transmute(DxEngLockShareSem as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngUnlockShareSem, pfn: Some(unsafe { core::mem::transmute(DxEngUnlockShareSem as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngEnumerateHdev, pfn: Some(unsafe { core::mem::transmute(DxEngEnumerateHdev as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngLockHdev, pfn: Some(unsafe { core::mem::transmute(DxEngLockHdev as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngUnlockHdev, pfn: Some(unsafe { core::mem::transmute(DxEngUnlockHdev as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngIsHdevLockedByCurrentThread, pfn: Some(unsafe { core::mem::transmute(DxEngIsHdevLockedByCurrentThread as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngReferenceHdev, pfn: Some(unsafe { core::mem::transmute(DxEngReferenceHdev as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngUnreferenceHdev, pfn: Some(unsafe { core::mem::transmute(DxEngUnreferenceHdev as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetDeviceGammaRamp, pfn: Some(unsafe { core::mem::transmute(DxEngGetDeviceGammaRamp as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetDeviceGammaRamp, pfn: Some(unsafe { core::mem::transmute(DxEngSetDeviceGammaRamp as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSpTearDownSprites, pfn: Some(unsafe { core::mem::transmute(DxEngSpTearDownSprites as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSpUnTearDownSprites, pfn: Some(unsafe { core::mem::transmute(DxEngSpUnTearDownSprites as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSpSpritesVisible, pfn: Some(unsafe { core::mem::transmute(DxEngSpSpritesVisible as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetHdevData, pfn: Some(unsafe { core::mem::transmute(DxEngGetHdevData as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetHdevData, pfn: Some(unsafe { core::mem::transmute(DxEngSetHdevData as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngCreateMemoryDC, pfn: Some(unsafe { core::mem::transmute(DxEngCreateMemoryDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetDesktopDC, pfn: Some(unsafe { core::mem::transmute(DxEngGetDesktopDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngDeleteDC, pfn: Some(unsafe { core::mem::transmute(DxEngDeleteDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngCleanDC, pfn: Some(unsafe { core::mem::transmute(DxEngCleanDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetDCOwner, pfn: Some(unsafe { core::mem::transmute(DxEngSetDCOwner as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngLockDC, pfn: Some(unsafe { core::mem::transmute(DxEngLockDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngUnlockDC, pfn: Some(unsafe { core::mem::transmute(DxEngUnlockDC as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetDCState, pfn: Some(unsafe { core::mem::transmute(DxEngSetDCState as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetDCState, pfn: Some(unsafe { core::mem::transmute(DxEngGetDCState as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSelectBitmap, pfn: Some(unsafe { core::mem::transmute(DxEngSelectBitmap as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetBitmapOwner, pfn: Some(unsafe { core::mem::transmute(DxEngSetBitmapOwner as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngDeleteSurface, pfn: Some(unsafe { core::mem::transmute(DxEngDeleteSurface as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetSurfaceData, pfn: Some(unsafe { core::mem::transmute(DxEngGetSurfaceData as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngAltLockSurface, pfn: Some(unsafe { core::mem::transmute(DxEngAltLockSurface as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngUploadPaletteEntryToSurface, pfn: Some(unsafe { core::mem::transmute(DxEngUploadPaletteEntryToSurface as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngMarkSurfaceAsDirectDraw, pfn: Some(unsafe { core::mem::transmute(DxEngMarkSurfaceAsDirectDraw as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSelectPaletteToSurface, pfn: Some(unsafe { core::mem::transmute(DxEngSelectPaletteToSurface as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSyncPaletteTableWithDevice, pfn: Some(unsafe { core::mem::transmute(DxEngSyncPaletteTableWithDevice as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngSetPaletteState, pfn: Some(unsafe { core::mem::transmute(DxEngSetPaletteState as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngGetRedirectionBitmap, pfn: Some(unsafe { core::mem::transmute(DxEngGetRedirectionBitmap as *const ()) }) },
    DRVFN { iFunc: DXENG_INDEX_DxEngLoadImage, pfn: Some(unsafe { core::mem::transmute(DxEngLoadImage as *const ()) }) },
];
pub static mut gulVisRgnUniqueness: ULONG = 0;

#[no_mangle]
pub unsafe extern "system" fn DxEngDispUniq() -> ULONG {
    (*GdiHandleTable).flDeviceUniq
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP) -> i32 {
    IntGetDeviceGammaRamp(hPDev, Ramp)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngLockDC(hDC: HDC) -> PDC {
    DC_LockDc(hDC)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUnlockDC(pDC: PDC) -> u8 {
    DC_UnlockDc(pDC);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngLockShareSem() -> u8 {
    if ghsemShareDevLock.is_null() {
        ghsemShareDevLock = EngCreateSemaphore();
    }
    EngAcquireSemaphore(ghsemShareDevLock);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUnlockShareSem() -> u8 {
    EngReleaseSemaphore(ghsemShareDevLock);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetDeviceGammaRamp(
    hPDev: HDEV,
    Ramp: PGAMMARAMP,
    Test: i32,
) -> i32 {
    IntSetDeviceGammaRamp(hPDev, Ramp, Test)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetHdevData(
    hDev: HDEV,
    Type: DWORD_PTR,
) -> DWORD_PTR {
    let mut retVal: DWORD_PTR = 0;
    let PDev: PPDEVOBJ = hDev as PPDEVOBJ;

    match Type as DXEGSHDEVDATA {
        DxEGShDevData_Surface => {
            retVal = (*PDev).pSurface as DWORD_PTR;
        }
        DxEGShDevData_hSpooler => {
            retVal = (*PDev).hSpooler as DWORD_PTR;
        }
        DxEGShDevData_DitherFmt => {
            retVal = (*PDev).devinfo.iDitherFormat as DWORD_PTR;
        }
        DxEGShDevData_FxCaps => {
            retVal = (*PDev).devinfo.flGraphicsCaps as DWORD_PTR;
        }
        DxEGShDevData_FxCaps2 => {
            retVal = (*PDev).devinfo.flGraphicsCaps2 as DWORD_PTR;
        }
        DxEGShDevData_DrvFuncs => {
            retVal = core::ptr::addr_of_mut!((*PDev).__bindgen_anon_1.DriverFunctions) as DWORD_PTR;
        }
        DxEGShDevData_dhpdev => {
            retVal = (*PDev).dhpdev as DWORD_PTR;
        }
        DxEGShDevData_eddg => {
            retVal = (*PDev).pEDDgpl as DWORD_PTR;
        }
        DxEGShDevData_dd_locks => {
            retVal = (*PDev).cDirectDrawDisableLocks as DWORD_PTR;
        }
        DxEGShDevData_dd_flags => {
            retVal = (*PDev).DxDd_Flags as DWORD_PTR;
        }
        DxEGShDevData_disable => {
            retVal = ((*PDev).flFlags & PDEV_DISABLED) as DWORD_PTR;
        }
        DxEGShDevData_metadev => {
            retVal = ((*PDev).flFlags & PDEV_META_DEVICE) as DWORD_PTR;
        }
        DxEGShDevData_display => {
            retVal = ((*PDev).flFlags & PDEV_DISPLAY) as DWORD_PTR;
        }
        DxEGShDevData_Parent => {
            retVal = (*PDev).ppdevParent as DWORD_PTR;
        }
        DxEGShDevData_OpenRefs => {
            retVal = ((*PDev).cPdevOpenRefs != 0) as DWORD_PTR;
        }
        DxEGShDevData_palette => {
            retVal = ((*PDev).gdiinfo.flRaster & RC_PALETTE) as DWORD_PTR;
        }
        DxEGShDevData_ldev => {
            retVal = (*PDev).pldev as DWORD_PTR;
        }
        DxEGShDevData_GDev => {
            retVal = (*PDev).pGraphicsDevice as DWORD_PTR;
        }
        DxEGShDevData_clonedev => {
            retVal = ((*PDev).flFlags & PDEV_CLONE_DEVICE) as DWORD_PTR;
        }
        _ => {}
    }

    retVal
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetHdevData(
    hDev: HDEV,
    Type: DXEGSHDEVDATA,
    Data: DWORD_PTR,
) -> BOOLEAN {
    let mut retVal: BOOLEAN = 0u8;

    if Type == DxEGShDevData_dd_locks {
        (*(hDev as PPDEVOBJ)).cDirectDrawDisableLocks = Data as ULONG;
        retVal = 1u8;
    }
    return retVal;
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetDCState(
    hDC: HDC,
    r#type: u32,
) -> DWORD_PTR {
    let pDC = DC_LockDc(hDC);
    let mut retVal: DWORD_PTR = 0;

    if !pDC.is_null() {
        match r#type {
            1 => {
                retVal = ((*pDC).fs & DC_FULLSCREEN as i32) as DWORD_PTR;
            }
            2 => {
                retVal = REGION_Complexity((*pDC).prgnVis) as DWORD_PTR;
            }
            3 => {
                retVal = (*pDC).ppdev as DWORD_PTR;
            }
            _ => {}
        }
        DC_UnlockDc(pDC);
    }

    retVal
}

#[no_mangle]
pub unsafe extern "system" fn DxEngIncDispUniq() -> u8 {
    InterlockedIncrement(core::ptr::addr_of_mut!((*GdiHandleTable).flDeviceUniq) as *mut i32);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngLockHdev(hDev: HDEV) -> u8 {
    let ppdev = hDev as PPDEVOBJ;
    let Resource: PERESOURCE;

    Resource = (*ppdev).hsemDevLock as PERESOURCE;

    if !Resource.is_null() {
        KeEnterCriticalRegion();
        ExAcquireResourceExclusiveLite(Resource.cast(), 1u8);
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUnlockHdev(hDev: HDEV) -> u8 {
    let ppdev = hDev as PPDEVOBJ;
    let Resource = (*ppdev).hsemDevLock as PERESOURCE;

    if !Resource.is_null() {
        ExReleaseResourceLite(Resource.cast());
        KeLeaveCriticalRegion();
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngReferenceHdev(hDev: HDEV) -> u8 {
    PDEVOBJ_vReference(hDev as PPDEVOBJ);
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngNUIsTermSrv() -> u8 {
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngRedrawDesktop() -> u8 {
    UserRedrawDesktop();
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngVisRgnUniq() -> ULONG {
    gulVisRgnUniqueness
}

#[no_mangle]
pub unsafe extern "system" fn DxEngCreateMemoryDC(hDev: HDEV) -> HDC {
    IntGdiCreateDisplayDC(hDev, DCTYPE_MEMORY, 0)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngScreenAccessCheck() -> DWORD_PTR {
    1
}

#[no_mangle]
pub unsafe extern "system" fn DxEngIsHdevLockedByCurrentThread(hDev: HDEV) -> u8 {
    let pSem: PERESOURCE = (*((hDev as PPDEVOBJ))).hsemDevLock as PERESOURCE;
    if (*pSem).OwnerEntry.OwnerThread == PsGetCurrentThread() as ERESOURCE_THREAD {
        1u8
    } else {
        0u8
    }
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUnreferenceHdev(hDev: HDEV) -> u32 {
    PDEVOBJ_vRelease(hDev as PPDEVOBJ);
    1
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetDesktopDC(
    DcType: u32,
    EmptyDC: i32,
    ValidatehWnd: i32,
) -> HDC {
    UserGetDesktopDC(DcType, EmptyDC, ValidatehWnd)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngDeleteDC(hdc: HDC, Force: i32) -> u8 {
    IntGdiDeleteDC(hdc, Force) as u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngCleanDC(hdc: HDC) -> u8 {
    IntGdiCleanDC(hdc) as u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetDCOwner(hObject: HGDIOBJ, OwnerMask: DWORD_PTR) -> BOOL {
    GreSetDCOwner(hObject.cast(), OwnerMask as ULONG)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetDCState(hDC: HDC, SetType: DWORD_PTR, Set: DWORD_PTR) -> u8 {
    let mut Ret: u8 = 0u8;
    let pDC: PDC = DC_LockDc(hDC);

    if !pDC.is_null() {
        if SetType == 1 {
            if Set != 0 {
                (*pDC).fs |= DC_FULLSCREEN as i32;
            } else {
                (*pDC).fs &= !(DC_FULLSCREEN as i32);
            }
            Ret = 1u8;
        }
        DC_UnlockDc(pDC);
        return Ret;
    }
    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSelectBitmap(hdc: HDC, hbmp: HBITMAP) -> HBITMAP {
    NtGdiSelectBitmap(hdc, hbmp)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetBitmapOwner(hbmp: HBITMAP, ulOwner: ULONG) -> BOOLEAN {
    GreSetBitmapOwner(hbmp, ulOwner) as u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngDeleteSurface(hsurf: HSURF) -> u8 {
    EngDeleteSurface(hsurf) as u8
}

// Hand-completed: the per-function scanner skipped these two (pointer return
// types HDEV*/SURFOBJ* tripped detection); bodies transcribed from dxeng.c.
#[no_mangle]
pub unsafe extern "system" fn DxEngEnumerateHdev(_hdev: *mut HDEV) -> *mut HDEV {
    /* FIXME: Enumerate all drivers in win32k (UNIMPLEMENTED in C) */
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn DxEngAltLockSurface(hsurf: HSURF) -> *mut SURFOBJ {
    EngLockSurface(hsurf)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetSurfaceData(x1: DWORD_PTR, x2: DWORD_PTR) -> DWORD_PTR {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUploadPaletteEntryToSurface(
    x1: DWORD,
    x2: DWORD,
    x3: DWORD,
    x4: DWORD,
) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngMarkSurfaceAsDirectDraw(x1: DWORD, x2: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSelectPaletteToSurface(x1: DWORD, x2: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSyncPaletteTableWithDevice(x1: DWORD, x2: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetPaletteState(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetRedirectionBitmap(x1: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngLoadImage(x1: DWORD, x2: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSpTearDownSprites(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSpUnTearDownSprites(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSpSpritesVisible(x1: u32) -> u32 {
    0
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:          ReactOS Win32 Subsystem
//  * LICENSE:          GPL - See COPYING in the top level directory
//  * FILE:             win32ss/reactx/ntddraw/dxeng.c
//  * PURPOSE:          Implementation of DxEng functions
//  * PROGRAMMERS:      Magnus Olsen (magnus@greatlord.com)
//  *                   Oleg Dubinskiy (oleg.dubinskij30@gmail.com)
//  */
//
// #include <win32k.h>
//
// // #define NDEBUG
// #include <debug.h>
//
// HSEMAPHORE  ghsemShareDevLock = NULL;
//
// ULONG gcEngFuncs = DXENG_INDEX_DxEngLoadImage + 1;
// DRVFN gaEngFuncs[] =
// {
//     {0, (PFN) NULL},
//     {DXENG_INDEX_DxEngNUIsTermSrv, (PFN)DxEngNUIsTermSrv},
//     {DXENG_INDEX_DxEngScreenAccessCheck, (PFN)DxEngScreenAccessCheck},
//     {DXENG_INDEX_DxEngRedrawDesktop, (PFN)DxEngRedrawDesktop},
//     {DXENG_INDEX_DxEngDispUniq, (PFN)DxEngDispUniq},
//     {DXENG_INDEX_DxEngIncDispUniq, (PFN)DxEngIncDispUniq},
//     {DXENG_INDEX_DxEngVisRgnUniq, (PFN)DxEngVisRgnUniq},
//     {DXENG_INDEX_DxEngLockShareSem, (PFN)DxEngLockShareSem},
//     {DXENG_INDEX_DxEngUnlockShareSem, (PFN)DxEngUnlockShareSem},
//     {DXENG_INDEX_DxEngEnumerateHdev, (PFN)DxEngEnumerateHdev},
//     {DXENG_INDEX_DxEngLockHdev, (PFN)DxEngLockHdev},
//     {DXENG_INDEX_DxEngUnlockHdev, (PFN)DxEngUnlockHdev},
//     {DXENG_INDEX_DxEngIsHdevLockedByCurrentThread, (PFN)DxEngIsHdevLockedByCurrentThread},
//     {DXENG_INDEX_DxEngReferenceHdev, (PFN)DxEngReferenceHdev},
//     {DXENG_INDEX_DxEngUnreferenceHdev, (PFN)DxEngUnreferenceHdev},
//     {DXENG_INDEX_DxEngGetDeviceGammaRamp, (PFN)DxEngGetDeviceGammaRamp},
//     {DXENG_INDEX_DxEngSetDeviceGammaRamp, (PFN)DxEngSetDeviceGammaRamp},
//     {DXENG_INDEX_DxEngSpTearDownSprites, (PFN)DxEngSpTearDownSprites},
//     {DXENG_INDEX_DxEngSpUnTearDownSprites, (PFN)DxEngSpUnTearDownSprites},
//     {DXENG_INDEX_DxEngSpSpritesVisible, (PFN)DxEngSpSpritesVisible},
//     {DXENG_INDEX_DxEngGetHdevData, (PFN)DxEngGetHdevData},
//     {DXENG_INDEX_DxEngSetHdevData, (PFN)DxEngSetHdevData},
//     {DXENG_INDEX_DxEngCreateMemoryDC, (PFN)DxEngCreateMemoryDC},
//     {DXENG_INDEX_DxEngGetDesktopDC, (PFN)DxEngGetDesktopDC},
//     {DXENG_INDEX_DxEngDeleteDC, (PFN)DxEngDeleteDC},
//     {DXENG_INDEX_DxEngCleanDC, (PFN)DxEngCleanDC},
//     {DXENG_INDEX_DxEngSetDCOwner, (PFN)DxEngSetDCOwner},
//     {DXENG_INDEX_DxEngLockDC, (PFN)DxEngLockDC},
//     {DXENG_INDEX_DxEngUnlockDC, (PFN)DxEngUnlockDC},
//     {DXENG_INDEX_DxEngSetDCState, (PFN)DxEngSetDCState},
//     {DXENG_INDEX_DxEngGetDCState, (PFN)DxEngGetDCState},
//     {DXENG_INDEX_DxEngSelectBitmap, (PFN)DxEngSelectBitmap},
//     {DXENG_INDEX_DxEngSetBitmapOwner, (PFN)DxEngSetBitmapOwner},
//     {DXENG_INDEX_DxEngDeleteSurface, (PFN)DxEngDeleteSurface},
//     {DXENG_INDEX_DxEngGetSurfaceData, (PFN)DxEngGetSurfaceData},
//     {DXENG_INDEX_DxEngAltLockSurface, (PFN)DxEngAltLockSurface},
//     {DXENG_INDEX_DxEngUploadPaletteEntryToSurface, (PFN)DxEngUploadPaletteEntryToSurface},
//     {DXENG_INDEX_DxEngMarkSurfaceAsDirectDraw, (PFN)DxEngMarkSurfaceAsDirectDraw},
//     {DXENG_INDEX_DxEngSelectPaletteToSurface, (PFN)DxEngSelectPaletteToSurface},
//     {DXENG_INDEX_DxEngSyncPaletteTableWithDevice, (PFN)DxEngSyncPaletteTableWithDevice},
//     {DXENG_INDEX_DxEngSetPaletteState, (PFN)DxEngSetPaletteState},
//     {DXENG_INDEX_DxEngGetRedirectionBitmap, (PFN)DxEngGetRedirectionBitmap},
//     {DXENG_INDEX_DxEngLoadImage, (PFN)DxEngLoadImage}
// };
//
// /*++
// * @name DxEngDispUniq
// * @implemented
// *
// * The function DxEngDispUniq returns the DisplayUniqVisrgn counter value from GDI shared memory
// *
// * @return
// * Returns the DisplayUniqVisrgn counter value from GDI shared memory
// *
// * @remarks.
// * none
// *
// *--*/
// ULONG
// __stdcall
// DxEngDispUniq(VOID)
// {
//     DPRINT("ReactX Calling : DxEngDispUniq\n");
//     return GdiHandleTable->flDeviceUniq;
// }
//
// /*++
// * @name DxEngGetDeviceGammaRamp
// * @implemented
// *
// * The function DxEngGetDeviceGammaRamp gets the gamma ramp to dxg.sys.
//
// * @param HDEV hPDev
// * The hdev.
// *
// * @param PGAMMARAMP Ramp
// * Pointer to store the gamma ramp value in.
// *
// * @return
// *Returns TRUE for success, FALSE for failure
// *
// * @remarks.
// * None
// *
// *--*/
// BOOL
// __stdcall
// DxEngGetDeviceGammaRamp(HDEV hPDev, PGAMMARAMP Ramp)
// {
//     DPRINT("ReactX Calling : DxEngGetDeviceGammaRamp\n");
//     return IntGetDeviceGammaRamp(hPDev, Ramp);
// }
//
// /*++
// * @name DxEngLockDC
// * @implemented
// *
// * The function DxEngLockDC locks a hdc from dxg.sys
// *
// * @param HDC hDC
// * The handle we want to lock
// *
// * @return
// * Returns PDC if lock succeeded or NULL if it failed.
// *
// * @remarks.
// * none
// *
// *--*/
// PDC
// __stdcall
// DxEngLockDC(HDC hDC)
// {
//     DPRINT("ReactX Calling : DxEngLockDC\n");
//     return DC_LockDc(hDC);
// }
//
// /*++
// * @name DxEngUnlockDC
// * @implemented
// *
// * The function DxEngUnlockDC unlocks a pDC (hdc) from dxg.sys.
//
// * @param PDC pDC
// * The handle we want to unlock.
// *
// * @return
// * This function returns TRUE no matter what.
// *
// * @remarks.
// * none
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngUnlockDC(PDC pDC)
// {
//     DPRINT("ReactX Calling : DxEngUnlockDC\n");
//     DC_UnlockDc(pDC);
//     return TRUE;
// }
//
// /*++
// * @name DxEngLockShareSem
// * @implemented
// *
// * The function DxEngLockShareSem locks a struct of type ghsemShareDevLock that can be shared.
// *
// * @return
// * This function returns TRUE for success and FALSE for failure.
// * FALSE must mean the struct has already been locked.
// *
// * @remarks.
// * It is being used in various ntuser* functions and ntgdi*
// * ReactOS specific: It is not in use yet?
// *SystemResourcesList
// *--*/
// BOOLEAN
// __stdcall
// DxEngLockShareSem(VOID)
// {
//     DPRINT("ReactX Calling : DxEngLockShareSem\n");
//     if(!ghsemShareDevLock) ghsemShareDevLock = EngCreateSemaphore(); // Hax, should be in dllmain.c
//     EngAcquireSemaphore(ghsemShareDevLock);
//     return TRUE;
// }
//
// /*++
// * @name DxEngUnlockShareSem
// * @implemented
// *
// * The function DxEngUnlockShareSem unlocks the struct of type ghsemShareDevLock.
// *
// * @return
// * This function returns TRUE no matter what.
// *
// * @remarks.
// * ReactOS specific: It is not in use yet?
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngUnlockShareSem(VOID)
// {
//     DPRINT("ReactX Calling : DxEngUnlockShareSem\n");
//     EngReleaseSemaphore(ghsemShareDevLock);
//     return TRUE;
// }
//
// /*++
// * @name DxEngSetDeviceGammaRamp
// * @implemented
// *
// * The function DxEngSetDeviceGammaRamp sets gamma ramp from dxg.sys
//
// * @param HDEV hPDev
// * The hdev
// *
// * @param PGAMMARAMP Ramp
// * Value to change gamma ramp to.
// *
// * @param BOOL Test
// * Whether gamma should be tested. TRUE to test, FALSE to not test.
// *
// * @return
// *Returns TRUE for success, FALSE for failure.
// *
// * @remarks.
// * None
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngSetDeviceGammaRamp(HDEV hPDev, PGAMMARAMP Ramp, BOOL Test)
// {
//     DPRINT("ReactX Calling : DxEngSetDeviceGammaRamp\n");
//     return IntSetDeviceGammaRamp(hPDev, Ramp, Test);
// }
//
// /*++
// * @name DxEngGetHdevData
// * @implemented
// *
// * The function DxEngGetHdevData retrieves a value from the HDEV
//
// * @param HDEV hPDev
// * The HDEV
// *
// * @param DXEGSHDEVDATA Type
// * The following typs are supported
// * Type                                            Purpose
// * DxEGShDevData_Surface      Retrieve pointer to Surface handle.
// * DxEGShDevData_hSpooler     Device object of graphics driver.
// * DxEGShDevData_DitherFmt    Retrieve the device iDitherFormat
// * DxEGShDevData_FxCaps       Retrieve the device flGraphicsCaps
// * DxEGShDevData_FxCaps2      Retrieve the device flGraphicsCaps2
// * DxEGShDevData_DrvFuncs     Retrieve the device DriverFunctions function table
// * DxEGShDevData_dhpdev       Retrieve the device hPDev, the real DHPDEV
// * DxEGShDevData_eddg         Retrieve the device pEDDgpl
// * DxEGShDevData_dd_nCount    Retrieve the device DxDd_nCount
// * DxEGShDevData_dd_flags     Retrieve the device DxDd_Flags
// * DxEGShDevData_disable      See if the device pdev is disabled
// * DxEGShDevData_metadev      See if the device pdev is a meta device
// * DxEGShDevData_display      See if the device is the primary display driver
// * DxEGShDevData_Parent       Retrieve the ppdevParent
// * DxEGShDevData_OpenRefs     Retrieve the pdevOpenRefs counter
// * DxEGShDevData_palette      See if the device RC_PALETTE is set
// * DxEGShDevData_ldev         ATM we do not support the Loader Device driver structure
// * DxEGShDevData_GDev         Retrieve the device pGraphicsDevice
// * DxEGShDevData_clonedev     Retrieve the device PDEV_CLONE_DEVICE flag is set or not
// *
// * @return
// * Returns the data we requested
// *
// * @remarks.
// * ReactOS specific: Implementation is incomplete, I do not save the value into the hdev yet.
// *
// *--*/
// DWORD_PTR
// __stdcall
// DxEngGetHdevData(HDEV hDev,
//                  DXEGSHDEVDATA Type)
// {
//     DWORD_PTR retVal = 0;
//     PPDEVOBJ PDev = (PPDEVOBJ)hDev;
//
//     DPRINT("ReactX Calling : DxEngGetHdevData, Type: %d\n", Type);
//
//     DPRINT("hDev: %p\n", hDev);
//
//     switch ( Type )
//     {
//       case DxEGShDevData_Surface:
//         DPRINT("requested DxEGShDevData_Surface\n");
//         retVal = (DWORD_PTR) PDev->pSurface; // ptr to Surface handle.
//         break;
//       case DxEGShDevData_hSpooler:
//         DPRINT("requested DxEGShDevData_hSpooler\n");
//         retVal = (DWORD_PTR) PDev->hSpooler;
//         break;
//       case DxEGShDevData_DitherFmt:
//         DPRINT("requested DxEGShDevData_DitherFmt\n");
//         retVal = (DWORD_PTR) PDev->devinfo.iDitherFormat;
//         break;
//       case DxEGShDevData_FxCaps:
//         DPRINT("requested DxEGShDevData_FxCaps\n");
//         retVal = (DWORD_PTR) PDev->devinfo.flGraphicsCaps;
//         break;
//       case DxEGShDevData_FxCaps2:
//         DPRINT("requested DxEGShDevData_FxCaps2\n");
//         retVal = (DWORD_PTR) PDev->devinfo.flGraphicsCaps2;
//         break;
//       case DxEGShDevData_DrvFuncs:
//         DPRINT("requested DxEGShDevData_DrvFuncs\n");
//         retVal = (DWORD_PTR) &PDev->DriverFunctions;
//         break;
//       case DxEGShDevData_dhpdev:
//         DPRINT("requested DxEGShDevData_dhpdev\n");
//         retVal = (DWORD_PTR) PDev->dhpdev; // DHPDEV
//         break;
//       case DxEGShDevData_eddg:
//         DPRINT("requested DxEGShDevData_eddg\n");
//         retVal = (DWORD_PTR) PDev->pEDDgpl;
//         break;
//       case DxEGShDevData_dd_locks:
//         DPRINT("requested DxEGShDevData_dd_locks\n");
//         retVal = (DWORD_PTR) PDev->cDirectDrawDisableLocks;
//         break;
//       case DxEGShDevData_dd_flags:
//         DPRINT("requested DxEGShDevData_dd_flags\n");
//         retVal = (DWORD_PTR) PDev->DxDd_Flags;
//         break;
//       case DxEGShDevData_disable:
//         DPRINT("requested DxEGShDevData_disable\n");
//         retVal = (DWORD_PTR) PDev->flFlags & PDEV_DISABLED;
//         break;
//       case DxEGShDevData_metadev:
//         DPRINT("requested DxEGShDevData_metadev\n");
//         retVal = (DWORD_PTR) PDev->flFlags & PDEV_META_DEVICE;
//         break;
//       case DxEGShDevData_display:
//         DPRINT("requested DxEGShDevData_display\n");
//         retVal = (DWORD_PTR) PDev->flFlags & PDEV_DISPLAY;
//         break;
//       case DxEGShDevData_Parent:
//         DPRINT("requested DxEGShDevData_Parent\n");
//         retVal = (DWORD_PTR) PDev->ppdevParent;
//         break;
//       case DxEGShDevData_OpenRefs:
//         DPRINT("requested DxEGShDevData_OpenRefs\n");
//         retVal = (DWORD_PTR) PDev->cPdevOpenRefs != 0;
//         break;
//       case DxEGShDevData_palette:
//         DPRINT("requested DxEGShDevData_palette\n");
//         retVal = (DWORD_PTR) PDev->gdiinfo.flRaster & RC_PALETTE;
//         break;
//       case DxEGShDevData_ldev:
//         DPRINT("requested DxEGShDevData_ldev\n");
//         retVal = (DWORD_PTR) PDev->pldev;
//         break;
//       case DxEGShDevData_GDev:
//         DPRINT("requested DxEGShDevData_GDev\n");
//         retVal = (DWORD_PTR) PDev->pGraphicsDevice; // P"GRAPHICS_DEVICE"
//         break;
//       case DxEGShDevData_clonedev:
//         DPRINT("requested DxEGShDevData_clonedev\n");
//         retVal = (DWORD_PTR) PDev->flFlags & PDEV_CLONE_DEVICE;
//         break;
//       default:
//         // TODO: Make this default a bit more useful.
//         break;
//     }
//
//     DPRINT("retVal: %08Ix\n", retVal);
//
//     return retVal;
// }
//
// /*++
// * @name DxEngSetHdevData
// * @implemented
// *
// * The function DxEngSetHdevData sets a value in hdev
//
// * @param HDEV hPDev
// * The hdev
// *
// * @param DXEGSHDEVDATA Type
// * Supports only DxEGShDevData_dd_nCount. It is an internal counter on how many times hdev has been locked and unlocked
// *
// * @param DWORD Data
// * The value to be saved to hdev's internal counter.
// *
// * @return
// * Returns TRUE for success, FALSE for failure
// *
// * @remarks.
// * none
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngSetHdevData(HDEV hDev,
//                  DXEGSHDEVDATA Type,
//                  DWORD_PTR Data)
// {
//     BOOLEAN retVal = FALSE; // Default, no set.
//
//     DPRINT("ReactX Calling : DxEngSetHdevData, Type: %d\n", Type);
//
//     if (Type == DxEGShDevData_dd_locks)
//     {
//         DPRINT("Assigning value: %Iu\n", Data);
//         ((PPDEVOBJ)hDev)->cDirectDrawDisableLocks = Data;
//         retVal = TRUE; // Set
//     }
//     return retVal;
// }
//
// /*++
// * @name DxEngGetDCState
// * @implemented
// *
// * The function DxEngGetDCState is capable of returning three
// * DC states depending on what value is passed in its second parameter:
// * 1. If the DC is full screen
// * 2. Get Complexity of visible region
// * 3. Get Driver hdev, which is ppdev
// *
// * @param HDC hdc
// * The DC handle
// *
// * @param DWORD type
// * value 1 = Is DC fullscreen
// * value 2 = Get Complexity of visible region.
// * value 3 = Get Driver hdev, which is a ppdev.
// *
// * @return
// * Return one of the type values
// *
// * @remarks.
// * none
// *
// *--*/
// DWORD_PTR
// __stdcall
// DxEngGetDCState(HDC hDC,
//                 DWORD type)
// {
//     PDC pDC = DC_LockDc(hDC);
//     DWORD_PTR retVal = 0;
//
//     DPRINT("ReactX Calling : DxEngGetDCState, type: %lu\n", type);
//
//     if (pDC)
//     {
//         switch (type)
//         {
//             case 1:
//                 retVal = (DWORD_PTR) pDC->fs & DC_FULLSCREEN;
//                 break;
//             case 2:
//                 /* Return the complexity of the visible region. */
//                 retVal = (DWORD_PTR) REGION_Complexity(pDC->prgnVis);
//                 break;
//             case 3:
//             {
//                 /* Return the HDEV of this DC. */
//                 retVal = (DWORD_PTR) pDC->ppdev;
//                 break;
//             }
//             default:
//                 /* If a valid type is not found, zero is returned */
//                 DPRINT1("Warning: did not find type %lu\n", type);
//                 break;
//         }
//         DC_UnlockDc(pDC);
//     }
//
//     DPRINT("retVal: %08Ix\n", retVal);
//
//     return retVal;
// }
//
// /*++
// * @name DxEngIncDispUniq
// * @implemented
// *
// * The function DxEngIncDispUniq increments the DisplayUniqVisrgn counter from GDI shared memory.
// *
// * @return
// * This function returns TRUE no matter what.
// *
// * @remarks.
// * none
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngIncDispUniq(VOID)
// {
//     DPRINT("ReactX Calling : DxEngIncDispUniq\n");
//
//     InterlockedIncrement((LONG*)&GdiHandleTable->flDeviceUniq);
//     return TRUE;
// }
//
// /*++
// * @name DxEngLockHdev
// * @implemented
// *
// * The function DxEngLockHdev lock the internal PDEV
// *
// * @param HDEV type
// * it is a pointer to win32k internal pdev struct known as PPDEVOBJ
//
// * @return
// * This function returns TRUE no matter what.
// *
// * @remarks.
// * none
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngLockHdev(HDEV hDev)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hDev;
//     PERESOURCE Resource;
//
//     DPRINT("ReactX Calling : DxEngLockHdev\n");
//
//     DPRINT("hDev: %p\n", hDev);
//
//     Resource = (PERESOURCE)ppdev->hsemDevLock;
//
//     if (Resource)
//     {
//         KeEnterCriticalRegion();
//         ExAcquireResourceExclusiveLite( Resource , TRUE); // Lock monitor.
//     }
//     return TRUE;
// }
//
// /*++
// * @name DxEngUnlockHdev
// * @implemented
// *
// * The function DxEngUnlockHdev unlock the internal PDEV
// *
// * @param HDEV type
// * it is a pointer to win32k internal pdev struct known as PPDEVOBJ
//
// * @return
// * This function returns TRUE no matter what.
// *
// * @remarks.
// * none
// *
// *--*/
// BOOLEAN
// __stdcall
// DxEngUnlockHdev(HDEV hDev)
// {
//     PPDEVOBJ ppdev = (PPDEVOBJ)hDev;
//     PERESOURCE Resource = (PERESOURCE)ppdev->hsemDevLock;
//
//     DPRINT("ReactX Calling : DxEngUnlockHdev\n");
//
//     if (Resource)
//     {
//         ExReleaseResourceLite( Resource );
//         KeLeaveCriticalRegion();
//     }
//     return TRUE;
// }
//
// /************************************************************************/
// /* DxEngReferenceHdev                                                   */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngReferenceHdev(HDEV hDev)
// {
//     PDEVOBJ_vReference((PPDEVOBJ)hDev);
//     /* ALWAYS return true */
//     return TRUE;
// }
//
// /************************************************************************/
// /* DxEngNUIsTermSrv                                                     */
// /************************************************************************/
//
// /* Notes: Check if terminal server got connections or not */
// BOOLEAN
// __stdcall
// DxEngNUIsTermSrv(VOID)
// {
//     /* FIXME: ReactOS does not suport terminal server yet, we can not check if we got connections or not */
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngRedrawDesktop                                                   */
// /************************************************************************/
//
// /* Notes: it always returns TRUE, and it updates whole screen
//    (redraws current desktop) */
// BOOLEAN
// __stdcall
// DxEngRedrawDesktop(VOID)
// {
//     UserRedrawDesktop();
//     return TRUE;
// }
//
// ULONG gulVisRgnUniqueness; // Increase count everytime client region is updated.
//
// /************************************************************************/
// /* DxEngVisRgnUniq                                                      */
// /************************************************************************/
// /* Notes: returns the VisRgnUniq counter for win32k */
// ULONG
// __stdcall
// DxEngVisRgnUniq(VOID)
// {
//     DPRINT("ReactX Calling : DxEngVisRgnUniq\n");
//
//     return gulVisRgnUniqueness;
// }
//
// /************************************************************************/
// /* DxEngEnumerateHdev                                                   */
// /************************************************************************/
// /* Enumerate all drivers in win32k */
// HDEV *
// __stdcall
// DxEngEnumerateHdev(HDEV *hdev)
// {
//     /* FIXME: Enumerate all drivers in win32k */
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngCreateMemoryDC                                                  */
// /************************************************************************/
// HDC
// __stdcall
// DxEngCreateMemoryDC(HDEV hDev)
// {
//     return IntGdiCreateDisplayDC(hDev, DCTYPE_MEMORY, FALSE);
// }
//
// /************************************************************************/
// /* DxEngScreenAccessCheck                                               */
// /************************************************************************/
// DWORD __stdcall DxEngScreenAccessCheck(VOID)
// {
//     UNIMPLEMENTED;
//
//     /* We're cheating here and telling dxg.sys it has always had permissions to access the screen */
//     return TRUE;
// }
//
// /************************************************************************/
// /* DxEngIsHdevLockedByCurrentThread                                     */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngIsHdevLockedByCurrentThread(HDEV hDev)
// {   // Based on EngIsSemaphoreOwnedByCurrentThread w/o the Ex call.
//     PERESOURCE pSem = (PERESOURCE)(((PPDEVOBJ)hDev)->hsemDevLock);
//     return pSem->OwnerEntry.OwnerThread == (ERESOURCE_THREAD)PsGetCurrentThread();
// }
//
// /************************************************************************/
// /* DxEngUnreferenceHdev                                                 */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngUnreferenceHdev(HDEV hDev)
// {
//     PDEVOBJ_vRelease((PPDEVOBJ)hDev);
//     return TRUE; // Always true.
// }
//
// /************************************************************************/
// /* DxEngGetDesktopDC                                                    */
// /************************************************************************/
// HDC
// __stdcall
// DxEngGetDesktopDC(ULONG DcType, BOOL EmptyDC, BOOL ValidatehWnd)
// {
//     return UserGetDesktopDC(DcType, EmptyDC, ValidatehWnd);
// }
//
// /************************************************************************/
// /* DxEngDeleteDC                                                        */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngDeleteDC(HDC hdc, BOOL Force)
// {
//    return IntGdiDeleteDC(hdc, Force);
// }
//
// /************************************************************************/
// /* DxEngCleanDC                                                         */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngCleanDC(HDC hdc)
// {
//     return IntGdiCleanDC(hdc);
// }
//
// /************************************************************************/
// /* DxEngSetDCOwner                                                      */
// /************************************************************************/
// BOOL __stdcall DxEngSetDCOwner(HGDIOBJ hObject, DWORD OwnerMask)
// {
//     DPRINT("ReactX Calling : DxEngSetDCOwner\n");
//
//     return GreSetDCOwner(hObject, OwnerMask);
// }
//
// /************************************************************************/
// /* DxEngSetDCState                                                      */
// /************************************************************************/
// BOOLEAN
// __stdcall
// DxEngSetDCState(HDC hDC, DWORD SetType, DWORD Set)
// {
//    BOOLEAN Ret = FALSE;
//    PDC pDC = DC_LockDc(hDC);
//
//    if (pDC)
//    {
//       if (SetType == 1)
//       {
//         if ( Set )
//             pDC->fs |= DC_FULLSCREEN;
//         else
//             pDC->fs &= ~DC_FULLSCREEN;
//         Ret = TRUE;
//       }
//       DC_UnlockDc(pDC);
//       return Ret; // Everything else returns FALSE.
//    }
//    return Ret;
// }
//
// /************************************************************************/
// /* DxEngSelectBitmap                                                    */
// /************************************************************************/
// HBITMAP __stdcall DxEngSelectBitmap(HDC hdc, HBITMAP hbmp)
// {
//     DPRINT("ReactX Calling : DxEngSelectBitmap\n");
//
//     return NtGdiSelectBitmap(hdc, hbmp);
// }
//
// /************************************************************************/
// /* DxEngSetBitmapOwner                                                  */
// /************************************************************************/
// BOOLEAN __stdcall DxEngSetBitmapOwner(HBITMAP hbmp, ULONG ulOwner)
// {
//     DPRINT("ReactX Calling : DxEngSetBitmapOwner\n");
//
//     return GreSetBitmapOwner(hbmp, ulOwner);
// }
//
// /************************************************************************/
// /* DxEngDeleteSurface                                                   */
// /************************************************************************/
// BOOLEAN __stdcall DxEngDeleteSurface(HSURF hsurf)
// {
//     DPRINT("ReactX Calling : DxEngDeleteSurface\n");
//
//     return EngDeleteSurface(hsurf);
// }
//
// /************************************************************************/
// /* DxEngGetSurfaceData                                                  */
// /************************************************************************/
// DWORD __stdcall DxEngGetSurfaceData(DWORD x1, DWORD x2)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngAltLockSurface                                                  */
// /************************************************************************/
// SURFOBJ * __stdcall DxEngAltLockSurface(HSURF hsurf)
// {
//     DPRINT("ReactX Calling : DxEngAltLockSurface\n");
//
//     return EngLockSurface(hsurf);
// }
//
// /************************************************************************/
// /* DxEngUploadPaletteEntryToSurface                                     */
// /************************************************************************/
// DWORD __stdcall DxEngUploadPaletteEntryToSurface(DWORD x1, DWORD x2,DWORD x3, DWORD x4)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngMarkSurfaceAsDirectDraw                                         */
// /************************************************************************/
// DWORD __stdcall DxEngMarkSurfaceAsDirectDraw(DWORD x1, DWORD x2)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSelectPaletteToSurface                                          */
// /************************************************************************/
// DWORD __stdcall DxEngSelectPaletteToSurface(DWORD x1, DWORD x2)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSyncPaletteTableWithDevice                                      */
// /************************************************************************/
// DWORD __stdcall DxEngSyncPaletteTableWithDevice(DWORD x1, DWORD x2)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSetPaletteState                                                 */
// /************************************************************************/
// DWORD __stdcall DxEngSetPaletteState(DWORD x1, DWORD x2, DWORD x3)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngGetRedirectionBitmap                                            */
// /************************************************************************/
// DWORD
// __stdcall
// DxEngGetRedirectionBitmap(DWORD x1)
// {
//     return FALSE; // Normal return.
// }
//
// /************************************************************************/
// /* DxEngLoadImage                                                       */
// /************************************************************************/
// DWORD __stdcall DxEngLoadImage(DWORD x1,DWORD x2)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSpTearDownSprites                                               */
// /************************************************************************/
// DWORD __stdcall DxEngSpTearDownSprites(DWORD x1, DWORD x2, DWORD x3)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSpUnTearDownSprites                                             */
// /************************************************************************/
// DWORD __stdcall DxEngSpUnTearDownSprites(DWORD x1, DWORD x2, DWORD x3)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// /************************************************************************/
// /* DxEngSpSpritesVisible                                                */
// /************************************************************************/
// DWORD __stdcall DxEngSpSpritesVisible(DWORD x1)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
