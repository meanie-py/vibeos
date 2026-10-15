//! wscript-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// The wine/debug.h channel (TRACE/ERR/WARN/FIXME): wine's own flag test on the default
// channel, then ros_dbg_log -- win32macros-rust's WINE_*! macros, which expand HERE (the
// channel storage and the ros_dbg_log extern are this crate's; `port resolve` writes them).
// MACROS, never out-of-line fns: an out-of-line wrapper is an optimization barrier that
// uncloaks latent ABI mismatches into ud2 (field manual §9.65).
#[macro_export] macro_rules! TRACE { ($($arg:tt)*) => { win32macros_rust::WINE_TRACE!($($arg)*) }; }
#[macro_export] macro_rules! ERR   { ($($arg:tt)*) => { win32macros_rust::WINE_ERR!($($arg)*) }; }
#[macro_export] macro_rules! WARN  { ($($arg:tt)*) => { win32macros_rust::WINE_WARN!($($arg)*) }; }
#[macro_export] macro_rules! FIXME { ($($arg:tt)*) => { win32macros_rust::WINE_FIXME!($($arg)*) }; }
pub use crate::{ERR, FIXME, TRACE, WARN};

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
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ARRAY_SIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, ERR_ON, FAILED_UNEXPECTEDLY, FIELD_OFFSET, FIXME_ON, GET_WHEEL_DELTA_WPARAM, GET_X_LPARAM, GET_Y_LPARAM, GetAValue, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HRESULT_FROM_WIN32, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, IS_INTRESOURCE, InitializeObjectAttributes, IntToPtr, InterlockedCompareExchange, InterlockedDecrement, InterlockedExchange, InterlockedIncrement, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTATOM, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RGBA, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, TRACE_ON, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNIMPLEMENTED_DBGBREAK, UNREACHABLE, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, WARN_ON, WINE_ERR, WINE_FIXME, WINE_TRACE, WINE_WARN, ZeroMemory, __HRESULT_FROM_WIN32, __p___argc, __p___argv, __p___wargv, __p__environ, __p__pgmptr, __p__wenviron, __p__wpgmptr, __wine_dprintf, __wine_is_debug_on, _countof, debugstr_a, debugstr_an, debugstr_guid, debugstr_w, debugstr_wn, heap_alloc, heap_alloc_zero, heap_calloc, heap_free, heap_realloc, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, wide_bytes, wine_dbgstr_a, wine_dbgstr_guid, wine_dbgstr_w};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ================= hand ports (above the generated blocks: they survive replays) ==============
// ---- msxml2.h vtables (the C reaches these through COBJMACROS, `->lpVtbl->slot`). The island does not
// carry them, so the layout is transcribed from the build's generated msxml2.h in slot order. Only the
// slots this module calls are typed; the others are placeholders of the same width (every slot is a
// function pointer), so the used slots sit at the right offsets.
// IXMLDOMNodeVtbl: 43 slots, layout from msxml2.h (generated build header)
#[allow(non_snake_case)]
#[repr(C)]
pub struct IXMLDOMNodeVtbl {
    pub QueryInterface: *const c_void, // not called by this module
    pub AddRef: *const c_void, // not called by this module
    pub Release: Option<unsafe extern "system" fn(This: *mut IXMLDOMNode) -> ULONG>,
    pub GetTypeInfoCount: *const c_void, // not called by this module
    pub GetTypeInfo: *const c_void, // not called by this module
    pub GetIDsOfNames: *const c_void, // not called by this module
    pub Invoke: *const c_void, // not called by this module
    pub get_nodeName: *const c_void, // not called by this module
    pub get_nodeValue: *const c_void, // not called by this module
    pub put_nodeValue: *const c_void, // not called by this module
    pub get_nodeType: *const c_void, // not called by this module
    pub get_parentNode: *const c_void, // not called by this module
    pub get_childNodes: *const c_void, // not called by this module
    pub get_firstChild: *const c_void, // not called by this module
    pub get_lastChild: *const c_void, // not called by this module
    pub get_previousSibling: *const c_void, // not called by this module
    pub get_nextSibling: *const c_void, // not called by this module
    pub get_attributes: Option<unsafe extern "system" fn(This: *mut IXMLDOMNode, attributeMap: *mut *mut IXMLDOMNamedNodeMap) -> HRESULT>,
    pub insertBefore: *const c_void, // not called by this module
    pub replaceChild: *const c_void, // not called by this module
    pub removeChild: *const c_void, // not called by this module
    pub appendChild: *const c_void, // not called by this module
    pub hasChildNodes: *const c_void, // not called by this module
    pub get_ownerDocument: *const c_void, // not called by this module
    pub cloneNode: *const c_void, // not called by this module
    pub get_nodeTypeString: *const c_void, // not called by this module
    pub get_text: Option<unsafe extern "system" fn(This: *mut IXMLDOMNode, p: *mut BSTR) -> HRESULT>,
    pub put_text: *const c_void, // not called by this module
    pub get_specified: *const c_void, // not called by this module
    pub get_definition: *const c_void, // not called by this module
    pub get_nodeTypedValue: *const c_void, // not called by this module
    pub put_nodeTypedValue: *const c_void, // not called by this module
    pub get_dataType: *const c_void, // not called by this module
    pub put_dataType: *const c_void, // not called by this module
    pub get_xml: *const c_void, // not called by this module
    pub transformNode: *const c_void, // not called by this module
    pub selectNodes: *const c_void, // not called by this module
    pub selectSingleNode: *const c_void, // not called by this module
    pub get_parsed: *const c_void, // not called by this module
    pub get_namespaceURI: *const c_void, // not called by this module
    pub get_prefix: *const c_void, // not called by this module
    pub get_baseName: *const c_void, // not called by this module
    pub transformNodeToObject: *const c_void, // not called by this module
}

