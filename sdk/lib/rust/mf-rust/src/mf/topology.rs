use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut next_node_id: LONG = 0;

static mut next_topology_id: TOPOID = 0;

// #define COBJMACROS, #undef INITGUID, WINE_DEFAULT_DEBUG_CHANNEL(...): not emitted (macros with no Rust declaration).

#[repr(C)]
pub struct node_stream {
    pub preferred_type: *mut IMFMediaType,
    pub connection: *mut topology_node,
    pub connection_stream: DWORD,
}

#[repr(C)]
pub struct node_streams {
    pub streams: *mut node_stream,
    pub size: usize,
    pub count: usize,
}

#[repr(C)]
pub struct topology_nodes {
    pub nodes: *mut *mut topology_node,
    pub size: usize,
    pub count: usize,
}

#[repr(C)]
pub struct topology_node {
    pub IMFTopologyNode_iface: IMFTopologyNode,
    pub refcount: LONG,
    pub attributes: *mut IMFAttributes,
    pub node_type: MF_TOPOLOGY_TYPE,
    pub id: TOPOID,
    pub object: *mut IUnknown,
    pub input_type: *mut IMFMediaType, // Only for tee nodes.
    pub inputs: node_streams,
    pub outputs: node_streams,
    pub cs: CRITICAL_SECTION,
}

#[repr(C)]
pub struct topology {
    pub IMFTopology_iface: IMFTopology,
    pub refcount: LONG,
    pub attributes: *mut IMFAttributes,
    pub nodes: topology_nodes,
    pub id: TOPOID,
}

#[repr(C)]
pub struct seq_source {
    pub IMFSequencerSource_iface: IMFSequencerSource,
    pub IMFMediaSourceTopologyProvider_iface: IMFMediaSourceTopologyProvider,
    pub refcount: LONG,
}

#[repr(C)]
pub struct type_handler {
    pub IMFMediaTypeHandler_iface: IMFMediaTypeHandler,
    pub refcount: LONG,
    pub node: *mut IMFTopologyNode,
    pub stream: DWORD,
    pub output: BOOL,
    pub transform: *mut IMFTransform,
}

#[repr(C)]
pub struct segment_offset {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub id: MFSequencerElementId,
    pub timeoffset: MFTIME,
}

// static const IMFTopologyNodeVtbl topologynodevtbl;            (vtable, skipped)
// static const IMFTopologyVtbl topologyvtbl = {...};            (vtable, skipped)
// static const IMFTopologyNodeVtbl topologynodevtbl = {...};    (vtable, skipped)
// static const IMFMediaTypeHandlerVtbl type_handler_vtbl = {...};  (vtable, skipped)
// static const IMFMediaSourceTopologyProviderVtbl seq_source_topology_provider_vtbl = {...}; (vtable, skipped)
// static const IMFSequencerSourceVtbl seqsourcevtbl = {...};    (vtable, skipped)
// static const IUnknownVtbl segment_offset_vtbl = {...};        (vtable, skipped)

pub const __WINE_DBCL_TRACE: core::ffi::c_uint = 3; // __WINE_DBCL_TRACE

unsafe extern "C" fn impl_from_IMFTopology(iface: *mut IMFTopology) -> *mut topology {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(topology, IMFTopology_iface)) as *mut topology;
}

unsafe extern "C" fn impl_from_IMFTopologyNode(iface: *mut IMFTopologyNode) -> *mut topology_node {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(topology_node, IMFTopologyNode_iface)) as *mut topology_node;
}

unsafe extern "C" fn unsafe_impl_from_IMFTopologyNode(iface: *mut IMFTopologyNode) -> *mut topology_node {
    if iface.is_null() || (*iface).lpVtbl != core::ptr::addr_of!(topologynodevtbl) {
        return core::ptr::null_mut();
    }
    return impl_from_IMFTopologyNode(iface);
}

unsafe extern "C" fn impl_from_IMFSequencerSource(iface: *mut IMFSequencerSource) -> *mut seq_source {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(seq_source, IMFSequencerSource_iface)) as *mut seq_source;
}

unsafe extern "C" fn impl_from_IMFMediaSourceTopologyProvider(
    iface: *mut IMFMediaSourceTopologyProvider,
) -> *mut seq_source {
    return CONTAINING_RECORD!(iface, seq_source, IMFMediaSourceTopologyProvider_iface);
}

unsafe extern "C" fn topology_node_reserve_streams(streams: *mut node_streams, index: DWORD) -> HRESULT {
    if index == DWORD_MAX {
        return E_OUTOFMEMORY;
    }

    if mf_array_reserve(
        core::ptr::addr_of_mut!((*streams).streams) as *mut *mut ::core::ffi::c_void,
        core::ptr::addr_of_mut!((*streams).size),
        (index.wrapping_add(1 as DWORD)) as size_t,
        core::mem::size_of::<node_stream>() as size_t,
    ) == 0
    {
        return E_OUTOFMEMORY;
    }

    if (index as size_t) >= (*streams).count {
        memset(
            (*streams).streams.offset((*streams).count as isize) as *mut node_stream as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            (((index as size_t).wrapping_sub((*streams).count)).wrapping_add(1 as size_t))
                .wrapping_mul(core::mem::size_of::<node_stream>() as size_t),
        );
        (*streams).count = (index.wrapping_add(1 as DWORD)) as size_t;
    }

    return S_OK;
}

unsafe extern "system" fn topology_QueryInterface(iface: *mut IMFTopology, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopology)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*topology).IMFTopology_iface) as *mut core::ffi::c_void;
    } else {
        *out = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    IUnknown_AddRef(*out as *mut IUnknown);
    return S_OK;
}

unsafe extern "system" fn topology_AddRef(iface: *mut IMFTopology) -> ULONG {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    let refcount: ULONG = InterlockedIncrement(
        &raw mut (*topology).refcount as *mut ::core::ffi::c_long,
    ) as ULONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            170 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }

    return refcount;
}

unsafe extern "C" fn topology_node_disconnect_output(node: *mut topology_node, output_index: DWORD) -> HRESULT {
    let mut connection: *mut topology_node = core::ptr::null_mut();
    let mut stream: *mut node_stream = core::ptr::null_mut();
    let mut connection_stream: DWORD = 0;
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if (output_index as usize) < (*node).outputs.count {
        stream = (*node).outputs.streams.add(output_index as usize);

        if !(*stream).connection.is_null() {
            connection = (*stream).connection;
            connection_stream = (*stream).connection_stream;
            (*stream).connection = core::ptr::null_mut();
            (*stream).connection_stream = 0;
        } else {
            hr = MF_E_NOT_FOUND;
        }
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if !connection.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!((*connection).cs));

        if (connection_stream as usize) < (*connection).inputs.count {
            stream = (*connection).inputs.streams.add(connection_stream as usize);

            if !(*stream).connection.is_null() {
                (*stream).connection = core::ptr::null_mut();
                (*stream).connection_stream = 0;
            }
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!((*connection).cs));

        IMFTopologyNode_Release(core::ptr::addr_of_mut!((*connection).IMFTopologyNode_iface));
        IMFTopologyNode_Release(core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface));
    }

    return hr;
}

unsafe extern "C" fn topology_node_disconnect(node: *mut topology_node) {
    let mut stream: *mut node_stream = core::ptr::null_mut();
    let mut i: size_t;

    i = 0 as size_t;
    while i < (*node).outputs.count {
        topology_node_disconnect_output(node, i as DWORD);
        i += 1;
    }

    i = 0 as size_t;
    while i < (*node).inputs.count {
        stream = (*node).inputs.streams.add(i) as *mut node_stream;
        if !(*stream).connection.is_null() {
            topology_node_disconnect_output((*stream).connection, (*stream).connection_stream);
        }
        i += 1;
    }
}

unsafe extern "C" fn topology_clear(topology: *mut topology) {
    let mut i: usize = 0 as usize;
    while i < (*topology).nodes.count {
        topology_node_disconnect((*(*topology).nodes.nodes.add(i)));
        IMFTopologyNode_Release(core::ptr::addr_of_mut!((**(*topology).nodes.nodes.add(i)).IMFTopologyNode_iface));
        i = i.wrapping_add(1);
    }
    free((*topology).nodes.nodes as *mut core::ffi::c_void);
    (*topology).nodes.nodes = core::ptr::null_mut();
    (*topology).nodes.count = 0 as usize;
    (*topology).nodes.size = 0 as usize;
}
unsafe extern "system" fn topology_Release(mut iface: *mut IMFTopology) -> ULONG {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*topology).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_Release\0" as *const u8 as *const ::core::ffi::c_char,
            263 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    if refcount == 0 {
        if !(*topology).attributes.is_null() {
            IMFAttributes_Release((*topology).attributes);
        }
        topology_clear(topology);
        free(topology as *mut ::core::ffi::c_void);
    }
    return refcount;
}

unsafe extern "system" fn topology_GetItem(iface: *mut IMFTopology, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_GetItem\0" as *const u8 as *const ::core::ffi::c_char,
            280 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetItem((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_GetItemType(
    iface: *mut IMFTopology,
    key: *const GUID,
    type_: *mut MF_ATTRIBUTE_TYPE,
) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    TRACE!(
        "%p, %s, %p.\n",
        iface,
        debugstr_guid(key as *const _GUID),
        type_
    );

    return IMFAttributes_GetItemType((*topology).attributes, key, type_);
}

unsafe extern "system" fn topology_CompareItem(
    iface: *mut IMFTopology,
    key: *const GUID,
    value: *const PROPVARIANT,
    result: *mut BOOL,
) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_CompareItem\0".as_ptr() as *const i8,
            298i32,
            b"%p, %s, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key),
            value,
            result,
        );
    }
    return IMFAttributes_CompareItem((*topology).attributes, key, value, result);
}
unsafe extern "system" fn topology_Compare(
    iface: *mut IMFTopology,
    theirs: *mut IMFAttributes,
    type_0: MF_ATTRIBUTES_MATCH_TYPE,
    result: *mut BOOL,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE as __wine_debug_class;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"topology_Compare\0".as_ptr() as *const core::ffi::c_char,
            308 as core::ffi::c_int,
            b"%p, %p, %d, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            theirs,
            type_0 as core::ffi::c_uint,
            result,
        );
    }
    return IMFAttributes_Compare((*topology).attributes, theirs, type_0, result);
}

