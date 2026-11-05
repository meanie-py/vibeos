use crate::ddraw::ddraw::ddraw_null_wined3d_parent_ops;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "ddraw";

unsafe extern "C" fn _dump_executedata(
    lpData: *const D3DEXECUTEDATA,
) {
    // TRACE("dwSize : %lu\n", lpData->dwSize);
    // TRACE("Vertex Offset : %lu Count : %lu\n", lpData->dwVertexOffset, lpData->dwVertexCount);
    // TRACE("Instruction Offset : %lu Length : %lu\n", lpData->dwInstructionOffset, lpData->dwInstructionLength);
    // TRACE("HVertex Offset : %lu\n", lpData->dwHVertexOffset);
}

unsafe extern "C" fn _dump_D3DEXECUTEBUFFERDESC(lpDesc: *const D3DEXECUTEBUFFERDESC) {
    // TRACE("dwSize : %lu\n", lpDesc->dwSize);
    // TRACE("dwFlags : %#lx\n", lpDesc->dwFlags);
    // TRACE("dwCaps : %#lx\n", lpDesc->dwCaps);
    // TRACE("dwBufferSize : %lu\n", lpDesc->dwBufferSize);
    // TRACE("lpData : %p\n", lpDesc->lpData);
}
#[no_mangle]
pub unsafe extern "C" fn d3d_execute_buffer_execute(
    buffer: *mut d3d_execute_buffer,
    device: *mut d3d_device,
) -> HRESULT {
    let mut is: DWORD = (*buffer).data.dwInstructionOffset;
    let mut instr: *mut ::core::ffi::c_char =
        ((*buffer).desc.lpData as *mut ::core::ffi::c_char).offset(is as isize);
    let mut i: ::core::ffi::c_uint = 0;
    let mut primitive_size: ::core::ffi::c_uint = 0;
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: ::core::ptr::null_mut(),
    };
    let mut box_0: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut hr: HRESULT = 0;

    TRACE!("ExecuteData :\n");
    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        _dump_executedata(::core::ptr::addr_of_mut!((*buffer).data));
    }

    'end_of_buffer: loop {
        let current: *mut D3DINSTRUCTION = instr as *mut D3DINSTRUCTION;
        let mut size: BYTE = 0;
        let mut count: WORD = 0;
        count = (*current).wCount;
        size = (*current).bSize;
        instr = instr.offset(::core::mem::size_of::<D3DINSTRUCTION>() as isize);
        primitive_size = 0;

        match ((*current).bOpcode) as u32 {
            D3DOP_POINT => {
                let p: *const D3DPOINT = instr as *const D3DPOINT;
                wined3d_device_context_set_primitive_type(
                    (*device).immediate_context,
                    WINED3D_PT_POINTLIST,
                    0,
                );
                wined3d_stateblock_set_stream_source(
                    (*device).state,
                    0,
                    (*buffer).dst_vertex_buffer,
                    0,
                    ::core::mem::size_of::<D3DTLVERTEX>() as UINT,
                );
                wined3d_stateblock_set_vertex_declaration(
                    (*device).state,
                    ddraw_find_decl((*device).ddraw, D3DFVF_TLVERTEX),
                );
                d3d_device_apply_state(device, (FALSE) as BOOL);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    wined3d_device_context_draw(
                        (*device).immediate_context,
                        (*p.offset(i as isize)).wFirst as ::core::ffi::c_uint,
                        (*p.offset(i as isize)).wCount as ::core::ffi::c_uint,
                        0,
                        0,
                    );
                    i = i.wrapping_add(1);
                }
                instr = instr.offset(
                    (::core::mem::size_of::<D3DPOINT>() as usize).wrapping_mul(count as usize)
                        as isize,
                );
                continue;
            }
            D3DOP_LINE => {
                primitive_size = 2;
                wined3d_device_context_set_primitive_type(
                    (*device).immediate_context,
                    WINED3D_PT_LINELIST,
                    0,
                );
                /* Drop through. */
            }
            D3DOP_TRIANGLE => {
                /* Triangle body follows the switch (reached by fall-through). */
            }
            D3DOP_MATRIXLOAD => {
                WARN!("MATRIXLOAD-s     (%u)\n", count as core::ffi::c_uint);
                instr = instr.offset((count as ::core::ffi::c_int * size as ::core::ffi::c_int) as isize);
                continue;
            }
            D3DOP_MATRIXMULTIPLY => {
                TRACE!("MATRIXMULTIPLY   (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_0: *mut D3DMATRIXMULTIPLY = instr as *mut D3DMATRIXMULTIPLY;
                    let mut a: *mut wined3d_matrix = ::core::ptr::null_mut();
                    let mut b: *mut wined3d_matrix = ::core::ptr::null_mut();
                    let mut c: *mut wined3d_matrix = ::core::ptr::null_mut();

                    a = ddraw_get_object(
                        ::core::ptr::null_mut(),
                        ((*ci_0).hDestMatrix as DWORD).wrapping_sub(1),
                        DDRAW_HANDLE_MATRIX,
                    ) as *mut wined3d_matrix;
                    b = ddraw_get_object(
                        ::core::ptr::null_mut(),
                        ((*ci_0).hSrcMatrix1 as DWORD).wrapping_sub(1),
                        DDRAW_HANDLE_MATRIX,
                    ) as *mut wined3d_matrix;
                    c = ddraw_get_object(
                        ::core::ptr::null_mut(),
                        ((*ci_0).hSrcMatrix2 as DWORD).wrapping_sub(1),
                        DDRAW_HANDLE_MATRIX,
                    ) as *mut wined3d_matrix;

                    if a.is_null() || b.is_null() || c.is_null() {
                        ERR!(
                            "Invalid matrix handle (a %#lx -> %p, b %#lx -> %p, c %#lx -> %p).\n",
                            (*ci_0).hDestMatrix,
                            a,
                            (*ci_0).hSrcMatrix1,
                            b,
                            (*ci_0).hSrcMatrix2,
                            c,
                        );
                    } else {
                        TRACE!("dst %p, src1 %p, src2 %p.\n", a, b, c);
                        multiply_matrix(a, c, b);
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_STATETRANSFORM => {
                TRACE!("STATETRANSFORM   (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_1: *mut D3DSTATE = instr as *mut D3DSTATE;
                    let mut m: *mut D3DMATRIX = ::core::ptr::null_mut();

                    m = ddraw_get_object(
                        ::core::ptr::null_mut(),
                        (*(*ci_1).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize))
                            .wrapping_sub(1),
                        DDRAW_HANDLE_MATRIX,
                    ) as *mut D3DMATRIX;
                    if m.is_null() {
                        ERR!(
                            "Invalid matrix handle %#lx.\n",
                            (*(*ci_1).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize)),
                        );
                    } else {
                        if (*ci_1).__bindgen_anon_1.dtstTransformStateType == D3DTRANSFORMSTATE_WORLD {
                            (*device).world = (*(*ci_1).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize))
                                as D3DMATRIXHANDLE;
                        }
                        if (*ci_1).__bindgen_anon_1.dtstTransformStateType == D3DTRANSFORMSTATE_VIEW {
                            (*device).view = (*(*ci_1).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize))
                                as D3DMATRIXHANDLE;
                        }
                        if (*ci_1).__bindgen_anon_1.dtstTransformStateType == D3DTRANSFORMSTATE_PROJECTION {
                            (*device).proj = (*(*ci_1).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize))
                                as D3DMATRIXHANDLE;
                        }
                        IDirect3DDevice3_SetTransform(
                            ::core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
                            (*ci_1).__bindgen_anon_1.dtstTransformStateType,
                            m as LPD3DMATRIX,
                        );
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_STATELIGHT => {
                TRACE!("STATELIGHT       (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_2: *mut D3DSTATE = instr as *mut D3DSTATE;

                    if IDirect3DDevice3_SetLightState(
                        ::core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
                        (*ci_2).__bindgen_anon_1.dlstLightStateType,
                        (*(*ci_2).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize)),
                    ) < 0
                    {
                        WARN!("Failed to set light state.\n");
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_STATERENDER => {
                TRACE!("STATERENDER      (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_3: *mut D3DSTATE = instr as *mut D3DSTATE;

                    if IDirect3DDevice3_SetRenderState(
                        ::core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
                        (*ci_3).__bindgen_anon_1.drstRenderStateType,
                        (*(*ci_3).__bindgen_anon_2.dwArg.as_ptr().add(0 as usize)),
                    ) < 0
                    {
                        WARN!("Failed to set render state.\n");
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_PROCESSVERTICES => {
                TRACE!("PROCESSVERTICES  (%d)\n", count as core::ffi::c_uint);

                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_4: *mut D3DPROCESSVERTICES = instr as *mut D3DPROCESSVERTICES;
                    let op: DWORD = (*ci_4).dwFlags & D3DPROCESSVERTICES_OPMASK;

                    TRACE!(
                        "  start %u, dest %u, count %lu, flags %#lx.\n",
                        (*ci_4).wStart as ::core::ffi::c_int,
                        (*ci_4).wDest as ::core::ffi::c_int,
                        (*ci_4).dwCount,
                        (*ci_4).dwFlags,
                    );

                    if (*ci_4).dwFlags & D3DPROCESSVERTICES_UPDATEEXTENTS != 0 {
                        FIXME!("D3DPROCESSVERTICES_UPDATEEXTENTS not implemented.\n");
                    }
                    if (*ci_4).dwFlags & D3DPROCESSVERTICES_NOCOLOR != 0 {
                        FIXME!("D3DPROCESSVERTICES_NOCOLOR not implemented.\n");
                    }

                    match op {
                        D3DPROCESSVERTICES_TRANSFORMLIGHT | D3DPROCESSVERTICES_TRANSFORM => {
                            wined3d_stateblock_set_stream_source(
                                (*device).state,
                                0,
                                (*buffer).src_vertex_buffer,
                                ((*buffer).src_vertex_pos as usize)
                                    .wrapping_mul(::core::mem::size_of::<D3DVERTEX>())
                                    as UINT,
                                ::core::mem::size_of::<D3DVERTEX>() as UINT,
                            );
                            wined3d_stateblock_set_render_state(
                                (*device).state,
                                WINED3D_RS_LIGHTING,
                                (op == D3DPROCESSVERTICES_TRANSFORMLIGHT && (*device).material != 0)
                                    as ::core::ffi::c_uint,
                            );
                            wined3d_stateblock_set_vertex_declaration(
                                (*device).state,
                                ddraw_find_decl(
                                    (*device).ddraw,
                                    (if op == D3DPROCESSVERTICES_TRANSFORMLIGHT {
                                        D3DFVF_VERTEX
                                    } else {
                                        D3DFVF_LVERTEX
                                    }) as DWORD,
                                ),
                            );
                            d3d_device_sync_surfaces(device);
                            wined3d_device_process_vertices(
                                (*device).wined3d_device,
                                (*device).state,
                                (*ci_4).wStart as UINT,
                                (*ci_4).wDest as UINT,
                                (*ci_4).dwCount as UINT,
                                (*buffer).dst_vertex_buffer,
                                ::core::ptr::null_mut(),
                                0,
                                D3DFVF_TLVERTEX,
                            );
                        }
                        D3DPROCESSVERTICES_COPY => {
                            box_0.left = (((*buffer)
                                .src_vertex_pos
                                .wrapping_add((*ci_4).wStart as ::core::ffi::c_uint))
                                as usize)
                                .wrapping_mul(::core::mem::size_of::<D3DTLVERTEX>())
                                as UINT;
                            box_0.right = ((box_0.left as usize).wrapping_add(
                                ((*ci_4).dwCount as usize)
                                    .wrapping_mul(::core::mem::size_of::<D3DTLVERTEX>()),
                            )) as UINT;
                            box_0.front = 0;
                            box_0.top = box_0.front;
                            box_0.back = 1;
                            box_0.bottom = box_0.back;
                            wined3d_device_context_copy_sub_resource_region(
                                (*device).immediate_context,
                                wined3d_buffer_get_resource((*buffer).dst_vertex_buffer),
                                0,
                                ((*ci_4).wDest as usize)
                                    .wrapping_mul(::core::mem::size_of::<D3DTLVERTEX>())
                                    as ::core::ffi::c_uint,
                                0,
                                0,
                                wined3d_buffer_get_resource((*buffer).src_vertex_buffer),
                                0,
                                ::core::ptr::addr_of_mut!(box_0),
                                0,
                            );
                        }
                        _ => {
                            FIXME!("Unhandled vertex processing op %#lx.\n", op);
                        }
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_TEXTURELOAD => {
                TRACE!("TEXTURELOAD    (%u)\n", count as core::ffi::c_uint);

                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_5: *mut D3DTEXTURELOAD = instr as *mut D3DTEXTURELOAD;
                    let mut dst: *mut ddraw_surface = ::core::ptr::null_mut();
                    let mut src: *mut ddraw_surface = ::core::ptr::null_mut();

                    instr = instr.offset(size as isize);

                    dst = ddraw_get_object(
                        ::core::ptr::null_mut(),
                        ((*ci_5).hDestTexture as DWORD).wrapping_sub(1),
                        DDRAW_HANDLE_SURFACE,
                    ) as *mut ddraw_surface;
                    if dst.is_null() {
                        WARN!("Invalid destination texture handle %#lx.\n", (*ci_5).hDestTexture);
                    } else {
                        src = ddraw_get_object(
                            ::core::ptr::null_mut(),
                            ((*ci_5).hSrcTexture as DWORD).wrapping_sub(1),
                            DDRAW_HANDLE_SURFACE,
                        ) as *mut ddraw_surface;
                        if src.is_null() {
                            WARN!("Invalid source texture handle %#lx.\n", (*ci_5).hSrcTexture);
                        } else {
                            IDirect3DTexture2_Load(
                                ::core::ptr::addr_of_mut!((*dst).IDirect3DTexture2_iface),
                                ::core::ptr::addr_of_mut!((*src).IDirect3DTexture2_iface),
                            );
                        }
                    }
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_EXIT => {
                TRACE!("EXIT             (%u)\n", count as core::ffi::c_uint);
                instr = instr.offset(size as isize);
                // C: `goto end_of_buffer;` -- out of the interpreter loop, to `return D3D_OK`.
                break 'end_of_buffer;
            }
            D3DOP_BRANCHFORWARD => {
                TRACE!("BRANCHFORWARD    (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    let ci_6: *mut D3DBRANCH = instr as *mut D3DBRANCH;

                    if ((*buffer).data.dsStatus.dwStatus & (*ci_6).dwMask) == (*ci_6).dwValue {
                        if (*ci_6).bNegate == 0 {
                            TRACE!(" Branch to %ld\n", (*ci_6).dwOffset);
                            if (*ci_6).dwOffset != 0 {
                                instr = (current as *mut ::core::ffi::c_char)
                                    .offset((*ci_6).dwOffset as isize);
                                break;
                            }
                        }
                    } else if (*ci_6).bNegate != 0 {
                        TRACE!(" Branch to %ld\n", (*ci_6).dwOffset);
                        if (*ci_6).dwOffset != 0 {
                            instr = (current as *mut ::core::ffi::c_char)
                                .offset((*ci_6).dwOffset as isize);
                            break;
                        }
                    }

                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            D3DOP_SPAN => {
                WARN!("SPAN-s           (%u)\n", count as core::ffi::c_uint);
                instr = instr.offset((count as ::core::ffi::c_int * size as ::core::ffi::c_int) as isize);
                continue;
            }
            D3DOP_SETSTATUS => {
                TRACE!("SETSTATUS        (%d)\n", count as core::ffi::c_uint);
                i = 0;
                while i < count as ::core::ffi::c_uint {
                    (*buffer).data.dsStatus = *(instr as *mut D3DSTATUS);
                    instr = instr.offset(size as isize);
                    i = i.wrapping_add(1);
                }
                continue;
            }
            _ => {
                ERR!(
                    "Unhandled OpCode %#x.\n",
                    (*current).bOpcode as ::core::ffi::c_int,
                );
                instr = instr.offset((count as ::core::ffi::c_int * size as ::core::ffi::c_int) as isize);
                continue;
            }
        }

        /* TRIANGLE processing (case COMMENT_TOKEN_ECHO____ body). */
        let mut indices: *mut WORD = ::core::ptr::null_mut();
        let mut index_pos: ::core::ffi::c_uint = (*buffer).index_pos;
        let mut index_count: ::core::ffi::c_uint = 0;

        TRACE!("TRIANGLE         (%d)\n", count as core::ffi::c_uint);
        if count == 0 {
            continue;
        }

        if primitive_size == 0 {
            wined3d_device_context_set_primitive_type(
                (*device).immediate_context,
                WINED3D_PT_TRIANGLELIST,
                0,
            );
            primitive_size = 3;
        }

        index_count = (count as ::core::ffi::c_uint).wrapping_mul(primitive_size);
        if (*buffer).index_size < index_count {
            let new_size: ::core::ffi::c_uint = max!(
                (*buffer).index_size.wrapping_mul(2),
                index_count,
            );
            let mut new_buffer: *mut wined3d_buffer = ::core::ptr::null_mut();
            let mut desc: wined3d_buffer_desc = wined3d_buffer_desc {
                byte_width: 0,
                usage: 0,
                bind_flags: 0,
                access: 0,
                misc_flags: 0,
                structure_byte_stride: 0,
            };

            desc.byte_width = (new_size as usize)
                .wrapping_mul(::core::mem::size_of::<WORD>()) as ::core::ffi::c_uint;
            desc.usage = WINED3DUSAGE_DYNAMIC | WINED3DUSAGE_STATICDECL;
            desc.bind_flags = WINED3D_BIND_INDEX_BUFFER;
            desc.access = WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_MAP_W;
            desc.misc_flags = 0;
            desc.structure_byte_stride = 0;

            hr = wined3d_buffer_create(
                (*device).wined3d_device,
                ::core::ptr::addr_of_mut!(desc),
                ::core::ptr::null(),
                ::core::ptr::null_mut(),
                ::core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
                ::core::ptr::addr_of_mut!(new_buffer),
            );
            if hr < 0 {
                return hr;
            }

            (*buffer).index_size = new_size;
            if !(*buffer).index_buffer.is_null() {
                wined3d_buffer_decref((*buffer).index_buffer);
            }
            (*buffer).index_buffer = new_buffer;
            index_pos = 0;
        } else if (*buffer).index_size.wrapping_sub(index_count) < index_pos {
            index_pos = 0;
        }

        box_0.left = (index_pos as usize).wrapping_mul(::core::mem::size_of::<WORD>()) as UINT;
        box_0.right = ((index_pos.wrapping_add(index_count)) as usize)
            .wrapping_mul(::core::mem::size_of::<WORD>()) as UINT;
        hr = wined3d_resource_map(
            wined3d_buffer_get_resource((*buffer).index_buffer),
            0,
            ::core::ptr::addr_of_mut!(map_desc),
            ::core::ptr::addr_of_mut!(box_0),
            WINED3D_MAP_WRITE
                | (if index_pos != 0 {
                    WINED3D_MAP_NOOVERWRITE
                } else {
                    WINED3D_MAP_DISCARD
                }),
        );
        if hr < 0 {
            return hr;
        }
        indices = map_desc.data as *mut WORD;

        i = 0;
        while i < count as ::core::ffi::c_uint {
            let ci: *mut D3DTRIANGLE = instr as *mut D3DTRIANGLE;

            TRACE!(
                "  v1: %d  v2: %d  v3: %d\n",
                (*ci).__bindgen_anon_1.v1 as ::core::ffi::c_int,
                (*ci).__bindgen_anon_2.v2 as ::core::ffi::c_int,
                (*ci).__bindgen_anon_3.v3 as ::core::ffi::c_int,
            );
            TRACE!("  Flags : ");

            if TRACE_ON!(__wine_dbch_ddraw) != 0 {
                /* Wireframe */
                if (*ci).wFlags & (D3DTRIFLAG_EDGEENABLE1 as u16) != 0 {
                    TRACE!("EDGEENABLE1 ");
                }
                if (*ci).wFlags & (D3DTRIFLAG_EDGEENABLE2 as u16) != 0 {
                    TRACE!("EDGEENABLE2 ");
                }
                if (*ci).wFlags & (D3DTRIFLAG_EDGEENABLE1 as u16) != 0 {
                    TRACE!("EDGEENABLE3 ");
                }
                /* Strips / Fans */
                if (*ci).wFlags == (D3DTRIFLAG_EVEN as u16) {
                    TRACE!("EVEN ");
                }
                if (*ci).wFlags == (D3DTRIFLAG_ODD as u16) {
                    TRACE!("ODD ");
                }
                if (*ci).wFlags == (D3DTRIFLAG_START as u16) {
                    TRACE!("START ");
                }
                if (*ci).wFlags > 0 && (*ci).wFlags < 30 {
                    TRACE!("STARTFLAT(%u) ", (*ci).wFlags as ::core::ffi::c_int);
                }
                TRACE!("\n");
            }

            match primitive_size {
                3 => {
                    *indices.offset(
                        i.wrapping_mul(primitive_size)
                            .wrapping_add(2) as isize,
                    ) = (*ci).__bindgen_anon_3.v3;
                    *indices.offset(
                        i.wrapping_mul(primitive_size)
                            .wrapping_add(1) as isize,
                    ) = (*ci).__bindgen_anon_2.v2;
                    *indices.offset(i.wrapping_mul(primitive_size) as isize) =
                        (*ci).__bindgen_anon_1.v1;
                }
                2 => {
                    *indices.offset(
                        i.wrapping_mul(primitive_size)
                            .wrapping_add(1) as isize,
                    ) = (*ci).__bindgen_anon_2.v2;
                    *indices.offset(i.wrapping_mul(primitive_size) as isize) =
                        (*ci).__bindgen_anon_1.v1;
                }
                _ => {}
            }

            instr = instr.offset(size as isize);
            i = i.wrapping_add(1);
        }

        wined3d_resource_unmap(
            wined3d_buffer_get_resource((*buffer).index_buffer),
            0,
        );

        wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            (*buffer).dst_vertex_buffer,
            0,
            ::core::mem::size_of::<D3DTLVERTEX>() as UINT,
        );
        wined3d_stateblock_set_vertex_declaration(
            (*device).state,
            ddraw_find_decl((*device).ddraw, D3DFVF_TLVERTEX),
        );
        wined3d_stateblock_set_index_buffer((*device).state, (*buffer).index_buffer, WINED3DFMT_R16_UINT);
        d3d_device_apply_state(device, (FALSE) as BOOL);
        wined3d_device_context_draw_indexed(
            (*device).immediate_context,
            0,
            index_pos,
            index_count,
            0,
            0,
        );

        (*buffer).index_pos = index_pos.wrapping_add(index_count);
    }

    return D3D_OK;
}

unsafe extern "C" fn impl_from_IDirect3DExecuteBuffer(
    iface: *mut IDirect3DExecuteBuffer,
) -> *mut d3d_execute_buffer {
    return CONTAINING_RECORD!(iface, d3d_execute_buffer, IDirect3DExecuteBuffer_iface);
}

unsafe extern "system" fn d3d_execute_buffer_QueryInterface(iface: *mut IDirect3DExecuteBuffer, iid: *const IID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DExecuteBuffer), iid)
            != false
            || IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), iid) != false
    {
        IDirect3DExecuteBuffer_AddRef(iface);
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(iid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d_execute_buffer_AddRef(iface: *mut IDirect3DExecuteBuffer) -> ULONG {
    let buffer: *mut d3d_execute_buffer = impl_from_IDirect3DExecuteBuffer(iface);
    let ref_: ULONG = InterlockedIncrement(&raw mut (*buffer).ref_) as ULONG;

    return ref_;
}

unsafe extern "system" fn d3d_execute_buffer_Release(iface: *mut IDirect3DExecuteBuffer) -> ULONG {
    let buffer: *mut d3d_execute_buffer = impl_from_IDirect3DExecuteBuffer(iface);
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*buffer).ref_) as *mut i32) as ULONG;

    if ref_0 == 0 {
        if (*buffer).need_free != 0 {
            free((*buffer).desc.lpData);
        }
        if !(*buffer).index_buffer.is_null() {
            wined3d_buffer_decref((*buffer).index_buffer);
        }
        if !(*buffer).dst_vertex_buffer.is_null() {
            wined3d_buffer_decref((*buffer).src_vertex_buffer);
            wined3d_buffer_decref((*buffer).dst_vertex_buffer);
        }
        free(buffer as *mut core::ffi::c_void);
    }

    return ref_0;
}
unsafe extern "system" fn d3d_execute_buffer_Initialize(
    iface: *mut IDirect3DExecuteBuffer,
    device: *mut IDirect3DDevice,
    desc: *mut D3DEXECUTEBUFFERDESC,
) -> HRESULT {
    // TRACE("iface %p, device %p, desc %p.\n", iface, device, desc);
    D3D_OK
}

unsafe extern "system" fn d3d_execute_buffer_Lock(
    iface: *mut IDirect3DExecuteBuffer,
    desc: *mut D3DEXECUTEBUFFERDESC,
) -> HRESULT {
    let buffer: *mut d3d_execute_buffer = impl_from_IDirect3DExecuteBuffer(iface);
    let mut dwSize: DWORD;

    dwSize = (*desc).dwSize;
    memcpy(
        desc as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*buffer).desc) as *const core::ffi::c_void,
        dwSize as usize,
    );

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        _dump_D3DEXECUTEBUFFERDESC(desc);
    }
    return D3D_OK;
}
unsafe extern "system" fn d3d_execute_buffer_Unlock(
    iface: *mut IDirect3DExecuteBuffer,
) -> HRESULT {
    // TRACE("iface %p.\n", iface);
    D3D_OK
}