// IXMLDOMDocumentVtbl: 76 slots, layout from msxml2.h (generated build header)
#[allow(non_snake_case)]
#[repr(C)]
pub struct IXMLDOMDocumentVtbl {
    pub QueryInterface: *const c_void, // not called by this module
    pub AddRef: *const c_void, // not called by this module
    pub Release: Option<unsafe extern "system" fn(This: *mut IXMLDOMDocument) -> ULONG>,
    pub GetTypeInfoCount: *const c_void, // not called by this module
    pub GetTypeInfo: *const c_void, // not called by this module
    pub GetIDsOfNames: *const c_void, // not called by this module
    pub Invoke: *const c_void, // not called by this module
    pub get_nodeName: *const c_void, // not called by this module
    pub get_nodeValue: *const c_void, // not called by this module
    pub put_nodeValue: *const c_void, // not called by this module
    pub get_nodeType: *const c_void, // not called by this module
    pub get_parentNode: *const c_void, // not called by this module
    pub get_childNodes: *const c_void, // not called by this module
    pub get_firstChild: *const c_void, // not called by this module
    pub get_lastChild: *const c_void, // not called by this module
    pub get_previousSibling: *const c_void, // not called by this module
    pub get_nextSibling: *const c_void, // not called by this module
    pub get_attributes: *const c_void, // not called by this module
    pub insertBefore: *const c_void, // not called by this module
    pub replaceChild: *const c_void, // not called by this module
    pub removeChild: *const c_void, // not called by this module
    pub appendChild: *const c_void, // not called by this module
    pub hasChildNodes: *const c_void, // not called by this module
    pub get_ownerDocument: *const c_void, // not called by this module
    pub cloneNode: *const c_void, // not called by this module
    pub get_nodeTypeString: *const c_void, // not called by this module
    pub get_text: *const c_void, // not called by this module
    pub put_text: *const c_void, // not called by this module
    pub get_specified: *const c_void, // not called by this module
    pub get_definition: *const c_void, // not called by this module
    pub get_nodeTypedValue: *const c_void, // not called by this module
    pub put_nodeTypedValue: *const c_void, // not called by this module
    pub get_dataType: *const c_void, // not called by this module
    pub put_dataType: *const c_void, // not called by this module
    pub get_xml: *const c_void, // not called by this module
    pub transformNode: *const c_void, // not called by this module
    pub selectNodes: *const c_void, // not called by this module
    pub selectSingleNode: *const c_void, // not called by this module
    pub get_parsed: *const c_void, // not called by this module
    pub get_namespaceURI: *const c_void, // not called by this module
    pub get_prefix: *const c_void, // not called by this module
    pub get_baseName: *const c_void, // not called by this module
    pub transformNodeToObject: *const c_void, // not called by this module
    pub get_doctype: *const c_void, // not called by this module
    pub get_implementation: *const c_void, // not called by this module
    pub get_documentElement: Option<unsafe extern "system" fn(This: *mut IXMLDOMDocument, domElement: *mut *mut IXMLDOMElement) -> HRESULT>,
    pub putref_documentElement: *const c_void, // not called by this module
    pub createElement: *const c_void, // not called by this module
    pub createDocumentFragment: *const c_void, // not called by this module
    pub createTextNode: *const c_void, // not called by this module
    pub createComment: *const c_void, // not called by this module
    pub createCDATASection: *const c_void, // not called by this module
    pub createProcessingInstruction: *const c_void, // not called by this module
    pub createAttribute: *const c_void, // not called by this module
    pub createEntityReference: *const c_void, // not called by this module
    pub getElementsByTagName: *const c_void, // not called by this module
    pub createNode: *const c_void, // not called by this module
    pub nodeFromID: *const c_void, // not called by this module
    pub load: Option<unsafe extern "system" fn(This: *mut IXMLDOMDocument, var1: VARIANT, pbool: *mut VARIANT_BOOL) -> HRESULT>,
    pub get_readyState: *const c_void, // not called by this module
    pub get_parseError: *const c_void, // not called by this module
    pub get_url: *const c_void, // not called by this module
    pub get_async: *const c_void, // not called by this module
    pub put_async: *const c_void, // not called by this module
    pub abort: *const c_void, // not called by this module
    pub loadXML: *const c_void, // not called by this module
    pub save: *const c_void, // not called by this module
    pub get_validateOnParse: *const c_void, // not called by this module
    pub put_validateOnParse: *const c_void, // not called by this module
    pub get_resolveExternals: *const c_void, // not called by this module
    pub put_resolveExternals: *const c_void, // not called by this module
    pub get_preserveWhiteSpace: *const c_void, // not called by this module
    pub put_preserveWhiteSpace: *const c_void, // not called by this module
    pub put_onreadystatechange: *const c_void, // not called by this module
    pub put_ondataavailable: *const c_void, // not called by this module
    pub put_ontransformnode: *const c_void, // not called by this module
}

