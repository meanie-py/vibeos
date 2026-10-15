use crate::mf::clock::MFCreatePresentationClock;
use crate::mf::quality::MFCreateStandardQualityManager;
use crate::mf::main::MFGetService;
use crate::mf::main::MFShutdownObject;
use crate::mf::sar::mf_is_sar_sink;
use crate::mf::quality::quality_manager;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut _MF_TOPONODE_IMFActivate: GUID = _GUID {
    Data1: 0x33706f4a as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0x309a as ::core::ffi::c_ushort,
    Data3: 0x49be as ::core::ffi::c_ushort,
    Data4: [
        0xa8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x79 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut local_mft_registration: IMFLocalMFTRegistration = unsafe {
    IMFLocalMFTRegistration {
        lpVtbl: &raw const local_mft_registration_vtbl,
    }
};

// #define COBJMACROS (no Rust equivalent; omitted)

// WINE_DEFAULT_DEBUG_CHANNEL(mfplat); (debug channel registration, omitted)

pub type session_command = i32;
pub const SESSION_CMD_CLEAR_TOPOLOGIES: session_command = 0;
pub const SESSION_CMD_CLOSE: session_command = 1;
pub const SESSION_CMD_SET_TOPOLOGY: session_command = 2;
pub const SESSION_CMD_START: session_command = 3;
pub const SESSION_CMD_PAUSE: session_command = 4;
pub const SESSION_CMD_STOP: session_command = 5;
pub const SESSION_CMD_SET_RATE: session_command = 6;
pub const SESSION_CMD_SHUTDOWN: session_command = 7;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct session_op_union_set_topology {
    pub flags: DWORD,
    pub topology: *mut IMFTopology,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct session_op_union_start {
    pub time_format: GUID,
    pub start_position: PROPVARIANT,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct session_op_union_set_rate {
    pub thin: BOOL,
    pub rate: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct session_op_union_notify_topology {
    pub topology: *mut IMFTopology,
}

// Anonymous union of session_op; C accesses members directly (op->set_topology etc.)
#[repr(C)]
pub union session_op_union {
    pub set_topology: session_op_union_set_topology,
    pub start: session_op_union_start,
    pub set_rate: session_op_union_set_rate,
    pub notify_topology: session_op_union_notify_topology,
}

#[repr(C)]
pub struct session_op {
    pub IUnknown_iface: IUnknown,
    pub refcount: LONG,
    pub command: session_command,
    // C's union is ANONYMOUS: bindgen's name, so crate-repair routes `op.start` through it.
    pub __bindgen_anon_1: session_op_union,
    pub entry: list,
}
pub type Psession_op = *mut session_op;

#[repr(C)]
pub struct queued_topology {
    pub entry: list,
    pub topology: *mut IMFTopology,
    pub status: MF_TOPOSTATUS,
}

pub type session_state = i32;
pub const SESSION_STATE_STOPPED: session_state = 0;
pub const SESSION_STATE_STARTED: session_state = 1;
pub const SESSION_STATE_PAUSED: session_state = 2;
pub const SESSION_STATE_CLOSED: session_state = 3;
pub const SESSION_STATE_SHUT_DOWN: session_state = 4;

pub type command_state = i32;
pub const COMMAND_STATE_COMPLETE: command_state = 0;
/// Command submitted but hasn't started execution.
/// invariant: the submitted command is always the head of session->commands.
pub const COMMAND_STATE_SUBMITTED: command_state = 1;
/// STOPPED | PAUSED | STARTED -> STARTED transition
/// -> COMMAND_STATE_STARTING_SOURCES
pub const COMMAND_STATE_RESTARTING_SOURCES: command_state = 2;
/// -> COMMAND_STATE_PREROLLING_SINKS | COMMAND_STATE_STARTING_SINKS
pub const COMMAND_STATE_STARTING_SOURCES: command_state = 3;
/// -> COMMAND_STATE_STARTING_SINKS
pub const COMMAND_STATE_PREROLLING_SINKS: command_state = 4;
/// -> SESSION_STATE_STARTED
pub const COMMAND_STATE_STARTING_SINKS: command_state = 5;
/// STARTED -> PAUSED transition
/// -> COMMAND_STATE_PAUSING_SOURCES
pub const COMMAND_STATE_PAUSING_SINKS: command_state = 6;
/// -> SESSION_STATE_PAUSED
pub const COMMAND_STATE_PAUSING_SOURCES: command_state = 7;
/// STARTED | PAUSED -> STOPPED transition
/// -> COMMAND_STATE_STOPPING_SOURCES
pub const COMMAND_STATE_STOPPING_SINKS: command_state = 8;
/// -> SESSION_STATE_STOPPED
pub const COMMAND_STATE_STOPPING_SOURCES: command_state = 9;
/// STARTED | PAUSED | STOPPED -> CLOSED transition
/// -> COMMAND_STATE_CLOSING_SOURCES
pub const COMMAND_STATE_CLOSING_SINKS: command_state = 10;
/// -> COMMAND_STATE_FINALIZING_SINKS
pub const COMMAND_STATE_CLOSING_SOURCES: command_state = 11;
/// -> SESSION_STATE_CLOSED
pub const COMMAND_STATE_FINALIZING_SINKS: command_state = 12;

pub type object_state = i32;
pub const OBJ_STATE_STOPPED: object_state = 0;
pub const OBJ_STATE_STARTED: object_state = 1;
pub const OBJ_STATE_PAUSED: object_state = 2;
pub const OBJ_STATE_PREROLLED: object_state = 3;
pub const OBJ_STATE_INVALID: object_state = 4;

pub const SOURCE_FLAG_END_OF_PRESENTATION: u32 = 0x1;

#[repr(C)]
pub union media_source_union {
    pub source: *mut IMFMediaSource,
    pub object: *mut IUnknown,
}

#[repr(C)]
pub struct media_source {
    pub entry: list,
    // anonymous in C
    pub __bindgen_anon_1: media_source_union,
    pub pd: *mut IMFPresentationDescriptor,
    pub state: object_state,
    pub flags: u32,
}

#[repr(C)]
pub union media_sink_union {
    pub sink: *mut IMFMediaSink,
    pub object: *mut IUnknown,
}

#[repr(C)]
pub struct media_sink {
    pub entry: list,
    // anonymous in C
    pub __bindgen_anon_1: media_sink_union,
    pub preroll: *mut IMFMediaSinkPreroll,
    pub event_generator: *mut IMFMediaEventGenerator,
    pub finalized: BOOL,
}

#[repr(C)]
pub struct event_entry {
    pub entry: list,
    pub event: *mut IMFMediaEvent,
}

#[repr(C)]
pub struct transform_stream {
    pub samples: list,
    pub requests: u32,
    pub raw_audio: BOOL,
    pub block_alignment: u32,
    pub bytes_per_second: u32,
    pub min_buffer_size: u32,
    pub samples_independent: BOOL,
    pub draining: BOOL,
}

pub const TOPO_NODE_END_OF_STREAM: u32 = 0x1;
pub const TOPO_NODE_SCRUB_SAMPLE_COMPLETE: u32 = 0x2;
pub const TOPO_NODE_MARKIN_HERE: u32 = 0x4;

#[repr(C)]
pub union topo_node_object_union {
    pub source_stream: *mut IMFMediaStream,
    pub sink_stream: *mut IMFStreamSink,
    pub transform: *mut IMFTransform,
    pub object: *mut IUnknown,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct topo_node_u_source {
    pub source: *mut IMFMediaSource,
    pub stream_id: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct topo_node_u_sink {
    pub requests: u32,
    pub notify_cb: IMFVideoSampleAllocatorNotify,
    pub allocator: *mut IMFVideoSampleAllocator,
    pub allocator_cb: *mut IMFVideoSampleAllocatorCallback,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct topo_node_u_transform {
    pub inputs: *mut transform_stream,
    pub input_map: *mut DWORD,
    pub input_count: u32,
    pub next_input: u32,

    pub outputs: *mut transform_stream,
    pub output_map: *mut DWORD,
    pub output_count: u32,

    pub async_: BOOL,
    pub event_source: *mut IMFMediaEventGenerator,
}

#[repr(C)]
pub union topo_node_u_union {
    pub source: topo_node_u_source,
    pub sink: topo_node_u_sink,
    pub transform: topo_node_u_transform,
}

#[repr(C)]
pub struct topo_node {
    pub entry: list,
    pub session: *mut media_session,
    pub type_: MF_TOPOLOGY_TYPE,
    pub node_id: TOPOID,
    pub node: *mut IMFTopologyNode,
    pub state: object_state,
    pub flags: u32,
    pub object: topo_node_object_union,
    pub u: topo_node_u_union,
}

pub const SESSION_FLAG_SOURCES_SUBSCRIBED: u32 = 0x1;
pub const SESSION_FLAG_PRESENTATION_CLOCK_SET: u32 = 0x2;
pub const SESSION_FLAG_NEEDS_PREROLL: u32 = 0x8;
pub const SESSION_FLAG_SOURCE_SHUTDOWN: u32 = 0x10;
pub const SESSION_FLAG_PENDING_RATE_CHANGE: u32 = 0x20;
pub const SESSION_FLAG_RESTARTING: u32 = 0x40;
pub const SESSION_FLAG_PRESENTATION_ENDING: u32 = 0x80;
pub const SESSION_FLAG_SINKS_SUBSCRIBED: u32 = 0x100;

#[repr(C)]
pub struct media_session_presentation {
    pub current_topology: *mut IMFTopology,
    pub topo_status: MF_TOPOSTATUS,
    pub clock_stop_time: MFTIME,
    pub flags: u32,
    pub sources: list,
    pub sinks: list,
    pub nodes: list,

    /// Latest Start() arguments.
    pub time_format: GUID,
    pub start_position: PROPVARIANT,
    /// Latest SetRate() arguments.
    pub thin: BOOL,
    pub rate: f32,

    pub thin_committed: BOOL,
}

#[repr(C)]
pub struct media_session {
    pub IMFMediaSession_iface: IMFMediaSession,
    pub IMFGetService_iface: IMFGetService,
    pub IMFRateSupport_iface: IMFRateSupport,
    pub IMFRateControl_iface: IMFRateControl,
    pub IMFTopologyNodeAttributeEditor_iface: IMFTopologyNodeAttributeEditor,
    pub commands_callback: IMFAsyncCallback,
    pub sa_ready_callback: IMFAsyncCallback,
    pub events_callback: IMFAsyncCallback,
    pub sink_finalizer_callback: IMFAsyncCallback,
    pub refcount: LONG,
    pub event_queue: *mut IMFMediaEventQueue,
    pub clock: *mut IMFPresentationClock,
    pub system_time_source: *mut IMFPresentationTimeSource,
    pub clock_rate_control: *mut IMFRateControl,
    pub topo_loader: *mut IMFTopoLoader,
    pub quality_manager: *mut IMFQualityManager,
    pub presentation: media_session_presentation,
    pub topologies: list,
    pub removed_topologies: list,
    pub commands: list,
    pub state: session_state,
    pub command_state: command_state,
    pub caps: DWORD,
    pub cs: CRITICAL_SECTION,
}

pub type p_renderer_node_test_func = Option<unsafe extern "C" fn(sink: *mut IMFMediaSink) -> BOOL>;

// static const IMFLocalMFTRegistrationVtbl local_mft_registration_vtbl  -> generated (vtbl_gen.py)

// static const IUnknownVtbl session_op_vtbl -> generated (vtbl_gen.py)
// static const IMFVideoSampleAllocatorNotifyVtbl node_sample_allocator_cb_vtbl -> generated
// static const IMFMediaSessionVtbl mfmediasessionvtbl -> generated
// static const IMFGetServiceVtbl session_get_service_vtbl -> generated
// static const IMFAsyncCallbackVtbl session_commands_callback_vtbl -> generated
// static const IMFAsyncCallbackVtbl session_sa_ready_callback_vtbl -> generated
// static const IMFAsyncCallbackVtbl session_events_callback_vtbl -> generated
// static const IMFAsyncCallbackVtbl session_sink_finalizer_callback_vtbl -> generated
// static const IMFRateSupportVtbl session_rate_support_vtbl -> generated
// static const IMFRateControlVtbl session_rate_control_vtbl -> generated
// static const IMFTopologyNodeAttributeEditorVtbl node_attribute_editor_vtbl -> generated

unsafe extern "C" fn impl_from_IMFMediaSession(iface: *mut IMFMediaSession) -> *mut media_session {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_session, IMFMediaSession_iface)) as *mut media_session;
}

unsafe extern "C" fn impl_from_commands_callback_IMFAsyncCallback(mut iface: *mut IMFAsyncCallback) -> *mut media_session {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_session, commands_callback)) as *mut media_session;
}

unsafe extern "C" fn impl_from_sa_ready_callback_IMFAsyncCallback(mut iface: *mut IMFAsyncCallback) -> *mut media_session {
    return (iface as *mut u8).sub(core::mem::offset_of!(media_session, sa_ready_callback)) as *mut media_session;
}

unsafe extern "C" fn impl_from_events_callback_IMFAsyncCallback(
    iface: *mut IMFAsyncCallback,
) -> *mut media_session {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(media_session, events_callback))
        as *mut media_session;
}

unsafe extern "C" fn impl_from_sink_finalizer_callback_IMFAsyncCallback(
    iface: *mut IMFAsyncCallback,
) -> *mut media_session {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(media_session, sink_finalizer_callback),
    ) as *mut media_session;
}

unsafe extern "C" fn impl_from_IMFGetService(mut iface: *mut IMFGetService) -> *mut media_session {
    return (iface as *mut u8).sub(core::mem::offset_of!(media_session, IMFGetService_iface)) as *mut media_session;
}

unsafe extern "C" fn impl_session_from_IMFRateSupport(iface: *mut IMFRateSupport) -> *mut media_session {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_session, IMFRateSupport_iface)) as *mut media_session;
}

unsafe extern "C" fn impl_session_from_IMFRateControl(mut iface: *mut IMFRateControl) -> *mut media_session {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_session, IMFRateControl_iface)) as *mut media_session;
}

unsafe extern "C" fn impl_session_from_IMFTopologyNodeAttributeEditor(iface: *mut IMFTopologyNodeAttributeEditor) -> *mut media_session {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(media_session, IMFTopologyNodeAttributeEditor_iface)) as *mut media_session;
}

unsafe extern "C" fn impl_op_from_IUnknown(iface: *mut IUnknown) -> *mut session_op {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(session_op, IUnknown_iface)) as *mut session_op;
}

unsafe extern "C" fn impl_node_from_IMFVideoSampleAllocatorNotify(iface: *mut IMFVideoSampleAllocatorNotify) -> *mut topo_node {
    return (iface as usize).wrapping_sub(core::mem::offset_of!(topo_node, u.sink.notify_cb)) as *mut topo_node;
}

unsafe extern "system" fn local_mft_registration_QueryInterface(iface: *mut IMFLocalMFTRegistration, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFLocalMFTRegistration)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFLocalMFTRegistration_AddRef(iface);
        return S_OK;
    }

    WARN!("Unexpected %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn local_mft_registration_AddRef(
    iface: *mut IMFLocalMFTRegistration,
) -> ULONG {
    2
}
unsafe extern "system" fn local_mft_registration_Release(
    iface: *mut IMFLocalMFTRegistration,
) -> ULONG {
    1
}

unsafe extern "system" fn local_mft_registration_RegisterMFTs(
    mut iface: *mut IMFLocalMFTRegistration,
    mut info: *mut MFT_REGISTRATION_INFO,
    mut count: DWORD,
) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut i: DWORD = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/session.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"local_mft_registration_RegisterMFTs\0" as *const u8 as *const ::core::ffi::c_char,
            384 as ::core::ffi::c_int,
            b"%p, %p, %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            info,
            count,
        );
    }
    i = 0;
    while i < count {
        hr = MFTRegisterLocalByCLSID(
            &raw mut (*info.offset(i as isize)).clsid,
            &raw mut (*info.offset(i as isize)).guidCategory,
            (*info.offset(i as isize)).pszName,
            (*info.offset(i as isize)).uiFlags,
            (*info.offset(i as isize)).cInTypes as UINT32,
            (*info.offset(i as isize)).pInTypes,
            (*info.offset(i as isize)).cOutTypes as UINT32,
            (*info.offset(i as isize)).pOutTypes,
        );
        if hr < 0 as HRESULT {
            break;
        }
        i = i.wrapping_add(1);
    }
    return hr;
}
unsafe extern "system" fn session_op_QueryInterface(iface: *mut IUnknown, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
        *obj = iface as *mut core::ffi::c_void;
        IUnknown_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn session_op_AddRef(iface: *mut IUnknown) -> ULONG {
    let op: *mut session_op = impl_op_from_IUnknown(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*op).refcount) as *mut ::core::ffi::c_long) as ULONG;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/session.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"session_op_AddRef\0" as *const u8 as *const ::core::ffi::c_char,
            426 as ::core::ffi::c_int,
            b"%p, refcount %lu.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            refcount,
        );
    }
    return refcount;
}

unsafe extern "system" fn session_op_Release(iface: *mut IUnknown) -> ULONG {
    let op: *mut session_op = impl_op_from_IUnknown(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*op).refcount)) as ULONG;

    if refcount == 0 {
        match ((*op).command as u32) as i32 {
            SESSION_CMD_SET_TOPOLOGY => {
                if !(*op).__bindgen_anon_1.set_topology.topology.is_null() {
                    IMFTopology_Release((*op).__bindgen_anon_1.set_topology.topology);
                }
            }
            SESSION_CMD_START => {
                PropVariantClear(core::ptr::addr_of_mut!((*op).__bindgen_anon_1.start.start_position));
            }
            _ => {}
        }
        free(op as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "C" fn create_session_op(command: session_command, ret: *mut *mut session_op) -> HRESULT {
    let mut op: *mut session_op = core::ptr::null_mut();

    op = calloc((1 as usize) as _, (core::mem::size_of::<session_op>() as usize) as _) as *mut session_op;
    if op.is_null() {
        return E_OUTOFMEMORY;
    }

    (*op).IUnknown_iface.lpVtbl = core::ptr::addr_of!(session_op_vtbl);
    (*op).refcount = 1 as LONG;
    (*op).command = command;

    *ret = op;

    return S_OK;
}

unsafe extern "C" fn session_is_shut_down(session: *mut media_session) -> HRESULT {
    return if (*session).state == SESSION_STATE_SHUT_DOWN { MF_E_SHUTDOWN } else { S_OK };
}

unsafe extern "C" fn session_submit_command(session: *mut media_session, op: *mut session_op) -> HRESULT {
    let mut hr: HRESULT = 0;
    TRACE!("session %p, op %p, command %u.\n", session, op, (*op).command as ::core::ffi::c_uint);
    EnterCriticalSection(&raw mut (*session).cs);
    hr = session_is_shut_down(session);
    if hr >= 0 {
        (*session).presentation.flags &= !SESSION_FLAG_PRESENTATION_ENDING;
        if list_empty(&raw mut (*session).commands) != 0
            && (*session).command_state as ::core::ffi::c_uint == (COMMAND_STATE_COMPLETE as u32)
        {
            hr = MFPutWorkItem(
                MFASYNC_CALLBACK_QUEUE_STANDARD,
                &raw mut (*session).commands_callback,
                &raw mut (*op).IUnknown_iface,
            );
            /* Since session->commands is empty, whether `op` is added to head or tail, it will
             * always be at the head of this list, thus the invariant holds. */
            if hr >= 0 {
                (*session).command_state = COMMAND_STATE_SUBMITTED;
            }
        }
        if (*op).command as ::core::ffi::c_uint == SESSION_CMD_SHUTDOWN as ::core::ffi::c_uint {
            list_add_head(&raw mut (*session).commands, &raw mut (*op).entry);
        } else {
            list_add_tail(&raw mut (*session).commands, &raw mut (*op).entry);
        }
        IUnknown_AddRef(&raw mut (*op).IUnknown_iface);
    }
    LeaveCriticalSection(&raw mut (*session).cs);

    return hr;
}

unsafe extern "C" fn session_submit_simple_command(session: *mut media_session, command: session_command) -> HRESULT {
    let mut op: *mut session_op = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = create_session_op(command, core::ptr::addr_of_mut!(op));
    if hr < 0 {
        return hr;
    }

    hr = session_submit_command(session, op);
    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));
    return hr;
}

unsafe extern "C" fn session_clear_queued_topologies(session: *mut media_session) {
    let mut ptr: *mut queued_topology = ((*session).topologies.next as *mut u8)
        .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    let mut next: *mut queued_topology = ((*ptr).entry.next as *mut u8)
        .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    while core::ptr::addr_of_mut!((*ptr).entry) != core::ptr::addr_of_mut!((*session).topologies) {
        list_remove(core::ptr::addr_of_mut!((*ptr).entry));
        list_add_tail(
            core::ptr::addr_of_mut!((*session).removed_topologies),
            core::ptr::addr_of_mut!((*ptr).entry),
        );
        ptr = next;
        next = ((*ptr).entry.next as *mut u8)
            .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    }
}

unsafe extern "C" fn session_set_topo_status(session: *mut media_session, status: HRESULT, topo_status: MF_TOPOSTATUS) {
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();
    let mut param: PROPVARIANT = core::mem::zeroed();

    if topo_status as ::core::ffi::c_uint == MF_TOPOSTATUS_INVALID as ::core::ffi::c_uint {
        return;
    }

    if list_empty(core::ptr::addr_of_mut!((*session).topologies)) != 0 {
        if ((*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int))
            != 0
        {
            let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
            ros_dbg_log(
                (__dbcl) as u32,
                __dbch,
                b"dll/win32/mf/session.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"session_set_topo_status\0" as *const u8 as *const ::core::ffi::c_char,
                549 as ::core::ffi::c_int,
                b"Unexpectedly empty topology queue.\n\0" as *const u8
                    as *const ::core::ffi::c_char,
            );
        }
        return;
    }

    if topo_status as ::core::ffi::c_uint
        > (*session).presentation.topo_status as ::core::ffi::c_uint
    {
        let topology: *mut queued_topology = (list_head(
            core::ptr::addr_of_mut!((*session).topologies),
        ) as *mut u8)
            .offset(-(core::mem::offset_of!(queued_topology, entry) as isize))
            as *mut queued_topology;

        param.vt = VT_UNKNOWN as VARTYPE;
        param.__bindgen_anon_1.punkVal = (*topology).topology as *mut IUnknown;

        if (MFCreateMediaEvent(
            MESessionTopologyStatus as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            status,
            core::ptr::addr_of_mut!(param),
            core::ptr::addr_of_mut!(event),
        )) < 0 as HRESULT
        {
            return;
        }

        (*session).presentation.topo_status = topo_status;

        IMFMediaEvent_SetUINT32(
            event,
            core::ptr::addr_of!(MF_EVENT_TOPOLOGY_STATUS),
            topo_status as UINT32,
        );
        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
        IMFMediaEvent_Release(event);
    }
}

unsafe extern "C" fn session_bind_output_nodes(mut topology: *mut IMFTopology) -> HRESULT {
    let mut stream_sink: *mut IMFStreamSink = ::core::ptr::null_mut::<IMFStreamSink>();
    let mut media_sink: *mut IMFMediaSink = ::core::ptr::null_mut::<IMFMediaSink>();
    let mut node_count: WORD = 0 as WORD;
    let mut i: WORD = 0 as WORD;
    let mut node: *mut IMFTopologyNode = ::core::ptr::null_mut::<IMFTopologyNode>();
    let mut activate: *mut IMFActivate = ::core::ptr::null_mut::<IMFActivate>();
    let mut stream_id: UINT32 = 0;
    let mut object: *mut IUnknown = ::core::ptr::null_mut::<IUnknown>();
    let mut hr: HRESULT = 0;

    hr = IMFTopology_GetNodeCount(topology, &raw mut node_count);

    i = 0 as WORD;
    while (i as ::core::ffi::c_int) < node_count as ::core::ffi::c_int {
        hr = IMFTopology_GetNode(topology, i, &raw mut node);
        if hr < 0 as HRESULT {
            break;
        }
        if topology_node_get_type(node) as ::core::ffi::c_uint
            != MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint
        {
            IMFTopologyNode_Release(node);
        } else {
            hr = IMFTopologyNode_GetObject(node, &raw mut object);
            if hr >= 0 as HRESULT {
                stream_sink = ::core::ptr::null_mut::<IMFStreamSink>();
                if IUnknown_QueryInterface(
                    object,
                    &raw const IID_IMFStreamSink,
                    &raw mut stream_sink as *mut *mut ::core::ffi::c_void,
                ) < 0 as HRESULT
                {
                    hr = IUnknown_QueryInterface(
                        object,
                        &raw const IID_IMFActivate,
                        &raw mut activate as *mut *mut ::core::ffi::c_void,
                    );
                    if hr >= 0 as HRESULT {
                        hr = IMFActivate_ActivateObject(
                            activate,
                            &raw const IID_IMFMediaSink,
                            &raw mut media_sink as *mut *mut ::core::ffi::c_void,
                        );
                        if hr >= 0 as HRESULT {
                            if IMFTopologyNode_GetUINT32(
                                node,
                                &raw const MF_TOPONODE_STREAMID,
                                &raw mut stream_id,
                            ) < 0 as HRESULT
                            {
                                stream_id = 0 as UINT32;
                            }
                            stream_sink = ::core::ptr::null_mut::<IMFStreamSink>();
                            if IMFMediaSink_GetStreamSinkById(
                                media_sink,
                                stream_id as DWORD,
                                &raw mut stream_sink,
                            ) < 0 as HRESULT
                            {
                                hr = IMFMediaSink_AddStreamSink(
                                    media_sink,
                                    stream_id as DWORD,
                                    ::core::ptr::null_mut::<IMFMediaType>(),
                                    &raw mut stream_sink,
                                );
                            }
                            if !stream_sink.is_null() {
                                hr = IMFTopologyNode_SetObject(node, stream_sink as *mut IUnknown);
                            }
                            IMFMediaSink_Release(media_sink);
                        }
                        if hr >= 0 as HRESULT {
                            IMFTopologyNode_SetUnknown(
                                node,
                                &raw const _MF_TOPONODE_IMFActivate,
                                activate as *mut IUnknown,
                            );
                        }
                        IMFActivate_Release(activate);
                    }
                }
                if !stream_sink.is_null() {
                    IMFStreamSink_Release(stream_sink);
                }
                IUnknown_Release(object);
            }
            IMFTopologyNode_Release(node);
        }
        i = i.wrapping_add(1);
    }

    return hr;
}

unsafe extern "C" fn session_init_media_types(mut topology: *mut IMFTopology) -> HRESULT {
    let mut node_count: WORD = 0;
    let mut i: WORD = 0;
    let mut j: WORD = 0;
    let mut node: *mut IMFTopologyNode = ::core::ptr::null_mut::<IMFTopologyNode>();
    let mut type_0: *mut IMFMediaType = ::core::ptr::null_mut::<IMFMediaType>();
    let mut input_count: DWORD = 0;
    let mut hr: HRESULT = 0;

    hr = IMFTopology_GetNodeCount(topology, &raw mut node_count);
    if hr < 0 {
        return hr;
    }

    i = 0 as WORD;
    while (i as ::core::ffi::c_int) < (node_count as ::core::ffi::c_int) {
        hr = IMFTopology_GetNode(topology, i, &raw mut node);
        if hr < 0 {
            break;
        }

        hr = IMFTopologyNode_GetInputCount(node, &raw mut input_count);
        if hr < 0
            || (topology_node_get_type(node) as ::core::ffi::c_uint)
                != (MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint)
        {
            IMFTopologyNode_Release(node);
        } else {
            j = 0 as WORD;
            while (j as DWORD) < input_count {
                let mut handler: *mut IMFMediaTypeHandler =
                    ::core::ptr::null_mut::<IMFMediaTypeHandler>();
                let mut up_node: *mut IMFTopologyNode = ::core::ptr::null_mut::<IMFTopologyNode>();
                let mut up_output: DWORD = 0;

                hr = IMFTopologyNode_GetInput(node, j as DWORD, &raw mut up_node, &raw mut up_output);
                if hr >= 0 {
                    hr = topology_node_init_media_type(
                        up_node,
                        up_output,
                        (TRUE) as BOOL,
                        &raw mut type_0,
                    );
                    IMFTopologyNode_Release(up_node);
                }
                if hr < 0 {
                    break;
                }

                hr = topology_node_get_type_handler(
                    node,
                    j as DWORD,
                    (FALSE) as BOOL,
                    &raw mut handler,
                );
                if hr >= 0 {
                    hr = IMFMediaTypeHandler_SetCurrentMediaType(handler, type_0);
                    IMFMediaTypeHandler_Release(handler);
                }

                IMFMediaType_Release(type_0);
                j = j.wrapping_add(1);
            }

            IMFTopologyNode_Release(node);
        }
        i = i.wrapping_add(1);
    }

    return hr;
}

