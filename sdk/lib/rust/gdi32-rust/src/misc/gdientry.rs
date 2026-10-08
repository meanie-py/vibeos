//! misc/gdientry.c → Rust. DirectDraw GDI entry points + DDHAL callbacks.
//
// Wave-4. Same crate/substrate as Waves 1a/1b/2/2b/3a/3b. Imports:
//   `use win32k_sys::win32k::*;` + `use crate::gdihelp::*;`
//
// ⚠️ CRITICAL NAMING (ddrawgdi.h #define map): the 15 "GDI entry" exports are
//   #define'd to GdiEntry1..15 in C, so the compiled/exported symbol is GdiEntryN,
//   NOT the DdXxx name. They are therefore named `GdiEntry1`..`GdiEntry15` directly
//   here (each carries a `// GdiEntryN — C source name DdXxx` comment). GdiEntry16
//   (DdSwapTextureHandles) already lives in historic.rs and is NOT redefined here.
//   All OTHER Dd*/Dvp*/D3d* functions are the DirectDraw HAL callbacks and keep
//   their real names (they are stored as callback pointers into the DDHAL tables).
//
// NEEDS-TYPE: `D3DHAL_CONTEXTCREATEDATA` / `LPD3DHAL_CONTEXTCREATEDATA` (param of
//   D3dContextCreate) is the user-mode struct from ddk/d3dhal.h; it is absent from
//   win32k_sys bindgen. Defined locally below (repr(C), layout per d3dhal.h:234).
// NEEDS-TYPE: `LPD3DHAL_CALLBACKS` / `LPD3DHAL_GLOBALDRIVERDATA` (params of GdiEntry2)
//   are forward-declared in ddrawgdi.h and absent from win32k_sys bindgen. Aliased
//   locally to the kernel NT variants (LPD3DNTHAL_CALLBACKS / LPD3DNTHAL_GLOBALDRIVERDATA),
//   which share the field names/layout the body touches (dwSize/ContextCreate/.. and
//   lpTextureFormats). The sizeof in the RtlMoveMemory uses the NT variant size.
// NEEDS-CHECK: the DDVIDEOPORTCALLBACKSSIZE / DDCOLORCONTROLCALLBACKSSIZE /
//   DDMISCELLANEOUSCALLBACKSSIZE / DDMISCELLANEOUS2CALLBACKSSIZE macros (ddrawi.h:630-634)
//   are `sizeof(DDHAL_DD*CALLBACKS)`; rendered as `size_of::<..>()` consts below.
// NEEDS-CHECK: DDERR_GENERIC/DDERR_OUTOFMEMORY/DD_OK are HRESULT macros (ddraw.h →
//   E_FAIL/E_OUTOFMEMORY/S_OK); absent from bindgen → local consts below.
// NEEDS-CHECK: callback function pointers are stored into bindgen `Option<extern fn>`
//   struct fields via `transmute(fn as usize)` (the `cbcast!` macro), mirroring the C
//   casts like `(LPDDHALVPORTCB_CREATEVIDEOPORT) DvpCreateVideoPort`.
// NEEDS-CHECK: DdAlphaBlt/DdColorControl/DdSetExclusiveMode/DdFlipToGDISurface pass
//   `&param` (address of the by-value pointer parameter) to win32k — a verbatim ReactOS
//   quirk, preserved with addr_of! (see inline notes).
// NEEDS-CHECK: `ghDirectDraw`/`gcDirectDraw` are non-static gdi32 globals → exported as
//   `#[no_mangle] pub static mut`. `GdiSharedHandleTable` is owned by misc.rs (referenced
//   here via an extern static).

use win32k_sys::win32k::*;
use crate::gdihelp::*;

use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, copy_nonoverlapping, null_mut, write_bytes};

// win32k_sys FALSE/TRUE are bindgen u32; BOOL is i32 — BOOL-typed shadows.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// ── HRESULT/DirectDraw status macros (ddraw.h) — absent from bindgen ──────────────
const DD_OK: HRESULT = 0; // S_OK
const DDERR_GENERIC: HRESULT = 0x8000_4005u32 as HRESULT; // E_FAIL
const DDERR_OUTOFMEMORY: HRESULT = 0x8007_000Eu32 as HRESULT; // E_OUTOFMEMORY

// ── gdi32 DirectDraw globals (gdientry.c lines 19-20, non-static) ─────────────────
#[no_mangle]
pub static mut ghDirectDraw: HANDLE = null_mut();
#[no_mangle]
pub static mut gcDirectDraw: ULONG = 0;

// ── Substrate global owned by misc.rs (gdi32 shared GDI handle table) ─────────────
extern "C" {
    static mut GdiSharedHandleTable: PGDI_SHARED_HANDLE_TABLE;
}

// ── kernel32 / gdi32 cross-DLL imports (WINAPI → extern "system") ─────────────────
extern "system" {
    fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
    fn SetLastError(dwErrCode: DWORD);
    fn OutputDebugStringA(lpOutputString: LPCSTR);
    fn CreateDCW(
        pwszDriver: LPCWSTR,
        pwszDevice: LPCWSTR,
        pszPort: LPCWSTR,
        pdm: *const DEVMODEW,
    ) -> HDC;
    fn DeleteDC(hdc: HDC) -> BOOL;
}

// ── Local types absent from win32k_sys bindgen ───────────────────────────────────
// User-mode D3DHAL_CONTEXTCREATEDATA (ddk/d3dhal.h:234). All four unions are pointer-
// sized; only lpDDLcl/lpDDSLcl/lpDDSZLcl are touched by the body.
#[repr(C)]
pub struct D3DHAL_CONTEXTCREATEDATA {
    pub lpDDLcl: LPDDRAWI_DIRECTDRAW_LCL,
    pub lpDDSLcl: LPDDRAWI_DDRAWSURFACE_LCL,
    pub lpDDSZLcl: LPDDRAWI_DDRAWSURFACE_LCL,
    pub dwPID: ULONG_PTR,
    pub dwhContext: ULONG_PTR,
    pub ddrval: HRESULT,
}
pub type LPD3DHAL_CONTEXTCREATEDATA = *mut D3DHAL_CONTEXTCREATEDATA;

// ddrawgdi.h forward decls — mapped to the kernel NT variants (layout-compatible for
// the fields the body uses).
pub type LPD3DHAL_CALLBACKS = LPD3DNTHAL_CALLBACKS;
pub type D3DHAL_GLOBALDRIVERDATA = D3DNTHAL_GLOBALDRIVERDATA;
pub type LPD3DHAL_GLOBALDRIVERDATA = LPD3DNTHAL_GLOBALDRIVERDATA;

// DDHAL callback-table size macros (ddrawi.h:630-634).
#[inline]
fn ddvideoportcallbackssize() -> DWORD {
    size_of::<DDHAL_DDVIDEOPORTCALLBACKS>() as DWORD
}
#[inline]
fn ddcolorcontrolcallbackssize() -> DWORD {
    size_of::<DDHAL_DDCOLORCONTROLCALLBACKS>() as DWORD
}
#[inline]
fn ddmiscellaneouscallbackssize() -> DWORD {
    size_of::<DDHAL_DDMISCELLANEOUSCALLBACKS>() as DWORD
}
#[inline]
fn ddmiscellaneous2callbackssize() -> DWORD {
    size_of::<DDHAL_DDMISCELLANEOUS2CALLBACKS>() as DWORD
}

// Store a Rust fn (or pointer) into a bindgen `Option<extern fn>` / LPVOID struct field,
// mirroring the C `(LPxxx) SomeFunc` casts. Target type is inferred from the LHS. Through
// a POINTER, not `usize`: transmuting an integer into an LPVOID field makes a pointer
// with no provenance, which the cast this mirrors never did.
macro_rules! cbcast {
    ($f:expr) => {
        core::mem::transmute($f as *const ())
    };
}

// GetDdHandle(Handle) macro: ((HANDLE)Handle ? (HANDLE)Handle : ghDirectDraw)
#[inline]
unsafe fn GetDdHandle(handle: ULONG_PTR) -> HANDLE {
    if handle != 0 {
        handle as HANDLE
    } else {
        ghDirectDraw
    }
}

// L"Display" wide literal for GdiEntry1's CreateDCW call.
static DISPLAY_W: [u16; 8] = [
    b'D' as u16,
    b'i' as u16,
    b's' as u16,
    b'p' as u16,
    b'l' as u16,
    b'a' as u16,
    b'y' as u16,
    0,
];

// IsEqualGUID (C macro / inline) — 16-byte compare.
#[inline]
unsafe fn IsEqualGUID(a: *const GUID, b: *const GUID) -> bool {
    let pa = a as *const u8;
    let pb = b as *const u8;
    let mut i = 0usize;
    while i < size_of::<GUID>() {
        if *pa.add(i) != *pb.add(i) {
            return false;
        }
        i += 1;
    }
    true
}

/* CALLBACKS *****************************************************************/

/// @implemented — DdAddAttachedSurface
#[no_mangle]
pub unsafe extern "system" fn DdAddAttachedSurface(Attach: LPDDHAL_ADDATTACHEDSURFACEDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdAddAttachedSurface(
        (*(*Attach).lpDDSurface).hDDSurface as HANDLE,
        (*(*Attach).lpSurfAttached).hDDSurface as HANDLE,
        Attach as PDD_ADDATTACHEDSURFACEDATA,
    )
}

/// @implemented — DdBlt
#[no_mangle]
pub unsafe extern "system" fn DdBlt(Blt: LPDDHAL_BLTDATA) -> DWORD {
    let mut Surface: HANDLE = null_mut();

    /* Use the right surface */
    if !(*Blt).lpDDSrcSurface.is_null() {
        Surface = (*(*Blt).lpDDSrcSurface).hDDSurface as HANDLE;
    }

    /* Call win32k */
    NtGdiDdBlt(
        (*(*Blt).lpDDDestSurface).hDDSurface as HANDLE,
        Surface,
        Blt as PDD_BLTDATA,
    )
}

/// @implemented — DdDestroySurface
#[no_mangle]
pub unsafe extern "system" fn DdDestroySurface(pDestroySurface: LPDDHAL_DESTROYSURFACEDATA) -> DWORD {
    let mut Return: DWORD = DDHAL_DRIVER_NOTHANDLED;

    if (*(*pDestroySurface).lpDDSurface).hDDSurface != 0 {
        /* Check if we should really destroy it */
        let RealDestroy: BOOL = if ((*(*pDestroySurface).lpDDSurface).dwFlags & DDRAWISURF_DRIVERMANAGED)
            == 0
            || ((*(*pDestroySurface).lpDDSurface).dwFlags & DDRAWISURF_INVALID) == 0
        {
            TRUE
        } else {
            FALSE
        };

        /* Call win32k */
        Return = NtGdiDdDestroySurface(
            (*(*pDestroySurface).lpDDSurface).hDDSurface as HANDLE,
            RealDestroy,
        );
    }

    Return
}

