use crate::mf::copier::MFCreateSampleCopierMFT;
use crate::mf::topology::MFCreateTopologyNode;
use crate::mf::main::MFGetService;
use crate::mf::topology::MFGetTopoNodeCurrentType;
use crate::mf::topology::create_topology;
use crate::mf::main::debugstr_media_type;
use crate::mf::topology::topology;
use crate::mf::topology::topology_node_get_object;
use crate::mf::topology::topology_node_get_type;
use crate::mf::topology::topology_node_get_type_handler;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const CONNECT_DIRECT: i32 = 1;
pub const CONNECT_CONVERTER: i32 = 2;
pub const CONNECT_DECODER: i32 = 4;

#[repr(C)]
pub struct topology_loader {
    pub IMFTopoLoader_iface: IMFTopoLoader,
    pub refcount: LONG,
}

#[repr(C)]
pub struct transform_output_type {
    pub type_: *mut IMFMediaType,
    pub transform: *mut IMFTransform,
    pub activate: *mut IMFActivate,
}

#[repr(C)]
pub struct topology_branch_end {
    pub node: *mut IMFTopologyNode,
    pub stream: DWORD,
    pub handler: *mut IMFMediaTypeHandler,
}

#[repr(C)]
pub struct topology_branch {
    pub up: topology_branch_end,
    pub down: topology_branch_end,
}
unsafe extern "C" fn connect_method_from_mf(mf_method: MF_CONNECT_METHOD) -> connect_method {
    let mut method: connect_method = (CONNECT_DIRECT as u32);
    if ((mf_method as ::core::ffi::c_uint)
        & (MF_CONNECT_ALLOW_CONVERTER as ::core::ffi::c_int as ::core::ffi::c_uint))
        == (MF_CONNECT_ALLOW_CONVERTER as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        method = ::core::mem::transmute::<::core::ffi::c_uint, connect_method>(
            (method as ::core::ffi::c_uint)
                | (CONNECT_CONVERTER as ::core::ffi::c_int as ::core::ffi::c_uint),
        );
    }
    if ((mf_method as ::core::ffi::c_uint)
        & (MF_CONNECT_ALLOW_DECODER as ::core::ffi::c_int as ::core::ffi::c_uint))
        == (MF_CONNECT_ALLOW_DECODER as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        method = ::core::mem::transmute::<::core::ffi::c_uint, connect_method>(
            (method as ::core::ffi::c_uint)
                | (CONNECT_DECODER as ::core::ffi::c_int as ::core::ffi::c_uint),
        );
    }
    return method;
}

unsafe extern "C" fn impl_from_IMFTopoLoader(iface: *mut IMFTopoLoader) -> *mut topology_loader {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(topology_loader, IMFTopoLoader_iface))
        as *mut topology_loader;
}

unsafe extern "system" fn topology_loader_QueryInterface(iface: *mut IMFTopoLoader, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopoLoader)) != 0
        || IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *out = iface as *mut core::ffi::c_void;
        IMFTopoLoader_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn topology_loader_AddRef(iface: *mut IMFTopoLoader) -> ULONG {
    let loader: *mut topology_loader = impl_from_IMFTopoLoader(iface);
    let refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*loader).refcount) as *mut LONG,
    ) as ULONG;
    return refcount;
}

unsafe extern "system" fn topology_loader_Release(iface: *mut IMFTopoLoader) -> ULONG {
    let loader: *mut topology_loader = impl_from_IMFTopoLoader(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*loader).refcount) as *mut LONG) as ULONG;

    TRACE!("iface %p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        free(loader as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn topology_node_transform_unlock_async(mut node: *mut IMFTopologyNode) {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut::<IMFAttributes>();
    attributes = topology_node_transform_async_get_attributes(node);
    if !attributes.is_null() {
        IMFAttributes_SetUINT32(
            attributes,
            core::ptr::addr_of!(MF_TRANSFORM_ASYNC_UNLOCK),
            TRUE as UINT32,
        );
        IMFAttributes_Release(attributes);
    }
}

unsafe extern "C" fn topology_clone_node(topology: *mut IMFTopology, node: *mut IMFTopologyNode, clone: *mut *mut IMFTopologyNode) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut id: TOPOID = 0;

    if (({ hr = IMFTopologyNode_GetTopoNodeID(node, core::ptr::addr_of_mut!(id)); hr }) < 0) {
        return hr;
    }
    if (({ hr = IMFTopology_GetNodeByID(topology, id, clone); hr }) >= 0) {
        return hr;
    }
    if (({ hr = MFCreateTopologyNode(topology_node_get_type(node), clone); hr }) < 0) {
        return hr;
    }

    hr = IMFTopologyNode_CloneFrom(*clone, node);
    if ((hr) >= 0) {
        /* unlock transforms now to enable querying of types */
        if (topology_node_get_type(*clone) == MF_TOPOLOGY_TRANSFORM_NODE) {
            topology_node_transform_unlock_async(*clone);
        }

        hr = IMFTopology_AddNode(topology, *clone);
    }

    if ((hr) < 0) {
        IMFTopologyNode_Release(*clone);
        *clone = core::ptr::null_mut();
    }
    return hr;
}

unsafe extern "C" fn debugstr_topology_type(type_0: MF_TOPOLOGY_TYPE) -> *const i8 {
    match type_0 {
        MF_TOPOLOGY_OUTPUT_NODE => return b"sink\0".as_ptr() as *const i8,
        MF_TOPOLOGY_SOURCESTREAM_NODE => return b"source\0".as_ptr() as *const i8,
        MF_TOPOLOGY_TRANSFORM_NODE => return b"transform\0".as_ptr() as *const i8,
        MF_TOPOLOGY_TEE_NODE => return b"tee\0".as_ptr() as *const i8,
        _ => return b"unknown\0".as_ptr() as *const i8,
    }
}

unsafe extern "C" fn debugstr_topology_branch(branch: *mut topology_branch) -> *const core::ffi::c_char {
    return wine_dbg_sprintf(
        b"%s %p:%lu to %s %p:%lu\0".as_ptr() as *const core::ffi::c_char,
        debugstr_topology_type(topology_node_get_type((*branch).up.node)),
        (*branch).up.node,
        (*branch).up.stream,
        debugstr_topology_type(topology_node_get_type((*branch).down.node)),
        (*branch).down.node,
        (*branch).down.stream,
    );
}

unsafe extern "C" fn topology_branch_create(source: *mut IMFTopologyNode, source_stream: DWORD, sink: *mut IMFTopologyNode, sink_stream: DWORD, out: *mut *mut topology_branch) -> HRESULT {
    let mut source_handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut sink_handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut branch: *mut topology_branch = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = topology_node_get_type_handler(source, source_stream, (TRUE) as BOOL, core::ptr::addr_of_mut!(source_handler));
    if hr < 0 {
        return hr;
    }
    hr = topology_node_get_type_handler(sink, sink_stream, (FALSE) as BOOL, core::ptr::addr_of_mut!(sink_handler));
    if hr < 0 {
        IMFMediaTypeHandler_Release(source_handler);
        return hr;
    }

    branch = calloc(1, (core::mem::size_of::<topology_branch>()) as _) as *mut topology_branch;
    if branch.is_null() {
        IMFMediaTypeHandler_Release(sink_handler);
        IMFMediaTypeHandler_Release(source_handler);
        return E_OUTOFMEMORY;
    }

    (*branch).up.node = source;
    IMFTopologyNode_AddRef((*branch).up.node);
    (*branch).up.stream = source_stream;
    (*branch).up.handler = source_handler;
    (*branch).down.node = sink;
    IMFTopologyNode_AddRef((*branch).down.node);
    (*branch).down.stream = sink_stream;
    (*branch).down.handler = sink_handler;

    *out = branch;
    return S_OK;
}