unsafe extern "C" fn session_set_caps(session: *mut media_session, caps: DWORD) {
    let mut delta: DWORD = (*session).caps ^ caps;
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();

    /* Delta is documented to reflect rate value changes as well, but it's not clear what to compare
       them to, since session always queries for current object rates. */
    if delta == 0 {
        return;
    }

    (*session).caps = caps;

    if (MFCreateMediaEvent(
        MESessionCapabilitiesChanged as MediaEventType,
        core::ptr::addr_of!(GUID_NULL),
        S_OK as HRESULT,
        core::ptr::null::<PROPVARIANT>(),
        core::ptr::addr_of_mut!(event),
    ) < 0 as HRESULT)
    {
        return;
    }

    IMFMediaEvent_SetUINT32(event, core::ptr::addr_of!(MF_EVENT_SESSIONCAPS), caps as UINT32);
    IMFMediaEvent_SetUINT32(event, core::ptr::addr_of!(MF_EVENT_SESSIONCAPS_DELTA), delta as UINT32);

    IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
    IMFMediaEvent_Release(event);
}

unsafe extern "C" fn transform_stream_push_sample(
    mut stream: *mut transform_stream,
    mut sample: *mut IMFSample,
) -> HRESULT {
    let mut value: PROPVARIANT = tagPROPVARIANT {
        vt: VT_UNKNOWN as VARTYPE,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        __bindgen_anon_1: tagPROPVARIANT__bindgen_ty_1 { punkVal: sample as *mut IUnknown,
        },
    };
    let mut entry: *mut event_entry = core::ptr::null_mut::<event_entry>();
    let mut hr: HRESULT = 0;
    entry = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<event_entry>() as size_t) as _,
    ) as *mut event_entry;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = MFCreateMediaEvent(
        MEMediaSample as core::ffi::c_int as MediaEventType,
        core::ptr::addr_of!(GUID_NULL),
        S_OK as HRESULT,
        core::ptr::addr_of_mut!(value),
        core::ptr::addr_of_mut!((*entry).event),
    );
    if hr < 0 {
        free(entry as *mut core::ffi::c_void);
        return hr;
    }
    list_add_tail(
        core::ptr::addr_of_mut!((*stream).samples),
        core::ptr::addr_of_mut!((*entry).entry),
    );
    return S_OK;
}

unsafe extern "C" fn transform_stream_push_format_change(
    mut stream: *mut transform_stream,
    mut media_type: *mut IMFMediaType,
) -> HRESULT {
    let mut value: PROPVARIANT = tagPROPVARIANT {
        vt: VT_UNKNOWN as VARTYPE,
        wReserved1: 0,
        wReserved2: 0,
        wReserved3: 0,
        __bindgen_anon_1: tagPROPVARIANT__bindgen_ty_1 { punkVal: media_type as *mut IUnknown,
        },
    };
    let mut entry: *mut event_entry = ::core::ptr::null_mut::<event_entry>();
    let mut hr: HRESULT = 0;
    entry = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<event_entry>() as size_t) as _,
    ) as *mut event_entry;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }
    hr = MFCreateMediaEvent(
        MEStreamFormatChanged as ::core::ffi::c_int as MediaEventType,
        &raw const GUID_NULL,
        S_OK as HRESULT,
        &raw mut value,
        &raw mut (*entry).event,
    );
    if hr < 0 as HRESULT {
        free(entry as *mut ::core::ffi::c_void);
        return hr;
    }
    list_add_tail(&raw mut (*stream).samples, &raw mut (*entry).entry);
    return S_OK;
}

unsafe extern "C" fn transform_stream_push_events(
    mut stream: *mut transform_stream,
    mut events: *mut IMFCollection,
) -> HRESULT {
    let mut entry: *mut event_entry = core::ptr::null_mut::<event_entry>();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut::<IMFMediaEvent>();
    while IMFCollection_RemoveElement(
        events,
        0 as DWORD,
        core::ptr::addr_of_mut!(event) as *mut *mut IUnknown,
    ) >= 0 as HRESULT
    {
        entry = calloc(
            (1 as size_t) as _,
            (core::mem::size_of::<event_entry>() as size_t) as _,
        ) as *mut event_entry;
        if entry.is_null() {
            IMFMediaEvent_Release(event);
            return E_OUTOFMEMORY as HRESULT;
        }
        (*entry).event = event;
        list_add_tail(
            core::ptr::addr_of_mut!((*stream).samples),
            core::ptr::addr_of_mut!((*entry).entry),
        );
    }
    return S_OK as HRESULT;
}

unsafe extern "C" fn transform_stream_pop_event(
    mut stream: *mut transform_stream,
    mut event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let mut entry: *mut event_entry = ::core::ptr::null_mut::<event_entry>();
    let mut ptr: *mut list = ::core::ptr::null_mut::<list>();
    ptr = list_head(core::ptr::addr_of_mut!((*stream).samples));
    if ptr.is_null() {
        return MF_E_TRANSFORM_NEED_MORE_INPUT;
    }
    entry = (ptr as *mut u8).sub(core::mem::offset_of!(event_entry, entry)) as *mut event_entry;
    list_remove(core::ptr::addr_of_mut!((*entry).entry));
    *event = (*entry).event;
    free(entry as *mut core::ffi::c_void);
    return S_OK;
}

unsafe extern "C" fn transform_stream_drop_events(stream: *mut transform_stream) {
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();
    while (transform_stream_pop_event(stream, core::ptr::addr_of_mut!(event)) >= 0) {
        IMFMediaEvent_Release(event);
    }
}

unsafe extern "C" fn release_topo_node(mut node: *mut topo_node) {
    let mut i: ::core::ffi::c_uint = 0;
    match (*node).type_ as ::core::ffi::c_uint {
        MF_TOPOLOGY_SOURCESTREAM_NODE => {
            if !(*node).u.source.source.is_null() {
                IMFMediaSource_Release((*node).u.source.source);
            }
        }
        MF_TOPOLOGY_TRANSFORM_NODE => {
            i = 0;
            while i < (*node).u.transform.input_count {
                transform_stream_drop_events(
                    (*node).u.transform.inputs.offset(i as isize) as *mut transform_stream
                );
                i = i.wrapping_add(1);
            }
            i = 0;
            while i < (*node).u.transform.output_count {
                transform_stream_drop_events(
                    (*node).u.transform.outputs.offset(i as isize) as *mut transform_stream
                );
                i = i.wrapping_add(1);
            }
            free((*node).u.transform.inputs as *mut ::core::ffi::c_void);
            free((*node).u.transform.outputs as *mut ::core::ffi::c_void);
            free((*node).u.transform.input_map as *mut ::core::ffi::c_void);
            free((*node).u.transform.output_map as *mut ::core::ffi::c_void);
            if !(*node).u.transform.event_source.is_null() {
                IMFMediaEventGenerator_Release((*node).u.transform.event_source);
            }
        }
        MF_TOPOLOGY_OUTPUT_NODE => {
            if !(*node).u.sink.allocator.is_null() {
                IMFVideoSampleAllocator_Release((*node).u.sink.allocator);
            }
            if !(*node).u.sink.allocator_cb.is_null() {
                IMFVideoSampleAllocatorCallback_SetCallback(
                    (*node).u.sink.allocator_cb,
                    ::core::ptr::null_mut::<IMFVideoSampleAllocatorNotify>(),
                );
                IMFVideoSampleAllocatorCallback_Release((*node).u.sink.allocator_cb);
            }
        }
        _ => {}
    }
    if !(*node).object.object.is_null() {
        IUnknown_Release((*node).object.object);
    }
    if !(*node).node.is_null() {
        IMFTopologyNode_Release((*node).node);
    }
    free(node as *mut ::core::ffi::c_void);
}

unsafe extern "C" fn topology_shutdown(mut topology: *mut IMFTopology) {
    let mut stream_sink: *mut IMFStreamSink = core::ptr::null_mut();
    let mut attributes: *mut IMFAttributes = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut activate: *mut IMFActivate = core::ptr::null_mut();
    let mut sink: *mut IMFMediaSink = core::ptr::null_mut();
    let mut idx: WORD = 0;
    let mut hr: HRESULT = 0;

    loop {
        let fresh = idx;
        idx = idx.wrapping_add(1);
        if !(IMFTopology_GetNode(topology, fresh, core::ptr::addr_of_mut!(node)) >= 0) {
            break;
        }
        let node_type: MF_TOPOLOGY_TYPE = topology_node_get_type(node);

        if node_type as u32 == MF_TOPOLOGY_OUTPUT_NODE as u32 {
            /* MF_TOPONODE_NOSHUTDOWN_ON_REMOVE is ignored, at least for sinks. */

            if IMFTopologyNode_GetUnknown(
                node,
                core::ptr::addr_of!(_MF_TOPONODE_IMFActivate),
                core::ptr::addr_of!(IID_IMFActivate),
                core::ptr::addr_of_mut!(activate) as *mut LPVOID,
            ) >= 0
            {
                hr = IMFActivate_ShutdownObject(activate);
                if hr < 0 {
                    WARN!("Failed to shut down activation object for the sink, hr %#lx.\n", hr);
                }
                IMFActivate_Release(activate);
            }
            if topology_node_get_object(
                node,
                core::ptr::addr_of!(IID_IMFStreamSink),
                core::ptr::addr_of_mut!(stream_sink) as *mut *mut core::ffi::c_void,
            ) >= 0
            {
                if IMFStreamSink_GetMediaSink(stream_sink, core::ptr::addr_of_mut!(sink)) >= 0 {
                    IMFMediaSink_Shutdown(sink);
                    IMFMediaSink_Release(sink);
                }

                IMFStreamSink_Release(stream_sink);
            }
        } else if node_type as u32 == MF_TOPOLOGY_TRANSFORM_NODE as u32
            && {
                attributes = topology_node_transform_async_get_attributes(node);
                !attributes.is_null()
            }
        {
            let mut shutdown: *mut IMFShutdown = core::ptr::null_mut();
            hr = topology_node_get_object(
                node,
                core::ptr::addr_of!(IID_IMFShutdown),
                core::ptr::addr_of_mut!(shutdown) as *mut *mut core::ffi::c_void,
            );
            if hr >= 0 {
                IMFShutdown_Shutdown(shutdown);
                IMFShutdown_Release(shutdown);
            } else {
                WARN!("Failed to get shutdown interface, hr %#lx.\n", hr);
            }
            IMFAttributes_Release(attributes);
        }

        IMFTopologyNode_Release(node);
    }
}

unsafe extern "C" fn session_clear_removed_topologies(session: *mut media_session) {
    let mut ptr: *mut queued_topology = core::ptr::null_mut::<queued_topology>();
    let mut next: *mut queued_topology = core::ptr::null_mut::<queued_topology>();
    ptr = ((*session).removed_topologies.next as *mut u8)
        .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    next = ((*ptr).entry.next as *mut u8)
        .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    while core::ptr::addr_of_mut!((*ptr).entry)
        != core::ptr::addr_of_mut!((*session).removed_topologies)
    {
        list_remove(core::ptr::addr_of_mut!((*ptr).entry));
        topology_shutdown((*ptr).topology);
        IMFTopology_Release((*ptr).topology);
        free(ptr as *mut core::ffi::c_void);
        ptr = next;
        next = ((*ptr).entry.next as *mut u8)
            .sub(core::mem::offset_of!(queued_topology, entry)) as *mut queued_topology;
    }
}

unsafe extern "C" fn session_clear_command_list(session: *mut media_session) {
    let mut op: *mut session_op = core::ptr::null_mut();
    let mut op2: *mut session_op = core::ptr::null_mut();
    op = ((*session).commands.next as *mut u8).sub(core::mem::offset_of!(session_op, entry)) as *mut session_op;
    op2 = ((*op).entry.next as *mut u8).sub(core::mem::offset_of!(session_op, entry)) as *mut session_op;
    while &raw mut (*op).entry != &raw mut (*session).commands {
        list_remove(&raw mut (*op).entry);
        IUnknown_Release(&raw mut (*op).IUnknown_iface);
        op = op2;
        op2 = ((*op).entry.next as *mut u8).sub(core::mem::offset_of!(session_op, entry)) as *mut session_op;
    }
}

unsafe extern "C" fn session_clear_presentation(session: *mut media_session) {
    let mut source: *mut media_source = core::ptr::null_mut();
    let mut source2: *mut media_source = core::ptr::null_mut();
    let mut sink: *mut media_sink = core::ptr::null_mut();
    let mut sink2: *mut media_sink = core::ptr::null_mut();
    let mut node: *mut topo_node = core::ptr::null_mut();
    let mut node2: *mut topo_node = core::ptr::null_mut();

    IMFTopology_Clear((*session).presentation.current_topology);
    (*session).presentation.topo_status = MF_TOPOSTATUS_INVALID;
    (*session).presentation.flags = 0;

    source = ((*session).presentation.sources.next as *mut u8)
        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    source2 = ((*source).entry.next as *mut u8)
        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    while core::ptr::addr_of_mut!((*source).entry) != core::ptr::addr_of_mut!((*session).presentation.sources) {
        list_remove(core::ptr::addr_of_mut!((*source).entry));
        session_release_media_source(source);
        source = source2;
        source2 = ((*source).entry.next as *mut u8)
            .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    }

    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    node2 = ((*node).entry.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry) != core::ptr::addr_of_mut!((*session).presentation.nodes) {
        list_remove(core::ptr::addr_of_mut!((*node).entry));
        release_topo_node(node);
        node = node2;
        node2 = ((*node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }

    sink = ((*session).presentation.sinks.next as *mut u8)
        .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
    sink2 = ((*sink).entry.next as *mut u8)
        .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
    while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*session).presentation.sinks) {
        list_remove(core::ptr::addr_of_mut!((*sink).entry));

        if !(*sink).__bindgen_anon_1.sink.is_null() {
            IMFMediaSink_Release((*sink).__bindgen_anon_1.sink);
        }
        if !(*sink).preroll.is_null() {
            IMFMediaSinkPreroll_Release((*sink).preroll);
        }
        if !(*sink).event_generator.is_null() {
            IMFMediaEventGenerator_Release((*sink).event_generator);
        }
        free(sink as *mut core::ffi::c_void);
        sink = sink2;
        sink2 = ((*sink).entry.next as *mut u8)
            .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
    }
}