/// @implemented — DdFlip
#[no_mangle]
pub unsafe extern "system" fn DdFlip(Flip: LPDDHAL_FLIPDATA) -> DWORD {
    let mut hSurfaceCurrentLeft: HANDLE = null_mut();
    let mut hSurfaceTargetLeft: HANDLE = null_mut();

    /* Auto flip off or on */
    if ((*Flip).dwFlags & DDFLIP_STEREO) != 0
        && !(*Flip).lpSurfTargLeft.is_null()
        && !(*Flip).lpSurfCurrLeft.is_null()
    {
        /* Auto flip on */
        hSurfaceTargetLeft = (*(*Flip).lpSurfTargLeft).hDDSurface as HANDLE;
        hSurfaceCurrentLeft = (*(*Flip).lpSurfCurrLeft).hDDSurface as HANDLE;
    }

    /* Call win32k */
    NtGdiDdFlip(
        (*(*Flip).lpSurfCurr).hDDSurface as HANDLE,
        (*(*Flip).lpSurfTarg).hDDSurface as HANDLE,
        hSurfaceCurrentLeft,
        hSurfaceTargetLeft,
        Flip as PDD_FLIPDATA,
    )
}

/// @implemented — DdLock
#[no_mangle]
pub unsafe extern "system" fn DdLock(Lock: LPDDHAL_LOCKDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdLock(
        (*(*Lock).lpDDSurface).hDDSurface as HANDLE,
        Lock as PDD_LOCKDATA,
        (*(*Lock).lpDDSurface).hDC as HDC,
    )
}

/// @implemented — DdUnlock
#[no_mangle]
pub unsafe extern "system" fn DdUnlock(pUnlock: *mut _DDHAL_UNLOCKDATA) -> DWORD {
    /* Call win32k */
    // NB: param renamed Unlock->pUnlock — `Unlock` is a glob-imported const
    // (_SECURITY_LOGON_TYPE_Unlock via win32k_sys), which would capture the param as a
    // const-pattern (Rust footgun) and shadow it with a u32 in the body.
    NtGdiDdUnlock(
        (*(*pUnlock).lpDDSurface).hDDSurface as HANDLE,
        pUnlock as PDD_UNLOCKDATA,
    )
}

/// @implemented — DdGetBltStatus
#[no_mangle]
pub unsafe extern "system" fn DdGetBltStatus(GetBltStatus: LPDDHAL_GETBLTSTATUSDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdGetBltStatus(
        (*(*GetBltStatus).lpDDSurface).hDDSurface as HANDLE,
        GetBltStatus as PDD_GETBLTSTATUSDATA,
    )
}

/// @implemented — DdGetFlipStatus
#[no_mangle]
pub unsafe extern "system" fn DdGetFlipStatus(GetFlipStatus: LPDDHAL_GETFLIPSTATUSDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdGetFlipStatus(
        (*(*GetFlipStatus).lpDDSurface).hDDSurface as HANDLE,
        GetFlipStatus as PDD_GETFLIPSTATUSDATA,
    )
}

/// @implemented — DdUpdateOverlay
#[no_mangle]
pub unsafe extern "system" fn DdUpdateOverlay(UpdateOverlay: LPDDHAL_UPDATEOVERLAYDATA) -> DWORD {
    /* We have to handle this manually here */
    if ((*UpdateOverlay).dwFlags & DDOVER_KEYDEST) != 0 {
        /* Use the override */
        (*UpdateOverlay).dwFlags &= !DDOVER_KEYDEST;
        (*UpdateOverlay).dwFlags |= DDOVER_KEYDESTOVERRIDE;

        /* Set the overlay */
        (*UpdateOverlay).overlayFX.dckDestColorkey =
            (*(*UpdateOverlay).lpDDDestSurface).ddckCKDestOverlay;
    }
    if ((*UpdateOverlay).dwFlags & DDOVER_KEYSRC) != 0 {
        /* Use the override */
        (*UpdateOverlay).dwFlags &= !DDOVER_KEYSRC;
        (*UpdateOverlay).dwFlags |= DDOVER_KEYSRCOVERRIDE;

        /* Set the overlay */
        (*UpdateOverlay).overlayFX.dckSrcColorkey =
            (*(*UpdateOverlay).lpDDSrcSurface).ddckCKSrcOverlay;
    }

    /* Call win32k */
    NtGdiDdUpdateOverlay(
        (*(*UpdateOverlay).lpDDDestSurface).hDDSurface as HANDLE,
        (*(*UpdateOverlay).lpDDSrcSurface).hDDSurface as HANDLE,
        UpdateOverlay as PDD_UPDATEOVERLAYDATA,
    )
}

/// @implemented — DdSetOverlayPosition
#[no_mangle]
pub unsafe extern "system" fn DdSetOverlayPosition(
    SetOverlayPosition: LPDDHAL_SETOVERLAYPOSITIONDATA,
) -> DWORD {
    /* Call win32k */
    NtGdiDdSetOverlayPosition(
        (*(*SetOverlayPosition).lpDDSrcSurface).hDDSurface as HANDLE,
        (*(*SetOverlayPosition).lpDDDestSurface).hDDSurface as HANDLE,
        SetOverlayPosition as PDD_SETOVERLAYPOSITIONDATA,
    )
}

/// @implemented — DdWaitForVerticalBlank
#[no_mangle]
pub unsafe extern "system" fn DdWaitForVerticalBlank(
    WaitForVerticalBlank: LPDDHAL_WAITFORVERTICALBLANKDATA,
) -> DWORD {
    /* Call win32k */
    NtGdiDdWaitForVerticalBlank(
        GetDdHandle((*(*WaitForVerticalBlank).lpDD).hDD),
        WaitForVerticalBlank as PDD_WAITFORVERTICALBLANKDATA,
    )
}

/// @implemented — DdCanCreateSurface
#[no_mangle]
pub unsafe extern "system" fn DdCanCreateSurface(CanCreateSurface: LPDDHAL_CANCREATESURFACEDATA) -> DWORD {
    /* Note: NtGdiDdCanCreateD3DBuffer and NtGdiDdCanCreateSurface merge in win32k. */
    /* Call win32k */
    NtGdiDdCanCreateSurface(
        GetDdHandle((*(*CanCreateSurface).lpDD).hDD),
        CanCreateSurface as PDD_CANCREATESURFACEDATA,
    )
}

