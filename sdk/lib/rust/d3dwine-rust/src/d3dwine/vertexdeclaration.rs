use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(d3d_decl)

#[repr(C)]
pub struct wined3d_fvf_convert_state {
    pub adapter: *const wined3d_adapter,
    pub elements: *mut wined3d_vertex_element,
    pub offset: core::ffi::c_uint,
    pub idx: core::ffi::c_uint,
}
pub type Pwined3d_fvf_convert_state = *mut wined3d_fvf_convert_state;

unsafe extern "C" fn dump_wined3d_vertex_element(element: *const wined3d_vertex_element) {
    WINE_TRACE!("                 format: %s (%#x)\n", debug_d3dformat((*element).format), (*element).format as u32);
    WINE_TRACE!("             input_slot: %u\n", (*element).input_slot);
    WINE_TRACE!("                 offset: %u\n", (*element).offset);
    WINE_TRACE!("            output_slot: %u\n", (*element).output_slot);
    WINE_TRACE!("       input slot class: %s\n", debug_d3dinput_classification((*element).input_slot_class));
    WINE_TRACE!("instance data step rate: %u\n", (*element).instance_data_step_rate);
    WINE_TRACE!("                 method: %s (%#x)\n", debug_d3ddeclmethod((*element).method as wined3d_decl_method), (*element).method as i32);
    WINE_TRACE!("                  usage: %s (%#x)\n", debug_d3ddeclusage((*element).usage as wined3d_decl_usage), (*element).usage as i32);
    WINE_TRACE!("              usage_idx: %u\n", (*element).usage_idx as i32);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vertex_declaration_incref(
    mut declaration: *mut wined3d_vertex_declaration,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*declaration).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", declaration, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_vertex_declaration_destroy_object(mut object: *mut ::core::ffi::c_void) {
    let mut declaration: *mut wined3d_vertex_declaration = object as *mut wined3d_vertex_declaration;
    free((*declaration).elements as *mut ::core::ffi::c_void);
    free(declaration as *mut ::core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vertex_declaration_decref(
    mut declaration: *mut wined3d_vertex_declaration,
) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedDecrement(
        core::ptr::addr_of_mut!((*declaration).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p decreasing refcount to %u.\n", declaration, refcount);
    if refcount == 0 {
        wined3d_mutex_lock();
        (*(*declaration).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*declaration).parent);
        wined3d_cs_destroy_object(
            (*(*declaration).device).cs as *mut wined3d_cs,
            Some(
                wined3d_vertex_declaration_destroy_object
                    as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            declaration as *mut ::core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_vertex_declaration_get_parent(
    declaration: *const wined3d_vertex_declaration,
) -> *mut core::ffi::c_void {
    // TRACE("declaration %p.\n", declaration);
    (*declaration).parent
}

unsafe extern "C" fn declaration_element_valid_ffp(
    element: *const wined3d_vertex_element,
) -> BOOL {
    match ((*element).usage) as u32 {
        WINED3D_DECL_USAGE_POSITION | WINED3D_DECL_USAGE_POSITIONT => match (*element).format {
            WINED3DFMT_R32G32_FLOAT | WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_R16G16_SINT | WINED3DFMT_R16G16B16A16_SINT | WINED3DFMT_R16G16_FLOAT | WINED3DFMT_R16G16B16A16_FLOAT => return (TRUE as i32),
            _ => return (FALSE as i32),
        },
        WINED3D_DECL_USAGE_BLEND_WEIGHT => match (*element).format {
            WINED3DFMT_R32_FLOAT | WINED3DFMT_R32G32_FLOAT | WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_B8G8R8A8_UNORM | WINED3DFMT_R8G8B8A8_UINT | WINED3DFMT_R16G16_SINT | WINED3DFMT_R16G16B16A16_SINT | WINED3DFMT_R16G16_FLOAT | WINED3DFMT_R16G16B16A16_FLOAT => return (TRUE as i32),
            _ => return (FALSE as i32),
        },
        WINED3D_DECL_USAGE_NORMAL => match (*element).format {
            WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_R16G16B16A16_SINT | WINED3DFMT_R16G16B16A16_FLOAT => return (TRUE as i32),
            _ => return (FALSE as i32),
        },
        WINED3D_DECL_USAGE_TEXCOORD => match (*element).format {
            WINED3DFMT_R32_FLOAT | WINED3DFMT_R32G32_FLOAT | WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_R16G16_SINT | WINED3DFMT_R16G16B16A16_SINT | WINED3DFMT_R16G16_FLOAT | WINED3DFMT_R16G16B16A16_FLOAT => return (TRUE as i32),
            _ => return (FALSE as i32),
        },
        WINED3D_DECL_USAGE_COLOR => match (*element).format {
            WINED3DFMT_R32G32B32_FLOAT | WINED3DFMT_R32G32B32A32_FLOAT | WINED3DFMT_B8G8R8A8_UNORM | WINED3DFMT_R8G8B8A8_UINT | WINED3DFMT_R16G16B16A16_SINT | WINED3DFMT_R8G8B8A8_UNORM | WINED3DFMT_R16G16B16A16_SNORM | WINED3DFMT_R16G16B16A16_UNORM | WINED3DFMT_R16G16B16A16_FLOAT => return (TRUE as i32),
            _ => return (FALSE as i32),
        },
        _ => return (FALSE as i32),
    };
}

unsafe extern "C" fn vertexdeclaration_init(
    mut declaration: *mut wined3d_vertex_declaration,
    mut device: *mut wined3d_device,
    mut elements: *const wined3d_vertex_element,
    mut element_count: UINT,
    mut parent: *mut ::core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
) -> HRESULT {
    let mut adapter: *const wined3d_adapter = (*device).adapter;
    let mut i: ::core::ffi::c_uint = 0;
    if ((TRACE_ON!(__wine_dbch_d3d_decl)) != 0) {
        i = 0 as ::core::ffi::c_uint;
        while i < element_count {
            dump_wined3d_vertex_element(elements.offset(i as isize));
            i = i.wrapping_add(1);
        }
    }
    (*declaration).ref_ = 1 as LONG;
    (*declaration).parent = parent;
    (*declaration).parent_ops = parent_ops;
    (*declaration).device = device;
    (*declaration).elements = calloc(
        (element_count as size_t) as _,
        (::core::mem::size_of::<wined3d_vertex_declaration_element>() as size_t) as _,
    ) as *mut wined3d_vertex_declaration_element;
    if (*declaration).elements.is_null() {
        ERR!("Failed to allocate elements memory.\n");
        return E_OUTOFMEMORY as HRESULT;
    }
    (*declaration).element_count = element_count as ::core::ffi::c_uint;

    i = 0 as ::core::ffi::c_uint;
    while i < element_count {
        let mut e: *mut wined3d_vertex_declaration_element =
            (*declaration).elements.offset(i as isize) as *mut wined3d_vertex_declaration_element;
        let mut alignment: ::core::ffi::c_uint = 0;
        (*e).format = wined3d_get_format(
            adapter,
            (*elements.offset(i as isize)).format,
            0 as ::core::ffi::c_uint,
        );
        (*e).ffp_valid = declaration_element_valid_ffp(
            elements.offset(i as isize) as *const wined3d_vertex_element
        );
        (*e).input_slot = (*elements.offset(i as isize)).input_slot;
        (*e).offset = (*elements.offset(i as isize)).offset;
        (*e).output_slot = (*elements.offset(i as isize)).output_slot;
        (*e).input_slot_class = (*elements.offset(i as isize)).input_slot_class;
        (*e).instance_data_step_rate = (*elements.offset(i as isize)).instance_data_step_rate;
        (*e).method = (*elements.offset(i as isize)).method;
        (*e).usage = (*elements.offset(i as isize)).usage;
        (*e).usage_idx = (*elements.offset(i as isize)).usage_idx;
        alignment = (*(*e).format).byte_count as ::core::ffi::c_uint;
        if alignment > 4 as ::core::ffi::c_uint {
            alignment = 4 as ::core::ffi::c_uint;
        }
        if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_POSITIONT as ::core::ffi::c_int {
            (*declaration).position_transformed = true;
        } else if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_PSIZE as ::core::ffi::c_int {
            (*declaration).point_size = true;
        } else if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int
            && (*e).usage_idx == 0
        {
            (*declaration).diffuse = true;
        } else if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_COLOR as ::core::ffi::c_int
            && (*e).usage_idx as ::core::ffi::c_int == 1 as ::core::ffi::c_int
        {
            (*declaration).specular = true;
        } else if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_NORMAL as ::core::ffi::c_int {
            (*declaration).normal = true;
        } else if (*e).usage as ::core::ffi::c_int == WINED3D_DECL_USAGE_TEXCOORD as ::core::ffi::c_int {
            (*declaration).texcoords = ((*declaration).texcoords as ::core::ffi::c_uint
                | (1 as ::core::ffi::c_uint) << (*e).usage_idx as ::core::ffi::c_int)
                as uint8_t;
        }
        if !((*e).input_slot >= WINED3D_MAX_STREAMS as ::core::ffi::c_uint) {
            if (*(*e).format).caps[WINED3D_GL_RES_TYPE_BUFFER as usize]
                & WINED3D_FORMAT_CAP_VERTEX_ATTRIBUTE as ::core::ffi::c_uint
                == 0
            {
                free((*declaration).elements as *mut ::core::ffi::c_void);
                return E_INVALIDARG as HRESULT;
            }
            if (*e).offset == WINED3D_APPEND_ALIGNED_ELEMENT as ::core::ffi::c_uint {
                let mut prev: *const wined3d_vertex_declaration_element =
                    ::core::ptr::null::<wined3d_vertex_declaration_element>();
                let mut j: ::core::ffi::c_uint = 0;
                (*e).offset = 0 as ::core::ffi::c_uint;
                j = 1 as ::core::ffi::c_uint;
                while j <= i {
                    prev = (*declaration).elements.offset(i.wrapping_sub(j) as isize)
                        as *mut wined3d_vertex_declaration_element;
                    if (*prev).input_slot == (*e).input_slot {
                        (*e).offset = (*prev)
                            .offset
                            .wrapping_add((*(*prev).format).byte_count as ::core::ffi::c_uint)
                            .wrapping_add(alignment)
                            .wrapping_sub(1 as ::core::ffi::c_uint)
                            & !alignment.wrapping_sub(1 as ::core::ffi::c_uint);
                        break;
                    } else {
                        j = j.wrapping_add(1);
                    }
                }
            }
            if (*e).offset & alignment.wrapping_sub(1 as ::core::ffi::c_uint) != 0 {
                WARN!(
                    "Declaration element %u with format %s and offset %u is not %u byte aligned.\n",
                    i,
                    debug_d3dformat((*elements.offset(i as isize)).format),
                    (*e).offset,
                    alignment,
                );
                free((*declaration).elements as *mut ::core::ffi::c_void);
                return E_INVALIDARG as HRESULT;
            }
        }
        i = i.wrapping_add(1);
    }

    return WINED3D_OK as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vertex_declaration_create(
    device: *mut wined3d_device,
    elements: *const wined3d_vertex_element,
    element_count: UINT,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    declaration: *mut *mut wined3d_vertex_declaration,
) -> HRESULT {
    let mut object: *mut wined3d_vertex_declaration = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<wined3d_vertex_declaration>()) as _) as *mut wined3d_vertex_declaration;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = vertexdeclaration_init(object, device, elements, element_count, parent, parent_ops);
    if hr < 0 {
        WARN!("Failed to initialize vertex declaration, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *declaration = object;

    return WINED3D_OK;
}

unsafe extern "C" fn append_decl_element(
    state: *mut wined3d_fvf_convert_state,
    format_id: wined3d_format_id,
    usage: wined3d_decl_usage,
    usage_idx: UINT,
) {
    let elements: *mut wined3d_vertex_element = (*state).elements;
    let mut format: *const wined3d_format = core::ptr::null();
    let offset: UINT = (*state).offset as UINT;
    let idx: UINT = (*state).idx as UINT;

    (*elements.offset(idx as isize)).format = format_id;
    (*elements.offset(idx as isize)).input_slot = 0;
    (*elements.offset(idx as isize)).offset = offset;
    (*elements.offset(idx as isize)).output_slot = WINED3D_OUTPUT_SLOT_SEMANTIC;
    (*elements.offset(idx as isize)).input_slot_class = WINED3D_INPUT_PER_VERTEX_DATA;
    (*elements.offset(idx as isize)).instance_data_step_rate = 0;
    (*elements.offset(idx as isize)).method = WINED3D_DECL_METHOD_DEFAULT as BYTE;
    (*elements.offset(idx as isize)).usage = usage as BYTE;
    (*elements.offset(idx as isize)).usage_idx = usage_idx as BYTE;

    format = wined3d_get_format((*state).adapter, format_id, 0);
    (*state).offset = (*state).offset.wrapping_add((*format).byte_count as UINT);
    (*state).idx = (*state).idx.wrapping_add(1);
}

unsafe extern "C" fn convert_fvf_to_declaration(
    adapter: *const wined3d_adapter,
    fvf: uint32_t,
    elements: *mut *mut wined3d_vertex_element,
) -> ::core::ffi::c_uint {
    let has_pos: BOOL = ((fvf & WINED3DFVF_POSITION_MASK) != 0) as BOOL;
    let has_blend: BOOL = ((fvf & WINED3DFVF_XYZB5) > WINED3DFVF_XYZRHW) as BOOL;
    let has_blend_idx: BOOL = (has_blend != 0
        && (((fvf & WINED3DFVF_XYZB5) == WINED3DFVF_XYZB5)
            || (fvf & WINED3DFVF_LASTBETA_D3DCOLOR) != 0
            || (fvf & WINED3DFVF_LASTBETA_UBYTE4) != 0)) as BOOL;
    let has_normal: BOOL = ((fvf & WINED3DFVF_NORMAL) != 0) as BOOL;
    let has_psize: BOOL = ((fvf & WINED3DFVF_PSIZE) != 0) as BOOL;
    let has_diffuse: BOOL = ((fvf & WINED3DFVF_DIFFUSE) != 0) as BOOL;
    let has_specular: BOOL = ((fvf & WINED3DFVF_SPECULAR) != 0) as BOOL;

    let num_textures: ::core::ffi::c_uint =
        ((fvf as ::core::ffi::c_uint & WINED3DFVF_TEXCOUNT_MASK as ::core::ffi::c_uint)
            >> WINED3DFVF_TEXCOUNT_SHIFT as ::core::ffi::c_uint) as ::core::ffi::c_uint;
    let texcoords: ::core::ffi::c_uint =
        ((fvf as ::core::ffi::c_uint & 0xffff0000 as ::core::ffi::c_uint)
            >> 16 as ::core::ffi::c_uint) as ::core::ffi::c_uint;
    let mut state: wined3d_fvf_convert_state = wined3d_fvf_convert_state {
        adapter: ::core::ptr::null::<wined3d_adapter>(),
        elements: ::core::ptr::null_mut::<wined3d_vertex_element>(),
        offset: 0,
        idx: 0,
    };
    let mut size: ::core::ffi::c_uint = 0;
    let mut idx: ::core::ffi::c_uint = 0;
    let mut num_blends: ::core::ffi::c_uint = (1 as ::core::ffi::c_uint)
        .wrapping_add(
            (((fvf as ::core::ffi::c_uint & WINED3DFVF_XYZB5 as ::core::ffi::c_uint)
                .wrapping_sub(WINED3DFVF_XYZB1 as ::core::ffi::c_uint))
                >> 1 as ::core::ffi::c_uint),
        );
    if has_blend_idx != 0 {
        num_blends = num_blends.wrapping_sub(1);
    }

    /* Compute declaration size */
    size = ((has_pos
        + (has_blend != 0 && num_blends > 0 as ::core::ffi::c_uint) as BOOL
        + has_blend_idx
        + has_normal
        + has_psize
        + has_diffuse
        + has_specular) as ::core::ffi::c_uint)
        .wrapping_add(num_textures);

    state.adapter = adapter;
    state.elements = calloc(
        (size as size_t) as _,
        (core::mem::size_of::<wined3d_vertex_element>() as size_t) as _,
    ) as *mut wined3d_vertex_element;
    if state.elements.is_null() {
        return !(0 as ::core::ffi::c_uint);
    }
    state.offset = 0 as ::core::ffi::c_uint;
    state.idx = 0 as ::core::ffi::c_uint;

    if has_pos != 0 {
        if has_blend == 0 && (fvf & WINED3DFVF_XYZRHW) != 0 {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_R32G32B32A32_FLOAT,
                WINED3D_DECL_USAGE_POSITIONT,
                0 as UINT,
            );
        } else if (fvf & WINED3DFVF_XYZW) == WINED3DFVF_XYZW {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_R32G32B32A32_FLOAT,
                WINED3D_DECL_USAGE_POSITION,
                0 as UINT,
            );
        } else {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_R32G32B32_FLOAT,
                WINED3D_DECL_USAGE_POSITION,
                0 as UINT,
            );
        }
    }

    if has_blend != 0 && (num_blends > 0 as ::core::ffi::c_uint) {
        if (fvf & WINED3DFVF_XYZB5) == WINED3DFVF_XYZB2 && (fvf & WINED3DFVF_LASTBETA_D3DCOLOR) != 0 {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_B8G8R8A8_UNORM,
                WINED3D_DECL_USAGE_BLEND_WEIGHT,
                0 as UINT,
            );
        } else {
            match num_blends {
                1 => {
                    append_decl_element(
                        core::ptr::addr_of_mut!(state),
                        WINED3DFMT_R32_FLOAT,
                        WINED3D_DECL_USAGE_BLEND_WEIGHT,
                        0 as UINT,
                    );
                }
                2 => {
                    append_decl_element(
                        core::ptr::addr_of_mut!(state),
                        WINED3DFMT_R32G32_FLOAT,
                        WINED3D_DECL_USAGE_BLEND_WEIGHT,
                        0 as UINT,
                    );
                }
                3 => {
                    append_decl_element(
                        core::ptr::addr_of_mut!(state),
                        WINED3DFMT_R32G32B32_FLOAT,
                        WINED3D_DECL_USAGE_BLEND_WEIGHT,
                        0 as UINT,
                    );
                }
                4 => {
                    append_decl_element(
                        core::ptr::addr_of_mut!(state),
                        WINED3DFMT_R32G32B32A32_FLOAT,
                        WINED3D_DECL_USAGE_BLEND_WEIGHT,
                        0 as UINT,
                    );
                }
                _ => {
                    ERR!("Unexpected amount of blend values: %u\n", num_blends);
                }
            }
        }
    }

    if has_blend_idx != 0 {
        if (fvf & WINED3DFVF_LASTBETA_UBYTE4) != 0
            || ((fvf & WINED3DFVF_XYZB5) == WINED3DFVF_XYZB2 && (fvf & WINED3DFVF_LASTBETA_D3DCOLOR) != 0)
        {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_R8G8B8A8_UINT,
                WINED3D_DECL_USAGE_BLEND_INDICES,
                0 as UINT,
            );
        } else if (fvf & WINED3DFVF_LASTBETA_D3DCOLOR) != 0 {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_B8G8R8A8_UNORM,
                WINED3D_DECL_USAGE_BLEND_INDICES,
                0 as UINT,
            );
        } else {
            append_decl_element(
                core::ptr::addr_of_mut!(state),
                WINED3DFMT_R32_FLOAT,
                WINED3D_DECL_USAGE_BLEND_INDICES,
                0 as UINT,
            );
        }
    }

    if has_normal != 0 {
        append_decl_element(
            core::ptr::addr_of_mut!(state),
            WINED3DFMT_R32G32B32_FLOAT,
            WINED3D_DECL_USAGE_NORMAL,
            0 as UINT,
        );
    }
    if has_psize != 0 {
        append_decl_element(
            core::ptr::addr_of_mut!(state),
            WINED3DFMT_R32_FLOAT,
            WINED3D_DECL_USAGE_PSIZE,
            0 as UINT,
        );
    }
    if has_diffuse != 0 {
        append_decl_element(
            core::ptr::addr_of_mut!(state),
            WINED3DFMT_B8G8R8A8_UNORM,
            WINED3D_DECL_USAGE_COLOR,
            0 as UINT,
        );
    }
    if has_specular != 0 {
        append_decl_element(
            core::ptr::addr_of_mut!(state),
            WINED3DFMT_B8G8R8A8_UNORM,
            WINED3D_DECL_USAGE_COLOR,
            1 as UINT,
        );
    }

    idx = 0 as ::core::ffi::c_uint;
    while idx < num_textures {
        match (texcoords >> idx.wrapping_mul(2 as ::core::ffi::c_uint))
            & (0x03 as ::core::ffi::c_uint)
        {
            WINED3DFVF_TEXTUREFORMAT1 => {
                append_decl_element(
                    core::ptr::addr_of_mut!(state),
                    WINED3DFMT_R32_FLOAT,
                    WINED3D_DECL_USAGE_TEXCOORD,
                    idx as UINT,
                );
            }
            WINED3DFVF_TEXTUREFORMAT2 => {
                append_decl_element(
                    core::ptr::addr_of_mut!(state),
                    WINED3DFMT_R32G32_FLOAT,
                    WINED3D_DECL_USAGE_TEXCOORD,
                    idx as UINT,
                );
            }
            WINED3DFVF_TEXTUREFORMAT3 => {
                append_decl_element(
                    core::ptr::addr_of_mut!(state),
                    WINED3DFMT_R32G32B32_FLOAT,
                    WINED3D_DECL_USAGE_TEXCOORD,
                    idx as UINT,
                );
            }
            WINED3DFVF_TEXTUREFORMAT4 => {
                append_decl_element(
                    core::ptr::addr_of_mut!(state),
                    WINED3DFMT_R32G32B32A32_FLOAT,
                    WINED3D_DECL_USAGE_TEXCOORD,
                    idx as UINT,
                );
            }
            _ => {}
        }
        idx = idx.wrapping_add(1);
    }

    *elements = state.elements;
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_vertex_declaration_create_from_fvf(
    mut device: *mut wined3d_device,
    mut fvf: uint32_t,
    mut parent: *mut core::ffi::c_void,
    mut parent_ops: *const wined3d_parent_ops,
    mut declaration: *mut *mut wined3d_vertex_declaration,
) -> HRESULT {
    let mut elements: *mut wined3d_vertex_element = core::ptr::null_mut();
    let mut size: u32;
    let mut hr: DWORD;

    size = convert_fvf_to_declaration((*device).adapter, fvf, core::ptr::addr_of_mut!(elements));
    if size == !(0) {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_vertex_declaration_create(
        device,
        elements,
        size as UINT,
        parent,
        parent_ops,
        declaration,
    ) as DWORD;
    free(elements as *mut core::ffi::c_void);
    return hr as HRESULT;
}