unsafe extern "C" fn session_get_topo_node(
    session: *const media_session,
    node: *mut IMFTopologyNode,
) -> *mut topo_node {
    let mut topo_node: *mut topo_node = core::ptr::null_mut();
    let mut id: TOPOID = 0;
    if (IMFTopologyNode_GetTopoNodeID(node, core::ptr::addr_of_mut!(id)) as HRESULT) < 0 {
        return core::ptr::null_mut();
    }
    topo_node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*topo_node).entry)
        != core::ptr::addr_of!((*session).presentation.nodes) as *mut list
    {
        if (*topo_node).node_id == id {
            return topo_node;
        }
        topo_node = ((*topo_node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn session_get_topo_node_input(session: *const media_session, down_node: *const topo_node, input: DWORD, output: *mut DWORD) -> *mut topo_node {
    let mut up_node: *mut topo_node = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    if (IMFTopologyNode_GetInput((*down_node).node, input, core::ptr::addr_of_mut!(node), output)) >= 0 {
        up_node = session_get_topo_node(session, node);
        IMFTopologyNode_Release(node);
    }
    return up_node;
}

unsafe extern "C" fn session_get_topo_node_output(session: *const media_session, up_node: *const topo_node, output: DWORD, input: *mut DWORD) -> *mut topo_node {
    let mut down_node: *mut topo_node = core::ptr::null_mut();
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();

    if (IMFTopologyNode_GetOutput((*up_node).node, output, core::ptr::addr_of_mut!(node), input)) >= 0 {
        down_node = session_get_topo_node(session, node);
        IMFTopologyNode_Release(node);
    }

    return down_node;
}

unsafe extern "C" fn session_command_complete(session: *mut media_session) {
    let mut op: *mut session_op = core::ptr::null_mut();
    let mut e: *mut list;
    let mut hr: HRESULT;

    (*session).command_state = COMMAND_STATE_COMPLETE;

    /* Submit next command. */
    e = list_head(core::ptr::addr_of_mut!((*session).commands));
    if !e.is_null() {
        op = (e as *mut u8).sub(core::mem::offset_of!(session_op, entry)) as *mut session_op;
        hr = MFPutWorkItem(
            MFASYNC_CALLBACK_QUEUE_STANDARD,
            core::ptr::addr_of_mut!((*session).commands_callback),
            core::ptr::addr_of_mut!((*op).IUnknown_iface),
        );
        if hr >= 0 {
            (*session).command_state = COMMAND_STATE_SUBMITTED;
        }
    }
}

unsafe extern "C" fn session_command_complete_with_event(session: *mut media_session, event: MediaEventType, status: HRESULT, param: *const PROPVARIANT) {
    IMFMediaEventQueue_QueueEventParamVar((*session).event_queue, event, &raw const GUID_NULL, status, param);
    session_command_complete(session);
}

unsafe extern "C" fn session_subscribe_sources(session: *mut media_session) -> HRESULT {
    let mut source: *mut media_source = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;

    if (*session).presentation.flags & SESSION_FLAG_SOURCES_SUBSCRIBED != 0 {
        return hr;
    }

    source = ((*session).presentation.sources.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    while core::ptr::addr_of_mut!((*source).entry)
        != core::ptr::addr_of_mut!((*session).presentation.sources)
    {
        hr = IMFMediaSource_BeginGetEvent(
            (*source).__bindgen_anon_1.source,
            core::ptr::addr_of_mut!((*session).events_callback),
            (*source).__bindgen_anon_1.object,
        );
        if hr < 0 {
            WARN!("Failed to subscribe to source events, hr %#lx.\n", hr);
            return hr;
        }
        source = ((*source).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    }

    (*session).presentation.flags |= SESSION_FLAG_SOURCES_SUBSCRIBED;
    return hr;
}

unsafe extern "C" fn session_subscribe_sinks(mut session: *mut media_session) {
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut hr: HRESULT = 0;
    if ((*session).presentation.flags & (SESSION_FLAG_SINKS_SUBSCRIBED as u32)) != 0 {
        return;
    }
    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry) != core::ptr::addr_of_mut!((*session).presentation.nodes) {
        if !(((*node).type_ as u32) != (MF_TOPOLOGY_OUTPUT_NODE as u32)) {
            hr = IMFStreamSink_BeginGetEvent(
                (*node).object.sink_stream,
                core::ptr::addr_of_mut!((*session).events_callback),
                (*node).object.object,
            );
            if hr < (0 as HRESULT) {
                WARN!(
                    "Failed to subscribe to stream sink events, hr %#lx.\n",
                    hr
                );
            }
        }
        node = ((*node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }
    (*session).presentation.flags |= SESSION_FLAG_SINKS_SUBSCRIBED as u32;
}

unsafe extern "C" fn session_flush_transform_node(mut node: *mut topo_node) {
    let mut stream: *mut transform_stream = ::core::ptr::null_mut::<transform_stream>();
    let mut i: UINT = 0;
    if (*node).type_ as ::core::ffi::c_uint
        == MF_TOPOLOGY_TRANSFORM_NODE as ::core::ffi::c_uint
    {
        i = 0 as UINT;
        while i < (*node).u.transform.input_count {
            stream = (*node).u.transform.inputs.offset(i as isize) as *mut transform_stream;
            transform_stream_drop_events(stream);
            (*stream).draining = FALSE as BOOL;
            i = i.wrapping_add(1);
        }
        IMFTransform_ProcessMessage(
            (*node).object.transform,
            MFT_MESSAGE_COMMAND_FLUSH,
            0 as ULONG_PTR,
        );
        i = 0 as UINT;
        while i < (*node).u.transform.output_count {
            stream = (*node).u.transform.outputs.offset(i as isize) as *mut transform_stream;
            transform_stream_drop_events(stream);
            (*stream).requests = 0 as ::core::ffi::c_uint;
            i = i.wrapping_add(1);
        }
    }
}

unsafe extern "C" fn session_flush_transforms(session: *mut media_session) {
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        session_flush_transform_node(node);
        node = ((*node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }
}

unsafe extern "C" fn session_flush_sinks(mut session: *mut media_session) {
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
        -(core::ptr::addr_of_mut!((*::core::ptr::null_mut::<topo_node>()).entry)
            as ::core::ffi::c_ulonglong as isize),
    ) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        if (*node).type_ as ::core::ffi::c_uint
            == MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint
        {
            if (*node).u.sink.requests != 0 {
                (*node).u.sink.requests = (*node).u.sink.requests.wrapping_sub(1);
                session_request_sample(session, (*node).object.sink_stream);
            }
            IMFStreamSink_Flush((*node).object.sink_stream);
        }
        node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::ptr::addr_of_mut!((*::core::ptr::null_mut::<topo_node>()).entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut topo_node;
    }
}

unsafe extern "C" fn session_flush_nodes(session: *mut media_session) {
    let mut node: *mut topo_node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char)
        .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        if (*node).type_ as ::core::ffi::c_uint == MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint {
            IMFStreamSink_Flush((*node).object.sink_stream);
        } else {
            session_flush_transform_node(node);
        }
        node = ((*node).entry.next as *mut ::core::ffi::c_char)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    }
}

unsafe extern "C" fn session_start(
    session: *mut media_session,
    time_format: *const GUID,
    start_position: *const PROPVARIANT,
) {
    let mut source: *mut media_source = core::ptr::null_mut::<media_source>();
    let mut topo_node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut keep_position: BOOL;
    let mut duration: MFTIME = 0;
    let mut hr: HRESULT = 0;
    let mut i: UINT = 0;
    let mut enter_start: bool = false;

    keep_position = (IsEqualGUID!(time_format, &GUID_NULL)
        && (*start_position).vt == VT_EMPTY) as BOOL;

    /* No position change - nothing to do. */
    if (*session).state == SESSION_STATE_STARTED && keep_position != 0 {
        session_command_complete_with_event(
            session,
            MESessionStarted,
            S_OK,
            core::ptr::null::<PROPVARIANT>(),
        );
        return;
    }

    match (*session).state {
        SESSION_STATE_PAUSED | SESSION_STATE_STARTED => {
            if keep_position == 0 {
                (*session).command_state = COMMAND_STATE_RESTARTING_SOURCES;

                /* We are seeking to a new position, check for invalid positions */
                source = ((*session).presentation.sources.next as *mut u8)
                    .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
                while core::ptr::addr_of_mut!((*source).entry)
                    != core::ptr::addr_of_mut!((*session).presentation.sources)
                {
                    hr = IMFPresentationDescriptor_GetUINT64(
                        (*source).pd,
                        &MF_PD_DURATION,
                        core::ptr::addr_of_mut!(duration) as *mut UINT64,
                    );
                    if ((hr) >= 0)
                        && IsEqualGUID!(time_format, &GUID_NULL)
                        && (*start_position).vt == (VT_I8 as u16)
                        && (*start_position).__bindgen_anon_1.hVal.QuadPart > duration
                    {
                        WARN!(
                            "Start position %s out of range, hr %#lx.\n",
                            wine_dbgstr_longlong(((*start_position).__bindgen_anon_1.hVal.QuadPart) as ULONGLONG),
                            hr
                        );
                        session_command_complete_with_event(
                            session,
                            MESessionStarted,
                            MF_E_INVALID_POSITION,
                            core::ptr::null::<PROPVARIANT>(),
                        );
                        return;
                    }
                    source = ((*source).entry.next as *mut u8)
                        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
                }

                /* Stop sources */
                source = ((*session).presentation.sources.next as *mut u8)
                    .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
                while core::ptr::addr_of_mut!((*source).entry)
                    != core::ptr::addr_of_mut!((*session).presentation.sources)
                {
                    hr = IMFMediaSource_Stop((*source).__bindgen_anon_1.source);
                    if hr < 0 {
                        WARN!(
                            "Failed to stop media source %p, hr %#lx.\n",
                            (*source).__bindgen_anon_1.source,
                            hr
                        );
                        if hr == MF_E_SHUTDOWN {
                            return session_handle_source_shutdown(session);
                        }
                        return session_command_complete_with_event(
                            session,
                            MESessionStarted,
                            hr,
                            core::ptr::null::<PROPVARIANT>(),
                        );
                    }
                    source = ((*source).entry.next as *mut u8)
                        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
                }

                (*session).presentation.time_format = *time_format;
                (*session).presentation.start_position.vt = VT_EMPTY;
                PropVariantCopy(
                    core::ptr::addr_of_mut!((*session).presentation.start_position),
                    start_position,
                );

                // break;
            } else {
                /* fallthrough; we're resuming from the current position */
                enter_start = true;
            }
        }
        SESSION_STATE_STOPPED => {
            enter_start = true;
        }
        SESSION_STATE_CLOSED | SESSION_STATE_SHUT_DOWN => {
            session_command_complete_with_event(
                session,
                MESessionStarted,
                MF_E_INVALIDREQUEST,
                core::ptr::null::<PROPVARIANT>(),
            );
        }
        _ => {}
    }

    if enter_start {
        /* Start request with no current topology. */
        if (*session).presentation.topo_status == MF_TOPOSTATUS_INVALID {
            session_command_complete_with_event(
                session,
                MESessionStarted,
                MF_E_INVALIDREQUEST,
                core::ptr::null::<PROPVARIANT>(),
            );
        } else {
            (*session).command_state = COMMAND_STATE_STARTING_SOURCES;

            (*session).presentation.time_format = *time_format;
            (*session).presentation.start_position.vt = VT_EMPTY;
            PropVariantCopy(
                core::ptr::addr_of_mut!((*session).presentation.start_position),
                start_position,
            );

            hr = session_subscribe_sources(session);
            if hr < 0 {
                if hr == MF_E_SHUTDOWN {
                    return session_handle_source_shutdown(session);
                }
                return session_command_complete_with_event(
                    session,
                    MESessionStarted,
                    hr,
                    core::ptr::null::<PROPVARIANT>(),
                );
            }

            source = ((*session).presentation.sources.next as *mut u8)
                .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
            while core::ptr::addr_of_mut!((*source).entry)
                != core::ptr::addr_of_mut!((*session).presentation.sources)
            {
                hr = IMFMediaSource_Start(
                    (*source).__bindgen_anon_1.source,
                    (*source).pd,
                    &GUID_NULL,
                    start_position,
                );
                if hr < 0 {
                    WARN!(
                        "Failed to start media source %p, hr %#lx.\n",
                        (*source).__bindgen_anon_1.source,
                        hr
                    );
                    if hr == MF_E_SHUTDOWN {
                        return session_handle_source_shutdown(session);
                    }
                    return session_command_complete_with_event(
                        session,
                        MESessionStarted,
                        hr,
                        core::ptr::null::<PROPVARIANT>(),
                    );
                }
                source = ((*source).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
            }

            topo_node = ((*session).presentation.nodes.next as *mut u8)
                .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
            while core::ptr::addr_of_mut!((*topo_node).entry)
                != core::ptr::addr_of_mut!((*session).presentation.nodes)
            {
                if (*topo_node).type_ == MF_TOPOLOGY_TRANSFORM_NODE {
                    i = 0;
                    while i < (*topo_node).u.transform.input_count {
                        let mut stream: *mut transform_stream =
                            (*topo_node).u.transform.inputs.offset(i as isize);
                        (*stream).draining = (FALSE as i32);
                        i = i.wrapping_add(1);
                    }
                }
                topo_node = ((*topo_node).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
            }
        }
    }
}

unsafe extern "C" fn session_set_started(mut session: *mut media_session) {
    let mut source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut event: *mut IMFMediaEvent = ::core::ptr::null_mut::<IMFMediaEvent>();
    let mut caps: ::core::ffi::c_uint = 0;
    let mut flags: DWORD = 0;
    (*session).state = SESSION_STATE_STARTED;
    caps = ((*session).caps | (MFSESSIONCAP_PAUSE as DWORD)) as ::core::ffi::c_uint;
    source = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut media_source;
    while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
        if IMFMediaSource_GetCharacteristics((*source).__bindgen_anon_1.source, &raw mut flags)
            >= (0 as HRESULT)
        {
            if (flags & (MFMEDIASOURCE_CAN_PAUSE as DWORD)) == 0 {
                caps &= !(MFSESSIONCAP_PAUSE as ::core::ffi::c_uint);
                break;
            }
        }
        source = ((*source).entry.next as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut media_source;
    }
    session_set_caps(session, caps as DWORD);
    if MFCreateMediaEvent(
        MESessionStarted as ::core::ffi::c_int as MediaEventType,
        &raw const GUID_NULL,
        S_OK as HRESULT,
        ::core::ptr::null::<PROPVARIANT>(),
        &raw mut event,
    ) >= (0 as HRESULT)
    {
        IMFMediaEvent_SetUINT64(
            event,
            &raw const MF_EVENT_PRESENTATION_TIME_OFFSET,
            0 as UINT64,
        );
        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
        IMFMediaEvent_Release(event);
    }
    session_command_complete(session);
}

unsafe extern "C" fn session_set_paused(mut session: *mut media_session, mut status: HRESULT) {
    /* Failed event status could indicate a failure during normal transition to paused state,
       or an attempt to pause from invalid initial state. To finalize failed transition in the former case,
       state is still forced to PAUSED, otherwise previous state is retained. */
    (*session).state = SESSION_STATE_PAUSED;
    if status >= 0 {
        session_set_caps(
            session,
            (*session).caps & !(MFSESSIONCAP_PAUSE as DWORD),
        );
    }
    session_command_complete_with_event(
        session,
        MESessionPaused as MediaEventType,
        status,
        core::ptr::null::<PROPVARIANT>(),
    );
}

unsafe extern "C" fn session_set_closed(mut session: *mut media_session, mut status: HRESULT) {
    (*session).state = SESSION_STATE_CLOSED;
    if status >= 0 {
        session_set_caps(
            session,
            (*session).caps & !((MFSESSIONCAP_START | MFSESSIONCAP_SEEK) as DWORD),
        );
    }
    session_command_complete_with_event(
        session,
        MESessionClosed as MediaEventType,
        status,
        core::ptr::null::<PROPVARIANT>(),
    );
}

unsafe extern "C" fn session_pause(mut session: *mut media_session) {
    let mut hr: HRESULT = 0;
    match ((*session).state as ::core::ffi::c_uint) as i32 {
        SESSION_STATE_STARTED => {
            (*session).command_state = COMMAND_STATE_PAUSING_SINKS;
            hr = IMFPresentationClock_Pause((*session).clock);
            if hr < 0 as HRESULT {
                session_set_paused(session, hr);
            }
        }
        SESSION_STATE_STOPPED => {
            session_command_complete_with_event(
                session,
                MESessionPaused as ::core::ffi::c_int as MediaEventType,
                MF_E_SESSION_PAUSEWHILESTOPPED,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        SESSION_STATE_PAUSED | SESSION_STATE_CLOSED | SESSION_STATE_SHUT_DOWN => {
            session_command_complete_with_event(
                session,
                MESessionPaused as ::core::ffi::c_int as MediaEventType,
                MF_E_INVALIDREQUEST,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        _ => {}
    };
}

unsafe extern "C" fn session_clear_end_of_presentation(session: *mut media_session) {
    let mut source: *mut media_source;
    let mut node: *mut topo_node;
    source = ((*session).presentation.sources.next as *mut u8)
        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
        (*source).flags &= !(SOURCE_FLAG_END_OF_PRESENTATION as ::core::ffi::c_uint);
        source = ((*source).entry.next as *mut u8)
            .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    }
    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
        (*node).flags &= !(TOPO_NODE_END_OF_STREAM as ::core::ffi::c_uint);
        node = ((*node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }
    (*session).presentation.topo_status = MF_TOPOSTATUS_READY as _;
}

unsafe extern "C" fn session_set_stopped(session: *mut media_session, event_type: MediaEventType, status: HRESULT) {
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();

    (*session).presentation.flags &= !(SESSION_FLAG_PRESENTATION_ENDING as u32);
    (*session).state = SESSION_STATE_STOPPED;

    if (MFCreateMediaEvent(event_type, core::ptr::addr_of!(GUID_NULL), status, core::ptr::null::<PROPVARIANT>(), core::ptr::addr_of_mut!(event))) >= 0
    {
        IMFMediaEvent_SetUINT64(event, core::ptr::addr_of!(MF_SESSION_APPROX_EVENT_OCCURRENCE_TIME), (*session).presentation.clock_stop_time as UINT64);
        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
        IMFMediaEvent_Release(event);
    }
    session_clear_end_of_presentation(session);
    session_command_complete(session);
}

unsafe extern "C" fn session_stop(session: *mut media_session) {
    let mut hr: HRESULT = MF_E_INVALIDREQUEST;
    let state = (*session).state as ::core::ffi::c_int;

    if state == SESSION_STATE_STARTED || state == SESSION_STATE_PAUSED {
        /* Transition in two steps - stop the clock, wait for sinks, then stop sources. */
        IMFPresentationClock_GetTime((*session).clock, core::ptr::addr_of_mut!((*session).presentation.clock_stop_time));
        hr = IMFPresentationClock_Stop((*session).clock);
        if hr >= 0 {
            (*session).command_state = COMMAND_STATE_STOPPING_SINKS;
        } else {
            session_set_stopped(session, MESessionStopped as MediaEventType, hr);
        }
    } else if state == SESSION_STATE_STOPPED {
        hr = S_OK;
        /* fallthrough */
        session_command_complete_with_event(session, MESessionStopped as MediaEventType, hr, core::ptr::null::<PROPVARIANT>());
    } else if state == SESSION_STATE_CLOSED || state == SESSION_STATE_SHUT_DOWN {
        session_command_complete_with_event(session, MESessionStopped as MediaEventType, hr, core::ptr::null::<PROPVARIANT>());
    }
}

unsafe extern "C" fn session_finalize_sinks(session: *mut media_session) -> HRESULT {
    let mut fin_sink: *mut IMFFinalizableMediaSink = core::ptr::null_mut::<IMFFinalizableMediaSink>();
    let mut sinks_finalized: BOOL = (TRUE as i32);
    let mut sink: *mut media_sink = core::ptr::null_mut::<media_sink>();
    let mut hr: HRESULT = S_OK;

    (*session).command_state = COMMAND_STATE_FINALIZING_SINKS;

    sink = ((*session).presentation.sinks.next as *mut u8)
        .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
    while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*session).presentation.sinks) {
        if (IMFMediaSink_QueryInterface(
            (*sink).__bindgen_anon_1.sink,
            core::ptr::addr_of!(IID_IMFFinalizableMediaSink),
            &mut fin_sink as *mut *mut IMFFinalizableMediaSink as *mut *mut core::ffi::c_void,
        )) >= 0
        {
            hr = IMFFinalizableMediaSink_BeginFinalize(
                fin_sink,
                core::ptr::addr_of_mut!((*session).sink_finalizer_callback),
                fin_sink as *mut IUnknown,
            );
            IMFFinalizableMediaSink_Release(fin_sink);
            if hr < 0 {
                break;
            }
            sinks_finalized = (FALSE as i32);
        } else {
            (*sink).finalized = (TRUE as i32);
        }
        sink = ((*sink).entry.next as *mut u8)
            .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
    }

    if sinks_finalized != 0 {
        session_set_closed(session, hr);
    }

    return hr;
}

unsafe extern "C" fn session_close(session: *mut media_session) {
    let mut hr: HRESULT = S_OK;

    match ((*session).state as ::core::ffi::c_uint) as i32 {
        SESSION_STATE_STOPPED => {
            hr = session_finalize_sinks(session);
        }
        SESSION_STATE_STARTED | SESSION_STATE_PAUSED => {
            hr = IMFPresentationClock_Stop((*session).clock);
            if hr >= 0 {
                (*session).command_state = COMMAND_STATE_CLOSING_SINKS;
            }
        }
        SESSION_STATE_CLOSED | SESSION_STATE_SHUT_DOWN => {
            hr = MF_E_INVALIDREQUEST;
        }
        _ => {}
    }

    session_clear_queued_topologies(session);
    if hr < 0 {
        session_set_closed(session, hr);
    }
}

unsafe extern "C" fn session_clear_topologies(session: *mut media_session) {
    let mut hr: HRESULT = S_OK;

    if (*session).state as i32 == SESSION_STATE_CLOSED {
        hr = MF_E_INVALIDREQUEST;
    } else {
        session_clear_queued_topologies(session);
    }
    session_command_complete_with_event(session, 102 as MediaEventType /* MESessionTopologiesCleared */, hr, core::ptr::null());
}

unsafe extern "C" fn session_is_presentation_rate_supported(
    mut session: *mut media_session,
    mut thin: BOOL,
    mut rate: f32,
    mut nearest_rate: *mut f32,
) -> HRESULT {
    let mut rate_support: *mut IMFRateSupport = core::ptr::null_mut::<IMFRateSupport>();
    let mut source: *mut media_source = core::ptr::null_mut::<media_source>();
    let mut sink: *mut media_sink = core::ptr::null_mut::<media_sink>();
    let mut value: f32 = 0.0f32;
    let mut tmp: f32 = 0.0f32;
    let mut hr: HRESULT = S_OK;
    let mut flags: DWORD;

    if nearest_rate.is_null() {
        nearest_rate = &raw mut tmp;
    }

    if rate == 0.0f32 {
        *nearest_rate = 1.0f32;
        return S_OK;
    }

    if (*session).presentation.topo_status != MF_TOPOSTATUS_INVALID {
        // LIST_FOR_EACH_ENTRY(source, &session->presentation.sources, struct media_source, entry)
        source = ((*session).presentation.sources.next as *mut u8).offset(
            -(core::mem::offset_of!(media_source, entry) as isize),
        ) as *mut media_source;
        if &raw mut (*source).entry != &raw mut (*session).presentation.sources {
            if ({
                hr = MFGetService(
                    (*source).__bindgen_anon_1.object,
                    &raw const MF_RATE_CONTROL_SERVICE,
                    &raw const IID_IMFRateSupport,
                    &raw mut rate_support as *mut *mut core::ffi::c_void,
                );
                hr
            }) < 0
            {
                value = 1.0f32;
            } else {
                value = rate;
                if ({
                    hr = IMFRateSupport_IsRateSupported(rate_support, thin, rate, &raw mut value);
                    hr
                }) < 0
                {
                    WARN!(
                        "Source does not support rate %f, hr %#lx.\n",
                        rate as f64,
                        hr
                    );
                }
                IMFRateSupport_Release(rate_support);
            }
        }

        if hr >= 0 {
            // For sinks only check if rate is supported, ignoring nearest values.
            // LIST_FOR_EACH_ENTRY(sink, &session->presentation.sinks, struct media_sink, entry)
            sink = ((*session).presentation.sinks.next as *mut u8).offset(
                -(core::mem::offset_of!(media_sink, entry) as isize),
            ) as *mut media_sink;
            while &raw mut (*sink).entry != &raw mut (*session).presentation.sinks {
                flags = 0;
                hr = IMFMediaSink_GetCharacteristics((*sink).__bindgen_anon_1.sink, &raw mut flags);
                if hr < 0 {
                    break;
                }

                if (flags & MEDIASINK_RATELESS) == 0 {
                    if !(MFGetService(
                        (*sink).__bindgen_anon_1.object,
                        &raw const MF_RATE_CONTROL_SERVICE,
                        &raw const IID_IMFRateSupport,
                        &raw mut rate_support as *mut *mut core::ffi::c_void,
                    ) < 0)
                    {
                        hr = IMFRateSupport_IsRateSupported(
                            rate_support,
                            thin,
                            rate,
                            core::ptr::null_mut::<f32>(),
                        );
                        IMFRateSupport_Release(rate_support);
                        if hr < 0 {
                            WARN!(
                                "Sink %p does not support rate %f, hr %#lx.\n",
                                (*sink).__bindgen_anon_1.sink,
                                rate as f64,
                                hr
                            );
                            break;
                        }
                    }
                }

                sink = ((*sink).entry.next as *mut u8).offset(
                    -(core::mem::offset_of!(media_sink, entry) as isize),
                ) as *mut media_sink;
            }
        }
    }

    *nearest_rate = value;

    return hr;
}

unsafe extern "C" fn session_set_consumed_clock(object: *mut IUnknown, clock: *mut IMFPresentationClock) {
    let mut consumer: *mut IMFClockConsumer = core::ptr::null_mut::<IMFClockConsumer>();
    if (*(*object).lpVtbl)
        .QueryInterface
        .unwrap_unchecked()(
        object,
        &raw const IID_IMFClockConsumer,
        &raw mut consumer as *mut *mut core::ffi::c_void,
    ) >= 0
    {
        IMFClockConsumer_SetPresentationClock(consumer, clock);
        IMFClockConsumer_Release(consumer);
    }
}

unsafe extern "C" fn session_set_presentation_clock(mut session: *mut media_session) {
    let mut time_source: *mut IMFPresentationTimeSource = core::ptr::null_mut();
    let mut source: *mut media_source = core::ptr::null_mut();
    let mut sink: *mut media_sink = core::ptr::null_mut();
    let mut node: *mut topo_node = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if ((*session).presentation.flags & SESSION_FLAG_PRESENTATION_CLOCK_SET) == 0 {
        /* Attempt to get time source from the sinks. */
        sink = ((*session).presentation.sinks.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
        while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*session).presentation.sinks) {
            if (IMFMediaSink_QueryInterface(
                (*sink).__bindgen_anon_1.sink,
                core::ptr::addr_of!(IID_IMFPresentationTimeSource),
                core::ptr::addr_of_mut!(time_source) as *mut *mut core::ffi::c_void,
            )) >= 0
            {
                break;
            }
            sink = ((*sink).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
        }

        if !time_source.is_null() {
            hr = IMFPresentationClock_SetTimeSource((*session).clock, time_source);
            IMFPresentationTimeSource_Release(time_source);
        } else {
            hr = IMFPresentationClock_SetTimeSource((*session).clock, (*session).system_time_source);
        }

        if hr < 0 {
            WARN!("Failed to set time source, hr %#lx.\n", hr);
        }

        /* Set clock for all topology nodes. */
        source = ((*session).presentation.sources.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
        while core::ptr::addr_of_mut!((*source).entry) != core::ptr::addr_of_mut!((*session).presentation.sources) {
            session_set_consumed_clock((*source).__bindgen_anon_1.object, (*session).clock);
            source = ((*source).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
        }

        sink = ((*session).presentation.sinks.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
        while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*session).presentation.sinks) {
            if !(*sink).event_generator.is_null()
                && {
                    hr = IMFMediaEventGenerator_BeginGetEvent(
                        (*sink).event_generator,
                        core::ptr::addr_of_mut!((*session).events_callback),
                        (*sink).event_generator as *mut IUnknown,
                    );
                    hr < 0
                }
            {
                WARN!("Failed to subscribe to sink events, hr %#lx.\n", hr);
            }

            hr = IMFMediaSink_SetPresentationClock((*sink).__bindgen_anon_1.sink, (*session).clock);
            if hr < 0 {
                WARN!("Failed to set presentation clock for the sink, hr %#lx.\n", hr);
            }

            sink = ((*sink).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
        }

        node = ((*session).presentation.nodes.next as *mut u8)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
        while core::ptr::addr_of_mut!((*node).entry) != core::ptr::addr_of_mut!((*session).presentation.nodes) {
            if !((*node).type_ != MF_TOPOLOGY_TRANSFORM_NODE) {
                session_set_consumed_clock((*node).object.object, (*session).clock);
            }
            node = ((*node).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
        }

        (*session).presentation.flags |= SESSION_FLAG_PRESENTATION_CLOCK_SET;
    }
}

unsafe extern "C" fn session_set_rate(
    mut session: *mut media_session,
    mut thin: BOOL,
    mut rate: ::core::ffi::c_float,
) {
    let mut rate_control: *mut IMFRateControl = ::core::ptr::null_mut::<IMFRateControl>();
    let mut source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut clock_rate: ::core::ffi::c_float = 0.0f32;
    let mut param: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    hr = session_is_presentation_rate_supported(
        session,
        thin,
        rate,
        ::core::ptr::null_mut::<::core::ffi::c_float>(),
    );
    if hr >= 0 as HRESULT {
        hr = IMFRateControl_GetRate(
            (*session).clock_rate_control,
            ::core::ptr::null_mut::<BOOL>(),
            &raw mut clock_rate,
        );
    }
    if hr >= 0 as HRESULT
        && (rate != clock_rate || thin != (*session).presentation.thin_committed)
        && {
            hr = session_subscribe_sources(session);
            hr >= 0 as HRESULT
        }
    {
        source = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(media_source, entry) as isize),
        ) as *mut media_source;
        if &raw mut (*source).entry != &raw mut (*session).presentation.sources {
            hr = MFGetService(
                (*source).__bindgen_anon_1.object,
                &raw const MF_RATE_CONTROL_SERVICE,
                &raw const IID_IMFRateControl,
                &raw mut rate_control as *mut *mut ::core::ffi::c_void,
            );
            if hr >= 0 as HRESULT {
                hr = IMFRateControl_SetRate(rate_control, thin, rate);
                IMFRateControl_Release(rate_control);
                if hr >= 0 as HRESULT {
                    (*session).presentation.flags |= SESSION_FLAG_PENDING_RATE_CHANGE as ::core::ffi::c_uint;
                    (*session).presentation.rate = rate;
                    (*session).presentation.thin = thin;
                    return;
                }
            }
        }
    }
    param.vt = VT_R4 as VARTYPE;
    param.__bindgen_anon_1.fltVal = rate;
    session_command_complete_with_event(
        session,
        MESessionRateChanged as ::core::ffi::c_int as MediaEventType,
        hr,
        if hr >= 0 as HRESULT {
            &raw mut param
        } else {
            ::core::ptr::null_mut::<PROPVARIANT>()
        },
    );
}

unsafe extern "C" fn session_complete_rate_change(session: *mut media_session) {
    let mut param: PROPVARIANT = core::mem::zeroed::<PROPVARIANT>();
    let mut hr: HRESULT = 0;

    if (*session).presentation.flags & SESSION_FLAG_PENDING_RATE_CHANGE as ::core::ffi::c_uint == 0 {
        return;
    }

    (*session).presentation.flags &= !(SESSION_FLAG_PENDING_RATE_CHANGE as ::core::ffi::c_int) as ::core::ffi::c_uint;
    session_set_presentation_clock(session);

    (*session).presentation.thin_committed = (*session).presentation.thin;

    hr = IMFRateControl_SetRate(
        (*session).clock_rate_control,
        (FALSE) as BOOL,
        (*session).presentation.rate,
    );

    param.vt = VT_R4 as VARTYPE;
    param.__bindgen_anon_1.fltVal = (*session).presentation.rate;

    IMFMediaEventQueue_QueueEventParamVar(
        (*session).event_queue,
        MESessionRateChanged as ::core::ffi::c_int as MediaEventType,
        core::ptr::addr_of!(GUID_NULL),
        hr,
        if hr >= 0 {
            core::ptr::addr_of_mut!(param)
        } else {
            core::ptr::null_mut::<PROPVARIANT>()
        },
    );
    session_command_complete(session);
}

unsafe extern "C" fn session_get_media_source(
    session: *mut media_session,
    source: *mut IMFMediaSource,
) -> *mut media_source {
    let mut cur: *mut media_source = core::ptr::null_mut();
    cur = ((*session).presentation.sources.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    while core::ptr::addr_of_mut!((*cur).entry)
        != core::ptr::addr_of_mut!((*session).presentation.sources)
    {
        if source == (*cur).__bindgen_anon_1.source {
            return cur;
        }
        cur = ((*cur).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn session_release_media_source(source: *mut media_source) {
    if !(*source).__bindgen_anon_1.source.is_null() {
        IMFMediaSource_Release((*source).__bindgen_anon_1.source);
    }
    if !(*source).pd.is_null() {
        IMFPresentationDescriptor_Release((*source).pd);
    }
    free(source as *mut core::ffi::c_void);
}

unsafe extern "C" fn session_add_media_source(
    mut session: *mut media_session,
    mut node: *mut IMFTopologyNode,
    mut source: *mut IMFMediaSource,
) -> HRESULT {
    let mut media_source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut hr: HRESULT = 0;
    if !session_get_media_source(session, source).is_null() {
        return S_FALSE;
    }
    media_source = calloc(
        (1 as size_t) as _,
        (::core::mem::size_of::<media_source>() as size_t) as _,
    ) as *mut media_source;
    if media_source.is_null() {
        return E_OUTOFMEMORY;
    }
    (*media_source).__bindgen_anon_1.source = source;
    IMFMediaSource_AddRef((*media_source).__bindgen_anon_1.source);
    hr = IMFTopologyNode_GetUnknown(
        node,
        &raw const MF_TOPONODE_PRESENTATION_DESCRIPTOR,
        &raw const IID_IMFPresentationDescriptor,
        &raw mut (*media_source).pd as *mut LPVOID,
    );
    if hr >= 0 {
        list_add_tail(
            &raw mut (*session).presentation.sources,
            &raw mut (*media_source).entry,
        );
    } else {
        session_release_media_source(media_source);
    }
    return hr;
}

unsafe extern "C" fn session_raise_topology_set(
    mut session: *mut media_session,
    mut topology: *mut IMFTopology,
    mut status: HRESULT,
) {
    let mut param: PROPVARIANT = core::mem::zeroed();

    param.vt = (if !topology.is_null() {
        VT_UNKNOWN as ::core::ffi::c_int
    } else {
        VT_EMPTY as ::core::ffi::c_int
    }) as VARTYPE;
    param.__bindgen_anon_1.punkVal = topology as *mut IUnknown;

    IMFMediaEventQueue_QueueEventParamVar(
        (*session).event_queue,
        MESessionTopologySet as ::core::ffi::c_int as MediaEventType,
        &raw const GUID_NULL,
        status,
        &raw mut param,
    );
}

unsafe extern "C" fn session_get_object_rate_caps(object: *mut IUnknown) -> DWORD {
    let mut rate_support: *mut IMFRateSupport = core::ptr::null_mut();
    let mut caps: DWORD = 0;
    let mut rate: f32;

    if (MFGetService(object, core::ptr::addr_of!(MF_RATE_CONTROL_SERVICE), core::ptr::addr_of!(IID_IMFRateSupport), core::ptr::addr_of_mut!(rate_support) as *mut *mut core::ffi::c_void) >= 0)
    {
        rate = 0.0f32;
        if (IMFRateSupport_GetFastestRate(rate_support, MFRATE_FORWARD, (TRUE) as BOOL, core::ptr::addr_of_mut!(rate)) >= 0) && rate != 0.0f32
        {
            caps |= MFSESSIONCAP_RATE_FORWARD;
        }

        rate = 0.0f32;
        if (IMFRateSupport_GetFastestRate(rate_support, MFRATE_REVERSE, (TRUE) as BOOL, core::ptr::addr_of_mut!(rate)) >= 0) && rate != 0.0f32
        {
            caps |= MFSESSIONCAP_RATE_REVERSE;
        }

        IMFRateSupport_Release(rate_support);
    }

    return caps;
}

unsafe extern "C" fn session_add_media_sink(
    mut session: *mut media_session,
    mut node: *mut IMFTopologyNode,
    mut sink: *mut IMFMediaSink,
) -> HRESULT {
    let mut media_sink: *mut media_sink = core::ptr::null_mut::<media_sink>();
    let mut disable_preroll: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    let mut flags: DWORD = 0;
    media_sink = ((*session).presentation.sinks.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(media_sink, entry) as isize),
    ) as *mut media_sink;
    while &raw mut (*media_sink).entry != &raw mut (*session).presentation.sinks {
        if sink == (*media_sink).__bindgen_anon_1.sink {
            return S_FALSE;
        }
        media_sink = ((*media_sink).entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(media_sink, entry) as isize),
        ) as *mut media_sink;
    }
    media_sink = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<media_sink>() as size_t) as _,
    ) as *mut media_sink;
    if media_sink.is_null() {
        return E_OUTOFMEMORY;
    }
    (*media_sink).__bindgen_anon_1.sink = sink;
    IMFMediaSink_AddRef((*media_sink).__bindgen_anon_1.sink);

    IMFMediaSink_QueryInterface(
        (*media_sink).__bindgen_anon_1.sink,
        &raw const MF_TOPONODE_DISABLE_PREROLL,
        &raw mut (*media_sink).event_generator as *mut *mut ::core::ffi::c_void,
    );

    IMFTopologyNode_GetUINT32(
        node,
        &raw const MF_TOPONODE_DISABLE_PREROLL,
        &raw mut disable_preroll,
    );
    if (IMFMediaSink_GetCharacteristics(sink, &raw mut flags)) >= 0 as HRESULT
        && (flags & (MEDIASINK_CAN_PREROLL as DWORD)) != 0
        && disable_preroll == 0
    {
        if (IMFMediaSink_QueryInterface(
            (*media_sink).__bindgen_anon_1.sink,
            &raw const IID_IMFMediaSinkPreroll,
            &raw mut (*media_sink).preroll as *mut *mut ::core::ffi::c_void,
        )) >= 0 as HRESULT
        {
            (*session).presentation.flags |= SESSION_FLAG_NEEDS_PREROLL as ::core::ffi::c_uint;
        }
    }

    list_add_tail(
        &raw mut (*session).presentation.sinks,
        &raw mut (*media_sink).entry,
    );

    return S_OK;
}

unsafe extern "C" fn transform_node_get_stream_id(node: *mut topo_node, output: BOOL, index: UINT32) -> DWORD {
    let map: *mut DWORD = if output != 0 {
        (*node).u.transform.output_map
    } else {
        (*node).u.transform.input_map
    };
    return if !map.is_null() {
        *map.add(index as usize)
    } else {
        index as DWORD
    };
}

unsafe extern "C" fn session_set_transform_stream_info(node: *mut topo_node) -> HRESULT {
    let mut input_map: *mut DWORD = core::ptr::null_mut();
    let mut output_map: *mut DWORD = core::ptr::null_mut();
    let mut major: GUID = core::mem::zeroed();
    let mut subtype: GUID = core::mem::zeroed();
    let mut i: DWORD;
    let mut input_count: DWORD = core::mem::zeroed();
    let mut output_count: DWORD = core::mem::zeroed();
    let mut streams: *mut transform_stream;
    let mut bytes_per_second: UINT32 = core::mem::zeroed();
    let mut value: UINT32 = core::mem::zeroed();
    let mut block_alignment: u32 = core::mem::zeroed();
    let mut attributes: *mut IMFAttributes;
    let mut media_type: *mut IMFMediaType = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = IMFTransform_GetStreamCount(
        (*node).object.transform,
        core::ptr::addr_of_mut!(input_count),
        core::ptr::addr_of_mut!(output_count),
    );
    if hr >= 0
        && (input_count > 1 || output_count > 1)
    {
        input_map = calloc((input_count as usize) as _, (core::mem::size_of::<DWORD>()) as _) as *mut DWORD;
        output_map = calloc((output_count as usize) as _, (core::mem::size_of::<DWORD>()) as _) as *mut DWORD;
        if IMFTransform_GetStreamIDs(
            (*node).object.transform,
            input_count,
            input_map,
            output_count,
            output_map,
        ) < 0
        {
            /* Assume sequential identifiers. */
            free(input_map as *mut core::ffi::c_void);
            free(output_map as *mut core::ffi::c_void);
            output_map = core::ptr::null_mut();
            input_map = core::ptr::null_mut();
        }
    }

    if hr >= 0 {
        (*node).u.transform.input_map = input_map;
        (*node).u.transform.output_map = output_map;

        streams = calloc((input_count as usize) as _, (core::mem::size_of::<transform_stream>()) as _) as *mut transform_stream;
        i = 0;
        while i < input_count {
            list_init(core::ptr::addr_of_mut!((*streams.offset(i as isize)).samples));
            i = i.wrapping_add(1);
        }
        (*node).u.transform.inputs = streams;
        (*node).u.transform.input_count = input_count;

        attributes = topology_node_transform_async_get_attributes((*node).node);
        if !attributes.is_null() {
            IMFAttributes_SetUINT32(
                attributes,
                &MF_TRANSFORM_ASYNC_UNLOCK,
                TRUE,
            );
            IMFAttributes_Release(attributes);

            (*node).u.transform.async_ = (TRUE as i32);
            hr = IMFTransform_QueryInterface(
                (*node).object.transform,
                &IID_IMFMediaEventGenerator,
                core::ptr::addr_of_mut!((*node).u.transform.event_source) as *mut *mut core::ffi::c_void,
            );
            if hr < 0 {
                WARN!("Failed to get event source, hr %#lx.\n", hr);
            }
        }

        streams = calloc((output_count as usize) as _, (core::mem::size_of::<transform_stream>()) as _) as *mut transform_stream;
        i = 0;
        while i < output_count {
            list_init(core::ptr::addr_of_mut!((*streams.offset(i as isize)).samples));

            if IMFTransform_GetOutputCurrentType(
                (*node).object.transform,
                transform_node_get_stream_id(node, (TRUE) as BOOL, i),
                core::ptr::addr_of_mut!(media_type),
            ) >= 0
            {
                if IMFMediaType_GetMajorType(media_type, core::ptr::addr_of_mut!(major)) >= 0
                    && IsEqualGUID!(&major, &MFMediaType_Audio)
                    && IMFMediaType_GetUINT32(
                        media_type,
                        &MF_MT_AUDIO_BLOCK_ALIGNMENT,
                        core::ptr::addr_of_mut!(block_alignment),
                    ) >= 0
                {
                    (*streams.offset(i as isize)).block_alignment = block_alignment;
                    (*streams.offset(i as isize)).min_buffer_size = block_alignment;
                    if IMFMediaType_GetUINT32(
                        media_type,
                        &MF_MT_AUDIO_AVG_BYTES_PER_SECOND,
                        core::ptr::addr_of_mut!(bytes_per_second),
                    ) >= 0
                    {
                        (*streams.offset(i as isize)).bytes_per_second = bytes_per_second;
                        (*streams.offset(i as isize)).min_buffer_size = max!(
                            (*streams.offset(i as isize)).min_buffer_size,
                            bytes_per_second,
                        );
                    }
                    if IMFMediaType_GetGUID(
                        media_type,
                        &MF_MT_SUBTYPE,
                        core::ptr::addr_of_mut!(subtype),
                    ) >= 0
                        && (IsEqualGUID!(&subtype, &MFAudioFormat_Float) || IsEqualGUID!(&subtype, &MFAudioFormat_PCM))
                    {
                        (*streams.offset(i as isize)).raw_audio = (TRUE as i32);
                    }
                }
                if IMFMediaType_GetUINT32(
                    media_type,
                    &MF_MT_ALL_SAMPLES_INDEPENDENT,
                    core::ptr::addr_of_mut!(value),
                ) >= 0
                    && value != 0
                {
                    (*streams.offset(i as isize)).samples_independent = (TRUE as i32);
                }
                IMFMediaType_Release(media_type);
            }
            i = i.wrapping_add(1);
        }
        (*node).u.transform.outputs = streams;
        (*node).u.transform.output_count = output_count;
    }

    return hr;
}

unsafe extern "C" fn session_get_stream_sink_type(sink: *mut IMFStreamSink, media_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut hr: HRESULT;

    if {
        hr = IMFStreamSink_GetMediaTypeHandler(sink, core::ptr::addr_of_mut!(handler));
        hr
    } >= 0 {
        hr = IMFMediaTypeHandler_GetCurrentMediaType(handler, media_type);
        IMFMediaTypeHandler_Release(handler);
    }

    return hr;
}

unsafe extern "system" fn node_sample_allocator_cb_QueryInterface(
    iface: *mut IMFVideoSampleAllocatorNotify,
    riid: *const GUID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFVideoSampleAllocatorNotify))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFVideoSampleAllocatorNotify_AddRef(iface);
        return S_OK;
    }

    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn node_sample_allocator_cb_AddRef(
    iface: *mut IMFVideoSampleAllocatorNotify,
) -> ULONG {
    2
}
unsafe extern "system" fn node_sample_allocator_cb_Release(
    iface: *mut IMFVideoSampleAllocatorNotify,
) -> ULONG {
    1
}

unsafe extern "system" fn node_sample_allocator_cb_NotifyRelease(iface: *mut IMFVideoSampleAllocatorNotify) -> HRESULT {
    let topo_node: *mut topo_node = impl_node_from_IMFVideoSampleAllocatorNotify(iface);
    MFPutWorkItem(MFASYNC_CALLBACK_QUEUE_STANDARD as DWORD, core::ptr::addr_of_mut!((*(*topo_node).session).sa_ready_callback), iface as *mut IUnknown);
    return S_OK;
}

unsafe extern "C" fn session_append_node(session: *mut media_session, node: *mut IMFTopologyNode) -> HRESULT {
    let mut topo_node: *mut topo_node = core::ptr::null_mut();
    let mut media_sink: *mut IMFMediaSink = core::ptr::null_mut();
    let mut media_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;
    let mut value: UINT32 = 0;

    topo_node = calloc(1, (core::mem::size_of::<topo_node>()) as _) as *mut topo_node;
    if topo_node.is_null() {
        return E_OUTOFMEMORY;
    }

    IMFTopologyNode_GetTopoNodeID(node, core::ptr::addr_of_mut!((*topo_node).node_id));
    (*topo_node).node = node;
    IMFTopologyNode_AddRef((*topo_node).node);
    (*topo_node).session = session;
    if (IMFTopologyNode_GetUINT32(node, core::ptr::addr_of!(MF_TOPONODE_MARKIN_HERE), core::ptr::addr_of_mut!(value)) >= 0) && value != 0 {
        (*topo_node).flags |= TOPO_NODE_MARKIN_HERE;
    }

    (*topo_node).type_ = topology_node_get_type(node);
    'sw: {
        match (*topo_node).type_ {
            MF_TOPOLOGY_OUTPUT_NODE => {
                (*topo_node).u.sink.notify_cb.lpVtbl = core::ptr::addr_of!(node_sample_allocator_cb_vtbl);

                hr = topology_node_get_object(
                    node,
                    core::ptr::addr_of!(IID_IMFStreamSink),
                    core::ptr::addr_of_mut!((*topo_node).object.object) as *mut *mut core::ffi::c_void,
                );
                if hr < 0 {
                    WARN!("Failed to get stream sink interface, hr %#lx.\n", hr);
                    break 'sw;
                }

                hr = IMFStreamSink_GetMediaSink((*topo_node).object.sink_stream, core::ptr::addr_of_mut!(media_sink));
                if hr < 0 {
                    break 'sw;
                }

                hr = session_add_media_sink(session, node, media_sink);
                if hr >= 0 {
                    let mut device_manager: *mut IUnknown = core::ptr::null_mut();

                    if session_get_stream_sink_type((*topo_node).object.sink_stream, core::ptr::addr_of_mut!(media_type)) >= 0 {
                        if MFGetService(
                            (*topo_node).object.object,
                            core::ptr::addr_of!(MR_VIDEO_ACCELERATION_SERVICE),
                            core::ptr::addr_of!(IID_IMFVideoSampleAllocator),
                            core::ptr::addr_of_mut!((*topo_node).u.sink.allocator) as *mut *mut core::ffi::c_void,
                        ) >= 0 {
                            hr = IMFVideoSampleAllocator_InitializeSampleAllocator(
                                (*topo_node).u.sink.allocator,
                                2,
                                media_type,
                            );
                            if hr < 0 {
                                WARN!("Failed to initialize sample allocator for the stream, hr %#lx.\n", hr);
                            }
                            IMFVideoSampleAllocator_QueryInterface(
                                (*topo_node).u.sink.allocator,
                                core::ptr::addr_of!(IID_IMFVideoSampleAllocatorCallback),
                                core::ptr::addr_of_mut!((*topo_node).u.sink.allocator_cb) as *mut *mut core::ffi::c_void,
                            );
                            IMFVideoSampleAllocatorCallback_SetCallback(
                                (*topo_node).u.sink.allocator_cb,
                                core::ptr::addr_of_mut!((*topo_node).u.sink.notify_cb),
                            );
                        }
                        IMFMediaType_Release(media_type);
                    }

                    if stream_sink_get_device_manager((*topo_node).object.sink_stream, core::ptr::addr_of_mut!(device_manager)) >= 0 {
                        let mut upstream: *mut IMFTopologyNode = core::ptr::null_mut();
                        let mut output: DWORD = 0;

                        if IMFTopologyNode_GetInput(
                            (*topo_node).node,
                            0,
                            core::ptr::addr_of_mut!(upstream),
                            core::ptr::addr_of_mut!(output),
                        ) >= 0 {
                            if topology_node_is_d3d_aware(upstream) != 0 {
                                topology_node_set_device_manager(upstream, device_manager);
                            }
                            IMFTopologyNode_Release(upstream);
                        }

                        IUnknown_Release(device_manager);
                    }
                }
                IMFMediaSink_Release(media_sink);
                break 'sw;
            }
            MF_TOPOLOGY_SOURCESTREAM_NODE => {
                if IMFTopologyNode_GetUnknown(
                    node,
                    core::ptr::addr_of!(MF_TOPONODE_SOURCE),
                    core::ptr::addr_of!(IID_IMFMediaSource),
                    core::ptr::addr_of_mut!((*topo_node).u.source.source) as *mut LPVOID,
                ) < 0 {
                    WARN!("Missing MF_TOPONODE_SOURCE, hr %#lx.\n", hr);
                    break 'sw;
                }

                hr = session_add_media_source(session, node, (*topo_node).u.source.source);
                if hr < 0 {
                    break 'sw;
                }

                hr = IMFTopologyNode_GetUnknown(
                    node,
                    core::ptr::addr_of!(MF_TOPONODE_STREAM_DESCRIPTOR),
                    core::ptr::addr_of!(IID_IMFStreamDescriptor),
                    core::ptr::addr_of_mut!(sd) as *mut LPVOID,
                );
                if hr < 0 {
                    WARN!("Missing MF_TOPONODE_STREAM_DESCRIPTOR, hr %#lx.\n", hr);
                    break 'sw;
                }

                hr = IMFStreamDescriptor_GetStreamIdentifier(sd, core::ptr::addr_of_mut!((*topo_node).u.source.stream_id));
                IMFStreamDescriptor_Release(sd);

                break 'sw;
            }
            MF_TOPOLOGY_TRANSFORM_NODE => {
                hr = topology_node_get_object(
                    node,
                    core::ptr::addr_of!(IID_IMFTransform),
                    core::ptr::addr_of_mut!((*topo_node).object.transform) as *mut *mut core::ffi::c_void,
                );
                if hr >= 0 {
                    hr = session_set_transform_stream_info(topo_node);
                } else {
                    WARN!("Failed to get IMFTransform for MFT node, hr %#lx.\n", hr);
                }

                break 'sw;
            }
            MF_TOPOLOGY_TEE_NODE => {
                FIXME!("Unsupported node type %d.\n", (*topo_node).type_);

                break 'sw;
            }
            _ => {}
        }
    }

    if hr >= 0 {
        list_add_tail(
            core::ptr::addr_of_mut!((*session).presentation.nodes),
            core::ptr::addr_of_mut!((*topo_node).entry),
        );
    } else {
        release_topo_node(topo_node);
    }

    return hr;
}

unsafe extern "C" fn session_collect_nodes(session: *mut media_session) -> HRESULT {
    let mut topology: *mut IMFTopology = (*session).presentation.current_topology;
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut();
    let mut i: WORD = 0;
    let mut count: WORD = 0;
    let mut hr: HRESULT = 0;

    if list_empty(core::ptr::addr_of_mut!((*session).presentation.nodes)) == 0 {
        return S_OK;
    }

    hr = IMFTopology_GetNodeCount(topology, core::ptr::addr_of_mut!(count));
    if hr < 0 {
        return hr;
    }

    i = 0;
    while (i as i32) < (count as i32) {
        hr = IMFTopology_GetNode(topology, i, core::ptr::addr_of_mut!(node));
        if hr < 0 {
            WARN!("Failed to get node %u.\n", i as i32);
            break;
        } else {
            hr = session_append_node(session, node);
            IMFTopologyNode_Release(node);
            if hr < 0 {
                WARN!("Failed to add node %u.\n", i as i32);
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }

    return hr;
}

unsafe extern "C" fn session_set_current_topology(
    session: *mut media_session,
    topology: *mut IMFTopology,
) -> HRESULT {
    let mut source: *mut media_source = core::ptr::null_mut::<media_source>();
    let mut caps: DWORD;
    let mut object_flags: DWORD;
    let mut sink: *mut media_sink = core::ptr::null_mut::<media_sink>();
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut::<IMFMediaEvent>();
    let mut hr: HRESULT;

    hr = IMFTopology_CloneFrom((*session).presentation.current_topology, topology);
    if hr < 0 {
        WARN!("Failed to clone topology, hr %#lx.\n", hr);
        return hr;
    }

    session_collect_nodes(session);

    node = ((*session).presentation.nodes.next as *mut u8)
        .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        if (*node).type_ == MF_TOPOLOGY_TRANSFORM_NODE {
            hr = IMFTransform_ProcessMessage(
                (*node).object.transform,
                MFT_MESSAGE_NOTIFY_BEGIN_STREAMING,
                0,
            );
            if hr < 0 {
                return hr;
            }
        }
        node = ((*node).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    }

    /* FIXME: attributes are all zero for now */
    if MFCreateMediaEvent(
        MESessionNotifyPresentationTime,
        core::ptr::addr_of!(GUID_NULL),
        S_OK,
        core::ptr::null::<PROPVARIANT>(),
        core::ptr::addr_of_mut!(event),
    ) >= 0
    {
        IMFMediaEvent_SetUINT64(
            event,
            core::ptr::addr_of!(MF_EVENT_START_PRESENTATION_TIME),
            0,
        );
        IMFMediaEvent_SetUINT64(
            event,
            core::ptr::addr_of!(MF_EVENT_PRESENTATION_TIME_OFFSET),
            0,
        );
        IMFMediaEvent_SetUINT64(
            event,
            core::ptr::addr_of!(MF_EVENT_START_PRESENTATION_TIME_AT_OUTPUT),
            0,
        );

        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
        IMFMediaEvent_Release(event);
    }

    /* Update session caps. */
    caps = MFSESSIONCAP_START
        | MFSESSIONCAP_SEEK
        | MFSESSIONCAP_RATE_FORWARD
        | MFSESSIONCAP_RATE_REVERSE
        | MFSESSIONCAP_DOES_NOT_USE_NETWORK;

    source = ((*session).presentation.sources.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    while core::ptr::addr_of_mut!((*source).entry)
        != core::ptr::addr_of_mut!((*session).presentation.sources)
    {
        if caps == 0 {
            break;
        }

        object_flags = 0;
        if IMFMediaSource_GetCharacteristics((*source).__bindgen_anon_1.source, core::ptr::addr_of_mut!(object_flags))
            >= 0
        {
            if (object_flags & MFMEDIASOURCE_DOES_NOT_USE_NETWORK) == 0 {
                caps &= !MFSESSIONCAP_DOES_NOT_USE_NETWORK;
            }
            if (object_flags & MFMEDIASOURCE_CAN_SEEK) == 0 {
                caps &= !MFSESSIONCAP_SEEK;
            }
        }

        /* Mask unsupported rate caps. */

        caps &= session_get_object_rate_caps((*source).__bindgen_anon_1.object)
            | !(MFSESSIONCAP_RATE_FORWARD | MFSESSIONCAP_RATE_REVERSE);

        source = ((*source).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_source, entry) as isize)) as *mut media_source;
    }

    sink = ((*session).presentation.sinks.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
    while core::ptr::addr_of_mut!((*sink).entry)
        != core::ptr::addr_of_mut!((*session).presentation.sinks)
    {
        if caps == 0 {
            break;
        }

        object_flags = 0;
        if IMFMediaSink_GetCharacteristics((*sink).__bindgen_anon_1.sink, core::ptr::addr_of_mut!(object_flags))
            >= 0
        {
            if (object_flags & MEDIASINK_RATELESS) == 0 {
                caps &= session_get_object_rate_caps((*sink).__bindgen_anon_1.object)
                    | !(MFSESSIONCAP_RATE_FORWARD | MFSESSIONCAP_RATE_REVERSE);
            }
        }

        sink = ((*sink).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
    }

    session_set_caps(session, caps);

    return S_OK;
}

unsafe extern "C" fn session_set_topology(
    mut session: *mut media_session,
    mut flags: DWORD,
    mut topology: *mut IMFTopology,
) {
    let mut resolved_topology: *mut IMFTopology = core::ptr::null_mut::<IMFTopology>();
    let mut hr: HRESULT = S_OK;

    /* Resolve unless claimed to be full. */
    if (flags & MFSESSION_SETTOPOLOGY_CLEAR_CURRENT as ::core::ffi::c_int as DWORD) == 0 && !topology.is_null() {
        if (flags & MFSESSION_SETTOPOLOGY_NORESOLUTION as ::core::ffi::c_int as DWORD) == 0 {
            hr = session_bind_output_nodes(topology);

            if hr >= 0 as HRESULT {
                hr = IMFTopoLoader_Load(
                    (*session).topo_loader,
                    topology,
                    core::ptr::addr_of_mut!(resolved_topology),
                    core::ptr::null_mut::<IMFTopology>(),
                );
            }
            if hr >= 0 as HRESULT {
                hr = session_init_media_types(resolved_topology);
            }

            if hr >= 0 as HRESULT {
                topology = resolved_topology;
            } else {
                topology = core::ptr::null_mut::<IMFTopology>();
            }
        }
    }

    if (flags & MFSESSION_SETTOPOLOGY_CLEAR_CURRENT as ::core::ffi::c_int as DWORD) != 0 {
        if (!topology.is_null() && topology == (*session).presentation.current_topology)
            || topology.is_null()
        {
            /* FIXME: stop current topology, queue next one. */
            session_clear_presentation(session);
        } else {
            hr = S_FALSE;
        }

        topology = core::ptr::null_mut::<IMFTopology>();
    } else if !topology.is_null()
        && (flags & MFSESSION_SETTOPOLOGY_IMMEDIATE as ::core::ffi::c_int as DWORD) != 0
    {
        session_clear_queued_topologies(session);
        session_clear_presentation(session);
    }

    session_raise_topology_set(session, topology, hr);

    /* With no current topology set it right away, otherwise queue. */
    if !topology.is_null() {
        let mut queued_topology: *mut queued_topology = core::ptr::null_mut::<queued_topology>();

        queued_topology = calloc(
            (1 as size_t) as _,
            (core::mem::size_of::<queued_topology>() as size_t) as _,
        ) as *mut queued_topology;
        if !queued_topology.is_null() {
            (*queued_topology).topology = topology;
            IMFTopology_AddRef((*queued_topology).topology);

            list_add_tail(
                core::ptr::addr_of_mut!((*session).topologies),
                core::ptr::addr_of_mut!((*queued_topology).entry),
            );
        }

        if (*session).presentation.topo_status as ::core::ffi::c_uint
            == MF_TOPOSTATUS_INVALID as ::core::ffi::c_uint
        {
            hr = session_set_current_topology(session, topology);
            session_set_topo_status(session, hr, MF_TOPOSTATUS_READY);
            if !(*session).quality_manager.is_null() {
                IMFQualityManager_NotifyTopology((*session).quality_manager, topology);
            }
        }
    }

    if !resolved_topology.is_null() {
        IMFTopology_Release(resolved_topology);
    }
}
unsafe extern "system" fn mfsession_QueryInterface(iface: *mut IMFMediaSession, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);

    *out = core::ptr::null_mut();

    if ((IsEqualIID!(riid, &raw const IID_IMFMediaSession)) != 0)
        || ((IsEqualIID!(riid, &raw const IID_IMFMediaEventGenerator)) != 0)
        || ((IsEqualIID!(riid, &raw const IID_IUnknown)) != 0)
    {
        *out = &raw mut (*session).IMFMediaSession_iface as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, &raw const IID_IMFGetService)) != 0) {
        *out = &raw mut (*session).IMFGetService_iface as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, &raw const IID_IMFRateSupport)) != 0) {
        *out = &raw mut (*session).IMFRateSupport_iface as *mut core::ffi::c_void;
    } else if ((IsEqualIID!(riid, &raw const IID_IMFRateControl)) != 0) {
        *out = &raw mut (*session).IMFRateControl_iface as *mut core::ffi::c_void;
    }

    if !(*out).is_null() {
        IMFMediaSession_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid as *const GUID));
    return E_NOINTERFACE;
}

unsafe extern "system" fn mfsession_AddRef(iface: *mut IMFMediaSession) -> ULONG {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    let refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*session).refcount) as *mut LONG) as ULONG;

    return refcount;
}