/// @implemented — DdCreateSurface
#[no_mangle]
pub unsafe extern "system" fn DdCreateSurface(pCreateSurface: LPDDHAL_CREATESURFACEDATA) -> DWORD {
    let mut Return: DWORD = DDHAL_DRIVER_NOTHANDLED;
    let SurfaceCount: ULONG = (*pCreateSurface).dwSCnt;
    let mut DdSurfaceLocal: DD_SURFACE_LOCAL = core::mem::zeroed();
    let mut DdSurfaceMore: DD_SURFACE_MORE = core::mem::zeroed();
    let mut DdSurfaceGlobal: DD_SURFACE_GLOBAL = core::mem::zeroed();

    let mut hPrevSurface: HANDLE = null_mut();
    let mut hSurface: HANDLE = null_mut();

    let mut pDdSurfaceLocal: PDD_SURFACE_LOCAL = null_mut();
    let mut pDdSurfaceMore: PDD_SURFACE_MORE = null_mut();
    let mut pDdSurfaceGlobal: PDD_SURFACE_GLOBAL = null_mut();

    let mut ptmpDdSurfaceLocal: PDD_SURFACE_LOCAL;
    let mut ptmpDdSurfaceMore: PDD_SURFACE_MORE;
    let mut ptmpDdSurfaceGlobal: PDD_SURFACE_GLOBAL;
    let mut phSurface: PHANDLE = null_mut();
    let mut puhSurface: PHANDLE = null_mut();
    let mut i: ULONG;
    let mut pSurfaceDesc: LPDDSURFACEDESC = null_mut();

    // Original C emits an OutputDebugStringA("Function ... Optimze max to N Surface ...")
    // here (a debug-only print). Rendered as a comment per the DPRINT convention.
    let _ = OutputDebugStringA; // keep the import referenced

    /* Check how many surfaces there are */
    if SurfaceCount != 1 {
        /* We got more than one surface, so allocate memory for them */
        pDdSurfaceLocal = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<DD_SURFACE_LOCAL>() * SurfaceCount as usize) as SIZE_T,
        ) as PDD_SURFACE_LOCAL;
        pDdSurfaceMore = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<DD_SURFACE_MORE>() * SurfaceCount as usize) as SIZE_T,
        ) as PDD_SURFACE_MORE;
        pDdSurfaceGlobal = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<DD_SURFACE_GLOBAL>() * SurfaceCount as usize) as SIZE_T,
        ) as PDD_SURFACE_GLOBAL;
        phSurface = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<HANDLE>() * SurfaceCount as usize) as SIZE_T,
        ) as PHANDLE;
        puhSurface = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (size_of::<HANDLE>() * SurfaceCount as usize) as SIZE_T,
        ) as PHANDLE;

        /* Check if we successfully allocated all memory we need */
        if pDdSurfaceLocal.is_null()
            || pDdSurfaceMore.is_null()
            || pDdSurfaceGlobal.is_null()
            || phSurface.is_null()
            || puhSurface.is_null()
        {
            (*pCreateSurface).ddRVal = DDERR_OUTOFMEMORY;

            if !pDdSurfaceLocal.is_null() {
                HeapFree(GetProcessHeap(), 0, pDdSurfaceLocal as LPVOID);
            }
            if !pDdSurfaceMore.is_null() {
                HeapFree(GetProcessHeap(), 0, pDdSurfaceMore as LPVOID);
            }
            if !pDdSurfaceGlobal.is_null() {
                HeapFree(GetProcessHeap(), 0, pDdSurfaceGlobal as LPVOID);
            }
            if !phSurface.is_null() {
                HeapFree(GetProcessHeap(), 0, phSurface as LPVOID);
            }
            if !puhSurface.is_null() {
                HeapFree(GetProcessHeap(), 0, puhSurface as LPVOID);
            }

            return DDHAL_DRIVER_HANDLED;
        }
    } else {
        /* We'll use what we have on the stack */
        pDdSurfaceLocal = &mut DdSurfaceLocal;
        pDdSurfaceMore = &mut DdSurfaceMore;
        pDdSurfaceGlobal = &mut DdSurfaceGlobal;
        phSurface = &mut hPrevSurface;
        puhSurface = &mut hSurface;

        /* Clear the structures */
        write_bytes(addr_of_mut!(DdSurfaceLocal) as *mut u8, 0, size_of::<DD_SURFACE_LOCAL>());
        write_bytes(addr_of_mut!(DdSurfaceGlobal) as *mut u8, 0, size_of::<DD_SURFACE_GLOBAL>());
        write_bytes(addr_of_mut!(DdSurfaceMore) as *mut u8, 0, size_of::<DD_SURFACE_MORE>());
    }

    /* check if we got a surface or not */
    if SurfaceCount != 0 {
        /* Loop for each surface */
        ptmpDdSurfaceGlobal = pDdSurfaceGlobal;
        ptmpDdSurfaceLocal = pDdSurfaceLocal;
        ptmpDdSurfaceMore = pDdSurfaceMore;
        pSurfaceDesc = (*pCreateSurface).lpDDSurfaceDesc;

        i = 0;
        while i < SurfaceCount {
            let lcl: LPDDRAWI_DDRAWSURFACE_LCL = *(*pCreateSurface).lplpSList.add(i as usize);
            let gpl: LPDDRAWI_DDRAWSURFACE_GBL = (*lcl).lpGbl;

            *phSurface.add(i as usize) = (*lcl).hDDSurface as HANDLE;
            (*ptmpDdSurfaceLocal).ddsCaps.dwCaps = (*lcl).ddsCaps.dwCaps;

            (*ptmpDdSurfaceLocal).dwFlags = ((*ptmpDdSurfaceLocal).dwFlags
                & (0xB0000000
                    | DDRAWISURF_INMASTERSPRITELIST
                    | DDRAWISURF_HELCB
                    | DDRAWISURF_FRONTBUFFER
                    | DDRAWISURF_BACKBUFFER
                    | DDRAWISURF_INVALID
                    | DDRAWISURF_DCIBUSY
                    | DDRAWISURF_DCILOCK))
                | ((*lcl).dwFlags & DDRAWISURF_DRIVERMANAGED);

            (*ptmpDdSurfaceGlobal).wWidth = (*gpl).wWidth as DWORD;
            (*ptmpDdSurfaceGlobal).wHeight = (*gpl).wHeight as DWORD;
            (*ptmpDdSurfaceGlobal).__bindgen_anon_3.lPitch = (*gpl).__bindgen_anon_4.lPitch;
            (*ptmpDdSurfaceGlobal).fpVidMem = (*gpl).fpVidMem;
            (*ptmpDdSurfaceGlobal).__bindgen_anon_2.dwBlockSizeX = (*gpl).__bindgen_anon_2.dwBlockSizeX;
            (*ptmpDdSurfaceGlobal).__bindgen_anon_1.dwBlockSizeY = (*gpl).__bindgen_anon_1.dwBlockSizeY;

            if ((*lcl).dwFlags & DDRAWISURF_HASPIXELFORMAT) != 0 {
                copy_nonoverlapping(
                    addr_of!((*gpl).ddpfSurface) as *const u8,
                    addr_of_mut!((*ptmpDdSurfaceGlobal).ddpfSurface) as *mut u8,
                    size_of::<DDPIXELFORMAT>(),
                );
                (*ptmpDdSurfaceGlobal).ddpfSurface.dwSize = size_of::<DDPIXELFORMAT>() as DWORD;
            } else {
                copy_nonoverlapping(
                    addr_of!((*(*gpl).__bindgen_anon_3.lpDD).vmiData.ddpfDisplay) as *const u8,
                    addr_of_mut!((*ptmpDdSurfaceGlobal).ddpfSurface) as *mut u8,
                    size_of::<DDPIXELFORMAT>(),
                );
            }

            /* If lcl->lpSurfMore is NULL the bindgen-zeroed struct already has these clear */
            if !(*lcl).lpSurfMore.is_null() {
                (*ptmpDdSurfaceMore).ddsCapsEx.dwCaps2 = (*(*lcl).lpSurfMore).ddsCapsEx.dwCaps2;
                (*ptmpDdSurfaceMore).ddsCapsEx.dwCaps3 = (*(*lcl).lpSurfMore).ddsCapsEx.dwCaps3;
                (*ptmpDdSurfaceMore).ddsCapsEx.__bindgen_anon_1.dwCaps4 = (*(*lcl).lpSurfMore).ddsCapsEx.__bindgen_anon_1.dwCaps4;
                (*ptmpDdSurfaceMore).dwSurfaceHandle = (*(*lcl).lpSurfMore).dwSurfaceHandle;
            }

            /* count to next SurfaceCount */
            ptmpDdSurfaceGlobal = ptmpDdSurfaceGlobal.add(1);
            ptmpDdSurfaceLocal = ptmpDdSurfaceLocal.add(1);
            ptmpDdSurfaceMore = ptmpDdSurfaceMore.add(1);

            i += 1;
        }
    }

    /* Call win32k now */
    (*pCreateSurface).ddRVal = DDERR_GENERIC;

    Return = NtGdiDdCreateSurface(
        GetDdHandle((*(*pCreateSurface).lpDD).hDD),
        phSurface,
        pSurfaceDesc,
        pDdSurfaceGlobal,
        pDdSurfaceLocal,
        pDdSurfaceMore,
        pCreateSurface as PDD_CREATESURFACEDATA,
        puhSurface,
    );

    if SurfaceCount == 0 {
        (*pCreateSurface).ddRVal = DDERR_GENERIC;
    } else {
        ptmpDdSurfaceMore = pDdSurfaceMore;
        ptmpDdSurfaceGlobal = pDdSurfaceGlobal;
        ptmpDdSurfaceLocal = pDdSurfaceLocal;

        i = 0;
        while i < SurfaceCount {
            let lcl: LPDDRAWI_DDRAWSURFACE_LCL = *(*pCreateSurface).lplpSList.add(i as usize);
            let gpl: LPDDRAWI_DDRAWSURFACE_GBL = (*lcl).lpGbl;

            (*gpl).__bindgen_anon_4.lPitch = (*ptmpDdSurfaceGlobal).__bindgen_anon_3.lPitch;
            (*gpl).fpVidMem = (*ptmpDdSurfaceGlobal).fpVidMem;
            (*gpl).__bindgen_anon_2.dwBlockSizeX = (*ptmpDdSurfaceGlobal).__bindgen_anon_2.dwBlockSizeX;
            (*gpl).__bindgen_anon_1.dwBlockSizeY = (*ptmpDdSurfaceGlobal).__bindgen_anon_1.dwBlockSizeY;

            if ((*lcl).dwFlags & DDRAWISURF_HASPIXELFORMAT) != 0 {
                copy_nonoverlapping(
                    addr_of!((*ptmpDdSurfaceGlobal).ddpfSurface) as *const u8,
                    addr_of_mut!((*gpl).ddpfSurface) as *mut u8,
                    size_of::<DDPIXELFORMAT>(),
                );
            }

            if (*pCreateSurface).ddRVal != DD_OK {
                (*gpl).fpVidMem = 0;
                if (*lcl).hDDSurface != 0 {
                    NtGdiDdDeleteSurfaceObject((*lcl).hDDSurface as HANDLE);
                }
                (*lcl).hDDSurface = 0;
            } else {
                (*lcl).hDDSurface = *puhSurface.add(i as usize) as ULONG_PTR;
            }

            (*lcl).ddsCaps.dwCaps = (*ptmpDdSurfaceLocal).ddsCaps.dwCaps;
            if !(*lcl).lpSurfMore.is_null() {
                (*(*lcl).lpSurfMore).ddsCapsEx.dwCaps2 = (*ptmpDdSurfaceMore).ddsCapsEx.dwCaps2;
                (*(*lcl).lpSurfMore).ddsCapsEx.dwCaps3 = (*ptmpDdSurfaceMore).ddsCapsEx.dwCaps3;
                (*(*lcl).lpSurfMore).ddsCapsEx.__bindgen_anon_1.dwCaps4 = (*ptmpDdSurfaceMore).ddsCapsEx.__bindgen_anon_1.dwCaps4;
            }

            /* count to next SurfaceCount */
            ptmpDdSurfaceGlobal = ptmpDdSurfaceGlobal.add(1);
            ptmpDdSurfaceLocal = ptmpDdSurfaceLocal.add(1);
            ptmpDdSurfaceMore = ptmpDdSurfaceMore.add(1);

            i += 1;
        }
    }

    /* Check if we have to free all our local allocations */
    if SurfaceCount > 1 {
        if !pDdSurfaceLocal.is_null() {
            HeapFree(GetProcessHeap(), 0, pDdSurfaceLocal as LPVOID);
        }
        if !pDdSurfaceMore.is_null() {
            HeapFree(GetProcessHeap(), 0, pDdSurfaceMore as LPVOID);
        }
        if !pDdSurfaceGlobal.is_null() {
            HeapFree(GetProcessHeap(), 0, pDdSurfaceGlobal as LPVOID);
        }
        if !phSurface.is_null() {
            HeapFree(GetProcessHeap(), 0, phSurface as LPVOID);
        }
        if !puhSurface.is_null() {
            HeapFree(GetProcessHeap(), 0, puhSurface as LPVOID);
        }
    }

    /* Return */
    Return
}

/// @implemented — DdSetColorKey
#[no_mangle]
pub unsafe extern "system" fn DdSetColorKey(pSetColorKey: LPDDHAL_SETCOLORKEYDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdSetColorKey(
        (*(*pSetColorKey).lpDDSurface).hDDSurface as HANDLE,
        pSetColorKey as PDD_SETCOLORKEYDATA,
    )
}

/// @implemented — DdGetScanLine
#[no_mangle]
pub unsafe extern "system" fn DdGetScanLine(pGetScanLine: LPDDHAL_GETSCANLINEDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdGetScanLine(
        GetDdHandle((*(*pGetScanLine).lpDD).hDD),
        pGetScanLine as PDD_GETSCANLINEDATA,
    )
}

/// @implemented — DvpCreateVideoPort
#[no_mangle]
pub unsafe extern "system" fn DvpCreateVideoPort(pDvdCreatePort: LPDDHAL_CREATEVPORTDATA) -> BOOL {
    (*(*pDvdCreatePort).lpVideoPort).hDDVideoPort = NtGdiDvpCreateVideoPort(
        GetDdHandle((*(*(*pDvdCreatePort).lpDD).lpGbl).hDD),
        pDvdCreatePort as PDD_CREATEVPORTDATA,
    );

    TRUE
}

/// @implemented — DvpDestroyVideoPort
#[no_mangle]
pub unsafe extern "system" fn DvpDestroyVideoPort(pDvdDestoryPort: LPDDHAL_DESTROYVPORTDATA) -> DWORD {
    NtGdiDvpDestroyVideoPort(
        (*(*pDvdDestoryPort).lpVideoPort).hDDVideoPort,
        pDvdDestoryPort as PDD_DESTROYVPORTDATA,
    )
}

/// @implemented — DvpFlipVideoPort
#[no_mangle]
pub unsafe extern "system" fn DvpFlipVideoPort(pDvdPortFlip: LPDDHAL_FLIPVPORTDATA) -> DWORD {
    NtGdiDvpFlipVideoPort(
        (*(*pDvdPortFlip).lpVideoPort).hDDVideoPort,
        (*(*pDvdPortFlip).lpSurfCurr).hDDSurface as HANDLE,
        (*(*pDvdPortFlip).lpSurfTarg).hDDSurface as HANDLE,
        pDvdPortFlip as PDD_FLIPVPORTDATA,
    )
}

/// @implemented — DvpGetVideoPortBandwidth
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortBandwidth(
    pDvdPortBandWidth: LPDDHAL_GETVPORTBANDWIDTHDATA,
) -> DWORD {
    NtGdiDvpGetVideoPortBandwidth(
        (*(*pDvdPortBandWidth).lpVideoPort).hDDVideoPort,
        pDvdPortBandWidth as PDD_GETVPORTBANDWIDTHDATA,
    )
}

/// @implemented — DvpColorControl
#[no_mangle]
pub unsafe extern "system" fn DvpColorControl(pDvdPortColorControl: LPDDHAL_VPORTCOLORDATA) -> DWORD {
    NtGdiDvpColorControl(
        (*(*pDvdPortColorControl).lpVideoPort).hDDVideoPort,
        pDvdPortColorControl as PDD_VPORTCOLORDATA,
    )
}

/// @implemented — DvpGetVideoSignalStatus
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoSignalStatus(
    pDvdPortVideoSignalStatus: LPDDHAL_GETVPORTSIGNALDATA,
) -> DWORD {
    NtGdiDvpGetVideoSignalStatus(
        (*(*pDvdPortVideoSignalStatus).lpVideoPort).hDDVideoPort,
        pDvdPortVideoSignalStatus as PDD_GETVPORTSIGNALDATA,
    )
}

