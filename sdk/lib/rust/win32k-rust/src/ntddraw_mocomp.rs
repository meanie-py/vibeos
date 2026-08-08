use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdBeginMoCompFrame(
    hMoComp: HANDLE,
    puBeginFrameData: PDD_BEGINMOCOMPFRAMEDATA,
) -> DWORD {
    let pfnDdBeginMoCompFrame: PGD_DDBEGINMOCOMPFRAME =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdBeginMoCompFrame as usize].pfn);

    if pfnDdBeginMoCompFrame.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdBeginMoCompFrame.unwrap())(hMoComp, puBeginFrameData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateMoComp(
    hDirectDraw: HANDLE,
    puCreateMoCompData: PDD_CREATEMOCOMPDATA,
) -> HANDLE {
    let pfnDdCreateMoComp: PGD_DXDDCREATEMOCOMP =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCreateMoComp as usize].pfn);

    if pfnDdCreateMoComp.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDdCreateMoComp.unwrap())(hDirectDraw, puCreateMoCompData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDestroyMoComp(
    hMoComp: HANDLE,
    puBeginFrameData: PDD_DESTROYMOCOMPDATA,
) -> DWORD {
    let pfnDdDestroyMoComp: PGD_DXDDDESTROYMOCOMP =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDestroyMoComp as usize].pfn);

    if pfnDdDestroyMoComp.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDdDestroyMoComp.unwrap())(hMoComp, puBeginFrameData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdEndMoCompFrame(
    hMoComp: HANDLE,
    puEndFrameData: PDD_ENDMOCOMPFRAMEDATA,
) -> DWORD {
    let pfnDdEndMoCompFrame: PGD_DXDDENDMOCOMPFRAME =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdEndMoCompFrame as usize].pfn);

    if pfnDdEndMoCompFrame.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDdEndMoCompFrame.unwrap())(hMoComp, puEndFrameData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetInternalMoCompInfo(
    hDirectDraw: HANDLE,
    puGetInternalData: PDD_GETINTERNALMOCOMPDATA,
) -> DWORD {
    let pfnDdGetInternalMoCompInfo: PGD_DXDDGETINTERNALMOCOMPINFO =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetInternalMoCompInfo as usize].pfn);

    if pfnDdGetInternalMoCompInfo.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetInternalMoCompInfo.unwrap())(hDirectDraw, puGetInternalData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetMoCompBuffInfo(
    hDirectDraw: HANDLE,
    puGetBuffData: PDD_GETMOCOMPCOMPBUFFDATA,
) -> DWORD {
    let pfnDdGetMoCompBuffInfo: PGD_DXDDGETMOCOMPBUFFINFO =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetMoCompBuffInfo as usize].pfn);

    if pfnDdGetMoCompBuffInfo.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetMoCompBuffInfo.unwrap())(hDirectDraw, puGetBuffData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetMoCompFormats(
    hDirectDraw: HANDLE,
    puGetMoCompFormatsData: PDD_GETMOCOMPFORMATSDATA,
) -> DWORD {
    let pfnDdGetMoCompFormats: PGD_DXDDGETMOCOMPFORMATS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetMoCompFormats as usize].pfn);

    if pfnDdGetMoCompFormats.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetMoCompFormats.unwrap())(hDirectDraw, puGetMoCompFormatsData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetMoCompGuids(
    hDirectDraw: HANDLE,
    puGetMoCompGuidsData: PDD_GETMOCOMPGUIDSDATA,
) -> DWORD {
    let pfnDdGetMoCompGuids: PGD_DXDDGETMOCOMPGUIDS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetMoCompGuids as usize].pfn);

    if pfnDdGetMoCompGuids.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetMoCompGuids.unwrap())(hDirectDraw, puGetMoCompGuidsData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdQueryMoCompStatus(
    hMoComp: HANDLE,
    puQueryMoCompStatusData: PDD_QUERYMOCOMPSTATUSDATA,
) -> DWORD {
    let pfnDdQueryMoCompStatus: PGD_DXDDQUERYMOCOMPSTATUS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdQueryMoCompStatus as usize].pfn);

    if pfnDdQueryMoCompStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdQueryMoCompStatus.unwrap())(hMoComp, puQueryMoCompStatusData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdRenderMoComp(
    hMoComp: HANDLE,
    puRenderMoCompData: PDD_RENDERMOCOMPDATA,
) -> DWORD {
    let pfnDdRenderMoComp: PGD_DXDDRENDERMOCOMP =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdRenderMoComp as usize].pfn);

    if pfnDdRenderMoComp.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdRenderMoComp.unwrap())(hMoComp, puRenderMoCompData)
}
