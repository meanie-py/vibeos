//! win32ss/reactx/dxg/ddraw.c

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::PsGetCurrentThreadProcessId;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::interlocked::InterlockedDecrement;
use crate::types::*;
use crate::tags::*;
use crate::ddhmg::{DdHmgLock, DdHmgAlloc};
use crate::driver_main::gpEngFuncs;

/// Local replacement for the inline `InlineIsEqualGUID` (not exported by the
/// sys crates): byte-compare two GUIDs. Returns BOOL (1 == equal).
#[inline]
unsafe fn InlineIsEqualGUID(g1: *const GUID, g2: *const GUID) -> BOOL {
    if (*g1).Data1 == (*g2).Data1
        && (*g1).Data2 == (*g2).Data2
        && (*g1).Data3 == (*g2).Data3
        && (*g1).Data4 == (*g2).Data4
    {
        1
    } else {
        0
    }
}

unsafe fn intDdGetDriverInfo(
    peDdGl: PEDD_DIRECTDRAW_GLOBAL,
    guid: GUID,
    callbackStruct: PVOID,
    callbackSize: ULONG,
    returnSize: *mut ULONG,
) -> BOOL {
    let mut ddGetDriverInfoData: DD_GETDRIVERINFODATA = core::mem::zeroed();

    if ((*peDdGl).ddHalInfo.dwFlags & DDHALINFO_GETDRIVERINFOSET) != 0
        && (*peDdGl).ddHalInfo.GetDriverInfo.is_some()
    {
        ddGetDriverInfoData.dwSize = core::mem::size_of::<DD_GETDRIVERINFODATA>() as DWORD;
        ddGetDriverInfoData.dhpdev = (*peDdGl).dhpdev;
        core::ptr::copy_nonoverlapping(
            &guid as *const GUID as *const u8,
            &mut ddGetDriverInfoData.guidInfo as *mut GUID as *mut u8,
            core::mem::size_of::<GUID>(),
        );
        ddGetDriverInfoData.dwExpectedSize = callbackSize;
        ddGetDriverInfoData.lpvData = callbackStruct;
        ddGetDriverInfoData.ddRVal = DDERR_CURRENTLYNOTAVAIL as HRESULT;

        if ((*peDdGl).ddHalInfo.GetDriverInfo.unwrap())(&mut ddGetDriverInfoData) != 0
            && ddGetDriverInfoData.ddRVal == 0
        {
            if !returnSize.is_null() {
                *returnSize = ddGetDriverInfoData.dwActualSize;
            }
            return 1;
        }
    }

    core::ptr::write_bytes(callbackStruct as *mut u8, 0, callbackSize as usize);
    if !returnSize.is_null() {
        *returnSize = 0;
    }
    0
}