// IXMLDOMElementVtbl: 52 slots, layout from msxml2.h (generated build header)
#[allow(non_snake_case)]
#[repr(C)]
pub struct IXMLDOMElementVtbl {
    pub QueryInterface: *const c_void, // not called by this module
    pub AddRef: *const c_void, // not called by this module
    pub Release: Option<unsafe extern "system" fn(This: *mut IXMLDOMElement) -> ULONG>,
    pub GetTypeInfoCount: *const c_void, // not called by this module
    pub GetTypeInfo: *const c_void, // not called by this module
    pub GetIDsOfNames: *const c_void, // not called by this module
    pub Invoke: *const c_void, // not called by this module
    pub get_nodeName: *const c_void, // not called by this module
    pub get_nodeValue: *const c_void, // not called by this module
    pub put_nodeValue: *const c_void, // not called by this module
    pub get_nodeType: *const c_void, // not called by this module
    pub get_parentNode: *const c_void, // not called by this module
    pub get_childNodes: *const c_void, // not called by this module
    pub get_firstChild: *const c_void, // not called by this module
    pub get_lastChild: *const c_void, // not called by this module
    pub get_previousSibling: *const c_void, // not called by this module
    pub get_nextSibling: *const c_void, // not called by this module
    pub get_attributes: *const c_void, // not called by this module
    pub insertBefore: *const c_void, // not called by this module
    pub replaceChild: *const c_void, // not called by this module
    pub removeChild: *const c_void, // not called by this module
    pub appendChild: *const c_void, // not called by this module
    pub hasChildNodes: *const c_void, // not called by this module
    pub get_ownerDocument: *const c_void, // not called by this module
    pub cloneNode: *const c_void, // not called by this module
    pub get_nodeTypeString: *const c_void, // not called by this module
    pub get_text: *const c_void, // not called by this module
    pub put_text: *const c_void, // not called by this module
    pub get_specified: *const c_void, // not called by this module
    pub get_definition: *const c_void, // not called by this module
    pub get_nodeTypedValue: *const c_void, // not called by this module
    pub put_nodeTypedValue: *const c_void, // not called by this module
    pub get_dataType: *const c_void, // not called by this module
    pub put_dataType: *const c_void, // not called by this module
    pub get_xml: *const c_void, // not called by this module
    pub transformNode: *const c_void, // not called by this module
    pub selectNodes: *const c_void, // not called by this module
    pub selectSingleNode: *const c_void, // not called by this module
    pub get_parsed: *const c_void, // not called by this module
    pub get_namespaceURI: *const c_void, // not called by this module
    pub get_prefix: *const c_void, // not called by this module
    pub get_baseName: *const c_void, // not called by this module
    pub transformNodeToObject: *const c_void, // not called by this module
    pub get_tagName: Option<unsafe extern "system" fn(This: *mut IXMLDOMElement, p: *mut BSTR) -> HRESULT>,
    pub getAttribute: *const c_void, // not called by this module
    pub setAttribute: *const c_void, // not called by this module
    pub removeAttribute: *const c_void, // not called by this module
    pub getAttributeNode: *const c_void, // not called by this module
    pub setAttributeNode: *const c_void, // not called by this module
    pub removeAttributeNode: *const c_void, // not called by this module
    pub getElementsByTagName: Option<unsafe extern "system" fn(This: *mut IXMLDOMElement, p: BSTR, resultList: *mut *mut IXMLDOMNodeList) -> HRESULT>,
    pub normalize: *const c_void, // not called by this module
}

// IXMLDOMNamedNodeMapVtbl: 17 slots, layout from msxml2.h (generated build header)
#[allow(non_snake_case)]
#[repr(C)]
pub struct IXMLDOMNamedNodeMapVtbl {
    pub QueryInterface: *const c_void, // not called by this module
    pub AddRef: *const c_void, // not called by this module
    pub Release: Option<unsafe extern "system" fn(This: *mut IXMLDOMNamedNodeMap) -> ULONG>,
    pub GetTypeInfoCount: *const c_void, // not called by this module
    pub GetTypeInfo: *const c_void, // not called by this module
    pub GetIDsOfNames: *const c_void, // not called by this module
    pub Invoke: *const c_void, // not called by this module
    pub getNamedItem: Option<unsafe extern "system" fn(This: *mut IXMLDOMNamedNodeMap, p: BSTR, outNode: *mut *mut IXMLDOMNode) -> HRESULT>,
    pub setNamedItem: *const c_void, // not called by this module
    pub removeNamedItem: *const c_void, // not called by this module
    pub get_item: *const c_void, // not called by this module
    pub get_length: *const c_void, // not called by this module
    pub getQualifiedItem: *const c_void, // not called by this module
    pub removeQualifiedItem: *const c_void, // not called by this module
    pub nextNode: *const c_void, // not called by this module
    pub reset: *const c_void, // not called by this module
    pub get__newEnum: *const c_void, // not called by this module
}

