//! win32ss/reactx/dxg/dxg_driver.h — DXG dispatch-table stub functions
//! and the `gaDxgFuncs[]` DRVFN dispatch table (main.c wires it into pDxgDrv->pdrvfn).
//!
//! Every Dx* below is a trivial `TRACE(); return 0;` stub from the .h. The real
//! (bodied) Dx* functions live in sibling modules and are referenced by the table.

use win32k_sys::win32k::*;
use crate::types::*;

// Real (translated) dispatch targets defined in sibling modules.
use crate::d3d::DxDdCanCreateD3DBuffer;
use crate::dd::DxDdCanCreateSurface;
use crate::d3d::DxDdCreateD3DBuffer;
use crate::ddraw::DxDdCreateDirectDrawObject;
use crate::ddraw::DxDdCreateSurfaceObject;
use crate::ddraw::DxDdEnableDirectDraw;
use crate::ddraw::DxDdGetDriverInfo;
use crate::dxgallocator::DxDdHeapVidMemAllocAligned;
use crate::historic::DxDdIoctl;
use crate::ddsurf::DxDdLock;
use crate::eng::DxDdLockDirectDrawSurface;
use crate::ddraw::DxDdQueryDirectDrawObject;
use crate::ddraw::DxDdReenableDirectDrawObject;
use crate::ddsurf::DxDdUnlock;
use crate::eng::DxDdUnlockDirectDrawSurface;
use crate::historic::DxDxgGenericThunk;

// ============================================================================
// Stub functions (dxg_driver.h) — faithful `return 0;` thunks.
// ============================================================================

