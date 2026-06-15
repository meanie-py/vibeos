//! dxg-private types + constants, translated from win32ss/reactx/dxg/dxg_int.h.
//! These are NOT in win32k-sys (they're driver-private): the DD handle-manager
//! entry/object structs, the ObjType_*/CapOver_* tags, and the DXENG_FUNCTIONS
//! thunk table (dxg's typed view of win32k's exported gaEngFuncs[] DRVFN array).
//! Base types (HGDIOBJ/HDEV/HDC/DD_SURFACE_LOCAL/PEDD_*/…) come from win32k-sys.

#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]

use win32k_sys::win32k::*;

// dxg treats this as opaque (dxg_int.h:28). `PDC` comes from win32k-sys.
pub type PW32THREAD = PVOID;

// `VOID` (the C `void` "value" type) is used in the translated modules as the
// target of `*mut VOID` casts in InterlockedDecrement(...) calls. win32k-sys
// only exposes PVOID, so alias VOID to the unit-ish c_void here.
pub type VOID = core::ffi::c_void;

// HRESULT/DirectDraw status codes the C pulls from <ddraw.h>/<winerror.h>.
// (Not exported by win32k-sys.) See sdk/include/psdk/ddraw.h + winerror.h.
pub const DD_OK: DWORD = 0; // S_OK
pub const DDERR_UNSUPPORTED: DWORD = 0x80004001; // E_NOTIMPL
pub const DDERR_CURRENTLYNOTAVAIL: DWORD = 0x88760028; // MAKE_DDHRESULT(40)
pub const E_FAIL: i32 = 0x80004005u32 as i32;

// NTSTATUS not present in ntkrnl-sys's curated set (others come from ntkrnl_sys::ddk).
pub const STATUS_INTERNAL_ERROR: NTSTATUS = 0xC00000E5u32 as i32;

// NOTE: the clean DxEGShDevData_* selectors are provided by win32k-sys's
// enum_aliases module (re-exported through `win32k_sys::win32k::*`), so they are
// NOT redefined here — doing so would make every use ambiguous across the two
// globs. We only keep the `DXEGSHDEVDATA` type alias below (win32k exposes the
// underscored `_DXEGSHDEVDATA` only).

// handle-manager limits (dxg_int.h:9-10)
pub const DDHMG_HANDLE_LIMIT: DWORD_PTR = 0x200000;
#[inline]
pub unsafe fn DDHMG_HTOI(DdHandle: HANDLE) -> DWORD_PTR {
    (DdHandle as DWORD_PTR) & (DDHMG_HANDLE_LIMIT - 1)
}

// capability-override flags (dxg_int.h:45-49)
pub const CapOver_DisableAccel: DWORD = 0x1;
pub const CapOver_DisableD3DDDAccel: DWORD = 0x2;
pub const CapOver_DisableD3DAccel: DWORD = 0x4;
pub const CapOver_DisableOGL: DWORD = 0x8;
pub const CapOver_DisableEscapes: DWORD = 0x10;

// object-type tags (dxg_int.h:51-55)
pub const ObjType_DDLOCAL_TYPE: UCHAR = 1;
pub const ObjType_DDSURFACE_TYPE: UCHAR = 2;
pub const ObjType_DDCONTEXT_TYPE: UCHAR = 3;
pub const ObjType_DDVIDEOPORT_TYPE: UCHAR = 4;
pub const ObjType_DDMOTIONCOMP_TYPE: UCHAR = 5;

// `DXEGSHDEVDATA` index type for the DxEngGetHdevData/SetHdevData thunks.
// (win32k-sys exposes only the underscored `_DXEGSHDEVDATA`; the clean
// `DxEGShDevData_*` selector constants come from win32k-sys's enum_aliases.)
pub type DXEGSHDEVDATA = core::ffi::c_uint;

// DD_BASEOBJECT (dxg_int.h:30-37)
#[repr(C)]
pub struct DD_BASEOBJECT {
    pub hHmgr: HGDIOBJ,
    pub ulShareCount: ULONG,
    pub cExclusiveLock: USHORT,
    pub BaseFlags: USHORT,
    pub Tid: PW32THREAD,
}
pub type PDD_BASEOBJECT = *mut DD_BASEOBJECT;

// DD_ENTRY (dxg_int.h:57-67) — leading anonymous union { pobj; NextFree }.
#[repr(C)]
pub union DD_ENTRY_0 {
    pub pobj: PDD_BASEOBJECT,
    pub NextFree: ULONG,
}
#[repr(C)]
pub struct DD_ENTRY {
    pub u: DD_ENTRY_0,
    pub Pid: HANDLE,
    pub FullUnique: USHORT,
    pub Objt: UCHAR,
}
pub type PDD_ENTRY = *mut DD_ENTRY;