unsafe extern "C" fn topology_branch_destroy(branch: *mut topology_branch) {
    IMFTopologyNode_Release((*branch).up.node);
    IMFMediaTypeHandler_Release((*branch).up.handler);
    IMFTopologyNode_Release((*branch).down.node);
    IMFMediaTypeHandler_Release((*branch).down.handler);
    free(branch as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn topology_branch_create_indirect(
    branch: *mut topology_branch,
    node: *mut IMFTopologyNode,
    up: *mut *mut topology_branch,
    down: *mut *mut topology_branch,
) -> HRESULT {
    let mut hr: HRESULT;

    hr = topology_branch_create((*branch).up.node, (*branch).up.stream, node, 0 as DWORD, up);
    if hr < 0 as HRESULT {
        return hr;
    }
    hr = topology_branch_create(node, 0 as DWORD, (*branch).down.node, (*branch).down.stream, down);
    if hr < 0 as HRESULT {
        topology_branch_destroy(*up);
    }

    return hr;
}

unsafe extern "C" fn topology_branch_create_for_output(node: *mut IMFTopologyNode, stream: DWORD, out: *mut *mut topology_branch) -> HRESULT {
    let mut down_node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut down_stream: DWORD = 0;
    let mut method: UINT32 = 0;
    let mut hr: HRESULT;

    if (({ hr = IMFTopologyNode_GetOutput(node, stream, core::ptr::addr_of_mut!(down_node), core::ptr::addr_of_mut!(down_stream)); hr }) < 0) {
        return hr;
    }
    while (IMFTopologyNode_GetUINT32(down_node, core::ptr::addr_of!(MF_TOPONODE_CONNECT_METHOD), core::ptr::addr_of_mut!(method)) >= 0)
        && ((method & MF_CONNECT_AS_OPTIONAL as UINT32) != 0)
    {
        let tmp: *mut IMFTopologyNode = down_node;
        hr = IMFTopologyNode_GetOutput(tmp, 0, core::ptr::addr_of_mut!(down_node), core::ptr::addr_of_mut!(down_stream));
        IMFTopologyNode_Release(tmp);
        if hr < 0 {
            return hr;
        }
    }
    hr = topology_branch_create(node, stream, down_node, down_stream, out);
    IMFTopologyNode_Release(down_node);

    return hr;
}

unsafe extern "C" fn topology_branch_create_for_input(node: *mut IMFTopologyNode, stream: DWORD, out: *mut *mut topology_branch) -> HRESULT {
    let mut up_node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut up_stream: DWORD = 0;
    let mut hr: HRESULT;
    if {
        hr = IMFTopologyNode_GetInput(node, stream, core::ptr::addr_of_mut!(up_node), core::ptr::addr_of_mut!(up_stream));
        hr
    } < 0 {
        return hr;
    }
    hr = topology_branch_create(up_node, up_stream, node, stream, out);
    IMFTopologyNode_Release(up_node);
    return hr;
}

unsafe extern "C" fn topology_branch_create_cloned(topology: *mut IMFTopology, up_node: *mut IMFTopologyNode, up_stream: DWORD, down_node: *mut IMFTopologyNode, down_stream: DWORD, out: *mut *mut topology_branch) -> HRESULT {
    let mut up: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut down: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = topology_clone_node(topology, up_node, core::ptr::addr_of_mut!(up));
    if hr < 0 {
        return hr;
    }
    hr = topology_clone_node(topology, down_node, core::ptr::addr_of_mut!(down));
    if hr < 0 {
        IMFTopologyNode_Release(up);
        return hr;
    }

    hr = topology_branch_create(up, up_stream, down, down_stream, out);
    IMFTopologyNode_Release(up);
    IMFTopologyNode_Release(down);
    return hr;
}

unsafe extern "C" fn topology_branch_create_optional(
    mut topology: *mut IMFTopology,
    mut branch: *mut topology_branch,
    mut optional: *mut IMFTopologyNode,
    mut stream: DWORD,
    mut up: *mut *mut topology_branch,
    mut down: *mut *mut topology_branch,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    hr = topology_branch_create_cloned(
        topology,
        (*branch).up.node,
        (*branch).up.stream,
        optional,
        stream,
        up,
    );
    if hr >= 0 {
        hr = topology_branch_create_cloned(
            topology,
            optional,
            0,
            (*branch).down.node,
            (*branch).down.stream,
            down,
        );
        if hr < 0 {
            topology_branch_destroy(*up);
        }
    }
    return hr;
}

unsafe extern "C" fn media_type_try_copy_attr(
    dst: *mut IMFMediaType,
    src: *mut IMFMediaType,
    attr: *const GUID,
    hr: *mut HRESULT,
) {
    let mut value: PROPVARIANT = core::mem::zeroed();

    PropVariantInit!(&raw mut value);
    if (*hr >= 0)
        && (IMFMediaType_GetItem(dst, attr, core::ptr::null_mut::<PROPVARIANT>()) < 0)
        && (IMFMediaType_GetItem(src, attr, &raw mut value) >= 0)
    {
        *hr = IMFMediaType_SetItem(dst, attr, &raw mut value);
    }
    PropVariantClear(&raw mut value);
}

unsafe extern "C" fn update_media_type_from_upstream(
    mut media_type: *mut IMFMediaType,
    mut upstream_type: *mut IMFMediaType,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;

    /* propagate common video attributes */
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_FRAME_SIZE, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_FRAME_RATE, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_DEFAULT_STRIDE, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_VIDEO_ROTATION, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_FIXED_SIZE_SAMPLES, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_PIXEL_ASPECT_RATIO, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_ALL_SAMPLES_INDEPENDENT, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_MINIMUM_DISPLAY_APERTURE, &raw mut hr);

    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_VIDEO_CHROMA_SITING, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_INTERLACE_MODE, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_TRANSFER_FUNCTION, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_VIDEO_PRIMARIES, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_YUV_MATRIX, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_VIDEO_LIGHTING, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_VIDEO_NOMINAL_RANGE, &raw mut hr);

    /* propagate common audio attributes */
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_NUM_CHANNELS, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_BLOCK_ALIGNMENT, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_BITS_PER_SAMPLE, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_SAMPLES_PER_SECOND, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_AVG_BYTES_PER_SECOND, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_CHANNEL_MASK, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_SAMPLES_PER_BLOCK, &raw mut hr);
    media_type_try_copy_attr(media_type, upstream_type, &raw const MF_MT_AUDIO_VALID_BITS_PER_SAMPLE, &raw mut hr);

    return hr;
}

unsafe extern "C" fn clone_media_type(in_type: *mut IMFMediaType, out_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut hr: HRESULT = MFCreateMediaType(out_type);
    if hr < (0 as HRESULT) || {
        hr = IMFMediaType_CopyAllItems(in_type, *out_type as *mut IMFAttributes);
        hr >= (0 as HRESULT)
    } {
        return hr;
    }
    IMFMediaType_Release(*out_type);
    *out_type = core::ptr::null_mut::<IMFMediaType>();
    return hr;
}

unsafe extern "C" fn media_type_handler_get_current(handler: *mut IMFMediaTypeHandler, type_: *mut *mut IMFMediaType) -> HRESULT {
    let mut tmp: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;
    if {
        hr = IMFMediaTypeHandler_GetCurrentMediaType(handler, core::ptr::addr_of_mut!(tmp));
        hr
    } < 0 {
        return hr;
    }
    hr = clone_media_type(tmp, type_);
    IMFMediaType_Release(tmp);
    return hr;
}

unsafe extern "C" fn media_type_handler_get_type(handler: *mut IMFMediaTypeHandler, i: UINT32, type_: *mut *mut IMFMediaType) -> HRESULT {
    let mut tmp: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;
    hr = IMFMediaTypeHandler_GetMediaTypeByIndex(handler, i, core::ptr::addr_of_mut!(tmp));
    if hr < 0 {
        return hr;
    }
    hr = clone_media_type(tmp, type_);
    IMFMediaType_Release(tmp);
    return hr;
}

