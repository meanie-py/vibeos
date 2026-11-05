use crate::ddraw::device::IID_D3DDEVICE_WineD3D;
use crate::ddraw::ddraw::ddraw_null_wined3d_parent_ops;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

static mut ddraw_surface_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            ddraw_surface_wined3d_object_destroyed,
        ),
    }
};

static mut ddraw_texture_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            ddraw_texture_wined3d_object_destroyed,
        ),
    }
};

static mut ddraw_view_wined3d_parent_ops: wined3d_parent_ops = unsafe {
    wined3d_parent_ops {
        wined3d_object_destroyed: Some(
            view_wined3d_object_destroyed,
        ),
    }
};

#[repr(C)]
pub struct callback_info2 {
    pub callback: LPDDENUMSURFACESCALLBACK2,
    pub context: *mut core::ffi::c_void,
}

#[repr(C)]
pub struct callback_info {
    pub callback: LPDDENUMSURFACESCALLBACK,
    pub context: *mut core::ffi::c_void,
}

const __WINE_DBCL_TRACE: i32 = 3;

unsafe extern "C" fn impl_from_IDirectDrawGammaControl(
    iface: *mut IDirectDrawGammaControl,
) -> *mut ddraw_surface {
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(ddraw_surface, IDirectDrawGammaControl_iface),
    ) as *mut ddraw_surface;
}

unsafe extern "C" fn ddraw_surface_is_lost(surface: *const ddraw_surface) -> BOOL {
    return (ddraw_surface_can_be_lost(surface) != 0
        && (((*(*surface).ddraw).device_state) as u32 != DDRAW_DEVICE_STATE_OK || (*surface).is_lost() != 0)) as BOOL;
}

unsafe extern "C" fn ddraw_gdi_is_front(ddraw: *mut ddraw) -> BOOL {
    let mut surface: *mut ddraw_surface = core::ptr::null_mut();
    if (*ddraw).gdi_surface.is_null() || {
        surface = wined3d_texture_get_sub_resource_parent((*ddraw).gdi_surface, 0) as *mut ddraw_surface;
        surface.is_null()
    } {
        return (FALSE) as BOOL;
    }

    return ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_FRONTBUFFER as DWORD) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_surface_update_frontbuffer(
    surface: *mut ddraw_surface,
    mut rect: *const RECT,
    mut read: BOOL,
    swap_interval: ::core::ffi::c_uint,
) -> HRESULT {
    let mut dst_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut();
    let ddraw: *mut ddraw = (*surface).ddraw;
    let mut surface_dc: HDC = core::ptr::null_mut();
    let mut screen_dc: HDC = core::ptr::null_mut();
    let mut x: ::core::ffi::c_int = 0;
    let mut y: ::core::ffi::c_int = 0;
    let mut w: ::core::ffi::c_int = 0;
    let mut h: ::core::ffi::c_int = 0;
    let mut hr: HRESULT = 0;
    let mut ret: BOOL = 0;
    let mut r: RECT = core::mem::zeroed();

    if ((*ddraw).flags & DDRAW_SWAPPED) != 0 && read == 0 {
        (*ddraw).flags &= !DDRAW_SWAPPED;
        rect = core::ptr::null();
    }

    if rect.is_null() {
        SetRect(
            core::ptr::addr_of_mut!(r),
            0,
            0,
            ((*surface).surface_desc.dwWidth) as i32,
            ((*surface).surface_desc.dwHeight) as i32,
        );
        rect = core::ptr::addr_of!(r);
    }

    x = (*rect).left;
    y = (*rect).top;
    w = (*rect).right - (*rect).left;
    h = (*rect).bottom - (*rect).top;

    if w <= 0 || h <= 0 {
        return DD_OK;
    }

    if read == 0 && ((TRACE_ON!(__wine_dbch_fps)) != 0) {
        let time: DWORD = GetTickCount();
        (*ddraw).frames = (*ddraw).frames.wrapping_add(1);

        /* every 1.5 seconds */
        if time.wrapping_sub((*ddraw).prev_frame_time) > 1500 {
            WINE_TRACE_!(__wine_dbch_fps,
                "%p @ approx %.2ffps\n",
                ddraw,
                1000.0f64 * (*ddraw).frames as f64
                    / time.wrapping_sub((*ddraw).prev_frame_time) as f64
            );
            (*ddraw).prev_frame_time = time;
            (*ddraw).frames = 0;
        }
    }

    /* The interaction between ddraw and GDI drawing is not all that well
     * documented, and somewhat arcane. In ddraw exclusive mode, GDI draws
     * seemingly go to the *original* frontbuffer/primary surface, while ddraw
     * draws/flips go to the *current* frontbuffer surface. The bottom line is
     * that if the current frontbuffer is not the GDI frontbuffer, and there's
     * e.g. a popup window in front of the ddraw swapchain window, we can't
     * use wined3d_swapchain_present() to get the ddraw contents to the screen
     * while in exclusive mode, since it would get obscured by the popup
     * window. On the other hand, if the current frontbuffer *is* the GDI
     * frontbuffer, that's what's supposed to happen; the popup should obscure
     * (part of) the ddraw swapchain window.
     *
     * This affects the "Deer Hunter" demo, which uses a popup window and GDI
     * draws to draw part of the user interface. See also the "fswindow"
     * sample is the DirectX 7 SDK. */
    if !(*ddraw).swapchain_window.is_null()
        && (((*ddraw).cooperative_level & DDSCL_EXCLUSIVE) == 0
            || (*ddraw).swapchain_window == GetForegroundWindow()
            || ddraw_gdi_is_front(ddraw) != 0)
    {
        /* Nothing to do, we control the frontbuffer, or at least the parts we
         * care about. */
        if read != 0 {
            return DD_OK;
        }

        if swap_interval != 0 {
            dst_texture = wined3d_swapchain_get_back_buffer((*ddraw).wined3d_swapchain, 0);
        } else {
            dst_texture = wined3d_swapchain_get_front_buffer((*ddraw).wined3d_swapchain);
        }

        hr = wined3d_device_context_blt(
            (*ddraw).immediate_context,
            dst_texture,
            0,
            rect,
            ddraw_surface_get_any_texture(surface, DDRAW_SURFACE_READ),
            (*surface).sub_resource_idx,
            rect,
            0,
            core::ptr::null(),
            WINED3D_TEXF_POINT,
        );
        if hr >= 0 && swap_interval != 0 {
            hr = wined3d_swapchain_present(
                (*ddraw).wined3d_swapchain,
                rect,
                rect,
                core::ptr::null_mut(),
                swap_interval,
                0,
            );
            (*ddraw).flags |= DDRAW_SWAPPED;
        }
        return hr;
    }

    wined3d_texture = ddraw_surface_get_default_texture(
        surface,
        if read != 0 {
            if !rect.is_null() {
                DDRAW_SURFACE_RW
            } else {
                DDRAW_SURFACE_WRITE
            }
        } else {
            DDRAW_SURFACE_READ
        },
    );

    hr = wined3d_texture_get_dc(
        wined3d_texture,
        (*surface).sub_resource_idx,
        core::ptr::addr_of_mut!(surface_dc),
    );
    if hr < 0 {
        ERR!("Failed to get surface DC, hr %#lx.\n", hr);
        return hr;
    }

    if !(*surface).palette.is_null() {
        wined3d_palette_apply_to_dc((*(*surface).palette).wined3d_palette, surface_dc);
    }

    screen_dc = GetDC(core::ptr::null_mut());
    if screen_dc.is_null() {
        wined3d_texture_release_dc(wined3d_texture, (*surface).sub_resource_idx, surface_dc);
        ERR!("Failed to get screen DC.\n");
        return E_FAIL;
    }

    if read != 0 {
        ret = BitBlt(
            surface_dc,
            x,
            y,
            w,
            h,
            screen_dc,
            x,
            y,
            SRCCOPY,
        );
    } else {
        ret = BitBlt(
            screen_dc,
            x,
            y,
            w,
            h,
            surface_dc,
            x,
            y,
            SRCCOPY,
        );
    }

    ReleaseDC(core::ptr::null_mut(), screen_dc);
    wined3d_texture_release_dc(wined3d_texture, (*surface).sub_resource_idx, surface_dc);

    if ret == 0 {
        ERR!("Failed to blit to/from screen.\n");
        return E_FAIL;
    }

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface7_QueryInterface(iface: *mut IDirectDrawSurface7, riid: REFIID, obj: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    /* According to COM docs, if the QueryInterface fails, obj should be set to NULL */
    *obj = core::ptr::null_mut();

    if riid.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawSurface7) != false {
        IDirectDrawSurface7_AddRef(iface);
        *obj = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawSurface4) != false {
        IDirectDrawSurface4_AddRef(&raw mut (*This).IDirectDrawSurface4_iface);
        *obj = &raw mut (*This).IDirectDrawSurface4_iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawSurface3) != false {
        IDirectDrawSurface3_AddRef(&raw mut (*This).IDirectDrawSurface3_iface);
        *obj = &raw mut (*This).IDirectDrawSurface3_iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawSurface2) != false {
        IDirectDrawSurface2_AddRef(&raw mut (*This).IDirectDrawSurface2_iface);
        *obj = &raw mut (*This).IDirectDrawSurface2_iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawSurface) != false
        || IsEqualGUID!(riid, &raw const IID_IUnknown) != false
    {
        IDirectDrawSurface_AddRef(&raw mut (*This).IDirectDrawSurface_iface);
        *obj = &raw mut (*This).IDirectDrawSurface_iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawGammaControl) != false {
        IDirectDrawGammaControl_AddRef(&raw mut (*This).IDirectDrawGammaControl_iface);
        *obj = &raw mut (*This).IDirectDrawGammaControl_iface as *mut core::ffi::c_void;
        return S_OK;
    }

    if IsEqualGUID!(riid, &raw const IID_IDirectDrawColorControl) != false {
        WARN!("Color control not implemented.\n");
        *obj = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    if (*This).version != 7 {
        if IsEqualGUID!(riid, &raw const IID_D3DDEVICE_WineD3D) != false
            || IsEqualGUID!(riid, &raw const IID_IDirect3DHALDevice) != false
            || IsEqualGUID!(riid, &raw const IID_IDirect3DRGBDevice) != false
        {
            wined3d_mutex_lock();
            if (*This).device1.is_null() {
                let mut hr: HRESULT = 0;

                hr = d3d_device_create(
                    (*This).ddraw,
                    riid as *const GUID,
                    This,
                    &raw mut (*This).IDirectDrawSurface_iface as *mut IUnknown,
                    1 as UINT,
                    &raw mut (*This).device1,
                    &raw mut (*This).IDirectDrawSurface_iface as *mut IUnknown,
                );
                if hr < 0 {
                    (*This).device1 = core::ptr::null_mut::<d3d_device>();
                    wined3d_mutex_unlock();
                    WARN!("Failed to create device, hr %#lx.\n", hr);
                    return hr;
                }
            }
            wined3d_mutex_unlock();

            IDirect3DDevice_AddRef(&raw mut (*(*This).device1).IDirect3DDevice_iface);
            *obj = &raw mut (*(*This).device1).IDirect3DDevice_iface as *mut core::ffi::c_void;
            return S_OK;
        }

        if IsEqualGUID!(&raw const IID_IDirect3DTexture2, riid) != false {
            IDirect3DTexture2_AddRef(&raw mut (*This).IDirect3DTexture2_iface);
            *obj = &raw mut (*This).IDirect3DTexture2_iface as *mut core::ffi::c_void;
            return S_OK;
        }

        if IsEqualGUID!(&raw const IID_IDirect3DTexture, riid) != false {
            IDirect3DTexture2_AddRef((&raw mut (*This).IDirect3DTexture_iface).cast());
            *obj = &raw mut (*This).IDirect3DTexture_iface as *mut core::ffi::c_void;
            return S_OK;
        }
    }

    WARN!("%s not implemented, returning E_NOINTERFACE.\n", debugstr_guid(riid));

    if (*This).version != 7 {
        return E_INVALIDARG;
    }

    return E_NOINTERFACE;
}
unsafe extern "system" fn ddraw_surface4_QueryInterface(
    mut iface: *mut IDirectDrawSurface4,
    riid: REFIID,
    mut object: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        riid,
        object,
    );
}
unsafe extern "system" fn ddraw_surface3_QueryInterface(
    mut iface: *mut IDirectDrawSurface3,
    riid: *const IID,
    mut object: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        riid,
        object,
    );
}

unsafe extern "system" fn ddraw_surface2_QueryInterface(iface: *mut IDirectDrawSurface2, riid: *const IID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), riid, object);
}
unsafe extern "system" fn ddraw_surface1_QueryInterface(
    iface: *mut IDirectDrawSurface,
    riid: *const IID,
    object: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        riid,
        object,
    );
}
unsafe extern "system" fn ddraw_gamma_control_QueryInterface(
    mut iface: *mut IDirectDrawGammaControl,
    riid: *const IID,
    mut object: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawGammaControl(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        riid,
        object,
    );
}
unsafe extern "system" fn d3d_texture2_QueryInterface(iface: *mut IDirect3DTexture2, riid: *const IID, object: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture2(iface);
    WINE_TRACE!("iface %p, riid %s, object %p.\n", iface, debugstr_guid(riid as *const _GUID), object);
    return ddraw_surface7_QueryInterface(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), riid, object);
}
unsafe extern "system" fn d3d_texture1_QueryInterface(
    mut iface: *mut IDirect3DTexture,
    riid: *const IID,
    mut object: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);

    return ddraw_surface7_QueryInterface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        riid,
        object,
    );
}

unsafe extern "C" fn ddraw_surface_add_iface(surface: *mut ddraw_surface) {
    let mut iface_count: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*surface).iface_count) as *mut ::core::ffi::c_long,
    ) as ULONG;

    if iface_count == 1 {
        if !(*surface).ifaceToRelease.is_null() {
            IUnknown_AddRef((*surface).ifaceToRelease);
        }
        wined3d_mutex_lock();
        if !(*surface).wined3d_rtv.is_null() {
            wined3d_rendertarget_view_incref((*surface).wined3d_rtv);
        }
        wined3d_texture_incref(if !(*surface).draw_texture.is_null() {
            (*surface).draw_texture
        } else {
            (*surface).wined3d_texture
        });
        wined3d_mutex_unlock();
    }
}
unsafe extern "system" fn ddraw_surface7_AddRef(mut iface: *mut IDirectDrawSurface7) -> ULONG {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref7) as *mut i32,
    ) as ULONG;
    WINE_TRACE!("iface %p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }
    return refcount;
}

unsafe extern "system" fn ddraw_surface4_AddRef(iface: *mut IDirectDrawSurface4) -> ULONG {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref4) as *mut LONG) as ULONG;

    TRACE!("iface %p increasing refcount to %lu.\n", iface, refcount);

    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }

    return refcount;
}
unsafe extern "system" fn ddraw_surface3_AddRef(mut iface: *mut IDirectDrawSurface3) -> ULONG {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        &raw mut (*This).ref3 as *mut ::core::ffi::c_long as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("iface %p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }
    return refcount;
}

unsafe extern "system" fn ddraw_surface2_AddRef(iface: *mut IDirectDrawSurface2) -> ULONG {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut refcount: ULONG = InterlockedIncrement(
        core::ptr::addr_of_mut!((*This).ref2) as *mut ::core::ffi::c_long,
    ) as ULONG;
    WINE_TRACE!("iface %p increasing refcount to %lu.\n", iface, refcount);
    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }
    return refcount;
}

unsafe extern "system" fn ddraw_surface1_AddRef(iface: *mut IDirectDrawSurface) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).ref1) as *mut i32) as ULONG;

    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_gamma_control_AddRef(iface: *mut IDirectDrawGammaControl) -> ULONG {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawGammaControl(iface);
    let mut refcount: ULONG = InterlockedIncrement(core::ptr::addr_of_mut!((*This).gamma_count) as *mut LONG) as ULONG;

    if refcount == 1 {
        ddraw_surface_add_iface(This);
    }

    return refcount;
}
unsafe extern "system" fn d3d_texture2_AddRef(iface: *mut IDirect3DTexture2) -> ULONG {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture2(iface);

    return IUnknown_AddRef((*surface).texture_outer);
}
unsafe extern "system" fn d3d_texture1_AddRef(iface: *mut IDirect3DTexture) -> ULONG {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);

    return IUnknown_AddRef((*surface).texture_outer);
}

unsafe extern "C" fn ddraw_surface_set_palette(
    surface: *mut ddraw_surface,
    palette: *mut IDirectDrawPalette,
) -> HRESULT {
    let palette_impl: *mut ddraw_palette = unsafe_impl_from_IDirectDrawPalette(palette);
    let mut prev: *mut ddraw_palette;

    TRACE!("iface %p, palette %p.\n", surface, palette);

    if !palette_impl.is_null()
        && ((*palette_impl).flags & DDPCAPS_ALPHA) != 0
        && ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_TEXTURE) == 0
    {
        WARN!("Alpha palette set on non-texture surface, returning DDERR_INVALIDSURFACETYPE.\n");
        return DDERR_INVALIDSURFACETYPE;
    }

    if format_is_paletteindexed(&raw mut (*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat) == 0 {
        return DDERR_INVALIDPIXELFORMAT;
    }

    wined3d_mutex_lock();

    prev = (*surface).palette as *mut ddraw_palette;
    if (*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0 {
        if !prev.is_null() {
            (*prev).flags &= !DDPCAPS_PRIMARYSURFACE;
        }
        if !palette_impl.is_null() {
            (*palette_impl).flags |= DDPCAPS_PRIMARYSURFACE;
        }
        wined3d_swapchain_set_palette(
            (*(*surface).ddraw).wined3d_swapchain,
            if !palette_impl.is_null() {
                (*palette_impl).wined3d_palette
            } else {
                core::ptr::null_mut()
            },
        );
        ddraw_surface_update_frontbuffer(
            surface,
            core::ptr::null(),
            (FALSE) as BOOL,
            0,
        );
    }
    if !palette_impl.is_null() {
        IDirectDrawPalette_AddRef(&raw mut (*palette_impl).IDirectDrawPalette_iface);
    }
    if !prev.is_null() {
        IDirectDrawPalette_Release(&raw mut (*prev).IDirectDrawPalette_iface);
    }
    (*surface).palette = palette_impl as *mut ddraw_palette;

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "C" fn ddraw_surface_cleanup(surface: *mut ddraw_surface) {
    let mut surf: *mut ddraw_surface = core::ptr::null_mut();
    let mut i: UINT;

    TRACE!("surface %p.\n", surface);

    /* The refcount test shows that the palette is detached when the surface
     * is destroyed. */
    ddraw_surface_set_palette(surface, core::ptr::null_mut());

    /* Loop through all complex attached surfaces and destroy them.
     *
     * Yet again, only the root can have more than one complexly attached
     * surface, all the others have a total of one. */
    i = 0;
    while i < MAX_COMPLEX_ATTACHED {
        if (*surface).complex_array[i as usize].is_null() {
            break;
        }

        surf = (*surface).complex_array[i as usize];
        (*surface).complex_array[i as usize] = core::ptr::null_mut();
        if (*surf).is_root() == 0 {
            let texture: *mut ddraw_texture =
                wined3d_texture_get_parent((*surf).wined3d_texture) as *mut ddraw_texture;
            let wined3d_device: *mut wined3d_device = (*texture).wined3d_device;
            let root: *mut ddraw_surface = (*texture).root;

            ddraw_surface_cleanup(surf);

            if surf == root {
                wined3d_device_decref(wined3d_device);
            }
        }
        i = i.wrapping_add(1);
    }

    if !(*surface).device1.is_null() {
        IUnknown_Release(core::ptr::addr_of_mut!((*(*surface).device1).IUnknown_inner));
    }

    if (*surface).iface_count > 1 {
        /* This can happen when a complex surface is destroyed, because the
         * 2nd surface was addref()ed when the app called
         * GetAttachedSurface(). */
        WARN!(
            "Destroying surface %p with refcounts 7: %lu 4: %lu 3: %lu 2: %lu 1: %lu.\n",
            surface,
            (*surface).ref7,
            (*surface).ref4,
            (*surface).ref3,
            (*surface).ref2,
            (*surface).ref1
        );
    }

    if !(*surface).wined3d_rtv.is_null() {
        wined3d_rendertarget_view_decref((*surface).wined3d_rtv);
    }
    wined3d_texture_decref(if !(*surface).draw_texture.is_null() {
        (*surface).draw_texture
    } else {
        (*surface).wined3d_texture
    });
}

unsafe extern "C" fn ddraw_surface_release_iface(This: *mut ddraw_surface) -> ULONG {
    let mut iface_count: ULONG;

    /* Prevent the surface from being destroyed if it's still attached to
     * another surface. It will be destroyed when the root is destroyed. */
    if (*This).iface_count == 1 && !(*This).attached_iface.is_null() {
        IUnknown_AddRef((*This).attached_iface);
    }
    iface_count = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).iface_count) as *mut LONG,
    ) as ULONG;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        TRACE!("%p decreasing iface count to %lu.\n", This, iface_count);
    }

    if iface_count == 0 {
        let texture: *mut ddraw_texture =
            wined3d_texture_get_parent((*This).wined3d_texture) as *mut ddraw_texture;
        let wined3d_device: *mut wined3d_device = (*texture).wined3d_device;
        let release_iface: *mut IUnknown = (*This).ifaceToRelease;

        /* Complex attached surfaces are destroyed implicitly when the root is released */
        wined3d_mutex_lock();
        if (*This).is_root() == 0 {
            WARN!("(%p) Attempt to destroy a surface that is not a complex root\n", This);
            wined3d_mutex_unlock();
            return iface_count;
        }
        if ((*This).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0)
            && ((*(*This).ddraw).flags & DDRAW_RESTORE_MODE != 0)
            && !(*(*This).ddraw).swapchain_window.is_null()
        {
            RedrawWindow(
                (*(*This).ddraw).swapchain_window,
                core::ptr::null(),
                core::ptr::null_mut(),
                (RDW_INVALIDATE | RDW_ERASE | RDW_FRAME) as UINT,
            );
        }
        ddraw_surface_cleanup(This);
        wined3d_mutex_unlock();

        if !release_iface.is_null() {
            IUnknown_Release(release_iface);
        }
        /* Release the device only after anything that may reference it (the
         * wined3d texture and rendertarget view in particular) is released. */
        wined3d_device_decref(wined3d_device);
    }

    return iface_count;
}

unsafe extern "system" fn ddraw_surface7_Release(iface: *mut IDirectDrawSurface7) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut refcount: ULONG =
        InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref7) as *mut i32) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_surface4_Release(iface: *mut IDirectDrawSurface4) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref4)) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_surface3_Release(iface: *mut IDirectDrawSurface3) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).ref3) as *mut i32) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_surface2_Release(iface: *mut IDirectDrawSurface2) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref2) as *mut core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_surface1_Release(iface: *mut IDirectDrawSurface) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let refcount: ULONG = InterlockedDecrement(
        core::ptr::addr_of_mut!((*This).ref1) as *mut core::ffi::c_long,
    ) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn ddraw_gamma_control_Release(iface: *mut IDirectDrawGammaControl) -> ULONG {
    let This: *mut ddraw_surface = impl_from_IDirectDrawGammaControl(iface);
    let refcount: ULONG = InterlockedDecrement(core::ptr::addr_of_mut!((*This).gamma_count) as *mut core::ffi::c_long) as ULONG;

    if refcount == 0 {
        ddraw_surface_release_iface(This);
    }

    return refcount;
}

unsafe extern "system" fn d3d_texture2_Release(iface: *mut IDirect3DTexture2) -> ULONG {
    let surface: *mut ddraw_surface = impl_from_IDirect3DTexture2(iface);

    return IUnknown_Release((*surface).texture_outer);
}

unsafe extern "system" fn d3d_texture1_Release(iface: *mut IDirect3DTexture) -> ULONG {
    let surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);

    return IUnknown_Release((*surface).texture_outer);
}
unsafe extern "system" fn ddraw_surface7_GetAttachedSurface(
    mut iface: *mut IDirectDrawSurface7,
    mut caps: *mut DDSCAPS2,
    mut surface: *mut *mut IDirectDrawSurface7,
) -> HRESULT {
    let mut head_surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut surf: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut our_caps: DDSCAPS2 = DDSCAPS2 {
        dwCaps: 0,
        dwCaps2: 0,
        dwCaps3: 0,
        __bindgen_anon_1: _DDSCAPS2__bindgen_ty_1 { dwCaps4: 0 },
    };
    let mut i: core::ffi::c_int = 0;

    TRACE!(
        "iface %p, caps %p, attachment %p.\n",
        iface,
        caps,
        surface
    );

    if ddraw_surface_is_lost(head_surface) != 0 {
        WARN!("Surface %p is lost.\n", head_surface);

        *surface = core::ptr::null_mut::<IDirectDrawSurface7>();
        return DDERR_SURFACELOST;
    }

    wined3d_mutex_lock();

    if (*head_surface).version < 7 {
        /* Earlier dx apps put garbage into these members, clear them */
        our_caps.dwCaps = (*caps).dwCaps;
        our_caps.dwCaps2 = 0;
        our_caps.dwCaps3 = 0;
        our_caps.__bindgen_anon_1.dwCaps4 = 0;
    } else {
        our_caps = *caps;
    }

    TRACE!(
        "head_surface %p, looking for caps %#lx, %#lx, %#lx, %#lx.\n",
        head_surface,
        our_caps.dwCaps,
        our_caps.dwCaps2,
        our_caps.dwCaps3,
        our_caps.__bindgen_anon_1.dwCaps4
    );

    /* FIXME: Better debugging */

    i = 0;
    while i < (MAX_COMPLEX_ATTACHED as _) {
        surf = (*head_surface).complex_array[i as usize];
        if surf.is_null() {
            break;
        }

        TRACE!(
            "Surface %p, caps %#lx, %#lx, %#lx, %#lx.\n",
            surf,
            (*surf).surface_desc.ddsCaps.dwCaps,
            (*surf).surface_desc.ddsCaps.dwCaps2,
            (*surf).surface_desc.ddsCaps.dwCaps3,
            (*surf).surface_desc.ddsCaps.__bindgen_anon_1.dwCaps4
        );

        if ((*surf).surface_desc.ddsCaps.dwCaps & our_caps.dwCaps) == our_caps.dwCaps
            && ((*surf).surface_desc.ddsCaps.dwCaps2 & our_caps.dwCaps2) == our_caps.dwCaps2
        {
            /* MSDN: "This method fails if more than one surface is attached
             * that matches the capabilities requested."
             *
             * Not sure how to test this.
             */

            TRACE!(
                "head_surface %p, returning surface %p.\n",
                head_surface,
                surf
            );
            *surface = core::ptr::addr_of_mut!((*surf).IDirectDrawSurface7_iface);
            ddraw_surface7_AddRef(*surface);
            wined3d_mutex_unlock();

            return DD_OK;
        }
        i += 1;
    }

    /* Next, look at the attachment chain */
    surf = head_surface;

    loop {
        surf = (*surf).next_attached;
        if surf.is_null() {
            break;
        }

        TRACE!(
            "Surface %p, caps %#lx, %#lx, %#lx, %#lx.\n",
            surf,
            (*surf).surface_desc.ddsCaps.dwCaps,
            (*surf).surface_desc.ddsCaps.dwCaps2,
            (*surf).surface_desc.ddsCaps.dwCaps3,
            (*surf).surface_desc.ddsCaps.__bindgen_anon_1.dwCaps4
        );

        if ((*surf).surface_desc.ddsCaps.dwCaps & our_caps.dwCaps) == our_caps.dwCaps
            && ((*surf).surface_desc.ddsCaps.dwCaps2 & our_caps.dwCaps2) == our_caps.dwCaps2
        {
            TRACE!(
                "head_surface %p, returning surface %p.\n",
                head_surface,
                surf
            );
            *surface = core::ptr::addr_of_mut!((*surf).IDirectDrawSurface7_iface);
            ddraw_surface7_AddRef(*surface);
            wined3d_mutex_unlock();
            return DD_OK;
        }
    }

    TRACE!(
        "head_surface %p, didn't find a valid surface.\n",
        head_surface
    );

    wined3d_mutex_unlock();

    *surface = core::ptr::null_mut::<IDirectDrawSurface7>();
    return DDERR_NOTFOUND;
}

unsafe extern "system" fn ddraw_surface4_GetAttachedSurface(
    iface: *mut IDirectDrawSurface4,
    caps: *mut DDSCAPS2,
    attachment: *mut *mut IDirectDrawSurface4,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut attachment_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut attachment7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, caps %p, attachment %p.\n",
        iface,
        caps,
        attachment
    );

    hr = ddraw_surface7_GetAttachedSurface(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        caps,
        &raw mut attachment7,
    );
    if (hr) < 0 {
        *attachment = core::ptr::null_mut();
        return hr;
    }
    attachment_impl = impl_from_IDirectDrawSurface7(attachment7);
    *attachment = &raw mut (*attachment_impl).IDirectDrawSurface4_iface;
    ddraw_surface4_AddRef(*attachment);
    ddraw_surface7_Release(attachment7);

    return hr;
}

unsafe extern "system" fn ddraw_surface3_GetAttachedSurface(
    iface: *mut IDirectDrawSurface3,
    caps: *mut DDSCAPS,
    attachment: *mut *mut IDirectDrawSurface3,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut attachment_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut attachment7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT;

    TRACE!(
        "iface %p, caps %p, attachment %p.\n",
        iface,
        caps,
        attachment
    );

    caps2.dwCaps = (*caps).dwCaps;
    caps2.dwCaps2 = 0;
    caps2.dwCaps3 = 0;
    caps2.__bindgen_anon_1.dwCaps4 = 0;

    hr = ddraw_surface7_GetAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        &mut caps2,
        &mut attachment7,
    );
    if hr < 0 {
        *attachment = core::ptr::null_mut();
        return hr;
    }
    attachment_impl = impl_from_IDirectDrawSurface7(attachment7);
    *attachment = core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface3_iface);
    ddraw_surface3_AddRef(*attachment);
    ddraw_surface7_Release(attachment7);

    return hr;
}
unsafe extern "system" fn ddraw_surface2_GetAttachedSurface(iface: *mut IDirectDrawSurface2, caps: *mut DDSCAPS, attachment: *mut *mut IDirectDrawSurface2) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut attachment_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut attachment7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, caps %p, attachment %p.\n", iface, caps, attachment);

    caps2.dwCaps = (*caps).dwCaps;
    caps2.dwCaps2 = 0 as DWORD;
    caps2.dwCaps3 = 0 as DWORD;
    caps2.__bindgen_anon_1.dwCaps4 = 0 as DWORD;

    hr = ddraw_surface7_GetAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        &mut caps2,
        &mut attachment7,
    );
    if hr < (0 as HRESULT) {
        *attachment = core::ptr::null_mut();
        return hr;
    }
    attachment_impl = impl_from_IDirectDrawSurface7(attachment7);
    *attachment = core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface2_iface);
    ddraw_surface2_AddRef(*attachment);
    ddraw_surface7_Release(attachment7);

    return hr;
}
unsafe extern "system" fn ddraw_surface1_GetAttachedSurface(
    iface: *mut IDirectDrawSurface,
    caps: *mut DDSCAPS,
    attachment: *mut *mut IDirectDrawSurface,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut attachment_impl: *mut ddraw_surface = core::ptr::null_mut();
    let mut attachment7: *mut IDirectDrawSurface7 = core::ptr::null_mut();
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    caps2.dwCaps = (*caps).dwCaps;
    caps2.dwCaps2 = 0;
    caps2.dwCaps3 = 0;
    caps2.__bindgen_anon_1.dwCaps4 = 0;

    hr = ddraw_surface7_GetAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        core::ptr::addr_of_mut!(caps2),
        core::ptr::addr_of_mut!(attachment7),
    );
    if hr < 0 {
        *attachment = core::ptr::null_mut();
        return hr;
    }
    attachment_impl = impl_from_IDirectDrawSurface7(attachment7);
    *attachment = core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface_iface)
        as *mut IDirectDrawSurface;
    ddraw_surface1_AddRef(*attachment);
    ddraw_surface7_Release(attachment7);

    return hr;
}