unsafe fn intDdGetAllDriverInfo(peDdGl: PEDD_DIRECTDRAW_GLOBAL) {
    if (*peDdGl).ddHalInfo.GetDriverInfo.is_some()
        && (*peDdGl).ddHalInfo.dwFlags & DDHALINFO_GETDRIVERINFOSET != 0
    {
        intDdGetDriverInfo(peDdGl, GUID_KernelCaps, core::ptr::addr_of_mut!((*peDdGl).ddKernelCaps) as PVOID, core::mem::size_of_val(&(*peDdGl).ddKernelCaps) as ULONG, core::ptr::null_mut());
        intDdGetDriverInfo(peDdGl, GUID_KernelCallbacks, core::ptr::addr_of_mut!((*peDdGl).ddKernelCallbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddKernelCallbacks) as ULONG, core::ptr::null_mut());

        if intDdGetDriverInfo(peDdGl, GUID_D3DCallbacks3, core::ptr::addr_of_mut!((*peDdGl).d3dNtHalCallbacks3) as PVOID, core::mem::size_of_val(&(*peDdGl).d3dNtHalCallbacks3) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_D3DCALLBACKS3;
        }

        if intDdGetDriverInfo(peDdGl, GUID_ColorControlCallbacks, core::ptr::addr_of_mut!((*peDdGl).ddColorControlCallbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddColorControlCallbacks) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_COLORCONTROLCALLBACKS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_MiscellaneousCallbacks, core::ptr::addr_of_mut!((*peDdGl).ddMiscellanousCallbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddMiscellanousCallbacks) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_MISCCALLBACKS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_Miscellaneous2Callbacks, core::ptr::addr_of_mut!((*peDdGl).ddMiscellanous2Callbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddMiscellanous2Callbacks) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_MISC2CALLBACKS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_NTCallbacks, core::ptr::addr_of_mut!((*peDdGl).ddNtCallbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddNtCallbacks) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_NTCALLBACKS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_DDMoreCaps, core::ptr::addr_of_mut!((*peDdGl).ddMoreCaps) as PVOID, core::mem::size_of_val(&(*peDdGl).ddMoreCaps) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_DDMORECAPS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_NTPrivateDriverCaps, core::ptr::addr_of_mut!((*peDdGl).ddNtPrivateDriverCaps) as PVOID, core::mem::size_of_val(&(*peDdGl).ddNtPrivateDriverCaps) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_PRIVATEDRIVERCAPS;
        }

        if intDdGetDriverInfo(peDdGl, GUID_MotionCompCallbacks, core::ptr::addr_of_mut!((*peDdGl).ddMotionCompCallbacks) as PVOID, core::mem::size_of_val(&(*peDdGl).ddMotionCompCallbacks) as ULONG, core::ptr::null_mut()) != 0 {
            (*peDdGl).dwCallbackFlags |= EDDDGBL_MOTIONCOMPCALLBACKS;
        }
    }
}

