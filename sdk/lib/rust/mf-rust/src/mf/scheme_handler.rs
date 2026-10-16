// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7B11FD6A69AB: [u16; 25] = [
    0x0061u16, 0x0070, 0x0070, 0x006C, 0x0069, 0x0063, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x002F, 0x006F, 0x0063, 0x0074, 0x0065, 0x0074, 0x002D, 0x0073, 0x0074,
    0x0072, 0x0065, 0x0061, 0x006D, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(mfplat): debug channel is provided by the DPRINT1!/ERR!/WARN! macros in the crate.
// COBJMACROS: no Rust counterpart; the COM methods are called directly through the vtables.

// cdecl: the C typedef carries no WINAPI.
pub type create_stream_cb =
    Option<unsafe extern "C" fn(url: *const WCHAR, flags: DWORD, out: *mut *mut IMFByteStream) -> HRESULT>;

#[repr(C)]
pub struct scheme_handler_result {
    pub entry: list,
    pub result: *mut IMFAsyncResult,
    pub obj_type: MF_OBJECT_TYPE,
    pub object: *mut IUnknown,
}
pub type PSCHEME_HANDLER_RESULT = *mut scheme_handler_result;

#[repr(C)]
pub struct scheme_handler {
    pub IMFSchemeHandler_iface: IMFSchemeHandler,
    pub IMFAsyncCallback_iface: IMFAsyncCallback,
    pub refcount: LONG,
    pub resolver: *mut IMFSourceResolver,
    pub results: list,
    pub cs: CRITICAL_SECTION,
    pub create_stream: create_stream_cb,
}
pub type PSCHEME_HANDLER = *mut scheme_handler;

#[repr(C)]
pub struct create_object_context {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,

    pub props: *mut IPropertyStore,
    pub url: *mut WCHAR,
    pub flags: DWORD,
}
pub type PCREATE_OBJECT_CONTEXT = *mut create_object_context;

// static const IUnknownVtbl create_object_context_vtbl = {...};        -- COM vtable, skipped (vtbl_gen.py)
// static const IMFSchemeHandlerVtbl scheme_handler_vtbl = {...};      -- COM vtable, skipped (vtbl_gen.py)
// static const IMFAsyncCallbackVtbl scheme_handler_callback_vtbl = {...}; -- COM vtable, skipped (vtbl_gen.py)

unsafe extern "C" fn impl_from_IMFSchemeHandler(iface: *mut IMFSchemeHandler) -> *mut scheme_handler {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(scheme_handler, IMFSchemeHandler_iface)) as *mut scheme_handler;
}

unsafe extern "C" fn impl_from_IMFAsyncCallback(iface: *mut IMFAsyncCallback) -> *mut scheme_handler {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(scheme_handler, IMFAsyncCallback_iface))
        as *mut scheme_handler;
}

unsafe extern "system" fn scheme_handler_QueryInterface(iface: *mut IMFSchemeHandler, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSchemeHandler)) != 0 || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IMFSchemeHandler_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn scheme_handler_AddRef(iface: *mut IMFSchemeHandler) -> ULONG {
    let handler: *mut scheme_handler = impl_from_IMFSchemeHandler(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*handler).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << (__WINE_DBCL_TRACE as ::core::ffi::c_int)))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/scheme_handler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"scheme_handler_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            90 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            handler,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn scheme_handler_Release(iface: *mut IMFSchemeHandler) -> ULONG {
    let handler: *mut scheme_handler = impl_from_IMFSchemeHandler(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*handler).refcount) as *mut core::ffi::c_long,
    ) as ULONG;
    let mut result: *mut scheme_handler_result;
    let mut next: *mut scheme_handler_result;

    if refcount == 0 {
        result = ((*handler).results.next as *mut u8).offset(
            -(core::mem::offset_of!(scheme_handler_result, entry) as isize),
        ) as *mut scheme_handler_result;
        next = ((*result).entry.next as *mut u8).offset(
            -(core::mem::offset_of!(scheme_handler_result, entry) as isize),
        ) as *mut scheme_handler_result;
        while core::ptr::addr_of_mut!((*result).entry) != core::ptr::addr_of_mut!((*handler).results) {
            list_remove((core::ptr::addr_of_mut!((*result).entry)).cast());
            IMFAsyncResult_Release((*result).result);
            if !(*result).object.is_null() {
                IUnknown_Release((*result).object);
            }
            free(result as *mut core::ffi::c_void);
            result = next;
            next = ((*result).entry.next as *mut u8).offset(
                -(core::mem::offset_of!(scheme_handler_result, entry) as isize),
            ) as *mut scheme_handler_result;
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*handler).cs));
        if !(*handler).resolver.is_null() {
            IMFSourceResolver_Release((*handler).resolver);
        }
        free(handler as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut create_object_context {
    return CONTAINING_RECORD!(iface, create_object_context, IUnknown_iface);
}

unsafe extern "system" fn create_object_context_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn create_object_context_AddRef(iface: *mut IUnknown) -> ULONG {
    let context: *mut create_object_context = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*context).refcount)) as ULONG;

    return refcount;
}