unsafe extern "C" fn is_better_media_type(type_: *mut IMFMediaType, best: *mut IMFMediaType) -> BOOL {
    let mut guid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    IMFMediaType_GetGUID(type_, &raw const MF_MT_SUBTYPE, &raw mut guid);
    if IsEqualGUID!(&raw mut guid, &raw const MFAudioFormat_Float) != false
        || IsEqualGUID!(&raw mut guid, &raw const MFAudioFormat_PCM) != false
    {
        IMFMediaType_GetGUID(best, &raw const MF_MT_SUBTYPE, &raw mut guid);
        return (IsEqualGUID!(&raw mut guid, &raw const MFAudioFormat_Float) == false) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn topology_branch_find_best_up_type(branch: *mut topology_branch, down_type: *mut IMFMediaType, type_: *mut *mut IMFMediaType) {
    let mut up_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    /* actually set the down type to enforce some transform internal parameters like frame size on the enumerated types */
    hr = IMFMediaTypeHandler_SetCurrentMediaType((*branch).up.handler, down_type);
    if hr < 0 {
        TRACE!("down type %s not supported by upstream, hr %#lx\n", debugstr_media_type(down_type), hr);
        return;
    }

    let mut i: UINT = 0;
    loop {
        hr = media_type_handler_get_type((*branch).up.handler, i, &raw mut up_type);
        if !(hr >= 0) {
            break;
        }

        let mut keep: bool = false;

        hr = update_media_type_from_upstream(up_type, down_type);
        if hr < 0 {
            WARN!("Failed to update media type, hr %#lx\n", hr);
        }

        hr = IMFMediaType_IsEqual(up_type, down_type, &raw mut flags);
        if hr != S_OK {
            TRACE!("up type %s differs from down type %s, hr %#lx flags %#lx\n",
                debugstr_media_type(up_type),
                debugstr_media_type(down_type), hr, flags);
        } else {
            hr = IMFMediaTypeHandler_IsMediaTypeSupported((*branch).down.handler, up_type, core::ptr::null_mut());
            if hr < 0 {
                TRACE!("up type %s not supported by downstream, hr %#lx\n", debugstr_media_type(up_type), hr);
            } else if !(*type_).is_null() && is_better_media_type(up_type, *type_) == 0 {
                TRACE!("up type %s isn't better than selected type %s\n",
                    debugstr_media_type(up_type),
                    debugstr_media_type(*type_));
            } else {
                TRACE!("selecting up type %s\n", debugstr_media_type(up_type));
                if !(*type_).is_null() {
                    IMFMediaType_Release(*type_);
                }
                *type_ = up_type;
                keep = true;
            }
        }

        if !keep {
            IMFMediaType_Release(up_type);
        }

        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn topology_branch_find_best_type(branch: *mut topology_branch, upstream: *mut IMFMediaType, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut enumerate: BOOL = (topology_node_get_type((*branch).down.node) == MF_TOPOLOGY_TRANSFORM_NODE) as BOOL;
    let mut down_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut hr: HRESULT;

    *type_0 = core::ptr::null_mut();

    hr = media_type_handler_get_current((*branch).down.handler, core::ptr::addr_of_mut!(down_type));
    if hr >= 0 {
        hr = update_media_type_from_upstream(down_type, upstream);
        if hr < 0 {
            WARN!("Failed to update media type, hr %#lx\n", hr);
        }
        topology_branch_find_best_up_type(branch, down_type, type_0);
        IMFMediaType_Release(down_type);
    } else if enumerate != 0 || hr == MF_E_NOT_INITIALIZED {
        let mut i: UINT = 0;
        loop {
            hr = media_type_handler_get_type((*branch).down.handler, i as UINT32, core::ptr::addr_of_mut!(down_type));
            if !(hr >= 0) {
                break;
            }
            hr = update_media_type_from_upstream(down_type, upstream);
            if hr < 0 {
                WARN!("Failed to update media type, hr %#lx\n", hr);
            }
            topology_branch_find_best_up_type(branch, down_type, type_0);
            IMFMediaType_Release(down_type);
            i = i.wrapping_add(1);
        }
    }

    if !(*type_0).is_null() { S_OK } else { MF_E_INVALIDMEDIATYPE }
}

unsafe extern "C" fn topology_branch_connect_with_type(topology: *mut IMFTopology, branch: *mut topology_branch, type_: *mut IMFMediaType) -> HRESULT {
    let mut hr: HRESULT = 0;

    if {
        hr = IMFMediaTypeHandler_SetCurrentMediaType((*branch).up.handler, type_);
        hr < 0
    } {
        WARN!("Failed to set upstream node media type, hr %#lx\n", hr);
        return hr;
    }

    if (if topology_node_get_type((*branch).down.node) == MF_TOPOLOGY_TRANSFORM_NODE {
        hr = IMFMediaTypeHandler_SetCurrentMediaType((*branch).down.handler, type_);
        hr < 0
    } else {
        hr = IMFMediaTypeHandler_IsMediaTypeSupported((*branch).down.handler, type_, core::ptr::null_mut());
        hr < 0
    }) {
        WARN!("Failed to set transform node media type, hr %#lx\n", hr);
        return hr;
    }

    TRACE!(
        "Connecting branch %s with type %s.\n",
        debugstr_topology_branch(branch),
        debugstr_media_type(type_)
    );
    return IMFTopologyNode_ConnectOutput(
        (*branch).up.node,
        (*branch).up.stream,
        (*branch).down.node,
        (*branch).down.stream,
    );
}

unsafe extern "C" fn topology_branch_connect_indirect(
    topology: *mut IMFTopology,
    decoder: BOOL,
    branch: *mut topology_branch,
    upstream: *mut IMFMediaType,
) -> HRESULT {
    let mut method_mask: connect_method = (CONNECT_DIRECT as u32);
    let mut input_info: MFT_REGISTER_TYPE_INFO = core::mem::zeroed();
    let mut transform: *mut IMFTransform = core::ptr::null_mut();
    let mut activates: *mut *mut IMFActivate = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut count: u32 = 0;
    let mut category: GUID = core::mem::zeroed();
    let mut guid: GUID = core::mem::zeroed();
    let mut enable_xvp: UINT32 = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "topology %p, decoder %u, branch %s, upstream %s.\n",
        topology,
        decoder,
        debugstr_topology_branch(branch),
        debugstr_media_type(upstream)
    );

    hr = IMFMediaType_GetMajorType(upstream, core::ptr::addr_of_mut!(input_info.guidMajorType));
    if hr < 0 {
        return hr;
    }
    hr = IMFMediaType_GetGUID(
        upstream,
        core::ptr::addr_of!(MF_MT_SUBTYPE),
        core::ptr::addr_of_mut!(input_info.guidSubtype),
    );
    if hr < 0 {
        return hr;
    }

    if IMFTopology_GetUINT32(
        topology,
        core::ptr::addr_of!(MF_TOPOLOGY_ENABLE_XVP_FOR_PLAYBACK),
        core::ptr::addr_of_mut!(enable_xvp),
    ) < 0
    {
        enable_xvp = 0;
    }

    if IsEqualGUID!(
        core::ptr::addr_of!(input_info.guidMajorType),
        core::ptr::addr_of!(MFMediaType_Audio),
    ) != false
    {
        category = if decoder != 0 {
            MFT_CATEGORY_AUDIO_DECODER
        } else {
            MFT_CATEGORY_AUDIO_EFFECT
        };
    } else if IsEqualGUID!(
        core::ptr::addr_of!(input_info.guidMajorType),
        core::ptr::addr_of!(MFMediaType_Video),
    ) != false
    {
        category = if decoder != 0 {
            MFT_CATEGORY_VIDEO_DECODER
        } else if enable_xvp != 0 {
            MFT_CATEGORY_VIDEO_PROCESSOR
        } else {
            MFT_CATEGORY_VIDEO_EFFECT
        };
    } else {
        return MF_E_INVALIDMEDIATYPE;
    }

    hr = MFCreateTopologyNode(
        MF_TOPOLOGY_TRANSFORM_NODE,
        core::ptr::addr_of_mut!(node),
    );
    if hr < 0 {
        return hr;
    }
    if decoder != 0 {
        IMFTopologyNode_SetUINT32(
            node,
            core::ptr::addr_of!(MF_TOPONODE_DECODER),
            1,
        );
        IMFTopologyNode_SetUINT32(
            node,
            core::ptr::addr_of!(MF_TOPONODE_MARKIN_HERE),
            1,
        );
        IMFTopologyNode_SetUINT32(
            node,
            core::ptr::addr_of!(MF_TOPONODE_MARKOUT_HERE),
            1,
        );
        method_mask = ::core::mem::transmute::<::core::ffi::c_uint, connect_method>(
            method_mask as ::core::ffi::c_uint
                | (CONNECT_CONVERTER as ::core::ffi::c_uint),
        );
    }

    hr = MFTEnumEx(
        category,
        MFT_ENUM_FLAG_ALL as UINT32,
        core::ptr::addr_of_mut!(input_info),
        core::ptr::null(),
        core::ptr::addr_of_mut!(activates),
        core::ptr::addr_of_mut!(count),
    );
    if hr < 0 {
        return hr;
    }

    i = 0;
    while i < count {
        let mut up_branch: *mut topology_branch = core::ptr::null_mut();
        let mut down_branch: *mut topology_branch = core::ptr::null_mut();
        let mut media_type: *mut IMFMediaType = core::ptr::null_mut();

        if !(IMFActivate_ActivateObject(
            *activates.offset(i as isize),
            core::ptr::addr_of!(IID_IMFTransform),
            (core::ptr::addr_of_mut!(transform) as *mut ::core::ffi::c_void).cast(),
        ) < 0)
        {
            IMFTopologyNode_SetObject(node, transform as *mut IUnknown);
            IMFTopologyNode_DeleteItem(node, core::ptr::addr_of!(MF_TOPONODE_TRANSFORM_OBJECTID));
            if IMFActivate_GetGUID(
                *activates.offset(i as isize),
                core::ptr::addr_of!(MFT_TRANSFORM_CLSID_Attribute),
                core::ptr::addr_of_mut!(guid),
            ) >= 0
                && IsEqualGUID!(
                    core::ptr::addr_of!(guid),
                    core::ptr::addr_of!(CLSID_VideoProcessorMFT),
                ) == false
            {
                IMFTopologyNode_SetGUID(
                    node,
                    core::ptr::addr_of!(MF_TOPONODE_TRANSFORM_OBJECTID),
                    core::ptr::addr_of_mut!(guid),
                );
            }
            IMFTransform_Release(transform);

            hr = topology_branch_create_indirect(
                branch,
                node,
                core::ptr::addr_of_mut!(up_branch),
                core::ptr::addr_of_mut!(down_branch),
            );
            if hr >= 0 {
                hr = topology_branch_connect_with_type(topology, up_branch, upstream);
                if hr >= 0 {
                    hr = topology_branch_find_best_type(
                        down_branch,
                        upstream,
                        core::ptr::addr_of_mut!(media_type),
                    );
                    if hr < 0 {
                        hr = topology_branch_connect(
                            topology,
                            method_mask,
                            down_branch,
                            upstream,
                            (FALSE) as BOOL,
                        );
                    } else {
                        hr = topology_branch_connect_with_type(topology, down_branch, media_type);
                        IMFMediaType_Release(media_type);
                    }
                }
                topology_branch_destroy(down_branch);
                topology_branch_destroy(up_branch);
            }

            if hr >= 0 {
                hr = IMFTopology_AddNode(topology, node);
            }
            if hr >= 0 {
                break;
            }
            IMFTopologyNode_DisconnectOutput((*branch).up.node, (*branch).up.stream);
        }
        i = i.wrapping_add(1);
    }

    IMFTopologyNode_Release(node);
    i = 0;
    while i < count {
        IMFActivate_Release(*activates.offset(i as isize));
        i = i.wrapping_add(1);
    }
    CoTaskMemFree(activates as LPVOID);

    if count == 0 || hr < 0 {
        hr = if decoder != 0 {
            MF_E_TOPO_CODEC_NOT_FOUND
        } else {
            MF_E_TRANSFORM_NOT_POSSIBLE_FOR_CURRENT_MEDIATYPE_COMBINATION
        };
    }

    TRACE!("returning %#lx\n", hr);
    return hr;
}

unsafe extern "C" fn get_first_supported_media_type(
    mut handler: *mut IMFMediaTypeHandler,
    mut type_0: *mut *mut IMFMediaType,
) -> HRESULT {
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
    let mut hr: HRESULT = 0;
    let mut i: DWORD = 0;

    hr = IMFMediaTypeHandler_GetCurrentMediaType(handler, type_0);
    if hr != MF_E_NOT_INITIALIZED {
        return hr;
    }

    i = 0;
    loop {
        hr = IMFMediaTypeHandler_GetMediaTypeByIndex(handler, i, core::ptr::addr_of_mut!(media_type));
        if !(hr >= 0) {
            break;
        }

        hr = IMFMediaTypeHandler_IsMediaTypeSupported(handler, media_type, core::ptr::null_mut::<*mut IMFMediaType>());
        if hr >= 0 {
            *type_0 = media_type;
            return hr;
        }

        IMFMediaType_Release(media_type);
        i = i.wrapping_add(1);
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_init_media_type(node: *mut IMFTopologyNode, stream: DWORD, output: BOOL, type_0: *mut *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut hr: HRESULT;
    if {
        hr = topology_node_get_type_handler(node, stream, output, core::ptr::addr_of_mut!(handler));
        hr
    } >= 0
    {
        if {
            hr = get_first_supported_media_type(handler, type_0);
            hr
        } >= 0
        {
            hr = IMFMediaTypeHandler_SetCurrentMediaType(handler, *type_0);
        }
        IMFMediaTypeHandler_Release(handler);
    }
    return hr;
}

unsafe extern "C" fn topology_branch_connect_direct(topology: *mut IMFTopology, branch: *mut topology_branch, up_type: *mut IMFMediaType) -> HRESULT {
    let mut down_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut type_: *mut IMFMediaType = core::ptr::null_mut();
    let mut flags: DWORD = 0;
    let mut hr: HRESULT = 0;

    TRACE!("topology %p, branch %s, up_type %s.\n", topology, debugstr_topology_branch(branch), debugstr_media_type(up_type));

    if (({ hr = IMFMediaTypeHandler_GetCurrentMediaType((*branch).down.handler, core::ptr::addr_of_mut!(down_type)); hr })) >= 0
    {
        if topology_node_get_type((*branch).down.node) != MF_TOPOLOGY_OUTPUT_NODE
            || IMFMediaType_IsEqual(up_type, down_type, core::ptr::addr_of_mut!(flags)) == S_OK
        {
            hr = topology_branch_connect_with_type(topology, branch, up_type);
        } else {
            TRACE!("current type %s differs from up type %s, hr %#lx\n", debugstr_media_type(down_type), debugstr_media_type(up_type), hr);
            hr = MF_E_INVALIDMEDIATYPE;
        }
        IMFMediaType_Release(down_type);

        TRACE!("returning %#lx\n", hr);
        return hr;
    }

    type_ = up_type;
    IMFMediaType_AddRef(type_);

    if hr == MF_E_NOT_INITIALIZED {
        let mut i: UINT = 0;
        loop {
            hr = IMFMediaTypeHandler_GetMediaTypeByIndex((*branch).down.handler, i, core::ptr::addr_of_mut!(down_type));
            if !(hr >= 0) {
                break;
            }
            'body: {
                hr = IMFMediaType_IsEqual(up_type, down_type, core::ptr::addr_of_mut!(flags));
                if hr != S_OK {
                    TRACE!("down type %s differs from up type %s, hr %#lx flags %#lx\n", debugstr_media_type(down_type), debugstr_media_type(up_type), hr, flags);
                } else if ({ hr = IMFMediaTypeHandler_IsMediaTypeSupported((*branch).down.handler, down_type, core::ptr::null_mut()); hr }) < 0 {
                    TRACE!("down type %s not supported by downstream, hr %#lx\n", debugstr_media_type(down_type), hr);
                } else if ({ hr = IMFMediaTypeHandler_IsMediaTypeSupported((*branch).up.handler, down_type, core::ptr::null_mut()); hr }) < 0 {
                    TRACE!("down type %s not supported by upstream, hr %#lx\n", debugstr_media_type(down_type), hr);
                } else {
                    IMFMediaType_Release(type_);
                    type_ = down_type;
                    break 'body;
                }
                IMFMediaType_Release(down_type);
            }
            i = i.wrapping_add(1);
        }
    }

    if ({ hr = IMFMediaTypeHandler_IsMediaTypeSupported((*branch).down.handler, type_, core::ptr::null_mut()); hr }) < 0 {
        TRACE!("type %s not supported by downstream, hr %#lx\n", debugstr_media_type(type_), hr);
    } else {
        hr = topology_branch_connect_with_type(topology, branch, type_);
    }

    IMFMediaType_Release(type_);

    TRACE!("returning %#lx\n", hr);
    return hr;
}

unsafe extern "C" fn topology_branch_connect_down(
    mut topology: *mut IMFTopology,
    mut method: connect_method,
    mut branch: *mut topology_branch,
    mut up_type: *mut IMFMediaType,
) -> HRESULT {
    let mut hr: HRESULT = MF_E_INVALIDMEDIATYPE as HRESULT;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/topology_loader.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_branch_connect_down\0" as *const u8 as *const ::core::ffi::c_char,
            694 as ::core::ffi::c_int,
            b"topology %p, branch %s, up_type %s.\n\0" as *const u8 as *const ::core::ffi::c_char,
            topology,
            debugstr_topology_branch(branch),
            debugstr_media_type(up_type),
        );
    }
    if (hr < 0 as HRESULT)
        && ((method as u32) & (CONNECT_DIRECT as u32)) != 0
    {
        hr = topology_branch_connect_direct(topology, branch, up_type);
    }
    if (hr < 0 as HRESULT)
        && ((method as u32) & (CONNECT_CONVERTER as u32)) != 0
    {
        hr = topology_branch_connect_indirect(topology, (FALSE) as BOOL, branch, up_type);
    }
    if (hr < 0 as HRESULT)
        && ((method as u32) & (CONNECT_DECODER as u32)) != 0
    {
        hr = topology_branch_connect_indirect(topology, (TRUE) as BOOL, branch, up_type);
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl_0) as u32,
            __dbch_0,
            b"dll/win32/mf/topology_loader.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"topology_branch_connect_down\0" as *const u8 as *const ::core::ffi::c_char,
            703 as ::core::ffi::c_int,
            b"returning %#lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            hr,
        );
    }
    return hr;
}