unsafe extern "system" fn mfsession_Release(iface: *mut IMFMediaSession) -> ULONG {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*session).refcount) as *mut core::ffi::c_long,
    ) as ULONG;

    TRACE!("%p, refcount %lu.\n", iface, refcount);

    if refcount == 0 {
        if (session_is_shut_down(session)) >= 0 {
            IMFMediaSession_Shutdown(iface);
        }
        session_clear_command_list(session);
        if !(*session).presentation.current_topology.is_null() {
            IMFTopology_Release((*session).presentation.current_topology);
        }
        if !(*session).event_queue.is_null() {
            IMFMediaEventQueue_Release((*session).event_queue);
        }
        if !(*session).clock.is_null() {
            IMFPresentationClock_Release((*session).clock);
        }
        if !(*session).system_time_source.is_null() {
            IMFPresentationTimeSource_Release((*session).system_time_source);
        }
        if !(*session).clock_rate_control.is_null() {
            IMFRateControl_Release((*session).clock_rate_control);
        }
        if !(*session).topo_loader.is_null() {
            IMFTopoLoader_Release((*session).topo_loader);
        }
        if !(*session).quality_manager.is_null() {
            IMFQualityManager_Release((*session).quality_manager);
        }
        DeleteCriticalSection(core::ptr::addr_of_mut!((*session).cs));
        free(session as *mut core::ffi::c_void);
    }

    return refcount;
}

unsafe extern "system" fn mfsession_GetEvent(iface: *mut IMFMediaSession, flags: DWORD, event: *mut *mut IMFMediaEvent) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    return IMFMediaEventQueue_GetEvent((*session).event_queue, flags, event);
}

unsafe extern "system" fn mfsession_BeginGetEvent(
    mut iface: *mut IMFMediaSession,
    mut callback: *mut IMFAsyncCallback,
    mut state: *mut IUnknown,
) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/session.c\0".as_ptr()
                as *const i8,
            b"mfsession_BeginGetEvent\0".as_ptr() as *const i8,
            2307 as i32,
            b"%p, %p, %p.\n\0".as_ptr() as *const i8,
            iface,
            callback,
            state,
        );
    }
    return IMFMediaEventQueue_BeginGetEvent((*session).event_queue, callback, state);
}

unsafe extern "system" fn mfsession_EndGetEvent(
    iface: *mut IMFMediaSession,
    result: *mut IMFAsyncResult,
    event: *mut *mut IMFMediaEvent,
) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    if ((*__wine_dbch___default).flags as core::ffi::c_int
        & ((1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int))
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/session.c\0".as_ptr()
                as *const core::ffi::c_char,
            b"mfsession_EndGetEvent\0".as_ptr() as *const core::ffi::c_char,
            2316 as core::ffi::c_int,
            b"%p, %p, %p.\n\0".as_ptr() as *const core::ffi::c_char,
            iface,
            result,
            event,
        );
    }
    return IMFMediaEventQueue_EndGetEvent((*session).event_queue, result, event);
}

unsafe extern "system" fn mfsession_QueueEvent(iface: *mut IMFMediaSession, event_type: MediaEventType, ext_type: REFGUID, hr: HRESULT, value: *const PROPVARIANT) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);

    return IMFMediaEventQueue_QueueEventParamVar((*session).event_queue, event_type, ext_type, hr, value);
}

unsafe extern "C" fn session_check_stream_descriptor(pd: *mut IMFPresentationDescriptor, sd: *mut IMFStreamDescriptor) -> HRESULT {
    let mut selected_sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut i: DWORD;
    let mut count: DWORD = core::mem::zeroed();
    let mut selected: BOOL = core::mem::zeroed();
    let mut hr: HRESULT;

    if {
        hr = IMFPresentationDescriptor_GetStreamDescriptorCount(pd, core::ptr::addr_of_mut!(count));
        hr
    } < 0
    {
        WARN!("Failed to get stream descriptor count, hr %#lx.\n", hr);
        return MF_E_TOPO_STREAM_DESCRIPTOR_NOT_SELECTED;
    }

    i = 0;
    while i < count {
        if {
            hr = IMFPresentationDescriptor_GetStreamDescriptorByIndex(
                pd,
                i,
                core::ptr::addr_of_mut!(selected),
                core::ptr::addr_of_mut!(selected_sd),
            );
            hr
        } < 0
        {
            WARN!("Failed to get stream descriptor %lu, hr %#lx.\n", i, hr);
            return MF_E_TOPO_STREAM_DESCRIPTOR_NOT_SELECTED;
        }
        IMFStreamDescriptor_Release(selected_sd);

        if selected_sd == sd {
            if selected != 0 {
                return S_OK;
            }

            WARN!("Presentation descriptor %p stream %p is not selected.\n", pd, sd);
            return MF_E_TOPO_STREAM_DESCRIPTOR_NOT_SELECTED;
        }
        i += 1;
    }

    WARN!("Failed to find stream descriptor %lu, hr %#lx.\n", i, hr);
    return MF_E_TOPO_STREAM_DESCRIPTOR_NOT_SELECTED;
}

unsafe extern "C" fn session_check_topology(mut topology: *mut IMFTopology) -> HRESULT {
    let mut node: *mut IMFTopologyNode = core::ptr::null_mut::<IMFTopologyNode>();
    let mut node_count: WORD = 0;
    let mut i: WORD = 0;
    let mut hr: HRESULT = 0;

    if topology.is_null() {
        return S_OK;
    }

    if (IMFTopology_GetNodeCount(topology, core::ptr::addr_of_mut!(node_count)) < 0 as HRESULT)
        || node_count as ::core::ffi::c_int == 0 as ::core::ffi::c_int
    {
        return E_INVALIDARG;
    }

    i = 0 as WORD;
    while (i as ::core::ffi::c_int) < (node_count as ::core::ffi::c_int) {
        hr = IMFTopology_GetNode(topology, i, core::ptr::addr_of_mut!(node));
        if hr < 0 as HRESULT {
            break;
        }

        match topology_node_get_type(node) as ::core::ffi::c_uint {
            MF_TOPOLOGY_SOURCESTREAM_NODE => {
                let mut pd: *mut IMFPresentationDescriptor =
                    core::ptr::null_mut::<IMFPresentationDescriptor>();
                let mut sd: *mut IMFStreamDescriptor =
                    core::ptr::null_mut::<IMFStreamDescriptor>();
                let mut source: *mut IMFMediaSource = core::ptr::null_mut::<IMFMediaSource>();

                if ({ hr = IMFTopologyNode_GetUnknown(
                    node,
                    &MF_TOPONODE_SOURCE,
                    core::ptr::addr_of!(IID_IMFMediaSource),
                    core::ptr::addr_of_mut!(source) as *mut LPVOID,
                ); hr }) < 0 as HRESULT
                {
                    WARN!("Missing MF_TOPONODE_SOURCE, hr %#lx.\n", hr);
                    IMFTopologyNode_Release(node);
                    return MF_E_TOPO_MISSING_SOURCE;
                }
                IMFMediaSource_Release(source);

                if ({ hr = IMFTopologyNode_GetUnknown(
                    node,
                    &MF_TOPONODE_PRESENTATION_DESCRIPTOR,
                    core::ptr::addr_of!(IID_IMFPresentationDescriptor),
                    core::ptr::addr_of_mut!(pd) as *mut LPVOID,
                ); hr }) < 0 as HRESULT
                {
                    WARN!("Missing MF_TOPONODE_PRESENTATION_DESCRIPTOR, hr %#lx.\n", hr);
                    IMFTopologyNode_Release(node);
                    return MF_E_TOPO_MISSING_PRESENTATION_DESCRIPTOR;
                }

                if ({ hr = IMFTopologyNode_GetUnknown(
                    node,
                    &MF_TOPONODE_STREAM_DESCRIPTOR,
                    core::ptr::addr_of!(IID_IMFStreamDescriptor),
                    core::ptr::addr_of_mut!(sd) as *mut LPVOID,
                ); hr }) < 0 as HRESULT
                {
                    WARN!("Missing MF_TOPONODE_STREAM_DESCRIPTOR, hr %#lx.\n", hr);
                    IMFPresentationDescriptor_Release(pd);
                    IMFTopologyNode_Release(node);
                    return MF_E_TOPO_MISSING_STREAM_DESCRIPTOR;
                }

                hr = session_check_stream_descriptor(pd, sd);
                IMFPresentationDescriptor_Release(pd);
                IMFStreamDescriptor_Release(sd);
                if hr < 0 as HRESULT {
                    IMFTopologyNode_Release(node);
                    return hr;
                }

                break;
            }

            _ => {}
        }

        IMFTopologyNode_Release(node);
        i = i.wrapping_add(1);
    }

    return hr;
}

unsafe extern "system" fn mfsession_SetTopology(iface: *mut IMFMediaSession, flags: DWORD, topology: *mut IMFTopology) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut op: *mut session_op = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("%p, %#lx, %p.\n", iface, flags, topology);

    hr = session_check_topology(topology);
    if hr < 0 {
        return hr;
    }

    hr = create_session_op(SESSION_CMD_SET_TOPOLOGY, core::ptr::addr_of_mut!(op));
    if hr < 0 {
        return hr;
    }

    (*op).__bindgen_anon_1.set_topology.flags = flags;
    (*op).__bindgen_anon_1.set_topology.topology = topology;
    if !(*op).__bindgen_anon_1.set_topology.topology.is_null() {
        IMFTopology_AddRef((*op).__bindgen_anon_1.set_topology.topology);
    }

    hr = session_submit_command(session, op);
    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));

    return hr;
}
unsafe extern "system" fn mfsession_ClearTopologies(iface: *mut IMFMediaSession) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);

    return session_submit_simple_command(session, SESSION_CMD_CLEAR_TOPOLOGIES);
}
unsafe extern "system" fn mfsession_Start(iface: *mut IMFMediaSession, format: *const GUID, start_position: *const PROPVARIANT) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut op: *mut session_op = core::ptr::null_mut::<session_op>();
    let mut hr: HRESULT = 0;

    if start_position.is_null() {
        return E_POINTER;
    }

    hr = create_session_op(SESSION_CMD_START, &mut op as *mut *mut session_op);
    if hr < 0 {
        return hr;
    }

    (*op).__bindgen_anon_1.start.time_format = if !format.is_null() {
        *format
    } else {
        GUID_NULL
    };
    hr = PropVariantCopy(
        core::ptr::addr_of_mut!((*op).__bindgen_anon_1.start.start_position),
        start_position,
    );

    if hr >= 0 {
        hr = session_submit_command(session, op);
    }

    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));
    return hr;
}

unsafe extern "system" fn mfsession_Pause(iface: *mut IMFMediaSession) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);

    TRACE!("%p.\n", iface);

    return session_submit_simple_command(session, SESSION_CMD_PAUSE);
}
unsafe extern "system" fn mfsession_Stop(iface: *mut IMFMediaSession) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);

    return session_submit_simple_command(session, SESSION_CMD_STOP);
}

