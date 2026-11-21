use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut WINED3D_READ_ONLY_BIND_MASK: uint32_t = (WINED3D_BIND_VERTEX_BUFFER
    | WINED3D_BIND_INDEX_BUFFER
    | WINED3D_BIND_CONSTANT_BUFFER
    | WINED3D_BIND_SHADER_RESOURCE
    | WINED3D_BIND_INDIRECT_BUFFER) as uint32_t;

static mut buffer_resource_ops: wined3d_resource_ops = unsafe {
    wined3d_resource_ops {
        resource_incref: Some(
            buffer_resource_incref as unsafe extern "C" fn(*mut wined3d_resource) -> ULONG,
        ),
        resource_decref: Some(
            buffer_resource_decref as unsafe extern "C" fn(*mut wined3d_resource) -> ULONG,
        ),
        resource_preload: Some(
            buffer_resource_preload as unsafe extern "C" fn(*mut wined3d_resource) -> (),
        ),
        resource_unload: Some(
            buffer_resource_unload as unsafe extern "C" fn(*mut wined3d_resource) -> (),
        ),
        resource_sub_resource_get_desc: Some(
            buffer_resource_sub_resource_get_desc
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut wined3d_sub_resource_desc,
                ) -> HRESULT,
        ),
        resource_sub_resource_get_map_pitch: Some(
            buffer_resource_sub_resource_get_map_pitch
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut ::core::ffi::c_uint,
                    *mut ::core::ffi::c_uint,
                ) -> (),
        ),
        resource_sub_resource_map: Some(
            buffer_resource_sub_resource_map
                as unsafe extern "C" fn(
                    *mut wined3d_resource,
                    ::core::ffi::c_uint,
                    *mut *mut ::core::ffi::c_void,
                    *const wined3d_box,
                    uint32_t,
                ) -> HRESULT,
        ),
        resource_sub_resource_unmap: Some(
            buffer_resource_sub_resource_unmap
                as unsafe extern "C" fn(*mut wined3d_resource, ::core::ffi::c_uint) -> HRESULT,
        ),
    }
};

// buffer.c's own `struct wined3d_buffer_ops`: TU-private, so the header island carries only an
// opaque shell of it, and `wined3d_buffer::buffer_ops` points at that shell -- cast across.
#[repr(C)]
pub struct wined3d_buffer_ops {
    pub buffer_prepare_location: Option<
        unsafe extern "C" fn(*mut wined3d_buffer, *mut wined3d_context, ::core::ffi::c_uint) -> BOOL,
    >,
    pub buffer_unload_location:
        Option<unsafe extern "C" fn(*mut wined3d_buffer, *mut wined3d_context, ::core::ffi::c_uint)>,
}