unsafe extern "system" fn topology_GetUINT32(iface: *mut IMFTopology, key: REFGUID, value: *mut UINT32) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetUINT32((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_GetUINT64(iface: *mut IMFTopology, key: *const GUID, value: *mut UINT64) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const i8,
            b"topology_GetUINT64\0" as *const u8 as *const i8,
            326 as i32,
            b"%p, %s, %p.\n\0" as *const u8 as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }

    return IMFAttributes_GetUINT64((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_GetDouble(iface: *mut IMFTopology, key: *const GUID, value: *mut f64) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetDouble((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_GetGUID(iface: *mut IMFTopology, key: *const GUID, value: *mut GUID) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetGUID((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_GetStringLength(iface: *mut IMFTopology, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetStringLength((*topology).attributes, key, length);
}
unsafe extern "system" fn topology_GetString(
    mut iface: *mut IMFTopology,
    key: *const GUID,
    mut value: *mut WCHAR,
    mut size: UINT32,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            363 as ::core::ffi::c_int,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            size,
            length,
        );
    }
    return IMFAttributes_GetString((*topology).attributes, key, value, size, length);
}

unsafe extern "system" fn topology_GetAllocatedString(
    iface: *mut IMFTopology,
    key: *const GUID,
    value: *mut *mut WCHAR,
    length: *mut UINT32,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    TRACE!(
        "%p, %s, %p, %p.\n",
        iface,
        debugstr_guid(key),
        value,
        length
    );
    return IMFAttributes_GetAllocatedString((*topology).attributes, key, value, length);
}

unsafe extern "system" fn topology_GetBlobSize(iface: *mut IMFTopology, key: REFGUID, size: *mut UINT32) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetBlobSize((*topology).attributes, key, size);
}
unsafe extern "system" fn topology_GetBlob(
    mut iface: *mut IMFTopology,
    key: *const GUID,
    mut buf: *mut UINT8,
    mut bufsize: UINT32,
    mut blobsize: *mut UINT32,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_GetBlob\0".as_ptr() as *const i8,
            392i32,
            b"%p, %s, %p, %d, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            bufsize,
            blobsize,
        );
    }
    return IMFAttributes_GetBlob((*topology).attributes, key, buf, bufsize, blobsize);
}
unsafe extern "system" fn topology_GetAllocatedBlob(iface: *mut IMFTopology, key: *const GUID, buf: *mut *mut UINT8, size: *mut UINT32) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetAllocatedBlob((*topology).attributes, key, buf, size);
}

unsafe extern "system" fn topology_GetUnknown(iface: *mut IMFTopology, key: *const GUID, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetUnknown((*topology).attributes, key, riid, ppv);
}

unsafe extern "system" fn topology_SetItem(iface: *mut IMFTopology, key: *const GUID, value: *const PROPVARIANT) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as i32 & (1i32) << __WINE_DBCL_TRACE as i32 != 0 {
        TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key as *const _GUID), value);
    }
    return IMFAttributes_SetItem((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_DeleteItem(iface: *mut IMFTopology, key: *const GUID) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    TRACE!("%p, %s.\n", topology, debugstr_guid(key));
    return IMFAttributes_DeleteItem((*topology).attributes, key);
}

unsafe extern "system" fn topology_DeleteAllItems(iface: *mut IMFTopology) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_DeleteAllItems((*topology).attributes);
}
unsafe extern "system" fn topology_SetUINT32(
    mut iface: *mut IMFTopology,
    key: *const GUID,
    mut value: UINT32,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_SetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            446 as ::core::ffi::c_int,
            b"%p, %s, %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_SetUINT32((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_SetUINT64(iface: *mut IMFTopology, key: *const GUID, value: UINT64) -> HRESULT {
    let topology = impl_from_IMFTopology(iface);
    TRACE!("%p, %s, %s.\n", iface, debugstr_guid(key), wine_dbgstr_longlong(value));
    return IMFAttributes_SetUINT64((*topology).attributes, key, value);
}
unsafe extern "system" fn topology_SetDouble(iface: *mut IMFTopology, key: *const GUID, value: f64) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_SetDouble((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_SetGUID(iface: *mut IMFTopology, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_SetGUID((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_SetString(iface: *mut IMFTopology, key: REFGUID, value: *const WCHAR) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_SetString\0".as_ptr() as *const i8,
            482 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*topology).attributes, key, value);
}

unsafe extern "system" fn topology_SetBlob(iface: *mut IMFTopology, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    TRACE!("%p, %s, %p, %d.\n", iface, debugstr_guid(key), buf, size);

    return IMFAttributes_SetBlob((*topology).attributes, key, buf, size);
}

unsafe extern "system" fn topology_SetUnknown(
    mut iface: *mut IMFTopology,
    key: *const GUID,
    mut unknown: *mut IUnknown,
) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            500 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            unknown,
        );
    }
    return IMFAttributes_SetUnknown((*topology).attributes, key, unknown);
}

unsafe extern "system" fn topology_LockStore(iface: *mut IMFTopology) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_LockStore\0".as_ptr() as *const i8,
            509i32,
            b"%p.\n\0".as_ptr() as *const i8,
            iface,
        );
    }
    return IMFAttributes_LockStore((*topology).attributes);
}
unsafe extern "system" fn topology_UnlockStore(iface: *mut IMFTopology) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_UnlockStore((*topology).attributes);
}

unsafe extern "system" fn topology_GetCount(iface: *mut IMFTopology, count: *mut UINT32) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_GetCount((*topology).attributes, count);
}

unsafe extern "system" fn topology_GetItemByIndex(iface: *mut IMFTopology, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_GetItemByIndex\0" as *const u8 as *const ::core::ffi::c_char,
            536 as ::core::ffi::c_int,
            b"%p, %u, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            key,
            value,
        );
    }
    return IMFAttributes_GetItemByIndex((*topology).attributes, index, key, value);
}

unsafe extern "system" fn topology_CopyAllItems(iface: *mut IMFTopology, dest: *mut IMFAttributes) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    return IMFAttributes_CopyAllItems((*topology).attributes, dest);
}

unsafe extern "system" fn topology_GetTopologyID(iface: *mut IMFTopology, id: *mut TOPOID) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    if id.is_null() {
        return E_POINTER;
    }

    *id = (*topology).id;

    return S_OK;
}

unsafe extern "C" fn topology_get_node_by_id(topology: *const topology, id: TOPOID, node: *mut *mut topology_node) -> HRESULT {
    let mut i: size_t = 0;

    i = 0;
    while i < (*topology).nodes.count {
        if (**(*topology).nodes.nodes.add(i)).id == id {
            *node = *(*topology).nodes.nodes.add(i);
            return S_OK;
        }
        i += 1;
    }

    return MF_E_NOT_FOUND;
}

unsafe extern "C" fn topology_add_node(topology: *mut topology, node: *mut topology_node) -> HRESULT {
    let mut match_: *mut topology_node = core::ptr::null_mut();

    if node.is_null() {
        return E_POINTER;
    }

    if topology_get_node_by_id(topology, (*node).id, core::ptr::addr_of_mut!(match_)) >= 0 {
        return E_INVALIDARG;
    }

    if mf_array_reserve(
        core::ptr::addr_of_mut!((*topology).nodes.nodes) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*topology).nodes.size),
        (*topology).nodes.count + 1,
        core::mem::size_of::<topology_node>(),
    ) == 0
    {
        return E_OUTOFMEMORY;
    }

    let idx = (*topology).nodes.count;
    (*topology).nodes.count += 1;
    *(*topology).nodes.nodes.add(idx) = node;
    IMFTopologyNode_AddRef(core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface));

    return S_OK;
}

unsafe extern "system" fn topology_AddNode(iface: *mut IMFTopology, node_iface: *mut IMFTopologyNode) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    let node: *mut topology_node = unsafe_impl_from_IMFTopologyNode(node_iface);

    TRACE!("%p, %p.\n", iface, node_iface);

    return topology_add_node(topology, node);
}

unsafe extern "system" fn topology_RemoveNode(
    iface: *mut IMFTopology,
    node: *mut IMFTopologyNode,
) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    let mut i: size_t = 0;
    let mut count: size_t = 0;

    i = 0;
    while i < (*topology).nodes.count {
        if core::ptr::addr_of_mut!((**(*topology).nodes.nodes.add(i)).IMFTopologyNode_iface) == node {
            topology_node_disconnect(*(*topology).nodes.nodes.add(i));
            IMFTopologyNode_Release(core::ptr::addr_of_mut!((**(*topology).nodes.nodes.add(i)).IMFTopologyNode_iface));
            count = (*topology)
                .nodes
                .count
                .wrapping_sub(i)
                .wrapping_sub(1);
            if count != 0 {
                memmove(
                    (*topology).nodes.nodes.add(i) as *mut core::ffi::c_void,
                    (*topology).nodes.nodes.add(i.wrapping_add(1)) as *const core::ffi::c_void,
                    count.wrapping_mul(core::mem::size_of::<topology_node>()),
                );
            }
            (*topology).nodes.count = (*topology).nodes.count.wrapping_sub(1);
            return S_OK;
        }
        i = i.wrapping_add(1);
    }

    return E_INVALIDARG;
}

unsafe extern "system" fn topology_GetNodeCount(iface: *mut IMFTopology, count: *mut WORD) -> HRESULT {
    let mut topology: *mut topology = impl_from_IMFTopology(iface);

    if count.is_null() {
        return E_POINTER;
    }

    *count = (*topology).nodes.count as WORD;

    return S_OK;
}