unsafe extern "system" fn mfsession_Close(iface: *mut IMFMediaSession) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    TRACE!("%p.\n", iface);
    return session_submit_simple_command(session, SESSION_CMD_CLOSE);
}

unsafe extern "system" fn mfsession_Shutdown(iface: *mut IMFMediaSession) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut hr: HRESULT = S_OK;

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));
    hr = session_is_shut_down(session);
    if hr >= 0 {
        (*session).state = SESSION_STATE_SHUT_DOWN;
        IMFMediaEventQueue_Shutdown((*session).event_queue);
        if !(*session).quality_manager.is_null() {
            IMFQualityManager_Shutdown((*session).quality_manager);
        }
        MFShutdownObject((*session).clock as *mut IUnknown);
        session_clear_presentation(session);
        session_clear_queued_topologies(session);
        session_clear_removed_topologies(session);
        session_submit_simple_command(session, SESSION_CMD_SHUTDOWN);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return hr;
}

unsafe extern "system" fn mfsession_GetClock(iface: *mut IMFMediaSession, clock: *mut *mut IMFClock) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));
    hr = session_is_shut_down(session);
    if hr >= 0 {
        *clock = (*session).clock as *mut IMFClock;
        IMFClock_AddRef(*clock);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return hr;
}

unsafe extern "system" fn mfsession_GetSessionCapabilities(iface: *mut IMFMediaSession, caps: *mut DWORD) -> HRESULT {
    let session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut hr: HRESULT = S_OK;

    if caps.is_null() {
        return E_POINTER;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));
    hr = session_is_shut_down(session);
    if hr >= 0 {
        *caps = (*session).caps;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return hr;
}
unsafe extern "system" fn mfsession_GetFullTopology(
    mut iface: *mut IMFMediaSession,
    mut flags: DWORD,
    mut id: TOPOID,
    mut topology: *mut *mut IMFTopology,
) -> HRESULT {
    let mut session: *mut media_session = impl_from_IMFMediaSession(iface);
    let mut queued: *mut queued_topology = ::core::ptr::null_mut::<queued_topology>();
    let mut topo_id: TOPOID = 0;
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
            b"dll/win32/mf/session.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"mfsession_GetFullTopology\0" as *const u8 as *const ::core::ffi::c_char,
            2595 as ::core::ffi::c_int,
            b"%p, %#lx, %s, %p.\n\0" as *const u8 as *const ::core::ffi::c_char,
            iface,
            flags,
            wine_dbgstr_longlong(id as ULONGLONG),
            topology,
        );
    }
    *topology = ::core::ptr::null_mut::<IMFTopology>();
    EnterCriticalSection(&raw mut (*session).cs);
    hr = session_is_shut_down(session);
    if hr >= 0 {
        if flags & MFSESSION_GETFULLTOPOLOGY_CURRENT as DWORD != 0 {
            if (*session).presentation.topo_status as ::core::ffi::c_uint
                != MF_TOPOSTATUS_INVALID as ::core::ffi::c_uint
            {
                *topology = (*session).presentation.current_topology;
            } else {
                hr = MF_E_INVALIDREQUEST;
            }
        } else {
            queued = ((*session).topologies.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<queued_topology>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut queued_topology;
            while &raw mut (*queued).entry != &raw mut (*session).topologies {
                if IMFTopology_GetTopologyID((*queued).topology, &raw mut topo_id) >= 0
                    && topo_id == id
                {
                    *topology = (*queued).topology;
                    break;
                } else {
                    queued = ((*queued).entry.next as *mut ::core::ffi::c_char).offset(
                        -(&raw mut (*::core::ptr::null_mut::<queued_topology>()).entry
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut queued_topology;
                }
            }
        }
        if !(*topology).is_null() {
            IMFTopology_AddRef(*topology);
        }
    }
    LeaveCriticalSection(&raw mut (*session).cs);
    return hr;
}

unsafe extern "system" fn session_get_service_QueryInterface(iface: *mut IMFGetService, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let session: *mut media_session = impl_from_IMFGetService(iface);
    return IMFMediaSession_QueryInterface(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface), riid, obj);
}

unsafe extern "system" fn session_get_service_AddRef(iface: *mut IMFGetService) -> ULONG {
    let session: *mut media_session = impl_from_IMFGetService(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_get_service_Release(iface: *mut IMFGetService) -> ULONG {
    let session: *mut media_session = impl_from_IMFGetService(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "C" fn session_video_renderer_test_func(sink: *mut IMFMediaSink) -> BOOL {
    let mut obj: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    /* Use first sink to support IMFVideoRenderer. */
    hr = IMFMediaSink_QueryInterface(
        sink,
        core::ptr::addr_of!(IID_IMFVideoRenderer),
        &mut obj as *mut *mut IUnknown as *mut *mut core::ffi::c_void,
    );
    if !obj.is_null() {
        IUnknown_Release(obj);
    }

    return (hr == S_OK) as BOOL;
}

unsafe extern "C" fn session_audio_renderer_test_func(sink: *mut IMFMediaSink) -> BOOL {
    mf_is_sar_sink(sink)
}

unsafe extern "C" fn session_get_renderer_node_service(
    mut session: *mut media_session,
    mut node_test_func: p_renderer_node_test_func,
    service: *const GUID,
    riid: *const IID,
    mut obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT = E_NOINTERFACE;
    let mut stream_sink: *mut IMFStreamSink = ::core::ptr::null_mut::<IMFStreamSink>();
    let mut node: *mut IMFTopologyNode = ::core::ptr::null_mut::<IMFTopologyNode>();
    let mut nodes: *mut IMFCollection = ::core::ptr::null_mut::<IMFCollection>();
    let mut sink: *mut IMFMediaSink = ::core::ptr::null_mut::<IMFMediaSink>();
    let mut i: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
    if !(*session).presentation.current_topology.is_null() {
        if IMFTopology_GetOutputNodeCollection(
            (*session).presentation.current_topology,
            core::ptr::addr_of_mut!(nodes),
        ) >= 0
        {
            loop {
                let fresh18 = i;
                i = i.wrapping_add(1);
                if !(IMFCollection_GetElement(
                    nodes,
                    fresh18 as DWORD,
                    core::ptr::addr_of_mut!(node) as *mut *mut IUnknown,
                ) == S_OK)
                {
                    break;
                }
                if topology_node_get_object(
                    node,
                    core::ptr::addr_of!(IID_IMFStreamSink),
                    core::ptr::addr_of_mut!(stream_sink) as *mut *mut ::core::ffi::c_void,
                ) >= 0
                {
                    if IMFStreamSink_GetMediaSink(stream_sink, core::ptr::addr_of_mut!(sink))
                        >= 0
                    {
                        if node_test_func.unwrap_unchecked()(sink) != 0 {
                            hr = MFGetService(
                                sink as *mut IUnknown,
                                service,
                                riid,
                                obj,
                            );
                            if hr < 0 {
                                if (*__wine_dbch___default).flags as ::core::ffi::c_int
                                    & (1 as ::core::ffi::c_int)
                                        << __WINE_DBCL_WARN as ::core::ffi::c_int
                                    != 0
                                {
                                    let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
                                    let __dbcl: __wine_debug_class = __WINE_DBCL_WARN;
                                    ros_dbg_log(
                                        (__dbcl) as u32,
                                        __dbch,
                                        b"dll/win32/mf/session.c\0"
                                            as *const u8 as *const ::core::ffi::c_char,
                                        b"session_get_renderer_node_service\0" as *const u8
                                            as *const ::core::ffi::c_char,
                                        2714 as ::core::ffi::c_int,
                                        b"Failed to get service from renderer node, %#lx.\n\0"
                                            as *const u8 as *const ::core::ffi::c_char,
                                        hr,
                                    );
                                }
                            }
                        }
                        IMFMediaSink_Release(sink);
                    }
                    IMFStreamSink_Release(stream_sink);
                }

                IMFTopologyNode_Release(node);

                if !(*obj).is_null() {
                    break;
                }
            }

            IMFCollection_Release(nodes);
        }
    }

    return hr;
}

unsafe extern "C" fn session_get_audio_render_service(
    session: *mut media_session,
    service: *const GUID,
    riid: *const IID,
    obj: *mut *mut core::ffi::c_void,
) -> HRESULT {
    return session_get_renderer_node_service(
        session,
        Some(session_audio_renderer_test_func as unsafe extern "C" fn(*mut IMFMediaSink) -> BOOL),
        service,
        riid,
        obj,
    );
}

unsafe extern "C" fn session_get_video_render_service(
    session: *mut media_session,
    service: *const GUID,
    riid: *const IID,
    obj: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    return session_get_renderer_node_service(
        session,
        Some(session_video_renderer_test_func as unsafe extern "C" fn(*mut IMFMediaSink) -> BOOL),
        service,
        riid,
        obj,
    );
}

unsafe extern "system" fn session_get_service_GetService(iface: *mut IMFGetService, service: REFGUID, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let session: *mut media_session = impl_from_IMFGetService(iface);
    let mut hr: HRESULT = S_OK;

    TRACE!("%p, %s, %s, %p.\n", iface, debugstr_guid(service), debugstr_guid(riid), obj);

    *obj = core::ptr::null_mut();

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));
    hr = session_is_shut_down(session);
    if hr < 0 {
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MF_RATE_CONTROL_SERVICE)) {
        if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFRateSupport))) != 0) {
            *obj = core::ptr::addr_of_mut!((*session).IMFRateSupport_iface) as *mut core::ffi::c_void;
        } else if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFRateControl))) != 0) {
            *obj = core::ptr::addr_of_mut!((*session).IMFRateControl_iface) as *mut core::ffi::c_void;
        } else {
            hr = E_NOINTERFACE;
        }

        if !(*obj).is_null() {
            IUnknown_AddRef(*obj as *mut IUnknown);
        }
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MF_LOCAL_MFT_REGISTRATION_SERVICE)) {
        hr = IMFLocalMFTRegistration_QueryInterface(core::ptr::addr_of_mut!(local_mft_registration), riid, obj);
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MF_TOPONODE_ATTRIBUTE_EDITOR_SERVICE)) {
        *obj = core::ptr::addr_of_mut!((*session).IMFTopologyNodeAttributeEditor_iface) as *mut core::ffi::c_void;
        IUnknown_AddRef(*obj as *mut IUnknown);
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_VIDEO_RENDER_SERVICE)) {
        hr = session_get_video_render_service(session, service, riid, obj);
    } else if IsEqualGUID!(service, core::ptr::addr_of!(MR_POLICY_VOLUME_SERVICE))
        || IsEqualGUID!(service, core::ptr::addr_of!(MR_STREAM_VOLUME_SERVICE))
    {
        hr = session_get_audio_render_service(session, service, riid, obj);
    } else {
        FIXME!("Unsupported service %s.\n", debugstr_guid(service));
        hr = E_NOINTERFACE;
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return hr;
}

unsafe extern "system" fn session_commands_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn session_commands_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_commands_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_commands_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_commands_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn session_commands_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn session_commands_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let mut op: *mut session_op = impl_op_from_IUnknown(IMFAsyncResult_GetStateNoAddRef(result));
    let mut session: *mut media_session = impl_from_commands_callback_IMFAsyncCallback(iface);

    EnterCriticalSection(&raw mut (*session).cs);

    assert!((*session).command_state == COMMAND_STATE_SUBMITTED);
    list_remove(&raw mut (*op).entry);

    match (*op).command {
        SESSION_CMD_CLEAR_TOPOLOGIES => {
            (*session).presentation.flags &= !(SESSION_FLAG_SOURCE_SHUTDOWN);
            session_clear_topologies(session);
        }
        SESSION_CMD_SET_TOPOLOGY => {
            (*session).presentation.flags &= !(SESSION_FLAG_SOURCE_SHUTDOWN);
            session_set_topology(session, (*op).__bindgen_anon_1.set_topology.flags, (*op).__bindgen_anon_1.set_topology.topology);
            session_command_complete(session);
        }
        SESSION_CMD_START => {
            if ((*session).presentation.flags & SESSION_FLAG_SOURCE_SHUTDOWN) != 0 {
                session_command_complete_with_event(session, MESessionStarted, MF_E_INVALIDREQUEST, core::ptr::null());
            } else {
                session_start(session, &raw mut (*op).__bindgen_anon_1.start.time_format, &raw mut (*op).__bindgen_anon_1.start.start_position);
            }
        }
        SESSION_CMD_PAUSE => {
            if ((*session).presentation.flags & SESSION_FLAG_SOURCE_SHUTDOWN) != 0 {
                session_command_complete_with_event(session, MESessionPaused, MF_E_SHUTDOWN, core::ptr::null());
            } else {
                session_pause(session);
            }
        }
        SESSION_CMD_STOP => {
            if ((*session).presentation.flags & SESSION_FLAG_SOURCE_SHUTDOWN) != 0 {
                session_command_complete_with_event(session, MESessionStopped, MF_E_SHUTDOWN, core::ptr::null());
            } else {
                session_stop(session);
            }
        }
        SESSION_CMD_CLOSE => {
            if ((*session).presentation.flags & SESSION_FLAG_SOURCE_SHUTDOWN) != 0 {
                session_command_complete_with_event(session, MESessionClosed, MF_E_SHUTDOWN, core::ptr::null());
            } else {
                session_close(session);
            }
        }
        SESSION_CMD_SET_RATE => {
            if ((*session).presentation.flags & SESSION_FLAG_SOURCE_SHUTDOWN) != 0 {
                session_command_complete_with_event(session, MESessionRateChanged, MF_E_SHUTDOWN, core::ptr::null());
            } else {
                session_set_rate(session, (*op).__bindgen_anon_1.set_rate.thin, (*op).__bindgen_anon_1.set_rate.rate);
            }
        }
        SESSION_CMD_SHUTDOWN => {
            session_clear_command_list(session);
            session_command_complete(session);
        }
        _ => {}
    }

    LeaveCriticalSection(&raw mut (*session).cs);

    IUnknown_Release(&raw mut (*op).IUnknown_iface);

    return S_OK;
}

unsafe extern "system" fn session_sa_ready_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const GUID, obj: *mut *mut ::core::ffi::c_void) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *obj = iface as *mut ::core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn session_sa_ready_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_sa_ready_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_sa_ready_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut session: *mut media_session = impl_from_sa_ready_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn session_sa_ready_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn session_sa_ready_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let notify: *mut IMFVideoSampleAllocatorNotify = IMFAsyncResult_GetStateNoAddRef(result) as *mut IMFVideoSampleAllocatorNotify;
    let topo_node: *mut topo_node = impl_node_from_IMFVideoSampleAllocatorNotify(notify);
    let mut up_node: *mut topo_node = core::ptr::null_mut();
    let session: *mut media_session = impl_from_sa_ready_callback_IMFAsyncCallback(iface);
    let mut output: DWORD = 0;

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    if (*topo_node).u.sink.requests != 0 {
        up_node = session_get_topo_node_input(session, topo_node, 0, core::ptr::addr_of_mut!(output));
        if up_node.is_null() {
            WARN!("Failed to node %p/%u input\n", topo_node, 0);
        } else {
            session_deliver_pending_samples(session, up_node);
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return S_OK;
}

unsafe extern "C" fn session_handle_source_shutdown(mut session: *mut media_session) {
    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    match ((*session).command_state as ::core::ffi::c_uint) as i32 {
        COMMAND_STATE_STARTING_SOURCES | COMMAND_STATE_RESTARTING_SOURCES | COMMAND_STATE_PREROLLING_SINKS | COMMAND_STATE_STARTING_SINKS => {
            session_clear_presentation(session);
            (*session).state = SESSION_STATE_STOPPED;
            session_command_complete_with_event(
                session,
                MESessionStarted as ::core::ffi::c_int as MediaEventType,
                MF_E_INVALIDREQUEST,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        COMMAND_STATE_PAUSING_SINKS | COMMAND_STATE_PAUSING_SOURCES => {
            session_clear_presentation(session);
            (*session).state = SESSION_STATE_STOPPED;
            session_command_complete_with_event(
                session,
                MESessionPaused as ::core::ffi::c_int as MediaEventType,
                MF_E_SHUTDOWN,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        COMMAND_STATE_STOPPING_SINKS | COMMAND_STATE_STOPPING_SOURCES => {
            session_clear_presentation(session);
            (*session).state = SESSION_STATE_STOPPED;
            session_command_complete_with_event(
                session,
                MESessionStopped as ::core::ffi::c_int as MediaEventType,
                MF_E_SHUTDOWN,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        COMMAND_STATE_CLOSING_SINKS | COMMAND_STATE_CLOSING_SOURCES | COMMAND_STATE_FINALIZING_SINKS => {
            session_clear_presentation(session);
            (*session).state = SESSION_STATE_CLOSED;
            session_command_complete_with_event(
                session,
                MESessionClosed as ::core::ffi::c_int as MediaEventType,
                MF_E_SHUTDOWN,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
        COMMAND_STATE_COMPLETE => {
            if (*session).state as ::core::ffi::c_uint == SESSION_STATE_STARTED as ::core::ffi::c_uint
                || (*session).state as ::core::ffi::c_uint == SESSION_STATE_PAUSED as ::core::ffi::c_uint
            {
                session_set_stopped(
                    session,
                    MESessionStopped as ::core::ffi::c_int as MediaEventType,
                    MF_E_SHUTDOWN,
                );
            }
        }
        COMMAND_STATE_SUBMITTED => {}
        _ => {}
    }

    (*session).presentation.flags |= SESSION_FLAG_SOURCE_SHUTDOWN as ::core::ffi::c_uint;

    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));
}

unsafe extern "system" fn session_events_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback))) != 0)
        || ((IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown))) != 0)
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn session_events_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_events_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_events_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_events_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn session_events_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "C" fn session_add_media_stream(
    session: *mut media_session,
    source: *mut IMFMediaSource,
    stream: *mut IMFMediaStream,
) -> HRESULT {
    let mut node: *mut topo_node = core::ptr::null_mut();
    let mut sd: *mut IMFStreamDescriptor = core::ptr::null_mut();
    let mut stream_id: DWORD = 0;
    let mut hr: HRESULT = 0;

    hr = IMFMediaStream_GetStreamDescriptor(stream, core::ptr::addr_of_mut!(sd));
    if hr < 0 {
        return hr;
    }

    hr = IMFStreamDescriptor_GetStreamIdentifier(sd, core::ptr::addr_of_mut!(stream_id));
    IMFStreamDescriptor_Release(sd);
    if hr < 0 {
        return hr;
    }

    node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(topo_node, entry) as isize),
    ) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        if (*node).type_ as ::core::ffi::c_uint == MF_TOPOLOGY_SOURCESTREAM_NODE as ::core::ffi::c_uint
            && (*node).u.source.source == source
            && (*node).u.source.stream_id == stream_id
        {
            if !(*node).object.source_stream.is_null() {
                WARN!("Node already has stream set.\n");
                return S_FALSE;
            }

            (*node).object.source_stream = stream;
            IMFMediaStream_AddRef((*node).object.source_stream);
            break;
        } else {
            node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(topo_node, entry) as isize),
            ) as *mut topo_node;
        }
    }

    return S_OK;
}

unsafe extern "C" fn session_is_source_nodes_state(
    session: *mut media_session,
    state: object_state,
) -> BOOL {
    let mut source: *mut media_source;
    let mut node: *mut topo_node;

    source = ((*session).presentation.sources.next as *mut u8)
        .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
        if (*source).state as u32 != state as u32 {
            return (FALSE) as BOOL;
        }
        source = ((*source).entry.next as *mut u8)
            .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
    }

    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
        if (*node).type_ as u32 == MF_TOPOLOGY_SOURCESTREAM_NODE as u32
            && (*node).state as u32 != state as u32
        {
            return (FALSE) as BOOL;
        }
        node = ((*node).entry.next as *mut u8)
            .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn session_is_output_nodes_state(
    mut session: *mut media_session,
    mut state: object_state,
) -> BOOL {
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    node = ((*session).presentation.nodes.next as *mut u8)
        .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
        if (*node).type_ as ::core::ffi::c_uint
            == MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint
            && (*node).state as ::core::ffi::c_uint != state as ::core::ffi::c_uint
        {
            return (FALSE) as BOOL;
        }
        node = ((*node).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn session_get_object_state_for_event(
    mut session: *mut media_session,
    mut event: MediaEventType,
) -> object_state {
    match event {
        203 | 204 | 201 | 202 | 301 => return OBJ_STATE_STARTED,
        209 | 210 | 303 => return OBJ_STATE_PAUSED,
        302 => {
            /* MEStreamSinkStopped may be sent when starting a new topology because of the sequence (which occurs in native):
             * media sink SetPresentationClock() -> AddClockStateSink() -> callback Invoke() -> OnClockStop().
             * Callback notification is suppressed for the first topology because the clock state is MFCLOCK_STATE_INVALID. */
            if (*session).command_state as ::core::ffi::c_uint
                == COMMAND_STATE_PREROLLING_SINKS as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*session).command_state as ::core::ffi::c_uint
                    == COMMAND_STATE_STARTING_SINKS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return OBJ_STATE_INVALID;
            }
            /* fall through */
        }
        207 | 208 => {}
        307 => return OBJ_STATE_PREROLLED,
        _ => return OBJ_STATE_INVALID,
    }
    return OBJ_STATE_STOPPED;
}

unsafe extern "C" fn session_start_clock(session: *mut media_session) -> HRESULT {
    let mut start_offset: LONGLONG = 0;
    let mut hr: HRESULT;

    if IsEqualGUID!(
        core::ptr::addr_of_mut!((*session).presentation.time_format),
        core::ptr::addr_of!(GUID_NULL),
    ) != false
    {
        if (*session).presentation.start_position.vt == VT_EMPTY {
            start_offset = (PRESENTATION_CURRENT_POSITION as _);
        } else if (*session).presentation.start_position.vt == (VT_I8 as u16) {
            start_offset = (*session).presentation.start_position.__bindgen_anon_1.hVal.QuadPart;
        } else {
            FIXME!(
                "Unhandled position type %d.\n",
                (*session).presentation.start_position.vt as core::ffi::c_uint
            );
        }
    } else {
        FIXME!(
            "Unhandled time format %s.\n",
            debugstr_guid(core::ptr::addr_of_mut!((*session).presentation.time_format))
        );
    }

    hr = IMFPresentationClock_Start((*session).clock, start_offset);
    if hr < 0 {
        WARN!("Failed to start session clock, hr %#lx.\n", hr);
    }

    return hr;
}

unsafe extern "C" fn session_get_node_object(
    session: *mut media_session,
    object: *mut IUnknown,
    node_type: MF_TOPOLOGY_TYPE,
) -> *mut topo_node {
    let mut node: *mut topo_node = core::ptr::null_mut();
    node = ((*session).presentation.nodes.next as *mut u8)
        .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry) != core::ptr::addr_of_mut!((*session).presentation.nodes) {
        if (*node).type_ as u32 == node_type as u32 && object == (*node).object.object {
            return node;
        }
        node = ((*node).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    }
    return core::ptr::null_mut();
}

unsafe extern "C" fn session_set_node_object_state(
    mut session: *mut media_session,
    mut object: *mut IUnknown,
    mut node_type: MF_TOPOLOGY_TYPE,
    mut state: object_state,
) -> BOOL {
    let mut node: *mut topo_node = core::ptr::null_mut();
    let mut changed: BOOL = (FALSE as i32);

    node = session_get_node_object(session, object, node_type);
    if !node.is_null() {
        changed = ((*node).state as ::core::ffi::c_uint != state as ::core::ffi::c_uint)
            as ::core::ffi::c_int as BOOL;
        (*node).state = state;
    }

    return changed;
}

unsafe extern "C" fn session_set_source_object_state(
    session: *mut media_session,
    object: *mut IUnknown,
    event_type: MediaEventType,
) {
    let mut source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut stream_sink: *mut IMFStreamSink = ::core::ptr::null_mut::<IMFStreamSink>();
    let mut src: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut sink: *mut media_sink = ::core::ptr::null_mut::<media_sink>();
    let mut state: object_state = OBJ_STATE_STOPPED;
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    let mut changed: BOOL = (FALSE as i32);
    let mut i: DWORD = 0;
    let mut count: DWORD = 0;
    let mut hr: HRESULT = 0;

    state = session_get_object_state_for_event(session, event_type);
    if state as ::core::ffi::c_uint == OBJ_STATE_INVALID as ::core::ffi::c_uint {
        return;
    }

    match event_type {
        203 | 201 | 209 | 207 => {
            src = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut media_source;
            while &raw mut (*src).entry != &raw mut (*session).presentation.sources {
                if object == (*src).__bindgen_anon_1.object {
                    changed = ((*src).state as ::core::ffi::c_uint != state as ::core::ffi::c_uint)
                        as ::core::ffi::c_int as BOOL;
                    (*src).state = state;
                    break;
                } else {
                    src = ((*src).entry.next as *mut ::core::ffi::c_char).offset(
                        -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut media_source;
                }
            }
        }
        204 | 202 | 210 | 208 => {
            changed = session_set_node_object_state(
                session,
                object,
                MF_TOPOLOGY_SOURCESTREAM_NODE,
                state,
            );
        }
        _ => {}
    }

    if changed == 0 {
        return;
    }

    'cmd: {
        match ((*session).command_state as ::core::ffi::c_uint) as i32 {
            COMMAND_STATE_STARTING_SOURCES => {
                if session_is_source_nodes_state(session, OBJ_STATE_STARTED) == 0 {
                    break 'cmd;
                }

                session_set_topo_status(session, S_OK, MF_TOPOSTATUS_STARTED_SOURCE);

                if event_type == MESourceStarted as ::core::ffi::c_int as MediaEventType
                    || event_type == MEStreamStarted as ::core::ffi::c_int as MediaEventType
                {
                    node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
                        -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut topo_node;
                    while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
                        if (*node).type_ as ::core::ffi::c_uint
                            == MF_TOPOLOGY_TRANSFORM_NODE as ::core::ffi::c_uint
                        {
                            if (*node).u.transform.async_ != 0 && {
                                hr = IMFMediaEventGenerator_BeginGetEvent(
                                    (*node).u.transform.event_source,
                                    &raw mut (*session).events_callback,
                                    (*node).u.transform.event_source as *mut IUnknown,
                                );
                                hr < 0
                            } {
                                WARN!("Failed to subscribe to transform events, hr %#lx.\n", hr);
                            }
                            IMFTransform_ProcessMessage(
                                (*node).object.transform,
                                MFT_MESSAGE_NOTIFY_START_OF_STREAM,
                                0,
                            );
                        }
                        node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
                            -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry
                                as ::core::ffi::c_ulonglong as isize),
                        ) as *mut topo_node;
                    }
                }

                session_subscribe_sinks(session);
                session_set_presentation_clock(session);

                if (*session).presentation.rate != 0.0f32
                    && ((*session).presentation.flags & SESSION_FLAG_NEEDS_PREROLL as ::core::ffi::c_uint) != 0
                    && session_is_output_nodes_state(session, OBJ_STATE_STOPPED) != 0
                {
                    let mut preroll_time: MFTIME = 0 as MFTIME;

                    if (*session).presentation.start_position.vt as ::core::ffi::c_int
                        == VT_I8 as ::core::ffi::c_int
                    {
                        preroll_time = (*session)
                            .presentation
                            .start_position

                            .__bindgen_anon_1.hVal
                            .QuadPart as MFTIME;
                    }

                    sink = ((*session).presentation.sinks.next as *mut ::core::ffi::c_char).offset(
                        -(&raw mut (*::core::ptr::null_mut::<media_sink>()).entry
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut media_sink;
                    while &raw mut (*sink).entry != &raw mut (*session).presentation.sinks {
                        if !(*sink).preroll.is_null() {
                            hr = IMFMediaSinkPreroll_NotifyPreroll((*sink).preroll, preroll_time);
                            if hr < 0 {
                                WARN!("Preroll notification failed, hr %#lx.\n", hr);
                            }
                        } else if IMFMediaSink_GetStreamSinkCount(
                            (*sink).__bindgen_anon_1.sink,
                            &raw mut count,
                        ) >= 0
                        {
                            i = 0;
                            while i < count {
                                if IMFMediaSink_GetStreamSinkByIndex(
                                    (*sink).__bindgen_anon_1.sink,
                                    i,
                                    &raw mut stream_sink,
                                ) >= 0
                                {
                                    session_set_node_object_state(
                                        session,
                                        stream_sink as *mut IUnknown,
                                        MF_TOPOLOGY_OUTPUT_NODE,
                                        OBJ_STATE_PREROLLED,
                                    );
                                    IMFStreamSink_Release(stream_sink);
                                }
                                i = i.wrapping_add(1);
                            }
                        }
                        sink = ((*sink).entry.next as *mut ::core::ffi::c_char).offset(
                            -(&raw mut (*::core::ptr::null_mut::<media_sink>()).entry
                                as ::core::ffi::c_ulonglong as isize),
                        ) as *mut media_sink;
                    }
                    (*session).command_state = COMMAND_STATE_PREROLLING_SINKS;
                } else {
                    if ((*session).presentation.flags & SESSION_FLAG_RESTARTING as ::core::ffi::c_uint) != 0 {
                        (*session).presentation.flags &=
                            !(SESSION_FLAG_RESTARTING as ::core::ffi::c_uint);
                        session_flush_sinks(session);
                    }

                    hr = session_start_clock(session);
                    if hr >= 0 {
                        /* If sinks are already started, start session immediately. This can happen when doing a
                         * seek from SESSION_STATE_STARTED (i.e. a seek without pause/stop) */
                        if session_is_output_nodes_state(session, OBJ_STATE_STARTED) != 0 {
                            session_set_started(session);
                        } else {
                            (*session).command_state = COMMAND_STATE_STARTING_SINKS;
                        }
                    } else {
                        WARN!("Failed to start session clock %p, hr %#lx.\n", session, hr);
                        session_command_complete_with_event(
                            session,
                            MESessionStarted as ::core::ffi::c_int as MediaEventType,
                            hr,
                            ::core::ptr::null::<PROPVARIANT>(),
                        );
                    }
                }
            }
            COMMAND_STATE_RESTARTING_SOURCES => {
                if session_is_source_nodes_state(session, OBJ_STATE_STOPPED) == 0 {
                    break 'cmd;
                }

                (*session).presentation.flags &= !(SESSION_FLAG_PRESENTATION_ENDING as ::core::ffi::c_uint);
                (*session).state = SESSION_STATE_STOPPED;
                (*session).command_state = COMMAND_STATE_STARTING_SOURCES;
                (*session).presentation.flags |= SESSION_FLAG_RESTARTING as ::core::ffi::c_uint;
                session_flush_transforms(session);

                /* Start sources */
                source = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
                    -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry
                        as ::core::ffi::c_ulonglong as isize),
                ) as *mut media_source;
                while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
                    hr = IMFMediaSource_Start(
                        (*source).__bindgen_anon_1.source,
                        (*source).pd,
                        &raw mut (*session).presentation.time_format,
                        &raw mut (*session).presentation.start_position,
                    );
                    if hr < 0 {
                        WARN!(
                            "Failed to start media source %p, hr %#lx.\n",
                            (*source).__bindgen_anon_1.source,
                            hr
                        );
                        if hr == MF_E_SHUTDOWN {
                            return session_handle_source_shutdown(session);
                        }
                        return session_command_complete_with_event(
                            session,
                            MESessionStarted as ::core::ffi::c_int as MediaEventType,
                            hr,
                            ::core::ptr::null::<PROPVARIANT>(),
                        );
                    }
                    source = ((*source).entry.next as *mut ::core::ffi::c_char).offset(
                        -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry
                            as ::core::ffi::c_ulonglong as isize),
                    ) as *mut media_source;
                }
            }
            COMMAND_STATE_PAUSING_SOURCES => {
                if session_is_source_nodes_state(session, OBJ_STATE_PAUSED) == 0 {
                    break 'cmd;
                }

                session_set_paused(session, S_OK);
            }
            COMMAND_STATE_STOPPING_SOURCES => {
                if session_is_source_nodes_state(session, OBJ_STATE_STOPPED) == 0 {
                    break 'cmd;
                }

                session_flush_nodes(session);
                session_set_caps(
                    session,
                    (*session).caps & !(MFSESSIONCAP_PAUSE as DWORD),
                );
                if ((*session).presentation.flags & SESSION_FLAG_PRESENTATION_ENDING as ::core::ffi::c_uint) != 0 {
                    session_set_stopped(
                        session,
                        MESessionEnded as ::core::ffi::c_int as MediaEventType,
                        S_OK,
                    );
                } else {
                    session_set_stopped(
                        session,
                        MESessionStopped as ::core::ffi::c_int as MediaEventType,
                        S_OK,
                    );
                }
            }
            COMMAND_STATE_CLOSING_SOURCES => {
                if session_is_source_nodes_state(session, OBJ_STATE_STOPPED) == 0 {
                    break 'cmd;
                }

                session_flush_nodes(session);
                session_set_caps(
                    session,
                    (*session).caps & !(MFSESSIONCAP_PAUSE as DWORD),
                );
                session_finalize_sinks(session);
            }
            COMMAND_STATE_SUBMITTED
            | COMMAND_STATE_COMPLETE
            | COMMAND_STATE_PREROLLING_SINKS
            | COMMAND_STATE_STARTING_SINKS
            | COMMAND_STATE_PAUSING_SINKS
            | COMMAND_STATE_STOPPING_SINKS
            | COMMAND_STATE_CLOSING_SINKS
            | COMMAND_STATE_FINALIZING_SINKS => {
                WARN!(
                    "Ignoring source state change in command state %#x\n",
                    (*session).command_state as ::core::ffi::c_uint
                );
            }
            _ => {}
        }
    }
}