unsafe extern "C" fn topology_branch_foreach_up_types(
    mut topology: *mut IMFTopology,
    mut method_mask: connect_method,
    mut branch: *mut topology_branch,
    mut upstream: *mut IMFMediaType,
    mut force_enumerate: BOOL,
) -> HRESULT {
    let mut hr: HRESULT = MF_E_INVALIDMEDIATYPE;
    let mut enumerate: UINT32 = TRUE;
    let mut up_type: *mut IMFMediaType = core::ptr::null_mut();

    TRACE!(
        "topology %p, method_mask %#x, branch %s, upstream %s.\n",
        topology,
        method_mask as u32,
        debugstr_topology_branch(branch),
        debugstr_media_type(upstream)
    );

    let __switch_node_type = topology_node_get_type((*branch).up.node);
    if __switch_node_type == MF_TOPOLOGY_SOURCESTREAM_NODE {
        if IMFTopology_GetUINT32(
            topology,
            &raw const MF_TOPOLOGY_ENUMERATE_SOURCE_TYPES,
            &raw mut enumerate,
        ) < 0
        {
            enumerate = FALSE;
        }
    } else {
        if IMFMediaTypeHandler_GetCurrentMediaType((*branch).up.handler, &raw mut up_type)
            >= 0
        {
            IMFMediaType_Release(up_type);
            enumerate = FALSE;
        }
    }

    if enumerate != 0 || force_enumerate != 0 {
        let mut i: UINT = 0 as UINT;
        loop {
            hr = media_type_handler_get_type((*branch).up.handler, i as UINT32, &raw mut up_type);
            if !(hr >= 0) {
                break;
            }
            if !upstream.is_null() && {
                hr = update_media_type_from_upstream(up_type, upstream);
                hr < 0
            } {
                WARN!(
                    "Failed to update type from upstream, hr %#lx\n",
                    hr
                );
            }
            hr = topology_branch_connect_down(topology, method_mask, branch, up_type);
            IMFMediaType_Release(up_type);
            if hr >= 0 {
                return hr;
            }
            i = i.wrapping_add(1);
        }
        return hr;
    }

    if {
        hr = media_type_handler_get_current((*branch).up.handler, &raw mut up_type);
        hr >= 0
    } || {
        hr = media_type_handler_get_type(
            (*branch).up.handler,
            0 as UINT32,
            &raw mut up_type,
        );
        hr >= 0
    } {
        if !upstream.is_null() && {
            hr = update_media_type_from_upstream(up_type, upstream);
            hr < 0
        } {
            WARN!(
                "Failed to update type from upstream, hr %#lx\n",
                hr
            );
        }
        hr = topology_branch_connect_down(topology, method_mask, branch, up_type);
        IMFMediaType_Release(up_type);
        return hr;
    }

    TRACE!("returning %#lx\n", hr);
    return hr;
}