/// @implemented — DvpGetVideoPortFlipStatus
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortFlipStatus(
    pDvdPortVideoPortFlipStatus: LPDDHAL_GETVPORTFLIPSTATUSDATA,
) -> DWORD {
    NtGdiDvpGetVideoPortFlipStatus(
        GetDdHandle((*(*(*pDvdPortVideoPortFlipStatus).lpDD).lpGbl).hDD),
        pDvdPortVideoPortFlipStatus as PDD_GETVPORTFLIPSTATUSDATA,
    )
}

/// @implemented — DvpCanCreateVideoPort
#[no_mangle]
pub unsafe extern "system" fn DvpCanCreateVideoPort(
    pDvdCanCreateVideoPort: LPDDHAL_CANCREATEVPORTDATA,
) -> DWORD {
    NtGdiDvpCanCreateVideoPort(
        GetDdHandle((*(*(*pDvdCanCreateVideoPort).lpDD).lpGbl).hDD),
        pDvdCanCreateVideoPort as PDD_CANCREATEVPORTDATA,
    )
}

/// @implemented — DvpWaitForVideoPortSync
#[no_mangle]
pub unsafe extern "system" fn DvpWaitForVideoPortSync(
    pDvdWaitForVideoPortSync: LPDDHAL_WAITFORVPORTSYNCDATA,
) -> DWORD {
    NtGdiDvpWaitForVideoPortSync(
        (*(*pDvdWaitForVideoPortSync).lpVideoPort).hDDVideoPort,
        pDvdWaitForVideoPortSync as PDD_WAITFORVPORTSYNCDATA,
    )
}

/// @implemented — DvpUpdateVideoPort
#[no_mangle]
pub unsafe extern "system" fn DvpUpdateVideoPort(pDvdUpdateVideoPort: LPDDHAL_UPDATEVPORTDATA) -> DWORD {
    /* XP limits to max 10 videoport surface and VBI handles; ReactOS guards the same. */
    let mut phSurfaceVideo: [HANDLE; 10] = [null_mut(); 10];
    let mut phSurfaceVbi: [HANDLE; 10] = [null_mut(); 10];

    if (*pDvdUpdateVideoPort).dwFlags != DDRAWI_VPORTSTOP {
        /* Take copy of lplpDDSurface (handle value gets modified in dxg) */
        let mut dwNumAutoflip: DWORD = (*pDvdUpdateVideoPort).dwNumAutoflip;
        if dwNumAutoflip == 0 && (*pDvdUpdateVideoPort).lplpDDSurface.is_null() {
            dwNumAutoflip += 1;
        }

        if dwNumAutoflip != 0 {
            if dwNumAutoflip > 10 {
                dwNumAutoflip = 10;
            }
            copy_nonoverlapping(
                (*pDvdUpdateVideoPort).lplpDDSurface as *const HANDLE,
                phSurfaceVideo.as_mut_ptr(),
                dwNumAutoflip as usize,
            );
        }

        /* Take copy of lplpDDVBISurface (handle value gets modified in dxg) */
        let mut dwNumVBIAutoflip: DWORD = (*pDvdUpdateVideoPort).dwNumVBIAutoflip;
        if dwNumVBIAutoflip == 0 && (*pDvdUpdateVideoPort).lplpDDVBISurface.is_null() {
            dwNumVBIAutoflip += 1;
        }

        if dwNumVBIAutoflip != 0 {
            if dwNumVBIAutoflip > 10 {
                dwNumVBIAutoflip = 10;
            }
            copy_nonoverlapping(
                (*pDvdUpdateVideoPort).lplpDDVBISurface as *const HANDLE,
                phSurfaceVbi.as_mut_ptr(),
                dwNumVBIAutoflip as usize,
            );
        }
    }

    /* Call Win32k */
    NtGdiDvpUpdateVideoPort(
        (*(*pDvdUpdateVideoPort).lpVideoPort).hDDVideoPort,
        phSurfaceVideo.as_mut_ptr(),
        phSurfaceVbi.as_mut_ptr(),
        pDvdUpdateVideoPort as PDD_UPDATEVPORTDATA,
    )
}

/// @implemented — DvpGetVideoPortField
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortField(pDvdGetVideoPortField: LPDDHAL_FLIPVPORTDATA) -> DWORD {
    NtGdiDvpGetVideoPortField(
        (*(*pDvdGetVideoPortField).lpVideoPort).hDDVideoPort,
        pDvdGetVideoPortField as PDD_GETVPORTFIELDDATA,
    )
}

/// @implemented — DvpGetVideoPortInputFormats
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortInputFormats(
    pDvdGetVideoPortInputFormat: LPDDHAL_GETVPORTINPUTFORMATDATA,
) -> DWORD {
    NtGdiDvpGetVideoPortInputFormats(
        (*(*pDvdGetVideoPortInputFormat).lpVideoPort).hDDVideoPort,
        pDvdGetVideoPortInputFormat as PDD_GETVPORTINPUTFORMATDATA,
    )
}

/// @implemented — DvpGetVideoPortLine
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortLine(pDvdGetVideoPortLine: LPDDHAL_GETVPORTLINEDATA) -> DWORD {
    NtGdiDvpGetVideoPortLine(
        (*(*pDvdGetVideoPortLine).lpVideoPort).hDDVideoPort,
        pDvdGetVideoPortLine as PDD_GETVPORTLINEDATA,
    )
}

/// @implemented — DvpGetVideoPortOutputFormats
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortOutputFormats(
    pDvdGetVideoPortOutputFormat: LPDDHAL_GETVPORTLINEDATA,
) -> DWORD {
    /* Note: faithfully forwards to NtGdiDvpGetVideoPortLine (as the C source does). */
    NtGdiDvpGetVideoPortLine(
        (*(*pDvdGetVideoPortOutputFormat).lpVideoPort).hDDVideoPort,
        pDvdGetVideoPortOutputFormat as PDD_GETVPORTLINEDATA,
    )
}

/// @implemented — DvpGetVideoPortConnectInfo
#[no_mangle]
pub unsafe extern "system" fn DvpGetVideoPortConnectInfo(
    pDvdGetVideoPortInfo: LPDDHAL_GETVPORTCONNECTDATA,
) -> DWORD {
    NtGdiDvpGetVideoPortConnectInfo(
        GetDdHandle((*(*(*pDvdGetVideoPortInfo).lpDD).lpGbl).hDD),
        pDvdGetVideoPortInfo as PDD_GETVPORTCONNECTDATA,
    )
}

/// @implemented — DdGetAvailDriverMemory
#[no_mangle]
pub unsafe extern "system" fn DdGetAvailDriverMemory(
    pDdGetAvailDriverMemory: LPDDHAL_GETAVAILDRIVERMEMORYDATA,
) -> DWORD {
    NtGdiDdGetAvailDriverMemory(
        GetDdHandle((*(*pDdGetAvailDriverMemory).lpDD).hDD),
        pDdGetAvailDriverMemory as PDD_GETAVAILDRIVERMEMORYDATA,
    )
}

/// @implemented — DdAlphaBlt
#[no_mangle]
pub unsafe extern "system" fn DdAlphaBlt(pDdAlphaBlt: LPDDHAL_BLTDATA) -> DWORD {
    let mut hDDSrcSurface: HANDLE = null_mut();

    if !(*pDdAlphaBlt).lpDDSrcSurface.is_null() {
        hDDSrcSurface = (*(*pDdAlphaBlt).lpDDSrcSurface).hDDSurface as HANDLE;
    }

    /* NOTE: ReactOS verbatim passes &pDdAlphaBlt (address of the by-value param). */
    NtGdiDdAlphaBlt(
        (*(*pDdAlphaBlt).lpDDDestSurface).hDDSurface as HANDLE,
        hDDSrcSurface,
        addr_of!(pDdAlphaBlt) as *const _ as PDD_BLTDATA,
    )
}

/// @implemented — DdCreateSurfaceEx
#[no_mangle]
pub unsafe extern "system" fn DdCreateSurfaceEx(pDdCreateSurfaceEx: LPDDHAL_CREATESURFACEEXDATA) -> DWORD {
    (*pDdCreateSurfaceEx).ddRVal = NtGdiDdCreateSurfaceEx(
        GetDdHandle((*(*(*pDdCreateSurfaceEx).lpDDLcl).lpGbl).hDD),
        (*(*pDdCreateSurfaceEx).lpDDSLcl).hDDSurface as HANDLE,
        (*(*(*pDdCreateSurfaceEx).lpDDSLcl).lpSurfMore).dwSurfaceHandle,
    ) as HRESULT;
    TRUE as DWORD
}

/// @implemented — DdColorControl
#[no_mangle]
pub unsafe extern "system" fn DdColorControl(pDdColorControl: LPDDHAL_COLORCONTROLDATA) -> DWORD {
    /* NOTE: ReactOS verbatim passes &pDdColorControl (address of the by-value param). */
    NtGdiDdColorControl(
        (*(*pDdColorControl).lpDDSurface).hDDSurface as HANDLE,
        addr_of!(pDdColorControl) as *const _ as PDD_COLORCONTROLDATA,
    )
}

/// @implemented — DdSetExclusiveMode
#[no_mangle]
pub unsafe extern "system" fn DdSetExclusiveMode(pDdSetExclusiveMode: LPDDHAL_SETEXCLUSIVEMODEDATA) -> DWORD {
    /* NOTE: ReactOS verbatim passes &pDdSetExclusiveMode (address of the by-value param). */
    NtGdiDdSetExclusiveMode(
        GetDdHandle((*(*pDdSetExclusiveMode).lpDD).hDD),
        addr_of!(pDdSetExclusiveMode) as *const _ as PDD_SETEXCLUSIVEMODEDATA,
    )
}

/// @implemented — DdFlipToGDISurface
#[no_mangle]
pub unsafe extern "system" fn DdFlipToGDISurface(pDdFlipToGDISurface: LPDDHAL_FLIPTOGDISURFACEDATA) -> DWORD {
    /* NOTE: ReactOS verbatim passes &pDdFlipToGDISurface (address of the by-value param). */
    NtGdiDdFlipToGDISurface(
        GetDdHandle((*(*pDdFlipToGDISurface).lpDD).hDD),
        addr_of!(pDdFlipToGDISurface) as *const _ as PDD_FLIPTOGDISURFACEDATA,
    )
}

