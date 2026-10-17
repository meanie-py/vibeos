//! rpc.rs — translation of dll/win32/ole32/rpc.c (the ORPC channel:
//! IRpcChannelBuffer client/server implementations, channel hooks, the RPC
//! dispatch glue, and the local-server named-pipe machinery).
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    unused_variables,
    unused_unsafe,
    unused_mut,
    unused_assignments,
    static_mut_refs,
    clippy::all
)]

use super::types::*;

// ---------------------------------------------------------------------------
// statics
// ---------------------------------------------------------------------------

/* we only use one function to dispatch calls for all methods - we use the
 * RPC_IF_OLE flag to tell the RPC runtime that this is the case */
static mut rpc_dispatch_table: [RPC_DISPATCH_FUNCTION; 1] = [dispatch_rpc]; /* (RO) */
static mut rpc_dispatch: RPC_DISPATCH_TABLE = RPC_DISPATCH_TABLE {
    DispatchTableCount: 1,
    DispatchTable: unsafe { core::ptr::addr_of_mut!(rpc_dispatch_table) as *mut RPC_DISPATCH_FUNCTION },
    Reserved: 0,
};

/* (CS csRegIf) */
static mut registered_interfaces: list = list {
    next: unsafe { core::ptr::addr_of_mut!(registered_interfaces) },
    prev: unsafe { core::ptr::addr_of_mut!(registered_interfaces) },
};
/* statically initialized like the C (see types::static_critsec re DebugInfo) */
static mut csRegIf: CRITICAL_SECTION = static_critsec();

/* (CS csChannelHook) */
static mut channel_hooks: list = list {
    next: unsafe { core::ptr::addr_of_mut!(channel_hooks) },
    prev: unsafe { core::ptr::addr_of_mut!(channel_hooks) },
};
static mut csChannelHook: CRITICAL_SECTION = static_critsec();

// "ncalrpc"
static wszRpcTransport: [u16; 8] = [
    'n' as u16, 'c' as u16, 'a' as u16, 'l' as u16, 'r' as u16, 'p' as u16, 'c' as u16, 0,
];

// ---------------------------------------------------------------------------
// local structs
// ---------------------------------------------------------------------------

#[repr(C)]
struct registered_if {
    entry: list,
    refs: DWORD,               /* ref count */
    If: RPC_SERVER_INTERFACE, /* interface registered with the RPC runtime */
}

/// get the pipe endpoint specified of the specified apartment
///
/// C: wsprintfW(endpoint, "\\pipe\\OLE_%08lx%08lx", (DWORD)(*oxid >> 32),
///    (DWORD)*oxid) — reimplemented locally (identical output bytes) to avoid
///    binding the varargs wsprintfW.
unsafe fn get_rpc_endpoint(endpoint: LPWSTR, oxid: *const OXID) {
    /* FIXME: should get endpoint from rpcss */
    const PREFIX: &[u16] = &[
        '\\' as u16, 'p' as u16, 'i' as u16, 'p' as u16, 'e' as u16, '\\' as u16, 'O' as u16,
        'L' as u16, 'E' as u16, '_' as u16,
    ];
    let mut p = endpoint;
    for &c in PREFIX {
        *p = c;
        p = p.add(1);
    }
    let hi = ((*oxid) >> 32) as u32;
    let lo = (*oxid) as u32;
    for v in [hi, lo] {
        let mut shift: i32 = 28;
        while shift >= 0 {
            let nibble = ((v >> shift) & 0xF) as u16;
            *p = if nibble < 10 {
                '0' as u16 + nibble
            } else {
                'a' as u16 + (nibble - 10)
            };
            p = p.add(1);
            shift -= 4;
        }
    }
    *p = 0;
}

#[repr(C)]
struct RpcChannelBuffer {
    IRpcChannelBuffer_iface: IRpcChannelBuffer,
    refs: LONG,
    dest_context: DWORD,             /* returned from GetDestCtx */
    dest_context_data: *mut c_void, /* returned from GetDestCtx */
}

#[repr(C)]
struct ClientRpcChannelBuffer {
    super_: RpcChannelBuffer,      /* superclass */
    bind: RPC_BINDING_HANDLE,      /* handle to the remote server */
    oxid: OXID,                    /* apartment in which the channel is valid */
    server_pid: DWORD,             /* id of server process */
    event: HANDLE,                 /* cached event handle */
    iid: IID,                      /* IID of the proxy this belongs to */
}

#[repr(C)]
struct message_state {
    binding_handle: RPC_BINDING_HANDLE,
    prefix_data_len: ULONG,
    channel_hook_info: SChannelHookCallInfo,
    bypass_rpcrt: BOOL,

    /* client only */
    target_hwnd: HWND,
    target_tid: DWORD,
    params: dispatch_params,
}

#[repr(C)]
struct WIRE_ORPC_EXTENT {
    conformance: ULONG, /* NDR */
    id: GUID,
    size: ULONG,
    /* [size_is((size+7)&~7)] */ data: [u8; 1],
}

#[repr(C)]
struct WIRE_ORPC_EXTENT_ARRAY {
    size: ULONG,
    reserved: ULONG,
    extent: [u8; 1],
}

#[repr(C)]
struct WIRE_ORPCTHIS {
    version: ULONG,
    flags: ULONG,
    reserved1: ULONG,
    cid: GUID,
    extensions: [u8; 1],
}

#[repr(C)]
struct WIRE_ORPCTHAT {
    flags: ULONG,
    extensions: [u8; 1],
}

const WIRE_ORPC_EXTENT_DATA: usize = core::mem::offset_of!(WIRE_ORPC_EXTENT, data);
const WIRE_ORPC_EXTENT_ARRAY_EXTENT: usize = core::mem::offset_of!(WIRE_ORPC_EXTENT_ARRAY, extent);
const WIRE_ORPCTHIS_EXTENSIONS: usize = core::mem::offset_of!(WIRE_ORPCTHIS, extensions);
const WIRE_ORPCTHAT_EXTENSIONS: usize = core::mem::offset_of!(WIRE_ORPCTHAT, extensions);
const SIZEOF_DWORD: usize = core::mem::size_of::<DWORD>();

#[repr(C)]
struct channel_hook_entry {
    entry: list,
    id: GUID,
    hook: *mut IChannelHook,
}

#[repr(C)]
struct channel_hook_buffer_data {
    id: GUID,
    extension_size: ULONG,
}

#[inline]
unsafe fn buf_add(p: *mut c_void, n: usize) -> *mut c_void {
    (p as *mut u8).add(n) as *mut c_void
}

// ---------------------------------------------------------------------------
// Channel Hook Functions
// ---------------------------------------------------------------------------

unsafe fn ChannelHooks_ClientGetSize(
    info: *mut SChannelHookCallInfo,
    data: *mut *mut channel_hook_buffer_data,
    hook_count: *mut c_uint,
    extension_count: *mut ULONG,
) -> ULONG {
    let mut total_size: ULONG = 0;
    let mut hook_index: usize = 0;

    *hook_count = 0;
    *extension_count = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        *hook_count += 1;
        cursor = (*cursor).next;
    }

    if *hook_count != 0 {
        *data = HeapAlloc(
            GetProcessHeap(),
            0,
            *hook_count as usize * core::mem::size_of::<channel_hook_buffer_data>(),
        ) as *mut channel_hook_buffer_data;
    } else {
        *data = core::ptr::null_mut();
    }

    cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut extension_size: ULONG = 0;

        ((*(*(*entry).hook).lpVtbl).ClientGetSize)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            &mut extension_size,
        );

        TRACE!("%s: extension_size = %u\n");

        extension_size = (extension_size + 7) & !7;
        (*(*data).add(hook_index)).id = (*entry).id;
        (*(*data).add(hook_index)).extension_size = extension_size;

        /* an extension is only put onto the wire if it has data to write */
        if extension_size != 0 {
            total_size += (WIRE_ORPC_EXTENT_DATA + extension_size as usize) as ULONG;
            *extension_count += 1;
        }

        hook_index += 1;
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    total_size
}

unsafe fn ChannelHooks_ClientFillBuffer(
    info: *mut SChannelHookCallInfo,
    mut buffer: *mut u8,
    data: *mut channel_hook_buffer_data,
    hook_count: c_uint,
) -> *mut u8 {
    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut extension_size: ULONG = 0;
        let wire_orpc_extent = buffer as *mut WIRE_ORPC_EXTENT;

        for i in 0..hook_count as usize {
            if IsEqualGUID(
                core::ptr::addr_of!((*entry).id),
                core::ptr::addr_of!((*data.add(i)).id),
            ) {
                extension_size = (*data.add(i)).extension_size;
            }
        }

        /* an extension is only put onto the wire if it has data to write */
        if extension_size == 0 {
            cursor = (*cursor).next;
            continue;
        }

        ((*(*(*entry).hook).lpVtbl).ClientFillBuffer)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            &mut extension_size,
            buffer.add(WIRE_ORPC_EXTENT_DATA) as *mut c_void,
        );

        TRACE!("%s: extension_size = %u\n");

        /* FIXME: set unused portion of wire_orpc_extent->data to 0? */

        (*wire_orpc_extent).conformance = (extension_size + 7) & !7;
        (*wire_orpc_extent).size = extension_size;
        (*wire_orpc_extent).id = (*entry).id;
        buffer = buffer.add(WIRE_ORPC_EXTENT_DATA + (*wire_orpc_extent).conformance as usize);

        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    buffer
}

unsafe fn ChannelHooks_ServerNotify(
    info: *mut SChannelHookCallInfo,
    lDataRep: DWORD,
    first_wire_orpc_extent: *mut WIRE_ORPC_EXTENT,
    extension_count: ULONG,
) {
    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut wire_orpc_extent: *mut WIRE_ORPC_EXTENT = first_wire_orpc_extent;
        let mut i: ULONG = 0;
        while i < extension_count {
            if IsEqualGUID(
                core::ptr::addr_of!((*entry).id),
                core::ptr::addr_of!((*wire_orpc_extent).id),
            ) {
                break;
            }
            wire_orpc_extent = (core::ptr::addr_of_mut!((*wire_orpc_extent).data) as *mut u8)
                .add((*wire_orpc_extent).conformance as usize)
                as *mut WIRE_ORPC_EXTENT;
            i += 1;
        }
        if i == extension_count {
            wire_orpc_extent = core::ptr::null_mut();
        }

        ((*(*(*entry).hook).lpVtbl).ServerNotify)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            if !wire_orpc_extent.is_null() {
                (*wire_orpc_extent).size
            } else {
                0
            },
            if !wire_orpc_extent.is_null() {
                core::ptr::addr_of_mut!((*wire_orpc_extent).data) as *mut c_void
            } else {
                core::ptr::null_mut()
            },
            lDataRep,
        );

        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
}

