//! Rust translation of `win32ss/gdi/ntgdi/d3dkmt.c` — D3DKMT dxgkrnl callbacks.
//! [Phase 2c wave 22: ntgdi D3DKMT / dxgkrnl thunk layer]
//!
//! ReactOS does not yet host a WDDM dxgkrnl, so this file is a thin syscall
//! surface over a callback table (`REACTOS_WIN32K_DXGKRNL_INTERFACE`) that is
//! obtained from dxgkrnl via an IOCTL. `DxStartupDxgkInt` (the populator) is a
//! TODO stub, so the table is permanently all-NULL and every adapter thunk
//! returns `STATUS_PROCEDURE_NOT_FOUND`. The structure is preserved verbatim so
//! that wiring up dxgkrnl later is a one-line change (populate the static).

use win32k_sys::win32k::*;

const STATUS_PROCEDURE_NOT_FOUND: NTSTATUS = 0xC000007Au32 as NTSTATUS;

/// One entry of the win32k<->dxgkrnl callback table. Each dxgkrnl adapter
/// routine is `NTSTATUS (APIENTRY *)(pointer)`; the table never stores a
/// non-null entry today (see module docs), and `Option<fn>` is null-niched so a
/// zeroed table is all-`None` — matching the C `= {0}` initializer.
type DxgPfn = Option<unsafe extern "system" fn(*mut core::ffi::c_void) -> NTSTATUS>;

/// `REACTOS_WIN32K_DXGKRNL_INTERFACE` (from `reactos/rddm/rxgkinterface.h`,
/// which only d3dkmt.c includes, so it is absent from the win32k-sys bindgen).
struct ReactosWin32kDxgkrnlInterface {
    RxgkIntPfnCreateAllocation: DxgPfn,
    RxgkIntPfnCheckMonitorPowerState: DxgPfn,
    RxgkIntPfnCheckOcclusion: DxgPfn,
    RxgkIntPfnCloseAdapter: DxgPfn,
    RxgkIntPfnCreateContext: DxgPfn,
    RxgkIntPfnCreateDevice: DxgPfn,
    RxgkIntPfnCreateOverlay: DxgPfn,
    RxgkIntPfnCreateSynchronizationObject: DxgPfn,
    RxgkIntPfnDestroyAllocation: DxgPfn,
    RxgkIntPfnDestroyContext: DxgPfn,
    RxgkIntPfnDestroyDevice: DxgPfn,
    RxgkIntPfnDestroyOverlay: DxgPfn,
    RxgkIntPfnDestroySynchronizationObject: DxgPfn,
    RxgkIntPfnEscape: DxgPfn,
    RxgkIntPfnFlipOverlay: DxgPfn,
    RxgkIntPfnGetContextSchedulingPriority: DxgPfn,
    RxgkIntPfnGetDeviceState: DxgPfn,
    RxgkIntPfnGetDisplayModeList: DxgPfn,
    RxgkIntPfnGetMultisampleMethodList: DxgPfn,
    RxgkIntPfnGetPresentHistory: DxgPfn,
    RxgkIntPfnGetRuntimeData: DxgPfn,
    RxgkIntPfnGetScanLine: DxgPfn,
    RxgkIntPfnGetSharedPrimaryHandle: DxgPfn,
    RxgkIntPfnInvalidateActiveVidPn: DxgPfn,
    RxgkIntPfnLock: DxgPfn,
    RxgkIntPfnOpenResource: DxgPfn,
    RxgkIntPfnPollDisplayChildren: DxgPfn,
    RxgkIntPfnPresent: DxgPfn,
    RxgkIntPfnQueryAdapterInfo: DxgPfn,
    RxgkIntPfnQueryAllocationResidency: DxgPfn,
    RxgkIntPfnQueryResourceInfo: DxgPfn,
    RxgkIntPfnQueryStatistics: DxgPfn,
    RxgkIntPfnReleaseProcessVidPnSourceOwners: DxgPfn,
    RxgkIntPfnRender: DxgPfn,
    RxgkIntPfnSetAllocationPriority: DxgPfn,
    RxgkIntPfnSetContextSchedulingPriority: DxgPfn,
    RxgkIntPfnSetDisplayMode: DxgPfn,
    RxgkIntPfnSetDisplayPrivateDriverFormat: DxgPfn,
    RxgkIntPfnSetGammaRamp: DxgPfn,
    RxgkIntPfnSetQueuedLimit: DxgPfn,
    RxgkIntPfnSetVidPnSourceOwner: DxgPfn,
    RxgkIntPfnUnlock: DxgPfn,
    RxgkIntPfnUpdateOverlay: DxgPfn,
    RxgkIntPfnWaitForIdle: DxgPfn,
    RxgkIntPfnWaitForSynchronizationObject: DxgPfn,
    RxgkIntPfnWaitForVerticalBlankEvent: DxgPfn,
    RxgkIntPfnSignalSynchronizationObject: DxgPfn,
}

