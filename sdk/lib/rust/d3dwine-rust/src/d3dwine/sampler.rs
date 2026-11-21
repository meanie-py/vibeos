use crate::d3dwine::context_vk::vk_compare_op_from_wined3d;
use crate::d3dwine::cs::wined3d_cs_init_object;
use crate::d3dwine::ffp_gl::wined3d_gl_compare_func;
use crate::prelude::*;
use core::ffi::c_void;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_58 {
    pub wined3d_colour: wined3d_color,
    pub vk_colour: VkBorderColor,
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_incref(mut sampler: *mut wined3d_sampler) -> ULONG {
    let mut refcount: ::core::ffi::c_uint = InterlockedIncrement(
        core::ptr::addr_of_mut!((*sampler).refcount) as *mut ::core::ffi::c_long,
    ) as ::core::ffi::c_uint;
    WINE_TRACE!("%p increasing refcount to %u.\n", sampler, refcount);
    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_decref(sampler: *mut wined3d_sampler) -> ULONG {
    let refcount: ::core::ffi::c_uint =
        wined3d_atomic_decrement_mutex_lock(core::ptr::addr_of_mut!((*sampler).refcount) as *mut LONG)
            as ::core::ffi::c_uint;

    if refcount == 0 {
        (*(*sampler).parent_ops)
            .wined3d_object_destroyed
            .unwrap_unchecked()((*sampler).parent);
        (*(*(*(*sampler).device).adapter).adapter_ops)
            .adapter_destroy_sampler
            .unwrap_unchecked()(sampler);
        wined3d_mutex_unlock();
    }

    return refcount as ULONG;
}

#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_get_parent(
    sampler: *const wined3d_sampler,
) -> *mut core::ffi::c_void {
    // TRACE("sampler %p.\n", sampler);
    (*sampler).parent
}

unsafe extern "C" fn wined3d_sampler_init(
    sampler: *mut wined3d_sampler,
    device: *mut wined3d_device,
    desc: *const wined3d_sampler_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
) {
    TRACE!(
        "sampler %p, device %p, desc %p, parent %p, parent_ops %p.\n",
        sampler,
        device,
        desc,
        parent,
        parent_ops
    );
    TRACE!(
        "    Address modes: U %#x, V %#x, W %#x.\n",
        (*desc).address_u as u32,
        (*desc).address_v as u32,
        (*desc).address_w as u32
    );
    TRACE!(
        "    Border colour: {%.8e, %.8e, %.8e, %.8e}.\n",
        (*desc).border_color[0] as f64,
        (*desc).border_color[1] as f64,
        (*desc).border_color[2] as f64,
        (*desc).border_color[3] as f64
    );
    TRACE!(
        "    Filters: mag %#x, min %#x, mip %#x.\n",
        (*desc).mag_filter as u32,
        (*desc).min_filter as u32,
        (*desc).mip_filter as u32
    );
    TRACE!("    LOD bias: %.8e.\n", (*desc).lod_bias as f64);
    TRACE!("    Minimum LOD: %.8e.\n", (*desc).min_lod as f64);
    TRACE!("    Maximum LOD: %.8e.\n", (*desc).max_lod as f64);
    TRACE!("    Base mip level: %u.\n", (*desc).mip_base_level);
    TRACE!("    Maximum anisotropy: %u.\n", (*desc).max_anisotropy);
    TRACE!("    Comparison: %d.\n", (*desc).compare);
    TRACE!("    Comparison func: %#x.\n", (*desc).comparison_func as u32);
    TRACE!("    SRGB decode: %d.\n", (*desc).srgb_decode);
    (*sampler).refcount = 1 as LONG;
    (*sampler).device = device;
    (*sampler).parent = parent;
    (*sampler).parent_ops = parent_ops;
    (*sampler).desc = *desc;
}

unsafe extern "C" fn wined3d_sampler_gl_cs_init(object: *mut core::ffi::c_void) {
    let sampler_gl: *mut wined3d_sampler_gl = object as *mut wined3d_sampler_gl;
    let mut desc: *const wined3d_sampler_desc = core::ptr::null::<wined3d_sampler_desc>();
    let mut gl_info: *const wined3d_gl_info = core::ptr::null::<wined3d_gl_info>();
    let mut context: *mut wined3d_context = core::ptr::null_mut::<wined3d_context>();
    let mut name: GLuint = 0;

    TRACE!("sampler_gl %p.\n", sampler_gl);

    context = context_acquire((*sampler_gl).s.device, core::ptr::null_mut(), 0);
    gl_info = (*wined3d_context_gl(context)).gl_info;

    desc = core::ptr::addr_of!((*sampler_gl).s.desc);
    ((*gl_info).gl_ops.ext.p_glGenSamplers.unwrap_unchecked())(1 as GLsizei, core::ptr::addr_of_mut!(name));
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_WRAP_S as GLenum,
        (*gl_info).wrap_lookup[((*desc).address_u as u32).wrapping_sub(WINED3D_TADDRESS_WRAP as u32) as usize],
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_WRAP_T as GLenum,
        (*gl_info).wrap_lookup[((*desc).address_v as u32).wrapping_sub(WINED3D_TADDRESS_WRAP as u32) as usize],
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_WRAP_R as GLenum,
        (*gl_info).wrap_lookup[((*desc).address_w as u32).wrapping_sub(WINED3D_TADDRESS_WRAP as u32) as usize],
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameterfv.unwrap_unchecked())(
        name,
        GL_TEXTURE_BORDER_COLOR as GLenum,
        (core::ptr::addr_of!((*desc).border_color) as *const GLfloat).offset(0 as isize),
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_MAG_FILTER as GLenum,
        wined3d_gl_mag_filter((*desc).mag_filter) as GLint,
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_MIN_FILTER as GLenum,
        wined3d_gl_min_mip_filter((*desc).min_filter, (*desc).mip_filter) as GLint,
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameterf.unwrap_unchecked())(
        name,
        GL_TEXTURE_LOD_BIAS as GLenum,
        (*desc).lod_bias as GLfloat,
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameterf.unwrap_unchecked())(
        name,
        GL_TEXTURE_MIN_LOD as GLenum,
        (*desc).min_lod as GLfloat,
    );
    ((*gl_info).gl_ops.ext.p_glSamplerParameterf.unwrap_unchecked())(
        name,
        GL_TEXTURE_MAX_LOD as GLenum,
        (*desc).max_lod as GLfloat,
    );
    if (*gl_info).supported[ARB_TEXTURE_FILTER_ANISOTROPIC as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
            name,
            GL_TEXTURE_MAX_ANISOTROPY as GLenum,
            (*desc).max_anisotropy as GLint,
        );
    }
    if (*desc).compare != 0 {
        ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
            name,
            GL_TEXTURE_COMPARE_MODE as GLenum,
            GL_COMPARE_R_TO_TEXTURE as GLint,
        );
    }
    ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
        name,
        GL_TEXTURE_COMPARE_FUNC as GLenum,
        wined3d_gl_compare_func((*desc).comparison_func) as GLint,
    );
    if ((*(*context).d3d_info).wined3d_creation_flags & (WINED3D_SRGB_READ_WRITE_CONTROL as u32)) != 0
        && (*gl_info).supported[EXT_TEXTURE_SRGB_DECODE as usize] != 0
        && (*desc).srgb_decode == 0
    {
        ((*gl_info).gl_ops.ext.p_glSamplerParameteri.unwrap_unchecked())(
            name,
            GL_TEXTURE_SRGB_DECODE_EXT as GLenum,
            GL_SKIP_DECODE_EXT as GLint,
        );
    }
    checkGLcall!(gl_info, "sampler creation");

    TRACE!("Created sampler %u.\n", name);

    (*sampler_gl).name = name;

    context_release(context);
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_gl_init(mut sampler_gl: *mut wined3d_sampler_gl, mut device: *mut wined3d_device, mut desc: *const wined3d_sampler_desc, mut parent: *mut ::core::ffi::c_void, mut parent_ops: *const wined3d_parent_ops) {
    WINE_TRACE!("sampler_gl %p, device %p, desc %p, parent %p, parent_ops %p.\n", sampler_gl, device, desc, parent, parent_ops);
    wined3d_sampler_init(&raw mut (*sampler_gl).s, device, desc, parent, parent_ops);
    if (*wined3d_adapter_gl((*device).adapter)).gl_info.supported
        [ARB_SAMPLER_OBJECTS as usize]
        != 0
    {
        wined3d_cs_init_object(
            (*device).cs as *mut wined3d_cs,
            Some(
                wined3d_sampler_gl_cs_init as unsafe extern "C" fn(*mut ::core::ffi::c_void) -> (),
            ),
            sampler_gl as *mut ::core::ffi::c_void,
        );
    }
}

