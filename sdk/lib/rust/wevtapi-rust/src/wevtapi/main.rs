use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut log_pathW: [WCHAR; 24] = unsafe {
    ::core::mem::transmute::<[u8; 48], [WCHAR; 24]>(
        *b"C\0:\0\\\0w\0i\0n\0d\0o\0w\0s\0\\\0t\0e\0m\0p\0\\\0e\0v\0t\0.\0l\0o\0g\0\0\0",
    )
};

// WINE_DEFAULT_DEBUG_CHANNEL(wevtapi); is a function-like macro; the debug channel is handled by the crate's DPRINT/WARN/ERR macros.

#[no_mangle]
pub unsafe extern "system" fn EvtOpenSession(
    login_class: EVT_LOGIN_CLASS,
    login: *mut core::ffi::c_void,
    timeout: DWORD,
    flags: DWORD,
) -> EVT_HANDLE {
    // FIXME("(%u %p %lu %lu) stub\n", login_class, login, timeout, flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn EvtOpenLog(
    session: EVT_HANDLE,
    path: *const WCHAR,
    flags: DWORD,
) -> EVT_HANDLE {
    // FIXME("(%p %s %lu) stub\n", session, debugstr_w(path), flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn EvtGetChannelConfigProperty(
    ChannelConfig: EVT_HANDLE,
    PropertyId: EVT_CHANNEL_CONFIG_PROPERTY_ID,
    Flags: DWORD,
    PropertyValueBufferSize: DWORD,
    PropertyValueBuffer: PEVT_VARIANT,
    PropertyValueBufferUsed: *mut DWORD,
) -> BOOL {
    match PropertyId as u32 {
        EvtChannelLoggingConfigLogFilePath => {
            *PropertyValueBufferUsed = (::core::mem::size_of::<[WCHAR; 24]>() as usize)
                .wrapping_add(::core::mem::size_of::<EVT_VARIANT>() as usize)
                as DWORD;

            if (PropertyValueBufferSize as usize)
                < (::core::mem::size_of::<[WCHAR; 24]>() as usize)
                    .wrapping_add(::core::mem::size_of::<EVT_VARIANT>() as usize)
                || PropertyValueBuffer.is_null()
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return (FALSE) as BOOL;
            }

            (*PropertyValueBuffer).__bindgen_anon_1.StringVal =
                PropertyValueBuffer.add(1 as usize) as LPWSTR;
            wcscpy(
                (*PropertyValueBuffer).__bindgen_anon_1.StringVal as *mut WCHAR,
                core::ptr::addr_of!(log_pathW) as *const WCHAR,
            );
            (*PropertyValueBuffer).Type = EvtVarTypeString as DWORD;
            return (TRUE) as BOOL;
        }
        _ => {
            SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
        }
    }

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EvtSetChannelConfigProperty(
    ChannelConfig: EVT_HANDLE,
    PropertyId: EVT_CHANNEL_CONFIG_PROPERTY_ID,
    Flags: DWORD,
    PropertyValue: PEVT_VARIANT,
) -> BOOL {
    // FIXME("(%p %i %lu %p) stub\n", ChannelConfig, PropertyId, Flags, PropertyValue);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn EvtSubscribe(
    Session: EVT_HANDLE,
    SignalEvent: HANDLE,
    ChannelPath: LPCWSTR,
    Query: LPCWSTR,
    Bookmark: EVT_HANDLE,
    context: PVOID,
    Callback: EVT_SUBSCRIBE_CALLBACK,
    Flags: DWORD,
) -> EVT_HANDLE {
    // FIXME("(%p %p %s %s %p %p %p %lu) stub\n", Session, SignalEvent, debugstr_w(ChannelPath), debugstr_w(Query), Bookmark, context, Callback, Flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn EvtOpenChannelEnum(
    session: EVT_HANDLE,
    flags: DWORD,
) -> EVT_HANDLE {
    // FIXME("(%p %lu) stub\n", session, flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn EvtNextChannelPath(
    channel_enum: EVT_HANDLE,
    buffer_len: DWORD,
    buffer: *mut WCHAR,
    used: *mut DWORD,
) -> BOOL {
    // FIXME("(%p %lu %p %p) stub\n", channel_enum, buffer_len, buffer, used);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn EvtOpenChannelConfig(Session: EVT_HANDLE, ChannelPath: LPCWSTR, Flags: DWORD) -> EVT_HANDLE {
    return (0xdeadbeef as usize) as EVT_HANDLE;
}

#[no_mangle]
pub unsafe extern "system" fn EvtQuery(
    session: EVT_HANDLE,
    path: *const WCHAR,
    query: *const WCHAR,
    flags: DWORD,
) -> EVT_HANDLE {
    // FIXME("(%p %s %s %lu) stub\n", session, debugstr_w(path), debugstr_w(query), flags);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn EvtSaveChannelConfig(channel: EVT_HANDLE, flags: DWORD) -> BOOL {
    // FIXME("(%p,%08lx) stub\n", channel, flags);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn EvtClose(handle: EVT_HANDLE) -> BOOL {
    // FIXME("(%p) stub\n", handle);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn EvtNext(result_set: EVT_HANDLE, size: DWORD, array: *mut EVT_HANDLE, timeout: DWORD, flags: DWORD, ret_count: *mut DWORD) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EvtExportLog(
    session: EVT_HANDLE,
    path: *const WCHAR,
    query: *const WCHAR,
    file: *const WCHAR,
    flags: DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn EvtCreateBookmark(bookmark_xml: *const WCHAR) -> EVT_HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn EvtCreateRenderContext(pathcount: DWORD, paths: *const *const WCHAR, flags: DWORD) -> EVT_HANDLE {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn EvtRender(
    context: EVT_HANDLE,
    fragment: EVT_HANDLE,
    flags: DWORD,
    buffer_size: DWORD,
    buffer: *mut core::ffi::c_void,
    buffer_used: *mut DWORD,
    property_count: *mut DWORD,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
