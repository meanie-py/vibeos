use crate::d3dwine::device::device_context_add;
use crate::d3dwine::device::device_invalidate_state;
use crate::d3dwine::resource::vk_access_mask_from_bind_flags;
use crate::d3dwine::context_gl::wined3d_context_gl_destroy;
use crate::d3dwine::context_gl::wined3d_context_gl_init;
use crate::d3dwine::context_gl::wined3d_context_gl_submit_command_fence;
use crate::d3dwine::context_vk::wined3d_context_vk_end_current_render_pass;
use crate::d3dwine::context_vk::wined3d_context_vk_get_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_image_barrier;
use crate::d3dwine::context_vk::wined3d_context_vk_submit_command_buffer;
use crate::d3dwine::context_vk::wined3d_context_vk_wait_command_buffer;
use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::d3dwine::cs::wined3d_cs_emit_present;
use crate::d3dwine::device::wined3d_device_release_focus_window;
use crate::d3dwine::device::wined3d_device_set_implicit_swapchain;
use crate::d3dwine::device::wined3d_device_uninit_3d;
use crate::d3dwine::palette::wined3d_palette_apply_to_dc;
use crate::d3dwine::resource::wined3d_resource_update_draw_binding;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_D0036AD7DA5A: [u16; 25] = [
    0x0077u16, 0x0069, 0x006E, 0x0065, 0x0064, 0x0033, 0x0064, 0x005F, 0x0073, 0x0065,
    0x0074, 0x005F, 0x0077, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x005F, 0x0073,
    0x0074, 0x0061, 0x0074, 0x0065, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut swapchain_gl_ops: wined3d_swapchain_ops = unsafe {
    wined3d_swapchain_ops {
        swapchain_present: Some(
            swapchain_gl_present
                as unsafe extern "C" fn(
                    *mut wined3d_swapchain,
                    *const RECT,
                    *const RECT,
                    ::core::ffi::c_uint,
                    uint32_t,
                ) -> (),
        ),
        swapchain_frontbuffer_updated: Some(
            swapchain_frontbuffer_updated as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
    }
};

static mut swapchain_vk_ops: wined3d_swapchain_ops = unsafe {
    wined3d_swapchain_ops {
        swapchain_present: Some(
            swapchain_vk_present
                as unsafe extern "C" fn(
                    *mut wined3d_swapchain,
                    *const RECT,
                    *const RECT,
                    ::core::ffi::c_uint,
                    uint32_t,
                ) -> (),
        ),
        swapchain_frontbuffer_updated: Some(
            swapchain_frontbuffer_updated as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
    }
};

static mut swapchain_no3d_ops: wined3d_swapchain_ops = unsafe {
    wined3d_swapchain_ops {
        swapchain_present: Some(
            swapchain_gdi_present
                as unsafe extern "C" fn(
                    *mut wined3d_swapchain,
                    *const RECT,
                    *const RECT,
                    ::core::ffi::c_uint,
                    uint32_t,
                ) -> (),
        ),
        swapchain_frontbuffer_updated: Some(
            swapchain_gdi_frontbuffer_updated as unsafe extern "C" fn(*mut wined3d_swapchain) -> (),
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d) / WINE_DECLARE_DEBUG_CHANNEL(d3d_perf):
// debug-channel declarations, handled by the crate's logging macros.

#[repr(C)]
pub struct wined3d_window_state {
    pub window: HWND,
    pub window_pos_after: HWND,
    pub style: i32,
    pub exstyle: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub flags: u32,
    pub set_style: bool,
    pub register_topmost_timer: bool,
    pub set_topmost_timer: bool,
}

pub const WINED3D_WINDOW_TOPMOST_TIMER_ID: i32 = 0x4242;
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_cleanup(swapchain: *mut wined3d_swapchain) {
    let mut hr: HRESULT;
    let mut i: UINT;

    wined3d_swapchain_state_cleanup(core::ptr::addr_of_mut!((*swapchain).state));
    wined3d_swapchain_set_gamma_ramp(swapchain, 0, core::ptr::addr_of_mut!((*swapchain).orig_gamma));

    /* Release the swapchain's draw buffers. Make sure swapchain->back_buffers[0]
     * is the last buffer to be destroyed, FindContext() depends on that. */
    if !(*swapchain).front_buffer.is_null() {
        wined3d_texture_set_swapchain((*swapchain).front_buffer, core::ptr::null_mut());
        if wined3d_texture_decref((*swapchain).front_buffer) != 0 {
            WARN!("Something's still holding the front buffer (%p).\n", (*swapchain).front_buffer);
        }
        (*swapchain).front_buffer = core::ptr::null_mut();
    }

    if !(*swapchain).back_buffers.is_null() {
        i = (*swapchain).state.desc.backbuffer_count as UINT;

        loop {
            let fresh = i;
            i = i.wrapping_sub(1);
            if fresh == 0 {
                break;
            }
            wined3d_texture_set_swapchain(*(*swapchain).back_buffers.offset(i as isize), core::ptr::null_mut());
            if wined3d_texture_decref(*(*swapchain).back_buffers.offset(i as isize)) != 0 {
                WARN!("Something's still holding back buffer %u (%p).\n", i, *(*swapchain).back_buffers.offset(i as isize));
            }
        }
        free((*swapchain).back_buffers as *mut core::ffi::c_void);
        (*swapchain).back_buffers = core::ptr::null_mut();
    }

    /* Restore the screen resolution if we rendered in fullscreen.
     * This will restore the screen resolution to what it was before creating
     * the swapchain. In case of d3d8 and d3d9 this will be the original
     * desktop resolution. In case of d3d7 this will be a NOP because ddraw
     * sets the resolution before starting up Direct3D, thus orig_width and
     * orig_height will be equal to the modes in the presentation params. */
    if (*swapchain).state.desc.windowed == 0 {
        if (*swapchain).state.desc.auto_restore_display_mode != 0 {
            hr = wined3d_restore_display_modes((*(*swapchain).device).wined3d);
            if hr < 0 {
                ERR!("Failed to restore display mode, hr %#lx.\n", hr);
            }

            if (*swapchain).state.desc.flags & WINED3D_SWAPCHAIN_RESTORE_WINDOW_RECT != 0 {
                wined3d_swapchain_state_restore_from_fullscreen(
                    core::ptr::addr_of_mut!((*swapchain).state),
                    (*swapchain).state.device_window,
                    core::ptr::addr_of_mut!((*swapchain).state.original_window_rect),
                );
                wined3d_device_release_focus_window((*swapchain).device);
            }
        } else {
            wined3d_swapchain_state_restore_from_fullscreen(
                core::ptr::addr_of_mut!((*swapchain).state),
                (*swapchain).state.device_window,
                core::ptr::null(),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_gl_cleanup(swapchain_gl: *mut wined3d_swapchain_gl) {
    wined3d_swapchain_cleanup(core::ptr::addr_of_mut!((*swapchain_gl).s));
}

unsafe extern "C" fn wined3d_swapchain_vk_destroy_vulkan_swapchain(swapchain_vk: *mut wined3d_swapchain_vk) {
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let mut vk_info: *const wined3d_vk_info = core::ptr::null::<wined3d_vk_info>();
    let mut i: u32;
    let mut vr: VkResult = 0; // VK_SUCCESS

    vk_info = core::ptr::addr_of!((*wined3d_adapter_vk((*device_vk).d.adapter)).vk_info);

    vr = ((*vk_info).vk_ops.vkQueueWaitIdle.unwrap_unchecked())((*device_vk).vk_queue);
    if (vr as i32) < 0 {
        ERR!("Failed to wait on queue, vr %s.\n", wined3d_debug_vkresult(vr));
    }
    free((*swapchain_vk).vk_images as *mut core::ffi::c_void);
    i = 0;
    while i < (*swapchain_vk).image_count {
        ((*vk_info).vk_ops.vkDestroySemaphore.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*(*swapchain_vk).vk_semaphores.offset(i as isize)).available,
            core::ptr::null::<VkAllocationCallbacks>(),
        );
        ((*vk_info).vk_ops.vkDestroySemaphore.unwrap_unchecked())(
            (*device_vk).vk_device,
            (*(*swapchain_vk).vk_semaphores.offset(i as isize)).presentable,
            core::ptr::null::<VkAllocationCallbacks>(),
        );
        i = i.wrapping_add(1);
    }
    free((*swapchain_vk).vk_semaphores as *mut core::ffi::c_void);
    ((*vk_info).vk_ops.vkDestroySwapchainKHR.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*swapchain_vk).vk_swapchain,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
    ((*vk_info).vk_ops.vkDestroySurfaceKHR.unwrap_unchecked())(
        (*vk_info).instance,
        (*swapchain_vk).vk_surface,
        core::ptr::null::<VkAllocationCallbacks>(),
    );
}

unsafe extern "C" fn wined3d_swapchain_vk_destroy_object(object: *mut core::ffi::c_void) {
    wined3d_swapchain_vk_destroy_vulkan_swapchain((object).cast());
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_vk_cleanup(swapchain_vk: *mut wined3d_swapchain_vk) {
    let cs: *mut wined3d_cs = (*(*swapchain_vk).s.device).cs as *mut wined3d_cs;

    wined3d_cs_destroy_object(
        cs,
        Some(wined3d_swapchain_vk_destroy_object as unsafe extern "C" fn(*mut core::ffi::c_void)),
        swapchain_vk as *mut core::ffi::c_void,
    );
    wined3d_cs_finish(cs, WINED3D_CS_QUEUE_DEFAULT);

    wined3d_swapchain_cleanup(core::ptr::addr_of_mut!((*swapchain_vk).s));
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_incref(swapchain: *mut wined3d_swapchain) -> ULONG {
    let mut refcount: u32 = InterlockedIncrement(core::ptr::addr_of_mut!((*swapchain).ref_)) as u32;
    WINE_TRACE!("%p increasing refcount to %u.\n", swapchain, refcount);
    return refcount as ULONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_decref(swapchain: *mut wined3d_swapchain) -> ULONG {
    let mut refcount: u32 = InterlockedDecrement(
        core::ptr::addr_of_mut!((*swapchain).ref_) as *mut core::ffi::c_long,
    ) as u32;
    if refcount == 0 {
        let mut device: *mut wined3d_device = core::ptr::null_mut();
        wined3d_mutex_lock();
        device = (*swapchain).device;
        if (*device).swapchain_count != 0
            && *(*device).swapchains.offset(0 as isize) == swapchain
        {
            wined3d_device_uninit_3d(device);
        }
        wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);

        (*(*swapchain).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*swapchain).parent);
        (*(*(*(*swapchain).device).adapter).adapter_ops)
            .adapter_destroy_swapchain
            .unwrap_unchecked()(swapchain);

        wined3d_mutex_unlock();
    }
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_parent(
    swapchain: *const wined3d_swapchain,
) -> *mut core::ffi::c_void {
    // TRACE("swapchain %p.\n", swapchain);
    (*swapchain).parent
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_set_window(swapchain: *mut wined3d_swapchain, mut window: HWND) {
    if window.is_null() {
        window = (*swapchain).state.device_window;
    }
    if window == (*swapchain).win_handle {
        return;
    }
    TRACE!(
        "Setting swapchain %p window from %p to %p.\n",
        swapchain,
        (*swapchain).win_handle,
        window
    );
    wined3d_cs_finish((*(*swapchain).device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
    (*swapchain).win_handle = window;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_present(
    swapchain: *mut wined3d_swapchain,
    mut src_rect: *const RECT,
    mut dst_rect: *const RECT,
    dst_window_override: HWND,
    swap_interval: u32,
    flags: u32,
) -> HRESULT {
    let mut s: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut d: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };

    if flags != 0 {
        FIXME!("Ignoring flags %#x.\n", flags);
    }

    wined3d_mutex_lock();

    if (*swapchain).back_buffers.is_null() {
        WARN!("Swapchain doesn't have a backbuffer, returning WINED3DERR_INVALIDCALL.\n");
        wined3d_mutex_unlock();
        return WINED3DERR_INVALIDCALL;
    }

    if src_rect.is_null() {
        SetRect(
            core::ptr::addr_of_mut!(s),
            0,
            0,
            (*swapchain).state.desc.backbuffer_width as i32,
            (*swapchain).state.desc.backbuffer_height as i32,
        );
        src_rect = core::ptr::addr_of_mut!(s);
    }

    if dst_rect.is_null() {
        GetClientRect((*swapchain).win_handle, core::ptr::addr_of_mut!(d));
        dst_rect = core::ptr::addr_of_mut!(d);
    }

    wined3d_cs_emit_present(
        (*(*swapchain).device).cs as *mut wined3d_cs,
        swapchain,
        src_rect,
        dst_rect,
        dst_window_override,
        swap_interval,
        flags,
    );

    wined3d_mutex_unlock();

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_front_buffer_data(
    mut swapchain: *const wined3d_swapchain,
    mut dst_texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
) -> HRESULT {
    let mut src_rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut dst_rect: RECT = tagRECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    WINE_TRACE!("swapchain %p, dst_texture %p, sub_resource_idx %u.\n", swapchain, dst_texture, sub_resource_idx);
    SetRect(
        &raw mut src_rect,
        0,
        0,
        (*(*swapchain).front_buffer).resource.width as ::core::ffi::c_int,
        (*(*swapchain).front_buffer).resource.height as ::core::ffi::c_int,
    );
    dst_rect = src_rect;
    if (*swapchain).state.desc.windowed != 0 {
        MapWindowPoints(
            (*swapchain).win_handle,
            ::core::ptr::null_mut::<HWND__>(),
            &raw mut dst_rect as LPPOINT,
            2 as UINT,
        );
        WINE_FIXME!("Using destination rect %s in windowed mode, this is likely wrong.\n", wine_dbgstr_rect(&raw mut dst_rect));
    }
    return wined3d_device_context_blt(
        &raw mut (*(*(*swapchain).device).cs).c,
        dst_texture,
        sub_resource_idx,
        &raw mut dst_rect,
        (*swapchain).front_buffer,
        0 as ::core::ffi::c_uint,
        &raw mut src_rect,
        0 as ::core::ffi::c_uint,
        ::core::ptr::null::<wined3d_blt_fx>(),
        WINED3D_TEXF_POINT,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_back_buffer(
    swapchain: *const wined3d_swapchain,
    back_buffer_idx: UINT,
) -> *mut wined3d_texture {
    TRACE!("swapchain %p, back_buffer_idx %u.\n", swapchain, back_buffer_idx);

    /* Return invalid if there is no backbuffer array, otherwise it will
     * crash when ddraw is used (there swapchain->back_buffers is always
     * NULL). We need this because this function is called from
     * stateblock_init_default_state() to get the default scissorrect
     * dimensions. */
    if (*swapchain).back_buffers.is_null() || back_buffer_idx >= (*swapchain).state.desc.backbuffer_count {
        WARN!("Invalid back buffer index.\n");
        /* Native d3d9 doesn't set NULL here, just as wine's d3d9. But set it
         * here in wined3d to avoid problems in other libs. */
        return core::ptr::null_mut();
    }

    TRACE!("Returning back buffer %p.\n", *(*swapchain).back_buffers.offset(back_buffer_idx as isize));

    return *(*swapchain).back_buffers.offset(back_buffer_idx as isize);
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_front_buffer(
    swapchain: *const wined3d_swapchain,
) -> *mut wined3d_texture {
    // TRACE("swapchain %p.\n", swapchain);
    (*swapchain).front_buffer
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_output(
    mut swapchain: *const wined3d_swapchain,
) -> *mut wined3d_output {
    WINE_TRACE!("swapchain %p.\n", swapchain);
    return (*swapchain).state.desc.output as *mut wined3d_output;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_raster_status(
    swapchain: *const wined3d_swapchain,
    raster_status: *mut wined3d_raster_status,
) -> HRESULT {
    let mut output: *mut wined3d_output = core::ptr::null_mut();

    output = wined3d_swapchain_get_output(swapchain);
    if output.is_null() {
        ERR!("Failed to get output from swapchain %p.\n", swapchain);
        return E_FAIL;
    }

    return wined3d_output_get_raster_status(output, raster_status);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_state(swapchain: *mut wined3d_swapchain) -> *mut wined3d_swapchain_state {
    return core::ptr::addr_of_mut!((*swapchain).state);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_display_mode(
    swapchain: *const wined3d_swapchain,
    mode: *mut wined3d_display_mode,
    rotation: *mut wined3d_display_rotation,
) -> HRESULT {
    let mut output: *mut wined3d_output;
    let mut hr: HRESULT;

    output = wined3d_swapchain_get_output(swapchain);
    if output.is_null() {
        ERR!("Failed to get output from swapchain %p.\n", swapchain);
        return E_FAIL;
    }

    hr = wined3d_output_get_display_mode(output, mode, rotation);

    return hr;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_device(
    swapchain: *const wined3d_swapchain,
) -> *mut wined3d_device {
    // TRACE("swapchain %p.\n", swapchain);
    (*swapchain).device
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_desc(
    mut swapchain: *const wined3d_swapchain,
    mut desc: *mut wined3d_swapchain_desc,
) {
    WINE_TRACE!("swapchain %p, desc %p.\n", swapchain, desc);
    *desc = (*swapchain).state.desc;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_set_gamma_ramp(
    swapchain: *const wined3d_swapchain,
    flags: u32,
    ramp: *const wined3d_gamma_ramp,
) -> HRESULT {
    let mut output: *mut wined3d_output;

    if flags != 0 {
        FIXME!("Ignoring flags %#x.\n", flags);
    }

    output = wined3d_swapchain_get_output(swapchain);
    if output.is_null() {
        ERR!("Failed to get output from swapchain %p.\n", swapchain);
        return E_FAIL;
    }

    return wined3d_output_set_gamma_ramp(output, ramp);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_set_palette(swapchain: *mut wined3d_swapchain, palette: *mut wined3d_palette) {
    wined3d_cs_finish((*(*swapchain).device).cs, WINED3D_CS_QUEUE_DEFAULT);

    (*swapchain).palette = palette;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_get_gamma_ramp(swapchain: *const wined3d_swapchain, ramp: *mut wined3d_gamma_ramp) -> HRESULT {
    let mut output: *mut wined3d_output = core::ptr::null_mut();
    TRACE!("swapchain %p, ramp %p.\n", swapchain, ramp);
    output = wined3d_swapchain_get_output(swapchain);
    if output.is_null() {
        ERR!("Failed to get output from swapchain %p.\n", swapchain);
        return E_FAIL;
    }
    return wined3d_output_get_gamma_ramp(output, ramp);
}

unsafe extern "C" fn swapchain_blit_gdi(
    swapchain: *mut wined3d_swapchain,
    context: *mut wined3d_context,
    src_rect: *const RECT,
    dst_rect: *const RECT,
) {
    let back_buffer: *mut wined3d_texture = *(*swapchain).back_buffers.add(0 as usize);
    let mut destroy_desc: D3DKMT_DESTROYDCFROMMEMORY = core::mem::zeroed();
    let mut create_desc: D3DKMT_CREATEDCFROMMEMORY = core::mem::zeroed();
    let mut format: *const wined3d_format;
    let mut row_pitch: u32 = core::mem::zeroed();
    let mut slice_pitch: u32 = core::mem::zeroed();
    let mut src_dc: HDC;
    let mut dst_dc: HDC;
    let mut status: NTSTATUS;
    let mut bitmap: HBITMAP;

    static mut once: u32 = 0;

    let fresh_once = once;
    once = once.wrapping_add(1);
    if fresh_once == 0 {
        FIXME!("Using GDI present.\n");
    }

    format = (*back_buffer).resource.format;
    if (*format).ddi_format as u64 == 0 {
        WARN!("Cannot create a DC for format %s.\n", debug_d3dformat((*format).id));
        return;
    }

    wined3d_texture_load_location(back_buffer, 0, context, WINED3D_LOCATION_SYSMEM);
    wined3d_texture_get_pitch(
        back_buffer,
        0,
        core::ptr::addr_of_mut!(row_pitch),
        core::ptr::addr_of_mut!(slice_pitch),
    );

    create_desc.pMemory = (*back_buffer).resource.heap_memory;
    create_desc.Format = (*format).ddi_format;
    create_desc.Width = wined3d_texture_get_level_width(back_buffer, 0) as UINT;
    create_desc.Height = wined3d_texture_get_level_height(back_buffer, 0) as UINT;
    create_desc.Pitch = row_pitch as UINT;
    create_desc.hDeviceDc = CreateCompatibleDC(core::ptr::null_mut());
    create_desc.pColorTable = core::ptr::null_mut();

    status = D3DKMTCreateDCFromMemory(core::ptr::addr_of_mut!(create_desc)) as NTSTATUS;
    DeleteDC(create_desc.hDeviceDc);
    if status != 0 {
        WARN!("Failed to create DC, status %#lx.\n", status);
        return;
    }

    src_dc = create_desc.hDc;
    bitmap = create_desc.hBitmap as HBITMAP;

    dst_dc = GetDCEx(
        (*swapchain).win_handle,
        core::ptr::null_mut(),
        (DCX_USESTYLE | DCX_CACHE) as DWORD,
    );
    if dst_dc.is_null() {
        ERR!("Failed to get destination DC.\n");
    }

    if StretchBlt(
        dst_dc,
        (*dst_rect).left as core::ffi::c_int,
        (*dst_rect).top as core::ffi::c_int,
        (*dst_rect).right as core::ffi::c_int - (*dst_rect).left as core::ffi::c_int,
        (*dst_rect).bottom as core::ffi::c_int - (*dst_rect).top as core::ffi::c_int,
        src_dc,
        (*src_rect).left as core::ffi::c_int,
        (*src_rect).top as core::ffi::c_int,
        (*src_rect).right as core::ffi::c_int - (*src_rect).left as core::ffi::c_int,
        (*src_rect).bottom as core::ffi::c_int - (*src_rect).top as core::ffi::c_int,
        SRCCOPY as DWORD,
    ) == 0
    {
        ERR!("Failed to blit.\n");
    }

    ReleaseDC((*swapchain).win_handle, dst_dc);
    destroy_desc.hDc = src_dc;
    destroy_desc.hBitmap = bitmap as HANDLE;
    status = D3DKMTDestroyDCFromMemory(core::ptr::addr_of_mut!(destroy_desc)) as NTSTATUS;
    if status != 0 {
        ERR!("Failed to destroy src dc, status %#lx.\n", status);
    }
}

unsafe extern "C" fn swapchain_blit(
    mut swapchain: *const wined3d_swapchain,
    mut context: *mut wined3d_context,
    mut src_rect: *const RECT,
    mut dst_rect: *const RECT,
) {
    let mut texture: *mut wined3d_texture = *(*swapchain).back_buffers.offset(0 as isize);
    let mut device: *mut wined3d_device = (*swapchain).device;
    let mut filter: wined3d_texture_filter_type = 0;
    let mut location: DWORD = 0;
    WINE_TRACE!("swapchain %p, context %p, src_rect %s, dst_rect %s.\n", swapchain, context, wine_dbgstr_rect(src_rect), wine_dbgstr_rect(dst_rect));
    if (*src_rect).right - (*src_rect).left == (*dst_rect).right - (*dst_rect).left
        && (*src_rect).bottom - (*src_rect).top == (*dst_rect).bottom - (*dst_rect).top
        || is_complex_fixup((*(*texture).resource.format).color_fixup) != 0
    {
        filter = WINED3D_TEXF_NONE;
    } else {
        filter = WINED3D_TEXF_LINEAR;
    }
    location = WINED3D_LOCATION_TEXTURE_RGB;
    if (*texture).resource.multisample_type as u64 != 0 {
        location = WINED3D_LOCATION_RB_RESOLVED;
    }
    wined3d_texture_validate_location(texture, 0, WINED3D_LOCATION_DRAWABLE);
    (*(*(*device).blitter).ops)
        .blitter_blit
        .unwrap_unchecked()(
        (*device).blitter,
        WINED3D_BLIT_OP_COLOR_BLIT,
        context,
        texture,
        0,
        location,
        src_rect,
        texture,
        0,
        WINED3D_LOCATION_DRAWABLE,
        dst_rect,
        ::core::ptr::null::<wined3d_color_key>(),
        filter,
        ::core::ptr::null::<wined3d_format>(),
    );
    wined3d_texture_invalidate_location(texture, 0, WINED3D_LOCATION_DRAWABLE);
}

unsafe extern "C" fn swapchain_gl_set_swap_interval(
    swapchain: *mut wined3d_swapchain,
    context_gl: *mut wined3d_context_gl,
    mut swap_interval: u32,
) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    swap_interval = if swap_interval <= 4 {
        swap_interval
    } else {
        1
    };
    if (*swapchain).swap_interval == swap_interval {
        return;
    }

    (*swapchain).swap_interval = swap_interval;

    if (*gl_info).supported[WGL_EXT_SWAP_CONTROL as usize] == 0 {
        return;
    }

    if ((*gl_info).gl_ops.ext.p_wglSwapIntervalEXT.unwrap_unchecked())(swap_interval as ::core::ffi::c_int) == 0 {
        ERR!(
            "Failed to set swap interval %u for context %p, last error %#lx.\n",
            swap_interval,
            context_gl,
            GetLastError()
        );
    }
}

unsafe extern "C" fn wined3d_swapchain_gl_rotate(swapchain: *mut wined3d_swapchain, context: *mut wined3d_context) {
    let mut sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut texture: *mut wined3d_texture_gl = core::ptr::null_mut();
    let mut texture_prev: *mut wined3d_texture_gl = core::ptr::null_mut();
    let mut tex0: gl_texture = core::mem::zeroed();
    let mut rb0: GLuint = 0;
    let mut locations0: DWORD = 0;
    let mut i: core::ffi::c_uint = 0;
    static mut supported_locations: DWORD = WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_RB_MULTISAMPLE;

    if (*swapchain).state.desc.backbuffer_count < 2 {
        return;
    }

    texture_prev = wined3d_texture_gl(*(*swapchain).back_buffers.offset(0 as isize));

    /* Back buffer 0 is already in the draw binding. */
    tex0 = (*texture_prev).texture_rgb;
    rb0 = (*texture_prev).rb_multisample;
    locations0 = (*(*texture_prev).t.sub_resources.offset(0 as isize)).locations;

    i = 1;
    while i < (*swapchain).state.desc.backbuffer_count {
        texture = wined3d_texture_gl(*(*swapchain).back_buffers.offset(i as isize));
        sub_resource = (*texture).t.sub_resources.offset(0 as isize) as *mut wined3d_texture_sub_resource;

        if (*sub_resource).locations & supported_locations == 0 {
            wined3d_texture_load_location(
                core::ptr::addr_of_mut!((*texture).t),
                0,
                context,
                (*texture).t.resource.draw_binding as uint32_t,
            );
        }

        (*texture_prev).texture_rgb = (*texture).texture_rgb;
        (*texture_prev).rb_multisample = (*texture).rb_multisample;

        wined3d_texture_validate_location(
            core::ptr::addr_of_mut!((*texture_prev).t),
            0,
            (*sub_resource).locations as uint32_t & supported_locations as uint32_t,
        );
        wined3d_texture_invalidate_location(
            core::ptr::addr_of_mut!((*texture_prev).t),
            0,
            !((*sub_resource).locations as uint32_t & supported_locations as uint32_t),
        );

        texture_prev = texture;
        i = i.wrapping_add(1);
    }

    (*texture_prev).texture_rgb = tex0;
    (*texture_prev).rb_multisample = rb0;

    wined3d_texture_validate_location(
        core::ptr::addr_of_mut!((*texture_prev).t),
        0,
        locations0 as uint32_t & supported_locations as uint32_t,
    );
    wined3d_texture_invalidate_location(
        core::ptr::addr_of_mut!((*texture_prev).t),
        0,
        !(locations0 as uint32_t & supported_locations as uint32_t),
    );

    device_invalidate_state((*swapchain).device, (STATE_FRAMEBUFFER as u32));
}

unsafe extern "C" fn swapchain_present_is_partial_copy(swapchain: *mut wined3d_swapchain, dst_rect: *const RECT) -> bool {
    let swap_effect: wined3d_swap_effect = (*swapchain).state.desc.swap_effect;
    let mut client_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut t: ::core::ffi::c_uint;

    if swap_effect as ::core::ffi::c_uint != WINED3D_SWAP_EFFECT_COPY
        && swap_effect as ::core::ffi::c_uint != WINED3D_SWAP_EFFECT_COPY_VSYNC
    {
        return false;
    }

    GetClientRect((*swapchain).win_handle, core::ptr::addr_of_mut!(client_rect));

    t = (client_rect.right - client_rect.left) as ::core::ffi::c_uint;
    if ((*dst_rect).left != 0 && (*dst_rect).right != 0)
        || abs(((*dst_rect).right - (*dst_rect).left) as ::core::ffi::c_int)
            as ::core::ffi::c_uint
            != t
    {
        return true;
    }
    t = (client_rect.bottom - client_rect.top) as ::core::ffi::c_uint;
    if ((*dst_rect).top != 0 && (*dst_rect).bottom != 0)
        || abs(((*dst_rect).bottom - (*dst_rect).top) as ::core::ffi::c_int)
            as ::core::ffi::c_uint
            != t
    {
        return true;
    }

    return false;
}

unsafe extern "C" fn swapchain_gl_present(
    swapchain: *mut wined3d_swapchain,
    src_rect: *const RECT,
    dst_rect: *const RECT,
    swap_interval: ::core::ffi::c_uint,
    flags: uint32_t,
) {
    let mut back_buffer: *mut wined3d_texture = *(*swapchain).back_buffers.offset(0 as isize);
    let mut pixel_format: *const wined3d_pixel_format;
    let mut gl_info: *const wined3d_gl_info;
    let mut context_gl: *mut wined3d_context_gl;
    let mut context: *mut wined3d_context;

    context = context_acquire((*swapchain).device, (*swapchain).front_buffer, 0);
    context_gl = wined3d_context_gl(context);
    if (*context_gl).valid() == 0 {
        context_release(context);
        WARN!("Invalid context, skipping present.\n");
        return;
    }

    pixel_format = (*wined3d_adapter_gl((*(*swapchain).device).adapter))
        .pixel_formats
        .offset((*context_gl).pixel_format as isize);
    if (*context_gl).dc == (*wined3d_device_gl((*swapchain).device)).backup_dc
        || ((*pixel_format).swap_method != (WGL_SWAP_COPY_ARB as i32)
            && (swapchain_present_is_partial_copy(swapchain, dst_rect) as i32) != 0)
    {
        swapchain_blit_gdi(swapchain, context, src_rect, dst_rect);
    } else {
        gl_info = (*context_gl).gl_info;

        swapchain_gl_set_swap_interval(swapchain, context_gl, swap_interval);

        wined3d_texture_load_location(back_buffer, 0, context, ((*back_buffer).resource.draw_binding) as uint32_t);

        swapchain_blit(swapchain, context, src_rect, dst_rect);

        if (*(*swapchain).device).context_count > 1 {
            WINE_WARN_!(__wine_dbch_d3d_perf, "Multiple contexts, calling glFinish() to enforce ordering.\n");
            (*gl_info).gl_ops.gl.p_glFinish.unwrap_unchecked()();
        }

        /* call wglSwapBuffers through the gl table to avoid confusing the Steam overlay */
        (*gl_info).gl_ops.wgl.p_wglSwapBuffers.unwrap_unchecked()((*context_gl).dc);
    }

    if (*(*context).d3d_info).fences() != 0 {
        wined3d_context_gl_submit_command_fence(context_gl);
    }

    wined3d_swapchain_gl_rotate(swapchain, context);

    wined3d_texture_validate_location((*swapchain).front_buffer, 0, WINED3D_LOCATION_DRAWABLE);
    wined3d_texture_invalidate_location((*swapchain).front_buffer, 0, !WINED3D_LOCATION_DRAWABLE);

    context_release(context);
}

unsafe extern "C" fn swapchain_frontbuffer_updated(swapchain: *mut wined3d_swapchain) {
    let mut front_buffer: *mut wined3d_texture = (*swapchain).front_buffer;
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();

    context = context_acquire((*swapchain).device, front_buffer, 0 as ::core::ffi::c_uint);
    wined3d_texture_load_location(front_buffer, 0 as ::core::ffi::c_uint, context, (*front_buffer).resource.draw_binding as u32);
    context_release(context);
    SetRectEmpty(core::ptr::addr_of_mut!((*swapchain).front_buffer_update));
}

unsafe extern "C" fn wined3d_swapchain_vk_present_mode_supported(
    swapchain_vk: *mut wined3d_swapchain_vk,
    vk_present_mode: VkPresentModeKHR,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let vk_info: *const wined3d_vk_info;
    let adapter_vk: *mut wined3d_adapter_vk;
    let vk_physical_device: VkPhysicalDevice;
    let vk_modes: *mut VkPresentModeKHR;
    let mut supported: bool = false;
    let mut count: u32 = 0;
    let mut i: u32 = 0;
    let mut vr: VkResult;

    adapter_vk = wined3d_adapter_vk((*device_vk).d.adapter);
    vk_physical_device = (*adapter_vk).physical_device;
    vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);

    vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap_unchecked())(
        vk_physical_device,
        (*swapchain_vk).vk_surface,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );
    if (vr as i32) < 0 {
        ERR!(
            "Failed to get supported present mode count, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return false;
    }

    vk_modes = calloc((count as usize) as _, (core::mem::size_of::<VkPresentModeKHR>()) as _)
        as *mut VkPresentModeKHR;
    if vk_modes.is_null() {
        return false;
    }

    'done: {
        vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfacePresentModesKHR.unwrap_unchecked())(
            vk_physical_device,
            (*swapchain_vk).vk_surface,
            core::ptr::addr_of_mut!(count),
            vk_modes,
        );
        if (vr as i32) < 0 {
            ERR!(
                "Failed to get supported present modes, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'done;
        }

        i = 0 as u32;
        while i < count {
            if *vk_modes.offset(i as isize) == vk_present_mode {
                supported = true;
                break 'done;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }

    free(vk_modes as *mut core::ffi::c_void);
    return supported;
}

unsafe extern "C" fn get_swapchain_fallback_format(mut vk_format: VkFormat) -> VkFormat {
    match vk_format as ::core::ffi::c_uint {
        VK_FORMAT_R8G8B8A8_SRGB => {
            return VK_FORMAT_B8G8R8A8_SRGB;
        }
        VK_FORMAT_R8G8B8A8_UNORM | VK_FORMAT_A2B10G10R10_UNORM_PACK32 | VK_FORMAT_R16G16B16A16_SFLOAT => {
            return VK_FORMAT_B8G8R8A8_UNORM;
        }
        _ => {
            WARN!("Unhandled format %#x.\n", vk_format);
            return VK_FORMAT_UNDEFINED;
        }
    }
}

unsafe extern "C" fn wined3d_swapchain_vk_select_vk_format(
    swapchain_vk: *mut wined3d_swapchain_vk,
    vk_surface: VkSurfaceKHR,
) -> VkFormat {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let desc: *const wined3d_swapchain_desc = core::ptr::addr_of!((*swapchain_vk).s.state.desc);
    let vk_info: *const wined3d_vk_info;
    let adapter_vk: *mut wined3d_adapter_vk;
    let format: *const wined3d_format;
    let vk_physical_device: VkPhysicalDevice;
    let vk_formats: *mut VkSurfaceFormatKHR;
    let mut format_count: uint32_t = 0;
    let mut i: uint32_t = 0;
    let mut vk_format: VkFormat;
    let mut vr: VkResult;

    adapter_vk = wined3d_adapter_vk((*device_vk).d.adapter);
    vk_physical_device = (*adapter_vk).physical_device;
    vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);

    format = wined3d_get_format(
        core::ptr::addr_of_mut!((*adapter_vk).a),
        (*desc).backbuffer_format,
        WINED3D_BIND_RENDER_TARGET,
    );
    if !format.is_null() {
        vk_format = (*wined3d_format_vk(format)).vk_format;
    } else {
        vk_format = VK_FORMAT_B8G8R8A8_UNORM;
    }

    vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap_unchecked())(
        vk_physical_device,
        vk_surface,
        core::ptr::addr_of_mut!(format_count),
        core::ptr::null_mut(),
    );
    if (vr as ::core::ffi::c_int) < 0 || format_count == 0 {
        WARN!(
            "Failed to get supported surface format count, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return VK_FORMAT_UNDEFINED;
    }

    vk_formats = calloc(
        (format_count as usize) as _,
        (core::mem::size_of::<VkSurfaceFormatKHR>()) as _,
    ) as *mut VkSurfaceFormatKHR;
    if vk_formats.is_null() {
        return VK_FORMAT_UNDEFINED;
    }

    vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfaceFormatsKHR.unwrap_unchecked())(
        vk_physical_device,
        vk_surface,
        core::ptr::addr_of_mut!(format_count),
        vk_formats,
    );
    if (vr as ::core::ffi::c_int) < 0 {
        WARN!(
            "Failed to get supported surface formats, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        free(vk_formats as *mut ::core::ffi::c_void);
        return VK_FORMAT_UNDEFINED;
    }

    i = 0;
    while i < format_count {
        if (*vk_formats.offset(i as isize)).format == vk_format
            && (*vk_formats.offset(i as isize)).colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
        {
            break;
        }
        i += 1;
    }
    if i == format_count {
        /* Try to create a swapchain with format conversion. */
        vk_format = get_swapchain_fallback_format(vk_format);
        WARN!(
            "Failed to find Vulkan swapchain format for %s.\n",
            debug_d3dformat((*desc).backbuffer_format)
        );
        i = 0;
        while i < format_count {
            if (*vk_formats.offset(i as isize)).format == vk_format
                && (*vk_formats.offset(i as isize)).colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
            {
                break;
            }
            i += 1;
        }
    }
    free(vk_formats as *mut ::core::ffi::c_void);
    if i == format_count {
        return VK_FORMAT_UNDEFINED;
    }

    vk_format
}

unsafe extern "C" fn wined3d_swapchain_vk_create_vulkan_swapchain_images(
    swapchain_vk: *mut wined3d_swapchain_vk,
    vk_swapchain: VkSwapchainKHR,
) -> bool {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let vk_info: *const wined3d_vk_info =
        core::ptr::addr_of!((*wined3d_adapter_vk((*device_vk).d.adapter)).vk_info);
    let mut semaphore_info: VkSemaphoreCreateInfo = core::mem::zeroed();
    let mut image_count: u32 = 0;
    let mut i: u32 = 0;
    let mut vr: VkResult = 0;

    if {
        vr = ((*vk_info).vk_ops.vkGetSwapchainImagesKHR.unwrap_unchecked())(
            (*device_vk).vk_device,
            vk_swapchain,
            core::ptr::addr_of_mut!(image_count),
            core::ptr::null_mut::<VkImage>(),
        );
        vr
    } < 0
    {
        ERR!("Failed to get image count, vr %s\n", wined3d_debug_vkresult(vr));
        return false;
    }

    (*swapchain_vk).vk_images = calloc(
        (image_count as usize) as _,
        (core::mem::size_of::<VkImage>()) as _,
    ) as *mut VkImage;
    if (*swapchain_vk).vk_images.is_null() {
        ERR!("Failed to allocate images array.\n");
        return false;
    }

    vr = ((*vk_info).vk_ops.vkGetSwapchainImagesKHR.unwrap_unchecked())(
        (*device_vk).vk_device,
        vk_swapchain,
        core::ptr::addr_of_mut!(image_count),
        (*swapchain_vk).vk_images,
    );
    if vr < 0 {
        ERR!("Failed to get swapchain images, vr %s.\n", wined3d_debug_vkresult(vr));
        free((*swapchain_vk).vk_images as *mut core::ffi::c_void);
        return false;
    }

    (*swapchain_vk).vk_semaphores = calloc(
        (image_count as usize) as _,
        (core::mem::size_of::<wined3d_swapchain_vk__bindgen_ty_1>()) as _,
    ) as *mut wined3d_swapchain_vk__bindgen_ty_1;
    if (*swapchain_vk).vk_semaphores.is_null() {
        ERR!("Failed to allocate semaphores array.\n");
        free((*swapchain_vk).vk_images as *mut core::ffi::c_void);
        return false;
    }

    semaphore_info.sType = VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO;
    semaphore_info.pNext = core::ptr::null::<core::ffi::c_void>();
    semaphore_info.flags = 0;
    i = 0;

    'fail: {
        while i < image_count {
            vr = ((*vk_info).vk_ops.vkCreateSemaphore.unwrap_unchecked())(
                (*device_vk).vk_device,
                core::ptr::addr_of_mut!(semaphore_info),
                core::ptr::null::<VkAllocationCallbacks>(),
                core::ptr::addr_of_mut!(
                    (*(*swapchain_vk).vk_semaphores.add(i as usize)).available
                ),
            );
            if vr < 0 {
                ERR!("Failed to create semaphore, vr %s.\n", wined3d_debug_vkresult(vr));
                break 'fail;
            }

            vr = ((*vk_info).vk_ops.vkCreateSemaphore.unwrap_unchecked())(
                (*device_vk).vk_device,
                core::ptr::addr_of_mut!(semaphore_info),
                core::ptr::null::<VkAllocationCallbacks>(),
                core::ptr::addr_of_mut!(
                    (*(*swapchain_vk).vk_semaphores.add(i as usize)).presentable
                ),
            );
            if vr < 0 {
                ERR!("Failed to create semaphore, vr %s.\n", wined3d_debug_vkresult(vr));
                break 'fail;
            }

            i = i.wrapping_add(1);
        }
        (*swapchain_vk).image_count = image_count;

        return true;
    }

    // fail:
    i = 0;
    while i < image_count {
        if (*(*swapchain_vk).vk_semaphores.add(i as usize)).available != 0 {
            ((*vk_info).vk_ops.vkDestroySemaphore.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*(*swapchain_vk).vk_semaphores.add(i as usize)).available,
                core::ptr::null::<VkAllocationCallbacks>(),
            );
        }
        if (*(*swapchain_vk).vk_semaphores.add(i as usize)).presentable != 0 {
            ((*vk_info).vk_ops.vkDestroySemaphore.unwrap_unchecked())(
                (*device_vk).vk_device,
                (*(*swapchain_vk).vk_semaphores.add(i as usize)).presentable,
                core::ptr::null::<VkAllocationCallbacks>(),
            );
        }
        i = i.wrapping_add(1);
    }
    free((*swapchain_vk).vk_semaphores as *mut core::ffi::c_void);
    free((*swapchain_vk).vk_images as *mut core::ffi::c_void);
    return false;
}

unsafe extern "C" fn wined3d_swapchain_vk_create_vulkan_swapchain(
    swapchain_vk: *mut wined3d_swapchain_vk,
) -> HRESULT {
    let device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let desc: *const wined3d_swapchain_desc = core::ptr::addr_of!((*swapchain_vk).s.state.desc);
    let mut vk_swapchain_desc: VkSwapchainCreateInfoKHR = core::mem::zeroed();
    let mut surface_desc: VkWin32SurfaceCreateInfoKHR = core::mem::zeroed();
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut image_count: u32 = 0;
    let mut vk_info: *const wined3d_vk_info = core::ptr::null();
    let mut surface_caps: VkSurfaceCapabilitiesKHR = core::mem::zeroed();
    let mut adapter_vk: *mut wined3d_adapter_vk = core::ptr::null_mut();
    let mut vk_present_mode: VkPresentModeKHR = 0;
    let mut vk_swapchain: VkSwapchainKHR = 0;
    let mut usage: VkImageUsageFlags = 0;
    let mut vk_surface: VkSurfaceKHR = 0;
    let mut supported: VkBool32 = 0;
    let mut vk_format: VkFormat = 0;
    let mut client_rect: RECT = core::mem::zeroed();
    let mut vr: VkResult = 0;

    adapter_vk = wined3d_adapter_vk((*device_vk).d.adapter);
    vk_info = core::ptr::addr_of!((*adapter_vk).vk_info);

    surface_desc.sType = VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR as _;
    surface_desc.pNext = core::ptr::null();
    surface_desc.flags = 0 as _;
    surface_desc.hinstance =
        GetWindowLongPtrW((*swapchain_vk).s.win_handle, GWLP_HINSTANCE) as HINSTANCE;
    surface_desc.hwnd = (*swapchain_vk).s.win_handle;
    vr = ((*vk_info).vk_ops.vkCreateWin32SurfaceKHR.unwrap_unchecked())(
        (*vk_info).instance,
        core::ptr::addr_of_mut!(surface_desc),
        core::ptr::null(),
        core::ptr::addr_of_mut!(vk_surface),
    );
    if vr < 0 {
        ERR!(
            "Failed to create Vulkan surface, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return E_FAIL;
    }
    (*swapchain_vk).vk_surface = vk_surface;

    'fail: {
        vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfaceSupportKHR.unwrap_unchecked())(
            (*adapter_vk).physical_device,
            (*device_vk).vk_queue_family_index,
            vk_surface,
            core::ptr::addr_of_mut!(supported),
        );
        if vr < 0 || supported == 0 {
            ERR!(
                "Queue family does not support presentation on this surface, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'fail;
        }

        vk_format = wined3d_swapchain_vk_select_vk_format(swapchain_vk, vk_surface);
        if vk_format == VK_FORMAT_UNDEFINED as VkFormat {
            ERR!("Failed to select swapchain format.\n");
            break 'fail;
        }

        vr = ((*vk_info).vk_ops.vkGetPhysicalDeviceSurfaceCapabilitiesKHR.unwrap_unchecked())(
            (*adapter_vk).physical_device,
            (*swapchain_vk).vk_surface,
            core::ptr::addr_of_mut!(surface_caps),
        );
        if vr < 0 {
            ERR!(
                "Failed to get surface capabilities, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'fail;
        }

        image_count = (*desc).backbuffer_count;
        if image_count < surface_caps.minImageCount {
            image_count = surface_caps.minImageCount;
        } else if surface_caps.maxImageCount != 0 && image_count > surface_caps.maxImageCount {
            image_count = surface_caps.maxImageCount;
        }

        if image_count != (*desc).backbuffer_count {
            WARN!(
                "Image count %u is not supported (%u-%u).\n",
                (*desc).backbuffer_count,
                surface_caps.minImageCount,
                surface_caps.maxImageCount
            );
        }

        GetClientRect(
            (*swapchain_vk).s.win_handle,
            core::ptr::addr_of_mut!(client_rect),
        );

        width = (client_rect.right - client_rect.left) as u32;
        if width < surface_caps.minImageExtent.width {
            width = surface_caps.minImageExtent.width;
        } else if width > surface_caps.maxImageExtent.width {
            width = surface_caps.maxImageExtent.width;
        }

        height = (client_rect.bottom - client_rect.top) as u32;
        if height < surface_caps.minImageExtent.height {
            height = surface_caps.minImageExtent.height;
        } else if height > surface_caps.maxImageExtent.height {
            height = surface_caps.maxImageExtent.height;
        }

        if width != (client_rect.right - client_rect.left) as u32
            || height != (client_rect.bottom - client_rect.top) as u32
        {
            WARN!(
                "Swapchain dimensions %lux%lu are not supported (%u-%u x %u-%u).\n",
                client_rect.right - client_rect.left,
                client_rect.bottom - client_rect.top,
                surface_caps.minImageExtent.width,
                surface_caps.maxImageExtent.width,
                surface_caps.minImageExtent.height,
                surface_caps.maxImageExtent.height
            );
        }

        usage = VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as VkImageUsageFlags;
        usage |= surface_caps.supportedUsageFlags & VK_IMAGE_USAGE_TRANSFER_SRC_BIT as VkImageUsageFlags;
        usage |= surface_caps.supportedUsageFlags & VK_IMAGE_USAGE_TRANSFER_DST_BIT as VkImageUsageFlags;
        if (usage & VK_IMAGE_USAGE_TRANSFER_SRC_BIT as VkImageUsageFlags) == 0
            || (usage & VK_IMAGE_USAGE_TRANSFER_DST_BIT as VkImageUsageFlags) == 0
        {
            WARN!("Transfer not supported for swapchain images.\n");
        }

        if (surface_caps.supportedCompositeAlpha & VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR as VkCompositeAlphaFlagsKHR)
            == 0
        {
            FIXME!("Unsupported alpha mode, %#x.\n", surface_caps.supportedCompositeAlpha);
            break 'fail;
        }

        vk_present_mode = VK_PRESENT_MODE_FIFO_KHR as VkPresentModeKHR;
        if (*swapchain_vk).s.swap_interval == 0 {
            if wined3d_swapchain_vk_present_mode_supported(swapchain_vk, VK_PRESENT_MODE_IMMEDIATE_KHR as VkPresentModeKHR) != false {
                vk_present_mode = VK_PRESENT_MODE_IMMEDIATE_KHR as VkPresentModeKHR;
            } else {
                FIXME!(
                    "Unsupported swap interval %u.\n",
                    (*swapchain_vk).s.swap_interval
                );
            }
        }

        vk_swapchain_desc.sType = VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR as _;
        vk_swapchain_desc.pNext = core::ptr::null();
        vk_swapchain_desc.flags = 0 as _;
        vk_swapchain_desc.surface = vk_surface;
        vk_swapchain_desc.minImageCount = image_count;
        vk_swapchain_desc.imageFormat = vk_format;
        vk_swapchain_desc.imageColorSpace = VK_COLOR_SPACE_SRGB_NONLINEAR_KHR as _;
        vk_swapchain_desc.imageExtent.width = width;
        vk_swapchain_desc.imageExtent.height = height;
        vk_swapchain_desc.imageArrayLayers = 1 as _;
        vk_swapchain_desc.imageUsage = usage;
        vk_swapchain_desc.imageSharingMode = VK_SHARING_MODE_EXCLUSIVE as _;
        vk_swapchain_desc.queueFamilyIndexCount = 0 as _;
        vk_swapchain_desc.pQueueFamilyIndices = core::ptr::null();
        vk_swapchain_desc.preTransform = surface_caps.currentTransform;
        vk_swapchain_desc.compositeAlpha = VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR as _;
        vk_swapchain_desc.presentMode = vk_present_mode;
        vk_swapchain_desc.clipped = VK_TRUE as _;
        vk_swapchain_desc.oldSwapchain = VK_NULL_HANDLE as _;
        vr = ((*vk_info).vk_ops.vkCreateSwapchainKHR.unwrap_unchecked())(
            (*device_vk).vk_device,
            core::ptr::addr_of_mut!(vk_swapchain_desc),
            core::ptr::null(),
            core::ptr::addr_of_mut!(vk_swapchain),
        );
        if vr < 0 {
            ERR!(
                "Failed to create Vulkan swapchain, vr %s.\n",
                wined3d_debug_vkresult(vr)
            );
            break 'fail;
        }
        (*swapchain_vk).vk_swapchain = vk_swapchain;

        if wined3d_swapchain_vk_create_vulkan_swapchain_images(swapchain_vk, vk_swapchain) == false {
            ((*vk_info).vk_ops.vkDestroySwapchainKHR.unwrap_unchecked())(
                (*device_vk).vk_device,
                vk_swapchain,
                core::ptr::null(),
            );
            break 'fail;
        }

        (*swapchain_vk).width = width;
        (*swapchain_vk).height = height;

        return WINED3D_OK;
    }

    ((*vk_info).vk_ops.vkDestroySurfaceKHR.unwrap_unchecked())(
        (*vk_info).instance,
        vk_surface,
        core::ptr::null(),
    );
    return E_FAIL;
}

unsafe extern "C" fn wined3d_swapchain_vk_recreate(swapchain_vk: *mut wined3d_swapchain_vk) -> HRESULT {
    TRACE!("swapchain_vk %p.\n", swapchain_vk);

    wined3d_swapchain_vk_destroy_vulkan_swapchain(swapchain_vk);

    return wined3d_swapchain_vk_create_vulkan_swapchain(swapchain_vk);
}

unsafe extern "C" fn wined3d_swapchain_vk_set_swap_interval(swapchain_vk: *mut wined3d_swapchain_vk, mut swap_interval: u32) {
    if swap_interval > 1 {
        if swap_interval <= 4 {
            FIXME!("Unsupported swap interval %u.\n", swap_interval);
        }
        swap_interval = 1;
    }

    if (*swapchain_vk).s.swap_interval == swap_interval {
        return;
    }

    (*swapchain_vk).s.swap_interval = swap_interval;
    wined3d_swapchain_vk_recreate(swapchain_vk);
}

unsafe extern "C" fn wined3d_swapchain_vk_blit(
    swapchain_vk: *mut wined3d_swapchain_vk,
    context_vk: *mut wined3d_context_vk,
    src_rect: *const RECT,
    mut dst_rect: *const RECT,
    swap_interval: u32,
) -> VkResult {
    let mut back_buffer_vk: *mut wined3d_texture_vk = wined3d_texture_vk(
        *(*swapchain_vk).s.back_buffers.offset(0 as isize),
    );
    let mut device_vk: *mut wined3d_device_vk = wined3d_device_vk((*swapchain_vk).s.device);
    let mut desc: *const wined3d_swapchain_desc = core::ptr::addr_of!((*swapchain_vk).s.state.desc);
    let mut vk_info: *const wined3d_vk_info = (*context_vk).vk_info;
    let mut vk_command_buffer: VkCommandBuffer = core::ptr::null_mut();
    let mut vk_range: VkImageSubresourceRange = core::mem::zeroed();
    let mut present_desc: VkPresentInfoKHR = core::mem::zeroed();
    let mut present_idx: u32;
    let mut vk_layout: VkImageLayout;
    let mut image_idx: u32 = 0;
    let mut dst_rect_tmp: RECT = core::mem::zeroed();
    let mut blit: VkImageBlit = core::mem::zeroed();
    let mut filter: VkFilter;
    let mut vr: VkResult = 0;

    static wait_stage: VkPipelineStageFlags = VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as VkPipelineStageFlags;

    wined3d_swapchain_vk_set_swap_interval(swapchain_vk, swap_interval);

    let fresh_current = (*swapchain_vk).current;
    (*swapchain_vk).current = fresh_current.wrapping_add(1);
    present_idx = fresh_current % (*swapchain_vk).image_count;
    wined3d_context_vk_wait_command_buffer(
        context_vk,
        (*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).command_buffer_id,
    );
    vr = ((*vk_info).vk_ops.vkAcquireNextImageKHR.unwrap_unchecked())(
        (*device_vk).vk_device,
        (*swapchain_vk).vk_swapchain,
        UINT64_MAX as u64,
        (*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).available,
        VK_NULL_HANDLE as VkFence,
        core::ptr::addr_of_mut!(image_idx),
    );
    if vr < 0 as VkResult {
        WARN!("Failed to acquire image, vr %s.\n", wined3d_debug_vkresult(vr));
        return vr;
    }

    if (*dst_rect).right as u32 > (*swapchain_vk).width
        || (*dst_rect).bottom as u32 > (*swapchain_vk).height
    {
        dst_rect_tmp = *dst_rect;
        if (*dst_rect).right as u32 > (*swapchain_vk).width {
            dst_rect_tmp.right = (*swapchain_vk).width as i32;
        }
        if (*dst_rect).bottom as u32 > (*swapchain_vk).height {
            dst_rect_tmp.bottom = (*swapchain_vk).height as i32;
        }
        dst_rect = core::ptr::addr_of!(dst_rect_tmp);
    }
    if (*src_rect).right - (*src_rect).left != (*dst_rect).right - (*dst_rect).left
        || (*src_rect).bottom - (*src_rect).top != (*dst_rect).bottom - (*dst_rect).top
    {
        filter = VK_FILTER_LINEAR as VkFilter;
    } else {
        filter = VK_FILTER_NEAREST as VkFilter;
    }
    vk_command_buffer = wined3d_context_vk_get_command_buffer(context_vk);

    wined3d_context_vk_end_current_render_pass(context_vk);

    vk_range.aspectMask = VK_IMAGE_ASPECT_COLOR_BIT as VkImageAspectFlags;
    vk_range.baseMipLevel = 0 as u32;
    vk_range.levelCount = 1 as u32;
    vk_range.baseArrayLayer = 0 as u32;
    vk_range.layerCount = 1 as u32;

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        vk_access_mask_from_bind_flags((*back_buffer_vk).t.resource.bind_flags as u32),
        VK_ACCESS_TRANSFER_READ_BIT as VkAccessFlags,
        (*back_buffer_vk).layout as VkImageLayout,
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout,
        (*back_buffer_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_range),
    );

    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        0 as VkAccessFlags,
        VK_ACCESS_TRANSFER_WRITE_BIT as VkAccessFlags,
        VK_IMAGE_LAYOUT_UNDEFINED as VkImageLayout,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL as VkImageLayout,
        *(*swapchain_vk).vk_images.offset(image_idx as isize),
        core::ptr::addr_of_mut!(vk_range),
    );

    blit.srcSubresource.aspectMask = vk_range.aspectMask;
    blit.srcSubresource.mipLevel = vk_range.baseMipLevel;
    blit.srcSubresource.baseArrayLayer = vk_range.baseArrayLayer;
    blit.srcSubresource.layerCount = vk_range.layerCount;
    blit.srcOffsets[0 as usize].x = (*src_rect).left as i32;
    blit.srcOffsets[0 as usize].y = (*src_rect).top as i32;
    blit.srcOffsets[0 as usize].z = 0 as i32;
    blit.srcOffsets[1 as usize].x = (*src_rect).right as i32;
    blit.srcOffsets[1 as usize].y = (*src_rect).bottom as i32;
    blit.srcOffsets[1 as usize].z = 1 as i32;
    blit.dstSubresource = blit.srcSubresource;
    blit.dstOffsets[0 as usize].x = (*dst_rect).left as i32;
    blit.dstOffsets[0 as usize].y = (*dst_rect).top as i32;
    blit.dstOffsets[0 as usize].z = 0 as i32;
    blit.dstOffsets[1 as usize].x = (*dst_rect).right as i32;
    blit.dstOffsets[1 as usize].y = (*dst_rect).bottom as i32;
    blit.dstOffsets[1 as usize].z = 1 as i32;
    ((*vk_info).vk_ops.vkCmdBlitImage.unwrap_unchecked())(
        vk_command_buffer,
        (*back_buffer_vk).image.vk_image,
        VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout,
        *(*swapchain_vk).vk_images.offset(image_idx as isize),
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL as VkImageLayout,
        1 as u32,
        core::ptr::addr_of_mut!(blit),
        filter,
    );

    wined3d_context_vk_reference_texture(context_vk, back_buffer_vk);
    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT as VkPipelineStageFlags,
        VK_ACCESS_TRANSFER_WRITE_BIT as VkAccessFlags,
        0 as VkAccessFlags,
        VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL as VkImageLayout,
        VK_IMAGE_LAYOUT_PRESENT_SRC_KHR as VkImageLayout,
        *(*swapchain_vk).vk_images.offset(image_idx as isize),
        core::ptr::addr_of_mut!(vk_range),
    );

    if (*desc).swap_effect as u32 == WINED3D_SWAP_EFFECT_DISCARD as u32
        || (*desc).swap_effect as u32 == WINED3D_SWAP_EFFECT_FLIP_DISCARD as u32
    {
        vk_layout = VK_IMAGE_LAYOUT_UNDEFINED as VkImageLayout;
    } else {
        vk_layout = VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL as VkImageLayout;
    }
    wined3d_context_vk_image_barrier(
        context_vk,
        vk_command_buffer,
        VK_PIPELINE_STAGE_TRANSFER_BIT as VkPipelineStageFlags,
        VK_PIPELINE_STAGE_ALL_COMMANDS_BIT as VkPipelineStageFlags,
        VK_ACCESS_TRANSFER_READ_BIT as VkAccessFlags,
        vk_access_mask_from_bind_flags((*back_buffer_vk).t.resource.bind_flags as u32),
        vk_layout,
        (*back_buffer_vk).layout as VkImageLayout,
        (*back_buffer_vk).image.vk_image,
        core::ptr::addr_of_mut!(vk_range),
    );
    (*back_buffer_vk).bind_mask = 0 as u32;

    (*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).command_buffer_id =
        (*context_vk).current_command_buffer.id;
    wined3d_context_vk_submit_command_buffer(
        context_vk,
        1 as u32,
        core::ptr::addr_of_mut!((*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).available),
        core::ptr::addr_of!(wait_stage),
        1 as u32,
        core::ptr::addr_of_mut!((*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).presentable),
    );

    present_desc.sType = VK_STRUCTURE_TYPE_PRESENT_INFO_KHR as VkStructureType;
    present_desc.pNext = core::ptr::null();
    present_desc.waitSemaphoreCount = 1 as u32;
    present_desc.pWaitSemaphores =
        core::ptr::addr_of!((*(*swapchain_vk).vk_semaphores.offset(present_idx as isize)).presentable);
    present_desc.swapchainCount = 1 as u32;
    present_desc.pSwapchains = core::ptr::addr_of!((*swapchain_vk).vk_swapchain);
    present_desc.pImageIndices = core::ptr::addr_of!(image_idx);
    present_desc.pResults = core::ptr::null_mut();
    vr = ((*vk_info).vk_ops.vkQueuePresentKHR.unwrap_unchecked())(
        (*device_vk).vk_queue,
        core::ptr::addr_of_mut!(present_desc),
    );
    if vr != 0 {
        WARN!("Present returned vr %s.\n", wined3d_debug_vkresult(vr));
    }
    return vr;
}

unsafe extern "C" fn wined3d_swapchain_vk_rotate(
    mut swapchain: *mut wined3d_swapchain,
    mut context_vk: *mut wined3d_context_vk,
) {
    let mut sub_resource: *mut wined3d_texture_sub_resource =
        ::core::ptr::null_mut::<wined3d_texture_sub_resource>();
    let mut texture: *mut wined3d_texture_vk = ::core::ptr::null_mut::<wined3d_texture_vk>();
    let mut texture_prev: *mut wined3d_texture_vk = ::core::ptr::null_mut::<wined3d_texture_vk>();
    let mut image0: wined3d_image_vk = core::mem::zeroed();
    let mut vk_info0: VkDescriptorImageInfo = core::mem::zeroed();
    let mut vk_layout0: VkImageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
    let mut bind_mask0: uint32_t = 0;
    let mut locations0: DWORD = 0;
    let mut i: ::core::ffi::c_uint = 0;

    let supported_locations: DWORD = WINED3D_LOCATION_TEXTURE_RGB | WINED3D_LOCATION_RB_MULTISAMPLE;

    if (*swapchain).state.desc.backbuffer_count < 2 {
        return;
    }

    texture_prev = wined3d_texture_vk(
        *(*swapchain).back_buffers.offset(0 as isize),
    );

    /* Back buffer 0 is already in the draw binding. */
    image0 = (*texture_prev).image;
    vk_layout0 = (*texture_prev).layout as VkImageLayout;
    bind_mask0 = (*texture_prev).bind_mask;
    vk_info0 = (*texture_prev).default_image_info;
    locations0 = (*(*texture_prev)
        .t
        .sub_resources
        .add(0 as usize))
    .locations;

    i = 1 as ::core::ffi::c_uint;
    while i < (*swapchain).state.desc.backbuffer_count {
        texture = wined3d_texture_vk(*(*swapchain).back_buffers.offset(i as isize));
        sub_resource = (*texture)
            .t
            .sub_resources
            .add(0 as usize);

        if (*sub_resource).locations & supported_locations == 0 {
            wined3d_texture_load_location(
                core::ptr::addr_of_mut!((*texture).t),
                0 as ::core::ffi::c_uint,
                core::ptr::addr_of_mut!((*context_vk).c),
                (*texture).t.resource.draw_binding as uint32_t,
            );
        }

        (*texture_prev).image = (*texture).image;
        (*texture_prev).layout = (*texture).layout;
        (*texture_prev).bind_mask = (*texture).bind_mask;
        (*texture_prev).default_image_info = (*texture).default_image_info;

        wined3d_texture_validate_location(
            core::ptr::addr_of_mut!((*texture_prev).t),
            0 as ::core::ffi::c_uint,
            (*sub_resource).locations as uint32_t & supported_locations as uint32_t,
        );
        wined3d_texture_invalidate_location(
            core::ptr::addr_of_mut!((*texture_prev).t),
            0 as ::core::ffi::c_uint,
            !((*sub_resource).locations as uint32_t & supported_locations as uint32_t),
        );

        texture_prev = texture;
        i = i.wrapping_add(1);
    }

    (*texture_prev).image = image0;
    (*texture_prev).layout = vk_layout0 as VkImageLayout;
    (*texture_prev).bind_mask = bind_mask0;
    (*texture_prev).default_image_info = vk_info0;

    wined3d_texture_validate_location(
        core::ptr::addr_of_mut!((*texture_prev).t),
        0 as ::core::ffi::c_uint,
        locations0 as uint32_t & supported_locations as uint32_t,
    );
    wined3d_texture_invalidate_location(
        core::ptr::addr_of_mut!((*texture_prev).t),
        0 as ::core::ffi::c_uint,
        !(locations0 as uint32_t & supported_locations as uint32_t),
    );

    device_invalidate_state((*swapchain).device, STATE_FRAMEBUFFER as ::core::ffi::c_uint);
}

unsafe extern "C" fn swapchain_vk_present(
    mut swapchain: *mut wined3d_swapchain,
    mut src_rect: *const RECT,
    mut dst_rect: *const RECT,
    mut swap_interval: ::core::ffi::c_uint,
    mut flags: uint32_t,
) {
    let mut swapchain_vk: *mut wined3d_swapchain_vk = wined3d_swapchain_vk(swapchain);
    let mut back_buffer: *mut wined3d_texture =
        *(*swapchain).back_buffers.offset(0 as isize);
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut::<wined3d_context_vk>();
    let mut vr: VkResult = VK_SUCCESS;
    let mut hr: HRESULT = 0;

    context_vk = wined3d_context_vk(context_acquire(
        (*swapchain).device,
        back_buffer,
        0 as ::core::ffi::c_uint,
    ));

    if (*swapchain_vk).vk_swapchain == 0
        || swapchain_present_is_partial_copy(swapchain, dst_rect) as ::core::ffi::c_int != 0
    {
        swapchain_blit_gdi(
            swapchain,
            core::ptr::addr_of_mut!((*context_vk).c),
            src_rect,
            dst_rect,
        );
    } else {
        wined3d_texture_load_location(
            back_buffer,
            0 as ::core::ffi::c_uint,
            core::ptr::addr_of_mut!((*context_vk).c),
            (*back_buffer).resource.draw_binding as uint32_t,
        );

        vr = wined3d_swapchain_vk_blit(swapchain_vk, context_vk, src_rect, dst_rect, swap_interval);
        if vr as u64 != 0 {
            if vr as ::core::ffi::c_int == VK_ERROR_OUT_OF_DATE_KHR
                || vr as ::core::ffi::c_int == VK_SUBOPTIMAL_KHR
            {
                hr = wined3d_swapchain_vk_recreate(swapchain_vk);
                if hr < 0 {
                    ERR!("Failed to recreate swapchain, hr %#lx.\n", hr);
                } else if vr as ::core::ffi::c_int == VK_ERROR_OUT_OF_DATE_KHR && {
                    vr = wined3d_swapchain_vk_blit(
                        swapchain_vk,
                        context_vk,
                        src_rect,
                        dst_rect,
                        swap_interval,
                    );
                    vr as u64 != 0
                } {
                    ERR!(
                        "Failed to blit image, vr %s.\n",
                        wined3d_debug_vkresult(vr)
                    );
                }
            } else {
                ERR!(
                    "Failed to blit image, vr %s.\n",
                    wined3d_debug_vkresult(vr)
                );
            }
        }
    }

    wined3d_swapchain_vk_rotate(swapchain, context_vk);

    wined3d_texture_validate_location(
        (*swapchain).front_buffer,
        0 as ::core::ffi::c_uint,
        WINED3D_LOCATION_DRAWABLE as uint32_t,
    );
    wined3d_texture_invalidate_location(
        (*swapchain).front_buffer,
        0 as ::core::ffi::c_uint,
        !(WINED3D_LOCATION_DRAWABLE) as uint32_t,
    );

    TRACE!("Starting new frame.\n");

    context_release(core::ptr::addr_of_mut!((*context_vk).c));
}

unsafe extern "C" fn swapchain_gdi_frontbuffer_updated(swapchain: *mut wined3d_swapchain) {
    let mut front: *mut wined3d_texture_wined3d_dc_info = core::ptr::null_mut();
    let mut offset: POINT = POINT {
        x: 0 as LONG,
        y: 0 as LONG,
    };
    let mut src_dc: HDC = core::ptr::null_mut();
    let mut dst_dc: HDC = core::ptr::null_mut();
    let mut draw_rect: RECT = RECT {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let mut window: HWND = core::ptr::null_mut();

    TRACE!("swapchain %p.\n", swapchain);

    front = (*(*swapchain).front_buffer)
        .dc_info
        .offset(0 as isize);
    if !(*swapchain).palette.is_null() {
        wined3d_palette_apply_to_dc((*swapchain).palette, (*front).dc);
    }

    if (*(*swapchain).front_buffer).resource.map_count != 0 {
        ERR!("Trying to blit a mapped surface.\n");
    }

    TRACE!("Copying surface %p to screen.\n", front);

    src_dc = (*front).dc;
    window = (*swapchain).win_handle;
    dst_dc = GetDCEx(
        window,
        0 as _,
        (DCX_CLIPSIBLINGS | DCX_CACHE) as DWORD,
    );

    /* Front buffer coordinates are screen coordinates. Map them to the
     * destination window if not fullscreened. */
    if (*swapchain).state.desc.windowed != 0 {
        ClientToScreen(window, core::ptr::addr_of_mut!(offset));
    }

    TRACE!("offset %s.\n", wine_dbgstr_point(core::ptr::addr_of_mut!(offset)));

    SetRect(
        core::ptr::addr_of_mut!(draw_rect),
        0 as _,
        0 as _,
        (*(*swapchain).front_buffer).resource.width as _,
        (*(*swapchain).front_buffer).resource.height as _,
    );
    IntersectRect(
        core::ptr::addr_of_mut!(draw_rect),
        core::ptr::addr_of_mut!(draw_rect) as LPCRECT,
        core::ptr::addr_of_mut!((*swapchain).front_buffer_update) as LPCRECT,
    );

    BitBlt(
        dst_dc,
        draw_rect.left as INT - offset.x as INT,
        draw_rect.top as INT - offset.y as INT,
        draw_rect.right as INT - draw_rect.left as INT,
        draw_rect.bottom as INT - draw_rect.top as INT,
        src_dc,
        draw_rect.left as INT,
        draw_rect.top as INT,
        SRCCOPY as DWORD,
    );
    ReleaseDC(window, dst_dc);

    SetRectEmpty(core::ptr::addr_of_mut!((*swapchain).front_buffer_update));
}

unsafe extern "C" fn swapchain_gdi_present(
    mut swapchain: *mut wined3d_swapchain,
    mut src_rect: *const RECT,
    mut dst_rect: *const RECT,
    mut swap_interval: ::core::ffi::c_uint,
    mut flags: uint32_t,
) {
    let mut front: *mut wined3d_dc_info = ::core::ptr::null_mut::<wined3d_dc_info>();
    let mut back: *mut wined3d_dc_info = ::core::ptr::null_mut::<wined3d_dc_info>();
    let mut bitmap: HBITMAP = ::core::ptr::null_mut::<HBITMAP__>();
    let mut data: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
    let mut dc: HDC = ::core::ptr::null_mut::<HDC__>();
    front = (core::ptr::addr_of_mut!((*(*swapchain).front_buffer).dc_info) as *mut wined3d_dc_info)
        .add(0);
    back = (core::ptr::addr_of_mut!((*(*(*swapchain).back_buffers.add(0))).dc_info)
        as *mut wined3d_dc_info)
        .add(0);
    dc = (*front).dc;
    bitmap = (*front).bitmap;
    data = (*(*swapchain).front_buffer).resource.heap_memory;
    (*front).dc = (*back).dc;
    (*front).bitmap = (*back).bitmap;
    (*(*swapchain).front_buffer).resource.heap_memory =
        (*(*(*swapchain).back_buffers.add(0))).resource.heap_memory;
    (*back).dc = dc;
    (*back).bitmap = bitmap;
    (*(*(*swapchain).back_buffers.add(0))).resource.heap_memory = data;
    SetRect(
        core::ptr::addr_of_mut!((*swapchain).front_buffer_update),
        0,
        0,
        (*(*swapchain).front_buffer).resource.width as ::core::ffi::c_int,
        (*(*swapchain).front_buffer).resource.height as ::core::ffi::c_int,
    );
    swapchain_gdi_frontbuffer_updated(swapchain);
}

unsafe extern "C" fn wined3d_swapchain_apply_sample_count_override(
    swapchain: *const wined3d_swapchain,
    format_id: wined3d_format_id,
    type_: *mut wined3d_multisample_type,
    quality: *mut ::core::ffi::c_uint,
) {
    let mut adapter: *const wined3d_adapter = ::core::ptr::null::<wined3d_adapter>();
    let mut format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    let mut t: ::core::ffi::c_uint = 0;
    if wined3d_settings.sample_count == !0 {
        return;
    }
    adapter = (*(*swapchain).device).adapter;
    format = wined3d_get_format(
        adapter,
        format_id,
        WINED3D_BIND_RENDER_TARGET,
    );
    if format.is_null() {
        return;
    }
    t = min!(
        wined3d_settings.sample_count,
        (*adapter).d3d_info.limits.sample_count,
    );
    if t != 0 {
        while ((*format).multisample_types as ::core::ffi::c_uint
            & (1 as ::core::ffi::c_uint)
                << (t.wrapping_sub(1 as ::core::ffi::c_uint)))
            == 0
        {
            t += 1;
        }
    }
    WINE_TRACE!("Using sample count %u.\n", t as ::core::ffi::c_uint);
    *type_ = t as wined3d_multisample_type;
    *quality = 0 as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn swapchain_set_max_frame_latency(swapchain: *mut wined3d_swapchain, device: *const wined3d_device) {
    (*swapchain).max_frame_latency = if (*device).max_frame_latency >= 2 {
        (*device).max_frame_latency - 1
    } else {
        1
    };
}

unsafe extern "C" fn adapter_format_from_backbuffer_format(
    mut adapter: *const wined3d_adapter,
    mut format_id: wined3d_format_id,
) -> wined3d_format_id {
    let mut backbuffer_format: *const wined3d_format = ::core::ptr::null::<wined3d_format>();
    backbuffer_format = wined3d_get_format(adapter, format_id, WINED3D_BIND_RENDER_TARGET);
    return pixelformat_for_depth(
        ((*backbuffer_format).byte_count as DWORD).wrapping_mul(CHAR_BIT as DWORD),
    );
}

unsafe extern "C" fn wined3d_swapchain_state_init(
    mut state: *mut wined3d_swapchain_state,
    mut desc: *const wined3d_swapchain_desc,
    mut window: HWND,
    mut wined3d: *mut wined3d,
    mut parent: *mut wined3d_swapchain_state_parent,
) -> HRESULT {
    let mut hr: HRESULT;

    (*state).desc = *desc;

    if {
        hr = wined3d_output_get_display_mode(
            (*desc).output,
            core::ptr::addr_of_mut!((*state).original_mode),
            core::ptr::null_mut(),
        );
        hr < 0
    } {
        ERR!("Failed to get current display mode, hr %#lx.\n", hr);
        return hr;
    }

    if (*state).desc.windowed != 0 {
        let mut client_rect: RECT = core::mem::zeroed();

        GetClientRect(window, core::ptr::addr_of_mut!(client_rect));
        TRACE!(
            "Client rect %s.\n",
            wine_dbgstr_rect(core::ptr::addr_of_mut!(client_rect))
        );

        if (*state).desc.backbuffer_width == 0 {
            (*state).desc.backbuffer_width = (if client_rect.right != 0 {
                client_rect.right
            } else {
                8 as _
            }) as UINT;
            TRACE!("Updating width to %u.\n", (*state).desc.backbuffer_width);
        }
        if (*state).desc.backbuffer_height == 0 {
            (*state).desc.backbuffer_height = (if client_rect.bottom != 0 {
                client_rect.bottom
            } else {
                8 as _
            }) as UINT;
            TRACE!("Updating height to %u.\n", (*state).desc.backbuffer_height);
        }

        if (*state).desc.backbuffer_format as ::core::ffi::c_uint
            == WINED3DFMT_UNKNOWN as ::core::ffi::c_uint
        {
            (*state).desc.backbuffer_format = (*state).original_mode.format_id;
            TRACE!(
                "Updating format to %s.\n",
                debug_d3dformat((*state).original_mode.format_id)
            );
        }
    } else {
        if ((*desc).flags & (WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH as uint32_t)) != 0 {
            (*state).d3d_mode.width = (*desc).backbuffer_width as UINT;
            (*state).d3d_mode.height = (*desc).backbuffer_height as UINT;
            (*state).d3d_mode.format_id = adapter_format_from_backbuffer_format(
                (*(*desc).output).adapter,
                (*desc).backbuffer_format,
            );
            (*state).d3d_mode.refresh_rate = (*desc).refresh_rate as UINT;
            (*state).d3d_mode.scanline_ordering = WINED3D_SCANLINE_ORDERING_UNKNOWN as _;
        } else {
            (*state).d3d_mode = (*state).original_mode;
        }
    }

    GetWindowRect(
        window,
        core::ptr::addr_of_mut!((*state).original_window_rect),
    );
    (*state).wined3d = wined3d;
    (*state).device_window = window;
    (*state).desc.device_window = window;
    (*state).parent = parent;

    if ((*desc).flags & (WINED3D_SWAPCHAIN_REGISTER_STATE as uint32_t)) != 0 {
        wined3d_swapchain_state_register(state);
    }

    return hr;
}

unsafe extern "C" fn swapchain_create_texture(
    swapchain: *mut wined3d_swapchain,
    front: bool,
    depth: bool,
    texture: *mut *mut wined3d_texture,
) -> HRESULT {
    let swapchain_desc: *const wined3d_swapchain_desc = core::ptr::addr_of!((*swapchain).state.desc);
    let device: *mut wined3d_device = (*swapchain).device;
    let mut texture_desc: wined3d_resource_desc = core::mem::zeroed();
    let mut texture_flags: uint32_t = 0;
    let mut hr: HRESULT = 0;

    texture_desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    texture_desc.format = if depth {
        (*swapchain_desc).auto_depth_stencil_format
    } else {
        (*swapchain_desc).backbuffer_format
    };
    texture_desc.multisample_type = (*swapchain_desc).multisample_type;
    texture_desc.multisample_quality = (*swapchain_desc).multisample_quality;
    texture_desc.usage = 0;
    if !depth && ((*(*device).wined3d).flags & WINED3D_NO3D) != 0 {
        texture_desc.usage |= WINED3DUSAGE_OWNDC;
    }
    if ((*(*device).wined3d).flags & WINED3D_NO3D) != 0 {
        texture_desc.access = WINED3D_RESOURCE_ACCESS_CPU;
    } else {
        texture_desc.access = WINED3D_RESOURCE_ACCESS_GPU;
    }
    if !depth && ((*swapchain_desc).flags & WINED3D_SWAPCHAIN_LOCKABLE_BACKBUFFER) != 0 {
        texture_desc.access |= WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    }
    texture_desc.width = (*swapchain_desc).backbuffer_width;
    texture_desc.height = (*swapchain_desc).backbuffer_height;
    texture_desc.depth = 1;
    texture_desc.size = 0;

    if front {
        texture_desc.bind_flags = 0;
    } else if depth {
        texture_desc.bind_flags = WINED3D_BIND_DEPTH_STENCIL;
    } else {
        texture_desc.bind_flags = (*swapchain_desc).backbuffer_bind_flags;
    }

    if ((*swapchain_desc).flags & WINED3D_SWAPCHAIN_GDI_COMPATIBLE) != 0 {
        texture_flags |= WINED3D_TEXTURE_CREATE_GET_DC;
    }

    hr = wined3d_texture_create(
        device,
        &mut texture_desc,
        1,
        1,
        texture_flags,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        texture,
    );
    if hr < 0 {
        WARN!("Failed to create texture, hr %#lx.\n", hr);
        return hr;
    }

    if !depth {
        wined3d_texture_set_swapchain(*texture, swapchain);
    }

    return S_OK;
}

unsafe extern "C" fn wined3d_swapchain_init(
    swapchain: *mut wined3d_swapchain,
    device: *mut wined3d_device,
    desc: *const wined3d_swapchain_desc,
    state_parent: *mut wined3d_swapchain_state_parent,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    swapchain_ops: *const wined3d_swapchain_ops,
) -> HRESULT {
    let mut output_desc: wined3d_output_desc = core::mem::zeroed();
    let mut displaymode_set: BOOL = (FALSE as i32);
    let mut hr: HRESULT = E_FAIL;
    let mut i: u32 = 0;
    let mut window: HWND = core::ptr::null_mut();

    wined3d_mutex_lock();

    if (*desc).backbuffer_count > 1 {
        FIXME!("The application requested more than one back buffer, this is not properly supported.\nPlease configure the application to use double buffering (1 back buffer) if possible.\n");
    }

    if (*desc).swap_effect != WINED3D_SWAP_EFFECT_DISCARD
        && (*desc).swap_effect != WINED3D_SWAP_EFFECT_SEQUENTIAL
        && (*desc).swap_effect != WINED3D_SWAP_EFFECT_COPY
    {
        FIXME!("Unimplemented swap effect %#x.\n", (*desc).swap_effect);
    }

    window = if !(*desc).device_window.is_null() {
        (*desc).device_window
    } else {
        (*device).create_parms.focus_window
    };

    TRACE!("Using target window %p.\n", window);

    hr = wined3d_swapchain_state_init(
        &raw mut (*swapchain).state,
        desc,
        window,
        (*device).wined3d,
        state_parent,
    );
    if hr < 0 {
        ERR!("Failed to initialise swapchain state, hr %#lx.\n", hr);
        wined3d_mutex_unlock();
        return hr;
    }

    (*swapchain).swapchain_ops = swapchain_ops;
    (*swapchain).device = device;
    (*swapchain).parent = parent;
    (*swapchain).parent_ops = parent_ops;
    (*swapchain).ref_ = 1;
    (*swapchain).win_handle = window;
    (*swapchain).swap_interval = WINED3D_SWAP_INTERVAL_DEFAULT;
    swapchain_set_max_frame_latency(swapchain, device);

    'err: {
        if (*swapchain).state.desc.windowed == 0 {
            hr = wined3d_output_get_desc((*desc).output, &raw mut output_desc);
            if hr < 0 {
                ERR!("Failed to get output description, hr %#lx.\n", hr);
                break 'err;
            }

            wined3d_swapchain_state_setup_fullscreen(
                &raw mut (*swapchain).state,
                window,
                output_desc.desktop_rect.left,
                output_desc.desktop_rect.top,
                ((*desc).backbuffer_width) as i32,
                ((*desc).backbuffer_height) as i32,
            );
        }
        wined3d_swapchain_apply_sample_count_override(
            swapchain,
            (*swapchain).state.desc.backbuffer_format,
            &raw mut (*swapchain).state.desc.multisample_type,
            &raw mut (*swapchain).state.desc.multisample_quality,
        );

        TRACE!("Creating front buffer.\n");
        hr = swapchain_create_texture(swapchain, true, false, &raw mut (*swapchain).front_buffer);
        if hr < 0 {
            WARN!("Failed to create front buffer, hr %#lx.\n", hr);
            break 'err;
        }

        if ((*(*device).wined3d).flags & WINED3D_NO3D) == 0 {
            wined3d_texture_validate_location(
                (*swapchain).front_buffer,
                0,
                WINED3D_LOCATION_DRAWABLE,
            );
            wined3d_texture_invalidate_location(
                (*swapchain).front_buffer,
                0,
                !WINED3D_LOCATION_DRAWABLE,
            );
        }

        /* MSDN says we're only allowed a single fullscreen swapchain per device,
         * so we should really check to see if there is a fullscreen swapchain
         * already. Does a single head count as full screen? */
        if (*desc).windowed == 0 && ((*desc).flags & WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH) != 0 {
            /* Change the display settings */
            hr = wined3d_output_set_display_mode(
                (*desc).output,
                &raw mut (*swapchain).state.d3d_mode,
            );
            if hr < 0 {
                WARN!("Failed to set display mode, hr %#lx.\n", hr);
                break 'err;
            }
            displaymode_set = (TRUE as i32);
        }

        if (*swapchain).state.desc.backbuffer_count > 0 {
            (*swapchain).back_buffers = calloc(
                ((*swapchain).state.desc.backbuffer_count as usize) as _,
                (core::mem::size_of::<*mut wined3d_texture>()) as _,
            ) as *mut *mut wined3d_texture;
            if (*swapchain).back_buffers.is_null() {
                ERR!("Failed to allocate backbuffer array memory.\n");
                hr = E_OUTOFMEMORY;
                break 'err;
            }

            i = 0;
            while i < (*swapchain).state.desc.backbuffer_count {
                TRACE!("Creating back buffer %u.\n", i);
                hr = swapchain_create_texture(
                    swapchain,
                    false,
                    false,
                    (*swapchain).back_buffers.offset(i as isize),
                );
                if hr < 0 {
                    WARN!("Failed to create back buffer %u, hr %#lx.\n", i, hr);
                    (*swapchain).state.desc.backbuffer_count = i;
                    break 'err;
                }
                i = i.wrapping_add(1);
            }
        }

        /* Swapchains share the depth/stencil buffer, so only create a single depthstencil surface. */
        if (*desc).enable_auto_depth_stencil != 0 {
            TRACE!("Creating depth/stencil buffer.\n");
            if (*device).auto_depth_stencil_view.is_null() {
                let mut desc: wined3d_view_desc = core::mem::zeroed();
                let mut ds: *mut wined3d_texture = core::ptr::null_mut();

                hr = swapchain_create_texture(swapchain, false, true, &raw mut ds);
                if hr < 0 {
                    WARN!("Failed to create the auto depth/stencil surface, hr %#lx.\n", hr);
                    break 'err;
                }

                desc.format_id = (*(*ds).resource.format).id;
                desc.flags = 0;
                desc.u.texture.level_idx = 0;
                desc.u.texture.level_count = 1;
                desc.u.texture.layer_idx = 0;
                desc.u.texture.layer_count = 1;
                hr = wined3d_rendertarget_view_create(
                    &raw mut desc,
                    &raw mut (*ds).resource,
                    core::ptr::null_mut(),
                    &raw const wined3d_null_parent_ops,
                    &raw mut (*device).auto_depth_stencil_view,
                );
                wined3d_texture_decref(ds);
                if hr < 0 {
                    ERR!("Failed to create rendertarget view, hr %#lx.\n", hr);
                    break 'err;
                }
            }
        }

        wined3d_swapchain_get_gamma_ramp(swapchain, &raw mut (*swapchain).orig_gamma);

        wined3d_mutex_unlock();

        return WINED3D_OK;
    }

    /* err: */
    if displaymode_set != 0 {
        if wined3d_restore_display_modes((*device).wined3d) < 0 {
            ERR!("Failed to restore display mode.\n");
        }
    }

    if !(*swapchain).back_buffers.is_null() {
        i = 0;
        while i < (*swapchain).state.desc.backbuffer_count {
            if !(*(*swapchain).back_buffers.offset(i as isize)).is_null() {
                wined3d_texture_set_swapchain(
                    *(*swapchain).back_buffers.offset(i as isize),
                    core::ptr::null_mut(),
                );
                wined3d_texture_decref(*(*swapchain).back_buffers.offset(i as isize));
            }
            i = i.wrapping_add(1);
        }
        free((*swapchain).back_buffers as *mut core::ffi::c_void);
    }

    if !(*swapchain).front_buffer.is_null() {
        wined3d_texture_set_swapchain((*swapchain).front_buffer, core::ptr::null_mut());
        wined3d_texture_decref((*swapchain).front_buffer);
    }

    wined3d_swapchain_state_cleanup(&raw mut (*swapchain).state);
    wined3d_mutex_unlock();

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_no3d_init(swapchain_no3d: *mut wined3d_swapchain, device: *mut wined3d_device, desc: *const wined3d_swapchain_desc, state_parent: *mut wined3d_swapchain_state_parent, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    WINE_TRACE!("swapchain_no3d %p, device %p, desc %p, state_parent %p, parent %p, parent_ops %p.\n", swapchain_no3d, device, desc, state_parent, parent, parent_ops);
    return wined3d_swapchain_init(
        swapchain_no3d,
        device,
        desc,
        state_parent,
        parent,
        parent_ops,
        core::ptr::addr_of!(swapchain_no3d_ops),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_gl_init(swapchain_gl: *mut wined3d_swapchain_gl, device: *mut wined3d_device, desc: *const wined3d_swapchain_desc, state_parent: *mut wined3d_swapchain_state_parent, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    WINE_TRACE!("swapchain_gl %p, device %p, desc %p, state_parent %p, parent %p, parent_ops %p.\n", swapchain_gl, device, desc, state_parent, parent, parent_ops);
    return wined3d_swapchain_init(
        core::ptr::addr_of_mut!((*swapchain_gl).s),
        device,
        desc,
        state_parent,
        parent,
        parent_ops,
        core::ptr::addr_of!(swapchain_gl_ops),
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_vk_init(swapchain_vk: *mut wined3d_swapchain_vk, device: *mut wined3d_device, desc: *const wined3d_swapchain_desc, state_parent: *mut wined3d_swapchain_state_parent, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) -> HRESULT {
    let mut hr: HRESULT = 0;

    hr = wined3d_swapchain_init(
        core::ptr::addr_of_mut!((*swapchain_vk).s),
        device,
        desc,
        state_parent,
        parent,
        parent_ops,
        core::ptr::addr_of!(swapchain_vk_ops),
    );
    if hr < 0 {
        return hr;
    }

    if (*swapchain_vk).s.win_handle == GetDesktopWindow() {
        WARN!("Creating a desktop window swapchain.\n");
        return hr;
    }

    hr = wined3d_swapchain_vk_create_vulkan_swapchain(swapchain_vk);
    if hr < 0 {
        WARN!("Failed to create a Vulkan swapchain, hr %#lx.\n", hr);
    }

    return WINED3D_OK;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_create(
    device: *mut wined3d_device,
    desc: *const wined3d_swapchain_desc,
    state_parent: *mut wined3d_swapchain_state_parent,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    swapchain: *mut *mut wined3d_swapchain,
) -> HRESULT {
    let mut object: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = ((*(*(*device).adapter).adapter_ops).adapter_create_swapchain.unwrap_unchecked())(
        device,
        desc,
        state_parent,
        parent,
        parent_ops,
        core::ptr::addr_of_mut!(object),
    );
    if hr < (0 as HRESULT) {
        return hr;
    }

    if ((*desc).flags & (WINED3D_SWAPCHAIN_IMPLICIT as uint32_t)) != 0 {
        wined3d_mutex_lock();
        hr = wined3d_device_set_implicit_swapchain(device, object);
        if hr < (0 as HRESULT) {
            wined3d_cs_finish((*device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);
            ((*(*(*device).adapter).adapter_ops).adapter_destroy_swapchain.unwrap_unchecked())(object);
            wined3d_mutex_unlock();
            return hr;
        }
        wined3d_mutex_unlock();
    }

    *swapchain = object;

    hr
}

unsafe extern "C" fn wined3d_swapchain_gl_create_context(
    mut swapchain_gl: *mut wined3d_swapchain_gl,
) -> *mut wined3d_context_gl {
    let mut device: *mut wined3d_device = (*swapchain_gl).s.device;
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();

    wined3d_from_cs((*device).cs);

    context_gl = calloc(
        (1 as usize) as _,
        (core::mem::size_of::<wined3d_context_gl>()) as _,
    ) as *mut wined3d_context_gl;
    if context_gl.is_null() {
        ERR!("Failed to allocate context memory.\n");
        return core::ptr::null_mut();
    }

    if (wined3d_context_gl_init(context_gl, swapchain_gl)) < 0 {
        WARN!("Failed to initialise context.\n");
        free(context_gl as *mut core::ffi::c_void);
        return core::ptr::null_mut();
    }

    if device_context_add(device, core::ptr::addr_of_mut!((*context_gl).c)) == 0 {
        ERR!("Failed to add the newly created context to the context list.\n");
        wined3d_context_gl_destroy(context_gl);
        return core::ptr::null_mut();
    }

    context_release(core::ptr::addr_of_mut!((*context_gl).c));

    return context_gl;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_gl_get_context(
    swapchain_gl: *mut wined3d_swapchain_gl,
) -> *mut wined3d_context_gl {
    let device: *mut wined3d_device = (*swapchain_gl).s.device;
    let tid: DWORD = GetCurrentThreadId();
    let mut i: ::core::ffi::c_uint;

    i = 0;
    while i < (*device).context_count {
        if (*wined3d_context_gl(*(*device).contexts.offset(i as isize))).tid == tid {
            return wined3d_context_gl(*(*device).contexts.offset(i as isize));
        }
        i = i.wrapping_add(1);
    }

    /* Create a new context for the thread. */
    return wined3d_swapchain_gl_create_context(swapchain_gl);
}
#[no_mangle]
pub unsafe extern "C" fn swapchain_update_draw_bindings(mut swapchain: *mut wined3d_swapchain) {
    let mut i: UINT = 0;
    wined3d_resource_update_draw_binding(core::ptr::addr_of_mut!((*(*swapchain).front_buffer).resource));
    while i < (*swapchain).state.desc.backbuffer_count {
        wined3d_resource_update_draw_binding(core::ptr::addr_of_mut!((**(*swapchain).back_buffers.offset(i as isize)).resource));
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_activate(swapchain: *mut wined3d_swapchain, activate: BOOL) {
    let device: *mut wined3d_device = (*swapchain).device;
    let window: HWND = (*swapchain).state.device_window;
    let mut output_desc: wined3d_output_desc = core::mem::zeroed::<wined3d_output_desc>();
    let mut screensaver_active: ::core::ffi::c_uint = 0;
    let mut output: *mut wined3d_output = core::ptr::null_mut();
    let focus_messages: BOOL =
        ((*(*device).wined3d).flags & WINED3D_FOCUS_MESSAGES as ::core::ffi::c_uint) as BOOL;
    let mut filter: BOOL = 0;
    let mut hr: HRESULT = 0;

    /* This code is not protected by the wined3d mutex, so it may run while
     * wined3d_device_reset is active. Testing on Windows shows that changing
     * focus during resets and resetting during focus change events causes
     * the application to crash with an invalid memory access. */

    if focus_messages == 0 {
        filter = wined3d_filter_messages(window, (TRUE) as BOOL);
    }

    if activate != 0 {
        SystemParametersInfoW(
            SPI_GETSCREENSAVEACTIVE as UINT,
            0 as UINT,
            core::ptr::addr_of_mut!(screensaver_active) as PVOID,
            0 as UINT,
        );
        (*device).set_restore_screensaver((screensaver_active != 0) as ::core::ffi::c_int as BYTE as BYTE);
        if (*device).restore_screensaver() != 0 {
            SystemParametersInfoW(
                SPI_SETSCREENSAVEACTIVE as UINT,
                FALSE as UINT,
                core::ptr::null_mut(),
                0 as UINT,
            );
        }

        if ((*device).create_parms.flags & (WINED3DCREATE_NOWINDOWCHANGES as uint32_t)) == 0 {
            /* The d3d versions do not agree on the exact messages here. D3d8 restores
             * the window but leaves the size untouched, d3d9 sets the size on an
             * invisible window, generates messages but doesn't change the window
             * properties. The implementation follows d3d9.
             *
             * Guild Wars 1 wants a WINDOWPOSCHANGED message on the device window to
             * resume drawing after a focus loss. */
            output = wined3d_swapchain_get_output(swapchain);
            if output.is_null() {
                ERR!("Failed to get output from swapchain %p.\n", swapchain);
                return;
            }

            hr = wined3d_output_get_desc(output, core::ptr::addr_of_mut!(output_desc));
            if hr >= 0 {
                SetWindowPos(
                    window,
                    core::ptr::null_mut(),
                    output_desc.desktop_rect.left,
                    output_desc.desktop_rect.top,
                    (*swapchain).state.desc.backbuffer_width as ::core::ffi::c_int,
                    (*swapchain).state.desc.backbuffer_height as ::core::ffi::c_int,
                    (SWP_NOACTIVATE | SWP_NOZORDER) as UINT,
                );
            } else {
                ERR!("Failed to get output description, hr %#lx.\n", hr);
            }
        }

        if (*(*device).wined3d).flags & WINED3D_RESTORE_MODE_ON_ACTIVATE as ::core::ffi::c_uint != 0 {
            output = wined3d_swapchain_get_output(swapchain);
            if output.is_null() {
                ERR!("Failed to get output from swapchain %p.\n", swapchain);
                return;
            }

            hr = wined3d_output_set_display_mode(output, core::ptr::addr_of_mut!((*swapchain).state.d3d_mode));
            if hr < 0 {
                ERR!("Failed to set display mode, hr %#lx.\n", hr);
            }
        }

        if swapchain == *(*device).swapchains.offset(0 as isize) {
            (*(*(*device).device_parent).ops)
                .activate
                .unwrap_unchecked()((*device).device_parent, (TRUE as i32));
        }
    } else {
        if (*device).restore_screensaver() != 0 {
            SystemParametersInfoW(
                SPI_SETSCREENSAVEACTIVE as UINT,
                TRUE as UINT,
                core::ptr::null_mut(),
                0 as UINT,
            );
            (*device).set_restore_screensaver(FALSE as BYTE as BYTE);
        }

        hr = wined3d_restore_display_modes((*device).wined3d);
        if hr < 0 {
            ERR!("Failed to restore display modes, hr %#lx.\n", hr);
        }

        (*swapchain).reapply_mode = TRUE != 0;

        /* Some DDraw apps (Deus Ex: GOTY, and presumably all UT 1 based games) destroy the device
         * during window minimization. Do our housekeeping now, as the device may not exist after
         * the ShowWindow call.
         *
         * In d3d9, the device is marked lost after the window is minimized. If we find an app
         * that needs this behavior (e.g. because it calls TestCooperativeLevel in the window proc)
         * we'll have to control this via a create flag. Note that the device and swapchain are not
         * safe to access after the ShowWindow call. */
        if swapchain == *(*device).swapchains.offset(0 as isize) {
            (*(*(*device).device_parent).ops)
                .activate
                .unwrap_unchecked()((*device).device_parent, (FALSE as i32));
        }

        if ((*device).create_parms.flags & (WINED3DCREATE_NOWINDOWCHANGES as uint32_t)) == 0
            && IsWindowVisible(window) != 0
        {
            ShowWindow(window, (SW_MINIMIZE as i32));
        }
    }

    if focus_messages == 0 {
        wined3d_filter_messages(window, filter);
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_resize_buffers(
    swapchain: *mut wined3d_swapchain,
    mut buffer_count: ::core::ffi::c_uint,
    mut width: ::core::ffi::c_uint,
    mut height: ::core::ffi::c_uint,
    mut format_id: wined3d_format_id,
    mut multisample_type: wined3d_multisample_type,
    mut multisample_quality: ::core::ffi::c_uint,
) -> HRESULT {
    let desc: *mut wined3d_swapchain_desc = core::ptr::addr_of_mut!((*swapchain).state.desc);
    let mut recreate: bool = false;

    wined3d_swapchain_apply_sample_count_override(
        swapchain,
        format_id,
        core::ptr::addr_of_mut!(multisample_type),
        core::ptr::addr_of_mut!(multisample_quality),
    );

    if buffer_count != 0 && buffer_count != (*desc).backbuffer_count {
        FIXME!("Cannot change the back buffer count yet.\n");
    }

    wined3d_cs_finish((*(*swapchain).device).cs, WINED3D_CS_QUEUE_DEFAULT);

    if width == 0 || height == 0 {
        let mut client_rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        /* The application is requesting that either the swapchain width or
         * height be set to the corresponding dimension in the window's
         * client rect. */

        if GetClientRect(
            (*swapchain).state.device_window,
            core::ptr::addr_of_mut!(client_rect),
        ) == 0
        {
            ERR!(
                "Failed to get client rect, last error %#lx.\n",
                GetLastError()
            );
            return WINED3DERR_INVALIDCALL;
        }

        if width == 0 {
            width = client_rect.right as ::core::ffi::c_uint;
        }

        if height == 0 {
            height = client_rect.bottom as ::core::ffi::c_uint;
        }
    }

    if width != (*desc).backbuffer_width || height != (*desc).backbuffer_height {
        (*desc).backbuffer_width = width;
        (*desc).backbuffer_height = height;
        recreate = true;
    }

    if format_id as ::core::ffi::c_uint == WINED3DFMT_UNKNOWN as ::core::ffi::c_uint {
        if (*desc).windowed == 0 {
            return WINED3DERR_INVALIDCALL;
        }
        format_id = (*swapchain).state.original_mode.format_id;
    }

    if format_id as ::core::ffi::c_uint != (*desc).backbuffer_format as ::core::ffi::c_uint {
        (*desc).backbuffer_format = format_id;
        recreate = true;
    }

    if multisample_type as ::core::ffi::c_uint != (*desc).multisample_type as ::core::ffi::c_uint
        || multisample_quality != (*desc).multisample_quality
    {
        (*desc).multisample_type = multisample_type;
        (*desc).multisample_quality = multisample_quality;
        recreate = true;
    }

    if recreate {
        let mut new_texture: *mut wined3d_texture = core::ptr::null_mut();
        let mut hr: HRESULT = 0;
        let mut i: UINT = 0;

        hr = swapchain_create_texture(swapchain, true, false, core::ptr::addr_of_mut!(new_texture));
        if hr < 0 {
            return hr;
        }
        wined3d_texture_set_swapchain((*swapchain).front_buffer, core::ptr::null_mut());
        if wined3d_texture_decref((*swapchain).front_buffer) != 0 {
            ERR!(
                "Something's still holding the front buffer (%p).\n",
                (*swapchain).front_buffer
            );
        }
        (*swapchain).front_buffer = new_texture;

        if (*(*(*swapchain).device).wined3d).flags & WINED3D_NO3D == 0 {
            wined3d_texture_validate_location(
                (*swapchain).front_buffer,
                0,
                WINED3D_LOCATION_DRAWABLE as uint32_t,
            );
            wined3d_texture_invalidate_location(
                (*swapchain).front_buffer,
                0,
                !WINED3D_LOCATION_DRAWABLE as uint32_t,
            );
        }

        i = 0;
        while i < (*desc).backbuffer_count {
            hr = swapchain_create_texture(swapchain, false, false, core::ptr::addr_of_mut!(new_texture));
            if hr < 0 {
                return hr;
            }
            wined3d_texture_set_swapchain(
                *(*swapchain).back_buffers.offset(i as isize),
                core::ptr::null_mut(),
            );
            if wined3d_texture_decref(*(*swapchain).back_buffers.offset(i as isize)) != 0 {
                ERR!(
                    "Something's still holding back buffer %u (%p).\n",
                    i,
                    *(*swapchain).back_buffers.offset(i as isize)
                );
            }
            *(*swapchain).back_buffers.offset(i as isize) = new_texture;
            i = i.wrapping_add(1);
        }
    }

    swapchain_update_draw_bindings(swapchain);

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_swapchain_state_set_display_mode(
    state: *mut wined3d_swapchain_state,
    output: *mut wined3d_output,
    mode: *mut wined3d_display_mode,
) -> HRESULT {
    let mut hr: HRESULT;

    if ((*state).desc.flags & WINED3D_SWAPCHAIN_USE_CLOSEST_MATCHING_MODE) != 0 {
        hr = wined3d_output_find_closest_matching_mode(output, mode);
        if hr < 0 {
            WARN!("Failed to find closest matching mode, hr %#lx.\n", hr);
        }
    }

    if output != (*state).desc.output {
        hr = wined3d_restore_display_modes((*state).wined3d);
        if hr < 0 {
            WARN!("Failed to restore display modes, hr %#lx.\n", hr);
            return hr;
        }

        hr = wined3d_output_get_display_mode(
            output,
            core::ptr::addr_of_mut!((*state).original_mode),
            core::ptr::null_mut(),
        );
        if hr < 0 {
            WARN!("Failed to get current display mode, hr %#lx.\n", hr);
            return hr;
        }
    }

    hr = wined3d_output_set_display_mode(output, mode);
    if hr < 0 {
        WARN!("Failed to set display mode, hr %#lx.\n", hr);
        return WINED3DERR_INVALIDCALL;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_resize_target(
    state: *mut wined3d_swapchain_state,
    mode: *const wined3d_display_mode,
) -> HRESULT {
    let mut actual_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_desc: wined3d_output_desc = core::mem::zeroed();
    let mut original_window_rect: RECT = core::mem::zeroed();
    let mut window_rect: RECT = core::mem::zeroed();
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut window: HWND = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    TRACE!("state %p, mode %p.\n", state, mode);

    wined3d_mutex_lock();

    window = (*state).device_window;

    if (*state).desc.windowed != 0 {
        SetRect(
            core::ptr::addr_of_mut!(window_rect),
            0,
            0,
            ((*mode).width) as i32,
            ((*mode).height) as i32,
        );
        AdjustWindowRectEx(
            core::ptr::addr_of_mut!(window_rect),
            GetWindowLongW(window, GWL_STYLE) as DWORD,
            (FALSE) as BOOL,
            GetWindowLongW(window, GWL_EXSTYLE) as DWORD,
        );
        GetWindowRect(window, core::ptr::addr_of_mut!(original_window_rect));

        x = original_window_rect.left;
        y = original_window_rect.top;
        width = window_rect.right - window_rect.left;
        height = window_rect.bottom - window_rect.top;
    } else {
        if ((*state).desc.flags & WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH) != 0 {
            actual_mode = *mode;
            hr = wined3d_swapchain_state_set_display_mode(
                state,
                (*state).desc.output as *mut wined3d_output,
                core::ptr::addr_of_mut!(actual_mode),
            );
            if hr < 0 {
                ERR!("Failed to set display mode, hr %#lx.\n", hr);
                wined3d_mutex_unlock();
                return hr;
            }
        }

        hr = wined3d_output_get_desc((*state).desc.output, core::ptr::addr_of_mut!(output_desc));
        if hr < 0 {
            ERR!("Failed to get output description, hr %#lx.\n", hr);
            wined3d_mutex_unlock();
            return hr;
        }

        x = output_desc.desktop_rect.left;
        y = output_desc.desktop_rect.top;
        width = output_desc.desktop_rect.right - output_desc.desktop_rect.left;
        height = output_desc.desktop_rect.bottom - output_desc.desktop_rect.top;
    }

    wined3d_mutex_unlock();

    MoveWindow(window, x, y, width, height, (TRUE) as BOOL);

    return WINED3D_OK;
}

unsafe extern "C" fn fullscreen_style(mut style: LONG) -> LONG {
    style |= (WS_POPUP | WS_SYSMENU) as LONG;
    style &= !((WS_CAPTION | WS_THICKFRAME) as LONG);

    return style;
}

unsafe extern "C" fn fullscreen_exstyle(mut exstyle: LONG) -> LONG {
    /* Filter out window decorations. */
    exstyle &= (!(WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE)) as i32;

    return exstyle;
}

unsafe extern "system" fn topmost_timer_proc(hwnd: HWND, msg: UINT, id: UINT_PTR, time: DWORD) {
    if (GetWindowLongW(hwnd, GWL_EXSTYLE) as i32 & WS_EX_TOPMOST as i32) == 0 {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0 as i32,
            0 as i32,
            0 as i32,
            0 as i32,
            (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) as UINT,
        );
    }
    KillTimer(hwnd, WINED3D_WINDOW_TOPMOST_TIMER_ID as UINT_PTR);
}

unsafe extern "system" fn set_window_state_thread(ctx: *mut ::core::ffi::c_void) -> DWORD {
    let s: *mut wined3d_window_state = ctx as *mut wined3d_window_state;
    let mut filter: bool;

    filter = wined3d_filter_messages((*s).window, (TRUE) as BOOL) != 0;

    if (*s).set_style {
        SetWindowLongW((*s).window, GWL_STYLE, (*s).style);
        SetWindowLongW((*s).window, GWL_EXSTYLE, (*s).exstyle);
    }
    SetWindowPos(
        (*s).window,
        (*s).window_pos_after,
        (*s).x,
        (*s).y,
        (*s).width,
        (*s).height,
        (*s).flags as UINT,
    );

    wined3d_filter_messages((*s).window, filter as BOOL);

    free(s as *mut ::core::ffi::c_void);

    return 0;
}

unsafe extern "C" fn set_window_state(s: *mut wined3d_window_state) {
    static timeout: UINT = 1500;
    let mut window_tid: DWORD = GetWindowThreadProcessId((*s).window, core::ptr::null_mut());
    let mut tid: DWORD = GetCurrentThreadId();
    let mut thread: HANDLE = core::ptr::null_mut();

    /* If the window belongs to a different thread, modifying the style and/or
     * position can potentially deadlock if that thread isn't processing
     * messages. */
    if window_tid == tid {
        /* Deus Ex: Game of the Year Edition removes WS_EX_TOPMOST after changing resolutions in
         * exclusive fullscreen mode. Tests show that WS_EX_TOPMOST will be restored when a ~1.5s
         * timer times out */
        if (*s).register_topmost_timer {
            if (*s).set_topmost_timer {
                SetTimer(
                    (*s).window,
                    WINED3D_WINDOW_TOPMOST_TIMER_ID as UINT_PTR,
                    timeout,
                    Some(topmost_timer_proc),
                );
            } else {
                KillTimer((*s).window, WINED3D_WINDOW_TOPMOST_TIMER_ID as UINT_PTR);
            }
        }

        set_window_state_thread(s as *mut core::ffi::c_void);
    } else {
        thread = CreateThread(
            core::ptr::null_mut(),
            0 as DWORD,
            Some(set_window_state_thread),
            s as PVOID,
            0 as DWORD,
            core::ptr::null_mut(),
        );
        if !thread.is_null() {
            SetThreadDescription(thread, RUSTCONV_WSTR_D0036AD7DA5A.as_ptr());
            CloseHandle(thread);
        } else {
            ERR!("Failed to create thread.\n");
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_setup_fullscreen(
    state: *mut wined3d_swapchain_state,
    window: HWND,
    x: ::core::ffi::c_int,
    y: ::core::ffi::c_int,
    width: ::core::ffi::c_int,
    height: ::core::ffi::c_int,
) -> HRESULT {
    let mut s: *mut wined3d_window_state = ::core::ptr::null_mut::<wined3d_window_state>();

    if IsWindow(window) == 0 {
        WARN!("%p is not a valid window.\n", window);
        return WINED3DERR_NOTAVAILABLE;
    }

    s = malloc(core::mem::size_of::<wined3d_window_state>() as size_t)
        as *mut wined3d_window_state;
    if s.is_null() {
        return E_OUTOFMEMORY;
    }
    (*s).window = window;
    (*s).window_pos_after = HWND_TOPMOST;
    (*s).x = x;
    (*s).y = y;
    (*s).width = width;
    (*s).height = height;

    if (*state).style != 0 || (*state).exstyle != 0 {
        ERR!(
            "Changing the window style for window %p, but another style (%08lx, %08lx) is already stored.\n",
            window,
            (*state).style,
            (*state).exstyle
        );
    }

    (*s).flags = (SWP_FRAMECHANGED | SWP_NOACTIVATE) as uint32_t;
    if ((*state).desc.flags & (WINED3D_SWAPCHAIN_NO_WINDOW_CHANGES as uint32_t)) != 0 {
        (*s).flags |= SWP_NOZORDER as uint32_t;
    } else {
        (*s).flags |= SWP_SHOWWINDOW as uint32_t;
    }

    (*state).style = GetWindowLongW(window, GWL_STYLE);
    (*state).exstyle = GetWindowLongW(window, GWL_EXSTYLE);

    (*s).style = fullscreen_style((*state).style);
    (*s).exstyle = fullscreen_exstyle((*state).exstyle);
    (*s).set_style = true;
    (*s).register_topmost_timer =
        ((*state).desc.flags & (WINED3D_SWAPCHAIN_REGISTER_TOPMOST_TIMER as uint32_t)) != 0;
    (*s).set_topmost_timer = true;

    set_window_state(s);
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_restore_from_fullscreen(
    state: *mut wined3d_swapchain_state,
    window: HWND,
    window_rect: *const RECT,
) {
    let mut s: *mut wined3d_window_state = core::ptr::null_mut::<wined3d_window_state>();
    let mut style: LONG = 0;
    let mut exstyle: LONG = 0;

    if (*state).style == 0 && (*state).exstyle == 0 {
        return;
    }

    s = malloc(core::mem::size_of::<wined3d_window_state>()) as *mut wined3d_window_state;
    if s.is_null() {
        return;
    }

    (*s).window = window;
    (*s).window_pos_after = core::ptr::null_mut::<HWND__>();
    (*s).flags = (SWP_FRAMECHANGED | SWP_NOZORDER | SWP_NOACTIVATE) as u32;

    if ((*state).desc.flags & (WINED3D_SWAPCHAIN_RESTORE_WINDOW_STATE as u32)) != 0
        && ((*state).desc.flags & (WINED3D_SWAPCHAIN_NO_WINDOW_CHANGES as u32)) == 0
    {
        (*s).window_pos_after = if ((*state).exstyle & (WS_EX_TOPMOST as LONG)) != 0 {
            HWND_TOPMOST
        } else {
            HWND_NOTOPMOST
        };
        (*s).flags |= (if ((*state).style & (WS_VISIBLE as LONG)) != 0 {
            SWP_SHOWWINDOW
        } else {
            SWP_HIDEWINDOW
        }) as u32;
        (*s).flags &= !(SWP_NOZORDER as u32);
    }

    style = GetWindowLongW(window, GWL_STYLE);
    exstyle = GetWindowLongW(window, GWL_EXSTYLE);

    /* These flags are set by wined3d_device_setup_fullscreen_window, not the
     * application, and we want to ignore them in the test below, since it's
     * not the application's fault that they changed. Additionally, we want to
     * preserve the current status of these flags (i.e. don't restore them) to
     * more closely emulate the behavior of Direct3D, which leaves these flags
     * alone when returning to windowed mode. */
    (*state).style ^= ((*state).style ^ style) & (WS_VISIBLE as LONG);
    (*state).exstyle ^= ((*state).exstyle ^ exstyle) & (WS_EX_TOPMOST as LONG);

    TRACE!(
        "Restoring window style of window %p to %08lx, %08lx.\n",
        window,
        (*state).style,
        (*state).exstyle
    );

    (*s).style = (*state).style;
    (*s).exstyle = (*state).exstyle;
    /* Only restore the style if the application didn't modify it during the
     * fullscreen phase. Some applications change it before calling Reset()
     * when switching between windowed and fullscreen modes (HL2), some
     * depend on the original style (Eve Online). */
    (*s).set_style = style == fullscreen_style((*state).style)
        && exstyle == fullscreen_exstyle((*state).exstyle);
    (*s).register_topmost_timer =
        ((*state).desc.flags & (WINED3D_SWAPCHAIN_REGISTER_TOPMOST_TIMER as u32)) != 0;
    (*s).set_topmost_timer = false;

    if !window_rect.is_null() {
        (*s).x = (*window_rect).left as ::core::ffi::c_int;
        (*s).y = (*window_rect).top as ::core::ffi::c_int;
        (*s).width = ((*window_rect).right - (*window_rect).left) as ::core::ffi::c_int;
        (*s).height = ((*window_rect).bottom - (*window_rect).top) as ::core::ffi::c_int;
    } else {
        (*s).height = 0 as ::core::ffi::c_int;
        (*s).width = (*s).height;
        (*s).y = (*s).width;
        (*s).x = (*s).y;
        (*s).flags |= (SWP_NOMOVE | SWP_NOSIZE) as u32;
    }

    set_window_state(s);

    /* Delete the old values. */
    (*state).style = 0 as LONG;
    (*state).exstyle = 0 as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_set_fullscreen(
    state: *mut wined3d_swapchain_state,
    swapchain_desc: *const wined3d_swapchain_desc,
    mode: *const wined3d_display_mode,
) -> HRESULT {
    let mut actual_mode: wined3d_display_mode = core::mem::zeroed();
    let mut output_desc: wined3d_output_desc = core::mem::zeroed();
    let mut windowed: BOOL = (*state).desc.windowed;
    let mut hr: HRESULT = 0;

    TRACE!("state %p, swapchain_desc %p, mode %p.\n", state, swapchain_desc, mode);

    if (*state).desc.flags & WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH != 0 {
        if !mode.is_null() {
            actual_mode = *mode;
            hr = wined3d_swapchain_state_set_display_mode(
                state,
                (*swapchain_desc).output as *mut wined3d_output,
                core::ptr::addr_of_mut!(actual_mode),
            );
            if hr < 0 {
                return hr;
            }
        } else {
            if (*swapchain_desc).windowed == 0 {
                actual_mode.width = (*swapchain_desc).backbuffer_width as UINT;
                actual_mode.height = (*swapchain_desc).backbuffer_height as UINT;
                actual_mode.refresh_rate = (*swapchain_desc).refresh_rate as UINT;
                actual_mode.format_id = adapter_format_from_backbuffer_format(
                    (*(*swapchain_desc).output).adapter,
                    (*swapchain_desc).backbuffer_format,
                );
                actual_mode.scanline_ordering = WINED3D_SCANLINE_ORDERING_UNKNOWN;
                hr = wined3d_swapchain_state_set_display_mode(
                    state,
                    (*swapchain_desc).output as *mut wined3d_output,
                    core::ptr::addr_of_mut!(actual_mode),
                );
                if hr < 0 {
                    return hr;
                }
            } else {
                hr = wined3d_restore_display_modes((*state).wined3d);
                if hr < 0 {
                    WARN!("Failed to restore display modes for all outputs, hr %#lx.\n", hr);
                    return hr;
                }
            }
        }
    } else {
        if !mode.is_null() {
            WARN!("WINED3D_SWAPCHAIN_ALLOW_MODE_SWITCH is not set, ignoring mode.\n");
        }

        hr = wined3d_output_get_display_mode(
            (*swapchain_desc).output,
            core::ptr::addr_of_mut!(actual_mode),
            core::ptr::null_mut(),
        );
        if hr < 0 {
            ERR!("Failed to get display mode, hr %#lx.\n", hr);
            return WINED3DERR_INVALIDCALL;
        }
    }

    if (*swapchain_desc).windowed == 0 {
        let width: core::ffi::c_uint = actual_mode.width as core::ffi::c_uint;
        let height: core::ffi::c_uint = actual_mode.height as core::ffi::c_uint;

        hr = wined3d_output_get_desc(
            (*swapchain_desc).output,
            core::ptr::addr_of_mut!(output_desc),
        );
        if hr < 0 {
            ERR!("Failed to get output description, hr %#lx.\n", hr);
            return hr;
        }

        if (*state).desc.windowed != 0 {
            /* Switch from windowed to fullscreen */
            hr = wined3d_swapchain_state_setup_fullscreen(
                state,
                (*state).device_window,
                output_desc.desktop_rect.left as core::ffi::c_int,
                output_desc.desktop_rect.top as core::ffi::c_int,
                width as core::ffi::c_int,
                height as core::ffi::c_int,
            );
            if hr < 0 {
                return hr;
            }
        } else {
            /* Fullscreen -> fullscreen mode change */
            let window: HWND = (*state).device_window;
            let mut filter: BOOL = 0;

            filter = wined3d_filter_messages(window, (TRUE) as BOOL);
            MoveWindow(
                window,
                output_desc.desktop_rect.left as core::ffi::c_int,
                output_desc.desktop_rect.top as core::ffi::c_int,
                width as core::ffi::c_int,
                height as core::ffi::c_int,
                (TRUE) as BOOL,
            );
            ShowWindow(window, (SW_SHOW as i32));
            wined3d_filter_messages(window, filter);
        }
        (*state).d3d_mode = actual_mode;
    } else if (*state).desc.windowed == 0 {
        /* Fullscreen -> windowed switch */
        let mut window_rect: *mut RECT = core::ptr::null_mut();
        if (*state).desc.flags & WINED3D_SWAPCHAIN_RESTORE_WINDOW_RECT != 0 {
            window_rect = core::ptr::addr_of_mut!((*state).original_window_rect);
        }
        wined3d_swapchain_state_restore_from_fullscreen(state, (*state).device_window, window_rect);
    }

    (*state).desc.output = (*swapchain_desc).output;
    (*state).desc.windowed = (*swapchain_desc).windowed;

    if windowed != (*state).desc.windowed {
        ((*(*(*state).parent).ops).windowed_state_changed.unwrap_unchecked())(
            (*state).parent,
            (*state).desc.windowed,
        );
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_is_windowed(state: *const wined3d_swapchain_state) -> BOOL {
    return (*state).desc.windowed;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_get_size(
    mut state: *const wined3d_swapchain_state,
    mut width: *mut ::core::ffi::c_uint,
    mut height: *mut ::core::ffi::c_uint,
) {
    WINE_TRACE!("state %p.\n", state);
    *width = (*state).desc.backbuffer_width;
    *height = (*state).desc.backbuffer_height;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_destroy(mut state: *mut wined3d_swapchain_state) {
    wined3d_swapchain_state_cleanup(state);
    free(state as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_swapchain_state_create(
    desc: *const wined3d_swapchain_desc,
    window: HWND,
    wined3d: *mut wined3d,
    state_parent: *mut wined3d_swapchain_state_parent,
    state: *mut *mut wined3d_swapchain_state,
) -> HRESULT {
    let mut s: *mut wined3d_swapchain_state = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    s = calloc((1 as usize) as _, (core::mem::size_of::<wined3d_swapchain_state>() as usize) as _) as *mut wined3d_swapchain_state;
    if s.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_swapchain_state_init(s, desc, window, wined3d, state_parent);
    if hr < 0 {
        free(s as *mut core::ffi::c_void);
        return hr;
    }

    *state = s;

    return hr;
}