unsafe extern "C" fn vk_filter_from_wined3d(f: wined3d_texture_filter_type) -> VkFilter {
    if f as ::core::ffi::c_uint == WINED3D_TEXF_POINT {
        // case COMMENT_TOKEN_ECHO____: falls through to the shared return below
    } else if f as ::core::ffi::c_uint == WINED3D_TEXF_LINEAR {
        return VK_FILTER_LINEAR;
    } else {
        ERR!("Invalid filter type %#x.\n", f);
        // default: falls through to the shared return below
    }
    return VK_FILTER_NEAREST;
}

unsafe extern "C" fn vk_mipmap_mode_from_wined3d(f: wined3d_texture_filter_type) -> VkSamplerMipmapMode {
    match f as ::core::ffi::c_uint {
        WINED3D_TEXF_NONE | WINED3D_TEXF_POINT => {
            return VK_SAMPLER_MIPMAP_MODE_NEAREST;
        }
        WINED3D_TEXF_LINEAR => {
            return VK_SAMPLER_MIPMAP_MODE_LINEAR;
        }
        _ => {
            ERR!("Invalid filter type %#x.\n", f);
            return VK_SAMPLER_MIPMAP_MODE_NEAREST;
        }
    }
}

unsafe extern "C" fn vk_address_mode_from_wined3d(mut a: wined3d_texture_address) -> VkSamplerAddressMode {
    match a as ::core::ffi::c_uint {
        WINED3D_TADDRESS_WRAP => return VK_SAMPLER_ADDRESS_MODE_REPEAT,
        WINED3D_TADDRESS_MIRROR => return VK_SAMPLER_ADDRESS_MODE_MIRRORED_REPEAT,
        WINED3D_TADDRESS_CLAMP => return VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        WINED3D_TADDRESS_BORDER => return VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_BORDER,
        WINED3D_TADDRESS_MIRROR_ONCE => return VK_SAMPLER_ADDRESS_MODE_MIRROR_CLAMP_TO_EDGE,
        _ => {
            ERR!("Invalid address mode %#x.\n", a as ::core::ffi::c_uint);
            return VK_SAMPLER_ADDRESS_MODE_REPEAT;
        }
    }
}