unsafe extern "system" fn create_object_context_Release(iface: *mut IUnknown) -> ULONG {
    let context: *mut create_object_context = impl_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*context).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if !(*context).props.is_null() {
            IPropertyStore_Release((*context).props);
        }
        free((*context).url as *mut core::ffi::c_void);
        free(context as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn scheme_handler_BeginCreateObject(iface: *mut IMFSchemeHandler, url: *const WCHAR, flags: DWORD, props: *mut IPropertyStore, cancel_cookie: *mut *mut IUnknown, callback: *mut IMFAsyncCallback, state: *mut IUnknown) -> HRESULT {
    let handler: *mut scheme_handler = impl_from_IMFSchemeHandler(iface);
    let mut context: *mut create_object_context = core::ptr::null_mut();
    let mut caller: *mut IMFAsyncResult = core::ptr::null_mut();
    let mut item: *mut IMFAsyncResult = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %s, %#lx, %p, %p, %p, %p.\n", iface, debugstr_w(url), flags, props, cancel_cookie, callback, state);

    if !cancel_cookie.is_null() {
        *cancel_cookie = core::ptr::null_mut();
    }

    hr = MFCreateAsyncResult(core::ptr::null_mut(), callback, state, core::ptr::addr_of_mut!(caller));
    if hr < 0 {
        return hr;
    }

    context = malloc(core::mem::size_of::<create_object_context>() as usize) as *mut create_object_context;
    if context.is_null() {
        IMFAsyncResult_Release(caller);
        return E_OUTOFMEMORY;
    }

    (*context).IUnknown_iface.lpVtbl = core::ptr::addr_of!(create_object_context_vtbl);
    (*context).refcount = 1;
    (*context).props = props;
    if !(*context).props.is_null() {
        IPropertyStore_AddRef((*context).props);
    }
    (*context).flags = flags;
    (*context).url = wcsdup(url) as *mut WCHAR;
    if (*context).url.is_null() {
        IMFAsyncResult_Release(caller);
        IUnknown_Release(core::ptr::addr_of_mut!((*context).IUnknown_iface));
        return E_OUTOFMEMORY;
    }

    hr = MFCreateAsyncResult(core::ptr::addr_of_mut!((*context).IUnknown_iface), core::ptr::addr_of_mut!((*handler).IMFAsyncCallback_iface), caller as *mut IUnknown, core::ptr::addr_of_mut!(item));
    IUnknown_Release(core::ptr::addr_of_mut!((*context).IUnknown_iface));
    if hr >= 0 {
        hr = MFPutWorkItemEx(MFASYNC_CALLBACK_QUEUE_IO, item);
        if hr >= 0 {
            if !cancel_cookie.is_null() {
                *cancel_cookie = caller as *mut IUnknown;
                IUnknown_AddRef(*cancel_cookie);
            }
        }

        IMFAsyncResult_Release(item);
    }
    IMFAsyncResult_Release(caller);

    return hr;
}

unsafe extern "system" fn scheme_handler_EndCreateObject(
    mut iface: *mut IMFSchemeHandler,
    mut result: *mut IMFAsyncResult,
    mut obj_type: *mut MF_OBJECT_TYPE,
    mut object: *mut *mut IUnknown,
) -> HRESULT {
    let mut handler: *mut scheme_handler = impl_from_IMFSchemeHandler(iface);
    let mut found: *mut scheme_handler_result = core::ptr::null_mut::<scheme_handler_result>();
    let mut cur: *mut scheme_handler_result = core::ptr::null_mut::<scheme_handler_result>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/scheme_handler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"scheme_handler_EndCreateObject\0" as *const u8 as *const ::core::ffi::c_char,
            251 as ::core::ffi::c_int,
            b"%p, %p, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            result,
            obj_type,
            object,
        );
    }
    EnterCriticalSection(&raw mut (*handler).cs);
    cur = ((*handler).results.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*core::ptr::null_mut::<scheme_handler_result>()).entry
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut scheme_handler_result;
    while &raw mut (*cur).entry != &raw mut (*handler).results {
        if result == (*cur).result {
            list_remove((&raw mut (*cur).entry).cast());
            found = cur;
            break;
        } else {
            cur = ((*cur).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*core::ptr::null_mut::<scheme_handler_result>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut scheme_handler_result;
        }
    }
    LeaveCriticalSection(&raw mut (*handler).cs);
    if !found.is_null() {
        *obj_type = (*found).obj_type;
        *object = (*found).object;
        hr = IMFAsyncResult_GetStatus((*found).result);
        IMFAsyncResult_Release((*found).result);
        free(found as *mut ::core::ffi::c_void);
    } else {
        *obj_type = MF_OBJECT_INVALID;
        *object = core::ptr::null_mut::<IUnknown>();
        hr = MF_E_UNEXPECTED;
    }
    return hr;
}
unsafe extern "system" fn scheme_handler_CancelObjectCreation(
    mut iface: *mut IMFSchemeHandler,
    mut cancel_cookie: *mut IUnknown,
) -> HRESULT {
    let mut handler: *mut scheme_handler = impl_from_IMFSchemeHandler(iface);
    let mut found: *mut scheme_handler_result = ::core::ptr::null_mut::<scheme_handler_result>();
    let mut cur: *mut scheme_handler_result = ::core::ptr::null_mut::<scheme_handler_result>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/scheme_handler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"scheme_handler_CancelObjectCreation\0" as *const u8 as *const ::core::ffi::c_char,
            290 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            cancel_cookie,
        );
    }
    EnterCriticalSection(&raw mut (*handler).cs);
    cur = ((*handler).results.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<scheme_handler_result>()).entry
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut scheme_handler_result;
    while &raw mut (*cur).entry != &raw mut (*handler).results {
        if cancel_cookie == (*cur).result as *mut IUnknown {
            list_remove((&raw mut (*cur).entry).cast());
            found = cur;
            break;
        } else {
            cur = ((*cur).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<scheme_handler_result>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut scheme_handler_result;
        }
    }
    LeaveCriticalSection(&raw mut (*handler).cs);
    if !found.is_null() {
        IMFAsyncResult_Release((*found).result);
        if !(*found).object.is_null() {
            IUnknown_Release((*found).object);
        }
        free(found as *mut ::core::ffi::c_void);
    }
    return if !found.is_null() {
        S_OK
    } else {
        MF_E_UNEXPECTED
    };
}

