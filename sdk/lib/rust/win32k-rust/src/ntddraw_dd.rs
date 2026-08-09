use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateSurface(
    hDirectDrawLocal: HANDLE,
    hSurface: *mut HANDLE,
    puSurfaceDescription: *mut DDSURFACEDESC,
    puSurfaceGlobalData: *mut DD_SURFACE_GLOBAL,
    puSurfaceLocalData: *mut DD_SURFACE_LOCAL,
    puSurfaceMoreData: *mut DD_SURFACE_MORE,
    puCreateSurfaceData: PDD_CREATESURFACEDATA,
    puhSurface: *mut HANDLE,
) -> DWORD {
    let pfnDdCreateSurface: PGD_DDCREATESURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCreateSurface as usize].pfn);

    if pfnDdCreateSurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDdCreateSurface.unwrap())(
        hDirectDrawLocal,
        hSurface,
        puSurfaceDescription,
        puSurfaceGlobalData,
        puSurfaceLocalData,
        puSurfaceMoreData,
        puCreateSurfaceData,
        puhSurface,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdWaitForVerticalBlank(
    hDirectDraw: HANDLE,
    puWaitForVerticalBlankData: PDD_WAITFORVERTICALBLANKDATA,
) -> DWORD {
    let pfnDdWaitForVerticalBlank: PGD_DXDDWAITFORVERTICALBLANK =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdWaitForVerticalBlank as usize].pfn);

    if pfnDdWaitForVerticalBlank.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdWaitForVerticalBlank.unwrap())(hDirectDraw, puWaitForVerticalBlankData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCanCreateSurface(
    hDirectDrawLocal: HANDLE,
    puCanCreateSurfaceData: PDD_CANCREATESURFACEDATA,
) -> DWORD {
    let pfnDdCanCreateSurface: PGD_DDCANCREATESURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCanCreateSurface as usize].pfn);

    if pfnDdCanCreateSurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdCanCreateSurface.unwrap())(hDirectDrawLocal, puCanCreateSurfaceData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetScanLine(
    hDirectDrawLocal: HANDLE,
    puGetScanLineData: PDD_GETSCANLINEDATA,
) -> DWORD {
    let pfnDdGetScanLine: PGD_DXDDGETSCANLINE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetScanLine as usize].pfn);

    if pfnDdGetScanLine.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetScanLine.unwrap())(hDirectDrawLocal, puGetScanLineData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateSurfaceEx(
    hDirectDraw: HANDLE,
    hSurface: HANDLE,
    dwSurfaceHandle: DWORD,
) -> DWORD {
    let pfnDdCreateSurfaceEx: PGD_DXDDCREATESURFACEEX =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCreateSurfaceEx as usize].pfn);

    if pfnDdCreateSurfaceEx.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdCreateSurfaceEx.unwrap())(hDirectDraw, hSurface, dwSurfaceHandle)
}
