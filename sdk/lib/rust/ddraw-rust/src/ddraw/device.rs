use crate::ddraw::main::global_handle_table;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tss_lookup_tag {
    pub sampler_state: BOOL,
    pub u: C2RustUnnamed_72,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_72 {
    pub texture_state: wined3d_texture_stage_state,
    pub sampler_state: wined3d_sampler_state,
}

#[no_mangle]
pub static mut IID_D3DDEVICE_WineD3D: GUID = _GUID {
    Data1: 0xaef72d43 as ::core::ffi::c_uint,
    Data2: 0xb09a as ::core::ffi::c_ushort,
    Data3: 0x4b7b as ::core::ffi::c_ushort,
    Data4: [
        0xb7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x98 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x77 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2a as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut tss_lookup: [tss_lookup_tag; 25] = [
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_INVALID,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_COLOR_OP,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_COLOR_ARG1,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_COLOR_ARG2,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_ALPHA_OP,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_ALPHA_ARG1,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_ALPHA_ARG2,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_MAT00,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_MAT01,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_MAT10,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_MAT11,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_TEXCOORD_INDEX,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_ADDRESS_U,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_ADDRESS_U,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_ADDRESS_V,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_BORDER_COLOR,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MAG_FILTER,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MIN_FILTER,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MIP_FILTER,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MIPMAP_LOD_BIAS,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MAX_MIP_LEVEL,
        },
    },
    tss_lookup_tag {
        sampler_state: (TRUE as i32),
        u: C2RustUnnamed_72 {
            sampler_state: WINED3D_SAMP_MAX_ANISOTROPY,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_LSCALE,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_BUMPENV_LOFFSET,
        },
    },
    tss_lookup_tag {
        sampler_state: (FALSE as i32),
        u: C2RustUnnamed_72 {
            texture_state: WINED3D_TSS_TEXTURE_TRANSFORM_FLAGS,
        },
    },
];

#[repr(C)]
pub union TssLookupU {
    pub texture_state: i32,
    pub sampler_state: i32,
}

#[repr(C)]
pub struct TssLookup {
    pub sampler_state: BOOL,
    pub u: TssLookupU,
}

pub type PTssLookup = *mut TssLookup;


unsafe extern "C" fn set_fpu_control_word(mut fpucw: WORD) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        core::arch::asm!(
            "fldcw word ptr [{0}]",
            in(reg) core::ptr::addr_of!(fpucw),
            options(nostack),
        );
    }
}

unsafe extern "C" fn d3d_fpu_setup() -> WORD {
    let mut oldcw: WORD = 0;
    static mut warned: BOOL = (FALSE as i32);
    if warned == 0 {
        WINE_FIXME!("FPUPRESERVE not implemented for this platform / compiler\n");
        warned = (TRUE as i32);
    }
    return 0;

    set_fpu_control_word(0x37f);

    return oldcw;
}

unsafe extern "C" fn wined3d_render_state_from_ddraw(mut state: D3DRENDERSTATETYPE) -> wined3d_render_state {
    match state as ::core::ffi::c_uint {
        D3DRENDERSTATE_ZBIAS => {
            return WINED3D_RS_DEPTHBIAS;
        }
        D3DRENDERSTATE_EDGEANTIALIAS => {
            return WINED3D_RS_ANTIALIASEDLINEENABLE;
        }
        _ => {
            return state as wined3d_render_state;
        }
    }
}

unsafe extern "C" fn wined3d_transform_state_from_ddraw(state: D3DTRANSFORMSTATETYPE) -> wined3d_transform_state {
    match state as u32 {
        D3DTRANSFORMSTATE_WORLD => {
            return WINED3D_TS_WORLD_MATRIX!(0);
        }
        D3DTRANSFORMSTATE_WORLD1 => {
            return WINED3D_TS_WORLD_MATRIX!(1);
        }
        D3DTRANSFORMSTATE_WORLD2 => {
            return WINED3D_TS_WORLD_MATRIX!(2);
        }
        D3DTRANSFORMSTATE_WORLD3 => {
            return WINED3D_TS_WORLD_MATRIX!(3);
        }
        _ => return state as wined3d_transform_state,
    }
}

unsafe extern "C" fn wined3d_primitive_type_from_ddraw(type_0: D3DPRIMITIVETYPE) -> wined3d_primitive_type {
    return type_0 as wined3d_primitive_type;
}

unsafe extern "C" fn wined3d_stateblock_type_from_ddraw(type_: D3DSTATEBLOCKTYPE) -> wined3d_stateblock_type {
    return type_ as wined3d_stateblock_type;
}

unsafe extern "C" fn impl_from_IUnknown(iface: *mut IUnknown) -> *mut d3d_device {
    return (iface as *mut u8).sub(core::mem::offset_of!(d3d_device, IUnknown_inner)) as *mut d3d_device;
}

unsafe extern "system" fn d3d_device_inner_QueryInterface(iface: *mut IUnknown, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let device: *mut d3d_device = impl_from_IUnknown(iface);

    if riid.is_null() {
        *out = core::ptr::null_mut();
        return DDERR_INVALIDPARAMS;
    }

    if IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid) {
        IDirect3DDevice7_AddRef(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
        *out = core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface) as *mut core::ffi::c_void;
        return S_OK;
    }

    if (*device).version == 7 {
        if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DDevice7), riid) {
            IDirect3DDevice7_AddRef(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
            *out = core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface) as *mut core::ffi::c_void;
            return S_OK;
        }
    } else {
        if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DDevice3), riid) && (*device).version == 3 {
            IDirect3DDevice3_AddRef(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface));
            *out = core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface) as *mut core::ffi::c_void;
            return S_OK;
        }

        if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DDevice2), riid) && (*device).version >= 2 {
            IDirect3DDevice2_AddRef(core::ptr::addr_of_mut!((*device).IDirect3DDevice2_iface));
            *out = core::ptr::addr_of_mut!((*device).IDirect3DDevice2_iface) as *mut core::ffi::c_void;
            return S_OK;
        }

        if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DDevice), riid) {
            IDirect3DDevice_AddRef(core::ptr::addr_of_mut!((*device).IDirect3DDevice_iface));
            *out = core::ptr::addr_of_mut!((*device).IDirect3DDevice_iface) as *mut core::ffi::c_void;
            return S_OK;
        }
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
}

unsafe extern "system" fn d3d_device7_QueryInterface(iface: *mut IDirect3DDevice7, riid: *const GUID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    return IUnknown_QueryInterface((*device).outer_unknown, riid, out);
}
unsafe extern "system" fn d3d_device3_QueryInterface(
    mut iface: *mut IDirect3DDevice3,
    riid: *const IID,
    mut out: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IUnknown_QueryInterface((*device).outer_unknown, riid, out);
}

unsafe extern "system" fn d3d_device2_QueryInterface(iface: *mut IDirect3DDevice2, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IUnknown_QueryInterface((*device).outer_unknown, riid, out);
}
unsafe extern "system" fn d3d_device1_QueryInterface(iface: *mut IDirect3DDevice, riid: REFIID, out: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    return IUnknown_QueryInterface((*device).outer_unknown, riid, out);
}

unsafe extern "system" fn d3d_device_inner_AddRef(iface: *mut IUnknown) -> ULONG {
    let device: *mut d3d_device = impl_from_IUnknown(iface);
    let ref_0: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*device).ref_) as *mut core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("%p increasing refcount to %lu.\n", device, ref_0);
    return ref_0;
}
unsafe extern "system" fn d3d_device7_AddRef(mut iface: *mut IDirect3DDevice7) -> ULONG {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    return IUnknown_AddRef((*device).outer_unknown);
}