unsafe fn intDdEnableDriver(peDdGl: PEDD_DIRECTDRAW_GLOBAL) {
    let DriverFunctions: PDRIVER_FUNCTIONS;
    let GlobalDriverData: LPD3DNTHAL_GLOBALDRIVERDATA;
    let HalCallbacks: LPD3DNTHAL_CALLBACKS;
    let D3DBufCallbacks: PDD_D3DBUFCALLBACKS;

    (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);
    DriverFunctions = (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_DrvFuncs) as PDRIVER_FUNCTIONS;

    if (*DriverFunctions).GetDirectDrawInfo.is_none()
        || (*DriverFunctions).EnableDirectDraw.is_none()
        || (*DriverFunctions).DisableDirectDraw.is_none()
    {
        (*peDdGl).dhpdev = core::ptr::null_mut();
    }

    (*peDdGl).fl &= 0xFFFFFFFE;

    if !(*peDdGl).dhpdev.is_null()
        && ((*DriverFunctions).GetDirectDrawInfo.unwrap())(
            (*peDdGl).dhpdev as DHPDEV,
            core::ptr::addr_of_mut!((*peDdGl).ddHalInfo),
            core::ptr::addr_of_mut!((*peDdGl).dwNumHeaps),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*peDdGl).dwNumFourCC),
            core::ptr::null_mut(),
        ) != 0
    {
        if (*peDdGl).dwNumHeaps != 0 {
            (*peDdGl).pvmList = EngAllocMem(FL_ZERO_MEMORY, ((*peDdGl).dwNumHeaps as usize * core::mem::size_of::<VIDEOMEMORY>()) as ULONG, TAG_GDDV) as *mut VIDEOMEMORY;
        }
        if (*peDdGl).dwNumFourCC != 0 {
            (*peDdGl).pdwFourCC = EngAllocMem(FL_ZERO_MEMORY, (*peDdGl).dwNumFourCC * 4, TAG_GDDF) as PDWORD;
        }

        if ((*DriverFunctions).GetDirectDrawInfo.unwrap())(
            (*peDdGl).dhpdev as DHPDEV,
            core::ptr::addr_of_mut!((*peDdGl).ddHalInfo),
            core::ptr::addr_of_mut!((*peDdGl).dwNumHeaps),
            (*peDdGl).pvmList,
            core::ptr::addr_of_mut!((*peDdGl).dwNumFourCC),
            (*peDdGl).pdwFourCC,
        ) == 0
        {
            if !(*peDdGl).pvmList.is_null() {
                EngFreeMem((*peDdGl).pvmList as PVOID);
            }
            if !(*peDdGl).pdwFourCC.is_null() {
                EngFreeMem((*peDdGl).pdwFourCC as PVOID);
            }
            (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
            return;
        }

        if !(*peDdGl).ddHalInfo.vmiData.pvPrimary.is_null()
            && ((*DriverFunctions).EnableDirectDraw.unwrap())(
                (*peDdGl).dhpdev as DHPDEV,
                core::ptr::addr_of_mut!((*peDdGl).ddCallbacks),
                core::ptr::addr_of_mut!((*peDdGl).ddSurfaceCallbacks),
                core::ptr::addr_of_mut!((*peDdGl).ddPaletteCallbacks),
            ) != 0
            && ((gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_dd_flags) as DWORD & CapOver_DisableD3DAccel) == 0
            && (*peDdGl).ddHalInfo.dwSize == core::mem::size_of::<DD_HALINFO>() as DWORD
        {
            GlobalDriverData = (*peDdGl).ddHalInfo.lpD3DGlobalDriverData as LPD3DNTHAL_GLOBALDRIVERDATA;
            HalCallbacks = (*peDdGl).ddHalInfo.lpD3DHALCallbacks as LPD3DNTHAL_CALLBACKS;
            D3DBufCallbacks = (*peDdGl).ddHalInfo.lpD3DHALCallbacks as PDD_D3DBUFCALLBACKS;

            if !GlobalDriverData.is_null() && (*GlobalDriverData).dwSize == core::mem::size_of::<D3DNTHAL_GLOBALDRIVERDATA>() as DWORD {
                core::ptr::copy_nonoverlapping(GlobalDriverData as *const u8, core::ptr::addr_of_mut!((*peDdGl).d3dNtGlobalDriverData) as *mut u8, core::mem::size_of::<D3DNTHAL_GLOBALDRIVERDATA>());
            }

            if !HalCallbacks.is_null() && (*HalCallbacks).dwSize == core::mem::size_of::<D3DNTHAL_CALLBACKS>() as DWORD {
                core::ptr::copy_nonoverlapping(HalCallbacks as *const u8, core::ptr::addr_of_mut!((*peDdGl).d3dNtHalCallbacks) as *mut u8, core::mem::size_of::<D3DNTHAL_CALLBACKS>());
            }

            if !D3DBufCallbacks.is_null() && (*D3DBufCallbacks).dwSize == core::mem::size_of::<DD_D3DBUFCALLBACKS>() as DWORD {
                core::ptr::copy_nonoverlapping(D3DBufCallbacks as *const u8, core::ptr::addr_of_mut!((*peDdGl).d3dBufCallbacks) as *mut u8, core::mem::size_of::<DD_D3DBUFCALLBACKS>());
            }

            intDdGetAllDriverInfo(peDdGl);

            if !(*peDdGl).pvmList.is_null() && (*peDdGl).dwNumHeaps > 0 {
                let mut VidMemEntry: *mut VIDEOMEMORY;
                let mut HeapIndex: DWORD;
                let DisplayPitch: DWORD = (*peDdGl).ddHalInfo.vmiData.lDisplayPitch as DWORD;

                HeapIndex = 0;
                VidMemEntry = (*peDdGl).pvmList;
                while HeapIndex < (*peDdGl).dwNumHeaps {
                    if ((*VidMemEntry).dwFlags & (VIDMEM_ISHEAP | VIDMEM_HEAPDISABLED)) != 0 {
                        HeapIndex += 1;
                        VidMemEntry = VidMemEntry.add(1);
                        continue;
                    }

                    let hs = EngAllocMem(FL_ZERO_MEMORY, core::mem::size_of::<VMEMHEAP>() as ULONG, TAG_GDDV) as LPVMEMHEAP;
                    if hs.is_null() {
                        (*VidMemEntry).dwFlags |= VIDMEM_HEAPDISABLED;
                        HeapIndex += 1;
                        VidMemEntry = VidMemEntry.add(1);
                        continue;
                    }

                    if ((*VidMemEntry).dwFlags & VIDMEM_ISLINEAR) != 0 {
                        (*hs).dwFlags = VMEMHEAP_LINEAR;
                        (*hs).dwTotalSize = ((*VidMemEntry).__bindgen_anon_1.fpEnd - (*VidMemEntry).fpStart + 1) as DWORD;
                    } else {
                        (*hs).dwFlags = VMEMHEAP_RECTANGULAR;
                        (*hs).stride = if DisplayPitch != 0 { DisplayPitch } else { (*VidMemEntry).__bindgen_anon_1.dwWidth };
                        (*hs).dwTotalSize = (*hs).stride * (*VidMemEntry).__bindgen_anon_2.dwHeight;
                    }

                    (*VidMemEntry).__bindgen_anon_2.lpHeap = hs;
                    (*VidMemEntry).dwFlags |= VIDMEM_ISHEAP;

                    HeapIndex += 1;
                    VidMemEntry = VidMemEntry.add(1);
                }
            }

            (*peDdGl).fl |= 1;
        } else {
            if !(*peDdGl).pvmList.is_null() {
                EngFreeMem((*peDdGl).pvmList as PVOID);
            }
            if !(*peDdGl).pdwFourCC.is_null() {
                EngFreeMem((*peDdGl).pdwFourCC as PVOID);
            }
        }
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
}

unsafe fn intDdCreateDirectDrawLocal(hDev: HDEV) -> PVOID {
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let AllocRet: PDD_ENTRY;

    peDdGl = (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev, DxEGShDevData_eddg) as PEDD_DIRECTDRAW_GLOBAL;

    AllocRet = DdHmgAlloc(core::mem::size_of::<EDD_DIRECTDRAW_LOCAL>() as ULONG, ObjType_DDLOCAL_TYPE as CHAR, 1) as PDD_ENTRY;
    if AllocRet.is_null() {
        return core::ptr::null_mut();
    }

    peDdL = (*AllocRet).u.pobj as PEDD_DIRECTDRAW_LOCAL;

    (*peDdL).peDirectDrawLocal_prev = (*peDdGl).peDirectDrawLocalList;
    (*peDdL).hCreatorProcess = PsGetCurrentThreadProcessId() as HANDLE;
    (*peDdL).Process = PsGetCurrentProcess() as PEPROCESS;

    (*peDdGl).peDirectDrawLocalList = peDdL;
    (*peDdL).peDirectDrawGlobal = peDdGl;
    (*peDdL).peDirectDrawGlobal2 = peDdGl;

    (gpEngFuncs.DxEngReferenceHdev.unwrap())(hDev);

    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    (*peDdL).pobj.hHmgr as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn DxDdCreateDirectDrawObject(hDC: HDC) -> DWORD_PTR {
    let pDC: PDC;
    let hDev: HDEV;
    let mut retVal: DWORD_PTR = 0;

    pDC = (gpEngFuncs.DxEngLockDC.unwrap())(hDC);
    if pDC.is_null() {
        return 0;
    }

    hDev = (gpEngFuncs.DxEngGetDCState.unwrap())(hDC, 3) as HDEV;
    if hDev.is_null() {
        (gpEngFuncs.DxEngUnlockDC.unwrap())(pDC);
        return 0;
    }

    if (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev, DxEGShDevData_display) == 0 {
        (gpEngFuncs.DxEngUnlockDC.unwrap())(pDC);
        return 0;
    }

    (gpEngFuncs.DxEngLockHdev.unwrap())(hDev);

    if (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev, DxEGShDevData_DitherFmt) >= BMF_8BPP as DWORD_PTR {
        retVal = intDdCreateDirectDrawLocal(hDev) as DWORD_PTR;
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())(hDev);
    (gpEngFuncs.DxEngUnlockDC.unwrap())(pDC);
    retVal
}

#[no_mangle]
pub unsafe extern "system" fn DxDdGetDriverInfo(
    DdHandle: HANDLE,
    drvInfoData: PDD_GETDRIVERINFODATA,
) -> DWORD {
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let mut pInfo: PVOID = core::ptr::null_mut();
    let mut dwInfoSize: DWORD = 0;
    let mut callbackStruct: [BYTE; 1024] = [0; 1024];
    let RetVal: DWORD = FALSE as DWORD;

    peDdL = DdHmgLock(DdHandle, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;
    if peDdL.is_null() {
        return RetVal;
    }

    peDdGl = (*peDdL).peDirectDrawGlobal2;

    if ((*peDdGl).dwCallbackFlags & EDDDGBL_VIDEOPORTCALLBACKS) != 0 {
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_VideoPortCallbacks)) != 0 {
            dwInfoSize = core::mem::size_of::<DD_VIDEOPORTCALLBACKS>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).ddVideoPortCallback) as PVOID;
        }
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_VideoPortCaps)) != 0 {
            pInfo = (*peDdGl).lpDDVideoPortCaps as PVOID;
            dwInfoSize = 72 * (*peDdGl).ddHalInfo.ddCaps.dwMaxVideoPorts;
        }
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_D3DCallbacks3)) != 0 {
            dwInfoSize = core::mem::size_of::<D3DNTHAL_CALLBACKS3>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).d3dNtHalCallbacks3) as PVOID;
        }
    }

    if ((*peDdGl).dwCallbackFlags & EDDDGBL_COLORCONTROLCALLBACKS) != 0 {
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_ColorControlCallbacks)) != 0 {
            dwInfoSize = core::mem::size_of::<DD_COLORCONTROLCALLBACKS>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).ddColorControlCallbacks) as PVOID;
        }
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_NTCallbacks)) != 0 {
            dwInfoSize = core::mem::size_of::<DD_NTCALLBACKS>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).ddNtCallbacks) as PVOID;
        }
    }

    if ((*peDdGl).dwCallbackFlags & EDDDGBL_MISCCALLBACKS) != 0 {
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_MiscellaneousCallbacks)) != 0 {
            dwInfoSize = core::mem::size_of::<DD_MISCELLANEOUSCALLBACKS>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).ddMiscellanousCallbacks) as PVOID;
        }
        if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_DDMoreCaps)) != 0 {
            dwInfoSize = core::mem::size_of::<DD_MORECAPS>() as DWORD;
            pInfo = core::ptr::addr_of_mut!((*peDdGl).ddMoreCaps) as PVOID;
        }
    }

    if ((*peDdGl).dwCallbackFlags & EDDDGBL_MISC2CALLBACKS) != 0
        && InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_Miscellaneous2Callbacks)) != 0
    {
        dwInfoSize = core::mem::size_of::<DD_MISCELLANEOUS2CALLBACKS>() as DWORD;
        pInfo = core::ptr::addr_of_mut!((*peDdGl).ddMiscellanous2Callbacks) as PVOID;
    }

    if ((*peDdGl).dwCallbackFlags & EDDDGBL_MOTIONCOMPCALLBACKS) != 0
        && InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_MotionCompCallbacks)) != 0
    {
        dwInfoSize = core::mem::size_of::<DD_MOTIONCOMPCALLBACKS>() as DWORD;
        pInfo = core::ptr::addr_of_mut!((*peDdGl).ddMotionCompCallbacks) as PVOID;
    }

    if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_KernelCaps)) != 0 {
        dwInfoSize = core::mem::size_of::<DD_KERNELCALLBACKS>() as DWORD;
        pInfo = core::ptr::addr_of_mut!((*peDdGl).ddKernelCaps) as PVOID;
    }

    if InlineIsEqualGUID(core::ptr::addr_of!((*drvInfoData).guidInfo), core::ptr::addr_of!(GUID_DDMoreSurfaceCaps)) != 0 {
        dwInfoSize = core::mem::size_of::<DDMORESURFACECAPS>() as DWORD;
        pInfo = core::ptr::addr_of_mut!((*peDdGl).ddMoreSurfaceCaps) as PVOID;
    }

    if dwInfoSize != 0 && !pInfo.is_null() {
        (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);
        intDdGetDriverInfo(
            peDdGl,
            (*drvInfoData).guidInfo,
            callbackStruct.as_mut_ptr() as PVOID,
            dwInfoSize,
            core::ptr::addr_of_mut!(dwInfoSize),
        );
        (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
        core::ptr::copy_nonoverlapping(
            callbackStruct.as_ptr(),
            (*drvInfoData).lpvData as *mut u8,
            dwInfoSize as usize,
        );
    }

    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    1
}