/* TODO (C source) — DdGetDriverInfo */
#[no_mangle]
pub unsafe extern "system" fn DdGetDriverInfo(pData: LPDDHAL_GETDRIVERINFODATA) -> DWORD {
    let mut pDrvInfoData: DDHAL_GETDRIVERINFODATA = core::mem::zeroed();
    let mut retValue: DWORD = DDHAL_DRIVER_NOTHANDLED;
    let hDD: HANDLE;

    /* FIXME add SEH around these functions */

    write_bytes(addr_of_mut!(pDrvInfoData) as *mut u8, 0, size_of::<DDHAL_GETDRIVERINFODATA>());
    pDrvInfoData.guidInfo = (*pData).guidInfo;

    hDD = GetDdHandle((*pData).dwContext);

    pDrvInfoData.dwSize = size_of::<DDHAL_GETDRIVERINFODATA>() as DWORD;
    pDrvInfoData.ddRVal = DDERR_GENERIC;
    pDrvInfoData.dwContext = hDD as ULONG_PTR;

    /* Videoport Callbacks check and setup for DirectX/ReactX */
    if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_VideoPortCallbacks)) {
        let mut pDvdPort: DDHAL_DDVIDEOPORTCALLBACKS = core::mem::zeroed();
        let pUserDvdPort: *mut DDHAL_DDVIDEOPORTCALLBACKS = (*pData).lpvData as *mut DDHAL_DDVIDEOPORTCALLBACKS;

        /* Clear internal out buffer and set it up */
        write_bytes(addr_of_mut!(pDvdPort) as *mut u8, 0, ddvideoportcallbackssize() as usize);
        pDvdPort.dwSize = ddvideoportcallbackssize();

        /* set up internal buffer */
        pDrvInfoData.lpvData = addr_of_mut!(pDvdPort) as PVOID;
        pDrvInfoData.dwExpectedSize = ddvideoportcallbackssize();

        /* Call win32k */
        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        /* Setup user out buffer and convert kmode callbacks to user mode */
        (*pUserDvdPort).dwSize = ddvideoportcallbackssize();
        pDrvInfoData.dwFlags = 0;
        (*pUserDvdPort).dwFlags = 0;

        (*pUserDvdPort).dwFlags = (pDrvInfoData.dwFlags
            & !(DDHAL_VPORT32_CREATEVIDEOPORT
                | DDHAL_VPORT32_FLIP
                | DDHAL_VPORT32_DESTROY
                | DDHAL_VPORT32_UPDATE
                | DDHAL_VPORT32_WAITFORSYNC))
            | (DDHAL_VPORT32_CREATEVIDEOPORT
                | DDHAL_VPORT32_FLIP
                | DDHAL_VPORT32_DESTROY
                | DDHAL_VPORT32_UPDATE);

        (*pData).dwActualSize = ddvideoportcallbackssize();
        (*pUserDvdPort).CreateVideoPort = cbcast!(DvpCreateVideoPort);
        (*pUserDvdPort).FlipVideoPort = cbcast!(DvpFlipVideoPort);
        (*pUserDvdPort).DestroyVideoPort = cbcast!(DvpDestroyVideoPort);
        (*pUserDvdPort).UpdateVideoPort = cbcast!(DvpUpdateVideoPort);

        if pDvdPort.CanCreateVideoPort.is_some() {
            (*pUserDvdPort).CanCreateVideoPort = cbcast!(DvpCanCreateVideoPort);
        }
        if pDvdPort.GetVideoPortBandwidth.is_some() {
            (*pUserDvdPort).GetVideoPortBandwidth = cbcast!(DvpGetVideoPortBandwidth);
        }
        if pDvdPort.GetVideoPortInputFormats.is_some() {
            (*pUserDvdPort).GetVideoPortInputFormats = cbcast!(DvpGetVideoPortInputFormats);
        }
        if pDvdPort.GetVideoPortOutputFormats.is_some() {
            (*pUserDvdPort).GetVideoPortOutputFormats = cbcast!(DvpGetVideoPortOutputFormats);
        }
        if pDvdPort.GetVideoPortField.is_some() {
            (*pUserDvdPort).GetVideoPortField = cbcast!(DvpGetVideoPortField);
        }
        if pDvdPort.GetVideoPortLine.is_some() {
            (*pUserDvdPort).GetVideoPortLine = cbcast!(DvpGetVideoPortLine);
        }
        if pDvdPort.GetVideoPortConnectInfo.is_some() {
            (*pUserDvdPort).GetVideoPortConnectInfo = cbcast!(DvpGetVideoPortConnectInfo);
        }
        if pDvdPort.GetVideoPortFlipStatus.is_some() {
            (*pUserDvdPort).GetVideoPortFlipStatus = cbcast!(DvpGetVideoPortFlipStatus);
        }
        if pDvdPort.WaitForVideoPortSync.is_some() {
            (*pUserDvdPort).WaitForVideoPortSync = cbcast!(DvpWaitForVideoPortSync);
        }
        if pDvdPort.GetVideoSignalStatus.is_some() {
            (*pUserDvdPort).GetVideoSignalStatus = cbcast!(DvpGetVideoSignalStatus);
        }
        if pDvdPort.ColorControl.is_some() {
            (*pUserDvdPort).ColorControl = cbcast!(DvpColorControl);
        }

        /* XP only reports whether we have a driver; ReactOS keeps the behavior */
        (*pData).ddRVal = retValue as HRESULT;
    }

    /* Color Control Callbacks check and setup for DirectX/ReactX */
    if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_ColorControlCallbacks)) {
        let mut pColorControl: DDHAL_DDCOLORCONTROLCALLBACKS = core::mem::zeroed();
        let pUserColorControl: *mut DDHAL_DDCOLORCONTROLCALLBACKS =
            (*pData).lpvData as *mut DDHAL_DDCOLORCONTROLCALLBACKS;

        write_bytes(addr_of_mut!(pColorControl) as *mut u8, 0, ddcolorcontrolcallbackssize() as usize);
        pColorControl.dwSize = ddcolorcontrolcallbackssize();

        pDrvInfoData.lpvData = addr_of_mut!(pColorControl) as PVOID;
        pDrvInfoData.dwExpectedSize = ddcolorcontrolcallbackssize();

        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        (*pData).dwActualSize = ddcolorcontrolcallbackssize();
        (*pData).dwFlags = pDrvInfoData.dwFlags;

        (*pUserColorControl).dwSize = ddcolorcontrolcallbackssize();
        (*pUserColorControl).dwFlags = pColorControl.dwFlags;

        if pColorControl.ColorControl.is_some() {
            (*pUserColorControl).ColorControl = cbcast!(DdColorControl);
        }

        (*pData).ddRVal = retValue as HRESULT;
    }
    /* Misc Callbacks check and setup for DirectX/ReactX */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_MiscellaneousCallbacks)) {
        let mut pMisc: DDHAL_DDMISCELLANEOUSCALLBACKS = core::mem::zeroed();
        let pUserMisc: *mut DDHAL_DDMISCELLANEOUSCALLBACKS =
            (*pData).lpvData as *mut DDHAL_DDMISCELLANEOUSCALLBACKS;

        write_bytes(addr_of_mut!(pMisc) as *mut u8, 0, ddmiscellaneouscallbackssize() as usize);
        pMisc.dwSize = ddmiscellaneouscallbackssize();

        pDrvInfoData.lpvData = addr_of_mut!(pMisc) as PVOID;
        pDrvInfoData.dwExpectedSize = ddmiscellaneouscallbackssize();

        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        (*pData).dwActualSize = ddmiscellaneouscallbackssize();

        /* Only one callback is supported */
        (*pUserMisc).dwFlags = pMisc.dwFlags & DDHAL_MISCCB32_GETAVAILDRIVERMEMORY;
        (*pUserMisc).GetAvailDriverMemory = cbcast!(DdGetAvailDriverMemory);

        /* UpdateNonLocalHeap/GetHeapAlignment/GetSysmemBltStatus are win9x-only */

        (*pData).ddRVal = retValue as HRESULT;
    }
    /* Misc 2 Callbacks check and setup for DirectX/ReactX */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_Miscellaneous2Callbacks)) {
        let mut pMisc: DDHAL_DDMISCELLANEOUS2CALLBACKS = core::mem::zeroed();
        let pUserMisc: *mut DDHAL_DDMISCELLANEOUS2CALLBACKS =
            (*pData).lpvData as *mut DDHAL_DDMISCELLANEOUS2CALLBACKS;

        write_bytes(addr_of_mut!(pMisc) as *mut u8, 0, ddmiscellaneous2callbackssize() as usize);
        pMisc.dwSize = ddmiscellaneous2callbackssize();

        pDrvInfoData.lpvData = addr_of_mut!(pMisc) as PVOID;
        pDrvInfoData.dwExpectedSize = ddmiscellaneous2callbackssize();

        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        (*pData).dwActualSize = ddmiscellaneous2callbackssize();

        (*pUserMisc).dwFlags = pMisc.dwFlags;

        /* Undocumented for this struct — directx/reactx alpha blend */
        if !pMisc.Reserved.is_null() {
            (*pUserMisc).Reserved = cbcast!(DdAlphaBlt);
        }
        if pMisc.CreateSurfaceEx.is_some() {
            (*pUserMisc).CreateSurfaceEx = cbcast!(DdCreateSurfaceEx);
        }
        if pMisc.GetDriverState.is_some() {
            (*pUserMisc).GetDriverState = cbcast!(NtGdiDdGetDriverState);
        }

        /* NOTE: pUserMisc->DestroyDDLocal is outdated and not touched */

        (*pData).ddRVal = retValue as HRESULT;
    }
    /* NT Callbacks check and setup for DirectX/ReactX */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_NTCallbacks)) {
        /* MS has no DDHAL_* version of these callbacks, so use PDD_* here */
        let mut pNtKernel: DD_NTCALLBACKS = core::mem::zeroed();
        let pUserNtKernel: PDD_NTCALLBACKS = (*pData).lpvData as PDD_NTCALLBACKS;

        write_bytes(addr_of_mut!(pNtKernel) as *mut u8, 0, size_of::<DD_NTCALLBACKS>());
        pNtKernel.dwSize = size_of::<DD_NTCALLBACKS>() as DWORD;

        pDrvInfoData.lpvData = addr_of_mut!(pNtKernel) as PVOID;
        pDrvInfoData.dwExpectedSize = size_of::<DD_NTCALLBACKS>() as DWORD;

        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        (*pData).dwActualSize = size_of::<DD_NTCALLBACKS>() as DWORD;

        (*pUserNtKernel).dwSize = size_of::<DD_NTCALLBACKS>() as DWORD;
        (*pUserNtKernel).dwFlags = pNtKernel.dwFlags;
        (*pUserNtKernel).FreeDriverMemory = None;

        if pNtKernel.SetExclusiveMode.is_some() {
            (*pUserNtKernel).SetExclusiveMode = cbcast!(DdSetExclusiveMode);
        }
        if pNtKernel.FlipToGDISurface.is_some() {
            (*pUserNtKernel).FlipToGDISurface = cbcast!(DdFlipToGDISurface);
        }

        (*pData).ddRVal = retValue as HRESULT;
    }
    /* D3D Callbacks version 2 */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_D3DCallbacks2)) {
        // FIXME GUID_D3DCallbacks2
    }
    /* D3D Callbacks version 3 */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_D3DCallbacks3)) {
        // FIXME GUID_D3DCallbacks3
    }
    /* D3DParseUnknownCommand Callbacks */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_D3DParseUnknownCommandCallback)) {
        // FIXME GUID_D3DParseUnknownCommandCallback
    }
    /* MotionComp Callbacks */
    else if IsEqualGUID(addr_of!((*pData).guidInfo), addr_of!(GUID_MotionCompCallbacks)) {
        // FIXME GUID_MotionCompCallbacks
    }
    /* FIXME VPE2 Callbacks (commented out in C) */
    else {
        /* set up internal buffer */
        pDrvInfoData.dwExpectedSize = (*pData).dwExpectedSize;
        pDrvInfoData.lpvData = (*pData).lpvData;

        /* We do not cover all callbacks for user mode; only kmode covers them */
        retValue = NtGdiDdGetDriverInfo(hDD, addr_of_mut!(pDrvInfoData) as PDD_GETDRIVERINFODATA);

        /* Setup return data */
        (*pData).dwActualSize = pDrvInfoData.dwActualSize;
        (*pData).lpvData = pDrvInfoData.lpvData;
        (*pData).ddRVal = retValue as HRESULT;
    }

    retValue
}

