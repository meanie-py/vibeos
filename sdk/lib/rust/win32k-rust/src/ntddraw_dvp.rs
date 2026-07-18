use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpCanCreateVideoPort(
    hDirectDraw: HANDLE,
    puCanCreateVPortData: PDD_CANCREATEVPORTDATA,
) -> DWORD {
    let pfnDvpCanCreateVideoPort: PGD_DVPCANCREATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpCanCreateVideoPort as usize].pfn);

    if pfnDvpCanCreateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpCanCreateVideoPort.unwrap())(hDirectDraw, puCanCreateVPortData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpCanCreateVideoPort(
    hDirectDraw: HANDLE,
    puCanCreateVPortData: PDD_CANCREATEVPORTDATA,
) -> DWORD {
    let pfnDvpCanCreateVideoPort: PGD_DVPCANCREATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpCanCreateVideoPort as usize].pfn);

    if pfnDvpCanCreateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpCanCreateVideoPort.unwrap())(hDirectDraw, puCanCreateVPortData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpColorControl(
    hVideoPort: HANDLE,
    puVPortColorData: PDD_VPORTCOLORDATA,
) -> DWORD {
    let pfnDvpColorControl: PGD_DVPCOLORCONTROL =
        core::mem::transmute::<_, PGD_DVPCOLORCONTROL>(gpDxFuncs[DXG_INDEX_DxDvpColorControl as usize].pfn);

    if pfnDvpColorControl.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpColorControl.unwrap())(hVideoPort, puVPortColorData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpColorControl(
    hVideoPort: HANDLE,
    puVPortColorData: PDD_VPORTCOLORDATA,
) -> DWORD {
    let pfnDvpColorControl: PGD_DVPCOLORCONTROL =
        core::mem::transmute::<_, PGD_DVPCOLORCONTROL>(gpDxFuncs[DXG_INDEX_DxDvpColorControl as usize].pfn);

    if pfnDvpColorControl.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpColorControl.unwrap())(hVideoPort, puVPortColorData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpCreateVideoPort(
    hDirectDraw: HANDLE,
    puCreateVPortData: PDD_CREATEVPORTDATA,
) -> HANDLE {
    let pfnDvpCreateVideoPort: PGD_DVPCREATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpCreateVideoPort as usize].pfn);

    if pfnDvpCreateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDvpCreateVideoPort.unwrap())(hDirectDraw, puCreateVPortData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpCreateVideoPort(
    hDirectDraw: HANDLE,
    puCreateVPortData: PDD_CREATEVPORTDATA,
) -> HANDLE {
    let pfnDvpCreateVideoPort: PGD_DVPCREATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpCreateVideoPort as usize].pfn);

    if pfnDvpCreateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDvpCreateVideoPort.unwrap())(hDirectDraw, puCreateVPortData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpDestroyVideoPort(
    hVideoPort: HANDLE,
    puDestroyVPortData: PDD_DESTROYVPORTDATA,
) -> DWORD {
    let pfnDvpDestroyVideoPort: PGD_DVPDESTROYVIDEOPORT = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDvpDestroyVideoPort as usize].pfn,
    );

    if pfnDvpDestroyVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpDestroyVideoPort.unwrap())(hVideoPort, puDestroyVPortData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpDestroyVideoPort(
    hVideoPort: HANDLE,
    puDestroyVPortData: PDD_DESTROYVPORTDATA,
) -> DWORD {
    let pfnDvpDestroyVideoPort: PGD_DVPDESTROYVIDEOPORT = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDvpDestroyVideoPort as usize].pfn,
    );

    if pfnDvpDestroyVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpDestroyVideoPort.unwrap())(hVideoPort, puDestroyVPortData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpFlipVideoPort(
    hVideoPort: HANDLE,
    hDDSurfaceCurrent: HANDLE,
    hDDSurfaceTarget: HANDLE,
    puFlipVPortData: PDD_FLIPVPORTDATA,
) -> DWORD {
    let pfnDvpFlipVideoPort: PGD_DVPFLIPVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpFlipVideoPort as usize].pfn);

    if pfnDvpFlipVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpFlipVideoPort.unwrap())(hVideoPort, hDDSurfaceCurrent, hDDSurfaceTarget, puFlipVPortData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpFlipVideoPort(
    hVideoPort: HANDLE,
    hDDSurfaceCurrent: HANDLE,
    hDDSurfaceTarget: HANDLE,
    puFlipVPortData: PDD_FLIPVPORTDATA,
) -> DWORD {
    let pfnDvpFlipVideoPort: PGD_DVPFLIPVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpFlipVideoPort as usize].pfn);

    if pfnDvpFlipVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpFlipVideoPort.unwrap())(hVideoPort, hDDSurfaceCurrent, hDDSurfaceTarget, puFlipVPortData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortBandwidth(
    hVideoPort: HANDLE,
    puGetVPortBandwidthData: PDD_GETVPORTBANDWIDTHDATA,
) -> DWORD {
    let pfnDvpGetVideoPortBandwidth: PGD_DVPGETVIDEOPORTBANDWITH =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortBandwidth as usize].pfn);

    if pfnDvpGetVideoPortBandwidth.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortBandwidth.unwrap())(hVideoPort, puGetVPortBandwidthData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortBandwidth(
    hVideoPort: HANDLE,
    puGetVPortBandwidthData: PDD_GETVPORTBANDWIDTHDATA,
) -> DWORD {
    let pfnDvpGetVideoPortBandwidth: PGD_DVPGETVIDEOPORTBANDWITH =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortBandwidth as usize].pfn);

    if pfnDvpGetVideoPortBandwidth.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortBandwidth.unwrap())(hVideoPort, puGetVPortBandwidthData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortFlipStatus(
    hDirectDraw: HANDLE,
    puGetVPortFlipStatusData: PDD_GETVPORTFLIPSTATUSDATA,
) -> DWORD {
    let pfnDvpGetVideoPortFlipStatus: PGD_DXDVPGETVIDEOPORTFLIPSTATUS = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortFlipStatus as usize].pfn,
    );

    if pfnDvpGetVideoPortFlipStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortFlipStatus.unwrap())(hDirectDraw, puGetVPortFlipStatusData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortFlipStatus(
    hDirectDraw: HANDLE,
    puGetVPortFlipStatusData: PDD_GETVPORTFLIPSTATUSDATA,
) -> DWORD {
    let pfnDvpGetVideoPortFlipStatus: PGD_DXDVPGETVIDEOPORTFLIPSTATUS = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortFlipStatus as usize].pfn,
    );

    if pfnDvpGetVideoPortFlipStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortFlipStatus.unwrap())(hDirectDraw, puGetVPortFlipStatusData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortInputFormats(
    hVideoPort: HANDLE,
    puGetVPortInputFormatData: PDD_GETVPORTINPUTFORMATDATA,
) -> DWORD {
    let pfnDvpGetVideoPortInputFormats: PGD_DXDVPGETVIDEOPORTINPUTFORMATS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortInputFormats as usize].pfn);

    if pfnDvpGetVideoPortInputFormats.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortInputFormats.unwrap())(hVideoPort, puGetVPortInputFormatData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortInputFormats(
    hVideoPort: HANDLE,
    puGetVPortInputFormatData: PDD_GETVPORTINPUTFORMATDATA,
) -> DWORD {
    let pfnDvpGetVideoPortInputFormats: PGD_DXDVPGETVIDEOPORTINPUTFORMATS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortInputFormats as usize].pfn);

    if pfnDvpGetVideoPortInputFormats.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortInputFormats.unwrap())(hVideoPort, puGetVPortInputFormatData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortLine(
    hVideoPort: HANDLE,
    puGetVPortLineData: PDD_GETVPORTLINEDATA,
) -> DWORD {
    let pfnDvpGetVideoPortLine: PGD_DXDVPGETVIDEOPORTLINE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortLine as usize].pfn);

    if pfnDvpGetVideoPortLine.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortLine.unwrap())(hVideoPort, puGetVPortLineData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortLine(
    hVideoPort: HANDLE,
    puGetVPortLineData: PDD_GETVPORTLINEDATA,
) -> DWORD {
    let pfnDvpGetVideoPortLine: PGD_DXDVPGETVIDEOPORTLINE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortLine as usize].pfn);

    if pfnDvpGetVideoPortLine.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortLine.unwrap())(hVideoPort, puGetVPortLineData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortOutputFormats(
    hVideoPort: HANDLE,
    puGetVPortOutputFormatData: PDD_GETVPORTOUTPUTFORMATDATA,
) -> DWORD {
    let pfnDvpGetVideoPortOutputFormats: PGD_DXDVPGETVIDEOPORTOUTPUTFORMATS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortOutputFormats as usize].pfn);

    if pfnDvpGetVideoPortOutputFormats.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortOutputFormats.unwrap())(hVideoPort, puGetVPortOutputFormatData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortOutputFormats(
    hVideoPort: HANDLE,
    puGetVPortOutputFormatData: PDD_GETVPORTOUTPUTFORMATDATA,
) -> DWORD {
    let pfnDvpGetVideoPortOutputFormats: PGD_DXDVPGETVIDEOPORTOUTPUTFORMATS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortOutputFormats as usize].pfn);

    if pfnDvpGetVideoPortOutputFormats.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortOutputFormats.unwrap())(hVideoPort, puGetVPortOutputFormatData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortConnectInfo(
    hDirectDraw: HANDLE,
    puGetVPortConnectData: PDD_GETVPORTCONNECTDATA,
) -> DWORD {
    let pfnDvpGetVideoPortConnectInfo: PGD_DXDVPGETVIDEOPORTCONNECTINFO =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortConnectInfo as usize].pfn);

    if pfnDvpGetVideoPortConnectInfo.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpGetVideoPortConnectInfo.unwrap())(hDirectDraw, puGetVPortConnectData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortConnectInfo(
    hDirectDraw: HANDLE,
    puGetVPortConnectData: PDD_GETVPORTCONNECTDATA,
) -> DWORD {
    let pfnDvpGetVideoPortConnectInfo: PGD_DXDVPGETVIDEOPORTCONNECTINFO =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortConnectInfo as usize].pfn);

    if pfnDvpGetVideoPortConnectInfo.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpGetVideoPortConnectInfo.unwrap())(hDirectDraw, puGetVPortConnectData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoSignalStatus(
    hVideoPort: HANDLE,
    puGetVPortSignalData: PDD_GETVPORTSIGNALDATA,
) -> DWORD {
    let pfnDvpGetVideoSignalStatus: PGD_DXDVPGETVIDEOSIGNALSTATUS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoSignalStatus as usize].pfn);

    if pfnDvpGetVideoSignalStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoSignalStatus.unwrap())(hVideoPort, puGetVPortSignalData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoSignalStatus(
    hVideoPort: HANDLE,
    puGetVPortSignalData: PDD_GETVPORTSIGNALDATA,
) -> DWORD {
    let pfnDvpGetVideoSignalStatus: PGD_DXDVPGETVIDEOSIGNALSTATUS =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoSignalStatus as usize].pfn);

    if pfnDvpGetVideoSignalStatus.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoSignalStatus.unwrap())(hVideoPort, puGetVPortSignalData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpUpdateVideoPort(
    hVideoPort: HANDLE,
    phSurfaceVideo: *mut HANDLE,
    phSurfaceVbi: *mut HANDLE,
    puUpdateVPortData: PDD_UPDATEVPORTDATA,
) -> DWORD {
    let pfnDvpUpdateVideoPort: PGD_DXDVPUPDATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpUpdateVideoPort as usize].pfn);

    if pfnDvpUpdateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpUpdateVideoPort.unwrap())(hVideoPort, phSurfaceVideo, phSurfaceVbi, puUpdateVPortData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpUpdateVideoPort(
    hVideoPort: HANDLE,
    phSurfaceVideo: *mut HANDLE,
    phSurfaceVbi: *mut HANDLE,
    puUpdateVPortData: PDD_UPDATEVPORTDATA,
) -> DWORD {
    let pfnDvpUpdateVideoPort: PGD_DXDVPUPDATEVIDEOPORT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpUpdateVideoPort as usize].pfn);

    if pfnDvpUpdateVideoPort.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as DWORD;
    }

    (pfnDvpUpdateVideoPort.unwrap())(hVideoPort, phSurfaceVideo, phSurfaceVbi, puUpdateVPortData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpWaitForVideoPortSync(
    hVideoPort: HANDLE,
    puWaitForVPortSyncData: PDD_WAITFORVPORTSYNCDATA,
) -> DWORD {
    let pfnDvpWaitForVideoPortSync: PGD_DXDVPWAITFORVIDEOPORTSYNC =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpWaitForVideoPortSync as usize].pfn);

    if pfnDvpWaitForVideoPortSync.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpWaitForVideoPortSync.unwrap())(hVideoPort, puWaitForVPortSyncData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpWaitForVideoPortSync(
    hVideoPort: HANDLE,
    puWaitForVPortSyncData: PDD_WAITFORVPORTSYNCDATA,
) -> DWORD {
    let pfnDvpWaitForVideoPortSync: PGD_DXDVPWAITFORVIDEOPORTSYNC =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpWaitForVideoPortSync as usize].pfn);

    if pfnDvpWaitForVideoPortSync.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpWaitForVideoPortSync.unwrap())(hVideoPort, puWaitForVPortSyncData)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpAcquireNotification(
    hVideoPort: HANDLE,
    hEvent: *mut HANDLE,
    pNotify: LPDDVIDEOPORTNOTIFY,
) -> DWORD {
    let pfnDvpAcquireNotification: PGD_DXDVPACQUIRENOTIFICATION =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpAcquireNotification as usize].pfn);

    if pfnDvpAcquireNotification.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpAcquireNotification.unwrap())(hVideoPort, hEvent, pNotify)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpAcquireNotification(
    hVideoPort: HANDLE,
    hEvent: *mut HANDLE,
    pNotify: LPDDVIDEOPORTNOTIFY,
) -> DWORD {
    let pfnDvpAcquireNotification: PGD_DXDVPACQUIRENOTIFICATION =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpAcquireNotification as usize].pfn);

    if pfnDvpAcquireNotification.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpAcquireNotification.unwrap())(hVideoPort, hEvent, pNotify)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpReleaseNotification(
    hVideoPort: HANDLE,
    hEvent: HANDLE,
) -> DWORD {
    let pfnDvpReleaseNotification: PGD_DXDVPRELEASENOTIFICATION =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpReleaseNotification as usize].pfn);

    if pfnDvpReleaseNotification.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpReleaseNotification.unwrap())(hVideoPort, hEvent)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpReleaseNotification(
    hVideoPort: HANDLE,
    hEvent: HANDLE,
) -> DWORD {
    let pfnDvpReleaseNotification: PGD_DXDVPRELEASENOTIFICATION =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpReleaseNotification as usize].pfn);

    if pfnDvpReleaseNotification.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpReleaseNotification.unwrap())(hVideoPort, hEvent)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn NtGdiDvpGetVideoPortField(
    hVideoPort: HANDLE,
    puGetVPortFieldData: PDD_GETVPORTFIELDDATA,
) -> DWORD {
    let pfnDvpGetVideoPortField: PGD_DXDVPGETVIDEOPORTFIELD =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortField as usize].pfn);

    if pfnDvpGetVideoPortField.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortField.unwrap())(hVideoPort, puGetVPortFieldData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NtGdiDvpGetVideoPortField(
    hVideoPort: HANDLE,
    puGetVPortFieldData: PDD_GETVPORTFIELDDATA,
) -> DWORD {
    let pfnDvpGetVideoPortField: PGD_DXDVPGETVIDEOPORTFIELD =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDvpGetVideoPortField as usize].pfn);

    if pfnDvpGetVideoPortField.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDvpGetVideoPortField.unwrap())(hVideoPort, puGetVPortFieldData)
}