unsafe extern "C" fn session_set_sink_stream_state(
    mut session: *mut media_session,
    mut stream: *mut IMFStreamSink,
    mut event_type: MediaEventType,
) {
    let mut source: *mut media_source = core::ptr::null_mut::<media_source>();
    let mut state: object_state = OBJ_STATE_STOPPED;
    let mut hr: HRESULT = S_OK;
    let mut changed: BOOL = 0;

    state = session_get_object_state_for_event(session, event_type);
    if state as ::core::ffi::c_uint == (OBJ_STATE_INVALID as u32) {
        return;
    }

    changed = session_set_node_object_state(
        session,
        stream as *mut IUnknown,
        MF_TOPOLOGY_OUTPUT_NODE,
        state,
    );
    if changed == 0 {
        return;
    }

    match ((*session).command_state as ::core::ffi::c_uint) as i32 {
        COMMAND_STATE_PREROLLING_SINKS => {
            if !(session_is_output_nodes_state(session, OBJ_STATE_PREROLLED) == 0) {
                if session_start_clock(session) >= 0 {
                    (*session).command_state = COMMAND_STATE_STARTING_SINKS;
                }
            }
        }
        COMMAND_STATE_STARTING_SINKS => {
            if !(session_is_output_nodes_state(session, OBJ_STATE_STARTED) == 0) {
                session_set_started(session);
            }
        }
        COMMAND_STATE_PAUSING_SINKS => {
            if !(session_is_output_nodes_state(session, OBJ_STATE_PAUSED) == 0) {
                (*session).command_state = COMMAND_STATE_PAUSING_SOURCES;
                source = ((*session).presentation.sources.next as *mut u8).offset(
                    -(core::mem::offset_of!(media_source, entry) as isize),
                ) as *mut media_source;
                while core::ptr::addr_of_mut!((*source).entry)
                    != core::ptr::addr_of_mut!((*session).presentation.sources)
                {
                    hr = IMFMediaSource_Pause((*source).__bindgen_anon_1.source);
                    if hr < 0 {
                        break;
                    }
                    source = ((*source).entry.next as *mut u8).offset(
                        -(core::mem::offset_of!(media_source, entry) as isize),
                    ) as *mut media_source;
                }
                if hr < 0 {
                    session_set_paused(session, hr);
                }
            }
        }
        COMMAND_STATE_STOPPING_SINKS => {
            if !(session_is_output_nodes_state(session, OBJ_STATE_STOPPED) == 0) {
                (*session).command_state = COMMAND_STATE_STOPPING_SOURCES;
                source = ((*session).presentation.sources.next as *mut u8).offset(
                    -(core::mem::offset_of!(media_source, entry) as isize),
                ) as *mut media_source;
                while core::ptr::addr_of_mut!((*source).entry)
                    != core::ptr::addr_of_mut!((*session).presentation.sources)
                {
                    hr = IMFMediaSource_Stop((*source).__bindgen_anon_1.source);
                    if hr < 0 {
                        break;
                    }
                    source = ((*source).entry.next as *mut u8).offset(
                        -(core::mem::offset_of!(media_source, entry) as isize),
                    ) as *mut media_source;
                }
                if hr < 0 {
                    session_set_stopped(session, MESessionStopped, hr);
                }
            }
        }
        COMMAND_STATE_CLOSING_SINKS => {
            if !(session_is_output_nodes_state(session, OBJ_STATE_STOPPED) == 0) {
                (*session).command_state = COMMAND_STATE_CLOSING_SOURCES;
                source = ((*session).presentation.sources.next as *mut u8).offset(
                    -(core::mem::offset_of!(media_source, entry) as isize),
                ) as *mut media_source;
                while core::ptr::addr_of_mut!((*source).entry)
                    != core::ptr::addr_of_mut!((*session).presentation.sources)
                {
                    hr = IMFMediaSource_Stop((*source).__bindgen_anon_1.source);
                    if hr < 0 {
                        break;
                    }
                    source = ((*source).entry.next as *mut u8).offset(
                        -(core::mem::offset_of!(media_source, entry) as isize),
                    ) as *mut media_source;
                }
                if hr < 0 {
                    session_set_closed(session, hr);
                }
            }
        }
        COMMAND_STATE_SUBMITTED
        | COMMAND_STATE_COMPLETE
        | COMMAND_STATE_RESTARTING_SOURCES
        | COMMAND_STATE_STARTING_SOURCES
        | COMMAND_STATE_PAUSING_SOURCES
        | COMMAND_STATE_STOPPING_SOURCES
        | COMMAND_STATE_CLOSING_SOURCES
        | COMMAND_STATE_FINALIZING_SINKS => {
            WARN!(
                "Ignoring sink state change in command state %#x\n",
                (*session).command_state as ::core::ffi::c_uint
            );
        }
        _ => {}
    }
}

unsafe extern "C" fn transform_get_external_output_sample(
    session: *const media_session,
    transform: *mut topo_node,
    output: DWORD,
    stream_info: *const MFT_OUTPUT_STREAM_INFO,
    sample: *mut *mut IMFSample,
) -> HRESULT {
    let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut();
    let mut topo_node: *mut topo_node = core::ptr::null_mut();
    let mut buffer_size: ::core::ffi::c_uint = 0;
    let mut input: DWORD = 0;
    let mut hr: HRESULT = 0;

    topo_node = session_get_topo_node_output(session, transform, output, core::ptr::addr_of_mut!(input));
    if topo_node.is_null() {
        WARN!("Failed to node %p/%lu output.\n", transform, output);
        return MF_E_UNEXPECTED;
    }

    if (*topo_node).type_ == MF_TOPOLOGY_OUTPUT_NODE && !(*topo_node).u.sink.allocator.is_null() {
        hr = IMFVideoSampleAllocator_AllocateSample((*topo_node).u.sink.allocator, sample);
    } else {
        buffer_size = max!(
            (*stream_info).cbSize,
            (*(*transform).u.transform.outputs.offset(output as isize)).min_buffer_size as DWORD,
        ) as ::core::ffi::c_uint;

        hr = MFCreateAlignedMemoryBuffer(
            buffer_size as DWORD,
            (*stream_info).cbAlignment,
            core::ptr::addr_of_mut!(buffer),
        );
        if hr >= 0 {
            hr = MFCreateSample(sample);
        }

        if hr >= 0 {
            hr = IMFSample_AddBuffer(*sample, buffer);
        }

        if !buffer.is_null() {
            IMFMediaBuffer_Release(buffer);
        }
    }

    return hr;
}

unsafe extern "C" fn transform_stream_update_output_type(node: *mut topo_node, stream: *mut transform_stream, id: UINT, old_output_type: *mut IMFMediaType, new_output_type: *mut *mut IMFMediaType) -> HRESULT {
    let mut subtype: GUID = core::mem::zeroed();
    let mut desired: GUID = core::mem::zeroed();
    let mut i: UINT = 0;
    let mut hr: HRESULT;

    IMFMediaType_GetGUID(old_output_type, &raw const MF_MT_SUBTYPE, &raw mut desired);

    /* find an available output type matching the desired subtype */
    loop {
        let fresh16 = i;
        i = i.wrapping_add(1);
        hr = IMFTransform_GetOutputAvailableType((*node).object.transform, id as DWORD, fresh16 as DWORD, new_output_type);
        if !(hr >= 0) {
            break;
        }
        IMFMediaType_GetGUID(*new_output_type, &raw const MF_MT_SUBTYPE, &raw mut subtype);
        if IsEqualGUID!(&raw mut subtype, &raw mut desired) != false {
            hr = IMFTransform_SetOutputType((*node).object.transform, id as DWORD, *new_output_type, 0);
            if hr < 0 {
                IMFMediaType_Release(*new_output_type);
                break;
            }
            return S_OK;
        }
        IMFMediaType_Release(*new_output_type);
    }

    *new_output_type = core::ptr::null_mut::<IMFMediaType>();
    return hr;
}

unsafe extern "C" fn transform_node_format_changed(node: *mut topo_node, buffers: *mut MFT_OUTPUT_DATA_BUFFER) -> HRESULT {
    let mut hr: HRESULT = S_OK;
    let mut i: ::core::ffi::c_uint = 0;
    while hr >= 0 && i < (*node).u.transform.output_count {
        'body: {
            let stream: *mut transform_stream = (*node).u.transform.outputs.offset(i as isize);
            let mut old_output_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
            let mut new_output_type: *mut IMFMediaType = core::ptr::null_mut::<IMFMediaType>();
            let id: UINT = (*buffers.offset(i as isize)).dwStreamID as UINT;

            if ((*buffers.offset(i as isize)).dwStatus & MFT_OUTPUT_DATA_BUFFER_FORMAT_CHANGE) == 0 {
                break 'body;
            }

            hr = IMFTransform_GetOutputCurrentType(
                (*node).object.transform,
                id as DWORD,
                core::ptr::addr_of_mut!(old_output_type),
            );
            if hr >= 0 {
                hr = transform_stream_update_output_type(
                    node,
                    stream,
                    id,
                    old_output_type,
                    core::ptr::addr_of_mut!(new_output_type),
                );
                if hr >= 0 {
                    if !(*buffers.offset(i as isize)).pEvents.is_null() || {
                        hr = MFCreateCollection(core::ptr::addr_of_mut!((*buffers.offset(i as isize)).pEvents));
                        hr >= 0
                    } {
                        hr = transform_stream_push_format_change(stream, new_output_type);
                        if hr < 0 {
                            WARN!("Failed to queue format change event, hr %#lx\n", hr);
                        }
                    }
                    IMFMediaType_Release(new_output_type);
                }
                IMFMediaType_Release(old_output_type);
            }
        }
        i = i.wrapping_add(1);
    }
    return hr;
}

unsafe extern "C" fn transform_stream_update_input_type(node: *mut topo_node, input: UINT, media_type: *mut IMFMediaType) -> HRESULT {
    let mut old_output_types: *mut *mut IMFMediaType = core::ptr::null_mut();
    let mut new_output_type: *mut IMFMediaType = core::ptr::null_mut();
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut output: UINT = 0;
    let mut hr: HRESULT = 0;

    old_output_types = calloc(((*node).u.transform.output_count as usize) as _, (core::mem::size_of::<IMFMediaType>()) as _) as *mut *mut IMFMediaType;
    if old_output_types.is_null() {
        return E_OUTOFMEMORY;
    }

    'done: {
        output = 0;
        while output < (*node).u.transform.output_count {
            let id: UINT = transform_node_get_stream_id(node, (TRUE) as BOOL, output) as UINT;
            hr = IMFTransform_GetOutputCurrentType((*node).object.transform, id as DWORD, old_output_types.offset(output as isize));
            if hr < 0 {
                break 'done;
            }
            output = output.wrapping_add(1);
        }

        hr = topology_node_get_type_handler((*node).node, input as DWORD, (FALSE) as BOOL, core::ptr::addr_of_mut!(handler));
        if hr >= 0 {
            hr = IMFMediaTypeHandler_SetCurrentMediaType(handler, media_type);
            if hr < 0 {
                WARN!("Failed to change note %p input %u media type\n", (*node).node, input);
            }
            IMFMediaTypeHandler_Release(handler);
        }

        output = 0;
        while hr >= 0 && output < (*node).u.transform.output_count {
            let stream: *mut transform_stream = (*node).u.transform.outputs.offset(output as isize);
            let id: UINT = transform_node_get_stream_id(node, (TRUE) as BOOL, output) as UINT;

            /* check if transform output type is still valid or if we need to update it as well */
            hr = IMFTransform_GetOutputCurrentType((*node).object.transform, id as DWORD, core::ptr::addr_of_mut!(new_output_type));
            if hr >= 0 {
                IMFMediaType_Release(new_output_type);
                output = output.wrapping_add(1);
                continue;
            }

            hr = transform_stream_update_output_type(node, stream, id, *old_output_types.offset(output as isize), core::ptr::addr_of_mut!(new_output_type));
            if hr >= 0 {
                hr = transform_stream_push_format_change(stream, new_output_type);
                if hr < 0 {
                    WARN!("Failed to queue format change event, hr %#lx\n", hr);
                }
                IMFMediaType_Release(new_output_type);
            }
            output = output.wrapping_add(1);
        }
    }

    output = 0;
    while output < (*node).u.transform.output_count {
        if !(*old_output_types.offset(output as isize)).is_null() {
            IMFMediaType_Release(*old_output_types.offset(output as isize));
        }
        output = output.wrapping_add(1);
    }
    free(old_output_types as *mut core::ffi::c_void);
    return hr;
}

unsafe extern "C" fn allocate_output_samples(
    session: *const media_session,
    node: *mut topo_node,
    buffers: *mut MFT_OUTPUT_DATA_BUFFER,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut i: UINT;

    i = 0 as UINT;
    while i < (*node).u.transform.output_count {
        let mut stream_info: MFT_OUTPUT_STREAM_INFO = MFT_OUTPUT_STREAM_INFO {
            dwFlags: 0 as DWORD,
            cbSize: 0,
            cbAlignment: 0,
        };

        (*buffers.offset(i as isize)).dwStreamID =
            transform_node_get_stream_id(node, TRUE as BOOL, i);

        hr = IMFTransform_GetOutputStreamInfo(
            (*node).object.transform,
            (*buffers.offset(i as isize)).dwStreamID,
            &raw mut stream_info,
        );
        if hr < 0 as HRESULT {
            return hr;
        }
        if (stream_info.dwFlags
            & (MFT_OUTPUT_STREAM_PROVIDES_SAMPLES as DWORD | MFT_OUTPUT_STREAM_CAN_PROVIDE_SAMPLES as DWORD))
            == 0
            && {
                hr = transform_get_external_output_sample(
                    session,
                    node,
                    i as DWORD,
                    &raw mut stream_info,
                    &raw mut (*buffers.offset(i as isize)).pSample,
                );
                hr < 0 as HRESULT
            }
        {
            return hr;
        }
        i = i.wrapping_add(1);
    }

    return S_OK as HRESULT;
}