unsafe extern "system" fn topology_GetNode(iface: *mut IMFTopology, index: WORD, node: *mut *mut IMFTopologyNode) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    if node.is_null() {
        return E_POINTER;
    }

    if (index as usize) >= (*topology).nodes.count {
        return MF_E_INVALIDINDEX;
    }

    *node = core::ptr::addr_of_mut!((**(*topology).nodes.nodes.offset(index as isize)).IMFTopologyNode_iface);
    IMFTopologyNode_AddRef(*node);

    return S_OK;
}

unsafe extern "system" fn topology_Clear(iface: *mut IMFTopology) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);

    topology_clear(topology);
    return S_OK;
}

unsafe extern "system" fn topology_CloneFrom(iface: *mut IMFTopology, src: *mut IMFTopology) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    let src_topology: *mut topology = unsafe_impl_from_IMFTopology(src);
    let mut node: *mut topology_node = core::ptr::null_mut();
    let mut i: size_t;
    let mut j: size_t;
    let mut hr: HRESULT;

    TRACE!("%p, %p.\n", iface, src);

    topology_clear(topology);

    /* Clone nodes. */
    i = 0;
    while i < (*src_topology).nodes.count {
        hr = create_topology_node(
            (**(*src_topology).nodes.nodes.add(i)).node_type,
            core::ptr::addr_of_mut!(node),
        );
        if hr < 0 {
            WARN!("Failed to create a node, hr %#lx.\n", hr);
            break;
        }

        hr = IMFTopologyNode_CloneFrom(
            core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface),
            core::ptr::addr_of_mut!((**(*src_topology).nodes.nodes.add(i)).IMFTopologyNode_iface),
        );
        if hr >= 0 {
            topology_add_node(topology, node);
        }

        IMFTopologyNode_Release(core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface));
        i = i.wrapping_add(1);
    }

    /* Clone connections. */
    i = 0;
    while i < (*src_topology).nodes.count {
        let outputs: *const node_streams =
            core::ptr::addr_of!((**(*src_topology).nodes.nodes.add(i)).outputs);

        j = 0;
        while j < (*outputs).count {
            let input_index: DWORD = (*(*outputs).streams.add(j)).connection_stream;
            let mut id: TOPOID;

            if !(*(*outputs).streams.add(j)).connection.is_null() {
                id = (*(*(*outputs).streams.add(j)).connection).id;

                /* Skip node lookup in destination topology, assuming same node order. */
                hr = topology_get_node_by_id(topology, id, core::ptr::addr_of_mut!(node));
                if hr >= 0 {
                    topology_node_connect_output(
                        *(*topology).nodes.nodes.add(i),
                        j as DWORD,
                        node,
                        input_index,
                    );
                }
            }
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }

    /* Copy attributes and id. */
    hr = IMFTopology_CopyAllItems(
        src,
        core::ptr::addr_of_mut!((*topology).IMFTopology_iface) as *mut IMFAttributes,
    );
    if hr >= 0 {
        (*topology).id = (*src_topology).id;
    }

    return S_OK;
}

unsafe extern "system" fn topology_GetNodeByID(
    iface: *mut IMFTopology,
    id: TOPOID,
    ret: *mut *mut IMFTopologyNode,
) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    let mut node: *mut topology_node = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = topology_get_node_by_id(topology, id, core::ptr::addr_of_mut!(node));
    if hr >= 0 {
        *ret = core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface);
        IMFTopologyNode_AddRef(*ret);
    } else {
        *ret = core::ptr::null_mut();
    }

    return hr;
}

unsafe extern "C" fn topology_get_node_collection(
    topology: *const topology,
    node_type: MF_TOPOLOGY_TYPE,
    collection: *mut *mut IMFCollection,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut i: size_t;

    if collection.is_null() {
        return E_POINTER;
    }

    hr = MFCreateCollection(collection);
    if hr < 0 {
        return hr;
    }

    i = 0;
    while i < (*topology).nodes.count {
        if (**(*topology).nodes.nodes.add(i)).node_type == node_type {
            hr = IMFCollection_AddElement(
                *collection,
                core::ptr::addr_of_mut!((**(*topology).nodes.nodes.add(i)).IMFTopologyNode_iface)
                    as *mut IUnknown,
            );
            if hr < 0 {
                IMFCollection_Release(*collection);
                *collection = core::ptr::null_mut::<IMFCollection>();
                break;
            }
        }
        i = i.wrapping_add(1);
    }

    return hr;
}

unsafe extern "system" fn topology_GetSourceNodeCollection(iface: *mut IMFTopology, collection: *mut *mut IMFCollection) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    return topology_get_node_collection(topology, MF_TOPOLOGY_SOURCESTREAM_NODE, collection);
}

unsafe extern "system" fn topology_GetOutputNodeCollection(iface: *mut IMFTopology, collection: *mut *mut IMFCollection) -> HRESULT {
    let topology: *mut topology = impl_from_IMFTopology(iface);
    TRACE!("%p, %p.\n", iface, collection);
    return topology_get_node_collection(topology, MF_TOPOLOGY_OUTPUT_NODE, collection);
}

unsafe extern "C" fn unsafe_impl_from_IMFTopology(iface: *mut IMFTopology) -> *mut topology {
    if iface.is_null() || (*iface).lpVtbl != &raw const topologyvtbl {
        return core::ptr::null_mut();
    }
    return impl_from_IMFTopology(iface);
}

unsafe extern "C" fn topology_generate_id() -> TOPOID {
    let mut old: TOPOID = 0;
    loop {
        old = next_topology_id;
        if !(InterlockedCompareExchange64(
            core::ptr::addr_of_mut!(next_topology_id) as *mut LONG64,
            old.wrapping_add(1 as TOPOID) as LONG64,
            old as LONG64,
        ) as TOPOID
            != old)
        {
            break;
        }
    }
    return old.wrapping_add(1 as TOPOID);
}
#[no_mangle]
pub unsafe extern "C" fn create_topology(id: TOPOID, topology: *mut *mut IMFTopology) -> HRESULT {
    let mut object: *mut topology = core::ptr::null_mut::<topology>();
    let mut hr: HRESULT;
    object = calloc(1, (core::mem::size_of::<topology>()) as _) as *mut topology;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }
    (*object).IMFTopology_iface.lpVtbl = core::ptr::addr_of!(topologyvtbl);
    (*object).refcount = 1;
    hr = MFCreateAttributes(core::ptr::addr_of_mut!((*object).attributes), 0);
    if hr < 0 {
        IMFTopology_Release(core::ptr::addr_of_mut!((*object).IMFTopology_iface));
        return hr;
    }
    (*object).id = id;
    *topology = core::ptr::addr_of_mut!((*object).IMFTopology_iface);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTopology(topology: *mut *mut IMFTopology) -> HRESULT {
    if topology.is_null() {
        return E_POINTER;
    }

    return create_topology(topology_generate_id(), topology);
}

unsafe extern "system" fn topology_node_QueryInterface(iface: *mut IMFTopologyNode, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopologyNode))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAttributes))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *out = core::ptr::addr_of_mut!((*node).IMFTopologyNode_iface) as *mut core::ffi::c_void;
        IMFTopologyNode_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();

    return E_NOINTERFACE;
}

unsafe extern "system" fn topology_node_AddRef(iface: *mut IMFTopologyNode) -> ULONG {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*node).refcount)) as ULONG;

    return refcount;
}
unsafe extern "system" fn topology_node_Release(iface: *mut IMFTopologyNode) -> ULONG {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*node).refcount) as *mut LONG) as ULONG;
    let mut i: core::ffi::c_uint;

    if refcount == 0 {
        if !(*node).object.is_null() {
            IUnknown_Release((*node).object);
        }
        if !(*node).input_type.is_null() {
            IMFMediaType_Release((*node).input_type);
        }
        i = 0;
        while (i as usize) < (*node).inputs.count {
            if !(*(*node).inputs.streams.offset(i as isize)).preferred_type.is_null() {
                IMFMediaType_Release((*(*node).inputs.streams.offset(i as isize)).preferred_type);
            }
            i = i.wrapping_add(1);
        }
        i = 0;
        while (i as usize) < (*node).outputs.count {
            if !(*(*node).outputs.streams.offset(i as isize)).preferred_type.is_null() {
                IMFMediaType_Release((*(*node).outputs.streams.offset(i as isize)).preferred_type);
            }
            i = i.wrapping_add(1);
        }
        free((*node).inputs.streams as *mut core::ffi::c_void);
        free((*node).outputs.streams as *mut core::ffi::c_void);
        IMFAttributes_Release((*node).attributes);
        DeleteCriticalSection(core::ptr::addr_of_mut!((*node).cs));
        free(node as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn topology_node_GetItem(iface: *mut IMFTopologyNode, key: *const GUID, value: *mut PROPVARIANT) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), value);

    return IMFAttributes_GetItem((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_GetItemType(iface: *mut IMFTopologyNode, key: *const GUID, type_: *mut MF_ATTRIBUTE_TYPE) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    TRACE!("%p, %s, %p.\n", iface, debugstr_guid(key), type_);

    return IMFAttributes_GetItemType((*node).attributes, key, type_);
}

unsafe extern "system" fn topology_node_CompareItem(iface: *mut IMFTopologyNode, key: REFGUID, value: REFPROPVARIANT, result: *mut BOOL) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!("%p, %s, %p, %p.\n", iface, debugstr_guid(key), value, result);
    return IMFAttributes_CompareItem((*node).attributes, key, value, result);
}
unsafe extern "system" fn topology_node_Compare(
    mut iface: *mut IMFTopologyNode,
    mut theirs: *mut IMFAttributes,
    mut type_0: MF_ATTRIBUTES_MATCH_TYPE,
    mut result: *mut BOOL,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_Compare((*node).attributes, theirs, type_0, result);
}

