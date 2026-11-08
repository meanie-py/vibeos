use crate::d3dwine::context_gl::wined3d_context_gl_alloc_fence;
use crate::d3dwine::context_gl::wined3d_context_gl_alloc_occlusion_query;
use crate::d3dwine::context_gl::wined3d_context_gl_alloc_pipeline_statistics_query;
use crate::d3dwine::context_gl::wined3d_context_gl_alloc_so_statistics_query;
use crate::d3dwine::context_gl::wined3d_context_gl_alloc_timestamp_query;
use crate::d3dwine::context_gl::wined3d_context_gl_free_fence;
use crate::d3dwine::context_gl::wined3d_context_gl_free_occlusion_query;
use crate::d3dwine::context_gl::wined3d_context_gl_free_pipeline_statistics_query;
use crate::d3dwine::context_gl::wined3d_context_gl_free_so_statistics_query;
use crate::d3dwine::context_gl::wined3d_context_gl_free_timestamp_query;
use crate::d3dwine::context_gl::wined3d_context_gl_reacquire;
use crate::d3dwine::context_vk::wined3d_context_vk_allocate_query;
use crate::d3dwine::context_vk::wined3d_context_vk_destroy_vk_event;
use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::device::wined3d_device_context_issue_query;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut event_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_event_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_event_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_event_query_ops_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut occlusion_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_occlusion_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_occlusion_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_occlusion_query_ops_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut timestamp_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_timestamp_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_timestamp_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_timestamp_query_ops_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut timestamp_disjoint_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_timestamp_disjoint_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_timestamp_disjoint_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_timestamp_disjoint_query_ops_destroy
                as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut so_statistics_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_so_statistics_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_so_statistics_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_so_statistics_query_ops_destroy
                as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut pipeline_query_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_pipeline_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_pipeline_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_pipeline_query_ops_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut wined3d_query_vk_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_query_vk_poll as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_query_vk_issue as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_query_vk_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut wined3d_query_event_vk_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_query_event_vk_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_query_event_vk_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_query_vk_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut wined3d_query_timestamp_vk_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_query_vk_poll as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_query_timestamp_vk_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_query_vk_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

static mut wined3d_query_timestamp_disjoint_vk_ops: wined3d_query_ops = unsafe {
    wined3d_query_ops {
        query_poll: Some(
            wined3d_timestamp_disjoint_query_ops_poll
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_issue: Some(
            wined3d_timestamp_disjoint_query_ops_issue
                as unsafe extern "C" fn(*mut wined3d_query, uint32_t) -> BOOL,
        ),
        query_destroy: Some(
            wined3d_query_vk_destroy as unsafe extern "C" fn(*mut wined3d_query) -> (),
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d): debug-channel declaration macro, no Rust equivalent emitted here.

unsafe extern "C" fn wined3d_query_buffer_invalidate(query: *mut wined3d_query) {
    *(*query).map_ptr.offset(0 as isize) = 0 as UINT64;
    *(*query).map_ptr.offset(1 as isize) = !(0 as UINT64);
}

unsafe extern "C" fn wined3d_query_buffer_is_valid(query: *mut wined3d_query) -> BOOL {
    return (*(*query).map_ptr.offset(0 as isize)
        == *(*query).map_ptr.offset(1 as isize)) as i32;
}

unsafe extern "C" fn wined3d_query_create_buffer_object(context_gl: *mut wined3d_context_gl, query: *mut wined3d_query) {
    let map_flags: GLuint = (GL_MAP_READ_BIT | GL_MAP_WRITE_BIT | GL_MAP_PERSISTENT_BIT | GL_MAP_COHERENT_BIT) as GLuint;
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut buffer_object: GLuint = 0;

    ((*gl_info).gl_ops.ext.p_glGenBuffers.unwrap_unchecked())(1 as GLsizei, core::ptr::addr_of_mut!(buffer_object));
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_QUERY_BUFFER as GLenum, buffer_object);
    ((*gl_info).gl_ops.ext.p_glBufferStorage.unwrap_unchecked())(
        GL_QUERY_BUFFER as GLenum,
        (core::mem::size_of_val(&*(*query).map_ptr.add(0 as usize)) * 2 as usize) as GLsizeiptr,
        core::ptr::null::<core::ffi::c_void>(),
        map_flags as GLbitfield,
    );
    (*query).map_ptr = ((*gl_info).gl_ops.ext.p_glMapBufferRange.unwrap_unchecked())(
        GL_QUERY_BUFFER as GLenum,
        0 as GLintptr,
        (core::mem::size_of_val(&*(*query).map_ptr.add(0 as usize)) * 2 as usize) as GLsizeiptr,
        map_flags as GLbitfield,
    ) as *mut UINT64;
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(GL_QUERY_BUFFER as GLenum, 0 as GLuint);
    checkGLcall!(gl_info, b"query buffer object creation\0".as_ptr() as *const core::ffi::c_char);

    wined3d_query_buffer_invalidate(query);
    (*query).buffer_object = buffer_object as core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_gl_destroy_buffer_object(context_gl: *mut wined3d_context_gl, query: *mut wined3d_query) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    ((*gl_info).gl_ops.ext.p_glDeleteBuffers.unwrap_unchecked())(1, core::ptr::addr_of_mut!((*query).buffer_object));
    checkGLcall!(gl_info, b"query buffer object destruction\0".as_ptr() as *const i8);
    (*query).buffer_object = 0;
    (*query).map_ptr = core::ptr::null_mut();
}

unsafe extern "C" fn wined3d_query_buffer_queue_result(
    context_gl: *mut wined3d_context_gl,
    query: *mut wined3d_query,
    id: GLuint,
) -> BOOL {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut tmp_sync: GLsync = core::ptr::null_mut();

    if (*gl_info).supported[ARB_QUERY_BUFFER_OBJECT as usize] == 0
        || (*gl_info).supported[ARB_BUFFER_STORAGE as usize] == 0
    {
        return (FALSE) as BOOL;
    }
    /* Don't use query buffers without CSMT, mainly for simplicity. */
    if (*(*(*context_gl).c.device).cs).thread.is_null() {
        return (FALSE) as BOOL;
    }

    if (*query).buffer_object != 0 {
        /* If there's still a query result in-flight for the existing buffer
         * object (i.e., the query was restarted before we received its
         * result), we can't reuse the existing buffer object. */
        if wined3d_query_buffer_is_valid(query) != 0 {
            wined3d_query_buffer_invalidate(query);
        } else {
            wined3d_query_gl_destroy_buffer_object(context_gl, query);
        }
    }

    if (*query).buffer_object == 0 {
        wined3d_query_create_buffer_object(context_gl, query);
    }

    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
        GL_QUERY_BUFFER as GLenum,
        (*query).buffer_object as GLuint,
    );
    /* Read the same value twice. We know we have the result if map_ptr[0] == map_ptr[1]. */
    ((*gl_info).gl_ops.ext.p_glGetQueryObjectui64v.unwrap_unchecked())(
        id,
        GL_QUERY_RESULT as GLenum,
        0 as *mut ::core::ffi::c_void as *mut GLuint64,
    );
    ((*gl_info).gl_ops.ext.p_glGetQueryObjectui64v.unwrap_unchecked())(
        id,
        GL_QUERY_RESULT as GLenum,
        core::mem::size_of_val(&*(*query).map_ptr)
            as *mut ::core::ffi::c_void as *mut GLuint64,
    );
    ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
        GL_QUERY_BUFFER as GLenum,
        0 as GLuint,
    );
    checkGLcall!(gl_info, "queue query result");

    /* ARB_buffer_storage requires the client to call FenceSync with
     * SYNC_GPU_COMMANDS_COMPLETE after the server does a write. This behavior
     * is not enforced by Mesa.
     */
    tmp_sync = ((*gl_info).gl_ops.ext.p_glFenceSync.unwrap_unchecked())(
        GL_SYNC_GPU_COMMANDS_COMPLETE as GLenum,
        0 as GLbitfield,
    );
    ((*gl_info).gl_ops.ext.p_glDeleteSync.unwrap_unchecked())(tmp_sync);
    checkGLcall!(gl_info, "query buffer sync");

    return (TRUE) as BOOL;
}

unsafe extern "C" fn get_query_result64(id: GLuint, gl_info: *const wined3d_gl_info) -> UINT64 {
    if (*gl_info).supported[ARB_TIMER_QUERY as usize] != 0 {
        let mut result: GLuint64 = 0;
        (*gl_info)
            .gl_ops
            .ext
            .p_glGetQueryObjectui64v
            .unwrap_unchecked()(
            id,
            GL_QUERY_RESULT as GLenum,
            &raw mut result,
        );
        return result as UINT64;
    } else {
        let mut result: GLuint = 0;
        (*gl_info)
            .gl_ops
            .ext
            .p_glGetQueryObjectuiv
            .unwrap_unchecked()(
            id,
            GL_QUERY_RESULT as GLenum,
            &raw mut result,
        );
        return result as UINT64;
    }
}

unsafe extern "C" fn wined3d_query_init(
    query: *mut wined3d_query,
    device: *mut wined3d_device,
    type_0: wined3d_query_type,
    data: *const ::core::ffi::c_void,
    data_size: DWORD,
    query_ops: *const wined3d_query_ops,
    parent: *mut ::core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) {
    (*query).ref_ = (1) as LONG;
    (*query).parent = parent;
    (*query).parent_ops = parent_ops;
    (*query).device = device;
    (*query).state = QUERY_CREATED;
    (*query).type_ = type_0;
    (*query).data = data;
    (*query).data_size = data_size;
    (*query).query_ops = query_ops;
    (*query).poll_in_cs = !(*(*device).cs).thread.is_null();
    list_init(core::ptr::addr_of_mut!((*query).poll_list_entry));
}

unsafe extern "C" fn wined3d_event_query_from_query(query: *mut wined3d_query) -> *mut wined3d_event_query {
    return CONTAINING_RECORD!(query, wined3d_event_query, query);
}

unsafe extern "C" fn wined3d_occlusion_query_from_query(
    query: *mut wined3d_query,
) -> *mut wined3d_occlusion_query {
    return (query as usize).wrapping_sub(core::mem::offset_of!(wined3d_occlusion_query, query))
        as *mut wined3d_occlusion_query;
}

unsafe extern "C" fn wined3d_timestamp_query_from_query(
    query: *mut wined3d_query,
) -> *mut wined3d_timestamp_query {
    return CONTAINING_RECORD!(query, wined3d_timestamp_query, query);
}

unsafe extern "C" fn wined3d_so_statistics_query_from_query(
    mut query: *mut wined3d_query,
) -> *mut wined3d_so_statistics_query {
    return (query as *mut u8).sub(core::mem::offset_of!(wined3d_so_statistics_query, query))
        as *mut wined3d_so_statistics_query;
}