/// @implemented — D3dContextCreate
#[no_mangle]
pub unsafe extern "system" fn D3dContextCreate(pdcci: LPD3DHAL_CONTEXTCREATEDATA) -> BOOL {
    let mut hSurfZ: HANDLE = null_mut();

    if !(*pdcci).lpDDSZLcl.is_null() {
        hSurfZ = (*(*pdcci).lpDDSZLcl).hDDSurface as HANDLE;
    }

    NtGdiD3dContextCreate(
        GetDdHandle((*(*pdcci).lpDDLcl).hDD),
        (*(*pdcci).lpDDSLcl).hDDSurface as HANDLE,
        hSurfZ,
        pdcci as *mut D3DNTHAL_CONTEXTCREATEI,
    )
}

/// @implemented — DdCanCreateD3DBuffer
#[no_mangle]
pub unsafe extern "system" fn DdCanCreateD3DBuffer(CanCreateD3DBuffer: LPDDHAL_CANCREATESURFACEDATA) -> DWORD {
    /* Note: NtGdiDdCanCreateD3DBuffer and NtGdiDdCanCreateSurface merge in win32k. */
    NtGdiDdCanCreateD3DBuffer(
        GetDdHandle((*(*CanCreateD3DBuffer).lpDD).hDD),
        CanCreateD3DBuffer as PDD_CANCREATESURFACEDATA,
    )
}

/// @implemented — DdCreateD3DBuffer
#[no_mangle]
pub unsafe extern "system" fn DdCreateD3DBuffer(pCreateSurface: LPDDHAL_CREATESURFACEDATA) -> DWORD {
    let mut puhSurface: HANDLE = null_mut();
    let pSurfGBL: *mut DDRAWI_DDRAWSURFACE_GBL;
    let pSurfLcl: *mut DDRAWI_DDRAWSURFACE_LCL;
    let mut puSurfaceGlobalData: DD_SURFACE_GLOBAL = core::mem::zeroed();
    let mut puSurfaceMoreData: DD_SURFACE_MORE = core::mem::zeroed();
    let mut puSurfaceLocalData: DD_SURFACE_LOCAL = core::mem::zeroed();
    let retValue: DWORD;

    /* Zero all local memory */
    write_bytes(addr_of_mut!(puSurfaceGlobalData) as *mut u8, 0, size_of::<DD_SURFACE_GLOBAL>());
    write_bytes(addr_of_mut!(puSurfaceMoreData) as *mut u8, 0, size_of::<DD_SURFACE_MORE>());
    write_bytes(addr_of_mut!(puSurfaceLocalData) as *mut u8, 0, size_of::<DD_SURFACE_LOCAL>());

    (*pCreateSurface).dwSCnt = 1;
    pSurfLcl = *(*pCreateSurface).lplpSList.add(0);
    pSurfGBL = (*pSurfLcl).lpGbl;

    /* Convert DDRAWI_DDRAWSURFACE_GBL to DD_SURFACE_GLOBAL */
    puSurfaceGlobalData.wWidth = (*pSurfGBL).wWidth as DWORD;
    puSurfaceGlobalData.wHeight = (*pSurfGBL).wHeight as DWORD;
    puSurfaceGlobalData.__bindgen_anon_3.dwLinearSize = (*pSurfGBL).__bindgen_anon_4.dwLinearSize;
    puSurfaceGlobalData.fpVidMem = (*pSurfGBL).fpVidMem;
    puSurfaceGlobalData.__bindgen_anon_2.dwBlockSizeX = (*pSurfGBL).__bindgen_anon_2.dwBlockSizeX;
    puSurfaceGlobalData.__bindgen_anon_1.dwBlockSizeY = (*pSurfGBL).__bindgen_anon_1.dwBlockSizeY;

    /* Convert DDRAWI_DDRAWSURFACE_MORE to DD_SURFACE_MORE */
    puSurfaceMoreData.dwSurfaceHandle = (*(*pSurfLcl).lpSurfMore).dwSurfaceHandle;
    puSurfaceMoreData.ddsCapsEx.dwCaps2 = (*(*pSurfLcl).lpSurfMore).ddsCapsEx.dwCaps2;
    puSurfaceMoreData.ddsCapsEx.dwCaps3 = (*(*pSurfLcl).lpSurfMore).ddsCapsEx.dwCaps3;
    puSurfaceMoreData.ddsCapsEx.__bindgen_anon_1.dwCaps4 = (*(*pSurfLcl).lpSurfMore).ddsCapsEx.__bindgen_anon_1.dwCaps4;

    /* Convert DDRAWI_DDRAWSURFACE_LCL to DD_SURFACE_LOCAL */
    puSurfaceLocalData.dwFlags = (*pSurfLcl).dwFlags;
    puSurfaceLocalData.ddsCaps.dwCaps = (*pSurfLcl).ddsCaps.dwCaps;

    /* Call win32k */
    retValue = NtGdiDdCreateD3DBuffer(
        GetDdHandle((*(*pCreateSurface).lpDD).hDD),
        addr_of_mut!((*pSurfLcl).hDDSurface) as *mut HANDLE,
        (*pCreateSurface).lpDDSurfaceDesc,
        &mut puSurfaceGlobalData,
        &mut puSurfaceLocalData,
        &mut puSurfaceMoreData,
        pCreateSurface as *mut DD_CREATESURFACEDATA,
        &mut puhSurface,
    );

    /* Setup surface handle if we got one back */
    if !puhSurface.is_null() {
        (*(*(*pCreateSurface).lplpSList.add(0))).hDDSurface = puhSurface as ULONG_PTR;
    }

    /* Convert DD_SURFACE_GLOBAL to DDRAWI_DDRAWSURFACE_GBL */
    (*pSurfGBL).__bindgen_anon_4.dwLinearSize = puSurfaceGlobalData.__bindgen_anon_3.dwLinearSize;
    (*pSurfGBL).fpVidMem = puSurfaceGlobalData.fpVidMem;
    (*pSurfGBL).__bindgen_anon_2.dwBlockSizeX = puSurfaceGlobalData.__bindgen_anon_2.dwBlockSizeX;
    (*pSurfGBL).__bindgen_anon_1.dwBlockSizeY = puSurfaceGlobalData.__bindgen_anon_1.dwBlockSizeY;

    retValue
}

/// @implemented — DdDestroyD3DBuffer
#[no_mangle]
pub unsafe extern "system" fn DdDestroyD3DBuffer(pDestroySurface: LPDDHAL_DESTROYSURFACEDATA) -> DWORD {
    let mut retValue: DWORD = 0;
    if (*(*pDestroySurface).lpDDSurface).hDDSurface != 0 {
        /* Call win32k */
        retValue = NtGdiDdDestroyD3DBuffer((*(*pDestroySurface).lpDDSurface).hDDSurface as HANDLE);
    }

    retValue
}

/// @implemented — DdLockD3D
#[no_mangle]
pub unsafe extern "system" fn DdLockD3D(Lock: LPDDHAL_LOCKDATA) -> DWORD {
    /* Call win32k */
    NtGdiDdLockD3D(
        (*(*Lock).lpDDSurface).hDDSurface as HANDLE,
        Lock as PDD_LOCKDATA,
    )
}

/// @implemented — DdUnlockD3D
#[no_mangle]
pub unsafe extern "system" fn DdUnlockD3D(pUnlock: *mut _DDHAL_UNLOCKDATA) -> DWORD {
    /* Call win32k */
    // param renamed Unlock->pUnlock (glob const `Unlock` const-pattern footgun — see DdUnlock)
    NtGdiDdUnlock(
        (*(*pUnlock).lpDDSurface).hDDSurface as HANDLE,
        pUnlock as PDD_UNLOCKDATA,
    )
}

/* PRIVATE FUNCTIONS *********************************************************/

#[no_mangle]
pub unsafe extern "system" fn bDDCreateSurface(
    pSurface: LPDDRAWI_DDRAWSURFACE_LCL,
    bComplete: BOOL,
) -> BOOL {
    let mut SurfaceLocal: DD_SURFACE_LOCAL = core::mem::zeroed();
    let mut SurfaceGlobal: DD_SURFACE_GLOBAL = core::mem::zeroed();
    let mut SurfaceMore: DD_SURFACE_MORE = core::mem::zeroed();

    /* Zero struct */
    write_bytes(addr_of_mut!(SurfaceLocal) as *mut u8, 0, size_of::<DD_SURFACE_LOCAL>());
    write_bytes(addr_of_mut!(SurfaceGlobal) as *mut u8, 0, size_of::<DD_SURFACE_GLOBAL>());
    write_bytes(addr_of_mut!(SurfaceMore) as *mut u8, 0, size_of::<DD_SURFACE_MORE>());

    /* Set up SurfaceLocal struct */
    SurfaceLocal.ddsCaps.dwCaps = (*pSurface).ddsCaps.dwCaps;
    SurfaceLocal.dwFlags = (*pSurface).dwFlags;

    /* Set up SurfaceMore struct */
    copy_nonoverlapping(
        addr_of!((*pSurface).ddckCKDestBlt) as *const u8,
        addr_of_mut!(SurfaceMore.ddsCapsEx) as *mut u8,
        size_of::<DDSCAPSEX>(),
    );
    SurfaceMore.dwSurfaceHandle = (*(*pSurface).lpSurfMore).dwSurfaceHandle;

    /* Set up SurfaceGlobal struct */
    SurfaceGlobal.fpVidMem = (*(*pSurface).lpGbl).fpVidMem;
    SurfaceGlobal.__bindgen_anon_3.dwLinearSize = (*(*pSurface).lpGbl).__bindgen_anon_4.dwLinearSize;
    SurfaceGlobal.wHeight = (*(*pSurface).lpGbl).wHeight as DWORD;
    SurfaceGlobal.wWidth = (*(*pSurface).lpGbl).wWidth as DWORD;

    /* Check if we have a pixel format */
    if ((*pSurface).dwFlags & DDSD_PIXELFORMAT) != 0 {
        /* Use global one */
        SurfaceGlobal.ddpfSurface = (*(*(*pSurface).lpGbl).__bindgen_anon_3.lpDD).vmiData.ddpfDisplay;
        SurfaceGlobal.ddpfSurface.dwSize = size_of::<DDPIXELFORMAT>() as DWORD;
    } else {
        /* Use local one */
        SurfaceGlobal.ddpfSurface = (*(*(*pSurface).lpGbl).__bindgen_anon_3.lpDD).vmiData.ddpfDisplay;
    }

    /* Create the object */
    (*pSurface).hDDSurface = NtGdiDdCreateSurfaceObject(
        GetDdHandle((*(*(*pSurface).lpGbl).__bindgen_anon_3.lpDD).hDD),
        (*pSurface).hDDSurface as HANDLE,
        &mut SurfaceLocal,
        &mut SurfaceMore,
        &mut SurfaceGlobal,
        bComplete,
    ) as DWORD_PTR;

    /* Return status */
    if (*pSurface).hDDSurface != 0 {
        return TRUE;
    }
    FALSE
}

/* PUBLIC FUNCTIONS **********************************************************/

