use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(ddraw): debug channel for this file; the crate's DPRINT macros use it.
pub const WINE_DEFAULT_DEBUG_CHANNEL_NAME: &str = "ddraw";

#[repr(C)]
pub struct transform_vertices_vertex_payload {
    pub p: [DWORD; 4],
}

#[repr(C)]
pub struct transform_vertices_vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32, /* w is unused in input data. */
    pub payload: transform_vertices_vertex_payload,
}
unsafe extern "C" fn update_clip_space(
    device: *mut d3d_device,
    scale: *mut wined3d_vec3,
    offset: *mut wined3d_vec3,
) {
    let mut clip_space: wined3d_matrix = wined3d_matrix {
        _11: (*scale).x,
        _12: 0.0f32,
        _13: 0.0f32,
        _14: 0.0f32,
        _21: 0.0f32,
        _22: (*scale).y,
        _23: 0.0f32,
        _24: 0.0f32,
        _31: 0.0f32,
        _32: 0.0f32,
        _33: (*scale).z,
        _34: 0.0f32,
        _41: (*offset).x,
        _42: (*offset).y,
        _43: (*offset).z,
        _44: 1.0f32,
    };
    let mut projection: wined3d_matrix = wined3d_matrix {
        _11: 0.,
        _12: 0.,
        _13: 0.,
        _14: 0.,
        _21: 0.,
        _22: 0.,
        _23: 0.,
        _24: 0.,
        _31: 0.,
        _32: 0.,
        _33: 0.,
        _34: 0.,
        _41: 0.,
        _42: 0.,
        _43: 0.,
        _44: 0.,
    };
    multiply_matrix(
        core::ptr::addr_of_mut!(projection),
        core::ptr::addr_of_mut!(clip_space),
        core::ptr::addr_of_mut!((*device).legacy_projection),
    );
    wined3d_stateblock_set_transform(
        (*device).state,
        WINED3D_TS_PROJECTION,
        core::ptr::addr_of_mut!(projection),
    );
    (*device).legacy_clipspace = clip_space;
}
#[no_mangle]
pub unsafe extern "C" fn viewport_activate(This: *mut d3d_viewport, ignore_lights: BOOL) {
    let mut scale: wined3d_vec3 = wined3d_vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut offset: wined3d_vec3 = wined3d_vec3 {
        x: 0.,
        y: 0.,
        z: 0.,
    };
    let mut vp: D3DVIEWPORT7 = _D3DVIEWPORT7 {
        dwX: 0,
        dwY: 0,
        dwWidth: 0,
        dwHeight: 0,
        dvMinZ: 0.,
        dvMaxZ: 0.,
    };
    if ignore_lights == 0 {
        let mut light: *mut d3d_light = core::ptr::null_mut::<d3d_light>();
        light = ((*This).light_list.next as *mut u8)
            .offset(-(core::mem::offset_of!(d3d_light, entry) as isize)) as *mut d3d_light;
        while core::ptr::addr_of_mut!((*light).entry)
            != core::ptr::addr_of_mut!((*This).light_list)
        {
            light_activate(light);
            light = ((*light).entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(d3d_light, entry) as isize)) as *mut d3d_light;
        }
    }
    if (*This).version == DDRAW_VIEWPORT_VERSION_NONE {
        WINE_TRACE!("Viewport data was not set.\n");
        return;
    }
    if (*This).version == DDRAW_VIEWPORT_VERSION_2 {
        vp.dwX = (*This).viewports.vp2.dwX;
        vp.dwY = (*This).viewports.vp2.dwY;
        vp.dwHeight = (*This).viewports.vp2.dwHeight;
        vp.dwWidth = (*This).viewports.vp2.dwWidth;
        vp.dvMinZ = 0.0f32 as D3DVALUE;
        vp.dvMaxZ = 1.0f32 as D3DVALUE;
        scale.x = (2.0f32 / (*This).viewports.vp2.dvClipWidth) as f32;
        scale.y = (2.0f32 / (*This).viewports.vp2.dvClipHeight) as f32;
        scale.z = (1.0f32 / ((*This).viewports.vp2.dvMaxZ - (*This).viewports.vp2.dvMinZ)) as f32;
        offset.x = -2.0f32 * (*This).viewports.vp2.dvClipX as f32
            / (*This).viewports.vp2.dvClipWidth as f32
            - 1.0f32;
        offset.y = -2.0f32 * (*This).viewports.vp2.dvClipY as f32
            / (*This).viewports.vp2.dvClipHeight as f32
            + 1.0f32;
        offset.z = (-(*This).viewports.vp2.dvMinZ
            / ((*This).viewports.vp2.dvMaxZ - (*This).viewports.vp2.dvMinZ)) as f32;
    } else {
        vp.dwX = (*This).viewports.vp1.dwX;
        vp.dwY = (*This).viewports.vp1.dwY;
        vp.dwHeight = (*This).viewports.vp1.dwHeight;
        vp.dwWidth = (*This).viewports.vp1.dwWidth;
        vp.dvMinZ = 0.0f32 as D3DVALUE;
        vp.dvMaxZ = 1.0f32 as D3DVALUE;
        scale.x = 2.0f32 * (*This).viewports.vp1.dvScaleX as f32
            / (*This).viewports.vp1.dwWidth as f32;
        scale.y = 2.0f32 * (*This).viewports.vp1.dvScaleY as f32
            / (*This).viewports.vp1.dwHeight as f32;
        scale.z = 1.0f32;
        offset.x = 0.0f32;
        offset.y = 0.0f32;
        offset.z = 0.0f32;
    }
    update_clip_space(
        (*This).active_device,
        core::ptr::addr_of_mut!(scale),
        core::ptr::addr_of_mut!(offset),
    );
    IDirect3DDevice7_SetViewport(
        core::ptr::addr_of_mut!((*(*This).active_device).IDirect3DDevice7_iface),
        core::ptr::addr_of_mut!(vp),
    );
}
#[no_mangle]
pub unsafe extern "C" fn viewport_deactivate(viewport: *mut d3d_viewport) {
    let mut light: *mut d3d_light = core::ptr::null_mut();
    light = ((*viewport).light_list.next as *mut u8)
        .offset(-(core::mem::offset_of!(d3d_light, entry) as isize)) as *mut d3d_light;
    while core::ptr::addr_of_mut!((*light).entry) != core::ptr::addr_of_mut!((*viewport).light_list) {
        light_deactivate(light);
        light = ((*light).entry.next as *mut u8)
            .offset(-(core::mem::offset_of!(d3d_light, entry) as isize)) as *mut d3d_light;
    }
}
#[no_mangle]
pub unsafe extern "C" fn viewport_alloc_active_light_index(light: *mut d3d_light) {
    let vp: *mut d3d_viewport = (*light).active_viewport;
    let mut i: DWORD;
    let mut map: DWORD;

    TRACE!(
        "vp %p, light %p, index %lu, active_lights_count %lu.\n",
        vp,
        light,
        (*light).active_light_index,
        (*vp).active_lights_count
    );

    if (*light).active_light_index != 0 {
        return;
    }

    if (*vp).active_lights_count >= DDRAW_MAX_ACTIVE_LIGHTS as DWORD {
        let mut l: *mut d3d_light = core::ptr::null_mut::<d3d_light>();

        l = ((*vp).light_list.next as *mut u8)
            .sub(core::mem::offset_of!(d3d_light, entry)) as *mut d3d_light;
        while &raw mut (*l).entry != &raw mut (*vp).light_list {
            if (*l).active_light_index != 0 {
                WARN!(
                    "Too many active lights, viewport %p, light %p, deactivating %p.\n",
                    vp,
                    light,
                    l
                );
                light_deactivate(l);

                /* Recycle active lights in a FIFO way. */
                list_remove(&raw mut (*light).entry);
                list_add_tail(&raw mut (*vp).light_list, &raw mut (*light).entry);
                break;
            } else {
                l = ((*l).entry.next as *mut u8)
                    .sub(core::mem::offset_of!(d3d_light, entry)) as *mut d3d_light;
            }
        }
    }

    map = !(*vp).map_lights;
    ASSERT!((*vp).active_lights_count < DDRAW_MAX_ACTIVE_LIGHTS as DWORD && map != 0);
    i = wined3d_bit_scan(&raw mut map) as DWORD;
    (*light).active_light_index = i.wrapping_add(1 as DWORD) as DWORD;
    (*vp).active_lights_count = (*vp).active_lights_count.wrapping_add(1);
    (*vp).map_lights |= (1 as DWORD) << i;
}
#[no_mangle]
pub unsafe extern "C" fn viewport_free_active_light_index(light: *mut d3d_light) {
    let mut vp: *mut d3d_viewport = (*light).active_viewport;

    if (*light).active_light_index == 0 {
        return;
    }

    assert!(
        ((*vp).map_lights & (1 << ((*light).active_light_index - 1))) != 0
    );

    (*vp).active_lights_count = (*vp).active_lights_count.wrapping_sub(1);
    (*vp).map_lights &= !(1 << ((*light).active_light_index - 1));
    (*light).active_light_index = 0;
}