unsafe extern "system" fn d3d_execute_buffer_SetExecuteData(iface: *mut IDirect3DExecuteBuffer, data: *mut D3DEXECUTEDATA) -> HRESULT {
    let buffer: *mut d3d_execute_buffer = impl_from_IDirect3DExecuteBuffer(iface);
    let mut map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut box_: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut buf_size: DWORD = (*buffer).desc.dwBufferSize;
    let mut copy_size: DWORD = 0;

    if (*data).dwSize != core::mem::size_of::<D3DEXECUTEDATA>() as DWORD {
        WARN!("data->dwSize is %lu, returning DDERR_INVALIDPARAMS.\n", (*data).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    /* Skip past previous vertex data. */
    (*buffer).src_vertex_pos = (*buffer).src_vertex_pos.wrapping_add((*buffer).data.dwVertexCount);

    if (*buffer).vertex_size < (*data).dwVertexCount {
        let new_size: DWORD = core::cmp::max((*data).dwVertexCount, (*buffer).vertex_size.wrapping_mul(2 as DWORD));
        let mut src_buffer: *mut wined3d_buffer = core::ptr::null_mut();
        let mut dst_buffer: *mut wined3d_buffer = core::ptr::null_mut();
        let mut desc: wined3d_buffer_desc = core::mem::zeroed();

        desc.byte_width = (new_size as usize).wrapping_mul(core::mem::size_of::<D3DVERTEX>()) as _;
        desc.usage = 0;
        desc.bind_flags = WINED3D_BIND_VERTEX_BUFFER;
        desc.access = WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
        desc.misc_flags = 0;
        desc.structure_byte_stride = 0;

        hr = wined3d_buffer_create(
            (*(*buffer).d3ddev).wined3d_device,
            core::ptr::addr_of_mut!(desc),
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
            core::ptr::addr_of_mut!(src_buffer),
        );
        if hr < 0 {
            return hr;
        }

        desc.byte_width = (new_size as usize).wrapping_mul(core::mem::size_of::<D3DTLVERTEX>()) as _;
        desc.usage = WINED3DUSAGE_STATICDECL;
        desc.access = WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_MAP_W;

        hr = wined3d_buffer_create(
            (*(*buffer).d3ddev).wined3d_device,
            core::ptr::addr_of_mut!(desc),
            core::ptr::null(),
            core::ptr::null_mut(),
            core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
            core::ptr::addr_of_mut!(dst_buffer),
        );
        if hr < 0 {
            wined3d_buffer_decref(src_buffer);
            return hr;
        }

        if !(*buffer).dst_vertex_buffer.is_null() {
            wined3d_buffer_decref((*buffer).src_vertex_buffer);
            wined3d_buffer_decref((*buffer).dst_vertex_buffer);
        }
        (*buffer).src_vertex_buffer = src_buffer;
        (*buffer).dst_vertex_buffer = dst_buffer;
        (*buffer).vertex_size = new_size;
        (*buffer).src_vertex_pos = 0;
    } else if (*buffer).vertex_size.wrapping_sub((*data).dwVertexCount) < (*buffer).src_vertex_pos {
        (*buffer).src_vertex_pos = 0;
    }

    if (*data).dwVertexCount != 0 && (buf_size == 0 || (*data).dwVertexOffset < buf_size) {
        box_.left = ((*buffer).src_vertex_pos as usize).wrapping_mul(core::mem::size_of::<D3DVERTEX>()) as _;
        box_.right = (box_.left as usize).wrapping_add(
            ((*data).dwVertexCount as usize).wrapping_mul(core::mem::size_of::<D3DVERTEX>()),
        ) as _;

        hr = wined3d_resource_map(
            wined3d_buffer_get_resource((*buffer).src_vertex_buffer),
            0,
            core::ptr::addr_of_mut!(map_desc),
            core::ptr::addr_of_mut!(box_),
            WINED3D_MAP_WRITE,
        );
        if hr < 0 {
            return hr;
        }

        copy_size = ((*data).dwVertexCount as usize).wrapping_mul(core::mem::size_of::<D3DVERTEX>()) as DWORD;
        if buf_size != 0 {
            copy_size = core::cmp::min(copy_size, buf_size.wrapping_sub((*data).dwVertexOffset));
        }

        memcpy(
            map_desc.data,
            ((*buffer).desc.lpData as *mut BYTE).add((*data).dwVertexOffset as usize) as *const core::ffi::c_void,
            copy_size as usize,
        );

        wined3d_resource_unmap(
            wined3d_buffer_get_resource((*buffer).src_vertex_buffer),
            0,
        );
    }

    memcpy(
        core::ptr::addr_of_mut!((*buffer).data) as *mut core::ffi::c_void,
        data as *const core::ffi::c_void,
        (*data).dwSize as usize,
    );

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        _dump_executedata(data);
    }

    return D3D_OK;
}

unsafe extern "system" fn d3d_execute_buffer_GetExecuteData(iface: *mut IDirect3DExecuteBuffer, data: *mut D3DEXECUTEDATA) -> HRESULT {
    let mut buffer: *mut d3d_execute_buffer = impl_from_IDirect3DExecuteBuffer(iface);

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        TRACE!("iface %p, data %p.\n", iface, data);
    }

    /* Tests show that dwSize is ignored. */
    memcpy(
        data as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!((*buffer).data) as *const core::ffi::c_void,
        core::mem::size_of::<D3DEXECUTEDATA>(),
    );

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        TRACE!("Returning data :\n");
        _dump_executedata(data);
    }

    return DD_OK;
}
unsafe extern "system" fn d3d_execute_buffer_Validate(
    iface: *mut IDirect3DExecuteBuffer,
    offset: *mut DWORD,
    callback: LPD3DVALIDATECALLBACK,
    context: *mut core::ffi::c_void,
    reserved: DWORD,
) -> HRESULT {
    // TRACE("iface %p, offset %p, callback %p, context %p, reserved %#lx.\n", iface, offset, callback, context, reserved);
    WARN!("Not implemented.\n");
    DDERR_UNSUPPORTED
}
unsafe extern "system" fn d3d_execute_buffer_Optimize(
    iface: *mut IDirect3DExecuteBuffer,
    reserved: DWORD,
) -> HRESULT {
    // TRACE("iface %p, reserved %#lx.\n", iface, reserved);
    WARN!("Not implemented.\n");
    DDERR_UNSUPPORTED
}
#[no_mangle]
pub unsafe extern "C" fn d3d_execute_buffer_init(
    mut execute_buffer: *mut d3d_execute_buffer,
    mut device: *mut d3d_device,
    mut desc: *mut D3DEXECUTEBUFFERDESC,
) -> HRESULT {
    (*execute_buffer).IDirect3DExecuteBuffer_iface.lpVtbl =
        core::ptr::addr_of!(d3d_execute_buffer_vtbl) as *const IDirect3DExecuteBufferVtbl;
    (*execute_buffer).ref_ = 1 as LONG;
    (*execute_buffer).d3ddev = device;

    /* Initializes memory */
    memcpy(
        core::ptr::addr_of_mut!((*execute_buffer).desc) as *mut core::ffi::c_void,
        desc as *const core::ffi::c_void,
        (*desc).dwSize as size_t,
    );

    /* No buffer given */
    if ((*execute_buffer).desc.dwFlags & (D3DDEB_LPDATA as DWORD)) == 0 {
        (*execute_buffer).desc.lpData = core::ptr::null_mut();
    }

    /* No buffer size given */
    if ((*execute_buffer).desc.dwFlags & (D3DDEB_BUFSIZE as DWORD)) == 0 {
        (*execute_buffer).desc.dwBufferSize = 0 as DWORD;
    }

    /* Create buffer if asked */
    if (*execute_buffer).desc.lpData.is_null() && (*execute_buffer).desc.dwBufferSize != 0 {
        (*execute_buffer).need_free = TRUE as BOOL;
        (*execute_buffer).desc.lpData = calloc(
            (1 as size_t) as _,
            ((*execute_buffer).desc.dwBufferSize as size_t) as _,
        );
        if (*execute_buffer).desc.lpData.is_null() {
            ERR!("Failed to allocate execute buffer data.\n");
            return DDERR_OUTOFMEMORY as HRESULT;
        }
    }

    (*execute_buffer).desc.dwFlags |= D3DDEB_LPDATA as DWORD;

    return D3D_OK as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DExecuteBuffer(iface: *mut IDirect3DExecuteBuffer) -> *mut d3d_execute_buffer {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl as *const _ == core::ptr::addr_of!(d3d_execute_buffer_vtbl));

    return impl_from_IDirect3DExecuteBuffer(iface);
}

// COM vtable tables — generated from executebuffer.c by vtbl_gen.py
/// executebuffer.c:742 `static const IDirect3DExecuteBufferVtbl d3d_execute_buffer_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_execute_buffer_vtbl: IDirect3DExecuteBufferVtbl = IDirect3DExecuteBufferVtbl {
    QueryInterface: Some(d3d_execute_buffer_QueryInterface),
    AddRef: Some(d3d_execute_buffer_AddRef),
    Release: Some(d3d_execute_buffer_Release),
    Initialize: Some(d3d_execute_buffer_Initialize),
    Lock: Some(d3d_execute_buffer_Lock),
    Unlock: Some(d3d_execute_buffer_Unlock),
    SetExecuteData: Some(d3d_execute_buffer_SetExecuteData),
    GetExecuteData: Some(d3d_execute_buffer_GetExecuteData),
    Validate: Some(d3d_execute_buffer_Validate),
    Optimize: Some(d3d_execute_buffer_Optimize),
};