unsafe extern "C" fn wined3d_pipeline_statistics_query_from_query(
    query: *mut wined3d_query,
) -> *mut wined3d_pipeline_statistics_query {
    return (query as usize).wrapping_sub(
        core::mem::offset_of!(wined3d_pipeline_statistics_query, query),
    ) as *mut wined3d_pipeline_statistics_query;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fence_test(
    fence: *const wined3d_fence,
    device: *mut wined3d_device,
    flags: uint32_t,
) -> wined3d_fence_result {
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut ret: wined3d_fence_result;
    let mut fence_result: BOOL;

    if (*fence).context_gl.is_null() {
        return WINED3D_FENCE_NOT_STARTED;
    }

    context_gl = wined3d_context_gl_reacquire((*fence).context_gl as *mut wined3d_context_gl);
    if context_gl.is_null() {
        if (*(*(*fence).context_gl).gl_info).supported[ARB_SYNC as usize] == 0 {
            WARN!("Fence tested from wrong thread.\n");
            return WINED3D_FENCE_WRONG_THREAD;
        }
        context_gl = wined3d_context_gl(context_acquire(
            device,
            core::ptr::null_mut(),
            0,
        ));
    }
    gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_SYNC as usize] != 0 {
        let mut gl_ret: GLenum = ((*gl_info).gl_ops.ext.p_glClientWaitSync.unwrap_unchecked())(
            (*fence).object.sync,
            (if flags & (WINED3DGETDATA_FLUSH as uint32_t) != 0 {
                GL_SYNC_FLUSH_COMMANDS_BIT
            } else {
                0
            }) as GLbitfield,
            0 as GLuint64,
        );
        checkGLcall!(gl_info, b"glClientWaitSync\0".as_ptr() as *const i8);

        match gl_ret {
            GL_ALREADY_SIGNALED | GL_CONDITION_SATISFIED => {
                ret = WINED3D_FENCE_OK;
            }
            GL_TIMEOUT_EXPIRED => {
                ret = WINED3D_FENCE_WAITING;
            }
            GL_WAIT_FAILED | _ => {
                ERR!("glClientWaitSync returned %#x.\n", gl_ret);
                ret = WINED3D_FENCE_ERROR;
            }
        }
    } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
        fence_result = ((*gl_info).gl_ops.ext.p_glTestFenceAPPLE.unwrap_unchecked())((*fence).object.id) as BOOL;
        checkGLcall!(gl_info, b"glTestFenceAPPLE\0".as_ptr() as *const i8);
        if fence_result != 0 {
            ret = WINED3D_FENCE_OK;
        } else {
            ret = WINED3D_FENCE_WAITING;
        }
    } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
        fence_result = ((*gl_info).gl_ops.ext.p_glTestFenceNV.unwrap_unchecked())((*fence).object.id) as BOOL;
        checkGLcall!(gl_info, b"glTestFenceNV\0".as_ptr() as *const i8);
        if fence_result != 0 {
            ret = WINED3D_FENCE_OK;
        } else {
            ret = WINED3D_FENCE_WAITING;
        }
    } else {
        ERR!("Fence created despite lack of GL support.\n");
        ret = WINED3D_FENCE_ERROR;
    }

    context_release(core::ptr::addr_of_mut!((*context_gl).c));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fence_wait(
    fence: *const wined3d_fence,
    device: *mut wined3d_device,
) -> wined3d_fence_result {
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut ret: wined3d_fence_result;

    if (*fence).context_gl.is_null() {
        return WINED3D_FENCE_NOT_STARTED;
    }
    gl_info = (*(*fence).context_gl).gl_info;

    context_gl = wined3d_context_gl_reacquire((*fence).context_gl as *mut wined3d_context_gl);
    if context_gl.is_null() {
        /* A glFinish does not reliably wait for draws in other contexts. The caller has
         * to find its own way to cope with the thread switch
         */
        if (*gl_info).supported[ARB_SYNC as usize] == 0 {
            WARN!("Fence finished from wrong thread.\n");
            return WINED3D_FENCE_WRONG_THREAD;
        }
        context_gl = wined3d_context_gl(context_acquire(
            device,
            core::ptr::null_mut::<wined3d_texture>(),
            0 as core::ffi::c_uint,
        ));
    }
    gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_SYNC as usize] != 0 {
        /* Timeouts near 0xffffffffffffffff may immediately return GL_TIMEOUT_EXPIRED,
         * possibly because macOS internally adds some slop to the timer. To avoid this,
         * we use a large number that isn't near the point of overflow (macOS 10.12.5).
         */
        let gl_ret: GLenum = ((*gl_info).gl_ops.ext.p_glClientWaitSync.unwrap_unchecked())(
            (*fence).object.sync,
            GL_SYNC_FLUSH_COMMANDS_BIT as GLbitfield,
            !(0 as GLuint64) >> 1,
        );
        checkGLcall!(gl_info, "glClientWaitSync");

        match gl_ret {
            GL_ALREADY_SIGNALED | GL_CONDITION_SATISFIED => {
                ret = WINED3D_FENCE_OK;
            }
            /* We don't expect a timeout for a ~292 year wait */
            _ => {
                ERR!("glClientWaitSync returned %#x.\n", gl_ret);
                ret = WINED3D_FENCE_ERROR;
            }
        }
    } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glFinishFenceAPPLE.unwrap_unchecked())((*fence).object.id);
        checkGLcall!(gl_info, "glFinishFenceAPPLE");
        ret = WINED3D_FENCE_OK;
    } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glFinishFenceNV.unwrap_unchecked())((*fence).object.id);
        checkGLcall!(gl_info, "glFinishFenceNV");
        ret = WINED3D_FENCE_OK;
    } else {
        ERR!("Fence created without GL support.\n");
        ret = WINED3D_FENCE_ERROR;
    }

    context_release(core::ptr::addr_of_mut!((*context_gl).c));
    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fence_issue(fence: *mut wined3d_fence, device: *mut wined3d_device) {
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut gl_info: *const wined3d_gl_info;

    if !(*fence).context_gl.is_null()
        && {
            context_gl = wined3d_context_gl_reacquire((*fence).context_gl);
            context_gl.is_null()
        }
        && (*(*(*fence).context_gl).gl_info).supported[ARB_SYNC as usize] == 0
    {
        wined3d_context_gl_free_fence(fence);
    }
    if context_gl.is_null() {
        context_gl = wined3d_context_gl(context_acquire(device, core::ptr::null_mut(), 0));
    }
    gl_info = (*context_gl).gl_info;
    if (*fence).context_gl.is_null() {
        wined3d_context_gl_alloc_fence(context_gl, fence);
    }

    if (*gl_info).supported[ARB_SYNC as usize] != 0 {
        if !(*fence).object.sync.is_null() {
            ((*gl_info).gl_ops.ext.p_glDeleteSync.unwrap_unchecked())((*fence).object.sync);
        }
        checkGLcall!(gl_info, "glDeleteSync");
        (*fence).object.sync = ((*gl_info).gl_ops.ext.p_glFenceSync.unwrap_unchecked())(GL_SYNC_GPU_COMMANDS_COMPLETE, 0);
        checkGLcall!(gl_info, "glFenceSync");
    } else if (*gl_info).supported[APPLE_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glSetFenceAPPLE.unwrap_unchecked())((*fence).object.id);
        checkGLcall!(gl_info, "glSetFenceAPPLE");
    } else if (*gl_info).supported[NV_FENCE as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glSetFenceNV.unwrap_unchecked())((*fence).object.id, GL_ALL_COMPLETED_NV);
        checkGLcall!(gl_info, "glSetFenceNV");
    }

    context_release(core::ptr::addr_of_mut!((*context_gl).c));
}