/// `static REACTOS_WIN32K_DXGKRNL_INTERFACE DxgAdapterCallbacks = {0};`
static DXG_ADAPTER_CALLBACKS: ReactosWin32kDxgkrnlInterface =
    ReactosWin32kDxgkrnlInterface {
        RxgkIntPfnCreateAllocation: None,
        RxgkIntPfnCheckMonitorPowerState: None,
        RxgkIntPfnCheckOcclusion: None,
        RxgkIntPfnCloseAdapter: None,
        RxgkIntPfnCreateContext: None,
        RxgkIntPfnCreateDevice: None,
        RxgkIntPfnCreateOverlay: None,
        RxgkIntPfnCreateSynchronizationObject: None,
        RxgkIntPfnDestroyAllocation: None,
        RxgkIntPfnDestroyContext: None,
        RxgkIntPfnDestroyDevice: None,
        RxgkIntPfnDestroyOverlay: None,
        RxgkIntPfnDestroySynchronizationObject: None,
        RxgkIntPfnEscape: None,
        RxgkIntPfnFlipOverlay: None,
        RxgkIntPfnGetContextSchedulingPriority: None,
        RxgkIntPfnGetDeviceState: None,
        RxgkIntPfnGetDisplayModeList: None,
        RxgkIntPfnGetMultisampleMethodList: None,
        RxgkIntPfnGetPresentHistory: None,
        RxgkIntPfnGetRuntimeData: None,
        RxgkIntPfnGetScanLine: None,
        RxgkIntPfnGetSharedPrimaryHandle: None,
        RxgkIntPfnInvalidateActiveVidPn: None,
        RxgkIntPfnLock: None,
        RxgkIntPfnOpenResource: None,
        RxgkIntPfnPollDisplayChildren: None,
        RxgkIntPfnPresent: None,
        RxgkIntPfnQueryAdapterInfo: None,
        RxgkIntPfnQueryAllocationResidency: None,
        RxgkIntPfnQueryResourceInfo: None,
        RxgkIntPfnQueryStatistics: None,
        RxgkIntPfnReleaseProcessVidPnSourceOwners: None,
        RxgkIntPfnRender: None,
        RxgkIntPfnSetAllocationPriority: None,
        RxgkIntPfnSetContextSchedulingPriority: None,
        RxgkIntPfnSetDisplayMode: None,
        RxgkIntPfnSetDisplayPrivateDriverFormat: None,
        RxgkIntPfnSetGammaRamp: None,
        RxgkIntPfnSetQueuedLimit: None,
        RxgkIntPfnSetVidPnSourceOwner: None,
        RxgkIntPfnUnlock: None,
        RxgkIntPfnUpdateOverlay: None,
        RxgkIntPfnWaitForIdle: None,
        RxgkIntPfnWaitForSynchronizationObject: None,
        RxgkIntPfnWaitForVerticalBlankEvent: None,
        RxgkIntPfnSignalSynchronizationObject: None,
    };

/// `DxStartupDxgkInt` — let dxgkrnl know it is time to start all adapters and
/// obtain the win32k<->dxgkrnl interface via an IOCTL. TODO: CORE-20027.
#[no_mangle]
pub unsafe extern "system" fn DxStartupDxgkInt() {
    // DPRINT("DxStartupDxgkInt: Entry\n");
}