#[no_mangle]
pub unsafe extern "system" fn DxDdQueryDirectDrawObject(
    DdHandle: HANDLE,
    pDdHalInfo: *mut DD_HALINFO,
    pCallBackFlags: *mut DWORD,
    pd3dNtHalCallbacks: LPD3DNTHAL_CALLBACKS,
    pd3dNtGlobalDriverData: LPD3DNTHAL_GLOBALDRIVERDATA,
    pd3dBufCallbacks: PDD_D3DBUFCALLBACKS,
    pTextureFormats: LPDDSURFACEDESC,
    _p8: *mut DWORD,
    _p9: *mut VIDEOMEMORY,
    pdwNumFourCC: *mut DWORD,
    pdwFourCC: *mut DWORD,
) -> BOOL {
    let mut RetVal: BOOL = 0;

    if DdHandle.is_null() {
        return RetVal;
    }

    if pDdHalInfo.is_null() {
        return RetVal;
    }

    if (gpEngFuncs.DxEngScreenAccessCheck.unwrap())() == 0 {
        return RetVal;
    }

    let peDdL = DdHmgLock(DdHandle, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;
    if !peDdL.is_null() {
        let peDdGl = (*peDdL).peDirectDrawGlobal2;
        (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);

        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*peDdGl).ddHalInfo) as *const u8,
            pDdHalInfo as *mut u8,
            core::mem::size_of::<DD_HALINFO>(),
        );

        if !pCallBackFlags.is_null() {
            *pCallBackFlags = (*peDdGl).ddCallbacks.dwFlags;
            *((pCallBackFlags as ULONG_PTR).wrapping_add(4) as *mut DWORD) = (*peDdGl).ddSurfaceCallbacks.dwFlags;
            *((pCallBackFlags as ULONG_PTR).wrapping_add(8) as *mut DWORD) = (*peDdGl).ddPaletteCallbacks.dwFlags;
        }

        if !pd3dNtHalCallbacks.is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peDdGl).d3dNtHalCallbacks) as *const u8,
                pd3dNtHalCallbacks as *mut u8,
                core::mem::size_of_val(&(*peDdGl).d3dNtHalCallbacks),
            );
        }

        if !pd3dNtGlobalDriverData.is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peDdGl).d3dNtGlobalDriverData) as *const u8,
                pd3dNtGlobalDriverData as *mut u8,
                core::mem::size_of_val(&(*peDdGl).d3dNtGlobalDriverData),
            );
        }

        if !pd3dBufCallbacks.is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peDdGl).d3dBufCallbacks) as *const u8,
                pd3dBufCallbacks as *mut u8,
                core::mem::size_of_val(&(*peDdGl).d3dBufCallbacks),
            );
        }

        if !pTextureFormats.is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peDdGl).d3dNtGlobalDriverData.lpTextureFormats) as *const u8,
                pTextureFormats as *mut u8,
                (*peDdGl).d3dNtGlobalDriverData.dwNumTextureFormats as usize * core::mem::size_of::<DDSURFACEDESC2>(),
            );
        }

        if !pdwNumFourCC.is_null() {
            *pdwNumFourCC = (*peDdGl).dwNumFourCC;
        }

        if !pdwFourCC.is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((*peDdGl).pdwFourCC) as *const u8,
                pdwFourCC as *mut u8,
                4 * (*peDdGl).dwNumFourCC as usize,
            );
        }

        RetVal = 1;

        (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);

        InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);
    }

    RetVal
}