unsafe extern "C" fn _dump_D3DVIEWPORT(lpvp: *const D3DVIEWPORT) {
    // TRACE(" - dwSize = %lu dwX = %lu dwY = %lu\n", lpvp->dwSize, lpvp->dwX, lpvp->dwY);
    // TRACE(" - dwWidth = %lu dwHeight = %lu\n", lpvp->dwWidth, lpvp->dwHeight);
    // TRACE(" - dvScaleX = %f dvScaleY = %f\n", lpvp->dvScaleX, lpvp->dvScaleY);
    // TRACE(" - dvMaxX = %f dvMaxY = %f\n", lpvp->dvMaxX, lpvp->dvMaxY);
    // TRACE(" - dvMinZ = %f dvMaxZ = %f\n", lpvp->dvMinZ, lpvp->dvMaxZ);
}

unsafe extern "C" fn _dump_D3DVIEWPORT2(lpvp: *const D3DVIEWPORT2) {
    // TRACE(" - dwSize = %lu dwX = %lu dwY = %lu\n", lpvp->dwSize, lpvp->dwX, lpvp->dwY);
    // TRACE(" - dwWidth = %lu dwHeight = %lu\n", lpvp->dwWidth, lpvp->dwHeight);
    // TRACE(" - dvClipX = %f dvClipY = %f\n", lpvp->dvClipX, lpvp->dvClipY);
    // TRACE(" - dvClipWidth = %f dvClipHeight = %f\n", lpvp->dvClipWidth, lpvp->dvClipHeight);
    // TRACE(" - dvMinZ = %f dvMaxZ = %f\n", lpvp->dvMinZ, lpvp->dvMaxZ);
}