unsafe fn ChannelHooks_ServerGetSize(
    info: *mut SChannelHookCallInfo,
    data: *mut *mut channel_hook_buffer_data,
    hook_count: *mut c_uint,
    extension_count: *mut ULONG,
) -> ULONG {
    let mut total_size: ULONG = 0;
    let mut hook_index: usize = 0;

    *hook_count = 0;
    *extension_count = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        *hook_count += 1;
        cursor = (*cursor).next;
    }

    if *hook_count != 0 {
        *data = HeapAlloc(
            GetProcessHeap(),
            0,
            *hook_count as usize * core::mem::size_of::<channel_hook_buffer_data>(),
        ) as *mut channel_hook_buffer_data;
    } else {
        *data = core::ptr::null_mut();
    }

    cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut extension_size: ULONG = 0;

        ((*(*(*entry).hook).lpVtbl).ServerGetSize)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            S_OK,
            &mut extension_size,
        );

        TRACE!("%s: extension_size = %u\n");

        extension_size = (extension_size + 7) & !7;
        (*(*data).add(hook_index)).id = (*entry).id;
        (*(*data).add(hook_index)).extension_size = extension_size;

        /* an extension is only put onto the wire if it has data to write */
        if extension_size != 0 {
            total_size += (WIRE_ORPC_EXTENT_DATA + extension_size as usize) as ULONG;
            *extension_count += 1;
        }

        hook_index += 1;
        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    total_size
}

unsafe fn ChannelHooks_ServerFillBuffer(
    info: *mut SChannelHookCallInfo,
    mut buffer: *mut u8,
    data: *mut channel_hook_buffer_data,
    hook_count: c_uint,
) -> *mut u8 {
    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut extension_size: ULONG = 0;
        let wire_orpc_extent = buffer as *mut WIRE_ORPC_EXTENT;

        for i in 0..hook_count as usize {
            if IsEqualGUID(
                core::ptr::addr_of!((*entry).id),
                core::ptr::addr_of!((*data.add(i)).id),
            ) {
                extension_size = (*data.add(i)).extension_size;
            }
        }

        /* an extension is only put onto the wire if it has data to write */
        if extension_size == 0 {
            cursor = (*cursor).next;
            continue;
        }

        ((*(*(*entry).hook).lpVtbl).ServerFillBuffer)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            &mut extension_size,
            buffer.add(WIRE_ORPC_EXTENT_DATA) as *mut c_void,
            S_OK,
        );

        TRACE!("%s: extension_size = %u\n");

        /* FIXME: set unused portion of wire_orpc_extent->data to 0? */

        (*wire_orpc_extent).conformance = (extension_size + 7) & !7;
        (*wire_orpc_extent).size = extension_size;
        (*wire_orpc_extent).id = (*entry).id;
        buffer = buffer.add(WIRE_ORPC_EXTENT_DATA + (*wire_orpc_extent).conformance as usize);

        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    buffer
}

unsafe fn ChannelHooks_ClientNotify(
    info: *mut SChannelHookCallInfo,
    lDataRep: DWORD,
    first_wire_orpc_extent: *mut WIRE_ORPC_EXTENT,
    extension_count: ULONG,
    hrFault: HRESULT,
) {
    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        let mut wire_orpc_extent: *mut WIRE_ORPC_EXTENT = first_wire_orpc_extent;
        let mut i: ULONG = 0;
        while i < extension_count {
            if IsEqualGUID(
                core::ptr::addr_of!((*entry).id),
                core::ptr::addr_of!((*wire_orpc_extent).id),
            ) {
                break;
            }
            wire_orpc_extent = (core::ptr::addr_of_mut!((*wire_orpc_extent).data) as *mut u8)
                .add((*wire_orpc_extent).conformance as usize)
                as *mut WIRE_ORPC_EXTENT;
            i += 1;
        }
        if i == extension_count {
            wire_orpc_extent = core::ptr::null_mut();
        }

        ((*(*(*entry).hook).lpVtbl).ClientNotify)(
            (*entry).hook,
            core::ptr::addr_of!((*entry).id),
            core::ptr::addr_of!((*info).iid),
            if !wire_orpc_extent.is_null() {
                (*wire_orpc_extent).size
            } else {
                0
            },
            if !wire_orpc_extent.is_null() {
                core::ptr::addr_of_mut!((*wire_orpc_extent).data) as *mut c_void
            } else {
                core::ptr::null_mut()
            },
            lDataRep,
            hrFault,
        );

        cursor = (*cursor).next;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
}

/// (compobj_private.h: HRESULT RPC_RegisterChannelHook(REFGUID, IChannelHook*))
#[no_mangle]
pub unsafe extern "C" fn RPC_RegisterChannelHook(
    rguid: REFGUID,
    hook: *mut IChannelHook,
) -> HRESULT {
    TRACE!("(%s, %p)\n");

    let entry = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<channel_hook_entry>(),
    ) as *mut channel_hook_entry;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }

    (*entry).id = *rguid;
    (*entry).hook = hook;
    hook.AddRef();

    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
    list_add_tail(
        core::ptr::addr_of_mut!(channel_hooks),
        core::ptr::addr_of_mut!((*entry).entry),
    );
    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));

    S_OK
}

/// (compobj_private.h: void RPC_UnregisterAllChannelHooks(void))
#[no_mangle]
pub unsafe extern "C" fn RPC_UnregisterAllChannelHooks() {
    EnterCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
    /* LIST_FOR_EACH_ENTRY_SAFE */
    let head = core::ptr::addr_of_mut!(channel_hooks);
    let mut cursor = (*head).next;
    while cursor != head {
        let next = (*cursor).next;
        let entry = containing_record!(cursor, channel_hook_entry, entry);
        HeapFree(GetProcessHeap(), 0, entry as PVOID);
        cursor = next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
    DeleteCriticalSection(core::ptr::addr_of_mut!(csChannelHook));
    DeleteCriticalSection(core::ptr::addr_of_mut!(csRegIf));
}

// ---------------------------------------------------------------------------
// RPC Channel Buffer Functions
// ---------------------------------------------------------------------------

unsafe extern "system" fn RpcChannelBuffer_QueryInterface(
    iface: *mut IRpcChannelBuffer,
    riid: REFIID,
    ppv: *mut *mut c_void,
) -> HRESULT {
    *ppv = core::ptr::null_mut();
    if IsEqualIID(riid, &IID_IRpcChannelBuffer) || IsEqualIID(riid, &IID_IUnknown) {
        *ppv = iface as *mut c_void;
        iface.AddRef();
        return S_OK;
    }
    E_NOINTERFACE
}

unsafe extern "system" fn RpcChannelBuffer_AddRef(iface: LPRPCCHANNELBUFFER) -> ULONG {
    let This = iface as *mut RpcChannelBuffer;
    InterlockedIncrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG
}

unsafe extern "system" fn ServerRpcChannelBuffer_Release(iface: LPRPCCHANNELBUFFER) -> ULONG {
    let This = iface as *mut RpcChannelBuffer;

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).refs)) as ULONG;
    if ref_ != 0 {
        return ref_;
    }

    HeapFree(GetProcessHeap(), 0, This as PVOID);
    0
}

unsafe extern "system" fn ClientRpcChannelBuffer_Release(iface: LPRPCCHANNELBUFFER) -> ULONG {
    let This = iface as *mut ClientRpcChannelBuffer;

    let ref_ = InterlockedDecrement(core::ptr::addr_of_mut!((*This).super_.refs)) as ULONG;
    if ref_ != 0 {
        return ref_;
    }

    if !(*This).event.is_null() {
        CloseHandle((*This).event);
    }
    RpcBindingFree(core::ptr::addr_of_mut!((*This).bind));
    HeapFree(GetProcessHeap(), 0, This as PVOID);
    0
}