unsafe extern "C" fn wined3d_fence_free(fence: *mut wined3d_fence) {
    if !(*fence).context_gl.is_null() {
        wined3d_context_gl_free_fence(fence);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fence_destroy(fence: *mut wined3d_fence) {
    wined3d_fence_free(fence);
    free((fence).cast());
}

unsafe extern "C" fn wined3d_fence_init(fence: *mut wined3d_fence, gl_info: *const wined3d_gl_info) -> HRESULT {
    let _ = fence;
    if ((wined3d_fence_supported(gl_info)) == 0) {
        WARN!("Fences not supported.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_fence_create(
    device: *mut wined3d_device,
    fence: *mut *mut wined3d_fence,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info = core::ptr::addr_of!(
        (*wined3d_adapter_gl((*device).adapter)).gl_info
    );
    let mut object: *mut wined3d_fence = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_fence>()) as _,
    ) as *mut wined3d_fence;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_fence_init(object, gl_info);
    if hr < 0 {
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *fence = object;

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_query_incref(query: *mut wined3d_query) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*query).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", query, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_query_destroy_object(object: *mut ::core::ffi::c_void) {
    let query: *mut wined3d_query = object as *mut wined3d_query;

    if list_empty(core::ptr::addr_of_mut!((*query).poll_list_entry)) == 0 {
        list_remove(core::ptr::addr_of_mut!((*query).poll_list_entry));
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_decref(mut query: *mut wined3d_query) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedDecrement(
        &raw mut (*query).ref_ as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p decreasing refcount to %u.\n", query, refcount);
    if refcount == 0 {
        let mut device: *mut wined3d_device = (*query).device;
        wined3d_mutex_lock();
        (*(*query).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*query).parent);
        wined3d_cs_destroy_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_query_destroy_object
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            query as *mut ::core::ffi::c_void,
        );
        (*(*(*device).adapter).adapter_ops)
            .adapter_destroy_query
            .unwrap_unchecked()(query);
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_get_data(
    query: *mut wined3d_query,
    data: *mut core::ffi::c_void,
    data_size: UINT,
    flags: uint32_t,
) -> HRESULT {
    if (*query).state == QUERY_BUILDING {
        WARN!("Query is building, returning S_FALSE.\n");
        return S_FALSE;
    }

    if (*query).state == QUERY_CREATED {
        WARN!("Query wasn't started yet.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if (*query).counter_main != (*query).counter_retrieved
        || ((*query).buffer_object != 0 && !(wined3d_query_buffer_is_valid(query) != 0))
    {
        if (flags & WINED3DGETDATA_FLUSH) != 0 && (*(*(*query).device).cs).queries_flushed == 0 {
            (((*(*(*(*query).device).cs).c.ops).flush).unwrap_unchecked())(
                core::ptr::addr_of_mut!((*(*(*query).device).cs).c),
            );
        }
        return S_FALSE;
    } else if (*query).poll_in_cs == false
        && (((*(*query).query_ops).query_poll).unwrap_unchecked())(query, flags) == 0
    {
        return S_FALSE;
    }

    if (*query).buffer_object != 0 {
        (*query).data = (*query).map_ptr as *const core::ffi::c_void;
    }

    if !data.is_null() {
        memcpy(
            data,
            (*query).data,
            min!(data_size, (*query).data_size) as size_t,
        );
    }

    return S_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_query_get_data_size(query: *const wined3d_query) -> UINT {
    // TRACE("query %p.\n", query);
    (*query).data_size
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_issue(query: *mut wined3d_query, flags: u32) -> HRESULT {
    TRACE!("query %p, flags %#x.\n", query, flags);
    wined3d_device_context_issue_query(
        core::ptr::addr_of_mut!((*(*(*query).device).cs).c),
        query,
        flags,
    );
    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_occlusion_query_ops_poll(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let mut oq: *mut wined3d_occlusion_query = wined3d_occlusion_query_from_query(query);
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut available: GLuint = 0;

    context_gl = wined3d_context_gl_reacquire((*oq).context_gl);
    if context_gl.is_null() {
        (*oq).samples = 1;
        return (TRUE) as BOOL;
    }
    gl_info = (*context_gl).gl_info;

    ((*gl_info).gl_ops.ext.p_glGetQueryObjectuiv.unwrap_unchecked())(
        (*oq).id,
        GL_QUERY_RESULT_AVAILABLE,
        core::ptr::addr_of_mut!(available),
    );

    if available != 0 {
        (*oq).samples = get_query_result64((*oq).id, gl_info);
    }

    checkGLcall!(gl_info, "poll occlusion query");
    context_release(core::ptr::addr_of_mut!((*context_gl).c));

    return available as BOOL;
}

unsafe extern "C" fn wined3d_event_query_ops_poll(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let event_query: *mut wined3d_event_query = wined3d_event_query_from_query(query);
    let mut ret: wined3d_fence_result;

    ret = wined3d_fence_test(core::ptr::addr_of_mut!((*event_query).fence), (*query).device, flags);
    match ret as u32 {
        WINED3D_FENCE_OK | WINED3D_FENCE_NOT_STARTED => {
            (*event_query).signalled = (TRUE as i32);
            return (*event_query).signalled;
        }
        WINED3D_FENCE_WAITING => {
            (*event_query).signalled = (FALSE as i32);
            return (*event_query).signalled;
        }
        WINED3D_FENCE_WRONG_THREAD => {
            FIXME!("(%p) Wrong thread, reporting GPU idle.\n", query);
            (*event_query).signalled = (TRUE as i32);
            return (*event_query).signalled;
        }
        WINED3D_FENCE_ERROR => {
            ERR!("The GL event query failed.\n");
            (*event_query).signalled = (TRUE as i32);
            return (*event_query).signalled;
        }
        _ => {
            ERR!("Unexpected wined3d_event_query_test result %#x.\n", ret as u32);
            (*event_query).signalled = (TRUE as i32);
            return (*event_query).signalled;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_query_get_parent(
    query: *const wined3d_query,
) -> *mut core::ffi::c_void {
    // TRACE("query %p.\n", query);
    (*query).parent
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_query_get_type(
    query: *const wined3d_query,
) -> wined3d_query_type {
    // TRACE("query %p.\n", query);
    (*query).type_
}

unsafe extern "C" fn wined3d_event_query_ops_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    if (flags & WINED3DISSUE_END as uint32_t) != 0 {
        let event_query: *mut wined3d_event_query = wined3d_event_query_from_query(query);
        wined3d_fence_issue(core::ptr::addr_of_mut!((*event_query).fence), (*query).device);
        return (TRUE) as BOOL;
    } else if (flags & WINED3DISSUE_BEGIN as uint32_t) != 0 {
        /* Started implicitly at query creation. */
        ERR!("Event query issued with START flag - what to do?\n");
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_occlusion_query_ops_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let mut oq: *mut wined3d_occlusion_query = wined3d_occlusion_query_from_query(query);
    let mut device: *mut wined3d_device = (*query).device;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut poll: BOOL = (FALSE as i32);

    /* This is allowed according to MSDN and our tests. Reset the query and
     * restart. */
    if (flags & WINED3DISSUE_BEGIN) != 0 {
        if (*oq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*oq).context_gl);
            if !context_gl.is_null() {
                gl_info = (*context_gl).gl_info;
                ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_SAMPLES_PASSED);
                checkGLcall!(gl_info, b"glEndQuery()\0".as_ptr() as *const ::core::ffi::c_char);
            } else {
                wined3d_context_gl_free_occlusion_query(oq);
                context_gl = wined3d_context_gl(context_acquire(device, core::ptr::null_mut(), 0));
                wined3d_context_gl_alloc_occlusion_query(context_gl, oq);
            }
        } else {
            if !(*oq).context_gl.is_null() {
                wined3d_context_gl_free_occlusion_query(oq);
            }
            context_gl = wined3d_context_gl(context_acquire(device, core::ptr::null_mut(), 0));
            wined3d_context_gl_alloc_occlusion_query(context_gl, oq);
        }
        gl_info = (*context_gl).gl_info;

        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_SAMPLES_PASSED, (*oq).id);
        checkGLcall!(gl_info, b"glBeginQuery()\0".as_ptr() as *const ::core::ffi::c_char);

        context_release(core::ptr::addr_of_mut!((*context_gl).c));
        (*oq).started = (TRUE as i32);
    }
    if (flags & WINED3DISSUE_END) != 0 {
        /* MSDN says END on a non-building occlusion query returns an error,
         * but our tests show that it returns OK. But OpenGL doesn't like it,
         * so avoid generating an error. */
        if (*oq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*oq).context_gl);
            if !context_gl.is_null() {
                gl_info = (*context_gl).gl_info;
                ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_SAMPLES_PASSED);
                checkGLcall!(gl_info, b"glEndQuery()\0".as_ptr() as *const ::core::ffi::c_char);
                wined3d_query_buffer_queue_result(context_gl, query, (*oq).id);

                context_release(core::ptr::addr_of_mut!((*context_gl).c));
                poll = (TRUE as i32);
            } else {
                /* Wrong thread, can't end query. */
            }
        }
        (*oq).started = (FALSE as i32);
    }

    return poll;
}

unsafe extern "C" fn wined3d_timestamp_query_ops_poll(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let tq: *mut wined3d_timestamp_query = wined3d_timestamp_query_from_query(query);
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut timestamp: GLuint64 = 0;
    let mut available: GLuint = 0;

    context_gl = wined3d_context_gl_reacquire((*tq).context_gl);
    if context_gl.is_null() {
        (*tq).timestamp = 1;
        return (TRUE) as BOOL;
    }
    gl_info = (*context_gl).gl_info;

    ((*gl_info).gl_ops.ext.p_glGetQueryObjectuiv.unwrap_unchecked())(
        (*tq).id,
        GL_QUERY_RESULT_AVAILABLE,
        core::ptr::addr_of_mut!(available),
    );
    checkGLcall!(gl_info, b"glGetQueryObjectuiv(GL_QUERY_RESULT_AVAILABLE)\0".as_ptr() as *const core::ffi::c_char);

    if available != 0 {
        ((*gl_info).gl_ops.ext.p_glGetQueryObjectui64v.unwrap_unchecked())(
            (*tq).id,
            GL_QUERY_RESULT,
            core::ptr::addr_of_mut!(timestamp),
        );
        checkGLcall!(gl_info, b"glGetQueryObjectui64v(GL_QUERY_RESULT)\0".as_ptr() as *const core::ffi::c_char);

        (*tq).timestamp = timestamp as uint64_t;
    }

    context_release(core::ptr::addr_of_mut!((*context_gl).c));

    return available as BOOL;
}

unsafe extern "C" fn wined3d_timestamp_query_ops_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let mut tq: *mut wined3d_timestamp_query = wined3d_timestamp_query_from_query(query);
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;

    if (flags & WINED3DISSUE_BEGIN) != 0 {
        WARN!("Ignoring WINED3DISSUE_BEGIN with a TIMESTAMP query.\n");
    }
    if (flags & WINED3DISSUE_END) != 0 {
        if !(*tq).context_gl.is_null() {
            wined3d_context_gl_free_timestamp_query(tq);
        }
        context_gl = wined3d_context_gl(context_acquire((*query).device, core::ptr::null_mut(), 0));
        gl_info = (*context_gl).gl_info;
        wined3d_context_gl_alloc_timestamp_query(context_gl, tq);
        ((*gl_info).gl_ops.ext.p_glQueryCounter.unwrap_unchecked())((*tq).id, GL_TIMESTAMP);
        checkGLcall!(gl_info, "glQueryCounter()");
        context_release(core::ptr::addr_of_mut!((*context_gl).c));

        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_timestamp_disjoint_query_ops_poll(
    query: *mut wined3d_query,
    flags: uint32_t,
) -> BOOL {
    // TRACE("query %p, flags %#x.\n", query, flags);
    (TRUE as i32)
}

unsafe extern "C" fn wined3d_timestamp_disjoint_query_ops_issue(
    query: *mut wined3d_query,
    flags: uint32_t,
) -> BOOL {
    // TRACE("query %p, flags %#x.\n", query, flags);
    (FALSE as i32)
}

unsafe extern "C" fn wined3d_so_statistics_query_ops_poll(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let pq: *mut wined3d_so_statistics_query = wined3d_so_statistics_query_from_query(query);
    let mut written_available: GLuint = 0;
    let mut generated_available: GLuint = 0;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();

    if {
        context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
        context_gl.is_null()
    } {
        memset(
            core::ptr::addr_of_mut!((*pq).statistics) as *mut c_void,
            0,
            core::mem::size_of::<wined3d_query_data_so_statistics>() as size_t,
        );
        return (TRUE) as BOOL;
    }
    gl_info = (*context_gl).gl_info;

    ((*gl_info).gl_ops.ext.p_glGetQueryObjectuiv.unwrap_unchecked())(
        (*pq).u.query.written,
        GL_QUERY_RESULT_AVAILABLE,
        core::ptr::addr_of_mut!(written_available),
    );
    ((*gl_info).gl_ops.ext.p_glGetQueryObjectuiv.unwrap_unchecked())(
        (*pq).u.query.generated,
        GL_QUERY_RESULT_AVAILABLE,
        core::ptr::addr_of_mut!(generated_available),
    );

    if written_available != 0 && generated_available != 0 {
        (*pq).statistics.primitives_written = get_query_result64((*pq).u.query.written, gl_info);
        (*pq).statistics.primitives_generated = get_query_result64((*pq).u.query.generated, gl_info);
    }

    checkGLcall!(gl_info, b"poll SO statistics query\0".as_ptr() as *const i8);
    context_release(core::ptr::addr_of_mut!((*context_gl).c));

    return (written_available != 0 && generated_available != 0) as BOOL;
}

unsafe extern "C" fn wined3d_so_statistics_query_end(query: *mut wined3d_so_statistics_query, context_gl: *mut wined3d_context_gl) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_TRANSFORM_FEEDBACK3 as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glEndQueryIndexed.unwrap_unchecked())(GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN, (*query).stream_idx);
        ((*gl_info).gl_ops.ext.p_glEndQueryIndexed.unwrap_unchecked())(GL_PRIMITIVES_GENERATED, (*query).stream_idx);
    } else {
        ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN);
        ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_PRIMITIVES_GENERATED);
    }
    checkGLcall!(gl_info, b"end query\0".as_ptr() as *const i8);
}

unsafe extern "C" fn wined3d_so_statistics_query_ops_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let mut pq: *mut wined3d_so_statistics_query = wined3d_so_statistics_query_from_query(query);
    let mut device: *mut wined3d_device = (*query).device;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut poll: BOOL = (FALSE as i32);

    TRACE!("query %p, flags %#x.\n", query, flags);

    if (flags & WINED3DISSUE_BEGIN) != 0 {
        if (*pq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
            if !context_gl.is_null() {
                wined3d_so_statistics_query_end(pq, context_gl);
            } else {
                FIXME!("Wrong thread, can't restart query.\n");
                wined3d_context_gl_free_so_statistics_query(pq);
                context_gl = wined3d_context_gl(context_acquire(
                    device,
                    core::ptr::null_mut(),
                    0,
                ));
                wined3d_context_gl_alloc_so_statistics_query(context_gl, pq);
            }
        } else {
            if !(*pq).context_gl.is_null() {
                wined3d_context_gl_free_so_statistics_query(pq);
            }
            context_gl = wined3d_context_gl(context_acquire(
                device,
                core::ptr::null_mut(),
                0,
            ));
            wined3d_context_gl_alloc_so_statistics_query(context_gl, pq);
        }
        gl_info = (*context_gl).gl_info;

        if (*gl_info).supported[ARB_TRANSFORM_FEEDBACK3 as usize] != 0 {
            ((*gl_info).gl_ops.ext.p_glBeginQueryIndexed.unwrap_unchecked())(
                GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN,
                (*pq).stream_idx,
                (*pq).u.query.written,
            );
            ((*gl_info).gl_ops.ext.p_glBeginQueryIndexed.unwrap_unchecked())(
                GL_PRIMITIVES_GENERATED,
                (*pq).stream_idx,
                (*pq).u.query.generated,
            );
        } else {
            ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(
                GL_TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN,
                (*pq).u.query.written,
            );
            ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(
                GL_PRIMITIVES_GENERATED,
                (*pq).u.query.generated,
            );
        }
        checkGLcall!(gl_info, "begin query");

        context_release(core::ptr::addr_of_mut!((*context_gl).c));
        (*pq).started = (TRUE as i32);
    }
    if (flags & WINED3DISSUE_END) != 0 {
        if (*pq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
            if !context_gl.is_null() {
                wined3d_so_statistics_query_end(pq, context_gl);

                context_release(core::ptr::addr_of_mut!((*context_gl).c));
                poll = (TRUE as i32);
            } else {
                FIXME!("Wrong thread, can't end query.\n");
            }
        }
        (*pq).started = (FALSE as i32);
    }

    return poll;
}

unsafe extern "C" fn wined3d_pipeline_query_ops_poll(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let pq: *mut wined3d_pipeline_statistics_query = wined3d_pipeline_statistics_query_from_query(query);
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut available: GLuint = 0;
    let mut i: i32;

    context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
    if context_gl.is_null() {
        memset(
            core::ptr::addr_of_mut!((*pq).statistics) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<wined3d_query_data_pipeline_statistics>() as size_t,
        );
        return (TRUE) as BOOL;
    }
    gl_info = (*context_gl).gl_info;

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*pq).u.id) as usize {
        ((*gl_info).gl_ops.ext.p_glGetQueryObjectuiv.unwrap_unchecked())(
            (*pq).u.id[i as usize],
            GL_QUERY_RESULT_AVAILABLE,
            core::ptr::addr_of_mut!(available),
        );
        if available == 0 {
            break;
        }
        i += 1;
    }

    if available != 0 {
        (*pq).statistics.vertices_submitted = get_query_result64((*pq).u.query.vertices, gl_info);
        (*pq).statistics.primitives_submitted = get_query_result64((*pq).u.query.primitives, gl_info);
        (*pq).statistics.vs_invocations = get_query_result64((*pq).u.query.vertex_shader, gl_info);
        (*pq).statistics.hs_invocations = get_query_result64((*pq).u.query.tess_control_shader, gl_info);
        (*pq).statistics.ds_invocations = get_query_result64((*pq).u.query.tess_eval_shader, gl_info);
        (*pq).statistics.gs_invocations = get_query_result64((*pq).u.query.geometry_shader, gl_info);
        (*pq).statistics.gs_primitives = get_query_result64((*pq).u.query.geometry_primitives, gl_info);
        (*pq).statistics.ps_invocations = get_query_result64((*pq).u.query.fragment_shader, gl_info);
        (*pq).statistics.cs_invocations = get_query_result64((*pq).u.query.compute_shader, gl_info);
        (*pq).statistics.clipping_input_primitives = get_query_result64((*pq).u.query.clipping_input, gl_info);
        (*pq).statistics.clipping_output_primitives = get_query_result64((*pq).u.query.clipping_output, gl_info);
    }

    checkGLcall!(gl_info, b"poll pipeline statistics query\0".as_ptr() as *const i8);
    context_release(core::ptr::addr_of_mut!((*context_gl).c));
    return available as BOOL;
}