unsafe extern "C" fn impl_from_IDirect3DViewport3(iface: *mut IDirect3DViewport3) -> *mut d3d_viewport {
    return (iface as *mut u8).wrapping_sub(core::mem::offset_of!(d3d_viewport, IDirect3DViewport3_iface)) as *mut d3d_viewport;
}

unsafe extern "system" fn d3d_viewport_QueryInterface(iface: *mut IDirect3DViewport3, riid: REFIID, object: *mut *mut core::ffi::c_void) -> HRESULT {

    if IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DViewport3), riid)
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DViewport2), riid)
            || IsEqualGUID!(core::ptr::addr_of!(IID_IDirect3DViewport), riid)
            || IsEqualGUID!(core::ptr::addr_of!(IID_IUnknown), riid)
    {
        IDirect3DViewport3_AddRef(iface);
        *object = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    *object = core::ptr::null_mut();
    return E_NOINTERFACE;
}
unsafe extern "system" fn d3d_viewport_AddRef(iface: *mut IDirect3DViewport3) -> ULONG {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let ref_: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*viewport).ref_) as *mut core::ffi::c_long) as ULONG;

    return ref_;
}

unsafe extern "system" fn d3d_viewport_Release(iface: *mut IDirect3DViewport3) -> ULONG {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let ref_0: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*viewport).ref_) as *mut LONG) as ULONG;
    // TRACE line elided: the C source has no trace statement at this point.

    if ref_0 == 0 {
        free(viewport as *mut core::ffi::c_void);
    }

    return ref_0;
}
unsafe extern "system" fn d3d_viewport_Initialize(
    iface: *mut IDirect3DViewport3,
    d3d: *mut IDirect3D,
) -> HRESULT {
    // TRACE("iface %p, d3d %p.\n", iface, d3d);
    DDERR_ALREADYINITIALIZED
}

