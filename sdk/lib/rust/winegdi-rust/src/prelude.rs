//! winegdi-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// wine/debug.h TRACE/WARN/FIXME/ERR: __WINE_DPRINTF checks the channel's class bit, then calls
// ros_dbg_log. MACROS, never out-of-line fns (field manual §9.65). The format is a string
// literal at every call site; the NUL is appended here. Variadic args must already be
// C-promoted (c_int / pointer / f64), exactly as the C call promotes them.
#[macro_export] macro_rules! __wine_dbg {
    ($cls:expr, $fmt:literal $(, $a:expr)* $(,)?) => {{
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        if (*__dbch).flags as c_int & (1 << $cls as c_int) != 0 {
            ros_dbg_log($cls, __dbch, concat!(file!(), "\0").as_ptr() as *const c_char,
                b"\0".as_ptr() as *const c_char, line!() as c_int,
                concat!($fmt, "\0").as_ptr() as *const c_char $(, $a)*);
        }
    }};
}
#[macro_export] macro_rules! TRACE { ($($a:tt)*) => { __wine_dbg!(__WINE_DBCL_TRACE, $($a)*) }; }
#[macro_export] macro_rules! ERR   { ($($a:tt)*) => { __wine_dbg!(__WINE_DBCL_ERR, $($a)*) }; }
#[macro_export] macro_rules! WARN  { ($($a:tt)*) => { __wine_dbg!(__WINE_DBCL_WARN, $($a)*) }; }
#[macro_export] macro_rules! FIXME { ($($a:tt)*) => { __wine_dbg!(__WINE_DBCL_FIXME, $($a)*) }; }
pub use crate::{__wine_dbg, ERR, FIXME, TRACE, WARN};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, debugstr_a, debugstr_an, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const GDILoObjType_LO_ALTDC_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_ALTDC_TYPE; // island enum
pub const GDILoObjType_LO_BITMAP_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_BITMAP_TYPE; // island enum
pub const GDILoObjType_LO_BRUSH_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_BRUSH_TYPE; // island enum
pub const GDILoObjType_LO_DC_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_DC_TYPE; // island enum
pub const GDILoObjType_LO_EXTPEN_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_EXTPEN_TYPE; // island enum
pub const GDILoObjType_LO_FONT_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_FONT_TYPE; // island enum
pub const GDILoObjType_LO_ICMLCS_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_ICMLCS_TYPE; // island enum
pub const GDILoObjType_LO_METADC16_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_METADC16_TYPE; // island enum
pub const GDILoObjType_LO_METAFILE16_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_METAFILE16_TYPE; // island enum
pub const GDILoObjType_LO_METAFILE_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_METAFILE_TYPE; // island enum
pub const GDILoObjType_LO_PALETTE_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_PALETTE_TYPE; // island enum
pub const GDILoObjType_LO_PEN_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_PEN_TYPE; // island enum
pub const GDILoObjType_LO_REGION_TYPE: GDILoObjType = GDILoObjType_GDILoObjType_LO_REGION_TYPE; // island enum
pub const INT_MAX: u32 = 2147483647; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_ERR: __wine_debug_class = 1; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_FIXME: __wine_debug_class = 0; // comctl32-rust/src/prelude.rs
pub const __WINE_DBCL_TRACE: __wine_debug_class = 3; // appwiz-rust/src/prelude.rs
pub const __WINE_DBCL_WARN: __wine_debug_class = 2; // appwiz-rust/src/prelude.rs
extern "C" {
    pub fn abs(j: i32) -> i32; // cnt-rust/src/cabi/math.rs
    pub fn atan2(y: f64, x: f64) -> f64; // atan2-rust/src/lib.rs
    pub fn cos(x: f64) -> f64; // crt-startup-rust/src/math.rs
    pub fn fabs(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn floor(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn sin(x: f64) -> f64; // crt-startup-rust/src/math.rs
    pub fn strlen(s: *const c_char) -> usize; // browser-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
}
extern "system" {
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CreateFileA( arg1: LPCSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileMappingA( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCSTR, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn GdiCreateClientObjLink(hobj: HGDIOBJ, pvObject: PVOID,) -> BOOL; // gdi32-rust/src/objects/clientobj.rs
    pub fn GdiRemoveClientObjLink(hobj: HGDIOBJ) -> PVOID; // gdi32-rust/src/objects/clientobj.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_amd64.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn HeapReAlloc(arg1: HANDLE, arg2: DWORD, arg3: PVOID, arg4: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapSize(arg1: HANDLE, arg2: DWORD, arg3: LPCVOID) -> SIZE_T; // aclui-rust/src/types_amd64.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_amd64.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn MulDiv( arg1: ::core::ffi::c_int, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn NtGdiDeleteColorSpace(hColorSpace: HANDLE) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtGdiDeleteObjectApp(hobj: HANDLE) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtGdiGetObjectBitmapHandle(hbr: HBRUSH, piUsage: *mut UINT) -> HBITMAP; // win32k-sys/src/bindgen_amd64.rs
    pub fn NtGdiSetVirtualResolution( hdc: HDC, cxVirtualDevicePixel: INT, cyVirtualDevicePixel: INT, cxVirtualDeviceMm: INT, cyVirtualDeviceMm: INT, ) -> BOOL; // win32k-sys/src/bindgen_amd64.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetLastError(dwErrCode: DWORD); // advapi32-rust/src/sec/misc.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn lstrcpynA(arg1: LPSTR, arg2: LPCSTR, arg3: ::core::ffi::c_int) -> LPSTR; // aclui-rust/src/types_amd64.rs
    pub fn lstrlenW(text: LPCWSTR) -> INT; // menuband-rust/src/enum_merged_folder.rs
}
// ==== end port resolve ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const CELTIC_CHARSET: BYTE = 0xF6; // clang, metadc.c
pub const ISO10_CHARSET: BYTE = 0xF5; // clang, metadc.c
pub const ISO3_CHARSET: BYTE = 0xF3; // clang, metadc.c
pub const ISO4_CHARSET: BYTE = 0xF4; // clang, metadc.c
pub const KOI8_CHARSET: BYTE = 0xF2; // clang, metadc.c
pub const TCVN_CHARSET: BYTE = 0xF1; // clang, metadc.c
pub const VISCII_CHARSET: BYTE = 0xF0; // clang, metadc.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// LDC: c2rust, rosglue.c
pub type LDC = _LDC;
// PLDC: c2rust, rosglue.c
pub type PLDC = *mut _LDC;
// emf: c2rust, emfdc.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct emf {
    pub emh: *mut ENHMETAHEADER,
    pub dc_attr: *mut WINEDC,
    pub handles_size: UINT,
    pub cur_handles: UINT,
    pub handles: *mut HGDIOBJ,
    pub file: HANDLE,
    pub dc_brush: HBRUSH,
    pub dc_pen: HPEN,
    pub path: BOOL,
}
// tagEMF_PaletteCopy: c2rust, enhmetafile.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tagEMF_PaletteCopy {
    pub cEntries: UINT,
    pub lpPe: LPPALETTEENTRY,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
pub static mut __wine_dbch_enhmetafile: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x65\x6e\x68\x6d\x65\x74\x61\x66\x69\x6c\x65\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_enhmetafile };
// ==== end wine debug channels ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::winegdi::rosglue::DeleteColorSpace;
pub use crate::winegdi::emfdrv::EMFDRV_ArcChordPie;
pub use crate::winegdi::emfdrv::EMFDRV_Ellipse;
pub use crate::winegdi::emfdrv::EMFDRV_LineTo;
pub use crate::winegdi::emfdrv::EMFDRV_Rectangle;
pub use crate::winegdi::emfdrv::EMFDRV_RoundRect;
pub use crate::winegdi::emfdrv::EMFDRV_SetPixel;
pub use crate::winegdi::enhmetafile::EMF_Create_HENHMETAFILE;
pub use crate::winegdi::rosglue::GDI_hdc_not_using_object;
pub use crate::winegdi::rosglue::GDI_hdc_using_object;
pub use crate::winegdi::enhmetafile::GetEnhMetaFileBits;
pub use crate::winegdi::enhmetafile::GetEnhMetaFileHeader;
pub use crate::winegdi::metafile::MF_Create_HMETAFILE;
pub use crate::winegdi::enhmetafile::PlayEnhMetaFile;
pub use crate::winegdi::rosglue::SetVirtualResolution;
pub use crate::winegdi::rosglue::alloc_dc_ptr;
pub use crate::winegdi::rosglue::get_brush_bitmap_info;
pub use crate::winegdi::rosglue::get_dc_ptr;
// ==== end cross-TU definitions ====

// ==== hand: gdi_private.h / gdi32p.h / ntgdihdl.h surface the island does not carry ====
// gdi_private.h: `#define GDI_HANDLE_GET_TYPE(h) (((ULONG_PTR)(h)) & GDI_HANDLE_TYPE_MASK)`.
// The C result is ULONG_PTR and every use compares it against a GDILOOBJTYPE enumerator (C
// promotes both sides). The mask is 0x007f0000, so narrowing to GDILOOBJTYPE is lossless and
// lets each comparison/match compile on all three widths without per-site casts.
#[macro_export] macro_rules! GDI_HANDLE_GET_TYPE {
    ($h:expr) => { ((($h) as ULONG_PTR) & (GDI_HANDLE_TYPE_MASK as ULONG_PTR)) as GDILOOBJTYPE };
}
pub use crate::GDI_HANDLE_GET_TYPE;

// gdi_private.h `static inline BOOL is_meta_dc( HDC hdc )`.
#[inline(always)]
pub unsafe fn is_meta_dc(hdc: HDC) -> BOOL {
    (GDI_HANDLE_GET_TYPE!(hdc) == GDILoObjType_LO_METADC16_TYPE) as BOOL
}

// gdi_private.h `static inline int get_dib_info_size( const BITMAPINFO *info, UINT coloruse )`.
#[inline(always)]
pub unsafe fn get_dib_info_size(info: *const BITMAPINFO, coloruse: UINT) -> c_int {
    if (*info).bmiHeader.biCompression == BI_BITFIELDS as DWORD {
        return (core::mem::size_of::<BITMAPINFOHEADER>() + 3 * core::mem::size_of::<DWORD>()) as c_int;
    }
    if coloruse == DIB_PAL_COLORS as UINT {
        return (core::mem::size_of::<BITMAPINFOHEADER>()
            + ((*info).bmiHeader.biClrUsed as usize).wrapping_mul(core::mem::size_of::<WORD>())) as c_int;
    }
    // FIELD_OFFSET( BITMAPINFO, bmiColors[info->bmiHeader.biClrUsed] )
    (core::mem::offset_of!(BITMAPINFO, bmiColors)
        + ((*info).bmiHeader.biClrUsed as usize).wrapping_mul(core::mem::size_of::<RGBQUAD>())) as c_int
}

// win32ss/include/ntgdityp.h
pub type PUMDHPDEV = PVOID;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _UNIVERSAL_FONT_ID {
    pub CheckSum: ULONG,
    pub Index: ULONG,
}
pub type UNIVERSAL_FONT_ID = _UNIVERSAL_FONT_ID;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _DEVCAPS {
    pub ulVersion: ULONG, pub ulTechnology: ULONG, pub ulHorzSizeM: ULONG, pub ulVertSizeM: ULONG,
    pub ulHorzSize: ULONG, pub ulVertSize: ULONG, pub ulHorzRes: ULONG, pub ulVertRes: ULONG,
    pub ulBitsPixel: ULONG, pub ulPlanes: ULONG, pub ulNumPens: ULONG, pub ulNumFonts: ULONG,
    pub ulNumColors: ULONG, pub ulRasterCaps: ULONG, pub ulAspectX: ULONG, pub ulAspectY: ULONG,
    pub ulAspectXY: ULONG, pub ulLogPixelsX: ULONG, pub ulLogPixelsY: ULONG,
    pub ulSizePalette: ULONG, pub ulColorRes: ULONG, pub ulPhysicalWidth: ULONG,
    pub ulPhysicalHeight: ULONG, pub ulPhysicalOffsetX: ULONG, pub ulPhysicalOffsetY: ULONG,
    pub ulTextCaps: ULONG, pub ulVRefresh: ULONG, pub ulDesktopHorzRes: ULONG,
    pub ulDesktopVertRes: ULONG, pub ulBltAlignment: ULONG, pub ulPanningHorzRes: ULONG,
    pub ulPanningVertRes: ULONG, pub xPanningAlignment: ULONG, pub yPanningAlignment: ULONG,
    pub ulShadeBlend: ULONG, pub ulColorMgmtCaps: ULONG,
}
pub type DEVCAPS = _DEVCAPS;

// win32ss/include/ntgdihdl.h `typedef struct _LDC {...} LDC, *PLDC;` (what rosglue.c sees via
// precomp.h). Layout transcribed field-for-field; win32k-sys's bindgen of the same header agrees.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _LDC {
    pub hDC: HDC,
    pub Flags: ULONG,
    pub iType: INT,
    pub pvEmfDC: PVOID,
    pub pwszPort: LPWSTR,
    pub pAbortProc: ABORTPROC,
    pub CallBackTick: DWORD,
    pub hPrinter: HANDLE,
    pub pdm: PDEVMODEW,
    pub pUMPDev: PVOID,
    pub pUMdhpdev: PUMDHPDEV,
    pub UFIHashTable: [PVOID; 3],
    pub ufi: UNIVERSAL_FONT_ID,
    pub pvEMFSpoolData: PVOID,
    pub cjSize: ULONG,
    pub leRecords: LIST_ENTRY,
    pub DevCaps: DEVCAPS,
    pub BrushColor: HBRUSH,
    pub PenColor: HPEN,
    pub dwData: [DWORD; 5],
}