unsafe extern "C" fn release_output_samples(node: *mut topo_node, buffers: *mut MFT_OUTPUT_DATA_BUFFER) {
    let mut i: UINT;
    i = 0 as UINT;
    while i < (*node).u.transform.output_count {
        if !(*buffers.offset(i as isize)).pSample.is_null() {
            IMFSample_Release((*buffers.offset(i as isize)).pSample);
        }
        if !(*buffers.offset(i as isize)).pEvents.is_null() {
            IMFCollection_Release((*buffers.offset(i as isize)).pEvents);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn transform_node_markin_need_more_input(session: *const media_session, node: *mut topo_node, buffers: *mut MFT_OUTPUT_DATA_BUFFER) -> BOOL {
    let mut need_more_input: BOOL = 0;
    let mut drop_sample: BOOL = 0;
    let mut have_duration: BOOL = 0;
    let mut have_time: BOOL = 0;
    let mut time: LONGLONG = 0;
    let mut duration: LONGLONG = 0;
    let mut hr: HRESULT = 0;
    let mut i: UINT = 0;

    if (*node).flags & TOPO_NODE_MARKIN_HERE == 0
        || (*session).presentation.start_position.vt as i32 != VT_I8 as i32
    {
        return (FALSE) as BOOL;
    }

    need_more_input = (TRUE as i32);

    i = 0 as UINT;
    while i < (*node).u.transform.output_count {
        let mut stream: *mut transform_stream = (*node).u.transform.outputs.offset(i as isize) as *mut transform_stream;

        have_time = (FALSE as i32);
        have_duration = have_time;
        drop_sample = have_duration;

        if !(*buffers.offset(i as isize)).pEvents.is_null() {
            need_more_input = (FALSE as i32);
        }

        if !((*buffers.offset(i as isize)).dwStatus & MFT_OUTPUT_DATA_BUFFER_NO_SAMPLE != 0) {
            if (*stream).samples_independent == 0 {
                need_more_input = (FALSE as i32);
            } else {
                hr = IMFSample_GetSampleTime((*buffers.offset(i as isize)).pSample, &mut time);
                if hr >= 0 {
                    have_time = (TRUE as i32);
                    hr = IMFSample_GetSampleDuration((*buffers.offset(i as isize)).pSample, &mut duration);
                    if hr >= 0 {
                        have_duration = (TRUE as i32);
                    } else {
                        WARN!("Failed to get sample duration, hr %#lx\n", hr);
                    }

                    if have_duration != 0
                        && time + duration <= (*session).presentation.start_position.__bindgen_anon_1.hVal.QuadPart
                    {
                        drop_sample = (TRUE as i32);
                    }
                } else {
                    WARN!("Failed to get sample time, hr %#lx\n", hr);
                }

                if have_time != 0
                    && drop_sample == 0
                    && time < (*session).presentation.start_position.__bindgen_anon_1.hVal.QuadPart
                {
                    let mut delta: LONGLONG = (*session).presentation.start_position.__bindgen_anon_1.hVal.QuadPart - time;

                    IMFSample_SetSampleTime(
                        (*buffers.offset(i as isize)).pSample,
                        (*session).presentation.start_position.__bindgen_anon_1.hVal.QuadPart,
                    );

                    if have_duration != 0 {
                        duration -= delta;
                        IMFSample_SetSampleDuration((*buffers.offset(i as isize)).pSample, duration);
                    }

                    if (*stream).raw_audio != 0
                        && (*stream).bytes_per_second != 0
                        && (*stream).block_alignment != 0
                    {
                        let mut buffer: *mut IMFMediaBuffer = core::ptr::null_mut::<IMFMediaBuffer>();
                        let mut delta_bytes: LONGLONG = 0;
                        let mut current_length: DWORD = 0;
                        let mut data: *mut BYTE = core::ptr::null_mut::<BYTE>();

                        delta_bytes = (*stream).bytes_per_second as LONGLONG * delta
                            / MFCLOCK_FREQUENCY_HNS as LONGLONG
                            / (*stream).block_alignment as LONGLONG
                            * (*stream).block_alignment as LONGLONG;

                        hr = IMFSample_GetTotalLength((*buffers.offset(i as isize)).pSample, &mut current_length);
                        if hr < 0 {
                            WARN!("Failed to get total length of sample, hr %#lx\n", hr);
                        } else if delta_bytes >= current_length as LONGLONG {
                            drop_sample = (TRUE as i32);
                        } else if IMFSample_ConvertToContiguousBuffer((*buffers.offset(i as isize)).pSample, &mut buffer) >= 0 {
                            if IMFMediaBuffer_Lock(
                                buffer,
                                &mut data,
                                core::ptr::null_mut::<DWORD>(),
                                core::ptr::null_mut::<DWORD>(),
                            ) >= 0
                            {
                                memmove(
                                    data as *mut core::ffi::c_void,
                                    data.offset(delta_bytes as isize) as *const core::ffi::c_void,
                                    (current_length as LONGLONG - delta_bytes) as usize,
                                );
                                IMFMediaBuffer_Unlock(buffer);
                                IMFMediaBuffer_SetCurrentLength(buffer, (current_length as LONGLONG - delta_bytes) as DWORD);
                            }
                            IMFMediaBuffer_Release(buffer);
                        }
                    }
                }

                if drop_sample != 0 {
                    IMFSample_Release((*buffers.offset(i as isize)).pSample);
                    (*buffers.offset(i as isize)).pSample = core::ptr::null_mut::<IMFSample>();
                    (*buffers.offset(i as isize)).dwStatus |= MFT_OUTPUT_DATA_BUFFER_NO_SAMPLE;
                } else {
                    need_more_input = (FALSE as i32);
                }
            }
        }

        i = i.wrapping_add(1);
    }

    return need_more_input;
}

unsafe extern "C" fn transform_node_pull_samples(session: *const media_session, node: *mut topo_node) -> HRESULT {
    let mut buffers: *mut MFT_OUTPUT_DATA_BUFFER = core::ptr::null_mut();
    let mut status: DWORD = 0;
    let mut hr: HRESULT = 0;
    let mut i: UINT = 0;

    buffers = calloc(
        ((*node).u.transform.output_count as size_t) as _,
        (core::mem::size_of::<MFT_OUTPUT_DATA_BUFFER>() as size_t) as _,
    ) as *mut MFT_OUTPUT_DATA_BUFFER;
    if buffers.is_null() {
        return E_OUTOFMEMORY;
    }

    'done: {
        hr = allocate_output_samples(session, node, buffers);
        if hr < 0 {
            break 'done;
        }

        status = 0;

        loop {
            hr = IMFTransform_ProcessOutput(
                (*node).object.transform,
                0,
                (*node).u.transform.output_count,
                buffers,
                core::ptr::addr_of_mut!(status),
            );
            if hr == MF_E_TRANSFORM_STREAM_CHANGE {
                hr = transform_node_format_changed(node, buffers);
                if hr >= 0 {
                    release_output_samples(node, buffers);

                    memset(
                        buffers as *mut ::core::ffi::c_void,
                        0,
                        ((*node).u.transform.output_count as size_t)
                            .wrapping_mul(core::mem::size_of::<MFT_OUTPUT_DATA_BUFFER>() as size_t),
                    );
                    hr = allocate_output_samples(session, node, buffers);
                    if hr < 0 {
                        break 'done;
                    }

                    hr = IMFTransform_ProcessOutput(
                        (*node).object.transform,
                        0,
                        (*node).u.transform.output_count,
                        buffers,
                        core::ptr::addr_of_mut!(status),
                    );
                }
            }
            if !(hr == S_OK
                && transform_node_markin_need_more_input(session, node, buffers) != 0)
            {
                break;
            }
        }

        /* Collect returned samples for all streams. */
        i = 0;
        while i < (*node).u.transform.output_count {
            let mut stream: *mut transform_stream =
                (*node).u.transform.outputs.offset(i as isize) as *mut transform_stream;

            'body: {
                if !(*buffers.offset(i as isize)).pEvents.is_null() {
                    hr = transform_stream_push_events(stream, (*buffers.offset(i as isize)).pEvents);
                    if hr < 0 {
                        WARN!("Failed to push transform events, hr %#lx\n", hr);
                    }
                }
                if ((*buffers.offset(i as isize)).dwStatus & MFT_OUTPUT_DATA_BUFFER_NO_SAMPLE) != 0 {
                    break 'body;
                }

                if hr >= 0 {
                    if !(*session).quality_manager.is_null() {
                        IMFQualityManager_NotifyProcessOutput(
                            (*session).quality_manager,
                            (*node).node,
                            i as LONG,
                            (*buffers.offset(i as isize)).pSample,
                        );
                    }
                    hr = transform_stream_push_sample(stream, (*buffers.offset(i as isize)).pSample);
                    if hr < 0 {
                        WARN!("Failed to queue output sample, hr %#lx\n", hr);
                    }
                }
            }
            i += 1;
        }
    }

    release_output_samples(node, buffers);
    free(buffers as *mut ::core::ffi::c_void);

    return hr;
}

unsafe extern "C" fn transform_node_is_drained(mut topo_node: *mut topo_node) -> BOOL {
    let mut i: UINT = 0;
    while i < (*topo_node).u.transform.input_count {
        let mut stream: *mut transform_stream =
            (*topo_node).u.transform.inputs.offset(i as isize) as *mut transform_stream;
        if !((*stream).draining != 0) {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn transform_node_has_requests(topo_node: *mut topo_node) -> BOOL {
    let mut i: UINT;

    i = 0;
    while i < (*topo_node).u.transform.output_count {
        if (*(*topo_node).u.transform.outputs.add(i as usize)).requests != 0 {
            return (TRUE) as BOOL;
        }
        i = i.wrapping_add(1);
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn transform_node_push_sample(session: *const media_session, topo_node: *mut topo_node, input: UINT, sample: *mut IMFSample) -> HRESULT {
    let mut stream: *mut transform_stream = (*topo_node).u.transform.inputs.offset(input as isize) as *mut transform_stream;
    let mut id: UINT = transform_node_get_stream_id(topo_node, (FALSE) as BOOL, input as ::core::ffi::c_uint) as UINT;
    let mut transform: *mut IMFTransform = (*topo_node).object.transform;
    let mut hr: HRESULT;

    if !sample.is_null() {
        hr = IMFTransform_ProcessInput(transform, id as DWORD, sample, 0 as DWORD);
        if hr == MF_E_NOTACCEPTING {
            hr = transform_stream_push_sample(stream, sample);
        }
    } else {
        hr = IMFTransform_ProcessMessage(transform, MFT_MESSAGE_COMMAND_DRAIN, id as ULONG_PTR);
        if hr < 0 {
            WARN!("Drain command failed for transform, hr %#lx.\n", hr);
        } else {
            (*stream).draining = TRUE as BOOL;
        }
    }

    return hr;
}

unsafe extern "C" fn transform_node_handle_format_change(
    session: *mut media_session,
    topo_node: *mut topo_node,
    input: UINT,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let stream: *mut transform_stream = (*topo_node).u.transform.inputs.add(input as usize);
    let id: UINT = transform_node_get_stream_id(topo_node, (FALSE) as BOOL, input) as UINT;
    let transform: *mut IMFTransform = (*topo_node).object.transform;
    let mut support_dynamic_format_change: UINT32 = 0;
    let mut attributes: *mut IMFAttributes = ::core::ptr::null_mut::<IMFAttributes>();
    let mut hr: HRESULT = 0;

    if IMFTransform_GetAttributes(transform, &raw mut attributes) >= 0 {
        if IMFAttributes_GetUINT32(
            attributes,
            &raw const MFT_SUPPORT_DYNAMIC_FORMAT_CHANGE,
            &raw mut support_dynamic_format_change,
        ) < 0
        {
            support_dynamic_format_change = 0;
        }
        IMFAttributes_Release(attributes);
    }

    /* all async MFTs must support dynamic format change, but async is checked here in case a bugged MFT doesn't set the flag */
    if support_dynamic_format_change == 0 && (*topo_node).u.transform.async_ == 0 {
        hr = IMFTransform_ProcessMessage(transform, MFT_MESSAGE_COMMAND_DRAIN, id as ULONG_PTR);
        if hr >= 0 {
            while {
                hr = transform_node_pull_samples(session, topo_node);
                hr >= 0
            } {
                transform_node_deliver_samples(session, topo_node);
            }
        }

        if hr != MF_E_TRANSFORM_NEED_MORE_INPUT {
            /* transform isn't fully drained, put the event in the stream input queue to try again later */
            if transform_stream_push_format_change(stream, media_type) < 0 {
                WARN!("Failed to queue input format change event\n");
            }
            return hr;
        }
    }

    return transform_stream_update_input_type(topo_node, input, media_type);
}

unsafe extern "C" fn session_handle_format_change(
    session: *mut media_session,
    topo_node: *mut topo_node,
    input: UINT,
    media_type: *mut IMFMediaType,
) -> HRESULT {
    let mut handler: *mut IMFMediaTypeHandler = core::ptr::null_mut();
    let mut hr: HRESULT;

    match (*topo_node).type_ as u32 {
        MF_TOPOLOGY_OUTPUT_NODE => {
            /* For H.264, the sink's input media type must be set to the aligned frame size
             * before MESessionStarted is sent. The decoder sends MF_E_TRANSFORM_STREAM_CHANGE
             * to trigger a refresh of its output type once the aligned frame size is known.
             * This occurs before any call to ProcessOutput() can succeed on the transform.
             * Satisfactory is known to use the sink frame size. */
            hr = topology_node_get_type_handler(
                (*topo_node).node,
                0,
                (FALSE) as BOOL,
                core::ptr::addr_of_mut!(handler),
            );
            if hr < 0 {
                WARN!("Failed to get type handler, hr %#lx.\n", hr);
            } else {
                hr = IMFMediaTypeHandler_SetCurrentMediaType(handler, media_type);
                if hr < 0 {
                    WARN!("Failed to set type, hr %#lx.\n", hr);
                }
            }

            if hr >= 0 && !(*topo_node).u.sink.allocator.is_null() {
                hr = IMFVideoSampleAllocator_UninitializeSampleAllocator((*topo_node).u.sink.allocator);
                if hr >= 0 {
                    hr = IMFVideoSampleAllocator_InitializeSampleAllocator(
                        (*topo_node).u.sink.allocator,
                        4,
                        media_type,
                    );
                }
            }
            return hr;
        }
        MF_TOPOLOGY_TRANSFORM_NODE => {
            return transform_node_handle_format_change(session, topo_node, input, media_type);
        }
        _ => {
            FIXME!("Unhandled downstream node type %d.\n", (*topo_node).type_ as u32);
            return E_NOTIMPL;
        }
    }
}

unsafe extern "C" fn transform_stream_handle_event(
    session: *mut media_session,
    stream: *mut transform_stream,
    topo_node: *mut topo_node,
    input: ::core::ffi::c_uint,
    event: *mut IMFMediaEvent,
) -> HRESULT {
    let mut type_0: MediaEventType = 0;
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    hr = IMFMediaEvent_GetType(event, core::ptr::addr_of_mut!(type_0));
    if hr < 0 {
        return hr;
    }
    PropVariantInit!(core::ptr::addr_of_mut!(value));
    match type_0 {
        213 /* MEMediaSample */ => {
            hr = IMFMediaEvent_GetValue(event, core::ptr::addr_of_mut!(value));
            if hr >= 0 {
                session_deliver_sample_to_node(
                    session,
                    topo_node,
                    input,
                    value.__bindgen_anon_1.punkVal as *mut IMFSample,
                );
            }
        }
        216 /* MEStreamFormatChanged */ => {
            hr = IMFMediaEvent_GetValue(event, core::ptr::addr_of_mut!(value));
            if hr >= 0 {
                hr = session_handle_format_change(
                    session,
                    topo_node,
                    input as UINT,
                    value.__bindgen_anon_1.punkVal as *mut IMFMediaType,
                );
            }
        }
        _ => {
            ERR!("Unexpected event type %lu\n", type_0);
            hr = E_NOTIMPL;
        }
    }
    PropVariantClear(core::ptr::addr_of_mut!(value));
    return hr;
}

unsafe extern "C" fn transform_node_deliver_samples(
    session: *mut media_session,
    topo_node: *mut topo_node,
) {
    let drained: BOOL = transform_node_is_drained(topo_node);
    let mut up_node: *mut topo_node = core::ptr::null_mut();
    let mut down_node: *mut topo_node = core::ptr::null_mut();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();
    let mut output: DWORD = 0;
    let mut input: DWORD = 0;
    let mut hr: HRESULT = S_OK;

    /* Push down all available output. */
    output = 0;
    while (hr >= 0) && output < (*topo_node).u.transform.output_count {
        let stream: *mut transform_stream =
            (*topo_node).u.transform.outputs.offset(output as isize);

        down_node = session_get_topo_node_output(
            session,
            topo_node,
            output,
            core::ptr::addr_of_mut!(input),
        );
        if down_node.is_null() {
            WARN!("Failed to node %p/%lu output\n", topo_node, output);
        } else {
            while (*stream).requests != 0 {
                let mut type_0: MediaEventType = 0;

                hr = transform_stream_pop_event(stream, core::ptr::addr_of_mut!(event));
                if hr < 0 {
                    if (*topo_node).u.transform.async_ != 0 {
                        break;
                    }
                    /* try getting more samples by calling IMFTransform_ProcessOutput */
                    hr = transform_node_pull_samples(session, topo_node);
                    if hr < 0 {
                        break;
                    }
                    hr = transform_stream_pop_event(stream, core::ptr::addr_of_mut!(event));
                    if hr < 0 {
                        break;
                    }
                }

                hr = transform_stream_handle_event(session, stream, down_node, input, event);
                if hr < 0 {
                    ERR!("Failed to handle stream event, hr %#lx\n", hr);
                } else if IMFMediaEvent_GetType(event, core::ptr::addr_of_mut!(type_0))
                    >= 0
                    && type_0 == MEMediaSample as MediaEventType
                {
                    (*stream).requests = (*stream).requests.wrapping_sub(1);
                }
                IMFMediaEvent_Release(event);
            }

            while (*stream).requests != 0 && drained != 0 {
                session_deliver_sample_to_node(session, down_node, input, core::ptr::null_mut());
                (*stream).requests = (*stream).requests.wrapping_sub(1);
            }
        }
        output = output.wrapping_add(1);
    }

    if (*topo_node).u.transform.async_ == 0
        && hr == MF_E_TRANSFORM_NEED_MORE_INPUT
        && transform_node_has_requests(topo_node) != 0
    {
        let stream: *mut transform_stream;

        input = (*topo_node).u.transform.next_input % (*topo_node).u.transform.input_count;
        (*topo_node).u.transform.next_input = (*topo_node).u.transform.next_input.wrapping_add(1);
        stream = (*topo_node).u.transform.inputs.offset(input as isize);

        if transform_stream_pop_event(stream, core::ptr::addr_of_mut!(event))
            >= 0
        {
            hr = transform_stream_handle_event(session, stream, topo_node, input, event);
            if hr < 0 {
                ERR!("Failed to handle stream event, hr %#lx\n", hr);
            }
            IMFMediaEvent_Release(event);
        } else {
            up_node = session_get_topo_node_input(
                session,
                topo_node,
                input,
                core::ptr::addr_of_mut!(output),
            );
            if up_node.is_null() {
                WARN!("Failed to node %p/%lu input\n", topo_node, input);
            } else {
                hr = session_request_sample_from_node(session, up_node, output);
                if hr < 0 {
                    WARN!(
                        "Failed to request sample from upstream node %p/%lu, hr %#lx\n",
                        up_node,
                        output,
                        hr
                    );
                }
            }
        }
    }
}

unsafe extern "C" fn session_deliver_sample_to_node(
    mut session: *mut media_session,
    mut topo_node: *mut topo_node,
    mut input: ::core::ffi::c_uint,
    mut sample: *mut IMFSample,
) {
    let mut hr: HRESULT = 0;
    if !(*session).quality_manager.is_null() {
        IMFQualityManager_NotifyProcessInput(
            (*session).quality_manager,
            (*topo_node).node,
            input as LONG,
            sample,
        );
    }
    match (*topo_node).type_ as ::core::ffi::c_uint {
        MF_TOPOLOGY_OUTPUT_NODE => {
            if (*topo_node).u.sink.requests != 0 {
                if !sample.is_null() {
                    hr = IMFStreamSink_ProcessSample((*topo_node).object.sink_stream, sample);
                    if hr >= 0 {
                        (*topo_node).u.sink.requests = (*topo_node).u.sink.requests.wrapping_sub(1);
                    } else {
                        WARN!("Stream sink failed to process sample, hr %#lx.\n", hr);
                        IMFMediaEventQueue_QueueEventParamVar(
                            (*session).event_queue,
                            MEError as ::core::ffi::c_int as MediaEventType,
                            &raw const GUID_NULL,
                            hr,
                            ::core::ptr::null::<PROPVARIANT>(),
                        );
                    }
                } else {
                    hr = IMFStreamSink_PlaceMarker(
                        (*topo_node).object.sink_stream,
                        MFSTREAMSINK_MARKER_ENDOFSEGMENT,
                        ::core::ptr::null::<PROPVARIANT>(),
                        ::core::ptr::null::<PROPVARIANT>(),
                    );
                    if hr < 0 {
                        WARN!("Failed to place sink marker, hr %#lx.\n", hr);
                    }
                }
            }
        }
        MF_TOPOLOGY_TRANSFORM_NODE => {
            hr = transform_node_push_sample(session, topo_node, input as UINT, sample);
            if hr < 0 {
                WARN!("Failed to push or queue sample to transform, hr %#lx\n", hr);
            }
            if (*topo_node).u.transform.async_ == 0 {
                transform_node_pull_samples(session, topo_node);
                transform_node_deliver_samples(session, topo_node);
            }
        }
        MF_TOPOLOGY_TEE_NODE => {
            FIXME!("Unhandled downstream node type %d.\n", (*topo_node).type_ as ::core::ffi::c_uint);
        }
        _ => {}
    };
}

unsafe extern "C" fn session_deliver_pending_samples(session: *mut media_session, topo_node: *mut topo_node) {
    match (*topo_node).type_ {
        MF_TOPOLOGY_TRANSFORM_NODE => {
            transform_node_pull_samples(session, topo_node);
            transform_node_deliver_samples(session, topo_node);
        }
        _ => {
            FIXME!("Unexpected node type %u.\n", (*topo_node).type_);
        }
    }
}

unsafe extern "C" fn session_request_sample_from_node(session: *mut media_session, topo_node: *mut topo_node, output: DWORD) -> HRESULT {
    let mut down_node: *mut topo_node = core::ptr::null_mut();
    let mut hr: HRESULT = S_OK;
    let mut input: DWORD = 0;

    match (*topo_node).type_ as ::core::ffi::c_uint {
        MF_TOPOLOGY_SOURCESTREAM_NODE => {
            if ({
                hr = IMFMediaStream_RequestSample((*topo_node).object.source_stream, core::ptr::null_mut());
                hr
            } < 0)
            {
                WARN!("Sample request failed, hr %#lx.\n", hr);
            }
        }
        MF_TOPOLOGY_TRANSFORM_NODE => {
            let stream: *mut transform_stream = (*topo_node).u.transform.outputs.offset(output as isize);
            let mut event: *mut IMFMediaEvent = core::ptr::null_mut();

            down_node = session_get_topo_node_output(session, topo_node, output, core::ptr::addr_of_mut!(input));
            if down_node.is_null() {
                WARN!("Failed to node %p/%lu output\n", topo_node, output);
            } else if (transform_stream_pop_event(stream, core::ptr::addr_of_mut!(event))) >= 0 {
                hr = transform_stream_handle_event(session, stream, down_node, input, event);
                if hr < 0 {
                    ERR!("Failed to handle stream event, hr %#lx\n", hr);
                }
                IMFMediaEvent_Release(event);
            } else if transform_node_has_requests(topo_node) != 0 || (*topo_node).u.transform.async_ != 0 {
                /* there's already requests pending, just increase the counter */
                (*stream).requests = (*stream).requests.wrapping_add(1);
            } else {
                (*stream).requests = (*stream).requests.wrapping_add(1);
                transform_node_deliver_samples(session, topo_node);
            }
        }
        MF_TOPOLOGY_TEE_NODE => {
            FIXME!("Unhandled upstream node type %d.\n", (*topo_node).type_ as ::core::ffi::c_uint);
            hr = E_UNEXPECTED;
        }
        _ => {
            hr = E_UNEXPECTED;
        }
    }

    return hr;
}

unsafe extern "C" fn session_request_sample(
    mut session: *mut media_session,
    mut sink_stream: *mut IMFStreamSink,
) {
    let mut sink_node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut up_node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut output: DWORD = 0;
    node = ((*session).presentation.nodes.next as *mut u8).offset(
        -(core::mem::offset_of!(topo_node, entry) as isize),
    ) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry) != core::ptr::addr_of_mut!((*session).presentation.nodes) {
        if (*node).type_ as u32 == MF_TOPOLOGY_OUTPUT_NODE as u32
            && (*node).object.sink_stream == sink_stream
        {
            sink_node = node;
            break;
        } else {
            node = ((*node).entry.next as *mut u8).offset(
                -(core::mem::offset_of!(topo_node, entry) as isize),
            ) as *mut topo_node;
        }
    }
    if sink_node.is_null() {
        return;
    }
    up_node = session_get_topo_node_input(
        session,
        sink_node,
        0,
        core::ptr::addr_of_mut!(output),
    );
    if up_node.is_null() {
        WARN!("Failed to node %p/%u input\n", sink_node, 0);
        return;
    }
    (*sink_node).u.sink.requests = (*sink_node).u.sink.requests.wrapping_add(1);
    if session_request_sample_from_node(session, up_node, output) < 0 {
        (*sink_node).u.sink.requests = (*sink_node).u.sink.requests.wrapping_sub(1);
    }
}

unsafe extern "C" fn session_deliver_sample(
    mut session: *mut media_session,
    mut stream: *mut IMFMediaStream,
    mut value: *const PROPVARIANT,
) {
    let mut source_node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    let mut input: DWORD = 0;

    if !value.is_null()
        && ((*value).vt as core::ffi::c_int != VT_UNKNOWN as core::ffi::c_int
            || (*value).__bindgen_anon_1.punkVal.is_null())
    {
        WARN!("Unexpected value type %d.\n", (*value).vt as core::ffi::c_int);
        return;
    }

    // LIST_FOR_EACH_ENTRY(node, &session->presentation.nodes, struct topo_node, entry)
    node = ((*session).presentation.nodes.next as *mut u8)
        .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of_mut!((*session).presentation.nodes)
    {
        if (*node).type_ as core::ffi::c_uint
            == MF_TOPOLOGY_SOURCESTREAM_NODE as core::ffi::c_uint
            && (*node).object.source_stream == stream
        {
            source_node = node;
            break;
        } else {
            node = ((*node).entry.next as *mut u8)
                .sub(core::mem::offset_of!(topo_node, entry)) as *mut topo_node;
        }
    }

    if source_node.is_null() {
        return;
    }

    if value.is_null() {
        (*source_node).flags |= TOPO_NODE_END_OF_STREAM as core::ffi::c_uint;
    }

    node = session_get_topo_node_output(
        session,
        source_node,
        0,
        core::ptr::addr_of_mut!(input),
    );
    if node.is_null() {
        WARN!(
            "Failed to node %p/%u output.\n",
            source_node,
            0 as core::ffi::c_int
        );
        return;
    }

    session_deliver_sample_to_node(
        session,
        node,
        input as core::ffi::c_uint,
        if !value.is_null() {
            (*value).__bindgen_anon_1.punkVal as *mut IMFSample
        } else {
            core::ptr::null_mut::<IMFSample>()
        },
    );
}

unsafe extern "C" fn session_sink_invalidated(
    mut session: *mut media_session,
    mut event: *mut IMFMediaEvent,
    mut sink: *mut IMFStreamSink,
) {
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    let mut sink_node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    let mut hr: HRESULT = 0;
    node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut topo_node;
    while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
        if (*node).type_ as ::core::ffi::c_uint
            == MF_TOPOLOGY_OUTPUT_NODE as ::core::ffi::c_uint
            && (*node).object.sink_stream == sink
        {
            sink_node = node;
            break;
        } else {
            node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut topo_node;
        }
    }
    if sink_node.is_null() {
        return;
    }
    if event.is_null() {
        hr = MFCreateMediaEvent(
            MESinkInvalidated as ::core::ffi::c_int as MediaEventType,
            &raw const GUID_NULL,
            S_OK as HRESULT,
            ::core::ptr::null::<PROPVARIANT>(),
            &raw mut event,
        );
        if hr < 0 as HRESULT {
            WARN!("Failed to create event, hr %#lx.\n", hr);
        }
    }
    if event.is_null() {
        return;
    }
    IMFMediaEvent_SetUINT64(
        event,
        &raw const MF_EVENT_OUTPUT_NODE,
        (*sink_node).node_id as UINT64,
    );
    IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
    session_set_topo_status(
        session,
        S_OK as HRESULT,
        MF_TOPOSTATUS_ENDED,
    );
}

unsafe extern "C" fn session_nodes_is_mask_set(
    mut session: *mut media_session,
    mut node_type: MF_TOPOLOGY_TYPE,
    mut flags: ::core::ffi::c_uint,
) -> BOOL {
    let mut source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    if node_type as ::core::ffi::c_uint == MF_TOPOLOGY_MAX as ::core::ffi::c_uint {
        source = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut media_source;
        while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
            if (*source).flags & flags != flags {
                return (FALSE) as BOOL;
            }
            source = ((*source).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<media_source>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut media_source;
        }
    } else {
        node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
            -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry as ::core::ffi::c_ulonglong
                as isize),
        ) as *mut topo_node;
        while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
            if (*node).type_ as ::core::ffi::c_uint == node_type as ::core::ffi::c_uint
                && (*node).flags & flags != flags
            {
                return (FALSE) as BOOL;
            }
            node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<topo_node>()).entry as ::core::ffi::c_ulonglong
                    as isize),
            ) as *mut topo_node;
        }
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn session_nodes_unset_mask(
    mut session: *mut media_session,
    mut node_type: MF_TOPOLOGY_TYPE,
    mut flags: ::core::ffi::c_uint,
) {
    let mut source: *mut media_source = ::core::ptr::null_mut::<media_source>();
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    if node_type as ::core::ffi::c_uint == MF_TOPOLOGY_MAX as ::core::ffi::c_uint {
        source = ((*session).presentation.sources.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(media_source, entry) as isize),
        ) as *mut media_source;
        while &raw mut (*source).entry != &raw mut (*session).presentation.sources {
            (*source).flags &= !flags;
            source = ((*source).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(media_source, entry) as isize),
            ) as *mut media_source;
        }
    } else {
        node = ((*session).presentation.nodes.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(topo_node, entry) as isize),
        ) as *mut topo_node;
        while &raw mut (*node).entry != &raw mut (*session).presentation.nodes {
            if (*node).type_ as ::core::ffi::c_uint == node_type as ::core::ffi::c_uint {
                (*node).flags &= !flags;
            }
            node = ((*node).entry.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(topo_node, entry) as isize),
            ) as *mut topo_node;
        }
    }
}

unsafe extern "C" fn session_raise_end_of_presentation(mut session: *mut media_session) {
    if session_nodes_is_mask_set(
        session,
        MF_TOPOLOGY_SOURCESTREAM_NODE,
        TOPO_NODE_END_OF_STREAM as ::core::ffi::c_int as ::core::ffi::c_uint,
    ) == 0
    {
        return;
    }
    if (*session).command_state as ::core::ffi::c_uint
        == COMMAND_STATE_COMPLETE as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if session_nodes_is_mask_set(
            session,
            MF_TOPOLOGY_MAX,
            SOURCE_FLAG_END_OF_PRESENTATION as ::core::ffi::c_int as ::core::ffi::c_uint,
        ) != 0
        {
            (*session).presentation.flags |=
                SESSION_FLAG_PRESENTATION_ENDING as ::core::ffi::c_int as ::core::ffi::c_uint;
            IMFMediaEventQueue_QueueEventParamVar(
                (*session).event_queue,
                MEEndOfPresentation as ::core::ffi::c_int as MediaEventType,
                core::ptr::addr_of!(GUID_NULL),
                S_OK,
                ::core::ptr::null::<PROPVARIANT>(),
            );
        }
    }
}

unsafe extern "C" fn session_handle_end_of_stream(session: *mut media_session, stream: *mut IMFMediaStream) {
    let mut node: *mut topo_node = core::ptr::null_mut();

    node = session_get_node_object(session, stream as *mut IUnknown, MF_TOPOLOGY_SOURCESTREAM_NODE);
    if node.is_null() || ((*node).flags & TOPO_NODE_END_OF_STREAM as u32) != 0 {
        return;
    }

    session_deliver_sample(session, stream, core::ptr::null::<PROPVARIANT>());

    session_raise_end_of_presentation(session);
}

unsafe extern "C" fn session_handle_end_of_presentation(
    session: *mut media_session,
    object: *mut IMFMediaSource,
) {
    let mut source: *mut media_source = ((*session).presentation.sources.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_source, entry) as isize))
        as *mut media_source;
    while core::ptr::addr_of_mut!((*source).entry)
        != core::ptr::addr_of_mut!((*session).presentation.sources)
    {
        if (*source).__bindgen_anon_1.source == object {
            if ((*source).flags & (SOURCE_FLAG_END_OF_PRESENTATION as u32)) == 0 {
                (*source).flags |= SOURCE_FLAG_END_OF_PRESENTATION as u32;
                session_raise_end_of_presentation(session);
            }
            break;
        } else {
            source = ((*source).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(media_source, entry) as isize))
                as *mut media_source;
        }
    }
}

unsafe extern "C" fn session_sink_stream_marker(
    mut session: *mut media_session,
    mut stream_sink: *mut IMFStreamSink,
) {
    let mut node: *mut topo_node = ::core::ptr::null_mut::<topo_node>();
    let mut hr: HRESULT = 0;
    node = session_get_node_object(
        session,
        stream_sink as *mut IUnknown,
        MF_TOPOLOGY_OUTPUT_NODE as MF_TOPOLOGY_TYPE,
    );
    if node.is_null()
        || (*node).flags & TOPO_NODE_END_OF_STREAM as ::core::ffi::c_uint != 0
    {
        return;
    }
    (*node).flags |= TOPO_NODE_END_OF_STREAM as ::core::ffi::c_uint;
    if (*session).presentation.flags
        & SESSION_FLAG_PRESENTATION_ENDING as ::core::ffi::c_uint
        != 0
        && session_nodes_is_mask_set(
            session,
            MF_TOPOLOGY_OUTPUT_NODE as MF_TOPOLOGY_TYPE,
            TOPO_NODE_END_OF_STREAM as ::core::ffi::c_uint,
        ) != 0
    {
        session_set_topo_status(
            session,
            S_OK as HRESULT,
            MF_TOPOSTATUS_ENDED as MF_TOPOSTATUS,
        );
        session_set_caps(
            session,
            (*session).caps & !(MFSESSIONCAP_PAUSE as DWORD),
        );

        IMFPresentationClock_GetTime(
            (*session).clock,
            core::ptr::addr_of_mut!((*session).presentation.clock_stop_time),
        );
        hr = IMFPresentationClock_Stop((*session).clock);
        if hr >= 0 as HRESULT {
            (*session).command_state = COMMAND_STATE_STOPPING_SINKS as command_state;
        } else {
            session_set_stopped(
                session,
                MESessionEnded as ::core::ffi::c_int as MediaEventType,
                hr,
            );
        }
    }
}

unsafe extern "C" fn session_sink_stream_scrub_complete(
    mut session: *mut media_session,
    mut stream_sink: *mut IMFStreamSink,
) {
    let mut node: *mut topo_node;

    node = session_get_node_object(
        session,
        stream_sink as *mut IUnknown,
        MF_TOPOLOGY_OUTPUT_NODE,
    );
    if node.is_null()
        || ((*node).flags & (TOPO_NODE_SCRUB_SAMPLE_COMPLETE as u32)) != 0
    {
        return;
    }

    (*node).flags |= TOPO_NODE_SCRUB_SAMPLE_COMPLETE as u32;

    /* Scrubbing event is not limited to the started state transition, or even the started state.
       Events are processed and forwarded at any point after transition from initial idle state. */
    if ((*session).presentation.flags & (SESSION_FLAG_SOURCES_SUBSCRIBED as u32)) != 0
        && session_nodes_is_mask_set(
            session,
            MF_TOPOLOGY_OUTPUT_NODE,
            TOPO_NODE_SCRUB_SAMPLE_COMPLETE as u32,
        ) != 0
    {
        IMFMediaEventQueue_QueueEventParamVar(
            (*session).event_queue,
            MESessionScrubSampleComplete as MediaEventType,
            core::ptr::addr_of!(GUID_NULL),
            S_OK,
            core::ptr::null::<PROPVARIANT>(),
        );
        session_nodes_unset_mask(
            session,
            MF_TOPOLOGY_OUTPUT_NODE,
            TOPO_NODE_SCRUB_SAMPLE_COMPLETE as u32,
        );
    }
}

unsafe extern "C" fn session_get_async_transform_node(
    session: *const media_session,
    event_source: *mut IMFMediaEventGenerator,
) -> *mut topo_node {
    let mut node: *mut topo_node = core::ptr::null_mut::<topo_node>();
    node = ((*session).presentation.nodes.next as *mut u8)
        .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    while core::ptr::addr_of_mut!((*node).entry)
        != core::ptr::addr_of!((*session).presentation.nodes) as *mut list
    {
        if (*node).type_ as u32 == MF_TOPOLOGY_TRANSFORM_NODE as u32
            && (*node).u.transform.event_source == event_source
        {
            return node;
        }
        node = ((*node).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(topo_node, entry) as isize)) as *mut topo_node;
    }
    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_FIXME as i32))) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            (__dbcl) as u32,
            __dbch,
            b"dll/win32/mf/session.c\0"
                as *const u8 as *const i8,
            b"session_get_async_transform_node\0" as *const u8 as *const i8,
            4442i32,
            b"Failed to get transform.\n\0" as *const u8 as *const i8,
        );
    }
    core::ptr::null_mut::<topo_node>()
}