unsafe extern "system" fn topology_node_GetUINT32(
    mut iface: *mut IMFTopologyNode,
    key: *const GUID,
    mut value: *mut UINT32,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetUINT32\0" as *const u8 as *const ::core::ffi::c_char,
            1017 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_GetUINT32((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_GetUINT64(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    value: *mut UINT64,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!(
        "%p, %s, %p.\n",
        iface,
        debugstr_guid(key as *const _GUID),
        value
    );
    return IMFAttributes_GetUINT64((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_GetDouble(
    mut iface: *mut IMFTopologyNode,
    key: *const GUID,
    mut value: *mut f64,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE as i32)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_node_GetDouble\0".as_ptr() as *const i8,
            1035i32,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key as *const GUID),
            value,
        );
    }
    return IMFAttributes_GetDouble((*node).attributes, key, value);
}
unsafe extern "system" fn topology_node_GetGUID(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    value: *mut GUID,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as i32 & ((1i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_node_GetGUID\0".as_ptr() as *const i8,
            1044,
            b"%p, %s, %p.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key),
            value,
        );
    }
    return IMFAttributes_GetGUID((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_GetStringLength(iface: *mut IMFTopologyNode, key: *const GUID, length: *mut UINT32) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_GetStringLength((*node).attributes, key, length);
}

unsafe extern "system" fn topology_node_GetString(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    value: *mut WCHAR,
    size: UINT32,
    length: *mut UINT32,
) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetString\0" as *const u8 as *const ::core::ffi::c_char,
            1063 as ::core::ffi::c_int,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            size,
            length,
        );
    }
    return IMFAttributes_GetString((*node).attributes, key, value, size, length);
}

unsafe extern "system" fn topology_node_GetAllocatedString(
    mut iface: *mut IMFTopologyNode,
    key: *const GUID,
    mut value: *mut *mut WCHAR,
    mut length: *mut UINT32,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetAllocatedString\0" as *const u8 as *const ::core::ffi::c_char,
            1073 as ::core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
            length,
        );
    }
    return IMFAttributes_GetAllocatedString((*node).attributes, key, value as *mut LPWSTR, length);
}

unsafe extern "system" fn topology_node_GetBlobSize(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    size: *mut UINT32,
) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetBlobSize\0" as *const u8 as *const ::core::ffi::c_char,
            1082 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            size,
        );
    }
    return IMFAttributes_GetBlobSize((*node).attributes, key, size);
}
unsafe extern "system" fn topology_node_GetBlob(
    mut iface: *mut IMFTopologyNode,
    key: *const GUID,
    mut buf: *mut UINT8,
    mut bufsize: UINT32,
    mut blobsize: *mut UINT32,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            1092 as ::core::ffi::c_int,
            b"%p, %s, %p, %d, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            bufsize,
            blobsize,
        );
    }
    return IMFAttributes_GetBlob((*node).attributes, key, buf, bufsize, blobsize);
}

unsafe extern "system" fn topology_node_GetAllocatedBlob(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    buf: *mut *mut UINT8,
    size: *mut UINT32,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetAllocatedBlob\0" as *const u8 as *const ::core::ffi::c_char,
            1101 as ::core::ffi::c_int,
            b"%p, %s, %p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }
    return IMFAttributes_GetAllocatedBlob((*node).attributes, key, buf, size);
}

unsafe extern "system" fn topology_node_GetUnknown(iface: *mut IMFTopologyNode, key: *const GUID, riid: *const GUID, ppv: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as core::ffi::c_int & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const core::ffi::c_char,
            b"topology_node_GetUnknown\0" as *const u8 as *const core::ffi::c_char,
            1110 as core::ffi::c_int,
            b"%p, %s, %s, %p.\n\0" as *const u8 as *const core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            debugstr_guid(riid as *const _GUID),
            ppv,
        );
    }
    return IMFAttributes_GetUnknown((*node).attributes, key, riid, ppv as *mut LPVOID);
}

unsafe extern "system" fn topology_node_SetItem(
    iface: *mut IMFTopologyNode,
    key: *const GUID,
    value: *const PROPVARIANT,
) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_SetItem\0" as *const u8 as *const ::core::ffi::c_char,
            1119 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }
    return IMFAttributes_SetItem((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_DeleteItem(iface: *mut IMFTopologyNode, key: *const GUID) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_DeleteItem((*node).attributes, key);
}

unsafe extern "system" fn topology_node_DeleteAllItems(iface: *mut IMFTopologyNode) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_DeleteAllItems((*node).attributes);
}

unsafe extern "system" fn topology_node_SetUINT32(iface: *mut IMFTopologyNode, key: *const GUID, value: UINT32) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    return IMFAttributes_SetUINT32((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_SetUINT64(iface: *mut IMFTopologyNode, key: *const GUID, value: UINT64) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_SetUINT64\0" as *const u8 as *const ::core::ffi::c_char,
            1155 as ::core::ffi::c_int,
            b"%p, %s, %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            wine_dbgstr_longlong(value as ULONGLONG),
        );
    }
    return IMFAttributes_SetUINT64((*node).attributes, key, value);
}
unsafe extern "system" fn topology_node_SetDouble(iface: *mut IMFTopologyNode, key: REFGUID, value: f64) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int & (1 as ::core::ffi::c_int) << 3 as ::core::ffi::c_int != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = 3 as __wine_debug_class; // __WINE_DBCL_TRACE
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const ::core::ffi::c_char,
            b"topology_node_SetDouble\0".as_ptr() as *const ::core::ffi::c_char,
            1164 as ::core::ffi::c_int,
            b"%p, %s, %f.\n\0".as_ptr() as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            value,
        );
    }

    return IMFAttributes_SetDouble((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_SetGUID(iface: *mut IMFTopologyNode, key: *const GUID, value: *const GUID) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!(
        "%p, %s, %s.\n",
        iface,
        debugstr_guid(key),
        debugstr_guid(value)
    );
    return IMFAttributes_SetGUID((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_SetString(iface: *mut IMFTopologyNode, key: *const GUID, value: *const WCHAR) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as i32 & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0".as_ptr() as *const i8,
            b"topology_node_SetString\0".as_ptr() as *const i8,
            1182 as i32,
            b"%p, %s, %s.\n\0".as_ptr() as *const i8,
            iface,
            debugstr_guid(key),
            debugstr_w(value),
        );
    }
    return IMFAttributes_SetString((*node).attributes, key, value);
}

unsafe extern "system" fn topology_node_SetBlob(iface: *mut IMFTopologyNode, key: *const GUID, buf: *const UINT8, size: UINT32) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_SetBlob\0" as *const u8 as *const ::core::ffi::c_char,
            1191 as ::core::ffi::c_int,
            b"%p, %s, %p, %d.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            buf,
            size,
        );
    }
    return IMFAttributes_SetBlob((*node).attributes, key, buf, size);
}
unsafe extern "system" fn topology_node_SetUnknown(iface: *mut IMFTopologyNode, key: *const GUID, unknown: *mut IUnknown) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_SetUnknown\0" as *const u8 as *const ::core::ffi::c_char,
            1200 as ::core::ffi::c_int,
            b"%p, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            debugstr_guid(key as *const _GUID),
            unknown,
        );
    }
    return IMFAttributes_SetUnknown((*node).attributes, key, unknown);
}

unsafe extern "system" fn topology_node_LockStore(iface: *mut IMFTopologyNode) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!("%p.\n", iface);
    return IMFAttributes_LockStore((*node).attributes);
}

unsafe extern "system" fn topology_node_UnlockStore(iface: *mut IMFTopologyNode) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_UnlockStore((*node).attributes);
}
unsafe extern "system" fn topology_node_GetCount(iface: *mut IMFTopologyNode, count: *mut UINT32) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!("%p, %p.\n", iface, count);
    return IMFAttributes_GetCount((*node).attributes, count);
}

unsafe extern "system" fn topology_node_GetItemByIndex(iface: *mut IMFTopologyNode, index: UINT32, key: *mut GUID, value: *mut PROPVARIANT) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!("%p, %u, %p, %p.\n", iface, index, key, value);
    return IMFAttributes_GetItemByIndex((*node).attributes, index, key, value);
}

unsafe extern "system" fn topology_node_CopyAllItems(iface: *mut IMFTopologyNode, dest: *mut IMFAttributes) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return IMFAttributes_CopyAllItems((*node).attributes, dest);
}

unsafe extern "C" fn topology_node_set_object(
    mut node: *mut topology_node,
    mut object: *mut IUnknown,
) -> HRESULT {
    static mut iids: [*const GUID; 3] = unsafe {
        [
            &raw const IID_IPersist,
            &raw const IID_IPersistStorage,
            &raw const IID_IPersistPropertyBag,
        ]
    };
    let mut persist: *mut IPersist = core::ptr::null_mut::<IPersist>();
    let mut has_object_id: BOOL = 0;
    let mut object_id: GUID = core::mem::zeroed::<GUID>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    has_object_id = (IMFAttributes_GetGUID(
        (*node).attributes,
        &raw const MF_TOPONODE_TRANSFORM_OBJECTID,
        &raw mut object_id,
    ) == S_OK) as BOOL;

    if !object.is_null() && has_object_id == 0 {
        i = 0 as ::core::ffi::c_uint;
        while (i as usize) < ARRAY_SIZE!(iids) as usize {
            persist = core::ptr::null_mut::<IPersist>();
            hr = IUnknown_QueryInterface(
                object,
                iids[i as usize],
                &raw mut persist as *mut *mut ::core::ffi::c_void,
            );
            if hr >= 0 {
                break;
            }
            i = i.wrapping_add(1);
        }

        if !persist.is_null() {
            hr = IPersist_GetClassID(persist, &raw mut object_id);
            if hr < 0 {
                IPersist_Release(persist);
                persist = core::ptr::null_mut::<IPersist>();
            }
        }
    }

    EnterCriticalSection(&raw mut (*node).cs);

    if !(*node).object.is_null() {
        IUnknown_Release((*node).object);
    }
    (*node).object = object;
    if !(*node).object.is_null() {
        IUnknown_AddRef((*node).object);
    }

    if !persist.is_null() {
        IMFAttributes_SetGUID(
            (*node).attributes,
            &raw const MF_TOPONODE_TRANSFORM_OBJECTID,
            &raw mut object_id,
        );
    }

    LeaveCriticalSection(&raw mut (*node).cs);

    if !persist.is_null() {
        IPersist_Release(persist);
    }

    return S_OK;
}