// gdi32p.h: `PLDC FASTCALL GdiGetLDC(HDC)` / `BOOL FASTCALL GdiSetLDC(HDC, PVOID)`, defined in
// gdi32's misc. FASTCALL is __fastcall on i386 only; elsewhere it is the platform C ABI.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn GdiGetLDC(hDC: HDC) -> PLDC;
    pub fn GdiSetLDC(hdc: HDC, pvLDC: PVOID) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn GdiGetLDC(hDC: HDC) -> PLDC;
    pub fn GdiSetLDC(hdc: HDC, pvLDC: PVOID) -> BOOL;
}
// wine/debug.h `static __inline const char *wine_dbgstr_rect( const RECT *rect )`.
pub unsafe fn wine_dbgstr_rect(rect: *const RECT) -> *const c_char {
    if rect.is_null() {
        return c"(null)".as_ptr();
    }
    wine_dbg_sprintf(c"(%ld,%ld)-(%ld,%ld)".as_ptr(), (*rect).left, (*rect).top, (*rect).right, (*rect).bottom)
}

// metadc.c / metafile.c: WINE_DEFAULT_DEBUG_CHANNEL(metafile). Those modules shadow the
// prelude's `__wine_dbch___default` (enhmetafile) with a module-local one pointing here.
pub static mut __wine_dbch_metafile: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"metafile\0\0\0\0\0\0\0") },
};
// gdi_private.h: `#define DbgRaiseAssertionFailure() __int2c()` (x86/x64) / `__break(0xf001)`
// (ARM64), then `#undef ASSERT` / `#define ASSERT(x) if (!(x)) DbgRaiseAssertionFailure()` --
// an ALWAYS-ON assert, unlike the DBG-only ntdef one, so win32macros' ASSERT is not used here.
#[macro_export] macro_rules! DbgRaiseAssertionFailure {
    () => {{
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        core::arch::asm!("int 0x2c");
        #[cfg(target_arch = "aarch64")]
        core::arch::asm!("brk #0xf001");
    }};
}
#[macro_export] macro_rules! ASSERT {
    ($e:expr $(,)?) => { if !($e) { DbgRaiseAssertionFailure!(); } };
}
pub use crate::{ASSERT, DbgRaiseAssertionFailure};
// enhmetafile.c's critsect_debug.
// C initialises the debug-info union with `(DWORD_PTR)(__FILE__ ": enhmetafile_cs")`, a
// pointer stored as an integer. Rust const-eval cannot turn a pointer into an integer, so the
// static is laid out as a field-for-field twin of RTL_CRITICAL_SECTION_DEBUG whose union member
// is pointer-typed (same bytes, same size: asserted below) and handed out cast to the real type.
#[repr(C)]
pub union critsect_debug_string {
    pub WineDebugString: *const ::core::ffi::c_char,
    pub _layout: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1,
}
#[repr(C)]
pub struct critsect_debug_init {
    pub Type: WORD,
    pub CreatorBackTraceIndex: WORD,
    pub CriticalSection: *mut _RTL_CRITICAL_SECTION,
    pub ProcessLocksList: LIST_ENTRY,
    pub EntryCount: DWORD,
    pub ContentionCount: DWORD,
    pub u: critsect_debug_string,
}
const _: () = assert!(
    core::mem::size_of::<critsect_debug_init>() == core::mem::size_of::<CRITICAL_SECTION_DEBUG>()
        && core::mem::offset_of!(critsect_debug_init, u)
            == core::mem::offset_of!(_RTL_CRITICAL_SECTION_DEBUG, __bindgen_anon_1)
);

// ==== end hand ====

// ==== hand: gdi_private.h's "special sauce for reactos" ====
// Object-like macro ALIASES: the C never calls these names, the preprocessor renames them.
// The island still declares the prototypes the macros override, so without these the
// translation linked against names no image exports (i386 LINK_FAIL in gdi32.dll).
pub use crate::types::RealizePalette as GDIRealizePalette;
pub use crate::types::SelectPalette as GDISelectPalette;
pub use crate::types::GdiFixUpHandle as get_full_gdi_handle;
