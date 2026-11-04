use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_40 {
    pub format: wined3d_format_id,
    pub component_count: ::core::ffi::c_uint,
    pub component_size: ::core::ffi::c_uint,
}

static mut d3d_dtype_lookup: [C2RustUnnamed_40; 17] = [
    C2RustUnnamed_40 {
        format: WINED3DFMT_R32_FLOAT,
        component_count: 1 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_float>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R32G32_FLOAT,
        component_count: 2 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_float>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R32G32B32_FLOAT,
        component_count: 3 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_float>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R32G32B32A32_FLOAT,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_float>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_B8G8R8A8_UNORM,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<BYTE>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R8G8B8A8_UINT,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<BYTE>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16_SINT,
        component_count: 2 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16B16A16_SINT,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R8G8B8A8_UNORM,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<BYTE>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16_SNORM,
        component_count: 2 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16B16A16_SNORM,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16_UNORM,
        component_count: 2 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16B16A16_UNORM,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R10G10B10X2_UINT,
        component_count: 3 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R10G10B10X2_SNORM,
        component_count: 3 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16_FLOAT,
        component_count: 2 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
    C2RustUnnamed_40 {
        format: WINED3DFMT_R16G16B16A16_FLOAT,
        component_count: 4 as ::core::ffi::c_uint,
        component_size: ::core::mem::size_of::<::core::ffi::c_short>() as ::core::ffi::c_uint,
    },
];

static mut d3d9_vertexdeclaration_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            d3d9_vertexdeclaration_wined3d_object_destroyed,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d9)

#[repr(C)]
pub struct d3d_dtype_lookup_entry {
    pub format: wined3d_format_id,
    pub component_count: u32,
    pub component_size: u32,
}

