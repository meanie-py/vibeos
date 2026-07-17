use win32k_sys::win32k::*;
use crate::framebuf::*;
use crate::screen::DrvGetModes;
use crate::surface::{DrvEnableSurface, DrvDisableSurface, DrvAssertMode};
use crate::palette::DrvSetPalette;
use crate::pointer::{DrvSetPointerShape, DrvMovePointer};

/// Reinterpret a DDI function as the engine's generic `PFN` slot. This is the
/// faithful Rust equivalent of the C driver's `(PFN)DrvFoo` reinterpret-cast:
/// each Drv* fn has a distinct signature, and the engine dispatches through
/// the index table with the correct prototype. `as PFN` won't compile (fn
/// item → `Option<fn>` is not a primitive cast), so we transmute a thin
/// pointer (both are pointer-sized; transmute is const-evaluable). The
/// `DrvFoo as *const ()` cast coerces the fn item to a fn pointer first.
macro_rules! pfn {
    ($f:path) => {
        unsafe { core::mem::transmute::<*const (), PFN>($f as *const ()) }
    };
}

static mut DrvFunctionTable: [DRVFN; 12] = [
    DRVFN { iFunc: INDEX_DrvEnablePDEV, pfn: pfn!(DrvEnablePDEV) },
    DRVFN { iFunc: INDEX_DrvCompletePDEV, pfn: pfn!(DrvCompletePDEV) },
    DRVFN { iFunc: INDEX_DrvDisablePDEV, pfn: pfn!(DrvDisablePDEV) },
    DRVFN { iFunc: INDEX_DrvEnableSurface, pfn: pfn!(DrvEnableSurface) },
    DRVFN { iFunc: INDEX_DrvDisableSurface, pfn: pfn!(DrvDisableSurface) },
    DRVFN { iFunc: INDEX_DrvAssertMode, pfn: pfn!(DrvAssertMode) },
    DRVFN { iFunc: INDEX_DrvGetModes, pfn: pfn!(DrvGetModes) },
    DRVFN { iFunc: INDEX_DrvSetPalette, pfn: pfn!(DrvSetPalette) },
    DRVFN { iFunc: INDEX_DrvSetPointerShape, pfn: pfn!(DrvSetPointerShape) },
    DRVFN { iFunc: INDEX_DrvMovePointer, pfn: pfn!(DrvMovePointer) },
    DRVFN { iFunc: INDEX_DrvEnableDirectDraw, pfn: pfn!(DrvEnableDirectDraw) },
    DRVFN { iFunc: INDEX_DrvDisableDirectDraw, pfn: pfn!(DrvDisableDirectDraw) },
];

#[no_mangle]
pub unsafe extern "system" fn DrvEnableDirectDraw(
    _dhpdev: DHPDEV,
    pCallbacks: *mut DD_CALLBACKS,
    pSurfaceCallbacks: *mut DD_SURFACECALLBACKS,
    pPaletteCallbacks: *mut DD_PALETTECALLBACKS,
) -> u32 {
    RtlZeroMemory(
        pCallbacks as *mut core::ffi::c_void,
        core::mem::size_of::<DD_CALLBACKS>(),
    );
    RtlZeroMemory(
        pSurfaceCallbacks as *mut core::ffi::c_void,
        core::mem::size_of::<DD_SURFACECALLBACKS>(),
    );
    RtlZeroMemory(
        pPaletteCallbacks as *mut core::ffi::c_void,
        core::mem::size_of::<DD_PALETTECALLBACKS>(),
    );

    (*pCallbacks).dwSize = core::mem::size_of::<DD_CALLBACKS>() as u32;
    (*pSurfaceCallbacks).dwSize = core::mem::size_of::<DD_SURFACECALLBACKS>() as u32;
    (*pPaletteCallbacks).dwSize = core::mem::size_of::<DD_PALETTECALLBACKS>() as u32;

    1u32
}

#[no_mangle]
pub unsafe extern "system" fn DrvDisableDirectDraw(_dhpdev: DHPDEV) {}

#[no_mangle]
pub unsafe extern "system" fn DrvEnableDriver(
    _iEngineVersion: u32,
    cj: u32,
    pded: PDRVENABLEDATA,
) -> u32 {
    if cj >= core::mem::size_of::<DRVENABLEDATA>() as u32 {
        (*pded).c = (core::mem::size_of::<[DRVFN; 12]>() / core::mem::size_of::<DRVFN>()) as u32;
        (*pded).pdrvfn = core::ptr::addr_of_mut!(DrvFunctionTable) as *mut DRVFN;
        (*pded).iDriverVersion = DDI_DRIVER_VERSION_NT5;
        1u32
    } else {
        0u32
    }
}

#[no_mangle]
pub unsafe extern "system" fn DrvEnablePDEV(
    pdm: *mut DEVMODEW,
    _pwszLogAddress: LPWSTR,
    _cPat: u32,
    _phsurfPatterns: *mut HSURF,
    cjCaps: u32,
    pdevcaps: *mut u32,
    cjDevInfo: u32,
    pdi: *mut DEVINFO,
    _hdev: HDEV,
    _pwszDeviceName: LPWSTR,
    hDriver: *mut core::ffi::c_void,
) -> DHPDEV {
    let mut GdiInfo: GDIINFO = core::mem::zeroed();
    let mut DevInfo: DEVINFO = core::mem::zeroed();

    let ppdev = EngAllocMem(
        FL_ZERO_MEMORY,
        core::mem::size_of::<PDEV>() as ULONG,
        crate::framebuf::ALLOC_TAG,
    ) as *mut crate::framebuf::PDEV;
    
    if ppdev.is_null() {
        return core::ptr::null_mut();
    }

    (*ppdev).hDriver = hDriver;

    if crate::screen::IntInitScreenInfo(ppdev, pdm, &mut GdiInfo, &mut DevInfo) == 0 {
        EngFreeMem(ppdev as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    if crate::palette::IntInitDefaultPalette(ppdev, &mut DevInfo) == 0 {
        EngFreeMem(ppdev as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    let copy_size_devinfo = core::cmp::min(
        core::mem::size_of::<DEVINFO>(),
        cjDevInfo as usize,
    );
    core::ptr::copy_nonoverlapping(
        &DevInfo as *const DEVINFO as *const u8,
        pdi as *mut u8,
        copy_size_devinfo,
    );

    let copy_size_gdiinfo = core::cmp::min(
        core::mem::size_of::<GDIINFO>(),
        cjCaps as usize,
    );
    core::ptr::copy_nonoverlapping(
        &GdiInfo as *const GDIINFO as *const u8,
        pdevcaps as *mut u8,
        copy_size_gdiinfo,
    );

    ppdev as DHPDEV
}

#[no_mangle]
pub unsafe extern "system" fn DrvCompletePDEV(dhpdev: DHPDEV, hdev: HDEV) {
    (*(dhpdev as *mut crate::framebuf::PDEV)).hDevEng = hdev;
}

#[no_mangle]
pub unsafe extern "system" fn DrvDisablePDEV(dhpdev: DHPDEV) {
    let ppdev = dhpdev as *mut crate::framebuf::PDEV;

    if !(*ppdev).DefaultPalette.is_null() {
        EngDeletePalette((*ppdev).DefaultPalette);
    }

    if !(*ppdev).PaletteEntries.is_null() {
        EngFreeMem((*ppdev).PaletteEntries as *mut core::ffi::c_void);
    }

    EngFreeMem(dhpdev as *mut core::ffi::c_void);
}