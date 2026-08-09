use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
use ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDestroySurface(hSurface: HANDLE, bRealDestroy: BOOL) -> DWORD {
    let pfnDdDestroySurface: PGD_DXDDDESTROYSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDestroySurface as usize].pfn);

    if pfnDdDestroySurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdDestroySurface.unwrap())(hSurface, bRealDestroy)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdFlip(
    hSurfaceCurrent: HANDLE,
    hSurfaceTarget: HANDLE,
    hSurfaceCurrentLeft: HANDLE,
    hSurfaceTargetLeft: HANDLE,
    puFlipData: PDD_FLIPDATA,
) -> DWORD {
    let pfnDdDdFlip: PGD_DXDDFLIP =
        core::mem::transmute::<_, PGD_DXDDFLIP>(gpDxFuncs[DXG_INDEX_DxDdFlip as usize].pfn);

    match pfnDdDdFlip {
        None => DDHAL_DRIVER_NOTHANDLED,
        Some(func) => func(
            hSurfaceCurrent,
            hSurfaceTarget,
            hSurfaceCurrentLeft,
            hSurfaceTargetLeft,
            puFlipData,
        ),
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdLock(
    hSurface: HANDLE,
    puLockData: PDD_LOCKDATA,
    hdcClip: HDC,
) -> DWORD {
    let pfnDdLock: PGD_DXDDLOCK =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdLock as usize].pfn);

    if pfnDdLock.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdLock.unwrap())(hSurface, puLockData, hdcClip)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdUnlock(
    hSurface: HANDLE,
    puUnlockData: PDD_UNLOCKDATA,
) -> DWORD {
    let pfnDdUnlock: PGD_DXDDUNLOCK =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUnlock as usize].pfn);

    if pfnDdUnlock.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdUnlock.unwrap())(hSurface, puUnlockData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdBlt(
    hSurfaceDest: HANDLE,
    hSurfaceSrc: HANDLE,
    puBltData: PDD_BLTDATA,
) -> DWORD {
    let pfnDdBlt: PGD_DDBLT = core::mem::transmute::<_, PGD_DDBLT>(gpDxFuncs[DXG_INDEX_DxDdBlt as usize].pfn);

    if pfnDdBlt.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDdBlt.unwrap())(hSurfaceDest, hSurfaceSrc, puBltData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdSetColorKey(
    hSurface: HANDLE,
    puSetColorKeyData: PDD_SETCOLORKEYDATA,
) -> DWORD {
    let pfnDdSetColorKey: PGD_DXDDSETCOLORKEY =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdSetColorKey as usize].pfn);

    if pfnDdSetColorKey.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdSetColorKey.unwrap())(hSurface, puSetColorKeyData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdAddAttachedSurface(
    hSurface: HANDLE,
    hSurfaceAttached: HANDLE,
    puAddAttachedSurfaceData: PDD_ADDATTACHEDSURFACEDATA,
) -> DWORD {
    let pfnDdAddAttachedSurface: PGD_DDADDATTACHEDSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdAddAttachedSurface as usize].pfn);

    if pfnDdAddAttachedSurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdAddAttachedSurface.unwrap())(hSurface, hSurfaceAttached, puAddAttachedSurfaceData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetBltStatus(
    hSurface: HANDLE,
    puGetBltStatusData: PDD_GETBLTSTATUSDATA,
) -> DWORD {
    let pfnDdGetBltStatus: PGD_DXDDGETBLTSTATUS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetBltStatus as usize].pfn);

    if pfnDdGetBltStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetBltStatus.unwrap())(hSurface, puGetBltStatusData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetFlipStatus(
    hSurface: HANDLE,
    puGetFlipStatusData: PDD_GETFLIPSTATUSDATA,
) -> DWORD {
    let pfnDdGetFlipStatus: PGD_DXDDGETFLIPSTATUS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetFlipStatus as usize].pfn);

    if pfnDdGetFlipStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetFlipStatus.unwrap())(hSurface, puGetFlipStatusData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdUpdateOverlay(
    hSurfaceDestination: HANDLE,
    hSurfaceSource: HANDLE,
    puUpdateOverlayData: PDD_UPDATEOVERLAYDATA,
) -> DWORD {
    let pfnDdUpdateOverlay: PGD_DXDDUPDATEOVERLAY =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUpdateOverlay as usize].pfn);

    if pfnDdUpdateOverlay.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdUpdateOverlay.unwrap())(hSurfaceDestination, hSurfaceSource, puUpdateOverlayData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdSetOverlayPosition(
    hSurfaceSource: HANDLE,
    hSurfaceDestination: HANDLE,
    puSetOverlayPositionData: PDD_SETOVERLAYPOSITIONDATA,
) -> DWORD {
    let pfnDdSetOverlayPosition: PGD_DXDDSETOVERLAYPOSITION =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdSetOverlayPosition as usize].pfn);

    if pfnDdSetOverlayPosition.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdSetOverlayPosition.unwrap())(hSurfaceSource, hSurfaceDestination, puSetOverlayPositionData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdAlphaBlt(
    hSurfaceDest: HANDLE,
    hSurfaceSrc: HANDLE,
    puBltData: PDD_BLTDATA,
) -> DWORD {
    let pfnDdAlphaBlt: PGD_DDALPHABLT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdAlphaBlt as usize].pfn);

    if pfnDdAlphaBlt.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdAlphaBlt.unwrap())(hSurfaceDest, hSurfaceSrc, puBltData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdAttachSurface(
    hSurfaceFrom: HANDLE,
    hSurfaceTo: HANDLE,
) -> BOOL {
    let pfnDdAttachSurface: PGD_DDATTACHSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdAttachSurface as usize].pfn);

    if pfnDdAttachSurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdAttachSurface.unwrap())(hSurfaceFrom, hSurfaceTo)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdUnattachSurface(
    hSurface: HANDLE,
    hSurfaceAttached: HANDLE,
) -> NTSTATUS {
    let pfnDdUnattachSurface: PGD_DXDDUNATTACHSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUnattachSurface as usize].pfn);

    if pfnDdUnattachSurface.is_none() {
        return STATUS_NOT_IMPLEMENTED;
    }

    (pfnDdUnattachSurface.unwrap())(hSurface, hSurfaceAttached) as i32
}