unsafe extern "system" fn d3d_device3_AddRef(iface: *mut IDirect3DDevice3) -> ULONG {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return IUnknown_AddRef((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device2_AddRef(mut iface: *mut IDirect3DDevice2) -> ULONG {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return IUnknown_AddRef((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device1_AddRef(iface: *mut IDirect3DDevice) -> ULONG {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    return IUnknown_AddRef((*device).outer_unknown);
}

unsafe extern "system" fn d3d_device_inner_Release(iface: *mut IUnknown) -> ULONG {
    let This: *mut d3d_device = impl_from_IUnknown(iface);
    let ref_0: ULONG = InterlockedDecrement(&raw mut (*This).ref_ as *mut ::core::ffi::c_long) as ULONG;
    let mut rt_iface: *mut IUnknown = core::ptr::null_mut();

    /* This method doesn't destroy the wined3d device, because it's still in
     * use for 2D rendering. IDirectDrawSurface7::Release will destroy the
     * wined3d device when the render target is released. */
    if ref_0 == 0 {
        static mut null_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
        let mut i: DWORD;
        let mut vp_entry: *mut list;
        let mut vp_entry2: *mut list;

        wined3d_mutex_lock();

        /* There is no need to unset any resources here, wined3d will take
         * care of that on uninit_3d(). */

        wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*This).index_buffer));
        wined3d_streaming_buffer_cleanup(core::ptr::addr_of_mut!((*This).vertex_buffer));

        wined3d_device_context_set_rendertarget_views(
            (*This).immediate_context,
            0,
            1,
            core::ptr::addr_of!(null_rtv),
            (FALSE) as BOOL,
        );

        /* Release the wined3d device. This won't destroy it. */
        if wined3d_device_decref((*This).wined3d_device) == 0 {
            ERR!(
                "The wined3d device (%p) was destroyed unexpectedly.\n",
                (*This).wined3d_device
            );
        }

        /* The texture handles should be unset by now, but there might be some bits
         * missing in our reference counting(needs test). Do a sanity check. */
        i = 0;
        while i < (*This).handle_table.entry_count {
            let entry: *mut ddraw_handle_entry =
                (*This).handle_table.entries.offset(i as isize) as *mut ddraw_handle_entry;

            match (*entry).type_ as ::core::ffi::c_uint {
                DDRAW_HANDLE_FREE => {}
                DDRAW_HANDLE_STATEBLOCK => {
                    /* No FIXME here because this might happen because of sloppy applications. */
                    WARN!(
                        "Leftover stateblock handle %#lx (%p), deleting.\n",
                        i.wrapping_add(1),
                        (*entry).object
                    );
                    IDirect3DDevice7_DeleteStateBlock(
                        core::ptr::addr_of_mut!((*This).IDirect3DDevice7_iface),
                        i.wrapping_add(1),
                    );
                }
                _ => {
                    FIXME!(
                        "Handle %#lx (%p) has unknown type %#x.\n",
                        i.wrapping_add(1),
                        (*entry).object,
                        (*entry).type_ as ::core::ffi::c_uint
                    );
                }
            }
            i = i.wrapping_add(1);
        }

        ddraw_handle_table_destroy(core::ptr::addr_of_mut!((*This).handle_table));

        vp_entry = (*This).viewport_list.next;
        vp_entry2 = (*vp_entry).next;
        while vp_entry != core::ptr::addr_of_mut!((*This).viewport_list) {
            let vp: *mut d3d_viewport = (vp_entry as *mut ::core::ffi::c_char)
                .offset(-(core::mem::offset_of!(d3d_viewport, entry) as isize))
                as *mut d3d_viewport;
            IDirect3DDevice3_DeleteViewport(
                core::ptr::addr_of_mut!((*This).IDirect3DDevice3_iface),
                core::ptr::addr_of_mut!((*vp).IDirect3DViewport3_iface),
            );
            vp_entry = vp_entry2;
            vp_entry2 = (*vp_entry).next;
        }

        wined3d_stateblock_decref((*This).state);
        if !(*This).recording.is_null() {
            wined3d_stateblock_decref((*This).recording);
        }

        /* Releasing the render target below may release the last reference to the ddraw object. Detach
         * the device from it before so it doesn't try to save / restore state on the teared down device. */
        if !(*This).ddraw.is_null() {
            if (*(*This).ddraw).device_last_applied_state == This {
                (*(*This).ddraw).device_last_applied_state = core::ptr::null_mut();
            }
            list_remove(core::ptr::addr_of_mut!((*This).ddraw_entry));
            (*This).ddraw = core::ptr::null_mut();
        }

        rt_iface = (*This).rt_iface;
        (*This).rt_iface = core::ptr::null_mut();
        (*This).target = core::ptr::null_mut();
        (*This).target_ds = core::ptr::null_mut();
        if (*This).version != 1 {
            IUnknown_Release(rt_iface);
        }

        /* Now free the structure */
        free(This as *mut ::core::ffi::c_void);
        wined3d_mutex_unlock();
    }

    TRACE!("Done\n");
    return ref_0;
}

unsafe extern "system" fn d3d_device7_Release(mut iface: *mut IDirect3DDevice7) -> ULONG {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    return IUnknown_Release((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device3_Release(iface: *mut IDirect3DDevice3) -> ULONG {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IUnknown_Release((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device2_Release(iface: *mut IDirect3DDevice2) -> ULONG {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IUnknown_Release((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device1_Release(iface: *mut IDirect3DDevice) -> ULONG {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    return IUnknown_Release((*device).outer_unknown);
}
unsafe extern "system" fn d3d_device1_Initialize(
    iface: *mut IDirect3DDevice,
    d3d: *mut IDirect3D,
    guid: *mut GUID,
    device_desc: *mut D3DDEVICEDESC,
) -> HRESULT {
    // FIXME("iface %p, d3d %p, guid %s, device_desc %p nop!\n", iface, d3d, debugstr_guid(guid), device_desc);
    D3D_OK
}
unsafe extern "C" fn d3d_device7_GetCaps(
    iface: *mut IDirect3DDevice7,
    device_desc: *mut D3DDEVICEDESC7,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    if device_desc.is_null() {
        WARN!("device_desc is NULL, returning DDERR_INVALIDPARAMS.\n");
        return DDERR_INVALIDPARAMS;
    }

    /* Call the same function used by IDirect3D, this saves code */
    return ddraw_get_d3dcaps((*device).ddraw, device_desc);
}
unsafe extern "system" fn d3d_device7_GetCaps_FPUSetup(
    iface: *mut IDirect3DDevice7,
    desc: *mut D3DDEVICEDESC7,
) -> HRESULT {
    d3d_device7_GetCaps(iface, desc)
}

unsafe extern "system" fn d3d_device7_GetCaps_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    desc: *mut D3DDEVICEDESC7,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetCaps(iface, desc);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn check_d3ddevicedesc_size(mut size: DWORD) -> BOOL {
    if size == core::mem::offset_of!(D3DDEVICEDESC, dwMinTextureWidth) as DWORD /* 172 */
        || size == core::mem::offset_of!(D3DDEVICEDESC, dwMaxTextureRepeat) as DWORD /* 204 */
        || size == core::mem::size_of::<D3DDEVICEDESC>() as DWORD /* 252 */
    {
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "system" fn d3d_device3_GetCaps(
    iface: *mut IDirect3DDevice3,
    HWDesc: *mut D3DDEVICEDESC,
    HelDesc: *mut D3DDEVICEDESC,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut desc7: D3DDEVICEDESC7 = core::mem::zeroed();
    let mut desc1: D3DDEVICEDESC = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!("iface %p, hw_desc %p, hel_desc %p.\n", iface, HWDesc, HelDesc);

    if HWDesc.is_null() {
        WARN!("HWDesc is NULL, returning DDERR_INVALIDPARAMS.\n");
        return DDERR_INVALIDPARAMS;
    }
    if check_d3ddevicedesc_size((*HWDesc).dwSize) == 0 {
        WARN!("HWDesc->dwSize is %lu, returning DDERR_INVALIDPARAMS.\n", (*HWDesc).dwSize);
        return DDERR_INVALIDPARAMS;
    }
    if HelDesc.is_null() {
        WARN!("HelDesc is NULL, returning DDERR_INVALIDPARAMS.\n");
        return DDERR_INVALIDPARAMS;
    }
    if check_d3ddevicedesc_size((*HelDesc).dwSize) == 0 {
        WARN!("HelDesc->dwSize is %lu, returning DDERR_INVALIDPARAMS.\n", (*HelDesc).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    hr = ddraw_get_d3dcaps((*device).ddraw, &raw mut desc7);
    if hr < 0 {
        return hr;
    }

    ddraw_d3dcaps1_from_7(&raw mut desc1, &raw mut desc7);
    DD_STRUCT_COPY_BYSIZE!(HWDesc, &raw mut desc1);
    DD_STRUCT_COPY_BYSIZE!(HelDesc, &raw mut desc1);
    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_GetCaps(
    mut iface: *mut IDirect3DDevice2,
    mut hw_desc: *mut D3DDEVICEDESC,
    mut hel_desc: *mut D3DDEVICEDESC,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, hw_desc %p, hel_desc %p.\n", iface, hw_desc, hel_desc);
    return d3d_device3_GetCaps(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        hw_desc,
        hel_desc,
    );
}
unsafe extern "system" fn d3d_device1_GetCaps(
    mut iface: *mut IDirect3DDevice,
    mut hw_desc: *mut D3DDEVICEDESC,
    mut hel_desc: *mut D3DDEVICEDESC,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    WINE_TRACE!("iface %p, hw_desc %p, hel_desc %p.\n", iface, hw_desc, hel_desc);
    return d3d_device3_GetCaps(&raw mut (*device).IDirect3DDevice3_iface, hw_desc, hel_desc);
}
unsafe extern "system" fn d3d_device2_SwapTextureHandles(
    iface: *mut IDirect3DDevice2,
    tex1: *mut IDirect3DTexture2,
    tex2: *mut IDirect3DTexture2,
) -> HRESULT {
    let surf1: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture2(tex1);
    let surf2: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture2(tex2);
    let mut h1: DWORD = 0;
    let mut h2: DWORD = 0;
    let mut h: DWORD = 0;
    let mut hr: HRESULT = 0;

    if (*surf1).Handle == 0 || (*surf2).Handle == 0 {
        return E_INVALIDARG;
    }

    hr = IDirect3DDevice2_GetRenderState(iface, D3DRENDERSTATE_TEXTUREHANDLE, core::ptr::addr_of_mut!(h));
    if hr < 0 {
        return hr;
    }

    wined3d_mutex_lock();

    h1 = (*surf1).Handle.wrapping_sub(1);
    h2 = (*surf2).Handle.wrapping_sub(1);
    (*global_handle_table.entries.offset(h1 as isize)).object = surf2 as *mut core::ffi::c_void;
    (*global_handle_table.entries.offset(h2 as isize)).object = surf1 as *mut core::ffi::c_void;
    (*surf2).Handle = h1.wrapping_add(1);
    (*surf1).Handle = h2.wrapping_add(1);

    wined3d_mutex_unlock();

    if h == (*surf1).Handle || h == (*surf2).Handle {
        hr = IDirect3DDevice2_SetRenderState(iface, D3DRENDERSTATE_TEXTUREHANDLE, h);
        if hr < 0 {
            return hr;
        }
    }
    return D3D_OK;
}
unsafe extern "system" fn d3d_device1_SwapTextureHandles(
    iface: *mut IDirect3DDevice,
    tex1: *mut IDirect3DTexture,
    tex2: *mut IDirect3DTexture,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let surf1: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture(tex1);
    let surf2: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture(tex2);
    let t1: *mut IDirect3DTexture2 = if !surf1.is_null() {
        &raw mut (*surf1).IDirect3DTexture2_iface
    } else {
        core::ptr::null_mut::<IDirect3DTexture2>()
    };
    let t2: *mut IDirect3DTexture2 = if !surf2.is_null() {
        &raw mut (*surf2).IDirect3DTexture2_iface
    } else {
        core::ptr::null_mut::<IDirect3DTexture2>()
    };
    WINE_TRACE!("iface %p, tex1 %p, tex2 %p.\n", iface, tex1, tex2);
    return d3d_device2_SwapTextureHandles(&raw mut (*device).IDirect3DDevice2_iface, t1, t2);
}
unsafe extern "system" fn d3d_device3_GetStats(
    mut iface: *mut IDirect3DDevice3,
    mut Stats: *mut D3DSTATS,
) -> HRESULT {
    WINE_FIXME!("iface %p, stats %p stub!\n", iface, Stats);
    if Stats.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    (*Stats).dwTrianglesDrawn = 0;
    (*Stats).dwLinesDrawn = 0;
    (*Stats).dwPointsDrawn = 0;
    (*Stats).dwSpansDrawn = 0;
    (*Stats).dwVerticesProcessed = 0;
    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_GetStats(iface: *mut IDirect3DDevice2, stats: *mut D3DSTATS) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, stats %p.\n", iface, stats);
    return d3d_device3_GetStats(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), stats);
}
unsafe extern "system" fn d3d_device1_GetStats(
    mut iface: *mut IDirect3DDevice,
    mut stats: *mut D3DSTATS,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    WINE_TRACE!("iface %p, stats %p.\n", iface, stats);
    return d3d_device3_GetStats(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), stats);
}

unsafe extern "system" fn d3d_device1_CreateExecuteBuffer(
    iface: *mut IDirect3DDevice,
    buffer_desc: *mut D3DEXECUTEBUFFERDESC,
    ExecuteBuffer: *mut *mut IDirect3DExecuteBuffer,
    outer_unknown: *mut IUnknown,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let mut object: *mut d3d_execute_buffer;
    let mut hr: HRESULT;

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    /* Allocate the new Execute Buffer */
    object = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<d3d_execute_buffer>() as size_t) as _,
    ) as *mut d3d_execute_buffer;
    if object.is_null() {
        ERR!("Failed to allocate execute buffer memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    hr = d3d_execute_buffer_init(object, device, buffer_desc);
    if hr < 0 {
        WARN!("Failed to initialize execute buffer, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *ExecuteBuffer = &raw mut (*object).IDirect3DExecuteBuffer_iface;

    return D3D_OK;
}

unsafe extern "system" fn d3d_device1_Execute(
    iface: *mut IDirect3DDevice,
    ExecuteBuffer: *mut IDirect3DExecuteBuffer,
    viewport: *mut IDirect3DViewport,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let buffer: *mut d3d_execute_buffer = unsafe_impl_from_IDirect3DExecuteBuffer(ExecuteBuffer);
    let viewport_impl: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport(viewport);
    let mut hr: HRESULT;

    if buffer.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    hr = IDirect3DDevice3_SetCurrentViewport(
        &raw mut (*device).IDirect3DDevice3_iface,
        &raw mut (*viewport_impl).IDirect3DViewport3_iface,
    );
    if hr < 0 {
        return hr;
    }

    /* Execute... */
    wined3d_mutex_lock();
    hr = d3d_execute_buffer_execute(buffer, device);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device3_AddViewport(iface: *mut IDirect3DDevice3, viewport: *mut IDirect3DViewport3) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport3(viewport);

    TRACE!("iface %p, viewport %p.\n", iface, viewport);

    /* Sanity check */
    if vp.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    IDirect3DViewport3_AddRef(viewport);
    list_add_head(core::ptr::addr_of_mut!((*device).viewport_list), core::ptr::addr_of_mut!((*vp).entry));
    /* Viewport must be usable for Clear() after AddViewport, so set active_device here. */
    (*vp).active_device = device;
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_AddViewport(
    iface: *mut IDirect3DDevice2,
    viewport: *mut IDirect3DViewport2,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport2(viewport);

    return d3d_device3_AddViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        core::ptr::addr_of_mut!((*vp).IDirect3DViewport3_iface),
    );
}

unsafe extern "system" fn d3d_device1_AddViewport(
    iface: *mut IDirect3DDevice,
    viewport: *mut IDirect3DViewport,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport(viewport);

    WINE_TRACE!("iface %p, viewport %p.\n", iface, viewport);

    return d3d_device3_AddViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        core::ptr::addr_of_mut!((*vp).IDirect3DViewport3_iface),
    );
}

unsafe extern "system" fn d3d_device3_DeleteViewport(iface: *mut IDirect3DDevice3, viewport: *mut IDirect3DViewport3) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport3(viewport);

    if vp.is_null() {
        WARN!("NULL viewport, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    if (*vp).active_device != device {
        WARN!("Viewport %p active device is %p.\n", vp, (*vp).active_device);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if (*device).current_viewport == vp {
        viewport_deactivate(vp);
        IDirect3DViewport3_Release(viewport);
        (*device).current_viewport = core::ptr::null_mut();
    }

    (*vp).active_device = core::ptr::null_mut();
    list_remove(core::ptr::addr_of_mut!((*vp).entry));

    IDirect3DViewport3_Release(viewport);

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_DeleteViewport(iface: *mut IDirect3DDevice2, viewport: *mut IDirect3DViewport2) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport2(viewport);

    return d3d_device3_DeleteViewport(
        &raw mut (*device).IDirect3DDevice3_iface,
        if !vp.is_null() {
            &raw mut (*vp).IDirect3DViewport3_iface
        } else {
            core::ptr::null_mut()
        },
    );
}

unsafe extern "system" fn d3d_device1_DeleteViewport(
    mut iface: *mut IDirect3DDevice,
    mut viewport: *mut IDirect3DViewport,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let mut vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport(viewport);
    WINE_TRACE!("iface %p, viewport %p.\n", iface, viewport);
    return d3d_device3_DeleteViewport(
        &raw mut (*device).IDirect3DDevice3_iface,
        if !vp.is_null() {
            &raw mut (*vp).IDirect3DViewport3_iface
        } else {
            ::core::ptr::null_mut::<IDirect3DViewport3>()
        },
    );
}

unsafe extern "system" fn d3d_device3_NextViewport(iface: *mut IDirect3DDevice3, Viewport3: *mut IDirect3DViewport3, lplpDirect3DViewport3: *mut *mut IDirect3DViewport3, flags: DWORD) -> HRESULT {
    let This: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport3(Viewport3);
    let mut next: *mut d3d_viewport;
    let mut entry: *mut list;

    if vp.is_null() {
        *lplpDirect3DViewport3 = core::ptr::null_mut();
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    match flags {
        D3DNEXT_NEXT => {
            entry = list_next(core::ptr::addr_of_mut!((*This).viewport_list), core::ptr::addr_of_mut!((*vp).entry));
        }
        D3DNEXT_HEAD => {
            entry = list_head(core::ptr::addr_of_mut!((*This).viewport_list));
        }
        D3DNEXT_TAIL => {
            entry = list_tail(core::ptr::addr_of_mut!((*This).viewport_list));
        }
        _ => {
            WARN!("Invalid flags %#lx.\n", flags);
            *lplpDirect3DViewport3 = core::ptr::null_mut();
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
    }

    if !entry.is_null() {
        next = LIST_ENTRY!(entry, d3d_viewport, entry);
        *lplpDirect3DViewport3 = core::ptr::addr_of_mut!((*next).IDirect3DViewport3_iface);
    } else {
        *lplpDirect3DViewport3 = core::ptr::null_mut();
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_device2_NextViewport(
    iface: *mut IDirect3DDevice2,
    viewport: *mut IDirect3DViewport2,
    next: *mut *mut IDirect3DViewport2,
    flags: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport2(viewport);
    let mut res: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut hr: HRESULT;
    WINE_TRACE!("iface %p, viewport %p, next %p, flags %#lx.\n", iface, viewport, next, flags);
    hr = d3d_device3_NextViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        core::ptr::addr_of_mut!((*vp).IDirect3DViewport3_iface),
        core::ptr::addr_of_mut!(res),
        flags,
    );
    *next = res as *mut IDirect3DViewport2;
    return hr;
}
unsafe extern "system" fn d3d_device1_NextViewport(
    iface: *mut IDirect3DDevice,
    viewport: *mut IDirect3DViewport,
    next: *mut *mut IDirect3DViewport,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport(viewport);
    let mut res: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, viewport %p, next %p, flags %#lx.\n",
        iface,
        viewport,
        next,
        flags
    );

    hr = d3d_device3_NextViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        core::ptr::addr_of_mut!((*vp).IDirect3DViewport3_iface),
        core::ptr::addr_of_mut!(res),
        flags,
    );
    *next = res as *mut IDirect3DViewport;
    return hr;
}
unsafe extern "system" fn d3d_device1_Pick(
    iface: *mut IDirect3DDevice,
    buffer: *mut IDirect3DExecuteBuffer,
    viewport: *mut IDirect3DViewport,
    flags: DWORD,
    rect: *mut D3DRECT,
) -> HRESULT {
    // FIXME("iface %p, buffer %p, viewport %p, flags %#lx, rect %s stub!\n", iface, buffer, viewport, flags, wine_dbgstr_rect((RECT *)rect));
    D3D_OK
}
unsafe extern "system" fn d3d_device1_GetPickRecords(
    iface: *mut IDirect3DDevice,
    count: *mut DWORD,
    records: *mut D3DPICKRECORD,
) -> HRESULT {
    // FIXME("iface %p, count %p, records %p stub!\n", iface, count, records);
    D3D_OK
}

unsafe extern "C" fn d3d_device7_EnumTextureFormats(
    iface: *mut IDirect3DDevice7,
    callback: LPD3DENUMPIXELFORMATSCALLBACK,
    context: *mut core::ffi::c_void,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut i: core::ffi::c_uint = 0;

    static FormatList: [wined3d_format_id; 13] = [
        /* 16 bit */
        WINED3DFMT_B5G5R5X1_UNORM,
        WINED3DFMT_B5G5R5A1_UNORM,
        WINED3DFMT_B4G4R4A4_UNORM,
        WINED3DFMT_B5G6R5_UNORM,
        /* 32 bit */
        WINED3DFMT_B8G8R8X8_UNORM,
        WINED3DFMT_B8G8R8A8_UNORM,
        /* 8 bit */
        WINED3DFMT_B2G3R3_UNORM,
        WINED3DFMT_P8_UINT,
        /* FOURCC codes */
        WINED3DFMT_DXT1,
        WINED3DFMT_DXT2,
        WINED3DFMT_DXT3,
        WINED3DFMT_DXT4,
        WINED3DFMT_DXT5,
    ];

    static BumpFormatList: [wined3d_format_id; 5] = [
        WINED3DFMT_R8G8_SNORM,
        WINED3DFMT_R5G5_SNORM_L6_UNORM,
        WINED3DFMT_R8G8_SNORM_L8X8_UNORM,
        WINED3DFMT_R10G11B11_SNORM,
        WINED3DFMT_R10G10B10_SNORM_A2_UNORM,
    ];

    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    memset(
        core::ptr::addr_of_mut!(mode) as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<wined3d_display_mode>(),
    );
    hr = wined3d_output_get_display_mode(
        (*(*device).ddraw).wined3d_output,
        core::ptr::addr_of_mut!(mode),
        core::ptr::null_mut(),
    );
    if hr < 0 {
        wined3d_mutex_unlock();
        WARN!("Failed to get output display mode, hr %#lx.\n", hr);
        return hr;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(FormatList) as usize {
        if wined3d_check_device_format(
            (*(*device).ddraw).wined3d,
            (*(*device).ddraw).wined3d_adapter,
            WINED3D_DEVICE_TYPE_HAL,
            mode.format_id,
            0,
            WINED3D_BIND_SHADER_RESOURCE,
            WINED3D_RTYPE_TEXTURE_2D,
            FormatList[i as usize],
        ) == D3D_OK
        {
            let mut pformat: DDPIXELFORMAT = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(pformat) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<DDPIXELFORMAT>(),
            );
            pformat.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
            ddrawformat_from_wined3dformat(core::ptr::addr_of_mut!(pformat), FormatList[i as usize]);

            hr = (callback.unwrap_unchecked())(core::ptr::addr_of_mut!(pformat), context);
            if hr != (DDENUMRET_OK as HRESULT) {
                TRACE!("Format enumeration cancelled by application\n");
                wined3d_mutex_unlock();
                return D3D_OK;
            }
        }
        i = i.wrapping_add(1);
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!(BumpFormatList) as usize {
        if wined3d_check_device_format(
            (*(*device).ddraw).wined3d,
            (*(*device).ddraw).wined3d_adapter,
            WINED3D_DEVICE_TYPE_HAL,
            mode.format_id,
            WINED3DUSAGE_QUERY_LEGACYBUMPMAP,
            WINED3D_BIND_SHADER_RESOURCE,
            WINED3D_RTYPE_TEXTURE_2D,
            BumpFormatList[i as usize],
        ) == D3D_OK
        {
            let mut pformat: DDPIXELFORMAT = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(pformat) as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<DDPIXELFORMAT>(),
            );
            pformat.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
            ddrawformat_from_wined3dformat(core::ptr::addr_of_mut!(pformat), BumpFormatList[i as usize]);

            hr = (callback.unwrap_unchecked())(core::ptr::addr_of_mut!(pformat), context);
            if hr != (DDENUMRET_OK as HRESULT) {
                TRACE!("Format enumeration cancelled by application\n");
                wined3d_mutex_unlock();
                return D3D_OK;
            }
        }
        i = i.wrapping_add(1);
    }

    TRACE!("End of enumeration\n");
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_EnumTextureFormats_FPUSetup(
    iface: *mut IDirect3DDevice7,
    callback: LPD3DENUMPIXELFORMATSCALLBACK,
    context: *mut core::ffi::c_void,
) -> HRESULT {
    d3d_device7_EnumTextureFormats(iface, callback, context)
}
unsafe extern "system" fn d3d_device7_EnumTextureFormats_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut callback: LPD3DENUMPIXELFORMATSCALLBACK,
    mut context: *mut core::ffi::c_void,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut old_fpucw: WORD = 0;
    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_EnumTextureFormats(iface, callback, context);
    set_fpu_control_word(old_fpucw);
    return hr;
}
unsafe extern "system" fn d3d_device3_EnumTextureFormats(
    mut iface: *mut IDirect3DDevice3,
    mut callback: LPD3DENUMPIXELFORMATSCALLBACK,
    mut context: *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    WINE_TRACE!("iface %p, callback %p, context %p.\n", iface, callback, context);
    return IDirect3DDevice7_EnumTextureFormats(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        callback,
        context,
    );
}

unsafe extern "system" fn d3d_device2_EnumTextureFormats(
    iface: *mut IDirect3DDevice2,
    callback: LPD3DENUMTEXTUREFORMATSCALLBACK,
    context: *mut ::core::ffi::c_void,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut i: ::core::ffi::c_uint = 0;

    static FormatList: [wined3d_format_id; 8] = [
        /* 16 bit */
        WINED3DFMT_B5G5R5X1_UNORM,
        WINED3DFMT_B5G5R5A1_UNORM,
        WINED3DFMT_B4G4R4A4_UNORM,
        WINED3DFMT_B5G6R5_UNORM,
        /* 32 bit */
        WINED3DFMT_B8G8R8X8_UNORM,
        WINED3DFMT_B8G8R8A8_UNORM,
        /* 8 bit */
        WINED3DFMT_B2G3R3_UNORM,
        WINED3DFMT_P8_UINT,
        /* FOURCC codes - Not in this version*/
    ];

    if callback.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    memset(
        core::ptr::addr_of_mut!(mode) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        core::mem::size_of::<wined3d_display_mode>() as usize,
    );
    hr = wined3d_output_get_display_mode(
        (*(*device).ddraw).wined3d_output,
        core::ptr::addr_of_mut!(mode),
        core::ptr::null_mut(),
    );
    if hr < 0 {
        wined3d_mutex_unlock();
        WARN!("Failed to get output display mode, hr %#lx.\n", hr);
        return hr;
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(FormatList) as usize {
        if wined3d_check_device_format(
            (*(*device).ddraw).wined3d,
            (*(*device).ddraw).wined3d_adapter,
            WINED3D_DEVICE_TYPE_HAL,
            mode.format_id,
            0,
            WINED3D_BIND_SHADER_RESOURCE,
            WINED3D_RTYPE_TEXTURE_2D,
            FormatList[i as usize],
        ) == D3D_OK
        {
            let mut sdesc: DDSURFACEDESC = core::mem::zeroed();

            memset(
                core::ptr::addr_of_mut!(sdesc) as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                core::mem::size_of::<DDSURFACEDESC>() as usize,
            );
            sdesc.dwSize = core::mem::size_of::<DDSURFACEDESC>() as DWORD;
            sdesc.dwFlags = (DDSD_PIXELFORMAT | DDSD_CAPS) as DWORD;
            sdesc.ddsCaps.dwCaps = DDSCAPS_TEXTURE as DWORD;
            sdesc.ddpfPixelFormat.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
            ddrawformat_from_wined3dformat(
                core::ptr::addr_of_mut!(sdesc.ddpfPixelFormat),
                FormatList[i as usize],
            );

            hr = (callback.unwrap_unchecked())(core::ptr::addr_of_mut!(sdesc), context);
            if hr != (DDENUMRET_OK as HRESULT) {
                wined3d_mutex_unlock();
                return D3D_OK;
            }
        }
        i = i.wrapping_add(1);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device1_EnumTextureFormats(
    iface: *mut IDirect3DDevice,
    callback: LPD3DENUMTEXTUREFORMATSCALLBACK,
    context: *mut core::ffi::c_void,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    WINE_TRACE!("iface %p, callback %p, context %p.\n", iface, callback, context);
    return d3d_device2_EnumTextureFormats(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice2_iface),
        callback,
        context,
    );
}

unsafe extern "system" fn d3d_device1_CreateMatrix(iface: *mut IDirect3DDevice, D3DMatHandle: *mut D3DMATRIXHANDLE) -> HRESULT {
    let mut matrix: *mut D3DMATRIX = core::ptr::null_mut();
    let mut h: DWORD = 0;

    if D3DMatHandle.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    matrix = calloc(1, (core::mem::size_of::<D3DMATRIX>() as usize) as _) as *mut D3DMATRIX;
    if matrix.is_null() {
        ERR!("Out of memory when allocating a D3DMATRIX\n");
        return DDERR_OUTOFMEMORY;
    }

    wined3d_mutex_lock();

    h = ddraw_allocate_handle(core::ptr::null_mut(), matrix as *mut core::ffi::c_void, DDRAW_HANDLE_MATRIX);
    if h == (DDRAW_INVALID_HANDLE) as u32 {
        ERR!("Failed to allocate a matrix handle.\n");
        free(matrix as *mut core::ffi::c_void);
        wined3d_mutex_unlock();
        return DDERR_OUTOFMEMORY;
    }

    *D3DMatHandle = h.wrapping_add(1 as DWORD) as D3DMATRIXHANDLE;

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_device1_SetMatrix(iface: *mut IDirect3DDevice, matrix_handle: D3DMATRIXHANDLE, matrix: *mut D3DMATRIX) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice(iface);
    let mut m: *mut D3DMATRIX;

    if matrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    m = ddraw_get_object(core::ptr::null_mut(), (matrix_handle as DWORD).wrapping_sub(1 as DWORD), DDRAW_HANDLE_MATRIX) as *mut D3DMATRIX;
    if m.is_null() {
        WARN!("Invalid matrix handle.\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        dump_D3DMATRIX(matrix);
    }

    *m = *matrix;

    if matrix_handle == (*device).world {
        wined3d_stateblock_set_transform((*device).state, WINED3D_TS_WORLD_MATRIX!(0), matrix as *mut wined3d_matrix);
    }

    if matrix_handle == (*device).view {
        wined3d_stateblock_set_transform((*device).state, WINED3D_TS_VIEW, matrix as *mut wined3d_matrix);
    }

    if matrix_handle == (*device).proj {
        wined3d_stateblock_set_transform((*device).state, WINED3D_TS_PROJECTION, matrix as *mut wined3d_matrix);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_device1_GetMatrix(
    iface: *mut IDirect3DDevice,
    D3DMatHandle: D3DMATRIXHANDLE,
    D3DMatrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut m: *mut D3DMATRIX;

    if D3DMatrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    TRACE!(
        "iface %p, matrix_handle %#lx, matrix %p.\n",
        iface,
        D3DMatHandle,
        D3DMatrix
    );

    wined3d_mutex_lock();

    m = ddraw_get_object(
        core::ptr::null_mut::<ddraw_handle_table>(),
        (D3DMatHandle as DWORD).wrapping_sub(1 as DWORD),
        DDRAW_HANDLE_MATRIX,
    ) as *mut D3DMATRIX;
    if m.is_null() {
        WARN!("Invalid matrix handle.\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    *D3DMatrix = *m;

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device1_DeleteMatrix(iface: *mut IDirect3DDevice, D3DMatHandle: D3DMATRIXHANDLE) -> HRESULT {
    let mut m: *mut D3DMATRIX;

    wined3d_mutex_lock();

    m = ddraw_free_handle(
        core::ptr::null_mut(),
        (D3DMatHandle as DWORD).wrapping_sub(1 as DWORD),
        DDRAW_HANDLE_MATRIX,
    ) as *mut D3DMATRIX;
    if m.is_null() {
        WARN!("Invalid matrix handle.\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_unlock();

    free(m as *mut core::ffi::c_void);

    return D3D_OK;
}

unsafe extern "C" fn d3d_device7_BeginScene(iface: *mut IDirect3DDevice7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    hr = wined3d_device_begin_scene((*device).wined3d_device);
    wined3d_mutex_unlock();

    if hr == WINED3D_OK {
        return D3D_OK;
    } else {
        return D3DERR_SCENE_IN_SCENE;
    }
}
unsafe extern "system" fn d3d_device7_BeginScene_FPUSetup(
    iface: *mut IDirect3DDevice7,
) -> HRESULT {
    d3d_device7_BeginScene(iface)
}
unsafe extern "system" fn d3d_device7_BeginScene_FPUPreserve(iface: *mut IDirect3DDevice7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_BeginScene(iface);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_BeginScene(iface: *mut IDirect3DDevice3) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    // TRACE("iface %p.\n", iface);

    return IDirect3DDevice7_BeginScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}
unsafe extern "system" fn d3d_device2_BeginScene(iface: *mut IDirect3DDevice2) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice7_BeginScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}

unsafe extern "system" fn d3d_device1_BeginScene(iface: *mut IDirect3DDevice) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    return IDirect3DDevice7_BeginScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}

unsafe extern "C" fn d3d_device7_EndScene(mut iface: *mut IDirect3DDevice7) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p.\n", iface);
    wined3d_mutex_lock();
    hr = wined3d_device_end_scene((*device).wined3d_device);
    wined3d_mutex_unlock();

    if hr == WINED3D_OK {
        return D3D_OK;
    } else {
        return D3DERR_SCENE_NOT_IN_SCENE;
    };
}
#[no_mangle]
pub unsafe extern "system" fn d3d_device7_EndScene_FPUSetup(
    iface: *mut IDirect3DDevice7,
) -> HRESULT {
    d3d_device7_EndScene(iface)
}
#[no_mangle]
pub unsafe extern "system" fn d3d_device7_EndScene_FPUPreserve(iface: *mut IDirect3DDevice7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_EndScene(iface);
    set_fpu_control_word(old_fpucw);

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn d3d_device3_EndScene(mut iface: *mut IDirect3DDevice3) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return IDirect3DDevice7_EndScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}
#[no_mangle]
pub unsafe extern "system" fn d3d_device2_EndScene(mut iface: *mut IDirect3DDevice2) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return IDirect3DDevice7_EndScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}
#[no_mangle]
pub unsafe extern "system" fn d3d_device1_EndScene(iface: *mut IDirect3DDevice) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    return IDirect3DDevice7_EndScene(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface));
}
unsafe extern "system" fn d3d_device7_GetDirect3D(iface: *mut IDirect3DDevice7, d3d: *mut *mut IDirect3D7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    if d3d.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    *d3d = core::ptr::addr_of_mut!((*(*device).ddraw).IDirect3D7_iface);
    IDirect3D7_AddRef(*d3d);

    return D3D_OK;
}

unsafe extern "system" fn d3d_device3_GetDirect3D(iface: *mut IDirect3DDevice3, d3d: *mut *mut IDirect3D3) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    if d3d.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    *d3d = core::ptr::addr_of_mut!((*(*device).ddraw).IDirect3D3_iface);
    IDirect3D3_AddRef(*d3d);

    return D3D_OK;
}

unsafe extern "system" fn d3d_device2_GetDirect3D(iface: *mut IDirect3DDevice2, d3d: *mut *mut IDirect3D2) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, d3d %p.\n", iface, d3d);
    if d3d.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    *d3d = &raw mut (*(*device).ddraw).IDirect3D2_iface;
    IDirect3D2_AddRef(*d3d);
    WINE_TRACE!("Returning interface %p.\n", *d3d);
    return D3D_OK;
}

unsafe extern "system" fn d3d_device1_GetDirect3D(iface: *mut IDirect3DDevice, d3d: *mut *mut IDirect3D) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice(iface);

    if d3d.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    *d3d = core::ptr::addr_of_mut!((*(*device).ddraw).IDirect3D_iface);
    IDirect3D_AddRef(*d3d);

    return D3D_OK;
}

unsafe extern "system" fn d3d_device3_SetCurrentViewport(iface: *mut IDirect3DDevice3, viewport: *mut IDirect3DViewport3) -> HRESULT {
    let mut vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport3(viewport);
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    if vp.is_null() {
        WARN!("Direct3DViewport3 is NULL.\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    /* Do nothing if the specified viewport is the same as the current one */
    if (*device).current_viewport == vp {
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    if (*vp).active_device != device {
        WARN!("Viewport %p, active device %p.\n", vp, (*vp).active_device);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    IDirect3DViewport3_AddRef(viewport);
    if !(*device).current_viewport.is_null() {
        viewport_deactivate((*device).current_viewport);
        IDirect3DViewport3_Release(core::ptr::addr_of_mut!((*(*device).current_viewport).IDirect3DViewport3_iface));
    }
    (*device).current_viewport = vp;
    viewport_activate((*device).current_viewport, (FALSE) as BOOL);

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_SetCurrentViewport(iface: *mut IDirect3DDevice2, viewport: *mut IDirect3DViewport2) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let vp: *mut d3d_viewport = unsafe_impl_from_IDirect3DViewport2(viewport);

    return d3d_device3_SetCurrentViewport(
        &raw mut (*device).IDirect3DDevice3_iface,
        if !vp.is_null() {
            &raw mut (*vp).IDirect3DViewport3_iface
        } else {
            core::ptr::null_mut::<IDirect3DViewport3>()
        },
    );
}

unsafe extern "system" fn d3d_device3_GetCurrentViewport(iface: *mut IDirect3DDevice3, viewport: *mut *mut IDirect3DViewport3) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    TRACE!("iface %p, viewport %p.\n", iface, viewport);

    wined3d_mutex_lock();
    if (*device).current_viewport.is_null() {
        wined3d_mutex_unlock();
        WARN!("No current viewport, returning D3DERR_NOCURRENTVIEWPORT\n");
        return D3DERR_NOCURRENTVIEWPORT;
    }

    *viewport = core::ptr::addr_of_mut!((*(*device).current_viewport).IDirect3DViewport3_iface);
    IDirect3DViewport3_AddRef(*viewport);

    TRACE!("Returning interface %p.\n", *viewport);
    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "system" fn d3d_device2_GetCurrentViewport(
    iface: *mut IDirect3DDevice2,
    viewport: *mut *mut IDirect3DViewport2,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, viewport %p.\n", iface, viewport);
    return d3d_device3_GetCurrentViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        viewport as *mut *mut IDirect3DViewport3,
    );
}

unsafe extern "C" fn validate_surface_palette(surface: *mut ddraw_surface) -> BOOL {
    return (format_is_paletteindexed(
        core::ptr::addr_of_mut!((*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat),
    ) == 0
        || !(*surface).palette.is_null()) as i32;
}

unsafe extern "C" fn d3d_device_set_render_target(
    device: *mut d3d_device,
    target: *mut ddraw_surface,
    rt_iface: *mut IUnknown,
) -> HRESULT {
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if (*device).rt_iface == rt_iface {
        return D3D_OK;
    }
    if target.is_null() {
        WARN!("Trying to set render target to NULL.\n");
        return DDERR_INVALIDPARAMS;
    }

    rtv = ddraw_surface_get_rendertarget_view(target);
    hr = wined3d_device_context_set_rendertarget_views(
        (*device).immediate_context,
        0,
        1,
        core::ptr::addr_of_mut!(rtv),
        (FALSE) as BOOL,
    );
    if hr < 0 {
        return hr;
    }

    IUnknown_AddRef(rt_iface);
    IUnknown_Release((*device).rt_iface);
    (*device).rt_iface = rt_iface;
    (*device).target = target;
    d3d_device_update_depth_stencil(device);

    return D3D_OK;
}

unsafe extern "C" fn d3d_device7_SetRenderTarget(
    iface: *mut IDirect3DDevice7,
    target: *mut IDirectDrawSurface7,
    flags: DWORD,
) -> HRESULT {
    let target_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(target);
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();

    if validate_surface_palette(target_impl) == 0 {
        WARN!("Surface %p has an indexed pixel format, but no palette.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if (*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_3DDEVICE == 0 {
        WARN!("Surface %p is not a render target.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if (*device).hardware_device != 0
        && (*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY == 0
    {
        WARN!("Surface %p is not in video memory.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if (*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER != 0 {
        WARN!("Surface %p is a depth buffer.\n", target_impl);
        IDirectDrawSurface7_AddRef(target);
        IUnknown_Release((*device).rt_iface);
        (*device).rt_iface = target as *mut IUnknown;
        (*device).target = core::ptr::null_mut::<ddraw_surface>();
        (*device).target_ds = core::ptr::null_mut::<ddraw_surface>();
        wined3d_mutex_unlock();
        return DDERR_INVALIDPIXELFORMAT;
    }

    hr = d3d_device_set_render_target(device, target_impl, target as *mut IUnknown);
    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d_device7_SetRenderTarget_FPUSetup(
    iface: *mut IDirect3DDevice7,
    NewTarget: *mut IDirectDrawSurface7,
    flags: DWORD,
) -> HRESULT {
    d3d_device7_SetRenderTarget(iface, NewTarget, flags)
}
unsafe extern "system" fn d3d_device7_SetRenderTarget_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut NewTarget: *mut IDirectDrawSurface7,
    mut flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetRenderTarget(iface, NewTarget, flags);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_SetRenderTarget(
    iface: *mut IDirect3DDevice3,
    target: *mut IDirectDrawSurface4,
    flags: DWORD,
) -> HRESULT {
    let target_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(target);
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();

    if !(validate_surface_palette(target_impl) != 0) {
        WARN!("Surface %p has an indexed pixel format, but no palette.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if !((*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_3DDEVICE as DWORD != 0) {
        WARN!("Surface %p is not a render target.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if (*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER as DWORD != 0 {
        WARN!("Surface %p is a depth buffer.\n", target_impl);
        IDirectDrawSurface4_AddRef(target);
        IUnknown_Release((*device).rt_iface);
        (*device).rt_iface = target as *mut IUnknown;
        (*device).target = core::ptr::null_mut();
        (*device).target_ds = core::ptr::null_mut();
        wined3d_mutex_unlock();
        return DDERR_INVALIDPIXELFORMAT;
    }

    if (*device).hardware_device != 0
        && !((*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY as DWORD != 0)
    {
        WARN!("Surface %p is not in video memory.\n", target_impl);
        IDirectDrawSurface4_AddRef(target);
        IUnknown_Release((*device).rt_iface);
        (*device).rt_iface = target as *mut IUnknown;
        (*device).target = core::ptr::null_mut();
        (*device).target_ds = core::ptr::null_mut();
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    hr = d3d_device_set_render_target(device, target_impl, target as *mut IUnknown);
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d_device2_SetRenderTarget(
    iface: *mut IDirect3DDevice2,
    target: *mut IDirectDrawSurface,
    flags: DWORD,
) -> HRESULT {
    let target_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(target);
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();

    if !(validate_surface_palette(target_impl) != 0) {
        WARN!("Surface %p has an indexed pixel format, but no palette.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if !(((*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_3DDEVICE) != 0) {
        WARN!("Surface %p is not a render target.\n", target_impl);
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }

    if ((*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) != 0 {
        WARN!("Surface %p is a depth buffer.\n", target_impl);
        IUnknown_Release((*device).rt_iface);
        (*device).rt_iface = target as *mut IUnknown;
        (*device).target = core::ptr::null_mut::<ddraw_surface>();
        (*device).target_ds = core::ptr::null_mut::<ddraw_surface>();
        wined3d_mutex_unlock();
        return DDERR_INVALIDPIXELFORMAT;
    }

    if (*device).hardware_device != 0
        && !(((*target_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY) != 0)
    {
        WARN!("Surface %p is not in video memory.\n", target_impl);
        IDirectDrawSurface_AddRef(target);
        IUnknown_Release((*device).rt_iface);
        (*device).rt_iface = target as *mut IUnknown;
        (*device).target = core::ptr::null_mut::<ddraw_surface>();
        (*device).target_ds = core::ptr::null_mut::<ddraw_surface>();
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    hr = d3d_device_set_render_target(device, target_impl, target as *mut IUnknown);
    wined3d_mutex_unlock();
    return hr;
}

unsafe extern "system" fn d3d_device7_GetRenderTarget(iface: *mut IDirect3DDevice7, RenderTarget: *mut *mut IDirectDrawSurface7) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    if RenderTarget.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    hr = IUnknown_QueryInterface((*device).rt_iface, &raw const IID_IDirectDrawSurface7, RenderTarget as *mut *mut core::ffi::c_void);
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d_device3_GetRenderTarget(
    iface: *mut IDirect3DDevice3,
    RenderTarget: *mut *mut IDirectDrawSurface4,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut RenderTarget7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut RenderTargetImpl: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT;

    if RenderTarget.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    hr = d3d_device7_GetRenderTarget(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        core::ptr::addr_of_mut!(RenderTarget7),
    );
    if hr != D3D_OK {
        return hr;
    }
    RenderTargetImpl = impl_from_IDirectDrawSurface7(RenderTarget7);
    *RenderTarget = core::ptr::addr_of_mut!((*RenderTargetImpl).IDirectDrawSurface4_iface);
    IDirectDrawSurface4_AddRef(*RenderTarget);
    IDirectDrawSurface7_Release(RenderTarget7);
    return D3D_OK;
}

unsafe extern "system" fn d3d_device2_GetRenderTarget(iface: *mut IDirect3DDevice2, RenderTarget: *mut *mut IDirectDrawSurface) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut RenderTarget7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut RenderTargetImpl: *mut ddraw_surface = core::ptr::null_mut();
    let mut hr: HRESULT;

    if RenderTarget.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    hr = d3d_device7_GetRenderTarget(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        core::ptr::addr_of_mut!(RenderTarget7),
    );
    if hr != D3D_OK {
        return hr;
    }
    RenderTargetImpl = impl_from_IDirectDrawSurface7(RenderTarget7);
    *RenderTarget = core::ptr::addr_of_mut!((*RenderTargetImpl).IDirectDrawSurface_iface);
    IDirectDrawSurface_AddRef(*RenderTarget);
    IDirectDrawSurface7_Release(RenderTarget7);
    return D3D_OK;
}

unsafe extern "system" fn d3d_device3_Begin(iface: *mut IDirect3DDevice3, primitive_type: D3DPRIMITIVETYPE, fvf: DWORD, flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    TRACE!("iface %p, primitive_type %#x, fvf %#lx, flags %#lx.\n", iface, primitive_type, fvf, flags);
    wined3d_mutex_lock();
    (*device).primitive_type = primitive_type;
    (*device).vertex_type = fvf;
    (*device).render_flags = flags;
    (*device).vertex_size = get_flexible_vertex_size((*device).vertex_type);
    (*device).nb_vertices = 0;
    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "system" fn d3d_device2_Begin(iface: *mut IDirect3DDevice2, primitive_type: D3DPRIMITIVETYPE, vertex_type: D3DVERTEXTYPE, flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut fvf: DWORD = 0;

    match vertex_type as u32 {
        D3DVT_VERTEX => {
            fvf = D3DFVF_VERTEX as DWORD;
        }
        D3DVT_LVERTEX => {
            fvf = D3DFVF_LVERTEX as DWORD;
        }
        D3DVT_TLVERTEX => {
            fvf = D3DFVF_TLVERTEX as DWORD;
        }
        _ => {
            ERR!("Unexpected vertex type %#x.\n", vertex_type);
            return DDERR_INVALIDPARAMS;
        }
    }

    return d3d_device3_Begin(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        primitive_type,
        fvf,
        flags,
    );
}
unsafe extern "system" fn d3d_device3_BeginIndexed(
    iface: *mut IDirect3DDevice3,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    // FIXME("iface %p, primitive_type %#x, fvf %#lx, vertices %p, vertex_count %lu, flags %#lx stub!\n", iface, primitive_type, fvf, vertices, vertex_count, flags);
    D3D_OK
}

unsafe extern "system" fn d3d_device2_BeginIndexed(
    iface: *mut IDirect3DDevice2,
    primitive_type: D3DPRIMITIVETYPE,
    vertex_type: D3DVERTEXTYPE,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut fvf: DWORD = 0;

    match vertex_type {
        D3DVT_VERTEX => {
            fvf = D3DFVF_VERTEX;
        }
        D3DVT_LVERTEX => {
            fvf = D3DFVF_LVERTEX;
        }
        D3DVT_TLVERTEX => {
            fvf = D3DFVF_TLVERTEX;
        }
        _ => {
            ERR!("Unexpected vertex type %#x.\n", vertex_type);
            return DDERR_INVALIDPARAMS; /* Should never happen */
        }
    }

    return d3d_device3_BeginIndexed(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        flags,
    );
}

unsafe extern "system" fn d3d_device3_Vertex(iface: *mut IDirect3DDevice3, vertex: *mut core::ffi::c_void) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    if vertex.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if (*device)
        .nb_vertices
        .wrapping_add(1)
        .wrapping_mul((*device).vertex_size)
        > (*device).buffer_size
    {
        let mut old_buffer: *mut BYTE = core::ptr::null_mut();

        (*device).buffer_size = if (*device).buffer_size != 0 {
            (*device).buffer_size.wrapping_mul(2)
        } else {
            (*device).vertex_size.wrapping_mul(3)
        };
        old_buffer = (*device).sysmem_vertex_buffer;
        (*device).sysmem_vertex_buffer =
            malloc((*device).buffer_size as usize) as *mut BYTE;
        if !old_buffer.is_null() {
            memcpy(
                (*device).sysmem_vertex_buffer as *mut core::ffi::c_void,
                old_buffer as *const core::ffi::c_void,
                (*device).nb_vertices.wrapping_mul((*device).vertex_size) as usize,
            );
            free(old_buffer as *mut core::ffi::c_void);
        }
    }

    let fresh17 = (*device).nb_vertices;
    (*device).nb_vertices = (*device).nb_vertices.wrapping_add(1);
    memcpy(
        (*device)
            .sysmem_vertex_buffer
            .add(fresh17.wrapping_mul((*device).vertex_size) as usize)
            as *mut core::ffi::c_void,
        vertex,
        (*device).vertex_size as usize,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_Vertex(mut iface: *mut IDirect3DDevice2, mut vertex: *mut core::ffi::c_void) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, vertex %p.\n", iface, vertex);
    return d3d_device3_Vertex(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), vertex);
}
unsafe extern "system" fn d3d_device3_Index(
    iface: *mut IDirect3DDevice3,
    index: WORD,
) -> HRESULT {
    // FIXME("iface %p, index %#x stub!\n", iface, index);
    D3D_OK
}

unsafe extern "system" fn d3d_device2_Index(iface: *mut IDirect3DDevice2, index: WORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    TRACE!("iface %p, index %#x.\n", iface, index as i32);
    return d3d_device3_Index(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), index);
}

unsafe extern "C" fn d3d_device7_GetRenderState(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut device_state: *const wined3d_stateblock_state = core::ptr::null();
    let mut hr: HRESULT = D3D_OK;

    if value.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    device_state = (*device).stateblock_state;
    match state {
        D3DRENDERSTATE_TEXTUREMAG => {
            let tex_mag: wined3d_texture_filter_type = (*device_state).sampler_states
                [0 as usize][WINED3D_SAMP_MAG_FILTER as usize]
                as wined3d_texture_filter_type;

            match tex_mag {
                WINED3D_TEXF_POINT => {
                    *value = D3DFILTER_NEAREST as DWORD;
                }
                WINED3D_TEXF_LINEAR => {
                    *value = D3DFILTER_LINEAR as DWORD;
                }
                _ => {
                    ERR!("Unhandled texture mag %d !\n", tex_mag as ::core::ffi::c_uint);
                    *value = 0 as DWORD;
                }
            }
        }

        D3DRENDERSTATE_TEXTUREMIN => {
            let mut tex_min: wined3d_texture_filter_type;
            let mut tex_mip: wined3d_texture_filter_type;

            tex_min = (*device_state).sampler_states[0 as usize]
                [WINED3D_SAMP_MIN_FILTER as usize] as wined3d_texture_filter_type;
            tex_mip = (*device_state).sampler_states[0 as usize]
                [WINED3D_SAMP_MIP_FILTER as usize] as wined3d_texture_filter_type;
            match tex_min {
                WINED3D_TEXF_POINT => {
                    match tex_mip {
                        WINED3D_TEXF_NONE => {
                            *value = D3DFILTER_NEAREST as DWORD;
                        }
                        WINED3D_TEXF_POINT => {
                            *value = D3DFILTER_MIPNEAREST as DWORD;
                        }
                        WINED3D_TEXF_LINEAR => {
                            *value = D3DFILTER_LINEARMIPNEAREST as DWORD;
                        }
                        _ => {
                            ERR!("Unhandled mip filter %#x.\n", tex_mip as ::core::ffi::c_uint);
                            *value = D3DFILTER_NEAREST as DWORD;
                        }
                    }
                }
                WINED3D_TEXF_LINEAR => {
                    match tex_mip {
                        WINED3D_TEXF_NONE => {
                            *value = D3DFILTER_LINEAR as DWORD;
                        }
                        WINED3D_TEXF_POINT => {
                            *value = D3DFILTER_MIPLINEAR as DWORD;
                        }
                        WINED3D_TEXF_LINEAR => {
                            *value = D3DFILTER_LINEARMIPLINEAR as DWORD;
                        }
                        _ => {
                            ERR!("Unhandled mip filter %#x.\n", tex_mip as ::core::ffi::c_uint);
                            *value = D3DFILTER_LINEAR as DWORD;
                        }
                    }
                }
                _ => {
                    ERR!("Unhandled texture min filter %#x.\n", tex_min as ::core::ffi::c_uint);
                    *value = D3DFILTER_NEAREST as DWORD;
                }
            }
        }

        D3DRENDERSTATE_TEXTUREADDRESS | D3DRENDERSTATE_TEXTUREADDRESSU => {
            *value = (*device_state).sampler_states[0 as usize]
                [WINED3D_SAMP_ADDRESS_U as usize] as DWORD;
        }
        D3DRENDERSTATE_TEXTUREADDRESSV => {
            *value = (*device_state).sampler_states[0 as usize]
                [WINED3D_SAMP_ADDRESS_V as usize] as DWORD;
        }

        D3DRENDERSTATE_BORDERCOLOR => {
            FIXME!("Unhandled render state D3DRENDERSTATE_BORDERCOLOR.\n");
            hr = E_NOTIMPL;
        }

        D3DRENDERSTATE_TEXTUREHANDLE | D3DRENDERSTATE_TEXTUREMAPBLEND => {
            WARN!("Render state %#x is invalid in d3d7.\n", state as ::core::ffi::c_uint);
            hr = DDERR_INVALIDPARAMS;
        }

        _ => {
            if (state as ::core::ffi::c_uint) >= (D3DRENDERSTATE_STIPPLEPATTERN00 as ::core::ffi::c_uint)
                && (state as ::core::ffi::c_uint) <= (D3DRENDERSTATE_STIPPLEPATTERN31 as ::core::ffi::c_uint)
            {
                FIXME!("Unhandled stipple pattern render state (%#x).\n", state as ::core::ffi::c_uint);
                hr = E_NOTIMPL;
            } else {
                *value = (*device_state).rs[wined3d_render_state_from_ddraw(state) as usize] as DWORD;
            }
        }
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_GetRenderState_FPUSetup(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    d3d_device7_GetRenderState(iface, state, value)
}

unsafe extern "system" fn d3d_device7_GetRenderState_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetRenderState(iface, state, value);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_GetRenderState(iface: *mut IDirect3DDevice3, state: D3DRENDERSTATETYPE, value: *mut DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    match state as u32 {
        D3DRENDERSTATE_TEXTUREHANDLE => {
            /* This state is wrapped to SetTexture in SetRenderState, so
             * it has to be wrapped to GetTexture here. */
            let mut tex: *mut wined3d_texture = core::ptr::null_mut();
            *value = 0;

            wined3d_mutex_lock();
            tex = (*(*device).stateblock_state).textures[0 as usize] as *mut wined3d_texture;
            if !tex.is_null() {
                /* The parent of the texture is the IDirectDrawSurface7
                 * interface of the ddraw surface. */
                let parent: *mut ddraw_texture = wined3d_texture_get_parent(tex) as *mut ddraw_texture;
                if !parent.is_null() {
                    *value = (*(*parent).root).Handle;
                }
            }
            wined3d_mutex_unlock();

            return D3D_OK;
        }

        D3DRENDERSTATE_TEXTUREMAPBLEND => {
            *value = (*device).texture_map_blend as DWORD;
            return D3D_OK;
        }

        D3DRENDERSTATE_LIGHTING | D3DRENDERSTATE_NORMALIZENORMALS | D3DRENDERSTATE_LOCALVIEWER => {
            *value = 0xffffffff;
            return D3D_OK;
        }

        _ => {
            return IDirect3DDevice7_GetRenderState(
                core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
                state,
                value,
            );
        }
    }
}
unsafe extern "system" fn d3d_device2_GetRenderState(iface: *mut IDirect3DDevice2, state: D3DRENDERSTATETYPE, value: *mut DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice3_GetRenderState(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), state, value);
}

unsafe extern "C" fn d3d_device7_SetRenderState(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT = D3D_OK;

    TRACE!("iface %p, state %#x, value %#lx.\n", iface, state, value);
    wined3d_mutex_lock();
    /* Some render states need special care */
    match state {
        /*
         * The ddraw texture filter mapping works like this:
         *     D3DFILTER_NEAREST            Point min/mag, no mip
         *     D3DFILTER_MIPNEAREST         Point min/mag, point mip
         *     D3DFILTER_LINEARMIPNEAREST:  Point min/mag, linear mip
         *
         *     D3DFILTER_LINEAR             Linear min/mag, no mip
         *     D3DFILTER_MIPLINEAR          Linear min/mag, point mip
         *     D3DFILTER_LINEARMIPLINEAR    Linear min/mag, linear mip
         *
         * This is the opposite of the GL naming convention,
         * D3DFILTER_LINEARMIPNEAREST corresponds to GL_NEAREST_MIPMAP_LINEAR.
         */
        D3DRENDERSTATE_TEXTUREMAG => {
            let mut tex_mag: wined3d_texture_filter_type = 0;

            match value {
                D3DFILTER_NEAREST | D3DFILTER_MIPNEAREST | D3DFILTER_LINEARMIPNEAREST => {
                    tex_mag = WINED3D_TEXF_POINT;
                }
                D3DFILTER_LINEAR | D3DFILTER_MIPLINEAR | D3DFILTER_LINEARMIPLINEAR => {
                    tex_mag = WINED3D_TEXF_LINEAR;
                }
                _ => {
                    tex_mag = WINED3D_TEXF_POINT;
                    FIXME!("Unhandled texture mag %#lx.\n", value);
                }
            }

            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_MAG_FILTER, tex_mag);
        }

        D3DRENDERSTATE_TEXTUREMIN => {
            let mut tex_min: wined3d_texture_filter_type = 0;
            let mut tex_mip: wined3d_texture_filter_type = 0;

            match value {
                D3DFILTER_NEAREST => {
                    tex_min = WINED3D_TEXF_POINT;
                    tex_mip = WINED3D_TEXF_NONE;
                }
                D3DFILTER_LINEAR => {
                    tex_min = WINED3D_TEXF_LINEAR;
                    tex_mip = WINED3D_TEXF_NONE;
                }
                D3DFILTER_MIPNEAREST => {
                    tex_min = WINED3D_TEXF_POINT;
                    tex_mip = WINED3D_TEXF_POINT;
                }
                D3DFILTER_MIPLINEAR => {
                    tex_min = WINED3D_TEXF_LINEAR;
                    tex_mip = WINED3D_TEXF_POINT;
                }
                D3DFILTER_LINEARMIPNEAREST => {
                    tex_min = WINED3D_TEXF_POINT;
                    tex_mip = WINED3D_TEXF_LINEAR;
                }
                D3DFILTER_LINEARMIPLINEAR => {
                    tex_min = WINED3D_TEXF_LINEAR;
                    tex_mip = WINED3D_TEXF_LINEAR;
                }

                _ => {
                    FIXME!("Unhandled texture min %#lx.\n", value);
                    tex_min = WINED3D_TEXF_POINT;
                    tex_mip = WINED3D_TEXF_NONE;
                }
            }

            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_MIP_FILTER, tex_mip);
            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_MIN_FILTER, tex_min);
        }

        D3DRENDERSTATE_TEXTUREADDRESS => {
            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_ADDRESS_V, value);
            /* Drop through */
            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_ADDRESS_U, value);
        }
        D3DRENDERSTATE_TEXTUREADDRESSU => {
            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_ADDRESS_U, value);
        }
        D3DRENDERSTATE_TEXTUREADDRESSV => {
            wined3d_stateblock_set_sampler_state((*device).state, 0, WINED3D_SAMP_ADDRESS_V, value);
        }

        D3DRENDERSTATE_BORDERCOLOR => {
            /* This should probably just forward to the corresponding sampler
             * state. Needs tests. */
            FIXME!("Unhandled render state D3DRENDERSTATE_BORDERCOLOR.\n");
            hr = E_NOTIMPL;
        }

        D3DRENDERSTATE_TEXTUREHANDLE | D3DRENDERSTATE_TEXTUREMAPBLEND => {
            WARN!("Render state %#x is invalid in d3d7.\n", state);
            hr = DDERR_INVALIDPARAMS;
        }

        _ => {
            if state >= D3DRENDERSTATE_STIPPLEPATTERN00
                && state <= D3DRENDERSTATE_STIPPLEPATTERN31
            {
                FIXME!("Unhandled stipple pattern render state (%#x).\n", state);
                hr = E_NOTIMPL;
            } else {
                wined3d_stateblock_set_render_state((*device).update_state, wined3d_render_state_from_ddraw(state), value);
            }
        }
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_SetRenderState_FPUSetup(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: DWORD,
) -> HRESULT {
    d3d_device7_SetRenderState(iface, state, value)
}

unsafe extern "system" fn d3d_device7_SetRenderState_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    state: D3DRENDERSTATETYPE,
    value: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetRenderState(iface, state, value);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn fixup_texture_alpha_op(device: *mut d3d_device) {
    /* This fixup is required by the way D3DTBLEND_MODULATE maps to texture stage states.
       See d3d_device3_SetRenderState() for details. */
    let mut tex: *mut wined3d_texture;
    let mut tex_alpha: BOOL = (TRUE as i32);
    let mut ddfmt: DDPIXELFORMAT = core::mem::zeroed();

    if !((*device).legacyTextureBlending != 0 && (*device).texture_map_blend as u32 == D3DTBLEND_MODULATE as u32) {
        return;
    }

    tex = (*(*device).stateblock_state).textures[0 as usize] as *mut wined3d_texture;
    if !tex.is_null() {
        let mut desc: wined3d_resource_desc = core::mem::zeroed();

        wined3d_resource_get_desc(wined3d_texture_get_resource(tex), &mut desc);
        ddfmt.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
        ddrawformat_from_wined3dformat(&mut ddfmt, desc.format);
        if ddfmt.__bindgen_anon_5.dwRGBAlphaBitMask == 0 {
            tex_alpha = (FALSE as i32);
        }
    }

    /* Args 1 and 2 are already set to WINED3DTA_TEXTURE/WINED3DTA_CURRENT in case of D3DTBLEND_MODULATE */
    wined3d_stateblock_set_texture_stage_state(
        (*device).state,
        0,
        WINED3D_TSS_ALPHA_OP,
        if tex_alpha != 0 { WINED3D_TOP_SELECT_ARG1 } else { WINED3D_TOP_SELECT_ARG2 },
    );
}

unsafe extern "system" fn d3d_device3_SetRenderState(
    iface: *mut IDirect3DDevice3,
    state: D3DRENDERSTATETYPE,
    value: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, state %#x, value %#lx.\n", iface, state as ::core::ffi::c_uint, value);

    if state as ::core::ffi::c_uint >= D3DSTATE_OVERRIDE_BIAS {
        WARN!("Unhandled state %#x.\n", state as ::core::ffi::c_uint);
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    'sw: {
        match state as ::core::ffi::c_uint {
            D3DRENDERSTATE_TEXTUREHANDLE => {
                let mut surf: *mut ddraw_surface = core::ptr::null_mut();

                if value == 0 {
                    wined3d_stateblock_set_texture(
                        (*device).state,
                        0,
                        core::ptr::null_mut(),
                    );
                    hr = D3D_OK;
                    break 'sw;
                }

                surf = ddraw_get_object(
                    core::ptr::null_mut(),
                    value.wrapping_sub(1),
                    DDRAW_HANDLE_SURFACE,
                ) as *mut ddraw_surface;
                if surf.is_null() {
                    WARN!("Invalid texture handle.\n");
                    hr = DDERR_INVALIDPARAMS;
                    break 'sw;
                }

                hr = IDirect3DDevice3_SetTexture(
                    iface,
                    0,
                    core::ptr::addr_of_mut!((*surf).IDirect3DTexture2_iface),
                );
                break 'sw;
            }

            D3DRENDERSTATE_TEXTUREMAPBLEND => {
                if value == (*device).texture_map_blend as DWORD {
                    TRACE!("Application is setting the same value over, nothing to do.\n");

                    hr = D3D_OK;
                    break 'sw;
                }

                (*device).legacyTextureBlending = (TRUE as i32);
                (*device).texture_map_blend = value as D3DTEXTUREBLEND;

                match value {
                    D3DTBLEND_MODULATE => {
                        fixup_texture_alpha_op(device);

                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_OP,
                            WINED3D_TOP_MODULATE,
                        );
                    }

                    D3DTBLEND_ADD => {
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_OP,
                            WINED3D_TOP_ADD,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_OP,
                            WINED3D_TOP_SELECT_ARG2,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG2,
                            WINED3DTA_CURRENT,
                        );
                    }

                    D3DTBLEND_MODULATEALPHA => {
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_OP,
                            WINED3D_TOP_MODULATE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_OP,
                            WINED3D_TOP_MODULATE,
                        );
                    }

                    D3DTBLEND_COPY | D3DTBLEND_DECAL => {
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_OP,
                            WINED3D_TOP_SELECT_ARG1,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_OP,
                            WINED3D_TOP_SELECT_ARG1,
                        );
                    }

                    D3DTBLEND_DECALALPHA => {
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_OP,
                            WINED3D_TOP_BLEND_TEXTURE_ALPHA,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG1,
                            WINED3DTA_TEXTURE,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_COLOR_ARG2,
                            WINED3DTA_CURRENT,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_OP,
                            WINED3D_TOP_SELECT_ARG2,
                        );
                        wined3d_stateblock_set_texture_stage_state(
                            (*device).state,
                            0,
                            WINED3D_TSS_ALPHA_ARG2,
                            WINED3DTA_CURRENT,
                        );
                    }

                    _ => {
                        FIXME!("Unhandled texture environment %#lx.\n", value);
                    }
                }
                hr = D3D_OK;
                break 'sw;
            }

            D3DRENDERSTATE_LIGHTING | D3DRENDERSTATE_NORMALIZENORMALS | D3DRENDERSTATE_LOCALVIEWER => {
                hr = D3D_OK;
                break 'sw;
            }

            _ => {
                hr = IDirect3DDevice7_SetRenderState(
                    core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
                    state,
                    value,
                );
                break 'sw;
            }
        }
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device2_SetRenderState(iface: *mut IDirect3DDevice2, state: D3DRENDERSTATETYPE, value: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice3_SetRenderState(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), state, value);
}

unsafe extern "system" fn d3d_device3_SetLightState(iface: *mut IDirect3DDevice3, state: D3DLIGHTSTATETYPE, value: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut hr: HRESULT;

    if state == 0 || (state > D3DLIGHTSTATE_COLORVERTEX) {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if state == D3DLIGHTSTATE_MATERIAL {
        if value != 0 {
            let mut m: *mut d3d_material;

            m = ddraw_get_object(core::ptr::null_mut(), value.wrapping_sub(1), DDRAW_HANDLE_MATERIAL) as *mut d3d_material;
            if m.is_null() {
                WARN!("Invalid material handle.\n");
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }

            material_activate(m);
        }

        (*device).material = value;
    } else if state == D3DLIGHTSTATE_COLORMODEL {
        match value {
            D3DCOLOR_MONO => {
                ERR!("DDCOLOR_MONO should not happen!\n");
            }
            D3DCOLOR_RGB => {
                /* We are already in this mode */
                TRACE!("Setting color model to RGB (no-op).\n");
            }
            _ => {
                ERR!("Unknown color model!\n");
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }
        }
    } else {
        let mut rs: D3DRENDERSTATETYPE = 0 as D3DRENDERSTATETYPE;
        match state {
            D3DLIGHTSTATE_AMBIENT => {
                rs = D3DRENDERSTATE_AMBIENT;
            }
            D3DLIGHTSTATE_FOGMODE => {
                rs = D3DRENDERSTATE_FOGVERTEXMODE;
            }
            D3DLIGHTSTATE_FOGSTART => {
                rs = D3DRENDERSTATE_FOGSTART;
            }
            D3DLIGHTSTATE_FOGEND => {
                rs = D3DRENDERSTATE_FOGEND;
            }
            D3DLIGHTSTATE_FOGDENSITY => {
                rs = D3DRENDERSTATE_FOGDENSITY;
            }
            D3DLIGHTSTATE_COLORVERTEX => {
                rs = D3DRENDERSTATE_COLORVERTEX;
            }
            _ => {
                FIXME!("Unhandled D3DLIGHTSTATETYPE %#x.\n", state);
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }
        }

        hr = IDirect3DDevice7_SetRenderState(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), rs, value);
        wined3d_mutex_unlock();
        return hr;
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_SetLightState(iface: *mut IDirect3DDevice2, state: D3DLIGHTSTATETYPE, value: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return d3d_device3_SetLightState(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), state, value);
}

unsafe extern "system" fn d3d_device3_GetLightState(iface: *mut IDirect3DDevice3, state: D3DLIGHTSTATETYPE, value: *mut DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut hr: HRESULT;

    if state as u32 == 0 || state as u32 > D3DLIGHTSTATE_COLORVERTEX as u32
    {
        return DDERR_INVALIDPARAMS;
    }

    if value.is_null()
    {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if state as u32 == D3DLIGHTSTATE_MATERIAL as u32
    {
        *value = (*device).material;
    }
    else if state as u32 == D3DLIGHTSTATE_COLORMODEL as u32
    {
        *value = D3DCOLOR_RGB as DWORD;
    }
    else
    {
        let mut rs: D3DRENDERSTATETYPE = 0 as D3DRENDERSTATETYPE;
        match state as u32
        {
            D3DLIGHTSTATE_AMBIENT =>
            {
                rs = D3DRENDERSTATE_AMBIENT;
            }
            D3DLIGHTSTATE_FOGMODE =>
            {
                rs = D3DRENDERSTATE_FOGVERTEXMODE;
            }
            D3DLIGHTSTATE_FOGSTART =>
            {
                rs = D3DRENDERSTATE_FOGSTART;
            }
            D3DLIGHTSTATE_FOGEND =>
            {
                rs = D3DRENDERSTATE_FOGEND;
            }
            D3DLIGHTSTATE_FOGDENSITY =>
            {
                rs = D3DRENDERSTATE_FOGDENSITY;
            }
            D3DLIGHTSTATE_COLORVERTEX =>
            {
                rs = D3DRENDERSTATE_COLORVERTEX;
            }
            _ =>
            {
                FIXME!("Unhandled D3DLIGHTSTATETYPE %#x.\n", state);
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }
        }

        hr = IDirect3DDevice7_GetRenderState(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), rs, value);
        wined3d_mutex_unlock();
        return hr;
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device2_GetLightState(iface: *mut IDirect3DDevice2, state: D3DLIGHTSTATETYPE, value: *mut DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    return d3d_device3_GetLightState(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), state, value);
}

unsafe extern "C" fn d3d_device7_SetTransform(
    mut iface: *mut IDirect3DDevice7,
    mut state: D3DTRANSFORMSTATETYPE,
    mut matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    TRACE!(
        "iface %p, state %#x, matrix %p.\n",
        iface,
        state as u32,
        matrix
    );
    if matrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    wined3d_stateblock_set_transform(
        (*device).update_state,
        wined3d_transform_state_from_ddraw(state),
        matrix as *const wined3d_matrix,
    );
    wined3d_mutex_unlock();
    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_SetTransform_FPUSetup(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    d3d_device7_SetTransform(iface, state, matrix)
}
unsafe extern "system" fn d3d_device7_SetTransform_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetTransform(iface, state, matrix);
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_SetTransform(iface: *mut IDirect3DDevice3, state: D3DTRANSFORMSTATETYPE, matrix: *mut D3DMATRIX) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    TRACE!("iface %p, state %#x, matrix %p.\n", iface, state as u32, matrix);

    if matrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if state == D3DTRANSFORMSTATE_PROJECTION {
        let mut projection: wined3d_matrix = core::mem::zeroed();

        wined3d_mutex_lock();
        multiply_matrix(core::ptr::addr_of_mut!(projection), core::ptr::addr_of_mut!((*device).legacy_clipspace), matrix as *mut wined3d_matrix);
        wined3d_stateblock_set_transform((*device).state, WINED3D_TS_PROJECTION, core::ptr::addr_of_mut!(projection));
        memcpy(core::ptr::addr_of_mut!((*device).legacy_projection) as *mut core::ffi::c_void, matrix as *const core::ffi::c_void, core::mem::size_of::<D3DMATRIX>());
        wined3d_mutex_unlock();

        return D3D_OK;
    }

    IDirect3DDevice7_SetTransform(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), state, matrix)
}

unsafe extern "system" fn d3d_device2_SetTransform(iface: *mut IDirect3DDevice2, state: D3DTRANSFORMSTATETYPE, matrix: *mut D3DMATRIX) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice3_SetTransform(&mut (*device).IDirect3DDevice3_iface, state, matrix);
}

unsafe extern "C" fn d3d_device7_GetTransform(
    mut iface: *mut IDirect3DDevice7,
    mut state: D3DTRANSFORMSTATETYPE,
    mut matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    if matrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    memcpy(
        matrix as *mut core::ffi::c_void,
        (core::ptr::addr_of!((*(*device).stateblock_state).transforms) as *const wined3d_matrix)
            .add(wined3d_transform_state_from_ddraw(state) as usize) as *const core::ffi::c_void,
        core::mem::size_of::<D3DMATRIX>() as usize,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_GetTransform_FPUSetup(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    d3d_device7_GetTransform(iface, state, matrix)
}

unsafe extern "system" fn d3d_device7_GetTransform_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetTransform(iface, state, matrix);
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_GetTransform(
    iface: *mut IDirect3DDevice3,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    if matrix.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if state == D3DTRANSFORMSTATE_PROJECTION {
        wined3d_mutex_lock();
        memcpy(
            matrix as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*device).legacy_projection) as *const core::ffi::c_void,
            core::mem::size_of::<D3DMATRIX>(),
        );
        wined3d_mutex_unlock();
        return DD_OK;
    }

    IDirect3DDevice7_GetTransform(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        state,
        matrix,
    )
}

unsafe extern "system" fn d3d_device2_GetTransform(iface: *mut IDirect3DDevice2, state: D3DTRANSFORMSTATETYPE, matrix: *mut D3DMATRIX) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice3_GetTransform(&mut (*device).IDirect3DDevice3_iface, state, matrix);
}

unsafe extern "C" fn d3d_device7_MultiplyTransform(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    TRACE!(
        "iface %p, state %#x, matrix %p.\n",
        iface,
        state as u32,
        matrix
    );

    /* Note: D3DMATRIX is compatible with struct wined3d_matrix. */
    wined3d_mutex_lock();
    wined3d_stateblock_multiply_transform(
        (*device).state,
        wined3d_transform_state_from_ddraw(state),
        matrix as *mut wined3d_matrix,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_MultiplyTransform_FPUSetup(
    iface: *mut IDirect3DDevice7,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    d3d_device7_MultiplyTransform(iface, state, matrix)
}
unsafe extern "system" fn d3d_device7_MultiplyTransform_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut state: D3DTRANSFORMSTATETYPE,
    mut matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut old_fpucw: WORD = 0;
    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_MultiplyTransform(iface, state, matrix);
    set_fpu_control_word(old_fpucw);
    return hr;
}

unsafe extern "system" fn d3d_device3_MultiplyTransform(
    iface: *mut IDirect3DDevice3,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    if state == D3DTRANSFORMSTATE_PROJECTION {
        let mut projection: wined3d_matrix = core::mem::zeroed();
        let mut tmp: wined3d_matrix = core::mem::zeroed();

        wined3d_mutex_lock();
        multiply_matrix(
            &raw mut tmp,
            &raw mut (*device).legacy_projection,
            matrix as *mut wined3d_matrix,
        );
        multiply_matrix(
            &raw mut projection,
            &raw mut (*device).legacy_clipspace,
            &raw mut tmp,
        );
        wined3d_stateblock_set_transform(
            (*device).state,
            WINED3D_TS_PROJECTION,
            &raw mut projection,
        );
        (*device).legacy_projection = tmp;
        wined3d_mutex_unlock();

        return D3D_OK;
    }

    IDirect3DDevice7_MultiplyTransform(
        &raw mut (*device).IDirect3DDevice7_iface,
        state,
        matrix,
    )
}

unsafe extern "system" fn d3d_device2_MultiplyTransform(
    iface: *mut IDirect3DDevice2,
    state: D3DTRANSFORMSTATETYPE,
    matrix: *mut D3DMATRIX,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice3_MultiplyTransform(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        state,
        matrix,
    );
}

unsafe extern "C" fn d3d_device_sync_rendertarget(device: *mut d3d_device) {
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut dsv: *mut wined3d_rendertarget_view = core::ptr::null_mut();

    rtv = if !(*device).target.is_null() {
        ddraw_surface_get_rendertarget_view((*device).target)
    } else {
        core::ptr::null_mut()
    };
    if !rtv.is_null() {
        if (wined3d_device_context_set_rendertarget_views(
            (*device).immediate_context,
            0,
            1,
            core::ptr::addr_of_mut!(rtv),
            (FALSE) as BOOL,
        )) < 0
        {
            ERR!("wined3d_device_context_set_rendertarget_views failed.\n");
        }
    } else if (*device).target.is_null() {
        /* NULL device->target may appear when the game was setting invalid render target which in some cases
         * still keeps the invalid render target in the device even while returning an error.
         *
         * TODO: make render go nowhere instead of lefover render target (like it seems to work on Windows on HW devices
         * while may just crash on software devices. */
    }

    dsv = if !(*device).target_ds.is_null() {
        ddraw_surface_get_rendertarget_view((*device).target_ds)
    } else {
        core::ptr::null_mut()
    };
    if (wined3d_device_context_set_depth_stencil_view((*device).immediate_context, dsv)) < 0 {
        ERR!("wined3d_device_context_set_depth_stencil_view failed.\n");
    }
    wined3d_stateblock_depth_buffer_changed((*device).state);

    if (*device).hardware_device != 0 {
        return;
    }

    if !rtv.is_null() {
        ddraw_surface_get_draw_texture(
            wined3d_rendertarget_view_get_parent(rtv) as *mut ddraw_surface,
            DDRAW_SURFACE_RW,
        );
    }

    if !dsv.is_null() {
        ddraw_surface_get_draw_texture(
            wined3d_rendertarget_view_get_parent(dsv) as *mut ddraw_surface,
            DDRAW_SURFACE_RW,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn d3d_device_sync_surfaces(device: *mut d3d_device) {
    let state: *const wined3d_stateblock_state = (*device).stateblock_state;
    let mut surface: *mut ddraw_surface = core::ptr::null_mut();
    let mut i: core::ffi::c_uint;
    let mut j: core::ffi::c_uint;

    d3d_device_sync_rendertarget(device);

    if (*device).have_draw_textures == 0 {
        return;
    }

    i = 0;
    while (i as usize) < ARRAY_SIZE!((*state).textures) as usize {
        'body: {
            if (*state).textures[i as usize].is_null() {
                break 'body;
            }

            j = 0;
            loop {
                surface = wined3d_texture_get_sub_resource_parent((*state).textures[i as usize], j) as *mut ddraw_surface;
                if surface.is_null() {
                    break;
                }
                if (*surface).draw_texture.is_null() {
                    break;
                }
                ddraw_surface_get_draw_texture(surface, DDRAW_SURFACE_READ);
                j = j.wrapping_add(1);
            }
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn d3d_device_apply_state(device: *mut d3d_device, clear_state: BOOL) {
    if !(*device).ddraw.is_null() && (*(*device).ddraw).device_last_applied_state != device {
        wined3d_stateblock_primary_dirtify_all_states((*device).wined3d_device, (*device).state);
        (*(*device).ddraw).device_last_applied_state = device;
    }
    if clear_state != 0 {
        wined3d_stateblock_apply_clear_state((*device).state, (*device).wined3d_device);
    } else {
        wined3d_device_apply_stateblock((*device).wined3d_device, (*device).state);
    }
    d3d_device_sync_surfaces(device);
}

unsafe extern "C" fn d3d_device7_DrawPrimitive(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut stride: UINT = 0;
    let mut vb_pos: UINT = 0;
    let mut size: UINT = 0;
    let mut hr: HRESULT = 0;

    'done: {
        if vertex_count == 0 {
            WARN!("0 vertex count.\n");
            return D3D_OK;
        }

        /* Get the stride */
        stride = get_flexible_vertex_size(fvf) as UINT;
        size = (vertex_count as UINT).wrapping_mul(stride);

        wined3d_mutex_lock();

        hr = wined3d_streaming_buffer_upload(
            (*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).vertex_buffer),
            vertices,
            size,
            stride,
            core::ptr::addr_of_mut!(vb_pos),
        );
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            (*device).vertex_buffer.buffer,
            0,
            stride,
        );
        if hr < 0 {
            break 'done;
        }

        wined3d_stateblock_set_vertex_declaration(
            (*device).state,
            ddraw_find_decl((*device).ddraw, fvf),
        );
        wined3d_device_context_set_primitive_type(
            (*device).immediate_context,
            wined3d_primitive_type_from_ddraw(primitive_type),
            0,
        );
        d3d_device_apply_state(device, (FALSE) as BOOL);
        wined3d_device_context_draw(
            (*device).immediate_context,
            vb_pos / stride,
            vertex_count,
            0,
            0,
        );
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d_device7_DrawPrimitive_FPUSetup(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawPrimitive(iface, primitive_type, fvf, vertices, vertex_count, flags)
}

unsafe extern "system" fn d3d_device7_DrawPrimitive_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawPrimitive(iface, primitive_type, fvf, vertices, vertex_count, flags);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn setup_lighting(device: *const d3d_device, fvf: DWORD, flags: DWORD) {
    let mut enable: BOOL = (TRUE as i32);

    /* Ignore the D3DFVF_XYZRHW case here, wined3d takes care of that */
    if (*device).material == 0
        || (fvf & D3DFVF_NORMAL) == 0
        || (flags & D3DDP_DONOTLIGHT) != 0
    {
        enable = (FALSE as i32);
    }

    wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_LIGHTING, enable as ::core::ffi::c_uint);
}

unsafe extern "system" fn d3d_device3_DrawPrimitive(
    iface: *mut IDirect3DDevice3,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    setup_lighting(device, fvf, flags);

    return IDirect3DDevice7_DrawPrimitive(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        flags,
    );
}

unsafe extern "system" fn d3d_device2_DrawPrimitive(
    iface: *mut IDirect3DDevice2,
    primitive_type: D3DPRIMITIVETYPE,
    vertex_type: D3DVERTEXTYPE,
    vertices: *mut ::core::ffi::c_void,
    vertex_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut fvf: DWORD;

    match vertex_type as ::core::ffi::c_uint {
        D3DVT_VERTEX => {
            fvf = D3DFVF_VERTEX as DWORD;
        }
        D3DVT_LVERTEX => {
            fvf = D3DFVF_LVERTEX as DWORD;
        }
        D3DVT_TLVERTEX => {
            fvf = D3DFVF_TLVERTEX as DWORD;
        }
        _ => {
            FIXME!("Unhandled vertex type %#x.\n", vertex_type);
            return DDERR_INVALIDPARAMS;
        }
    }

    return d3d_device3_DrawPrimitive(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        flags,
    );
}

unsafe extern "C" fn d3d_device7_DrawIndexedPrimitive(iface: *mut IDirect3DDevice7, primitive_type: D3DPRIMITIVETYPE, fvf: DWORD, vertices: *mut ::core::ffi::c_void, vertex_count: DWORD, indices: *mut WORD, index_count: DWORD, flags: DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let idx_size: ::core::ffi::c_uint = (index_count as ::core::ffi::c_uint).wrapping_mul(::core::mem::size_of::<WORD>() as ::core::ffi::c_uint);
    let mut min_index: ::core::ffi::c_ushort = (USHRT_MAX as u16);
    let mut max_index: ::core::ffi::c_ushort = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;
    let mut stride: UINT = get_flexible_vertex_size(fvf);
    let mut vb_pos: UINT = 0;
    let mut ib_pos: UINT = 0;

    if vertex_count == 0 || index_count == 0 {
        WARN!("0 vertex or index count.\n");
        return D3D_OK;
    }

    i = 0;
    while i < index_count {
        min_index = min!(min_index as ::core::ffi::c_int, *indices.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_ushort;
        max_index = max!(max_index as ::core::ffi::c_int, *indices.offset(i as isize) as ::core::ffi::c_int) as ::core::ffi::c_ushort;
        i = i.wrapping_add(1);
    }

    wined3d_mutex_lock();

    'done: {
        hr = wined3d_streaming_buffer_upload((*device).wined3d_device, core::ptr::addr_of_mut!((*device).vertex_buffer),
            (vertices as *mut ::core::ffi::c_char).offset((min_index as UINT).wrapping_mul(stride) as isize) as *const ::core::ffi::c_void,
            ((max_index as ::core::ffi::c_int + 1 - min_index as ::core::ffi::c_int) as ::core::ffi::c_uint).wrapping_mul(stride),
            stride, core::ptr::addr_of_mut!(vb_pos));
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_streaming_buffer_upload((*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).index_buffer), indices as *const ::core::ffi::c_void, idx_size,
            ::core::mem::size_of::<WORD>() as ::core::ffi::c_uint, core::ptr::addr_of_mut!(ib_pos));
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_stateblock_set_stream_source((*device).state, 0, (*device).vertex_buffer.buffer, 0, stride);
        if hr < 0 {
            break 'done;
        }
        wined3d_stateblock_set_index_buffer((*device).state, (*device).index_buffer.buffer, WINED3DFMT_R16_UINT);

        wined3d_stateblock_set_vertex_declaration((*device).state, ddraw_find_decl((*device).ddraw, fvf));
        wined3d_device_context_set_primitive_type((*device).immediate_context,
            wined3d_primitive_type_from_ddraw(primitive_type), 0);
        d3d_device_apply_state(device, (FALSE) as BOOL);
        wined3d_device_context_draw_indexed((*device).immediate_context, (vb_pos.wrapping_div(stride)) as ::core::ffi::c_int - min_index as ::core::ffi::c_int,
            ib_pos.wrapping_div(::core::mem::size_of::<WORD>() as UINT), index_count, 0, 0);
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d_device7_DrawIndexedPrimitive_FPUSetup(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    indices: *mut WORD,
    index_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawIndexedPrimitive(iface, primitive_type, fvf, vertices, vertex_count, indices, index_count, flags)
}
unsafe extern "system" fn d3d_device7_DrawIndexedPrimitive_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut ::core::ffi::c_void,
    vertex_count: DWORD,
    indices: *mut WORD,
    index_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawIndexedPrimitive(
        iface,
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        indices,
        index_count,
        flags,
    );
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_DrawIndexedPrimitive(
    iface: *mut IDirect3DDevice3,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    vertices: *mut core::ffi::c_void,
    vertex_count: DWORD,
    indices: *mut WORD,
    index_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    setup_lighting(device, fvf, flags);

    return IDirect3DDevice7_DrawIndexedPrimitive(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        indices,
        index_count,
        flags,
    );
}
unsafe extern "system" fn d3d_device2_DrawIndexedPrimitive(
    mut iface: *mut IDirect3DDevice2,
    mut primitive_type: D3DPRIMITIVETYPE,
    mut vertex_type: D3DVERTEXTYPE,
    mut vertices: *mut ::core::ffi::c_void,
    mut vertex_count: DWORD,
    mut indices: *mut WORD,
    mut index_count: DWORD,
    mut flags: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    let mut fvf: DWORD = 0;
    TRACE!(
        "iface %p, primitive_type %#x, vertex_type %#x, vertices %p, vertex_count %lu, indices %p, index_count %lu, flags %#lx.\n",
        iface,
        primitive_type as ::core::ffi::c_uint,
        vertex_type as ::core::ffi::c_uint,
        vertices,
        vertex_count,
        indices,
        index_count,
        flags,
    );
    match vertex_type as ::core::ffi::c_uint {
        D3DVT_VERTEX => {
            fvf = D3DFVF_VERTEX as DWORD;
        }
        D3DVT_LVERTEX => {
            fvf = D3DFVF_LVERTEX as DWORD;
        }
        D3DVT_TLVERTEX => {
            fvf = D3DFVF_TLVERTEX as DWORD;
        }
        _ => {
            ERR!(
                "Unhandled vertex type %#x.\n",
                vertex_type as ::core::ffi::c_uint,
            );
            return DDERR_INVALIDPARAMS;
        }
    }
    return d3d_device3_DrawIndexedPrimitive(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        primitive_type,
        fvf,
        vertices,
        vertex_count,
        indices,
        index_count,
        flags,
    );
}

unsafe extern "system" fn d3d_device3_End(
    mut iface: *mut IDirect3DDevice3,
    mut flags: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return d3d_device3_DrawPrimitive(
        &raw mut (*device).IDirect3DDevice3_iface,
        (*device).primitive_type,
        (*device).vertex_type,
        (*device).sysmem_vertex_buffer as *mut ::core::ffi::c_void,
        (*device).nb_vertices,
        (*device).render_flags,
    );
}
unsafe extern "system" fn d3d_device2_End(mut iface: *mut IDirect3DDevice2, mut flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return d3d_device3_End(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), flags);
}
unsafe extern "system" fn d3d_device7_SetClipStatus(
    iface: *mut IDirect3DDevice7,
    clip_status: *mut D3DCLIPSTATUS,
) -> HRESULT {
    // FIXME("iface %p, clip_status %p stub!\n", iface, clip_status);
    D3D_OK
}

unsafe extern "system" fn d3d_device3_SetClipStatus(iface: *mut IDirect3DDevice3, clip_status: *mut D3DCLIPSTATUS) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IDirect3DDevice7_SetClipStatus(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), clip_status);
}

unsafe extern "system" fn d3d_device2_SetClipStatus(iface: *mut IDirect3DDevice2, clip_status: *mut D3DCLIPSTATUS) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice7_SetClipStatus(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), clip_status);
}
unsafe extern "system" fn d3d_device7_GetClipStatus(
    mut iface: *mut IDirect3DDevice7,
    mut clip_status: *mut D3DCLIPSTATUS,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut vp: wined3d_viewport = wined3d_viewport {
        x: 0.,
        y: 0.,
        width: 0.,
        height: 0.,
        min_z: 0.,
        max_z: 0.,
    };
    WINE_FIXME!("iface %p, clip_status %p stub.\n", iface, clip_status);
    vp = (*(*device).stateblock_state).viewport;
    (*clip_status).minx = vp.x;
    (*clip_status).maxx = vp.x + vp.width;
    (*clip_status).miny = vp.y;
    (*clip_status).maxy = vp.y + vp.height;
    (*clip_status).minz = 0.0f32;
    (*clip_status).maxz = 0.0f32;
    (*clip_status).dwFlags = D3DCLIPSTATUS_EXTENTS2 as DWORD;
    (*clip_status).dwStatus = 0 as DWORD;
    return D3D_OK;
}

unsafe extern "system" fn d3d_device3_GetClipStatus(iface: *mut IDirect3DDevice3, clip_status: *mut D3DCLIPSTATUS) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IDirect3DDevice7_GetClipStatus(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), clip_status);
}

unsafe extern "system" fn d3d_device2_GetClipStatus(iface: *mut IDirect3DDevice2, clip_status: *mut D3DCLIPSTATUS) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice2(iface);

    return IDirect3DDevice7_GetClipStatus(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), clip_status);
}

unsafe extern "C" fn pack_strided_data(
    mut dst: *mut BYTE,
    mut count: DWORD,
    mut src: *const D3DDRAWPRIMITIVESTRIDEDDATA,
    mut fvf: DWORD,
) {
    let mut i: DWORD = 0;
    let mut tex: DWORD = 0;
    let mut offset: DWORD = 0;
    i = 0 as DWORD;
    while i < count {
        /* The contents of the strided data are determined by the fvf,
         * not by the members set in src. So it's valid
         * to have diffuse.lpvData set to 0xdeadbeef if the diffuse flag is
         * not set in the fvf. */
        if (fvf & (D3DFVF_POSITION_MASK as DWORD)) != 0 {
            offset = i.wrapping_mul((*src).position.dwStride);
            if (fvf & (D3DFVF_XYZRHW as DWORD)) != 0 {
                memcpy(
                    dst as *mut core::ffi::c_void,
                    ((*src).position.lpvData as *mut BYTE).add(offset as usize)
                        as *const core::ffi::c_void,
                    (4 as usize)
                        .wrapping_mul(core::mem::size_of::<f32>()),
                );
                dst = dst.add(
                    (4 as usize)
                        .wrapping_mul(core::mem::size_of::<f32>()),
                );
            } else {
                memcpy(
                    dst as *mut core::ffi::c_void,
                    ((*src).position.lpvData as *mut BYTE).add(offset as usize)
                        as *const core::ffi::c_void,
                    (3 as usize)
                        .wrapping_mul(core::mem::size_of::<f32>()),
                );
                dst = dst.add(
                    (3 as usize)
                        .wrapping_mul(core::mem::size_of::<f32>()),
                );
            }
        }

        if (fvf & (D3DFVF_NORMAL as DWORD)) != 0 {
            offset = i.wrapping_mul((*src).normal.dwStride);
            memcpy(
                dst as *mut core::ffi::c_void,
                ((*src).normal.lpvData as *mut BYTE).add(offset as usize)
                    as *const core::ffi::c_void,
                (3 as usize)
                    .wrapping_mul(core::mem::size_of::<f32>()),
            );
            dst = dst.add(
                (3 as usize)
                    .wrapping_mul(core::mem::size_of::<f32>()),
            );
        }

        if (fvf & (D3DFVF_DIFFUSE as DWORD)) != 0 {
            offset = i.wrapping_mul((*src).diffuse.dwStride);
            memcpy(
                dst as *mut core::ffi::c_void,
                ((*src).diffuse.lpvData as *mut BYTE).add(offset as usize)
                    as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>() as usize,
            );
            dst = dst.add(core::mem::size_of::<DWORD>() as usize);
        }

        if (fvf & (D3DFVF_SPECULAR as DWORD)) != 0 {
            offset = i.wrapping_mul((*src).specular.dwStride);
            memcpy(
                dst as *mut core::ffi::c_void,
                ((*src).specular.lpvData as *mut BYTE).add(offset as usize)
                    as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>() as usize,
            );
            dst = dst.add(core::mem::size_of::<DWORD>() as usize);
        }

        tex = 0 as DWORD;
        while tex < GET_TEXCOUNT_FROM_FVF!(fvf) {
            let mut attrib_count: DWORD = GET_TEXCOORD_SIZE_FROM_FVF!(fvf, tex);
            offset = i.wrapping_mul((*src).textureCoords[tex as usize].dwStride);
            memcpy(
                dst as *mut core::ffi::c_void,
                ((*src).textureCoords[tex as usize].lpvData as *mut BYTE).add(offset as usize)
                    as *const core::ffi::c_void,
                (attrib_count as usize).wrapping_mul(core::mem::size_of::<f32>()),
            );
            dst = dst.add(
                (attrib_count as usize).wrapping_mul(core::mem::size_of::<f32>()),
            );
            tex = tex.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn d3d_device7_DrawPrimitiveStrided(iface: *mut IDirect3DDevice7, primitive_type: D3DPRIMITIVETYPE, fvf: DWORD, strided_data: *mut D3DDRAWPRIMITIVESTRIDEDDATA, vertex_count: DWORD, flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT = 0;
    let mut dst_stride: UINT = get_flexible_vertex_size(fvf) as UINT;
    let mut dst_size: UINT = dst_stride.wrapping_mul(vertex_count as UINT);
    let mut dst_data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut vb_pos: UINT = 0;

    if vertex_count == 0 {
        WARN!("0 vertex count.\n");
        return D3D_OK;
    }

    wined3d_mutex_lock();

    'done: {
        hr = wined3d_streaming_buffer_map((*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).vertex_buffer), dst_size, dst_stride, core::ptr::addr_of_mut!(vb_pos), core::ptr::addr_of_mut!(dst_data));
        if hr < 0 {
            break 'done;
        }
        pack_strided_data(dst_data as *mut BYTE, vertex_count, strided_data, fvf);
        wined3d_streaming_buffer_unmap(core::ptr::addr_of_mut!((*device).vertex_buffer));

        hr = wined3d_stateblock_set_stream_source((*device).state, 0, (*device).vertex_buffer.buffer, 0, dst_stride);
        if hr < 0 {
            break 'done;
        }
        wined3d_stateblock_set_vertex_declaration((*device).state, ddraw_find_decl((*device).ddraw, fvf));

        wined3d_device_context_set_primitive_type((*device).immediate_context,
            wined3d_primitive_type_from_ddraw(primitive_type), 0);
        d3d_device_apply_state(device, (FALSE) as BOOL);
        wined3d_device_context_draw((*device).immediate_context, vb_pos / dst_stride, vertex_count, 0, 0);
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d_device7_DrawPrimitiveStrided_FPUSetup(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    VertexType: DWORD,
    D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    VertexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawPrimitiveStrided(iface, PrimitiveType, VertexType, D3DDrawPrimStrideData, VertexCount, Flags)
}
unsafe extern "system" fn d3d_device7_DrawPrimitiveStrided_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut PrimitiveType: D3DPRIMITIVETYPE,
    mut VertexType: DWORD,
    mut D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    mut VertexCount: DWORD,
    mut Flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawPrimitiveStrided(
        iface,
        PrimitiveType,
        VertexType,
        D3DDrawPrimStrideData,
        VertexCount,
        Flags,
    );
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_DrawPrimitiveStrided(
    iface: *mut IDirect3DDevice3,
    PrimitiveType: D3DPRIMITIVETYPE,
    VertexType: DWORD,
    D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    VertexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    setup_lighting(device as *const d3d_device, VertexType, Flags);

    return IDirect3DDevice7_DrawPrimitiveStrided(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        PrimitiveType,
        VertexType,
        D3DDrawPrimStrideData,
        VertexCount,
        Flags,
    );
}

unsafe extern "C" fn d3d_device7_DrawIndexedPrimitiveStrided(
    iface: *mut IDirect3DDevice7,
    primitive_type: D3DPRIMITIVETYPE,
    fvf: DWORD,
    strided_data: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    vertex_count: DWORD,
    indices: *mut WORD,
    index_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let vtx_dst_stride: UINT = get_flexible_vertex_size(fvf) as UINT;
    let vtx_dst_size: UINT = vertex_count.wrapping_mul(vtx_dst_stride);
    let idx_size: UINT = index_count.wrapping_mul(core::mem::size_of::<WORD>() as UINT);
    let mut dst_data: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut vb_pos: UINT = 0;
    let mut ib_pos: UINT = 0;
    let mut hr: HRESULT = 0;

    TRACE!(
        "iface %p, primitive_type %#x, fvf %#lx, strided_data %p, vertex_count %lu, indices %p, index_count %lu, flags %#lx.\n",
        iface,
        primitive_type,
        fvf,
        strided_data,
        vertex_count,
        indices,
        index_count,
        flags
    );

    if vertex_count == 0 || index_count == 0 {
        WARN!("0 vertex or index count.\n");
        return D3D_OK;
    }

    wined3d_mutex_lock();

    'done: {
        hr = wined3d_streaming_buffer_map(
            (*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).vertex_buffer),
            vtx_dst_size,
            vtx_dst_stride,
            core::ptr::addr_of_mut!(vb_pos),
            core::ptr::addr_of_mut!(dst_data),
        );
        if hr < 0 {
            break 'done;
        }
        pack_strided_data(dst_data as *mut BYTE, vertex_count, strided_data, fvf);
        wined3d_streaming_buffer_unmap(core::ptr::addr_of_mut!((*device).vertex_buffer));

        hr = wined3d_streaming_buffer_upload(
            (*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).index_buffer),
            indices as *const core::ffi::c_void,
            idx_size,
            core::mem::size_of::<WORD>() as UINT,
            core::ptr::addr_of_mut!(ib_pos),
        );
        if hr < 0 {
            break 'done;
        }

        hr = wined3d_stateblock_set_stream_source(
            (*device).state,
            0,
            (*device).vertex_buffer.buffer,
            0,
            vtx_dst_stride,
        );
        if hr < 0 {
            break 'done;
        }
        wined3d_stateblock_set_index_buffer((*device).state, (*device).index_buffer.buffer, WINED3DFMT_R16_UINT);

        wined3d_stateblock_set_vertex_declaration((*device).state, ddraw_find_decl((*device).ddraw, fvf));
        wined3d_device_context_set_primitive_type(
            (*device).immediate_context,
            wined3d_primitive_type_from_ddraw(primitive_type),
            0,
        );
        d3d_device_apply_state(device, (FALSE) as BOOL);
        wined3d_device_context_draw_indexed(
            (*device).immediate_context,
            (vb_pos / vtx_dst_stride) as i32,
            ib_pos / core::mem::size_of::<WORD>() as UINT,
            index_count,
            0,
            0,
        );
    }

    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn d3d_device7_DrawIndexedPrimitiveStrided_FPUSetup(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    VertexType: DWORD,
    D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    VertexCount: DWORD,
    Indices: *mut WORD,
    IndexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawIndexedPrimitiveStrided(iface, PrimitiveType, VertexType, D3DDrawPrimStrideData, VertexCount, Indices, IndexCount, Flags)
}
unsafe extern "system" fn d3d_device7_DrawIndexedPrimitiveStrided_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    VertexType: DWORD,
    D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA,
    VertexCount: DWORD,
    Indices: *mut WORD,
    IndexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawIndexedPrimitiveStrided(
        iface,
        PrimitiveType,
        VertexType,
        D3DDrawPrimStrideData,
        VertexCount,
        Indices,
        IndexCount,
        Flags,
    );
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_DrawIndexedPrimitiveStrided(iface: *mut IDirect3DDevice3, PrimitiveType: D3DPRIMITIVETYPE, VertexType: DWORD, D3DDrawPrimStrideData: *mut D3DDRAWPRIMITIVESTRIDEDDATA, VertexCount: DWORD, Indices: *mut WORD, IndexCount: DWORD, Flags: DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    setup_lighting(device, VertexType, Flags);

    return IDirect3DDevice7_DrawIndexedPrimitiveStrided(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), PrimitiveType, VertexType, D3DDrawPrimStrideData, VertexCount, Indices, IndexCount, Flags);
}

unsafe extern "C" fn d3d_device7_DrawPrimitiveVB(iface: *mut IDirect3DDevice7, primitive_type: D3DPRIMITIVETYPE, vb: *mut IDirect3DVertexBuffer7, start_vertex: DWORD, vertex_count: DWORD, flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut vb_impl: *mut d3d_vertex_buffer = unsafe_impl_from_IDirect3DVertexBuffer7(vb);
    let mut wined3d_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut wined3d_map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    let mut wined3d_box: wined3d_box = wined3d_box {
        left: 0 as UINT,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut stride: DWORD = 0;
    let mut hr: HRESULT = 0;

    if vertex_count == 0 {
        WARN!("0 vertex count.\n");
        return D3D_OK;
    }

    (*vb_impl).discarded = false;

    stride = get_flexible_vertex_size((*vb_impl).fvf);

    if (*vb_impl).sysmem {
        wined3d_mutex_lock();
        wined3d_resource = wined3d_buffer_get_resource((*vb_impl).wined3d_buffer);
        wined3d_box.left = start_vertex.wrapping_mul(stride) as UINT;
        wined3d_box.right = (wined3d_box.left as DWORD).wrapping_add(vertex_count.wrapping_mul(stride)) as UINT;
        hr = wined3d_resource_map(
            wined3d_resource,
            0,
            core::ptr::addr_of_mut!(wined3d_map_desc),
            core::ptr::addr_of_mut!(wined3d_box),
            WINED3D_MAP_READ,
        );
        if hr < 0 {
            wined3d_mutex_unlock();
            return D3DERR_VERTEXBUFFERLOCKED;
        }
        hr = d3d_device7_DrawPrimitive(
            iface,
            primitive_type,
            (*vb_impl).fvf,
            wined3d_map_desc.data,
            vertex_count,
            flags,
        );
        wined3d_resource_unmap(wined3d_resource, 0);
        wined3d_mutex_unlock();
        return hr;
    }

    wined3d_mutex_lock();
    wined3d_stateblock_set_vertex_declaration((*device).state, (*vb_impl).wined3d_declaration);
    hr = wined3d_stateblock_set_stream_source(
        (*device).state,
        0,
        (*vb_impl).wined3d_buffer,
        0,
        stride as UINT,
    );
    if hr < 0 {
        WARN!("Failed to set stream source, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    /* Now draw the primitives */
    wined3d_device_context_set_primitive_type(
        (*device).immediate_context,
        wined3d_primitive_type_from_ddraw(primitive_type),
        0,
    );
    d3d_device_apply_state(device, (FALSE) as BOOL);
    wined3d_device_context_draw(
        (*device).immediate_context,
        start_vertex as ::core::ffi::c_uint,
        vertex_count as ::core::ffi::c_uint,
        0,
        0,
    );

    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_DrawPrimitiveVB_FPUSetup(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    D3DVertexBuf: *mut IDirect3DVertexBuffer7,
    StartVertex: DWORD,
    NumVertices: DWORD,
    Flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawPrimitiveVB(iface, PrimitiveType, D3DVertexBuf, StartVertex, NumVertices, Flags)
}
unsafe extern "system" fn d3d_device7_DrawPrimitiveVB_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut PrimitiveType: D3DPRIMITIVETYPE,
    mut D3DVertexBuf: *mut IDirect3DVertexBuffer7,
    mut StartVertex: DWORD,
    mut NumVertices: DWORD,
    mut Flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut old_fpucw: WORD = 0;
    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawPrimitiveVB(
        iface,
        PrimitiveType,
        D3DVertexBuf,
        StartVertex,
        NumVertices,
        Flags,
    );
    set_fpu_control_word(old_fpucw);
    return hr;
}

unsafe extern "system" fn d3d_device3_DrawPrimitiveVB(iface: *mut IDirect3DDevice3, PrimitiveType: D3DPRIMITIVETYPE, D3DVertexBuf: *mut IDirect3DVertexBuffer, StartVertex: DWORD, NumVertices: DWORD, Flags: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut vb: *mut d3d_vertex_buffer = unsafe_impl_from_IDirect3DVertexBuffer7(D3DVertexBuf as *mut IDirect3DVertexBuffer7);
    WINE_TRACE!("iface %p, primitive_type %#x, vb %p, start_vertex %lu, vertex_count %lu, flags %#lx.\n", iface, PrimitiveType as ::core::ffi::c_uint, D3DVertexBuf, StartVertex, NumVertices, Flags);
    setup_lighting(device, (*vb).fvf, Flags);
    return IDirect3DDevice7_DrawPrimitiveVB(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        PrimitiveType,
        core::ptr::addr_of_mut!((*vb).IDirect3DVertexBuffer7_iface),
        StartVertex,
        NumVertices,
        Flags,
    );
}

unsafe extern "C" fn d3d_device7_DrawIndexedPrimitiveVB(iface: *mut IDirect3DDevice7, primitive_type: D3DPRIMITIVETYPE, vb: *mut IDirect3DVertexBuffer7, start_vertex: DWORD, vertex_count: DWORD, indices: *mut WORD, index_count: DWORD, flags: DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let vb_impl: *mut d3d_vertex_buffer = unsafe_impl_from_IDirect3DVertexBuffer7(vb);
    let stride: DWORD = get_flexible_vertex_size((*vb_impl).fvf);
    let mut wined3d_resource: *mut wined3d_resource = core::ptr::null_mut();
    let mut wined3d_map_desc: wined3d_map_desc = core::mem::zeroed();
    let mut wined3d_box: wined3d_box = core::mem::zeroed();
    let mut hr: HRESULT;
    let mut ib_pos: UINT = 0;

    if vertex_count == 0 || index_count == 0 {
        WARN!("0 vertex or index count.\n");
        return D3D_OK;
    }

    (*vb_impl).discarded = false;

    if (*vb_impl).sysmem != false {
        wined3d_mutex_lock();
        wined3d_box.left = (start_vertex.wrapping_mul(stride)) as UINT;
        wined3d_box.right = ((wined3d_box.left as DWORD).wrapping_add(vertex_count.wrapping_mul(stride))) as UINT;
        wined3d_resource = wined3d_buffer_get_resource((*vb_impl).wined3d_buffer);
        if {
            hr = wined3d_resource_map(wined3d_resource, 0, core::ptr::addr_of_mut!(wined3d_map_desc),
                core::ptr::addr_of_mut!(wined3d_box), WINED3D_MAP_READ);
            hr < 0
        } {
            wined3d_mutex_unlock();
            return D3DERR_VERTEXBUFFERLOCKED;
        }
        hr = d3d_device7_DrawIndexedPrimitive(iface, primitive_type, (*vb_impl).fvf,
                wined3d_map_desc.data, vertex_count, indices, index_count, flags);
        wined3d_resource_unmap(wined3d_resource, 0);
        wined3d_mutex_unlock();
        return hr;
    }

    /* Steps:
     * 1) Upload the indices to the index buffer
     * 2) Set the index source
     * 3) Set the Vertex Buffer as the Stream source
     * 4) Call wined3d_device_context_draw_indexed()
     */

    wined3d_mutex_lock();

    wined3d_stateblock_set_vertex_declaration((*device).state, (*vb_impl).wined3d_declaration);

    if {
        hr = wined3d_streaming_buffer_upload((*device).wined3d_device,
            core::ptr::addr_of_mut!((*device).index_buffer), indices as *const core::ffi::c_void,
            ((index_count as usize) * core::mem::size_of::<WORD>()) as UINT,
            core::mem::size_of::<WORD>() as UINT,
            core::ptr::addr_of_mut!(ib_pos));
        hr < 0
    } {
        wined3d_mutex_unlock();
        return hr;
    }

    /* Set the index stream */
    wined3d_stateblock_set_index_buffer((*device).state, (*device).index_buffer.buffer, WINED3DFMT_R16_UINT);

    /* Set the vertex stream source */
    if {
        hr = wined3d_stateblock_set_stream_source((*device).state,
            0, (*vb_impl).wined3d_buffer, 0, stride as UINT);
        hr < 0
    } {
        ERR!("Failed to set stream source for device %p, hr %#lx.\n", device, hr);
        wined3d_mutex_unlock();
        return hr;
    }

    wined3d_device_context_set_primitive_type((*device).immediate_context,
            wined3d_primitive_type_from_ddraw(primitive_type), 0);
    d3d_device_apply_state(device, (FALSE) as BOOL);
    wined3d_device_context_draw_indexed((*device).immediate_context, start_vertex as core::ffi::c_int,
            ((ib_pos as usize) / core::mem::size_of::<WORD>()) as UINT, index_count as UINT, 0, 0);

    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_DrawIndexedPrimitiveVB_FPUSetup(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    D3DVertexBuf: *mut IDirect3DVertexBuffer7,
    StartVertex: DWORD,
    NumVertices: DWORD,
    Indices: *mut WORD,
    IndexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    d3d_device7_DrawIndexedPrimitiveVB(iface, PrimitiveType, D3DVertexBuf, StartVertex, NumVertices, Indices, IndexCount, Flags)
}

unsafe extern "system" fn d3d_device7_DrawIndexedPrimitiveVB_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    PrimitiveType: D3DPRIMITIVETYPE,
    D3DVertexBuf: *mut IDirect3DVertexBuffer7,
    StartVertex: DWORD,
    NumVertices: DWORD,
    Indices: *mut WORD,
    IndexCount: DWORD,
    Flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DrawIndexedPrimitiveVB(iface, PrimitiveType,
            D3DVertexBuf, StartVertex, NumVertices, Indices, IndexCount, Flags);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_DrawIndexedPrimitiveVB(
    iface: *mut IDirect3DDevice3,
    primitive_type: D3DPRIMITIVETYPE,
    vertex_buffer: *mut IDirect3DVertexBuffer,
    indices: *mut WORD,
    index_count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let vb: *mut d3d_vertex_buffer =
        unsafe_impl_from_IDirect3DVertexBuffer7(vertex_buffer as *mut IDirect3DVertexBuffer7);
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut stride: DWORD;

    WINE_TRACE!("iface %p, primitive_type %#x, vb %p, indices %p, index_count %lu, flags %#lx.\n", iface, primitive_type as u32, vertex_buffer, indices, index_count, flags);

    setup_lighting(device, (*vb).fvf, flags);

    stride = get_flexible_vertex_size((*vb).fvf);
    if stride == 0 {
        return D3D_OK;
    }

    IDirect3DDevice7_DrawIndexedPrimitiveVB(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        primitive_type,
        core::ptr::addr_of_mut!((*vb).IDirect3DVertexBuffer7_iface),
        0,
        (*vb).size / stride,
        indices,
        index_count,
        flags,
    )
}

unsafe extern "C" fn in_plane(
    mut idx: UINT,
    mut p: wined3d_vec4,
    mut center: D3DVECTOR,
    mut radius: D3DVALUE,
    mut equality: BOOL,
) -> DWORD {
    let mut distance: f32;
    let mut norm: f32;
    norm = sqrtf(p.x * p.x + p.y * p.y + p.z * p.z);
    distance = (p.x * center.__bindgen_anon_1.x + p.y * center.__bindgen_anon_2.y + p.z * center.__bindgen_anon_3.z + p.w) / norm;
    if equality != 0 {
        if fabs(distance as f64) <= radius as f64 {
            return (D3DSTATUS_CLIPUNIONLEFT << idx) as DWORD;
        }
        if distance <= -radius {
            return ((D3DSTATUS_CLIPUNIONLEFT | D3DSTATUS_CLIPINTERSECTIONLEFT) << idx) as DWORD;
        }
    } else {
        if fabs(distance as f64) < radius as f64 {
            return (D3DSTATUS_CLIPUNIONLEFT << idx) as DWORD;
        }
        if distance < -radius {
            return ((D3DSTATUS_CLIPUNIONLEFT | D3DSTATUS_CLIPINTERSECTIONLEFT) << idx) as DWORD;
        }
    }
    return 0 as DWORD;
}

unsafe extern "C" fn prepare_clip_space_planes(device: *mut d3d_device, plane: *mut wined3d_vec4) {
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut m: wined3d_matrix = core::mem::zeroed();

    /* We want the wined3d matrices since those include the legacy viewport
     * transformation. */
    wined3d_mutex_lock();
    state = (*device).stateblock_state;
    multiply_matrix(
        core::ptr::addr_of_mut!(m),
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_VIEW as usize]) as *const wined3d_matrix,
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_WORLD as usize]) as *const wined3d_matrix,
    );
    multiply_matrix(
        core::ptr::addr_of_mut!(m),
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_PROJECTION as usize]) as *const wined3d_matrix,
        core::ptr::addr_of_mut!(m),
    );
    wined3d_mutex_unlock();

    /* Left plane. */
    (*plane.offset(0 as isize)).x = m._14 + m._11;
    (*plane.offset(0 as isize)).y = m._24 + m._21;
    (*plane.offset(0 as isize)).z = m._34 + m._31;
    (*plane.offset(0 as isize)).w = m._44 + m._41;

    /* Right plane. */
    (*plane.offset(1 as isize)).x = m._14 - m._11;
    (*plane.offset(1 as isize)).y = m._24 - m._21;
    (*plane.offset(1 as isize)).z = m._34 - m._31;
    (*plane.offset(1 as isize)).w = m._44 - m._41;

    /* Top plane. */
    (*plane.offset(2 as isize)).x = m._14 - m._12;
    (*plane.offset(2 as isize)).y = m._24 - m._22;
    (*plane.offset(2 as isize)).z = m._34 - m._32;
    (*plane.offset(2 as isize)).w = m._44 - m._42;

    /* Bottom plane. */
    (*plane.offset(3 as isize)).x = m._14 + m._12;
    (*plane.offset(3 as isize)).y = m._24 + m._22;
    (*plane.offset(3 as isize)).z = m._34 + m._32;
    (*plane.offset(3 as isize)).w = m._44 + m._42;

    /* Front plane. */
    (*plane.offset(4 as isize)).x = m._13;
    (*plane.offset(4 as isize)).y = m._23;
    (*plane.offset(4 as isize)).z = m._33;
    (*plane.offset(4 as isize)).w = m._43;

    /* Back plane. */
    (*plane.offset(5 as isize)).x = m._14 - m._13;
    (*plane.offset(5 as isize)).y = m._24 - m._23;
    (*plane.offset(5 as isize)).z = m._34 - m._33;
    (*plane.offset(5 as isize)).w = m._44 - m._43;
}

unsafe extern "C" fn compute_sphere_visibility(planes: *const wined3d_vec4, enabled_planes: DWORD, equality: BOOL, centres: *const D3DVECTOR, radii: *const D3DVALUE, sphere_count: core::ffi::c_uint, return_values: *mut DWORD) {
    let mut mask: core::ffi::c_uint = 0;
    let mut i: core::ffi::c_uint = 0;
    let mut j: core::ffi::c_uint = 0;

    memset(return_values as *mut core::ffi::c_void, 0 as core::ffi::c_int, (sphere_count as usize).wrapping_mul(core::mem::size_of::<DWORD>()));
    i = 0 as core::ffi::c_uint;
    while i < sphere_count {
        mask = enabled_planes as core::ffi::c_uint;
        while mask != 0 {
            j = wined3d_bit_scan(core::ptr::addr_of_mut!(mask)) as core::ffi::c_uint;
            *return_values.offset(i as isize) |= in_plane(
                j as UINT,
                *planes.offset(j as isize),
                *centres.offset(i as isize),
                *radii.offset(i as isize),
                equality,
            );
        }
        i = i.wrapping_add(1);
    }
}
unsafe extern "system" fn d3d_device7_ComputeSphereVisibility(
    iface: *mut IDirect3DDevice7,
    centers: *mut D3DVECTOR,
    radii: *mut D3DVALUE,
    sphere_count: DWORD,
    flags: DWORD,
    return_values: *mut DWORD,
) -> HRESULT {
    let mut plane: [wined3d_vec4; 12] = core::mem::zeroed();
    let mut enabled_planes: DWORD = 0x3f;
    let mut user_clip_planes: DWORD = 0;
    let mut j: UINT;

    prepare_clip_space_planes(
        impl_from_IDirect3DDevice7(iface),
        core::ptr::addr_of_mut!(plane) as *mut wined3d_vec4,
    );

    IDirect3DDevice7_GetRenderState(
        iface,
        D3DRENDERSTATE_CLIPPLANEENABLE,
        core::ptr::addr_of_mut!(user_clip_planes),
    );
    enabled_planes |= user_clip_planes << 6;
    j = 6;
    while j < 12 {
        IDirect3DDevice7_GetClipPlane(
            iface,
            j - 6,
            (core::ptr::addr_of_mut!(plane) as *mut wined3d_vec4).offset(j as isize)
                as *mut wined3d_vec4 as *mut D3DVALUE,
        );
        j = j.wrapping_add(1);
    }

    compute_sphere_visibility(
        core::ptr::addr_of_mut!(plane) as *mut wined3d_vec4,
        enabled_planes,
        (FALSE) as BOOL,
        centers,
        radii,
        sphere_count,
        return_values,
    );
    return D3D_OK;
}

unsafe extern "system" fn d3d_device3_ComputeSphereVisibility(
    iface: *mut IDirect3DDevice3,
    centers: *mut D3DVECTOR,
    radii: *mut D3DVALUE,
    sphere_count: DWORD,
    flags: DWORD,
    return_values: *mut DWORD,
) -> HRESULT {
    static enabled_planes: DWORD = 0x3f;
    let mut plane: [wined3d_vec4; 6] = [wined3d_vec4 {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 0.,
    }; 6];
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;

    TRACE!(
        "iface %p, centers %p, radii %p, sphere_count %lu, flags %#lx, return_values %p.\n",
        iface,
        centers,
        radii,
        sphere_count,
        flags,
        return_values
    );

    prepare_clip_space_planes(
        impl_from_IDirect3DDevice3(iface),
        core::ptr::addr_of_mut!(plane) as *mut wined3d_vec4,
    );

    compute_sphere_visibility(
        core::ptr::addr_of_mut!(plane) as *mut wined3d_vec4,
        enabled_planes,
        (TRUE) as BOOL,
        centers,
        radii,
        sphere_count as ::core::ffi::c_uint,
        return_values,
    );
    i = 0;
    while i < sphere_count {
        let mut intersect_frustum: BOOL = (FALSE as i32);
        let mut outside_frustum: BOOL = (FALSE as i32);
        let mut d3d7_result: DWORD = *return_values.offset(i as isize);

        *return_values.offset(i as isize) = 0;

        j = 0;
        while j < 6 {
            let clip: DWORD = (d3d7_result >> j)
                & (D3DSTATUS_CLIPUNIONLEFT | D3DSTATUS_CLIPINTERSECTIONLEFT);

            if clip == D3DSTATUS_CLIPUNIONLEFT {
                *return_values.offset(i as isize) |=
                    (D3DVIS_INTERSECT_LEFT << (j * 2)) as DWORD;
                intersect_frustum = (TRUE as i32);
            } else if clip != 0 {
                *return_values.offset(i as isize) |=
                    (D3DVIS_OUTSIDE_LEFT << (j * 2)) as DWORD;
                outside_frustum = (TRUE as i32);
            }
            j = j.wrapping_add(1);
        }
        if outside_frustum != 0 {
            *return_values.offset(i as isize) |= D3DVIS_OUTSIDE_FRUSTUM as DWORD;
        } else if intersect_frustum != 0 {
            *return_values.offset(i as isize) |= D3DVIS_INTERSECT_FRUSTUM as DWORD;
        }
        i = i.wrapping_add(1);
    }
    return D3D_OK;
}

unsafe extern "C" fn d3d_device7_GetTexture(
    mut iface: *mut IDirect3DDevice7,
    mut stage: DWORD,
    mut texture: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut ddraw_texture: *mut ddraw_texture = core::ptr::null_mut::<ddraw_texture>();

    if texture.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if stage >= DDRAW_MAX_TEXTURES as DWORD {
        WARN!("Invalid stage %lu.\n", stage);
        *texture = core::ptr::null_mut::<IDirectDrawSurface7>();
        return D3D_OK;
    }

    wined3d_mutex_lock();
    wined3d_texture = (*(*device).stateblock_state).textures[stage as usize] as *mut wined3d_texture;
    if wined3d_texture.is_null() {
        *texture = core::ptr::null_mut::<IDirectDrawSurface7>();
        wined3d_mutex_unlock();
        return D3D_OK;
    }

    ddraw_texture = wined3d_texture_get_parent(wined3d_texture) as *mut ddraw_texture;
    *texture = core::ptr::addr_of_mut!((*(*ddraw_texture).root).IDirectDrawSurface7_iface);
    IDirectDrawSurface7_AddRef(*texture);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_GetTexture_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    Texture: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    d3d_device7_GetTexture(iface, stage, Texture)
}
unsafe extern "system" fn d3d_device7_GetTexture_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut stage: DWORD,
    mut Texture: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetTexture(iface, stage, Texture);
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_GetTexture(iface: *mut IDirect3DDevice3, stage: DWORD, Texture2: *mut *mut IDirect3DTexture2) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut ret_val_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut ret: HRESULT = 0;
    let mut ret_val: *mut IDirectDrawSurface7 = core::ptr::null_mut();

    ret = IDirect3DDevice7_GetTexture(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), stage, core::ptr::addr_of_mut!(ret_val));

    ret_val_impl = unsafe_impl_from_IDirectDrawSurface7(ret_val);
    *Texture2 = if !ret_val_impl.is_null() {
        core::ptr::addr_of_mut!((*ret_val_impl).IDirect3DTexture2_iface)
    } else {
        core::ptr::null_mut()
    };

    return ret;
}

unsafe extern "C" fn d3d_device7_SetTexture(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    texture: *mut IDirectDrawSurface7,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let surf: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(texture);
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    WINE_TRACE!("iface %p, stage %lu, texture %p.\n", iface, stage, texture);
    if !surf.is_null() && ((*surf).surface_desc.ddsCaps.dwCaps & (DDSCAPS_TEXTURE as DWORD)) != 0 {
        if !(*surf).draw_texture.is_null() {
            wined3d_texture = (*surf).draw_texture;
            (*device).have_draw_textures = TRUE as BOOL;
        } else {
            wined3d_texture = (*surf).wined3d_texture;
        }
    }
    wined3d_mutex_lock();
    wined3d_stateblock_set_texture((*device).update_state, stage, wined3d_texture);
    wined3d_mutex_unlock();
    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_SetTexture_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    texture: *mut IDirectDrawSurface7,
) -> HRESULT {
    d3d_device7_SetTexture(iface, stage, texture)
}
unsafe extern "system" fn d3d_device7_SetTexture_FPUPreserve(iface: *mut IDirect3DDevice7, stage: DWORD, texture: *mut IDirectDrawSurface7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetTexture(iface, stage, texture);
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "system" fn d3d_device3_SetTexture(
    mut iface: *mut IDirect3DDevice3,
    mut stage: DWORD,
    mut texture: *mut IDirect3DTexture2,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut tex: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture2(texture);
    let mut wined3d_texture: *mut wined3d_texture = ::core::ptr::null_mut::<wined3d_texture>();
    WINE_TRACE!("iface %p, stage %lu, texture %p.\n", iface, stage, texture);
    wined3d_mutex_lock();
    if !tex.is_null()
        && ((*tex).surface_desc.ddsCaps.dwCaps & DDSCAPS_TEXTURE as DWORD != 0
            || (*device).hardware_device == 0)
    {
        if !(*tex).draw_texture.is_null() {
            wined3d_texture = (*tex).draw_texture;
            (*device).have_draw_textures = TRUE as BOOL;
        } else {
            wined3d_texture = (*tex).wined3d_texture;
        }
    }
    wined3d_stateblock_set_texture((*device).state, stage as UINT, wined3d_texture);
    fixup_texture_alpha_op(device);
    wined3d_mutex_unlock();
    return D3D_OK;
}

unsafe extern "C" fn d3d_device7_GetTextureStageState(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut device_state: *const wined3d_stateblock_state = core::ptr::null();
    let mut l: *const tss_lookup_tag = core::ptr::null();

    if value.is_null() {
        return DDERR_INVALIDPARAMS as HRESULT;
    }

    if (state as u32) > (D3DTSS_TEXTURETRANSFORMFLAGS as u32) {
        WARN!("Invalid state %#x passed.\n", state as u32);
        return DD_OK as HRESULT;
    }

    if stage >= (DDRAW_MAX_TEXTURES as DWORD) {
        WARN!("Invalid stage %lu.\n", stage);
        *value = 0 as DWORD;
        return D3D_OK as HRESULT;
    }

    l = core::ptr::addr_of!(tss_lookup[state as usize]);

    wined3d_mutex_lock();

    device_state = (*device).stateblock_state;

    if (*l).sampler_state != 0 {
        *value = (*device_state).sampler_states[stage as usize][(*l).u.sampler_state as usize] as DWORD;

        if (state as u32) == (D3DTSS_MIPFILTER as u32) {
            /* Mipfilter is a sampler state with different values */
            if *value == (WINED3D_TEXF_NONE as DWORD) {
                *value = D3DTFP_NONE as DWORD;
            } else if *value == (WINED3D_TEXF_POINT as DWORD) {
                *value = D3DTFP_POINT as DWORD;
            } else if *value == (WINED3D_TEXF_LINEAR as DWORD) {
                *value = D3DTFP_LINEAR as DWORD;
            } else {
                ERR!("Unexpected mipfilter value %#lx.\n", *value);
                *value = D3DTFP_NONE as DWORD;
            }
        } else if (state as u32) == (D3DTSS_MAGFILTER as u32) {
            /* Magfilter has slightly different values */
            if *value == (WINED3D_TEXF_POINT as DWORD) {
                *value = D3DTFG_POINT as DWORD;
            } else if *value == (WINED3D_TEXF_LINEAR as DWORD) {
                *value = D3DTFG_LINEAR as DWORD;
            } else if *value == (WINED3D_TEXF_ANISOTROPIC as DWORD) {
                *value = D3DTFG_ANISOTROPIC as DWORD;
            } else if *value == (WINED3D_TEXF_FLAT_CUBIC as DWORD) {
                *value = D3DTFG_FLATCUBIC as DWORD;
            } else if *value == (WINED3D_TEXF_GAUSSIAN_CUBIC as DWORD) {
                *value = D3DTFG_GAUSSIANCUBIC as DWORD;
            } else {
                ERR!("Unexpected wined3d mag filter value %#lx.\n", *value);
                *value = D3DTFG_POINT as DWORD;
            }
        } else {
        }
    } else {
        *value = (*device_state).texture_states[stage as usize][(*l).u.texture_state as usize] as DWORD;
    }

    wined3d_mutex_unlock();

    return D3D_OK as HRESULT;
}
unsafe extern "system" fn d3d_device7_GetTextureStageState_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    d3d_device7_GetTextureStageState(iface, stage, state, value)
}

unsafe extern "system" fn d3d_device7_GetTextureStageState_FPUPreserve(iface: *mut IDirect3DDevice7, stage: DWORD, state: D3DTEXTURESTAGESTATETYPE, value: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetTextureStageState(iface, stage, state, value);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_GetTextureStageState(
    iface: *mut IDirect3DDevice3,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: *mut DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IDirect3DDevice7_GetTextureStageState(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface),
        stage,
        state,
        value,
    );
}

unsafe extern "C" fn d3d_device7_SetTextureStageState(
    mut iface: *mut IDirect3DDevice7,
    mut stage: DWORD,
    mut state: D3DTEXTURESTAGESTATETYPE,
    mut value: DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut l: *const tss_lookup_tag = core::ptr::null::<tss_lookup_tag>();

    TRACE!(
        "iface %p, stage %lu, state %#x, value %#lx.\n",
        iface,
        stage,
        state as u32,
        value
    );

    if state > D3DTSS_TEXTURETRANSFORMFLAGS {
        WARN!("Invalid state %#x passed.\n", state);
        return DD_OK;
    }

    l = (core::ptr::addr_of!(tss_lookup) as *const tss_lookup_tag).offset(state as isize) as *const tss_lookup_tag;

    wined3d_mutex_lock();

    if (*l).sampler_state != 0 {
        match state {
            D3DTSS_MIPFILTER => {
                match value {
                    D3DTFP_NONE => {
                        value = WINED3D_TEXF_NONE;
                    }
                    D3DTFP_POINT => {
                        value = WINED3D_TEXF_POINT;
                    }
                    0 | D3DTFP_LINEAR => {
                        value = WINED3D_TEXF_LINEAR;
                    }
                    _ => {
                        ERR!("Unexpected mipfilter value %#lx.\n", value);
                        value = WINED3D_TEXF_NONE;
                    }
                }
            }
            D3DTSS_MAGFILTER => {
                match value {
                    D3DTFG_POINT => {
                        value = WINED3D_TEXF_POINT;
                    }
                    D3DTFG_LINEAR => {
                        value = WINED3D_TEXF_LINEAR;
                    }
                    D3DTFG_FLATCUBIC => {
                        value = WINED3D_TEXF_FLAT_CUBIC;
                    }
                    D3DTFG_GAUSSIANCUBIC => {
                        value = WINED3D_TEXF_GAUSSIAN_CUBIC;
                    }
                    D3DTFG_ANISOTROPIC => {
                        value = WINED3D_TEXF_ANISOTROPIC;
                    }
                    _ => {
                        ERR!("Unexpected d3d7 mag filter value %#lx.\n", value);
                        value = WINED3D_TEXF_POINT;
                    }
                }
            }
            D3DTSS_ADDRESS => {
                wined3d_stateblock_set_sampler_state(
                    (*device).state,
                    stage as UINT,
                    WINED3D_SAMP_ADDRESS_V,
                    value as u32,
                );
            }
            _ => {}
        }

        wined3d_stateblock_set_sampler_state(
            (*device).state,
            stage as UINT,
            (*l).u.sampler_state,
            value as u32,
        );
    } else {
        wined3d_stateblock_set_texture_stage_state(
            (*device).update_state,
            stage as UINT,
            (*l).u.texture_state,
            value as u32,
        );
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_SetTextureStageState_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: DWORD,
) -> HRESULT {
    d3d_device7_SetTextureStageState(iface, stage, state, value)
}
unsafe extern "system" fn d3d_device7_SetTextureStageState_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    stage: DWORD,
    state: D3DTEXTURESTAGESTATETYPE,
    value: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetTextureStageState(iface, stage, state, value);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device3_SetTextureStageState(iface: *mut IDirect3DDevice3, stage: DWORD, state: D3DTEXTURESTAGESTATETYPE, value: DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);
    let mut old_value: DWORD = 0;
    let mut hr: HRESULT = 0;

    /* Tests show that legacy texture blending is not reset if the texture stage state
     * value is unchanged. */
    hr = IDirect3DDevice7_GetTextureStageState(&raw mut (*device).IDirect3DDevice7_iface,
                stage, state, &raw mut old_value);
    if hr < 0 {
        return hr;
    }

    if old_value == value
    {
        return D3D_OK;
    }

    (*device).legacyTextureBlending = (FALSE as i32);

    return IDirect3DDevice7_SetTextureStageState(&raw mut (*device).IDirect3DDevice7_iface, stage, state, value);
}
unsafe extern "C" fn d3d_device7_ValidateDevice(iface: *mut IDirect3DDevice7, pass_count: *mut DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_device_validate_device((*device).wined3d_device, (*device).stateblock_state, pass_count);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_ValidateDevice_FPUSetup(
    iface: *mut IDirect3DDevice7,
    pass_count: *mut DWORD,
) -> HRESULT {
    d3d_device7_ValidateDevice(iface, pass_count)
}
unsafe extern "system" fn d3d_device7_ValidateDevice_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut pass_count: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;
    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_ValidateDevice(iface, pass_count);
    set_fpu_control_word(old_fpucw);
    return hr;
}

unsafe extern "system" fn d3d_device3_ValidateDevice(iface: *mut IDirect3DDevice3, pass_count: *mut DWORD) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice3(iface);

    return IDirect3DDevice7_ValidateDevice(core::ptr::addr_of_mut!((*device).IDirect3DDevice7_iface), pass_count);
}

unsafe extern "C" fn d3d_device7_Clear(
    mut iface: *mut IDirect3DDevice7,
    mut count: DWORD,
    mut rects: *mut D3DRECT,
    mut flags: DWORD,
    mut color: D3DCOLOR,
    mut z: D3DVALUE,
    mut stencil: DWORD,
) -> HRESULT {
    let c: wined3d_color = wined3d_color {
        r: (((color >> 16) & 0xff) as f32) / 255.0f32,
        g: (((color >> 8) & 0xff) as f32) / 255.0f32,
        b: ((color & 0xff) as f32) / 255.0f32,
        a: (((color >> 24) & 0xff) as f32) / 255.0f32,
    };
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    if count != 0 && rects.is_null() {
        WARN!("count %lu with NULL rects.\n", count);
        count = 0;
    }

    wined3d_mutex_lock();
    d3d_device_apply_state(device, (TRUE) as BOOL);
    hr = wined3d_device_clear(
        (*device).wined3d_device,
        count,
        rects as *mut RECT,
        flags,
        core::ptr::addr_of!(c),
        z,
        stencil,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_Clear_FPUSetup(
    iface: *mut IDirect3DDevice7,
    count: DWORD,
    rects: *mut D3DRECT,
    flags: DWORD,
    color: D3DCOLOR,
    z: D3DVALUE,
    stencil: DWORD,
) -> HRESULT {
    d3d_device7_Clear(iface, count, rects, flags, color, z, stencil)
}

unsafe extern "system" fn d3d_device7_Clear_FPUPreserve(iface: *mut IDirect3DDevice7, count: DWORD, rects: *mut D3DRECT, flags: DWORD, color: D3DCOLOR, z: D3DVALUE, stencil: DWORD) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut old_fpucw: WORD = 0;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_Clear(iface, count, rects, flags, color, z, stencil);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_SetViewport(
    iface: *mut IDirect3DDevice7,
    viewport: *mut D3DVIEWPORT7,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut rt_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut surface: *mut ddraw_surface = core::ptr::null_mut();
    let mut vp: wined3d_viewport = core::mem::zeroed();

    if viewport.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    rtv = if !(*device).target.is_null() {
        ddraw_surface_get_rendertarget_view((*device).target)
    } else {
        core::ptr::null_mut()
    };
    if rtv.is_null() {
        wined3d_mutex_unlock();
        return DDERR_INVALIDCAPS;
    }
    surface = wined3d_rendertarget_view_get_sub_resource_parent(rtv) as *mut ddraw_surface;
    wined3d_texture_get_sub_resource_desc(
        (*surface).wined3d_texture,
        (*surface).sub_resource_idx,
        core::ptr::addr_of_mut!(rt_desc),
    );

    if !wined3d_bound_range(
        (*viewport).dwX as core::ffi::c_uint,
        (*viewport).dwWidth as core::ffi::c_uint,
        rt_desc.width,
    ) || !wined3d_bound_range(
        (*viewport).dwY as core::ffi::c_uint,
        (*viewport).dwHeight as core::ffi::c_uint,
        rt_desc.height,
    ) {
        WARN!("Invalid viewport, returning E_INVALIDARG.\n");
        wined3d_mutex_unlock();
        return E_INVALIDARG;
    }

    vp.x = (*viewport).dwX as f32;
    vp.y = (*viewport).dwY as f32;
    vp.width = (*viewport).dwWidth as f32;
    vp.height = (*viewport).dwHeight as f32;
    vp.min_z = (*viewport).dvMinZ as f32;
    vp.max_z = (*viewport).dvMaxZ as f32;

    wined3d_stateblock_set_viewport((*device).update_state, core::ptr::addr_of_mut!(vp));
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_SetViewport_FPUSetup(
    iface: *mut IDirect3DDevice7,
    viewport: *mut D3DVIEWPORT7,
) -> HRESULT {
    d3d_device7_SetViewport(iface, viewport)
}

unsafe extern "system" fn d3d_device7_SetViewport_FPUPreserve(iface: *mut IDirect3DDevice7, viewport: *mut D3DVIEWPORT7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetViewport(iface, viewport);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_GetViewport(iface: *mut IDirect3DDevice7, viewport: *mut D3DVIEWPORT7) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_viewport: wined3d_viewport = core::mem::zeroed();

    if viewport.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    wined3d_viewport = (*(*device).stateblock_state).viewport;
    wined3d_mutex_unlock();

    (*viewport).dwX = wined3d_viewport.x as DWORD;
    (*viewport).dwY = wined3d_viewport.y as DWORD;
    (*viewport).dwWidth = wined3d_viewport.width as DWORD;
    (*viewport).dwHeight = wined3d_viewport.height as DWORD;
    (*viewport).dvMinZ = wined3d_viewport.min_z as D3DVALUE;
    (*viewport).dvMaxZ = wined3d_viewport.max_z as D3DVALUE;

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_GetViewport_FPUSetup(
    iface: *mut IDirect3DDevice7,
    viewport: *mut D3DVIEWPORT7,
) -> HRESULT {
    d3d_device7_GetViewport(iface, viewport)
}
unsafe extern "system" fn d3d_device7_GetViewport_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut viewport: *mut D3DVIEWPORT7,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetViewport(iface, viewport);
    set_fpu_control_word(old_fpucw);

    return hr;
}
unsafe extern "C" fn d3d_device7_SetMaterial(iface: *mut IDirect3DDevice7, material: *mut D3DMATERIAL7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    TRACE!("iface {:p}, material {:p}.\n", iface, material);

    if material.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    /* Note: D3DMATERIAL7 is compatible with struct wined3d_material. */
    wined3d_stateblock_set_material((*device).update_state, material as *const wined3d_material);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_SetMaterial_FPUSetup(
    iface: *mut IDirect3DDevice7,
    material: *mut D3DMATERIAL7,
) -> HRESULT {
    d3d_device7_SetMaterial(iface, material)
}
unsafe extern "system" fn d3d_device7_SetMaterial_FPUPreserve(iface: *mut IDirect3DDevice7, material: *mut D3DMATERIAL7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetMaterial(iface, material);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_GetMaterial(iface: *mut IDirect3DDevice7, material: *mut D3DMATERIAL7) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    TRACE!("iface %p, material %p.\n", iface, material);

    wined3d_mutex_lock();
    /* Note: D3DMATERIAL7 is compatible with struct wined3d_material. */
    memcpy(
        material as *mut core::ffi::c_void,
        core::ptr::addr_of!((*(*device).stateblock_state).material) as *const core::ffi::c_void,
        core::mem::size_of::<D3DMATERIAL7>() as usize,
    );
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_GetMaterial_FPUSetup(
    iface: *mut IDirect3DDevice7,
    material: *mut D3DMATERIAL7,
) -> HRESULT {
    d3d_device7_GetMaterial(iface, material)
}
unsafe extern "system" fn d3d_device7_GetMaterial_FPUPreserve(iface: *mut IDirect3DDevice7, material: *mut D3DMATERIAL7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetMaterial(iface, material);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_SetLight(iface: *mut IDirect3DDevice7, light_idx: DWORD, light: *mut D3DLIGHT7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_light((*device).update_state, light_idx, light as *const wined3d_light);
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_SetLight_FPUSetup(
    iface: *mut IDirect3DDevice7,
    light_idx: DWORD,
    light: *mut D3DLIGHT7,
) -> HRESULT {
    d3d_device7_SetLight(iface, light_idx, light)
}
unsafe extern "system" fn d3d_device7_SetLight_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut light_idx: DWORD,
    mut light: *mut D3DLIGHT7,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetLight(iface, light_idx, light);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_GetLight(iface: *mut IDirect3DDevice7, light_idx: DWORD, light: *mut D3DLIGHT7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut enabled: BOOL = 0;
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_light((*device).state, light_idx as UINT, light as *mut wined3d_light, core::ptr::addr_of_mut!(enabled));
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_GetLight_FPUSetup(
    iface: *mut IDirect3DDevice7,
    light_idx: DWORD,
    light: *mut D3DLIGHT7,
) -> HRESULT {
    d3d_device7_GetLight(iface, light_idx, light)
}

unsafe extern "system" fn d3d_device7_GetLight_FPUPreserve(iface: *mut IDirect3DDevice7, light_idx: DWORD, light: *mut D3DLIGHT7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetLight(iface, light_idx, light);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_BeginStateBlock(iface: *mut IDirect3DDevice7) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut stateblock: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!("iface %p.\n", iface);

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to begin a stateblock while recording, returning D3DERR_INBEGINSTATEBLOCK.\n");
        return D3DERR_INBEGINSTATEBLOCK;
    }
    hr = wined3d_stateblock_create(
        (*device).wined3d_device,
        core::ptr::null(),
        WINED3D_SBT_RECORDED,
        &mut stateblock,
    );
    if hr >= 0 {
        (*device).recording = stateblock;
        (*device).update_state = stateblock;
    }
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_BeginStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
) -> HRESULT {
    d3d_device7_BeginStateBlock(iface)
}
unsafe extern "system" fn d3d_device7_BeginStateBlock_FPUPreserve(iface: *mut IDirect3DDevice7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_BeginStateBlock(iface);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_EndStateBlock(iface: *mut IDirect3DDevice7, stateblock: *mut DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_sb: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut h: DWORD = 0;

    if stateblock.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if (*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to end a stateblock, but no stateblock is being recorded.\n");
        return D3DERR_NOTINBEGINSTATEBLOCK;
    }
    wined3d_sb = (*device).recording;
    wined3d_stateblock_init_contained_states(wined3d_sb);
    (*device).recording = core::ptr::null_mut::<wined3d_stateblock>();
    (*device).update_state = (*device).state;

    h = ddraw_allocate_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        wined3d_sb as *mut core::ffi::c_void,
        DDRAW_HANDLE_STATEBLOCK,
    );
    if h == (DDRAW_INVALID_HANDLE) as u32 {
        ERR!("Failed to allocate a stateblock handle.\n");
        wined3d_stateblock_decref(wined3d_sb);
        wined3d_mutex_unlock();
        *stateblock = 0;
        return DDERR_OUTOFMEMORY;
    }

    wined3d_mutex_unlock();
    *stateblock = h.wrapping_add(1);

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_EndStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stateblock: *mut DWORD,
) -> HRESULT {
    d3d_device7_EndStateBlock(iface, stateblock)
}
unsafe extern "system" fn d3d_device7_EndStateBlock_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    stateblock: *mut DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_EndStateBlock(iface, stateblock);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_PreLoad(
    mut iface: *mut IDirect3DDevice7,
    mut texture: *mut IDirectDrawSurface7,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(texture);
    WINE_TRACE!("iface %p, texture %p.\n", iface, texture);
    if texture.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    wined3d_mutex_lock();
    wined3d_resource_preload(wined3d_texture_get_resource(
        if !(*surface).draw_texture.is_null() {
            (*surface).draw_texture
        } else {
            (*surface).wined3d_texture
        },
    ));
    wined3d_mutex_unlock();
    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_PreLoad_FPUSetup(
    iface: *mut IDirect3DDevice7,
    texture: *mut IDirectDrawSurface7,
) -> HRESULT {
    d3d_device7_PreLoad(iface, texture)
}
unsafe extern "system" fn d3d_device7_PreLoad_FPUPreserve(iface: *mut IDirect3DDevice7, texture: *mut IDirectDrawSurface7) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_PreLoad(iface, texture);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_ApplyStateBlock(iface: *mut IDirect3DDevice7, stateblock: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_sb: *mut wined3d_stateblock = core::ptr::null_mut();

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to apply a stateblock while recording, returning D3DERR_INBEGINSTATEBLOCK.\n");
        return D3DERR_INBEGINSTATEBLOCK;
    }
    wined3d_sb = ddraw_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        stateblock.wrapping_sub(1),
        DDRAW_HANDLE_STATEBLOCK,
    ) as *mut wined3d_stateblock;
    if wined3d_sb.is_null() {
        WARN!("Invalid stateblock handle.\n");
        wined3d_mutex_unlock();
        return D3DERR_INVALIDSTATEBLOCK;
    }

    wined3d_stateblock_apply(wined3d_sb, (*device).state);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_ApplyStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stateblock: DWORD,
) -> HRESULT {
    d3d_device7_ApplyStateBlock(iface, stateblock)
}
unsafe extern "system" fn d3d_device7_ApplyStateBlock_FPUPreserve(mut iface: *mut IDirect3DDevice7, mut stateblock: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_ApplyStateBlock(iface, stateblock);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_CaptureStateBlock(iface: *mut IDirect3DDevice7, stateblock: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_sb: *mut wined3d_stateblock;

    wined3d_mutex_lock();
    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to capture a stateblock while recording, returning D3DERR_INBEGINSTATEBLOCK.\n");
        return D3DERR_INBEGINSTATEBLOCK;
    }
    wined3d_sb = ddraw_get_object(
        core::ptr::addr_of_mut!((*device).handle_table),
        stateblock.wrapping_sub(1),
        DDRAW_HANDLE_STATEBLOCK,
    ) as *mut wined3d_stateblock;
    if wined3d_sb.is_null() {
        WARN!("Invalid stateblock handle.\n");
        wined3d_mutex_unlock();
        return D3DERR_INVALIDSTATEBLOCK;
    }

    wined3d_stateblock_capture(wined3d_sb, (*device).state);
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_CaptureStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stateblock: DWORD,
) -> HRESULT {
    d3d_device7_CaptureStateBlock(iface, stateblock)
}
unsafe extern "system" fn d3d_device7_CaptureStateBlock_FPUPreserve(iface: *mut IDirect3DDevice7, stateblock: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_CaptureStateBlock(iface, stateblock);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_DeleteStateBlock(iface: *mut IDirect3DDevice7, stateblock: DWORD) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_sb: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut ref_0: ULONG = 0;

    wined3d_mutex_lock();

    wined3d_sb = ddraw_free_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        stateblock.wrapping_sub(1),
        DDRAW_HANDLE_STATEBLOCK,
    ) as *mut wined3d_stateblock;
    if wined3d_sb.is_null() {
        WARN!("Invalid stateblock handle.\n");
        wined3d_mutex_unlock();
        return D3DERR_INVALIDSTATEBLOCK;
    }

    ref_0 = wined3d_stateblock_decref(wined3d_sb);
    if ref_0 != 0 {
        ERR!("Something is still holding stateblock %p (refcount %lu).\n", wined3d_sb, ref_0);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_DeleteStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
    stateblock: DWORD,
) -> HRESULT {
    d3d_device7_DeleteStateBlock(iface, stateblock)
}
unsafe extern "system" fn d3d_device7_DeleteStateBlock_FPUPreserve(iface: *mut IDirect3DDevice7, stateblock: DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_DeleteStateBlock(iface, stateblock);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_CreateStateBlock(
    iface: *mut IDirect3DDevice7,
    type_: D3DSTATEBLOCKTYPE,
    stateblock: *mut DWORD,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_sb: *mut wined3d_stateblock = core::ptr::null_mut();
    let mut hr: HRESULT = 0;
    let mut h: DWORD = 0;

    if stateblock.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if type_ != D3DSBT_ALL
        && type_ != D3DSBT_PIXELSTATE
        && type_ != D3DSBT_VERTEXSTATE
    {
        WARN!("Unexpected stateblock type, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    if !(*device).recording.is_null() {
        wined3d_mutex_unlock();
        WARN!("Trying to apply a stateblock while recording, returning D3DERR_INBEGINSTATEBLOCK.\n");
        return D3DERR_INBEGINSTATEBLOCK;
    }

    hr = wined3d_stateblock_create(
        (*device).wined3d_device,
        (*device).state,
        wined3d_stateblock_type_from_ddraw(type_),
        core::ptr::addr_of_mut!(wined3d_sb),
    );
    if hr < 0 {
        WARN!("Failed to create stateblock, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr_ddraw_from_wined3d(hr);
    }

    h = ddraw_allocate_handle(
        core::ptr::addr_of_mut!((*device).handle_table),
        wined3d_sb as *mut core::ffi::c_void,
        DDRAW_HANDLE_STATEBLOCK,
    );
    if h == (DDRAW_INVALID_HANDLE) as u32 {
        ERR!("Failed to allocate stateblock handle.\n");
        wined3d_stateblock_decref(wined3d_sb);
        wined3d_mutex_unlock();
        return DDERR_OUTOFMEMORY;
    }

    *stateblock = h + 1;
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_CreateStateBlock_FPUSetup(
    iface: *mut IDirect3DDevice7,
    r#type: D3DSTATEBLOCKTYPE,
    stateblock: *mut DWORD,
) -> HRESULT {
    d3d_device7_CreateStateBlock(iface, r#type, stateblock)
}
unsafe extern "system" fn d3d_device7_CreateStateBlock_FPUPreserve(iface: *mut IDirect3DDevice7, type_: D3DSTATEBLOCKTYPE, stateblock: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_CreateStateBlock(iface, type_, stateblock);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn is_mip_level_subset(
    mut dest: *mut ddraw_surface,
    mut src: *mut ddraw_surface,
) -> BOOL {
    let mut src_level: *mut ddraw_surface = core::ptr::null_mut();
    let mut dest_level: *mut ddraw_surface = core::ptr::null_mut();
    let mut temp: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut ddsd: DDSURFACEDESC2 = core::mem::zeroed();
    let mut levelFound: BOOL = 0;

    levelFound = (FALSE as i32);

    src_level = src;
    dest_level = dest;

    while !src_level.is_null() && !dest_level.is_null() {
        if (*src_level).surface_desc.dwWidth == (*dest_level).surface_desc.dwWidth
            && (*src_level).surface_desc.dwHeight == (*dest_level).surface_desc.dwHeight
        {
            levelFound = (TRUE as i32);

            ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
            ddsd.ddsCaps.dwCaps2 = DDSCAPS2_MIPMAPSUBLEVEL;
            IDirectDrawSurface7_GetAttachedSurface(
                &raw mut (*dest_level).IDirectDrawSurface7_iface,
                &raw mut ddsd.ddsCaps,
                &raw mut temp,
            );

            if dest_level != dest {
                IDirectDrawSurface7_Release(&raw mut (*dest_level).IDirectDrawSurface7_iface);
            }

            dest_level = unsafe_impl_from_IDirectDrawSurface7(temp);
        }

        ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
        ddsd.ddsCaps.dwCaps2 = DDSCAPS2_MIPMAPSUBLEVEL;
        IDirectDrawSurface7_GetAttachedSurface(
            &raw mut (*src_level).IDirectDrawSurface7_iface,
            &raw mut ddsd.ddsCaps,
            &raw mut temp,
        );

        if src_level != src {
            IDirectDrawSurface7_Release(&raw mut (*src_level).IDirectDrawSurface7_iface);
        }

        src_level = unsafe_impl_from_IDirectDrawSurface7(temp);
    }

    if !src_level.is_null() && src_level != src {
        IDirectDrawSurface7_Release(&raw mut (*src_level).IDirectDrawSurface7_iface);
    }
    if !dest_level.is_null() && dest_level != dest {
        IDirectDrawSurface7_Release(&raw mut (*dest_level).IDirectDrawSurface7_iface);
    }

    return (dest_level.is_null() && levelFound != 0) as ::core::ffi::c_int;
}

unsafe extern "C" fn copy_mipmap_chain(
    device: *mut d3d_device,
    dst: *mut ddraw_surface,
    src: *mut ddraw_surface,
    DestPoint: *const POINT,
    SrcRect: *const RECT,
) {
    let mut dst_level: *mut ddraw_surface = core::ptr::null_mut();
    let mut src_level: *mut ddraw_surface = core::ptr::null_mut();
    let mut temp: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut ddsd: DDSURFACEDESC2 = core::mem::zeroed();
    let mut point: POINT = core::mem::zeroed();
    let mut src_rect: RECT = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut pal: *mut IDirectDrawPalette = core::ptr::null_mut();
    let mut pal_src: *mut IDirectDrawPalette = core::ptr::null_mut();
    let mut ckeyflag: DWORD = 0;
    let mut ddckey: DDCOLORKEY = core::mem::zeroed();

    /* Copy palette, if possible. */
    IDirectDrawSurface7_GetPalette(&raw mut (*src).IDirectDrawSurface7_iface, &raw mut pal_src);
    IDirectDrawSurface7_GetPalette(&raw mut (*dst).IDirectDrawSurface7_iface, &raw mut pal);

    if !pal_src.is_null() && !pal.is_null() {
        let mut palent: [PALETTEENTRY; 256] = core::mem::zeroed();

        IDirectDrawPalette_GetEntries(
            pal_src,
            0,
            0,
            256,
            &raw mut palent as LPPALETTEENTRY,
        );
        IDirectDrawPalette_SetEntries(
            pal,
            0,
            0,
            256,
            &raw mut palent as LPPALETTEENTRY,
        );
    }

    if !pal.is_null() {
        IDirectDrawPalette_Release(pal);
    }
    if !pal_src.is_null() {
        IDirectDrawPalette_Release(pal_src);
    }

    /* Copy colorkeys, if present. */
    ckeyflag = DDCKEY_DESTBLT;
    while ckeyflag <= DDCKEY_SRCOVERLAY {
        hr = IDirectDrawSurface7_GetColorKey(
            &raw mut (*src).IDirectDrawSurface7_iface,
            ckeyflag,
            &raw mut ddckey,
        );

        if hr >= 0 {
            IDirectDrawSurface7_SetColorKey(
                &raw mut (*dst).IDirectDrawSurface7_iface,
                ckeyflag,
                &raw mut ddckey,
            );
        }
        ckeyflag <<= 1;
    }

    src_level = src;
    dst_level = dst;

    point = *DestPoint;
    src_rect = *SrcRect;

    while !src_level.is_null() && !dst_level.is_null() {
        if (*src_level).surface_desc.dwWidth == (*dst_level).surface_desc.dwWidth
            && (*src_level).surface_desc.dwHeight == (*dst_level).surface_desc.dwHeight
        {
            let src_w: UINT = (src_rect.right - src_rect.left) as UINT;
            let src_h: UINT = (src_rect.bottom - src_rect.top) as UINT;
            let mut dst_rect: RECT = RECT {
                left: point.x,
                top: point.y,
                right: (point.x as UINT).wrapping_add(src_w) as LONG,
                bottom: (point.y as UINT).wrapping_add(src_h) as LONG,
            };

            hr = wined3d_device_context_blt(
                (*device).immediate_context,
                ddraw_surface_get_any_texture(dst_level, DDRAW_SURFACE_RW),
                (*dst_level).sub_resource_idx,
                &raw mut dst_rect,
                ddraw_surface_get_any_texture(src_level, DDRAW_SURFACE_READ),
                (*src_level).sub_resource_idx,
                &raw mut src_rect,
                0,
                core::ptr::null::<wined3d_blt_fx>(),
                WINED3D_TEXF_POINT,
            );
            if hr < 0 {
                ERR!("Blit failed, hr %#lx.\n", hr);
            }

            ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
            ddsd.ddsCaps.dwCaps2 = DDSCAPS2_MIPMAPSUBLEVEL;
            IDirectDrawSurface7_GetAttachedSurface(
                &raw mut (*dst_level).IDirectDrawSurface7_iface,
                &raw mut ddsd.ddsCaps,
                &raw mut temp,
            );

            if dst_level != dst {
                IDirectDrawSurface7_Release(&raw mut (*dst_level).IDirectDrawSurface7_iface);
            }

            dst_level = unsafe_impl_from_IDirectDrawSurface7(temp);
        }

        ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
        ddsd.ddsCaps.dwCaps2 = DDSCAPS2_MIPMAPSUBLEVEL;
        IDirectDrawSurface7_GetAttachedSurface(
            &raw mut (*src_level).IDirectDrawSurface7_iface,
            &raw mut ddsd.ddsCaps,
            &raw mut temp,
        );

        if src_level != src {
            IDirectDrawSurface7_Release(&raw mut (*src_level).IDirectDrawSurface7_iface);
        }

        src_level = unsafe_impl_from_IDirectDrawSurface7(temp);

        point.x /= 2;
        point.y /= 2;

        src_rect.top /= 2;
        src_rect.left /= 2;
        src_rect.right = ((src_rect.right + 1) / 2) as LONG;
        src_rect.bottom = ((src_rect.bottom + 1) / 2) as LONG;
    }

    if !src_level.is_null() && src_level != src {
        IDirectDrawSurface7_Release(&raw mut (*src_level).IDirectDrawSurface7_iface);
    }
    if !dst_level.is_null() && dst_level != dst {
        IDirectDrawSurface7_Release(&raw mut (*dst_level).IDirectDrawSurface7_iface);
    }
}

unsafe extern "C" fn d3d_device7_Load(
    iface: *mut IDirect3DDevice7,
    dst_texture: *mut IDirectDrawSurface7,
    dst_pos: *mut POINT,
    src_texture: *mut IDirectDrawSurface7,
    src_rect: *mut RECT,
    flags: DWORD,
) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let dest: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(dst_texture);
    let src: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(src_texture);
    let mut destpoint: POINT = POINT { x: 0, y: 0 };
    let mut srcrect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    if src.is_null() || dest.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    if src_rect.is_null() {
        SetRect(
            &raw mut srcrect,
            0 as i32,
            0 as i32,
            (*src).surface_desc.dwWidth as i32,
            (*src).surface_desc.dwHeight as i32,
        );
    } else {
        srcrect = *src_rect;
    }

    if dst_pos.is_null() {
        destpoint.x = 0;
        destpoint.y = 0;
    } else {
        destpoint = *dst_pos;
    }

    /* Check bad dimensions. dst_pos is validated against src, not dest, because
     * destination can be a subset of mip levels, in which case actual coordinates used
     * for it may be divided. If any dimension of dest is larger than source, it can't be
     * mip level subset, so an error can be returned early.
     */
    if IsRectEmpty(&raw mut srcrect as LPCRECT) != 0
        || srcrect.right as DWORD > (*src).surface_desc.dwWidth
        || srcrect.bottom as DWORD > (*src).surface_desc.dwHeight
        || (destpoint.x + srcrect.right - srcrect.left) as DWORD > (*src).surface_desc.dwWidth
        || (destpoint.y + srcrect.bottom - srcrect.top) as DWORD > (*src).surface_desc.dwHeight
        || (*dest).surface_desc.dwWidth > (*src).surface_desc.dwWidth
        || (*dest).surface_desc.dwHeight > (*src).surface_desc.dwHeight
    {
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    /* Must be top level surfaces. */
    if ((*src).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_MIPMAPSUBLEVEL) != 0
        || ((*dest).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_MIPMAPSUBLEVEL) != 0
    {
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if ((*src).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP) != 0 {
        let mut src_face: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
        let mut dest_face: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
        let mut src_face_flag: DWORD = 0;
        let mut dest_face_flag: DWORD = 0;
        let mut temp: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
        let mut ddsd: DDSURFACEDESC2 = core::mem::zeroed();
        let mut i: i32;

        if ((*dest).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP) == 0 {
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }

        /* Iterate through cube faces 2 times. First time is just to check INVALIDPARAMS conditions, second
         * time it's actual surface loading. */
        i = 0;
        while i < 2 {
            dest_face = dest;
            src_face = src;

            while !dest_face.is_null() && !src_face.is_null() {
                src_face_flag = (*src_face).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP_ALLFACES;
                dest_face_flag = (*dest_face).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP_ALLFACES;

                if src_face_flag == dest_face_flag {
                    if i == 0 {
                        /* Destination mip levels must be subset of source mip levels. */
                        if is_mip_level_subset(dest_face, src_face) == 0 {
                            wined3d_mutex_unlock();
                            return DDERR_INVALIDPARAMS;
                        }
                    } else if (flags & dest_face_flag) != 0 {
                        copy_mipmap_chain(
                            device,
                            dest_face,
                            src_face,
                            &raw mut destpoint,
                            &raw mut srcrect,
                        );
                    }

                    if src_face_flag < DDSCAPS2_CUBEMAP_NEGATIVEZ {
                        ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
                        ddsd.ddsCaps.dwCaps2 =
                            DDSCAPS2_CUBEMAP | (src_face_flag << 1);
                        IDirectDrawSurface7_GetAttachedSurface(
                            &raw mut (*src).IDirectDrawSurface7_iface,
                            &raw mut ddsd.ddsCaps,
                            &raw mut temp,
                        );

                        if src_face != src {
                            IDirectDrawSurface7_Release(
                                &raw mut (*src_face).IDirectDrawSurface7_iface,
                            );
                        }

                        src_face = unsafe_impl_from_IDirectDrawSurface7(temp);
                    } else {
                        if src_face != src {
                            IDirectDrawSurface7_Release(
                                &raw mut (*src_face).IDirectDrawSurface7_iface,
                            );
                        }

                        src_face = core::ptr::null_mut::<ddraw_surface>();
                    }
                }

                if dest_face_flag < DDSCAPS2_CUBEMAP_NEGATIVEZ {
                    ddsd.ddsCaps.dwCaps = DDSCAPS_TEXTURE;
                    ddsd.ddsCaps.dwCaps2 =
                        DDSCAPS2_CUBEMAP | (dest_face_flag << 1);
                    IDirectDrawSurface7_GetAttachedSurface(
                        &raw mut (*dest).IDirectDrawSurface7_iface,
                        &raw mut ddsd.ddsCaps,
                        &raw mut temp,
                    );

                    if dest_face != dest {
                        IDirectDrawSurface7_Release(
                            &raw mut (*dest_face).IDirectDrawSurface7_iface,
                        );
                    }

                    dest_face = unsafe_impl_from_IDirectDrawSurface7(temp);
                } else {
                    if dest_face != dest {
                        IDirectDrawSurface7_Release(
                            &raw mut (*dest_face).IDirectDrawSurface7_iface,
                        );
                    }

                    dest_face = core::ptr::null_mut::<ddraw_surface>();
                }
            }

            if i == 0 {
                /* Native returns error if src faces are not subset of dest faces. */
                if !src_face.is_null() {
                    wined3d_mutex_unlock();
                    return DDERR_INVALIDPARAMS;
                }
            }

            i += 1;
        }

        wined3d_mutex_unlock();
        return D3D_OK;
    } else if ((*dest).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP) != 0 {
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    /* Handle non cube map textures. */

    /* Destination mip levels must be subset of source mip levels. */
    if is_mip_level_subset(dest, src) == 0 {
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    copy_mipmap_chain(device, dest, src, &raw mut destpoint, &raw mut srcrect);

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_Load_FPUSetup(
    iface: *mut IDirect3DDevice7,
    dst_texture: *mut IDirectDrawSurface7,
    dst_pos: *mut POINT,
    src_texture: *mut IDirectDrawSurface7,
    src_rect: *mut RECT,
    flags: DWORD,
) -> HRESULT {
    d3d_device7_Load(iface, dst_texture, dst_pos, src_texture, src_rect, flags)
}

unsafe extern "system" fn d3d_device7_Load_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    dst_texture: *mut IDirectDrawSurface7,
    dst_pos: *mut POINT,
    src_texture: *mut IDirectDrawSurface7,
    src_rect: *mut RECT,
    flags: DWORD,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_Load(iface, dst_texture, dst_pos, src_texture, src_rect, flags);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_LightEnable(
    iface: *mut IDirect3DDevice7,
    light_idx: DWORD,
    enabled: BOOL,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, light_idx %lu, enabled %#x.\n", iface, light_idx, enabled);
    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_light_enable((*device).update_state, light_idx as UINT, enabled);
    wined3d_mutex_unlock();
    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_LightEnable_FPUSetup(
    iface: *mut IDirect3DDevice7,
    light_idx: DWORD,
    enabled: BOOL,
) -> HRESULT {
    d3d_device7_LightEnable(iface, light_idx, enabled)
}
unsafe extern "system" fn d3d_device7_LightEnable_FPUPreserve(mut iface: *mut IDirect3DDevice7, mut light_idx: DWORD, mut enabled: BOOL) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_LightEnable(iface, light_idx, enabled);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_GetLightEnable(
    mut iface: *mut IDirect3DDevice7,
    mut light_idx: DWORD,
    mut enabled: *mut BOOL,
) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut light: wined3d_light = core::mem::zeroed();
    let mut hr: HRESULT;

    if enabled.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    WINE_TRACE!("iface %p, light_idx %lu, enabled %p.\n", iface, light_idx, enabled);

    wined3d_mutex_lock();
    hr = wined3d_stateblock_get_light((*device).state, light_idx as UINT, &raw mut light, enabled);
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn d3d_device7_GetLightEnable_FPUSetup(
    iface: *mut IDirect3DDevice7,
    light_idx: DWORD,
    enabled: *mut BOOL,
) -> HRESULT {
    d3d_device7_GetLightEnable(iface, light_idx, enabled)
}
unsafe extern "system" fn d3d_device7_GetLightEnable_FPUPreserve(
    mut iface: *mut IDirect3DDevice7,
    mut light_idx: DWORD,
    mut enabled: *mut BOOL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetLightEnable(iface, light_idx, enabled);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_SetClipPlane(iface: *mut IDirect3DDevice7, idx: DWORD, plane: *mut D3DVALUE) -> HRESULT {
    let mut device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);
    let mut wined3d_plane: *const wined3d_vec4;
    let mut hr: HRESULT;

    TRACE!("iface %p, idx %lu, plane %p.\n", iface, idx, plane);

    if plane.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_plane = plane as *const wined3d_vec4;

    wined3d_mutex_lock();
    hr = wined3d_stateblock_set_clip_plane((*device).update_state, idx, wined3d_plane);
    if (idx as usize) < ARRAY_SIZE!((*device).user_clip_planes) as usize {
        (*device).user_clip_planes[idx as usize] = *wined3d_plane;
        if hr == WINED3DERR_INVALIDCALL {
            WARN!("Clip plane %lu is not supported.\n", idx);
            hr = D3D_OK;
        }
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn d3d_device7_SetClipPlane_FPUSetup(
    iface: *mut IDirect3DDevice7,
    idx: DWORD,
    plane: *mut D3DVALUE,
) -> HRESULT {
    d3d_device7_SetClipPlane(iface, idx, plane)
}
unsafe extern "system" fn d3d_device7_SetClipPlane_FPUPreserve(iface: *mut IDirect3DDevice7, idx: DWORD, plane: *mut D3DVALUE) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_SetClipPlane(iface, idx, plane);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "C" fn d3d_device7_GetClipPlane(iface: *mut IDirect3DDevice7, idx: DWORD, plane: *mut D3DVALUE) -> HRESULT {
    let device: *mut d3d_device = impl_from_IDirect3DDevice7(iface);

    if plane.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if idx < WINED3D_MAX_CLIP_DISTANCES {
        memcpy(plane as *mut core::ffi::c_void, (&raw const (*(*device).stateblock_state).clip_planes as *const wined3d_vec4).offset(idx as isize) as *const wined3d_vec4 as *const core::ffi::c_void, core::mem::size_of::<wined3d_vec4>() as size_t);
    } else {
        WARN!("Clip plane %lu is not supported.\n", idx);
        if (idx as usize) < ARRAY_SIZE!((*device).user_clip_planes) as usize {
            memcpy(plane as *mut core::ffi::c_void, (&raw mut (*device).user_clip_planes as *mut wined3d_vec4).offset(idx as isize) as *mut wined3d_vec4 as *const core::ffi::c_void, core::mem::size_of::<wined3d_vec4>() as size_t);
        }
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_device7_GetClipPlane_FPUSetup(
    iface: *mut IDirect3DDevice7,
    idx: DWORD,
    plane: *mut D3DVALUE,
) -> HRESULT {
    d3d_device7_GetClipPlane(iface, idx, plane)
}
unsafe extern "system" fn d3d_device7_GetClipPlane_FPUPreserve(
    iface: *mut IDirect3DDevice7,
    idx: DWORD,
    plane: *mut D3DVALUE,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut old_fpucw: WORD;

    old_fpucw = d3d_fpu_setup();
    hr = d3d_device7_GetClipPlane(iface, idx, plane);
    set_fpu_control_word(old_fpucw);

    return hr;
}

unsafe extern "system" fn d3d_device7_GetInfo(iface: *mut IDirect3DDevice7, info_id: DWORD, info: *mut core::ffi::c_void, info_size: DWORD) -> HRESULT {
    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        match info_id {
            D3DDEVINFOID_TEXTUREMANAGER => { TRACE!("D3DDEVINFOID_TEXTUREMANAGER\n"); }
            D3DDEVINFOID_D3DTEXTUREMANAGER => { TRACE!("D3DDEVINFOID_D3DTEXTUREMANAGER\n"); }
            D3DDEVINFOID_TEXTURING => { TRACE!("D3DDEVINFOID_TEXTURING\n"); }
            _ => {
                ERR!(" invalid flag !!!\n");
                return DDERR_INVALIDPARAMS;
            }
        }
    }

    return S_FALSE; /* According to MSDN, this is valid for a non-debug driver */
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DDevice7(iface: *mut IDirect3DDevice7) -> *mut d3d_device {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(
        ((*iface).lpVtbl as *const _) == core::ptr::addr_of!(d3d_device7_fpu_preserve_vtbl) as *const _
            || ((*iface).lpVtbl as *const _) == core::ptr::addr_of!(d3d_device7_fpu_setup_vtbl) as *const _
    );
    return (iface as usize).wrapping_sub(core::mem::offset_of!(d3d_device, IDirect3DDevice7_iface))
        as *mut d3d_device;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DDevice3(iface: *mut IDirect3DDevice3) -> *mut d3d_device {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl == core::ptr::addr_of!(d3d_device3_vtbl));
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(d3d_device, IDirect3DDevice3_iface)) as *mut d3d_device;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DDevice2(iface: *mut IDirect3DDevice2) -> *mut d3d_device {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(((*iface).lpVtbl == core::ptr::addr_of!(d3d_device2_vtbl)));
    return (iface as *mut u8).sub(core::mem::offset_of!(d3d_device, IDirect3DDevice2_iface)) as *mut d3d_device;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DDevice(iface: *mut IDirect3DDevice) -> *mut d3d_device {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl == &raw const d3d_device1_vtbl);
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(d3d_device, IDirect3DDevice_iface)) as *mut d3d_device;
}
#[no_mangle]
pub unsafe extern "C" fn d3d_device_update_depth_stencil(
    mut device: *mut d3d_device,
) -> wined3d_depth_buffer_type {
    let mut depthStencil: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
    let mut render_target: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
    static mut depthcaps: DDSCAPS2 = DDSCAPS2 {
        dwCaps: DDSCAPS_ZBUFFER as DWORD,
        dwCaps2: 0 as DWORD,
        dwCaps3: 0 as DWORD,
        __bindgen_anon_1: _DDSCAPS2__bindgen_ty_1 { dwCaps4: 0 as DWORD,
        },
    };
    if !(*device).rt_iface.is_null()
        && IUnknown_QueryInterface(
            (*device).rt_iface,
            &raw const IID_IDirectDrawSurface7,
            &raw mut render_target as *mut *mut core::ffi::c_void,
        ) >= 0
    {
        IDirectDrawSurface7_GetAttachedSurface(
            render_target,
            &raw mut depthcaps,
            &raw mut depthStencil,
        );
        IDirectDrawSurface7_Release(render_target);
    }
    if depthStencil.is_null() {
        WINE_TRACE!("Setting wined3d depth stencil to NULL\n");
        wined3d_device_context_set_depth_stencil_view(
            (*device).immediate_context,
            core::ptr::null_mut::<wined3d_rendertarget_view>(),
        );
        wined3d_stateblock_depth_buffer_changed((*device).state);
        (*device).target_ds = core::ptr::null_mut::<ddraw_surface>();
        return WINED3D_ZB_FALSE;
    }

    (*device).target_ds = impl_from_IDirectDrawSurface7(depthStencil);
    wined3d_device_context_set_depth_stencil_view(
        (*device).immediate_context,
        ddraw_surface_get_rendertarget_view((*device).target_ds),
    );
    wined3d_stateblock_depth_buffer_changed((*device).state);

    IDirectDrawSurface7_Release(depthStencil);
    return WINED3D_ZB_TRUE;
}

unsafe extern "C" fn device_reset_viewport_state(mut device: *mut d3d_device) {
    let mut vp: wined3d_viewport = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    wined3d_device_context_get_viewports(
        (*device).immediate_context,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(vp),
    );
    wined3d_stateblock_set_viewport((*device).state, core::ptr::addr_of_mut!(vp));
    wined3d_device_context_get_scissor_rects(
        (*device).immediate_context,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(rect),
    );
    wined3d_stateblock_set_scissor_rect((*device).state, core::ptr::addr_of_mut!(rect));
}

unsafe extern "C" fn d3d_device_init(device: *mut d3d_device, ddraw: *mut ddraw, guid: *const GUID, target: *mut ddraw_surface, rt_iface: *mut IUnknown, version: UINT, outer_unknown: *mut IUnknown) -> HRESULT {
    static ident: wined3d_matrix = wined3d_matrix {
        _11: 1.0f32,
        _12: 0.0f32,
        _13: 0.0f32,
        _14: 0.0f32,
        _21: 0.0f32,
        _22: 1.0f32,
        _23: 0.0f32,
        _24: 0.0f32,
        _31: 0.0f32,
        _32: 0.0f32,
        _33: 1.0f32,
        _34: 0.0f32,
        _41: 0.0f32,
        _42: 0.0f32,
        _43: 0.0f32,
        _44: 1.0f32,
    };
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut hr: HRESULT;

    if (*ddraw).cooperative_level & DDSCL_FPUPRESERVE != 0 {
        (*device).IDirect3DDevice7_iface.lpVtbl = &raw const d3d_device7_fpu_preserve_vtbl as *const IDirect3DDevice7Vtbl;
    } else {
        (*device).IDirect3DDevice7_iface.lpVtbl = &raw const d3d_device7_fpu_setup_vtbl as *const IDirect3DDevice7Vtbl;
    }
    (*device).IDirect3DDevice3_iface.lpVtbl = &raw const d3d_device3_vtbl as *const IDirect3DDevice3Vtbl;
    (*device).IDirect3DDevice2_iface.lpVtbl = &raw const d3d_device2_vtbl as *const IDirect3DDevice2Vtbl;
    (*device).IDirect3DDevice_iface.lpVtbl = &raw const d3d_device1_vtbl as *const IDirect3DDeviceVtbl;
    (*device).IUnknown_inner.lpVtbl = &raw const d3d_device_inner_vtbl as *const IUnknownVtbl;
    (*device).ref_ = 1 as LONG;
    (*device).version = version;
    (*device).hardware_device = (IsEqualGUID!(&raw const IID_IDirect3DTnLHalDevice, guid) != false
        || IsEqualGUID!(&raw const IID_IDirect3DHALDevice, guid) != false) as BOOL;

    if (*device).hardware_device != 0 && (*target).surface_desc.ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY == 0 {
        WARN!("Surface %p is not in video memory.\n", target);
        return D3DERR_SURFACENOTINVIDMEM;
    }

    if !outer_unknown.is_null() {
        (*device).outer_unknown = outer_unknown;
    } else {
        (*device).outer_unknown = core::ptr::addr_of_mut!((*device).IUnknown_inner);
    }

    (*device).ddraw = ddraw;
    list_init(core::ptr::addr_of_mut!((*device).viewport_list));

    if ddraw_handle_table_init(core::ptr::addr_of_mut!((*device).handle_table), 64) == 0 {
        ERR!("Failed to initialize handle table.\n");
        return DDERR_OUTOFMEMORY;
    }

    (*device).legacyTextureBlending = (FALSE as i32);
    (*device).legacy_projection = ident;
    (*device).legacy_clipspace = ident;

    hr = wined3d_stateblock_create((*ddraw).wined3d_device, core::ptr::null::<wined3d_stateblock>(), WINED3D_SBT_PRIMARY, core::ptr::addr_of_mut!((*device).state));
    if hr < 0 {
        ERR!("Failed to create the primary stateblock, hr %#lx.\n", hr);
        ddraw_handle_table_destroy(core::ptr::addr_of_mut!((*device).handle_table));
        return hr;
    }
    (*device).stateblock_state = wined3d_stateblock_get_state((*device).state);
    (*device).update_state = (*device).state;

    /* This is for convenience. */
    (*device).wined3d_device = (*ddraw).wined3d_device;
    (*device).immediate_context = (*ddraw).immediate_context;
    wined3d_device_incref((*ddraw).wined3d_device);

    wined3d_streaming_buffer_init(core::ptr::addr_of_mut!((*device).vertex_buffer), WINED3D_BIND_VERTEX_BUFFER);
    wined3d_streaming_buffer_init(core::ptr::addr_of_mut!((*device).index_buffer), WINED3D_BIND_INDEX_BUFFER);

    /* Render to the back buffer */
    rtv = ddraw_surface_get_rendertarget_view(target);
    hr = wined3d_device_context_set_rendertarget_views((*device).immediate_context, 0, 1, core::ptr::addr_of_mut!(rtv), (TRUE) as BOOL);
    if hr < 0 {
        ERR!("Failed to set render target, hr %#lx.\n", hr);
        wined3d_stateblock_decref((*device).state);
        ddraw_handle_table_destroy(core::ptr::addr_of_mut!((*device).handle_table));
        return hr;
    }

    (*device).rt_iface = rt_iface;
    (*device).target = target;
    if version != 1 {
        IUnknown_AddRef((*device).rt_iface);
    }

    list_add_head(core::ptr::addr_of_mut!((*ddraw).d3ddevice_list), core::ptr::addr_of_mut!((*device).ddraw_entry));

    wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_ZENABLE, d3d_device_update_depth_stencil(device) as DWORD);
    if version == 1 {
        /* Color keying is initially enabled for version 1 devices. */
        wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_COLORKEYENABLE, TRUE);
    } else if version == 2 {
        wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_SPECULARENABLE, TRUE);
    }
    if version < 7 {
        wined3d_stateblock_set_render_state((*device).state, WINED3D_RS_NORMALIZENORMALS, TRUE);
        IDirect3DDevice3_SetRenderState(core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface), D3DRENDERSTATE_TEXTUREMAPBLEND, D3DTBLEND_MODULATE);
    }
    device_reset_viewport_state(device);
    return D3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn d3d_device_create(ddraw: *mut ddraw, guid: *const GUID, target: *mut ddraw_surface, rt_iface: *mut IUnknown, version: UINT, device: *mut *mut d3d_device, outer_unknown: *mut IUnknown) -> HRESULT {
    let mut object: *mut d3d_device = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!("ddraw %p, guid %s, target %p, version %u, device %p, outer_unknown %p.\n", ddraw, debugstr_guid(guid), target, version, device, outer_unknown);

    if ((*target).surface_desc.ddsCaps.dwCaps & DDSCAPS_3DDEVICE) == 0
            || ((*target).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) != 0
    {
        WARN!("Surface %p is not a render target.\n", target);
        return DDERR_INVALIDCAPS;
    }

    if !(validate_surface_palette(target) != 0)
    {
        WARN!("Surface %p has an indexed pixel format, but no palette.\n", target);
        return DDERR_NOPALETTEATTACHED;
    }

    if ((*ddraw).flags & DDRAW_NO3D) != 0
    {
        WINE_ERR_!(__wine_dbch_winediag, "The application wants to create a Direct3D device, but the current DirectDrawRenderer does not support this.\n");

        return DDERR_OUTOFMEMORY;
    }

    object = calloc(1, (core::mem::size_of::<d3d_device>()) as _) as *mut d3d_device;
    if object.is_null()
    {
        ERR!("Failed to allocate device memory.\n");
        return DDERR_OUTOFMEMORY;
    }

    hr = d3d_device_init(object, ddraw, guid, target, rt_iface, version, outer_unknown);
    if hr < 0
    {
        WARN!("Failed to initialize device, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    TRACE!("Created device %p.\n", object);

    *device = object;

    return D3D_OK;
}

// COM vtable tables — generated from device.c by vtbl_gen.py
/// device.c:6557 `static const IDirect3DDevice7Vtbl d3d_device7_fpu_setup_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_device7_fpu_setup_vtbl: IDirect3DDevice7Vtbl = IDirect3DDevice7Vtbl {
    QueryInterface: Some(d3d_device7_QueryInterface),
    AddRef: Some(d3d_device7_AddRef),
    Release: Some(d3d_device7_Release),
    GetCaps: Some(d3d_device7_GetCaps_FPUSetup),
    EnumTextureFormats: Some(d3d_device7_EnumTextureFormats_FPUSetup),
    BeginScene: Some(d3d_device7_BeginScene_FPUSetup),
    EndScene: Some(d3d_device7_EndScene_FPUSetup),
    GetDirect3D: Some(d3d_device7_GetDirect3D),
    SetRenderTarget: Some(d3d_device7_SetRenderTarget_FPUSetup),
    GetRenderTarget: Some(d3d_device7_GetRenderTarget),
    Clear: Some(d3d_device7_Clear_FPUSetup),
    SetTransform: Some(d3d_device7_SetTransform_FPUSetup),
    GetTransform: Some(d3d_device7_GetTransform_FPUSetup),
    SetViewport: Some(d3d_device7_SetViewport_FPUSetup),
    MultiplyTransform: Some(d3d_device7_MultiplyTransform_FPUSetup),
    GetViewport: Some(d3d_device7_GetViewport_FPUSetup),
    SetMaterial: Some(d3d_device7_SetMaterial_FPUSetup),
    GetMaterial: Some(d3d_device7_GetMaterial_FPUSetup),
    SetLight: Some(d3d_device7_SetLight_FPUSetup),
    GetLight: Some(d3d_device7_GetLight_FPUSetup),
    SetRenderState: Some(d3d_device7_SetRenderState_FPUSetup),
    GetRenderState: Some(d3d_device7_GetRenderState_FPUSetup),
    BeginStateBlock: Some(d3d_device7_BeginStateBlock_FPUSetup),
    EndStateBlock: Some(d3d_device7_EndStateBlock_FPUSetup),
    PreLoad: Some(d3d_device7_PreLoad_FPUSetup),
    DrawPrimitive: Some(d3d_device7_DrawPrimitive_FPUSetup),
    DrawIndexedPrimitive: Some(d3d_device7_DrawIndexedPrimitive_FPUSetup),
    SetClipStatus: Some(d3d_device7_SetClipStatus),
    GetClipStatus: Some(d3d_device7_GetClipStatus),
    DrawPrimitiveStrided: Some(d3d_device7_DrawPrimitiveStrided_FPUSetup),
    DrawIndexedPrimitiveStrided: Some(d3d_device7_DrawIndexedPrimitiveStrided_FPUSetup),
    DrawPrimitiveVB: Some(d3d_device7_DrawPrimitiveVB_FPUSetup),
    DrawIndexedPrimitiveVB: Some(d3d_device7_DrawIndexedPrimitiveVB_FPUSetup),
    ComputeSphereVisibility: Some(d3d_device7_ComputeSphereVisibility),
    GetTexture: Some(d3d_device7_GetTexture_FPUSetup),
    SetTexture: Some(d3d_device7_SetTexture_FPUSetup),
    GetTextureStageState: Some(d3d_device7_GetTextureStageState_FPUSetup),
    SetTextureStageState: Some(d3d_device7_SetTextureStageState_FPUSetup),
    ValidateDevice: Some(d3d_device7_ValidateDevice_FPUSetup),
    ApplyStateBlock: Some(d3d_device7_ApplyStateBlock_FPUSetup),
    CaptureStateBlock: Some(d3d_device7_CaptureStateBlock_FPUSetup),
    DeleteStateBlock: Some(d3d_device7_DeleteStateBlock_FPUSetup),
    CreateStateBlock: Some(d3d_device7_CreateStateBlock_FPUSetup),
    Load: Some(d3d_device7_Load_FPUSetup),
    LightEnable: Some(d3d_device7_LightEnable_FPUSetup),
    GetLightEnable: Some(d3d_device7_GetLightEnable_FPUSetup),
    SetClipPlane: Some(d3d_device7_SetClipPlane_FPUSetup),
    GetClipPlane: Some(d3d_device7_GetClipPlane_FPUSetup),
    GetInfo: Some(d3d_device7_GetInfo),
};

/// device.c:6612 `static const IDirect3DDevice7Vtbl d3d_device7_fpu_preserve_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_device7_fpu_preserve_vtbl: IDirect3DDevice7Vtbl = IDirect3DDevice7Vtbl {
    QueryInterface: Some(d3d_device7_QueryInterface),
    AddRef: Some(d3d_device7_AddRef),
    Release: Some(d3d_device7_Release),
    GetCaps: Some(d3d_device7_GetCaps_FPUPreserve),
    EnumTextureFormats: Some(d3d_device7_EnumTextureFormats_FPUPreserve),
    BeginScene: Some(d3d_device7_BeginScene_FPUPreserve),
    EndScene: Some(d3d_device7_EndScene_FPUPreserve),
    GetDirect3D: Some(d3d_device7_GetDirect3D),
    SetRenderTarget: Some(d3d_device7_SetRenderTarget_FPUPreserve),
    GetRenderTarget: Some(d3d_device7_GetRenderTarget),
    Clear: Some(d3d_device7_Clear_FPUPreserve),
    SetTransform: Some(d3d_device7_SetTransform_FPUPreserve),
    GetTransform: Some(d3d_device7_GetTransform_FPUPreserve),
    SetViewport: Some(d3d_device7_SetViewport_FPUPreserve),
    MultiplyTransform: Some(d3d_device7_MultiplyTransform_FPUPreserve),
    GetViewport: Some(d3d_device7_GetViewport_FPUPreserve),
    SetMaterial: Some(d3d_device7_SetMaterial_FPUPreserve),
    GetMaterial: Some(d3d_device7_GetMaterial_FPUPreserve),
    SetLight: Some(d3d_device7_SetLight_FPUPreserve),
    GetLight: Some(d3d_device7_GetLight_FPUPreserve),
    SetRenderState: Some(d3d_device7_SetRenderState_FPUPreserve),
    GetRenderState: Some(d3d_device7_GetRenderState_FPUPreserve),
    BeginStateBlock: Some(d3d_device7_BeginStateBlock_FPUPreserve),
    EndStateBlock: Some(d3d_device7_EndStateBlock_FPUPreserve),
    PreLoad: Some(d3d_device7_PreLoad_FPUPreserve),
    DrawPrimitive: Some(d3d_device7_DrawPrimitive_FPUPreserve),
    DrawIndexedPrimitive: Some(d3d_device7_DrawIndexedPrimitive_FPUPreserve),
    SetClipStatus: Some(d3d_device7_SetClipStatus),
    GetClipStatus: Some(d3d_device7_GetClipStatus),
    DrawPrimitiveStrided: Some(d3d_device7_DrawPrimitiveStrided_FPUPreserve),
    DrawIndexedPrimitiveStrided: Some(d3d_device7_DrawIndexedPrimitiveStrided_FPUPreserve),
    DrawPrimitiveVB: Some(d3d_device7_DrawPrimitiveVB_FPUPreserve),
    DrawIndexedPrimitiveVB: Some(d3d_device7_DrawIndexedPrimitiveVB_FPUPreserve),
    ComputeSphereVisibility: Some(d3d_device7_ComputeSphereVisibility),
    GetTexture: Some(d3d_device7_GetTexture_FPUPreserve),
    SetTexture: Some(d3d_device7_SetTexture_FPUPreserve),
    GetTextureStageState: Some(d3d_device7_GetTextureStageState_FPUPreserve),
    SetTextureStageState: Some(d3d_device7_SetTextureStageState_FPUPreserve),
    ValidateDevice: Some(d3d_device7_ValidateDevice_FPUPreserve),
    ApplyStateBlock: Some(d3d_device7_ApplyStateBlock_FPUPreserve),
    CaptureStateBlock: Some(d3d_device7_CaptureStateBlock_FPUPreserve),
    DeleteStateBlock: Some(d3d_device7_DeleteStateBlock_FPUPreserve),
    CreateStateBlock: Some(d3d_device7_CreateStateBlock_FPUPreserve),
    Load: Some(d3d_device7_Load_FPUPreserve),
    LightEnable: Some(d3d_device7_LightEnable_FPUPreserve),
    GetLightEnable: Some(d3d_device7_GetLightEnable_FPUPreserve),
    SetClipPlane: Some(d3d_device7_SetClipPlane_FPUPreserve),
    GetClipPlane: Some(d3d_device7_GetClipPlane_FPUPreserve),
    GetInfo: Some(d3d_device7_GetInfo),
};

/// device.c:6667 `static const IDirect3DDevice3Vtbl d3d_device3_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_device3_vtbl: IDirect3DDevice3Vtbl = IDirect3DDevice3Vtbl {
    QueryInterface: Some(d3d_device3_QueryInterface),
    AddRef: Some(d3d_device3_AddRef),
    Release: Some(d3d_device3_Release),
    GetCaps: Some(d3d_device3_GetCaps),
    GetStats: Some(d3d_device3_GetStats),
    AddViewport: Some(d3d_device3_AddViewport),
    DeleteViewport: Some(d3d_device3_DeleteViewport),
    NextViewport: Some(d3d_device3_NextViewport),
    EnumTextureFormats: Some(d3d_device3_EnumTextureFormats),
    BeginScene: Some(d3d_device3_BeginScene),
    EndScene: Some(d3d_device3_EndScene),
    GetDirect3D: Some(d3d_device3_GetDirect3D),
    SetCurrentViewport: Some(d3d_device3_SetCurrentViewport),
    GetCurrentViewport: Some(d3d_device3_GetCurrentViewport),
    SetRenderTarget: Some(d3d_device3_SetRenderTarget),
    GetRenderTarget: Some(d3d_device3_GetRenderTarget),
    Begin: Some(d3d_device3_Begin),
    BeginIndexed: Some(d3d_device3_BeginIndexed),
    Vertex: Some(d3d_device3_Vertex),
    Index: Some(d3d_device3_Index),
    End: Some(d3d_device3_End),
    GetRenderState: Some(d3d_device3_GetRenderState),
    SetRenderState: Some(d3d_device3_SetRenderState),
    GetLightState: Some(d3d_device3_GetLightState),
    SetLightState: Some(d3d_device3_SetLightState),
    SetTransform: Some(d3d_device3_SetTransform),
    GetTransform: Some(d3d_device3_GetTransform),
    MultiplyTransform: Some(d3d_device3_MultiplyTransform),
    DrawPrimitive: Some(d3d_device3_DrawPrimitive),
    DrawIndexedPrimitive: Some(d3d_device3_DrawIndexedPrimitive),
    SetClipStatus: Some(d3d_device3_SetClipStatus),
    GetClipStatus: Some(d3d_device3_GetClipStatus),
    DrawPrimitiveStrided: Some(d3d_device3_DrawPrimitiveStrided),
    DrawIndexedPrimitiveStrided: Some(d3d_device3_DrawIndexedPrimitiveStrided),
    DrawPrimitiveVB: Some(d3d_device3_DrawPrimitiveVB),
    DrawIndexedPrimitiveVB: Some(d3d_device3_DrawIndexedPrimitiveVB),
    ComputeSphereVisibility: Some(d3d_device3_ComputeSphereVisibility),
    GetTexture: Some(d3d_device3_GetTexture),
    SetTexture: Some(d3d_device3_SetTexture),
    GetTextureStageState: Some(d3d_device3_GetTextureStageState),
    SetTextureStageState: Some(d3d_device3_SetTextureStageState),
    ValidateDevice: Some(d3d_device3_ValidateDevice),
};

/// device.c:6715 `static const IDirect3DDevice2Vtbl d3d_device2_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_device2_vtbl: IDirect3DDevice2Vtbl = IDirect3DDevice2Vtbl {
    QueryInterface: Some(d3d_device2_QueryInterface),
    AddRef: Some(d3d_device2_AddRef),
    Release: Some(d3d_device2_Release),
    GetCaps: Some(d3d_device2_GetCaps),
    SwapTextureHandles: Some(d3d_device2_SwapTextureHandles),
    GetStats: Some(d3d_device2_GetStats),
    AddViewport: Some(d3d_device2_AddViewport),
    DeleteViewport: Some(d3d_device2_DeleteViewport),
    NextViewport: Some(d3d_device2_NextViewport),
    EnumTextureFormats: Some(d3d_device2_EnumTextureFormats),
    BeginScene: Some(d3d_device2_BeginScene),
    EndScene: Some(d3d_device2_EndScene),
    GetDirect3D: Some(d3d_device2_GetDirect3D),
    SetCurrentViewport: Some(d3d_device2_SetCurrentViewport),
    GetCurrentViewport: Some(d3d_device2_GetCurrentViewport),
    SetRenderTarget: Some(d3d_device2_SetRenderTarget),
    GetRenderTarget: Some(d3d_device2_GetRenderTarget),
    Begin: Some(d3d_device2_Begin),
    BeginIndexed: Some(d3d_device2_BeginIndexed),
    Vertex: Some(d3d_device2_Vertex),
    Index: Some(d3d_device2_Index),
    End: Some(d3d_device2_End),
    GetRenderState: Some(d3d_device2_GetRenderState),
    SetRenderState: Some(d3d_device2_SetRenderState),
    GetLightState: Some(d3d_device2_GetLightState),
    SetLightState: Some(d3d_device2_SetLightState),
    SetTransform: Some(d3d_device2_SetTransform),
    GetTransform: Some(d3d_device2_GetTransform),
    MultiplyTransform: Some(d3d_device2_MultiplyTransform),
    DrawPrimitive: Some(d3d_device2_DrawPrimitive),
    DrawIndexedPrimitive: Some(d3d_device2_DrawIndexedPrimitive),
    SetClipStatus: Some(d3d_device2_SetClipStatus),
    GetClipStatus: Some(d3d_device2_GetClipStatus),
};

/// device.c:6754 `static const IDirect3DDeviceVtbl d3d_device1_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_device1_vtbl: IDirect3DDeviceVtbl = IDirect3DDeviceVtbl {
    QueryInterface: Some(d3d_device1_QueryInterface),
    AddRef: Some(d3d_device1_AddRef),
    Release: Some(d3d_device1_Release),
    Initialize: Some(d3d_device1_Initialize),
    GetCaps: Some(d3d_device1_GetCaps),
    SwapTextureHandles: Some(d3d_device1_SwapTextureHandles),
    CreateExecuteBuffer: Some(d3d_device1_CreateExecuteBuffer),
    GetStats: Some(d3d_device1_GetStats),
    Execute: Some(d3d_device1_Execute),
    AddViewport: Some(d3d_device1_AddViewport),
    DeleteViewport: Some(d3d_device1_DeleteViewport),
    NextViewport: Some(d3d_device1_NextViewport),
    Pick: Some(d3d_device1_Pick),
    GetPickRecords: Some(d3d_device1_GetPickRecords),
    EnumTextureFormats: Some(d3d_device1_EnumTextureFormats),
    CreateMatrix: Some(d3d_device1_CreateMatrix),
    SetMatrix: Some(d3d_device1_SetMatrix),
    GetMatrix: Some(d3d_device1_GetMatrix),
    DeleteMatrix: Some(d3d_device1_DeleteMatrix),
    BeginScene: Some(d3d_device1_BeginScene),
    EndScene: Some(d3d_device1_EndScene),
    GetDirect3D: Some(d3d_device1_GetDirect3D),
};

/// device.c:6782 `static const IUnknownVtbl d3d_device_inner_vtbl`  (slot order: com-sys/bindgen_i386.rs)
pub static d3d_device_inner_vtbl: IUnknownVtbl = IUnknownVtbl {
    QueryInterface: Some(d3d_device_inner_QueryInterface),
    AddRef: Some(d3d_device_inner_AddRef),
    Release: Some(d3d_device_inner_Release),
};