unsafe extern "C" fn topology_branch_connect(
    mut topology: *mut IMFTopology,
    mut method_mask: connect_method,
    mut branch: *mut topology_branch,
    mut upstream: *mut IMFMediaType,
    mut force_enumerate: BOOL,
) -> HRESULT {
    let mut hr: HRESULT = MF_E_INVALIDMEDIATYPE;
    let mut up_method: UINT32 = 0;
    let mut down_method: UINT32 = 0;

    if (IMFTopologyNode_GetUINT32(
        (*branch).down.node,
        core::ptr::addr_of!(MF_TOPONODE_CONNECT_METHOD),
        core::ptr::addr_of_mut!(down_method),
    ) < 0)
    {
        down_method = MF_CONNECT_ALLOW_DECODER;
    }
    down_method = (connect_method_from_mf(down_method as MF_CONNECT_METHOD) as UINT32
        & method_mask as UINT32) as UINT32;

    if (topology_node_get_type((*branch).up.node) as u32 != MF_TOPOLOGY_SOURCESTREAM_NODE as u32)
        || (IMFTopologyNode_GetUINT32(
            (*branch).up.node,
            core::ptr::addr_of!(MF_TOPONODE_CONNECT_METHOD),
            core::ptr::addr_of_mut!(up_method),
        ) < 0)
    {
        up_method = MF_CONNECT_DIRECT;
    }

    if (up_method & MF_CONNECT_RESOLVE_INDEPENDENT_OUTPUTTYPES) != 0 {
        hr = topology_branch_foreach_up_types(topology, down_method as connect_method, branch, upstream, force_enumerate);
    } else {
        if (hr < 0) && ((down_method & (CONNECT_DIRECT as u32)) != 0) {
            hr = topology_branch_foreach_up_types(topology, (CONNECT_DIRECT) as connect_method, branch, upstream, force_enumerate);
        }
        if (hr < 0) && ((down_method & (CONNECT_CONVERTER as u32)) != 0) {
            hr = topology_branch_foreach_up_types(topology, (CONNECT_CONVERTER) as connect_method, branch, upstream, force_enumerate);
        }
        if (hr < 0) && ((down_method & (CONNECT_DECODER as u32)) != 0) {
            hr = topology_branch_foreach_up_types(topology, (CONNECT_DECODER) as connect_method, branch, upstream, force_enumerate);
        }
    }

    return hr;
}