#[no_mangle]
pub unsafe extern "system" fn DxD3dContextCreate(_p1: PVOID, _p2: PVOID, _p3: PVOID, _p4: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxD3dContextDestroy(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxD3dContextDestroyAll(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxD3dValidateTextureStageState(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxD3dDrawPrimitives2(_p1: PVOID, _p2: PVOID, _p3: PVOID, _p4: PVOID, _p5: PVOID, _p6: PVOID, _p7: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetDriverState(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdAddAttachedSurface(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdAlphaBlt(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdAttachSurface(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdBeginMoCompFrame(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdBlt(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdColorControl(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdCreateMoComp(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDeleteDirectDrawObject(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDeleteSurfaceObject(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDestroyMoComp(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDestroySurface(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDestroyD3DBuffer(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdEndMoCompFrame(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdFlip(_p1: PVOID, _p2: PVOID, _p3: PVOID, _p4: PVOID, _p5: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdFlipToGDISurface(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetAvailDriverMemory(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetBltStatus(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetDC(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetDxHandle(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetFlipStatus(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetInternalMoCompInfo(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetMoCompBuffInfo(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetMoCompGuids(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetMoCompFormats(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetScanLine(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdLockD3D(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdQueryMoCompStatus(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdReleaseDC(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdRenderMoComp(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdResetVisrgn(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSetColorKey(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSetExclusiveMode(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSetGammaRamp(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdCreateSurfaceEx(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSetOverlayPosition(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdUnattachSurface(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdUnlockD3D(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdUpdateOverlay(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdWaitForVerticalBlank(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpCanCreateVideoPort(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpColorControl(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpCreateVideoPort(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpDestroyVideoPort(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpFlipVideoPort(_p1: PVOID, _p2: PVOID, _p3: PVOID, _p4: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortBandwidth(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortField(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortFlipStatus(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortInputFormats(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortLine(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortOutputFormats(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoPortConnectInfo(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpGetVideoSignalStatus(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpUpdateVideoPort(_p1: PVOID, _p2: PVOID, _p3: PVOID, _p4: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpWaitForVideoPortSync(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpAcquireNotification(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDvpReleaseNotification(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdHeapVidMemFree(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDisableDirectDraw(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSuspendDirectDraw(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdResumeDirectDraw(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdDynamicModeChange(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdCloseProcess(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetDirectDrawBound(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdEnableDirectDrawRedirection(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdAllocPrivateUserMem(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdFreePrivateUserMem(_p1: PVOID, _p2: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdSetAccelLevel(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdGetSurfaceLock(_p1: PVOID) -> DWORD { 0 }
#[no_mangle]
pub unsafe extern "system" fn DxDdEnumLockedSurfaceRect(_p1: PVOID, _p2: PVOID, _p3: PVOID) -> DWORD { 0 }

// ============================================================================
// gaDxgFuncs[] — DRVFN dispatch table (dxg_driver.h:774-868).
// Idiom matches win32k-rust ntddraw_dxeng.rs gaEngFuncs.
// ============================================================================
pub static mut gaDxgFuncs: [DRVFN; 92] = [
    DRVFN { iFunc: DXG_INDEX_DxDxgGenericThunk, pfn: Some(unsafe { core::mem::transmute(DxDxgGenericThunk as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxD3dContextCreate, pfn: Some(unsafe { core::mem::transmute(DxD3dContextCreate as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxD3dContextDestroy, pfn: Some(unsafe { core::mem::transmute(DxD3dContextDestroy as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxD3dContextDestroyAll, pfn: Some(unsafe { core::mem::transmute(DxD3dContextDestroyAll as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxD3dValidateTextureStageState, pfn: Some(unsafe { core::mem::transmute(DxD3dValidateTextureStageState as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxD3dDrawPrimitives2, pfn: Some(unsafe { core::mem::transmute(DxD3dDrawPrimitives2 as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetDriverState, pfn: Some(unsafe { core::mem::transmute(DxDdGetDriverState as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdAddAttachedSurface, pfn: Some(unsafe { core::mem::transmute(DxDdAddAttachedSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdAlphaBlt, pfn: Some(unsafe { core::mem::transmute(DxDdAlphaBlt as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdAttachSurface, pfn: Some(unsafe { core::mem::transmute(DxDdAttachSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdBeginMoCompFrame, pfn: Some(unsafe { core::mem::transmute(DxDdBeginMoCompFrame as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdBlt, pfn: Some(unsafe { core::mem::transmute(DxDdBlt as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCanCreateSurface, pfn: Some(unsafe { core::mem::transmute(DxDdCanCreateSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCanCreateD3DBuffer, pfn: Some(unsafe { core::mem::transmute(DxDdCanCreateD3DBuffer as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdColorControl, pfn: Some(unsafe { core::mem::transmute(DxDdColorControl as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateDirectDrawObject, pfn: Some(unsafe { core::mem::transmute(DxDdCreateDirectDrawObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateSurface, pfn: Some(unsafe { core::mem::transmute(DxDdCreateD3DBuffer as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateD3DBuffer, pfn: Some(unsafe { core::mem::transmute(DxDdCreateD3DBuffer as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateMoComp, pfn: Some(unsafe { core::mem::transmute(DxDdCreateMoComp as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateSurfaceObject, pfn: Some(unsafe { core::mem::transmute(DxDdCreateSurfaceObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDeleteDirectDrawObject, pfn: Some(unsafe { core::mem::transmute(DxDdDeleteDirectDrawObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDeleteSurfaceObject, pfn: Some(unsafe { core::mem::transmute(DxDdDeleteSurfaceObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDestroyMoComp, pfn: Some(unsafe { core::mem::transmute(DxDdDestroyMoComp as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDestroySurface, pfn: Some(unsafe { core::mem::transmute(DxDdDestroySurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDestroyD3DBuffer, pfn: Some(unsafe { core::mem::transmute(DxDdDestroyD3DBuffer as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdEndMoCompFrame, pfn: Some(unsafe { core::mem::transmute(DxDdEndMoCompFrame as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdFlip, pfn: Some(unsafe { core::mem::transmute(DxDdFlip as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdFlipToGDISurface, pfn: Some(unsafe { core::mem::transmute(DxDdFlipToGDISurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetAvailDriverMemory, pfn: Some(unsafe { core::mem::transmute(DxDdGetAvailDriverMemory as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetBltStatus, pfn: Some(unsafe { core::mem::transmute(DxDdGetBltStatus as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetDC, pfn: Some(unsafe { core::mem::transmute(DxDdGetDC as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetDriverInfo, pfn: Some(unsafe { core::mem::transmute(DxDdGetDriverInfo as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetDxHandle, pfn: Some(unsafe { core::mem::transmute(DxDdGetDxHandle as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetFlipStatus, pfn: Some(unsafe { core::mem::transmute(DxDdGetFlipStatus as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetInternalMoCompInfo, pfn: Some(unsafe { core::mem::transmute(DxDdGetInternalMoCompInfo as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetMoCompBuffInfo, pfn: Some(unsafe { core::mem::transmute(DxDdGetMoCompBuffInfo as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetMoCompGuids, pfn: Some(unsafe { core::mem::transmute(DxDdGetMoCompGuids as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetMoCompFormats, pfn: Some(unsafe { core::mem::transmute(DxDdGetMoCompFormats as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetScanLine, pfn: Some(unsafe { core::mem::transmute(DxDdGetScanLine as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdLock, pfn: Some(unsafe { core::mem::transmute(DxDdLock as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdLockD3D, pfn: Some(unsafe { core::mem::transmute(DxDdLockD3D as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdQueryDirectDrawObject, pfn: Some(unsafe { core::mem::transmute(DxDdQueryDirectDrawObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdQueryMoCompStatus, pfn: Some(unsafe { core::mem::transmute(DxDdQueryMoCompStatus as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdReenableDirectDrawObject, pfn: Some(unsafe { core::mem::transmute(DxDdReenableDirectDrawObject as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdReleaseDC, pfn: Some(unsafe { core::mem::transmute(DxDdReleaseDC as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdRenderMoComp, pfn: Some(unsafe { core::mem::transmute(DxDdRenderMoComp as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdResetVisrgn, pfn: Some(unsafe { core::mem::transmute(DxDdResetVisrgn as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSetColorKey, pfn: Some(unsafe { core::mem::transmute(DxDdSetColorKey as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSetExclusiveMode, pfn: Some(unsafe { core::mem::transmute(DxDdSetExclusiveMode as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSetGammaRamp, pfn: Some(unsafe { core::mem::transmute(DxDdSetGammaRamp as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCreateSurfaceEx, pfn: Some(unsafe { core::mem::transmute(DxDdCreateSurfaceEx as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSetOverlayPosition, pfn: Some(unsafe { core::mem::transmute(DxDdSetOverlayPosition as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdUnattachSurface, pfn: Some(unsafe { core::mem::transmute(DxDdUnattachSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdUnlock, pfn: Some(unsafe { core::mem::transmute(DxDdUnlock as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdUnlockD3D, pfn: Some(unsafe { core::mem::transmute(DxDdUnlockD3D as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdUpdateOverlay, pfn: Some(unsafe { core::mem::transmute(DxDdUpdateOverlay as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdWaitForVerticalBlank, pfn: Some(unsafe { core::mem::transmute(DxDdWaitForVerticalBlank as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpCanCreateVideoPort, pfn: Some(unsafe { core::mem::transmute(DxDvpCanCreateVideoPort as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpColorControl, pfn: Some(unsafe { core::mem::transmute(DxDvpColorControl as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpCreateVideoPort, pfn: Some(unsafe { core::mem::transmute(DxDvpCreateVideoPort as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpDestroyVideoPort, pfn: Some(unsafe { core::mem::transmute(DxDvpDestroyVideoPort as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpFlipVideoPort, pfn: Some(unsafe { core::mem::transmute(DxDvpFlipVideoPort as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortBandwidth, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortBandwidth as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortField, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortField as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortFlipStatus, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortFlipStatus as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortInputFormats, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortInputFormats as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortLine, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortLine as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortOutputFormats, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortOutputFormats as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoPortConnectInfo, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoPortConnectInfo as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpGetVideoSignalStatus, pfn: Some(unsafe { core::mem::transmute(DxDvpGetVideoSignalStatus as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpUpdateVideoPort, pfn: Some(unsafe { core::mem::transmute(DxDvpUpdateVideoPort as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpWaitForVideoPortSync, pfn: Some(unsafe { core::mem::transmute(DxDvpWaitForVideoPortSync as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpAcquireNotification, pfn: Some(unsafe { core::mem::transmute(DxDvpAcquireNotification as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDvpReleaseNotification, pfn: Some(unsafe { core::mem::transmute(DxDvpReleaseNotification as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdHeapVidMemAllocAligned, pfn: Some(unsafe { core::mem::transmute(DxDdHeapVidMemAllocAligned as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdHeapVidMemFree, pfn: Some(unsafe { core::mem::transmute(DxDdHeapVidMemFree as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdEnableDirectDraw, pfn: Some(unsafe { core::mem::transmute(DxDdEnableDirectDraw as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDisableDirectDraw, pfn: Some(unsafe { core::mem::transmute(DxDdDisableDirectDraw as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSuspendDirectDraw, pfn: Some(unsafe { core::mem::transmute(DxDdSuspendDirectDraw as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdResumeDirectDraw, pfn: Some(unsafe { core::mem::transmute(DxDdResumeDirectDraw as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdDynamicModeChange, pfn: Some(unsafe { core::mem::transmute(DxDdDynamicModeChange as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdCloseProcess, pfn: Some(unsafe { core::mem::transmute(DxDdCloseProcess as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetDirectDrawBound, pfn: Some(unsafe { core::mem::transmute(DxDdGetDirectDrawBound as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdEnableDirectDrawRedirection, pfn: Some(unsafe { core::mem::transmute(DxDdEnableDirectDrawRedirection as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdAllocPrivateUserMem, pfn: Some(unsafe { core::mem::transmute(DxDdAllocPrivateUserMem as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdFreePrivateUserMem, pfn: Some(unsafe { core::mem::transmute(DxDdFreePrivateUserMem as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdLockDirectDrawSurface, pfn: Some(unsafe { core::mem::transmute(DxDdLockDirectDrawSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdUnlockDirectDrawSurface, pfn: Some(unsafe { core::mem::transmute(DxDdUnlockDirectDrawSurface as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdSetAccelLevel, pfn: Some(unsafe { core::mem::transmute(DxDdSetAccelLevel as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdGetSurfaceLock, pfn: Some(unsafe { core::mem::transmute(DxDdGetSurfaceLock as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdEnumLockedSurfaceRect, pfn: Some(unsafe { core::mem::transmute(DxDdEnumLockedSurfaceRect as *const ()) }) },
    DRVFN { iFunc: DXG_INDEX_DxDdIoctl, pfn: Some(unsafe { core::mem::transmute(DxDdIoctl as *const ()) }) },
];

