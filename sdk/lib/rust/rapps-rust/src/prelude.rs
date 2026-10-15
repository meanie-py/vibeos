//! rapps-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use crate::gdiplus_flat::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= ATL ========================================================================
// ATL's CStringW/CComPtr/containers, ported once in atlclasses-rust.
pub use atl_rust::{atl_format, atl_format_id, CAtlList, CComPtr, CHeapPtr, CSimpleArray, CLocalAllocator, CSimpleMap, CStringA, CStringW, POSITION, _AtlBaseModule};
pub use atl_rust::CStringW as CSimpleString;
pub use atl_rust::CPath;
pub use atl_rust::CPath as CPathW;
pub use atl_rust::com::ShellObjectCreator;

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __debugbreak, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port cppapp: the module's C++ classes (generated -- `rustconv port cppapp`) ====
pub use crate::rapps::cpp_classes::*;
pub use crate::rapps::cpp_classes::{ARGB, Bitmap, Bitmap__VTBL_b0, Bitmap__delete, Bitmap__make_0, Bitmap__make_1, Bitmap__make_10, Bitmap__make_2, Bitmap__make_3, Bitmap__make_4, Bitmap__make_5, Bitmap__make_6, Bitmap__make_7, Bitmap__make_8, Bitmap__make_9, Bitmap__new_0, Bitmap__new_1, Bitmap__new_10, Bitmap__new_2, Bitmap__new_3, Bitmap__new_4, Bitmap__new_5, Bitmap__new_6, Bitmap__new_7, Bitmap__new_8, Bitmap__new_9, Bitmap__new_in_0, Bitmap__new_in_1, Bitmap__new_in_10, Bitmap__new_in_2, Bitmap__new_in_3, Bitmap__new_in_4, Bitmap__new_in_5, Bitmap__new_in_6, Bitmap__new_in_7, Bitmap__new_in_8, Bitmap__new_in_9, Bitmap__zeroed, Brush, BrushType, Brush__VTBL_own, Brush__delete, Brush__make_0, Brush__make_1, Brush__new_0, Brush__new_1, Brush__new_in_0, Brush__new_in_1, Brush__vtbl, Brush__zeroed, CAppDB, CAppDB__delete, CAppDB__make_0, CAppDB__new_0, CAppDB__new_in_0, CAppDB__zeroed, CAppInfo, CAppInfoDisplay, CAppInfoDisplay__VTBL_b0b0, CAppInfoDisplay__VTBL_b0b1, CAppInfoDisplay__delete, CAppInfoDisplay__make_0, CAppInfoDisplay__new_0, CAppInfoDisplay__new_in_0, CAppInfoDisplay__zeroed, CAppInfo__VTBL_own, CAppInfo__delete, CAppInfo__make_0, CAppInfo__new_0, CAppInfo__new_in_0, CAppInfo__vtbl, CAppInfo__zeroed, CAppRichEdit, CAppRichEdit__VTBL_b0b0, CAppRichEdit__VTBL_b0b1, CAppRichEdit__VTBL_b0b2, CAppRichEdit__delete, CAppRichEdit__make_0, CAppRichEdit__new_0, CAppRichEdit__new_in_0, CAppRichEdit__zeroed, CAppScrnshotPreview, CAppScrnshotPreview__VTBL_own, CAppScrnshotPreview__delete, CAppScrnshotPreview__make_0, CAppScrnshotPreview__new_0, CAppScrnshotPreview__new_in_0, CAppScrnshotPreview__vtbl, CAppScrnshotPreview__zeroed, CApplicationView, CApplicationView__VTBL_b0b0, CApplicationView__VTBL_b0b1, CApplicationView__delete, CApplicationView__make_0, CApplicationView__new_0, CApplicationView__new_in_0, CApplicationView__zeroed, CAppsListView, CAppsListView__CLASS_NAME, CAppsListView__VTBL_b0b0, CAppsListView__VTBL_b0b1, CAppsListView__WND_CLASS, CAppsListView__delete, CAppsListView__make_0, CAppsListView__new_0, CAppsListView__new_in_0, CAppsListView__zeroed, CAsyncLoadIcon, CAsyncLoadIcon__delete, CAsyncLoadIcon__make_0, CAsyncLoadIcon__new_0, CAsyncLoadIcon__new_in_0, CAsyncLoadIcon__zeroed, CAtlWinModule, CAvailableApplicationInfo, CAvailableApplicationInfo__VTBL_b0, CAvailableApplicationInfo__delete, CAvailableApplicationInfo__make_0, CAvailableApplicationInfo__new_0, CAvailableApplicationInfo__new_in_0, CAvailableApplicationInfo__zeroed, CComModule, CComboBox, CComboBox__delete, CComboBox__make_0, CComboBox__new_0, CComboBox__new_in_0, CComboBox__zeroed, CConfigParser, CConfigParser__delete, CConfigParser__make_0, CConfigParser__new_0, CConfigParser__new_in_0, CConfigParser__zeroed, CDowloadingAppsListView, CDowloadingAppsListView__delete, CDowloadingAppsListView__make_0, CDowloadingAppsListView__new_0, CDowloadingAppsListView__new_in_0, CDowloadingAppsListView__zeroed, CDownloadManager, CDownloadManager__IUnknown_VTBL, CDownloadManager__delete, CDownloadManager__make_0, CDownloadManager__new_0, CDownloadManager__new_in_0, CDownloadManager__zeroed, CDownloaderProgress, CDownloaderProgress__CLASS_NAME, CDownloaderProgress__VTBL_own, CDownloaderProgress__WND_CLASS, CDownloaderProgress__delete, CDownloaderProgress__make_0, CDownloaderProgress__new_0, CDownloaderProgress__new_in_0, CDownloaderProgress__vtbl, CDownloaderProgress__zeroed, CEnumInstalledRootKey, CEnumInstalledRootKey__delete, CEnumInstalledRootKey__make_0, CEnumInstalledRootKey__new_0, CEnumInstalledRootKey__new_in_0, CEnumInstalledRootKey__zeroed, CInstalledApplicationInfo, CInstalledApplicationInfo__VTBL_b0, CInstalledApplicationInfo__delete, CInstalledApplicationInfo__make_0, CInstalledApplicationInfo__new_0, CInstalledApplicationInfo__new_in_0, CInstalledApplicationInfo__zeroed, CListView, CListView__delete, CListView__make_0, CListView__new_0, CListView__new_in_0, CListView__zeroed, CLocalPtr_char, CLocalPtr_char__delete, CLocalPtr_char__make_0, CLocalPtr_char__new_0, CLocalPtr_char__new_in_0, CLocalPtr_char__zeroed, CLocalPtr_wchar_t, CLocalPtr_wchar_t__delete, CLocalPtr_wchar_t__make_0, CLocalPtr_wchar_t__new_0, CLocalPtr_wchar_t__new_in_0, CLocalPtr_wchar_t__zeroed, CLocaleSections, CLocaleSections__delete, CLocaleSections__make_0, CLocaleSections__new_0, CLocaleSections__new_in_0, CLocaleSections__zeroed, CMainToolbar, CMainToolbar__VTBL_b0b0, CMainToolbar__VTBL_b0b1, CMainToolbar__delete, CMainToolbar__make_0, CMainToolbar__new_0, CMainToolbar__new_in_0, CMainToolbar__zeroed, CMainWindow, CMainWindow__VTBL_own, CMainWindow__delete, CMainWindow__m_PendingInstalledViewRefresh, CMainWindow__m_hLastFocus, CMainWindow__make_0, CMainWindow__new_0, CMainWindow__new_in_0, CMainWindow__vtbl, CMainWindow__zeroed, CPoint, CPoint__delete, CPoint__make_0, CPoint__make_1, CPoint__make_2, CPoint__make_3, CPoint__make_4, CPoint__new_0, CPoint__new_1, CPoint__new_2, CPoint__new_3, CPoint__new_4, CPoint__new_in_0, CPoint__new_in_1, CPoint__new_in_2, CPoint__new_in_3, CPoint__new_in_4, CPoint__zeroed, CPointerArray_CUiPrimitive, CPointerArray_CUiPrimitive__VTBL_own, CPointerArray_CUiPrimitive__delete, CPointerArray_CUiPrimitive__make_0, CPointerArray_CUiPrimitive__new_0, CPointerArray_CUiPrimitive__new_in_0, CPointerArray_CUiPrimitive__vtbl, CPointerArray_CUiPrimitive__zeroed, CRect, CRect__delete, CRect__make_0, CRect__make_1, CRect__make_2, CRect__make_3, CRect__make_4, CRect__make_5, CRect__new_0, CRect__new_1, CRect__new_2, CRect__new_3, CRect__new_4, CRect__new_5, CRect__new_in_0, CRect__new_in_1, CRect__new_in_2, CRect__new_in_3, CRect__new_in_4, CRect__new_in_5, CRect__zeroed, CRegKey, CRegKey__delete, CRegKey__make_0, CRegKey__make_1, CRegKey__make_2, CRegKey__new_0, CRegKey__new_1, CRegKey__new_2, CRegKey__new_in_0, CRegKey__new_in_1, CRegKey__new_in_2, CRegKey__zeroed, CRichEdit, CRichEdit__VTBL_own, CRichEdit__delete, CRichEdit__make_0, CRichEdit__new_0, CRichEdit__new_in_0, CRichEdit__vtbl, CRichEdit__zeroed, CScopedMutex, CScopedMutex__delete, CScopedMutex__make_0, CScopedMutex__new_0, CScopedMutex__new_in_0, CScopedMutex__zeroed, CSearchBar, CSearchBar__delete, CSearchBar__make_0, CSearchBar__new_0, CSearchBar__new_in_0, CSearchBar__zeroed, CSectionNames, CSectionNames__delete, CSectionNames__make_0, CSectionNames__new_0, CSectionNames__new_in_0, CSectionNames__zeroed, CSideTreeView, CSideTreeView__VTBL_b0b0, CSideTreeView__VTBL_b0b1, CSideTreeView__delete, CSideTreeView__make_0, CSideTreeView__new_0, CSideTreeView__new_in_0, CSideTreeView__zeroed, CSize, CSize__delete, CSize__make_0, CSize__make_1, CSize__make_2, CSize__make_3, CSize__make_4, CSize__new_0, CSize__new_1, CSize__new_2, CSize__new_3, CSize__new_4, CSize__new_in_0, CSize__new_in_1, CSize__new_in_2, CSize__new_in_3, CSize__new_in_4, CSize__zeroed, CStatusBar, CStatusBar__delete, CStatusBar__make_0, CStatusBar__new_0, CStatusBar__new_in_0, CStatusBar__zeroed, CToolbar, CToolbar__delete, CToolbar__make_0, CToolbar__new_0, CToolbar__new_in_0, CToolbar__zeroed, CTooltips, CTooltips__delete, CTooltips__make_0, CTooltips__new_0, CTooltips__new_in_0, CTooltips__zeroed, CTreeView, CTreeView__delete, CTreeView__make_0, CTreeView__new_0, CTreeView__new_in_0, CTreeView__zeroed, CUiBox, CUiBox__VTBL_own, CUiBox__delete, CUiBox__make_0, CUiBox__new_0, CUiBox__new_in_0, CUiBox__vtbl, CUiBox__zeroed, CUiCollection, CUiCollection__VTBL_b0, CUiCollection__delete, CUiCollection__make_0, CUiCollection__new_0, CUiCollection__new_in_0, CUiCollection__zeroed, CUiContainer, CUiContainer__delete, CUiContainer__make_0, CUiContainer__new_0, CUiContainer__new_in_0, CUiContainer__zeroed, CUiMargin, CUiMargin__delete, CUiMargin__make_0, CUiMargin__make_1, CUiMargin__make_2, CUiMargin__new_0, CUiMargin__new_1, CUiMargin__new_2, CUiMargin__new_in_0, CUiMargin__new_in_1, CUiMargin__new_in_2, CUiMargin__zeroed, CUiMeasure, CUiMeasure__delete, CUiMeasure__make_0, CUiMeasure__make_1, CUiMeasure__new_0, CUiMeasure__new_1, CUiMeasure__new_in_0, CUiMeasure__new_in_1, CUiMeasure__zeroed, CUiPanel, CUiPanel__VTBL_b0, CUiPanel__VTBL_b1, CUiPanel__delete, CUiPanel__make_0, CUiPanel__new_0, CUiPanel__new_in_0, CUiPanel__zeroed, CUiPrimitive, CUiPrimitive__VTBL_own, CUiPrimitive__delete, CUiPrimitive__make_0, CUiPrimitive__new_0, CUiPrimitive__new_in_0, CUiPrimitive__vtbl, CUiPrimitive__zeroed, CUiRect, CUiRect__delete, CUiRect__make_0, CUiRect__make_1, CUiRect__new_0, CUiRect__new_1, CUiRect__new_in_0, CUiRect__new_in_1, CUiRect__zeroed, CUiSplitPanel, CUiSplitPanel__CLASS_NAME, CUiSplitPanel__THICKNESS, CUiSplitPanel__VTBL_b0, CUiSplitPanel__VTBL_b1, CUiSplitPanel__WND_CLASS, CUiSplitPanel__delete, CUiSplitPanel__make_0, CUiSplitPanel__new_0, CUiSplitPanel__new_in_0, CUiSplitPanel__zeroed, CUiWindow_CComboBox, CUiWindow_CComboBox__VTBL_b0, CUiWindow_CComboBox__VTBL_b1, CUiWindow_CComboBox__delete, CUiWindow_CComboBox__make_0, CUiWindow_CComboBox__new_0, CUiWindow_CComboBox__new_in_0, CUiWindow_CComboBox__vtbl, CUiWindow_CComboBox__zeroed, CUiWindow_CRichEdit, CUiWindow_CRichEdit__VTBL_b0, CUiWindow_CRichEdit__VTBL_b1, CUiWindow_CRichEdit__VTBL_b2, CUiWindow_CRichEdit__delete, CUiWindow_CRichEdit__make_0, CUiWindow_CRichEdit__new_0, CUiWindow_CRichEdit__new_in_0, CUiWindow_CRichEdit__vtbl, CUiWindow_CRichEdit__zeroed, CUiWindow_CSearchBar, CUiWindow_CSearchBar__VTBL_b0, CUiWindow_CSearchBar__VTBL_b1, CUiWindow_CSearchBar__delete, CUiWindow_CSearchBar__make_0, CUiWindow_CSearchBar__new_0, CUiWindow_CSearchBar__new_in_0, CUiWindow_CSearchBar__vtbl, CUiWindow_CSearchBar__zeroed, CUiWindow_CStatusBar, CUiWindow_CStatusBar__VTBL_b0, CUiWindow_CStatusBar__VTBL_b1, CUiWindow_CStatusBar__delete, CUiWindow_CStatusBar__make_0, CUiWindow_CStatusBar__new_0, CUiWindow_CStatusBar__new_in_0, CUiWindow_CStatusBar__vtbl, CUiWindow_CStatusBar__zeroed, CUiWindow_CToolbar, CUiWindow_CToolbar__VTBL_b0, CUiWindow_CToolbar__VTBL_b1, CUiWindow_CToolbar__delete, CUiWindow_CToolbar__make_0, CUiWindow_CToolbar__new_0, CUiWindow_CToolbar__new_in_0, CUiWindow_CToolbar__vtbl, CUiWindow_CToolbar__zeroed, CUiWindow_CTreeView, CUiWindow_CTreeView__VTBL_b0, CUiWindow_CTreeView__VTBL_b1, CUiWindow_CTreeView__delete, CUiWindow_CTreeView__make_0, CUiWindow_CTreeView__new_0, CUiWindow_CTreeView__new_in_0, CUiWindow_CTreeView__vtbl, CUiWindow_CTreeView__zeroed, CUiWindow_CWindowImpl_CAppInfoDisplay, CUiWindow_CWindowImpl_CAppInfoDisplay__VTBL_b0, CUiWindow_CWindowImpl_CAppInfoDisplay__VTBL_b1, CUiWindow_CWindowImpl_CAppInfoDisplay__delete, CUiWindow_CWindowImpl_CAppInfoDisplay__make_0, CUiWindow_CWindowImpl_CAppInfoDisplay__new_0, CUiWindow_CWindowImpl_CAppInfoDisplay__new_in_0, CUiWindow_CWindowImpl_CAppInfoDisplay__vtbl, CUiWindow_CWindowImpl_CAppInfoDisplay__zeroed, CUiWindow_CWindowImpl_CApplicationView, CUiWindow_CWindowImpl_CApplicationView__VTBL_b0, CUiWindow_CWindowImpl_CApplicationView__VTBL_b1, CUiWindow_CWindowImpl_CApplicationView__delete, CUiWindow_CWindowImpl_CApplicationView__make_0, CUiWindow_CWindowImpl_CApplicationView__new_0, CUiWindow_CWindowImpl_CApplicationView__new_in_0, CUiWindow_CWindowImpl_CApplicationView__vtbl, CUiWindow_CWindowImpl_CApplicationView__zeroed, CUiWindow_CWindowImpl_CAppsListView_CListView, CUiWindow_CWindowImpl_CAppsListView_CListView__VTBL_b0, CUiWindow_CWindowImpl_CAppsListView_CListView__VTBL_b1, CUiWindow_CWindowImpl_CAppsListView_CListView__delete, CUiWindow_CWindowImpl_CAppsListView_CListView__make_0, CUiWindow_CWindowImpl_CAppsListView_CListView__new_0, CUiWindow_CWindowImpl_CAppsListView_CListView__new_in_0, CUiWindow_CWindowImpl_CAppsListView_CListView__vtbl, CUiWindow_CWindowImpl_CAppsListView_CListView__zeroed, CUpdateDatabaseMutex, CUpdateDatabaseMutex__delete, CUpdateDatabaseMutex__make_0, CUpdateDatabaseMutex__new_0, CUpdateDatabaseMutex__new_in_0, CUpdateDatabaseMutex__zeroed, CWindow, CWindow__delete, CWindow__make_0, CWindow__new_0, CWindow__new_in_0, CWindow__rcDefault, CWindow__zeroed, CWndClassInfo, Color, ColorPalette, Color__delete, Color__make_0, Color__make_1, Color__make_2, Color__make_3, Color__new_0, Color__new_1, Color__new_2, Color__new_3, Color__new_in_0, Color__new_in_1, Color__new_in_2, Color__new_in_3, Color__zeroed, CombineMode, CombineModeExclude, CombineModeIntersect, CommonInfo, CommonInfo__delete, CommonInfo__make_0, CommonInfo__new_0, CommonInfo__new_in_0, CommonInfo__zeroed, CompositingMode, CompositingQuality, CoordinateSpace, DLSTATUS_DOWNLOADING, DLSTATUS_FINISHED, DLSTATUS_INSTALLED, DLSTATUS_INSTALLING, DLSTATUS_WAITING, DLSTATUS_WAITING_INSTALL, DLTYPE_APPLICATION, DLTYPE_DBUPDATE, DLTYPE_DBUPDATE_UNOFFICIAL, DebugEventProc, Deleter_CAvailableApplicationInfo, Deleter_CAvailableApplicationInfo__delete, Deleter_CAvailableApplicationInfo__make_0, Deleter_CAvailableApplicationInfo__new_0, Deleter_CAvailableApplicationInfo__new_in_0, Deleter_CAvailableApplicationInfo__zeroed, DownloadInfo, DownloadInfo__delete, DownloadInfo__make_0, DownloadInfo__make_1, DownloadInfo__new_0, DownloadInfo__new_1, DownloadInfo__new_in_0, DownloadInfo__new_in_1, DownloadInfo__zeroed, DownloadStatus, DownloadType, DrawImageAbort, EnumerateMetafileProc, FillMode, FlushIntention, GdiplusBase, GdiplusBase__delete, GdiplusBase__make_0, GdiplusBase__new_0, GdiplusBase__new_in_0, GdiplusBase__zeroed, GdiplusStartupInput, GdiplusStartupInput__delete, GdiplusStartupInput__make_0, GdiplusStartupInput__new_0, GdiplusStartupInput__new_in_0, GdiplusStartupInput__zeroed, GetThumbnailImageAbort, GpBitmap, GpBrush, GpGraphics, GpImage, GpSolidFill, Graphics, GraphicsContainer, GraphicsState, Graphics__delete, Graphics__make_0, Graphics__make_1, Graphics__make_2, Graphics__make_3, Graphics__new_0, Graphics__new_1, Graphics__new_2, Graphics__new_3, Graphics__new_in_0, Graphics__new_in_1, Graphics__new_in_2, Graphics__new_in_3, Graphics__zeroed, Image, ImageAttributes, ImageType, Image__VTBL_own, Image__delete, Image__make_0, Image__make_1, Image__make_2, Image__make_3, Image__new_0, Image__new_1, Image__new_2, Image__new_3, Image__new_in_0, Image__new_in_1, Image__new_in_2, Image__new_in_3, Image__vtbl, Image__zeroed, InstallInfo, InstallInfo__delete, InstallInfo__make_0, InstallInfo__new_0, InstallInfo__new_in_0, InstallInfo__zeroed, InterpolationMode, InvalidParameter, Matrix, MatrixOrder, MeasureType, MeasureType_Type_FitContent, MeasureType_Type_FitParent, MeasureType_Type_Fixed, MeasureType_Type_Percent, NotImplemented, NotifyData, NotifyData__delete, NotifyData__make_0, NotifyData__new_0, NotifyData__new_in_0, NotifyData__zeroed, PixelFormat, PixelOffsetMode, Point, PointF, REAL, RESTORELISTSELECTION, RESTORELISTSELECTION__delete, RESTORELISTSELECTION__make_0, RESTORELISTSELECTION__new_0, RESTORELISTSELECTION__new_in_0, RESTORELISTSELECTION__zeroed, Rect, RectF, SETTINGS_INFO__delete, SETTINGS_INFO__make_0, SETTINGS_INFO__new_0, SETTINGS_INFO__new_in_0, SETTINGS_INFO__zeroed, SettingsField, SettingsFieldBool, SettingsFieldBool__VTBL_b0, SettingsFieldBool__delete, SettingsFieldBool__make_0, SettingsFieldBool__new_0, SettingsFieldBool__new_in_0, SettingsFieldBool__zeroed, SettingsFieldInt, SettingsFieldInt__VTBL_b0, SettingsFieldInt__delete, SettingsFieldInt__make_0, SettingsFieldInt__new_0, SettingsFieldInt__new_in_0, SettingsFieldInt__zeroed, SettingsFieldString, SettingsFieldString__VTBL_b0, SettingsFieldString__delete, SettingsFieldString__make_0, SettingsFieldString__new_0, SettingsFieldString__new_in_0, SettingsFieldString__zeroed, SettingsField__VTBL_own, SettingsField__delete, SettingsField__make_0, SettingsField__new_0, SettingsField__new_in_0, SettingsField__vtbl, SettingsField__zeroed, SettingsInfo, Size, SizeF, SmoothingMode, SmoothingModeAntiAlias, SolidBrush, SolidBrush__VTBL_b0, SolidBrush__delete, SolidBrush__make_0, SolidBrush__make_1, SolidBrush__new_0, SolidBrush__new_1, SolidBrush__new_in_0, SolidBrush__new_in_1, SolidBrush__zeroed, SortContext, SortContext__delete, SortContext__make_0, SortContext__new_0, SortContext__new_in_0, SortContext__zeroed, Status, THICKNESS, TextRenderingHint, UINSTALLSTAGECOUNT, US_CONTAINERS, US_ITEMS, UninstallInfo, UninstallInfo__delete, UninstallInfo__make_0, UninstallInfo__new_0, UninstallInfo__new_in_0, UninstallInfo__zeroed, UninstallStage, Unit, UsageString, _ATL_OBJMAP_ENTRY, _IDHWNDPAIR, _IDHWNDPAIR__delete, _IDHWNDPAIR__make_0, _IDHWNDPAIR__new_0, _IDHWNDPAIR__new_in_0, _IDHWNDPAIR__zeroed, _U_MENUorID, _U_RECT, __AsyncInet__delete, __AsyncInet__make_0, __AsyncInet__new_0, __AsyncInet__new_in_0, __AsyncInet__zeroed, __ScrnshotDownloadParam, __ScrnshotDownloadParam__delete, __ScrnshotDownloadParam__make_0, __ScrnshotDownloadParam__new_0, __ScrnshotDownloadParam__new_in_0, __ScrnshotDownloadParam__zeroed, _bindgen_ty_1, cpp_alloc, cpp_delete, cpp_delete_array, cpp_free, cpp_new, cpp_new_array, cpp_new_in, cpp_pure_virtual_call, g_Names, m_AppDisplayTypeDetails, m_AppDisplayTypeList, m_AppDisplayTypeTile};
// ==== end port cppapp ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! CreateDialogIndirectW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { CreateDialogIndirectParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! CreateWindowW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr, $p4:expr, $p5:expr, $p6:expr, $p7:expr, $p8:expr, $p9:expr, $p10:expr $(,)?) => { CreateWindowExW(((0) as _), (($p0) as _), (($p1) as _), (($p2) as _), (($p3) as _), (($p4) as _), (($p5) as _), (($p6) as _), (($p7) as _), (($p8) as _), (($p9) as _), (($p10) as _)) }; }
#[macro_export] macro_rules! DPA_GetPtrCount { ($p0:expr $(,)?) => { (*(($p0) as *mut c_int)) }; }
#[macro_export] macro_rules! DialogBoxW { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { DialogBoxParamW((($p0) as _), (($p1) as _), (($p2) as _), $p3, ((0) as _)) }; }
#[macro_export] macro_rules! GetStockFont { ($p0:expr $(,)?) => { ((GetStockObject((($p0) as _))) as HFONT) }; }
#[macro_export] macro_rules! Header_GetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 11)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! Header_SetItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1200 + 12)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! INDEXTOSTATEIMAGEMASK { ($p0:expr $(,)?) => { ($p0 << 12) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! ImageList_RemoveAll { ($p0:expr $(,)?) => { ImageList_Remove((($p0) as _), (((-(1 as c_int))) as _)) }; }
#[macro_export] macro_rules! ListView_FindItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 83)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetCheckState { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), (((0x1000 + 44)) as _), (($p1) as WPARAM), ((0xF000) as _))) as UINT) >> 12) - 1) }; }
#[macro_export] macro_rules! ListView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 75)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetItemCount { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 4)) as _), ((0) as _), ((0) as _))) as c_int) }; }
#[macro_export] macro_rules! ListView_GetItemRect { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 14)) as _), (((($p1) as c_int)) as WPARAM), (((if ($p2) as isize != 0 { { { (*(($p2) as *mut RECT)).left = (($p3) as _); }; (($p2) as LPARAM) } } else { ((0) as LPARAM) })) as _))) as BOOL) }; }
#[macro_export] macro_rules! ListView_GetNextItem { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 12)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_HitTest { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 18)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertColumn { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 97)) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 77)) as _), ((0) as _), (($p1) as LPARAM))) as c_int) }; }
#[macro_export] macro_rules! ListView_Scroll { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 20)) as _), (((($p1) as c_int)) as WPARAM), (((($p2) as c_int)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetCheckState { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = ((0xF000) as _); }; { _ms_lvi.state = ((((if ($p2) as isize != 0 { 2 } else { 1 }) << 12)) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! ListView_SetColumnWidth { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 30)) as _), (((($p1) as c_int)) as WPARAM), (((((((((((((($p2) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | ((((((((0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetExtendedListViewStyleEx { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 54)) as _), (($p1) as _), (($p2) as _))) as DWORD) }; }
#[macro_export] macro_rules! ListView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1000 + 76)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! ListView_SetItemState { ($p0:expr, $p1:expr, $p2:expr, $p3:expr $(,)?) => { { let mut _ms_lvi: LVITEMW = core::mem::zeroed(); { _ms_lvi.stateMask = (($p3) as _); }; { _ms_lvi.state = (($p2) as _); }; SendMessageW((($p0) as _), (((0x1000 + 43)) as _), (($p1) as WPARAM), ((core::ptr::addr_of_mut!(_ms_lvi)) as LPARAM)); } }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! SelectFont { ($p0:expr, $p1:expr $(,)?) => { ((SelectObject((($p0) as _), (((($p1) as HFONT)) as HGDIOBJ))) as HFONT) }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! min { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 < $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
pub const ATTACH_PARENT_PROCESS: DWORD = (((-(1 as c_int))) as DWORD);
pub const CW_USEDEFAULT: c_int = ((0x80000000u32) as c_int);
pub const DB_GENINST_DELDIR: *const u16 = (&[0x0044, 0x0065, 0x006c, 0x0044, 0x0069, 0x0072, 0x0000u16]).as_ptr(); // L"DelDir"
pub const DB_GENINST_DELDIREMPTY: *const u16 = (&[0x0044, 0x0065, 0x006c, 0x0044, 0x0069, 0x0072, 0x0045, 0x006d, 0x0070, 0x0074, 0x0079, 0x0000u16]).as_ptr(); // L"DelDirEmpty"
pub const DB_GENINST_DELFILE: *const u16 = (&[0x0044, 0x0065, 0x006c, 0x0046, 0x0069, 0x006c, 0x0065, 0x0000u16]).as_ptr(); // L"DelFile"
pub const DB_GENINST_DELREG: *const u16 = (&[0x0044, 0x0065, 0x006c, 0x0052, 0x0065, 0x0067, 0x0000u16]).as_ptr(); // L"DelReg"
pub const DB_GENINST_DELREGEMPTY: *const u16 = (&[0x0044, 0x0065, 0x006c, 0x0052, 0x0065, 0x0067, 0x0045, 0x006d, 0x0070, 0x0074, 0x0079, 0x0000u16]).as_ptr(); // L"DelRegEmpty"
pub const DB_GENINST_DIR: *const u16 = (&[0x0044, 0x0069, 0x0072, 0x0000u16]).as_ptr(); // L"Dir"
pub const DB_GENINST_FILES: *const u16 = (&[0x0046, 0x0069, 0x006c, 0x0065, 0x0073, 0x0000u16]).as_ptr(); // L"Files"
pub const DB_GENINST_ICON: *const u16 = (&[0x0049, 0x0063, 0x006f, 0x006e, 0x0000u16]).as_ptr(); // L"Icon"
pub const DB_GENINST_LNK: *const u16 = (&[0x004c, 0x006e, 0x006b, 0x0000u16]).as_ptr(); // L"Lnk"
pub const DB_GENINST_LNKARGS: *const u16 = (&[0x004c, 0x006e, 0x006b, 0x0050, 0x0061, 0x0072, 0x0061, 0x006d, 0x0065, 0x0074, 0x0065, 0x0072, 0x0073, 0x0000u16]).as_ptr(); // L"LnkParameters"
pub const DB_NONE: *const u16 = (&[0x0021, 0x0000u16]).as_ptr(); // L"!"
pub const ENUM_CURRENT_SETTINGS: DWORD = (((-(1 as c_int))) as DWORD);
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const IDC_APPSTARTING: LPWSTR = 0x7f8ausize as LPWSTR;
pub const IDC_ARROW: LPWSTR = 0x7f00usize as LPWSTR;
pub const IDC_SIZENS: LPWSTR = 0x7f85usize as LPWSTR;
pub const IDC_SIZEWE: LPWSTR = 0x7f84usize as LPWSTR;
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub const LOCALE_USER_DEFAULT: DWORD = ((((((((0x0) as USHORT)) as DWORD) << 16) | ((((((((((0x01) as USHORT)) as c_int) << 10) | ((((0x00) as USHORT)) as c_int))) as USHORT)) as DWORD))) as DWORD);
pub type LPNMTREEVIEW = LPNMTREEVIEWW;
pub type LPTOOLTIPTEXT = LPNMTTDISPINFOW;
pub type LVCOLUMN = LVCOLUMNW;
pub type LVFINDINFO = LVFINDINFOW;
pub type LVITEM = LVITEMW;
pub type LV_ITEM = LVITEMW;
pub const MANIFEST_DOTEXT: *const u16 = (&[0x002e, 0x0074, 0x0078, 0x0074, 0x0000u16]).as_ptr(); // L".txt"
pub const MSG_SUCCESS_INSTALL: ULONG = ((0xfff0001) as ULONG);
pub const MSG_SUCCESS_REMOVE: ULONG = ((0xfff0002) as ULONG);
pub const REGPATH_UNINSTALL: *const u16 = (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0055, 0x006e, 0x0069, 0x006e, 0x0073, 0x0074, 0x0061, 0x006c, 0x006c, 0x0000u16]).as_ptr(); // L"Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"
pub const SETTINGSSUBKEY: *const u16 = (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004f, 0x0053, 0x005c, 0x0052, 0x0041, 0x0070, 0x0070, 0x0073, 0x0000u16]).as_ptr(); // L"Software\\ReactOS\\" L"RApps"
pub const STILL_ACTIVE: NTSTATUS = 0x103usize as NTSTATUS;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub const TVI_LAST: HTREEITEM = (((((-(0xfffe as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub type TV_INSERTSTRUCT = TVINSERTSTRUCTW;
pub const UNINSTALL_SUBKEY: *const u16 = (&[0x0053, 0x006f, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005c, 0x004d, 0x0069, 0x0063, 0x0072, 0x006f, 0x0073, 0x006f, 0x0066, 0x0074, 0x005c, 0x0057, 0x0069, 0x006e, 0x0064, 0x006f, 0x0077, 0x0073, 0x005c, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006e, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006f, 0x006e, 0x005c, 0x0055, 0x006e, 0x0069, 0x006e, 0x0073, 0x0074, 0x0061, 0x006c, 0x006c, 0x0000u16]).as_ptr(); // L"Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall"
pub const WC_COMBOBOX: *const u16 = (&[0x0043, 0x006f, 0x006d, 0x0062, 0x006f, 0x0042, 0x006f, 0x0078, 0x0000u16]).as_ptr(); // L"ComboBox"
pub const WC_LISTVIEW: *const u16 = (&[0x0053, 0x0079, 0x0073, 0x004c, 0x0069, 0x0073, 0x0074, 0x0056, 0x0069, 0x0065, 0x0077, 0x0033, 0x0032, 0x0000u16]).as_ptr(); // L"SysListView32"
pub use crate::{CreateDialogIndirectW, CreateWindowW, DPA_GetPtrCount, DialogBoxW, GetStockFont, Header_GetItem, Header_SetItem, INDEXTOSTATEIMAGEMASK, ImageList_AddIcon, ImageList_RemoveAll, ListView_FindItem, ListView_GetCheckState, ListView_GetItem, ListView_GetItemCount, ListView_GetItemRect, ListView_GetNextItem, ListView_HitTest, ListView_InsertColumn, ListView_InsertItem, ListView_Scroll, ListView_SetCheckState, ListView_SetColumnWidth, ListView_SetExtendedListViewStyleEx, ListView_SetItem, ListView_SetItemState, MAKEINTRESOURCE, MAKELPARAM, SelectFont, max, min};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/rapps/include/appview.h:60: #define STATEIMAGETOINDEX(x) (((x)&LVIS_STATEIMAGEMASK) >> 12)
#[allow(unused_macros)]
macro_rules! STATEIMAGETOINDEX {
    ($x:expr $(,)?) => {
        ((($x) & LVIS_STATEIMAGEMASK) >> 12)
    };
}
#[allow(unused_imports)]
pub(crate) use STATEIMAGETOINDEX;
// ==== end port macros ====

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const ASSOCSTR_COMMAND: ASSOCSTR = ASSOCSTR_ASSOCSTR_COMMAND; // island enum
pub const ASYNCINET_CANCELLED: ASYNC_EVENT = ASYNC_EVENT_ASYNCINET_CANCELLED; // island enum
pub const ASYNCINET_COMPLETE: ASYNC_EVENT = ASYNC_EVENT_ASYNCINET_COMPLETE; // island enum
pub const ASYNCINET_DATA: ASYNC_EVENT = ASYNC_EVENT_ASYNCINET_DATA; // island enum
pub const ASYNCINET_ERROR: ASYNC_EVENT = ASYNC_EVENT_ASYNCINET_ERROR; // island enum
pub const AppViewTypeAvailableApps: APPLICATION_VIEW_TYPE = APPLICATION_VIEW_TYPE_AppViewTypeAvailableApps; // island enum
pub const AppViewTypeInstalledApps: APPLICATION_VIEW_TYPE = APPLICATION_VIEW_TYPE_AppViewTypeInstalledApps; // island enum
pub const DAF_MODAL: DOWNLOADAPPFLAGS = DOWNLOADAPPFLAGS_DAF_MODAL; // island enum
pub const DAF_SILENT: DOWNLOADAPPFLAGS = DOWNLOADAPPFLAGS_DAF_SILENT; // island enum
pub const ENUM_ALL_AVAILABLE: AppsCategories = AppsCategories_ENUM_ALL_AVAILABLE; // island enum
pub const ENUM_ALL_INSTALLED: AppsCategories = AppsCategories_ENUM_ALL_INSTALLED; // island enum
pub const ENUM_AVAILABLE_MAX: AppsCategories = AppsCategories_ENUM_AVAILABLE_MAX; // island enum
pub const ENUM_AVAILABLE_MIN: AppsCategories = AppsCategories_ENUM_AVAILABLE_MIN; // island enum
pub const ENUM_CAT_AUDIO: AppsCategories = AppsCategories_ENUM_CAT_AUDIO; // island enum
pub const ENUM_CAT_DEVEL: AppsCategories = AppsCategories_ENUM_CAT_DEVEL; // island enum
pub const ENUM_CAT_DRIVERS: AppsCategories = AppsCategories_ENUM_CAT_DRIVERS; // island enum
pub const ENUM_CAT_EDU: AppsCategories = AppsCategories_ENUM_CAT_EDU; // island enum
pub const ENUM_CAT_ENGINEER: AppsCategories = AppsCategories_ENUM_CAT_ENGINEER; // island enum
pub const ENUM_CAT_FINANCE: AppsCategories = AppsCategories_ENUM_CAT_FINANCE; // island enum
pub const ENUM_CAT_GAMES: AppsCategories = AppsCategories_ENUM_CAT_GAMES; // island enum
pub const ENUM_CAT_GRAPHICS: AppsCategories = AppsCategories_ENUM_CAT_GRAPHICS; // island enum
pub const ENUM_CAT_INTERNET: AppsCategories = AppsCategories_ENUM_CAT_INTERNET; // island enum
pub const ENUM_CAT_LIBS: AppsCategories = AppsCategories_ENUM_CAT_LIBS; // island enum
pub const ENUM_CAT_OFFICE: AppsCategories = AppsCategories_ENUM_CAT_OFFICE; // island enum
pub const ENUM_CAT_OTHER: AppsCategories = AppsCategories_ENUM_CAT_OTHER; // island enum
pub const ENUM_CAT_SCIENCE: AppsCategories = AppsCategories_ENUM_CAT_SCIENCE; // island enum
pub const ENUM_CAT_SELECTED: AppsCategories = AppsCategories_ENUM_CAT_SELECTED; // island enum
pub const ENUM_CAT_THEMES: AppsCategories = AppsCategories_ENUM_CAT_THEMES; // island enum
pub const ENUM_CAT_TOOLS: AppsCategories = AppsCategories_ENUM_CAT_TOOLS; // island enum
pub const ENUM_CAT_VIDEO: AppsCategories = AppsCategories_ENUM_CAT_VIDEO; // island enum
pub const ENUM_INSTALLED_APPLICATIONS: AppsCategories = AppsCategories_ENUM_INSTALLED_APPLICATIONS; // island enum
pub const ENUM_INSTALLED_MAX: AppsCategories = AppsCategories_ENUM_INSTALLED_MAX; // island enum
pub const ENUM_INSTALLED_MIN: AppsCategories = AppsCategories_ENUM_INSTALLED_MIN; // island enum
pub const ENUM_INVALID: AppsCategories = AppsCategories_ENUM_INVALID; // island enum
pub const ENUM_LASTCATEGORY: AppsCategories = AppsCategories_ENUM_LASTCATEGORY; // island enum
pub const ENUM_UPDATES: AppsCategories = AppsCategories_ENUM_UPDATES; // island enum
pub const FDIERROR_NONE: FDIERROR = FDIERROR_FDIERROR_NONE; // island enum
pub const INSTALLER_EXEINZIP: InstallerType = InstallerType_INSTALLER_EXEINZIP; // island enum
pub const INSTALLER_GENERATE: InstallerType = InstallerType_INSTALLER_GENERATE; // island enum
pub const INSTALLER_INNO: InstallerType = InstallerType_INSTALLER_INNO; // island enum
pub const INSTALLER_MSI: InstallerType = InstallerType_INSTALLER_MSI; // island enum
pub const INSTALLER_NSIS: InstallerType = InstallerType_INSTALLER_NSIS; // island enum
pub const INSTALLER_UNKNOWN: InstallerType = InstallerType_INSTALLER_UNKNOWN; // island enum
pub const INTERNET_MAX_URL_LENGTH: u32 = 0x824; // dwnl-rust/src/prelude.rs
pub const INTERNET_SCHEME_FILE: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_FILE; // island enum
pub const INTERNET_SCHEME_FTP: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_FTP; // island enum
pub const INTERNET_SCHEME_HTTP: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_HTTP; // island enum
pub const INTERNET_SCHEME_HTTPS: INTERNET_SCHEME = INTERNET_SCHEME_INTERNET_SCHEME_HTTPS; // island enum
pub const INT_MAX: u32 = 2147483647; // comctl32-rust/src/prelude.rs
pub const KF_FLAG_CREATE: KNOWN_FOLDER_FLAG = KNOWN_FOLDER_FLAG_KF_FLAG_CREATE; // island enum
pub const LANG_USER_DEFAULT: u32 = 1024; // arp-rust/src/prelude.rs
pub const LICENSE_FREEWARE: LicenseType = LicenseType_LICENSE_FREEWARE; // island enum
pub const LICENSE_MAX: LicenseType = LicenseType_LICENSE_MAX; // island enum
pub const LICENSE_NONE: LicenseType = LicenseType_LICENSE_NONE; // island enum
pub const LICENSE_OPENSOURCE: LicenseType = LicenseType_LICENSE_OPENSOURCE; // island enum
pub const LICENSE_TRIAL: LicenseType = LicenseType_LICENSE_TRIAL; // island enum
pub const LVM_GETITEM: i32 = 0x104B; // servman-rust/src/prelude.rs
pub const LVM_INSERTCOLUMN: i32 = 0x1061; // sndvol32-rust/src/prelude.rs
pub const LVM_INSERTITEM: i32 = 0x104D; // dxdiag-rust/src/prelude.rs
pub const LVM_SETITEMTEXT: i32 = 0x1074; // msconfig-rust/src/prelude.rs
pub const NM_CLICK: u32 = 0xFFFFFFFE; // comctl32-rust/src/prelude.rs
pub const PERCEIVED_TYPE_APPLICATION: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_APPLICATION; // island enum
pub const PERCEIVED_TYPE_COMPRESSED: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_COMPRESSED; // island enum
pub const PERCEIVED_TYPE_UNKNOWN: tagPERCEIVED = tagPERCEIVED_PERCEIVED_TYPE_UNKNOWN; // island enum
pub const SB_SETTEXT: i32 = 0x40B; // charmap-rust/src/prelude.rs
pub const SCRNSHOT_PREV_EMPTY: SCRNSHOT_STATUS = SCRNSHOT_STATUS_SCRNSHOT_PREV_EMPTY; // island enum
pub const SCRNSHOT_PREV_FAILED: SCRNSHOT_STATUS = SCRNSHOT_STATUS_SCRNSHOT_PREV_FAILED; // island enum
pub const SCRNSHOT_PREV_IMAGE: SCRNSHOT_STATUS = SCRNSHOT_STATUS_SCRNSHOT_PREV_IMAGE; // island enum
pub const SCRNSHOT_PREV_LOADING: SCRNSHOT_STATUS = SCRNSHOT_STATUS_SCRNSHOT_PREV_LOADING; // island enum
pub const SHGFP_TYPE_CURRENT: SHGFP_TYPE = SHGFP_TYPE_SHGFP_TYPE_CURRENT; // island enum
pub const TB_ADDBUTTONS: i32 = 0x444; // mplay32-rust/src/prelude.rs
pub const UCF_MODIFY: UninstallCommandFlags = UninstallCommandFlags_UCF_MODIFY; // island enum
pub const UCF_NONE: UninstallCommandFlags = UninstallCommandFlags_UCF_NONE; // island enum
pub const UCF_SAMEPROCESS: UninstallCommandFlags = UninstallCommandFlags_UCF_SAMEPROCESS; // island enum
pub const UCF_SILENT: UninstallCommandFlags = UninstallCommandFlags_UCF_SILENT; // island enum
pub const UINT_MAX: u32 = 4294967295; // comctl32-rust/src/prelude.rs
pub const ULONG_MAX: u32 = 4294967295; // calc-rust/src/prelude.rs
pub const UiAlign_LeftTop: CUiAlignment = CUiAlignment_UiAlign_LeftTop; // island enum
pub const UiAlign_Middle: CUiAlignment = CUiAlignment_UiAlign_Middle; // island enum
pub const UiAlign_RightBtm: CUiAlignment = CUiAlignment_UiAlign_RightBtm; // island enum
pub const UiAlign_Stretch: CUiAlignment = CUiAlignment_UiAlign_Stretch; // island enum
pub const _O_CREAT: u32 = 256; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_RDWR: u32 = 2; // msvcrt-sys/src/bindgen_amd64.rs
pub const _O_WRONLY: u32 = 1; // msvcrt-sys/src/bindgen_amd64.rs
pub const fdintCABINET_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCABINET_INFO; // island enum
pub const fdintCLOSE_FILE_INFO: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCLOSE_FILE_INFO; // island enum
pub const fdintCOPY_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintCOPY_FILE; // island enum
pub const fdintENUMERATE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintENUMERATE; // island enum
pub const fdintNEXT_CABINET: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintNEXT_CABINET; // island enum
pub const fdintPARTIAL_FILE: FDINOTIFICATIONTYPE = FDINOTIFICATIONTYPE_fdintPARTIAL_FILE; // island enum
extern "C" {
    pub fn _beginthreadex( security: *mut c_void, stack_size: u32, start_address: Option<unsafe extern "system" fn(*mut c_void) -> u32>, arglist: *mut c_void, initflag: u32, thrdaddr: *mut u32, ) -> usize; // explorer-rust/src/explorer/syspager.rs
    pub fn _swprintf(buf: *mut WCHAR, fmt: *const WCHAR, ...) -> c_int; // at-rust/src/prelude.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // apphelp-rust/src/prelude.rs
    pub fn floor(x: f64) -> f64; // cnt-rust/src/cabi/math.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn sin(x: f64) -> f64; // calc-rust/src/prelude.rs
    pub fn sqrt(x: f64) -> f64; // calc-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // access-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
    pub fn wcstoul(s: *const u16, endptr: *mut *mut u16, base: i32) -> u32; // at-rust/src/prelude.rs
    pub fn wsprintfW(arg1: LPWSTR, arg2: LPCWSTR, ...) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
extern "system" {
    pub fn AllocConsole() -> BOOL; // cmd-rust/src/prelude.rs
    pub fn AttachConsole(dwProcessId: u32) -> BOOL; // kernel32-rust/src/console/console.rs
    pub fn BeginDeferWindowPos(arg1: ::core::ffi::c_int) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn BeginPaint(arg1: HWND, arg2: LPPAINTSTRUCT) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn CheckRadioButton( arg1: HWND, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ClientToScreen(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoTaskMemFree(ptr: LPVOID); // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CopyFileW( lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateDialogIndirectParamW( arg1: HINSTANCE, arg2: LPCDLGTEMPLATE, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreateDialogParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn CreateDirectoryW( lpPathName: LPCWSTR, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateMutexW(arg1: LPSECURITY_ATTRIBUTES, arg2: BOOL, arg3: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateWindowExW( dwExStyle: DWORD, lpClassName: LPCWSTR, lpWindowName: LPCWSTR, dwStyle: DWORD, X: ::core::ffi::c_int, Y: ::core::ffi::c_int, nWidth: ::core::ffi::c_int, nHeight: ::core::ffi::c_int, hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: LPVOID, ) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn DefWindowProcW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DeferWindowPos( arg1: HDWP, arg2: HWND, arg3: HWND, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: ::core::ffi::c_int, arg8: UINT, ) -> HDWP; // aclui-rust/src/types_i386.rs
    pub fn DeleteCriticalSection(arg1: PCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn DeleteFileW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeleteMenu(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DeregisterEventSource(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyMenu(arg1: HMENU) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DestroyWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn DialogBoxParamW( arg1: HINSTANCE, arg2: LPCWSTR, arg3: HWND, arg4: DLGPROC, arg5: LPARAM, ) -> INT_PTR; // aclui-rust/src/types_i386.rs
    pub fn DispatchMessageW(arg1: *const MSG) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn DrawIconEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, arg8: HBRUSH, arg9: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EmptyClipboard() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableMenuItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDeferWindowPos(arg1: HDWP) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndDialog(arg1: HWND, arg2: INT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EndPaint(arg1: HWND, arg2: *const PAINTSTRUCT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn EnterCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn EnumDisplaySettingsW(arg1: LPCWSTR, arg2: DWORD, arg3: PDEVMODEW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FileTimeToSystemTime(arg1: *const FILETIME, arg2: LPSYSTEMTIME) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn FindWindowW(arg1: LPCWSTR, arg2: LPCWSTR) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GdiplusShutdown(arg1: ULONG_PTR); // desk-rust/src/types_i386.rs
    pub fn GetCapture() -> HWND; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetClassLongW")]
    pub fn GetClassLongPtrW(hWnd: HWND, nIndex: i32) -> ULONG_PTR; // user32-rust/src/controls/appswitch.rs
    pub fn GetClientRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentProcessId() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCurrentThread() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetCursorPos(arg1: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetDC(arg1: HWND) -> HDC; // aclui-rust/src/types_i386.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFileSizeEx(arg1: HANDLE, arg2: PLARGE_INTEGER) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetFocus() -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetKeyState(arg1: ::core::ffi::c_int) -> SHORT; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetMenu(arg1: HWND) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetMenuItemInfoW(arg1: HMENU, arg2: UINT, arg3: BOOL, arg4: LPMENUITEMINFOW) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn GetPrivateProfileSectionW( lpAppName: LPCWSTR, lpReturnedString: LPWSTR, nSize: DWORD, lpFileName: LPCWSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetSubMenu(arg1: HMENU, arg2: ::core::ffi::c_int) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn GetSysColor(arg1: ::core::ffi::c_int) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetSysColorBrush(arg1: ::core::ffi::c_int) -> HBRUSH; // aclui-rust/src/types_i386.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetSystemTime(lpSystemTime: LPSYSTEMTIME); // aclui-rust/src/types_i386.rs
    pub fn GetTempFileNameW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: UINT, arg4: LPWSTR) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GetTempPathW(arg1: DWORD, arg2: LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetVersion() -> DWORD; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn GetWindowPlacement(arg1: HWND, arg2: *mut WINDOWPLACEMENT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowRect(arg1: HWND, arg2: LPRECT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextW( hWnd: HWND, lpString: LPWSTR, nMaxCount: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_i386.rs
    pub fn GlobalAlloc(arg1: UINT, arg2: SIZE_T) -> HGLOBAL; // aclui-rust/src/types_i386.rs
    pub fn GlobalLock(arg1: HGLOBAL) -> LPVOID; // aclui-rust/src/types_i386.rs
    pub fn GlobalUnlock(arg1: HGLOBAL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn HeapFree(arg1: HANDLE, arg2: DWORD, arg3: PVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InflateRect(arg1: LPRECT, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn InitializeCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn InvalidateRect(arg1: HWND, arg2: LPCRECT, arg3: BOOL) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn IsWindowVisible(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn LeaveCriticalSection(arg1: LPCRITICAL_SECTION); // aclui-rust/src/types_i386.rs
    pub fn LoadAcceleratorsW(arg1: HINSTANCE, arg2: LPCWSTR) -> HACCEL; // aclui-rust/src/types_i386.rs
    pub fn LoadCursorW(arg1: HINSTANCE, arg2: LPCWSTR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_i386.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_i386.rs
    pub fn LoadMenuW(arg1: HINSTANCE, arg2: LPCWSTR) -> HMENU; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MsgWaitForMultipleObjects( nCount: DWORD, pHandles: *const HANDLE, fWaitAll: BOOL, dwMilliseconds: DWORD, dwWakeMask: DWORD, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn OpenClipboard(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn OutputDebugStringA(arg1: LPCSTR); // aclui-rust/src/types_i386.rs
    pub fn PeekMessageW(arg1: LPMSG, arg2: HWND, arg3: UINT, arg4: UINT, arg5: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PostQuitMessage(arg1: ::core::ffi::c_int); // aclui-rust/src/types_i386.rs
    pub fn ReadFile( arg1: HANDLE, arg2: PVOID, arg3: DWORD, arg4: PDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RedrawWindow(arg1: HWND, arg2: LPCRECT, arg3: HRGN, arg4: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RegisterEventSourceW(arg1: LPCWSTR, arg2: LPCWSTR) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn ReleaseCapture() -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReleaseDC(arg1: HWND, arg2: HDC) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn ReleaseMutex(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn RemoveDirectoryW(arg1: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ReportEventW( hEventLog: HANDLE, wType: WORD, wCategory: WORD, dwEventID: DWORD, lpUserSid: PSID, wNumStrings: WORD, dwDataSize: DWORD, lpStrings: *mut LPCWSTR, lpRawData: LPVOID, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ScreenToClient(arg1: HWND, arg2: LPPOINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_i386.rs
    pub fn SetCapture(hWnd: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetClipboardData(arg1: UINT, arg2: HANDLE) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn SetCursor(arg1: HCURSOR) -> HCURSOR; // aclui-rust/src/types_i386.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetFilePointer(arg1: HANDLE, arg2: LONG, arg3: PLONG, arg4: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_i386.rs
    pub fn SetForegroundWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetLastError(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SetMenuDefaultItem(arg1: HMENU, arg2: UINT, arg3: UINT) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetProcessDefaultLayout(arg1: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetThreadPriority(arg1: HANDLE, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_i386.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // charmap-rust/src/prelude.rs
    pub fn SetWindowPos( arg1: HWND, arg2: HWND, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SwitchToThisWindow(arg1: HWND, arg2: BOOL); // aclui-rust/src/types_i386.rs
    pub fn SystemParametersInfoW( uiAction: UINT, uiParam: UINT, pvParam: PVOID, fWinIni: UINT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SystemTimeToTzSpecificLocalTime( arg1: *const TIME_ZONE_INFORMATION, arg2: *const SYSTEMTIME, arg3: LPSYSTEMTIME, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TrackPopupMenu( arg1: HMENU, arg2: UINT, arg3: ::core::ffi::c_int, arg4: ::core::ffi::c_int, arg5: ::core::ffi::c_int, arg6: HWND, arg7: LPCRECT, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn TranslateAcceleratorW(arg1: HWND, arg2: HACCEL, arg3: LPMSG) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn TranslateMessage(arg1: *const MSG) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn UpdateWindow(arg1: HWND) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::rapps::asyncinet_cpp_h::AsyncInetCancel;
pub use crate::rapps::asyncinet_cpp_h::AsyncInetDownload;
pub use crate::rapps::asyncinet_cpp_h::AsyncInetRelease;
pub use crate::rapps::misc_h_h::BuildPath__T0;
pub use crate::rapps::misc_h_h::BuildPath__T1;
pub use crate::rapps::misc_h_h::BuildPath__T2;
pub use crate::rapps::misc_h_h::BuildPath__T3;
pub use crate::rapps::misc_cpp_h::ClassifyFile;
pub use crate::rapps::misc_cpp_h::CopyTextToClipboard;
pub use crate::rapps::misc_cpp_h::CreateDirectoryTree;
pub use crate::rapps::settingsdlg_cpp_h::CreateSettingsDlg;
pub use crate::rapps::misc_cpp_h::DeleteDirectoryTree;
pub use crate::rapps::loaddlg_cpp_h::DownloadApplicationsDB;
pub use crate::rapps::loaddlg_cpp_h::DownloadListOfApplications;
pub use crate::rapps::misc_cpp_h::EmulateDialogReposition;
pub use crate::rapps::misc_cpp_h::ErrorBox__0;
pub use crate::rapps::misc_h_h::ErrorFromHResult;
pub use crate::rapps::misc_cpp_h::ExpandEnvStrings;
pub use crate::rapps::cabinet_cpp_h::ExtractFilesFromCab__0;
pub use crate::rapps::cabinet_cpp_h::ExtractFilesFromCab__1;
pub use crate::rapps::misc_cpp_h::FreeLogs;
pub use crate::rapps::misc_cpp_h::GetInstalledVersion;
pub use crate::rapps::misc_cpp_h::GetNTVersion;
pub use crate::rapps::misc_cpp_h::GetProgramFilesPath;
pub use crate::rapps::misc_cpp_h::GetRegString;
pub use crate::rapps::misc_cpp_h::GetSilentInstallParameters;
pub use crate::rapps::misc_cpp_h::GetSpecialPath;
pub use crate::rapps::misc_cpp_h::GetStorageDirectory;
pub use crate::rapps::misc_cpp_h::GetSystemColorDepth;
pub use crate::rapps::misc_h_h::HResultFromWin32;
pub use crate::rapps::misc_cpp_h::InitLogs;
pub use crate::rapps::appinfo_h_h::IsKnownLicenseType;
pub use crate::rapps::misc_cpp_h::IsSystem64Bit;
pub use crate::rapps::misc_h_h::NotifyFileExtractCallback;
pub use crate::rapps::misc_cpp_h::OpensWithExplorer;
pub use crate::rapps::unattended_cpp_h::ParseCmdAndExecute;
pub use crate::rapps::settings_cpp_h::SaveSettings;
pub use crate::rapps::misc_cpp_h::SearchPatternMatch;
pub use crate::rapps::misc_cpp_h::ShowPopupMenuEx;
pub use crate::rapps::misc_cpp_h::SplitFileAndDirectory;
pub use crate::rapps::misc_cpp_h::StartProcess;
pub use crate::rapps::geninst_cpp_h::UninstallGenerated;
pub use crate::rapps::misc_cpp_h::UnixTimeToFileTime;
pub use crate::rapps::misc_cpp_h::WriteLogMessage;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, RtlUnicodeStringPrintf, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const BFFM_VALIDATEFAILED: i32 = 4; // clang, settingsdlg.cpp
pub const FILENAME_VALID_CHAR: i32 = 0x1E5; // clang, loaddlg.cpp
pub const IM_END: u32 = 0x8002; // clang, geninst.cpp
pub const IM_PROGRESS: u32 = 0x8001; // clang, geninst.cpp
pub const IM_STARTPROGRESS: u32 = 0x8000; // clang, geninst.cpp
pub const LVM_FINDITEM: i32 = 0x1053; // clang, appdb.cpp
pub const LVM_GETITEMTEXT: i32 = 0x1073; // clang, appdb.cpp
pub const MAX_SCRNSHOT_NUM: i32 = 0x10; // clang, appinfo.cpp
pub const NM_DBLCLK: u32 = 0xFFFFFFFD; // clang, appview.cpp
pub const SEARCH_TIMER_ID: i32 = 0x5352; // clang, gui.cpp
pub const TB_GETBUTTONINFO: i32 = 0x43F; // clang, appdb.cpp
pub const TTN_GETDISPINFO: u32 = 0xFFFFFDEE; // clang, appview.cpp
pub const TVM_INSERTITEM: i32 = 0x1132; // clang, appdb.cpp
pub const TVN_ITEMEXPANDING: u32 = 0xFFFFFE3A; // clang, gui.cpp
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // clang, gui.cpp
pub const UNOP_DIR: u32 = 0x44; // clang, geninst.cpp
pub const UNOP_EMPTYDIR: u32 = 0x64; // clang, geninst.cpp
pub const UNOP_EMPTYREGKEY: u32 = 0x6B; // clang, geninst.cpp
pub const UNOP_FILE: u32 = 0x46; // clang, geninst.cpp
pub const UNOP_REGKEY: u32 = 0x4B; // clang, geninst.cpp
pub const WM_GETINSTANCE: u32 = 0x8002; // clang, loaddlg.cpp
pub const WM_GETNEXT: u32 = 0x8003; // clang, loaddlg.cpp
pub const WM_ISCANCELLED: u32 = 0x8000; // clang, loaddlg.cpp
pub const WM_SETSTATUS: u32 = 0x8001; // clang, loaddlg.cpp
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetPrivateProfileStringW( lpAppName: LPCWSTR, lpKeyName: LPCWSTR, lpDefault: LPCWSTR, lpReturnedString: LPWSTR, nSize: DWORD, lpFileName: LPCWSTR, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int) -> LONG; // aclui-rust/src/types_i386.rs
    pub fn GetWindowTextLengthW(arg1: HWND) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MoveFileExW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn PathParseIconLocationW(arg1: LPWSTR) -> ::core::ffi::c_int; // certmgr-rust/src/types_i386.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegEnumValueW( hKey: HKEY, dwIndex: DWORD, lpValueName: LPWSTR, lpcchValueName: LPDWORD, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryInfoKeyW( hKey: HKEY, lpClass: LPWSTR, lpcchClass: LPDWORD, lpReserved: LPDWORD, lpcSubKeys: LPDWORD, lpcbMaxSubKeyLen: LPDWORD, lpcbMaxClassLen: LPDWORD, lpcValues: LPDWORD, lpcbMaxValueNameLen: LPDWORD, lpcbMaxValueLen: LPDWORD, lpcbSecurityDescriptor: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn SetWindowLongW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LONG) -> LONG; // aclui-rust/src/types_i386.rs
}
pub use crate::prelude::CreateEventW as CreateEvent; // sdk alias
pub use crate::prelude::CreateWindowExW as CreateWindowEx; // sdk alias
pub use crate::prelude::DeleteFileW as DeleteFile; // sdk alias
pub use crate::prelude::DispatchMessageW as DispatchMessage; // sdk alias
pub use crate::prelude::GetModuleFileNameW as GetModuleFileName; // sdk alias
pub use crate::prelude::GetPrivateProfileStringW as GetPrivateProfileString; // sdk alias
pub use crate::prelude::GetWindowLongW as GetWindowLong; // sdk alias
pub use crate::prelude::GetWindowLongPtrW as GetWindowLongPtr; // sdk alias
pub use crate::prelude::GetWindowTextW as GetWindowText; // sdk alias
pub use crate::prelude::GetWindowTextLengthW as GetWindowTextLength; // sdk alias
pub use crate::prelude::LoadCursorW as LoadCursor; // sdk alias
pub use crate::prelude::LoadImageW as LoadImage; // sdk alias
pub use crate::prelude::MessageBoxW as MessageBox; // sdk alias
pub use crate::prelude::MoveFileExW as MoveFileEx; // sdk alias
pub use crate::prelude::PathParseIconLocationW as PathParseIconLocation; // sdk alias
pub use crate::prelude::PostMessageW as PostMessage; // sdk alias
pub use crate::prelude::RegCreateKeyExW as RegCreateKeyEx; // sdk alias
pub use crate::prelude::RegEnumKeyExW as RegEnumKeyEx; // sdk alias
pub use crate::prelude::RegEnumValueW as RegEnumValue; // sdk alias
pub use crate::prelude::RegOpenKeyExW as RegOpenKeyEx; // sdk alias
pub use crate::prelude::RegQueryInfoKeyW as RegQueryInfoKey; // sdk alias
pub use crate::prelude::RegQueryValueExW as RegQueryValueEx; // sdk alias
pub use crate::prelude::RegSetValueExW as RegSetValueEx; // sdk alias
pub use crate::prelude::RemoveDirectoryW as RemoveDirectory; // sdk alias
pub use crate::prelude::SendMessageW as SendMessage; // sdk alias
pub use crate::prelude::SetWindowLongW as SetWindowLong; // sdk alias
pub use crate::prelude::SetWindowLongPtrW as SetWindowLongPtr; // sdk alias
pub use crate::prelude::SetWindowTextW as SetWindowText; // sdk alias
pub use crate::prelude::wcslen as _tcslen; // sdk alias
// ==== end port externs ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
pub type LPCDLGTEMPLATE = LPCDLGTEMPLATEW;
pub type MENUITEMINFO = MENUITEMINFOW;
pub type WNDCLASSEX = WNDCLASSEXW;
// ==== end UNICODE TCHAR family ====