/// We do not support DWM, so exclusive ownership is always FALSE.
#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICheckExclusiveOwnership() -> BOOLEAN {
    FALSE as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetProcessSchedulingPriorityClass(
    _unnamedParam1: HANDLE,
    _unnamedParam2: *mut D3DKMT_SCHEDULINGPRIORITYCLASS,
) -> NTSTATUS {
    1
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetProcessSchedulingPriorityClass(
    _unnamedParam1: HANDLE,
    _unnamedParam2: D3DKMT_SCHEDULINGPRIORITYCLASS,
) -> NTSTATUS {
    1
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISharedPrimaryLockNotification(
    _unnamedParam1: *const D3DKMT_SHAREDPRIMARYLOCKNOTIFICATION,
) -> NTSTATUS {
    1
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISharedPrimaryUnLockNotification(
    _unnamedParam1: *const D3DKMT_SHAREDPRIMARYUNLOCKNOTIFICATION,
) -> NTSTATUS {
    1
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIOpenAdapterFromGdiDisplayName(
    _unnamedParam1: *mut D3DKMT_OPENADAPTERFROMGDIDISPLAYNAME,
) -> NTSTATUS {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIOpenAdapterFromHdc(
    _unnamedParam1: *mut D3DKMT_OPENADAPTERFROMHDC,
) -> NTSTATUS {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIOpenAdapterFromDeviceName(
    _unnamedParam1: *mut D3DKMT_OPENADAPTERFROMDEVICENAME,
) -> NTSTATUS {
    0
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateAllocation(unnamedParam1: *mut D3DKMT_CREATEALLOCATION) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCreateAllocation {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICheckMonitorPowerState(unnamedParam1: *const D3DKMT_CHECKMONITORPOWERSTATE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCheckMonitorPowerState {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICheckOcclusion(unnamedParam1: *const D3DKMT_CHECKOCCLUSION) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCheckOcclusion {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICloseAdapter(unnamedParam1: *const D3DKMT_CLOSEADAPTER) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCloseAdapter {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateContext(unnamedParam1: *mut D3DKMT_CREATECONTEXT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCreateContext {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateDevice(unnamedParam1: *mut D3DKMT_CREATEDEVICE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCreateDevice {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateOverlay(unnamedParam1: *mut D3DKMT_CREATEOVERLAY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCreateOverlay {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDICreateSynchronizationObject(unnamedParam1: *mut D3DKMT_CREATESYNCHRONIZATIONOBJECT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnCreateSynchronizationObject {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroyAllocation(unnamedParam1: *const D3DKMT_DESTROYALLOCATION) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnDestroyAllocation {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroyContext(unnamedParam1: *const D3DKMT_DESTROYCONTEXT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnDestroyContext {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroyDevice(unnamedParam1: *const D3DKMT_DESTROYDEVICE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnDestroyDevice {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroyOverlay(unnamedParam1: *const D3DKMT_DESTROYOVERLAY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnDestroyOverlay {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIDestroySynchronizationObject(unnamedParam1: *const D3DKMT_DESTROYSYNCHRONIZATIONOBJECT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnDestroySynchronizationObject {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIEscape(unnamedParam1: *const D3DKMT_ESCAPE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnEscape {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIFlipOverlay(unnamedParam1: *const D3DKMT_FLIPOVERLAY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnFlipOverlay {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetContextSchedulingPriority(unnamedParam1: *mut D3DKMT_GETCONTEXTSCHEDULINGPRIORITY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetContextSchedulingPriority {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetDeviceState(unnamedParam1: *mut D3DKMT_GETDEVICESTATE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetDeviceState {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetDisplayModeList(unnamedParam1: *mut D3DKMT_GETDISPLAYMODELIST) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetDisplayModeList {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetMultisampleMethodList(unnamedParam1: *mut D3DKMT_GETMULTISAMPLEMETHODLIST) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetMultisampleMethodList {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetPresentHistory(unnamedParam1: *mut D3DKMT_GETPRESENTHISTORY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetPresentHistory {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetRuntimeData(unnamedParam1: *const D3DKMT_GETRUNTIMEDATA) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetRuntimeData {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetScanLine(unnamedParam1: *mut D3DKMT_GETSCANLINE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetScanLine {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIGetSharedPrimaryHandle(unnamedParam1: *mut D3DKMT_GETSHAREDPRIMARYHANDLE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnGetSharedPrimaryHandle {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIInvalidateActiveVidPn(unnamedParam1: *const D3DKMT_INVALIDATEACTIVEVIDPN) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnInvalidateActiveVidPn {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDILock(unnamedParam1: *mut D3DKMT_LOCK) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnLock {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIOpenResource(unnamedParam1: *mut D3DKMT_OPENRESOURCE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnOpenResource {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIPollDisplayChildren(unnamedParam1: *const D3DKMT_POLLDISPLAYCHILDREN) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnPollDisplayChildren {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIPresent(unnamedParam1: *mut D3DKMT_PRESENT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnPresent {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIQueryAdapterInfo(unnamedParam1: *const D3DKMT_QUERYADAPTERINFO) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnQueryAdapterInfo {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIQueryAllocationResidency(unnamedParam1: *const D3DKMT_QUERYALLOCATIONRESIDENCY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnQueryAllocationResidency {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIQueryResourceInfo(unnamedParam1: *mut D3DKMT_QUERYRESOURCEINFO) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnQueryResourceInfo {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIQueryStatistics(unnamedParam1: *const D3DKMT_QUERYSTATISTICS) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnQueryStatistics {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIReleaseProcessVidPnSourceOwners(unnamedParam1: HANDLE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnReleaseProcessVidPnSourceOwners {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIRender(unnamedParam1: *mut D3DKMT_RENDER) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnRender {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetAllocationPriority(unnamedParam1: *const D3DKMT_SETALLOCATIONPRIORITY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetAllocationPriority {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetContextSchedulingPriority(unnamedParam1: *const D3DKMT_SETCONTEXTSCHEDULINGPRIORITY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetContextSchedulingPriority {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetDisplayMode(unnamedParam1: *const D3DKMT_SETDISPLAYMODE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetDisplayMode {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetDisplayPrivateDriverFormat(unnamedParam1: *const D3DKMT_SETDISPLAYPRIVATEDRIVERFORMAT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetDisplayPrivateDriverFormat {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetGammaRamp(unnamedParam1: *const D3DKMT_SETGAMMARAMP) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetGammaRamp {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetQueuedLimit(unnamedParam1: *const D3DKMT_SETQUEUEDLIMIT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetQueuedLimit {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISetVidPnSourceOwner(unnamedParam1: *const D3DKMT_SETVIDPNSOURCEOWNER) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSetVidPnSourceOwner {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIUnlock(unnamedParam1: *const D3DKMT_UNLOCK) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnUnlock {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIUpdateOverlay(unnamedParam1: *const D3DKMT_UPDATEOVERLAY) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnUpdateOverlay {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIWaitForIdle(unnamedParam1: *const D3DKMT_WAITFORIDLE) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnWaitForIdle {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIWaitForSynchronizationObject(unnamedParam1: *const D3DKMT_WAITFORSYNCHRONIZATIONOBJECT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnWaitForSynchronizationObject {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDIWaitForVerticalBlankEvent(unnamedParam1: *const D3DKMT_WAITFORVERTICALBLANKEVENT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnWaitForVerticalBlankEvent {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

#[no_mangle]
pub unsafe extern "system" fn NtGdiDdDDISignalSynchronizationObject(unnamedParam1: *const D3DKMT_SIGNALSYNCHRONIZATIONOBJECT) -> NTSTATUS {
    // C had a no-op `if (!unnamedParam1) STATUS_INVALID_PARAMETER;` (missing
    // `return`) — dead, omitted.
    match DXG_ADAPTER_CALLBACKS.RxgkIntPfnSignalSynchronizationObject {
        None => STATUS_PROCEDURE_NOT_FOUND,
        Some(pfn) => pfn(unnamedParam1 as *mut core::ffi::c_void),
    }
}