unsafe extern "C" fn topology_branch_connect_optional_chain(
    topology: *mut IMFTopology,
    branch: *mut topology_branch,
    down_node: *mut IMFTopologyNode,
    down_stream: DWORD,
) -> HRESULT {
    let mut current: *mut topology_branch = core::ptr::null_mut();
    let mut up_branch: *mut topology_branch = core::ptr::null_mut();
    let mut down_branch: *mut topology_branch = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut stream: DWORD = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "topology %p, branch %s, down_node %p, down_stream %lu\n",
        topology,
        debugstr_topology_branch(branch),
        down_node,
        down_stream
    );

    if (({ hr = IMFTopologyNode_GetOutput((*branch).up.node, (*branch).up.stream, &raw mut node, &raw mut stream); hr }) < 0) {
        return hr;
    }
    while ((hr) >= 0) && node != (*branch).down.node {
        let mut tmp: *mut IMFTopologyNode = node;
        let mut orig: *mut IMFMediaType = core::ptr::null_mut();
        let mut method: UINT32 = 0;

        if (IMFTopologyNode_GetUINT32(node, &raw const MF_TOPONODE_CONNECT_METHOD, &raw mut method)) < 0 {
            method = MF_CONNECT_DIRECT as UINT32;
        }
        method &= MF_CONNECT_ALLOW_CONVERTER as UINT32;

        /* The loader may have inserted one or more transforms upstream. Only try inserting the optional node
         * downstream of them. Support for upstream insertion is untested in native, but seems unlikely to work. */
        hr = topology_branch_create_for_input(down_node, down_stream, &raw mut current);
        if hr < 0 {
            break;
        }
        hr = IMFMediaTypeHandler_GetCurrentMediaType((*current).up.handler, &raw mut orig);
        if hr < 0 {
            topology_branch_destroy(current);
            break;
        }

        hr = topology_branch_create_optional(topology, current, node, stream, &raw mut up_branch, &raw mut down_branch);
        if hr >= 0 {
            hr = topology_branch_connect(
                topology,
                (CONNECT_DIRECT) as connect_method,
                up_branch,
                core::ptr::null_mut(),
                (method != 0) as core::ffi::c_int,
            );
            if hr < 0
                || {
                    hr = topology_branch_connect(
                        topology,
                        (CONNECT_DIRECT) as connect_method,
                        down_branch,
                        core::ptr::null_mut(),
                        (FALSE) as BOOL,
                    );
                    hr < 0
                }
            {
                hr = topology_branch_connect_with_type(topology, current, orig);
                if hr < 0 {
                    WARN!("Failed to restore previous branch %s\n", debugstr_topology_branch(current));
                }
                IMFTopology_RemoveNode(topology, (*down_branch).up.node);
            }
            topology_branch_destroy(down_branch);
            topology_branch_destroy(up_branch);
        }
        IMFMediaType_Release(orig);
        topology_branch_destroy(current);

        hr = IMFTopologyNode_GetOutput(tmp, 0, &raw mut node, &raw mut stream);
        IMFTopologyNode_Release(tmp);
        if hr < 0 {
            return hr;
        }
    }
    IMFTopologyNode_Release(node);

    TRACE!("returning %#lx\n", hr);
    return hr;
}

unsafe extern "C" fn topology_loader_resolve_node(topology: *mut IMFTopology, node: *mut IMFTopologyNode) -> HRESULT {
    let mut method_mask: connect_method = connect_method_from_mf(MF_CONNECT_ALLOW_DECODER);
    let mut branch: *mut topology_branch = core::ptr::null_mut::<topology_branch>();
    let mut cloned: *mut topology_branch = core::ptr::null_mut::<topology_branch>();
    let mut output_count: DWORD = 0;
    let mut method: UINT32 = 0;
    let mut hr: HRESULT = 0;

    TRACE!("topology %p, node %p\n", topology, node);

    hr = IMFTopologyNode_GetOutputCount(node, core::ptr::addr_of_mut!(output_count));
    if hr >= 0 && output_count == 0 {
        return if topology_node_get_type(node) == MF_TOPOLOGY_OUTPUT_NODE {
            S_OK
        } else {
            MF_E_TOPO_UNSUPPORTED
        };
    }

    if IMFTopologyNode_GetUINT32(node, core::ptr::addr_of!(MF_TOPONODE_CONNECT_METHOD), core::ptr::addr_of_mut!(method)) >= 0
        && (method & MF_CONNECT_AS_OPTIONAL_BRANCH) != 0
    {
        FIXME!("MF_CONNECT_AS_OPTIONAL_BRANCH not implemented\n");
    }

    let mut i: UINT = 0;
    while hr >= 0
        && topology_branch_create_for_output(node, i as DWORD, core::ptr::addr_of_mut!(branch)) >= 0
    {
        hr = topology_branch_create_cloned(
            topology,
            (*branch).up.node,
            (*branch).up.stream,
            (*branch).down.node,
            (*branch).down.stream,
            core::ptr::addr_of_mut!(cloned),
        );
        if hr >= 0 {
            hr = topology_branch_connect(
                topology,
                method_mask,
                cloned,
                core::ptr::null_mut::<IMFMediaType>(),
                (FALSE) as BOOL,
            );
            if hr >= 0 {
                topology_branch_connect_optional_chain(topology, branch, (*cloned).down.node, (*cloned).down.stream);
            }
            topology_branch_destroy(cloned);
        }
        if hr >= 0 {
            hr = topology_loader_resolve_node(topology, (*branch).down.node);
        }
        topology_branch_destroy(branch);
        i = i.wrapping_add(1);
    }

    TRACE!("returning %#lx\n", hr);

    return hr;
}

