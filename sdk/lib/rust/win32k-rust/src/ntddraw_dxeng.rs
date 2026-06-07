use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAcquireResourceExclusiveLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, PsGetCurrentThread, Type};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut ghsemShareDevLock: HSEMAPHORE = core::ptr::null_mut();

pub static mut gcEngFuncs: ULONG = DXENG_INDEX_DxEngLoadImage + 1;

pub static gaEngFuncs: [DRVFN; 42] = [
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

unsafe extern "system" fn DxEngGetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP) -> BOOL {
    IntGetDeviceGammaRamp(hPDev, Ramp)
}

unsafe extern "system" fn DxEngLockDC(hDC: HDC) -> PDC {
    DC_LockDc(hDC)
}

unsafe extern "system" fn DxEngUnlockDC(pDC: PDC) -> BOOLEAN {
    DC_UnlockDc(pDC);
    1u8
}

unsafe extern "system" fn DxEngLockShareSem() -> BOOLEAN {
    if ghsemShareDevLock.is_null() {
        ghsemShareDevLock = EngCreateSemaphore();
    }
    EngAcquireSemaphore(ghsemShareDevLock);
    1u8
}

unsafe extern "system" fn DxEngUnlockShareSem() -> u8 {
    EngReleaseSemaphore(ghsemShareDevLock);
    1u8
}

