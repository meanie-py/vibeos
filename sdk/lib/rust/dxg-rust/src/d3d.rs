//! win32ss/reactx/dxg/d3d.c

use win32k_sys::win32k::*;
use ntkrnl_sys::interlocked::InterlockedDecrement;
use crate::types::*;
use crate::tags::*;
use crate::ddhmg::DdHmgLock;
use crate::ddraw::intDdCreateNewSurfaceObject;
use crate::driver_main::gpEngFuncs;

#[no_mangle]
pub unsafe extern "system" fn DxDdCanCreateD3DBuffer(
    DdHandle: HANDLE,
    SurfaceData: PDD_CANCREATESURFACEDATA,
) -> DWORD {
    let mut RetVal: DWORD = DDHAL_DRIVER_NOTHANDLED;

    let peDdL = DdHmgLock(DdHandle, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;
    if peDdL.is_null() {
        return RetVal;
    }

    let peDdGl = (*peDdL).peDirectDrawGlobal2;
    (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);

    (*SurfaceData).lpDD = peDdGl as PDD_DIRECTDRAW_GLOBAL;

    if (*peDdGl).d3dBufCallbacks.CanCreateD3DBuffer.is_some() {
        RetVal = ((*peDdGl).d3dBufCallbacks.CanCreateD3DBuffer.unwrap())(SurfaceData);
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    RetVal
}

unsafe extern "C" fn intDdCreateSurfaceOrBuffer(
    hDirectDrawLocal: HANDLE,
    mut pDdSurfList: PEDD_SURFACE,
    _a3: *mut DDSURFACEDESC2,
    mut pDdSurfGlob: *mut DD_SURFACE_GLOBAL,
    mut pDdSurfLoc: *mut DD_SURFACE_LOCAL,
    mut pDdSurfMore: *mut DD_SURFACE_MORE,
    pDdCreateSurfaceData: *mut DD_CREATESURFACEDATA,
    mut Address: PVOID,
) -> DWORD {
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let mut pCurSurf: PEDD_SURFACE;

    let mut CurSurf: ULONG;

    if pDdCreateSurfaceData.is_null() {
        return FALSE as DWORD;
    }

    if (*pDdCreateSurfaceData).dwSCnt == 0 {
        (*pDdCreateSurfaceData).ddRVal = E_FAIL;
        return FALSE as DWORD;
    }

    peDdL = DdHmgLock(hDirectDrawLocal, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;
    if peDdL.is_null() {
        return FALSE as DWORD;
    }

    peDdGl = (*peDdL).peDirectDrawGlobal2;

    if ((*pDdSurfLoc).ddsCaps.dwCaps & DDSCAPS_VISIBLE) == 0
        && ((*peDdGl).ddCallbacks.dwFlags & DDHAL_CB32_CREATESURFACE) == 0
    {
        (*pDdCreateSurfaceData).ddRVal = E_FAIL;
        return FALSE as DWORD;
    }

    pDdSurfList = EngAllocMem(
        FL_ZERO_MEMORY,
        ((*pDdCreateSurfaceData).dwSCnt as usize * core::mem::size_of::<EDD_SURFACE>()) as ULONG,
        TAG_GDDP,
    ) as PEDD_SURFACE;
    pDdSurfGlob = EngAllocMem(
        FL_ZERO_MEMORY,
        ((*pDdCreateSurfaceData).dwSCnt as usize * core::mem::size_of::<DD_SURFACE_GLOBAL>()) as ULONG,
        TAG_GDDP,
    ) as *mut DD_SURFACE_GLOBAL;
    pDdSurfLoc = EngAllocMem(
        FL_ZERO_MEMORY,
        ((*pDdCreateSurfaceData).dwSCnt as usize * core::mem::size_of::<DD_SURFACE_LOCAL>()) as ULONG,
        TAG_GDDP,
    ) as *mut DD_SURFACE_LOCAL;
    pDdSurfMore = EngAllocMem(
        FL_ZERO_MEMORY,
        ((*pDdCreateSurfaceData).dwSCnt as usize * core::mem::size_of::<DD_SURFACE_MORE>()) as ULONG,
        TAG_GDDP,
    ) as *mut DD_SURFACE_MORE;

    (gpEngFuncs.DxEngLockShareSem.unwrap())();
    (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);

    CurSurf = 0;
    while CurSurf < (*pDdCreateSurfaceData).dwSCnt {
        pCurSurf = pDdSurfList.add(CurSurf as usize);
        let pCurSurfLocal = pDdSurfLoc.add(CurSurf as usize);
        let pCurSurfGlobal = pDdSurfGlob.add(CurSurf as usize);
        let pCurSurfMore = pDdSurfMore.add(CurSurf as usize);

        pCurSurf = intDdCreateNewSurfaceObject(
            peDdL,
            pCurSurf as HANDLE,
            pCurSurfGlobal,
            pCurSurfLocal,
            pCurSurfMore,
        );
        Address = pCurSurf as PVOID;

        CurSurf += 1;
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
    (gpEngFuncs.DxEngUnlockShareSem.unwrap())();

    DDHAL_DRIVER_HANDLED
}

#[no_mangle]
pub unsafe extern "system" fn DxDdCreateD3DBuffer(
    hDirectDrawLocal: HANDLE,
    pDdSurfList: PEDD_SURFACE,
    a3: *mut DDSURFACEDESC2,
    pDdSurfGlob: *mut DD_SURFACE_GLOBAL,
    pDdSurfLoc: *mut DD_SURFACE_LOCAL,
    pDdSurfMore: *mut DD_SURFACE_MORE,
    pDdCreateSurfaceData: *mut DD_CREATESURFACEDATA,
    Address: PVOID,
) -> DWORD {
    intDdCreateSurfaceOrBuffer(
        hDirectDrawLocal,
        pDdSurfList,
        a3,
        pDdSurfGlob,
        pDdSurfLoc,
        pDdSurfMore,
        pDdCreateSurfaceData,
        Address,
    )
}