static mut wined3d_buffer_no3d_ops: wined3d_buffer_ops = unsafe {
    wined3d_buffer_ops {
        buffer_prepare_location: Some(
            wined3d_buffer_no3d_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        buffer_unload_location: Some(
            wined3d_buffer_no3d_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

static mut wined3d_buffer_gl_ops: wined3d_buffer_ops = unsafe {
    wined3d_buffer_ops {
        buffer_prepare_location: Some(
            wined3d_buffer_gl_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        buffer_unload_location: Some(
            wined3d_buffer_gl_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

static mut wined3d_buffer_vk_ops: wined3d_buffer_ops = unsafe {
    wined3d_buffer_ops {
        buffer_prepare_location: Some(
            wined3d_buffer_vk_prepare_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> BOOL,
        ),
        buffer_unload_location: Some(
            wined3d_buffer_vk_unload_location
                as unsafe extern "C" fn(
                    *mut wined3d_buffer,
                    *mut wined3d_context,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
    }
};

pub const WINE_DEFAULT_DEBUG_CHANNEL_D3D: &str = "d3d"; // WINE_DEFAULT_DEBUG_CHANNEL(d3d)

pub const WINED3D_BUFFER_HASDESC: u32 = 0x01; // A vertex description has been found.

pub const WINED3D_BUFFER_USE_BO: u32 = 0x02; // Use a buffer object for this buffer.

pub const SB_MIN_SIZE: usize = 512 * 1024; // Minimum size of an allocated streaming buffer.

pub type Pwined3d_buffer_ops = *mut wined3d_buffer_ops;

// Field names of wined3d_resource_ops are assumed from the wined3d private header;
// the initializer order matches the C source.

pub const __WINE_DBCL_TRACE: i32 = 3;

unsafe extern "C" fn wined3d_buffer_evict_sysmem(mut buffer: *mut wined3d_buffer) {
    if (*buffer).resource.pin_sysmem() != 0 {
        TRACE!("Not evicting system memory for buffer %p.\n", buffer);
        return;
    }

    TRACE!("Evicting system memory for buffer %p.\n", buffer);
    wined3d_buffer_invalidate_location(buffer, WINED3D_LOCATION_SYSMEM as uint32_t);
    wined3d_resource_free_sysmem(core::ptr::addr_of_mut!((*buffer).resource));
}

unsafe extern "C" fn buffer_invalidate_bo_range(
    buffer: *mut wined3d_buffer,
    offset: ::core::ffi::c_uint,
    size: ::core::ffi::c_uint,
) {
    'invalidate_all: {
        if offset == 0 && (size == 0 || size == (*buffer).resource.size) {
            break 'invalidate_all;
        }

        if offset > (*buffer).resource.size
            || size > ((*buffer).resource.size as ::core::ffi::c_uint).wrapping_sub(offset)
        {
            WARN!("Invalid range specified, invalidating entire buffer.\n");
            break 'invalidate_all;
        }

        if wined3d_array_reserve(
            &raw mut (*buffer).dirty_ranges as *mut *mut ::core::ffi::c_void,
            &raw mut (*buffer).dirty_ranges_capacity,
            (*buffer).dirty_range_count.wrapping_add(1),
            ::core::mem::size_of::<wined3d_range>() as SIZE_T,
        ) == 0
        {
            ERR!("Failed to allocate dirty ranges array, invalidating entire buffer.\n");
            break 'invalidate_all;
        }

        (*(*buffer)
            .dirty_ranges
            .offset((*buffer).dirty_range_count as isize))
        .offset = offset;
        (*(*buffer)
            .dirty_ranges
            .offset((*buffer).dirty_range_count as isize))
        .size = size;
        (*buffer).dirty_range_count = (*buffer).dirty_range_count.wrapping_add(1);
        return;
    }

    (*buffer).dirty_range_count = 1;
    (*(*buffer)
        .dirty_ranges
        .offset(0 as isize))
    .offset = 0;
    (*(*buffer)
        .dirty_ranges
        .offset(0 as isize))
    .size = (*buffer).resource.size as ::core::ffi::c_uint;
}

unsafe extern "C" fn buffer_clear_dirty_areas(buffer: *mut wined3d_buffer) {
    (*buffer).dirty_range_count = 0;
}

unsafe extern "C" fn buffer_is_dirty(buffer: *const wined3d_buffer) -> BOOL {
    return ((*buffer).dirty_range_count != 0) as BOOL;
}

unsafe extern "C" fn buffer_is_fully_dirty(buffer: *const wined3d_buffer) -> BOOL {
    return ((*buffer).dirty_range_count == 1
        && (*(*buffer).dirty_ranges.offset(0 as isize)).offset == 0
        && (*(*buffer).dirty_ranges.offset(0 as isize)).size == (*buffer).resource.size) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_validate_location(buffer: *mut wined3d_buffer, location: uint32_t) {
    if location & WINED3D_LOCATION_BUFFER != 0 {
        buffer_clear_dirty_areas(buffer);
    }
    (*buffer).locations |= location;
}

unsafe extern "C" fn wined3d_buffer_invalidate_range(buffer: *mut wined3d_buffer, location: DWORD, offset: u32, size: u32) {
    if (location & WINED3D_LOCATION_BUFFER) != 0 && ((*buffer).flags & WINED3D_BUFFER_USE_BO) != 0 {
        buffer_invalidate_bo_range(buffer, offset, size);
    }

    (*buffer).locations &= !location;

    if (*buffer).locations == 0 {
        ERR!("Buffer %p does not have any up to date location.\n", buffer);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_invalidate_location(buffer: *mut wined3d_buffer, location: uint32_t) {
    wined3d_buffer_invalidate_range(buffer, location as DWORD, 0, 0);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_gl_binding_from_bind_flags(gl_info: *const wined3d_gl_info, bind_flags: uint32_t) -> GLenum {
    if bind_flags == 0 {
        return GL_PIXEL_UNPACK_BUFFER as GLenum;
    }

    /* We must always return GL_ELEMENT_ARRAY_BUFFER here;
     * wined3d_device_gl_create_bo() checks the GL binding to see whether we
     * can suballocate, and we cannot suballocate if this BO might be used for
     * an index buffer. */
    if bind_flags & WINED3D_BIND_INDEX_BUFFER as uint32_t != 0 {
        return GL_ELEMENT_ARRAY_BUFFER as GLenum;
    }

    if bind_flags & (WINED3D_BIND_SHADER_RESOURCE as uint32_t | WINED3D_BIND_UNORDERED_ACCESS as uint32_t) != 0
        && (*gl_info).supported[ARB_TEXTURE_BUFFER_OBJECT as usize] != 0
    {
        return GL_TEXTURE_BUFFER as GLenum;
    }

    if bind_flags & WINED3D_BIND_CONSTANT_BUFFER as uint32_t != 0 {
        return GL_UNIFORM_BUFFER as GLenum;
    }

    if bind_flags & WINED3D_BIND_STREAM_OUTPUT as uint32_t != 0 {
        return GL_TRANSFORM_FEEDBACK_BUFFER as GLenum;
    }

    if bind_flags & WINED3D_BIND_INDIRECT_BUFFER as uint32_t != 0
        && (*gl_info).supported[ARB_DRAW_INDIRECT as usize] != 0
    {
        return GL_DRAW_INDIRECT_BUFFER as GLenum;
    }

    if bind_flags & !((WINED3D_BIND_VERTEX_BUFFER as uint32_t) | (WINED3D_BIND_INDEX_BUFFER as uint32_t)) != 0 {
        FIXME!("Unhandled bind flags %#x.\n", bind_flags);
    }

    return GL_ARRAY_BUFFER as GLenum;
}

unsafe extern "C" fn wined3d_buffer_gl_destroy_buffer_object(
    mut buffer_gl: *mut wined3d_buffer_gl,
    mut context_gl: *mut wined3d_context_gl,
) {
    let mut resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*buffer_gl).b.resource);
    let mut bo_gl: *mut wined3d_bo_gl = core::ptr::null_mut();

    if (*buffer_gl).b.buffer_object.is_null() {
        return;
    }
    bo_gl = wined3d_bo_gl((*buffer_gl).b.buffer_object);

    if (((*context_gl).c.transform_feedback_active()) != 0)
        && ((*resource).bind_flags & WINED3D_BIND_STREAM_OUTPUT) != 0
        && wined3d_context_is_graphics_state_dirty(core::ptr::addr_of_mut!((*context_gl).c), (STATE_STREAM_OUTPUT as u32))
    {
        /* It's illegal to (un)bind GL_TRANSFORM_FEEDBACK_BUFFER while transform
         * feedback is active. Deleting a buffer implicitly unbinds it, so we
         * need to end transform feedback here if this buffer was bound.
         *
         * This should only be possible if STATE_STREAM_OUTPUT is dirty; if we
         * do a draw call before destroying this buffer then the draw call will
         * already rebind the GL target. */
        WARN!("Deleting buffer object for buffer %p, disabling transform feedback.\n", buffer_gl);
        wined3d_context_gl_end_transform_feedback(context_gl);
    }

    if (*buffer_gl).b.bo_user.valid {
        (*buffer_gl).b.bo_user.valid = false;
        list_remove(core::ptr::addr_of_mut!((*buffer_gl).b.bo_user.entry));
    }
    (*bo_gl).b.refcount -= 1;
    if (*bo_gl).b.refcount == 0 {
        wined3d_context_gl_destroy_bo(context_gl, bo_gl);
        free(bo_gl as *mut core::ffi::c_void);
    }
    (*buffer_gl).b.buffer_object = core::ptr::null_mut();
}

unsafe extern "C" fn wined3d_buffer_gl_create_buffer_object(
    mut buffer_gl: *mut wined3d_buffer_gl,
    mut context_gl: *mut wined3d_context_gl,
) -> BOOL {
    let mut device_gl: *mut wined3d_device_gl = wined3d_device_gl((*buffer_gl).b.resource.device);
    let mut gl_info: *const wined3d_gl_info = (*context_gl).gl_info;
    let mut usage: GLenum = GL_STATIC_DRAW;
    let mut gl_storage_flags: GLbitfield = 0;
    let mut bo: *mut wined3d_bo_gl = core::ptr::null_mut::<wined3d_bo_gl>();
    let mut coherent: bool = true;
    let mut size: GLsizeiptr = 0;
    let mut binding: GLenum = 0;

    bo = malloc(core::mem::size_of::<wined3d_bo_gl>() as size_t) as *mut wined3d_bo_gl;
    if bo.is_null() {
        return (FALSE) as BOOL;
    }

    size = (*buffer_gl).b.resource.size as GLsizeiptr;
    binding = wined3d_buffer_gl_binding_from_bind_flags(gl_info, (*buffer_gl).b.resource.bind_flags as uint32_t);
    if (*buffer_gl).b.resource.usage & WINED3DUSAGE_DYNAMIC as uint32_t != 0 {
        usage = GL_STREAM_DRAW_ARB;
        coherent = false;
    }
    gl_storage_flags = wined3d_resource_gl_storage_flags(&raw mut (*buffer_gl).b.resource);
    if !wined3d_device_gl_create_bo(
        device_gl,
        context_gl,
        size,
        binding,
        usage,
        coherent,
        gl_storage_flags,
        bo,
    ) {
        ERR!("Failed to create OpenGL buffer object.\n");
        (*buffer_gl).b.flags &= !(WINED3D_BUFFER_USE_BO) as uint32_t;
        buffer_clear_dirty_areas(&raw mut (*buffer_gl).b);
        free(bo as *mut ::core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    (*buffer_gl).b.buffer_object = &raw mut (*bo).b;
    buffer_invalidate_bo_range(
        &raw mut (*buffer_gl).b,
        0,
        0,
    );

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_incref(buffer: *mut wined3d_buffer) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(core::ptr::addr_of_mut!((*buffer).resource.ref_) as *mut i32) as u32;
    TRACE!("%p increasing refcount to %u.\n", buffer, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_prepare_location(buffer: *mut wined3d_buffer, context: *mut wined3d_context, location: ::core::ffi::c_uint) -> BOOL {
    return ((*((*buffer).buffer_ops as *const wined3d_buffer_ops)).buffer_prepare_location.unwrap_unchecked())(buffer, context, location);
}

unsafe extern "C" fn wined3d_buffer_unload_location(
    buffer: *mut wined3d_buffer,
    context: *mut wined3d_context,
    location: u32,
) {
    ((*((*buffer).buffer_ops as *const wined3d_buffer_ops)).buffer_unload_location.unwrap_unchecked())(buffer, context, location);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_load_location(buffer: *mut wined3d_buffer, context: *mut wined3d_context, location: uint32_t) -> BOOL {
    let mut src: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut dst: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };

    if (*buffer).locations as uint32_t & location != 0 {
        return (TRUE) as BOOL;
    }

    if (*buffer).locations as uint32_t == 0 {
        ERR!("Buffer %p does not have any up to date location.\n", buffer);
        wined3d_buffer_validate_location(buffer, WINED3D_LOCATION_DISCARDED);
        return wined3d_buffer_load_location(buffer, context, location);
    }

    if wined3d_buffer_prepare_location(buffer, context, location) == 0 {
        return (FALSE) as BOOL;
    }

    if (*buffer).locations as uint32_t & WINED3D_LOCATION_DISCARDED != 0 {
        wined3d_buffer_validate_location(buffer, location);
        wined3d_buffer_invalidate_location(buffer, WINED3D_LOCATION_DISCARDED);
        return (TRUE) as BOOL;
    }

    match location {
        WINED3D_LOCATION_SYSMEM => {
            if (*buffer).locations as uint32_t & WINED3D_LOCATION_CLEARED != 0 {
                memset(
                    (*buffer).resource.heap_memory,
                    0,
                    (*buffer).resource.size as size_t,
                );
            } else {
                dst.buffer_object = ::core::ptr::null_mut::<wined3d_bo>();
                dst.addr = (*buffer).resource.heap_memory as *mut BYTE;
                src.buffer_object = (*buffer).buffer_object;
                src.addr = ::core::ptr::null_mut::<BYTE>();
                range.offset = 0 as ::core::ffi::c_uint;
                range.size = (*buffer).resource.size as ::core::ffi::c_uint;
                wined3d_context_copy_bo_address(
                    context,
                    &raw mut dst,
                    &raw mut src,
                    1 as ::core::ffi::c_uint,
                    &raw mut range,
                    WINED3D_MAP_WRITE as uint32_t,
                );
            }
        }

        WINED3D_LOCATION_BUFFER => {
            let mut map_flags: uint32_t = WINED3D_MAP_WRITE as uint32_t;

            if (*buffer).locations as uint32_t & WINED3D_LOCATION_CLEARED != 0 {
                /* FIXME: Clear the buffer on the GPU if possible. */
                if wined3d_buffer_prepare_location(
                    buffer,
                    context,
                    WINED3D_LOCATION_SYSMEM as ::core::ffi::c_uint,
                ) == 0
                {
                    return (FALSE) as BOOL;
                }
                memset(
                    (*buffer).resource.heap_memory,
                    0,
                    (*buffer).resource.size as size_t,
                );
            }

            dst.buffer_object = (*buffer).buffer_object;
            dst.addr = ::core::ptr::null_mut::<BYTE>();
            src.buffer_object = ::core::ptr::null_mut::<wined3d_bo>();
            src.addr = (*buffer).resource.heap_memory as *mut BYTE;

            if buffer_is_fully_dirty(buffer) != 0 {
                map_flags |= WINED3D_MAP_DISCARD as uint32_t;
            }

            wined3d_context_copy_bo_address(
                context,
                &raw mut dst,
                &raw mut src,
                (*buffer).dirty_range_count as ::core::ffi::c_uint,
                (*buffer).dirty_ranges,
                map_flags,
            );
        }

        _ => {
            ERR!("Invalid location %s.\n", wined3d_debug_location(location));
            return (FALSE) as BOOL;
        }
    }

    wined3d_buffer_validate_location(buffer, location);
    if !(*buffer).resource.heap_memory.is_null()
        && location == WINED3D_LOCATION_BUFFER as uint32_t
        && ((*buffer).resource.usage as uint32_t & WINED3DUSAGE_DYNAMIC as uint32_t) == 0
    {
        wined3d_buffer_evict_sysmem(buffer);
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_load_sysmem(buffer: *mut wined3d_buffer, context: *mut wined3d_context) -> *mut ::core::ffi::c_void {
    if wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_SYSMEM) != 0 {
        (*buffer).resource.set_pin_sysmem((1) as _);
    }
    return (*buffer).resource.heap_memory;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_get_memory(mut buffer: *mut wined3d_buffer, mut context: *mut wined3d_context, mut data: *mut wined3d_bo_address) -> DWORD {
    let mut locations: ::core::ffi::c_uint = (*buffer).locations as ::core::ffi::c_uint;

    if locations & (WINED3D_LOCATION_DISCARDED | WINED3D_LOCATION_CLEARED) as ::core::ffi::c_uint != 0 {
        locations = (if (*buffer).flags & WINED3D_BUFFER_USE_BO as uint32_t != 0 {
            WINED3D_LOCATION_BUFFER
        } else {
            WINED3D_LOCATION_SYSMEM
        }) as ::core::ffi::c_uint;
        if wined3d_buffer_load_location(buffer, context, locations as uint32_t) == 0 {
            (*data).buffer_object = core::ptr::null_mut();
            (*data).addr = ::core::ptr::null_mut::<BYTE>();
            return 0;
        }
    }
    if locations & WINED3D_LOCATION_BUFFER as ::core::ffi::c_uint != 0 {
        (*data).buffer_object = (*buffer).buffer_object;
        (*data).addr = ::core::ptr::null_mut::<BYTE>();
        return WINED3D_LOCATION_BUFFER;
    }
    if locations & WINED3D_LOCATION_SYSMEM as ::core::ffi::c_uint != 0 {
        (*data).buffer_object = core::ptr::null_mut();
        (*data).addr = (*buffer).resource.heap_memory as *mut BYTE;
        return WINED3D_LOCATION_SYSMEM;
    }

    ERR!("Unexpected locations %s.\n", wined3d_debug_location(locations as uint32_t));
    (*data).buffer_object = core::ptr::null_mut();
    (*data).addr = ::core::ptr::null_mut::<BYTE>();
    return 0;
}

unsafe extern "C" fn buffer_resource_unload(resource: *mut wined3d_resource) {
    let buffer: *mut wined3d_buffer = buffer_from_resource(resource);

    if !(*buffer).buffer_object.is_null() {
        let context: *mut wined3d_context = context_acquire(
            (*resource).device,
            core::ptr::null_mut(),
            0 as ::core::ffi::c_uint,
        );

        wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_SYSMEM as uint32_t);
        wined3d_buffer_invalidate_location(buffer, WINED3D_LOCATION_BUFFER as uint32_t);
        wined3d_buffer_unload_location(
            buffer,
            context,
            WINED3D_LOCATION_BUFFER as ::core::ffi::c_uint,
        );
        buffer_clear_dirty_areas(buffer);

        context_release(context);

        (*buffer).flags &= !WINED3D_BUFFER_HASDESC as uint32_t;
    }

    resource_unload(resource);
}

unsafe extern "C" fn wined3d_buffer_drop_bo(buffer: *mut wined3d_buffer) {
    (*buffer).flags &= !(WINED3D_BUFFER_USE_BO as u32);
    buffer_resource_unload(core::ptr::addr_of_mut!((*buffer).resource));
}

unsafe extern "C" fn wined3d_buffer_destroy_object(mut object: *mut core::ffi::c_void) {
    let mut buffer: *mut wined3d_buffer = object as *mut wined3d_buffer;
    let mut context: *mut wined3d_context = core::ptr::null_mut();

    TRACE!("buffer %p.\n", buffer);
    if !(*buffer).buffer_object.is_null() {
        context = context_acquire(
            (*buffer).resource.device,
            core::ptr::null_mut::<wined3d_texture>(),
            0,
        );
        wined3d_buffer_unload_location(buffer, context, WINED3D_LOCATION_BUFFER);
        context_release(context);
    }
    free((*buffer).dirty_ranges as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_cleanup(buffer: *mut wined3d_buffer) {
    wined3d_cs_destroy_object(
        (*(*buffer).resource.device).cs as *mut wined3d_cs,
        Some(wined3d_buffer_destroy_object as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()),
        buffer as *mut ::core::ffi::c_void,
    );
    resource_cleanup(&raw mut (*buffer).resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_decref(mut buffer: *mut wined3d_buffer) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedDecrement(
        &raw mut (*buffer).resource.ref_ as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p decreasing refcount to %u.\n", buffer, refcount);
    if refcount == 0 {
        wined3d_mutex_lock();
        (*(*buffer).resource.parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*buffer).resource.parent);
        (*(*(*(*buffer).resource.device).adapter).adapter_ops)
            .adapter_destroy_buffer
            .unwrap_unchecked()(buffer);
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_get_parent(buffer: *const wined3d_buffer) -> *mut core::ffi::c_void {
    // The anchor's TRACE("buffer %p.\n", buffer) debug-channel check is omitted:
    // it is not present in the C source and only emits logging, not behavior.
    return (*buffer).resource.parent;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_load(buffer: *mut wined3d_buffer, context: *mut wined3d_context, state: *const wined3d_state) {
    if (*buffer).resource.map_count != 0 && !(*buffer).map_ptr.is_null() {
        return;
    } else if (*buffer).resource.map_count != 0 {
        WARN!("Loading mapped buffer.\n");
    }

    if ((*buffer).flags & WINED3D_BUFFER_USE_BO) == 0 {
        return;
    }

    if wined3d_buffer_prepare_location(buffer, context, WINED3D_LOCATION_BUFFER) == 0 {
        ERR!("Failed to prepare buffer location.\n");
        return;
    }

    /* Reading the declaration makes only sense if we have valid state information
     * (i.e., if this function is called during draws). */
    if !state.is_null() {
        (*buffer).flags |= WINED3D_BUFFER_HASDESC;
    }

    if !(((*buffer).flags & WINED3D_BUFFER_HASDESC) != 0 && buffer_is_dirty(buffer) != 0) {
        return;
    }

    if wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_BUFFER) == 0 {
        ERR!("Failed to load buffer location.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_get_resource(
    mut buffer: *mut wined3d_buffer,
) -> *mut wined3d_resource {
    WINE_TRACE!("buffer %p.\n", buffer);
    return core::ptr::addr_of_mut!((*buffer).resource);
}

unsafe extern "C" fn buffer_resource_sub_resource_get_desc(
    resource: *mut wined3d_resource,
    sub_resource_idx: u32,
    desc: *mut wined3d_sub_resource_desc,
) -> HRESULT {
    if sub_resource_idx != 0 {
        WARN!("Invalid sub_resource_idx %u.\n", sub_resource_idx);
        return E_INVALIDARG;
    }

    (*desc).format = WINED3DFMT_R8_UNORM;
    (*desc).multisample_type = WINED3D_MULTISAMPLE_NONE;
    (*desc).multisample_quality = 0;
    (*desc).usage = (*resource).usage as u32;
    (*desc).bind_flags = (*resource).bind_flags;
    (*desc).access = (*resource).access;
    (*desc).width = (*resource).size as u32;
    (*desc).height = 1;
    (*desc).depth = 1;
    (*desc).size = (*resource).size as u32;
    return S_OK;
}

unsafe extern "C" fn buffer_resource_sub_resource_get_map_pitch(
    mut resource: *mut wined3d_resource,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut row_pitch: *mut ::core::ffi::c_uint,
    mut slice_pitch: *mut ::core::ffi::c_uint,
) {
    *slice_pitch = (*resource).size as ::core::ffi::c_uint;
    *row_pitch = *slice_pitch;
}

unsafe extern "C" fn buffer_resource_sub_resource_map(
    mut resource: *mut wined3d_resource,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut map_ptr: *mut *mut ::core::ffi::c_void,
    mut box_0: *const wined3d_box,
    mut flags: uint32_t,
) -> HRESULT {
    let mut buffer: *mut wined3d_buffer = buffer_from_resource(resource);
    let mut offset: ::core::ffi::c_uint = 0;
    let mut size: ::core::ffi::c_uint = 0;
    let mut dirty_offset: ::core::ffi::c_uint = 0;
    let mut dirty_size: ::core::ffi::c_uint = 0;
    let mut device: *mut wined3d_device = (*resource).device;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut addr: wined3d_bo_address = core::mem::zeroed();
    let mut base: *mut uint8_t = core::ptr::null_mut::<uint8_t>();
    let mut count: LONG = 0;

    offset = (*box_0).left as ::core::ffi::c_uint;
    dirty_offset = offset;
    size = (*box_0).right.wrapping_sub((*box_0).left) as ::core::ffi::c_uint;
    dirty_size = size;

    (*resource).map_count += 1;
    count = (*resource).map_count;

    /* DISCARD invalidates the entire buffer, regardless of the specified
     * offset and size. Some applications also depend on the entire buffer
     * being uploaded in that case. Two such applications are Port Royale
     * and Darkstar One. */
    if flags & WINED3D_MAP_DISCARD != 0 {
        dirty_offset = 0;
        dirty_size = 0;
    }

    if (((flags & WINED3D_MAP_WRITE) != 0 && (flags & (WINED3D_MAP_NOOVERWRITE | WINED3D_MAP_DISCARD)) == 0)
        || ((flags & WINED3D_MAP_WRITE) == 0 && ((*buffer).locations & WINED3D_LOCATION_SYSMEM) != 0)
        || (*buffer).resource.pin_sysmem() != 0
        || ((*buffer).flags & WINED3D_BUFFER_USE_BO) == 0)
    {
        if ((*buffer).locations & WINED3D_LOCATION_SYSMEM) == 0 {
            context = context_acquire(device, core::ptr::null_mut::<wined3d_texture>(), 0);
            wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_SYSMEM);
            context_release(context);
        }

        if flags & WINED3D_MAP_WRITE != 0 {
            wined3d_buffer_invalidate_range(buffer, !WINED3D_LOCATION_SYSMEM, dirty_offset, dirty_size);
        }
    } else {
        context = context_acquire(device, core::ptr::null_mut::<wined3d_texture>(), 0);

        if flags & WINED3D_MAP_DISCARD != 0 {
            if wined3d_buffer_prepare_location(buffer, context, WINED3D_LOCATION_BUFFER) == 0 {
                context_release(context);
                return E_OUTOFMEMORY;
            }
            wined3d_buffer_validate_location(buffer, WINED3D_LOCATION_BUFFER);
        } else {
            wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_BUFFER);
        }

        if flags & WINED3D_MAP_WRITE != 0 {
            wined3d_buffer_acquire_bo_for_write(buffer, context);
            wined3d_buffer_invalidate_location(buffer, !WINED3D_LOCATION_BUFFER);
            buffer_invalidate_bo_range(buffer, dirty_offset, dirty_size);
        }

        if (flags & WINED3D_MAP_DISCARD) != 0 && !(*resource).heap_memory.is_null() {
            wined3d_buffer_evict_sysmem(buffer);
        }

        if count == 1 {
            addr.buffer_object = (*buffer).buffer_object;
            addr.addr = 0 as *mut BYTE;
            (*buffer).map_ptr = wined3d_context_map_bo_address(
                context,
                core::ptr::addr_of_mut!(addr),
                (*resource).size as usize,
                flags,
            );

            /* We are accessing buffer->resource.client from the CS thread,
             * but it's safe because the client thread will wait for the
             * map to return, thus completely serializing this call with
             * other client code. */
            if (*(*context).d3d_info).persistent_map() != 0 {
                (*buffer).resource.client.addr = addr;
            }

            if ((((*buffer).map_ptr as DWORD_PTR)
                & ((RESOURCE_ALIGNMENT - 1) as DWORD_PTR))
                != 0)
            {
                WARN!("Pointer %p is not %u byte aligned.\n", (*buffer).map_ptr, RESOURCE_ALIGNMENT);

                wined3d_context_unmap_bo_address(
                    context,
                    core::ptr::addr_of_mut!(addr),
                    0,
                    core::ptr::null(),
                );
                (*buffer).map_ptr = core::ptr::null_mut();

                if (*resource).usage & WINED3DUSAGE_DYNAMIC != 0 {
                    /* The extra copy is more expensive than not using VBOs
                     * at all on the NVIDIA Linux driver, which is the
                     * only driver that returns unaligned pointers. */
                    wined3d_buffer_drop_bo(buffer);
                } else {
                    /* Fall back to double-buffered operation through sysmem. */
                    wined3d_buffer_load_location(buffer, context, WINED3D_LOCATION_SYSMEM);
                    (*buffer).resource.set_pin_sysmem((1) as _);
                }
            }
        }

        context_release(context);
    }

    base = (if !(*buffer).map_ptr.is_null() {
        (*buffer).map_ptr
    } else {
        (*resource).heap_memory
    }) as *mut uint8_t;
    *map_ptr = base.offset(offset as isize) as *mut ::core::ffi::c_void;

    return WINED3D_OK;
}

unsafe extern "C" fn buffer_resource_sub_resource_unmap(resource: *mut wined3d_resource, sub_resource_idx: ::core::ffi::c_uint) -> HRESULT {
    let mut buffer: *mut wined3d_buffer = buffer_from_resource(resource);
    let mut device: *mut wined3d_device = (*resource).device;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut addr: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut::<wined3d_bo>(),
        addr: core::ptr::null_mut::<BYTE>(),
    };

    if sub_resource_idx != 0 {
        WARN!("Invalid sub_resource_idx %u.\n", sub_resource_idx);
        return E_INVALIDARG;
    }

    if (*resource).map_count == 0 {
        WARN!("Unmap called without a previous map call.\n");
        return WINED3D_OK;
    }

    (*resource).map_count -= 1;
    if (*resource).map_count != 0 {
        /* Delay loading the buffer until everything is unmapped. */
        return WINED3D_OK;
    }

    if (*buffer).map_ptr.is_null() {
        return WINED3D_OK;
    }

    context = context_acquire(device, core::ptr::null_mut::<wined3d_texture>(), 0);

    addr.buffer_object = (*buffer).buffer_object;
    addr.addr = core::ptr::null_mut();
    wined3d_context_unmap_bo_address(context, core::ptr::addr_of_mut!(addr), (*buffer).dirty_range_count as ::core::ffi::c_uint, (*buffer).dirty_ranges);

    context_release(context);

    buffer_clear_dirty_areas(buffer);
    (*buffer).map_ptr = core::ptr::null_mut();

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_buffer_set_bo(
    buffer: *mut wined3d_buffer,
    context: *mut wined3d_context,
    bo: *mut wined3d_bo,
) {
    let mut prev_bo: *mut wined3d_bo = (*buffer).buffer_object;

    if !prev_bo.is_null() {
        let mut bo_user: *mut wined3d_bo_user = core::ptr::null_mut::<wined3d_bo_user>();

        /* The previous BO might have users in other buffers which were valid,
         * and should in theory remain valid. The problem is that it's not easy
         * to tell which users belong to this buffer and which don't. We could
         * add a field, but for now it's easier and probably fine to just
         * invalidate every user. */
        bo_user = ((*prev_bo).users.next as *mut u8)
            .sub(core::mem::offset_of!(wined3d_bo_user, entry))
            as *mut wined3d_bo_user;
        while core::ptr::addr_of_mut!((*bo_user).entry)
            != core::ptr::addr_of_mut!((*prev_bo).users)
        {
            (*bo_user).valid = false;
            bo_user = ((*bo_user).entry.next as *mut u8)
                .sub(core::mem::offset_of!(wined3d_bo_user, entry))
                as *mut wined3d_bo_user;
        }
        list_init(core::ptr::addr_of_mut!((*prev_bo).users));

        (*prev_bo).refcount = (*prev_bo).refcount.wrapping_sub(1);
        if (*prev_bo).refcount == 0 {
            wined3d_context_destroy_bo(context, prev_bo);
            free(prev_bo as *mut core::ffi::c_void);
        }
    }

    (*buffer).buffer_object = bo;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_acquire_bo_for_write(buffer: *mut wined3d_buffer, context: *mut wined3d_context) {
    let range: wined3d_range = wined3d_range {
        offset: 0,
        size: (*buffer).resource.size as ::core::ffi::c_uint,
    };
    let mut dst: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut src: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut bo: *mut wined3d_bo = ::core::ptr::null_mut::<wined3d_bo>();

    bo = (*buffer).buffer_object;
    if bo.is_null() {
        return;
    }

    /* If we are the only owner of this BO, there is nothing to do. */
    if (*bo).refcount == 1 {
        return;
    }

    TRACE!("Performing copy-on-write for BO %p.\n", bo);

    /* Grab a reference to the current BO. It's okay if this overflows, because
     * the following unload will release it. */
    (*bo).refcount = (*bo).refcount.wrapping_add(1);

    /* Unload and re-prepare to get a new buffer. */
    wined3d_buffer_unload_location(buffer, context, WINED3D_LOCATION_BUFFER);
    wined3d_buffer_prepare_location(buffer, context, WINED3D_LOCATION_BUFFER);

    /* And finally, perform the actual copy. */
    assert!((*buffer).buffer_object != bo);
    dst.buffer_object = (*buffer).buffer_object;
    dst.addr = ::core::ptr::null_mut::<BYTE>();
    src.buffer_object = bo;
    src.addr = ::core::ptr::null_mut::<BYTE>();
    wined3d_context_copy_bo_address(
        context,
        core::ptr::addr_of_mut!(dst),
        core::ptr::addr_of_mut!(src),
        1,
        core::ptr::addr_of!(range),
        WINED3D_MAP_WRITE | WINED3D_MAP_DISCARD,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_copy_bo_address(mut dst_buffer: *mut wined3d_buffer, mut context: *mut wined3d_context, mut dst_offset: ::core::ffi::c_uint, mut src_addr: *const wined3d_const_bo_address, mut size: ::core::ffi::c_uint) {
    let mut map_flags: uint32_t = (WINED3D_MAP_WRITE) as uint32_t;
    let mut dst_addr: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut dst_location: DWORD = 0;
    if dst_offset == 0 && size == (*dst_buffer).resource.size {
        map_flags |= (WINED3D_MAP_DISCARD) as uint32_t;
    }
    if (map_flags & ((WINED3D_MAP_DISCARD) as uint32_t)) != 0 {
        wined3d_buffer_acquire_bo_for_write(dst_buffer, context);
    }
    dst_location = wined3d_buffer_get_memory(dst_buffer, context, &raw mut dst_addr);
    dst_addr.addr = dst_addr.addr.offset(dst_offset as isize);
    range.offset = (0) as ::core::ffi::c_uint;
    range.size = size;
    wined3d_context_copy_bo_address(
        context,
        &raw mut dst_addr,
        src_addr as *const wined3d_bo_address,
        (1) as ::core::ffi::c_uint,
        &raw mut range,
        map_flags,
    );
    wined3d_buffer_invalidate_range(dst_buffer, !dst_location, dst_offset, size);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_copy(mut dst_buffer: *mut wined3d_buffer, mut dst_offset: ::core::ffi::c_uint, mut src_buffer: *mut wined3d_buffer, mut src_offset: ::core::ffi::c_uint, mut size: ::core::ffi::c_uint) {
    let mut context: *mut wined3d_context = ::core::ptr::null_mut::<wined3d_context>();
    let mut src: wined3d_bo_address = wined3d_bo_address {
        buffer_object: ::core::ptr::null_mut::<wined3d_bo>(),
        addr: ::core::ptr::null_mut::<BYTE>(),
    };
    WINE_TRACE!("dst_buffer %p, dst_offset %u, src_buffer %p, src_offset %u, size %u.\n", dst_buffer, dst_offset, src_buffer, src_offset, size);

    context = context_acquire(
        (*dst_buffer).resource.device,
        core::ptr::null_mut(),
        0,
    );

    wined3d_buffer_get_memory(src_buffer, context, &raw mut src);
    src.addr = src.addr.offset(src_offset as isize);

    wined3d_buffer_copy_bo_address(
        dst_buffer,
        context,
        dst_offset,
        wined3d_const_bo_address(&raw mut src),
        size,
    );

    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_update_sub_resource(buffer: *mut wined3d_buffer, context: *mut wined3d_context, upload_bo: *const upload_bo, offset: ::core::ffi::c_uint, size: ::core::ffi::c_uint) {
    let mut bo: *mut wined3d_bo = (*upload_bo).addr.buffer_object;
    let mut flags: uint32_t = (*upload_bo).flags;

    /* Try to take this buffer for COW. Don't take it if we've saturated the
     * refcount. */
    if offset == 0
        && size == (*buffer).resource.size
        && !bo.is_null()
        && (((*bo).refcount as ::core::ffi::c_int) < (UINT8_MAX as i32))
        && ((*upload_bo).flags & UPLOAD_BO_RENAME_ON_UNMAP == 0)
    {
        flags |= UPLOAD_BO_RENAME_ON_UNMAP;
        (*bo).refcount = (*bo).refcount.wrapping_add(1);
    }

    if flags & UPLOAD_BO_RENAME_ON_UNMAP != 0 {
        /* Don't increment the refcount. UPLOAD_BO_RENAME_ON_UNMAP transfers an
         * existing reference.
         *
         * FIXME: We could degenerate RENAME to a copy + free and rely on the
         * COW logic to detect this case.
         */
        wined3d_buffer_set_bo(buffer, context, (*upload_bo).addr.buffer_object);
        wined3d_buffer_validate_location(buffer, WINED3D_LOCATION_BUFFER);
        wined3d_buffer_invalidate_location(buffer, !WINED3D_LOCATION_BUFFER);
    }

    if !(*upload_bo).addr.buffer_object.is_null()
        && (*upload_bo).addr.buffer_object == (*buffer).buffer_object
    {
        let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };

        /* We need to flush changes, which is implicitly done by
         * wined3d_context_unmap_bo_address() even if we aren't actually going
         * to unmap.
         *
         * We would also like to free up virtual address space used by this BO
         * if it's at a premium—note that this BO was allocated for an
         * accelerated map. Hence we unmap the BO instead of merely flushing it;
         * if we don't care about unmapping BOs then
         * wined3d_context_unmap_bo_address() will flush and return.
         */
        range.offset = offset;
        range.size = size;
        if !(*(*upload_bo).addr.buffer_object).map_ptr.is_null() {
            wined3d_context_unmap_bo_address(
                context,
                core::ptr::addr_of!((*upload_bo).addr) as *const wined3d_bo_address,
                1,
                core::ptr::addr_of_mut!(range),
            );
        }
    } else {
        wined3d_buffer_copy_bo_address(buffer, context, offset, core::ptr::addr_of!((*upload_bo).addr), size);
    }
}

unsafe extern "C" fn wined3d_buffer_init_data(
    mut buffer: *mut wined3d_buffer,
    mut device: *mut wined3d_device,
    mut data: *const wined3d_sub_resource_data,
) {
    let mut resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*buffer).resource);
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    if (*buffer).flags & (WINED3D_BUFFER_USE_BO as uint32_t) != 0 {
        wined3d_box_set(
            core::ptr::addr_of_mut!(box_0),
            0 as ::core::ffi::c_uint,
            0 as ::core::ffi::c_uint,
            (*resource).size as ::core::ffi::c_uint,
            1 as ::core::ffi::c_uint,
            0 as ::core::ffi::c_uint,
            1 as ::core::ffi::c_uint,
        );
        wined3d_device_context_emit_update_sub_resource(
            core::ptr::addr_of_mut!((*(*device).cs).c),
            resource,
            0 as ::core::ffi::c_uint,
            core::ptr::addr_of_mut!(box_0),
            (*data).data,
            (*data).row_pitch,
            (*data).slice_pitch,
        );
    } else {
        memcpy(
            (*buffer).resource.heap_memory,
            (*data).data,
            (*resource).size as size_t,
        );
        wined3d_buffer_validate_location(buffer, WINED3D_LOCATION_SYSMEM as uint32_t);
        wined3d_buffer_invalidate_location(buffer, !(WINED3D_LOCATION_SYSMEM as uint32_t));
    }
}

unsafe extern "C" fn buffer_resource_incref(resource: *mut wined3d_resource) -> ULONG {
    return wined3d_buffer_incref(buffer_from_resource(resource));
}

unsafe extern "C" fn buffer_resource_decref(resource: *mut wined3d_resource) -> ULONG {
    return wined3d_buffer_decref(buffer_from_resource(resource));
}

unsafe extern "C" fn buffer_resource_preload(resource: *mut wined3d_resource) {
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    context = context_acquire(
        (*resource).device,
        core::ptr::null_mut::<wined3d_texture>(),
        0 as core::ffi::c_uint,
    );
    wined3d_buffer_load(
        buffer_from_resource(resource),
        context,
        core::ptr::null::<wined3d_state>(),
    );
    context_release(context);
}

unsafe extern "C" fn wined3d_buffer_init(
    buffer: *mut wined3d_buffer,
    device: *mut wined3d_device,
    desc: *const wined3d_buffer_desc,
    data: *const wined3d_sub_resource_data,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    buffer_ops: *const wined3d_buffer_ops,
) -> HRESULT {
    let format: *const wined3d_format = wined3d_get_format((*device).adapter, WINED3DFMT_R8_UNORM, (*desc).bind_flags);
    let resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*buffer).resource);
    let mut access: u32;
    let mut hr: HRESULT;

    if (*desc).byte_width == 0 {
        WARN!("Size 0 requested, returning E_INVALIDARG.\n");
        return E_INVALIDARG;
    }

    if ((*desc).bind_flags & WINED3D_BIND_CONSTANT_BUFFER) != 0
        && ((*desc).byte_width & (WINED3D_CONSTANT_BUFFER_ALIGNMENT - 1)) != 0
    {
        WARN!("Size %#x is not suitably aligned for constant buffers.\n", (*desc).byte_width);
        return E_INVALIDARG;
    }

    if !data.is_null() && (*data).data.is_null() {
        WARN!("Invalid sub-resource data specified.\n");
        return E_INVALIDARG;
    }

    access = (*desc).access;
    if ((*desc).bind_flags & WINED3D_BIND_CONSTANT_BUFFER) != 0 && wined3d_settings.cb_access_map_w != false {
        access |= WINED3D_RESOURCE_ACCESS_MAP_W;
    }

    hr = resource_init(
        resource,
        device,
        WINED3D_RTYPE_BUFFER,
        format,
        WINED3D_MULTISAMPLE_NONE,
        0,
        (*desc).usage,
        (*desc).bind_flags,
        access,
        (*desc).byte_width,
        1,
        1,
        (*desc).byte_width,
        parent,
        parent_ops,
        core::ptr::addr_of!(buffer_resource_ops),
    );
    if hr < 0 {
        WARN!("Failed to initialize resource, hr %#lx.\n", hr);
        return hr;
    }
    (*buffer).buffer_ops = buffer_ops as *const _ as _;
    (*buffer).structure_byte_stride = (*desc).structure_byte_stride;
    (*buffer).locations = WINED3D_LOCATION_CLEARED;

    if ((*device).create_parms.flags & WINED3DCREATE_SOFTWARE_VERTEXPROCESSING) != 0
        || ((*desc).usage & WINED3DUSAGE_MANAGED) != 0
    {
        /* SWvp and managed buffers always return the same pointer in buffer
         * maps and retain data in DISCARD maps. Keep a system memory copy of
         * the buffer to provide the same behavior to the application. */
        (*buffer).resource.set_pin_sysmem((1) as _);
        (*buffer).locations = WINED3D_LOCATION_SYSMEM;
    }

    if ((*buffer).locations & WINED3D_LOCATION_SYSMEM) != 0 || ((*buffer).flags & WINED3D_BUFFER_USE_BO) == 0 {
        if wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*buffer).resource)) == 0 {
            return E_OUTOFMEMORY;
        }
    }

    if ((*buffer).flags & WINED3D_BUFFER_USE_BO) != 0
        && wined3d_array_reserve(
            core::ptr::addr_of_mut!((*buffer).dirty_ranges) as *mut *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*buffer).dirty_ranges_capacity),
            1,
            (core::mem::size_of::<wined3d_range>()) as SIZE_T,
        ) == 0
    {
        ERR!("Out of memory.\n");
        buffer_resource_unload(resource);
        resource_cleanup(resource);
        wined3d_resource_wait_idle(resource);
        return E_OUTOFMEMORY;
    }

    if ((*buffer).locations & WINED3D_LOCATION_DISCARDED) != 0 {
        (*buffer).resource.client.addr.buffer_object = CLIENT_BO_DISCARDED;
    }

    if !data.is_null() {
        wined3d_buffer_init_data(buffer, device, data);
    }

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_buffer_no3d_prepare_location(buffer: *mut wined3d_buffer, context: *mut wined3d_context, location: ::core::ffi::c_uint) -> BOOL {
    if location == WINED3D_LOCATION_SYSMEM {
        return wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*buffer).resource));
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn wined3d_buffer_no3d_unload_location(
    buffer: *mut wined3d_buffer,
    context: *mut wined3d_context,
    location: ::core::ffi::c_uint,
) {
    WINE_TRACE!("buffer %p, context %p, location %s.\n", buffer, context, wined3d_debug_location(location as uint32_t));
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_no3d_init(buffer_no3d: *mut wined3d_buffer, device: *mut wined3d_device, desc: *const wined3d_buffer_desc, data: *const wined3d_sub_resource_data, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    if ((*__wine_dbch___default).flags as i32) & ((1 as i32) << __WINE_DBCL_TRACE as i32) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE as __wine_debug_class;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/directx/wine/wined3d/buffer.c\0"
                .as_ptr() as *const i8,
            b"wined3d_buffer_no3d_init\0".as_ptr() as *const i8,
            1045,
            b"buffer_no3d %p, device %p, desc %p, data %p, parent %p, parent_ops %p.\n\0"
                .as_ptr() as *const i8,
            buffer_no3d,
            device,
            desc,
            data,
            parent,
            parent_ops,
        );
    }
    return wined3d_buffer_init(
        buffer_no3d,
        device,
        desc,
        data,
        parent,
        parent_ops,
        core::ptr::addr_of!(wined3d_buffer_no3d_ops),
    );
}

unsafe extern "C" fn wined3d_buffer_gl_prepare_location(
    mut buffer: *mut wined3d_buffer,
    mut context: *mut wined3d_context,
    mut location: ::core::ffi::c_uint,
) -> BOOL {
    let mut context_gl: *mut wined3d_context_gl = wined3d_context_gl(context);
    let mut buffer_gl: *mut wined3d_buffer_gl = wined3d_buffer_gl(buffer);
    match location {
        WINED3D_LOCATION_SYSMEM => return wined3d_resource_prepare_sysmem(&raw mut (*buffer).resource),
        WINED3D_LOCATION_BUFFER => {
            if !(*buffer).buffer_object.is_null() {
                return (TRUE) as BOOL;
            }
            if ((*buffer).flags & WINED3D_BUFFER_USE_BO as uint32_t) == 0 {
                WARN!(
                    "Trying to create BO for buffer %p with no WINED3D_BUFFER_USE_BO.\n",
                    buffer
                );
                return (FALSE) as BOOL;
            }
            return wined3d_buffer_gl_create_buffer_object(buffer_gl, context_gl);
        }
        _ => {
            ERR!(
                "Invalid location %s.\n",
                wined3d_debug_location(location as uint32_t)
            );
            return (FALSE) as BOOL;
        }
    };
}

unsafe extern "C" fn wined3d_buffer_gl_unload_location(
    buffer: *mut wined3d_buffer,
    context: *mut wined3d_context,
    location: ::core::ffi::c_uint,
) {
    match location {
        WINED3D_LOCATION_BUFFER => {
            wined3d_buffer_gl_destroy_buffer_object(wined3d_buffer_gl(buffer), wined3d_context_gl(context));
        }
        _ => {
            ERR!("Unhandled location %s.\n", wined3d_debug_location(location));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_gl_init(buffer_gl: *mut wined3d_buffer_gl, device: *mut wined3d_device, desc: *const wined3d_buffer_desc, data: *const wined3d_sub_resource_data, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let gl_info: *const wined3d_gl_info = &raw const (*wined3d_adapter_gl((*device).adapter)).gl_info;

    WINE_TRACE!("buffer_gl %p, device %p, desc %p, data %p, parent %p, parent_ops %p.\n", buffer_gl, device, desc, data, parent, parent_ops);

    if ((*desc).access & WINED3D_RESOURCE_ACCESS_GPU) == 0 {
        WINE_TRACE!("Not creating a BO because the buffer is not GPU accessible.\n");
    } else if (*gl_info).supported[ARB_VERTEX_BUFFER_OBJECT as usize] == 0 {
        WINE_TRACE!("Not creating a BO because GL_ARB_vertex_buffer is not supported.\n");
    } else if !(((*gl_info).supported[APPLE_FLUSH_BUFFER_RANGE as usize] != 0)
        || ((*gl_info).supported[ARB_MAP_BUFFER_RANGE as usize] != 0))
        && ((*desc).usage & WINED3DUSAGE_DYNAMIC) != 0
    {
        WINE_TRACE!("Not creating a BO because the buffer has dynamic usage and no GL support.\n");
    } else {
        (*buffer_gl).b.flags |= WINED3D_BUFFER_USE_BO as u32;
    }

    return wined3d_buffer_init(
        &raw mut (*buffer_gl).b,
        device,
        desc,
        data,
        parent,
        parent_ops,
        &raw const wined3d_buffer_gl_ops,
    );
}
#[no_mangle]
pub unsafe extern "C" fn vk_buffer_usage_from_bind_flags(mut bind_flags: uint32_t) -> VkBufferUsageFlags {
    let mut usage: VkBufferUsageFlags;
    usage = (VK_BUFFER_USAGE_TRANSFER_SRC_BIT as ::core::ffi::c_int | VK_BUFFER_USAGE_TRANSFER_DST_BIT as ::core::ffi::c_int) as VkBufferUsageFlags;
    if bind_flags & WINED3D_BIND_VERTEX_BUFFER as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_VERTEX_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_INDEX_BUFFER as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_INDEX_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_CONSTANT_BUFFER as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_SHADER_RESOURCE as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_UNIFORM_TEXEL_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_STREAM_OUTPUT as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_TRANSFORM_FEEDBACK_BUFFER_BIT_EXT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_UNORDERED_ACCESS as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_STORAGE_TEXEL_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & WINED3D_BIND_INDIRECT_BUFFER as uint32_t != 0 {
        usage |= VK_BUFFER_USAGE_INDIRECT_BUFFER_BIT as ::core::ffi::c_int as VkBufferUsageFlags;
    }
    if bind_flags & (WINED3D_BIND_RENDER_TARGET | WINED3D_BIND_DEPTH_STENCIL) as uint32_t != 0 {
        FIXME!("Ignoring some bind flags %#x.\n", bind_flags);
    }
    return usage;
}
#[no_mangle]
pub unsafe extern "C" fn vk_memory_type_from_access_flags(mut access: uint32_t, mut usage: uint32_t) -> VkMemoryPropertyFlags {
    let mut memory_type: VkMemoryPropertyFlags = 0 as VkMemoryPropertyFlags;

    if (access & (WINED3D_RESOURCE_ACCESS_MAP_R as uint32_t)) != 0 {
        memory_type |= (VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags)
            | (VK_MEMORY_PROPERTY_HOST_CACHED_BIT as VkMemoryPropertyFlags);
    } else if (access & (WINED3D_RESOURCE_ACCESS_MAP_W as uint32_t)) != 0 {
        memory_type |= VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags;
    } else if (usage & (WINED3DUSAGE_DYNAMIC as uint32_t)) == 0 {
        memory_type |= VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as VkMemoryPropertyFlags;
    }
    return memory_type;
}

unsafe extern "C" fn wined3d_buffer_vk_create_buffer_object(
    buffer_vk: *mut wined3d_buffer_vk,
    context_vk: *mut wined3d_context_vk,
) -> BOOL {
    let resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*buffer_vk).b.resource);
    let mut bo_vk: *mut wined3d_bo_vk;

    bo_vk = malloc(core::mem::size_of::<wined3d_bo_vk>() as size_t) as *mut wined3d_bo_vk;
    if bo_vk.is_null() {
        return (FALSE) as BOOL;
    }

    if wined3d_context_vk_create_bo(
        context_vk,
        (*resource).size as VkDeviceSize,
        vk_buffer_usage_from_bind_flags((*resource).bind_flags as uint32_t),
        vk_memory_type_from_access_flags((*resource).access as uint32_t, (*resource).usage),
        bo_vk,
    ) == 0
    {
        WARN!("Failed to create Vulkan buffer.\n");
        free(bo_vk as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }

    (*buffer_vk).b.buffer_object = core::ptr::addr_of_mut!((*bo_vk).b);
    buffer_invalidate_bo_range(
        core::ptr::addr_of_mut!((*buffer_vk).b),
        0,
        0,
    );

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_vk_get_buffer_info(mut buffer_vk: *mut wined3d_buffer_vk) -> *const VkDescriptorBufferInfo {
    let mut bo: *mut wined3d_bo_vk = wined3d_bo_vk((*buffer_vk).b.buffer_object);

    if (*buffer_vk).b.bo_user.valid != false {
        return core::ptr::addr_of_mut!((*buffer_vk).buffer_info);
    }

    (*buffer_vk).buffer_info.buffer = (*bo).vk_buffer;
    (*buffer_vk).buffer_info.offset = (*bo).b.buffer_offset as VkDeviceSize;
    (*buffer_vk).buffer_info.range = (*buffer_vk).b.resource.size as VkDeviceSize;
    wined3d_buffer_validate_user(core::ptr::addr_of_mut!((*buffer_vk).b));

    return core::ptr::addr_of_mut!((*buffer_vk).buffer_info);
}

unsafe extern "C" fn wined3d_buffer_vk_prepare_location(
    buffer: *mut wined3d_buffer,
    context: *mut wined3d_context,
    location: ::core::ffi::c_uint,
) -> BOOL {
    match location {
        WINED3D_LOCATION_SYSMEM => {
            return wined3d_resource_prepare_sysmem(core::ptr::addr_of_mut!((*buffer).resource));
        }

        WINED3D_LOCATION_BUFFER => {
            if !(*buffer).buffer_object.is_null() {
                return (TRUE) as BOOL;
            }

            return wined3d_buffer_vk_create_buffer_object(
                wined3d_buffer_vk(buffer),
                wined3d_context_vk(context),
            );
        }

        _ => {
            FIXME!(
                "Unhandled location %s.\n",
                wined3d_debug_location(location)
            );
            return (FALSE) as BOOL;
        }
    }
}

unsafe extern "C" fn wined3d_buffer_vk_unload_location(mut buffer: *mut wined3d_buffer, mut context: *mut wined3d_context, mut location: u32) {
    let mut context_vk: *mut wined3d_context_vk = wined3d_context_vk(context);
    let mut bo_vk: *mut wined3d_bo_vk = wined3d_bo_vk((*buffer).buffer_object);

    match location {
        WINED3D_LOCATION_BUFFER => {
            if (*buffer).bo_user.valid {
                (*buffer).bo_user.valid = false;
                list_remove(&raw mut (*buffer).bo_user.entry);
            }
            (*bo_vk).b.refcount = (*bo_vk).b.refcount.wrapping_sub(1);
            if (*bo_vk).b.refcount == 0 {
                wined3d_context_vk_destroy_bo(context_vk, bo_vk);
                free(bo_vk as *mut ::core::ffi::c_void);
            }
            (*buffer).buffer_object = core::ptr::null_mut();
        }
        _ => {
            ERR!("Unhandled location %s.\n", wined3d_debug_location(location));
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_vk_init(buffer_vk: *mut wined3d_buffer_vk, device: *mut wined3d_device, desc: *const wined3d_buffer_desc, data: *const wined3d_sub_resource_data, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let vk_info: *const wined3d_vk_info =
        core::ptr::addr_of!((*wined3d_adapter_vk((*device).adapter)).vk_info);

    TRACE!(
        "buffer_vk %p, device %p, desc %p, data %p, parent %p, parent_ops %p.\n",
        buffer_vk,
        device,
        desc,
        data,
        parent,
        parent_ops
    );

    if ((*desc).bind_flags & WINED3D_BIND_STREAM_OUTPUT) != 0
        && (*vk_info).supported[WINED3D_VK_EXT_TRANSFORM_FEEDBACK as usize] == 0
    {
        WARN!("The Vulkan implementation does not support transform feedback.\n");
        return WINED3DERR_INVALIDCALL;
    }

    if ((*desc).access & WINED3D_RESOURCE_ACCESS_GPU) != 0 {
        (*buffer_vk).b.flags |= WINED3D_BUFFER_USE_BO as u32;
    }

    return wined3d_buffer_init(
        core::ptr::addr_of_mut!((*buffer_vk).b),
        device,
        desc,
        data,
        parent,
        parent_ops,
        core::ptr::addr_of!(wined3d_buffer_vk_ops),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_vk_barrier(buffer_vk: *mut wined3d_buffer_vk, context_vk: *mut wined3d_context_vk, bind_mask: uint32_t) {
    let mut src_bind_mask: uint32_t = 0;

    if (bind_mask & !WINED3D_READ_ONLY_BIND_MASK) != 0 {
        src_bind_mask = (*buffer_vk).bind_mask & WINED3D_READ_ONLY_BIND_MASK;
        if src_bind_mask == 0 {
            src_bind_mask = (*buffer_vk).bind_mask;
        }

        (*buffer_vk).bind_mask = bind_mask;
    } else if ((*buffer_vk).bind_mask & bind_mask) != bind_mask {
        src_bind_mask = (*buffer_vk).bind_mask & !WINED3D_READ_ONLY_BIND_MASK;
        (*buffer_vk).bind_mask |= bind_mask;
    }

    if src_bind_mask != 0 {
        let bo: *const wined3d_bo_vk = wined3d_bo_vk((*buffer_vk).b.buffer_object);
        let vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
        let mut vk_barrier: VkBufferMemoryBarrier = core::mem::zeroed();

        wined3d_context_vk_end_current_render_pass(context_vk);

        vk_barrier.sType = VK_STRUCTURE_TYPE_BUFFER_MEMORY_BARRIER;
        vk_barrier.pNext = core::ptr::null();
        vk_barrier.srcAccessMask = vk_access_mask_from_bind_flags(src_bind_mask);
        vk_barrier.dstAccessMask = vk_access_mask_from_bind_flags(bind_mask);
        vk_barrier.srcQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier.dstQueueFamilyIndex = (VK_QUEUE_FAMILY_IGNORED) as u32;
        vk_barrier.buffer = (*bo).vk_buffer;
        vk_barrier.offset = (*bo).b.buffer_offset as VkDeviceSize;
        vk_barrier.size = (*buffer_vk).b.resource.size as VkDeviceSize;
        ((*vk_info).vk_ops.vkCmdPipelineBarrier.unwrap_unchecked())(
            wined3d_context_vk_get_command_buffer(context_vk),
            vk_pipeline_stage_mask_from_bind_flags(src_bind_mask),
            vk_pipeline_stage_mask_from_bind_flags(bind_mask),
            0,
            0,
            core::ptr::null(),
            1,
            core::ptr::addr_of_mut!(vk_barrier),
            0,
            core::ptr::null()
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_buffer_create(
    device: *mut wined3d_device,
    desc: *const wined3d_buffer_desc,
    data: *const wined3d_sub_resource_data,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    buffer: *mut *mut wined3d_buffer,
) -> HRESULT {
    WINE_TRACE!("device %p, desc %p, data %p, parent %p, parent_ops %p, buffer %p.\n", device, desc, data, parent, parent_ops, buffer);
    return (*(*(*device).adapter).adapter_ops)
        .adapter_create_buffer
        .unwrap_unchecked()(
        device, desc, data, parent, parent_ops, buffer
    );
}

unsafe extern "C" fn wined3d_streaming_buffer_prepare(
    device: *mut wined3d_device,
    buffer: *mut wined3d_streaming_buffer,
    min_size: core::ffi::c_uint,
) -> HRESULT {
    let mut wined3d_buffer: *mut wined3d_buffer = core::ptr::null_mut();
    let mut desc: wined3d_buffer_desc = core::mem::zeroed();
    let mut old_size: core::ffi::c_uint = 0;
    let mut size: core::ffi::c_uint;
    let mut hr: HRESULT;

    if !(*buffer).buffer.is_null() {
        old_size = (*(*buffer).buffer).resource.size as core::ffi::c_uint;
        if old_size >= min_size {
            return S_OK;
        }
    }

    size = max!(
        (SB_MIN_SIZE) as core::ffi::c_uint,
        max!(
            (old_size.wrapping_mul(2)) as core::ffi::c_uint,
            (min_size) as core::ffi::c_uint
        ),
    );

    TRACE!("Growing buffer to %u bytes.\n", size);

    desc.byte_width = size;
    desc.usage = WINED3DUSAGE_DYNAMIC;
    desc.bind_flags = (*buffer).bind_flags;
    desc.access = WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_MAP_W;
    desc.misc_flags = 0;
    desc.structure_byte_stride = 0;

    hr = wined3d_buffer_create(
        device,
        core::ptr::addr_of_mut!(desc),
        core::ptr::null::<wined3d_sub_resource_data>(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        core::ptr::addr_of_mut!(wined3d_buffer),
    );
    if hr >= 0 {
        if !(*buffer).buffer.is_null() {
            wined3d_buffer_decref((*buffer).buffer);
        }
        (*buffer).buffer = wined3d_buffer;
        (*buffer).pos = 0;
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_streaming_buffer_map(
    device: *mut wined3d_device,
    buffer: *mut wined3d_streaming_buffer,
    size: ::core::ffi::c_uint,
    stride: ::core::ffi::c_uint,
    ret_pos: *mut ::core::ffi::c_uint,
    ret_data: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut map_flags: ::core::ffi::c_uint = WINED3D_MAP_WRITE as ::core::ffi::c_uint;
    let mut resource: *mut wined3d_resource = ::core::ptr::null_mut::<wined3d_resource>();
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut pos: ::core::ffi::c_uint = 0;
    let mut align_0: ::core::ffi::c_uint = 0;
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut hr: HRESULT = 0;
    WINE_TRACE!("device %p, buffer %p, size %u, stride %u, ret_pos %p, ret_data %p.\n", device, buffer, size, stride, ret_pos, ret_data);
    hr = wined3d_streaming_buffer_prepare(device, buffer, size);
    if hr < 0 {
        return hr;
    }
    resource = &raw mut (*(*buffer).buffer).resource;

    pos = (*buffer).pos;
    align_0 = pos.wrapping_rem(stride);
    if align_0 != 0 {
        align_0 = stride.wrapping_sub(align_0);
    }
    if pos.wrapping_add(size).wrapping_add(align_0) > (*resource).size {
        pos = 0 as ::core::ffi::c_uint;
        map_flags |= WINED3D_MAP_DISCARD as ::core::ffi::c_uint;
    } else {
        pos = pos.wrapping_add(align_0);
        map_flags |= WINED3D_MAP_NOOVERWRITE as ::core::ffi::c_uint;
    }

    wined3d_box_set(
        &raw mut box_0,
        pos,
        0 as ::core::ffi::c_uint,
        pos.wrapping_add(size),
        1 as ::core::ffi::c_uint,
        0 as ::core::ffi::c_uint,
        1 as ::core::ffi::c_uint,
    );
    hr = wined3d_resource_map(
        resource,
        0 as ::core::ffi::c_uint,
        &raw mut map_desc,
        &raw mut box_0,
        map_flags as uint32_t,
    );
    if hr >= 0 {
        *ret_pos = pos;
        *ret_data = map_desc.data;
        (*buffer).pos = pos.wrapping_add(size);
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_streaming_buffer_unmap(buffer: *mut wined3d_streaming_buffer) {
    wined3d_resource_unmap(core::ptr::addr_of_mut!((*(*buffer).buffer).resource), 0);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_streaming_buffer_upload(device: *mut wined3d_device, buffer: *mut wined3d_streaming_buffer, data: *const core::ffi::c_void, size: core::ffi::c_uint, stride: core::ffi::c_uint, ret_pos: *mut core::ffi::c_uint) -> HRESULT {
    let mut dst_data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = wined3d_streaming_buffer_map(device, buffer, size, stride, ret_pos, core::ptr::addr_of_mut!(dst_data) as *mut *mut core::ffi::c_void);
    if hr >= 0 {
        memcpy(dst_data, data, size as usize);
        wined3d_streaming_buffer_unmap(buffer);
    }
    return hr;
}