unsafe extern "system" fn topology_node_SetObject(iface: *mut IMFTopologyNode, object: *mut IUnknown) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    return topology_node_set_object(node, object);
}

unsafe extern "system" fn topology_node_GetObject(iface: *mut IMFTopologyNode, object: *mut *mut IUnknown) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    if object.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    *object = (*node).object;
    if !(*object).is_null() {
        IUnknown_AddRef(*object);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return if !(*object).is_null() { S_OK } else { E_FAIL };
}

unsafe extern "system" fn topology_node_GetNodeType(iface: *mut IMFTopologyNode, node_type: *mut MF_TOPOLOGY_TYPE) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    *node_type = (*node).node_type;

    return S_OK;
}

unsafe extern "system" fn topology_node_GetTopoNodeID(iface: *mut IMFTopologyNode, id: *mut TOPOID) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);

    *id = (*node).id;

    return S_OK;
}

unsafe extern "system" fn topology_node_SetTopoNodeID(iface: *mut IMFTopologyNode, id: TOPOID) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    TRACE!("%p, %s.\n", iface, wine_dbgstr_longlong(id as u64));

    (*node).id = id;

    return S_OK;
}
unsafe extern "system" fn topology_node_GetInputCount(iface: *mut IMFTopologyNode, count: *mut DWORD) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if ((*__wine_dbch___default).flags as ::core::ffi::c_int & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_GetInputCount\0" as *const u8 as *const ::core::ffi::c_char,
            1365 as ::core::ffi::c_int,
            b"%p, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            count,
        );
    }
    *count = (*node).inputs.count as DWORD;
    return S_OK;
}

unsafe extern "system" fn topology_node_GetOutputCount(iface: *mut IMFTopologyNode, count: *mut DWORD) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    *count = (*node).outputs.count as DWORD;
    return S_OK;
}

unsafe extern "C" fn topology_node_set_stream_type(stream: *mut node_stream, mediatype: *mut IMFMediaType) {
    if !(*stream).preferred_type.is_null() {
        IMFMediaType_Release((*stream).preferred_type);
    }
    (*stream).preferred_type = mediatype;
    if !(*stream).preferred_type.is_null() {
        IMFMediaType_AddRef((*stream).preferred_type);
    }
}

unsafe extern "C" fn topology_node_connect_output(
    mut node: *mut topology_node,
    mut output_index: DWORD,
    mut connection: *mut topology_node,
    mut input_index: DWORD,
) -> HRESULT {
    let mut stream: *mut node_stream = ::core::ptr::null_mut::<node_stream>();
    let mut hr: HRESULT = 0;
    if (*node).node_type as ::core::ffi::c_uint
        == MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*connection).node_type as ::core::ffi::c_uint
            == MF_TOPOLOGY_SOURCESTREAM_NODE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        return E_FAIL;
    }
    EnterCriticalSection(&raw mut (*node).cs);
    EnterCriticalSection(&raw mut (*connection).cs);
    topology_node_disconnect_output(node, output_index);
    if (input_index as size_t) < (*connection).inputs.count {
        stream = (*connection).inputs.streams.offset(input_index as isize) as *mut node_stream;
        if !(*stream).connection.is_null() {
            topology_node_disconnect_output(
                (*stream).connection as *mut topology_node,
                (*stream).connection_stream,
            );
        }
    }
    hr = topology_node_reserve_streams(&raw mut (*node).outputs, output_index);
    if hr >= 0 as HRESULT {
        let mut old_count: size_t = (*connection).inputs.count;
        hr = topology_node_reserve_streams(&raw mut (*connection).inputs, input_index);
        if hr >= 0 as HRESULT
            && old_count == 0
            && !(*connection).input_type.is_null()
        {
            topology_node_set_stream_type((*connection).inputs.streams, (*connection).input_type);
            IMFMediaType_Release((*connection).input_type);
            (*connection).input_type = ::core::ptr::null_mut::<IMFMediaType>();
        }
    }
    if hr >= 0 as HRESULT {
        let ref mut fresh20 = (*(*node).outputs.streams.offset(output_index as isize)).connection;
        *fresh20 = connection as *mut topology_node;
        IMFTopologyNode_AddRef(
            &raw mut (*(*(*node).outputs.streams.offset(output_index as isize)).connection)
                .IMFTopologyNode_iface,
        );
        (*(*node).outputs.streams.offset(output_index as isize)).connection_stream = input_index;
        let ref mut fresh21 =
            (*(*connection).inputs.streams.offset(input_index as isize)).connection;
        *fresh21 = node as *mut topology_node;
        IMFTopologyNode_AddRef(
            &raw mut (*(*(*connection).inputs.streams.offset(input_index as isize)).connection)
                .IMFTopologyNode_iface,
        );
        (*(*connection).inputs.streams.offset(input_index as isize)).connection_stream =
            output_index;
    }
    LeaveCriticalSection(&raw mut (*connection).cs);
    LeaveCriticalSection(&raw mut (*node).cs);
    return hr;
}

unsafe extern "system" fn topology_node_ConnectOutput(iface: *mut IMFTopologyNode, output_index: DWORD, peer: *mut IMFTopologyNode, input_index: DWORD) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let connection: *mut topology_node = unsafe_impl_from_IMFTopologyNode(peer);
    if connection.is_null() {
        WARN!("External node implementations are not supported.\n");
        return E_UNEXPECTED;
    }
    return topology_node_connect_output(node, output_index, connection, input_index);
}

unsafe extern "system" fn topology_node_DisconnectOutput(iface: *mut IMFTopologyNode, output_index: DWORD) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_DisconnectOutput\0" as *const u8 as *const ::core::ffi::c_char,
            1462 as ::core::ffi::c_int,
            b"%p, %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            output_index,
        );
    }
    return topology_node_disconnect_output(node, output_index);
}

unsafe extern "system" fn topology_node_GetInput(iface: *mut IMFTopologyNode, input_index: DWORD, ret: *mut *mut IMFTopologyNode, output_index: *mut DWORD) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if (input_index as usize) < (*node).inputs.count {
        let stream: *const node_stream = (*node).inputs.streams.add(input_index as usize);

        if !(*stream).connection.is_null() {
            *ret = core::ptr::addr_of_mut!((*(*stream).connection).IMFTopologyNode_iface);
            IMFTopologyNode_AddRef(*ret);
            *output_index = (*stream).connection_stream;
        } else {
            hr = MF_E_NOT_FOUND;
        }
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return hr;
}

unsafe extern "system" fn topology_node_GetOutput(iface: *mut IMFTopologyNode, output_index: DWORD, ret: *mut *mut IMFTopologyNode, input_index: *mut DWORD) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if (output_index as usize) < (*node).outputs.count {
        let stream: *const node_stream = (*node).outputs.streams.add(output_index as usize);

        if !(*stream).connection.is_null() {
            *ret = core::ptr::addr_of_mut!((*(*stream).connection).IMFTopologyNode_iface);
            IMFTopologyNode_AddRef(*ret);
            *input_index = (*stream).connection_stream;
        } else {
            hr = MF_E_NOT_FOUND;
        }
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return hr;
}
unsafe extern "system" fn topology_node_SetOutputPrefType(iface: *mut IMFTopologyNode, index: DWORD, mediatype: *mut IMFMediaType) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(&raw mut (*node).cs);

    if (*node).node_type != MF_TOPOLOGY_OUTPUT_NODE {
        hr = topology_node_reserve_streams(&raw mut (*node).outputs, index);
        if hr >= 0 {
            topology_node_set_stream_type((*node).outputs.streams.offset(index as isize) as *mut node_stream, mediatype);
        }
    } else {
        hr = E_NOTIMPL;
    }

    LeaveCriticalSection(&raw mut (*node).cs);

    return hr;
}

unsafe extern "C" fn topology_node_get_pref_type(streams: *mut node_streams, index: UINT32, mediatype: *mut *mut IMFMediaType) -> HRESULT {
    *mediatype = (*(*streams).streams.add(index as usize)).preferred_type;
    if !(*mediatype).is_null() {
        IMFMediaType_AddRef(*mediatype);
        return S_OK;
    }

    return E_FAIL;
}