/// GdiEntry1 — C source name DdCreateDirectDrawObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry1(
    pDirectDrawGlobal: LPDDRAWI_DIRECTDRAW_GBL,
    mut hdc: HDC,
) -> BOOL {
    let mut Return: BOOL = FALSE;

    /* Check if the global hDC (hdc == 0) is being used */
    if hdc.is_null() {
        /* We'll only allow this if the global object doesn't exist yet */
        if ghDirectDraw.is_null() {
            /* Create the DC */
            hdc = CreateDCW(DISPLAY_W.as_ptr(), null_mut(), null_mut(), null_mut());
            if !hdc.is_null() {
                /* Create the DDraw Object */
                ghDirectDraw = NtGdiDdCreateDirectDrawObject(hdc);

                /* Delete our DC */
                DeleteDC(hdc);
            }
        }

        /* If we created the object, or had one ... */
        if !ghDirectDraw.is_null() {
            /* Increase count and set success */
            gcDirectDraw += 1;
            Return = TRUE;
        }

        /* Zero the handle */
        (*pDirectDrawGlobal).hDD = 0;
    } else {
        /* Using the per-process object, so create it */
        (*pDirectDrawGlobal).hDD = NtGdiDdCreateDirectDrawObject(hdc) as ULONG_PTR;

        /* Set the return value */
        Return = if (*pDirectDrawGlobal).hDD != 0 { TRUE } else { FALSE };
    }

    /* Return to caller */
    Return
}

/// GdiEntry2 — C source name DdQueryDirectDrawObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry2(
    pDirectDrawGlobal: LPDDRAWI_DIRECTDRAW_GBL,
    pHalInfo: LPDDHALINFO,
    pDDCallbacks: LPDDHAL_DDCALLBACKS,
    pDDSurfaceCallbacks: LPDDHAL_DDSURFACECALLBACKS,
    pDDPaletteCallbacks: LPDDHAL_DDPALETTECALLBACKS,
    pD3dCallbacks: LPD3DHAL_CALLBACKS,
    pD3dDriverData: LPD3DHAL_GLOBALDRIVERDATA,
    pD3dBufferCallbacks: LPDDHAL_DDEXEBUFCALLBACKS,
    pD3dTextureFormats: LPDDSURFACEDESC,
    pdwFourCC: LPDWORD,
    pvmList: LPVIDMEM,
) -> BOOL {
    let mut VidMemList: PVIDEOMEMORY = null_mut();
    let mut HalInfo: DD_HALINFO = core::mem::zeroed();
    let mut D3dCallbacks: D3DNTHAL_CALLBACKS = core::mem::zeroed();
    let mut D3dDriverData: D3DNTHAL_GLOBALDRIVERDATA = core::mem::zeroed();
    let mut D3dBufferCallbacks: DD_D3DBUFCALLBACKS = core::mem::zeroed();
    let mut CallbackFlags: [DWORD; 3] = [0; 3];
    let mut dwNumHeaps: DWORD = 0;
    let mut FourCCs: DWORD = 0;
    let mut Flags: DWORD;
    let mut retVal: BOOL = TRUE;

    /* Clear the structures */
    write_bytes(addr_of_mut!(HalInfo) as *mut u8, 0, size_of::<DD_HALINFO>());
    write_bytes(addr_of_mut!(D3dCallbacks) as *mut u8, 0, size_of::<D3DNTHAL_CALLBACKS>());
    write_bytes(addr_of_mut!(D3dDriverData) as *mut u8, 0, size_of::<D3DNTHAL_GLOBALDRIVERDATA>());
    write_bytes(addr_of_mut!(D3dBufferCallbacks) as *mut u8, 0, size_of::<DD_D3DBUFCALLBACKS>());
    write_bytes(addr_of_mut!(CallbackFlags) as *mut u8, 0, size_of::<DWORD>() * 3);

    /* Note: XP always allocs 24*sizeof(VIDEOMEMORY) of pvmList; we match it */
    if !pvmList.is_null() && (*pHalInfo).vmiData.dwNumHeaps != 0 {
        VidMemList = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            ((size_of::<VIDEOMEMORY>() * 24) * (*pHalInfo).vmiData.dwNumHeaps as usize) as SIZE_T,
        ) as PVIDEOMEMORY;
    }

    /* Do the query */
    if NtGdiDdQueryDirectDrawObject(
        GetDdHandle((*pDirectDrawGlobal).hDD),
        &mut HalInfo,
        CallbackFlags.as_mut_ptr(),
        &mut D3dCallbacks,
        &mut D3dDriverData,
        &mut D3dBufferCallbacks,
        pD3dTextureFormats,
        &mut dwNumHeaps,
        VidMemList,
        &mut FourCCs,
        pdwFourCC,
    ) == 0
    {
        /* We failed, free the memory and return */
        retVal = FALSE;
        if !VidMemList.is_null() {
            HeapFree(GetProcessHeap(), 0, VidMemList as LPVOID);
        }
        return retVal;
    }

    /* Clear the incoming pointer */
    write_bytes(pHalInfo as *mut u8, 0, size_of::<DDHALINFO>());

    /* Convert all the data */
    (*pHalInfo).dwSize = size_of::<DDHALINFO>() as DWORD;
    (*pHalInfo).lpDDCallbacks = pDDCallbacks;
    (*pHalInfo).lpDDSurfaceCallbacks = pDDSurfaceCallbacks;
    (*pHalInfo).lpDDPaletteCallbacks = pDDPaletteCallbacks;

    /* Check for NT5+ D3D Data */
    if D3dCallbacks.dwSize != 0 && D3dDriverData.dwSize != 0 {
        /* Write these down */
        (*pHalInfo).lpD3DGlobalDriverData = pD3dDriverData as ULONG_PTR;
        (*pHalInfo).lpD3DHALCallbacks = pD3dCallbacks as ULONG_PTR;

        /* Check for Buffer Callbacks */
        if D3dBufferCallbacks.dwSize != 0 {
            /* Write this one too */
            (*pHalInfo).lpDDExeBufCallbacks = pD3dBufferCallbacks;
        }
    }

    /* Continue converting the rest */
    (*pHalInfo).vmiData.dwFlags = HalInfo.vmiData.dwFlags;
    (*pHalInfo).vmiData.dwDisplayWidth = HalInfo.vmiData.dwDisplayWidth;
    (*pHalInfo).vmiData.dwDisplayHeight = HalInfo.vmiData.dwDisplayHeight;
    (*pHalInfo).vmiData.lDisplayPitch = HalInfo.vmiData.lDisplayPitch;
    (*pHalInfo).vmiData.fpPrimary = 0;

    copy_nonoverlapping(
        addr_of!(HalInfo.vmiData.ddpfDisplay) as *const u8,
        addr_of_mut!((*pHalInfo).vmiData.ddpfDisplay) as *mut u8,
        size_of::<DDPIXELFORMAT>(),
    );

    (*pHalInfo).vmiData.dwOffscreenAlign = HalInfo.vmiData.dwOffscreenAlign;
    (*pHalInfo).vmiData.dwOverlayAlign = HalInfo.vmiData.dwOverlayAlign;
    (*pHalInfo).vmiData.dwTextureAlign = HalInfo.vmiData.dwTextureAlign;
    (*pHalInfo).vmiData.dwZBufferAlign = HalInfo.vmiData.dwZBufferAlign;
    (*pHalInfo).vmiData.dwAlphaAlign = HalInfo.vmiData.dwAlphaAlign;

    (*pHalInfo).vmiData.dwNumHeaps = dwNumHeaps;
    (*pHalInfo).vmiData.pvmList = pvmList;

    copy_nonoverlapping(
        addr_of!(HalInfo.ddCaps) as *const u8,
        addr_of_mut!((*pHalInfo).ddCaps) as *mut u8,
        size_of::<DDCORECAPS>(),
    );

    (*pHalInfo).ddCaps.dwNumFourCCCodes = FourCCs;
    (*pHalInfo).lpdwFourCC = pdwFourCC;

    /* always force rop 0x1000 for HAL — only source copy is supported */
    (*pHalInfo).ddCaps.dwRops[6] = 0x1000;

    /* Force DDHALINFO_GETDRIVERINFOSET (XP-compat hack for bad drivers) */
    (*pHalInfo).dwFlags = (HalInfo.dwFlags & !DDHALINFO_GETDRIVERINFOSET) | DDHALINFO_GETDRIVERINFOSET;
    (*pHalInfo).GetDriverInfo = cbcast!(DdGetDriverInfo);

    /* Now check if we got any DD callbacks */
    if !pDDCallbacks.is_null() {
        /* Zero the structure */
        write_bytes(pDDCallbacks as *mut u8, 0, size_of::<DDHAL_DDCALLBACKS>());
        (*pDDCallbacks).dwSize = size_of::<DDHAL_DDCALLBACKS>() as DWORD;

        /* Force DDHAL_CB32_CREATESURFACE (XP-compat hack for bad drivers) */
        Flags = (CallbackFlags[0] & !DDHAL_CB32_CREATESURFACE) | DDHAL_CB32_CREATESURFACE;
        (*pDDCallbacks).dwFlags = Flags;

        /* Write the always-on functions */
        (*pDDCallbacks).CreateSurface = cbcast!(DdCreateSurface);

        /* Now write the pointers, if applicable */
        if (Flags & DDHAL_CB32_WAITFORVERTICALBLANK) != 0 {
            (*pDDCallbacks).WaitForVerticalBlank = cbcast!(DdWaitForVerticalBlank);
        }
        if (Flags & DDHAL_CB32_CANCREATESURFACE) != 0 {
            (*pDDCallbacks).CanCreateSurface = cbcast!(DdCanCreateSurface);
        }
        if (Flags & DDHAL_CB32_GETSCANLINE) != 0 {
            (*pDDCallbacks).GetScanLine = cbcast!(DdGetScanLine);
        }
    }

    /* Check for DD Surface Callbacks */
    if !pDDSurfaceCallbacks.is_null() {
        write_bytes(pDDSurfaceCallbacks as *mut u8, 0, size_of::<DDHAL_DDSURFACECALLBACKS>());
        (*pDDSurfaceCallbacks).dwSize = size_of::<DDHAL_DDSURFACECALLBACKS>() as DWORD;

        /* Force LOCK|UNLOCK|SETCOLORKEY|DESTROYSURFACE (XP-compat hack) */
        Flags = (CallbackFlags[1]
            & !(DDHAL_SURFCB32_LOCK
                | DDHAL_SURFCB32_UNLOCK
                | DDHAL_SURFCB32_SETCOLORKEY
                | DDHAL_SURFCB32_DESTROYSURFACE))
            | (DDHAL_SURFCB32_LOCK
                | DDHAL_SURFCB32_UNLOCK
                | DDHAL_SURFCB32_SETCOLORKEY
                | DDHAL_SURFCB32_DESTROYSURFACE);

        (*pDDSurfaceCallbacks).dwFlags = Flags;

        /* Write the always-on functions */
        (*pDDSurfaceCallbacks).Lock = cbcast!(DdLock);
        (*pDDSurfaceCallbacks).Unlock = cbcast!(DdUnlock);
        (*pDDSurfaceCallbacks).SetColorKey = cbcast!(DdSetColorKey);
        (*pDDSurfaceCallbacks).DestroySurface = cbcast!(DdDestroySurface);

        /* Write the optional ones */
        if (Flags & DDHAL_SURFCB32_FLIP) != 0 {
            (*pDDSurfaceCallbacks).Flip = cbcast!(DdFlip);
        }
        if (Flags & DDHAL_SURFCB32_BLT) != 0 {
            (*pDDSurfaceCallbacks).Blt = cbcast!(DdBlt);
        }
        if (Flags & DDHAL_SURFCB32_GETBLTSTATUS) != 0 {
            (*pDDSurfaceCallbacks).GetBltStatus = cbcast!(DdGetBltStatus);
        }
        if (Flags & DDHAL_SURFCB32_GETFLIPSTATUS) != 0 {
            (*pDDSurfaceCallbacks).GetFlipStatus = cbcast!(DdGetFlipStatus);
        }
        if (Flags & DDHAL_SURFCB32_UPDATEOVERLAY) != 0 {
            (*pDDSurfaceCallbacks).UpdateOverlay = cbcast!(DdUpdateOverlay);
        }
        if (Flags & DDHAL_SURFCB32_SETOVERLAYPOSITION) != 0 {
            (*pDDSurfaceCallbacks).SetOverlayPosition = cbcast!(DdSetOverlayPosition);
        }
        if (Flags & DDHAL_SURFCB32_ADDATTACHEDSURFACE) != 0 {
            (*pDDSurfaceCallbacks).AddAttachedSurface = cbcast!(DdAddAttachedSurface);
        }
    }

    /* Check for DD Palette Callbacks — dead for user mode, just report support */
    if !pDDPaletteCallbacks.is_null() {
        write_bytes(pDDPaletteCallbacks as *mut u8, 0, size_of::<DDHAL_DDPALETTECALLBACKS>());
        (*pDDPaletteCallbacks).dwSize = size_of::<DDHAL_DDPALETTECALLBACKS>() as DWORD;
        (*pDDPaletteCallbacks).dwFlags = CallbackFlags[2];
    }

    if !pD3dCallbacks.is_null() {
        write_bytes(pD3dCallbacks as *mut u8, 0, size_of::<DDHAL_DDEXEBUFCALLBACKS>());

        /* Check if we have one */
        if D3dCallbacks.dwSize != 0 {
            (*pD3dCallbacks).dwSize = size_of::<DDHAL_DDEXEBUFCALLBACKS>() as DWORD;

            /* Now check for each callback */
            if D3dCallbacks.ContextCreate.is_some() {
                (*pD3dCallbacks).ContextCreate = cbcast!(D3dContextCreate);
            }
            if D3dCallbacks.ContextDestroy.is_some() {
                (*pD3dCallbacks).ContextDestroy = cbcast!(NtGdiD3dContextDestroy);
            }
            if D3dCallbacks.ContextDestroyAll.is_some() {
                (*pD3dCallbacks).ContextDestroyAll = cbcast!(NtGdiD3dContextDestroyAll);
            }
        }
    }

    /* Check for D3D Driver Data */
    if !pD3dDriverData.is_null() {
        /* Copy the struct */
        *pD3dDriverData = D3dDriverData;

        /* Write the pointer to the texture formats */
        (*pD3dDriverData).lpTextureFormats = pD3dTextureFormats;
    }

    /* Check for D3D Buffer Callbacks */
    if !pD3dBufferCallbacks.is_null() {
        write_bytes(pD3dBufferCallbacks as *mut u8, 0, size_of::<DDHAL_DDEXEBUFCALLBACKS>());

        if D3dBufferCallbacks.dwSize != 0 {
            (*pD3dBufferCallbacks).dwSize = D3dBufferCallbacks.dwSize;
            (*pD3dBufferCallbacks).dwFlags = D3dBufferCallbacks.dwFlags;

            if D3dBufferCallbacks.CanCreateD3DBuffer.is_some() {
                (*pD3dBufferCallbacks).CanCreateExecuteBuffer = cbcast!(DdCanCreateD3DBuffer);
            }
            if D3dBufferCallbacks.CreateD3DBuffer.is_some() {
                (*pD3dBufferCallbacks).CreateExecuteBuffer = cbcast!(DdCreateD3DBuffer);
            }
            if D3dBufferCallbacks.DestroyD3DBuffer.is_some() {
                (*pD3dBufferCallbacks).DestroyExecuteBuffer = cbcast!(DdDestroyD3DBuffer);
            }
            if D3dBufferCallbacks.LockD3DBuffer.is_some() {
                (*pD3dBufferCallbacks).LockExecuteBuffer = cbcast!(DdLockD3D);
            }
            if D3dBufferCallbacks.UnlockD3DBuffer.is_some() {
                (*pD3dBufferCallbacks).UnlockExecuteBuffer = cbcast!(DdUnlockD3D);
            }
        }
    }

    /* FIXME VidMemList */

    /* cleanup */
    if !VidMemList.is_null() {
        HeapFree(GetProcessHeap(), 0, VidMemList as LPVOID);
    }

    retVal
}