unsafe extern "system" fn ServerRpcChannelBuffer_GetBuffer(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
    riid: REFIID,
) -> HRESULT {
    let This = iface as *mut RpcChannelBuffer;
    let msg = olemsg as *mut RPC_MESSAGE;
    let status: RPC_STATUS;
    let mut channel_hook_data: *mut channel_hook_buffer_data = core::ptr::null_mut();
    let mut channel_hook_count: c_uint = 0;
    let mut extension_count: ULONG = 0;

    TRACE!("(%p)->(%p,%s)\n");

    let message_state = (*msg).Handle as *mut message_state;
    /* restore the binding handle and the real start of data */
    (*msg).Handle = (*message_state).binding_handle;
    (*msg).Buffer = (((*msg).Buffer as *mut u8)
        .sub((*message_state).prefix_data_len as usize)) as *mut c_void;

    let extensions_size = ChannelHooks_ServerGetSize(
        core::ptr::addr_of_mut!((*message_state).channel_hook_info),
        &mut channel_hook_data,
        &mut channel_hook_count,
        &mut extension_count,
    );

    (*msg).BufferLength += (WIRE_ORPCTHAT_EXTENSIONS + SIZEOF_DWORD) as c_uint;
    if extensions_size != 0 {
        (*msg).BufferLength += (WIRE_ORPC_EXTENT_ARRAY_EXTENT
            + 2 * SIZEOF_DWORD
            + extensions_size as usize) as c_uint;
        if extension_count & 1 != 0 {
            (*msg).BufferLength += WIRE_ORPC_EXTENT_DATA as c_uint;
        }
    }

    if (*message_state).bypass_rpcrt != FALSE {
        (*msg).Buffer = HeapAlloc(GetProcessHeap(), 0, (*msg).BufferLength as usize);
        if !(*msg).Buffer.is_null() {
            status = RPC_S_OK;
        } else {
            HeapFree(GetProcessHeap(), 0, channel_hook_data as PVOID);
            return E_OUTOFMEMORY;
        }
    } else {
        status = I_RpcGetBuffer(msg);
    }

    let orpcthat = (*msg).Buffer as *mut ORPCTHAT;
    (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPCTHAT_EXTENSIONS);

    (*orpcthat).flags = ORPCF_NULL /* FIXME? */;

    /* NDR representation of orpcthat->extensions */
    *((*msg).Buffer as *mut DWORD) = if extensions_size != 0 { 1 } else { 0 };
    (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

    if extensions_size != 0 {
        let orpc_extent_array = (*msg).Buffer as *mut WIRE_ORPC_EXTENT_ARRAY;
        (*orpc_extent_array).size = extension_count;
        (*orpc_extent_array).reserved = 0;
        (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPC_EXTENT_ARRAY_EXTENT);
        /* NDR representation of orpc_extent_array->extent */
        *((*msg).Buffer as *mut DWORD) = 1;
        (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);
        /* NDR representation of [size_is] attribute of orpc_extent_array->extent */
        *((*msg).Buffer as *mut DWORD) = (extension_count + 1) & !1;
        (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

        (*msg).Buffer = ChannelHooks_ServerFillBuffer(
            core::ptr::addr_of_mut!((*message_state).channel_hook_info),
            (*msg).Buffer as *mut u8,
            channel_hook_data,
            channel_hook_count,
        ) as *mut c_void;

        /* we must add a dummy extension if there is an odd extension
         * count to meet the contract specified by the size_is attribute */
        if extension_count & 1 != 0 {
            let wire_orpc_extent = (*msg).Buffer as *mut WIRE_ORPC_EXTENT;
            (*wire_orpc_extent).conformance = 0;
            (*wire_orpc_extent).id = GUID_NULL;
            (*wire_orpc_extent).size = 0;
            (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPC_EXTENT_DATA);
        }
    }

    HeapFree(GetProcessHeap(), 0, channel_hook_data as PVOID);

    /* store the prefixed data length so that we can restore the real buffer
     * later */
    (*message_state).prefix_data_len =
        ((*msg).Buffer as usize - orpcthat as usize) as ULONG;
    (*msg).BufferLength -= (*message_state).prefix_data_len;
    /* save away the message state again */
    (*msg).Handle = message_state as *mut c_void;

    TRACE!("-- %d\n");

    HRESULT_FROM_WIN32(status as u32)
}

unsafe fn ClientRpcChannelBuffer_GetEventHandle(This: *mut ClientRpcChannelBuffer) -> HANDLE {
    let mut event = InterlockedExchangePointer(core::ptr::addr_of_mut!((*This).event), core::ptr::null_mut());

    /* Note: must be auto-reset event so we can reuse it without a call
     * to ResetEvent */
    if event.is_null() {
        event = CreateEventW(core::ptr::null_mut(), FALSE, FALSE, core::ptr::null());
    }

    event
}

unsafe fn ClientRpcChannelBuffer_ReleaseEventHandle(This: *mut ClientRpcChannelBuffer, event: HANDLE) {
    if !InterlockedCompareExchangePointer(
        core::ptr::addr_of_mut!((*This).event),
        event,
        core::ptr::null_mut(),
    )
    .is_null()
    {
        /* already a handle cached in This */
        CloseHandle(event);
    }
}

unsafe extern "system" fn ClientRpcChannelBuffer_GetBuffer(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
    riid: REFIID,
) -> HRESULT {
    let This = iface as *mut ClientRpcChannelBuffer;
    let msg = olemsg as *mut RPC_MESSAGE;
    let mut status: RPC_STATUS;
    let mut channel_hook_data: *mut channel_hook_buffer_data = core::ptr::null_mut();
    let mut channel_hook_count: c_uint = 0;
    let mut extension_count: ULONG = 0;
    let mut ipid: IPID = GUID_NULL;
    let mut apt: *mut apartment = core::ptr::null_mut();

    TRACE!("(%p)->(%p,%s)\n");

    let cif = HeapAlloc(
        GetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<RPC_CLIENT_INTERFACE>(),
    ) as *mut RPC_CLIENT_INTERFACE;
    if cif.is_null() {
        return E_OUTOFMEMORY;
    }

    let message_state =
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<message_state>()) as *mut message_state;
    if message_state.is_null() {
        HeapFree(GetProcessHeap(), 0, cif as PVOID);
        return E_OUTOFMEMORY;
    }

    (*cif).Length = core::mem::size_of::<RPC_CLIENT_INTERFACE>() as c_uint;
    /* RPC interface ID = COM interface ID */
    (*cif).InterfaceId.SyntaxGUID = (*This).iid;
    /* COM objects always have a version of 0.0 */
    (*cif).InterfaceId.SyntaxVersion.MajorVersion = 0;
    (*cif).InterfaceId.SyntaxVersion.MinorVersion = 0;
    (*msg).Handle = (*This).bind;
    (*msg).RpcInterfaceInformation = cif as *mut c_void;

    (*message_state).prefix_data_len = 0;
    (*message_state).binding_handle = (*This).bind;

    (*message_state).channel_hook_info.iid = *riid;
    (*message_state).channel_hook_info.cbSize =
        core::mem::size_of::<SChannelHookCallInfo>() as DWORD;
    (*message_state).channel_hook_info.uCausality = COM_CurrentCausalityId();
    (*message_state).channel_hook_info.dwServerPid = (*This).server_pid;
    (*message_state).channel_hook_info.iMethod = (*msg).ProcNum & !RPC_FLAGS_VALID_BIT;
    (*message_state).channel_hook_info.pObject = core::ptr::null_mut(); /* only present on server-side */
    (*message_state).target_hwnd = core::ptr::null_mut();
    (*message_state).target_tid = 0;
    memset(
        core::ptr::addr_of_mut!((*message_state).params) as PVOID,
        0,
        core::mem::size_of::<dispatch_params>(),
    );

    let extensions_size = ChannelHooks_ClientGetSize(
        core::ptr::addr_of_mut!((*message_state).channel_hook_info),
        &mut channel_hook_data,
        &mut channel_hook_count,
        &mut extension_count,
    );

    (*msg).BufferLength += (WIRE_ORPCTHIS_EXTENSIONS + SIZEOF_DWORD) as c_uint;
    if extensions_size != 0 {
        (*msg).BufferLength += (WIRE_ORPC_EXTENT_ARRAY_EXTENT
            + 2 * SIZEOF_DWORD
            + extensions_size as usize) as c_uint;
        if extension_count & 1 != 0 {
            (*msg).BufferLength += WIRE_ORPC_EXTENT_DATA as c_uint;
        }
    }

    RpcBindingInqObject((*message_state).binding_handle, &mut ipid);
    let hr = ipid_get_dispatch_params(
        &ipid,
        &mut apt,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*message_state).params.stub),
        core::ptr::addr_of_mut!((*message_state).params.chan),
        core::ptr::addr_of_mut!((*message_state).params.iid),
        core::ptr::addr_of_mut!((*message_state).params.iface),
    );
    if hr == S_OK {
        /* stub, chan, iface and iid are unneeded in multi-threaded case as we go
         * via the RPC runtime */
        if (*apt).multi_threaded != FALSE {
            (*message_state).params.stub.Release();
            (*message_state).params.stub = core::ptr::null_mut();
            (*message_state).params.chan.Release();
            (*message_state).params.chan = core::ptr::null_mut();
            (*message_state).params.iface = core::ptr::null_mut();
        } else {
            (*message_state).params.bypass_rpcrt = TRUE;
            (*message_state).target_hwnd = apartment_getwindow(apt);
            (*message_state).target_tid = (*apt).tid;
            /* we assume later on that this being non-NULL is the indicator that
             * means call directly instead of going through RPC runtime */
            if (*message_state).target_hwnd.is_null() {
                ERR!("window for apartment %s is NULL\n");
            }
        }
    }
    if !apt.is_null() {
        apartment_release(apt);
    }
    (*message_state).params.handle = ClientRpcChannelBuffer_GetEventHandle(This);
    /* Note: message_state->params.msg is initialised in
     * ClientRpcChannelBuffer_SendReceive */

    /* shortcut the RPC runtime */
    if !(*message_state).target_hwnd.is_null() {
        (*msg).Buffer = HeapAlloc(GetProcessHeap(), 0, (*msg).BufferLength as usize);
        if !(*msg).Buffer.is_null() {
            status = RPC_S_OK;
        } else {
            status = ERROR_OUTOFMEMORY as RPC_STATUS;
        }
    } else {
        status = I_RpcGetBuffer(msg);
    }

    (*msg).Handle = message_state as *mut c_void;

    if status == RPC_S_OK {
        let orpcthis = (*msg).Buffer as *mut ORPCTHIS;
        (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPCTHIS_EXTENSIONS);

        (*orpcthis).version.MajorVersion = COM_MAJOR_VERSION;
        (*orpcthis).version.MinorVersion = COM_MINOR_VERSION;
        (*orpcthis).flags = if (*message_state).channel_hook_info.dwServerPid != 0 {
            ORPCF_LOCAL
        } else {
            ORPCF_NULL
        };
        (*orpcthis).reserved1 = 0;
        (*orpcthis).cid = (*message_state).channel_hook_info.uCausality;

        /* NDR representation of orpcthis->extensions */
        *((*msg).Buffer as *mut DWORD) = if extensions_size != 0 { 1 } else { 0 };
        (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

        if extensions_size != 0 {
            let orpc_extent_array = (*msg).Buffer as *mut WIRE_ORPC_EXTENT_ARRAY;
            (*orpc_extent_array).size = extension_count;
            (*orpc_extent_array).reserved = 0;
            (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPC_EXTENT_ARRAY_EXTENT);
            /* NDR representation of orpc_extent_array->extent */
            *((*msg).Buffer as *mut DWORD) = 1;
            (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);
            /* NDR representation of [size_is] attribute of orpc_extent_array->extent */
            *((*msg).Buffer as *mut DWORD) = (extension_count + 1) & !1;
            (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

            (*msg).Buffer = ChannelHooks_ClientFillBuffer(
                core::ptr::addr_of_mut!((*message_state).channel_hook_info),
                (*msg).Buffer as *mut u8,
                channel_hook_data,
                channel_hook_count,
            ) as *mut c_void;

            /* we must add a dummy extension if there is an odd extension
             * count to meet the contract specified by the size_is attribute */
            if extension_count & 1 != 0 {
                let wire_orpc_extent = (*msg).Buffer as *mut WIRE_ORPC_EXTENT;
                (*wire_orpc_extent).conformance = 0;
                (*wire_orpc_extent).id = GUID_NULL;
                (*wire_orpc_extent).size = 0;
                (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPC_EXTENT_DATA);
            }
        }

        /* store the prefixed data length so that we can restore the real buffer
         * pointer in ClientRpcChannelBuffer_SendReceive. */
        (*message_state).prefix_data_len =
            ((*msg).Buffer as usize - orpcthis as usize) as ULONG;
        (*msg).BufferLength -= (*message_state).prefix_data_len;
    }

    HeapFree(GetProcessHeap(), 0, channel_hook_data as PVOID);

    TRACE!("-- %d\n");

    HRESULT_FROM_WIN32(status as u32)
}

unsafe extern "system" fn ServerRpcChannelBuffer_SendReceive(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
    pstatus: *mut ULONG,
) -> HRESULT {
    FIXME!("stub\n");
    E_NOTIMPL
}

/// this thread runs an outgoing RPC
unsafe extern "system" fn rpc_sendreceive_thread(param: LPVOID) -> DWORD {
    let data = param as *mut dispatch_params;

    /* Note: I_RpcSendReceive doesn't raise exceptions like the higher-level
     * RPC functions do */
    (*data).status = I_RpcSendReceive((*data).msg as *mut RPC_MESSAGE);

    TRACE!("completed with status 0x%x\n");

    SetEvent((*data).handle);

    0
}

#[inline]
unsafe fn ClientRpcChannelBuffer_IsCorrectApartment(
    This: *mut ClientRpcChannelBuffer,
    apt: *mut apartment,
) -> HRESULT {
    let mut oxid: OXID = 0;
    if apt.is_null() {
        return S_FALSE;
    }
    if apartment_getoxid(apt, &mut oxid) != S_OK {
        return S_FALSE;
    }
    if (*This).oxid != oxid {
        return S_FALSE;
    }
    S_OK
}

unsafe extern "system" fn ClientRpcChannelBuffer_SendReceive(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
    pstatus: *mut ULONG,
) -> HRESULT {
    let This = iface as *mut ClientRpcChannelBuffer;
    let mut hr: HRESULT;
    let msg = olemsg as *mut RPC_MESSAGE;
    let status: RPC_STATUS;
    let mut index: DWORD = 0;
    let mut orpcthat: ORPCTHAT = core::mem::zeroed();
    let mut orpc_ext_array: ORPC_EXTENT_ARRAY = core::mem::zeroed();
    let mut first_wire_orpc_extent: *mut WIRE_ORPC_EXTENT = core::ptr::null_mut();
    let mut hrFault: HRESULT = S_OK;
    let apt = apartment_get_current_or_mta();

    TRACE!("(%p) iMethod=%d\n");

    hr = ClientRpcChannelBuffer_IsCorrectApartment(This, apt);
    if hr != S_OK {
        ERR!("called from wrong apartment, should have been 0x%s\n");
        if !apt.is_null() {
            apartment_release(apt);
        }
        return RPC_E_WRONG_THREAD;
    }
    /* This situation should be impossible in multi-threaded apartments,
     * because the calling thread isn't re-enterable.
     * Note: doing a COM call during the processing of a sent message is
     * only disallowed if a client call is already being waited for
     * completion */
    if (*apt).multi_threaded == FALSE
        && (*COM_CurrentInfo()).pending_call_count_client != 0
        && InSendMessage() != FALSE
    {
        ERR!("can't make an outgoing COM call in response to a sent message\n");
        apartment_release(apt);
        return RPC_E_CANTCALLOUT_ININPUTSYNCCALL;
    }

    let message_state = (*msg).Handle as *mut message_state;
    /* restore the binding handle and the real start of data */
    (*msg).Handle = (*message_state).binding_handle;
    (*msg).Buffer = (((*msg).Buffer as *mut u8)
        .sub((*message_state).prefix_data_len as usize)) as *mut c_void;
    (*msg).BufferLength += (*message_state).prefix_data_len;

    /* Note: this is an optimization in the Microsoft OLE runtime that we need
     * to copy, as shown by the test_no_couninitialize_client test. without
     * short-circuiting the RPC runtime in the case below, the test will
     * deadlock on the loader lock due to the RPC runtime needing to create
     * a thread to process the RPC when this function is called indirectly
     * from DllMain */

    (*message_state).params.msg = olemsg;
    if (*message_state).params.bypass_rpcrt != FALSE {
        TRACE!("Calling apartment thread 0x%08x...\n");

        (*msg).ProcNum &= !RPC_FLAGS_VALID_BIT;

        if PostMessageW(
            (*message_state).target_hwnd,
            DM_EXECUTERPC,
            0,
            core::ptr::addr_of_mut!((*message_state).params) as LPARAM,
        ) == FALSE
        {
            ERR!("PostMessage failed with error %u\n");

            /* Note: message_state->params.iface doesn't have a reference and
             * so doesn't need to be released */

            hr = HRESULT_FROM_WIN32(GetLastError());
        }
    } else {
        /* we use a separate thread here because we need to be able to
         * pump the message loop in the application thread: if we do not,
         * any windows created by this thread will hang and RPCs that try
         * and re-enter this STA from an incoming server thread will
         * deadlock. InstallShield is an example of that.
         */
        if QueueUserWorkItem(
            rpc_sendreceive_thread,
            core::ptr::addr_of_mut!((*message_state).params) as PVOID,
            WT_EXECUTEDEFAULT,
        ) == FALSE
        {
            ERR!("QueueUserWorkItem failed with error %u\n");
            hr = E_UNEXPECTED;
        } else {
            hr = S_OK;
        }
    }

    if hr == S_OK {
        if WaitForSingleObject((*message_state).params.handle, 0) != 0 {
            (*COM_CurrentInfo()).pending_call_count_client += 1;
            hr = CoWaitForMultipleHandles(
                0,
                INFINITE,
                1,
                core::ptr::addr_of!((*message_state).params.handle),
                &mut index,
            );
            (*COM_CurrentInfo()).pending_call_count_client -= 1;
        }
    }
    ClientRpcChannelBuffer_ReleaseEventHandle(This, (*message_state).params.handle);

    /* for WM shortcut, faults are returned in params->hr */
    if hr == S_OK {
        hrFault = (*message_state).params.hr;
    }

    status = (*message_state).params.status;

    orpcthat.flags = ORPCF_NULL;
    orpcthat.extensions = core::ptr::null_mut();

    TRACE!("RPC call status: 0x%x\n");
    if status != RPC_S_OK {
        hr = HRESULT_FROM_WIN32(status as u32);
    }

    TRACE!("hrFault = 0x%08x\n");

    /* FIXME: this condition should be
     * "hr == S_OK && (!hrFault || msg->BufferLength > FIELD_OFFSET(ORPCTHAT, extensions) + 4)"
     * but we don't currently reset the message length for PostMessage
     * dispatched calls */
    if hr == S_OK && hrFault == S_OK {
        let original_buffer = (*msg).Buffer;

        /* handle ORPCTHAT and client extensions */

        let hr2 = unmarshal_ORPCTHAT(msg, &mut orpcthat, &mut orpc_ext_array, &mut first_wire_orpc_extent);
        if FAILED(hr2) {
            hr = hr2;
        }

        (*message_state).prefix_data_len =
            ((*msg).Buffer as usize - original_buffer as usize) as ULONG;
        (*msg).BufferLength -= (*message_state).prefix_data_len;
    } else {
        (*message_state).prefix_data_len = 0;
    }

    if hr == S_OK {
        ChannelHooks_ClientNotify(
            core::ptr::addr_of_mut!((*message_state).channel_hook_info),
            (*msg).DataRepresentation,
            first_wire_orpc_extent,
            if !orpcthat.extensions.is_null() && !first_wire_orpc_extent.is_null() {
                (*orpcthat.extensions).size
            } else {
                0
            },
            hrFault,
        );
    }

    /* save away the message state again */
    (*msg).Handle = message_state as *mut c_void;

    if !pstatus.is_null() {
        *pstatus = status as ULONG;
    }

    if hr == S_OK {
        hr = hrFault;
    }

    TRACE!("-- 0x%08x\n");

    apartment_release(apt);
    hr
}

unsafe extern "system" fn ServerRpcChannelBuffer_FreeBuffer(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
) -> HRESULT {
    let msg = olemsg as *mut RPC_MESSAGE;
    let status: RPC_STATUS;

    TRACE!("(%p)\n");

    let message_state = (*msg).Handle as *mut message_state;
    /* restore the binding handle and the real start of data */
    (*msg).Handle = (*message_state).binding_handle;
    (*msg).Buffer = (((*msg).Buffer as *mut u8)
        .sub((*message_state).prefix_data_len as usize)) as *mut c_void;
    (*msg).BufferLength += (*message_state).prefix_data_len;
    (*message_state).prefix_data_len = 0;

    if (*message_state).bypass_rpcrt != FALSE {
        HeapFree(GetProcessHeap(), 0, (*msg).Buffer);
        status = RPC_S_OK;
    } else {
        status = I_RpcFreeBuffer(msg);
    }

    (*msg).Handle = message_state as *mut c_void;

    TRACE!("-- %d\n");

    HRESULT_FROM_WIN32(status as u32)
}

unsafe extern "system" fn ClientRpcChannelBuffer_FreeBuffer(
    iface: LPRPCCHANNELBUFFER,
    olemsg: *mut RPCOLEMESSAGE,
) -> HRESULT {
    let msg = olemsg as *mut RPC_MESSAGE;
    let status: RPC_STATUS;

    TRACE!("(%p)\n");

    let message_state = (*msg).Handle as *mut message_state;
    /* restore the binding handle and the real start of data */
    (*msg).Handle = (*message_state).binding_handle;
    (*msg).Buffer = (((*msg).Buffer as *mut u8)
        .sub((*message_state).prefix_data_len as usize)) as *mut c_void;
    (*msg).BufferLength += (*message_state).prefix_data_len;

    if (*message_state).params.bypass_rpcrt != FALSE {
        HeapFree(GetProcessHeap(), 0, (*msg).Buffer);
        status = RPC_S_OK;
    } else {
        status = I_RpcFreeBuffer(msg);
    }

    HeapFree(GetProcessHeap(), 0, (*msg).RpcInterfaceInformation);
    (*msg).RpcInterfaceInformation = core::ptr::null_mut();

    if !(*message_state).params.stub.is_null() {
        (*message_state).params.stub.Release();
    }
    if !(*message_state).params.chan.is_null() {
        (*message_state).params.chan.Release();
    }
    HeapFree(GetProcessHeap(), 0, message_state as PVOID);

    TRACE!("-- %d\n");

    HRESULT_FROM_WIN32(status as u32)
}

unsafe extern "system" fn ClientRpcChannelBuffer_GetDestCtx(
    iface: LPRPCCHANNELBUFFER,
    pdwDestContext: *mut DWORD,
    ppvDestContext: *mut *mut c_void,
) -> HRESULT {
    let This = iface as *mut ClientRpcChannelBuffer;

    TRACE!("(%p,%p)\n");

    *pdwDestContext = (*This).super_.dest_context;
    *ppvDestContext = (*This).super_.dest_context_data;

    S_OK
}

unsafe extern "system" fn ServerRpcChannelBuffer_GetDestCtx(
    iface: LPRPCCHANNELBUFFER,
    dest_context: *mut DWORD,
    dest_context_data: *mut *mut c_void,
) -> HRESULT {
    let This = iface as *mut RpcChannelBuffer;

    TRACE!("(%p,%p)\n");

    *dest_context = (*This).dest_context;
    *dest_context_data = (*This).dest_context_data;
    S_OK
}

unsafe extern "system" fn RpcChannelBuffer_IsConnected(iface: LPRPCCHANNELBUFFER) -> HRESULT {
    TRACE!("()\n");
    /* native does nothing too */
    S_OK
}

static ClientRpcChannelBufferVtbl: IRpcChannelBufferVtbl = IRpcChannelBufferVtbl {
    QueryInterface: RpcChannelBuffer_QueryInterface,
    AddRef: RpcChannelBuffer_AddRef,
    Release: ClientRpcChannelBuffer_Release,
    GetBuffer: ClientRpcChannelBuffer_GetBuffer,
    SendReceive: ClientRpcChannelBuffer_SendReceive,
    FreeBuffer: ClientRpcChannelBuffer_FreeBuffer,
    GetDestCtx: ClientRpcChannelBuffer_GetDestCtx,
    IsConnected: RpcChannelBuffer_IsConnected,
};

static ServerRpcChannelBufferVtbl: IRpcChannelBufferVtbl = IRpcChannelBufferVtbl {
    QueryInterface: RpcChannelBuffer_QueryInterface,
    AddRef: RpcChannelBuffer_AddRef,
    Release: ServerRpcChannelBuffer_Release,
    GetBuffer: ServerRpcChannelBuffer_GetBuffer,
    SendReceive: ServerRpcChannelBuffer_SendReceive,
    FreeBuffer: ServerRpcChannelBuffer_FreeBuffer,
    GetDestCtx: ServerRpcChannelBuffer_GetDestCtx,
    IsConnected: RpcChannelBuffer_IsConnected,
};

/// returns a channel buffer for proxies
/// (compobj_private.h: HRESULT RPC_CreateClientChannel(...))
#[no_mangle]
pub unsafe extern "C" fn RPC_CreateClientChannel(
    oxid: *const OXID,
    ipid: *const IPID,
    oxid_info: *const OXID_INFO,
    iid: *const IID,
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    chan: *mut *mut IRpcChannelBuffer,
    apt: *mut apartment,
) -> HRESULT {
    let mut endpoint: [WCHAR; 200] = [0; 200];
    let mut bind: RPC_BINDING_HANDLE = core::ptr::null_mut();
    let mut status: RPC_STATUS;
    let mut string_binding: LPWSTR = core::ptr::null_mut();

    /* FIXME: get the endpoint from oxid_info->psa instead */
    get_rpc_endpoint(endpoint.as_mut_ptr(), oxid);

    TRACE!("proxy pipe: connecting to endpoint: %s\n");

    status = RpcStringBindingComposeW(
        core::ptr::null(),
        wszRpcTransport.as_ptr(),
        core::ptr::null(),
        endpoint.as_ptr(),
        core::ptr::null(),
        &mut string_binding,
    );

    if status == RPC_S_OK {
        status = RpcBindingFromStringBindingW(string_binding, &mut bind);

        if status == RPC_S_OK {
            let mut ipid2: IPID = *ipid; /* why can't RpcBindingSetObject take a const? */
            status = RpcBindingSetObject(bind, &mut ipid2);
            if status != RPC_S_OK {
                RpcBindingFree(&mut bind);
            }
        }

        RpcStringFreeW(&mut string_binding);
    }

    if status != RPC_S_OK {
        ERR!("Couldn't get binding for endpoint %s, status = %d\n");
        return HRESULT_FROM_WIN32(status as u32);
    }

    let This = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<ClientRpcChannelBuffer>(),
    ) as *mut ClientRpcChannelBuffer;
    if This.is_null() {
        RpcBindingFree(&mut bind);
        return E_OUTOFMEMORY;
    }

    (*This).super_.IRpcChannelBuffer_iface.lpVtbl = &ClientRpcChannelBufferVtbl;
    (*This).super_.refs = 1;
    (*This).super_.dest_context = dest_context;
    (*This).super_.dest_context_data = dest_context_data;
    (*This).bind = bind;
    apartment_getoxid(apt, core::ptr::addr_of_mut!((*This).oxid));
    (*This).server_pid = (*oxid_info).dwPid;
    (*This).event = core::ptr::null_mut();
    (*This).iid = *iid;

    *chan = core::ptr::addr_of_mut!((*This).super_.IRpcChannelBuffer_iface);

    S_OK
}