unsafe extern "system" fn scheme_handler_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const IID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut ::core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn scheme_handler_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let handler: *mut scheme_handler = impl_from_IMFAsyncCallback(iface);
    return IMFSchemeHandler_AddRef(core::ptr::addr_of_mut!((*handler).IMFSchemeHandler_iface));
}

unsafe extern "system" fn scheme_handler_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let handler: *mut scheme_handler = impl_from_IMFAsyncCallback(iface);
    return IMFSchemeHandler_Release(core::ptr::addr_of_mut!((*handler).IMFSchemeHandler_iface));
}
unsafe extern "system" fn scheme_handler_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn scheme_handler_get_resolver(handler: *mut scheme_handler, resolver: *mut *mut IMFSourceResolver) -> HRESULT {
    let mut hr: HRESULT = 0;

    if (*handler).resolver.is_null() {
        let mut resolver: *mut IMFSourceResolver = core::ptr::null_mut();

        hr = MFCreateSourceResolver(core::ptr::addr_of_mut!(resolver));
        if hr < 0 {
            return hr;
        }

        if !InterlockedCompareExchangePointer(
            core::ptr::addr_of_mut!((*handler).resolver) as *mut *mut core::ffi::c_void,
            resolver as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            IMFSourceResolver_Release(resolver);
        }
    }

    *resolver = (*handler).resolver;
    IMFSourceResolver_AddRef(*resolver);

    return S_OK;
}