unsafe extern "system" fn d3d_viewport_GetViewport(iface: *mut IDirect3DViewport3, vp: *mut D3DVIEWPORT) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut size: DWORD;

    if vp.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*viewport).version == DDRAW_VIEWPORT_VERSION_NONE {
        WARN!("Viewport data was not set.\n");
        return D3DERR_VIEWPORTDATANOTSET;
    }

    wined3d_mutex_lock();

    size = (*vp).dwSize;
    if (*viewport).version == DDRAW_VIEWPORT_VERSION_1 {
        memcpy(
            vp as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*viewport).viewports.vp1) as *const core::ffi::c_void,
            size as usize,
        );
    } else {
        let mut vp1: D3DVIEWPORT = core::mem::zeroed();

        vp1.dwSize = core::mem::size_of::<D3DVIEWPORT>() as DWORD;
        vp1.dwX = (*viewport).viewports.vp2.dwX;
        vp1.dwY = (*viewport).viewports.vp2.dwY;
        vp1.dwWidth = (*viewport).viewports.vp2.dwWidth;
        vp1.dwHeight = (*viewport).viewports.vp2.dwHeight;

        vp1.dvScaleX = vp1.dwWidth as D3DVALUE / (*viewport).viewports.vp2.dvClipWidth;
        vp1.dvScaleY = vp1.dwHeight as D3DVALUE / (*viewport).viewports.vp2.dvClipHeight;
        vp1.dvMaxX = (*viewport).viewports.vp2.dvClipWidth + (*viewport).viewports.vp2.dvClipX;
        vp1.dvMaxY = (*viewport).viewports.vp2.dvClipY;
        vp1.dvMinZ = 0.0f32 as D3DVALUE;
        vp1.dvMaxZ = 1.0f32 as D3DVALUE;
        memcpy(
            vp as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(vp1) as *const core::ffi::c_void,
            size as usize,
        );
    }

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        _dump_D3DVIEWPORT(vp);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_SetViewport(
    iface: *mut IDirect3DViewport3,
    vp: *mut D3DVIEWPORT,
) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let device: *mut d3d_device = (*viewport).active_device;
    let mut rt_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut current_viewport: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut surface: *mut ddraw_surface = core::ptr::null_mut();

    if vp.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*vp).dwSize as usize != core::mem::size_of::<D3DVIEWPORT>() {
        WARN!("Invalid D3DVIEWPORT size %lu.\n", (*vp).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        _dump_D3DVIEWPORT(vp);
    }

    if device.is_null() {
        WARN!("Viewport not bound to a device, returning D3DERR_VIEWPORTHASNODEVICE.\n");
        return D3DERR_VIEWPORTHASNODEVICE;
    }

    wined3d_mutex_lock();

    if (*device).version > 1 {
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
            (*vp).dwX as core::ffi::c_uint,
            (*vp).dwWidth as core::ffi::c_uint,
            rt_desc.width,
        ) || !wined3d_bound_range(
            (*vp).dwY as core::ffi::c_uint,
            (*vp).dwHeight as core::ffi::c_uint,
            rt_desc.height,
        ) {
            WARN!("Invalid viewport, returning DDERR_INVALIDPARAMS.\n");
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
    }

    (*viewport).version = DDRAW_VIEWPORT_VERSION_1;
    (*viewport).viewports.vp1 = *vp;

    /* Empirical testing on a couple of d3d1 games showed that these values
     * should be ignored. */
    (*viewport).viewports.vp1.dvMinZ = 0.0f32;
    (*viewport).viewports.vp1.dvMaxZ = 1.0f32;

    if IDirect3DDevice3_GetCurrentViewport(
        core::ptr::addr_of_mut!((*device).IDirect3DDevice3_iface),
        core::ptr::addr_of_mut!(current_viewport),
    ) >= 0
    {
        if current_viewport == iface {
            viewport_activate(viewport, (FALSE) as BOOL);
        }
        IDirect3DViewport3_Release(current_viewport);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_TransformVertices(
    iface: *mut IDirect3DViewport3,
    dwVertexCount: DWORD,
    data: *mut D3DTRANSFORMDATA,
    dwFlags: DWORD,
    offscreen: *mut DWORD,
) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut state: *const wined3d_stateblock_state = core::ptr::null();
    let mut vp: D3DVIEWPORT = (*viewport).viewports.vp1;
    let mut in_: *mut transform_vertices_vertex = core::ptr::null_mut();
    let mut out: *mut transform_vertices_vertex = core::ptr::null_mut();
    let mut mat: wined3d_matrix = core::mem::zeroed();
    let mut x: f32 = 0.0;
    let mut y: f32 = 0.0;
    let mut z: f32 = 0.0;
    let mut w: f32 = 0.0;
    let mut i: u32 = 0;
    let mut outH: *mut D3DHVERTEX = core::ptr::null_mut();
    let device: *mut d3d_device = (*viewport).active_device;
    let activate: BOOL = ((*device).current_viewport != viewport) as i32;

    /* Tests on windows show that Windows crashes when this occurs,
     * so don't return the (intuitive) return value
    if (!device)
    {
        WARN("No device active, returning D3DERR_VIEWPORTHASNODEVICE\n");
        return D3DERR_VIEWPORTHASNODEVICE;
    }
     */

    if data.is_null() || (*data).dwSize as usize != core::mem::size_of::<D3DTRANSFORMDATA>() as usize {
        WARN!("Transform data is NULL or size is incorrect, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }
    if (dwFlags & (D3DTRANSFORM_UNCLIPPED | D3DTRANSFORM_CLIPPED)) == 0 {
        WARN!("No clipping flag passed, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if activate != 0 {
        viewport_activate(viewport, (TRUE) as BOOL);
    }

    state = (*device).stateblock_state;
    multiply_matrix(
        core::ptr::addr_of_mut!(mat),
        core::ptr::addr_of!((*state).transforms[(WINED3D_TS_VIEW) as usize]),
        core::ptr::addr_of!((*state).transforms[WINED3D_TS_WORLD_MATRIX!(0) as usize]),
    );
    multiply_matrix(
        core::ptr::addr_of_mut!(mat),
        core::ptr::addr_of!((*state).transforms[(WINED3D_TS_PROJECTION) as usize]),
        core::ptr::addr_of_mut!(mat),
    );

    /* The pointer is not tested against NULL on Windows. */
    if (dwFlags & D3DTRANSFORM_CLIPPED) != 0 {
        *offscreen = !(0);
    } else {
        *offscreen = 0;
    }

    outH = (*data).lpHOut;
    i = 0;
    while i < dwVertexCount {
        'vertex: {
            in_ = ((*data).lpIn as *mut core::ffi::c_char)
                .offset(((*data).dwInSize as u32).wrapping_mul(i) as isize)
                as *mut transform_vertices_vertex;
            out = ((*data).lpOut as *mut core::ffi::c_char)
                .offset(((*data).dwOutSize as u32).wrapping_mul(i) as isize)
                as *mut transform_vertices_vertex;

            x = (*in_).x * mat._11 + (*in_).y * mat._21 + (*in_).z * mat._31 + mat._41;
            y = (*in_).x * mat._12 + (*in_).y * mat._22 + (*in_).z * mat._32 + mat._42;
            z = (*in_).x * mat._13 + (*in_).y * mat._23 + (*in_).z * mat._33 + mat._43;
            w = (*in_).x * mat._14 + (*in_).y * mat._24 + (*in_).z * mat._34 + mat._44;

            if (dwFlags & D3DTRANSFORM_CLIPPED) != 0 {
                /* If clipping is enabled, Windows assumes that outH is
                 * a valid pointer. */
                (*outH.offset(i as isize)).__bindgen_anon_1.hx =
                    ((x - (*device).legacy_clipspace._41 * w) / (*device).legacy_clipspace._11)
                        as D3DVALUE;
                (*outH.offset(i as isize)).__bindgen_anon_2.hy =
                    ((y - (*device).legacy_clipspace._42 * w) / (*device).legacy_clipspace._22)
                        as D3DVALUE;
                (*outH.offset(i as isize)).__bindgen_anon_3.hz =
                    ((z - (*device).legacy_clipspace._43 * w) / (*device).legacy_clipspace._33)
                        as D3DVALUE;

                (*outH.offset(i as isize)).dwFlags = 0;
                if x > w {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_RIGHT;
                }
                if x < -w {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_LEFT;
                }
                if y > w {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_TOP;
                }
                if y < -w {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_BOTTOM;
                }
                if z < 0.0f32 {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_FRONT;
                }
                if z > w {
                    (*outH.offset(i as isize)).dwFlags |= D3DCLIP_BACK;
                }

                *offscreen &= (*outH.offset(i as isize)).dwFlags;

                if (*outH.offset(i as isize)).dwFlags != 0 {
                    /* Looks like native just drops the vertex, leaves whatever data
                     * it has in the output buffer and goes on with the next vertex.
                     * The exact scheme hasn't been figured out yet, but windows
                     * definitely writes something there.
                     */
                    (*out).x = x;
                    (*out).y = y;
                    (*out).z = z;
                    (*out).w = w;
                    break 'vertex;
                }
            }

            w = 1 as f32 / w;
            x *= w;
            y *= w;
            z *= w;

            (*out).x = (((x + 1.0f32) * vp.dwWidth as f32) as f64 * 0.5f64
                + vp.dwX as f64) as f32;
            (*out).y = (((-y + 1.0f32) * vp.dwHeight as f32) as f64 * 0.5f64
                + vp.dwY as f64) as f32;
            (*out).z = z;
            (*out).w = w;
            // C struct assignment of a non-Copy union: a bitwise copy of one element.
            core::ptr::copy_nonoverlapping(&raw const (*in_).payload, &raw mut (*out).payload, 1);
        }
        i = i.wrapping_add(1);
    }

    if activate != 0 && !(*device).current_viewport.is_null() {
        viewport_activate(
            (*device).current_viewport as *mut d3d_viewport,
            (TRUE) as BOOL,
        );
    }

    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn d3d_viewport_LightElements(
    iface: *mut IDirect3DViewport3,
    element_count: DWORD,
    data: *mut D3DLIGHTDATA,
) -> HRESULT {
    // TRACE("iface %p, element_count %lu, data %p.\n", iface, element_count, data);
    DDERR_UNSUPPORTED
}

unsafe extern "system" fn d3d_viewport_SetBackground(iface: *mut IDirect3DViewport3, material: D3DMATERIALHANDLE) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut m: *mut d3d_material;

    TRACE!("iface {:p}, material {:#x}.\n", iface, material);

    wined3d_mutex_lock();

    m = ddraw_get_object(
        core::ptr::null_mut::<ddraw_handle_table>(),
        (material as DWORD).wrapping_sub(1 as DWORD),
        DDRAW_HANDLE_MATERIAL,
    ) as *mut d3d_material;
    if m.is_null() {
        WARN!("Invalid material handle {:#x}.\n", material);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    TRACE!(
        "Setting background color : {:.8e} {:.8e} {:.8e} {:.8e}.\n",
        (*m).mat.__bindgen_anon_1.diffuse.__bindgen_anon_1.r as f64,
        (*m).mat.__bindgen_anon_1.diffuse.__bindgen_anon_2.g as f64,
        (*m).mat.__bindgen_anon_1.diffuse.__bindgen_anon_3.b as f64,
        (*m).mat.__bindgen_anon_1.diffuse.__bindgen_anon_4.a as f64,
    );

    (*viewport).background = m;

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_GetBackground(
    mut iface: *mut IDirect3DViewport3,
    mut material: *mut D3DMATERIALHANDLE,
    mut valid: *mut BOOL,
) -> HRESULT {
    let mut viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    WINE_TRACE!("iface %p, material %p, valid %p.\n", iface, material, valid);
    wined3d_mutex_lock();
    if !valid.is_null() {
        *valid = !(*viewport).background.is_null() as ::core::ffi::c_int as BOOL;
    }
    if !material.is_null() {
        *material = (if !(*viewport).background.is_null() {
            (*(*viewport).background).Handle
        } else {
            0
        }) as D3DMATERIALHANDLE;
    }
    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_viewport_SetBackgroundDepth(
    iface: *mut IDirect3DViewport3,
    surface: *mut IDirectDrawSurface,
) -> HRESULT {
    // FIXME("iface %p, surface %p stub!\n", iface, surface);
    D3D_OK
}
unsafe extern "system" fn d3d_viewport_GetBackgroundDepth(
    iface: *mut IDirect3DViewport3,
    surface: *mut *mut IDirectDrawSurface,
    valid: *mut BOOL,
) -> HRESULT {
    // FIXME("iface %p, surface %p, valid %p stub!\n", iface, surface, valid);
    DD_OK
}

unsafe extern "system" fn d3d_viewport_Clear(
    iface: *mut IDirect3DViewport3,
    rect_count: DWORD,
    rects: *mut D3DRECT,
    flags: DWORD,
) -> HRESULT {
    let This: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut color: DWORD = 0x00000000;
    let mut hr: HRESULT = 0;
    let mut current_viewport: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut d3d_device3: *mut IDirect3DDevice3 = core::ptr::null_mut();

    if rects.is_null() || rect_count == 0 {
        WARN!("rect_count = %lu, rects = %p, ignoring clear\n", rect_count, rects);
        return D3D_OK;
    }

    if (*This).active_device.is_null() {
        ERR!(" Trying to clear a viewport not attached to a device!\n");
        return D3DERR_VIEWPORTHASNODEVICE;
    }
    d3d_device3 = core::ptr::addr_of_mut!((*(*This).active_device).IDirect3DDevice3_iface);

    wined3d_mutex_lock();

    if (flags & D3DCLEAR_TARGET) != 0 {
        if (*This).background.is_null() {
            WARN!("No background material set.\n");
        } else {
            // d3dtypes.h D3DRGBA is a `long`; C converts it to the DWORD color implicitly.
            color = (D3DRGBA!(
                (*(*This).background).mat.__bindgen_anon_1.diffuse.__bindgen_anon_1.r,
                (*(*This).background).mat.__bindgen_anon_1.diffuse.__bindgen_anon_2.g,
                (*(*This).background).mat.__bindgen_anon_1.diffuse.__bindgen_anon_3.b,
                (*(*This).background).mat.__bindgen_anon_1.diffuse.__bindgen_anon_4.a,
            )) as DWORD;
        }
    }

    /* Need to temporarily activate the viewport to clear it. The previously
     * active one will be restored afterwards. */
    viewport_activate(This, (TRUE) as BOOL);

    hr = IDirect3DDevice7_Clear(
        core::ptr::addr_of_mut!((*(*This).active_device).IDirect3DDevice7_iface),
        rect_count,
        rects,
        flags & (D3DCLEAR_ZBUFFER | D3DCLEAR_TARGET),
        color,
        1.0,
        0x00000000,
    );

    if IDirect3DDevice3_GetCurrentViewport(d3d_device3, core::ptr::addr_of_mut!(current_viewport))
        >= 0
    {
        let vp: *mut d3d_viewport = impl_from_IDirect3DViewport3(current_viewport);
        viewport_activate(vp, (TRUE) as BOOL);
        IDirect3DViewport3_Release(current_viewport);
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d_viewport_AddLight(viewport: *mut IDirect3DViewport3, light: *mut IDirect3DLight) -> HRESULT {
    let light_impl: *mut d3d_light = unsafe_impl_from_IDirect3DLight(light);
    let vp: *mut d3d_viewport = impl_from_IDirect3DViewport3(viewport);

    TRACE!("viewport %p, light %p.\n", viewport, light);

    wined3d_mutex_lock();

    if !(*light_impl).active_viewport.is_null() {
        wined3d_mutex_unlock();
        WARN!("Light %p is active in viewport %p.\n", light_impl, (*light_impl).active_viewport);
        return D3DERR_LIGHTHASVIEWPORT;
    }

    (*light_impl).active_viewport = vp;

    /* Add the light in the 'linked' chain */
    list_add_tail(core::ptr::addr_of_mut!((*vp).light_list), core::ptr::addr_of_mut!((*light_impl).entry));
    IDirect3DLight_AddRef(light);

    light_activate(light_impl);

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_DeleteLight(iface: *mut IDirect3DViewport3, lpDirect3DLight: *mut IDirect3DLight) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let l: *mut d3d_light = unsafe_impl_from_IDirect3DLight(lpDirect3DLight);

    wined3d_mutex_lock();

    if (*l).active_viewport != viewport {
        WARN!("Light %p active viewport is %p.\n", l, (*l).active_viewport);
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    light_deactivate(l);
    list_remove(core::ptr::addr_of_mut!((*l).entry));
    (*l).active_viewport = core::ptr::null_mut();
    IDirect3DLight_Release(lpDirect3DLight);

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_NextLight(
    iface: *mut IDirect3DViewport3,
    lpDirect3DLight: *mut IDirect3DLight,
    lplpDirect3DLight: *mut *mut IDirect3DLight,
    flags: DWORD,
) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let l: *mut d3d_light = unsafe_impl_from_IDirect3DLight(lpDirect3DLight);
    let mut entry: *mut list = core::ptr::null_mut();
    let mut hr: HRESULT;

    if lplpDirect3DLight.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    match flags {
        D3DNEXT_NEXT => {
            if l.is_null() || (*l).active_viewport != viewport {
                if !l.is_null() {
                    WARN!("Light %p active viewport is %p.\n", l, (*l).active_viewport);
                }
                entry = core::ptr::null_mut();
            } else {
                entry = list_next(&raw mut (*viewport).light_list, &raw mut (*l).entry);
            }
        }

        D3DNEXT_HEAD => {
            entry = list_head(&raw mut (*viewport).light_list);
        }

        D3DNEXT_TAIL => {
            entry = list_tail(&raw mut (*viewport).light_list);
        }

        _ => {
            entry = core::ptr::null_mut();
            WARN!("Invalid flags %#lx.\n", flags);
        }
    }

    if !entry.is_null() {
        *lplpDirect3DLight = (entry as *mut u8)
            .sub(core::mem::offset_of!(d3d_light, entry))
            as *mut d3d_light as *mut IDirect3DLight;
        IDirect3DLight_AddRef(*lplpDirect3DLight);
        hr = D3D_OK;
    } else {
        *lplpDirect3DLight = core::ptr::null_mut();
        hr = DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d_viewport_GetViewport2(iface: *mut IDirect3DViewport3, vp: *mut D3DVIEWPORT2) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut size: DWORD;

    if vp.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*viewport).version == DDRAW_VIEWPORT_VERSION_NONE {
        WARN!("Viewport data was not set.\n");
        return D3DERR_VIEWPORTDATANOTSET;
    }

    wined3d_mutex_lock();
    size = (*vp).dwSize;
    if (*viewport).version == DDRAW_VIEWPORT_VERSION_2 {
        memcpy(
            vp as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!((*viewport).viewports.vp2) as *const core::ffi::c_void,
            size as usize,
        );
    } else {
        let mut vp2: D3DVIEWPORT2 = core::mem::zeroed();

        vp2.dwSize = core::mem::size_of::<D3DVIEWPORT2>() as DWORD;
        vp2.dwX = (*viewport).viewports.vp1.dwX;
        vp2.dwY = (*viewport).viewports.vp1.dwY;
        vp2.dwWidth = (*viewport).viewports.vp1.dwWidth;
        vp2.dwHeight = (*viewport).viewports.vp1.dwHeight;

        vp2.dvClipWidth = (*viewport).viewports.vp1.dwWidth as D3DVALUE / (*viewport).viewports.vp1.dvScaleX;
        vp2.dvClipHeight = (*viewport).viewports.vp1.dwHeight as D3DVALUE / (*viewport).viewports.vp1.dvScaleY;
        vp2.dvClipX = -vp2.dvClipWidth / 2.0f32;
        vp2.dvClipY = vp2.dvClipHeight / 2.0f32;
        vp2.dvMinZ = 0.0f32;
        vp2.dvMaxZ = 1.0f32;
        memcpy(
            vp as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(vp2) as *const core::ffi::c_void,
            size as usize,
        );
    }

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        _dump_D3DVIEWPORT2(vp);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_viewport_SetViewport2(iface: *mut IDirect3DViewport3, vp: *mut D3DVIEWPORT2) -> HRESULT {
    let mut viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut device: *mut d3d_device = (*viewport).active_device;
    let mut rt_desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut current_viewport: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut surface: *mut ddraw_surface = core::ptr::null_mut();

    if vp.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*vp).dwSize as usize != core::mem::size_of::<D3DVIEWPORT2>() {
        WARN!("Invalid D3DVIEWPORT2 size %lu.\n", (*vp).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        _dump_D3DVIEWPORT2(vp);
    }

    if device.is_null() {
        WARN!("Viewport not bound to a device, returning D3DERR_VIEWPORTHASNODEVICE.\n");
        return D3DERR_VIEWPORTHASNODEVICE;
    }

    wined3d_mutex_lock();

    if (*device).version > 1 {
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
        wined3d_texture_get_sub_resource_desc((*surface).wined3d_texture, (*surface).sub_resource_idx, &raw mut rt_desc);

        if !wined3d_bound_range((*vp).dwX as core::ffi::c_uint, (*vp).dwWidth as core::ffi::c_uint, rt_desc.width)
            || !wined3d_bound_range((*vp).dwY as core::ffi::c_uint, (*vp).dwHeight as core::ffi::c_uint, rt_desc.height)
        {
            WARN!("Invalid viewport, returning DDERR_INVALIDPARAMS.\n");
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
    }

    (*viewport).version = DDRAW_VIEWPORT_VERSION_2;
    (*viewport).viewports.vp2 = *vp;

    if IDirect3DDevice3_GetCurrentViewport(&raw mut (*device).IDirect3DDevice3_iface, &raw mut current_viewport) >= 0 {
        if current_viewport == iface {
            viewport_activate(viewport, (FALSE) as BOOL);
        }
        IDirect3DViewport3_Release(current_viewport);
    }

    wined3d_mutex_unlock();

    return D3D_OK;
}
unsafe extern "system" fn d3d_viewport_SetBackgroundDepth2(
    iface: *mut IDirect3DViewport3,
    surface: *mut IDirectDrawSurface4,
) -> HRESULT {
    // FIXME("iface %p, surface %p stub!\n", iface, surface);
    D3D_OK
}
unsafe extern "system" fn d3d_viewport_GetBackgroundDepth2(
    iface: *mut IDirect3DViewport3,
    surface: *mut *mut IDirectDrawSurface4,
    valid: *mut BOOL,
) -> HRESULT {
    // FIXME("iface %p, surface %p, valid %p stub!\n", iface, surface, valid);
    D3D_OK
}

unsafe extern "system" fn d3d_viewport_Clear2(iface: *mut IDirect3DViewport3, rect_count: DWORD, rects: *mut D3DRECT, flags: DWORD, color: DWORD, depth: D3DVALUE, stencil: DWORD) -> HRESULT {
    let viewport: *mut d3d_viewport = impl_from_IDirect3DViewport3(iface);
    let mut hr: HRESULT = 0;
    let mut current_viewport: *mut IDirect3DViewport3 = core::ptr::null_mut();
    let mut d3d_device3: *mut IDirect3DDevice3 = core::ptr::null_mut();

    TRACE!("iface %p, rect_count %lu, rects %p, flags %#lx, color 0x%08lx, depth %.8e, stencil %lu.\n", iface, rect_count, rects, flags, color, depth as f64, stencil);

    if rects.is_null() || rect_count == 0 {
        WARN!("rect_count = %lu, rects = %p, ignoring clear\n", rect_count, rects);
        return D3D_OK;
    }

    wined3d_mutex_lock();

    if (*viewport).active_device.is_null() {
        WARN!("Trying to clear a viewport not attached to a device.\n");
        wined3d_mutex_unlock();
        return D3DERR_VIEWPORTHASNODEVICE;
    }
    d3d_device3 = &raw mut (*(*viewport).active_device).IDirect3DDevice3_iface;
    /* Need to temporarily activate viewport to clear it. Previously active
     * one will be restored afterwards. */
    viewport_activate(viewport, (TRUE) as BOOL);

    hr = IDirect3DDevice7_Clear(&raw mut (*(*viewport).active_device).IDirect3DDevice7_iface,
            rect_count, rects, flags, color, depth, stencil);
    if (IDirect3DDevice3_GetCurrentViewport(d3d_device3, &raw mut current_viewport)) >= 0
    {
        let vp: *mut d3d_viewport = impl_from_IDirect3DViewport3(current_viewport);
        viewport_activate(vp, (TRUE) as BOOL);
        IDirect3DViewport3_Release(current_viewport);
    }

    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DViewport3(
    iface: *mut IDirect3DViewport3,
) -> *mut d3d_viewport {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(((((*iface).lpVtbl == &raw const d3d_viewport_vtbl) as core::ffi::c_int) != 0));
    return ((iface as usize).wrapping_sub(
        core::mem::offset_of!(d3d_viewport, IDirect3DViewport3_iface),
    )) as *mut d3d_viewport;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DViewport2(iface: *mut IDirect3DViewport2) -> *mut d3d_viewport {
    /* IDirect3DViewport and IDirect3DViewport3 use the same iface. */
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(
        ((((*iface).lpVtbl
            == &raw const d3d_viewport_vtbl as *mut IDirect3DViewport2Vtbl
                as *const IDirect3DViewport2Vtbl) as core::ffi::c_int) != 0),
    );
    return ((iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(d3d_viewport, IDirect3DViewport3_iface) as ULONG_PTR))
        as *mut d3d_viewport;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DViewport(
    iface: *mut IDirect3DViewport,
) -> *mut d3d_viewport {
    /* IDirect3DViewport and IDirect3DViewport3 use the same iface. */
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(
        ((((*iface).lpVtbl as *const IDirect3DViewportVtbl
            == core::ptr::addr_of!(d3d_viewport_vtbl) as *const IDirect3DViewportVtbl)
            as core::ffi::c_int) != 0),
    );
    return ((iface as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(d3d_viewport, IDirect3DViewport3_iface) as ULONG_PTR))
        as *mut d3d_viewport;
}
#[no_mangle]
pub unsafe extern "C" fn d3d_viewport_init(viewport: *mut d3d_viewport, ddraw: *mut ddraw) {
    (*viewport).IDirect3DViewport3_iface.lpVtbl = &raw const d3d_viewport_vtbl as *const IDirect3DViewport3Vtbl;
    (*viewport).ref_ = 1;
    (*viewport).ddraw = ddraw;
    (*viewport).version = DDRAW_VIEWPORT_VERSION_NONE;
    list_init(core::ptr::addr_of_mut!((*viewport).light_list));
}

// COM vtable tables — generated from viewport.c by vtbl_gen.py
/// viewport.c:1180 `static const IDirect3DViewport3Vtbl d3d_viewport_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_viewport_vtbl: IDirect3DViewport3Vtbl = IDirect3DViewport3Vtbl {
    QueryInterface: Some(d3d_viewport_QueryInterface),
    AddRef: Some(d3d_viewport_AddRef),
    Release: Some(d3d_viewport_Release),
    Initialize: Some(d3d_viewport_Initialize),
    GetViewport: Some(d3d_viewport_GetViewport),
    SetViewport: Some(d3d_viewport_SetViewport),
    TransformVertices: Some(d3d_viewport_TransformVertices),
    LightElements: Some(d3d_viewport_LightElements),
    SetBackground: Some(d3d_viewport_SetBackground),
    GetBackground: Some(d3d_viewport_GetBackground),
    SetBackgroundDepth: Some(d3d_viewport_SetBackgroundDepth),
    GetBackgroundDepth: Some(d3d_viewport_GetBackgroundDepth),
    Clear: Some(d3d_viewport_Clear),
    AddLight: Some(d3d_viewport_AddLight),
    DeleteLight: Some(d3d_viewport_DeleteLight),
    NextLight: Some(d3d_viewport_NextLight),
    GetViewport2: Some(d3d_viewport_GetViewport2),
    SetViewport2: Some(d3d_viewport_SetViewport2),
    SetBackgroundDepth2: Some(d3d_viewport_SetBackgroundDepth2),
    GetBackgroundDepth2: Some(d3d_viewport_GetBackgroundDepth2),
    Clear2: Some(d3d_viewport_Clear2),
};