unsafe extern "C" fn vk_border_colour_from_wined3d(colour: *const wined3d_color) -> VkBorderColor {
    let mut i: ::core::ffi::c_uint = 0;
    static mut colours: [C2RustUnnamed_58; 3] = [
        C2RustUnnamed_58 {
            wined3d_colour: wined3d_color {
                r: 0.0f32,
                g: 0.0f32,
                b: 0.0f32,
                a: 0.0f32,
            },
            vk_colour: VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK,
        },
        C2RustUnnamed_58 {
            wined3d_colour: wined3d_color {
                r: 0.0f32,
                g: 0.0f32,
                b: 0.0f32,
                a: 1.0f32,
            },
            vk_colour: VK_BORDER_COLOR_FLOAT_OPAQUE_BLACK,
        },
        C2RustUnnamed_58 {
            wined3d_colour: wined3d_color {
                r: 1.0f32,
                g: 1.0f32,
                b: 1.0f32,
                a: 1.0f32,
            },
            vk_colour: VK_BORDER_COLOR_FLOAT_OPAQUE_WHITE,
        },
    ];
    i = 0 as ::core::ffi::c_uint;
    while (i as usize) < ARRAY_SIZE!(colours) as usize {
        if memcmp(
            core::ptr::addr_of!(colours[i as usize]) as *const ::core::ffi::c_void,
            colour as *const ::core::ffi::c_void,
            ::core::mem::size_of::<wined3d_color>() as size_t,
        ) == 0
        {
            return colours[i as usize].vk_colour;
        }
        i = i.wrapping_add(1);
    }

    return VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK;
}