unsafe extern "C" fn topology_node_get_object_attributes(node: *mut IMFTopologyNode, attributes: *mut *mut IMFAttributes) -> BOOL {
    let mut transform: *mut IMFTransform = core::ptr::null_mut();
    let mut hr: HRESULT;
    if (topology_node_get_object(node, core::ptr::addr_of!(IID_IMFTransform), core::ptr::addr_of_mut!(transform) as *mut *mut core::ffi::c_void)) >= 0
    {
        hr = IMFTransform_GetAttributes(transform, attributes);
        IMFTransform_Release(transform);
        return hr as BOOL;
    }

    return topology_node_get_object(node, core::ptr::addr_of!(IID_IMFAttributes), attributes as *mut *mut core::ffi::c_void) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_is_d3d_aware(node: *mut IMFTopologyNode) -> BOOL {
    let mut d3d_aware: UINT32 = 0;
    let mut d3d11_aware: UINT32 = 0;
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut::<IMFAttributes>();

    if (topology_node_get_object_attributes(node, &raw mut attributes) as HRESULT) < 0 as HRESULT {
        return FALSE as BOOL;
    }
    if (IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D_AWARE, &raw mut d3d_aware) as HRESULT) < 0 as HRESULT {
        d3d_aware = FALSE as UINT32;
    }
    if (IMFAttributes_GetUINT32(attributes, &raw const MF_SA_D3D11_AWARE, &raw mut d3d11_aware) as HRESULT) < 0 as HRESULT {
        d3d11_aware = FALSE as UINT32;
    }
    IMFAttributes_Release(attributes);

    return (d3d_aware != 0 || d3d11_aware != 0) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_transform_async_get_attributes(node: *mut IMFTopologyNode) -> *mut IMFAttributes {
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    if topology_node_get_object_attributes(node, core::ptr::addr_of_mut!(attributes)) as HRESULT >= 0 {
        let mut async_: UINT32 = 0;
        if IMFAttributes_GetUINT32(attributes, core::ptr::addr_of!(MF_TRANSFORM_ASYNC), core::ptr::addr_of_mut!(async_)) >= 0 && async_ != 0 {
            return attributes;
        }
        IMFAttributes_Release(attributes);
    }

    core::ptr::null_mut()
}

unsafe extern "C" fn topology_loader_create_copier(upstream_node: *mut IMFTopologyNode, upstream_output: DWORD, downstream_node: *mut IMFTopologyNode, downstream_input: core::ffi::c_uint, copier: *mut *mut IMFTransform) -> HRESULT {
    let mut up_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut transform: *mut IMFTransform = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = MFCreateSampleCopierMFT(core::ptr::addr_of_mut!(transform));
    if hr < 0 {
        return hr;
    }

    hr = MFGetTopoNodeCurrentType(upstream_node, upstream_output, (TRUE) as BOOL, core::ptr::addr_of_mut!(up_type));
    if hr < 0 {
        WARN!("Failed to get upstream media type hr %#lx.\n", hr);
    }

    if hr >= 0 {
        hr = IMFTransform_SetInputType(transform, 0, up_type, 0);
        if hr < 0 {
            WARN!("Input type wasn't accepted, hr %#lx.\n", hr);
        }
    }

    // We assume, that up_type is set to a value compatible with the down node by the branch resolver.
    if hr >= 0 {
        hr = IMFTransform_SetOutputType(transform, 0, up_type, 0);
        if hr < 0 {
            WARN!("Output type wasn't accepted, hr %#lx.\n", hr);
        }
    }

    if hr >= 0 {
        *copier = transform;
        IMFTransform_AddRef(*copier);
    }

    if !up_type.is_null() {
        IMFMediaType_Release(up_type);
    }

    IMFTransform_Release(transform);

    return hr;
}

unsafe extern "C" fn topology_loader_connect_copier(topology: *mut IMFTopology, upstream_node: *mut IMFTopologyNode, upstream_output: DWORD, downstream_node: *mut IMFTopologyNode, downstream_input: DWORD, copier: *mut IMFTransform) -> HRESULT {
    let mut copier_node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut hr: HRESULT;
    hr = MFCreateTopologyNode(MF_TOPOLOGY_TRANSFORM_NODE, core::ptr::addr_of_mut!(copier_node));
    if hr < 0 {
        return hr;
    }

    IMFTopologyNode_SetObject(copier_node, copier as *mut IUnknown);
    IMFTopology_AddNode(topology, copier_node);
    IMFTopologyNode_ConnectOutput(upstream_node, upstream_output, copier_node, 0);
    IMFTopologyNode_ConnectOutput(copier_node, 0, downstream_node, downstream_input);

    IMFTopologyNode_Release(copier_node);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn topology_node_set_device_manager(
    mut node: *mut IMFTopologyNode,
    mut device_manager: *mut IUnknown,
) -> HRESULT {
    let mut transform: *mut IMFTransform = ::core::ptr::null_mut::<IMFTransform>();
    let mut hr: HRESULT = 0;
    if {
        hr = topology_node_get_object(
            node,
            &raw const IID_IMFTransform,
            &raw mut transform as *mut *mut ::core::ffi::c_void,
        );
        hr >= 0
    } {
        hr = IMFTransform_ProcessMessage(
            transform,
            MFT_MESSAGE_SET_D3D_MANAGER,
            device_manager as LONG_PTR as ULONG_PTR,
        );
        IMFTransform_Release(transform);
    }

    if hr >= 0 {
        let mut upstream: *mut IMFTopologyNode = ::core::ptr::null_mut::<IMFTopologyNode>();
        let mut i: DWORD = 0;
        let mut count: DWORD = 0;
        let mut output: DWORD = 0;

        hr = IMFTopologyNode_GetInputCount(node, &raw mut count);

        i = 0 as DWORD;
        while hr >= 0 && i < count {
            'body: {
                if IMFTopologyNode_GetInput(node, i, &raw mut upstream, &raw mut output)
                    < 0
                {
                    break 'body;
                }

                if topology_node_is_d3d_aware(upstream) != 0 {
                    topology_node_set_device_manager(upstream, device_manager);
                }

                IMFTopologyNode_Release(upstream);
            }
            i = i.wrapping_add(1);
        }
    }

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn stream_sink_get_device_manager(stream_sink: *mut IMFStreamSink, device_manager: *mut *mut IUnknown) -> HRESULT {
    let mut hr: HRESULT;
    if {
        hr = MFGetService(stream_sink as *mut IUnknown, &raw const MR_VIDEO_ACCELERATION_SERVICE,
            &raw const IID_IMFDXGIDeviceManager, device_manager as *mut *mut core::ffi::c_void);
        hr >= 0
    } {
        return hr;
    }
    if {
        hr = MFGetService(stream_sink as *mut IUnknown, &raw const MR_VIDEO_ACCELERATION_SERVICE,
            &raw const IID_IDirect3DDeviceManager9, device_manager as *mut *mut core::ffi::c_void);
        hr >= 0
    } {
        return hr;
    }

    return hr;
}

unsafe extern "C" fn topology_loader_connect_d3d_aware_sink(
    mut topology: *mut IMFTopology,
    mut node: *mut IMFTopologyNode,
    mut dxva_mode: MFTOPOLOGY_DXVA_MODE,
) -> HRESULT {
    let mut upstream_node: *mut IMFTopologyNode = core::ptr::null_mut::<IMFTopologyNode>();
    let mut copier: *mut IMFTransform = core::ptr::null_mut::<IMFTransform>();
    let mut stream_sink: *mut IMFStreamSink = core::ptr::null_mut::<IMFStreamSink>();
    let mut device_manager: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut upstream_output: DWORD = 0;
    let mut hr: HRESULT = 0;

    if {
        hr = topology_node_get_object(
            node,
            core::ptr::addr_of!(IID_IMFStreamSink),
            core::ptr::addr_of_mut!(stream_sink) as *mut *mut ::core::ffi::c_void,
        );
        hr < 0 as HRESULT
    } {
        return hr;
    }

    if {
        hr = stream_sink_get_device_manager(stream_sink, core::ptr::addr_of_mut!(device_manager));
        hr >= 0 as HRESULT
    } {
        if IMFTopologyNode_GetInput(
            node,
            0 as DWORD,
            core::ptr::addr_of_mut!(upstream_node),
            core::ptr::addr_of_mut!(upstream_output),
        ) >= 0 as HRESULT
        {
            let mut needs_copier: BOOL = (dxva_mode as ::core::ffi::c_uint
                == MFTOPOLOGY_DXVA_DEFAULT as ::core::ffi::c_uint)
                as ::core::ffi::c_int;
            let mut transform: *mut IMFTransform = core::ptr::null_mut::<IMFTransform>();

            if needs_copier != 0
                && topology_node_get_object(
                    upstream_node,
                    core::ptr::addr_of!(IID_IMFTransform),
                    core::ptr::addr_of_mut!(transform) as *mut *mut ::core::ffi::c_void,
                ) >= 0 as HRESULT
            {
                let mut info: MFT_OUTPUT_STREAM_INFO = _MFT_OUTPUT_STREAM_INFO {
                    dwFlags: 0 as DWORD,
                    cbSize: 0,
                    cbAlignment: 0,
                };

                if IMFTransform_GetOutputStreamInfo(
                    transform,
                    upstream_output,
                    core::ptr::addr_of_mut!(info),
                ) < 0 as HRESULT
                    || (info.dwFlags
                        & (MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES as DWORD | MFT_OUTPUT_STREAM_PROVIDES_SAMPLES as DWORD)) == 0
                {
                    needs_copier = FALSE as BOOL;
                }

                IMFTransform_Release(transform);
            }

            if needs_copier != 0 && {
                hr = topology_loader_create_copier(
                    upstream_node,
                    upstream_output,
                    node,
                    0 as ::core::ffi::c_uint,
                    core::ptr::addr_of_mut!(copier),
                );
                hr >= 0 as HRESULT
            } {
                hr = topology_loader_connect_copier(
                    topology,
                    upstream_node,
                    upstream_output,
                    node,
                    0 as DWORD,
                    copier,
                );
                IMFTransform_Release(copier);
            }

            if dxva_mode as ::core::ffi::c_uint
                == MFTOPOLOGY_DXVA_FULL as ::core::ffi::c_uint
                && topology_node_is_d3d_aware(upstream_node) != 0
            {
                topology_node_set_device_manager(upstream_node, device_manager);
            }

            IMFTopologyNode_Release(upstream_node);
        }

        IUnknown_Release(device_manager);
    }

    IMFStreamSink_Release(stream_sink);

    return hr;
}

unsafe extern "C" fn topology_loader_resolve_complete(mut topology: *mut IMFTopology) {
    let mut dxva_mode: MFTOPOLOGY_DXVA_MODE = 0;
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut i: WORD = 0;
    let mut node_count: WORD = 0;
    let mut hr: HRESULT = 0;

    IMFTopology_GetNodeCount(topology, core::ptr::addr_of_mut!(node_count));

    if IMFTopology_GetUINT32(
        topology,
        core::ptr::addr_of!(MF_TOPOLOGY_DXVA_MODE),
        core::ptr::addr_of_mut!(dxva_mode) as *mut UINT32,
    ) < 0
    {
        dxva_mode = 0;
    }

    i = 0 as WORD;
    while (i as i32) < (node_count as i32) {
        if IMFTopology_GetNode(topology, i, core::ptr::addr_of_mut!(node)) >= 0 {
            match topology_node_get_type(node) as u32 {
                MF_TOPOLOGY_OUTPUT_NODE => {
                    /* Set MF_TOPONODE_STREAMID for all outputs. */
                    if IMFTopologyNode_GetItem(
                        node,
                        core::ptr::addr_of!(MF_TOPONODE_STREAMID),
                        core::ptr::null_mut(),
                    ) < 0
                    {
                        IMFTopologyNode_SetUINT32(
                            node,
                            core::ptr::addr_of!(MF_TOPONODE_STREAMID),
                            0,
                        );
                    }

                    hr = topology_loader_connect_d3d_aware_sink(topology, node, dxva_mode);
                    if hr < 0 {
                        WARN!("Failed to connect D3D-aware input, hr %#lx.\n", hr);
                    }
                }
                MF_TOPOLOGY_SOURCESTREAM_NODE => {
                    /* Set MF_TOPONODE_MEDIASTART for all sources. */
                    if IMFTopologyNode_GetItem(
                        node,
                        core::ptr::addr_of!(MF_TOPONODE_MEDIASTART),
                        core::ptr::null_mut(),
                    ) < 0
                    {
                        IMFTopologyNode_SetUINT64(
                            node,
                            core::ptr::addr_of!(MF_TOPONODE_MEDIASTART),
                            0,
                        );
                    }
                }
                _ => {}
            }

            IMFTopologyNode_Release(node);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn clone_topology(
    input_topology: *mut IMFTopology,
    sources: *mut IMFCollection,
    output_topology: *mut *mut IMFTopology,
) -> HRESULT {
    let mut has_source: BOOL = (FALSE as i32);
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut sink: *mut IMFStreamSink = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut topoid: TOPOID = 0;
    let mut method: UINT32 = 0;

    IMFTopology_GetTopologyID(input_topology, core::ptr::addr_of_mut!(topoid));
    hr = create_topology(topoid, output_topology);
    if hr < 0 {
        return hr;
    }
    hr = IMFTopology_CopyAllItems(input_topology, *output_topology as *mut IMFAttributes);

    let mut i: UINT = 0;
    while hr >= 0
        && IMFTopology_GetNode(input_topology, i as WORD, core::ptr::addr_of_mut!(node))
            >= 0
    {
        match topology_node_get_type(node) {
            MF_TOPOLOGY_OUTPUT_NODE => {
                if IMFTopologyNode_GetUINT32(
                    node,
                    &MF_TOPONODE_CONNECT_METHOD,
                    core::ptr::addr_of_mut!(method),
                ) >= 0
                    && (method & MF_CONNECT_AS_OPTIONAL) != 0
                {
                    hr = MF_E_TOPO_UNSUPPORTED;
                } else {
                    hr = IMFTopologyNode_GetObject(node, core::ptr::addr_of_mut!(object));
                    if hr >= 0 {
                        if IUnknown_QueryInterface(
                            object,
                            core::ptr::addr_of!(IID_IMFStreamSink),
                            core::ptr::addr_of_mut!(sink) as *mut *mut core::ffi::c_void,
                        ) < 0
                        {
                            hr = MF_E_TOPO_SINK_ACTIVATES_UNSUPPORTED;
                        } else {
                            IMFStreamSink_Release(sink);
                        }
                        IUnknown_Release(object);
                    }
                }
            }
            MF_TOPOLOGY_SOURCESTREAM_NODE => {
                hr = IMFTopologyNode_GetItem(
                    node,
                    &MF_TOPONODE_STREAM_DESCRIPTOR,
                    core::ptr::null_mut(),
                );
                if hr >= 0 {
                    hr = IMFCollection_AddElement(sources, node as *mut IUnknown);
                }
                has_source = (TRUE as i32);
            }
            _ => {}
        }

        IMFTopologyNode_Release(node);
        i = i.wrapping_add(1);
    }

    if hr >= 0 && has_source == 0 {
        hr = MF_E_TOPO_UNSUPPORTED;
    }
    if hr >= 0 {
        return hr;
    }

    IMFTopology_Release(*output_topology);
    return hr;
}
unsafe extern "system" fn topology_loader_Load(
    iface: *mut IMFTopoLoader,
    input_topology: *mut IMFTopology,
    ret_topology: *mut *mut IMFTopology,
    current_topology: *mut IMFTopology,
) -> HRESULT {
    let mut output_topology: *mut IMFTopology = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut nodes: *mut IMFCollection = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if !current_topology.is_null() {
        FIXME!("Current topology instance is ignored.\n");
    }

    hr = MFCreateCollection(core::ptr::addr_of_mut!(nodes));
    if hr < 0 {
        return hr;
    }

    'done: {
        hr = clone_topology(input_topology, nodes, core::ptr::addr_of_mut!(output_topology));
        if hr < 0 {
            break 'done;
        }

        while hr >= 0
            && IMFCollection_RemoveElement(
                nodes,
                0 as DWORD,
                core::ptr::addr_of_mut!(node) as *mut *mut IUnknown,
            ) >= 0
        {
            hr = topology_loader_resolve_node(output_topology, node);
            IMFTopologyNode_Release(node);
        }

        if hr < 0 {
            IMFTopology_Release(output_topology);
        } else {
            topology_loader_resolve_complete(output_topology);
            *ret_topology = output_topology;
        }
    }

    IMFCollection_Release(nodes);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateTopoLoader(loader: *mut *mut IMFTopoLoader) -> HRESULT {
    let mut object: *mut topology_loader = core::ptr::null_mut();

    if loader.is_null() {
        return E_POINTER;
    }

    object = calloc((1 as usize) as _, (core::mem::size_of::<topology_loader>() as usize) as _) as *mut topology_loader;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFTopoLoader_iface.lpVtbl = core::ptr::addr_of!(topology_loader_vtbl);
    (*object).refcount = 1 as LONG;

    *loader = core::ptr::addr_of_mut!((*object).IMFTopoLoader_iface);

    return S_OK;
}

// COM vtable tables — generated from topology_loader.c by vtbl_gen.py
/// topology_loader.c:1217 `static const IMFTopoLoaderVtbl topology_loader_vtbl`  (slot order: rust:types_amd64.rs)
pub static topology_loader_vtbl: IMFTopoLoaderVtbl = IMFTopoLoaderVtbl {
    QueryInterface: Some(topology_loader_QueryInterface),
    AddRef: Some(topology_loader_AddRef),
    Release: Some(topology_loader_Release),
    Load: Some(topology_loader_Load),
};