// EDD_SURFACE_LOCAL (dxg_int.h:69-73)
#[repr(C)]
pub struct EDD_SURFACE_LOCAL {
    pub Object: DD_BASEOBJECT,
    pub Surfacelcl: DD_SURFACE_LOCAL,
}
pub type PEDD_SURFACE_LOCAL = *mut EDD_SURFACE_LOCAL;

// APIENTRY thunk pointers (dxg_int.h:76-117). APIENTRY = stdcall on i386,
// cdecl on amd64/arm64 → `extern "system"` resolves per-arch.
pub type PFN_DxEngNUIsTermSrv = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type PFN_DxEngScreenAccessCheck = Option<unsafe extern "system" fn() -> DWORD>;
pub type PFN_DxEngRedrawDesktop = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type PFN_DxEngDispUniq = Option<unsafe extern "system" fn() -> ULONG>;
pub type PFN_DxEngIncDispUniq = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type PFN_DxEngVisRgnUniq = Option<unsafe extern "system" fn() -> ULONG>;
pub type PFN_DxEngLockShareSem = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type PFN_DxEngUnlockShareSem = Option<unsafe extern "system" fn() -> BOOLEAN>;
pub type PFN_DxEngEnumerateHdev = Option<unsafe extern "system" fn(*mut HDEV) -> *mut HDEV>;
pub type PFN_DxEngLockHdev = Option<unsafe extern "system" fn(HDEV) -> BOOLEAN>;
pub type PFN_DxEngUnlockHdev = Option<unsafe extern "system" fn(HDEV) -> BOOLEAN>;
pub type PFN_DxEngIsHdevLockedByCurrentThread = Option<unsafe extern "system" fn(HDEV) -> BOOLEAN>;
pub type PFN_DxEngReferenceHdev = Option<unsafe extern "system" fn(HDEV) -> BOOLEAN>;
pub type PFN_DxEngUnreferenceHdev = Option<unsafe extern "system" fn(HDEV) -> BOOLEAN>;
pub type PFN_DxEngGetDeviceGammaRamp = Option<unsafe extern "system" fn(HDEV, PGAMMARAMP) -> BOOL>;
pub type PFN_DxEngSetDeviceGammaRamp = Option<unsafe extern "system" fn(HDEV, PGAMMARAMP, BOOL) -> BOOLEAN>;
pub type PFN_DxEngSpTearDownSprites = Option<unsafe extern "system" fn(DWORD, DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSpUnTearDownSprites = Option<unsafe extern "system" fn(DWORD, DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSpSpritesVisible = Option<unsafe extern "system" fn(DWORD) -> DWORD>;
pub type PFN_DxEngGetHdevData = Option<unsafe extern "system" fn(HDEV, DXEGSHDEVDATA) -> DWORD_PTR>;
pub type PFN_DxEngSetHdevData = Option<unsafe extern "system" fn(HDEV, DXEGSHDEVDATA, DWORD_PTR) -> BOOLEAN>;
pub type PFN_DxEngCreateMemoryDC = Option<unsafe extern "system" fn(HDEV) -> HDC>;
pub type PFN_DxEngGetDesktopDC = Option<unsafe extern "system" fn(ULONG, BOOL, BOOL) -> HDC>;
pub type PFN_DxEngDeleteDC = Option<unsafe extern "system" fn(HDC, BOOL) -> BOOLEAN>;
pub type PFN_DxEngCleanDC = Option<unsafe extern "system" fn(HDC) -> BOOLEAN>;
pub type PFN_DxEngSetDCOwner = Option<unsafe extern "system" fn(HGDIOBJ, DWORD) -> BOOL>;
pub type PFN_DxEngLockDC = Option<unsafe extern "system" fn(HDC) -> PDC>;
pub type PFN_DxEngUnlockDC = Option<unsafe extern "system" fn(PDC) -> BOOLEAN>;
pub type PFN_DxEngSetDCState = Option<unsafe extern "system" fn(HDC, DWORD, DWORD) -> BOOLEAN>;
pub type PFN_DxEngGetDCState = Option<unsafe extern "system" fn(HDC, DWORD) -> DWORD_PTR>;
pub type PFN_DxEngSelectBitmap = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSetBitmapOwner = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngDeleteSurface = Option<unsafe extern "system" fn(DWORD) -> DWORD>;
pub type PFN_DxEngGetSurfaceData = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngAltLockSurface = Option<unsafe extern "system" fn(DWORD) -> DWORD>;
pub type PFN_DxEngUploadPaletteEntryToSurface = Option<unsafe extern "system" fn(DWORD, DWORD, DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngMarkSurfaceAsDirectDraw = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSelectPaletteToSurface = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSyncPaletteTableWithDevice = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngSetPaletteState = Option<unsafe extern "system" fn(DWORD, DWORD, DWORD) -> DWORD>;
pub type PFN_DxEngGetRedirectionBitmap = Option<unsafe extern "system" fn(DWORD) -> DWORD>;
pub type PFN_DxEngLoadImage = Option<unsafe extern "system" fn(DWORD, DWORD) -> DWORD>;

// DXENG_FUNCTIONS (dxg_int.h:120-165) — the named thunk table. Populated in
// DxDdStartupDxGraphics from win32k's pDxEngDrv (DRVENABLEDATA → DRVFN[]).
#[repr(C)]
pub struct DXENG_FUNCTIONS {
    pub Reserved: PVOID,
    pub DxEngNUIsTermSrv: PFN_DxEngNUIsTermSrv,
    pub DxEngScreenAccessCheck: PFN_DxEngScreenAccessCheck,
    pub DxEngRedrawDesktop: PFN_DxEngRedrawDesktop,
    pub DxEngDispUniq: PFN_DxEngDispUniq,
    pub DxEngIncDispUniq: PFN_DxEngIncDispUniq,
    pub DxEngVisRgnUniq: PFN_DxEngVisRgnUniq,
    pub DxEngLockShareSem: PFN_DxEngLockShareSem,
    pub DxEngUnlockShareSem: PFN_DxEngUnlockShareSem,
    pub DxEngEnumerateHdev: PFN_DxEngEnumerateHdev,
    pub DxEngLockHdev: PFN_DxEngLockHdev,
    pub DxEngUnlockHdev: PFN_DxEngUnlockHdev,
    pub DxEngIsHdevLockedByCurrentThread: PFN_DxEngIsHdevLockedByCurrentThread,
    pub DxEngReferenceHdev: PFN_DxEngReferenceHdev,
    pub DxEngUnreferenceHdev: PFN_DxEngUnreferenceHdev,
    pub DxEngGetDeviceGammaRamp: PFN_DxEngGetDeviceGammaRamp,
    pub DxEngSetDeviceGammaRamp: PFN_DxEngSetDeviceGammaRamp,
    pub DxEngSpTearDownSprites: PFN_DxEngSpTearDownSprites,
    pub DxEngSpUnTearDownSprites: PFN_DxEngSpUnTearDownSprites,
    pub DxEngSpSpritesVisible: PFN_DxEngSpSpritesVisible,
    pub DxEngGetHdevData: PFN_DxEngGetHdevData,
    pub DxEngSetHdevData: PFN_DxEngSetHdevData,
    pub DxEngCreateMemoryDC: PFN_DxEngCreateMemoryDC,
    pub DxEngGetDesktopDC: PFN_DxEngGetDesktopDC,
    pub DxEngDeleteDC: PFN_DxEngDeleteDC,
    pub DxEngCleanDC: PFN_DxEngCleanDC,
    pub DxEngSetDCOwner: PFN_DxEngSetDCOwner,
    pub DxEngLockDC: PFN_DxEngLockDC,
    pub DxEngUnlockDC: PFN_DxEngUnlockDC,
    pub DxEngSetDCState: PFN_DxEngSetDCState,
    pub DxEngGetDCState: PFN_DxEngGetDCState,
    pub DxEngSelectBitmap: PFN_DxEngSelectBitmap,
    pub DxEngSetBitmapOwner: PFN_DxEngSetBitmapOwner,
    pub DxEngDeleteSurface: PFN_DxEngDeleteSurface,
    pub DxEngGetSurfaceData: PFN_DxEngGetSurfaceData,
    pub DxEngAltLockSurface: PFN_DxEngAltLockSurface,
    pub DxEngUploadPaletteEntryToSurface: PFN_DxEngUploadPaletteEntryToSurface,
    pub DxEngMarkSurfaceAsDirectDraw: PFN_DxEngMarkSurfaceAsDirectDraw,
    pub DxEngSelectPaletteToSurface: PFN_DxEngSelectPaletteToSurface,
    pub DxEngSyncPaletteTableWithDevice: PFN_DxEngSyncPaletteTableWithDevice,
    pub DxEngSetPaletteState: PFN_DxEngSetPaletteState,
    pub DxEngGetRedirectionBitmap: PFN_DxEngGetRedirectionBitmap,
    pub DxEngLoadImage: PFN_DxEngLoadImage,
}
pub type PDXENG_FUNCTIONS = *mut DXENG_FUNCTIONS;
