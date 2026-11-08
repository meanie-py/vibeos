use crate::d3dwine::buffer::wined3d_buffer_get_memory;
use crate::d3dwine::buffer::wined3d_buffer_load;
use crate::d3dwine::buffer::wined3d_buffer_load_sysmem;
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn context_resource_released(device: *const wined3d_device, resource: *mut wined3d_resource) {
    let mut i: ::core::ffi::c_uint = 0;
    i = 0 as ::core::ffi::c_uint;
    while i < (*device).context_count {
        let mut context: *mut wined3d_context = *(*device).contexts.offset(i as isize);
        if core::ptr::addr_of_mut!((*(*context).current_rt.texture).resource) == resource {
            (*context).current_rt.texture = core::ptr::null_mut::<wined3d_texture>();
            (*context).current_rt.sub_resource_idx = 0 as ::core::ffi::c_uint;
        }
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_context_cleanup(context: *mut wined3d_context) {}
#[no_mangle]
pub unsafe extern "C" fn context_restore(mut context: *mut wined3d_context, mut texture: *mut wined3d_texture, mut sub_resource_idx: ::core::ffi::c_uint) {
    if !texture.is_null()
        && ((*context).current_rt.texture != texture
            || (*context).current_rt.sub_resource_idx != sub_resource_idx)
    {
        context_release(context);
        context = context_acquire((*texture).resource.device, texture, sub_resource_idx);
    }

    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn context_invalidate_compute_state(mut context: *mut wined3d_context, mut state_id: DWORD) {
    let mut representative: DWORD = ((*(*context).state_table.offset(state_id as isize)).representative as DWORD)
        .wrapping_sub(STATE_COMPUTE_OFFSET as DWORD);
    let mut index: ::core::ffi::c_uint = 0;
    let mut shift: ::core::ffi::c_uint = 0;
    index = (representative as usize)
        .wrapping_div((::core::mem::size_of::<uint32_t>() as usize).wrapping_mul(CHAR_BIT as usize))
        as ::core::ffi::c_uint;
    shift = (representative as usize
        & (::core::mem::size_of::<uint32_t>() as usize)
            .wrapping_mul(CHAR_BIT as usize)
            .wrapping_sub(1 as usize)) as ::core::ffi::c_uint;
    (*(*context).dirty_compute_states.as_mut_ptr().add(index as usize)) =
        ((*(*context).dirty_compute_states.as_ptr().add(index as usize)) as ::core::ffi::c_uint
            | (1 as ::core::ffi::c_uint) << shift) as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn context_invalidate_state(mut context: *mut wined3d_context, mut state_id: ::core::ffi::c_uint) {
    let mut representative: ::core::ffi::c_uint =
        (*(*context).state_table.add(state_id as usize)).representative;
    let mut index: ::core::ffi::c_uint;
    let mut shift: ::core::ffi::c_uint;

    if representative == 0 {
        ERR!("Invalidating representative 0, state_id %u.\n", state_id);
    }

    index = representative
        / ((::core::mem::size_of::<uint32_t>() as ::core::ffi::c_uint) * CHAR_BIT);
    shift = representative
        & (((::core::mem::size_of::<uint32_t>() as ::core::ffi::c_uint) * CHAR_BIT)
            - 1);
    (*context).dirty_graphics_states[index as usize] |=
        (1 << shift) as uint32_t;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_init(context: *mut wined3d_context, swapchain: *mut wined3d_swapchain) {
    let mut device: *mut wined3d_device = (*swapchain).device;
    let mut state: DWORD = 0;

    (*context).d3d_info = core::ptr::addr_of_mut!((*(*device).adapter).d3d_info);
    (*context).state_table = core::ptr::addr_of_mut!((*device).state_table) as *mut wined3d_state_entry;

    /* Mark all states dirty to force a proper initialization of the states on
     * the first use of the context. Compute states do not need initialization. */
    state = 0;
    while state <= (STATE_HIGHEST as DWORD) {
        if (*(*context).state_table.offset(state as isize)).representative != 0 && !STATE_IS_COMPUTE!(state) {
            context_invalidate_state(context, state);
        }
        state = state.wrapping_add(1);
    }

    (*context).device = device;
    (*context).swapchain = swapchain;
    (*context).current_rt.texture = (*swapchain).front_buffer;
    (*context).current_rt.sub_resource_idx = 0;

    (*context).set_shader_update_mask(((1 << WINED3D_SHADER_TYPE_PIXEL)
        | (1 << WINED3D_SHADER_TYPE_VERTEX)
        | (1 << WINED3D_SHADER_TYPE_GEOMETRY)
        | (1 << WINED3D_SHADER_TYPE_HULL)
        | (1 << WINED3D_SHADER_TYPE_DOMAIN)
        | (1 << WINED3D_SHADER_TYPE_COMPUTE)) as _);

    (*context).set_update_primitive_type((1) as _);
    (*context).set_update_patch_vertex_count((1) as _);
    (*context).set_update_multisample_state((1) as _);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_context_no3d_init(mut context_no3d: *mut wined3d_context, mut swapchain: *mut wined3d_swapchain) -> HRESULT {
    if (*__wine_dbch___default).flags as i32 & (1 as i32) << __WINE_DBCL_TRACE as i32 != 0 {
        let __dbch: *mut core::ffi::c_void = __wine_dbch___default as *mut core::ffi::c_void;
        let __dbcl: u32 = __WINE_DBCL_TRACE as u32;
        ros_dbg_log(
            (__dbcl) as __wine_debug_class,
            (__dbch).cast(),
            b"dll/directx/wine/wined3d/context.c\0"
                as *const u8 as *const i8,
            b"wined3d_context_no3d_init\0" as *const u8 as *const i8,
            124 as i32,
            b"context_no3d %p, swapchain %p.\n\0" as *const u8 as *const i8,
            context_no3d,
            swapchain,
        );
    }
    wined3d_context_init(context_no3d, swapchain);
    return WINED3D_OK;
}

unsafe extern "C" fn fixed_get_input(
    mut usage: BYTE,
    mut usage_idx: BYTE,
    mut regnum: *mut ::core::ffi::c_uint,
) -> BOOL {
    if (usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_POSITION as ::core::ffi::c_int
        || usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_POSITIONT as ::core::ffi::c_int)
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_POSITION as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_BLEND_WEIGHT as ::core::ffi::c_int
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_BLENDWEIGHT as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_BLEND_INDICES as ::core::ffi::c_int
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_BLENDINDICES as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_NORMAL as ::core::ffi::c_int
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_NORMAL as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_PSIZE as ::core::ffi::c_int
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_PSIZE as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int
        && usage_idx == 0
    {
        *regnum = WINED3D_FFP_DIFFUSE as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int
        && usage_idx as ::core::ffi::c_int == 1 as ::core::ffi::c_int
    {
        *regnum = WINED3D_FFP_SPECULAR as ::core::ffi::c_int as ::core::ffi::c_uint;
    } else if usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int
        && (usage_idx as ::core::ffi::c_int) < WINED3DDP_MAXTEXCOORD as ::core::ffi::c_int
    {
        *regnum = (WINED3D_FFP_TEXCOORD0 as ::core::ffi::c_int
            + usage_idx as ::core::ffi::c_int) as ::core::ffi::c_uint;
    } else {
        WARN!(
            "Unsupported input stream [usage=%s, usage_idx=%u].\n",
            debug_d3ddeclusage(usage as wined3d_decl_usage),
            usage_idx as ::core::ffi::c_int
        );
        *regnum = !(0 as ::core::ffi::c_uint);
        return FALSE as BOOL;
    }
    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_stream_info_from_declaration(stream_info: *mut wined3d_stream_info, state: *const wined3d_state, d3d_info: *const wined3d_d3d_info) {
    /* We need to deal with frequency data! */
    let use_vshader: BOOL = (use_vs(state) != 0
        || ((*d3d_info).ffp_hlsl() != 0
            && !(*state).shader[(WINED3D_SHADER_TYPE_VERTEX) as usize].is_null())) as BOOL;
    let declaration: *mut wined3d_vertex_declaration = (*state).vertex_declaration;
    let mut i: u32;

    (*stream_info).use_map = 0;
    (*stream_info).swizzle_map = 0;
    (*stream_info).set_position_transformed(0);

    if declaration.is_null() {
        return;
    }

    (*stream_info).set_position_transformed(((*declaration).position_transformed) as u32);

    /* Translate the declaration into strided data. */
    i = 0;
    while i < (*declaration).element_count {
        let element: *const wined3d_vertex_declaration_element =
            (*declaration).elements.add(i as usize);
        let stream: *const wined3d_stream_state = (core::ptr::addr_of!((*state).streams)
            as *const wined3d_stream_state)
            .add((*element).input_slot as usize);
        let mut stride_used: BOOL = 0;
        let mut idx: u32 = 0;

        TRACE!(
            "%p Element %p (%u of %u).\n",
            (*declaration).elements,
            element,
            i.wrapping_add(1),
            (*declaration).element_count
        );

        if !(*stream).buffer.is_null() {
            TRACE!(
                "offset %u input_slot %u usage_idx %d.\n",
                (*element).offset,
                (*element).input_slot,
                (*element).usage_idx as i32
            );

            if use_vshader != 0 {
                if (*element).output_slot == WINED3D_OUTPUT_SLOT_UNUSED {
                    stride_used = (FALSE as i32);
                } else if (*element).output_slot == WINED3D_OUTPUT_SLOT_SEMANTIC {
                    /* TODO: Assuming vertexdeclarations are usually used with the
                     * same or a similar shader, it might be worth it to store the
                     * last used output slot and try that one first. */
                    stride_used = vshader_get_input(
                        (*state).shader[(WINED3D_SHADER_TYPE_VERTEX) as usize],
                        (*element).usage as u8,
                        (*element).usage_idx as u8,
                        core::ptr::addr_of_mut!(idx),
                    ) as BOOL;
                } else {
                    idx = (*element).output_slot;
                    stride_used = (TRUE as i32);
                }
            } else {
                stride_used = fixed_get_input(
                    (*element).usage,
                    (*element).usage_idx,
                    core::ptr::addr_of_mut!(idx),
                );
            }

            if stride_used != 0 {
                TRACE!(
                    "Load %s array %u [usage %s, usage_idx %u, input_slot %u, offset %u, stride %u, format %s, class %s, step_rate %u].\n",
                    if use_vshader != 0 {
                        b"shader\0" as *const u8 as *const i8
                    } else {
                        b"fixed function\0" as *const u8 as *const i8
                    },
                    idx,
                    debug_d3ddeclusage((*element).usage as wined3d_decl_usage),
                    (*element).usage_idx as i32,
                    (*element).input_slot,
                    (*element).offset,
                    (*stream).stride,
                    debug_d3dformat((*(*element).format).id),
                    debug_d3dinput_classification((*element).input_slot_class),
                    (*element).instance_data_step_rate
                );

                (*stream_info).elements[idx as usize].format =
                    (*element).format as *const wined3d_format;
                (*stream_info).elements[idx as usize].data.buffer_object = core::ptr::null_mut();
                (*stream_info).elements[idx as usize].data.addr = (core::ptr::null_mut::<BYTE>())
                    .wrapping_add((*stream).offset as usize)
                    .wrapping_add((*element).offset as usize);
                (*stream_info).elements[idx as usize].stride = (*stream).stride as u32;
                (*stream_info).elements[idx as usize].stream_idx = (*element).input_slot;
                if ((*stream).flags as u32 & WINED3DSTREAMSOURCE_INSTANCEDATA) != 0 {
                    (*stream_info).elements[idx as usize].divisor = 1;
                    (*stream_info).elements[idx as usize].instanced = true;
                } else if (*element).input_slot_class == WINED3D_INPUT_PER_INSTANCE_DATA {
                    (*stream_info).elements[idx as usize].divisor =
                        (*element).instance_data_step_rate;
                    (*stream_info).elements[idx as usize].instanced = true;
                } else {
                    (*stream_info).elements[idx as usize].divisor = 0;
                    (*stream_info).elements[idx as usize].instanced = false;
                }

                if (*d3d_info).vertex_bgra() == 0
                    && (*(*element).format).id as u32 == WINED3DFMT_B8G8R8A8_UNORM
                {
                    (*stream_info).swizzle_map |= 1 << idx;
                }
                (*stream_info).use_map |= 1 << idx;
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn context_update_stream_info(context: *mut wined3d_context, state: *const wined3d_state) {
    let mut stream_info: *mut wined3d_stream_info = core::ptr::addr_of_mut!((*context).stream_info);
    let mut d3d_info: *const wined3d_d3d_info = (*context).d3d_info;
    let mut prev_all_vbo: DWORD = (*stream_info).all_vbo();
    let mut i: core::ffi::c_uint = 0;
    let mut map: uint32_t = 0;

    wined3d_stream_info_from_declaration(stream_info, state, d3d_info);

    (*stream_info).set_all_vbo((1) as _);
    map = (*stream_info).use_map as uint32_t;
    while map != 0 {
        let mut element: *mut wined3d_stream_info_element = core::ptr::null_mut::<wined3d_stream_info_element>();
        let mut data: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut buffer: *mut wined3d_buffer = core::ptr::null_mut::<wined3d_buffer>();

        i = wined3d_bit_scan(core::ptr::addr_of_mut!(map)) as core::ffi::c_uint;
        element = (core::ptr::addr_of_mut!((*stream_info).elements) as *mut wined3d_stream_info_element)
            .offset(i as isize);
        buffer = (*state).streams[(*element).stream_idx as usize].buffer as *mut wined3d_buffer;

        /* We can't use VBOs if the base vertex index is negative. OpenGL
         * doesn't accept negative offsets (or rather offsets bigger than the
         * VBO, because the pointer is unsigned), so use system memory
         * sources. In most sane cases the pointer - offset will still be > 0,
         * otherwise it will wrap around to some big value. Hope that with the
         * indices the driver wraps it back internally. If not,
         * draw_primitive_immediate_mode() is needed, including a vertex buffer
         * path. */
        if (*state).load_base_vertex_index < 0 {
            WINE_WARN_!(__wine_dbch_d3d_perf,
                "load_base_vertex_index is < 0 (%d), not using VBOs.\n",
                (*state).load_base_vertex_index,
            );
            (*element).data.buffer_object = core::ptr::null_mut();
            (*element).data.addr = (*element).data.addr.wrapping_add(
                wined3d_buffer_load_sysmem(buffer, context) as ULONG_PTR as usize,
            );
            if ((*element).data.addr as UINT_PTR)
                < (-(*state).load_base_vertex_index as core::ffi::c_uint)
                    .wrapping_mul((*element).stride) as UINT_PTR
            {
                FIXME!("System memory vertex data load offset is negative!\n");
            }
        } else {
            wined3d_buffer_load(buffer, context, state);
            wined3d_buffer_get_memory(buffer, context, core::ptr::addr_of_mut!(data));
            (*element).data.buffer_object = data.buffer_object;
            (*element).data.addr = (*element).data.addr.wrapping_add(data.addr as ULONG_PTR as usize);
        }

        if (*element).data.buffer_object.is_null() {
            (*stream_info).set_all_vbo((0) as _);
        }
    }

    if prev_all_vbo != (*stream_info).all_vbo() {
        context_invalidate_state(context, (STATE_INDEXBUFFER as u32));
    }
}