unsafe extern "system" fn DxEngSetDeviceGammaRamp(hPDev: HDEV, Ramp: PGAMMARAMP, Test: BOOL) -> BOOLEAN {
    IntSetDeviceGammaRamp(hPDev, Ramp, Test)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetHdevData(hDev: HDEV, Type: DXEGSHDEVDATA) -> DWORD_PTR {
    let mut retVal: DWORD_PTR = 0;
    let PDev: PPDEVOBJ = hDev as PPDEVOBJ;

    match Type {
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
            retVal = core::ptr::addr_of_mut!((*PDev).DriverFunctions) as DWORD_PTR;
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

extern "system" fn DxEngSetHdevData(hDev: HDEV, Type: Type, Data: DWORD_PTR) -> BOOLEAN {
    let mut retVal: BOOLEAN = 0u8;

    if Type == DxEGShDevData_dd_locks {
        (*(hDev as PPDEVOBJ)).cDirectDrawDisableLocks = Data;
        retVal = 1u8;
    }
    retVal
}

#[no_mangle]
pub unsafe extern "system" fn DxEngGetDCState(hDC: HDC, type_: DWORD) -> DWORD_PTR {
    let pDC: PDC = DC_LockDc(hDC);
    let mut retVal: DWORD_PTR = 0;

    if !pDC.is_null() {
        match type_ {
            1 => {
                retVal = ((*pDC).fs & DC_FULLSCREEN) as DWORD_PTR;
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

unsafe extern "system" fn DxEngIncDispUniq() -> BOOLEAN {
    InterlockedIncrement(core::ptr::addr_of_mut!((*GdiHandleTable).flDeviceUniq) as *mut i32);
    1u8
}

unsafe extern "system" fn DxEngLockHdev(hDev: HDEV) -> u8 {
    let ppdev: PPDEVOBJ = hDev as PPDEVOBJ;

    let Resource: PERESOURCE = (*ppdev).hsemDevLock as PERESOURCE;

    if !Resource.is_null() {
        KeEnterCriticalRegion();
        ExAcquireResourceExclusiveLite(Resource, 1u8);
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngUnlockHdev(hDev: HDEV) -> u8 {
    let ppdev = hDev as PPDEVOBJ;
    let Resource = (*ppdev).hsemDevLock as PERESOURCE;

    if !Resource.is_null() {
        ExReleaseResourceLite(Resource);
        KeLeaveCriticalRegion();
    }
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngReferenceHdev(hDev: HDEV) -> u8 {
    PDEVOBJ_vReference(hDev as PPDEVOBJ);
    1u8
}

unsafe extern "system" fn DxEngNUIsTermSrv() -> BOOLEAN {
    0u8
}

unsafe extern "system" fn DxEngRedrawDesktop() -> BOOLEAN {
    UserRedrawDesktop();
    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DxEngVisRgnUniq() -> ULONG {
    gulVisRgnUniqueness
}

unsafe extern "system" fn DxEngCreateMemoryDC(hDev: HDEV) -> HDC {
    IntGdiCreateDisplayDC(hDev, DCTYPE_MEMORY, 0u8)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngScreenAccessCheck() -> DWORD {
    1
}

#[no_mangle]
pub unsafe extern "system" fn DxEngIsHdevLockedByCurrentThread(hDev: HDEV) -> u8 {
    let pSem: PERESOURCE = (*(hDev as PPDEVOBJ)).hsemDevLock as PERESOURCE;
    ((*pSem).OwnerEntry.OwnerThread == PsGetCurrentThread() as ERESOURCE_THREAD) as u8
}

unsafe extern "system" fn DxEngUnreferenceHdev(hDev: HDEV) -> u8 {
    PDEVOBJ_vRelease(hDev as PPDEVOBJ);
    1u8
}

unsafe extern "system" fn DxEngGetDesktopDC(DcType: ULONG, EmptyDC: BOOL, ValidatehWnd: BOOL) -> HDC {
    UserGetDesktopDC(DcType, EmptyDC, ValidatehWnd)
}

unsafe extern "system" fn DxEngDeleteDC(hdc: HDC, Force: BOOL) -> BOOLEAN {
    IntGdiDeleteDC(hdc, Force)
}

unsafe extern "system" fn DxEngCleanDC(hdc: HDC) -> BOOLEAN {
    IntGdiCleanDC(hdc)
}

unsafe extern "system" fn DxEngSetDCOwner(hObject: HGDIOBJ, OwnerMask: u32) -> i32 {
    GreSetDCOwner(hObject, OwnerMask)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetDCState(hDC: HDC, SetType: DWORD, Set: DWORD) -> BOOLEAN {
    let mut Ret: BOOLEAN = 0u8;
    let pDC: PDC = DC_LockDc(hDC);

    if !pDC.is_null() {
        if SetType == 1 {
            if Set != 0 {
                (*pDC).fs |= DC_FULLSCREEN;
            } else {
                (*pDC).fs &= !DC_FULLSCREEN;
            }
            Ret = 1u8;
        }
        DC_UnlockDc(pDC);
        return Ret;
    }
    Ret
}

unsafe extern "system" fn DxEngSelectBitmap(hdc: HDC, hbmp: HBITMAP) -> HBITMAP {
    NtGdiSelectBitmap(hdc, hbmp)
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSetBitmapOwner(hbmp: HBITMAP, ulOwner: ULONG) -> BOOLEAN {
    GreSetBitmapOwner(hbmp, ulOwner)
}

unsafe extern "system" fn DxEngDeleteSurface(hsurf: HSURF) -> BOOLEAN {
    EngDeleteSurface(hsurf)
}

unsafe extern "system" fn DxEngGetSurfaceData(x1: DWORD, x2: DWORD) -> DWORD {
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

unsafe extern "system" fn DxEngSelectPaletteToSurface(x1: DWORD, x2: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSyncPaletteTableWithDevice(_x1: DWORD, _x2: DWORD) -> DWORD {
    0
}

unsafe extern "system" fn DxEngSetPaletteState(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

unsafe extern "system" fn DxEngGetRedirectionBitmap(x1: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngLoadImage(_x1: DWORD, _x2: DWORD) -> DWORD {
    0
}

unsafe extern "system" fn DxEngSpTearDownSprites(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

unsafe extern "system" fn DxEngSpUnTearDownSprites(x1: DWORD, x2: DWORD, x3: DWORD) -> DWORD {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxEngSpSpritesVisible(_x1: DWORD) -> DWORD {
    0
}