unsafe extern "C" fn surface_lock(
    surface: *mut ddraw_surface,
    rect: *mut RECT,
    surface_desc: *mut DDSURFACEDESC2,
    surface_desc_size: ::core::ffi::c_uint,
    flags: DWORD,
    h: HANDLE,
) -> HRESULT {
    let mut map_desc: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut wined3d_flags: ::core::ffi::c_uint = 0;
    let mut box_: wined3d_box = wined3d_box {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
        front: 0,
        back: 0,
    };
    let mut hr: HRESULT = DD_OK;

    /* surface->surface_desc.dwWidth and dwHeight are changeable, thus lock */
    wined3d_mutex_lock();

    /* Should I check for the handle to be NULL?
     *
     * The DDLOCK flags and the D3DLOCK flags are equal
     * for the supported values. The others are ignored by WineD3D
     */

    /* Windows zeroes this if the rect is invalid */
    (*surface_desc).lpSurface = NULL as LPVOID;

    if !rect.is_null() {
        if ((*rect).left < 0) || ((*rect).top < 0)
            || ((*rect).left > (*rect).right)
            || ((*rect).right as DWORD > (*surface).surface_desc.dwWidth)
            || ((*rect).top > (*rect).bottom)
            || ((*rect).bottom as DWORD > (*surface).surface_desc.dwHeight)
        {
            WARN!("Trying to lock an invalid rectangle, returning DDERR_INVALIDPARAMS\n");
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
        wined3d_box_set(
            ::core::ptr::addr_of_mut!(box_),
            (*rect).left as ::core::ffi::c_uint,
            (*rect).top as ::core::ffi::c_uint,
            (*rect).right as ::core::ffi::c_uint,
            (*rect).bottom as ::core::ffi::c_uint,
            0,
            1,
        );
    }

    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0 {
        hr = ddraw_surface_update_frontbuffer(surface, rect, (TRUE) as BOOL, 0);
    }
    if hr >= 0 {
        wined3d_flags = wined3dmapflags_from_ddrawmapflags(flags as ::core::ffi::c_uint);
        hr = wined3d_resource_map(
            wined3d_texture_get_resource(ddraw_surface_get_default_texture(
                surface,
                (if (wined3d_flags & WINED3D_MAP_WRITE) != 0 {
                    DDRAW_SURFACE_RW
                } else {
                    DDRAW_SURFACE_READ
                }) as ::core::ffi::c_uint,
            )),
            (*surface).sub_resource_idx,
            ::core::ptr::addr_of_mut!(map_desc),
            if !rect.is_null() {
                ::core::ptr::addr_of_mut!(box_)
            } else {
                ::core::ptr::null_mut::<wined3d_box>()
            },
            wined3d_flags as uint32_t,
        );
    }
    if hr < 0 {
        wined3d_mutex_unlock();
        match hr {
            /* D3D8 and D3D9 return the general D3DERR_INVALIDCALL error, but ddraw has a more
             * specific error. But since wined3d returns that error in this only occasion,
             * keep d3d8 and d3d9 free from the return value override. There are many different
             * places where d3d8/9 would have to catch the DDERR_SURFACEBUSY, it is much easier
             * to do it in one place in ddraw.
             */
            WINED3DERR_INVALIDCALL => {
                return DDERR_SURFACEBUSY;
            }
            _ => {
                return hr;
            }
        }
    }

    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0 {
        if (flags & DDLOCK_READONLY) != 0 {
            SetRectEmpty(::core::ptr::addr_of_mut!((*(*surface).ddraw).primary_lock));
        } else if !rect.is_null() {
            (*(*surface).ddraw).primary_lock = *rect;
        } else {
            SetRect(
                ::core::ptr::addr_of_mut!((*(*surface).ddraw).primary_lock),
                0,
                0,
                (*surface).surface_desc.dwWidth as ::core::ffi::c_int,
                (*surface).surface_desc.dwHeight as ::core::ffi::c_int,
            );
        }
    }

    /* Windows does not set DDSD_LPSURFACE on locked surfaces. */
    DD_STRUCT_COPY_BYSIZE_!(
        surface_desc,
        ::core::ptr::addr_of_mut!((*surface).surface_desc),
        surface_desc_size,
        (*surface).surface_desc.dwSize,
    );
    (*surface_desc).lpSurface = map_desc.data as LPVOID;

    if TRACE_ON!(__wine_dbch_ddraw) != 0 {
        DDRAW_dump_surface_desc(surface_desc);
    }

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "C" fn surface_validate_lock_desc(
    surface: *mut ddraw_surface,
    desc: *const DDSURFACEDESC,
    size: *mut ::core::ffi::c_uint,
) -> BOOL {
    if desc.is_null() {
        return (FALSE) as BOOL;
    }

    if (*desc).dwSize as usize == ::core::mem::size_of::<DDSURFACEDESC>() as usize
        || (*desc).dwSize as usize == ::core::mem::size_of::<DDSURFACEDESC2>() as usize
    {
        *size = (*desc).dwSize as ::core::ffi::c_uint;
        return (TRUE) as BOOL;
    }

    if (*surface).version == 7
        && (*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_TEXTURE as DWORD != 0
        && (*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY as DWORD == 0
    {
        if (*desc).dwSize as usize >= ::core::mem::size_of::<DDSURFACEDESC2>() as usize {
            *size = ::core::mem::size_of::<DDSURFACEDESC2>() as ::core::ffi::c_uint;
        } else {
            *size = ::core::mem::size_of::<DDSURFACEDESC>() as ::core::ffi::c_uint;
        }
        return (TRUE) as BOOL;
    }

    WARN!("Invalid structure size %lu.\n", (*desc).dwSize);
    return (FALSE) as BOOL;
}

unsafe extern "system" fn ddraw_surface7_Lock(
    iface: *mut IDirectDrawSurface7,
    rect: *mut RECT,
    surface_desc: *mut DDSURFACEDESC2,
    flags: DWORD,
    h: HANDLE,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut surface_desc_size: u32 = 0;

    if surface_validate_lock_desc(
        surface,
        surface_desc as *mut DDSURFACEDESC,
        core::ptr::addr_of_mut!(surface_desc_size),
    ) == 0
    {
        return DDERR_INVALIDPARAMS;
    }

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface is lost.\n");
        return DDERR_SURFACELOST;
    }

    return surface_lock(surface, rect, surface_desc, surface_desc_size, flags, h);
}

unsafe extern "system" fn ddraw_surface4_Lock(
    iface: *mut IDirectDrawSurface4,
    rect: *mut RECT,
    surface_desc: *mut DDSURFACEDESC2,
    flags: DWORD,
    h: HANDLE,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut surface_desc_size: u32 = 0;

    TRACE!(
        "iface %p, rect %s, surface_desc %p, flags %#lx, h %p.\n",
        iface,
        wine_dbgstr_rect(rect),
        surface_desc,
        flags,
        h
    );

    if surface_validate_lock_desc(
        surface,
        surface_desc as *mut DDSURFACEDESC,
        &raw mut surface_desc_size,
    ) == 0
    {
        return DDERR_INVALIDPARAMS;
    }

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface is lost.\n");
        return DDERR_SURFACELOST;
    }

    return surface_lock(surface, rect, surface_desc, surface_desc_size, flags, h);
}

unsafe extern "C" fn ddraw_surface_lock_ddsd(
    surface: *mut ddraw_surface,
    rect: *mut RECT,
    surface_desc: *mut DDSURFACEDESC,
    flags: DWORD,
    h: HANDLE,
) -> HRESULT {
    let mut surface_desc_size: core::ffi::c_uint = 0;
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();
    let mut hr: HRESULT;

    if surface_validate_lock_desc(surface, surface_desc, core::ptr::addr_of_mut!(surface_desc_size)) == 0 {
        return DDERR_INVALIDPARAMS;
    }

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface is lost.\n");
        return DDERR_SURFACELOST;
    }

    surface_desc2.dwSize = (*surface_desc).dwSize;
    surface_desc2.dwFlags = 0;
    hr = surface_lock(
        surface,
        rect,
        core::ptr::addr_of_mut!(surface_desc2),
        surface_desc_size,
        flags,
        h,
    );
    DDSD2_to_DDSD(core::ptr::addr_of_mut!(surface_desc2), surface_desc);
    (*surface_desc).dwSize = surface_desc2.dwSize;
    return hr;
}

unsafe extern "system" fn ddraw_surface3_Lock(
    mut iface: *mut IDirectDrawSurface3,
    mut rect: *mut RECT,
    mut surface_desc: *mut DDSURFACEDESC,
    mut flags: DWORD,
    mut h: HANDLE,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, rect %s, surface_desc %p, flags %#lx, h %p.\n", iface, wine_dbgstr_rect(rect), surface_desc, flags, h);
    return ddraw_surface_lock_ddsd(surface, rect, surface_desc, flags, h);
}

unsafe extern "system" fn ddraw_surface2_Lock(
    iface: *mut IDirectDrawSurface2,
    rect: *mut RECT,
    surface_desc: *mut DDSURFACEDESC,
    flags: DWORD,
    h: HANDLE,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    WINE_TRACE!("iface %p, rect %s, surface_desc %p, flags %#lx, h %p.\n", iface, wine_dbgstr_rect(rect), surface_desc, flags, h);

    return ddraw_surface_lock_ddsd(surface, rect, surface_desc, flags, h);
}
unsafe extern "system" fn ddraw_surface1_Lock(iface: *mut IDirectDrawSurface, rect: *mut RECT, surface_desc: *mut DDSURFACEDESC, flags: DWORD, h: HANDLE) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    return ddraw_surface_lock_ddsd(surface, rect, surface_desc, flags, h);
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface1_Unlock(
    mut iface: *mut IDirectDrawSurface,
    mut data: *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut hr: HRESULT = 0;
    WINE_TRACE!("iface %p, data %p.\n", iface, data);
    wined3d_mutex_lock();
    hr = wined3d_resource_unmap(
        wined3d_texture_get_resource(ddraw_surface_get_default_texture(surface, 0)),
        (*surface).sub_resource_idx,
    );
    if hr >= 0
        && ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0
    {
        hr = ddraw_surface_update_frontbuffer(
            surface,
            core::ptr::addr_of_mut!((*(*surface).ddraw).primary_lock),
            (FALSE) as BOOL,
            0,
        );
    }
    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface7_Unlock(iface: *mut IDirectDrawSurface7, rect: *mut RECT) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    WINE_TRACE!("iface %p, rect %s.\n", iface, wine_dbgstr_rect(rect));
    return ddraw_surface1_Unlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface_iface), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface4_Unlock(iface: *mut IDirectDrawSurface4, rect: *mut RECT) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface1_Unlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface_iface), core::ptr::null_mut());
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface3_Unlock(iface: *mut IDirectDrawSurface3, data: *mut core::ffi::c_void) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface1_Unlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface_iface), data);
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface2_Unlock(
    mut iface: *mut IDirectDrawSurface2,
    mut data: *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, data %p.\n", iface, data);
    return ddraw_surface1_Unlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface_iface), data);
}

unsafe extern "C" fn ddraw_swap_interval_from_flags(flags: DWORD) -> ::core::ffi::c_uint {
    if flags & (DDFLIP_NOVSYNC as DWORD) != 0 {
        return 0 as ::core::ffi::c_uint;
    }
    let masked: DWORD = flags & ((DDFLIP_INTERVAL2 | DDFLIP_INTERVAL3 | DDFLIP_INTERVAL4) as DWORD);
    if masked == (DDFLIP_INTERVAL2 as DWORD) {
        return 2 as ::core::ffi::c_uint;
    } else if masked == (DDFLIP_INTERVAL3 as DWORD) {
        return 3 as ::core::ffi::c_uint;
    } else if masked == (DDFLIP_INTERVAL4 as DWORD) {
        return 4 as ::core::ffi::c_uint;
    } else {
        return 1 as ::core::ffi::c_uint;
    }
}