#[no_mangle]
pub unsafe extern "system" fn DxDdEnableDirectDraw(hDev: HANDLE, _arg2: BOOL) -> BOOL {
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;

    if (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev as HDEV, DxEGShDevData_display) != 0 {
        peDdGl = (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev as HDEV, DxEGShDevData_eddg) as PEDD_DIRECTDRAW_GLOBAL;
        (*peDdGl).hDev = hDev as HDEV;
        (*peDdGl).bSuspended = 0;
        (*peDdGl).dhpdev = (gpEngFuncs.DxEngGetHdevData.unwrap())(hDev as HDEV, DxEGShDevData_dhpdev) as PVOID;
        intDdEnableDriver(peDdGl);
        return 1;
    }

    0
}

#[no_mangle]
pub unsafe extern "system" fn DxDdReenableDirectDrawObject(
    DdHandle: HANDLE,
    _p2: PVOID,
) -> DWORD {
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let peDdGl: PEDD_DIRECTDRAW_GLOBAL;
    let hDC: HDC;
    let mut RetVal: DWORD = FALSE as DWORD;

    peDdL = DdHmgLock(DdHandle, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;

    if peDdL.is_null() {
        return RetVal;
    }

    peDdGl = (*peDdL).peDirectDrawGlobal2;

    hDC = (gpEngFuncs.DxEngGetDesktopDC.unwrap())(0, 0, 0);

    (gpEngFuncs.DxEngLockShareSem.unwrap())();
    (gpEngFuncs.DxEngLockHdev.unwrap())((*peDdGl).hDev);

    if ((*peDdGl).fl & 1) != 0
        && (gpEngFuncs.DxEngGetDCState.unwrap())(hDC, 2) != 1
        && (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_OpenRefs) == 0
        && (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_disable) == 0
        && (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_dd_locks) == 0
        && (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_DitherFmt) >= BMF_8BPP as DWORD_PTR
    {
        (*peDdGl).fl &= 0xFFFFFFFD;
        (*peDdGl).bSuspended = 0;
        (*peDdGl).dhpdev = (gpEngFuncs.DxEngGetHdevData.unwrap())((*peDdGl).hDev, DxEGShDevData_dhpdev) as PVOID;

        RetVal = TRUE as DWORD;
    }

    (gpEngFuncs.DxEngUnlockHdev.unwrap())((*peDdGl).hDev);
    (gpEngFuncs.DxEngUnlockShareSem.unwrap())();

    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    RetVal
}

#[no_mangle]
pub unsafe extern "system" fn intDdCreateNewSurfaceObject(
    peDdL: PEDD_DIRECTDRAW_LOCAL,
    hDirectDrawLocal: HANDLE,
    pDdSurfGlob: PDD_SURFACE_GLOBAL,
    pDdSurfLoc: PDD_SURFACE_LOCAL,
    pDdSurfMore: PDD_SURFACE_MORE,
) -> PEDD_SURFACE {
    let mut pSurface: PEDD_SURFACE = core::ptr::null_mut();

    if !hDirectDrawLocal.is_null() {
        pSurface = DdHmgLock(hDirectDrawLocal, ObjType_DDSURFACE_TYPE, 0) as PEDD_SURFACE;
        if !pSurface.is_null()
            && ((*pSurface).peDirectDrawLocal != peDdL || (*pSurface).hSecure.is_null())
        {
            InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);
            return core::ptr::null_mut();
        }
    }

    if pSurface.is_null() {
        pSurface = DdHmgAlloc(core::mem::size_of::<EDD_SURFACE>() as ULONG, ObjType_DDSURFACE_TYPE as CHAR, 1) as PEDD_SURFACE;
        if !pSurface.is_null() {
            (*pSurface).ddsSurfaceLocal.lpGbl = core::ptr::addr_of_mut!((*pSurface).ddsSurfaceGlobal);
            (*pSurface).ddsSurfaceLocal.lpSurfMore = core::ptr::addr_of_mut!((*pSurface).ddsSurfaceMore);
            (*pSurface).ddsSurfaceInt.lpLcl = core::ptr::addr_of_mut!((*pSurface).ddsSurfaceLocal);
            (*pSurface).peDirectDrawLocal = peDdL;
            (*pSurface).peDirectDrawGlobalNext = (*peDdL).peDirectDrawGlobal2;
            (*pSurface).ldev = (gpEngFuncs.DxEngGetHdevData.unwrap())((*(*pSurface).peDirectDrawGlobalNext).hDev, DxEGShDevData_ldev) as ULONG;
            (*pSurface).gdev = (gpEngFuncs.DxEngGetHdevData.unwrap())((*(*pSurface).peDirectDrawGlobalNext).hDev, DxEGShDevData_GDev) as ULONG;
            (*pSurface).hSecure = 1usize as HANDLE;
        }
    }

    if !pSurface.is_null() {
        (*pSurface).ddsSurfaceGlobal.fpVidMem = (*pDdSurfGlob).fpVidMem;
        (*pSurface).ddsSurfaceGlobal.__bindgen_anon_3.lPitch = (*pDdSurfGlob).__bindgen_anon_3.lPitch;
        (*pSurface).ddsSurfaceGlobal.wWidth = (*pDdSurfGlob).wWidth;
        (*pSurface).ddsSurfaceGlobal.wHeight = (*pDdSurfGlob).wHeight;
        (*pSurface).wWidth = (*pDdSurfGlob).wWidth as ULONG;
        (*pSurface).wHeight = (*pDdSurfGlob).wHeight as ULONG;
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*pDdSurfGlob).ddpfSurface) as *const u8,
            core::ptr::addr_of_mut!((*pSurface).ddsSurfaceGlobal.ddpfSurface) as *mut u8,
            core::mem::size_of_val(&(*pSurface).ddsSurfaceGlobal.ddpfSurface),
        );
        (*pSurface).ddsSurfaceLocal.ddsCaps.dwCaps = (*pDdSurfLoc).ddsCaps.dwCaps;
        (*pSurface).ddsSurfaceMore.ddsCapsEx.dwCaps2 = (*pDdSurfMore).ddsCapsEx.dwCaps2;
        (*pSurface).ddsSurfaceMore.ddsCapsEx.dwCaps3 = (*pDdSurfMore).ddsCapsEx.dwCaps3;
        (*pSurface).ddsSurfaceMore.ddsCapsEx.__bindgen_anon_1.dwCaps4 = (*pDdSurfMore).ddsCapsEx.__bindgen_anon_1.dwCaps4;
        (*pSurface).ddsSurfaceMore.dwSurfaceHandle = (*pDdSurfMore).dwSurfaceHandle;
        (*pSurface).hSecure = 1usize as HANDLE;

        (*peDdL).peSurface_DdList = pSurface;
        (*peDdL).hSurface = (*pSurface).pobj.hHmgr as ULONG_PTR as ULONG;
    }

    pSurface
}