// IXMLDOMNodeListVtbl: 12 slots, layout from msxml2.h (generated build header)
#[allow(non_snake_case)]
#[repr(C)]
pub struct IXMLDOMNodeListVtbl {
    pub QueryInterface: *const c_void, // not called by this module
    pub AddRef: *const c_void, // not called by this module
    pub Release: Option<unsafe extern "system" fn(This: *mut IXMLDOMNodeList) -> ULONG>,
    pub GetTypeInfoCount: *const c_void, // not called by this module
    pub GetTypeInfo: *const c_void, // not called by this module
    pub GetIDsOfNames: *const c_void, // not called by this module
    pub Invoke: *const c_void, // not called by this module
    pub get_item: Option<unsafe extern "system" fn(This: *mut IXMLDOMNodeList, lIndex: LONG, outNode: *mut *mut IXMLDOMNode) -> HRESULT>,
    pub get_length: *const c_void, // not called by this module
    pub nextNode: *const c_void, // not called by this module
    pub reset: *const c_void, // not called by this module
    pub get__newEnum: *const c_void, // not called by this module
}


// ---- COBJMACROS shims: `IXMLDOMNode_get_text(This,p)` == `(This)->lpVtbl->get_text(This,p)`. Each
// dispatches through the slot above; a null slot is the same crash the C would take.
pub unsafe fn IXMLDOMNode_Release(This: *mut IXMLDOMNode) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) }
pub unsafe fn IXMLDOMNode_get_attributes(This: *mut IXMLDOMNode, attributeMap: *mut *mut IXMLDOMNamedNodeMap) -> HRESULT { ((*(*This).lpVtbl).get_attributes.unwrap_unchecked())(This, attributeMap) }
pub unsafe fn IXMLDOMNode_get_text(This: *mut IXMLDOMNode, p: *mut BSTR) -> HRESULT { ((*(*This).lpVtbl).get_text.unwrap_unchecked())(This, p) }
pub unsafe fn IXMLDOMNamedNodeMap_Release(This: *mut IXMLDOMNamedNodeMap) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) }
pub unsafe fn IXMLDOMNamedNodeMap_getNamedItem(This: *mut IXMLDOMNamedNodeMap, p: BSTR, outNode: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).getNamedItem.unwrap_unchecked())(This, p, outNode) }
pub unsafe fn IXMLDOMNodeList_get_item(This: *mut IXMLDOMNodeList, lIndex: LONG, outNode: *mut *mut IXMLDOMNode) -> HRESULT { ((*(*This).lpVtbl).get_item.unwrap_unchecked())(This, lIndex, outNode) }
pub unsafe fn IXMLDOMDocument_load(This: *mut IXMLDOMDocument, var1: VARIANT, pbool: *mut VARIANT_BOOL) -> HRESULT { ((*(*This).lpVtbl).load.unwrap_unchecked())(This, var1, pbool) }
pub unsafe fn IXMLDOMDocument_get_documentElement(This: *mut IXMLDOMDocument, domElement: *mut *mut IXMLDOMElement) -> HRESULT { ((*(*This).lpVtbl).get_documentElement.unwrap_unchecked())(This, domElement) }
pub unsafe fn IXMLDOMElement_get_tagName(This: *mut IXMLDOMElement, p: *mut BSTR) -> HRESULT { ((*(*This).lpVtbl).get_tagName.unwrap_unchecked())(This, p) }
pub unsafe fn IXMLDOMElement_getElementsByTagName(This: *mut IXMLDOMElement, p: BSTR, resultList: *mut *mut IXMLDOMNodeList) -> HRESULT { ((*(*This).lpVtbl).getElementsByTagName.unwrap_unchecked())(This, p, resultList) }

// C: activscp.h `#define IActiveScriptParse IActiveScriptParse64` (or 32) on _WIN64 / otherwise.
#[cfg(target_pointer_width = "64")]
pub type IActiveScriptParse = IActiveScriptParse64;
#[cfg(not(target_pointer_width = "64"))]
pub type IActiveScriptParse = IActiveScriptParse32;

// ---- activscp.h: IActiveScriptParse is the 64 interface on _WIN64 and the 32 one otherwise (the C's
// IActiveScriptParse_* macros pick the same by _WIN64). The island carries both vtables.
macro_rules! activscript_parse_shims {
    ($I:ident, $Cookie:ty) => {
        pub unsafe fn IActiveScriptParse_Release(This: *mut $I) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) }
        pub unsafe fn IActiveScriptParse_InitNew(This: *mut $I) -> HRESULT { ((*(*This).lpVtbl).InitNew.unwrap_unchecked())(This) }
        pub unsafe fn IActiveScriptParse_ParseScriptText(This: *mut $I, pstrCode: LPCOLESTR, pstrItemName: LPCOLESTR, punkContext: *mut IUnknown, pstrDelimiter: LPCOLESTR, dwSourceContextCookie: $Cookie, ulStartingLineNumber: ULONG, dwFlags: DWORD, pvarResult: *mut VARIANT, pexcepinfo: *mut EXCEPINFO) -> HRESULT {
            ((*(*This).lpVtbl).ParseScriptText.unwrap_unchecked())(This, pstrCode, pstrItemName, punkContext, pstrDelimiter, dwSourceContextCookie, ulStartingLineNumber, dwFlags, pvarResult, pexcepinfo)
        }
    };
}
#[cfg(target_pointer_width = "64")]
activscript_parse_shims!(IActiveScriptParse64, DWORDLONG);
#[cfg(not(target_pointer_width = "64"))]
activscript_parse_shims!(IActiveScriptParse32, DWORD);
#[cfg(target_pointer_width = "64")]
pub use crate::types::IID_IActiveScriptParse64 as IID_IActiveScriptParse;
#[cfg(not(target_pointer_width = "64"))]
pub use crate::types::IID_IActiveScriptParse32 as IID_IActiveScriptParse;