unsafe extern "system" fn topology_node_GetOutputPrefType(iface: *mut IMFTopologyNode, index: DWORD, mediatype: *mut *mut IMFMediaType) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if (index as usize) < (*node).outputs.count {
        hr = topology_node_get_pref_type(core::ptr::addr_of_mut!((*node).outputs), index as u32, mediatype);
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return hr;
}
unsafe extern "system" fn topology_node_SetInputPrefType(mut iface: *mut IMFTopologyNode, mut index: DWORD, mut mediatype: *mut IMFMediaType) -> HRESULT {
    let mut node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology.c\0" as *const u8 as *const ::core::ffi::c_char,
            b"topology_node_SetInputPrefType\0" as *const u8 as *const ::core::ffi::c_char,
            1587 as ::core::ffi::c_int,
            b"%p, %lu, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            index,
            mediatype,
        );
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    match (*node).node_type as ::core::ffi::c_uint {
        MF_TOPOLOGY_TEE_NODE => {
            if index != 0 {
                hr = MF_E_INVALIDTYPE;
            } else if (*node).inputs.count != 0 {
                topology_node_set_stream_type(
                    (*node).inputs.streams.offset(index as isize) as *mut node_stream,
                    mediatype,
                );
            } else {
                if !(*node).input_type.is_null() {
                    IMFMediaType_Release((*node).input_type);
                }
                (*node).input_type = mediatype;
                if !(*node).input_type.is_null() {
                    IMFMediaType_AddRef((*node).input_type);
                }
            }
        }
        MF_TOPOLOGY_SOURCESTREAM_NODE => {
            hr = E_NOTIMPL;
        }
        _ => {
            if ({
                hr = topology_node_reserve_streams(core::ptr::addr_of_mut!((*node).inputs), index);
                hr
            }) >= 0
            {
                topology_node_set_stream_type(
                    (*node).inputs.streams.offset(index as isize) as *mut node_stream,
                    mediatype,
                );
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return hr;
}

unsafe extern "system" fn topology_node_GetInputPrefType(mut iface: *mut IMFTopologyNode, mut index: DWORD, mut mediatype: *mut *mut IMFMediaType) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if (index as usize) < (*node).inputs.count {
        hr = topology_node_get_pref_type(core::ptr::addr_of_mut!((*node).inputs), index, mediatype);
    } else if (*node).node_type as u32 == MF_TOPOLOGY_TEE_NODE && !(*node).input_type.is_null() {
        *mediatype = (*node).input_type;
        IMFMediaType_AddRef(*mediatype);
    } else {
        hr = E_INVALIDARG;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    return hr;
}

unsafe extern "system" fn topology_node_CloneFrom(iface: *mut IMFTopologyNode, src_node: *mut IMFTopologyNode) -> HRESULT {
    let node: *mut topology_node = impl_from_IMFTopologyNode(iface);
    let mut mediatype: *mut IMFMediaType = core::ptr::null_mut();
    let mut object: *mut IUnknown;
    let mut count: DWORD = 0;
    let mut i: DWORD = 0;
    let mut topoid: TOPOID = 0;
    let mut hr: HRESULT = 0;

    if (*node).node_type != topology_node_get_type(src_node) {
        return MF_E_INVALIDREQUEST;
    }

    hr = IMFTopologyNode_GetTopoNodeID(src_node, core::ptr::addr_of_mut!(topoid));
    if hr < 0 {
        return hr;
    }

    object = core::ptr::null_mut();
    IMFTopologyNode_GetObject(src_node, core::ptr::addr_of_mut!(object));

    EnterCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    hr = IMFTopologyNode_CopyAllItems(src_node, (*node).attributes);

    if hr >= 0 {
        hr = topology_node_set_object(node, object);
    }

    if hr >= 0 {
        (*node).id = topoid;
    }

    if IMFTopologyNode_GetInputCount(src_node, core::ptr::addr_of_mut!(count)) >= 0 {
        i = 0;
        while i < count {
            if IMFTopologyNode_GetInputPrefType(src_node, i, core::ptr::addr_of_mut!(mediatype)) >= 0 {
                IMFTopologyNode_SetInputPrefType(iface, i, mediatype);
                IMFMediaType_Release(mediatype);
            }
            i += 1;
        }
    }

    if IMFTopologyNode_GetOutputCount(src_node, core::ptr::addr_of_mut!(count)) >= 0 {
        i = 0;
        while i < count {
            if IMFTopologyNode_GetOutputPrefType(src_node, i, core::ptr::addr_of_mut!(mediatype)) >= 0 {
                IMFTopologyNode_SetOutputPrefType(iface, i, mediatype);
                IMFMediaType_Release(mediatype);
            }
            i += 1;
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*node).cs));

    if !object.is_null() {
        IUnknown_Release(object);
    }

    hr
}

unsafe extern "C" fn create_topology_node(
    mut node_type: MF_TOPOLOGY_TYPE,
    mut node: *mut *mut topology_node,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    *node = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<topology_node>() as size_t) as _,
    ) as *mut topology_node;
    if (*node).is_null() {
        return E_OUTOFMEMORY;
    }
    (**node).IMFTopologyNode_iface.lpVtbl = &raw const topologynodevtbl;
    (**node).refcount = 1 as LONG;
    (**node).node_type = node_type;
    hr = MFCreateAttributes(&raw mut (**node).attributes, 0 as UINT32);
    if hr < 0 as HRESULT {
        free(*node as *mut ::core::ffi::c_void);
        return hr;
    }
    (**node).id = (GetCurrentProcessId() as TOPOID) << 32
        | InterlockedIncrement(&raw mut next_node_id as *mut ::core::ffi::c_long)
            as TOPOID;
    InitializeCriticalSection(&raw mut (**node).cs);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_get_type(node: *mut IMFTopologyNode) -> MF_TOPOLOGY_TYPE {
    let mut type_: MF_TOPOLOGY_TYPE = core::mem::zeroed();
    let mut hr: HRESULT;

    if {
        hr = IMFTopologyNode_GetNodeType(node, core::ptr::addr_of_mut!(type_));
        hr
    } < 0
    {
        ERR!("Failed to get node type, hr %#lx.\n", hr);
        return (1 as u32).wrapping_neg() as MF_TOPOLOGY_TYPE;
    }

    return type_;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_get_object(node: *mut IMFTopologyNode, riid: *const IID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    let mut unk: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    *obj = core::ptr::null_mut();

    if {
        hr = IMFTopologyNode_GetObject(node, core::ptr::addr_of_mut!(unk));
        hr
    } >= 0
    {
        hr = IUnknown_QueryInterface(unk, riid, obj);
        IUnknown_Release(unk);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTopologyNode(node_type: MF_TOPOLOGY_TYPE, node: *mut *mut IMFTopologyNode) -> HRESULT {
    let mut object: *mut topology_node = core::ptr::null_mut();
    let mut hr: HRESULT;

    if node.is_null() {
        return E_POINTER;
    }

    hr = create_topology_node(node_type, core::ptr::addr_of_mut!(object));
    if hr >= 0 {
        *node = core::ptr::addr_of_mut!((*object).IMFTopologyNode_iface);
    }

    return hr;
}

unsafe extern "C" fn impl_from_IMFMediaTypeHandler(iface: *mut IMFMediaTypeHandler) -> *mut type_handler {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(type_handler, IMFMediaTypeHandler_iface)) as *mut type_handler;
}

unsafe extern "system" fn type_handler_QueryInterface(iface: *mut IMFMediaTypeHandler, riid: *const GUID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaTypeHandler)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        IMFMediaTypeHandler_AddRef({ *obj = iface as *mut ::core::ffi::c_void; *obj as *mut IMFMediaTypeHandler });
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn type_handler_AddRef(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*handler).refcount) as *mut LONG) as ULONG;
    return refcount;
}

unsafe extern "system" fn type_handler_Release(iface: *mut IMFMediaTypeHandler) -> ULONG {
    let handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*handler).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        if !(*handler).transform.is_null() {
            IMFTransform_Release((*handler).transform);
        }
        IMFTopologyNode_Release((*handler).node);
        free(handler as *mut ::core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn type_handler_IsMediaTypeSupported(iface: *mut IMFMediaTypeHandler, in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT {
    let handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);
    let mut type_0: *mut IMFMediaType = core::ptr::null_mut();
    let mut flags: DWORD = 0;
    let mut hr: HRESULT;

    if !out_type.is_null() {
        *out_type = core::ptr::null_mut();
    }

    if !(*handler).transform.is_null() {
        if (*handler).output != 0 {
            return IMFTransform_SetOutputType((*handler).transform, (*handler).stream, in_type, MFT_SET_TYPE_TEST_ONLY as DWORD);
        } else {
            return IMFTransform_SetInputType((*handler).transform, (*handler).stream, in_type, MFT_SET_TYPE_TEST_ONLY as DWORD);
        }
    }

    hr = IMFMediaTypeHandler_GetCurrentMediaType(iface, core::ptr::addr_of_mut!(type_0));
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaType_IsEqual(type_0, in_type, core::ptr::addr_of_mut!(flags));
    IMFMediaType_Release(type_0);
    return hr;
}
unsafe extern "system" fn type_handler_GetMediaTypeCount(
    iface: *mut IMFMediaTypeHandler,
    count: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn type_handler_GetMediaTypeByIndex(iface: *mut IMFMediaTypeHandler, index: DWORD, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);

    if !(*handler).transform.is_null() {
        if (*handler).output != 0 {
            return IMFTransform_GetOutputAvailableType((*handler).transform, (*handler).stream, index, type_0);
        } else {
            return IMFTransform_GetInputAvailableType((*handler).transform, (*handler).stream, index, type_0);
        }
    }

    if index != 0 {
        return MF_E_NO_MORE_TYPES;
    }

    return IMFMediaTypeHandler_GetCurrentMediaType(iface, type_0);
}

unsafe extern "system" fn type_handler_SetCurrentMediaType(iface: *mut IMFMediaTypeHandler, type_: *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);

    if !(*handler).transform.is_null() {
        if (*handler).output != 0 {
            return IMFTransform_SetOutputType((*handler).transform, (*handler).stream, type_, 0 as DWORD);
        } else {
            return IMFTransform_SetInputType((*handler).transform, (*handler).stream, type_, 0 as DWORD);
        }
    }

    return IMFTopologyNode_SetInputPrefType((*handler).node, (*handler).stream, type_);
}

unsafe extern "system" fn type_handler_GetCurrentMediaType(iface: *mut IMFMediaTypeHandler, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut type_handler = impl_from_IMFMediaTypeHandler(iface);
    let mut output: UINT32 = 0;

    if !(*handler).transform.is_null() {
        if (*handler).output != 0 {
            return IMFTransform_GetOutputCurrentType((*handler).transform, (*handler).stream, type_0);
        } else {
            return IMFTransform_GetInputCurrentType((*handler).transform, (*handler).stream, type_0);
        }
    }

    if IMFTopologyNode_GetInputPrefType((*handler).node, 0, type_0) >= 0 {
        return S_OK;
    }

    if IMFTopologyNode_GetUINT32((*handler).node, &raw const MF_TOPONODE_PRIMARYOUTPUT, &raw mut output) < 0 {
        output = 0;
    }

    return IMFTopologyNode_GetOutputPrefType((*handler).node, output as DWORD, type_0);
}

