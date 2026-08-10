use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCanCreateD3DBuffer(
    hDirectDraw: HANDLE,
    puCanCreateSurfaceData: PDD_CANCREATESURFACEDATA,
) -> DWORD {
    let pfnDdCanCreateD3DBuffer: PGD_DDCANCREATED3DBUFFER =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCanCreateD3DBuffer as usize].pfn);

    if pfnDdCanCreateD3DBuffer.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdCanCreateD3DBuffer.unwrap())(hDirectDraw, puCanCreateSurfaceData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiD3dContextCreate(
    hDirectDrawLocal: HANDLE,
    hSurfColor: HANDLE,
    hSurfZ: HANDLE,
    pdcci: LPD3DNTHAL_CONTEXTCREATEDATA,
) -> BOOL {
    let pfnD3dContextCreate: PGD_D3DCONTEXTCREATE =
        core::mem::transmute::<_, PGD_D3DCONTEXTCREATE>(gpDxFuncs[DXG_INDEX_DxD3dContextCreate as usize].pfn);

    if pfnD3dContextCreate.is_none() {
        return 0;
    }

    (pfnD3dContextCreate.unwrap())(hDirectDrawLocal, hSurfColor, hSurfZ, pdcci)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiD3dContextDestroy(pContextDestroyData: LPD3DNTHAL_CONTEXTDESTROYDATA) -> DWORD {
    let pfnD3dContextDestroy: PGD_D3DCONTEXTDESTROY = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxD3dContextDestroy as usize].pfn,
    );

    if pfnD3dContextDestroy.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnD3dContextDestroy.unwrap())(pContextDestroyData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiD3dContextDestroyAll(pdcad: LPD3DNTHAL_CONTEXTDESTROYALLDATA) -> DWORD {
    let pfnD3dContextDestroyAll: PGD_D3DCONTEXTDESTROYALL =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxD3dContextDestroyAll as usize].pfn);

    if pfnD3dContextDestroyAll.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnD3dContextDestroyAll.unwrap())(pdcad)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateD3DBuffer(
    hDirectDraw: HANDLE,
    hSurface: *mut HANDLE,
    puSurfaceDescription: *mut DDSURFACEDESC,
    puSurfaceGlobalData: *mut DD_SURFACE_GLOBAL,
    puSurfaceLocalData: *mut DD_SURFACE_LOCAL,
    puSurfaceMoreData: *mut DD_SURFACE_MORE,
    puCreateSurfaceData: PDD_CREATESURFACEDATA,
    puhSurface: *mut HANDLE,
) -> DWORD {
    let pfnDdCreateD3DBuffer: PGD_DDCREATED3DBUFFER =
        core::mem::transmute::<_, PGD_DDCREATED3DBUFFER>(gpDxFuncs[DXG_INDEX_DxDdCreateD3DBuffer as usize].pfn);

    if pfnDdCreateD3DBuffer.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdCreateD3DBuffer.unwrap())(
        hDirectDraw,
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
pub unsafe extern "system" fn NtGdiDdDestroyD3DBuffer(hSurface: HANDLE) -> DWORD {
    let pfnDdDestroyD3DBuffer: PGD_DXDDDESTROYD3DBUFFER =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDestroyD3DBuffer as usize].pfn);

    if pfnDdDestroyD3DBuffer.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDdDestroyD3DBuffer.unwrap())(hSurface)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiD3dDrawPrimitives2(
    hCmdBuf: HANDLE,
    hVBuf: HANDLE,
    pded: LPD3DNTHAL_DRAWPRIMITIVES2DATA,
    pfpVidMemCmd: *mut FLATPTR,
    pdwSizeCmd: *mut DWORD,
    pfpVidMemVtx: *mut FLATPTR,
    pdwSizeVtx: *mut DWORD,
) -> DWORD {
    let pfnD3dDrawPrimitives2: PGD_D3DDRAWPRIMITIVES2 =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxD3dDrawPrimitives2 as usize].pfn);

    if pfnD3dDrawPrimitives2.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnD3dDrawPrimitives2.unwrap())(
        hCmdBuf,
        hVBuf,
        pded,
        pfpVidMemCmd,
        pdwSizeCmd,
        pfpVidMemVtx,
        pdwSizeVtx,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdLockD3D(
    hSurface: HANDLE,
    puLockData: PDD_LOCKDATA,
) -> DWORD {
    let pfnDdLockD3D: PGD_DXDDLOCKD3D = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDdLockD3D as usize].pfn,
    );

    if pfnDdLockD3D.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdLockD3D.unwrap())(hSurface, puLockData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiD3dValidateTextureStageState(
    pData: LPD3DNTHAL_VALIDATETEXTURESTAGESTATEDATA,
) -> DWORD {
    let pfnD3dValidateTextureStageState: PGD_D3DVALIDATETEXTURESTAGESTATE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxD3dValidateTextureStageState as usize].pfn);

    if pfnD3dValidateTextureStageState.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnD3dValidateTextureStageState.unwrap())(pData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdUnlockD3D(
    hSurface: HANDLE,
    puUnlockData: PDD_UNLOCKDATA,
) -> DWORD {
    let pfnDdUnlockD3D: PGD_DXDDUNLOCKD3D =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUnlockD3D as usize].pfn);

    if pfnDdUnlockD3D.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdUnlockD3D.unwrap())(hSurface, puUnlockData)
}