/// GdiEntry3 — C source name DdDeleteDirectDrawObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry3(pDirectDrawGlobal: LPDDRAWI_DIRECTDRAW_GBL) -> BOOL {
    let mut Return: BOOL = FALSE;

    /* If this is the global object */
    if (*pDirectDrawGlobal).hDD != 0 {
        /* Free it */
        Return = NtGdiDdDeleteDirectDrawObject((*pDirectDrawGlobal).hDD as HANDLE);
        if Return != 0 {
            (*pDirectDrawGlobal).hDD = 0;
        }
    } else if !ghDirectDraw.is_null() {
        /* Always success here */
        Return = TRUE;

        /* Make sure this is the last instance */
        gcDirectDraw -= 1;
        if gcDirectDraw == 0 {
            /* Delete the object */
            Return = NtGdiDdDeleteDirectDrawObject(ghDirectDraw);
            if Return != 0 {
                ghDirectDraw = null_mut();
            }
        }
    }

    /* Return */
    Return
}

/// GdiEntry4 — C source name DdCreateSurfaceObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry4(
    pSurfaceLocal: LPDDRAWI_DDRAWSURFACE_LCL,
    bPrimarySurface: BOOL,
) -> BOOL {
    let _ = bPrimarySurface;
    bDDCreateSurface(pSurfaceLocal, TRUE)
}

/// GdiEntry5 — C source name DdDeleteSurfaceObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry5(pSurfaceLocal: LPDDRAWI_DDRAWSURFACE_LCL) -> BOOL {
    let mut Return: BOOL = FALSE;

    /* Make sure there is one */
    if (*pSurfaceLocal).hDDSurface != 0 {
        /* Delete it */
        Return = NtGdiDdDeleteSurfaceObject((*pSurfaceLocal).hDDSurface as HANDLE);
        (*pSurfaceLocal).hDDSurface = 0;
    }

    Return
}

/// GdiEntry6 — C source name DdResetVisrgn (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry6(
    pSurfaceLocal: LPDDRAWI_DDRAWSURFACE_LCL,
    hWnd: HWND,
) -> BOOL {
    /* Call win32k directly */
    NtGdiDdResetVisrgn((*pSurfaceLocal).hDDSurface as HANDLE, hWnd)
}

/// GdiEntry7 — C source name DdGetDC (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry7(
    pSurfaceLocal: LPDDRAWI_DDRAWSURFACE_LCL,
    pColorTable: LPPALETTEENTRY,
) -> HDC {
    /* Call win32k directly */
    NtGdiDdGetDC((*pSurfaceLocal).hDDSurface as HANDLE, pColorTable)
}

/// GdiEntry8 — C source name DdReleaseDC (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry8(pSurfaceLocal: LPDDRAWI_DDRAWSURFACE_LCL) -> BOOL {
    /* Call win32k directly */
    NtGdiDdReleaseDC((*pSurfaceLocal).hDDSurface as HANDLE)
}

/// GdiEntry9 — C source name DdCreateDIBSection (ddrawgdi.h #define)
/// @unimplemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry9(
    hdc: HDC,
    pbmi: *const BITMAPINFO,
    iUsage: UINT,
    ppvBits: *mut *mut core::ffi::c_void,
    hSectionApp: HANDLE,
    dwOffset: DWORD,
) -> HBITMAP {
    let _ = (hdc, pbmi, iUsage, ppvBits, hSectionApp, dwOffset);
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    null_mut()
}

/// GdiEntry10 — C source name DdReenableDirectDrawObject (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry10(
    pDirectDrawGlobal: LPDDRAWI_DIRECTDRAW_GBL,
    pbNewMode: *mut BOOL,
) -> BOOL {
    /* Call win32k directly */
    NtGdiDdReenableDirectDrawObject(GetDdHandle((*pDirectDrawGlobal).hDD), pbNewMode)
}

/// GdiEntry11 — C source name DdAttachSurface (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry11(
    pSurfaceFrom: LPDDRAWI_DDRAWSURFACE_LCL,
    pSurfaceTo: LPDDRAWI_DDRAWSURFACE_LCL,
) -> BOOL {
    /* Create Surface if there isn't one */
    if (*pSurfaceFrom).hDDSurface == 0 && bDDCreateSurface(pSurfaceFrom, FALSE) == 0 {
        return FALSE;
    }

    /* Create Surface if there isn't one */
    if (*pSurfaceTo).hDDSurface == 0 && bDDCreateSurface(pSurfaceTo, FALSE) == 0 {
        return FALSE;
    }

    /* Call win32k */
    NtGdiDdAttachSurface(
        (*pSurfaceFrom).hDDSurface as HANDLE,
        (*pSurfaceTo).hDDSurface as HANDLE,
    )
}

/// GdiEntry12 — C source name DdUnattachSurface (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry12(
    pSurface: LPDDRAWI_DDRAWSURFACE_LCL,
    pSurfaceAttached: LPDDRAWI_DDRAWSURFACE_LCL,
) {
    /* Call win32k */
    NtGdiDdUnattachSurface(
        (*pSurface).hDDSurface as HANDLE,
        (*pSurfaceAttached).hDDSurface as HANDLE,
    );
}

/// GdiEntry13 — C source name DdQueryDisplaySettingsUniqueness (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry13() -> ULONG {
    (*GdiSharedHandleTable).flDeviceUniq
}

/// GdiEntry14 — C source name DdGetDxHandle (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry14(
    pDDraw: LPDDRAWI_DIRECTDRAW_LCL,
    pSurface: LPDDRAWI_DDRAWSURFACE_LCL,
    bRelease: BOOL,
) -> HANDLE {
    let mut hDD: HANDLE = null_mut();
    let mut hSurface: HANDLE = null_mut();

    /* Check if we already have a surface */
    if pSurface.is_null() {
        /* We don't have one, use the DirectDraw Object handle instead */
        hDD = GetDdHandle((*(*pDDraw).lpGbl).hDD);
    } else {
        hSurface = (*pSurface).hDDSurface as HANDLE;
    }

    /* Call the API */
    NtGdiDdGetDxHandle(hDD, hSurface, bRelease)
}

/// GdiEntry15 — C source name DdSetGammaRamp (ddrawgdi.h #define)
/// @implemented
#[no_mangle]
pub unsafe extern "system" fn GdiEntry15(
    pDDraw: LPDDRAWI_DIRECTDRAW_LCL,
    hdc: HDC,
    lpGammaRamp: LPVOID,
) -> BOOL {
    /* Call win32k directly */
    NtGdiDdSetGammaRamp(GetDdHandle((*(*pDDraw).lpGbl).hDD), hdc, lpGammaRamp)
}
