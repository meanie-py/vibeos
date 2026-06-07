use win32k_sys::win32k::*;
use core::ffi::c_void;
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdBeginMoCompFrame(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdBeginMoCompFrame(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdCreateMoComp(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdCreateMoComp(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdDestroyMoComp(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdDestroyMoComp(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdEndMoCompFrame(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdEndMoCompFrame(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetInternalMoCompInfo(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetInternalMoCompInfo(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetMoCompBuffInfo(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetMoCompBuffInfo(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetMoCompFormats(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetMoCompFormats(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdGetMoCompGuids(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdGetMoCompGuids(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdQueryMoCompStatus(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdQueryMoCompStatus(
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
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDdRenderMoComp(
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

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDdRenderMoComp(
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