unsafe extern "C" fn wined3d_pipeline_statistics_query_end(
    query: *mut wined3d_pipeline_statistics_query,
    context_gl: *mut wined3d_context_gl,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_VERTICES_SUBMITTED_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_PRIMITIVES_SUBMITTED_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_VERTEX_SHADER_INVOCATIONS_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_TESS_CONTROL_SHADER_PATCHES_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_TESS_EVALUATION_SHADER_INVOCATIONS_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_GEOMETRY_SHADER_INVOCATIONS);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_GEOMETRY_SHADER_PRIMITIVES_EMITTED_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_FRAGMENT_SHADER_INVOCATIONS_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_COMPUTE_SHADER_INVOCATIONS_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_CLIPPING_INPUT_PRIMITIVES_ARB);
    ((*gl_info).gl_ops.ext.p_glEndQuery.unwrap_unchecked())(GL_CLIPPING_OUTPUT_PRIMITIVES_ARB);
    checkGLcall!(gl_info, b"end query\0" as *const u8 as *const ::core::ffi::c_char);
}

unsafe extern "C" fn wined3d_pipeline_query_ops_issue(query: *mut wined3d_query, flags: u32) -> BOOL {
    let mut pq: *mut wined3d_pipeline_statistics_query = wined3d_pipeline_statistics_query_from_query(query);
    let mut device: *mut wined3d_device = (*query).device;
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut poll: BOOL = (FALSE as i32);

    TRACE!(
        b"query %p, flags %#x.\n\0".as_ptr() as *const i8,
        query,
        flags
    );

    if (flags & WINED3DISSUE_BEGIN) != 0 {
        if (*pq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
            if !context_gl.is_null() {
                wined3d_pipeline_statistics_query_end(pq, context_gl);
            } else {
                FIXME!(
                    b"Wrong thread, can't restart query.\n\0".as_ptr() as *const i8
                );
                wined3d_context_gl_free_pipeline_statistics_query(pq);
                context_gl = wined3d_context_gl(context_acquire(
                    device,
                    core::ptr::null_mut(),
                    0,
                ));
                wined3d_context_gl_alloc_pipeline_statistics_query(context_gl, pq);
            }
        } else {
            if !(*pq).context_gl.is_null() {
                wined3d_context_gl_free_pipeline_statistics_query(pq);
            }
            context_gl = wined3d_context_gl(context_acquire(
                device,
                core::ptr::null_mut(),
                0,
            ));
            wined3d_context_gl_alloc_pipeline_statistics_query(context_gl, pq);
        }
        gl_info = (*context_gl).gl_info;

        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_VERTICES_SUBMITTED_ARB, (*pq).u.query.vertices);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_PRIMITIVES_SUBMITTED_ARB, (*pq).u.query.primitives);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_VERTEX_SHADER_INVOCATIONS_ARB, (*pq).u.query.vertex_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_TESS_CONTROL_SHADER_PATCHES_ARB, (*pq).u.query.tess_control_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_TESS_EVALUATION_SHADER_INVOCATIONS_ARB, (*pq).u.query.tess_eval_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_GEOMETRY_SHADER_INVOCATIONS, (*pq).u.query.geometry_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_GEOMETRY_SHADER_PRIMITIVES_EMITTED_ARB, (*pq).u.query.geometry_primitives);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_FRAGMENT_SHADER_INVOCATIONS_ARB, (*pq).u.query.fragment_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_COMPUTE_SHADER_INVOCATIONS_ARB, (*pq).u.query.compute_shader);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_CLIPPING_INPUT_PRIMITIVES_ARB, (*pq).u.query.clipping_input);
        ((*gl_info).gl_ops.ext.p_glBeginQuery.unwrap_unchecked())(GL_CLIPPING_OUTPUT_PRIMITIVES_ARB, (*pq).u.query.clipping_output);
        checkGLcall!(gl_info, b"begin query\0".as_ptr() as *const i8);

        context_release(core::ptr::addr_of_mut!((*context_gl).c));
        (*pq).started = (TRUE as i32);
    }
    if (flags & WINED3DISSUE_END) != 0 {
        if (*pq).started != 0 {
            context_gl = wined3d_context_gl_reacquire((*pq).context_gl);
            if !context_gl.is_null() {
                wined3d_pipeline_statistics_query_end(pq, context_gl);
                context_release(core::ptr::addr_of_mut!((*context_gl).c));
                poll = (TRUE as i32);
            } else {
                FIXME!(
                    b"Wrong thread, can't end query.\n\0".as_ptr() as *const i8
                );
            }
        }
        (*pq).started = (FALSE as i32);
    }

    return poll;
}

unsafe extern "C" fn wined3d_event_query_ops_destroy(query: *mut wined3d_query) {
    let event_query: *mut wined3d_event_query = wined3d_event_query_from_query(query);

    wined3d_fence_free(core::ptr::addr_of_mut!((*event_query).fence));
    free((event_query).cast());
}