unsafe extern "system" fn type_handler_GetMajorType(iface: *mut IMFMediaTypeHandler, type_: *mut GUID) -> HRESULT {
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IMFMediaTypeHandler_GetCurrentMediaType(iface, core::ptr::addr_of_mut!(media_type));
    if hr < 0 as HRESULT {
        return hr;
    }

    hr = IMFMediaType_GetMajorType(media_type, type_);
    IMFMediaType_Release(media_type);
    return hr;
}

unsafe extern "C" fn type_handler_create(node: *mut IMFTopologyNode, stream: DWORD, output: BOOL, transform: *mut IMFTransform, out: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let mut handler: *mut type_handler = core::ptr::null_mut();

    handler = calloc((1 as size_t) as _, (core::mem::size_of::<type_handler>() as size_t) as _) as *mut type_handler;
    if handler.is_null() {
        return E_OUTOFMEMORY;
    }
    (*handler).IMFMediaTypeHandler_iface.lpVtbl = core::ptr::addr_of!(type_handler_vtbl);
    (*handler).refcount = 1 as LONG;
    (*handler).stream = stream;
    (*handler).output = output;
    (*handler).node = node;
    IMFTopologyNode_AddRef((*handler).node);
    if !transform.is_null() {
        (*handler).transform = transform;
        IMFTransform_AddRef((*handler).transform);
    }

    *out = core::ptr::addr_of_mut!((*handler).IMFMediaTypeHandler_iface);
    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_get_type_handler(node: *mut IMFTopologyNode, stream: DWORD, output: BOOL, handler: *mut *mut IMFMediaTypeHandler) -> HRESULT {
    let mut node_type: MF_TOPOLOGY_TYPE = MF_TOPOLOGY_OUTPUT_NODE;
    let mut stream_sink: *mut IMFStreamSink = core::ptr::null_mut::<IMFStreamSink>();
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut::<IMFStreamDescriptor>();
    let mut transform: *mut IMFTransform = core::ptr::null_mut::<IMFTransform>();
    let mut hr: HRESULT = 0;

    node_type = topology_node_get_type(node);
    match node_type as u32 {
        MF_TOPOLOGY_OUTPUT_NODE => {
            if output != 0 || stream != 0 {
                return MF_E_INVALIDSTREAMNUMBER;
            }
            hr = topology_node_get_object(
                node,
                core::ptr::addr_of!(IID_IMFStreamSink),
                core::ptr::addr_of_mut!(stream_sink) as *mut *mut core::ffi::c_void,
            );
            if hr >= 0 {
                hr = IMFStreamSink_GetMediaTypeHandler(stream_sink, handler);
                IMFStreamSink_Release(stream_sink);
            }
        }
        MF_TOPOLOGY_SOURCESTREAM_NODE => {
            if output == 0 || stream != 0 {
                return MF_E_INVALIDSTREAMNUMBER;
            }
            hr = IMFTopologyNode_GetUnknown(
                node,
                core::ptr::addr_of!(MF_TOPONODE_STREAM_DESCRIPTOR),
                core::ptr::addr_of!(IID_IMFStreamDescriptor),
                core::ptr::addr_of_mut!(sd) as *mut LPVOID,
            );
            if hr >= 0 {
                hr = IMFStreamDescriptor_GetMediaTypeHandler(sd, handler);
                IMFStreamDescriptor_Release(sd);
            }
        }
        MF_TOPOLOGY_TRANSFORM_NODE => {
            hr = topology_node_get_object(
                node,
                core::ptr::addr_of!(IID_IMFTransform),
                core::ptr::addr_of_mut!(transform) as *mut *mut core::ffi::c_void,
            );
            if hr >= 0 {
                hr = type_handler_create(node, stream, output, transform, handler);
                IMFTransform_Release(transform);
            }
        }
        MF_TOPOLOGY_TEE_NODE => {
            hr = type_handler_create(
                node,
                stream,
                output,
                core::ptr::null_mut::<IMFTransform>(),
                handler,
            );
        }
        _ => {
            WARN!("Unexpected node type %u.\n", node_type);
            return MF_E_UNEXPECTED;
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFGetTopoNodeCurrentType(node: *mut IMFTopologyNode, stream: DWORD, output: BOOL, type_: *mut *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = topology_node_get_type_handler(node, stream, output, core::ptr::addr_of_mut!(handler));
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaTypeHandler_GetCurrentMediaType(handler, type_);
    IMFMediaTypeHandler_Release(handler);
    return hr;
}

unsafe extern "system" fn seq_source_QueryInterface(iface: *mut IMFSequencerSource, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let seq_source: *mut seq_source = impl_from_IMFSequencerSource(iface);

    *out = core::ptr::null_mut();

    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFSequencerSource)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = core::ptr::addr_of_mut!((*seq_source).IMFSequencerSource_iface) as *mut core::ffi::c_void;
    } else if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFMediaSourceTopologyProvider)) != 0 {
        *out = core::ptr::addr_of_mut!((*seq_source).IMFMediaSourceTopologyProvider_iface) as *mut core::ffi::c_void;
    } else {
        WARN!("Unimplemented %s.\n", debugstr_guid(riid));
        return E_NOINTERFACE;
    }

    if !(*out).is_null() {
        IUnknown_AddRef(*out as *mut IUnknown);
    }

    return S_OK;
}

unsafe extern "system" fn seq_source_AddRef(iface: *mut IMFSequencerSource) -> ULONG {
    let seq_source: *mut seq_source = impl_from_IMFSequencerSource(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*seq_source).refcount) as *mut i32) as ULONG;

    return refcount;
}

unsafe extern "system" fn seq_source_Release(iface: *mut IMFSequencerSource) -> ULONG {
    let seq_source: *mut seq_source = impl_from_IMFSequencerSource(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*seq_source).refcount) as *mut LONG,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        free(seq_source as *mut core::ffi::c_void);
    }

    return refcount;
}
unsafe extern "system" fn seq_source_AppendTopology(
    iface: *mut IMFSequencerSource,
    topology: *mut IMFTopology,
    flags: DWORD,
    id: *mut MFSequencerElementId,
) -> HRESULT {
    // FIXME("%p, %p, %lx, %p.\n", iface, topology, flags, id);
    E_NOTIMPL
}
unsafe extern "system" fn seq_source_DeleteTopology(
    iface: *mut IMFSequencerSource,
    id: MFSequencerElementId,
) -> HRESULT {
    // FIXME("%p, %#lx.\n", iface, id);
    E_NOTIMPL
}
unsafe extern "system" fn seq_source_GetPresentationContext(
    iface: *mut IMFSequencerSource,
    descriptor: *mut IMFPresentationDescriptor,
    id: *mut MFSequencerElementId,
    topology: *mut *mut IMFTopology,
) -> HRESULT {
    // FIXME("%p, %p, %p, %p.\n", iface, descriptor, id, topology);
    E_NOTIMPL
}
unsafe extern "system" fn seq_source_UpdateTopology(
    iface: *mut IMFSequencerSource,
    id: MFSequencerElementId,
    topology: *mut IMFTopology,
) -> HRESULT {
    // FIXME("%p, %#lx, %p.\n", iface, id, topology);
    E_NOTIMPL
}
unsafe extern "system" fn seq_source_UpdateTopologyFlags(
    iface: *mut IMFSequencerSource,
    id: MFSequencerElementId,
    flags: DWORD,
) -> HRESULT {
    // FIXME("%p, %#lx, %#lx.\n", iface, id, flags);
    E_NOTIMPL
}

unsafe extern "system" fn seq_source_topology_provider_QueryInterface(iface: *mut IMFMediaSourceTopologyProvider, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let seq_source: *mut seq_source = impl_from_IMFMediaSourceTopologyProvider(iface);
    return IMFSequencerSource_QueryInterface(core::ptr::addr_of_mut!((*seq_source).IMFSequencerSource_iface), riid, obj);
}

unsafe extern "system" fn seq_source_topology_provider_AddRef(
    iface: *mut IMFMediaSourceTopologyProvider,
) -> ULONG {
    let seq_source: *mut seq_source = impl_from_IMFMediaSourceTopologyProvider(iface);
    return IMFSequencerSource_AddRef(core::ptr::addr_of_mut!((*seq_source).IMFSequencerSource_iface));
}

unsafe extern "system" fn seq_source_topology_provider_Release(
    iface: *mut IMFMediaSourceTopologyProvider,
) -> ULONG {
    let seq_source: *mut seq_source = impl_from_IMFMediaSourceTopologyProvider(iface);
    return IMFSequencerSource_Release(core::ptr::addr_of_mut!((*seq_source).IMFSequencerSource_iface));
}
unsafe extern "system" fn seq_source_topology_provider_GetMediaSourceTopology(
    iface: *mut IMFMediaSourceTopologyProvider,
    pd: *mut IMFPresentationDescriptor,
    topology: *mut *mut IMFTopology,
) -> HRESULT {
    // FIXME("%p, %p, %p.\n", iface, pd, topology);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSequencerSource(reserved: *mut IUnknown, seq_source: *mut *mut IMFSequencerSource) -> HRESULT {
    let mut object: *mut seq_source = core::ptr::null_mut();

    if seq_source.is_null() {
        return E_POINTER;
    }

    object = calloc(1, (core::mem::size_of::<seq_source>()) as _) as *mut seq_source;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFSequencerSource_iface.lpVtbl = &raw const seqsourcevtbl;
    (*object).IMFMediaSourceTopologyProvider_iface.lpVtbl = &raw const seq_source_topology_provider_vtbl;
    (*object).refcount = 1;

    *seq_source = &raw mut (*object).IMFSequencerSource_iface;

    return S_OK;
}

unsafe extern "C" fn impl_offset_from_IUnknown(iface: *mut IUnknown) -> *mut segment_offset {
    return (iface as *mut u8).sub(core::mem::offset_of!(segment_offset, IUnknown_iface)) as *mut segment_offset;
}

