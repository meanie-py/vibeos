use win32k_sys::win32k::*;
use core::ffi::c_void;
use ntkrnl_sys::base::STATUS_NOT_IMPLEMENTED;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdDestroySurface(hSurface: HANDLE, bRealDestroy: BOOL) -> DWORD {
    let pfnDdDestroySurface: PGD_DXDDDESTROYSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDestroySurface as usize].pfn);

    if pfnDdDestroySurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdDestroySurface.unwrap())(hSurface, bRealDestroy)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdDestroySurface(hSurface: HANDLE, bRealDestroy: BOOL) -> DWORD {
    let pfnDdDestroySurface: PGD_DXDDDESTROYSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDestroySurface as usize].pfn);

    if pfnDdDestroySurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdDestroySurface.unwrap())(hSurface, bRealDestroy)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdFlip(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdFlip(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdLock(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdLock(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdUnlock(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdUnlock(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdBlt(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdBlt(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdSetColorKey(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdSetColorKey(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdAddAttachedSurface(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdAddAttachedSurface(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetBltStatus(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetBltStatus(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetFlipStatus(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetFlipStatus(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdUpdateOverlay(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdUpdateOverlay(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdSetOverlayPosition(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdSetOverlayPosition(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdAlphaBlt(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdAlphaBlt(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdAttachSurface(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdAttachSurface(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdUnattachSurface(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdUnattachSurface(
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