unsafe extern "C" fn wined3d_event_query_create(device: *mut wined3d_device, type_: wined3d_query_type, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops, query: *mut *mut wined3d_query) -> HRESULT {
    let gl_info: *const wined3d_gl_info = &raw const (*wined3d_adapter_gl((*device).adapter)).gl_info;
    let mut object: *mut wined3d_event_query;
    let mut hr: HRESULT = 0;

    object = calloc((1 as size_t) as _, (core::mem::size_of::<wined3d_event_query>() as size_t) as _) as *mut wined3d_event_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_fence_init(&raw mut (*object).fence, gl_info);
    if hr < 0 {
        WARN!("Event queries not supported.\n");
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    wined3d_query_init(
        &raw mut (*object).query,
        device,
        type_,
        &raw mut (*object).signalled as *const core::ffi::c_void,
        core::mem::size_of::<BOOL>() as DWORD,
        &raw const event_query_ops,
        parent,
        parent_ops,
    );

    *query = &raw mut (*object).query;

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_occlusion_query_ops_destroy(query: *mut wined3d_query) {
    let oq: *mut wined3d_occlusion_query = wined3d_occlusion_query_from_query(query);
    if !(*oq).context_gl.is_null() {
        wined3d_context_gl_free_occlusion_query(oq);
    }
    free(oq as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_occlusion_query_create(
    device: *mut wined3d_device,
    type_: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info =
        core::ptr::addr_of!((*wined3d_adapter_gl((*device).adapter)).gl_info);
    let mut object: *mut wined3d_occlusion_query = core::ptr::null_mut();

    if (*gl_info).supported[ARB_OCCLUSION_QUERY as usize] == 0 {
        WARN!("Unsupported in local OpenGL implementation: ARB_OCCLUSION_QUERY.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_occlusion_query>() as size_t) as _,
    ) as *mut wined3d_occlusion_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_query_init(
        core::ptr::addr_of_mut!((*object).query),
        device,
        type_,
        core::ptr::addr_of_mut!((*object).samples) as *const core::ffi::c_void,
        core::mem::size_of::<uint64_t>() as DWORD,
        core::ptr::addr_of!(occlusion_query_ops),
        parent,
        parent_ops,
    );

    *query = core::ptr::addr_of_mut!((*object).query);

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_timestamp_query_ops_destroy(query: *mut wined3d_query) {
    let tq: *mut wined3d_timestamp_query = wined3d_timestamp_query_from_query(query);

    if !(*tq).context_gl.is_null() {
        wined3d_context_gl_free_timestamp_query(tq);
    }
    free(tq as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_timestamp_query_create(
    device: *mut wined3d_device,
    type_0: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info =
        core::ptr::addr_of!((*wined3d_adapter_gl((*device).adapter)).gl_info);
    let mut object: *mut wined3d_timestamp_query = core::ptr::null_mut();

    TRACE!(
        "device %p, type %#x, parent %p, parent_ops %p, query %p.\n",
        device,
        type_0 as u32,
        parent,
        parent_ops,
        query
    );

    if (*gl_info).supported[ARB_TIMER_QUERY as usize] == 0 {
        WARN!("Unsupported in local OpenGL implementation: ARB_TIMER_QUERY.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_timestamp_query>() as size_t) as _,
    ) as *mut wined3d_timestamp_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_query_init(
        core::ptr::addr_of_mut!((*object).query),
        device,
        type_0,
        core::ptr::addr_of_mut!((*object).timestamp) as *const core::ffi::c_void,
        core::mem::size_of::<UINT64>() as DWORD,
        core::ptr::addr_of!(timestamp_query_ops),
        parent,
        parent_ops,
    );

    TRACE!("Created query %p.\n", object);

    *query = core::ptr::addr_of_mut!((*object).query);

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_timestamp_disjoint_query_ops_destroy(query: *mut wined3d_query) {
    free((query).cast());
}

unsafe extern "C" fn wined3d_timestamp_disjoint_query_create(
    device: *mut wined3d_device,
    type_0: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info =
        core::ptr::addr_of!((*wined3d_adapter_gl((*device).adapter)).gl_info);
    let mut object: *mut wined3d_query = core::ptr::null_mut();

    if (*gl_info).supported[ARB_TIMER_QUERY as usize] == 0 {
        WARN!("Unsupported in local OpenGL implementation: ARB_TIMER_QUERY.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_query>()) as _,
    ) as *mut wined3d_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    if (type_0 as core::ffi::c_uint)
        == (WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT as core::ffi::c_uint)
    {
        static mut disjoint_data: wined3d_query_data_timestamp_disjoint =
            wined3d_query_data_timestamp_disjoint {
                frequency: (1000
                    * 1000
                    * 1000) as UINT64,
                disjoint: (FALSE as i32),
            };

        wined3d_query_init(
            object,
            device,
            type_0,
            core::ptr::addr_of!(disjoint_data) as *const core::ffi::c_void,
            core::mem::size_of::<wined3d_query_data_timestamp_disjoint>() as DWORD,
            core::ptr::addr_of!(timestamp_disjoint_query_ops),
            parent,
            parent_ops,
        );
    } else {
        static mut freq: UINT64 = (1000
            * 1000
            * 1000) as UINT64;

        wined3d_query_init(
            object,
            device,
            type_0,
            core::ptr::addr_of!(freq) as *const core::ffi::c_void,
            core::mem::size_of::<UINT64>() as DWORD,
            core::ptr::addr_of!(timestamp_disjoint_query_ops),
            parent,
            parent_ops,
        );
    }

    *query = object;

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_so_statistics_query_ops_destroy(query: *mut wined3d_query) {
    let pq: *mut wined3d_so_statistics_query = wined3d_so_statistics_query_from_query(query);
    if !(*pq).context_gl.is_null() {
        wined3d_context_gl_free_so_statistics_query(pq);
    }
    free(pq as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_so_statistics_query_create(
    device: *mut wined3d_device,
    type_0: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info =
        &raw const (*wined3d_adapter_gl((*device).adapter)).gl_info;
    let mut object: *mut wined3d_so_statistics_query = core::ptr::null_mut();
    let mut stream_idx: core::ffi::c_uint;

    if WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0 as core::ffi::c_uint <= type_0 as core::ffi::c_uint
        && type_0 as core::ffi::c_uint <= WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 as core::ffi::c_uint
    {
        stream_idx = (type_0 as core::ffi::c_uint).wrapping_sub(WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0 as core::ffi::c_uint);
    } else if type_0 as core::ffi::c_uint == WINED3D_QUERY_TYPE_SO_STATISTICS as core::ffi::c_uint {
        stream_idx = 0 as core::ffi::c_uint;
    } else {
        return WINED3DERR_NOTAVAILABLE;
    }

    if (*gl_info).supported[WINED3D_GL_PRIMITIVE_QUERY as usize] == 0 {
        WARN!("OpenGL implementation does not support primitive queries.\n");
        return WINED3DERR_NOTAVAILABLE;
    }
    if stream_idx != 0 && (*gl_info).supported[ARB_TRANSFORM_FEEDBACK3 as usize] == 0 {
        WARN!("OpenGL implementation does not support indexed queries.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    object = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_so_statistics_query>()) as _,
    ) as *mut wined3d_so_statistics_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_query_init(
        &raw mut (*object).query,
        device,
        type_0,
        &raw mut (*object).statistics as *const core::ffi::c_void,
        core::mem::size_of::<wined3d_query_data_so_statistics>() as DWORD,
        &raw const so_statistics_query_ops,
        parent,
        parent_ops,
    );
    (*object).stream_idx = stream_idx;

    *query = &raw mut (*object).query;

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_pipeline_query_ops_destroy(query: *mut wined3d_query) {
    let pq: *mut wined3d_pipeline_statistics_query = wined3d_pipeline_statistics_query_from_query(query);
    if !(*pq).context_gl.is_null() {
        wined3d_context_gl_free_pipeline_statistics_query(pq);
    }
    free(pq as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_pipeline_query_create(
    device: *mut wined3d_device,
    type_0: wined3d_query_type,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    query: *mut *mut wined3d_query,
) -> HRESULT {
    let gl_info: *const wined3d_gl_info = &raw const (*wined3d_adapter_gl((*device).adapter)).gl_info;
    let mut object: *mut wined3d_pipeline_statistics_query = core::ptr::null_mut();

    if (*gl_info).supported[ARB_PIPELINE_STATISTICS_QUERY as usize] == 0 {
        WARN!("OpenGL implementation does not support pipeline statistics queries.\n");
        return WINED3DERR_NOTAVAILABLE;
    }

    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<wined3d_pipeline_statistics_query>() as size_t) as _,
    ) as *mut wined3d_pipeline_statistics_query;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    wined3d_query_init(
        &raw mut (*object).query,
        device,
        type_0,
        &raw mut (*object).statistics as *mut core::ffi::c_void,
        core::mem::size_of::<wined3d_query_data_pipeline_statistics>() as DWORD,
        &raw const pipeline_query_ops,
        parent,
        parent_ops,
    );

    *query = &raw mut (*object).query;

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_gl_create(mut device: *mut wined3d_device, mut type_0: wined3d_query_type, mut parent: *mut ::core::ffi::c_void, mut parent_ops: *const wined3d_parent_ops, mut query: *mut *mut wined3d_query) -> HRESULT {
    TRACE!(
        "device %p, type %#x, parent %p, parent_ops %p, query %p.\n",
        device,
        type_0 as ::core::ffi::c_uint,
        parent,
        parent_ops,
        query
    );
    match type_0 as ::core::ffi::c_uint {
        WINED3D_QUERY_TYPE_EVENT => {
            return wined3d_event_query_create(device, type_0, parent, parent_ops, query);
        }
        WINED3D_QUERY_TYPE_OCCLUSION => {
            return wined3d_occlusion_query_create(device, type_0, parent, parent_ops, query);
        }
        WINED3D_QUERY_TYPE_TIMESTAMP => {
            return wined3d_timestamp_query_create(device, type_0, parent, parent_ops, query);
        }
        WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT | WINED3D_QUERY_TYPE_TIMESTAMP_FREQ => {
            return wined3d_timestamp_disjoint_query_create(device, type_0, parent, parent_ops, query);
        }
        WINED3D_QUERY_TYPE_SO_STATISTICS
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 => {
            return wined3d_so_statistics_query_create(device, type_0, parent, parent_ops, query);
        }
        WINED3D_QUERY_TYPE_PIPELINE_STATISTICS => {
            return wined3d_pipeline_query_create(device, type_0, parent, parent_ops, query);
        }
        _ => {
            FIXME!(
                "Unhandled query type %#x.\n",
                type_0 as ::core::ffi::c_uint
            );
            return WINED3DERR_NOTAVAILABLE;
        }
    }
}

unsafe extern "C" fn wined3d_query_pool_vk_mark_complete(
    mut pool_vk: *mut wined3d_query_pool_vk,
    mut idx: size_t,
    mut context_vk: *mut wined3d_context_vk,
) {
    wined3d_bitmap_set(
        core::ptr::addr_of_mut!((*pool_vk).completed) as *mut uint32_t,
        idx as ::core::ffi::c_uint,
    );
    if list_empty(core::ptr::addr_of_mut!((*pool_vk).completed_entry)) != 0 {
        list_add_tail(
            core::ptr::addr_of_mut!((*context_vk).completed_query_pools),
            core::ptr::addr_of_mut!((*pool_vk).completed_entry),
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_pool_vk_allocate_query(pool_vk: *mut wined3d_query_pool_vk, idx: *mut size_t) -> bool {
    *idx = wined3d_bitmap_ffz(core::ptr::addr_of_mut!((*pool_vk).allocated) as *mut uint32_t, WINED3D_QUERY_POOL_SIZE as core::ffi::c_uint, 0 as core::ffi::c_uint) as size_t;
    if *idx > WINED3D_QUERY_POOL_SIZE as size_t {
        return false;
    }
    wined3d_bitmap_set(core::ptr::addr_of_mut!((*pool_vk).allocated) as *mut uint32_t, *idx as core::ffi::c_uint);

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_pool_vk_mark_free(
    context_vk: *mut wined3d_context_vk,
    pool_vk: *mut wined3d_query_pool_vk,
    start: uint32_t,
    count: uint32_t,
) {
    let mut idx: ::core::ffi::c_uint;
    let end: ::core::ffi::c_uint = (start as ::core::ffi::c_uint).wrapping_add(count as ::core::ffi::c_uint);

    idx = start as ::core::ffi::c_uint;
    while idx < end {
        wined3d_bitmap_clear(core::ptr::addr_of_mut!((*pool_vk).allocated) as *mut uint32_t, idx);
        idx = idx.wrapping_add(1);
    }

    if list_empty(core::ptr::addr_of_mut!((*pool_vk).entry)) != 0 {
        list_add_tail((*pool_vk).free_list, core::ptr::addr_of_mut!((*pool_vk).entry));
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_pool_vk_cleanup(pool_vk: *mut wined3d_query_pool_vk, context_vk: *mut wined3d_context_vk) {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;

    ((*vk_info).vk_ops.vkDestroyQueryPool.unwrap_unchecked())((*device_vk).vk_device, (*pool_vk).vk_query_pool, core::ptr::null_mut());
    if (*pool_vk).vk_event != 0 {
        ((*vk_info).vk_ops.vkDestroyEvent.unwrap_unchecked())((*device_vk).vk_device, (*pool_vk).vk_event, core::ptr::null_mut());
    }
    list_remove(core::ptr::addr_of_mut!((*pool_vk).entry));
    list_remove(core::ptr::addr_of_mut!((*pool_vk).completed_entry));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_pool_vk_init(
    pool_vk: *mut wined3d_query_pool_vk,
    context_vk: *mut wined3d_context_vk,
    type_0: wined3d_query_type,
    free_pools: *mut list,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*context_vk).c.device);
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pool_info: VkQueryPoolCreateInfo = core::mem::zeroed();
    let mut vr: VkResult;

    list_init(core::ptr::addr_of_mut!((*pool_vk).entry));
    list_init(core::ptr::addr_of_mut!((*pool_vk).completed_entry));
    (*pool_vk).free_list = free_pools;

    pool_info.sType = VK_STRUCTURE_TYPE_QUERY_POOL_CREATE_INFO;
    pool_info.pNext = core::ptr::null();
    pool_info.flags = 0;
    pool_info.queryCount = WINED3D_QUERY_POOL_SIZE;

    match type_0 {
        WINED3D_QUERY_TYPE_OCCLUSION => {
            pool_info.queryType = VK_QUERY_TYPE_OCCLUSION;
            pool_info.pipelineStatistics = 0;
        }

        WINED3D_QUERY_TYPE_TIMESTAMP => {
            pool_info.queryType = VK_QUERY_TYPE_TIMESTAMP;
            pool_info.pipelineStatistics = 0;
        }

        WINED3D_QUERY_TYPE_PIPELINE_STATISTICS => {
            pool_info.queryType = VK_QUERY_TYPE_PIPELINE_STATISTICS;
            pool_info.pipelineStatistics = VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_VERTICES_BIT
                | VK_QUERY_PIPELINE_STATISTIC_INPUT_ASSEMBLY_PRIMITIVES_BIT
                | VK_QUERY_PIPELINE_STATISTIC_VERTEX_SHADER_INVOCATIONS_BIT
                | VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_INVOCATIONS_BIT
                | VK_QUERY_PIPELINE_STATISTIC_GEOMETRY_SHADER_PRIMITIVES_BIT
                | VK_QUERY_PIPELINE_STATISTIC_CLIPPING_INVOCATIONS_BIT
                | VK_QUERY_PIPELINE_STATISTIC_CLIPPING_PRIMITIVES_BIT
                | VK_QUERY_PIPELINE_STATISTIC_FRAGMENT_SHADER_INVOCATIONS_BIT
                | VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_CONTROL_SHADER_PATCHES_BIT
                | VK_QUERY_PIPELINE_STATISTIC_TESSELLATION_EVALUATION_SHADER_INVOCATIONS_BIT
                | VK_QUERY_PIPELINE_STATISTIC_COMPUTE_SHADER_INVOCATIONS_BIT;
        }

        WINED3D_QUERY_TYPE_SO_STATISTICS
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 => {
            pool_info.queryType = VK_QUERY_TYPE_TRANSFORM_FEEDBACK_STREAM_EXT;
            pool_info.pipelineStatistics = 0;
        }

        _ => {
            FIXME!("Unhandled query type %#x.\n", type_0);
            return false;
        }
    }

    vr = ((*vk_info).vk_ops.vkCreateQueryPool.unwrap_unchecked())(
        (*device_vk).vk_device,
        &mut pool_info,
        core::ptr::null(),
        core::ptr::addr_of_mut!((*pool_vk).vk_query_pool),
    );
    if (vr as core::ffi::c_int) < 0 {
        ERR!(
            "Failed to create Vulkan query pool, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return false;
    }

    list_add_head(free_pools, core::ptr::addr_of_mut!((*pool_vk).entry));

    return true;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_vk_accumulate_data(
    mut query_vk: *mut wined3d_query_vk,
    mut device_vk: *mut wined3d_device_vk,
    mut pool_idx: *const wined3d_query_pool_idx_vk,
) -> bool {
    let mut ps_tmp: *const wined3d_query_data_pipeline_statistics =
        core::ptr::null::<wined3d_query_data_pipeline_statistics>();
    let mut vk_info: *const wined3d_vk_info = core::ptr::addr_of_mut!((*device_vk).vk_info);
    let mut ps_result: *mut wined3d_query_data_pipeline_statistics =
        core::ptr::null_mut::<wined3d_query_data_pipeline_statistics>();
    let mut so_result: *mut wined3d_query_data_so_statistics =
        core::ptr::null_mut::<wined3d_query_data_so_statistics>();
    let mut so_tmp: *const wined3d_query_data_so_statistics =
        core::ptr::null::<wined3d_query_data_so_statistics>();
    let mut vr: VkResult = VK_SUCCESS;
    let mut tmp: [u64; 11] = [0; 11];
    let mut result: *mut [u64; 11] = core::ptr::null_mut::<[u64; 11]>();

    if !((*(*pool_idx).pool_vk).vk_event == 0) {
        /* Check if the pool's initial reset command executed. */
        vr = ((*vk_info).vk_ops.vkGetEventStatus.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*(*pool_idx).pool_vk).vk_event,
        );
        if vr as i32 == VK_EVENT_RESET {
            return false;
        } else if vr as i32 != VK_EVENT_SET {
            ERR!(
                "Failed to get event status, vr %s\n",
                wined3d_debug_vkresult(vr)
            );
            return false;
        }
    }

    vr = ((*vk_info).vk_ops.vkGetQueryPoolResults.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*(*pool_idx).pool_vk).vk_query_pool,
        (*pool_idx).idx as u32,
        1 as u32,
        core::mem::size_of::<[u64; 11]>() as usize,
        core::ptr::addr_of_mut!(tmp) as *mut core::ffi::c_void,
        core::mem::size_of::<[u64; 11]>() as u64,
        VK_QUERY_RESULT_64_BIT as u32,
    );
    if (vr as i32) < 0 {
        ERR!(
            "Failed to get query results, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return false;
    }

    if vr as i32 == VK_NOT_READY {
        return false;
    }

    result = (*query_vk).q.data as *mut [u64; 11];
    match (*query_vk).q.type_ as u32 {
        WINED3D_QUERY_TYPE_OCCLUSION => {
            (*result)[0] = (*result)[0].wrapping_add(tmp[0]);
        }

        WINED3D_QUERY_TYPE_TIMESTAMP => {
            (*result)[0] = tmp[0];
        }

        WINED3D_QUERY_TYPE_PIPELINE_STATISTICS => {
            ps_result = core::ptr::addr_of_mut!((*result))
                as *mut wined3d_query_data_pipeline_statistics;
            ps_tmp = core::ptr::addr_of_mut!(tmp) as *const wined3d_query_data_pipeline_statistics;
            (*ps_result).vertices_submitted = (*ps_result)
                .vertices_submitted
                .wrapping_add((*ps_tmp).vertices_submitted);
            (*ps_result).primitives_submitted = (*ps_result)
                .primitives_submitted
                .wrapping_add((*ps_tmp).primitives_submitted);
            (*ps_result).vs_invocations = (*ps_result)
                .vs_invocations
                .wrapping_add((*ps_tmp).vs_invocations);
            (*ps_result).gs_invocations = (*ps_result)
                .gs_invocations
                .wrapping_add((*ps_tmp).gs_invocations);
            (*ps_result).gs_primitives = (*ps_result)
                .gs_primitives
                .wrapping_add((*ps_tmp).gs_primitives);
            (*ps_result).clipping_input_primitives = (*ps_result)
                .clipping_input_primitives
                .wrapping_add((*ps_tmp).clipping_input_primitives);
            (*ps_result).clipping_output_primitives = (*ps_result)
                .clipping_output_primitives
                .wrapping_add((*ps_tmp).clipping_output_primitives);
            (*ps_result).ps_invocations = (*ps_result)
                .ps_invocations
                .wrapping_add((*ps_tmp).ps_invocations);
            (*ps_result).hs_invocations = (*ps_result)
                .hs_invocations
                .wrapping_add((*ps_tmp).hs_invocations);
            (*ps_result).ds_invocations = (*ps_result)
                .ds_invocations
                .wrapping_add((*ps_tmp).ds_invocations);
            (*ps_result).cs_invocations = (*ps_result)
                .cs_invocations
                .wrapping_add((*ps_tmp).cs_invocations);
        }

        WINED3D_QUERY_TYPE_SO_STATISTICS
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 => {
            so_result = core::ptr::addr_of_mut!((*result)) as *mut wined3d_query_data_so_statistics;
            so_tmp = core::ptr::addr_of_mut!(tmp) as *const wined3d_query_data_so_statistics;
            (*so_result).primitives_written = (*so_result)
                .primitives_written
                .wrapping_add((*so_tmp).primitives_written);
            (*so_result).primitives_generated = (*so_result)
                .primitives_generated
                .wrapping_add((*so_tmp).primitives_generated);
        }

        _ => {
            FIXME!(
                "Unhandled query type %#x.\n",
                (*query_vk).q.type_ as u32
            );
            return false;
        }
    }

    return true;
}

unsafe extern "C" fn wined3d_query_vk_begin(
    mut query_vk: *mut wined3d_query_vk,
    mut context_vk: *mut wined3d_context_vk,
    mut vk_command_buffer: VkCommandBuffer,
) {
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pool_vk: *mut wined3d_query_pool_vk = ::core::ptr::null_mut::<wined3d_query_pool_vk>();
    let mut idx: size_t = 0;
    pool_vk = (*query_vk).pool_idx.pool_vk;
    idx = (*query_vk).pool_idx.idx;
    if (*query_vk).q.type_ as ::core::ffi::c_uint
        >= WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1 as ::core::ffi::c_uint
        && (*query_vk).q.type_ as ::core::ffi::c_uint
            <= WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 as ::core::ffi::c_uint
    {
        (*vk_info)
            .vk_ops
            .vkCmdBeginQueryIndexedEXT
            .unwrap_unchecked()(
            vk_command_buffer,
            (*pool_vk).vk_query_pool,
            idx as uint32_t,
            (*query_vk).control_flags as VkQueryControlFlags,
            ((*query_vk).q.type_ as uint32_t).wrapping_sub(
                WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0 as uint32_t,
            ),
        );
    } else {
        (*vk_info)
            .vk_ops
            .vkCmdBeginQuery
            .unwrap_unchecked()(
            vk_command_buffer,
            (*pool_vk).vk_query_pool,
            idx as uint32_t,
            (*query_vk).control_flags as VkQueryControlFlags,
        );
    }
    wined3d_context_vk_reference_query(context_vk, query_vk);
}

unsafe extern "C" fn wined3d_query_vk_end(
    query_vk: *mut wined3d_query_vk,
    context_vk: *mut wined3d_context_vk,
    vk_command_buffer: VkCommandBuffer,
) {
    let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut pool_vk: *mut wined3d_query_pool_vk = core::ptr::null_mut();
    let mut idx: usize = 0;

    pool_vk = (*query_vk).pool_idx.pool_vk;
    idx = (*query_vk).pool_idx.idx;

    if ((*query_vk).q.type_ as u32) >= WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        && ((*query_vk).q.type_ as u32) <= WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3
    {
        ((*vk_info).vk_ops.vkCmdEndQueryIndexedEXT.unwrap_unchecked())(
            vk_command_buffer,
            (*pool_vk).vk_query_pool,
            idx as u32,
            ((*query_vk).q.type_ as u32).wrapping_sub(WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0),
        );
    } else {
        ((*vk_info).vk_ops.vkCmdEndQuery.unwrap_unchecked())(
            vk_command_buffer,
            (*pool_vk).vk_query_pool,
            idx as u32,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_vk_resume(query_vk: *mut wined3d_query_vk, context_vk: *mut wined3d_context_vk) {
    let mut vk_command_buffer: VkCommandBuffer = (*context_vk).current_command_buffer.vk_command_buffer;

    wined3d_query_vk_begin(query_vk, context_vk, vk_command_buffer);
    (*query_vk).flags = ((*query_vk).flags as ::core::ffi::c_int | (WINED3D_QUERY_VK_FLAG_ACTIVE as i32)) as uint8_t;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_vk_suspend(query_vk: *mut wined3d_query_vk, context_vk: *mut wined3d_context_vk) {
    let vk_command_buffer: VkCommandBuffer = (*context_vk).current_command_buffer.vk_command_buffer;

    wined3d_query_vk_end(query_vk, context_vk, vk_command_buffer);

    if wined3d_array_reserve(
        core::ptr::addr_of_mut!((*query_vk).pending) as *mut *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*query_vk).pending_size),
        (*query_vk).pending_count.wrapping_add(1),
        core::mem::size_of::<wined3d_query_pool_idx_vk>() as SIZE_T,
    ) == 0
    {
        ERR!("Failed to allocate entry.\n");
        return;
    }

    let fresh20 = (*query_vk).pending_count;
    (*query_vk).pending_count = (*query_vk).pending_count.wrapping_add(1);
    *(*query_vk).pending.offset(fresh20 as isize) = (*query_vk).pool_idx;
    (*query_vk).pool_idx.pool_vk = core::ptr::null_mut::<wined3d_query_pool_vk>();
    (*query_vk).flags =
        (((*query_vk).flags as core::ffi::c_int) & !(WINED3D_QUERY_VK_FLAG_ACTIVE as core::ffi::c_int)) as u8;
}

unsafe extern "C" fn wined3d_query_vk_poll(mut query: *mut wined3d_query, mut flags: uint32_t) -> BOOL {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*query).device);
    let mut query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let mut i: ::core::ffi::c_uint = 0;

    memset(
        (*query).data as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (*query).data_size as size_t,
    );

    'unavailable: {
        if !(*query_vk).pool_idx.pool_vk.is_null()
            && !wined3d_query_vk_accumulate_data(
                query_vk,
                device_vk,
                core::ptr::addr_of_mut!((*query_vk).pool_idx),
            )
        {
            break 'unavailable;
        }

        i = 0 as ::core::ffi::c_uint;
        while (i as SIZE_T) < (*query_vk).pending_count {
            if !wined3d_query_vk_accumulate_data(
                query_vk,
                device_vk,
                (*query_vk).pending.offset(i as isize) as *mut wined3d_query_pool_idx_vk,
            ) {
                break 'unavailable;
            }
            i = i.wrapping_add(1);
        }

        return TRUE as BOOL;
    }

    // unavailable:
    if (flags & WINED3DGETDATA_FLUSH as uint32_t) != 0
        && (*(*(*query).device).cs).queries_flushed == 0
    {
        (*(*(*(*query).device).cs).c.ops)
            .flush
            .unwrap_unchecked()(core::ptr::addr_of_mut!(
            (*(*(*query).device).cs).c
        ));
    }

    return FALSE as BOOL;
}

unsafe extern "C" fn wined3d_query_vk_remove_pending_queries(
    mut context_vk: *mut wined3d_context_vk,
    mut query_vk: *mut wined3d_query_vk,
) {
    let mut i: size_t = 0;
    while i < ((*query_vk).pending_count) as _ {
        wined3d_query_pool_vk_mark_complete(
            (*(*query_vk).pending.offset(i as isize)).pool_vk,
            (*(*query_vk).pending.offset(i as isize)).idx,
            context_vk,
        );
        i = i.wrapping_add(1);
    }
    (*query_vk).pending_count = 0;
}

unsafe extern "C" fn wined3d_query_vk_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*query).device);
    let mut query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut::<wined3d_context_vk>();
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut::<VkCommandBuffer_T>();
    let mut poll: bool = false;

    TRACE!("query %p, flags %#x.\n", query, flags);

    if (flags & WINED3DISSUE_BEGIN) != 0 {
        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut::<wined3d_texture>(),
            0,
        ));

        if (*query_vk).pending_count != 0 {
            wined3d_query_vk_remove_pending_queries(context_vk, query_vk);
        }
        vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
        if ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_STARTED as i32)) != 0 {
            if ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_ACTIVE as i32)) != 0 {
                wined3d_query_vk_end(query_vk, context_vk, vk_command_buffer);
            }
            list_remove(core::ptr::addr_of_mut!((*query_vk).entry));
        }
        if !(*query_vk).pool_idx.pool_vk.is_null() {
            wined3d_query_pool_vk_mark_complete(
                (*query_vk).pool_idx.pool_vk,
                (*query_vk).pool_idx.idx,
                context_vk,
            );
        }

        if !wined3d_context_vk_allocate_query(
            context_vk,
            (*query_vk).q.type_,
            core::ptr::addr_of_mut!((*query_vk).pool_idx),
        ) {
            ERR!("Failed to allocate new query.\n");
            return false as BOOL;
        }

        /* A query needs to either begin and end inside a single render pass
         * or begin and end outside of a render pass. Occlusion queries, if
         * issued outside of a render pass, are queued up and only begun when
         * a render pass is started, to avoid interrupting the render pass
         * when the query ends. */
        if (*context_vk).vk_render_pass != 0 {
            wined3d_query_vk_begin(query_vk, context_vk, vk_command_buffer);
            list_add_head(
                core::ptr::addr_of_mut!((*context_vk).render_pass_queries),
                core::ptr::addr_of_mut!((*query_vk).entry),
            );
            (*query_vk).flags = (((*query_vk).flags as ::core::ffi::c_int)
                | ((WINED3D_QUERY_VK_FLAG_ACTIVE | WINED3D_QUERY_VK_FLAG_RENDER_PASS)) as i32)
                as uint8_t;
        } else if (*query).type_ as ::core::ffi::c_uint
            == WINED3D_QUERY_TYPE_OCCLUSION as ::core::ffi::c_uint
        {
            list_add_head(
                core::ptr::addr_of_mut!((*context_vk).render_pass_queries),
                core::ptr::addr_of_mut!((*query_vk).entry),
            );
            (*query_vk).flags = (((*query_vk).flags as ::core::ffi::c_int)
                | (WINED3D_QUERY_VK_FLAG_RENDER_PASS as i32)) as uint8_t;
        } else {
            wined3d_query_vk_begin(query_vk, context_vk, vk_command_buffer);
            list_add_head(
                core::ptr::addr_of_mut!((*context_vk).active_queries),
                core::ptr::addr_of_mut!((*query_vk).entry),
            );
            (*query_vk).flags = (((*query_vk).flags as ::core::ffi::c_int)
                | (WINED3D_QUERY_VK_FLAG_ACTIVE as i32)) as uint8_t;
        }

        (*query_vk).flags =
            (((*query_vk).flags as ::core::ffi::c_int) | (WINED3D_QUERY_VK_FLAG_STARTED as i32)) as uint8_t;
        context_release(core::ptr::addr_of_mut!((*context_vk).c));
    }
    if ((flags & WINED3DISSUE_END) != 0)
        && ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_STARTED as i32)) != 0
    {
        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut::<wined3d_texture>(),
            0,
        ));

        /* If the query was already ended because the command buffer was
         * flushed or the render pass ended, we don't need to end it here. */
        if ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_ACTIVE as i32)) != 0 {
            vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
        }

        /* wined3d_context_vk_get_command_buffer() might have triggered
         * submission of the previous command buffer (due to retired
         * resource accumulation or periodic submit).
         * This will have suspended the query, and if it's a render pass
         * query, it won't have been resumed and therefore shouldn't be ended
         * here. Therefore we need to check whether it's active again. */
        if ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_ACTIVE as i32)) != 0 {
            if ((*query_vk).flags as ::core::ffi::c_int & (WINED3D_QUERY_VK_FLAG_RENDER_PASS as i32)) == 0 {
                wined3d_context_vk_end_current_render_pass(context_vk);
            }
            wined3d_query_vk_end(query_vk, context_vk, vk_command_buffer);
        } else if !(*query_vk).pool_idx.pool_vk.is_null() {
            /* It was queued, but never activated. */
            wined3d_query_pool_vk_mark_complete(
                (*query_vk).pool_idx.pool_vk,
                (*query_vk).pool_idx.idx,
                context_vk,
            );
            (*query_vk).pool_idx.pool_vk = core::ptr::null_mut::<wined3d_query_pool_vk>();
        }
        list_remove(core::ptr::addr_of_mut!((*query_vk).entry));
        (*query_vk).flags = 0;
        poll = true;

        context_release(core::ptr::addr_of_mut!((*context_vk).c));
    }

    return poll as BOOL;
}

unsafe extern "C" fn wined3d_query_vk_destroy(query: *mut wined3d_query) {
    let query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let mut context_vk: *mut wined3d_context_vk;

    if ((*query_vk).flags as uint32_t) & (WINED3D_QUERY_VK_FLAG_STARTED as uint32_t) != 0 {
        list_remove(core::ptr::addr_of_mut!((*query_vk).entry));
    }
    context_vk = wined3d_context_vk(context_acquire((*query_vk).q.device, core::ptr::null_mut(), 0));
    wined3d_query_vk_remove_pending_queries(context_vk, query_vk);
    if !(*query_vk).pool_idx.pool_vk.is_null() {
        wined3d_query_pool_vk_mark_complete((*query_vk).pool_idx.pool_vk, (*query_vk).pool_idx.idx, context_vk);
    }
    if !((*query_vk).vk_event == 0) {
        wined3d_context_vk_destroy_vk_event(context_vk, (*query_vk).vk_event, (*query_vk).command_buffer_id);
    }
    context_release(core::ptr::addr_of_mut!((*context_vk).c));
    free((*query_vk).pending as *mut core::ffi::c_void);
    free(query_vk as *mut core::ffi::c_void);
}

unsafe extern "C" fn wined3d_query_event_vk_poll(
    query: *mut wined3d_query,
    flags: uint32_t,
) -> BOOL {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*query).device);
    let query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let vk_info: *const wined3d_vk_info = &raw const (*device_vk).vk_info;
    let mut signalled: BOOL;

    signalled = (((*vk_info).vk_ops.vkGetEventStatus.unwrap_unchecked())((*device_vk).vk_device, (*query_vk).vk_event)
        == VK_EVENT_SET) as BOOL;
    if signalled == 0
        && (flags & WINED3DGETDATA_FLUSH as uint32_t) != 0
        && (*(*(*query).device).cs).queries_flushed == 0
    {
        (*(*(*(*query).device).cs).c.ops)
            .flush
            .unwrap_unchecked()(&raw mut (*(*(*query).device).cs).c);
    }
    let data: *mut BOOL = (*query).data as *mut BOOL;
    *data = signalled;
    return *data;
}

unsafe extern "C" fn wined3d_query_event_vk_issue(mut query: *mut wined3d_query, mut flags: uint32_t) -> BOOL {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*query).device);
    let mut vk_info: *const wined3d_vk_info = core::ptr::addr_of!((*device_vk).vk_info);
    let mut query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut();
    let mut create_info: VkEventCreateInfo = core::mem::zeroed();
    let mut vr: VkResult = core::mem::zeroed();

    TRACE!("query %p, flags %#x.\n", query, flags);

    if (flags & WINED3DISSUE_END) != 0 {
        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut(),
            0,
        ));
        wined3d_context_vk_end_current_render_pass(context_vk);

        if (*query_vk).vk_event != 0 {
            if (*query_vk).command_buffer_id > (*context_vk).completed_command_buffer_id {
                /* Cannot reuse this event, as it may still get signalled by previous usage. */
                /* Throw it away and create a new one, but if that happens a lot we may want to pool instead. */
                wined3d_context_vk_destroy_vk_event(context_vk, (*query_vk).vk_event, (*query_vk).command_buffer_id);
                (*query_vk).vk_event = (VK_NULL_HANDLE as _);
            } else {
                ((*vk_info).vk_ops.vkResetEvent.unwrap_unchecked())((*device_vk).vk_device, (*query_vk).vk_event);
            }
        }

        if (*query_vk).vk_event == 0 {
            create_info.sType = VK_STRUCTURE_TYPE_EVENT_CREATE_INFO;
            create_info.pNext = core::ptr::null();
            create_info.flags = 0;

            vr = ((*vk_info).vk_ops.vkCreateEvent.unwrap_unchecked())(
                (*device_vk).vk_device,
                core::ptr::addr_of_mut!(create_info),
                core::ptr::null(),
                core::ptr::addr_of_mut!((*query_vk).vk_event),
            );
            if vr != VK_SUCCESS {
                ERR!("Failed to create Vulkan event, vr %s\n", wined3d_debug_vkresult(vr));
                context_release(core::ptr::addr_of_mut!((*context_vk).c));
                return (FALSE) as BOOL;
            }
        }

        wined3d_context_vk_reference_query(context_vk, query_vk);
        ((*vk_info).vk_ops.vkCmdSetEvent.unwrap_unchecked())(
            wined3d_context_vk_get_command_buffer(context_vk),
            (*query_vk).vk_event,
            VK_PIPELINE_STAGE_ALL_COMMANDS_BIT,
        );
        context_release(core::ptr::addr_of_mut!((*context_vk).c));

        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_query_timestamp_vk_issue(query: *mut wined3d_query, flags: uint32_t) -> BOOL {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*query).device);
    let query_vk: *mut wined3d_query_vk = wined3d_query_vk(query);
    let mut vk_info: *const wined3d_vk_info;
    let mut context_vk: *mut wined3d_context_vk;
    let mut command_buffer: VkCommandBuffer;

    if (flags & WINED3DISSUE_BEGIN) != 0 {
        TRACE!("Ignoring WINED3DISSUE_BEGIN.\n");
    }

    if (flags & WINED3DISSUE_END) != 0 {
        context_vk = wined3d_context_vk(context_acquire(
            core::ptr::addr_of_mut!((*device_vk).d),
            core::ptr::null_mut(),
            0,
        ));
        vk_info = (*context_vk).vk_info;

        command_buffer = wined3d_context_vk_get_command_buffer(context_vk);
        if !(*query_vk).pool_idx.pool_vk.is_null() {
            wined3d_query_pool_vk_mark_complete((*query_vk).pool_idx.pool_vk, (*query_vk).pool_idx.idx, context_vk);
        }
        if wined3d_context_vk_allocate_query(
            context_vk,
            (*query_vk).q.type_,
            core::ptr::addr_of_mut!((*query_vk).pool_idx),
        ) == false
        {
            ERR!("Failed to allocate new query.\n");
            return (FALSE) as BOOL;
        }
        ((*vk_info).vk_ops.vkCmdWriteTimestamp.unwrap_unchecked())(
            command_buffer,
            VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
            (*(*query_vk).pool_idx.pool_vk).vk_query_pool,
            (*query_vk).pool_idx.idx as uint32_t,
        );
        wined3d_context_vk_reference_query(context_vk, query_vk);

        context_release(core::ptr::addr_of_mut!((*context_vk).c));

        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_query_vk_create(device: *mut wined3d_device, type_0: wined3d_query_type, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops, query: *mut *mut wined3d_query) -> HRESULT {
    let mut disjoint_data: *mut wined3d_query_data_timestamp_disjoint = core::ptr::null_mut();
    let mut ops: *const wined3d_query_ops = core::ptr::addr_of!(wined3d_query_vk_ops);
    let mut query_vk: *mut wined3d_query_vk = core::ptr::null_mut();
    let mut data_size: u32 = 0;
    let mut data: *mut ::core::ffi::c_void = core::ptr::null_mut();

    TRACE!(
        "device %p, type %#x, parent %p, parent_ops %p, query %p.\n",
        device,
        type_0 as u32,
        parent,
        parent_ops,
        query
    );

    match type_0 as u32 {
        WINED3D_QUERY_TYPE_EVENT => {
            ops = core::ptr::addr_of!(wined3d_query_event_vk_ops);
            data_size = core::mem::size_of::<BOOL>() as u32;
        }
        WINED3D_QUERY_TYPE_OCCLUSION => {
            data_size = core::mem::size_of::<uint64_t>() as u32;
        }
        WINED3D_QUERY_TYPE_TIMESTAMP => {
            if (*wined3d_device_vk(device)).timestamp_bits == 0 {
                WARN!("Timestamp queries not supported.\n");
                return WINED3DERR_NOTAVAILABLE;
            }
            ops = core::ptr::addr_of!(wined3d_query_timestamp_vk_ops);
            data_size = core::mem::size_of::<uint64_t>() as u32;
        }
        WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT => {
            if (*wined3d_device_vk(device)).timestamp_bits == 0 {
                WARN!("Timestamp queries not supported.\n");
                return WINED3DERR_NOTAVAILABLE;
            }
            ops = core::ptr::addr_of!(wined3d_query_timestamp_disjoint_vk_ops);
            data_size = core::mem::size_of::<wined3d_query_data_timestamp_disjoint>() as u32;
        }
        WINED3D_QUERY_TYPE_PIPELINE_STATISTICS => {
            data_size = core::mem::size_of::<wined3d_query_data_pipeline_statistics>() as u32;
        }
        WINED3D_QUERY_TYPE_SO_STATISTICS
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM0
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM1
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM2
        | WINED3D_QUERY_TYPE_SO_STATISTICS_STREAM3 => {
            if (*wined3d_adapter_vk((*device).adapter)).vk_info.supported
                [WINED3D_VK_EXT_TRANSFORM_FEEDBACK as usize]
                == 0
            {
                WARN!("Stream output queries not supported.\n");
                return WINED3DERR_NOTAVAILABLE;
            }
            data_size = core::mem::size_of::<wined3d_query_data_so_statistics>() as u32;
        }
        _ => {
            FIXME!("Unhandled query type %#x.\n", type_0 as u32);
            return WINED3DERR_NOTAVAILABLE;
        }
    }

    query_vk = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_query_vk>().wrapping_add(data_size as usize)) as _,
    ) as *mut wined3d_query_vk;
    if query_vk.is_null() {
        return E_OUTOFMEMORY;
    }
    data = query_vk.offset(1 as isize) as *mut ::core::ffi::c_void;

    wined3d_query_init(
        core::ptr::addr_of_mut!((*query_vk).q),
        device,
        type_0,
        data,
        data_size as DWORD,
        ops,
        parent,
        parent_ops,
    );
    (*query_vk).q.poll_in_cs = false;

    match type_0 as u32 {
        WINED3D_QUERY_TYPE_OCCLUSION => {
            (*query_vk).control_flags = VK_QUERY_CONTROL_PRECISE_BIT as uint32_t;
        }
        WINED3D_QUERY_TYPE_TIMESTAMP_DISJOINT => {
            disjoint_data = data as *mut wined3d_query_data_timestamp_disjoint;
            (*disjoint_data).frequency = (1000000000 as f32
                / (*wined3d_adapter_vk((*device).adapter)).device_limits.timestampPeriod)
                as UINT64;
            (*disjoint_data).disjoint = FALSE as BOOL;
        }
        _ => {}
    }

    TRACE!("Created query %p.\n", query_vk);

    *query = core::ptr::addr_of_mut!((*query_vk).q);

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_query_create(
    mut device: *mut wined3d_device,
    mut type_0: wined3d_query_type,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut query: *mut *mut wined3d_query,
) -> HRESULT {
    WINE_TRACE!("device %p, type %#x, parent %p, parent_ops %p, query %p.\n", device, type_0 as ::core::ffi::c_uint, parent, parent_ops, query);
    return (*(*(*device).adapter).adapter_ops)
        .adapter_create_query
        .unwrap_unchecked()(device, type_0, parent, parent_ops, query);
}
