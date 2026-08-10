use win32k_sys::win32k::*;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::constants::STATUS_PROCEDURE_NOT_FOUND;
use crate::ntddraw_dxeng::{gaEngFuncs, gcEngFuncs};

const STATUS_DLL_NOT_FOUND: i32 = 0xC0000135_u32 as i32;

// ---- file-scope preamble (translated) ----
pub static mut gpfnStartupDxGraphics: PGD_DXDDSTARTUPDXGRAPHICS = None;
pub static mut gpfnCleanupDxGraphics: PGD_DXDDCLEANUPDXGRAPHICS = None;
// #[no_mangle]: the DirectDraw cohab flip commented out reactx/ntddraw/*.c, so the
// global DXG dispatch table must be exported under its C-ABI name for the rest of
// win32k (eng/pdevobj etc.) that references it unmangled.
#[no_mangle]
pub static mut gpDxFuncs: [DRVFN; (DXG_INDEX_DxDdIoctl + 1) as usize] = unsafe { core::mem::zeroed() };
pub static mut ghDxGraphics: HANDLE = core::ptr::null_mut();
pub static mut gdwDirectDrawContext: ULONG = 0;
pub const DXDBG: u32 = 1;

#[no_mangle]
pub unsafe extern "system" fn DxDdStartupDxGraphics(
    _ulc1: ULONG,
    _DxEngDrvOld: PDRVENABLEDATA,
    _ulc2: ULONG,
    _DxgDrvOld: PDRVENABLEDATA,
    _DirectDrawContext: PULONG,
    Proc: PEPROCESS,
) -> NTSTATUS {
    let mut DxEngDrv: DRVENABLEDATA = core::mem::zeroed();
    let mut DxgDrv: DRVENABLEDATA = core::mem::zeroed();

    let mut Status: NTSTATUS = STATUS_PROCEDURE_NOT_FOUND;

    let path_bytes = b"\\SystemRoot\\System32\\drivers\\dxg.sys\0";
    let mut wide_buf: [u16; 64] = [0; 64];
    let mut i = 0;
    while i < path_bytes.len() && i < 64 {
        wide_buf[i] = path_bytes[i] as u16;
        i += 1;
    }
    ghDxGraphics = EngLoadImage(wide_buf.as_mut_ptr());

    if ghDxGraphics.is_null() {
        Status = STATUS_DLL_NOT_FOUND;
    } else {
        gpfnStartupDxGraphics = core::mem::transmute(EngFindImageProcAddress(
            ghDxGraphics,
            b"DxDdStartupDxGraphics\0".as_ptr() as *mut _,
        ));
        gpfnCleanupDxGraphics = core::mem::transmute(EngFindImageProcAddress(
            ghDxGraphics,
            b"DxDdCleanupDxGraphics\0".as_ptr() as *mut _,
        ));

        if (*(&raw const gpfnStartupDxGraphics)).is_some() && (*(&raw const gpfnCleanupDxGraphics)).is_some() {
            DxEngDrv.iDriverVersion = DDI_DRIVER_VERSION_NT5_01 as _;
            DxEngDrv.pdrvfn = core::ptr::addr_of!(gaEngFuncs) as *mut DRVFN ;
            DxEngDrv.c = gcEngFuncs;

            Status = (gpfnStartupDxGraphics.unwrap())(
                core::mem::size_of::<DRVENABLEDATA>() as ULONG,
                &mut DxEngDrv,
                core::mem::size_of::<DRVENABLEDATA>() as ULONG,
                &mut DxgDrv,
                core::ptr::addr_of_mut!(gdwDirectDrawContext),
                Proc,
            );
        }

        if !NT_SUCCESS(Status) {
            gpfnStartupDxGraphics = None;
            gpfnCleanupDxGraphics = None;
            if !ghDxGraphics.is_null() {
                EngUnloadImage(ghDxGraphics);
                ghDxGraphics = core::ptr::null_mut();
            }
        } else {
            let lstDrvFN: PDRVFN = DxgDrv.pdrvfn;
            let mut t: INT = 0;
            while t <= DXG_INDEX_DxDdIoctl as INT {
                let iFunc = (*lstDrvFN.offset(t as isize)).iFunc;
                gpDxFuncs[iFunc as usize].iFunc = iFunc;
                gpDxFuncs[iFunc as usize].pfn = (*lstDrvFN.offset(t as isize)).pfn;
                t += 1;
            }
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateDirectDrawObject(hdc: HDC) -> HANDLE {
    let pfnDdCreateDirectDrawObject: PGD_DDCREATEDIRECTDRAWOBJECT;

    if hdc.is_null() {
        return 0 as HANDLE;
    }

    pfnDdCreateDirectDrawObject = core::mem::transmute::<_, PGD_DDCREATEDIRECTDRAWOBJECT>(
        gpDxFuncs[DXG_INDEX_DxDdCreateDirectDrawObject as usize].pfn,
    );

    if pfnDdCreateDirectDrawObject.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDdCreateDirectDrawObject.unwrap())(hdc)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDxgGenericThunk(
    ulIndex: ULONG_PTR,
    ulHandle: ULONG_PTR,
    pdwSizeOfPtr1: *mut SIZE_T,
    pvPtr1: PVOID,
    pdwSizeOfPtr2: *mut SIZE_T,
    pvPtr2: PVOID,
) -> DWORD {
    let pfnDxgGenericThunk: PGD_DXGENERICTRUNK =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDxgGenericThunk as usize].pfn);

    if pfnDxgGenericThunk.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDxgGenericThunk.unwrap())(
        ulIndex,
        ulHandle,
        pdwSizeOfPtr1,
        pvPtr1,
        pdwSizeOfPtr2,
        pvPtr2,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetDriverState(pdata: PDD_GETDRIVERSTATEDATA) -> DWORD {
    let pfnDdGetDriverState: PGD_DDGETDRIVERSTATE = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDdGetDriverState as usize].pfn,
    );

    if pfnDdGetDriverState.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetDriverState.unwrap())(pdata)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdColorControl(
    hSurface: HANDLE,
    puColorControlData: PDD_COLORCONTROLDATA,
) -> DWORD {
    let pfnDdColorControl: PGD_DDCOLORCONTROL =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdColorControl as usize].pfn);

    if pfnDdColorControl.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdColorControl.unwrap())(hSurface, puColorControlData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdCreateSurfaceObject(
    hDirectDrawLocal: HANDLE,
    hSurface: HANDLE,
    puSurfaceLocal: PDD_SURFACE_LOCAL,
    puSurfaceMore: PDD_SURFACE_MORE,
    puSurfaceGlobal: PDD_SURFACE_GLOBAL,
    bComplete: BOOL,
) -> HANDLE {
    let pfnDdCreateSurfaceObject: PGD_DXDDCREATESURFACEOBJECT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdCreateSurfaceObject as usize].pfn);

    if pfnDdCreateSurfaceObject.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDdCreateSurfaceObject.unwrap())(
        hDirectDrawLocal,
        hSurface,
        puSurfaceLocal,
        puSurfaceMore,
        puSurfaceGlobal,
        bComplete,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDeleteDirectDrawObject(hDirectDrawLocal: HANDLE) -> BOOL {
    let pfnDdDeleteDirectDrawObject: PGD_DXDDDELETEDIRECTDRAWOBJECT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDeleteDirectDrawObject as usize].pfn);

    if pfnDdDeleteDirectDrawObject.is_none() {
        return 0;
    }

    if hDirectDrawLocal.is_null() {
        return 0;
    }

    (pfnDdDeleteDirectDrawObject.unwrap())(hDirectDrawLocal)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDeleteSurfaceObject(hSurface: HANDLE) -> BOOL {
    let pfnDdDeleteSurfaceObject: PGD_DXDDDELETESURFACEOBJECT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdDeleteSurfaceObject as usize].pfn);

    if pfnDdDeleteSurfaceObject.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdDeleteSurfaceObject.unwrap())(hSurface)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdQueryDirectDrawObject(
    hDirectDrawLocal: HANDLE,
    pHalInfo: *mut DD_HALINFO,
    pCallBackFlags: *mut DWORD,
    puD3dCallbacks: LPD3DNTHAL_CALLBACKS,
    puD3dDriverData: LPD3DNTHAL_GLOBALDRIVERDATA,
    puD3dBufferCallbacks: PDD_D3DBUFCALLBACKS,
    puD3dTextureFormats: LPDDSURFACEDESC,
    puNumHeaps: *mut DWORD,
    puvmList: *mut VIDEOMEMORY,
    puNumFourCC: *mut DWORD,
    puFourCC: *mut DWORD,
) -> BOOL {
    let pfnDdQueryDirectDrawObject: PGD_DXDDQUERYDIRECTDRAWOBJECT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdQueryDirectDrawObject as usize].pfn);

    if pfnDdQueryDirectDrawObject.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdQueryDirectDrawObject.unwrap())(
        hDirectDrawLocal,
        pHalInfo,
        pCallBackFlags,
        puD3dCallbacks,
        puD3dDriverData,
        puD3dBufferCallbacks,
        puD3dTextureFormats,
        puNumHeaps,
        puvmList,
        puNumFourCC,
        puFourCC,
    )
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdReenableDirectDrawObject(
    hDirectDrawLocal: HANDLE,
    pubNewMode: *mut BOOL,
) -> BOOL {
    let pfnDdReenableDirectDrawObject: PGD_DXDDREENABLEDIRECTDRAWOBJECT =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdReenableDirectDrawObject as usize].pfn);

    if pfnDdReenableDirectDrawObject.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    let status = (pfnDdReenableDirectDrawObject.unwrap())(hDirectDrawLocal, pubNewMode);
    status as i32
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetDriverInfo(
    hDirectDrawLocal: HANDLE,
    puGetDriverInfoData: PDD_GETDRIVERINFODATA,
) -> DWORD {
    let pfnDdGetDriverInfo: PGD_DXDDGETDRIVERINFO =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetDriverInfo as usize].pfn);

    if pfnDdGetDriverInfo.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetDriverInfo.unwrap())(hDirectDrawLocal, puGetDriverInfoData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetAvailDriverMemory(
    hDirectDrawLocal: HANDLE,
    puGetAvailDriverMemoryData: PDD_GETAVAILDRIVERMEMORYDATA,
) -> DWORD {
    let pfnDdGetAvailDriverMemory: PGD_DXDDGETAVAILDRIVERMEMORY =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetAvailDriverMemory as usize].pfn);

    if pfnDdGetAvailDriverMemory.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdGetAvailDriverMemory.unwrap())(hDirectDrawLocal, puGetAvailDriverMemoryData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdSetExclusiveMode(
    hDirectDraw: HANDLE,
    puSetExclusiveModeData: PDD_SETEXCLUSIVEMODEDATA,
) -> DWORD {
    let pfnDdSetExclusiveMode: PGD_DXDDSETEXCLUSIVEMODE = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDdSetExclusiveMode as usize].pfn,
    );

    if pfnDdSetExclusiveMode.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdSetExclusiveMode.unwrap())(hDirectDraw, puSetExclusiveModeData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdFlipToGDISurface(
    hDirectDraw: HANDLE,
    puFlipToGDISurfaceData: PDD_FLIPTOGDISURFACEDATA,
) -> DWORD {
    let pfnDdFlipToGDISurface: PGD_DXDDFLIPTOGDISURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdFlipToGDISurface as usize].pfn);

    if pfnDdFlipToGDISurface.is_none() {
        return DDHAL_DRIVER_NOTHANDLED;
    }

    (pfnDdFlipToGDISurface.unwrap())(hDirectDraw, puFlipToGDISurfaceData)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetDC(
    hSurface: HANDLE,
    puColorTable: *mut PALETTEENTRY,
) -> HDC {
    let pfnDdGetDC: PGD_DDGETDC = core::mem::transmute(
        gpDxFuncs[DXG_INDEX_DxDdGetDC as usize].pfn,
    );

    if pfnDdGetDC.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HDC;
    }

    (pfnDdGetDC.unwrap())(hSurface, puColorTable)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdGetDxHandle(
    hDirectDraw: HANDLE,
    hSurface: HANDLE,
    bRelease: BOOL,
) -> HANDLE {
    let pfnDdGetDxHandle: PGD_DDGETDXHANDLE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdGetDxHandle as usize].pfn);

    if pfnDdGetDxHandle.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as HANDLE;
    }

    (pfnDdGetDxHandle.unwrap())(hDirectDraw, hSurface, bRelease)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdReleaseDC(hSurface: HANDLE) -> BOOL {
    let pfnDdReleaseDC: PGD_DDRELEASEDC =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdReleaseDC as usize].pfn);

    if pfnDdReleaseDC.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdReleaseDC.unwrap())(hSurface)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdResetVisrgn(hSurface: HANDLE, hwnd: HWND) -> BOOL {
    let pfnDdResetVisrgn: PGD_DDRESTVISRGN =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdResetVisrgn as usize].pfn);

    if pfnDdResetVisrgn.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdResetVisrgn.unwrap())(hSurface, hwnd)
}
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdSetGammaRamp(
    hDirectDraw: HANDLE,
    hdc: HDC,
    lpGammaRamp: LPVOID,
) -> BOOL {
    let pfnDdSetGammaRamp: PGD_DDSETGAMMARAMP =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdSetGammaRamp as usize].pfn);

    if pfnDdSetGammaRamp.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as BOOL;
    }

    (pfnDdSetGammaRamp.unwrap())(hDirectDraw, hdc, lpGammaRamp)
}
