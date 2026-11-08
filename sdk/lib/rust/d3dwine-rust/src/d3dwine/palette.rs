use crate::d3dwine::cs::wined3d_cs_destroy_object;
use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_incref(palette: *mut wined3d_palette) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*palette).ref_) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", palette, refcount);
    return refcount as ULONG;
}

unsafe extern "C" fn wined3d_palette_destroy_object(object: *mut core::ffi::c_void) {
    // TRACE("object %p.\n", object);
    free(object);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_decref(palette: *mut wined3d_palette) -> ULONG {
    let refcount: u32 = InterlockedDecrement(&raw mut (*palette).ref_ as *mut i32) as u32;

    if refcount == 0 {
        wined3d_mutex_lock();
        wined3d_cs_destroy_object(
            (*(*palette).device).cs as *mut wined3d_cs,
            Some(wined3d_palette_destroy_object as unsafe extern "C" fn(*mut core::ffi::c_void)),
            palette as *mut core::ffi::c_void,
        );
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_get_entries(
    palette: *const wined3d_palette,
    flags: uint32_t,
    start: ::core::ffi::c_uint,
    count: ::core::ffi::c_uint,
    entries: *mut PALETTEENTRY,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;
    WINE_TRACE!("palette %p, flags %#x, start %u, count %u, entries %p.\n", palette, flags, start, count, entries);
    if flags != 0 {
        return WINED3DERR_INVALIDCALL;
    }
    if !wined3d_bound_range(start, count, (*palette).size) {
        return WINED3DERR_INVALIDCALL;
    }
    if (*palette).flags & (WINED3D_PALETTE_8BIT_ENTRIES as uint32_t) != 0 {
        let mut entry: *mut BYTE = entries as *mut BYTE;
        i = start;
        while i < count.wrapping_add(start) {
            let fresh16 = entry;
            entry = entry.offset(1);
            *fresh16 = (*palette).colors[i as usize].rgbRed;
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as ::core::ffi::c_uint;
        while i < count {
            (*entries.offset(i as isize)).peRed =
                (*palette).colors[i.wrapping_add(start) as usize].rgbRed;
            (*entries.offset(i as isize)).peGreen =
                (*palette).colors[i.wrapping_add(start) as usize].rgbGreen;
            (*entries.offset(i as isize)).peBlue =
                (*palette).colors[i.wrapping_add(start) as usize].rgbBlue;
            (*entries.offset(i as isize)).peFlags =
                (*palette).colors[i.wrapping_add(start) as usize].rgbReserved;
            i = i.wrapping_add(1);
        }
    }
    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_apply_to_dc(palette: *const wined3d_palette, dc: HDC) {
    if SetDIBColorTable(dc, 0 as u32, 256 as u32, core::ptr::addr_of!((*palette).colors) as *const RGBQUAD) != 256 {
        ERR!("Failed to set DIB color table.\n");
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_set_entries(
    mut palette: *mut wined3d_palette,
    mut flags: uint32_t,
    mut start: ::core::ffi::c_uint,
    mut count: ::core::ffi::c_uint,
    mut entries: *const PALETTEENTRY,
) -> HRESULT {
    let mut i: ::core::ffi::c_uint = 0;

    wined3d_cs_finish((*(*palette).device).cs as *mut wined3d_cs, WINED3D_CS_QUEUE_DEFAULT);

    if ((*palette).flags & (WINED3D_PALETTE_8BIT_ENTRIES as uint32_t)) != 0 {
        let mut entry: *const BYTE = entries as *const BYTE;

        i = start;
        while i < count.wrapping_add(start) {
            let fresh15 = entry;
            entry = entry.offset(1);
            (*palette).colors[i as usize].rgbRed = *fresh15;
            i = i.wrapping_add(1);
        }
    } else {
        i = 0 as ::core::ffi::c_uint;
        while i < count {
            (*palette).colors[i.wrapping_add(start) as usize].rgbRed =
                (*entries.offset(i as isize)).peRed;
            (*palette).colors[i.wrapping_add(start) as usize].rgbGreen =
                (*entries.offset(i as isize)).peGreen;
            (*palette).colors[i.wrapping_add(start) as usize].rgbBlue =
                (*entries.offset(i as isize)).peBlue;
            (*palette).colors[i.wrapping_add(start) as usize].rgbReserved =
                (*entries.offset(i as isize)).peFlags;
            i = i.wrapping_add(1);
        }

        if ((*palette).flags & (WINED3D_PALETTE_ALLOW_256 as uint32_t)) == 0 {
            (*palette).colors[0 as usize].rgbRed = 0 as BYTE;
            (*palette).colors[0 as usize].rgbGreen = 0 as BYTE;
            (*palette).colors[0 as usize].rgbBlue = 0 as BYTE;

            (*palette).colors[255 as usize].rgbRed = 255 as BYTE;
            (*palette).colors[255 as usize].rgbGreen = 255 as BYTE;
            (*palette).colors[255 as usize].rgbBlue = 255 as BYTE;
        }
    }

    return WINED3D_OK;
}

unsafe extern "C" fn wined3d_palette_init(palette: *mut wined3d_palette, device: *mut wined3d_device, flags: uint32_t, entry_count: ::core::ffi::c_uint, entries: *const PALETTEENTRY) -> HRESULT {
    let mut hr: HRESULT;

    (*palette).ref_ = 1;
    (*palette).device = device;
    (*palette).flags = flags;
    (*palette).size = entry_count;

    if {
        hr = wined3d_palette_set_entries(palette, 0, 0, entry_count, entries);
        hr
    } < 0
    {
        WARN!("Failed to set palette entries, hr %#lx.\n", hr);
        return hr;
    }

    return WINED3D_OK;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_palette_create(
    device: *mut wined3d_device,
    flags: uint32_t,
    entry_count: core::ffi::c_uint,
    entries: *const PALETTEENTRY,
    palette: *mut *mut wined3d_palette,
) -> HRESULT {
    let mut object: *mut wined3d_palette = core::ptr::null_mut();
    let mut hr: HRESULT;

    object = calloc(1, (core::mem::size_of::<wined3d_palette>()) as _) as *mut wined3d_palette;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    hr = wined3d_palette_init(object, device, flags, entry_count, entries);
    if hr < 0 {
        WARN!("Failed to initialize palette, hr %#lx.\n", hr);
        free(object as *mut core::ffi::c_void);
        return hr;
    }

    *palette = object;

    return WINED3D_OK;
}