unsafe extern "C" fn ddraw_texture_rename_to(
    dst_texture: *mut ddraw_texture,
    wined3d_texture: *mut wined3d_texture,
    draw_texture: *mut wined3d_texture,
    rtv: *mut wined3d_rendertarget_view,
    texture_memory: *mut core::ffi::c_void,
    current_rtv: *mut wined3d_rendertarget_view,
) {
    let mut rtv = rtv;
    let dst_surface: *mut ddraw_surface = (*dst_texture).root;

    /* We don't have to worry about potential texture bindings, since
     * flippable surfaces can never be textures. */

    if current_rtv == (*dst_surface).wined3d_rtv {
        wined3d_device_context_set_rendertarget_views(
            (*(*dst_surface).ddraw).immediate_context,
            0,
            1,
            &raw mut rtv,
            (FALSE) as BOOL,
        );
    }
    wined3d_rendertarget_view_set_parent(
        rtv,
        dst_surface as *mut core::ffi::c_void,
        &raw const ddraw_view_wined3d_parent_ops,
    );
    (*dst_surface).wined3d_rtv = rtv;

    if (*dst_surface).sub_resource_idx != 0 {
        ERR!("Invalid sub-resource index %u for surface %p.\n", (*dst_surface).sub_resource_idx, dst_surface);
    }

    if !draw_texture.is_null() {
        wined3d_texture_set_sub_resource_parent(
            draw_texture,
            0,
            dst_surface as *mut core::ffi::c_void,
            &raw const ddraw_surface_wined3d_parent_ops,
        );
        wined3d_texture_set_sub_resource_parent(
            wined3d_texture,
            0,
            dst_surface as *mut core::ffi::c_void,
            &raw const ddraw_null_wined3d_parent_ops,
        );
        wined3d_resource_set_parent(
            wined3d_texture_get_resource(draw_texture),
            dst_texture as *mut core::ffi::c_void,
            &raw const ddraw_texture_wined3d_parent_ops,
        );
        wined3d_resource_set_parent(
            wined3d_texture_get_resource(wined3d_texture),
            dst_texture as *mut core::ffi::c_void,
            &raw const ddraw_null_wined3d_parent_ops,
        );
    } else {
        wined3d_texture_set_sub_resource_parent(
            wined3d_texture,
            0,
            dst_surface as *mut core::ffi::c_void,
            &raw const ddraw_surface_wined3d_parent_ops,
        );
        wined3d_resource_set_parent(
            wined3d_texture_get_resource(wined3d_texture),
            dst_texture as *mut core::ffi::c_void,
            &raw const ddraw_texture_wined3d_parent_ops,
        );
    }

    (*dst_surface).wined3d_texture = wined3d_texture;
    (*dst_surface).draw_texture = draw_texture;

    (*dst_texture).texture_memory = texture_memory;
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface1_Flip(
    iface: *mut IDirectDrawSurface,
    src: *mut IDirectDrawSurface,
    flags: DWORD,
) -> HRESULT {
    let mut dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(src);
    let mut dst_ddraw_texture: *mut ddraw_texture = core::ptr::null_mut();
    let mut src_ddraw_texture: *mut ddraw_texture = core::ptr::null_mut();
    let mut tmp_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut current_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut draw_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut caps: DDSCAPS = DDSCAPS {
        dwCaps: (DDSCAPS_FLIP) as DWORD,
    };
    let mut current: *mut IDirectDrawSurface = core::ptr::null_mut();
    let mut texture_memory: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if src == iface
        || ((*dst_impl).surface_desc.ddsCaps.dwCaps
            & ((DDSCAPS_FRONTBUFFER | DDSCAPS_OVERLAY) as DWORD))
            == 0
    {
        return DDERR_NOTFLIPPABLE;
    }

    if ddraw_surface_is_lost(dst_impl) != 0 {
        return DDERR_SURFACELOST;
    }

    wined3d_mutex_lock();

    if ((*dst_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0
        && ((*(*dst_impl).ddraw).cooperative_level & (DDSCL_EXCLUSIVE as DWORD)) == 0
    {
        WARN!("Not in exclusive mode.\n");
        wined3d_mutex_unlock();
        return DDERR_NOEXCLUSIVEMODE;
    }

    tmp_rtv = ddraw_surface_get_rendertarget_view(dst_impl);
    texture = (*dst_impl).wined3d_texture;
    dst_ddraw_texture = wined3d_texture_get_parent((*dst_impl).wined3d_texture) as *mut ddraw_texture;
    texture_memory = (*dst_ddraw_texture).texture_memory;
    draw_texture = (*dst_impl).draw_texture;
    current_rtv = wined3d_device_context_get_rendertarget_view(
        (*(*dst_impl).ddraw).immediate_context,
        0,
    );

    if !src_impl.is_null() {
        current = iface;
        while current != src {
            hr = ddraw_surface1_GetAttachedSurface(current, core::ptr::addr_of_mut!(caps), core::ptr::addr_of_mut!(current));
            if hr < 0 {
                WARN!(
                    "Surface %p is not on the same flip chain as surface %p.\n",
                    src,
                    iface
                );
                wined3d_mutex_unlock();
                return DDERR_NOTFLIPPABLE;
            }
            ddraw_surface1_Release(current);
            if current == iface {
                WARN!(
                    "Surface %p is not on the same flip chain as surface %p.\n",
                    src,
                    iface
                );
                wined3d_mutex_unlock();
                return DDERR_NOTFLIPPABLE;
            }
        }

        src_ddraw_texture =
            wined3d_texture_get_parent((*src_impl).wined3d_texture) as *mut ddraw_texture;

        ddraw_texture_rename_to(
            dst_ddraw_texture,
            (*src_impl).wined3d_texture,
            (*src_impl).draw_texture,
            ddraw_surface_get_rendertarget_view(src_impl),
            (*src_ddraw_texture).texture_memory,
            current_rtv,
        );

        dst_ddraw_texture = src_ddraw_texture;
    } else {
        current = iface;
        loop {
            hr = ddraw_surface1_GetAttachedSurface(current, core::ptr::addr_of_mut!(caps), core::ptr::addr_of_mut!(current));
            if hr < 0 {
                ERR!("Can't find a flip target\n");
                wined3d_mutex_unlock();
                return DDERR_NOTFLIPPABLE;
            }
            ddraw_surface1_Release(current);
            if current == iface {
                dst_impl = impl_from_IDirectDrawSurface(iface);
                break;
            }

            src_impl = impl_from_IDirectDrawSurface(current);
            src_ddraw_texture =
                wined3d_texture_get_parent((*src_impl).wined3d_texture) as *mut ddraw_texture;

            ddraw_texture_rename_to(
                dst_ddraw_texture,
                (*src_impl).wined3d_texture,
                (*src_impl).draw_texture,
                ddraw_surface_get_rendertarget_view(src_impl),
                (*src_ddraw_texture).texture_memory,
                current_rtv,
            );

            dst_ddraw_texture = src_ddraw_texture;
            dst_impl = src_impl;
        }
    }

    ddraw_texture_rename_to(
        dst_ddraw_texture,
        texture,
        draw_texture,
        tmp_rtv,
        texture_memory,
        current_rtv,
    );

    if flags
        & !((DDFLIP_NOVSYNC | DDFLIP_INTERVAL2 | DDFLIP_INTERVAL3 | DDFLIP_INTERVAL4) as DWORD)
        != 0
    {
        static mut once: UINT = 0;
        let fresh = once;
        once = once.wrapping_add(1);
        if fresh == 0 {
            FIXME!("Ignoring flags %#lx.\n", flags);
        } else {
            WARN!("Ignoring flags %#lx.\n", flags);
        }
    }

    if ((*dst_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0 {
        hr = ddraw_surface_update_frontbuffer(
            dst_impl,
            core::ptr::null::<RECT>(),
            (FALSE) as BOOL,
            ddraw_swap_interval_from_flags(flags),
        );
    } else {
        hr = DD_OK;
    }

    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface7_Flip(
    iface: *mut IDirectDrawSurface7,
    src: *mut IDirectDrawSurface7,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(src);

    return ddraw_surface1_Flip(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface_iface),
        if !src_impl.is_null() {
            core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface_iface)
        } else {
            core::ptr::null_mut()
        },
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface4_Flip(
    mut iface: *mut IDirectDrawSurface4,
    mut src: *mut IDirectDrawSurface4,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(src);
    WINE_TRACE!("iface %p, src %p, flags %#lx.\n", iface, src, flags);
    return ddraw_surface1_Flip(
        &raw mut (*surface).IDirectDrawSurface_iface,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface_iface
        } else {
            ::core::ptr::null_mut::<IDirectDrawSurface>()
        },
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface3_Flip(
    iface: *mut IDirectDrawSurface3,
    src: *mut IDirectDrawSurface3,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(src);

    return ddraw_surface1_Flip(
        &raw mut (*surface).IDirectDrawSurface_iface,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface>()
        },
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface2_Flip(
    mut iface: *mut IDirectDrawSurface2,
    mut src: *mut IDirectDrawSurface2,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(src);

    return ddraw_surface1_Flip(
        &raw mut (*surface).IDirectDrawSurface_iface,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface>()
        },
        flags,
    );
}

unsafe extern "C" fn ddraw_surface_blt(
    dst_surface: *mut ddraw_surface,
    dst_rect: *const RECT,
    src_surface: *mut ddraw_surface,
    src_rect: *const RECT,
    flags: DWORD,
    fill_colour: DWORD,
    fx: *const wined3d_blt_fx,
    filter: wined3d_texture_filter_type,
) -> HRESULT {
    let ddraw: *mut ddraw = (*dst_surface).ddraw;
    let mut colour: wined3d_color = wined3d_color {
        r: 0.,
        g: 0.,
        b: 0.,
        a: 0.,
    };
    let mut wined3d_flags: DWORD = 0;

    if (flags & (DDBLT_COLORFILL as DWORD)) != 0 {
        let location_flags: ::core::ffi::c_uint = (if !dst_rect.is_null() {
            DDRAW_SURFACE_RW
        } else {
            DDRAW_SURFACE_WRITE
        }) as ::core::ffi::c_uint;

        wined3d_flags = WINED3DCLEAR_TARGET as DWORD;
        if (flags & (DDBLT_ASYNC as DWORD)) == 0 {
            wined3d_flags |= WINED3DCLEAR_SYNCHRONOUS as DWORD;
        }

        if wined3d_colour_from_ddraw_colour(
            &raw mut (*dst_surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat,
            (*dst_surface).palette,
            fill_colour,
            &raw mut colour,
        ) == 0
        {
            return DDERR_INVALIDPARAMS as HRESULT;
        }

        if ((*dst_surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_SYSTEMMEMORY as DWORD)) != 0 {
            return wined3d_device_context_clear_sysmem_texture(
                (*ddraw).immediate_context,
                ddraw_surface_get_default_texture(dst_surface, location_flags),
                (*dst_surface).sub_resource_idx,
                dst_rect,
                wined3d_flags as ::core::ffi::c_uint,
                &raw mut colour,
            );
        }

        ddraw_surface_get_draw_texture(dst_surface, location_flags);
        return wined3d_device_context_clear_rendertarget_view(
            (*ddraw).immediate_context,
            ddraw_surface_get_rendertarget_view(dst_surface),
            dst_rect,
            wined3d_flags as ::core::ffi::c_uint,
            &raw mut colour,
            0.0f32,
            0 as ::core::ffi::c_uint,
        );
    }

    if (flags & (DDBLT_DEPTHFILL as DWORD)) != 0 {
        wined3d_flags = WINED3DCLEAR_ZBUFFER as DWORD;
        if (flags & (DDBLT_ASYNC as DWORD)) == 0 {
            wined3d_flags |= WINED3DCLEAR_SYNCHRONOUS as DWORD;
        }

        if wined3d_colour_from_ddraw_colour(
            &raw mut (*dst_surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat,
            (*dst_surface).palette,
            fill_colour,
            &raw mut colour,
        ) == 0
        {
            return DDERR_INVALIDPARAMS as HRESULT;
        }

        ddraw_surface_get_draw_texture(
            dst_surface,
            (if !dst_rect.is_null() {
                DDRAW_SURFACE_RW
            } else {
                DDRAW_SURFACE_WRITE
            }) as ::core::ffi::c_uint,
        );
        return wined3d_device_context_clear_rendertarget_view(
            (*ddraw).immediate_context,
            ddraw_surface_get_rendertarget_view(dst_surface),
            dst_rect,
            wined3d_flags as ::core::ffi::c_uint,
            ::core::ptr::null::<wined3d_color>(),
            colour.r,
            0 as ::core::ffi::c_uint,
        );
    }

    wined3d_flags = flags & !(DDBLT_ASYNC as DWORD);
    if (wined3d_flags & !(WINED3D_BLT_MASK as DWORD)) != 0 {
        FIXME!("Unhandled flags %#lx.\n", flags);
        return E_NOTIMPL as HRESULT;
    }

    if (flags & (DDBLT_ASYNC as DWORD)) == 0 {
        wined3d_flags |= WINED3D_BLT_SYNCHRONOUS as DWORD;
    }

    return wined3d_device_context_blt(
        (*ddraw).immediate_context,
        ddraw_surface_get_any_texture(dst_surface, DDRAW_SURFACE_RW as ::core::ffi::c_uint),
        (*dst_surface).sub_resource_idx,
        dst_rect,
        ddraw_surface_get_any_texture(src_surface, DDRAW_SURFACE_READ as ::core::ffi::c_uint),
        (*src_surface).sub_resource_idx,
        src_rect,
        wined3d_flags as ::core::ffi::c_uint,
        fx,
        filter,
    );
}

unsafe extern "C" fn ddraw_surface_blt_clipped(mut dst_surface: *mut ddraw_surface, mut dst_rect_in: *const RECT, mut src_surface: *mut ddraw_surface, mut src_rect_in: *const RECT, mut flags: DWORD, mut fill_colour: DWORD, mut fx: *const wined3d_blt_fx, mut filter: wined3d_texture_filter_type) -> HRESULT {
    let mut src_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut dst_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut scale_x: f32 = 0.0;
    let mut scale_y: f32 = 0.0;
    let mut clip_rect: *const RECT = core::ptr::null::<RECT>();
    let mut clip_list_size: DWORD = 0;
    let mut clip_list: *mut RGNDATA = core::ptr::null_mut::<RGNDATA>();
    let mut hr: HRESULT = DD_OK;
    let mut i: UINT = 0;

    if dst_rect_in.is_null() {
        SetRect(
            &raw mut dst_rect,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*dst_surface).surface_desc.dwWidth as ::core::ffi::c_int,
            (*dst_surface).surface_desc.dwHeight as ::core::ffi::c_int,
        );
    } else {
        dst_rect = *dst_rect_in;
    }

    if IsRectEmpty(&raw mut dst_rect as LPCRECT) != 0 {
        return DDERR_INVALIDRECT;
    }

    if !src_surface.is_null() {
        if src_rect_in.is_null() {
            SetRect(
                &raw mut src_rect,
                0 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (*src_surface).surface_desc.dwWidth as ::core::ffi::c_int,
                (*src_surface).surface_desc.dwHeight as ::core::ffi::c_int,
            );
        } else {
            src_rect = *src_rect_in;
        }

        if IsRectEmpty(&raw mut src_rect as LPCRECT) != 0 {
            return DDERR_INVALIDRECT;
        }
    } else {
        SetRectEmpty(&raw mut src_rect);
    }

    if (*dst_surface).clipper.is_null() {
        if !src_surface.is_null()
            && ((*src_surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0
        {
            hr = ddraw_surface_update_frontbuffer(
                src_surface,
                &raw mut src_rect,
                TRUE as BOOL,
                0 as ::core::ffi::c_uint,
            );
        }
        if hr >= 0 {
            hr = ddraw_surface_blt(
                dst_surface,
                &raw mut dst_rect,
                src_surface,
                &raw mut src_rect,
                flags,
                fill_colour,
                fx,
                filter,
            );
        }
        if hr >= 0
            && ((*dst_surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0
        {
            hr = ddraw_surface_update_frontbuffer(
                dst_surface,
                &raw mut dst_rect,
                FALSE as BOOL,
                0 as ::core::ffi::c_uint,
            );
        }

        return hr;
    }

    if ddraw_clipper_is_valid((*dst_surface).clipper) == 0 {
        return DDERR_INVALIDPARAMS;
    }

    scale_x = (src_rect.right - src_rect.left) as f32
        / (dst_rect.right - dst_rect.left) as f32;
    scale_y = (src_rect.bottom - src_rect.top) as f32
        / (dst_rect.bottom - dst_rect.top) as f32;

    hr = IDirectDrawClipper_GetClipList(
        &raw mut (*(*dst_surface).clipper).IDirectDrawClipper_iface,
        &raw mut dst_rect,
        core::ptr::null_mut::<_RGNDATA>(),
        &raw mut clip_list_size,
    );
    if hr < 0 {
        WARN!("Failed to get clip list size, hr %#lx.\n", hr);
        return hr;
    }

    clip_list = malloc(clip_list_size as size_t) as *mut RGNDATA;
    if clip_list.is_null() {
        WARN!("Failed to allocate clip list.\n");
        return E_OUTOFMEMORY;
    }

    hr = IDirectDrawClipper_GetClipList(
        &raw mut (*(*dst_surface).clipper).IDirectDrawClipper_iface,
        &raw mut dst_rect,
        clip_list as LPRGNDATA,
        &raw mut clip_list_size,
    );
    if hr < 0 {
        WARN!("Failed to get clip list, hr %#lx.\n", hr);
        free(clip_list as *mut ::core::ffi::c_void);
        return hr;
    }

    clip_rect = &raw mut (*clip_list).Buffer as *mut RECT;
    i = 0 as UINT;
    while i < (*clip_list).rdh.nCount {
        let mut src_rect_clipped: RECT = src_rect;

        if !src_surface.is_null() {
            src_rect_clipped.left += (((*clip_rect.offset(i as isize)).left - dst_rect.left)
                as f32
                * scale_x) as LONG;
            src_rect_clipped.top += (((*clip_rect.offset(i as isize)).top - dst_rect.top)
                as f32
                * scale_y) as LONG;
            src_rect_clipped.right -= ((dst_rect.right - (*clip_rect.offset(i as isize)).right)
                as f32
                * scale_x) as LONG;
            src_rect_clipped.bottom -= ((dst_rect.bottom - (*clip_rect.offset(i as isize)).bottom)
                as f32
                * scale_y) as LONG;

            if ((*src_surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0 {
                hr = ddraw_surface_update_frontbuffer(
                    src_surface,
                    &raw mut src_rect_clipped,
                    TRUE as BOOL,
                    0 as ::core::ffi::c_uint,
                );
                if hr < 0 {
                    break;
                }
            }
        }

        hr = ddraw_surface_blt(
            dst_surface,
            clip_rect.offset(i as isize) as *const RECT,
            src_surface,
            &raw mut src_rect_clipped,
            flags,
            fill_colour,
            fx,
            filter,
        );
        if hr < 0 {
            break;
        }

        if ((*dst_surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0 {
            hr = ddraw_surface_update_frontbuffer(
                dst_surface,
                clip_rect.offset(i as isize) as *const RECT,
                FALSE as BOOL,
                0 as ::core::ffi::c_uint,
            );
            if hr < 0 {
                break;
            }
        }

        i = i.wrapping_add(1);
    }

    free(clip_list as *mut ::core::ffi::c_void);
    return hr;
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface1_Blt(
    iface: *mut IDirectDrawSurface,
    dst_rect: *mut RECT,
    src_surface: *mut IDirectDrawSurface,
    src_rect: *mut RECT,
    mut flags: DWORD,
    mut fx: *mut DDBLTFX,
) -> HRESULT {
    let dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(src_surface);
    let mut wined3d_fx: wined3d_blt_fx = core::mem::zeroed();
    let mut unsupported_flags: DWORD;
    let mut fill_colour: DWORD = 0;
    let mut hr: HRESULT = DD_OK;
    let mut rop_fx: DDBLTFX = core::mem::zeroed();

    unsupported_flags = (DDBLT_ALPHADEST
        | DDBLT_ALPHADESTCONSTOVERRIDE
        | DDBLT_ALPHADESTNEG
        | DDBLT_ALPHADESTSURFACEOVERRIDE
        | DDBLT_ALPHAEDGEBLEND
        | DDBLT_ALPHASRC
        | DDBLT_ALPHASRCCONSTOVERRIDE
        | DDBLT_ALPHASRCNEG
        | DDBLT_ALPHASRCSURFACEOVERRIDE
        | DDBLT_ZBUFFER
        | DDBLT_ZBUFFERDESTCONSTOVERRIDE
        | DDBLT_ZBUFFERDESTOVERRIDE
        | DDBLT_ZBUFFERSRCCONSTOVERRIDE
        | DDBLT_ZBUFFERSRCOVERRIDE);
    if (flags & unsupported_flags) != 0 {
        WARN!("Ignoring unsupported flags %#lx.\n", flags & unsupported_flags);
        flags &= !unsupported_flags;
    }

    if (flags & DDBLT_KEYSRCOVERRIDE) != 0 && (fx.is_null() || (flags & DDBLT_KEYSRC) != 0) {
        WARN!("Invalid source color key parameters, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }

    if (flags & DDBLT_KEYDESTOVERRIDE) != 0 && (fx.is_null() || (flags & DDBLT_KEYDEST) != 0) {
        WARN!("Invalid destination color key parameters, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }

    if (flags & DDBLT_DDROPS) != 0 {
        if !fx.is_null() {
            FIXME!("    rop %#lx, pattern %p.\n", (*fx).dwDDROP, (*fx).__bindgen_anon_5.lpDDSPattern);
        }
        return DDERR_NORASTEROPHW;
    }

    wined3d_mutex_lock();

    if (flags & (DDBLT_COLORFILL | DDBLT_DEPTHFILL)) != 0 {
        if (flags & DDBLT_ROP) != 0 {
            wined3d_mutex_unlock();
            WARN!("DDBLT_ROP used with DDBLT_COLORFILL or DDBLT_DEPTHFILL, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }
        if !src_impl.is_null() {
            wined3d_mutex_unlock();
            WARN!("Depth or colorfill is not compatible with source surfaces, returning DDERR_INVALIDPARAMS\n");
            return DDERR_INVALIDPARAMS;
        }
        if fx.is_null() {
            wined3d_mutex_unlock();
            WARN!("Depth or colorfill used with NULL fx, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }

        if (flags & (DDBLT_COLORFILL | DDBLT_DEPTHFILL)) == (DDBLT_COLORFILL | DDBLT_DEPTHFILL) {
            flags &= !DDBLT_DEPTHFILL;
        }

        if ((*dst_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) != 0 && (flags & DDBLT_COLORFILL) != 0 {
            wined3d_mutex_unlock();
            WARN!("DDBLT_COLORFILL used on a depth buffer, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }
        if ((*dst_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) == 0 && (flags & DDBLT_DEPTHFILL) != 0 {
            wined3d_mutex_unlock();
            WARN!("DDBLT_DEPTHFILL used on a color buffer, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }
    }

    if (flags & DDBLT_ROP) != 0 {
        if fx.is_null() {
            wined3d_mutex_unlock();
            WARN!("DDBLT_ROP used with NULL fx, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }

        if !src_impl.is_null()
            && !src_rect.is_null()
            && (((*src_rect).left as ULONG) >= ((*src_rect).right as ULONG)
                || ((*src_rect).right as DWORD) > (*src_impl).surface_desc.dwWidth
                || ((*src_rect).top as ULONG) >= ((*src_rect).bottom as ULONG)
                || ((*src_rect).bottom as DWORD) > (*src_impl).surface_desc.dwHeight)
        {
            wined3d_mutex_unlock();
            WARN!("Invalid source rectangle.\n");
            return DDERR_INVALIDRECT;
        }

        flags &= !DDBLT_ROP;
        match (*fx).dwROP {
            SRCCOPY => {}

            WHITENESS | BLACKNESS => {
                rop_fx = *fx;

                if (*fx).dwROP == WHITENESS {
                    rop_fx.__bindgen_anon_5.dwFillColor = 0xffffffff;
                } else {
                    rop_fx.__bindgen_anon_5.dwFillColor = 0;
                }

                if ((*dst_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) != 0 {
                    flags |= DDBLT_DEPTHFILL;
                } else {
                    flags |= DDBLT_COLORFILL;
                }

                fx = core::ptr::addr_of_mut!(rop_fx);
            }

            _ => {
                wined3d_mutex_unlock();
                WARN!("Unsupported ROP %#lx used, returning DDERR_NORASTEROPHW.\n", (*fx).dwROP);
                return DDERR_NORASTEROPHW;
            }
        }
    }

    if (flags & (DDBLT_COLORFILL | DDBLT_DEPTHFILL)) == 0 && src_impl.is_null() {
        WARN!("No source surface.\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if (flags & DDBLT_KEYSRC) != 0 && (src_impl.is_null() || ((*src_impl).surface_desc.dwFlags & DDSD_CKSRCBLT) == 0) {
        WARN!("DDBLT_KEYSRC blit without color key in surface, returning DDERR_INVALIDPARAMS\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }
    if (flags & DDBLT_KEYDEST) != 0 && ((*dst_impl).surface_desc.dwFlags & DDSD_CKDESTBLT) == 0 {
        WARN!("DDBLT_KEYDEST blit without color key in surface, returning DDERR_INVALIDPARAMS\n");
        wined3d_mutex_unlock();
        return DDERR_INVALIDPARAMS;
    }

    if !fx.is_null() {
        wined3d_fx.fx = (*fx).dwDDFX as uint32_t;
        fill_colour = (*fx).__bindgen_anon_5.dwFillColor;
        wined3d_fx.dst_color_key.color_space_low_value = (*fx).ddckDestColorkey.dwColorSpaceLowValue as ::core::ffi::c_uint;
        wined3d_fx.dst_color_key.color_space_high_value = (*fx).ddckDestColorkey.dwColorSpaceHighValue as ::core::ffi::c_uint;
        wined3d_fx.src_color_key.color_space_low_value = (*fx).ddckSrcColorkey.dwColorSpaceLowValue as ::core::ffi::c_uint;
        wined3d_fx.src_color_key.color_space_high_value = (*fx).ddckSrcColorkey.dwColorSpaceHighValue as ::core::ffi::c_uint;
    }

    hr = ddraw_surface_blt_clipped(
        dst_impl,
        dst_rect,
        src_impl,
        src_rect,
        flags,
        fill_colour,
        if !fx.is_null() {
            core::ptr::addr_of_mut!(wined3d_fx)
        } else {
            core::ptr::null_mut::<wined3d_blt_fx>()
        },
        WINED3D_TEXF_LINEAR,
    );

    wined3d_mutex_unlock();
    match hr {
        WINED3DERR_NOTAVAILABLE => return DDERR_UNSUPPORTED,
        _ => return hr,
    }
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface7_Blt(
    mut iface: *mut IDirectDrawSurface7,
    mut dst_rect: *mut RECT,
    mut src_surface: *mut IDirectDrawSurface7,
    mut src_rect: *mut RECT,
    mut flags: DWORD,
    mut fx: *mut DDBLTFX,
) -> HRESULT {
    let mut dst: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut src: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(src_surface);
    WINE_TRACE!("iface %p, dst_rect %s, src_surface %p, src_rect %s, flags %#lx, fx %p.\n", iface, wine_dbgstr_rect(dst_rect), src_surface, wine_dbgstr_rect(src_rect), flags, fx);
    return ddraw_surface1_Blt(
        core::ptr::addr_of_mut!((*dst).IDirectDrawSurface_iface),
        dst_rect,
        if !src.is_null() {
            core::ptr::addr_of_mut!((*src).IDirectDrawSurface_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface>()
        },
        src_rect,
        flags,
        fx,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface4_Blt(
    iface: *mut IDirectDrawSurface4,
    dst_rect: *mut RECT,
    src_surface: *mut IDirectDrawSurface4,
    src_rect: *mut RECT,
    flags: DWORD,
    fx: *mut DDBLTFX,
) -> HRESULT {
    let dst: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let src: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(src_surface);
    WINE_TRACE!("iface %p, dst_rect %s, src_surface %p, src_rect %s, flags %#lx, fx %p.\n", iface, wine_dbgstr_rect(dst_rect), src_surface, wine_dbgstr_rect(src_rect), flags, fx);
    return ddraw_surface1_Blt(
        core::ptr::addr_of_mut!((*dst).IDirectDrawSurface_iface),
        dst_rect,
        if !src.is_null() {
            core::ptr::addr_of_mut!((*src).IDirectDrawSurface_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface>()
        },
        src_rect,
        flags,
        fx,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface3_Blt(iface: *mut IDirectDrawSurface3, dst_rect: *mut RECT, src_surface: *mut IDirectDrawSurface3, src_rect: *mut RECT, flags: DWORD, fx: *mut DDBLTFX) -> HRESULT {
    let mut dst: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut src: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(src_surface);
    WINE_TRACE!("iface %p, dst_rect %s, src_surface %p, src_rect %s, flags %#lx, fx %p.\n", iface, wine_dbgstr_rect(dst_rect), src_surface, wine_dbgstr_rect(src_rect), flags, fx);
    return ddraw_surface1_Blt(
        core::ptr::addr_of_mut!((*dst).IDirectDrawSurface_iface),
        dst_rect,
        if !src.is_null() {
            core::ptr::addr_of_mut!((*src).IDirectDrawSurface_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface>()
        },
        src_rect,
        flags,
        fx,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface2_Blt(iface: *mut IDirectDrawSurface2, dst_rect: *mut RECT, src_surface: *mut IDirectDrawSurface2, src_rect: *mut RECT, flags: DWORD, fx: *mut DDBLTFX) -> HRESULT {
    let mut dst: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut src: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(src_surface);

    return ddraw_surface1_Blt(
        core::ptr::addr_of_mut!((*dst).IDirectDrawSurface_iface),
        dst_rect,
        if !src.is_null() {
            core::ptr::addr_of_mut!((*src).IDirectDrawSurface_iface)
        } else {
            core::ptr::null_mut()
        },
        src_rect,
        flags,
        fx,
    );
}

unsafe extern "C" fn ddraw_surface_attach_surface(This: *mut ddraw_surface, Surf: *mut ddraw_surface) -> HRESULT {
    let mut device: *mut d3d_device = core::ptr::null_mut();

    if Surf == This {
        return DDERR_CANNOTATTACHSURFACE; /* unchecked */
    }

    wined3d_mutex_lock();

    /* Check if the surface is already attached somewhere */
    if !(*Surf).next_attached.is_null() || (*Surf).first_attached != Surf {
        /* TODO: Test for the structure of the manual attachment. Is it a
         * chain or a list? What happens if one surface is attached to 2
         * different surfaces? */
        WARN!("Surface %p is already attached somewhere. next_attached %p, first_attached %p.\n",
                Surf, (*Surf).next_attached, (*Surf).first_attached);

        wined3d_mutex_unlock();
        return DDERR_SURFACEALREADYATTACHED;
    }

    /* This inserts the new surface at the 2nd position in the chain, right after the root surface */
    (*Surf).next_attached = (*This).next_attached;
    (*Surf).first_attached = (*This).first_attached;
    (*This).next_attached = Surf;

    /* Check if the WineD3D depth stencil needs updating */
    device = ((*(*This).ddraw).d3ddevice_list.next as *mut u8).offset(
        -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
    ) as *mut d3d_device;
    while core::ptr::addr_of_mut!((*device).ddraw_entry)
        != core::ptr::addr_of_mut!((*(*This).ddraw).d3ddevice_list)
    {
        d3d_device_update_depth_stencil(device);
        device = ((*device).ddraw_entry.next as *mut u8).offset(
            -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
        ) as *mut d3d_device;
    }

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface7_AddAttachedSurface(iface: *mut IDirectDrawSurface7, attachment: *mut IDirectDrawSurface7) -> HRESULT {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(attachment);
    let mut hr: HRESULT;

    TRACE!("iface %p, attachment %p.\n", iface, attachment);

    /* Version 7 of this interface seems to refuse everything except z buffers, as per msdn */
    if ((*attachment_impl).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) == 0
    {

        WARN!("Application tries to attach a non Z buffer surface. caps %#lx.\n",
              (*attachment_impl).surface_desc.ddsCaps.dwCaps);
        return DDERR_CANNOTATTACHSURFACE;
    }

    hr = ddraw_surface_attach_surface(This, attachment_impl);
    if (hr < 0)
    {
        return hr;
    }
    (*attachment_impl).attached_iface = attachment as *mut IUnknown;
    IUnknown_AddRef((*attachment_impl).attached_iface);
    return hr;
}

unsafe extern "system" fn ddraw_surface4_AddAttachedSurface(iface: *mut IDirectDrawSurface4, attachment: *mut IDirectDrawSurface4) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(attachment);
    let mut hr: HRESULT = 0;

    /* Tests suggest that
     * -> offscreen plain surfaces can be attached to other offscreen plain surfaces
     * -> offscreen plain surfaces can be attached to primaries
     * -> primaries can be attached to offscreen plain surfaces
     * -> z buffers can be attached to primaries */
    if ((*surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE | DDSCAPS_OFFSCREENPLAIN) as DWORD) != 0
        && ((*attachment_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE | DDSCAPS_OFFSCREENPLAIN) as DWORD) != 0
    {
        /* Sizes have to match */
        if (*attachment_impl).surface_desc.dwWidth != (*surface).surface_desc.dwWidth
            || (*attachment_impl).surface_desc.dwHeight != (*surface).surface_desc.dwHeight
        {
            WARN!("Surface sizes do not match.\n");
            return DDERR_CANNOTATTACHSURFACE;
        }
    } else if ((*surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE | DDSCAPS_3DDEVICE) as DWORD) == 0
        || ((*attachment_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_ZBUFFER) as DWORD) == 0
    {
        WARN!("Invalid attachment combination.\n");
        return DDERR_CANNOTATTACHSURFACE;
    }

    if ({ hr = ddraw_surface_attach_surface(surface, attachment_impl); hr }) < 0 {
        return hr;
    }

    (*attachment_impl).attached_iface = attachment as *mut IUnknown;
    IUnknown_AddRef((*attachment_impl).attached_iface);
    return hr;
}
unsafe extern "system" fn ddraw_surface3_AddAttachedSurface(
    iface: *mut IDirectDrawSurface3,
    attachment: *mut IDirectDrawSurface3,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(attachment);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, attachment %p.\n", iface, attachment);

    hr = ddraw_surface4_AddAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface4_iface),
        if !attachment_impl.is_null() {
            core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface4_iface)
        } else {
            core::ptr::null_mut()
        },
    );
    if hr < 0 {
        return hr;
    }

    (*attachment_impl).attached_iface = attachment as *mut IUnknown;
    IUnknown_AddRef((*attachment_impl).attached_iface);
    ddraw_surface4_Release(core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface4_iface));
    return hr;
}
unsafe extern "system" fn ddraw_surface2_AddAttachedSurface(iface: *mut IDirectDrawSurface2, attachment: *mut IDirectDrawSurface2) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(attachment);
    let mut hr: HRESULT;

    TRACE!("iface %p, attachment %p.\n", iface, attachment);

    hr = ddraw_surface4_AddAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface4_iface),
        if !attachment_impl.is_null() {
            core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface4_iface)
        } else {
            core::ptr::null_mut()
        },
    );
    if hr < 0 {
        return hr;
    }

    (*attachment_impl).attached_iface = attachment as *mut IUnknown;
    IUnknown_AddRef((*attachment_impl).attached_iface);
    ddraw_surface4_Release(core::ptr::addr_of_mut!((*attachment_impl).IDirectDrawSurface4_iface));
    return hr;
}
unsafe extern "system" fn ddraw_surface1_AddAttachedSurface(iface: *mut IDirectDrawSurface, attachment: *mut IDirectDrawSurface) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(attachment);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, attachment %p.\n", iface, attachment);

    hr = ddraw_surface4_AddAttachedSurface(
        &raw mut (*surface).IDirectDrawSurface4_iface,
        if !attachment_impl.is_null() {
            &raw mut (*attachment_impl).IDirectDrawSurface4_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface4>()
        },
    );
    if hr < 0 {
        return hr;
    }

    (*attachment_impl).attached_iface = attachment as *mut IUnknown;
    IUnknown_AddRef((*attachment_impl).attached_iface);
    ddraw_surface4_Release(&raw mut (*attachment_impl).IDirectDrawSurface4_iface);
    return hr;
}

unsafe extern "C" fn ddraw_surface_delete_attached_surface(
    surface: *mut ddraw_surface,
    attachment: *mut ddraw_surface,
    detach_iface: *mut IUnknown,
) -> HRESULT {
    let mut dsv: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut prev: *mut ddraw_surface = surface;
    let mut device: *mut d3d_device = core::ptr::null_mut();

    wined3d_mutex_lock();
    if attachment.is_null() || (*attachment).first_attached != surface || attachment == surface {
        wined3d_mutex_unlock();
        return DDERR_CANNOTDETACHSURFACE;
    }

    if (*attachment).attached_iface != detach_iface {
        WARN!(
            "attachment->attach_iface %p != detach_iface %p.\n",
            (*attachment).attached_iface,
            detach_iface
        );
        wined3d_mutex_unlock();
        return DDERR_SURFACENOTATTACHED;
    }

    /* Remove MIPMAPSUBLEVEL if this seemed to be one */
    if ((*surface).surface_desc.ddsCaps.dwCaps
        & (*attachment).surface_desc.ddsCaps.dwCaps
        & DDSCAPS_MIPMAP)
        != 0
    {
        (*attachment).surface_desc.ddsCaps.dwCaps2 &= !(DDSCAPS2_MIPMAPSUBLEVEL);
        /* FIXME: we should probably also subtract from dwMipMapCount of this
         * and all parent surfaces */
    }

    /* Find the predecessor of the detached surface */
    while (*prev).next_attached != attachment {
        prev = (*prev).next_attached;
        if prev.is_null() {
            ERR!("Failed to find predecessor of %p.\n", attachment);
            wined3d_mutex_unlock();
            return DDERR_SURFACENOTATTACHED;
        }
    }

    /* Unchain the surface */
    (*prev).next_attached = (*attachment).next_attached;
    (*attachment).next_attached = core::ptr::null_mut();
    (*attachment).first_attached = attachment;

    /* Check if the wined3d depth stencil needs updating. Note that we don't
     * just call d3d_device_update_depth_stencil() here since it uses
     * QueryInterface(). Some applications, SCP - Containment Breach in
     * particular, modify the QueryInterface() pointer in the surface vtbl
     * but don't cleanup properly after the relevant dll is unloaded. */
    dsv = wined3d_device_context_get_depth_stencil_view((*(*surface).ddraw).immediate_context);
    if ((*attachment).surface_desc.ddsCaps.dwCaps & DDSCAPS_ZBUFFER) != 0
        && dsv == (*attachment).wined3d_rtv
    {
        wined3d_device_context_set_depth_stencil_view((*(*surface).ddraw).immediate_context, core::ptr::null_mut());
        /* LIST_FOR_EACH_ENTRY(device, &surface->ddraw->d3ddevice_list, struct d3d_device, ddraw_entry) */
        device = ((*(*surface).ddraw).d3ddevice_list.next as *mut u8)
            .offset(-(core::mem::offset_of!(d3d_device, ddraw_entry) as isize))
            as *mut d3d_device;
        while core::ptr::addr_of_mut!((*device).ddraw_entry)
            != core::ptr::addr_of_mut!((*(*surface).ddraw).d3ddevice_list)
        {
            wined3d_stateblock_depth_buffer_changed((*device).state);
            device = ((*device).ddraw_entry.next as *mut u8)
                .offset(-(core::mem::offset_of!(d3d_device, ddraw_entry) as isize))
                as *mut d3d_device;
        }
    }
    wined3d_mutex_unlock();

    /* Set attached_iface to NULL before releasing it, the surface may go
     * away. */
    (*attachment).attached_iface = core::ptr::null_mut();
    IUnknown_Release(detach_iface);

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface7_DeleteAttachedSurface(
    mut iface: *mut IDirectDrawSurface7,
    mut flags: DWORD,
    mut attachment: *mut IDirectDrawSurface7,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(attachment);
    // TRACE("iface %p, flags %#lx, attachment %p.\n", iface, flags, attachment) omitted: debug logging only
    return ddraw_surface_delete_attached_surface(
        surface,
        attachment_impl,
        attachment as *mut IUnknown,
    );
}
unsafe extern "system" fn ddraw_surface4_DeleteAttachedSurface(
    mut iface: *mut IDirectDrawSurface4,
    mut flags: DWORD,
    mut attachment: *mut IDirectDrawSurface4,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(attachment);
    WINE_TRACE!("iface %p, flags %#lx, attachment %p.\n", iface, flags, attachment);
    return ddraw_surface_delete_attached_surface(
        surface,
        attachment_impl,
        attachment as *mut IUnknown,
    );
}
unsafe extern "system" fn ddraw_surface3_DeleteAttachedSurface(
    mut iface: *mut IDirectDrawSurface3,
    mut flags: DWORD,
    mut attachment: *mut IDirectDrawSurface3,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(attachment);
    WINE_TRACE!("iface %p, flags %#lx, attachment %p.\n", iface, flags, attachment);
    return ddraw_surface_delete_attached_surface(
        surface,
        attachment_impl,
        attachment as *mut IUnknown,
    );
}

unsafe extern "system" fn ddraw_surface2_DeleteAttachedSurface(
    iface: *mut IDirectDrawSurface2,
    flags: DWORD,
    attachment: *mut IDirectDrawSurface2,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(attachment);

    TRACE!("iface %p, flags %#lx, attachment %p.\n", iface, flags, attachment);

    return ddraw_surface_delete_attached_surface(surface, attachment_impl, attachment as *mut IUnknown);
}

unsafe extern "system" fn ddraw_surface1_DeleteAttachedSurface(
    mut iface: *mut IDirectDrawSurface,
    mut flags: DWORD,
    mut attachment: *mut IDirectDrawSurface,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut attachment_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(attachment);
    WINE_TRACE!("iface %p, flags %#lx, attachment %p.\n", iface, flags, attachment);
    return ddraw_surface_delete_attached_surface(
        surface,
        attachment_impl,
        attachment as *mut IUnknown,
    );
}
unsafe extern "system" fn ddraw_surface7_AddOverlayDirtyRect(
    iface: *mut IDirectDrawSurface7,
    Rect: *mut RECT,
) -> HRESULT {
    // TRACE("iface %p, rect %s.\n", iface, wine_dbgstr_rect(Rect));
    DDERR_UNSUPPORTED
}

unsafe extern "system" fn ddraw_surface4_AddOverlayDirtyRect(iface: *mut IDirectDrawSurface4, rect: *mut RECT) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    WINE_TRACE!("iface %p, rect %s.\n", iface, wine_dbgstr_rect(rect));

    return ddraw_surface7_AddOverlayDirtyRect(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), rect);
}
unsafe extern "system" fn ddraw_surface3_AddOverlayDirtyRect(iface: *mut IDirectDrawSurface3, rect: *mut RECT) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_AddOverlayDirtyRect(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), rect);
}

unsafe extern "system" fn ddraw_surface2_AddOverlayDirtyRect(iface: *mut IDirectDrawSurface2, rect: *mut RECT) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_AddOverlayDirtyRect(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), rect);
}

unsafe extern "system" fn ddraw_surface1_AddOverlayDirtyRect(
    iface: *mut IDirectDrawSurface,
    rect: *mut RECT,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, rect %s.\n", iface, wine_dbgstr_rect(rect));
    return ddraw_surface7_AddOverlayDirtyRect(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), rect);
}
unsafe extern "system" fn ddraw_surface7_GetDC(iface: *mut IDirectDrawSurface7, dc: *mut HDC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut hr: HRESULT = DD_OK;

    if dc.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if !(*surface).dc.is_null() {
        hr = DDERR_DCALREADYCREATED;
    } else if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0 {
        hr = ddraw_surface_update_frontbuffer(surface, core::ptr::null(), (TRUE) as BOOL, 0);
    }
    if hr >= 0 {
        hr = wined3d_texture_get_dc(
            ddraw_surface_get_default_texture(surface, DDRAW_SURFACE_RW),
            (*surface).sub_resource_idx,
            dc,
        );
    }

    if hr >= 0 {
        (*surface).dc = *dc;

        if format_is_paletteindexed(core::ptr::addr_of_mut!((*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat)) != 0 {
            let mut palette: *const ddraw_palette;

            if !(*surface).palette.is_null() {
                palette = (*surface).palette;
            } else if !(*(*surface).ddraw).primary.is_null() {
                palette = (*(*(*surface).ddraw).primary).palette;
            } else {
                palette = core::ptr::null();
            }

            if !palette.is_null() {
                wined3d_palette_apply_to_dc((*palette).wined3d_palette, *dc);
            }
        }
    }

    wined3d_mutex_unlock();
    match hr {
        WINED3DERR_INVALIDCALL => {
            *dc = core::ptr::null_mut();
            return DDERR_CANTCREATEDC;
        }
        _ => {
            return hr;
        }
    }
}
unsafe extern "system" fn ddraw_surface4_GetDC(iface: *mut IDirectDrawSurface4, dc: *mut HDC) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_GetDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}
unsafe extern "system" fn ddraw_surface3_GetDC(iface: *mut IDirectDrawSurface3, dc: *mut HDC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, dc %p.\n", iface, dc);
    return ddraw_surface7_GetDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}
unsafe extern "system" fn ddraw_surface2_GetDC(iface: *mut IDirectDrawSurface2, dc: *mut HDC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, dc %p.\n", iface, dc);
    return ddraw_surface7_GetDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}

unsafe extern "system" fn ddraw_surface1_GetDC(iface: *mut IDirectDrawSurface, dc: *mut HDC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_GetDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}

unsafe extern "system" fn ddraw_surface7_ReleaseDC(iface: *mut IDirectDrawSurface7, hdc: HDC) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    if (*surface).dc.is_null() {
        hr = DDERR_NODC;
    } else {
        hr = wined3d_texture_release_dc(
            ddraw_surface_get_default_texture(surface, 0),
            (*surface).sub_resource_idx,
            hdc,
        );
        if hr >= 0 {
            (*surface).dc = core::ptr::null_mut();
            if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0 {
                hr = ddraw_surface_update_frontbuffer(surface, core::ptr::null(), (FALSE) as BOOL, 0);
            }
        }
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn ddraw_surface4_ReleaseDC(iface: *mut IDirectDrawSurface4, dc: HDC) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_ReleaseDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}
unsafe extern "system" fn ddraw_surface3_ReleaseDC(
    mut iface: *mut IDirectDrawSurface3,
    mut dc: HDC,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, dc %p.\n", iface, dc);
    return ddraw_surface7_ReleaseDC(&raw mut (*surface).IDirectDrawSurface7_iface, dc);
}
unsafe extern "system" fn ddraw_surface2_ReleaseDC(iface: *mut IDirectDrawSurface2, dc: HDC) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_ReleaseDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}
unsafe extern "system" fn ddraw_surface1_ReleaseDC(iface: *mut IDirectDrawSurface, dc: HDC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_ReleaseDC(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), dc);
}
unsafe extern "system" fn ddraw_surface7_GetCaps(mut iface: *mut IDirectDrawSurface7, mut Caps: *mut DDSCAPS2) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    WINE_TRACE!("iface %p, caps %p.\n", iface, Caps);
    if Caps.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    *Caps = (*surface).surface_desc.ddsCaps;
    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_GetCaps(
    mut iface: *mut IDirectDrawSurface4,
    mut caps: *mut DDSCAPS2,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, caps %p.\n", iface, caps);
    return ddraw_surface7_GetCaps(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), caps);
}
unsafe extern "system" fn ddraw_surface3_GetCaps(iface: *mut IDirectDrawSurface3, caps: *mut DDSCAPS) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, caps %p.\n", iface, caps);

    hr = ddraw_surface7_GetCaps(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        core::ptr::addr_of_mut!(caps2),
    );
    if hr < 0 {
        return hr;
    }

    (*caps).dwCaps = caps2.dwCaps;
    return hr;
}
unsafe extern "system" fn ddraw_surface2_GetCaps(iface: *mut IDirectDrawSurface2, caps: *mut DDSCAPS) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = ddraw_surface7_GetCaps(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), core::ptr::addr_of_mut!(caps2));
    if hr < 0 {
        return hr;
    }

    (*caps).dwCaps = caps2.dwCaps;
    return hr;
}
unsafe extern "system" fn ddraw_surface1_GetCaps(
    iface: *mut IDirectDrawSurface,
    caps: *mut DDSCAPS,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut caps2: DDSCAPS2 = core::mem::zeroed();
    let mut hr: HRESULT = 0;

    WINE_TRACE!("iface %p, caps %p.\n", iface, caps);

    hr = ddraw_surface7_GetCaps(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        &raw mut caps2,
    );
    if hr < 0 {
        return hr;
    }

    (*caps).dwCaps = caps2.dwCaps;
    return hr;
}
unsafe extern "system" fn ddraw_surface7_SetPriority(mut iface: *mut IDirectDrawSurface7, mut priority: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut managed: DWORD = DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE;
    let mut hr: HRESULT;
    let mut resource: *mut wined3d_resource;

    wined3d_mutex_lock();
    /* No need to check for offscreen plain surfaces or mipmap sublevels. SetPriority
     * calls on such surfaces segfault on Windows. */
    if ((*surface).surface_desc.ddsCaps.dwCaps2 & managed) == 0 {
        WARN!("Called on non-managed texture returning DDERR_INVALIDPARAMS.\n");
        hr = DDERR_INVALIDPARAMS;
    } else {
        resource = wined3d_texture_get_resource((*surface).wined3d_texture);
        wined3d_resource_set_priority(resource, priority);
        if !(*surface).draw_texture.is_null() {
            wined3d_resource_set_priority(wined3d_texture_get_resource((*surface).draw_texture), priority);
        }

        hr = DD_OK;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn ddraw_surface7_GetPriority(iface: *mut IDirectDrawSurface7, priority: *mut DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut resource: *const wined3d_resource = core::ptr::null();
    let managed: DWORD = DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE;
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_OFFSCREENPLAIN) != 0 {
        WARN!("Called on offscreenplain surface, returning DDERR_INVALIDOBJECT.\n");
        hr = DDERR_INVALIDOBJECT;
    } else if ((*surface).surface_desc.ddsCaps.dwCaps2 & managed) == 0 || (*surface).is_root() == 0 {
        WARN!("Called on non-managed texture or non-root surface, returning DDERR_INVALIDPARAMS.\n");
        hr = DDERR_INVALIDPARAMS;
    } else {
        resource = wined3d_texture_get_resource((*surface).wined3d_texture);
        *priority = wined3d_resource_get_priority(resource) as DWORD;
        hr = DD_OK;
    }
    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn ddraw_surface7_SetPrivateData(
    iface: *mut IDirectDrawSurface7,
    tag: *const GUID,
    data: *mut core::ffi::c_void,
    size: DWORD,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut hr: HRESULT;

    if data.is_null() {
        WARN!("data is NULL, returning DDERR_INVALIDPARAMS.\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    hr = wined3d_private_store_set_private_data(
        core::ptr::addr_of_mut!((*surface).private_store),
        tag,
        data,
        size,
        flags,
    );
    wined3d_mutex_unlock();
    return hr_ddraw_from_wined3d(hr);
}

unsafe extern "system" fn ddraw_surface4_SetPrivateData(iface: *mut IDirectDrawSurface4, tag: *const GUID, data: *mut core::ffi::c_void, size: DWORD, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_SetPrivateData(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), tag, data, size, flags);
}

unsafe extern "system" fn ddraw_surface7_GetPrivateData(iface: *mut IDirectDrawSurface7, tag: REFGUID, data: *mut core::ffi::c_void, size: *mut DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut stored_data: *const wined3d_private_data;
    let mut hr: HRESULT = 0;

    wined3d_mutex_lock();
    stored_data = wined3d_private_store_get_private_data(core::ptr::addr_of_mut!((*surface).private_store), tag);
    'done: {
        if stored_data.is_null() {
            hr = DDERR_NOTFOUND;
            break 'done;
        }
        if size.is_null() {
            hr = DDERR_INVALIDPARAMS;
            break 'done;
        }
        if *size < (*stored_data).size {
            *size = (*stored_data).size;
            hr = DDERR_MOREDATA;
            break 'done;
        }
        if data.is_null() {
            hr = DDERR_INVALIDPARAMS;
            break 'done;
        }

        *size = (*stored_data).size;
        memcpy(data, core::ptr::addr_of!((*stored_data).content.data) as *const core::ffi::c_void, (*stored_data).size as usize);
        hr = DD_OK;
    }
    wined3d_mutex_unlock();
    return hr;
}
unsafe extern "system" fn ddraw_surface4_GetPrivateData(
    mut iface: *mut IDirectDrawSurface4,
    tag: *const GUID,
    mut data: *mut ::core::ffi::c_void,
    mut size: *mut DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, tag %s, data %p, data_size %p.\n", iface, debugstr_guid(tag as *const _GUID), data, size);
    return ddraw_surface7_GetPrivateData(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        tag,
        data,
        size,
    );
}

unsafe extern "system" fn ddraw_surface7_FreePrivateData(
    mut iface: *mut IDirectDrawSurface7,
    tag: *const GUID,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut entry: *mut wined3d_private_data = core::ptr::null_mut::<wined3d_private_data>();
    WINE_TRACE!("iface %p, tag %s.\n", iface, debugstr_guid(tag as *const GUID));
    wined3d_mutex_lock();
    entry = wined3d_private_store_get_private_data(core::ptr::addr_of_mut!((*surface).private_store), tag);
    if entry.is_null() {
        wined3d_mutex_unlock();
        return DDERR_NOTFOUND;
    }

    wined3d_private_store_free_private_data(core::ptr::addr_of_mut!((*surface).private_store), entry);
    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface4_FreePrivateData(iface: *mut IDirectDrawSurface4, tag: *const GUID) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    return ddraw_surface7_FreePrivateData(&raw mut (*surface).IDirectDrawSurface7_iface, tag);
}
unsafe extern "system" fn ddraw_surface7_PageLock(
    iface: *mut IDirectDrawSurface7,
    Flags: DWORD,
) -> HRESULT {
    // TRACE("iface %p, flags %#lx.\n", iface, Flags);
    DD_OK
}

unsafe extern "system" fn ddraw_surface4_PageLock(
    iface: *mut IDirectDrawSurface4,
    flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_PageLock(&raw mut (*surface).IDirectDrawSurface7_iface, flags);
}

unsafe extern "system" fn ddraw_surface3_PageLock(
    iface: *mut IDirectDrawSurface3,
    flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_PageLock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface2_PageLock(
    mut iface: *mut IDirectDrawSurface2,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_PageLock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface7_PageUnlock(
    iface: *mut IDirectDrawSurface7,
    Flags: DWORD,
) -> HRESULT {
    // TRACE("iface %p, flags %#lx.\n", iface, Flags);
    DD_OK
}
unsafe extern "system" fn ddraw_surface4_PageUnlock(
    iface: *mut IDirectDrawSurface4,
    flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_PageUnlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}

unsafe extern "system" fn ddraw_surface3_PageUnlock(iface: *mut IDirectDrawSurface3, flags: DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_PageUnlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}

unsafe extern "system" fn ddraw_surface2_PageUnlock(iface: *mut IDirectDrawSurface2, flags: DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_PageUnlock(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface7_BltBatch(
    iface: *mut IDirectDrawSurface7,
    Batch: *mut DDBLTBATCH,
    Count: DWORD,
    Flags: DWORD,
) -> HRESULT {
    TRACE!("iface %p, batch %p, count %lu, flags %#lx.\n", iface, Batch, Count, Flags);
    DDERR_UNSUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface4_BltBatch(
    iface: *mut IDirectDrawSurface4,
    batch: *mut DDBLTBATCH,
    count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    WINE_TRACE!("iface %p, batch %p, count %lu, flags %#lx.\n", iface, batch, count, flags);

    return ddraw_surface7_BltBatch(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        batch,
        count,
        flags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface3_BltBatch(
    iface: *mut IDirectDrawSurface3,
    batch: *mut DDBLTBATCH,
    count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, batch %p, count %lu, flags %#lx.\n", iface, batch, count, flags);
    return ddraw_surface7_BltBatch(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        batch,
        count,
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface2_BltBatch(
    iface: *mut IDirectDrawSurface2,
    batch: *mut DDBLTBATCH,
    count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, batch %p, count %lu, flags %#lx.\n", iface, batch, count, flags);
    return ddraw_surface7_BltBatch(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        batch,
        count,
        flags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ddraw_surface1_BltBatch(
    iface: *mut IDirectDrawSurface,
    batch: *mut DDBLTBATCH,
    count: DWORD,
    flags: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, batch %p, count %lu, flags %#lx.\n", iface, batch, count, flags);
    return ddraw_surface7_BltBatch(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        batch,
        count,
        flags,
    );
}

unsafe extern "system" fn ddraw_surface7_EnumAttachedSurfaces(
    mut iface: *mut IDirectDrawSurface7,
    mut context: *mut ::core::ffi::c_void,
    mut cb: LPDDENUMSURFACESCALLBACK7,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut surf: *mut ddraw_surface = ::core::ptr::null_mut::<ddraw_surface>();
    let mut desc: DDSURFACEDESC2 = core::mem::zeroed();
    let mut i: ::core::ffi::c_int = 0;

    /* Attached surfaces aren't handled in WineD3D */

    if cb.is_none() {
        return DDERR_INVALIDPARAMS;
    }

    WINE_TRACE!("iface %p, context %p, callback %p.\n", iface, context, cb);

    wined3d_mutex_lock();

    i = 0;
    while ((i) as u32) < (MAX_COMPLEX_ATTACHED as UINT) {
        surf = (*surface).complex_array[i as usize];
        if surf.is_null() {
            break;
        }

        ddraw_surface7_AddRef(&raw mut (*surf).IDirectDrawSurface7_iface);
        desc = (*surf).surface_desc;
        /* check: != DDENUMRET_OK or == DDENUMRET_CANCEL? */
        if (cb.unwrap_unchecked())(
            &raw mut (*surf).IDirectDrawSurface7_iface,
            &raw mut desc,
            context as LPVOID,
        ) == (DDENUMRET_CANCEL as i32)
        {
            wined3d_mutex_unlock();
            return DD_OK;
        }
        i += 1;
    }

    surf = (*surface).next_attached;
    while !surf.is_null() {
        ddraw_surface7_AddRef(&raw mut (*surf).IDirectDrawSurface7_iface);
        desc = (*surf).surface_desc;
        /* check: != DDENUMRET_OK or == DDENUMRET_CANCEL? */
        if (cb.unwrap_unchecked())(
            &raw mut (*surf).IDirectDrawSurface7_iface,
            &raw mut desc,
            context as LPVOID,
        ) == (DDENUMRET_CANCEL as i32)
        {
            wined3d_mutex_unlock();
            return DD_OK;
        }
        surf = (*surf).next_attached;
    }

    WINE_TRACE!(" end of enumeration.\n");

    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn EnumCallback2(surface: *mut IDirectDrawSurface7, surface_desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void) -> HRESULT {
    let surface_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(surface);
    let info: *const callback_info2 = context as *const callback_info2;

    ddraw_surface4_AddRef(core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface4_iface));
    ddraw_surface7_Release(surface);

    return (*info).callback.unwrap_unchecked()(
        core::ptr::addr_of_mut!((*surface_impl).IDirectDrawSurface4_iface),
        surface_desc,
        (*info).context,
    );
}

unsafe extern "system" fn EnumCallback(surface: *mut IDirectDrawSurface7, surface_desc: *mut DDSURFACEDESC2, context: *mut core::ffi::c_void) -> HRESULT {
    let mut surface_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(surface);
    let mut info: *const callback_info = context as *const callback_info;

    ddraw_surface1_AddRef(&raw mut (*surface_impl).IDirectDrawSurface_iface);
    ddraw_surface7_Release(surface);

    /* FIXME: Check surface_test.dwSize */
    return (*info).callback.unwrap_unchecked()(
        &raw mut (*surface_impl).IDirectDrawSurface_iface,
        surface_desc as LPDDSURFACEDESC,
        (*info).context as LPVOID,
    );
}

unsafe extern "system" fn ddraw_surface4_EnumAttachedSurfaces(
    iface: *mut IDirectDrawSurface4,
    context: *mut core::ffi::c_void,
    callback: LPDDENUMSURFACESCALLBACK2,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut info: callback_info2 = callback_info2 {
        callback: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };

    info.callback = callback;
    info.context = context;

    return ddraw_surface7_EnumAttachedSurfaces(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
        Some(
            EnumCallback2,
        ),
    );
}
unsafe extern "system" fn ddraw_surface3_EnumAttachedSurfaces(
    mut iface: *mut IDirectDrawSurface3,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    WINE_TRACE!("iface %p, context %p, callback %p.\n", iface, context, callback);
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumAttachedSurfaces(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        &raw mut info as *mut ::core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}
unsafe extern "system" fn ddraw_surface2_EnumAttachedSurfaces(
    mut iface: *mut IDirectDrawSurface2,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    WINE_TRACE!("iface %p, context %p, callback %p.\n", iface, context, callback);
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumAttachedSurfaces(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        &raw mut info as *mut ::core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}
unsafe extern "system" fn ddraw_surface1_EnumAttachedSurfaces(
    mut iface: *mut IDirectDrawSurface,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    WINE_TRACE!("iface %p, context %p, callback %p.\n", iface, context, callback);
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumAttachedSurfaces(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        &raw mut info as *mut ::core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}
unsafe extern "system" fn ddraw_surface7_EnumOverlayZOrders(
    iface: *mut IDirectDrawSurface7,
    Flags: DWORD,
    context: *mut core::ffi::c_void,
    cb: LPDDENUMSURFACESCALLBACK7,
) -> HRESULT {
    // FIXME("iface %p, flags %#lx, context %p, callback %p stub!\n", iface, Flags, context, cb);
    DD_OK
}
unsafe extern "system" fn ddraw_surface4_EnumOverlayZOrders(
    mut iface: *mut IDirectDrawSurface4,
    mut flags: DWORD,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK2,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut info: callback_info2 = callback_info2 {
        callback: None,
        context: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    WINE_TRACE!("iface %p, flags %#lx, context %p, callback %p.\n", iface, flags, context, callback);
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumOverlayZOrders(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        &raw mut info as *mut ::core::ffi::c_void,
        Some(
            EnumCallback2,
        ),
    );
}

unsafe extern "system" fn ddraw_surface3_EnumOverlayZOrders(
    iface: *mut IDirectDrawSurface3,
    flags: DWORD,
    context: *mut core::ffi::c_void,
    callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };

    info.callback = callback;
    info.context = context;

    return ddraw_surface7_EnumOverlayZOrders(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        flags,
        core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}
unsafe extern "system" fn ddraw_surface2_EnumOverlayZOrders(
    mut iface: *mut IDirectDrawSurface2,
    mut flags: DWORD,
    mut context: *mut core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: core::ptr::null_mut::<core::ffi::c_void>(),
    };
    TRACE!(
        "iface %p, flags %#lx, context %p, callback %p.\n",
        iface,
        flags,
        context,
        callback
    );
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumOverlayZOrders(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        flags,
        core::ptr::addr_of_mut!(info) as *mut core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}
unsafe extern "system" fn ddraw_surface1_EnumOverlayZOrders(
    mut iface: *mut IDirectDrawSurface,
    mut flags: DWORD,
    mut context: *mut ::core::ffi::c_void,
    mut callback: LPDDENUMSURFACESCALLBACK,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut info: callback_info = callback_info {
        callback: None,
        context: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    info.callback = callback;
    info.context = context;
    return ddraw_surface7_EnumOverlayZOrders(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        &raw mut info as *mut ::core::ffi::c_void,
        Some(
            EnumCallback,
        ),
    );
}

unsafe extern "system" fn ddraw_surface7_GetBltStatus(
    mut iface: *mut IDirectDrawSurface7,
    mut Flags: DWORD,
) -> HRESULT {
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, Flags);
    match Flags {
        WINEDDGBS_CANBLT | WINEDDGBS_ISBLTDONE => {
            return DD_OK;
        }
        _ => {
            return DDERR_INVALIDPARAMS;
        }
    };
}
unsafe extern "system" fn ddraw_surface4_GetBltStatus(iface: *mut IDirectDrawSurface4, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_GetBltStatus(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface3_GetBltStatus(iface: *mut IDirectDrawSurface3, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_GetBltStatus(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface2_GetBltStatus(iface: *mut IDirectDrawSurface2, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    return ddraw_surface7_GetBltStatus(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface1_GetBltStatus(iface: *mut IDirectDrawSurface, flags: DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_GetBltStatus(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}

unsafe extern "system" fn ddraw_surface7_GetColorKey(iface: *mut IDirectDrawSurface7, Flags: DWORD, CKey: *mut DDCOLORKEY) -> HRESULT {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    if CKey.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();

    match Flags {
        DDCKEY_DESTBLT => {
            if (*This).surface_desc.dwFlags & DDSD_CKDESTBLT == 0 {
                wined3d_mutex_unlock();
                return DDERR_NOCOLORKEY;
            }
            *CKey = (*This).surface_desc.ddckCKDestBlt;
        }
        DDCKEY_DESTOVERLAY => {
            if (*This).surface_desc.dwFlags & DDSD_CKDESTOVERLAY == 0 {
                wined3d_mutex_unlock();
                return DDERR_NOCOLORKEY;
            }
            *CKey = (*This).surface_desc.__bindgen_anon_4.ddckCKDestOverlay;
        }
        DDCKEY_SRCBLT => {
            if (*This).surface_desc.dwFlags & DDSD_CKSRCBLT == 0 {
                wined3d_mutex_unlock();
                return DDERR_NOCOLORKEY;
            }
            *CKey = (*This).surface_desc.ddckCKSrcBlt;
        }
        DDCKEY_SRCOVERLAY => {
            if (*This).surface_desc.dwFlags & DDSD_CKSRCOVERLAY == 0 {
                wined3d_mutex_unlock();
                return DDERR_NOCOLORKEY;
            }
            *CKey = (*This).surface_desc.ddckCKSrcOverlay;
        }
        _ => {
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
    }

    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_GetColorKey(iface: *mut IDirectDrawSurface4, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, flags %#lx, color_key %p.\n", iface, flags, color_key);
    return ddraw_surface7_GetColorKey(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags, color_key);
}
unsafe extern "system" fn ddraw_surface3_GetColorKey(
    mut iface: *mut IDirectDrawSurface3,
    mut flags: DWORD,
    mut color_key: *mut DDCOLORKEY,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, flags %#lx, color_key %p.\n", iface, flags, color_key);
    return ddraw_surface7_GetColorKey(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        flags,
        color_key,
    );
}

unsafe extern "system" fn ddraw_surface2_GetColorKey(
    iface: *mut IDirectDrawSurface2,
    flags: DWORD,
    color_key: *mut DDCOLORKEY,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, flags %#lx, color_key %p.\n", iface, flags, color_key);
    return ddraw_surface7_GetColorKey(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        color_key,
    );
}
unsafe extern "system" fn ddraw_surface1_GetColorKey(iface: *mut IDirectDrawSurface, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_GetColorKey(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags, color_key);
}

unsafe extern "system" fn ddraw_surface7_GetFlipStatus(iface: *mut IDirectDrawSurface7, Flags: DWORD) -> HRESULT {
    /* XXX: DDERR_INVALIDSURFACETYPE */

    if Flags == WINEDDGFS_CANFLIP || Flags == WINEDDGFS_ISFLIPDONE {
        return DD_OK;
    }
    return DDERR_INVALIDPARAMS;
}
unsafe extern "system" fn ddraw_surface4_GetFlipStatus(
    mut iface: *mut IDirectDrawSurface4,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_GetFlipStatus(&raw mut (*surface).IDirectDrawSurface7_iface, flags);
}
unsafe extern "system" fn ddraw_surface3_GetFlipStatus(iface: *mut IDirectDrawSurface3, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_GetFlipStatus(&raw mut (*surface).IDirectDrawSurface7_iface, flags);
}
unsafe extern "system" fn ddraw_surface2_GetFlipStatus(iface: *mut IDirectDrawSurface2, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_GetFlipStatus(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface1_GetFlipStatus(
    mut iface: *mut IDirectDrawSurface,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_GetFlipStatus(&raw mut (*surface).IDirectDrawSurface7_iface, flags);
}

unsafe extern "system" fn ddraw_surface7_GetOverlayPosition(
    iface: *mut IDirectDrawSurface7,
    x: *mut LONG,
    y: *mut LONG,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut hr: HRESULT = 0;

    TRACE!("iface %p, x %p, y %p.\n", iface, x, y);
    wined3d_mutex_lock();
    hr = wined3d_texture_get_overlay_position(
        (*surface).wined3d_texture,
        (*surface).sub_resource_idx,
        x,
        y,
    );
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn ddraw_surface4_GetOverlayPosition(iface: *mut IDirectDrawSurface4, x: *mut LONG, y: *mut LONG) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    if ((*__wine_dbch___default).flags as i32 & (1i32 << __WINE_DBCL_TRACE)) != 0 {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: i32 = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/directx/wine/ddraw/surface.c\0".as_ptr() as *const i8,
            b"ddraw_surface4_GetOverlayPosition\0".as_ptr() as *const i8,
            3475,
            b"iface %p, x %p, y %p.\n\0".as_ptr() as *const i8,
            iface,
            x,
            y,
        );
    }
    return ddraw_surface7_GetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}
unsafe extern "system" fn ddraw_surface3_GetOverlayPosition(iface: *mut IDirectDrawSurface3, x: *mut LONG, y: *mut LONG) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_GetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}

unsafe extern "system" fn ddraw_surface2_GetOverlayPosition(iface: *mut IDirectDrawSurface2, x: *mut LONG, y: *mut LONG) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_GetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}

unsafe extern "system" fn ddraw_surface1_GetOverlayPosition(
    iface: *mut IDirectDrawSurface,
    x: *mut LONG,
    y: *mut LONG,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, x %p, y %p.\n", iface, x, y);
    return ddraw_surface7_GetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}
unsafe extern "system" fn ddraw_surface7_GetPixelFormat(iface: *mut IDirectDrawSurface7, PixelFormat: *mut DDPIXELFORMAT) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    if PixelFormat.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    DD_STRUCT_COPY_BYSIZE!(PixelFormat, core::ptr::addr_of_mut!((*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat));
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_GetPixelFormat(mut iface: *mut IDirectDrawSurface4, mut pixel_format: *mut DDPIXELFORMAT) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, pixel_format %p.\n", iface, pixel_format);
    return ddraw_surface7_GetPixelFormat(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        pixel_format,
    );
}
unsafe extern "system" fn ddraw_surface3_GetPixelFormat(iface: *mut IDirectDrawSurface3, pixel_format: *mut DDPIXELFORMAT) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_GetPixelFormat(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), pixel_format);
}
unsafe extern "system" fn ddraw_surface2_GetPixelFormat(
    mut iface: *mut IDirectDrawSurface2,
    mut pixel_format: *mut DDPIXELFORMAT,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, pixel_format %p.\n", iface, pixel_format);
    return ddraw_surface7_GetPixelFormat(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        pixel_format,
    );
}
unsafe extern "system" fn ddraw_surface1_GetPixelFormat(
    mut iface: *mut IDirectDrawSurface,
    mut pixel_format: *mut DDPIXELFORMAT,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, pixel_format %p.\n", iface, pixel_format);
    return ddraw_surface7_GetPixelFormat(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        pixel_format,
    );
}

unsafe extern "system" fn ddraw_surface7_GetSurfaceDesc(iface: *mut IDirectDrawSurface7, surface_desc: *mut DDSURFACEDESC2) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    if surface_desc.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC2>() {
        WARN!("Incorrect struct size %lu.\n", (*surface_desc).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    DD_STRUCT_COPY_BYSIZE!(surface_desc, core::ptr::addr_of_mut!((*surface).surface_desc));
    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        DDRAW_dump_surface_desc(surface_desc);
    }
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_GetSurfaceDesc(
    mut iface: *mut IDirectDrawSurface4,
    mut DDSD: *mut DDSURFACEDESC2,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, surface_desc %p.\n", iface, DDSD);
    return ddraw_surface7_GetSurfaceDesc(&raw mut (*surface).IDirectDrawSurface7_iface, DDSD);
}

unsafe extern "system" fn ddraw_surface3_GetSurfaceDesc(iface: *mut IDirectDrawSurface3, surface_desc: *mut DDSURFACEDESC) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    TRACE!("iface %p, surface_desc %p.\n", iface, surface_desc);

    if surface_desc.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    if (*surface_desc).dwSize as usize != core::mem::size_of::<DDSURFACEDESC>() {
        WARN!("Incorrect structure size %lu.\n", (*surface_desc).dwSize);
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    DDSD2_to_DDSD(core::ptr::addr_of_mut!((*surface).surface_desc), surface_desc);

    TRACE!("Returning surface desc:\n");

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        /* DDRAW_dump_surface_desc handles the smaller size */
        DDRAW_dump_surface_desc(surface_desc as *mut DDSURFACEDESC2);
    }
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface2_GetSurfaceDesc(
    mut iface: *mut IDirectDrawSurface2,
    mut DDSD: *mut DDSURFACEDESC,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    TRACE!("iface %p, surface_desc %p.\n", iface, DDSD);
    return ddraw_surface3_GetSurfaceDesc(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface3_iface),
        DDSD,
    );
}
unsafe extern "system" fn ddraw_surface1_GetSurfaceDesc(
    iface: *mut IDirectDrawSurface,
    DDSD: *mut DDSURFACEDESC,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    TRACE!("iface %p, surface_desc %p.\n", iface, DDSD);

    return ddraw_surface3_GetSurfaceDesc(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface3_iface), DDSD);
}
unsafe extern "system" fn ddraw_surface7_Initialize(
    iface: *mut IDirectDrawSurface7,
    ddraw: *mut IDirectDraw,
    surface_desc: *mut DDSURFACEDESC2,
) -> HRESULT {
    // TRACE("iface %p, ddraw %p, surface_desc %p.\n", iface, ddraw, surface_desc);
    DDERR_ALREADYINITIALIZED
}

unsafe extern "system" fn ddraw_surface4_Initialize(
    iface: *mut IDirectDrawSurface4,
    ddraw: *mut IDirectDraw,
    surface_desc: *mut DDSURFACEDESC2,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, ddraw %p, surface_desc %p.\n", iface, ddraw, surface_desc);
    return ddraw_surface7_Initialize(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        ddraw,
        surface_desc,
    );
}
unsafe extern "system" fn ddraw_surface3_Initialize(
    iface: *mut IDirectDrawSurface3,
    ddraw: *mut IDirectDraw,
    surface_desc: *mut DDSURFACEDESC,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();

    if ((*__wine_dbch___default).flags as i32 & ((1 as i32) << (__WINE_DBCL_TRACE as i32)) != 0)
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/directx/wine/ddraw/surface.c\0"
                as *const u8 as *const i8,
            b"ddraw_surface3_Initialize\0" as *const u8 as *const i8,
            3703 as i32,
            b"iface %p, ddraw %p, surface_desc %p.\n\0" as *const u8 as *const i8,
            iface,
            ddraw,
            surface_desc,
        );
    }
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, core::ptr::addr_of_mut!(surface_desc2));
    }
    return ddraw_surface7_Initialize(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        ddraw,
        if !surface_desc.is_null() {
            core::ptr::addr_of_mut!(surface_desc2)
        } else {
            core::ptr::null_mut()
        },
    );
}

unsafe extern "system" fn ddraw_surface2_Initialize(
    iface: *mut IDirectDrawSurface2,
    ddraw: *mut IDirectDraw,
    surface_desc: *mut DDSURFACEDESC,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed::<DDSURFACEDESC2>();
    WINE_TRACE!("iface %p, ddraw %p, surface_desc %p.\n", iface, ddraw, surface_desc);
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, core::ptr::addr_of_mut!(surface_desc2));
    }
    return ddraw_surface7_Initialize(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        ddraw,
        if !surface_desc.is_null() {
            core::ptr::addr_of_mut!(surface_desc2)
        } else {
            core::ptr::null_mut::<DDSURFACEDESC2>()
        },
    );
}

unsafe extern "system" fn ddraw_surface1_Initialize(
    mut iface: *mut IDirectDrawSurface,
    mut ddraw: *mut IDirectDraw,
    mut surface_desc: *mut DDSURFACEDESC,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed();
    WINE_TRACE!("iface %p, ddraw %p, surface_desc %p.\n", iface, ddraw, surface_desc);
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, core::ptr::addr_of_mut!(surface_desc2));
    }
    return ddraw_surface7_Initialize(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        ddraw,
        if !surface_desc.is_null() {
            core::ptr::addr_of_mut!(surface_desc2)
        } else {
            core::ptr::null_mut::<DDSURFACEDESC2>()
        },
    );
}
unsafe extern "system" fn d3d_texture1_Initialize(
    iface: *mut IDirect3DTexture,
    device: *mut IDirect3DDevice,
    surface: *mut IDirectDrawSurface,
) -> HRESULT {
    // TRACE("iface %p, device %p, surface %p.\n", iface, device, surface);
    DDERR_UNSUPPORTED
}

unsafe extern "system" fn ddraw_surface7_IsLost(iface: *mut IDirectDrawSurface7) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    return if ddraw_surface_is_lost(surface) != 0 { DDERR_SURFACELOST } else { DD_OK };
}

unsafe extern "system" fn ddraw_surface4_IsLost(iface: *mut IDirectDrawSurface4) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return if ddraw_surface_is_lost(surface) != 0 { DDERR_SURFACELOST } else { DD_OK };
}

unsafe extern "system" fn ddraw_surface3_IsLost(iface: *mut IDirectDrawSurface3) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return if ddraw_surface_is_lost(surface) != 0 { DDERR_SURFACELOST } else { DD_OK };
}
unsafe extern "system" fn ddraw_surface2_IsLost(iface: *mut IDirectDrawSurface2) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return if ddraw_surface_is_lost(surface) != 0 { DDERR_SURFACELOST } else { DD_OK };
}

unsafe extern "system" fn ddraw_surface1_IsLost(iface: *mut IDirectDrawSurface) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return if ddraw_surface_is_lost(surface) != 0 { DDERR_SURFACELOST } else { DD_OK };
}

unsafe extern "system" fn ddraw_surface7_Restore(iface: *mut IDirectDrawSurface7) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut attachment: *mut ddraw_surface = core::ptr::null_mut();
    let mut i: u32 = 0;
    let mut hr: HRESULT;

    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0
    {
        let swapchain: *mut wined3d_swapchain = (*(*surface).ddraw).wined3d_swapchain;
        let mut wined3d_desc: wined3d_sub_resource_desc = core::mem::zeroed();
        let mut mode: wined3d_display_mode = core::mem::zeroed();

        hr = wined3d_swapchain_get_display_mode(swapchain, core::ptr::addr_of_mut!(mode), core::ptr::null_mut());
        if hr < 0
        {
            WARN!("Failed to get display mode, hr %#lx.\n", hr);
            return hr;
        }

        hr = wined3d_texture_get_sub_resource_desc((*surface).wined3d_texture, 0, core::ptr::addr_of_mut!(wined3d_desc));
        if hr < 0
        {
            WARN!("Failed to get resource desc, hr %#lx.\n", hr);
            return hr;
        }

        if mode.width != wined3d_desc.width || mode.height != wined3d_desc.height
        {
            WARN!("Display mode dimensions %ux%u don't match surface dimensions %ux%u.\n",
                    mode.width, mode.height, wined3d_desc.width, wined3d_desc.height);
            return DDERR_WRONGMODE;
        }

        if mode.format_id != wined3d_desc.format
        {
            WARN!("Display mode format %#x doesn't match surface format %#x.\n",
                    mode.format_id, wined3d_desc.format);
            return DDERR_WRONGMODE;
        }
    }

    if ddraw_surface_can_be_lost(surface) == 0
        { return DD_OK; }
    ddraw_update_lost_surfaces((*surface).ddraw);
    if ((*(*surface).ddraw).device_state) as u32 == DDRAW_DEVICE_STATE_LOST
        { return DDERR_WRONGMODE; }

    (*surface).set_is_lost((FALSE) as _);

    i = 0;
    while i < MAX_COMPLEX_ATTACHED
    {
        attachment = (*surface).complex_array[i as usize];
        while !attachment.is_null()
        {
            (*attachment).set_is_lost((FALSE) as _);
            attachment = (*attachment).complex_array[0 as usize];
            if attachment == (*surface).complex_array[i as usize]
                { break; }
        }
        i = i.wrapping_add(1);
    }

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface4_Restore(iface: *mut IDirectDrawSurface4) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    TRACE!("iface %p.\n", iface);

    return ddraw_surface7_Restore(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface));
}

unsafe extern "system" fn ddraw_surface3_Restore(iface: *mut IDirectDrawSurface3) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    WINE_TRACE!("iface %p.\n", iface);

    return ddraw_surface7_Restore(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface));
}
unsafe extern "system" fn ddraw_surface2_Restore(mut iface: *mut IDirectDrawSurface2) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p.\n", iface);
    return ddraw_surface7_Restore(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface));
}
unsafe extern "system" fn ddraw_surface1_Restore(iface: *mut IDirectDrawSurface) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_Restore(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface));
}
unsafe extern "system" fn ddraw_surface7_SetOverlayPosition(iface: *mut IDirectDrawSurface7, x: LONG, y: LONG) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let hr: HRESULT;

    wined3d_mutex_lock();
    hr = wined3d_texture_set_overlay_position((*surface).wined3d_texture, (*surface).sub_resource_idx, x, y);
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn ddraw_surface4_SetOverlayPosition(iface: *mut IDirectDrawSurface4, x: LONG, y: LONG) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_SetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}
unsafe extern "system" fn ddraw_surface3_SetOverlayPosition(iface: *mut IDirectDrawSurface3, x: LONG, y: LONG) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_SetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}
unsafe extern "system" fn ddraw_surface2_SetOverlayPosition(iface: *mut IDirectDrawSurface2, x: LONG, y: LONG) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_SetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}
unsafe extern "system" fn ddraw_surface1_SetOverlayPosition(iface: *mut IDirectDrawSurface, x: LONG, y: LONG) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_SetOverlayPosition(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), x, y);
}

unsafe extern "system" fn ddraw_surface7_UpdateOverlay(iface: *mut IDirectDrawSurface7, src_rect: *mut RECT, dst_surface: *mut IDirectDrawSurface7, dst_rect: *mut RECT, flags: DWORD, fx: *mut DDOVERLAYFX) -> HRESULT {
    let mut src_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut dst_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(dst_surface);
    let mut dst_wined3d_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut dst_sub_resource_idx: u32 = 0;
    let mut hr: HRESULT = 0;

    if !fx.is_null() {
        FIXME!("Ignoring fx %p.\n", fx);
    }

    wined3d_mutex_lock();
    if !dst_impl.is_null() {
        dst_wined3d_texture = (*dst_impl).wined3d_texture;
        dst_sub_resource_idx = (*dst_impl).sub_resource_idx;
    }
    hr = wined3d_texture_update_overlay((*src_impl).wined3d_texture, (*src_impl).sub_resource_idx,
            src_rect, dst_wined3d_texture, dst_sub_resource_idx, dst_rect, flags);
    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}
unsafe extern "system" fn ddraw_surface4_UpdateOverlay(iface: *mut IDirectDrawSurface4, src_rect: *mut RECT, dst_surface: *mut IDirectDrawSurface4, dst_rect: *mut RECT, flags: DWORD, fx: *mut DDOVERLAYFX) -> HRESULT {
    let mut src_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let mut dst_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(dst_surface);

    return ddraw_surface7_UpdateOverlay(
        &raw mut (*src_impl).IDirectDrawSurface7_iface,
        src_rect,
        if !dst_impl.is_null() {
            &raw mut (*dst_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        dst_rect,
        flags,
        fx,
    );
}

unsafe extern "system" fn ddraw_surface3_UpdateOverlay(iface: *mut IDirectDrawSurface3, src_rect: *mut RECT, dst_surface: *mut IDirectDrawSurface3, dst_rect: *mut RECT, flags: DWORD, fx: *mut DDOVERLAYFX) -> HRESULT {
    let mut src_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut dst_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(dst_surface);
    WINE_TRACE!("iface %p, src_rect %s, dst_surface %p, dst_rect %s, flags %#lx, fx %p.\n", iface, wine_dbgstr_rect(src_rect), dst_surface, wine_dbgstr_rect(dst_rect), flags, fx);
    return ddraw_surface7_UpdateOverlay(
        core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface7_iface),
        src_rect,
        if !dst_impl.is_null() {
            core::ptr::addr_of_mut!((*dst_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut()
        },
        dst_rect,
        flags,
        fx,
    );
}

unsafe extern "system" fn ddraw_surface2_UpdateOverlay(iface: *mut IDirectDrawSurface2, src_rect: *mut RECT, dst_surface: *mut IDirectDrawSurface2, dst_rect: *mut RECT, flags: DWORD, fx: *mut DDOVERLAYFX) -> HRESULT {
    let src_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let dst_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(dst_surface);

    return ddraw_surface7_UpdateOverlay(
        core::ptr::addr_of_mut!((*src_impl).IDirectDrawSurface7_iface),
        src_rect,
        if !dst_impl.is_null() {
            core::ptr::addr_of_mut!((*dst_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut()
        },
        dst_rect,
        flags,
        fx,
    );
}

unsafe extern "system" fn ddraw_surface1_UpdateOverlay(iface: *mut IDirectDrawSurface, src_rect: *mut RECT, dst_surface: *mut IDirectDrawSurface, dst_rect: *mut RECT, flags: DWORD, fx: *mut DDOVERLAYFX) -> HRESULT {
    let mut src_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut dst_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(dst_surface);
    return ddraw_surface7_UpdateOverlay(
        &raw mut (*src_impl).IDirectDrawSurface7_iface,
        src_rect,
        if !dst_impl.is_null() {
            &raw mut (*dst_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        dst_rect,
        flags,
        fx,
    );
}
unsafe extern "system" fn ddraw_surface7_UpdateOverlayDisplay(
    iface: *mut IDirectDrawSurface7,
    Flags: DWORD,
) -> HRESULT {
    // TRACE("iface %p, flags %#lx.\n", iface, Flags);
    DDERR_UNSUPPORTED
}
unsafe extern "system" fn ddraw_surface4_UpdateOverlayDisplay(
    mut iface: *mut IDirectDrawSurface4,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, flags %#lx.\n", iface, flags);
    return ddraw_surface7_UpdateOverlayDisplay(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        flags,
    );
}

unsafe extern "system" fn ddraw_surface3_UpdateOverlayDisplay(iface: *mut IDirectDrawSurface3, flags: DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_UpdateOverlayDisplay(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}

unsafe extern "system" fn ddraw_surface2_UpdateOverlayDisplay(iface: *mut IDirectDrawSurface2, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    return ddraw_surface7_UpdateOverlayDisplay(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}

unsafe extern "system" fn ddraw_surface1_UpdateOverlayDisplay(iface: *mut IDirectDrawSurface, flags: DWORD) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface7_UpdateOverlayDisplay(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), flags);
}
unsafe extern "system" fn ddraw_surface7_UpdateOverlayZOrder(
    iface: *mut IDirectDrawSurface7,
    flags: DWORD,
    reference: *mut IDirectDrawSurface7,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    wined3d_mutex_lock();
    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_OVERLAY) == 0 {
        WARN!("Not an overlay surface.\n");
        wined3d_mutex_unlock();
        return DDERR_NOTAOVERLAYSURFACE;
    }
    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface4_UpdateOverlayZOrder(iface: *mut IDirectDrawSurface4, flags: DWORD, reference: *mut IDirectDrawSurface4) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let reference_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(reference);

    return ddraw_surface7_UpdateOverlayZOrder(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        if !reference_impl.is_null() {
            &raw mut (*reference_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
    );
}
unsafe extern "system" fn ddraw_surface3_UpdateOverlayZOrder(
    mut iface: *mut IDirectDrawSurface3,
    mut flags: DWORD,
    mut reference: *mut IDirectDrawSurface3,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut reference_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(reference);

    return ddraw_surface7_UpdateOverlayZOrder(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        if !reference_impl.is_null() {
            &raw mut (*reference_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
    );
}

unsafe extern "system" fn ddraw_surface2_UpdateOverlayZOrder(
    iface: *mut IDirectDrawSurface2,
    flags: DWORD,
    reference: *mut IDirectDrawSurface2,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let reference_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(reference);
    WINE_TRACE!("iface %p, flags %#lx, reference %p.\n", iface, flags, reference);
    return ddraw_surface7_UpdateOverlayZOrder(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        flags,
        if !reference_impl.is_null() {
            core::ptr::addr_of_mut!((*reference_impl).IDirectDrawSurface7_iface)
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
    );
}
unsafe extern "system" fn ddraw_surface1_UpdateOverlayZOrder(
    mut iface: *mut IDirectDrawSurface,
    mut flags: DWORD,
    mut reference: *mut IDirectDrawSurface,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut reference_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(reference);
    WINE_TRACE!("iface %p, flags %#lx, reference %p.\n", iface, flags, reference);
    return ddraw_surface7_UpdateOverlayZOrder(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        flags,
        if !reference_impl.is_null() {
            &raw mut (*reference_impl).IDirectDrawSurface7_iface
        } else {
            ::core::ptr::null_mut::<IDirectDrawSurface7>()
        },
    );
}
unsafe extern "system" fn ddraw_surface7_GetDDInterface(mut iface: *mut IDirectDrawSurface7, mut DD: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    TRACE!("iface %p, ddraw %p.\n", iface, DD);

    if DD.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    match (*This).version {
        7 => {
            *DD = &raw mut (*(*This).ddraw).IDirectDraw7_iface as *mut core::ffi::c_void;
        }
        4 => {
            *DD = &raw mut (*(*This).ddraw).IDirectDraw4_iface as *mut core::ffi::c_void;
        }
        2 => {
            *DD = &raw mut (*(*This).ddraw).IDirectDraw2_iface as *mut core::ffi::c_void;
        }
        1 => {
            *DD = &raw mut (*(*This).ddraw).IDirectDraw_iface as *mut core::ffi::c_void;
        }
        _ => {}
    }
    IUnknown_AddRef(*DD as *mut IUnknown);

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface4_GetDDInterface(iface: *mut IDirectDrawSurface4, ddraw: *mut *mut core::ffi::c_void) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    return ddraw_surface7_GetDDInterface(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), ddraw);
}
unsafe extern "system" fn ddraw_surface3_GetDDInterface(
    mut iface: *mut IDirectDrawSurface3,
    mut ddraw: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, ddraw %p.\n", iface, ddraw);
    return ddraw_surface7_GetDDInterface(&raw mut (*surface).IDirectDrawSurface7_iface, ddraw);
}

unsafe extern "system" fn ddraw_surface2_GetDDInterface(
    mut iface: *mut IDirectDrawSurface2,
    mut ddraw: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, ddraw %p.\n", iface, ddraw);
    return ddraw_surface7_GetDDInterface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        ddraw,
    );
}
unsafe extern "system" fn ddraw_surface7_ChangeUniquenessValue(
    iface: *mut IDirectDrawSurface7,
) -> HRESULT {
    // TRACE("iface %p.\n", iface);
    DD_OK
}

unsafe extern "system" fn ddraw_surface4_ChangeUniquenessValue(iface: *mut IDirectDrawSurface4) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_ChangeUniquenessValue(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface));
}
unsafe extern "system" fn ddraw_surface7_GetUniquenessValue(iface: *mut IDirectDrawSurface7, pValue: *mut DWORD) -> HRESULT {
    *pValue = 0;

    return DD_OK;
}

unsafe extern "system" fn ddraw_surface4_GetUniquenessValue(iface: *mut IDirectDrawSurface4, pValue: *mut DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    TRACE!("iface %p, value %p.\n", iface, pValue);
    return ddraw_surface7_GetUniquenessValue(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), pValue);
}
unsafe extern "system" fn ddraw_surface7_SetLOD(
    mut iface: *mut IDirectDrawSurface7,
    mut MaxLOD: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut device: *mut d3d_device = ::core::ptr::null_mut::<d3d_device>();
    WINE_TRACE!("iface %p, lod %lu.\n", iface, MaxLOD);
    wined3d_mutex_lock();
    if ((*surface).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_TEXTUREMANAGE) == 0 {
        wined3d_mutex_unlock();
        return DDERR_INVALIDOBJECT;
    }

    wined3d_texture_set_lod((*surface).wined3d_texture, MaxLOD as ::core::ffi::c_uint);
    if !(*surface).draw_texture.is_null() {
        wined3d_texture_set_lod((*surface).draw_texture, MaxLOD as ::core::ffi::c_uint);
    }

    device = ((*(*surface).ddraw).d3ddevice_list.next as *mut ::core::ffi::c_char).offset(
        -(core::mem::offset_of!(d3d_device, ddraw_entry) as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut d3d_device;
    while &raw mut (*device).ddraw_entry != &raw mut (*(*surface).ddraw).d3ddevice_list {
        wined3d_stateblock_texture_changed((*device).state, (*surface).wined3d_texture);
        if !(*surface).draw_texture.is_null() {
            wined3d_stateblock_texture_changed((*device).state, (*surface).draw_texture);
        }
        device = ((*device).ddraw_entry.next as *mut ::core::ffi::c_char).offset(
            -(core::mem::offset_of!(d3d_device, ddraw_entry)
                as ::core::ffi::c_ulonglong as isize),
        ) as *mut d3d_device;
    }
    wined3d_mutex_unlock();
    return DD_OK;
}
unsafe extern "system" fn ddraw_surface7_GetLOD(
    mut iface: *mut IDirectDrawSurface7,
    mut MaxLOD: *mut DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    WINE_TRACE!("iface %p, lod %p.\n", iface, MaxLOD);
    if MaxLOD.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    wined3d_mutex_lock();
    if ((*surface).surface_desc.ddsCaps.dwCaps2 & (DDSCAPS2_TEXTUREMANAGE as DWORD)) == 0 {
        wined3d_mutex_unlock();
        return DDERR_INVALIDOBJECT;
    }
    *MaxLOD = wined3d_texture_get_lod((*surface).wined3d_texture) as DWORD;
    wined3d_mutex_unlock();
    return DD_OK;
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface7_BltFast(
    mut iface: *mut IDirectDrawSurface7,
    mut dst_x: DWORD,
    mut dst_y: DWORD,
    mut src_surface: *mut IDirectDrawSurface7,
    mut src_rect: *mut RECT,
    mut trans: DWORD,
) -> HRESULT {
    let mut dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface7(src_surface);
    let mut flags: DWORD = WINED3D_BLT_SYNCHRONOUS as DWORD;
    let mut src_w: DWORD = 0;
    let mut src_h: DWORD = 0;
    let mut dst_w: DWORD = 0;
    let mut dst_h: DWORD = 0;
    let mut hr: HRESULT = DD_OK as HRESULT;
    let mut dst_rect: RECT = core::mem::zeroed();
    let mut s: RECT = core::mem::zeroed();

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        TRACE!(
            "iface %p, dst_x %lu, dst_y %lu, src_surface %p, src_rect %s, flags %#lx.\n",
            iface,
            dst_x,
            dst_y,
            src_surface,
            wine_dbgstr_rect(src_rect),
            trans
        );
    }

    dst_w = (*dst_impl).surface_desc.dwWidth;
    dst_h = (*dst_impl).surface_desc.dwHeight;

    if src_rect.is_null() {
        SetRect(
            &raw mut s,
            0 as ::core::ffi::c_int,
            0 as ::core::ffi::c_int,
            (*src_impl).surface_desc.dwWidth as ::core::ffi::c_int,
            (*src_impl).surface_desc.dwHeight as ::core::ffi::c_int,
        );
        src_rect = &raw mut s;
    }

    src_w = ((*src_rect).right - (*src_rect).left) as DWORD;
    src_h = ((*src_rect).bottom - (*src_rect).top) as DWORD;
    if src_w > dst_w
        || dst_x > dst_w.wrapping_sub(src_w)
        || src_h > dst_h
        || dst_y > dst_h.wrapping_sub(src_h)
    {
        WARN!("Destination area out of bounds, returning DDERR_INVALIDRECT.\n");
        return DDERR_INVALIDRECT as HRESULT;
    }

    SetRect(
        &raw mut dst_rect,
        dst_x as ::core::ffi::c_int,
        dst_y as ::core::ffi::c_int,
        dst_x.wrapping_add(src_w) as ::core::ffi::c_int,
        dst_y.wrapping_add(src_h) as ::core::ffi::c_int,
    );
    if trans & (DDBLTFAST_SRCCOLORKEY as DWORD) != 0 {
        flags |= WINED3D_BLT_SRC_CKEY as DWORD;
    }
    if trans & (DDBLTFAST_DESTCOLORKEY as DWORD) != 0 {
        flags |= WINED3D_BLT_DST_CKEY as DWORD;
    }
    if trans & (DDBLTFAST_WAIT as DWORD) != 0 {
        flags |= WINED3D_BLT_WAIT as DWORD;
    }
    if trans & (DDBLTFAST_DONOTWAIT as DWORD) != 0 {
        flags |= WINED3D_BLT_DO_NOT_WAIT as DWORD;
    }

    wined3d_mutex_lock();
    if !(*dst_impl).clipper.is_null() {
        wined3d_mutex_unlock();
        WARN!("Destination surface has a clipper set, returning DDERR_BLTFASTCANTCLIP.\n");
        return DDERR_BLTFASTCANTCLIP as HRESULT;
    }

    if (*src_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD) != 0 {
        hr = ddraw_surface_update_frontbuffer(
            src_impl,
            src_rect,
            TRUE as BOOL,
            0 as ::core::ffi::c_uint,
        );
    }
    if hr >= (0 as HRESULT) {
        hr = wined3d_device_context_blt(
            (*(*dst_impl).ddraw).immediate_context,
            ddraw_surface_get_any_texture(dst_impl, DDRAW_SURFACE_RW as ::core::ffi::c_uint),
            (*dst_impl).sub_resource_idx,
            &raw mut dst_rect,
            ddraw_surface_get_any_texture(src_impl, DDRAW_SURFACE_READ as ::core::ffi::c_uint),
            (*src_impl).sub_resource_idx,
            src_rect,
            flags as ::core::ffi::c_uint,
            ::core::ptr::null::<wined3d_blt_fx>(),
            WINED3D_TEXF_POINT as _,
        );
    }
    if (hr >= (0 as HRESULT))
        && ((*dst_impl).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD) != 0)
    {
        hr = ddraw_surface_update_frontbuffer(
            dst_impl,
            &raw mut dst_rect,
            FALSE as BOOL,
            0 as ::core::ffi::c_uint,
        );
    }
    wined3d_mutex_unlock();

    match hr {
        WINED3DERR_NOTAVAILABLE => return DDERR_UNSUPPORTED as HRESULT,
        _ => return hr,
    }
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface4_BltFast(iface: *mut IDirectDrawSurface4, dst_x: DWORD, dst_y: DWORD, src_surface: *mut IDirectDrawSurface4, src_rect: *mut RECT, flags: DWORD) -> HRESULT {
    let dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    let src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface4(src_surface);

    return ddraw_surface7_BltFast(
        &raw mut (*dst_impl).IDirectDrawSurface7_iface,
        dst_x,
        dst_y,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        src_rect,
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface3_BltFast(iface: *mut IDirectDrawSurface3, dst_x: DWORD, dst_y: DWORD, src_surface: *mut IDirectDrawSurface3, src_rect: *mut RECT, flags: DWORD) -> HRESULT {
    let mut dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface3(src_surface);

    return ddraw_surface7_BltFast(
        &raw mut (*dst_impl).IDirectDrawSurface7_iface,
        dst_x,
        dst_y,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface7_iface
        } else {
            core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        src_rect,
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface2_BltFast(
    mut iface: *mut IDirectDrawSurface2,
    mut dst_x: DWORD,
    mut dst_y: DWORD,
    mut src_surface: *mut IDirectDrawSurface2,
    mut src_rect: *mut RECT,
    mut flags: DWORD,
) -> HRESULT {
    let mut dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface2(src_surface);
    WINE_TRACE!("iface %p, dst_x %lu, dst_y %lu, src_surface %p, src_rect %s, flags %#lx.\n", iface, dst_x, dst_y, src_surface, wine_dbgstr_rect(src_rect), flags);
    return ddraw_surface7_BltFast(
        &raw mut (*dst_impl).IDirectDrawSurface7_iface,
        dst_x,
        dst_y,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface7_iface
        } else {
            ::core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        src_rect,
        flags,
    );
}
#[no_mangle]
pub unsafe extern "system" fn ddraw_surface1_BltFast(
    mut iface: *mut IDirectDrawSurface,
    mut dst_x: DWORD,
    mut dst_y: DWORD,
    mut src_surface: *mut IDirectDrawSurface,
    mut src_rect: *mut RECT,
    mut flags: DWORD,
) -> HRESULT {
    let mut dst_impl: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    let mut src_impl: *mut ddraw_surface = unsafe_impl_from_IDirectDrawSurface(src_surface);
    WINE_TRACE!("iface %p, dst_x %lu, dst_y %lu, src_surface %p, src_rect %s, flags %#lx.\n", iface, dst_x, dst_y, src_surface, wine_dbgstr_rect(src_rect), flags);
    return ddraw_surface7_BltFast(
        &raw mut (*dst_impl).IDirectDrawSurface7_iface,
        dst_x,
        dst_y,
        if !src_impl.is_null() {
            &raw mut (*src_impl).IDirectDrawSurface7_iface
        } else {
            ::core::ptr::null_mut::<IDirectDrawSurface7>()
        },
        src_rect,
        flags,
    );
}
unsafe extern "system" fn ddraw_surface7_GetClipper(iface: *mut IDirectDrawSurface7, clipper: *mut *mut IDirectDrawClipper) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    if clipper.is_null() {
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if (*surface).clipper.is_null() {
        wined3d_mutex_unlock();
        *clipper = core::ptr::null_mut();
        return DDERR_NOCLIPPERATTACHED;
    }

    *clipper = core::ptr::addr_of_mut!((*(*surface).clipper).IDirectDrawClipper_iface);
    if ddraw_clipper_is_valid((*surface).clipper) != 0 {
        IDirectDrawClipper_AddRef(*clipper);
    }
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_GetClipper(
    mut iface: *mut IDirectDrawSurface4,
    mut clipper: *mut *mut IDirectDrawClipper,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, clipper %p.\n", iface, clipper);
    return ddraw_surface7_GetClipper(&raw mut (*surface).IDirectDrawSurface7_iface, clipper);
}

unsafe extern "system" fn ddraw_surface3_GetClipper(iface: *mut IDirectDrawSurface3, clipper: *mut *mut IDirectDrawClipper) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface7_GetClipper(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), clipper);
}
unsafe extern "system" fn ddraw_surface2_GetClipper(
    iface: *mut IDirectDrawSurface2,
    clipper: *mut *mut IDirectDrawClipper,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, clipper %p.\n", iface, clipper);
    return ddraw_surface7_GetClipper(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), clipper);
}
unsafe extern "system" fn ddraw_surface1_GetClipper(
    mut iface: *mut IDirectDrawSurface,
    mut clipper: *mut *mut IDirectDrawClipper,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, clipper %p.\n", iface, clipper);
    return ddraw_surface7_GetClipper(&raw mut (*surface).IDirectDrawSurface7_iface, clipper);
}
unsafe extern "system" fn ddraw_surface7_SetClipper(
    mut iface: *mut IDirectDrawSurface7,
    mut iclipper: *mut IDirectDrawClipper,
) -> HRESULT {
    let mut This: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut clipper: *mut ddraw_clipper = unsafe_impl_from_IDirectDrawClipper(iclipper);
    let mut old_clipper: *mut ddraw_clipper = (*This).clipper as *mut ddraw_clipper;
    let mut clipWindow: HWND = ::core::ptr::null_mut::<HWND__>();
    WINE_TRACE!("iface %p, clipper %p.\n", iface, iclipper);
    wined3d_mutex_lock();
    if clipper == (*This).clipper {
        wined3d_mutex_unlock();
        return DD_OK;
    }
    (*This).clipper = clipper as *mut ddraw_clipper;
    if !clipper.is_null() {
        IDirectDrawClipper_AddRef(iclipper);
    }
    if !old_clipper.is_null() && ddraw_clipper_is_valid(old_clipper) != 0 {
        IDirectDrawClipper_Release(core::ptr::addr_of_mut!((*old_clipper).IDirectDrawClipper_iface));
    }
    if ((*This).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0
        && !(*(*This).ddraw).wined3d_swapchain.is_null()
    {
        clipWindow = ::core::ptr::null_mut::<HWND__>();
        if !clipper.is_null() {
            IDirectDrawClipper_GetHWnd(iclipper, core::ptr::addr_of_mut!(clipWindow));
        }
        if !clipWindow.is_null() {
            wined3d_swapchain_set_window((*(*This).ddraw).wined3d_swapchain, clipWindow);
            ddraw_set_swapchain_window((*This).ddraw, clipWindow);
        } else {
            wined3d_swapchain_set_window(
                (*(*This).ddraw).wined3d_swapchain,
                (*(*This).ddraw).d3d_window,
            );
            ddraw_set_swapchain_window((*This).ddraw, (*(*This).ddraw).dest_window);
        }
    }
    wined3d_mutex_unlock();
    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_SetClipper(iface: *mut IDirectDrawSurface4, clipper: *mut IDirectDrawClipper) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_SetClipper(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), clipper);
}
unsafe extern "system" fn ddraw_surface3_SetClipper(
    iface: *mut IDirectDrawSurface3,
    clipper: *mut IDirectDrawClipper,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, clipper %p.\n", iface, clipper);
    return ddraw_surface7_SetClipper(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        clipper,
    );
}
unsafe extern "system" fn ddraw_surface2_SetClipper(
    mut iface: *mut IDirectDrawSurface2,
    mut clipper: *mut IDirectDrawClipper,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, clipper %p.\n", iface, clipper);
    return ddraw_surface7_SetClipper(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), clipper);
}
unsafe extern "system" fn ddraw_surface1_SetClipper(mut iface: *mut IDirectDrawSurface, mut clipper: *mut IDirectDrawClipper) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    return ddraw_surface7_SetClipper(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), clipper);
}

unsafe extern "C" fn ddraw_surface_set_wined3d_textures_colour_key(
    surface: *mut ddraw_surface,
    flags: DWORD,
    color_key: *mut wined3d_color_key,
) -> HRESULT {
    let mut device: *mut d3d_device = core::ptr::null_mut();
    let mut hr: HRESULT;
    hr = wined3d_texture_set_color_key((*surface).wined3d_texture, flags, color_key);
    if !(*surface).draw_texture.is_null() && hr >= 0 {
        hr = wined3d_texture_set_color_key((*surface).draw_texture, flags, color_key);
    }
    device = ((*(*surface).ddraw).d3ddevice_list.next as *mut u8).offset(
        -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
    ) as *mut d3d_device;
    while core::ptr::addr_of_mut!((*device).ddraw_entry)
        != core::ptr::addr_of_mut!((*(*surface).ddraw).d3ddevice_list)
    {
        wined3d_stateblock_texture_changed((*device).state, (*surface).wined3d_texture);
        if !(*surface).draw_texture.is_null() {
            wined3d_stateblock_texture_changed((*device).state, (*surface).draw_texture);
        }
        device = ((*device).ddraw_entry.next as *mut u8).offset(
            -(core::mem::offset_of!(d3d_device, ddraw_entry) as isize),
        ) as *mut d3d_device;
    }
    return hr;
}

unsafe extern "C" fn ddraw_surface_sync_color_keys(surface: *mut ddraw_surface) {
    let desc: *const DDSURFACEDESC2 = core::ptr::addr_of!((*surface).surface_desc);
    if (*desc).dwFlags & DDSD_CKDESTOVERLAY != 0 {
        ddraw_surface_set_wined3d_textures_colour_key(
            surface,
            DDCKEY_DESTOVERLAY,
            core::ptr::addr_of!((*desc).__bindgen_anon_4.ddckCKDestOverlay) as *mut wined3d_color_key,
        );
    }
    if (*desc).dwFlags & DDSD_CKDESTBLT != 0 {
        ddraw_surface_set_wined3d_textures_colour_key(
            surface,
            DDCKEY_DESTBLT,
            core::ptr::addr_of!((*desc).ddckCKDestBlt) as *mut wined3d_color_key,
        );
    }
    if (*desc).dwFlags & DDSD_CKSRCOVERLAY != 0 {
        ddraw_surface_set_wined3d_textures_colour_key(
            surface,
            DDCKEY_SRCOVERLAY,
            core::ptr::addr_of!((*desc).ddckCKSrcOverlay) as *mut wined3d_color_key,
        );
    }
    if (*desc).dwFlags & DDSD_CKSRCBLT != 0 {
        ddraw_surface_set_wined3d_textures_colour_key(
            surface,
            DDCKEY_SRCBLT,
            core::ptr::addr_of!((*desc).ddckCKSrcBlt) as *mut wined3d_color_key,
        );
    }
}

unsafe extern "system" fn ddraw_surface7_SetSurfaceDesc(iface: *mut IDirectDrawSurface7, DDSD: *mut DDSURFACEDESC2, Flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut current_format_id: wined3d_format_id = WINED3DFMT_UNKNOWN;
    let mut format_id: wined3d_format_id = WINED3DFMT_UNKNOWN;
    let mut hr: HRESULT = 0;
    let allowed_flags: DWORD = (DDSD_LPSURFACE | DDSD_PIXELFORMAT | DDSD_WIDTH
        | DDSD_HEIGHT | DDSD_PITCH | DDSD_CAPS) as DWORD;
    let mut pitch: UINT = 0;
    let mut width: UINT = 0;
    let mut height: UINT = 0;

    if DDSD.is_null() {
        WARN!("DDSD is NULL, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }
    if Flags != 0 {
        WARN!("Flags is %lx, returning DDERR_INVALIDPARAMS\n", Flags);
        return DDERR_INVALIDPARAMS;
    }
    if ((*surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_SYSTEMMEMORY as DWORD)) == 0
        || ((*surface).surface_desc.ddsCaps.dwCaps & (DDSCAPS_PRIMARYSURFACE as DWORD)) != 0
        || ((*surface).surface_desc.ddsCaps.dwCaps2 & ((DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) as DWORD)) != 0
    {
        WARN!("Surface is not in system memory, returning DDERR_INVALIDSURFACETYPE.\n");
        return DDERR_INVALIDSURFACETYPE;
    }

    if ((*DDSD).dwFlags & !allowed_flags) != 0 {
        WARN!("Invalid flags %#lx set, returning DDERR_INVALIDPARAMS\n", (*DDSD).dwFlags);
        return DDERR_INVALIDPARAMS;
    }
    if ((*DDSD).dwFlags & (DDSD_LPSURFACE as DWORD)) == 0
        || (*DDSD).lpSurface.is_null()
    {
        WARN!("DDSD_LPSURFACE is not set or lpSurface is NULL, returning DDERR_INVALIDPARAMS\n");
        return DDERR_INVALIDPARAMS;
    }
    if ((*DDSD).dwFlags & (DDSD_CAPS as DWORD)) != 0
        && (*DDSD).ddsCaps.dwCaps != 0
    {
        WARN!("DDSD_CAPS is set, returning DDERR_INVALIDCAPS.\n");
        return DDERR_INVALIDCAPS;
    }
    if ((*DDSD).dwFlags & (DDSD_WIDTH as DWORD)) != 0 {
        if ((*DDSD).dwFlags & (DDSD_PITCH as DWORD)) == 0 {
            WARN!("DDSD_WIDTH is set, but DDSD_PITCH is not, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }
        if (*DDSD).dwWidth == 0
            || (*DDSD).__bindgen_anon_1.lPitch <= 0
            || ((*DDSD).__bindgen_anon_1.lPitch & 0x3) != 0
        {
            WARN!("Pitch is %ld, width is %lu, returning DDERR_INVALIDPARAMS.\n",
                (*DDSD).__bindgen_anon_1.lPitch, (*DDSD).dwWidth);
            return DDERR_INVALIDPARAMS;
        }
        if (*DDSD).dwWidth != (*surface).surface_desc.dwWidth {
            TRACE!("Surface width changed from %lu to %lu.\n", (*surface).surface_desc.dwWidth, (*DDSD).dwWidth);
        }
        if (*DDSD).__bindgen_anon_1.lPitch != (*surface).surface_desc.__bindgen_anon_1.lPitch {
            TRACE!("Surface pitch changed from %lu to %lu.\n", (*surface).surface_desc.__bindgen_anon_1.lPitch, (*DDSD).__bindgen_anon_1.lPitch);
        }
        pitch = (*DDSD).__bindgen_anon_1.lPitch as UINT;
        width = (*DDSD).dwWidth as UINT;
    } else if ((*DDSD).dwFlags & (DDSD_PITCH as DWORD)) != 0 {
        WARN!("DDSD_PITCH is set, but DDSD_WIDTH is not, returning DDERR_INVALIDPARAMS.\n");
        return DDERR_INVALIDPARAMS;
    } else {
        pitch = (*surface).surface_desc.__bindgen_anon_1.lPitch as UINT;
        width = (*surface).surface_desc.dwWidth as UINT;
    }

    if ((*DDSD).dwFlags & (DDSD_HEIGHT as DWORD)) != 0 {
        if (*DDSD).dwHeight == 0 {
            WARN!("Height is 0, returning DDERR_INVALIDPARAMS.\n");
            return DDERR_INVALIDPARAMS;
        }
        if (*DDSD).dwHeight != (*surface).surface_desc.dwHeight {
            TRACE!("Surface height changed from %lu to %lu.\n", (*surface).surface_desc.dwHeight, (*DDSD).dwHeight);
        }
        height = (*DDSD).dwHeight as UINT;
    } else {
        height = (*surface).surface_desc.dwHeight as UINT;
    }

    wined3d_mutex_lock();
    if ((*DDSD).dwFlags & (DDSD_PIXELFORMAT as DWORD)) != 0 {
        format_id = wined3dformat_from_ddrawformat(core::ptr::addr_of_mut!((*DDSD).__bindgen_anon_5.ddpfPixelFormat));

        if format_id == WINED3DFMT_UNKNOWN {
            ERR!("Requested to set an unknown pixelformat\n");
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }
        current_format_id = wined3dformat_from_ddrawformat(core::ptr::addr_of_mut!((*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat));
        if format_id != current_format_id {
            TRACE!("Surface format changed from %#x to %#x.\n", current_format_id, format_id);
        }
    } else {
        format_id = wined3dformat_from_ddrawformat(core::ptr::addr_of_mut!((*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat));
        current_format_id = format_id;
    }

    if width == (*surface).surface_desc.dwWidth && height == (*surface).surface_desc.dwHeight
        && format_id == current_format_id
    {
        /* Updating memory only. */

        hr = wined3d_texture_update_desc((*surface).wined3d_texture,
            (*surface).sub_resource_idx, (*DDSD).lpSurface as *mut ::core::ffi::c_void, pitch);
        if hr < 0 {
            WARN!("Failed to update surface desc, hr %#lx.\n", hr);
            wined3d_mutex_unlock();
            return hr_ddraw_from_wined3d(hr);
        }
    } else {
        let mut ddraw_texture: *mut ddraw_texture = wined3d_texture_get_parent((*surface).wined3d_texture) as *mut ddraw_texture;
        let mut old_rtv: *mut wined3d_rendertarget_view = (*surface).wined3d_rtv;
        let mut old_draw_texture: *mut wined3d_texture = (*surface).draw_texture;
        let mut old_texture: *mut wined3d_texture = (*surface).wined3d_texture;
        let mut new_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
        let mut new_draw_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
        let mut wined3d_desc: wined3d_resource_desc = core::mem::zeroed();
        let mut ddraw: *mut ddraw = (*surface).ddraw;

        /* Updating surface attributes; recreate the texture. */

        if wined3d_texture_get_level_count(old_texture) > 1
            || ((*surface).surface_desc.ddsCaps.dwCaps2 & (DDSCAPS2_CUBEMAP_ALLFACES as DWORD)) != 0
        {
            FIXME!("Texture has multiple sub-resources, not supported.\n");
            wined3d_mutex_unlock();
            return DDERR_INVALIDPARAMS;
        }

        /* Create the new textures. */

        wined3d_resource_get_desc(wined3d_texture_get_resource(old_texture), core::ptr::addr_of_mut!(wined3d_desc));

        wined3d_desc.width = width;
        wined3d_desc.height = height;
        wined3d_desc.format = format_id;

        hr = wined3d_texture_create((*ddraw).wined3d_device, core::ptr::addr_of_mut!(wined3d_desc),
            1 as UINT, 1 as UINT,
            WINED3D_TEXTURE_CREATE_GET_DC_LENIENT as uint32_t, core::ptr::null::<wined3d_sub_resource_data>(),
            core::ptr::null_mut::<::core::ffi::c_void>(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
            core::ptr::addr_of_mut!(new_texture));
        if hr < 0 {
            ERR!("Failed to create texture, hr %#lx.\n", hr);
            wined3d_mutex_unlock();
            return hr_ddraw_from_wined3d(hr);
        }

        hr = wined3d_texture_update_desc(new_texture, 0 as UINT,
            (*DDSD).lpSurface as *mut ::core::ffi::c_void, pitch);
        if hr < 0 {
            ERR!("Failed to set user memory, hr %#lx.\n", hr);
            wined3d_texture_decref(new_texture);
            wined3d_mutex_unlock();
            return hr_ddraw_from_wined3d(hr);
        }

        if !old_draw_texture.is_null() {
            wined3d_resource_get_desc(wined3d_texture_get_resource(old_draw_texture), core::ptr::addr_of_mut!(wined3d_desc));

            wined3d_desc.width = width;
            wined3d_desc.height = height;
            wined3d_desc.format = format_id;

            hr = wined3d_texture_create((*ddraw).wined3d_device, core::ptr::addr_of_mut!(wined3d_desc),
                1 as UINT, 1 as UINT,
                0 as uint32_t, core::ptr::null::<wined3d_sub_resource_data>(),
                core::ptr::null_mut::<::core::ffi::c_void>(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
                core::ptr::addr_of_mut!(new_draw_texture));
            if hr < 0 {
                ERR!("Failed to create draw texture, hr %#lx.\n", hr);
                wined3d_texture_decref(new_texture);
                wined3d_mutex_unlock();
                return hr_ddraw_from_wined3d(hr);
            }
        }

        wined3d_resource_set_parent(wined3d_texture_get_resource(old_texture),
            core::ptr::null_mut(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));
        wined3d_texture_set_sub_resource_parent(old_texture, 0 as UINT, core::ptr::null_mut(),
            core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));

        if !(*surface).draw_texture.is_null() {
            wined3d_resource_set_parent(wined3d_texture_get_resource(old_draw_texture),
                core::ptr::null_mut(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));
            wined3d_texture_set_sub_resource_parent(old_draw_texture, 0 as UINT, core::ptr::null_mut(),
                core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));

            wined3d_resource_set_parent(wined3d_texture_get_resource(new_draw_texture),
                ddraw_texture as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_texture_wined3d_parent_ops));
            wined3d_resource_set_parent(wined3d_texture_get_resource(new_texture),
                ddraw_texture as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));

            wined3d_texture_set_sub_resource_parent(new_draw_texture, 0 as UINT,
                surface as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_surface_wined3d_parent_ops));
            wined3d_texture_set_sub_resource_parent(new_texture, 0 as UINT,
                surface as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));

            wined3d_texture_decref(old_draw_texture);
        } else {
            wined3d_resource_set_parent(wined3d_texture_get_resource(new_texture),
                ddraw_texture as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_texture_wined3d_parent_ops));

            wined3d_texture_set_sub_resource_parent(new_texture, 0 as UINT,
                surface as *mut ::core::ffi::c_void, core::ptr::addr_of!(ddraw_surface_wined3d_parent_ops));
        }

        (*surface).wined3d_texture = new_texture;
        (*surface).draw_texture = new_draw_texture;

        wined3d_texture_decref(old_texture);

        /* Don't try to replace existing bound textures. Testing implies that
         * SetSurfaceDesc() doesn't quite work with texture surfaces anyway:
         * it succeeds, and mapping the surface does return the new user memory,
         * but drawing from the surface doesn't actually sample from that
         * memory. */

        /* Destroy the existing RTV. */

        if !old_rtv.is_null() {
            let mut current_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut::<wined3d_rendertarget_view>();
            let mut new_rtv: *mut wined3d_rendertarget_view = core::ptr::null_mut::<wined3d_rendertarget_view>();

            (*surface).wined3d_rtv = core::ptr::null_mut::<wined3d_rendertarget_view>();

            current_rtv = wined3d_device_context_get_rendertarget_view((*ddraw).immediate_context, 0 as UINT);
            if old_rtv == current_rtv {
                new_rtv = ddraw_surface_get_rendertarget_view(surface);
                wined3d_device_context_set_rendertarget_views((*ddraw).immediate_context, 0 as UINT,
                    1 as UINT, core::ptr::addr_of_mut!(new_rtv), (FALSE) as BOOL);
            }

            wined3d_rendertarget_view_set_parent(old_rtv, core::ptr::null_mut(), core::ptr::addr_of!(ddraw_null_wined3d_parent_ops));
            wined3d_rendertarget_view_decref(old_rtv);
        }

        ddraw_surface_sync_color_keys(surface);

        /* No need to sync the LOD or overlays; neither can be set on sysmem
         * surfaces, and SetSurfaceDesc() can only be used on sysmem surfaces.
         *
         * No need to sync the palette either, since wined3d doesn't track it
         * except on primary surfaces, and SetSurfaceDesc() is illegal on those
         * too. */
    }

    if ((*DDSD).dwFlags & (DDSD_WIDTH as DWORD)) != 0 {
        (*surface).surface_desc.dwWidth = width as DWORD;
    }
    if ((*DDSD).dwFlags & (DDSD_PITCH as DWORD)) != 0 {
        (*surface).surface_desc.__bindgen_anon_1.lPitch = (*DDSD).__bindgen_anon_1.lPitch;
    }
    if ((*DDSD).dwFlags & (DDSD_HEIGHT as DWORD)) != 0 {
        (*surface).surface_desc.dwHeight = height as DWORD;
    }
    if ((*DDSD).dwFlags & (DDSD_PIXELFORMAT as DWORD)) != 0 {
        (*surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat = (*DDSD).__bindgen_anon_5.ddpfPixelFormat;
    }

    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn ddraw_surface4_SetSurfaceDesc(iface: *mut IDirectDrawSurface4, surface_desc: *mut DDSURFACEDESC2, flags: DWORD) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface7_SetSurfaceDesc(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), surface_desc, flags);
}
unsafe extern "system" fn ddraw_surface3_SetSurfaceDesc(
    mut iface: *mut IDirectDrawSurface3,
    mut surface_desc: *mut DDSURFACEDESC,
    mut flags: DWORD,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    let mut surface_desc2: DDSURFACEDESC2 = core::mem::zeroed::<DDSURFACEDESC2>();
    WINE_TRACE!("iface %p, surface_desc %p, flags %#lx.\n", iface, surface_desc, flags);
    if !surface_desc.is_null() {
        DDSD_to_DDSD2(surface_desc, &raw mut surface_desc2);
    }
    return ddraw_surface7_SetSurfaceDesc(
        &raw mut (*surface).IDirectDrawSurface7_iface,
        if !surface_desc.is_null() {
            &raw mut surface_desc2
        } else {
            ::core::ptr::null_mut::<DDSURFACEDESC2>()
        },
        flags,
    );
}

unsafe extern "system" fn ddraw_surface7_GetPalette(iface: *mut IDirectDrawSurface7, palette: *mut *mut IDirectDrawPalette) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    let mut palette_impl: *mut ddraw_palette = core::ptr::null_mut();
    let mut hr: HRESULT = DD_OK;

    if palette.is_null() {
        return DDERR_INVALIDPARAMS;
    }
    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }

    wined3d_mutex_lock();
    palette_impl = (*surface).palette as *mut ddraw_palette;
    if !palette_impl.is_null() {
        *palette = core::ptr::addr_of_mut!((*palette_impl).IDirectDrawPalette_iface);
        IDirectDrawPalette_AddRef(*palette);
    } else {
        *palette = core::ptr::null_mut();
        hr = DDERR_NOPALETTEATTACHED;
    }
    wined3d_mutex_unlock();

    return hr;
}
unsafe extern "system" fn ddraw_surface4_GetPalette(
    mut iface: *mut IDirectDrawSurface4,
    mut palette: *mut *mut IDirectDrawPalette,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);
    WINE_TRACE!("iface %p, palette %p.\n", iface, palette);
    return ddraw_surface7_GetPalette(&raw mut (*surface).IDirectDrawSurface7_iface, palette);
}
unsafe extern "system" fn ddraw_surface3_GetPalette(
    iface: *mut IDirectDrawSurface3,
    palette: *mut *mut IDirectDrawPalette,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);
    WINE_TRACE!("iface %p, palette %p.\n", iface, palette);
    return ddraw_surface7_GetPalette(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), palette);
}
unsafe extern "system" fn ddraw_surface2_GetPalette(
    mut iface: *mut IDirectDrawSurface2,
    mut palette: *mut *mut IDirectDrawPalette,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, palette %p.\n", iface, palette);
    return ddraw_surface7_GetPalette(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        palette,
    );
}
unsafe extern "system" fn ddraw_surface1_GetPalette(
    mut iface: *mut IDirectDrawSurface,
    mut palette: *mut *mut IDirectDrawPalette,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    WINE_TRACE!("iface %p, palette %p.\n", iface, palette);
    return ddraw_surface7_GetPalette(core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface), palette);
}

unsafe extern "C" fn ddraw_surface_set_color_key(
    mut surface: *mut ddraw_surface,
    mut flags: DWORD,
    mut color_key: *mut DDCOLORKEY,
) -> HRESULT {
    let mut fixed_color_key: DDCOLORKEY = _DDCOLORKEY {
        dwColorSpaceLowValue: 0,
        dwColorSpaceHighValue: 0,
    };
    let mut hr: HRESULT = WINED3D_OK;

    if flags & (DDCKEY_COLORSPACE as DWORD) != 0 {
        if !color_key.is_null()
            && (*color_key).dwColorSpaceLowValue != (*color_key).dwColorSpaceHighValue
        {
            WARN!("Range color keys are not supported, returning DDERR_NOCOLORKEYHW.\n");
            return DDERR_NOCOLORKEYHW;
        }
        flags &= !(DDCKEY_COLORSPACE as DWORD);
    }

    wined3d_mutex_lock();

    if !color_key.is_null() {
        fixed_color_key.dwColorSpaceHighValue = (*color_key).dwColorSpaceLowValue;
        fixed_color_key.dwColorSpaceLowValue = fixed_color_key.dwColorSpaceHighValue;
        match flags & !(DDCKEY_COLORSPACE as DWORD) {
            DDCKEY_DESTBLT => {
                (*surface).surface_desc.ddckCKDestBlt = fixed_color_key;
                (*surface).surface_desc.dwFlags |= DDSD_CKDESTBLT as DWORD;
            }
            DDCKEY_DESTOVERLAY => {
                (*surface).surface_desc.__bindgen_anon_4.ddckCKDestOverlay = fixed_color_key;
                (*surface).surface_desc.dwFlags |= DDSD_CKDESTOVERLAY as DWORD;
            }
            DDCKEY_SRCOVERLAY => {
                (*surface).surface_desc.ddckCKSrcOverlay = fixed_color_key;
                (*surface).surface_desc.dwFlags |= DDSD_CKSRCOVERLAY as DWORD;
            }
            DDCKEY_SRCBLT => {
                (*surface).surface_desc.ddckCKSrcBlt = fixed_color_key;
                (*surface).surface_desc.dwFlags |= DDSD_CKSRCBLT as DWORD;
            }
            _ => {
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }
        }
    } else {
        match flags & !(DDCKEY_COLORSPACE as DWORD) {
            DDCKEY_DESTBLT => {
                (*surface).surface_desc.dwFlags &= !(DDSD_CKDESTBLT as DWORD);
            }
            DDCKEY_DESTOVERLAY => {
                (*surface).surface_desc.dwFlags &= !(DDSD_CKDESTOVERLAY as DWORD);
            }
            DDCKEY_SRCOVERLAY => {
                (*surface).surface_desc.dwFlags &= !(DDSD_CKSRCOVERLAY as DWORD);
            }
            DDCKEY_SRCBLT => {
                (*surface).surface_desc.dwFlags &= !(DDSD_CKSRCBLT as DWORD);
            }
            _ => {
                wined3d_mutex_unlock();
                return DDERR_INVALIDPARAMS;
            }
        }
    }

    if (*surface).is_root() != 0 {
        hr = ddraw_surface_set_wined3d_textures_colour_key(
            surface,
            flags,
            if !color_key.is_null() {
                core::ptr::addr_of_mut!(fixed_color_key) as *mut wined3d_color_key
            } else {
                ::core::ptr::null_mut::<wined3d_color_key>()
            },
        );
    }

    wined3d_mutex_unlock();

    return hr_ddraw_from_wined3d(hr);
}

unsafe extern "system" fn ddraw_surface7_SetColorKey(iface: *mut IDirectDrawSurface7, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);
    WINE_TRACE!("iface %p, flags %#lx, color_key %p.\n", iface, flags, color_key);
    if ((*surface).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_MIPMAPSUBLEVEL) != 0 {
        return DDERR_NOTONMIPMAPSUBLEVEL;
    }
    return ddraw_surface_set_color_key(surface, flags, color_key);
}
unsafe extern "system" fn ddraw_surface4_SetColorKey(iface: *mut IDirectDrawSurface4, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    return ddraw_surface_set_color_key(surface, flags, color_key);
}
unsafe extern "system" fn ddraw_surface3_SetColorKey(iface: *mut IDirectDrawSurface3, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    return ddraw_surface_set_color_key(surface, flags, color_key);
}
unsafe extern "system" fn ddraw_surface2_SetColorKey(iface: *mut IDirectDrawSurface2, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);
    WINE_TRACE!("iface %p, flags %#lx, color_key %p.\n", iface, flags, color_key);
    return ddraw_surface_set_color_key(surface, flags, color_key);
}
unsafe extern "system" fn ddraw_surface1_SetColorKey(iface: *mut IDirectDrawSurface, flags: DWORD, color_key: *mut DDCOLORKEY) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);

    return ddraw_surface_set_color_key(surface, flags, color_key);
}

unsafe extern "system" fn ddraw_surface7_SetPalette(iface: *mut IDirectDrawSurface7, palette: *mut IDirectDrawPalette) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface7(iface);

    if ((*surface).surface_desc.ddsCaps.dwCaps2 & DDSCAPS2_MIPMAPSUBLEVEL) != 0 {
        return DDERR_NOTONMIPMAPSUBLEVEL;
    }
    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }

    return ddraw_surface_set_palette(surface, palette);
}

unsafe extern "system" fn ddraw_surface4_SetPalette(iface: *mut IDirectDrawSurface4, palette: *mut IDirectDrawPalette) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface4(iface);

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }

    return ddraw_surface_set_palette(surface, palette);
}

unsafe extern "system" fn ddraw_surface3_SetPalette(iface: *mut IDirectDrawSurface3, palette: *mut IDirectDrawPalette) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirectDrawSurface3(iface);

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }

    return ddraw_surface_set_palette(surface, palette);
}

unsafe extern "system" fn ddraw_surface2_SetPalette(
    iface: *mut IDirectDrawSurface2,
    palette: *mut IDirectDrawPalette,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface2(iface);

    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }

    return ddraw_surface_set_palette(surface, palette);
}

unsafe extern "system" fn ddraw_surface1_SetPalette(iface: *mut IDirectDrawSurface, palette: *mut IDirectDrawPalette) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawSurface(iface);
    if ddraw_surface_is_lost(surface) != 0 {
        WARN!("Surface lost, returning DDERR_SURFACELOST.\n");
        return DDERR_SURFACELOST;
    }
    return ddraw_surface_set_palette(surface, palette);
}
unsafe extern "system" fn ddraw_gamma_control_GetGammaRamp(iface: *mut IDirectDrawGammaControl, flags: DWORD, gamma_ramp: *mut DDGAMMARAMP) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawGammaControl(iface);

    if gamma_ramp.is_null() {
        WARN!("Invalid gamma_ramp passed.\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if (*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0 {
        /* Note: DDGAMMARAMP is compatible with struct wined3d_gamma_ramp. */
        wined3d_device_get_gamma_ramp((*(*surface).ddraw).wined3d_device, 0, gamma_ramp as *mut wined3d_gamma_ramp);
    } else {
        ERR!("Not implemented for non-primary surfaces.\n");
    }
    wined3d_mutex_unlock();

    return DD_OK;
}

unsafe extern "system" fn ddraw_gamma_control_SetGammaRamp(
    iface: *mut IDirectDrawGammaControl,
    flags: DWORD,
    gamma_ramp: *mut DDGAMMARAMP,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirectDrawGammaControl(iface);

    if gamma_ramp.is_null() {
        WARN!("Invalid gamma_ramp passed.\n");
        return DDERR_INVALIDPARAMS;
    }

    wined3d_mutex_lock();
    if ((*surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE) != 0 {
        /* Note: DDGAMMARAMP is compatible with struct wined3d_gamma_ramp. */
        wined3d_device_set_gamma_ramp(
            (*(*surface).ddraw).wined3d_device,
            0,
            flags,
            gamma_ramp as *mut wined3d_gamma_ramp,
        );
    } else {
        ERR!("Not implemented for non-primary surfaces.\n");
    }
    wined3d_mutex_unlock();

    return DD_OK;
}
unsafe extern "system" fn d3d_texture2_PaletteChanged(
    iface: *mut IDirect3DTexture2,
    start: DWORD,
    count: DWORD,
) -> HRESULT {
    // FIXME("iface %p, start %lu, count %lu stub!\n", iface, start, count);
    D3D_OK
}

unsafe extern "system" fn d3d_texture1_PaletteChanged(
    iface: *mut IDirect3DTexture,
    start: DWORD,
    count: DWORD,
) -> HRESULT {
    let surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);

    return d3d_texture2_PaletteChanged(core::ptr::addr_of_mut!((*surface).IDirect3DTexture2_iface), start, count);
}
unsafe extern "system" fn d3d_texture1_Unload(iface: *mut IDirect3DTexture) -> HRESULT {
    WARN!("iface %p. Not implemented.\n", iface);
    DDERR_UNSUPPORTED
}
unsafe extern "system" fn d3d_texture2_GetHandle(
    iface: *mut IDirect3DTexture2,
    device: *mut IDirect3DDevice2,
    handle: *mut D3DTEXTUREHANDLE,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture2(iface);

    TRACE!("iface %p, device %p, handle %p.\n", iface, device, handle);

    wined3d_mutex_lock();

    if (*surface).Handle == 0 {
        let mut h: DWORD = ddraw_allocate_handle(
            core::ptr::null_mut(),
            surface as *mut core::ffi::c_void,
            DDRAW_HANDLE_SURFACE,
        );
        if h == (DDRAW_INVALID_HANDLE) as u32 {
            ERR!("Failed to allocate a texture handle.\n");
            wined3d_mutex_unlock();
            return DDERR_OUTOFMEMORY;
        }

        (*surface).Handle = h.wrapping_add(1);
    }

    TRACE!("Returning handle %08lx.\n", (*surface).Handle);

    *handle = (*surface).Handle as D3DTEXTUREHANDLE;

    wined3d_mutex_unlock();

    return D3D_OK;
}

unsafe extern "system" fn d3d_texture1_GetHandle(
    mut iface: *mut IDirect3DTexture,
    mut device: *mut IDirect3DDevice,
    mut handle: *mut D3DTEXTUREHANDLE,
) -> HRESULT {
    let mut surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);
    let mut device_impl: *mut d3d_device = unsafe_impl_from_IDirect3DDevice(device);
    WINE_TRACE!("iface %p, device %p, handle %p.\n", iface, device, handle);
    return d3d_texture2_GetHandle(
        &raw mut (*surface).IDirect3DTexture2_iface,
        if !device_impl.is_null() {
            &raw mut (*device_impl).IDirect3DDevice2_iface
        } else {
            ::core::ptr::null_mut::<IDirect3DDevice2>()
        },
        handle,
    );
}

unsafe extern "C" fn get_sub_mimaplevel(surface: *mut ddraw_surface) -> *mut ddraw_surface {
    static mut mipmap_caps: DDSCAPS2 = DDSCAPS2 {
        dwCaps: (DDSCAPS_MIPMAP | DDSCAPS_TEXTURE) as DWORD,
        dwCaps2: 0 as DWORD,
        dwCaps3: 0 as DWORD,
        __bindgen_anon_1: _DDSCAPS2__bindgen_ty_1 { dwCaps4: 0 as DWORD,
        },
    };
    let mut next_level: *mut IDirectDrawSurface7 = core::ptr::null_mut::<IDirectDrawSurface7>();
    let mut hr: HRESULT = 0;
    hr = ddraw_surface7_GetAttachedSurface(
        core::ptr::addr_of_mut!((*surface).IDirectDrawSurface7_iface),
        core::ptr::addr_of_mut!(mipmap_caps),
        core::ptr::addr_of_mut!(next_level),
    );
    if hr < 0 {
        return core::ptr::null_mut::<ddraw_surface>();
    }
    ddraw_surface7_Release(next_level);
    return impl_from_IDirectDrawSurface7(next_level);
}

unsafe extern "system" fn d3d_texture2_Load(iface: *mut IDirect3DTexture2, src_texture: *mut IDirect3DTexture2) -> HRESULT {
    let mut dst_surface: *mut ddraw_surface = impl_from_IDirect3DTexture2(iface);
    let mut src_surface: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture2(src_texture);
    let mut dst_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut src_resource: *mut wined3d_resource = core::ptr::null_mut::<wined3d_resource>();
    let mut hr: HRESULT = 0;

    if src_surface == dst_surface {
        return D3D_OK;
    }

    wined3d_mutex_lock();

    dst_resource = wined3d_texture_get_resource(ddraw_surface_get_default_texture(dst_surface, DDRAW_SURFACE_WRITE));
    src_resource = wined3d_texture_get_resource(ddraw_surface_get_default_texture(src_surface, DDRAW_SURFACE_READ));

    if (((*src_surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_MIPMAP)
        != ((*dst_surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_MIPMAP))
        || ((*src_surface).surface_desc.__bindgen_anon_3.dwMipMapCount
            != (*dst_surface).surface_desc.__bindgen_anon_3.dwMipMapCount)
    {
        ERR!("Trying to load surfaces with different mip-map counts.\n");
    }

    loop {
        let mut dst_pal: *mut ddraw_palette = core::ptr::null_mut::<ddraw_palette>();
        let mut src_pal: *mut ddraw_palette = core::ptr::null_mut::<ddraw_palette>();
        let mut src_desc: *mut DDSURFACEDESC = core::ptr::null_mut::<DDSURFACEDESC>();
        let mut dst_desc: *mut DDSURFACEDESC = core::ptr::null_mut::<DDSURFACEDESC>();

        /* Suppress the ALLOCONLOAD flag */
        (*dst_surface).surface_desc.ddsCaps.dwCaps &= !DDSCAPS_ALLOCONLOAD;

        /* Get the palettes */
        dst_pal = (*dst_surface).palette as *mut ddraw_palette;
        src_pal = (*src_surface).palette as *mut ddraw_palette;

        if !src_pal.is_null() {
            let mut palent: [PALETTEENTRY; 256] = core::mem::zeroed();

            if dst_pal.is_null() {
                wined3d_mutex_unlock();
                return DDERR_NOPALETTEATTACHED;
            }
            IDirectDrawPalette_GetEntries(
                &raw mut (*src_pal).IDirectDrawPalette_iface,
                0,
                0,
                256,
                &raw mut palent as LPPALETTEENTRY,
            );
            IDirectDrawPalette_SetEntries(
                &raw mut (*dst_pal).IDirectDrawPalette_iface,
                0,
                0,
                256,
                &raw mut palent as LPPALETTEENTRY,
            );
        }

        /* Copy one surface on the other */
        dst_desc = &raw mut (*dst_surface).surface_desc as *mut DDSURFACEDESC;
        src_desc = &raw mut (*src_surface).surface_desc as *mut DDSURFACEDESC;

        if ((*src_desc).dwWidth != (*dst_desc).dwWidth) || ((*src_desc).dwHeight != (*dst_desc).dwHeight) {
            /* Should also check for same pixel format, lPitch, ... */
            ERR!("Error in surface sizes.\n");
            wined3d_mutex_unlock();
            return D3DERR_TEXTURE_LOAD_FAILED;
        } else {
            let mut src_map_desc: wined3d_map_desc = core::mem::zeroed();
            let mut dst_map_desc: wined3d_map_desc = core::mem::zeroed();

            /* Copy the src blit color key if the source has one, don't erase
             * the destination's ckey if the source has none */
            if (*src_desc).dwFlags & DDSD_CKSRCBLT != 0 {
                IDirectDrawSurface7_SetColorKey(
                    &raw mut (*dst_surface).IDirectDrawSurface7_iface,
                    DDCKEY_SRCBLT,
                    &raw mut (*src_desc).ddckCKSrcBlt,
                );
            }

            hr = wined3d_resource_map(
                src_resource,
                (*src_surface).sub_resource_idx,
                &raw mut src_map_desc,
                core::ptr::null::<wined3d_box>(),
                WINED3D_MAP_READ,
            );
            if hr < 0 {
                ERR!("Failed to lock source surface, hr %#lx.\n", hr);
                wined3d_mutex_unlock();
                return D3DERR_TEXTURE_LOAD_FAILED;
            }

            hr = wined3d_resource_map(
                dst_resource,
                (*dst_surface).sub_resource_idx,
                &raw mut dst_map_desc,
                core::ptr::null::<wined3d_box>(),
                WINED3D_MAP_WRITE,
            );
            if hr < 0 {
                ERR!("Failed to lock destination surface, hr %#lx.\n", hr);
                wined3d_resource_unmap(src_resource, (*src_surface).sub_resource_idx);
                wined3d_mutex_unlock();
                return D3DERR_TEXTURE_LOAD_FAILED;
            }

            if (*dst_surface).surface_desc.__bindgen_anon_5.ddpfPixelFormat.dwFlags & DDPF_FOURCC != 0 {
                memcpy(
                    dst_map_desc.data,
                    src_map_desc.data,
                    (*src_surface).surface_desc.__bindgen_anon_1.dwLinearSize as usize,
                );
            } else {
                memcpy(
                    dst_map_desc.data,
                    src_map_desc.data,
                    (src_map_desc.row_pitch as DWORD).wrapping_mul((*src_desc).dwHeight) as usize,
                );
            }

            wined3d_resource_unmap(dst_resource, (*dst_surface).sub_resource_idx);
            wined3d_resource_unmap(src_resource, (*src_surface).sub_resource_idx);
        }

        if (*src_surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_MIPMAP != 0 {
            src_surface = get_sub_mimaplevel(src_surface);
        } else {
            src_surface = core::ptr::null_mut::<ddraw_surface>();
        }

        if (*dst_surface).surface_desc.ddsCaps.dwCaps & DDSCAPS_MIPMAP != 0 {
            dst_surface = get_sub_mimaplevel(dst_surface);
        } else {
            dst_surface = core::ptr::null_mut::<ddraw_surface>();
        }

        if src_surface.is_null() || dst_surface.is_null() {
            if src_surface != dst_surface {
                ERR!("Loading surface with different mipmap structure.\n");
            }
            break;
        }
    }

    wined3d_mutex_unlock();

    return hr;
}

unsafe extern "system" fn d3d_texture1_Load(
    iface: *mut IDirect3DTexture,
    src_texture: *mut IDirect3DTexture,
) -> HRESULT {
    let dst_surface: *mut ddraw_surface = impl_from_IDirect3DTexture(iface);
    let src_surface: *mut ddraw_surface = unsafe_impl_from_IDirect3DTexture(src_texture);

    return d3d_texture2_Load(
        core::ptr::addr_of_mut!((*dst_surface).IDirect3DTexture2_iface),
        if !src_surface.is_null() {
            core::ptr::addr_of_mut!((*src_surface).IDirect3DTexture2_iface)
        } else {
            core::ptr::null_mut()
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirectDrawSurface7(
    mut iface: *mut IDirectDrawSurface7,
) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != (&raw const ddraw_surface7_vtbl).cast_mut() as *const IDirectDrawSurface7Vtbl {
        let mut hr: HRESULT = IDirectDrawSurface7_QueryInterface(
            iface,
            core::ptr::addr_of!(IID_IDirectDrawSurface7),
            core::ptr::addr_of_mut!(iface) as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Object %p doesn't expose interface IDirectDrawSurface7.\n", iface);
            return core::ptr::null_mut();
        }
        IDirectDrawSurface7_Release(iface);
    }
    return (iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(ddraw_surface, IDirectDrawSurface7_iface) as ULONG_PTR,
    ) as *mut ddraw_surface;
}

#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirectDrawSurface4(
    mut iface: *mut IDirectDrawSurface4,
) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != (&raw const ddraw_surface4_vtbl).cast_mut() as *const IDirectDrawSurface4Vtbl {
        let mut hr: HRESULT = IDirectDrawSurface4_QueryInterface(
            iface,
            &raw const IID_IDirectDrawSurface4,
            &raw mut iface as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Object %p doesn't expose interface IDirectDrawSurface4.\n", iface);
            return core::ptr::null_mut();
        }
        IDirectDrawSurface4_Release(iface);
    }
    return (iface as usize).wrapping_sub(
        core::mem::offset_of!(ddraw_surface, IDirectDrawSurface4_iface),
    ) as *mut ddraw_surface;
}

unsafe extern "C" fn unsafe_impl_from_IDirectDrawSurface3(mut iface: *mut IDirectDrawSurface3) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != core::ptr::addr_of!(ddraw_surface3_vtbl) as *const IDirectDrawSurface3Vtbl {
        let mut hr: HRESULT = IDirectDrawSurface3_QueryInterface(
            iface,
            core::ptr::addr_of!(IID_IDirectDrawSurface3),
            core::ptr::addr_of_mut!(iface) as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Object %p doesn't expose interface IDirectDrawSurface3.\n", iface);
            return core::ptr::null_mut();
        }
        IDirectDrawSurface3_Release(iface);
    }
    return CONTAINING_RECORD!(iface, ddraw_surface, IDirectDrawSurface3_iface);
}

unsafe extern "C" fn unsafe_impl_from_IDirectDrawSurface2(mut iface: *mut IDirectDrawSurface2) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl as *const IDirectDrawSurface2Vtbl != core::ptr::addr_of!(ddraw_surface2_vtbl) as *const IDirectDrawSurface2Vtbl {
        let mut hr: HRESULT = IDirectDrawSurface2_QueryInterface(
            iface,
            core::ptr::addr_of!(IID_IDirectDrawSurface2),
            core::ptr::addr_of_mut!(iface) as *mut LPVOID,
        );
        if hr < 0 {
            WARN!("Object %p doesn't expose interface IDirectDrawSurface2.\n", iface);
            return core::ptr::null_mut();
        }
        IDirectDrawSurface2_Release(iface);
    }
    return ((iface as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(ddraw_surface, IDirectDrawSurface2_iface) as ULONG_PTR,
    )) as *mut ddraw_surface;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirectDrawSurface(
    mut iface: *mut IDirectDrawSurface,
) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    if (*iface).lpVtbl != core::ptr::addr_of!(ddraw_surface1_vtbl) as *const _ {
        let mut hr: HRESULT = IDirectDrawSurface_QueryInterface(
            iface,
            core::ptr::addr_of!(IID_IDirectDrawSurface),
            core::ptr::addr_of_mut!(iface) as *mut *mut core::ffi::c_void,
        );
        if (hr) < 0 {
            WARN!(
                "Object %p doesn't expose interface IDirectDrawSurface.\n",
                iface
            );
            return core::ptr::null_mut();
        }
        IDirectDrawSurface_Release(iface);
    }
    return (iface as usize).wrapping_sub(core::mem::offset_of!(
        ddraw_surface,
        IDirectDrawSurface_iface
    )) as *mut ddraw_surface;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DTexture2(iface: *mut IDirect3DTexture2) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!((*iface).lpVtbl == core::ptr::addr_of!(d3d_texture2_vtbl));
    return (iface as *mut u8).sub(core::mem::offset_of!(ddraw_surface, IDirect3DTexture2_iface)) as *mut ddraw_surface;
}
#[no_mangle]
pub unsafe extern "C" fn unsafe_impl_from_IDirect3DTexture(iface: *mut IDirect3DTexture) -> *mut ddraw_surface {
    if iface.is_null() {
        return core::ptr::null_mut();
    }
    assert!(((*iface).lpVtbl == core::ptr::addr_of!(d3d_texture1_vtbl)));
    return ((iface as usize).wrapping_sub(
        core::mem::offset_of!(ddraw_surface, IDirect3DTexture_iface) as usize,
    )) as *mut ddraw_surface;
}
unsafe extern "system" fn ddraw_surface_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let mut surface: *mut ddraw_surface = parent as *mut ddraw_surface;

    /* This shouldn't happen, ddraw_surface_release_iface() should prevent the
     * surface from being destroyed in this case. */
    if (*surface).first_attached != surface {
        ERR!("Surface is still attached to surface %p.\n", (*surface).first_attached);
    }

    while !(*surface).next_attached.is_null() {
        if (ddraw_surface_delete_attached_surface(
            surface,
            (*surface).next_attached,
            (*(*surface).next_attached).attached_iface,
        ) < 0)
        {
            ERR!("DeleteAttachedSurface failed.\n");
        }
    }

    /* Having a texture handle set implies that the device still exists. */
    if (*surface).Handle != 0 {
        ddraw_free_handle(
            core::ptr::null_mut(),
            (*surface).Handle - 1,
            DDRAW_HANDLE_SURFACE,
        );
    }

    /* Reduce the ddraw surface count. */
    list_remove(core::ptr::addr_of_mut!((*surface).surface_list_entry));

    if !(*surface).clipper.is_null() && ddraw_clipper_is_valid((*surface).clipper) != 0 {
        IDirectDrawClipper_Release(core::ptr::addr_of_mut!((*(*surface).clipper).IDirectDrawClipper_iface));
    }

    if surface == (*(*surface).ddraw).primary {
        (*(*surface).ddraw).primary = core::ptr::null_mut();
        (*(*surface).ddraw).gdi_surface = core::ptr::null_mut();
    }

    wined3d_private_store_cleanup(core::ptr::addr_of_mut!((*surface).private_store));

    if !(*surface).draw_texture.is_null() {
        wined3d_texture_decref((*surface).wined3d_texture);
    }

    free(surface as *mut core::ffi::c_void);
}

unsafe extern "C" fn ddraw_surface_init(
    mut surface: *mut ddraw_surface,
    mut ddraw: *mut ddraw,
    mut wined3d_texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
) {
    let mut texture: *mut ddraw_texture =
        wined3d_texture_get_parent(wined3d_texture) as *mut ddraw_texture;
    let mut version: ::core::ffi::c_uint = (*texture).version;
    (*surface).IDirectDrawSurface7_iface.lpVtbl = (&raw const ddraw_surface7_vtbl).cast_mut();
    (*surface).IDirectDrawSurface4_iface.lpVtbl = (&raw const ddraw_surface4_vtbl).cast_mut();
    (*surface).IDirectDrawSurface3_iface.lpVtbl = (&raw const ddraw_surface3_vtbl).cast_mut();
    (*surface).IDirectDrawSurface2_iface.lpVtbl = (&raw const ddraw_surface2_vtbl).cast_mut();
    (*surface).IDirectDrawSurface_iface.lpVtbl = (&raw const ddraw_surface1_vtbl).cast_mut();
    (*surface).IDirectDrawGammaControl_iface.lpVtbl =
        &raw const ddraw_gamma_control_vtbl as *const IDirectDrawGammaControlVtbl;
    (*surface).IDirect3DTexture2_iface.lpVtbl =
        &raw const d3d_texture2_vtbl as *const IDirect3DTexture2Vtbl;
    (*surface).IDirect3DTexture_iface.lpVtbl =
        &raw const d3d_texture1_vtbl as *const IDirect3DTextureVtbl;
    (*surface).iface_count = 1 as LONG;
    (*surface).version = version as ::core::ffi::c_int;
    (*surface).ddraw = ddraw;
    if version == 7 as ::core::ffi::c_uint {
        (*surface).ref7 = 1 as LONG;
        (*surface).texture_outer = &raw mut (*surface).IDirectDrawSurface7_iface as *mut IUnknown;
    } else if version == 4 as ::core::ffi::c_uint {
        (*surface).ref4 = 1 as LONG;
        (*surface).texture_outer = &raw mut (*surface).IDirectDrawSurface4_iface as *mut IUnknown;
    } else {
        (*surface).ref1 = 1 as LONG;
        (*surface).texture_outer = &raw mut (*surface).IDirectDrawSurface_iface as *mut IUnknown;
    }
    (*surface).first_attached = surface;
    (*surface).wined3d_texture = wined3d_texture;
    wined3d_texture_incref((*surface).wined3d_texture);
    (*surface).sub_resource_idx = sub_resource_idx;
    (*surface).texture_location = DDRAW_SURFACE_LOCATION_DEFAULT as ::core::ffi::c_uint;
    wined3d_private_store_init(&raw mut (*surface).private_store);
}
unsafe extern "system" fn ddraw_texture_wined3d_object_destroyed(mut parent: *mut core::ffi::c_void) {
    let mut texture: *mut ddraw_texture = parent as *mut ddraw_texture;
    TRACE!(
        "texture %p, texture_memory %p.\n",
        texture,
        (*texture).texture_memory
    );
    free((*texture).texture_memory);
    free(parent);
}

unsafe extern "C" fn ddraw_reset_enum_callback(resource: *mut wined3d_resource) -> HRESULT {
    DD_OK
}

unsafe extern "C" fn ddraw_surface_reserve_memory(
    wined3d_texture: *mut wined3d_texture,
    sub_resource_count: core::ffi::c_uint,
) -> HRESULT {
    let extra_size: core::ffi::c_uint = 0x10000;

    let texture: *mut ddraw_texture = wined3d_texture_get_parent(wined3d_texture) as *mut ddraw_texture;
    let mut resource_desc: wined3d_resource_desc = core::mem::zeroed();
    let mut desc: wined3d_sub_resource_desc = core::mem::zeroed();
    let mut pitch: core::ffi::c_uint = 0;
    let mut slice_pitch: core::ffi::c_uint = 0;
    let mut hr: HRESULT = WINED3D_OK;
    let mut offset: core::ffi::c_uint;
    let mut i: core::ffi::c_uint;

    wined3d_resource_get_desc(wined3d_texture_get_resource(wined3d_texture), core::ptr::addr_of_mut!(resource_desc));
    (*texture).texture_memory = calloc(
        (1 as usize) as _,
        (resource_desc.size.wrapping_add(extra_size) as usize) as _,
    );
    if (*texture).texture_memory.is_null()
    {
        ERR!("Out of memory.\n");
        return E_OUTOFMEMORY;
    }

    offset = 0;
    i = 0;
    while i < sub_resource_count
    {
        hr = wined3d_texture_get_sub_resource_desc(wined3d_texture, i, core::ptr::addr_of_mut!(desc));
        if hr < 0
        {
            ERR!("Subresource %u not found.\n", i);
            free((*texture).texture_memory);
            (*texture).texture_memory = core::ptr::null_mut();
            return hr;
        }
        wined3d_texture_get_pitch(
            wined3d_texture,
            i,
            core::ptr::addr_of_mut!(pitch),
            core::ptr::addr_of_mut!(slice_pitch),
        );

        hr = wined3d_texture_update_desc(
            wined3d_texture,
            i,
            ((*texture).texture_memory as *mut BYTE).offset(offset as isize) as *mut core::ffi::c_void,
            pitch,
        );
        if hr < 0
        {
            free((*texture).texture_memory);
            (*texture).texture_memory = core::ptr::null_mut();
            break;
        }
        offset = offset.wrapping_add(desc.size);
        i = i.wrapping_add(1);
    }
    return hr;
}

unsafe extern "C" fn wined3d_resource_desc_from_ddraw(
    mut ddraw: *mut ddraw,
    mut wined3d_desc: *mut wined3d_resource_desc,
    mut desc: *const DDSURFACEDESC2,
) {
    let caps: DWORD = (*desc).ddsCaps.dwCaps;
    let caps2: DWORD = (*desc).ddsCaps.dwCaps2;

    (*wined3d_desc).resource_type = WINED3D_RTYPE_TEXTURE_2D;
    (*wined3d_desc).format = wined3dformat_from_ddrawformat(&raw const (*desc).__bindgen_anon_5.ddpfPixelFormat);
    (*wined3d_desc).multisample_type = WINED3D_MULTISAMPLE_NONE;
    (*wined3d_desc).multisample_quality = 0;
    (*wined3d_desc).usage = WINED3DUSAGE_VIDMEM_ACCOUNTING;
    (*wined3d_desc).bind_flags = 0;
    (*wined3d_desc).access = WINED3D_RESOURCE_ACCESS_GPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    (*wined3d_desc).width = (*desc).dwWidth;
    (*wined3d_desc).height = (*desc).dwHeight;
    (*wined3d_desc).depth = 1;
    (*wined3d_desc).size = 0;

    /* Always specify WINED3D_BIND_SHADER_RESOURCE.
     * Surfaces without it can be bound on software devices on ddraw < 4. */
    (*wined3d_desc).bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
    if caps & DDSCAPS_ZBUFFER != 0 {
        (*wined3d_desc).bind_flags |= WINED3D_BIND_DEPTH_STENCIL;
    } else if caps & DDSCAPS_3DDEVICE != 0 {
        (*wined3d_desc).bind_flags |= WINED3D_BIND_RENDER_TARGET;
    }

    if (caps & DDSCAPS_SYSTEMMEMORY != 0)
        && !(caps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) != 0)
    {
        (*wined3d_desc).access = WINED3D_RESOURCE_ACCESS_CPU
            | WINED3D_RESOURCE_ACCESS_MAP_R
            | WINED3D_RESOURCE_ACCESS_MAP_W;
    } else {
        if caps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) != 0 {
            (*wined3d_desc).access |= WINED3D_RESOURCE_ACCESS_CPU;
            (*wined3d_desc).usage |= WINED3DUSAGE_MANAGED;
        } else if caps & DDSCAPS_VIDEOMEMORY != 0 {
            /* Dynamic resources can't be written by the GPU. */
            if !(caps & (DDSCAPS_3DDEVICE | DDSCAPS_ZBUFFER) != 0) {
                (*wined3d_desc).usage |= WINED3DUSAGE_DYNAMIC;
            }
        }
    }

    if caps & DDSCAPS_OVERLAY != 0 {
        (*wined3d_desc).usage |= WINED3DUSAGE_OVERLAY;
    }

    if caps & DDSCAPS_OWNDC != 0 {
        (*wined3d_desc).usage |= WINED3DUSAGE_OWNDC;
    }

    if caps2 & DDSCAPS2_CUBEMAP != 0 {
        (*wined3d_desc).usage |= WINED3DUSAGE_LEGACY_CUBEMAP;
    }
}

unsafe extern "C" fn ddraw_texture_init(
    texture: *mut ddraw_texture,
    ddraw: *mut ddraw,
    layers: ::core::ffi::c_uint,
    levels: ::core::ffi::c_uint,
    sysmem_fallback: bool,
    reserve_memory: bool,
) -> HRESULT {
    let mut wined3d_device: *mut wined3d_device = (*ddraw).wined3d_device;
    let desc: *const DDSURFACEDESC2 = core::ptr::addr_of!((*texture).surface_desc);
    let mut draw_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut wined3d_desc: wined3d_resource_desc = core::mem::zeroed();
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut parent: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut root: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
    let mut draw_bind_flags: ::core::ffi::c_uint = 0;
    let mut pitch: ::core::ffi::c_uint = 0;
    let mut i: ::core::ffi::c_uint = 0;
    let mut j: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = 0;

    wined3d_resource_desc_from_ddraw(ddraw, core::ptr::addr_of_mut!(wined3d_desc), desc);

    if wined3d_desc.format as ::core::ffi::c_uint == WINED3DFMT_UNKNOWN {
        WARN!("Unsupported / unknown pixelformat.\n");
        return DDERR_INVALIDPIXELFORMAT;
    }

    /* Validate the pitch. */
    if ((*desc).dwFlags & DDSD_LPSURFACE) != 0 {
        if format_is_compressed(core::ptr::addr_of!((*desc).__bindgen_anon_5.ddpfPixelFormat)) != 0 {
            if ((*desc).dwFlags & DDSD_LINEARSIZE) != 0
                && (*desc).__bindgen_anon_1.dwLinearSize
                    < wined3d_calculate_format_pitch(
                        (*ddraw).wined3d_adapter,
                        wined3d_desc.format,
                        wined3d_desc.width,
                    )
                    .wrapping_mul(
                        wined3d_desc
                            .height
                            .wrapping_add(3)
                            .wrapping_div(4),
                    )
            {
                WARN!(
                    "Invalid linear size %lu specified.\n",
                    (*desc).__bindgen_anon_1.dwLinearSize
                );
                return DDERR_INVALIDPARAMS;
            }
        } else {
            if ((*desc).__bindgen_anon_1.lPitch as ::core::ffi::c_uint)
                < wined3d_calculate_format_pitch(
                    (*ddraw).wined3d_adapter,
                    wined3d_desc.format,
                    wined3d_desc.width,
                )
                || ((*desc).__bindgen_anon_1.lPitch & 3) != 0
            {
                WARN!("Invalid pitch %lu specified.\n", (*desc).__bindgen_anon_1.lPitch);
                return DDERR_INVALIDPARAMS;
            }

            pitch = (*desc).__bindgen_anon_1.lPitch as ::core::ffi::c_uint;
        }
    }

    draw_bind_flags = wined3d_desc.bind_flags;

    if (wined3d_desc.access & WINED3D_RESOURCE_ACCESS_GPU) == 0 {
        wined3d_desc.bind_flags = 0;
    } else if (wined3d_desc.usage & WINED3DUSAGE_MANAGED) != 0 {
        wined3d_desc.bind_flags &= !WINED3D_BIND_RENDER_TARGET;
    }

    if draw_bind_flags != 0 && (wined3d_desc.bind_flags != draw_bind_flags) {
        let mut draw_texture_desc: wined3d_resource_desc = core::mem::zeroed();

        draw_texture_desc = wined3d_desc;
        draw_texture_desc.bind_flags = draw_bind_flags;
        draw_texture_desc.access = WINED3D_RESOURCE_ACCESS_GPU;
        draw_texture_desc.usage = 0;

        hr = wined3d_texture_create(
            wined3d_device,
            core::ptr::addr_of_mut!(draw_texture_desc),
            layers,
            levels,
            0,
            core::ptr::null::<wined3d_sub_resource_data>(),
            texture as *mut ::core::ffi::c_void,
            core::ptr::addr_of!(ddraw_texture_wined3d_parent_ops),
            core::ptr::addr_of_mut!(draw_texture),
        );
        if hr < 0 {
            WARN!("Failed to create draw texture, hr %#lx.\n", hr);
        }
    }

    /* Some applications assume surfaces will always be mapped at the same
     * address. Some of those also assume that this address is valid even when
     * the surface isn't mapped, and that updates done this way will be
     * visible on the screen. The game Nox is such an application,
     * Commandos: Behind Enemy Lines is another. Setting
     * WINED3D_TEXTURE_CREATE_GET_DC_LENIENT will ensure this. */

    'fail: {
        if !draw_texture.is_null() {
            hr = wined3d_texture_create(
                wined3d_device,
                core::ptr::addr_of_mut!(wined3d_desc),
                layers,
                levels,
                WINED3D_TEXTURE_CREATE_GET_DC_LENIENT,
                core::ptr::null::<wined3d_sub_resource_data>(),
                core::ptr::null_mut::<::core::ffi::c_void>(),
                core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
                core::ptr::addr_of_mut!(wined3d_texture),
            );
            if hr < 0 {
                break 'fail;
            }

            wined3d_resource_set_parent(
                wined3d_texture_get_resource(wined3d_texture),
                texture as *mut ::core::ffi::c_void,
                core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
            );
        } else {
            /* DDSCAPS_TEXTURE is not actually necessary to bind a surface as a
             * texture for a software device. Worse, one can bind a hardware
             * surface to a software device. However, some formats (e.g. YUY2) can
             * live on the GPU but truly aren't texturable, and we don't want to
             * fail to create them.
             *
             * Hence, if we fail to create a texture, and the user didn't
             * explicitly ask for a texture, strip WINED3D_BIND_SHADER_RESOURCE and
             * try again. */

            hr = wined3d_texture_create(
                wined3d_device,
                core::ptr::addr_of_mut!(wined3d_desc),
                layers,
                levels,
                WINED3D_TEXTURE_CREATE_GET_DC_LENIENT,
                core::ptr::null::<wined3d_sub_resource_data>(),
                texture as *mut ::core::ffi::c_void,
                core::ptr::addr_of!(ddraw_texture_wined3d_parent_ops),
                core::ptr::addr_of_mut!(wined3d_texture),
            );
            if hr < 0 {
                if ((*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE) != 0
                    || ((*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP) != 0
                {
                    return hr;
                }

                wined3d_desc.bind_flags &= !WINED3D_BIND_SHADER_RESOURCE;
                hr = wined3d_texture_create(
                    wined3d_device,
                    core::ptr::addr_of_mut!(wined3d_desc),
                    layers,
                    levels,
                    WINED3D_TEXTURE_CREATE_GET_DC_LENIENT,
                    core::ptr::null::<wined3d_sub_resource_data>(),
                    texture as *mut ::core::ffi::c_void,
                    core::ptr::addr_of!(ddraw_texture_wined3d_parent_ops),
                    core::ptr::addr_of_mut!(wined3d_texture),
                );
                if hr < 0 {
                    return hr;
                }
            }
        }

        if ((*desc).dwFlags & DDSD_LPSURFACE) != 0 && {
            hr = wined3d_texture_update_desc(
                wined3d_texture,
                0,
                (*desc).lpSurface as *mut ::core::ffi::c_void,
                pitch,
            );
            hr < 0
        } {
            ERR!("Failed to set surface memory, hr %#lx.\n", hr);
            break 'fail;
        }

        i = 0;
        while i < layers {
            let mut prev_level: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();

            j = 0;
            while j < levels {
                let mut wined3d_mip_desc: wined3d_sub_resource_desc = core::mem::zeroed();
                let mut sub_resource_idx: ::core::ffi::c_uint =
                    i.wrapping_mul(levels).wrapping_add(j);
                let mut row_pitch: ::core::ffi::c_uint = 0;
                let mut slice_pitch: ::core::ffi::c_uint = 0;
                let mut mip: *mut ddraw_surface = core::ptr::null_mut::<ddraw_surface>();
                let mut mip_desc: *mut DDSURFACEDESC2 = core::ptr::null_mut::<DDSURFACEDESC2>();

                mip = calloc(
                    (1 as size_t) as _,
                    (core::mem::size_of::<ddraw_surface>() as size_t) as _,
                ) as *mut ddraw_surface;
                if mip.is_null() {
                    hr = DDERR_OUTOFVIDEOMEMORY;
                    break 'fail;
                }

                ddraw_surface_init(mip, ddraw, wined3d_texture, sub_resource_idx);

                if !draw_texture.is_null() {
                    wined3d_texture_set_sub_resource_parent(
                        draw_texture,
                        sub_resource_idx,
                        mip as *mut ::core::ffi::c_void,
                        core::ptr::addr_of!(ddraw_surface_wined3d_parent_ops),
                    );
                    wined3d_texture_set_sub_resource_parent(
                        wined3d_texture,
                        sub_resource_idx,
                        mip as *mut ::core::ffi::c_void,
                        core::ptr::addr_of!(ddraw_null_wined3d_parent_ops),
                    );
                    (*mip).draw_texture = draw_texture;
                    wined3d_texture_incref((*mip).draw_texture);
                } else {
                    wined3d_texture_set_sub_resource_parent(
                        wined3d_texture,
                        sub_resource_idx,
                        mip as *mut ::core::ffi::c_void,
                        core::ptr::addr_of!(ddraw_surface_wined3d_parent_ops),
                    );
                }

                (*mip).set_sysmem_fallback((sysmem_fallback) as _);
                mip_desc = core::ptr::addr_of_mut!((*mip).surface_desc);

                *mip_desc = *desc;

                wined3d_texture_get_pitch(
                    wined3d_texture,
                    j,
                    core::ptr::addr_of_mut!(row_pitch),
                    core::ptr::addr_of_mut!(slice_pitch),
                );
                if format_is_compressed(core::ptr::addr_of!((*desc).__bindgen_anon_5.ddpfPixelFormat)) != 0
                {
                    if ((*desc).dwFlags & DDSD_LPSURFACE) != 0 {
                        (*mip_desc).__bindgen_anon_1.dwLinearSize = !0;
                    } else {
                        (*mip_desc).__bindgen_anon_1.dwLinearSize = slice_pitch as DWORD;
                    }
                    (*mip_desc).dwFlags |= DDSD_LINEARSIZE;
                    (*mip_desc).dwFlags &= !DDSD_PITCH;
                } else {
                    if ((*desc).dwFlags & DDSD_LPSURFACE) == 0 {
                        (*mip_desc).__bindgen_anon_1.lPitch = row_pitch as LONG;
                    }
                    (*mip_desc).dwFlags |= DDSD_PITCH;
                    (*mip_desc).dwFlags &= !DDSD_LINEARSIZE;
                }

                (*mip_desc).dwFlags &= !DDSD_LPSURFACE;
                (*mip_desc).lpSurface = core::ptr::null_mut();

                if ((*desc).ddsCaps.dwCaps & DDSCAPS_MIPMAP) != 0 {
                    (*mip_desc).__bindgen_anon_3.dwMipMapCount =
                        levels.wrapping_sub(j) as DWORD;
                }

                if j != 0 {
                    wined3d_texture_get_sub_resource_desc(
                        wined3d_texture,
                        sub_resource_idx,
                        core::ptr::addr_of_mut!(wined3d_mip_desc),
                    );
                    (*mip_desc).dwWidth = wined3d_mip_desc.width as DWORD;
                    (*mip_desc).dwHeight = wined3d_mip_desc.height as DWORD;

                    (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_MIPMAPSUBLEVEL;
                } else {
                    (*mip_desc).ddsCaps.dwCaps2 &= !DDSCAPS2_MIPMAPSUBLEVEL;
                }

                if ((*mip_desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP) != 0 {
                    (*mip_desc).ddsCaps.dwCaps2 &= !DDSCAPS2_CUBEMAP_ALLFACES;

                    match i {
                        WINED3D_CUBEMAP_FACE_POSITIVE_X => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_POSITIVEX;
                        }
                        WINED3D_CUBEMAP_FACE_NEGATIVE_X => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_NEGATIVEX;
                        }
                        WINED3D_CUBEMAP_FACE_POSITIVE_Y => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_POSITIVEY;
                        }
                        WINED3D_CUBEMAP_FACE_NEGATIVE_Y => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_NEGATIVEY;
                        }
                        WINED3D_CUBEMAP_FACE_POSITIVE_Z => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_POSITIVEZ;
                        }
                        WINED3D_CUBEMAP_FACE_NEGATIVE_Z => {
                            (*mip_desc).ddsCaps.dwCaps2 |= DDSCAPS2_CUBEMAP_NEGATIVEZ;
                        }
                        _ => {}
                    }
                }

                ddraw_update_lost_surfaces(ddraw);
                list_add_head(
                    core::ptr::addr_of_mut!((*ddraw).surface_list),
                    core::ptr::addr_of_mut!((*mip).surface_list_entry),
                );

                TRACE!("Created ddraw surface %p.\n", mip);

                if j == 0 {
                    if i == 0 {
                        (*texture).root = mip;
                    } else {
                        (*(*texture).root).complex_array[layers
                            .wrapping_sub(1)
                            .wrapping_sub(i)
                            as usize] = mip;
                    }
                } else if j == 1 && i == 0 {
                    (*(*texture).root).complex_array
                        [layers.wrapping_sub(1) as usize] = mip;
                } else {
                    (*prev_level).complex_array[0 as usize] = mip;
                }
                prev_level = mip;
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }

        root = (*texture).root;

        (*texture).wined3d_device = (*ddraw).wined3d_device;
        wined3d_device_incref((*texture).wined3d_device);

        ddraw_surface_sync_color_keys(root);

        wined3d_texture_decref(wined3d_texture);
        if !draw_texture.is_null() {
            wined3d_texture_decref(draw_texture);
        }

        if reserve_memory && {
            hr = ddraw_surface_reserve_memory(wined3d_texture, 1);
            hr < 0
        } {
            hr = hr_ddraw_from_wined3d(hr);
            break 'fail;
        }

        TRACE!(
            "Surface %p, created draw_texture %p, wined3d_texture %p.\n",
            root,
            draw_texture,
            wined3d_texture
        );
        return D3D_OK;
    }

    if !draw_texture.is_null() {
        wined3d_texture_decref(draw_texture);

        parent = wined3d_texture_get_sub_resource_parent(draw_texture, 0) as *mut ddraw_surface;
        if (*texture).version == 7 {
            IDirectDrawSurface7_Release(core::ptr::addr_of_mut!((*parent).IDirectDrawSurface7_iface));
        } else if (*texture).version == 4 {
            IDirectDrawSurface4_Release(core::ptr::addr_of_mut!((*parent).IDirectDrawSurface4_iface));
        } else {
            IDirectDrawSurface_Release(core::ptr::addr_of_mut!((*parent).IDirectDrawSurface_iface));
        }
    }
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_surface_create(
    ddraw: *mut ddraw,
    surface_desc: *const DDSURFACEDESC2,
    surface: *mut *mut ddraw_surface,
    outer_unknown: *mut IUnknown,
    version: ::core::ffi::c_uint,
) -> HRESULT {
    let mut wined3d_display_mode_format: DDPIXELFORMAT = core::mem::zeroed();
    let mut root: *mut ddraw_surface = core::ptr::null_mut();
    let mut attach: *mut *mut ddraw_surface = core::ptr::null_mut();
    let mut mode: wined3d_display_mode = core::mem::zeroed();
    let mut texture: *mut ddraw_texture = core::ptr::null_mut();
    let mut sysmem_fallback: BOOL = (FALSE as i32);
    let mut layers: ::core::ffi::c_uint = 1;
    let mut desc: *mut DDSURFACEDESC2 = core::ptr::null_mut();
    let mut reserve_memory: bool = false;
    let mut levels: UINT = 0;
    let mut i: UINT = 0;
    let mut hr: HRESULT = 0;

    if ((TRACE_ON!(__wine_dbch_ddraw)) != 0) {
        DDRAW_dump_surface_desc(surface_desc);
    }

    if !outer_unknown.is_null() {
        return CLASS_E_NOAGGREGATION;
    }

    if surface.is_null() {
        return E_POINTER;
    }

    texture = malloc(core::mem::size_of::<ddraw_texture>() as size_t) as *mut ddraw_texture;
    if texture.is_null() {
        return E_OUTOFMEMORY;
    }

    (*texture).texture_memory = core::ptr::null_mut();
    (*texture).version = version;
    (*texture).surface_desc = *surface_desc;
    desc = &raw mut (*texture).surface_desc;

    /* Ensure DDSD_CAPS is always set. */
    (*desc).dwFlags |= DDSD_CAPS;

    if (*desc).ddsCaps.dwCaps & DDSCAPS_COMPLEX != 0
        && (*desc).ddsCaps.dwCaps & (DDSCAPS_FLIP | DDSCAPS_MIPMAP) == 0
        && (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP == 0
    {
        WARN!("DDSCAPS_COMPLEX specified for a surface which is neither flippable, mipmapped, nor a cubemap.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    if (*desc).dwFlags & DDSD_MIPMAPCOUNT != 0 && (*desc).ddsCaps.dwCaps & DDSCAPS_COMPLEX == 0
    {
        /* This is illegal even if there is only one mipmap level. */
        WARN!("DDSD_MIPMAPCOUNT specified without DDSCAPS_COMPLEX.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_FLIP != 0
    {
        if (*desc).dwFlags & DDSD_BACKBUFFERCOUNT == 0 || (*desc).__bindgen_anon_2.dwBackBufferCount == 0
        {
            WARN!("Tried to create a flippable surface without any back buffers.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }

        if (*desc).ddsCaps.dwCaps & DDSCAPS_COMPLEX == 0
        {
            WARN!("Tried to create a flippable surface without DDSCAPS_COMPLEX.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }

        if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0
        {
            WARN!("Tried to create a flippable cubemap.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDPARAMS;
        }

        if (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE != 0
        {
            /* Flippable textures are not implemented. */
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }
    }
    else
    {
        if (*desc).dwFlags & DDSD_BACKBUFFERCOUNT != 0
        {
            WARN!("Tried to specify a back buffer count for a non-flippable surface.\n");
            hr = if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0 { DDERR_INVALIDPARAMS } else { DDERR_INVALIDCAPS };
            free(texture as *mut ::core::ffi::c_void);
            return hr;
        }
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0
    {
        if (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE != 0
        {
            WARN!("Tried to create a primary surface with DDSCAPS_TEXTURE.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }

        if (*desc).ddsCaps.dwCaps & DDSCAPS_COMPLEX != 0 && (*desc).ddsCaps.dwCaps & DDSCAPS_FLIP == 0
        {
            WARN!("Tried to create a flippable primary surface without both DDSCAPS_FLIP and DDSCAPS_COMPLEX.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }

        if (*desc).ddsCaps.dwCaps & DDSCAPS_FLIP != 0 && (*ddraw).cooperative_level & DDSCL_EXCLUSIVE == 0
        {
            WARN!("Tried to create a flippable primary surface without DDSCL_EXCLUSIVE.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_NOEXCLUSIVEMODE;
        }
    }

    /* This is a special case in ddrawex, but not allowed in ddraw. */
    if ((*desc).ddsCaps.dwCaps & (DDSCAPS_VIDEOMEMORY | DDSCAPS_SYSTEMMEMORY))
        == (DDSCAPS_VIDEOMEMORY | DDSCAPS_SYSTEMMEMORY)
    {
        WARN!("Tried to create a surface in both system and video memory.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    if (*desc).ddsCaps.dwCaps & (DDSCAPS_ALLOCONLOAD | DDSCAPS_MIPMAP) != 0
        && (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE == 0
    {
        WARN!("Caps %#lx require DDSCAPS_TEXTURE.\n", (*desc).ddsCaps.dwCaps);
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP_ALLFACES != 0
        && (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP == 0
    {
        WARN!("Cube map faces requested without cube map flag.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0
        && (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP_ALLFACES == 0
    {
        WARN!("Cube map without faces requested.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDPARAMS;
    }

    if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0
        && ((*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP_ALLFACES) != DDSCAPS2_CUBEMAP_ALLFACES
    {
        FIXME!("Partial cube maps not implemented.\n");
    }

    if (*desc).ddsCaps.dwCaps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) != 0
    {
        if (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE == 0
        {
            WARN!("DDSCAPS2_TEXTUREMANAGE used without DDSCAPS_TEXTURE, returning DDERR_INVALIDCAPS.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }
        if (*desc).ddsCaps.dwCaps & (DDSCAPS_VIDEOMEMORY | DDSCAPS_SYSTEMMEMORY) != 0
        {
            WARN!("DDSCAPS2_TEXTUREMANAGE used with DDSCAPS_VIDEOMEMORY or DDSCAPS_SYSTEMMEMORY, returning DDERR_INVALIDCAPS.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_WRITEONLY != 0
        && (*desc).ddsCaps.dwCaps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) == 0
    {
        WARN!("DDSCAPS_WRITEONLY used without DDSCAPS2_TEXTUREMANAGE, returning DDERR_INVALIDCAPS.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDCAPS;
    }

    hr = wined3d_output_get_display_mode((*ddraw).wined3d_output, &raw mut mode, core::ptr::null_mut());
    if hr < 0 {
        ERR!("Failed to get display mode, hr %#lx.\n", hr);
        free(texture as *mut ::core::ffi::c_void);
        return hr_ddraw_from_wined3d(hr);
    }

    wined3d_display_mode_format.dwSize = core::mem::size_of::<DDPIXELFORMAT>() as DWORD;
    ddrawformat_from_wined3dformat(&raw mut wined3d_display_mode_format, mode.format_id);

    /* No pixelformat given? Use the current screen format. */
    if (*desc).dwFlags & DDSD_PIXELFORMAT == 0
    {
        (*desc).dwFlags |= DDSD_PIXELFORMAT;
        (*desc).__bindgen_anon_5.ddpfPixelFormat = wined3d_display_mode_format;
    }

    /* No width or no height? Use the screen size. */
    if (*desc).dwFlags & DDSD_WIDTH == 0 || (*desc).dwFlags & DDSD_HEIGHT == 0
    {
        if (*desc).ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE == 0
        {
            WARN!("No width / height specified.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDPARAMS;
        }

        (*desc).dwFlags |= DDSD_WIDTH | DDSD_HEIGHT;
        (*desc).dwWidth = mode.width as DWORD;
        (*desc).dwHeight = mode.height as DWORD;
    }

    if (*desc).dwWidth == 0 || (*desc).dwHeight == 0
    {
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_INVALIDPARAMS;
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_FLIP != 0
    {
        (*desc).ddsCaps.dwCaps |= DDSCAPS_FRONTBUFFER;
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0
    {
        /* The first surface is a front buffer, the back buffers are created
         * afterwards. */
        (*desc).ddsCaps.dwCaps |= DDSCAPS_VISIBLE;
        if (*ddraw).cooperative_level & DDSCL_EXCLUSIVE != 0
        {
            let mut swapchain_desc: wined3d_swapchain_desc = core::mem::zeroed();

            wined3d_swapchain_get_desc((*ddraw).wined3d_swapchain, &raw mut swapchain_desc);
            swapchain_desc.backbuffer_width = mode.width;
            swapchain_desc.backbuffer_height = mode.height;
            swapchain_desc.backbuffer_format = mode.format_id;

            hr = wined3d_device_reset(
                (*ddraw).wined3d_device,
                &raw mut swapchain_desc,
                core::ptr::null::<wined3d_display_mode>(),
                Some(ddraw_reset_enum_callback as unsafe extern "C" fn(*mut wined3d_resource) -> HRESULT),
                (FALSE) as BOOL,
            );
            if hr < 0
            {
                ERR!("Failed to reset device.\n");
                free(texture as *mut ::core::ffi::c_void);
                return hr_ddraw_from_wined3d(hr);
            }
        }
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_3DDEVICE != 0 && (*ddraw).flags & DDRAW_NO3D != 0
    {
        WARN!("The application requests a 3D capable surface, but the ddraw object was created without 3D support.\n");
        /* Do not fail surface creation, only fail 3D device creation. */
    }

    /* Mipmap count fixes */
    if (*desc).ddsCaps.dwCaps & DDSCAPS_MIPMAP != 0
    {
        if (*desc).ddsCaps.dwCaps & DDSCAPS_COMPLEX != 0
        {
            if (*desc).dwFlags & DDSD_MIPMAPCOUNT != 0
            {
                /* Mipmap count is given, should not be 0. */
                if (*desc).__bindgen_anon_3.dwMipMapCount == 0
                {
                    free(texture as *mut ::core::ffi::c_void);
                    return DDERR_INVALIDPARAMS;
                }
            }
            else
            {
                /* Undocumented feature: Create sublevels until either the
                 * width or the height is 1. */
                if version == 7
                {
                    (*desc).__bindgen_anon_3.dwMipMapCount = wined3d_log2i(max!((*desc).dwWidth, (*desc).dwHeight)) + 1;
                }
                else
                {
                    (*desc).__bindgen_anon_3.dwMipMapCount = wined3d_log2i(min!((*desc).dwWidth, (*desc).dwHeight)) + 1;
                }
            }
        }
        else
        {
            (*desc).__bindgen_anon_3.dwMipMapCount = 1;
        }

        (*desc).dwFlags |= DDSD_MIPMAPCOUNT;
        levels = (*desc).__bindgen_anon_3.dwMipMapCount as UINT;
    }
    else
    {
        levels = 1;
    }

    if (*desc).ddsCaps.dwCaps & (DDSCAPS_VIDEOMEMORY | DDSCAPS_SYSTEMMEMORY) == 0
    {
        if (*desc).ddsCaps.dwCaps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) == 0
        {
            let mut bind_flags: ::core::ffi::c_uint = 0;
            let mut usage: DWORD = 0;

            if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0
            {
                usage |= WINED3DUSAGE_LEGACY_CUBEMAP;
                bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
            }
            else if (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE != 0
            {
                bind_flags |= WINED3D_BIND_SHADER_RESOURCE;
            }

            if (*desc).ddsCaps.dwCaps & DDSCAPS_ZBUFFER != 0
            {
                bind_flags |= WINED3D_BIND_DEPTH_STENCIL;
            }
            else if (*desc).ddsCaps.dwCaps & DDSCAPS_3DDEVICE != 0
            {
                bind_flags |= WINED3D_BIND_RENDER_TARGET;
            }

            let mut cdf_ok: bool = false;
            if (*ddraw).flags & DDRAW_NO3D == 0
            {
                hr = wined3d_check_device_format(
                    (*ddraw).wined3d,
                    (*ddraw).wined3d_adapter,
                    WINED3D_DEVICE_TYPE_HAL,
                    mode.format_id,
                    usage,
                    bind_flags,
                    WINED3D_RTYPE_TEXTURE_2D,
                    wined3dformat_from_ddrawformat(&raw mut (*desc).__bindgen_anon_5.ddpfPixelFormat),
                );
                cdf_ok = hr >= 0;
            }
            if cdf_ok
            {
                (*desc).ddsCaps.dwCaps |= DDSCAPS_VIDEOMEMORY;
            }
            else
            {
                (*desc).ddsCaps.dwCaps |= DDSCAPS_SYSTEMMEMORY;
                sysmem_fallback = (TRUE as i32);
            }
        }
        else if (*desc).ddsCaps.dwCaps & DDSCAPS_TEXTURE == 0
        {
            /* Tests show surfaces without memory flags get these flags added
             * right after creation. */
            (*desc).ddsCaps.dwCaps |= DDSCAPS_LOCALVIDMEM | DDSCAPS_VIDEOMEMORY;
        }
    }

    if ((*desc).ddsCaps.dwCaps & (DDSCAPS_OVERLAY | DDSCAPS_SYSTEMMEMORY))
        == (DDSCAPS_OVERLAY | DDSCAPS_SYSTEMMEMORY)
    {
        WARN!("System memory overlays are not allowed.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_NOOVERLAYHW;
    }

    if (*desc).ddsCaps.dwCaps2 & (DDSCAPS2_TEXTUREMANAGE | DDSCAPS2_D3DTEXTUREMANAGE) != 0
    {
        (*desc).ddsCaps.dwCaps |= DDSCAPS_SYSTEMMEMORY;
    }

    if (*desc).ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY != 0
    {
        (*desc).ddsCaps.dwCaps |= DDSCAPS_LOCALVIDMEM;
    }

    if (*desc).dwFlags & DDSD_LPSURFACE != 0
    {
        if (*desc).ddsCaps.dwCaps & DDSCAPS_SYSTEMMEMORY == 0
        {
            WARN!("User memory surfaces should not be GPU accessible.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDCAPS;
        }

        if version < 4
        {
            WARN!("User memory surfaces not supported before version 4.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDPARAMS;
        }

        if (*desc).lpSurface.is_null()
        {
            WARN!("NULL surface memory pointer specified.\n");
            free(texture as *mut ::core::ffi::c_void);
            return DDERR_INVALIDPARAMS;
        }

        if format_is_compressed(&raw mut (*desc).__bindgen_anon_5.ddpfPixelFormat) != 0
        {
            if version != 4 && (*desc).dwFlags & DDSD_PITCH != 0
            {
                WARN!("Pitch specified on a compressed user memory surface.\n");
                free(texture as *mut ::core::ffi::c_void);
                return DDERR_INVALIDPARAMS;
            }

            if (*desc).dwFlags & (DDSD_LINEARSIZE | DDSD_PITCH) == 0
            {
                WARN!("Compressed user memory surfaces should explicitly specify the linear size.\n");
                free(texture as *mut ::core::ffi::c_void);
                return DDERR_INVALIDPARAMS;
            }
        }
        else
        {
            if (*desc).dwFlags & DDSD_PITCH == 0
            {
                WARN!("User memory surfaces should explicitly specify the pitch.\n");
                free(texture as *mut ::core::ffi::c_void);
                return DDERR_INVALIDPARAMS;
            }
        }
    }

    if ((*desc).dwFlags & DDSD_CKDESTOVERLAY != 0
            && (*desc).__bindgen_anon_4.ddckCKDestOverlay.dwColorSpaceLowValue
                != (*desc).__bindgen_anon_4.ddckCKDestOverlay.dwColorSpaceHighValue)
            || ((*desc).dwFlags & DDSD_CKDESTBLT != 0
            && (*desc).ddckCKDestBlt.dwColorSpaceLowValue != (*desc).ddckCKDestBlt.dwColorSpaceHighValue)
            || ((*desc).dwFlags & DDSD_CKSRCOVERLAY != 0
            && (*desc).ddckCKSrcOverlay.dwColorSpaceLowValue != (*desc).ddckCKSrcOverlay.dwColorSpaceHighValue)
            || ((*desc).dwFlags & DDSD_CKSRCBLT != 0
            && (*desc).ddckCKSrcBlt.dwColorSpaceLowValue != (*desc).ddckCKSrcBlt.dwColorSpaceHighValue)
    {
        WARN!("Range color keys not supported, returning DDERR_NOCOLORKEYHW.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_NOCOLORKEYHW;
    }

    if (*ddraw).flags & DDRAW_NO3D != 0 && (*desc).ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY != 0
    {
        WARN!("Video memory surfaces not supported without 3D support.\n");
        free(texture as *mut ::core::ffi::c_void);
        return DDERR_NODIRECTDRAWHW;
    }

    if (*desc).ddsCaps.dwCaps2 & DDSCAPS2_CUBEMAP != 0
    {
        layers = 6;
    }

    reserve_memory = (*desc).dwFlags & DDSD_LPSURFACE == 0
            && (*desc).ddsCaps.dwCaps & DDSCAPS_VIDEOMEMORY != 0
            && wined3d_display_mode_format.__bindgen_anon_1.dwRGBBitCount <= 16;

    hr = ddraw_texture_init(texture, ddraw, layers, levels as ::core::ffi::c_uint, sysmem_fallback != 0, reserve_memory);
    if hr < 0
    {
        WARN!("Failed to create wined3d texture, hr %#lx.\n", hr);
        free(texture as *mut ::core::ffi::c_void);
        return hr_ddraw_from_wined3d(hr);
    }

    root = (*texture).root;
    (*root).set_is_root(TRUE as ::core::ffi::c_uint);

    'fail: {
        if (*desc).dwFlags & DDSD_BACKBUFFERCOUNT != 0
        {
            let count: ::core::ffi::c_uint = (*desc).__bindgen_anon_2.dwBackBufferCount as ::core::ffi::c_uint;
            let mut last: *mut ddraw_surface = root;

            attach = (&raw mut (*last).complex_array as *mut *mut ddraw_surface)
                .offset(0 as isize) as *mut *mut ddraw_surface;
            i = 0;
            while i < count
            {
                texture = malloc(core::mem::size_of::<ddraw_texture>() as size_t) as *mut ddraw_texture;
                if texture.is_null()
                {
                    hr = E_OUTOFMEMORY;
                    break 'fail;
                }

                (*texture).texture_memory = core::ptr::null_mut();
                (*texture).version = version;
                (*texture).surface_desc = (*root).surface_desc;
                desc = &raw mut (*texture).surface_desc;

                /* Only one surface in the flipping chain is a back buffer, one is
                 * a front buffer, the others are just flippable surfaces. */
                (*desc).ddsCaps.dwCaps &= !(DDSCAPS_VISIBLE | DDSCAPS_PRIMARYSURFACE | DDSCAPS_FRONTBUFFER
                        | DDSCAPS_BACKBUFFER);
                if i == 0
                {
                    (*desc).ddsCaps.dwCaps |= DDSCAPS_BACKBUFFER;
                }
                (*desc).__bindgen_anon_2.dwBackBufferCount = 0;

                hr = ddraw_texture_init(texture, ddraw, 1, 1, sysmem_fallback != 0, reserve_memory);
                if hr < 0
                {
                    free(texture as *mut ::core::ffi::c_void);
                    hr = hr_ddraw_from_wined3d(hr);
                    break 'fail;
                }

                last = (*texture).root;

                *attach = last;
                attach = (&raw mut (*last).complex_array as *mut *mut ddraw_surface)
                    .offset(0 as isize) as *mut *mut ddraw_surface;
                i = i.wrapping_add(1);
            }
            *attach = root;
        }

        if (*surface_desc).ddsCaps.dwCaps & DDSCAPS_PRIMARYSURFACE != 0
        {
            (*ddraw).primary = root;
            (*ddraw).gdi_surface = (*root).wined3d_texture;
        }
        *surface = root;

        return DD_OK;
    }

    /* fail: */
    if version == 7
    {
        IDirectDrawSurface7_Release(&raw mut (*root).IDirectDrawSurface7_iface);
    }
    else if version == 4
    {
        IDirectDrawSurface4_Release(&raw mut (*root).IDirectDrawSurface4_iface);
    }
    else
    {
        IDirectDrawSurface_Release(&raw mut (*root).IDirectDrawSurface_iface);
    }

    return hr;
}
unsafe extern "system" fn view_wined3d_object_destroyed(parent: *mut core::ffi::c_void) {
    let mut surface: *mut ddraw_surface = parent as *mut ddraw_surface;
    (*surface).wined3d_rtv = core::ptr::null_mut::<wined3d_rendertarget_view>();
}
#[no_mangle]
pub unsafe extern "C" fn ddraw_surface_get_rendertarget_view(
    surface: *mut ddraw_surface,
) -> *mut wined3d_rendertarget_view {
    let mut wined3d_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut hr: HRESULT;

    if !(*surface).wined3d_rtv.is_null() {
        return (*surface).wined3d_rtv;
    }

    wined3d_texture = if !(*surface).draw_texture.is_null() {
        (*surface).draw_texture
    } else {
        (*surface).wined3d_texture
    };
    hr = wined3d_rendertarget_view_create_from_sub_resource(
        wined3d_texture,
        (*surface).sub_resource_idx,
        surface as *mut core::ffi::c_void,
        core::ptr::addr_of!(ddraw_view_wined3d_parent_ops),
        core::ptr::addr_of_mut!((*surface).wined3d_rtv),
    );
    if ((hr) < 0) < false {
        ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
        return core::ptr::null_mut();
    }

    (*surface).wined3d_rtv
}

// COM vtable tables — generated from surface.c by vtbl_gen.py
/// surface.c:5664 `static const IDirectDrawSurface7Vtbl ddraw_surface7_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_surface7_vtbl: IDirectDrawSurface7Vtbl = IDirectDrawSurface7Vtbl {
    QueryInterface: Some(ddraw_surface7_QueryInterface),
    AddRef: Some(ddraw_surface7_AddRef),
    Release: Some(ddraw_surface7_Release),
    AddAttachedSurface: Some(ddraw_surface7_AddAttachedSurface),
    AddOverlayDirtyRect: Some(ddraw_surface7_AddOverlayDirtyRect),
    Blt: Some(ddraw_surface7_Blt),
    BltBatch: Some(ddraw_surface7_BltBatch),
    BltFast: Some(ddraw_surface7_BltFast),
    DeleteAttachedSurface: Some(ddraw_surface7_DeleteAttachedSurface),
    EnumAttachedSurfaces: Some(ddraw_surface7_EnumAttachedSurfaces),
    EnumOverlayZOrders: Some(ddraw_surface7_EnumOverlayZOrders),
    Flip: Some(ddraw_surface7_Flip),
    GetAttachedSurface: Some(ddraw_surface7_GetAttachedSurface),
    GetBltStatus: Some(ddraw_surface7_GetBltStatus),
    GetCaps: Some(ddraw_surface7_GetCaps),
    GetClipper: Some(ddraw_surface7_GetClipper),
    GetColorKey: Some(ddraw_surface7_GetColorKey),
    GetDC: Some(ddraw_surface7_GetDC),
    GetFlipStatus: Some(ddraw_surface7_GetFlipStatus),
    GetOverlayPosition: Some(ddraw_surface7_GetOverlayPosition),
    GetPalette: Some(ddraw_surface7_GetPalette),
    GetPixelFormat: Some(ddraw_surface7_GetPixelFormat),
    GetSurfaceDesc: Some(ddraw_surface7_GetSurfaceDesc),
    Initialize: Some(ddraw_surface7_Initialize),
    IsLost: Some(ddraw_surface7_IsLost),
    Lock: Some(ddraw_surface7_Lock),
    ReleaseDC: Some(ddraw_surface7_ReleaseDC),
    Restore: Some(ddraw_surface7_Restore),
    SetClipper: Some(ddraw_surface7_SetClipper),
    SetColorKey: Some(ddraw_surface7_SetColorKey),
    SetOverlayPosition: Some(ddraw_surface7_SetOverlayPosition),
    SetPalette: Some(ddraw_surface7_SetPalette),
    Unlock: Some(ddraw_surface7_Unlock),
    UpdateOverlay: Some(ddraw_surface7_UpdateOverlay),
    UpdateOverlayDisplay: Some(ddraw_surface7_UpdateOverlayDisplay),
    UpdateOverlayZOrder: Some(ddraw_surface7_UpdateOverlayZOrder),
    GetDDInterface: Some(ddraw_surface7_GetDDInterface),
    PageLock: Some(ddraw_surface7_PageLock),
    PageUnlock: Some(ddraw_surface7_PageUnlock),
    SetSurfaceDesc: Some(ddraw_surface7_SetSurfaceDesc),
    SetPrivateData: Some(ddraw_surface7_SetPrivateData),
    GetPrivateData: Some(ddraw_surface7_GetPrivateData),
    FreePrivateData: Some(ddraw_surface7_FreePrivateData),
    GetUniquenessValue: Some(ddraw_surface7_GetUniquenessValue),
    ChangeUniquenessValue: Some(ddraw_surface7_ChangeUniquenessValue),
    SetPriority: Some(ddraw_surface7_SetPriority),
    GetPriority: Some(ddraw_surface7_GetPriority),
    SetLOD: Some(ddraw_surface7_SetLOD),
    GetLOD: Some(ddraw_surface7_GetLOD),
};

/// surface.c:5724 `static const IDirectDrawSurface4Vtbl ddraw_surface4_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_surface4_vtbl: IDirectDrawSurface4Vtbl = IDirectDrawSurface4Vtbl {
    QueryInterface: Some(ddraw_surface4_QueryInterface),
    AddRef: Some(ddraw_surface4_AddRef),
    Release: Some(ddraw_surface4_Release),
    AddAttachedSurface: Some(ddraw_surface4_AddAttachedSurface),
    AddOverlayDirtyRect: Some(ddraw_surface4_AddOverlayDirtyRect),
    Blt: Some(ddraw_surface4_Blt),
    BltBatch: Some(ddraw_surface4_BltBatch),
    BltFast: Some(ddraw_surface4_BltFast),
    DeleteAttachedSurface: Some(ddraw_surface4_DeleteAttachedSurface),
    EnumAttachedSurfaces: Some(ddraw_surface4_EnumAttachedSurfaces),
    EnumOverlayZOrders: Some(ddraw_surface4_EnumOverlayZOrders),
    Flip: Some(ddraw_surface4_Flip),
    GetAttachedSurface: Some(ddraw_surface4_GetAttachedSurface),
    GetBltStatus: Some(ddraw_surface4_GetBltStatus),
    GetCaps: Some(ddraw_surface4_GetCaps),
    GetClipper: Some(ddraw_surface4_GetClipper),
    GetColorKey: Some(ddraw_surface4_GetColorKey),
    GetDC: Some(ddraw_surface4_GetDC),
    GetFlipStatus: Some(ddraw_surface4_GetFlipStatus),
    GetOverlayPosition: Some(ddraw_surface4_GetOverlayPosition),
    GetPalette: Some(ddraw_surface4_GetPalette),
    GetPixelFormat: Some(ddraw_surface4_GetPixelFormat),
    GetSurfaceDesc: Some(ddraw_surface4_GetSurfaceDesc),
    Initialize: Some(ddraw_surface4_Initialize),
    IsLost: Some(ddraw_surface4_IsLost),
    Lock: Some(ddraw_surface4_Lock),
    ReleaseDC: Some(ddraw_surface4_ReleaseDC),
    Restore: Some(ddraw_surface4_Restore),
    SetClipper: Some(ddraw_surface4_SetClipper),
    SetColorKey: Some(ddraw_surface4_SetColorKey),
    SetOverlayPosition: Some(ddraw_surface4_SetOverlayPosition),
    SetPalette: Some(ddraw_surface4_SetPalette),
    Unlock: Some(ddraw_surface4_Unlock),
    UpdateOverlay: Some(ddraw_surface4_UpdateOverlay),
    UpdateOverlayDisplay: Some(ddraw_surface4_UpdateOverlayDisplay),
    UpdateOverlayZOrder: Some(ddraw_surface4_UpdateOverlayZOrder),
    GetDDInterface: Some(ddraw_surface4_GetDDInterface),
    PageLock: Some(ddraw_surface4_PageLock),
    PageUnlock: Some(ddraw_surface4_PageUnlock),
    SetSurfaceDesc: Some(ddraw_surface4_SetSurfaceDesc),
    SetPrivateData: Some(ddraw_surface4_SetPrivateData),
    GetPrivateData: Some(ddraw_surface4_GetPrivateData),
    FreePrivateData: Some(ddraw_surface4_FreePrivateData),
    GetUniquenessValue: Some(ddraw_surface4_GetUniquenessValue),
    ChangeUniquenessValue: Some(ddraw_surface4_ChangeUniquenessValue),
};

/// surface.c:5779 `static const IDirectDrawSurface3Vtbl ddraw_surface3_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_surface3_vtbl: IDirectDrawSurface3Vtbl = IDirectDrawSurface3Vtbl {
    QueryInterface: Some(ddraw_surface3_QueryInterface),
    AddRef: Some(ddraw_surface3_AddRef),
    Release: Some(ddraw_surface3_Release),
    AddAttachedSurface: Some(ddraw_surface3_AddAttachedSurface),
    AddOverlayDirtyRect: Some(ddraw_surface3_AddOverlayDirtyRect),
    Blt: Some(ddraw_surface3_Blt),
    BltBatch: Some(ddraw_surface3_BltBatch),
    BltFast: Some(ddraw_surface3_BltFast),
    DeleteAttachedSurface: Some(ddraw_surface3_DeleteAttachedSurface),
    EnumAttachedSurfaces: Some(ddraw_surface3_EnumAttachedSurfaces),
    EnumOverlayZOrders: Some(ddraw_surface3_EnumOverlayZOrders),
    Flip: Some(ddraw_surface3_Flip),
    GetAttachedSurface: Some(ddraw_surface3_GetAttachedSurface),
    GetBltStatus: Some(ddraw_surface3_GetBltStatus),
    GetCaps: Some(ddraw_surface3_GetCaps),
    GetClipper: Some(ddraw_surface3_GetClipper),
    GetColorKey: Some(ddraw_surface3_GetColorKey),
    GetDC: Some(ddraw_surface3_GetDC),
    GetFlipStatus: Some(ddraw_surface3_GetFlipStatus),
    GetOverlayPosition: Some(ddraw_surface3_GetOverlayPosition),
    GetPalette: Some(ddraw_surface3_GetPalette),
    GetPixelFormat: Some(ddraw_surface3_GetPixelFormat),
    GetSurfaceDesc: Some(ddraw_surface3_GetSurfaceDesc),
    Initialize: Some(ddraw_surface3_Initialize),
    IsLost: Some(ddraw_surface3_IsLost),
    Lock: Some(ddraw_surface3_Lock),
    ReleaseDC: Some(ddraw_surface3_ReleaseDC),
    Restore: Some(ddraw_surface3_Restore),
    SetClipper: Some(ddraw_surface3_SetClipper),
    SetColorKey: Some(ddraw_surface3_SetColorKey),
    SetOverlayPosition: Some(ddraw_surface3_SetOverlayPosition),
    SetPalette: Some(ddraw_surface3_SetPalette),
    Unlock: Some(ddraw_surface3_Unlock),
    UpdateOverlay: Some(ddraw_surface3_UpdateOverlay),
    UpdateOverlayDisplay: Some(ddraw_surface3_UpdateOverlayDisplay),
    UpdateOverlayZOrder: Some(ddraw_surface3_UpdateOverlayZOrder),
    GetDDInterface: Some(ddraw_surface3_GetDDInterface),
    PageLock: Some(ddraw_surface3_PageLock),
    PageUnlock: Some(ddraw_surface3_PageUnlock),
    SetSurfaceDesc: Some(ddraw_surface3_SetSurfaceDesc),
};

/// surface.c:5828 `static const IDirectDrawSurface2Vtbl ddraw_surface2_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_surface2_vtbl: IDirectDrawSurface2Vtbl = IDirectDrawSurface2Vtbl {
    QueryInterface: Some(ddraw_surface2_QueryInterface),
    AddRef: Some(ddraw_surface2_AddRef),
    Release: Some(ddraw_surface2_Release),
    AddAttachedSurface: Some(ddraw_surface2_AddAttachedSurface),
    AddOverlayDirtyRect: Some(ddraw_surface2_AddOverlayDirtyRect),
    Blt: Some(ddraw_surface2_Blt),
    BltBatch: Some(ddraw_surface2_BltBatch),
    BltFast: Some(ddraw_surface2_BltFast),
    DeleteAttachedSurface: Some(ddraw_surface2_DeleteAttachedSurface),
    EnumAttachedSurfaces: Some(ddraw_surface2_EnumAttachedSurfaces),
    EnumOverlayZOrders: Some(ddraw_surface2_EnumOverlayZOrders),
    Flip: Some(ddraw_surface2_Flip),
    GetAttachedSurface: Some(ddraw_surface2_GetAttachedSurface),
    GetBltStatus: Some(ddraw_surface2_GetBltStatus),
    GetCaps: Some(ddraw_surface2_GetCaps),
    GetClipper: Some(ddraw_surface2_GetClipper),
    GetColorKey: Some(ddraw_surface2_GetColorKey),
    GetDC: Some(ddraw_surface2_GetDC),
    GetFlipStatus: Some(ddraw_surface2_GetFlipStatus),
    GetOverlayPosition: Some(ddraw_surface2_GetOverlayPosition),
    GetPalette: Some(ddraw_surface2_GetPalette),
    GetPixelFormat: Some(ddraw_surface2_GetPixelFormat),
    GetSurfaceDesc: Some(ddraw_surface2_GetSurfaceDesc),
    Initialize: Some(ddraw_surface2_Initialize),
    IsLost: Some(ddraw_surface2_IsLost),
    Lock: Some(ddraw_surface2_Lock),
    ReleaseDC: Some(ddraw_surface2_ReleaseDC),
    Restore: Some(ddraw_surface2_Restore),
    SetClipper: Some(ddraw_surface2_SetClipper),
    SetColorKey: Some(ddraw_surface2_SetColorKey),
    SetOverlayPosition: Some(ddraw_surface2_SetOverlayPosition),
    SetPalette: Some(ddraw_surface2_SetPalette),
    Unlock: Some(ddraw_surface2_Unlock),
    UpdateOverlay: Some(ddraw_surface2_UpdateOverlay),
    UpdateOverlayDisplay: Some(ddraw_surface2_UpdateOverlayDisplay),
    UpdateOverlayZOrder: Some(ddraw_surface2_UpdateOverlayZOrder),
    GetDDInterface: Some(ddraw_surface2_GetDDInterface),
    PageLock: Some(ddraw_surface2_PageLock),
    PageUnlock: Some(ddraw_surface2_PageUnlock),
};

/// surface.c:5875 `static const IDirectDrawSurfaceVtbl ddraw_surface1_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_surface1_vtbl: IDirectDrawSurfaceVtbl = IDirectDrawSurfaceVtbl {
    QueryInterface: Some(ddraw_surface1_QueryInterface),
    AddRef: Some(ddraw_surface1_AddRef),
    Release: Some(ddraw_surface1_Release),
    AddAttachedSurface: Some(ddraw_surface1_AddAttachedSurface),
    AddOverlayDirtyRect: Some(ddraw_surface1_AddOverlayDirtyRect),
    Blt: Some(ddraw_surface1_Blt),
    BltBatch: Some(ddraw_surface1_BltBatch),
    BltFast: Some(ddraw_surface1_BltFast),
    DeleteAttachedSurface: Some(ddraw_surface1_DeleteAttachedSurface),
    EnumAttachedSurfaces: Some(ddraw_surface1_EnumAttachedSurfaces),
    EnumOverlayZOrders: Some(ddraw_surface1_EnumOverlayZOrders),
    Flip: Some(ddraw_surface1_Flip),
    GetAttachedSurface: Some(ddraw_surface1_GetAttachedSurface),
    GetBltStatus: Some(ddraw_surface1_GetBltStatus),
    GetCaps: Some(ddraw_surface1_GetCaps),
    GetClipper: Some(ddraw_surface1_GetClipper),
    GetColorKey: Some(ddraw_surface1_GetColorKey),
    GetDC: Some(ddraw_surface1_GetDC),
    GetFlipStatus: Some(ddraw_surface1_GetFlipStatus),
    GetOverlayPosition: Some(ddraw_surface1_GetOverlayPosition),
    GetPalette: Some(ddraw_surface1_GetPalette),
    GetPixelFormat: Some(ddraw_surface1_GetPixelFormat),
    GetSurfaceDesc: Some(ddraw_surface1_GetSurfaceDesc),
    Initialize: Some(ddraw_surface1_Initialize),
    IsLost: Some(ddraw_surface1_IsLost),
    Lock: Some(ddraw_surface1_Lock),
    ReleaseDC: Some(ddraw_surface1_ReleaseDC),
    Restore: Some(ddraw_surface1_Restore),
    SetClipper: Some(ddraw_surface1_SetClipper),
    SetColorKey: Some(ddraw_surface1_SetColorKey),
    SetOverlayPosition: Some(ddraw_surface1_SetOverlayPosition),
    SetPalette: Some(ddraw_surface1_SetPalette),
    Unlock: Some(ddraw_surface1_Unlock),
    UpdateOverlay: Some(ddraw_surface1_UpdateOverlay),
    UpdateOverlayDisplay: Some(ddraw_surface1_UpdateOverlayDisplay),
    UpdateOverlayZOrder: Some(ddraw_surface1_UpdateOverlayZOrder),
};

/// surface.c:5917 `static const IDirectDrawGammaControlVtbl ddraw_gamma_control_vtbl`  (slot order: rust:types_amd64.rs)
pub static ddraw_gamma_control_vtbl: IDirectDrawGammaControlVtbl = IDirectDrawGammaControlVtbl {
    QueryInterface: Some(ddraw_gamma_control_QueryInterface),
    AddRef: Some(ddraw_gamma_control_AddRef),
    Release: Some(ddraw_gamma_control_Release),
    GetGammaRamp: Some(ddraw_gamma_control_GetGammaRamp),
    SetGammaRamp: Some(ddraw_gamma_control_SetGammaRamp),
};

/// surface.c:5926 `static const IDirect3DTexture2Vtbl d3d_texture2_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_texture2_vtbl: IDirect3DTexture2Vtbl = IDirect3DTexture2Vtbl {
    QueryInterface: Some(d3d_texture2_QueryInterface),
    AddRef: Some(d3d_texture2_AddRef),
    Release: Some(d3d_texture2_Release),
    GetHandle: Some(d3d_texture2_GetHandle),
    PaletteChanged: Some(d3d_texture2_PaletteChanged),
    Load: Some(d3d_texture2_Load),
};

/// surface.c:5936 `static const IDirect3DTextureVtbl d3d_texture1_vtbl`  (slot order: rust:types_amd64.rs)
pub static d3d_texture1_vtbl: IDirect3DTextureVtbl = IDirect3DTextureVtbl {
    QueryInterface: Some(d3d_texture1_QueryInterface),
    AddRef: Some(d3d_texture1_AddRef),
    Release: Some(d3d_texture1_Release),
    Initialize: Some(d3d_texture1_Initialize),
    GetHandle: Some(d3d_texture1_GetHandle),
    PaletteChanged: Some(d3d_texture1_PaletteChanged),
    Load: Some(d3d_texture1_Load),
    Unload: Some(d3d_texture1_Unload),
};
