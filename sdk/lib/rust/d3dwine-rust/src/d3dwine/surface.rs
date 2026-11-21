use crate::d3dwine::context::context_restore;
use crate::d3dwine::utils::debug_box;
use crate::d3dwine::utils::debug_color;
use crate::d3dwine::utils::debug_d3dformat;
use crate::d3dwine::utils::debug_d3dtexturefiltertype;
use crate::d3dwine::utils::debug_d3dusage;
use crate::d3dwine::context_gl::wined3d_context_gl_apply_fbo_state_explicit;
use crate::d3dwine::context_gl::wined3d_context_gl_check_fbo_status;
use crate::d3dwine::context_gl::wined3d_context_gl_get_offscreen_gl_buffer;
use crate::d3dwine::utils::wined3d_debug_location;
use crate::d3dwine::utils::wined3d_format_calculate_pitch;
use crate::d3dwine::utils::wined3d_get_format;
use crate::d3dwine::view::wined3d_rendertarget_view_get_box;
use crate::d3dwine::resource::wined3d_resource_is_offscreen;
use crate::d3dwine::resource::wined3d_resource_memory_colour_fill;
use crate::d3dwine::swapchain::wined3d_swapchain_present;
use crate::d3dwine::texture::wined3d_texture_create;
use crate::d3dwine::texture::wined3d_texture_decref;
use crate::d3dwine::texture::wined3d_texture_download_from_texture;
use crate::d3dwine::texture::wined3d_texture_get_bo_address;
use crate::d3dwine::texture::wined3d_texture_get_gl_buffer;
use crate::d3dwine::texture::wined3d_texture_get_pitch;
use crate::d3dwine::texture::wined3d_texture_invalidate_location;
use crate::d3dwine::texture::wined3d_texture_load_location;
use crate::d3dwine::texture::wined3d_texture_prepare_location;
use crate::d3dwine::texture::wined3d_texture_upload_from_texture;
use crate::d3dwine::texture::wined3d_texture_validate_location;
use crate::prelude::*;
use core::ffi::c_void;


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut converters: [d3dfmt_converter_desc; 6] = unsafe {
    [
        d3dfmt_converter_desc {
            from: WINED3DFMT_R32_FLOAT,
            to: WINED3DFMT_R16_FLOAT,
            convert: Some(
                convert_r32_float_r16_float
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        d3dfmt_converter_desc {
            from: WINED3DFMT_B5G6R5_UNORM,
            to: WINED3DFMT_B8G8R8X8_UNORM,
            convert: Some(
                convert_r5g6b5_x8r8g8b8
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        d3dfmt_converter_desc {
            from: WINED3DFMT_B8G8R8A8_UNORM,
            to: WINED3DFMT_B8G8R8X8_UNORM,
            convert: Some(
                convert_a8r8g8b8_x8r8g8b8
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        d3dfmt_converter_desc {
            from: WINED3DFMT_B8G8R8X8_UNORM,
            to: WINED3DFMT_B8G8R8A8_UNORM,
            convert: Some(
                convert_a8r8g8b8_x8r8g8b8
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        d3dfmt_converter_desc {
            from: WINED3DFMT_YUY2,
            to: WINED3DFMT_B8G8R8X8_UNORM,
            convert: Some(
                convert_yuy2_x8r8g8b8
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
        d3dfmt_converter_desc {
            from: WINED3DFMT_YUY2,
            to: WINED3DFMT_B5G6R5_UNORM,
            convert: Some(
                convert_yuy2_r5g6b5
                    as unsafe extern "C" fn(
                        *const BYTE,
                        *mut BYTE,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                        ::core::ffi::c_uint,
                    ) -> (),
            ),
        },
    ]
};

static mut cpu_blitter_ops: wined3d_blitter_ops = unsafe {
    wined3d_blitter_ops {
        blitter_destroy: Some(
            cpu_blitter_destroy
                as unsafe extern "C" fn(*mut wined3d_blitter, *mut wined3d_context) -> (),
        ),
        blitter_clear: Some(
            cpu_blitter_clear
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    *mut wined3d_device,
                    ::core::ffi::c_uint,
                    *const wined3d_fb_state,
                    ::core::ffi::c_uint,
                    *const RECT,
                    *const RECT,
                    uint32_t,
                    *const wined3d_color,
                    ::core::ffi::c_float,
                    ::core::ffi::c_uint,
                ) -> (),
        ),
        blitter_blit: Some(
            cpu_blitter_blit
                as unsafe extern "C" fn(
                    *mut wined3d_blitter,
                    wined3d_blit_op,
                    *mut wined3d_context,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *mut wined3d_texture,
                    ::core::ffi::c_uint,
                    DWORD,
                    *const RECT,
                    *const wined3d_color_key,
                    wined3d_texture_filter_type,
                    *const wined3d_format,
                ) -> DWORD,
        ),
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(d3d): debug channel declaration macro, handled by the crate's DPRINT macros.
// WINE_DECLARE_DEBUG_CHANNEL(d3d_perf): debug channel declaration macro, handled by the crate's DPRINT macros.

#[repr(C)]
pub struct d3dfmt_converter_desc {
    pub from: wined3d_format_id,
    pub to: wined3d_format_id,
    pub convert: Option<unsafe extern "C" fn(
        src: *const BYTE,
        dst: *mut BYTE,
        pitch_in: u32,
        pitch_out: u32,
        w: u32,
        h: u32,
    )>,
}

// converters: defined exactly by c2rust's file-scope data -- not emitted here
// cpu_blitter_ops: defined exactly by c2rust's file-scope data -- not emitted here
unsafe extern "C" fn get_color_masks(format: *const wined3d_format, masks: *mut uint32_t) {
    *masks.add(0 as usize) = wined3d_mask_from_size((*format).red_size as u32) << (*format).red_offset;
    *masks.add(1 as usize) = wined3d_mask_from_size((*format).green_size as u32) << (*format).green_offset;
    *masks.add(2 as usize) = wined3d_mask_from_size((*format).blue_size as u32) << (*format).blue_offset;
}

unsafe extern "C" fn convert_r32_float_r16_float(src: *const BYTE, dst: *mut BYTE, pitch_in: u32, pitch_out: u32, w: u32, h: u32) {
    let mut dst_s: *mut u16 = core::ptr::null_mut();
    let mut src_f: *const f32 = core::ptr::null();
    let mut x: u32;
    let mut y: u32;
    WINE_TRACE!("Converting %ux%u pixels, pitches %u %u.\n", w, h, pitch_in, pitch_out);
    y = 0;
    while y < h {
        src_f = src.offset(y.wrapping_mul(pitch_in) as isize) as *const f32;
        dst_s = dst.offset(y.wrapping_mul(pitch_out) as isize) as *mut u16;
        x = 0;
        while x < w {
            *dst_s.offset(x as isize) = float_32_to_16(src_f.offset(x as isize));
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_r5g6b5_x8r8g8b8(
    src: *const BYTE,
    dst: *mut BYTE,
    pitch_in: ::core::ffi::c_uint,
    pitch_out: ::core::ffi::c_uint,
    w: ::core::ffi::c_uint,
    h: ::core::ffi::c_uint,
) {
    static convert_5to8: [::core::ffi::c_uchar; 32] = [
        0x00, 0x08, 0x10, 0x19, 0x21, 0x29, 0x31, 0x3a,
        0x42, 0x4a, 0x52, 0x5a, 0x63, 0x6b, 0x73, 0x7b,
        0x84, 0x8c, 0x94, 0x9c, 0xa5, 0xad, 0xb5, 0xbd,
        0xc5, 0xce, 0xd6, 0xde, 0xe6, 0xef, 0xf7, 0xff,
    ];
    static convert_6to8: [::core::ffi::c_uchar; 64] = [
        0x00, 0x04, 0x08, 0x0c, 0x10, 0x14, 0x18, 0x1c,
        0x20, 0x24, 0x28, 0x2d, 0x31, 0x35, 0x39, 0x3d,
        0x41, 0x45, 0x49, 0x4d, 0x51, 0x55, 0x59, 0x5d,
        0x61, 0x65, 0x69, 0x6d, 0x71, 0x75, 0x79, 0x7d,
        0x82, 0x86, 0x8a, 0x8e, 0x92, 0x96, 0x9a, 0x9e,
        0xa2, 0xa6, 0xaa, 0xae, 0xb2, 0xb6, 0xba, 0xbe,
        0xc2, 0xc6, 0xca, 0xce, 0xd2, 0xd7, 0xdb, 0xdf,
        0xe3, 0xe7, 0xeb, 0xef, 0xf3, 0xf7, 0xfb, 0xff,
    ];
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    WINE_TRACE!("Converting %ux%u pixels, pitches %u %u.\n", w, h, pitch_in, pitch_out);
    y = 0;
    while y < h {
        let mut src_line: *const WORD =
            src.offset(y.wrapping_mul(pitch_in) as isize) as *const WORD;
        let mut dst_line: *mut DWORD = dst.offset(y.wrapping_mul(pitch_out) as isize) as *mut DWORD;
        x = 0;
        while x < w {
            let mut pixel: WORD = *src_line.offset(x as isize);
            *dst_line.offset(x as isize) = (0xff000000 as ::core::ffi::c_uint
                | ((convert_5to8[((pixel as ::core::ffi::c_uint & 0xf800 as ::core::ffi::c_uint)
                    >> 11 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int)
                    << 16 as ::core::ffi::c_int) as ::core::ffi::c_uint
                | ((convert_6to8[((pixel as ::core::ffi::c_uint & 0x07e0 as ::core::ffi::c_uint)
                    >> 5 as ::core::ffi::c_int) as usize]
                    as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int) as ::core::ffi::c_uint
                | convert_5to8
                    [(pixel as ::core::ffi::c_uint & 0x001f as ::core::ffi::c_uint) as usize]
                    as ::core::ffi::c_uint) as DWORD;
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_a8r8g8b8_x8r8g8b8(
    src: *const BYTE,
    dst: *mut BYTE,
    pitch_in: ::core::ffi::c_uint,
    pitch_out: ::core::ffi::c_uint,
    w: ::core::ffi::c_uint,
    h: ::core::ffi::c_uint,
) {
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    WINE_TRACE!("Converting %ux%u pixels, pitches %u %u.\n", w, h, pitch_in, pitch_out);
    y = 0;
    while y < h {
        let src_line: *const DWORD =
            src.offset(y.wrapping_mul(pitch_in) as isize) as *const DWORD;
        let dst_line: *mut DWORD = dst.offset(y.wrapping_mul(pitch_out) as isize) as *mut DWORD;
        x = 0;
        while x < w {
            *dst_line.offset(x as isize) = 0xff000000 as DWORD
                | (*src_line.offset(x as isize) & 0xffffff as DWORD);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn cliptobyte(mut x: ::core::ffi::c_int) -> BYTE {
    return (if x < 0 {
        0
    } else if x > 255 {
        255
    } else {
        x
    }) as BYTE;
}

unsafe extern "C" fn convert_yuy2_x8r8g8b8(
    mut src: *const BYTE,
    mut dst: *mut BYTE,
    mut pitch_in: ::core::ffi::c_uint,
    mut pitch_out: ::core::ffi::c_uint,
    mut w: ::core::ffi::c_uint,
    mut h: ::core::ffi::c_uint,
) {
    let mut c2: ::core::ffi::c_int = 0;
    let mut d: ::core::ffi::c_int = 0;
    let mut e: ::core::ffi::c_int = 0;
    let mut r2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    WINE_TRACE!("Converting %ux%u pixels, pitches %u %u.\n", w, h, pitch_in, pitch_out);
    y = 0 as ::core::ffi::c_uint;
    while y < h {
        let mut src_line: *const BYTE = src.offset(y.wrapping_mul(pitch_in) as isize);
        let mut dst_line: *mut DWORD = dst.offset(y.wrapping_mul(pitch_out) as isize) as *mut DWORD;
        x = 0 as ::core::ffi::c_uint;
        while x < w {
            if (x & 1 as ::core::ffi::c_uint) == 0 {
                d = *src_line.offset(1 as isize) as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int;
                e = *src_line.offset(3 as isize) as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int;
                r2 = 409 as ::core::ffi::c_int * e
                    + 128 as ::core::ffi::c_int;
                g2 = -(100 as ::core::ffi::c_int) * d
                    - 208 as ::core::ffi::c_int * e
                    + 128 as ::core::ffi::c_int;
                b2 = 516 as ::core::ffi::c_int * d
                    + 128 as ::core::ffi::c_int;
            }
            c2 = 298 as ::core::ffi::c_int
                * (*src_line.offset(0 as isize) as ::core::ffi::c_int
                    - 16 as ::core::ffi::c_int);
            *dst_line.offset(x as isize) = ((0xff000000 as DWORD)
                | ((cliptobyte((c2 + r2) >> 8) as DWORD)
                    << 16)
                | ((cliptobyte((c2 + g2) >> 8) as DWORD)
                    << 8)
                | (cliptobyte((c2 + b2) >> 8) as DWORD))
                as DWORD;
            src_line = src_line.offset(2 as isize);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn convert_yuy2_r5g6b5(mut src: *const BYTE, mut dst: *mut BYTE, mut pitch_in: ::core::ffi::c_uint, mut pitch_out: ::core::ffi::c_uint, mut w: ::core::ffi::c_uint, mut h: ::core::ffi::c_uint) {
    let mut x: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut c2: ::core::ffi::c_int = 0;
    let mut d: ::core::ffi::c_int = 0;
    let mut e: ::core::ffi::c_int = 0;
    let mut r2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut g2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut b2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    TRACE!("Converting %ux%u pixels, pitches %u %u\n", w, h, pitch_in, pitch_out);
    y = 0 as ::core::ffi::c_uint;
    while y < h {
        let mut src_line: *const BYTE = src.offset(y.wrapping_mul(pitch_in) as isize);
        let mut dst_line: *mut WORD = dst.offset(y.wrapping_mul(pitch_out) as isize) as *mut WORD;
        x = 0 as ::core::ffi::c_uint;
        while x < w {
            if x & (1 as ::core::ffi::c_uint) == 0 {
                d = *src_line.offset(1 as isize) as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int;
                e = *src_line.offset(3 as isize) as ::core::ffi::c_int
                    - 128 as ::core::ffi::c_int;
                r2 = 409 as ::core::ffi::c_int * e
                    + 128 as ::core::ffi::c_int;
                g2 = -(100 as ::core::ffi::c_int) * d
                    - 208 as ::core::ffi::c_int * e
                    + 128 as ::core::ffi::c_int;
                b2 = 516 as ::core::ffi::c_int * d
                    + 128 as ::core::ffi::c_int;
            }
            c2 = 298 as ::core::ffi::c_int
                * (*src_line.offset(0 as isize) as ::core::ffi::c_int
                    - 16 as ::core::ffi::c_int);
            *dst_line.offset(x as isize) = ((cliptobyte(
                (c2 + r2) >> 8 as ::core::ffi::c_int,
            ) as ::core::ffi::c_int
                >> 3 as ::core::ffi::c_int)
                << 11 as ::core::ffi::c_int
                | (cliptobyte(
                    (c2 + g2) >> 8 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    >> 2 as ::core::ffi::c_int)
                    << 5 as ::core::ffi::c_int
                | cliptobyte(
                    (c2 + b2) >> 8 as ::core::ffi::c_int,
                ) as ::core::ffi::c_int
                    >> 3 as ::core::ffi::c_int) as WORD;
            src_line = src_line.offset(2 as isize);
            x = x.wrapping_add(1);
        }
        y = y.wrapping_add(1);
    }
}

unsafe extern "C" fn find_converter(
    from: wined3d_format_id,
    to: wined3d_format_id,
) -> *const d3dfmt_converter_desc {
    let mut i: ::core::ffi::c_uint = 0;

    while (i as usize) < (ARRAY_SIZE!(converters) as usize) {
        if converters[i as usize].from as ::core::ffi::c_uint == from as ::core::ffi::c_uint
            && converters[i as usize].to as ::core::ffi::c_uint == to as ::core::ffi::c_uint
        {
            return (core::ptr::addr_of!(converters) as *const d3dfmt_converter_desc)
                .add(i as usize);
        }
        i += 1;
    }

    return core::ptr::null::<d3dfmt_converter_desc>();
}

unsafe extern "C" fn surface_convert_format(
    mut src_texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut dst_format: *const wined3d_format,
) -> *mut wined3d_texture {
    let mut texture_level: ::core::ffi::c_uint = sub_resource_idx % (*src_texture).level_count;
    let mut src_format: *const wined3d_format = (*src_texture).resource.format;
    let mut device: *mut wined3d_device = (*src_texture).resource.device;
    let mut conv: *const d3dfmt_converter_desc = core::ptr::null::<d3dfmt_converter_desc>();
    let mut src_row_pitch: ::core::ffi::c_uint = 0;
    let mut src_slice_pitch: ::core::ffi::c_uint = 0;
    let mut dst_texture: *mut wined3d_texture = core::ptr::null_mut::<wined3d_texture>();
    let mut src_data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut::<wined3d_bo>(),
        addr: core::ptr::null_mut::<BYTE>(),
    };
    let mut desc: wined3d_resource_desc = wined3d_resource_desc {
        resource_type: WINED3D_RTYPE_NONE,
        format: WINED3DFMT_UNKNOWN,
        multisample_type: WINED3D_MULTISAMPLE_NONE,
        multisample_quality: 0,
        usage: 0,
        bind_flags: 0,
        access: 0,
        width: 0,
        height: 0,
        depth: 0,
        size: 0,
    };
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut map_binding: DWORD = 0;

    conv = find_converter((*src_format).id, (*dst_format).id);
    if conv.is_null()
        && (((*(*device).wined3d).flags & WINED3D_NO3D != 0)
            || is_identity_fixup((*src_format).color_fixup) == 0
            || (*src_format).conv_byte_count != 0
            || is_identity_fixup((*dst_format).color_fixup) == 0
            || (*dst_format).conv_byte_count != 0
            || (((*src_format).attrs & WINED3D_FORMAT_ATTR_COMPRESSED) != 0
                && (*src_format).decompress.is_none()))
    {
        FIXME!(
            "Cannot find a conversion function from format %s to %s.\n",
            debug_d3dformat((*src_format).id),
            debug_d3dformat((*dst_format).id)
        );
        return core::ptr::null_mut::<wined3d_texture>();
    }

    /* FIXME: Multisampled conversion? */
    desc.resource_type = WINED3D_RTYPE_TEXTURE_2D;
    desc.format = (*dst_format).id;
    desc.multisample_type = WINED3D_MULTISAMPLE_NONE;
    desc.multisample_quality = 0;
    desc.usage = WINED3DUSAGE_SCRATCH | WINED3DUSAGE_CS;
    desc.bind_flags = 0;
    desc.access = WINED3D_RESOURCE_ACCESS_CPU | WINED3D_RESOURCE_ACCESS_MAP_R | WINED3D_RESOURCE_ACCESS_MAP_W;
    desc.width = wined3d_texture_get_level_width(src_texture, texture_level);
    desc.height = wined3d_texture_get_level_height(src_texture, texture_level);
    desc.depth = 1;
    desc.size = 0;
    if (wined3d_texture_create(
        device,
        core::ptr::addr_of_mut!(desc),
        1,
        1,
        WINED3D_TEXTURE_CREATE_DISCARD,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::addr_of!(wined3d_null_parent_ops),
        core::ptr::addr_of_mut!(dst_texture),
    ) < 0)
    {
        ERR!("Failed to create a destination texture for conversion.\n");
        return core::ptr::null_mut::<wined3d_texture>();
    }

    context = context_acquire(device, core::ptr::null_mut(), 0);

    map_binding = ((*src_texture).resource.map_binding) as _;
    if wined3d_texture_load_location(src_texture, sub_resource_idx, context, map_binding) == 0 {
        ERR!(
            "Failed to load the source sub-resource into %s.\n",
            wined3d_debug_location(map_binding)
        );
    }
    wined3d_texture_get_pitch(
        src_texture,
        texture_level,
        core::ptr::addr_of_mut!(src_row_pitch),
        core::ptr::addr_of_mut!(src_slice_pitch),
    );
    wined3d_texture_get_bo_address(
        src_texture,
        sub_resource_idx,
        core::ptr::addr_of_mut!(src_data),
        map_binding,
    );

    if !conv.is_null() {
        let mut dst_row_pitch: ::core::ffi::c_uint = 0;
        let mut dst_slice_pitch: ::core::ffi::c_uint = 0;
        let mut dst_data: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut::<wined3d_bo>(),
            addr: core::ptr::null_mut::<BYTE>(),
        };
        let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
        let mut src: *const BYTE = core::ptr::null::<BYTE>();
        let mut dst: *mut BYTE = core::ptr::null_mut::<BYTE>();

        map_binding = ((*dst_texture).resource.map_binding) as _;
        if wined3d_texture_load_location(dst_texture, 0, context, map_binding) == 0 {
            ERR!(
                "Failed to load the destination sub-resource into %s.\n",
                wined3d_debug_location(map_binding)
            );
        }
        wined3d_texture_get_pitch(
            dst_texture,
            0,
            core::ptr::addr_of_mut!(dst_row_pitch),
            core::ptr::addr_of_mut!(dst_slice_pitch),
        );
        wined3d_texture_get_bo_address(
            dst_texture,
            0,
            core::ptr::addr_of_mut!(dst_data),
            map_binding,
        );

        src = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(src_data),
            (*(*src_texture).sub_resources.offset(sub_resource_idx as isize)).size as usize,
            WINED3D_MAP_READ,
        ) as *const BYTE;
        dst = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(dst_data),
            (*(*dst_texture).sub_resources.offset(0 as isize)).size as usize,
            WINED3D_MAP_WRITE,
        ) as *mut BYTE;

        ((*conv).convert.unwrap_unchecked())(
            src,
            dst,
            src_row_pitch,
            dst_row_pitch,
            desc.width,
            desc.height,
        );

        range.offset = 0;
        range.size = (*(*dst_texture).sub_resources.offset(0 as isize)).size;
        wined3d_texture_invalidate_location(dst_texture, 0, !map_binding);
        wined3d_context_unmap_bo_address(
            context,
            core::ptr::addr_of_mut!(dst_data),
            1,
            core::ptr::addr_of_mut!(range),
        );
        wined3d_context_unmap_bo_address(
            context,
            core::ptr::addr_of_mut!(src_data),
            0,
            core::ptr::null::<wined3d_range>(),
        );
    } else {
        let mut src_box: wined3d_box = wined3d_box {
            left: 0,
            top: 0,
            right: desc.width,
            bottom: desc.height,
            front: 0,
            back: 1,
        };

        TRACE!("Using upload conversion.\n");
        wined3d_texture_prepare_location(
            dst_texture,
            0,
            context,
            WINED3D_LOCATION_TEXTURE_RGB,
        );
        (*(*dst_texture).texture_ops)
            .texture_upload_data
            .unwrap_unchecked()(
            context,
            wined3d_const_bo_address(core::ptr::addr_of!(src_data)),
            src_format,
            core::ptr::addr_of_mut!(src_box),
            src_row_pitch,
            src_slice_pitch,
            dst_texture,
            0,
            WINED3D_LOCATION_TEXTURE_RGB,
            0,
            0,
            0,
        );

        wined3d_texture_validate_location(dst_texture, 0, WINED3D_LOCATION_TEXTURE_RGB);
        wined3d_texture_invalidate_location(dst_texture, 0, !WINED3D_LOCATION_TEXTURE_RGB);
    }

    context_release(context);

    return dst_texture;
}
#[no_mangle]
pub unsafe extern "C" fn texture2d_read_from_framebuffer(
    mut texture: *mut wined3d_texture,
    mut sub_resource_idx: ::core::ffi::c_uint,
    mut context: *mut wined3d_context,
    mut src_location: DWORD,
    mut dst_location: DWORD,
) {
    let mut resource: *mut wined3d_resource = core::ptr::addr_of_mut!((*texture).resource);
    let mut device: *mut wined3d_device = (*resource).device;
    let mut format_gl: *const wined3d_format_gl = core::ptr::null();
    let mut restore_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut gl_info: *const wined3d_gl_info = core::ptr::null();
    let mut context_gl: *mut wined3d_context_gl = core::ptr::null_mut();
    let mut row_pitch: ::core::ffi::c_uint = 0;
    let mut slice_pitch: ::core::ffi::c_uint = 0;
    let mut width: ::core::ffi::c_uint = 0;
    let mut height: ::core::ffi::c_uint = 0;
    let mut level: ::core::ffi::c_uint = 0;
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };
    let mut restore_context: bool = false;
    let mut restore_idx: ::core::ffi::c_uint = 0;
    let mut row: *mut BYTE = core::ptr::null_mut();
    let mut top: *mut BYTE = core::ptr::null_mut();
    let mut bottom: *mut BYTE = core::ptr::null_mut();
    let mut src_is_upside_down: BOOL = 0;
    let mut mem: *mut BYTE = core::ptr::null_mut();
    let mut offset: *mut uint8_t = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint = 0;

    /* dst_location was already prepared by the caller. */
    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        core::ptr::addr_of_mut!(data),
        dst_location as uint32_t,
    );
    offset = data.addr as *mut uint8_t;

    restore_texture = (*context).current_rt.texture;
    restore_idx = (*context).current_rt.sub_resource_idx;
    if wined3d_resource_is_offscreen(resource) == 0
        && (restore_texture != texture || restore_idx != sub_resource_idx)
    {
        context = context_acquire(device, texture, sub_resource_idx);
        restore_context = true;
    }
    context_gl = wined3d_context_gl(context);
    gl_info = (*context_gl).gl_info;

    if (*(*resource).format).depth_size != 0 || (*(*resource).format).stencil_size != 0 {
        wined3d_context_gl_apply_fbo_state_explicit(
            context_gl,
            GL_READ_FRAMEBUFFER,
            core::ptr::null_mut(),
            0,
            resource,
            sub_resource_idx,
            src_location as uint32_t,
        );
    } else {
        wined3d_context_gl_apply_fbo_state_explicit(
            context_gl,
            GL_READ_FRAMEBUFFER,
            resource,
            sub_resource_idx,
            core::ptr::null_mut(),
            0,
            src_location as uint32_t,
        );
    }

    /* Select the correct read buffer, and give some debug output. There is no
     * need to keep track of the current read buffer or reset it, every part
     * of the code that reads pixels sets the read buffer as desired. */
    if src_location != WINED3D_LOCATION_DRAWABLE || wined3d_resource_is_offscreen(resource) != 0 {
        /* Mapping the primary render target which is not on a swapchain.
         * Read from the back buffer. */
        ((*gl_info).gl_ops.gl.p_glReadBuffer.unwrap_unchecked())(
            wined3d_context_gl_get_offscreen_gl_buffer(context_gl),
        );
        src_is_upside_down = (TRUE as i32);
    } else {
        /* Onscreen surfaces are always part of a swapchain */
        let mut buffer: GLenum = wined3d_texture_get_gl_buffer(texture);
        ((*gl_info).gl_ops.gl.p_glReadBuffer.unwrap_unchecked())(buffer);
        src_is_upside_down = (FALSE as i32);
    }
    checkGLcall!(gl_info, b"glReadBuffer\0".as_ptr() as *const i8);
    wined3d_context_gl_check_fbo_status(context_gl, GL_READ_FRAMEBUFFER);

    if !data.buffer_object.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            (*wined3d_bo_gl(data.buffer_object)).id,
        );
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const i8);
        offset = offset.offset((*data.buffer_object).buffer_offset as isize);
    } else {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            0,
        );
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const i8);
    }

    level = sub_resource_idx % (*texture).level_count;
    wined3d_texture_get_pitch(
        texture,
        level,
        core::ptr::addr_of_mut!(row_pitch),
        core::ptr::addr_of_mut!(slice_pitch),
    );
    format_gl = wined3d_format_gl((*resource).format);

    /* Setup pixel store pack state -- to glReadPixels into the correct place */
    ((*gl_info).gl_ops.gl.p_glPixelStorei.unwrap_unchecked())(
        GL_PACK_ROW_LENGTH,
        (row_pitch as UINT).wrapping_div((*format_gl).f.byte_count) as GLint,
    );
    checkGLcall!(gl_info, b"glPixelStorei\0".as_ptr() as *const i8);

    width = wined3d_texture_get_level_width(texture, level);
    height = wined3d_texture_get_level_height(texture, level);
    ((*gl_info).gl_ops.gl.p_glReadPixels.unwrap_unchecked())(
        0,
        0,
        width as GLsizei,
        height as GLsizei,
        (*format_gl).format as GLenum,
        (*format_gl).type_ as GLenum,
        offset as *mut ::core::ffi::c_void,
    );
    checkGLcall!(gl_info, b"glReadPixels\0".as_ptr() as *const i8);

    /* Reset previous pixel store pack state */
    ((*gl_info).gl_ops.gl.p_glPixelStorei.unwrap_unchecked())(
        GL_PACK_ROW_LENGTH,
        0,
    );
    checkGLcall!(gl_info, b"glPixelStorei\0".as_ptr() as *const i8);

    'error: {
        if src_is_upside_down == 0 {
            /* glReadPixels returns the image upside down, and there is no way to
             * prevent this. Flip the lines in software. */

            row = malloc(row_pitch as size_t) as *mut BYTE;
            if row.is_null() {
                break 'error;
            }

            if !data.buffer_object.is_null() {
                mem = ((*gl_info).gl_ops.ext.p_glMapBuffer.unwrap_unchecked())(
                    GL_PIXEL_PACK_BUFFER,
                    GL_READ_WRITE,
                ) as *mut BYTE;
                checkGLcall!(gl_info, b"glMapBuffer\0".as_ptr() as *const i8);
            }
            mem = mem.offset(offset as uintptr_t as isize);

            top = mem;
            bottom = mem.offset(
                row_pitch.wrapping_mul(height.wrapping_sub(1)) as isize,
            );
            i = 0;
            while i < height / 2 {
                memcpy(row as *mut ::core::ffi::c_void, top as *const ::core::ffi::c_void, row_pitch as size_t);
                memcpy(top as *mut ::core::ffi::c_void, bottom as *const ::core::ffi::c_void, row_pitch as size_t);
                memcpy(bottom as *mut ::core::ffi::c_void, row as *const ::core::ffi::c_void, row_pitch as size_t);
                top = top.offset(row_pitch as isize);
                bottom = bottom.offset(-(row_pitch as isize));
                i = i.wrapping_add(1);
            }
            free(row as *mut ::core::ffi::c_void);

            if !data.buffer_object.is_null() {
                ((*gl_info).gl_ops.ext.p_glUnmapBuffer.unwrap_unchecked())(GL_PIXEL_PACK_BUFFER);
            }
        }
    }

    /* error: */
    if !data.buffer_object.is_null() {
        ((*gl_info).gl_ops.ext.p_glBindBuffer.unwrap_unchecked())(
            GL_PIXEL_PACK_BUFFER,
            0,
        );
        wined3d_context_gl_reference_bo(context_gl, wined3d_bo_gl(data.buffer_object));
        checkGLcall!(gl_info, b"glBindBuffer\0".as_ptr() as *const i8);
    }

    if restore_context {
        context_restore(context, restore_texture, restore_idx);
    }
}

unsafe extern "C" fn cpu_blitter_destroy(
    blitter: *mut wined3d_blitter,
    context: *mut wined3d_context,
) {
    let mut next: *mut wined3d_blitter;
    next = (*blitter).next;
    if !next.is_null() {
        (*(*next).ops)
            .blitter_destroy
            .unwrap_unchecked()(next, context);
    }
    free(blitter as *mut core::ffi::c_void);
}

unsafe extern "C" fn surface_cpu_blt_compressed(
    src_data: *const BYTE,
    dst_data: *mut BYTE,
    src_pitch: UINT,
    dst_pitch: UINT,
    update_w: UINT,
    update_h: UINT,
    format: *const wined3d_format,
    flags: uint32_t,
    fx: *const wined3d_blt_fx,
) -> HRESULT {
    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct block {
        color: [WORD; 2],
        control_row: [BYTE; 4],
    }
    #[repr(C)]
    #[allow(non_camel_case_types)]
    struct block_0 {
        alpha_row: [WORD; 4],
        color: [WORD; 2],
        control_row: [BYTE; 4],
    }

    let mut row_block_count: UINT = 0;
    let mut src_row: *const BYTE = src_data;
    let mut dst_row: *mut BYTE = dst_data;
    let mut x: UINT = 0;
    let mut y: UINT = 0;

    src_row = src_data;
    dst_row = dst_data;

    row_block_count = (update_w
        .wrapping_add((*format).block_width)
        .wrapping_sub(1))
        .wrapping_div((*format).block_width);

    if flags == 0 {
        y = 0;
        while y < update_h {
            memcpy(
                dst_row as *mut ::core::ffi::c_void,
                src_row as *const ::core::ffi::c_void,
                row_block_count.wrapping_mul((*format).block_byte_count) as usize,
            );
            src_row = src_row.offset(src_pitch as isize);
            dst_row = dst_row.offset(dst_pitch as isize);
            y = y.wrapping_add((*format).block_height);
        }

        return WINED3D_OK;
    }

    if flags == WINED3D_BLT_FX && (*fx).fx == WINEDDBLTFX_MIRRORUPDOWN {
        src_row = src_row.offset(
            (update_h
                .wrapping_div((*format).block_height)
                .wrapping_sub(1)
                .wrapping_mul(src_pitch)) as isize,
        );

        match (*format).id as ::core::ffi::c_uint {
            WINED3DFMT_DXT1 => {
                y = 0;
                while y < update_h {
                    let s: *const block = src_row as *const block;
                    let d: *mut block = dst_row as *mut block;

                    x = 0;
                    while x < row_block_count {
                        (*d.offset(x as isize)).color[0] =
                            (*s.offset(x as isize)).color[0];
                        (*d.offset(x as isize)).color[1] =
                            (*s.offset(x as isize)).color[1];
                        (*d.offset(x as isize)).control_row[0] =
                            (*s.offset(x as isize)).control_row[3];
                        (*d.offset(x as isize)).control_row[1] =
                            (*s.offset(x as isize)).control_row[2];
                        (*d.offset(x as isize)).control_row[2] =
                            (*s.offset(x as isize)).control_row[1];
                        (*d.offset(x as isize)).control_row[3] =
                            (*s.offset(x as isize)).control_row[0];
                        x = x.wrapping_add(1);
                    }
                    src_row = src_row.offset(-(src_pitch as isize));
                    dst_row = dst_row.offset(dst_pitch as isize);
                    y = y.wrapping_add((*format).block_height);
                }
                return WINED3D_OK;
            }

            WINED3DFMT_DXT2 | WINED3DFMT_DXT3 => {
                y = 0;
                while y < update_h {
                    let s: *const block_0 = src_row as *const block_0;
                    let d: *mut block_0 = dst_row as *mut block_0;

                    x = 0;
                    while x < row_block_count {
                        (*d.offset(x as isize)).alpha_row[0] =
                            (*s.offset(x as isize)).alpha_row[3];
                        (*d.offset(x as isize)).alpha_row[1] =
                            (*s.offset(x as isize)).alpha_row[2];
                        (*d.offset(x as isize)).alpha_row[2] =
                            (*s.offset(x as isize)).alpha_row[1];
                        (*d.offset(x as isize)).alpha_row[3] =
                            (*s.offset(x as isize)).alpha_row[0];
                        (*d.offset(x as isize)).color[0] =
                            (*s.offset(x as isize)).color[0];
                        (*d.offset(x as isize)).color[1] =
                            (*s.offset(x as isize)).color[1];
                        (*d.offset(x as isize)).control_row[0] =
                            (*s.offset(x as isize)).control_row[3];
                        (*d.offset(x as isize)).control_row[1] =
                            (*s.offset(x as isize)).control_row[2];
                        (*d.offset(x as isize)).control_row[2] =
                            (*s.offset(x as isize)).control_row[1];
                        (*d.offset(x as isize)).control_row[3] =
                            (*s.offset(x as isize)).control_row[0];
                        x = x.wrapping_add(1);
                    }
                    src_row = src_row.offset(-(src_pitch as isize));
                    dst_row = dst_row.offset(dst_pitch as isize);
                    y = y.wrapping_add((*format).block_height);
                }
                return WINED3D_OK;
            }

            _ => {
                FIXME!(
                    "Compressed flip not implemented for format %s.\n",
                    debug_d3dformat((*format).id)
                );
                return E_NOTIMPL;
            }
        }
    }

    return E_NOTIMPL;
}

unsafe extern "C" fn surface_cpu_blt(
    mut dst_texture: *mut wined3d_texture,
    mut dst_sub_resource_idx: ::core::ffi::c_uint,
    mut dst_box: *const wined3d_box,
    mut src_texture: *mut wined3d_texture,
    mut src_sub_resource_idx: ::core::ffi::c_uint,
    mut src_box: *const wined3d_box,
    mut flags: uint32_t,
    mut fx: *const wined3d_blt_fx,
    mut filter: wined3d_texture_filter_type,
) -> HRESULT {
    let mut bpp: ::core::ffi::c_uint = 0;
    let mut src_height: ::core::ffi::c_uint = 0;
    let mut src_width: ::core::ffi::c_uint = 0;
    let mut dst_height: ::core::ffi::c_uint = 0;
    let mut dst_width: ::core::ffi::c_uint = 0;
    let mut row_byte_count: ::core::ffi::c_uint = 0;
    let mut device: *mut wined3d_device = (*dst_texture).resource.device;
    let mut src_format: *const wined3d_format = core::ptr::null();
    let mut dst_format: *const wined3d_format = core::ptr::null();
    let mut converted_texture: *mut wined3d_texture = core::ptr::null_mut();
    let mut src_data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };
    let mut dst_data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };
    let mut src_fmt_attrs: ::core::ffi::c_uint = 0;
    let mut dst_fmt_attrs: ::core::ffi::c_uint = 0;
    let mut dst_map: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut(),
    };
    let mut src_map: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut(),
    };
    let mut x: ::core::ffi::c_uint = 0;
    let mut sx: ::core::ffi::c_uint = 0;
    let mut xinc: ::core::ffi::c_uint = 0;
    let mut y: ::core::ffi::c_uint = 0;
    let mut sy: ::core::ffi::c_uint = 0;
    let mut yinc: ::core::ffi::c_uint = 0;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut dst_range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut texture_level: ::core::ffi::c_uint = 0;
    let mut hr: HRESULT = WINED3D_OK;
    let mut same_sub_resource: BOOL = 0;
    let mut upload: BOOL = (FALSE as i32);
    let mut map_binding: DWORD = 0;
    let mut sbase: *const BYTE = core::ptr::null();
    let mut sbuf: *const BYTE = core::ptr::null();
    let mut dbuf: *mut BYTE = core::ptr::null_mut();

    TRACE!("dst_texture %p, dst_sub_resource_idx %u, dst_box %s, src_texture %p, src_sub_resource_idx %u, src_box %s, flags %#x, fx %p, filter %s.\n",
        dst_texture, dst_sub_resource_idx, debug_box(dst_box), src_texture, src_sub_resource_idx,
        debug_box(src_box), flags, fx, debug_d3dtexturefiltertype(filter));

    context = context_acquire(device, core::ptr::null_mut(), 0);

    src_format = (*src_texture).resource.format;
    dst_format = (*dst_texture).resource.format;

    if wined3d_format_is_typeless(src_format) != 0
        && (*src_format).id as ::core::ffi::c_uint == (*dst_format).typeless_id as ::core::ffi::c_uint
    {
        src_format = dst_format;
    }
    if wined3d_format_is_typeless(dst_format) != 0
        && (*dst_format).id as ::core::ffi::c_uint == (*src_format).typeless_id as ::core::ffi::c_uint
    {
        dst_format = src_format;
    }

    src_height = (*src_box).bottom.wrapping_sub((*src_box).top) as ::core::ffi::c_uint;
    src_width = (*src_box).right.wrapping_sub((*src_box).left) as ::core::ffi::c_uint;
    dst_height = (*dst_box).bottom.wrapping_sub((*dst_box).top) as ::core::ffi::c_uint;
    dst_width = (*dst_box).right.wrapping_sub((*dst_box).left) as ::core::ffi::c_uint;

    dst_range.offset = 0;
    dst_range.size = (*(*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize)).size;
    if src_texture == dst_texture && src_sub_resource_idx == dst_sub_resource_idx {
        same_sub_resource = (TRUE as i32);

        map_binding = (*dst_texture).resource.map_binding as DWORD;
        texture_level = dst_sub_resource_idx % (*dst_texture).level_count;
        if wined3d_texture_load_location(
            dst_texture,
            dst_sub_resource_idx,
            context,
            map_binding as uint32_t,
        ) == 0
        {
            ERR!("Failed to load the destination sub-resource into %s.\n",
                wined3d_debug_location(map_binding as uint32_t));
        }
        wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !(map_binding as uint32_t));
        wined3d_texture_get_pitch(
            dst_texture,
            texture_level,
            core::ptr::addr_of_mut!(dst_map.row_pitch),
            core::ptr::addr_of_mut!(dst_map.slice_pitch),
        );
        wined3d_texture_get_bo_address(
            dst_texture,
            dst_sub_resource_idx,
            core::ptr::addr_of_mut!(dst_data),
            map_binding as uint32_t,
        );
        dst_map.data = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(dst_data),
            (*(*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize)).size as size_t,
            WINED3D_MAP_READ | WINED3D_MAP_WRITE,
        );

        src_map = dst_map;
    } else {
        same_sub_resource = (FALSE as i32);
        upload = (((*dst_format).attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0
            && (dst_width != src_width || dst_height != src_height)) as BOOL;

        if upload != 0 {
            dst_format = if ((*src_format).attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0 {
                wined3d_get_format(
                    (*device).adapter,
                    WINED3DFMT_B8G8R8A8_UNORM,
                    0,
                )
            } else {
                src_format
            };
        }

        if (flags & WINED3D_BLT_RAW) == 0
            && (*dst_format).id as ::core::ffi::c_uint != (*src_format).id as ::core::ffi::c_uint
        {
            converted_texture =
                surface_convert_format(src_texture, src_sub_resource_idx, dst_format);
            if converted_texture.is_null() {
                FIXME!("Cannot convert %s to %s.\n",
                    debug_d3dformat((*src_format).id),
                    debug_d3dformat((*dst_format).id));
                context_release(context);
                return WINED3DERR_NOTAVAILABLE;
            }
            src_texture = converted_texture;
            src_sub_resource_idx = 0;
            src_format = (*src_texture).resource.format;
        }

        map_binding = (*src_texture).resource.map_binding as DWORD;
        texture_level = src_sub_resource_idx % (*src_texture).level_count;
        if wined3d_texture_load_location(
            src_texture,
            src_sub_resource_idx,
            context,
            map_binding as uint32_t,
        ) == 0
        {
            ERR!("Failed to load the source sub-resource into %s.\n",
                wined3d_debug_location(map_binding as uint32_t));
        }
        wined3d_texture_get_pitch(
            src_texture,
            texture_level,
            core::ptr::addr_of_mut!(src_map.row_pitch),
            core::ptr::addr_of_mut!(src_map.slice_pitch),
        );
        wined3d_texture_get_bo_address(
            src_texture,
            src_sub_resource_idx,
            core::ptr::addr_of_mut!(src_data),
            map_binding as uint32_t,
        );
        src_map.data = wined3d_context_map_bo_address(
            context,
            core::ptr::addr_of_mut!(src_data),
            (*(*src_texture).sub_resources.offset(src_sub_resource_idx as isize)).size as size_t,
            WINED3D_MAP_READ,
        );

        if upload != 0 {
            wined3d_format_calculate_pitch(
                dst_format,
                1,
                (*dst_box).right as UINT,
                (*dst_box).bottom as UINT,
                core::ptr::addr_of_mut!(dst_map.row_pitch),
                core::ptr::addr_of_mut!(dst_map.slice_pitch),
            );
            dst_map.data = malloc(dst_map.slice_pitch as size_t);
        } else {
            map_binding = (*dst_texture).resource.map_binding as DWORD;
            texture_level = dst_sub_resource_idx % (*dst_texture).level_count;
            if wined3d_texture_load_location(
                dst_texture,
                dst_sub_resource_idx,
                context,
                map_binding as uint32_t,
            ) == 0
            {
                ERR!("Failed to load the destination sub-resource into %s.\n",
                    wined3d_debug_location(map_binding as uint32_t));
            }

            wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !(map_binding as uint32_t));
            wined3d_texture_get_pitch(
                dst_texture,
                texture_level,
                core::ptr::addr_of_mut!(dst_map.row_pitch),
                core::ptr::addr_of_mut!(dst_map.slice_pitch),
            );
            wined3d_texture_get_bo_address(
                dst_texture,
                dst_sub_resource_idx,
                core::ptr::addr_of_mut!(dst_data),
                map_binding as uint32_t,
            );
            dst_map.data = wined3d_context_map_bo_address(
                context,
                core::ptr::addr_of_mut!(dst_data),
                (*(*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize)).size as size_t,
                WINED3D_MAP_WRITE,
            );
        }
    }
    src_fmt_attrs = (*src_format).attrs as ::core::ffi::c_uint;
    dst_fmt_attrs = (*dst_format).attrs as ::core::ffi::c_uint;
    flags &= !(WINED3D_BLT_RAW);

    bpp = (*dst_format).byte_count as ::core::ffi::c_uint;
    row_byte_count = dst_width.wrapping_mul(bpp);

    sbase = (src_map.data as *mut BYTE)
        .offset(
            (*src_box)
                .top
                .wrapping_div((*src_format).block_height)
                .wrapping_mul(src_map.row_pitch) as isize,
        )
        .offset(
            (*src_box)
                .left
                .wrapping_div((*src_format).block_width)
                .wrapping_mul((*src_format).block_byte_count) as isize,
        );
    dbuf = (dst_map.data as *mut BYTE)
        .offset(
            (*dst_box)
                .top
                .wrapping_div((*dst_format).block_height)
                .wrapping_mul(dst_map.row_pitch) as isize,
        )
        .offset(
            (*dst_box)
                .left
                .wrapping_div((*dst_format).block_width)
                .wrapping_mul((*dst_format).block_byte_count) as isize,
        );

    'release: {
        'error: {
            if (src_fmt_attrs & dst_fmt_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0 {
                TRACE!("%s -> %s copy.\n",
                    debug_d3dformat((*src_format).id),
                    debug_d3dformat((*dst_format).id));

                if same_sub_resource != 0 {
                    FIXME!("Only plain blits supported on compressed surfaces.\n");
                    hr = E_NOTIMPL;
                    break 'release;
                }

                hr = surface_cpu_blt_compressed(
                    sbase,
                    dbuf,
                    src_map.row_pitch,
                    dst_map.row_pitch,
                    dst_width as UINT,
                    dst_height as UINT,
                    src_format,
                    flags,
                    fx,
                );
                break 'release;
            }

            if ((src_fmt_attrs | dst_fmt_attrs) & WINED3D_FORMAT_ATTR_HEIGHT_SCALE) != 0 {
                FIXME!("Unsupported blit between height-scaled formats (src %s, dst %s).\n",
                    debug_d3dformat((*src_format).id),
                    debug_d3dformat((*dst_format).id));
                hr = E_NOTIMPL;
                break 'release;
            }

            if (filter as ::core::ffi::c_uint) != WINED3D_TEXF_NONE
                && (filter as ::core::ffi::c_uint) != WINED3D_TEXF_POINT
                && (src_width != dst_width || src_height != dst_height)
            {
                /* Can happen when d3d9 apps do a StretchRect() call which isn't handled in GL. */
                FIXME!("Filter %s not supported in software blit.\n",
                    debug_d3dtexturefiltertype(filter));
            }

            xinc = (src_width << 16) / dst_width;
            yinc = (src_height << 16) / dst_height;

            if flags == 0 {
                /* No effects, we can cheat here. */
                if dst_width == src_width {
                    if dst_height == src_height {
                        /* No stretching in either direction. This needs to be as fast
                         * as possible. */
                        sbuf = sbase;

                        /* Check for overlapping surfaces. */
                        if same_sub_resource == 0
                            || (*dst_box).top < (*src_box).top
                            || (*dst_box).right <= (*src_box).left
                            || (*src_box).right <= (*dst_box).left
                        {
                            /* No overlap, or dst above src, so copy from top downwards. */
                            y = 0;
                            while y < dst_height {
                                memcpy(
                                    dbuf as *mut ::core::ffi::c_void,
                                    sbuf as *const ::core::ffi::c_void,
                                    row_byte_count as size_t,
                                );
                                sbuf = sbuf.offset(src_map.row_pitch as isize);
                                dbuf = dbuf.offset(dst_map.row_pitch as isize);
                                y = y.wrapping_add(1);
                            }
                        } else if (*dst_box).top > (*src_box).top {
                            /* Copy from bottom upwards. */
                            sbuf = sbuf.offset(
                                (src_map.row_pitch as ::core::ffi::c_uint)
                                    .wrapping_mul(dst_height) as isize,
                            );
                            dbuf = dbuf.offset(
                                (dst_map.row_pitch as ::core::ffi::c_uint)
                                    .wrapping_mul(dst_height) as isize,
                            );
                            y = 0;
                            while y < dst_height {
                                sbuf = sbuf.offset(-(src_map.row_pitch as isize));
                                dbuf = dbuf.offset(-(dst_map.row_pitch as isize));
                                memcpy(
                                    dbuf as *mut ::core::ffi::c_void,
                                    sbuf as *const ::core::ffi::c_void,
                                    row_byte_count as size_t,
                                );
                                y = y.wrapping_add(1);
                            }
                        } else {
                            /* Src and dst overlapping on the same line, use memmove. */
                            y = 0;
                            while y < dst_height {
                                memmove(
                                    dbuf as *mut ::core::ffi::c_void,
                                    sbuf as *const ::core::ffi::c_void,
                                    row_byte_count as size_t,
                                );
                                sbuf = sbuf.offset(src_map.row_pitch as isize);
                                dbuf = dbuf.offset(dst_map.row_pitch as isize);
                                y = y.wrapping_add(1);
                            }
                        }
                    } else {
                        /* Stretching in y direction only. */
                        sy = 0;
                        y = sy;
                        while y < dst_height {
                            sbuf = sbase.offset(
                                (sy >> 16).wrapping_mul(src_map.row_pitch)
                                    as isize,
                            );
                            memcpy(
                                dbuf as *mut ::core::ffi::c_void,
                                sbuf as *const ::core::ffi::c_void,
                                row_byte_count as size_t,
                            );
                            dbuf = dbuf.offset(dst_map.row_pitch as isize);
                            y = y.wrapping_add(1);
                            sy = sy.wrapping_add(yinc);
                        }
                    }
                } else {
                    /* Stretching in X direction. */
                    let mut last_sy: ::core::ffi::c_uint = !(0);
                    sy = 0;
                    y = sy;
                    while y < dst_height {
                        sbuf = sbase.offset(
                            (sy >> 16).wrapping_mul(src_map.row_pitch) as isize,
                        );

                        if (sy >> 16) == (last_sy >> 16) {
                            /* This source row is the same as last source row -
                             * Copy the already stretched row. */
                            memcpy(
                                dbuf as *mut ::core::ffi::c_void,
                                dbuf.offset(-(dst_map.row_pitch as isize))
                                    as *const ::core::ffi::c_void,
                                row_byte_count as size_t,
                            );
                        } else {
                            match bpp {
                                1 => {
                                    let s: *const BYTE = sbuf as *const BYTE;
                                    let d: *mut BYTE = dbuf as *mut BYTE;
                                    sx = 0;
                                    x = sx;
                                    while x < dst_width {
                                        *d.offset(x as isize) =
                                            *s.offset((sx >> 16) as isize);
                                        x = x.wrapping_add(1);
                                        sx = sx.wrapping_add(xinc);
                                    }
                                }
                                2 => {
                                    let s: *const WORD =
                                        sbuf as *const WORD;
                                    let d: *mut WORD =
                                        dbuf as *mut WORD;
                                    sx = 0;
                                    x = sx;
                                    while x < dst_width {
                                        *d.offset(x as isize) =
                                            *s.offset((sx >> 16) as isize);
                                        x = x.wrapping_add(1);
                                        sx = sx.wrapping_add(xinc);
                                    }
                                }
                                4 => {
                                    let s: *const DWORD = sbuf as *const DWORD;
                                    let d: *mut DWORD = dbuf as *mut DWORD;
                                    sx = 0;
                                    x = sx;
                                    while x < dst_width {
                                        *d.offset(x as isize) =
                                            *s.offset((sx >> 16) as isize);
                                        x = x.wrapping_add(1);
                                        sx = sx.wrapping_add(xinc);
                                    }
                                }
                                3 => {
                                    let mut s: *const BYTE = core::ptr::null();
                                    let mut d: *mut BYTE = dbuf;
                                    sx = 0;
                                    x = sx;
                                    while x < dst_width {
                                        let mut pixel: DWORD = 0;

                                        s = sbuf.offset(
                                            (3
                                                * (sx >> 16))
                                                as isize,
                                        );
                                        pixel = (*s.offset(0 as isize) as DWORD)
                                            | ((*s.offset(1 as isize) as DWORD)
                                                << 8)
                                            | ((*s.offset(2 as isize) as DWORD)
                                                << 16);
                                        *d.offset(0 as isize) =
                                            ((pixel) & 0xff) as BYTE;
                                        *d.offset(1 as isize) =
                                            ((pixel >> 8)
                                                & 0xff) as BYTE;
                                        *d.offset(2 as isize) =
                                            ((pixel >> 16)
                                                & 0xff) as BYTE;
                                        d = d.offset(3 as isize);
                                        x = x.wrapping_add(1);
                                        sx = sx.wrapping_add(xinc);
                                    }
                                }
                                _ => {
                                    FIXME!("Stretched blit not implemented for bpp %u.\n",
                                        bpp.wrapping_mul(8));
                                    hr = WINED3DERR_NOTAVAILABLE;
                                    break 'error;
                                }
                            }
                        }
                        dbuf = dbuf.offset(dst_map.row_pitch as isize);
                        last_sy = sy;
                        y = y.wrapping_add(1);
                        sy = sy.wrapping_add(yinc);
                    }
                }
            } else {
                let mut dstyinc: LONG = dst_map.row_pitch as LONG;
                let mut dstxinc: LONG = bpp as LONG;
                let mut keylow: DWORD = 0xffffffff;
                let mut keyhigh: DWORD = 0;
                let mut keymask: DWORD = 0xffffffff;
                let mut destkeylow: DWORD = 0x0;
                let mut destkeyhigh: DWORD = 0xffffffff;
                let mut destkeymask: DWORD = 0xffffffff;
                if (flags
                    & (WINED3D_BLT_SRC_CKEY
                        | WINED3D_BLT_DST_CKEY
                        | WINED3D_BLT_SRC_CKEY_OVERRIDE
                        | WINED3D_BLT_DST_CKEY_OVERRIDE))
                    != 0
                {
                    /* The color keying flags are checked for correctness in ddraw. */
                    if (flags & WINED3D_BLT_SRC_CKEY) != 0 {
                        keylow = (*src_texture)
                            .async_
                            .src_blt_color_key
                            .color_space_low_value as DWORD;
                        keyhigh = (*src_texture)
                            .async_
                            .src_blt_color_key
                            .color_space_high_value as DWORD;
                    } else if (flags & WINED3D_BLT_SRC_CKEY_OVERRIDE) != 0 {
                        keylow = (*fx).src_color_key.color_space_low_value as DWORD;
                        keyhigh = (*fx).src_color_key.color_space_high_value as DWORD;
                    }

                    if (flags & WINED3D_BLT_DST_CKEY) != 0 {
                        /* Destination color keys are taken from the source surface! */
                        destkeylow = (*src_texture)
                            .async_
                            .dst_blt_color_key
                            .color_space_low_value as DWORD;
                        destkeyhigh = (*src_texture)
                            .async_
                            .dst_blt_color_key
                            .color_space_high_value as DWORD;
                    } else if (flags & WINED3D_BLT_DST_CKEY_OVERRIDE) != 0 {
                        destkeylow = (*fx).dst_color_key.color_space_low_value as DWORD;
                        destkeyhigh = (*fx).dst_color_key.color_space_high_value as DWORD;
                    }

                    if bpp == 1 {
                        keymask = 0xff;
                    } else {
                        let mut masks: [uint32_t; 3] =
                            [0; 3];
                        get_color_masks(src_format, masks.as_mut_ptr());
                        keymask = (masks[0 as usize]
                            | masks[1 as usize]
                            | masks[2 as usize]) as DWORD;
                    }
                    flags &= !(WINED3D_BLT_SRC_CKEY
                        | WINED3D_BLT_DST_CKEY
                        | WINED3D_BLT_SRC_CKEY_OVERRIDE
                        | WINED3D_BLT_DST_CKEY_OVERRIDE);
                }

                if (flags & WINED3D_BLT_FX) != 0 {
                    let mut dTopLeft: *mut BYTE = core::ptr::null_mut();
                    let mut dTopRight: *mut BYTE = core::ptr::null_mut();
                    let mut dBottomLeft: *mut BYTE = core::ptr::null_mut();
                    let mut dBottomRight: *mut BYTE = core::ptr::null_mut();
                    let mut tmp: *mut BYTE = core::ptr::null_mut();
                    let mut tmpxy: LONG = 0;
                    dTopLeft = dbuf;
                    dTopRight = dbuf.offset(
                        dst_width
                            .wrapping_sub(1)
                            .wrapping_mul(bpp) as isize,
                    );
                    dBottomLeft = dTopLeft.offset(
                        (dst_height as UINT)
                            .wrapping_sub(1)
                            .wrapping_mul(dst_map.row_pitch) as isize,
                    );
                    dBottomRight = dBottomLeft.offset(
                        dst_width
                            .wrapping_sub(1)
                            .wrapping_mul(bpp) as isize,
                    );

                    if ((*fx).fx & WINEDDBLTFX_ARITHSTRETCHY) != 0 {
                        /* I don't think we need to do anything about this flag. */
                        WARN!("Nothing done for WINEDDBLTFX_ARITHSTRETCHY.\n");
                    }
                    if ((*fx).fx & WINEDDBLTFX_MIRRORLEFTRIGHT) != 0 {
                        tmp = dTopRight;
                        dTopRight = dTopLeft;
                        dTopLeft = tmp;
                        tmp = dBottomRight;
                        dBottomRight = dBottomLeft;
                        dBottomLeft = tmp;
                        dstxinc = dstxinc * -(1);
                    }
                    if ((*fx).fx & WINEDDBLTFX_MIRRORUPDOWN) != 0 {
                        tmp = dTopLeft;
                        dTopLeft = dBottomLeft;
                        dBottomLeft = tmp;
                        tmp = dTopRight;
                        dTopRight = dBottomRight;
                        dBottomRight = tmp;
                        dstyinc = dstyinc * -(1);
                    }
                    if ((*fx).fx & WINEDDBLTFX_NOTEARING) != 0 {
                        /* I don't think we need to do anything about this flag. */
                        WARN!("Nothing done for WINEDDBLTFX_NOTEARING.\n");
                    }
                    if ((*fx).fx & WINEDDBLTFX_ROTATE180) != 0 {
                        tmp = dBottomRight;
                        dBottomRight = dTopLeft;
                        dTopLeft = tmp;
                        tmp = dBottomLeft;
                        dBottomLeft = dTopRight;
                        dTopRight = tmp;
                        dstxinc = dstxinc * -(1);
                        dstyinc = dstyinc * -(1);
                    }
                    if ((*fx).fx & WINEDDBLTFX_ROTATE270) != 0 {
                        tmp = dTopLeft;
                        dTopLeft = dBottomLeft;
                        dBottomLeft = dBottomRight;
                        dBottomRight = dTopRight;
                        dTopRight = tmp;
                        tmpxy = dstxinc;
                        dstxinc = dstyinc;
                        dstyinc = tmpxy;
                        dstxinc = dstxinc * -(1);
                    }
                    if ((*fx).fx & WINEDDBLTFX_ROTATE90) != 0 {
                        tmp = dTopLeft;
                        dTopLeft = dTopRight;
                        dTopRight = dBottomRight;
                        dBottomRight = dBottomLeft;
                        dBottomLeft = tmp;
                        tmpxy = dstxinc;
                        dstxinc = dstyinc;
                        dstyinc = tmpxy;
                        dstyinc = dstyinc * -(1);
                    }
                    if ((*fx).fx & WINEDDBLTFX_ZBUFFERBASEDEST) != 0 {
                        /* I don't think we need to do anything about this flag. */
                        WARN!("Nothing done for WINEDDBLTFX_ZBUFFERBASEDEST.\n");
                    }
                    dbuf = dTopLeft;
                    flags &= !(WINED3D_BLT_FX);
                }

                match bpp {
                    1 => {
                        let mut s: *const BYTE = core::ptr::null();
                        let mut d: *mut BYTE = dbuf as *mut BYTE;
                        let mut dx: *mut BYTE = core::ptr::null_mut();
                        let mut tmp: BYTE = 0;
                        sy = 0;
                        y = sy;
                        while y < dst_height {
                            s = sbase.offset(
                                (sy >> 16).wrapping_mul(src_map.row_pitch) as isize,
                            );
                            dx = d;
                            sx = 0;
                            x = sx;
                            while x < dst_width {
                                tmp = *s.offset((sx >> 16) as isize);
                                if (((tmp as DWORD) & keymask) < keylow
                                    || ((tmp as DWORD) & keymask) > keyhigh)
                                    && (((*dx.offset(0) as DWORD) & destkeymask) >= destkeylow
                                        && ((*dx.offset(0) as DWORD) & destkeymask) <= destkeyhigh)
                                {
                                    *dx.offset(0) = tmp;
                                }
                                dx = dx.offset(dstxinc as isize);
                                x = x.wrapping_add(1);
                                sx = sx.wrapping_add(xinc);
                            }
                            d = d.offset(dstyinc as isize);
                            y = y.wrapping_add(1);
                            sy = sy.wrapping_add(yinc);
                        }
                    }
                    2 => {
                        let mut s: *const WORD = core::ptr::null();
                        let mut d: *mut WORD =
                            dbuf as *mut WORD;
                        let mut dx: *mut WORD = core::ptr::null_mut();
                        let mut tmp: WORD = 0;
                        sy = 0;
                        y = sy;
                        while y < dst_height {
                            s = sbase.offset(
                                (sy >> 16).wrapping_mul(src_map.row_pitch) as isize,
                            ) as *const WORD;
                            dx = d;
                            sx = 0;
                            x = sx;
                            while x < dst_width {
                                tmp = *s.offset((sx >> 16) as isize);
                                if (((tmp as DWORD) & keymask) < keylow
                                    || ((tmp as DWORD) & keymask) > keyhigh)
                                    && (((*dx.offset(0) as DWORD) & destkeymask) >= destkeylow
                                        && ((*dx.offset(0) as DWORD) & destkeymask) <= destkeyhigh)
                                {
                                    *dx.offset(0) = tmp;
                                }
                                dx = (dx as *mut BYTE).offset(dstxinc as isize)
                                    as *mut WORD;
                                x = x.wrapping_add(1);
                                sx = sx.wrapping_add(xinc);
                            }
                            d = (d as *mut BYTE).offset(dstyinc as isize)
                                as *mut WORD;
                            y = y.wrapping_add(1);
                            sy = sy.wrapping_add(yinc);
                        }
                    }
                    4 => {
                        let mut s: *const DWORD = core::ptr::null();
                        let mut d: *mut DWORD = dbuf as *mut DWORD;
                        let mut dx: *mut DWORD = core::ptr::null_mut();
                        let mut tmp: DWORD = 0;
                        sy = 0;
                        y = sy;
                        while y < dst_height {
                            s = sbase.offset(
                                (sy >> 16).wrapping_mul(src_map.row_pitch) as isize,
                            ) as *const DWORD;
                            dx = d;
                            sx = 0;
                            x = sx;
                            while x < dst_width {
                                tmp = *s.offset((sx >> 16) as isize);
                                if ((tmp as DWORD) & keymask < keylow
                                    || ((tmp as DWORD) & keymask) > keyhigh)
                                    && ((*dx.offset(0) & destkeymask) >= destkeylow
                                        && (*dx.offset(0) & destkeymask) <= destkeyhigh)
                                {
                                    *dx.offset(0) = tmp;
                                }
                                dx = (dx as *mut BYTE).offset(dstxinc as isize) as *mut DWORD;
                                x = x.wrapping_add(1);
                                sx = sx.wrapping_add(xinc);
                            }
                            d = (d as *mut BYTE).offset(dstyinc as isize) as *mut DWORD;
                            y = y.wrapping_add(1);
                            sy = sy.wrapping_add(yinc);
                        }
                    }
                    3 => {
                        let mut s: *const BYTE = core::ptr::null();
                        let mut d: *mut BYTE = dbuf;
                        let mut dx: *mut BYTE = core::ptr::null_mut();
                        sy = 0;
                        y = sy;
                        while y < dst_height {
                            sbuf = sbase.offset(
                                (sy >> 16).wrapping_mul(src_map.row_pitch)
                                    as isize,
                            );
                            dx = d;
                            sx = 0;
                            x = sx;
                            while x < dst_width {
                                let mut pixel: DWORD = 0;
                                let mut dpixel: DWORD = 0;
                                s = sbuf.offset(
                                    (3 * (sx >> 16))
                                        as isize,
                                );
                                pixel = (*s.offset(0 as isize) as DWORD)
                                    | ((*s.offset(1 as isize) as DWORD)
                                        << 8)
                                    | ((*s.offset(2 as isize) as DWORD)
                                        << 16);
                                dpixel = (*dx.offset(0 as isize) as DWORD)
                                    | ((*dx.offset(1 as isize) as DWORD)
                                        << 8)
                                    | ((*dx.offset(2 as isize) as DWORD)
                                        << 16);
                                if ((pixel & keymask) < keylow || (pixel & keymask) > keyhigh)
                                    && ((dpixel & keymask) >= destkeylow
                                        || (dpixel & keymask) <= keyhigh)
                                {
                                    *dx.offset(0 as isize) =
                                        ((pixel) & 0xff) as BYTE;
                                    *dx.offset(1 as isize) =
                                        ((pixel >> 8)
                                            & 0xff) as BYTE;
                                    *dx.offset(2 as isize) =
                                        ((pixel >> 16)
                                            & 0xff) as BYTE;
                                }
                                dx = dx.offset(dstxinc as isize);
                                x = x.wrapping_add(1);
                                sx = sx.wrapping_add(xinc);
                            }
                            d = d.offset(dstyinc as isize);
                            y = y.wrapping_add(1);
                            sy = sy.wrapping_add(yinc);
                        }
                    }
                    _ => {
                        FIXME!("%s color-keyed blit not implemented for bpp %u.\n",
                            if (flags & WINED3D_BLT_SRC_CKEY) != 0 {
                                b"Source\0".as_ptr() as *const ::core::ffi::c_char
                            } else {
                                b"Destination\0".as_ptr() as *const ::core::ffi::c_char
                            },
                            bpp.wrapping_mul(8));
                        hr = WINED3DERR_NOTAVAILABLE;
                        break 'error;
                    }
                }
            }
        }
        if flags != 0 {
            FIXME!("    Unsupported flags %#x.\n", flags);
        }
    }

    if upload != 0 && hr == WINED3D_OK {
        let mut data: wined3d_bo_address = wined3d_bo_address {
            buffer_object: core::ptr::null_mut(),
            addr: core::ptr::null_mut(),
        };
        data.buffer_object = core::ptr::null_mut();
        data.addr = dst_map.data as *mut BYTE;

        texture_level = dst_sub_resource_idx % (*dst_texture).level_count;

        wined3d_texture_prepare_location(dst_texture, texture_level, context, WINED3D_LOCATION_TEXTURE_RGB);
        (*(*dst_texture).texture_ops)
            .texture_upload_data
            .unwrap_unchecked()(
            context,
            wined3d_const_bo_address(core::ptr::addr_of_mut!(data)),
            dst_format,
            dst_box,
            dst_map.row_pitch as ::core::ffi::c_uint,
            dst_map.slice_pitch as ::core::ffi::c_uint,
            dst_texture,
            texture_level,
            WINED3D_LOCATION_TEXTURE_RGB,
            (*dst_box).left as ::core::ffi::c_uint,
            (*dst_box).top as ::core::ffi::c_uint,
            0,
        );

        wined3d_texture_validate_location(dst_texture, texture_level, WINED3D_LOCATION_TEXTURE_RGB);
        wined3d_texture_invalidate_location(dst_texture, texture_level, !(WINED3D_LOCATION_TEXTURE_RGB));
    }

    if upload != 0 {
        free(dst_map.data);
    } else {
        wined3d_context_unmap_bo_address(
            context,
            core::ptr::addr_of_mut!(dst_data),
            1,
            core::ptr::addr_of_mut!(dst_range),
        );
    }

    if same_sub_resource == 0 {
        wined3d_context_unmap_bo_address(
            context,
            core::ptr::addr_of_mut!(src_data),
            0,
            core::ptr::null(),
        );
    }
    if hr >= 0
        && !(*dst_texture).swapchain.is_null()
        && (*(*dst_texture).swapchain).front_buffer == dst_texture
    {
        SetRect(
            core::ptr::addr_of_mut!((*(*dst_texture).swapchain).front_buffer_update),
            (*dst_box).left as ::core::ffi::c_int,
            (*dst_box).top as ::core::ffi::c_int,
            (*dst_box).right as ::core::ffi::c_int,
            (*dst_box).bottom as ::core::ffi::c_int,
        );
        (*(*(*dst_texture).swapchain).swapchain_ops)
            .swapchain_frontbuffer_updated
            .unwrap_unchecked()((*dst_texture).swapchain);
    }
    if !converted_texture.is_null() {
        wined3d_texture_decref(converted_texture);
    }
    context_release(context);

    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn cpu_blitter_clear_texture(texture: *mut wined3d_texture, sub_resource_idx: c_uint, box_: *const wined3d_box, colour: *const wined3d_color) {
    let device: *mut wined3d_device = (*texture).resource.device;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut data: wined3d_bo_address = wined3d_bo_address {
        buffer_object: core::ptr::null_mut(),
        addr: core::ptr::null_mut(),
    };
    let mut level_box: wined3d_box = core::mem::zeroed();
    let mut map: wined3d_map_desc = wined3d_map_desc {
        row_pitch: 0,
        slice_pitch: 0,
        data: core::ptr::null_mut(),
    };
    let mut range: wined3d_range = wined3d_range { offset: 0, size: 0 };
    let mut full_subresource: bool = false;
    let mut level: c_uint = 0;
    let mut map_binding: DWORD = 0;

    DPRINT1!(
        "texture %p, sub_resource_idx %u, box %s, colour %s.\n",
        texture,
        sub_resource_idx,
        debug_box(box_),
        debug_color(colour)
    );

    if ((*texture).resource.format_attrs & WINED3D_FORMAT_ATTR_BLOCKS) != 0 {
        DPRINT1!(
            "Not implemented for format %s.\n",
            debug_d3dformat((*(*texture).resource.format).id)
        );
        return;
    }

    context = context_acquire(device, core::ptr::null_mut(), 0);

    level = sub_resource_idx % (*texture).level_count;
    wined3d_texture_get_level_box(texture, level, core::ptr::addr_of_mut!(level_box));
    full_subresource = memcmp(
        box_ as *const c_void,
        core::ptr::addr_of_mut!(level_box) as *const c_void,
        core::mem::size_of::<wined3d_box>() as size_t,
    ) == 0;

    map_binding = (*texture).resource.map_binding as DWORD;
    if wined3d_texture_load_location(texture, sub_resource_idx, context, map_binding) == 0 {
        ERR!(
            "Failed to load the sub-resource into %s.\n",
            wined3d_debug_location(map_binding)
        );
    }
    wined3d_texture_invalidate_location(texture, sub_resource_idx, !map_binding);
    wined3d_texture_get_pitch(
        texture,
        level,
        core::ptr::addr_of_mut!(map.row_pitch),
        core::ptr::addr_of_mut!(map.slice_pitch),
    );
    wined3d_texture_get_bo_address(
        texture,
        sub_resource_idx,
        core::ptr::addr_of_mut!(data),
        map_binding,
    );
    map.data = wined3d_context_map_bo_address(
        context,
        core::ptr::addr_of_mut!(data),
        (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size as size_t,
        WINED3D_MAP_WRITE,
    );
    range.offset = 0;
    range.size = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).size;

    wined3d_resource_memory_colour_fill(
        core::ptr::addr_of_mut!((*texture).resource),
        core::ptr::addr_of_mut!(map),
        colour,
        box_,
        full_subresource,
    );

    wined3d_context_unmap_bo_address(
        context,
        core::ptr::addr_of_mut!(data),
        1,
        core::ptr::addr_of_mut!(range),
    );
    context_release(context);
}

unsafe extern "C" fn cpu_blitter_clear_rtv(
    view: *mut wined3d_rendertarget_view,
    box_0: *const wined3d_box,
    colour: *const wined3d_color,
) {
    if (*(*view).format).id as ::core::ffi::c_uint
        != (*(*(*view).resource).format).id as ::core::ffi::c_uint
    {
        FIXME!(
            "View format %s doesn't match resource format %s.\n",
            debug_d3dformat((*(*view).format).id),
            debug_d3dformat((*(*(*view).resource).format).id)
        );
    }

    if (*(*view).resource).type_ as ::core::ffi::c_uint
        == WINED3D_RTYPE_BUFFER as ::core::ffi::c_uint
    {
        return;
    }

    cpu_blitter_clear_texture(
        texture_from_resource((*view).resource),
        (*view).sub_resource_idx,
        box_0,
        colour,
    );
}

unsafe extern "C" fn wined3d_box_intersect(
    ret: *mut wined3d_box,
    b1: *const wined3d_box,
    b2: *const wined3d_box,
) -> bool {
    wined3d_box_set(
        ret,
        max!((*b1).left, (*b2).left) as ::core::ffi::c_uint,
        max!((*b1).top, (*b2).top) as ::core::ffi::c_uint,
        min!((*b1).right, (*b2).right) as ::core::ffi::c_uint,
        min!((*b1).bottom, (*b2).bottom) as ::core::ffi::c_uint,
        max!((*b1).front, (*b2).front) as ::core::ffi::c_uint,
        min!((*b1).back, (*b2).back) as ::core::ffi::c_uint,
    );
    return (*ret).right > (*ret).left && (*ret).bottom > (*ret).top && (*ret).back > (*ret).front;
}

unsafe extern "C" fn cpu_blitter_clear(
    blitter: *mut wined3d_blitter,
    device: *mut wined3d_device,
    mut rt_count: ::core::ffi::c_uint,
    fb: *const wined3d_fb_state,
    mut rect_count: ::core::ffi::c_uint,
    mut clear_rects: *const RECT,
    draw_rect: *const RECT,
    flags: uint32_t,
    colour: *const wined3d_color,
    depth: ::core::ffi::c_float,
    stencil: ::core::ffi::c_uint,
) {
    let mut c: wined3d_color = wined3d_color {
        r: depth,
        g: 0.0f32,
        b: 0.0f32,
        a: 0.0f32,
    };
    let mut box_0: wined3d_box = core::mem::zeroed();
    let mut box_clip: wined3d_box = core::mem::zeroed();
    let mut box_view: wined3d_box = core::mem::zeroed();
    let mut view: *mut wined3d_rendertarget_view = core::ptr::null_mut();
    let mut i: ::core::ffi::c_uint;
    let mut j: ::core::ffi::c_uint;

    if rect_count == 0 {
        rect_count = 1 as ::core::ffi::c_uint;
        clear_rects = draw_rect;
    }

    i = 0 as ::core::ffi::c_uint;
    while i < rect_count {
        'body: {
            box_0.left = max!(
                (*clear_rects.offset(i as isize)).left,
                (*draw_rect).left,
            ) as UINT;
            box_0.top = max!(
                (*clear_rects.offset(i as isize)).top,
                (*draw_rect).top,
            ) as UINT;
            box_0.right = min!(
                (*clear_rects.offset(i as isize)).right,
                (*draw_rect).right,
            ) as UINT;
            box_0.bottom = min!(
                (*clear_rects.offset(i as isize)).bottom,
                (*draw_rect).bottom,
            ) as UINT;
            box_0.front = 0 as UINT;
            box_0.back = !(0 as UINT);

            if box_0.left >= box_0.right || box_0.top >= box_0.bottom {
                break 'body;
            }

            if (flags & WINED3DCLEAR_TARGET as uint32_t) != 0 {
                j = 0 as ::core::ffi::c_uint;
                while j < rt_count {
                    view = (*fb).render_targets[j as usize];
                    if !view.is_null() {
                        wined3d_rendertarget_view_get_box(view, core::ptr::addr_of_mut!(box_view));
                        if wined3d_box_intersect(
                            core::ptr::addr_of_mut!(box_clip),
                            core::ptr::addr_of_mut!(box_view),
                            core::ptr::addr_of_mut!(box_0),
                        ) {
                            cpu_blitter_clear_rtv(
                                view,
                                core::ptr::addr_of_mut!(box_clip),
                                colour,
                            );
                        }
                    }
                    j = j.wrapping_add(1);
                }
            }

            if (flags & (WINED3DCLEAR_ZBUFFER as uint32_t | WINED3DCLEAR_STENCIL as uint32_t)) != 0
                && {
                    view = (*fb).depth_stencil;
                    !view.is_null()
                }
            {
                if ((*(*view).format).depth_size != 0
                    && (flags & WINED3DCLEAR_ZBUFFER as uint32_t) == 0)
                    || ((*(*view).format).stencil_size != 0
                        && (flags & WINED3DCLEAR_STENCIL as uint32_t) == 0)
                {
                    FIXME!(
                        "Clearing %#x on %s.\n",
                        flags,
                        debug_d3dformat((*(*view).format).id)
                    );
                }

                wined3d_rendertarget_view_get_box(view, core::ptr::addr_of_mut!(box_view));
                if wined3d_box_intersect(
                    core::ptr::addr_of_mut!(box_clip),
                    core::ptr::addr_of_mut!(box_view),
                    core::ptr::addr_of_mut!(box_0),
                ) {
                    cpu_blitter_clear_rtv(
                        view,
                        core::ptr::addr_of_mut!(box_clip),
                        core::ptr::addr_of_mut!(c),
                    );
                }
            }
        }
        i = i.wrapping_add(1);
    }
}

unsafe extern "C" fn cpu_blitter_blit(
    blitter: *mut wined3d_blitter,
    op: wined3d_blit_op,
    context: *mut wined3d_context,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: ::core::ffi::c_uint,
    src_location: DWORD,
    src_rect: *const RECT,
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: ::core::ffi::c_uint,
    dst_location: DWORD,
    dst_rect: *const RECT,
    color_key: *const wined3d_color_key,
    filter: wined3d_texture_filter_type,
    resolve_format: *const wined3d_format,
) -> DWORD {
    let mut dst_box: wined3d_box = wined3d_box {
        left: (*dst_rect).left as UINT,
        top: (*dst_rect).top as UINT,
        right: (*dst_rect).right as UINT,
        bottom: (*dst_rect).bottom as UINT,
        front: 0,
        back: 1,
    };
    let mut src_box: wined3d_box = wined3d_box {
        left: (*src_rect).left as UINT,
        top: (*src_rect).top as UINT,
        right: (*src_rect).right as UINT,
        bottom: (*src_rect).bottom as UINT,
        front: 0,
        back: 1,
    };
    let mut fx: wined3d_blt_fx = core::mem::zeroed();
    let mut flags: DWORD = 0;

    memset(
        core::ptr::addr_of_mut!(fx) as *mut ::core::ffi::c_void,
        0,
        core::mem::size_of::<wined3d_blt_fx>() as size_t,
    );
    match op as ::core::ffi::c_uint {
        WINED3D_BLIT_OP_COLOR_BLIT | WINED3D_BLIT_OP_DEPTH_BLIT => {}
        WINED3D_BLIT_OP_RAW_BLIT => {
            flags |= WINED3D_BLT_RAW;
        }
        WINED3D_BLIT_OP_COLOR_BLIT_ALPHATEST => {
            flags |= WINED3D_BLT_ALPHA_TEST;
        }
        WINED3D_BLIT_OP_COLOR_BLIT_CKEY => {
            flags |= WINED3D_BLT_SRC_CKEY_OVERRIDE | WINED3D_BLT_FX;
            fx.src_color_key = *color_key;
        }
        _ => {
            FIXME!("Unhandled op %#x.\n", op);
        }
    }

    if (surface_cpu_blt(
        dst_texture,
        dst_sub_resource_idx,
        core::ptr::addr_of_mut!(dst_box),
        src_texture,
        src_sub_resource_idx,
        core::ptr::addr_of_mut!(src_box),
        flags,
        core::ptr::addr_of_mut!(fx),
        filter,
    )) < 0
    {
        ERR!("Failed to blit.\n");
    }
    wined3d_texture_load_location(dst_texture, dst_sub_resource_idx, context, dst_location);

    return dst_location
        | ((*(*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize)).locations
            & ((*dst_texture).resource.map_binding) as u32);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_cpu_blitter_create() -> *mut wined3d_blitter {
    let mut blitter: *mut wined3d_blitter = core::ptr::null_mut();

    blitter = malloc(core::mem::size_of::<wined3d_blitter>() as usize) as *mut wined3d_blitter;
    if blitter.is_null() {
        return core::ptr::null_mut();
    }

    (*blitter).ops = core::ptr::addr_of!(cpu_blitter_ops) as *const wined3d_blitter_ops;
    (*blitter).next = core::ptr::null_mut();

    return blitter;
}

unsafe extern "C" fn wined3d_is_colour_blit(blit_op: wined3d_blit_op) -> bool {
    match blit_op {
        WINED3D_BLIT_OP_COLOR_BLIT | WINED3D_BLIT_OP_COLOR_BLIT_ALPHATEST | WINED3D_BLIT_OP_COLOR_BLIT_CKEY => {
            return true;
        }
        _ => {
            return false;
        }
    }
}

unsafe extern "C" fn sub_resource_is_on_cpu(texture: *const wined3d_texture, sub_resource_idx: ::core::ffi::c_uint) -> bool {
    let mut locations: DWORD = (*(*texture).sub_resources.offset(sub_resource_idx as isize)).locations;

    if locations & ((WINED3D_LOCATION_BUFFER | WINED3D_LOCATION_SYSMEM) as DWORD) != 0 {
        return true;
    }

    if ((*texture).resource.access & (WINED3D_RESOURCE_ACCESS_GPU as DWORD)) == 0
        && (locations & (WINED3D_LOCATION_CLEARED as DWORD)) != 0
    {
        return true;
    }

    return false;
}
#[no_mangle]
pub unsafe extern "C" fn texture2d_blt(
    dst_texture: *mut wined3d_texture,
    dst_sub_resource_idx: u32,
    dst_box: *const wined3d_box,
    src_texture: *mut wined3d_texture,
    src_sub_resource_idx: u32,
    src_box: *const wined3d_box,
    mut flags: u32,
    fx: *const wined3d_blt_fx,
    filter: wined3d_texture_filter_type,
) -> HRESULT {
    let mut src_sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let mut dst_sub_resource: *mut wined3d_texture_sub_resource = core::ptr::null_mut();
    let device: *mut wined3d_device = (*dst_texture).resource.device;
    let mut src_swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut dst_swapchain: *mut wined3d_swapchain = core::ptr::null_mut();
    let mut scale: BOOL = 0;
    let mut convert: BOOL = 0;
    let mut resolve: BOOL = 0;
    let mut resolve_typeless: BOOL = (FALSE as i32);
    let mut resolve_format: *const wined3d_format = core::ptr::null();
    let mut colour_key: *const wined3d_color_key = core::ptr::null();
    let mut src_location: DWORD = 0;
    let mut dst_location: DWORD = 0;
    let mut valid_locations: DWORD = 0;
    let mut context: *mut wined3d_context = core::ptr::null_mut();
    let mut blit_op: wined3d_blit_op = WINED3D_BLIT_OP_COLOR_BLIT;
    let mut src_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut dst_rect: RECT = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    let mut src_ds: bool = false;
    let mut dst_ds: bool = false;

    let simple_blit: DWORD = (WINED3D_BLT_SRC_CKEY as DWORD)
        | (WINED3D_BLT_SRC_CKEY_OVERRIDE as DWORD)
        | (WINED3D_BLT_ALPHA_TEST as DWORD)
        | (WINED3D_BLT_RAW as DWORD);

    TRACE!("dst_texture %p, dst_sub_resource_idx %u, dst_box %s, src_texture %p, src_sub_resource_idx %u, src_box %s, flags %#x, fx %p, filter %s.\n",
        dst_texture, dst_sub_resource_idx, debug_box(dst_box), src_texture, src_sub_resource_idx,
        debug_box(src_box), flags, fx, debug_d3dtexturefiltertype(filter));
    TRACE!("Usage is %s.\n", debug_d3dusage((*dst_texture).resource.usage));

    if !fx.is_null() {
        TRACE!("fx %#x.\n", (*fx).fx);
        TRACE!("dst_color_key {0x%08x, 0x%08x}.\n",
            (*fx).dst_color_key.color_space_low_value,
            (*fx).dst_color_key.color_space_high_value);
        TRACE!("src_color_key {0x%08x, 0x%08x}.\n",
            (*fx).src_color_key.color_space_low_value,
            (*fx).src_color_key.color_space_high_value);
        TRACE!("resolve_format_id %s.\n", debug_d3dformat((*fx).resolve_format_id));

        if (*fx).resolve_format_id as u32 != WINED3DFMT_UNKNOWN as u32 {
            resolve_format = wined3d_get_format(
                (*device).adapter,
                (*fx).resolve_format_id,
                0 as u32,
            );
        }
    }

    dst_sub_resource = (*dst_texture).sub_resources.offset(dst_sub_resource_idx as isize);
    src_sub_resource = (*src_texture).sub_resources.offset(src_sub_resource_idx as isize);

    if (*src_sub_resource).locations & (WINED3D_LOCATION_DISCARDED as DWORD) != 0 {
        WARN!("Source sub-resource is discarded, nothing to do.\n");
        return WINED3D_OK;
    }

    SetRect(
        core::ptr::addr_of_mut!(src_rect),
        (*src_box).left as i32,
        (*src_box).top as i32,
        (*src_box).right as i32,
        (*src_box).bottom as i32,
    );
    SetRect(
        core::ptr::addr_of_mut!(dst_rect),
        (*dst_box).left as i32,
        (*dst_box).top as i32,
        (*dst_box).right as i32,
        (*dst_box).bottom as i32,
    );

    if fx.is_null() || (*fx).fx == 0 {
        flags &= !(WINED3D_BLT_FX as u32);
    }

    /* WINED3D_BLT_DO_NOT_WAIT appeared in DX7. */
    if flags & (WINED3D_BLT_DO_NOT_WAIT as u32) != 0 {
        static mut once: u32 = 0;

        let fresh15 = once;
        once = once.wrapping_add(1);
        if fresh15 == 0 {
            FIXME!("Can't handle WINED3D_BLT_DO_NOT_WAIT flag.\n");
        }
    }

    flags &= !((WINED3D_BLT_SYNCHRONOUS as u32)
        | (WINED3D_BLT_DO_NOT_WAIT as u32)
        | (WINED3D_BLT_WAIT as u32));

    'cpu: {
        if flags & !simple_blit != 0 {
            WINE_WARN_!(__wine_dbch_d3d_perf, "Using CPU fallback for complex blit (%#x).\n", flags);
            break 'cpu;
        }

        src_swapchain = (*src_texture).swapchain;
        dst_swapchain = (*dst_texture).swapchain;

        if !src_swapchain.is_null()
            && !dst_swapchain.is_null()
            && src_swapchain != dst_swapchain
            && src_texture == (*src_swapchain).front_buffer
        {
            /* TODO: We could support cross-swapchain blits by first downloading
             * the source to a texture. */
            FIXME!("Cross-swapchain blit not supported.\n");
            return WINED3DERR_INVALIDCALL;
        }

        scale = ((*src_box).right.wrapping_sub((*src_box).left)
            != (*dst_box).right.wrapping_sub((*dst_box).left)
            || (*src_box).bottom.wrapping_sub((*src_box).top)
                != (*dst_box).bottom.wrapping_sub((*dst_box).top)) as i32 as BOOL;
        convert = ((*(*src_texture).resource.format).id as u32
            != (*(*dst_texture).resource.format).id as u32) as i32 as BOOL;
        resolve = ((*src_texture).resource.multisample_type as u32
            != (*dst_texture).resource.multisample_type as u32) as i32 as BOOL;
        if resolve != 0 {
            resolve_typeless = ((wined3d_format_is_typeless((*src_texture).resource.format) != 0
                || wined3d_format_is_typeless((*dst_texture).resource.format) != 0)
                && (*(*src_texture).resource.format).typeless_id as u32
                    == (*(*dst_texture).resource.format).typeless_id as u32) as i32 as BOOL;
            if resolve_typeless != 0 && resolve_format.is_null() {
                WARN!("Resolve format for typeless resolve not specified.\n");
            }
        }

        dst_ds = (*(*dst_texture).resource.format).depth_size != 0
            || (*(*dst_texture).resource.format).stencil_size != 0;
        src_ds = (*(*src_texture).resource.format).depth_size != 0
            || (*(*src_texture).resource.format).stencil_size != 0;

        if src_ds || dst_ds {
            TRACE!("Depth/stencil blit.\n");

            if (*dst_texture).resource.access & (WINED3D_RESOURCE_ACCESS_GPU as u32) != 0 {
                dst_location = (*dst_texture).resource.draw_binding as DWORD;
            } else {
                dst_location = (*dst_texture).resource.map_binding as DWORD;
            }

            if (flags & (WINED3D_BLT_RAW as u32) != 0)
                || (scale == 0 && convert == 0 && resolve == 0)
            {
                blit_op = WINED3D_BLIT_OP_RAW_BLIT;
            } else {
                blit_op = WINED3D_BLIT_OP_DEPTH_BLIT;
            }

            context = context_acquire(device, dst_texture, dst_sub_resource_idx);
            valid_locations = (*(*(*device).blitter).ops)
                .blitter_blit
                .unwrap_unchecked()(
                (*device).blitter,
                blit_op,
                context,
                src_texture,
                src_sub_resource_idx,
                (*src_texture).resource.draw_binding as DWORD,
                core::ptr::addr_of_mut!(src_rect),
                dst_texture,
                dst_sub_resource_idx,
                dst_location,
                core::ptr::addr_of_mut!(dst_rect),
                core::ptr::null(),
                filter,
                resolve_format,
            );
            context_release(context);

            wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, valid_locations as u32);
            wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !(valid_locations as u32));

            return WINED3D_OK;
        }

        TRACE!("Colour blit.\n");

        /* In principle this would apply to depth blits as well, but we don't
         * implement those in the CPU blitter at the moment. */
        if (*dst_sub_resource).locations & (*dst_texture).resource.map_binding as DWORD != 0
            && (*src_sub_resource).locations & (*src_texture).resource.map_binding as DWORD != 0
        {
            if scale != 0 {
                TRACE!("Not doing sysmem blit because of scaling.\n");
            } else if convert != 0 {
                TRACE!("Not doing sysmem blit because of format conversion.\n");
            } else {
                break 'cpu;
            }
        }

        blit_op = WINED3D_BLIT_OP_COLOR_BLIT;
        if flags & (WINED3D_BLT_SRC_CKEY_OVERRIDE as u32) != 0 {
            colour_key = core::ptr::addr_of!((*fx).src_color_key);
            blit_op = WINED3D_BLIT_OP_COLOR_BLIT_CKEY;
        } else if flags & (WINED3D_BLT_SRC_CKEY as u32) != 0 {
            colour_key = core::ptr::addr_of!((*src_texture).async_.src_blt_color_key);
            blit_op = WINED3D_BLIT_OP_COLOR_BLIT_CKEY;
        } else if flags & (WINED3D_BLT_ALPHA_TEST as u32) != 0 {
            blit_op = WINED3D_BLIT_OP_COLOR_BLIT_ALPHATEST;
        } else if sub_resource_is_on_cpu(src_texture, src_sub_resource_idx) as i32 != 0
            && !(sub_resource_is_on_cpu(dst_texture, dst_sub_resource_idx) as i32 != 0)
            && (*dst_texture).resource.access & (WINED3D_RESOURCE_ACCESS_GPU as u32) != 0
        {
            /* Upload */
            if scale != 0 {
                TRACE!("Not doing upload because of scaling.\n");
            } else if convert != 0 {
                TRACE!("Not doing upload because of format conversion.\n");
            } else if (*(*dst_texture).resource.format).conv_byte_count != 0 {
                TRACE!("Not doing upload because the destination format needs conversion.\n");
            } else {
                wined3d_texture_upload_from_texture(
                    dst_texture,
                    dst_sub_resource_idx,
                    (*dst_box).left as u32,
                    (*dst_box).top as u32,
                    (*dst_box).front as u32,
                    src_texture,
                    src_sub_resource_idx,
                    src_box,
                );
                if wined3d_resource_is_offscreen(core::ptr::addr_of_mut!((*dst_texture).resource)) == 0 {
                    context = context_acquire(device, dst_texture, dst_sub_resource_idx);
                    wined3d_texture_load_location(
                        dst_texture,
                        dst_sub_resource_idx,
                        context,
                        (*dst_texture).resource.draw_binding as u32,
                    );
                    context_release(context);
                }
                return WINED3D_OK;
            }
        } else if !(sub_resource_is_on_cpu(src_texture, src_sub_resource_idx) as i32 != 0)
            && !((*dst_texture).resource.access & (WINED3D_RESOURCE_ACCESS_GPU as u32) != 0)
        {
            /* Download */
            if scale != 0 {
                TRACE!("Not doing download because of scaling.\n");
            } else if convert != 0 {
                TRACE!("Not doing download because of format conversion.\n");
            } else if (*(*src_texture).resource.format).conv_byte_count != 0 {
                TRACE!("Not doing download because the source format needs conversion.\n");
            } else if (*src_texture).flags & (WINED3D_TEXTURE_DOWNLOADABLE as u32) == 0 {
                TRACE!("Not doing download because texture is not downloadable.\n");
            } else if !(wined3d_texture_is_full_rect(
                src_texture,
                src_sub_resource_idx.wrapping_rem((*src_texture).level_count),
                core::ptr::addr_of_mut!(src_rect),
            ) as i32 != 0)
            {
                TRACE!("Not doing download because of partial download (src).\n");
            } else if !(wined3d_texture_is_full_rect(
                dst_texture,
                dst_sub_resource_idx.wrapping_rem((*dst_texture).level_count),
                core::ptr::addr_of_mut!(dst_rect),
            ) as i32 != 0)
            {
                TRACE!("Not doing download because of partial download (dst).\n");
            } else {
                wined3d_texture_download_from_texture(
                    dst_texture,
                    dst_sub_resource_idx,
                    src_texture,
                    src_sub_resource_idx,
                );
                return WINED3D_OK;
            }
        } else if !dst_swapchain.is_null()
            && !(*dst_swapchain).back_buffers.is_null()
            && dst_texture == (*dst_swapchain).front_buffer
            && src_texture
                == *(*dst_swapchain)
                    .back_buffers
                    .offset(0 as isize)
        {
            /* Use present for back -> front blits. The idea behind this is that
             * present is potentially faster than a blit, in particular when FBO
             * blits aren't available. Some ddraw applications like Half-Life and
             * Prince of Persia 3D use Blt() from the backbuffer to the
             * frontbuffer instead of doing a Flip(). D3d8 and d3d9 applications
             * can't blit directly to the frontbuffer. */
            let swap_effect: wined3d_swap_effect = (*dst_swapchain).state.desc.swap_effect;

            TRACE!("Using present for backbuffer -> frontbuffer blit.\n");

            /* Set the swap effect to COPY, we don't want the backbuffer to become
             * undefined. */
            (*dst_swapchain).state.desc.swap_effect = WINED3D_SWAP_EFFECT_COPY;
            wined3d_swapchain_present(
                dst_swapchain,
                core::ptr::null::<RECT>(),
                core::ptr::null::<RECT>(),
                (*dst_swapchain).win_handle,
                (*dst_swapchain).swap_interval,
                0 as u32,
            );
            (*dst_swapchain).state.desc.swap_effect = swap_effect;

            return WINED3D_OK;
        }

        if (flags & (WINED3D_BLT_RAW as u32) != 0)
            || (blit_op as u32 == WINED3D_BLIT_OP_COLOR_BLIT as u32
                && scale == 0
                && convert == 0
                && resolve == 0)
        {
            blit_op = WINED3D_BLIT_OP_RAW_BLIT;
        }

        context = context_acquire(device, dst_texture, dst_sub_resource_idx);

        if (*src_texture).resource.multisample_type as u32 != WINED3D_MULTISAMPLE_NONE as u32
            && resolve_typeless == 0
            && ((scale != 0 && (*(*context).d3d_info).scaled_resolve() == 0)
                || convert != 0
                || !(wined3d_is_colour_blit(blit_op) as i32 != 0))
        {
            src_location = WINED3D_LOCATION_RB_RESOLVED as DWORD;
        } else {
            src_location = (*src_texture).resource.draw_binding as DWORD;
        }

        if !((*dst_texture).resource.access & (WINED3D_RESOURCE_ACCESS_GPU as u32) != 0) {
            dst_location = (*dst_texture).resource.map_binding as DWORD;
        } else if (*dst_texture).resource.multisample_type as u32 != WINED3D_MULTISAMPLE_NONE as u32
            && (scale != 0 || convert != 0 || !(wined3d_is_colour_blit(blit_op) as i32 != 0))
        {
            dst_location = WINED3D_LOCATION_RB_RESOLVED as DWORD;
        } else {
            dst_location = (*dst_texture).resource.draw_binding as DWORD;
        }

        valid_locations = (*(*(*device).blitter).ops)
            .blitter_blit
            .unwrap_unchecked()(
            (*device).blitter,
            blit_op,
            context,
            src_texture,
            src_sub_resource_idx,
            src_location,
            core::ptr::addr_of_mut!(src_rect),
            dst_texture,
            dst_sub_resource_idx,
            dst_location,
            core::ptr::addr_of_mut!(dst_rect),
            colour_key,
            filter,
            resolve_format,
        );

        context_release(context);

        wined3d_texture_validate_location(dst_texture, dst_sub_resource_idx, valid_locations as u32);
        wined3d_texture_invalidate_location(dst_texture, dst_sub_resource_idx, !(valid_locations as u32));

        return WINED3D_OK;
    }

    return surface_cpu_blt(
        dst_texture,
        dst_sub_resource_idx,
        dst_box,
        src_texture,
        src_sub_resource_idx,
        src_box,
        flags,
        fx,
        filter,
    );
}
