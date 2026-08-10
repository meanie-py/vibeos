use win32k_sys::win32k::*;
use crate::ntddraw_ddraw::gpDxFuncs; // real [DRVFN; 92], shadows bindgen [DRVFN; 0]
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpCanCreateVideoPort(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpColorControl(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpCreateVideoPort(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpDestroyVideoPort(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpFlipVideoPort(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortBandwidth(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortFlipStatus(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortInputFormats(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortLine(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortOutputFormats(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortConnectInfo(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoSignalStatus(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpUpdateVideoPort(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpWaitForVideoPortSync(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpAcquireNotification(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpReleaseNotification(
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
#[no_mangle]
pub unsafe extern "system" fn NtGdiDvpGetVideoPortField(
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
