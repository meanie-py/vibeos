use crate::d3dwine::context::context_resource_released;
use crate::d3dwine::device::device_resource_add;
use crate::d3dwine::device::device_resource_released;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_emit_preload_resource;
use crate::d3dwine::device::wined3d_device_context_map;
use crate::d3dwine::device::wined3d_device_context_unmap;
use crate::d3dwine::device::wined3d_device_get_available_texture_mem;
use crate::prelude::*;
use core::ffi::c_void;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_45 {
    pub type_0: wined3d_resource_type,
    pub cube_usage: DWORD,
    pub gl_type: wined3d_gl_resource_type,
}

unsafe extern "C" fn resource_check_usage(usage: uint32_t, access: ::core::ffi::c_uint) {
    static handled: uint32_t = (WINED3DUSAGE_DYNAMIC
        | WINED3DUSAGE_STATICDECL
        | WINED3DUSAGE_OVERLAY
        | WINED3DUSAGE_SCRATCH
        | WINED3DUSAGE_MANAGED
        | WINED3DUSAGE_CS
        | WINED3DUSAGE_LEGACY_CUBEMAP
        | !WINED3DUSAGE_MASK) as uint32_t;

    /* Write-only CPU access is supposed to result in write-combined mappings
     * being returned. OpenGL doesn't give us explicit control over that, but
     * the hints and access flags we set for typical access patterns on
     * dynamic resources should in theory have the same effect on the OpenGL
     * driver. */

    if (usage & !handled) != 0 {
        FIXME!("Unhandled usage flags %#x.\n", usage & !handled);
    }
    if (usage & WINED3DUSAGE_DYNAMIC as uint32_t) != 0 && (access & WINED3D_RESOURCE_ACCESS_MAP_R) != 0 {
        WINE_WARN_!(__wine_dbch_d3d_perf,"WINED3DUSAGE_DYNAMIC used with WINED3D_RESOURCE_ACCESS_MAP_R.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn resource_init(mut resource: *mut wined3d_resource, mut device: *mut wined3d_device, mut type_0: wined3d_resource_type, mut format: *const wined3d_format, mut multisample_type: wined3d_multisample_type, mut multisample_quality: ::core::ffi::c_uint, mut usage: ::core::ffi::c_uint, mut bind_flags: ::core::ffi::c_uint, mut access: ::core::ffi::c_uint, mut width: ::core::ffi::c_uint, mut height: ::core::ffi::c_uint, mut depth: ::core::ffi::c_uint, mut size: ::core::ffi::c_uint, mut parent: *mut ::core::ffi::c_void, mut parent_ops: *const wined3d_parent_ops, mut resource_ops: *const wined3d_resource_ops) -> HRESULT {
    let mut base_type: wined3d_gl_resource_type = WINED3D_GL_RES_TYPE_COUNT;
    let mut gl_type: wined3d_gl_resource_type = WINED3D_GL_RES_TYPE_COUNT;
    let mut i: ::core::ffi::c_uint = 0;
    static mut resource_types: [C2RustUnnamed_45; 6] = [
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_BUFFER,
            cube_usage: 0,
            gl_type: WINED3D_GL_RES_TYPE_BUFFER,
        },
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_TEXTURE_1D,
            cube_usage: 0,
            gl_type: WINED3D_GL_RES_TYPE_TEX_1D,
        },
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_TEXTURE_2D,
            cube_usage: 0,
            gl_type: WINED3D_GL_RES_TYPE_TEX_2D,
        },
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_TEXTURE_2D,
            cube_usage: 0,
            gl_type: WINED3D_GL_RES_TYPE_RB,
        },
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_TEXTURE_2D,
            cube_usage: WINED3DUSAGE_LEGACY_CUBEMAP,
            gl_type: WINED3D_GL_RES_TYPE_TEX_CUBE,
        },
        C2RustUnnamed_45 {
            type_0: WINED3D_RTYPE_TEXTURE_3D,
            cube_usage: 0,
            gl_type: WINED3D_GL_RES_TYPE_TEX_3D,
        },
    ];
    resource_check_usage(usage, access);
    if usage & WINED3DUSAGE_SCRATCH as uint32_t != 0
        && access & WINED3D_RESOURCE_ACCESS_GPU as ::core::ffi::c_uint != 0
    {
        ERR!(
            "Trying to create a scratch resource with access flags %s.\n",
            wined3d_debug_resource_access(access)
        );
        return WINED3DERR_INVALIDCALL;
    }
    if bind_flags
        & (WINED3D_BIND_RENDER_TARGET | WINED3D_BIND_DEPTH_STENCIL) as ::core::ffi::c_uint
        != 0
    {
        if access
            & (WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_GPU) as ::core::ffi::c_uint
            != WINED3D_RESOURCE_ACCESS_GPU as ::core::ffi::c_uint
        {
            WARN!(
                "Bind flags %s are incompatible with resource access %s.\n",
                wined3d_debug_bind_flags(bind_flags),
                wined3d_debug_resource_access(access)
            );
            return WINED3DERR_INVALIDCALL;
        }
        /* Dynamic and managed usages are incompatible with GPU writes. */
        if usage & (WINED3DUSAGE_DYNAMIC | WINED3DUSAGE_MANAGED) as uint32_t != 0 {
            WARN!(
                "Bind flags %s are incompatible with resource usage %s.\n",
                wined3d_debug_bind_flags(bind_flags),
                debug_d3dusage(usage)
            );
            return WINED3DERR_INVALIDCALL;
        }
    }
    if size == 0 {
        ERR!("Attempting to create a zero-sized resource.\n");
    }
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(resource_types) as usize {
        if !(resource_types[i as usize].type_0 as ::core::ffi::c_uint
            != type_0 as ::core::ffi::c_uint
            || resource_types[i as usize].cube_usage as uint32_t
                != usage & WINED3DUSAGE_LEGACY_CUBEMAP as uint32_t)
        {
            gl_type = resource_types[i as usize].gl_type;
            if base_type as ::core::ffi::c_uint
                == WINED3D_GL_RES_TYPE_COUNT as ::core::ffi::c_uint
            {
                base_type = gl_type;
            }
            if type_0 as ::core::ffi::c_uint
                == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
            {
                break;
            }
            if bind_flags & WINED3D_BIND_RENDER_TARGET as ::core::ffi::c_uint != 0
                && (*format).caps[gl_type as usize]
                    & WINED3D_FORMAT_CAP_RENDERTARGET as ::core::ffi::c_uint
                    == 0
            {
                WARN!(
                    "Format %s cannot be used for render targets.\n",
                    debug_d3dformat((*format).id)
                );
            } else if bind_flags & WINED3D_BIND_DEPTH_STENCIL as ::core::ffi::c_uint != 0
                && (*format).caps[gl_type as usize]
                    & WINED3D_FORMAT_CAP_DEPTH_STENCIL as ::core::ffi::c_uint
                    == 0
            {
                WARN!(
                    "Format %s cannot be used for depth/stencil buffers.\n",
                    debug_d3dformat((*format).id)
                );
            } else {
                if !(bind_flags & WINED3D_BIND_SHADER_RESOURCE as ::core::ffi::c_uint != 0
                    && (*format).caps[gl_type as usize]
                        & WINED3D_FORMAT_CAP_TEXTURE as ::core::ffi::c_uint
                        == 0)
                {
                    break;
                }
                WARN!(
                    "Format %s cannot be used for texturing.\n",
                    debug_d3dformat((*format).id)
                );
            }
        }
        i = i.wrapping_add(1);
    }
    if base_type as ::core::ffi::c_uint
        != WINED3D_GL_RES_TYPE_COUNT as ::core::ffi::c_uint
        && i as usize == ARRAY_SIZE!(resource_types) as usize
    {
        if usage & WINED3DUSAGE_SCRATCH as uint32_t != 0 {
            /* Needed for proper format information. */
            gl_type = base_type;
        } else {
            WARN!(
                "Did not find a suitable GL resource type for resource type %s.\n",
                debug_d3dresourcetype(type_0)
            );
            return WINED3DERR_INVALIDCALL;
        }
    }
    if base_type as ::core::ffi::c_uint
        != WINED3D_GL_RES_TYPE_COUNT as ::core::ffi::c_uint
        && ((*format).attrs
            & (WINED3D_FORMAT_ATTR_BLOCKS | WINED3D_FORMAT_ATTR_BLOCKS_NO_VERIFY) as ::core::ffi::c_uint)
            == WINED3D_FORMAT_ATTR_BLOCKS as ::core::ffi::c_uint
    {
        let mut width_mask: UINT = (*format).block_width.wrapping_sub(1 as UINT);
        let mut height_mask: UINT = (*format).block_height.wrapping_sub(1 as UINT);
        if width as UINT & width_mask != 0 || height as UINT & height_mask != 0 {
            return WINED3DERR_INVALIDCALL;
        }
    }
    (*resource).ref_ = 1 as LONG;
    (*resource).device = device;
    (*resource).type_ = type_0;
    (*resource).gl_type = gl_type;
    (*resource).format = format;
    (*resource).format_attrs = (*format).attrs;
    if (gl_type as ::core::ffi::c_uint)
        < WINED3D_GL_RES_TYPE_COUNT as ::core::ffi::c_uint
    {
        (*resource).format_caps = (*format).caps[gl_type as usize];
    }
    (*resource).multisample_type = multisample_type;
    (*resource).multisample_quality = multisample_quality as UINT;
    (*resource).usage = usage;
    (*resource).bind_flags = bind_flags;
    if (*resource).format_attrs & WINED3D_FORMAT_ATTR_MAPPABLE as ::core::ffi::c_uint != 0 {
        access |= WINED3D_RESOURCE_ACCESS_MAP_R as ::core::ffi::c_uint
            | WINED3D_RESOURCE_ACCESS_MAP_W as ::core::ffi::c_uint;
    }
    (*resource).access = access;
    (*resource).width = width as UINT;
    (*resource).height = height as UINT;
    (*resource).depth = depth as UINT;
    (*resource).size = size as UINT;
    (*resource).priority = 0;
    (*resource).parent = parent;
    (*resource).parent_ops = parent_ops;
    (*resource).resource_ops = resource_ops;
    (*resource).map_binding = WINED3D_LOCATION_SYSMEM as WORD;
    (*resource).heap_pointer = core::ptr::null_mut();
    (*resource).heap_memory = core::ptr::null_mut();
    /* Check that we have enough video ram left */
    if access & WINED3D_RESOURCE_ACCESS_CPU as ::core::ffi::c_uint == 0
        && usage & WINED3DUSAGE_VIDMEM_ACCOUNTING as uint32_t != 0
    {
        if size > wined3d_device_get_available_texture_mem(device) {
            ERR!("Out of adapter memory.\n");
            return WINED3DERR_OUTOFVIDEOMEMORY;
        }
        adapter_adjust_memory((*device).adapter, size as INT64);
    }
    if usage & WINED3DUSAGE_CS as uint32_t == 0 {
        device_resource_add(device, resource);
    }
    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_resource_destroy_object(mut object: *mut ::core::ffi::c_void) {
    let mut resource: *mut wined3d_resource = object as *mut wined3d_resource;
    WINE_TRACE!("resource %p.\n", resource);
    wined3d_resource_free_sysmem(resource);
    context_resource_released((*resource).device, resource);
}
#[no_mangle]
pub unsafe extern "C" fn resource_cleanup(resource: *mut wined3d_resource) {
    if ((*resource).access & WINED3D_RESOURCE_ACCESS_CPU) == 0
        && ((*resource).usage & WINED3DUSAGE_VIDMEM_ACCOUNTING) != 0
    {
        adapter_adjust_memory(
            (*(*resource).device).adapter,
            0 as INT64 - (*resource).size as INT64,
        );
    }

    if ((*resource).usage & WINED3DUSAGE_CS) == 0 {
        device_resource_released((*resource).device, resource);
    }

    wined3d_resource_reference(resource);
    wined3d_cs_destroy_object(
        (*(*resource).device).cs,
        Some(wined3d_resource_destroy_object as unsafe extern "C" fn(*mut core::ffi::c_void)),
        (resource).cast(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn resource_unload(resource: *mut wined3d_resource) {
    if (*resource).map_count != 0 {
        ERR!("Resource %p is being unloaded while mapped.\n", resource);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_set_priority(
    resource: *mut wined3d_resource,
    priority: u32,
) -> u32 {
    let mut prev: u32;

    if ((*resource).usage & WINED3DUSAGE_MANAGED) == 0 {
        WARN!("Called on non-managed resource %p, ignoring.\n", resource);
        return 0;
    }

    prev = (*resource).priority;
    (*resource).priority = priority;

    TRACE!(
        "resource %p, new priority %u, returning old priority %u.\n",
        resource,
        priority,
        prev
    );

    return prev;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_get_priority(
    resource: *const wined3d_resource,
) -> u32 {
    // TRACE("resource %p, returning %u.\n", resource, resource->priority);
    (*resource).priority
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_get_parent(
    resource: *const wined3d_resource,
) -> *mut core::ffi::c_void {
    (*resource).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_set_parent(resource: *mut wined3d_resource, parent: *mut core::ffi::c_void, parent_ops: *const wined3d_parent_ops) {
    (*resource).parent = parent;
    (*resource).parent_ops = parent_ops;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_get_desc(
    resource: *const wined3d_resource,
    desc: *mut wined3d_resource_desc,
) {
    (*desc).resource_type = (*resource).type_;
    (*desc).format = (*(*resource).format).id;
    (*desc).multisample_type = (*resource).multisample_type;
    (*desc).multisample_quality = (*resource).multisample_quality as ::core::ffi::c_uint;
    (*desc).usage = (*resource).usage as ::core::ffi::c_uint;
    (*desc).bind_flags = (*resource).bind_flags;
    (*desc).access = (*resource).access;
    (*desc).width = (*resource).width as ::core::ffi::c_uint;
    (*desc).height = (*resource).height as ::core::ffi::c_uint;
    (*desc).depth = (*resource).depth as ::core::ffi::c_uint;
    (*desc).size = (*resource).size as ::core::ffi::c_uint;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_map(
    resource: *mut wined3d_resource,
    sub_resource_idx: ::core::ffi::c_uint,
    map_desc: *mut wined3d_map_desc,
    box_: *const wined3d_box,
    flags: uint32_t,
) -> HRESULT {
    return wined3d_device_context_map(
        core::ptr::addr_of_mut!((*(*(*resource).device).cs).c),
        resource,
        sub_resource_idx,
        map_desc,
        box_,
        flags,
    );
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_unmap(resource: *mut wined3d_resource, sub_resource_idx: u32) -> HRESULT {
    WINE_TRACE!("resource %p, sub_resource_idx %u.\n", resource, sub_resource_idx);
    return wined3d_device_context_unmap(
        core::ptr::addr_of_mut!((*(*(*resource).device).cs).c),
        resource,
        sub_resource_idx,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_preload(resource: *mut wined3d_resource) {
    wined3d_cs_emit_preload_resource((*(*resource).device).cs as *mut wined3d_cs, resource);
}

unsafe extern "C" fn wined3d_resource_allocate_sysmem(resource: *mut wined3d_resource) -> BOOL {
    /* Overallocate and add padding to the allocated pointer, to guard against
     * games (for instance Railroad Tycoon 2) writing before the locked resource
     * memory pointer.
     */
    static align: SIZE_T = RESOURCE_ALIGNMENT as SIZE_T;
    let mut mem: *mut core::ffi::c_void = core::ptr::null_mut();

    mem = calloc(
        (1 as SIZE_T) as u32,
        (((*resource).size as SIZE_T).wrapping_add(align)) as u32,
    );
    if mem.is_null() {
        ERR!("Failed to allocate system memory.\n");
        return (FALSE) as BOOL;
    }

    (*resource).heap_memory = (((mem as ULONG_PTR).wrapping_add(align as ULONG_PTR))
        & !((RESOURCE_ALIGNMENT - 1) as ULONG_PTR))
        as *mut core::ffi::c_void;
    (*resource).heap_pointer = mem;

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_prepare_sysmem(resource: *mut wined3d_resource) -> BOOL {
    if !(*resource).heap_memory.is_null() {
        return (TRUE) as BOOL;
    }

    return wined3d_resource_allocate_sysmem(resource);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_free_sysmem(resource: *mut wined3d_resource) {
    if (*resource).heap_memory.is_null() {
        return;
    }
    (*resource).heap_memory = core::ptr::null_mut();

    free((*resource).heap_pointer);
    (*resource).heap_pointer = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_gl_storage_flags(resource: *const wined3d_resource) -> GLbitfield {
    let access: uint32_t = (*resource).access as uint32_t;
    let mut flags: GLbitfield = 0 as GLbitfield;

    if ((*resource).usage & WINED3DUSAGE_DYNAMIC as uint32_t) != 0 {
        flags |= GL_CLIENT_STORAGE_BIT as GLbitfield;
    }
    if (access & WINED3D_RESOURCE_ACCESS_CPU as uint32_t) == 0 {
        if (access & WINED3D_RESOURCE_ACCESS_MAP_W as uint32_t) != 0 {
            flags |= GL_MAP_WRITE_BIT as GLbitfield;
        }
        if (access & WINED3D_RESOURCE_ACCESS_MAP_R as uint32_t) != 0 {
            flags |= GL_MAP_READ_BIT as GLbitfield;
        }
    }

    return flags;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_gl_map_flags(bo: *const wined3d_bo_gl, d3d_flags: DWORD) -> GLbitfield {
    let mut ret: GLbitfield = 0 as GLbitfield;

    if (d3d_flags & (WINED3D_MAP_WRITE as DWORD)) != 0 {
        ret |= GL_MAP_WRITE_BIT as GLbitfield;
        if !(*bo).b.coherent {
            ret |= GL_MAP_FLUSH_EXPLICIT_BIT as GLbitfield;
        }
    }
    if (d3d_flags & (WINED3D_MAP_READ as DWORD)) != 0 {
        ret |= GL_MAP_READ_BIT as GLbitfield;
    } else {
        ret |= GL_MAP_UNSYNCHRONIZED_BIT as GLbitfield;
    }

    return ret;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_gl_legacy_map_flags(d3d_flags: DWORD) -> GLenum {
    let masked: ::core::ffi::c_uint = (d3d_flags as ::core::ffi::c_uint)
        & (WINED3D_MAP_READ | WINED3D_MAP_WRITE);
    if masked == WINED3D_MAP_READ {
        return GL_READ_ONLY_ARB as GLenum;
    } else if masked == WINED3D_MAP_WRITE {
        return GL_WRITE_ONLY_ARB as GLenum;
    } else {
        return GL_READ_WRITE_ARB as GLenum;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_is_offscreen(resource: *mut wined3d_resource) -> BOOL {
    let mut swapchain: *mut wined3d_swapchain;

    /* Only 2D texture resources can be onscreen. */
    if (*resource).type_ != WINED3D_RTYPE_TEXTURE_2D {
        return (TRUE) as BOOL;
    }

    /* Not on a swapchain - must be offscreen */
    swapchain = (*texture_from_resource(resource)).swapchain;
    if swapchain.is_null() {
        return (TRUE) as BOOL;
    }

    /* The front buffer is always onscreen */
    if resource == core::ptr::addr_of_mut!((*(*swapchain).front_buffer).resource) {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_update_draw_binding(resource: *mut wined3d_resource) {
    let d3d_info: *const wined3d_d3d_info = core::ptr::addr_of!((*(*(*resource).device).adapter).d3d_info);

    if wined3d_resource_is_offscreen(resource) == 0 {
        (*resource).draw_binding = WINED3D_LOCATION_DRAWABLE as WORD;
    } else if (*resource).multisample_type != 0 {
        (*resource).draw_binding = (*d3d_info).multisample_draw_location as WORD;
    } else if (*resource).gl_type as ::core::ffi::c_uint == WINED3D_GL_RES_TYPE_RB as ::core::ffi::c_uint {
        (*resource).draw_binding = WINED3D_LOCATION_RB_RESOLVED as WORD;
    } else {
        (*resource).draw_binding = WINED3D_LOCATION_TEXTURE_RGB as WORD;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_get_decompress_format(resource: *const wined3d_resource) -> *const wined3d_format {
    let adapter: *const wined3d_adapter = (*(*resource).device).adapter;
    if ((*resource).format_caps & (WINED3D_FORMAT_CAP_SRGB_READ | WINED3D_FORMAT_CAP_SRGB_WRITE)) != 0
        && ((*adapter).d3d_info.wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) == 0
    {
        return wined3d_get_format(adapter, WINED3DFMT_B8G8R8A8_UNORM_SRGB, (*resource).bind_flags);
    }
    return wined3d_get_format(adapter, WINED3DFMT_B8G8R8A8_UNORM, (*resource).bind_flags);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_get_sample_count(mut resource: *const wined3d_resource) -> ::core::ffi::c_uint {
    let mut format: *const wined3d_format = (*resource).format;
    if (*resource).multisample_type as ::core::ffi::c_uint
        == WINED3D_MULTISAMPLE_NON_MASKABLE as ::core::ffi::c_uint
    {
        let mut i: ::core::ffi::c_uint = 0;
        let mut count: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;
        i = 0 as ::core::ffi::c_uint;
        while (i as usize)
            < (::core::mem::size_of::<DWORD>() as usize)
                .wrapping_mul(CHAR_BIT as usize)
        {
            if (*format).multisample_types as ::core::ffi::c_uint
                & (1 as ::core::ffi::c_uint) << i
                != 0
            {
                let fresh15 = count;
                count = count.wrapping_add(1);
                if (*resource).multisample_quality == fresh15 {
                    break;
                }
            }
            i = i.wrapping_add(1);
        }
        return i.wrapping_add(1 as ::core::ffi::c_uint);
    }
    return (*resource).multisample_type as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_check_box_dimensions(resource: *mut wined3d_resource, sub_resource_idx: ::core::ffi::c_uint, box_: *const wined3d_box) -> HRESULT {
    let format: *const wined3d_format = (*resource).format;
    let mut desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut width_mask: core::ffi::c_uint = 0;
    let mut height_mask: core::ffi::c_uint = 0;

    wined3d_resource_get_sub_resource_desc(resource, sub_resource_idx, core::ptr::addr_of_mut!(desc));

    if (*box_).left >= (*box_).right
        || (*box_).right > desc.width
        || (*box_).top >= (*box_).bottom
        || (*box_).bottom > desc.height
        || (*box_).front >= (*box_).back
        || (*box_).back > desc.depth
    {
        WARN!("Box %s is invalid.\n", debug_box(box_));
        return WINEDDERR_INVALIDRECT;
    }

    if ((*resource).format_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0 {
        /* This assumes power of two block sizes, but NPOT block sizes would
         * be silly anyway.
         *
         * This also assumes that the format's block depth is 1. */
        width_mask = (*format).block_width.wrapping_sub(1);
        height_mask = (*format).block_height.wrapping_sub(1);

        if (((*box_).left as core::ffi::c_uint & width_mask) != 0)
            || (((*box_).top as core::ffi::c_uint & height_mask) != 0)
            || ((((*box_).right as core::ffi::c_uint & width_mask) != 0)
                && ((*box_).right != desc.width))
            || ((((*box_).bottom as core::ffi::c_uint & height_mask) != 0)
                && ((*box_).bottom != desc.height))
        {
            WARN!(
                "Box %s is misaligned for %ux%u blocks.\n",
                debug_box(box_),
                (*format).block_width,
                (*format).block_height
            );
            return WINED3DERR_INVALIDCALL;
        }
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn vk_access_mask_from_bind_flags(bind_flags: uint32_t) -> VkAccessFlags {
    let mut flags: VkAccessFlags = 0;

    if bind_flags & WINED3D_BIND_VERTEX_BUFFER != 0 {
        flags |= VK_ACCESS_VERTEX_ATTRIBUTE_READ_BIT;
    }
    if bind_flags & WINED3D_BIND_INDEX_BUFFER != 0 {
        flags |= VK_ACCESS_INDEX_READ_BIT;
    }
    if bind_flags & WINED3D_BIND_CONSTANT_BUFFER != 0 {
        flags |= VK_ACCESS_UNIFORM_READ_BIT;
    }
    if bind_flags & WINED3D_BIND_SHADER_RESOURCE != 0 {
        flags |= VK_ACCESS_SHADER_READ_BIT;
    }
    if bind_flags & WINED3D_BIND_UNORDERED_ACCESS != 0 {
        flags |= VK_ACCESS_SHADER_READ_BIT | VK_ACCESS_SHADER_WRITE_BIT;
    }
    if bind_flags & WINED3D_BIND_INDIRECT_BUFFER != 0 {
        flags |= VK_ACCESS_INDIRECT_COMMAND_READ_BIT;
    }
    if bind_flags & WINED3D_BIND_RENDER_TARGET != 0 {
        flags |= VK_ACCESS_COLOR_ATTACHMENT_READ_BIT | VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT;
    }
    if bind_flags & WINED3D_BIND_DEPTH_STENCIL != 0 {
        flags |= VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_READ_BIT | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT;
    }
    if bind_flags & WINED3D_BIND_STREAM_OUTPUT != 0 {
        flags |= VK_ACCESS_TRANSFORM_FEEDBACK_WRITE_BIT_EXT;
    }

    return flags;
}
#[no_mangle]
pub unsafe extern "C" fn vk_pipeline_stage_mask_from_bind_flags(bind_flags: uint32_t) -> VkPipelineStageFlags {
    let mut flags: VkPipelineStageFlags = 0 as VkPipelineStageFlags;

    if bind_flags & ((WINED3D_BIND_VERTEX_BUFFER | WINED3D_BIND_INDEX_BUFFER) as uint32_t) != 0 {
        flags |= VK_PIPELINE_STAGE_VERTEX_INPUT_BIT as VkPipelineStageFlags;
    }
    if bind_flags & ((WINED3D_BIND_CONSTANT_BUFFER | WINED3D_BIND_SHADER_RESOURCE | WINED3D_BIND_UNORDERED_ACCESS) as uint32_t) != 0 {
        flags |= (VK_PIPELINE_STAGE_VERTEX_SHADER_BIT | VK_PIPELINE_STAGE_TESSELLATION_CONTROL_SHADER_BIT
            | VK_PIPELINE_STAGE_TESSELLATION_EVALUATION_SHADER_BIT | VK_PIPELINE_STAGE_GEOMETRY_SHADER_BIT
            | VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT | VK_PIPELINE_STAGE_COMPUTE_SHADER_BIT) as VkPipelineStageFlags;
    }
    if bind_flags & (WINED3D_BIND_INDIRECT_BUFFER as uint32_t) != 0 {
        flags |= VK_PIPELINE_STAGE_DRAW_INDIRECT_BIT as VkPipelineStageFlags;
    }
    if bind_flags & (WINED3D_BIND_RENDER_TARGET as uint32_t) != 0 {
        flags |= VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as VkPipelineStageFlags;
    }
    if bind_flags & (WINED3D_BIND_DEPTH_STENCIL as uint32_t) != 0 {
        flags |= (VK_PIPELINE_STAGE_LATE_FRAGMENT_TESTS_BIT | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT) as VkPipelineStageFlags;
    }
    if bind_flags & (WINED3D_BIND_STREAM_OUTPUT as uint32_t) != 0 {
        flags |= VK_PIPELINE_STAGE_TRANSFORM_FEEDBACK_BIT_EXT as VkPipelineStageFlags;
    }

    return flags;
}
#[no_mangle]
pub unsafe extern "C" fn resource_offset_map_pointer(resource: *mut wined3d_resource, sub_resource_idx: ::core::ffi::c_uint, base_memory: *mut uint8_t, box_0: *const wined3d_box) -> *mut ::core::ffi::c_void {
    let format: *const wined3d_format = (*resource).format;
    let mut row_pitch: ::core::ffi::c_uint = 0;
    let mut slice_pitch: ::core::ffi::c_uint = 0;

    wined3d_resource_get_sub_resource_map_pitch(
        resource,
        sub_resource_idx,
        core::ptr::addr_of_mut!(row_pitch),
        core::ptr::addr_of_mut!(slice_pitch),
    );

    if ((*resource).format_attrs & (WINED3D_FORMAT_ATTR_BLOCKS | WINED3D_FORMAT_ATTR_BROKEN_PITCH))
        == WINED3D_FORMAT_ATTR_BLOCKS
    {
        /* Compressed textures are block based, so calculate the offset of
         * the block that contains the top-left pixel of the mapped box. */
        return base_memory
            .add(((*box_0).front as ::core::ffi::c_uint).wrapping_mul(slice_pitch) as usize)
            .add(
                ((*box_0).top as ::core::ffi::c_uint)
                    .wrapping_div((*format).block_height as ::core::ffi::c_uint)
                    .wrapping_mul(row_pitch) as usize,
            )
            .add(
                (*box_0)
                    .left
                    .wrapping_div((*format).block_width)
                    .wrapping_mul((*format).block_byte_count) as usize,
            ) as *mut ::core::ffi::c_void;
    } else {
        return base_memory
            .add(((*box_0).front as ::core::ffi::c_uint).wrapping_mul(slice_pitch) as usize)
            .add(((*box_0).top as ::core::ffi::c_uint).wrapping_mul(row_pitch) as usize)
            .add((*box_0).left.wrapping_mul((*format).byte_count) as usize)
            as *mut ::core::ffi::c_void;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_resource_memory_colour_fill(resource: *mut wined3d_resource, map: *const wined3d_map_desc, colour: *const wined3d_color, box_0: *const wined3d_box, full_subresource: bool) {
    let format: *const wined3d_format = (*resource).format;
    let mut w: u32 = 0;
    let mut h: u32 = 0;
    let mut d: u32 = 0;
    let mut x: u32 = 0;
    let mut y: u32 = 0;
    let mut z: u32 = 0;
    let mut bpp: u32 = 0;
    let mut dst: *mut u8 = core::ptr::null_mut();
    let mut dst2: *mut u8 = core::ptr::null_mut();
    let mut c: [u32; 4] = [0; 4];

    /* Fast and simple path for setting everything to zero. The C library's memset is
     * more sophisticated than our code below. Also this works for block formats, which
     * we still need to zero-initialize for newly created resources. */
    if full_subresource && (*colour).r == 0.0 && (*colour).g == 0.0 && (*colour).b == 0.0 && (*colour).a == 0.0
    {
        memset(
            (*map).data,
            0 as core::ffi::c_int,
            (*map).slice_pitch.wrapping_mul((*box_0).back) as usize,
        );
        return;
    }

    w = (*box_0).right.wrapping_sub((*box_0).left);
    h = (*box_0).bottom.wrapping_sub((*box_0).top);
    d = (*box_0).back.wrapping_sub((*box_0).front);

    dst = ((*map).data as *mut u8)
        .offset((*box_0).front.wrapping_mul((*map).slice_pitch) as isize)
        .offset(
            (*box_0)
                .top
                .wrapping_div((*format).block_height)
                .wrapping_mul((*map).row_pitch) as isize,
        )
        .offset(
            (*box_0)
                .left
                .wrapping_div((*format).block_width)
                .wrapping_mul((*format).block_byte_count) as isize,
        );

    wined3d_format_convert_from_float(format, colour, core::ptr::addr_of_mut!(c) as *mut core::ffi::c_void);
    bpp = (*format).byte_count;

    match bpp {
        1 => {
            x = 0;
            while x < w {
                *dst.offset(x as isize) = c[0] as u8;
                x = x.wrapping_add(1);
            }
        }
        2 => {
            x = 0;
            while x < w {
                *(dst as *mut u16).offset(x as isize) = c[0] as u16;
                x = x.wrapping_add(1);
            }
        }
        3 => {
            dst2 = dst;
            x = 0;
            while x < w {
                *dst2.offset(0 as isize) =
                    ((c[0]) & 0xff) as u8;
                *dst2.offset(1 as isize) =
                    (((c[0] >> 8) & 0xff)) as u8;
                *dst2.offset(2 as isize) =
                    (((c[0] >> 16) & 0xff)) as u8;
                x = x.wrapping_add(1);
                dst2 = dst2.offset(3 as isize);
            }
        }
        4 => {
            x = 0;
            while x < w {
                *(dst as *mut u32).offset(x as isize) = c[0];
                x = x.wrapping_add(1);
            }
        }
        8 | 12 | 16 => {
            x = 0;
            while x < w {
                memcpy(
                    ((*map).data as *mut u8).offset(x.wrapping_mul(bpp) as isize) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!(c) as *const core::ffi::c_void,
                    bpp as usize,
                );
                x = x.wrapping_add(1);
            }
        }
        _ => {
            FIXME!("Not implemented for bpp %u.\n", bpp);
            return;
        }
    }

    dst2 = dst;
    y = 1;
    while y < h {
        dst2 = dst2.offset((*map).row_pitch as isize);
        memcpy(
            dst2 as *mut core::ffi::c_void,
            dst as *const core::ffi::c_void,
            w.wrapping_mul(bpp) as usize,
        );
        y = y.wrapping_add(1);
    }

    dst2 = dst;
    z = 1;
    while z < d {
        dst2 = dst2.offset((*map).slice_pitch as isize);
        memcpy(
            dst2 as *mut core::ffi::c_void,
            dst as *const core::ffi::c_void,
            w.wrapping_mul(h).wrapping_mul(bpp) as usize,
        );
        z = z.wrapping_add(1);
    }
}