/// (compobj_private.h: HRESULT RPC_CreateServerChannel(...))
#[no_mangle]
pub unsafe extern "C" fn RPC_CreateServerChannel(
    dest_context: DWORD,
    dest_context_data: *mut c_void,
    chan: *mut *mut IRpcChannelBuffer,
) -> HRESULT {
    let This = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<RpcChannelBuffer>())
        as *mut RpcChannelBuffer;
    if This.is_null() {
        return E_OUTOFMEMORY;
    }

    (*This).IRpcChannelBuffer_iface.lpVtbl = &ServerRpcChannelBufferVtbl;
    (*This).refs = 1;
    (*This).dest_context = dest_context;
    (*This).dest_context_data = dest_context_data;

    *chan = core::ptr::addr_of_mut!((*This).IRpcChannelBuffer_iface);

    S_OK
}

/// unmarshals ORPC_EXTENT_ARRAY according to NDR rules, but doesn't allocate
/// any memory
unsafe fn unmarshal_ORPC_EXTENT_ARRAY(
    msg: *mut RPC_MESSAGE,
    end: *const u8,
    extensions: *mut ORPC_EXTENT_ARRAY,
    first_wire_orpc_extent: *mut *mut WIRE_ORPC_EXTENT,
) -> HRESULT {
    memcpy(
        extensions as PVOID,
        (*msg).Buffer as *const c_void,
        WIRE_ORPC_EXTENT_ARRAY_EXTENT,
    );
    (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPC_EXTENT_ARRAY_EXTENT);

    if ((*msg).Buffer as *const u8).add(2 * SIZEOF_DWORD) > end {
        return RPC_E_INVALID_HEADER;
    }

    let pointer_id: DWORD = *((*msg).Buffer as *const DWORD);
    (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);
    (*extensions).extent = core::ptr::null_mut();

    if pointer_id != 0 {
        /* conformance */
        if *((*msg).Buffer as *const DWORD) != (((*extensions).size + 1) & !1) {
            return RPC_S_INVALID_BOUND;
        }

        (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

        /* arbitrary limit for security (don't know what native does) */
        if (*extensions).size > 256 {
            ERR!("too many extensions: %d\n");
            return RPC_S_INVALID_BOUND;
        }

        let mut wire_orpc_extent = (*msg).Buffer as *mut WIRE_ORPC_EXTENT;
        *first_wire_orpc_extent = wire_orpc_extent;
        for _i in 0..(((*extensions).size + 1) & !1) {
            if (core::ptr::addr_of!((*wire_orpc_extent).data) as *const u8) > end {
                return RPC_S_INVALID_BOUND;
            }
            if (*wire_orpc_extent).conformance != (((*wire_orpc_extent).size + 7) & !7) {
                return RPC_S_INVALID_BOUND;
            }
            if (core::ptr::addr_of!((*wire_orpc_extent).data) as *const u8)
                .add((*wire_orpc_extent).conformance as usize)
                > end
            {
                return RPC_S_INVALID_BOUND;
            }
            TRACE!("size %u, guid %s\n");
            wire_orpc_extent = (core::ptr::addr_of_mut!((*wire_orpc_extent).data) as *mut u8)
                .add((*wire_orpc_extent).conformance as usize)
                as *mut WIRE_ORPC_EXTENT;
        }
        (*msg).Buffer = wire_orpc_extent as *mut c_void;
    }

    S_OK
}

/// unmarshals ORPCTHIS according to NDR rules, but doesn't allocate any memory
unsafe fn unmarshal_ORPCTHIS(
    msg: *mut RPC_MESSAGE,
    orpcthis: *mut ORPCTHIS,
    orpc_ext_array: *mut ORPC_EXTENT_ARRAY,
    first_wire_orpc_extent: *mut *mut WIRE_ORPC_EXTENT,
) -> HRESULT {
    let end: *const u8 = ((*msg).Buffer as *const u8).add((*msg).BufferLength as usize);

    *first_wire_orpc_extent = core::ptr::null_mut();

    if ((*msg).BufferLength as usize) < WIRE_ORPCTHIS_EXTENSIONS + SIZEOF_DWORD {
        ERR!("invalid buffer length\n");
        return RPC_E_INVALID_HEADER;
    }

    memcpy(
        orpcthis as PVOID,
        (*msg).Buffer as *const c_void,
        WIRE_ORPCTHIS_EXTENSIONS,
    );
    (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPCTHIS_EXTENSIONS);

    if ((*msg).Buffer as *const u8).add(SIZEOF_DWORD) > end { // nolint: magic-offset (NDR wire bounds check: advance u8 buffer by sizeof(DWORD)=4 bytes vs end)
        return RPC_E_INVALID_HEADER;
    }

    if *((*msg).Buffer as *const DWORD) != 0 {
        (*orpcthis).extensions = orpc_ext_array;
    } else {
        (*orpcthis).extensions = core::ptr::null_mut();
    }

    (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

    if !(*orpcthis).extensions.is_null() {
        let hr = unmarshal_ORPC_EXTENT_ARRAY(msg, end, orpc_ext_array, first_wire_orpc_extent);
        if FAILED(hr) {
            return hr;
        }
    }

    if ((*orpcthis).version.MajorVersion != COM_MAJOR_VERSION)
        || ((*orpcthis).version.MinorVersion > COM_MINOR_VERSION)
    {
        ERR!("COM version {%d, %d} not supported\n");
        return RPC_E_VERSION_MISMATCH;
    }

    if (*orpcthis).flags & !(ORPCF_LOCAL | ORPCF_RESERVED1 | ORPCF_RESERVED2 | ORPCF_RESERVED3 | ORPCF_RESERVED4)
        != 0
    {
        ERR!("invalid flags 0x%x\n");
        return RPC_E_INVALID_HEADER;
    }

    S_OK
}

unsafe fn unmarshal_ORPCTHAT(
    msg: *mut RPC_MESSAGE,
    orpcthat: *mut ORPCTHAT,
    orpc_ext_array: *mut ORPC_EXTENT_ARRAY,
    first_wire_orpc_extent: *mut *mut WIRE_ORPC_EXTENT,
) -> HRESULT {
    let end: *const u8 = ((*msg).Buffer as *const u8).add((*msg).BufferLength as usize);

    *first_wire_orpc_extent = core::ptr::null_mut();

    if ((*msg).BufferLength as usize) < WIRE_ORPCTHAT_EXTENSIONS + SIZEOF_DWORD {
        ERR!("invalid buffer length\n");
        return RPC_E_INVALID_HEADER;
    }

    memcpy(
        orpcthat as PVOID,
        (*msg).Buffer as *const c_void,
        WIRE_ORPCTHAT_EXTENSIONS,
    );
    (*msg).Buffer = buf_add((*msg).Buffer, WIRE_ORPCTHAT_EXTENSIONS);

    if ((*msg).Buffer as *const u8).add(SIZEOF_DWORD) > end { // nolint: magic-offset (NDR wire bounds check: advance u8 buffer by sizeof(DWORD)=4 bytes vs end)
        return RPC_E_INVALID_HEADER;
    }

    if *((*msg).Buffer as *const DWORD) != 0 {
        (*orpcthat).extensions = orpc_ext_array;
    } else {
        (*orpcthat).extensions = core::ptr::null_mut();
    }

    (*msg).Buffer = buf_add((*msg).Buffer, SIZEOF_DWORD);

    if !(*orpcthat).extensions.is_null() {
        let hr = unmarshal_ORPC_EXTENT_ARRAY(msg, end, orpc_ext_array, first_wire_orpc_extent);
        if FAILED(hr) {
            return hr;
        }
    }

    if (*orpcthat).flags & !(ORPCF_LOCAL | ORPCF_RESERVED1 | ORPCF_RESERVED2 | ORPCF_RESERVED3 | ORPCF_RESERVED4)
        != 0
    {
        ERR!("invalid flags 0x%x\n");
        return RPC_E_INVALID_HEADER;
    }

    S_OK
}

/// (compobj_private.h: void RPC_ExecuteCall(struct dispatch_params*)) —
/// called by the apartment window procedure (Lane A) on DM_EXECUTERPC.
#[no_mangle]
pub unsafe extern "C" fn RPC_ExecuteCall(params: *mut dispatch_params) {
    let mut message_state_p: *mut message_state = core::ptr::null_mut();
    let msg = (*params).msg as *mut RPC_MESSAGE;
    let original_buffer: *mut c_void = (*msg).Buffer;
    let mut orpcthis: ORPCTHIS = core::mem::zeroed();
    let mut orpc_ext_array: ORPC_EXTENT_ARRAY = core::mem::zeroed();
    let mut first_wire_orpc_extent: *mut WIRE_ORPC_EXTENT = core::ptr::null_mut();

    'exit: {
        /* handle ORPCTHIS and server extensions */

        (*params).hr = unmarshal_ORPCTHIS(msg, &mut orpcthis, &mut orpc_ext_array, &mut first_wire_orpc_extent);
        if (*params).hr != S_OK {
            (*msg).Buffer = original_buffer;
            break 'exit;
        }

        message_state_p =
            HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<message_state>()) as *mut message_state;
        if message_state_p.is_null() {
            (*params).hr = E_OUTOFMEMORY;
            (*msg).Buffer = original_buffer;
            break 'exit;
        }

        (*message_state_p).prefix_data_len =
            ((*msg).Buffer as usize - original_buffer as usize) as ULONG;
        (*message_state_p).binding_handle = (*msg).Handle;
        (*message_state_p).bypass_rpcrt = (*params).bypass_rpcrt;

        (*message_state_p).channel_hook_info.iid = (*params).iid;
        (*message_state_p).channel_hook_info.cbSize =
            core::mem::size_of::<SChannelHookCallInfo>() as DWORD;
        (*message_state_p).channel_hook_info.uCausality = orpcthis.cid;
        (*message_state_p).channel_hook_info.dwServerPid = GetCurrentProcessId();
        (*message_state_p).channel_hook_info.iMethod = (*msg).ProcNum;
        (*message_state_p).channel_hook_info.pObject = (*params).iface as *mut c_void;

        if !orpcthis.extensions.is_null()
            && !first_wire_orpc_extent.is_null()
            && (*orpcthis.extensions).size != 0
        {
            ChannelHooks_ServerNotify(
                core::ptr::addr_of_mut!((*message_state_p).channel_hook_info),
                (*msg).DataRepresentation,
                first_wire_orpc_extent,
                (*orpcthis.extensions).size,
            );
        }

        (*msg).Handle = message_state_p as *mut c_void;
        (*msg).BufferLength -= (*message_state_p).prefix_data_len;

        'exit_reset_state: {
            /* call message filter */

            if !(*COM_CurrentApt()).filter.is_null() {
                let mut interface_info = INTERFACEINFO {
                    pUnk: (*params).iface,
                    iid: (*params).iid,
                    wMethod: (*msg).ProcNum as WORD,
                };
                let calltype: CALLTYPE;

                if IsEqualGUID(
                    core::ptr::addr_of!(orpcthis.cid),
                    core::ptr::addr_of!((*COM_CurrentInfo()).causality_id),
                ) {
                    calltype = CALLTYPE_NESTED;
                } else if (*COM_CurrentInfo()).pending_call_count_server == 0 {
                    calltype = CALLTYPE_TOPLEVEL;
                } else {
                    calltype = CALLTYPE_TOPLEVEL_CALLPENDING;
                }

                let filter = (*COM_CurrentApt()).filter;
                let handlecall = ((*(*filter).lpVtbl).HandleInComingCall)(
                    filter,
                    calltype,
                    UlongToHandle(GetCurrentProcessId()),
                    0, /* FIXME */
                    &mut interface_info,
                );
                TRACE!("IMessageFilter_HandleInComingCall returned %d\n");
                match handlecall {
                    SERVERCALL_REJECTED => {
                        (*params).hr = RPC_E_CALL_REJECTED;
                        break 'exit_reset_state;
                    }
                    SERVERCALL_RETRYLATER => {
                        /* FIXME: handle retries on the client side before
                         * enabling the RPC_E_RETRY path (disabled in the C via
                         * #if 0 too) */
                        FIXME!("retry call later not implemented\n");
                    }
                    /* SERVERCALL_ISHANDLED */
                    _ => {}
                }
            }

            /* invoke the method */

            /* save the old causality ID - note: any calls executed while
             * processing messages received during the SendReceive will appear
             * to originate from this call - this should be checked with what
             * Windows does */
            let old_causality_id = (*COM_CurrentInfo()).causality_id;
            (*COM_CurrentInfo()).causality_id = orpcthis.cid;
            (*COM_CurrentInfo()).pending_call_count_server += 1;
            (*params).hr = (*params).stub.Invoke((*params).msg, (*params).chan);
            (*COM_CurrentInfo()).pending_call_count_server -= 1;
            (*COM_CurrentInfo()).causality_id = old_causality_id;

            /* the invoke allocated a new buffer, so free the old one */
            if (*message_state_p).bypass_rpcrt != FALSE && original_buffer != (*msg).Buffer {
                HeapFree(GetProcessHeap(), 0, original_buffer);
            }
        } /* exit_reset_state: */

        message_state_p = (*msg).Handle as *mut message_state;
        (*msg).Handle = (*message_state_p).binding_handle;
        (*msg).Buffer = (((*msg).Buffer as *mut u8)
            .sub((*message_state_p).prefix_data_len as usize)) as *mut c_void;
        (*msg).BufferLength += (*message_state_p).prefix_data_len;
    } /* exit: */

    HeapFree(GetProcessHeap(), 0, message_state_p as PVOID);
    if !(*params).handle.is_null() {
        SetEvent((*params).handle);
    }
}