unsafe extern "system" fn session_events_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let mut session: *mut media_session = impl_from_events_callback_IMFAsyncCallback(iface);
    let mut event_source: *mut IMFMediaEventGenerator = core::ptr::null_mut();
    let mut event: *mut IMFMediaEvent = core::ptr::null_mut();
    let mut event_type: MediaEventType = 0;
    let mut object: *mut IUnknown = core::ptr::null_mut();
    let mut source: *mut IMFMediaSource = core::ptr::null_mut();
    let mut stream: *mut IMFMediaStream = core::ptr::null_mut();
    let mut value: PROPVARIANT = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    hr = IMFAsyncResult_GetState(result, &raw mut event_source as *mut *mut IUnknown);
    if hr < 0 {
        return hr;
    }

    hr = IMFMediaEventGenerator_EndGetEvent(event_source, result, &raw mut event);
    if hr < 0 {
        if hr == MF_E_SHUTDOWN
            && !session_get_media_source(session, event_source as *mut IMFMediaSource).is_null()
        {
            session_handle_source_shutdown(session);
        } else {
            WARN!("Failed to get event from %p, hr %#lx.\n", event_source, hr);
            IMFMediaEventQueue_QueueEventParamVar(
                (*session).event_queue,
                MEError as MediaEventType,
                &raw const GUID_NULL,
                hr,
                core::ptr::null(),
            );
        }
        IMFMediaEventGenerator_Release(event_source);
        return hr;
    }

    'failed: {
        hr = IMFMediaEvent_GetType(event, &raw mut event_type);
        if hr < 0 {
            WARN!("Failed to get event type, hr %#lx.\n", hr);
            break 'failed;
        }

        value.vt = VT_EMPTY;
        hr = IMFMediaEvent_GetValue(event, &raw mut value);
        if hr < 0 {
            WARN!("Failed to get event value, hr %#lx.\n", hr);
            break 'failed;
        }

        'sw: {
            match event_type {
                203 | 204 => {
                    FIXME!("Source/stream seeking, semi-stub!\n");
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_set_source_object_state(session, event_source as *mut IUnknown, event_type);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                201 | 209 | 207 | 202 | 210 | 208 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_set_source_object_state(session, event_source as *mut IUnknown, event_type);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                217 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_complete_rate_change(session);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                122 | 123 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    if !session_get_media_source(session, event_source as *mut IMFMediaSource).is_null() {
                        if event_type == MEBufferingStarted as MediaEventType {
                            IMFPresentationClock_Pause((*session).clock);
                        } else {
                            IMFPresentationClock_Start((*session).clock, (PRESENTATION_CURRENT_POSITION) as LONGLONG);
                        }
                        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
                    }
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                126 | 127 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    if !session_get_media_source(session, event_source as *mut IMFMediaSource).is_null() {
                        IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
                    }
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                2 | 128 | 309 => {
                    IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
                }
                205 => {
                    stream = value.__bindgen_anon_1.punkVal as *mut IMFMediaStream;
                    if value.vt != VT_UNKNOWN || stream.is_null() {
                        WARN!("Unexpected event value.\n");
                        break 'sw;
                    }

                    hr = IMFMediaStream_GetMediaSource(stream, &raw mut source);
                    if hr < 0 {
                        break 'sw;
                    }

                    EnterCriticalSection(&raw mut (*session).cs);
                    hr = session_add_media_stream(session, source, stream);
                    if hr >= 0 {
                        hr = IMFMediaStream_BeginGetEvent(
                            stream,
                            &raw mut (*session).events_callback,
                            stream as *mut IUnknown,
                        );
                    }
                    LeaveCriticalSection(&raw mut (*session).cs);

                    IMFMediaSource_Release(source);
                }
                301 | 303 | 302 | 307 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_set_sink_stream_state(session, event_source as *mut IMFStreamSink, event_type);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                306 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_sink_stream_marker(session, event_source as *mut IMFStreamSink);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                305 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_request_sample(session, event_source as *mut IMFStreamSink);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                308 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_sink_stream_scrub_complete(session, event_source as *mut IMFStreamSink);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                213 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_deliver_sample(session, event_source as *mut IMFMediaStream, &raw mut value);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                212 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_handle_end_of_stream(session, event_source as *mut IMFMediaStream);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                211 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_handle_end_of_presentation(session, event_source as *mut IMFMediaSource);
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                317 | 318 | 313 | 314 => {
                    IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
                }
                315 | 320 | 321 | 319 | 316 => {
                    IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
                    // fallthrough to MESinkInvalidated
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_sink_invalidated(
                        session,
                        if event_type == MESinkInvalidated as MediaEventType {
                            event
                        } else {
                            core::ptr::null_mut()
                        },
                        event_source as *mut IMFStreamSink,
                    );
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                312 => {
                    EnterCriticalSection(&raw mut (*session).cs);
                    session_sink_invalidated(
                        session,
                        if event_type == MESinkInvalidated as MediaEventType {
                            event
                        } else {
                            core::ptr::null_mut()
                        },
                        event_source as *mut IMFStreamSink,
                    );
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                311 => {
                    if !(*session).quality_manager.is_null() {
                        if IMFMediaEventGenerator_QueryInterface(
                            event_source,
                            &raw const IID_IMFStreamSink,
                            &raw mut object as *mut *mut core::ffi::c_void,
                        ) < 0
                        {
                            IMFMediaEventGenerator_QueryInterface(
                                event_source,
                                &raw const IID_IMFTransform,
                                &raw mut object as *mut *mut core::ffi::c_void,
                            );
                        }
                        if !object.is_null() {
                            IMFQualityManager_NotifyQualityEvent((*session).quality_manager, object, event);
                            IUnknown_Release(object);
                        }
                    }
                }
                601 => {
                    let mut up_node: *mut topo_node = core::ptr::null_mut();
                    let mut topo_node: *mut topo_node = session_get_async_transform_node(session, event_source);
                    let mut stream: *mut transform_stream = core::ptr::null_mut();
                    let mut stream_event: *mut IMFMediaEvent = core::ptr::null_mut();
                    let mut input: UINT32 = 0;
                    let mut is_sample: BOOL = 0;
                    let mut output: DWORD = 0;

                    if topo_node.is_null() {
                        break 'sw;
                    }

                    hr = IMFMediaEvent_GetUINT32(event, &raw const MF_EVENT_MFT_INPUT_STREAM_ID, &raw mut input);
                    if hr < 0 {
                        WARN!("Failed to get input id, hr %#lx.\n", hr);
                    }

                    stream = (*topo_node).u.transform.inputs.offset(input as isize);

                    is_sample = (FALSE as i32);
                    while is_sample == 0 && hr >= 0 {
                        hr = transform_stream_pop_event(stream, &raw mut stream_event);
                        if hr >= 0 {
                            let mut event_type: MediaEventType = 0;
                            hr = transform_stream_handle_event(session, stream, topo_node, input, stream_event);
                            if hr < 0 {
                                ERR!("Failed to handle stream event, hr %#lx\n", hr);
                            }
                            hr = IMFMediaEvent_GetType(stream_event, &raw mut event_type);
                            is_sample = (hr >= 0
                                && event_type == MEMediaSample as MediaEventType) as BOOL;
                            IMFMediaEvent_Release(stream_event);
                        }
                    }

                    if is_sample != 0 {
                        break 'sw;
                    }

                    up_node = session_get_topo_node_input(session, topo_node, input, &raw mut output);
                    if up_node.is_null() {
                        WARN!("Failed to get node %p/%u input\n", topo_node, input);
                    } else {
                        hr = session_request_sample_from_node(session, up_node, output);
                        if hr < 0 {
                            WARN!(
                                "Failed to request sample from upstream node %p/%lu, hr %#lx\n",
                                up_node,
                                output,
                                hr
                            );
                        }
                    }
                    break 'sw;
                }
                602 => {
                    let mut topo_node: *mut topo_node = session_get_async_transform_node(session, event_source);

                    if !topo_node.is_null() {
                        transform_node_pull_samples(session, topo_node);
                        transform_node_deliver_samples(session, topo_node);
                    }
                }
                1 => {
                    /* Wine-specific extension */
                    EnterCriticalSection(&raw mut (*session).cs);
                    if IMFMediaEvent_GetStatus(event, &raw mut hr) >= 0
                        && hr == MF_E_SHUTDOWN
                        && !session_get_media_source(session, event_source as *mut IMFMediaSource).is_null()
                    {
                        session_handle_source_shutdown(session);
                    }
                    LeaveCriticalSection(&raw mut (*session).cs);
                }
                _ => {}
            }
        }

        PropVariantClear(&raw mut value);
    }

    // failed:
    hr = IMFMediaEventGenerator_BeginGetEvent(event_source, iface, event_source as *mut IUnknown);
    if hr < 0 {
        if hr == MF_E_SHUTDOWN
            && !session_get_media_source(session, event_source as *mut IMFMediaSource).is_null()
        {
            session_handle_source_shutdown(session);
        } else {
            WARN!("Failed to re-subscribe, hr %#lx.\n", hr);
            IMFMediaEventQueue_QueueEvent((*session).event_queue, event);
        }
    }

    if !event.is_null() {
        IMFMediaEvent_Release(event);
    }

    IMFMediaEventGenerator_Release(event_source);

    return hr;
}

unsafe extern "system" fn session_sink_finalizer_callback_QueryInterface(iface: *mut IMFAsyncCallback, riid: *const IID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFAsyncCallback)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFAsyncCallback_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn session_sink_finalizer_callback_AddRef(iface: *mut IMFAsyncCallback) -> ULONG {
    let session: *mut media_session = impl_from_sink_finalizer_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_sink_finalizer_callback_Release(iface: *mut IMFAsyncCallback) -> ULONG {
    let mut session: *mut media_session = impl_from_sink_finalizer_callback_IMFAsyncCallback(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn session_sink_finalizer_callback_GetParameters(
    iface: *mut IMFAsyncCallback,
    flags: *mut DWORD,
    queue: *mut DWORD,
) -> HRESULT {
    E_NOTIMPL
}

unsafe extern "system" fn session_sink_finalizer_callback_Invoke(iface: *mut IMFAsyncCallback, result: *mut IMFAsyncResult) -> HRESULT {
    let session: *mut media_session = impl_from_sink_finalizer_callback_IMFAsyncCallback(iface);
    let mut fin_sink: *mut IMFFinalizableMediaSink = core::ptr::null_mut();
    let mut sinks_finalized: BOOL = (TRUE as i32);
    let mut sink: *mut media_sink;
    let mut state: *mut IUnknown = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IMFAsyncResult_GetState(result, core::ptr::addr_of_mut!(state));
    if hr < 0 {
        return hr;
    }

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    sink = ((*session).presentation.sinks.next as *mut u8)
        .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
    while core::ptr::addr_of_mut!((*sink).entry) != core::ptr::addr_of_mut!((*session).presentation.sinks) {
        if state == (*sink).__bindgen_anon_1.object {
            hr = IMFMediaSink_QueryInterface(
                (*sink).__bindgen_anon_1.sink,
                core::ptr::addr_of!(IID_IMFFinalizableMediaSink),
                core::ptr::addr_of_mut!(fin_sink) as *mut *mut core::ffi::c_void,
            );
            if hr < 0 {
                WARN!("Unexpected, missing IMFFinalizableMediaSink, hr %#lx.\n", hr);
            }
        } else {
            sinks_finalized &= (*sink).finalized;
            if sinks_finalized == 0 {
                break;
            }
        }
        sink = ((*sink).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(media_sink, entry) as isize)) as *mut media_sink;
    }

    IUnknown_Release(state);

    if !fin_sink.is_null() {
        /* Complete session transition, or close prematurely on error. */
        if ({ hr = IMFFinalizableMediaSink_EndFinalize(fin_sink, result); hr }) >= 0 {
            (*sink).finalized = (TRUE as i32);
            if sinks_finalized != 0 {
                session_set_closed(session, hr);
            }
        }
        IMFFinalizableMediaSink_Release(fin_sink);
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    return S_OK;
}

unsafe extern "system" fn session_rate_support_QueryInterface(iface: *mut IMFRateSupport, riid: *const GUID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateSupport(iface);
    return IMFMediaSession_QueryInterface(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface), riid, obj);
}

unsafe extern "system" fn session_rate_support_AddRef(iface: *mut IMFRateSupport) -> ULONG {
    let session: *mut media_session = impl_session_from_IMFRateSupport(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_rate_support_Release(iface: *mut IMFRateSupport) -> ULONG {
    let mut session: *mut media_session = impl_session_from_IMFRateSupport(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "C" fn session_presentation_object_get_rate(object: *mut IUnknown, direction: MFRATE_DIRECTION, thin: BOOL, fastest: BOOL, result: *mut f32) -> HRESULT {
    let mut rate_support: *mut IMFRateSupport = core::ptr::null_mut();
    let mut rate: f32;
    let mut hr: HRESULT;

    /* For objects that don't support rate control, it's assumed that only forward direction is allowed, at 1.0f. */

    if {
        hr = MFGetService(object, core::ptr::addr_of!(MF_RATE_CONTROL_SERVICE), core::ptr::addr_of!(IID_IMFRateSupport), core::ptr::addr_of_mut!(rate_support) as *mut *mut core::ffi::c_void);
        hr
    } < 0
    {
        if direction == MFRATE_FORWARD {
            *result = 1.0f32;
            return S_OK;
        } else {
            return MF_E_REVERSE_UNSUPPORTED;
        }
    }

    rate = 0.0f32;
    if fastest != 0 {
        if {
            hr = IMFRateSupport_GetFastestRate(rate_support, direction, thin, core::ptr::addr_of_mut!(rate));
            hr
        } >= 0
        {
            *result = min!(fabsf(rate), *result);
        }
    } else {
        if {
            hr = IMFRateSupport_GetSlowestRate(rate_support, direction, thin, core::ptr::addr_of_mut!(rate));
            hr
        } >= 0
        {
            *result = max!(fabsf(rate), *result);
        }
    }

    IMFRateSupport_Release(rate_support);

    return hr;
}

unsafe extern "C" fn session_get_presentation_rate(
    session: *mut media_session,
    direction: MFRATE_DIRECTION,
    thin: BOOL,
    fastest: BOOL,
    result: *mut f32,
) -> HRESULT {
    let mut source: *mut media_source = core::ptr::null_mut();
    let mut sink: *mut media_sink = core::ptr::null_mut();
    let mut hr: HRESULT = E_POINTER;
    let mut rate: f32;

    rate = if fastest != 0 { FLT_MAX } else { 0.0f32 };

    EnterCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    if ((*session).presentation.topo_status as i32) as u32 != MF_TOPOSTATUS_INVALID {
        source = ((*session).presentation.sources.next as *mut u8)
            .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
        while core::ptr::addr_of_mut!((*source).entry)
            != core::ptr::addr_of_mut!((*session).presentation.sources)
        {
            hr = session_presentation_object_get_rate(
                (*source).__bindgen_anon_1.object,
                direction,
                thin,
                fastest,
                core::ptr::addr_of_mut!(rate),
            );
            if hr < 0 {
                break;
            }
            source = ((*source).entry.next as *mut u8)
                .sub(core::mem::offset_of!(media_source, entry)) as *mut media_source;
        }

        if hr >= 0 {
            sink = ((*session).presentation.sinks.next as *mut u8)
                .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
            while core::ptr::addr_of_mut!((*sink).entry)
                != core::ptr::addr_of_mut!((*session).presentation.sinks)
            {
                hr = session_presentation_object_get_rate(
                    (*sink).__bindgen_anon_1.object,
                    direction,
                    thin,
                    fastest,
                    core::ptr::addr_of_mut!(rate),
                );
                if hr < 0 {
                    break;
                }
                sink = ((*sink).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(media_sink, entry)) as *mut media_sink;
            }
        }
    }

    LeaveCriticalSection(core::ptr::addr_of_mut!((*session).cs));

    if hr >= 0 {
        *result = if (direction as i32) as u32 == MFRATE_FORWARD { rate } else { -rate };
    }

    return hr;
}

unsafe extern "system" fn session_rate_support_GetSlowestRate(iface: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateSupport(iface);

    return session_get_presentation_rate(session, direction, thin, (FALSE) as BOOL, rate);
}

unsafe extern "system" fn session_rate_support_GetFastestRate(iface: *mut IMFRateSupport, direction: MFRATE_DIRECTION, thin: BOOL, rate: *mut f32) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateSupport(iface);

    return session_get_presentation_rate(session, direction, thin, (TRUE) as BOOL, rate);
}

unsafe extern "system" fn session_rate_support_IsRateSupported(iface: *mut IMFRateSupport, thin: BOOL, rate: f32, nearest_supported_rate: *mut f32) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateSupport(iface);
    let mut hr: HRESULT;

    EnterCriticalSection(&raw mut (*session).cs);
    hr = session_is_presentation_rate_supported(session, thin, rate, nearest_supported_rate);
    LeaveCriticalSection(&raw mut (*session).cs);

    return hr;
}

unsafe extern "system" fn session_rate_control_QueryInterface(iface: *mut IMFRateControl, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateControl(iface);
    return IMFMediaSession_QueryInterface(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface), riid, obj);
}

unsafe extern "system" fn session_rate_control_AddRef(mut iface: *mut IMFRateControl) -> ULONG {
    let mut session: *mut media_session = impl_session_from_IMFRateControl(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn session_rate_control_Release(mut iface: *mut IMFRateControl) -> ULONG {
    let mut session: *mut media_session = impl_session_from_IMFRateControl(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn session_rate_control_SetRate(iface: *mut IMFRateControl, thin: BOOL, rate: f32) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateControl(iface);
    let mut op: *mut session_op = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = create_session_op(SESSION_CMD_SET_RATE, core::ptr::addr_of_mut!(op));
    if hr < 0 {
        return hr;
    }

    (*op).__bindgen_anon_1.set_rate.thin = thin;
    (*op).__bindgen_anon_1.set_rate.rate = rate;
    hr = session_submit_command(session, op);
    IUnknown_Release(core::ptr::addr_of_mut!((*op).IUnknown_iface));
    return hr;
}

unsafe extern "system" fn session_rate_control_GetRate(iface: *mut IMFRateControl, thin: *mut BOOL, rate: *mut f32) -> HRESULT {
    let session: *mut media_session = impl_session_from_IMFRateControl(iface);

    if !thin.is_null() {
        *thin = (*session).presentation.thin_committed;
    }

    return IMFRateControl_GetRate((*session).clock_rate_control, core::ptr::null_mut(), rate);
}
unsafe extern "system" fn node_attribute_editor_QueryInterface(iface: *mut IMFTopologyNodeAttributeEditor, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualIID!(riid, core::ptr::addr_of!(IID_IMFTopologyNodeAttributeEditor)) != 0 ||
            IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0
    {
        *obj = iface as *mut core::ffi::c_void;
        IMFTopologyNodeAttributeEditor_AddRef(iface);
        return S_OK;
    }

    WARN!("Unsupported interface %s.\n", debugstr_guid(riid));
    *obj = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn node_attribute_editor_AddRef(iface: *mut IMFTopologyNodeAttributeEditor) -> ULONG {
    let session: *mut media_session = impl_session_from_IMFTopologyNodeAttributeEditor(iface);
    return IMFMediaSession_AddRef(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}

unsafe extern "system" fn node_attribute_editor_Release(iface: *mut IMFTopologyNodeAttributeEditor) -> ULONG {
    let session: *mut media_session = impl_session_from_IMFTopologyNodeAttributeEditor(iface);
    return IMFMediaSession_Release(core::ptr::addr_of_mut!((*session).IMFMediaSession_iface));
}
unsafe extern "system" fn node_attribute_editor_UpdateNodeAttributes(
    iface: *mut IMFTopologyNodeAttributeEditor,
    id: TOPOID,
    count: DWORD,
    updates: *mut MFTOPONODE_ATTRIBUTE_UPDATE,
) -> HRESULT {
    // FIXME("%p, %s, %lu, %p.\n", iface, wine_dbgstr_longlong(id), count, updates);
    E_NOTIMPL
}

#[no_mangle]
pub unsafe extern "system" fn MFCreateMediaSession(config: *mut IMFAttributes, session: *mut *mut IMFMediaSession) -> HRESULT {
    let mut without_quality_manager: BOOL = (FALSE as i32);
    let mut object: *mut media_session;
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<media_session>()) as _) as *mut media_session;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    (*object).IMFMediaSession_iface.lpVtbl = &raw const mfmediasessionvtbl;
    (*object).IMFGetService_iface.lpVtbl = &raw const session_get_service_vtbl;
    (*object).IMFRateSupport_iface.lpVtbl = &raw const session_rate_support_vtbl;
    (*object).IMFRateControl_iface.lpVtbl = &raw const session_rate_control_vtbl;
    (*object).IMFTopologyNodeAttributeEditor_iface.lpVtbl = &raw const node_attribute_editor_vtbl;
    (*object).commands_callback.lpVtbl = &raw const session_commands_callback_vtbl;
    (*object).sa_ready_callback.lpVtbl = &raw const session_sa_ready_callback_vtbl;
    (*object).events_callback.lpVtbl = &raw const session_events_callback_vtbl;
    (*object).sink_finalizer_callback.lpVtbl = &raw const session_sink_finalizer_callback_vtbl;
    (*object).refcount = 1;
    list_init(&raw mut (*object).topologies);
    list_init(&raw mut (*object).removed_topologies);
    list_init(&raw mut (*object).commands);
    list_init(&raw mut (*object).presentation.sources);
    list_init(&raw mut (*object).presentation.sinks);
    list_init(&raw mut (*object).presentation.nodes);
    (*object).presentation.rate = 1.0f32;
    InitializeCriticalSection(&raw mut (*object).cs);

    'failed: {
        hr = MFCreateTopology(&raw mut (*object).presentation.current_topology);
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreateEventQueue(&raw mut (*object).event_queue);
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreatePresentationClock(&raw mut (*object).clock);
        if hr < 0 {
            break 'failed;
        }

        hr = MFCreateSystemTimeSource(&raw mut (*object).system_time_source);
        if hr < 0 {
            break 'failed;
        }

        hr = IMFPresentationClock_QueryInterface(
            (*object).clock,
            &raw const IID_IMFRateControl,
            &raw mut (*object).clock_rate_control as *mut *mut core::ffi::c_void,
        );
        if hr < 0 {
            break 'failed;
        }

        if !config.is_null() {
            let mut clsid: GUID = core::mem::zeroed();

            if IMFAttributes_GetGUID(config, &raw const MF_SESSION_TOPOLOADER, &raw mut clsid) >= 0 {
                hr = CoCreateInstance(
                    &raw mut clsid,
                    core::ptr::null_mut(),
                    CLSCTX_INPROC_SERVER as DWORD,
                    &raw const IID_IMFTopoLoader,
                    &raw mut (*object).topo_loader as *mut LPVOID,
                );
                if hr < 0 {
                    WARN!("Failed to create custom topology loader, hr %#lx.\n", hr);
                }
            }

            if IMFAttributes_GetGUID(config, &raw const MF_SESSION_QUALITY_MANAGER, &raw mut clsid) >= 0 {
                without_quality_manager = (IsEqualGUID!(&raw mut clsid, &raw const GUID_NULL)) as i32;
                if without_quality_manager == 0 {
                    hr = CoCreateInstance(
                        &raw mut clsid,
                        core::ptr::null_mut(),
                        CLSCTX_INPROC_SERVER as DWORD,
                        &raw const IID_IMFQualityManager,
                        &raw mut (*object).quality_manager as *mut LPVOID,
                    );
                    if hr < 0 {
                        WARN!("Failed to create custom quality manager, hr %#lx.\n", hr);
                    }
                }
            }
        }

        if (*object).topo_loader.is_null() {
            hr = MFCreateTopoLoader(&raw mut (*object).topo_loader);
            if hr < 0 {
                break 'failed;
            }
        }

        if (*object).quality_manager.is_null() && without_quality_manager == 0 {
            hr = MFCreateStandardQualityManager(&raw mut (*object).quality_manager);
            if hr < 0 {
                break 'failed;
            }
        }

        if !(*object).quality_manager.is_null() {
            hr = IMFQualityManager_NotifyPresentationClock((*object).quality_manager, (*object).clock);
            if hr < 0 {
                break 'failed;
            }
        }

        *session = &raw mut (*object).IMFMediaSession_iface;

        return S_OK;
    }

    IMFMediaSession_Release(&raw mut (*object).IMFMediaSession_iface);
    return hr;
}

// COM vtable tables — generated from session.c by vtbl_gen.py
/// session.c:398 `static const IMFLocalMFTRegistrationVtbl local_mft_registration_vtbl`  (slot order: rust:types_amd64.rs)
pub static local_mft_registration_vtbl: IMFLocalMFTRegistrationVtbl = IMFLocalMFTRegistrationVtbl {
    QueryInterface: Some(local_mft_registration_QueryInterface),
    AddRef: Some(local_mft_registration_AddRef),
    Release: Some(local_mft_registration_Release),
    RegisterMFTs: Some(local_mft_registration_RegisterMFTs),
};

/// session.c:458 `static const IUnknownVtbl session_op_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static session_op_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(session_op_QueryInterface),
    AddRef: Some(session_op_AddRef),
    Release: Some(session_op_Release),
};

/// session.c:1901 `static const IMFVideoSampleAllocatorNotifyVtbl node_sample_allocator_cb_vtbl`  (slot order: rust:types_amd64.rs)
pub static node_sample_allocator_cb_vtbl: IMFVideoSampleAllocatorNotifyVtbl = IMFVideoSampleAllocatorNotifyVtbl {
    QueryInterface: Some(node_sample_allocator_cb_QueryInterface),
    AddRef: Some(node_sample_allocator_cb_AddRef),
    Release: Some(node_sample_allocator_cb_Release),
    NotifyRelease: Some(node_sample_allocator_cb_NotifyRelease),
};

/// session.c:2631 `static const IMFMediaSessionVtbl mfmediasessionvtbl`  (slot order: rust:types_amd64.rs)
pub static mfmediasessionvtbl: IMFMediaSessionVtbl = IMFMediaSessionVtbl {
    QueryInterface: Some(mfsession_QueryInterface),
    AddRef: Some(mfsession_AddRef),
    Release: Some(mfsession_Release),
    GetEvent: Some(mfsession_GetEvent),
    BeginGetEvent: Some(mfsession_BeginGetEvent),
    EndGetEvent: Some(mfsession_EndGetEvent),
    QueueEvent: Some(mfsession_QueueEvent),
    SetTopology: Some(mfsession_SetTopology),
    ClearTopologies: Some(mfsession_ClearTopologies),
    Start: Some(mfsession_Start),
    Pause: Some(mfsession_Pause),
    Stop: Some(mfsession_Stop),
    Close: Some(mfsession_Close),
    Shutdown: Some(mfsession_Shutdown),
    GetClock: Some(mfsession_GetClock),
    GetSessionCapabilities: Some(mfsession_GetSessionCapabilities),
    GetFullTopology: Some(mfsession_GetFullTopology),
};

/// session.c:2806 `static const IMFGetServiceVtbl session_get_service_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_get_service_vtbl: IMFGetServiceVtbl = IMFGetServiceVtbl {
    QueryInterface: Some(session_get_service_QueryInterface),
    AddRef: Some(session_get_service_AddRef),
    Release: Some(session_get_service_Release),
    GetService: Some(session_get_service_GetService),
};

/// session.c:2916 `static const IMFAsyncCallbackVtbl session_commands_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_commands_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(session_commands_callback_QueryInterface),
    AddRef: Some(session_commands_callback_AddRef),
    Release: Some(session_commands_callback_Release),
    GetParameters: Some(session_commands_callback_GetParameters),
    Invoke: Some(session_commands_callback_Invoke),
};

/// session.c:2979 `static const IMFAsyncCallbackVtbl session_sa_ready_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_sa_ready_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(session_sa_ready_callback_QueryInterface),
    AddRef: Some(session_sa_ready_callback_AddRef),
    Release: Some(session_sa_ready_callback_Release),
    GetParameters: Some(session_sa_ready_callback_GetParameters),
    Invoke: Some(session_sa_ready_callback_Invoke),
};

/// session.c:4749 `static const IMFAsyncCallbackVtbl session_events_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_events_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(session_events_callback_QueryInterface),
    AddRef: Some(session_events_callback_AddRef),
    Release: Some(session_events_callback_Release),
    GetParameters: Some(session_events_callback_GetParameters),
    Invoke: Some(session_events_callback_Invoke),
};

/// session.c:4838 `static const IMFAsyncCallbackVtbl session_sink_finalizer_callback_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_sink_finalizer_callback_vtbl: IMFAsyncCallbackVtbl = IMFAsyncCallbackVtbl {
    QueryInterface: Some(session_sink_finalizer_callback_QueryInterface),
    AddRef: Some(session_sink_finalizer_callback_AddRef),
    Release: Some(session_sink_finalizer_callback_Release),
    GetParameters: Some(session_sink_finalizer_callback_GetParameters),
    Invoke: Some(session_sink_finalizer_callback_Invoke),
};

/// session.c:4975 `static const IMFRateSupportVtbl session_rate_support_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_rate_support_vtbl: IMFRateSupportVtbl = IMFRateSupportVtbl {
    QueryInterface: Some(session_rate_support_QueryInterface),
    AddRef: Some(session_rate_support_AddRef),
    Release: Some(session_rate_support_Release),
    GetSlowestRate: Some(session_rate_support_GetSlowestRate),
    GetFastestRate: Some(session_rate_support_GetFastestRate),
    IsRateSupported: Some(session_rate_support_IsRateSupported),
};

/// session.c:5033 `static const IMFRateControlVtbl session_rate_control_vtbl`  (slot order: rust:types_amd64.rs)
pub static session_rate_control_vtbl: IMFRateControlVtbl = IMFRateControlVtbl {
    QueryInterface: Some(session_rate_control_QueryInterface),
    AddRef: Some(session_rate_control_AddRef),
    Release: Some(session_rate_control_Release),
    SetRate: Some(session_rate_control_SetRate),
    GetRate: Some(session_rate_control_GetRate),
};

/// session.c:5080 `static const IMFTopologyNodeAttributeEditorVtbl node_attribute_editor_vtbl`  (slot order: rust:types_amd64.rs)
pub static node_attribute_editor_vtbl: IMFTopologyNodeAttributeEditorVtbl = IMFTopologyNodeAttributeEditorVtbl {
    QueryInterface: Some(node_attribute_editor_QueryInterface),
    AddRef: Some(node_attribute_editor_AddRef),
    Release: Some(node_attribute_editor_Release),
    UpdateNodeAttributes: Some(node_attribute_editor_UpdateNodeAttributes),
};