#[no_mangle]
pub unsafe extern "system" fn DxDdCreateSurfaceObject(
    hDirectDrawLocal: HANDLE,
    hSurface: HANDLE,
    puSurfaceLocal: PDD_SURFACE_LOCAL,
    puSurfaceMore: PDD_SURFACE_MORE,
    puSurfaceGlobal: PDD_SURFACE_GLOBAL,
    _bComplete: BOOL,
) -> HANDLE {
    let mut RetVal: HANDLE = core::ptr::null_mut();
    let peDdL: PEDD_DIRECTDRAW_LOCAL;
    let pDdSurface: PEDD_SURFACE;

    peDdL = DdHmgLock(hDirectDrawLocal, ObjType_DDLOCAL_TYPE, 0) as PEDD_DIRECTDRAW_LOCAL;

    if peDdL.is_null() {
        return RetVal;
    }

    pDdSurface = intDdCreateNewSurfaceObject(peDdL, hSurface, puSurfaceGlobal, puSurfaceLocal, puSurfaceMore);
    if pDdSurface.is_null() {
        InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);
        return RetVal;
    }

    RetVal = (*pDdSurface).pobj.hHmgr;

    InterlockedDecrement(core::ptr::addr_of_mut!((*pDdSurface).pobj.cExclusiveLock) as *mut LONG);
    InterlockedDecrement(core::ptr::addr_of_mut!((*peDdL).pobj.cExclusiveLock) as *mut LONG);

    RetVal
}