unsafe extern "system" fn dispatch_rpc(msg: *mut RPC_MESSAGE) {
    let mut ipid: IPID = GUID_NULL;
    let mut apt: *mut apartment = core::ptr::null_mut();
    let mut stub_manager_p: *mut stub_manager = core::ptr::null_mut();

    RpcBindingInqObject((*msg).Handle, &mut ipid);

    TRACE!("ipid = %s, iMethod = %d\n");

    let params =
        HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<dispatch_params>()) as *mut dispatch_params;
    if params.is_null() {
        RpcRaiseException(E_OUTOFMEMORY);
    }

    let mut hr = ipid_get_dispatch_params(
        &ipid,
        &mut apt,
        &mut stub_manager_p,
        core::ptr::addr_of_mut!((*params).stub),
        core::ptr::addr_of_mut!((*params).chan),
        core::ptr::addr_of_mut!((*params).iid),
        core::ptr::addr_of_mut!((*params).iface),
    );
    if hr != S_OK {
        ERR!("no apartment found for ipid %s\n");
        HeapFree(GetProcessHeap(), 0, params as PVOID);
        RpcRaiseException(hr);
    }

    (*params).msg = msg as *mut RPCOLEMESSAGE;
    (*params).status = RPC_S_OK;
    (*params).hr = S_OK;
    (*params).handle = core::ptr::null_mut();
    (*params).bypass_rpcrt = FALSE;

    /* Note: this is the important difference between STAs and MTAs - we
     * always execute RPCs to STAs in the thread that originally created the
     * apartment (i.e. the one that pumps messages to the window) */
    if (*apt).multi_threaded == FALSE {
        (*params).handle = CreateEventW(core::ptr::null_mut(), FALSE, FALSE, core::ptr::null());

        TRACE!("Calling apartment thread 0x%08x...\n");

        if PostMessageW(apartment_getwindow(apt), DM_EXECUTERPC, 0, params as LPARAM) != FALSE {
            WaitForSingleObject((*params).handle, INFINITE);
        } else {
            ERR!("PostMessage failed with error %u\n");
            (*params).chan.Release();
            (*params).stub.Release();
        }
        CloseHandle((*params).handle);
    } else {
        let mut joined = FALSE;
        let info = COM_CurrentInfo();

        if (*info).apt.is_null() {
            enter_apartment(info, COINIT_MULTITHREADED);
            joined = TRUE;
        }
        RPC_ExecuteCall(params);
        if joined != FALSE {
            leave_apartment(info);
        }
    }

    hr = (*params).hr;
    if !(*params).chan.is_null() {
        (*params).chan.Release();
    }
    if !(*params).stub.is_null() {
        (*params).stub.Release();
    }
    HeapFree(GetProcessHeap(), 0, params as PVOID);

    stub_manager_int_release(stub_manager_p);
    apartment_release(apt);

    /* if IRpcStubBuffer_Invoke fails, we should raise an exception to tell
     * the RPC runtime that the call failed */
    if hr != S_OK {
        RpcRaiseException(hr);
    }
}