unsafe extern "system" fn scheme_handler_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let handler: *mut scheme_handler = impl_from_IMFAsyncCallback(iface);
    let mut handler_result: *mut scheme_handler_result = core::ptr::null_mut();
    let mut obj_type: MF_OBJECT_TYPE = MF_OBJECT_INVALID;
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut context_object: *mut IUnknown = core::ptr::null_mut();
    let mut context: *mut create_object_context = core::ptr::null_mut();
    let mut resolver: *mut IMFSourceResolver = core::ptr::null_mut();
    let mut caller: *mut IMFAsyncResult = core::ptr::null_mut();
    let mut stream: *mut IMFByteStream = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    caller = IMFAsyncResult_GetStateNoAddRef(result) as *mut IMFAsyncResult;

    if ({ hr = IMFAsyncResult_GetObject(result, core::ptr::addr_of_mut!(context_object)); hr }) < 0 {
        WARN!("Expected context set for callee result.\n");
        return hr;
    }

    context = impl_from_IUnknown(context_object);

    if {
        hr = (*handler).create_stream.unwrap_unchecked()(
            (*context).url,
            (*context).flags,
            core::ptr::addr_of_mut!(stream),
        );
        hr
    } >= 0 {
        if ((*context).flags & MF_RESOLUTION_MEDIASOURCE) != 0 {
            if {
                hr = scheme_handler_get_resolver(handler, core::ptr::addr_of_mut!(resolver));
                hr
            } >= 0 {
                hr = IMFSourceResolver_CreateObjectFromByteStream(
                    resolver,
                    stream,
                    (*context).url,
                    (*context).flags,
                    (*context).props,
                    core::ptr::addr_of_mut!(obj_type),
                    core::ptr::addr_of_mut!(object),
                );
                IMFSourceResolver_Release(resolver);
            }
            IMFByteStream_Release(stream);
        } else {
            object = stream as *mut IUnknown;
            obj_type = MF_OBJECT_BYTESTREAM;
        }
    }

    handler_result = malloc(core::mem::size_of::<scheme_handler_result>() as size_t)
        as *mut scheme_handler_result;
    if !handler_result.is_null() {
        (*handler_result).result = caller;
        IMFAsyncResult_AddRef((*handler_result).result);
        (*handler_result).obj_type = obj_type;
        (*handler_result).object = object;

        EnterCriticalSection(core::ptr::addr_of_mut!((*handler).cs));
        list_add_tail(
            (core::ptr::addr_of_mut!((*handler).results)).cast(),
            (core::ptr::addr_of_mut!((*handler_result).entry)).cast(),
        );
        LeaveCriticalSection(core::ptr::addr_of_mut!((*handler).cs));
    } else {
        if !object.is_null() {
            IUnknown_Release(object);
        }
        hr = E_OUTOFMEMORY;
    }

    IUnknown_Release(core::ptr::addr_of_mut!((*context).IUnknown_iface));

    IMFAsyncResult_SetStatus(caller, hr);
    MFInvokeCallback(caller);

    return S_OK;
}

unsafe extern "C" fn file_stream_create(mut url: *const WCHAR, flags: DWORD, out: *mut *mut IMFByteStream) -> HRESULT {
    let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut len: DWORD;

    len = ARRAY_SIZE!(path);
    if (PathCreateFromUrlW(url, core::ptr::addr_of_mut!(path) as LPWSTR, core::ptr::addr_of_mut!(len), 0)) >= 0 {
        url = core::ptr::addr_of_mut!(path) as *mut WCHAR;
    }

    return MFCreateFile(
        if (flags & MF_RESOLUTION_WRITE) != 0 { MF_ACCESSMODE_READWRITE } else { MF_ACCESSMODE_READ },
        (MF_OPENMODE_FAIL_IF_NOT_EXIST) as MF_FILE_OPENMODE,
        (MF_FILEFLAGS_NONE) as MF_FILE_FLAGS,
        url,
        out,
    );
}
#[no_mangle]
pub unsafe extern "C" fn file_scheme_handler_construct(riid: *const IID, mut obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut handler: *mut scheme_handler = ::core::ptr::null_mut::<scheme_handler>();
    let mut hr: HRESULT = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/scheme_handler.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"file_scheme_handler_construct\0" as *const u8 as *const ::core::ffi::c_char,
            475 as ::core::ffi::c_int,
            b"%s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_guid(riid as *const _GUID),
            obj,
        );
    }
    handler = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<scheme_handler>() as size_t) as _,
    ) as *mut scheme_handler;
    if handler.is_null() {
        return E_OUTOFMEMORY;
    }
    (*handler).IMFSchemeHandler_iface.lpVtbl = &raw const scheme_handler_vtbl;
    (*handler).IMFAsyncCallback_iface.lpVtbl = &raw const scheme_handler_callback_vtbl;
    (*handler).refcount = 1 as LONG;
    list_init((&raw mut (*handler).results).cast());
    InitializeCriticalSection(&raw mut (*handler).cs);
    (*handler).create_stream = Some(
        file_stream_create
            as unsafe extern "C" fn(*const WCHAR, DWORD, *mut *mut IMFByteStream) -> HRESULT,
    );
    hr = IMFSchemeHandler_QueryInterface(&raw mut (*handler).IMFSchemeHandler_iface, riid, obj);
    IMFSchemeHandler_Release(&raw mut (*handler).IMFSchemeHandler_iface);
    return hr;
}