unsafe extern "C" fn impl_from_IDirect3DVertexDeclaration9(iface: *mut IDirect3DVertexDeclaration9) -> *mut d3d9_vertex_declaration {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(d3d9_vertex_declaration, IDirect3DVertexDeclaration9_iface)) as *mut d3d9_vertex_declaration;
}
#[no_mangle]
pub unsafe extern "C" fn vdecl_convert_fvf(
    fvf: DWORD,
    ppVertexElements: *mut *mut D3DVERTEXELEMENT9,
) -> HRESULT {
    let mut idx: ::core::ffi::c_uint = 0;
    let mut idx2: ::core::ffi::c_uint = 0;
    let mut offset: ::core::ffi::c_uint = 0;
    let has_pos: BOOL = ((fvf & (D3DFVF_POSITION_MASK as DWORD)) != (0 as DWORD)) as BOOL;
    let has_blend: BOOL = ((fvf & (D3DFVF_XYZB5 as DWORD)) > (D3DFVF_XYZRHW as DWORD)) as BOOL;
    let has_blend_idx: BOOL = (has_blend != 0
        && (((fvf & (D3DFVF_XYZB5 as DWORD)) == (D3DFVF_XYZB5 as DWORD))
            || (fvf & (D3DFVF_LASTBETA_D3DCOLOR as DWORD)) != 0
            || (fvf & (D3DFVF_LASTBETA_UBYTE4 as DWORD)) != 0)) as BOOL;
    let has_normal: BOOL = ((fvf & (D3DFVF_NORMAL as DWORD)) != (0 as DWORD)) as BOOL;
    let has_psize: BOOL = ((fvf & (D3DFVF_PSIZE as DWORD)) != (0 as DWORD)) as BOOL;

    let has_diffuse: BOOL = ((fvf & (D3DFVF_DIFFUSE as DWORD)) != (0 as DWORD)) as BOOL;
    let has_specular: BOOL = ((fvf & (D3DFVF_SPECULAR as DWORD)) != (0 as DWORD)) as BOOL;

    let num_textures: DWORD = (fvf & (D3DFVF_TEXCOUNT_MASK as DWORD)) >> (D3DFVF_TEXCOUNT_SHIFT as DWORD);
    let texcoords: DWORD = (fvf & (0xFFFF0000 as DWORD)) >> (16 as DWORD);

    let end_element: D3DVERTEXELEMENT9 = D3DVERTEXELEMENT9 { Stream: 0xFF, Offset: 0, Type: D3DDECLTYPE_UNUSED as BYTE, Method: 0, Usage: 0, UsageIndex: 0 };
    let mut elements: *mut D3DVERTEXELEMENT9 = core::ptr::null_mut();

    let mut size: ::core::ffi::c_uint = 0;

    let mut num_blends: DWORD = (1 as DWORD).wrapping_add(
        ((fvf & (D3DFVF_XYZB5 as DWORD)).wrapping_sub(D3DFVF_XYZB1 as DWORD))
            >> (1 as DWORD),
    );
    if has_blend_idx != 0 {
        num_blends = num_blends.wrapping_sub(1);
    }

    /* Compute declaration size */
    size = ((has_pos
        + ((has_blend != 0 && num_blends > (0 as DWORD)) as BOOL)
        + has_blend_idx
        + has_normal
        + has_psize
        + has_diffuse
        + has_specular) as DWORD)
        .wrapping_add(num_textures)
        .wrapping_add(1 as DWORD) as ::core::ffi::c_uint;

    /* convert the declaration */
    elements = malloc(
        (size as usize).wrapping_mul(core::mem::size_of::<D3DVERTEXELEMENT9>()),
    ) as *mut D3DVERTEXELEMENT9;
    if elements.is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    *elements.offset(size.wrapping_sub(1 as ::core::ffi::c_uint) as isize) = end_element;
    idx = 0 as ::core::ffi::c_uint;
    if has_pos != 0 {
        if has_blend == 0 && (fvf & (D3DFVF_XYZRHW as DWORD)) != 0 {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT4 as BYTE;
            (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_POSITIONT as BYTE;
        } else if has_blend == 0 && (fvf & (D3DFVF_XYZW as DWORD)) == (D3DFVF_XYZW as DWORD) {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT4 as BYTE;
            (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_POSITION as BYTE;
        } else {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT3 as BYTE;
            (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_POSITION as BYTE;
        }
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_blend != 0 && (num_blends > (0 as DWORD)) {
        if ((fvf & (D3DFVF_XYZB5 as DWORD)) == (D3DFVF_XYZB2 as DWORD))
            && (fvf & (D3DFVF_LASTBETA_D3DCOLOR as DWORD)) != 0
        {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_D3DCOLOR as BYTE;
        } else {
            match num_blends {
                x if x == (1 as DWORD) => {
                    (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT1 as BYTE;
                }
                x if x == (2 as DWORD) => {
                    (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT2 as BYTE;
                }
                x if x == (3 as DWORD) => {
                    (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT3 as BYTE;
                }
                x if x == (4 as DWORD) => {
                    (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT4 as BYTE;
                }
                _ => {
                    ERR!("Unexpected amount of blend values: %lu\n", num_blends);
                }
            }
        }
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_BLENDWEIGHT as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_blend_idx != 0 {
        if (fvf & (D3DFVF_LASTBETA_UBYTE4 as DWORD)) != 0
            || (((fvf & (D3DFVF_XYZB5 as DWORD)) == (D3DFVF_XYZB2 as DWORD))
                && (fvf & (D3DFVF_LASTBETA_D3DCOLOR as DWORD)) != 0)
        {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_UBYTE4 as BYTE;
        } else if (fvf & (D3DFVF_LASTBETA_D3DCOLOR as DWORD)) != 0 {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_D3DCOLOR as BYTE;
        } else {
            (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT1 as BYTE;
        }
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_BLENDINDICES as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_normal != 0 {
        (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT3 as BYTE;
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_NORMAL as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_psize != 0 {
        (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT1 as BYTE;
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_PSIZE as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_diffuse != 0 {
        (*elements.offset(idx as isize)).Type = D3DDECLTYPE_D3DCOLOR as BYTE;
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_COLOR as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 0 as BYTE;
        idx = idx.wrapping_add(1);
    }
    if has_specular != 0 {
        (*elements.offset(idx as isize)).Type = D3DDECLTYPE_D3DCOLOR as BYTE;
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_COLOR as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = 1 as BYTE;
        idx = idx.wrapping_add(1);
    }
    idx2 = 0 as ::core::ffi::c_uint;
    while idx2 < num_textures {
        let mut numcoords: ::core::ffi::c_uint = ((texcoords as ::core::ffi::c_uint)
            >> (idx2.wrapping_mul(2 as ::core::ffi::c_uint)))
            & (0x03 as ::core::ffi::c_uint);
        match numcoords {
            x if x == (D3DFVF_TEXTUREFORMAT1 as ::core::ffi::c_uint) => {
                (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT1 as BYTE;
            }
            x if x == (D3DFVF_TEXTUREFORMAT2 as ::core::ffi::c_uint) => {
                (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT2 as BYTE;
            }
            x if x == (D3DFVF_TEXTUREFORMAT3 as ::core::ffi::c_uint) => {
                (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT3 as BYTE;
            }
            x if x == (D3DFVF_TEXTUREFORMAT4 as ::core::ffi::c_uint) => {
                (*elements.offset(idx as isize)).Type = D3DDECLTYPE_FLOAT4 as BYTE;
            }
            _ => {}
        }
        (*elements.offset(idx as isize)).Usage = D3DDECLUSAGE_TEXCOORD as BYTE;
        (*elements.offset(idx as isize)).UsageIndex = idx2 as BYTE;
        idx = idx.wrapping_add(1);
        idx2 = idx2.wrapping_add(1);
    }

    /* Now compute offsets, and initialize the rest of the fields */
    idx = 0 as ::core::ffi::c_uint;
    offset = 0 as ::core::ffi::c_uint;
    while idx < size.wrapping_sub(1 as ::core::ffi::c_uint) {
        (*elements.offset(idx as isize)).Stream = 0 as WORD;
        (*elements.offset(idx as isize)).Method = D3DDECLMETHOD_DEFAULT as BYTE;
        (*elements.offset(idx as isize)).Offset = offset as WORD;
        offset = offset.wrapping_add(
            d3d_dtype_lookup[(*elements.offset(idx as isize)).Type as usize]
                .component_count
                .wrapping_mul(
                    d3d_dtype_lookup[(*elements.offset(idx as isize)).Type as usize].component_size,
                ),
        );
        idx = idx.wrapping_add(1);
    }

    *ppVertexElements = elements;
    return D3D_OK;
}

unsafe extern "system" fn d3d9_vertex_declaration_QueryInterface(iface: *mut IDirect3DVertexDeclaration9, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(riid, core::ptr::addr_of!(IID_IDirect3DVertexDeclaration9))
        || IsEqualGUID!(riid, core::ptr::addr_of!(IID_IUnknown))
    {
        IDirect3DVertexDeclaration9_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn d3d9_vertex_declaration_AddRef(mut iface: *mut IDirect3DVertexDeclaration9) -> ULONG {
    let mut declaration: *mut d3d9_vertex_declaration = impl_from_IDirect3DVertexDeclaration9(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*declaration).refcount) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        IDirect3DDevice9Ex_AddRef((*declaration).parent_device);
        wined3d_vertex_declaration_incref((*declaration).wined3d_declaration);
    }
    return refcount;
}

unsafe extern "system" fn d3d9_vertex_declaration_Release(iface: *mut IDirect3DVertexDeclaration9) -> ULONG {
    let declaration: *mut d3d9_vertex_declaration = impl_from_IDirect3DVertexDeclaration9(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*declaration).refcount)) as ULONG;

    TRACE!("%p decreasing refcount to %lu.\n", iface, refcount);

    if refcount == 0 {
        let parent_device: *mut IDirect3DDevice9Ex = (*declaration).parent_device;
        wined3d_vertex_declaration_decref((*declaration).wined3d_declaration);

        /* Release the device last, as it may cause the device to be destroyed. */
        IDirect3DDevice9Ex_Release(parent_device);
    }

    return refcount;
}

unsafe extern "system" fn d3d9_vertex_declaration_GetDevice(iface: *mut IDirect3DVertexDeclaration9, device: *mut *mut IDirect3DDevice9) -> HRESULT {
    let mut declaration: *mut d3d9_vertex_declaration = impl_from_IDirect3DVertexDeclaration9(iface);

    *device = (*declaration).parent_device as *mut IDirect3DDevice9;
    IDirect3DDevice9_AddRef(*device);

    return D3D_OK;
}

unsafe extern "system" fn d3d9_vertex_declaration_GetDeclaration(
    iface: *mut IDirect3DVertexDeclaration9,
    elements: *mut D3DVERTEXELEMENT9,
    element_count: *mut UINT,
) -> HRESULT {
    let declaration: *mut d3d9_vertex_declaration = impl_from_IDirect3DVertexDeclaration9(iface);

    *element_count = (*declaration).element_count;

    /* Passing a NULL elements is used to just retrieve the number of elements */
    if elements.is_null() {
        return D3D_OK;
    }

    memcpy(
        elements as *mut core::ffi::c_void,
        (*declaration).elements as *const core::ffi::c_void,
        (core::mem::size_of::<D3DVERTEXELEMENT9>() as usize)
            .wrapping_mul((*declaration).element_count as usize),
    );

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DVertexDeclaration9(iface: *mut IDirect3DVertexDeclaration9) -> *mut d3d9_vertex_declaration {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    ASSERT!((*iface).lpVtbl == core::ptr::addr_of!(d3d9_vertex_declaration_vtbl));
    return (iface as usize).wrapping_sub(core::mem::offset_of!(d3d9_vertex_declaration, IDirect3DVertexDeclaration9_iface)) as *mut d3d9_vertex_declaration;
}
unsafe extern "system" fn d3d9_vertexdeclaration_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let declaration: *mut d3d9_vertex_declaration = parent as *mut d3d9_vertex_declaration;
    free((*declaration).elements as *mut core::ffi::c_void);
    free(declaration as *mut core::ffi::c_void);
}

unsafe extern "C" fn convert_to_wined3d_declaration(
    d3d9_elements: *const D3DVERTEXELEMENT9,
    wined3d_elements: *mut *mut wined3d_vertex_element,
    element_count: *mut UINT,
    stream_map: *mut DWORD,
) -> HRESULT {
    let mut element: *const D3DVERTEXELEMENT9 = d3d9_elements;
    let mut count: UINT = 1;
    let mut i: UINT;

    *stream_map = 0;

    element = d3d9_elements;
    loop {
        let fresh15 = element;
        element = element.offset(1);
        if !((*fresh15).Stream as UINT != 0xff && {
            let fresh16 = count;
            count = count.wrapping_add(1);
            fresh16 < 128
        }) {
            break;
        }
    }

    if count == 128 {
        return E_FAIL;
    }

    /* Skip the END element */
    count = count.wrapping_sub(1);

    *wined3d_elements = malloc(
        (count as usize).wrapping_mul(core::mem::size_of::<wined3d_vertex_element>() as usize),
    ) as *mut wined3d_vertex_element;

    if (*wined3d_elements).is_null() {
        return D3DERR_OUTOFVIDEOMEMORY;
    }

    i = 0;
    while i < count {
        if (*d3d9_elements.offset(i as isize)).Type as usize >= ARRAY_SIZE!(d3d_dtype_lookup) as usize {
            WARN!("Invalid element type %#x.\n", (*d3d9_elements.offset(i as isize)).Type as i32);
            free(*wined3d_elements as *mut core::ffi::c_void);
            return E_FAIL;
        }
        (*(*wined3d_elements).offset(i as isize)).format =
            d3d_dtype_lookup[(*d3d9_elements.offset(i as isize)).Type as usize].format;
        (*(*wined3d_elements).offset(i as isize)).input_slot =
            (*d3d9_elements.offset(i as isize)).Stream as UINT;
        (*(*wined3d_elements).offset(i as isize)).offset =
            (*d3d9_elements.offset(i as isize)).Offset as UINT;
        (*(*wined3d_elements).offset(i as isize)).output_slot = WINED3D_OUTPUT_SLOT_SEMANTIC;
        (*(*wined3d_elements).offset(i as isize)).input_slot_class = WINED3D_INPUT_PER_VERTEX_DATA;
        (*(*wined3d_elements).offset(i as isize)).instance_data_step_rate = 0;
        (*(*wined3d_elements).offset(i as isize)).method =
            (*d3d9_elements.offset(i as isize)).Method;
        (*(*wined3d_elements).offset(i as isize)).usage =
            (*d3d9_elements.offset(i as isize)).Usage;
        (*(*wined3d_elements).offset(i as isize)).usage_idx =
            (*d3d9_elements.offset(i as isize)).UsageIndex;
        *stream_map |= 1 << ((*d3d9_elements.offset(i as isize)).Stream as UINT);
        i = i.wrapping_add(1);
    }

    *element_count = count;

    return D3D_OK;
}

unsafe extern "C" fn vertexdeclaration_init(
    declaration: *mut d3d9_vertex_declaration,
    device: *mut d3d9_device,
    elements: *const D3DVERTEXELEMENT9,
) -> HRESULT {
    let mut wined3d_elements: *mut wined3d_vertex_element = core::ptr::null_mut();
    let mut wined3d_element_count: UINT = 0;
    let mut element_count: UINT = 0;
    let mut hr: HRESULT;

    hr = convert_to_wined3d_declaration(
        elements,
        core::ptr::addr_of_mut!(wined3d_elements),
        core::ptr::addr_of_mut!(wined3d_element_count),
        core::ptr::addr_of_mut!((*declaration).stream_map),
    );
    if hr < 0 {
        WARN!("Failed to create wined3d vertex declaration elements, hr %#lx.\n", hr);
        return hr;
    }

    (*declaration).IDirect3DVertexDeclaration9_iface.lpVtbl =
        core::ptr::addr_of!(d3d9_vertex_declaration_vtbl) as *const IDirect3DVertexDeclaration9Vtbl;
    (*declaration).refcount = 1;

    element_count = wined3d_element_count + 1;
    (*declaration).elements = malloc(
        (element_count as usize).wrapping_mul(core::mem::size_of::<D3DVERTEXELEMENT9>()),
    ) as *mut D3DVERTEXELEMENT9;
    if (*declaration).elements.is_null() {
        free(wined3d_elements as *mut core::ffi::c_void);
        ERR!("Failed to allocate vertex declaration elements memory.\n");
        return D3DERR_OUTOFVIDEOMEMORY;
    }
    memcpy(
        (*declaration).elements as *mut core::ffi::c_void,
        elements as *const core::ffi::c_void,
        (element_count as usize).wrapping_mul(core::mem::size_of::<D3DVERTEXELEMENT9>()),
    );
    (*declaration).element_count = element_count;

    wined3d_mutex_lock();
    hr = wined3d_vertex_declaration_create(
        (*device).wined3d_device,
        wined3d_elements,
        wined3d_element_count,
        declaration as *mut core::ffi::c_void,
        core::ptr::addr_of!(d3d9_vertexdeclaration_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*declaration).wined3d_declaration),
    );
    wined3d_mutex_unlock();
    free(wined3d_elements as *mut core::ffi::c_void);
    if hr < 0 {
        free((*declaration).elements as *mut core::ffi::c_void);
        WARN!("Failed to create wined3d vertex declaration, hr %#lx.\n", hr);
        if hr == E_INVALIDARG {
            hr = E_FAIL;
        }
        return hr;
    }

    (*declaration).parent_device = core::ptr::addr_of_mut!((*device).IDirect3DDevice9Ex_iface);
    IDirect3DDevice9Ex_AddRef((*declaration).parent_device);

    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d9_vertex_declaration_create(device: *mut d3d9_device, elements: *const D3DVERTEXELEMENT9, declaration: *mut *mut d3d9_vertex_declaration) -> HRESULT {
    let mut object: *mut d3d9_vertex_declaration = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc(((1) as usize) as _, (core::mem::size_of::<d3d9_vertex_declaration>()) as _) as *mut d3d9_vertex_declaration;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = vertexdeclaration_init(object, device, elements);
    if hr < 0 {
        WARN!("Failed to initialize vertex declaration, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *declaration = object;

    return D3D_OK;
}

// COM vtable tables — generated from vertexdeclaration.c by vtbl_gen.py
/// vertexdeclaration.c:286 `static const IDirect3DVertexDeclaration9Vtbl d3d9_vertex_declaration_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d9_vertex_declaration_vtbl: IDirect3DVertexDeclaration9Vtbl = IDirect3DVertexDeclaration9Vtbl {
    QueryInterface: Some(d3d9_vertex_declaration_QueryInterface),
    AddRef: Some(d3d9_vertex_declaration_AddRef),
    Release: Some(d3d9_vertex_declaration_Release),
    GetDevice: Some(d3d9_vertex_declaration_GetDevice),
    GetDeclaration: Some(d3d9_vertex_declaration_GetDeclaration),
};