unsafe extern "C" fn wined3d_sampler_vk_cs_init(object: *mut core::ffi::c_void) {
    let sampler_vk: *mut wined3d_sampler_vk = object as *mut wined3d_sampler_vk;
    let mut desc: *const wined3d_sampler_desc = core::ptr::null::<wined3d_sampler_desc>();
    let mut d3d_info: *const wined3d_d3d_info = core::ptr::null::<wined3d_d3d_info>();
    let mut sampler_desc: VkSamplerCreateInfo = core::mem::zeroed();
    let mut vk_info: *const wined3d_vk_info = core::ptr::null::<wined3d_vk_info>();
    let mut context_vk: *mut wined3d_context_vk = core::ptr::null_mut::<wined3d_context_vk>();
    let mut device_vk: *mut wined3d_device_vk = core::ptr::null_mut::<wined3d_device_vk>();
    let mut vk_sampler: VkSampler = 0;
    let mut vr: VkResult = 0;

    TRACE!(
        "sampler_vk %p.\n",
        sampler_vk
    );

    context_vk = wined3d_context_vk(context_acquire(
        (*sampler_vk).s.device,
        core::ptr::null_mut(),
        0,
    ));
    device_vk = wined3d_device_vk((*context_vk).c.device);
    d3d_info = (*context_vk).c.d3d_info;
    vk_info = (*context_vk).vk_info;

    desc = core::ptr::addr_of!((*sampler_vk).s.desc);
    sampler_desc.sType = VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO;
    sampler_desc.pNext = core::ptr::null::<core::ffi::c_void>();
    sampler_desc.flags = 0;
    sampler_desc.magFilter = vk_filter_from_wined3d((*desc).mag_filter);
    sampler_desc.minFilter = vk_filter_from_wined3d((*desc).min_filter);
    sampler_desc.mipmapMode = vk_mipmap_mode_from_wined3d((*desc).mip_filter);
    sampler_desc.addressModeU = vk_address_mode_from_wined3d((*desc).address_u);
    sampler_desc.addressModeV = vk_address_mode_from_wined3d((*desc).address_v);
    sampler_desc.addressModeW = vk_address_mode_from_wined3d((*desc).address_w);
    sampler_desc.mipLodBias = (*desc).lod_bias;
    sampler_desc.anisotropyEnable =
        ((*desc).max_anisotropy != 1) as i32 as VkBool32;
    sampler_desc.maxAnisotropy = (*desc).max_anisotropy as f32;
    sampler_desc.compareEnable = ((*desc).compare != 0) as i32 as VkBool32;
    sampler_desc.compareOp = vk_compare_op_from_wined3d((*desc).comparison_func);
    sampler_desc.minLod = (*desc).min_lod;
    sampler_desc.maxLod = (*desc).max_lod;
    sampler_desc.borderColor = VK_BORDER_COLOR_FLOAT_TRANSPARENT_BLACK;
    sampler_desc.unnormalizedCoordinates = VK_FALSE as VkBool32;

    if (*desc).address_u == WINED3D_TADDRESS_BORDER
        || (*desc).address_v == WINED3D_TADDRESS_BORDER
        || (*desc).address_w == WINED3D_TADDRESS_BORDER
    {
        sampler_desc.borderColor = vk_border_colour_from_wined3d(
            core::ptr::addr_of!((*desc).border_color) as *const wined3d_color,
        );
    }
    if (*desc).mip_base_level != 0 {
        FIXME!(
            "Unhandled mip_base_level %u.\n",
            (*desc).mip_base_level
        );
    }
    if ((*d3d_info).wined3d_creation_flags & WINED3D_SRGB_READ_WRITE_CONTROL) != 0
        && (*desc).srgb_decode == 0
    {
        FIXME!(
            "Unhandled srgb_decode %#x.\n",
            (*desc).srgb_decode
        );
    }

    vr = ((*vk_info).vk_ops.vkCreateSampler.unwrap_unchecked())(
        (*device_vk).vk_device,
        core::ptr::addr_of_mut!(sampler_desc),
        core::ptr::null::<VkAllocationCallbacks>(),
        core::ptr::addr_of_mut!(vk_sampler),
    );
    context_release(core::ptr::addr_of_mut!((*context_vk).c));
    if vr < 0 {
        ERR!(
            "Failed to create Vulkan sampler, vr %s.\n",
            wined3d_debug_vkresult(vr)
        );
        return;
    }

    TRACE!(
        "Created sampler 0x%s.\n",
        wine_dbgstr_longlong(vk_sampler as ULONGLONG)
    );

    (*sampler_vk).vk_image_info.sampler = vk_sampler;
    (*sampler_vk).vk_image_info.imageView = (VK_NULL_HANDLE as _);
    (*sampler_vk).vk_image_info.imageLayout = VK_IMAGE_LAYOUT_UNDEFINED;
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_vk_init(sampler_vk: *mut wined3d_sampler_vk, device: *mut wined3d_device, desc: *const wined3d_sampler_desc, parent: *mut ::core::ffi::c_void, parent_ops: *const wined3d_parent_ops) {
    TRACE!(
        "sampler_vk %p, device %p, desc %p, parent %p, parent_ops %p.\n",
        sampler_vk,
        device,
        desc,
        parent,
        parent_ops
    );

    wined3d_sampler_init(
        core::ptr::addr_of_mut!((*sampler_vk).s),
        device,
        desc,
        parent,
        parent_ops,
    );
    wined3d_cs_init_object(
        (*device).cs as *mut wined3d_cs,
        Some(wined3d_sampler_vk_cs_init as unsafe extern "C" fn(*mut core::ffi::c_void) -> ()),
        sampler_vk as *mut core::ffi::c_void,
    );
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_create(
    device: *mut wined3d_device,
    desc: *const wined3d_sampler_desc,
    parent: *mut core::ffi::c_void,
    parent_ops: *const wined3d_parent_ops,
    sampler: *mut *mut wined3d_sampler,
) -> HRESULT {
    TRACE!(
        "device %p, desc %p, parent %p, parent_ops %p, sampler %p.\n",
        device,
        desc,
        parent,
        parent_ops,
        sampler
    );

    if ((*desc).address_u as u32) < (WINED3D_TADDRESS_WRAP as u32)
        || ((*desc).address_u as u32) > (WINED3D_TADDRESS_MIRROR_ONCE as u32)
        || ((*desc).address_v as u32) < (WINED3D_TADDRESS_WRAP as u32)
        || ((*desc).address_v as u32) > (WINED3D_TADDRESS_MIRROR_ONCE as u32)
        || ((*desc).address_w as u32) < (WINED3D_TADDRESS_WRAP as u32)
        || ((*desc).address_w as u32) > (WINED3D_TADDRESS_MIRROR_ONCE as u32)
    {
        return WINED3DERR_INVALIDCALL;
    }

    if ((*desc).mag_filter as u32) < (WINED3D_TEXF_POINT as u32)
        || ((*desc).mag_filter as u32) > (WINED3D_TEXF_LINEAR as u32)
        || ((*desc).min_filter as u32) < (WINED3D_TEXF_POINT as u32)
        || ((*desc).min_filter as u32) > (WINED3D_TEXF_LINEAR as u32)
        || ((*desc).mip_filter as u32) > (WINED3D_TEXF_LINEAR as u32)
    {
        return WINED3DERR_INVALIDCALL;
    }

    return ((*(*(*device).adapter).adapter_ops)
        .adapter_create_sampler
        .unwrap_unchecked())(device, desc, parent, parent_ops, sampler);
}

unsafe extern "C" fn texture_gl_apply_base_level(
    mut texture_gl: *mut wined3d_texture_gl,
    mut desc: *const wined3d_sampler_desc,
    mut gl_info: *const wined3d_gl_info,
) {
    let mut gl_tex: *mut gl_texture = ::core::ptr::null_mut::<gl_texture>();
    gl_tex = wined3d_texture_gl_get_gl_texture(
        texture_gl,
        ((*texture_gl).t.flags & WINED3D_TEXTURE_IS_SRGB) != 0,
    );
    if (*desc).mip_base_level != (*gl_tex).sampler_desc.mip_base_level {
        (*gl_info)
            .gl_ops
            .gl
            .p_glTexParameteri
            .unwrap_unchecked()(
            (*texture_gl).target,
            GL_TEXTURE_BASE_LEVEL as GLenum,
            (*desc).mip_base_level as GLint,
        );
        (*gl_tex).sampler_desc.mip_base_level = (*desc).mip_base_level;
    }
}
#[no_mangle]
pub unsafe extern "C" fn wined3d_sampler_gl_bind(sampler_gl: *mut wined3d_sampler_gl, unit: ::core::ffi::c_uint, texture_gl: *mut wined3d_texture_gl, context_gl: *const wined3d_context_gl) {
    let gl_info: *const wined3d_gl_info = (*context_gl).gl_info;

    if (*gl_info).supported[ARB_SAMPLER_OBJECTS as usize] != 0 {
        ((*gl_info).gl_ops.ext.p_glBindSampler.unwrap_unchecked())(unit as GLuint, (*sampler_gl).name);
        checkGLcall!(gl_info, b"bind sampler\0".as_ptr() as *const core::ffi::c_char);
    } else if !texture_gl.is_null() {
        wined3d_texture_gl_apply_sampler_desc(texture_gl, core::ptr::addr_of_mut!((*sampler_gl).s.desc), context_gl);
    } else {
        ERR!("Could not apply sampler state.\n");
    }

    if !texture_gl.is_null() {
        texture_gl_apply_base_level(texture_gl, core::ptr::addr_of_mut!((*sampler_gl).s.desc), gl_info);
    }
}