unsafe extern "C" fn urlmon_stream_create(url: *const WCHAR, flags: DWORD, out: *mut *mut IMFByteStream) -> HRESULT {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut stream: *mut IStream = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (flags & MF_RESOLUTION_WRITE) != 0 {
        return E_INVALIDARG;
    }

    hr = URLOpenBlockingStreamW(
        core::ptr::null_mut(),
        url,
        core::ptr::addr_of_mut!(stream),
        0,
        core::ptr::null_mut(),
    );
    if hr < 0 {
        WARN!("Failed to open url %s, hr %#lx\n", debugstr_w(url), hr);
        return hr;
    }

    hr = MFCreateMFByteStreamOnStream(stream, out);
    IStream_Release(stream);
    if hr < 0 {
        return hr;
    }

    IMFByteStream_QueryInterface(
        *out,
        core::ptr::addr_of!(IID_IMFAttributes),
        core::ptr::addr_of_mut!(attributes) as *mut *mut core::ffi::c_void,
    );
    IMFAttributes_DeleteItem(attributes, core::ptr::addr_of!(MF_BYTESTREAM_ORIGIN_NAME));
    IMFAttributes_SetString(attributes, core::ptr::addr_of!(MF_BYTESTREAM_EFFECTIVE_URL), url);
    IMFAttributes_SetString(
        attributes,
        core::ptr::addr_of!(MF_BYTESTREAM_CONTENT_TYPE),
        RUSTCONV_WSTR_7B11FD6A69AB.as_ptr(),
    );
    IMFAttributes_Release(attributes);
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn urlmon_scheme_handler_construct(riid: *const IID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut handler: *mut scheme_handler = core::ptr::null_mut::<scheme_handler>();
    let mut hr: HRESULT = 0;

    handler = calloc((1 as usize) as _, (core::mem::size_of::<scheme_handler>() as usize) as _) as *mut scheme_handler;
    if handler.is_null() {
        return E_OUTOFMEMORY;
    }

    (*handler).IMFSchemeHandler_iface.lpVtbl = core::ptr::addr_of!(scheme_handler_vtbl);
    (*handler).IMFAsyncCallback_iface.lpVtbl = core::ptr::addr_of!(scheme_handler_callback_vtbl);
    (*handler).refcount = 1;
    list_init((core::ptr::addr_of_mut!((*handler).results)).cast());
    InitializeCriticalSection(core::ptr::addr_of_mut!((*handler).cs));
    (*handler).create_stream = Some(
        urlmon_stream_create
            as unsafe extern "C" fn(*const WCHAR, DWORD, *mut *mut IMFByteStream) -> HRESULT,
    );

    hr = IMFSchemeHandler_QueryInterface(core::ptr::addr_of_mut!((*handler).IMFSchemeHandler_iface), riid, obj);
    IMFSchemeHandler_Release(core::ptr::addr_of_mut!((*handler).IMFSchemeHandler_iface));

    return hr;
}

// COM vtable tables — generated from scheme_handler.c by vtbl_gen.py
/// scheme_handler.c:181 `static const IUnknownVtbl create_object_context_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static create_object_context_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(create_object_context_QueryInterface),
    AddRef: Some(create_object_context_AddRef),
    Release: Some(create_object_context_Release),
};

/// scheme_handler.c:317 `static const IMFSchemeHandlerVtbl scheme_handler_vtbl`  (slot order: rust:types_amd64.rs)
pub static scheme_handler_vtbl: IMFSchemeHandlerVtbl = IMFSchemeHandlerVtbl {
    QueryInterface: Some(scheme_handler_QueryInterface),
    AddRef: Some(scheme_handler_AddRef),
    Release: Some(scheme_handler_Release),
    BeginCreateObject: Some(scheme_handler_BeginCreateObject),
    EndCreateObject: Some(scheme_handler_EndCreateObject),
    CancelObjectCreation: Some(scheme_handler_CancelObjectCreation),
};

/// scheme_handler.c:448 `static const IMFAsyncCallbackVtbl scheme_handler_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static scheme_handler_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(scheme_handler_callback_QueryInterface),
    AddRef: Some(scheme_handler_callback_AddRef),
    Release: Some(scheme_handler_callback_Release),
    GetParameters: Some(scheme_handler_callback_GetParameters),
    Invoke: Some(scheme_handler_callback_Invoke),
};
