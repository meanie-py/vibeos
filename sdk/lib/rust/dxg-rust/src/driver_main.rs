//! win32ss/reactx/dxg/main.c — dxg.sys driver entry + DxEng thunk-table setup.

use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::ExFreePoolWithTag;
use ntkrnl_sys::{STATUS_SUCCESS, STATUS_NO_MEMORY};
use ntkrnl_sys::constants::STATUS_BUFFER_TOO_SMALL;
use crate::types::*;
use crate::ddhmg::{DdHmgCreate, DdHmgDestroy};

// ---- file-scope globals owned by main.c (dxg_int.h:181-186) ----
pub static mut gcDummyPageRefCnt: LONG = 0;
pub static mut ghsemDummyPage: HSEMAPHORE = core::ptr::null_mut();
pub static mut gpDummyPage: PVOID = core::ptr::null_mut();
pub static mut gpepSession: PEPROCESS = core::ptr::null_mut();
pub static mut gpLockShortDelay: PLARGE_INTEGER = core::ptr::null_mut();

/// `DXENG_FUNCTIONS gpEngFuncs;` — the DxEng thunk table, populated in
/// DxDdStartupDxGraphics from the DRVFN[] array win32k.sys hands us. Every
/// pointer field starts None / null (C zero-init of a BSS global).
pub static mut gpEngFuncs: DXENG_FUNCTIONS = DXENG_FUNCTIONS {
    Reserved: core::ptr::null_mut(),
    DxEngNUIsTermSrv: None,
    DxEngScreenAccessCheck: None,
    DxEngRedrawDesktop: None,
    DxEngDispUniq: None,
    DxEngIncDispUniq: None,
    DxEngVisRgnUniq: None,
    DxEngLockShareSem: None,
    DxEngUnlockShareSem: None,
    DxEngEnumerateHdev: None,
    DxEngLockHdev: None,
    DxEngUnlockHdev: None,
    DxEngIsHdevLockedByCurrentThread: None,
    DxEngReferenceHdev: None,
    DxEngUnreferenceHdev: None,
    DxEngGetDeviceGammaRamp: None,
    DxEngSetDeviceGammaRamp: None,
    DxEngSpTearDownSprites: None,
    DxEngSpUnTearDownSprites: None,
    DxEngSpSpritesVisible: None,
    DxEngGetHdevData: None,
    DxEngSetHdevData: None,
    DxEngCreateMemoryDC: None,
    DxEngGetDesktopDC: None,
    DxEngDeleteDC: None,
    DxEngCleanDC: None,
    DxEngSetDCOwner: None,
    DxEngLockDC: None,
    DxEngUnlockDC: None,
    DxEngSetDCState: None,
    DxEngGetDCState: None,
    DxEngSelectBitmap: None,
    DxEngSetBitmapOwner: None,
    DxEngDeleteSurface: None,
    DxEngGetSurfaceData: None,
    DxEngAltLockSurface: None,
    DxEngUploadPaletteEntryToSurface: None,
    DxEngMarkSurfaceAsDirectDraw: None,
    DxEngSelectPaletteToSurface: None,
    DxEngSyncPaletteTableWithDevice: None,
    DxEngSetPaletteState: None,
    DxEngGetRedirectionBitmap: None,
    DxEngLoadImage: None,
};

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    _Context1: PVOID,
    _Context2: PVOID,
) -> NTSTATUS {
    0
}

#[no_mangle]
pub unsafe extern "system" fn DxDdStartupDxGraphics(
    SizeEngDrv: ULONG,
    pDxEngDrv: PDRVENABLEDATA,
    SizeDxgDrv: ULONG,
    pDxgDrv: PDRVENABLEDATA,
    DirectDrawContext: *mut ULONG,
    Proc: PEPROCESS,
) -> NTSTATUS {
    /* Test see if the data is valid we got from win32k.sys */
    if (SizeEngDrv != core::mem::size_of::<DRVENABLEDATA>() as ULONG)
        || (SizeDxgDrv != core::mem::size_of::<DRVENABLEDATA>() as ULONG)
    {
        return STATUS_BUFFER_TOO_SMALL;
    }

    /* reset static value */
    gpDummyPage = core::ptr::null_mut();
    gcDummyPageRefCnt = 0;
    ghsemDummyPage = core::ptr::null_mut();

    /*
     * Setup internal driver functions list we got from dxg driver functions list.
     *
     * NOTE: the C populates pDxgDrv->pdrvfn with gaDxgFuncs (dxg_driver.h), the
     * giant DRVFN[] dispatch table. That table is C glue in dxg_driver.h whose
     * ~90 entries are unported stub fns, so it stays on the C side of the cohab
     * build (the C TU that defines gaDxgFuncs assigns pdrvfn). We still publish
     * the version and count fields faithfully here.
     */
    (*pDxgDrv).iDriverVersion = 0x80000; /* DirectX 8 */
    (*pDxgDrv).c = DXG_INDEX_DxDdIoctl + 1;

    /* check how many driver functions and fail if the value does not match */
    if (*pDxEngDrv).c != DXENG_INDEX_DxEngLoadImage + 1 {
        return STATUS_INTERNAL_ERROR;
    }

    /*
     * Check if all drv functions are sorted right and if it really are exported,
     * copying each DxEng thunk into gpEngFuncs (treated as a flat PFN[] array,
     * exactly as the C does: `peng_funcs = (PFN*)&gpEngFuncs`).
     */
    let peng_funcs = core::ptr::addr_of_mut!(gpEngFuncs) as *mut PFN;

    let mut i: ULONG = 1;
    while i < DXENG_INDEX_DxEngLoadImage + 1 {
        let drv_func: PDRVFN = (*pDxEngDrv).pdrvfn.add(i as usize);

        if ((*drv_func).iFunc != i) || (*drv_func).pfn.is_none() {
            return STATUS_INTERNAL_ERROR;
        }

        *peng_funcs.add(i as usize) = (*drv_func).pfn;
        i += 1;
    }

    /* Note 12/1-2004 : Why is this set to 0x618 */
    *DirectDrawContext = 0x618;

    if DdHmgCreate() != 0 {
        ghsemDummyPage = EngCreateSemaphore();

        if !ghsemDummyPage.is_null() {
            gpepSession = Proc;
            return STATUS_SUCCESS;
        }
    }

    DdHmgDestroy();

    if !ghsemDummyPage.is_null() {
        EngDeleteSemaphore(ghsemDummyPage);
        ghsemDummyPage = core::ptr::null_mut();
    }

    STATUS_NO_MEMORY
}

#[no_mangle]
pub unsafe extern "system" fn DxDdCleanupDxGraphics() -> NTSTATUS {
    DdHmgDestroy();

    if !ghsemDummyPage.is_null() {
        if !gpDummyPage.is_null() {
            ExFreePoolWithTag(gpDummyPage, 0);
            gpDummyPage = core::ptr::null_mut();
            gcDummyPageRefCnt = 0;
        }
        EngDeleteSemaphore(ghsemDummyPage);
        ghsemDummyPage = core::ptr::null_mut();
    }

    0
}
