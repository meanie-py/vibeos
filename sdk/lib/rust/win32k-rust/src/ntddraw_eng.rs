use win32k_sys::win32k::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// No file-scope declarations to translate.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn HeapVidMemAllocAligned(
    lpVidMem: LPVIDMEM,
    dwWidth: u32,
    dwHeight: u32,
    lpAlignment: LPSURFACEALIGNMENT,
    lpNewPitch: LPLONG,
) -> FLATPTR {
    let pfnHeapVidMemAllocAligned: PGD_HEAPVIDMEMALLOCALIGNED =
        gpDxFuncs[DXG_INDEX_DxDdHeapVidMemAllocAligned as usize].pfn as PGD_HEAPVIDMEMALLOCALIGNED;

    if pfnHeapVidMemAllocAligned.is_none() {
        return 0;
    }

    (pfnHeapVidMemAllocAligned.unwrap())(lpVidMem, dwWidth, dwHeight, lpAlignment, lpNewPitch)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn HeapVidMemAllocAligned(
    lpVidMem: LPVIDMEM,
    dwWidth: u32,
    dwHeight: u32,
    lpAlignment: LPSURFACEALIGNMENT,
    lpNewPitch: LPLONG,
) -> FLATPTR {
    let pfnHeapVidMemAllocAligned: PGD_HEAPVIDMEMALLOCALIGNED =
        gpDxFuncs[DXG_INDEX_DxDdHeapVidMemAllocAligned as usize].pfn as PGD_HEAPVIDMEMALLOCALIGNED;

    if pfnHeapVidMemAllocAligned.is_none() {
        return 0;
    }

    (pfnHeapVidMemAllocAligned.unwrap())(lpVidMem, dwWidth, dwHeight, lpAlignment, lpNewPitch)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn VidMemFree(pvmh: LPVMEMHEAP, ptr: FLATPTR) {
    let pfnVidMemFree: PGD_VIDMEMFREE = core::mem::transmute(
        gpDxFuncs
            .add(DXG_INDEX_DxDdHeapVidMemFree as usize)
            .as_ref()
            .unwrap()
            .pfn,
    );

    if pfnVidMemFree.is_none() {
    } else {
        (pfnVidMemFree.unwrap())(pvmh, ptr);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn VidMemFree(pvmh: LPVMEMHEAP, ptr: FLATPTR) {
    let pfnVidMemFree: PGD_VIDMEMFREE = core::mem::transmute(
        gpDxFuncs
            .add(DXG_INDEX_DxDdHeapVidMemFree as usize)
            .as_ref()
            .unwrap()
            .pfn,
    );

    if pfnVidMemFree.is_none() {
    } else {
        (pfnVidMemFree.unwrap())(pvmh, ptr);
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngAllocPrivateUserMem(
    psl: PDD_SURFACE_LOCAL,
    cjMemSize: SIZE_T,
    ulTag: ULONG,
) -> PVOID {
    let pfnEngAllocPrivateUserMem: PGD_ENGALLOCPRIVATEUSERMEM =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdAllocPrivateUserMem as usize].pfn);

    if pfnEngAllocPrivateUserMem.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as PVOID;
    }

    (pfnEngAllocPrivateUserMem.unwrap())(psl, cjMemSize, ulTag)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngAllocPrivateUserMem(
    psl: PDD_SURFACE_LOCAL,
    cjMemSize: SIZE_T,
    ulTag: ULONG,
) -> PVOID {
    let pfnEngAllocPrivateUserMem: PGD_ENGALLOCPRIVATEUSERMEM =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdAllocPrivateUserMem as usize].pfn);

    if pfnEngAllocPrivateUserMem.is_none() {
        return DDHAL_DRIVER_NOTHANDLED as PVOID;
    }

    (pfnEngAllocPrivateUserMem.unwrap())(psl, cjMemSize, ulTag)
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngFreePrivateUserMem(psl: PDD_SURFACE_LOCAL, pv: *mut core::ffi::c_void) {
    let pfnEngFreePrivateUserMem: PGD_ENGFREEPRIVATEUSERMEM =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdFreePrivateUserMem as usize].pfn);

    if pfnEngFreePrivateUserMem.is_none() {
    } else {
        (pfnEngFreePrivateUserMem.unwrap())(psl, pv);
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngFreePrivateUserMem(psl: PDD_SURFACE_LOCAL, pv: *mut core::ffi::c_void) {
    let pfnEngFreePrivateUserMem: PGD_ENGFREEPRIVATEUSERMEM =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdFreePrivateUserMem as usize].pfn);

    if pfnEngFreePrivateUserMem.is_none() {
    } else {
        (pfnEngFreePrivateUserMem.unwrap())(psl, pv);
    }
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngDxIoctl(
    ulIoctl: ULONG,
    pBuffer: PVOID,
    ulBufferSize: ULONG,
) -> HRESULT {
    let pfnEngDxIoctl: PGD_ENGDXIOCTL =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdIoctl as usize].pfn);
    let mut retVal: DWORD = DDERR_UNSUPPORTED as DWORD;

    if let Some(pfnEngDxIoctl) = pfnEngDxIoctl {
        retVal = pfnEngDxIoctl(ulIoctl, pBuffer, ulBufferSize);
    }

    retVal as HRESULT
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngDxIoctl(
    ulIoctl: ULONG,
    pBuffer: PVOID,
    ulBufferSize: ULONG,
) -> HRESULT {
    let pfnEngDxIoctl: PGD_ENGDXIOCTL =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdIoctl as usize].pfn);
    let mut retVal: DWORD = DDERR_UNSUPPORTED as DWORD;

    if let Some(pfnEngDxIoctl) = pfnEngDxIoctl {
        retVal = pfnEngDxIoctl(ulIoctl, pBuffer, ulBufferSize);
    }

    retVal as HRESULT
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngLockDirectDrawSurface(hSurface: HANDLE) -> PDD_SURFACE_LOCAL {
    let pfnEngLockDirectDrawSurface: PGD_ENGLOCKDIRECTDRAWSURFACE =
        gpDxFuncs[DXG_INDEX_DxDdLockDirectDrawSurface as usize].pfn as PGD_ENGLOCKDIRECTDRAWSURFACE;
    let mut retVal: PDD_SURFACE_LOCAL = core::ptr::null_mut();

    if let Some(pfn) = pfnEngLockDirectDrawSurface {
        retVal = pfn(hSurface);
    }

    retVal
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngLockDirectDrawSurface(hSurface: HANDLE) -> PDD_SURFACE_LOCAL {
    let pfnEngLockDirectDrawSurface: PGD_ENGLOCKDIRECTDRAWSURFACE =
        gpDxFuncs[DXG_INDEX_DxDdLockDirectDrawSurface as usize].pfn as PGD_ENGLOCKDIRECTDRAWSURFACE;
    let mut retVal: PDD_SURFACE_LOCAL = core::ptr::null_mut();

    if let Some(pfn) = pfnEngLockDirectDrawSurface {
        retVal = pfn(hSurface);
    }

    retVal
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "stdcall" fn EngUnlockDirectDrawSurface(pSurface: PDD_SURFACE_LOCAL) -> i32 {
    let pfnEngUnlockDirectDrawSurface: PGD_ENGUNLOCKDIRECTDRAWSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUnlockDirectDrawSurface as usize].pfn);
    let mut retVal: i32 = 0;

    if let Some(pfn) = pfnEngUnlockDirectDrawSurface {
        retVal = pfn(pSurface);
    }

    retVal
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EngUnlockDirectDrawSurface(pSurface: PDD_SURFACE_LOCAL) -> i32 {
    let pfnEngUnlockDirectDrawSurface: PGD_ENGUNLOCKDIRECTDRAWSURFACE =
        core::mem::transmute(gpDxFuncs[DXG_INDEX_DxDdUnlockDirectDrawSurface as usize].pfn);
    let mut retVal: i32 = 0;

    if let Some(pfn) = pfnEngUnlockDirectDrawSurface {
        retVal = pfn(pSurface);
    }

    retVal
}