unsafe extern "system" fn segment_offset_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn segment_offset_AddRef(iface: *mut IUnknown) -> ULONG {
    let offset: *mut segment_offset = impl_offset_from_IUnknown(iface);
    return InterlockedIncrement(core::ptr::addr_of_mut!((*offset).refcount)) as ULONG;
}
unsafe extern "system" fn segment_offset_Release(iface: *mut IUnknown) -> ULONG {
    let offset: *mut segment_offset = impl_offset_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*offset).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        free(offset as *mut ::core::ffi::c_void);
    }

    return refcount;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateSequencerSegmentOffset(
    mut id: MFSequencerElementId,
    mut timeoffset: MFTIME,
    mut ret: *mut PROPVARIANT,
) -> HRESULT {
    let mut offset: *mut segment_offset = core::ptr::null_mut::<segment_offset>();
    TRACE!(
        "%#lx, %s, %p.\n",
        id,
        debugstr_time(timeoffset as LONGLONG),
        ret
    );
    if ret.is_null() {
        return E_POINTER;
    }
    offset = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<segment_offset>() as size_t) as _,
    ) as *mut segment_offset;
    if offset.is_null() {
        return E_OUTOFMEMORY;
    }
    (*offset).IUnknown_iface.lpVtbl = core::ptr::addr_of!(segment_offset_vtbl);
    (*offset).refcount = 1 as LONG;
    (*offset).id = id;
    (*offset).timeoffset = timeoffset;

    /* ReactOS: our PROPVARIANT has no n1 union, so the VARIANT V_ macros don't apply. */
    (*ret).vt = VT_UNKNOWN as VARTYPE;
    (*ret).__bindgen_anon_1.punkVal = core::ptr::addr_of_mut!((*offset).IUnknown_iface);

    return S_OK;
}

// COM vtable tables — generated from topology.c by vtbl_gen.py
/// topology.c:806 `static const IMFTopologyVtbl topologyvtbl`  (slot order: rust:types_amd64.rs)
pub static topologyvtbl: IMFTopologyVtbl = IMFTopologyVtbl {
    QueryInterface: Some(topology_QueryInterface),
    AddRef: Some(topology_AddRef),
    Release: Some(topology_Release),
    GetItem: Some(topology_GetItem),
    GetItemType: Some(topology_GetItemType),
    CompareItem: Some(topology_CompareItem),
    Compare: Some(topology_Compare),
    GetUINT32: Some(topology_GetUINT32),
    GetUINT64: Some(topology_GetUINT64),
    GetDouble: Some(topology_GetDouble),
    GetGUID: Some(topology_GetGUID),
    GetStringLength: Some(topology_GetStringLength),
    GetString: Some(topology_GetString),
    GetAllocatedString: Some(topology_GetAllocatedString),
    GetBlobSize: Some(topology_GetBlobSize),
    GetBlob: Some(topology_GetBlob),
    GetAllocatedBlob: Some(topology_GetAllocatedBlob),
    GetUnknown: Some(topology_GetUnknown),
    SetItem: Some(topology_SetItem),
    DeleteItem: Some(topology_DeleteItem),
    DeleteAllItems: Some(topology_DeleteAllItems),
    SetUINT32: Some(topology_SetUINT32),
    SetUINT64: Some(topology_SetUINT64),
    SetDouble: Some(topology_SetDouble),
    SetGUID: Some(topology_SetGUID),
    SetString: Some(topology_SetString),
    SetBlob: Some(topology_SetBlob),
    SetUnknown: Some(topology_SetUnknown),
    LockStore: Some(topology_LockStore),
    UnlockStore: Some(topology_UnlockStore),
    GetCount: Some(topology_GetCount),
    GetItemByIndex: Some(topology_GetItemByIndex),
    CopyAllItems: Some(topology_CopyAllItems),
    GetTopologyID: Some(topology_GetTopologyID),
    AddNode: Some(topology_AddNode),
    RemoveNode: Some(topology_RemoveNode),
    GetNodeCount: Some(topology_GetNodeCount),
    GetNode: Some(topology_GetNode),
    Clear: Some(topology_Clear),
    CloneFrom: Some(topology_CloneFrom),
    GetNodeByID: Some(topology_GetNodeByID),
    GetSourceNodeCollection: Some(topology_GetSourceNodeCollection),
    GetOutputNodeCollection: Some(topology_GetOutputNodeCollection),
};

/// topology.c:1711 `static const IMFTopologyNodeVtbl topologynodevtbl`  (slot order: rust:types_amd64.rs)
pub static topologynodevtbl: IMFTopologyNodeVtbl = IMFTopologyNodeVtbl {
    QueryInterface: Some(topology_node_QueryInterface),
    AddRef: Some(topology_node_AddRef),
    Release: Some(topology_node_Release),
    GetItem: Some(topology_node_GetItem),
    GetItemType: Some(topology_node_GetItemType),
    CompareItem: Some(topology_node_CompareItem),
    Compare: Some(topology_node_Compare),
    GetUINT32: Some(topology_node_GetUINT32),
    GetUINT64: Some(topology_node_GetUINT64),
    GetDouble: Some(topology_node_GetDouble),
    GetGUID: Some(topology_node_GetGUID),
    GetStringLength: Some(topology_node_GetStringLength),
    GetString: Some(topology_node_GetString),
    GetAllocatedString: Some(topology_node_GetAllocatedString),
    GetBlobSize: Some(topology_node_GetBlobSize),
    GetBlob: Some(topology_node_GetBlob),
    GetAllocatedBlob: Some(topology_node_GetAllocatedBlob),
    GetUnknown: Some(topology_node_GetUnknown),
    SetItem: Some(topology_node_SetItem),
    DeleteItem: Some(topology_node_DeleteItem),
    DeleteAllItems: Some(topology_node_DeleteAllItems),
    SetUINT32: Some(topology_node_SetUINT32),
    SetUINT64: Some(topology_node_SetUINT64),
    SetDouble: Some(topology_node_SetDouble),
    SetGUID: Some(topology_node_SetGUID),
    SetString: Some(topology_node_SetString),
    SetBlob: Some(topology_node_SetBlob),
    SetUnknown: Some(topology_node_SetUnknown),
    LockStore: Some(topology_node_LockStore),
    UnlockStore: Some(topology_node_UnlockStore),
    GetCount: Some(topology_node_GetCount),
    GetItemByIndex: Some(topology_node_GetItemByIndex),
    CopyAllItems: Some(topology_node_CopyAllItems),
    SetObject: Some(topology_node_SetObject),
    GetObject: Some(topology_node_GetObject),
    GetNodeType: Some(topology_node_GetNodeType),
    GetTopoNodeID: Some(topology_node_GetTopoNodeID),
    SetTopoNodeID: Some(topology_node_SetTopoNodeID),
    GetInputCount: Some(topology_node_GetInputCount),
    GetOutputCount: Some(topology_node_GetOutputCount),
    ConnectOutput: Some(topology_node_ConnectOutput),
    DisconnectOutput: Some(topology_node_DisconnectOutput),
    GetInput: Some(topology_node_GetInput),
    GetOutput: Some(topology_node_GetOutput),
    SetOutputPrefType: Some(topology_node_SetOutputPrefType),
    GetOutputPrefType: Some(topology_node_GetOutputPrefType),
    SetInputPrefType: Some(topology_node_SetInputPrefType),
    GetInputPrefType: Some(topology_node_GetInputPrefType),
    CloneFrom: Some(topology_node_CloneFrom),
};

/// topology.c:1991 `static const IMFMediaTypeHandlerVtbl type_handler_vtbl`  (slot order: rust:types_amd64.rs)
pub static type_handler_vtbl: IMFMediaTypeHandlerVtbl = IMFMediaTypeHandlerVtbl {
    QueryInterface: Some(type_handler_QueryInterface),
    AddRef: Some(type_handler_AddRef),
    Release: Some(type_handler_Release),
    IsMediaTypeSupported: Some(type_handler_IsMediaTypeSupported),
    GetMediaTypeCount: Some(type_handler_GetMediaTypeCount),
    GetMediaTypeByIndex: Some(type_handler_GetMediaTypeByIndex),
    SetCurrentMediaType: Some(type_handler_SetCurrentMediaType),
    GetCurrentMediaType: Some(type_handler_GetCurrentMediaType),
    GetMajorType: Some(type_handler_GetMajorType),
};

/// topology.c:2206 `static const IMFMediaSourceTopologyProviderVtbl seq_source_topology_provider_vtbl`  (slot order: rust:types_amd64.rs)
pub static seq_source_topology_provider_vtbl: IMFMediaSourceTopologyProviderVtbl = IMFMediaSourceTopologyProviderVtbl {
    QueryInterface: Some(seq_source_topology_provider_QueryInterface),
    AddRef: Some(seq_source_topology_provider_AddRef),
    Release: Some(seq_source_topology_provider_Release),
    GetMediaSourceTopology: Some(seq_source_topology_provider_GetMediaSourceTopology),
};

/// topology.c:2214 `static const IMFSequencerSourceVtbl seqsourcevtbl`  (slot order: rust:types_amd64.rs)
pub static seqsourcevtbl: IMFSequencerSourceVtbl = IMFSequencerSourceVtbl {
    QueryInterface: Some(seq_source_QueryInterface),
    AddRef: Some(seq_source_AddRef),
    Release: Some(seq_source_Release),
    AppendTopology: Some(seq_source_AppendTopology),
    DeleteTopology: Some(seq_source_DeleteTopology),
    GetPresentationContext: Some(seq_source_GetPresentationContext),
    UpdateTopology: Some(seq_source_UpdateTopology),
    UpdateTopologyFlags: Some(seq_source_UpdateTopologyFlags),
};

/// topology.c:2293 `static const IUnknownVtbl segment_offset_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static segment_offset_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(segment_offset_QueryInterface),
    AddRef: Some(segment_offset_AddRef),
    Release: Some(segment_offset_Release),
};