/// stub registration
/// (compobj_private.h: HRESULT RPC_RegisterInterface(REFIID))
#[no_mangle]
pub unsafe extern "C" fn RPC_RegisterInterface(riid: REFIID) -> HRESULT {
    let mut found = FALSE;
    let mut hr: HRESULT = S_OK;

    TRACE!("(%s)\n");

    EnterCriticalSection(core::ptr::addr_of_mut!(csRegIf));
    let head = core::ptr::addr_of_mut!(registered_interfaces);
    let mut cursor = (*head).next;
    while cursor != head {
        let rif = containing_record!(cursor, registered_if, entry);
        if IsEqualGUID(core::ptr::addr_of!((*rif).If.InterfaceId.SyntaxGUID), riid) {
            (*rif).refs += 1;
            found = TRUE;
            break;
        }
        cursor = (*cursor).next;
    }
    if found == FALSE {
        TRACE!("Creating new interface\n");

        let rif = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<registered_if>(),
        ) as *mut registered_if;
        if !rif.is_null() {
            (*rif).refs = 1;
            (*rif).If.Length = core::mem::size_of::<RPC_SERVER_INTERFACE>() as c_uint;
            /* RPC interface ID = COM interface ID */
            (*rif).If.InterfaceId.SyntaxGUID = *riid;
            (*rif).If.DispatchTable = core::ptr::addr_of_mut!(rpc_dispatch);
            /* all other fields are 0, including the version as COM objects
             * always have a version of 0.0 */
            let status = RpcServerRegisterIfEx(
                core::ptr::addr_of_mut!((*rif).If) as RPC_IF_HANDLE,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                RPC_IF_OLE | RPC_IF_AUTOLISTEN,
                RPC_C_LISTEN_MAX_CALLS_DEFAULT,
                core::ptr::null_mut(),
            );
            if status == RPC_S_OK {
                list_add_tail(
                    core::ptr::addr_of_mut!(registered_interfaces),
                    core::ptr::addr_of_mut!((*rif).entry),
                );
            } else {
                ERR!("RpcServerRegisterIfEx failed with error %d\n");
                HeapFree(GetProcessHeap(), 0, rif as PVOID);
                hr = HRESULT_FROM_WIN32(status as u32);
            }
        } else {
            hr = E_OUTOFMEMORY;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRegIf));
    hr
}