// ---- CRT and shlwapi externs the C calls through wine/ReactOS macros
extern "C" {
    pub fn _wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> c_int;
    pub fn _wcsnicmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> c_int;
}
pub use crate::prelude::_wcsicmp as wcsicmp; // sdk alias (C: wcsicmp -> msvcrt _wcsicmp)
pub use crate::prelude::_wcsnicmp as wcsnicmp; // sdk alias
extern "system" {
    pub fn UrlCreateFromPathW(pszPath: LPCWSTR, pszUrl: LPWSTR, pcchUrl: LPDWORD, dwReserved: DWORD) -> HRESULT; // shlwapi.h
}

// ---- wine/debug.h `debugstr_variant` is a static inline that formats the VARIANT for WARN only.
// Same placeholder as oleaut32-rust's debug_shims: the string is diagnostic text, never program logic.
pub fn debugstr_variant(_p: *const VARIANT) -> *const c_char {
    b"?\0".as_ptr() as *const c_char
}

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
pub const CLSCTX_INPROC_HANDLER: DWORD = 0x2; // ole32-rust/src/compobj/com.rs
pub const CLSCTX_INPROC_SERVER: DWORD = 0x1; // cleanmgr-rust/src/prelude.rs
pub const CLSCTX_LOCAL_SERVER: DWORD = 0x4; // ole32-rust/src/compobj/com.rs
pub const CLSCTX_REMOTE_SERVER: DWORD = 0x10; // ole32-rust/src/compobj/com.rs
pub const CP_ACP: u32 = 0; // advapi32-sys/src/bindgen_amd64.rs
pub const SCRIPTSTATE_INITIALIZED: tagSCRIPTSTATE = tagSCRIPTSTATE_SCRIPTSTATE_INITIALIZED; // island enum
pub const SCRIPTSTATE_STARTED: tagSCRIPTSTATE = tagSCRIPTSTATE_SCRIPTSTATE_STARTED; // island enum
pub const STD_ERROR_HANDLE: u32 = 4294967284; // cmd-rust/src/prelude.rs
pub const STD_OUTPUT_HANDLE: u32 = 4294967285; // cmd-rust/src/prelude.rs
pub const VT_BSTR: DWORD = 8; // cleanmgr-rust/src/prelude.rs
pub const VT_EMPTY: u16 = 0; // msctf-rust/src/prelude.rs
pub const VT_NULL: u16 = 1; // oleaut32-rust/src/consts.rs
extern "C" {
    pub fn calloc(arg1: ::core::ffi::c_uint, arg2: ::core::ffi::c_uint) -> *mut ::core::ffi::c_void; // msvcrt-sys/src/bindgen_i386.rs
    pub fn free(p: *mut c_void); // atlclasses-rust/src/lib.rs
    pub fn malloc(size: usize) -> *mut c_void; // atlclasses-rust/src/lib.rs
    pub fn ros_dbg_log( cls: ::core::ffi::c_uint, ch: *mut __wine_debug_channel, file: *const ::core::ffi::c_char, func: *const ::core::ffi::c_char, line: ::core::ffi::c_int, format: *const ::core::ffi::c_char, ... ) -> ::core::ffi::c_int; // winegdi-rust/src/types_i386.rs
    // The COUNTED swprintf is a header inline over _vsnwprintf: msvcrt.dll's own `swprintf`
    // export is the legacy (buffer, format, ...) form, so the count would be read as the
    // format. `scan_swprintf_count` gates it.
    #[link_name = "_snwprintf"]
    pub fn swprintf(s: *mut u16, n: usize, fmt: *const u16, ...) -> c_int; // msiexec-rust/src/prelude.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
    pub fn wcstol(s: *const u16, endptr: *mut *mut u16, base: i32) -> i32; // cleanmgr-rust/src/prelude.rs
    pub static IID_IUnknown: GUID; // cicero-rust/src/types_amd64.rs
    pub static IID_IXMLDOMDocument: IID; // mmc-rust/src/prelude.rs
    pub static IID_IXMLDOMElement: IID; // mmc-rust/src/prelude.rs
}
extern "stdcall" {
    pub fn WriteConsoleW( hConsoleOutput: *mut ::core::ffi::c_void, lpBuffer: *const ::core::ffi::c_void, nNumberOfCharsToWrite: u32, lpNumberOfCharsWritten: LPDWORD, lpReserved: *mut ::core::ffi::c_void, ) -> BOOL; // msvcrt-sys/src/bindgen_i386.rs
}
extern "system" {
    pub fn CLSIDFromProgID(progid: LPCOLESTR, riid: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CLSIDFromString(arg1: LPCOLESTR, arg2: LPCLSID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CloseHandle(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn CoCreateInstance( rclsid: *const CLSID, pUnkOuter: LPUNKNOWN, dwClsContext: DWORD, iid: *const IID, ppv: *mut LPVOID, ) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoInitialize(lpReserved: LPVOID) -> HRESULT; // aclui-rust/src/types_i386.rs
    pub fn CoUninitialize(); // aclui-rust/src/types_i386.rs
    pub fn CommandLineToArgvW(arg1: LPCWSTR, arg2: *mut ::core::ffi::c_int) -> *mut LPWSTR; // ctfmon-rust/src/types_i386.rs
    pub fn CreateEventW( arg1: LPSECURITY_ATTRIBUTES, bManualReset: BOOL, bInitialState: BOOL, lpName: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileMappingW( arg1: HANDLE, arg2: LPSECURITY_ATTRIBUTES, arg3: DWORD, arg4: DWORD, arg5: DWORD, arg6: LPCWSTR, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateFileW( arg1: LPCWSTR, arg2: DWORD, arg3: DWORD, arg4: LPSECURITY_ATTRIBUTES, arg5: DWORD, arg6: DWORD, arg7: HANDLE, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn CreateThread( arg1: LPSECURITY_ATTRIBUTES, arg2: DWORD, arg3: LPTHREAD_START_ROUTINE, arg4: PVOID, arg5: DWORD, arg6: PDWORD, ) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn ExitProcess(arg1: UINT) -> !; // aclui-rust/src/types_i386.rs
    pub fn FormatMessageW( dwFlags: DWORD, lpSource: LPCVOID, dwMessageId: DWORD, dwLanguageId: DWORD, lpBuffer: LPWSTR, nSize: DWORD, Arguments: *mut va_list, ) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetCommandLineW() -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn GetFileSize(arg1: HANDLE, arg2: PDWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetFullPathNameW(arg1: LPCWSTR, arg2: DWORD, arg3: LPWSTR, arg4: *mut LPWSTR) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetLastError() -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: LPWSTR, nSize: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn GetModuleHandleW(arg1: LPCWSTR) -> HMODULE; // aclui-rust/src/types_i386.rs
    pub fn GetOEMCP() -> UINT; // desk-rust/src/types_i386.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_i386.rs
    pub fn GetStdHandle(arg1: DWORD) -> HANDLE; // aclui-rust/src/types_i386.rs
    pub fn GetUserDefaultLCID() -> LCID; // desk-rust/src/types_i386.rs
    pub fn LoadStringW( hInstance: HINSTANCE, uID: UINT, lpBuffer: LPWSTR, cchBufferMax: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn LoadTypeLib(arg1: *const OLECHAR, arg2: *mut *mut ITypeLib) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn LocalAlloc(arg1: UINT, arg2: SIZE_T) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalFree(arg1: HLOCAL) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn LocalReAlloc(arg1: HLOCAL, arg2: SIZE_T, arg3: UINT) -> HLOCAL; // aclui-rust/src/types_i386.rs
    pub fn MapViewOfFile( arg1: HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: SIZE_T, ) -> PVOID; // aclui-rust/src/types_i386.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn MultiByteToWideChar( arg1: UINT, arg2: DWORD, arg3: LPCSTR, arg4: ::core::ffi::c_int, arg5: LPWSTR, arg6: ::core::ffi::c_int, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegOpenKeyW(arg1: HKEY, arg2: LPCWSTR, arg3: PHKEY) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn RegQueryValueW( hKey: HKEY, lpSubKey: LPCWSTR, lpData: LPWSTR, lpcbData: PLONG, ) -> LSTATUS; // desk-rust/src/types_i386.rs
    pub fn ResetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn SafeArrayAccessData( arg1: *mut SAFEARRAY, arg2: *mut *mut ::core::ffi::c_void, ) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayGetDim(arg1: *mut SAFEARRAY) -> UINT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayGetLBound(arg1: *mut SAFEARRAY, arg2: UINT, arg3: *mut LONG) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayGetUBound(arg1: *mut SAFEARRAY, arg2: UINT, arg3: *mut LONG) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SafeArrayUnaccessData(arg1: *mut SAFEARRAY) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn SetEvent(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn Sleep(arg1: DWORD); // aclui-rust/src/types_i386.rs
    pub fn SysAllocString(arg1: *const OLECHAR) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysAllocStringLen(arg1: *const OLECHAR, ui: UINT) -> BSTR; // msctf-rust/src/types_i386.rs
    pub fn SysFreeString(arg1: BSTR); // msctf-rust/src/types_i386.rs
    pub fn SysStringLen(pbstr: BSTR) -> UINT; // msctf-rust/src/types_i386.rs
    pub fn UnmapViewOfFile(arg1: LPCVOID) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn VariantChangeType( arg1: *mut VARIANTARG, arg2: *const VARIANTARG, arg3: USHORT, arg4: ::core::ffi::c_ushort, ) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn VariantClear(arg1: *mut VARIANT) -> HRESULT; // msctf-rust/src/types_i386.rs
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD; // aclui-rust/src/types_i386.rs
    pub fn WideCharToMultiByte( arg1: UINT, arg2: DWORD, arg3: LPCWSTR, arg4: ::core::ffi::c_int, arg5: LPSTR, arg6: ::core::ffi::c_int, arg7: LPCSTR, arg8: LPBOOL, ) -> ::core::ffi::c_int; // desk-rust/src/types_i386.rs
    pub fn WriteFile( arg1: HANDLE, arg2: LPCVOID, arg3: DWORD, arg4: LPDWORD, arg5: LPOVERLAPPED, ) -> BOOL; // aclui-rust/src/types_i386.rs
    pub fn lstrcatW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcmpiW(arg1: LPCWSTR, arg2: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
    pub fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR; // aclui-rust/src/types_i386.rs
    pub fn lstrlenW(arg1: LPCWSTR) -> ::core::ffi::c_int; // aclui-rust/src/types_i386.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::wscript::main::IID_IArguments2;
pub use crate::wscript::main::IID_IHost;
pub use crate::wscript::arguments::arguments_obj;
pub use crate::wscript::main::arguments_ti;
pub use crate::wscript::arguments::argums;
pub use crate::wscript::host::host_obj;
pub use crate::wscript::main::host_ti;
pub use crate::wscript::arguments::numOfArgs;
pub use crate::wscript::main::scriptFullName;
pub use crate::wscript::host::wshInteractive;
pub use crate::wscript::host::wshTimeout;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatNW, StringCbCatW, StringCbCopyExW, StringCbCopyNExW, StringCbCopyNW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCbVPrintfW, StringCchCatNW, StringCchCatW, StringCchCopyA, StringCchCopyExW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthA, StringCchLengthW, StringCchPrintfA, StringCchPrintfExA, StringCchPrintfExW, StringCchPrintfW, StringCchVPrintfA, StringCchVPrintfExA, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};
pub use conutils_h::{AnsiText, Binary, CON_STREAM, CON_STREAM_MODE, CON_WRITE_FUNC, ConInitStdStreams, ConInitStdStreamsAndMode, ConMsgPrintf, ConMsgPuts, ConPrintf, ConPrintfV, ConPuts, ConResMsgPrintf, ConResMsgPrintfEx, ConResPrintf, ConResPrintfEx, ConResPrintfV, ConResPuts, ConResPutsEx, ConSetThreadUILanguage, ConStdStreamsSetCacheCodePage, ConStreamGetOSHandle, ConStreamInit, ConStreamInitEx, ConStreamSetCacheCodePage, ConStreamSetMode, ConStreamSetOSHandle, ConStreamWrite, ConWrite, INVALID_CP, K32LoadStringW, PCON_STREAM, StdErr, StdIn, StdOut, UTF16Text, UTF8Text, WideText, csStdErr, csStdIn, csStdOut};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// IXMLDOMDocument: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IXMLDOMDocument {
    pub lpVtbl: *const IXMLDOMDocumentVtbl,
}
// IXMLDOMElement: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IXMLDOMElement {
    pub lpVtbl: *const IXMLDOMElementVtbl,
}
// IXMLDOMNamedNodeMap: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IXMLDOMNamedNodeMap {
    pub lpVtbl: *const IXMLDOMNamedNodeMapVtbl,
}
// IXMLDOMNode: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IXMLDOMNode {
    pub lpVtbl: *const IXMLDOMNodeVtbl,
}
// IXMLDOMNodeList: c2rust, main.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct IXMLDOMNodeList {
    pub lpVtbl: *const IXMLDOMNodeListVtbl,
}
// SCRIPTSTATE: c2rust, main.c
pub type SCRIPTSTATE = tagSCRIPTSTATE;
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
// ==== end port externs ====

// ==== port resolve: wine debug channels (generated) ====
#[repr(C)]
pub struct __wine_debug_channel { // wine/debug.h
    pub flags: ::core::ffi::c_uchar,
    pub name: [::core::ffi::c_char; 15],
}
pub static mut __wine_dbch_wscript: __wine_debug_channel = __wine_debug_channel {
    flags: !0u8, // (unsigned char)~0
    name: unsafe { ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(*b"\x77\x73\x63\x72\x69\x70\x74\x00\x00\x00\x00\x00\x00\x00\x00") },
};
pub static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw mut __wine_dbch_wscript };
// ==== end wine debug channels ====

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
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! V_BSTR { ($p0:expr $(,)?) => { (*$p0).n1.n2.n3.bstrVal }; }
#[macro_export] macro_rules! V_VT { ($p0:expr $(,)?) => { (*$p0).n1.n2.vt }; }
#[macro_export] macro_rules! max { ($p0:expr, $p1:expr $(,)?) => { (if ((($p0 > $p1) as c_int)) as isize != 0 { $p0 } else { $p1 }) }; }
#[macro_export] macro_rules! va_end { ($p0:expr $(,)?) => { __builtin_va_end((($p0) as _)) }; }
#[macro_export] macro_rules! va_start { ($p0:expr, $p1:expr $(,)?) => { __builtin_va_start((($p0) as _), (($p1) as _)) }; }
pub const E_FAIL: HRESULT = ((0x80004005u32) as HRESULT);
pub const E_INVALIDARG: HRESULT = ((0x80070057u32) as HRESULT);
pub const E_NOINTERFACE: HRESULT = ((0x80004002u32) as HRESULT);
pub const E_NOTIMPL: HRESULT = ((0x80004001u32) as HRESULT);
pub const E_OUTOFMEMORY: HRESULT = ((0x8007000Eu32) as HRESULT);
pub const HKEY_CLASSES_ROOT: HKEY = ((((((0x80000000u32) as LONG)) as LONG_PTR)) as HKEY);
#[inline]
pub unsafe fn IActiveScriptError_GetExceptionInfo(This: *mut IActiveScriptError, pexcepinfo: *mut EXCEPINFO) -> HRESULT { ((*(*This).lpVtbl).GetExceptionInfo.unwrap_unchecked())(This, pexcepinfo) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScriptError_GetSourcePosition(This: *mut IActiveScriptError, pdwSourceContext: *mut DWORD, pulLineNumber: *mut ULONG, plCharacterPosition: *mut LONG) -> HRESULT { ((*(*This).lpVtbl).GetSourcePosition.unwrap_unchecked())(This, pdwSourceContext, pulLineNumber, plCharacterPosition) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_AddNamedItem(This: *mut IActiveScript, pstrName: LPCOLESTR, dwFlags: DWORD) -> HRESULT { ((*(*This).lpVtbl).AddNamedItem.unwrap_unchecked())(This, pstrName, dwFlags) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_Close(This: *mut IActiveScript) -> HRESULT { ((*(*This).lpVtbl).Close.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_QueryInterface(This: *mut IActiveScript, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_Release(This: *mut IActiveScript) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_SetScriptSite(This: *mut IActiveScript, pass: *mut IActiveScriptSite) -> HRESULT { ((*(*This).lpVtbl).SetScriptSite.unwrap_unchecked())(This, pass) } // widl C wrapper
#[inline]
pub unsafe fn IActiveScript_SetScriptState(This: *mut IActiveScript, ss: SCRIPTSTATE) -> HRESULT { ((*(*This).lpVtbl).SetScriptState.unwrap_unchecked())(This, ss) } // widl C wrapper
#[inline]
pub unsafe fn IHost_AddRef(This: *mut IHost) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ITypeInfo_AddRef(This: *mut ITypeInfo) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ITypeInfo_GetIDsOfNames(This: *mut ITypeInfo, rgszNames: *mut LPOLESTR, cNames: UINT, pMemId: *mut MEMBERID) -> HRESULT { ((*(*This).lpVtbl).GetIDsOfNames.unwrap_unchecked())(This, rgszNames, cNames, pMemId) } // widl C wrapper
#[inline]
pub unsafe fn ITypeInfo_Invoke(This: *mut ITypeInfo, pvInstance: PVOID, memid: MEMBERID, wFlags: WORD, pDispParams: *mut DISPPARAMS, pVarResult: *mut VARIANT, pExcepInfo: *mut EXCEPINFO, puArgErr: *mut UINT) -> HRESULT { ((*(*This).lpVtbl).Invoke.unwrap_unchecked())(This, pvInstance, memid, wFlags, pDispParams, pVarResult, pExcepInfo, puArgErr) } // widl C wrapper
#[inline]
pub unsafe fn ITypeInfo_Release(This: *mut ITypeInfo) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn ITypeLib_GetTypeInfoOfGuid(This: *mut ITypeLib, guid: *const GUID, ppTinfo: *mut *mut ITypeInfo) -> HRESULT { ((*(*This).lpVtbl).GetTypeInfoOfGuid.unwrap_unchecked())(This, guid, ppTinfo) } // widl C wrapper
#[inline]
pub unsafe fn ITypeLib_Release(This: *mut ITypeLib) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_AddRef(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).AddRef.unwrap_unchecked())(This) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_QueryInterface(This: *mut IUnknown, riid: *const IID, ppvObject: *mut *mut ::core::ffi::c_void) -> HRESULT { ((*(*This).lpVtbl).QueryInterface.unwrap_unchecked())(This, riid, ppvObject) } // widl C wrapper
#[inline]
pub unsafe fn IUnknown_Release(This: *mut IUnknown) -> ULONG { ((*(*This).lpVtbl).Release.unwrap_unchecked())(This) } // widl C wrapper
pub type REFIID = *const IID;
pub const S_FALSE: HRESULT = ((1) as HRESULT);
pub const S_OK: HRESULT = ((0) as HRESULT);
pub const VARIANT_FALSE: VARIANT_BOOL = 0x0usize as VARIANT_BOOL;
pub const VARIANT_TRUE: VARIANT_BOOL = 0xffffusize as VARIANT_BOOL;
pub use crate::{V_BSTR, V_VT, max, va_end, va_start};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// base/applications/cmdutils/wscript/main.c:104: #define SVect_Delete(pV, pItem) ( (pV), (pItem) )
#[allow(unused_macros)]
macro_rules! SVect_Delete {
    ($pV:expr, $pItem:expr $(,)?) => {
        ({
            let _ = $pV;
            $pItem
        })
    };
}
#[allow(unused_imports)]
pub(crate) use SVect_Delete;
// ==== end port macros ====

// ==== port variadic: C-variadic entry points, trampolines in the module's rust_varargs_shim.c (generated -- `rustconv port integrate`) ====
extern "C" {
    pub fn print_resource(id: ::core::ffi::c_uint, ...);
}
// ==== end port variadic ====