/// stub unregistration
/// (compobj_private.h: void RPC_UnregisterInterface(REFIID, BOOL))
#[no_mangle]
pub unsafe extern "C" fn RPC_UnregisterInterface(riid: REFIID, wait: BOOL) {
    EnterCriticalSection(core::ptr::addr_of_mut!(csRegIf));
    let head = core::ptr::addr_of_mut!(registered_interfaces);
    let mut cursor = (*head).next;
    while cursor != head {
        let rif = containing_record!(cursor, registered_if, entry);
        if IsEqualGUID(core::ptr::addr_of!((*rif).If.InterfaceId.SyntaxGUID), riid) {
            (*rif).refs -= 1;
            if (*rif).refs == 0 {
                RpcServerUnregisterIf(
                    core::ptr::addr_of_mut!((*rif).If) as RPC_IF_HANDLE,
                    core::ptr::null_mut(),
                    wait as c_uint,
                );
                list_remove(core::ptr::addr_of_mut!((*rif).entry));
                HeapFree(GetProcessHeap(), 0, rif as PVOID);
            }
            break;
        }
        cursor = (*cursor).next;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csRegIf));
}

/// get the info for an OXID, including the IPID for the rem unknown interface
/// and the string binding
/// (compobj_private.h: HRESULT RPC_ResolveOxid(OXID, OXID_INFO*))
#[no_mangle]
pub unsafe extern "C" fn RPC_ResolveOxid(oxid: OXID, oxid_info: *mut OXID_INFO) -> HRESULT {
    TRACE!("%s\n");

    (*oxid_info).dwTid = 0;
    (*oxid_info).dwPid = 0;
    (*oxid_info).dwAuthnHint = RPC_C_AUTHN_LEVEL_NONE;
    /* FIXME: this is a hack around not having an OXID resolver yet -
     * this function should contact the machine's OXID resolver and then it
     * should give us the IPID of the IRemUnknown interface */
    (*oxid_info).ipidRemUnknown.Data1 = 0xffffffff;
    (*oxid_info).ipidRemUnknown.Data2 = 0xffff;
    (*oxid_info).ipidRemUnknown.Data3 = 0xffff;
    memcpy(
        core::ptr::addr_of_mut!((*oxid_info).ipidRemUnknown.Data4) as PVOID,
        core::ptr::addr_of!(oxid) as *const c_void,
        core::mem::size_of::<OXID>(),
    );
    (*oxid_info).psa = core::ptr::null_mut() /* FIXME */;

    S_OK
}

/// make the apartment reachable by other threads and processes and create
/// the IRemUnknown object
/// (compobj_private.h: void RPC_StartRemoting(struct apartment*))
#[no_mangle]
pub unsafe extern "C" fn RPC_StartRemoting(apt: *mut apartment) {
    if InterlockedExchange(core::ptr::addr_of_mut!((*apt).remoting_started), TRUE) == 0 {
        let mut endpoint: [WCHAR; 200] = [0; 200];

        get_rpc_endpoint(endpoint.as_mut_ptr(), core::ptr::addr_of!((*apt).oxid));

        let status = RpcServerUseProtseqEpW(
            wszRpcTransport.as_ptr(),
            RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
            endpoint.as_ptr(),
            core::ptr::null_mut(),
        );
        if status != RPC_S_OK {
            ERR!("Couldn't register endpoint %s\n");
        }

        /* FIXME: move remote unknown exporting into this function */
    }
    start_apartment_remote_unknown(apt);
}

// ---------------------------------------------------------------------------
// local server activation
// ---------------------------------------------------------------------------

unsafe fn create_server(rclsid: REFCLSID, process: *mut HANDLE) -> HRESULT {
    let wszLocalServer32: &[u16] = wide!("LocalServer32");
    let embedding: &[u16] = wide!(" -Embedding"); /* ARRAY_SIZE == 12 incl. NUL */
    let mut key: HKEY = core::ptr::null_mut();
    let mut command: [WCHAR; MAX_PATH + 12] = [0; MAX_PATH + 12];
    let mut size: DWORD = ((MAX_PATH + 1) * core::mem::size_of::<WCHAR>()) as DWORD;
    let mut sinfo: STARTUPINFOW = core::mem::zeroed();
    let mut pinfo: PROCESS_INFORMATION = core::mem::zeroed();

    let hres = COM_OpenKeyForCLSID(rclsid, wszLocalServer32.as_ptr(), KEY_READ, &mut key);
    if FAILED(hres) {
        ERR!("class %s not registered\n");
        return hres;
    }

    let ret = RegQueryValueExW(
        key,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        command.as_mut_ptr() as LPBYTE,
        &mut size,
    );
    RegCloseKey(key);
    if ret != 0 {
        WARN!("No default value for LocalServer32 key\n");
        return REGDB_E_CLASSNOTREG; /* FIXME: check retval */
    }

    memset(
        core::ptr::addr_of_mut!(sinfo) as PVOID,
        0,
        core::mem::size_of::<STARTUPINFOW>(),
    );
    sinfo.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;

    /* EXE servers are started with the -Embedding switch. */

    lstrcatW(command.as_mut_ptr(), embedding.as_ptr());

    TRACE!("activating local server %s for %s\n");

    /* FIXME: Win2003 supports a ServerExecutable value that is passed into
     * CreateProcess */
    if CreateProcessW(
        core::ptr::null(),
        command.as_mut_ptr(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        FALSE,
        DETACHED_PROCESS,
        core::ptr::null_mut(),
        core::ptr::null(),
        &mut sinfo,
        &mut pinfo,
    ) == FALSE
    {
        WARN!("failed to run local server %s\n");
        return HRESULT_FROM_WIN32(GetLastError());
    }
    *process = pinfo.hProcess;
    CloseHandle(pinfo.hThread);

    S_OK
}

/// start_local_service() - start a service given its name and parameters
unsafe fn start_local_service(name: LPCWSTR, num: DWORD, params: *const LPCWSTR) -> DWORD {
    let mut r: DWORD = ERROR_FUNCTION_FAILED;

    TRACE!("Starting service %s %d params\n");

    let handle = OpenSCManagerW(core::ptr::null(), core::ptr::null(), SC_MANAGER_CONNECT);
    if handle.is_null() {
        return r;
    }
    let hsvc = OpenServiceW(handle, name, SERVICE_START);
    if !hsvc.is_null() {
        if StartServiceW(hsvc, num, params) != FALSE {
            r = ERROR_SUCCESS;
        } else {
            r = GetLastError();
        }
        if r == ERROR_SERVICE_ALREADY_RUNNING {
            r = ERROR_SUCCESS;
        }
        CloseServiceHandle(hsvc);
    } else {
        r = GetLastError();
    }
    CloseServiceHandle(handle);

    TRACE!("StartService returned error %u (%s)\n");

    r
}

/// create_local_service() - start a COM server in a service
///
///   To start a Local Service, we read the AppID value under
/// the class's CLSID key, then open the HKCR\\AppId key specified
/// there and check for a LocalService value.
///
/// Note:  Local Services are not supported under Windows 9x
unsafe fn create_local_service(rclsid: REFCLSID) -> HRESULT {
    let mut buf: [WCHAR; CHARS_IN_GUID] = [0; CHARS_IN_GUID];
    let szLocalService: &[u16] = wide!("LocalService");
    let szServiceParams: &[u16] = wide!("ServiceParams");
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut r: LONG;
    let mut type_: DWORD = 0;
    let mut sz: DWORD;

    TRACE!("Attempting to start Local service for %s\n");

    let mut hres = COM_OpenKeyForAppIdFromCLSID(rclsid, KEY_READ, &mut hkey);
    if FAILED(hres) {
        return hres;
    }

    /* read the LocalService and ServiceParameters values from the AppID key */
    sz = core::mem::size_of::<[WCHAR; CHARS_IN_GUID]>() as DWORD;
    r = RegQueryValueExW(
        hkey,
        szLocalService.as_ptr(),
        core::ptr::null_mut(),
        &mut type_,
        buf.as_mut_ptr() as LPBYTE,
        &mut sz,
    );
    if r == ERROR_SUCCESS as LONG && type_ == REG_SZ {
        let mut num_args: DWORD = 0;
        let mut args: [LPWSTR; 1] = [core::ptr::null_mut()];

        /*
         * FIXME: I'm not really sure how to deal with the service parameters.
         *        I suspect that the string returned from RegQueryValueExW
         *        should be split into a number of arguments by spaces.
         *        It would make more sense if ServiceParams contained a
         *        REG_MULTI_SZ here, but it's a REG_SZ for the services
         *        that I'm interested in for the moment.
         */
        r = RegQueryValueExW(
            hkey,
            szServiceParams.as_ptr(),
            core::ptr::null_mut(),
            &mut type_,
            core::ptr::null_mut(),
            &mut sz,
        );
        if r == ERROR_SUCCESS as LONG && type_ == REG_SZ && sz != 0 {
            args[0] = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sz as usize) as LPWSTR;
            num_args += 1;
            RegQueryValueExW(
                hkey,
                szServiceParams.as_ptr(),
                core::ptr::null_mut(),
                &mut type_,
                args[0] as LPBYTE,
                &mut sz,
            );
        }
        let sr = start_local_service(buf.as_ptr(), num_args, args.as_ptr() as *const LPCWSTR);
        if sr != ERROR_SUCCESS {
            hres = REGDB_E_CLASSNOTREG; /* FIXME: check retval */
        }
        HeapFree(GetProcessHeap(), 0, args[0] as PVOID);
    } else {
        WARN!("No LocalService value\n");
        hres = REGDB_E_CLASSNOTREG; /* FIXME: check retval */
    }
    RegCloseKey(hkey);

    hres
}

unsafe fn get_localserver_pipe_name(pipefn: *mut WCHAR, rclsid: REFCLSID) {
    let wszPipeRef: &[u16] = wide!("\\\\.\\pipe\\"); /* 9 chars + NUL */
    lstrcpyW(pipefn, wszPipeRef.as_ptr());
    StringFromGUID2(rclsid, pipefn.add(wszPipeRef.len() - 1), CHARS_IN_GUID as c_int);
}

/// FIXME: should call to rpcss instead
/// (compobj_private.h: HRESULT RPC_GetLocalClassObject(REFCLSID, REFIID, LPVOID*))
#[no_mangle]
pub unsafe extern "C" fn RPC_GetLocalClassObject(
    rclsid: REFCLSID,
    iid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut hres: HRESULT;
    let mut hPipe: HANDLE;
    let mut pipefn: [WCHAR; 100] = [0; 100];
    let mut res: DWORD = 0;
    let mut bufferlen: DWORD = 0;
    let mut marshalbuffer: [u8; 200] = [0; 200];
    let mut pStm: *mut IStream = core::ptr::null_mut();
    let seekto: LARGE_INTEGER;
    let mut newpos: ULARGE_INTEGER = 0;
    let mut tries: c_int = 0;
    let mut local_server: *mut IServiceProvider = core::ptr::null_mut();

    const MAXTRIES: c_int = 30; /* 30 seconds */

    TRACE!("rclsid=%s, iid=%s\n");

    get_localserver_pipe_name(pipefn.as_mut_ptr(), rclsid);

    loop {
        tries += 1;
        if tries > MAXTRIES {
            break;
        }
        TRACE!("waiting for %s\n");

        WaitNamedPipeW(pipefn.as_ptr(), NMPWAIT_WAIT_FOREVER);
        hPipe = CreateFileW(
            pipefn.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );
        if hPipe == INVALID_HANDLE_VALUE {
            let mut index: DWORD = 0;
            let start_ticks: DWORD;
            let mut process: HANDLE = core::ptr::null_mut();
            if tries == 1 {
                hres = create_local_service(rclsid);
                if hres != 0 {
                    hres = create_server(rclsid, &mut process);
                    if hres != 0 {
                        return hres;
                    }
                }
            } else {
                WARN!("Connecting to %s, no response yet, retrying: le is %u\n");
            }
            /* wait for one second, even if messages arrive */
            start_ticks = GetTickCount();
            loop {
                if SUCCEEDED(CoWaitForMultipleHandles(
                    0,
                    1000,
                    (!process.is_null()) as ULONG,
                    &process,
                    &mut index,
                )) && !process.is_null()
                    && index == 0
                {
                    WARN!("server for %s failed to start\n");
                    CloseHandle(hPipe);
                    CloseHandle(process);
                    return E_NOINTERFACE;
                }
                if GetTickCount().wrapping_sub(start_ticks) >= 1000 {
                    break;
                }
            }
            if !process.is_null() {
                CloseHandle(process);
            }
            continue;
        }
        bufferlen = 0;
        if ReadFile(
            hPipe,
            marshalbuffer.as_mut_ptr() as LPVOID,
            core::mem::size_of::<[u8; 200]>() as DWORD,
            &mut bufferlen,
            core::ptr::null_mut(),
        ) == FALSE
        {
            FIXME!("Failed to read marshal id from classfactory of %s.\n");
            CloseHandle(hPipe);
            Sleep(1000);
            continue;
        }
        TRACE!("read marshal id from pipe\n");
        CloseHandle(hPipe);
        break;
    }

    if tries >= MAXTRIES {
        return E_NOINTERFACE;
    }

    hres = CreateStreamOnHGlobal(core::ptr::null_mut(), TRUE, (&raw mut pStm).cast());
    if hres != S_OK {
        return hres;
    }
    'out: {
        hres = pStm.Write(
            marshalbuffer.as_ptr() as *const c_void,
            bufferlen,
            &mut res,
        );
        if hres != S_OK {
            break 'out;
        }
        seekto = 0;
        hres = pStm.Seek(seekto, STREAM_SEEK_SET, &mut newpos);

        TRACE!("unmarshalling local server\n");
        hres = CoUnmarshalInterface(
            pStm,
            &IID_IServiceProvider,
            core::ptr::addr_of_mut!(local_server) as *mut LPVOID,
        );
        if SUCCEEDED(hres) {
            hres = local_server.QueryService(rclsid, iid, ppv as *mut *mut c_void);
        }
        /* NOTE: the C releases `local_server` unconditionally here even though
         * it is UNINITIALIZED stack garbage when CoUnmarshalInterface failed
         * (latent Wine bug, fixed upstream later). We null-init and guard —
         * the success path is byte-identical; the failure path avoids the
         * undefined crash. Noted in MANIFEST. */
        if !local_server.is_null() {
            local_server.Release();
        }
    }
    /* out: */
    pStm.Release();
    hres
}

#[repr(C)]
struct local_server_params {
    clsid: CLSID,
    stream: *mut IStream,
    pipe: HANDLE,
    stop_event: HANDLE,
    thread: HANDLE,
    multi_use: BOOL,
}

/// FIXME: should call to rpcss instead
unsafe extern "system" fn local_server_thread(param: LPVOID) -> DWORD {
    let lsp = param as *mut local_server_params;
    let mut pipefn: [WCHAR; 100] = [0; 100];
    let mut hres: HRESULT;
    let pStm: *mut IStream = (*lsp).stream;
    let mut ststg: STATSTG = core::mem::zeroed();
    let mut buffer: *mut u8;
    let mut buflen: c_int;
    let mut seekto: LARGE_INTEGER;
    let mut newpos: ULARGE_INTEGER = 0;
    let mut res: ULONG = 0;
    let multi_use = (*lsp).multi_use;
    let mut ovl: OVERLAPPED = core::mem::zeroed();
    let pipe_event: HANDLE;
    let mut hPipe: HANDLE = (*lsp).pipe;
    let mut bytes: DWORD = 0;

    TRACE!("Starting threader for %s.\n");

    memset(
        core::ptr::addr_of_mut!(ovl) as PVOID,
        0,
        core::mem::size_of::<OVERLAPPED>(),
    );
    get_localserver_pipe_name(pipefn.as_mut_ptr(), core::ptr::addr_of!((*lsp).clsid));
    pipe_event = CreateEventW(core::ptr::null_mut(), FALSE, FALSE, core::ptr::null());
    ovl.hEvent = pipe_event;

    loop {
        if ConnectNamedPipe(hPipe, &mut ovl) == FALSE {
            let error = GetLastError();
            if error == ERROR_IO_PENDING {
                let handles: [HANDLE; 2] = [pipe_event, (*lsp).stop_event];
                let ret = WaitForMultipleObjects(2, handles.as_ptr(), FALSE, INFINITE);
                if ret != WAIT_OBJECT_0 {
                    break;
                }
            }
            /* client already connected isn't an error */
            else if error != ERROR_PIPE_CONNECTED {
                ERR!("ConnectNamedPipe failed with error %d\n");
                break;
            }
        }

        TRACE!("marshalling LocalServer to client\n");

        hres = pStm.Stat(&mut ststg, STATFLAG_NONAME);
        if hres != S_OK {
            break;
        }

        seekto = 0;
        hres = pStm.Seek(seekto, STREAM_SEEK_SET, &mut newpos);
        if hres != S_OK {
            FIXME!("IStream_Seek failed, %x\n");
            break;
        }

        buflen = ststg.cbSize as u32 as c_int; /* cbSize.u.LowPart */
        buffer = HeapAlloc(GetProcessHeap(), 0, buflen as usize) as *mut u8;

        hres = pStm.Read(buffer as *mut c_void, buflen as ULONG, &mut res);
        if hres != S_OK {
            FIXME!("Stream Read failed, %x\n");
            HeapFree(GetProcessHeap(), 0, buffer as PVOID);
            break;
        }

        WriteFile(
            hPipe,
            buffer as *const c_void,
            buflen as DWORD,
            &mut res as *mut ULONG as *mut DWORD,
            &mut ovl,
        );
        GetOverlappedResult(hPipe, &mut ovl, &mut bytes, TRUE);
        HeapFree(GetProcessHeap(), 0, buffer as PVOID);

        FlushFileBuffers(hPipe);
        DisconnectNamedPipe(hPipe);
        TRACE!("done marshalling LocalServer\n");

        if multi_use == FALSE {
            TRACE!("single use object, shutting down pipe %s\n");
            break;
        }
        let new_pipe = CreateNamedPipeW(
            pipefn.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
            PIPE_TYPE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            4096,
            4096,
            500, /* 0.5 second timeout */
            core::ptr::null_mut(),
        );
        if new_pipe == INVALID_HANDLE_VALUE {
            FIXME!("pipe creation failed for %s, le is %u\n");
            break;
        }
        CloseHandle(hPipe);
        hPipe = new_pipe;
    }

    CloseHandle(pipe_event);
    CloseHandle(hPipe);
    0
}

/// starts listening for a local server
/// (compobj_private.h: HRESULT RPC_StartLocalServer(REFCLSID, IStream*, BOOL, void**))
#[no_mangle]
pub unsafe extern "C" fn RPC_StartLocalServer(
    clsid: REFCLSID,
    stream: *mut IStream,
    multi_use: BOOL,
    registration: *mut *mut c_void,
) -> HRESULT {
    let mut tid: DWORD = 0;
    let err: DWORD;
    let mut pipefn: [WCHAR; 100] = [0; 100];

    let lsp = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<local_server_params>(),
    ) as *mut local_server_params;
    if lsp.is_null() {
        return E_OUTOFMEMORY;
    }

    (*lsp).clsid = *clsid;
    (*lsp).stream = stream;
    stream.AddRef();
    (*lsp).stop_event = CreateEventW(core::ptr::null_mut(), FALSE, FALSE, core::ptr::null());
    if (*lsp).stop_event.is_null() {
        HeapFree(GetProcessHeap(), 0, lsp as PVOID);
        return HRESULT_FROM_WIN32(GetLastError());
    }
    (*lsp).multi_use = multi_use;

    get_localserver_pipe_name(pipefn.as_mut_ptr(), core::ptr::addr_of!((*lsp).clsid));
    (*lsp).pipe = CreateNamedPipeW(
        pipefn.as_ptr(),
        PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
        PIPE_TYPE_BYTE | PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        4096,
        4096,
        500, /* 0.5 second timeout */
        core::ptr::null_mut(),
    );
    if (*lsp).pipe == INVALID_HANDLE_VALUE {
        err = GetLastError();
        FIXME!("pipe creation failed for %s, le is %u\n");
        CloseHandle((*lsp).stop_event);
        HeapFree(GetProcessHeap(), 0, lsp as PVOID);
        return HRESULT_FROM_WIN32(err);
    }

    (*lsp).thread = CreateThread(
        core::ptr::null_mut(),
        0,
        local_server_thread,
        lsp as LPVOID,
        0,
        &mut tid,
    );
    if (*lsp).thread.is_null() {
        CloseHandle((*lsp).pipe);
        CloseHandle((*lsp).stop_event);
        HeapFree(GetProcessHeap(), 0, lsp as PVOID);
        return HRESULT_FROM_WIN32(GetLastError());
    }

    *registration = lsp as *mut c_void;
    S_OK
}

/// stops listening for a local server
/// (compobj_private.h: void RPC_StopLocalServer(void*))
#[no_mangle]
pub unsafe extern "C" fn RPC_StopLocalServer(registration: *mut c_void) {
    let lsp = registration as *mut local_server_params;

    /* signal local_server_thread to stop */
    SetEvent((*lsp).stop_event);
    /* wait for it to exit */
    WaitForSingleObject((*lsp).thread, INFINITE);

    (*lsp).stream.Release();
    CloseHandle((*lsp).stop_event);
    CloseHandle((*lsp).thread);
    HeapFree(GetProcessHeap(), 0, lsp as PVOID);
}

// Compile-time tripwires for the wire-header offsets rpc.c hardcodes via
// FIELD_OFFSET (identical on all three targets — no pointers in the headers).
const _: () = {
    assert!(WIRE_ORPCTHIS_EXTENSIONS == 28);
    assert!(WIRE_ORPCTHAT_EXTENSIONS == 4);
    assert!(WIRE_ORPC_EXTENT_DATA == 24);
    assert!(WIRE_ORPC_EXTENT_ARRAY_EXTENT == 8);
};
